//! Interactive per-tile deep-zoom synthesis — `LOD_TILING_INTEGRATION_SCOPE.md`
//! milestone M1: "a minimal interactive Z2... tile the deep-zoom case only,
//! not the whole map."
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own module doc argues for: `lib.rs` owns the thin
//! `Variant`<->Rust conversion and the `#[func]` surface (`lod_tile_cells`/
//! `lod_synthesize_tile`); this module owns the actual synthesis — which
//! tile, at what bounds, coloured how — with its own `#[cfg(test)]` suite,
//! exercised by plain `cargo test -p cartalith-godot` with no Godot runtime.
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
//!   reproduces that same clipping formula (`min(tile_size, remaining)`)
//!   directly against borrowed `gw`/`gh`, at the actual cost the query has —
//!   O(1), not O(field size).
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
//! range of `(tile_x, tile_y)` via [`TILE_CELLS`] is the same shape of
//! computation. This module supplies [`tile_bounds`] so the *authoritative*
//! clipped bounds for any given tile index are computed once, in Rust, from
//! the real `gw`/`gh` — GDScript never has to duplicate the edge-clipping
//! rule to stay in agreement with what a tile request actually returns.
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

use cartalith_spatial::{FloatRegion, Region};
use cartalith_terrain::amplify::{amplify_region, AmplifyOpts};
use cartalith_terrain::tile_render::{shade_tile, u8_clamped};

/// Coarse grid cells spanned by one synthesized tile, along each axis.
///
/// This port's own choice — the reference has no per-tile interactive LOD
/// concept to match (its `drawLODView` re-synthesizes the whole visible
/// canvas each frame, not a tile grid) — picked for the same reason
/// `sculpt_bridge::SCULPT_TILE_SIZE` independently landed on the same
/// number: small enough that a typical deep-zoom viewport touches a handful
/// of tiles, not one giant one (no locality benefit) or hundreds of tiny
/// ones (per-tile call overhead for no reason) at this port's 512-8192
/// range.
pub const TILE_CELLS: usize = 64;

/// Output resolution (pixels, square) for a tile at `detail_level == 0`.
pub const BASE_TILE_PX: usize = 256;

/// `detail_level` above this is clamped rather than honoured — a defensive
/// ceiling against a runaway caller value, not a reference constant. At
/// [`MAX_DETAIL_LEVEL`] the output is `BASE_TILE_PX << 2` = 1024px per tile,
/// comfortably inside a single `Image`/`ImageTexture` and cheap relative to
/// the ~7s *whole-map* `build_color_texture` estimate at the 8192 ceiling
/// (`LOD_TILING_INTEGRATION_SCOPE.md` §1) — this synthesizes one small tile,
/// not the map.
pub const MAX_DETAIL_LEVEL: i32 = 2;

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

/// `BASE_TILE_PX` doubled once per `detail_level`, clamped to
/// `[0, MAX_DETAIL_LEVEL]` first so an out-of-range caller value degrades to
/// the nearest real tier instead of panicking or allocating unboundedly.
pub fn tile_px_for_level(detail_level: i32) -> usize {
    let lvl = detail_level.clamp(0, MAX_DETAIL_LEVEL) as u32;
    BASE_TILE_PX << lvl
}

/// The `(tile_x, tile_y)` tile's bounds in a `gw x gh` field's own cell
/// grid, clipped at the field's real edges — see this module's own top doc
/// comment for why this is plain arithmetic rather than a real
/// `TiledField::tile_bounds` call. `None` for a negative index, an empty
/// field (`gw == 0 || gh == 0` — no `generate()`/`load_save()` yet), or a
/// tile index whose origin already falls outside the field.
pub fn tile_bounds(gw: usize, gh: usize, tile_x: i32, tile_y: i32) -> Option<Region> {
    if gw == 0 || gh == 0 || tile_x < 0 || tile_y < 0 {
        return None;
    }
    let x = (tile_x as usize).checked_mul(TILE_CELLS)?;
    let y = (tile_y as usize).checked_mul(TILE_CELLS)?;
    if x >= gw || y >= gh {
        return None;
    }
    let w = TILE_CELLS.min(gw - x);
    let h = TILE_CELLS.min(gh - y);
    Some(Region::new(x, y, w, h))
}

/// The continuous-coordinate region to hand `amplify_region` so that an
/// `out`-pixel-square tile's **texel centres** land on the same ground the
/// base raster draws at the same screen position.
///
/// `amplify_region` maps output index `ox` to source coordinate
/// `rx + ox/(out-1) * (rw-1)` — endpoints inclusive, a *sample* convention.
/// The base raster is a `gw × gh` texture stretched over the map rect, so its
/// texel `i` covers screen span `[i, i+1)` cells and its centre sits at
/// `i + 0.5`; `viewport_host.gd`'s `_lod_tile_rect` places a tile over the
/// screen span `[bx, bx+bw]` to match. Passing `bounds.to_float()` straight
/// through — what this module did until 2026-08-23 — therefore stretched `bw`
/// cells' worth of screen over `bw - 1` cells' worth of data (a 1.6% scale
/// error at `TILE_CELLS == 64`) *and* offset it half a cell, so a tile's
/// relief sat visibly off the terrain it belongs to and every tile boundary
/// was a discontinuity. Solving `cx + 0.5 == bx + (ox + 0.5) * bw / out` for
/// the region gives this, and adjacent tiles then sample exactly one texel
/// apart across their shared edge — continuous, no seam.
fn tile_sample_region(bounds: Region, out: usize) -> FloatRegion {
    let out_f = out as f64;
    let (bw, bh) = (bounds.w as f64, bounds.h as f64);
    FloatRegion {
        x: bounds.x as f64 - 0.5 + 0.5 * bw / out_f,
        y: bounds.y as f64 - 0.5 + 0.5 * bh / out_f,
        w: 1.0 + bw * (out_f - 1.0) / out_f,
        h: 1.0 + bh * (out_f - 1.0) / out_f,
    }
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
/// [`tile_bounds`] finds the coarse-grid footprint, [`tile_sample_region`]
/// turns it into sampling coordinates aligned with the base raster,
/// `amplify_region` produces the tile twice (with the procedural detail, and
/// with `detail_amp = 0` — the plain bilinear upsample the base raster's own
/// shading already reflects), and `shade_tile` reduces each to the multiplier
/// `render_height_tile_rgba` would have applied. Their ratio is what the
/// detail *adds*, which is exactly what is missing from the base raster and
/// nothing else.
///
/// `seed`/`sea` are read from the caller's own world state (`WorldGen::seed`/
/// `sea_level`), the same convention `region_export_tiles` already uses —
/// "an export must match the world it was drawn over, not a caller-guessed
/// one" applies just as much to an interactive tile.
///
/// Returns `(rgba_bytes, out_w, out_h)` — `rgba_bytes.len() == out_w * out_h
/// * 4`, ready to hand `Image::create_from_data` directly. `None` for
/// anything [`tile_bounds`] itself rejects, or when `field` is shorter than
/// `gw * gh` — the same precondition `amplify_region` would otherwise panic
/// on, checked here instead so a caller error surfaces as "no tile" rather
/// than taking the whole Godot process down with it
/// (`cartalith-rust-conventions`: no panic crosses the gdext boundary).
pub fn synthesize_tile_rgba(
    field: &[f32],
    gw: usize,
    gh: usize,
    tile_x: i32,
    tile_y: i32,
    detail_level: i32,
    seed: i32,
    sea: f64,
) -> Option<(Vec<u8>, usize, usize)> {
    if field.len() < gw.checked_mul(gh)? {
        return None;
    }
    let bounds = tile_bounds(gw, gh, tile_x, tile_y)?;
    let out = tile_px_for_level(detail_level);
    let region = tile_sample_region(bounds, out);
    let opts = AmplifyOpts { seed, sea, ..AmplifyOpts::default() };
    let detailed = amplify_region(field, gw, gh, &region, out, out, &opts);
    // The same region, same sampler, same clamping -- only the detail term
    // switched off. Reusing `amplify_region` rather than reimplementing its
    // bilinear upsample is what guarantees the two differ *only* by the
    // detail, which is the entire meaning of the ratio below.
    let plain_opts = AmplifyOpts { detail_amp: 0.0, ..opts };
    let plain = amplify_region(field, gw, gh, &region, out, out, &plain_opts);

    let sd = shade_tile(&detailed, out, out, sea, SUN_AZ_DEG, EXAG);
    let sp = shade_tile(&plain, out, out, sea, SUN_AZ_DEG, EXAG);

    let mut rgba = vec![255u8; out * out * 4];
    for i in 0..out * out {
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
    Some((rgba, out, out))
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

    #[test]
    fn tile_bounds_none_before_any_world() {
        assert_eq!(tile_bounds(0, 0, 0, 0), None);
    }

    #[test]
    fn tile_bounds_none_for_negative_index() {
        assert_eq!(tile_bounds(256, 256, -1, 0), None);
        assert_eq!(tile_bounds(256, 256, 0, -1), None);
    }

    #[test]
    fn tile_bounds_none_past_the_grid() {
        // 256 / 64 = exactly 4 tiles per axis; index 4 starts at cell 256,
        // already outside a 256-wide field.
        assert_eq!(tile_bounds(256, 256, 4, 0), None);
        assert_eq!(tile_bounds(256, 256, 0, 4), None);
    }

    #[test]
    fn tile_bounds_interior_tile_is_a_full_tile_cells_square() {
        let r = tile_bounds(256, 256, 1, 2).unwrap();
        assert_eq!(r, Region::new(64, 128, 64, 64));
    }

    #[test]
    fn tile_bounds_edge_tile_is_clipped_not_out_of_range() {
        // 300 is not a multiple of TILE_CELLS (64): tiles cover
        // 64,64,64,64,44 along that axis.
        let r = tile_bounds(300, 300, 4, 4).unwrap();
        assert_eq!(r, Region::new(256, 256, 44, 44));
    }

    #[test]
    fn tile_px_for_level_doubles_per_level_and_clamps() {
        assert_eq!(tile_px_for_level(0), 256);
        assert_eq!(tile_px_for_level(1), 512);
        assert_eq!(tile_px_for_level(2), 1024);
        // Clamped both directions rather than panicking or overflowing.
        assert_eq!(tile_px_for_level(-5), 256);
        assert_eq!(tile_px_for_level(99), 1024);
    }

    #[test]
    fn synthesize_tile_rgba_none_for_a_too_short_field() {
        // Guards amplify_region's own panic precondition rather than
        // letting it panic across what would be the gdext boundary.
        let short = vec![0.5f32; 10];
        assert_eq!(synthesize_tile_rgba(&short, 64, 64, 0, 0, 0, 1234, 0.42), None);
    }

    #[test]
    fn synthesize_tile_rgba_none_for_an_out_of_range_tile() {
        let field = synthetic_field(128, 128);
        assert_eq!(synthesize_tile_rgba(&field, 128, 128, 10, 10, 0, 1234, 0.42), None);
    }

    #[test]
    fn synthesize_tile_rgba_produces_the_right_number_of_opaque_pixels() {
        let field = synthetic_field(256, 256);
        let (rgba, w, h) = synthesize_tile_rgba(&field, 256, 256, 0, 0, 0, 1234, 0.42).unwrap();
        assert_eq!(w, 256);
        assert_eq!(h, 256);
        assert_eq!(rgba.len(), w * h * 4);
        assert!(rgba.chunks(4).all(|p| p[3] == 255), "every pixel must be opaque");
    }

    #[test]
    fn synthesize_tile_rgba_respects_detail_level_resolution() {
        let field = synthetic_field(256, 256);
        let (rgba, w, h) = synthesize_tile_rgba(&field, 256, 256, 0, 0, 1, 1234, 0.42).unwrap();
        assert_eq!((w, h), (512, 512));
        assert_eq!(rgba.len(), 512 * 512 * 4);
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
        // Tile (2, 2), not (0, 0): `synthetic_field` ramps west-to-east from
        // 0.25, so the western tiles are entirely below `sea` and correctly
        // come back flat-neutral (deep water gets no detail). Asserting
        // variation there would pin the wrong thing.
        let (rgba, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 2, 0, 1234, 0.42).unwrap();
        let distinct: std::collections::HashSet<u8> =
            rgba.chunks(4).map(|p| p[0]).collect();
        assert!(distinct.len() > 8, "only {} distinct shade levels", distinct.len());
        // Grey, not tinted: R == G == B everywhere, alpha opaque.
        assert!(rgba.chunks(4).all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
    }

    #[test]
    fn a_tile_with_no_added_detail_is_exactly_neutral() {
        // The property the whole fix rests on: where `amplify_region` adds
        // nothing, the ratio is 1.0 and the shader leaves the base map's own
        // pixels alone. Deep water tapers the detail term to zero
        // (`amplify_region`'s `underwater`/`taper`), so a field well below sea
        // level must come back as the encoded identity, byte for byte -- not
        // "close to" it, which is what centring the fixed point on
        // `SHADE_RATIO_MID` buys.
        let field = vec![0.10f32; 128 * 128];
        let (rgba, _, _) = synthesize_tile_rgba(&field, 128, 128, 0, 0, 0, 1234, 0.42).unwrap();
        let neutral = SHADE_RATIO_MID as u8;
        assert!(
            rgba.chunks(4).all(|p| p[0] == neutral && p[1] == neutral && p[2] == neutral),
            "a detail-free tile must encode exactly {neutral}"
        );
    }

    #[test]
    fn a_detailed_land_tile_actually_perturbs_the_base() {
        // The other half of the property above: where detail *is* added, the
        // mask must leave neutral. A field above sea level with real relief
        // (so `taper` is non-zero) is the case the milestone exists for.
        let field = synthetic_field(256, 256);
        let (rgba, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 2, 0, 1234, 0.42).unwrap();
        let neutral = SHADE_RATIO_MID as u8;
        let moved = rgba.chunks(4).filter(|p| p[0] != neutral).count();
        assert!(moved > rgba.len() / 4 / 2, "only {moved} pixels carry any detail shading");
    }

    #[test]
    fn tile_sample_region_puts_texel_centres_on_cell_centres() {
        // `amplify_region` maps ox -> rx + ox/(out-1)*(rw-1). Restate that
        // here and check both ends land where `_lod_tile_rect` draws them:
        // texel `ox`'s centre must sit at cell coordinate
        // `bx - 0.5 + (ox + 0.5) * bw / out`.
        let out = 256usize;
        let b = Region::new(64, 128, 64, 64);
        let r = tile_sample_region(b, out);
        let at = |ox: usize| r.x + (ox as f64 / (out as f64 - 1.0)) * (r.w - 1.0);
        let want = |ox: usize| 64.0 - 0.5 + (ox as f64 + 0.5) * 64.0 / out as f64;
        for ox in [0usize, 1, 137, out - 1] {
            assert!((at(ox) - want(ox)).abs() < 1e-9, "texel {ox}: {} vs {}", at(ox), want(ox));
        }
    }

    #[test]
    fn adjacent_tiles_sample_exactly_one_texel_apart() {
        // The seam test. Tile n's last sample and tile n+1's first sample must
        // be one texel step apart -- no overlap (double-drawn ground) and no
        // gap (a discontinuity that reads as a hairline down every tile edge).
        let out = 256usize;
        let step = 64.0 / out as f64;
        let left = tile_sample_region(Region::new(0, 0, 64, 64), out);
        let right = tile_sample_region(Region::new(64, 0, 64, 64), out);
        let left_last = left.x + (left.w - 1.0);
        assert!((right.x - left_last - step).abs() < 1e-9, "{} -> {}", left_last, right.x);
    }

    #[test]
    fn an_edge_clipped_tile_still_aligns() {
        // 300 is not a multiple of TILE_CELLS: the last tile is 44 cells wide
        // but still 256 px, so its texel step differs from an interior tile's
        // and the alignment maths has to hold on both.
        let out = 256usize;
        let r = tile_sample_region(Region::new(256, 256, 44, 44), out);
        let at = |ox: usize| r.x + (ox as f64 / (out as f64 - 1.0)) * (r.w - 1.0);
        let want = |ox: usize| 256.0 - 0.5 + (ox as f64 + 0.5) * 44.0 / out as f64;
        for ox in [0usize, out - 1] {
            assert!((at(ox) - want(ox)).abs() < 1e-9, "texel {ox}: {} vs {}", at(ox), want(ox));
        }
    }

    #[test]
    fn different_tiles_of_the_same_world_synthesize_different_content() {
        let field = synthetic_field(256, 256);
        let (a, _, _) = synthesize_tile_rgba(&field, 256, 256, 0, 0, 0, 1234, 0.42).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&field, 256, 256, 2, 2, 0, 1234, 0.42).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_for_the_same_inputs() {
        // Same standard `PARITY_TESTING.md`-adjacent expectation every
        // synthesis path in this crate holds to: no hidden randomness.
        let field = synthetic_field(200, 200);
        let (a, _, _) = synthesize_tile_rgba(&field, 200, 200, 1, 1, 1, 42, 0.42).unwrap();
        let (b, _, _) = synthesize_tile_rgba(&field, 200, 200, 1, 1, 1, 42, 0.42).unwrap();
        assert_eq!(a, b);
    }
}
