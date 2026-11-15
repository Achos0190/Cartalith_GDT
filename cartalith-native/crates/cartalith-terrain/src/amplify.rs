//! Regional amplification — `UNIFIED_TOOL_PLAN.md` milestone E, the compute
//! half of the Region select/export tool.
//!
//! Ports `amplifyRegion` (reference line 10265) and `refineTile` (10305), the
//! two pure functions behind both of the reference's region operations: the
//! tiled `Refine & export` (`exportRegionTiles` → `refineTile`) and
//! `regionNewWorldBtn`'s "replace the world with a higher-resolution version
//! of this region" (`amplifyRegion` directly). The reference's own header
//! calls them *"Pure function: takes src explicitly (no globals) so it is
//! worker-ready"* and *"All headless-testable"* — which is exactly why they
//! port cleanly while the `.zip`/PNG assembly around them does not.
//!
//! **Why `cartalith-terrain`.** Milestone B's third placement category:
//! subsystem-domain math belongs to the crate that owns the field. This is an
//! upsample of a height field plus world-space fBm detail tapered by local
//! relief and faded out underwater — height formula, start to finish, over the
//! same `fbm`/`ridged` this crate already generates terrain with.
//! `cartalith-engine` would be wrong for milestone B's reason
//! (*"cartalith-engine orchestrates; it does not compute"*), and
//! `cartalith-spatial` for milestone C's (a sea-level-aware relief taper is
//! not generic machinery). The *selection* rectangle it consumes is generic
//! and does live in `cartalith-spatial` ([`FloatRegion`]).
//!
//! # The seam property, and why the region is fractional
//!
//! Both the upsample and the detail are pure functions of the shared **coarse**
//! coordinate `(cx, cy)`, so two tiles that overlap by exactly one coarse
//! column agree bit-for-bit along that shared edge. `refine_tile` reproduces
//! that overlap (`w: stepX + 1`), and `stepX = region.w / cols` is generally
//! **not** an integer — rounding the sub-bounds to whole cells would break the
//! agreement outright, which is why [`FloatRegion`] exists.
//!
//! # A real division by zero, ported rather than fixed
//!
//! `cy = rh > 1 ? ry + (oy/(outH-1))*(rh-1) : ry` divides by `outH - 1`. With
//! `outH == 1` **and** `rh > 1` that is `0/0`, and the entire output comes back
//! `NaN` — verified against the reference, not inferred. No shipped caller
//! reaches it, because `tile_dims` floors both edges at 2px and
//! `regionNewWorldBtn` goes through `tile_dims` too. Ported as written
//! (`DECISIONS.md`'s rule: the reference's behaviour is the specification), and
//! pinned by a golden case so nobody later "fixes" the port into disagreeing.
//!
//! # One deliberate departure from the reference lives in this file
//!
//! Everything else here is a literal port. [`clamp_toward_sea`] is not: it caps
//! a detail excursion *toward sea level* at half the remaining headroom, in
//! [`amplify_region`], [`add_zoom_detail`] and [`sample_elevation`] alike.
//! That is the HTML's own **v2.69** fix (`RC_ENGINE_CHANGES.md` §8.1), landed
//! here under **Ruling O** (`LARGE_ITEM_RULINGS.md`, owner, 2026-09-21)
//! against `OUTSTANDING_WORK.md`'s v2.69 row. It re-based goldens in
//! `tests/golden_parity_amplify.rs`, `tests/golden_parity_zoom_detail.rs` and
//! `cartalith-engine`'s `golden_parity_bake.rs` /
//! `golden_parity_region_export.rs`; each of those files records what moved
//! and by how much. Read [`clamp_toward_sea`] before changing anything near
//! the shelf.

use cartalith_spatial::FloatRegion;
use rayon::prelude::*;

use crate::sculpt::js_hypot;

/// `amplifyRegion`'s `opts` bag, with the reference's own defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmplifyOpts {
    /// `opts.seed`, `|0`-truncated by the reference. Default 1234.
    pub seed: i32,
    /// Noise cycles per coarse cell at octave 0. Default 1.0.
    pub detail_freq: f64,
    /// Maximum added relief in normalised height. Default 0.14.
    pub detail_amp: f64,
    /// Sea level, below which detail is faded out. Default 0.42.
    pub sea: f64,
    /// Use `ridged` instead of `fbm` for the detail term. Default false.
    pub ridged: bool,
    /// `opts.zBase` — the pyramid level at which [`add_zoom_detail`] starts
    /// adding octaves. Default 2, the reference's own. Read by
    /// [`add_zoom_detail`] only; [`amplify_region`] ignores it, exactly as the
    /// reference's single shared `opts` bag does.
    pub z_base: i32,
    /// `opts.zoomDetailK` — the user's "zoom detail" amount. Default 1, which
    /// the reference documents as *"legacy, bit-identical"*.
    pub zoom_detail_k: f64,
    /// **Ruling CI item 1** (owner, 2026-10-17): the kilometre-keyed zoom
    /// detail law, which replaces [`add_zoom_detail`]'s per-coarse-cell
    /// octave schedule when `Some`. **`None` — the default — is the
    /// reference's own schedule by control flow**: the per-pixel body never
    /// reads this field on that branch, so every golden that builds its opts
    /// from [`AmplifyOpts::default`] is untouched. Only the app's on-screen
    /// deep-zoom tiles set it (`cartalith-godot`'s `render::tile_amplify_opts`,
    /// gated on `TerrainAppearance::zoom_detail_km`). See [`KmZoomDetail`].
    pub km: Option<KmZoomDetail>,
    /// The upper bound of the refined height. **1.0 -- the default -- is the
    /// reference's own `v>1?1:v`**. The app passes
    /// [`crate::orogen::field_top`] of the world's coarse field, which is
    /// exactly 1.0 unless the world was generated under owner Ruling DB item
    /// 3's higher ceiling (`TectonicParams::high_ceiling`), so a summit above
    /// 1 keeps its relief and its detail at depth instead of a flat cap at 1.
    pub top: f64,
}

impl Default for AmplifyOpts {
    /// The reference's own hard-coded defaults for `amplifyRegion`/`refineTile`
    /// (reference lines 10265/10305) — pinned by
    /// `the_defaults_are_the_references_own` below. `km: None` is the
    /// reference's octave schedule (Ruling CI's law is app-side only).
    fn default() -> Self {
        AmplifyOpts {
            seed: 1234,
            detail_freq: 1.0,
            detail_amp: 0.14,
            sea: 0.42,
            ridged: false,
            z_base: 2,
            zoom_detail_k: 1.0,
            km: None,
            // The reference's clip at 1 (Ruling DB's higher top is app-side).
            top: 1.0,
        }
    }
}

/// The cell size the kilometre law is **anchored** at: `800 km / 2048` cells,
/// the New World dialog's default width over the app's default grid.
///
/// Source: the HTML project's own `terrainDetailK` (`docs/research/
/// scale-invariant-terrain.md` §3, Stage B: *"`REF_CELLKM = 800/2048` — the
/// app's own literal untouched default"*), which re-keyed the generator's
/// relief frequency to kilometres by the same anchor-at-the-default rule. At
/// this cell size [`KmZoomDetail`]'s octave lattice lands exactly on the
/// reference's per-cell octaves (wavelength `2^-(o+1)` cells) and its relief
/// taper is the reference's own, so the two laws differ there only in the
/// octave decay and the noise basis — the parts Ruling CI changes on purpose.
pub const ZOOM_DETAIL_REF_CELL_KM: f64 = 800.0 / 2048.0;

/// **Ruling CI item 1, re-based by Ruling FG** (owner, 2026-11-15): zoom
/// detail keyed to **ground distance**, not to the coarse grid, and sized by
/// **the coarse field's own local relief** -- the law [`add_zoom_detail`] runs
/// when [`AmplifyOpts::km`] is `Some`.
///
/// # Why
///
/// Ruling CI (2026-10-17) re-keyed the reference's per-coarse-cell octave
/// schedule to kilometres, because `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md`
/// §4 measured detail per screen pixel falling at every zoom step past x4.
/// Its amplitude, though, was `A0 · 0.5^j` in **normalised height**: the top
/// octave sits just under one coarse cell, so the synthetic slope per cell grew
/// linearly with kilometres per cell, while its relief taper's slope term was
/// multiplied by `1.5625 km / L` and faded out as the cells grew.
/// `WORLD_EMBOSS_FINDINGS_2026-11-15.md` §2 measured the result: on a 40 075 km
/// world the detail was ~10⁴ times the terrain's own per-cell slope, uniform
/// over every biome, flat-topped by the two clamps and snow-capped by the tile
/// lapse -- the owner's *"40k km becomes more uniform and stays it no matter
/// the zoom"*. Ruling FG keeps the kilometre lattice and replaces the
/// amplitude and the taper (F1 of that file).
///
/// # The spectral model, and what it is sourced from
///
/// Fractional Brownian motion as a sum of octaves of band-limited noise
/// (Mandelbrot & Van Ness 1968; the terrain-synthesis practice of Musgrave,
/// Kolb & Mace 1989, which `docs/research/scale-invariant-terrain.md` cites
/// for pegging octaves to real wavelengths). Octave `j` has wavelength and
/// amplitude
///
/// ```text
/// λ_j = (ZOOM_DETAIL_REF_CELL_KM / 2) · 2^-j  km,   j ∈ ℤ
/// a_j = strength · relief_gain · ρ(x) · R(x) · (λ_j / L)^H        (H = −log2 octave_gain)
/// ```
///
/// on a lattice fixed **in kilometres**, so one place on the ground gets the
/// same octaves whatever grid it was generated on. `L` is kilometres per
/// coarse cell, so `λ_j / L` is the octave's wavelength **in cells**, and
/// `a_j / (λ_j / L)` -- the octave's slope per coarse cell -- is
/// `strength · relief_gain · ρ · R` at `H = 1`: **proportional to the
/// terrain's own per-cell relief and independent of `L`**. A world and a
/// region laid out with the same normalised terrain in cells therefore get the
/// same relative roughness, which is bar B1 of the findings. `H = 1`
/// (`octave_gain = 0.5`) is kept from Ruling CI: every octave adds the same
/// slope amplitude, so the hillshade -- which reads slope per coarse cell
/// (`render::tile_shade_exag`) -- gains detail at the same rate at every
/// level, the owner's "zoom always gains detail" requirement stated as a
/// spectrum ([`ZOOM_DETAIL_KM_OCTAVE_GAIN`]; a labelled judgement, not a
/// measured property of real terrain -- published `H` for topography clusters
/// between about 0.5 and 0.9 (Turcotte, *Fractals and Chaos in Geology and
/// Geophysics*, 1997, cited from memory)).
///
/// `R(x)` ([`km_local_relief`]) is the coarse field's own relief at one cell:
/// the hypotenuse of its gradient per cell and its four-neighbour curvature, so
/// a ridge crest or a valley floor (zero gradient, high curvature) counts as
/// relief too. It is floored at [`KmZoomDetail::relief_floor`] so flat ground
/// keeps a faint grain. `ρ(x)` is the caller's biome and rock roughness
/// (`cartalith-godot`'s `render::detail_roughness`), passed per pixel through
/// [`Roughness`]; `None` -- every caller in this crate and the engine -- is
/// `ρ = 1`. Because the floor is multiplied by `ρ` like everything else,
/// smooth biomes (wetland, sand) keep a smaller floor than forest or rock:
/// the findings' *"a small floor only on non-flat biomes"*.
///
/// # Which octaves a pixel gets
///
/// Two smooth cut-offs in `log2` wavelength, so nothing switches on at a
/// threshold:
///
/// - **Top**, `L/2 ..= L`: weight `1` at half a cell, `0` at one cell.
///   Wavelengths the grid itself carries are left to the grid; this is the
///   reference's own first octave (half a cell).
/// - **Bottom**, `λ_min/2 ..= λ_min` with `λ_min = L · 2^(z_base − z) ·
///   2^nyquist_margin`. `L · 2^(z_base − z)` is the finest octave the
///   reference's schedule admits at level `z`; the shipped margin of one
///   octave keeps the finest full-weight octave at about eight tile texels and
///   fades the one below it ([`ZOOM_DETAIL_KM_NYQUIST_MARGIN`]).
///
/// [`KmZoomDetail::max_octaves`] bounds the count. It is a cost cap, not a
/// look: since Ruling FG's F2 lets a coarse-celled map's pyramid go deeper
/// than level 10 (`cartalith-godot`'s `lod_bridge::max_level_for_map`), the cap
/// must cover every octave between one cell and the ground floor, or zoom
/// would stop adding detail at the cap instead of at the floor.
///
/// # The excursion cap, in metres
///
/// The summed excursion is soft-limited ([`soft_cap`], a `tanh`) to
/// [`KmZoomDetail::cap`] normalised units -- the caller sets it to
/// [`ZOOM_DETAIL_KM_CAP_FRAC`] of the land's height range. A
/// smooth limit, not a clip, so a peak that reaches it rounds over instead of
/// going flat-topped, and the tile lapse (`render::TileCryo::delta_t`) and
/// `render::micro_relief_tone` never see a kilometre-scale noise peak.
/// `f64::INFINITY` (the default) is no cap.
///
/// # The shore fade
///
/// Kept from Ruling CI: the law fades to zero over
/// [`ZOOM_DETAIL_KM_SHORE_BAND`] above sea level, so beaches and coastal
/// flats stay smooth and their colour bands do not flicker.
///
/// # What was removed, and why
///
/// Ruling CI's taper, `max(0.05, min(1, 8·|∇h|·1.5625/L), 0.3·land)`: its
/// slope term faded with `L`, and its elevation term made every high plateau
/// equally rough whatever its relief. `R` replaces both (F1 of the findings;
/// `DECISIONS.md` §7p, the Ruling FG entry).
///
/// # Cost
///
/// One `vnoise` per octave, plus one more bilinear sample of the coarse field
/// for the curvature, plus whatever [`Roughness`] the caller passes.
///
/// # What this must never do
///
/// - Run on the parity path: [`AmplifyOpts::default`] has `km: None`, and so
///   does every opts the engine's bake, export and EF-0 point query builds.
/// - Move the coastline: the write-back keeps [`clamp_toward_sea`] and the
///   hard `base < sea` skip, exactly as the reference branch does.
/// - Depend on anything but `(cx, cy, z)`, the frozen coarse field and a
///   roughness that is itself a function of `(cx, cy)` -- which is what keeps
///   two tiles seam-exact (`sample_elevation` asserts it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KmZoomDetail {
    /// Kilometres per coarse cell (`cartalith_spatial::cell_km`). Must be
    /// finite and positive; anything else makes the law add nothing.
    pub km_per_cell: f64,
    /// Amplitude ratio between octave `j + 1` and octave `j` -- `2^-H`.
    /// Shipped at [`ZOOM_DETAIL_KM_OCTAVE_GAIN`].
    pub octave_gain: f64,
    /// Multiplies the whole law (`TerrainAppearance::zoom_detail_km`). The app
    /// ships `2.0`; `0.0` adds nothing.
    pub strength: f64,
    /// Slope per coarse cell each octave adds, per unit of [`km_local_relief`]
    /// and of `strength`. Shipped at [`ZOOM_DETAIL_KM_RELIEF_GAIN`].
    pub relief_gain: f64,
    /// The floor under [`km_local_relief`], in normalised height per coarse
    /// cell. Shipped at [`ZOOM_DETAIL_KM_RELIEF_FLOOR`].
    pub relief_floor: f64,
    /// The cost cap: at most this many octaves per pixel. Shipped at
    /// [`ZOOM_DETAIL_KM_MAX_OCTAVES`].
    pub max_octaves: u32,
    /// How many octaves **above** the reference's finest wavelength the
    /// bottom fade sits: the fade runs over `[λ_min·2^(m−1), λ_min·2^m]`.
    /// Shipped at [`ZOOM_DETAIL_KM_NYQUIST_MARGIN`].
    pub nyquist_margin: f64,
    /// The soft limit on the summed excursion, in normalised height (see
    /// "The excursion cap" above). `f64::INFINITY` is none.
    pub cap: f64,
}

/// [`KmZoomDetail::octave_gain`]'s shipped value -- `H = 1`, the self-similar
/// spectrum (see [`KmZoomDetail`], "The spectral model": a labelled
/// judgement, chosen because it makes every level add the same slope).
pub const ZOOM_DETAIL_KM_OCTAVE_GAIN: f64 = 0.5;

/// [`KmZoomDetail::relief_gain`]'s shipped value. **Measured: set so the
/// 800 km / 2048 anchor keeps Ruling CI's shipped look** (bar B6 of
/// `WORLD_EMBOSS_FINDINGS_2026-11-15.md`), on the Rust zoom ladder
/// (`cartalith-godot`'s `tests/zoom_detail_ladder.rs`, `ZD_LAWS=2`, the three
/// 800 km worlds, 29 crops; median mean-luminance-gradient after / before).
/// With the floor at 0.0017: gain 5 → x16 0.925, x64 0.827; 7 → 0.965 /
/// 0.891; 9 → 0.997 / 0.953; 11 → 1.000 / 1.005. With the shipped floor
/// ([`ZOOM_DETAIL_KM_RELIEF_FLOOR`]) at 11: x4 1.000, x16 1.000, x64 1.004.
/// The run, frames and the 3 000 km world's numbers are in `DECISIONS.md`
/// §7p's Ruling FG entry.
pub const ZOOM_DETAIL_KM_RELIEF_GAIN: f64 = 11.0;

/// [`KmZoomDetail::relief_floor`]'s shipped value, in normalised height per
/// coarse cell. **A labelled judgement** with a measured motive: Ruling CI's
/// floor (`0.05` of a full taper) was chosen so a flat forest crop does not go
/// to glass at x64 (its doc: `0.0` fell 1.31 → 0.45, `0.05` held 1.31 → 1.21).
/// This floor gives flat **forest** the octave slope the old floor gave all
/// flat ground at the anchor and the shipped strength: `0.168 · 2 · 0.05 /
/// (2 · relief_gain · ρ_forest)` = `0.0168 / (22 · 0.5)` ≈ `0.00153`, with
/// `ρ_forest` = 0.5 (`cartalith-godot`'s `render::DETAIL_RHO_FOREST`), so
/// smoother biomes keep proportionally less. Measured on the B6 ladder: at
/// `0.00076` the 800 km flat and coast crops at x64 held 0.79-1.00 of before;
/// at this value 0.82-1.01, and the all-crop medians 1.000 / 1.000 / 1.004.
pub const ZOOM_DETAIL_KM_RELIEF_FLOOR: f64 = 0.00153;

/// [`KmZoomDetail::max_octaves`]'s shipped value. `14`: Ruling FG's deepest
/// pyramid level on the coarsest supported map (a 40 075 km world on 1 024
/// cells, `lod_bridge::LEVEL_CEILING = 16`) spans `z − z_base − 1 = 11`
/// full-weight octaves between one cell and the ground floor, plus the two
/// faded ends -- 13 -- and one spare. On a default 800 km map at level 10 the
/// window holds about six octaves, so the cap never binds there and the cost
/// is Ruling CI's.
pub const ZOOM_DETAIL_KM_MAX_OCTAVES: u32 = 14;

/// [`KmZoomDetail::nyquist_margin`]'s shipped value: `1`. **A labelled
/// judgement** from the Ruling CI frames: at `0` (the reference's own floor)
/// the finest octave read as a pixel-scale grain over every surface -- the
/// law's slopes are equal per octave, so the finest one is as strong as any
/// and the eye takes it for noise.
pub const ZOOM_DETAIL_KM_NYQUIST_MARGIN: f64 = 1.0;

/// The soft cap on the law's summed excursion, as a **fraction of the land's
/// height range** (`1 − sea` in normalised height, i.e. sea level to the
/// world's peak altitude): the findings' *"≤ peak_m/10"* exactly, since the
/// height unit is `peak_m / (1 − sea)` metres -- 400 m at the app's default
/// 4 000 m peak. A labelled judgement: large enough that the anchor's ladder
/// crops are not reshaped by it (bar B6 is the check), small enough that a
/// sub-cell bump can never be a mountain, so the tile lapse cannot whiten it
/// (bar B7). `cartalith-godot`'s `render::zoom_detail_km_law` sets
/// [`KmZoomDetail::cap`] from it and the world's sea level.
pub const ZOOM_DETAIL_KM_CAP_FRAC: f64 = 0.1;

/// Per-octave rotation of the noise lattice, in radians: `2π / φ²`
/// (≈ 137.5°, the golden angle). Value noise is built on an axis-aligned
/// lattice, and stacking octaves on aligned lattices reinforces the
/// axis-aligned squares the gap assessment already reports elsewhere
/// (`render.rs::bio_jitter`'s mottle, §5). Rotating each octave by an angle
/// whose multiples never repeat keeps the lattices from lining up. A labelled
/// judgement; the golden angle is the standard choice for "never aligns".
const KM_OCTAVE_ROTATION: f64 = 2.399_963_229_728_653;

/// The caller's biome and rock roughness `ρ` at a coarse coordinate `(cx,
/// cy)` (see [`KmZoomDetail`]): a pure function of the coordinate, so a tile
/// and its neighbour agree on their shared edge. `Sync` because the tile
/// bodies are row-parallel.
pub type Roughness<'a> = &'a (dyn Fn(f64, f64) -> f64 + Sync);

/// `GLSL smoothstep(0, 1, t)` on a clamped argument.
#[inline]
fn smooth01(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// [`KmZoomDetail`]'s per-pixel octave **shape** at coarse coordinate
/// `(cx, cy)` for pyramid level `z`: `Σ_j (n_j − ½) · w_j · (λ_j / L)^H`, a
/// unitless sum whose every octave has unit slope per coarse cell at `H = 1`.
/// [`km_detail_delta`] multiplies it by the amplitude.
///
/// Shared by [`add_zoom_detail_km_padded`] and [`sample_elevation_rough`] so
/// the point query and the tile cannot disagree (the EF-0 identity test pins
/// it). Returns `0.0` for a non-finite or non-positive cell size and for a
/// level with no octave inside both cut-offs.
pub fn km_zoom_detail_sum(cx: f64, cy: f64, z: i32, opts: &AmplifyOpts, km: &KmZoomDetail) -> f64 {
    let l = km.km_per_cell;
    if !(l.is_finite() && l > 0.0) || km.max_octaves == 0 {
        return 0.0;
    }
    // Ground position, km. The lattice is fixed in km, so this is the only
    // place the grid's own resolution enters the noise.
    let (gx, gy) = (cx * l, cy * l);
    let lam_ref = ZOOM_DETAIL_REF_CELL_KM * 0.5;
    // The bottom fade's full-weight wavelength: the reference's finest octave
    // (`2^(z_base − z)` cells), raised by the margin.
    let lam_min = l * f64::powi(2.0, opts.z_base - z) * km.nyquist_margin.exp2();
    // Lattice indices whose wavelength lies strictly inside (λ_min/2, L):
    // λ_j < L  ⇔ j > log2(λ_ref/L);  λ_j > λ_min/2 ⇔ j < log2(2·λ_ref/λ_min).
    let j_lo = (lam_ref / l).log2().floor() as i32 + 1;
    let j_hi = (2.0 * lam_ref / lam_min).log2().ceil() as i32 - 1;
    if j_hi < j_lo {
        return 0.0;
    }
    let j_hi = j_hi.min(j_lo + km.max_octaves as i32 - 1);
    // `H`, from the gain: `(λ_j/L)^H = gain^(log2(L/λ_j))`.
    let h_exp = -km.octave_gain.log2();
    let mut sum = 0.0;
    for j in j_lo..=j_hi {
        let lam = lam_ref * f64::powi(2.0, -j);
        // Top fade: 1 at λ <= L/2, 0 at λ >= L. Bottom fade: 1 at λ >= λ_min,
        // 0 at λ <= λ_min/2. Both in log2 wavelength.
        let w = smooth01((l / lam).log2()) * smooth01((lam / lam_min).log2() + 1.0);
        if w <= 0.0 {
            continue;
        }
        // `j + 1`, not `j`: octave 0 must not keep the axis-aligned lattice
        // (measured under Ruling CI: it did, and coastal flats showed a
        // square checker).
        let (s, c) = ((j + 1) as f64 * KM_OCTAVE_ROTATION).sin_cos();
        let (u, v) = ((gx * c - gy * s) / lam, (gx * s + gy * c) / lam);
        // A seed per octave, offset from the reference branch's `seed + 1 + o`
        // so the two laws never draw the same lattice by accident.
        let n = cartalith_noise::vnoise(u, v, opts.seed.wrapping_add(7919).wrapping_add(j.wrapping_mul(131))) - 0.5;
        sum += n * (lam / l).powf(h_exp) * w;
    }
    sum
}

/// The height band above sea level, in normalised height, over which the
/// kilometre law fades in from nothing at the shoreline. **A labelled
/// judgement** (Ruling CI), twice the renderer's beach band (`land_color`'s
/// `smoothstep(0.03, 0, r)` on `r = (h − sea)/(1 − sea)`, i.e. `0.0174` at the
/// default sea level of `0.42`). Why: on flat coastal land the bumps crossed
/// the beach band and the slope thresholds, and the Ruling CI frames showed
/// sand and grass flipping pixel by pixel along every low coast.
pub const ZOOM_DETAIL_KM_SHORE_BAND: f64 = 0.035;

/// The coarse field's **local relief** at one cell, for [`KmZoomDetail`]'s
/// amplitude: `hypot(|∇h|, κ)` where `gx`/`gy` are the reference's own
/// half-differences (height per coarse cell) and `lap` is the four-neighbour
/// curvature `(l + r + u + d)/4 − c`. The curvature term is what makes a ridge
/// crest, a summit or a valley floor -- zero gradient, strong curvature --
/// count as relief; weighting it `1` against the gradient is a labelled
/// judgement (a one-cell crest that stands `δ` above its neighbours reads as
/// much relief as a slope of `δ` per cell). Normalised height per cell, so it
/// is the same number for the same terrain in cells whatever the map's width.
#[inline]
pub fn km_local_relief(gx: f64, gy: f64, lap: f64) -> f64 {
    js_hypot(js_hypot(gx, gy), lap)
}

/// A smooth limit: `cap · tanh(x / cap)`, `x` itself for an infinite or
/// non-positive cap. Odd and monotone, so it keeps the excursion's sign and
/// order, and within about a third of the cap it is the identity to 1 %.
#[inline]
pub fn soft_cap(x: f64, cap: f64) -> f64 {
    if cap.is_finite() && cap > 0.0 { cap * (x / cap).tanh() } else { x }
}

/// [`KmZoomDetail`]'s excursion at one pixel, before [`clamp_toward_sea`]:
/// the octave shape `sum` ([`km_zoom_detail_sum`]) times `strength ·
/// relief_gain · rho · max(relief, relief_floor)`, soft-capped at
/// [`KmZoomDetail::cap`], times the shore fade on the refined height `base`.
/// `relief` is [`km_local_relief`]; `rho` the caller's [`Roughness`] (`1.0`
/// when none). Written out in [`KmZoomDetail`]'s doc.
#[inline]
pub fn km_detail_delta(sum: f64, relief: f64, rho: f64, base: f64, sea: f64, km: &KmZoomDetail) -> f64 {
    let amp = km.strength * km.relief_gain * rho * js_max(relief, km.relief_floor);
    let shore = smooth01((base - sea) / ZOOM_DETAIL_KM_SHORE_BAND);
    soft_cap(sum * amp, km.cap) * shore
}

// `Math.min`/`Math.max`, which propagate NaN where Rust's absorb it.
// `amplifyRegion`'s `Math.min(1, hypot(gx,gy)*8)` is reached with NaN inputs by
// the `outW == 1` case above, and `f64::min` there would silently turn an
// all-NaN tile into a plausible-looking one. Moved to `cartalith-jsmath`, which
// also settles the signed-zero disagreement this copy had with
// `cartalith-urban`'s and `cartalith-civ`'s (`JS_SEMANTICS_AUDIT.md` §3.3):
// each of the three was right in one argument order and wrong in the other.
pub(crate) use cartalith_jsmath::{js_max, js_min};

/// **v2.69's sea-level guard, and the one deliberate departure from the frozen
/// v2.11 reference in this module.**
///
/// Caps a detail excursion *toward* sea level at **half the remaining
/// headroom**, and leaves an excursion *away* from it completely alone.
/// `RC_ENGINE_CHANGES.md` §8.1's v2.69 row states the mechanism and why it is
/// that shape rather than the obvious one:
///
/// > **Clamp the DELTA, not the result**: clamping the height to sea level
/// > pins a coastal band to one value and makes a flat shelf, so cap the
/// > excursion TOWARD sea level at half the remaining headroom — no constant,
/// > never reaches the far side, and bit-identical wherever the detail is
/// > under half the headroom.
///
/// Three properties follow directly from `half = (base − sea) / 2`, and they
/// are what the tests pin:
///
/// 1. **No cell can cross.** `base ≥ sea` gives `base + δ ≥ (base + sea)/2 ≥
///    sea`; `base < sea` gives `base + δ ≤ (base + sea)/2 < sea`. Refinement
///    adds resolution and does not move the coastline — which matters because
///    settlement placement, the water mask, the flooded-cell test and the road
///    network are every one of them built against that boundary.
/// 2. **No flat shelf.** The bound is proportional to the distance from sea
///    level, so it goes to zero only *at* sea level; a coastal band keeps its
///    relief instead of being pinned to one value.
/// 3. **No constant, and bit-identical away from the coast.** Wherever the
///    detail is already under half the headroom — everywhere but a narrow
///    band either side of the shelf — `delta` comes back unchanged and the
///    output is byte-for-byte what it was before.
///
/// NaN propagates: every comparison here is false for a NaN `base` or `delta`,
/// so `delta` is returned unchanged and the NaN reaches the write-back, which
/// is what keeps [`amplify_region`]'s documented `0/0` case observable.
///
/// **Authorised by `LARGE_ITEM_RULINGS.md`'s Ruling O (owner, 2026-09-21)**,
/// against `OUTSTANDING_WORK.md`'s v2.69 row. It moves generated pixels near
/// every coastline. Four golden files were re-based by it —
/// `tests/golden_parity_amplify.rs`, `tests/golden_parity_zoom_detail.rs` and
/// `cartalith-engine`'s `golden_parity_bake.rs` and
/// `golden_parity_region_export.rs` — and each records what moved at its own
/// symbol. That list is what `cargo test --workspace` reported failing on the
/// change and nothing else; no other golden in the workspace reaches this
/// code near sea level.
#[inline]
pub(crate) fn clamp_toward_sea(base: f64, delta: f64, sea: f64) -> f64 {
    let half = (base - sea) * 0.5;
    if base >= sea {
        // Land. `half >= 0`, and going *down* is toward sea.
        if delta < -half { -half } else { delta }
    } else {
        // Water (and NaN, whose comparisons are all false either way).
        // `half < 0`, so `-half` is the positive half-headroom and going *up*
        // is toward sea.
        if delta > -half { -half } else { delta }
    }
}

/// The reference's inline bilinear sampler over the coarse source, clamped at
/// every edge. Reads `f32` storage, computes in `f64` — the same split as the
/// rest of this crate.
#[inline]
fn samp(src: &[f32], src_w: usize, src_h: usize, fx: f64, fy: f64) -> f64 {
    let (sw, sh) = (src_w as f64, src_h as f64);
    let fx = fx.clamp(0.0, sw - 1.0);
    let fy = fy.clamp(0.0, sh - 1.0);
    // `fx|0` truncates toward zero; fx is non-negative here after the clamp,
    // and NaN truncates to 0 exactly as `NaN|0` does in JS.
    let x0 = fx as i64 as usize;
    let y0 = fy as i64 as usize;
    let x1 = if x0 < src_w - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < src_h - 1 { y0 + 1 } else { y0 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let (a, b) = (src[y0 * src_w + x0] as f64, src[y0 * src_w + x1] as f64);
    let (c, d) = (src[y1 * src_w + x0] as f64, src[y1 * src_w + x1] as f64);
    (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (c * (1.0 - tx) + d * tx) * ty
}

/// `amplifyRegion(src, srcW, srcH, region, outW, outH, opts)` (reference line
/// 10265).
///
/// Refines `region` of the coarse field `src` to `out_w × out_h`: bilinear
/// upsample of the coarse constraint (which keeps continents and ranges),
/// plus world-space high-frequency detail, tapered by local relief so plains
/// and ocean floors stay smooth, and faded out below the shelf.
///
/// # The v2.69 sea-level guard — a deliberate departure from the reference
///
/// The `underwater` fade is **one-sided**: it shrinks detail going *down* into
/// water and does nothing going *up*, so a land pixel a hair above the shelf
/// took the full `±detailAmp/2` band and could be pushed under. Since
/// **Ruling O** (`LARGE_ITEM_RULINGS.md`, owner, 2026-09-21) the detail
/// excursion is additionally capped toward sea level at half the remaining
/// headroom — see [`clamp_toward_sea`]. The fade itself is untouched; the cap
/// sits on top of it, and away from the shelf the output is byte-identical to
/// the frozen v2.11 reference's.
///
/// # Panics
///
/// Panics if `src.len() < src_w * src_h`, or if either source dimension is
/// zero — the reference indexes `src` unchecked and would read `undefined`.
/// The reference's own signature, argument for argument: grouping them into a
/// struct would put a coordinate frame and a noise configuration in one bag and
/// make the port harder to check against `amplifyRegion` line by line.
#[allow(clippy::too_many_arguments)]
pub fn amplify_region(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    out_w: usize,
    out_h: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    amplify_region_padded(src, src_w, src_h, region, out_w, out_h, 0, opts)
}

/// [`amplify_region`] with a **halo** of `pad` extra texels on every side:
/// the result is `(out_w + 2·pad) × (out_h + 2·pad)`, and its texel
/// `(ox + pad, oy + pad)` is [`amplify_region`]'s texel `(ox, oy)` **bit for
/// bit** — the same expression with the same operands, since a halo texel is
/// only the same mapping evaluated at `ox = -pad..0` and `out_w..out_w+pad`.
/// The detail is sampled in continuous coarse coordinates, so the halo holds
/// exactly the texels the neighbouring tile has at those positions.
///
/// It exists for the deep-zoom tile colouriser (`cartalith-godot`'s
/// `lod_bridge::synthesize_tile_rgba`), whose shading reads neighbours up to a
/// few texels away: without a halo it had to clamp at the tile edge, and the
/// two tiles either side of a boundary shaded their shared edge differently.
/// `pad == 0` is [`amplify_region`] itself.
#[allow(clippy::too_many_arguments)]
pub fn amplify_region_padded(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    out_w: usize,
    out_h: usize,
    pad: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    assert!(src_w > 0 && src_h > 0, "amplify_region needs a non-empty source field");
    assert!(
        src.len() >= src_w * src_h,
        "amplify_region source is {} cells, needs {}",
        src.len(),
        src_w * src_h
    );
    let FloatRegion { x: rx, y: ry, w: rw, h: rh } = *region;
    let (pw, ph) = (out_w + 2 * pad, out_h + 2 * pad);
    let padf = pad as f64;
    let mut out = vec![0.0f32; pw * ph];
    // Row-parallel (`CPU_MULTITHREADING_SCOPE.md`'s own `output[i] = f(input, i)`
    // bar): every output pixel is a pure function of the frozen `src` and its
    // own `(ox, oy)`, so the arithmetic per pixel is untouched and unreordered
    // and the result is bit-identical to the sequential form -- the goldens
    // pass at exact equality, not a new tolerance. Parallelising *here* rather
    // than over tiles is what lets the interactive deep-zoom path benefit
    // without the shell changing how it asks for a tile; `bake_tiles` already
    // goes wide one level up and simply nests.
    //
    // ponytail: ceiling is ~7x of the ~8.8x `PERFORMANCE_BENCHMARKS.md` §5.4
    // measured, because a 48-tile burst pays this dispatch 48 times over where
    // one tile-parallel dispatch would pay it once. Upgrade path if that last
    // ~25% is ever wanted: a batch entry point on `WorldGen` taking the whole
    // `build_keys` set, with `viewport_host.gd`'s per-tile loop calling it once
    // -- `LOD_TILING_INTEGRATION_SCOPE.md`'s milestone, and a shell change.
    out.par_chunks_mut(pw).enumerate().for_each(|(oy, out_row)| {
        // `oy - pad` as a float: exactly `oy` at `pad == 0`.
        let oy = oy as f64 - padf;
        let cy = if rh > 1.0 {
            ry + (oy / (out_h as f64 - 1.0)) * (rh - 1.0)
        } else {
            ry
        };
        for (ox, o) in out_row.iter_mut().enumerate() {
            let ox = ox as f64 - padf;
            let cx = if rw > 1.0 {
                rx + (ox / (out_w as f64 - 1.0)) * (rw - 1.0)
            } else {
                rx
            };
            let base = samp(src, src_w, src_h, cx, cy);
            // local relief from the coarse field -> detail amplitude
            let e = 1.0;
            let gx = (samp(src, src_w, src_h, cx + e, cy) - samp(src, src_w, src_h, cx - e, cy)) * 0.5;
            let gy = (samp(src, src_w, src_h, cx, cy + e) - samp(src, src_w, src_h, cx, cy - e)) * 0.5;
            let relief = js_min(1.0, js_hypot(gx, gy) * 8.0);
            // fade detail out below the shelf. `NaN < sea` is false in both
            // languages, so a NaN base takes the `0` branch identically.
            let underwater = if base < opts.sea {
                js_max(0.0, (opts.sea - base) / 0.06)
            } else {
                0.0
            };
            let taper = relief * js_max(0.0, 1.0 - underwater);
            // detail sampled in continuous coarse coords -> seamless across
            // tiles sharing (cx, cy)
            let d = if opts.ridged {
                cartalith_noise::ridged(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
            } else {
                cartalith_noise::fbm(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
            } - 0.5;
            // v2.69's sea-level guard -- see [`clamp_toward_sea`]. The frozen
            // v2.11 reference writes `base + d*detailAmp*taper` bare here, and
            // the `underwater` term above only fades detail out going DOWN
            // into water, so a land pixel a hair above the shelf took the full
            // band and could be pushed under. Ruling O.
            let v = base + clamp_toward_sea(base, d * opts.detail_amp * taper, opts.sea);
            // The reference's own `v<0?0:v>1?1:v`. `f64::clamp` is that
            // expression exactly, NaN included: NaN fails both comparisons and
            // falls through unchanged, which is what makes the `out_w == 1`
            // case observable at all.
            *o = v.clamp(0.0, opts.top) as f32;
        }
    });
    out
}

/// `refineTile(src, srcW, srcH, region, cols, rows, col, row, tileW, tileH,
/// opts)` (reference line 10305).
///
/// Splits `region` into a `cols × rows` grid and amplifies one tile of it.
/// Each tile's coarse sub-bounds **overlap its neighbour by exactly one coarse
/// column/row** (`w: stepX + 1`), so the shared edge pixels map to the same
/// coarse coordinate and, because the detail is sampled in continuous coarse
/// coords, adjacent tiles agree exactly — seam delta zero, asserted by test.
#[allow(clippy::too_many_arguments)]
pub fn refine_tile(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
    tile_w: usize,
    tile_h: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    refine_tile_padded(src, src_w, src_h, region, cols, rows, col, row, tile_w, tile_h, 0, opts)
}

/// [`refine_tile`] with [`amplify_region_padded`]'s halo of `pad` texels.
#[allow(clippy::too_many_arguments)]
pub fn refine_tile_padded(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
    tile_w: usize,
    tile_h: usize,
    pad: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    let step_x = region.w / cols as f64;
    let step_y = region.h / rows as f64;
    let sub = FloatRegion {
        x: region.x + col as f64 * step_x,
        y: region.y + row as f64 * step_y,
        w: step_x + 1.0,
        h: step_y + 1.0,
    };
    amplify_region_padded(src, src_w, src_h, &sub, tile_w, tile_h, pad, opts)
}

/// `addZoomDetail(data, W, H, coarse, cW, cH, b, z, opts)` (reference line
/// 10467) — the pyramid's *progressive* (fractal) zoom detail, applied in place
/// on top of a [`refine_tile`] result.
///
/// The reference's own header states the problem it solves: *"amplifyRegion
/// adds detail at a FIXED coarse-space frequency, so the fbm runs out of
/// octaves at high zoom and the surface goes smooth ('details don't get more
/// intricate'). This adds `z − zBase` extra finer octaves, each 2× frequency,
/// sampled in SHARED coarse coords with a COARSE-relief taper — so adjacent
/// same-level tiles stay seam-Δ=0 exactly (and oceans/flats stay smooth)."*
///
/// Three properties worth stating because they are what the tests pin:
///
/// 1. **`z <= opts.z_base` is a no-op**, byte for byte — `extra` is
///    non-positive and the function returns before touching `data`. That is
///    what makes it safe to call unconditionally from [`crate`]'s pyramid path
///    at every level.
/// 2. **The octave count is capped at 6** (`Math.min(6, z - zBase)`), so an
///    absurd level costs no more than a sane deep one.
/// 3. **Below `sea` nothing is touched at all** (`if(base<sea) continue`),
///    which is a *different* rule from [`amplify_region`]'s smooth
///    `underwater` fade — the reference really does use a hard cut here.
///
/// `b` is the tile's coarse-cell footprint, i.e. exactly what
/// `cartalith_spatial::pyramid::pyramid_tile_bounds` returns; the detail is
/// sampled at the shared coarse coordinate it maps each output texel to, which
/// is the whole reason two neighbouring tiles agree on their shared edge.
///
/// # Two scale properties, checked 2026-09-21 for `LOD_DETAIL_SCOPE.md`
/// LOD-D5 -- and the owner ruling they waited for
///
/// **2026-10-17: Ruling CI decided them.** This branch (`opts.km == None`) is
/// unchanged and is still every golden's and every engine data product's; the
/// app's deep-zoom tiles now run [`KmZoomDetail`] instead, which re-keys all
/// three properties below to kilometres. The text below describes this
/// reference branch.
///
/// LOD-D5's last scope bullet is *"check at the symbols whether
/// `add_zoom_detail`'s frequency ignores cell size. If it does, record it for
/// an owner ruling; the golden schedule does not change here."* It does, in
/// two independent ways, and neither is touched:
///
/// 1. **The frequency is per COARSE CELL, not per kilometre.** `cx`/`cy` are
///    coarse sample coordinates and `f` starts at `opts.detail_freq * 2` —
///    `AmplifyOpts::detail_freq`'s own doc says *"noise cycles per coarse cell
///    at octave 0"*. Nothing in this function, or in the `AmplifyOpts` it
///    reads, carries a map width. So a 40 000 km world and an 800 km world on
///    the same grid get detail of the same wavelength **in cells** and
///    therefore of fifty times the wavelength **in kilometres**: the same
///    pyramid level renders mountain-scale corrugation on one and
///    continent-scale on the other. Whether that is wrong is a judgement about
///    what a fantasy map should look like at scale, not a defect in the port
///    — the reference does exactly this — which is why it is an owner ruling
///    and not a fix. Keying it on `cartalith_spatial::cell_km` would move
///    every pyramid tile and is a golden re-baseline.
///
/// 2. **The octave COUNT is keyed on the pyramid level, not on ground scale
///    either** (`min(6, z - z_base)`), so the same `z` adds the same six
///    octaves whatever a cell is worth on the ground. This is the same ruling
///    in its second half and moves with it.
///
/// A third property is **not** a defect and is recorded so it is not
/// re-reported as one: the amplitude decays `0.6` per octave while the
/// frequency doubles, so per-pixel relief falls by `0.6` for each level a
/// viewer descends. That is why LOD-D5's *"detail per screen pixel is
/// non-decreasing from zoom 4 to 40"* is not attainable without changing this
/// schedule — measured and attributed in `cartalith-godot`'s
/// `lod_d5_detail_per_screen_pixel_and_the_bar_it_does_not_meet`. It is the
/// same ruling a third time.
///
/// # The v2.69 sea-level guard — a deliberate departure from the reference
///
/// `OUTSTANDING_WORK.md`'s v2.69 row measured this pass pushing land pixels
/// under the shelf and never the other way: the `if base < sea { continue }`
/// of property 3 exempts water cells outright while a land cell took up to six
/// *unguarded* octaves, so the drift was strictly one-directional. Since
/// **Ruling O** (`LARGE_ITEM_RULINGS.md`, owner, 2026-09-21) the write-back
/// caps its excursion toward sea level at half the remaining headroom — see
/// [`clamp_toward_sea`] for the mechanism, its three properties, and the
/// `RC_ENGINE_CHANGES.md` §8.1 passage it is ported from.
///
/// The hard cut in property 3 **stays**: the fix is the clamp alone, which is
/// what the HTML's own v2.69 did, and giving water cells octaves they have
/// never had would be a second, unauthorised change. With both in place a
/// coarse-water cell is skipped here and a coarse-land cell cannot be pushed
/// under, so the coastline is fixed in *both* directions.
///
/// Measured over the **whole** refine-plus-zoom path ([`sample_elevation`]),
/// on a synthetic coastal gradient, 4x supersampled, 29 673 samples, `z` =
/// 2/4/6/8 — so `z = 2` is [`amplify_region`]'s contribution alone, since
/// `extra` is non-positive there. Before → after: land→sea
/// **0.19 / 0.29 / 0.29 / 0.29 %** of all samples → **0 % at every level**;
/// sea→land **0.19 / 0.15 / 0.14 / 0.14 %** → **0 % at every level**. The
/// growth with depth is this function's share, and it is the one-directional
/// part; `amplify_region`'s own contribution at `z = 2` was already
/// symmetric. The regression test is
/// `refinement_never_moves_the_coastline_in_either_direction`, which rebuilds
/// that fixture.
///
/// # Panics
///
/// Panics if `data` is shorter than `w * h`, or `coarse` shorter than
/// `cw * ch`.
pub fn add_zoom_detail(
    data: &mut [f32],
    w: usize,
    h: usize,
    coarse: &[f32],
    cw: usize,
    ch: usize,
    b: &FloatRegion,
    z: i32,
    opts: &AmplifyOpts,
) {
    add_zoom_detail_padded(data, w, h, 0, coarse, cw, ch, b, z, opts)
}

/// [`add_zoom_detail`] over a tile carrying [`amplify_region_padded`]'s halo:
/// `data` is `(w + 2·pad) × (h + 2·pad)`, `w`/`h`/`b` describe the tile
/// **without** it, and every texel — halo included — gets the octaves at the
/// same coarse coordinate `b.x + (ox/(w-1))·b.w` extends to. The core texels
/// are [`add_zoom_detail`]'s own, bit for bit; `pad == 0` is that function.
#[allow(clippy::too_many_arguments)]
pub fn add_zoom_detail_padded(
    data: &mut [f32],
    w: usize,
    h: usize,
    pad: usize,
    coarse: &[f32],
    cw: usize,
    ch: usize,
    b: &FloatRegion,
    z: i32,
    opts: &AmplifyOpts,
) {
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let padf = pad as f64;
    assert!(data.len() >= pw * ph, "add_zoom_detail data is too short");
    assert!(cw > 0 && ch > 0 && coarse.len() >= cw * ch, "add_zoom_detail coarse is too short");
    // Ruling CI / FG: the kilometre law, a separate branch so the reference
    // schedule below is untouched by control flow (`km: None`). `ρ = 1` here:
    // a caller with a biome roughness calls [`add_zoom_detail_km_padded`].
    if opts.km.is_some() {
        add_zoom_detail_km_padded(data, w, h, pad, coarse, cw, ch, b, z, opts, None);
        return;
    }
    let extra = i32::min(6, z - opts.z_base);
    if extra <= 0 {
        return;
    }
    let sea = opts.sea;
    // The reference declares its own local `samp` here rather than reusing
    // `amplifyRegion`'s; the two are the same bilinear clamp, so this reuses
    // `samp` above -- an internal restructuring that preserves output, which is
    // the kind `cartalith-porting-discipline` explicitly allows.
    let base_freq = opts.detail_freq;
    let base_amp = opts.detail_amp;
    let zk = opts.zoom_detail_k;
    // Row-parallel for the same reason [`amplify_region`] is: each pixel reads
    // only its own `data[i]` and the frozen `coarse`, and writes only `data[i]`.
    // This is the term that grows with depth (up to 6 extra fbm octaves per
    // pixel), so it is most of a deep tile's cost.
    data[..pw * ph].par_chunks_mut(pw).enumerate().for_each(|(oy, row)| {
        let oy = oy as f64 - padf;
        let cy = b.y + if h > 1 { oy / (h as f64 - 1.0) * b.h } else { 0.0 };
        for (ox, cell) in row.iter_mut().enumerate() {
            let base = *cell as f64;
            if base < sea {
                continue;
            }
            let ox = ox as f64 - padf;
            let cx = b.x + if w > 1 { ox / (w as f64 - 1.0) * b.w } else { 0.0 };
            let gx = (samp(coarse, cw, ch, cx + 1.0, cy) - samp(coarse, cw, ch, cx - 1.0, cy)) * 0.5;
            let gy = (samp(coarse, cw, ch, cx, cy + 1.0) - samp(coarse, cw, ch, cx, cy - 1.0)) * 0.5;
            let relief = js_min(1.0, js_hypot(gx, gy) * 8.0);
            if relief <= 0.0 {
                continue;
            }
            let mut amp = base_amp * 0.6 * zk;
            let mut f = base_freq * 2.0;
            let mut sum = 0.0;
            for o in 0..extra {
                sum += (cartalith_noise::fbm(cx * f, cy * f, opts.seed + 1 + o) - 0.5) * amp;
                f *= 2.0;
                amp *= 0.6;
            }
            // The reference writes back unclamped -- `data[i]=base+sum*relief`,
            // no `[0,1]` clamp, unlike `amplifyRegion`'s. That is still true
            // in the `[0,1]` direction: a peak may still leave the unit range
            // upward and a seabed downward, and the goldens pin it.
            //
            // What *is* guarded, since Ruling O, is the excursion toward SEA
            // LEVEL -- see [`clamp_toward_sea`]. `RC_ENGINE_CHANGES.md` §8.1:
            // *"the band is added in TWO places (`amplifyRegion` and
            // `addZoomDetail`); guarding one left a third of the drift"*, so
            // this one moves with `amplify_region`'s above.
            *cell = (base + clamp_toward_sea(base, sum * relief, sea)) as f32;
        }
    });
}

/// [`add_zoom_detail_padded`]'s kilometre branch ([`KmZoomDetail`]), with
/// the caller's biome and rock roughness `rough` (`None` is `ρ = 1`, what
/// [`add_zoom_detail_padded`] itself passes). A no-op when `opts.km` is
/// `None`. Every texel, halo included, is a pure function of its coarse
/// coordinate, the frozen `coarse` field and `rough` at that coordinate, so two
/// tiles agree on their shared edge bit for bit, and [`sample_elevation_rough`]
/// reproduces it texel for texel.
///
/// Per texel: the reference's hard `base < sea` skip (water is never given
/// octaves), the coarse field's [`km_local_relief`] from its half-differences
/// and four-neighbour curvature at `(cx, cy)`, the octave shape
/// ([`km_zoom_detail_sum`]), the amplitude and cap ([`km_detail_delta`]), and
/// Ruling O's [`clamp_toward_sea`] on the write-back.
///
/// # Panics
///
/// As [`add_zoom_detail_padded`].
#[allow(clippy::too_many_arguments)]
pub fn add_zoom_detail_km_padded(
    data: &mut [f32],
    w: usize,
    h: usize,
    pad: usize,
    coarse: &[f32],
    cw: usize,
    ch: usize,
    b: &FloatRegion,
    z: i32,
    opts: &AmplifyOpts,
    rough: Option<Roughness>,
) {
    let Some(km) = opts.km else { return };
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let padf = pad as f64;
    assert!(data.len() >= pw * ph, "add_zoom_detail data is too short");
    assert!(cw > 0 && ch > 0 && coarse.len() >= cw * ch, "add_zoom_detail coarse is too short");
    let sea = opts.sea;
    data[..pw * ph].par_chunks_mut(pw).enumerate().for_each(|(oy, row)| {
        let oy = oy as f64 - padf;
        let cy = b.y + if h > 1 { oy / (h as f64 - 1.0) * b.h } else { 0.0 };
        for (ox, cell) in row.iter_mut().enumerate() {
            let base = *cell as f64;
            // The reference's hard cut, kept: water is never given octaves.
            if base < sea {
                continue;
            }
            let ox = ox as f64 - padf;
            let cx = b.x + if w > 1 { ox / (w as f64 - 1.0) * b.w } else { 0.0 };
            let sum = km_zoom_detail_sum(cx, cy, z, opts, &km);
            if sum == 0.0 {
                continue;
            }
            *cell = (base + km_texel_delta(coarse, cw, ch, cx, cy, base, sum, opts, &km, rough)) as f32;
        }
    });
}

/// One texel's guarded kilometre-law excursion: the coarse field's relief at
/// `(cx, cy)`, the roughness there, [`km_detail_delta`], then Ruling O's
/// [`clamp_toward_sea`]. The one body [`add_zoom_detail_km_padded`] and
/// [`sample_elevation_rough`] share, so the point query and the tile are one
/// expression on identical inputs.
#[allow(clippy::too_many_arguments)]
#[inline]
fn km_texel_delta(coarse: &[f32], cw: usize, ch: usize, cx: f64, cy: f64, base: f64, sum: f64, opts: &AmplifyOpts, km: &KmZoomDetail, rough: Option<Roughness>) -> f64 {
    let (l, r) = (samp(coarse, cw, ch, cx - 1.0, cy), samp(coarse, cw, ch, cx + 1.0, cy));
    let (u, d) = (samp(coarse, cw, ch, cx, cy - 1.0), samp(coarse, cw, ch, cx, cy + 1.0));
    let c = samp(coarse, cw, ch, cx, cy);
    let relief = km_local_relief((r - l) * 0.5, (d - u) * 0.5, (l + r + u + d) * 0.25 - c);
    let rho = rough.map_or(1.0, |f| f(cx, cy));
    clamp_toward_sea(base, km_detail_delta(sum, relief, rho, base, opts.sea, km), opts.sea)
}

/// `opts.z_base` for a tile rendered `tile_size` pixels across — the octave
/// schedule [`add_zoom_detail`] should run at that output resolution.
///
/// [`AmplifyOpts::z_base`]'s default of `2` is the reference's own and it is
/// quoted against the reference's own `_lodTile = 1024`: at level `z` a
/// 1024 px tile resolves `1024·2^z/(cw−1)` px per coarse cell, and
/// `min(6, z − 2)` extra octaves is as many as that can carry without
/// aliasing. A tile a quarter that size reaches the same ground resolution
/// two levels deeper, so leaving `z_base` at `2` for it schedules two octaves
/// *past* what the tile can resolve — which reads as noise, not as detail.
/// Shifting `z_base` by the same `log2` keeps the schedule identical at equal
/// ground resolution, which is what makes a screen tile and a baked chunk over
/// the same ground agree.
///
/// **This changes no existing output.** At `tile_size == 1024` it returns the
/// reference's own `2` exactly — the size every shipped bake runs at and the
/// value [`AmplifyOpts::default`] already carries — so it only stops a
/// differently-sized tile from getting the wrong schedule.
///
/// `cartalith_godot::lod_bridge::z_base()` already derives this same quantity
/// for its own fixed `TILE_PX`, by the same argument and to the same number
/// (`z_base_for_tile_size(256) == 4`). Routing that call site through here is
/// a follow-up: this pass deliberately touches no Godot bridge, so there are
/// two spellings of one formula until it lands.
pub fn z_base_for_tile_size(tile_size: usize) -> i32 {
    // `1024` is the reference's `_lodTile` default (reference line 10656) --
    // the tile size `AmplifyOpts::default().z_base` is quoted against, not a
    // tuning constant. `max(1)` keeps a zero out of `log2`, where it would go
    // to +inf and saturate the cast.
    AmplifyOpts::default().z_base + (1024.0 / tile_size.max(1) as f64).log2().round() as i32
}

/// **EF-0** (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §5) — the elevation
/// field as a **point query**: the refined height at one continuous coarse
/// coordinate, at one pyramid level.
///
/// `cx`/`cy` are **coarse sample coordinates**, the `[0, cw−1] × [0, ch−1]`
/// space `cartalith_spatial::pyramid::pyramid_tile_bounds` returns and
/// [`amplify_region`] maps each of its output pixels into — not pixels, and
/// not normalised. Outside that box the coarse sampler clamps at every edge,
/// exactly as it does for [`amplify_region`].
///
/// # The property this exists for
///
/// This is **the same field** [`refine_tile`] + [`add_zoom_detail`] produce,
/// not an approximation of it. For every texel `(ox, oy)` of a pyramid tile at
/// level `z`, with `b = pyramid_tile_bounds(cw, ch, z, col, row)`,
///
/// ```text
/// tile[oy * w + ox] == sample_elevation(coarse, cw, ch,
///                                       b.x + ox/(w-1) * b.w,
///                                       b.y + oy/(h-1) * b.h, z, opts)
/// ```
///
/// **bit for bit** — pinned by `sample_elevation_reproduces_the_tile_path_
/// texel_for_texel` in this module, over four levels and every texel of each
/// tile. Two consequences follow, and they are the whole point:
///
/// 1. **Seam consistency is inherited, not re-derived.** Two tiles that share
///    an edge map that edge to the *same* `(cx, cy)` (the reason
///    [`refine_tile`]'s sub-bounds overlap by one coarse column), and a pure
///    function of `(cx, cy)` cannot disagree with itself. The seam property is
///    therefore a property of this function's signature.
/// 2. **Determinism is structural.** No RNG state, no cache, no global: the
///    result is a pure function of its arguments, and `fbm`/`ridged` are
///    seeded hash noise. The same world and the same coordinate always produce
///    the same bytes.
///
/// # Why this is a transcription rather than a call
///
/// [`amplify_region`] and [`add_zoom_detail`] are row-parallel over a whole
/// output rectangle, and both derive `(cx, cy)` from an output grid a point
/// query does not have. Calling them for one sample would mean inventing a
/// 1×1 grid, which is exactly [`amplify_region`]'s documented `0/0` NaN case.
/// So the two per-pixel bodies are reproduced here — and the texel-for-texel
/// test above is what keeps the copy honest: change either of them without
/// changing this and the test goes red.
///
/// That degenerate 1×1 mapping is also why nothing here can reach it: this
/// takes the coordinate directly, so there is no `outW − 1` to divide by.
///
/// # Panics
///
/// Panics if `coarse` is shorter than `cw * ch`, or either dimension is zero —
/// the same precondition, with the same reasoning, as [`amplify_region`].
pub fn sample_elevation(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    cx: f64,
    cy: f64,
    z: i32,
    opts: &AmplifyOpts,
) -> f32 {
    sample_elevation_rough(coarse, cw, ch, cx, cy, z, opts, None)
}

/// [`sample_elevation`] with the kilometre law's biome and rock roughness
/// (Ruling FG): the point query of [`add_zoom_detail_km_padded`] called with
/// the same `rough`, texel for texel. `None` is [`sample_elevation`].
#[allow(clippy::too_many_arguments)]
pub fn sample_elevation_rough(
    coarse: &[f32],
    cw: usize,
    ch: usize,
    cx: f64,
    cy: f64,
    z: i32,
    opts: &AmplifyOpts,
    rough: Option<Roughness>,
) -> f32 {
    assert!(cw > 0 && ch > 0, "sample_elevation needs a non-empty coarse field");
    assert!(
        coarse.len() >= cw * ch,
        "sample_elevation coarse is {} cells, needs {}",
        coarse.len(),
        cw * ch
    );

    // ---- `amplify_region`'s per-pixel body, at this coordinate -------------
    let base = samp(coarse, cw, ch, cx, cy);
    // `amplify_region`'s `e = 1.0` and `add_zoom_detail`'s literal `1.0` are
    // the same offset over the same field, so the tile path's two relief terms
    // are one expression evaluated twice on identical inputs. Computing it
    // once here is bit-identical to that, not a re-association: nothing is
    // reordered, only not repeated.
    let gx = (samp(coarse, cw, ch, cx + 1.0, cy) - samp(coarse, cw, ch, cx - 1.0, cy)) * 0.5;
    let gy = (samp(coarse, cw, ch, cx, cy + 1.0) - samp(coarse, cw, ch, cx, cy - 1.0)) * 0.5;
    let relief = js_min(1.0, js_hypot(gx, gy) * 8.0);
    let underwater =
        if base < opts.sea { js_max(0.0, (opts.sea - base) / 0.06) } else { 0.0 };
    let taper = relief * js_max(0.0, 1.0 - underwater);
    let d = if opts.ridged {
        cartalith_noise::ridged(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
    } else {
        cartalith_noise::fbm(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
    } - 0.5;
    // Both the `[0,1]` clamp **and** the `f32` narrowing happen before the
    // zoom-detail half reads the value back, because in the tile path they do:
    // `refine_tile` writes a `Vec<f32>` and `add_zoom_detail` starts from
    // `*cell as f64`. Carrying full `f64` through instead would agree to about
    // 1e-8 and fail every `to_bits` comparison in this module.
    // v2.69's sea-level guard, exactly as `amplify_region` applies it -- see
    // [`clamp_toward_sea`]. It has to be here too, for this function's whole
    // reason to exist: `sample_elevation_reproduces_the_tile_path_texel_for_
    // texel` asserts this is the *same* field bit for bit, so guarding one
    // copy and not the other turns a pinned identity into a pinned lie.
    let refined =
        (base + clamp_toward_sea(base, d * opts.detail_amp * taper, opts.sea)).clamp(0.0, opts.top) as f32;

    // ---- `add_zoom_detail`'s per-pixel body, at the same coordinate --------
    let b = refined as f64;
    // The kilometre branch (Ruling CI, re-based by Ruling FG), exactly as
    // `add_zoom_detail_km_padded` runs it: the hard sea cut, the shared octave
    // shape, the shared guarded excursion (`km_texel_delta`).
    if let Some(km) = opts.km {
        if b < opts.sea {
            return refined;
        }
        let sum = km_zoom_detail_sum(cx, cy, z, opts, &km);
        if sum == 0.0 {
            return refined;
        }
        return (b + km_texel_delta(coarse, cw, ch, cx, cy, b, sum, opts, &km, rough)) as f32;
    }
    let extra = i32::min(6, z - opts.z_base);
    // Three separate skips, in `add_zoom_detail`'s own order and with its own
    // semantics: `extra <= 0` is the documented shallow-level no-op,
    // `base < sea` is its **hard** cut (not `amplify_region`'s smooth
    // `underwater` fade), and `relief <= 0` is the flat-ground skip. NaN takes
    // the same branch it takes there -- every comparison is false, so a NaN
    // base falls through and stays NaN rather than being quietly dropped.
    if extra <= 0 || b < opts.sea || relief <= 0.0 {
        return refined;
    }
    let mut amp = opts.detail_amp * 0.6 * opts.zoom_detail_k;
    let mut f = opts.detail_freq * 2.0;
    let mut sum = 0.0;
    for o in 0..extra {
        sum += (cartalith_noise::fbm(cx * f, cy * f, opts.seed + 1 + o) - 0.5) * amp;
        f *= 2.0;
        amp *= 0.6;
    }
    // Written back with `add_zoom_detail`'s own guard and no other: still
    // unclamped in the `[0,1]` direction, capped toward sea level.
    (b + clamp_toward_sea(b, sum * relief, opts.sea)) as f32
}

/// Inline unit coverage for `amplify_region`/`refine_tile`/`add_zoom_detail`/
/// `sample_elevation`: seam agreement, the halo, the `0/0` NaN case, the
/// Ruling O sea-level guard's arithmetic, and the EF-0 point-query identity.
/// The golden-parity byte comparisons against the V8 reference live in
/// `tests/golden_parity_amplify.rs` and `tests/golden_parity_zoom_detail.rs`;
/// this module checks properties a golden hash cannot state on its own.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::pyramid::{pyramid_dims, pyramid_tile_bounds, ChunkId};
    use cartalith_spatial::Region;

    /// The same synthetic field the golden harness builds, in the same order:
    /// pure arithmetic (no `sin`/`cos`/`exp`) so V8 and Rust cannot disagree,
    /// with a deliberately **quantised** `% 11` term.
    pub(crate) fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let cx = gw as f64 * 0.42;
        let cy = gh as f64 * 0.55;
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
                let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
                v += 0.05 * ((q as f64 / 10.0) - 0.5);
                v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    /// Protects: the output buffer is sized `out_w * out_h`, not the
    /// source or region dimensions.
    #[test]
    fn output_is_exactly_out_w_times_out_h() {
        let src = synthetic_field(16, 12, 0);
        let o = amplify_region(&src, 16, 12, &Region { x: 2, y: 2, w: 8, h: 6 }.to_float(), 20, 15,
                               &AmplifyOpts::default());
        assert_eq!(o.len(), 300);
    }

    /// Protects: the final `[0,1]` clamp in `amplify_region`'s write-back
    /// holds even with an exaggerated detail amplitude.
    #[test]
    fn every_sample_stays_inside_the_unit_range() {
        let src = synthetic_field(24, 18, 3);
        let o = amplify_region(&src, 24, 18, &Region { x: 0, y: 0, w: 24, h: 18 }.to_float(), 40, 30,
                               &AmplifyOpts { detail_amp: 5.0, ..Default::default() });
        assert!(o.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    /// Protects: `detail_amp == 0.0` makes the noise term's coefficient
    /// zero, so the noise family and seed cannot change the output.
    #[test]
    fn zero_detail_amplitude_is_a_pure_upsample() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let with = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts { detail_amp: 0.0, ..Default::default() });
        let flat = amplify_region(&src, 16, 12, &reg, 20, 16,
                                  &AmplifyOpts { detail_amp: 0.0, ridged: true, seed: 999, ..Default::default() });
        // With no amplitude the noise family and seed cannot matter at all.
        assert_eq!(with, flat);
    }

    /// Protects: the converse of the zero-amplitude test above — a
    /// non-zero `detail_amp` really does perturb the output (guards
    /// against a term that got wired but multiplied away).
    #[test]
    fn the_detail_actually_changes_the_result() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let plain = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        let none = amplify_region(&src, 16, 12, &reg, 20, 16,
                                  &AmplifyOpts { detail_amp: 0.0, ..Default::default() });
        assert_ne!(plain, none, "detail_amp 0.14 produced the same field as 0.0");
    }

    /// Protects: `opts.ridged` actually switches the noise family used for
    /// the detail term, not just its label.
    #[test]
    fn ridged_and_fbm_detail_differ() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let a = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        let b = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts { ridged: true, ..Default::default() });
        assert_ne!(a, b);
    }

    /// Protects: the seam property this module's header describes —
    /// `refine_tile`'s one-coarse-column overlap gives two horizontally
    /// adjacent tiles a byte-identical shared edge.
    #[test]
    fn adjacent_tiles_agree_exactly_on_their_shared_edge() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (tw, th) = (16, 12);
        let left = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 0, tw, th, &o);
        let right = refine_tile(&src, 48, 32, &reg, 2, 2, 1, 0, tw, th, &o);
        for y in 0..th {
            assert_eq!(
                left[y * tw + (tw - 1)].to_bits(),
                right[y * tw].to_bits(),
                "seam mismatch at row {y}"
            );
        }
    }

    /// Protects: the same seam property on the other axis — the row
    /// overlap, not just the column one.
    #[test]
    fn vertically_adjacent_tiles_agree_too() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (tw, th) = (16, 12);
        let top = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 0, tw, th, &o);
        let bottom = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 1, tw, th, &o);
        for x in 0..tw {
            assert_eq!(top[(th - 1) * tw + x].to_bits(), bottom[x].to_bits(), "seam mismatch at col {x}");
        }
    }

    /// The halo is only more of the same mapping: the padded tile's core is
    /// the unpadded tile **bit for bit**, through both the refine and the zoom
    /// octaves, so a halo can never move a texel the atlas already has.
    #[test]
    fn a_padded_tiles_core_is_the_unpadded_tile_bit_for_bit() {
        // Protects: adding a halo never perturbs the tile's own core texels —
        // padded and unpadded core must agree to the bit through both the
        // refine and zoom-detail passes.
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, z_base: 1, ..Default::default() };
        let (tw, th, pad) = (16, 12, 3);
        let b = FloatRegion { x: 16.0, y: 4.0, w: 12.0, h: 8.0 };
        let mut plain = refine_tile(&src, 48, 32, &reg, 2, 2, 1, 0, tw, th, &o);
        add_zoom_detail(&mut plain, tw, th, &src, 48, 32, &b, 4, &o);
        let mut padded = refine_tile_padded(&src, 48, 32, &reg, 2, 2, 1, 0, tw, th, pad, &o);
        add_zoom_detail_padded(&mut padded, tw, th, pad, &src, 48, 32, &b, 4, &o);
        let pw = tw + 2 * pad;
        assert_eq!(padded.len(), pw * (th + 2 * pad));
        for y in 0..th {
            for x in 0..tw {
                assert_eq!(plain[y * tw + x].to_bits(), padded[(y + pad) * pw + x + pad].to_bits(), "texel {x},{y}");
            }
        }
    }

    /// …and the halo holds the **neighbour's** texels: the left tile's halo
    /// past its right edge is the right tile's first columns. Not to the bit —
    /// the two evaluate the same coarse coordinate by different sums — but to
    /// well under one `f32` step of anything a shader could see.
    #[test]
    fn a_tiles_halo_is_its_neighbours_own_texels() {
        // Protects: the halo the deep-zoom colouriser reads for its
        // neighbour-aware shading is really the neighbour's own field, to
        // well under one f32 step — not an edge-clamped guess.
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (tw, th, pad) = (16, 12, 3);
        let left = refine_tile_padded(&src, 48, 32, &reg, 2, 2, 0, 0, tw, th, pad, &o);
        let right = refine_tile(&src, 48, 32, &reg, 2, 2, 1, 0, tw, th, &o);
        let pw = tw + 2 * pad;
        let mut worst = 0.0f32;
        for y in 0..th {
            for k in 0..=pad {
                let a = left[(y + pad) * pw + pad + tw - 1 + k];
                worst = worst.max((a - right[y * tw + k]).abs());
            }
        }
        assert!(worst < 1e-6, "halo differs from the neighbour by {worst}");
    }

    /// Protects: a 1x1 region degenerates to a constant field, not NaN —
    /// the `rw <= 1`/`rh <= 1` branches of the coordinate mapping.
    #[test]
    fn a_collapsed_region_samples_one_coarse_point_everywhere() {
        let src = synthetic_field(16, 12, 2);
        let o = amplify_region(&src, 16, 12, &FloatRegion { x: 5.0, y: 5.0, w: 1.0, h: 1.0 }, 6, 6,
                               &AmplifyOpts::default());
        // rw <= 1 and rh <= 1 both take the `: rx` / `: ry` branch, so every
        // output cell reads the same coarse coordinate -- constant, not NaN.
        assert!(o.iter().all(|v| v.to_bits() == o[0].to_bits()));
        assert!(o[0].is_finite());
    }

    /// Protects: the documented `0/0` divide-by-`outW-1` case is ported
    /// as-written, not silently fixed — a real (non-degenerate) region
    /// with `out_w == out_h == 1` must still come back NaN.
    #[test]
    fn a_single_pixel_output_over_a_real_region_is_all_nan_like_the_reference() {
        let src = synthetic_field(16, 12, 2);
        let o = amplify_region(&src, 16, 12, &Region { x: 2, y: 2, w: 10, h: 10 }.to_float(), 1, 1,
                               &AmplifyOpts::default());
        // 0/0 in the coordinate mapping. tile_dims' max(2, ..) floor is why no
        // shipped caller reaches this; pinned so it is not silently "fixed".
        assert_eq!(o.len(), 1);
        assert!(o[0].is_nan());
    }

    /// Protects: `samp`'s edge behaviour is clamp, not wrap or panic —
    /// querying far outside the source grid returns the nearest corner.
    #[test]
    fn the_sampler_clamps_rather_than_wrapping_at_every_edge() {
        let src = synthetic_field(8, 6, 0);
        assert_eq!(samp(&src, 8, 6, -5.0, -5.0), src[0] as f64);
        assert_eq!(samp(&src, 8, 6, 99.0, 99.0), src[5 * 8 + 7] as f64);
    }

    /// Protects: the `underwater` fade really does drive the detail term
    /// to zero, leaving a pure upsample, once the whole field is below
    /// sea level.
    #[test]
    fn underwater_cells_lose_their_detail() {
        // A field entirely below sea level gets `underwater` >= 1 everywhere,
        // so taper is 0 and the output is the pure upsample.
        let src = vec![0.10f32; 16 * 12];
        let reg = Region { x: 0, y: 0, w: 16, h: 12 }.to_float();
        let o = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        assert!(o.iter().all(|v| (*v - 0.10).abs() < 1e-7));
    }

    // ---- Ruling O: the v2.69 sea-level guard -------------------------------

    /// Every arm of [`clamp_toward_sea`] against a **literal** it must equal,
    /// never against the expression that computed it — `MISTAKES.md`'s
    /// "write a test that pins a constant" row. The `0.5` in `half` is the
    /// only number in the function and these are what hold it: mutating it to
    /// `0.25` or `1.0` turns the two capped arms red, and widening
    /// `base >= sea` to `base > sea` turns the exactly-at-sea arm red.
    #[test]
    fn clamp_toward_sea_caps_at_exactly_half_the_headroom_and_nothing_else() {
        // Protects: every arm of clamp_toward_sea's cap, land and water,
        // including the exactly-at-sea boundary and NaN propagation.
        // Every number here is an exact binary fraction, so the literals are
        // the arithmetic and not a rounding of it.
        const SEA: f64 = 0.5;
        // Land at 0.75: 0.25 of headroom, so the cap is 0.125.
        assert_eq!(clamp_toward_sea(0.75, -0.5, SEA), -0.125, "land: a big dive caps at half");
        assert_eq!(clamp_toward_sea(0.75, -0.125, SEA), -0.125, "land: exactly half is untouched");
        assert_eq!(clamp_toward_sea(0.75, -0.0625, SEA), -0.0625, "land: under half passes through");
        assert_eq!(clamp_toward_sea(0.75, 0.5, SEA), 0.5, "land: going UP is never capped");
        // Water at 0.125: 0.375 of headroom, so the cap is 0.1875 -- a
        // different number from the land arm's, so one wrong factor cannot
        // satisfy both.
        assert_eq!(clamp_toward_sea(0.125, 0.5, SEA), 0.1875, "water: a big rise caps at half");
        assert_eq!(clamp_toward_sea(0.125, 0.1875, SEA), 0.1875, "water: exactly half is untouched");
        assert_eq!(clamp_toward_sea(0.125, 0.0625, SEA), 0.0625, "water: under half passes through");
        assert_eq!(clamp_toward_sea(0.125, -0.5, SEA), -0.5, "water: going DOWN is never capped");
        // Exactly at sea level: zero headroom either way, so no movement
        // toward it at all -- and this is the arm that distinguishes `>=`
        // from `>`, since the two branches disagree only here.
        assert_eq!(clamp_toward_sea(SEA, -0.5, SEA), 0.0);
        assert_eq!(clamp_toward_sea(SEA, 0.5, SEA), 0.5);
        // NaN propagates rather than being absorbed -- what keeps
        // `amplify_region`'s documented `0/0` case observable.
        assert_eq!(clamp_toward_sea(f64::NAN, -0.5, SEA), -0.5);
        assert!(clamp_toward_sea(0.6, f64::NAN, SEA).is_nan());
    }

    /// The property Ruling O exists for, stated as a **count of crossings**
    /// rather than as a hash: refinement adds resolution and does not move the
    /// coastline, in either direction, at any depth.
    ///
    /// The fixture is a synthetic coastal gradient — height ramps 0.20 → 0.80
    /// across `x`, crossing `sea` about a third of the way in, with a
    /// quantised per-cell wobble so the coastline wanders and local relief is
    /// non-zero (`add_zoom_detail` skips `relief <= 0`, and a smooth ramp
    /// would structurally fail to reach the code under test).
    ///
    /// Measured on this exact fixture immediately before the guard landed:
    /// land→sea **0.19 / 0.29 / 0.29 / 0.29 %** of all samples at
    /// `z` = 2/4/6/8 and sea→land **0.19 / 0.15 / 0.14 / 0.14 %** — the
    /// one-directional growth with depth is `add_zoom_detail`'s share.
    #[test]
    fn refinement_never_moves_the_coastline_in_either_direction() {
        // Protects: the property Ruling O's guard exists for — refinement
        // adds resolution and never flips a sample across the shelf, at any
        // pyramid level, in either direction.
        const CW: usize = 48; // fixture grid width, arbitrary but non-trivial
        const CH: usize = 40; // fixture grid height, arbitrary but non-trivial
        const SEA: f64 = 0.42; // AmplifyOpts::default().sea, restated as a fixture constant
        let mut f = vec![0.0f32; CW * CH];
        for y in 0..CH {
            for x in 0..CW {
                let mut v = 0.20 + 0.60 * (x as f64 / (CW as f64 - 1.0));
                let q = (x as i64 * 7 + y as i64 * 13 + 5).rem_euclid(11);
                v += 0.06 * ((q as f64 / 10.0) - 0.5);
                f[y * CW + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        let opts = AmplifyOpts { seed: 4242, sea: SEA, ..Default::default() };
        // The fixture has to actually straddle the shelf, or this passes vacuously.
        assert!(f.iter().any(|&v| (v as f64) < SEA) && f.iter().any(|&v| (v as f64) >= SEA));
        for z in [2i32, 4, 6, 8] {
            let (mut l2s, mut s2l, mut land, mut sea_n) = (0usize, 0usize, 0usize, 0usize);
            // 4x supersample of the coarse domain -- the density a deep tile
            // reads it at, and 29 673 samples.
            for iy in 0..((CH - 1) * 4 + 1) {
                for ix in 0..((CW - 1) * 4 + 1) {
                    let (cx, cy) = (ix as f64 / 4.0, iy as f64 / 4.0);
                    let base = samp(&f, CW, CH, cx, cy);
                    let r = sample_elevation(&f, CW, CH, cx, cy, z, &opts) as f64;
                    if base >= SEA {
                        land += 1;
                        l2s += usize::from(r < SEA);
                    } else {
                        sea_n += 1;
                        s2l += usize::from(r >= SEA);
                    }
                }
            }
            assert!(land > 1_000 && sea_n > 1_000, "z={z}: the fixture is not really coastal");
            assert_eq!(l2s, 0, "z={z}: {l2s} of {land} land samples were drowned");
            assert_eq!(s2l, 0, "z={z}: {s2l} of {sea_n} water samples were raised to land");
        }
    }

    /// The other half of the ruling's mechanism: *"no flat shelf"*. Clamping
    /// the **result** to sea level would pin a coastal band to one value; the
    /// half-headroom bound on the **delta** goes to zero only exactly at the
    /// shelf, so the band keeps its relief.
    #[test]
    fn the_guard_leaves_a_coastal_band_with_relief_rather_than_a_flat_shelf() {
        // Protects: the delta-clamp's "no flat shelf" property — a coastal
        // band keeps measurable relief spread rather than pinning to one
        // value the way a result-clamp would.
        const CW: usize = 48; // fixture grid width, arbitrary but non-trivial
        const CH: usize = 40; // fixture grid height, arbitrary but non-trivial
        const SEA: f64 = 0.42; // AmplifyOpts::default().sea, restated as a fixture constant
        let mut f = vec![0.0f32; CW * CH];
        for y in 0..CH {
            for x in 0..CW {
                let mut v = 0.20 + 0.60 * (x as f64 / (CW as f64 - 1.0));
                let q = (x as i64 * 7 + y as i64 * 13 + 5).rem_euclid(11);
                v += 0.06 * ((q as f64 / 10.0) - 0.5);
                f[y * CW + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        let opts = AmplifyOpts { seed: 4242, sea: SEA, ..Default::default() };
        // Every sample within 0.01 of the shelf on the land side.
        let band: Vec<f32> = (0..((CH - 1) * 4 + 1))
            .flat_map(|iy| (0..((CW - 1) * 4 + 1)).map(move |ix| (ix, iy)))
            .filter_map(|(ix, iy)| {
                let (cx, cy) = (ix as f64 / 4.0, iy as f64 / 4.0);
                let base = samp(&f, CW, CH, cx, cy);
                ((SEA..SEA + 0.01).contains(&base))
                    .then(|| sample_elevation(&f, CW, CH, cx, cy, 8, &opts))
            })
            .collect();
        assert!(band.len() > 100, "the band is only {} samples", band.len());
        let lo = band.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = band.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        // A result-clamp would make this spread ~0; the delta-clamp keeps it.
        assert!(hi - lo > 0.005, "the coastal band flattened to a {:.6} spread", hi - lo);
        assert!(band.iter().all(|&v| v as f64 >= SEA), "the band crossed the shelf");
    }

    /// Protects: the 3 x 3 grid of `refine_tile` calls covers the whole region. Every tile's
    /// samples must equal, bit for bit, `amplify_region` over the sub-region written out here
    /// by hand (origin `region + index * step`, extent `step + 1` -- the one-cell overlap), so
    /// a wrong tile origin or extent changes real output and fails. The hand-written
    /// sub-regions are asserted to start at the region's near edge, to reach (at least) its far
    /// edge on both axes, and to join without a gap; and the tiles are asserted pairwise
    /// distinct, so a `refine_tile` that ignored `col`/`row` cannot pass.
    #[test]
    fn refine_tile_covers_the_whole_region_across_its_grid() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts::default();
        let (cols, rows) = (3usize, 3usize);
        let (step_x, step_y) = (reg.w / cols as f64, reg.h / rows as f64);
        let mut tiles: Vec<Vec<f32>> = Vec::new();
        let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY);
        for row in 0..rows {
            for col in 0..cols {
                let sub = FloatRegion {
                    x: reg.x + col as f64 * step_x,
                    y: reg.y + row as f64 * step_y,
                    w: step_x + 1.0,
                    h: step_y + 1.0,
                };
                min_x = min_x.min(sub.x);
                max_x = max_x.max(sub.x + sub.w);
                min_y = min_y.min(sub.y);
                max_y = max_y.max(sub.y + sub.h);
                let expect = amplify_region(&src, 48, 32, &sub, 8, 8, &o);
                let t = refine_tile(&src, 48, 32, &reg, cols, rows, col, row, 8, 8, &o);
                assert_eq!(t.len(), 64);
                assert!(
                    t.iter().zip(&expect).all(|(a, b)| a.to_bits() == b.to_bits()),
                    "tile ({col},{row}) is not the amplified hand-computed sub-region"
                );
                tiles.push(t);
            }
        }
        assert_eq!(min_x, reg.x, "tiles must start at the region's near x edge");
        assert_eq!(min_y, reg.y, "tiles must start at the region's near y edge");
        assert_eq!(max_x, reg.x + reg.w + 1.0, "tiles must reach the far x edge plus the overlap");
        assert_eq!(max_y, reg.y + reg.h + 1.0, "tiles must reach the far y edge plus the overlap");
        for i in 0..tiles.len() {
            for j in i + 1..tiles.len() {
                assert_ne!(tiles[i], tiles[j], "tiles {i} and {j} are identical");
            }
        }
    }

    /// Protects: an empty/zero-dimension source panics with a named
    /// assertion instead of indexing past the end of `src`.
    #[test]
    #[should_panic(expected = "non-empty source field")]
    fn an_empty_source_is_rejected_rather_than_read_out_of_bounds() {
        amplify_region(&[], 0, 0, &FloatRegion { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, 4, 4,
                       &AmplifyOpts::default());
    }

    /// Protects: `AmplifyOpts::default()` matches the reference's own
    /// hard-coded `amplifyRegion`/`refineTile` defaults.
    #[test]
    fn the_defaults_are_the_references_own() {
        let d = AmplifyOpts::default();
        assert_eq!(d.seed, 1234);
        assert_eq!(d.detail_freq, 1.0);
        assert_eq!(d.detail_amp, 0.14);
        assert_eq!(d.sea, 0.42);
        assert!(!d.ridged);
        assert_eq!(d.top, 1.0, "the reference's clip at 1");
    }

    /// Protects: Ruling DB item 3's `AmplifyOpts::top` in both refined paths
    /// (`amplify_region` and the tile path's `sample_elevation`): a summit
    /// ramp at 1.1-1.5 is clipped to 1 at the default top, and keeps its
    /// height and its spread (values both above and below 1.3) at a top of 1.6.
    #[test]
    fn a_higher_top_keeps_a_summit_above_one() {
        let (cw, ch) = (16usize, 16usize);
        // A ramp from 1.1 to 1.5 (detail fades on dead-flat ground).
        let src: Vec<f32> = (0..cw * ch).map(|i| 1.1 + 0.4 * (i % cw) as f32 / (cw - 1) as f32).collect();
        let r = FloatRegion { x: 2.0, y: 2.0, w: 8.0, h: 8.0 };
        let low = amplify_region(&src, cw, ch, &r, 32, 32, &AmplifyOpts::default());
        assert!(low.iter().all(|&v| v <= 1.0), "the default top clips at 1");
        let hi_opts = AmplifyOpts { top: 1.6, ..AmplifyOpts::default() };
        let hi = amplify_region(&src, cw, ch, &r, 32, 32, &hi_opts);
        let (mn, mx) = hi.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        assert!(mx > 1.3 && mn < 1.3, "detail on both sides of the summit: {mn}..{mx}");
        assert!(hi.iter().all(|&v| v > 1.0 && v <= 1.6));
        assert!(sample_elevation(&src, cw, ch, 5.5, 5.5, 3, &AmplifyOpts::default()) <= 1.0);
        assert!(sample_elevation(&src, cw, ch, 5.5, 5.5, 3, &hi_opts) > 1.0);
    }

    // -- EF-0: `sample_elevation` ------------------------------------------

    /// The tile path, composed exactly as `cartalith_engine::bake::
    /// pyramid_tile` composes it, so this module can check itself against it
    /// without depending on `cartalith-engine` (which depends on this crate).
    fn tile_path(
        src: &[f32],
        dims: (usize, usize),
        id: ChunkId,
        out: (usize, usize),
        opts: &AmplifyOpts,
    ) -> (Vec<f32>, FloatRegion) {
        let ((cw, ch), (tw, th), z) = (dims, out, id.z as i32);
        let n = pyramid_dims(z).cols as usize;
        let region = Region { x: 0, y: 0, w: cw - 1, h: ch - 1 }.to_float();
        let mut data =
            refine_tile(src, cw, ch, &region, n, n, id.col as usize, id.row as usize, tw, th, opts);
        let b = pyramid_tile_bounds(cw, ch, z, id.col, id.row);
        add_zoom_detail(&mut data, tw, th, src, cw, ch, &b, z, opts);
        (data, b)
    }

    /// The property [`sample_elevation`]'s doc comment states: the point query
    /// **is** the tile, not an approximation of it. Every texel, four levels,
    /// exact bits.
    ///
    /// This is also what makes the seam property structural rather than
    /// re-derived: two tiles sharing an edge map it to the same `(cx, cy)`,
    /// and a pure function of `(cx, cy)` cannot disagree with itself.
    #[test]
    fn sample_elevation_reproduces_the_tile_path_texel_for_texel() {
        // Protects: EF-0's whole reason to exist — the point query is
        // bit-identical to the tile path's own output at every texel it
        // maps to, across levels.
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 5);
        let opts = AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, ..Default::default() };
        let (tw, th) = (24usize, 18usize);
        let mut checked = 0usize;
        for z in [0i32, 2, 3, 5] {
            let n = pyramid_dims(z).cols;
            for (col, row) in [(0, 0), (n / 2, n / 3), (n - 1, n - 1)] {
                let (data, b) =
                    tile_path(&src, (cw, ch), ChunkId::new(z as u32, col, row), (tw, th), &opts);
                for oy in 0..th {
                    let cy = b.y + oy as f64 / (th as f64 - 1.0) * b.h;
                    for ox in 0..tw {
                        let cx = b.x + ox as f64 / (tw as f64 - 1.0) * b.w;
                        let q = sample_elevation(&src, cw, ch, cx, cy, z, &opts);
                        assert_eq!(
                            q.to_bits(),
                            data[oy * tw + ox].to_bits(),
                            "z={z} tile=({col},{row}) texel=({ox},{oy})"
                        );
                        checked += 1;
                    }
                }
            }
        }
        // A loop that silently ranged over nothing would pass every assertion
        // in it -- the empty-golden failure this project has been bitten by
        // four times.
        assert_eq!(checked, 4 * 3 * tw * th);
    }

    /// Ridged detail is a different noise family, and the correspondence has
    /// to hold for it too -- `opts.ridged` reaches only the first half of the
    /// body, so a transcription that switched families in the wrong place
    /// would still pass the fbm case above.
    #[test]
    fn the_texel_correspondence_holds_for_ridged_detail_too() {
        // Protects: the same tile/point-query identity under `opts.ridged`,
        // not just the default fbm path.
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 2);
        let opts = AmplifyOpts { seed: 77, ridged: true, ..Default::default() };
        let (tw, th) = (12usize, 10usize);
        let (data, b) = tile_path(&src, (cw, ch), ChunkId::new(4, 6, 9), (tw, th), &opts);
        for oy in 0..th {
            let cy = b.y + oy as f64 / (th as f64 - 1.0) * b.h;
            for ox in 0..tw {
                let cx = b.x + ox as f64 / (tw as f64 - 1.0) * b.w;
                assert_eq!(
                    sample_elevation(&src, cw, ch, cx, cy, 4, &opts).to_bits(),
                    data[oy * tw + ox].to_bits(),
                    "ridged texel=({ox},{oy})"
                );
            }
        }
    }

    /// Protects: `sample_elevation` is a pure function (same arguments,
    /// same bits, twice) and two adjacent tiles' shared coarse edge really
    /// does map to one value under it.
    #[test]
    fn sample_elevation_is_deterministic_and_seam_agreement_is_a_single_value() {
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 5);
        let opts = AmplifyOpts { seed: 4242, ..Default::default() };
        // Two adjacent tiles at level 4 share a coarse column exactly -- the
        // arithmetic `refine_tile`'s one-column overlap exists to produce.
        let l = pyramid_tile_bounds(cw, ch, 4, 6, 9);
        let r = pyramid_tile_bounds(cw, ch, 4, 7, 9);
        assert_eq!((l.x + l.w).to_bits(), r.x.to_bits(), "tile bounds do not share an edge");
        for k in 0..16 {
            let cy = l.y + k as f64 / 15.0 * l.h;
            let a = sample_elevation(&src, cw, ch, l.x + l.w, cy, 4, &opts);
            let b = sample_elevation(&src, cw, ch, r.x, cy, 4, &opts);
            assert_eq!(a.to_bits(), b.to_bits(), "shared edge disagrees at k={k}");
            // Same arguments twice: no cache, no RNG state, no global.
            assert_eq!(a.to_bits(), sample_elevation(&src, cw, ch, l.x + l.w, cy, 4, &opts).to_bits());
        }
    }

    /// Every argument changes the answer. Derived from the signature rather
    /// than from a guess at which ones matter -- the "derive the list from the
    /// definition" rule, applied to a pure function's own parameter list.
    #[test]
    fn every_input_of_the_point_query_moves_its_output() {
        // Protects: every argument of `sample_elevation` (coarse field, cx,
        // cy, z, and every `AmplifyOpts` field) actually participates in the
        // output, so none can be silently dropped by a future edit.
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 5);
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (cx, cy, z) = (20.3, 17.1, 6);
        let at = |c: &[f32], x: f64, y: f64, lvl: i32, op: &AmplifyOpts| {
            sample_elevation(c, cw, ch, x, y, lvl, op).to_bits()
        };
        let ref_bits = at(&src, cx, cy, z, &o);
        assert_ne!(ref_bits, at(&synthetic_field(cw, ch, 6), cx, cy, z, &o), "coarse field");
        assert_ne!(ref_bits, at(&src, cx + 0.05, cy, z, &o), "cx");
        assert_ne!(ref_bits, at(&src, cx, cy + 0.05, z, &o), "cy");
        assert_ne!(ref_bits, at(&src, cx, cy, z + 1, &o), "z");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { seed: 4243, ..o }), "seed");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { detail_freq: 1.3, ..o }), "detail_freq");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { detail_amp: 0.2, ..o }), "detail_amp");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { ridged: true, ..o }), "ridged");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { z_base: 4, ..o }), "z_base");
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { zoom_detail_k: 0.5, ..o }), "zoom_detail_k");
        // `sea` moves it because this point is land: raising sea past it
        // switches the hard cut in the zoom-detail half.
        assert_ne!(ref_bits, at(&src, cx, cy, z, &AmplifyOpts { sea: 0.99, ..o }), "sea");
    }

    /// Protects: `sample_elevation` stays finite (never panics or returns
    /// NaN from an index issue) for coordinates far outside the coarse
    /// field — `samp`'s clamp reaches through the whole point-query path.
    #[test]
    fn the_point_query_clamps_outside_the_field_rather_than_panicking() {
        let (cw, ch) = (16usize, 12usize);
        let src = synthetic_field(cw, ch, 0);
        let o = AmplifyOpts::default();
        // Far outside on both axes, both signs. `samp` clamps, so these are
        // the corner samples -- finite, not NaN, no index panic.
        for (x, y) in [(-500.0, -500.0), (5e3, 5e3), (-1.0, 7.0), (7.0, 99.0)] {
            assert!(sample_elevation(&src, cw, ch, x, y, 5, &o).is_finite(), "({x},{y})");
        }
    }

    /// Protects: `z_base_for_tile_size` returns the reference's own `2` at
    /// its 1024px default, holds the "halving the tile is one level
    /// deeper" relationship, and never sends a zero into `log2`.
    /// Literals, not `AmplifyOpts::default().z_base`: an assertion written
    /// against the constant holds for every value of it.
    #[test]
    fn z_base_for_tile_size_matches_the_references_own_quoted_sizes() {
        assert_eq!(z_base_for_tile_size(1024), 2, "the reference's own _lodTile");
        assert_eq!(z_base_for_tile_size(512), 3);
        assert_eq!(z_base_for_tile_size(256), 4, "cartalith_godot::lod_bridge::TILE_PX");
        assert_eq!(z_base_for_tile_size(2048), 1);
        // Halving the tile size is exactly one level deeper, by construction.
        for (a, b) in [(1024, 512), (512, 256), (256, 128)] {
            assert_eq!(z_base_for_tile_size(b) - z_base_for_tile_size(a), 1, "{a} -> {b}");
        }
        // A zero must not reach `log2`.
        assert!(z_base_for_tile_size(0) > 0);
    }

    /// Sum of squared discrete Laplacians over the interior — how much
    /// *curvature*, i.e. high-frequency content, a sampled patch carries.
    /// Bilinear interpolation is linear along both axes inside one coarse
    /// cell, so an upsample of coarse data scores ~0 on it by construction,
    /// which is the whole point of measuring it.
    fn curvature_energy(v: &[f64], n: usize) -> f64 {
        let mut e = 0.0;
        for y in 1..n - 1 {
            for x in 1..n - 1 {
                let lx = v[y * n + x - 1] - 2.0 * v[y * n + x] + v[y * n + x + 1];
                let ly = v[(y - 1) * n + x] - 2.0 * v[y * n + x] + v[(y + 1) * n + x];
                e += lx * lx + ly * ly;
            }
        }
        e
    }

    /// **The "not upsampling" proof.** At level 8 a tile's coarse footprint is
    /// a fifth of one cell wide, so bilinear upsampling of the coarse field
    /// over it is a plane broken only where it crosses a cell boundary — no
    /// new information, and the crease itself is the blockiness this design
    /// exists to remove. The refined field over the *same* footprint carries
    /// real structure instead, and this measures how much more.
    ///
    /// Three tiles, because one is one sample: the ratio has to be a property
    /// of the method, not of where the fixture happened to be read.
    #[test]
    fn deep_levels_add_real_structure_rather_than_smoothing_the_coarse_field() {
        // Protects: the "not upsampling" claim — a deep-zoom tile's curvature
        // energy is orders of magnitude above a bilinear upsample of the same
        // footprint, across several tiles.
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 5);
        let opts = AmplifyOpts { seed: 4242, ..Default::default() };
        let z = 8;
        let n = 48usize;
        // Three tiles over the dome's crown: land (base ~0.9, well clear of
        // `sea`) and sloped, so neither the hard sea cut nor the `relief <= 0`
        // skip is what is being measured.
        for (col, row) in [(147u32, 145u32), (155, 130), (60, 160)] {
            let b = pyramid_tile_bounds(cw, ch, z, col, row);
            assert!(b.w < 0.2 && b.h < 0.2, "footprint {}x{} is not sub-cell", b.w, b.h);
            let mut refined = vec![0.0f64; n * n];
            let mut upsampled = vec![0.0f64; n * n];
            for y in 0..n {
                let cy = b.y + y as f64 / (n as f64 - 1.0) * b.h;
                for x in 0..n {
                    let cx = b.x + x as f64 / (n as f64 - 1.0) * b.w;
                    refined[y * n + x] = sample_elevation(&src, cw, ch, cx, cy, z, &opts) as f64;
                    upsampled[y * n + x] = samp(&src, cw, ch, cx, cy);
                }
            }
            assert!(upsampled[0] > opts.sea, "tile ({col},{row}) is underwater: {}", upsampled[0]);
            let e_ref = curvature_energy(&refined, n);
            let e_up = curvature_energy(&upsampled, n);
            let max_dev =
                refined.iter().zip(&upsampled).map(|(a, b)| (a - b).abs()).fold(0.0f64, f64::max);
            println!(
                "tile ({col},{row}): curvature refined {e_ref:.4e} upsampled {e_up:.4e} \
                 ratio {:.0}x, max |refined-upsampled| {max_dev:.5}",
                e_ref / e_up
            );
            // Measured, this run, the three tiles in order:
            //
            //   (147,145)  refined 8.12e-4  upsampled 2.87e-6   ratio  283x
            //   (155,130)  refined 1.17e-3  upsampled 3.88e-29  ratio  ~1e25
            //   (60,160)   refined 2.21e-3  upsampled 8.32e-29  ratio  ~1e25
            //
            // with max deviations of 0.0110, 0.0111 and 0.0216. The two
            // regimes are worth naming rather than averaging away: a footprint
            // that crosses a coarse cell boundary gets a real crease from the
            // bilinear upsample (that crease **is** the blockiness), and one
            // that does not is planar to f64 rounding. The bars below clear
            // the weaker of each by roughly 2x, so they bite on a regression
            // without pinning one run's noise.
            assert!(
                e_ref > 100.0 * e_up,
                "tile ({col},{row}) added only {:.0}x the curvature of a bilinear upsample",
                e_ref / e_up
            );
            assert!(
                max_dev > 0.005,
                "tile ({col},{row}) deviates from the upsample by only {max_dev:.6}"
            );
        }
    }

    // ---- Ruling CI / FG: the kilometre-keyed zoom-detail law ------------

    /// The shipped law at one cell size, every other field literal (no cap,
    /// unit strength, so the tests read the shape and the amplitude directly).
    fn km_at(km_per_cell: f64) -> KmZoomDetail {
        KmZoomDetail { km_per_cell, octave_gain: 0.5, strength: 1.0, relief_gain: 11.0, relief_floor: 0.00153, max_octaves: 14, nyquist_margin: 1.0, cap: f64::INFINITY }
    }

    /// Octave `j` of the shape, written out independently of
    /// `km_zoom_detail_sum`: wavelength `(800/2048/2)·2^-j` km, rotation
    /// `(j+1)·2.399963229728653` rad, seed `seed + 7919 + 131·j`, weight
    /// `λ_j / L` (`H = 1`) for a cell `l_km` wide.
    fn octave_by_hand(gx_km: f64, gy_km: f64, j: i32, seed: i32, l_km: f64) -> f64 {
        let lam = 800.0 / 2048.0 / 2.0 * 2f64.powi(-j);
        let (s, c) = (((j + 1) as f64) * 2.399963229728653).sin_cos();
        let (u, v) = ((gx_km * c - gy_km * s) / lam, (gx_km * s + gy_km * c) / lam);
        (cartalith_noise::vnoise(u, v, seed + 7919 + 131 * j) - 0.5) * lam / l_km
    }

    /// Protects: the shipped constants, as literals (`MISTAKES.md`: assert a
    /// literal, never the constant against itself) -- changing one is a
    /// re-baseline of every app deep-zoom tile and must fail here first.
    #[test]
    fn the_km_law_ships_these_constants() {
        assert_eq!(ZOOM_DETAIL_REF_CELL_KM, 0.390625);
        assert_eq!(ZOOM_DETAIL_KM_OCTAVE_GAIN, 0.5);
        assert_eq!(ZOOM_DETAIL_KM_RELIEF_GAIN, 11.0);
        assert_eq!(ZOOM_DETAIL_KM_RELIEF_FLOOR, 0.00153);
        assert_eq!(ZOOM_DETAIL_KM_MAX_OCTAVES, 14);
        assert_eq!(ZOOM_DETAIL_KM_NYQUIST_MARGIN, 1.0);
        assert_eq!(ZOOM_DETAIL_KM_CAP_FRAC, 0.1);
        assert_eq!(ZOOM_DETAIL_KM_SHORE_BAND, 0.035);
        assert_eq!(KM_OCTAVE_ROTATION, 2.399963229728653);
        assert!(AmplifyOpts::default().km.is_none(), "the parity default must stay the reference schedule");
    }

    /// Protects: the octave shape. At the anchor cell size and level `z =
    /// z_base + 4`, the octaves at full weight are exactly `j = 0..=2` (top
    /// fade `1` at half a cell; bottom fade, with the one-octave margin, `1`
    /// down to `L/8`, i.e. `j = 2`, and `0` at `j = 3`), each weighted by its
    /// wavelength in cells. Mutating the gain, the rotation, the seed offset or
    /// either fade breaks it.
    #[test]
    fn the_km_shape_is_the_specified_octaves_at_the_anchor() {
        let opts = AmplifyOpts { seed: 4242, z_base: 2, ..Default::default() };
        let l = ZOOM_DETAIL_REF_CELL_KM;
        let km = km_at(l);
        for k in 0..64 {
            let (cx, cy) = (3.0 + k as f64 * 0.731, 5.0 + k as f64 * 0.377);
            let got = km_zoom_detail_sum(cx, cy, 6, &opts, &km);
            let want: f64 = (0..=2).map(|j| octave_by_hand(cx * l, cy * l, j, 4242, l)).sum();
            assert!((got - want).abs() < 1e-12, "k={k}: {got} vs {want}");
            assert!(got != 0.0);
        }
    }

    /// Protects: the kilometre keying of the lattice. The same ground on a
    /// grid half as fine, one level deeper (so the finest octave is the same
    /// wavelength in km), draws **the same octaves**: in kilometres (`shape ·
    /// L`) the two sums differ only by octave `j = −1` (one anchor cell),
    /// which the finer grid carries itself and the coarser one lacks below its
    /// own cell.
    #[test]
    fn the_same_km_gets_the_same_octaves_whatever_the_grid() {
        let opts = AmplifyOpts { seed: 99, z_base: 2, ..Default::default() };
        let (lf, lc) = (ZOOM_DETAIL_REF_CELL_KM, 2.0 * ZOOM_DETAIL_REF_CELL_KM);
        let (fine, coarse) = (km_at(lf), km_at(lc));
        for k in 0..40 {
            let (gx, gy) = (7.3 + k as f64 * 0.91, 2.1 + k as f64 * 0.53);
            let a = km_zoom_detail_sum(gx / lf, gy / lf, 6, &opts, &fine) * lf;
            let b = km_zoom_detail_sum(gx / lc, gy / lc, 7, &opts, &coarse) * lc;
            let only_coarse = octave_by_hand(gx, gy, -1, 99, 1.0);
            assert!((b - a - only_coarse).abs() < 1e-12, "k={k}: {b} - {a} != {only_coarse}");
        }
    }

    /// Protects: bar B1 of `WORLD_EMBOSS_FINDINGS_2026-11-15.md` by
    /// construction -- the octave's **slope per coarse cell** does not depend
    /// on kilometres per cell. At a fixed level offset below the top, the shape
    /// on a 0.195 km cell and on a 39 km cell (a 200 km and a 40 075 km map on
    /// 1 024 cells) has the same spread in cells, so the excursion per unit of
    /// relief is the same. Ruling CI's law grew 200x across the same span.
    #[test]
    fn the_km_shape_per_cell_is_width_invariant() {
        let opts = AmplifyOpts { seed: 5, z_base: 4, ..Default::default() };
        let spread = |l: f64, z: i32| {
            let km = km_at(l);
            let v: Vec<f64> = (0..4000).map(|k| km_zoom_detail_sum(1.0 + (k % 80) as f64 * 0.37, 1.0 + (k / 80) as f64 * 0.41, z, &opts, &km)).collect();
            let m = v.iter().sum::<f64>() / v.len() as f64;
            (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
        };
        // Level 10 on both: the window spans the same number of octaves in
        // cells; the lattice phases differ, so the spreads agree statistically.
        let (fine, world) = (spread(200.0 / 1024.0, 10), spread(40075.0 / 1024.0, 10));
        assert!(fine > 0.02 && world > 0.02, "premise: both draw octaves ({fine}, {world})");
        assert!((world / fine - 1.0).abs() < 0.15, "per-cell spread moved with L: {fine} vs {world}");
    }

    /// Protects: the amplitude law -- `strength · gain · ρ · max(R, floor) ·
    /// shape`, the floor, the roughness, the shore fade to exactly zero at sea
    /// level, and the soft cap. Literals.
    #[test]
    fn the_km_delta_scales_with_relief_roughness_and_fades_at_the_shore() {
        let km = KmZoomDetail { strength: 2.0, ..km_at(19.0) };
        let close = |x: f64, y: f64| (x - y).abs() < 1e-12;
        let up = 0.42 + 0.2; // well above the shore band
        assert!(close(km_detail_delta(0.1, 0.01, 1.0, up, 0.42, &km), 0.1 * 2.0 * 11.0 * 0.01));
        assert!(close(km_detail_delta(0.1, 0.01, 0.5, up, 0.42, &km), 0.1 * 2.0 * 11.0 * 0.005));
        // Flat ground reads the floor, not zero.
        assert!(close(km_detail_delta(0.1, 0.0, 1.0, up, 0.42, &km), 0.1 * 2.0 * 11.0 * 0.00153));
        // The same at any cell size: no `L` in the amplitude.
        assert!(close(km_detail_delta(0.1, 0.01, 1.0, up, 0.42, &km_at(0.2)), 0.1 * 11.0 * 0.01));
        assert_eq!(km_detail_delta(0.1, 0.01, 1.0, 0.42, 0.42, &km), 0.0);
        let mid = km_detail_delta(0.1, 0.01, 1.0, 0.42 + 0.0175, 0.42, &km);
        assert!(close(mid, 0.5 * 0.022), "half the band is smoothstep(0.5) = 0.5: {mid}");
        // The cap: a tanh, never past it, the identity far below it.
        let capped = KmZoomDetail { cap: 0.01, ..km };
        assert!(close(km_detail_delta(0.01, 0.05, 1.0, up, 0.42, &capped), 0.01 * (1.1f64).tanh())); // 0.01 · 2·11·0.05 = 0.011
        assert!(close(soft_cap(0.3, 1.0), 0.3f64.tanh()));
        assert!(close(soft_cap(-0.02, 0.01), -0.01 * 2.0f64.tanh()));
        assert_eq!(soft_cap(0.7, f64::INFINITY), 0.7);
        assert_eq!(soft_cap(0.7, 0.0), 0.7);
    }

    /// Protects: [`km_local_relief`] counts both the slope and the curvature,
    /// so a crest with no gradient still reads as relief. Literals.
    #[test]
    fn the_local_relief_is_the_gradient_and_the_curvature() {
        assert!((km_local_relief(0.003, 0.004, 0.0) - 0.005).abs() < 1e-15);
        assert!((km_local_relief(0.0, 0.0, -0.012) - 0.012).abs() < 1e-15);
        assert!((km_local_relief(0.003, 0.004, 0.012) - 0.013).abs() < 1e-15);
    }

    /// Protects: the texel body reads the coarse field's relief as the
    /// gradient **and** the four-neighbour curvature at the texel's own coarse
    /// coordinate. A field flat at `0.6` but for one cell raised by `0.02`: at
    /// that cell the half-differences are zero and the curvature is `−0.02`,
    /// so the excursion per unit of shape is `strength · gain · ρ · 0.02`; one
    /// cell further on, the gradient is `0.01` and the curvature `0.005`.
    #[test]
    fn the_texel_relief_is_read_from_the_coarse_field() {
        let (cw, ch) = (9usize, 9usize);
        let mut coarse = vec![0.6f32; cw * ch];
        coarse[4 * cw + 4] = 0.62;
        let opts = AmplifyOpts { sea: 0.42, km: Some(km_at(1.0)), ..Default::default() };
        let km = opts.km.unwrap();
        let rho = |_: f64, _: f64| 0.5;
        let r: Roughness = &rho;
        let per = |cx: f64, cy: f64| km_texel_delta(&coarse, cw, ch, cx, cy, 0.6, 1.0, &opts, &km, Some(r));
        let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
        // The spike's top: gradient 0, curvature -0.02 (f32 storage, hence 1e-6).
        assert!(close(per(4.0, 4.0), 11.0 * 0.5 * 0.02), "{}", per(4.0, 4.0));
        // Its east neighbour: gx = (0.6 - 0.62)/2 = -0.01, lap = (0.62 + 3·0.6)/4 - 0.6 = 0.005.
        assert!(close(per(5.0, 4.0), 11.0 * 0.5 * (0.0001f64 + 0.000025).sqrt()), "{}", per(5.0, 4.0));
        // Far from it: flat, the floor.
        assert!(close(per(1.0, 1.0), 11.0 * 0.5 * 0.00153));
    }

    /// Protects: what the law must never do -- move the coastline (no cell
    /// crosses sea level in either direction, the Ruling O guard kept) or
    /// touch water -- and that it does change land (so the identity is not
    /// between two nothings).
    #[test]
    fn the_km_law_never_moves_the_coast_and_does_move_land() {
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 5);
        let base = AmplifyOpts { seed: 4242, sea: 0.42, ..Default::default() };
        let km = AmplifyOpts { km: Some(KmZoomDetail { strength: 60.0, ..km_at(ZOOM_DETAIL_REF_CELL_KM) }), ..base };
        let (mut moved, mut total) = (0usize, 0usize);
        for z in [5i32, 7, 9] {
            let n = pyramid_dims(z).cols;
            for (col, row) in [(0, 0), (n / 2, n / 2), (n - 1, n - 1)] {
                let id = ChunkId::new(z as u32, col, row);
                let (r, _) = tile_path(&src, (cw, ch), id, (32, 32), &AmplifyOpts { z_base: z, ..base });
                let (k, _) = tile_path(&src, (cw, ch), id, (32, 32), &km);
                for (a, b) in r.iter().zip(&k) {
                    assert_eq!((*a as f64) < 0.42, (*b as f64) < 0.42, "a cell crossed sea level");
                    if (*a as f64) < 0.42 {
                        assert_eq!(a.to_bits(), b.to_bits(), "water was given octaves");
                    }
                    moved += (a != b) as usize;
                    total += 1;
                }
            }
        }
        assert!(moved * 4 > total, "the law moved only {moved} of {total} texels");
    }

    /// Protects: seam continuity under the km law, numerically, **with a
    /// roughness** -- two horizontally and two vertically adjacent tiles at the
    /// same level agree bit for bit on their shared edge, and the point query
    /// with the same roughness is the tile texel for texel (EF-0's identity).
    /// Also that the roughness reaches the tile (a premise, else the identity
    /// holds between two `ρ = 1` tiles).
    #[test]
    fn the_km_law_is_seam_exact_and_matches_the_point_query() {
        let (cw, ch) = (48usize, 32usize);
        let src = synthetic_field(cw, ch, 3);
        let opts = AmplifyOpts { seed: 7, sea: 0.42, km: Some(km_at(0.9)), ..Default::default() };
        let rho = |cx: f64, cy: f64| 0.3 + 0.6 * ((cx * 0.21).sin() * (cy * 0.17).cos()).abs();
        let rough: Roughness = &rho;
        let (tw, th) = (24usize, 18usize);
        let tile = |id: ChunkId| {
            let z = id.z as i32;
            let n = pyramid_dims(z).cols as usize;
            let region = Region { x: 0, y: 0, w: cw - 1, h: ch - 1 }.to_float();
            let mut data = refine_tile(&src, cw, ch, &region, n, n, id.col as usize, id.row as usize, tw, th, &opts);
            let b = pyramid_tile_bounds(cw, ch, z, id.col, id.row);
            add_zoom_detail_km_padded(&mut data, tw, th, 0, &src, cw, ch, &b, z, &opts, Some(rough));
            (data, b)
        };
        let mut edges = 0;
        for z in [3i32, 6] {
            // Land tiles near the field's centre (the synthetic dome).
            let c0 = pyramid_dims(z).cols / 2 - 1;
            let (l, bl) = tile(ChunkId::new(z as u32, c0, c0));
            let (r, _) = tile(ChunkId::new(z as u32, c0 + 1, c0));
            let (d, _) = tile(ChunkId::new(z as u32, c0, c0 + 1));
            let (plain, _) = tile_path(&src, (cw, ch), ChunkId::new(z as u32, c0, c0), (tw, th), &opts);
            assert!(plain.iter().zip(&l).any(|(a, b)| a != b), "z={z}: the roughness did not reach the tile");
            for y in 0..th {
                assert_eq!(l[y * tw + tw - 1].to_bits(), r[y * tw].to_bits(), "z={z} vertical seam row {y}");
                edges += 1;
            }
            for x in 0..tw {
                assert_eq!(l[(th - 1) * tw + x].to_bits(), d[x].to_bits(), "z={z} horizontal seam col {x}");
                edges += 1;
            }
            for oy in 0..th {
                for ox in 0..tw {
                    let cx = bl.x + ox as f64 / (tw as f64 - 1.0) * bl.w;
                    let cy = bl.y + oy as f64 / (th as f64 - 1.0) * bl.h;
                    assert_eq!(sample_elevation_rough(&src, cw, ch, cx, cy, z, &opts, Some(rough)).to_bits(), l[oy * tw + ox].to_bits());
                    assert_eq!(sample_elevation(&src, cw, ch, cx, cy, z, &opts).to_bits(), plain[oy * tw + ox].to_bits());
                }
            }
        }
        assert_eq!(edges, 2 * (tw + th));
    }

    /// Protects: the Ruling O guard inside the km branch, on its own. Land a
    /// hair above the shore band (`sea + 0.04`) under an absurd strength (the
    /// excursion is then many times the headroom): no texel may reach sea
    /// level, and some must have been pushed toward it (so the guard is what
    /// held).
    #[test]
    fn the_km_branch_keeps_the_sea_level_guard() {
        let (cw, ch) = (32usize, 32usize);
        let coarse: Vec<f32> = (0..cw * ch).map(|i| (0.46 + 0.01 * ((i % cw) as f64 * 0.7).sin()) as f32).collect();
        let opts = AmplifyOpts { seed: 3, sea: 0.42, km: Some(KmZoomDetail { strength: 20000.0, ..km_at(ZOOM_DETAIL_REF_CELL_KM) }), ..Default::default() };
        let b = FloatRegion { x: 4.0, y: 4.0, w: 6.0, h: 6.0 };
        let mut data = vec![0.46f32; 64 * 64];
        add_zoom_detail(&mut data, 64, 64, &coarse, cw, ch, &b, 9, &opts);
        assert!(data.iter().all(|&v| v as f64 > 0.42), "a land texel reached sea level");
        assert!(data.iter().filter(|&&v| (v as f64) < 0.441).count() > 50, "premise: the sum pushed land toward the sea");
    }
}
