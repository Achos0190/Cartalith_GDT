//! The LOD tile pyramid's addressing — the index arithmetic the reference's
//! bake, atlas and deep-zoom viewer all share.
//!
//! Five pure functions, ported from reference lines 10461-10645:
//! `pyramidDims` (10461), `pyramidTileBounds` (10594), `pyramidLevelForZoom`
//! (10600), `tilesInView` (10637), and the `chunkParent`/`chunkChildren` pair
//! (10933-10934, which the reference keeps beside its chunk-debug overlay but
//! which `bakedCover`'s ancestor walk is the real consumer of).
//!
//! **Why `cartalith-spatial`, and why not `TiledField`.** This is the same
//! placement argument [`crate::region`]'s own header makes — *"a clamped
//! integer rectangle over a grid is exactly the generic spatial machinery this
//! crate exists for: neither function knows what a heightmap is"* — and it
//! holds here word for word. Nothing below reads a single cell of any field.
//! [`crate::TiledField`] is a different tiling: **fixed-size** tiles over a
//! field it owns, whereas a pyramid level splits the *whole* field into
//! `2^z × 2^z` tiles whose coarse-cell footprint therefore shrinks with depth
//! and is generally fractional. Both are "tiling"; only one of them is this.
//!
//! # The one-cell inset, which is not an off-by-one
//!
//! Every function here works over `cW - 1` × `cH - 1`, not `cW` × `cH`. That is
//! the reference's own convention and it is deliberate: a pyramid tile is
//! sampled by `refineTile`, whose coordinates are *sample* coordinates (cell
//! **centres**, endpoints inclusive), so the addressable span of a `cW`-wide
//! field is `[0, cW-1]` — `cW - 1` wide, not `cW`. `pyramidTile` passes exactly
//! `{x:0, y:0, w:cW-1, h:cH-1}` as its region for the same reason. Widening
//! this to `cW` would put every tile's right edge one cell past the last sample
//! and shear the whole pyramid against the base raster.

use crate::FloatRegion;
use serde::{Deserialize, Serialize};

/// One tile's address in the pyramid: level `z`, column, row.
///
/// The reference passes `(z, col, row)` as three loose arguments everywhere;
/// they travel together through the atlas key, the bake loop and the ancestor
/// walk, so they are one value here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChunkId {
    pub z: u32,
    pub col: u32,
    pub row: u32,
}

impl ChunkId {
    pub const fn new(z: u32, col: u32, row: u32) -> Self {
        ChunkId { z, col, row }
    }
}

/// A pyramid level's tile grid. `pyramidDims`' own return shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyramidDims {
    pub cols: u32,
    pub rows: u32,
}

/// `pyramidDims(z)` (reference line 10461) — `2^z × 2^z` tiles per level.
///
/// The reference's `1 << Math.max(0, z|0)` floors a negative level at 0, which
/// this reproduces by taking `i32` and clamping rather than by taking `u32` and
/// making the case unrepresentable: `pyramid_level_for_zoom` and the debug
/// overlay both hand it computed values, and silently agreeing with the
/// reference on a negative one is cheaper than proving no caller can produce
/// it.
///
/// Levels above 31 would overflow the shift; they are clamped to 31, which is
/// far beyond `state.lodMaxLevel`'s own ceiling of 8 and exists only so a
/// runaway caller cannot panic across the gdext boundary.
pub fn pyramid_dims(z: i32) -> PyramidDims {
    let n = 1u32 << z.clamp(0, 31);
    PyramidDims { cols: n, rows: n }
}

/// Total tiles in a pyramid of depth `max_z` inclusive — `(4^(max_z+1) - 1)/3`,
/// the count the reference's own `bakeAllTiles` comment spells out (*"depth 3 =
/// 85 tiles, depth 4 = 341, depth 5 = 1365 (large!)"*).
///
/// Saturating rather than wrapping: this feeds a progress denominator and a
/// user-facing estimate, and a wrapped one would read as a plausible small
/// number rather than as the absurdity it is.
pub fn pyramid_tile_count(max_z: i32) -> u64 {
    let max_z = max_z.clamp(0, 15);
    (0..=max_z).map(|z| { let n = 1u64 << z; n * n }).sum()
}

/// `pyramidTileBounds(cW, cH, z, col, row)` (reference line 10594) — the
/// world rectangle, in coarse cells, that one tile covers.
///
/// Fractional by construction (`stepX = (cW-1)/cols`), which is why it returns
/// a [`FloatRegion`]: rounding it to whole cells would break the exact seam
/// agreement between adjacent tiles that the pyramid rests on, the same
/// argument [`FloatRegion`]'s own doc comment makes for `refineTile`.
pub fn pyramid_tile_bounds(cw: usize, ch: usize, z: i32, col: u32, row: u32) -> FloatRegion {
    let d = pyramid_dims(z);
    let step_x = (cw as f64 - 1.0) / d.cols as f64;
    let step_y = (ch as f64 - 1.0) / d.rows as f64;
    FloatRegion { x: col as f64 * step_x, y: row as f64 * step_y, w: step_x, h: step_y }
}

/// `pyramidLevelForZoom(scale, baseW, tileSize, maxLevel)` (reference line
/// 10600) — the reference's own *"reverse mipmap"*: the pyramid level whose
/// tile pixels best match the on-screen pixels at a given zoom.
///
/// `max_level` is `None` where the reference passes `null`, which it defaults
/// to 6 (its callers pass `state.lodMaxLevel || 8`).
pub fn pyramid_level_for_zoom(scale: f64, base_w: f64, tile_size: f64, max_level: Option<i32>) -> i32 {
    // `Math.max(0.01, scale)` first, exactly as written -- a zero or negative
    // scale would otherwise put `log2` at -inf and `Math.round` at NaN.
    let want = f64::max(1.0, (base_w * f64::max(0.01, scale)) / tile_size);
    let lvl = cartalith_jsmath::js_round(want.log2());
    // `Math.max(0, Math.min(maxLevel, ...))` -- min before max, as written.
    let cap = max_level.unwrap_or(6) as f64;
    f64::max(0.0, f64::min(cap, lvl)) as i32
}

/// `tilesInView`'s return shape (reference line 10637).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilesInView {
    pub cols: u32,
    pub rows: u32,
    pub c0: u32,
    pub c1: u32,
    pub r0: u32,
    pub r1: u32,
    /// `(c1-c0+1) * (r1-r0+1)`, reproduced in signed arithmetic because the
    /// reference's is: a rectangle passed corners-reversed (`vx1 < vx0`) gives
    /// a non-positive count there, and rounding that up to 1 here would be a
    /// silent divergence in the one value a progress readout displays.
    pub count: i64,
}

/// `tilesInView(z, vx0, vy0, vx1, vy1, cW, cH)` (reference line 10637) — the
/// inclusive tile-index range a view rectangle (in coarse cells) touches.
///
/// A rectangle entirely off the grid does **not** come back empty: the
/// reference clamps each edge into `[0, cols-1]` independently, so an
/// off-to-the-north-west view yields the single tile `(0, 0)`. Ported as
/// written — the viewer relies on always having something to draw.
pub fn tiles_in_view(z: i32, vx0: f64, vy0: f64, vx1: f64, vy1: f64, cw: usize, ch: usize) -> TilesInView {
    let d = pyramid_dims(z);
    let sx = (cw as f64 - 1.0) / d.cols as f64;
    let sy = (ch as f64 - 1.0) / d.rows as f64;
    // `Math.max(0, Math.min(n-1, Math.floor(v)))`. A non-finite index (an empty
    // field makes `sx` zero, so `0/0` is NaN) lands at 0 rather than panicking
    // on the cast -- the same "no panic crosses the boundary" rule the rest of
    // this port holds to, and the same tile the reference's clamp would pick
    // for any unusable value.
    let clampc = |v: f64, n: u32| -> u32 {
        let f = v.floor();
        if !f.is_finite() { return 0; }
        f.clamp(0.0, (n - 1) as f64) as u32
    };
    let c0 = clampc(vx0 / sx, d.cols);
    let c1 = clampc(vx1 / sx, d.cols);
    let r0 = clampc(vy0 / sy, d.rows);
    let r1 = clampc(vy1 / sy, d.rows);
    let count = (i64::from(c1) - i64::from(c0) + 1) * (i64::from(r1) - i64::from(r0) + 1);
    TilesInView { cols: d.cols, rows: d.rows, c0, c1, r0, r1, count }
}

/// `chunkParent(z, col, row)` (reference line 10933) — the tile one level
/// shallower that covers the same ground. `None` at the root.
pub fn chunk_parent(c: ChunkId) -> Option<ChunkId> {
    if c.z == 0 { None } else { Some(ChunkId::new(c.z - 1, c.col >> 1, c.row >> 1)) }
}

/// `chunkChildren(z, col, row)` (reference line 10934) — the four tiles one
/// level deeper, in the reference's own order (NW, NE, SW, SE).
pub fn chunk_children(c: ChunkId) -> [ChunkId; 4] {
    let n = c.z + 1;
    let (x, y) = (c.col * 2, c.row * 2);
    [
        ChunkId::new(n, x, y),
        ChunkId::new(n, x + 1, y),
        ChunkId::new(n, x, y + 1),
        ChunkId::new(n, x + 1, y + 1),
    ]
}

/// `bakedCover(z, col, row)` (reference line 10715) — is this chunk, **or any
/// ancestor of it**, already baked?
///
/// The reference's own comment: *"drives 'no refinement beneath baked
/// chunks'"* — a baked shallow tile is authoritative over everything under it,
/// so the viewer must not re-synthesise a deeper tile inside it and the editor
/// must not compose an edit into it.
///
/// `is_baked` is a predicate rather than a set so the caller owns the storage;
/// the reference's is a `Set` of key strings, which is `cartalith-io`'s
/// concern, not this crate's.
pub fn baked_cover(c: ChunkId, is_baked: impl Fn(ChunkId) -> bool) -> bool {
    let mut cur = Some(c);
    while let Some(k) = cur {
        if is_baked(k) {
            return true;
        }
        cur = chunk_parent(k);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_double_per_level_and_floor_at_the_root() {
        assert_eq!(pyramid_dims(0), PyramidDims { cols: 1, rows: 1 });
        assert_eq!(pyramid_dims(3), PyramidDims { cols: 8, rows: 8 });
        // The reference's `Math.max(0, z|0)`: a negative level is the root.
        assert_eq!(pyramid_dims(-1), PyramidDims { cols: 1, rows: 1 });
    }

    #[test]
    fn tile_count_matches_the_references_own_worked_numbers() {
        // From `bakeAllTiles`' own comment, verbatim.
        assert_eq!(pyramid_tile_count(3), 85);
        assert_eq!(pyramid_tile_count(4), 341);
        assert_eq!(pyramid_tile_count(5), 1365);
        assert_eq!(pyramid_tile_count(0), 1);
    }

    #[test]
    fn a_levels_tiles_tile_the_whole_inset_field_exactly() {
        // Every tile at a level, laid end to end, must cover [0, cW-1] with no
        // gap and no overlap -- the property the fractional step exists for.
        let (cw, ch) = (48usize, 32usize);
        for z in 0..4 {
            let d = pyramid_dims(z);
            let last = pyramid_tile_bounds(cw, ch, z, d.cols - 1, d.rows - 1);
            assert!((last.x + last.w - (cw as f64 - 1.0)).abs() < 1e-12, "z={z} right edge");
            assert!((last.y + last.h - (ch as f64 - 1.0)).abs() < 1e-12, "z={z} bottom edge");
            for c in 1..d.cols {
                let prev = pyramid_tile_bounds(cw, ch, z, c - 1, 0);
                let this = pyramid_tile_bounds(cw, ch, z, c, 0);
                assert!((prev.x + prev.w - this.x).abs() < 1e-12, "z={z} col {c} seam");
            }
        }
    }

    #[test]
    fn a_view_entirely_off_the_grid_still_names_one_tile() {
        let t = tiles_in_view(2, -50.0, -50.0, -40.0, -40.0, 48, 32);
        assert_eq!((t.c0, t.c1, t.r0, t.r1), (0, 0, 0, 0));
        assert_eq!(t.count, 1);
    }

    #[test]
    fn parent_and_children_are_inverses() {
        let c = ChunkId::new(3, 5, 7);
        for kid in chunk_children(c) {
            assert_eq!(chunk_parent(kid), Some(c));
        }
        assert_eq!(chunk_parent(ChunkId::new(0, 0, 0)), None);
    }

    #[test]
    fn baked_cover_walks_all_the_way_to_the_root() {
        let root = ChunkId::new(0, 0, 0);
        // Only the root is baked; a deep descendant of it is still covered.
        assert!(baked_cover(ChunkId::new(5, 31, 31), |k| k == root));
        assert!(baked_cover(root, |k| k == root));
        // Nothing baked -> nothing covered, and the walk terminates.
        assert!(!baked_cover(ChunkId::new(8, 200, 200), |_| false));
        // A *sibling* being baked must not count.
        let sib = ChunkId::new(5, 30, 31);
        assert!(!baked_cover(ChunkId::new(5, 31, 31), |k| k == sib));
    }

    #[test]
    fn level_for_zoom_is_monotonic_in_scale_and_respects_the_cap() {
        let mut last = -1;
        for e in 0..14 {
            let l = pyramid_level_for_zoom(2f64.powi(e), 2048.0, 1024.0, Some(8));
            assert!(l >= last, "level went backwards at 2^{e}");
            assert!((0..=8).contains(&l));
            last = l;
        }
        // `null` maxLevel defaults to 6, not 8.
        assert_eq!(pyramid_level_for_zoom(1024.0, 2048.0, 1024.0, None), 6);
    }
}
