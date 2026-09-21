#![allow(clippy::excessive_precision)]
//! Golden-parity tests for `buildSettlementSuitability`/`findSettlementSeeds`
//! (reference HTML lines 6319/6418) -- `PHASE2_SCOPE.md` milestone 7, the
//! "v1.30 one function" `ROADMAP.md` originally flagged. Generated from a
//! Node `vm` extraction run against `reference/Cartalith Gen1 v2.10.html`
//! (harness itself is transient, not checked in) that calls the reference's
//! own `currentSettlementSuitability()`/`findSettlementSeeds()` directly,
//! not a hand-composed reimplementation.
//!
//! # Ruling N: this file holds TWO expected sets, and both are asserted
//!
//! **`LARGE_ITEM_RULINGS.md`, 2026-09-20 — a deliberate, owner-ruled
//! divergence from the reference, in TWO terms, landed in two passes.**
//!
//! - **The river term** used to be `riverOrder[i]`'s Strahler ladder plus a
//!   `flow[i]` local-maximum bonus, both sampled at the settlement's own cell
//!   — a proxy that scored full river marks for locally-high flow
//!   accumulation belonging to no connected waterway. It is now
//!   `build_river_reach`: proximity to a real traced river polyline, with the
//!   reference's own ladder read at that river rather than at the querying
//!   cell.
//! - **The coastal term** (added when EF-6's coastline tracer unblocked it)
//!   used to be `1 - (-coast_sdf[i])/5`: `build_coast_sdf` is distance to the
//!   nearest water cell **of any kind**, so a mountain tarn five cells away
//!   scored the same coastal weight the sea would, and the separate `lake`
//!   term collected on the same cell again. It is now `build_coast_reach`:
//!   proximity to a real traced *ocean* coastline.
//!
//! So each test below carries:
//!
//! - **`reference_suit` / `reference_seeds`** — the reference engine's own
//!   captured output, unchanged and still asserted. The test rebuilds both
//!   *removed* proxies (`legacy_river_proxy` and `legacy_coast_proxy`, the
//!   only copies of them left in the tree) and feeds them through the
//!   *unchanged* `build_settlement_suitability` via `ctx.river_reach` /
//!   `ctx.coast_reach`. Those numbers come back. That is what makes this a
//!   two-term change rather than an unnoticed drift somewhere else in the
//!   function, and it is why the reference's numbers were not simply
//!   overwritten.
//! - **`expected_suit` / `expected_seeds`** — what the production composition
//!   produces now. Not the reference's, and labelled as such.
//!
//! A third assertion demands the two arms actually differ, so a mistake that
//! quietly made them the same function could not pass both.
//!
//! What moved, measured over these two fixtures: the river pass moved every
//! scored land cell in case 0 (79 of 79) and 127 of 139 in case 1, by at most
//! 0.100 and 0.064. **Corrected 2026-09-20 by an adversarial verifier** — the
//! coastal pass's own increment on top of that is 27 of 79 (case 0) and 74 of
//! 127 (case 1), all upward in case 0 and all downward in case 1 (max
//! -0.207); comparing the fully-legacy arm against the fully-production arm
//! (both terms changed) moves 79 of 79 and 127 of 127, which is the combined
//! figure, not the coastal pass's own. Note these fixtures are 14x11 and
//! 16x12 cells, where the five
//! cells of `SUIT_RIVER_REACH_CELLS`/`SUIT_COAST_REACH_CELLS` span a third of
//! the map — the fraction of cells that move is a property of the fixture
//! size, not of a production world. The measurements that justify the coastal
//! change are on real worlds and live at `build_coast_reach`'s own doc
//! comment, not here.
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
    let reference_suit: Vec<f32> = vec![0.4659339487552643f32, 0.3375842571258545f32, 0.4565603733062744f32, 0.6558908224105835f32, 0.6296071410179138f32, 0.4674827456474304f32, 0.6542048454284668f32, 0.6109026670455933f32, 0.6224120259284973f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.44464123249053955f32, 0.40760141611099243f32, 0.6773497462272644f32, 0.7246860265731812f32, 0.7633237838745117f32, 0.6374057531356812f32, 0.6403632164001465f32, 0.5944005846977234f32, 0.5993370413780212f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.3701127767562866f32, 0.40003034472465515f32, 0.6749517917633057f32, 0.7370033860206604f32, 0.7288358211517334f32, 0.6190601587295532f32, 0.6241977214813232f32, 0.7306162118911743f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.40878555178642273f32, 0.46150800585746765f32, 0.7211224436759949f32, 0.7708563804626465f32, 0.6935853958129883f32, 0.5495160818099976f32, 0f32, 0f32, 0f32, 0.7972438931465149f32, 0f32, 0.457638680934906f32, 0.740919291973114f32, 0f32, 0.5308757424354553f32, 0.5646592974662781f32, 0.7021728157997131f32, 0.6968492269515991f32, 0.6747370362281799f32, 0.6216966509819031f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.4727352559566498f32, 0.5129222869873047f32, 0f32, 0.6029186248779297f32, 0.6250901222229004f32, 0.7722831964492798f32, 0.7853245139122009f32, 0.7570300698280334f32, 0.5855845212936401f32, 0f32, 0f32, 0.38852351903915405f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.6610569953918457f32, 0.7439841032028198f32, 0.794422447681427f32, 0.800812840461731f32, 0.801021158695221f32, 0.6340886354446411f32, 0f32, 0f32, 0.4966275691986084f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.6386557221412659f32, 0.7684215903282166f32, 0.7663425803184509f32, 0.7553013563156128f32, 0.6142658591270447f32, 0.6353833079338074f32, 0.6777088046073914f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7287659645080566f32, 0.7417961955070496f32, 0.7657946348190308f32, 0.5716677904129028f32, 0.6372779011726379f32, 0f32, 0f32, 0.6168814301490784f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7675237655639648f32, 0.7517905235290527f32, 0.7257106304168701f32, 0.5895572900772095f32, 0.5522850155830383f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0.7185184359550476f32, 0.7432905435562134f32, 0.7543020248413086f32, 0.6079452037811279f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32];
    let reference_seeds: Vec<(usize, usize, f32)> = vec![(4, 6, 0.801021158695221f32), (9, 3, 0.7972438931465149f32), (4, 1, 0.7633237838745117f32)];
    // Ruling N re-baseline -- the river and coastal terms; see this file's header.
    let expected_suit: Vec<f32> = vec![0.4586566f32, 0.3357084f32, 0.6008115f32, 0.67431873f32, 0.6565822f32, 0.48056224f32, 0.5541799f32, 0.6089041f32, 0.60849696f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.44256794f32, 0.48983854f32, 0.6534472f32, 0.76371735f32, 0.78814656f32, 0.66972065f32, 0.6664071f32, 0.60529196f32, 0.58823913f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.36331394f32, 0.5495262f32, 0.7220843f32, 0.7344663f32, 0.7771482f32, 0.678298f32, 0.67240715f32, 0.75264883f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.39465463f32, 0.61965126f32, 0.72191447f32, 0.7723367f32, 0.6472414f32, 0.49717894f32, 0.0f32, 0.0f32, 0.0f32, 0.78189725f32, 0.0f32, 0.4061625f32, 0.6986236f32, 0.0f32, 0.51620936f32, 0.7064824f32, 0.7466559f32, 0.73962635f32, 0.67657787f32, 0.6236702f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.4208787f32, 0.4605076f32, 0.0f32, 0.60090584f32, 0.74949604f32, 0.79417557f32, 0.80408245f32, 0.78232473f32, 0.6197628f32, 0.0f32, 0.0f32, 0.37764978f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.7288633f32, 0.7199676f32, 0.80585414f32, 0.80178666f32, 0.8050072f32, 0.6399156f32, 0.0f32, 0.0f32, 0.4664444f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.75874966f32, 0.7836527f32, 0.73204756f32, 0.7144487f32, 0.61227363f32, 0.6285743f32, 0.6620771f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.7257549f32, 0.7466987f32, 0.7642847f32, 0.5196546f32, 0.58748275f32, 0.0f32, 0.0f32, 0.5907659f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.76153725f32, 0.75141454f32, 0.72403544f32, 0.58752316f32, 0.49997684f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.71442807f32, 0.74168444f32, 0.7133458f32, 0.5569229f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32];
    let expected_seeds: Vec<(usize, usize, f32)> = vec![(2, 6, 0.80585414f32), (4, 1, 0.78814656f32), (9, 3, 0.78189725f32), (6, 7, 0.6620771f32)];

    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    // The reference's own capture, still asserted: feed the *removed* proxy
    // through the *unchanged* function and the reference numbers come back.
    // That is what makes the re-baseline above a river-term change and not a
    // silent drift in some other term.
    let (legacy, legacy_seeds) =
        compute_suitability_and_seeds(&ws, 14, 11, false, p.map_width_km, p.river_density, RiverTerm::LegacyProxy, CoastTerm::LegacySdf);
    assert_close(&legacy, &reference_suit, "legacy-proxy suitability case0_region");
    assert_seeds_close(&legacy_seeds, &reference_seeds, "legacy-proxy seeds case0_region");

    let (suit, seeds) =
        compute_suitability_and_seeds(&ws, 14, 11, false, p.map_width_km, p.river_density, RiverTerm::Real, CoastTerm::Real);
    assert_close(&suit, &expected_suit, "suitability case0_region");
    assert_seeds_close(&seeds, &expected_seeds, "seeds case0_region");

    // ... and the two arms must really differ, or the pair above proves
    // nothing about either.
    let moved = suit.iter().zip(legacy.iter()).filter(|(a, b)| (**a - **b).abs() > 1e-4).count();
    assert!(moved > 0, "case0_region: the re-baseline moved no cell at all");
}

#[test]
fn settlement_suitability_case_1_world_wrap() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true
    // CASE1 sea=0.42 flowThresh=0.07680000000000001 seedsCount=5
    let reference_suit: Vec<f32> = vec![0f32, 0f32, 0.4544396996498108f32, 0.6829801797866821f32, 0.6716033816337585f32, 0.7098731398582458f32, 0.7363941669464111f32, 0.8240528106689453f32, 0.7810350656509399f32, 0.5809214115142822f32, 0.6753418445587158f32, 0.6026200652122498f32, 0.5720908045768738f32, 0.5295939445495605f32, 0.7118990421295166f32, 0.5681145787239075f32, 0.4841911792755127f32, 0f32, 0.4294154942035675f32, 0.5891220569610596f32, 0.5186370611190796f32, 0f32, 0.6469618678092957f32, 0.7927274107933044f32, 0.7248112559318542f32, 0f32, 0.618212878704071f32, 0.6563898324966431f32, 0.6516931653022766f32, 0.5132604837417603f32, 0.7425177693367004f32, 0.6197832822799683f32, 0f32, 0.490617573261261f32, 0.43964463472366333f32, 0.38720008730888367f32, 0f32, 0f32, 0.6940781474113464f32, 0.7959432005882263f32, 0.8302209973335266f32, 0.597190797328949f32, 0f32, 0.764442503452301f32, 0.7290124297142029f32, 0.7565300464630127f32, 0.5328009128570557f32, 0f32, 0f32, 0.7213306427001953f32, 0f32, 0f32, 0f32, 0.6953161954879761f32, 0f32, 0.7508717775344849f32, 0.7712700366973877f32, 0.8109160661697388f32, 0.8138898015022278f32, 0.8104649186134338f32, 0.8133845329284668f32, 0.7196357250213623f32, 0f32, 0f32, 0.54398512840271f32, 0.4139922559261322f32, 0f32, 0f32, 0.6010671257972717f32, 0.7966353297233582f32, 0f32, 0.6192946434020996f32, 0.6879088282585144f32, 0f32, 0.6882720589637756f32, 0.877501904964447f32, 0.7467853426933289f32, 0.5740789175033569f32, 0.5160160660743713f32, 0.42500898241996765f32, 0.4940870404243469f32, 0.7125096321105957f32, 0f32, 0f32, 0f32, 0.8474334478378296f32, 0.5898798704147339f32, 0f32, 0.5701712369918823f32, 0f32, 0.7266634702682495f32, 0.7777393460273743f32, 0.6724652647972107f32, 0f32, 0.5727471113204956f32, 0.4766692817211151f32, 0f32, 0.649103045463562f32, 0.5776664018630981f32, 0f32, 0.5558050274848938f32, 0.40618836879730225f32, 0f32, 0f32, 0f32, 0f32, 0.5526706576347351f32, 0.7049570083618164f32, 0f32, 0f32, 0.7597787380218506f32, 0f32, 0f32, 0f32, 0.6265968084335327f32, 0.7486276626586914f32, 0.4574846625328064f32, 0f32, 0f32, 0f32, 0f32, 0.46842536330223083f32, 0f32, 0.7595784068107605f32, 0f32, 0.543763279914856f32, 0.6808619499206543f32, 0f32, 0f32, 0.4536745846271515f32, 0.6735608577728271f32, 0.647739589214325f32, 0.5560805797576904f32, 0.8199706077575684f32, 0.5670826435089111f32, 0f32, 0f32, 0.49764010310173035f32, 0.39544785022735596f32, 0.36451399326324463f32, 0f32, 0.6790796518325806f32, 0.7422176599502563f32, 0f32, 0f32, 0.6350211501121521f32, 0.46249744296073914f32, 0f32, 0.6249449849128723f32, 0.6116637587547302f32, 0.6004941463470459f32, 0.4994843304157257f32, 0.5849137306213379f32, 0.4600963294506073f32, 0.37229734659194946f32, 0.49899518489837646f32, 0.5173693299293518f32, 0f32, 0.6083006858825684f32, 0.46872764825820923f32, 0.5247565507888794f32, 0.6404932737350464f32, 0.44346559047698975f32, 0f32, 0f32, 0.5352204442024231f32, 0.5418827533721924f32, 0.5260804891586304f32, 0.8081161975860596f32, 0f32, 0f32, 0.41098812222480774f32, 0f32, 0f32, 0f32, 0.3890637755393982f32, 0.5443041324615479f32, 0.4659217894077301f32, 0f32, 0.7042208909988403f32, 0.47862139344215393f32, 0.5131184458732605f32, 0.5875740647315979f32, 0.7066688537597656f32, 0.7202219367027283f32, 0f32, 0f32, 0f32, 0f32, 0.5120180249214172f32, 0.6315444707870483f32, 0.5630244016647339f32];
    let reference_seeds: Vec<(usize, usize, f32)> = vec![(11, 4, 0.877501904964447f32), (5, 5, 0.8474334478378296f32), (8, 10, 0.8081161975860596f32), (14, 8, 0.7422176599502563f32), (1, 3, 0.7213306427001953f32)];
    // RE-BASELINED 2026-09-21 (`LARGE_ITEM_RULINGS.md`'s Ruling Q),
    // superseding the Ruling N re-baseline this comment used to describe
    // alone. `build_water_bodies` moved to a topology-primary ocean/lake
    // rule (`golden_parity_waterbodies.rs`'s header has the full account),
    // which reaches this arm through `carrying_cap`/`resources`/`landmass`/
    // `ctx.water_bodies`/`ctx.coast_reach` -- five real inputs, not one term
    // -- so `expected_suit`/`expected_seeds` move again on top of Ruling
    // N's original river/coastal re-baseline. `reference_suit`/
    // `reference_seeds` above are UNCHANGED and still match the true JS
    // capture: the legacy arm now runs `legacy_build_water_bodies` (this
    // file's own frozen pre-Ruling-Q reproduction) specifically so it can
    // keep proving that. Every value below is
    // `build_settlement_suitability`/`find_settlement_seeds`'s own actual
    // production output on this fixture, captured 2026-09-21.
    let expected_suit: Vec<f32> = vec![0.0f32, 0.0f32, 0.42552647f32, 0.66747874f32, 0.6651142f32, 0.68680036f32, 0.67455703f32, 0.67592025f32, 0.59927183f32, 0.38030753f32, 0.50881064f32, 0.3976645f32, 0.30968052f32, 0.3867283f32, 0.55167115f32, 0.36803934f32, 0.39287186f32, 0.0f32, 0.40289035f32, 0.57481825f32, 0.5165397f32, 0.0f32, 0.0f32, 0.62232554f32, 0.52997756f32, 0.0f32, 0.43954918f32, 0.50473374f32, 0.44890153f32, 0.36002466f32, 0.58276504f32, 0.41453344f32, 0.0f32, 0.45087546f32, 0.41294494f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.6254461f32, 0.67214835f32, 0.3882614f32, 0.0f32, 0.6111694f32, 0.51010305f32, 0.600797f32, 0.34001425f32, 0.0f32, 0.0f32, 0.6868142f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.55823225f32, 0.57561517f32, 0.6357983f32, 0.6794996f32, 0.65002525f32, 0.6039985f32, 0.5271661f32, 0.0f32, 0.0f32, 0.49159992f32, 0.37031132f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.39552596f32, 0.4625225f32, 0.0f32, 0.0f32, 0.7502052f32, 0.68582547f32, 0.49867266f32, 0.42578307f32, 0.22103882f32, 0.4393851f32, 0.6666695f32, 0.0f32, 0.0f32, 0.0f32, 0.7165498f32, 0.0f32, 0.0f32, 0.49438056f32, 0.0f32, 0.67950785f32, 0.7288801f32, 0.62979054f32, 0.0f32, 0.51581407f32, 0.39508066f32, 0.0f32, 0.5581752f32, 0.4738414f32, 0.0f32, 0.4789173f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.7034937f32, 0.0f32, 0.0f32, 0.7333619f32, 0.0f32, 0.0f32, 0.0f32, 0.57008237f32, 0.7034162f32, 0.39091417f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.72801304f32, 0.0f32, 0.5024244f32, 0.6397641f32, 0.0f32, 0.0f32, 0.40872702f32, 0.64212865f32, 0.62447006f32, 0.53406274f32, 0.7888718f32, 0.5278393f32, 0.0f32, 0.0f32, 0.45194003f32, 0.3464979f32, 0.3173829f32, 0.0f32, 0.63170666f32, 0.7000481f32, 0.0f32, 0.0f32, 0.59580463f32, 0.4334789f32, 0.0f32, 0.6180582f32, 0.59778094f32, 0.58869755f32, 0.4818179f32, 0.5561618f32, 0.41927278f32, 0.32467404f32, 0.4466985f32, 0.42494777f32, 0.0f32, 0.557291f32, 0.41696307f32, 0.47230518f32, 0.602982f32, 0.41670638f32, 0.0f32, 0.0f32, 0.48278567f32, 0.53979677f32, 0.5114022f32, 0.79061157f32, 0.0f32, 0.0f32, 0.32478186f32, 0.0f32, 0.0f32, 0.0f32, 0.34045902f32, 0.49192134f32, 0.42498824f32, 0.0f32, 0.6892856f32, 0.47132075f32, 0.51101965f32, 0.58046174f32, 0.648069f32, 0.65253115f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.4109793f32, 0.5326655f32, 0.46143815f32];
    let expected_seeds: Vec<(usize, usize, f32)> = vec![(8, 10, 0.79061157f32), (11, 4, 0.7502052f32), (5, 5, 0.7165498f32), (14, 8, 0.7000481f32), (1, 3, 0.6868142f32)];

    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    // The reference's own capture, still asserted: feed the *removed* proxy
    // through the *unchanged* function and the reference numbers come back.
    // That is what makes the re-baseline above a river-term change and not a
    // silent drift in some other term.
    let (legacy, legacy_seeds) =
        compute_suitability_and_seeds(&ws, 16, 12, true, p.map_width_km, p.river_density, RiverTerm::LegacyProxy, CoastTerm::LegacySdf);
    assert_close(&legacy, &reference_suit, "legacy-proxy suitability case1_world_wrap");
    assert_seeds_close(&legacy_seeds, &reference_seeds, "legacy-proxy seeds case1_world_wrap");

    let (suit, seeds) =
        compute_suitability_and_seeds(&ws, 16, 12, true, p.map_width_km, p.river_density, RiverTerm::Real, CoastTerm::Real);
    assert_close(&suit, &expected_suit, "suitability case1_world_wrap");
    assert_seeds_close(&seeds, &expected_seeds, "seeds case1_world_wrap");

    // ... and the two arms must really differ, or the pair above proves
    // nothing about either.
    let moved = suit.iter().zip(legacy.iter()).filter(|(a, b)| (**a - **b).abs() > 1e-4).count();
    assert!(moved > 0, "case1_world_wrap: the re-baseline moved no cell at all");
}

/// Assembles every affordance field milestones 1-6 provide, resolves a
/// fresh river-order pass (see this file's own doc comment for why
/// `ws.stream_order` isn't the right input), and runs
/// `build_settlement_suitability`/`find_settlement_seeds` -- the exact
/// production composition `currentSettlementSuitability()` performs.
#[derive(Clone, Copy, PartialEq)]
enum RiverTerm {
    /// Ruling N: `build_river_reach` over the traced polylines.
    Real,
    /// What the term was before Ruling N, reproduced here and nowhere else in
    /// the tree -- see [`legacy_river_proxy`].
    LegacyProxy,
}

#[derive(Clone, Copy, PartialEq)]
enum CoastTerm {
    /// Ruling N: `build_coast_reach` over the traced ocean coastline.
    Real,
    /// What the term was before Ruling N, reproduced here and nowhere else in
    /// the tree -- see [`legacy_coast_proxy`].
    LegacySdf,
}

/// `buildSettlementSuitability`'s river term exactly as the reference writes
/// it (HTML lines 6345-6352): the Strahler ladder at the settlement's own
/// cell, plus a `flow[i] > thresh*2` local-maximum bonus, clamped to 1.
///
/// Rendered as a raster so it can be handed to the *unchanged* function
/// through `ctx.river_reach`. This is the only copy of the removed code in
/// the tree, and it exists for exactly one reason: without it the
/// re-baselined arrays above would be this port asserting its own output,
/// with the reference's numbers no longer checked by anything.
fn legacy_river_proxy(flow: &[f32], order: &[i16], gw: usize, gh: usize, flow_thresh: f64) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![0f32; n];
    for (i, o) in out.iter_mut().enumerate() {
        let (x, y) = (i % gw, i / gw);
        let oo = order[i];
        let mut river: f64 = if oo >= 4 {
            1.0
        } else if oo >= 3 {
            0.7
        } else if oo >= 2 {
            0.3
        } else {
            0.0
        };
        if flow[i] as f64 > flow_thresh * 2.0 {
            let mut is_max = true;
            'nb: for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let (nx, ny) = (x as isize + dx, y as isize + dy);
                    if nx < 0 || nx >= gw as isize || ny < 0 || ny >= gh as isize {
                        continue;
                    }
                    if flow[ny as usize * gw + nx as usize] > flow[i] {
                        is_max = false;
                        break 'nb;
                    }
                }
            }
            let bonus = if is_max {
                0.5
            } else if flow[i] as f64 > flow_thresh * 5.0 {
                0.25
            } else {
                0.0
            };
            river = (river + bonus).min(1.0);
        }
        *o = river as f32;
    }
    out
}

/// `buildSettlementSuitability`'s coastal term exactly as the reference
/// writes it (HTML line 6338): `1 - dist/5` over `buildCoastSDF`'s distance
/// to the nearest water cell, clamped at zero.
///
/// The estuary bonus that follows it in the reference is NOT reproduced here:
/// it survived Ruling N unchanged and still lives inside
/// `build_settlement_suitability`, which applies it to whatever this raster
/// hands over. Adding it here too would double it.
///
/// Rendered as a raster so it can be handed to the *unchanged* function
/// through `ctx.coast_reach`, the same way `legacy_river_proxy` is. This is
/// the only copy of the removed code in the tree.
fn legacy_coast_proxy(coast_sdf: &[f32]) -> Vec<f32> {
    coast_sdf
        .iter()
        .map(|&sdf| {
            let dist = -(sdf as f64);
            if dist >= 0.0 { (1.0 - dist / 5.0).max(0.0) as f32 } else { 0.0 }
        })
        .collect()
}

// ---- legacy_water_bodies: `build_water_bodies` BEFORE Ruling Q ----
//
// `LARGE_ITEM_RULINGS.md`'s Ruling Q (2026-09-21) moved
// `cartalith_civ::build_water_bodies` from the reference's own size-primary
// ocean/lake rule to a topology-primary one. That function is now a
// DELIBERATE DIVERGENCE from the reference -- and `carrying_cap`/
// `resources`/`landmass`/`ctx.water_bodies` all read its output, so even
// the "legacy" river/coast arm below (which is supposed to reproduce
// `reference_suit`/`reference_seeds`, the JS engine's own real capture,
// bit-for-bit) would silently drift once `build_water_bodies` itself
// stopped matching the reference. This is the SAME reasoning that already
// justifies `legacy_river_proxy`/`legacy_coast_proxy`, extended to a third
// term Ruling Q's own re-baseline reached into: without a frozen
// reproduction of the pre-Ruling-Q algorithm, the "legacy" arm would be
// asserting this port's own new output against itself, not against the
// reference -- and the isolation this file exists to prove (that Ruling
// N's two terms are the ONLY thing that moved) would no longer hold.
//
// A faithful, hand-verified copy of `build_water_bodies` as it stood
// immediately before Ruling Q (same connected-components pass, same
// largest-component-wins ocean rule, same priority-flood depression pass
// -- including the hand-ported `MinHeap`, whose own doc comment in
// `src/lib.rs` warns that a `std::collections::BinaryHeap` substitute is
// not safe here because equal-priority pop order decides lake shape).
// Returns just the classification, which is all this file's `wb`/`biome`
// derivation needs.

struct LegacyMinHeap {
    p: Vec<f32>,
    v: Vec<usize>,
}

impl LegacyMinHeap {
    fn with_capacity(cap: usize) -> Self {
        LegacyMinHeap { p: Vec::with_capacity(cap), v: Vec::with_capacity(cap) }
    }
    fn size(&self) -> usize {
        self.p.len()
    }
    fn push(&mut self, pr: f32, va: usize) {
        self.p.push(pr);
        self.v.push(va);
        let mut i = self.p.len() - 1;
        while i > 0 {
            let pa = (i - 1) / 2;
            if self.p[pa] <= self.p[i] {
                break;
            }
            self.p.swap(pa, i);
            self.v.swap(pa, i);
            i = pa;
        }
    }
    fn pop(&mut self) -> usize {
        let rv = self.v[0];
        let last = self.p.len() - 1;
        if last > 0 {
            self.p[0] = self.p[last];
            self.v[0] = self.v[last];
        }
        self.p.pop();
        self.v.pop();
        let m = self.p.len();
        let mut i = 0usize;
        loop {
            let l = 2 * i + 1;
            let r = 2 * i + 2;
            let mut s = i;
            if l < m && self.p[l] < self.p[s] {
                s = l;
            }
            if r < m && self.p[r] < self.p[s] {
                s = r;
            }
            if s == i {
                break;
            }
            self.p.swap(s, i);
            self.v.swap(s, i);
            i = s;
        }
        rv
    }
}

fn legacy_wb_seed(i: usize, filled: &[f32], done: &mut [bool], heap: &mut LegacyMinHeap) {
    if !done[i] {
        done[i] = true;
        heap.push(filled[i], i);
    }
}

#[allow(clippy::too_many_arguments)]
fn legacy_wb_visit(
    nx: isize,
    ny: isize,
    cur: f64,
    gw: isize,
    gh: isize,
    world: bool,
    filled: &mut [f32],
    done: &mut [bool],
    heap: &mut LegacyMinHeap,
) {
    let nx = if world {
        ((nx % gw) + gw) % gw
    } else {
        if nx < 0 || nx >= gw {
            return;
        }
        nx
    };
    if ny < 0 || ny >= gh {
        return;
    }
    let j = (ny * gw + nx) as usize;
    if done[j] {
        return;
    }
    done[j] = true;
    const EPS: f64 = 1e-6;
    if (filled[j] as f64) <= cur {
        filled[j] = (cur + EPS) as f32;
    }
    heap.push(filled[j], j);
}

#[allow(clippy::too_many_arguments)]
fn legacy_cc_visit(
    nx: isize,
    ny: isize,
    gw: isize,
    gh: isize,
    world: bool,
    sea: f64,
    field: &[f32],
    lab: &mut [i32],
    comp: i32,
    stack: &mut Vec<usize>,
) {
    let nx = if world {
        ((nx % gw) + gw) % gw
    } else {
        if nx < 0 || nx >= gw {
            return;
        }
        nx
    };
    if ny < 0 || ny >= gh {
        return;
    }
    let j = (ny * gw + nx) as usize;
    if lab[j] < 0 && (field[j] as f64) < sea {
        lab[j] = comp;
        stack.push(j);
    }
}

fn legacy_build_water_bodies(field: &[f32], gw: usize, gh: usize, sea: f64, world: bool, rain: Option<&[f32]>) -> Vec<u8> {
    let n = gw * gh;
    let gw_i = gw as isize;
    let gh_i = gh as isize;
    let mut out = vec![0u8; n];

    let mut lab = vec![-1i32; n];
    let mut comp: i32 = 0;
    let mut sizes: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for s in 0..n {
        if lab[s] >= 0 || (field[s] as f64) >= sea {
            continue;
        }
        lab[s] = comp;
        stack.clear();
        stack.push(s);
        let mut cnt = 0usize;
        while let Some(i) = stack.pop() {
            cnt += 1;
            let x = (i % gw) as isize;
            let y = (i / gw) as isize;
            legacy_cc_visit(x - 1, y, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            legacy_cc_visit(x + 1, y, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            legacy_cc_visit(x, y - 1, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
            legacy_cc_visit(x, y + 1, gw_i, gh_i, world, sea, field, &mut lab, comp, &mut stack);
        }
        sizes.push(cnt);
        comp += 1;
    }

    let mut ocean_comp: i32 = -1;
    let mut best: i64 = -1;
    for (c, &sz) in sizes.iter().enumerate() {
        if sz as i64 > best {
            best = sz as i64;
            ocean_comp = c as i32;
        }
    }
    for i in 0..n {
        if (field[i] as f64) < sea {
            out[i] = if lab[i] == ocean_comp { 1 } else { 2 };
        }
    }

    let mut filled: Vec<f32> = field.to_vec();
    let mut done = vec![false; n];
    let mut heap = LegacyMinHeap::with_capacity(n);

    for x in 0..gw {
        legacy_wb_seed(x, &filled, &mut done, &mut heap);
        legacy_wb_seed((gh - 1) * gw + x, &filled, &mut done, &mut heap);
    }
    if !world {
        for y in 0..gh {
            legacy_wb_seed(y * gw, &filled, &mut done, &mut heap);
            legacy_wb_seed(y * gw + gw - 1, &filled, &mut done, &mut heap);
        }
    }
    for (i, &o) in out.iter().enumerate() {
        if o == 1 {
            legacy_wb_seed(i, &filled, &mut done, &mut heap);
        }
    }

    while heap.size() > 0 {
        let i = heap.pop();
        let x = (i % gw) as isize;
        let y = (i / gw) as isize;
        let cur = filled[i] as f64;
        legacy_wb_visit(x - 1, y, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        legacy_wb_visit(x + 1, y, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        legacy_wb_visit(x, y - 1, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
        legacy_wb_visit(x, y + 1, cur, gw_i, gh_i, world, &mut filled, &mut done, &mut heap);
    }

    let lake_depth = 0.004_f64;
    let lake_rain = 0.22_f64;
    for i in 0..n {
        if out[i] == 0 {
            let depth = filled[i] as f64 - field[i] as f64;
            if depth > lake_depth {
                let rain_ok = match rain {
                    Some(r) => (r[i] as f64) >= lake_rain,
                    None => true,
                };
                if rain_ok {
                    out[i] = 2;
                }
            }
        }
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn compute_suitability_and_seeds(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
    river_term: RiverTerm,
    coast_term: CoastTerm,
) -> (Vec<f32>, Vec<cartalith_civ::SettlementSeed>) {
    // RE-BASELINED 2026-09-21 (Ruling Q, see this file's own
    // `legacy_build_water_bodies` doc comment above): the legacy arm
    // (`RiverTerm::LegacyProxy`/`CoastTerm::LegacySdf`, always paired in
    // this file's own two call sites) must keep reproducing the reference's
    // real captured numbers, so it uses the frozen pre-Ruling-Q water-body
    // classification; the production arm uses today's real one.
    let wb_classification: Vec<u8> = match river_term {
        RiverTerm::LegacyProxy => legacy_build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall)),
        RiverTerm::Real => cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall)).classification,
    };
    let biome = cartalith_civ::build_biome_raster(&wb_classification, &ws.temperature, &ws.rainfall);

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
    // Ruling N's coastal half: proximity to a real traced OCEAN coastline.
    // `build_coast_sdf` is still built, for the legacy arm only.
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, ws.sea_level);
    let coast_reach = match coast_term {
        CoastTerm::Real => cartalith_civ::build_coast_reach(
            &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, ws.sea_level),
            &wb_classification,
            gw,
            gh,
        ),
        CoastTerm::LegacySdf => legacy_coast_proxy(&coast_sdf),
    };
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, ws.sea_level);

    // Fresh river order on the FINAL post-carve field/flow -- see this
    // file's own module doc comment for why `ws.stream_order` is not a
    // substitute.
    let (river_order, river_polys) =
        cartalith_civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, river_density, map_width_km);
    // Ruling N: proximity to a real traced polyline, not `river_order[i]`.
    let river_reach = match river_term {
        RiverTerm::Real => cartalith_civ::build_river_reach(&river_polys, &river_order, gw, gh),
        RiverTerm::LegacyProxy => legacy_river_proxy(&ws.flow_discharge, &river_order, gw, gh, flow_thresh),
    };

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb_classification),
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

    let slope_n = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &slope_n, gw, gh, ws.sea_level, Some(&ctx));
    // 0.65 = findSettlementSeeds' own bare default, matching the
    // settlement_seeds.json export path (see this file's module doc
    // comment) -- NOT cartalith_civ::SETTLE_SEED_THRESH (0.42), which is
    // the interactive advisory view's override this port doesn't have.
    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.65, (gw as f64 / 20.0).max(4.0));

    (suit, seeds)
}

