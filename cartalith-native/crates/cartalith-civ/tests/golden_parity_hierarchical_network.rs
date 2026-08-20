//! Golden-parity tests for the civ auto-populate road network's raw
//! topology -- `PHASE2_SCOPE.md` milestone 12: `_civHierarchicalNetwork`
//! (reference HTML line ~21526) and its direct helpers
//! (`_civEnhancedTravelCost` ~20958, `_civRoutingGrid` ~21022,
//! `_civApplySettlementGravity` ~21119), stopping before corridor
//! consolidation/Catmull-Rom smoothing/road-class-and-name emission
//! (reference lines ~21670-21739) -- see `civ_hierarchical_network_topology`'s
//! own doc comment in `src/lib.rs` for why that boundary was drawn.
//!
//! This is the real dependency `_civSeedVillages` needs (`civWays`), NOT
//! `build_road_network` (milestone 11) -- confirmed by reading every real
//! call site of both in the reference: the auto-populate flow
//! (`_civIterativeAutoWorld`, lines ~25581-25680) calls
//! `_civHierarchicalNetwork(places,{})` with EMPTY opts (no
//! `existingWays`) and never calls `_civPreferSeaRoutes` at all -- that
//! function is only used by the separate `_civAutoRoutes` (manual-tool-
//! adjacent) caller. Sea routes (`_civMstRoutes(ports,true)`) are a real,
//! separate, simpler MST with its own new dependencies (current/wind-
//! costed sea edges, sea-lane augmentation, path smoothing) -- its own
//! future milestone, not ported here.
//!
//! Node `vm` harness: fresh per this project's established practice (not
//! checked in). Blocks #1 (2083-14556) + #2 (14562-26720) concatenated,
//! per `golden_parity_settlement_naming.rs`'s own already-documented block
//! boundaries -- `_civHierarchicalNetwork` lives in block #2, well clear of
//! the later urban-morphology block's `function generate` name collision
//! (~30931) that an earlier milestone found and root-caused. `state.tect.seed`
//! (not the dead `state.seed`), `allocate()` with zero arguments (reading
//! global `GW`/`GH` directly -- passing them as call arguments silently
//! produces a full default-resolution generation instead, a real gotcha an
//! earlier milestone's fork hit and documented).
//!
//! `_civHierarchicalNetwork` only *returns* the post-consolidation `ways`
//! (plus `usageCount`/`degreeOf`, which ARE returned unmodified and
//! verified directly here) -- the raw `allEdges` this port's own topology
//! matches was captured by instrumenting the extracted function source,
//! inserting a capture statement immediately before the reference's own
//! `/* Classify, CONSOLIDATE and smooth. ... */` comment (its `allEdges`
//! variable at that exact point is precisely what pass 1-3 produce, before
//! any consolidation touches it).
//!
//! Settlement inputs are the SAME already-verified `(x,y,faction,kind)`
//! triples `golden_parity_settlement_naming.rs`'s own fixture already
//! confirmed correct (all-capital in both cases here) -- not re-derived.
//! `field[0..5]` cross-checked against a direct `generate_terrain` call
//! with identical params (`w_iters=12`, matching every sibling fixture in
//! this crate) before trusting the extraction.
//!
//! Case 0 (`gw=14 gh=11 seed=24601 world=false`, 3 capitals) is a real,
//! meaningful edge case, not a synthetic one: place 1 (9,3) turns out
//! **unreachable** from places 0/2 over the terrain-cost grid -- `degreeOf`
//! comes back `[1,0,1]`, a single MST edge (0-2) is the network's entirety,
//! and the min-degree-fill pass correctly finds `by_dist` empty for place 1
//! (no finite-cost path exists) rather than looping forever or panicking.
//! Case 1 (`gw=16 gh=12 seed=314159 world=true`, 5 capitals) exercises the
//! OTHER edge of the min-degree-fill pass: every capital requires degree 5,
//! but only 4 other places exist, so the fill pass runs to its natural
//! ceiling (every possible pair connected) rather than the requirement --
//! `degreeOf` comes back `[4,4,4,4,4]` and the network becomes the complete
//! graph K5 (10 edges), which also exercises pass 3 (shortcut-detour-relief)
//! finding nothing left to add (every pair already directly connected).
//!
//! Continuous outputs (usage counts are integers, checked exactly; edge
//! topology -- `a`/`b` indices and cell-path sequences -- categorical,
//! checked exactly) -- no float tolerance needed here, unlike the affordance
//! fields upstream.

fn settlement(x: usize, y: usize, faction: i32) -> cartalith_civ::SettlementPlacement {
    cartalith_civ::SettlementPlacement {
        x,
        y,
        suit: 0.0,
        faction,
        capital: true,
        kind: cartalith_civ::SettlementKind::Capital,
        coastal: true,
    }
}

fn affordance_inputs(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
) -> (Vec<u8>, Vec<u8>, Vec<i16>) {
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, river_density, map_width_km);
    (wb.classification, biome, river_order)
}

#[test]
fn hierarchical_network_case_0_region_unreachable_place() {
    // case0_region: gw=14 gh=11 seed=24601 world=false
    // Same (x,y,faction) triples as golden_parity_settlement_naming.rs's
    // case0. Place 1 (9,3) is unreachable from places 0/2 over the
    // terrain-cost grid -- a real, verified property of this fixture, not
    // an artifact of a synthetic test.
    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");
    assert!((ws.field[0] - 0.8640472292900085f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let (water_bodies, biome, river_order) = affordance_inputs(&ws, 14, 11, false, p.map_width_km, p.river_density);
    let places = vec![settlement(7, 2, 1), settlement(9, 3, 2), settlement(6, 2, 3)];

    let net = cartalith_civ::civ_hierarchical_network_topology(
        &places, 14, 11, ws.sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &water_bodies, false, p.map_width_km,
    );

    assert_eq!(net.degree_of, vec![1, 0, 1], "case0: degree_of mismatch");
    assert_eq!(net.edges.len(), 1, "case0: edge count mismatch");
    assert_eq!(net.edges[0].a, 0);
    assert_eq!(net.edges[0].b, 2);
    assert_eq!(net.edges[0].path, vec![35, 34], "case0: edge path mismatch");

    let nonzero: Vec<(usize, u16)> = net.usage_count.iter().enumerate().filter(|&(_, &u)| u != 0).map(|(i, &u)| (i, u)).collect();
    assert_eq!(nonzero, vec![(34, 1), (35, 1)], "case0: usage_count nonzero cells mismatch");
}

#[test]
fn hierarchical_network_case_1_world_wrap_complete_graph() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true
    // Same 5 (x,y,faction) triples as golden_parity_settlement_naming.rs's
    // case1. Every capital requires degree 5 but only 4 other places
    // exist -- the min-degree-fill pass runs to its natural ceiling (the
    // complete graph K5, 10 edges) rather than the unreachable requirement,
    // and pass 3 (shortcut-detour-relief) correctly finds nothing left to
    // add since every pair is already directly connected.
    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");
    assert!((ws.field[0] - 0.2477419376373291f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let (water_bodies, biome, river_order) = affordance_inputs(&ws, 16, 12, true, p.map_width_km, p.river_density);
    let places = vec![settlement(9, 3, 1), settlement(5, 8, 2), settlement(8, 9, 3), settlement(10, 5, 4), settlement(4, 7, 5)];

    let net = cartalith_civ::civ_hierarchical_network_topology(
        &places, 16, 12, ws.sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &water_bodies, true, p.map_width_km,
    );

    assert_eq!(net.degree_of, vec![4, 4, 4, 4, 4], "case1: degree_of mismatch (expected complete graph K5)");
    assert_eq!(net.edges.len(), 10, "case1: edge count mismatch (expected complete graph K5 = 10 edges)");

    let expected_edges: Vec<(usize, usize, Vec<usize>)> = vec![
        (0, 3, vec![57, 74, 90]),
        (3, 2, vec![90, 106, 121, 137, 152]),
        (2, 1, vec![152, 151, 150, 133]),
        (1, 4, vec![133, 116]),
        (0, 2, vec![57, 74, 90, 106, 121, 137, 152]),
        (0, 1, vec![57, 74, 90, 106, 121, 137, 152, 151, 150, 133]),
        (0, 4, vec![57, 74, 90, 106, 121, 137, 152, 151, 150, 133, 116]),
        (1, 3, vec![133, 150, 151, 152, 137, 121, 106, 90]),
        (2, 4, vec![152, 151, 150, 133, 116]),
        (3, 4, vec![90, 106, 121, 137, 152, 151, 150, 133, 116]),
    ];
    for (i, e) in net.edges.iter().enumerate() {
        assert_eq!((e.a, e.b, e.path.clone()), expected_edges[i], "case1: edge {i} mismatch");
    }

    let expected_usage: Vec<(usize, u16)> = vec![
        (57, 4), (74, 4), (90, 7), (106, 6), (116, 4), (121, 6), (133, 7), (137, 6), (150, 6), (151, 6), (152, 8),
    ];
    let mut nonzero: Vec<(usize, u16)> = net.usage_count.iter().enumerate().filter(|&(_, &u)| u != 0).map(|(i, &u)| (i, u)).collect();
    nonzero.sort_by_key(|&(i, _)| i);
    assert_eq!(nonzero, expected_usage, "case1: usage_count nonzero cells mismatch");
}
