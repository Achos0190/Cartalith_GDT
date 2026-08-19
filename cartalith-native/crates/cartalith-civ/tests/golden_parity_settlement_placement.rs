//! Golden-parity tests for the pure, non-DOM-coupled core of
//! `_civIterativeAutoWorld` (reference HTML line ~25336) --
//! `PHASE2_SCOPE.md` milestone 8: land-component labelling, snap seeds
//! onto land then coast, faction assignment by landmass, settlement tier
//! classification, ocean-port detection. Generated from a Node `vm`
//! extraction run against `reference/Cartalith Gen1 v2.10.html` (harness
//! itself is transient, not checked in) that calls the reference's own
//! `_civSnapLand`/`_civSnapCoast`/`_civIsCoastal`/`_civAssignLandmassFactions`
//! directly (wired together via a small harness-only function that mirrors
//! `_civIterativeAutoWorld`'s own candidate-building loop verbatim, since
//! that loop itself is inline, not a standalone callable in the reference)
//! -- not a hand-composed reimplementation of the algorithm itself.
//!
//! Both fixture cases reuse this crate's existing configs (gw/gh/seed/
//! world, `w_iters=12`, `state.tect.seed` not `state.seed`, matching every
//! milestone in this crate). Cross-checked before trusting this data:
//! `field[0..5]` matched `golden_parity_carve.rs`'s `expected_field[0..5]`
//! exactly for both cases on the first extraction attempt, and the
//! extracted `seeds` matched `golden_parity_settlement_suitability.rs`'s
//! own already-verified seed list exactly.
//!
//! **Both fixtures genuinely exercise the multi-capital (K>1 seats)
//! branch of `_civAssignLandmassFactions`** -- not a degenerate all-
//! trivial-single-seat case. Case 0 has candidates on 2 landmasses
//! (2 candidates on one, 1 on the other); with `factionCount=6` spare
//! seats are apportioned and the 2-candidate landmass earns a second
//! seat, triggering the suitability+spacing capital-seeding loop. Case 1
//! has all 5 candidates on a single landmass (world-wrap connects it),
//! which earns seats up to its full candidate count (5), so every
//! candidate becomes its own capital via the same branch. No third,
//! larger fixture was needed (unlike milestone 6's route-corridors case)
//! since these two already exercise the interesting code path.
//!
//! Faction assignment, capital flags, and settlement tier are all
//! categorical/discrete -- checked bit-exact, this crate's standing
//! convention for non-continuous output.

#[derive(Debug, Clone, Copy, PartialEq)]
struct ExpectedPlace {
    x: usize,
    y: usize,
    faction: i32,
    capital: bool,
    kind: cartalith_civ::SettlementKind,
    coastal: bool,
}

fn assert_places_match(
    actual: &[cartalith_civ::SettlementPlacement],
    expected: &[ExpectedPlace],
    label: &str,
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{label}: place count mismatch"
    );
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(a.x, e.x, "{label} place {i}: x mismatch");
        assert_eq!(a.y, e.y, "{label} place {i}: y mismatch");
        assert_eq!(a.faction, e.faction, "{label} place {i}: faction mismatch");
        assert_eq!(a.capital, e.capital, "{label} place {i}: capital mismatch");
        assert_eq!(a.kind, e.kind, "{label} place {i}: kind mismatch");
        assert_eq!(a.coastal, e.coastal, "{label} place {i}: coastal mismatch");
    }
}

#[test]
fn settlement_placement_case_0_region() {
    // case0_region: gw=14 gh=11 seed=24601 world=false
    // seeds (already verified against golden_parity_settlement_suitability.rs):
    //   (4,6,0.801021158695221) (9,3,0.7972438931465149) (4,1,0.7633237838745117)
    // candidates after snap: (7,2,cont0) (9,3,cont1) (6,2,cont0) -- 2 landmasses.
    // factionCount=6, spare seats apportioned: landmass0 (2 candidates) earns a
    // 2nd seat and exercises the multi-capital spacing branch; landmass1 (1
    // candidate) stays single-seat.
    let expected = vec![
        ExpectedPlace {
            x: 7,
            y: 2,
            faction: 1,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 9,
            y: 3,
            faction: 2,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 6,
            y: 2,
            faction: 3,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
    ];

    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!(
        (ws.sea_level - 0.42f64).abs() < 1e-9,
        "sea_level mismatch, harness assumption broken"
    );

    let places = compute_placements(&ws, 14, 11, false, p.map_width_km, p.river_density);
    assert_places_match(&places, &expected, "case0_region");
}

#[test]
fn settlement_placement_case_1_world_wrap() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true
    // All 5 seeds land on ONE connected landmass (world-wrap). factionCount=6
    // > L=1 landmass, so all 5 candidates earn their own seat and every one
    // becomes its own capital -- the K=5 multi-capital spacing branch.
    let expected = vec![
        ExpectedPlace {
            x: 9,
            y: 3,
            faction: 1,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 5,
            y: 8,
            faction: 2,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 8,
            y: 9,
            faction: 3,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 10,
            y: 5,
            faction: 4,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        ExpectedPlace {
            x: 4,
            y: 7,
            faction: 5,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
    ];

    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!(
        (ws.sea_level - 0.42f64).abs() < 1e-9,
        "sea_level mismatch, harness assumption broken"
    );

    let places = compute_placements(&ws, 16, 12, true, p.map_width_km, p.river_density);
    assert_places_match(&places, &expected, "case1_world_wrap");
}

/// Assembles every affordance field milestones 1-7 provide and runs
/// `place_settlements` -- the exact composition
/// `_civIterativeAutoWorld`'s pure core performs (reference lines
/// ~25336-25425), matching `golden_parity_settlement_suitability.rs`'s
/// own `compute_suitability_and_seeds` for the shared upstream fields.
fn compute_placements(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
) -> Vec<cartalith_civ::SettlementPlacement> {
    let wb = cartalith_civ::build_water_bodies(
        &ws.field,
        gw,
        gh,
        ws.sea_level,
        world,
        Some(&ws.rainfall),
    );
    let biome =
        cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = cartalith_civ::build_lithology(
        &ws.field,
        &ws.age_field,
        &ws.volcanic_field,
        &ws.crust_field,
        &ws.resistance_field,
        &ws.rainfall,
        ws.sea_level,
    );
    let soil = cartalith_civ::build_soil_fertility(
        &lithology,
        &ws.temperature,
        &ws.rainfall,
        &soil_slope,
        &ws.age_field,
    );

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(
        &ws.flow_discharge,
        &ws.field,
        gw,
        gh,
        ws.sea_level,
        flow_thresh,
    );
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil,
        &water_access,
        Some(&biome),
        &ws.temperature,
        &ws.field,
        ws.sea_level,
        0.0,
        None,
    );

    let resources = cartalith_civ::build_resource_potentials(
        &lithology,
        Some(&ws.boundary_type),
        Some(&ws.shear_field),
        Some(&ws.flow_discharge),
        Some(&biome),
        &ws.field,
        &ws.rainfall,
        &ws.age_field,
        gw,
        gh,
        ws.sea_level,
        Some(&ws.volcanic_field),
        true,
        false,
    );

    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = cartalith_civ::build_route_corridors(
        &ws.field,
        &raw_slope,
        Some(&ws.flow_discharge),
        gw,
        gh,
        ws.sea_level,
        world,
        flow_thresh,
    );
    let landmass = cartalith_civ::build_landmass_quality(
        &ws.field,
        Some(&carrying_cap),
        gw,
        gh,
        ws.sea_level,
        world,
    );
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, ws.sea_level);
    let flood = cartalith_civ::build_flood_field(
        &ws.field,
        &ws.flow_discharge,
        &raw_slope,
        gw,
        gh,
        ws.sea_level,
    );

    let river_order = cartalith_civ::fresh_river_order(
        &ws.field,
        &ws.flow_discharge,
        gw,
        gh,
        ws.sea_level,
        world,
        river_density,
        map_width_km,
    );

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_order: Some(&river_order),
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };

    let slope_n = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let suit = cartalith_civ::build_settlement_suitability(
        &soil,
        &water_access,
        &carrying_cap,
        &ws.field,
        &slope_n,
        gw,
        gh,
        ws.sea_level,
        Some(&ctx),
    );
    let seeds =
        cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.65, (gw as f64 / 20.0).max(4.0));

    // factionCount = CIV_FACTIONS.length-1 = 7-1 = 6 (reference line 14568:
    // CIV_FACTIONS has 7 entries, index 0 = "Unclaimed").
    cartalith_civ::place_settlements_with_water_edge_snap(
        &seeds,
        &suit,
        &ws.field,
        &wb.classification,
        &wb.fill_level,
        gw,
        gh,
        ws.sea_level,
        world,
        6,
        &flood,
        &ws.flow_discharge,
        flow_thresh,
        map_width_km,
    )
}
