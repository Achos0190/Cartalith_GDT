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

use cartalith_spatial::{FloatRegion, Region};
use cartalith_terrain::amplify::{amplify_region, AmplifyOpts};
use cartalith_terrain::tile_render::render_height_tile_rgba;

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

/// Synthesizes and colours one deep-zoom tile: [`tile_bounds`] to find its
/// coarse-grid footprint, `amplify_region` to upsample+detail it to
/// `tile_px_for_level(detail_level)` pixels, `render_height_tile_rgba` to
/// colour the result — the exact pipeline `region_export_tiles`'s own
/// `visual: true` branch runs per export tile, called here for one
/// interactive tile instead of a whole export bundle.
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
    let region: FloatRegion = bounds.to_float();
    let out = tile_px_for_level(detail_level);
    let opts = AmplifyOpts { seed, sea, ..AmplifyOpts::default() };
    let heights = amplify_region(field, gw, gh, &region, out, out, &opts);
    let rgba = render_height_tile_rgba(&heights, out, out, sea, SUN_AZ_DEG, EXAG);
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
    fn synthesize_tile_rgba_is_not_a_flat_colour() {
        // A silently-constant tile passes every structural check above, so
        // say it explicitly -- same reasoning tile_render.rs's own
        // `render_is_not_flat` test states for the function this wraps.
        let field = synthetic_field(256, 256);
        let (rgba, _, _) = synthesize_tile_rgba(&field, 256, 256, 0, 0, 0, 1234, 0.42).unwrap();
        let distinct: std::collections::HashSet<u8> = rgba.iter().copied().collect();
        assert!(distinct.len() > 20, "only {} distinct byte values", distinct.len());
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
