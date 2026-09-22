//! Generic tiling, dirty-tracking and geometry pieces with no opinion on what
//! is stored in them — `LOD_TILING_BASE_SCOPE.md`, then
//! `UNIFIED_TOOL_PLAN.md` milestone A.
//!
//! Built ahead of need on the owner's direction (2026-08-17: "The base should
//! be present before integration"). The trigger turned out to be the DCC tool
//! system, not LOD rendering, and what survived contact with real callers is:
//!
//! - [`DirtyTracker`] — generic per-tile dirty flags plus a monotonic version
//!   counter, with a caller-supplied reason string rather than Cartalith
//!   field names.
//! - [`PassBuffer`] — a non-destructive draft stack of [`Stamp`]s that
//!   previews without writing, commits the whole stack in one ordered pass,
//!   and discards by simply forgetting (see [`pass`]).
//! - [`StageGraph`] — deferred, lazily-evaluated staleness across a DAG of
//!   pipeline stages, each owning its own [`DirtyTracker`] (see
//!   [`staleness`]).
//! - [`pyramid`] — the LOD pyramid's `2^z × 2^z` chunk addressing, which is
//!   what the interactive deep-zoom compositor and the atlas bake use.
//!
//! **Retired 2026-09-22: `TiledField<T>` and `QuadTree<T>`.** Both were built
//! in the first pass and neither ever gained a caller in five weeks. Every
//! consumer that considered them needed *dimensions*, not an owned field:
//! `PassBuffer` holds `width`/`height`/`tile_size` and borrows the data, and
//! `cartalith_godot::lod_bridge` resolves visible chunks with `pyramid`
//! index arithmetic, O(tiles on screen), where building a quadtree first
//! costs an O(field) min/max scan. `TiledField` owned its `Vec<T>`, so using
//! it on a live field meant cloning up to 192 MiB to reach arithmetic that
//! never read the data. The Z3 tier they were shaped for
//! (`LOD_TILING_INTEGRATION_SCOPE.md`, splitting the base raster) is out of
//! scope by that document's own numbers. Recover them from git history if Z3
//! is ever triggered.

pub mod contour;
pub mod geo;
pub mod measure;
pub mod paint;
pub mod pass;
pub mod pyramid;
pub mod region;
pub mod staleness;

pub use contour::contour_polylines;
pub use geo::{geo_xy, id_mask, js_to_fixed, mask_outline_coords, point_in_ring, ring_area, trace_mask_rings};
pub use measure::{
    cell_km, measure, measure_path, point_in_polygon, polygon_area, polygon_centroid,
    polygon_perimeter_km, Measurement,
};
pub use paint::{PaintLayer, PaintStamp};
pub use pass::{CommitSummary, PassBuffer, PassEntry, Stamp};
pub use pyramid::{
    baked_cover, chunk_children, chunk_parent, pyramid_dims, pyramid_level_for_zoom,
    pyramid_tile_bounds, pyramid_tile_count, tiles_in_view, ChunkId, PyramidDims, TilesInView,
};
pub use region::{js_round, norm_region, tile_dims, FloatRegion, TileDims};
pub use staleness::{StageGraph, StageId, Staleness};

use serde::{Deserialize, Serialize};

// ============================================================================
// Region
// ============================================================================

/// An axis-aligned integer rectangle in field-cell coordinates: `[x, x+w)` ×
/// `[y, y+h)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Region {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    /// Whether this region shares any cell with `other`. An empty region
    /// (`w == 0` or `h == 0`) never intersects anything, including itself.
    pub fn intersects(&self, other: &Region) -> bool {
        self.w > 0
            && self.h > 0
            && other.w > 0
            && other.h > 0
            && self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }
}

// ============================================================================
// DirtyTracker
// ============================================================================

/// One tile's dirty/version state.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TileStatus {
    dirty: bool,
    reason: Option<String>,
    version: u64,
}

/// Generic per-tile dirty flags plus a monotonic version counter, indexed by
/// a plain `usize` tile index (`ty * tiles_x + tx`, sized by
/// [`PassBuffer::tile_count`]). No Cartalith-specific field-dependency semantics —
/// `TERRAIN_ARCHITECTURE_RESEARCH.md` §16/17's `HEIGHT_DIRTY`/`BIOME_DIRTY`
/// distinction is a real idea, but the dependency graph it implies has no
/// real caller yet, so this stays a generic caller-supplied reason string
/// rather than a set of Cartalith field names baked into a library crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyTracker {
    tiles: Vec<TileStatus>,
}

impl DirtyTracker {
    /// All `tile_count` tiles start clean, at version 0.
    pub fn new(tile_count: usize) -> Self {
        Self {
            tiles: (0..tile_count)
                .map(|_| TileStatus {
                    dirty: false,
                    reason: None,
                    version: 0,
                })
                .collect(),
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Marks a tile dirty and bumps its version — the version counter
    /// tracks *real changes*, so this is the only thing that increments it.
    pub fn mark_dirty(&mut self, tile_index: usize, reason: impl Into<String>) {
        let tile = &mut self.tiles[tile_index];
        tile.dirty = true;
        tile.reason = Some(reason.into());
        tile.version += 1;
    }

    /// Acknowledges a tile's dirty state has been handled. Does **not**
    /// bump the version — clearing isn't itself a change to the tile's
    /// data, only marking dirty is.
    pub fn clear_dirty(&mut self, tile_index: usize) {
        let tile = &mut self.tiles[tile_index];
        tile.dirty = false;
        tile.reason = None;
    }

    pub fn is_dirty(&self, tile_index: usize) -> bool {
        self.tiles[tile_index].dirty
    }

    pub fn version(&self, tile_index: usize) -> u64 {
        self.tiles[tile_index].version
    }

    pub fn reason(&self, tile_index: usize) -> Option<&str> {
        self.tiles[tile_index].reason.as_deref()
    }

    pub fn dirty_tiles(&self) -> impl Iterator<Item = usize> + '_ {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.dirty)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    // ---- DirtyTracker ----

    #[test]
    fn dirty_tracker_starts_clean_at_version_zero() {
        let tracker = DirtyTracker::new(4);
        for i in 0..4 {
            assert!(!tracker.is_dirty(i));
            assert_eq!(tracker.version(i), 0);
            assert_eq!(tracker.reason(i), None);
        }
    }

    #[test]
    fn dirty_tracker_mark_sets_dirty_reason_and_bumps_version() {
        let mut tracker = DirtyTracker::new(2);
        tracker.mark_dirty(1, "brush");
        assert!(tracker.is_dirty(1));
        assert_eq!(tracker.reason(1), Some("brush"));
        assert_eq!(tracker.version(1), 1);
        assert!(!tracker.is_dirty(0));
        assert_eq!(tracker.version(0), 0);
    }

    #[test]
    fn dirty_tracker_clear_does_not_bump_version() {
        let mut tracker = DirtyTracker::new(1);
        tracker.mark_dirty(0, "edit");
        assert_eq!(tracker.version(0), 1);
        tracker.clear_dirty(0);
        assert!(!tracker.is_dirty(0));
        assert_eq!(tracker.reason(0), None);
        // Version reflects how many real changes happened, not whether
        // they've been acknowledged -- clearing isn't a change.
        assert_eq!(tracker.version(0), 1);
    }

    #[test]
    fn dirty_tracker_repeated_marks_keep_bumping_version() {
        let mut tracker = DirtyTracker::new(1);
        tracker.mark_dirty(0, "a");
        tracker.mark_dirty(0, "b");
        tracker.mark_dirty(0, "c");
        assert_eq!(tracker.version(0), 3);
        assert_eq!(tracker.reason(0), Some("c"));
    }

    #[test]
    fn dirty_tiles_lists_only_dirty_indices() {
        let mut tracker = DirtyTracker::new(5);
        tracker.mark_dirty(1, "x");
        tracker.mark_dirty(3, "y");
        let dirty: Vec<usize> = tracker.dirty_tiles().collect();
        assert_eq!(dirty, vec![1, 3]);
    }

    // ---- Serialization round-trips ----

    #[test]
    fn dirty_tracker_round_trips_through_json() {
        let mut tracker = DirtyTracker::new(3);
        tracker.mark_dirty(1, "reason");
        let json = serde_json::to_string(&tracker).unwrap();
        let back: DirtyTracker = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tile_count(), 3);
        assert!(back.is_dirty(1));
        assert_eq!(back.reason(1), Some("reason"));
        assert_eq!(back.version(1), 1);
    }

    #[test]
    fn region_intersects_is_correct_including_touching_edges() {
        let a = Region::new(0, 0, 4, 4);
        let b = Region::new(4, 0, 4, 4); // touches a's right edge, no overlap
        let c = Region::new(3, 3, 4, 4); // overlaps a's bottom-right cell
        let empty = Region::new(0, 0, 0, 4);
        assert!(!a.intersects(&b));
        assert!(a.intersects(&c));
        assert!(!a.intersects(&empty));
    }
}
