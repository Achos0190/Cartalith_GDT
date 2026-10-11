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
