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
//! Excluded: rockSlope refinement, wetness darkening,
//! geology microtexture and dune ripples, procedural texture synthesis,
//! ridged-relief creases, curvature shading,
//! SVF/cast-shadow fields, and the coast/river SDF
//! tinting plus vector river overlay (the last two depend on subsystems
//! this port hasn't built yet; the existing simple river channel-mask tint
//! in `lib.rs` stays as this port's stand-in for "rivers visible",
//! `MVP_SCOPE.md`'s point 2).
//!
//! Ported despite being extras: the `bioBlend` grey-desaturation blend
//! (0.90 default) and the edge haze fade, both unconditional in the
//! reference at its own default settings.
//!
//! ## The NPR block ([`Npr`], `GUI_GAP_REGISTER.md` RN-01)
//!
//! The ten "Painter" hand-drawn styles, the coastal wave lines and the
//! multi-sun light rig were on the excluded list above until they were
//! ported literally (`apply_npr`, `coast_distance`, `multi_sun_from_normal`),
//! and so was the **paint-brush biome/terrain override**, removed from that
//! list on 2026-08-24 when [`land_color`] gained its `paint` parameter — see
//! that blend's own comment, and [`PaintOverride`], for what is and is not
//! reachable.
//! They are `state.viz.*`-gated in the reference and off at every default
//! here, so this changed no pixel of the shipped look; what it changed is
//! that they are now reachable, through `WorldGen::set_npr`.
//!
//! The eleventh member of that block, **animated water**, is not in this
//! file at all: it is per-frame rather than per-pixel, and lives on a Godot
//! overlay (`water_anim_layer.gd` + `water_anim.gdshader`) exactly as the
//! reference keeps it on its own separate RAF-driven canvas.
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
//!
//! ## Geology and local contrast (`TERRAIN_APPEARANCE_SCOPE.md` milestone 5)
//!
//! - **Geological material exposure** (§12) — the world's real rock type
//!   (`cartalith_civ::build_lithology`, seven `LITH_KEYS` types built from
//!   the tectonic substrate) reaches the image two ways: the rock material's
//!   own colour blends toward that rock's palette (`rock_material_col`), and
//!   bedrock shows through thin soil where slope, low vegetation and low
//!   moisture say the cover is thin (in `land_color`). Attached via
//!   [`RenderCtx::with_lithology`]; absent for a loaded save, whose format
//!   stores no tectonic substrate, in which case both stages do nothing.
//! - **Local contrast** (§18) — [`apply_local_contrast`], the only stage in
//!   this file that is *not* per-pixel, because a neighbourhood of the
//!   finished colour cannot exist until the raster does. Runs over the
//!   output buffer; `cell_color` is untouched by it.
//!
//! Both are gated to `0.0` in `js_reference()` and both early-return on that
//! `0.0`, the same rule every stage since milestone 2 follows.

use cartalith_noise::{fbm, vnoise};
// Milestone 6 (§21/§23): the per-pixel appearance pass and the
// whole-raster local-contrast pass are element-wise over the grid, so they
// parallelize without changing a single float. Every sibling engine crate
// already does this (`CPU_MULTITHREADING_SCOPE.md` milestones 2-3); this
// renderer was the last O(gw*gh) loop in the workspace still on one core.
use rayon::prelude::*;

type Rgb = (f64, f64, f64);

/// A colour and how much of it there is: `0-255` per channel like [`Rgb`], plus
/// an opacity in `[0, 1]`. Only [`ElevationRamp`] uses it — the ramp is the one
/// layer in this renderer with an authored per-sample opacity, and everything
/// else here composites at a fixed strength.
type Rgba = (f64, f64, f64, f64);

/// `CART_BIOME_COLS` (reference HTML line 6813), 1-based like `CART_BIOMES`.
///
/// **Canonical here, not in `sample_bridge.rs`**, which re-exports these two
/// rather than keeping a second copy: `landColorCore`'s paint blend
/// ([`land_color`]'s own `paint` parameter) is the reference's *primary*
/// consumer, and this module is `#[path]`-included standalone by five test
/// targets, so it cannot reach a sibling module's copy.
pub const CART_BIOME_COLS: [(u8, u8, u8); 15] = [
    (90, 147, 184),
    (58, 122, 74),
    (168, 163, 90),
    (74, 120, 120),
    (158, 149, 96),
    (42, 106, 58),
    (58, 106, 90),
    (122, 122, 138),
    (154, 138, 106),
    (201, 165, 90),
    (165, 181, 197),
    (106, 74, 74),
    (122, 138, 74),
    (58, 122, 184),
    (30, 70, 110),
];

/// `CART_TERRAIN_COLS` (reference HTML line 6858), 1-based like
/// `CART_TERRAINS`; `0` is water/unpainted and drawn separately.
pub const CART_TERRAIN_COLS: [(u8, u8, u8); 13] = [
    (138, 138, 138),
    (154, 122, 74),
    (194, 160, 96),
    (176, 176, 96),
    (111, 95, 51),
    (138, 154, 82),
    (154, 154, 154),
    (122, 122, 138),
    (99, 99, 122),
    (86, 106, 70),
    (212, 184, 122),
    (213, 224, 234),
    (122, 106, 106),
];

/// The reference's `pBio`/`pTer`/`pSplat` triple — `landColorCore`'s own last
/// three parameters (reference 7720), read at each of its three call sites
/// from `paintBiome`/`paintTerrain`/`paintSplat` (`paintBiome?paintBiome[i]:0`
/// on the main map at 8168, `_paintSampleAt` in both bake paths at 11731 and
/// 11970).
///
/// `0` in every field is "nothing painted here", which is what every caller
/// that has no paint editor at all passes — and, being the reference's own
/// unpainted value, makes the whole paint stage a no-op rather than a
/// special case. [`PaintOverride::default`] is therefore also the pinned
/// JS-parity state, so `golden_parity_render.rs` needed no change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PaintOverride {
    /// 1-based `CART_BIOMES` index, `0` = unpainted.
    pub bio: u8,
    /// 1-based `CART_TERRAINS` index, `0` = unpainted.
    pub ter: u8,
    /// 1-based `SPLAT_PAINT_SLOTS` index, `0` = unpainted.
    pub splat: u8,
}

impl PaintOverride {
    fn is_empty(self) -> bool {
        self.bio == 0 && self.ter == 0 && self.splat == 0
    }
}

/// One decoded ground-material channel plus its baked inverse-mean
/// (`finalizePackTexture`) — real pixel data from a loaded pack
/// (`ASSET_LIBRARY_SCOPE.md` milestone 7). Defined here rather than in
/// `crate::pack` so `render.rs` keeps compiling standalone under
/// `golden_parity_render.rs`'s own `#[path = "../src/render.rs"] mod
/// render;` inclusion, which has no sibling `pack` module to resolve a
/// cross-module `use` against — `crate::pack::load_pack_from_bytes` builds
/// this type directly instead.
#[derive(Clone)]
pub struct SplatChannel {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub inv: [f64; 3],
}

/// Real ground-texture channels for splat blending (`ASSET_LIBRARY_SCOPE.md`
/// milestone 7) — the six `SPLAT_PAINT_SLOTS` from a loaded pack, borrowed
/// rather than owned so `RenderCtx` doesn't need to know anything about pack
/// lifetime beyond "outlives this render". `None` fields are the common case
/// (a pack with textures for some but not all six channels; `land_color`'s
/// `sp()`-equivalent already treats a missing channel as zero coverage, same
/// as the reference).
#[derive(Clone, Copy, Default)]
pub struct SplatTextures<'a> {
    pub grass: Option<&'a SplatChannel>,
    pub rock: Option<&'a SplatChannel>,
    pub sand: Option<&'a SplatChannel>,
    pub snow: Option<&'a SplatChannel>,
    pub wetland: Option<&'a SplatChannel>,
    pub canopy: Option<&'a SplatChannel>,
}

/// `sp(tex, rampCol, wt)` (reference line 7761-7762) — a texture channel's
/// texel, re-tinted by the material's own procedural ramp colour as a
/// deviation-around-the-mean ratio (`texel * inv_mean`), accumulated by
/// material weight. UV: one texel per grid cell, nearest, wrapped — the same
/// addressing `_paintedTex` uses, so a texture tiles identically whichever
/// path samples it.
fn splat_sample(tex: &SplatChannel, ramp: Rgb, wt: f64, x: usize, y: usize, acc: &mut Rgb, cov: &mut f64) {
    if wt <= 0.0 {
        return;
    }
    let (tw, th) = (tex.w as i64, tex.h as i64);
    let sx = (((x as i64) % tw) + tw) % tw;
    let sy = (((y as i64) % th) + th) % th;
    let o = ((sy * tw + sx) * 4) as usize;
    let (r, g, b) = (tex.rgba[o] as f64, tex.rgba[o + 1] as f64, tex.rgba[o + 2] as f64);
    acc.0 += ramp.0 * r * tex.inv[0] * wt;
    acc.1 += ramp.1 * g * tex.inv[1] * wt;
    acc.2 += ramp.2 * b * tex.inv[2] * wt;
    *cov += wt;
}

/// The reference's `state.viz.*` non-photorealistic block: the ten "Painter"
/// hand-drawn styles (reference HTML lines 7903-7962), the coastal wave lines
/// (8442-8443, 8555-8558), the multi-sun light rig (8351-8370) and the
/// animated-water toggle (8667-8690).
///
/// **Every field is off at `Default`**, and each style is skipped by its own
/// `> 0.0` gate exactly as the reference skips it — so
/// `TerrainAppearance::default()` and `js_reference()` are both bit-identical
/// to what they produced before this struct existed, and
/// `golden_parity_render.rs` never enters a single line of it.
///
/// The intensities are the reference's own `0..1` slider values, not
/// renormalised: `contours: 0.5` here means the same picture the HTML app's
/// "contour veins" slider at 50 produces.
///
/// `animate_water` is the one member that this renderer never reads. It is a
/// **presentation flag carried with the rest of the block** because the effect
/// it names is per-*frame*, not per-pixel: the reference draws it on a separate
/// RAF-driven overlay canvas (`waterAnimFrame`) and this port draws it on a
/// separate Godot overlay too (`water_anim_layer.gd`). Keeping the flag here
/// means the whole NPR vocabulary crosses the gdext boundary through one
/// `set_npr()` rather than through one method plus an unrelated second one.
// `Serialize`/`Deserialize` for `GUI_GAP_REGISTER.md` CA-08: a saved
// appearance preset carries the whole look, and the Painter block is part of
// the look. `#[serde(default)]` on the struct so a preset written by an older
// build (one style short) still loads, gaining that style at its own default
// rather than failing the whole file.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Npr {
    /// `state.viz.contours` — constant-width elevation isolines, every fifth
    /// an index line.
    pub contours: f64,
    /// `state.viz.contourM` — contour interval in **metres** (5-50). `0`
    /// takes the reference's own legacy fixed `0.05`-of-relief interval, and
    /// is what `Default` gives.
    pub contour_m: f64,
    /// `state.peakM`, needed only to turn [`Self::contour_m`] into a fraction
    /// of relief. `0` falls back to the reference's own `||4000`.
    pub peak_m: f64,
    /// `state.viz.ink` — pen outlines on curvature×slope landform edges.
    pub ink: f64,
    /// `state.viz.hachure` — downslope hatching, denser on steep ground.
    pub hachure: f64,
    /// `state.viz.watercolor` — pigment pooling, paper granulation, edge blooms.
    pub watercolor: f64,
    /// `state.viz.cel` — posterised flat toon bands.
    pub cel: f64,
    /// `state.viz.crosshatch` — antique engraving; more hatch directions the
    /// darker the cell.
    pub crosshatch: f64,
    /// `state.viz.stipple` — pen dot-density shading. Distinct from
    /// [`TerrainAppearance::stipple_strength`], which is milestone 4's
    /// canopy-driven *forest* stipple: this one is driven by luminance and
    /// covers all land.
    pub stipple: f64,
    /// `state.viz.sepia` — the classic warm ochre toning matrix.
    pub sepia: f64,
    /// `state.viz.risograph` — indigo→amber duotone with a halftone screen.
    pub risograph: f64,
    /// `state.viz.pointillism` — Seurat-style coloured dot field.
    pub pointillism: f64,
    /// `state.viz.waves` — foam contours hugging the shore, fading offshore.
    /// Water cells only; costs a chamfer distance transform when on and
    /// nothing at all when off.
    pub waves: bool,
    /// `state.viz.waveDist` — how far the foam reaches offshore. `<= 0` means
    /// the reference's own `1` (its `waveDist>0?waveDist:1`), so `Default`
    /// needs no non-zero value.
    pub wave_dist: f64,
    /// `state.viz.multiSun` — the reference's four-light painterly rig
    /// (primary at 45°, fill at azimuth+90°/35°, a zenith light, and a 0.10
    /// ambient floor; weights 0.40/0.30/0.20/0.10). Replaces **only** the
    /// macro hillshade, exactly as `macroShade` does; the meso shade and the
    /// sea shade stay single-sun, as they do in the reference.
    pub multi_sun: bool,
    /// `state.viz.waterAnim` — see this struct's own doc comment for why a
    /// flag this renderer never reads lives here.
    pub animate_water: bool,
}

/// One breakpoint of an elevation-keyed colour ramp: a position in
/// **relative land elevation** and the colour the map takes there.
///
/// `at` is `land_color`'s own `r` — `0.0` at the shoreline, `1.0` at the
/// world's highest point — not metres. Metres are a *presentation* of this
/// (`peak_m` turns one into the other, and the panel does exactly that), and
/// storing them here would make a saved ramp mean a different picture on a
/// world with a different peak, which is the one thing a saved look must not
/// do.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RampStop {
    /// Relative elevation above sea level, `[0, 1]`.
    pub at: f64,
    /// 0-255 per channel, matching every other colour in this file.
    pub col: Rgb,
    /// How opaque this stop's colour is over the material colour, `[0, 1]`,
    /// interpolated between stops exactly as `col` is and then multiplied into
    /// [`TerrainAppearance::ramp_strength`]. `1.0` is the ramp taking over
    /// completely at full strength; `0.0` is a stop that reveals the material
    /// model beneath it, which is how a ramp is authored to tint only the
    /// summits (or only the lowlands) and leave the rest of the map alone.
    ///
    /// `#[serde(default = "one")]`, not `#[serde(default)]`: a saved look
    /// written before this field existed described **opaque** stops, and
    /// `f64::default()` would load every one of them as invisible.
    #[serde(default = "one")]
    pub a: f64,
}

/// `serde`'s default for [`RampStop::a`] — see there.
fn one() -> f64 {
    1.0
}

/// How [`ElevationRamp::sample`] crosses from one stop to the next.
///
/// A property of the **ramp**, not of a stop: `DCC_SHELL_SPEC.md` §7 draws one
/// picker above the stop list, and it is also the honest model — "step" is a
/// statement about the whole plate (a banded hypsometric map is banded
/// everywhere), not about one breakpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RampMode {
    /// Straight lerp between neighbours. What CA-02 shipped, and the default,
    /// so nothing an existing saved look describes moves.
    #[default]
    Linear,
    /// The same lerp eased at both ends (`k²(3-2k)`, this file's own
    /// [`smoothstep`] curve), which flattens the ramp at each stop and puts the
    /// colour change in the middle of the interval. Reads as broad bands with
    /// soft joins rather than as a continuous wash.
    Ease,
    /// No blend at all: every sample takes the colour of the stop at or below
    /// it, so each pair of stops is one flat band with a hard edge. This is how
    /// a classic banded hypsometric plate is drawn.
    Step,
}

/// The mode names the panel's picker shows, in [`RampMode`]'s own order —
/// the engine's list rather than a second copy of it in GDScript, the same
/// rule `RAMP_PRESETS` follows.
#[allow(dead_code)]
pub const RAMP_MODES: &[&str] = &["Linear", "Ease", "Step"];

impl RampMode {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        RAMP_MODES[self as usize]
    }

    /// One of [`RAMP_MODES`] by exact name, or `None` — an unknown name is the
    /// caller's problem to report, exactly as `ElevationRamp::preset`'s is.
    #[allow(dead_code)]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Linear" => Some(RampMode::Linear),
            "Ease" => Some(RampMode::Ease),
            "Step" => Some(RampMode::Step),
            _ => None,
        }
    }

    /// Reshape the `[0, 1]` fraction of the way from one stop to the next.
    ///
    /// `Step` tests `>= 1.0` rather than returning a flat `0.0` so a sample
    /// landing exactly **on** a stop takes that stop's own colour: the band is
    /// half-open `[a, b)`, which is what makes two stops at the same position
    /// still draw the hard edge they draw under `Linear`.
    fn curve(self, k: f64) -> f64 {
        match self {
            RampMode::Linear => k,
            RampMode::Ease => k * k * (3.0 - 2.0 * k),
            RampMode::Step => {
                if k >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// `GUI_GAP_REGISTER.md` **CA-02** — the elevation-keyed colour ramp this
/// renderer did not have, as a real breakpoint list rather than as a second
/// material model.
///
/// **What this is and is not.** This module's own doc comment records the
/// milestone-1 audit finding: colour here comes from `material_weights`, a
/// continuous climate/slope/relief blend, and *"a literal MapTiler-style
/// elevation ramp would be a genuinely new visual layer/mode to design on top
/// of (or blended with) this material model in a future milestone"*. This is
/// that layer, built the way the finding said it had to be — **on top of**,
/// blended by [`TerrainAppearance::ramp_strength`], and `0.0` by default, so
/// the material model stays the shipped look and nothing about it moved.
///
/// It is applied to **land only** and **before lighting**, so the hillshade,
/// AO, paper ground, haze, vignette and the whole Painter block still act on
/// it exactly as they act on the material colour. That is what makes
/// `ramp_strength = 1.0` a *hypsometric tint over shaded relief* — the classic
/// atlas construction — rather than a flat elevation key pasted over the map.
/// Water keeps its own bathymetric ramp (`sea_color_core`), which is already
/// depth-keyed and is a different thing.
///
/// Stops are kept **sorted by `at`**, which is what makes "reorder" a
/// meaningful operation rather than a list-index shuffle: dragging a stop past
/// another *is* the reorder, and [`Self::normalized`] is the single place that
/// invariant is established.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ElevationRamp {
    stops: Vec<RampStop>,
    /// `#[serde(default)]` at field level because this struct has none at
    /// struct level: a saved look written before the mode existed describes a
    /// `Linear` ramp, and without this it would fail to parse rather than load.
    #[serde(default)]
    mode: RampMode,
}

impl ElevationRamp {
    /// Build from arbitrary caller data: positions clamped to `[0, 1]`,
    /// channels to `[0, 255]`, alpha to `[0, 1]`, and the whole list sorted.
    /// Non-finite positions are dropped rather than sorted against
    /// (`partial_cmp` has no answer for NaN, and `cartalith-rust-conventions`
    /// requires a stated NaN policy wherever floats are ordered — the policy
    /// here is "a stop with no position is not a stop"). A non-finite **alpha**
    /// is not a reason to drop a stop, so it becomes `1.0`: an opaque stop is a
    /// visible mistake, an invisible one looks like the engine ignored the edit.
    ///
    /// The result is always [`RampMode::Linear`]; a caller replacing the stops
    /// of a ramp that has a mode carries it over with [`Self::set_mode`], which
    /// is what `WorldGen::set_color_ramp` does.
    pub fn normalized(stops: impl IntoIterator<Item = RampStop>) -> Self {
        let mut stops: Vec<RampStop> = stops
            .into_iter()
            .filter(|s| s.at.is_finite())
            .map(|s| RampStop {
                at: clamp01(s.at),
                col: (s.col.0.clamp(0.0, 255.0), s.col.1.clamp(0.0, 255.0), s.col.2.clamp(0.0, 255.0)),
                a: if s.a.is_finite() { clamp01(s.a) } else { 1.0 },
            })
            .collect();
        // Every `at` is finite by the filter above, so this `unwrap` cannot
        // fire -- the reason it is safe is the line above it, not optimism.
        stops.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap());
        ElevationRamp { stops, mode: RampMode::Linear }
    }

    pub fn stops(&self) -> &[RampStop] {
        &self.stops
    }

    #[allow(dead_code)]
    pub fn mode(&self) -> RampMode {
        self.mode
    }

    #[allow(dead_code)]
    pub fn set_mode(&mut self, mode: RampMode) {
        self.mode = mode;
    }

    /// The colour **and opacity** at relative elevation `t`, or `None` for an
    /// empty ramp — the caller decides what "no ramp" means rather than being
    /// handed a black that would look like a rendered result.
    ///
    /// Interpolation between neighbouring stops is [`Self::mode`]'s; beyond the
    /// ends it is flat in every mode, because there is no second stop to blend
    /// towards. CA-02 shipped `Linear` only and said so in this comment; `Ease`
    /// and `Step` are `DCC_SHELL_SPEC.md` §7's other two, added 2026-08-24.
    ///
    /// The alpha rides the same `k` as the colour, so a stop's opacity crosses
    /// to its neighbour's exactly the way its colour does — including under
    /// `Step`, where both change at once at the band edge, which is the whole
    /// point of that mode.
    pub fn sample(&self, t: f64) -> Option<Rgba> {
        let t = clamp01(t);
        let first = self.stops.first()?;
        if t <= first.at {
            return Some((first.col.0, first.col.1, first.col.2, first.a));
        }
        let last = self.stops.last()?;
        if t >= last.at {
            return Some((last.col.0, last.col.1, last.col.2, last.a));
        }
        let hi = self.stops.iter().position(|s| s.at >= t)?;
        let (a, b) = (self.stops[hi - 1], self.stops[hi]);
        let span = b.at - a.at;
        // Two stops at the same position are a legal thing to author (it is
        // how a hard band edge is drawn with linear interpolation), and it is
        // also the one input that would divide by zero.
        let k = if span <= 0.0 { 1.0 } else { (t - a.at) / span };
        let k = self.mode.curve(k);
        let c = mix(a.col, b.col, k);
        Some((c.0, c.1, c.2, lerp(a.a, b.a, k)))
    }
}

impl Default for ElevationRamp {
    /// The first entry of [`RAMP_PRESETS`]. A ramp is always populated even
    /// though `ramp_strength` starts at `0.0`, so the editor opens on
    /// something to edit and the strength slider has something to reveal —
    /// an empty default would make the one control that turns the feature on
    /// appear dead, which is the exact failure CA-11 was.
    fn default() -> Self {
        ElevationRamp::preset(RAMP_PRESETS[0].0).expect("RAMP_PRESETS[0] must name itself")
    }
}

/// The named ramps `DCC_SHELL_SPEC.md` §7's Colour ramp popover lists, as
/// stops in relative land elevation. Pure data — nine tables, no logic.
///
/// Two of the spec's nine are re-read rather than transcribed, and the reason
/// is stated here rather than hidden: **Imhof** is the classic Swiss-style
/// warm-lowland/cool-highland progression, and **Atlas** is the muted
/// physical-atlas green-to-buff-to-brown; the spec names them without
/// defining their colours, so these are this port's reading of two very
/// well-known cartographic conventions. `Elevation`, `Mono`, `Ice`,
/// `Dark ice`, `Desert` and `Dark atlas` are the same kind of reading.
///
/// `Earth` is first because it is [`ElevationRamp::default`], and it is
/// deliberately the closest of the nine to what `material_weights` already
/// produces — so a user who raises the strength slider without touching
/// anything else sees the ramp *taking over*, not a different planet.
#[allow(dead_code)]
pub const RAMP_PRESETS: &[(&str, &[(f64, (u8, u8, u8))])] = &[
    ("Earth", &[(0.00, (152, 168, 116)), (0.10, (128, 150, 96)), (0.28, (160, 162, 112)), (0.50, (166, 146, 106)), (0.72, (146, 132, 122)), (0.88, (176, 172, 168)), (1.00, (246, 248, 250))]),
    ("Elevation", &[(0.00, (0, 97, 71)), (0.20, (114, 168, 84)), (0.40, (224, 214, 130)), (0.60, (204, 148, 78)), (0.80, (150, 90, 62)), (1.00, (255, 255, 255))]),
    ("Atlas", &[(0.00, (183, 199, 152)), (0.18, (206, 214, 160)), (0.38, (226, 216, 158)), (0.58, (214, 186, 132)), (0.78, (190, 156, 122)), (1.00, (236, 232, 226))]),
    ("Mono", &[(0.00, (58, 58, 58)), (1.00, (242, 242, 242))]),
    ("Imhof", &[(0.00, (176, 186, 140)), (0.22, (198, 196, 146)), (0.45, (206, 184, 142)), (0.66, (186, 162, 148)), (0.84, (170, 168, 178)), (1.00, (238, 242, 248))]),
    ("Ice", &[(0.00, (150, 176, 190)), (0.35, (186, 208, 218)), (0.70, (218, 232, 238)), (1.00, (252, 254, 255))]),
    ("Dark ice", &[(0.00, (18, 28, 40)), (0.35, (44, 68, 88)), (0.70, (104, 138, 158)), (1.00, (198, 218, 230))]),
    ("Desert", &[(0.00, (198, 168, 112)), (0.25, (214, 178, 118)), (0.52, (196, 144, 96)), (0.76, (168, 112, 84)), (1.00, (226, 206, 186))]),
    ("Dark atlas", &[(0.00, (26, 38, 34)), (0.24, (44, 62, 50)), (0.50, (78, 84, 62)), (0.74, (104, 92, 78)), (1.00, (176, 172, 164))]),
];

impl ElevationRamp {
    /// One named preset by exact name, or `None` — an unknown name is the
    /// caller's problem to report, not this function's to paper over with a
    /// default that would silently be the wrong ramp.
    #[allow(dead_code)]
    pub fn preset(name: &str) -> Option<Self> {
        let (_, stops) = RAMP_PRESETS.iter().find(|(n, _)| *n == name)?;
        // Every preset stop is opaque: a named ramp is a complete picture, and
        // one that revealed the material model in places would be a look nobody
        // asked for hiding inside a list of nine.
        Some(ElevationRamp::normalized(stops.iter().map(|&(at, (r, g, b))| RampStop { at, col: (r as f64, g as f64, b as f64), a: 1.0 })))
    }
}

/// The renderer's editable colour data and shading constants — what used to
/// be 26 free-floating module consts, now one real, owned, inspectable
/// structure. `Default` reproduces today's exact values (pixel-identical
/// output). See this module's own doc comment for why the *material* colour
/// here is a palette table rather than an elevation-breakpoint ramp — and
/// [`ElevationRamp`] for the separate, opt-in ramp layer (CA-02) that sits on
/// top of it at `ramp_strength`.
///
/// Live through `WorldGen::{get_appearance, set_appearance,
/// list_appearance_tunables, reset_appearance}` since 2026-08-24; the
/// "wired to no UI" note this comment used to carry is no longer true.
// `Clone` so a caller can hold *one* appearance value, hand it to
// `RenderCtx::with_appearance`, and still measure the plate frame with
// `border_cover` afterwards — rather than the raster and the overlays each
// constructing their own `default()` and hoping the two agree.
//
// `Serialize`/`Deserialize` for `GUI_GAP_REGISTER.md` CA-08 — §7.15's "the one
// Rust line the whole feature depends on". `#[serde(default)]` at struct level
// is what makes a preset file survive this struct growing a field: an older
// file loads with the new field at its `Default` value instead of failing, and
// the loader then renders a look that is exactly what the file described plus
// today's default for what it could not have described.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
    // ---- Milestone 5: the five rock types the reference's own material
    // vocabulary never had a colour for. `LITH_KEYS` has seven entries;
    // granite and sandstone already had palettes above (they are the two
    // the JS heuristic happened to name), so only five are new.
    pub rock_basalt: [Rgb; 3],
    pub rock_andesite: [Rgb; 3],
    pub rock_limestone: [Rgb; 3],
    pub rock_shale: [Rgb; 3],
    pub rock_metamorphic: [Rgb; 3],
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

    // ---- Milestone 5 (`TERRAIN_APPEARANCE_SCOPE.md`): geology (§12) ----
    /// How far the **rock material's own colour** moves from the reference's
    /// climate heuristic (`rock_col`: scree above 0.82 relative elevation,
    /// sandstone when hot and dry, granite otherwise) toward the palette of
    /// the rock actually under the cell
    /// (`cartalith_civ::build_lithology`'s seven `LITH_KEYS` types).
    /// `0.0` disables it and `rock_material_col` early-returns the
    /// heuristic colour unchanged, which is `js_reference()`'s state; it is
    /// also inert whenever no lithology field is attached (a loaded save,
    /// which stores none — `SAVEFILE_COMPAT.md`).
    ///
    /// A **blend**, not a replacement: the heuristic still carries the
    /// climate-and-relief character of the surface (scree really is paler
    /// and greyer than the parent rock on a shattered summit), and the
    /// lithology supplies the identity underneath it.
    pub litho_strength: f64,
    /// How strongly bedrock shows **through the soil cover**
    /// (`TERRAIN_APPEARANCE_RESEARCH.md` §12's own list: material visibility
    /// depends on slope, erosion, elevation, vegetation, moisture,
    /// lithology). `0.0` disables the stage entirely.
    ///
    /// This is the half that answers §12's actual complaint — *"a mountain
    /// should not simply become brown because it is high; its visible
    /// material should emerge from the underlying world model"*. It reads
    /// only values `land_color` already has (`slope`, the vegetation
    /// potential `w.c`, effective moisture, and the rock/snow fractions),
    /// so it adds no physical input beyond the lithology index itself, and
    /// it never touches `material_weights` — the golden-verified fraction
    /// blend §32 warns is easiest to break.
    pub litho_exposure: f64,

    // ---- Milestone 5: local contrast (§18) ----
    /// Local-contrast gain (`TERRAIN_APPEARANCE_RESEARCH.md` §18): how much
    /// of the band-limited luminance detail is added back to the finished
    /// image, to make neighbouring terrain materials distinguishable after
    /// milestone 4's paper wash deliberately took ~13-26% of the chroma out.
    /// `0.0` disables the whole pass, which early-returns before allocating
    /// anything.
    ///
    /// Not a sharpen. §18's constraints are "avoid excessive sharpening, no
    /// haloing, no visible edge-detection artifacts", and the response curve
    /// in `apply_local_contrast` is built to satisfy them literally: the
    /// gain **falls to zero** on strong edges (coastlines, snowlines), so
    /// the classic unsharp overshoot has nowhere to form.
    pub local_contrast: f64,
    /// Detail-band radius as a fraction of grid width, so "local" is a
    /// fixed *world* size across this port's 512²-8192² range — the same
    /// reasoning `ao_radius_frac`, `hydro_wet_radius_frac` and
    /// `stipple_scale_frac` already use.
    pub local_contrast_radius_frac: f64,
    /// The luminance-difference scale (in 0-255 levels) at which the local
    /// contrast response peaks, and past which it rolls off toward zero.
    /// Small differences (material texture, a forest edge) get the full
    /// gain; a coastline's 40-plus-level step gets almost none. This single
    /// number is what makes §18's "no haloing" a property of the maths
    /// rather than a hope about the tuning.
    pub local_contrast_knee: f64,

    // ---- `GUI_GAP_REGISTER.md` CA-02: the elevation colour ramp ----
    /// How far the material colour is pulled toward [`Self::ramp`]'s colour
    /// for that cell's relative elevation. `0.0` is the shipped default and
    /// the whole stage is skipped on it, so the ramp changes no pixel until a
    /// caller asks for it; `1.0` is a full hypsometric tint, still shaded,
    /// still on the sheet.
    ///
    /// A *blend*, deliberately, and not a mode switch: at 0.3-0.5 the ramp
    /// gives the map the readable elevation key an atlas plate has while
    /// `material_weights`' climate and slope information still shows through,
    /// which is the picture `TERRAIN_APPEARANCE_RESEARCH.md` §30 asks for
    /// ("make the physical differences visually legible") and a hard override
    /// would throw away.
    pub ramp_strength: f64,
    /// The ramp itself. Always populated — see [`ElevationRamp::default`] for
    /// why a feature that is off by default still ships with real stops.
    pub ramp: ElevationRamp,

    // ---- Milestone 7 (`ASSET_LIBRARY_SCOPE.md`): ground-texture splat ----
    /// Strength of the ground-texture splat blend (reference `state.viz.
    /// splat`, real default `0.7` — unlike `state.viz.icons`, splat is
    /// **not** off-by-default in the reference; it is gated purely by
    /// `assetPack.texAny` (line 8410's own comment: "textures only in the
    /// biome material render"). This field matches that: it is inert
    /// whenever `RenderCtx.splat` is `None` (the case for every existing
    /// caller and for `golden_parity_render.rs`, which never attaches a
    /// pack), and becomes real the moment a real pack with real ground
    /// textures is loaded — genuinely additive/opt-in rather than a
    /// JS-parity-gated stretch feature, since there is no pack-less version
    /// of "blend in a texture that doesn't exist" to be bit-identical with.
    pub splat_strength: f64,

    /// The reference's non-photorealistic block ([`Npr`]) — all off by
    /// default, so this field changes nothing until a caller sets it.
    pub npr: Npr,
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
            // Milestone 5 (§12). Chosen for *separation in hue and value*
            // between the seven types rather than for photographic accuracy:
            // basalt near-black and cool, limestone pale and warm, shale
            // dark olive-brown, metamorphic mid grey-green, andesite a
            // neutral mid grey. Two of §30's anti-list items — "overuse of
            // brown for mountains" and terrain that reads as decorated
            // rather than described — are precisely what a single grey
            // granite for every uplift produces.
            rock_basalt: [(52.0, 55.0, 60.0), (69.0, 73.0, 79.0), (91.0, 95.0, 102.0)],
            rock_andesite: [(97.0, 91.0, 92.0), (117.0, 111.0, 111.0), (139.0, 133.0, 133.0)],
            rock_limestone: [(163.0, 158.0, 141.0), (187.0, 182.0, 164.0), (209.0, 205.0, 188.0)],
            rock_shale: [(80.0, 77.0, 68.0), (98.0, 94.0, 83.0), (118.0, 113.0, 101.0)],
            rock_metamorphic: [(97.0, 101.0, 94.0), (117.0, 121.0, 112.0), (139.0, 143.0, 133.0)],
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
            litho_strength: 0.62,
            litho_exposure: 0.55,
            local_contrast: 0.55,
            local_contrast_radius_frac: 0.010,
            local_contrast_knee: 26.0,
            // CA-02: off, so the shipped look is unchanged; the ramp behind it
            // is real so the slider has something to reveal.
            ramp_strength: 0.0,
            ramp: ElevationRamp::default(),
            splat_strength: 0.7,
            npr: Npr::default(),
        }
    }
}

/// `TERRAIN_APPEARANCE_RESEARCH.md` §29's four quality presets, as a real
/// mechanism rather than a policy. **Which tier a given device should get is
/// the owner's decision**, so this type only *provides* the ladder and a
/// recommendation; nothing here changes what the app renders by default.
///
/// [`QualityTier::Quality`] is exactly [`TerrainAppearance::default()`],
/// bit-for-bit -- the tier ladder was introduced without moving the look the
/// previous five milestones tuned. `Performance`/`Balanced` step *down* from
/// it and `Ultra` steps up.
///
/// The ladder drops **texture, never identity**: every tier keeps the paper
/// tint, the paper wash and the plate frame, because those are what make the
/// sheet read as an atlas plate (`VISION.md`) and they are also nearly free --
/// the tint and the wash are a handful of multiplies, and the frame reads no
/// world data at all. What the cheap tiers give up is the per-pixel
/// coherent-noise work (paper fibre/mottle, forest stipple, the lithology
/// jitter lookup), the AO/hydrology precomputes, the extra light directions
/// and the whole-raster local-contrast pass -- i.e. exactly the stages whose
/// cost scales with `gw*gh` and whose absence degrades the image gracefully
/// instead of breaking it.
// `golden_parity_render.rs` `#[path]`-includes this file standalone, with no
// `lib.rs` to construct tiers from -- the same reason `js_reference()` and
// `border_cover` already carry this attribute, in reverse.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QualityTier {
    /// §29's "basic colour relief... minimal microvariation", **re-derived
    /// from this renderer's own measured stage costs** rather than taken
    /// literally. Everything that measured at or below the noise floor stays
    /// -- the full six-direction relief, AO, the hydrology tint -- and
    /// everything that measured as real time goes: a smooth (untextured)
    /// sheet, no stipple, no geology, no local contrast.
    Performance,
    /// §29: "colour relief, multidirectional shading, slope modulation,
    /// lightweight AO". Exactly `Quality` minus the two stages the cost table
    /// puts at the top: the whole-raster local-contrast pass and the
    /// sheet-scale paper mottle.
    Balanced,
    /// §29's "full material modulation, multidirectional hillshade, AO,
    /// curvature, multi-scale detail" -- and this port's own default look as
    /// milestones 1-5 left it. Bit-identical to `TerrainAppearance::default()`.
    Quality,
    /// §29: "highest available precision, enhanced AO, full material/lighting
    /// pipeline, wide gamut/HDR where supported". The first two are real here
    /// (ten light directions instead of six, stronger AO, higher local
    /// contrast); **the precision/HDR half is not** -- that is research §20's
    /// high-precision display pipeline, which this port has not built, and
    /// claiming it here would be dishonest.
    Ultra,
}

#[allow(dead_code)]
impl QualityTier {
    /// Cheapest first. The order is load-bearing: `TERRAIN_APPEARANCE_SCOPE.md`
    /// milestone 6's cost table and the tier-monotonicity test both walk it.
    pub const ALL: [QualityTier; 4] = [QualityTier::Performance, QualityTier::Balanced, QualityTier::Quality, QualityTier::Ultra];

    /// Stable lowercase identifier -- what crosses the gdext boundary
    /// (`WorldGen::set_quality_tier`) and what a preset file would store.
    pub fn name(self) -> &'static str {
        match self {
            QualityTier::Performance => "performance",
            QualityTier::Balanced => "balanced",
            QualityTier::Quality => "quality",
            QualityTier::Ultra => "ultra",
        }
    }

    /// Case-insensitive parse of [`Self::name`]. `None` for anything else --
    /// the caller decides what to do rather than being silently handed a
    /// default it did not ask for.
    pub fn from_name(s: &str) -> Option<Self> {
        QualityTier::ALL.into_iter().find(|t| t.name().eq_ignore_ascii_case(s))
    }
}

/// A tier this machine can plausibly afford, for a caller that wants to
/// *offer* one. Deliberately **not applied anywhere**: `WorldGen` still
/// starts at `Quality` on every device, and picking a device-appropriate
/// default is an owner policy decision, not this function's.
///
/// The renderer's per-pixel pass is `rayon`-parallel as of milestone 6, so
/// the honest predictor of what a device can afford is its core count: two
/// cores at the app's own 2048x1311 is roughly eight times the wall clock of
/// sixteen. Android is capped one rung below what its core count suggests,
/// because a phone's cores are neither as fast nor as sustainably clocked as
/// the count implies -- the real device pass measured ~31 s for a single
/// 2048x1311 generation.
///
/// Never returns `Ultra`: that tier costs more than `Quality` for a
/// difference only a deliberate choice justifies.
#[allow(dead_code)]
pub fn recommended_quality_tier() -> QualityTier {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let tier = if cores <= 2 {
        QualityTier::Performance
    } else if cores <= 6 {
        QualityTier::Balanced
    } else {
        QualityTier::Quality
    };
    if cfg!(target_os = "android") && tier == QualityTier::Quality { QualityTier::Balanced } else { tier }
}

impl TerrainAppearance {
    /// The appearance data for one §29 quality tier.
    ///
    /// `Quality` returns `default()` **unchanged and unreconstructed**, so
    /// the tier ladder cannot drift from the look milestones 1-5 tuned even
    /// by a typo; the other three are written as struct-update expressions
    /// over it, so any field nobody tiers is by construction identical in
    /// every tier.
    #[allow(dead_code)]
    pub fn for_tier(tier: QualityTier) -> Self {
        let q = TerrainAppearance::default();
        match tier {
            QualityTier::Quality => q,
            QualityTier::Performance => TerrainAppearance {
                // **The ladder drops stages in measured cost order**, and the
                // measurement is this milestone's own `cost_table` in
                // `appearance_ab_dump.rs` (best of three, all three test
                // worlds, at the app's own 2048x2048). Marginal cost of each
                // stage, largest first: local contrast 30-53 ms, the paper's
                // four `vnoise` calls 6-18 ms, stipple 3-6 ms, geology 0-6 ms
                // -- and then hydrology, AO and the five extra light
                // directions, all of which sit **at or below the noise floor
                // of the measurement itself**.
                //
                // That is the opposite of research §29's own Performance
                // recipe ("basic hillshade, no expensive AO"), which assumes
                // a raymarched AO and a per-light full shading pass. Here AO
                // is one separable box blur computed once, and the extra
                // lights are five dot products against a normal that is
                // computed anyway. Dropping them would have surrendered the
                // whole of milestone 2's relief legibility to buy nothing
                // measurable, so this tier keeps them and gives up the
                // texture and the second pass instead.
                paper_grain: 0.0,
                paper_mottle: 0.0,
                stipple_strength: 0.0,
                litho_strength: 0.0,
                litho_exposure: 0.0,
                local_contrast: 0.0,
                ..q
            },
            QualityTier::Balanced => TerrainAppearance {
                // Exactly `Quality` minus the two most expensive stages, and
                // nothing else -- lightening a stage that costs 3 ms would be
                // giving up image for no time. The sheet-scale ageing mottle
                // is half of the paper's `vnoise` bill and the least legible
                // of the sheet's cues at a glance; the fibre, which is what
                // actually reads as paper, survives.
                paper_mottle: 0.0,
                local_contrast: 0.0,
                ..q
            },
            QualityTier::Ultra => TerrainAppearance {
                relief_lights: 10,
                ao_strength: 0.32,
                local_contrast: 0.62,
                ..q
            },
        }
    }

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
            // Milestone 5, same rule again: `rock_material_col` returns the
            // reference's own `rock_col` before it looks at any palette,
            // the bedrock-exposure block is inside an `if`, and
            // `apply_local_contrast` returns before allocating a buffer.
            // (Lithology is additionally never attached on the parity path
            // — `RenderCtx::with_lithology` is a builder the golden test
            // does not call — so §12 is off twice over, by data and by
            // parameter.)
            litho_strength: 0.0,
            litho_exposure: 0.0,
            local_contrast: 0.0,
            ..TerrainAppearance::default()
        }
    }
}

/// Declares the scalar fields a UI may read and write **by name**, together
/// with the range each one is meaningful over, from a single list — so the
/// name table, the reader and the writer cannot drift apart the way three
/// hand-written matches would. `WorldGen::{get_appearance, set_appearance}`
/// are the only callers; everything else keeps using the fields directly.
///
/// Why by name at all: the alternative is ~20 `#[func]` pairs, and the shell
/// would then have to know which of them exist on the cdylib it happens to be
/// running against. `set_npr` already established the "one dictionary, every
/// key optional, returns the number applied" contract for exactly this, and a
/// second mechanism for the same job would be one more thing to keep in sync.
macro_rules! tunables {
    ($($key:literal => $field:ident, $min:expr, $max:expr, $label:literal;)*) => {
        #[allow(dead_code)]
        impl TerrainAppearance {
            /// Every tunable, as `(key, min, max, human label)`. The order is
            /// the order a panel should show them in.
            pub const TUNABLE: &'static [(&'static str, f64, f64, &'static str)] =
                &[$(($key, $min, $max, $label)),*];

            /// The current value of one tunable, or `None` for a key that is
            /// not one — the caller decides what an unknown key means.
            pub fn tunable(&self, key: &str) -> Option<f64> {
                match key { $($key => Some(self.$field),)* _ => None }
            }

            /// Write one tunable, **clamped to its declared range**. Returns
            /// `false` (and changes nothing) for an unknown key. Clamping is
            /// not defensive politeness: several of these multiply straight
            /// into a `1 - a` term, where a value past 1 inverts the image
            /// rather than intensifying it — the same reasoning `set_npr`
            /// already documents for the Painter intensities.
            pub fn set_tunable(&mut self, key: &str, v: f64) -> bool {
                match key { $($key => { self.$field = v.clamp($min, $max); true })* _ => false }
            }
        }
    };
}

tunables! {
    // -- Reference Cartography ▸ Map view (HTML lines 1706-1717) --
    "exag"                  => exag,                  0.0,  12.0,  "Relief exaggeration";
    "sun_az_deg"            => sun_az_deg,            0.0, 360.0,  "Sun azimuth";
    "sun_alt_deg"           => sun_alt_deg,           5.0,  85.0,  "Sun elevation";
    "bio_blend"             => bio_blend,             0.0,   1.0,  "Relief <-> biome";
    // -- Relief rig (milestone 2; the reference has no counterpart) --
    "relief_directionality" => relief_directionality, 0.0,   1.0,  "Directionality";
    "relief_ambient"        => relief_ambient,        0.0,   1.0,  "Ambient floor";
    "relief_gain"           => relief_gain,           0.0,   2.0,  "Light gain";
    // -- Reference Rendering-advanced ▸ Ambient occlusion (`aoR`) --
    "ao_strength"           => ao_strength,           0.0,   1.0,  "Ambient occlusion";
    "ao_radius_frac"        => ao_radius_frac,        0.0,   0.05, "AO radius";
    // -- Reference Rendering-advanced ▸ Wetness (`wetnessR`) --
    "hydro_wet_strength"    => hydro_wet_strength,    0.0,   1.0,  "Wetness";
    // -- Reference Rendering-advanced ▸ Parchment (`parch`), plus the three
    //    sheet parameters this port's own paper ground added on top of it --
    "paper_strength"        => paper_strength,        0.0,   1.0,  "Parchment";
    "paper_grain"           => paper_grain,           0.0,   0.2,  "Paper grain";
    "paper_mottle"          => paper_mottle,          0.0,   0.2,  "Paper mottle";
    "paper_wash"            => paper_wash,            0.0,   0.6,  "Paper wash";
    "stipple_strength"      => stipple_strength,      0.0,   1.0,  "Forest stipple";
    "border_width_frac"     => border_width_frac,     0.0,   0.06, "Plate border";
    // -- Reference Rendering-advanced ▸ Geology materials (`geologyR`) --
    "litho_strength"        => litho_strength,        0.0,   1.0,  "Geology tint";
    "litho_exposure"        => litho_exposure,        0.0,   1.0,  "Bedrock exposure";
    "local_contrast"        => local_contrast,        0.0,   1.0,  "Local contrast";
    // -- `GUI_GAP_REGISTER.md` CA-02's colour relief (no reference counterpart:
    //    the reference has no elevation ramp either) --
    "ramp_strength"         => ramp_strength,         0.0,   1.0,  "Colour relief";
    // -- Reference Paint brush ▸ Texture strength (`splat`) --
    "splat_strength"        => splat_strength,        0.0,   1.0,  "Texture strength";
}

/// The one tunable that is not an `f64`: the number of hillshade light
/// directions. Kept out of [`tunables!`] rather than stored as a float,
/// because `1` is not "a small amount of multidirectional" — it takes
/// `shade`'s dedicated single-light early return, which is what keeps
/// [`TerrainAppearance::js_reference`] bit-identical to JS. A caller sets it
/// through the same dictionary under the key below; `WorldGen` rounds.
// Same reason `js_reference` and `border_cover` already carry this: the
// golden-parity targets `#[path]`-include this file with no `lib.rs` to use it.
#[allow(dead_code)]
pub const TUNABLE_LIGHTS: (&str, f64, f64, &str) = ("relief_lights", 1.0, 12.0, "Light directions");

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
// Milestone 6: row-parallel. Each row runs its own independent sliding-window
// accumulator over its own `w` source values and writes its own `w`
// destination values, so the split reassociates nothing -- this stays
// bit-identical, which matters because `smooth_sea_h` (and therefore every
// ocean pixel on the JS-parity path) goes through here.
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64, wrap: bool) {
    let norm = 1.0 / (2 * r + 1) as f64;
    dst[..w * h].par_chunks_mut(w).enumerate().for_each(|(y, drow)| {
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
        for (x, d) in drow.iter_mut().enumerate() {
            *d = (acc * norm) as f32;
            let o = idx(x as i64 - r);
            let i = idx(x as i64 + r + 1);
            acc += src[row + i] as f64 - src[row + o] as f64;
        }
    });
}

// Deliberately **not** parallelized, unlike `box_h`: this pass walks columns,
// so each task would need `&mut` to a disjoint *stride* of every row, which
// rayon cannot express over a flat buffer without `unsafe`. The honest trade
// is that half of each separable blur is parallel and half is not; the
// alternative (blur-transpose-blur-transpose) would double the memory traffic
// and touch `smooth_sea_h`, which is on the JS-parity path. Revisit only if a
// profile says the serial half still dominates.
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
///
/// `pub(crate)` rather than private (`SCULPT_LIVE_SCOPE.md` milestone L0):
/// the L0 timing harness (`tests/sculpt_live_l0_bench.rs`) needs to call
/// this, `build_ao` and `build_hydro_wetness` in isolation to break down
/// `with_appearance`'s cost -- the whole point of L0 is measuring whether
/// these three precomputes really dominate rather than inferring it from
/// reading the code. Visibility-only change: still unreachable from outside
/// this crate (this crate is `cdylib`-only per `ARCHITECTURE.md`, so
/// `pub(crate)` is already as narrow as `pub` would be to any real
/// consumer), no behaviour differs, `golden_parity_render.rs` is untouched.
pub(crate) fn smooth_sea_h(src: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
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
/// `pub(crate)` for `SCULPT_LIVE_SCOPE.md` milestone L0 -- see
/// [`smooth_sea_h`]'s doc comment for why.
pub(crate) fn build_ao(field: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool, a: &TerrainAppearance) -> Vec<f32> {
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
/// comparable scale to the existing hydrology math), gate it to the cells
/// that really carry a channel so ordinary hillside sheet-flow doesn't tint
/// the whole map, and blur that into a soft halo rather than a hard one-cell
/// channel outline.
///
/// ## The `GUI_GAP_REGISTER.md` CA-11 retune (2026-08-24)
///
/// Both halves of the original were **tuned at a small grid and shrank with
/// resolution**, which is why the slider measured 1.216 % of pixels at
/// 512x384 and 0.002 % at the app's own 2048x1311:
///
/// * The gate was a `smoothstep(0.55, 0.88, …)` over the world's own
///   **min-max-normalized** log-flow range. That reads as adaptive, and it is
///   — but the quantity it normalizes is already scale-free (`flow / (gw*gh)`
///   is the *fraction of the map* a cell drains), so re-normalizing it bought
///   nothing and cost the threshold its meaning. In practice `lo` pinned to
///   the `1e-4` clamp floor and `hi` to the largest basin, putting the 0.55
///   knee at ~0.8 % of the map's area drained: the trunk river and nothing
///   else. Replaced with an **absolute** upstream-area gate
///   ([`WET_AREA_LO`]/[`WET_AREA_HI`]) — the same set of *channels* at every
///   resolution, which is what resolution-invariance means for a drainage
///   network.
/// * The blur then **diluted what survived**. A box blur conserves the mean,
///   so smearing a one-cell-wide line over a radius-`r` window drops its peak
///   by about `1 / (2r + 1)` — and `r` is `gw * 0.006`, so the dilution got
///   worse exactly as the grid got finer (3 cells at 512 wide, 12 at 2048).
///   The blur is what makes the halo soft and it stays; what is added is the
///   matching **gain that restores its peak**, `2r + 1`, clamped. A channel
///   corridor now reaches full strength at any grid size and falls off over
///   the same *world* distance.
///
/// Together these make the stage depend on the world and not on the raster:
/// see `appearance_ab_dump.rs`'s `hydro_wetness_visibility_by_resolution`,
/// which measures all three sizes and fails if any of them goes quiet again.
/// This deliberately **moves the shipped default look** (owner-authorised) —
/// `hydro_wet_strength` stays at its own `0.38`; what changed is that 0.38
/// now renders.
/// `pub(crate)` for `SCULPT_LIVE_SCOPE.md` milestone L0 -- see
/// [`smooth_sea_h`]'s doc comment for why.
pub(crate) fn build_hydro_wetness(flow: Option<&[f32]>, gw: usize, gh: usize, world: bool, a: &TerrainAppearance) -> Vec<f32> {
    let n = gw * gh;
    if a.hydro_wet_strength <= 0.0 {
        return vec![0f32; n];
    }
    let Some(flow) = flow else {
        return vec![0f32; n];
    };
    let denom = (gw * gh) as f64;
    let (lo, hi) = (WET_AREA_LO.ln(), WET_AREA_HI.ln());
    let gate: Vec<f32> = flow.iter().map(|&f| smoothstep(lo, hi, (((f as f64) / denom).max(1e-4)).ln()) as f32).collect();

    let rad = ((gw as f64 * a.hydro_wet_radius_frac).round() as i64).max(1);
    let gain = (2 * rad + 1) as f32;
    let mut out = blur_once(&gate, gw, gh, rad, world);
    for v in out.iter_mut() {
        *v = (*v * gain).clamp(0.0, 1.0);
    }
    out
}

/// Upstream area (as a fraction of the whole map) at which a cell starts to
/// read as "near water", and the area at which it is fully wet — the CA-11
/// gate, in the one unit that is the same at every grid size.
///
/// `6e-4` is roughly a minor tributary on a continent-sized landmass and
/// `8e-3` a named river; below the first is hillside sheet flow, which §13 is
/// explicit must **not** tint the map. Deliberately far lower than the
/// old normalized gate's effective `8e-3 … 1.1e-1`, which only ever caught
/// the single largest trunk.
///
/// The pair was picked by measurement, not by taste: `1e-3 … 1.2e-2` left the
/// working resolution at 0.67 % of pixels and `3e-4 … 5e-3` took it to 3.4 %,
/// which is a wet-valley wash rather than a river corridor. These land at
/// 1.42 % at 2048x1311 and 7.76 % at 512x384, at the shipped `0.38` strength.
///
/// Absolute rather than per-world on purpose, and the trade is stated: a world
/// whose basins are all smaller than `WET_AREA_LO` (a dense archipelago of
/// tiny islands) gets no wetness at all, where the old min-max gate would have
/// tinted its biggest stream whatever its size. That is the honest answer —
/// an island with no river has no river to tint — and it is the same choice
/// `build_ao` declines to make only because occlusion is a *relief* statistic,
/// which really is relative, while drainage area is an absolute one.
const WET_AREA_LO: f64 = 6.0e-4;
const WET_AREA_HI: f64 = 8.0e-3;

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

/// `multiSunFromNormal` (8356-8363) — the reference's four-light "Painter"
/// rig: a primary sun at the scene azimuth and a fixed 45° altitude, a fill
/// sun 90° round at 35°, a zenith light, and a constant ambient floor, at
/// weights 0.40 / 0.30 / 0.20 / 0.10 (Kennelly & Kimerling).
///
/// A literal port, constant for constant — including the two **hardcoded**
/// altitudes, which are the reference's own and are deliberately *not*
/// `sun_alt_deg`: the rig's softness comes from the fixed 45°/35° pair, and
/// substituting the scene's own sun altitude would silently make this a
/// different effect at any altitude but 40°. Only the azimuth is shared.
///
/// Distinct from this port's own [`build_lights`] multidirectional relief
/// (`relief_lights`, six evenly-spaced weighted lights): that one keeps the
/// primary dominant to reveal ridgelines parallel to the sun, this one
/// deliberately flattens toward a soft painterly wash with no black voids.
/// Both exist because they are different pictures and the reference names
/// this one as its own control.
pub fn multi_sun_from_normal(a: &TerrainAppearance, nx: f64, ny: f64, nz: f64) -> f64 {
    let az = a.sun_az_deg.to_radians();
    let a1 = 45.0_f64.to_radians();
    let az2 = az + std::f64::consts::FRAC_PI_2;
    let a2 = 35.0_f64.to_radians();
    let (l1x, l1y, l1z) = (a1.cos() * az.sin(), -a1.cos() * az.cos(), a1.sin());
    let (l2x, l2y, l2z) = (a2.cos() * az2.sin(), -a2.cos() * az2.cos(), a2.sin());
    let s1 = (nx * l1x + ny * l1y + nz * l1z).max(0.0);
    let s2 = (nx * l2x + ny * l2y + nz * l2z).max(0.0);
    let sz = nz.max(0.0);
    (0.40 * s1 + 0.30 * s2 + 0.20 * sz + 0.10).min(1.0)
}

/// `computeCoastDistance` (7398-7413) — a two-pass chamfer distance
/// transform giving every **ocean** cell its distance in cells from the
/// nearest land; land cells stay `0`. Drives the wave/foam contours.
///
/// World-wrap is ignored, exactly as the reference ignores it and for the
/// reference's own stated reason ("the effect is subtle decoration and a
/// one-cell seam in the bands is imperceptible") — matching rather than
/// improving, per `cartalith-rust-conventions`.
///
/// Built only when `npr.waves` is on; `RenderCtx` holds an empty `Vec`
/// otherwise, so the whole stage costs nothing at the default settings.
pub fn coast_distance(field: &[f32], gw: usize, gh: usize, sea: f64) -> Vec<f32> {
    const D1: f32 = 1.0;
    // The reference's own decimal literal (7399), deliberately not
    // `f32::consts::SQRT_2`: in JS it is an f64 constant that lands in a
    // `Float32Array`, so writing it exactly as the reference writes it is what
    // makes the two provably the same rounding rather than approximately so.
    #[allow(clippy::approx_constant, clippy::excessive_precision)]
    const D2: f32 = 1.4142135623730951;
    const INF: f32 = 1e9;
    let n = gw * gh;
    let mut d: Vec<f32> = field
        .iter()
        .map(|&h| if (h as f64) < sea { INF } else { 0.0 })
        .collect();
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if d[i] == 0.0 {
                continue;
            }
            let mut m = d[i];
            if x > 0 {
                m = m.min(d[i - 1] + D1);
            }
            if y > 0 {
                m = m.min(d[i - gw] + D1);
            }
            if x > 0 && y > 0 {
                m = m.min(d[i - gw - 1] + D2);
            }
            if x + 1 < gw && y > 0 {
                m = m.min(d[i - gw + 1] + D2);
            }
            d[i] = m;
        }
    }
    for y in (0..gh).rev() {
        for x in (0..gw).rev() {
            let i = y * gw + x;
            if d[i] == 0.0 {
                continue;
            }
            let mut m = d[i];
            if x + 1 < gw {
                m = m.min(d[i + 1] + D1);
            }
            if y + 1 < gh {
                m = m.min(d[i + gw] + D1);
            }
            if x + 1 < gw && y + 1 < gh {
                m = m.min(d[i + gw + 1] + D2);
            }
            if x > 0 && y + 1 < gh {
                m = m.min(d[i + gw - 1] + D2);
            }
            d[i] = m;
        }
    }
    debug_assert_eq!(d.len(), n);
    d
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
    /// `_coastDCache` (8440) — per-cell distance in cells to the nearest
    /// land, for the wave/foam contours. **Empty** unless `npr.waves` is on,
    /// which is what makes the whole stage free at the default settings
    /// (`cell_color` tests the length, not a flag, so the two can never
    /// disagree).
    coast_d: Vec<f32>,
    /// The renderer's colour data/shading constants (`TerrainAppearance`'s
    /// own doc comment). Settable via `with_appearance` as of milestone 2 —
    /// still not wired to any UI/`#[func]` (that's `GUI_SHELL_SCOPE.md`'s
    /// own deferred terrain-appearance panel), but the golden-parity test
    /// uses it to pin the exact JS path.
    appearance: TerrainAppearance,
    /// Real ground-texture channels for splat blending (milestone 7),
    /// `None` by construction — attach with [`Self::with_splat`]. Never set
    /// by `golden_parity_render.rs`, so the pinned JS-parity path never
    /// enters `land_color`'s splat branch at all.
    splat: Option<SplatTextures<'a>>,
    /// Per-cell rock type (`cartalith_civ::build_lithology`, indices per
    /// [`LITHO_PALETTE_ORDER`]), `None` by construction — attach with
    /// [`Self::with_lithology`]. `None` is the honest state for a **loaded
    /// save**, whose format stores none of the tectonic substrate
    /// (`SAVEFILE_COMPAT.md`; `CivData`'s own doc comment says the same
    /// thing about the civilisation layer), exactly as `flow` is already
    /// `None` there — the geology stages then do nothing rather than
    /// inventing a rock type.
    lithology: Option<&'a [u8]>,
    /// The three Cartography paint-override grids (`paintBiome`/
    /// `paintTerrain`/`paintSplat`, `0` = unpainted, else a 1-based palette
    /// index), `None` by construction — attach with [`Self::with_paint`].
    /// `None` and an all-zero grid are the same picture, which is what keeps
    /// every existing caller (and `golden_parity_render.rs`, which never
    /// calls the builder) byte-identical.
    paint_biome: Option<&'a [u8]>,
    paint_terrain: Option<&'a [u8]>,
    paint_splat: Option<&'a [u8]>,
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
        let coast_d = if appearance.npr.waves { coast_distance(field, gw, gh, sea_level) } else { Vec::new() };
        RenderCtx { field, temperature, rainfall, flow, gw, gh, sea_level, world, lat_n, lat_s, sea_h, sea_shade, ao, hydro_wet, lights, coast_d, appearance, splat: None, lithology: None, paint_biome: None, paint_terrain: None, paint_splat: None }
    }

    /// Attach the world's real rock types (milestone 5, §12). A builder for
    /// the same reason `with_splat` is one: `golden_parity_render.rs`
    /// constructs its `RenderCtx` positionally, and three milestones of
    /// leaving that file untouched is a property worth keeping. `len` is
    /// checked against the grid rather than trusted — a mismatched field
    /// would otherwise index out of bounds inside the render loop, and a
    /// panic there crosses the gdext boundary (`cartalith-rust-conventions`).
    #[allow(dead_code)]
    pub fn with_lithology(mut self, lithology: &'a [u8]) -> Self {
        if lithology.len() == self.gw * self.gh {
            self.lithology = Some(lithology);
        }
        self
    }

    /// Attach real ground-texture channels (milestone 7). A builder method
    /// rather than a `new`/`with_appearance` parameter so every existing
    /// caller — including `golden_parity_render.rs`, which constructs a
    /// `RenderCtx` positionally — keeps compiling unchanged; splat stays
    /// `None` (inert) unless a caller opts in explicitly.
    // Used by `lib.rs`'s real render path; the test target (`#[path]`-includes
    // this file standalone and never calls it) alone sees it as unreachable,
    // same situation as `js_reference()` in reverse.
    #[allow(dead_code)]
    pub fn with_splat(mut self, splat: SplatTextures<'a>) -> Self {
        self.splat = Some(splat);
        self
    }

    /// Attach the Cartography paint-brush override grids
    /// (`cartalith_spatial::PaintLayer`'s committed cells, via
    /// `paint_bridge::PaintEditor`). Each must be `gw * gh` long or it is
    /// dropped — a short grid would otherwise index-panic per pixel inside
    /// the parallel render, and `cartalith-rust-conventions`' own rule is
    /// that a panic must not cross the gdext boundary.
    ///
    /// A builder method for the same reason [`Self::with_splat`] is one:
    /// every existing caller, including the five `#[path]`-including test
    /// targets, keeps compiling and keeps rendering byte-identically.
    #[allow(dead_code)]
    pub fn with_paint(mut self, biome: Option<&'a [u8]>, terrain: Option<&'a [u8]>, splat: Option<&'a [u8]>) -> Self {
        let n = self.gw * self.gh;
        let fit = |g: Option<&'a [u8]>| g.filter(|c| c.len() >= n);
        self.paint_biome = fit(biome);
        self.paint_terrain = fit(terrain);
        self.paint_splat = fit(splat);
        self
    }

    /// The reference's own per-cell read of the three grids — `paintBiome[i]`
    /// etc. on the main map (8168), which is exactly this index because the
    /// main render walks whole cells.
    fn paint_at(&self, i: usize) -> PaintOverride {
        let g = |a: Option<&[u8]>| a.map_or(0, |c| c[i]);
        PaintOverride { bio: g(self.paint_biome), ter: g(self.paint_terrain), splat: g(self.paint_splat) }
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

    /// `gradAt` (7586) — ∇field, the hachure's downslope direction. Same
    /// neighbours and same world-wrap asymmetry as `slope_at` (which is this
    /// vector's magnitude); kept separate because the reference computes it
    /// only when hachure is on, and so does `cell_color`.
    fn grad_at(&self, x: usize, y: usize) -> (f64, f64) {
        let (gw, gh) = (self.gw, self.gh);
        let (xl, xr) = if self.world {
            ((x + gw - 1) % gw, (x + 1) % gw)
        } else {
            (
                if x > 0 { x - 1 } else { x },
                if x + 1 < gw { x + 1 } else { x },
            )
        };
        let u = if y > 0 {
            self.h(x, y - 1)
        } else {
            self.h(x, y)
        };
        let d = if y + 1 < gh {
            self.h(x, y + 1)
        } else {
            self.h(x, y)
        };
        ((self.h(xr, y) - self.h(xl, y)) * 0.5, (d - u) * 0.5)
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

        // `macroShade` (8371): the multi-sun rig replaces the *macro* shade
        // only, which is what `step == 1` is. The reference's `shadeFactor2`
        // (meso, `step == 3`) and `seaShadeFrom` both stay single-sun, so
        // this branch is deliberately narrower than "if multi_sun".
        if self.appearance.npr.multi_sun && step == 1 {
            return multi_sun_from_normal(&self.appearance, nx, ny, nz);
        }

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

    /// The rock type under a cell (milestone 5, §12), sampled through a
    /// **coherent positional jitter** of a couple of cells rather than
    /// straight.
    ///
    /// `build_lithology` is categorical and single-pass, so its contacts are
    /// exact grid curves. Sampled straight, a granite/limestone boundary
    /// would render as a clean vector line across the terrain — §30's
    /// "artificial outlines" and "hard biome borders" in one. Displacing the
    /// lookup by a low-frequency noise field breaks that line into a ragged
    /// natural contact at roughly a ten-cell wavelength, which is exactly
    /// what `bio_jitter` already does for the reference's own biome
    /// classification (`state.viz.sharpBiomes`), so this is the renderer's
    /// established idiom rather than a new one.
    ///
    /// Deterministic (§27) — a pure function of the cell coordinates.
    fn litho_at(&self, x: usize, y: usize) -> Option<u8> {
        let lith = self.lithology?;
        if self.appearance.litho_strength <= 0.0 && self.appearance.litho_exposure <= 0.0 {
            return None;
        }
        let (xf, yf) = (x as f64, y as f64);
        let jx = (vnoise(xf * 0.09, yf * 0.09, 81) - 0.5) * 4.4;
        let jy = (vnoise(xf * 0.09, yf * 0.09, 83) - 0.5) * 4.4;
        let (gw, gh) = (self.gw as i64, self.gh as i64);
        let sx = (xf + jx).round() as i64;
        let sy = (yf + jy).round() as i64;
        // X wraps in world mode (the same asymmetry `slope_at` has); Y never
        // does, since the poles are not adjacent.
        let sx = if self.world { ((sx % gw) + gw) % gw } else { sx.clamp(0, gw - 1) };
        let sy = sy.clamp(0, gh - 1);
        Some(lith[(sy * gw + sx) as usize])
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

/// The rock-type order this renderer's lithology palettes are indexed by —
/// **must stay identical to `cartalith_civ::LITH_KEYS`**, which is the
/// vocabulary `build_lithology` actually emits.
///
/// Spelled out here as data rather than imported because `render.rs` is
/// `#[path]`-included standalone by two test targets (see `SplatChannel`'s
/// own note); `appearance_ab_dump.rs` — which can see both crates — asserts
/// the two orders match, so this is a checked duplicate, not a hopeful one.
#[allow(dead_code)]
pub const LITHO_PALETTE_ORDER: [&str; 7] = ["granite", "basalt", "andesite", "limestone", "sandstone", "shale", "metamorphic"];

/// The palette for one `LITHO_PALETTE_ORDER` index. Out-of-range falls back
/// to granite — `build_lithology` cannot emit anything else, but a save
/// format or a future vocabulary extension could, and a renderer is the
/// wrong place to panic (`cartalith-rust-conventions`: a panic crossing the
/// gdext boundary takes the Godot process down).
fn litho_palette(a: &TerrainAppearance, lith: u8) -> &[Rgb; 3] {
    match lith {
        1 => &a.rock_basalt,
        2 => &a.rock_andesite,
        3 => &a.rock_limestone,
        4 => &a.rock_sandstone,
        5 => &a.rock_shale,
        6 => &a.rock_metamorphic,
        _ => &a.rock_granite,
    }
}

/// The rock material's colour: the reference's climate/relief heuristic
/// (`rock_col`), blended toward the palette of the rock actually under the
/// cell (`TERRAIN_APPEARANCE_RESEARCH.md` §12).
///
/// Early-returns the untouched heuristic when there is no lithology field
/// (a loaded save) or when the blend is off — `js_reference()`'s state —
/// rather than relying on `mix(.., 0.0)` evaluating to a no-op, the same
/// discipline `relief_lights <= 1` and the three atlas stages already
/// follow.
fn rock_material_col(a: &TerrainAppearance, t: f64, m: f64, r: f64, tt: f64, lith: Option<u8>) -> Rgb {
    let base = rock_col(a, t, m, r, tt);
    let Some(li) = lith else {
        return base;
    };
    if a.litho_strength <= 0.0 {
        return base;
    }
    mix(base, ramp3(litho_palette(a, li), tt), a.litho_strength)
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
fn land_color(appearance: &TerrainAppearance, t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64, sh: f64, sh_m: f64, vig: f64, ao: f64, hydro_wet: f64, lith: Option<u8>, grad: (f64, f64), x: usize, y: usize, gw: usize, gh: usize, splat: Option<&SplatTextures>, paint: PaintOverride) -> Rgb {
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
    add(&mut c, rock_material_col(appearance, te, me, r, tt, lith), w.rock);
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

    // Milestone 7 (`ASSET_LIBRARY_SCOPE.md`): real ground-texture splat.
    // `sp()`'s six calls (reference lines 7773-7778), re-tinting each pack
    // channel by the *same* material-weight fraction and procedural ramp
    // colour `land_color` already computed above — no new logic, splat is a
    // read-only consumer of `w`/`te`/`me`/`r`/`tt`. `_splatK=0` (no pack /
    // strength 0) is a byte-untouched no-op, matching the reference's own
    // "the add() mix above is byte-untouched" comment.
    if let Some(splat) = splat
        && appearance.splat_strength > 0.0
    {
        let mut acc: Rgb = (0.0, 0.0, 0.0);
        let mut cov = 0.0;
        if paint.splat > 0 {
            // The reference's paint-brush Splat override (7765-7773): force
            // **one** slot at full coverage, bypassing the
            // `materialWeights`-weighted multi-slot blend below entirely —
            // "a genuine 'paint this ground texture here' rather than a
            // tint". `sp()` is reused unchanged, and an empty slot is still
            // a no-op (`splat_sample` early-returns on `None`), which is the
            // reference's own "empty slots stay procedural" contract.
            //
            // Indices are 1-based into `SPLAT_PAINT_SLOTS`
            // (`["grass","rock","sand","snow","wetland","canopy"]`), whose
            // order this match reproduces; anything out of range paints
            // nothing rather than wrapping onto the wrong channel.
            let (tex, ramp) = match paint.splat {
                1 => (splat.grass, grass_col(appearance, te, me, r, tt)),
                2 => (splat.rock, rock_material_col(appearance, te, me, r, tt, lith)),
                3 => (splat.sand, sand_col(appearance, te, me, tt)),
                4 => (splat.snow, snow_col(appearance, te, tt)),
                5 => (splat.wetland, wetland_col(appearance, te, w.is_mangrove, tt)),
                6 => (splat.canopy, forest_col(appearance, te, w.meff, tt)),
                _ => (None, (0.0, 0.0, 0.0)),
            };
            if let Some(tex) = tex {
                splat_sample(tex, ramp, 1.0, x, y, &mut acc, &mut cov);
            }
        } else {
            if let Some(tex) = splat.grass {
                splat_sample(tex, grass_col(appearance, te, me, r, tt), w.grass, x, y, &mut acc, &mut cov);
            }
            if let Some(tex) = splat.rock {
                splat_sample(tex, rock_material_col(appearance, te, me, r, tt, lith), w.rock, x, y, &mut acc, &mut cov);
            }
            if let Some(tex) = splat.sand {
                splat_sample(tex, sand_col(appearance, te, me, tt), w.sand, x, y, &mut acc, &mut cov);
            }
            if let Some(tex) = splat.snow {
                splat_sample(tex, snow_col(appearance, te, tt), w.snow, x, y, &mut acc, &mut cov);
            }
            if let Some(tex) = splat.wetland {
                splat_sample(tex, wetland_col(appearance, te, w.is_mangrove, tt), w.wetland, x, y, &mut acc, &mut cov);
            }
            if let Some(tex) = splat.canopy {
                splat_sample(tex, forest_col(appearance, te, w.meff, tt), w.canopy, x, y, &mut acc, &mut cov);
            }
        }
        if cov > 0.0 {
            let k = appearance.splat_strength * cov;
            c.0 = c.0 * (1.0 - k) + (acc.0 / cov) * k;
            c.1 = c.1 * (1.0 - k) + (acc.1 / cov) * k;
            c.2 = c.2 * (1.0 - k) + (acc.2 / cov) * k;
        }
    }

    // Milestone 5 (`TERRAIN_APPEARANCE_RESEARCH.md` §12): bedrock showing
    // through thin soil. §12's complaint is that "a mountain should not
    // simply become brown because it is high" — its visible material should
    // emerge from the world model. The rock *fraction* already does emerge
    // (from `material_weights`, untouched here); what didn't was the rock's
    // *identity*, and the fact that a grassed slope over shattered basalt
    // does not look like the same slope over limestone.
    //
    // Exposure is built from §12's own list, using only values already in
    // hand: slope (soil sheds), vegetation potential (`w.c` — root mat and
    // litter hide the parent rock), and effective moisture (deep wet soils
    // bury it). It is scaled by the cover fraction that is *not* already
    // rock or snow, so it is self-limiting: where the surface reads as bare
    // rock it changes nothing, and it never bleeds through an icecap.
    //
    // Every gate is a `smoothstep`, so there are no hard material borders
    // (§30) — and the lithology index itself is sampled through a coherent
    // positional jitter in `cell_color`, so a geological contact reads as a
    // ragged natural boundary rather than a vector line.
    //
    // **Slope is normalized by grid width here, unlike everywhere else in
    // this file.** `slope_at` is a per-*cell* height difference, so the same
    // mountain measures ~6x steeper at 512² than at 2048² — measured, not
    // assumed: median land slope over the Classic test world is 0.00354 at
    // 512² and 0.00054 at 2048². The reference's own `material_weights`
    // normalizers (`slope/0.04`, `slope/0.08`) inherit that dependence, and
    // they are golden-verified so they stay exactly as they are; but a *new*
    // threshold written in raw slope units would have silently gated this
    // stage down to the steepest ~5% of land at the resolution the app
    // actually runs at, which is how an effect ends up passing every
    // mechanical check and being invisible on screen. `slope * gw` is this
    // project's own established normalization for exactly this
    // (`cartalith_civ::build_slope_field` stores `slopeAt(x,y)*GW`).
    if appearance.litho_exposure > 0.0
        && let Some(li) = lith
    {
        let steep = smoothstep(1.5, 9.0, slope * gw as f64);
        let bare = smoothstep(0.62, 0.10, w.c);
        let thin = smoothstep(0.55, 0.15, me);
        let cover = clamp01(1.0 - w.rock - w.snow);
        let e = appearance.litho_exposure * steep * bare * (0.40 + 0.60 * thin) * cover;
        if e > 0.0 {
            let lc = ramp3(litho_palette(appearance, li), tt);
            c.0 += (lc.0 - c.0) * e;
            c.1 += (lc.1 - c.1) * e;
            c.2 += (lc.2 - c.2) * e;
        }
    }

    // `GUI_GAP_REGISTER.md` CA-02: the elevation colour ramp, blended over the
    // finished *material* colour and **before** the light curve below, so the
    // hillshade, AO, paper, haze, vignette and the Painter block all still act
    // on it. That ordering is the whole difference between a hypsometric tint
    // over shaded relief (the atlas construction) and a flat elevation key
    // pasted on top of a map. Land only: `land_color` is the land branch, and
    // water already has its own depth-keyed ramp in `sea_color_core`.
    //
    // Placed after splat and bedrock exposure so a loaded pack's ground
    // texture is tinted by the ramp rather than painted over it, and before
    // the beach blend below so a shoreline keeps its sand whatever the ramp's
    // bottom stop is -- the beach is a *material* fact about the coast, not an
    // elevation band.
    //
    // Skipped entirely at `0.0`, which is `default()` and `js_reference()`,
    // the same dedicated-branch rule every stage since milestone 2 follows.
    //
    // The stop's own alpha multiplies the strength rather than replacing it:
    // the slider stays "how far the ramp takes over" for the whole layer and
    // the alpha is "how far this band participates", which composes. A default
    // ramp is opaque throughout, so `k` is exactly `ramp_strength` and nothing
    // about the 2026-08-24 look moved.
    if appearance.ramp_strength > 0.0
        && let Some(rc) = appearance.ramp.sample(r)
    {
        let k = appearance.ramp_strength * rc.3;
        if k > 0.0 {
            c.0 += (rc.0 - c.0) * k;
            c.1 += (rc.1 - c.1) * k;
            c.2 += (rc.2 - c.2) * k;
        }
    }

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

    // The paint brush's Biome/Terrain tint (7897-7901) — the reference's own
    // slot for it: "after every other colour/lighting step and before the
    // Painter/NPR block so hand-drawn styles still apply consistently on top
    // of painted cells". A final alpha blend of the painted index's flat
    // palette colour over the *fully shaded* colour at weight `0.60`,
    // deliberately not a rewrite of the `material_weights` mix above, "so
    // hillshade/AO/crest/splat/haze still show through and painted cells
    // don't read as flat pasted stickers". Both layers can coexist on one
    // cell and are applied sequentially, Biome then Terrain.
    //
    // **`_paintedTex` is not reachable in this port and this is not a
    // silent gap**: the reference's v1.28 refinement blends a *pack*
    // texture's true colour instead of the flat swatch when the loaded pack
    // supplies one for that index, at the same 0.60 weight and the same
    // position. `pack.rs`'s own module doc records that `LoadedPack` parses
    // but does not decode the `biomes`/`terrains` families, so there is no
    // texture to reach for — which is exactly the reference's own
    // `_t || CART_BIOME_COLS[pBio-1]` fallback, the branch a pack-less world
    // takes there too. Nothing here diverges; the texture half becomes
    // reachable the day those two families are decoded.
    //
    // Water is deliberately untouched: this blend lives in `landColorCore`,
    // so `cell_color`'s sea branch never sees it. In the reference that is
    // moot (`_paintAt` is unconditionally land-gated), and in this port it
    // is the honest consequence of `Brush::land_only` being a toggle — a dab
    // placed on water with the gate off is stored, exported and previewed,
    // and simply has no place in the map's water colour.
    if !paint.is_empty() {
        let swatch = |table: &[(u8, u8, u8)], v: u8| if v == 0 { None } else { table.get(v as usize - 1).copied() };
        for p in [swatch(&CART_BIOME_COLS, paint.bio), swatch(&CART_TERRAIN_COLS, paint.ter)].into_iter().flatten() {
            l.0 += (p.0 as f64 - l.0) * 0.60;
            l.1 += (p.1 as f64 - l.1) * 0.60;
            l.2 += (p.2 as f64 - l.2) * 0.60;
        }
    }

    // The "Painter" NPR block (7903-7962) — land only (`r > 0`), and exactly
    // here: after every colour and lighting step, before the final
    // `ao * vignette`. See `apply_npr`. Off at every default, and the whole
    // call is skipped rather than entered and no-opped.
    let l = if r > 0.0 && npr_any(&appearance.npr) {
        apply_npr(appearance, l, r, slope, curv, grad, x, y, gw)
    } else {
        l
    };

    // `ao * vignette` (7959-7960). `ao` was a hardcoded `1.0` before
    // milestone 2 (the reference's AO/SVF/shadow fields are all off at its
    // defaults); it now carries `build_ao`'s cavity map, and is still
    // exactly `1.0` under `js_reference()`.
    let k = ao * vig;
    (l.0 * k, l.1 * k, l.2 * k)
}

/// The reference's `state.viz` **"Painter" NPR block** (7903-7962) — ten
/// opt-in hand-drawn styles, each with its own intensity, applied in the
/// reference's own order: watercolor wash → contour veins → ink edges →
/// hachure → cel → engraving → stipple → sepia → risograph → pointillism.
///
/// **A literal per-pixel port, constant for constant.** These are ordinary
/// arithmetic on the finished land colour — no per-frame state, no
/// neighbourhood, no texture sampling — so there is nothing here a shader
/// would do differently, and porting them literally keeps them inside the
/// `rayon`-parallel `cell_color` pass milestone 6 already built rather than
/// adding a second compositing stage.
///
/// Every style is skipped by its own `> 0.0` gate exactly as the reference
/// skips it, and the whole block is skipped on water (`r > 0.0`) and when
/// nothing at all is on — so `TerrainAppearance::default()` and
/// `js_reference()` are both bit-untouched.
///
/// **Position in the pipeline is the reference's**: after every colour and
/// lighting step and before the final `ao * vignette` multiply, which is why
/// `land_color` calls this where it does. Styles stack freely, and stacking
/// them is what the reference's own hint text invites ("stack them freely").
///
/// `js_round` rather than `f64::round` for the two rounding sites (the
/// contour's isoline index and cel's quantiser): JS rounds half toward `+∞`
/// and Rust rounds half away from zero, which differ for negative inputs, and
/// a colour channel can sit fractionally below zero after the fine-grain
/// term. `cartalith-rust-conventions`' "V8's libm is not Rust's", applied to
/// rounding.
#[allow(clippy::too_many_arguments)]
pub fn apply_npr(
    a: &TerrainAppearance,
    l: Rgb,
    r: f64,
    slope: f64,
    curv: f64,
    grad: (f64, f64),
    x: usize,
    y: usize,
    gw: usize,
) -> Rgb {
    let n = &a.npr;
    let (mut l0, mut l1, mut l2) = l;
    let (px, py) = (x as f64, y as f64);
    let gwf = if gw == 0 { 1.0 } else { gw as f64 };

    // D-watercolor: pigment pooling + paper granulation + edge blooms.
    if n.watercolor > 0.0 {
        let (fx, fy) = (px / gwf, py / gwf);
        let pool = fbm(fx * 7.0, fy * 7.0, 151) - 0.5;
        let gran = vnoise(fx * 180.0, fy * 180.0, 153) - 0.5;
        let m = 1.0 + n.watercolor * (0.18 * pool + 0.10 * gran);
        l0 *= m;
        l1 *= m;
        l2 *= m;
        let bloom = (curv.abs() * 40.0).min(1.0) * n.watercolor * 0.25;
        if bloom > 0.0 {
            l0 *= 1.0 - bloom * 0.40;
            l1 *= 1.0 - bloom * 0.35;
            l2 *= 1.0 - bloom * 0.30;
        }
    }

    // D-contours: constant-width elevation isolines, every fifth an index
    // line. `contour_m >= 5` takes the metre-based interval (the reference's
    // R5 addition); anything else takes its legacy `0.05`-of-relief one, and
    // `peak_m == 0` falls back to the reference's own `||4000`.
    if n.contours > 0.0 {
        let iv = if n.contour_m >= 5.0 {
            let peak = if n.peak_m > 0.0 { n.peak_m } else { 4000.0 };
            // `max` then `min`, not `clamp`: the reference writes
            // `Math.min(0.2, Math.max(0.001, ...))` and JS's `Math.min`/`max`
            // propagate NaN where `f64::clamp` has its own rule. Same picture
            // for every real input, and the same one for the unreal ones too.
            #[allow(clippy::manual_clamp)]
            (n.contour_m / peak).max(0.001).min(0.2)
        } else {
            0.05
        };
        let fpos = r / iv;
        let d = (fpos - cartalith_jsmath::js_round(fpos)).abs() * iv;
        let cw = (iv * 0.04).max(slope * 0.5);
        let mut t = 1.0 - (d / cw).min(1.0);
        t *= t;
        if t > 0.0 {
            let idx_line = if (cartalith_jsmath::js_round(fpos) as i64) % 5 == 0 {
                1.4
            } else {
                1.0
            };
            let k = (n.contours * 0.55 * t * idx_line).min(0.9);
            l0 *= 1.0 - k;
            l1 *= 1.0 - k;
            l2 *= 1.0 - k;
        }
    }

    // D-ink: pen outline on strong landform edges, with hand-drawn wobble.
    if n.ink > 0.0 {
        let (fx, fy) = (px / gwf, py / gwf);
        let wob = 0.7 + 0.6 * fbm(fx * 120.0, fy * 120.0, 161);
        let edge = (curv.abs() * 55.0 * wob).min(1.0) * (slope * 6.0).min(1.0);
        if edge > 0.18 {
            let k = (n.ink * 0.8 * (edge - 0.18)).min(0.9);
            l0 *= 1.0 - k;
            l1 *= 1.0 - k;
            l2 *= 1.0 - k;
        }
    }

    // D-hachure: downslope hatching, denser and darker on steeper ground.
    if n.hachure > 0.0 && (grad.0 != 0.0 || grad.1 != 0.0) {
        let sl_n = (slope / 0.08).min(1.0);
        if sl_n > 0.12 {
            let gl = grad.0.hypot(grad.1);
            let gl = if gl == 0.0 { 1.0 } else { gl };
            let u = px * (-grad.1 / gl) + py * (grad.0 / gl);
            let freq = 0.9 * (0.6 + 0.8 * sl_n);
            let stroke = (u * freq).sin().max(0.0) * sl_n;
            let k = (n.hachure * 0.6 * stroke).min(0.85);
            l0 *= 1.0 - k;
            l1 *= 1.0 - k;
            l2 *= 1.0 - k;
        }
    }

    // D-cel: posterize the lit colour into flat toon bands.
    if n.cel > 0.0 {
        let q = |c: f64| cartalith_jsmath::js_round(c / 255.0 * 4.0) / 4.0 * 255.0;
        l0 = l0 * (1.0 - n.cel) + q(l0) * n.cel;
        l1 = l1 * (1.0 - n.cel) + q(l1) * n.cel;
        l2 = l2 * (1.0 - n.cel) + q(l2) * n.cel;
    }

    // D-crosshatch: antique engraving — more hatch directions as the cell darkens.
    if n.crosshatch > 0.0 {
        let dark = 1.0 - (l0 * 0.3 + l1 * 0.59 + l2 * 0.11) / 255.0;
        if dark > 0.22 {
            const F: f64 = 0.7;
            let mut h = ((px + py) * F).sin().max(0.0);
            if dark > 0.42 {
                h = h.max(((px - py) * F).sin());
            }
            if dark > 0.62 {
                h = h.max((px * F * 1.4).sin());
            }
            let k = (n.crosshatch * 0.75 * h * ((dark - 0.22) * 2.5).min(1.0)).min(0.85);
            l0 *= 1.0 - k;
            l1 *= 1.0 - k;
            l2 *= 1.0 - k;
        }
    }

    // D-stipple: pen stippling — denser dark dots in darker regions.
    if n.stipple > 0.0 {
        let dark = 1.0 - (l0 * 0.3 + l1 * 0.59 + l2 * 0.11) / 255.0;
        let dot = vnoise(px * 0.6, py * 0.6, 171);
        if dot < dark * 0.85 {
            let k = (n.stipple * 0.65).min(0.8);
            l0 *= 1.0 - k;
            l1 *= 1.0 - k;
            l2 *= 1.0 - k;
        }
    }

    // D-sepia: the classic warm ochre/brown toning matrix.
    if n.sepia > 0.0 {
        let (rv, gv, bv) = (l0 / 255.0, l1 / 255.0, l2 / 255.0);
        let sr = ((rv * 0.393 + gv * 0.769 + bv * 0.189) * 255.0).min(255.0);
        let sg = ((rv * 0.349 + gv * 0.686 + bv * 0.168) * 255.0).min(255.0);
        let sb = ((rv * 0.272 + gv * 0.534 + bv * 0.131) * 255.0).min(255.0);
        l0 = l0 * (1.0 - n.sepia) + sr * n.sepia;
        l1 = l1 * (1.0 - n.sepia) + sg * n.sepia;
        l2 = l2 * (1.0 - n.sepia) + sb * n.sepia;
    }

    // D-risograph: shadow→indigo / highlight→amber duotone + halftone dots.
    if n.risograph > 0.0 {
        let lum = ((l0 * 0.3 + l1 * 0.59 + l2 * 0.11) / 255.0).min(1.0);
        let dot = ((px * 0.70).sin() * (py * 0.70).sin()).max(0.0) * 0.10;
        let cr = (15.0 * (1.0 - lum) + 245.0 * lum + dot * 90.0).min(255.0);
        let cg = (25.0 * (1.0 - lum) + 205.0 * lum + dot * 55.0).min(255.0);
        let cb = (85.0 * (1.0 - lum) + 130.0 * lum + dot * 20.0).min(255.0);
        l0 = l0 * (1.0 - n.risograph) + cr * n.risograph;
        l1 = l1 * (1.0 - n.risograph) + cg * n.risograph;
        l2 = l2 * (1.0 - n.risograph) + cb * n.risograph;
    }

    // D-pointillism: Seurat coloured dot field, per-dot hue variance.
    if n.pointillism > 0.0 {
        let d1 = vnoise(px * 0.28, py * 0.28, 203) - 0.5;
        let d2 = vnoise(px * 0.70, py * 0.70, 207) - 0.5;
        let cr = (l0 * (1.0 + d1 * 0.45 + d2 * 0.15)).clamp(0.0, 255.0);
        let cg = (l1 * (1.0 + d2 * 0.18)).clamp(0.0, 255.0);
        let cb = (l2 * (1.0 - d1 * 0.35 + d2 * 0.12)).clamp(0.0, 255.0);
        l0 = l0 * (1.0 - n.pointillism) + cr * n.pointillism;
        l1 = l1 * (1.0 - n.pointillism) + cg * n.pointillism;
        l2 = l2 * (1.0 - n.pointillism) + cb * n.pointillism;
    }

    (l0, l1, l2)
}

/// B4 coastal wave lines (8442-8443, 8555-8558) — concentric foam contours
/// hugging the shore and fading into deep water, brighter near the shore.
/// Water cells only; the caller checks that, exactly as `renderNow` does.
///
/// The band width and the contour period are both keyed to grid width, so
/// the foam is a fixed *world* size across this port's 512²-8192² range —
/// the reference's own `Math.max(8, GW/40)` / `Math.max(2.5, GW/180)`, floors
/// included, ported literally.
pub fn apply_waves(a: &TerrainAppearance, c: Rgb, cd: f64, gw: usize) -> Rgb {
    let wave_dist = if a.npr.wave_dist > 0.0 {
        a.npr.wave_dist
    } else {
        1.0
    };
    let band = (8.0_f64).max(gw as f64 / 40.0) * wave_dist;
    let per = (2.5_f64).max(gw as f64 / 180.0);
    if !(cd > 0.0 && cd < band) {
        return c;
    }
    let fade = 1.0 - cd / band;
    // `cd/WAVE_PER*Math.PI*2`, in that association: `x * PI * 2` is
    // `(x * PI) * 2` — one rounding then an exact doubling — which is not
    // always the same double as `x * TAU`.
    let crest = (cd / per * std::f64::consts::PI * 2.0)
        .sin()
        .max(0.0)
        .powi(3);
    let w = fade * crest * 0.5 * (0.5 + 0.5 * fade);
    (
        c.0 * (1.0 - w) + 222.0 * w,
        c.1 * (1.0 - w) + 236.0 * w,
        c.2 * (1.0 - w) + 246.0 * w,
    )
}

/// Is any per-pixel Painter style on? Read once per render rather than ten
/// times per pixel, and the single place `land_color` decides whether to pay
/// for `grad_at` at all.
fn npr_any(n: &Npr) -> bool {
    n.watercolor > 0.0
        || n.contours > 0.0
        || n.ink > 0.0
        || n.hachure > 0.0
        || n.cel > 0.0
        || n.crosshatch > 0.0
        || n.stipple > 0.0
        || n.sepia > 0.0
        || n.risograph > 0.0
        || n.pointillism > 0.0
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

    // Milestone 6 (§29): the fibre and the mottle each early-return on
    // **their own** zero, not just on `paper_strength`. The two of them are
    // four `vnoise` calls on every pixel of the sheet, ocean included —
    // measured at milestone 4 as the whole 598→915 ms jump at 2048² — and
    // they are what the `Performance`/`Balanced` tiers drop while keeping
    // the tint, the wash and the frame. This is the same per-stage gating
    // rule milestone 2 established (`relief_lights <= 1`), applied one level
    // finer; the arithmetic is unchanged when both are on, so
    // `TerrainAppearance::default()` stays bit-identical.
    let grain = if a.paper_grain > 0.0 {
        let tooth = vnoise(xf * 0.31, yf * 0.31, 61);
        // Stretched along Y: laid lines. Cheap, and it's the single cue that
        // reads as "sheet" rather than "noise overlay" at a glance.
        let laid = vnoise(xf * 0.27, yf * 0.075, 63);
        ((tooth - 0.5) * 0.55 + (laid - 0.5) * 0.45) * 2.0
    } else {
        0.0
    };
    let mottle_dev = if a.paper_mottle > 0.0 {
        let mottle = 0.65 * vnoise(xf / gwf * 5.0, yf / gwf * 5.0, 65) + 0.35 * vnoise(xf / gwf * 13.0, yf / gwf * 13.0, 67);
        (mottle - 0.5) * 2.0
    } else {
        0.0
    };

    let v = 1.0 + grain * a.paper_grain + mottle_dev * a.paper_mottle;
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
/// The frame is a **uniform number of cells on all four sides** (a real
/// atlas plate's margin is uniform, not proportional per axis), so it is
/// keyed to `gw` alone — which is also what keeps
/// `WorldGen::get_border_inset_frac`'s "fraction of texture width" contract
/// exact under the uniform fit `map_overlay.gd` applies.
///
/// The one-sided `gh` guard exists because `apply_border`/`border_cover`
/// measure distance as `min(dx, dy)`: on a plate much wider than it is tall,
/// a width-derived margin can exceed half the height and swallow the entire
/// sheet, producing a blank image rather than a framed one. It is deliberately
/// one-sided (`gh < gw` only) so that **every square and every tall grid keeps
/// exactly the width it had before non-square generation existed** — a
/// guard that also fired on square grids would silently change the frame at
/// small square resolutions.
pub fn border_width_cells(a: &TerrainAppearance, gw: usize, gh: usize) -> f64 {
    if a.border_width_frac <= 0.0 {
        return 0.0;
    }
    let w = (a.border_width_frac * gw as f64).max(10.0);
    if gh < gw { w.min(gh as f64 * 0.25) } else { w }
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
    let w = border_width_cells(a, gw, gh);
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
    let w = border_width_cells(a, gw, gh);
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

/// Local contrast (`TERRAIN_APPEARANCE_RESEARCH.md` §18), applied in place
/// to a finished tightly-packed `RGB8` raster.
///
/// **Why this one stage is not per-pixel.** Everything else in this file is
/// a pure function of one cell, which is what let milestones 2-4 stay inside
/// `cell_color`. §18 cannot be: "make neighbouring terrain materials
/// visually distinguishable" is a statement about a *neighbourhood* of the
/// finished colour, which does not exist until the whole raster does. So
/// this is a second pass over the output buffer, and `cell_color`'s
/// signature and behaviour are untouched — `golden_parity_render.rs` never
/// reaches this code at all, and is additionally off by parameter
/// (`local_contrast: 0.0` early-returns before allocating).
///
/// **How §18's three constraints are satisfied by construction, not by
/// tuning.**
///
/// - *No haloing.* The response `d · exp(-(d/knee)²)` **falls to zero** as
///   the luminance difference grows, so the strongest edges in the image
///   (coastline, snowline, the plate neatline) receive essentially no boost.
///   An unsharp mask's halo is an overshoot proportional to edge strength;
///   here the gain is inversely related to it, so there is nothing to
///   overshoot with.
/// - *No edge-detection artifacts.* The correction is **additive on all
///   three channels equally** — a pure luminance nudge. A multiplicative or
///   per-channel version would shift hue at boundaries, which is what makes
///   naive local contrast look like edge detection.
/// - *Avoid excessive sharpening.* The detail band is a wide box blur
///   (`local_contrast_radius_frac` of grid width, ~20 cells at the app's
///   own 2048²), not a 3×3 kernel, so this acts on material-sized regions
///   rather than on pixel edges.
///
/// It also fades out under the plate frame via `border_cover`, so the bare
/// margin's paper grain is never amplified — the same rule milestone 4's
/// own follow-up established for every overlay that draws over the raster.
// Called from `lib.rs` and the A/B harness; the golden test target compiles
// this file standalone and never calls it — same situation as `border_cover`.
#[allow(dead_code)]
pub fn apply_local_contrast(a: &TerrainAppearance, rgb: &mut [u8], gw: usize, gh: usize, world: bool) {
    if a.local_contrast <= 0.0 || gw == 0 || gh == 0 {
        return;
    }
    let n = gw * gh;
    if rgb.len() < n * 3 {
        return;
    }

    // Rec.709 luma of the finished image, in 0-255 levels. Milestone 6: this
    // and the correction loop below are `rayon`-parallel — both are
    // element-wise over the raster, so the result is bit-identical to the
    // serial version regardless of how the work is split (§27 determinism is
    // a property of the maths here, not of the schedule).
    let mut luma = vec![0f32; n];
    {
        let src: &[u8] = rgb;
        luma.par_iter_mut().enumerate().for_each(|(i, l)| {
            let o = i * 3;
            *l = (0.2126 * src[o] as f64 + 0.7152 * src[o + 1] as f64 + 0.0722 * src[o + 2] as f64) as f32;
        });
    }

    // Radius floored at 3 cells (below that this stops being *local*
    // contrast and becomes sharpening, which §18 forbids) and additionally
    // capped against the *short* axis: `local_contrast_radius_frac` is keyed
    // to `gw` like every other radius in this file, and on a very wide
    // non-square plate a width-derived radius can exceed the whole height,
    // which turns the "local" mean into a full-column average and the
    // detail band into global contrast.
    let rad = ((gw as f64 * a.local_contrast_radius_frac).round() as i64).max(3).min((gh as i64 / 4).max(3));

    // **A band-pass, not a high-pass** — and this is the difference between
    // local contrast and a noise amplifier.
    //
    // `luma - blur(luma)` sweeps in *everything* finer than the radius,
    // which in this renderer means milestone 4's paper grain (~3-cell
    // features) and the C¹ seams of the value-noise lattices under the
    // mottle and the stipple. Boosting those is precisely §30's "random
    // texture noise", and it was plainly visible as a faint quilting across
    // land and sea in the first version of this pass — found by looking at
    // a downsampled real dump, not by any statistic, the same way milestone
    // 2's AO speckle and milestone 4's halftone stipple were.
    //
    // Subtracting a small blur instead of the raw image band-limits the
    // detail from below as well as above, so the boosted band is the
    // *material* scale (roughly 6-40 cells at the app's 2048²) and the
    // sheet's own texture passes through untouched. Same precedent as
    // `build_ao`'s `r_fine` floor: coherent noise at a couple of cells is
    // indistinguishable from speckle, so no stage may key off it.
    // The two blurs read the same buffer and write their own, so they are
    // independent whole passes -- worth a `join` on top of `box_h`'s own
    // row-parallelism, since this stage measured as the largest single
    // remaining cost in the appearance pipeline once `cell_color` went
    // parallel (milestone 6's own cost table).
    let r_inner = (rad / 8).max(2);
    let (fine, blurred) = rayon::join(|| blur_once(&luma, gw, gh, r_inner, world), || blur_once(&luma, gw, gh, rad, world));

    let knee = a.local_contrast_knee.max(1e-3);
    let inv_knee2 = 1.0 / (knee * knee);
    rgb[..n * 3].par_chunks_mut(3).enumerate().for_each(|(i, px)| {
        let d = fine[i] as f64 - blurred[i] as f64;
        let mut delta = a.local_contrast * d * (-(d * d) * inv_knee2).exp();
        if delta == 0.0 {
            return;
        }
        let (x, y) = (i % gw, i / gw);
        let cover = border_cover(a, x, y, gw, gh);
        if cover > 0.0 {
            delta *= 1.0 - cover;
        }
        for c in px.iter_mut() {
            *c = (*c as f64 + delta).clamp(0.0, 255.0) as u8;
        }
    });
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
        // `surfaceColor`'s own hachure guard (8160): the gradient is derived
        // only when hachure is actually on, so the default path pays nothing.
        let grad = if ctx.appearance.npr.hachure > 0.0 { ctx.grad_at(x, y) } else { (0.0, 0.0) };
        land_color(&ctx.appearance, t, m, slope, r_frac, twi, asp, curv, ctx.macro_shade(x, y), ctx.meso_shade(x, y), ctx.vignette_at(x, y), ctx.ao[i] as f64, ctx.hydro_wet[i] as f64, ctx.litho_at(x, y), grad, x, y, ctx.gw, ctx.gh, ctx.splat.as_ref(), ctx.paint_at(i))
    };

    // B4 coastal wave lines (8555-8558): foam contours hugging the shore and
    // fading into deep water, brighter near the shore. Water cells only, and
    // in the reference's own slot — after the colour branches, before the
    // parchment. `coast_d` is empty unless `npr.waves` is on, so this is a
    // single length test on every other path.
    let (r, g, b) = if h < ctx.sea_level && !ctx.coast_d.is_empty() {
        apply_waves(&ctx.appearance, (r, g, b), ctx.coast_d[i] as f64, ctx.gw)
    } else {
        (r, g, b)
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
