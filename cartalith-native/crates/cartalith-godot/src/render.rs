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
//! Excluded, and now down to one thing rather than four: the reference's
//! **vector river overlay**, `drawRiverWays`, a Catmull-Rom spline over
//! `_riverNet` drawn *on the canvas after* the raster. This port drew its
//! own equivalent (RV-2's strokes) the same way, in `map_overlay.gd`, from
//! the owner's 2026-09-22 ruling until 2026-09-27, when the owner found them
//! *"drawn on top of the style"*: since then the same strokes are rasterized
//! into the map ([`RiverLayer`], `river_stroke::rasterize`) and composited in
//! [`land_color`] before the Painter styles, the paper and the grade -- a
//! per-pixel stage after all, but not the reference's, which is why it is
//! reached only through an attached layer and never by `js_reference()`.
//! The channel-mask tint ([`channel_tint`]) is no longer baked for a
//! generated world anywhere -- screen, tiles or exports (RV-5, 2026-09-28:
//! the exports rasterize the same strokes at their own resolution,
//! [`paint_vector_rivers`]). Only a loaded save, which has no channel network
//! to draw, keeps it (`WorldGen::save_river_flag`).
//!
//! **Struck from this list on 2026-09-03, second pass**: geology microtexture
//! and dune ripples (`geo_micro`, [`litho_microtexture`]), the SVF and
//! cast-shadow fields ([`build_svf`], [`build_sun_shadow`], folded into the
//! reference's own `aoC` product), and the coast leg of the SDF tinting
//! ([`apply_coast_sdf`]). All four ship at `0.0`, so this changed no pixel of
//! `default()` or `js_reference()` — same convention, and the same reason to
//! strike rather than annotate, as the `rockSlope` paragraph below.
//!
//! **Struck on 2026-09-03, third pass — the other two SDF legs**
//! ([`build_river_sdf`] + [`apply_river_sdf`], [`build_biome_boundary_dist`] +
//! [`sdf_eco_k`]). `OUTSTANDING_WORK.md` §2.5 files them as *"depends on
//! subsystems the renderer's own doc says are not built"*, and the paragraph
//! it means is the one this text replaces. Both of that paragraph's blockers
//! were real and both dissolved on inspection rather than on new work:
//!
//! - The **private distance transform.** `cartalith_civ`'s `jfa_dist` is
//!   still private, and neither builder below needs it: `buildCoastSDF` and
//!   `buildRiverSDF` are the *same function over a different mask* in the
//!   reference itself, so [`build_river_sdf`] is exactly
//!   `cartalith_civ::build_coast_sdf` over the discharge mask — not an
//!   approximation of it, an algebraic identity, proved term by term in that
//!   function's own doc and pinned by a test. The unsigned biome distance is
//!   the same public call with the seed cells zeroed.
//! - The **missing map width.** `riverFlowThresh` needs one and a `RenderCtx`
//!   carried none; it now takes one through [`RenderCtx::with_map_scale`],
//!   the same builder shape `with_lithology` and `with_ground_tiles` already
//!   use. Not calling it leaves both fields empty, which is the off state.
//!
//! The water-body classification `buildBiomeRaster` needs is rebuilt inside
//! that builder from the field and rainfall a `RenderCtx` already holds, with
//! the identical `cartalith_civ::build_water_bodies` call the civilisation
//! pass makes (`lib.rs`), so the two cannot classify the same world
//! differently. It is paid only when the biome slider is up.
//!
//! Ported despite being extras: the `bioBlend` grey-desaturation blend
//! (0.90 default) and the edge haze fade, both unconditional in the
//! reference at its own default settings.
//!
//! **`rockSlope` refinement and wetness darkening were on the excluded list
//! above until they were ported on 2026-09-03**, behind defaults that leave
//! the shipped image byte-identical — the same convention the NPR paragraph
//! below records. They are struck from the list rather than left on it: this
//! module doc is `OUTSTANDING_WORK.md`'s own cited location for that row, so
//! an exclusion list naming a feature the file implements is a false reason of
//! exactly the class this project's unwired audit exists to catch.
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
//! ## The raster is separable (`GUI_GAP_REGISTER.md` CA-03/CA-04, RD-10, 2026-09-03)
//!
//! The three categories §7 calls Terrain, Colour relief and Hillshade are no
//! longer welded into [`land_color`] in a fixed order with fixed operators:
//! they are [`LayerStack`], an orderable, blendable, per-layer-opacity stack
//! that both consumer paths read from [`TerrainAppearance`]. The section above
//! [`RasterLayer`] carries the whole argument — what was actually in the way,
//! why this is one register-resident composite rather than three buffers (and
//! what those buffers would have cost), and why the shipped image stays
//! byte-identical **by branch** rather than by arithmetic. That is the
//! precondition CA-04 named; the panel that drives it is built
//! (`render_workspace.gd::_build_layer_stack`, `layer_stack_changed`).
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
//!
//! ## The four remaining reference stages, and the look layer (2026-08-24)
//!
//! The Excluded list above lost four members in one pass, all literal ports
//! and all in the reference's own pipeline slots: **ridge crests**
//! (`build_crest`/`apply_crest`, reference 8005-8023 and 8171), **surface
//! texture** and **ridged relief** (both in [`land_color`], 7841-7862), and
//! **curvature shading** (also in [`land_color`], 7870-7876). Three controls
//! with no reference counterpart came with them — [`TerrainAppearance::
//! biome_sat`], [`TerrainAppearance::relief_chroma`] and the finished raster's
//! [`apply_color_grade`] — plus [`TerrainAppearance::haze_strength`], which is
//! the reference's own `0.18` literal hoisted into the table.
//!
//! **None of it moved `default()` or `js_reference()`.** Every new field is at
//! a no-op value in `Default`, so the tier ladder still renders the image
//! milestones 1-7 tuned and `golden_parity_render.rs` is untouched and still
//! passes at its original `1e-4`. What changed the *shipped* look is a new
//! layer: a **named look** ([`LOOK_PRESETS`], [`TerrainAppearance::with_look`])
//! that sits on top of the quality tier, with `WorldGen` opening on
//! [`LOOK_VIBRANT`]. That split is deliberate — the tier decides what the
//! renderer *spends*, the look decides what the picture *is*, and a phone
//! answers only the first question differently.
//!
//! ### Where the owner's numbers and this port's state disagreed
//!
//! The specification for `Natural Vibrant` was written against the reference
//! HTML, where every enhancement slider sits at `0`. Three of them are not at
//! zero here, and two of those were left alone rather than lowered to the
//! reference-relative figure:
//!
//! - **Geology 25%** — this port's equivalent is the `litho_strength` /
//!   `litho_exposure` pair, already shipping at `0.62` / `0.55` from milestone
//!   5, which is *more* geology than the specified figure asks for. Lowering
//!   them to `0.25` would have made the vibrant look less geological than the
//!   plain tier, so the look leaves them at the tier's values.
//! - **AO 20%** — the tier ships `0.28`; the look takes it *down* to `0.20` as
//!   specified, which is coherent here because crests, curvature and ridged
//!   relief now carry the local relief the broad cavity map used to carry
//!   alone.
//! - **Wetness 12%** — the tier ships `0.38` after the same day's CA-11
//!   retune; the look takes it to `0.12` as specified. That is a real
//!   reduction of an owner-authorised value, made because this instruction is
//!   the later one and names the number explicitly.

use std::borrow::Cow;

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
///
/// **CA-19 (Ruling P, 2026-09-21): this is now the *default*, not the only,
/// biome colour table.** `land_color`'s paint blend reads
/// `TerrainAppearance::biome_cols` (initialised to this array byte-for-byte
/// in `Default::default()`), not this constant directly — `WorldGen::
/// set_biome_color` (`lib.rs`) can override individual entries at runtime.
/// This array itself never changes and stays the fallback every override is
/// diffed against; `paint_bridge::swatch_color`'s legend/picker path still
/// reads it directly on its default 3-arg form (see `swatch_color_with` for
/// its own override-aware sibling).
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
fn splat_sample(tex: &SplatChannel, ramp: Rgb, wt: f64, x: f64, y: f64, acc: &mut Rgb, cov: &mut f64) {
    if wt <= 0.0 {
        return;
    }
    let (tw, th) = (tex.w as i64, tex.h as i64);
    let sx = (((x.floor() as i64) % tw) + tw) % tw;
    let sy = (((y.floor() as i64) % th) + th) % th;
    let o = ((sy * tw + sx) * 4) as usize;
    let (r, g, b) = (tex.rgba[o] as f64, tex.rgba[o + 1] as f64, tex.rgba[o + 2] as f64);
    acc.0 += ramp.0 * r * tex.inv[0] * wt;
    acc.1 += ramp.1 * g * tex.inv[1] * wt;
    acc.2 += ramp.2 * b * tex.inv[2] * wt;
    *cov += wt;
}

/// One decoded ground tile for a painted Biome/Terrain index — a pack's
/// `biomes`/`terrains` family (`ASSET_LIBRARY_SCOPE.md` §1, reference v1.28).
///
/// **Deliberately not a [`SplatChannel`]: there is no `inv`, and that
/// asymmetry is the reference's.** Splat stores a per-channel inverse mean so
/// it can modulate a *procedural* ramp by `texel/mean`; a painted tile is
/// blended as **true colour** (`_paintedTex` reads `t.data[o]` raw), because
/// "dividing out a tile's absolute hue is right for splat and wrong for
/// paint" (`ASSET_LIBRARY_SCOPE.md`, reference comment at line 12246). Giving
/// this type an unused `inv` field would invite exactly the port error that
/// document warns about, so it does not have one.
///
/// Defined here rather than in `crate::pack` for the same reason
/// [`SplatChannel`] is — `render.rs` is `#[path]`-included standalone by ten
/// test targets, none of which has a sibling `pack` module to resolve a
/// cross-module `use` against.
#[derive(Clone)]
pub struct GroundTile {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// A loaded pack's two painted-ground families, borrowed for one render.
///
/// Each slice is indexed by `palette index - 1` — position `n` is
/// `PACK_BIOME_SLOTS[n]` / `PACK_TERRAIN_SLOTS[n]`, which the format freezes
/// 1:1 against `CART_BIOMES`/`CART_TERRAINS` (`slots.rs`' own "slot N here is
/// index N+1 in those arrays"). `None` at a position means the pack carries no
/// art for that index; the **empty slice** — [`GroundTiles::default`], and what
/// every caller with no pack passes — means the same thing for every index at
/// once, so the whole texture path collapses to the reference's own
/// `_t || CART_BIOME_COLS[pBio-1]` flat-swatch branch and a pack-less render is
/// byte-untouched.
#[derive(Clone, Copy, Default)]
pub struct GroundTiles<'a> {
    pub biomes: &'a [Option<GroundTile>],
    pub terrains: &'a [Option<GroundTile>],
}

/// `_paintedTex(fam, slots, idx, px, py)` (reference HTML 12187-12196) — the
/// painted index's pack tile, sampled at **one texel per grid cell, nearest,
/// wrapped**, returned as true colour on 0-255 channels.
///
/// The addressing is `splat_sample`'s, character for character, because the
/// reference's own two samplers are (`sp()`'s comment: "one texel per cell,
/// nearest, wrapped (px,py >= 0 in both render and bake)", which is why
/// `floor` and JS's `|0` truncation agree here).
///
/// `None` — the caller falls back to the flat palette swatch — for an
/// unpainted cell, an index past the family's vocabulary, a slot the pack has
/// no art for, or a degenerate 0-by-N image. That last guard is not
/// theoretical: `%` by zero panics in Rust where JS yields `NaN`, and a panic
/// here crosses the gdext boundary.
fn painted_tex(tiles: &[Option<GroundTile>], idx: u8, x: f64, y: f64) -> Option<Rgb> {
    if idx == 0 {
        return None;
    }
    let t = tiles.get(idx as usize - 1)?.as_ref()?;
    let (tw, th) = (t.w as i64, t.h as i64);
    if tw <= 0 || th <= 0 {
        return None;
    }
    let sx = (((x.floor() as i64) % tw) + tw) % tw;
    let sy = (((y.floor() as i64) % th) + th) % th;
    let o = ((sy * tw + sx) * 4) as usize;
    let px = t.rgba.get(o..o + 3)?;
    Some((px[0] as f64, px[1] as f64, px[2] as f64))
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
    /// `state.viz.cel` — posterises the **finished colour**, per channel, into
    /// four levels, texture noise included. A literal port pinned by
    /// `golden_parity_npr.rs`, kept as it is. It is **not** the "Cel / Toon"
    /// style preset: that one bands the light and flattens the albedo instead
    /// ([`TerrainAppearance::toon_strength`]), because posterising the colour
    /// measured as mottled blotches rather than a toon look (2026-09-27).
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
    /// The v2.70 "Village map" style (`RC_ENGINE_CHANGES.md`'s v2.70 row) — a
    /// flat, limited-palette look, opt-in and land **and** water (unlike every
    /// flag above it in this struct, which is water-only or land-only). A
    /// single on/off flag rather than an intensity, matching [`Self::waves`]/
    /// [`Self::multi_sun`]'s shape: the reference names it a *whole style*
    /// costing "one flag", not a slider.
    ///
    /// Read in two places: [`apply_npr`]'s own last step (the land half — see
    /// [`quantize_flat_palette`]) and [`sea_color_core`]'s final line (the
    /// water half, a REPLACEMENT rather than another mix-in, per the same
    /// row). Both quantise the already-fully-lit colour rather than the raw
    /// material mix, which is the row's own first rule: the lit colour has
    /// already absorbed slope, aspect, material and shading, so quantising it
    /// keeps every distinction and removes only the gradient.
    ///
    /// **`Cartalith_RC` was not present on this machine**, so the exact
    /// reference quantisation constant could not be read from
    /// `hash_gen1.js`'s `landColorCore`/`seaColorCore` — only the row's prose
    /// description was available. [`quantize_flat_palette`]'s own doc records
    /// the band count this port chose and why.
    pub village: bool,
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
    // Ruling BI (`LARGE_ITEM_RULINGS.md`, 2026-09-27): four presets'
    // land ramps, from `MAP_STYLE_RESEARCH.md` §4. Each is this document's own
    // reading of the cited palette, the same standing every other ramp in
    // this table already has -- none is a measured value.
    //
    // "Blueprint" -- §4.3: a cyanotype's Prussian-blue field, dark at the
    // shoreline toward a pale cyan at the summit, so the elevation gradient
    // still reads once `biome_sat` is driven low and the ramp takes over the
    // hue entirely.
    ("Blueprint", &[(0.00, (10, 34, 66)), (1.00, (206, 232, 248))]),
    // "Ink wash" -- §4.4: shan-shui's warm-grey ink values, not Blueprint's
    // cool blue -- the two share a two-stop mono shape but not a palette.
    ("Ink wash", &[(0.00, (54, 48, 42)), (1.00, (236, 226, 208))]),
    // "Night" -- §4.8: a dark-mode base distinct from `Dark ice`/`Dark
    // atlas` (both keyed to a specific biome, not a general night read) --
    // near-black lowlands rising only to a deep slate at the peaks, so the
    // map stays legibly dark at every elevation rather than blowing out at
    // the summits the way lightening toward white would.
    ("Night", &[(0.00, (9, 13, 20)), (0.30, (17, 25, 36)), (0.60, (29, 41, 54)), (1.00, (68, 84, 100))]),
    // "Vintage atlas" -- §4.6, the cited mid-century palette in stop order:
    // Philippine Brown `#582119`, Forest Brown `#906c54`, Muted Bronze
    // `#d0a772`, Bleach White `#fef1d7`.
    ("Vintage atlas", &[(0.00, (88, 33, 25)), (0.35, (144, 108, 84)), (0.65, (208, 167, 114)), (1.00, (254, 241, 215))]),
];

/// Ruling BI's "Nautical" preset (`MAP_STYLE_RESEARCH.md` §3/§4.7) — the one
/// genuinely new table the eight researched presets need: a depth-banded
/// bathymetric tint, in the same `(relative depth, RGB)` stop shape
/// [`RAMP_PRESETS`] uses, but blended into [`sea_color_core`] by
/// [`TerrainAppearance::sea_ramp_strength`] rather than into `land_color` by
/// `ramp_strength` — water's ramp, not land's.
///
/// Depth is `sea_color_core`'s own `depth` parameter: `0.0` at the shoreline,
/// `1.0` in the abyss. Colours follow §2.17's own reading of a real chart's
/// depth convention (NOAA/Amnautical, cited there) rather than the intuitive
/// "dark = deep": intertidal green at the coast, darkening through the very
/// shallow band, then lightening back through pale blue toward white in open
/// water, exactly as the source describes it.
pub const SEA_RAMP_NAUTICAL: &[(f64, (u8, u8, u8))] = &[
    (0.00, (130, 178, 122)),
    (0.15, (36, 78, 138)),
    (0.45, (118, 176, 214)),
    (1.00, (238, 246, 250)),
];

/// Linear interpolation over a static `(position, RGB)` stop table — the same
/// curve [`ElevationRamp::sample`] under [`RampMode::Linear`] would give, but
/// without building an [`ElevationRamp`] (an allocation and a sort) for a
/// fixed, already-sorted table sampled per pixel. Flat beyond both ends,
/// matching every other ramp in this file.
fn sample_ramp_table(stops: &[(f64, (u8, u8, u8))], t: f64) -> Rgb {
    let to_rgb = |c: (u8, u8, u8)| (c.0 as f64, c.1 as f64, c.2 as f64);
    let Some(&(first_at, first_c)) = stops.first() else { return (0.0, 0.0, 0.0) };
    if t <= first_at {
        return to_rgb(first_c);
    }
    let Some(&(last_at, last_c)) = stops.last() else { return (0.0, 0.0, 0.0) };
    if t >= last_at {
        return to_rgb(last_c);
    }
    for w in stops.windows(2) {
        let (a, b) = (w[0], w[1]);
        if t >= a.0 && t <= b.0 {
            let span = b.0 - a.0;
            let k = if span <= 0.0 { 1.0 } else { (t - a.0) / span };
            return mix(to_rgb(a.1), to_rgb(b.1), k);
        }
    }
    to_rgb(last_c)
}

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

// ===========================================================================
// The separable layer stack (`GUI_GAP_REGISTER.md` CA-03 / CA-04, RD-10)
// ===========================================================================
//
// # What was actually in the way
//
// CA-04's own reason — *"terrain, hillshade and colour relief are composited
// into one raster by `render.rs` before it crosses the boundary, so there are
// no separable outputs to order or blend"* — was true about the **order and
// the blend**, and misleading about the *pixels*. The three categories were
// never mixed together into an unrecoverable sum: [`land_color`] computes a
// material colour, blends the ramp over it with a hardcoded `lerp`, and then
// multiplies by a light factor. Both composites were already there; both had
// their operator and their position written into the source rather than into
// data. Separability here is therefore not "recover three images from one" —
// it is "stop hardcoding the two composite steps that already exist".
//
// # Why not N buffers
//
// The obvious reading of "separable outputs" is one RGB8 buffer per category,
// composited afterwards. Measured, that is the expensive answer for no gain:
// at the app's own 2048x1311 an RGB8 buffer is 2 048 x 1 311 x 3 =
// **8 054 784 B (7.68 MiB)**, so three categories cost **23.05 MiB** of new
// resident memory on every redraw — and the export path is where it stops
// being affordable, because `bake_dims(8192, ...)` on that grid is
// 8 192 x 5 244 = 42 958 848 px, i.e. **122.9 MiB per buffer, 368.8 MiB for
// three**, on top of the 23 B/px `export_raster.rs::PEAK_BYTES_PER_PIXEL`
// already budgets (**15 B/px when this was written** -- it undercounted
// `blur_once`, which allocates two buffers, not one, and both live until it
// returns; the corrected bound was checked against a measured 21.7 B/px). `MEMORY_OPTIMIZATION_SCOPE.md`'s whole premise is that this
// renderer's peak is what limits the export ceiling.
//
// The cheap version is enough for blend + reorder + opacity + visibility, and
// it is what this is: **one pass that composites per-category contributions in
// registers**, in the order and with the operators this table declares. Zero
// additional allocation, and it lands inside [`land_color`] — which is the
// single function both consumer paths already share ([`cell_color`] for the
// on-screen texture, [`BakeFields::pixel`] for every PNG export, region crop
// and layer export), so there is no path that can get this and no path that
// can miss it.
//
// # How the default stays byte-identical
//
// By **control flow**, on `apply_color_space`'s own precedent
// ([`ColorSpace::Srgb`] returns before a byte is read): [`LayerStack::
// is_default`] is tested once per pixel, and at the default the two composite
// sites run the *original expressions, unchanged* — the ramp's `lerp` stays in
// its mid-material slot and the light stays a bare `c * light`. Nothing about
// the shipped image passes through a blend-mode `match`, so no operator here
// can move it. That matters concretely: the composite works in 0-255 space, so
// a "multiply" written generically is `d * (s / 255)` with `s = light * 255`,
// and `light * 255.0 / 255.0` is not `light` to the last ulp. The identity is
// the branch, not the arithmetic.
//
// `tests/layer_stack.rs` pins both halves — a digest literal transcribed from
// the build *before* this existed, and a separate one for the bake path.
//
// # What this deliberately is not
//
// Land only. `sea_color_core` folds its own `0.82 + 0.18 * sh` shade into the
// water colour and has no ramp at all; the ramp has been land-only since CA-02
// for the same reason. §7's own layer list agrees — Water is its own row, a
// sibling of Terrain, not one of Terrain's three children.
//
// And it is the **precondition, not just the controls**: `WorldGen::{get_layer_stack,
// set_layer_stack, list_blend_modes}` bind it, and `render_workspace.gd`'s
// layer-stack panel (`_build_layer_stack`, `layer_stack_changed`) draws it.

/// One of the terrain raster's three separable categories.
///
/// These are `DCC_SHELL_SPEC.md` §7's own children of the Terrain row, in the
/// vocabulary CA-03 names them with. "Hand-drawn hillshade", §7's fourth
/// child, is not one: it is the Painter block ([`Npr`]), which is already
/// independently switchable through `WorldGen::set_npr` and composites after
/// every stage below rather than among them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum RasterLayer {
    /// The material colour — [`material_weights`]' six-fraction blend and
    /// everything that tints it (splat, bedrock, beach, texture, stipple).
    /// The bottom of the stack at the default, and the only category with an
    /// opaque source.
    Terrain,
    /// [`ElevationRamp`] sampled at relative land elevation, at
    /// [`TerrainAppearance::ramp_strength`] times the stop's own alpha.
    ColourRelief,
    /// The light curve `relief_ambient + relief_gain * sh^0.85`, as a neutral
    /// grey. Its source colour is `light * 255` per channel, so at the default
    /// `Multiply` it reproduces `c * light` exactly — which is what it is.
    Hillshade,
}

impl RasterLayer {
    /// The stable string a save file and a panel address this layer by.
    /// Deliberately not the `Debug` name: `Debug` is free to change.
    pub fn id(self) -> &'static str {
        match self {
            RasterLayer::Terrain => "terrain",
            RasterLayer::ColourRelief => "colour_relief",
            RasterLayer::Hillshade => "hillshade",
        }
    }

    /// The human label §7's layer list draws.
    pub fn label(self) -> &'static str {
        match self {
            RasterLayer::Terrain => "Terrain",
            RasterLayer::ColourRelief => "Colour relief",
            RasterLayer::Hillshade => "Hillshade",
        }
    }

    /// One layer by its [`id`](Self::id), or `None`. An unknown id is the
    /// caller's to report — the whole point of a keyed stack is that a panel
    /// written against a newer engine loses a row instead of silently
    /// addressing the wrong one.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "terrain" => Some(RasterLayer::Terrain),
            "colour_relief" => Some(RasterLayer::ColourRelief),
            "hillshade" => Some(RasterLayer::Hillshade),
            _ => None,
        }
    }
}

/// How one layer combines with everything under it.
///
/// The five a layer panel actually offers, in the order one lists them. All
/// five are the standard separable Porter-Duff-adjacent formulas evaluated per
/// channel in this file's own 0-255 space, then folded by the layer's alpha —
/// see [`BlendMode::apply`].
///
/// Deliberately small. A blend mode nobody can reach from a panel is the
/// enabled-and-inert shape this row exists to avoid, and the list can grow
/// without any of the arithmetic below changing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    /// `s` — the source replaces the backdrop, faded by alpha. The default for
    /// Terrain and Colour relief, and exactly the `lerp` CA-02 already used.
    Normal,
    /// `d * s / 255`. The default for Hillshade, and exactly `c * light` when
    /// the source is `light * 255`.
    Multiply,
    /// `255 - (255 - d)(255 - s) / 255`.
    Screen,
    /// Multiply under mid-grey, screen over it — contrast about the backdrop.
    Overlay,
    /// `d + s`, unclamped here (`cell_color` clamps once, at the end).
    Add,
}

/// Every blend mode, in the order a picker should show them. `Normal` first
/// because it is the identity a reader looks for at the top of the list.
pub const BLEND_MODES: &[&str] = &["Normal", "Multiply", "Screen", "Overlay", "Add"];

impl BlendMode {
    /// The name in [`BLEND_MODES`].
    pub fn name(self) -> &'static str {
        BLEND_MODES[self as usize]
    }

    /// One mode by name, case-insensitively, or `None` for a name this build
    /// does not have.
    pub fn from_name(s: &str) -> Option<Self> {
        BLEND_MODES.iter().position(|n| n.eq_ignore_ascii_case(s)).map(|i| match i {
            0 => BlendMode::Normal,
            1 => BlendMode::Multiply,
            2 => BlendMode::Screen,
            3 => BlendMode::Overlay,
            _ => BlendMode::Add,
        })
    }

    /// `dst` blended with `src` at `a`, per channel, in 0-255 space.
    ///
    /// The alpha fold is `d + (blended - d) * a`, the same form every other
    /// composite in this file uses, so `a == 1` is the raw operator and
    /// `a == 0` is the backdrop.
    ///
    /// **Nothing is clamped here.** `light` legitimately exceeds `1.0`
    /// (`relief_ambient + relief_gain` is `1.47` at the default), so a
    /// `Multiply` whose source is above 255 brightens — which is what the
    /// shipped light curve does and what this must not "fix". `cell_color`
    /// clamps once, at the end, exactly as it always has.
    // `pub(crate)` only so `tests/layer_stack.rs` — which `#[path]`-includes
    // this file as a child module of its own crate root — can assert the five
    // formulas against literal expected values rather than against themselves.
    pub(crate) fn apply(self, dst: Rgb, src: Rgb, a: f64) -> Rgb {
        let ch = |d: f64, s: f64| -> f64 {
            let b = match self {
                BlendMode::Normal => s,
                BlendMode::Multiply => d * s / 255.0,
                BlendMode::Screen => 255.0 - (255.0 - d) * (255.0 - s) / 255.0,
                BlendMode::Overlay => {
                    if d < 127.5 {
                        2.0 * d * s / 255.0
                    } else {
                        255.0 - 2.0 * (255.0 - d) * (255.0 - s) / 255.0
                    }
                }
                BlendMode::Add => d + s,
            };
            d + (b - d) * a
        };
        (ch(dst.0, src.0), ch(dst.1, src.1), ch(dst.2, src.2))
    }
}

/// One row of the stack: which category, whether it draws, how far, and how it
/// combines with what is under it.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayerEntry {
    pub layer: RasterLayer,
    pub visible: bool,
    /// `0..=1`. Multiplies whatever alpha the category itself carries — for
    /// Colour relief that is `ramp_strength * stop alpha`, so the two compose
    /// the same way CA-02's own strength and stop alpha already do.
    pub opacity: f64,
    pub blend: BlendMode,
}

/// The terrain raster's three categories, **bottom-first** — the order they
/// composite in, which is the reverse of the order §7's layer list draws them.
///
/// The binding (`WorldGen::get_layer_stack`) hands a panel the top-first view;
/// this type stores compositing order because that is what the renderer walks.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayerStack {
    entries: [LayerEntry; 3],
}

impl LayerStack {
    /// The shipped stack — the one arrangement whose render is the image every
    /// golden fixture and both digest literals in `tests/layer_stack.rs` pin.
    ///
    /// A `const` rather than a constructed `Default::default()`, because
    /// [`is_default`](Self::is_default) is called once per pixel and must not
    /// build a value to compare against.
    pub const DEFAULT: LayerStack = LayerStack {
        entries: [
            LayerEntry { layer: RasterLayer::Terrain, visible: true, opacity: 1.0, blend: BlendMode::Normal },
            LayerEntry { layer: RasterLayer::ColourRelief, visible: true, opacity: 1.0, blend: BlendMode::Normal },
            LayerEntry { layer: RasterLayer::Hillshade, visible: true, opacity: 1.0, blend: BlendMode::Multiply },
        ],
    };

    /// Bottom-first, the order [`composite`](Self::composite) walks.
    pub fn entries(&self) -> &[LayerEntry; 3] {
        &self.entries
    }

    /// Replace the whole stack. Refused (`false`, nothing changed) unless
    /// `entries` names each of the three categories **exactly once** — a stack
    /// missing a category is not a stack with that category hidden, and
    /// silently inventing the missing row is the "encode no value as a
    /// plausible value" mistake this project has now made five times. Opacity
    /// is clamped; nothing else can be out of range by construction.
    pub fn set(&mut self, mut entries: [LayerEntry; 3]) -> bool {
        let mut seen = [false; 3];
        for e in &entries {
            // `get_mut`, not `seen[..]`: a fourth `RasterLayer` variant added
            // without widening this array would otherwise panic here, and this
            // is reached from a `#[func]` — a panic crossing the gdext boundary
            // takes the Godot process with it (`cartalith-rust-conventions`).
            let Some(slot) = seen.get_mut(e.layer as usize) else { return false };
            *slot = true;
        }
        // **One test, not two.** With three slots and three categories,
        // "every category present" and "no category named twice" are the same
        // statement — a repeat necessarily leaves a gap. A separate duplicate
        // check was here first and mutation testing showed it survived every
        // mutant, because no input can reach it: it is a branch that cannot
        // fire, which is the same defect as prose describing behaviour that
        // does not happen. This is the requirement stated from the requirement's
        // side, which is also the form that mutates red.
        if !seen.iter().all(|s| *s) {
            return false;
        }
        for e in &mut entries {
            e.opacity = clamp01(e.opacity);
        }
        self.entries = entries;
        true
    }

    /// Whether this is [`LayerStack::DEFAULT`] exactly.
    ///
    /// **The whole byte-identity guarantee rests on this one test**, so it is
    /// a plain structural comparison of nine small values rather than anything
    /// cleverer: no cached flag that could go stale against a mutated field,
    /// and no tolerance.
    #[inline]
    pub fn is_default(&self) -> bool {
        self.entries == Self::DEFAULT.entries
    }

    /// The non-default composite: walk the stack bottom-first over a white
    /// ground, folding each visible category in with its own operator.
    ///
    /// Never called at the default — [`land_color`] takes the original
    /// expressions there instead. See this section's header for why that is a
    /// branch and not an optimisation.
    ///
    /// The ground is white rather than black or transparent because the only
    /// two operators that can be reached with nothing under them are `Normal`
    /// (which ignores the backdrop at full alpha) and `Multiply` (for which
    /// white is the identity) — so a stack with Terrain hidden and Hillshade
    /// multiplying renders the grey relief plate a reader means by "hillshade
    /// alone", rather than black.
    ///
    /// `relief` is the ramp sample the caller already had to compute, passed
    /// in rather than recomputed: `None` means this pixel's ramp contributes
    /// nothing (no stops, or `ramp_strength == 0`), and the Colour relief row
    /// is then skipped rather than folded at zero — the same "omit, do not
    /// default" rule the setter above follows.
    fn composite(&self, a: &TerrainAppearance, material: Rgb, light: f64, relief: Option<Rgba>) -> Rgb {
        let mut acc: Rgb = (255.0, 255.0, 255.0);
        for e in &self.entries {
            if !e.visible || e.opacity <= 0.0 {
                continue;
            }
            let (src, alpha) = match e.layer {
                RasterLayer::Terrain => (material, 1.0),
                RasterLayer::ColourRelief => match relief {
                    Some(rc) => ((rc.0, rc.1, rc.2), a.ramp_strength * rc.3),
                    None => continue,
                },
                RasterLayer::Hillshade => {
                    let g = light * 255.0;
                    ((g, g, g), 1.0)
                }
            };
            acc = e.blend.apply(acc, src, alpha * e.opacity);
        }
        acc
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        LayerStack::DEFAULT
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
    /// **CA-19 (`LARGE_ITEM_RULINGS.md`, Ruling P, 2026-09-21): the biome
    /// colour table, made user-editable.** `Default::default()` sets this to
    /// [`CART_BIOME_COLS`] byte-for-byte, so a `WorldGen` nobody has called
    /// `set_biome_color` on renders exactly what it rendered before this
    /// field existed — the parity guarantee the ruling requires ("with the
    /// change recorded at the symbol", here). `WorldGen::appearance()`
    /// (`lib.rs`) overlays `biome_col_overrides` onto this copy of the table
    /// per generation-config build; nothing else may mutate it. The land
    /// paint blend ([`land_color`]'s `paint` parameter, `_t ||
    /// CART_BIOME_COLS[pBio-1]`) reads `appearance.biome_cols` instead of
    /// the module constant directly, which is the one place in this port
    /// `CART_BIOME_COLS` was ever consumed as data rather than legend/swatch
    /// display (`paint_bridge::swatch_color_with` covers those; see its own
    /// doc for why they are a separate, still-constant-defaulting path).
    pub biome_cols: [(u8, u8, u8); 15],
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

    // ---- §16 (`TERRAIN_APPEARANCE_SCOPE.md`): multi-scale detail, as an
    //      explicit control set (`OUTSTANDING_WORK.md` §2.5) ----
    //
    // `land_color` already blends three hillshade bands -- `ctx.macro_shade`,
    // `ctx.meso_shade` and a per-pixel micro band synthesised from high-
    // frequency `vnoise` jitter over the macro band -- into one `sh_combined`
    // that drives the light curve. The three weights below were the hardcoded
    // literals `0.40`/`0.40`/`0.20` in that blend; hoisting them here does not
    // change what the blend computes; it exposes the same three numbers.
    /// Weight of the **macro** band -- `ctx.macro_shade`, the broad relief
    /// silhouette a multidirectional hillshade already computes.
    pub detail_macro_weight: f64,
    /// Weight of the **meso** band -- `ctx.meso_shade`, the same hillshade at
    /// a smoothed/coarser scale, which is what keeps large landforms legible
    /// once the macro band starts responding to per-cell noise.
    pub detail_meso_weight: f64,
    /// Weight of the **micro** band -- a per-pixel jitter of the macro band
    /// by high-frequency coherent noise (`land_color`'s own `n_hi`), which is
    /// what reads as fine surface grain rather than a smooth gradient.
    pub detail_micro_weight: f64,
    /// **LOD-D5** (`LOD_DETAIL_SCOPE.md`, *"scale-aware shading weights, and
    /// hydrology that resolves"*) — how far the four scale-dependent tile
    /// stages are allowed to move from the grid's own answer. `0.0` is the
    /// grid's answer at every zoom, which is what this port did before the
    /// milestone and is what [`Self::js_reference`] carries.
    ///
    /// # It is one number over four stages, and all four are continuous in it
    ///
    /// Every stage is written so that `0.0` is the **identity**, by
    /// arithmetic that cannot round to it:
    ///
    /// 1. the three [`detail_macro_weight`](Self::detail_macro_weight)-family
    ///    band weights are re-balanced by a transfer fraction that is a
    ///    product with this number;
    /// 2. the micro band cross-fades from value noise to the tile's own
    ///    relief residual at a mix that is a product with this number;
    /// 3. the crest stencil's ground width is `(1/cells_per_px)^k`, which is
    ///    exactly `1` at `k = 0`;
    /// 4. the per-tile river threshold is `river_thresh ·
    ///    cells_per_px^(2k)`, which is exactly `river_thresh` at `k = 0`.
    ///
    /// # And all four are the identity at grid resolution whatever it is
    ///
    /// The curve's own argument is `-log2(cells_per_px)` — **octaves past the
    /// simulation grid**, not an absolute ground size. At one tile pixel per
    /// coarse cell that argument is exactly `0` and every stage above returns
    /// the grid's own value, which is what makes
    /// `golden_parity_tile_biome.rs`'s screen-identity check unaffected by
    /// this milestone rather than merely close to unaffected. See
    /// [`detail_scale_octaves`] for why the parameterisation is in octaves
    /// and not in km.
    ///
    /// **A tile stage only.** `cell_color` passes [`land_color`]'s
    /// `scale` argument a literal `None`, so the shipped screen render is
    /// byte-identical to what it was before the milestone whatever this
    /// number is — which is why `every_tunable_is_load_bearing` exempts it
    /// from the grid sweep and proves it on a tile instead, exactly as it
    /// does for [`Self::ice_strength`].
    pub detail_scale_strength: f64,
    /// **v2.25's `tileShadeExag(bounds, W)`** (`RC_ENGINE_CHANGES.md` §4,
    /// followed under Ruling AP): a tile's hillshade exaggeration is scaled
    /// by tile pixels per coarse cell, `(W - 1) / bounds.w`, clamped at `1`
    /// so it never *reduces* the exaggeration. See [`tile_shade_exag`].
    ///
    /// Why: the main map shades with `exag / s` (`shadeFactor2`, v2.10 7684 —
    /// *"/s keeps slope magnitude consistent with 1-px sample"*), and the
    /// tile's own material slope is divided by `cx`/`cy` one line later, but
    /// the tile's shading term used the bare `exag`. A tile pixel spans `cx`
    /// coarse cells, so a height difference across it is `1/cx` times too
    /// small a slope, and relief flattened exactly as the view zoomed in.
    ///
    /// A `bool` rather than a strength: this is a unit correction, not a
    /// look, and there is no meaningful value between the two. `true` in
    /// `default()` (the DCC line's behaviour); `false` in
    /// [`Self::js_reference`], which is the v2.11 `ex = state.exag` that
    /// `tests/golden_parity_tile_biome.rs` pins byte for byte. **At one tile
    /// pixel per coarse cell or coarser the two are identical** — the clamp
    /// makes the factor exactly `1` — so the main map, the screen-identity
    /// check and LOD entry are untouched either way.
    pub tile_shade_exag_scaled: bool,

    /// **RV-4: a smooth, sub-cell shoreline** (owner, 2026-09-27: coasts and
    /// lake shores were *"pixel-stepped"*; `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`'s
    /// bar, no square artefacts at any zoom). When `true`:
    ///
    /// - `WorldGen::build_color_texture` also builds [`shore_field`], which the
    ///   shell's base map (`map_shore.gdshader`) contours at zero, below the
    ///   deep-zoom switch;
    /// - a deep-zoom tile's lake band ([`is_lake_pixel`]) is water where the
    ///   tile's own ground lies below the same field's surface, on the wet
    ///   side of its rain margin ([`shore_margins`]), instead of the
    ///   reference's `fq > 0.35`
    ///   membership cut and its flat-lake nearest-cell stamp, which drew the
    ///   z16 lake shore as square steps.
    ///
    /// Classification never changes either way: which cells are lake and
    /// sea is `cartalith_civ::build_water_bodies`' answer, and every cell
    /// centre keeps its class (the field's clamp, [`SHORE_EPS`]). A `bool`
    /// rather than a strength: a shoreline is where it is, there is nothing
    /// between. `true` in `default()`; `false` in [`Self::js_reference`], whose
    /// tile lake band `tests/golden_parity_tile_biome.rs` pins to the
    /// reference's v1.05 rule byte for byte.
    pub smooth_shores: bool,

    /// **RV-3: shade the valley along the drawn river line, not the carve**
    /// (`LARGE_ITEM_RULINGS.md` Ruling BD). When `true`, the screen texture,
    /// the deep-zoom tiles and every export shade a height field with the
    /// generation carve filled back in and a smooth valley cut along each
    /// drawn river (`valley_shade::valley_shade_field`, reached through
    /// `WorldGen::valley_shade_field`), so no dark stepped groove shows beside
    /// a drawn river or along a run the map does not draw. The world's own
    /// height, and everything computed from it, are unchanged either way.
    ///
    /// A `bool` for the reason `smooth_shores` is one: the valley is on the
    /// line or it is not. `true` in `default()`; `false` in
    /// [`Self::js_reference`], which shades the carve as the reference does.
    /// Read only by `WorldGen`, never by this file's own render functions --
    /// a `RenderCtx` shades whatever field it is handed.
    pub smooth_valleys: bool,

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

    // ---- The reference's own two R5 lighting fields (2026-09-03) ----
    /// `state.viz.svf` (`svfR`) — sky-view factor: how much of the sky
    /// hemisphere a cell can see, so enclosed valleys and gorge floors lose
    /// diffuse skylight that open ridgetops keep ([`build_svf`]).
    ///
    /// **A different measurement from [`Self::ao_strength`], not a second
    /// dial on the same one.** AO here is a *cavity map* — a comparison
    /// against a blurred copy of the field, which asks "does this cell sit
    /// below its neighbourhood". SVF ray-casts eight azimuths and asks "how
    /// high does the terrain rise around this cell", which is a horizon
    /// measurement: a broad shallow basin is a strong cavity and a weak
    /// enclosure, and a narrow gorge between two walls is the reverse. The
    /// reference multiplies its own three fields together for exactly that
    /// reason and this port does the same (see [`RenderCtx::with_appearance`]).
    ///
    /// `0.0` skips the whole precompute, which is the reference's own default
    /// and `js_reference()`'s.
    pub svf_strength: f64,
    /// `state.viz.shadows` (`shadowsR`) — horizon cast shadows: march each
    /// cell toward the sun and darken it where terrain rises above the sun ray
    /// ([`build_sun_shadow`]). Long soft shadows thrown off major ranges,
    /// which is the one relief cue a per-cell hillshade cannot produce however
    /// many light directions it is given — a hillshade only ever asks about
    /// the *local* normal.
    ///
    /// `0.0` skips the whole precompute; the reference's default and
    /// `js_reference()`'s.
    pub shadow_strength: f64,

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
    /// `state.viz.geology`'s **texture** half (reference 7801-7832): the
    /// per-rock-type procedural microtexture — granite mineral speckle and
    /// fracture creases, basalt lava-field patchiness, andesite ash and cinder
    /// darkening, limestone karst pitting, sandstone and shale strata banded
    /// by elevation, metamorphic folded gneiss — plus the wind-ripple banding
    /// the same slider gives gentle sandy ground ([`litho_microtexture`], and
    /// the dune branch in [`land_color`]).
    ///
    /// **Only the texture.** The reference's `geoK` block also blends toward a
    /// flat per-lithology colour; in this port that colour arrives already,
    /// from [`Self::litho_strength`] and [`Self::litho_exposure`] over
    /// [`litho_palette`]'s seven editable three-stop ramps. Porting the
    /// reference's recolour on top would be a *second* geology colour
    /// vocabulary disagreeing with the first — so this stage contributes the
    /// reference's `(1 + mt)` factor and nothing else, over whatever colour
    /// the two stages above put there. Stated rather than silent, per
    /// `CLAUDE.md`'s rule on deviating from a literal port.
    ///
    /// Inert with no lithology attached (a loaded save), which is the
    /// reference's own `lith!==undefined` gate, and `0.0` skips it entirely.
    pub geo_micro: f64,

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

    // ---- `LOD_DETAIL_SCOPE.md` LOD-D4: ice and snow from existing fields ----
    /// How much of [`TileFields`]' glacier-potential field reaches the
    /// colour — the strength of LOD-D4's third stage (*"where glacier
    /// potential is high, snow takes `snow_glac` plus a slope- and
    /// flow-aligned brightness term"*).
    ///
    /// **`0.0` in [`Self::js_reference`], which is what makes the whole
    /// milestone inert on the parity path** — the scope's own requirement
    /// (*"Three derived stages, all inert under `js_reference()`"*). The
    /// reference HTML has no ice layer at all, so there is nothing for a
    /// golden to disagree with; the gate is a dedicated branch in
    /// [`land_color`] rather than a `* 0.0`, on the same rule
    /// `relief_lights <= 1` and `litho_strength` already follow.
    ///
    /// It scales the *potential*, not the colour, so at `0.0` the ice cover
    /// is zero and `material_weights`' own Σ=1 blend is returned untouched —
    /// including the rock fraction, which is what keeps the scope's third
    /// acceptance bar (*"pixels above the snowline with slope > 0.08 are
    /// rock-dominant"*) a property of the same arithmetic in both states.
    ///
    /// # It gates stage 2 as well, and is therefore the milestone's off switch
    ///
    /// `> 0.0` also decides whether [`TileFields`]' [`TileCryo`] is applied at
    /// all. That is deliberate and it is **not** a second meaning smuggled
    /// into one number: the scope asks for *"three derived stages, all inert
    /// under `js_reference()`"*, and the sub-cell temperature is one of the
    /// three. A tile carrying a lapse correction is not the reference's tile
    /// whatever its ice looks like.
    ///
    /// The gate is a **branch**, not a factor: the lapse rate is
    /// `cartalith-climate`'s own relation and scaling it by a render slider
    /// would let the picture disagree with the simulation that produced the
    /// temperature raster. So this number is continuous for stage 3 and
    /// boolean for stage 2, which is why the GUI row is one group called
    /// *"Ice & snow"* rather than a strength beside an unrelated checkbox.
    pub ice_strength: f64,
    /// Ruling AP (2026-09-23): how far a slope's facing moves the snow term, in
    /// degrees C at full slope strength -- shaded (poleward-facing) slopes
    /// hold snow warmer, sun-facing ones lose it colder. `0.0` is off by a
    /// BRANCH in [`material_weights`], and `js_reference()` sets it, so the
    /// parity path keeps the reference's temperature-only snow. `2.0`
    /// shipped: a 200-300 m snowline difference between the two sides of a
    /// ridge at the usual ~6.5 C/km lapse rate.
    pub snow_aspect_c: f64,

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

    // ---- 2026-08-24: the four reference render stages this port had not
    //      ported, plus the three presentation controls it never had ----
    //
    // Every one of these is `0.0` (or, for `haze_strength`, the reference's
    // own literal) in `Default`, so `default()` and `js_reference()` render
    // exactly the image they rendered before — the same
    // early-return-on-its-own-gate rule every stage since milestone 2 follows.
    // What turns them on is a **named look** ([`TerrainAppearance::with_look`]),
    // not the tier ladder.
    /// `state.viz.crest` (reference `buildCrestField`/`applyCrest`, HTML
    /// 8008-8023) — thin bright strokes along convex, steep ridge lines.
    /// Costs a whole-grid precompute when non-zero and nothing at all when
    /// zero (`RenderCtx.crest` is an empty `Vec`, tested by length).
    pub crest_strength: f64,
    /// `state.viz.texture` (HTML 7845-7851) — a three-frequency fbm
    /// multiplicative modulation of the material colour, evaluated in grid
    /// coordinates so a tiled bake stays seamless.
    pub tex_strength: f64,
    /// `state.viz.ridgedRelief` (HTML 7857-7862) — folded-crease brightness
    /// modulation from a five-octave ridged multifractal, weighted by `r²` so
    /// it concentrates in the highlands and leaves the lowlands alone.
    pub ridged_strength: f64,
    /// `state.viz.curveShade` (HTML 7875-7876) — sun-independent lighting from
    /// the Laplacian: convex ridges brighten, concave valleys darken. Land
    /// only, exactly as the reference gates it.
    pub curve_shade: f64,

    // ---- 2026-09-03 (`OUTSTANDING_WORK.md` §2.5): the last two `state.viz.*`
    //      colour stages of the reference's own `landColorCore`, which
    //      `render.rs`'s exclusion list named and nothing else in this
    //      repository registered at all ----
    //
    // Same rule as the four above, for the same reason: `0.0` in both
    // `default()` and `js_reference()`, each one behind its own `if` rather
    // than an arithmetic no-op, so the shipped image and the golden-verified
    // reference image are byte-for-byte what they were. `tests/color_space.rs`'s
    // `FINISHED_RENDER_FNV1A` and `tests/layer_stack.rs`'s
    // `the_default_stack_renders_the_pre_change_image` are the guards that say
    // so with a digest rather than with an inference.
    /// `state.viz.rockSlope` (reference HTML 7788-7790, "R2 slope-material
    /// refinement") — extra `G^1.5`-weighted rock exposure on steep ground,
    /// blended **over** the finished material mix rather than into it, so the
    /// golden-verified `Σ = 1` `material_weights` output is untouched.
    ///
    /// **Its steepness normalizer is the reference's own `slope / 0.08` and
    /// therefore inherits `material_weights`' resolution dependence** — the
    /// per-cell height difference `slope_at` returns is ~6x smaller at 2048²
    /// than at 512² (measured: median land slope 0.00354 vs 0.00054, see
    /// [`Self::litho_exposure`]'s note). A cliff still crosses the ramp at
    /// either size; gentle ground gets nothing at either. What moves with
    /// resolution is how much of the middle qualifies. Left exactly as the
    /// reference writes it on purpose: this is a *ported* threshold, not a new
    /// one, and re-normalizing it would be a silent divergence from the file
    /// the row asks it to be ported from.
    pub rock_slope: f64,
    /// `state.viz.wetness` (reference HTML 7796-7798, "R5 wetness rendering")
    /// — darkens and cools land where water persistently accumulates, keyed on
    /// the **raw** TWI the caller already computes (`ln(area / max(slope,
    /// 0.002))`), not on `land_color`'s jittered `twi_e`; the reference reads
    /// its own `twi` parameter there and the distinction is a real one, since
    /// `twi_e` exists to ragged the *wetland material* boundary.
    ///
    /// **Not the same stage as [`Self::hydro_wet_strength`]**, despite that
    /// field's row in [`tunables!`] being filed under the reference's
    /// `wetnessR` slider. That one is this port's own milestone-3 invention: a
    /// blurred *flow-accumulation* halo, applied after the hillshade and haze,
    /// pulling toward a fixed cool grey-green. This one is the reference's:
    /// unblurred TWI, applied in **material space before the light curve**
    /// ("so lighting stays honest", the reference's own comment), and a
    /// multiply rather than a lerp. They can both be on; they are different
    /// pictures of "wet".
    pub wetness: f64,
    /// `state.viz.sdfCoast` (`sdfCoastR`) — the reference's B2 coast bands:
    /// a bright wet-sand shore band and a lusher coastal plain behind it,
    /// keyed on **signed distance to the coastline** rather than on elevation,
    /// so both read at a constant width whatever the relief does
    /// ([`apply_coast_sdf`]).
    ///
    /// `0.0` skips the whole thing, including the distance transform
    /// ([`RenderCtx::with_appearance`] does not build the field at all) — the
    /// reference's own default and `js_reference()`'s.
    ///
    /// Its two siblings are [`Self::sdf_rivers`] and [`Self::sdf_biomes`],
    /// which landed 2026-09-03 (third pass). **This doc comment used to say
    /// they were unportable** — "a missing builder rather than a decision",
    /// naming `cartalith-civ`'s private `jfa_dist` — and that reason was
    /// wrong, not merely stale: neither leg ever needed it. See the module
    /// doc's third-pass paragraph.
    pub sdf_coast: f64,
    /// `state.viz.sdfRivers` (`sdfRiversR`) — the reference's B3 river bands:
    /// damp bank, wetland green and floodplain, in three widening rings
    /// measured from the **discharge channel mask** rather than from the
    /// stamped river raster, so they read at a constant width at any
    /// resolution ([`apply_river_sdf`]).
    ///
    /// `0.0` skips the whole thing, including the distance transform — and so
    /// does a `RenderCtx` whose [`RenderCtx::with_map_scale`] was never
    /// called, or one built from a **loaded save**, which carries no flow
    /// field (`SAVEFILE_COMPAT.md`, the same absence `flow` already documents).
    /// The consumer tests the field's length rather than this number, so the
    /// three cannot disagree.
    pub sdf_rivers: f64,
    /// `state.viz.sdfBiomes` (`sdfBiomesR`) — the reference's B4 ecotone
    /// widener. Not a tint: it scales the **noise jitter** [`land_color`]
    /// already applies to its climate inputs, in proportion to how close the
    /// cell is to a biome boundary ([`sdf_eco_k`]), so material edges ragged
    /// where two biomes meet and stay crisp in a biome's interior.
    ///
    /// `1.0` is the multiplier at rest, not `0.0` — [`sdf_eco_k`] returns
    /// `1 + k·1.5·smoothstep(…)`, the reference's own shape — which is why
    /// this one is gated on the *field* being empty rather than on a
    /// zero-valued product: at `0.0` the field is never built, so `land_color`
    /// is called with the literal `1.0` the jitter has always used.
    pub sdf_biomes: f64,

    // ---- 2026-09-03 (`OUTSTANDING_WORK.md` §2.5): the ocean lattice ----
    /// How far the sea-grain sample lattice is rotated and domain-warped away
    /// from the reference's axis-aligned one — the fix for the ocean lattice
    /// artefact, **on in the shipped look** (`1.0` in `default()`, Ruling AS,
    /// 2026-09-24). `0.0` is the reference's `vnoise(x·25.6/GW, y·25.6/GW, 5)`
    /// **exactly**, by a dedicated branch in [`sea_grain`], and is what
    /// `js_reference()` pins so the golden-parity path keeps rendering the
    /// reference's own sea.
    ///
    /// `TERRAIN_APPEARANCE_SCOPE.md` milestone 6 recorded rectangular
    /// blockiness in the open ocean — squares ~80 grid cells across at 2048² —
    /// coming from the reference's own axis-aligned sample lattice. Under
    /// `DECISIONS.md` §7p a rendering improvement is correct on its own terms,
    /// so the shipped look fixes it; parity stays verified on `js_reference()`.
    /// `n_low` is not decoration: it is the `t` of every water `ramp3` in
    /// [`sea_color_core`] plus its `(n_low - 0.5)·5` grain, so a different
    /// sample is a different colour at every sea pixel.
    ///
    /// The fix is milestone 4's own, reused rather than invented: the stipple
    /// read as a halftone screen for exactly this reason (value noise on an
    /// axis-aligned lattice at a few cells per feature), and was fixed by
    /// rotating the lattice and domain-warping it with a second coherent
    /// field. Same treatment, same rotation, one frequency. `1.0` is the
    /// strength [`sea_grain`] was written to reach — half a lattice cell of
    /// warp, its "fully broken-up" end of the slider.
    pub sea_grain_warp: f64,
    /// **Ruling BI** (`LARGE_ITEM_RULINGS.md`, 2026-09-27), the "Nautical"
    /// preset's own small renderer addition, flagged by
    /// `MAP_STYLE_RESEARCH.md` §3/§4 as the one genuinely new table among the
    /// eight researched presets: a depth-banded bathymetric tint blended over
    /// [`sea_color_core`]'s existing material-based shelf/deep/abyss mix, the
    /// same way [`TerrainAppearance::ramp_strength`] blends a land ramp over
    /// `material_weights` — same mechanism, water side.
    ///
    /// `0.0` (default, and [`TerrainAppearance::js_reference`]'s own value) is
    /// untouched: `sea_color_core` takes exactly the path it always did, so
    /// neither the shipped default nor a single JS golden moves. Above zero,
    /// [`sea_color_core`] mixes its computed colour toward
    /// [`SEA_RAMP_NAUTICAL`] sampled at the same `depth` it already carries,
    /// by this fraction.
    pub sea_ramp_strength: f64,
    /// **The river symbol's style** (owner, 2026-09-27: *"the only issue I
    /// have with the rivers: they're drawn on top of the style"*). Since then
    /// a generated world's rivers take the style: the deep-zoom tiles
    /// rasterize RV-2's strokes into themselves ([`RiverLayer`],
    /// `river_stroke::rasterize`), and the base view draws the vector stroke
    /// textured with the map composited at full river coverage
    /// (`WorldGen::river_color_texture`) -- both through [`land_color`]. So
    /// every one of the five fields below is a per-preset treatment of that symbol, and
    /// all five are inert on a render with no river layer attached — every
    /// JS golden, every export and `js_reference()` among them.
    ///
    /// Width multiplier on RV-2's per-point width, before the 1 px floor.
    /// `1.0` is RV-2's own width.
    pub river_width: f64,
    /// The river symbol's alpha multiplier. `1.0` is RV-2's opaque stroke.
    pub river_opacity: f64,
    /// How far the per-Strahler-order palette (`lib.rs`'s `RIVER_ORDER_RGB`,
    /// light headwater to dark trunk) moves toward [`Self::river_ink_r`]/`g`/`b`
    /// — `0.0` is RV-2's palette untouched, `1.0` draws every river in the
    /// one ink (a Blueprint's white line, a woodcut's black one).
    pub river_ink: f64,
    /// The ink [`Self::river_ink`] pulls toward, `0..=255` per channel.
    /// Defaults to the palette's own darkest (trunk) stop, `(22, 36, 80)`.
    pub river_ink_r: f64,
    pub river_ink_g: f64,
    pub river_ink_b: f64,
    /// **Where the river sits relative to the Painter styles**
    /// ([`apply_npr`]). `1.0`: under them — the river is composited into the
    /// lit colour before the Painter block, so sepia tones it, crosshatch and
    /// stipple engrave it, the village quantiser flattens it, exactly as the
    /// reference's paint-brush tint sits "before the Painter/NPR block so
    /// hand-drawn styles still apply consistently on top". `0.0`: over them —
    /// composited after, so a Blueprint's white line is not hatched. The sheet
    /// (paper, frame), local contrast and the colour grade act on the river
    /// at every value: they run after this stage whichever way it is set.
    pub river_through: f64,
    /// Chroma of the **material** colour, as a delta about the mix
    /// `material_weights` produced: `+0.20` is 20% more chroma at the same
    /// luminance, `-1.0` is greyscale. No reference counterpart — the
    /// reference's only chroma control is `bio_blend`, which pulls toward a
    /// *fixed* grey and therefore changes the value structure as well as the
    /// chroma. Applied about the pixel's own Rec.709 luma, so it can never
    /// move a material lighter or darker relative to its neighbour, only more
    /// or less colourful.
    pub biome_sat: f64,
    /// How far the relief lighting is **luminance-preserving** rather than the
    /// reference's `blend`-toward-`185·light` grey.
    ///
    /// `0.0` is the reference exactly. At `1.0` the `bio_blend` desaturation
    /// targets a grey of *the pixel's own* luminance instead of a fixed one,
    /// and the light factor additionally cools and slightly desaturates
    /// shadow while warming and slightly saturating sun — the way a real
    /// scene's shadow (lit by sky) and sunlight (lit by a warm source) differ.
    /// The reference's grey lerp does neither: it drags every shaded pixel
    /// toward the same neutral, which is what makes a `bio_blend` under 1
    /// read as "the map faded" rather than "the light changed".
    pub relief_chroma: f64,
    /// **Cel / toon shading** (owner, 2026-09-27: *"cel shading for a bit of a
    /// more stylized look 'cartoonish'"*; `OUTSTANDING_WORK.md`'s "Cel / Toon"
    /// row). No reference counterpart.
    ///
    /// What: at `1.0` the hillshade term is cut into [`TOON_BANDS`] flat light
    /// steps with a hard terminator ([`toon_band`]), the six-material blend is
    /// sharpened toward its dominant material ([`toon_sharpen_weights`]), and
    /// the material colour loses its within-material noise (the `ramp3`
    /// micro-ramp position and the fine grain) — so each biome reads as one
    /// flat colour per light step, with a crisp edge to the next biome.
    /// Between `0` and `1` both blend linearly from the smooth light and the
    /// textured colour.
    ///
    /// Why a new stage and not the Painter block's `D-cel`
    /// ([`Npr::cel`]): `D-cel` posterises the **finished colour**, texture
    /// noise included, which the 2026-09-27 preview measured as mottled
    /// pink/tan/olive blotches rather than a toon look — and it is a literal
    /// port pinned by `golden_parity_npr.rs`, so it cannot be reworked. Toon
    /// shading bands the **light**, not the colour.
    ///
    /// Never: this does not touch water, the colour grade or any Painter style,
    /// and at `0.0` (`default()`, `js_reference()`, every tier and look) its
    /// branches are never entered, so those images are bit-identical to the
    /// tree before it existed (`tests/cel_toon.rs` pins the digests).
    pub toon_strength: f64,
    /// Opacity of the **toon outline**: a dark keyline on the land side of
    /// every coast and lake shore, [`TOON_OUTLINE_R`] cells (screen/export) or
    /// tile pixels (deep zoom) wide — see [`toon_outline_cover`]. `0.0` is off
    /// and costs nothing: the neighbourhood is never read.
    ///
    /// Why its own key rather than part of [`Self::toon_strength`]: the outline
    /// is a separate stage in a separate place (after `land_color`, where the
    /// neighbouring cells can be asked whether they are water), and a keyline
    /// over a smoothly lit map is a legitimate look of its own.
    pub toon_outline: f64,
    /// Strength of the edge-of-plate atmospheric haze (HTML 7880-7882). The
    /// reference's own literal `0.18`, hoisted out of `land_color` so a look
    /// can dial it back; the haze *colour* (208, 218, 230) stays the
    /// reference's, since it is the sky it fades toward rather than a taste.
    pub haze_strength: f64,

    // ---- §19 (`TERRAIN_APPEARANCE_RESEARCH.md`): atmospheric / distance
    //      effects (`OUTSTANDING_WORK.md` §2.5) ----
    //
    // §19, in full: "far terrain: slightly lower contrast, slightly reduced
    // saturation, increased atmospheric tint / near terrain: retain full
    // material contrast... this should be optional for 2D cartographic maps.
    // Do not impose a 3D-game aesthetic on the default Cartalith map." The
    // third clause is `haze_strength` above, already reading a radial
    // distance-from-plate-centre factor. These two read the identical
    // factor -- there is no camera, so "far" is this plate-relative
    // distance, exactly as `haze_strength` already treats it -- and supply
    // the other two clauses research §19 named. Both `0.0` at rest: an
    // *added* stage, not a retuned one, so the shipped look and every
    // golden are unmoved until a caller asks for either.
    /// Saturation lost at the plate edge, as a fraction of full desaturation
    /// (`land_color`'s own [`saturate`] with `k = 1 - atmo_desaturation *
    /// distance`). Near the centre `distance` is ~0 and this is a no-op;
    /// only the outer plate loses chroma.
    pub atmo_desaturation: f64,
    /// Contrast lost at the plate edge, about the same 128 mid-grey pivot
    /// [`grade_contrast`](Self::grade_contrast) uses, scaled by the same
    /// distance factor. Only ever *reduces* contrast (§19 has no "far
    /// terrain gains contrast" case), so this cannot invert or boost.
    pub atmo_contrast: f64,

    // ---- The colour-grade stage (2026-08-24) ----
    //
    // A final, restrained grade over the **finished raster**, in
    // [`apply_color_grade`] — the only other whole-image pass in this file
    // besides [`apply_local_contrast`], and for the same reason: a grade is a
    // statement about the picture, not about a pixel's material.
    //
    // **Presentation only, and structurally so**: it runs on the output
    // buffer after every field has already been consumed, so there is no path
    // by which it could reach the heightmap, climate, geology or hydrology.
    // Every parameter is `0.0` at rest and [`ColorGradeExt::grade_is_identity`]
    // early-returns on that, so the pass costs one test on the default path.
    /// Exposure, as a linear gain of `1 + exposure` (`-1` black, `+1` double).
    pub grade_exposure: f64,
    /// Contrast about mid-grey (128). `+1` roughly doubles the slope.
    pub grade_contrast: f64,
    /// Saturation delta about Rec.709 luma, like [`Self::biome_sat`] but over
    /// the whole finished image rather than the material mix alone.
    pub grade_saturation: f64,
    /// Colour temperature on a blue↔amber axis: `-1` fully cool, `+1` fully
    /// warm. Luminance-compensated (the green channel takes a small share), so
    /// warming a map does not also brighten it.
    pub grade_temperature: f64,
    /// Tint of the **shadows** on the same blue↔amber axis, weighted by
    /// `1 - luma`, so it lands in the dark half of the image only.
    pub grade_shadow_tint: f64,
    /// Tint of the **highlights**, weighted by `luma`.
    pub grade_highlight_tint: f64,
    /// Gamma, as a symmetric power curve about the `[0,1]` channel range:
    /// the exponent is `2^-gamma`, so `+1` lifts the midtones (exponent
    /// `0.5`), `-1` sinks them (exponent `2`) and `0` is the exact identity —
    /// no `powf` runs at all at rest, which is what keeps the default path
    /// bit-identical to the six-axis grade that shipped first.
    pub grade_gamma: f64,

    // ---- The four field-influence weights (2026-08-24) ----
    //
    // `design/Cartalith Menu Structure v2.dc.html`, MAP ▸ TERRAIN APPEARANCE ▸
    // COLOUR ▸ "+ Field influence weights: Biome · elevation · moisture ·
    // geology", and `TERRAIN_APPEARANCE_RESEARCH.md` §17, which lists the same
    // four at the bottom of its COLOUR VIBRANCY SYSTEM control list. They are
    // weights *on the grade*, not axes of their own: each one lets the grade's
    // strength track one underlying field instead of landing flat across the
    // sheet. See [`build_grade_influence`] for the per-cell signals and for
    // which part of the mapping is this port's own choice.
    /// How strongly the grade follows **biome vegetation cover**
    /// ([`BIOME_VEGETATION_COVER`]). `0` is flat, `+1` grades bare ice and
    /// desert least and closed forest most, `-1` reverses that.
    pub grade_field_biome: f64,
    /// How strongly the grade follows **relative land elevation** (`0` at and
    /// below sea level, `1` at the top of the land range).
    pub grade_field_elevation: f64,
    /// How strongly the grade follows **moisture** (the rainfall field, `0..1`).
    pub grade_field_moisture: f64,
    /// How strongly the grade follows **geology** — the lightness of the
    /// cell's own rock in the current lithology palette, so a dark basalt and a
    /// pale limestone sit at opposite ends of the same axis the map already
    /// draws. Inert on a loaded save, which carries no lithology.
    pub grade_field_geology: f64,

    /// The reference's non-photorealistic block ([`Npr`]) — all off by
    /// default, so this field changes nothing until a caller sets it.
    pub npr: Npr,

    /// How the raster's three separable categories stack, blend and order
    /// ([`LayerStack`], `GUI_GAP_REGISTER.md` CA-03/CA-04, RD-10).
    ///
    /// [`LayerStack::DEFAULT`] renders the shipped image through the original
    /// expressions, by branch — see the section above [`RasterLayer`]. Lives
    /// on the appearance rather than on `WorldGen` so that it reaches the
    /// on-screen texture and every export through the one `RenderCtx` both
    /// already build, and so a saved look (CA-08) carries its own stack.
    pub layers: LayerStack,
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
            biome_cols: CART_BIOME_COLS,
            exag: 3.4,
            sun_az_deg: 315.0,
            sun_alt_deg: 40.0,
            bio_blend: 0.90,
            relief_lights: 6,
            relief_directionality: 0.62,
            relief_ambient: 0.34,
            relief_gain: 1.16,
            // §16: the exact literals `sh_combined` always used, now named
            // rather than hardcoded -- `default()` and `js_reference()` both
            // render the identical image this blend always produced.
            detail_macro_weight: 0.40,
            detail_meso_weight: 0.40,
            detail_micro_weight: 0.20,
            // LOD-D5. Full strength, for `ice_strength`'s reason a second
            // time: the quantity it scales is itself exactly zero at grid
            // resolution, so on the main map and on a tile drawn at one pixel
            // per cell this changes no pixel whatever this number is. The
            // `0.0` that matters is `js_reference`'s.
            detail_scale_strength: 1.0,
            // v2.25 `tileShadeExag` (RC_ENGINE_CHANGES.md §4, Ruling AP): on
            // in the shipped look. `js_reference()` pins `false`.
            tile_shade_exag_scaled: true,
            // RV-4: the smooth shoreline, on in the shipped look.
            // `js_reference()` pins `false`.
            smooth_shores: true,
            // RV-3: the valley shaded along the drawn line, on in the shipped
            // look. `js_reference()` pins `false`.
            smooth_valleys: true,
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
            // LOD-D4. Full strength, because the field it scales is itself
            // `0.0` everywhere the glacial gate does not hold: on a world
            // whose ground is below the snowline or above freezing this
            // changes no pixel whatever this number is, and on a glaciated
            // one the stage is the milestone. The `0.0` that matters is
            // `js_reference`'s.
            ice_strength: 1.0,
            // Ruling AP: snow follows slope facing on the shipped map.
            snow_aspect_c: 2.0,
            // CA-02: off, so the shipped look is unchanged; the ramp behind it
            // is real so the slider has something to reveal.
            ramp_strength: 0.0,
            ramp: ElevationRamp::default(),
            splat_strength: 0.7,
            // The four unported reference stages and the three new controls:
            // every one at the value that makes it a no-op, so `default()` is
            // the image milestones 1-7 tuned and `js_reference()` is still the
            // reference. `NATURAL_VIBRANT` is what turns them on.
            crest_strength: 0.0,
            tex_strength: 0.0,
            ridged_strength: 0.0,
            curve_shade: 0.0,
            // The two remaining reference stages (2026-09-03), at the value
            // that makes them a no-op — `tests/color_space.rs`'s
            // `FINISHED_RENDER_FNV1A` holds the digest that proves it.
            rock_slope: 0.0,
            wetness: 0.0,
            // The last three reference viz stages this file had no port for
            // (2026-09-03, second pass) — the two R5 lighting fields and the
            // B2 coast bands. Same rule, third time: `0.0` is the value at
            // which each one's own branch is never entered, so `default()`,
            // `js_reference()` and every tier below are the images they
            // already were. `color_space.rs`'s `FINISHED_RENDER_FNV1A` is the
            // hash that proves it for `default()`.
            svf_strength: 0.0,
            shadow_strength: 0.0,
            geo_micro: 0.0,
            sdf_coast: 0.0,
            // The trio's other two legs (2026-09-03, third pass). Same rule
            // again: at `0.0` neither field is built, so `land_color`'s
            // `eco_k` is the literal `1.0` it has always been and the river
            // band's own call site is a length test that never fires.
            sdf_rivers: 0.0,
            sdf_biomes: 0.0,
            // The ocean lattice fix, on (Ruling AS, 2026-09-24): `1.0` is the
            // full strength `sea_grain` was written for. `js_reference()` pins
            // `0.0`, the reference's own lattice.
            sea_grain_warp: 1.0,
            // Ruling BI: off by default, exactly as `js_reference()` pins it
            // below -- see the field's own doc comment.
            sea_ramp_strength: 0.0,
            // The river symbol at RV-2's own look: its width, opaque, its
            // palette untouched, under the Painter styles.
            river_width: 1.0,
            river_opacity: 1.0,
            river_ink: 0.0,
            river_ink_r: 22.0,
            river_ink_g: 36.0,
            river_ink_b: 80.0,
            river_through: 1.0,
            biome_sat: 0.0,
            relief_chroma: 0.0,
            // Cel / toon: off, by branch -- see the fields.
            toon_strength: 0.0,
            toon_outline: 0.0,
            haze_strength: 0.18,
            // §19: an added stage, at rest -- see the field doc comments.
            atmo_desaturation: 0.0,
            atmo_contrast: 0.0,
            grade_exposure: 0.0,
            grade_contrast: 0.0,
            grade_saturation: 0.0,
            grade_temperature: 0.0,
            grade_shadow_tint: 0.0,
            grade_highlight_tint: 0.0,
            grade_gamma: 0.0,
            grade_field_biome: 0.0,
            grade_field_elevation: 0.0,
            grade_field_moisture: 0.0,
            grade_field_geology: 0.0,
            npr: Npr::default(),
            layers: LayerStack::DEFAULT,
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
    /// high-precision display pipeline. Only its first, tier-independent step
    /// is built (Ruling AN: one quantisation after the finishing stages, see
    /// `finish_rgb`); there is still no HDR or wide-gamut *output*, so
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
            // LOD-D4, same rule a third time: the reference has no ice layer,
            // so the parity path must not have one either. `land_color`'s ice
            // block is inside an `if`, so this is off by control flow and not
            // by arithmetic.
            ice_strength: 0.0,
            // Ruling AP's snow aspect term: the reference's snow reads
            // temperature alone, so the parity path has none (a branch).
            snow_aspect_c: 0.0,
            // LOD-D5, same rule a fourth time: the reference's tile shades at
            // one fixed balance of bands, draws its crest over a one-pixel
            // stencil and hands its river SDF the grid's own threshold at
            // every level. `0.0` is that behaviour exactly, and each of the
            // four stages is a product with this number rather than a
            // branch — see the field's own doc comment for why the identity
            // is safe here where `ice_strength` needed a branch.
            detail_scale_strength: 0.0,
            // Ruling AS: `default()` turns the ocean lattice fix on, so the
            // parity path pins the reference's own axis-aligned sea-grain
            // lattice here rather than inheriting it. `sea_grain` returns the
            // reference expression from a dedicated branch at `0.0`.
            sea_grain_warp: 0.0,
            // Ruling BI: the bathymetric ramp is a shipped-look addition, not
            // a reference row -- `0.0` here too, so `sea_color_core` takes
            // the reference path exactly.
            sea_ramp_strength: 0.0,
            // The reference draws no river symbol into its map at all
            // (`drawRiverWays` is a separate canvas pass), and nothing here
            // attaches a `RiverLayer`, so these five are unread on the parity
            // path. Pinned at `default()`'s values so a caller that attaches a
            // layer to a reference-appearance render still gets RV-2's look.
            river_width: 1.0,
            river_opacity: 1.0,
            river_ink: 0.0,
            river_ink_r: 22.0,
            river_ink_g: 36.0,
            river_ink_b: 80.0,
            river_through: 1.0,
            // Cel / toon shading is a port-only style with no reference row:
            // pinned off here explicitly rather than inherited, so a future
            // change to `default()` cannot reach the parity path through it.
            toon_strength: 0.0,
            toon_outline: 0.0,
            // v2.25's `tileShadeExag` (RC_ENGINE_CHANGES.md §4) is on in
            // `default()`; the frozen v2.11 reference shades a tile with the
            // bare `ex = state.exag` (11670), and the golden pins that.
            tile_shade_exag_scaled: false,
            // RV-4's smooth shoreline is a port-only drawing: the reference
            // cuts its tile lake band by bilinear membership (`fq > 0.35`,
            // 11717-11740) and its map draws whole cells, and the tile golden
            // pins that. Off by control flow (`is_lake_pixel`'s branch).
            smooth_shores: false,
            // RV-3's valley along the drawn line is port-only too: the
            // reference shades its carved field as it stands.
            smooth_valleys: false,
            ..TerrainAppearance::default()
        }
    }

    /// One of [`LOOK_PRESETS`], layered **over** whatever tier this value came
    /// from. An unrecognised name (including [`LOOK_TIER`]) returns `self`
    /// unchanged, so a look saved by a newer build degrades to the tier's own
    /// image rather than to something nobody chose.
    ///
    /// # Why a layer rather than a replacement
    ///
    /// `for_tier` decides **what the renderer spends** (which per-pixel noise
    /// passes and whole-raster passes run at all); a look decides **what the
    /// picture is**. Those are different questions and a phone answers the
    /// first one differently from a workstation, so a look that replaced the
    /// tier would silently hand a phone the workstation's cost. Every entry
    /// here is therefore written as a struct-update over `self`, and touches
    /// only colour, chroma, light shaping and grade — never a radius, a light
    /// count, or a stage a cheap tier switched off.
    #[allow(dead_code)]
    pub fn with_look(self, name: &str) -> Self {
        match name {
            LOOK_VIBRANT => TerrainAppearance {
                // --- 1. The four base ramps, re-pitched.
                //
                // The reference's own values (which this port transcribed
                // exactly, and which `js_reference()` still renders) are
                // low-chroma and close together in hue: temperate grass and
                // desert sand differ by about as much as two shades of the
                // same khaki. These push each ramp toward its **own hue
                // family** — grass to a true yellow-green, tropical forest to
                // a deep saturated green, desert to ochre-gold, red desert to
                // a genuine iron red — while keeping every ramp's own
                // low→high value progression, which is what the `ramp3`
                // micro-ramp reads as surface variety.
                grass_temp: [(104.0, 132.0, 58.0), (128.0, 158.0, 68.0), (156.0, 181.0, 82.0)],
                wood_trop: [(20.0, 78.0, 35.0), (25.0, 108.0, 43.0), (35.0, 138.0, 55.0)],
                sand_desert: [(202.0, 154.0, 72.0), (222.0, 172.0, 78.0), (238.0, 194.0, 105.0)],
                sand_red: [(174.0, 83.0, 47.0), (202.0, 101.0, 57.0), (224.0, 123.0, 69.0)],
                // --- 2. Lighting keeps its chroma instead of fading to grey.
                relief_chroma: 1.0,
                // --- 3./4. The reference's own zeroed enhancement sliders, at
                // the levels the owner specified. `ao_strength` moves *down*
                // from the tier's 0.28: the crest, curvature and ridged stages
                // now carry local relief, so the broad cavity darkening no
                // longer has to do it alone.
                ao_strength: 0.20,
                crest_strength: 0.12,
                tex_strength: 0.18,
                ridged_strength: 0.10,
                curve_shade: 0.28,
                hydro_wet_strength: 0.12,
                biome_sat: 0.20,
                haze_strength: 0.09,
                ..self
            },
            LOOK_ANTIQUE => TerrainAppearance {
                // The hand-illustrated parchment plate: warm earth over an
                // aged sheet, the register's own MapEffects reference. Not a
                // second "Natural Vibrant" with a filter on it — the palettes
                // move toward ochre/umber rather than toward chroma, the
                // sheet does more of the work, and the grade is where the
                // warmth lives rather than in the material colours, so relief
                // and biome separation survive it.
                paper_strength: 1.0,
                paper_tint: (236.0, 214.0, 178.0),
                paper_grain: 0.065,
                paper_mottle: 0.075,
                paper_wash: 0.30,
                border_ink: (66.0, 50.0, 36.0),
                grass_temp: [(126.0, 134.0, 78.0), (146.0, 154.0, 92.0), (168.0, 174.0, 112.0)],
                wood_trop: [(44.0, 74.0, 44.0), (58.0, 94.0, 55.0), (74.0, 114.0, 68.0)],
                sand_desert: [(198.0, 162.0, 104.0), (216.0, 182.0, 124.0), (230.0, 202.0, 150.0)],
                sand_red: [(166.0, 104.0, 68.0), (188.0, 124.0, 84.0), (206.0, 146.0, 104.0)],
                relief_chroma: 0.65,
                ao_strength: 0.24,
                crest_strength: 0.08,
                tex_strength: 0.22,
                curve_shade: 0.18,
                haze_strength: 0.06,
                biome_sat: -0.08,
                grade_temperature: 0.26,
                grade_saturation: -0.10,
                grade_contrast: 0.08,
                grade_shadow_tint: 0.18,
                ..self
            },
            _ => self,
        }
    }
}

/// The identity look: whatever [`TerrainAppearance::for_tier`] produced, with
/// nothing layered on it. This is the image milestones 1-7 tuned, and what
/// "Reset to quality tier" restores.
pub const LOOK_TIER: &str = "Quality tier";
/// The shipped default look (2026-08-24) — see [`TerrainAppearance::with_look`].
pub const LOOK_VIBRANT: &str = "Natural Vibrant";
/// The hand-illustrated parchment plate.
pub const LOOK_ANTIQUE: &str = "Antique Parchment";

/// Every named look, in the order a picker should show them. The **first entry
/// is not the default** — `WorldGen` opens on [`LOOK_VIBRANT`]; this list is
/// ordered plainest first so the identity sits at the top where a reader looks
/// for "off".
// The golden-parity and tier targets `#[path]`-include this file standalone.
#[allow(dead_code)]
pub const LOOK_PRESETS: &[&str] = &[LOOK_TIER, LOOK_VIBRANT, LOOK_ANTIQUE];

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
    // -- §16: the multi-scale detail blend's three band weights (no
    //    reference counterpart -- the reference's own hillshade is one band) --
    "detail_macro_weight"   => detail_macro_weight,   0.0,   1.0,  "Macro detail";
    "detail_meso_weight"    => detail_meso_weight,    0.0,   1.0,  "Meso detail";
    "detail_micro_weight"   => detail_micro_weight,   0.0,   1.0,  "Micro detail";
    // -- LOD-D5: the same three bands, re-balanced by how far past the
    //    simulation grid the view has zoomed. A tile stage only --
    "detail_scale_strength" => detail_scale_strength, 0.0,   1.0,  "Scale-aware detail";
    // -- Reference Rendering-advanced ▸ Ambient occlusion (`aoR`) --
    "ao_strength"           => ao_strength,           0.0,   1.0,  "Ambient occlusion";
    "ao_radius_frac"        => ao_radius_frac,        0.0,   0.05, "AO radius";
    // -- Reference Rendering-advanced ▸ the two R5 lighting fields (`svfR`,
    //    `shadowsR`). They sit beside AO because the reference multiplies all
    //    three into one `aoC`, and so does this port --
    "svf_strength"          => svf_strength,          0.0,   1.0,  "Sky view factor";
    "shadow_strength"       => shadow_strength,       0.0,   1.0,  "Cast shadows";
    // -- Milestone 3's flow-accumulation wetness. **This row used to be filed
    //    under the reference's `wetnessR` slider and that was wrong**: the
    //    reference's own wetness stage is TWI-keyed, unblurred and applied in
    //    material space, and it did not exist in this file until 2026-09-03,
    //    when it landed below as `wetness`. This one is a `TERRAIN_APPEARANCE_
    //    RESEARCH.md` §13 invention with no reference counterpart. The labels
    //    say which is which rather than both reading "Wetness" --
    "hydro_wet_strength"    => hydro_wet_strength,    0.0,   1.0,  "Wetness (near channels)";
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
    // The same `geologyR` slider's texture half. One row, not two, because the
    // reference drives both the rock microtexture and the dune ripples from
    // this one number -- and because a separate dune row would be inert on
    // every fixture without a hot arid coast to act on, which is a row that
    // looks live and is not.
    "geo_micro"             => geo_micro,             0.0,   1.0,  "Rock microtexture";
    "local_contrast"        => local_contrast,        0.0,   1.0,  "Local contrast";
    // -- `LOD_DETAIL_SCOPE.md` LOD-D4's "Ice & snow" group. One row, not
    //    three: the lapse term and the glacier gate are not tuning, they are
    //    `cartalith-climate`'s own lapse relation and `glacial_kernel`'s own
    //    gate, and a slider over either would let the render disagree with
    //    the simulation that produced the field. This row scales how much of
    //    the resulting potential reaches the colour, and nothing else.
    //
    //    The LABEL is "Glacier ice" and the GROUP the shell draws it in is
    //    "Ice & snow" (`render_workspace.gd`'s `APPEARANCE_GROUPS`). The scope
    //    names the group, not the row, and a one-row group whose header and
    //    whose slider carry the same words reads as a bug --
    "ice_strength"          => ice_strength,          0.0,   1.0,  "Glacier ice";
    // -- `GUI_GAP_REGISTER.md` CA-02's colour relief (no reference counterpart:
    //    the reference has no elevation ramp either) --
    "ramp_strength"         => ramp_strength,         0.0,   1.0,  "Colour relief";
    // -- Reference Paint brush ▸ Texture strength (`splat`) --
    "splat_strength"        => splat_strength,        0.0,   1.0,  "Texture strength";
    // -- Reference Rendering-advanced ▸ the four rows this port had no stage
    //    for until 2026-08-24 (`crestR`, `texR`, `ridgeR`, `curveShadeR`) --
    "crest_strength"        => crest_strength,        0.0,   1.0,  "Ridge crests";
    "tex_strength"          => tex_strength,          0.0,   1.0,  "Surface texture";
    "ridged_strength"       => ridged_strength,       0.0,   1.0,  "Ridged relief";
    "curve_shade"           => curve_shade,           0.0,   1.0,  "Curvature shading";
    // -- Reference Rendering-advanced ▸ the last two rows with no stage here
    //    until 2026-09-03 (`rockR`, `wetnessR`). `wetnessR` is the slider
    //    `hydro_wet_strength` above is *also* filed under; the two are
    //    different stages and the labels say so rather than colliding --
    "rock_slope"            => rock_slope,            0.0,   1.0,  "Slope rock";
    "wetness"               => wetness,               0.0,   1.0,  "Wet ground (TWI)";
    // -- Reference Rendering-advanced ▸ the SDF trio (`sdfCoastR`,
    //    `sdfRiversR`, `sdfBiomesR`), in the reference's own panel order.
    //    The last two had no row here until 2026-09-03 --
    "sdf_coast"             => sdf_coast,             0.0,   1.0,  "Coast bands (SDF)";
    "sdf_rivers"            => sdf_rivers,            0.0,   1.0,  "River bands (SDF)";
    "sdf_biomes"            => sdf_biomes,            0.0,   1.0,  "Biome blend (SDF)";
    // -- Not a reference row: the fix for the reference's ocean lattice, on
    //    at `1.0` in the shipped look; `0.0` is the reference exactly. See
    //    the field's doc comment --
    "sea_grain_warp"        => sea_grain_warp,        0.0,   1.0,  "Ocean grain warp";
    // Ruling BI's "Nautical" preset -- see the field's own doc comment.
    "sea_ramp_strength"     => sea_ramp_strength,     0.0,   1.0,  "Bathymetric sea ramp";
    // -- The river symbol (owner, 2026-09-27: rivers must go through the
    //    style, not sit on top of it). Inert without a river layer, which
    //    only the base view's colour field and the LOD tiles build -- see the
    //    fields. Ranges are labelled judgements: width 0.25x..3x of RV-2's
    //    (thinner vanishes under the 1 px floor anyway, wider swamps a
    //    valley), ink channels the byte range, the rest fractions --
    "river_width"           => river_width,           0.25,  3.0,  "River width";
    "river_opacity"         => river_opacity,         0.0,   1.0,  "River opacity";
    "river_ink"             => river_ink,             0.0,   1.0,  "River ink";
    "river_ink_r"           => river_ink_r,           0.0, 255.0,  "River ink red";
    "river_ink_g"           => river_ink_g,           0.0, 255.0,  "River ink green";
    "river_ink_b"           => river_ink_b,           0.0, 255.0,  "River ink blue";
    "river_through"         => river_through,         0.0,   1.0,  "River under styles";
    // -- Chroma and atmosphere (no reference counterpart for the first two;
    //    the third is the reference's own literal, made adjustable) --
    "biome_sat"             => biome_sat,            -1.0,   1.0,  "Biome saturation";
    "relief_chroma"         => relief_chroma,         0.0,   1.0,  "Chroma-preserving light";
    // -- Cel / toon (owner, 2026-09-27; no reference counterpart). Both are
    //    0..1 fractions: `toon_strength` blends smooth light to banded light,
    //    `toon_outline` is the keyline's opacity --
    "toon_strength"         => toon_strength,         0.0,   1.0,  "Toon light bands";
    "toon_outline"          => toon_outline,          0.0,   1.0,  "Toon outline";
    "haze_strength"         => haze_strength,         0.0,   0.6,  "Atmospheric haze";
    // -- §19: the other two atmospheric-perspective axes research asked for,
    //    over the same plate-edge distance factor as the haze above --
    "atmo_desaturation"     => atmo_desaturation,     0.0,   1.0,  "Distance desaturation";
    "atmo_contrast"         => atmo_contrast,         0.0,   1.0,  "Distance contrast loss";
    // -- The colour grade (presentation-only post-process) --
    "grade_exposure"        => grade_exposure,       -1.0,   1.0,  "Exposure";
    "grade_contrast"        => grade_contrast,       -1.0,   1.0,  "Contrast";
    "grade_saturation"      => grade_saturation,     -1.0,   1.0,  "Saturation";
    "grade_temperature"     => grade_temperature,    -1.0,   1.0,  "Temperature";
    "grade_shadow_tint"     => grade_shadow_tint,    -1.0,   1.0,  "Shadow tint";
    "grade_highlight_tint"  => grade_highlight_tint, -1.0,   1.0,  "Highlight tint";
    "grade_gamma"           => grade_gamma,          -1.0,   1.0,  "Gamma";
    // -- The grade's four field-influence weights. Not axes: a weight scales
    //    the six axes above per cell, so all four are inert while the grade
    //    itself is at rest (which is why `every_tunable_is_load_bearing`
    //    exempts them by name and a dedicated test covers them instead). --
    "grade_field_biome"     => grade_field_biome,    -1.0,   1.0,  "Biome influence";
    "grade_field_elevation" => grade_field_elevation,-1.0,   1.0,  "Elevation influence";
    "grade_field_moisture"  => grade_field_moisture, -1.0,   1.0,  "Moisture influence";
    "grade_field_geology"   => grade_field_geology,  -1.0,   1.0,  "Geology influence";
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

// `smoothstep()` (reference HTML line 7569). One implementation, in
// `cartalith-jsmath` — this copy guarded a zero width but not a NaN one, and
// the reference's `||` is JS truthiness, which is falsy for both.
use cartalith_jsmath::smoothstep;

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
}

/// Rec.709 relative luminance of a 0-255 colour. The one weighting this file
/// already used in three places (`paper_tone`, `apply_paper`,
/// `apply_local_contrast`), named so the chroma and grade stages below cannot
/// drift onto a different one.
fn luma(c: Rgb) -> f64 {
    0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2
}

/// Scale a colour's chroma about its own luminance. `k = 1` is the identity,
/// `0` is greyscale, `> 1` is more colourful — and the luminance is **exactly**
/// unchanged at every `k`, which is what separates this from the reference's
/// `bio_blend` lerp toward a fixed grey (that one changes value as well as
/// chroma, so a desaturated map is also a flatter one).
fn saturate(c: Rgb, k: f64) -> Rgb {
    let y = luma(c);
    (y + (c.0 - y) * k, y + (c.1 - y) * k, y + (c.2 - y) * k)
}

/// Shift a colour along a blue↔amber axis by `t` (`-1` cool, `+1` warm),
/// keeping the luminance approximately fixed: red and blue move in opposition
/// and green takes the small share that balances their Rec.709 weights, so a
/// warmed map is not also a brightened one.
fn temperature_shift(c: Rgb, t: f64) -> Rgb {
    // 0.2126·(+a) + 0.0722·(-a) leaves +0.1404·a of luma to cancel, and
    // green carries 0.7152 of it — hence the -0.1963 coefficient.
    let a = t * 0.18;
    (c.0 * (1.0 + a), c.1 * (1.0 - a * 0.1963), c.2 * (1.0 - a))
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

/// The reference's own `SVF_MAX`/`SVF_RELK_DIV` (line 8031) and `SHADOW_MAX`
/// (8056). `relK = W / SVF_RELK_DIV` is what converts a height *difference* in
/// the field's own `[0, 1]` units into a slope against a distance measured in
/// cells — the two lighting fields below share it, which is why the divisor is
/// named once here rather than twice inline.
const SVF_MAX: f64 = 0.55;
const SVF_RELK_DIV: f64 = 6.0;
const SHADOW_MAX: f64 = 0.45;

/// The sun elevation `buildSunShadowField` is called with, at **both** of the
/// reference's own call sites (8436 on screen, 11921 in the bake): a literal
/// `20`, deliberately not `state.sunAlt`.
///
/// Kept literal here rather than wired to [`TerrainAppearance::sun_alt_deg`],
/// because it is doing a different job from the hillshade's sun: a horizon
/// shadow only exists at all at a *low* sun, and at this port's default
/// `sun_alt_deg` of 40° almost nothing on a real heightfield rises above the
/// ray. Reading the map's own sun would therefore make the stage silently
/// inert at the default rather than adjustable — the reference's constant is
/// the behaviour, and this is a note that it is a constant on purpose.
const SHADOW_SUN_ALT_DEG: f64 = 20.0;

/// `buildSVFField` (reference HTML 8032-8051) — sky-view factor as a per-cell
/// brightness **multiplier**, `1` on open ground and down to `1 - SVF_MAX·k`
/// in fully enclosed terrain.
///
/// Eight azimuths, six log-spaced distances; each direction contributes
/// `sin θ_horizon`, derived from the tangent by `s / hypot(1, s)` exactly as
/// the reference derives it. `js_hypot` and `js_round` rather than the Rust
/// library's, per `cartalith-rust-conventions`' "V8's libm is not Rust's" —
/// the rounding matters here in particular, since it chooses which *cell* each
/// sample lands on and a half-cell disagreement is a different height.
///
/// The `break` on leaving the grid is the reference's, and is not the same as
/// `continue`: a direction that runs off the edge stops looking rather than
/// skipping to a farther sample, so the horizon is never completed across a
/// gap. Edge-clamping instead would invent a wall at the plate boundary.
pub(crate) fn build_svf(field: &[f32], gw: usize, gh: usize, k: f64) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![1f32; n];
    let rel_k = gw as f64 / SVF_RELK_DIV;
    const DIRS: usize = 8;
    let mut dcos = [0f64; DIRS];
    let mut dsin = [0f64; DIRS];
    for d in 0..DIRS {
        let a = d as f64 * std::f64::consts::PI * 2.0 / DIRS as f64;
        dcos[d] = a.cos();
        dsin[d] = a.sin();
    }
    const DIST: [f64; 6] = [2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
    // Row-parallel on `BakeFields::new`'s own determinism argument: every
    // output cell is a pure function of the immutable field and each row
    // writes its own disjoint slice.
    out.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        for x in 0..gw {
            let h = field[y * gw + x] as f64;
            let mut sum = 0.0;
            for d in 0..DIRS {
                let mut max_s = 0.0f64;
                for st in DIST {
                    let nx = cartalith_jsmath::js_round(x as f64 + dcos[d] * st);
                    let ny = cartalith_jsmath::js_round(y as f64 + dsin[d] * st);
                    if nx < 0.0 || nx >= gw as f64 || ny < 0.0 || ny >= gh as f64 {
                        break;
                    }
                    let dh = field[ny as usize * gw + nx as usize] as f64 - h;
                    if dh <= 0.0 {
                        continue;
                    }
                    let s = (dh * rel_k) / st;
                    if s > max_s {
                        max_s = s;
                    }
                }
                sum += max_s / cartalith_jsmath::js_hypot(1.0, max_s);
            }
            let svf = 1.0 - sum / DIRS as f64;
            row[x] = (1.0 - k.min(1.0) * SVF_MAX * (1.0 - svf)) as f32;
        }
    });
    out
}

/// `buildSunShadowField` (reference HTML 8057-8074) — horizon cast shadows as
/// a per-cell multiplier, `1` lit and down to `1 - SHADOW_MAX·k` in full
/// shadow, with a `smoothstep(0, 0.25, block)` penumbra.
///
/// One march per cell toward the sun over ten log-spaced steps, keeping the
/// largest amount by which a blocker rises *above the sun ray* rather than the
/// first hit — which is what makes the shadow soften with the blocker's height
/// instead of being a hard binary mask.
///
/// `dx = sin(az)`, `dy = -cos(az)` is the reference's own light-vector
/// convention, shared with `shadeFactor`, so the shadows fall on the side of a
/// range the hillshade already darkens rather than at right angles to it.
pub(crate) fn build_sun_shadow(field: &[f32], gw: usize, gh: usize, az_deg: f64, alt_deg: f64, k: f64) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![1f32; n];
    let rel_k = gw as f64 / SVF_RELK_DIV;
    let az = az_deg.to_radians();
    let (dx, dy) = (az.sin(), -az.cos());
    let tan_alt = alt_deg.to_radians().tan();
    const DIST: [f64; 10] = [2.0, 3.0, 5.0, 8.0, 12.0, 18.0, 27.0, 40.0, 60.0, 90.0];
    out.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        for x in 0..gw {
            let h = field[y * gw + x] as f64;
            let mut block = 0.0f64;
            for st in DIST {
                let nx = cartalith_jsmath::js_round(x as f64 + dx * st);
                let ny = cartalith_jsmath::js_round(y as f64 + dy * st);
                if nx < 0.0 || nx >= gw as f64 || ny < 0.0 || ny >= gh as f64 {
                    break;
                }
                let dh = field[ny as usize * gw + nx as usize] as f64 - h;
                if dh <= 0.0 {
                    continue;
                }
                let over = (dh * rel_k) / st - tan_alt;
                if over > block {
                    block = over;
                }
            }
            row[x] = (1.0 - k.min(1.0) * SHADOW_MAX * smoothstep(0.0, 0.25, block)) as f32;
        }
    });
    out
}

/// `aoC = (_aoField…) * (_svfField…) * (_shadowField…)` (reference 8167, and
/// the bake's own copy at 11967) — fold the two horizon fields into the
/// cavity-map AO multiplier the rest of this renderer already carries.
///
/// **Folded into `ao` rather than carried as two more `RenderCtx` fields, and
/// that is the point.** `ao` has two consumers — [`cell_color`] reads
/// `ctx.ao[i]`, [`BakeFields::pixel`] reads `sample_arr(&ctx.ao, …)` — and
/// this project's recorded failure mode is a capability that reaches one
/// consumer path and not the other (`with_ground_tiles` shipped that way and
/// every exported PNG was a different picture from the screen). Multiplying
/// here makes the on-screen texture and every export path pick both fields up
/// by construction, with no second site to keep in sync.
///
/// Byte-identity at rest is **by control flow**: at `0.0` neither field is
/// built and `ao` is not written at all, so there is no `× 1.0` to reason
/// about.
///
/// `pub(crate)` rather than private so `geology_micro_and_sky_fields.rs` can
/// assert what [`SHADOW_SUN_ALT_DEG`] claims: that this call passes the
/// reference’s literal `20` and **not** [`TerrainAppearance::sun_alt_deg`].
/// Through a rendered image that claim is untestable — moving the map’s sun
/// moves the hillshade too, so every pixel differs either way and no
/// assertion can tell which caused it.
pub(crate) fn fold_lighting_fields(ao: &mut [f32], field: &[f32], gw: usize, gh: usize, a: &TerrainAppearance) {
    if a.svf_strength > 0.0 {
        let svf = build_svf(field, gw, gh, a.svf_strength);
        for (o, s) in ao.iter_mut().zip(svf) {
            *o *= s;
        }
    }
    if a.shadow_strength > 0.0 {
        let sh = build_sun_shadow(field, gw, gh, a.sun_az_deg, SHADOW_SUN_ALT_DEG, a.shadow_strength);
        for (o, s) in ao.iter_mut().zip(sh) {
            *o *= s;
        }
    }
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
/// `CREST_SLOPE_HI` (reference HTML line 8005) — the slope at which the
/// `G^1.5` crest weight saturates, in coarse-cell units.
const CREST_SLOPE_HI: f64 = 0.05;

/// `buildCrestField(fld, W, H, sea, sx, sy)` (reference HTML 8008-8022) —
/// per-cell strength of the thin bright stroke that runs along a ridge line:
/// convexity (a negative Laplacian) times a `G^1.5` slope weight, and zero
/// everywhere the ground is concave, flat or under water.
///
/// `sx`/`sy` are **coarse cells per sample step** — the reference's own
/// parameters, `1` on the `GW x GH` main map and `cx`/`cy` on an amplified
/// tile, so the slope gate and the Laplacian stay on the coarse-cell scale
/// whatever resolution the samples were taken at. Added 2026-09-21 for
/// LOD-D1; the main map passes `1.0, 1.0`, where `invc` is exactly `1.0` and
/// `2.0 * sx` is exactly `2.0`, so that call is bit-identical to what it was
/// before the parameters existed (`build_crest_at_unit_scale_leaves_the_screen_unchanged`).
///
/// `js_hypot` rather than `f64::hypot`: `cartalith-rust-conventions`' "V8's
/// libm is not Rust's", and this is a gradient magnitude feeding a `powf`
/// whose result multiplies a colour.
///
/// Built only when `crest_strength > 0`; `RenderCtx` holds an empty `Vec`
/// otherwise, so the stage costs one length test on every other path.
/// `step` is the stencil's half-width **in samples**, added 2026-09-21 for
/// LOD-D5's *"crest and AO radii are set in ground units"*. `1` is the
/// reference's own stencil and every pre-LOD-D5 call site passes it, so those
/// calls are bit-identical to what they were before the parameter existed
/// (`golden_parity_tile_biome.rs`'s
/// `build_crest_at_unit_scale_leaves_the_screen_unchanged`, which holds the
/// screen path, and `tests/lod_d5_scale_aware.rs`'s
/// `a_unit_step_is_the_stencil_the_crest_always_had`, which holds the field
/// itself). A tile drawn at
/// `cells_per_px < 1` passes a larger step **and scales `sx`/`sy` by the same
/// number**, so the curvature and the slope gate keep measuring one coarse
/// cell of ground however far past the grid the view has zoomed — which is
/// what stops the crest stroke from collapsing onto whatever single tile pixel
/// happens to be convex and shimmering as the view moves.
///
/// **The trade-off is real and is chosen deliberately.** A one-pixel stencil
/// at deep zoom picks out the crests of `add_zoom_detail`'s own octaves, which
/// is *more* line work, not less. It is refused because that line work is a
/// screen-space feature — one pixel wide at every zoom — and a feature whose
/// size is fixed in pixels is exactly the thing LOD-D3's no-popping criterion
/// is written against. A crest that keeps its ground width grows on screen as
/// the view comes in, which is what a ridge does.
fn build_crest(field: &[f32], gw: usize, gh: usize, sea: f64, sx: f64, sy: f64, step: usize, a: &TerrainAppearance) -> Vec<f32> {
    if a.crest_strength <= 0.0 {
        return Vec::new();
    }
    // `sx=sy=1` in the reference's own default-argument form (`sx=sx||1`).
    let invc = 1.0 / (sx * sy);
    let p = step.max(1);
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let h = field[i] as f64;
            if h < sea {
                continue;
            }
            let xl = if x >= p { x - p } else { x };
            let xr = if x + p < gw { x + p } else { x };
            let yu = if y >= p { y - p } else { y };
            let yd = if y + p < gh { y + p } else { y };
            let l = field[y * gw + xl] as f64;
            let r = field[y * gw + xr] as f64;
            let u = field[yu * gw + x] as f64;
            let d = field[yd * gw + x] as f64;
            let curv = (l + r + u + d - 4.0 * h) * invc;
            if curv >= 0.0 {
                continue;
            }
            let g = cartalith_jsmath::js_hypot((r - l) / (2.0 * sx), (d - u) / (2.0 * sy));
            let sg = (g / CREST_SLOPE_HI).min(1.0).powf(1.5);
            let conv = clamp01(-curv * 250.0);
            out[i] = (conv * sg) as f32;
        }
    }
    out
}

/// `applyCoastRiverSDFv`'s coast leg (reference HTML 8135-8138) — two
/// constant-width bands measured from the coastline itself rather than from
/// elevation: a bright wet shore-sand band inside 2.5 cells, and a lusher
/// coastal plain from there out to 14.
///
/// `S = max(1, gw/256)` is the reference's resolution normaliser, and it is
/// what "constant width" means here — the bands stay the same *world* width
/// whether the grid is 512 or 8192 cells across, because the distance is
/// divided by it before either `smoothstep` sees it.
///
/// `din = -sdf / S`, positive on land: the field's sign convention is negative
/// inland (`cartalith_civ::build_coast_sdf`, matching `buildCoastSDF`), so the
/// negation is the reference's own and not a correction of it. An offshore
/// cell yields a negative `din` and takes neither band — which is why this
/// needs no water test, and also why it is called on the land path only,
/// exactly where `surfaceColor` calls it.
///
/// `plain` carries the reference's `(1 - beach)` factor so the two bands do
/// not both paint the first 2.5 cells.
///
/// **Not the sub-pixel coastline AA.** The reference has a second, unrelated
/// use of the same field (8544): blending the sea and land branches across a
/// `smoothstep(-0.6, 0.6, sd)` band instead of the hard `isWater` step. That
/// is a different feature in a different place (the branch dispatch, not the
/// material path) and is not ported here.
pub(crate) fn apply_coast_sdf(c: Rgb, sdf: f64, k: f64, gw: usize) -> Rgb {
    let s = (gw as f64 / 256.0).max(1.0);
    let din = -sdf / s;
    if din < 0.0 {
        return c;
    }
    let beach = smoothstep(2.5, 0.0, din);
    let plain = smoothstep(14.0, 2.5, din) * (1.0 - beach);
    let mut c = c;
    if beach > 0.0 {
        let bk = k * beach * 0.5;
        c = (c.0 * (1.0 - bk) + 232.0 * bk, c.1 * (1.0 - bk) + 214.0 * bk, c.2 * (1.0 - bk) + 168.0 * bk);
    }
    if plain > 0.0 {
        let pk = k * plain * 0.22;
        c = (c.0 * (1.0 - pk) + 150.0 * pk, c.1 * (1.0 - pk) + 168.0 * pk, c.2 * (1.0 - pk) + 108.0 * pk);
    }
    c
}

/// `applyCoastRiverSDFv`'s river leg (reference HTML 8178-8182) — three
/// widening rings out from a channel: a damp bank inside 2 cells, wetland
/// green out to 7, floodplain out to 16, each carrying the `(1 - inner)`
/// factors that stop the bands stacking on the same pixel.
///
/// `S = max(1, gw/256)` is the same resolution normaliser [`apply_coast_sdf`]
/// documents, and `dr = sdf / S` is **not** negated here: `buildRiverSDF`'s
/// sign convention is already negative *inside* a channel and positive away
/// from it — the opposite way round from `buildCoastSDF` — so the reference
/// negates one and not the other. The `dr > 0` test is what keeps the bands
/// off the channel cells themselves, exactly as `din >= 0` keeps the coast
/// bands out of the water.
pub(crate) fn apply_river_sdf(c: Rgb, sdf: f64, k: f64, gw: usize) -> Rgb {
    let s = (gw as f64 / 256.0).max(1.0);
    let dr = sdf / s;
    if dr <= 0.0 {
        return c;
    }
    let bank = smoothstep(2.0, 0.0, dr);
    let wet = smoothstep(7.0, 2.0, dr) * (1.0 - bank);
    let flood = smoothstep(16.0, 7.0, dr) * (1.0 - bank) * (1.0 - wet);
    let mut c = c;
    if bank > 0.0 {
        let bk = k * bank * 0.30;
        c = (c.0 * (1.0 - bk) + 96.0 * bk, c.1 * (1.0 - bk) + 120.0 * bk, c.2 * (1.0 - bk) + 96.0 * bk);
    }
    if wet > 0.0 {
        let wk = k * wet * 0.22;
        c = (c.0 * (1.0 - wk) + 88.0 * wk, c.1 * (1.0 - wk) + 128.0 * wk, c.2 * (1.0 - wk) + 86.0 * wk);
    }
    if flood > 0.0 {
        let fk = k * flood * 0.14;
        c = (c.0 * (1.0 - fk) + 120.0 * fk, c.1 * (1.0 - fk) + 150.0 * fk, c.2 * (1.0 - fk) + 104.0 * fk);
    }
    c
}

/// `sdfEcoKv(biomeBDv)` (reference HTML 8172) — the ecotone widener
/// [`land_color`] takes as `eco_k`: `1` in a biome's interior, rising to
/// `1 + 1.5·k` on a boundary, over a `6·S`-cell falloff.
///
/// **`1.0` is off, not `0.0`.** The reference's own `ecoK != null ? ecoK : 1`
/// says the same thing, and it is why the caller passes a literal `1.0`
/// whenever the field is empty rather than calling this with a sentinel
/// distance — a "no boundary here" distance does not exist, and inventing one
/// (`1e9`, say) would put a real number through `smoothstep` and rely on it
/// rounding back to `1`.
pub(crate) fn sdf_eco_k(biome_bd: f64, k: f64, gw: usize) -> f64 {
    let s = (gw as f64 / 256.0).max(1.0);
    1.0 + k * 1.5 * smoothstep(6.0 * s, 0.0, biome_bd)
}

/// `buildRiverSDF(flow, W, H, {euclid: true})` (reference HTML 7509-7516):
/// signed distance to the discharge channel mask, **negative inside a
/// channel** and positive away from it, in cells.
///
/// # This is `build_coast_sdf` over a different mask, and that is exact
///
/// The reference writes `buildCoastSDF` and `buildRiverSDF` as the same six
/// lines with two names substituted, so composing the public one is an
/// identity rather than a reuse of convenience. Term by term, with
/// `mask[i] = 1 if flow[i] > thresh else 0` and `sea = 0.5`:
///
/// | `buildCoastSDF(mask, 0.5)` | `buildRiverSDF(flow, thresh)` |
/// |---|---|
/// | `water = mask < 0.5` | `notRiv = flow <= thresh` — the same cells |
/// | `land  = mask >= 0.5` | `riv    = flow >  thresh` — the same cells |
/// | `dToLand  = distMask(land)` | `dToRiv = distMask(riv)` |
/// | `dToWater = distMask(water)` | `dToNot = distMask(notRiv)` |
/// | out: `mask<0.5 ? +dToLand : -dToWater` | out: `flow>thresh ? -dToNot : +dToRiv` |
///
/// The two output expressions are the same two branches with the same two
/// fields, so every bit of the result is identical — including the `1e9`
/// no-seed sentinel, which both inherit from the same `jfa_dist`. Building
/// the mask at the `>` boundary rather than reusing `flow` directly as the
/// field is the load-bearing step: `build_coast_sdf` splits at `<`, and the
/// two would disagree on any cell whose discharge equalled the threshold
/// exactly.
///
/// The alternative was a second `jfa_dist` in this crate, since
/// `cartalith_civ`'s is private. One distance transform with one set of
/// golden tests is worth more than a private duplicate that agrees today —
/// and **the transform is the reference's jump flood, which is an
/// approximation**, whatever its own comment (and `cartalith_civ::jfa_dist`'s,
/// which copies it) calls it. Exact from a single seed; from many it can keep
/// a farther seed than the true nearest, measured at one cell of an 11x9 grid
/// in `sdf_river_and_biome.rs`. Reproducing those misses is the requirement,
/// not a defect to fix: the reference's picture is the target.
pub(crate) fn build_river_sdf(flow: &[f32], gw: usize, gh: usize, thresh: f64) -> Vec<f32> {
    let mask: Vec<f32> = flow.iter().map(|&f| if f as f64 > thresh { 1.0 } else { 0.0 }).collect();
    cartalith_civ::build_coast_sdf(&mask, gw, gh, 0.5)
}

/// `buildBiomeBoundaryDist(biome, W, H, {euclid: true})` (reference HTML
/// 7519-7525): unsigned distance in cells to the nearest cell of a
/// *different* biome index — `0` on a boundary, growing into each biome's
/// interior.
///
/// The edge mask is the reference's own 4-neighbour test, with its
/// grid-interior guards (a cell on the left column is never compared against
/// the right column's last cell, which in `world` mode would otherwise wrap).
///
/// Same composition as [`build_river_sdf`], with one extra step:
/// `build_coast_sdf` returns `-dToNonEdge` on the seed cells, where
/// `distMask` returns `0`. Those are written back as `0.0` rather than
/// `.max(0.0)`-clamped, because `jfa_dist` yields exactly `sqrt(0)` at a seed
/// and an exact zero is what the reference's `smoothstep(6S, 0, d)` needs to
/// reach its full value on a boundary. The wasted second transform inside
/// `build_coast_sdf` is the price of not duplicating `jfa_dist`; it is paid
/// only while the slider is up.
pub(crate) fn build_biome_boundary_dist(biome: &[u8], gw: usize, gh: usize) -> Vec<f32> {
    let n = gw * gh;
    let mut edge = vec![0f32; n];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let b = biome[i];
            if (x > 0 && biome[i - 1] != b)
                || (x + 1 < gw && biome[i + 1] != b)
                || (y > 0 && biome[i - gw] != b)
                || (y + 1 < gh && biome[i + gw] != b)
            {
                edge[i] = 1.0;
            }
        }
    }
    let mut d = cartalith_civ::build_coast_sdf(&edge, gw, gh, 0.5);
    for i in 0..n {
        if edge[i] != 0.0 {
            d[i] = 0.0;
        }
    }
    d
}

/// `applyCrest(c, s)` (reference HTML line 8023) — blend the finished colour
/// toward the crest's own near-white sunlit-rock tone at weight `s`, clamped
/// at 1. `s` arrives already folded with the slider **and the reference's own
/// `0.7`** by the caller, exactly as `surfaceColor` folds it (8171).
fn apply_crest(c: Rgb, s: f64) -> Rgb {
    if s <= 0.0 {
        return c;
    }
    let k = if s > 1.0 { 1.0 } else { s };
    (c.0 * (1.0 - k) + 240.0 * k, c.1 * (1.0 - k) + 238.0 * k, c.2 * (1.0 - k) + 232.0 * k)
}

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
    sea_h: Cow<'a, [f32]>,
    sea_shade: Cow<'a, [f32]>,
    /// The reference's `aoC` (8167): the cavity-map AO multiplier
    /// (`build_ao`) **times** the sky-view and cast-shadow multipliers
    /// ([`fold_lighting_fields`]) when either of those is on. All `1.0` when
    /// all three strengths are `0`, which is the reference's own state — and
    /// exactly `build_ao`'s output alone whenever only AO is on, which is
    /// `default()`'s state and every tier's.
    ao: Cow<'a, [f32]>,
    /// `_coastSDF` (12089) — signed distance to the coastline in cells,
    /// negative inland and positive offshore
    /// (`cartalith_civ::build_coast_sdf`, whose sign convention is the
    /// reference's own). **Empty** unless `appearance.sdf_coast > 0`, on
    /// [`Self::coast_d`]'s contract: the consumer tests the length rather
    /// than re-reading the flag, so the field and the gate cannot disagree.
    ///
    /// Not the same field as [`Self::coast_d`], despite the names: that one
    /// is an unsigned chamfer distance to land, built only for the wave
    /// contours and only over water.
    coast_sdf: Cow<'a, [f32]>,
    /// `_riverSDF` (8486) — signed distance to the discharge channel mask,
    /// **negative inside a channel** ([`build_river_sdf`], whose sign is the
    /// reference's and is the opposite way round from [`Self::coast_sdf`]).
    ///
    /// **Empty by construction**, and filled only by
    /// [`Self::with_map_scale`] — the threshold it needs is
    /// `riverFlowThresh`, which takes a map width in km that nothing else in
    /// this struct carries. A caller that never attaches one gets the off
    /// state rather than a guessed width, and every consumer tests this
    /// field's length rather than `appearance.sdf_rivers`.
    river_sdf: Cow<'a, [f32]>,
    /// `_biomeBD` (8487) — unsigned distance in cells to the nearest cell of
    /// a different biome ([`build_biome_boundary_dist`]). Empty by
    /// construction, on `river_sdf`'s contract; where it is empty the
    /// ecotone widener is the literal `1.0`, not a computed one.
    biome_bd: Cow<'a, [f32]>,
    /// Per-cell "near water" tint strength (`build_hydro_wetness`). All
    /// `0.0` when `appearance.hydro_wet_strength == 0`, which is the
    /// reference's own state.
    hydro_wet: Cow<'a, [f32]>,
    /// Precomputed weighted light directions (`build_lights`).
    lights: Cow<'a, [(f64, f64, f64, f64)]>,
    /// `_coastDCache` (8440) — per-cell distance in cells to the nearest
    /// land, for the wave/foam contours. **Empty** unless `npr.waves` is on,
    /// which is what makes the whole stage free at the default settings
    /// (`cell_color` tests the length, not a flag, so the two can never
    /// disagree).
    coast_d: Cow<'a, [f32]>,
    /// `_crestField` (8434) — per-cell ridge-crest stroke strength
    /// (`build_crest`). **Empty** unless `crest_strength > 0`, on `coast_d`'s
    /// own contract: the consumer tests the length rather than a flag, so the
    /// two cannot disagree.
    crest: Cow<'a, [f32]>,
    /// The renderer's colour data/shading constants (`TerrainAppearance`'s
    /// own doc comment). Settable via `with_appearance` as of milestone 2,
    /// and wired to the shell: `WorldGen::set_appearance` (`lib.rs`) writes
    /// `appearance_over`, which `render_workspace.gd`'s appearance panel
    /// drives; the golden-parity test builds a `TerrainAppearance` directly
    /// to pin the exact JS path.
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
    /// `riverFlowThresh(GW, GH)` — the discharge above which a cell counts as
    /// a river channel, for `river_sdf` and for the **tile** path's own
    /// per-tile SDF (`render_biome_tile_rgba`, reference 11683, which passes
    /// the grid's threshold rather than one derived from the tile so a tile
    /// and the map cannot disagree about which channels are rivers).
    ///
    /// `0.0` by construction and set only by [`Self::with_map_scale`], on
    /// `river_sdf`'s own contract: the threshold needs a map width in km that
    /// nothing else in this struct carries, and a caller that never attaches
    /// one gets the off state rather than a guessed width. The tile path tests
    /// this value rather than `appearance.sdf_rivers`, so the gate and the
    /// number cannot disagree.
    river_thresh: f64,
    /// A loaded pack's decoded `biomes`/`terrains` ground tiles, which the
    /// paint blend prefers over the flat palette swatch (reference v1.28's
    /// `_paintedTex`). Empty by construction — attach with
    /// [`Self::with_ground_tiles`]. Empty is the pack-less state and is
    /// byte-identical to what this renderer did before the field existed.
    ground: GroundTiles<'a>,
    /// `_lakeWB` (reference 8460) -- the water-body classification (`2` =
    /// lake) the base map loop stamps above-sea lakes from (v0.103, 8580).
    /// `None` by construction; attach with [`Self::with_lakes`].
    lake_class: Option<&'a [u8]>,
    /// The rivers rasterized at THIS raster's resolution ([`RiverLayer`]),
    /// one pixel per cell. `None` draws no river symbol -- the state of every
    /// render but the screen texture's (`WorldGen::build_color_texture`).
    river_layer: Option<&'a RiverLayer>,
    /// **RV-3: the world's own height, when `field` is not it.** `field` is
    /// the height this context SHADES; since RV-3 the app hands it
    /// `WorldGen::valley_shade_field` (the carve filled in, a valley cut along
    /// each drawn river), which must never decide where water is. Everything
    /// here that classifies water or colours the sea from surrounding ground
    /// -- the water bodies a tile builds, the biome-boundary distance, the
    /// smoothed bathymetry -- reads [`Self::water_height`] instead, which is
    /// this when attached and `field` otherwise. `None` by construction
    /// (`field` is then the world's height, as it was for every caller before
    /// RV-3); attach with [`Self::with_water_height`] or, over a precompute
    /// built by [`GridPrecompute::build_split`], with
    /// [`Self::with_precomputed_water_height`].
    water_field: Option<&'a [f32]>,
}

/// Everything [`RenderCtx::with_appearance`] and [`RenderCtx::with_map_scale`]
/// compute for themselves, owned separately from any `RenderCtx` — so a
/// caller that builds many contexts over **one world and one appearance** pays
/// for them once.
///
/// This exists for the LOD tile path (`LOD_DETAIL_SCOPE.md` LOD-D2) and for
/// nothing else today. A `RenderCtx` borrows five input slices plus a
/// lithology `Vec` its caller owns, so it cannot be stored beside the world it
/// reads (`export_raster.rs::export_render`'s own doc comment records that
/// conclusion, and takes a closure instead). What *can* be stored is this:
/// grid-resolution rasters that depend only on `(field, temperature, rainfall,
/// flow, sea_level, world, appearance, map_width_km)` and on nothing the
/// caller re-derives per call.
///
/// **Measured, which is why it exists:** at 2048x1311, release, this machine —
/// `with_appearance` + `with_map_scale` is 201.5 ms (199.5..202.2 over 5) and
/// [`TileFields::new`] a further 278.2 ms (273.2..281.7). One 256^2 tile is
/// 5.98 ms on all cores (51 ms single-threaded), so rebuilding either per tile
/// would cost eighty times the tile itself, and `viewport_host.gd` asks for up
/// to 48 of them on one zoom notch. `lod_bridge`'s
/// `the_tile_context_costs_an_order_of_magnitude_more_than_the_tile_it_serves`
/// is where those numbers come from and is how to re-take them.
///
/// **Byte-identical by construction, not by re-derivation.**
/// `with_appearance` now calls [`Self::build`] rather than holding a second
/// copy of the same eight calls, so there is one place where a precompute is
/// defined and the cached and uncached paths cannot drift.
pub struct GridPrecompute {
    sea_h: Vec<f32>,
    sea_shade: Vec<f32>,
    ao: Vec<f32>,
    coast_sdf: Vec<f32>,
    river_sdf: Vec<f32>,
    biome_bd: Vec<f32>,
    hydro_wet: Vec<f32>,
    lights: Vec<(f64, f64, f64, f64)>,
    coast_d: Vec<f32>,
    crest: Vec<f32>,
    river_thresh: f64,
    /// The grid this was built for, so [`RenderCtx::from_precomputed`] can
    /// refuse a mismatched one rather than index out of bounds — a panic here
    /// crosses the gdext boundary (`cartalith-rust-conventions`).
    gw: usize,
    gh: usize,
}

impl GridPrecompute {
    /// The prologue of [`RenderCtx::with_appearance`], plus
    /// [`RenderCtx::with_map_scale`]'s two fields when `map_width_km` is
    /// `Some`.
    ///
    /// `map_width_km: None` is exactly the state a `RenderCtx` is in before
    /// `with_map_scale` is called — empty `river_sdf`/`biome_bd` and a
    /// `river_thresh` of `0.0` — which is what makes `with_appearance`'s own
    /// call to this function a no-behaviour-change refactor.
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn build(
        field: &[f32],
        temperature: &[f32],
        rainfall: &[f32],
        flow: Option<&[f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        appearance: &TerrainAppearance,
        map_width_km: Option<f64>,
    ) -> Self {
        Self::build_forced(field, temperature, rainfall, flow, gw, gh, sea_level, world, appearance, map_width_km, None)
    }

    /// [`Self::build`] with Ruling BO's forced-lake mask, which only the
    /// `sdf_biomes` leg reads ([`grid_biome_boundary_dist`]) -- so a forced
    /// lake gets the same biome band at its shore a natural lake does, in the
    /// LOD snapshot and the export session exactly as on screen
    /// ([`RenderCtx::with_map_scale_forced`]). `None` is [`Self::build`],
    /// byte for byte; it must never be substituted with an all-zero mask
    /// standing in for "no forced lake" (`MISTAKES.md`: no value is `None`).
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn build_forced(
        field: &[f32],
        temperature: &[f32],
        rainfall: &[f32],
        flow: Option<&[f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        appearance: &TerrainAppearance,
        map_width_km: Option<f64>,
        forced_lakes: Option<&[u8]>,
    ) -> Self {
        Self::build_split(field, field, temperature, rainfall, flow, gw, gh, sea_level, world, appearance, map_width_km, forced_lakes)
    }

    /// [`Self::build_forced`] over two heights (RV-3): `field`, the one the
    /// map is shaded from (`WorldGen::valley_shade_field`), for the relief
    /// rasters -- AO and its sky/shadow folds, the crest strokes -- and
    /// `water`, the world's own, for everything that decides or colours
    /// water -- the smoothed bathymetry and its shade, the wave and coast
    /// distances, the biome-boundary distance. `build_forced` is this with
    /// `water == field`, byte for byte. A context over the result attaches
    /// `water` with [`RenderCtx::with_precomputed_water_height`].
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn build_split(
        field: &[f32],
        water: &[f32],
        temperature: &[f32],
        rainfall: &[f32],
        flow: Option<&[f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        appearance: &TerrainAppearance,
        map_width_km: Option<f64>,
        forced_lakes: Option<&[u8]>,
    ) -> Self {
        let sea_h = smooth_sea_h(water, gw, gh, world);
        let sea_shade = sea_shade_from(&sea_h, gw, gh, appearance);
        let mut ao = build_ao(field, gw, gh, sea_level, world, appearance);
        fold_lighting_fields(&mut ao, field, gw, gh, appearance);
        let hydro_wet = build_hydro_wetness(flow, gw, gh, world, appearance);
        let lights = build_lights(appearance);
        let coast_d = if appearance.npr.waves { coast_distance(water, gw, gh, sea_level) } else { Vec::new() };
        // The reference builds its SDFs in `renderNow` (8446) rather than in
        // the material path, and only while the slider is up — `_coastSDF`'s
        // own comment is "null ⇒ off ⇒ render unchanged". A JFA over the whole
        // grid is far too expensive to pay for per render when nothing reads
        // it, so the gate is the allocation, not a branch inside the loop.
        let coast_sdf = if appearance.sdf_coast > 0.0 { cartalith_civ::build_coast_sdf(water, gw, gh, sea_level) } else { Vec::new() };
        // LOD-D5 gave `build_crest` a stencil step. The screen and bake path
        // passes `1` -- the reference's own stencil and the one this call has
        // always used -- so it is bit-identical to what it was before the
        // parameter existed.
        let crest = build_crest(field, gw, gh, sea_level, 1.0, 1.0, 1, appearance);
        // `with_map_scale`'s own body, to its own gates — see its doc comment
        // for why the threshold is set whenever a width is supplied and the
        // SDF only while its slider is up.
        let (mut river_sdf, mut biome_bd, mut river_thresh) = (Vec::new(), Vec::new(), 0.0);
        if let Some(km) = map_width_km {
            river_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, km);
            if appearance.sdf_rivers > 0.0
                && let Some(flow) = flow
            {
                river_sdf = build_river_sdf(flow, gw, gh, river_thresh);
            }
            if appearance.sdf_biomes > 0.0 {
                biome_bd = grid_biome_boundary_dist(water, temperature, rainfall, gw, gh, sea_level, world, forced_lakes);
            }
        }
        GridPrecompute { sea_h, sea_shade, ao, coast_sdf, river_sdf, biome_bd, hydro_wet, lights, coast_d, crest, river_thresh, gw, gh }
    }

    /// Retained bytes — the figure `LOD_DETAIL_SCOPE.md`'s Android line needs,
    /// counted rather than estimated.
    #[allow(dead_code)]
    pub fn bytes(&self) -> usize {
        (self.sea_h.len() + self.sea_shade.len() + self.ao.len() + self.coast_sdf.len() + self.river_sdf.len() + self.biome_bd.len() + self.hydro_wet.len() + self.coast_d.len() + self.crest.len()) * 4 + self.lights.len() * 32
    }
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
        // One body, shared with the cached path — see [`GridPrecompute`].
        // `None` for the map width is this constructor's own prior state:
        // `river_sdf`/`biome_bd` empty and `river_thresh` `0.0` until
        // `with_map_scale` is called.
        let p = GridPrecompute::build(field, temperature, rainfall, flow, gw, gh, sea_level, world, &appearance, None);
        RenderCtx {
            field,
            temperature,
            rainfall,
            flow,
            gw,
            gh,
            sea_level,
            world,
            lat_n,
            lat_s,
            sea_h: Cow::Owned(p.sea_h),
            sea_shade: Cow::Owned(p.sea_shade),
            ao: Cow::Owned(p.ao),
            coast_sdf: Cow::Owned(p.coast_sdf),
            river_sdf: Cow::Owned(p.river_sdf),
            biome_bd: Cow::Owned(p.biome_bd),
            river_thresh: p.river_thresh,
            hydro_wet: Cow::Owned(p.hydro_wet),
            lights: Cow::Owned(p.lights),
            coast_d: Cow::Owned(p.coast_d),
            crest: Cow::Owned(p.crest),
            appearance,
            splat: None,
            lithology: None,
            paint_biome: None,
            paint_terrain: None,
            paint_splat: None,
            ground: GroundTiles::default(),
            lake_class: None,
            river_layer: None,
            water_field: None,
        }
    }

    /// [`Self::with_appearance`] + [`Self::with_map_scale`] against rasters
    /// **already built**, borrowed rather than recomputed — the LOD tile
    /// path's own constructor (`LOD_DETAIL_SCOPE.md` LOD-D2).
    ///
    /// Every field a `RenderCtx` derives from the world comes from `p`; every
    /// field it borrows from the caller is still passed in, so the world's own
    /// slices are read in place and nothing is copied. The result is the same
    /// struct `with_appearance(...).with_map_scale(km)` would have produced,
    /// **provided `p` was built from the same arguments** — which is the
    /// caller's cache key to get right, not something this function can check
    /// (`MISTAKES.md`: *"Derive the list from the definition — every argument
    /// of the function you are guarding"*). What it *can* check is the grid,
    /// and it does: a `p` of the wrong size is refused with `None` rather than
    /// indexed.
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn from_precomputed(
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
        p: &'a GridPrecompute,
    ) -> Option<Self> {
        if p.gw != gw || p.gh != gh || gw == 0 || gh == 0 {
            return None;
        }
        Some(RenderCtx {
            field,
            temperature,
            rainfall,
            flow,
            gw,
            gh,
            sea_level,
            world,
            lat_n,
            lat_s,
            sea_h: Cow::Borrowed(&p.sea_h),
            sea_shade: Cow::Borrowed(&p.sea_shade),
            ao: Cow::Borrowed(&p.ao),
            coast_sdf: Cow::Borrowed(&p.coast_sdf),
            river_sdf: Cow::Borrowed(&p.river_sdf),
            biome_bd: Cow::Borrowed(&p.biome_bd),
            river_thresh: p.river_thresh,
            hydro_wet: Cow::Borrowed(&p.hydro_wet),
            lights: Cow::Borrowed(&p.lights),
            coast_d: Cow::Borrowed(&p.coast_d),
            crest: Cow::Borrowed(&p.crest),
            appearance,
            splat: None,
            lithology: None,
            paint_biome: None,
            paint_terrain: None,
            paint_splat: None,
            ground: GroundTiles::default(),
            lake_class: None,
            river_layer: None,
            water_field: None,
        })
    }

    /// Attach the world's real map width in km, which is the one input the
    /// B3/B4 SDF legs need and nothing else in this struct carries — and
    /// build both fields, when their sliders are up.
    ///
    /// A builder rather than a `with_appearance` parameter for the reason
    /// [`Self::with_lithology`] already records: `golden_parity_render.rs`
    /// and five other test files construct a `RenderCtx` positionally, and
    /// leaving those untouched is a property worth keeping. **Not calling it
    /// is the off state**, so every existing caller stays byte-identical
    /// whatever the sliders say — which is also why the two consumer paths
    /// test the fields' lengths and not the strengths.
    ///
    /// `riverFlowThresh(GW, GH)` is the reference's own threshold
    /// (`gw·gh·0.0004 / (terrainDetailK(GW, mapWidthKm) · riverCoarseEase(
    /// mapWidthKm))`), reached here through the port's single canonical copy
    /// in `cartalith_hydrology` rather than re-derived — the same call the
    /// civilisation pass, the landmark detector and the export atlas all
    /// make, so a river the map paints a band around is a river those agree
    /// exists.
    ///
    /// The water-body classification is rebuilt here with the identical
    /// arguments `lib.rs`'s civilisation pass uses, rather than threaded in:
    /// the renderer has to work for a world generated with the civilisation
    /// layer switched off, which is `with_lithology`'s own argument in
    /// `build_color_texture`.
    #[allow(dead_code)]
    pub fn with_map_scale(self, map_width_km: f64) -> Self {
        self.with_map_scale_forced(map_width_km, None)
    }

    /// [`Self::with_map_scale`], with Ruling BO's forced-lake mask
    /// (`WorldGen::forced_lake_mask`) applied to the water classification the
    /// `sdf_biomes` leg classifies biomes from -- so the biome band a lake's
    /// shore gets is drawn round a forced lake too, as the drawn water,
    /// Sample and routing already treat it (`STATUS.md`, "Map-data
    /// residuals"). Only `biome_bd` reads the mask; the river SDF and its
    /// threshold do not depend on water bodies at all.
    ///
    /// `None` is [`Self::with_map_scale`] byte for byte, and must stay so:
    /// every world without a forced lake, and every golden, goes through it.
    /// A mask of the wrong length is ignored ([`apply_forced_lakes`]).
    #[allow(dead_code)]
    pub fn with_map_scale_forced(mut self, map_width_km: f64, forced_lakes: Option<&[u8]>) -> Self {
        let (gw, gh) = (self.gw, self.gh);
        if self.appearance.sdf_rivers > 0.0
            && let Some(flow) = self.flow
        {
            let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
            self.river_sdf = Cow::Owned(build_river_sdf(flow, gw, gh, thresh));
        }
        // Retained separately from the field it built, because the **tile**
        // path needs the number and not the grid's distance transform: it
        // builds its own SDF over its own sampled flow and hands this
        // threshold to it (reference 11683). Set whenever a map width is
        // supplied, not only when the grid's own SDF stage is on, so a tile
        // can draw river bands on a map whose grid raster does not.
        self.river_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
        if self.appearance.sdf_biomes > 0.0 {
            // The world's height, not the shaded one (RV-3): this classifies
            // water bodies.
            self.biome_bd = Cow::Owned(grid_biome_boundary_dist(self.water_height(), self.temperature, self.rainfall, gw, gh, self.sea_level, self.world, forced_lakes));
        }
        self
    }

    /// The height water is decided from: the attached world height (RV-3,
    /// [`Self::water_field`]) or, when none is, the shaded `field` itself.
    pub fn water_height(&self) -> &'a [f32] {
        self.water_field.unwrap_or(self.field)
    }

    /// Attach the world's own height (RV-3), for a context whose `field` is
    /// the shaded valley field, and rebuild from it the two grid rasters this
    /// constructor built from `field` that read ground around the sea: the
    /// smoothed bathymetry and its shade (`smooth_sea_h` blurs over land
    /// within a few cells of the coast, which the valley can change). The
    /// wave and coast-SDF distances are per-cell land/water tests the valley
    /// cannot change (it never moves a cell across sea level), so they are
    /// left as built.
    ///
    /// Call before [`Self::with_map_scale_forced`], which reads
    /// [`Self::water_height`] for the biome-boundary distance. Refused (left
    /// `None`) when the length is not `gw * gh`, so the context keeps
    /// deciding water from `field` rather than indexing past a short grid;
    /// a no-op when `water` is `field` itself.
    #[allow(dead_code)]
    pub fn with_water_height(mut self, water: &'a [f32]) -> Self {
        if water.len() != self.gw * self.gh || std::ptr::eq(water, self.field) {
            return self;
        }
        self.water_field = Some(water);
        let sea_h = smooth_sea_h(water, self.gw, self.gh, self.world);
        self.sea_shade = Cow::Owned(sea_shade_from(&sea_h, self.gw, self.gh, &self.appearance));
        self.sea_h = Cow::Owned(sea_h);
        self
    }

    /// [`Self::with_water_height`] for a context from
    /// [`Self::from_precomputed`] whose `GridPrecompute` was built by
    /// [`GridPrecompute::build_split`] with this same `water`: the rasters
    /// are already the right ones, so only the height is attached. Must never
    /// be used over a precompute built from the shaded field alone -- the sea
    /// would then be coloured from it.
    #[allow(dead_code)]
    pub fn with_precomputed_water_height(mut self, water: &'a [f32]) -> Self {
        if water.len() == self.gw * self.gh && !std::ptr::eq(water, self.field) {
            self.water_field = Some(water);
        }
        self
    }

    /// Attach the water-body classification (`cartalith_civ::build_water_bodies`'
    /// `classification`), so [`cell_color`] draws above-sea lakes as water --
    /// the reference's v0.103 base-map stamp (8460/8580): *"above-sea lakes
    /// render as flat freshwater (below-sea lakes/inland seas already pass the
    /// isWater test -> rendered as sea)"*.
    ///
    /// **This was unported until 2026-09-23**, on a misreading of v1.05's
    /// *"The BASE per-cell map loop is untouched"* (11740) as "the base loop
    /// draws no lakes" -- it means v1.05's shoreline did not change the base
    /// loop, whose v0.103 stamp already drew them. The LOD tile path did draw
    /// them, so every place the tile blends against the base raster (the LOD
    /// entry band, a level whose parents are still building) faded a lake in
    /// and out: the owner's "lakes flickering on zoom". A builder, like
    /// [`Self::with_lithology`], so positional test callers stay unchanged.
    #[allow(dead_code)]
    pub fn with_lakes(mut self, lake_class: &'a [u8]) -> Self {
        if lake_class.len() == self.gw * self.gh {
            self.lake_class = Some(lake_class);
        }
        self
    }

    /// The look this context renders with -- read by the tile path to style
    /// the river layer it builds for the same tile (`lod_bridge`). Reads only.
    #[allow(dead_code)]
    pub fn appearance(&self) -> &TerrainAppearance {
        &self.appearance
    }

    /// Attach the rivers rasterized at grid resolution ([`RiverLayer`], one
    /// pixel per cell), read by [`cell_color`]. Refused (left `None`) when the
    /// layer is not exactly `gw x gh`, so a layer built for another grid cannot
    /// index past it. The app composites its river colour field per pixel
    /// instead (`cell_color_river`), so this is the tests' way in
    /// (`tests/appearance_tiers.rs`); it must never be attached on a golden or
    /// export path.
    #[allow(dead_code)]
    pub fn with_river_layer(mut self, layer: &'a RiverLayer) -> Self {
        if layer.width() == self.gw && layer.height() == self.gh {
            self.river_layer = Some(layer);
        }
        self
    }

    /// Attach the world's real rock types (milestone 5, §12). A builder for
    /// the same reason `with_splat` is one: `golden_parity_render.rs`
    /// constructs its `RenderCtx` positionally, and three milestones of
    /// leaving that file untouched is a property worth keeping. `len` is
    /// checked against the grid rather than trusted — a mismatched field
    /// would otherwise index out of bounds inside the render loop, and a
    /// panic there crosses the gdext boundary (`cartalith-rust-conventions`).
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

    /// Attach a loaded pack's decoded painted-ground tiles
    /// ([`GroundTiles`], built by `crate::pack::load_pack_from_bytes`).
    ///
    /// This is the *only* thing that makes the reference's v1.28 refinement
    /// reachable: without it every painted cell takes the flat-swatch branch,
    /// which is what a pack-less reference render takes too. Attaching tiles
    /// changes nothing on a cell no brush has painted, so a loaded pack alone
    /// still cannot move a pixel here.
    ///
    /// A builder method for the same reason [`Self::with_splat`] is one.
    #[allow(dead_code)]
    pub fn with_ground_tiles(mut self, ground: GroundTiles<'a>) -> Self {
        self.ground = ground;
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
        for &(lx, ly, lz, w) in self.lights.iter() {
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

    // -----------------------------------------------------------------
    // Fractional twins, for the export raster only
    //
    // The reference's own `curvatureAtF`/`aspectFactorF` (7624/7627) with
    // the same justification it gives there: `sampleArr` at an exact
    // integer coordinate returns the cell value outright, so each of these
    // is bit-identical to the integer method above at every cell — which
    // `tests/bake_raster.rs` asserts rather than assumes. The reference
    // stopped at two; this port needs six, because `cell_color` reaches
    // further than `surfaceColor` does (vignette, lithology jitter, paint,
    // and the hachure gradient are all its own).
    //
    // They are separate methods rather than a widened signature on the
    // integer ones deliberately: `cell_color` is the hot path for every
    // frame the shell draws, and it should keep indexing straight into the
    // field rather than paying a clamp and two lerps per read for a
    // coordinate it knows is a whole cell.
    // -----------------------------------------------------------------

    /// [`Self::h`] at a fractional position.
    fn h_f(&self, x: f64, y: f64) -> f64 {
        sample_arr(self.field, x, y, self.gw, self.gh)
    }

    /// `latAt` at a fractional row.
    fn lat_at_f(&self, y: f64) -> f64 {
        if self.world {
            90.0 - (y / (self.gh.max(2) - 1) as f64) * 180.0
        } else {
            self.lat_n + (y / (self.gh.max(2) - 1) as f64) * (self.lat_s - self.lat_n)
        }
    }

    /// [`Self::vignette_at`] at a fractional position.
    fn vignette_at_f(&self, x: f64, y: f64) -> f64 {
        let vx = x / (self.gw.max(2) - 1) as f64 - 0.5;
        let vy = y / (self.gh.max(2) - 1) as f64 - 0.5;
        1.0 - smoothstep(0.34, 0.74, vx.hypot(vy)) * 0.42
    }

    /// `aspectFactorF` (reference line 7627). Clamps on Y exactly as
    /// [`Self::aspect_factor`] does — `sample_arr`'s own clamp is the same
    /// rule, so this is the reference's one-liner unchanged.
    fn aspect_factor_f(&self, x: f64, y: f64) -> f64 {
        let dzdy = (self.h_f(x, y + 1.0) - self.h_f(x, y - 1.0)) * 0.5;
        if self.lat_at_f(y) >= 0.0 { -dzdy } else { dzdy }
    }

    /// `curvatureAtF` (reference line 7624). Never wraps, on either axis —
    /// matching [`Self::curvature_at`], and for the same reason it does.
    fn curvature_at_f(&self, x: f64, y: f64) -> f64 {
        self.h_f(x - 1.0, y) + self.h_f(x + 1.0, y) + self.h_f(x, y - 1.0) + self.h_f(x, y + 1.0) - 4.0 * self.h_f(x, y)
    }

    /// [`Self::grad_at`] at a fractional position — the hachure's downslope
    /// direction.
    ///
    /// This one cannot be written as four bare `sample_arr` calls the way
    /// `curvature_at_f` can: [`Self::grad_at`] **wraps X in world mode**
    /// and `sample_arr` clamps, so a straight transcription would disagree
    /// with the screen along the two vertical edges of a wrapping world.
    /// The wrap is applied to the coordinate before sampling instead, which
    /// is the same cell at every integer position.
    fn grad_at_f(&self, x: f64, y: f64) -> (f64, f64) {
        let (gw, gh) = (self.gw as f64, self.gh as f64);
        let (xl, xr) = if self.world {
            ((((x - 1.0) % gw) + gw) % gw, (x + 1.0) % gw)
        } else {
            ((x - 1.0).max(0.0), (x + 1.0).min(gw - 1.0))
        };
        let u = (y - 1.0).max(0.0);
        let d = (y + 1.0).min(gh - 1.0);
        ((self.h_f(xr, y) - self.h_f(xl, y)) * 0.5, (self.h_f(x, d) - self.h_f(x, u)) * 0.5)
    }

    /// [`Self::litho_at`] at a fractional position. Categorical, so the
    /// sample is nearest-neighbour after the same coherent jitter — the
    /// reference's own `lithB` bake read (`_geoLith[round(gy)*GW+
    /// round(gx)]`) is nearest for exactly this reason.
    fn litho_at_f(&self, x: f64, y: f64) -> Option<u8> {
        let lith = self.lithology?;
        if self.appearance.litho_strength <= 0.0 && self.appearance.litho_exposure <= 0.0 {
            return None;
        }
        let jx = (vnoise(x * 0.09, y * 0.09, 81) - 0.5) * 4.4;
        let jy = (vnoise(x * 0.09, y * 0.09, 83) - 0.5) * 4.4;
        let (gw, gh) = (self.gw as i64, self.gh as i64);
        let sx = (x + jx).round() as i64;
        let sy = (y + jy).round() as i64;
        let sx = if self.world { ((sx % gw) + gw) % gw } else { sx.clamp(0, gw - 1) };
        let sy = sy.clamp(0, gh - 1);
        Some(lith[(sy * gw + sx) as usize])
    }

    /// `_paintSampleAt` (reference line 4774) — the paint grids read
    /// nearest-neighbour, never bilinear: they hold **categorical palette
    /// indices**, and blending two of them would produce a meaningless
    /// third index. The reference's own comment says exactly this.
    fn paint_at_f(&self, x: f64, y: f64) -> PaintOverride {
        let ix = x.round().clamp(0.0, (self.gw - 1) as f64) as usize;
        let iy = y.round().clamp(0.0, (self.gh - 1) as f64) as usize;
        self.paint_at(iy * self.gw + ix)
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

/// The reference's per-lithology procedural microtexture `mt` (HTML 7812-7819)
/// — a signed multiplicative deviation applied to the rock's own colour, one
/// expression per rock type, transcribed constant for constant and seed for
/// seed.
///
/// | `lith` | rock | what the expression draws |
/// |---|---|---|
/// | 0 | granite | mineral speckle (`vnoise` at 420) plus fracture creases (a 4-octave ridged fbm) |
/// | 1 | basalt | broad lava-field patchiness |
/// | 2 | andesite | volcanic ash speckle, darkened where cinder accumulates |
/// | 3 | limestone | pale karst pitting — one-sided, it only ever darkens |
/// | 4 | sandstone | warm strata bands, **layered by elevation** (`r`), fbm-warped |
/// | 5 | shale | the same construction at 1.8× the band frequency and 0.6× the amplitude — thin dense strata |
/// | _ | metamorphic | folded gneiss banding, layered along `fx` rather than up `r` |
///
/// `r` (relative elevation) is a real input for sandstone and shale and not
/// merely a phase: strata are *horizontal*, so banding a cliff by height is
/// what makes it read as bedding planes rather than as a stripe pattern.
///
/// `fx`/`fy` are world coordinates divided by `gw`, which is the reference's
/// own `px/(GW||1)` — the same normalisation `land_color`'s surface-texture
/// stage uses, and the reason a tiled bake stays seamless (§8).
///
/// The `_` arm is metamorphic *and* the fallback for an index outside
/// `LITHO_PALETTE_ORDER`, matching [`litho_palette`]'s own defensive shape:
/// `build_lithology` cannot emit one, but a save could, and a panic here
/// crosses the gdext boundary (`cartalith-rust-conventions`).
pub(crate) fn litho_microtexture(lith: u8, fx: f64, fy: f64, r: f64) -> f64 {
    match lith {
        0 => (vnoise(fx * 420.0, fy * 420.0, 211) - 0.5) * 0.22 + (cartalith_noise::ridged_oct(fx * 40.0, fy * 40.0, 4, 213) - 0.5) * 0.12,
        1 => (fbm(fx * 26.0, fy * 26.0, 217) - 0.5) * 0.20,
        2 => (vnoise(fx * 300.0, fy * 300.0, 219) - 0.5) * 0.16 - smoothstep(0.6, 0.95, fbm(fx * 18.0, fy * 18.0, 221)) * 0.18,
        3 => -smoothstep(0.72, 0.95, vnoise(fx * 240.0, fy * 240.0, 223)) * 0.25,
        4 => (r * 90.0 + fbm(fx * 30.0, fy * 30.0, 227) * 6.0).sin() * 0.13,
        5 => (r * 160.0 + fbm(fx * 36.0, fy * 36.0, 229) * 4.0).sin() * 0.08,
        _ => (fx * 70.0 + fbm(fx * 12.0, fy * 12.0, 233) * 9.0).sin() * 0.12,
    }
}

/// The reference's rock-exposure gate for the geology block (HTML 7809-7810):
/// `clamp01(G^1.5·0.85 + smoothstep(0.5, 0.8, r)·0.45) · (1 - snow)`.
///
/// Two terms and a veto — steep ground sheds its cover, high ground has less
/// to shed, and an icecap hides whatever is underneath whatever the other two
/// say. Deliberately **not** [`TerrainAppearance::litho_exposure`]'s own
/// `steep · bare · thin · cover` formula: that one answers "how much soil is
/// there", which is the right question for a *colour*, and this one answers
/// "how much bare rock face is there", which is the right question for a
/// *texture*. Keeping the reference's own gate for the reference's own stage
/// is also what makes the two independently checkable.
///
/// `G^1.5` reuses the `slope/0.08` knee and the 1.5 exponent
/// [`rock_slope_mix`] already pins against independently-derived literals; it
/// is spelled out rather than calling that helper because the reference does
/// not fold the slider into this one (`gr2` has no `geoK` in it).
pub(crate) fn geo_exposure(slope: f64, r: f64, snow: f64) -> f64 {
    let gr2 = (slope / 0.08).min(1.0).powf(1.5);
    clamp01(gr2 * 0.85 + smoothstep(0.5, 0.8, r) * 0.45) * (1.0 - snow)
}

/// How far the tile's own meso-versus-macro shade difference brightens or
/// darkens glacier ice — LOD-D4 stage 3's *"slope- and flow-aligned brightness
/// term from the tile's own height"*.
///
/// **Both alignments are already in the two factors it multiplies**, which is
/// why no new geometry is derived for it: `sh_m - sh` is the meso shade minus
/// the macro shade, both taken from the *tile's* height at two sample steps,
/// so it is the local slope structure and nothing else; and it is scaled by
/// `ice`, which is [`build_glacier_potential`]'s catchment-weighted field, so
/// it is strongest down the line the ice flows and fades on the ground beside
/// it. On a smooth trough floor the two shades agree and the factor is `1`.
///
/// `0.6` against a difference that is empirically inside `±0.3` is a `±18%`
/// swing — enough for a tongue to read as ribbed rather than as a flat sheet,
/// short of the banding a `1.0` gives on a crevassed field.
const ICE_SHEEN: f64 = 0.6;

/// LOD-D4 stage 3, the fraction half: how much of this pixel's surface is
/// glacier ice, and the six material fractions rebalanced around it.
///
/// **The slope term is [`geo_exposure`]'s own `gr2`, reused rather than
/// re-invented.** The scope's line is *"rock exposure keeps
/// `geo_exposure(slope, r, snow)`, so steep faces above the snowline stay
/// rock"*, and the cheapest way to make that true is to take ice off exactly
/// the ground that term already calls bare: `gr2` is `min(1, slope/0.08)^1.5`,
/// the fraction of a face at this slope that is rock rather than cover, and
/// ice is what lies **on** ground. A separate ice-slope constant would have
/// been a second opinion about the same question, free to disagree with the
/// first — and disagreeing is exactly how the scope's second and third
/// acceptance bars (*"potential >= 0.5 renders as ice or snow"* against
/// *"slope > 0.08 is rock-dominant"*) would have been made to contradict each
/// other.
///
/// `Σ = 1` is preserved by construction and not by arithmetic luck: the five
/// non-snow fractions are scaled by `1 - ice` and `ice` is added to snow, so
/// the sum is `(1 - ice) * 1 + ice`.
///
/// Returns the rebalanced weights and the ice fraction itself, because the
/// colour stage needs the second number and cannot recover it from the first.
pub(crate) fn apply_ice_cover(mut w: Weights, glacier: f64, slope: f64) -> (Weights, f64) {
    let gr2 = (slope / 0.08).min(1.0).powf(1.5);
    let ice = clamp01(glacier) * (1.0 - gr2);
    if ice <= 0.0 {
        return (w, 0.0);
    }
    let keep = 1.0 - ice;
    w.rock *= keep;
    w.sand *= keep;
    w.wetland *= keep;
    w.canopy *= keep;
    w.grass *= keep;
    w.snow = w.snow * keep + ice;
    (w, ice)
}

/// [`snow_col`] with LOD-D4's ice tint and sheen folded in.
///
/// Where `ice` is zero this **is** [`snow_col`], returned from a dedicated
/// early branch rather than through a `lerp` by `0.0` — the same
/// identity-by-control-flow rule the rest of this file follows, and what makes
/// `js_reference()` (where `ice_strength` is `0.0`, so `ice` is always `0.0`)
/// bit-identical to the renderer before this milestone.
///
/// `pub(crate)` so `tests/lod_d4_ice_and_snow.rs`'s
/// `the_ice_sheen_constant_is_pinned` can pin [`ICE_SHEEN`] against an
/// independently-computed ratio. Mutation-tested 2026-09-21: without that
/// test, setting `ICE_SHEEN` to `0.0` SURVIVED the whole suite.
///
/// The tint target is [`TerrainAppearance::snow_glac`], the glacier-ice ramp
/// this appearance has carried since milestone 1 and which nothing but a
/// `t < -12` test had ever reached. That is the scope's own instruction
/// (*"snow takes `snow_glac`"*) and it means the ice palette is editable
/// through the same three stops as every other material, rather than through
/// a constant introduced here.
pub(crate) fn snow_material_col(a: &TerrainAppearance, t: f64, tt: f64, ice: f64, sh: f64, sh_m: f64) -> Rgb {
    let base = snow_col(a, t, tt);
    if ice <= 0.0 {
        return base;
    }
    let g = ramp3(&a.snow_glac, tt);
    let k = 1.0 + ICE_SHEEN * ice * (sh_m - sh);
    (
        (base.0 + (g.0 - base.0) * ice) * k,
        (base.1 + (g.1 - base.1) * ice) * k,
        (base.2 + (g.2 - base.2) * ice) * k,
    )
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
///
/// `pub(crate)` so `tests/lod_d4_ice_and_snow.rs` can assert that LOD-D4's
/// ice cover preserves that Σ rather than taking the two lines that do it on
/// trust — the same visibility, for the same reason, as [`geo_exposure`].
pub(crate) struct Weights {
    pub(crate) snow: f64,
    pub(crate) rock: f64,
    pub(crate) sand: f64,
    pub(crate) wetland: f64,
    pub(crate) canopy: f64,
    pub(crate) grass: f64,
    pub(crate) c: f64,
    pub(crate) meff: f64,
    pub(crate) is_mangrove: bool,
}

/// A tile pixel's facing toward the equator from the tile's own y-gradient
/// `gy` (coarse units, the one its `slope` was built from): -1 shaded .. +1
/// sun-facing, flipped south of the equator exactly as `aspect_factor` is.
pub(crate) fn tile_snow_facing(ctx: &RenderCtx, gy: f64, slope: f64, wy: f64) -> f64 {
    if slope <= 1e-9 {
        return 0.0;
    }
    let f = gy / slope;
    if ctx.lat_at_f(wy) >= 0.0 { -f } else { f }
}

/// Ruling AP's snow-aspect shift, in degrees C: `a.snow_aspect_c` times the
/// slope's `facing` toward the equator (-1 shaded .. +1 sun-facing), times
/// slope strength (`slope / 0.04`, as `material_weights` uses). `facing` must
/// come from the same gradient as `slope`: the LOD tile has its own
/// sub-cell slope, and pairing that with the coarse grid's facing left snow
/// uncorrelated with the tile's northness (LOD-D4 bar 1b, measured
/// 2026-09-24). `0.0` whenever the appearance has the term off.
pub(crate) fn snow_aspect_shift(a: &TerrainAppearance, facing: f64, slope: f64) -> f64 {
    if a.snow_aspect_c <= 0.0 || slope <= 1e-9 {
        return 0.0;
    }
    a.snow_aspect_c * facing.clamp(-1.0, 1.0) * (slope / 0.04).min(1.0)
}

pub(crate) fn material_weights(t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64, snow_shift_c: f64) -> Weights {
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

    // Ruling AP (2026-09-23): snow holds on shaded slopes and melts off
    // sun-facing ones -- `snow_shift_c` from [`snow_aspect_shift`], computed
    // by the caller from a facing at the SAME scale as its `slope`. A
    // branch, so `0.0` (the parity path) is the reference's
    // temperature-only term exactly.
    let t_snow = if snow_shift_c != 0.0 { t + snow_shift_c } else { t };
    let snow = smoothstep(3.0, -5.0, t_snow);
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
fn bio_jitter(x: f64, y: f64, gw: usize) -> f64 {
    let (xf, yf) = (x, y);
    let gw = gw as f64;
    0.6 * vnoise(xf / gw * 44.0, yf / gw * 44.0, 31) + 0.4 * vnoise(xf / gw * 150.0, yf / gw * 150.0, 33)
}

/// The R2 slope-material fraction (reference HTML 7789): how much of the rock
/// colour steep ground takes on, `clamp01(min(1, slope/0.08)^1.5 · k)`.
///
/// A named function rather than three lines inside [`land_color`] because
/// mutation-testing the whole render only ever proves that a stage *moved*
/// something: `powf(1.5) -> powf(1.0)` and the wetness tilt below both survived
/// a full-image battery, which is this project's recorded "a test that compares
/// a constant against itself pins nothing" failure in its other shape. Pulled
/// out, the exponent and the `0.08` knee are assertable against numbers derived
/// independently of them — `0.25^1.5` is exactly `0.125`, `0.5^1.5` is neither
/// `0.5` nor `0.25`.
///
/// `clamp01` after the multiply is the reference's own order (`_t = clamp01(gr)`
/// inside `if (gr > 0)`), and clamping cannot change the sign, so [`land_color`]
/// tests this value for `> 0` where the reference tests the unclamped one.
pub(crate) fn rock_slope_mix(slope: f64, k: f64) -> f64 {
    clamp01((slope / 0.08).min(1.0).powf(1.5) * k)
}

/// The R5 wetness multiply (reference HTML 7797-7798), whole: a common `dk`
/// darken from the topographic wetness index plus the fixed `0.95 / 1.00 /
/// 1.05` channel tilt that is the difference between *wet* and merely *dark*.
///
/// **The `wv > 0` guard is load-bearing and is the reference's own**: at `wv =
/// 0` the darken is `dk = 1`, but the tilt is not `1`, so dropping the guard
/// would cool every dry cell on the map by 5% while claiming to change nothing.
///
/// Written as one function taking the colour so the per-channel expressions keep
/// the reference's exact left-to-right association (`c · dk · 0.95`, not
/// `c · (dk · 0.95)`) — see `cartalith-rust-conventions` on reordering.
pub(crate) fn apply_wetness(c: Rgb, twi: f64, k: f64) -> Rgb {
    let wv = clamp01((twi + 1.0) / 4.0);
    // `k <= 0.0` is the reference's own outer `if (wtK > 0)`, folded in beside
    // the `wv` guard rather than left only at the call site. Same reason: at
    // `k = 0` the darken is `dk = 1` and the tilt is not, so a caller that
    // reached here with the stage switched off would cool the whole map by 5%
    // and no gate above would say so. [`land_color`] still tests it too, for
    // the dedicated-branch rule every other stage in this file follows.
    if wv <= 0.0 || k <= 0.0 {
        return c;
    }
    let dk = 1.0 - 0.30 * k * wv;
    (c.0 * dk * 0.95, c.1 * dk, c.2 * dk * 1.05)
}

// ===========================================================================
// Cel / toon shading ([`TerrainAppearance::toon_strength`],
// [`TerrainAppearance::toon_outline`])
// ===========================================================================

/// How many flat light steps [`toon_band`] cuts the hillshade into.
///
/// Provenance: the owner's row asks for *"3-4 steps"*; this port's own
/// judgement picks **4**, because three steps put the flat-ground level at the
/// top or the bottom of the ladder for most sun elevations (so one side of every
/// ridge had no step to go to), while four leave a step above flat ground for
/// sun-facing slopes and two below it for the shadowed side. `MAP_STYLE_
/// RESEARCH.md` does not cover toon shading, so there is no document value to
/// follow.
pub const TOON_BANDS: usize = 4;

/// Width of the softened terminator between two light steps, in **shade units**
/// (the hillshade's own `0..1` scale), centred on the edge.
///
/// Provenance: a judgement, stated in the units it acts in. `0.03` is 12% of a
/// band (`1 / TOON_BANDS = 0.25`): wide enough that a slope whose shade crosses
/// an edge over a single pixel still gets one intermediate value instead of a
/// stair-step alias, narrow enough that the edge reads as a hard terminator.
/// On the map, a gentle slope (shade changing ~0.01 per pixel) spreads it over
/// ~3 px; a steep one (~0.05 per pixel) collapses it under one pixel.
pub const TOON_EDGE: f64 = 0.03;

/// The light step containing shade value `s`, for a ladder anchored on `flat` —
/// the shade flat ground receives under the current light rig ([`toon_flat_shade`]).
///
/// **Anchored, not fixed at quarters.** Band edges sit at `flat ± band/2 +
/// k·band`, so flat ground is always the dead centre of a band. A fixed
/// quarter ladder put flat ground *on* an edge whenever the sun elevation made
/// `sin(alt)` a multiple of `0.25` (30° gives exactly `0.5`) and under the
/// multi-sun rig (flat ground `0.755`, a hair off the `0.75` edge): every
/// millimetre of relief on a plain then flickered between two steps, which is
/// the mottle this style exists to remove.
///
/// **Always exactly [`TOON_BANDS`] levels over the whole `0..1` range.** The
/// ladder is clamped to the lowest band centre strictly above `0` and the
/// highest at or below `1`; with `band = 1 / TOON_BANDS` that is
/// `(ceil(f/b) - 1) + floor((1-f)/b) + 1 = TOON_BANDS` for every `flat` in
/// `(0, 1]`. A partial band at either end joins its neighbour rather than
/// becoming a fifth, sliver-thin level.
///
/// The step itself is `smoothstep`-softened over [`TOON_EDGE`] around each
/// edge, so the value is continuous and monotone in `s`, and exactly flat
/// everywhere more than `TOON_EDGE / 2` from an edge.
///
/// Never returns a value outside `[0, 1]`.
pub fn toon_band(s: f64, flat: f64) -> f64 {
    let b = 1.0 / TOON_BANDS as f64;
    // Half the terminator width, in band units (the coordinate `fr` is in).
    let e = TOON_EDGE / (2.0 * b);
    // `t` counts bands from the one centred on `flat`; its integer part is the
    // band, its fraction the position inside it, edges at whole numbers.
    let t = (s - flat) / b + 0.5;
    let i = t.floor();
    let fr = t - i;
    // `i - 1` plus the two half-steps: the lower edge's upper half (`fr` just
    // above 0) and the upper edge's lower half (`fr` just below 1). Mid-band
    // both are 1 and 0, so the level is exactly `i`.
    let q = i - 1.0 + smoothstep(-e, e, fr) + smoothstep(1.0 - e, 1.0 + e, fr);
    // `max(0.0)` on the lower count: only reachable at `flat == 0` (every
    // detail weight zeroed), where `ceil(0) - 1` would otherwise put the lowest
    // level ABOVE flat ground.
    let lo = flat - b * ((flat / b).ceil() - 1.0).max(0.0);
    let hi = flat + b * ((1.0 - flat) / b).floor();
    // `f64::clamp` panics on `lo > hi` or a NaN bound, and a panic here would
    // cross the gdext boundary (`cartalith-rust-conventions`). Unreachable for
    // any finite `flat` in `[0, 1]`, which `toon_flat_shade` guarantees; a
    // non-finite one falls back to the unbanded shade rather than aborting.
    if !(lo <= hi) {
        return clamp01(s);
    }
    (flat + b * q).clamp(lo, hi).clamp(0.0, 1.0)
}

/// The combined shade flat ground receives — the anchor [`toon_band`] centres a
/// band on — for the light rig `a` describes and the three detail-band weights
/// `land_color` is blending with.
///
/// Derived, not measured per pixel: a flat normal is `(0, 0, 1)`, so the single
/// sun and every one of `build_lights`' evenly weighted directions (weights
/// normalised to 1, all at `sun_alt_deg`) give `sin(alt)`, and the multi-sun
/// rig gives its own [`multi_sun_from_normal`] of that normal. The multi-sun rig
/// replaces the **macro** shade only (`RenderCtx::shade`'s own `step == 1`
/// rule, and the tile renderer's `sh`), and the micro band is a zero-mean
/// jitter of the macro one, so both take the macro value; the meso band is
/// always single-sun.
pub fn toon_flat_shade(a: &TerrainAppearance, w_macro: f64, w_meso: f64, w_micro: f64) -> f64 {
    let single = a.sun_alt_deg.to_radians().sin();
    // Multi-sun is the one rig whose flat response is not `sin(alt)`: its two
    // suns sit at fixed 45°/35° and it adds a zenith light and an ambient floor.
    let macro_flat = if a.npr.multi_sun { multi_sun_from_normal(a, 0.0, 0.0, 1.0) } else { single };
    clamp01(w_macro * macro_flat + w_meso * single + w_micro * macro_flat)
}

/// The exponent [`toon_sharpen_weights`] raises each material weight to.
///
/// Provenance: a judgement, checked by arithmetic rather than taste alone.
/// Two materials at `0.6 / 0.4` (well inside a smooth hand-over) become
/// `0.6^8 / (0.6^8 + 0.4^8) = 0.9624` -- the dominant one's colour -- while an
/// exact `0.5 / 0.5` tie stays `0.5`, so the edge keeps a sub-pixel-to-few-pixel
/// soft seam instead of a hard aliased step. `2` left the airbrushed wash
/// visibly in place (`0.69` at `0.6 / 0.4`); `8` is the smallest power of two
/// that puts that pair above `0.95`.
pub const TOON_MATERIAL_POW: i32 = 8;

/// Sharpen `material_weights`' six blend weights toward their largest:
/// `w_i^P / Σ w_j^P` with `P =` [`TOON_MATERIAL_POW`]. The result sums to 1,
/// like its input, and keeps the input's ordering.
///
/// Never divides by zero: an all-zero input (unreachable -- `grass` takes
/// whatever budget the others leave) is returned unchanged rather than as NaN.
pub fn toon_sharpen_weights(w: [f64; 6]) -> [f64; 6] {
    let p = w.map(|v| v.max(0.0).powi(TOON_MATERIAL_POW));
    let sum: f64 = p.iter().sum();
    if sum <= 0.0 {
        return w;
    }
    p.map(|v| v / sum)
}

/// Radius of the toon outline, in **cells** on the screen and export paths and
/// in **tile pixels** on the deep-zoom path — so the keyline is a constant
/// width on screen at every zoom rather than a constant width of ground.
///
/// Provenance: a judgement. `2.0` draws a line about two units wide on the land
/// side: bold enough to read as a cartoon keyline at the fit zoom the owner
/// looks at (~1.4 screen px per cell), thin enough not to swallow a one-cell
/// isthmus. It must stay `<= 2`: the tile halo (`tile_halo_px`) is at least
/// the meso step, which is never below 2, and a wider disc would read past it
/// and seam at every tile edge.
pub const TOON_OUTLINE_R: f64 = 2.0;

/// The keyline's ink, `0..255` sRGB. Provenance: a judgement — a very dark
/// slate rather than pure black, which reads as a printed keyline over the
/// saturated toon palette instead of as a hole in it.
pub const TOON_INK: Rgb = (30.0, 34.0, 46.0);

/// How much toon-outline ink a **land** sample carries, `[0, 1]`, given a
/// predicate that says whether the sample `(dx, dy)` units away is water.
///
/// The coverage is the nearest water neighbour's distance `d` inside the disc
/// of radius [`TOON_OUTLINE_R`], mapped to `clamp01(R + 0.5 - d)`: `1` for an
/// edge-adjacent or diagonal neighbour, `0.5` at the rim. The half-coverage rim
/// is the anti-aliasing: a binary disc drew a stair-stepped line.
///
/// Why a neighbourhood test and not the relative elevation `r` the land
/// branch already has: `r` is height above **sea** level, so it says nothing
/// about a lake shore, and a band of `r` is wide on a gentle coast and
/// vanishing on a cliff. Asking the neighbours is the definition of an edge.
///
/// Never called on water: the outline is drawn on the land side only, so the
/// sea and lake colours are untouched.
pub fn toon_outline_cover(is_water: impl Fn(i64, i64) -> bool) -> f64 {
    let r = TOON_OUTLINE_R as i64;
    let mut best = f64::INFINITY;
    for dy in -r..=r {
        for dx in -r..=r {
            let d = ((dx * dx + dy * dy) as f64).sqrt();
            // `d == 0` is the sample itself (land, by the caller's contract);
            // `d > R` is outside the disc.
            if d == 0.0 || d > TOON_OUTLINE_R || d >= best {
                continue;
            }
            if is_water(dx, dy) {
                best = d;
            }
        }
    }
    if best.is_finite() { clamp01(TOON_OUTLINE_R + 0.5 - best) } else { 0.0 }
}

/// Blend `c` toward [`TOON_INK`] by `cover · strength`. `0` returns `c` itself.
pub fn apply_toon_outline(c: Rgb, cover: f64, strength: f64) -> Rgb {
    let k = clamp01(cover * strength);
    if k <= 0.0 {
        return c;
    }
    (c.0 + (TOON_INK.0 - c.0) * k, c.1 + (TOON_INK.1 - c.1) * k, c.2 + (TOON_INK.2 - c.2) * k)
}

/// `landColorCore`'s unconditional core (7720-7960): eco-jitter, the
/// six-material blend with canopy understory shadow, the beach rim, fine
/// noise grain, multi-scale hillshade, the `bioBlend` grey blend, the edge
/// haze fade, and the final `ao * vignette` multiply (7959-7960 — easy to
/// miss since it sits after the whole gated "Painter" NPR block, but is
/// itself unconditional). Every other `state.viz.*`-gated extra is omitted
/// — see this module's doc comment.
///
/// **This block sat above [`rock_slope_mix`] until 2026-09-03**, where it
/// documented the wrong function, and its parenthetical said `ao` was *"fixed
/// at `1.0` here, matching this port's AO/SVF/shadow fields all being off"*.
/// Both halves had stopped being true: `ao` is the caller's `ctx.ao[i]`, which
/// has carried [`build_ao`]'s cavity map since milestone 2 and now also carries
/// [`build_svf`] and [`build_sun_shadow`] folded in by [`fold_lighting_fields`].
/// `1.0` is what it is under `js_reference()` and at `default()`, which is a
/// statement about those two appearance records rather than about this port.
#[allow(clippy::too_many_arguments)]
fn land_color(appearance: &TerrainAppearance, t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64, sh: f64, sh_m: f64, vig: f64, ao: f64, eco_k: f64, hydro_wet: f64, lith: Option<u8>, grad: (f64, f64), x: f64, y: f64, gw: usize, gh: usize, splat: Option<&SplatTextures>, paint: PaintOverride, ground: GroundTiles, glacier: f64, scale: Option<DetailScale>, snow_facing: f64, river: Option<[f32; 4]>) -> Rgb {
    // CA-03/CA-04's one per-pixel test. At the default it selects the original
    // expressions at both composite sites below, so no blend-mode arithmetic
    // exists on the shipped path — see the section above [`RasterLayer`] for
    // why that identity has to be a branch and not a formula.
    let stack_default = appearance.layers.is_default();

    let n_low = vnoise(x * 0.06, y * 0.06, 11);
    let n_hi = vnoise(x * 96.0 / gw as f64, y * 96.0 / gw as f64, 23);
    let n_bio = bio_jitter(x, y, gw);

    // B4's ecotone widener (`eK`, reference 7763-7765). `1.0` is the
    // reference's own "off" and the value every caller passed before the
    // parameter existed, so it gets a **dedicated branch** rather than a
    // `* 1.0`: the reference groups the jitter as `T + eK·(a + b)` while this
    // port has always summed it as `(T + a) + b`, and those two are not the
    // same float. Multiplying unconditionally would have re-associated the
    // shipped and golden-pinned expression to buy nothing at the default —
    // identity by control flow, the rule this file follows everywhere else.
    // Cel / toon: the biome jitter fades with `toon_strength`. The jitter is
    // what makes a biome edge ragged, and ragged is right for a smooth blend
    // -- but once [`toon_sharpen_weights`] turns the blend into a crisp edge,
    // the jitter drags that edge back and forth wherever two materials are
    // near balance. Measured on `tests/cel_toon.rs`'s flat temperate fixture
    // (median 3x3 luma sd, default -> toon):
    //
    // * jitter kept whole: 5.38 -> 9.06 at 128 wide -- MORE mottle than the
    //   smooth map, as speckle;
    // * only its fine octaves faded (`bio_jitter`'s `vnoise(150/gw)` and
    //   `n_hi`), its broad `vnoise(44/gw)` and `n_low` kept: 1.86 -> 1.30 at
    //   512 wide -- the thresholded value-noise lattice shows as a checker of
    //   ~12-cell SQUARES, which is worse than speckle;
    // * faded whole (this arm): the edge follows the climate's own smooth
    //   contour, a clean toon shape.
    //
    // The known cost of the third, stated rather than hidden: an edge now
    // shows the climate raster exactly as it is, and where that raster has an
    // artefact the ragged blend used to hide, it shows too -- on seed 483920 a
    // ~200-cell dead-straight vertical forest edge at the west of the map
    // (`_stylepresets_probe.gd`'s Cel texture, 2026-09-27), which Default
    // draws as a blocky rectangle under its noise.
    //
    // First arm, so the two below -- `default()`'s path among them -- are
    // exactly the expressions they were.
    let (te, me, twi_e) = if appearance.toon_strength > 0.0 {
        let j = eco_k * (1.0 - clamp01(appearance.toon_strength));
        (
            t + j * ((n_bio - 0.5) * 7.0 + (n_low - 0.5) * 2.5),
            clamp01(m + j * ((n_bio - 0.5) * 0.15 + (n_hi - 0.5) * 0.05)),
            twi + j * ((n_bio - 0.5) * 0.7),
        )
    } else if eco_k == 1.0 {
        (
            t + (n_bio - 0.5) * 7.0 + (n_low - 0.5) * 2.5,
            clamp01(m + (n_bio - 0.5) * 0.15 + (n_hi - 0.5) * 0.05),
            twi + (n_bio - 0.5) * 0.7,
        )
    } else {
        (
            t + eco_k * ((n_bio - 0.5) * 7.0 + (n_low - 0.5) * 2.5),
            clamp01(m + eco_k * ((n_bio - 0.5) * 0.15 + (n_hi - 0.5) * 0.05)),
            twi + eco_k * ((n_bio - 0.5) * 0.7),
        )
    };
    let asp_e = asp * (1.0 + (n_low - 0.5) * 0.3);

    let w = material_weights(te, me, slope, r, twi_e, asp_e, curv, snow_aspect_shift(appearance, snow_facing, slope));
    let tt = clamp01(0.5 + (n_low - 0.5) * 1.1 + (n_hi - 0.5) * 0.5);
    // Cel / toon: flat albedo. `tt` is where each material sits on its own
    // three-stop ramp, driven by two noise octaves -- the within-material
    // mottle. Pulled toward the ramp's middle stop so a biome is one colour
    // per light step. A branch, not a `* (1 - 0)`, so the default path keeps
    // the exact `tt` it always had.
    let tt = if appearance.toon_strength > 0.0 { 0.5 + (tt - 0.5) * (1.0 - clamp01(appearance.toon_strength)) } else { tt };

    // `LOD_DETAIL_SCOPE.md` LOD-D4 stage 3. `glacier` is the caller's already
    // strength-scaled glacier potential — `0.0` at every grid-resolution call
    // site and under `js_reference()`, where this whole block is skipped by
    // the branch rather than evaluated to a no-op. See [`apply_ice_cover`] for
    // why the slope term is `geo_exposure`'s and not a new one.
    let (w, ice) = if glacier > 0.0 { apply_ice_cover(w, glacier, slope) } else { (w, 0.0) };

    let mut c = (0.0, 0.0, 0.0);
    let add = |c: &mut Rgb, m: Rgb, w: f64| {
        c.0 += m.0 * w;
        c.1 += m.1 * w;
        c.2 += m.2 * w;
    };
    // The snow colour is computed ONCE and reused by the splat path below,
    // because the two must be the same material: a pack-textured world
    // re-tints its snow channel by this exact colour (`splat_sample`'s own
    // argument), and a glacier that was ice in the flat blend and plain snow
    // under a pack would be one material with two identities.
    let snow_c = snow_material_col(appearance, te, tt, ice, sh, sh_m);
    add(&mut c, snow_c, w.snow);
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
                4 => (splat.snow, snow_c),
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
                splat_sample(tex, snow_c, w.snow, x, y, &mut acc, &mut cov);
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

    // Cel / toon: flat biome regions with crisp edges. `material_weights`
    // blends its six materials over deliberately wide smooth ramps (§30's "no
    // hard biome borders"), which under banded light still reads as a soft
    // airbrushed wash between grass and forest -- the opposite of a toon
    // fill. Each weight is sharpened by [`toon_sharpen_weights`], so a pixel
    // takes its dominant material's colour and the hand-over narrows to a thin
    // soft edge. After the splat block, so under full toon a loaded pack's
    // ground texture is flattened too: texture is what this style removes.
    if appearance.toon_strength > 0.0 {
        let k = clamp01(appearance.toon_strength);
        let ws = toon_sharpen_weights([w.snow, w.rock, w.sand, w.wetland, w.canopy, w.grass]);
        let cols = [
            snow_c,
            rock_material_col(appearance, te, me, r, tt, lith),
            sand_col(appearance, te, me, tt),
            wetland_col(appearance, te, w.is_mangrove, tt),
            forest_col(appearance, te, w.meff, tt),
            grass_col(appearance, te, me, r, tt),
        ];
        let mut flat = (0.0, 0.0, 0.0);
        for (wi, ci) in ws.iter().zip(cols.iter()) {
            add(&mut flat, *ci, *wi);
        }
        c = (c.0 + (flat.0 - c.0) * k, c.1 + (flat.1 - c.1) * k, c.2 + (flat.2 - c.2) * k);
    }

    // R2 slope-material refinement (reference HTML 7788-7790) — extra
    // `G^1.5`-weighted rock exposure on steep ground, as a tint *over* the
    // finished mix. The reference's own comment is the whole design: it
    // "leaves the Σ=1 materialWeights untouched", so nothing here can move the
    // golden-verified fraction blend above.
    //
    // Literal, constant for constant. Two substitutions, both this port's
    // established ones: `rock_material_col` for `rockCol` (the same
    // substitution the splat block's rock channel already makes — it *is*
    // `rock_col` whenever there is no lithology or `litho_strength` is `0.0`,
    // which is the reference path exactly, and using the bare heuristic here
    // would make the tint pull a lithology-tinted cell away from its own rock
    // colour), and `clamp01` for the reference's `clamp01`.
    //
    // The `r > 0.0` guard is the reference's, kept even though `land_color` is
    // already the land branch — the same "the reference tests it and so does
    // this" note `ridged_strength` below carries.
    if appearance.rock_slope > 0.0 && r > 0.0 {
        let k = rock_slope_mix(slope, appearance.rock_slope);
        if k > 0.0 {
            let mc = rock_material_col(appearance, te, me, r, tt, lith);
            c.0 += (mc.0 - c.0) * k;
            c.1 += (mc.1 - c.1) * k;
            c.2 += (mc.2 - c.2) * k;
        }
    }

    // R5 wetness (reference HTML 7796-7798) — darken and cool land where water
    // persistently accumulates. **Material space, before the light curve**,
    // which is the reference's own stated reason ("so lighting stays honest")
    // and the thing that separates it from this port's `hydro_wet` tint, which
    // is applied after the hillshade and the haze and is keyed on flow
    // accumulation rather than TWI.
    //
    // `twi`, not `twi_e`: the reference reads its own raw `twi` parameter
    // here, and `twiE` exists to jitter the *wetland material* edge in
    // `materialWeights`, which has already run. The `(twi + 1) / 4` calibration
    // is the reference's, with its comment: the wetland material itself ramps
    // over `twi ≈ -1 .. 2`.
    //
    // The three channel factors are literal — `0.95 / 1.00 / 1.05` is what
    // makes this cool rather than merely dark.
    if appearance.wetness > 0.0 && r > 0.0 {
        c = apply_wetness(c, twi, appearance.wetness);
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

    // R5 geology **microtexture** and dune ripples (reference HTML 7801-7832),
    // the two members the module doc's Excluded list carried until 2026-09-03.
    // Both are inside the reference's own single `geoK` gate, and both are
    // therefore driven by one `geo_micro` here — the reference's `geologyR` is
    // one slider.
    //
    // **The recolour that shares that gate in the reference is deliberately
    // not repeated.** Its `_r = mc[0]*(1+mt)` blends toward a flat per-rock
    // colour; this port already put a lithology colour on this pixel, twice
    // over (`rock_material_col`'s blend and the bedrock-exposure block just
    // above), from `litho_palette`'s seven editable three-stop ramps. A third
    // geology colour vocabulary would be two engines painting the same rock —
    // so what lands here is the reference's `(1 + mt)` *factor* over whatever
    // colour those stages produced, which is the texture and only the texture.
    // Stated, not silent: `CLAUDE.md`'s rule on deviating from a literal port.
    //
    // `r > 0.0` and the lithology test are the reference's own gates
    // (`geoK > 0 && r > 0 && px !== undefined`, with `lith !== undefined`
    // folded into `geoK` itself), so a loaded save — which carries no
    // tectonic substrate and therefore no lithology — takes neither branch.
    if appearance.geo_micro > 0.0
        && r > 0.0
        && let Some(li) = lith
    {
        let (fx, fy) = (x / gw as f64, y / gw as f64);
        let expo = geo_exposure(slope, r, w.snow);
        // `> 0.02`, the reference's own floor rather than `> 0.0`: below it
        // the blend is a fraction of a level and all it can add is dither.
        if expo > 0.02 {
            let m = 1.0 + (appearance.geo_micro * expo).min(0.85) * litho_microtexture(li, fx, fy, r);
            c.0 *= m;
            c.1 *= m;
            c.2 *= m;
        }
        // Dune ripples (7827-7832): wind banding on gentle sandy ground. The
        // ripple phase is in **raw grid coordinates** (`px*0.55 + py*0.25`),
        // not the `fx`/`fy` above — that is the reference's, and it is what
        // fixes the ripple wavelength at a few cells rather than letting it
        // scale with the map. Only the fbm warp is map-relative.
        //
        // `w.sand > 0.4 && slope < 0.03` is the reference's gate, so this
        // needs a hot arid coast or desert to act on at all; `synth()` in
        // `appearance_tiers.rs` is far too cold for `material_weights`' own
        // `smoothstep(17, 26, t)` sand term, which is why the dune branch has
        // its own fixture in `geology_micro_and_sky_fields.rs` rather than
        // relying on `every_tunable_is_load_bearing` to reach it.
        if w.sand > 0.4 && slope < 0.03 {
            let rip = (x * 0.55 + y * 0.25 + fbm(fx * 22.0, fy * 22.0, 235) * 8.0).sin() * 0.5 + 0.5;
            let dk = 1.0 - appearance.geo_micro * w.sand * 0.12 * rip;
            c.0 *= dk;
            c.1 *= dk;
            c.2 *= dk;
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
    //
    // **CA-03/CA-04 moved the *gate*, not the slot.** The sample is taken here
    // either way, because the composite below needs the identical value; what
    // the stack decides is *where it is folded in*. At the default it is
    // folded here, by the same three lines, which is what keeps the shipped
    // image byte-identical. Under any other stack the ramp becomes a real row
    // — orderable above or below the light, with its own operator — and it
    // therefore has to leave this mid-material slot, which is a visible change
    // and the point of the row rather than a side effect of it.
    let relief = if appearance.ramp_strength > 0.0 { appearance.ramp.sample(r) } else { None };
    if stack_default
        && let Some(rc) = relief
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
    // Cel / toon: the fine grain is texture noise too, faded with the same
    // strength as the `tt` flattening above. Branch for the same reason.
    let g = if appearance.toon_strength > 0.0 { g * (1.0 - clamp01(appearance.toon_strength)) } else { g };
    c.0 += g;
    c.1 += g;
    c.2 += g;

    // R3 procedural texture synthesis (reference HTML 7841-7851) — a
    // three-frequency 1:4:16 fbm stack modulating the material colour
    // multiplicatively, `C' = C·(1 + 0.2·k·(T - 0.5))`. Evaluated in grid
    // coordinates divided by `gw`, exactly as the reference divides its world
    // coordinates by `GW`, so a tiled bake stays seamless. Literal, constant
    // for constant; skipped at `0.0`.
    if appearance.tex_strength > 0.0 {
        let (fx, fy) = (x / gw as f64, y / gw as f64);
        let tt2 = 0.5 * fbm(fx * 16.0, fy * 16.0, 71) + 0.3 * fbm(fx * 64.0, fy * 64.0, 73) + 0.2 * fbm(fx * 256.0, fy * 256.0, 77);
        let m = 1.0 + 0.2 * appearance.tex_strength * (tt2 - 0.5);
        c.0 *= m;
        c.1 *= m;
        c.2 *= m;
    }

    // R4 ridged-noise elevation-weighted relief (reference HTML 7853-7862) —
    // folded creases lit and shaded from a five-octave ridged multifractal,
    // gated by `r²` so it concentrates in the highlands and never contaminates
    // the lowlands. Land only (`r > 0`), which is where `land_color` already
    // is, but the reference tests it and so does this.
    if appearance.ridged_strength > 0.0 && r > 0.0 {
        let rr = cartalith_noise::ridged_oct(x / gw as f64 * 96.0, y / gw as f64 * 96.0, 5, 53);
        let m2 = 1.0 + 0.5 * appearance.ridged_strength * (r * r) * (rr - 0.5);
        c.0 *= m2;
        c.1 *= m2;
        c.2 *= m2;
    }

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
            let (xf, yf) = (x, y);
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

    // Material chroma, before the light touches it. About the mix's own
    // luminance, so the material *ordering* — which is what
    // `material_weights` spent its whole blend establishing — is untouched;
    // only how far each material sits from grey moves. Skipped at `0.0`.
    if appearance.biome_sat != 0.0 {
        c = saturate(c, 1.0 + appearance.biome_sat);
    }

    // LOD-D5 stages 1 and 2. `None` is the pre-milestone expression by
    // control flow -- `cell_color` and `BakeFields::pixel` pass it and never
    // evaluate either curve, so the shipped screen and bake are byte-identical
    // whatever `detail_scale_strength` says. On the `Some` path the two
    // substitutions are, in order:
    //
    // 1. the three band weights come from `scaled_detail_weights` rather than
    //    from the appearance directly (they ARE the appearance's own three at
    //    grid resolution, bit for bit, which is what that function returns
    //    there);
    // 2. the micro band's `n` cross-fades from `n_hi` -- coherent value noise,
    //    which is grain and not relief -- toward the tile's own residual over
    //    the coarse field. At `micro_mix == 0` the `lerp` is `n_hi + 0.0`,
    //    which is `n_hi` exactly.
    //
    // The `(n - 0.5) * 0.20` fold and the `clamp01` are untouched: what
    // changes is what `n` is a measurement OF, not how it enters the light.
    let (w_macro, w_meso, w_micro, n_micro) = match scale {
        Some(s) => (s.weights.0, s.weights.1, s.weights.2, n_hi + (s.micro_n - n_hi) * s.micro_mix),
        None => (appearance.detail_macro_weight, appearance.detail_meso_weight, appearance.detail_micro_weight, n_hi),
    };
    let sh_micro = clamp01(sh + (n_micro - 0.5) * 0.20);
    let sh_combined = w_macro * sh + w_meso * sh_m + w_micro * sh_micro;
    let shade = clamp01(sh_combined);
    // Cel / toon: band the LIGHT, not the colour -- see
    // [`TerrainAppearance::toon_strength`]. Before the `0.85` light curve, so
    // the ambient floor and gain still shape the steps exactly as they shape a
    // smooth hillshade, and before every stage below that reads `light`
    // (relief chroma's warm-sun/cool-shadow split then bands with it). Skipped
    // entirely at `0.0`, so the default light is the expression it always was.
    let shade = if appearance.toon_strength > 0.0 {
        let k = clamp01(appearance.toon_strength);
        let banded = toon_band(shade, toon_flat_shade(appearance, w_macro, w_meso, w_micro));
        shade + (banded - shade) * k
    } else {
        shade
    };
    let light = appearance.relief_ambient + appearance.relief_gain * shade.powf(0.85);
    // The stack's second and last composite site. The default arm is the line
    // this file has always had; the other arm is the only place a blend mode
    // or a reordering can take effect. Everything below — bio_blend,
    // relief_chroma, curvature, atmosphere, haze, paint, NPR, `ao * vig` —
    // reads `l` and `light` exactly as before and is untouched by either arm.
    let mut l = if stack_default {
        (c.0 * light, c.1 * light, c.2 * light)
    } else {
        appearance.layers.composite(appearance, c, light, relief)
    };
    if appearance.bio_blend < 1.0 {
        // The reference's own grey is a *fixed* 185·light, so a `bio_blend`
        // below 1 pulls every pixel toward the same neutral — it costs chroma
        // and value together, which is why the shipped 0.90 reads as a faded
        // map rather than as a lit one. `relief_chroma` moves that target
        // toward a grey of **this pixel's own** luminance, at which point the
        // blend is exactly a desaturation and the relief structure survives it
        // intact. `0.0` is the reference, byte for byte.
        let grey = 185.0 * light;
        let (g0, g1, g2) = if appearance.relief_chroma > 0.0 {
            let y = luma(l);
            (lerp(grey, y, appearance.relief_chroma), lerp(grey, y, appearance.relief_chroma), lerp(grey, y, appearance.relief_chroma))
        } else {
            (grey, grey, grey)
        };
        l = (g0 + (l.0 - g0) * appearance.bio_blend, g1 + (l.1 - g1) * appearance.bio_blend, g2 + (l.2 - g2) * appearance.bio_blend);
    }
    if appearance.relief_chroma > 0.0 {
        // Shadow is lit by the sky and sun is lit by the sun: the two differ
        // in colour, not only in brightness. `s` is where this pixel sits on
        // the light curve, `-1` in full shadow and `+1` in full sun, derived
        // from the curve's own ambient/gain rather than from a magic constant
        // so a look that reshapes the curve keeps its own midpoint.
        let span = appearance.relief_gain.max(1e-6);
        let s = (clamp01((light - appearance.relief_ambient) / span) - 0.5) * 2.0;
        let k = appearance.relief_chroma;
        // Sun gains a little chroma, shadow loses a little — deliberately
        // small (±11%), because the point is that shadow stops being grey,
        // not that it becomes blue.
        l = saturate(l, 1.0 + k * 0.11 * s);
        l = temperature_shift(l, k * 0.30 * s);
    }

    // R5 curvature shading (reference HTML 7870-7876) — a sun-independent
    // cue straight from the Laplacian, brightening convex ridges and darkening
    // concave valleys, so a landform stays legible where it happens to run
    // parallel to the sun. Land only (`r > 0`), the reference's own gate.
    // Literal, including the `-curv*90` normalisation and the `0.28` fold.
    if appearance.curve_shade > 0.0 && r > 0.0 {
        let cc = (-curv * 90.0).clamp(-1.0, 1.0);
        let m = 1.0 + appearance.curve_shade * 0.28 * cc;
        l = (l.0 * m, l.1 * m, l.2 * m);
    }

    let dx = x / gw as f64 - 0.5;
    let dy = y / gh as f64 - 0.5;
    // Plate-relative distance from centre, 0 at the middle and 1 at the
    // corner -- the one "how far is this pixel" signal `haze_strength` has
    // always read. §19's other two axes share it rather than inventing a
    // second notion of "far".
    let dist = clamp01(dx.hypot(dy) * 1.9).powf(2.2);
    if appearance.atmo_contrast > 0.0 {
        // Same "pivot about 128" `grade_contrast` uses, but only ever the
        // contrast-losing branch -- distance never sharpens.
        let factor = (1.0 - appearance.atmo_contrast * dist * 0.75).max(0.1);
        l = (128.0 + (l.0 - 128.0) * factor, 128.0 + (l.1 - 128.0) * factor, 128.0 + (l.2 - 128.0) * factor);
    }
    if appearance.atmo_desaturation > 0.0 {
        l = saturate(l, 1.0 - appearance.atmo_desaturation * dist);
    }
    let haze = dist * appearance.haze_strength;
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
    // **The v1.28 refinement is live** (this comment said `_paintedTex` was
    // "not reachable in this port" until the `biomes`/`terrains` families
    // were decoded): when the loaded pack supplies a ground tile for the
    // painted index, that tile's **true colour** is blended instead of the
    // flat swatch — same 0.60 weight, same position in the pipeline, so
    // relief and hillshade keep showing through exactly as before. See
    // [`painted_tex`] for the sampler and [`GroundTile`] for why it carries
    // no inverse mean. Falling back to the swatch whenever no tile is loaded
    // is the reference's own `_t || CART_BIOME_COLS[pBio-1]`, and it is the
    // branch a pack-less world takes in both engines — which is what keeps
    // this port's default output byte-untouched.
    //
    // Water is deliberately untouched: this blend lives in `landColorCore`,
    // so `cell_color`'s sea branch never sees it. In the reference that is
    // moot (`_paintAt` is unconditionally land-gated), and in this port it
    // is the honest consequence of `Brush::land_only` being a toggle — a dab
    // placed on water with the gate off is stored, exported and previewed,
    // and simply has no place in the map's water colour.
    if !paint.is_empty() {
        // `_t || CART_*_COLS[p-1]`: the pack tile first, the flat swatch
        // second, nothing at all for an index outside the palette (where the
        // reference would read `undefined` and throw).
        let layer = |tiles: &[Option<GroundTile>], table: &[(u8, u8, u8)], v: u8| -> Option<Rgb> {
            painted_tex(tiles, v, x, y).or_else(|| {
                let c = if v == 0 { None } else { table.get(v as usize - 1).copied() }?;
                Some((c.0 as f64, c.1 as f64, c.2 as f64))
            })
        };
        for p in [layer(ground.biomes, &appearance.biome_cols, paint.bio), layer(ground.terrains, &CART_TERRAIN_COLS, paint.ter)].into_iter().flatten() {
            l.0 += (p.0 - l.0) * 0.60;
            l.1 += (p.1 - l.1) * 0.60;
            l.2 += (p.2 - l.2) * 0.60;
        }
    }

    // The "Painter" NPR block (7903-7962) — land only (`r > 0`), and exactly
    // here: after every colour and lighting step, before the final
    // `ao * vignette`. See `apply_npr`. Off at every default, and the whole
    // call is skipped rather than entered and no-opped.
    let npr_on = r > 0.0 && npr_any(&appearance.npr);
    let l = match river {
        // No river here -- the path every render without a `RiverLayer`
        // takes, JS goldens and exports included, unchanged.
        None => {
            if npr_on {
                apply_npr(appearance, l, r, slope, curv, grad, x, y, gw)
            } else {
                l
            }
        }
        // The river symbol ([`RiverLayer`]), in the paint brush's own slot
        // just above: after the light, before the Painter block, so the
        // hand-drawn styles act on it as on everything else the map draws
        // -- and before the `ao * vignette`, the paper, the frame, local
        // contrast and the grade, which all run after this point.
        // `river_through` below 1 blends toward the river laid over the
        // Painter styles instead (a Blueprint's white line, unhatched).
        Some(rv) => {
            let under = river_over(l, rv);
            if !npr_on {
                under
            } else {
                let through = apply_npr(appearance, under, r, slope, curv, grad, x, y, gw);
                if appearance.river_through >= 1.0 {
                    through
                } else {
                    let over = river_over(apply_npr(appearance, l, r, slope, curv, grad, x, y, gw), rv);
                    mix(over, through, appearance.river_through.clamp(0.0, 1.0))
                }
            }
        }
    };

    // Terrain occlusion is not applied to the water surface: `ao` is how
    // enclosed the GROUND is, and a river runs along exactly the valley
    // floors it darkens most, so the symbol would read darker the deeper its
    // valley. The vignette is the sheet's, and the river is on the sheet.
    let ao = match river {
        Some(rv) => ao + (1.0 - ao) * rv[3] as f64,
        None => ao,
    };

    // `ao * vignette` (7959-7960). `ao` was a hardcoded `1.0` before
    // milestone 2 (the reference's AO/SVF/shadow fields are all off at its
    // defaults); it now carries the reference's whole `aoC` product (8167) --
    // `build_ao`'s cavity map times `build_svf` and `build_sun_shadow`, folded
    // by `fold_lighting_fields` -- and is still exactly `1.0` under
    // `js_reference()`, where all three strengths are `0.0`.
    let k = ao * vig;
    (l.0 * k, l.1 * k, l.2 * k)
}

/// The reference's `state.viz` **"Painter" NPR block** (7903-7962) — ten
/// opt-in hand-drawn styles, each with its own intensity, applied in the
/// reference's own order: watercolor wash → contour veins → ink edges →
/// hachure → cel → engraving → stipple → sepia → risograph → pointillism —
/// plus an eleventh added 2026-09-21, v2.70's "Village map" flat
/// limited-palette style (see [`Npr::village`] and
/// [`quantize_flat_palette`]), a single on/off replacement rather than a
/// blended intensity and deliberately last in the chain.
///
/// **A literal per-pixel port, constant for constant, for the original ten.**
/// These are ordinary arithmetic on the finished land colour — no per-frame
/// state, no neighbourhood, no texture sampling — so there is nothing here a
/// shader would do differently, and porting them literally keeps them inside
/// the `rayon`-parallel `cell_color` pass milestone 6 already built rather
/// than adding a second compositing stage. The eleventh could not be checked
/// against source — see [`quantize_flat_palette`]'s doc.
///
/// Every style is skipped by its own `> 0.0` gate (or, for the eleventh, its
/// own `false`) exactly as the reference skips the first ten, and the whole
/// block is skipped on water (`r > 0.0`) and when nothing at all is on — so
/// `TerrainAppearance::default()` and `js_reference()` are both bit-untouched.
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
    x: f64,
    y: f64,
    gw: usize,
) -> Rgb {
    let n = &a.npr;
    let (mut l0, mut l1, mut l2) = l;
    let (px, py) = (x, y);
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

    // D-cel: posterize the lit colour into flat bands (the reference's own,
    // colour not light -- the "Cel / Toon" preset uses `toon_strength`).
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

    // D-village ("Village map", `RC_ENGINE_CHANGES.md` v2.70): the land half
    // of the flat limited-palette style — see [`Npr::village`]'s own doc and
    // [`quantize_flat_palette`]. Deliberately last in this chain (the row's
    // own text: "one more step in the existing per-pixel style chain"), so it
    // quantises whatever every earlier Painter style produced rather than
    // being undone by one running after it. A REPLACEMENT, not a blend by
    // intensity — the flag is a single on/off switch, not a slider.
    if n.village {
        let (q0, q1, q2) = quantize_flat_palette((l0, l1, l2));
        l0 = q0;
        l1 = q1;
        l2 = q2;
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
        || n.village
}

/// The v2.70 "Village map" style's shared quantiser (`RC_ENGINE_CHANGES.md`'s
/// v2.70 row) — a flat, limited-palette look applied as a REPLACEMENT over an
/// already-fully-lit/shaded colour, never over the raw material mix. Shared
/// between [`apply_npr`]'s land step and [`sea_color_core`]'s water step so
/// the two halves of one map style cannot drift onto two different band
/// counts.
///
/// **The exact reference constant could not be checked against source.**
/// `Cartalith_RC` (`hash_gen1.js`'s `landColorCore`/`seaColorCore`) is not
/// present on this machine — `RC_ENGINE_CHANGES.md`'s v2.70 row gives the
/// *shape* of the change (quantise the lit colour; water is a replacement,
/// not a mix-in; turn the hillshade off in the recipe) but not this constant.
/// Both this port's original choice and its replacement are its own.
///
/// **`BANDS = 3.0` (four flat levels per channel) shipped and the owner
/// reported it harsh** ("yellow/purple/teal"; `OUTSTANDING_WORK.md` §2.11).
/// **Ruling AZ** (`LARGE_ITEM_RULINGS.md`, 2026-09-28): *"Village map style.
/// Softer: more colour bands, matching the other presets."* Re-baselined to
/// **`BANDS = 5.0`** (six flat levels per channel — finer than D-cel's own
/// four divisions, `apply_npr`'s `n.cel` step above, rather than coarser as
/// the original choice was) by measurement, not taste alone: the
/// harsh-transition metric (the fraction of horizontally/vertically adjacent
/// pixel pairs whose Euclidean RGB distance exceeds 90 of 441.7, on a fixed
/// seed 20260927, 512×328, sRGB, windowed —
/// `godot-project/_villagebands_probe.gd`) put Village at **0.044551** with
/// `BANDS = 3` against the other six `STYLE_PRESETS` entries' own range
/// **[0.029251, 0.032794]** on the same world; at `BANDS = 5` Village is
/// **0.031397**, inside that range (`BANDS = 7` gives 0.029340, also inside
/// but not the smallest count that clears it). The other six presets'
/// figures do not move with `BANDS`, since `quantize_flat_palette` only ever
/// runs when [`Npr::village`] is set and no other preset sets it (checked by
/// grep over `STYLE_PRESETS`) — so `BANDS` did not need to become a
/// per-preset parameter to keep this a Village-only change.
///
/// `js_round`, not `f64::round`, for the same reason [`apply_npr`]'s own
/// rounding sites use it: a channel can sit fractionally below an exact band
/// edge after earlier blends, and JS rounds half toward `+∞` where Rust
/// rounds half away from zero (`cartalith-rust-conventions`).
fn quantize_flat_palette(c: Rgb) -> Rgb {
    const BANDS: f64 = 5.0;
    let q = |v: f64| (cartalith_jsmath::js_round((v / 255.0 * BANDS).clamp(0.0, BANDS)) / BANDS * 255.0).clamp(0.0, 255.0);
    (q(c.0), q(c.1), q(c.2))
}

/// The sea's own noise sample — the `nLow` argument `seaColor` (v2.10 8280)
/// and the tile renderer `renderBiomeTileRGBA` (v2.10 11684, 11718) both
/// build as `vnoise(x·25.6/GW, y·25.6/GW, 5)`, factored out so the paths
/// cannot disagree.
///
/// **At `sea_grain_warp == 0.0` this is that expression and nothing else**,
/// returned from a dedicated branch — the `ColorSpace::Srgb => return` rule,
/// not an arithmetic identity that a float reassociation could break. That
/// is the `js_reference()` path.
///
/// Above zero — the shipped look, `1.0` since Ruling AS — it fixes
/// `TERRAIN_APPEARANCE_SCOPE.md` milestone 6's recorded ocean-lattice
/// artefact (see [`TerrainAppearance::sea_grain_warp`]). The treatment is
/// milestone 4's own stipple fix reused — rotate the sampling lattice off the axes so its
/// iso-contours stop forming a rectangular quilt, and domain-warp it with a
/// second coherent field so the remaining lattice period stops being one
/// period. The `(0.8290, 0.5592)` rotation is the stipple's own `(cos, sin)`
/// pair (~34°, far from every multiple of 45°, so no axis- *or*
/// diagonal-aligned seam survives) — copied rather than shared, because
/// hoisting it would edit a stage that is `0.20` in the shipped default and
/// this row is not allowed to move that image.
///
/// The warp is scaled by `sea_grain_warp` so the slider sweeps continuously
/// from the reference lattice to the fully broken-up one; the rotation is not,
/// because a partly-rotated lattice is still a lattice.
fn sea_grain(a: &TerrainAppearance, x: f64, y: f64, gw: usize) -> f64 {
    let (u, v) = (x * 25.6 / gw as f64, y * 25.6 / gw as f64);
    if a.sea_grain_warp <= 0.0 {
        return vnoise(u, v, 5);
    }
    let (ru, rv) = (u * 0.8290 + v * 0.5592, -u * 0.5592 + v * 0.8290);
    // Half a lattice cell of warp at full strength: enough that no straight
    // seam survives, not so much that the grain stops reading as one field.
    let k = a.sea_grain_warp * 0.5;
    let wu = ru + (vnoise(ru * 2.0, rv * 2.0, 41) - 0.5) * k;
    let wv = rv + (vnoise(rv * 2.0, ru * 2.0, 43) - 0.5) * k;
    vnoise(wu, wv, 5)
}

/// The v1.05 lake surface (reference 11741-11742): `seaColorCore(0.30, T,
/// grain, 0.95, vig)` re-tinted toward fresh water with the reference's own
/// six literals. Opaque and unshaded (`sh` fixed at `0.95`). Lakes only:
/// rivers were drawn in it from 2026-09-22 until the owner's 2026-09-23
/// ruling moved them to a Strahler-order palette (`lib.rs`'s
/// `RIVER_ORDER_RGB`).
fn lake_color(a: &TerrainAppearance, t: f64, n_low: f64, vig: f64) -> Rgb {
    let lc = sea_color_core(a, 0.30, t, n_low, 0.95, vig);
    ((lc.0 * 0.9 + 12.0).min(255.0), (lc.1 * 0.96 + 16.0).min(255.0), (lc.2 * 0.94 + 6.0).min(255.0))
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
    // Ruling BI's "Nautical" preset -- see `sea_ramp_strength`'s own doc.
    // `<= 0.0` early-returns exactly like every other stage in this function,
    // so the default and `js_reference()` path never evaluate `mix` at all.
    if appearance.sea_ramp_strength > 0.0 {
        let ramp_c = sample_ramp_table(SEA_RAMP_NAUTICAL, depth);
        wc = mix(wc, ramp_c, appearance.sea_ramp_strength);
    }
    let tex = (n_low - 0.5) * 5.0;
    let sh2 = 0.82 + 0.18 * clamp01(sh);
    let out = ((wc.0 + tex) * sh2 * vig, (wc.1 + tex) * sh2 * vig, (wc.2 + tex) * sh2 * vig);
    // v2.70 "Village map" (`RC_ENGINE_CHANGES.md`): the water half of the
    // flat limited-palette style, applied at the very end of this function —
    // the row's own instruction, because by this line the seabed grain
    // (`tex`), the bathymetric hillshade (`sh2`) and the smooth depth ramp
    // are all already folded into `out`. **A REPLACEMENT, not another mix
    // into the colour** — the row's own distinction from every ramp/tint
    // stage above, which all blend toward a target. See [`Npr::village`]'s
    // doc and the land half in [`apply_npr`].
    if appearance.npr.village {
        quantize_flat_palette(out)
    } else {
        out
    }
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
fn paper_tone(a: &TerrainAppearance, x: f64, y: f64, gw: usize) -> Rgb {
    if a.paper_strength <= 0.0 {
        return (1.0, 1.0, 1.0);
    }
    let (xf, yf, gwf) = (x, y, gw as f64);
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
    border_cover_f(a, x as f64, y as f64, gw, gh)
}

/// [`border_cover`] at a **fractional** sample position — the export
/// raster's own caller (`bake_pixel`), whose `gx`/`gy` land between cells.
///
/// The integer entry point above delegates here rather than the other way
/// round, so there is exactly one frame-geometry expression: at an integer
/// `x`, `x as f64` and `(gw - 1 - x) as f64` are the same doubles the
/// `usize` form computed (every value is a small exact integer), which is
/// what keeps `golden_parity_render.rs` and the live map byte-identical
/// across this change.
///
/// `f64::min` rather than [`crate::render::js_min`]-style NaN propagation:
/// both coordinates are clamped into `[0, gw-1]`/`[0, gh-1]` by
/// [`BakeFields::pixel`] before they reach here, so NaN is unreachable and
/// the two spellings cannot differ.
fn border_cover_f(a: &TerrainAppearance, x: f64, y: f64, gw: usize, gh: usize) -> f64 {
    let w = border_width_cells(a, gw, gh);
    if w <= 0.0 {
        return 0.0;
    }
    let d = x.min((gw - 1) as f64 - x).min(y.min((gh - 1) as f64 - y));
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
fn apply_border(a: &TerrainAppearance, c: Rgb, tone: Rgb, x: f64, y: f64, gw: usize, gh: usize) -> Rgb {
    let w = border_width_cells(a, gw, gh);
    if w <= 0.0 {
        return c;
    }
    let dx = x.min((gw - 1) as f64 - x);
    let dy = y.min((gh - 1) as f64 - y);
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
        let along = if dx < dy { y } else { x };
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
    apply_local_contrast_rows(a, rgb, gw, gh, 0, gh, world);
}

/// The local-contrast radius, in rows/columns of the **whole** `gw × gh`
/// raster. Hoisted out of [`apply_local_contrast_rows`] because the banded
/// export needs it before any band exists: it is [`ExportBandPlan`]'s apron.
/// See the comment at its use below for the floor and the short-axis cap.
fn local_contrast_radius(a: &TerrainAppearance, gw: usize, gh: usize) -> i64 {
    ((gw as f64 * a.local_contrast_radius_frac).round() as i64).max(3).min((gh as i64 / 4).max(3))
}

/// [`apply_local_contrast`] over a horizontal band: `rgb` holds rows
/// `y0 .. y0 + rows` of a `gw × gh` raster (`EXPORT_SCOPE.md` §4, milestone
/// E1). The whole-raster call is exactly `y0 = 0, rows = gh`.
///
/// Two things are keyed to the **full** raster and never to the band, and
/// both are §4.2's named traps: the radius (a band deriving its own would
/// boost differently band by band, and every seam would show as a step) and
/// the plate-frame fade (`border_cover` gets the image-space row and the full
/// height, or a neatline is drawn across every band boundary).
///
/// The blur itself runs over the band alone, clamping at the band's edges.
/// That equals the whole-raster blur on every row at least `radius` rows from
/// a band edge that is not an image edge — which is why a band carries that
/// many apron rows each side ([`ExportBandPlan`]) and discards them after.
#[allow(dead_code)]
pub fn apply_local_contrast_rows(a: &TerrainAppearance, rgb: &mut [u8], gw: usize, gh: usize, y0: usize, rows: usize, world: bool) {
    if let Some(lc) = local_contrast_rows(a, rgb, gw, gh, y0, rows, world) {
        let n = gw * rows;
        finish_rgb(&mut rgb[..n * 3], Some(|i| lc.delta(i)), None, ColorSpace::Srgb);
    }
}

/// Local contrast's **measurement** half: the two blurred luma bands of a
/// finished raster, from which [`LocalContrast::delta`] gives each pixel's
/// correction. Split from the **application** half (2026-09-23, owner ruling on
/// `OUTSTANDING_WORK.md` §20 — see [`finish_rgb`]) so the correction can be
/// added to a pixel that is still continuous instead of being truncated to a
/// byte before the grade reads it. `None` exactly where the pass is a no-op.
pub struct LocalContrast<'a> {
    a: &'a TerrainAppearance,
    fine: Vec<f32>,
    blurred: Vec<f32>,
    inv_knee2: f64,
    gw: usize,
    gh: usize,
    y0: usize,
}

impl LocalContrast<'_> {
    /// The additive correction for pixel `i` of the measured rows, faded out
    /// under the plate frame. `0.0` for an untouched pixel.
    pub fn delta(&self, i: usize) -> f64 {
        let a = self.a;
        let d = self.fine[i] as f64 - self.blurred[i] as f64;
        let mut delta = a.local_contrast * d * (-(d * d) * self.inv_knee2).exp();
        if delta == 0.0 {
            return 0.0;
        }
        let cover = border_cover(a, i % self.gw, self.y0 + i / self.gw, self.gw, self.gh);
        if cover > 0.0 {
            delta *= 1.0 - cover;
        }
        delta
    }
}

/// [`apply_local_contrast_rows`]'s measurement, without applying it.
#[allow(dead_code)]
pub fn local_contrast_rows<'a>(a: &'a TerrainAppearance, rgb: &[u8], gw: usize, gh: usize, y0: usize, rows: usize, world: bool) -> Option<LocalContrast<'a>> {
    if a.local_contrast <= 0.0 || gw == 0 || gh == 0 || rows == 0 {
        return None;
    }
    let n = gw * rows;
    if rgb.len() < n * 3 {
        return None;
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
    // detail band into global contrast. Both bounds are the FULL raster's,
    // so a band gets the whole image's radius, never its own.
    let rad = local_contrast_radius(a, gw, gh);

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
    let (fine, blurred) = rayon::join(|| blur_once(&luma, gw, rows, r_inner, world), || blur_once(&luma, gw, rows, rad, world));

    let knee = a.local_contrast_knee.max(1e-3);
    Some(LocalContrast { a, fine, blurred, inv_knee2: 1.0 / (knee * knee), gw, gh, y0 })
}

/// The whole finishing chain — local contrast, the colour grade, the output
/// colour space — over a finished `gw × gh` raster, quantised **once**.
/// What `build_color_texture` and every whole-raster export run.
#[allow(dead_code)]
pub fn finish_raster(a: &TerrainAppearance, rgb: &mut [u8], gw: usize, gh: usize, world: bool, influence: &[f32], space: ColorSpace) {
    let lc = local_contrast_rows(a, rgb, gw, gh, 0, gh, world);
    finish_rgb(rgb, lc.as_ref().map(|l| |i| l.delta(i)), Some((a, influence)), space);
}

/// The three whole-raster correction stages fused into **one** pass with one
/// quantisation (owner ruling, 2026-09-23, `LARGE_ITEM_RULINGS.md` Ruling AN —
/// build toward HDR/wide-gamut output, staged).
///
/// Until that ruling each stage was its own `u8 → u8` pass, so a pixel was
/// truncated to a byte after local contrast, read back and truncated again
/// after the grade, then decoded from that byte and rounded again by the
/// colour space. Here the pixel stays `f64` from the byte `cell_color`'s
/// caller wrote to the byte that leaves this function: `c + delta` (local
/// contrast), then the grade, then the display re-encode, then one quantiser.
/// That single quantiser is the seam a higher-precision output (a 16-bit or
/// float encoder) plugs into later; nothing downstream of it exists yet —
/// Godot's texture is `RGB8` and both export encoders are 8-bit.
///
/// **Why fused per pixel, not an `f32` buffer between passes.** The staged
/// design named a widened intermediate buffer. Every one of these stages is a
/// function of its own pixel once local contrast's blurs exist, so a buffer
/// buys nothing a register does not — and it would cost 12 B/px on top of
/// `export_raster.rs`'s *measured* 23 B/px peak gate, invalidating the
/// measurement that keeps a 32K export from aborting the process.
///
/// **What is kept, deliberately.** Each stage still clamps to `0..=255`
/// (range, not precision: the grade's pivots and tints are written against
/// that range, and dropping the clamp is the HDR step, not this one). The
/// quantiser is the one the last stage already used: truncation, the
/// convention of every computing stage in this file, unless the colour space
/// re-encoded the pixel, which rounds for [`apply_color_space`]'s documented
/// reason. So a chain that runs only one stage — the default render, where
/// the grade is at rest and the space is sRGB — is **byte-identical** to the
/// old pass by construction, and the three `apply_*` functions are this
/// function with two stages switched off.
///
/// `delta` is the per-pixel local-contrast correction (`None` = off); `grade`
/// is the appearance and [`build_grade_influence`]'s buffer (`None` = off).
/// `rayon`-parallel per pixel: each output byte is a pure function of its own
/// three input bytes and its own index.
#[allow(dead_code)]
pub fn finish_rgb<F: Fn(usize) -> f64 + Sync>(rgb: &mut [u8], delta: Option<F>, grade: Option<(&TerrainAppearance, &[f32])>, space: ColorSpace) {
    let grade = grade.filter(|(a, _)| !a.grade_is_identity());
    let matrix = match space {
        ColorSpace::Srgb => None,
        ColorSpace::DisplayP3 => Some(&SRGB_TO_P3),
    };
    if delta.is_none() && grade.is_none() && matrix.is_none() {
        return;
    }
    let weighted = grade.is_some_and(|(_, inf)| !inf.is_empty() && inf.len() == rgb.len() / 3);
    // Hoisted for the flat grade: the per-pixel path costs a division for
    // `contrast` that there is no reason to pay a million times for a constant.
    let flat = grade.map(|(a, _)| GradeAxes::at(a, 1.0));
    rgb.par_chunks_mut(3).enumerate().for_each(|(i, px)| {
        if px.len() < 3 {
            return;
        }
        let d = delta.as_ref().map_or(0.0, |f| f(i));
        let g = match (grade, flat) {
            (Some((a, inf)), Some(flat)) => Some(if weighted { GradeAxes::at(a, inf[i] as f64) } else { flat }),
            _ => None,
        };
        let c = finish_px((px[0] as f64, px[1] as f64, px[2] as f64), d, g.as_ref(), matrix);
        // The one quantiser (see the doc above for why it truncates unless the
        // space re-encoded).
        let q = |v: f64| if matrix.is_some() { v.round().clamp(0.0, 255.0) as u8 } else { v as u8 };
        px[0] = q(c.0);
        px[1] = q(c.1);
        px[2] = q(c.2);
    });
}

/// One pixel through [`finish_rgb`]'s three stages, **before** its quantiser:
/// `0..=255`-scale and continuous.
fn finish_px(mut c: Rgb, d: f64, g: Option<&GradeAxes>, m: Option<&[[f64; 3]; 3]>) -> Rgb {
    if d != 0.0 {
        c = ((c.0 + d).clamp(0.0, 255.0), (c.1 + d).clamp(0.0, 255.0), (c.2 + d).clamp(0.0, 255.0));
    }
    if let Some(g) = g {
        let o = grade_pixel(g, c);
        c = (o.0.clamp(0.0, 255.0), o.1.clamp(0.0, 255.0), o.2.clamp(0.0, 255.0));
    }
    if let Some(m) = m {
        let (r, g, b) = (srgb_eotf(c.0 / 255.0), srgb_eotf(c.1 / 255.0), srgb_eotf(c.2 / 255.0));
        let enc = |row: &[f64; 3]| srgb_oetf(row[0] * r + row[1] * g + row[2] * b) * 255.0;
        c = (enc(&m[0]), enc(&m[1]), enc(&m[2]));
    }
    c
}

/// [`finish_px`] for a caller outside this file: the unquantised finished
/// value of one pixel, given its local-contrast `delta` and its grade
/// influence multiplier `m` (`1.0` = the flat grade). `pub` for
/// `tests/color_space.rs`, which measures both quantised pipelines against it
/// — the continuous value is the only honest answer to "which is closer".
#[allow(dead_code)]
pub fn finish_pixel_continuous(c: Rgb, delta: f64, grade: Option<(&TerrainAppearance, f64)>, space: ColorSpace) -> Rgb {
    let g = grade.filter(|(a, _)| !a.grade_is_identity()).map(|(a, m)| GradeAxes::at(a, m));
    let m = match space {
        ColorSpace::Srgb => None,
        ColorSpace::DisplayP3 => Some(&SRGB_TO_P3),
    };
    finish_px(c, delta, g.as_ref(), m)
}

// ===========================================================================
// The colour grade (2026-08-24) — a genuinely new pipeline stage
// ===========================================================================
//
// The reference has no counterpart. Every other stage in this file answers
// "what is this piece of ground made of, and how is it lit"; this one answers
// "how is the finished sheet printed", which is a different question and
// belongs at a different place in the pipeline.
//
// # Where it runs, and why there
//
// On the **finished raster**, after [`apply_local_contrast`] and before the
// Godot overlays draw labels, settlement icons, territory and the
// scale bar (`map_overlay.gd` and its siblings, which composite over the
// `ImageTexture` this raster becomes). The rivers are in the raster since
// 2026-09-27, so the grade reaches them -- which is the point. That is the reference's own ordering
// intent read into this port's split: the grade is a statement about the
// *terrain image*, and grading the vector furniture on top of it would move
// a label's ink and a route's colour along with the ground, which is not
// what a grade is for.
//
// # Presentation only, structurally
//
// It receives an RGB8 buffer and the appearance, and nothing else. There is
// no path from here to the heightmap, climate, geology, hydrology or the
// seed — not by convention but by what it is handed. Nothing here marks a
// generation stage stale, and re-grading is a re-render of the texture over
// the world that is already there.
//
// # Why one pass rather than seven
//
// Exposure, gamma, contrast, temperature, the two tints and saturation
// compose into a single read-modify-write per pixel. Seven passes over a
// 2048² buffer would be seven times the memory traffic for arithmetic that
// costs less than the traffic does.
//
// # The four field-influence weights
//
// The one thing in this stage that is *not* purely a function of the pixel.
// [`build_grade_influence`] turns the four weights into one `f32` multiplier
// per output pixel, built once from the grid fields before the pass runs;
// [`apply_color_grade`] then scales every axis' departure from rest by it. The
// buffer is **empty** whenever all four weights are `0.0`, and the pass reads
// its length rather than a flag — `coast_d`/`crest`'s own contract in
// [`RenderCtx`], for the same reason: a gate and a buffer that can disagree
// eventually do.
//
// Multiplying by an exactly-`1.0` multiplier is exact in IEEE-754, so the
// no-weight path is bit-identical to the six-axis grade that shipped first
// rather than merely close to it.

impl TerrainAppearance {
    /// Whether the grade is the identity — every parameter at rest. The pass
    /// early-returns on this rather than evaluating seven no-ops per pixel,
    /// the same gate rule every stage in this file follows.
    ///
    /// The four field weights are deliberately **not** part of this test: a
    /// weight scales the axes above and scaling nothing is still nothing, so a
    /// grade with weights but no axes is the identity and must early-return
    /// like one.
    #[allow(dead_code)]
    pub fn grade_is_identity(&self) -> bool {
        self.grade_exposure == 0.0
            && self.grade_contrast == 0.0
            && self.grade_saturation == 0.0
            && self.grade_temperature == 0.0
            && self.grade_shadow_tint == 0.0
            && self.grade_highlight_tint == 0.0
            && self.grade_gamma == 0.0
    }

    /// Whether any field-influence weight is set. Only [`build_grade_influence`]
    /// needs it, and it is the gate that keeps the whole builder off the
    /// default path.
    #[allow(dead_code)]
    pub fn grade_influence_is_flat(&self) -> bool {
        self.grade_field_biome == 0.0 && self.grade_field_elevation == 0.0 && self.grade_field_moisture == 0.0 && self.grade_field_geology == 0.0
    }
}

/// Standing vegetation cover per `cartalith_civ::BIOME_KEYS` index (`0` =
/// ocean, `13` = lake), `0..1`.
///
/// **This table is this port's own**, and it is the one part of the
/// field-influence feature the design did not specify. Biome is a *category*,
/// and a weight needs a scalar; of the properties every one of the thirteen
/// categories has, standing vegetation cover is the one a colourist means by
/// "let the grade lean on biome" — it is what puts chroma on a map. The
/// ordering (bare ice and desert at the bottom, closed temperate and tropical
/// rainforest at the top) is the only claim being made, not the exact figures.
#[allow(dead_code)]
pub const BIOME_VEGETATION_COVER: [f64; 14] = [
    0.00, // ocean
    0.00, // ice
    0.15, // tundra
    0.75, // boreal
    0.80, // conifer
    0.90, // temperate forest
    1.00, // temperate rainforest
    0.40, // grassland
    0.30, // shrubland
    0.05, // desert
    0.50, // savanna
    0.70, // tropical dry forest
    1.00, // tropical rainforest
    0.00, // lake
];

/// One `f32` multiplier per output pixel for [`apply_color_grade`], or an
/// **empty** `Vec` when every field-influence weight is at rest.
///
/// # What each weight reads
///
/// Each field is reduced to a `0..1` signal per grid cell and then centred, so
/// a weight of `0` contributes exactly nothing and the multiplier at rest is
/// exactly `1.0`:
///
/// | weight | signal |
/// |---|---|
/// | elevation | relative land elevation, `(h - sea) / (1 - sea)`; `0` on water |
/// | moisture | the rainfall field, clamped to `0..1` |
/// | biome | [`BIOME_VEGETATION_COVER`] of `classify_biome(temperature, rainfall)`; `0` on water |
/// | geology | the Rec.709 luma of the cell's own rock in the current lithology palette; the neutral `0.5` where there is no lithology (a loaded save) |
///
/// `m = 1 + Σ wₖ·(2·sₖ − 1)`, clamped to `0..2` so a stack of weights can
/// double the grade or cancel it but never invert it.
///
/// # Resolution
///
/// The multiplier is a **field** quantity, so it is built per grid cell and
/// sampled nearest-neighbour into the output raster. At the on-screen size
/// (`w == gw`, `h == gh`) that mapping is the identity; at an 8K export it is
/// a block sample, which is right for a term that only scales how hard a
/// restrained grade lands and wrong to spend a bilinear read on.
///
/// # NaN
///
/// The three float fields are `f32` grids the generator produced and every use
/// below goes through `clamp01`, which pins a NaN to the low end rather than
/// propagating it (`cartalith-rust-conventions`: this file never sorts or
/// `partial_cmp`s a float, so there is no panic path here).
#[allow(dead_code)]
pub fn build_grade_influence(ctx: &RenderCtx, w: usize, h: usize) -> Vec<f32> {
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let cell = build_grade_influence_cells(ctx);
    if cell.is_empty() || (w == ctx.gw && h == ctx.gh) {
        return cell;
    }
    grade_influence_rows(&cell, ctx.gw, ctx.gh, w, h, 0, h)
}

/// [`build_grade_influence`]'s **cell half**: the multiplier per grid cell, or
/// an empty `Vec` when the grade has no field weight (or grades nothing).
/// Resolution-free, so a banded export builds it once for the whole image
/// (`EXPORT_SCOPE.md` §4.1) and lifts it per band with [`grade_influence_rows`].
#[allow(dead_code)]
pub fn build_grade_influence_cells(ctx: &RenderCtx) -> Vec<f32> {
    let a = &ctx.appearance;
    let (gw, gh) = (ctx.gw, ctx.gh);
    if a.grade_influence_is_flat() || a.grade_is_identity() || gw == 0 || gh == 0 {
        return Vec::new();
    }
    let n = gw * gh;
    if ctx.field.len() < n || ctx.temperature.len() < n || ctx.rainfall.len() < n {
        return Vec::new();
    }
    let span = (1.0 - ctx.sea_level).max(1e-6);
    let lith = ctx.lithology.filter(|l| l.len() >= n);
    let mut cell = vec![1f32; n];
    cell.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        for (x, m) in row.iter_mut().enumerate() {
            let i = y * gw + x;
            let hv = ctx.field[i] as f64;
            let water = hv < ctx.sea_level;
            let s_elev = if water { 0.0 } else { clamp01((hv - ctx.sea_level) / span) };
            let s_moist = clamp01(ctx.rainfall[i] as f64);
            let s_biome = if water {
                0.0
            } else {
                let b = cartalith_civ::classify_biome(ctx.temperature[i] as f64, s_moist) as usize;
                BIOME_VEGETATION_COVER[b.min(BIOME_VEGETATION_COVER.len() - 1)]
            };
            let s_geo = match lith {
                Some(l) => clamp01(luma(litho_palette(a, l[i])[1]) / 255.0),
                None => 0.5,
            };
            let v = 1.0
                + a.grade_field_elevation * (2.0 * s_elev - 1.0)
                + a.grade_field_moisture * (2.0 * s_moist - 1.0)
                + a.grade_field_biome * (2.0 * s_biome - 1.0)
                + a.grade_field_geology * (2.0 * s_geo - 1.0);
            *m = v.clamp(0.0, 2.0) as f32;
        }
    });
    cell
}

/// [`build_grade_influence`]'s **row half**: `cell` (a `gw × gh` field from
/// [`build_grade_influence_cells`]) sampled nearest-cell into rows
/// `y0 .. y0 + rows` of a `w × h` raster. The row mapping is `oy · gh / h`
/// over the image-space row and the **full** `h` — a band passing its own
/// height would stretch the whole field into every band. Empty in, empty out.
#[allow(dead_code)]
pub fn grade_influence_rows(cell: &[f32], gw: usize, gh: usize, w: usize, h: usize, y0: usize, rows: usize) -> Vec<f32> {
    if cell.is_empty() || w == 0 || h == 0 || gw == 0 || gh == 0 {
        return Vec::new();
    }
    let mut out = vec![1f32; w * rows];
    out.par_chunks_mut(w).enumerate().for_each(|(r, row)| {
        let gy = ((y0 + r) * gh / h).min(gh - 1);
        for (ox, m) in row.iter_mut().enumerate() {
            *m = cell[gy * gw + (ox * gw / w).min(gw - 1)];
        }
    });
    out
}

/// The grade's seven scalar axes resolved at one influence multiplier.
///
/// `at(a, 1.0)` reproduces the unweighted grade exactly — every term is a
/// multiplication by `1.0`, which IEEE-754 guarantees is the identity for
/// every finite value.
#[derive(Clone, Copy)]
struct GradeAxes {
    exposure: f64,
    /// The **weight**, not the exponent — kept in the parameter's own units so
    /// the `!= 0.0` gate below still means "gamma is at rest" and no `powf`
    /// runs on the default path.
    gamma: f64,
    contrast: f64,
    temperature: f64,
    shadow: f64,
    highlight: f64,
    sat: f64,
}

impl GradeAxes {
    fn at(a: &TerrainAppearance, m: f64) -> Self {
        let k = a.grade_contrast * m;
        GradeAxes {
            exposure: 1.0 + a.grade_exposure * m,
            gamma: a.grade_gamma * m,
            // `1 + c` for a lift and `1/(1 - c)` for a boost gives a symmetric
            // feel either side of 0 without the slope ever going negative
            // (which would invert the image rather than flatten it) or
            // infinite.
            contrast: if k >= 0.0 { 1.0 / (1.0 - k * 0.75).max(0.1) } else { 1.0 + k * 0.75 },
            temperature: a.grade_temperature * m,
            shadow: a.grade_shadow_tint * m,
            highlight: a.grade_highlight_tint * m,
            sat: 1.0 + a.grade_saturation * m,
        }
    }
}

/// One channel through the gamma curve. The base is floored at `0` rather than
/// clamped to `0..1`: a negative base with a fractional exponent is NaN in
/// Rust as it is in JS, while a base above `1` is a highlight exposure already
/// pushed there and crushing it here would undo the axis above it.
fn gamma_channel(v: f64, exponent: f64) -> f64 {
    255.0 * (v / 255.0).max(0.0).powf(exponent)
}

/// The grade applied to one pixel at one influence multiplier.
fn grade_pixel(g: &GradeAxes, mut c: Rgb) -> Rgb {
    c = (c.0 * g.exposure, c.1 * g.exposure, c.2 * g.exposure);
    if g.gamma != 0.0 {
        let e = (-g.gamma).exp2();
        c = (gamma_channel(c.0, e), gamma_channel(c.1, e), gamma_channel(c.2, e));
    }
    c = (128.0 + (c.0 - 128.0) * g.contrast, 128.0 + (c.1 - 128.0) * g.contrast, 128.0 + (c.2 - 128.0) * g.contrast);
    if g.temperature != 0.0 {
        c = temperature_shift(c, g.temperature);
    }
    if g.shadow != 0.0 || g.highlight != 0.0 {
        let y = clamp01(luma(c) / 255.0);
        let t = g.shadow * (1.0 - y) + g.highlight * y;
        if t != 0.0 {
            c = temperature_shift(c, t);
        }
    }
    if g.sat != 1.0 {
        c = saturate(c, g.sat);
    }
    c
}

/// The colour grade, over the finished RGB8 raster in place.
///
/// Order is the order a colourist works in and the order that makes each
/// control mean what its name says: **exposure** (a linear gain, so it moves
/// the whole image together), **gamma** (the midtone bend, immediately after
/// the gain, which is the lift-gamma-gain order every grading tool uses),
/// **contrast** (about mid-grey, so exposure and gamma have already put the
/// image where the pivot should sit), **temperature** (global, on the
/// blue↔amber axis), the **two tints** (weighted by luma and its complement,
/// so they land in the halves they name), and **saturation** last, about the
/// graded luma rather than the original.
///
/// `influence` is [`build_grade_influence`]'s per-pixel multiplier, or an
/// **empty** slice for the flat grade. Any other length is treated as empty —
/// a mismatched buffer is a caller bug, and grading half an image is a worse
/// answer than grading it evenly.
///
/// Every stage is luminance-aware rather than a raw channel multiply:
/// saturation is exactly luminance-preserving and both hue axes are
/// luminance-compensated, so a graded map keeps the value structure the relief
/// pipeline built. `TERRAIN_APPEARANCE_RESEARCH.md` §30's anti-list — no black
/// valleys, no blown highlights, no oversaturation — is the reason the ranges
/// are ±1 rather than open-ended and the reason the tints are ±18% at full.
///
/// **The plate frame is graded too**, unlike `apply_local_contrast`, which
/// fades itself out under the border. That is deliberate and it is the
/// difference between the two stages: local contrast is about making *terrain*
/// legible and has no business sharpening a neatline, while a grade is about
/// how the sheet is printed — and a warm print with a cold margin is not a
/// print, it is a terrain image pasted onto a frame.
///
/// `rayon`-parallel per pixel, on `apply_local_contrast`'s own determinism
/// argument: each output byte is a pure function of its own three input bytes.
#[allow(dead_code)]
pub fn apply_color_grade(a: &TerrainAppearance, rgb: &mut [u8], influence: &[f32]) {
    finish_rgb(rgb, None::<fn(usize) -> f64>, Some((a, influence)), ColorSpace::Srgb);
}

// ---- The output colour space (`LARGE_ITEM_RULINGS.md`, Colour management) ----
//
// The ruling: *"a colour space on the render target, threaded through to the
// texture"*, with one cost stated and accepted — *"every golden-parity fixture
// is sRGB, so this touches the one surface the parity harnesses pin. Do it
// behind a default that leaves sRGB byte-identical, or re-baseline deliberately
// and say so."* The first option is taken here, by the *anchor-at-the-default*
// discipline this port already uses for `terrain_detail_k` and `_V3D_RATIO0`:
// `ColorSpace::Srgb` is an **early return**, not a round trip through a matrix
// that happens to be near-identity, so the default path cannot drift by a
// rounding decision. `tests/color_space.rs` pins a hash of the finished default
// render taken before this block was written.

/// The display device the finished raster is encoded for.
///
/// **A display device, not a working space, and the distinction is why this is
/// a two-member enum rather than the three-row radio `menus.gd` still draws.**
/// `GUI_GAP_REGISTER.md` §7.6 read Blender 4.x's Color Management panel and
/// found the design row (`sRGB · Display P3 · linear`) offers one axis where
/// there are two: sRGB and Display P3 are **display devices**; linear is a
/// **working space**, and shipping the three together *"would be a category
/// error that becomes very expensive to unpick later"*. So this axis carries
/// the two display devices, and the working space stays what it measurably is —
/// 8-bit sRGB, stated as a readout rather than offered as a choice.
///
/// The working-space half is not merely unbuilt, it is **unshippable at this
/// precision**, and Godot's own API documentation says so — read on
/// `Image::srgb_to_linear`, whose generated `godot` binding carries the
/// upstream doc string verbatim: *"the 8-bit formats required by this method
/// are not suitable for storing linearly encoded values; a significant amount
/// of color information will be lost in darker values"*. That is the
/// honest answer to whether a colour space means anything before
/// `OUTSTANDING_WORK.md` §2.5's high-precision display pipeline lands: **a
/// gamut change does, a linear working space does not** — see
/// [`apply_color_space`] for the three properties that make the gamut half
/// sound at 8 bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[allow(dead_code)]
pub enum ColorSpace {
    /// IEC 61966-2-1. The app's default, every golden-parity fixture's
    /// encoding, and an exact no-op — [`apply_color_space`] returns before
    /// touching a byte.
    #[default]
    Srgb,
    /// Apple's Display P3: the DCI-P3 primaries on a D65 white point with
    /// sRGB's own transfer function.
    DisplayP3,
}

/// Every colour space this build has, in the order a picker should show them —
/// the engine's list rather than a second copy of it in GDScript, the same rule
/// [`RAMP_MODES`] and `LOOK_PRESETS` follow. The **first entry is the
/// default**, unlike `LOOK_PRESETS`.
// The golden-parity and tier targets `#[path]`-include this file standalone.
#[allow(dead_code)]
pub const COLOR_SPACES: &[&str] = &["sRGB", "Display P3"];

#[allow(dead_code)]
impl ColorSpace {
    /// This space's name, as [`COLOR_SPACES`] spells it.
    pub fn name(self) -> &'static str {
        COLOR_SPACES[self as usize]
    }

    /// One of [`COLOR_SPACES`] by name, or `None` — a caller written against a
    /// newer engine loses a row rather than silently getting the wrong
    /// encoding, the same degrade [`RampMode::from_name`] gives.
    pub fn from_name(n: &str) -> Option<Self> {
        match n {
            "sRGB" => Some(ColorSpace::Srgb),
            "Display P3" => Some(ColorSpace::DisplayP3),
            _ => None,
        }
    }
}

/// Linear sRGB → linear Display P3, derived from the two sets of chromaticities
/// (sRGB `.64/.33 .30/.60 .15/.06`, P3 `.68/.32 .265/.69 .15/.06`, both on D65
/// `.3127/.3290`) by the standard RGB→XYZ construction, then inverted and
/// composed. It agrees to seven decimals with the matrix Chromium and
/// `color.js` ship, which is the cross-check rather than the source.
///
/// **Two properties of this specific matrix are what make the transform sound
/// on 8-bit data, and both are asserted in `tests/color_space.rs`:**
///
/// 1. Every entry is non-negative and **every row sums to exactly 1**, so each
///    output channel is a convex combination of three values in `[0, 1]` and
///    therefore lands in `[0, 1]` too. sRGB is a strict subset of P3, so this
///    direction can never clip — there is no gamut mapping decision to get
///    wrong, only a re-encoding.
/// 2. A row summing to 1 also means a **neutral is exactly preserved**:
///    `r == g == b` in gives the same value out on all three channels, and the
///    decode/encode pair round-trips a byte exactly. The paper ground, the
///    neatlines and the whole grey hillshade view therefore do not move at all;
///    only chroma is re-encoded.
// `pub` so `tests/color_space.rs` can assert properties 1 and 2 of the matrix
// itself rather than only their consequences in the pixels -- a row that stops
// summing to 1 is a defect whatever the picture happens to look like.
#[allow(dead_code)]
pub const SRGB_TO_P3: [[f64; 3]; 3] = [
    [0.822_461_968_714_362_3, 0.177_538_031_285_637_7, 0.0],
    [0.033_194_198_850_961_62, 0.966_805_801_149_038_4, 0.0],
    [0.017_082_630_721_120_03, 0.072_397_440_663_963_47, 0.910_519_928_614_916_5],
];

/// The sRGB EOTF (IEC 61966-2-1) — encoded `[0, 1]` to linear light. Shared by
/// both spaces: Display P3 uses sRGB's transfer function unchanged, and only
/// its primaries differ.
fn srgb_eotf(v: f64) -> f64 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// The sRGB OETF — linear light back to an encoded `[0, 1]`. The exact inverse
/// of [`srgb_eotf`].
fn srgb_oetf(v: f64) -> f64 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Re-encode the finished RGB8 raster for one display device, in place.
///
/// **[`ColorSpace::Srgb`] returns immediately.** That is the whole of the
/// byte-identity guarantee the ruling asked for, and it is a control-flow
/// property rather than a numerical one: the default never enters the transform,
/// so no rounding rule, matrix constant or transfer function below can move a
/// byte of the shipped image.
///
/// # What this is, and what Godot is not doing
///
/// This is an **output transform performed in the engine**, and it has to be:
/// an `Image`/`ImageTexture` carries a `Format` and no colour-space tag, and
/// this project ships the `gl_compatibility` renderer (`project.godot`'s
/// `renderer/rendering_method`; `forward_plus` loses the device on the owner's
/// RX 7800 XT), whose 2D path does no colour management at all. So there is
/// nothing to tag and nothing that would honour a tag — the numbers in the
/// buffer are what reaches the panel. Converting them here is the only place
/// the conversion can happen, and choosing Display P3 is a statement about the
/// **monitor**, for a wide-gamut display that is *not* colour-managing sRGB
/// input for itself. On such a panel untransformed sRGB numbers read
/// oversaturated; these read correct.
///
/// **The boundary, stated rather than implied.** This reaches the terrain
/// raster — the render target. It does **not** reach the overlays Godot draws
/// over that texture (rivers, labels, settlement markers, territory, the scale
/// bar, and the whole DCC chrome), because those are `Color` values going
/// straight into Godot's canvas with no hook to convert them on the way. On a
/// P3 display with this set to Display P3, the map is correct and the furniture
/// over it is not. That is a real limitation of shipping an engine-side
/// transform under a renderer with no colour management, and the panel says so.
///
/// # Precision
///
/// Called on its own, 8 bits in, 8 bits out, so this costs one quantisation. It is bounded and
/// small — the transform cannot clip (see [`SRGB_TO_P3`]) and it preserves
/// neutrals exactly, so the error is confined to chroma; `tests/color_space.rs`
/// measures the worst round-trip channel error over the whole 24-bit cube.
/// The shipped paths no longer call it on its own: since Ruling AN it is the
/// last stage of [`finish_rgb`], whose input is the *continuous* output of
/// local contrast and the grade rather than a byte, so it adds no
/// quantisation of its own there. (Standalone, it is still reached by the
/// screen path when an asset pack is loaded.) What is still not built is an
/// output deeper than 8 bits.
///
/// Rounds where the rest of this file truncates, and the difference is
/// deliberate: truncation is the reference canvas's own behaviour on a value
/// this renderer *computed*, but this stage re-encodes a byte that already
/// exists, and truncating a decode/encode round trip would bias the whole image
/// down by half a level and break the exact-neutral property above.
///
/// `rayon`-parallel per pixel, on [`apply_color_grade`]'s own determinism
/// argument: each output byte is a pure function of its own three input bytes.
// Used by `lib.rs::build_color_texture`; the `#[path]`-including test targets
// see it as unreachable.
#[allow(dead_code)]
pub fn apply_color_space(space: ColorSpace, rgb: &mut [u8]) {
    // `finish_rgb` returns before touching a byte when `space` is sRGB. The
    // decode is no longer a 256-entry table: since Ruling AN its input is a
    // continuous value whenever an earlier stage ran in the same pass.
    finish_rgb(rgb, None::<fn(usize) -> f64>, None, space);
}

/// `state.mode === 'shade'` (reference `renderNow` line 8535, repeated
/// verbatim in `debugBaseColor` at 8202) — the grey hillshade view, as an RGB8
/// raster the size of the grid.
///
/// ```js
/// const s = 0.15 + 0.85 * shadeFactor(x, y); let c = s * 235; r = g = b = c;
/// if (isWater(vw)) { r = c * 0.45; g = c * 0.6; b = Math.min(255, c * 0.9 + 40); }
/// ```
///
/// `shadeFactor` is this file's [`RenderCtx::macro_shade`], which is why this
/// lives here rather than in `sample_bridge.rs` next to the other layer
/// rasters: that module deliberately carries no `RenderCtx` and has no access
/// to the shading rig at all.
///
/// # What it deliberately leaves out
///
/// The reference reaches this branch through the whole of `renderNow`, so its
/// own `layers/hillshade.png` also carries whatever river, wave and tide
/// overlays happen to be enabled when the export runs. This does not, and that
/// is the point of the layer: a hillshade with rivers painted into it is not a
/// hillshade, and `layers/biome.png` beside it already carries them.
///
/// **[`LayerStack`] does not reach this either, deliberately.** The stack says
/// how the light curve composites *into the map*; this is the data layer on its
/// own, at grid resolution, beside the `.f32` blobs. Hiding the Hillshade row
/// so the map reads as flat colour is not a request to stop exporting the
/// hillshade, and dimming it to 40 % would write a PNG that is not a hillshade
/// of anything. `layers/biome.png`, which *is* the composited map, goes through
/// [`bake_rect`] and carries the stack in full.
///
/// `rayon`-parallel by row on the same argument every other whole-grid pass in
/// this file makes: `macro_shade` is a pure function of the immutable field and
/// each row owns disjoint output bytes.
#[allow(dead_code)]
pub fn hillshade_raster(ctx: &RenderCtx) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let mut out = vec![0u8; gw * gh * 3];
    if gw == 0 || gh == 0 || ctx.field.len() < gw * gh {
        return out;
    }
    out.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
        for x in 0..gw {
            let c = (0.15 + 0.85 * ctx.macro_shade(x, y)) * 235.0;
            let (r, g, b) =
                if ctx.h(x, y) < ctx.sea_level { (c * 0.45, c * 0.6, cartalith_jsmath::js_min(255.0, c * 0.9 + 40.0)) } else { (c, c, c) };
            let o = x * 3;
            row[o] = r.clamp(0.0, 255.0) as u8;
            row[o + 1] = g.clamp(0.0, 255.0) as u8;
            row[o + 2] = b.clamp(0.0, 255.0) as u8;
        }
    });
    out
}

/// Is grid cell `(x, y)` water for the toon outline — sea (`h < sea_level`) or
/// an above-sea lake (`lake_class == 2`), exactly the two tests `cell_color`
/// uses to leave the land branch.
///
/// Off the grid is **not** water, and x never wraps even in world mode: the
/// plate edge is not a coast, and [`BakeFields::pixel`]'s twin
/// ([`toon_water_f`]) cannot wrap either (`sample_arr` clamps), so wrapping
/// here would make the export disagree with the screen at the seam.
fn toon_water_cell(ctx: &RenderCtx, x: i64, y: i64) -> bool {
    if x < 0 || y < 0 || x >= ctx.gw as i64 || y >= ctx.gh as i64 {
        return false;
    }
    let (xu, yu) = (x as usize, y as usize);
    ctx.h(xu, yu) < ctx.sea_level || ctx.lake_class.is_some_and(|l| l[yu * ctx.gw + xu] == 2)
}

/// [`toon_water_cell`] at a fractional grid position, for the export bake:
/// `sample_arr`'s bilinear height and [`bake_lake_at`]'s lake rule — the two
/// tests [`BakeFields::pixel`] itself branches on. At an integer position both
/// reduce to the cell's own value, so an export at the grid's resolution draws
/// the screen's keyline cell for cell.
fn toon_water_f(ctx: &RenderCtx, gx: f64, gy: f64) -> bool {
    let (gw, gh) = (ctx.gw, ctx.gh);
    // Off the plate is not water -- the same rule as the grid twin, and not
    // `sample_arr`'s clamp, which would repeat the edge cell outward.
    if gx < 0.0 || gy < 0.0 || gx > (gw - 1) as f64 || gy > (gh - 1) as f64 {
        return false;
    }
    sample_arr(ctx.field, gx, gy, gw, gh) < ctx.sea_level || ctx.lake_class.is_some_and(|l| bake_lake_at(l, gx, gy, gw, gh))
}

/// Top-level per-cell colour, `[0,1]` per channel — `isWater(v) ?
/// seaColor(...) : surfaceColor(...)` (`debugBaseColor`'s `'biome'`
/// branch, 8204; the main renderer's own default mode).
pub fn cell_color(ctx: &RenderCtx, x: usize, y: usize) -> (f64, f64, f64) {
    cell_color_river(ctx, x, y, ctx.river_layer.and_then(|l| l.at(x, y)))
}

/// [`cell_color`] with the river pixel supplied by the caller -- `None` is
/// the terrain alone under the same context, which is what
/// `WorldGen::build_color_texture` measures local contrast from (and hands
/// the deep-zoom tiles as their detail band), so a river is never the edge
/// that stage enhances: it would ring every river with a halo at the blur's
/// radius, measured on the tiles as a soft ghost of the grid-resolution river
/// around the crisp one.
pub fn cell_color_river(ctx: &RenderCtx, x: usize, y: usize, river: Option<[f32; 4]>) -> (f64, f64, f64) {
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
        let n_low = sea_grain(&ctx.appearance, x as f64, y as f64, ctx.gw);
        sea_color_core(&ctx.appearance, depth, t, n_low, shw, ctx.vignette_at(x, y))
    } else if ctx.lake_class.is_some_and(|l| l[i] == 2) {
        // v0.103 (8580): an above-sea lake is flat freshwater -- `lakeColor`
        // (8324), which is [`lake_color`], the same surface the tile draws.
        lake_color(&ctx.appearance, t, sea_grain(&ctx.appearance, x as f64, y as f64, ctx.gw), ctx.vignette_at(x, y))
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
        // B4 ecotone widening (8208's `sdfEcoKv(_biomeBD ? _biomeBD[i] : null)`)
        // — the literal `1.0` where the field is empty is the reference's own
        // `ecoK != null ? ecoK : 1`, not a stand-in for a missing distance.
        let eco_k = if ctx.biome_bd.is_empty() { 1.0 } else { sdf_eco_k(ctx.biome_bd[i] as f64, ctx.appearance.sdf_biomes, ctx.gw) };
        let c = land_color(&ctx.appearance, t, m, slope, r_frac, twi, asp, curv, ctx.macro_shade(x, y), ctx.meso_shade(x, y), ctx.vignette_at(x, y), ctx.ao[i] as f64, eco_k, ctx.hydro_wet[i] as f64, ctx.litho_at(x, y), grad, x as f64, y as f64, ctx.gw, ctx.gh, ctx.splat.as_ref(), ctx.paint_at(i), ctx.ground,
            // LOD-D4 is a TILE stage. The grid path passes a literal `0.0`,
            // which takes `land_color`'s dedicated no-ice branch, so the
            // shipped screen render is byte-identical to what it was before
            // the milestone. The scope's stage 1 says the glacier field is
            // *"usable by the main map"* -- `build_glacier_potential` is
            // `pub` and takes only grid inputs, so it is -- and stops short
            // of saying the main map uses it, which would re-baseline every
            // default-appearance render in the tree.
            0.0,
            // LOD-D5 is a TILE stage for the same reason, and the `None` is
            // the same kind of gate: the grid path has no cells-per-pixel to
            // put through the curve (it draws one pixel per cell by
            // definition, where the curve is the identity anyway), so there
            // is nothing for it to evaluate.
            None,
            // Ruling AP's snow facing, at the grid's scale -- the same
            // gradient `slope` came from.
            if slope > 1e-9 { asp / slope } else { 0.0 },
            // The river symbol at this cell, when the screen texture attached
            // its layer (`RenderCtx::with_river_layer`); `None` everywhere else.
            river);
        // R2 ridge crests (8171) — the reference's own slot, immediately after
        // `landColorCore` and folded with its own `0.7`. `crest` is empty
        // unless the stage is on, so this is a length test everywhere else.
        let c = if ctx.crest.is_empty() { c } else { apply_crest(c, ctx.crest[i] as f64 * ctx.appearance.crest_strength * 0.7) };
        // B2 coast bands (8172) — the reference's own slot, immediately after
        // the crest strokes. `coast_sdf` is empty unless the stage is on, so
        // this is a length test on every other path.
        let c = if ctx.coast_sdf.is_empty() { c } else { apply_coast_sdf(c, ctx.coast_sdf[i] as f64, ctx.appearance.sdf_coast, ctx.gw) };
        // B3 river bands — the second half of the reference's single
        // `applyCoastRiverSDFv` call on that same line, in its own order
        // (coast first, then rivers, so a river mouth reads as river over
        // beach and not the other way round).
        let c = if ctx.river_sdf.is_empty() { c } else { apply_river_sdf(c, ctx.river_sdf[i] as f64, ctx.appearance.sdf_rivers, ctx.gw) };
        // Cel / toon keyline on coasts and lake shores, last of the land
        // stages so it sits over the bands above and under the sheet below
        // (the paper tints it like everything else printed on the plate).
        // Off at `0.0`: the neighbourhood is never read.
        if ctx.appearance.toon_outline > 0.0 {
            let (xi, yi) = (x as i64, y as i64);
            apply_toon_outline(c, toon_outline_cover(|dx, dy| toon_water_cell(ctx, xi + dx, yi + dy)), ctx.appearance.toon_outline)
        } else {
            c
        }
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
    let tone = paper_tone(&ctx.appearance, x as f64, y as f64, ctx.gw);
    let (r, g, b) = apply_paper(&ctx.appearance, (r, g, b), tone);
    let (r, g, b) = apply_border(&ctx.appearance, (r, g, b), tone, x as f64, y as f64, ctx.gw, ctx.gh);

    (clamp01(r / 255.0), clamp01(g / 255.0), clamp01(b / 255.0))
}

// ===========================================================================
// The export raster — `bakeDims`/`bakePixel`/`bakeSingle`/`bakeTiled`
// (reference HTML lines 10241, 11931, 11975, 11982)
// ===========================================================================
//
// **This is not the LOD tile pyramid.** The reference has two systems that
// share the verb "bake" and nothing else. `bakeAllTiles`/`atlas*` builds the
// deep-zoom pyramid (`cartalith_engine::bake`, shipped); `bakeDims`/
// `bakePixel`/`bakeSingle`/`bakeTiled` renders **one flat export raster** at
// a user-chosen resolution — `exportZip`'s `map.png`, the `bakeRes` (2K/4K/
// 8K) and `bakeTiles` header controls. The scoping correction in commit
// `f11111f` is what separated them; this section is the second one.
//
// # Why it could not simply call `cell_color` in a loop
//
// The export raster is *finer than the grid*: 8192 px across a 2048-cell
// world is four output pixels per cell on each axis. `cell_color` takes
// `(x, y): usize` — a cell index — so a loop over output pixels could only
// ever ask it for the nearest cell, which is a nearest-neighbour upscale of
// the screen image rather than a higher-resolution render of the world.
//
// The reference's answer, reproduced here: sample every *field* bilinearly
// at the fractional grid position the output pixel lands on, and run the
// **whole material path** on those sampled values. Materials, hillshade,
// noise grain, the paper and the plate frame are then all evaluated per
// output pixel, so a 4K export really does carry four times the material
// detail of a 2K one rather than the same picture resampled.
//
// # What makes that safe for parity
//
// `sampleArr` at an exact integer coordinate returns the cell value with
// zero interpolation weight on its neighbours — the reference states this
// itself, at `curvatureAtF` (7620), as the reason it could add fractional
// twins of `curvatureAt`/`aspectFactor` without touching the main map's own
// per-pixel render. The same property holds here, and
// `tests/bake_raster.rs` pins it: [`BakeFields::pixel`] at every integer
// cell of a real fixture is **bit-identical** to [`cell_color`] at the same
// cell. That is the parity check this port can actually make — the
// reference's `bakePixel` diverges from its own `surfaceColor` in two small
// ways (a different sea-noise expression, and no wave/paper/frame stages,
// none of which existed when it was written), so matching *it* pixel-for-
// pixel would mean shipping an export that does not match this port's own
// screen. The reference's stated intent is the one worth honouring — its
// own comments say *"bakes match the screen"*, twice — and the
// integer-identity test states it more strongly than a golden dump could.
//
// "Bit-identical" is exact on that test's 24x17 fixture and **f32-tight**
// at scale: the prologue below is `f32` (see [`BakeFields::pixel`]), so at
// 2048x1312 through the real binding a dozen or so bytes of 8 060 928 come
// back one level off (12 and 17 on two runs). `the_integer_identity_is_f32_tight_not_bit_exact_at_scale`
// pins that bound and explains why widening the prologue would be the
// wrong fix.
//
// # The prologue is not an optimisation
//
// [`BakeFields`] precomputes slope, macro shade and meso shade at **grid**
// resolution, exactly as the reference's bake prologue does (the block
// immediately above 11931: `for(y…)for(x…){ gridSlope[i]=slopeAt(x,y);
// gridShade[i]=macroShade(x,y); gridShadeMeso[i]=shadeFactor2(x,y); }`).
// Computing them from bilinearly-sampled neighbours instead would be
// **wrong, not merely different**: all three are per-*cell* height
// differences, so evaluating them on a 4x-finer lattice divides every slope
// by four, and `material_weights`' own normalizers (`slope/0.04`,
// `slope/0.08`) would reclassify most rock and scree as grass. Sampling the
// grid-resolution field keeps a mountain as steep in the export as it is on
// screen.

/// `sampleArr(a, fx, fy)` (reference HTML line 10242) — bilinear sample of a
/// grid-resolution field at a fractional cell position, clamped to the grid
/// on both axes.
///
/// Transcribed in the reference's own operand order, including the clamp
/// form and the final blend expression's association —
/// `cartalith-rust-conventions`' "do not reorder float operations" applies
/// to a sampler as much as to a formula, and this one feeds every value in
/// the material path.
///
/// Never wraps, even in world mode. That is the reference's behaviour and it
/// is also the right one for an export: the raster's left and right edges
/// are the plate's edges, not a seam to blend across.
fn sample_arr(a: &[f32], fx: f64, fy: f64, gw: usize, gh: usize) -> f64 {
    let fx = if fx < 0.0 {
        0.0
    } else if fx > (gw - 1) as f64 {
        (gw - 1) as f64
    } else {
        fx
    };
    let fy = if fy < 0.0 {
        0.0
    } else if fy > (gh - 1) as f64 {
        (gh - 1) as f64
    } else {
        fy
    };
    let x0 = fx as usize;
    let y0 = fy as usize;
    let x1 = if x0 < gw - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < gh - 1 { y0 + 1 } else { y0 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    (a[y0 * gw + x0] as f64 * (1.0 - tx) + a[y0 * gw + x1] as f64 * tx) * (1.0 - ty)
        + (a[y1 * gw + x0] as f64 * (1.0 - tx) + a[y1 * gw + x1] as f64 * tx) * ty
}

/// `bakeDims(W)` (reference line 10241) — the output height that keeps the
/// export at the world's own aspect ratio, `Math.round(W*GH/GW)`.
///
/// `js_round` rather than `f64::round`: the two differ on an exact `.5`
/// (JS rounds half toward `+∞`, Rust rounds half away from zero), and
/// `W*GH/GW` lands on one for every square grid at any export width — the
/// most common case there is, not an exotic one.
pub fn bake_dims(w: usize, gw: usize, gh: usize) -> (usize, usize) {
    if gw == 0 || w == 0 {
        return (0, 0);
    }
    let h = cartalith_jsmath::js_round(w as f64 * gh as f64 / gw as f64);
    (w, (h.max(1.0)) as usize)
}

/// The bake prologue's three grid-resolution derived fields (see the section
/// header for why they are precomputed rather than sampled).
pub struct BakeFields {
    slope: Vec<f32>,
    shade: Vec<f32>,
    meso: Vec<f32>,
    /// RV-5: the screen's smooth-shore field ([`shore_field_forced`], the same
    /// numbers `WorldGen::shore_field_texture` hands `map_shore.gdshader`).
    /// **Empty by construction** -- attach with [`Self::with_shore_field`] --
    /// and empty is the pre-RV-5 bake exactly: water by cell/`bake_lake_at`
    /// rule, which is what every golden fixture draws.
    shore: Vec<f32>,
}

impl BakeFields {
    /// The reference's bake prologue, run once per export.
    ///
    /// `rayon`-parallel by row, bit-identical by construction:
    /// `slope_at`/`macro_shade`/`meso_shade` are pure functions of the
    /// immutable field and each row writes its own disjoint slice, so
    /// nothing depends on the schedule (§27, the same argument
    /// `build_color_texture`'s own parallel loop makes).
    pub fn new(ctx: &RenderCtx) -> Self {
        let gw = ctx.gw;
        let n = gw * ctx.gh;
        let mut slope = vec![0f32; n];
        let mut shade = vec![0f32; n];
        let mut meso = vec![0f32; n];
        slope
            .par_chunks_mut(gw)
            .zip(shade.par_chunks_mut(gw))
            .zip(meso.par_chunks_mut(gw))
            .enumerate()
            .for_each(|(y, ((sl, sh), ms))| {
                for x in 0..gw {
                    sl[x] = ctx.slope_at(x, y) as f32;
                    sh[x] = ctx.macro_shade(x, y) as f32;
                    ms[x] = ctx.meso_shade(x, y) as f32;
                }
            });
        BakeFields { slope, shade, meso, shore: Vec::new() }
    }

    /// **RV-5: draw the export's water with the screen's smooth shoreline.**
    /// Attaches `shore` ([`shore_field_forced`] over the drawn classification,
    /// its fill level and the forced-lake mask -- what `build_color_texture`
    /// builds for the screen) so [`Self::pixel_at`] decides water the way
    /// `map_shore.gdshader` does: the bilinear of the field at the pixel,
    /// antialiased by its change per output pixel. Why: the export drew
    /// whole-cell lakes (`bake_lake_at`) that read as stair steps at 16K,
    /// while the screen draws a smooth contour (RV-4) -- including forced
    /// lakes (Ruling BO), whose edge is in the field.
    ///
    /// Refused (left empty, the cell rule kept) when `shore` is not `gw * gh`
    /// or the look draws the reference's cell coast (`smooth_shores` false,
    /// `js_reference()`), so a golden cannot be moved by attaching it. Must
    /// never classify anything: it only decides which colour a pixel shows.
    pub fn with_shore_field(mut self, ctx: &RenderCtx, shore: Vec<f32>) -> Self {
        if ctx.appearance.smooth_shores && shore.len() == ctx.gw * ctx.gh {
            self.shore = shore;
        }
        self
    }

    /// Whether [`Self::with_shore_field`] took its field -- for a caller's
    /// test that the attachment reached the consumer. Reads only.
    #[allow(dead_code)]
    pub fn has_shore_field(&self) -> bool {
        !self.shore.is_empty()
    }

    /// `bakePixel(gx, gy)` (reference line 11931) — one export-raster pixel:
    /// the full material path at a **fractional** grid position.
    ///
    /// Stage for stage the same sequence [`cell_color`] runs, with every
    /// grid read replaced by [`sample_arr`] and every integer-coordinate
    /// helper by its `_f` twin. At an integer `(gx, gy)` every substitution
    /// is an identity *except* the three prologue fields below, which are
    /// `f32` here and `f64` in `cell_color` — so the two agree to `f32`
    /// rounding (`< 1e-7`, and at most one quantized level) rather than to
    /// the bit. `tests/bake_raster.rs` asserts the exact form cell-by-cell
    /// on a small fixture and the bound on a large one.
    ///
    /// **`f32` for the three prologue fields, not `f64`.** Slope and both
    /// shades are stored at the same precision every other field in
    /// `RenderCtx` is (`cartalith-rust-conventions`' "match the original
    /// precision" — the reference's own `gridSlope`/`gridShade`/
    /// `gridShadeMeso` are `Float32Array`s), so a sample of them rounds
    /// exactly where the reference's does.
    pub fn pixel(&self, ctx: &RenderCtx, gx: f64, gy: f64) -> (f64, f64, f64) {
        self.pixel_at(ctx, gx, gy, (1.0, 1.0), None)
    }

    /// [`Self::pixel`] for a pixel of a raster whose neighbouring pixels are
    /// `step` grid cells apart (`(sx, sy)` in [`bake_rect`]), with the river
    /// symbol `river` (premultiplied, [`RiverLayer::at`]) composited inside
    /// [`land_color`] exactly where a deep-zoom tile composites its own. `step`
    /// is read only by the smooth shore ([`Self::with_shore_field`]) -- it is
    /// `fwidth`'s denominator, so the shore's antialiasing is one output pixel
    /// wide at every export resolution. `river` never reaches a water pixel:
    /// water is decided first and is drawn above the river, as on screen
    /// (`river_under_water.gdshader`).
    pub fn pixel_at(&self, ctx: &RenderCtx, gx: f64, gy: f64, step: (f64, f64), river: Option<[f32; 4]>) -> (f64, f64, f64) {
        let (gw, gh) = (ctx.gw, ctx.gh);
        // Clamped once, up front, so every helper below — and
        // `border_cover_f`, whose doc comment relies on it — sees a
        // coordinate that is on the plate. The reference clamps inside
        // `sampleArr` on every call instead; this is the same value at one
        // test per pixel rather than one per field read.
        let gx = gx.clamp(0.0, (gw - 1) as f64);
        let gy = gy.clamp(0.0, (gh - 1) as f64);

        let h = sample_arr(ctx.field, gx, gy, gw, gh);
        let t = sample_arr(ctx.temperature, gx, gy, gw, gh);
        let vig = ctx.vignette_at_f(gx, gy);

        let lake = || lake_color(&ctx.appearance, t, sea_grain(&ctx.appearance, gx, gy, gw), vig);
        let (r, g, b) = match self.shore_cover(ctx, gx, gy, step) {
            // No shore field (every golden, `js_reference()`): the cell rule,
            // byte for byte what this bake drew before RV-5.
            None => {
                if h < ctx.sea_level {
                    self.sea_px(ctx, gx, gy, t, vig)
                } else if ctx.lake_class.is_some_and(|l| bake_lake_at(l, gx, gy, gw, gh)) {
                    // v0.103's above-sea lake, as `cell_color` draws it: flat
                    // freshwater, `lake_color`. This branch was missing from the
                    // export bake until 2026-09-24, so an exported PNG showed dry
                    // ground where the screen showed a lake. Inert without
                    // `with_lakes`, which no golden fixture attaches.
                    lake()
                } else {
                    self.land_px(ctx, gx, gy, h, t, vig, river)
                }
            }
            // RV-5: `map_shore.gdshader`'s rule. Wholly water or wholly land
            // skips the other colour; only a shore pixel pays for both.
            Some((cov, kind)) => {
                let water = || match kind {
                    Some((sx, sy)) => self.sea_px(ctx, sx, sy, t, vig),
                    None => lake(),
                };
                if cov >= 1.0 {
                    water()
                } else if cov <= 0.0 {
                    self.land_px(ctx, gx, gy, h, t, vig, river)
                } else {
                    let (w, l) = (water(), self.land_px(ctx, gx, gy, h, t, vig, river));
                    (l.0 + (w.0 - l.0) * cov, l.1 + (w.1 - l.1) * cov, l.2 + (w.2 - l.2) * cov)
                }
            }
        };

        let tone = paper_tone(&ctx.appearance, gx, gy, gw);
        let (r, g, b) = apply_paper(&ctx.appearance, (r, g, b), tone);
        let (r, g, b) = apply_border(&ctx.appearance, (r, g, b), tone, gx, gy, gw, gh);

        (clamp01(r / 255.0), clamp01(g / 255.0), clamp01(b / 255.0))
    }

    /// **RV-5: how much of this pixel the water covers, and whose water it
    /// is** -- `map_shore.gdshader` transcribed onto the bake. `None` without
    /// a shore field (the cell rule applies). Otherwise `(cover, sea)`:
    ///
    /// - all four surrounding cell centres water -> `1`; all land -> `0` (the
    ///   shader leaves such a pixel alone, so it is its class outright);
    /// - between the two classes -> `0.5 + s / fwidth(s)`, clamped
    ///   (`shore_field.gdshaderinc::shore_cover`), `s` the bilinear field and
    ///   `fwidth` its change across one output pixel (`step` cells), from the
    ///   bilinear's own derivative rather than a finite difference -- the
    ///   same quantity, exact here because the bake knows its step.
    ///
    /// The second value is **which water** the pixel shows: `None` for lake
    /// colour, `Some(position)` for sea colour evaluated at `position`. The
    /// rule, and why it is not the shader's nearest-texel rule:
    ///
    /// - the pixel's own bilinear height below sea level -> sea, at the pixel
    ///   (the sea-level contour, the pre-RV-5 bake's and the deep-zoom tile's
    ///   own sea test, so the boundary between a coastal lake and the sea is
    ///   that smooth contour);
    /// - otherwise lake, if any wet corner is an above-sea cell (`cell_color`
    ///   draws those in lake colour);
    /// - otherwise sea, evaluated at the nearest wet corner. At the pixel
    ///   itself the depth would be zero, the palest shallow tint -- measured on
    ///   the first 16K export of this code as pale slivers along every coast.
    ///
    /// Taking the kind from the nearest water corner, as `map_shore.gdshader`
    /// does, was built first and drew the lake/sea boundary inside the water
    /// as cell-sized stair steps at 16K -- the screen's base view has the same
    /// steps under its NEAREST texture, but it is shown only below x2.2, and
    /// the tiles that replace it there use the height rule above. The corners
    /// are `sample_arr`'s (its clamp at the last row/column), so the field is
    /// read on the grid the terrain is.
    fn shore_cover(&self, ctx: &RenderCtx, gx: f64, gy: f64, step: (f64, f64)) -> Option<(f64, Option<(f64, f64)>)> {
        let (gw, gh) = (ctx.gw, ctx.gh);
        if self.shore.len() != gw * gh || gw == 0 || gh == 0 {
            return None;
        }
        let (x0, y0) = (gx as usize, gy as usize);
        let x1 = if x0 < gw - 1 { x0 + 1 } else { x0 };
        let y1 = if y0 < gh - 1 { y0 + 1 } else { y0 };
        let (tx, ty) = (gx - x0 as f64, gy - y0 as f64);
        let c = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
        let s: Vec<f64> = c.iter().map(|&(x, y)| self.shore[y * gw + x] as f64).collect();
        let wet = [s[0] > 0.0, s[1] > 0.0, s[2] > 0.0, s[3] > 0.0];
        // The nearest water corner decides the kind; it is also the "any
        // water" test.
        let d = [tx * tx + ty * ty, (tx - 1.0).powi(2) + ty * ty, tx * tx + (ty - 1.0).powi(2), (tx - 1.0).powi(2) + (ty - 1.0).powi(2)];
        let mut best: Option<(f64, usize)> = None;
        for k in 0..4 {
            if wet[k] && best.is_none_or(|(bd, _)| d[k] <= bd) {
                best = Some((d[k], k));
            }
        }
        let Some((_, k)) = best else { return Some((0.0, None)) };
        let kind = if sample_arr(ctx.field, gx, gy, gw, gh) < ctx.sea_level {
            Some((gx, gy))
        } else if (0..4).any(|j| wet[j] && ctx.field[c[j].1 * gw + c[j].0] as f64 >= ctx.sea_level) {
            None
        } else {
            Some((c[k].0 as f64, c[k].1 as f64))
        };
        if wet.iter().all(|&w| w) {
            return Some((1.0, kind));
        }
        let v = (s[0] * (1.0 - tx) + s[1] * tx) * (1.0 - ty) + (s[2] * (1.0 - tx) + s[3] * tx) * ty;
        let dx = ((1.0 - ty) * (s[1] - s[0]) + ty * (s[3] - s[2])) * step.0;
        let dy = ((1.0 - tx) * (s[2] - s[0]) + tx * (s[3] - s[1])) * step.1;
        // `max(w, 1e-12)`: the shader's own guard -- a flat field is a hard step.
        let fw = (dx.abs() + dy.abs()).max(1e-12);
        Some(((0.5 + v / fw).clamp(0.0, 1.0), kind))
    }

    /// How much of the bake pixel at `(gx, gy)` (neighbours `step` cells
    /// apart) is drawn as water, `0..=1`: [`Self::shore_cover`]'s answer with
    /// a shore field, and the cell rule's `0`/`1` without one (the same two
    /// tests [`Self::pixel_at`] branches on). Exists so a test can measure
    /// WHERE the export puts a shoreline independently of what colour it
    /// paints either side (`export_raster.rs`'s RV-5 parity test). Reads only;
    /// must never be used to classify.
    #[allow(dead_code)]
    pub fn water_cover(&self, ctx: &RenderCtx, gx: f64, gy: f64, step: (f64, f64)) -> f64 {
        let (gw, gh) = (ctx.gw, ctx.gh);
        let gx = gx.clamp(0.0, (gw - 1) as f64);
        let gy = gy.clamp(0.0, (gh - 1) as f64);
        match self.shore_cover(ctx, gx, gy, step) {
            Some((cov, _)) => cov,
            None => {
                let wet = sample_arr(ctx.field, gx, gy, gw, gh) < ctx.sea_level || ctx.lake_class.is_some_and(|l| bake_lake_at(l, gx, gy, gw, gh));
                if wet { 1.0 } else { 0.0 }
            }
        }
    }

    /// The sea colour at a bake pixel, waves included -- the pre-RV-5 bake's
    /// sea branch and its trailing `apply_waves` (which only ever ran on that
    /// branch), folded into one place so the smooth shore can ask for it.
    fn sea_px(&self, ctx: &RenderCtx, gx: f64, gy: f64, t: f64, vig: f64) -> (f64, f64, f64) {
        let (gw, gh) = (ctx.gw, ctx.gh);
        let hs = sample_arr(&ctx.sea_h, gx, gy, gw, gh);
        let shw = sample_arr(&ctx.sea_shade, gx, gy, gw, gh);
        let depth = if ctx.sea_level <= 0.0 { 0.0 } else { clamp01((ctx.sea_level - hs) / ctx.sea_level) };
        let n_low = sea_grain(&ctx.appearance, gx, gy, gw);
        let c = sea_color_core(&ctx.appearance, depth, t, n_low, shw, vig);
        if ctx.coast_d.is_empty() { c } else { apply_waves(&ctx.appearance, c, sample_arr(&ctx.coast_d, gx, gy, gw, gh), gw) }
    }

    /// The land colour at a bake pixel (the pre-RV-5 bake's land branch),
    /// with the river symbol `river` composited inside [`land_color`].
    #[allow(clippy::too_many_arguments)]
    fn land_px(&self, ctx: &RenderCtx, gx: f64, gy: f64, h: f64, t: f64, vig: f64, river: Option<[f32; 4]>) -> (f64, f64, f64) {
        let (gw, gh) = (ctx.gw, ctx.gh);
        {
            let m = sample_arr(ctx.rainfall, gx, gy, gw, gh);
            let r_frac = if (1.0 - ctx.sea_level) <= 0.0 { 0.0 } else { (h - ctx.sea_level) / (1.0 - ctx.sea_level) };
            let slope = sample_arr(&self.slope, gx, gy, gw, gh);
            let flow = ctx.flow.map(|f| sample_arr(f, gx, gy, gw, gh)).unwrap_or(0.0);
            let a = (flow / (gw * gh) as f64).max(1e-4);
            let beta = slope.max(0.002);
            let twi = (a / beta).ln();
            let asp = ctx.aspect_factor_f(gx, gy);
            let curv = ctx.curvature_at_f(gx, gy);
            let grad = if ctx.appearance.npr.hachure > 0.0 { ctx.grad_at_f(gx, gy) } else { (0.0, 0.0) };
            let c = land_color(
                &ctx.appearance,
                t,
                m,
                slope,
                r_frac,
                twi,
                asp,
                curv,
                sample_arr(&self.shade, gx, gy, gw, gh),
                sample_arr(&self.meso, gx, gy, gw, gh),
                vig,
                sample_arr(&ctx.ao, gx, gy, gw, gh),
                // The bake's own `sdfEcoKv(sampleArr(_biomeBD, gx, gy))`
                // (12008). `sample_arr` on an empty field is not defined, so
                // the empty case is the same literal `1.0` the screen path
                // uses — the two must agree or the PNG is a different picture.
                if ctx.biome_bd.is_empty() { 1.0 } else { sdf_eco_k(sample_arr(&ctx.biome_bd, gx, gy, gw, gh), ctx.appearance.sdf_biomes, gw) },
                sample_arr(&ctx.hydro_wet, gx, gy, gw, gh),
                ctx.litho_at_f(gx, gy),
                grad,
                gx,
                gy,
                gw,
                gh,
                ctx.splat.as_ref(),
                ctx.paint_at_f(gx, gy),
                ctx.ground,
                // The bake draws the grid, not a tile -- same `0.0`, same
                // reason as `cell_color`'s above.
                0.0,
                // LOD-D5. `None` here is NOT "the bake is one pixel per
                // cell" -- `bake_rect` magnifies, and this path samples the
                // grid at fractional coordinates exactly as a tile does. It
                // is that the bake upsamples the **coarse** field and never
                // runs `add_zoom_detail`, so the residual the micro band
                // would read is identically zero and the extra relief the
                // re-weighting exists to expose is not in this buffer. The
                // bake's own golden (`tests/bake_raster.rs`) is the second
                // reason: giving it the curve would re-baseline it for a
                // stage with no input.
                None,
                // Ruling AP's snow facing, at the grid's scale, as `cell_color`.
                if slope > 1e-9 { asp / slope } else { 0.0 },
                // RV-5: the vector river at this export pixel
                // ([`paint_vector_rivers`]), in the tile's slot; `None` for
                // the terrain-only pass and every golden.
                river,
            );
            // R2 ridge crests, the bake's own slot (11971) — `sampleArr` of
            // the same field, folded with the same `0.7`.
            let c = if ctx.crest.is_empty() { c } else { apply_crest(c, sample_arr(&ctx.crest, gx, gy, gw, gh) * ctx.appearance.crest_strength * 0.7) };
            // B2 coast bands, the bake's own slot (11972) — the reference's
            // own comment on that line is why this exists at all: *"bake SDF
            // coast/river bands to match the screen"*. A capability wired to
            // `cell_color` and not here is the failure this project has
            // already shipped once (`with_ground_tiles`).
            let c = if ctx.coast_sdf.is_empty() { c } else { apply_coast_sdf(c, sample_arr(&ctx.coast_sdf, gx, gy, gw, gh), ctx.appearance.sdf_coast, gw) };
            // B3 river bands, the bake's own slot (12011) — the reference's
            // comment there is *"bake SDF coast/river bands to match the
            // screen"*, and the plural is the whole point.
            let c = if ctx.river_sdf.is_empty() { c } else { apply_river_sdf(c, sample_arr(&ctx.river_sdf, gx, gy, gw, gh), ctx.appearance.sdf_rivers, gw) };
            // The toon keyline, in `cell_color`'s slot and in **cell** units
            // (neighbours one grid cell away, not one output pixel), so an 8K
            // export is the screen's picture with the keyline at the same
            // width of ground, rather than a hairline four times thinner.
            if ctx.appearance.toon_outline > 0.0 {
                apply_toon_outline(c, toon_outline_cover(|dx, dy| toon_water_f(ctx, gx + dx as f64, gy + dy as f64)), ctx.appearance.toon_outline)
            } else {
                c
            }
        }
    }
}

/// **A loaded save's river flag at cell `i`**: `1.0` where the save's
/// `strahler_order` raster is non-zero, `0.0` elsewhere and past its end.
///
/// What it is for: a legacy `.zip` opened without its substrate
/// (`WorldSource::Loaded`) has no traced network -- `SAVEFILE_COMPAT.md`
/// stores no channel topology -- so there is no vector geometry to draw, and
/// this one-cell flag is the only river such a world has. The screen, the
/// tiles and every export tint it the same way ([`channel_tint`]), so the
/// three agree for a save as they agree for a generated world.
///
/// **RV-5 (2026-09-28) retired everything else this used to carry.** It was
/// `RiverInk`, an enum of this flag and `Stamped` -- `stamp_river_intensity`'s
/// disc raster, which after 2026-09-22 only the exports still drew, while the
/// screen drew RV-2's vector strokes. Ruling AZ (*"the vector line, not the
/// baked ink"*) and DECISIONS §7p removed the stamped path: a generated world
/// now exports its vector rivers ([`ExportRivers::vector`]). Must never be
/// handed a generated world's channel mask: that world draws vectors, and a
/// flag under them would put a second, stepped river beneath the smooth one.
#[inline]
pub fn save_flag_at(v: &[u8], i: usize) -> f32 {
    // Past the end is "no river", never a panic across the gdext boundary.
    if v.get(i).copied().unwrap_or(0) != 0 { 1.0 } else { 0.0 }
}

/// One rectangle of an export raster, in the terms a river rasterizer needs:
/// its pixel `(x, y)` samples grid coordinate `(bx + x * cx, by + y * cy)` --
/// [`bake_rect`]'s own mapping (a cell's centre at its integer index) -- and it
/// is `w x h` pixels. It is `river_stroke::RasterMap::tile`'s four arguments
/// plus a size, so an export rectangle is placed exactly as a deep-zoom tile
/// is, and a river cannot land half a pixel off the terrain under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RasterRect {
    pub bx: f64,
    pub by: f64,
    pub cx: f64,
    pub cy: f64,
    pub w: usize,
    pub h: usize,
}

/// Rasterizes the world's vector rivers for one [`RasterRect`] under one look
/// -- in the app, `river_stroke::rasterize` over `WorldGen::river_geometry_any`
/// (this file compiles standalone for its tests, so it cannot name that module;
/// the caller supplies the closure). The look is an argument so the export's
/// style override, not the session's, decides the rivers' width and colour.
pub type VectorRivers<'a> = &'a (dyn Fn(&TerrainAppearance, RasterRect) -> RiverLayer + Sync);

/// **What rivers an export draws** (RV-5). At most one of the two is set by
/// the app: a generated world has a vector network, a loaded save has only its
/// flag. `Default` is no rivers -- the terrain alone, which is what a caller
/// comparing against [`cell_color`] directly (and the export's `rivers: false`
/// content option) wants.
///
/// - `vector`: RV-2's strokes, rasterized per rectangle at the export's own
///   resolution through `river_stroke::river_px_width` (the one width seam the
///   screen and the tiles use) and the preset's river treatment
///   ([`river_style_color`]), composited inside [`land_color`] like a tile's
///   ([`paint_vector_rivers`]). This is Ruling AZ's *"the vector line, not the
///   baked ink"*.
/// - `save_flag`: a loaded save's one-cell flag ([`save_flag_at`]), tinted by
///   [`channel_tint`] inside [`bake_rect`] as the screen tints it.
#[derive(Clone, Copy, Default)]
pub struct ExportRivers<'a> {
    pub vector: Option<VectorRivers<'a>>,
    pub save_flag: Option<&'a [u8]>,
}

/// Side of the square blocks a [`RiverLayer`] allocates on demand. Rivers
/// cover a small share of a map, so the layer stores only the blocks a stroke
/// touched; at 2048x1311 a dense layer of `[f32; 4]` would be 43 MB, and at
/// this port's 8192 ceiling 1 GB. `32` is a labelled judgement (16 KB a
/// block); measured with it, the base view's colour field at 2048x1312 holds
/// 20.4 MB (`_rivstyle_probe.gd --grid 2048x1312 --timing-only`).
const RIVER_BLOCK: usize = 32;

/// **The rivers, rasterized into a raster** (owner, 2026-09-27: *"the only
/// issue I have with the rivers: they're drawn on top of the style"* and
/// *"can we put the rivers into the map again with the same vector
/// approach?"*).
///
/// RV-2's strokes are one tapered, antialiased triangle strip per river piece
/// (`river_stroke::stroke_mesh`). Until this layer they were handed to
/// `RenderingServer.canvas_item_add_triangle_array` by `map_overlay.gd` and
/// drawn in one fixed palette *after* the map, so no preset's paper, Painter
/// style or grade ever reached them. This is the same mesh, filled here into
/// a coverage layer at the resolution of the raster it belongs to — one pixel
/// per cell for the screen texture, each tile's own pixels for a deep-zoom
/// tile, never an upsampled copy of either — and composited inside
/// [`land_color`] before the Painter block, the paper and the grade.
///
/// Each pixel is **premultiplied** RGBA: RGB on `land_color`'s own `0..=255`
/// scale already multiplied by alpha, alpha in `0..=1`. `None` from [`Self::at`]
/// is "no stroke reached this pixel", not a transparent colour.
///
/// Must never be read by a render path the JS goldens use -- they attach
/// none, which is what keeps them byte-identical. Since RV-5 the exports DO
/// read one, built per export rectangle at the export's own resolution
/// ([`ExportRivers::vector`], [`paint_vector_rivers`]); a golden fixture
/// passes `ExportRivers::default()` and never reaches it.
pub struct RiverLayer {
    w: usize,
    h: usize,
    bw: usize,
    blocks: Vec<Option<Box<[[f32; 4]]>>>,
}

impl RiverLayer {
    /// An empty `w x h` layer: no block allocated until a stroke reaches it.
    pub fn new(w: usize, h: usize) -> Self {
        let bw = w.div_ceil(RIVER_BLOCK);
        let bh = h.div_ceil(RIVER_BLOCK);
        RiverLayer { w, h, bw, blocks: (0..bw * bh).map(|_| None).collect() }
    }

    /// Width in pixels -- checked by every consumer against its own raster
    /// before indexing (`with_river_layer`, `render_biome_tile_rgba_rivers`).
    pub fn width(&self) -> usize {
        self.w
    }

    /// Height in pixels; see [`Self::width`].
    pub fn height(&self) -> usize {
        self.h
    }

    /// The premultiplied colour at `(x, y)`, or `None` where no stroke drew.
    #[inline]
    pub fn at(&self, x: usize, y: usize) -> Option<[f32; 4]> {
        if x >= self.w || y >= self.h {
            return None;
        }
        let b = self.blocks[(y / RIVER_BLOCK) * self.bw + x / RIVER_BLOCK].as_ref()?;
        let p = b[(y % RIVER_BLOCK) * RIVER_BLOCK + x % RIVER_BLOCK];
        (p[3] > 0.0).then_some(p)
    }

    /// Scale every pixel, premultiplied colour and alpha together -- a uniform
    /// opacity applied once to a layer drawn opaque.
    pub fn scale(&mut self, k: f32) {
        for b in self.blocks.iter_mut().flatten() {
            for p in b.iter_mut() {
                for c in p.iter_mut() {
                    *c *= k;
                }
            }
        }
    }

    /// Bytes the allocated blocks hold (the layer's memory, less its index).
    pub fn allocated_bytes(&self) -> usize {
        self.blocks.iter().flatten().count() * RIVER_BLOCK * RIVER_BLOCK * std::mem::size_of::<[f32; 4]>()
    }

    /// Pixels any stroke reached with non-zero alpha.
    pub fn covered(&self) -> usize {
        self.blocks.iter().flatten().map(|b| b.iter().filter(|p| p[3] > 0.0).count()).sum()
    }

    /// The pixel's slot, allocating its block on first touch. Callers bound
    /// `x`/`y` first; must never be called out of range (it would panic).
    fn px_mut(&mut self, x: usize, y: usize) -> &mut [f32; 4] {
        let bi = (y / RIVER_BLOCK) * self.bw + x / RIVER_BLOCK;
        let b = self.blocks[bi].get_or_insert_with(|| vec![[0.0f32; 4]; RIVER_BLOCK * RIVER_BLOCK].into_boxed_slice());
        &mut b[(y % RIVER_BLOCK) * RIVER_BLOCK + x % RIVER_BLOCK]
    }

    /// Fill an indexed triangle list, **in order**, compositing each
    /// triangle "over" what the earlier ones left — what the GPU did with
    /// the same mesh, so a trunk still covers its tributaries' ends.
    ///
    /// `pts` are in this layer's pixel space with pixel `(x, y)` sampled at
    /// exactly `(x, y)` (a GPU samples at `x + 0.5`; the caller's transform
    /// absorbs the half pixel). `colors` are straight (not premultiplied)
    /// RGBA in `0..=1` per vertex, interpolated linearly across the triangle
    /// as a GPU interpolates vertex colours.
    ///
    /// Coverage is a point sample per pixel under the **top-left rule**, so a
    /// pixel exactly on an edge two triangles of one strip share is filled by
    /// one of them, not blended twice. Antialiasing is the strip's own:
    /// `stroke_mesh` feathers alpha to zero over a fringe outside each edge.
    pub fn fill_triangles(&mut self, pts: &[(f32, f32)], colors: &[[f32; 4]], idx: &[i32]) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let n = pts.len().min(colors.len());
        for t in idx.chunks_exact(3) {
            let (Ok(i0), Ok(i1), Ok(i2)) = (usize::try_from(t[0]), usize::try_from(t[1]), usize::try_from(t[2])) else { continue };
            if i0 >= n || i1 >= n || i2 >= n {
                continue;
            }
            self.fill_one([pts[i0], pts[i1], pts[i2]], [colors[i0], colors[i1], colors[i2]]);
        }
    }

    /// One triangle of [`Self::fill_triangles`]: orientation normalised,
    /// bounding box clipped to the layer, each pixel centre tested by edge
    /// functions under the top-left rule, colour interpolated barycentrically
    /// and composited over. Edge functions in `f64` so a thin fringe triangle
    /// does not lose its pixels to `f32` rounding. Degenerate and non-finite
    /// triangles draw nothing.
    fn fill_one(&mut self, mut p: [(f32, f32); 3], mut c: [[f32; 4]; 3]) {
        let e = |a: (f32, f32), b: (f32, f32), q: (f64, f64)| -> f64 {
            (b.0 as f64 - a.0 as f64) * (q.1 - a.1 as f64) - (b.1 as f64 - a.1 as f64) * (q.0 - a.0 as f64)
        };
        let mut area = e(p[0], p[1], (p[2].0 as f64, p[2].1 as f64));
        if !area.is_finite() || area == 0.0 {
            return;
        }
        if area < 0.0 {
            p.swap(1, 2);
            c.swap(1, 2);
            area = -area;
        }
        // With `area > 0` (edges run one way round), an edge owns the pixels
        // exactly on it when it is a "top" or "left" edge; the triangle on
        // the far side of the same edge walks it the other way and does not.
        let owns = |a: (f32, f32), b: (f32, f32)| -> bool {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            dy < 0.0 || (dy == 0.0 && dx > 0.0)
        };
        let own = [owns(p[1], p[2]), owns(p[2], p[0]), owns(p[0], p[1])];
        let minx = p.iter().map(|q| q.0).fold(f32::INFINITY, f32::min).ceil().max(0.0);
        let maxx = p.iter().map(|q| q.0).fold(f32::NEG_INFINITY, f32::max).floor().min((self.w - 1) as f32);
        let miny = p.iter().map(|q| q.1).fold(f32::INFINITY, f32::min).ceil().max(0.0);
        let maxy = p.iter().map(|q| q.1).fold(f32::NEG_INFINITY, f32::max).floor().min((self.h - 1) as f32);
        if !(minx <= maxx && miny <= maxy) {
            return;
        }
        for y in miny as usize..=maxy as usize {
            for x in minx as usize..=maxx as usize {
                let q = (x as f64, y as f64);
                let w = [e(p[1], p[2], q), e(p[2], p[0], q), e(p[0], p[1], q)];
                if !(0..3).all(|k| w[k] > 0.0 || (w[k] == 0.0 && own[k])) {
                    continue;
                }
                let l = [w[0] / area, w[1] / area, w[2] / area];
                let ch = |k: usize| (l[0] * c[0][k] as f64 + l[1] * c[1][k] as f64 + l[2] * c[2][k] as f64) as f32;
                let a = ch(3).clamp(0.0, 1.0);
                if a <= 0.0 {
                    continue;
                }
                let (r, g, b) = (ch(0).clamp(0.0, 1.0) * 255.0, ch(1).clamp(0.0, 1.0) * 255.0, ch(2).clamp(0.0, 1.0) * 255.0);
                let d = self.px_mut(x, y);
                let k = 1.0 - a;
                *d = [r * a + d[0] * k, g * a + d[1] * k, b * a + d[2] * k, a + d[3] * k];
            }
        }
    }
}

/// One RV-2 palette colour (straight RGBA, `0..=1`) through the preset's
/// river treatment: [`TerrainAppearance::river_ink`] toward the ink, alpha
/// times [`TerrainAppearance::river_opacity`]. At the default (`ink 0`,
/// `opacity 1`) this returns its input bit for bit, by control flow. Must
/// never touch width: that is [`TerrainAppearance::river_width`]'s, applied in
/// `river_stroke::river_px_width`.
pub fn river_style_color(a: &TerrainAppearance, c: [f32; 4]) -> [f32; 4] {
    let mut o = c;
    if a.river_ink > 0.0 {
        let t = a.river_ink.clamp(0.0, 1.0) as f32;
        let ink = [a.river_ink_r as f32 / 255.0, a.river_ink_g as f32 / 255.0, a.river_ink_b as f32 / 255.0];
        for k in 0..3 {
            o[k] = c[k] + (ink[k] - c[k]) * t;
        }
    }
    if a.river_opacity < 1.0 {
        o[3] = c[3] * a.river_opacity.clamp(0.0, 1.0) as f32;
    }
    o
}

/// The river pixel composited into a land colour — the one stage
/// [`land_color`] adds for a [`RiverLayer`]. `rv` is premultiplied. Must never
/// be applied to a water pixel (the callers only reach it on land).
#[inline]
fn river_over(l: Rgb, rv: [f32; 4]) -> Rgb {
    let k = 1.0 - rv[3] as f64;
    (rv[0] as f64 + l.0 * k, rv[1] as f64 + l.1 * k, rv[2] as f64 + l.2 * k)
}

/// `bakeSingle`/`bakeTiled`'s shared inner loop (reference lines 11975 and
/// 11982) — render one axis-aligned rectangle of an `out_w × out_h` export
/// raster into tightly-packed RGB8.
///
/// The two reference functions differ only in *which* rectangles they ask
/// for: `bakeSingle` walks full-width horizontal strips of one image,
/// `bakeTiled` walks 1024 px tiles and writes each as its own PNG. Both use
/// the identical sample mapping — `sx=(GW-1)/Math.max(1,w-1)`,
/// `sy=(GH-1)/Math.max(1,H-1)`, sampled at `x*sx`, `y*sy` — so it is
/// written once here and the caller chooses the geometry. That mapping puts
/// the first output pixel exactly on cell `0` and the last exactly on cell
/// `GW-1`, which is why an export at the grid's own resolution reproduces
/// the screen image cell-for-cell rather than half a cell off.
///
/// `save_flag` is a loaded save's one-cell river flag ([`save_flag_at`],
/// [`ExportRivers::save_flag`]), tinted as `build_color_texture` tints it --
/// see [`channel_tint`] for why it belongs inside this loop rather than in a
/// pass over the result. `None` renders the terrain alone. **A generated
/// world's rivers are not drawn here**: they are vector strokes, painted over
/// this terrain by [`paint_vector_rivers`] after local contrast has been
/// measured on it (RV-5; the screen measures it on river-free terrain too).
///
/// `rayon`-parallel by output row, on the same determinism argument
/// [`BakeFields::new`] makes.
#[allow(clippy::too_many_arguments)]
pub fn bake_rect(ctx: &RenderCtx, bf: &BakeFields, save_flag: Option<&[u8]>, out_w: usize, out_h: usize, x0: usize, y0: usize, w: usize, h: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; w * h * 3];
    if w == 0 || h == 0 || out_w == 0 || out_h == 0 {
        return bytes;
    }
    let (gw, gh) = (ctx.gw, ctx.gh);
    let ink = save_flag.filter(|m| m.len() >= gw * gh);
    let (sx, sy) = bake_steps(gw, gh, out_w, out_h);
    bytes.par_chunks_mut(w * 3).enumerate().for_each(|(row, out)| {
        let gy = (y0 + row) as f64 * sy;
        let cy = (gy.round().clamp(0.0, (gh - 1) as f64)) as usize;
        for col in 0..w {
            let gx = (x0 + col) as f64 * sx;
            let (r, g, b) = bf.pixel_at(ctx, gx, gy, (sx, sy), None);
            // `build_color_texture`'s own two lines: the same nearest-cell
            // lookup, and the same `1/255` floor below which the tint cannot
            // move a byte anyway.
            let ci = cy * gw + (gx.round().clamp(0.0, (gw - 1) as f64)) as usize;
            // **Land only** -- see `build_color_texture`'s identical guard
            // for why: `bf.pixel` already branched sea vs. land internally
            // (`h < ctx.sea_level`) to pick `sea_color_core` or `land_color`,
            // but that branch never reached this caller, so the ink used to
            // get composited over a water pixel's finished colour with no
            // way to tell. Checked at the same nearest cell `t` itself reads.
            let t = if (ctx.field[ci] as f64) < ctx.sea_level { 0.0 } else { ink.map_or(0.0, |m| save_flag_at(m, ci)) as f64 };
            let (r, g, b) = if t > 1.0 / 255.0 { channel_tint(&ctx.appearance, (r, g, b), t, gx, gy, gw, gh) } else { (r, g, b) };
            let o = col * 3;
            out[o] = (r * 255.0) as u8;
            out[o + 1] = (g * 255.0) as u8;
            out[o + 2] = (b * 255.0) as u8;
        }
    });
    bytes
}

/// **A whole `w x h` export raster, finished** -- `export_raster_png`'s and
/// `export_layer_previews`' four stages in the screen's order: the terrain
/// ([`bake_rect`], with a save's flag), local contrast **measured** on that
/// river-free terrain, the vector rivers painted in ([`paint_vector_rivers`]),
/// then contrast, grade (`influence`, [`build_grade_influence`] at `w x h`) and
/// colour space applied in one quantisation ([`finish_rgb`]).
///
/// With no vector rivers this is exactly the pre-RV-5 `bake_rect` +
/// [`finish_raster`] pair (the measurement reads the same bytes it always
/// read), which is what keeps every monolithic golden where it was.
pub fn bake_and_finish(ctx: &RenderCtx, bf: &BakeFields, rivers: ExportRivers<'_>, w: usize, h: usize, influence: &[f32], space: ColorSpace) -> Vec<u8> {
    let mut px = bake_rect(ctx, bf, rivers.save_flag, w, h, 0, 0, w, h);
    let lc = local_contrast_rows(&ctx.appearance, &px, w, h, 0, h, ctx.world);
    if let Some(v) = rivers.vector {
        paint_vector_rivers(ctx, bf, v, &mut px, w, h, 0, 0, w, h);
    }
    finish_rgb(&mut px, lc.as_ref().map(|l| |i| l.delta(i)), Some((&ctx.appearance, influence)), space);
    px
}

/// Grid cells between neighbouring pixels of an `out_w x out_h` export:
/// `bakeSingle`'s `sx=(GW-1)/Math.max(1,w-1)` and its `sy`. `out_w.max(2) - 1`
/// is `Math.max(1, w-1)` without the `usize` underflow at `out_w == 1`. One
/// function so [`bake_rect`] and [`paint_vector_rivers`] cannot place a
/// pixel differently.
fn bake_steps(gw: usize, gh: usize, out_w: usize, out_h: usize) -> (f64, f64) {
    ((gw.max(1) - 1) as f64 / (out_w.max(2) - 1) as f64, (gh.max(1) - 1) as f64 / (out_h.max(2) - 1) as f64)
}

/// **RV-5: the vector rivers, painted into a finished bake rectangle.**
/// `px` is [`bake_rect`]'s output for the same `(out_w, out_h, x0, y0, w,
/// h)`; `vector` rasterizes the network for exactly that rectangle at the
/// export's own resolution ([`RasterRect`], placed by [`bake_steps`] as
/// `bake_rect` places its pixels), and every pixel a stroke reached is
/// re-rendered with the river composited inside [`land_color`] -- the slot a
/// deep-zoom tile composites its own layer in, so the preset's Painter
/// block, paper and frame reach the river, and water (decided first, with the
/// smooth shore) stays above it. Returns the pixels repainted.
///
/// Why a second pass and not a `river` argument to `bake_rect`: the callers
/// measure local contrast on the terrain **before** this runs, as
/// `build_color_texture` measures it on the river-free map -- a river in
/// that measurement rings every river with a halo at the blur's radius.
/// Only the river's own pixels are rendered twice, so the cost is the
/// river's share of the raster, not a second bake.
///
/// Memory: one sparse [`RiverLayer`] for the rectangle (16 KB per 32x32 block
/// a stroke touches), dropped on return -- a band's, never the whole image's.
/// Must never paint a pixel no stroke reached.
#[allow(clippy::too_many_arguments)]
pub fn paint_vector_rivers(ctx: &RenderCtx, bf: &BakeFields, vector: VectorRivers<'_>, px: &mut [u8], out_w: usize, out_h: usize, x0: usize, y0: usize, w: usize, h: usize) -> usize {
    if w == 0 || h == 0 || out_w == 0 || out_h == 0 || px.len() < w * h * 3 {
        return 0;
    }
    let (sx, sy) = bake_steps(ctx.gw, ctx.gh, out_w, out_h);
    let layer = vector(&ctx.appearance, RasterRect { bx: x0 as f64 * sx, by: y0 as f64 * sy, cx: sx, cy: sy, w, h });
    if layer.width() != w || layer.height() != h {
        return 0;
    }
    px[..w * h * 3]
        .par_chunks_mut(w * 3)
        .enumerate()
        .map(|(row, out)| {
            let gy = (y0 + row) as f64 * sy;
            let mut n = 0usize;
            for col in 0..w {
                let Some(rv) = layer.at(col, row) else { continue };
                let (r, g, b) = bf.pixel_at(ctx, (x0 + col) as f64 * sx, gy, (sx, sy), Some(rv));
                let o = col * 3;
                out[o] = (r * 255.0) as u8;
                out[o + 1] = (g * 255.0) as u8;
                out[o + 2] = (b * 255.0) as u8;
                n += 1;
            }
            n
        })
        .sum()
}

// ===========================================================================
// The banded export (`EXPORT_SCOPE.md` §4, milestone E1)
// ===========================================================================
//
// A 16K/32K raster does not fit whole through the finishing passes, so it is
// rendered in full-width horizontal bands. Of the four stages
// (`bake_rect` → `apply_local_contrast` → `build_grade_influence` →
// `apply_color_grade`) only local contrast reads a neighbourhood, and only
// ±radius rows of it vertically, so a band that renders that many apron rows
// above and below itself — clipped at the image edges, where the whole-raster
// blur clamps too — and discards them afterwards is **byte-identical** to the
// monolithic render (§4.2 has the exactness argument; `tests/export_bands.rs`
// measures it). The shipped `export_raster_png` path does not go through here.

/// How an `w × h` export is cut into bands. Every field is derived from the
/// **full** `(w, h)`, so the plan is the same whatever the band size — which is
/// what lets the identity measured at a small width stand for 32K.
///
/// A single band always has a **zero** apron: that is the monolithic render,
/// not a banded approximation of it.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportBandPlan {
    pub w: usize,
    pub h: usize,
    /// Output rows per band; the last band may be shorter.
    pub rows_per_band: usize,
    /// Context rows rendered each side of a band before clipping at the image
    /// edge: the local-contrast radius at the full `(w, h)`, or `0`.
    pub apron: usize,
}

/// One band of an [`ExportBandPlan`]: output rows `y0 .. y0 + rows`, rendered
/// with `top`/`bottom` apron rows (already clipped at the image edges).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportBand {
    pub y0: usize,
    pub rows: usize,
    pub top: usize,
    pub bottom: usize,
}

#[allow(dead_code)]
impl ExportBandPlan {
    /// Bands of `rows` output rows each (clamped to `1..=h`).
    pub fn with_rows(a: &TerrainAppearance, w: usize, h: usize, rows: usize) -> Self {
        let rows = rows.clamp(1, h.max(1));
        let apron = if rows >= h || a.local_contrast <= 0.0 || w == 0 { 0 } else { local_contrast_radius(a, w, h) as usize };
        ExportBandPlan { w, h, rows_per_band: rows, apron }
    }

    /// The tallest bands whose rendered height (rows plus both aprons) fits in
    /// `budget_px` output pixels. A budget of the whole raster or more is one
    /// band with no apron. Floored at one row per band, so a budget smaller
    /// than `w · (2 · apron + 1)` is exceeded rather than refused — refusing is
    /// the caller's gate (`export_raster.rs`'s affordability check).
    pub fn for_budget(a: &TerrainAppearance, w: usize, h: usize, budget_px: u64) -> Self {
        let fit = (budget_px / w.max(1) as u64).min(usize::MAX as u64) as usize;
        if fit >= h {
            return Self::with_rows(a, w, h, h);
        }
        let apron = Self::with_rows(a, w, h, 1).apron;
        Self::with_rows(a, w, h, fit.saturating_sub(2 * apron))
    }

    pub fn band_count(&self) -> usize {
        self.h.div_ceil(self.rows_per_band)
    }

    pub fn bands(&self) -> impl Iterator<Item = ExportBand> + '_ {
        (0..self.band_count()).map(move |i| {
            let y0 = i * self.rows_per_band;
            let rows = self.rows_per_band.min(self.h - y0);
            ExportBand { y0, rows, top: self.apron.min(y0), bottom: self.apron.min(self.h - y0 - rows) }
        })
    }
}

/// Render one band of a banded export: the same four stages
/// `export_raster_png` runs over the whole raster, over `band`'s rows plus its
/// apron, returning the band's own `band.rows × plan.w` RGB8 rows.
///
/// `grade_cells` is [`build_grade_influence_cells`], built **once** per export
/// and shared by every band. `rivers` are the same rivers the whole-raster
/// path is handed; `ctx.appearance` and `ctx.world` are the appearance and
/// wrap it is handed.
///
/// RV-5: the vector rivers are rasterized for **this band's own rows only**
/// ([`paint_vector_rivers`]) -- after local contrast is measured on the
/// river-free terrain of band + apron, and after the apron is discarded, so
/// no river pixel is rendered for a row that never reaches the file and the
/// river layer's memory is bounded by one band, whatever the export width.
#[allow(dead_code)]
pub fn bake_export_band(ctx: &RenderCtx, bf: &BakeFields, rivers: ExportRivers<'_>, plan: &ExportBandPlan, band: ExportBand, grade_cells: &[f32]) -> Vec<u8> {
    let (w, h) = (plan.w, plan.h);
    let ay0 = band.y0 - band.top;
    let ah = band.rows + band.top + band.bottom;
    let mut px = bake_rect(ctx, bf, rivers.save_flag, w, h, 0, ay0, w, ah);
    // Measured over band + apron, applied to the band alone: the apron rows
    // are discarded, so correcting them (as the pre-Ruling-AN pass did) was
    // work that never reached a file. `delta`'s index is apron-relative.
    let lc = local_contrast_rows(&ctx.appearance, &px, w, h, ay0, ah, ctx.world);
    px.drain(..band.top * w * 3);
    px.truncate(band.rows * w * 3);
    if let Some(v) = rivers.vector {
        paint_vector_rivers(ctx, bf, v, &mut px, w, h, 0, band.y0, w, band.rows);
    }
    let inf = grade_influence_rows(grade_cells, ctx.gw, ctx.gh, w, h, band.y0, band.rows);
    let skip = band.top * w;
    // The working space, never the display's: `export_raster.rs`'s module doc.
    finish_rgb(&mut px, lc.as_ref().map(|l| move |i| l.delta(i + skip)), Some((&ctx.appearance, &inf)), ColorSpace::Srgb);
    px
}

/// The river-channel tint `build_color_texture` composites over its own
/// finished raster, transcribed for the export raster.
///
/// # Why the export needs it at all
///
/// Found by measuring, not by reading. An export at the grid's own
/// resolution came back with 291 815 of 8 060 928 bytes different from the
/// on-screen map, worst channel delta 132 — and every differing byte was a
/// river. `build_color_texture`'s own doc comment says what this tint is: a
/// stand-in for the reference's vector `drawRiverWays` overlay, which is
/// what keeps `MVP_SCOPE.md`'s "rivers visible" satisfied in this port. An
/// export without it is a map of a world with no rivers in it, which is not
/// the map the user was looking at when they pressed the button.
///
/// # Before quantization, not after
///
/// Applied here, on `pixel`'s `f64` colour, rather than as a second pass
/// over the finished bytes. A byte pass cannot be bit-identical to the
/// screen: `build_color_texture` tints in `f64` and quantizes *once*, so
/// re-deriving `r` from an already-truncated byte shifts the blue channel by
/// a level for half of all inputs (`b*0.5 + 0.45` lands on a `.75` fraction,
/// where `floor` stops commuting with the halving). The order is the screen
/// loop's order exactly — after `apply_border`, before
/// [`apply_local_contrast`].
///
/// # `t` is the ink, and it is not a flag
///
/// **Corrected 2026-09-06.** This function used to be entered on `mask[i] !=
/// 0` and tint at full strength, which was right only while the screen did
/// the same. Since `58dd5b2` the screen composites `r + (fr - r) * ink` with
/// `RiverInk`'s stamped disc value (retired by RV-5), so a channel cell whose stamp reads
/// `0.45` gets 45 % of the tint on screen and used to get 100 % of it in the
/// PNG. `t` is that same ink, and the three lines below are
/// `build_color_texture`'s three lines, in its order, so the two cannot
/// disagree by construction rather than by review.
///
/// **Since RV-5 (2026-09-28) the only ink left is a loaded save's flag**
/// ([`save_flag_at`]): the stamp (`RiverInk::Stamped`, the enum's other
/// arm) is retired, and a generated world's export draws RV-2's vector
/// strokes instead ([`paint_vector_rivers`]).
///
/// # Nearest cell, not a bilinear sample
///
/// Kept as nearest, and the reason changed with the mask. The old reason was
/// that the mask was categorical (`chan` a flag, `strahler_order` a stream
/// order); the stamp is not — it is a continuous parabolic disc. What still
/// holds is the property the choice was made for: nearest-cell keeps a river
/// the same width in **world** terms at every export resolution, and makes an
/// export at the grid's own width land on exactly the cell the screen read.
/// Interpolating would soften the stamp's own edge a second time, more at 8K
/// than at 2K — and `the_river_tint_keeps_its_world_width_at_every_resolution`
/// is what would notice.
fn channel_tint(a: &TerrainAppearance, c: (f64, f64, f64), t: f64, gx: f64, gy: f64, gw: usize, gh: usize) -> (f64, f64, f64) {
    let cover = border_cover_f(a, gx, gy, gw, gh);
    if cover >= 1.0 {
        return c;
    }
    let (r, g, b) = c;
    let (fr, fg, fb) = (r * 0.5, (g * 0.5 + 0.3).min(1.0), (b * 0.5 + 0.45).min(1.0));
    let (tr, tg, tb) = (r + (fr - r) * t, g + (fg - g) * t, b + (fb - b) * t);
    (tr + (r - tr) * cover, tg + (g - tg) * cover, tb + (b - tb) * cover)
}

// ===========================================================================
// The LOD / atlas biome tile — `renderBiomeTileRGBA`
// (reference HTML `Cartalith Gen1 v2.11.html` lines 11668-11779)
// ===========================================================================
//
// `LOD_DETAIL_SCOPE.md` milestone **LOD-D1**, Ruling K step 1a. The reference
// picks its tile coloriser off the view mode (`_lodBuildTileRGBA`, 11183:
// `biome ? renderBiomeTileRGBA : renderHeightTileRGBA`), so the height ramp is
// what *Relief* mode shows and the full `landColorCore` look is what the
// default *Biome* mode shows. This port had only the height-ramp half
// (`cartalith_terrain::tile_render`), which is why `lod_bridge` composites a
// scalar *shade ratio* over `map_view`'s grid-resolution colours instead of
// carrying colour of its own. This is the other half.
//
// # Where it lives, and why not in `cartalith-terrain` beside its sibling
//
// `render_height_tile_rgba` is a height formula start to finish — a
// hypsometric ramp times a normal-from-height Lambert term — so it belongs in
// the terrain crate. `renderBiomeTileRGBA` is not: it is `landColorCore`,
// `seaColorCore`, `materialWeights`, the six material palettes, the paint
// overrides, the splat channels and the SDF appliers, every one of which lives
// in this file and most of which are private to it. Putting the tile path
// anywhere else would mean making eleven private colour helpers public to
// export one function, which is how two renderers end up drifting apart
// (`RiverInk`'s own doc records what that cost last time).
// `LOD_DETAIL_SCOPE.md` says `render.rs` for the same reason.
//
// # What a tile computes for itself, and what it samples
//
// The split is the reference's, stated in its own header comment at 11662:
// *"Height + slope + macro/meso hillshade come from the tile's own amplified
// heightmap; temperature/moisture/flow/aspect/curvature are sampled from the
// coarse climate fields at the tile's world coordinates; noise + vignette +
// splat UV use world coords so adjacent tiles stay seamless and match the main
// map."*
//
// - **From the tile's own height:** the macro normal (through the v1.29
//   `edge_*` extrapolators, shared with `render_height_tile_rgba` rather than
//   re-derived), the meso normal at the `ms` step, the coarse-unit slope, the
//   relative height `r`, the hachure gradient, the crest field and the coast
//   SDF.
// - **Bilinear at world coordinates** (`sample_arr`, the reference's
//   `sampleArr`): temperature, moisture, flow, the shared sea height and sea
//   shade, AO, hydrological wetness, the local-contrast detail band and the
//   grade influence; plus `aspect_factor_f` / `curvature_at_f` /
//   `vignette_at_f`, which are `sample_arr` underneath.
// - **Nearest cell:** lithology (through `litho_at_f`, which carries this
//   port's coherent jitter), the paint overrides, the lake mask and pooled
//   lake surface, and the river ink.
//
// # Three places this deliberately does not follow the reference, and why
//
// Read `LOD_DETAIL_SCOPE.md`'s owner question 1 before changing any of them.
// Its stated default is *"both"* — golden against the reference under
// `js_reference()`, and identical to this port's own screen under the shipped
// look. **Those two are not simultaneously satisfiable**, and these are the
// three places where they part company. Each is inert under `js_reference()`,
// so the golden is unaffected by all three; what they decide is what a tile
// looks like beside the *shipped* map at LOD entry, which is LOD-D2's
// acceptance bar (`mean |dL*| <= 2.0` with the layer shown vs hidden).
//
// 1. **Ambient occlusion is sampled from the grid, not rebuilt per tile.**
//    The reference blurs the tile's own heightmap twice at
//    `rad = max(2, min(W,H)/24)` tile pixels and applies its `aoMul`
//    (`AO_GAIN = 12`, `AO_MAX = 0.5`, reference 8032). This port's screen AO is
//    [`build_ao`], a **different algorithm** — a two-scale cavity signal, each
//    scale normalised by its own RMS over land. Re-implementing the
//    reference's single-scale form per tile would give a tile an occlusion
//    term its own map does not have; running *this* port's form per tile is
//    worse still, because the RMS is taken over the whole field and a
//    per-tile RMS would seam at every tile boundary. So the tile samples
//    `ctx.ao`, exactly as [`BakeFields::pixel`] does, which is bit-identical
//    to `cell_color` at an integer cell. The cost is real and bounded: AO is a
//    broad cavity term at `ao_radius_frac` of the map width (tens of cells),
//    so a bilinear read of it is visually indistinguishable from recomputing
//    it — what is lost is a *scale-aware* radius, which is
//    `LOD_DETAIL_SCOPE.md` LOD-D5's own row (*"Radii. Crest and AO radii are
//    set in ground units"*), not this milestone's.
// 2. **The meso shade is the reference's, including its missing normaliser.**
//    The main map's `shadeFactor2` (7681) divides the exaggeration by its step
//    — its own comment says *"/s keeps slope magnitude consistent with 1-px
//    sample"* — and the tile's meso block (11742) does not. The tile's meso
//    gradient is therefore about `ms` times stronger than the map's at the
//    same ground scale. `MISTAKES.md`: *"The reference's errors are part of
//    the contract"*, so it is transcribed as written and measured rather than
//    quietly corrected; `tests/golden_parity_tile_biome.rs` attributes the screen-identity
//    residual to it by holding the term equal on both sides.
//
//    **Not to be confused with the DCC line's v2.25 `tileShadeExag`**, which
//    this port does follow in the shipped look (Ruling AP; see
//    [`tile_shade_exag`] and [`TerrainAppearance::tile_shade_exag_scaled`]).
//    That one scales `exag` by tile pixels per coarse cell and is exactly `1`
//    at one pixel per cell or coarser, so it leaves this LOD-entry asymmetry
//    as it was and removes only the flattening *with zoom*:
//    `tests/tile_shade_exag.rs` measures both.
// 3. **Lakes.** The tile draws above-sea lakes with the v1.05 `_lakeFill`
//    shoreline (11717-11740); `cell_color` draws them per cell, the v0.103
//    stamp (8580), once a caller attaches the classification with
//    [`RenderCtx::with_lakes`]. That block's closing *"The BASE per-cell map
//    loop is untouched (default render identical)"* means v1.05 left the base
//    loop's own v0.103 stamp alone -- NOT that the base draws no lakes, which
//    is how this note read until 2026-09-23 and why the screen lacked them.
//    A context without `with_lakes` (every golden fixture) still draws none.
//
// # The port-only stages, in `cell_color`'s order
//
// The reference's tile stops at `applyCoastRiverSDFv`. This port's screen does
// not: `cell_color` continues with the wave contours, the parchment and the
// plate frame, `build_color_texture` adds the river ink, and three whole-raster
// passes follow it. `LOD_DETAIL_SCOPE.md`: *"The port-only per-pixel stages
// (paper, border, waves, stipple, grade from a sampled influence field, colour
// space) are applied in `cell_color`'s own order, so the tile matches the
// screen."* Stipple lives inside [`land_color`] and comes for free; the rest
// are below, in that order, and every one of them is `0.0` in `js_reference()`.
//
// **The river ink is included although the scope's list omits it**, because
// `RiverInk`'s doc comment (retired by RV-5; see [`save_flag_at`]) recorded what omitting it costs: the
// screen drew a stamped disc while the export drew a one-cell flag, measured
// at 199 909 differing bytes. A tile with no river ink is a tile of a world
// with no rivers in it. It is an `Option` on [`TileFields`], so a caller that
// has no mask gets the terrain alone rather than a guess.
//
// `apply_local_contrast` is the one stage that cannot be evaluated per tile.
// It reads a *neighbourhood of the finished colour*, and a tile-local
// neighbourhood truncates at the tile border — which is precisely the v1.29
// seam, in a second subsystem (the reference's own note at 11700 is the same
// finding for its sea blur). So [`TileFields`] carries the grid's detail band
// and the tile samples it, which two adjacent tiles cannot disagree about.

// ---------------------------------------------------------------------------
// `LOD_DETAIL_SCOPE.md` LOD-D4 — ice and snow from fields that already exist
//
// The milestone's own framing is that **nothing here is a new simulation**:
// every input already exists and is already computed by a shipped pass. What
// was missing was a *field* — the scope's gap table says so in one line,
// *"No ice or glacier field exists anywhere; the kernel keeps no mask"* — and
// the three stages below are that field, a sub-cell temperature, and a colour.
//
// 1. [`build_glacier_potential`]: `glacial_kernel`'s **own gate**, verbatim
//    (`h >= sea + (1 - sea) * snowline` and `T < 0`), weighted by the
//    catchment the cell drains and blurred one cell.
// 2. [`TileCryo`]: `cartalith_climate::compute_temperature`'s **own lapse
//    relation**, applied to the difference between the tile's height and the
//    grid's, so snow follows sub-cell relief instead of stopping at the cell
//    boundary.
// 3. [`land_color`]'s ice block: the potential raises the snow fraction and
//    tints it toward [`TerrainAppearance::snow_glac`], the glacier-ice ramp
//    this appearance has carried since milestone 1 and which nothing but a
//    `T < -12` test has ever reached.
//
// **Why the discharge weight is keyed on catchment km² and not on flow cells.**
// `RC_ENGINE_CHANGES.md` §6k measured the alternative and ruled against it: an
// `order >= 3` label spans 6 417 km² on an 800 km map and 1 429 009 km² on a
// 40 000 km one, while catchment area is resolution-free (483 vs 482 km²
// across a 4x cell-count change). A glacier that appeared at one map width and
// not another would be the same defect in a third subsystem, so the two
// thresholds below are areas and the caller converts.
// ---------------------------------------------------------------------------

/// The catchment at which a cirque starts to hold ice, in km².
///
/// Sized from the ground the feature is named after rather than from the
/// render: the small alpine cirque glaciers around the Aletsch basin sit in
/// the 0.5-2 km² class, and below that a hollow collects snow but not a body
/// of ice. It is the **lower** edge of a `smoothstep`, so it is where the
/// potential leaves zero, not where ice is drawn.
const GLACIER_CIRQUE_KM2: f64 = 0.5;

/// The catchment at which the potential is saturated — a valley tongue rather
/// than a cirque. The Aletsch's own catchment is of the order of 200 km²;
/// `8.0` is two orders below that on purpose, because this is the point at
/// which ice *fills* its trough, not the point at which it is the largest
/// glacier in the Alps.
const GLACIER_TONGUE_KM2: f64 = 8.0;

/// The one-cell blur the scope asks for (*"then blurred one cell so trough
/// floors read as tongues"*). Named so it can be mutated; at `0` the field is
/// the raw per-cell gate and tongues break up at the cell lattice.
const GLACIER_BLUR_CELLS: i64 = 1;

/// LOD-D4 stage 1 — **glacier potential** at grid resolution, `0.0..=1.0`.
///
/// `glacial_kernel`'s own gate (`cartalith_erosion::passes::glacial_kernel`:
/// *"A cell erodes only where it is both above the snowline and below
/// freezing"*), weighted by the catchment that cell drains and blurred by
/// [`GLACIER_BLUR_CELLS`].
///
/// # This is the kernel's gate, not a model of it
///
/// `snowline` is `ErosionPassParams::glacial_snowline` — the same number the
/// carving pass used — and `sea_level` the same sea level, so the ice is drawn
/// exactly where the trough was cut. The scope's non-goal is explicit: *"any
/// change to `glacial_kernel` or its golden"*, and there is none. This reads
/// two fields the kernel also read and writes a third the kernel never kept.
///
/// # `None` flow returns an empty field, deliberately
///
/// A **loaded save stores no flow accumulation** (`SAVEFILE_COMPAT.md`, and
/// `RenderCtx::flow` is already `None` there for the same reason). Without it
/// there is no catchment, so there is no honest weight — and the scope says
/// what to do: *"On a loaded save, which has no flow, it falls back to snow
/// only"*. An empty field is that fallback, and it is the absence rather than
/// a plausible substitute (`MISTAKES.md`: never encode "no value" as a value).
/// Every consumer tests the length, not a flag.
#[allow(clippy::too_many_arguments)]
pub fn build_glacier_potential(
    field: &[f32],
    temperature: &[f32],
    flow: Option<&[f32]>,
    gw: usize,
    gh: usize,
    sea_level: f64,
    snowline: f64,
    km_per_cell: f64,
    world: bool,
) -> Vec<f32> {
    let n = gw * gh;
    let flow = match flow {
        Some(f) if n > 0 && f.len() >= n && field.len() >= n && temperature.len() >= n => f,
        _ => return Vec::new(),
    };
    if !(km_per_cell.is_finite() && km_per_cell > 0.0) {
        return Vec::new();
    }
    // The kernel's own `snow_el`, spelled the same way it spells it.
    let snow_el = sea_level + (1.0 - sea_level) * snowline;
    let cell_km2 = km_per_cell * km_per_cell;
    let mut p = vec![0f32; n];
    p.par_iter_mut().enumerate().for_each(|(i, v)| {
        if (field[i] as f64) < snow_el || temperature[i] as f64 >= 0.0 {
            return;
        }
        // `flow` is in cells of upslope contributing area; one cell is
        // `cell_km2` of ground, so this is the catchment in km².
        let km2 = flow[i] as f64 * cell_km2;
        *v = smoothstep(GLACIER_CIRQUE_KM2, GLACIER_TONGUE_KM2, km2) as f32;
    });
    if GLACIER_BLUR_CELLS > 0 { blur_once(&p, gw, gh, GLACIER_BLUR_CELLS, world) } else { p }
}

/// LOD-D4 stage 2 — the **lapse relation** a tile needs to give its own
/// height a temperature, carried beside the tile fields because
/// `RenderCtx` holds a temperature *raster* and not the relation that
/// produced it.
///
/// `cartalith_climate::compute_temperature` is
/// `t_sea - lapse_rate * g * (above_sea * mpu / 1000)`, where `mpu` is
/// `meters_per_unit(peak_m, sea_level)`. The grid's raster already carries
/// that term for the **coarse** height, so a tile only owes the *difference*:
///
/// ```text
/// t_tile = t_sampled - lapse_rate * g * (h_tile - h_coarse) * mpu / 1000
/// ```
///
/// which is `LOD_DETAIL_SCOPE.md` LOD-D4 stage 2 written out. At one tile
/// pixel per grid cell `h_tile == h_coarse` and the correction is exactly
/// zero, which is why `golden_parity_tile_biome.rs`'s screen-identity check
/// is unaffected by this milestone rather than merely close to unaffected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileCryo {
    /// `state.climate.lapseRate`, °C per km.
    pub lapse_rate: f64,
    /// `state.planet.g`. The reference scales the lapse rate by gravity and
    /// so does this.
    pub g: f64,
    /// `meters_per_unit(peak_m, sea_level)` — metres of real elevation per
    /// unit of normalised height. Kept as metres, not km, so the three
    /// numbers here are the three the climate pass itself holds.
    pub meters_per_unit: f64,
}

impl TileCryo {
    /// The temperature correction for a sub-cell height difference `dh`, in
    /// normalised height units. Positive `dh` (the tile is higher than the
    /// cell it sits in) returns a positive number, which the caller
    /// **subtracts**.
    pub fn delta_t(&self, dh: f64) -> f64 {
        self.lapse_rate * self.g * (dh * self.meters_per_unit / 1000.0)
    }
}

/// The coarse-coordinate rectangle a tile covers — `bounds` in
/// `renderBiomeTileRGBA(tile, W, H, bounds)`, the reference's `{x, y, w, h}`
/// in grid cells.
///
/// `x`/`y` are the world (grid) coordinate of the tile's **first pixel
/// centre**, and `w`/`h` the span from the first pixel centre to the last —
/// so a tile whose `bounds` is `(0, 0, gw - 1, gh - 1)` at `W = gw`, `H = gh`
/// samples exactly the integer cells `0 ..= gw-1`. That is the reference's own
/// convention (`cx = bounds.w / max(1, W - 1)`), and it is what makes the
/// screen-identity check in `tests/golden_parity_tile_biome.rs` expressible at all.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileBounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

// ---------------------------------------------------------------------------
// LOD-D5 — scale-aware shading and hydrology that resolves
// (`LOD_DETAIL_SCOPE.md`, *"the balance of shading scales and the drainage
// detail change continuously with ground scale"*).
//
// # The one judgment call, stated rather than taken silently
//
// The scope writes every stage here as a curve over **km per pixel**; this
// implementation parameterises them by **coarse cells per tile pixel**, and
// the two are the same curve whenever the anchor is the map's own resolution:
// `km_per_px = cells_per_px · km_per_cell`, so a curve in `km_per_px` that is
// the identity at `km_per_cell` is exactly a curve in `cells_per_px` that is
// the identity at `1`. They differ only if the anchor is an absolute ground
// size (*"one kilometre per pixel"*), which would make the shading of a
// 40 000 km world differ from an 800 km world's at the same pyramid level and
// the same grid — and, decisively, would make the zoom-1 identity the scope
// itself requires (*"a test holds the zoom-1 weights byte-identical to
// today"*) unreachable. Cells per pixel is therefore the parameterisation,
// and the consequence of the choice is that these stages respond to *how far
// past the simulation grid the view has zoomed* and not to world size.
//
// A second consequence, recorded so it is not mistaken for an oversight: none
// of this needs a map width, so `RenderCtx` gains no field and
// `with_map_scale` is unchanged.
//
// # What was checked and NOT changed
//
// **AO's radius is already in ground units.** `TerrainAppearance::
// ao_radius_frac` is a fraction of grid width and `build_ao` multiplies it by
// `gw`, and the tile path reads the finished grid field through
// `sample_arr(&ctx.ao, ...)` — so a tile's occlusion already has a fixed
// ground radius at every zoom, and the scope's *"crest and AO radii are set
// in ground units"* is half already true. Only the crest needed a step.
// ---------------------------------------------------------------------------

/// LOD-D5 — how many octaves past the simulation grid the scale curve takes
/// to saturate.
///
/// **`6` is `add_zoom_detail`'s own cap**, not a tuning constant:
/// `cartalith_terrain::amplify::add_zoom_detail` adds `min(6, z − z_base)`
/// extra octaves, so from six levels past `z_base` onward a tile's height
/// stops gaining detail and there is nothing further for the shading to
/// rebalance toward. Saturating the curve at the same depth is what keeps the
/// two from disagreeing about where "as close as this gets" is.
const DETAIL_SCALE_OCTAVES: f64 = 6.0;

/// LOD-D5 — `add_zoom_detail`'s own `Math.min(6, …)` octave cap, quoted here
/// because [`zoom_detail_peak_amplitude`] has to walk the same schedule.
const ZOOM_DETAIL_MAX_OCTAVES: usize = 6;

/// LOD-D5 — the fraction of the **macro** band's weight handed to the meso
/// and micro bands at full saturation and full strength.
///
/// At the shipped `0.40/0.40/0.20` this takes macro to `0.20` and gives the
/// other two `0.20` split in their own existing ratio (`0.533/0.267`), so the
/// three still sum to what they summed to and the light curve's input keeps
/// its range. Half rather than all, because a deep-zoom view with no macro
/// band at all loses the landform the fine bands are detail *of*.
const DETAIL_SHIFT_MAX: f64 = 0.5;

/// LOD-D5 — how far below its theoretical peak the tile's relief residual is
/// taken to be "full scale" for the micro band.
///
/// [`zoom_detail_peak_amplitude`] is the amplitude the octave schedule could
/// reach if every octave landed at its extreme simultaneously and the relief
/// taper were `1`, which no real pixel does. Dividing by this is what stops
/// the micro band from being a barely-visible wobble around `0.5`. **It is a
/// global constant and not a per-tile normalisation on purpose**: two
/// adjacent tiles normalised by their own residual spreads would shade the
/// same ground differently and the difference would land exactly on their
/// shared edge, which is the seam metric LOD-D2 left open.
const MICRO_RESIDUAL_HEADROOM: f64 = 4.0;

/// LOD-D5 — the exponent on cells-per-pixel in the per-tile river threshold.
///
/// **Two, because the threshold is an area.** `river_flow_thresh` is
/// `gw·gh·0.0004 / (detail_k · ease)` in units of coarse cells of catchment,
/// which in km² is `0.0004 · map_w · map_h / (detail_k · ease)` — independent
/// of the grid, and the resolution-free *area* threshold
/// `RC_ENGINE_CHANGES.md` §6k argues every hydrological threshold should be.
/// Keeping the drawn channel's catchment a fixed multiple of the ground **one
/// drawn pixel covers** therefore means scaling the threshold by the pixel's
/// own ground area, which is `cells_per_px²`.
const RIVER_THRESH_SCALE_EXP: f64 = 2.0;

/// LOD-D5 — the floor on the per-tile river threshold, in coarse cells of
/// accumulated catchment.
///
/// `compute_flow` seeds one unit per cell, so a cell that drains only itself
/// carries `1.0` and a threshold at or below that draws **every land pixel**
/// as a river. The floor is four rather than one because a D8 tree's
/// finest branches are single cells: four cells of catchment is the smallest
/// upstream area that can be a *line* in the sampled field rather than a
/// point, and below it the bilinear read of one coarse cell is a blob.
///
/// It is also the point past which zooming reveals nothing further — the
/// sampled flow field is the coarse grid's, and no depth of zoom puts a
/// channel into it that the coarse pass did not accumulate. That limit is
/// structural, and `cartalith_hydrology::tile` (EF-1) is the mechanism that
/// would lift it; wiring it is out of this milestone's scope by its own
/// non-goal (*"channels the flow field does not carry"*).
const RIVER_TILE_THRESH_FLOOR: f64 = 4.0;

/// LOD-D5 — the scale curve's argument: **octaves past the simulation grid**,
/// `−log2(cells_per_px)`, normalised by [`DETAIL_SCALE_OCTAVES`] and clamped
/// to `[−1, 1]`.
///
/// `0.0` at one tile pixel per coarse cell — exactly, since `log2(1.0)` is
/// exactly zero — which is what makes every stage below the identity there.
/// Positive is zoomed **in** (a pixel covers less than a cell), negative is
/// zoomed out. A non-finite or non-positive argument returns `0.0` rather
/// than a plausible-looking number.
pub(crate) fn detail_scale_u(cells_per_px: f64) -> f64 {
    if !(cells_per_px.is_finite() && cells_per_px > 0.0) {
        return 0.0;
    }
    (-cells_per_px.log2() / DETAIL_SCALE_OCTAVES).clamp(-1.0, 1.0)
}

/// LOD-D5 stage 1 — the three shading-band weights at scale `u`
/// ([`detail_scale_u`]).
///
/// Weight is moved **out of** the macro band as the view comes in and back
/// into it as the view pulls out, and the amount moved is redistributed
/// between meso and micro **in their own existing ratio** — so the only thing
/// this curve decides is how much leaves macro, and the shipped balance
/// between the two fine bands is preserved rather than re-invented. The sum
/// is conserved wherever no clamp bites, which keeps `sh_combined` on the same
/// range the light curve was tuned against.
///
/// Returns the appearance's own three weights **bit-identically** whenever the
/// transfer fraction is zero — which is `detail_scale_strength == 0.0`
/// (`js_reference()`), `u == 0.0` (grid resolution), or a degenerate
/// `meso + micro`.
pub(crate) fn scaled_detail_weights(a: &TerrainAppearance, u: f64) -> (f64, f64, f64) {
    let (mac, mes, mic) = (a.detail_macro_weight, a.detail_meso_weight, a.detail_micro_weight);
    let f = a.detail_scale_strength * DETAIL_SHIFT_MAX * u;
    let fine = mes + mic;
    if f == 0.0 || fine <= 0.0 {
        return (mac, mes, mic);
    }
    let moved = mac * f;
    ((mac - moved).max(0.0), (mes + moved * (mes / fine)).max(0.0), (mic + moved * (mic / fine)).max(0.0))
}

/// LOD-D5 stage 2 — the peak height `add_zoom_detail` can add, from its own
/// schedule.
///
/// `amp = detail_amp · 0.6 · zoom_detail_k`, halved because `fbm − 0.5` spans
/// `±0.5`, geometric at `0.6` over [`ZOOM_DETAIL_MAX_OCTAVES`] octaves. It is
/// derived from `AmplifyOpts::default()` rather than written as a literal, so
/// a change to the detail amplitude moves the micro band's normaliser with it
/// instead of leaving a stale number behind.
pub(crate) fn zoom_detail_peak_amplitude() -> f64 {
    let o = cartalith_terrain::amplify::AmplifyOpts::default();
    let mut amp = o.detail_amp * 0.6 * o.zoom_detail_k;
    let mut peak = 0.0;
    for _ in 0..ZOOM_DETAIL_MAX_OCTAVES {
        peak += amp * 0.5;
        amp *= 0.6;
    }
    peak
}

/// LOD-D5 stage 2 — the micro band's `n` from the tile's own relief residual,
/// on the same `[0, 1]` scale `land_color`'s value noise produces.
///
/// `residual` is the tile's amplified height minus the bilinear read of the
/// coarse field at the same world coordinate: **everything the tile added
/// beyond the simulation grid** — `refine_tile`'s own fixed-frequency detail
/// as well as `add_zoom_detail`'s progressive octaves. It is deliberately not
/// claimed to be `add_zoom_detail`'s output alone: separating the two would
/// mean synthesising the tile twice, and both terms are relief the grid does
/// not carry, which is the property the micro band wants.
///
/// `full_scale` is the residual at which the band **saturates** — `±full_scale`
/// maps to `1.0` and `0.0`, the two ends `vnoise` itself can reach. Written
/// that way round, rather than as a half-range, so the name and the number
/// agree: a caller reading "full scale" gets the residual that produces a
/// full-strength micro band.
///
/// `0.5` — the neutral value, where `(n − 0.5)` is zero — for a zero residual
/// and for a degenerate scale, so a tile at grid resolution contributes
/// nothing rather than a plausible-looking jitter.
pub(crate) fn micro_n_from_residual(residual: f64, full_scale: f64) -> f64 {
    if !(full_scale > 0.0) || !residual.is_finite() {
        return 0.5;
    }
    0.5 * (1.0 + (residual / full_scale).clamp(-1.0, 1.0))
}

/// LOD-D5 stage 2 — the residual at which the micro band saturates, from the
/// octave schedule and [`MICRO_RESIDUAL_HEADROOM`].
///
/// A named function rather than an expression at the one call site, so the
/// headroom constant is reachable by a test: written inline it survives
/// mutation, because nothing outside the renderer can see the number the
/// renderer divided by. `tests/lod_d5_scale_aware.rs`'s
/// `the_micro_band_reads_the_residual_and_saturates_at_a_derived_scale`
/// asserts this against `zoom_detail_peak_amplitude() / 4.0` with the `4.0`
/// as a literal.
pub(crate) fn micro_full_scale() -> f64 {
    zoom_detail_peak_amplitude() / MICRO_RESIDUAL_HEADROOM
}

/// LOD-D5 stage 3 — [`build_crest`]'s stencil half-width in tile pixels, so
/// the stroke keeps a **ground** width of about one coarse cell.
///
/// `(1 / cells_per_px)^k`, rounded: exactly `1` at `k == 0` (the identity, by
/// `x^0`), exactly `1` at one pixel per cell, and growing as the view comes
/// in.
///
/// # Why the cap is a fraction of the tile and not a constant
///
/// `build_crest`'s stencil **clamps at the tile's edge** — a pixel closer than
/// `step` to the border reads itself instead of its neighbour, exactly as the
/// reference's own one-pixel version does at the outermost column. At `step ==
/// 1` that is a one-pixel skirt nobody can see; at a free-running ground-unit
/// step it is a `step`-pixel skirt on each side. Holding that skirt to a fixed
/// *fraction* of the tile rather than a fixed pixel count is the reason for
/// the cap, and the divisor is [`CREST_STEP_TILE_FRAC`]: 8 px on a 256 px
/// `TILE_PX` tile, 3.1% of each edge. It takes the tile's own dimension as an
/// argument rather than reading `TILE_PX`, because `render_biome_tile_rgba`
/// renders whatever size it is handed and an atlas chunk is not a screen tile.
///
/// **What the cap is NOT for, recorded because it is what it was first built
/// for and the hypothesis was wrong.** The LOD-D0 harness at 512x384, seed
/// 483920, shows the median per-frame seam ratio rising from 1.7383 with the
/// control off to 1.8315 with it on — 5.4%, on a bar (`<= 1.5`) LOD-D2 left
/// open. The skirt was the obvious suspect. Tightening the cap from a flat 32
/// to a thirty-second of the tile (6 px on that grid's 256x192 tiles) and
/// re-running the identical probe gave **1.8361** — no movement. The cap is
/// kept on its own reasoning and for the detail it recovers (zoom-40 detail
/// 50.0% -> 50.7% of the zoom-1 value), and the seam rise is **unattributed**;
/// `tests/lod_d5_scale_aware.rs`'s
/// `which_stage_moves_the_shared_column_between_two_tiles` carries the rest of
/// that measurement.
pub(crate) fn crest_step_for_scale(cells_per_px: f64, k: f64, tile_min_dim: usize) -> usize {
    if k <= 0.0 || !(cells_per_px.is_finite() && cells_per_px > 0.0) {
        return 1;
    }
    let s = (1.0 / cells_per_px).powf(k);
    if !s.is_finite() {
        return 1;
    }
    let cap = (tile_min_dim / CREST_STEP_TILE_FRAC).max(1);
    s.round().clamp(1.0, cap as f64) as usize
}

/// The widest stencil [`crest_step_for_scale`] will ask for, as a divisor of
/// the tile's shorter side. `32` puts the clamped skirt at 3.1% of each edge.
const CREST_STEP_TILE_FRAC: usize = 32;

/// LOD-D5 stage 4 — the discharge a per-tile river SDF calls a channel at
/// this zoom.
///
/// `base · cells_per_px^(2k)`, clamped into `[floor, base]`:
///
/// - **never above `base`**, so pulling the view out can only ever return the
///   map's own answer and never erase a river the map draws (a parent tile at
///   level 0 has `cells_per_px` well above `1`);
/// - **never below [`RIVER_TILE_THRESH_FLOOR`]**, and never below `base`
///   either, so a world whose own threshold is already under the floor is not
///   raised to it.
///
/// Exactly `base` at `k == 0` and at one pixel per cell, by `x^0` and `1^x`.
pub(crate) fn tile_river_thresh(base: f64, cells_per_px: f64, k: f64) -> f64 {
    if !(base > 0.0) || k <= 0.0 || !(cells_per_px.is_finite() && cells_per_px > 0.0) {
        return base;
    }
    let t = base * cells_per_px.powf(RIVER_THRESH_SCALE_EXP * k);
    if !t.is_finite() {
        return base;
    }
    t.clamp(RIVER_TILE_THRESH_FLOOR.min(base), base)
}

/// LOD-D5's per-pixel inputs to [`land_color`], resolved once per tile
/// (the three weights, the micro mix) and once per pixel (the residual `n`).
///
/// `None` at the call site is the pre-LOD-D5 behaviour by **control flow**:
/// `cell_color` passes `None` and never touches any of this, so the shipped
/// screen render is byte-identical whatever `detail_scale_strength` says.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DetailScale {
    /// [`scaled_detail_weights`]' three bands, in macro/meso/micro order.
    pub weights: (f64, f64, f64),
    /// How far the micro band has crossed from value noise to relief —
    /// `detail_scale_strength · max(u, 0)`, so it is `0` at and outside grid
    /// resolution and `1` at full strength six octaves in.
    pub micro_mix: f64,
    /// [`micro_n_from_residual`] at this pixel.
    pub micro_n: f64,
}

/// The grid-resolution precomputes a tile needs and cannot build for itself,
/// built **once per world and appearance** and shared by every tile.
///
/// Three of the five rasters exist because the stage they feed is a
/// *neighbourhood* pass, and a neighbourhood truncates at a tile border. The
/// other two are world-scale classifications no single tile can see.
///
/// # Budget
///
/// `LOD_DETAIL_SCOPE.md` LOD-D1: *"`TileFields` holds <= 32 MiB at
/// 2048x1311."* [`Self::bytes`] reports the real figure and
/// `tile_fields_stay_inside_the_scope_budget` pins it. The detail band is
/// stored as the single difference `fine - blurred` rather than as both
/// blurs: `apply_local_contrast` only ever uses that difference, bilinear
/// interpolation is linear, so sampling the difference and differencing two
/// samples are the same number — at half the memory.
pub struct TileFields<'a> {
    gw: usize,
    gh: usize,
    /// `fine - blurred` of the finished grid raster's Rec.709 luma —
    /// `apply_local_contrast`'s own `d`, at grid resolution. Empty when
    /// `local_contrast == 0.0` or no grid raster was supplied, and the
    /// consumer tests the length rather than the flag (`RenderCtx::coast_d`'s
    /// contract).
    contrast_d: Cow<'a, [f32]>,
    /// [`build_grade_influence`]'s per-cell multiplier. Empty when all four
    /// field weights are `0.0`, which is every render that leaves them alone.
    grade_influence: Cow<'a, [f32]>,
    /// `currentWaterBodies()` (reference 5846) — `0` land, `1` ocean, `2`
    /// lake. Empty when lakes are switched off.
    lake_class: Cow<'a, [u8]>,
    /// `_lakeFill` — the pooled lake surface from the same priority-flood
    /// (`cartalith_civ::WaterBodies::fill_level`). The v1.05 shoreline needs
    /// it; without it the lake branch falls back to the reference's own
    /// nearest-cell stamp, which is what draws square lakes.
    lake_fill: Cow<'a, [f32]>,
    /// Ruling BO's forced-lake mask (`WorldGen::forced_lakes`), already
    /// applied to `lake_class` by [`Self::with_forced_lakes`], kept so the
    /// smooth band draws a forced lake's edge the way the base map's shore
    /// field does (`shore_depth_forced`). **Empty by construction** -- a
    /// world with no forced lake -- and the consumer tests the length.
    lake_forced: Cow<'a, [u8]>,
    /// A loaded save's one-cell river flag ([`save_flag_at`]) -- the ink
    /// `build_color_texture` composites for a world with no vector network.
    /// `None` renders the terrain alone, and is every generated world's
    /// state: its tiles draw vector rivers (`river_stroke::rasterize`).
    ink: Option<&'a [u8]>,
    /// [`build_glacier_potential`] — LOD-D4's grid-resolution glacier field.
    /// **Empty by construction**, on `RenderCtx::coast_d`'s contract: attach
    /// it with [`Self::with_cryo`], and the tile path tests this slice's
    /// length rather than `appearance.ice_strength`, so the field and its
    /// gate cannot disagree. Empty is also the honest state for a loaded
    /// save, which stores no flow to derive a catchment from.
    glacier: Cow<'a, [f32]>,
    /// The lapse relation the sub-cell temperature needs ([`TileCryo`]).
    /// `None` leaves the tile on the grid's own sampled temperature, which
    /// is exactly what every tile did before LOD-D4.
    ///
    /// **An attached `TileCryo` is still ignored where
    /// [`TerrainAppearance::ice_strength`] is `0.0`**, so this being `Some`
    /// is not on its own enough to move a pixel — see that field for why the
    /// milestone's two live stages share one gate.
    cryo: Option<TileCryo>,
    /// The display device the finished tile is encoded for, so a tile and the
    /// map under it are in the same gamut. `Srgb` is an early return.
    color_space: ColorSpace,
}

impl<'a> TileFields<'a> {
    /// Build the shared fields for one world and one appearance.
    ///
    /// `grid_rgb` is the finished grid raster — `build_color_texture`'s own
    /// `bytes`, tightly packed RGB8, `gw * gh * 3` long. It is what the
    /// local-contrast detail band is measured from, and there is no
    /// substitute: the band is a property of the *finished colour*, not of the
    /// height field. Pass `None` (or a wrong-length buffer) and the stage is
    /// off for tiles, which is honest rather than approximate — the tile then
    /// differs from the screen by exactly that stage, and `tests/golden_parity_tile_biome.rs`
    /// measures it.
    #[allow(dead_code)]
    pub fn new(ctx: &RenderCtx, grid_rgb: Option<&[u8]>) -> TileFields<'static> {
        let (gw, gh) = (ctx.gw, ctx.gh);
        let n = gw * gh;
        let a = &ctx.appearance;

        // `apply_local_contrast`'s own prologue, to its own line: Rec.709
        // luma of the finished raster, the same radius floor and short-axis
        // cap, the same `r_inner`, the same two blurs. A transcription of that
        // function rather than a call into it, because that one writes a
        // correction into a buffer and this one needs the band itself.
        let contrast_d = match grid_rgb {
            Some(rgb) if a.local_contrast > 0.0 && n > 0 && rgb.len() >= n * 3 => {
                let mut luma = vec![0f32; n];
                luma.par_iter_mut().enumerate().for_each(|(i, l)| {
                    let o = i * 3;
                    *l = (0.2126 * rgb[o] as f64 + 0.7152 * rgb[o + 1] as f64 + 0.0722 * rgb[o + 2] as f64) as f32;
                });
                let rad = ((gw as f64 * a.local_contrast_radius_frac).round() as i64).max(3).min((gh as i64 / 4).max(3));
                let r_inner = (rad / 8).max(2);
                let (fine, blurred) = rayon::join(|| blur_once(&luma, gw, gh, r_inner, ctx.world), || blur_once(&luma, gw, gh, rad, ctx.world));
                let mut d = vec![0f32; n];
                d.par_iter_mut().enumerate().for_each(|(i, v)| *v = (fine[i] as f64 - blurred[i] as f64) as f32);
                d
            }
            _ => Vec::new(),
        };

        let grade_influence = build_grade_influence(ctx, gw, gh);

        // `currentWaterBodies()` — the identical call `with_map_scale` and the
        // civilisation pass both make, so the three cannot classify one world
        // differently. `fill_level` is `_lakeFill`, captured from the same
        // priority-flood, which is what makes the v1.05 organic shoreline
        // reachable rather than the square-lake fallback.
        //
        // From the world's height (`water_height`), never the shaded field
        // RV-3 hands `ctx.field`: the valley re-cut is not hydrology, and a
        // depression it opened or closed must not become or stop being a lake.
        let wb = cartalith_civ::build_water_bodies(ctx.water_height(), gw, gh, ctx.sea_level, ctx.world, Some(ctx.rainfall));

        TileFields {
            gw,
            gh,
            contrast_d: Cow::Owned(contrast_d),
            grade_influence: Cow::Owned(grade_influence),
            lake_class: Cow::Owned(wb.classification),
            lake_fill: Cow::Owned(wb.fill_level),
            lake_forced: Cow::Borrowed(&[]),
            ink: None,
            // LOD-D4 is **opt-in here and not built by `new`**, on the same
            // argument `with_ink` and `with_lithology` already make: the
            // glacier field needs two generation parameters (`peak_m` and
            // `glacial_snowline`) and a map width that a `RenderCtx` does not
            // carry, so building it here would mean guessing three numbers.
            // `with_cryo` is where a caller that has them says so.
            glacier: Cow::Borrowed(&[]),
            cryo: None,
            color_space: ColorSpace::Srgb,
        }
    }

    /// A second `TileFields` over the **same rasters**, borrowed rather than
    /// copied — so a cached one can be re-pointed at this call's river ink
    /// without rebuilding anything.
    ///
    /// [`Self::new`] is 278.2 ms at 2048x1311 with the local-contrast band and
    /// 243.5 ms without it (273.2..281.7 and 243.0..246.2, release, this
    /// machine, medians over five), and its four rasters are 9 B/cell — so
    /// neither recomputing nor cloning them per tile is affordable. The returned value carries no ink
    /// and sRGB; chain [`Self::with_ink`] / [`Self::with_color_space`] /
    /// [`Self::without_lakes`] onto it exactly as onto a fresh one. Those
    /// builders take `self` by value, and on a borrowed `TileFields`
    /// `without_lakes` re-points the two lake `Cow`s at an empty slice rather
    /// than dropping the cache's own — the cache is untouched either way.
    #[allow(dead_code)]
    pub fn borrowed(&'a self) -> TileFields<'a> {
        TileFields {
            gw: self.gw,
            gh: self.gh,
            contrast_d: Cow::Borrowed(&self.contrast_d),
            grade_influence: Cow::Borrowed(&self.grade_influence),
            lake_class: Cow::Borrowed(&self.lake_class),
            lake_fill: Cow::Borrowed(&self.lake_fill),
            // Carried with the classification it was applied to: dropping it
            // would leave the forced cells lake with no edge to draw.
            lake_forced: Cow::Borrowed(&self.lake_forced),
            ink: None,
            // Carried, not dropped: the cached `TileFields` is the only place
            // the glacier field is built, and a re-pointed borrow that lost
            // it would silently draw the world without its ice while every
            // other stage matched.
            glacier: Cow::Borrowed(&self.glacier),
            cryo: self.cryo,
            color_space: self.color_space,
        }
    }

    /// Attach a loaded save's river flag, the ink the screen composites for
    /// it ([`save_flag_at`]). Must never be given a generated world's mask:
    /// that world's tiles draw vector rivers.
    #[allow(dead_code)]
    pub fn with_ink(mut self, ink: &'a [u8]) -> Self {
        self.ink = Some(ink);
        self
    }

    /// Attach LOD-D4's two derived inputs: [`build_glacier_potential`]'s
    /// field and the [`TileCryo`] lapse relation.
    ///
    /// **Both or neither, in one call, on purpose.** They are the two halves
    /// of one milestone and they answer the same question at two scales — the
    /// field says *where ice can be* at grid resolution, the relation says
    /// *how cold this pixel is* below it. A builder per half would make
    /// "glacier field attached, lapse relation missing" reachable, which
    /// renders ice at a temperature the ice was not derived from.
    ///
    /// A `glacier` of the wrong length is treated as absent by the consumer
    /// (it tests `len() == gw * gh`), which is what makes a loaded save's
    /// empty field the documented snow-only fallback rather than an error.
    #[allow(dead_code)]
    pub fn with_cryo(mut self, glacier: impl Into<Cow<'a, [f32]>>, cryo: TileCryo) -> Self {
        self.glacier = glacier.into();
        self.cryo = Some(cryo);
        self
    }

    /// Encode finished tiles for a display device other than sRGB.
    #[allow(dead_code)]
    pub fn with_color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    /// `state.viz.showLakes === false` (reference 11674) — drop the lake mask,
    /// so above-sea pools render as the terrain under them.
    #[allow(dead_code)]
    pub fn without_lakes(mut self) -> Self {
        self.lake_class = Cow::Borrowed(&[]);
        self.lake_fill = Cow::Borrowed(&[]);
        self.lake_forced = Cow::Borrowed(&[]);
        self
    }

    /// **Ruling BO: draw forced lakes in the tiles.** Applies `mask`
    /// (`WorldGen::forced_lakes`) to this field's classification -- the
    /// reference's `forceLake` post-pass, `apply_forced_lakes` -- and
    /// keeps it for the smooth band's forced edge (`shore_depth_forced`), so
    /// a tile draws exactly the water the base map draws.
    ///
    /// A builder, like [`Self::with_cryo`], because [`Self::new`] takes a
    /// `RenderCtx` and the mask is `WorldGen` state no context carries; not
    /// calling it is the no-forced-lake state, which is why every existing
    /// caller stays byte-identical. A mask that is not `gw * gh` long, or a
    /// field whose lakes are off ([`Self::without_lakes`]), is left alone --
    /// forcing into an empty classification would index past it.
    #[allow(dead_code)]
    pub fn with_forced_lakes(mut self, mask: &[u8]) -> Self {
        let n = self.gw * self.gh;
        if mask.len() == n && self.lake_class.len() == n {
            apply_forced_lakes(self.lake_class.to_mut(), Some(mask));
            self.lake_forced = Cow::Owned(mask.to_vec());
        }
        self
    }

    /// Retained bytes, for the scope's own budget line. Counts the five
    /// rasters; the two `usize`s, the two enums and [`TileCryo`]'s three
    /// `f64`s are not worth counting and are not counted, which is stated so
    /// the number is reproducible.
    ///
    /// LOD-D4's own budget line is *"glacier field <= 10 MiB"*, and
    /// [`Self::glacier_bytes`] reports that one on its own so the two budgets
    /// are not read off one number.
    #[allow(dead_code)]
    pub fn bytes(&self) -> usize {
        self.contrast_d.len() * 4 + self.grade_influence.len() * 4 + self.lake_class.len() + self.lake_fill.len() * 4 + self.lake_forced.len() + self.glacier_bytes()
    }

    /// The glacier field's own retained bytes — LOD-D4's `<= 10 MiB` budget.
    #[allow(dead_code)]
    pub fn glacier_bytes(&self) -> usize {
        self.glacier.len() * 4
    }
}

/// `(Math.min(W, H) / 64) | 0` (reference 11672) — the meso-shade sample step,
/// in tile pixels, before its floor of `2`.
///
/// Named so it can be mutated: it is one of the three free constants this tile
/// path introduces. Both goldens in `tests/golden_parity_tile_biome.rs` turn
/// red when it moves (mutation-tested 64.0 -> 8.0, killed by both).
const MESO_STEP_DIV: f64 = 64.0;

/// The bilinear lake-membership fraction below which the v1.05 shoreline stops
/// cutting water, and the pooled depth below which a lake counts as flat or
/// brush-painted and keeps its whole cell (reference 11734 and 11737).
const LAKE_MEMBERSHIP_MIN: f64 = 0.35;
const LAKE_FLAT_EPS: f64 = 0.004;

/// One land pixel of a tile, as `LOD_DETAIL_SCOPE.md`'s **Aletsch comparison
/// sheet** needs to see it — the quantities LOD-D4's four acceptance bars are
/// written in, at the resolution the milestone changed.
///
/// LOD-D0 built that sheet *"as far as today's fields allow and NO further"*
/// and it reads `sample_cell` at **grid** resolution, which is exactly where
/// LOD-D4's lapse correction is zero by construction — so the D0 sheet cannot
/// see this milestone at all, whatever it does. That is not a defect in the
/// D0 probe; it is the measurement moving with the thing measured, and this
/// struct is where it moves to.
///
/// # Why this carries `#[allow(dead_code)]`
///
/// Its only callers are `tests/lod_d4_ice_and_snow.rs` (checked present
/// 2026-09-21) — the harness that measures the scope's four acceptance bars,
/// and `the_metrics_agree_with_the_picture`, which holds it to the same
/// answer the renderer gives. A test is not part of the lib build, so the
/// lib-only `cargo check` cannot see either use.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CryoSample {
    /// The tile's own amplified height at this pixel, normalised as the rest
    /// of the engine spells height.
    pub elevation: f64,
    /// The coarse-unit slope the renderer classifies rock with — the tile's
    /// central difference over the ground it spans, identical to the
    /// expression in [`render_biome_tile_rgba`]'s land branch.
    pub slope: f64,
    /// `cos(aspect)`, pole-ward positive — the same number the D0 probe takes
    /// as `cos(deg_to_rad(aspect_deg))`, from the same `(dx, dy)` convention
    /// `sample_bridge::aspect_deg` documents (`+y` is south, so the downslope
    /// northward component is `+dy`). `0.0` on a pixel with no slope, where
    /// an aspect is undefined.
    pub northness: f64,
    /// The **tile's own** temperature — LOD-D4 stage 2 applied, or the
    /// grid's sampled value where no [`TileCryo`] is attached.
    pub temperature_c: f64,
    /// `material_weights`' own snow term at that temperature, then raised by
    /// the ice cover. This is the D0 sheet's `snow` with the milestone in it.
    pub snow: f64,
    /// The ice cover itself — [`apply_ice_cover`]'s second return. The
    /// numerator of the sheet's **ice fraction**, which D0 could only report
    /// as absent.
    pub ice: f64,
    /// The rock fraction after the same rebalance, for the scope's third bar.
    pub rock: f64,
    /// The glacier potential sampled at this pixel, before `ice_strength` and
    /// before the slope term — the population the scope's second bar selects
    /// on (*"cells with glacier potential >= 0.5"*).
    pub glacier: f64,
}

/// The [`CryoSample`] of every **land** pixel of one tile, in row-major order,
/// skipping sea and lake exactly as the D0 sheet skips anything whose `water`
/// is not `"land"`.
///
/// # What this shares with the renderer, and what it repeats
///
/// It calls the same [`TileCryo::delta_t`], the same [`material_weights`] and
/// the same [`apply_ice_cover`] the colour path calls, so the three stages the
/// milestone adds are measured, not modelled. What it repeats is two
/// one-liners — the slope expression and the world-coordinate mapping — which
/// are repeated rather than factored out because factoring them would put a
/// function call inside the per-pixel loop of the hot path to buy a guarantee
/// that `the_metrics_agree_with_the_picture` already gives by measurement.
///
/// The eco-jitter is deliberately **not** applied here: the D0 sheet's own
/// baseline is `smoothstep(3, -5, t)` on the plain temperature, and a metric
/// that quietly added a noise term would not be comparable to the number it
/// is supposed to be moving.
#[allow(dead_code)]
pub fn tile_cryo_samples(ctx: &RenderCtx, tile: &[f32], w: usize, h: usize, bounds: TileBounds, tf: &TileFields) -> Vec<CryoSample> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    if w == 0 || h == 0 || tile.len() < w * h || gw == 0 || gh == 0 || tf.gw != gw || tf.gh != gh {
        return Vec::new();
    }
    let a = &ctx.appearance;
    let sl = ctx.sea_level;
    let denom = if (1.0 - sl) > 0.0 { 1.0 - sl } else { 1.0 };
    let cx = bounds.w / (w.max(2) - 1) as f64;
    let cy = bounds.h / (h.max(2) - 1) as f64;
    let lakes = tf.lake_class.len() == gw * gh;
    let lake_fill_ok = tf.lake_fill.len() == gw * gh;
    let glacier_on = tf.glacier.len() == gw * gh && a.ice_strength > 0.0;
    let cryo = if a.ice_strength > 0.0 { tf.cryo } else { None };
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        let wy = bounds.y + y as f64 * cy;
        let ro = y * w;
        for x in 0..w {
            let i = ro + x;
            let ht = tile[i] as f64;
            let wx = bounds.x + x as f64 * cx;
            if ht < sl || (lakes && is_lake_pixel(tf, ctx, wx, wy, ht, lake_fill_ok)) {
                continue;
            }
            let l = cartalith_terrain::tile_render::edge_l(tile, w, x, ro);
            let r = cartalith_terrain::tile_render::edge_r(tile, w, x, ro);
            let u = cartalith_terrain::tile_render::edge_u(tile, w, h, x, y);
            let d = cartalith_terrain::tile_render::edge_d(tile, w, h, x, y);
            let (gx, gy) = ((r - l) / (2.0 * cx), (d - u) / (2.0 * cy));
            let slope = cartalith_jsmath::js_hypot(gx, gy);
            let northness = if slope > 0.0 { gy / slope } else { 0.0 };

            let t0 = sample_arr(ctx.temperature, wx, wy, gw, gh);
            // The same gate `render_biome_tile_rgba` applies, spelled the same
            // way. If the metrics applied the lapse correction where the
            // picture does not, `the_metrics_agree_with_the_picture` would be
            // measuring two different renders.
            let t = match cryo {
                Some(c) => t0 - c.delta_t(ht - sample_arr(ctx.field, wx, wy, gw, gh)),
                None => t0,
            };
            let m = sample_arr(ctx.rainfall, wx, wy, gw, gh);
            let r_frac = (ht - sl) / denom;
            let flow = ctx.flow.map(|f| sample_arr(f, wx, wy, gw, gh)).unwrap_or(0.0);
            let acc = (flow / (gw * gh) as f64).max(1e-4);
            let twi = (acc / slope.max(0.002)).ln();
            let asp = ctx.aspect_factor_f(wx, wy);
            let curv = ctx.curvature_at_f(wx, wy);
            let glacier = if glacier_on { sample_arr(&tf.glacier, wx, wy, gw, gh) } else { 0.0 };
            let wts = material_weights(t, m, slope, r_frac, twi, asp, curv, snow_aspect_shift(a, tile_snow_facing(ctx, gy, slope, wy), slope));
            let scaled = glacier * a.ice_strength;
            let (wts, ice) = if scaled > 0.0 { apply_ice_cover(wts, scaled, slope) } else { (wts, 0.0) };
            out.push(CryoSample { elevation: ht, slope, northness, temperature_c: t, snow: wts.snow, ice, rock: wts.rock, glacier });
        }
    }
    out
}

/// LOD-D5's river seeds for one tile — the mask [`build_river_sdf`] measures
/// its distance from, as `1`/`0` bytes in the tile's own row-major order.
///
/// This is `render_biome_tile_rgba`'s own river prologue with the SDF left
/// off: the same `sample_arr` of the same `ctx.flow` at the same world
/// coordinates, and the same [`tile_river_thresh`] of the same
/// `ctx.river_thresh`. It exists because the milestone's acceptance bar is
/// written about the seeds — *"the count of distinct channel components in
/// view rises with zoom, and every drawn channel pixel has sampled discharge
/// ≥ the threshold"* — and a metric that re-derived either the sampling or
/// the threshold would be measuring a second renderer.
///
/// Returns the threshold beside the mask, because the second half of that bar
/// is a statement about a number the caller cannot otherwise see.
///
/// Empty (and `0.0`) for a malformed call, for a `ctx` with no flow, and for
/// a threshold that was never attached — the same three conditions under
/// which the renderer itself builds no river SDF, so the two agree about
/// "there are no seeds" rather than this one inventing some.
///
/// # Why this carries `#[allow(dead_code)]`
///
/// Its only caller is `tests/lod_d5_scale_aware.rs` (checked present
/// 2026-09-21), and a test is not part of the lib build — `tile_cryo_samples`
/// carries the same attribute for the same reason.
#[allow(dead_code)]
pub fn tile_river_seeds(ctx: &RenderCtx, w: usize, h: usize, bounds: TileBounds) -> (Vec<u8>, f64) {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let flow = match ctx.flow {
        Some(f) if w > 0 && h > 0 && gw > 0 && gh > 0 && f.len() >= gw * gh && ctx.river_thresh > 0.0 => f,
        _ => return (Vec::new(), 0.0),
    };
    let cx = bounds.w / (w.max(2) - 1) as f64;
    let cy = bounds.h / (h.max(2) - 1) as f64;
    let thresh = tile_river_thresh(ctx.river_thresh, (cx * cy).sqrt(), ctx.appearance.detail_scale_strength);
    let mut mask = vec![0u8; w * h];
    for y in 0..h {
        let wy = bounds.y + y as f64 * cy;
        for x in 0..w {
            // `build_river_sdf`'s own `f as f64 > thresh`, **through the same
            // `f32`**: the renderer stores its sampled flow into an `f32`
            // buffer before the comparison, so a mask that compared the `f64`
            // sample would disagree with it on any pixel whose two roundings
            // straddle the threshold. The boundary is strict for the same
            // reason — `>=` would disagree on a pixel that sampled it exactly.
            if (sample_arr(flow, bounds.x + x as f64 * cx, wy, gw, gh) as f32) as f64 > thresh {
                mask[y * w + x] = 1;
            }
        }
    }
    (mask, thresh)
}

/// `tileShadeExag(bounds, W)` (DCC line v2.25, `RC_ENGINE_CHANGES.md` §4) —
/// the hillshade exaggeration for a tile `w` pixels wide covering `bounds_w`
/// coarse cells: `exag * max(1, (w - 1) / bounds_w)`.
///
/// `(w - 1) / bounds_w` is tile pixels per coarse cell, the reciprocal of the
/// `cx` [`render_biome_tile_rgba`] steps by, so the shading gradient is read
/// per coarse cell exactly as the main map's `exag / s` reads it per sample.
/// The clamp at `1` is the reference's: a tile *coarser* than the grid keeps
/// the bare exaggeration rather than losing relief.
///
/// A `bounds_w` that is not a positive finite number returns `exag` — the
/// reference's own *"bounds omitted ⇒ the previous value exactly"*, and the
/// value that cannot divide by zero into an infinite normal.
pub fn tile_shade_exag(exag: f64, bounds_w: f64, w: usize) -> f64 {
    if !(bounds_w > 0.0 && bounds_w.is_finite()) {
        return exag;
    }
    exag * ((w as f64 - 1.0) / bounds_w).max(1.0)
}

/// `renderBiomeTileRGBA(tile, W, H, bounds)` (reference HTML 11668-11779) —
/// one LOD/atlas tile of **amplified** height as the full biome look, RGBA8,
/// row-major, four bytes per pixel, alpha always `255`.
///
/// `tile` is `w * h` amplified height samples (`cartalith_engine::bake::
/// pyramid_tile`, or `refine_tile` + `add_zoom_detail`); `bounds` is the
/// coarse-coordinate rect it covers. Everything else comes from `ctx` and
/// `tf`. Pure compute — no Godot type, no global, no `state`.
///
/// See the section comment above for the split between what a tile derives
/// from its own height and what it samples at world coordinates, for the three
/// deliberate departures from the reference, and for the port-only stages.
///
/// # Panics
///
/// Does not. A `tile` shorter than `w * h`, a zero dimension, or a `tf` built
/// for a different grid returns an empty `Vec` rather than indexing out of
/// bounds — `cartalith-rust-conventions`: a panic here crosses the gdext
/// boundary and takes the process with it, and this is reached from the LOD
/// bridge on every zoom notch.
#[allow(dead_code)]
pub fn render_biome_tile_rgba(ctx: &RenderCtx, tile: &[f32], w: usize, h: usize, bounds: TileBounds, tf: &TileFields) -> Vec<u8> {
    render_biome_tile_rgba_padded(ctx, tile, w, h, 0, bounds, tf)
}

/// How many texels of halo [`render_biome_tile_rgba_padded`] needs around a
/// `w × h` tile covering `bounds` so that **no shading stencil clamps at the
/// tile edge**: the widest of the macro normal's 1, the meso step `ms` and the
/// crest stencil `crest_step` — the three neighbourhood reads that take their
/// neighbours from the tile's own height. Computed by the same two
/// expressions the renderer uses, from the same arguments, so the halo and
/// the stencils cannot disagree.
///
/// The per-tile SDFs (coast, river, biome boundary) also read the halo, but
/// their range is many cells and is not what this is sized for; measured on
/// `_lodsweep_probe.gd`'s pan they were not a seam contributor at the halo's
/// absence (`OUTSTANDING_WORK.md`'s LOD-tile sawtooth row).
pub fn tile_halo_px(ctx: &RenderCtx, w: usize, h: usize, bounds: TileBounds) -> usize {
    if w < 2 || h < 2 {
        return 0;
    }
    let cx = bounds.w / (w - 1) as f64;
    let cy = bounds.h / (h - 1) as f64;
    let ms = ((w.min(h) as f64 / MESO_STEP_DIV).trunc() as usize).max(2);
    let crest = crest_step_for_scale((cx * cy).sqrt(), ctx.appearance.detail_scale_strength, w.min(h));
    ms.max(crest).max(1)
}

/// [`render_biome_tile_rgba`] over a tile that carries a **halo** of `pad`
/// texels on every side (`cartalith_engine::bake::pyramid_tile_padded`):
/// `tile` is `(w + 2·pad) × (h + 2·pad)`, `w`/`h`/`bounds` describe the tile
/// **without** it, and the result is the `w × h` core only.
///
/// Every per-tile constant (`cx`, `cy`, the meso step, the crest step, the
/// scaled exaggeration) is computed from the core `w`/`h`, so the halo changes
/// nothing but what a stencil reads past the edge: with it, the macro normal,
/// the meso normal and the crest take real neighbours where they used to
/// clamp, and two tiles either side of a boundary shade their shared edge
/// sample identically. Without it the meso normal — `ms` texels a side, with
/// no `/ms` — halved its gradient over an `ms`-wide band at every tile edge,
/// which `_lodsweep_probe.gd`'s pan measured as the dominant seam at 512×384
/// (`OUTSTANDING_WORK.md`'s LOD-tile sawtooth row). `pad == 0` is the
/// reference's `renderBiomeTileRGBA`, and `golden_parity_tile_biome.rs` pins
/// it there.
#[allow(dead_code)]
pub fn render_biome_tile_rgba_padded(ctx: &RenderCtx, tile: &[f32], w: usize, h: usize, pad: usize, bounds: TileBounds, tf: &TileFields) -> Vec<u8> {
    render_biome_tile_rgba_rivers(ctx, tile, w, h, pad, bounds, tf, None)
}

/// [`render_biome_tile_rgba_padded`] with the rivers drawn into it: `rivers`
/// is the tile's own [`RiverLayer`], `w x h` (the core, no halo), rasterized
/// at this tile's resolution from the same vector strokes the screen texture
/// uses (`lod_bridge::synthesize_tile_rgba`). A layer of any other size is
/// ignored rather than indexed past. `None` is the tile exactly as it was.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn render_biome_tile_rgba_rivers(ctx: &RenderCtx, tile: &[f32], w: usize, h: usize, pad: usize, bounds: TileBounds, tf: &TileFields, rivers: Option<&RiverLayer>) -> Vec<u8> {
    let rivers = rivers.filter(|l| l.width() == w && l.height() == h);
    let (gw, gh) = (ctx.gw, ctx.gh);
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let padf = pad as f64;
    if w == 0 || h == 0 || tile.len() < pw * ph || gw == 0 || gh == 0 || tf.gw != gw || tf.gh != gh {
        return Vec::new();
    }
    let a = &ctx.appearance;
    let sl = ctx.sea_level;
    // v2.11's `ex = state.exag` (11670) under `js_reference()`; v2.25's
    // `ex = tileShadeExag(bounds, W)` otherwise. One `ex`, used by both the
    // macro normal and the meso normal below, exactly as the reference's one
    // `const ex` is.
    let ex = if a.tile_shade_exag_scaled { tile_shade_exag(a.exag, bounds.w, w) } else { a.exag };
    let az = a.sun_az_deg.to_radians();
    let alt = a.sun_alt_deg.to_radians();
    let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());

    // `cx, cy` — coarse cells per tile pixel (11671).
    let cx = bounds.w / (w.max(2) - 1) as f64;
    let cy = bounds.h / (h.max(2) - 1) as f64;
    // `ms = Math.max(2, (Math.min(W,H)/64)|0)` (11672). `|0` truncates.
    let ms = ((w.min(h) as f64 / MESO_STEP_DIV).trunc() as usize).max(2);
    let denom = if (1.0 - sl) > 0.0 { 1.0 - sl } else { 1.0 };

    // ---- the per-tile prologue (11673-11710) --------------------------------
    //
    // Each of these is built only when its own strength is above zero, and
    // each consumer below tests the buffer's *length* rather than re-reading
    // the flag — `RenderCtx::coast_d`'s contract, so a gate and its field can
    // never disagree.

    // ---- LOD-D5's four scale-dependent quantities, resolved once per tile --
    //
    // `cells_per_px` is the geometric mean of the two axes' cell steps. The
    // mean rather than one axis because `pyramid_tile_bounds` is aspect-
    // matched and the two differ on a non-square map, and *geometric* because
    // the curve's argument is a `log2` — the geometric mean is the one whose
    // log is the mean of the logs, so a 2:1 tile sits exactly half an octave
    // from a square one instead of somewhere that depends on which axis was
    // picked.
    let cells_per_px = (cx * cy).sqrt();
    let u = detail_scale_u(cells_per_px);
    let scale_k = a.detail_scale_strength;
    let detail_w = scaled_detail_weights(a, u);
    // `max(u, 0.0)`: the micro band crosses to relief only as the view comes
    // IN. Pulled out past grid resolution there is no residual to read (the
    // tile is a decimation of the grid, not a refinement of it), and fading
    // toward a field that is structurally zero would just remove the grain.
    let micro_mix = scale_k * u.max(0.0);
    let micro_full_scale = micro_full_scale();
    let crest_step = crest_step_for_scale(cells_per_px, scale_k, w.min(h));

    // R2 crest, from the tile's own height, coarse-scaled by `cx, cy` — the
    // reference's `buildCrestField(tile, W, H, sl, cx, cy)`, with LOD-D5's
    // ground-unit stencil. **Both the step and the scale move together**: the
    // stencil spans `crest_step` pixels and `cx * crest_step` coarse cells, so
    // the curvature and the slope gate are still read in coarse-cell units and
    // `CREST_SLOPE_HI` still means what it meant. Passing one without the
    // other would silently re-scale the gate by the step.
    let crest_b = build_crest(tile, pw, ph, sl, cx * crest_step as f64, cy * crest_step as f64, crest_step, a);

    // B5 coast SDF, from the tile's own height. `buildCoastSDF` and this are
    // the same function over the same mask; `build_river_sdf`'s doc comment
    // carries the term-by-term proof for the sibling case.
    let coast_b = if a.sdf_coast > 0.0 { cartalith_civ::build_coast_sdf(tile, pw, ph, sl) } else { Vec::new() };

    // B3 river SDF, from flow sampled at the tile's world coordinates, with
    // the **grid's** `riverFlowThresh(GW, GH)` — the reference's own argument
    // (11683), so a tile and the map agree on which channels are rivers.
    // `river_thresh` is `0.0` until `with_map_scale` supplies a map width, and
    // that is the off state rather than a guessed threshold.
    let river_b = match ctx.flow {
        Some(flow) if a.sdf_rivers > 0.0 && ctx.river_thresh > 0.0 => {
            // Over the halo too, so the SDF sees channels just past the edge.
            // `yy as f64 - padf` is exactly `yy as f64` at `pad == 0`.
            let mut tfl = vec![0f32; pw * ph];
            for yy in 0..ph {
                let wyy = bounds.y + (yy as f64 - padf) * cy;
                for xx in 0..pw {
                    tfl[yy * pw + xx] = sample_arr(flow, bounds.x + (xx as f64 - padf) * cx, wyy, gw, gh) as f32;
                }
            }
            // LOD-D5 stage 4: the grid's threshold at grid resolution, and
            // continuously lower as the view comes in, so a tributary the
            // coarse pass accumulated but the map does not call a river is
            // drawn once one drawn pixel covers little enough ground. The
            // reference's own argument (11683) is preserved where it bites —
            // `tile_river_thresh` never returns more than `ctx.river_thresh`,
            // so a tile can still only ever agree with the map or add to it.
            build_river_sdf(&tfl, pw, ph, tile_river_thresh(ctx.river_thresh, cells_per_px, scale_k))
        }
        _ => Vec::new(),
    };

    // B4 biome-boundary distance, from `classifyBiome` of the sampled climate
    // (11685-11688). Deliberately the reference's **simplified** raster —
    // water is index `0` outright, with no water-body classification — and not
    // `build_biome_raster`, which is what the grid's own `biome_bd` uses. The
    // reference calls these distances *"local-per-tile (decoration)"* in the
    // same breath; matching its raster is what keeps the ecotone widths a tile
    // draws the same ones the reference draws.
    let biome_bd = if a.sdf_biomes > 0.0 {
        let mut bio = vec![0u8; pw * ph];
        for yy in 0..ph {
            let wyy = bounds.y + (yy as f64 - padf) * cy;
            for xx in 0..pw {
                let hh = tile[yy * pw + xx] as f64;
                bio[yy * pw + xx] = if hh < sl {
                    0
                } else {
                    let wxx = bounds.x + (xx as f64 - padf) * cx;
                    // `BIOME_INDEX[classifyBiome(t, m)]` — `classify_biome`
                    // already returns that 1-based index (`0` is ocean).
                    cartalith_civ::classify_biome(sample_arr(ctx.temperature, wxx, wyy, gw, gh), sample_arr(ctx.rainfall, wxx, wyy, gw, gh))
                };
            }
        }
        build_biome_boundary_dist(&bio, pw, ph)
    } else {
        Vec::new()
    };

    // v1.29, THE LOD SEAM (11690-11710): the sea floor and its shade are read
    // from the **world-wide** smoothed fields the main map already built, not
    // from a tile-local blur whose box truncates on a different side in each
    // of two neighbouring tiles. `RenderCtx` holds `sea_h`/`sea_shade`
    // unconditionally, so the reference's `hasOcean` scan — which only decides
    // whether to pay for building that cache — has nothing to decide here and
    // is omitted. A tile with no cell below `sl` never enters the branch.

    let lakes = tf.lake_class.len() == gw * gh;
    let lake_fill_ok = tf.lake_fill.len() == gw * gh;
    // The toon keyline's water mask, over the halo too so a shore just past
    // the tile edge still inks the edge pixel (the halo is >= 2, which is why
    // `TOON_OUTLINE_R` may not exceed it). The two tests are the loop's own
    // `water` branch below, at the same world position. Empty -- and never
    // built -- while the outline is off.
    let toon_water: Vec<bool> = if a.toon_outline > 0.0 {
        (0..pw * ph)
            .map(|k| {
                let ht = tile[k] as f64;
                let (xx, yy) = ((k % pw) as f64 - padf, (k / pw) as f64 - padf);
                ht < sl || (lakes && is_lake_pixel(tf, ctx, bounds.x + xx * cx, bounds.y + yy * cy, ht, lake_fill_ok))
            })
            .collect()
    } else {
        Vec::new()
    };
    let ink = tf.ink.filter(|m| m.len() >= gw * gh);
    let has_grade_influence = tf.grade_influence.len() == gw * gh;
    // LOD-D4's two gates, both length/`Option` tests rather than a re-read of
    // `appearance.ice_strength`, on `RenderCtx::coast_d`'s contract. The
    // strength still multiplies the sampled potential below, so `0.0` (which
    // is `js_reference()`'s value) reaches `land_color` as a literal zero and
    // takes its no-ice branch.
    //
    // **`ice_strength` gates stage 2 as well, and that is not decoration.**
    // The scope's requirement is *"three derived stages, all inert under
    // `js_reference()`"*, and the sub-cell temperature is one of the three: it
    // moves the WHOLE material path, not just snow, so a tile rendered with a
    // `TileCryo` attached and `ice_strength = 0.0` is not the reference's
    // tile. Measured, not reasoned: with this gate absent,
    // `the_whole_milestone_is_inert_under_js_reference` fails on the first
    // pixel of a 64x64 glaciated tile. The gate is a branch and not a `* 0.0`
    // so the off state is identity by control flow.
    let glacier_on = tf.glacier.len() == gw * gh && a.ice_strength > 0.0;
    let cryo = if a.ice_strength > 0.0 { tf.cryo } else { None };
    let contrast_on = a.local_contrast > 0.0 && tf.contrast_d.len() == gw * gh;
    let knee = a.local_contrast_knee.max(1e-3);
    let inv_knee2 = 1.0 / (knee * knee);

    let mut rgb = vec![0u8; w * h * 3];
    // Per-tile grade influence, sampled from the grid's. Built only when the
    // grid's exists, so the flat-grade fast path in `apply_color_grade` is
    // still taken on every render that leaves the four field weights alone.
    let mut influence: Vec<f32> = if has_grade_influence { vec![0f32; w * h] } else { Vec::new() };

    // Row-parallel on `BakeFields::new`'s own determinism argument: every
    // output pixel is a pure function of immutable inputs and its own
    // coordinates, each row writes disjoint bytes, and nothing accumulates
    // across pixels. The result does not depend on the schedule.
    rgb.par_chunks_mut(w * 3).enumerate().for_each(|(y, out)| {
        let wy = bounds.y + y as f64 * cy;
        // `i` indexes the (halo-carrying) height buffer and every per-tile
        // field built over it; `x`/`y` stay core coordinates for the world
        // position and the output.
        let (yp, ro) = (y + pad, (y + pad) * pw);
        for x in 0..w {
            let xp = x + pad;
            let i = ro + xp;
            let ht = tile[i] as f64;
            let wx = bounds.x + x as f64 * cx;

            // --- 1. the macro normal, from the tile's own height (11714) ----
            let l = cartalith_terrain::tile_render::edge_l(tile, pw, xp, ro);
            let r = cartalith_terrain::tile_render::edge_r(tile, pw, xp, ro);
            let u = cartalith_terrain::tile_render::edge_u(tile, pw, ph, xp, yp);
            let d = cartalith_terrain::tile_render::edge_d(tile, pw, ph, xp, yp);
            let (mut nx, mut ny, mut nz) = (-(r - l) * ex, -(d - u) * ex, 1.0f64);
            let il = 1.0 / cartalith_jsmath::js_hypot3(nx, ny, nz);
            nx *= il;
            ny *= il;
            nz *= il;
            let sh = if a.npr.multi_sun { multi_sun_from_normal(a, nx, ny, nz) } else { cartalith_jsmath::js_max(0.0, nx * lx + ny * ly + nz * lz) };

            // --- 2. the sampled climate and vignette (11716) ----------------
            let t = sample_arr(ctx.temperature, wx, wy, gw, gh);
            let vig = ctx.vignette_at_f(wx, wy);

            // --- 3. the three colour branches -------------------------------
            let water = if ht < sl {
                // Ocean (11719-11721). Sampled sea floor and sea shade, per
                // the v1.29 seam fix above.
                let hs = sample_arr(&ctx.sea_h, wx, wy, gw, gh);
                let shw = sample_arr(&ctx.sea_shade, wx, wy, gw, gh);
                let depth = if sl <= 0.0 { 0.0 } else { clamp01((sl - hs) / sl) };
                Some(sea_color_core(a, depth, t, sea_grain(a, wx, wy, gw), shw, vig))
            } else if lakes && is_lake_pixel(tf, ctx, wx, wy, ht, lake_fill_ok) {
                // v1.05 lake (11741-11742) -- see [`lake_color`].
                Some(lake_color(a, t, sea_grain(a, wx, wy, gw), vig))
            } else {
                None
            };

            let (cr, cg, cb) = match water {
                Some(v) => v,
                None => {
                    // --- Land (11743-11772) ---------------------------------
                    let m = sample_arr(ctx.rainfall, wx, wy, gw, gh);

                    // LOD-D4 stage 2: the tile's OWN temperature. `ctx.
                    // temperature` is a grid raster, so `t` above is the
                    // temperature of the CELL this pixel sits in; the tile
                    // knows its own height and `cartalith-climate` owns the
                    // relation between height and temperature, so the pixel
                    // can have its own. `h_coarse` is the same bilinear read
                    // of the grid height the temperature raster was built
                    // from, which is what makes the correction a difference
                    // rather than a second opinion — and what makes it
                    // EXACTLY zero at one tile pixel per cell, where
                    // `sample_arr` at integer coordinates returns the cell.
                    //
                    // It replaces `t` for the WHOLE material path and not
                    // just for snow, deliberately: the scope's stage is
                    // *"tile temperature"*, and a sub-cell ridge 300 m above
                    // its cell centre is colder for the treeline exactly as
                    // it is colder for the snowline.
                    // The tile's relief residual over the coarse field, read
                    // once and used twice: LOD-D4's lapse correction needs
                    // exactly this difference, and LOD-D5's micro band is a
                    // measurement of it. Computed only when one of the two is
                    // live, so a tile with both off pays nothing.
                    let residual = if cryo.is_some() || micro_mix > 0.0 { ht - sample_arr(ctx.field, wx, wy, gw, gh) } else { 0.0 };
                    let t = match cryo {
                        Some(c) => t - c.delta_t(residual),
                        None => t,
                    };

                    // Meso shade at the `ms` step. **No `/ ms`** — see
                    // departure 2 in the section comment; that normaliser is
                    // `shadeFactor2`'s, and the reference's tile does not
                    // carry it.
                    let l2 = tile[ro + if xp >= ms { xp - ms } else { xp }] as f64;
                    // `x + ms < w`, not `x < w - ms`: the reference's `usize`
                    // is a double and `W - ms` is simply negative on a tile
                    // narrower than the step, while here it UNDERFLOWS to
                    // 18446744073709551615 and the comparison then indexes off
                    // the end. Same value on every tile the bridge builds, and
                    // the difference is a panic across the gdext boundary on a
                    // 1x1 one (`malformed_calls_return_empty_rather_than_panicking`).
                    let r2 = tile[ro + if xp + ms < pw { xp + ms } else { xp }] as f64;
                    let u2 = tile[(if yp >= ms { yp - ms } else { yp }) * pw + xp] as f64;
                    let d2 = tile[(if yp + ms < ph { yp + ms } else { yp }) * pw + xp] as f64;
                    let (mut mx, mut my, mut mz) = (-(r2 - l2) * ex, -(d2 - u2) * ex, 1.0f64);
                    let iml = 1.0 / cartalith_jsmath::js_hypot3(mx, my, mz);
                    mx *= iml;
                    my *= iml;
                    mz *= iml;
                    let sh_m = cartalith_jsmath::js_max(0.0, mx * lx + my * ly + mz * lz);

                    // Coarse-unit slope: the tile's own central difference
                    // divided by the ground distance it spans, so
                    // `material_weights`' `slope / 0.04` and `slope / 0.08`
                    // thresholds classify rock at a tile exactly as they do on
                    // the map. The reference says so itself at 11748.
                    let slope = cartalith_jsmath::js_hypot((r - l) / (2.0 * cx), (d - u) / (2.0 * cy));
                    let r_frac = (ht - sl) / denom;
                    let flow = ctx.flow.map(|f| sample_arr(f, wx, wy, gw, gh)).unwrap_or(0.0);
                    let acc = (flow / (gw * gh) as f64).max(1e-4);
                    let beta = slope.max(0.002);
                    let twi = (acc / beta).ln();
                    let asp = ctx.aspect_factor_f(wx, wy);
                    let curv = ctx.curvature_at_f(wx, wy);
                    let ao = sample_arr(&ctx.ao, wx, wy, gw, gh);
                    // B4 in tiles: tile-pixel distance scaled to coarse cells
                    // (`x cx`) before `sdfEcoKv` sees it, so the ecotone reads
                    // the same width at any zoom (11753).
                    let eco_k = if biome_bd.is_empty() { 1.0 } else { sdf_eco_k(biome_bd[i] as f64 * cx, a.sdf_biomes, gw) };
                    let grad = if a.npr.hachure > 0.0 { ((r - l) / (2.0 * cx), (d - u) / (2.0 * cy)) } else { (0.0, 0.0) };
                    let cc = land_color(
                        a,
                        t,
                        m,
                        slope,
                        r_frac,
                        twi,
                        asp,
                        curv,
                        sh,
                        sh_m,
                        vig,
                        ao,
                        eco_k,
                        sample_arr(&ctx.hydro_wet, wx, wy, gw, gh),
                        ctx.litho_at_f(wx, wy),
                        grad,
                        wx,
                        wy,
                        gw,
                        gh,
                        ctx.splat.as_ref(),
                        ctx.paint_at_f(wx, wy),
                        ctx.ground,
                        // LOD-D4 stage 3's input: the grid-resolution
                        // potential, sampled bilinearly like every other
                        // world-scale field a tile reads, scaled by the one
                        // tunable this milestone adds. Two tiles over the
                        // same ground cannot disagree about it, which a
                        // per-tile derivation could not promise.
                        if glacier_on { sample_arr(&tf.glacier, wx, wy, gw, gh) * a.ice_strength } else { 0.0 },
                        // LOD-D5 stages 1 and 2. `Some` unconditionally on
                        // the tile path, because at `detail_scale_strength ==
                        // 0.0` and at one pixel per cell the three weights
                        // ARE the appearance's own and `micro_mix` is `0.0`,
                        // so this branch is the identity by arithmetic that
                        // cannot round -- which is what keeps
                        // `golden_parity_tile_biome.rs`'s reference goldens
                        // and its screen-identity check green rather than
                        // nearly green. Asserted, not claimed, by
                        // `tests/lod_d5_scale_aware.rs`.
                        Some(DetailScale { weights: detail_w, micro_mix, micro_n: micro_n_from_residual(residual, micro_full_scale) }),
                        // Ruling AP's snow facing from the TILE's own gradient,
                        // the one `slope` above came from: the coarse
                        // `aspect_factor_f` barely varies across a tile, so
                        // pairing it with the tile slope left snow blind to
                        // the tile's northness (LOD-D4 bar 1b, 2026-09-24).
                        tile_snow_facing(ctx, (d - u) / (2.0 * cy), slope, wy),
                        // The river symbol at this tile pixel, rasterized at
                        // the tile's own resolution. Land only, as the
                        // `water` branch above already decided.
                        rivers.and_then(|l| l.at(x, y)),
                    );
                    // R2 crest, then the two SDF bands — `applyCrest` and
                    // `applyCoastRiverSDFv`, in the reference's own order and
                    // with its own `0.7` (11767-11772).
                    let cc = if crest_b.is_empty() { cc } else { apply_crest(cc, crest_b[i] as f64 * a.crest_strength * 0.7) };
                    let cc = if coast_b.is_empty() { cc } else { apply_coast_sdf(cc, coast_b[i] as f64 * cx, a.sdf_coast, gw) };
                    let cc = if river_b.is_empty() { cc } else { apply_river_sdf(cc, river_b[i] as f64 * cx, a.sdf_rivers, gw) };
                    // The toon keyline, in `cell_color`'s slot, in TILE pixels
                    // (so it stays the same width on screen at every zoom).
                    if toon_water.is_empty() {
                        cc
                    } else {
                        let cover = toon_outline_cover(|dx, dy| {
                            let (qx, qy) = (xp as i64 + dx, yp as i64 + dy);
                            // Past the halo is unknown ground, not water.
                            qx >= 0 && qy >= 0 && (qx as usize) < pw && (qy as usize) < ph && toon_water[qy as usize * pw + qx as usize]
                        });
                        apply_toon_outline(cc, cover, a.toon_outline)
                    }
                }
            };

            // --- 4. the port-only per-pixel stages, in `cell_color`'s order --
            // Waves: water only, and `coast_d` is empty unless `npr.waves` is
            // on, so this is one length test on every other path.
            let (cr, cg, cb) = if ht < sl && !ctx.coast_d.is_empty() { apply_waves(a, (cr, cg, cb), sample_arr(&ctx.coast_d, wx, wy, gw, gh), gw) } else { (cr, cg, cb) };
            let tone = paper_tone(a, wx, wy, gw);
            let (cr, cg, cb) = apply_paper(a, (cr, cg, cb), tone);
            let (cr, cg, cb) = apply_border(a, (cr, cg, cb), tone, wx, wy, gw, gh);
            // The river ink, `build_color_texture`'s own three lines through
            // the shared `channel_tint`, nearest cell exactly as `bake_rect`
            // takes it and for the same reason (a river keeps its world width
            // at every zoom).
            //
            // **Land only.** `water` (stage 3 above) already carries this
            // pixel's real ocean-or-lake classification -- `Some` took the
            // early-return sea/lake colour, `None` fell through to
            // `land_color`. The reference's own tile renderer (11719 ocean,
            // 11753 lake) never reaches its per-pixel material path for a
            // water pixel at all: both branches `continue` before it. This
            // ink stage is a port-only addition on top of that path (see the
            // section header above), so it has to repeat the same land-only
            // guard rather than being reachable for a `water` pixel the
            // reference itself never colours through the land path.
            let ci = (wy.round().clamp(0.0, (gh - 1) as f64) as usize) * gw + (wx.round().clamp(0.0, (gw - 1) as f64) as usize);
            let ink_t = if water.is_some() { 0.0 } else { ink.map_or(0.0, |mk| save_flag_at(mk, ci)) as f64 };
            // `channel_tint`'s contract is [`bake_rect`]'s (and the screen's
            // own `build_color_texture`): an Rgb normalised to `[0, 1]`, which
            // is what its two `.min(1.0)` clamps and its `0.3`/`0.45` literals
            // are written against (see its own doc comment's blend, and
            // `bake_rect`'s `(r * 255.0) as u8` right after the same call).
            // This tile loop's own `cr, cg, cb` are this **file's other**
            // convention -- byte-scale `[0, 255]`, the same scale
            // `u8_clamped` consumes a few lines below with no further
            // multiply. Calling `channel_tint` directly on the byte-scale
            // triple made `g * 0.5 + 0.3` and `b * 0.5 + 0.45` clamp to `1.0`
            // for any pixel brighter than a few levels -- collapsing the
            // green and blue channels of the tint's own target colour to
            // near-black while the red channel stayed at half its value, so
            // every inked river pixel this loop ever drew went dark red-brown
            // instead of the intended blue-green brightening. Scale down,
            // tint, scale back -- `bake_rect`'s own round trip, not a new one.
            let (cr, cg, cb) = if ink_t > 1.0 / 255.0 {
                let (tr, tg, tb) = channel_tint(a, (cr / 255.0, cg / 255.0, cb / 255.0), ink_t, wx, wy, gw, gh);
                (tr * 255.0, tg * 255.0, tb * 255.0)
            } else {
                (cr, cg, cb)
            };

            // --- 5. quantize, then the whole-raster stages -------------------
            // `Uint8ClampedArray`'s `ToUint8Clamp` (round, ties to even) — the
            // reference's own store, and the one `tile_render.rs` is pinned
            // against. `build_color_texture` truncates instead, so a tile and
            // the screen can land one level apart on the same colour; that is
            // the whole of the quantization difference and it is measured in
            // `tests/golden_parity_tile_biome.rs`.
            let o = x * 3;
            out[o] = cartalith_jsmath::u8_clamped(cr);
            out[o + 1] = cartalith_jsmath::u8_clamped(cg);
            out[o + 2] = cartalith_jsmath::u8_clamped(cb);
        }
    });

    if has_grade_influence {
        influence.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
            let wy = bounds.y + y as f64 * cy;
            for (x, v) in row.iter_mut().enumerate() {
                *v = sample_arr(&tf.grade_influence, bounds.x + x as f64 * cx, wy, gw, gh) as f32;
            }
        });
    }
    // Local contrast, from the grid's detail band sampled at this pixel's world
    // coordinate — `LocalContrast::delta`'s own correction, faded out under the
    // plate frame exactly as it fades it — then the grade and the encode, all
    // through the shipped `finish_rgb` on the tile's own buffer, so a tile
    // cannot be finished or quantised differently from the map it sits over.
    let lc_delta = |i: usize| {
        let (wx, wy) = (bounds.x + (i % w) as f64 * cx, bounds.y + (i / w) as f64 * cy);
        let dd = sample_arr(&tf.contrast_d, wx, wy, gw, gh);
        let mut delta = a.local_contrast * dd * (-(dd * dd) * inv_knee2).exp();
        if delta != 0.0 {
            let cover = border_cover_f(a, wx, wy, gw, gh);
            if cover > 0.0 {
                delta *= 1.0 - cover;
            }
        }
        delta
    };
    finish_rgb(&mut rgb, contrast_on.then_some(lc_delta), Some((a, &influence)), tf.color_space);

    // RGB8 -> RGBA8. The buffer is RGB up to here so the finishing pass above
    // could be the shipped function rather than a tile-local copy of it.
    let mut out = vec![255u8; w * h * 4];
    out.par_chunks_mut(4).zip(rgb.par_chunks(3)).for_each(|(o, s)| {
        o[0] = s[0];
        o[1] = s[1];
        o[2] = s[2];
    });
    // **The river's coverage, in the alpha.** The tile is opaque everywhere
    // and its RGB already carries the river; alpha `255 - coverage` tells
    // `lod_tile.gdshader` which pixels are river, so it draws them from this
    // tile alone instead of mixing them with a coarser partner by LOD-D3's
    // `morph` (that mix left a cell-wide ghost of the base map's river around
    // every crisp one). No river layer,
    // or a pixel no stroke reached, keeps its `255`: the tile of every
    // existing caller and golden is byte-identical.
    //
    // **Land pixels only** (river mouths, 2026-09-27). A water pixel's RGB is
    // the water whatever the layer holds (stage 3's early return), and since
    // every end meeting water is carried on into it
    // (`river_stroke::extend_shore_ends`) the layer covers water pixels at
    // every mouth. Marking those as river would draw them from this tile
    // alone mid-morph while the water beside them mixes with the parent -- a
    // river-shaped seam in the water, the stroke showing through the water
    // that must sit above it. The test is stage 3's own, at the same
    // position.
    if let Some(layer) = rivers {
        out.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
            let wy = bounds.y + y as f64 * cy;
            for x in 0..w {
                if let Some(p) = layer.at(x, y) {
                    let ht = tile[(y + pad) * pw + x + pad] as f64;
                    let wx = bounds.x + x as f64 * cx;
                    if ht < sl || (lakes && is_lake_pixel(tf, ctx, wx, wy, ht, lake_fill_ok)) {
                        continue;
                    }
                    row[x * 4 + 3] = 255 - (p[3].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
    }
    out
}

/// **RV-4's clamp**, in height units: [`shore_depth`] never returns less than
/// this on a water cell nor more than its negative on a land one, so every
/// cell centre keeps the class `build_water_bodies` gave it however the
/// ground lies. Labelled judgement, not a measured value: it binds only where
/// the margins and the classification disagree (a tie at exactly zero -- a
/// cell exactly at sea level is land; a land cell between two waters measured
/// against the higher one), and is small enough there that the shoreline stays
/// within a few thousandths of a cell of the land centre. It must stay above half-float's smallest
/// subnormal (`2^-24`, about `6e-8`), since the shell's copy of the field is
/// half-float ([`f16_bits`]) and a zero would lose the sign.
pub const SHORE_EPS: f64 = 1e-5;

/// [`shore_depth`] on a land cell with no water in its 8-neighbourhood. Such a
/// cell is never a corner of a square that holds water (the four corners of a
/// square are each within the others' 8-neighbourhood), so the value is read
/// by nothing that contours; any negative would do, and a whole unit of
/// height (the field's full range) says plainly "far from any water". (Its
/// sign is also forced by [`shore_depth`]'s land clamp, so a mutation of it
/// survives the tests: the value is documentation of intent, not load.)
pub const SHORE_FAR_LAND: f64 = -1.0;

/// **RV-4: the two margins of the lake rule at cell (`x`, `y`)**, unclamped:
/// `(depth, rain)`, each positive where that half of the rule says water.
///
/// `build_water_bodies` makes an above-sea cell lake where BOTH its pooled
/// depth exceeds [`cartalith_civ::LAKE_DEPTH`] AND its rainfall reaches
/// [`cartalith_civ::LAKE_RAIN`]; below sea level everything is water. So a
/// shoreline is the zero line of one of two smooth fields, and which one is a
/// fact about the place: where the ground rises out of the water it is the
/// terrain, and where the pooled surface runs on into drier country the
/// rainfall threshold cuts the lake off (measured on seed 483920's lake at
/// cell 671,97: land cells 0.02 below the water beside them, kept dry by the
/// rain gate -- the edge the membership cut then drew as steps).
///
/// - `depth`: the surface of the water (for a land cell, of the water beside
///   it) minus the ground, in height units. The surface is `sea_level` for the
///   ocean; for a lake the larger of `sea_level` and `fill - LAKE_DEPTH`, the
///   level the classification thresholds. A land cell takes the highest
///   surface among the water cells in its 8-neighbourhood (the water it is the
///   shore of), and [`SHORE_FAR_LAND`] when there is none. On a sea coast the
///   zero line is the height field's own coastline -- the one
///   `cartalith_terrain::vector::trace_coastline` traces.
/// - `rain`: `SHORE_RAIN_SCALE * (rainfall - LAKE_RAIN)` where that surface
///   is an above-sea lake's (the only water the rain gate applies to), and
///   [`SHORE_RAIN_UNGATED`] elsewhere.
///
/// `class`/`fill` are one `build_water_bodies` call's
/// `classification`/`fill_level`, `rain` the rainfall it was given, all `gw *
/// gh` (the caller checks). `world` wraps the neighbourhood in x, as that
/// call's own flood does. Must never be used to classify.
#[allow(clippy::too_many_arguments)]
pub fn shore_margins(field: &[f32], class: &[u8], fill: &[f32], rain: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool, x: usize, y: usize) -> (f64, f64) {
    // `(surface, rain-gated)` of water cell `j`.
    // No ocean branch: the flood seeds every ocean cell at its own (below-sea)
    // height, so its `fill - LAKE_DEPTH` is below sea level and the second
    // arm already answers `sea_level` for it (a separate `class == 1` arm was
    // written first and survived mutation as the same answer).
    let surface = |j: usize| -> (f64, bool) {
        let lake = fill[j] as f64 - cartalith_civ::LAKE_DEPTH;
        // Above sea level the flood pass (and its rain gate) made this lake;
        // at or below it the cell is water whatever the rain.
        if lake > sea_level { (lake, true) } else { (sea_level, false) }
    };
    let i = y * gw + x;
    let h = field[i] as f64;
    let (level, gated) = if class[i] != 0 {
        surface(i)
    } else {
        let mut best: Option<(f64, bool)> = None;
        for dy in -1i64..=1 {
            let yy = y as i64 + dy;
            if yy < 0 || yy >= gh as i64 {
                continue;
            }
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut xx = x as i64 + dx;
                // A wrapped world's x edge is not an edge (the flood's own rule).
                if xx < 0 || xx >= gw as i64 {
                    if !world {
                        continue;
                    }
                    xx = xx.rem_euclid(gw as i64);
                }
                let j = yy as usize * gw + xx as usize;
                if class[j] != 0 {
                    let s = surface(j);
                    if best.is_none_or(|b| s.0 > b.0) {
                        best = Some(s);
                    }
                }
            }
        }
        match best {
            Some(b) => b,
            None => return (SHORE_FAR_LAND, SHORE_RAIN_UNGATED),
        }
    };
    let r = if gated { SHORE_RAIN_SCALE * (rain[i] as f64 - cartalith_civ::LAKE_RAIN) } else { SHORE_RAIN_UNGATED };
    (level - h, r)
}

/// Height units per unit of rainfall margin in [`shore_margins`]' `rain`.
/// Both fields are the engine's normalised `0..1` rasters, so `1.0` keeps
/// them on one footing. Labelled judgement, and a weak one to get wrong: the
/// zero line of `min(depth, k * rain)` is the same for every `k > 0` at the
/// cell centres; `k` only weighs the two where they are interpolated across
/// one cell square that holds both kinds of edge.
pub const SHORE_RAIN_SCALE: f64 = 1.0;

/// [`shore_margins`]' `rain` where no rain gate applies (the sea, water below
/// sea level, a land cell with no water beside it): larger than any real
/// margin, `SHORE_RAIN_SCALE * (1 - LAKE_RAIN)` = 0.78, so the `min` in
/// [`shore_depth`] always takes `depth` there.
pub const SHORE_RAIN_UNGATED: f64 = 1.0;

/// **RV-4: the signed water depth at cell (`x`, `y`)** -- the smaller of the
/// two [`shore_margins`], so positive where the lake rule (or the sea) says
/// water and negative where it says land, and zero on the smooth shoreline
/// between cell centres. The shore field ([`shore_field`]) is this at every
/// cell.
///
/// Then clamped by class, `>= SHORE_EPS` on water and `<= -SHORE_EPS` on land,
/// which is what makes the drawn water agree with the classification at every
/// cell centre and makes a point between four water centres always water
/// (bilinear of positives). The margins already agree with the class at every
/// cell the rule classifies; the clamp is for the rest (a tie at exactly zero,
/// and the neighbour-surface choice on a land cell between two waters). The
/// water half never binds today -- a water cell's margins are strictly
/// positive by the rule that made it water (`LAKE_RAIN` is not an `f32`, so
/// the rain margin cannot be exactly zero) -- and survives mutation for that
/// reason; it is kept as the stated guarantee the shader relies on. Why
/// it exists: the base map drew whole-cell water -- square steps (owner,
/// 2026-09-27). Must never be used to classify: it moves no cell between land
/// and water, and it is not written back to anything.
#[allow(clippy::too_many_arguments)]
pub fn shore_depth(field: &[f32], class: &[u8], fill: &[f32], rain: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool, x: usize, y: usize) -> f64 {
    let (d, r) = shore_margins(field, class, fill, rain, gw, gh, sea_level, world, x, y);
    let v = d.min(r);
    if class[y * gw + x] != 0 { v.max(SHORE_EPS) } else { v.min(-SHORE_EPS) }
}

/// **RV-4: the shore field** -- [`shore_depth`] at every cell, row-major,
/// `gw * gh`; empty when any input is shorter than that. The shell contours
/// its bilinear interpolation at zero to draw the base map's coast at
/// sub-cell precision (`shell/map_shore.gdshader`, and the river stroke's
/// `river_under_water.gdshader`, which must hide under exactly that water).
///
/// Cost: one pass, nine neighbour reads per land cell, `rayon` by rows --
/// linear in cells, the same order as the classification it reads.
/// No shipping caller since Ruling BO (`build_color_texture` calls
/// [`shore_field_forced`]); kept as the no-forcing form this file's own RV-4
/// tests pin the field through.
#[allow(clippy::too_many_arguments, dead_code)]
pub fn shore_field(field: &[f32], class: &[u8], fill: &[f32], rain: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool) -> Vec<f32> {
    shore_field_forced(field, class, fill, rain, None, gw, gh, sea_level, world)
}

/// [`shore_field`] with Ruling BO's forced lakes: [`shore_depth_forced`] at
/// every cell. `forced` `None` (or not `gw * gh` long) is exactly
/// [`shore_field`] -- that function is this one with `None`, so a world with
/// no forced lake draws the field it always drew.
#[allow(clippy::too_many_arguments)]
pub fn shore_field_forced(field: &[f32], class: &[u8], fill: &[f32], rain: &[f32], forced: Option<&[u8]>, gw: usize, gh: usize, sea_level: f64, world: bool) -> Vec<f32> {
    let n = gw * gh;
    if n == 0 || field.len() < n || class.len() < n || fill.len() < n || rain.len() < n {
        return Vec::new();
    }
    let mut out = vec![0f32; n];
    out.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        for (x, v) in row.iter_mut().enumerate() {
            *v = shore_depth_forced(field, class, fill, rain, forced, gw, gh, sea_level, world, x, y) as f32;
        }
    });
    out
}

/// Ruling BO's post-pass on a drawn classification: every cell `forced`
/// marks becomes a lake (`2`) -- `cartalith_civ::apply_force_lake`, the
/// reference's `forceLake`, which is its whole body.
///
/// A plain function so the drawn map, the sculpt preview, the PNG export, the
/// LOD tiles and the civ layer apply one rule, and so it is testable without a `Gd<WorldGen>`. A mask
/// whose length is not the classification's is **ignored rather than
/// applied**: it was drawn over another grid (`forced_lakes` is reset with the
/// world, so this is defence, not a live path), and `apply_force_lake`'s own
/// short-mask tolerance would otherwise force the wrong cells. `None` is a
/// no-op, which is what keeps every world without a forced lake -- and every
/// golden -- byte-identical.
#[allow(dead_code)]
pub fn apply_forced_lakes(classification: &mut [u8], forced: Option<&[u8]>) {
    if let Some(m) = forced
        && m.len() == classification.len()
    {
        cartalith_civ::apply_force_lake(classification, m);
    }
}

/// The grid's B4 biome-boundary distance (`sdf_biomes`): the water-body
/// classification, Ruling BO's forced lakes applied to it, the biome raster
/// classified from that, and its distance transform.
///
/// One body for [`GridPrecompute::build_forced`] and
/// [`RenderCtx::with_map_scale_forced`], so the cached (LOD, export session)
/// and uncached (screen, sculpt preview, PNG export) paths cannot disagree
/// about which cells are water. A forced cell becomes a lake (`2`) before
/// `build_biome_raster` runs, and that raster lets water override climate
/// (`cartalith_civ`'s `build_biome_raster_water_overrides_climate`), so a
/// forced lake gets the same water-edge band a natural lake gets. Before
/// 2026-09-28 this leg rebuilt the classification without the mask and
/// classified a forced lake as the land biome under it -- no band.
///
/// `forced: None` computes exactly what this leg computed before the mask
/// existed. It must never be given a mask built over another grid: that is
/// [`apply_forced_lakes`]' length check, which ignores it. It is NOT the LOD
/// tile's own per-tile raster (`render_biome_tile_rgba`), which follows the
/// reference in treating only below-sea cells as water and so draws no band
/// at any lake, natural or forced.
#[allow(clippy::too_many_arguments)]
fn grid_biome_boundary_dist(field: &[f32], temperature: &[f32], rainfall: &[f32], gw: usize, gh: usize, sea_level: f64, world: bool, forced: Option<&[u8]>) -> Vec<f32> {
    let mut class = cartalith_civ::build_water_bodies(field, gw, gh, sea_level, world, Some(rainfall)).classification;
    apply_forced_lakes(&mut class, forced);
    let biome = cartalith_civ::build_biome_raster(&class, temperature, rainfall);
    build_biome_boundary_dist(&biome, gw, gh)
}

/// **Ruling BO: the signed depth a forced lake's shore is contoured at**, in
/// height units -- `+` this on a forced cell, `-` this on a land cell whose
/// only water neighbours are forced ([`shore_depth_forced`]).
///
/// Equal and opposite on purpose: the bilinear of `+F` and `-F` crosses zero
/// exactly halfway, so a forced lake's shore runs midway between its outer
/// cell centres and the land beside them -- the membership contour of the
/// painted cells, as smooth as the base map's other shores and the same
/// outline the reference's nearest-cell stamp for a flat painted lake
/// (`is_lake_pixel`'s v1.05 fallback) approximates in squares. The magnitude
/// is labelled judgement, not a measured value: small (a thousandth of the
/// field's `0..1` range), so where a forced lake touches a natural shore the
/// natural field's own margins, usually larger, dominate that edge rather
/// than this number; and far above half-float's smallest subnormal, so the
/// shell's `RH` copy keeps it exactly ([`f16_bits`]).
pub const FORCED_SHORE_DEPTH: f64 = 1e-3;

/// [`shore_depth`] with Ruling BO's forced lakes laid over it.
///
/// The lake rule [`shore_margins`] reads never made a forced cell water -- a
/// forced lake is water *because the user said so* (the reference's
/// `forceLake`), whether or not its basin pools or its rain passes
/// `LAKE_RAIN` -- so the field alone draws it as a sliver at its cell
/// centres (the class clamp gives it [`SHORE_EPS`] against land a whole
/// height step below). Three cases, in order:
///
/// - a forced cell: at least [`FORCED_SHORE_DEPTH`] -- more where the natural
///   field already says deeper water;
/// - any other water cell: the natural depth, untouched;
/// - a land cell: exactly `-FORCED_SHORE_DEPTH` when every water cell in its
///   8-neighbourhood is forced (so the shore between it and the forced lake
///   runs halfway), and the natural depth when any neighbour is natural
///   water -- a natural shore keeps its own contour.
///
/// `forced` must be the mask already applied to `class` (`class` is `2` on
/// every forced cell). `None`, or a mask that is not `gw * gh`, is exactly
/// [`shore_depth`]. `world` wraps the neighbourhood in x, as [`shore_margins`]
/// does. Must never be used to classify.
#[allow(clippy::too_many_arguments)]
pub fn shore_depth_forced(field: &[f32], class: &[u8], fill: &[f32], rain: &[f32], forced: Option<&[u8]>, gw: usize, gh: usize, sea_level: f64, world: bool, x: usize, y: usize) -> f64 {
    let d = shore_depth(field, class, fill, rain, gw, gh, sea_level, world, x, y);
    let Some(m) = forced.filter(|m| m.len() == gw * gh) else { return d };
    let i = y * gw + x;
    if m[i] != 0 {
        return d.max(FORCED_SHORE_DEPTH);
    }
    // A natural water cell: the forcing does not move its surface.
    if class[i] != 0 {
        return d;
    }
    let mut near_forced = false;
    for dy in -1i64..=1 {
        let yy = y as i64 + dy;
        if yy < 0 || yy >= gh as i64 {
            continue;
        }
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let mut xx = x as i64 + dx;
            // A wrapped world's x edge is not an edge (`shore_margins`' rule).
            if xx < 0 || xx >= gw as i64 {
                if !world {
                    continue;
                }
                xx = xx.rem_euclid(gw as i64);
            }
            let j = yy as usize * gw + xx as usize;
            if class[j] != 0 {
                // Natural water beside this land: its shore is the lake
                // rule's, and a forced neighbour must not drag it.
                if m[j] == 0 {
                    return d;
                }
                near_forced = true;
            }
        }
    }
    if near_forced { -FORCED_SHORE_DEPTH } else { d }
}

/// IEEE 754 half-float bits of `v`, rounded to nearest, for the shore field's
/// `RH` texture (2 bytes a cell where `RF` would take 4 -- at the 8192-wide
/// ceiling, 86 MB instead of 172). **Never rounds a non-zero value to zero**:
/// a value below half-float's smallest subnormal comes back as that subnormal
/// with its sign, because the shader reads the sign of every cell as its class
/// ([`SHORE_EPS`]). Overflow saturates to infinity; NaN stays NaN. No crate:
/// the workspace carries no half-float type, and this is the one caller.
pub fn f16_bits(v: f32) -> u16 {
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32;
    let man = b & 0x007f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if man != 0 { 0x0200 } else { 0 };
    }
    if exp == 0 && man == 0 {
        return sign;
    }
    // Rebias 127 -> 15.
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    if e <= 0 {
        // Subnormal: shift the full significand (implicit 1 restored) down.
        let m = man | 0x0080_0000;
        let shift = (14 - e) as u32;
        let r = if shift >= 32 { 0 } else { (m + (1u32 << (shift - 1))) >> shift };
        // The sign is the class: never let a non-zero value become zero.
        return sign | (r.max(1) as u16);
    }
    // Normal: round the 23-bit mantissa to 10 bits; a carry rolls into the
    // exponent, which is the right answer -- and from the top exponent (`e`
    // is at most 0x1e here) it lands exactly on 0x7c00, infinity, so no
    // separate saturation is needed (one was written and survived mutation).
    let out = ((e as u32) << 10) + ((man + 0x0000_1000) >> 13);
    sign | out as u16
}

/// Whether an export-bake pixel at grid position (`gx`, `gy`) lies in an
/// above-sea lake (class `2`): all four surrounding cells lake -> yes, none ->
/// no, mixed -> the nearest cell decides. [`is_lake_pixel`]'s rule for a tile
/// with no lake-fill surface, so the export's shoreline is the one a tile
/// drawn without that surface has.
fn bake_lake_at(lake: &[u8], gx: f64, gy: f64, gw: usize, gh: usize) -> bool {
    let fx = gx.clamp(0.0, gw as f64 - 1.001);
    let fy = gy.clamp(0.0, gh as f64 - 1.001);
    let (x0, y0) = (fx as usize, fy as usize);
    let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
    let n = [lake[y0 * gw + x0], lake[y0 * gw + x1], lake[y1 * gw + x0], lake[y1 * gw + x1]]
        .iter()
        .filter(|&&c| c == 2)
        .count();
    match n {
        4 => true,
        0 => false,
        _ => {
            let ix = gx.round().clamp(0.0, (gw - 1) as f64) as usize;
            let iy = gy.round().clamp(0.0, (gh - 1) as f64) as usize;
            lake[iy * gw + ix] == 2
        }
    }
}

/// The v1.05 lake shoreline (reference 11717-11740), issue #96 *"square lakes
/// when LOD zooming"*.
///
/// The old test stamped the whole coarse cell from a nearest-neighbour lake
/// sample, so a lake magnified past the grid resolution read as axis-aligned
/// blue squares. This one follows the terrain: deep inside the lake (all four
/// surrounding coarse cells are lake) the pixel is water outright; on the
/// boundary band it is water only where the tile's own amplified terrain lies
/// **below the pooled lake surface** `_lakeFill` captured from
/// `buildWaterBodies`' priority-flood — so the shore is the curve where the
/// visible ground rises out of the water.
///
/// Two fallbacks, both the reference's:
///
/// - where the shelf is flat, `h < s` would hold across the whole band and
///   degenerate to a straight window-limit edge, so the bilinear membership
///   fraction `fq > 0.35` cuts a marching-squares curve between the cell
///   centres instead;
/// - a water-brush or otherwise flat lake has pooled nothing (`fill - field <=
///   0.004`) and keeps its painted cell shape through the nearest-cell stamp.
///
/// `fill_ok` false is the reference's own `_lakeFill`-missing branch, a plain
/// nearest-cell stamp. This port always has a fill surface —
/// `cartalith_civ::WaterBodies::fill_level` is `fillOut` from the same flood —
/// so it is reachable only through a length mismatch, and it is kept rather
/// than dropped because the alternative to a stated fallback is silently
/// drawing squares.
///
/// **RV-4 (2026-09-27): the smooth band, `TerrainAppearance::smooth_shores`.**
/// Both fallbacks above draw squares -- the membership cut is a curve between
/// cell centres of a 0/1 field, so a staircase of lake cells stays a staircase
/// with its corners cut, and the flat-lake stamp is the cell itself; the z16
/// lake shores the owner saw (2026-09-27) were exactly those. With the flag
/// on, the band pixel is water where the base map's own shore field says so:
/// the bilinear of [`shore_depth`] at the pixel's four corners is positive.
/// That field reads `build_water_bodies`' lake rule smoothly -- the ground
/// rising through the pooled surface less [`cartalith_civ::LAKE_DEPTH`], and
/// the rainfall crossing [`cartalith_civ::LAKE_RAIN`] ([`shore_margins`]) --
/// so the shore is a smooth curve, and a lake keeps exactly one outline from
/// fit zoom to the deepest tile.
///
/// **The tile's own detail (`ht`) does not move it**, unlike the sea's
/// `ht < sl` and the v1.05 band. Tried first and measured on seed 483920: with
/// `ht` the lake shores went ragged tile pixel by tile pixel wherever the
/// ground is nearly as high as the water (the added noise decides every pixel
/// there), which is a pixel artefact of its own, and a river running beside
/// such a shore showed through the tatters (`_rivstyle_probe.gd` M3, 4-7 px
/// at two lake mouths). A lake surface is flat, so its outline at this scale
/// is the field's contour; the sea keeps the reference's rule, which HEAD
/// already drew smooth.
///
/// All four corners lake is water outright, as before (the region
/// `river_stroke.rs` carries a river mouth into; the field is positive there
/// anyway), and no lake corner is land. `smooth_shores` false
/// (`js_reference()`) runs the v1.05 body untouched.
fn is_lake_pixel(tf: &TileFields, ctx: &RenderCtx, wx: f64, wy: f64, ht: f64, fill_ok: bool) -> bool {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let lake = &tf.lake_class;
    let fx = wx.clamp(0.0, gw as f64 - 1.001);
    let fy = wy.clamp(0.0, gh as f64 - 1.001);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let x1 = (x0 + 1).min(gw - 1);
    let y1 = (y0 + 1).min(gh - 1);
    let (la, lb, lc, ld) = (lake[y0 * gw + x0] == 2, lake[y0 * gw + x1] == 2, lake[y1 * gw + x0] == 2, lake[y1 * gw + x1] == 2);
    let n_l = la as u32 + lb as u32 + lc as u32 + ld as u32;
    if n_l == 4 {
        return true;
    }
    // No lake corner: land, on both bands. (The smooth band's own rule could
    // not say water here anyway: every corner's `shore_depth` is negative.)
    if n_l == 0 {
        return false;
    }
    let ix = (wx.round().clamp(0.0, (gw - 1) as f64)) as usize;
    let iy = (wy.round().clamp(0.0, (gh - 1) as f64)) as usize;
    let ni = iy * gw + ix;
    if !fill_ok {
        return lake[ni] == 2;
    }
    // RV-4: the smooth band (see this function's doc) -- the base map's own
    // shoreline: water where the bilinear of the four corners' `shore_depth`
    // is positive, the same number `map_shore.gdshader` contours. `ht`, the
    // tile's added detail, does not move it. Never reached with
    // `smooth_shores` false, which is what keeps the reference's own band
    // below byte-identical.
    if ctx.appearance.smooth_shores {
        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;
        // Ruling BO: the base map's own forced-lake edge
        // (`shore_depth_forced`), so a forced lake keeps one outline from fit
        // zoom into the tiles. `None` for a world with none.
        let forced = Some(&tf.lake_forced[..]).filter(|m| m.len() == gw * gh);
        let d = |x: usize, y: usize| shore_depth_forced(ctx.field, lake, &tf.lake_fill, ctx.rainfall, forced, gw, gh, ctx.sea_level, ctx.world, x, y);
        let s = (1.0 - tx) * (1.0 - ty) * d(x0, y0) + tx * (1.0 - ty) * d(x1, y0) + (1.0 - tx) * ty * d(x0, y1) + tx * ty * d(x1, y1);
        return s > 0.0;
    }
    let fill = &tf.lake_fill;
    let mut s = -1.0f64;
    if la {
        s = s.max(fill[y0 * gw + x0] as f64);
    }
    if lb {
        s = s.max(fill[y0 * gw + x1] as f64);
    }
    if lc {
        s = s.max(fill[y1 * gw + x0] as f64);
    }
    if ld {
        s = s.max(fill[y1 * gw + x1] as f64);
    }
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let fq = (if la { (1.0 - tx) * (1.0 - ty) } else { 0.0 }) + (if lb { tx * (1.0 - ty) } else { 0.0 }) + (if lc { (1.0 - tx) * ty } else { 0.0 }) + (if ld { tx * ty } else { 0.0 });
    if ht < s && fq > LAKE_MEMBERSHIP_MIN {
        return true;
    }
    lake[ni] == 2 && (fill[ni] as f64 - ctx.field[ni] as f64) <= LAKE_FLAT_EPS
}

/// RV-4's smooth shoreline (`shore_margins`, `shore_depth`, `shore_field`,
/// `f16_bits`, and `is_lake_pixel`'s smooth band), each pinned against an
/// independent answer: an analytic shoreline the test worlds are built with,
/// or a literal bit pattern -- never against the constants under test.
#[cfg(test)]
mod shore_tests {
    use super::*;

    const SL: f64 = 0.42;

    /// Bilinear between cell centres at cell-index coordinates `(u, v)`
    /// (centres at integers) -- the shader's `shore_value`, in Rust.
    fn bilinear(f: &[f32], gw: usize, u: f64, v: f64) -> f64 {
        let (x0, y0) = (u.floor() as usize, v.floor() as usize);
        let (tx, ty) = (u - x0 as f64, v - y0 as f64);
        let g = |x: usize, y: usize| f[y * gw + x] as f64;
        (1.0 - tx) * (1.0 - ty) * g(x0, y0) + tx * (1.0 - ty) * g(x0 + 1, y0) + (1.0 - tx) * ty * g(x0, y0 + 1) + tx * ty * g(x0 + 1, y0 + 1)
    }

    /// A tilted plane crossing sea level along a 27-degree line: no lakes,
    /// one ocean, and a coastline known exactly.
    #[allow(clippy::type_complexity)]
    fn plane() -> (Vec<f32>, Vec<f32>, usize, usize, Box<dyn Fn(f64, f64) -> f64>) {
        let (gw, gh) = (32usize, 32usize);
        let (c, s) = (27f64.to_radians().cos(), 27f64.to_radians().sin());
        let h = move |u: f64, v: f64| SL + 0.004 * ((u - 15.3) * c + (v - 15.7) * s);
        let field = (0..gw * gh).map(|i| h((i % gw) as f64, (i / gw) as f64) as f32).collect();
        (field, vec![0.5f32; gw * gh], gw, gh, Box::new(h))
    }

    /// A bowl (radius 8 cells, floor 0.6) in a 0.8 plateau: the priority
    /// flood pools it to 0.8, so it is lake where `0.8 - h > LAKE_DEPTH`,
    /// r < 7.92 -- and rainfall `0.22 + 0.02 * (18.3 - x)` falls below the
    /// rain gate at x = 18.3, cutting the bowl's east side off dry.
    fn bowl() -> (Vec<f32>, Vec<f32>, usize, usize) {
        let (gw, gh) = (32usize, 32usize);
        let mut field = vec![0f32; gw * gh];
        let mut rain = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let r = ((x as f64 - 16.0).powi(2) + (y as f64 - 16.0).powi(2)).sqrt();
                field[y * gw + x] = if r < 8.0 { 0.6 + 0.2 * (r / 8.0).powi(2) } else { 0.8 } as f32;
                rain[y * gw + x] = (0.22 + 0.02 * (18.3 - x as f64)) as f32;
            }
        }
        (field, rain, gw, gh)
    }

    /// The zero crossing of `f` between `a` and `b`, by bisection (the
    /// function changes sign once there in every case below).
    fn crossing(f: impl Fn(f64) -> f64, mut a: f64, mut b: f64) -> f64 {
        assert!(f(a) * f(b) < 0.0, "no crossing between {a} and {b}");
        for _ in 0..60 {
            let m = 0.5 * (a + b);
            if (f(m) > 0.0) == (f(a) > 0.0) {
                a = m
            } else {
                b = m
            }
        }
        0.5 * (a + b)
    }

    /// Protects `shore_margins`' sea branch: the base map's coast is the
    /// height field's own contour at sea level, sub-cell, not a cell edge.
    /// At every sample point the sign of the bilinear field is the side of
    /// the analytic line, and (premise) the nearest cell's class is wrong at
    /// a good share of those same points -- the step the field removes.
    #[test]
    fn the_sea_shore_is_the_height_fields_own_contour() {
        let (field, rain, gw, gh, h) = plane();
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let sf = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        assert_eq!(sf.len(), gw * gh);
        let (mut n, mut nearest_wrong) = (0, 0);
        for j in 0..200 {
            for i in 0..200 {
                let (u, v) = (3.0 + 25.0 * i as f64 / 199.0, 3.0 + 25.0 * j as f64 / 199.0);
                let truth = SL - h(u, v);
                // f32 storage of the heights: within a few 1e-8 of the line
                // either answer is right.
                if truth.abs() < 1e-6 {
                    continue;
                }
                n += 1;
                assert_eq!(bilinear(&sf, gw, u, v) > 0.0, truth > 0.0, "wrong side at ({u:.3}, {v:.3})");
                let near = wb.classification[(v.round() as usize) * gw + u.round() as usize] != 0;
                nearest_wrong += (near != (truth > 0.0)) as usize;
            }
        }
        assert!(n > 39_000);
        // The cell coast is wrong in a sliver along the line (measured 411
        // of the 40 000 samples); what matters is that it is wrong at all
        // where the field is right everywhere.
        assert!(nearest_wrong > 200, "premise: the cell coast is off the line somewhere ({nearest_wrong})");
    }

    /// Protects `shore_depth`'s clamp and both of `shore_margins`' lake
    /// halves: over a real `build_water_bodies` answer (the producer the app
    /// uses), every cell centre keeps its class -- water positive, land
    /// negative -- with a lake that has both a terrain edge and a rain edge.
    #[test]
    fn every_cell_centre_keeps_its_class() {
        let (field, rain, gw, gh) = bowl();
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let lake = wb.classification.iter().filter(|&&c| c == 2).count();
        // Premise: a lake cut by the rain gate on its east side, not a whole bowl.
        assert!(lake > 100, "the bowl pools ({lake} lake cells)");
        assert_eq!(wb.classification[16 * gw + 18], 2, "west of the rain line is lake");
        assert_eq!(wb.classification[16 * gw + 19], 0, "east of it is dry although the bowl is deep there");
        let sf = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        for (i, &v) in sf.iter().enumerate() {
            assert_eq!(v > 0.0, wb.classification[i] != 0, "cell {i}: field {v} against class {}", wb.classification[i]);
            assert!(v != 0.0, "cell {i} sits on the shoreline");
        }
    }

    /// Protects `shore_margins`' rain half and its depth half on a lake
    /// shore: along the bowl's middle row the drawn shoreline crosses where
    /// the ground rises through the lake's surface (`0.8 - LAKE_DEPTH`) and
    /// where the rainfall crosses the gate (x = 18.3) -- sub-cell, where the
    /// cell coast would sit on the edges at 8.5 and 18.5.
    #[test]
    fn a_lake_shore_follows_the_ground_and_the_rain_line() {
        let (field, rain, gw, gh) = bowl();
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let sf = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        let row = |u: f64| bilinear(&sf, gw, u, 16.0);
        let west = crossing(row, 7.5, 9.5);
        let east = crossing(row, 17.5, 19.5);
        // The ground between centres 8 (0.8, the plateau) and 9 (0.6 + 0.2 *
        // (7/8)^2 = 0.753125) crosses the surface 0.796 at t = 0.004 / 0.046875
        // = 0.0853 past centre 8.
        assert!((west - 8.0853).abs() < 0.001, "west shore at {west}");
        assert!((east - 18.3).abs() < 0.001, "east shore at {east}");
    }

    fn tile_ctx<'a>(field: &'a [f32], rain: &'a [f32], temp: &'a [f32], gw: usize, gh: usize, smooth: bool) -> RenderCtx<'a> {
        let a = TerrainAppearance { smooth_shores: smooth, ..TerrainAppearance::default() };
        RenderCtx::with_appearance(field, temp, rain, None, gw, gh, SL, false, 70.0, -70.0, a)
    }

    /// Protects `is_lake_pixel`'s smooth band against the reference rule it
    /// replaces: along the bowl's middle row, with the tile's ground exactly
    /// the bilinear grid (no added detail), the smooth band's shores sit on
    /// the analytic ones (8.0853, 18.3); the reference's `fq > 0.35` cut puts
    /// them 0.35 of a cell from the lake centre (8.35, 18.65) -- the premise
    /// that this test can tell the two apart.
    #[test]
    fn the_tile_lake_band_is_the_smooth_shore_and_the_reference_is_not() {
        let (field, rain, gw, gh) = bowl();
        let temp = vec![15f32; gw * gh];
        for (smooth, west_want, east_want) in [(true, 8.0853, 18.3), (false, 8.35, 18.65)] {
            let ctx = tile_ctx(&field, &rain, &temp, gw, gh, smooth);
            let tf = TileFields::new(&ctx, None);
            let lake_at = |u: f64| {
                let ht = bilinear(&field, gw, u, 16.0);
                if is_lake_pixel(&tf, &ctx, u, 16.0, ht, true) {
                    1.0
                } else {
                    -1.0
                }
            };
            let west = crossing(lake_at, 7.5, 9.5);
            let east = crossing(lake_at, 17.5, 19.5);
            assert!((west - west_want).abs() < 0.002, "smooth {smooth}: west shore at {west}, want {west_want}");
            assert!((east - east_want).abs() < 0.002, "smooth {smooth}: east shore at {east}, want {east_want}");
        }
    }

    /// Protects the smooth band's identity with the base map: at every
    /// sample point around the bowl lake, the tile says lake exactly where the
    /// base map's shore field (`shore_field`, bilinear, the shader's rule) is
    /// positive -- whatever the tile's own ground `ht` does there, 0.05 above
    /// or below the grid's. (The reference band, run as the premise, disagrees
    /// with the field somewhere, so the comparison can fail.)
    #[test]
    fn the_tile_lake_band_is_the_base_maps_shoreline() {
        let (field, rain, gw, gh) = bowl();
        let temp = vec![15f32; gw * gh];
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let sf = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        for smooth in [true, false] {
            let ctx = tile_ctx(&field, &rain, &temp, gw, gh, smooth);
            let tf = TileFields::new(&ctx, None);
            let (mut n, mut differ) = (0, 0);
            for j in 0..120 {
                for i in 0..120 {
                    let (u, v) = (6.0 + 20.0 * i as f64 / 119.0, 6.0 + 20.0 * j as f64 / 119.0);
                    let want = bilinear(&sf, gw, u, v) > 0.0;
                    let hb = bilinear(&field, gw, u, v);
                    for ht in [hb - 0.05, hb + 0.05] {
                        // Below sea level the tile's sea test runs first; not this band.
                        if ht < SL {
                            continue;
                        }
                        n += 1;
                        let got = is_lake_pixel(&tf, &ctx, u, v, ht, true);
                        if smooth {
                            assert_eq!(got, want, "tile vs map at ({u:.3}, {v:.3}), ht {ht:.3}");
                        }
                        differ += (got != want) as usize;
                    }
                }
            }
            assert!(n > 20_000);
            if !smooth {
                assert!(differ > 100, "premise: the reference band is another shoreline ({differ})");
            }
        }
    }

    /// A dry 0.8 plateau (rain 0.1, below `LAKE_RAIN`, so the lake rule makes
    /// no water anywhere) with cells x 10..=14, y 10..=14 forced -- the
    /// arid basin a Lake stamp is for -- and its classification with the
    /// forcing applied, as `drawn_water_bodies` builds it.
    #[allow(clippy::type_complexity)]
    fn forced_plateau() -> (Vec<f32>, Vec<f32>, Vec<u8>, cartalith_civ::WaterBodies, usize, usize) {
        let (gw, gh) = (32usize, 32usize);
        let field = vec![0.8f32; gw * gh];
        let rain = vec![0.1f32; gw * gh];
        let mut mask = vec![0u8; gw * gh];
        for y in 10..=14 {
            for x in 10..=14 {
                mask[y * gw + x] = 1;
            }
        }
        let mut wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        assert!(wb.classification.iter().all(|&c| c == 0), "premise: the plateau has no water of its own");
        apply_forced_lakes(&mut wb.classification, Some(&mask));
        (field, rain, mask, wb, gw, gh)
    }

    /// Protects Ruling BO's forced-lake edge in the base map's shore field
    /// (`shore_field_forced`): the forced block is water (positive) at every
    /// cell centre and its shore runs exactly halfway to the land beside it,
    /// x = 9.5 and 14.5 along row 12 -- literal answers from the block's
    /// geometry, not from `FORCED_SHORE_DEPTH`. Without the mask the same
    /// classification draws only a sliver at the centres: the premise, and
    /// what the map drew before this ruling reached the field.
    #[test]
    fn a_forced_lake_is_drawn_to_halfway_between_its_cells_and_the_land() {
        let (field, rain, mask, wb, gw, gh) = forced_plateau();
        let sf = shore_field_forced(&field, &wb.classification, &wb.fill_level, &rain, Some(&mask), gw, gh, SL, false);
        for y in 10..=14 {
            for x in 10..=14 {
                assert!(sf[y * gw + x] > 0.0, "forced cell ({x}, {y}) reads {}", sf[y * gw + x]);
            }
        }
        assert!(sf[12 * gw + 20] < 0.0, "far land stays land");
        let row = |u: f64| bilinear(&sf, gw, u, 12.0);
        assert!((crossing(row, 9.0, 10.0) - 9.5).abs() < 1e-9);
        assert!((crossing(row, 14.0, 15.0) - 14.5).abs() < 1e-9);
        // A diagonal corner is cut, not squared: the block's outer corner
        // point (14.5, 14.5) is land.
        assert!(bilinear(&sf, gw, 14.5, 14.5) < 0.0);
        // The premise: the unforced field over the same classification puts
        // the shore within a thousandth of a cell of the forced centres.
        let bare = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        let bare_row = |u: f64| bilinear(&bare, gw, u, 12.0);
        assert!(crossing(bare_row, 14.0, 15.0) - 14.0 < 1e-3, "premise: without the mask the lake is a sliver");
    }

    /// Protects `shore_depth_forced`'s two no-op guarantees: `None` is
    /// `shore_field` bit for bit on a world with a real lake (the bowl), and a
    /// forced block far from that lake moves no cell of the lake's own
    /// shore -- a natural shore keeps its contour.
    #[test]
    fn forcing_leaves_every_natural_shore_where_it_was() {
        let (field, rain, gw, gh) = bowl();
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let plain = shore_field(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, SL, false);
        let none = shore_field_forced(&field, &wb.classification, &wb.fill_level, &rain, None, gw, gh, SL, false);
        assert_eq!(plain.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), none.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
        let mut mask = vec![0u8; gw * gh];
        for (x, y) in [(28, 28), (29, 28), (28, 29), (29, 29)] {
            mask[y * gw + x] = 1;
        }
        let mut class = wb.classification.clone();
        apply_forced_lakes(&mut class, Some(&mask));
        let forced = shore_field_forced(&field, &class, &wb.fill_level, &rain, Some(&mask), gw, gh, SL, false);
        let mut moved = 0;
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let near_block = (26..=30).contains(&x) && (26..=30).contains(&y);
                if !near_block {
                    assert_eq!(forced[i].to_bits(), plain[i].to_bits(), "cell ({x}, {y}) moved");
                } else if forced[i] != plain[i] {
                    moved += 1;
                }
            }
        }
        assert!(moved >= 4, "premise: the forced block itself changed the field ({moved})");
    }

    /// Protects the tiles' half of Ruling BO (`TileFields::with_forced_lakes`
    /// plus `is_lake_pixel`'s forced edge): the tile says lake exactly where
    /// the base map's forced shore field is positive, at 14 400 points over
    /// the block and its surroundings -- one outline from fit zoom to the
    /// deepest tile. Without the builder the tile draws none of it: the
    /// premise that this can fail.
    #[test]
    fn a_tile_draws_a_forced_lake_where_the_base_map_does() {
        let (field, rain, mask, wb, gw, gh) = forced_plateau();
        let temp = vec![15f32; gw * gh];
        let sf = shore_field_forced(&field, &wb.classification, &wb.fill_level, &rain, Some(&mask), gw, gh, SL, false);
        let ctx = tile_ctx(&field, &rain, &temp, gw, gh, true);
        let plain = TileFields::new(&ctx, None);
        let tf = TileFields::new(&ctx, None).with_forced_lakes(&mask);
        let (mut water, mut plain_water) = (0, 0);
        for j in 0..120 {
            for i in 0..120 {
                let (u, v) = (6.0 + 14.0 * i as f64 / 119.0, 6.0 + 14.0 * j as f64 / 119.0);
                let want = bilinear(&sf, gw, u, v) > 0.0;
                let got = is_lake_pixel(&tf, &ctx, u, v, 0.8, true);
                assert_eq!(got, want, "tile vs map at ({u:.3}, {v:.3})");
                water += got as usize;
                plain_water += is_lake_pixel(&plain, &ctx, u, v, 0.8, true) as usize;
            }
        }
        // The block spans 9.5..14.5 on both axes, 25 square cells less its
        // four cut corners, and the grid samples 14 x 14 cells at 120 x 120:
        // 25 / 196 * 14 400 = 1 837 is the uncut upper bound.
        assert!((1700..1837).contains(&water), "the forced block must draw ({water} lake samples)");
        assert_eq!(plain_water, 0, "premise: a tile without the mask draws no lake here");
        // A mask for another grid is refused, not indexed.
        let refused = TileFields::new(&ctx, None).with_forced_lakes(&mask[..10]);
        assert!(!is_lake_pixel(&refused, &ctx, 12.0, 12.0, 0.8, true));
    }

    /// A context with the `sdf_biomes` leg on at 0.4 (`golden_parity_tile_biome.rs`'s
    /// own test value; any value above zero builds the same distance field)
    /// and a map scale attached, with or without a forced-lake mask.
    fn biome_ctx<'a>(field: &'a [f32], rain: &'a [f32], temp: &'a [f32], gw: usize, gh: usize, forced: Option<&[u8]>) -> RenderCtx<'a> {
        let a = TerrainAppearance { sdf_biomes: 0.4, ..TerrainAppearance::default() };
        RenderCtx::with_appearance(field, temp, rain, None, gw, gh, SL, false, 70.0, -70.0, a).with_map_scale_forced(800.0, forced)
    }

    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }

    /// Protects the no-forced-lake path of the `sdf_biomes` leg
    /// (`grid_biome_boundary_dist`): on the bowl, which has a natural lake and
    /// so a real band, `with_map_scale`, `with_map_scale_forced(None)`, a
    /// wrong-length mask, and both `GridPrecompute` entry points all give,
    /// bit for bit, the pipeline this leg ran before the mask existed --
    /// written out here from the `cartalith_civ` calls, not from the helper.
    #[test]
    fn the_biome_band_without_a_forced_lake_is_unchanged() {
        let (field, rain, gw, gh) = bowl();
        let temp = vec![15f32; gw * gh];
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SL, false, Some(&rain));
        let old = build_biome_boundary_dist(&cartalith_civ::build_biome_raster(&wb.classification, &temp, &rain), gw, gh);
        assert!(old.iter().any(|&d| d == 0.0), "premise: the bowl's lake gives the band a boundary");
        let a = TerrainAppearance { sdf_biomes: 0.4, ..TerrainAppearance::default() };
        let plain = RenderCtx::with_appearance(&field, &temp, &rain, None, gw, gh, SL, false, 70.0, -70.0, a.clone()).with_map_scale(800.0);
        assert_eq!(bits(&plain.biome_bd), bits(&old));
        assert_eq!(bits(&biome_ctx(&field, &rain, &temp, gw, gh, None).biome_bd), bits(&old));
        let short = vec![1u8; 10];
        assert_eq!(bits(&biome_ctx(&field, &rain, &temp, gw, gh, Some(&short)).biome_bd), bits(&old), "a mask for another grid is ignored");
        let p = GridPrecompute::build(&field, &temp, &rain, None, gw, gh, SL, false, &a, Some(800.0));
        let pf = GridPrecompute::build_forced(&field, &temp, &rain, None, gw, gh, SL, false, &a, Some(800.0), None);
        assert_eq!(bits(&p.biome_bd), bits(&old));
        assert_eq!(bits(&pf.biome_bd), bits(&old));
    }

    /// Protects Ruling BO reaching the `sdf_biomes` leg: on the dry plateau,
    /// whose one climate is one biome, the unforced band has no boundary
    /// anywhere; with the 5x5 forced block the block is water and the band
    /// runs round its edge. Distances are literal answers from the block's
    /// geometry (x 10..=14 on row 12): `0` on both sides of the shore (9 and
    /// 10, 14 and 15), `2` at the block's centre (12) and `3` at x = 6. The
    /// cached path (`GridPrecompute::build_forced`, the LOD and export
    /// session) gives the same field as the screen's.
    #[test]
    fn a_forced_lake_gets_a_biome_band_at_its_edge() {
        let (field, rain, mask, _wb, gw, gh) = forced_plateau();
        let temp = vec![15f32; gw * gh];
        let at = |d: &[f32], x: usize| d[12 * gw + x];
        let bare = biome_ctx(&field, &rain, &temp, gw, gh, None);
        // Premise: without the mask nothing is a boundary, so the ecotone
        // widener is off (1.0) at the shore cell -- the pre-fix picture.
        assert!(bare.biome_bd.iter().all(|&d| d > 6.0), "premise: one biome, no band");
        assert_eq!(sdf_eco_k(at(&bare.biome_bd, 9) as f64, 0.4, gw), 1.0);
        let forced = biome_ctx(&field, &rain, &temp, gw, gh, Some(&mask));
        for (x, want) in [(9, 0.0), (10, 0.0), (14, 0.0), (15, 0.0), (12, 2.0), (6, 3.0)] {
            assert_eq!(at(&forced.biome_bd, x), want, "distance at ({x}, 12)");
        }
        assert!(sdf_eco_k(at(&forced.biome_bd, 9) as f64, 0.4, gw) > 1.0, "the shore cell is widened");
        let a = TerrainAppearance { sdf_biomes: 0.4, ..TerrainAppearance::default() };
        let pf = GridPrecompute::build_forced(&field, &temp, &rain, None, gw, gh, SL, false, &a, Some(800.0), Some(&mask));
        assert_eq!(bits(&pf.biome_bd), bits(&forced.biome_bd));
    }

    /// Protects `shore_depth`'s land clamp: a land cell exactly at sea level
    /// has a zero margin, and the field must still read it as land (the
    /// classification: below sea level only is water), never as the shoreline
    /// itself.
    #[test]
    fn a_cell_exactly_at_sea_level_is_drawn_land() {
        let (gw, gh) = (8usize, 8usize);
        // West half below sea level, one cell of the east half exactly at it.
        let mut field: Vec<f32> = (0..gw * gh).map(|i| if i % gw < 4 { 0.3 } else { 0.5 }).collect();
        field[3 * gw + 4] = SL as f32;
        let sl = field[3 * gw + 4] as f64;
        let rain = vec![0.5f32; gw * gh];
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, sl, false, Some(&rain));
        assert_eq!(wb.classification[3 * gw + 4], 0, "premise: at sea level is land");
        assert_eq!(wb.classification[3 * gw + 3], 1, "premise: beside the sea");
        let d = shore_depth(&field, &wb.classification, &wb.fill_level, &rain, gw, gh, sl, false, 4, 3);
        assert!(d < 0.0, "the sea-level cell reads {d}");
    }

    /// Protects `shore_margins`' neighbour rule: a land cell between the sea
    /// and an above-sea lake is the shore of the HIGHER water -- the lake
    /// (surface `0.8 - LAKE_DEPTH` = 0.796, rain-gated) -- not the sea
    /// (0.42). Literal answers from a hand-made three-cell strip.
    #[test]
    fn a_land_cell_is_the_shore_of_the_higher_water_beside_it() {
        let class = [1u8, 0, 2];
        let field = [0.3f32, 0.79, 0.7];
        let fill = [0.3f32, 0.79, 0.8];
        let rain = [0.5f32; 3];
        let (d, r) = shore_margins(&field, &class, &fill, &rain, 3, 1, SL, false, 1, 0);
        assert!((d - (0.796 - 0.79f32 as f64)).abs() < 1e-7, "depth margin {d}");
        assert!((r - (0.5f32 as f64 - 0.22)).abs() < 1e-7, "rain margin {r}");
        // The sea beside it alone: surface 0.42, no rain gate.
        let (d2, r2) = shore_margins(&field, &[1u8, 0, 0], &fill, &rain, 3, 1, SL, false, 1, 0);
        assert!((d2 - (0.42 - 0.79f32 as f64)).abs() < 1e-7, "sea depth margin {d2}");
        assert_eq!(r2, 1.0, "no rain gate beside the sea");
    }

    /// Protects `f16_bits`: literal IEEE half-float patterns, and the one
    /// property the shader depends on -- a tiny non-zero value keeps its sign.
    #[test]
    fn half_floats_are_exact_and_never_lose_a_sign() {
        assert_eq!(f16_bits(1.0), 0x3c00);
        assert_eq!(f16_bits(-2.0), 0xc000);
        assert_eq!(f16_bits(0.5), 0x3800);
        // 1.0007: 0.7168 of the last mantissa step, rounded up, not cut.
        assert_eq!(f16_bits(1.0007), 0x3c01);
        assert_eq!(f16_bits(65504.0), 0x7bff);
        assert_eq!(f16_bits(1.0e6), 0x7c00);
        assert_eq!(f16_bits(0.0), 0x0000);
        assert_eq!(f16_bits(-0.0), 0x8000);
        // 2^-14, the smallest normal; 2^-24, the smallest subnormal.
        assert_eq!(f16_bits(6.103515625e-5), 0x0400);
        assert_eq!(f16_bits(5.960464477539063e-8), 0x0001);
        // 1e-5 is subnormal in half precision: 1e-5 / 2^-24 = 167.77 -> 168.
        assert_eq!(f16_bits(1.0e-5), 168);
        assert_eq!(f16_bits(-1.0e-5), 0x8000 | 168);
        // Below the smallest subnormal: rounded away from zero, sign kept.
        assert_eq!(f16_bits(1.0e-12), 0x0001);
        assert_eq!(f16_bits(-1.0e-12), 0x8001);
    }
}

/// RV-3 (Ruling BD): the shaded valley field (`WorldGen::valley_shade_field`)
/// is handed to a `RenderCtx` as `field`, and must never decide water. These
/// protect the split: water bodies, the sea's smoothed bathymetry and its
/// shade come from the world's own height ([`RenderCtx::water_height`]), the
/// relief rasters from the shaded one.
#[cfg(test)]
mod valley_split_tests {
    use super::*;

    const SL: f64 = 0.42;
    const GW: usize = 32;
    const GH: usize = 24;

    /// A wet 0.8 plateau with sea along its west edge (x < 3) -- no lake --
    /// and the same world "shaded" with a pit 0.1 deep next to the coast
    /// (x 4..=6, y 8..=12): deep enough to pool as a lake, and inside
    /// `smooth_sea_h`'s reach of the sea cells at x = 2.
    fn worlds() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        let mut truth = vec![0.8f32; GW * GH];
        for y in 0..GH {
            for x in 0..3 {
                truth[y * GW + x] = 0.3;
            }
        }
        let mut shaded = truth.clone();
        for y in 8..=12 {
            for x in 4..=6 {
                shaded[y * GW + x] = 0.7;
            }
        }
        (truth, shaded, vec![0.9f32; GW * GH], vec![12.0f32; GW * GH])
    }

    fn ctx<'a>(f: &'a [f32], rain: &'a [f32], temp: &'a [f32]) -> RenderCtx<'a> {
        RenderCtx::with_appearance(f, temp, rain, None, GW, GH, SL, false, 50.0, 40.0, TerrainAppearance::default())
    }

    // Protects: a tile's water bodies are built from the world's height, not
    // the shaded one -- a depression the valley field opened is not a lake.
    #[test]
    fn a_depression_in_the_shaded_field_is_not_a_lake() {
        let (truth, shaded, rain, temp) = worlds();
        let pit = 10 * GW + 5;
        // Positive control: the fixture's pit IS a lake when classified from
        // the shaded field, so the split below has something to refuse.
        let from_shaded = TileFields::new(&ctx(&shaded, &rain, &temp), None);
        assert_eq!(from_shaded.lake_class[pit], 2, "premise: the shaded pit pools as a lake");
        let want = TileFields::new(&ctx(&truth, &rain, &temp), None);
        let got = TileFields::new(&ctx(&shaded, &rain, &temp).with_water_height(&truth), None);
        assert_eq!(got.lake_class.as_ref(), want.lake_class.as_ref(), "classification from the world's height");
        assert_eq!(bits(&got.lake_fill), bits(&want.lake_fill), "fill level from the world's height");
    }

    // Protects: the sea is coloured from the world's own ground, both on a
    // context built directly (`with_water_height`) and over a split
    // precompute (`build_split`), while the relief rasters come from the
    // shaded field.
    #[test]
    fn the_sea_and_the_relief_come_from_the_right_heights() {
        let (truth, shaded, rain, temp) = worlds();
        let a = TerrainAppearance::default();
        let t = ctx(&truth, &rain, &temp);
        // Positive control: the shaded field alone moves the sea's
        // bathymetry at the coast, so equality below is not vacuous.
        assert_ne!(bits(&ctx(&shaded, &rain, &temp).sea_h), bits(&t.sea_h), "premise: the pit reaches the sea's blur");
        let s = ctx(&shaded, &rain, &temp).with_water_height(&truth);
        assert_eq!(bits(&s.sea_h), bits(&t.sea_h));
        assert_eq!(bits(&s.sea_shade), bits(&t.sea_shade));
        let split = GridPrecompute::build_split(&shaded, &truth, &temp, &rain, None, GW, GH, SL, false, &a, Some(800.0), None);
        let water = GridPrecompute::build_forced(&truth, &temp, &rain, None, GW, GH, SL, false, &a, Some(800.0), None);
        let relief = GridPrecompute::build_forced(&shaded, &temp, &rain, None, GW, GH, SL, false, &a, Some(800.0), None);
        assert_eq!(bits(&split.sea_h), bits(&water.sea_h));
        assert_eq!(bits(&split.sea_shade), bits(&water.sea_shade));
        assert_eq!(bits(&split.ao), bits(&relief.ao), "AO is the shaded relief's");
        assert_ne!(bits(&split.ao), bits(&water.ao), "premise: the pit moves AO");
        // `build_forced` is `build_split` with one height, byte for byte.
        let same = GridPrecompute::build_split(&truth, &truth, &temp, &rain, None, GW, GH, SL, false, &a, Some(800.0), None);
        assert_eq!(bits(&same.ao), bits(&water.ao));
        assert_eq!(bits(&same.sea_h), bits(&water.sea_h));
        // And the attach-only builder leaves the precompute's sea alone.
        let c = RenderCtx::from_precomputed(&shaded, &temp, &rain, None, GW, GH, SL, false, 50.0, 40.0, a.clone(), &split).unwrap().with_precomputed_water_height(&truth);
        assert!(std::ptr::eq(c.water_height(), truth.as_slice()));
        assert_eq!(bits(&c.sea_h), bits(&water.sea_h));
    }

    // Protects: "no value" is not a plausible value -- a short world height is
    // refused, and the context keeps deciding water from its own field.
    #[test]
    fn a_short_water_height_is_refused() {
        let (truth, shaded, rain, temp) = worlds();
        let c = ctx(&shaded, &rain, &temp).with_water_height(&truth[..GW]);
        assert!(std::ptr::eq(c.water_height(), shaded.as_slice()));
        let c = ctx(&shaded, &rain, &temp).with_precomputed_water_height(&truth[..GW]);
        assert!(std::ptr::eq(c.water_height(), shaded.as_slice()));
    }

    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }
}
