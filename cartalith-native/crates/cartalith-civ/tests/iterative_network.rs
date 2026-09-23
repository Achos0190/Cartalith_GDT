//! `civ_iterative_network` -- `_civIterativeAutoWorld`'s centrality -> tier
//! loop (reference v2.11 lines 26099-26123) -- over a real generated world.
//!
//! The per-round rule and the betweenness it reads are golden-verified against
//! the reference's own source in `golden_parity_centrality_feedback.rs`. What
//! this file checks is the part that golden cannot see, because it needs a
//! network: that the place pairs the loop feeds the metrics are exactly the
//! pairs the consolidated ways carry, in the same order; that one pass is
//! byte-identical to the single network build the port made before the loop
//! existed; and that the loop runs `passes - 1` re-tiering rounds, not more or
//! fewer.
//!
//! `world_placements` mirrors `cartalith-godot`'s `compute_civilisation`
//! auto-populate path up to the network call (that function is private to a
//! cdylib and cannot be called from here), with the app's own four
//! `params::defaults()` divergence flags switched on.

use cartalith_civ as civ;

pub struct CivWorld {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub map_width_km: f64,
    pub ws: cartalith_engine::WorldState,
    pub wb: civ::WaterBodies,
    pub biome: Vec<u8>,
    pub river_order: Vec<i16>,
    pub places: Vec<civ::SettlementPlacement>,
}

pub fn world_placements(size: usize, seed: i32) -> CivWorld {
    let mut p = cartalith_engine::WorldParams::defaults(size, size * 3 / 4, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    let (gw, gh, world, mwk) = (p.gw, p.gh, p.world, p.map_width_km);
    let ws = cartalith_engine::generate_terrain(&p);
    let sea = ws.sea_level;
    let wb = civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
    let biome = civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let soil_slope = civ::build_slope_field(&ws.field, gw, gh, world);
    let lith = civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea);
    let soil = civ::build_soil_fertility(&lith, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, mwk);
    let water_access = civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
    let cap = civ::build_carrying_capacity(&soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea, 0.0, None);
    let resources = civ::build_resource_potentials(
        &lith, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome),
        &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), true, false,
    );
    let raw_slope = civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
    let landmass = civ::build_landmass_quality(&ws.field, Some(&cap), gw, gh, sea, world);
    let coast_reach = civ::build_coast_reach(
        &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea), &wb.classification, gw, gh,
    );
    let flood = civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
    let (river_order, river_polys) =
        civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, mwk, ws.integrated_drainage);
    let river_reach = civ::build_river_reach(&river_polys, &river_order, gw, gh);
    let ctx = civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_reach: Some(&river_reach),
        coast_reach: Some(&coast_reach),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };
    let suit = civ::build_settlement_suitability(&soil, &water_access, &cap, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx));
    let seeds = civ::find_settlement_seeds(&suit, gw, gh, p.civ.seed_thresh, (gw as f64 / p.civ.seed_suppress_div).floor().max(6.0));
    let places = civ::place_settlements_with_water_edge_snap(
        &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea, world, p.civ.factions.max(1),
        &flood, &ws.flow_discharge, flow_thresh, mwk,
    );
    CivWorld { gw, gh, world, map_width_km: mwk, ws, wb, biome, river_order, places }
}

impl CivWorld {
    pub fn network(&self, places: &[civ::SettlementPlacement]) -> civ::HierarchicalNetworkResult {
        civ::civ_hierarchical_network_topology(
            places, self.gw, self.gh, self.ws.sea_level, &self.ws.field, &self.ws.flow_discharge,
            &self.river_order, &self.biome, &self.wb.classification, self.world, self.map_width_km,
        )
    }
    pub fn iterate(&self, places: &mut [civ::SettlementPlacement], passes: usize) -> civ::HierarchicalNetworkResult {
        civ::civ_iterative_network(
            places, passes, self.gw, self.gh, self.ws.sea_level, &self.ws.field, &self.ws.flow_discharge,
            &self.river_order, &self.biome, &self.wb.classification, self.world, self.map_width_km,
        )
    }
}

fn same_net(a: &civ::HierarchicalNetworkResult, b: &civ::HierarchicalNetworkResult) -> bool {
    a.usage_count == b.usage_count
        && a.degree_of == b.degree_of
        && a.edges.len() == b.edges.len()
        && a.edges.iter().zip(&b.edges).all(|(x, y)| x.a == y.a && x.b == y.b && x.path == y.path)
}

#[test]
fn way_pairs_are_the_consolidated_ways_pairs_in_order() {
    let w = world_placements(256, 12345);
    assert!(w.places.len() >= 8, "fixture too small: {} places", w.places.len());
    let net = w.network(&w.places);
    let named: Vec<civ::NamedSettlement> = w
        .places
        .iter()
        .map(|&placement| civ::NamedSettlement { tid: 0, placement, name: String::new(), pop: 0 })
        .collect();
    let ways = civ::civ_consolidate_and_smooth_ways(&net, &named, &w.ws.field, &w.wb.classification, w.gw, w.gh, w.map_width_km);
    // Every way the metrics read, in order, with each edge's run of ways
    // collapsed to one entry -- a repeat is a no-op in the adjacency `Set`.
    let mut from_ways: Vec<(usize, usize)> = Vec::new();
    for way in &ways {
        assert!(way.pts.len() >= 2, "a way the metrics would skip");
        if from_ways.last() != Some(&(way.a_idx, way.b_idx)) {
            from_ways.push((way.a_idx, way.b_idx));
        }
    }
    let pairs = civ::civ_network_way_pairs(&net);
    assert_eq!(pairs.len(), net.edges.len());
    assert_eq!(from_ways, pairs);
    // And the order is not the raw edge order on this world, so the check
    // above would catch a pairs function that skipped the sort.
    let raw: Vec<(usize, usize)> = net.edges.iter().map(|e| (e.a, e.b)).collect();
    assert_ne!(raw, pairs, "fixture does not reorder edges; pick another seed");
}

#[test]
fn one_pass_is_the_single_network_build() {
    let w = world_placements(256, 12345);
    let before = w.places.clone();
    let single = w.network(&w.places);
    let mut places = w.places.clone();
    let looped = w.iterate(&mut places, 1);
    assert!(same_net(&single, &looped));
    assert_eq!(places, before, "one pass must not re-tier");
    // `passes == 0` is clamped to one pass, not three.
    let mut places0 = w.places.clone();
    assert!(same_net(&single, &w.iterate(&mut places0, 0)));
    assert_eq!(places0, before);
}

#[test]
fn three_passes_run_two_feedback_rounds() {
    let w = world_placements(256, 12345);
    // By hand: network, re-tier, network, re-tier, network.
    let mut by_hand = w.places.clone();
    for _ in 0..2 {
        let net = w.network(&by_hand);
        let btw = civ::civ_network_betweenness(by_hand.len(), &civ::civ_network_way_pairs(&net));
        civ::civ_centrality_tier_feedback(&mut by_hand, &btw);
    }
    let last = w.network(&by_hand);

    let mut places = w.places.clone();
    let net = w.iterate(&mut places, civ::CIV_AUTO_WORLD_PASSES);
    assert_eq!(civ::CIV_AUTO_WORLD_PASSES, 3);
    assert_eq!(places, by_hand);
    assert!(same_net(&net, &last));

    // Each round is observable on this world, so an off-by-one in the loop
    // count cannot pass the equality above by coincidence.
    let mut two = w.places.clone();
    w.iterate(&mut two, 2);
    assert_ne!(two, w.places, "round 1 moved nothing on this fixture");
    assert_ne!(two, places, "round 2 moved nothing on this fixture");
    // Only `kind` moves; position, faction, seat flag and port flag do not.
    for (a, b) in places.iter().zip(&w.places) {
        assert_eq!((a.x, a.y, a.faction, a.capital, a.coastal), (b.x, b.y, b.faction, b.capital, b.coastal));
    }
}

/// The re-baseline this loop is, measured: per world, the tier histogram the
/// port produced before the loop existed (one pass) against the one it
/// produces now (`CIV_AUTO_WORLD_PASSES`), plus how many seats changed kind.
/// `cargo test -p cartalith-civ --test iterative_network -- --ignored --nocapture`
#[test]
#[ignore = "measurement, prints a table"]
fn measure_tier_shift() {
    use civ::SettlementKind::*;
    let hist = |ps: &[civ::SettlementPlacement]| {
        [Capital, City, Town, Village, Hamlet].map(|k| ps.iter().filter(|p| p.kind == k).count())
    };
    println!("size seed | places | before C/Ci/T/V/H | after C/Ci/T/V/H | moved | seat demoted | promoted to capital | edges before->after");
    for size in [256usize, 384, 512] {
        for seed in [12345, 24601, 314159, 7, 99991] {
            let w = world_placements(size, seed);
            let before_net = w.network(&w.places);
            let mut after = w.places.clone();
            let after_net = w.iterate(&mut after, civ::CIV_AUTO_WORLD_PASSES);
            let moved = after.iter().zip(&w.places).filter(|(a, b)| a.kind != b.kind).count();
            let seat_demoted = after.iter().filter(|p| p.capital && p.kind != Capital).count();
            let new_cap = after.iter().filter(|p| !p.capital && p.kind == Capital).count();
            println!(
                "{size} {seed} | {} | {:?} | {:?} | {moved} | {seat_demoted} | {new_cap} | {}->{}",
                w.places.len(), hist(&w.places), hist(&after), before_net.edges.len(), after_net.edges.len()
            );
        }
    }
}
