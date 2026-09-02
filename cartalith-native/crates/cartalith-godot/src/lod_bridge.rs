//! Interactive per-tile deep-zoom synthesis — `LOD_TILING_INTEGRATION_SCOPE.md`
//! milestone M1: "a minimal interactive Z2... tile the deep-zoom case only,
//! not the whole map."
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own module doc argues for: `lib.rs` owns the thin
//! `Variant`<->Rust conversion and the `#[func]` surface (`lod_level_for_zoom`/
//! `lod_tiles_per_axis`/`lod_synthesize_tile`); this module owns the actual
//! synthesis — which tile, at what bounds, coloured how — with its own
//! `#[cfg(test)]` suite, exercised by plain `cargo test -p cartalith-godot`
//! with no Godot runtime.
//!
//! # Why this exists at all — and why it stops here
//!
//! §1 of the scope document computed real texture/memory/render-cost numbers
//! across this port's whole 512-8192 resolution range and found **no**
//! trigger for streaming/tiling the base raster itself (its "Z3"): the
//! single-raster approach stays cheap enough at every size this port
//! targets, and `viewport_host.gd`'s Z1 zoom/pan (already shipped, see its
//! own `_camera` doc comment) needs nothing from `cartalith-spatial` at all.
//!
//! What §2 found instead is a real, already-reported gap: the reference's
//! own owner complaint — *"There is still a certain pixilated quality to the
//! map when we zoom. The graphics should be finer than that"*
//! (`docs/HANDOFF.md`) — on exactly the failure mode `viewport_host.gd`'s
//! `_raster()` is set up to reproduce (`CanvasItem.TEXTURE_FILTER_NEAREST`,
//! no deep-zoom handling). §3 found the fix's actual math was already
//! ported and golden-tested — `cartalith_terrain::amplify`'s
//! `amplify_region` (bilinear upsample of the coarse height field plus
//! world-space fBm/ridged detail, tapered by relief and faded out
//! underwater) and `cartalith_terrain::tile_render::render_height_tile_rgba`
//! (the same hypsometric-tint × hillshade the Z4 export path already uses,
//! `region_export_tiles`'s own `visual: true` branch) — just never reachable
//! from an interactive, camera-driven caller. This module is that caller,
//! and nothing more: no atlas cache (Z5, milestone M3, deferred), no auto/
//! manual toggle (auto-on-zoom-threshold is what `viewport_host.gd` ships),
//! no chunk debug overlay (needs this to exist first).
//!
//! # Why not `TiledField`/`QuadTree` literally, despite the scope doc naming
//! them as "exactly the shape a Z2 compositor would want"
//!
//! Both were checked against this port's own real numbers, the same
//! discipline §1 applies to the base-raster question:
//!
//! - [`cartalith_spatial::TiledField`]'s constructor takes ownership of a
//!   `width * height` `Vec<T>`. Wrapping the *live* height field (up to
//!   8192² = 192 MiB — §1's own table, one-third of it for the height field
//!   alone at `f32`) would mean cloning that on every tile request just to
//!   reach `tile_bounds`, a method that reads only `width`/`height`/
//!   `tile_size` and never touches the data at all. [`tile_bounds`] below
//!   answers the same question from borrowed `gw`/`gh` via
//!   `cartalith_spatial::pyramid`, at the actual cost the query has — O(1),
//!   not O(field size).
//! - [`cartalith_spatial::QuadTree`]'s whole value is rejecting subtrees by
//!   their aggregate *value* range (min/max over cells) without visiting
//!   every cell — real for a predicate like "which regions contain water".
//!   "Which fixed-size tiles intersect this rect" has no such predicate to
//!   reject on; it is pure index arithmetic, and *building* a tree just to
//!   answer it would cost a real O(field size) scan (`QuadTree::build`'s own
//!   `region_min_max` per node) for a query whose real cost, done directly,
//!   is O(tiles on screen) — a handful, at any zoom level this milestone
//!   targets. Using it here would be strictly worse than not using it,
//!   which is the same "checked against real numbers, not asserted" standard
//!   §1 already applies to whether Z3 is needed at all.
//!
//! `cartalith_spatial::Region`/`FloatRegion` — the actually-generic pieces,
//! not the two data structures built for a different access pattern — are
//! used directly below. Nothing in `cartalith-spatial` is modified.
//!
//! # Where "which tiles are visible" is decided
//!
//! In GDScript (`viewport_host.gd`'s deep-zoom compositor), not here. The
//! camera-visible rect in grid-cell coordinates is exactly the kind of
//! screen<->local transform arithmetic `_zoom_at` already does in that file
//! (geometry, not a value the JS engine ever computed, so there is no
//! parity obligation on it); converting that rect into a small integer
//! range of `(col, row)` at the level [`level_for_zoom`] picks is the same
//! shape of computation. This module supplies [`tile_bounds`] and
//! [`tile_size_px`] so the *authoritative* footprint and pixel size for any
//! given `(z, col, row)` are computed once, in Rust, from the real `gw`/`gh` —
//! GDScript never has to duplicate the pyramid's own fractional-step rule to
//! stay in agreement with what a tile request actually returns.
//!
//! # What a tile actually contains, and why it is not a picture
//!
//! **This changed on 2026-08-23, fixing the owner's "a zoom action exposes the
//! underlying heightmap".** Until then [`synthesize_tile_rgba`] returned
//! `render_height_tile_rgba`'s hypsometric-tint × hillshade pixels and
//! `viewport_host.gd` laid them over the base raster opaquely — so crossing the
//! zoom threshold swapped the map's full cartographic plate (biome colour,
//! river network, AO, paper frame — `render.rs`'s `cell_color` pipeline behind
//! `build_color_texture`) for a bare green/gold/grey elevation ramp. Confirmed
//! live, not inferred: the same world, same camera, screenshotted with the LOD
//! layer shown and hidden.
//!
//! The reference never did that. `_lodBuildTileRGBA` (reference 11148) picks
//! the tile coloriser off the **view mode** —
//! `biome ? renderBiomeTileRGBA : renderHeightTileRGBA` — and `'biome'` is the
//! app's own default (`state.mode`, reference 2260). `renderHeightTileRGBA` is
//! what *Relief* mode shows; a tile under the default view is coloured by the
//! same `landColorCore` material the main map is. This port has only the
//! height-ramp half of that pair, and its map view is always the biome look, so
//! wiring the LOD compositor straight to `render_height_tile_rgba` guaranteed
//! the two rendering paths disagreed at every pixel.
//!
//! Porting `renderBiomeTileRGBA` itself is a milestone, not a bug fix: it needs
//! temperature, rainfall, lithology, flow and the whole `TerrainAppearance`
//! bag at sub-cell resolution, none of which reaches this module. What it does
//! *not* need to reinvent is the colour, because the base raster already has
//! it, and because `renderBiomeTileRGBA` samples its own colour inputs (T, M,
//! lithology, biome) bilinearly off the same coarse grid — only the
//! height-derived terms (slope, curvature, shade) run at tile resolution.
//!
//! So a tile now carries exactly the height-derived part the base raster
//! cannot have: the **relief-detail shade ratio**, the factor by which
//! `amplify_region`'s procedural sub-cell detail changes the hillshade at that
//! pixel, relative to the same tile with no detail added. `viewport_host.gd`'s
//! `lod_tile.gdshader` multiplies it into the base colour sampled at the same
//! ground position. Where the amplifier adds nothing — underwater, on plains,
//! wherever `taper` is zero — the ratio is exactly `1.0` and the map is
//! byte-unchanged. That is the property that makes the two paths agree by
//! construction rather than by coincidence.
//!
//! The encoding is [`SHADE_RATIO_MID`]/[`SHADE_RATIO_GAIN`] fixed point stored
//! in R, G and B (the same byte three times, so a caller that ignores the
//! shader still sees a plausible grey mask rather than a colour cast),
//! alpha `255`, because
//! `lib.rs`'s `lod_synthesize_tile` builds a `Format::RGBA8` `Image` and that
//! signature is not this module's to change.
//!
//! # Why a *pyramid* tile, since 2026-08-24 — the owner's "LOD zooming doesn't
//! seem to go that deep either"
//!
//! Until then this module addressed tiles on a fixed [`TILE_CELLS`]`= 64`
//! coarse-cell grid and grew the *output* resolution (256/512/1024 px) with a
//! `detail_level`, and it called `amplify_region` alone. Both halves of that
//! capped the reachable depth, and the second one is a failure mode the
//! reference names in `addZoomDetail`'s own header: *"amplifyRegion adds detail
//! at a FIXED coarse-space frequency, so the fbm runs out of octaves at high
//! zoom and the surface goes smooth ('details don't get more intricate')."*
//! Measured live at `ZOOM_MAX = 8` on a 512×384 world: 13.9 screen px per cell,
//! a 1024 px tile over 64 cells, and a picture with no sub-cell relief in it at
//! all — the base raster smoothly magnified, plus a shade ratio whose finest
//! octave is one cycle per *coarse cell* and therefore invisible.
//!
//! The reference's answer is the tile pyramid it already builds for the bake
//! (`cartalith_engine::bake::pyramid_tile`, `cartalith_spatial::pyramid`):
//! level `z` divides the map into `2^z × 2^z` tiles of one fixed pixel size, so
//! the *coarse-cell footprint* shrinks as you zoom while the pixel cost per
//! tile — and, because the tile count on screen stays roughly constant, per
//! *view* — does not. `add_zoom_detail` then adds `z − zBase` further octaves,
//! each 2× finer, which is where the extra intricacy actually comes from.
//!
//! So this module now synthesizes exactly `pyramid_tile`'s content, at exactly
//! the reference's own chunk addressing. Two consequences worth stating:
//!
//! - It is the same content a *baked* chunk holds, by construction rather than
//!   by coincidence — which is the precondition any future atlas read at draw
//!   time needs, and the reason the `cartalith-engine` import this module's own
//!   header once argued against is now the right dependency to have. (The atlas
//!   read itself is still not wired, and deliberately: see the note below.)
//! - The `cartalith-engine` half is already golden-tested against the reference
//!   (`bake.rs`, sixteen goldens including six FNV-1a-64 hashes of
//!   `addZoomDetail` output), so this file adds no new numerical logic of its
//!   own — only the shade-ratio reduction it already owned.
//!
//! ## Why the baked atlas is *not* read here
//!
//! Checked rather than assumed, and it is not the depth fix it looks like:
//! a baked chunk's PNG is `region_export::tile_png_bytes`, the **Relief**
//! coloriser — the very hypsometric ramp the 2026-08-23 fix above removed from
//! this path because it disagrees with the biome map at every pixel. Drawing it
//! would reintroduce the owner's "a zoom action exposes the underlying
//! heightmap" verbatim. The reusable half is the chunk's *height* (`rg16`,
//! `cartalith_io::decode_chunk`), which has no `#[func]` yet. And the depths
//! that matter are past baking anyway: a depth-7 pyramid is 21 845 tiles, so
//! the atlas can only ever serve the shallow levels, where live synthesis is
//! already a few milliseconds. Recorded as its own milestone, not folded in.

use cartalith_engine::bake::pyramid_tile;
use cartalith_spatial::pyramid::{
    pyramid_dims, pyramid_level_for_zoom, pyramid_tile_bounds, ChunkId,
};
use cartalith_spatial::{tile_dims, FloatRegion, Region};
use cartalith_terrain::amplify::{refine_tile, AmplifyOpts};
use cartalith_terrain::tile_render::{shade_tile, u8_clamped};

/// Output resolution (pixels) for one interactive pyramid tile — the
/// reference's `_lodTile`, at a quarter of its 1024 px default.
///
/// The reference composites its whole LOD view into one canvas on a frame
/// budget; this port draws one `TextureRect` per tile and synthesises them
/// synchronously across a per-call budget and a per-frame backlog, so the unit
/// of stall is one tile. Measured on a 512×384 world: 251 ms for a 1024 px
/// tile, 16.5 ms for a 256 px one — the same cost per output pixel either way,
/// but a sixteenth of the hitch, and a view's worth of tiles is the same ~1 Mpx
/// at any depth. [`z_base`] compensates the octave schedule for the difference
/// so a tile here and a baked 1024 px chunk over the same ground get the same
/// detail, not two more octaves of it.
pub const TILE_PX: usize = 256;

/// The reference's own `_lodTile` default (reference line 10656) — not the size
/// used here, only the size its `zBase = 2` and `lodMaxLevel = 8` are quoted
/// against. See [`TILE_PX`], [`z_base`] and [`MAX_LEVEL`].
pub const REFERENCE_TILE_PX: usize = 1024;

/// The deepest pyramid level [`level_for_zoom`] will return.
///
/// The reference's `state.lodMaxLevel` is `8` (line 2271) at `_lodTile = 1024`;
/// four-times-smaller tiles need two more levels to reach the same ground
/// resolution, so `8 + log2(1024/256) = 10` — which is also exactly where its
/// own `lodLevels` selector tops out (line 1245). Past this `add_zoom_detail`
/// has nothing left to add either: its octave count is `min(6, z − zBase)`, so
/// the finest octave is fixed from level `zBase + 6` on.
pub const MAX_LEVEL: i32 = 10;

/// The reference's own `TileVisual::default()` values
/// (`cartalith_engine::region_export`), mirrored here as plain constants
/// rather than pulled in as a dependency on that struct: a Z2 screen tile
/// and a Z4 export tile over the same ground should shade under the same
/// sun, and this is the smallest way to keep that true without adding an
/// `cartalith-engine` import to a module that otherwise has none.
const SUN_AZ_DEG: f64 = 315.0;
const EXAG: f64 = 3.4;

/// Fixed point for the relief-detail shade ratio a tile carries (see this
/// module's own "What a tile actually contains" section): the encoded byte is
/// `SHADE_RATIO_MID + (ratio - 1) * `[`SHADE_RATIO_GAIN`], so a ratio of
/// exactly `1.0` — "the detail changes nothing here" — is byte `128` and
/// round-trips with no error at all rather than landing one least-significant
/// bit off and tinting an untouched map.
///
/// **Centred and gained rather than a plain `ratio * 128`**, which was the
/// first cut and measured wrong: the ratio lives within a few percent of `1.0`
/// (it is the *difference* one octave of sub-cell detail makes to a hillshade,
/// not the hillshade), so a scale that spans `[0, 2]` resolved a whole
/// synthetic tile into three distinct byte values. At `SHADE_RATIO_GAIN` the
/// quantisation step is `1/256` of the multiplier — finer than the 8-bit
/// colour it multiplies, so nothing bands — over a representable window of
/// `[0.5, 1.5]`. The theoretical extreme is `1.0 / 0.4 = 2.5×` (a fully
/// backlit plain surface relit by detail), which clips; a 50% shading swing
/// from one detail octave does not occur on real terrain, and clipping it is
/// better than spending precision nothing uses.
pub const SHADE_RATIO_MID: f64 = 128.0;
/// See [`SHADE_RATIO_MID`]. Kept in step with `lod_tile.gdshader`, which
/// hardcodes both numbers on the decode side.
pub const SHADE_RATIO_GAIN: f64 = 256.0;

/// `opts.zBase` for a [`TILE_PX`]-sized tile.
///
/// The reference's `zBase = 2` (`AmplifyOpts::default`) is quoted against its
/// own `_lodTile = 1024`: at level `z` a 1024 px tile resolves
/// `1024·2^z/(gw−1)` px per coarse cell, and `min(6, z − 2)` extra octaves is
/// as many as that can carry without aliasing. A [`TILE_PX`] tile reaches the
/// same ground resolution two levels deeper, so using `2` here would add two
/// octaves *past* what the tile can resolve — visible as noise, not as detail.
/// Shifting `zBase` by the same `log2` keeps the schedule identical at equal
/// ground resolution, which is also what makes a tile from here and a baked
/// 1024 px chunk agree.
pub fn z_base() -> i32 {
    AmplifyOpts::default().z_base + (REFERENCE_TILE_PX / TILE_PX).ilog2() as i32
}

/// `pyramidLevelForZoom` (reference line 10600, already ported) in this
/// caller's own terms: the pyramid level whose tile pixels best match the
/// screen at `px_per_cell` screen pixels per coarse cell.
///
/// The reference asks the same question against its render canvas —
/// `pyramidLevelForZoom(span, _lodRenderW(), _lodTile, state.lodMaxLevel||8)`
/// (line 11009), where `_lodRenderW() · span` is the map's total width in
/// render pixels at that zoom. This port draws straight to the screen, so the
/// same quantity is `gw · px_per_cell`, which is what `base_w · scale` is
/// below.
pub fn level_for_zoom(px_per_cell: f64, gw: usize) -> i32 {
    pyramid_level_for_zoom(px_per_cell, gw as f64, TILE_PX as f64, Some(MAX_LEVEL))
}

/// Tiles per axis at pyramid level `z` — `2^z`, [`pyramid_dims`] re-exported
/// so the GDScript compositor reads the count rather than recomputing it.
pub fn tiles_per_axis(z: i32) -> u32 {
    pyramid_dims(z.clamp(0, MAX_LEVEL)).cols
}

/// The `(z, col, row)` chunk's footprint in coarse **sample** coordinates —
/// `pyramidTileBounds` exactly, i.e. `[0, gw−1] × [0, gh−1]` split `2^z` ways
/// per axis, so adjacent tiles *share* their edge sample and agree on it
/// bit-for-bit.
///
/// `None` before any world (`gw < 2 || gh < 2`, which would make the step
/// zero or negative) or for an index outside the level's own `2^z × 2^z` grid.
pub fn tile_bounds(gw: usize, gh: usize, z: i32, col: i32, row: i32) -> Option<FloatRegion> {
    if gw < 2 || gh < 2 || !(0..=MAX_LEVEL).contains(&z) || col < 0 || row < 0 {
        return None;
    }
    let n = pyramid_dims(z).cols;
    if col as u32 >= n || row as u32 >= n {
        return None;
    }
    Some(pyramid_tile_bounds(gw, gh, z, col as u32, row as u32))
}

/// The tile's output size in pixels — `tile_dims` over the level's own grid,
/// the same call [`pyramid_tile`] makes internally. Square for a square map,
/// aspect-matched otherwise.
pub fn tile_size_px(gw: usize, gh: usize, z: i32) -> (usize, usize) {
    let n = pyramid_dims(z.clamp(0, MAX_LEVEL)).cols as usize;
    let sel = Region { x: 0, y: 0, w: gw.saturating_sub(1), h: gh.saturating_sub(1) };
    let d = tile_dims(&sel, n, n, TILE_PX);
    (d.w, d.h)
}

/// Synthesizes one deep-zoom tile's **relief-detail shade ratio** — not a
/// picture. See this module's own "What a tile actually contains, and why it
/// is not a picture" section for the full argument; the short version is that
/// the reference colours an LOD tile with the *view mode's* coloriser
/// (`_lodBuildTileRGBA`, reference 11148) and this port only ever had the
/// Relief-mode one, so a tile now carries the one thing the base raster
/// genuinely cannot — the sub-cell relief — and takes its colour from that
/// raster instead of inventing a second, disagreeing palette.
///
/// The detailed half is [`pyramid_tile`] verbatim — `refine_tile`'s bilinear
/// upsample plus its fixed coarse-frequency detail, then `add_zoom_detail`'s
/// `min(6, z − `[`z_base`]`)` progressively finer octaves — so a tile drawn
/// here and a chunk baked into the atlas over the same ground are the same
/// numbers. The plain half is the same `refine_tile` call with `detail_amp =
/// 0` and no zoom detail: the pure bilinear upsample the base raster's own
/// shading already reflects. `shade_tile` reduces each to the multiplier
/// `render_height_tile_rgba` would have applied, and their ratio is what the
/// detail *adds* — exactly what is missing from the base raster and nothing
/// else.
///
/// `seed`/`sea` are read from the caller's own world state (`WorldGen::seed`/
/// `sea_level`), the same convention `region_export_tiles` already uses —
/// "an export must match the world it was drawn over, not a caller-guessed
/// one" applies just as much to an interactive tile.
///
/// Returns `(rgba_bytes, out_w, out_h)` — `rgba_bytes.len() == out_w * out_h
/// * 4`, ready to hand `Image::create_from_data` directly. `None` for
/// anything [`tile_bounds`] itself rejects, or when `field` is shorter than
/// `gw * gh` — the same precondition `refine_tile` would otherwise panic
/// on, checked here instead so a caller error surfaces as "no tile" rather
/// than taking the whole Godot process down with it
/// (`cartalith-rust-conventions`: no panic crosses the gdext boundary).
pub fn synthesize_tile_rgba(
    field: &[f32],
    gw: usize,
    gh: usize,
    z: i32,
    col: i32,
    row: i32,
    seed: i32,
    sea: f64,
) -> Option<(Vec<u8>, usize, usize)> {
    synthesize_tile_rgba_with_z_base(field, gw, gh, z, col, row, seed, sea, z_base())
}

/// [`synthesize_tile_rgba`] with `opts.zBase` supplied rather than taken from
/// [`z_base`] — the seam through which the tests can switch `add_zoom_detail`
/// off (`z_base == z` makes it a documented no-op) and measure what the
/// progressive octaves are actually worth at each depth. Not part of the
/// `#[func]` surface: the shell has no business choosing this.
#[allow(clippy::too_many_arguments)]
fn synthesize_tile_rgba_with_z_base(
    field: &[f32],
    gw: usize,
    gh: usize,
    z: i32,
    col: i32,
    row: i32,
    seed: i32,
    sea: f64,
    zb: i32,
) -> Option<(Vec<u8>, usize, usize)> {
    if field.len() < gw.checked_mul(gh)? {
        return None;
    }
    tile_bounds(gw, gh, z, col, row)?;
    let opts = AmplifyOpts { seed, sea, z_base: zb, ..AmplifyOpts::default() };
    let tile = pyramid_tile(field, gw, gh, ChunkId::new(z as u32, col as u32, row as u32), TILE_PX, &opts);
    // `pyramid_tile` sizes itself with the same `tile_dims` call; taking the
    // dimensions from `tile_size_px` and checking rather than reading them
    // off the result is what lets a caller (`viewport_host.gd`'s tile rect,
    // and the tests) ask for the size *without* synthesising a tile first,
    // with no second formula that could drift.
    let (out_w, out_h) = tile_size_px(gw, gh, z);
    if (tile.w, tile.h) != (out_w, out_h) {
        return None;
    }
    let detailed = tile.data;

    // The same level, same sub-region, same sampler -- only the detail term
    // switched off, and `add_zoom_detail` not applied at all. Reusing
    // `refine_tile` (which is what `pyramid_tile` calls first) rather than
    // reimplementing its bilinear upsample is what guarantees the two differ
    // *only* by the detail, which is the entire meaning of the ratio below.
    let n = pyramid_dims(z).cols as usize;
    let region = Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }.to_float();
    let plain_opts = AmplifyOpts { detail_amp: 0.0, ..opts };
    let plain = refine_tile(
        field, gw, gh, &region, n, n, col as usize, row as usize, out_w, out_h, &plain_opts,
    );

    // `shade_tile` differences *adjacent pixels* with a fixed exaggeration, so
    // the same ground slope shades `1/px_per_cell` as hard once a tile spreads
    // one coarse cell over many pixels -- which is every level past the first.
    // Measured before compensating: on a dome fixture the mask went from 34%
    // of pixels carrying any shading at level 4 to 3% at level 7, i.e. deep
    // zoom converged on "no relief at all" no matter how many octaves
    // `add_zoom_detail` put into the height. Scaling the exaggeration by the
    // tile's own pixels per cell undoes exactly that geometric factor and
    // makes the ratio scale-invariant instead: the same terrain reads with the
    // same relief at every depth.
    //
    // Free to choose, and chosen rather than inherited: the shade *ratio* is
    // this port's own construct (the reference colours an LOD tile outright,
    // it never computes a ratio), so `EXAG` here is a parameter of this
    // module's encoding, not a reference constant under a parity obligation.
    // It stays `TileVisual::default()`'s 3.4 at one pixel per cell, which is
    // the resolution a Z4 export tile is written at.
    let exag = EXAG * (out_w as f64 / tile_bounds(gw, gh, z, col, row)?.w).max(1.0);
    let sd = shade_tile(&detailed, out_w, out_h, sea, SUN_AZ_DEG, exag);
    let sp = shade_tile(&plain, out_w, out_h, sea, SUN_AZ_DEG, exag);

    let mut rgba = vec![255u8; out_w * out_h * 4];
    for i in 0..out_w * out_h {
        // `shade_tile` never returns zero for a real height (its bands floor
        // at 0.4 and 0.75), but a NaN tile -- `amplify_region`'s documented
        // `out_w == 1` division by zero -- makes both terms NaN, and NaN/NaN
        // is NaN, which `u8_clamped` maps to 0. Guard to a neutral 1.0 so a
        // degenerate tile leaves the map alone instead of blacking it out.
        let ratio = if sp[i] > 0.0 { sd[i] / sp[i] } else { 1.0 };
        let ratio = if ratio.is_nan() { 1.0 } else { ratio };
        let b = u8_clamped(SHADE_RATIO_MID + (ratio - 1.0) * SHADE_RATIO_GAIN);
        rgba[i * 4] = b;
        rgba[i * 4 + 1] = b;
        rgba[i * 4 + 2] = b;
    }
    Some((rgba, out_w, out_h))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Same shape amplify.rs's own `synthetic_field` test helper uses (pure
    /// arithmetic, a quantised term so distinct tiles are actually distinct)
    /// — not reused directly since that helper is private to `amplify.rs`'s
    /// own test module.
    fn synthetic_field(gw: usize, gh: usize) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let q = ((x * 7 + y * 13) % 11) as f64 / 10.0;
                let v = 0.25 + 0.5 * (x as f64 / gw as f64) + 0.08 * (q - 0.5);
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }


    // -- addressing -------------------------------------------------------

    #[test]
    fn tile_bounds_none_before_any_world() {
        assert_eq!(tile_bounds(0, 0, 0, 0, 0), None);
        assert_eq!(tile_bounds(1, 1, 0, 0, 0), None);
    }

    #[test]
    fn tile_bounds_none_outside_the_level_grid() {
        // Level 2 is a 4x4 grid, so index 4 does not exist at all -- unlike
        // the old fixed-cell grid, whose last tile was merely clipped.
        assert_eq!(tile_bounds(256, 256, 2, 4, 0), None);
        assert_eq!(tile_bounds(256, 256, 2, 0, 4), None);
        assert_eq!(tile_bounds(256, 256, -1, 0, 0), None);
        assert_eq!(tile_bounds(256, 256, MAX_LEVEL + 1, 0, 0), None);
    }

    #[test]
    fn tile_bounds_tile_the_whole_sample_range_and_share_their_edges() {
        // The pyramid convention `pyramid_tile_bounds` states: `[0, gw-1]`
        // split `2^z` ways, adjacent tiles sharing the edge sample. Both
        // halves matter downstream -- the shared sample is what makes the
        // seam exact, and the `gw-1` (not `gw`) is what the half-texel rect
        // in `viewport_host.gd`'s `_lod_tile_rect` is written against.
        let a = tile_bounds(257, 257, 2, 0, 0).unwrap();
        let b = tile_bounds(257, 257, 2, 1, 0).unwrap();
        let last = tile_bounds(257, 257, 2, 3, 3).unwrap();
        assert_eq!((a.x, a.w), (0.0, 64.0));
        assert_eq!(b.x, a.x + a.w, "tile 1 must start on tile 0's last sample");
        assert_eq!(last.x + last.w, 256.0, "the last tile must end at gw-1");
    }

    #[test]
    fn tiles_per_axis_doubles_per_level_and_clamps() {
        assert_eq!(tiles_per_axis(0), 1);
        assert_eq!(tiles_per_axis(3), 8);
        assert_eq!(tiles_per_axis(MAX_LEVEL), 1 << MAX_LEVEL);
        assert_eq!(tiles_per_axis(MAX_LEVEL + 5), 1 << MAX_LEVEL);
        assert_eq!(tiles_per_axis(-3), 1);
    }

    #[test]
    fn level_for_zoom_tracks_the_screen_and_stops_at_max() {
        // The property the depth fix rests on: one more level per doubling of
        // screen px per cell, so the tile's own px/cell keeps up with the
        // camera instead of saturating at a fixed tier.
        let gw = 512;
        let l1 = level_for_zoom(4.0, gw);
        let l2 = level_for_zoom(8.0, gw);
        let l3 = level_for_zoom(16.0, gw);
        assert_eq!(l2, l1 + 1);
        assert_eq!(l3, l2 + 1);
        // A cell filling the screen still resolves to a real level, capped.
        assert_eq!(level_for_zoom(1.0e6, gw), MAX_LEVEL);
        // ...and a degenerate zoom does not fall off the bottom.
        assert_eq!(level_for_zoom(0.0, gw), 0);
        assert_eq!(level_for_zoom(-4.0, gw), 0);
    }

    #[test]
    fn the_deepest_level_out_resolves_the_old_fixed_tier_ceiling() {
        // The measured cap this change exists to lift: the old model's best
        // was a 1024 px tile over 64 coarse cells, i.e. 16 screen px per cell,
        // and `viewport_host.gd`'s old `ZOOM_MAX = 8` was set to match it.
        // State the new ceiling as a number rather than a claim.
        let (gw, gh) = (512usize, 512usize);
        let (w, _) = tile_size_px(gw, gh, MAX_LEVEL);
        let cells = tile_bounds(gw, gh, MAX_LEVEL, 0, 0).unwrap().w;
        let px_per_cell = w as f64 / cells;
        assert!(px_per_cell > 16.0 * 8.0, "only {px_per_cell} px per cell at the deepest level");
    }

    #[test]
    fn z_base_matches_the_reference_schedule_at_equal_ground_resolution() {
        // A `TILE_PX` tile at level z resolves the same ground as a 1024 px
        // tile at level z - log2(1024/TILE_PX), so its octave count must be
        // the same: `z - z_base()` here == `z2 - 2` there.
        let shift = (REFERENCE_TILE_PX / TILE_PX).ilog2() as i32;
        assert_eq!(z_base(), AmplifyOpts::default().z_base + shift);
        for z_ref in 2..=8 {
            assert_eq!(z_ref + shift - z_base(), z_ref - AmplifyOpts::default().z_base);
        }
    }

    // -- synthesis --------------------------------------------------------

    #[test]
    fn synthesize_tile_rgba_none_for_a_too_short_field() {
        // Guards `refine_tile`'s own panic precondition rather than letting it
        // panic across what would be the gdext boundary.
        let short = vec![0.5f32; 10];
        assert_eq!(synthesize_tile_rgba(&short, 64, 64, 0, 0, 0, 1234, 0.42), None);
    }

    #[test]
    fn synthesize_tile_rgba_none_for_an_out_of_range_tile() {
        let field = synthetic_field(128, 128);
        assert_eq!(synthesize_tile_rgba(&field, 128, 128, 2, 10, 10, 1234, 0.42), None);
    }

    #[test]
    fn synthesize_tile_rgba_produces_the_right_number_of_opaque_pixels() {
        let field = synthetic_field(256, 256);
        let (rgba, w, h) = synthesize_tile_rgba(&field, 256, 256, 2, 0, 0, 1234, 0.42).unwrap();
        assert_eq!((w, h), tile_size_px(256, 256, 2));
        assert_eq!((w, h), (TILE_PX, TILE_PX), "a square map gives a square tile");
        assert_eq!(rgba.len(), w * h * 4);
        assert!(rgba.chunks(4).all(|p| p[3] == 255), "every pixel must be opaque");
    }

    #[test]
    fn a_non_square_map_gives_an_aspect_matched_tile() {
        // `tile_dims` keeps the tile's aspect, so a 2:1 map gives 2:1 tiles --
        // which is what `_lod_tile_rect`'s half-texel maths reads back off the
        // real texture rather than assuming square.
        let field = synthetic_field(257, 129);
        let (rgba, w, h) = synthesize_tile_rgba(&field, 257, 129, 2, 1, 1, 1234, 0.42).unwrap();
        assert_eq!((w, h), (TILE_PX, TILE_PX / 2));
        assert_eq!(rgba.len(), w * h * 4);
    }

    #[test]
    fn synthesize_tile_rgba_is_not_a_flat_mask() {
        // A silently-constant tile passes every structural check above, so
        // say it explicitly -- same reasoning tile_render.rs's own
        // `render_is_not_flat` test states. A ratio mask sits close to 1.0 by
        // design (that is the point: it perturbs the base map, it does not
        // replace it), so the bar is "really varies", not "spans the byte
        // range".
        let field = synthetic_field(256, 256);
        // The eastern half: `synthetic_field` ramps west-to-east from 0.25, so
        // the western tiles are entirely below `sea` and correctly come back
        // flat-neutral (deep water gets no detail). Asserting variation there
        // would pin the wrong thing.
        let (rgba, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 3, 2, 1234, 0.42).unwrap();
        let distinct: std::collections::HashSet<u8> = rgba.chunks(4).map(|p| p[0]).collect();
        assert!(distinct.len() > 8, "only {} distinct shade levels", distinct.len());
        // Grey, not tinted: R == G == B everywhere, alpha opaque.
        assert!(rgba.chunks(4).all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
    }

    #[test]
    fn a_tile_with_no_added_detail_is_exactly_neutral() {
        // The property the whole compositor rests on: where the amplifier adds
        // nothing, the ratio is 1.0 and the shader leaves the base map's own
        // pixels alone. Deep water tapers the detail term to zero
        // (`amplify_region`'s `underwater`/`taper`) and `add_zoom_detail` skips
        // anything below `sea` outright, so a field well below sea level must
        // come back as the encoded identity, byte for byte -- not "close to"
        // it, which is what centring the fixed point on `SHADE_RATIO_MID` buys.
        let field = vec![0.10f32; 128 * 128];
        let (rgba, _, _) = synthesize_tile_rgba(&field, 128, 128, 2, 1, 1, 1234, 0.42).unwrap();
        let neutral = SHADE_RATIO_MID as u8;
        assert!(
            rgba.chunks(4).all(|p| p[0] == neutral && p[1] == neutral && p[2] == neutral),
            "a detail-free tile must encode exactly {neutral}"
        );
    }

    #[test]
    fn a_detailed_land_tile_actually_perturbs_the_base() {
        // The other half of the property above: where detail *is* added, the
        // mask must leave neutral.
        let field = synthetic_field(256, 256);
        let (rgba, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 3, 2, 1234, 0.42).unwrap();
        let neutral = SHADE_RATIO_MID as u8;
        let moved = rgba.chunks(4).filter(|p| p[0] != neutral).count();
        assert!(moved > rgba.len() / 4 / 2, "only {moved} pixels carry any detail shading");
    }

    /// A field with real coarse *relief*, unlike `synthetic_field`'s
    /// near-linear west-to-east ramp -- the same dome-plus-ridge shape
    /// `amplify.rs`'s own golden fixture uses, and for the same reason the
    /// depth test below needs it: `add_zoom_detail` multiplies every octave by
    /// `min(1, hypot(coarse gradient) * 8)`, which on a 0.5-over-1024-cells
    /// ramp is about `0.004` -- the octaves are there but attenuated 250x, so
    /// a ramp would measure the taper rather than the detail.
    fn relief_field(gw: usize, gh: usize) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let (cx, cy) = (gw as f64 * 0.42, gh as f64 * 0.55);
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let (dx, dy) = (x as f64 - cx, y as f64 - cy);
                let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
                let q = ((x * 7 + y * 13) % 11) as f64 / 10.0;
                v += 0.05 * (q - 0.5);
                v += 0.10
                    * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    /// The mask's **fine** structure: mean absolute difference between
    /// horizontally adjacent pixels, in encoded byte units.
    ///
    /// Deliberately not `max - min`, which the depth test below was first
    /// written against and which measured the wrong thing: a mask's extremes
    /// are set by a handful of outlier pixels and barely move with depth
    /// (83 -> 78 across four levels, octaves or not), while what "the detail
    /// gets more intricate" means is precisely that neighbouring pixels stop
    /// agreeing. That is this.
    fn fine_detail(rgba: &[u8], w: usize, h: usize) -> f64 {
        let px = |x: usize, y: usize| rgba[(y * w + x) * 4] as f64;
        let mut sum = 0.0;
        for y in 0..h {
            for x in 1..w {
                sum += (px(x, y) - px(x - 1, y)).abs();
            }
        }
        sum / ((w - 1) * h) as f64
    }

    #[test]
    fn deeper_levels_carry_strictly_finer_detail() {
        // THE regression test for the owner report this change answers: "LOD
        // zooming does not seem to go that deep either". Before it, this module
        // called `amplify_region` alone, whose detail sits at a fixed
        // coarse-space frequency -- so zooming past the first tier resolved the
        // *same* relief more smoothly and nothing new ever appeared. With
        // `add_zoom_detail` in the path, each level past `z_base()` adds an
        // octave, and the mask over the same ground must therefore keep gaining
        // structure. Compared over the SAME ground at every level -- the tile
        // whose north-west corner sits at (0.75, 0.25) of the map, which
        // `synthetic_field`'s west-to-east ramp puts well above `sea` (tile
        // (0,0) is deep water there, where the detail is correctly zero and
        // the spread is flat 0 at every level).
        //
        // Two separate claims, because they come from two separate halves of
        // the change and either could regress alone:
        //
        // 1. The mask keeps *gaining* fine structure with depth. That is what
        //    the scale-normalised `exag` above buys, and it is the half the
        //    owner actually sees: before it the numbers ran the other way
        //    (0.30 -> 0.20 -> 0.096 -> 0.031 on this very fixture), i.e. deep
        //    zoom converged on a flat mask over a smooth blur.
        // 2. `add_zoom_detail`'s octaves are what carry that at depth. Stated
        //    against a no-octave baseline synthesised the same way, and as a
        //    ratio rather than a difference, since the baseline moves too.
        let (gw, gh) = (512usize, 512usize);
        let field = relief_field(gw, gh);
        let mut seen = Vec::new();
        for z in [z_base(), z_base() + 1, z_base() + 2, z_base() + 3] {
            // The same ground at every level: the tile whose north-west corner
            // sits at (5/16, 7/16) of the map, on the dome's own flank. Not
            // the map's own quarter point, which this fixture puts on flat
            // near-sea ground where there is correctly no detail to find and
            // the test would pin the taper instead.
            let n = 1 << z;
            let (col, row) = (5 * n / 16, 7 * n / 16);
            let (with, tw, th) =
                synthesize_tile_rgba(&field, gw, gh, z, col, row, 1234, 0.42).unwrap();
            // `zb == z` makes `add_zoom_detail`'s `extra` non-positive, i.e.
            // exactly the pre-2026-08-24 `amplify_region`-only content.
            let (without, _, _) =
                synthesize_tile_rgba_with_z_base(&field, gw, gh, z, col, row, 1234, 0.42, z)
                    .unwrap();
            seen.push((z, fine_detail(&with, tw, th), fine_detail(&without, tw, th)));
        }

        let fine: Vec<f64> = seen.iter().map(|&(_, w, _)| w).collect();
        assert!(
            fine.windows(2).all(|p| p[1] > p[0]),
            "the mask stopped gaining detail with depth: {seen:?}"
        );

        let (z0, w0, o0) = seen[0];
        assert_eq!(w0, o0, "at z_base ({z0}) the octaves must be a no-op: {seen:?}");
        let ratios: Vec<f64> = seen.iter().map(|&(_, w, o)| w / f64::max(1e-9, o)).collect();
        assert!(
            ratios[1..].windows(2).all(|p| p[1] > p[0]),
            "the octaves stopped paying off with depth: {seen:?} ratios {ratios:?}"
        );
        // One octave over `amplify_region`'s own detail is nearly a wash (the
        // measured ratio at `z_base + 1` is 0.99); by three it is not, and
        // that is the depth the camera now reaches.
        assert!(
            *ratios.last().unwrap() > 1.05,
            "the deepest level barely differs from no octaves at all: {ratios:?}"
        );
    }

    #[test]
    fn at_z_base_the_zoom_octaves_are_a_no_op() {
        // `add_zoom_detail`'s own documented property, restated at this
        // caller's `z_base()` so a future change to `TILE_PX` that forgets to
        // move `z_base()` with it fails here rather than silently over- or
        // under-detailing every tile.
        let field = synthetic_field(512, 512);
        let z = z_base();
        let opts = AmplifyOpts { seed: 1234, sea: 0.42, z_base: z, ..AmplifyOpts::default() };
        let n = pyramid_dims(z).cols as usize;
        let region = Region { x: 0, y: 0, w: 511, h: 511 }.to_float();
        let (w, h) = tile_size_px(512, 512, z);
        let plain = refine_tile(&field, 512, 512, &region, n, n, 0, 0, w, h, &opts);
        let via_pyramid =
            pyramid_tile(&field, 512, 512, ChunkId::new(z as u32, 0, 0), TILE_PX, &opts).data;
        assert_eq!(plain, via_pyramid, "at z == z_base the extra octaves must change nothing");
    }

    #[test]
    fn different_tiles_of_the_same_world_synthesize_different_content() {
        let field = synthetic_field(256, 256);
        let (a, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 2, 2, 1234, 0.42).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 3, 2, 1234, 0.42).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_for_the_same_inputs() {
        // Same standard `PARITY_TESTING.md`-adjacent expectation every
        // synthesis path in this crate holds to: no hidden randomness.
        let field = synthetic_field(200, 200);
        let (a, _, _) = synthesize_tile_rgba(&field, 200, 200, 3, 1, 1, 42, 0.42).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&field, 200, 200, 3, 1, 1, 42, 0.42).unwrap();
        assert_eq!(a, b);
    }

}
