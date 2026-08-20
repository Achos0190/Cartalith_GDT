//! Golden-parity tests for the road network algorithm --
//! `PHASE2_SCOPE.md` milestone 11: `buildTravelCost` (reference HTML line
//! 3257), `roadDijkstra` (line 3275), `buildRoadNetwork` (line 3316).
//!
//! Generated from a Node `vm` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` block 1 only (lines 2084-14556 --
//! `buildTravelCost`/`roadDijkstra`/`buildRoadNetwork`/`generate()` all
//! live in block 1; no civ-block code is needed for this milestone,
//! unlike milestone 9's harness). The reference's own trailing "v0.67:
//! boot" auto-generate call (`GW=state.resW; GH=gridH(GW); allocate();
//! withBusy('generating…',generate);`) was stripped from the extracted
//! source so the harness controls seed/resolution/generation explicitly
//! instead of racing an auto-run at load time -- the harness would
//! otherwise auto-generate with wrong parameters before the driver script
//! gets a chance to set `state.tect.seed`/`GW`/`GH`.
//!
//! Reuses milestone 9's own already-verified settlement positions
//! (`golden_parity_settlement_naming.rs`'s expected `(x,y)` pairs) as the
//! `places` input directly, rather than re-deriving the civ pipeline in
//! JS -- `SettlementPlacement.faction`/`.kind`/etc. are irrelevant here
//! (`build_road_network` only reads `.x`/`.y`, matching the reference's
//! own `buildRoadNetwork`, which only reads `places[s].x`/`.y`).
//!
//! `field[0..5]` cross-checked against `golden_parity_carve.rs`'s already-
//! trusted `expected_field` before trusting the extraction -- matched
//! exactly for both cases (both fixtures use the crate's established
//! `p.climate.w_iters = 12` override, same as every other Phase 2
//! milestone's fixtures).
//!
//! Cost field is continuous `f32` -- checked at this crate's established
//! `1e-4` tolerance (water cells checked for `is_infinite()` rather than
//! numeric equality, since `Infinity` has no meaningful "close to"). Edge
//! topology (`a`/`b` node indices, per-edge cell-index `path`) is
//! discrete -- checked bit-exact.
//!
//! **Real terrain data confirmed the "unreachable place" branch, not a
//! synthetic test**: case0's 3 places produce only ONE MST edge (`a=0,
//! b=2`), not the two a fully-connected 3-node MST would have -- place
//! index 1 sits on a landmass the cost-distance search from place 0 never
//! reaches (an island only reachable by sea in this generated world), so
//! `best[1]` stays `Infinity` and the Prim loop's `bu===Infinity` guard
//! breaks early, exactly as `build_road_network_unreachable_landmass_gets_no_edge`'s
//! synthetic unit test already covers -- this is the same real-data
//! signal `golden_parity_settlement_placement.rs`'s own unreachable-branch
//! note describes finding elsewhere in this crate. case1's 5 places
//! produce exactly 4 edges (fully connected, all mutually reachable).

const F32_TOLERANCE: f32 = 1e-4;

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() < F32_TOLERANCE,
        "{label}: expected {expected}, got {actual} (diff {})",
        (actual - expected).abs()
    );
}

fn places_from_xy(xy: &[(usize, usize)]) -> Vec<cartalith_civ::SettlementPlacement> {
    xy.iter()
        .map(|&(x, y)| cartalith_civ::SettlementPlacement {
            x,
            y,
            suit: 0.0,
            faction: 0,
            capital: false,
            kind: cartalith_civ::SettlementKind::Hamlet,
            coastal: false,
        })
        .collect()
}

#[test]
fn road_network_case_0_region() {
    // case0_region: gw=14 gh=11 seed=24601 world=false.
    // Places from golden_parity_settlement_naming.rs's own case0 (x,y) pairs.
    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let expected_field0_5 = [0.8640562295913696f32, 0.7786418199539185, 0.6850417256355286, 0.6560115814208984, 0.6181289553642273];
    for (i, &e) in expected_field0_5.iter().enumerate() {
        assert_close(ws.field[i], e, &format!("field[{i}]"));
    }

    let cost = cartalith_civ::build_travel_cost(&ws.field, 14, 11, ws.sea_level);
    // Sampled cost[0..9] from the real extraction (water cells are Infinity).
    let expected_cost_sample: [f32; 10] = [
        1.0934662818908691,
        1.4491047859191895,
        1.2081507444381714,
        1.0584183931350708,
        1.4013566970825195,
        1.3861287832260132,
        1.0052038431167603,
        1.0065035820007324,
        1.0748504400253296,
        f32::INFINITY,
    ];
    for (i, &e) in expected_cost_sample.iter().enumerate() {
        if e.is_infinite() {
            assert!(cost[i].is_infinite(), "cost[{i}] expected infinite (water), got {}", cost[i]);
        } else {
            assert_close(cost[i], e, &format!("cost[{i}]"));
        }
    }

    let places = places_from_xy(&[(7, 2), (9, 3), (6, 2)]);
    let edges = cartalith_civ::build_road_network(&places, &cost, 14, 11, false);

    assert_eq!(edges.len(), 1, "case0: expected exactly 1 MST edge (place 1 is unreachable)");
    assert_eq!(edges[0].a, 0);
    assert_eq!(edges[0].b, 2);
    assert_eq!(edges[0].path, vec![34usize, 35usize]);
}

#[test]
fn road_network_case_1_world_wrap() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true.
    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let expected_field0_5 = [0.24780429899692535f32, 0.2490912228822708, 0.678697407245636, 0.7500452399253845, 0.6631426811218262];
    for (i, &e) in expected_field0_5.iter().enumerate() {
        assert_close(ws.field[i], e, &format!("field[{i}]"));
    }

    let cost = cartalith_civ::build_travel_cost(&ws.field, 16, 12, ws.sea_level);
    let expected_cost_sample: [f32; 10] = [
        f32::INFINITY,
        f32::INFINITY,
        4.142317771911621,
        1.0330177545547485,
        1.57987642288208,
        1.2246530055999756,
        1.0427093505859375,
        1.0641155242919922,
        1.251558780670166,
        1.0516183376312256,
    ];
    for (i, &e) in expected_cost_sample.iter().enumerate() {
        if e.is_infinite() {
            assert!(cost[i].is_infinite(), "cost[{i}] expected infinite (water), got {}", cost[i]);
        } else {
            assert_close(cost[i], e, &format!("cost[{i}]"));
        }
    }

    let places = places_from_xy(&[(9, 3), (5, 8), (8, 9), (10, 5), (4, 7)]);
    let edges = cartalith_civ::build_road_network(&places, &cost, 16, 12, true);

    assert_eq!(edges.len(), 4, "case1: expected exactly 4 MST edges (5 mutually-reachable places)");
    let expected: Vec<(usize, usize, Vec<usize>)> = vec![
        (0, 3, vec![90, 74, 57]),
        (0, 4, vec![116, 100, 85, 70, 55, 56, 57]),
        (4, 1, vec![133, 116]),
        (1, 2, vec![152, 167, 150, 133]),
    ];
    for (i, (edge, (ea, eb, epath))) in edges.iter().zip(expected.iter()).enumerate() {
        assert_eq!(edge.a, *ea, "edge {i}: a mismatch");
        assert_eq!(edge.b, *eb, "edge {i}: b mismatch");
        assert_eq!(&edge.path, epath, "edge {i}: path mismatch");
    }
}
