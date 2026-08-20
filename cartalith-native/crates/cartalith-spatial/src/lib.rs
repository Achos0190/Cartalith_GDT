//! Standalone tiling/spatial-index data structures — `LOD_TILING_BASE_SCOPE.md`.
//!
//! Built ahead of any real need, on the owner's own explicit direction
//! (2026-08-17): "LOD and zoom etc might be out of scope for the base, but
//! they're still goals in this project. The base should be present before
//! integration." **Nothing in this crate is used by any other crate in the
//! workspace.** `generate_terrain`/`compute_civilisation`/rendering all stay
//! exactly as they were — one full-resolution flat array per generation,
//! unchanged. This crate exists so that whenever Phase 3 (3D, `ROADMAP.md`)
//! or a real large-world need actually starts LOD/tiling integration, it
//! starts from a tested foundation instead of a green field.
//!
//! Three independent pieces, matching `LOD_TILING_BASE_SCOPE.md`'s own
//! numbered scope:
//!
//! 1. [`TiledField`] — zero-copy tile/region/row/column views over a flat,
//!    row-major `Vec<T>` (the same Structure-of-Arrays layout
//!    `WorldState`/`CivData` already use).
//! 2. [`QuadTree`] — a packed (`Vec<Node<T>>`, integer child indices, no
//!    `Box`/pointers) spatial index with generic per-node aggregate
//!    metadata (bounds, min/max, a caller-defined flag bitmask), built from
//!    a flat field.
//! 3. [`DirtyTracker`] — generic per-tile dirty flags plus a monotonic
//!    version counter, with no Cartalith-specific field-dependency
//!    semantics baked in (no real caller exists yet to say what those
//!    should be).
//!
//! None of these three know about height, climate, biome, or any other
//! Cartalith-specific concept — that's deliberate: a data-structure crate
//! with no opinion on what's stored in it is what stays cheap to leave
//! unintegrated for however long that turns out to be.
//!
//! ## The trigger arrived (`UNIFIED_TOOL_PLAN.md` milestone A, 2026-08-18)
//!
//! "Whenever a real large-world need actually triggers integration" turned
//! out to be the DCC tool system, not LOD rendering. Two more pieces live
//! here now, built on the three above and holding to the same
//! stay-generic rule:
//!
//! 4. [`PassBuffer`] — a non-destructive draft stack of [`Stamp`]s that
//!    previews without writing, commits the whole stack in one ordered pass,
//!    and discards by simply forgetting (see [`pass`]).
//! 5. [`StageGraph`] — deferred, lazily-evaluated staleness across a DAG of
//!    pipeline stages, each owning its own [`DirtyTracker`] (see
//!    [`staleness`]).
//!
//! Both stayed generic, and neither needed [`DirtyTracker`] extended: its
//! `mark_dirty` already is the "my data changed here, here's why, bump the
//! version" primitive both editing and recomputation need.

pub mod geo;
pub mod measure;
pub mod paint;
pub mod pass;
pub mod region;
pub mod staleness;

pub use geo::{geo_xy, id_mask, js_to_fixed, mask_outline_coords, point_in_ring, ring_area, trace_mask_rings};
pub use measure::{cell_km, measure, measure_path, Measurement};
pub use paint::{PaintLayer, PaintStamp};
pub use pass::{CommitSummary, PassBuffer, PassEntry, Stamp};
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
// TiledField<T>
// ============================================================================

/// A flat, row-major `Vec<T>` (`data[y * width + x]`) with tile/region/row/
/// column *views* layered on top — no data is ever duplicated by taking a
/// view; every view indexes into the same backing storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiledField<T> {
    width: usize,
    height: usize,
    tile_size: usize,
    data: Vec<T>,
}

impl<T> TiledField<T> {
    /// `data` must be exactly `width * height` elements, row-major — the
    /// same layout every flat Cartalith field already uses, so an existing
    /// `Vec<f32>` can be wrapped directly with no conversion.
    ///
    /// `tile_size` is a constructor parameter, not a crate constant: there
    /// is no real workload exercising this crate yet to benchmark 64 vs.
    /// 128 vs. 256 against (`TERRAIN_ARCHITECTURE_RESEARCH.md` §31 flags
    /// exactly this — pick when there's a real caller, not now).
    pub fn new(width: usize, height: usize, tile_size: usize, data: Vec<T>) -> Self {
        assert_eq!(
            data.len(),
            width * height,
            "data length must equal width * height"
        );
        assert!(tile_size > 0, "tile_size must be positive");
        Self {
            width,
            height,
            tile_size,
            data,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_size(&self) -> usize {
        self.tile_size
    }

    /// Tiles needed to cover `width`, rounding up — the last column is
    /// narrower than `tile_size` whenever `width` isn't an exact multiple
    /// of it.
    pub fn tiles_x(&self) -> usize {
        self.width.div_ceil(self.tile_size)
    }

    /// Tiles needed to cover `height`, rounding up — same edge case as
    /// [`Self::tiles_x`], along the other axis.
    pub fn tiles_y(&self) -> usize {
        self.height.div_ceil(self.tile_size)
    }

    pub fn whole(&self) -> &[T] {
        &self.data
    }

    pub fn whole_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.data[y * self.width + x]
    }

    /// A full row is contiguous in row-major storage, so this is a real
    /// zero-copy slice, not a view struct.
    pub fn row(&self, y: usize) -> &[T] {
        let start = y * self.width;
        &self.data[start..start + self.width]
    }

    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        let start = y * self.width;
        &mut self.data[start..start + self.width]
    }

    /// A column is *not* contiguous (stride = `width`), so unlike
    /// [`Self::row`] this can't be a `&[T]` slice — a lazy, zero-allocation
    /// iterator is the zero-copy equivalent.
    pub fn column(&self, x: usize) -> impl Iterator<Item = &T> + '_ {
        (0..self.height).map(move |y| &self.data[y * self.width + x])
    }

    /// The tile's bounds, clipped to the field's real dimensions. Edge
    /// tiles are narrower/shorter than `tile_size` whenever the field
    /// dimensions aren't an exact multiple of it — this is the one place
    /// that edge case has to be handled, every view built from it inherits
    /// the correct (possibly smaller) size automatically.
    pub fn tile_bounds(&self, tx: usize, ty: usize) -> Region {
        let x = tx * self.tile_size;
        let y = ty * self.tile_size;
        let w = self.tile_size.min(self.width.saturating_sub(x));
        let h = self.tile_size.min(self.height.saturating_sub(y));
        Region::new(x, y, w, h)
    }

    pub fn region(&self, bounds: Region) -> RegionView<'_, T> {
        RegionView {
            data: &self.data,
            field_width: self.width,
            bounds,
        }
    }

    pub fn region_mut(&mut self, bounds: Region) -> RegionViewMut<'_, T> {
        RegionViewMut {
            data: &mut self.data,
            field_width: self.width,
            bounds,
        }
    }

    pub fn tile(&self, tx: usize, ty: usize) -> RegionView<'_, T> {
        self.region(self.tile_bounds(tx, ty))
    }

    pub fn tile_mut(&mut self, tx: usize, ty: usize) -> RegionViewMut<'_, T> {
        let bounds = self.tile_bounds(tx, ty);
        self.region_mut(bounds)
    }
}

/// A read-only, zero-copy window onto a [`TiledField`]'s backing storage.
/// Holds a reference to the *whole* backing slice plus the field's real
/// width (needed to convert local `(x, y)` coordinates into the right
/// backing-array index) and this view's own bounds — no cells are copied
/// out to construct it.
pub struct RegionView<'a, T> {
    data: &'a [T],
    field_width: usize,
    bounds: Region,
}

impl<T> RegionView<'_, T> {
    pub fn bounds(&self) -> Region {
        self.bounds
    }

    pub fn width(&self) -> usize {
        self.bounds.w
    }

    pub fn height(&self) -> usize {
        self.bounds.h
    }

    pub fn get(&self, local_x: usize, local_y: usize) -> &T {
        let gx = self.bounds.x + local_x;
        let gy = self.bounds.y + local_y;
        &self.data[gy * self.field_width + gx]
    }

    pub fn row(&self, local_y: usize) -> &[T] {
        let gy = self.bounds.y + local_y;
        let start = gy * self.field_width + self.bounds.x;
        &self.data[start..start + self.bounds.w]
    }

    /// `(local_x, local_y, &value)` for every cell in the view, in
    /// row-major order.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &T)> + '_ {
        (0..self.bounds.h)
            .flat_map(move |ly| (0..self.bounds.w).map(move |lx| (lx, ly, self.get(lx, ly))))
    }
}

/// The mutable counterpart to [`RegionView`]. No `iter_mut` — the
/// integration this crate is a base for doesn't exist yet, and
/// `get_mut`/`row_mut` already cover every write pattern this pass actually
/// needs (`LOD_TILING_BASE_SCOPE.md`'s own scope: prove writes land in the
/// right cells, nothing more).
pub struct RegionViewMut<'a, T> {
    data: &'a mut [T],
    field_width: usize,
    bounds: Region,
}

impl<T> RegionViewMut<'_, T> {
    pub fn bounds(&self) -> Region {
        self.bounds
    }

    pub fn width(&self) -> usize {
        self.bounds.w
    }

    pub fn height(&self) -> usize {
        self.bounds.h
    }

    pub fn get(&self, local_x: usize, local_y: usize) -> &T {
        let gx = self.bounds.x + local_x;
        let gy = self.bounds.y + local_y;
        &self.data[gy * self.field_width + gx]
    }

    pub fn get_mut(&mut self, local_x: usize, local_y: usize) -> &mut T {
        let gx = self.bounds.x + local_x;
        let gy = self.bounds.y + local_y;
        &mut self.data[gy * self.field_width + gx]
    }

    pub fn row_mut(&mut self, local_y: usize) -> &mut [T] {
        let gy = self.bounds.y + local_y;
        let start = gy * self.field_width + self.bounds.x;
        &mut self.data[start..start + self.bounds.w]
    }
}

// ============================================================================
// QuadTree<T>
// ============================================================================

/// Sentinel for "no child here" in [`Node::children`] — a leaf has all four
/// slots set to this.
pub const NO_CHILD: usize = usize::MAX;

/// One quadtree node: its region, the min/max of whatever `T` is being
/// indexed over that region, a caller-defined flag bitmask this crate
/// assigns no meaning to at all (`TERRAIN_ARCHITECTURE_RESEARCH.md` §14/15's
/// "contains water"/"contains river" idea — deliberately not baked in by
/// name, since no real caller exists yet to say which bits mean what), and
/// up to four children by index into the owning [`QuadTree`]'s node vector
/// (never `Box<Node>` — see `LOD_TILING_BASE_SCOPE.md`'s own "packed, not
/// pointer-heavy" requirement).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    pub bounds: Region,
    pub min: T,
    pub max: T,
    pub flags: u32,
    pub children: [usize; 4],
}

impl<T> Node<T> {
    pub fn is_leaf(&self) -> bool {
        self.children == [NO_CHILD; 4]
    }
}

/// A packed quadtree: one `Vec<Node<T>>`, children referenced by index —
/// contiguous, cache-friendly, and trivially [`Serialize`]/[`Deserialize`],
/// unlike a pointer-chasing `Box<Node>` tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadTree<T> {
    nodes: Vec<Node<T>>,
}

impl<T: Copy + PartialOrd> QuadTree<T> {
    /// Builds a quadtree over `data` (row-major, `width * height` elements)
    /// by recursively splitting into up to four quadrants until a node's
    /// bounds are `<= leaf_max` on both axes. `flags_of` computes each
    /// node's own caller-defined flag bitmask from its bounds — this crate
    /// has no opinion on what any bit means.
    ///
    /// Handles non-power-of-two and odd dimensions directly: a quadrant with
    /// zero width or height after splitting is simply omitted rather than
    /// built as a degenerate empty node, so `Node::children` may have fewer
    /// than four real children even for a non-leaf.
    pub fn build(
        data: &[T],
        width: usize,
        height: usize,
        leaf_max: usize,
        flags_of: &dyn Fn(Region) -> u32,
    ) -> Self {
        assert_eq!(data.len(), width * height, "data length must equal width * height");
        assert!(leaf_max > 0, "leaf_max must be positive");
        let mut nodes = Vec::new();
        if width > 0 && height > 0 {
            Self::build_node(data, width, Region::new(0, 0, width, height), leaf_max, flags_of, &mut nodes);
        }
        Self { nodes }
    }

    fn region_min_max(data: &[T], field_width: usize, bounds: Region) -> (T, T) {
        let mut cells = (0..bounds.h).flat_map(|ly| {
            let gy = bounds.y + ly;
            (0..bounds.w).map(move |lx| data[gy * field_width + bounds.x + lx])
        });
        let first = cells.next().expect("region passed to region_min_max must be non-empty");
        let mut min = first;
        let mut max = first;
        for v in cells {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        (min, max)
    }

    fn build_node(
        data: &[T],
        field_width: usize,
        bounds: Region,
        leaf_max: usize,
        flags_of: &dyn Fn(Region) -> u32,
        nodes: &mut Vec<Node<T>>,
    ) -> usize {
        let (min, max) = Self::region_min_max(data, field_width, bounds);
        let idx = nodes.len();
        nodes.push(Node {
            bounds,
            min,
            max,
            flags: flags_of(bounds),
            children: [NO_CHILD; 4],
        });

        if bounds.w <= leaf_max && bounds.h <= leaf_max {
            return idx;
        }

        let left_w = bounds.w.div_ceil(2);
        let right_w = bounds.w - left_w;
        let top_h = bounds.h.div_ceil(2);
        let bottom_h = bounds.h - top_h;

        let mut quads = Vec::with_capacity(4);
        quads.push(Region::new(bounds.x, bounds.y, left_w, top_h));
        if right_w > 0 {
            quads.push(Region::new(bounds.x + left_w, bounds.y, right_w, top_h));
        }
        if bottom_h > 0 {
            quads.push(Region::new(bounds.x, bounds.y + top_h, left_w, bottom_h));
        }
        if right_w > 0 && bottom_h > 0 {
            quads.push(Region::new(bounds.x + left_w, bounds.y + top_h, right_w, bottom_h));
        }

        let mut children = [NO_CHILD; 4];
        for (slot, quad) in quads.into_iter().enumerate() {
            children[slot] = Self::build_node(data, field_width, quad, leaf_max, flags_of, nodes);
        }
        nodes[idx].children = children;
        idx
    }
}

impl<T> QuadTree<T> {
    pub fn root_index(&self) -> usize {
        0
    }

    pub fn node(&self, idx: usize) -> &Node<T> {
        &self.nodes[idx]
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Leaf-node indices whose bounds intersect `query`, found by rejecting
    /// whole subtrees whose bounds don't intersect rather than visiting
    /// every cell.
    pub fn query_region(&self, query: Region) -> Vec<usize> {
        self.query_region_counted(query).0
    }

    /// Same as [`Self::query_region`], but also returns how many nodes were
    /// actually visited during the traversal — the number that matters for
    /// proving rejection works: it should be far smaller than [`Self::len`]
    /// for a query that only overlaps a small part of the tree.
    pub fn query_region_counted(&self, query: Region) -> (Vec<usize>, usize) {
        let mut out = Vec::new();
        let mut visited = 0usize;
        if !self.nodes.is_empty() {
            self.query_recursive(self.root_index(), query, &mut out, &mut visited);
        }
        (out, visited)
    }

    fn query_recursive(&self, idx: usize, query: Region, out: &mut Vec<usize>, visited: &mut usize) {
        *visited += 1;
        let node = &self.nodes[idx];
        if !node.bounds.intersects(&query) {
            return;
        }
        if node.is_leaf() {
            out.push(idx);
            return;
        }
        for &child in &node.children {
            if child != NO_CHILD {
                self.query_recursive(child, query, out, visited);
            }
        }
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
/// a plain `usize` tile index (e.g. `ty * tiles_x + tx` from a
/// [`TiledField`]). No Cartalith-specific field-dependency semantics —
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

    // ---- TiledField ----

    #[test]
    fn tiled_field_get_matches_row_major_layout() {
        let data: Vec<i32> = (0..12).collect(); // 4 wide, 3 tall
        let field = TiledField::new(4, 3, 2, data);
        assert_eq!(*field.get(0, 0), 0);
        assert_eq!(*field.get(3, 0), 3);
        assert_eq!(*field.get(0, 2), 8);
        assert_eq!(*field.get(3, 2), 11);
    }

    #[test]
    fn tiled_field_row_is_contiguous_and_correct() {
        let data: Vec<i32> = (0..12).collect();
        let field = TiledField::new(4, 3, 2, data);
        assert_eq!(field.row(1), &[4, 5, 6, 7]);
    }

    #[test]
    fn tiled_field_column_iterates_with_correct_stride() {
        let data: Vec<i32> = (0..12).collect();
        let field = TiledField::new(4, 3, 2, data);
        let col: Vec<i32> = field.column(1).copied().collect();
        assert_eq!(col, vec![1, 5, 9]);
    }

    #[test]
    fn tile_bounds_off_by_one_edge_tiles_are_clipped() {
        // 10x10 field, tile_size 3 -> tiles cover 3,3,3,1 along each axis.
        let data = vec![0i32; 100];
        let field = TiledField::new(10, 10, 3, data);
        assert_eq!(field.tiles_x(), 4);
        assert_eq!(field.tiles_y(), 4);

        // A full interior tile.
        assert_eq!(field.tile_bounds(0, 0), Region::new(0, 0, 3, 3));
        // The last tile along each axis is clipped to width 1, not 3.
        assert_eq!(field.tile_bounds(3, 0), Region::new(9, 0, 1, 3));
        assert_eq!(field.tile_bounds(0, 3), Region::new(0, 9, 3, 1));
        assert_eq!(field.tile_bounds(3, 3), Region::new(9, 9, 1, 1));
    }

    #[test]
    fn tile_bounds_exact_multiple_has_no_clipped_edge() {
        // 9x9 field, tile_size 3 -> exactly 3 tiles per axis, no clipping.
        let data = vec![0i32; 81];
        let field = TiledField::new(9, 9, 3, data);
        assert_eq!(field.tiles_x(), 3);
        assert_eq!(field.tiles_y(), 3);
        assert_eq!(field.tile_bounds(2, 2), Region::new(6, 6, 3, 3));
    }

    #[test]
    fn tile_view_reads_correct_cells() {
        let data: Vec<i32> = (0..100).collect(); // 10 wide
        let field = TiledField::new(10, 10, 3, data);
        let view = field.tile(3, 0); // Region { x: 9, y: 0, w: 1, h: 3 }
        assert_eq!(view.width(), 1);
        assert_eq!(view.height(), 3);
        assert_eq!(*view.get(0, 0), 9); // row 0, col 9
        assert_eq!(*view.get(0, 1), 19); // row 1, col 9
        assert_eq!(*view.get(0, 2), 29); // row 2, col 9
    }

    #[test]
    fn mutable_view_writes_land_in_the_backing_array() {
        let data = vec![0i32; 100];
        let mut field = TiledField::new(10, 10, 3, data);
        {
            let mut view = field.tile_mut(1, 1); // Region { x: 3, y: 3, w: 3, h: 3 }
            *view.get_mut(0, 0) = 42;
            view.row_mut(1)[2] = 99; // local (2, 1) -> global (5, 4)
        }
        assert_eq!(*field.get(3, 3), 42);
        assert_eq!(*field.get(5, 4), 99);
        // Nothing outside the written cells changed.
        assert_eq!(*field.get(0, 0), 0);
        assert_eq!(*field.get(9, 9), 0);
    }

    #[test]
    fn region_view_iter_covers_every_cell_once() {
        let data: Vec<i32> = (0..16).collect(); // 4x4
        let field = TiledField::new(4, 4, 2, data);
        let view = field.region(Region::new(1, 1, 2, 2));
        let cells: Vec<(usize, usize, i32)> = view.iter().map(|(x, y, v)| (x, y, *v)).collect();
        assert_eq!(
            cells,
            vec![(0, 0, 5), (1, 0, 6), (0, 1, 9), (1, 1, 10)]
        );
    }

    // ---- QuadTree ----

    #[test]
    fn quadtree_root_min_max_matches_full_data_range() {
        let width = 8;
        let height = 8;
        let data: Vec<i32> = (0..64).collect();
        let tree = QuadTree::build(&data, width, height, 2, &|_| 0);
        let root = tree.node(tree.root_index());
        assert_eq!(root.min, 0);
        assert_eq!(root.max, 63);
        assert_eq!(root.bounds, Region::new(0, 0, 8, 8));
    }

    #[test]
    fn quadtree_leaf_min_max_matches_its_own_region_only() {
        let width = 8;
        let data: Vec<i32> = (0..64).collect();
        let tree = QuadTree::build(&data, width, 8, 2, &|_| 0);
        // Find a leaf covering the top-left 2x2 corner (cells 0,1,8,9).
        let (candidates, _) = tree.query_region_counted(Region::new(0, 0, 1, 1));
        assert_eq!(candidates.len(), 1);
        let leaf = tree.node(candidates[0]);
        assert!(leaf.is_leaf());
        assert_eq!(leaf.bounds, Region::new(0, 0, 2, 2));
        assert_eq!(leaf.min, 0);
        assert_eq!(leaf.max, 9); // cells 0, 1, 8, 9
    }

    #[test]
    fn quadtree_handles_non_power_of_two_dimensions() {
        // 5x3: not a power of two on either axis, must not panic and must
        // still cover every cell in some leaf.
        let width = 5;
        let height = 3;
        let data: Vec<i32> = (0..15).collect();
        let tree = QuadTree::build(&data, width, height, 1, &|_| 0);
        let (leaves, _) = tree.query_region_counted(Region::new(0, 0, width, height));
        let mut covered = [false; 15];
        for &idx in &leaves {
            let b = tree.node(idx).bounds;
            for ly in 0..b.h {
                for lx in 0..b.w {
                    covered[(b.y + ly) * width + (b.x + lx)] = true;
                }
            }
        }
        assert!(covered.iter().all(|&c| c), "every cell must be covered by exactly one leaf's bounds");
    }

    #[test]
    fn quadtree_query_rejects_subtrees_without_visiting_them() {
        // A reasonably deep tree: 64x64, leaf_max 4 -> several levels.
        let width = 64;
        let height = 64;
        let data = vec![0i32; width * height];
        let tree = QuadTree::build(&data, width, height, 4, &|_| 0);
        let total_nodes = tree.len();
        assert!(total_nodes > 100, "test assumes a real multi-level tree");

        // Query a tiny region in one corner -- most of the tree's other
        // three quadrants (and everything under them) must be rejected at
        // their own root without ever being visited.
        let (leaves, visited) = tree.query_region_counted(Region::new(0, 0, 1, 1));
        assert_eq!(leaves.len(), 1, "a 1x1 query should land in exactly one leaf");
        // A full traversal would visit every node; rejection means far
        // fewer nodes are actually visited than exist in the tree.
        assert!(
            visited < total_nodes / 4,
            "expected rejection to skip most of the tree: visited={visited}, total={total_nodes}"
        );
    }

    #[test]
    fn quadtree_predicate_search_within_region_uses_rejection() {
        // Cells hold their own flat index as the value; "matches" means
        // value == 500 exactly. Confirm the match is found by descending
        // only into leaves whose bounds overlap a *partial* query region
        // (not the whole field, which would reject nothing), using the
        // node bounds from query_region against the real source data.
        let width = 32;
        let height = 32;
        let data: Vec<i32> = (0..1024).collect();
        let tree = QuadTree::build(&data, width, height, 4, &|_| 0);

        // Cell 500 is at (x=20, y=15). Query a small region around it,
        // not the whole field -- most of the tree must be rejected.
        let query = Region::new(16, 12, 8, 8);
        let (leaves, visited) = tree.query_region_counted(query);
        assert!(
            visited < tree.len(),
            "a partial-region query must reject most of the tree, not visit every node"
        );

        let mut found = None;
        for &idx in &leaves {
            let b = tree.node(idx).bounds;
            if tree.node(idx).min > 500 || tree.node(idx).max < 500 {
                continue; // this leaf's own aggregate proves 500 can't be inside it
            }
            for ly in 0..b.h {
                for lx in 0..b.w {
                    let gx = b.x + lx;
                    let gy = b.y + ly;
                    if data[gy * width + gx] == 500 {
                        found = Some((gx, gy));
                    }
                }
            }
        }
        assert_eq!(found, Some((500 % width, 500 / width)));
    }

    #[test]
    fn quadtree_flags_of_is_called_per_node_with_that_nodes_bounds() {
        let width = 4;
        let height = 4;
        let data: Vec<i32> = (0..16).collect();
        // Flag every node whose bounds contain cell (3, 3) (value 15).
        let tree = QuadTree::build(&data, width, height, 2, &|b| {
            if b.x <= 3 && 3 < b.x + b.w && b.y <= 3 && 3 < b.y + b.h {
                1
            } else {
                0
            }
        });
        assert_eq!(tree.node(tree.root_index()).flags, 1);
        let (leaves, _) = tree.query_region_counted(Region::new(3, 3, 1, 1));
        assert_eq!(leaves.len(), 1);
        assert_eq!(tree.node(leaves[0]).flags, 1);
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
    fn tiled_field_round_trips_through_json() {
        let data: Vec<f32> = (0..12).map(|i| i as f32 * 0.5).collect();
        let field = TiledField::new(4, 3, 2, data);
        let json = serde_json::to_string(&field).unwrap();
        let back: TiledField<f32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.width(), field.width());
        assert_eq!(back.height(), field.height());
        assert_eq!(back.tile_size(), field.tile_size());
        assert_eq!(back.whole(), field.whole());
    }

    #[test]
    fn quadtree_round_trips_through_json() {
        let data: Vec<i32> = (0..64).collect();
        let tree = QuadTree::build(&data, 8, 8, 2, &|_| 7);
        let json = serde_json::to_string(&tree).unwrap();
        let back: QuadTree<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), tree.len());
        assert_eq!(back.node(back.root_index()).min, tree.node(tree.root_index()).min);
        assert_eq!(back.node(back.root_index()).max, tree.node(tree.root_index()).max);
        assert_eq!(back.node(back.root_index()).flags, 7);
    }

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
