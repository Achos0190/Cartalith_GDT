//! Real default-settings 2D map rendering, ported from the reference HTML's
//! material-synthesis renderer (`materialWeights`/`landColorCore`/
//! `seaColorCore`, reference HTML lines ~7560-8370) — replaces the previous
//! placeholder hypsometric tint (`color_for_height`/`hillshade` in
//! `lib.rs`, removed).
//!
//! Presentation-only math for the 2D map view: no simulation logic, no new
//! subsystem crate, matching `ARCHITECTURE.md`'s existing precedent of the
//! old placeholder living directly in `cartalith-godot`.
//!
//! Deliberately excludes every `state.viz.*`-gated stretch feature the
//! reference renderer supports, all `0`/`false` at JS's own defaults so
//! omitting them changes nothing about the *default* view.
//!
//! Excluded: splat texturing, rockSlope refinement, wetness darkening,
//! geology microtexture and dune ripples, procedural texture synthesis,
//! ridged-relief creases, curvature shading, the paint-brush biome/terrain
//! override, the "Painter" NPR block (watercolor/contours/ink/hachure),
//! multi-sun hillshade, AO/SVF/shadow fields, and the coast/river SDF
//! tinting plus vector river overlay (the last two depend on subsystems
//! this port hasn't built yet; the existing simple river channel-mask tint
//! in `lib.rs` stays as this port's stand-in for "rivers visible",
//! `MVP_SCOPE.md`'s point 2).
//!
//! Ported despite being extras: the `bioBlend` grey-desaturation blend
//! (0.90 default) and the edge haze fade, both unconditional in the
//! reference at its own default settings.
//!
//! ## `TerrainAppearance` (`TERRAIN_APPEARANCE_SCOPE.md` milestone 1, 2026-08-17)
//!
//! A real, owned, data-driven structure (below) replaces what used to be 26
//! bare module-level consts (19 material palettes, 6 water palettes,
//! `EXAG`/`SUN_AZ_DEG`/`BIO_BLEND`) — a behavior-preserving refactor only,
//! verified byte-identical against `golden_parity_render.rs` (unmodified).
//!
//! **Audit finding, corrected from the milestone's own initial assumption**:
//! there is no elevation-keyed colour *breakpoint ramp* anywhere in this
//! renderer, despite `TERRAIN_APPEARANCE_RESEARCH.md`'s MapTiler-style
//! mental model (`0m → green, 300m → yellow-green, ...`). Colour instead
//! comes from `material_weights()`, a continuous multi-input blend over
//! temperature/moisture/slope/relative-elevation/aspect/curvature that
//! produces six material *fractions* (snow/rock/sand/wetland/canopy/grass),
//! each material contributing its own colour via a **noise-jittered**
//! 3-stop micro-ramp (`ramp3`, selected by `tt` — a per-pixel texture-variety
//! value derived from coherent noise, not from elevation). Relative
//! elevation (`r` in `material_weights`) is one continuous input among
//! several `smoothstep` terms, not a lookup axis. So "the current hardcoded
//! elevation bands" the original milestone plan expected to re-encode as a
//! ramp don't exist in that shape — what *does* exist, and is real and
//! editable now, is this palette-and-constants table. A literal MapTiler-
//! style elevation ramp would be a genuinely new visual layer/mode to
//! design on top of (or blended with) this material model in a future
//! milestone, not a re-encoding of something already here.
//!
//! ## The atlas look (`TERRAIN_APPEARANCE_SCOPE.md` milestone 4, 2026-08-17)
//!
//! Three presentation stages toward `VISION.md`'s hand-drawn atlas target,
//! all gated to `0.0` in `js_reference()` and all early-returning on that
//! `0.0` rather than merely evaluating to a no-op:
//!
//! - **paper/vellum ground** (`paper_tone`) — a luminance-neutral parchment
//!   tone with fibre and ageing, applied in `cell_color` over land *and*
//!   sea so the whole map sits on one sheet;
//! - **forest stippling** (in `land_color`) — zero-mean coherent marks
//!   weighted by `material_weights`' own `canopy` fraction;
//! - **physical plate border** (`apply_border`) — paper margin plus a thick
//!   and a thin neatline.
//!
//! Hand-lettered settlement glyphs, the fourth element `VISION.md` names,
//! are deliberately *not* here: settlement markers are drawn by
//! `godot-project/map_overlay.gd`, not by this raster.

use cartalith_noise::vnoise;

type Rgb = (f64, f64, f64);

/// The renderer's editable colour data and shading constants — what used to
/// be 26 free-floating module consts, now one real, owned, inspectable
/// structure. `Default` reproduces today's exact values (pixel-identical
/// output); nothing here is wired to any UI/`#[func]` yet, matching
/// `cartalith-spatial`'s own "standalone, real, unintegrated" precedent
/// from earlier this session. See this module's own doc comment for why
/// this is a palette table, not an elevation-breakpoint ramp.
// `Clone` so a caller can hold *one* appearance value, hand it to
// `RenderCtx::with_appearance`, and still measure the plate frame with
// `border_cover` afterwards — rather than the raster and the overlays each
// constructing their own `default()` and hoping the two agree.
#[derive(Clone)]
pub struct TerrainAppearance {
    pub w_abyss: [Rgb; 3],
    pub w_deep: [Rgb; 3],
    pub w_shelf: [Rgb; 3],
    pub w_trop: [Rgb; 3],
    pub w_glac: [Rgb; 3],
    pub sand_beach: [Rgb; 3],
    pub sand_trop: [Rgb; 3],
    pub sand_desert: [Rgb; 3],
    pub sand_red: [Rgb; 3],
    pub grass_dry: [Rgb; 3],
    pub grass_temp: [Rgb; 3],
    pub grass_boreal: [Rgb; 3],
    pub grass_sav: [Rgb; 3],
    pub wood_temp: [Rgb; 3],
    pub wood_dense: [Rgb; 3],
    pub wood_boreal: [Rgb; 3],
    pub wood_trop: [Rgb; 3],
    pub rock_granite: [Rgb; 3],
    pub rock_sandstone: [Rgb; 3],
    pub rock_scree: [Rgb; 3],
    pub snow_seas: [Rgb; 3],
    pub snow_perm: [Rgb; 3],
    pub snow_glac: [Rgb; 3],
    pub wetland_temp: [Rgb; 3],
    pub wetland_trop: [Rgb; 3],
    pub mangrove: [Rgb; 3],
    /// `state.exag`'s literal default (reference HTML line 2260) — this
    /// port has no exposure/UI for it, fixed at the JS default.
    pub exag: f64,
    /// `state.sunAz`'s literal default (reference HTML line 2260).
    pub sun_az_deg: f64,
    /// Sun elevation angle. Was hardcoded `40.0` in two separate places
    /// (`shade`/`sea_shade_from`) before milestone 2 hoisted it here;
    /// `TERRAIN_APPEARANCE_RESEARCH.md` §14 lists "elevation angle" as a
    /// real control, so it belongs in the table, not inline.
    pub sun_alt_deg: f64,
    /// `state.bioBlend`'s literal default (reference HTML line 2260) — the
    /// grey-desaturation blend in `land_color` is unconditional at this
    /// value (`blend < 1`), not a `state.viz`-gated stretch feature.
    pub bio_blend: f64,

    // ---- Milestone 2 (`TERRAIN_APPEARANCE_SCOPE.md`): relief lighting ----
    /// Number of hillshade light directions, evenly spaced around the
    /// compass starting at `sun_az_deg` (`TERRAIN_APPEARANCE_RESEARCH.md`
    /// §14). **`1` reproduces the reference's exact single-sun shading**
    /// via a dedicated early-return path in `shade`, so
    /// `TerrainAppearance::js_reference()` stays bit-identical to JS.
    /// Higher counts reveal landforms whose ridgelines run *parallel* to
    /// the primary sun — invisible under single-light shading, which is
    /// the whole point of multidirectional relief.
    pub relief_lights: usize,
    /// How strongly the primary sun dominates the secondary lights, as the
    /// exponent `p = relief_directionality * 3` in each light's weight
    /// `((1 + cos θ)/2)^p` (θ = angular offset from the primary).
    /// `1.0` ≈ near-single-light, `0.0` = fully omnidirectional (which
    /// flattens relief completely — the classic multidirectional failure
    /// mode, avoided here by keeping the primary dominant).
    pub relief_directionality: f64,
    /// Ambient floor of the light curve (`light = ambient + gain·sh^0.85`).
    /// Multidirectional shading compresses `sh`'s range upward (fewer
    /// surfaces sit at zero), so the reference's `0.45` floor would wash
    /// the image out; the multi-light default lowers it and raises `gain`
    /// to restore comparable contrast. `TERRAIN_APPEARANCE_RESEARCH.md`
    /// §14's "ambient contribution".
    pub relief_ambient: f64,
    /// Gain of the light curve — see `relief_ambient`.
    pub relief_gain: f64,

    // ---- Milestone 2: ambient occlusion ----
    /// AO darkening strength (`TERRAIN_APPEARANCE_RESEARCH.md` §15).
    /// `0.0` disables AO entirely (and skips its precompute); the
    /// reference has no AO at its defaults, so `js_reference()` uses `0.0`.
    /// Deliberately modest — §15's explicit warning is "do not allow AO to
    /// turn terrain black", so `ao` is floored at `1 - ao_strength`.
    pub ao_strength: f64,
    /// Broad AO blur radius as a fraction of grid width, so the occlusion
    /// reads at a consistent *world* scale rather than a pixel scale across
    /// this port's 512²–8192² resolution range (same reasoning as
    /// `smooth_sea_h`'s own `gw/200` radius).
    pub ao_radius_frac: f64,

    // ---- Milestone 3 (`TERRAIN_APPEARANCE_SCOPE.md`): hydrology tint ----
    /// Ambient "near water" darkening/cooling strength
    /// (`TERRAIN_APPEARANCE_RESEARCH.md` §13). `0.0` disables it entirely
    /// (and skips the precompute); the reference has no such effect at its
    /// defaults, so `js_reference()` uses `0.0`. Deliberately a *tint*, not
    /// a repaint — §13 is explicit that "river rendering itself must remain
    /// a separate vector/layer system; do not paint rivers into the terrain
    /// colour raster", so this never approaches wetland-tint strength and
    /// never touches `material_weights` (the golden-verified fraction
    /// blend already has its own, independent TWI-driven wetland channel —
    /// this is a lighting-layer echo of "there's a lot of flow near here",
    /// not a second material classifier).
    pub hydro_wet_strength: f64,
    /// Blur radius (fraction of grid width) used to turn the per-cell flow
    /// field into a soft halo around channels rather than a hard one-cell
    /// outline — same reasoning as `ao_radius_frac`, but tighter, since a
    /// river corridor is a much narrower feature than a drainage basin.
    pub hydro_wet_radius_frac: f64,

    // ---- Milestone 4 (`TERRAIN_APPEARANCE_SCOPE.md`): the atlas look ----
    /// Strength of the paper/vellum ground (`VISION.md`'s "a paper/vellum
    /// ground with a physical border"). `0.0` disables it entirely and the
    /// whole stage early-returns, which is what `js_reference()` uses.
    ///
    /// The paper is applied as a **luminance-neutral multiplicative tone**
    /// over the finished image, land *and* sea alike — see `paper_tone`.
    pub paper_strength: f64,
    /// The parchment colour. Only its *hue/chroma* is used: `paper_tone`
    /// divides it by its own Rec.709 luma first, so raising
    /// `paper_strength` warms the sheet without dimming it.
    /// `TERRAIN_APPEARANCE_RESEARCH.md` §30's anti-list is explicit that the
    /// goal is legibility of real physical differences, and a straight
    /// multiply by an off-white would cost ~10% luma across the whole map
    /// for nothing.
    pub paper_tint: Rgb,
    /// Amplitude of the sheet's fibre/tooth — two fixed-cell-frequency
    /// coherent-noise octaves, one isotropic (tooth) and one stretched
    /// along Y (laid lines). Deterministic coherent noise only, per §16/§27;
    /// frequencies are chosen so a feature is never smaller than ~3 cells,
    /// the floor milestone 2's AO speckle regression established.
    pub paper_grain: f64,
    /// Amplitude of the broad age/stain mottle, expressed at *sheet* scale
    /// (a handful of blotches across the whole map) rather than cell scale,
    /// so it reads as the sheet being unevenly aged rather than as noise.
    pub paper_mottle: f64,
    /// How far the wash is muted toward a paper-coloured grey of the **same
    /// luminance**. This is the half of the paper ground that actually
    /// changes the tonal *feel*: pigment soaked into a sheet is never as
    /// chromatic as an emitted colour, and the tint alone (which only
    /// rotates hue) leaves a digital-looking saturated ocean.
    ///
    /// Luminance-preserving by construction, so it costs no relief or biome
    /// legibility — only chroma — which is the distinction
    /// `TERRAIN_APPEARANCE_RESEARCH.md` §30 draws when it says the goal is
    /// "not make the map more colourful" but "make the physical differences
    /// visually legible". Ordering between materials is untouched.
    pub paper_wash: f64,

    /// Forest stippling strength (`VISION.md`'s "forest stippling").
    /// `0.0` disables it; `js_reference()` uses `0.0`. Driven by
    /// `material_weights`' own `canopy` fraction — real data, not decorative
    /// noise — and applied as a **zero-mean** modulation so the canopy gains
    /// texture without being net-darkened (§30: no black valleys, no
    /// excessive contrast).
    pub stipple_strength: f64,
    /// Mark spacing as a fraction of grid width, so a stand of trees is a
    /// fixed *world* size rather than a fixed pixel size. Floored at 3.2
    /// cells inside `land_color` for exactly the reason `build_ao` floors
    /// its radii: below that a coherent-noise field is indistinguishable
    /// from per-pixel speckle, which is on §30's anti-list.
    pub stipple_scale_frac: f64,

    /// Width of the physical plate border as a fraction of grid width
    /// (floored at 10 cells). `0.0` disables it; `js_reference()` uses
    /// `0.0`. Drawn as a bare-paper margin carrying a thick and a thin
    /// neatline — the classic atlas plate edge.
    pub border_width_frac: f64,
    /// Neatline ink colour. Deliberately a warm sepia rather than black:
    /// pure black rules read as UI chrome, not as ink on a sheet.
    pub border_ink: Rgb,
}

impl Default for TerrainAppearance {
    fn default() -> Self {
        TerrainAppearance {
            w_abyss: [(8.0, 36.0, 58.0), (10.0, 45.0, 70.0), (18.0, 59.0, 89.0)],
            w_deep: [(16.0, 58.0, 87.0), (26.0, 75.0, 104.0), (42.0, 96.0, 122.0)],
            w_shelf: [(47.0, 118.0, 150.0), (76.0, 151.0, 182.0), (111.0, 179.0, 207.0)],
            w_trop: [(88.0, 184.0, 181.0), (121.0, 206.0, 197.0), (149.0, 222.0, 210.0)],
            w_glac: [(127.0, 174.0, 190.0), (165.0, 197.0, 207.0), (194.0, 215.0, 222.0)],
            sand_beach: [(200.0, 180.0, 138.0), (215.0, 195.0, 154.0), (227.0, 208.0, 167.0)],
            sand_trop: [(228.0, 212.0, 181.0), (239.0, 226.0, 197.0), (246.0, 234.0, 213.0)],
            sand_desert: [(201.0, 169.0, 104.0), (215.0, 182.0, 118.0), (226.0, 197.0, 138.0)],
            sand_red: [(168.0, 101.0, 61.0), (191.0, 119.0, 75.0), (208.0, 137.0, 92.0)],
            grass_dry: [(154.0, 138.0, 93.0), (176.0, 154.0, 106.0), (192.0, 171.0, 119.0)],
            grass_temp: [(127.0, 138.0, 86.0), (143.0, 155.0, 97.0), (162.0, 175.0, 112.0)],
            grass_boreal: [(102.0, 114.0, 79.0), (115.0, 128.0, 90.0), (133.0, 145.0, 107.0)],
            grass_sav: [(181.0, 160.0, 94.0), (198.0, 176.0, 109.0), (216.0, 193.0, 128.0)],
            wood_temp: [(53.0, 65.0, 40.0), (66.0, 82.0, 50.0), (85.0, 104.0, 67.0)],
            wood_dense: [(40.0, 51.0, 31.0), (50.0, 64.0, 38.0), (64.0, 80.0, 48.0)],
            wood_boreal: [(47.0, 56.0, 44.0), (57.0, 68.0, 53.0), (70.0, 84.0, 69.0)],
            wood_trop: [(29.0, 71.0, 37.0), (40.0, 96.0, 50.0), (52.0, 120.0, 63.0)],
            rock_granite: [(123.0, 117.0, 108.0), (147.0, 139.0, 128.0), (170.0, 161.0, 149.0)],
            rock_sandstone: [(167.0, 122.0, 87.0), (188.0, 141.0, 103.0), (208.0, 159.0, 118.0)],
            rock_scree: [(106.0, 102.0, 95.0), (122.0, 118.0, 110.0), (141.0, 137.0, 128.0)],
            snow_seas: [(217.0, 215.0, 210.0), (232.0, 231.0, 228.0), (245.0, 245.0, 245.0)],
            snow_perm: [(237.0, 240.0, 242.0), (245.0, 247.0, 248.0), (252.0, 252.0, 252.0)],
            snow_glac: [(184.0, 210.0, 219.0), (203.0, 224.0, 230.0), (221.0, 236.0, 239.0)],
            wetland_temp: [(58.0, 72.0, 52.0), (72.0, 88.0, 63.0), (89.0, 108.0, 78.0)],
            wetland_trop: [(46.0, 68.0, 44.0), (60.0, 86.0, 55.0), (76.0, 106.0, 68.0)],
            mangrove: [(38.0, 56.0, 42.0), (50.0, 72.0, 52.0), (64.0, 90.0, 65.0)],
            exag: 3.4,
            sun_az_deg: 315.0,
            sun_alt_deg: 40.0,
            bio_blend: 0.90,
            relief_lights: 6,
            relief_directionality: 0.62,
            relief_ambient: 0.34,
            relief_gain: 1.16,
            ao_strength: 0.28,
            ao_radius_frac: 0.012,
            hydro_wet_strength: 0.38,
            hydro_wet_radius_frac: 0.006,
            paper_strength: 0.85,
            paper_tint: (238.0, 228.0, 205.0),
            paper_grain: 0.050,
            paper_mottle: 0.045,
            paper_wash: 0.16,
            stipple_strength: 0.20,
            stipple_scale_frac: 0.0045,
            border_width_frac: 0.014,
            border_ink: (74.0, 61.0, 47.0),
        }
    }
}

impl TerrainAppearance {
    /// The reference HTML's exact default-settings shading: one sun, no
    /// ambient occlusion, the original light curve. Produces **bit-identical
    /// output to this renderer before milestone 2** — `relief_lights: 1`
    /// takes `shade`'s dedicated single-light early return, and
    /// `ao_strength: 0.0` skips the AO precompute and leaves `ao = 1.0`,
    /// which is literally what the code hardcoded before.
    ///
    /// This exists so `golden_parity_render.rs` keeps verifying real JS
    /// parity at its original `1e-4` tolerance rather than being
    /// re-baselined: `DECISIONS.md` §7a's principled-equivalence carve-out
    /// is scoped to paths where JS parity is *impractical* (GPU/f32), and
    /// says in as many words that the CPU rendering port "stays
    /// golden-verified against the JS engine and that work is not being
    /// discarded or devalued". A deliberate visual improvement is not the
    /// same thing as an impractical one, so the reference path stays
    /// tested — this also satisfies `TERRAIN_APPEARANCE_RESEARCH.md` §1.5's
    /// "preserve the current renderer as a fallback/reference
    /// implementation" literally rather than in spirit.
    // Used by `golden_parity_render.rs`, which compiles as its own target,
    // so the lib target alone sees it as unreachable.
    #[allow(dead_code)]
    pub fn js_reference() -> Self {
        TerrainAppearance {
            relief_lights: 1,
            relief_ambient: 0.45,
            relief_gain: 1.02,
            ao_strength: 0.0,
            hydro_wet_strength: 0.0,
            // Milestone 4: every atlas-presentation stage is off on the
            // reference path, and each one early-returns on its own `0.0`
            // rather than merely evaluating to a no-op — the same
            // "dedicated branch so parity can never drift on a float
            // reassociation" rule `relief_lights <= 1` already follows.
            paper_strength: 0.0,
            stipple_strength: 0.0,
            border_width_frac: 0.0,
            ..TerrainAppearance::default()
        }
    }
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let d = b - a;
    let d = if d == 0.0 { 1e-6 } else { d };
    let t = clamp01((x - a) / d);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
}

fn ramp3(p: &[Rgb; 3], t: f64) -> Rgb {
    let t = clamp01(t);
    if t < 0.5 { mix(p[0], p[1], t / 0.5) } else { mix(p[1], p[2], (t - 0.5) / 0.5) }
}

/// `boxH`/`boxV` (reference HTML lines 2511-2512) — separable box blur,
/// sliding-window accumulator. `f32` storage throughout (`dst` writes),
/// matching JS's `Float32Array` truncate-on-every-store semantics
/// (`cartalith-rust-conventions`).
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64, wrap: bool) {
    let norm = 1.0 / (2 * r + 1) as f64;
    for y in 0..h {
        let row = y * w;
        let mut acc = 0.0f64;
        let idx = |k: i64| -> usize {
            if wrap {
                (((k % w as i64) + w as i64) % w as i64) as usize
            } else {
                k.clamp(0, w as i64 - 1) as usize
            }
        };
        for k in -r..=r {
            acc += src[row + idx(k)] as f64;
        }
        for x in 0..w {
            dst[row + x] = (acc * norm) as f32;
            let o = idx(x as i64 - r);
            let i = idx(x as i64 + r + 1);
            acc += src[row + i] as f64 - src[row + o] as f64;
        }
    }
}

fn box_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64) {
    let norm = 1.0 / (2 * r + 1) as f64;
    let clamp_y = |k: i64| -> usize { k.clamp(0, h as i64 - 1) as usize };
    for x in 0..w {
        let mut acc = 0.0f64;
        for k in -r..=r {
            acc += src[clamp_y(k) * w + x] as f64;
        }
        for y in 0..h {
            dst[y * w + x] = (acc * norm) as f32;
            let o = clamp_y(y as i64 - r);
            let i = clamp_y(y as i64 + r + 1);
            acc += src[i * w + x] as f64 - src[o * w + x] as f64;
        }
    }
}

/// `smoothSeaH` (7966-7970) — two separable box passes, radius ∝
/// resolution, flatten the bathymetry into broad shelf/deep/abyss zones.
fn smooth_sea_h(src: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let rad = ((gw as f64 / 200.0).round() as i64).max(1);
    let mut a = src.to_vec();
    let mut b = vec![0f32; src.len()];
    for _ in 0..2 {
        box_h(&a, &mut b, gw, gh, rad, world);
        box_v(&b, &mut a, gw, gh, rad);
    }
    a
}

/// Separable box blur at `rad`, one pass (`smooth_sea_h` does two at a
/// fixed radius; AO wants a single pass at each of two radii instead).
fn blur_once(src: &[f32], gw: usize, gh: usize, rad: i64, world: bool) -> Vec<f32> {
    let mut b = vec![0f32; src.len()];
    let mut out = vec![0f32; src.len()];
    box_h(src, &mut b, gw, gh, rad, world);
    box_v(&b, &mut out, gw, gh, rad);
    out
}

/// Ambient occlusion over the heightfield (`TERRAIN_APPEARANCE_RESEARCH.md`
/// §15), returned as a per-cell multiplier in `[1 - ao_strength, 1]`.
///
/// Method: a **cavity map** — compare each cell's height against a blurred
/// version of the same field. Sitting below the local mean means sitting in
/// a hollow (valley floor, ravine, basin) and therefore seeing less sky;
/// sitting above it means a ridge or spur. This is a standard heightfield
/// AO approximation and it targets exactly what §15 asks for ("valleys,
/// ravines, canyon floors, depressions, terrain surrounded by steep
/// slopes") without the cost of real horizon ray-marching, which would be
/// far too expensive per-pixel on CPU at this port's 8192² ceiling.
///
/// Two radii are combined so both broad basins and narrow ravines register.
///
/// **Each scale is normalized by its own RMS over land cells**, which is
/// what makes this hold up across wildly different worlds — §32 warns that
/// appearance work flattering one terrain type often destroys another, and
/// a fixed magnitude threshold would do exactly that (a low-relief world
/// would get no AO at all, an alpine one would get crushed). Normalizing
/// against the world's own relief statistics gives a flat world the same
/// *relative* depth cue as a mountainous one. Deterministic: a pure
/// function of the field, per §27.
fn build_ao(field: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool, a: &TerrainAppearance) -> Vec<f32> {
    if a.ao_strength <= 0.0 {
        // The reference has no AO; `land_color` used a hardcoded `1.0`
        // before milestone 2, and this reproduces it with no work done.
        return vec![1f32; field.len()];
    }
    let r_broad = ((gw as f64 * a.ao_radius_frac).round() as i64).max(2);
    // Floored at 2: a radius-1 blur is close enough to the raw field that
    // the cavity signal picks up per-cell heightfield noise and renders as
    // speckle on flat ground — "random texture noise", on §30's anti-list.
    // Caught by a 3x zoom of the real A/B dump, not by reading the code.
    let r_fine = (r_broad / 3).max(2);
    let b_broad = blur_once(field, gw, gh, r_broad, world);
    let b_fine = blur_once(field, gw, gh, r_fine, world);

    // RMS of each scale's cavity signal, over land only — sea cells would
    // otherwise dominate the statistics with bathymetry that never gets
    // AO applied to it anyway (`land_color` is the only consumer).
    let (mut acc_b, mut acc_f, mut n) = (0.0f64, 0.0f64, 0usize);
    for i in 0..field.len() {
        if (field[i] as f64) < sea_level {
            continue;
        }
        let cb = (b_broad[i] - field[i]) as f64;
        let cf = (b_fine[i] - field[i]) as f64;
        acc_b += cb * cb;
        acc_f += cf * cf;
        n += 1;
    }
    if n == 0 {
        return vec![1f32; field.len()];
    }
    let rms_b = (acc_b / n as f64).sqrt().max(1e-9);
    let rms_f = (acc_f / n as f64).sqrt().max(1e-9);

    let mut out = vec![1f32; field.len()];
    for i in 0..field.len() {
        let cb = (b_broad[i] - field[i]) as f64 / rms_b;
        let cf = (b_fine[i] - field[i]) as f64 / rms_f;
        // Only concavity darkens. Convexity is left at 1.0 rather than
        // brightening ridges: brightening risks clipping into the
        // oversaturation §30 lists in its anti-list, and §15 describes AO
        // purely as a darkening term.
        let occ = clamp01(0.62 * cb + 0.38 * cf);
        out[i] = (1.0 - a.ao_strength * occ) as f32;
    }
    out
}

/// Ambient "near water" tint field (`TERRAIN_APPEARANCE_RESEARCH.md` §13),
/// returned as a per-cell `[0, 1]` strength (0 = no effect). `flow` is
/// `None` for a loaded save (`SAVEFILE_COMPAT.md` carries no flow field,
/// same fallback `cell_color`'s own TWI calculation already documents) —
/// this returns an all-zero field rather than guessing.
///
/// Method: log-compress flow the same way `cell_color`'s own TWI term
/// already does (`(flow / (gw*gh)).max(1e-4)`, so this stays on a
/// comparable scale to the existing hydrology math), min-max normalize so
/// it holds up across worlds with wildly different total flow (the same
/// reason `build_ao` normalizes by RMS rather than a fixed threshold —
/// §32's "flatters one terrain, destroys another" failure), then keep only
/// the top of that range with a smoothstep so ordinary hillside sheet-flow
/// doesn't tint the whole map, and blur it into a soft halo rather than a
/// hard one-cell channel outline.
fn build_hydro_wetness(flow: Option<&[f32]>, gw: usize, gh: usize, world: bool, a: &TerrainAppearance) -> Vec<f32> {
    let n = gw * gh;
    if a.hydro_wet_strength <= 0.0 {
        return vec![0f32; n];
    }
    let Some(flow) = flow else {
        return vec![0f32; n];
    };
    let denom = (gw * gh) as f64;
    let mut logf: Vec<f32> = flow.iter().map(|&f| (((f as f64) / denom).max(1e-4)).ln() as f32).collect();

    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for &v in &logf {
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }
    let range = (hi - lo).max(1e-6);
    for v in logf.iter_mut() {
        *v = smoothstep(0.55, 0.88, ((*v - lo) / range) as f64) as f32;
    }

    let rad = ((gw as f64 * a.hydro_wet_radius_frac).round() as i64).max(1);
    blur_once(&logf, gw, gh, rad, world)
}

/// The weighted multidirectional light table (`lx, ly, lz, weight`), built
/// once per render rather than re-deriving six sin/cos pairs per pixel.
/// Weights are normalized to sum to 1, so the combined shade stays on the
/// same `[0,1]` scale the single-light path produces.
fn build_lights(a: &TerrainAppearance) -> Vec<(f64, f64, f64, f64)> {
    let alt = a.sun_alt_deg.to_radians();
    let n = a.relief_lights.max(1);
    let p = (a.relief_directionality * 3.0).max(0.0);
    let mut out: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(n);
    let mut total = 0.0;
    for k in 0..n {
        let theta = (k as f64) * std::f64::consts::TAU / n as f64;
        let w = ((1.0 + theta.cos()) * 0.5).powf(p);
        let az = a.sun_az_deg.to_radians() + theta;
        out.push((alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin(), w));
        total += w;
    }
    let inv = if total > 0.0 { 1.0 / total } else { 1.0 };
    for l in &mut out {
        l.3 *= inv;
    }
    out
}

/// `seaShadeFrom` (8112-8121) — single-sun hillshade of the smoothed
/// bathymetry, edge-clamped (never wraps, even in world mode, matching the
/// reference exactly). Deliberately stays single-light even when land
/// shading is multidirectional: the bathymetry it reads is already heavily
/// smoothed (`smooth_sea_h`), so cross-lighting would only flatten it
/// further with nothing left to reveal.
fn sea_shade_from(hf: &[f32], gw: usize, gh: usize, appearance: &TerrainAppearance) -> Vec<f32> {
    let az = appearance.sun_az_deg.to_radians();
    let alt = appearance.sun_alt_deg.to_radians();
    let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let l = if x > 0 { hf[i - 1] } else { hf[i] } as f64;
            let r = if x + 1 < gw { hf[i + 1] } else { hf[i] } as f64;
            let u = if y > 0 { hf[i - gw] } else { hf[i] } as f64;
            let d = if y + 1 < gh { hf[i + gw] } else { hf[i] } as f64;
            let (nx, ny, nz) = (-(r - l) * appearance.exag, -(d - u) * appearance.exag, 1.0_f64);
            let il = 1.0 / nx.hypot(ny).hypot(nz);
            let (nx, ny, nz) = (nx * il, ny * il, nz * il);
            out[i] = (nx * lx + ny * ly + nz * lz).max(0.0) as f32;
        }
    }
    out
}

/// Everything the renderer needs about the last generated/loaded world.
/// `flow` is `None` for a loaded save (`SAVEFILE_COMPAT.md`'s save format
/// carries no flow field) — TWI-driven wetland placement falls back to the
/// driest case (`a` floored at its own `1e-4` minimum) rather than
/// guessing a value the save never stored.
pub struct RenderCtx<'a> {
    pub field: &'a [f32],
    pub temperature: &'a [f32],
    pub rainfall: &'a [f32],
    pub flow: Option<&'a [f32]>,
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    /// `smoothSeaH(field)` / `seaShadeFrom(_seaH)` (7966-8121) — `seaColor`
    /// reads these instead of the raw field/macro-shade whenever the app's
    /// default `state.mode==='biome'` map view is active (`renderNow`,
    /// 8422-8428), which is JS's own literal default (`mode:'biome'`, line
    /// 2260) so this isn't a stretch feature to skip: without it, shallow
    /// water reads with visible per-cell seabed noise the real app never
    /// shows. Computed once in `RenderCtx::new` rather than per cell.
    sea_h: Vec<f32>,
    sea_shade: Vec<f32>,
    /// Per-cell ambient-occlusion multiplier (`build_ao`). All `1.0` when
    /// `appearance.ao_strength == 0`, which is the reference's own state.
    ao: Vec<f32>,
    /// Per-cell "near water" tint strength (`build_hydro_wetness`). All
    /// `0.0` when `appearance.hydro_wet_strength == 0`, which is the
    /// reference's own state.
    hydro_wet: Vec<f32>,
    /// Precomputed weighted light directions (`build_lights`).
    lights: Vec<(f64, f64, f64, f64)>,
    /// The renderer's colour data/shading constants (`TerrainAppearance`'s
    /// own doc comment). Settable via `with_appearance` as of milestone 2 —
    /// still not wired to any UI/`#[func]` (that's `GUI_SHELL_SCOPE.md`'s
    /// own deferred terrain-appearance panel), but the golden-parity test
    /// uses it to pin the exact JS path.
    appearance: TerrainAppearance,
}

impl<'a> RenderCtx<'a> {
    // Used by `lib.rs`'s real render path; the test target (which calls
    // `with_appearance` directly) alone sees it as unreachable.
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn new(
        field: &'a [f32],
        temperature: &'a [f32],
        rainfall: &'a [f32],
        flow: Option<&'a [f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        lat_n: f64,
        lat_s: f64,
    ) -> Self {
        Self::with_appearance(field, temperature, rainfall, flow, gw, gh, sea_level, world, lat_n, lat_s, TerrainAppearance::default())
    }

    /// As `new`, but with caller-supplied appearance data. Pass
    /// `TerrainAppearance::js_reference()` for the reference HTML's exact
    /// default-settings output (what `golden_parity_render.rs` pins).
    #[allow(clippy::too_many_arguments)]
    pub fn with_appearance(
        field: &'a [f32],
        temperature: &'a [f32],
        rainfall: &'a [f32],
        flow: Option<&'a [f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        lat_n: f64,
        lat_s: f64,
        appearance: TerrainAppearance,
    ) -> Self {
        let sea_h = smooth_sea_h(field, gw, gh, world);
        let sea_shade = sea_shade_from(&sea_h, gw, gh, &appearance);
        let ao = build_ao(field, gw, gh, sea_level, world, &appearance);
        let hydro_wet = build_hydro_wetness(flow, gw, gh, world, &appearance);
        let lights = build_lights(&appearance);
        RenderCtx { field, temperature, rainfall, flow, gw, gh, sea_level, world, lat_n, lat_s, sea_h, sea_shade, ao, hydro_wet, lights, appearance }
    }

    fn h(&self, x: usize, y: usize) -> f64 {
        self.field[y * self.gw + x] as f64
    }

    /// `latAt` (reference HTML line 4965).
    fn lat_at(&self, y: usize) -> f64 {
        if self.world {
            90.0 - (y as f64 / (self.gh.max(2) - 1) as f64) * 180.0
        } else {
            self.lat_n + (y as f64 / (self.gh.max(2) - 1) as f64) * (self.lat_s - self.lat_n)
        }
    }

    /// `slopeAt` (7584) — X wraps in world mode, Y never wraps.
    fn slope_at(&self, x: usize, y: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let (xl, xr) = if self.world {
            ((x + gw - 1) % gw, (x + 1) % gw)
        } else {
            (if x > 0 { x - 1 } else { x }, if x + 1 < gw { x + 1 } else { x })
        };
        let (yu, yd) = (if y > 0 { y - 1 } else { y }, if y + 1 < gh { y + 1 } else { y });
        let l = self.h(xl, y);
        let r = self.h(xr, y);
        let u = self.h(x, yu);
        let d = self.h(x, yd);
        ((r - l) * 0.5).hypot((d - u) * 0.5)
    }

    /// `vignetteAt` (7585).
    fn vignette_at(&self, x: usize, y: usize) -> f64 {
        let vx = x as f64 / (self.gw.max(2) - 1) as f64 - 0.5;
        let vy = y as f64 / (self.gh.max(2) - 1) as f64 - 0.5;
        1.0 - smoothstep(0.34, 0.74, vx.hypot(vy)) * 0.42
    }

    /// `aspectFactor` (7590) — never wraps, matching the reference exactly.
    fn aspect_factor(&self, x: usize, y: usize) -> f64 {
        let gh = self.gh;
        let u = if y > 0 { self.h(x, y - 1) } else { self.h(x, y) };
        let d = if y + 1 < gh { self.h(x, y + 1) } else { self.h(x, y) };
        let dzdy = (d - u) * 0.5;
        let lat = self.lat_at(y);
        if lat >= 0.0 { -dzdy } else { dzdy }
    }

    /// `curvatureAt` (7599) — clamps on both axes; unlike `slope_at` this
    /// never wraps even in world mode, matching the reference exactly.
    fn curvature_at(&self, x: usize, y: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let xl = if x > 0 { x - 1 } else { x };
        let xr = if x + 1 < gw { x + 1 } else { x };
        let yu = if y > 0 { y - 1 } else { y };
        let yd = if y + 1 < gh { y + 1 } else { y };
        self.h(xl, y) + self.h(xr, y) + self.h(x, yu) + self.h(x, yd) - 4.0 * self.h(x, y)
    }

    /// `shadeFactor` (8342, "macro") / `shadeFactor2` (7642, "meso") share
    /// the same single-sun light vector; `step` is 1 for macro, 3 for meso.
    fn shade(&self, x: usize, y: usize, step: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let xl = x.saturating_sub(step);
        let xr = (x + step).min(gw - 1);
        let yu = y.saturating_sub(step);
        let yd = (y + step).min(gh - 1);
        let l = self.h(xl, y);
        let r = self.h(xr, y);
        let u = self.h(x, yu);
        let d = self.h(x, yd);
        let ex = self.appearance.exag / step as f64;
        let dzdx = (r - l) * ex;
        let dzdy = (d - u) * ex;
        let (nx, ny, nz) = (-dzdx, -dzdy, 1.0_f64);
        let il = 1.0 / nx.hypot(ny).hypot(nz);
        let (nx, ny, nz) = (nx * il, ny * il, nz * il);

        if self.appearance.relief_lights <= 1 {
            // Reference path, byte-for-byte as it was before milestone 2 —
            // kept as its own branch rather than falling out of the general
            // weighted sum, so JS parity can never drift on a float
            // reassociation. `js_reference()` takes this path.
            let az = self.appearance.sun_az_deg.to_radians();
            let alt = self.appearance.sun_alt_deg.to_radians();
            let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
            return (nx * lx + ny * ly + nz * lz).max(0.0);
        }

        // Multidirectional (`TERRAIN_APPEARANCE_RESEARCH.md` §14): each
        // light is clamped at the horizon *before* weighting, so a light
        // below the surface contributes nothing rather than subtracting.
        let mut sum = 0.0;
        for &(lx, ly, lz, w) in &self.lights {
            sum += w * (nx * lx + ny * ly + nz * lz).max(0.0);
        }
        sum
    }

    fn macro_shade(&self, x: usize, y: usize) -> f64 {
        self.shade(x, y, 1)
    }

    fn meso_shade(&self, x: usize, y: usize) -> f64 {
        self.shade(x, y, 3)
    }
}

/// `grassCol`/`forestCol`/`sandCol`/`rockCol`/`snowCol`/`wetlandCol`
/// (7632-7638).
fn grass_col(a: &TerrainAppearance, t: f64, m: f64, r: f64, tt: f64) -> Rgb {
    let c = if t < 4.0 {
        mix(ramp3(&a.grass_boreal, tt), ramp3(&a.grass_temp, tt), clamp01(m))
    } else if t > 22.0 && m < 0.4 {
        mix(ramp3(&a.grass_sav, tt), ramp3(&a.grass_dry, tt), clamp01(m * 2.0))
    } else {
        mix(ramp3(&a.grass_dry, tt), ramp3(&a.grass_temp, tt), clamp01(m))
    };
    let d = 1.0 - r * 0.16;
    (c.0 * d, c.1 * d, c.2 * d)
}

fn forest_col(a: &TerrainAppearance, t: f64, m: f64, tt: f64) -> Rgb {
    if t < 3.0 {
        ramp3(&a.wood_boreal, tt)
    } else if t > 20.0 && m > 0.45 {
        ramp3(&a.wood_trop, tt)
    } else if m > 0.62 {
        ramp3(&a.wood_dense, tt)
    } else {
        ramp3(&a.wood_temp, tt)
    }
}

fn sand_col(a: &TerrainAppearance, t: f64, m: f64, tt: f64) -> Rgb {
    if t > 24.0 && m < 0.1 { ramp3(&a.sand_red, tt) } else { ramp3(&a.sand_desert, tt) }
}

fn rock_col(a: &TerrainAppearance, t: f64, m: f64, r: f64, tt: f64) -> Rgb {
    if r > 0.82 {
        ramp3(&a.rock_scree, tt)
    } else if t > 18.0 && m < 0.32 {
        ramp3(&a.rock_sandstone, tt)
    } else {
        ramp3(&a.rock_granite, tt)
    }
}

fn snow_col(a: &TerrainAppearance, t: f64, tt: f64) -> Rgb {
    if t < -12.0 {
        ramp3(&a.snow_glac, tt)
    } else if t < -4.0 {
        ramp3(&a.snow_perm, tt)
    } else {
        ramp3(&a.snow_seas, tt)
    }
}

fn wetland_col(a: &TerrainAppearance, t: f64, mangrove: bool, tt: f64) -> Rgb {
    if mangrove { ramp3(&a.mangrove, tt) } else { ramp3(if t > 20.0 { &a.wetland_trop } else { &a.wetland_temp }, tt) }
}

/// `materialWeights` (7655-7707) — the six material fractions, Σ=1.
struct Weights {
    snow: f64,
    rock: f64,
    sand: f64,
    wetland: f64,
    canopy: f64,
    grass: f64,
    c: f64,
    meff: f64,
    is_mangrove: bool,
}

fn material_weights(t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64) -> Weights {
    let slope_str = (slope / 0.04).min(1.0);
    let asp_dry = clamp01(asp * slope_str * 0.22);
    let asp_wet = clamp01(-asp * slope_str * 0.12);

    let curv_norm = clamp01(curv.abs() * 300.0);
    let concave = if curv > 0.0 { curv_norm } else { 0.0 };
    let convex = if curv < 0.0 { curv_norm } else { 0.0 };

    let m_adj = clamp01(m - asp_dry + asp_wet + concave * 0.12);

    let fire = smoothstep(18.0, 26.0, t) * smoothstep(0.45, 0.15, m_adj) * smoothstep(0.08, 0.30, m_adj);

    let tn = clamp01((t + 5.0) / 35.0);
    let sl = (slope / 0.08).min(1.0);
    let sd0 = (-1.5 * sl).exp() * (0.4 + 0.6 * m_adj);
    let vp0 = m_adj.powf(0.7) * tn.powf(0.5) * sd0.max(0.0).powf(0.8);
    let c0 = 1.0 - (-2.0 * vp0).exp();

    let recycle = clamp01(0.1 + (t - 10.0).max(0.0) / 50.0);
    let meff = clamp01(m_adj + c0 * recycle * 0.5);
    let soil_d = (-1.5 * sl).exp() * (0.4 + 0.6 * meff);
    let vp_raw = meff.powf(0.7) * tn.powf(0.5) * soil_d.max(0.0).powf(0.8);
    let vp = vp_raw * (1.0 - fire * 0.40);
    let c = 1.0 - (-2.0 * vp).exp();

    let snow = smoothstep(3.0, -5.0, t);
    let mut bud = 1.0 - snow;

    let rexp = sl.powf(1.8) * (1.0 - vp) * (1.0 - meff) + convex * 0.25;
    let rock = clamp01(rexp * 0.8 + smoothstep(0.7, 0.95, r) * 0.35) * bud;
    bud -= rock;

    let sand = smoothstep(17.0, 26.0, t) * smoothstep(0.24, 0.05, meff) * (1.0 - vp * 0.7) * bud;
    bud -= sand;

    let mangrove_frac = smoothstep(18.0, 24.0, t) * smoothstep(0.08, 0.0, r) * smoothstep(0.10, 0.32, m_adj) * bud * 0.55;
    let wet_base = smoothstep(-1.0, 2.0, twi) * smoothstep(0.08, 0.28, m_adj) * smoothstep(0.06, 0.01, slope) * bud * 0.50;
    let wet_curv = concave * smoothstep(0.08, 0.28, m_adj) * bud * 0.22;
    let wetland = bud.min(mangrove_frac.max(wet_base + wet_curv));
    bud -= wetland;
    let is_mangrove = mangrove_frac > wet_base + wet_curv;

    let canopy = c * bud;
    bud -= canopy;
    let grass = bud.max(0.0);

    Weights { snow, rock, sand, wetland, canopy, grass, c, meff, is_mangrove }
}

/// `bioJitter` (7715-7719) at `state.viz.sharpBiomes`'s default (`true`).
fn bio_jitter(x: usize, y: usize, gw: usize) -> f64 {
    let (xf, yf) = (x as f64, y as f64);
    let gw = gw as f64;
    0.6 * vnoise(xf / gw * 44.0, yf / gw * 44.0, 31) + 0.4 * vnoise(xf / gw * 150.0, yf / gw * 150.0, 33)
}

/// `landColorCore`'s unconditional core (7720-7960): eco-jitter, the
/// six-material blend with canopy understory shadow, the beach rim, fine
/// noise grain, multi-scale hillshade, the `bioBlend` grey blend, the edge
/// haze fade, and the final `ao * vignette` multiply (7959-7960 — easy to
/// miss since it sits after the whole gated "Painter" NPR block, but is
/// itself unconditional; `ao` is fixed at `1.0` here, matching this port's
/// AO/SVF/shadow fields all being off). Every other `state.viz.*`-gated
/// extra is omitted — see this module's doc comment.
#[allow(clippy::too_many_arguments)]
fn land_color(appearance: &TerrainAppearance, t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64, sh: f64, sh_m: f64, vig: f64, ao: f64, hydro_wet: f64, x: usize, y: usize, gw: usize, gh: usize) -> Rgb {
    let n_low = vnoise(x as f64 * 0.06, y as f64 * 0.06, 11);
    let n_hi = vnoise(x as f64 * 96.0 / gw as f64, y as f64 * 96.0 / gw as f64, 23);
    let n_bio = bio_jitter(x, y, gw);

    let te = t + (n_bio - 0.5) * 7.0 + (n_low - 0.5) * 2.5;
    let me = clamp01(m + (n_bio - 0.5) * 0.15 + (n_hi - 0.5) * 0.05);
    let twi_e = twi + (n_bio - 0.5) * 0.7;
    let asp_e = asp * (1.0 + (n_low - 0.5) * 0.3);

    let w = material_weights(te, me, slope, r, twi_e, asp_e, curv);
    let tt = clamp01(0.5 + (n_low - 0.5) * 1.1 + (n_hi - 0.5) * 0.5);

    let mut c = (0.0, 0.0, 0.0);
    let add = |c: &mut Rgb, m: Rgb, w: f64| {
        c.0 += m.0 * w;
        c.1 += m.1 * w;
        c.2 += m.2 * w;
    };
    add(&mut c, snow_col(appearance, te, tt), w.snow);
    add(&mut c, rock_col(appearance, te, me, r, tt), w.rock);
    add(&mut c, sand_col(appearance, te, me, tt), w.sand);
    add(&mut c, wetland_col(appearance, te, w.is_mangrove, tt), w.wetland);

    if w.canopy > 0.0 {
        let understory = smoothstep(0.70, 0.94, w.c) * w.canopy * 0.28;
        c.0 += 20.0 * understory;
        c.1 += 43.0 * understory;
        c.2 += 25.0 * understory;
        add(&mut c, forest_col(appearance, te, w.meff, tt), w.canopy - understory);
    }

    add(&mut c, grass_col(appearance, te, me, r, tt), w.grass);

    let beach_t = smoothstep(0.03, 0.0, r) * 0.6;
    if beach_t > 0.0 {
        let bc = ramp3(if te > 22.0 { &appearance.sand_trop } else { &appearance.sand_beach }, tt);
        c.0 += (bc.0 - c.0) * beach_t;
        c.1 += (bc.1 - c.1) * beach_t;
        c.2 += (bc.2 - c.2) * beach_t;
    }

    let g = (n_hi - 0.5) * 9.0;
    c.0 += g;
    c.1 += g;
    c.2 += g;

    // Milestone 4: forest stippling (`VISION.md`). Texture over canopy, from
    // `material_weights`' own `canopy` fraction — real data, not decorative
    // noise laid over "wherever looks green".
    //
    // Three things make this survive `TERRAIN_APPEARANCE_RESEARCH.md` §30:
    // the gate is a `smoothstep` (no hard biome borders); the mark field is
    // deterministic coherent noise floored at 3.2 cells per mark (§16/§27,
    // and the same speckle floor milestone 2's AO needed); and the
    // modulation is **zero-mean** — marks darken, gaps lighten by the same
    // amount — so a forest gains texture without the whole canopy going
    // darker, which would read as excessive contrast rather than as ink.
    if appearance.stipple_strength > 0.0 && w.canopy > 0.0 {
        let gate = smoothstep(0.30, 0.72, w.canopy);
        if gate > 0.0 {
            let per_mark = (appearance.stipple_scale_frac * gw as f64).max(4.0);
            let f = 1.0 / per_mark;
            let (xf, yf) = (x as f64, y as f64);
            // Rotate the sampling lattice (~34°) and domain-warp it. Value
            // noise sampled on the axis-aligned grid at a few cells per
            // feature reads as a regular halftone screen — caught by
            // looking at a 6x crop of the first version of this, the same
            // way milestone 2's AO speckle was. Rotation breaks the axis
            // alignment; the warp breaks the lattice regularity, so the
            // marks clump the way drawn stippling does.
            let (rx, ry) = (xf * 0.8290 + yf * 0.5592, -xf * 0.5592 + yf * 0.8290);
            let wx = (vnoise(rx * f * 0.42, ry * f * 0.42, 75) - 0.5) * 1.8;
            let wy = (vnoise(ry * f * 0.42, rx * f * 0.42, 77) - 0.5) * 1.8;
            let n = 0.62 * vnoise(rx * f + wx, ry * f + wy, 71) + 0.38 * vnoise(ry * f * 2.13 - wy, rx * f * 2.13 + wx, 73);
            // Signed, then pushed toward its extremes (exponent < 1) so the
            // field clumps into discrete marks instead of reading as a soft
            // wobble. Symmetric about zero, hence zero-mean.
            let d = (n - 0.5) * 2.0;
            let d = d.signum() * d.abs().powf(0.65);
            let s = appearance.stipple_strength * gate * d;
            // Marks sit as a slightly deeper, greener ink; gaps as lighter
            // wash — the red channel moves most, so the texture is a hue
            // modulation as well as a value one.
            c.0 *= 1.0 - s * 1.15;
            c.1 *= 1.0 - s * 0.95;
            c.2 *= 1.0 - s * 1.05;
        }
    }

    let sh_micro = clamp01(sh + (n_hi - 0.5) * 0.20);
    let sh_combined = 0.40 * sh + 0.40 * sh_m + 0.20 * sh_micro;
    let light = appearance.relief_ambient + appearance.relief_gain * clamp01(sh_combined).powf(0.85);
    let mut l = (c.0 * light, c.1 * light, c.2 * light);
    if appearance.bio_blend < 1.0 {
        let grey = 185.0 * light;
        l = (grey + (l.0 - grey) * appearance.bio_blend, grey + (l.1 - grey) * appearance.bio_blend, grey + (l.2 - grey) * appearance.bio_blend);
    }

    let dx = x as f64 / gw as f64 - 0.5;
    let dy = y as f64 / gh as f64 - 0.5;
    let haze = clamp01(dx.hypot(dy) * 1.9).powf(2.2) * 0.18;
    let mut l = (l.0 + (208.0 - l.0) * haze, l.1 + (218.0 - l.1) * haze, l.2 + (230.0 - l.2) * haze);

    // Milestone 3: ambient "near water" tint (`TERRAIN_APPEARANCE_RESEARCH.md`
    // §13) — a soft pull toward a cool, muted green-grey near high flow
    // accumulation, deliberately short of `wetland_temp`'s own darkest stop
    // so it never reads as a second, competing material classification (that
    // channel already exists, independently, inside `material_weights`).
    // `hydro_wet` is `0.0` whenever `hydro_wet_strength == 0` (including
    // `js_reference()`), so this is a no-op on the pinned JS-parity path.
    if hydro_wet > 0.0 {
        let wet = hydro_wet * appearance.hydro_wet_strength;
        let target = (50.0, 68.0, 74.0);
        l = (l.0 + (target.0 - l.0) * wet, l.1 + (target.1 - l.1) * wet, l.2 + (target.2 - l.2) * wet);
    }

    // `ao * vignette` (7959-7960). `ao` was a hardcoded `1.0` before
    // milestone 2 (the reference's AO/SVF/shadow fields are all off at its
    // defaults); it now carries `build_ao`'s cavity map, and is still
    // exactly `1.0` under `js_reference()`.
    let k = ao * vig;
    (l.0 * k, l.1 * k, l.2 * k)
}

/// `seaColorCore` (8122-8130).
fn sea_color_core(appearance: &TerrainAppearance, depth: f64, t: f64, n_low: f64, sh: f64, vig: f64) -> Rgb {
    let mut wc = if depth < 0.2 {
        ramp3(&appearance.w_shelf, n_low)
    } else if depth < 0.55 {
        mix(ramp3(&appearance.w_shelf, n_low), ramp3(&appearance.w_deep, n_low), (depth - 0.2) / 0.35)
    } else {
        mix(ramp3(&appearance.w_deep, n_low), ramp3(&appearance.w_abyss, n_low), (depth - 0.55) / 0.45)
    };
    if t > 22.0 {
        wc = mix(wc, ramp3(&appearance.w_trop, n_low), smoothstep(22.0, 28.0, t) * (1.0 - depth) * 0.8);
    }
    if t < 5.0 {
        wc = mix(wc, ramp3(&appearance.w_glac, n_low), smoothstep(5.0, -3.0, t) * 0.7);
    }
    if t < -2.0 {
        wc = mix(wc, (226.0, 233.0, 239.0), clamp01((-2.0 - t) / 6.0) * 0.85);
    }
    let surf = smoothstep(0.03, 0.0, depth);
    if surf > 0.0 {
        wc = mix(wc, (176.0, 214.0, 221.0), surf * 0.5);
    }
    let tex = (n_low - 0.5) * 5.0;
    let sh2 = 0.82 + 0.18 * clamp01(sh);
    ((wc.0 + tex) * sh2 * vig, (wc.1 + tex) * sh2 * vig, (wc.2 + tex) * sh2 * vig)
}

/// The paper/vellum ground as a **per-channel multiplicative tone** around
/// `1.0` (`TERRAIN_APPEARANCE_SCOPE.md` milestone 4, `VISION.md`'s
/// "paper/vellum ground"). Returned rather than applied so that both the
/// map wash (`apply_paper`) and the plate margin (`apply_border`) sit on
/// the *same* sheet — the fibre has to run continuously under both, or the
/// border reads as a separate graphic pasted on top.
///
/// Three deliberate properties:
///
/// 1. **Luminance-neutral tint.** `paper_tint` is divided by its own
///    Rec.709 luma, so the parchment shifts hue (warmer reds, muted blues)
///    without darkening the image. A straight multiply by an off-white
///    would cost ~10% luma everywhere and flatten exactly the relief and
///    biome legibility milestones 2 and 3 just bought —
///    `TERRAIN_APPEARANCE_RESEARCH.md` §30's whole point.
/// 2. **Grain frequencies fixed in *cell* units**, never finer than ~3
///    cells per feature (0.31 and 0.27 cycles/cell here). Milestone 2's AO
///    speckle regression is the precedent: coherent noise at ~1 cell is
///    indistinguishable from the "random texture noise" §30 forbids.
/// 3. **Mottle at *sheet* scale** (5 and 13 features across the map), so
///    ageing reads as a property of the sheet rather than of the terrain.
///
/// Deterministic — pure `vnoise` of the cell coordinates, per §27.
fn paper_tone(a: &TerrainAppearance, x: usize, y: usize, gw: usize) -> Rgb {
    if a.paper_strength <= 0.0 {
        return (1.0, 1.0, 1.0);
    }
    let (xf, yf, gwf) = (x as f64, y as f64, gw as f64);
    let t = a.paper_tint;
    let luma = (0.2126 * t.0 + 0.7152 * t.1 + 0.0722 * t.2).max(1e-6);
    let t = (t.0 / luma, t.1 / luma, t.2 / luma);

    let tooth = vnoise(xf * 0.31, yf * 0.31, 61);
    // Stretched along Y: laid lines. Cheap, and it's the single cue that
    // reads as "sheet" rather than "noise overlay" at a glance.
    let laid = vnoise(xf * 0.27, yf * 0.075, 63);
    let grain = ((tooth - 0.5) * 0.55 + (laid - 0.5) * 0.45) * 2.0;
    let mottle = 0.65 * vnoise(xf / gwf * 5.0, yf / gwf * 5.0, 65) + 0.35 * vnoise(xf / gwf * 13.0, yf / gwf * 13.0, 67);

    let v = 1.0 + grain * a.paper_grain + (mottle - 0.5) * 2.0 * a.paper_mottle;
    let s = a.paper_strength;
    (1.0 + (t.0 * v - 1.0) * s, 1.0 + (t.1 * v - 1.0) * s, 1.0 + (t.2 * v - 1.0) * s)
}

/// Lay the finished colour onto the sheet: the parchment tint (a pure hue
/// rotation, see `paper_tone`) followed by the muting toward a
/// luminance-matched paper grey (`paper_wash`). Both stages leave luminance
/// alone, so relief and biome legibility are untouched — only chroma moves.
fn apply_paper(a: &TerrainAppearance, c: Rgb, tone: Rgb) -> Rgb {
    if a.paper_strength <= 0.0 {
        return c;
    }
    let c = (c.0 * tone.0, c.1 * tone.1, c.2 * tone.2);
    if a.paper_wash <= 0.0 {
        return c;
    }
    let y = 0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2;
    let t = a.paper_tint;
    let tl = (0.2126 * t.0 + 0.7152 * t.1 + 0.0722 * t.2).max(1e-6);
    let grey = (t.0 / tl * y, t.1 / tl * y, t.2 / tl * y);
    mix(c, grey, a.paper_wash * a.paper_strength)
}

/// Width of the plate frame **in cells**, or `0.0` when there is no frame.
/// The single source of truth for the frame's geometry: `apply_border`
/// draws with it, `border_cover` measures with it, and `WorldGen::
/// get_border_inset_frac` hands it across the gdext boundary so
/// `map_overlay.gd` can keep its markers inside the neatline. Anything that
/// re-derives `0.014 * gw` by hand is a second source of truth and will
/// drift the first time the frame is retuned.
pub fn border_width_cells(a: &TerrainAppearance, gw: usize) -> f64 {
    if a.border_width_frac <= 0.0 {
        return 0.0;
    }
    (a.border_width_frac * gw as f64).max(10.0)
}

/// How much of this cell the plate frame covers: `0.0` anywhere the frame
/// has no influence at all, ramping to `1.0` under the bare-paper margin,
/// using the *same* soft edge `apply_border` composites with (so a caller
/// that fades by `1 - cover` lines up with the frame exactly rather than
/// approximately).
///
/// This exists for the two systems that draw **over** the finished raster
/// and would otherwise paint on what is supposed to read as blank paper:
/// `lib.rs`'s river channel tint and its territory/province overlays.
/// Returns `0.0` everywhere when `border_width_frac == 0.0`, so every
/// caller is a bit-exact no-op on the `js_reference()` path.
// Called from `lib.rs`, which the test targets (they compile `render.rs`
// standalone) don't include — same situation as `js_reference()` in reverse.
#[allow(dead_code)]
pub fn border_cover(a: &TerrainAppearance, x: usize, y: usize, gw: usize, gh: usize) -> f64 {
    let w = border_width_cells(a, gw);
    if w <= 0.0 {
        return 0.0;
    }
    let d = (x.min(gw - 1 - x)).min(y.min(gh - 1 - y)) as f64;
    if d >= w {
        return 0.0;
    }
    1.0 - smoothstep(w - 1.5, w, d)
}

/// The physical plate border (`VISION.md`'s "physical border"): a bare-paper
/// margin carrying a thick and a thin neatline, the classic atlas plate
/// edge. Pure presentation — it composites over the finished colour and
/// reads no world data at all.
///
/// The ink density is modulated by low-frequency coherent noise along the
/// rule, so the lines read as drawn rather than as a CSS box. Widths are
/// floored in absolute cells so the frame survives this port's 512²–8192²
/// resolution range without the two rules merging at the small end.
fn apply_border(a: &TerrainAppearance, c: Rgb, tone: Rgb, x: usize, y: usize, gw: usize, gh: usize) -> Rgb {
    let w = border_width_cells(a, gw);
    if w <= 0.0 {
        return c;
    }
    let dx = x.min(gw - 1 - x) as f64;
    let dy = y.min(gh - 1 - y) as f64;
    let d = dx.min(dy);
    if d >= w {
        return c;
    }

    // Bare sheet, carrying the same fibre as the map itself.
    let sheet = (a.paper_tint.0 * tone.0, a.paper_tint.1 * tone.1, a.paper_tint.2 * tone.2);
    // Soft over ~1.5 cells: the wash stopping at the neatline, not a
    // hard-aliased cut — §30's "artificial outlines".
    let cover = 1.0 - smoothstep(w - 1.5, w, d);
    let mut out = mix(c, sheet, cover);

    let rule = |centre: f64, half: f64| 1.0 - smoothstep(half - 0.75, half + 0.75, (d - centre).abs());
    let thick = rule(0.34 * w, (0.075 * w).max(2.0));
    let thin = rule(0.80 * w, (0.028 * w).max(0.9));
    let mut ink = thick.max(thin);
    if ink > 0.0 {
        // Hand-drawn density variation along the line (deterministic, §27).
        let along = if dx < dy { y as f64 } else { x as f64 };
        ink *= 0.80 + 0.20 * vnoise(along * 0.05, d * 0.4, 69);
        let ic = (a.border_ink.0 * tone.0, a.border_ink.1 * tone.1, a.border_ink.2 * tone.2);
        out = mix(out, ic, ink);
    }
    out
}

/// Top-level per-cell colour, `[0,1]` per channel — `isWater(v) ?
/// seaColor(...) : surfaceColor(...)` (`debugBaseColor`'s `'biome'`
/// branch, 8204; the main renderer's own default mode).
pub fn cell_color(ctx: &RenderCtx, x: usize, y: usize) -> (f64, f64, f64) {
    let i = y * ctx.gw + x;
    let h = ctx.h(x, y);
    let t = ctx.temperature[i] as f64;

    let (r, g, b) = if h < ctx.sea_level {
        // `seaColor` (8277-8281) — reads the smoothed bathymetry/shade
        // (`ctx.sea_h`/`ctx.sea_shade`), not the raw field/macro-shade;
        // see `RenderCtx::new`'s doc comment on why that's the real
        // default, not a stretch feature.
        let hs = ctx.sea_h[i] as f64;
        let shw = ctx.sea_shade[i] as f64;
        let depth = if ctx.sea_level <= 0.0 { 0.0 } else { clamp01((ctx.sea_level - hs) / ctx.sea_level) };
        let n_low = vnoise(x as f64 * 25.6 / ctx.gw as f64, y as f64 * 25.6 / ctx.gw as f64, 5);
        sea_color_core(&ctx.appearance, depth, t, n_low, shw, ctx.vignette_at(x, y))
    } else {
        // `surfaceColor` (8145-8196), unconditional parts only.
        let m = ctx.rainfall[i] as f64;
        let r_frac = if (1.0 - ctx.sea_level) <= 0.0 { 0.0 } else { (h - ctx.sea_level) / (1.0 - ctx.sea_level) };
        let slope = ctx.slope_at(x, y);
        let flow = ctx.flow.map(|f| f[i] as f64).unwrap_or(0.0);
        let a = (flow / (ctx.gw * ctx.gh) as f64).max(1e-4);
        let beta = slope.max(0.002);
        let twi = (a / beta).ln();
        let asp = ctx.aspect_factor(x, y);
        let curv = ctx.curvature_at(x, y);
        land_color(&ctx.appearance, t, m, slope, r_frac, twi, asp, curv, ctx.macro_shade(x, y), ctx.meso_shade(x, y), ctx.vignette_at(x, y), ctx.ao[i] as f64, ctx.hydro_wet[i] as f64, x, y, ctx.gw, ctx.gh)
    };

    // Milestone 4: the sheet. Applied *here*, after both branches, rather
    // than inside `land_color` — the ocean has to sit on the same paper as
    // the land or the map reads as terrain-art pasted onto a parchment
    // background. `paper_tone` returns `(1,1,1)` and `apply_border` returns
    // its input unchanged whenever their strengths are `0.0`, which is
    // `js_reference()`'s state, so the pinned JS-parity path never enters
    // any of this.
    let tone = paper_tone(&ctx.appearance, x, y, ctx.gw);
    let (r, g, b) = apply_paper(&ctx.appearance, (r, g, b), tone);
    let (r, g, b) = apply_border(&ctx.appearance, (r, g, b), tone, x, y, ctx.gw, ctx.gh);

    (clamp01(r / 255.0), clamp01(g / 255.0), clamp01(b / 255.0))
}
