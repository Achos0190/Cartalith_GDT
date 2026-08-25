//! `trade.rs`'s own tests.
//!
//! The ported pieces are checked against the reference's literals and its
//! branch order; the one new rule (the allocation) is checked as an
//! invariant rather than a golden value, because it has no reference to
//! match — `PARITY_TESTING.md`'s own distinction.

use super::*;
use crate::{SettlementKind, SettlementPlacement};

fn place(x: usize, y: usize, coastal: bool, pop: u32) -> NamedSettlement {
    NamedSettlement {
        tid: 0,
        placement: SettlementPlacement {
            x,
            y,
            suit: 0.5,
            faction: 1,
            capital: false,
            kind: SettlementKind::Town,
            coastal,
        },
        name: String::new(),
        pop,
    }
}

fn nav(kind: NavKind) -> Navigability {
    Navigability { kind, basis: "test" }
}

fn balance(exports: &[&'static str], imports: &[&'static str]) -> TradeBalance {
    TradeBalance { exports: exports.to_vec(), imports: imports.to_vec() }
}

fn way(a: usize, b: usize, km: f64) -> Way {
    Way {
        tid: 0,
        pts: vec![(0.0, 0.0), (1.0, 1.0)],
        brks: vec![],
        km,
        name: String::new(),
        way_type: crate::WayType::Road,
        a_idx: a,
        b_idx: b,
        hidden: false,
    }
}

// ------------------------------------------------------------- ported rules

/// Every one of the fifteen keys `TradeBalance` ranges over must land in
/// exactly one of the two reference tables. A key in neither would silently
/// take `_civGoodReach`'s middle branch — the "worked good" case — and read
/// as something the reference never classified it as.
#[test]
fn every_resource_key_is_classified_bulk_or_luxury() {
    for &k in CIV_RESOURCE_KEYS.iter() {
        let bulk = BULK_GOODS.contains(&k);
        let lux = LUXURY_GOODS.contains(&k);
        assert!(bulk ^ lux, "{k} is bulk={bulk} luxury={lux}; must be exactly one");
    }
    assert_eq!(BULK_GOODS.len() + LUXURY_GOODS.len(), CIV_RESOURCE_KEYS.len());
}

/// `civ_trade_bridge.rs` labels a good by indexing `RESOURCE_NAMES` with a
/// position found in `CIV_RESOURCE_KEYS`. The two vocabularies are
/// documented as *different* (`RESOURCE_KEYS`' own doc comment says so), and
/// they happen to coincide in this port — so the coincidence is asserted
/// here rather than assumed at the call site, where a future divergence
/// would silently relabel every row.
#[test]
fn the_two_resource_vocabularies_still_coincide() {
    assert_eq!(crate::RESOURCE_KEYS, CIV_RESOURCE_KEYS);
    assert_eq!(crate::RESOURCE_NAMES.len(), CIV_RESOURCE_KEYS.len());
}

/// `_civGoodReach`'s four branches, in its own order.
#[test]
fn good_reach_matches_the_reference_branches() {
    // luxury: anywhere, whatever the water
    assert_eq!(good_reach("gold", nav(NavKind::None)), Reach::Long);
    assert_eq!(good_reach("gems", nav(NavKind::Sea)), Reach::Long);
    // bulk on the three kinds of water
    assert_eq!(good_reach("timber", nav(NavKind::Sea)), Reach::Long);
    assert_eq!(good_reach("timber", nav(NavKind::River)), Reach::Regional);
    assert_eq!(good_reach("timber", nav(NavKind::Stream)), Reach::Local);
    assert_eq!(good_reach("timber", nav(NavKind::None)), Reach::Local);
    // the same surplus is regional from a river port and local inland --
    // the reference's own sentence about why bulk sites cluster on water
    assert_ne!(good_reach("iron", nav(NavKind::River)), good_reach("iron", nav(NavKind::None)));
}

/// `_civFoodMode` — cheapest mode available to *both* ends. A sea port and
/// an inland town trade overland, which is the whole point of the function.
#[test]
fn trade_mode_is_the_cheaper_of_what_both_ends_have() {
    assert_eq!(trade_mode(nav(NavKind::Sea), nav(NavKind::Sea)), TradeMode::Sea);
    assert_eq!(trade_mode(nav(NavKind::Sea), nav(NavKind::River)), TradeMode::River);
    assert_eq!(trade_mode(nav(NavKind::River), nav(NavKind::River)), TradeMode::River);
    assert_eq!(trade_mode(nav(NavKind::Sea), nav(NavKind::None)), TradeMode::Land);
    assert_eq!(trade_mode(nav(NavKind::Stream), nav(NavKind::Sea)), TradeMode::Land);
    assert_eq!(trade_mode(nav(NavKind::None), nav(NavKind::None)), TradeMode::Land);
}

/// `2^(-d/D)`, at the doubling distance and at the reach cliff. The three
/// constants are mutation-checked here: changing any one of the six literals
/// moves a value this test pins.
#[test]
fn deliverable_halves_at_the_doubling_distance_and_cliffs_at_the_reach() {
    for (i, mode) in [TradeMode::Land, TradeMode::River, TradeMode::Sea].iter().enumerate() {
        assert_eq!(deliverable(0.0, *mode), 1.0);
        let half = deliverable(DOUBLE_KM[i], *mode);
        assert!((half - 0.5).abs() < 1e-12, "{mode:?} at D should be 0.5, got {half}");
        // just inside the cliff is a real (tiny) fraction; just past it is 0
        assert!(deliverable(MAX_REACH_KM[i], *mode) > 0.0);
        assert_eq!(deliverable(MAX_REACH_KM[i] + 1e-9, *mode), 0.0);
    }
    // the land cliff is well inside the river one -- the asymmetry that
    // makes a river port a different kind of place from an inland town
    assert_eq!(deliverable(500.0, TradeMode::Land), 0.0);
    assert!(deliverable(500.0, TradeMode::River) > 0.0);
}

/// The `!(distKm>=0)` guard, kept in its negated form for NaN.
#[test]
fn deliverable_absorbs_nan_the_way_js_does() {
    assert_eq!(deliverable(f64::NAN, TradeMode::Land), 0.0);
    assert_eq!(deliverable(-1.0, TradeMode::Sea), 0.0);
}

/// `_civFoodConnected`'s three lines, including the one that matters most:
/// inside the local radius no road is needed at all.
#[test]
fn connectivity_is_local_radius_then_water_then_road() {
    let ways = vec![way(0, 1, 40.0)];
    let mut rc = RoadComponents::build(3, &ways);
    // 0-1 share a way; 2 is isolated
    assert!(rc.connected(0, 1));
    assert!(!rc.connected(0, 2));
    // under the local radius: connected regardless
    assert!(connected(&mut rc, 0, 2, LOCAL_RADIUS_KM, TradeMode::Land));
    // past it, overland, with no road: not connected
    assert!(!connected(&mut rc, 0, 2, LOCAL_RADIUS_KM + 0.001, TradeMode::Land));
    // past it, but sharing water: connected -- water IS the connection
    assert!(connected(&mut rc, 0, 2, 900.0, TradeMode::Sea));
    assert!(connected(&mut rc, 0, 2, 900.0, TradeMode::River));
    // past it, overland, with a road: connected
    assert!(connected(&mut rc, 0, 1, 200.0, TradeMode::Land));
}

/// Transitivity, which is the reason this is a union-find and not a lookup:
/// A-B and B-C means A can be supplied from C.
#[test]
fn road_components_are_transitive() {
    let ways = vec![way(0, 1, 10.0), way(1, 2, 10.0), way(3, 4, 10.0)];
    let mut rc = RoadComponents::build(5, &ways);
    assert!(rc.connected(0, 2));
    assert!(!rc.connected(0, 3));
    assert!(rc.connected(3, 4));
}

/// A way whose endpoints are the same settlement, or out of range, must not
/// merge anything or panic -- `civ_consolidate_and_smooth_ways` is not
/// required to guarantee either.
#[test]
fn road_components_ignore_degenerate_ways() {
    let ways = vec![way(1, 1, 5.0), way(0, 99, 5.0)];
    let mut rc = RoadComponents::build(3, &ways);
    assert!(!rc.connected(0, 1));
    assert!(!rc.connected(1, 2));
}

// ------------------------------------------------------- the allocation rule

fn tiny_world() -> (Vec<f32>, Vec<f32>) {
    // 16x16, all land, no rivers -- so every settlement reads `landlocked`
    // and navigability comes from `coastal` alone, which is what lets these
    // tests set water access by hand.
    (vec![1.0; 256], vec![0.0; 256])
}

fn run(
    settlements: &[NamedSettlement],
    balances: &[TradeBalance],
    ways: &[Way],
    map_width_km: f64,
) -> TradeNetwork {
    let (field, flow) = tiny_world();
    let polys: Vec<Vec<(f64, f64)>> = Vec::new();
    let w = UrbanWorld {
        field: &field,
        flow: &flow,
        water_bodies: &[],
        order: None,
        river_polys: &polys,
        gw: 16,
        gh: 16,
        sea_level: 0.42,
        map_width_km,
        flow_thresh: 1.0,
        world_seed: 1,
    };
    let input = TradeInput { settlements, balances, ways, map_width_km, gw: 16 };
    trade_flows(&input, &w)
}

/// The shape of the whole thing on the smallest world that has one: a
/// surplus, a deficit, and a road between them.
#[test]
fn a_surplus_and_a_deficit_on_one_road_is_a_flow() {
    let s = vec![place(0, 0, false, 1000), place(4, 0, false, 500)];
    let b = vec![balance(&["iron"], &[]), balance(&[], &["iron"])];
    let ways = vec![way(0, 1, 40.0)];
    let net = run(&s, &b, &ways, 160.0); // 10 km/cell -> 40 km apart

    assert_eq!(net.flows.len(), 1, "one exporter, one importer, one good");
    let f = &net.flows[0];
    assert_eq!((f.from, f.to, f.good), (0, 1, "iron"));
    assert_eq!(f.mode, TradeMode::Land);
    assert_eq!(f.reach, Reach::Local, "bulk with no navigable water");
    assert!((f.distance_km - 40.0).abs() < 1e-9);
    // one candidate, so the whole demand, capped by the supplier's share
    assert!((f.volume - 500.0).abs() < 1e-9, "volume {} should be the importer's pop", f.volume);
    assert!(net.unmet.is_empty());
    // the road carries it
    assert!((net.way_load[0] - f.volume).abs() < 1e-9);
    assert_eq!(net.transient_bytes > 0, true);
}

/// The supplier cap is real and it binds: a large consumer next to a small
/// producer does not get its whole demand.
#[test]
fn one_consumer_never_draws_a_suppliers_whole_surplus() {
    let s = vec![place(0, 0, false, 100), place(1, 0, false, 10_000)];
    let b = vec![balance(&["iron"], &[]), balance(&[], &["iron"])];
    let net = run(&s, &b, &[], 160.0);
    assert_eq!(net.flows.len(), 1);
    assert!(
        (net.flows[0].volume - SUPPLIER_SHARE * 100.0).abs() < 1e-9,
        "capped at SUPPLIER_SHARE x supplier pop, got {}",
        net.flows[0].volume
    );
}

/// Two reachable suppliers split one demand, and the nearer one gets more --
/// which is the decay curve doing the work, not a rank.
///
/// Both suppliers are inside `LOCAL_RADIUS_KM`, deliberately: `iron` is bulk
/// and neither end has navigable water, so `_civGoodReach` puts it at
/// `local` and anything further would be correctly refused rather than
/// merely decayed. That refusal is what the test below this one checks.
#[test]
fn demand_splits_across_suppliers_by_deliverability() {
    // 10 km/cell. Importer at x=8; suppliers at x=7 (10 km) and x=4 (40 km).
    let s = vec![
        place(7, 0, false, 100_000),
        place(4, 0, false, 100_000),
        place(8, 0, false, 1000),
    ];
    let b = vec![balance(&["iron"], &[]), balance(&["iron"], &[]), balance(&[], &["iron"])];
    let ways: Vec<Way> = Vec::new();
    let net = run(&s, &b, &ways, 160.0);

    assert_eq!(net.flows.len(), 2);
    let near = net.flows.iter().find(|f| f.from == 0).unwrap();
    let far = net.flows.iter().find(|f| f.from == 1).unwrap();
    assert!(near.volume > far.volume, "near {} far {}", near.volume, far.volume);
    // neither is capped here (suppliers are huge), so the two sum to demand
    let total: f64 = net.flows.iter().map(|f| f.volume).sum();
    assert!((total - 1000.0).abs() < 1e-6, "uncapped flows must cover the whole demand, got {total}");
    // and the split is exactly the deliverable ratio
    let ratio = near.volume / far.volume;
    let expect = deliverable(10.0, TradeMode::Land) / deliverable(40.0, TradeMode::Land);
    assert!((ratio - expect).abs() < 1e-9, "{ratio} vs {expect}");
    assert!(ratio > 1.13 && ratio < 1.15, "the curve must actually separate them, got {ratio}");
}

/// The reach gate refuses what the decay curve would merely have thinned: a
/// bulk good from a landlocked supplier stops at `LOCAL_RADIUS_KM`, even
/// though `deliverable` at 80 km is still a healthy 0.71. This is the rule
/// behind the reference's own sentence about why bulk-producing sites
/// cluster on water, and it would be invisible without a test that sits
/// either side of it.
#[test]
fn reach_refuses_a_bulk_good_the_decay_curve_would_still_have_carried() {
    assert!(deliverable(80.0, TradeMode::Land) > 0.7, "the curve alone would allow it");
    let s = vec![place(0, 0, false, 100_000), place(8, 0, false, 1000)];
    let b = vec![balance(&["iron"], &[]), balance(&[], &["iron"])];
    // a road exists, so connectivity is not what refuses this
    let ways = vec![way(0, 1, 80.0)];
    let net = run(&s, &b, &ways, 160.0);
    assert!(net.flows.is_empty());
    assert_eq!(net.unmet.len(), 1);

    // the identical pair, both on the sea: `long` reach, and it trades
    let s2 = vec![place(0, 0, true, 100_000), place(8, 0, true, 1000)];
    let net2 = run(&s2, &b, &ways, 160.0);
    assert_eq!(net2.flows.len(), 1);
    assert_eq!(net2.flows[0].reach, Reach::Long);
}

/// A need with an exporter that cannot reach it is an unmet need, not a
/// silent zero — the reference's own `foodUnsupported` distinction.
#[test]
fn an_unreachable_exporter_leaves_an_unmet_need() {
    // 15 cells apart at 20 km/cell = 300 km: past the land cliff (220 km),
    // and with no road, so `_civFoodConnected` refuses it too.
    let s = vec![place(0, 0, false, 1000), place(15, 0, false, 1000)];
    let b = vec![balance(&["iron"], &[]), balance(&[], &["iron"])];
    let net = run(&s, &b, &[], 320.0);
    assert!(net.flows.is_empty());
    assert_eq!(net.unmet.len(), 1);
    assert_eq!(net.unmet[0].settlement, 1);
    assert_eq!(net.unmet[0].good, "iron");
    assert!(net.unmet[0].exporter_exists, "somebody exports it; nobody in reach does");
}

/// The same pair, both on the sea, does trade — that is the whole reason the
/// mode exists, and it is the sharpest single check that the ported curve is
/// wired to the ported navigability.
#[test]
fn the_same_pair_trades_once_both_ends_are_ports() {
    let s = vec![place(0, 0, true, 1000), place(15, 0, true, 1000)];
    let b = vec![balance(&["iron"], &[]), balance(&[], &["iron"])];
    let net = run(&s, &b, &[], 320.0);
    assert_eq!(net.flows.len(), 1);
    assert_eq!(net.flows[0].mode, TradeMode::Sea);
    assert_eq!(net.flows[0].reach, Reach::Long);
    assert!(net.unmet.is_empty());
    // a sea flow lands on no way
    assert!(net.way_load.iter().all(|&v| v == 0.0));
}

/// Nobody exports it at all: still unmet, and the flag says which kind of
/// nothing it is.
#[test]
fn a_good_nobody_exports_is_unmet_with_the_flag_down() {
    let s = vec![place(0, 0, false, 1000), place(1, 0, false, 1000)];
    let b = vec![balance(&[], &["salt"]), balance(&[], &["salt"])];
    let net = run(&s, &b, &[], 160.0);
    assert_eq!(net.unmet.len(), 2);
    assert!(net.unmet.iter().all(|u| !u.exporter_exists));
}

/// A flow routes over the ways that join the two, not over a straight line:
/// a two-hop path loads both hops and leaves the unrelated way alone.
#[test]
fn a_flow_loads_every_way_on_its_shortest_path() {
    let s = vec![
        place(0, 0, false, 1000),
        place(2, 0, false, 1000),
        place(4, 0, false, 1000),
        place(0, 8, false, 1000),
    ];
    let b = vec![
        balance(&["iron"], &[]),
        balance(&[], &[]),
        balance(&[], &["iron"]),
        balance(&[], &[]),
    ];
    let ways = vec![way(0, 1, 20.0), way(1, 2, 20.0), way(0, 3, 80.0)];
    let net = run(&s, &b, &ways, 160.0);
    assert_eq!(net.flows.len(), 1);
    let v = net.flows[0].volume;
    assert!((net.way_load[0] - v).abs() < 1e-9, "first hop");
    assert!((net.way_load[1] - v).abs() < 1e-9, "second hop");
    assert_eq!(net.way_load[2], 0.0, "an unrelated way carries nothing");
}

/// The seven keys `CIV_CONSUMED_RESOURCES` excludes can never be imports, so
/// they can never be flows however much of them a settlement has. This
/// asserts `TradeBalance`'s asymmetry survives into the match rather than
/// being quietly worked around.
#[test]
fn a_good_that_can_never_be_an_import_produces_no_flow() {
    let s = vec![place(0, 0, false, 1000), place(1, 0, false, 1000)];
    // `gold` is exportable and never importable, so no importer list exists
    let b = vec![balance(&["gold"], &[]), balance(&["gold"], &[])];
    let net = run(&s, &b, &[], 160.0);
    assert!(net.flows.is_empty());
    assert!(net.unmet.is_empty());
}

/// Two runs on the same input agree exactly. Nothing is cached across calls
/// and nothing is order-dependent past the settlement order itself.
#[test]
fn the_match_is_deterministic() {
    let s = vec![
        place(0, 0, false, 1200),
        place(3, 2, false, 900),
        place(9, 5, true, 400),
        place(12, 1, true, 2100),
    ];
    let b = vec![
        balance(&["iron", "timber"], &["salt"]),
        balance(&["salt"], &["iron"]),
        balance(&["salt"], &["timber"]),
        balance(&[], &["iron", "salt", "timber"]),
    ];
    let ways = vec![way(0, 1, 35.0), way(1, 3, 90.0)];
    let a = run(&s, &b, &ways, 200.0);
    let c = run(&s, &b, &ways, 200.0);
    assert_eq!(a.flows, c.flows);
    assert_eq!(a.unmet, c.unmet);
    assert_eq!(a.way_load, c.way_load);
    assert!(!a.flows.is_empty(), "the fixture must actually reach the code");
}

/// Empty and mismatched inputs return an empty network rather than panicking
/// — this runs behind a user-pressed button on whatever state the app is in.
#[test]
fn degenerate_inputs_are_empty_not_a_panic() {
    assert_eq!(run(&[], &[], &[], 800.0), TradeNetwork::default());
    let s = vec![place(0, 0, false, 100)];
    assert_eq!(run(&s, &[], &[], 800.0), TradeNetwork::default());
}
