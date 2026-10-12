//! Found-bug sweep 2026-10-11, item R2: `RoadEdge::path` holds **routing-grid**
//! indices (`civ_hierarchical_network_topology` builds edges on a grid
//! `min(gw, RW_CAP)` wide), but the planner (`jp_road_cells`) and the Route
//! tool's router (`tools::civ_road_edge_polylines`) read them as `(i % gw,
//! i / gw)` of the map. Right while the routing grid is 1:1 (maps up to 1024
//! cells wide), wrong above it. These tests pin both halves of the fix:
//!
//! - on a map wider than `RW_CAP`, the credited cells are where the routing
//!   cells really lie (and not where the old reading put them), and the
//!   converted points map back to the same routing cells under the router's
//!   own forward map;
//! - on maps up to `RW_CAP` wide (every golden world), the result is exactly
//!   the old reading, checked against an independent per-point oracle.
//!
//! Every expectation is either a literal worked by hand in the comment beside
//! it or an oracle computed independently of the code under test.

use std::collections::HashMap;

use cartalith_civ::tools;
use cartalith_civ::{JpRoadCell, RoadEdge, RW_CAP};

/// The plain-road row every credited edge carries (`way_class::planner_row(Road)`,
/// the reference's `["Dirt Track","Standard"]`, pri 1), written as a literal.
const EDGE_ROW: JpRoadCell = JpRoadCell { terrain: "Dirt Track", cond: "Standard", pri: 1 };

/// The old reading, written out independently as the oracle for a 1:1 grid:
/// every path cell `(i % gw, i / gw)` dilated 3x3, first writer kept.
fn old_reading(path: &[usize], gw: usize) -> HashMap<(i64, i64), JpRoadCell> {
    let mut m = HashMap::new();
    for &i in path {
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                m.entry((x + dx, y + dy)).or_insert(EDGE_ROW);
            }
        }
    }
    assert!(!m.is_empty(), "the oracle must not be empty");
    m
}

/// Protects: R2 on a map wider than `RW_CAP`. At `gw = 2048` the routing grid is
/// 1024 wide (`sc = 0.5`), so routing cell `(cx, cy)` covers map cells around
/// `(2 cx, 2 cy)`. An edge along routing row 10, `cx = 100..=103`, must credit
/// map row 20 from x = 200 to 206 (each cell dilated by one: x 199..=207, y
/// 19..=21), continuously. The old reading put routing index `10 * 1024 + 100 =
/// 10340` at `(10340 % 2048, 10340 / 2048) = (100, 5)`: those cells must not be
/// credited now.
#[test]
fn a_wide_maps_edge_is_credited_where_the_routing_cells_lie() {
    let gw = 2048;
    assert!(gw > RW_CAP, "premise: the routing grid is downsampled");
    let path: Vec<usize> = (100..=103).map(|cx| 10 * 1024 + cx).collect();
    let edge = RoadEdge { a: 0, b: 1, path };
    let cells = cartalith_civ::jp_road_cells(&[], &[], std::slice::from_ref(&edge), gw);
    for x in 199..=207 {
        for y in 19..=21 {
            assert_eq!(cells.get(&(x, y)), Some(&EDGE_ROW), "({x}, {y}) lies on the edge and must be credited");
        }
    }
    assert_eq!(cells.len(), 9 * 3, "exactly the 9 x 3 band, continuous between the 2-cell-apart points");
    assert!(!cells.contains_key(&(100, 5)) && !cells.contains_key(&(103, 5)), "the old misplaced cells are gone");
}

/// Protects: R2 for the router. On the same wide map the converted polyline is
/// `(200, 20), (202, 20), (204, 20), (206, 20)` (hand-worked: `cx / 0.5`), and
/// every point maps back to its own routing cell under the router's forward map
/// `round(x * sc)` (`_civMarkWayNeighborhood`), which is what makes the router
/// discount the cells the edge was routed through.
#[test]
fn a_wide_maps_edge_polyline_maps_back_to_its_own_routing_cells() {
    let gw = 2048;
    let path: Vec<usize> = (100..=103).map(|cx| 10 * 1024 + cx).collect();
    let edge = RoadEdge { a: 0, b: 1, path: path.clone() };
    let polys = tools::civ_road_edge_polylines(&[&edge], gw);
    assert_eq!(polys, vec![vec![(200.0, 20.0), (202.0, 20.0), (204.0, 20.0), (206.0, 20.0)]]);
    let sc = 1024.0 / 2048.0;
    for (&(x, y), &i) in polys[0].iter().zip(&path) {
        let back = ((y * sc).round() as usize) * 1024 + (x * sc).round() as usize;
        assert_eq!(back, i, "({x}, {y}) maps back to routing cell {i}");
    }
    // A non-integral scale: gw = 1500 gives sc = 1024 / 1500 = 0.6827, where
    // rounding (not truncating) `cx / sc` is what makes the round trip exact
    // (cx = 2: 2 / sc = 2.93 -> 3, and round(3 * sc) = round(2.05) = 2; a floor
    // would give 2 -> round(1.37) = 1).
    let (gw, sc) = (1500usize, 1024.0 / 1500.0);
    let path: Vec<usize> = (0..=40).map(|cx| 3 * 1024 + cx).collect();
    let edge = RoadEdge { a: 0, b: 1, path: path.clone() };
    let polys = tools::civ_road_edge_polylines(&[&edge], gw);
    assert_eq!(polys[0].len(), path.len());
    for (&(x, y), &i) in polys[0].iter().zip(&path) {
        let back = ((y * sc).round() as usize) * 1024 + (x * sc).round() as usize;
        assert_eq!(back, i, "gw 1500: ({x}, {y}) maps back to routing cell {i}");
    }
}

/// Protects: R2's no-golden-moves proof for every map up to `RW_CAP` wide. A
/// path with straight, diagonal and backtracking 8-adjacent steps plus one
/// non-adjacent jump (a break) credits exactly the old per-point reading at
/// `gw` 40, 384 and 1024, keys and rows alike; and the router's polyline is the
/// old `(i % gw, i / gw)` point for point.
#[test]
fn up_to_rw_cap_the_new_reading_is_the_old_one_exactly() {
    for gw in [40usize, 384, 1024] {
        let at = |x: usize, y: usize| y * gw + x;
        // Right, right, diagonal, down, back left, then a jump of 9 cells.
        let path = vec![at(3, 2), at(4, 2), at(5, 2), at(6, 3), at(6, 4), at(5, 4), at(14, 4), at(15, 5)];
        let edge = RoadEdge { a: 0, b: 1, path: path.clone() };
        let cells = cartalith_civ::jp_road_cells(&[], &[], std::slice::from_ref(&edge), gw);
        assert_eq!(cells, old_reading(&path, gw), "gw {gw}");
        let polys = tools::civ_road_edge_polylines(&[&edge], gw);
        let old: Vec<(f64, f64)> = path.iter().map(|&i| ((i % gw) as f64, (i / gw) as f64)).collect();
        assert_eq!(polys, vec![old], "gw {gw}");
    }
}

// ---- Saves written under the old 384 cap (OUTSTANDING_WORK.md row "Saves
// written while `RW_CAP` was 384", 2026-10-12) ----------------------------------
//
// A project saved between 2026-09-24 and 2026-09-28 of a map wider than 384
// holds `RoadEdge::path` indices on a 384-wide routing grid, and says nothing
// about it. `road_edges_infer_grid_w` decides which grid a keyless list was laid
// on; `road_edge_regrid` re-expresses it on today's. Every expected value below
// is worked by hand beside it.

use cartalith_civ::{road_edge_regrid, road_edges_infer_grid_w, RW_CAP_BEFORE_RULING_BT};

/// Protects: the regrid of a 384-grid path onto today's grid at gw = 768
/// (today `rw = 768`, `sc = 1`; then `sc = 0.5`, `rh = round(500 x 0.5) = 250`).
/// Old cells (10, 5), (11, 5), (12, 6) go to map cells (20, 10), (22, 10),
/// (24, 12): each old 8-adjacent step becomes a straight run, so the new path
/// is (20,10) (21,10) (22,10) (23,11) (24,12) -- continuous, which is what keeps
/// the road credited between the old cell centres. A later non-adjacent old
/// step (12,6) -> (20,6) stays a jump: (40, 12) alone.
#[test]
fn a_384_grid_path_is_re_expressed_on_todays_grid_continuously() {
    let (gw, gh) = (768usize, 500usize);
    assert_eq!(RW_CAP_BEFORE_RULING_BT, 384, "literal: the cap before Ruling BT");
    let old = |x: usize, y: usize| y * 384 + x;
    let new = |x: usize, y: usize| y * 768 + x;
    let got = road_edge_regrid(&[old(10, 5), old(11, 5), old(12, 6), old(20, 6)], 384, gw, gh).expect("on the old grid");
    assert_eq!(got, vec![new(20, 10), new(21, 10), new(22, 10), new(23, 11), new(24, 12), new(40, 12)]);
    // A cell off the old grid (rows 0..250) refuses the edge, never guesses.
    assert_eq!(road_edge_regrid(&[old(0, 250)], 384, gw, gh), None);
}

/// Protects: identity by control flow for a path already on today's grid --
/// every save written since Ruling BT reopens byte for byte -- including a
/// path a regrid would have changed (a jump and a repeat).
#[test]
fn a_path_on_todays_grid_is_returned_unchanged() {
    for (gw, gh) in [(300usize, 200usize), (768, 500), (2048, 1311)] {
        let rw = gw.min(RW_CAP);
        let path = vec![5 * rw + 3, 5 * rw + 3, 9 * rw + 40];
        assert_eq!(road_edge_regrid(&path, rw, gw, gh), Some(path.clone()), "gw {gw}");
    }
}

/// Protects: the inference. Two settlements at map (100, 100) and (140, 120) on
/// a 768 x 500 map; an edge from b back to a. Laid on the 384 grid its ends are
/// routing cells (70, 60) and (50, 50) -- indices 23 110 and 19 250; read on
/// today's 768 grid those same indices are (70, 30) and (50, 25), far from both
/// settlements, so the 384 grid wins. The same edge laid on today's grid (ends
/// (140, 120) and (100, 100)) is judged today's. A map no wider than 384 has one
/// candidate and decides nothing.
#[test]
fn a_keyless_edge_list_is_judged_by_where_its_ends_land() {
    let (gw, gh) = (768usize, 500usize);
    let places = [(100usize, 100usize), (140, 120)];
    let old = RoadEdge { a: 0, b: 1, path: vec![60 * 384 + 70, 55 * 384 + 60, 50 * 384 + 50] };
    assert_eq!((old.path[0], old.path[2]), (23_110, 19_250), "the hand-worked indices");
    assert_eq!(road_edges_infer_grid_w(std::slice::from_ref(&old), &places, gw, gh), (384, true));
    let new = RoadEdge { a: 0, b: 1, path: vec![120 * 768 + 140, 110 * 768 + 120, 100 * 768 + 100] };
    assert_eq!(road_edges_infer_grid_w(std::slice::from_ref(&new), &places, gw, gh), (768, true));
    // No usable evidence (endpoints out of range): today's, undecided.
    let stale = RoadEdge { a: 7, b: 9, path: old.path.clone() };
    assert_eq!(road_edges_infer_grid_w(&[stale], &places, gw, gh), (768, false));
    // Up to 384 wide the two grids are one grid.
    assert_eq!(road_edges_infer_grid_w(&[old], &places, 384, 250), (384, false));
}

/// Protects: a candidate that puts a cell off its own grid is ruled out. On a
/// 2048 x 1311 map the 384 grid has 384 x 246 = 94 464 cells; an index above
/// that can only be today's 1024 grid, even with no settlements to judge by.
#[test]
fn an_index_past_the_old_grid_rules_the_old_grid_out() {
    let e = RoadEdge { a: 0, b: 0, path: vec![200 * 1024 + 10] };
    assert!(e.path[0] >= 384 * 246, "premise: off the 384 grid");
    assert_eq!(road_edges_infer_grid_w(&[e], &[], 2048, 1311), (1024, true));
}
