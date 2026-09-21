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
//! **That paragraph described this module from 2026-08-23 to 2026-09-21, and
//! LOD-D2 ended it.** `renderBiomeTileRGBA` is ported (LOD-D1,
//! `render::render_biome_tile_rgba`, golden-verified byte-identical against
//! the frozen reference), so the milestone that was "not a bug fix" is built
//! and a tile is a **picture** again — this time the same coloriser the map
//! itself runs, not the Relief ramp that disagreed with it at every pixel.
//!
//! What a tile carries now: RGBA8, alpha `255`, the full biome look evaluated
//! at *tile* resolution. The height-derived terms (slope, macro and meso
//! shade, relative height, TWI, the crest field, the coast SDF) are computed
//! from the tile's own amplified heightmap, and the colour inputs (T, M, flow,
//! AO, lithology, the local-contrast band) are sampled bilinearly off the same
//! coarse grid the map reads — so a deeper level genuinely shows more, which
//! the shade ratio structurally could not (a scalar multiplier cannot add a
//! material boundary, a river band or a crest stroke that the base raster does
//! not already have).
//!
//! **The two inputs a tile cannot derive** are therefore `&RenderCtx` and
//! `&TileFields`, and [`synthesize_tile_rgba`] takes them rather than the
//! loose `(field, gw, gh, seed, sea)` it used to: those five were enough to
//! shade a tile and are nowhere near enough to colour one. Building them costs
//! **201.5 ms + 278.2 ms** at 2048x1311 (release, this machine, medians over
//! five — `the_tile_context_costs_an_order_of_magnitude_more_than_the_tile_it_serves`
//! below prints them), against 5.98 ms for a tile on all cores, so they
//! are built once per world-and-appearance and cached by the caller;
//! `lib.rs`'s `WorldGen::lod_ctx` is the only such caller today and owns the
//! cache key.
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
use cartalith_terrain::amplify::AmplifyOpts;
use crate::render::{self, RenderCtx, TileBounds, TileFields};

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

// `SUN_AZ_DEG`, `EXAG`, `SHADE_RATIO_MID` and `SHADE_RATIO_GAIN` lived here
// until LOD-D2 (2026-09-21) and are **gone, not deprecated**: they were the
// shade-ratio encoding's own constants, and a tile is a picture now. The sun
// and the exaggeration come from `TerrainAppearance` (the same values the map
// shades under, which is stronger than the `TileVisual::default()` mirror they
// were), and there is no fixed point to keep in step with
// `lod_tile.gdshader`, which no longer decodes one. `tile_producer_id`'s `v2`
// is what stops last week's shade-ratio tiles being read back as colour.

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

/// Synthesizes one deep-zoom tile as **the map's own biome colour**, at tile
/// resolution — `renderBiomeTileRGBA` over `pyramid_tile`'s amplified height.
///
/// This is `LOD_DETAIL_SCOPE.md` LOD-D2's whole scope line: *"`lod_bridge::
/// synthesize_tile_rgba` returns `render_biome_tile_rgba(pyramid_tile(...))`"*.
///
/// # The two halves, and why each is where it is
///
/// **The height** is [`pyramid_tile`] verbatim — `refine_tile`'s bilinear
/// upsample plus its fixed coarse-frequency detail, then `add_zoom_detail`'s
/// `min(6, z - `[`z_base`]`)` progressively finer octaves — so a tile drawn
/// here and a chunk baked into the atlas over the same ground are the same
/// numbers. Unchanged by this milestone.
///
/// **The colour** is [`render::render_biome_tile_rgba`], which is the
/// reference's own function (LOD-D1: golden, worst delta `0`). Everything it
/// needs beyond the tile's own height comes from `ctx` and `tf`, which the
/// caller owns and caches — see this module's header for the measurement that
/// forces that split, and `render::GridPrecompute` for the mechanism.
///
/// `bounds` is [`tile_bounds`] unchanged, and the two conventions line up
/// exactly rather than approximately: `pyramid_tile_bounds` returns the tile's
/// span in **sample** coordinates (`x = col * step`, `w = step`, `step =
/// (gw-1)/2^z`), and `amplify_region` maps output pixel `ox` to `rx + ox/(W-1)
/// * step` — so pixel `0` sits at `bounds.x` and pixel `W-1` at `bounds.x +
/// bounds.w`, which is `TileBounds`' documented "first pixel centre to last
/// pixel centre" to the letter. `adjacent_tiles_share_their_edge_column`
/// asserts it rather than leaving it to this comment.
///
/// `seed` is the caller's own world seed (`WorldGen::seed`), the same
/// convention `region_export_tiles` uses — "an export must match the world it
/// was drawn over, not a caller-guessed one" applies just as much to an
/// interactive tile. The **sea level and the height field come from `ctx`**,
/// not from separate arguments: they used to be passed in beside a field the
/// caller also passed in, which made it possible to amplify one world and
/// colour another. One source now.
///
/// Returns `(rgba_bytes, out_w, out_h)` — `rgba_bytes.len() == out_w * out_h *
/// 4`, ready to hand `Image::create_from_data` directly. `None` for anything
/// [`tile_bounds`] itself rejects, for a `ctx` whose field is shorter than
/// `gw * gh`, and for a `tf` built for a different grid (which
/// `render_biome_tile_rgba` reports as an empty `Vec`, checked here so a
/// caller error surfaces as "no tile" rather than as a mis-sized `Image`) —
/// `cartalith-rust-conventions`: no panic crosses the gdext boundary.
pub fn synthesize_tile_rgba(
    ctx: &RenderCtx,
    tf: &TileFields,
    z: i32,
    col: i32,
    row: i32,
    seed: i32,
) -> Option<(Vec<u8>, usize, usize)> {
    synthesize_tile_rgba_with_z_base(ctx, tf, z, col, row, seed, z_base())
}

/// [`synthesize_tile_rgba`] with `opts.zBase` supplied rather than taken from
/// [`z_base`] — the seam through which the tests can switch `add_zoom_detail`
/// off (`z_base == z` makes it a documented no-op) and measure what the
/// progressive octaves are actually worth at each depth. Not part of the
/// `#[func]` surface: the shell has no business choosing this.
#[allow(clippy::too_many_arguments)]
fn synthesize_tile_rgba_with_z_base(
    ctx: &RenderCtx,
    tf: &TileFields,
    z: i32,
    col: i32,
    row: i32,
    seed: i32,
    zb: i32,
) -> Option<(Vec<u8>, usize, usize)> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    if ctx.field.len() < gw.checked_mul(gh)? {
        return None;
    }
    let bounds = tile_bounds(gw, gh, z, col, row)?;
    let opts = AmplifyOpts { seed, sea: ctx.sea_level, z_base: zb, ..AmplifyOpts::default() };
    let tile = pyramid_tile(ctx.field, gw, gh, ChunkId::new(z as u32, col as u32, row as u32), TILE_PX, &opts);
    // `pyramid_tile` sizes itself with the same `tile_dims` call; taking the
    // dimensions from `tile_size_px` and checking rather than reading them
    // off the result is what lets a caller (`viewport_host.gd`'s tile rect,
    // and the tests) ask for the size *without* synthesising a tile first,
    // with no second formula that could drift.
    let (out_w, out_h) = tile_size_px(gw, gh, z);
    if (tile.w, tile.h) != (out_w, out_h) {
        return None;
    }
    let rgba = render::render_biome_tile_rgba(
        ctx,
        &tile.data,
        out_w,
        out_h,
        TileBounds { x: bounds.x, y: bounds.y, w: bounds.w, h: bounds.h },
        tf,
    );
    if rgba.len() != out_w * out_h * 4 {
        return None;
    }
    Some((rgba, out_w, out_h))
}

// ---------------------------------------------------------------------------
// Storing a pyramid in the project archive — owner ruling 28, 2026-09-06
//
// The five functions below are `pub` and, as of 2026-09-06, have **no caller
// outside this file's own tests**, so a plain `cargo build -p cartalith-godot`
// reports each of them "is never used". That is accurate and is left visible
// on purpose: they are the producer half of ruling 28, and the `#[func]` plus
// `project_bridge.rs` save path that consumes them is the next step, not part
// of this pass.
//
// **This note is here so a dead-code sweep reads it before deleting them** —
// the same protection ruling 22 required for `--good`/`--accH`, and for the
// same reason: without a recorded reason, a sweep removing them is right by
// its own rule. An `#[allow(dead_code)]` was the alternative and was refused,
// because the warning is a true signal that the slot is built and unwired,
// and silencing it would make "wired" and "unwired" look the same.
// ---------------------------------------------------------------------------

/// One tile's storable form: **RGB**, three bytes per pixel, without the
/// constant alpha [`synthesize_tile_rgba`] pads it out to.
///
/// **This dropped from four bytes to one and back to three, and the middle
/// number is why the producer id has to change.** Until 2026-09-21 a tile was
/// a shade-ratio mask -- one byte written three times, so `tile_mask` took the
/// R channel and `mask_to_rgba` fanned it back out, losslessly, at a quarter
/// of the bytes. A tile is a picture now (LOD-D2), R/G/B differ, and taking
/// one channel would store a greyscale map of a colour one. So the ratio the
/// archive pays moves with it: the *raw* cost is **3x** the old mask
/// ([`pyramid_mask_bytes`] is multiplied by exactly that), and the deflated
/// cost is **not** predictable from the old measurement, because what
/// compressed so well before was the three-identical-channels redundancy that
/// no longer exists. The old figures (levels 0..=6 over three real 2048x1311
/// worlds: masks 21.9 / 23.6 / 27.8 MiB against RGB PNGs at 65.2 / 69.9 /
/// 81.1 MiB) are kept here as the **prior** they are, not carried forward as a
/// claim about what RGB tiles will deflate to. Whoever wires ruling 28's save
/// path re-measures it; `measure_a_stored_pyramid` in this file's tests is the
/// harness that produced them and is still the way to.
///
/// Alpha is dropped rather than stored because
/// [`render::render_biome_tile_rgba`] documents it as *"alpha always `255`"*
/// -- and the round trip below asserts that on real synthesized tiles instead
/// of trusting the sentence.
pub fn tile_mask(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]).collect()
}

/// [`tile_mask`]'s inverse -- a stored tile back in the `RGBA8` shape
/// `Image::create_from_data` takes.
pub fn mask_to_rgba(mask: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(mask.len() / 3 * 4);
    for px in mask.chunks_exact(3) {
        out.extend_from_slice(&[px[0], px[1], px[2], 255]);
    }
    out
}

/// What this build's tile synthesizer calls itself, for
/// `cartalith_io::project::LodTiles::producer`.
///
/// The archive's own key covers the *world* -- the heightmap, the grid, the
/// seed and the sea level. It cannot cover the producer's **constants** or the
/// **look**, which is what this is for: a build that moves `TILE_PX` or the
/// octave schedule, or a user who changes the appearance, would otherwise
/// decode last week's tiles over this week's map.
///
/// # `v2`, and why a stored `v1` must be refused rather than decoded
///
/// `LOD_DETAIL_SCOPE.md` LOD-D2: *"`tile_producer_id` becomes
/// `cartalith-lod/v2`. Stored v1 masks are refused by producer id, never
/// decoded under the new shader."* A v1 tile is a **shade-ratio mask** -- one
/// byte per pixel, `128` meaning "the detail changes nothing here" -- and
/// `lod_tile.gdshader` no longer multiplies anything, so a v1 pyramid read as
/// RGB would draw a flat mid-grey sheet over the map and read as a rendering
/// bug rather than as a stale cache. The leading version is **not** derived
/// and has to be bumped by hand when the arithmetic moves; this is that bump.
///
/// # The appearance is part of the id now, and that is new
///
/// A shade ratio was a property of the *height* alone, so a look change could
/// not invalidate a v1 tile. A coloured tile is `land_color`'s output, so
/// every ramp, strength, light and NPR flag on `TerrainAppearance` is an input
/// (`MISTAKES.md`: *"derive the list from the definition -- every argument of
/// the function you are guarding"*). `fp` below is an FNV-1a-64 of the
/// appearance's own **serde serialization**, which is derived from the struct
/// definition rather than from a hand-list -- so a field added to
/// `TerrainAppearance` is covered the day it is added, with nothing to
/// remember here. `the_producer_id_moves_with_every_appearance_field`
/// exercises that rather than asserting it.
///
/// **What it does not cover, stated rather than implied:** the river ink, the
/// lake mask, the colour space and the paint/pack overrides also reach a tile,
/// through `TileFields` and the `RenderCtx` builders. Those are *world* state,
/// which is the archive key's half of this contract -- except the colour
/// space, which is neither, and is the one honest gap here. It is a display
/// setting, it changes a stored tile's bytes, and nothing in this id or in the
/// archive key moves when it changes. Recorded rather than assumed away: the
/// storage path is unwired (see the section header above), so it costs nothing
/// today and it is whoever wires it who has to close it.
///
/// The interpolation of the `const`s is not test-covered and cannot be: no
/// test can vary a `const`, so `px={TILE_PX}` and the literal `px=256` are the
/// same string to any assertion. The test beside this one covers *membership*
/// -- a constant dropped from the id -- and says so.
///
/// `AmplifyOpts::default()`'s `seed` and `sea` are deliberately absent:
/// [`synthesize_tile_rgba`] overrides both from the live world, so the
/// defaults are never used and the live values are the archive key's job.
pub fn tile_producer_id(appearance: &render::TerrainAppearance) -> String {
    let o = AmplifyOpts::default();
    format!(
        "cartalith-lod/v2;px={TILE_PX};zb={};fp={:016x};freq={};amp={};ridged={};k={}",
        z_base(),
        appearance_fingerprint(appearance),
        o.detail_freq,
        o.detail_amp,
        o.ridged,
        o.zoom_detail_k,
    )
}

/// FNV-1a-64 of `serde_json` of the appearance -- the whole struct, field by
/// field, without this file naming any of them.
///
/// Serialization rather than `Debug` deliberately: `TerrainAppearance` carries
/// `#[serde(default)]` and is already the format a saved look is written in
/// (`WorldGen::set_appearance_preset`), so this fingerprint changes exactly
/// when a saved look would, and it cannot be moved by someone tidying a
/// `Debug` impl. A serialization failure returns `0` -- a fingerprint that
/// matches nothing rather than a panic across the gdext boundary.
pub(crate) fn appearance_fingerprint(a: &render::TerrainAppearance) -> u64 {
    let json = match serde_json::to_string(a) {
        Ok(j) => j,
        Err(_) => return 0,
    };
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in json.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The **raw** bytes a stored pyramid of levels `0..=z_max` occupies, before
/// the archive's deflate — `sum(4^z) * tile_w * tile_h * 3`.
///
/// **The `* 3` is LOD-D2's** (*"`pyramid_mask_bytes` is recomputed for RGB
/// tiles, since the size Rulings 28/29 show at save grows about 3x raw"*): a
/// stored tile was one byte per pixel while it was a shade ratio and is three
/// now that it is a picture. See [`tile_mask`] for why the *deflated* figure
/// does not simply scale with it.
///
/// Exact and instant: it synthesizes nothing, so a save dialog can show the
/// cost before the user commits to paying it (ruling 28: *"off, with the
/// size shown at save time"*). `None` for a `z_max` [`tile_bounds`] itself
/// would reject.
///
/// **The stored figure is smaller and is content-dependent, so it is not
/// returned from here.** Measured on three real 2048×1311 worlds
/// (`measure_a_stored_pyramid`): levels 0-5 deflate to **8.1-9.4%** of this
/// number and levels 0-6 to **10.0-12.7%** — the band widens with depth
/// because deeper tiles carry more sub-cell detail and compress less. A
/// single ratio baked in here would be a model presented as a measurement.
pub fn pyramid_mask_bytes(gw: usize, gh: usize, z_max: i32) -> Option<u64> {
    if gw < 2 || gh < 2 || !(0..=MAX_LEVEL).contains(&z_max) {
        return None;
    }
    let (w, h) = tile_size_px(gw, gh, 0);
    Some(cartalith_spatial::pyramid::pyramid_tile_count(z_max) * (w * h * 3) as u64)
}

/// Synthesizes every tile of levels `0..=z_max` as storable masks — the
/// **producer** ruling 28 found this port did not have, because
/// [`synthesize_tile_rgba`] makes tiles on demand and the atlas cache is
/// deferred at `LOD_TILING_INTEGRATION_SCOPE.md` M3, so nothing held a tile
/// set to write.
///
/// Returns `(tile_w, tile_h, tiles)` in exactly the shape
/// `cartalith_io::project::LodTiles` takes. `None` on the same preconditions
/// [`synthesize_tile_rgba`] rejects, plus a `z_max` outside `0..=`[`MAX_LEVEL`].
///
/// One size covers the whole pyramid because [`tile_size_px`] is
/// level-independent: a level changes a tile's *footprint*, not its pixel
/// count. This loop does **not** re-check that per tile — a guard here proved
/// unreachable under a mutation run, and the format layer already enforces it
/// where it matters: `write_project` refuses any tile that is not exactly
/// `tile_w * tile_h` bytes, naming the tile
/// (`a_tile_of_the_wrong_size_is_refused_at_write_time`).
///
/// # This is O(4^z_max) and the caller has to mean it
///
/// Levels `0..=6` over a real 2048×1311 world took **14.3 s** — one sample,
/// release, this machine, `measure_a_stored_pyramid`'s own print. It is an
/// order of magnitude, not a benchmark: each level is 4× the tiles of the one
/// before, so the shape (seconds at 6, minutes past 7) is the useful part and
/// no depth makes it cheap. That is the other half of why the slot is off by
/// default.
///
/// No progress callback and no cancellation: this is deliberately the plain
/// loop, and the caller that wires it to a save dialog is the right place for
/// both — the same split `bake_all_tiles` already draws.
///
/// # Invalidation is two problems, and only one of them lives in the archive
///
/// **Across a save**, the archive owns it: `cartalith_io::project` computes a
/// `source_key` from the heightmap it writes, recomputes it from the heightmap
/// it reads, and drops the tiles on a mismatch. `StageGraph` structurally
/// cannot do that half — every stage in a fresh `StageGraph::new` starts at
/// version 0, so two different worlds' graphs are indistinguishable and none
/// of it survives a process.
///
/// **Within a session**, `cartalith_spatial::StageGraph` is the mechanism and
/// no second one should be invented: `WorldGen.stages` already marks
/// `PipelineStage::Height` on every path that moves terrain — `sculpt_commit`,
/// `carve_fjords`, `erode`, `undo`, `redo` — so a stored or in-memory tile set
/// is stale exactly when `any_stale`/the height stage's version says it is.
/// Whatever wires this into the save should drop the set on that signal rather
/// than on a heuristic. **A note for that lane, from reading the shell rather
/// than from this crate:** `viewport_host.gd` clears its live tiles in
/// `refresh()` (a new `generate()`/`load_save()`) and on `_set_lod_active(false)`,
/// and the sculpt-commit path deliberately does not call `refresh()` — it
/// assigns `map_view.texture` directly (`world_workspace.gd`, and its own
/// comment says why) — so a commit at a fixed zoom is a case worth checking
/// before assuming the in-session half is already covered.
pub fn synthesize_pyramid_masks(
    ctx: &RenderCtx,
    tf: &TileFields,
    z_max: i32,
    seed: i32,
) -> Option<(usize, usize, std::collections::BTreeMap<ChunkId, Vec<u8>>)> {
    if !(0..=MAX_LEVEL).contains(&z_max) {
        return None;
    }
    let (gw, gh) = (ctx.gw, ctx.gh);
    if gw < 2 || gh < 2 {
        return None;
    }
    let (tile_w, tile_h) = tile_size_px(gw, gh, 0);
    let mut tiles = std::collections::BTreeMap::new();
    for z in 0..=z_max {
        let n = tiles_per_axis(z) as i32;
        for col in 0..n {
            for row in 0..n {
                let (rgba, _, _) = synthesize_tile_rgba(ctx, tf, z, col, row, seed)?;
                tiles.insert(ChunkId::new(z as u32, col as u32, row as u32), tile_mask(&rgba));
            }
        }
    }
    Some((tile_w, tile_h, tiles))
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

    /// One world's worth of the inputs a coloured tile needs, owned together
    /// so a `RenderCtx` can borrow them all.
    ///
    /// Since LOD-D2 a tile is `render_biome_tile_rgba`'s output, so every test
    /// below needs a temperature and a moisture field as well as a height one
    /// — the height alone could shade a tile and cannot colour it. They are
    /// synthesised here rather than captured: what these tests assert is this
    /// module's own contract (sizes, addressing, determinism, the storage
    /// round trip, detail against depth), and the *colour* is
    /// `tests/golden_parity_tile_biome.rs`'s job, against the real reference.
    ///
    /// The climate is a north-south temperature ramp crossed with an
    /// east-west moisture one, so a tile of any size spans several biomes —
    /// a single-biome fixture would make `land_color`'s material path
    /// constant and several assertions below vacuous.
    struct TestWorld {
        field: Vec<f32>,
        temp: Vec<f32>,
        rain: Vec<f32>,
        gw: usize,
        gh: usize,
        pre: render::GridPrecompute,
    }

    const TEST_SEA: f64 = 0.42;

    impl TestWorld {
        fn new(field: Vec<f32>, gw: usize, gh: usize) -> Self {
            let mut temp = vec![0f32; gw * gh];
            let mut rain = vec![0f32; gw * gh];
            for y in 0..gh {
                for x in 0..gw {
                    let (u, v) = (x as f64 / gw as f64, y as f64 / gh as f64);
                    temp[y * gw + x] = (24.0 - 38.0 * v) as f32;
                    rain[y * gw + x] = (0.28 + 0.60 * (u * 3.0 + 0.7).sin().abs()) as f32;
                }
            }
            let a = Self::appearance();
            let pre = render::GridPrecompute::build(&field, &temp, &rain, Some(&field), gw, gh, TEST_SEA, false, &a, Some(800.0));
            TestWorld { field, temp, rain, gw, gh, pre }
        }

        /// The look `WorldGen` opens on, not `default()` — the tiles these
        /// tests measure should be the tiles the app draws.
        fn appearance() -> render::TerrainAppearance {
            render::TerrainAppearance::default().with_look(render::LOOK_VIBRANT)
        }

        fn ctx(&self) -> RenderCtx<'_> {
            render::RenderCtx::from_precomputed(
                &self.field, &self.temp, &self.rain, Some(&self.field), self.gw, self.gh, TEST_SEA, false, 55.0, 5.0, Self::appearance(), &self.pre,
            )
            .expect("the precompute is for this grid")
        }

        /// No grid raster, so the local-contrast band is off — `TileFields`'
        /// own documented `None` case. Deliberate: rendering a full grid
        /// raster per test would dominate their runtime, and no assertion
        /// below is about local contrast.
        fn fields(&self, ctx: &RenderCtx) -> TileFields<'static> {
            TileFields::new(ctx, None)
        }
    }

    /// Rec.709 luma of one pixel, for the tests that measure *structure*
    /// rather than colour.
    fn luma(px: &[u8]) -> f64 {
        0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
    }

    #[test]
    fn synthesize_tile_rgba_none_for_a_too_short_field() {
        // Guards `refine_tile`'s own panic precondition rather than letting it
        // panic across what would be the gdext boundary. Built by pointing a
        // valid context at a short field, which is the shape the real failure
        // takes: `WorldGen` hands `synthesize_tile_rgba` whatever its live
        // `WorldSource` carries, and a truncated one must not be indexed.
        let w = TestWorld::new(synthetic_field(64, 64), 64, 64);
        let short = vec![0.5f32; 10];
        let ctx = render::RenderCtx::from_precomputed(&short, &w.temp, &w.rain, None, 64, 64, TEST_SEA, false, 55.0, 5.0, TestWorld::appearance(), &w.pre).unwrap();
        let tf = w.fields(&w.ctx());
        assert_eq!(synthesize_tile_rgba(&ctx, &tf, 0, 0, 0, 1234), None);
    }

    #[test]
    fn synthesize_tile_rgba_none_for_an_out_of_range_tile() {
        let w = TestWorld::new(synthetic_field(128, 128), 128, 128);
        let (ctx, tf) = (w.ctx(), w.fields(&w.ctx()));
        assert_eq!(synthesize_tile_rgba(&ctx, &tf, 2, 10, 10, 1234), None);
    }

    #[test]
    fn a_tile_fields_built_for_another_grid_is_refused_rather_than_indexed() {
        // `render_biome_tile_rgba` returns an empty `Vec` for a mismatched
        // `TileFields`; this module turns that into `None` rather than letting
        // a mis-sized buffer reach `Image::create_from_data`. Reached from the
        // LOD bridge on every zoom notch, so it is the gdext-boundary rule.
        let w = TestWorld::new(synthetic_field(128, 128), 128, 128);
        let other = TestWorld::new(synthetic_field(64, 64), 64, 64);
        let other_ctx = other.ctx();
        assert_eq!(synthesize_tile_rgba(&w.ctx(), &other.fields(&other_ctx), 1, 0, 0, 1234), None);
    }

    #[test]
    fn synthesize_tile_rgba_produces_the_right_number_of_opaque_pixels() {
        let tw = TestWorld::new(synthetic_field(256, 256), 256, 256);
        let ctx = tw.ctx();
        let (rgba, w, h) = synthesize_tile_rgba(&ctx, &tw.fields(&ctx), 2, 0, 0, 1234).unwrap();
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
        let tw = TestWorld::new(synthetic_field(257, 129), 257, 129);
        let ctx = tw.ctx();
        let (rgba, w, h) = synthesize_tile_rgba(&ctx, &tw.fields(&ctx), 2, 1, 1, 1234).unwrap();
        assert_eq!((w, h), (TILE_PX, TILE_PX / 2));
        assert_eq!(rgba.len(), w * h * 4);
    }

    #[test]
    fn synthesize_tile_rgba_is_not_a_flat_picture() {
        // A silently-constant tile passes every structural check above, so say
        // it explicitly -- the same reasoning `tile_render.rs`'s own
        // `render_is_not_flat` states, and the same bar LOD-D1's capture
        // script refuses a fixture under (*"fewer than 16 distinct colours"*).
        //
        // Two claims, not one, because a shade ratio could satisfy the first
        // and never the second: the tile VARIES, and it is in COLOUR. Before
        // LOD-D2 every pixel here was grey by construction (one byte written
        // into R, G and B), so `r != b` somewhere is the assertion that goes
        // red if this module ever regresses to a mask.
        let tw = TestWorld::new(synthetic_field(256, 256), 256, 256);
        let ctx = tw.ctx();
        // The eastern half: `synthetic_field` ramps west-to-east from 0.25, so
        // the western tiles are entirely below sea level and correctly come
        // back as flat water. Asserting variety there would pin the wrong
        // thing.
        let (rgba, _, _) = synthesize_tile_rgba(&ctx, &tw.fields(&ctx), 2, 3, 2, 1234).unwrap();
        let distinct: std::collections::HashSet<[u8; 3]> = rgba.chunks(4).map(|p| [p[0], p[1], p[2]]).collect();
        assert!(distinct.len() > 16, "only {} distinct colours", distinct.len());
        assert!(rgba.chunks(4).any(|p| p[0] != p[2]), "every pixel is grey -- this is a mask, not a picture");
        assert!(rgba.chunks(4).all(|p| p[3] == 255), "every pixel must be opaque");
    }

    #[test]
    fn where_the_amplifier_adds_nothing_the_octaves_change_nothing() {
        // The property the compositor rested on when a tile was a ratio was
        // "a detail-free tile encodes exactly the identity byte". There is no
        // identity colour to compare against now, so the same claim is made
        // the way it survives the change: deep water is where
        // `amplify_region`'s `taper` is zero and `add_zoom_detail` skips
        // outright, so the tile with the octaves and the tile without them
        // must be **byte-identical** -- not close.
        //
        // The positive control is the test below: the same comparison on land
        // must differ, or this one would pass on a build where the octaves
        // were removed entirely.
        let tw = TestWorld::new(vec![0.10f32; 128 * 128], 128, 128);
        let ctx = tw.ctx();
        let tf = tw.fields(&ctx);
        let z = z_base() + 3;
        let (with, _, _) = synthesize_tile_rgba(&ctx, &tf, z, 1, 1, 1234).unwrap();
        let (without, _, _) = synthesize_tile_rgba_with_z_base(&ctx, &tf, z, 1, 1, 1234, z).unwrap();
        assert_eq!(with, without, "a wholly underwater tile must not move when the zoom octaves are switched off");
    }

    #[test]
    fn a_detailed_land_tile_actually_moves_under_the_octaves() {
        // The other half, and the positive control for the test above.
        let tw = TestWorld::new(relief_field(256, 256), 256, 256);
        let ctx = tw.ctx();
        let tf = tw.fields(&ctx);
        let z = z_base() + 3;
        let n = 1 << z;
        let (col, row) = (5 * n / 16, 7 * n / 16);
        let (with, w, h) = synthesize_tile_rgba(&ctx, &tf, z, col, row, 1234).unwrap();
        let (without, _, _) = synthesize_tile_rgba_with_z_base(&ctx, &tf, z, col, row, 1234, z).unwrap();
        let moved = with.chunks(4).zip(without.chunks(4)).filter(|(a, b)| a[..3] != b[..3]).count();
        assert!(moved > w * h / 10, "only {moved} of {} pixels moved when the octaves were switched on", w * h);
    }
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
        // Luma of the pixel rather than its red byte: a tile is a colour
        // picture since LOD-D2, and red alone would measure one channel of a
        // material change as if it were relief.
        let px = |x: usize, y: usize| luma(&rgba[(y * w + x) * 4..]);
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
        // THE regression test for the owner report this subsystem answers:
        // "LOD zooming does not seem to go that deep either". Before
        // 2026-08-24 this module called `amplify_region` alone, whose detail
        // sits at a fixed coarse-space frequency -- so zooming past the first
        // tier resolved the *same* relief more smoothly and nothing new ever
        // appeared. With `add_zoom_detail` in the path, each level past
        // `z_base()` adds an octave.
        //
        // # Rewritten for LOD-D2, and the statistic had to change with it
        //
        // `LOD_DETAIL_SCOPE.md` LOD-D2: *"`deeper_levels_carry_strictly_finer_detail`
        // is rewritten against colour tiles and stays red if `add_zoom_detail`
        // is removed."* The old test measured mean |difference| between
        // horizontally adjacent bytes of a shade-ratio mask and asserted it
        // rose with depth. That assertion is **geometrically wrong for a
        // picture** and would have been kept only by accident: a deeper tile
        // covers less ground per pixel, so two adjacent pixels are physically
        // closer together and *must* differ less. Measured on this very
        // fixture, per adjacent pixel pair: 7.29, 4.63, 3.55, 3.46 L-units
        // across four levels. The old path hid that behind a scale-normalised
        // exaggeration of its own invention (`EXAG * px_per_cell`), which a
        // ratio was free to choose and a colour is not -- a tile shades under
        // the same sun and the same `exag` as the map, or it does not match it.
        //
        // So the same question is asked over the same GROUND instead:
        // adjacent-pixel difference times pixels per coarse cell, i.e. how
        // much the picture varies across one cell of terrain. That is what
        // "a deeper level shows more" means, and it is what the owner sees.
        //
        // Three claims, because they come from three separate halves and any
        // one could regress alone:
        //
        // 1. **Per unit ground, a deeper level carries strictly more.**
        //    Measured 58.3 -> 74.1 -> 113.5 -> 221.4 at the four levels below.
        // 2. **At `z_base` the octaves are a no-op**, byte for byte -- the
        //    property `add_zoom_detail` documents, restated at this caller's
        //    own `z_base()` so a `TILE_PX` change that forgets to move it
        //    fails here.
        // 3. **The octaves are what carry it at depth**, stated against a
        //    no-octave baseline synthesised the same way and as a ratio, since
        //    the baseline moves too. Measured 1.000, 1.048, 1.258, 1.787 --
        //    and every one of them is exactly 1.000 if `add_zoom_detail` is
        //    removed, which is the red this test owes the scope.
        let (gw, gh) = (512usize, 512usize);
        let world = TestWorld::new(relief_field(gw, gh), gw, gh);
        let ctx = world.ctx();
        let tf = world.fields(&ctx);
        let mut seen = Vec::new();
        for z in [z_base(), z_base() + 1, z_base() + 2, z_base() + 3] {
            // The same ground at every level: the tile whose north-west corner
            // sits at (5/16, 7/16) of the map, on the dome's own flank. Not
            // the map's own quarter point, which this fixture puts on flat
            // near-sea ground where there is correctly no detail to find and
            // the test would pin the taper instead.
            let n = 1 << z;
            let (col, row) = (5 * n / 16, 7 * n / 16);
            let (with, tw, th) = synthesize_tile_rgba(&ctx, &tf, z, col, row, 1234).unwrap();
            // `zb == z` makes `add_zoom_detail`'s `extra` non-positive, i.e.
            // exactly the pre-2026-08-24 `amplify_region`-only content.
            let (without, _, _) = synthesize_tile_rgba_with_z_base(&ctx, &tf, z, col, row, 1234, z).unwrap();
            // Pixels per coarse cell, from the tile's own bounds rather than
            // from `2^z` -- the two agree, and reading it off the addressing
            // is what makes this survive a `TILE_PX` change.
            let per_cell = tw as f64 / tile_bounds(gw, gh, z, col, row).unwrap().w;
            seen.push((z, fine_detail(&with, tw, th) * per_cell, fine_detail(&without, tw, th) * per_cell));
        }

        let fine: Vec<f64> = seen.iter().map(|&(_, w, _)| w).collect();
        assert!(
            fine.windows(2).all(|p| p[1] > p[0]),
            "a deeper level stopped carrying more detail per unit ground: {seen:?}"
        );

        let (z0, w0, o0) = seen[0];
        assert_eq!(w0, o0, "at z_base ({z0}) the octaves must be a no-op: {seen:?}");
        let ratios: Vec<f64> = seen.iter().map(|&(_, w, o)| w / f64::max(1e-9, o)).collect();
        assert!(
            ratios[1..].windows(2).all(|p| p[1] > p[0]),
            "the octaves stopped paying off with depth: {seen:?} ratios {ratios:?}"
        );
        // One octave over `amplify_region`'s own detail is nearly a wash
        // (measured 1.048 at `z_base + 1`); by three it is not, and that is
        // the depth the camera now reaches.
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
        let plain = cartalith_terrain::amplify::refine_tile(&field, 512, 512, &region, n, n, 0, 0, w, h, &opts);
        let via_pyramid =
            pyramid_tile(&field, 512, 512, ChunkId::new(z as u32, 0, 0), TILE_PX, &opts).data;
        assert_eq!(plain, via_pyramid, "at z == z_base the extra octaves must change nothing");
    }

    #[test]
    fn different_tiles_of_the_same_world_synthesize_different_content() {
        let tw = TestWorld::new(synthetic_field(256, 256), 256, 256);
        let ctx = tw.ctx();
        let tf = tw.fields(&ctx);
        let (a, _, _) = synthesize_tile_rgba(&ctx, &tf, 2, 2, 2, 1234).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&ctx, &tf, 2, 3, 2, 1234).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_for_the_same_inputs() {
        // Same standard `PARITY_TESTING.md`-adjacent expectation every
        // synthesis path in this crate holds to: no hidden randomness.
        let tw = TestWorld::new(synthetic_field(200, 200), 200, 200);
        let ctx = tw.ctx();
        let tf = tw.fields(&ctx);
        let (a, _, _) = synthesize_tile_rgba(&ctx, &tf, 3, 1, 1, 42).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&ctx, &tf, 3, 1, 1, 42).unwrap();
        assert_eq!(a, b);
    }

    // -- storing a pyramid: the round trip, the id, and the real cost -----

    #[test]
    fn a_stored_tile_is_the_synthesized_tile_exactly() {
        // The claim that makes a 3-byte-per-pixel slot legitimate: the fourth
        // byte `synthesize_tile_rgba` writes is always `255`. Asserted over
        // real tiles, not a hand-built buffer -- and over several, since a
        // uniformly flat tile would pass on a constant buffer.
        let tw = TestWorld::new(relief_field(256, 256), 256, 256);
        let ctx = tw.ctx();
        let tf = tw.fields(&ctx);
        let mut distinct = std::collections::BTreeSet::new();
        for (z, col, row) in [(0, 0, 0), (2, 1, 2), (4, 5, 9)] {
            let (rgba, w, h) = synthesize_tile_rgba(&ctx, &tf, z, col, row, 7).unwrap();
            let stored = tile_mask(&rgba);
            assert_eq!(stored.len(), w * h * 3, "a stored tile is three bytes per pixel since LOD-D2");
            assert_eq!(
                mask_to_rgba(&stored),
                rgba,
                "the stored form must be lossless at ({z},{col},{row})"
            );
            distinct.extend(stored.chunks_exact(3).map(|p| [p[0], p[1], p[2]]));
        }
        assert!(
            distinct.len() > 16,
            "the fixture's tiles carry only {} distinct colours -- a constant tile would pass this vacuously",
            distinct.len()
        );
    }

    /// **What this establishes and what it cannot.** It establishes that
    /// every constant a stored tile depends on is *named* in the id, which is
    /// the failure that actually happens — a constant added and left out. It
    /// does **not** establish that each is interpolated rather than
    /// transcribed: replacing `px={TILE_PX}` with the literal `px=256`
    /// produces a byte-identical string and survives this test (measured
    /// under a mutation run, 2026-09-06), because no test can vary a `const`.
    /// That half is verified by reading the one format string in
    /// [`tile_producer_id`], not by an assertion, and is stated here rather
    /// than claimed as coverage it does not have.
    ///
    /// The appearance half is different and **is** covered, by
    /// `the_producer_id_moves_with_every_appearance_field` below: a
    /// serialization fingerprint is derived from the struct, so a test can
    /// vary it.
    #[test]
    fn the_producer_id_names_every_constant_a_stored_tile_depends_on() {
        let a = TestWorld::appearance();
        let id = tile_producer_id(&a);
        let o = AmplifyOpts::default();
        for (what, needle) in [
            ("tile size", format!("px={TILE_PX}")),
            ("z_base", format!("zb={}", z_base())),
            ("appearance fingerprint", format!("fp={:016x}", appearance_fingerprint(&a))),
            ("detail frequency", format!("freq={}", o.detail_freq)),
            ("detail amplitude", format!("amp={}", o.detail_amp)),
            ("zoom detail k", format!("k={}", o.zoom_detail_k)),
        ] {
            assert!(id.contains(&needle), "{what} is not in the producer id: {id}");
        }
        // And the two that must NOT be: `synthesize_tile_rgba` takes both from
        // the live world, so the defaults are never used -- keying the cache
        // on them would invalidate on a value nothing reads.
        assert!(!id.contains(&format!("seed={}", o.seed)), "{id}");
        assert!(!id.contains(&format!("sea={}", o.sea)), "{id}");
        // The hand-bumped half, asserted so a reader knows it is not derived.
        // `v2` is LOD-D2's: a v1 tile is a shade-ratio mask and must be
        // refused by this id rather than decoded as colour.
        assert!(id.starts_with("cartalith-lod/v2;"), "{id}");
        assert!(!id.contains("cartalith-lod/v1"), "{id}");
    }

    /// The appearance reaches a stored tile through `land_color`, so a look
    /// change has to move the producer id — and the id must cover the whole
    /// struct, not the fields someone remembered.
    ///
    /// **Exercised field by field rather than asserted**, on the rule that a
    /// capability's coverage is derived from the definition: every tunable
    /// `TerrainAppearance` exposes is nudged in turn and each must move the
    /// id. A hand-list would only ever test the fields whose omission
    /// somebody already thought of.
    #[test]
    fn the_producer_id_moves_with_every_appearance_field() {
        let base = TestWorld::appearance();
        let id = tile_producer_id(&base);
        let mut checked = 0usize;
        for (name, lo, hi, _label) in render::TerrainAppearance::TUNABLE {
            let mut a = base.clone();
            // Away from wherever it currently sits, and inside its own range.
            let now = a.tunable(name).unwrap_or(*lo);
            let next = if (now - hi).abs() > 1e-9 { (now + (hi - lo) * 0.37).min(*hi) } else { *lo };
            if (next - now).abs() < 1e-12 {
                continue;
            }
            a.set_tunable(name, next);
            assert_ne!(tile_producer_id(&a), id, "moving `{name}` from {now} to {next} did not move the producer id");
            checked += 1;
        }
        assert!(checked > 20, "only {checked} tunables were actually varied -- this test is not covering what it claims");
        // The three that are not tunables: the light count, the named look's
        // whole ramp set, and the NPR flags.
        let mut a = base.clone();
        a.relief_lights += 1;
        assert_ne!(tile_producer_id(&a), id, "the light count is not in the producer id");
        let other = render::TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
        assert_ne!(tile_producer_id(&other), id, "a different named look is not in the producer id");
        let mut a = base.clone();
        a.npr.waves = !a.npr.waves;
        assert_ne!(tile_producer_id(&a), id, "the NPR flags are not in the producer id");
    }
    #[test]
    fn a_synthesized_pyramid_holds_every_tile_of_every_level() {
        let world = TestWorld::new(relief_field(128, 96), 128, 96);
        let ctx = world.ctx();
        let tf = world.fields(&ctx);
        let (tw, th, tiles) = synthesize_pyramid_masks(&ctx, &tf, 2, 7).unwrap();
        assert_eq!((tw, th), (256, 191), "one tile is 256 x round(256*95/127) px");
        assert_eq!(tiles.len(), 1 + 4 + 16);
        // Every tile is the one `synthesize_tile_rgba` would have produced on
        // demand -- the property the whole cache rests on.
        for z in 0..=2 {
            let n = tiles_per_axis(z);
            for col in 0..n {
                for row in 0..n {
                    let id = ChunkId::new(z as u32, col, row);
                    let mask = tiles.get(&id).expect("every address of every level");
                    assert_eq!(mask.len(), tw * th * 3);
                    let (rgba, _, _) = synthesize_tile_rgba(&ctx, &tf, z, col as i32, row as i32, 7).unwrap();
                    assert_eq!(*mask, tile_mask(&rgba), "tile {id:?} disagrees with on-demand synthesis");
                }
            }
        }
    }

    #[test]
    fn a_pyramid_is_refused_wherever_a_single_tile_would_be() {
        let world = TestWorld::new(relief_field(64, 64), 64, 64);
        let ctx = world.ctx();
        let tf = world.fields(&ctx);
        assert!(synthesize_pyramid_masks(&ctx, &tf, -1, 7).is_none());
        assert!(synthesize_pyramid_masks(&ctx, &tf, MAX_LEVEL + 1, 7).is_none());
        // A grid too small to tile at all, and a field shorter than its own
        // grid -- the two `synthesize_tile_rgba` itself refuses, reached here
        // through the same contexts it would be in production.
        let tiny = TestWorld::new(vec![0.5f32; 1], 1, 1);
        let tiny_ctx = tiny.ctx();
        assert!(synthesize_pyramid_masks(&tiny_ctx, &tiny.fields(&tiny_ctx), 0, 7).is_none());
        let short = vec![0.5f32; 10];
        let short_ctx = render::RenderCtx::from_precomputed(&short, &world.temp, &world.rain, None, 64, 64, TEST_SEA, false, 55.0, 5.0, TestWorld::appearance(), &world.pre).unwrap();
        assert!(synthesize_pyramid_masks(&short_ctx, &tf, 0, 7).is_none());
        assert!(pyramid_mask_bytes(1, 1, 0).is_none());
        assert!(pyramid_mask_bytes(64, 64, MAX_LEVEL + 1).is_none());
        assert!(pyramid_mask_bytes(64, 64, -1).is_none());
    }

    #[test]
    fn the_size_estimate_is_what_the_producer_actually_makes() {
        // A "size shown at save time" that disagrees with the save is worse
        // than none, so the estimate is checked against real bytes rather
        // than against its own formula.
        let world = TestWorld::new(relief_field(128, 96), 128, 96);
        let ctx = world.ctx();
        let tf = world.fields(&ctx);
        for z_max in 0..=2 {
            let (_, _, tiles) = synthesize_pyramid_masks(&ctx, &tf, z_max, 7).unwrap();
            let real: u64 = tiles.values().map(|t| t.len() as u64).sum();
            assert_eq!(pyramid_mask_bytes(128, 96, z_max), Some(real), "at z_max {z_max}");
        }
        // A literal rather than the formula restated. A 128x96 world's tile
        // aspect is (128-1)/(96-1) = 127/95, so `tile_dims` gives 256 x
        // round(256*95/127) = 256x191 px at every level, and levels 0..=1 are
        // five of them. Hand-computing 192 here is what this assertion caught.
        assert_eq!(pyramid_mask_bytes(128, 96, 1), Some(5 * 256 * 191 * 3));
    }

    /// **The measurement the cache exists for**, kept runnable rather than
    /// reported once — every figure quoted in [`render::GridPrecompute`]'s doc
    /// comment, in `WorldGen::lod_synthesize_tile`'s and in this module's
    /// header comes from here.
    ///
    /// `#[ignore]` because a timing taken under a parallel test suite is not a
    /// timing (`MISTAKES.md`: *"Run the harness alone, never under a parallel
    /// suite"*), and because it allocates a real 2048x1311 world's worth of
    /// rasters. Run it on its own, in release:
    ///
    /// ```text
    /// cargo test -p cartalith-godot --release --lib -- --ignored --nocapture --test-threads=1 the_tile_context_costs
    /// ```
    ///
    /// Median with min..max over five, never a point estimate. What it
    /// establishes is the **ratio**, which is what the design decision rested
    /// on: the context costs an order of magnitude more than the tile it
    /// serves, and `viewport_host.gd` asks for up to
    /// `MAX_LOD_TILES_PER_UPDATE` = 48 tiles per zoom notch.
    ///
    /// It asserts a regression ceiling rather than a budget. The budget for a
    /// once-per-world build is not this file's to set, and asserting the
    /// measured value as if it were one would turn an overrun into a green
    /// test — the same reasoning `golden_parity_tile_biome.rs`'s own timing
    /// test states for the per-tile figure.
    #[test]
    #[ignore = "a real 2048x1311 context; run alone, in release"]
    fn the_tile_context_costs_an_order_of_magnitude_more_than_the_tile_it_serves() {
        let (gw, gh) = (2048usize, 1311usize);
        let mut field = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let (u, v) = (x as f64 / gw as f64, y as f64 / gh as f64);
                field[y * gw + x] = (0.46 + 0.30 * (u * 6.0).sin() * (v * 5.0).cos() + 0.02 * (u * 61.0).sin()) as f32;
            }
        }
        let world = TestWorld::new(field, gw, gh);
        let med = |mut v: Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            (v[v.len() / 2], v[0], v[v.len() - 1])
        };
        let a = TestWorld::appearance();
        let pre_ms: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let p = render::GridPrecompute::build(&world.field, &world.temp, &world.rain, Some(&world.field), gw, gh, TEST_SEA, false, &a, Some(800.0));
                let e = t.elapsed().as_secs_f64() * 1000.0;
                assert!(p.bytes() > 0, "the precompute came back empty -- the timing below would be meaningless");
                e
            })
            .collect();
        let ctx = world.ctx();
        // **Both `TileFields` cases, because they are different functions.**
        // With a grid raster it also builds the local-contrast band (a luma
        // pass and two blurs); without one that stage is off, which is the
        // state a tile is in when no raster snapshot matches the cache key.
        // The band's cost is a function of the buffer's SIZE, not of its
        // content, so a cheap fill is a legitimate stand-in for a real raster
        // here and nothing but the timing reads it.
        let fake_rgb = vec![96u8; gw * gh * 3];
        let tf_ms: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let f = TileFields::new(&ctx, None);
                let e = t.elapsed().as_secs_f64() * 1000.0;
                assert!(f.bytes() > 0, "TileFields came back empty");
                e
            })
            .collect();
        let tfb_ms: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let f = TileFields::new(&ctx, Some(&fake_rgb));
                let e = t.elapsed().as_secs_f64() * 1000.0;
                assert!(f.bytes() > 0, "TileFields came back empty");
                e
            })
            .collect();
        let tf = world.fields(&ctx);
        // One tile of the same world, for the ratio. Warmed once, untimed.
        let _ = synthesize_tile_rgba(&ctx, &tf, 6, 20, 20, 1234).unwrap();
        let tile_ms: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let (px, tw, th) = synthesize_tile_rgba(&ctx, &tf, 6, 20, 20, 1234).unwrap();
                let e = t.elapsed().as_secs_f64() * 1000.0;
                // A 2048x1311 world is not square, so its tiles are not
                // `TILE_PX` on both axes -- `tile_size_px` is the one formula.
                assert_eq!((tw, th), tile_size_px(gw, gh, 6));
                assert_eq!(px.len(), tw * th * 4, "the timed tile came back the wrong size");
                e
            })
            .collect();
        let (p50, plo, phi) = med(pre_ms);
        let (f50, flo, fhi) = med(tf_ms);
        let (b50, blo, bhi) = med(tfb_ms);
        let (t50, tlo, thi) = med(tile_ms);
        println!("{gw}x{gh}: GridPrecompute {p50:.1} ms ({plo:.1}..{phi:.1}) | TileFields no band {f50:.1} ms ({flo:.1}..{fhi:.1}) | TileFields with band {b50:.1} ms ({blo:.1}..{bhi:.1}) | one tile {t50:.2} ms ({tlo:.2}..{thi:.2}), all cores");
        println!("context/tile ratio: {:.1}x with the band, {:.1}x without", (p50 + b50) / t50.max(1e-9), (p50 + f50) / t50.max(1e-9));
        assert!(p50 + b50 > t50 * 3.0, "the context is no longer worth caching against the tile it serves: {p50:.1} + {b50:.1} vs {t50:.2} ms");
        assert!(p50 < 900.0 && b50 < 1400.0, "a context build regressed past its measured cost by 4x: {p50:.1} / {b50:.1} ms");
    }

    // -- what a stored pyramid actually costs ------------------------------

    fn env_usize(key: &str, default: usize) -> usize {
        std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    /// Owner ruling 28 asks for measured bytes before any default is written
    /// into the UI. This is that measurement, kept runnable rather than
    /// reported once: it generates a real world, runs the **shipping**
    /// producer ([`synthesize_pyramid_masks`]) over it, and reports the bytes
    /// **as the archive would hold them** -- every tile written into a real
    /// deflate zip and read back for its compressed size, not a codec
    /// measured in isolation.
    ///
    /// It measures the RGB-PNG alternative in the same pass, which is what
    /// settled the encoding: a PNG's own deflate never sees the redundancy
    /// the container's does, so the same pyramid costs ~2.9x more.
    ///
    /// ```text
    /// cargo test -p cartalith-godot --release --lib -- --ignored --nocapture measure_a_stored_pyramid
    /// CARTALITH_LOD_GW=2048 CARTALITH_LOD_GH=1311 CARTALITH_LOD_ZMAX=6 CARTALITH_LOD_SEED=24601 ...
    /// ```
    ///
    /// What it printed, 2026-09-06, release, three real 2048x1311 worlds
    /// (seeds 1337 / 987654 / 24601), deflated **inside the archive**:
    ///
    /// | levels | stored | against the 24.9 MiB whole archive at this grid |
    /// |---|---|---|
    /// | 0..=5 | 4.40 .. 5.12 MiB | ~19% |
    /// | 0..=6 | 21.87 .. 27.75 MiB | **88% .. 111%** |
    #[test]
    #[ignore = "generates a real world and synthesizes a whole pyramid; run explicitly"]
    fn measure_a_stored_pyramid() {
        use std::io::Write as _;
        let gw = env_usize("CARTALITH_LOD_GW", 512);
        let gh = env_usize("CARTALITH_LOD_GH", 384);
        let zmax = env_usize("CARTALITH_LOD_ZMAX", 3) as i32;
        let seed = env_usize("CARTALITH_LOD_SEED", 24601) as i32;

        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
        p.map_width_km = 800.0;
        let t = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        println!("generate_terrain {gw}x{gh} seed {seed}: {:.2}s", t.elapsed().as_secs_f64());
        let lo = ws.field.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ws.field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(hi - lo > 0.2, "the generated world is nearly flat: [{lo}, {hi}]");

        let a = TestWorld::appearance();
        let pre = render::GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
        let ctx = render::RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("a real world builds a context");
        let tf = TileFields::new(&ctx, None);
        let t_all = std::time::Instant::now();
        let (tw, th, tiles) = synthesize_pyramid_masks(&ctx, &tf, zmax, seed).expect("the pyramid synthesizes");
        println!(
            "{} tiles of {tw}x{th}px, synth {:.2}s",
            tiles.len(),
            t_all.elapsed().as_secs_f64()
        );
        let raw_total = pyramid_mask_bytes(gw, gh, zmax).expect("a real world has an estimate");
        assert_eq!(
            raw_total,
            tiles.values().map(|t| t.len() as u64).sum::<u64>(),
            "the save-time estimate must be the real byte count"
        );

        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            for (id, mask) in &tiles {
                w.start_file(format!("mask/{}/{}/{}.u8", id.z, id.col, id.row), opts).unwrap();
                w.write_all(mask).unwrap();
                let png = cartalith_assets::raster::encode_png_rgb8(tw as u32, th as u32, mask.clone()).expect("the tile encodes");
                w.start_file(format!("png/{}/{}/{}.png", id.z, id.col, id.row), stored).unwrap();
                w.write_all(&png).unwrap();
            }
            w.finish().unwrap();
        }

        let mut r = zip::ZipArchive::new(std::io::Cursor::new(&buf)).unwrap();
        let mut mask_z = vec![0u64; zmax as usize + 1];
        let mut png_z = vec![0u64; zmax as usize + 1];
        for i in 0..r.len() {
            let e = r.by_index_raw(i).unwrap();
            let name = e.name().to_string();
            let z: usize = name.split('/').nth(1).unwrap().parse().unwrap();
            if name.starts_with("mask/") {
                mask_z[z] += e.compressed_size();
            } else {
                png_z[z] += e.compressed_size();
            }
        }
        let mib = |b: u64| b as f64 / (1024.0 * 1024.0);
        println!("level | tiles | raw mask | deflated in-zip | RGB PNG stored");
        for z in 0..=zmax as usize {
            let n = 1u64 << z;
            println!(
                "  z{z} | {:>7} | {:>8.2} MiB | {:>8.2} MiB | {:>8.2} MiB",
                n * n,
                mib(n * n * (tw * th) as u64),
                mib(mask_z[z]),
                mib(png_z[z])
            );
        }
        let deflated: u64 = mask_z.iter().sum();
        println!(
            "TOTAL 0..={zmax}: raw {:.2} MiB, deflated {:.2} MiB ({:.1}% of raw), RGB PNG {:.2} MiB",
            mib(raw_total),
            mib(deflated),
            100.0 * deflated as f64 / raw_total as f64,
            mib(png_z.iter().sum::<u64>())
        );
    }
}
