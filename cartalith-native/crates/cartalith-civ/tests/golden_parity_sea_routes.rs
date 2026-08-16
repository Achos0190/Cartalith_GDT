//! Golden-parity tests for sea-lane routing -- `PHASE2_SCOPE.md` milestone
//! 13: `_civMstRoutes(ports, true)` (reference HTML line 21240, `isSea`
//! branch only -- see `civ_sea_routes`'s own doc comment in `src/lib.rs`
//! for why the `isSea=false` land branch and `_civSeaTimeEdgeCost`
//! wind/current-costed routing are both out of scope).
//!
//! Node `vm` harness: fresh per this project's established practice (not
//! checked in). Blocks #1 (2084-14556) + #2 (14563-26720), same ranges
//! `golden_parity_road_consolidation.rs` already established. `generate()`
//! is `async` -- MUST be awaited (a bare unawaited call left `field` at
//! its default-zero fill and `currentWaterBodies()` reporting 100% ocean,
//! a real harness bug caught by cross-checking `field[0]` and the
//! land/ocean/lake cell counts against already-trusted fixtures before
//! trusting extraction, not assumed). `state.tect.seed` (not the dead
//! `state.seed`).
//!
//! Fixtures: the SAME (gw/gh/seed/world, x/y/faction/name/pop) triples as
//! `golden_parity_road_consolidation.rs`'s case0/case1, which are in turn
//! `golden_parity_settlement_naming.rs`/`golden_parity_hierarchical_network.rs`'s
//! own already-verified fixtures -- not re-derived. Chosen because
//! `golden_parity_settlement_placement.rs` already confirmed (via real
//! extraction) that every one of these settlements is genuinely coastal,
//! and this task's own harness confirmed real mixed land/ocean/lake
//! geography at both grids (case0: 79 land / 75 ocean / 0 lake of 154
//! cells; case1: 127 land / 13 ocean / 52 lake of 192 cells) -- large
//! enough to exercise real Dijkstra sea-pathing and Prim's MST, not
//! degenerate all-land or all-ocean grids. case0 has 3 ports (n>2, so the
//! v0.73 nearest-port sea-lane augmentation branch is exercised); case1
//! has 5 ports (K5 land topology in the sibling test, but sea connectivity
//! is independent -- MST here only found the reachable pairs).
//!
//! `field[0]` cross-checked against both sibling tests' own already-passing
//! assertions before trusting this harness's extraction.
//!
//! Two of case1's four routes carry `km:0` despite having 3 real points --
//! confirmed a genuine reference behavior, not a harness bug: `_civSmoothPath`
//! accumulates `km` over the ROUNDED sample points (integer pixel
//! coordinates) BEFORE its own final step restores full-precision
//! endpoints, so a short diagonal hop whose only interior sample rounds to
//! coincide with the (pre-restore) rounded start point contributes zero
//! distance -- the same class of short-segment rounding quirk
//! `golden_parity_road_consolidation.rs`'s case0 already documents for
//! land routes (`js_round(6.5)=7` there; a coincident-rounding zero-km hop
//! here).
//!
//! Continuous point coordinates and km checked at `1e-4` (this crate's
//! established tolerance); name/point-count checked exactly.

fn named(x: usize, y: usize, faction: i32, name: &str, pop: u32) -> cartalith_civ::NamedSettlement {
    cartalith_civ::NamedSettlement {
        placement: cartalith_civ::SettlementPlacement {
            x,
            y,
            suit: 0.0,
            faction,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        name: name.to_string(),
        pop,
    }
}

fn build_water_bodies(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
) -> Vec<u8> {
    cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall)).classification
}

fn assert_pts_match(actual: &[(f64, f64)], expected: &[(f64, f64)], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: point count mismatch: {actual:?} vs {expected:?}");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!((a.0 - e.0).abs() < 1e-4, "{label}: pt {i} x mismatch: {a:?} vs {e:?}");
        assert!((a.1 - e.1).abs() < 1e-4, "{label}: pt {i} y mismatch: {a:?} vs {e:?}");
    }
}

#[test]
fn sea_routes_case_0_three_ports_augmentation() {
    // case0_region: gw=14 gh=11 seed=24601 world=false. Same 3 ports as
    // golden_parity_road_consolidation.rs's case0. Real extraction:
    // _civMstRoutes(ports,true) -- 2 edges (n=3 -> 2 MST edges; the v0.73
    // augmentation pass found no additional pair within the 1.15x cap, so
    // the result is exactly the MST).
    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.field[0] - 0.8640472292900085f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let water_bodies = build_water_bodies(&ws, 14, 11, false);
    let land = water_bodies.iter().filter(|&&w| w == 0).count();
    let ocean = water_bodies.iter().filter(|&&w| w == 1).count();
    assert_eq!((land, ocean), (79, 75), "case0: water-body cell counts mismatch, harness assumption broken");

    let ports = vec![
        named(7, 2, 1, "Sevjuniana", 19465),
        named(9, 3, 2, "Hurngarngarnhaskcairn", 20094),
        named(6, 2, 3, "Ghalbahrghaltazdune", 22094),
    ];

    let routes = cartalith_civ::civ_sea_routes(&ports, &ws.field, &water_bodies, 14, 11, false, p.map_width_km);

    assert_eq!(routes.len(), 2, "case0: route count mismatch");

    assert_eq!(routes[0].name, "Sevjuniana \u{2192} Hurngarngarnhaskcairn", "case0 route0: name mismatch");
    assert_pts_match(&routes[0].pts, &[(7.0, 2.0), (8.0, 3.0), (9.0, 3.0)], "case0 route0 pts");
    assert!((routes[0].km - 114.28571428571429).abs() < 1e-4, "case0 route0: km mismatch: {}", routes[0].km);
    assert!(routes[0].brks.is_empty(), "case0 route0: brks should be empty");

    assert_eq!(routes[1].name, "Sevjuniana \u{2192} Ghalbahrghaltazdune", "case0 route1: name mismatch");
    assert_pts_match(&routes[1].pts, &[(7.0, 2.0), (8.0, 2.0), (6.0, 2.0)], "case0 route1 pts");
    assert!((routes[1].km - 127.77531299998797).abs() < 1e-4, "case0 route1: km mismatch: {}", routes[1].km);
    assert!(routes[1].brks.is_empty(), "case0 route1: brks should be empty");
}

#[test]
fn sea_routes_case_1_five_ports_mixed_geography() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true. Same 5 ports as
    // golden_parity_road_consolidation.rs's case1. Real extraction:
    // _civMstRoutes(ports,true) -- 4 edges (n=5 -> 4 MST edges; again no
    // additional augmentation edge survived the 1.15x cap). Two routes
    // carry km:0 -- a genuine rounding quirk, see module doc comment.
    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.field[0] - 0.2477419376373291f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let water_bodies = build_water_bodies(&ws, 16, 12, true);
    let land = water_bodies.iter().filter(|&&w| w == 0).count();
    let ocean = water_bodies.iter().filter(|&&w| w == 1).count();
    let lake = water_bodies.iter().filter(|&&w| w == 2).count();
    assert_eq!((land, ocean, lake), (127, 13, 52), "case1: water-body cell counts mismatch, harness assumption broken");

    let ports = vec![
        named(9, 3, 1, "Sevjuniana", 20354),
        named(5, 8, 2, "Hurngarngarnhaskcairn", 20697),
        named(8, 9, 3, "Ghalbahrghaltazdune", 22698),
        named(10, 5, 4, "Orenelywash", 15972),
        named(4, 7, 5, "Taela'elorashade", 22508),
    ];

    let routes = cartalith_civ::civ_sea_routes(&ports, &ws.field, &water_bodies, 16, 12, true, p.map_width_km);

    assert_eq!(routes.len(), 4, "case1: route count mismatch");

    struct Expect {
        pts: Vec<(f64, f64)>,
        km: f64,
        name: &'static str,
    }
    let expected = [
        Expect { pts: vec![(9.0, 3.0), (9.0, 4.0), (10.0, 5.0)], km: 0.0, name: "Sevjuniana \u{2192} Orenelywash" },
        Expect { pts: vec![(9.0, 3.0), (9.0, 5.0), (8.0, 7.0), (8.0, 9.0)], km: 232.51407699364424, name: "Sevjuniana \u{2192} Ghalbahrghaltazdune" },
        Expect { pts: vec![(8.0, 9.0), (7.0, 8.0), (5.0, 8.0)], km: 111.80339887498948, name: "Ghalbahrghaltazdune \u{2192} Hurngarngarnhaskcairn" },
        Expect { pts: vec![(5.0, 8.0), (5.0, 7.0), (4.0, 7.0)], km: 0.0, name: "Hurngarngarnhaskcairn \u{2192} Taela'elorashade" },
    ];

    for (i, (r, e)) in routes.iter().zip(expected.iter()).enumerate() {
        let label = format!("case1 route{i}");
        assert_eq!(r.name, e.name, "{label}: name mismatch");
        assert_pts_match(&r.pts, &e.pts, &label);
        assert!((r.km - e.km).abs() < 1e-4, "{label}: km mismatch: {} vs {}", r.km, e.km);
        assert!(r.brks.is_empty(), "{label}: brks should be empty");
    }
}
