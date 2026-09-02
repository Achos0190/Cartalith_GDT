//! `trade.rs`'s own tests.
//!
//! The ported pieces are checked against the reference's literals and its
//! branch order; the one new rule (the allocation) is checked as an
//! invariant rather than a golden value, because it has no reference to
//! match — `PARITY_TESTING.md`'s own distinction.

use super::*;
use crate::{SettlementKind, SettlementPlacement};

fn place(x: usize, y: usize, coastal: bool, pop: u32) -> NamedSettlement {
    place_kind(x, y, coastal, pop, SettlementKind::Town)
}

fn place_kind(x: usize, y: usize, coastal: bool, pop: u32, kind: SettlementKind) -> NamedSettlement {
    NamedSettlement {
        tid: 0,
        placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind, coastal },
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

// ----------------------------------------------------------- civ_food_shed
//
// `ECONOMY_SCOPE.md` milestone 2's own function -- see `civ_food_shed`'s doc
// comment for the reference lines and what this pass found already ported
// versus what it built. No golden fixture: pure, branch-complete, no
// RNG/state, every branch traceable to the reference with no iteration
// order to get subtly wrong -- the same justification `civ_resource_trade_balance`'s
// own tests (this crate's `lib.rs`) already used for real unit tests in
// place of a Node-harness extraction, and it applies here for the same
// reasons.

fn landlocked_nav() -> Navigability {
    Navigability { kind: NavKind::None, basis: "test" }
}

/// Every field this fixture family holds constant: `soil_ref` matches the
/// uniform `0.5` soil every test below uses (so `foodSurplusRatio` is being
/// exercised at its own calibration point, not off to one side of it), and
/// `sea`/`world_wrap` match every `place()`d settlement's own assumptions.
#[allow(clippy::too_many_arguments)]
fn shed_input<'a>(
    settlements: &'a [NamedSettlement],
    navigability: &'a [Navigability],
    farmers_per_urbanite: &'a [f64],
    dens: &'a [f32],
    soil: &'a [f32],
    field: &'a [f32],
    gw: usize,
    gh: usize,
    map_width_km: f64,
) -> FoodShedInput<'a> {
    FoodShedInput {
        settlements,
        navigability,
        farmers_per_urbanite,
        dens,
        soil,
        soil_ref: 0.5,
        field,
        gw,
        gh,
        sea: 0.42,
        world_wrap: false,
        map_width_km,
    }
}

/// `(settlements, farmers_per_urbanite, dens, soil, field)` -- everything
/// [`whole_grid_is_one_catchment`] builds, named once so its signature
/// reads rather than a five-tuple of collections.
type CatchmentFixture = (Vec<NamedSettlement>, [f64; 1], Vec<f32>, Vec<f32>, Vec<f32>);

/// A 5x5 world small enough that a `Metropolis`' own catchment radius (its
/// own doc comment on `catchment_radius_raw` explains the raw-vs-cells
/// distinction) covers every cell, including the far corner: `catR ≈ 5.64`
/// cells against a farthest corner-to-centre distance of `hypot(2,2) ≈
/// 2.83`. Every cell therefore fails `civ_food_shed`'s own
/// `dist_cells<=cat_r` hinterland-exclusion test, so `hinterland_capacity`
/// is `0.0` by construction and `local_capacity` alone is checkable in
/// closed form.
fn whole_grid_is_one_catchment(pop: u32, fpu: f64) -> CatchmentFixture {
    let gw = 5;
    let field = vec![0.6f32; gw * gw];
    let dens = vec![10.0f32; gw * gw]; // uniform -> the catchment mean is exactly 10.0
    let soil = vec![0.5f32; gw * gw]; // == soil_ref, so foodSurplusRatio sits at its own baseline
    let settlements = vec![place_kind(2, 2, false, pop, SettlementKind::Metropolis)];
    (settlements, [fpu], dens, soil, field)
}

/// `local_capacity` in closed form: `civ_catchment_pop` over a uniform
/// density field is just `density * catchment_km2` (the mean of a constant
/// is the constant), and at `soil == soil_ref` with the default
/// `FARMERS_PER_URBANITE`, `foodSurplusRatio` collapses to exactly
/// `FOOD_BASE_SURPLUS_RATIO = 1/9` (`food_surplus_ratio`'s own doc comment
/// on `is_default`) -- `civ_catchment_km2(Metropolis) == 2500.0`, so
/// `local_capacity == 25000/9`. `hinterland_capacity`/`import_capacity` are
/// both `0.0` (the fixture's own doc comment; `import` because there is
/// only one settlement), so `supported == local_capacity` exactly, and the
/// two `pop` values sit either side of `supported` -- the `sustainable`
/// flag and `over_by`'s rounding both get a real, closed-form assertion
/// rather than a sign check.
#[test]
fn food_shed_local_capacity_closed_form_when_hinterland_and_import_are_both_zero() {
    let nav = [landlocked_nav()];
    let expect_local = 25_000.0 / 9.0;

    let (s, fpu, dens, soil, field) = whole_grid_is_one_catchment(1000, FARMERS_PER_URBANITE);
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, 5, 5, 25.0);
    let mut rc = RoadComponents::build(1, &[]);
    let out = civ_food_shed(&input, &mut rc, 0);
    assert_eq!(out.hinterland_capacity, 0.0, "every cell is inside the settlement's own catchment");
    assert_eq!(out.import_capacity, 0.0, "one settlement has no possible supplier");
    assert!(
        (out.local_capacity - expect_local).abs() < 1e-6,
        "got {} want {expect_local}",
        out.local_capacity
    );
    assert!((out.supported - out.local_capacity).abs() < 1e-12);
    assert_eq!(out.limited_by, "local");
    assert!(out.sustainable, "pop 1000 is well under supported {expect_local}");
    assert_eq!(out.over_by, 0.0);

    // Same world, pop pushed past `supported` -- deficit, and `overBy` is
    // `Math.round(pop-supported)` on a genuinely positive value.
    let (s2, fpu2, dens2, soil2, field2) = whole_grid_is_one_catchment(3000, FARMERS_PER_URBANITE);
    let input2 = shed_input(&s2, &nav, &fpu2, &dens2, &soil2, &field2, 5, 5, 25.0);
    let mut rc2 = RoadComponents::build(1, &[]);
    let out2 = civ_food_shed(&input2, &mut rc2, 0);
    assert!(!out2.sustainable, "pop 3000 exceeds supported {expect_local}");
    let expect_over = js_round(3000.0 - expect_local);
    assert!((out2.over_by - expect_over).abs() < 1e-6, "got {} want {expect_over}", out2.over_by);
}

/// The money test: the SAME grid, the same population, two different
/// `farmers_per_urbanite` values -- and `local_capacity` genuinely differs.
/// This is `AG_TECH_LEVELS`' route to influencing the trade/economy layer;
/// `roster.rs`'s own module doc used to say no such route was ported.
#[test]
fn food_shed_ag_tech_genuinely_changes_local_capacity() {
    let nav = [landlocked_nav()];

    let (s9, fpu9, dens9, soil9, field9) = whole_grid_is_one_catchment(1000, FARMERS_PER_URBANITE);
    let input9 = shed_input(&s9, &nav, &fpu9, &dens9, &soil9, &field9, 5, 5, 25.0);
    let mut rc9 = RoadComponents::build(1, &[]);
    let traditional = civ_food_shed(&input9, &mut rc9, 0).local_capacity;

    // farmers_per_urbanite = 1.0 ("Improved Agrarian" in AG_TECH_LEVELS) --
    // far fewer farmers needed per urbanite, so far more of the catchment's
    // ceiling is exportable surplus.
    let (s1, fpu1, dens1, soil1, field1) = whole_grid_is_one_catchment(1000, 1.0);
    let input1 = shed_input(&s1, &nav, &fpu1, &dens1, &soil1, &field1, 5, 5, 25.0);
    let mut rc1 = RoadComponents::build(1, &[]);
    let improved = civ_food_shed(&input1, &mut rc1, 0).local_capacity;

    assert!((traditional - 25_000.0 / 9.0).abs() < 1e-6);
    assert!((improved - 12_500.0).abs() < 1e-6, "0.5 * 25000 exactly, both operands exact in f64");
    assert!(
        improved > traditional * 4.0,
        "advanced ag-tech must lift the ceiling by more than a rounding error: {improved} vs {traditional}"
    );
}

/// A world where `cell_km` is chosen so `MAX_REACH_KM[Land]/cell_km == 1`
/// exactly, so the hinterland sweep only ever visits the immediate 3x3
/// block. Of its 8 non-centre cells, the 4 orthogonal ones sit at distance
/// `1.0` (inside the reach) and the 4 diagonal ones at `sqrt(2)` (just
/// outside it) -- a discrete geometric fact, not a re-derivation of the
/// `deliverable` decay curve (which has its own dedicated tests above and
/// is only ever called here, not recomputed).
#[test]
fn food_shed_hinterland_counts_exactly_the_four_orthogonal_neighbours_at_the_reach_cliff() {
    let gw = 5;
    let cell_km = 220.0; // == MAX_REACH_KM[Land]
    let map_width_km = cell_km * gw as f64;
    let field = vec![0.6f32; gw * gw];
    let dens = vec![10.0f32; gw * gw];
    let soil = vec![0.5f32; gw * gw];
    let s = vec![place_kind(2, 2, false, 1, SettlementKind::Hamlet)];
    let fpu = [FARMERS_PER_URBANITE];
    let nav = [landlocked_nav()];
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, gw, gw, map_width_km);
    let mut rc = RoadComponents::build(1, &[]);
    let out = civ_food_shed(&input, &mut rc, 0);

    let frac = deliverable(1.0 * cell_km, TradeMode::Land);
    let sr = food_surplus_ratio(0.5, 0.5, FARMERS_PER_URBANITE);
    let expect = 4.0 * 10.0 * cell_km * cell_km * frac * sr;
    assert!(
        (out.hinterland_capacity - expect).abs() < 1e-6,
        "got {} want {expect} (4 orthogonal cells, not 8)",
        out.hinterland_capacity
    );

    // The same world with every cell OUTSIDE the scanned 3x3 block zeroed
    // out must read identically -- proving those cells are never visited,
    // not merely zero-weighted.
    let mut dens_clipped = vec![0.0f32; gw * gw];
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            let (x, y) = (2 + dx, 2 + dy);
            dens_clipped[y as usize * gw + x as usize] = 10.0;
        }
    }
    let input2 = shed_input(&s, &nav, &fpu, &dens_clipped, &soil, &field, gw, gw, map_width_km);
    let mut rc2 = RoadComponents::build(1, &[]);
    let out2 = civ_food_shed(&input2, &mut rc2, 0);
    assert_eq!(out.hinterland_capacity, out2.hinterland_capacity, "cells past the reach cliff must not matter");
}

/// A supplier one kilometre away, well inside `LOCAL_RADIUS_KM`, needs no
/// road at all (`_civFoodConnected`'s own first branch) -- so a zero-pop
/// neighbour with real catchment capacity shows up as real import capacity.
#[test]
fn food_shed_import_draws_from_a_nearby_zero_pop_neighbour_with_no_road_needed() {
    let gw = 3;
    let field = vec![0.6f32; gw * gw];
    let dens = vec![10.0f32; gw * gw];
    let soil = vec![0.5f32; gw * gw];
    // 1 km apart: cell_km = map_width_km / gw = 3/3 = 1.0.
    let s = vec![place(0, 0, false, 500), place(1, 0, false, 0)];
    let fpu = [FARMERS_PER_URBANITE, FARMERS_PER_URBANITE];
    let nav = [landlocked_nav(), landlocked_nav()];
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, gw, gw, 3.0);
    let mut rc = RoadComponents::build(2, &[]);
    let out = civ_food_shed(&input, &mut rc, 0);
    assert_eq!(out.suppliers, 1);
    assert!(out.import_capacity > 0.0);
    assert_eq!(out.best_mode, TradeMode::Land);
}

/// The same neighbour, moved past `LOCAL_RADIUS_KM` with no road between
/// them: `_civFoodConnected` refuses it, so the same spare capacity
/// contributes nothing, mirroring `connectivity_is_local_radius_then_water_then_road`
/// above but through `civ_food_shed` rather than `connected` directly.
#[test]
fn food_shed_import_is_refused_without_connectivity_even_with_real_spare_capacity() {
    let gw = 15;
    let field = vec![0.6f32; gw * gw];
    let dens = vec![10.0f32; gw * gw];
    let soil = vec![0.5f32; gw * gw];
    // 10 cells apart at cell_km = 150/15 = 10 -> 100 km: past LOCAL_RADIUS_KM
    // (50) and past neither end has water, so no road means no connection.
    let s = vec![place(0, 0, false, 500), place(10, 0, false, 0)];
    let fpu = [FARMERS_PER_URBANITE, FARMERS_PER_URBANITE];
    let nav = [landlocked_nav(), landlocked_nav()];
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, gw, gw, 150.0);
    let mut rc = RoadComponents::build(2, &[]); // no ways
    let out = civ_food_shed(&input, &mut rc, 0);
    assert_eq!(out.suppliers, 0);
    assert_eq!(out.import_capacity, 0.0);
}

/// The reference's negated `!(spare>0)` form matters when
/// `farmers_per_urbanite` itself is `NaN` (a defensive case, not a real
/// roster value): `js_max`/division propagate the `NaN` into `food_surplus_ratio`'s
/// own `y_sub`, and that function's OWN negated guard collapses it to a
/// clean `0.0` ratio rather than a `NaN` surplus -- proved end to end here,
/// not just at `food_surplus_ratio` in isolation.
#[test]
fn food_shed_nan_farmers_per_urbanite_yields_zero_local_capacity_not_nan() {
    let nav = [landlocked_nav()];
    let (s, _, dens, soil, field) = whole_grid_is_one_catchment(1000, 0.0);
    let fpu = [f64::NAN];
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, 5, 5, 25.0);
    let mut rc = RoadComponents::build(1, &[]);
    let out = civ_food_shed(&input, &mut rc, 0);
    assert_eq!(out.local_capacity, 0.0, "must collapse to zero, not propagate NaN");
    assert!(out.supported.is_finite());
}

/// `soilAt?soilAt[li]:0.5` -- an empty/mis-sized soil slice reads as
/// uniform `0.5` everywhere, matching a world with real `0.5` soil exactly.
#[test]
fn food_shed_missing_soil_defaults_to_one_half_everywhere() {
    let nav = [landlocked_nav()];
    let (s, fpu, dens, soil, field) = whole_grid_is_one_catchment(1000, FARMERS_PER_URBANITE);
    let with_soil = {
        let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, 5, 5, 25.0);
        let mut rc = RoadComponents::build(1, &[]);
        civ_food_shed(&input, &mut rc, 0)
    };
    let without_soil: Vec<f32> = Vec::new();
    let missing = {
        let input = shed_input(&s, &nav, &fpu, &dens, &without_soil, &field, 5, 5, 25.0);
        let mut rc = RoadComponents::build(1, &[]);
        civ_food_shed(&input, &mut rc, 0)
    };
    assert_eq!(with_soil, missing);
}

/// A `p_idx` past the settlement slice returns the reference's own baseline
/// object untouched, matching the reference's `if(!p||...) return out;`
/// guard rather than panicking.
#[test]
fn food_shed_out_of_range_index_returns_the_default() {
    let nav = [landlocked_nav()];
    let (s, fpu, dens, soil, field) = whole_grid_is_one_catchment(1000, FARMERS_PER_URBANITE);
    let input = shed_input(&s, &nav, &fpu, &dens, &soil, &field, 5, 5, 25.0);
    let mut rc = RoadComponents::build(1, &[]);
    assert_eq!(civ_food_shed(&input, &mut rc, 1), FoodShed::default());
    assert_eq!(civ_food_shed(&input, &mut rc, 99), FoodShed::default());
}

/// `pop<=supported*1.0001` -- the reference's own 0.01% slack, checked on
/// both sides against the SAME multiplication `civ_food_shed` performs
/// (not an assumed decimal), since `1.0001` is not exactly representable.
#[test]
fn food_shed_sustainable_flag_honours_the_reference_slack() {
    let nav = [landlocked_nav()];
    let supported = 12_500.0; // whole_grid_is_one_catchment @ fpu=1.0, from the closed-form test above
    let boundary = supported * 1.0001;

    let (s_in, fpu, dens, soil, field) = whole_grid_is_one_catchment((boundary - 0.01) as u32, 1.0);
    let input_in = shed_input(&s_in, &nav, &fpu, &dens, &soil, &field, 5, 5, 25.0);
    let mut rc_in = RoadComponents::build(1, &[]);
    assert!(civ_food_shed(&input_in, &mut rc_in, 0).sustainable, "just inside the slack");

    let (s_out, fpu2, dens2, soil2, field2) = whole_grid_is_one_catchment((boundary + 1.0) as u32, 1.0);
    let input_out = shed_input(&s_out, &nav, &fpu2, &dens2, &soil2, &field2, 5, 5, 25.0);
    let mut rc_out = RoadComponents::build(1, &[]);
    assert!(!civ_food_shed(&input_out, &mut rc_out, 0).sustainable, "past the slack");
}

// ------------------------------------- smelting and salt (`ECONOMY_SCOPE.md`)
//
// `golden_parity_smelting_salt.rs` compares both functions against the
// reference over three worlds. These tests exist because a mutation sweep
// over the new constants and branches found six the golden fixtures could
// not reach -- every test below was written to kill a named survivor, and
// says which.

fn zero_pots(n: usize) -> ResourcePotentials {
    let z = || vec![0.0f32; n];
    ResourcePotentials {
        copper: z(),
        tin: z(),
        iron: z(),
        gold: z(),
        salt: z(),
        timber: z(),
        lead: z(),
        silver: z(),
        clay: z(),
        buildstone: z(),
        flint: z(),
        obsidian: z(),
        gems: z(),
        sulfur: z(),
        alum: z(),
    }
}

/// A 3x3 all-land world at the default 800 km map width, where every tier's
/// catchment radius is the `max(1, ...)` floor, so the disc around the
/// centre is exactly the five orthogonally-connected cells.
fn uniform_smelt(iron: f32, timber: f32, sea: f64, height: f32) -> Smelting {
    let n = 9;
    let field = vec![height; n];
    let mut res = zero_pots(n);
    res.iron = vec![iron; n];
    res.timber = vec![timber; n];
    let w = PlaceWorld {
        res: &res,
        field: &field,
        biome: &[],
        rain: &[],
        gw: 3,
        gh: 3,
        sea,
        map_width_km: 800.0,
    };
    civ_place_smelting(&w, 1, 1, SettlementKind::Village)
}

/// Kills `ocean < sea -> <= sea`. The reference excludes a cell only when
/// `field[i] < sea`, so a cell sitting *exactly* at sea level is land and
/// counts -- a boundary no generated fixture lands on.
#[test]
fn smelting_counts_a_cell_exactly_at_sea_level() {
    let at = uniform_smelt(1.0, 0.0, 0.5, 0.5);
    assert!(at.ore_kg_yr > 0.0, "a cell at exactly sea level must count as land");
    let below = uniform_smelt(1.0, 0.0, 0.5, 0.499);
    assert_eq!(below.ore_kg_yr, 0.0, "a cell below sea level must not");
    // Five cells (the r=1 disc), each 800/3 km square, 100 ha per km^2.
    let cell_ha = (800.0f64 / 3.0) * (800.0 / 3.0) * 100.0;
    assert!((at.ore_kg_yr - 5.0 * cell_ha * ORE_KG_PER_HA_YR).abs() < 1e-6);
}

/// Kills `fuel_poor 0.5 -> 0.6` and `ore_rich 2.0 -> 1.9`. Both flags are
/// bracketed from each side, at fuel/ore ratios no generated world produced:
/// `iron_from_fuel/iron_from_ore` is `(1000/6.7)/(900*0.33) = 0.50254` times
/// `timber/iron`, so the four pairs below sit at 0.283, 0.565, 1.950 and
/// 2.010 -- two of them inside the `[0.5,0.6)` and `(1.9,2.0]` windows the
/// mutants would have moved the thresholds into.
#[test]
fn fuel_poor_and_ore_rich_brackets_are_the_reference_multiples() {
    let ratio = |iron: f32, timber: f32| {
        let s = uniform_smelt(iron, timber, 0.42, 1.0);
        let from_fuel = s.charcoal_kg_yr / CHARCOAL_PER_IRON_KG;
        let from_ore = s.ore_kg_yr * ORE_TO_BLOOM_RECOVERY;
        (s, from_fuel / from_ore)
    };

    let (poor, r) = ratio(0.8, 0.45);
    assert!(r < 0.5, "ratio {r} should be under the fuel-poor threshold");
    assert!(poor.fuel_poor && poor.limited_by == "fuel");

    let (just_ok, r) = ratio(0.8, 0.9);
    assert!((0.5..0.6).contains(&r), "ratio {r} must sit between 0.5 and 0.6");
    assert!(!just_ok.fuel_poor, "0.5 is the threshold, not 0.6");
    assert_eq!(just_ok.limited_by, "fuel", "fuel still binds below 1.0");

    let (just_under, r) = ratio(0.25, 0.97);
    assert!((1.9..2.0).contains(&r), "ratio {r} must sit between 1.9 and 2.0");
    assert!(!just_under.ore_rich, "2.0 is the threshold, not 1.9");

    let (over, r) = ratio(0.25, 1.0);
    assert!(r > 2.0, "ratio {r} should clear the ore-rich threshold");
    assert!(over.ore_rich && over.limited_by == "ore");
}

/// `Math.min` propagates `NaN`; `f64::min` absorbs it. A single unusable
/// resource cell must leave `iron_kg_yr` unusable rather than silently
/// reporting the other budget as the answer.
#[test]
fn smelting_propagates_nan_the_way_math_min_does() {
    let mut res = zero_pots(9);
    res.iron = vec![f32::NAN; 9];
    res.timber = vec![1.0; 9];
    let field = vec![1.0f32; 9];
    let w = PlaceWorld {
        res: &res,
        field: &field,
        biome: &[],
        rain: &[],
        gw: 3,
        gh: 3,
        sea: 0.42,
        map_width_km: 800.0,
    };
    let s = civ_place_smelting(&w, 1, 1, SettlementKind::Village);
    assert!(s.iron_kg_yr.is_nan(), "f64::min would have returned the fuel budget here");
    assert!(s.charcoal_kg_yr > 0.0, "the fuel side is still real");
    assert_eq!(s.limited_by, "ore", "`NaN < x` is false in JS and in Rust");
}

/// The reference's two early-outs (`!field`, `!pots.iron`) and its
/// `if(pots.timber)` guard, which is the only one that still produces a
/// number: no woodland at all is maximally fuel-poor, not an error.
#[test]
fn smelting_absent_fields_match_the_reference_guards() {
    let res = zero_pots(9);
    let field = vec![1.0f32; 9];
    let world = |res: &ResourcePotentials, field: &[f32]| civ_place_smelting(
        &PlaceWorld {
            res,
            field,
            biome: &[],
            rain: &[],
            gw: 3,
            gh: 3,
            sea: 0.42,
            map_width_km: 800.0,
        },
        1,
        1,
        SettlementKind::Village,
    );
    // `!field`
    assert_eq!(world(&res, &[]), Smelting::default());
    // `!pots.iron`
    let mut no_iron = zero_pots(9);
    no_iron.iron = vec![];
    assert_eq!(world(&no_iron, &field), Smelting::default());
    // `if(pots.timber)` -- ore with no woodland field at all.
    let mut no_timber = zero_pots(9);
    no_timber.iron = vec![1.0; 9];
    no_timber.timber = vec![];
    let s = world(&no_timber, &field);
    assert_eq!(s.woodland_ha, 0.0);
    assert!(s.ore_kg_yr > 0.0 && s.iron_kg_yr == 0.0 && s.fuel_poor && s.limited_by == "fuel");
}

/// A world whose only variable is the salt mean, so the 0.25 threshold can
/// be approached from both sides. Kills `SALT_DEPOSIT_MEAN .25 -> .20`
/// (0.22 must stay `none`) and pins the reference's strict `>` (an exact
/// 0.25 is not a deposit).
#[test]
fn salt_deposit_threshold_is_strictly_above_one_quarter() {
    let verdict = |salt: f32| {
        let n = 81;
        let mut res = zero_pots(n);
        res.salt = vec![salt; n];
        let field = vec![1.0f32; n];
        let w = PlaceWorld {
            res: &res,
            field: &field,
            biome: &[],
            rain: &[],
            gw: 9,
            gh: 9,
            sea: 0.42,
            map_width_km: 800.0,
        };
        civ_salt_access(&w, 4, 4, NavKind::None)
    };
    assert_eq!(verdict(0.22).source, "none", "0.22 is not a deposit");
    assert_eq!(verdict(0.25).source, "none", "the reference's test is `>0.25`, not `>=`");
    assert_eq!(verdict(0.26), SaltAccess { has: true, source: "salt deposit" });
}

/// Kills `salt radius /128 -> /8`. `_civSaltAccess` calls
/// `_civPlaceResourceContext(p)` with **no** radius, so the window is that
/// function's own default `max(3, round(GW/128))` -- not the settlement's
/// catchment radius and not any other derivation of `GW`. A nine-cell salt
/// blob on a 32-wide grid means 9/29 = 0.310 at radius 3 (a deposit) and
/// 9/49 = 0.184 at radius 4 (not one), so the verdict names the radius.
#[test]
fn salt_deposit_window_is_the_resource_contexts_own_default_radius() {
    let (gw, gh) = (32usize, 32usize);
    let n = gw * gh;
    let mut res = zero_pots(n);
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            res.salt[(16 + dy) as usize * gw + (16 + dx) as usize] = 1.0;
        }
    }
    let field = vec![1.0f32; n];
    let w = PlaceWorld {
        res: &res,
        field: &field,
        biome: &[],
        rain: &[],
        gw,
        gh,
        sea: 0.42,
        map_width_km: 800.0,
    };
    // The default radius really is 3 here, and it is not the catchment one.
    assert_eq!(js_max(3.0, js_round(gw as f64 / 128.0)) as usize, 3);
    let mean3 = crate::civ_place_resource_context(&res, &field, gw, gh, 0.42, 16, 16, 3, false);
    let mean4 = crate::civ_place_resource_context(&res, &field, gw, gh, 0.42, 16, 16, 4, false);
    assert!(mean3["salt"] > SALT_DEPOSIT_MEAN && mean4["salt"] < SALT_DEPOSIT_MEAN);
    assert_eq!(civ_salt_access(&w, 16, 16, NavKind::None).source, "salt deposit");
}

/// Kills `salt-lake cell clamp removed`. Branch 3 comes from
/// `_umSiteProfile`, which clamps its cell into the grid before indexing;
/// branch 2 comes from `_civPlaceResourceContext`, which does not clamp and
/// bounds-tests each disc cell instead. Both are reproduced, so an
/// out-of-range position reads the corner cell for the lake test while its
/// resource window comes back empty -- an asymmetry no in-range fixture can
/// show.
#[test]
fn salt_lake_cell_is_clamped_while_the_deposit_window_is_not() {
    let (gw, gh) = (8usize, 8usize);
    let n = gw * gh;
    let mut res = zero_pots(n);
    res.salt = vec![1.0f32; n]; // a deposit everywhere, if the window reached
    let field = vec![1.0f32; n];
    let mut biome = vec![crate::BIOME_GRASS; n];
    biome[n - 1] = BIOME_LAKE;
    let mut rain = vec![0.9f32; n];
    rain[n - 1] = 0.1;
    let w = PlaceWorld {
        res: &res,
        field: &field,
        biome: &biome,
        rain: &rain,
        gw,
        gh,
        sea: 0.42,
        map_width_km: 800.0,
    };
    // Far enough out that the whole radius-3 disc misses the grid.
    let out = civ_salt_access(&w, gw + 10, gh + 10, NavKind::None);
    assert_eq!(out, SaltAccess { has: true, source: "salt lake" }, "clamped to (gw-1, gh-1)");
    // In range on the same corner, the deposit branch fires first.
    assert_eq!(civ_salt_access(&w, gw - 1, gh - 1, NavKind::None).source, "salt deposit");
}

/// The reference tries sea, then deposit, then lake, and the first that
/// fires wins. A coastal settlement standing on an arid salt lake with a
/// deposit under it still reports `sea salt`.
#[test]
fn salt_branch_order_is_sea_then_deposit_then_lake() {
    let n = 81;
    let mut res = zero_pots(n);
    res.salt = vec![1.0f32; n];
    let field = vec![1.0f32; n];
    let biome = vec![BIOME_LAKE; n];
    let rain = vec![0.1f32; n];
    let w = |r: &ResourcePotentials, nav: NavKind| {
        civ_salt_access(
            &PlaceWorld {
                res: r,
                field: &field,
                biome: &biome,
                rain: &rain,
                gw: 9,
                gh: 9,
                sea: 0.42,
                map_width_km: 800.0,
            },
            4,
            4,
            nav,
        )
    };
    assert_eq!(w(&res, NavKind::Sea).source, "sea salt");
    assert_eq!(w(&res, NavKind::River).source, "salt deposit");
    // Without the deposit, the same place falls through to the lake.
    let dry = zero_pots(n);
    assert_eq!(w(&dry, NavKind::River).source, "salt lake");
    assert_eq!(w(&dry, NavKind::Stream).source, "salt lake");
}
