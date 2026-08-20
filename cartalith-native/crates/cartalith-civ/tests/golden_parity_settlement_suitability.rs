#![allow(clippy::excessive_precision)]
//! Golden-parity tests for `buildSettlementSuitability`/`findSettlementSeeds`
//! (reference HTML lines 6319/6418) -- `PHASE2_SCOPE.md` milestone 7, the
//! "v1.30 one function" `ROADMAP.md` originally flagged. Generated from a
//! Node `vm` extraction run against `reference/Cartalith Gen1 v2.10.html`
//! (harness itself is transient, not checked in) that calls the reference's
//! own `currentSettlementSuitability()`/`findSettlementSeeds()` directly,
//! not a hand-composed reimplementation.
//!
//! **River-network resolution (this milestone's first real question, per
//! `PHASE2_SCOPE.md`)**: `WorldState.stream_order` is NOT the right input
//! for `riverOrder` here. `cartalith_hydrology::build_channels` is already
//! a line-for-line port of `buildRiverNetwork`'s channelization loop
//! (confirmed via its own doc comment citing reference lines 4503-4522) --
//! the ALGORITHM already matches. But the reference's own
//! `carveRiverValleys()` explicitly nulls `_riverNet` at its very end
//! (reference line 8783: `_riverNet=null`), so `currentSettlementSuitability()`
//! always rebuilds the river network fresh on the FINAL, post-carve
//! `field`/`flowField` the next time anything asks for it -- never reusing
//! whatever was computed mid-carve. `WorldState.stream_order` is computed
//! at that earlier, mid-carve point (before the channel-lock stamp that
//! follows it in `generate_terrain`), so it's stale for this specific
//! caller even though it's a correct value for its own original purpose.
//! Fixed by `fresh_river_order()`, which reuses `build_channels`/
//! `strahler_from_receivers` directly on `ws.field`/`ws.flow_discharge`
//! (the final, fully-carved state) rather than porting a second receiver-
//! tree implementation.
//!
//! Both fixture cases reuse this crate's existing configs (gw/gh/seed/
//! world, `w_iters=12`, matching `golden_parity_carve.rs`/every other
//! milestone in this crate). Cross-checked before trusting this data:
//! this harness's own `field[0..5]` matched `golden_parity_carve.rs`'s
//! `expected_field[0..5]` exactly for both cases, and determinism was
//! independently confirmed by running case 0 twice and diffing byte-for-
//! byte identical JSON output.
//!
//! `buildFloodField` (reference line 5634) had no prior port anywhere in
//! this crate -- a real gap, not assumed away, closed here as
//! `build_flood_field` since `buildSettlementSuitability`'s `ctx.flood`
//! genuinely reads it (not `null`/absent in production, unlike some other
//! optional `ctx` fields). No geoid field exists in this port (`field[i]-
//! geoAt(i)` becomes just `field[i]`, matching `build_water_bodies`'s own
//! established `geo: None` pattern for the same absence).
//!
//! Suitability is continuous `f32` -- `1e-4` tolerance, this crate's
//! standing convention. Seeds are checked both by count and by exact
//! `(x, y, score)` triples in score-descending order (the same suppression-
//! radius greedy algorithm the reference uses, so tie-break order matters).
//!
//! **Threshold: `0.65`, not `SETTLE_SEED_THRESH` (`0.42`).** A real,
//! substantive finding, not an oversight: `findSettlementSeeds` has TWO
//! genuinely different real call sites in the reference. The interactive
//! advisory debug view (reference lines 8461/11517) passes
//! `{thresh: SETTLE_SEED_THRESH}` (0.42) -- but that view doesn't exist
//! anywhere in this port. The `settlement_seeds.json` export (reference
//! line 12445: `findSettlementSeeds(currentSettlementSuitability(),GW,GH)`,
//! no opts) is the only headless/non-interactive real production caller,
//! and this port's own closest analog -- it uses the function's bare
//! internal default, `0.65`. First extraction attempt (before this was
//! understood) used `{thresh: SETTLE_SEED_THRESH}` explicitly and found a
//! genuine mismatch (6 seeds vs. this fixture's 5) even though the
//! suitability field itself was already bit-identical -- root-caused to
//! the wrong threshold, not a formula bug, before trusting either number.
//! `SETTLE_SEED_THRESH` stays ported as a named constant in
//! `cartalith-civ` (it's a real, correctly-valued reference constant) for
//! whenever an interactive advisory view is built in this port -- just not
//! what this golden fixture exercises.

fn assert_close(actual: &[f32], expected: &[f32], label: &str) {
    const ATOL: f32 = 1e-4;
    const RTOL: f32 = 1e-4;
    assert_eq!(actual.len(), expected.len(), "{label}: length mismatch");
    for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
        let tol = ATOL + RTOL * e.abs();
        assert!(
            (a - e).abs() <= tol,
            "{label} index {i}: got {a}, expected {e} (diff {}, tol {tol})",
            (a - e).abs()
        );
    }
}

fn assert_seeds_close(actual: &[cartalith_civ::SettlementSeed], expected: &[(usize, usize, f32)], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: seed count mismatch");
    for (i, (a, &(ex, ey, escore))) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(a.x, ex, "{label} seed {i}: x mismatch");
        assert_eq!(a.y, ey, "{label} seed {i}: y mismatch");
        assert!(
            (a.score - escore).abs() <= 1e-4,
            "{label} seed {i}: score got {}, expected {escore}",
            a.score
        );
    }
}

#[test]
fn settlement_suitability_case_0_region() {
    // case0_region: gw=14 gh=11 seed=24601 world=false
    // CASE0 sea=0.42 flowThresh=0.0616 seedsCount=3
    let expected_suit: Vec<f32> = vec![0.4659339487552643f32, 0.3375842571258545f32, 0.4565603733062744f32, 0.6558908224105835f32, 0.6296071410179138f32, 0.4674827456474304f32, 0.6542048454284668f32, 0.6109026670455933f32, 0.6224120259284973f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.44464123249053955f32, 0.40760141611099243f32, 0.6773497462272644f32, 0.7246860265731812f32, 0.7633237838745117f32, 0.6374057531356812f32, 0.6403632164001465f32, 0.5944005846977234f32, 0.5993370413780212f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.3701127767562866f32, 0.40003034472465515f32, 0.6749517917633057f32, 0.7370033860206604f32, 0.7288358211517334f32, 0.6190601587295532f32, 0.6241977214813232f32, 0.7306162118911743f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.40878555178642273f32, 0.46150800585746765f32, 0.7211224436759949f32, 0.7708563804626465f32, 0.6935853958129883f32, 0.5495160818099976f32, 0f32, 0f32, 0f32, 0.7972438931465149f32, 0f32, 0.457638680934906f32, 0.740919291973114f32, 0f32, 0.5308757424354553f32, 0.5646592974662781f32, 0.7021728157997131f32, 0.6968492269515991f32, 0.6747370362281799f32, 0.6216966509819031f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.4727352559566498f32, 0.5129222869873047f32, 0f32, 0.6029186248779297f32, 0.6250901222229004f32, 0.7722831964492798f32, 0.7853245139122009f32, 0.7570300698280334f32, 0.5855845212936401f32, 0f32, 0f32, 0.38852351903915405f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.6610569953918457f32, 0.7439841032028198f32, 0.794422447681427f32, 0.800812840461731f32, 0.801021158695221f32, 0.6340886354446411f32, 0f32, 0f32, 0.4966275691986084f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.6386557221412659f32, 0.7684215903282166f32, 0.7663425803184509f32, 0.7553013563156128f32, 0.6142658591270447f32, 0.6353833079338074f32, 0.6777088046073914f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7287659645080566f32, 0.7417961955070496f32, 0.7657946348190308f32, 0.5716677904129028f32, 0.6372779011726379f32, 0f32, 0f32, 0.6168814301490784f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7675237655639648f32, 0.7517905235290527f32, 0.7257106304168701f32, 0.5895572900772095f32, 0.5522850155830383f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7185184359550476f32, 0.7432905435562134f32, 0.7543020248413086f32, 0.6079452037811279f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32];
    let expected_seeds: Vec<(usize, usize, f32)> = vec![(4, 6, 0.801021158695221f32), (9, 3, 0.7972438931465149f32), (4, 1, 0.7633237838745117f32)];

    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let (suit, seeds) = compute_suitability_and_seeds(&ws, 14, 11, false, p.map_width_km, p.river_density);
    assert_close(&suit, &expected_suit, "suitability case0_region");
    assert_seeds_close(&seeds, &expected_seeds, "seeds case0_region");
}

#[test]
fn settlement_suitability_case_1_world_wrap() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true
    // CASE1 sea=0.42 flowThresh=0.07680000000000001 seedsCount=5
    let expected_suit: Vec<f32> = vec![0f32, 0f32, 0.4544396996498108f32, 0.6829801797866821f32, 0.6716033816337585f32, 0.7098731398582458f32, 0.7363941669464111f32, 0.8240528106689453f32, 0.7810350656509399f32, 0.5809214115142822f32, 0.6753418445587158f32, 0.6026200652122498f32, 0.5720908045768738f32, 0.5295939445495605f32, 0.7118990421295166f32, 0.5681145787239075f32, 0.4841911792755127f32, 0f32, 0.4294154942035675f32, 0.5891220569610596f32, 0.5186370611190796f32, 0f32, 0.6469618678092957f32, 0.7927274107933044f32, 0.7248112559318542f32, 0f32, 0.618212878704071f32, 0.6563898324966431f32, 0.6516931653022766f32, 0.5132604837417603f32, 0.7425177693367004f32, 0.6197832822799683f32, 0f32, 0.490617573261261f32, 0.43964463472366333f32, 0.38720008730888367f32, 0f32, 0f32, 0.6940781474113464f32, 0.7959432005882263f32, 0.8302209973335266f32, 0.597190797328949f32, 0f32, 0.764442503452301f32, 0.7290124297142029f32, 0.7565300464630127f32, 0.5328009128570557f32, 0f32, 0f32, 0.7213306427001953f32, 0f32, 0f32, 0f32, 0.6953161954879761f32, 0f32, 0.7508717775344849f32, 0.7712700366973877f32, 0.8109160661697388f32, 0.8138898015022278f32, 0.8104649186134338f32, 0.8133845329284668f32, 0.7196357250213623f32, 0f32, 0f32, 0.54398512840271f32, 0.4139922559261322f32, 0f32, 0f32, 0.6010671257972717f32, 0.7966353297233582f32, 0f32, 0.6192946434020996f32, 0.6879088282585144f32, 0f32, 0.6882720589637756f32, 0.877501904964447f32, 0.7467853426933289f32, 0.5740789175033569f32, 0.5160160660743713f32, 0.42500898241996765f32, 0.4940870404243469f32, 0.7125096321105957f32, 0f32, 0f32, 0f32, 0.8474334478378296f32, 0.5898798704147339f32, 0f32, 0.5701712369918823f32, 0f32, 0.7266634702682495f32, 0.7777393460273743f32, 0.6724652647972107f32, 0f32, 0.5727471113204956f32, 0.4766692817211151f32, 0f32, 0.649103045463562f32, 0.5776664018630981f32, 0f32, 0.5558050274848938f32, 0.40618836879730225f32, 0f32, 0f32, 0f32, 0f32, 0.5526706576347351f32, 0.7049570083618164f32, 0f32, 0f32, 0.7597787380218506f32, 0f32, 0f32, 0f32, 0.6265968084335327f32, 0.7486276626586914f32, 0.4574846625328064f32, 0f32, 0f32, 0f32, 0f32, 0.46842536330223083f32, 0f32, 0.7595784068107605f32, 0f32, 0.543763279914856f32, 0.6808619499206543f32, 0f32, 0f32, 0.4536745846271515f32, 0.6735608577728271f32, 0.647739589214325f32, 0.5560805797576904f32, 0.8199706077575684f32, 0.5670826435089111f32, 0f32, 0f32, 0.49764010310173035f32, 0.39544785022735596f32, 0.36451399326324463f32, 0f32, 0.6790796518325806f32, 0.7422176599502563f32, 0f32, 0f32, 0.6350211501121521f32, 0.46249744296073914f32, 0f32, 0.6249449849128723f32, 0.6116637587547302f32, 0.6004941463470459f32, 0.4994843304157257f32, 0.5849137306213379f32, 0.4600963294506073f32, 0.37229734659194946f32, 0.49899518489837646f32, 0.5173693299293518f32, 0f32, 0.6083006858825684f32, 0.46872764825820923f32, 0.5247565507888794f32, 0.6404932737350464f32, 0.44346559047698975f32, 0f32, 0f32, 0.5352204442024231f32, 0.5418827533721924f32, 0.5260804891586304f32, 0.8081161975860596f32, 0f32, 0f32, 0.41098812222480774f32, 0f32, 0f32, 0f32, 0.3890637755393982f32, 0.5443041324615479f32, 0.4659217894077301f32, 0f32, 0.7042208909988403f32, 0.47862139344215393f32, 0.5131184458732605f32, 0.5875740647315979f32, 0.7066688537597656f32, 0.7202219367027283f32, 0f32, 0f32, 0f32, 0f32, 0.5120180249214172f32, 0.6315444707870483f32, 0.5630244016647339f32];
    let expected_seeds: Vec<(usize, usize, f32)> = vec![(11, 4, 0.877501904964447f32), (5, 5, 0.8474334478378296f32), (8, 10, 0.8081161975860596f32), (14, 8, 0.7422176599502563f32), (1, 3, 0.7213306427001953f32)];

    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let (suit, seeds) = compute_suitability_and_seeds(&ws, 16, 12, true, p.map_width_km, p.river_density);
    assert_close(&suit, &expected_suit, "suitability case1_world_wrap");
    assert_seeds_close(&seeds, &expected_seeds, "seeds case1_world_wrap");
}

/// Assembles every affordance field milestones 1-6 provide, resolves a
/// fresh river-order pass (see this file's own doc comment for why
/// `ws.stream_order` isn't the right input), and runs
/// `build_settlement_suitability`/`find_settlement_seeds` -- the exact
/// production composition `currentSettlementSuitability()` performs.
fn compute_suitability_and_seeds(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
) -> (Vec<f32>, Vec<cartalith_civ::SettlementSeed>) {
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level,
    );
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, ws.sea_level, flow_thresh);
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, ws.sea_level, 0.0, None,
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
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, ws.sea_level, world, flow_thresh);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, ws.sea_level, world);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, ws.sea_level);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, ws.sea_level);

    // Fresh river order on the FINAL post-carve field/flow -- see this
    // file's own module doc comment for why `ws.stream_order` is not a
    // substitute.
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, river_density, map_width_km);

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
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &slope_n, gw, gh, ws.sea_level, Some(&ctx));
    // 0.65 = findSettlementSeeds' own bare default, matching the
    // settlement_seeds.json export path (see this file's module doc
    // comment) -- NOT cartalith_civ::SETTLE_SEED_THRESH (0.42), which is
    // the interactive advisory view's override this port doesn't have.
    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.65, (gw as f64 / 20.0).max(4.0));

    (suit, seeds)
}
