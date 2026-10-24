//! FH-R2 milestone R2-0's **M0 baseline** and R2-1's **M1 "zero bytes moved"**
//! check (`FH_R2_PLAN.md` §6), as one ignored test over the real pipeline.
//!
//! What it is for. R2-1 appends seven religions to
//! `cartalith_civ::roster::CIV_RELIGIONS` under Ruling CG item 3, which approved
//! the plan's Q2 default: the new archetypes stay out of
//! `civ_default_religion`'s pick, so the append must move **no generated
//! output**. This harness fingerprints the outputs that could move if that
//! exclusion were missing or leaky, on real worlds, and compares each one with
//! a literal recorded from the tree **before** R2-1's code change (commit
//! `8e5be475`, recorded 2026-10-12). A mismatch is a failure, not a re-baseline
//! (`FH_R2_PLAN.md` §6, M1: "Any move is a failure").
//!
//! What is fingerprinted, per world (each one a 64-bit FNV-1a over a
//! `Debug`/`to_bits` rendering, so a one-ulp float change moves it):
//!
//! - **roster** -- every faction's name, culture, religion and government, as
//!   `compute_civilisation` seeds them (`FactionRoster::seeded` ->
//!   `FactionEntry::default_for` -> `civ_default_religion` /
//!   `civ_default_government`). The GeoJSON export's `religion` property is
//!   this column read through `engine_key` (`geojson_bridge.rs`), so the GeoJSON
//!   bytes are covered through their only religion input; the full GeoJSON
//!   document is **not** hashed here (it needs a live `WorldGen`).
//! - **relations** -- `civ_faction_relations` over the world's territory, the
//!   roster's cultures and engine religions, with an **empty**
//!   `FactionAggregates` (the app's aggregates need `WorldGen`'s resource
//!   passes). So the border, culture and religion terms and the stance they
//!   drive are covered; the trade and rivalry terms are not the app's values.
//! - **belief** -- a 100-year `CivData::civ_belief_run` (seed plus 100 steps),
//!   every settlement's share vector over the **first eight slots** (the
//!   reference's eight religions) by `f64::to_bits`. Slots past eight exist only
//!   after R2-1; the harness asserts every one of them is exactly `0.0` rather
//!   than hashing them, so the fingerprint is the same function before and
//!   after the vocabulary grows. **Since FH-R2 R2-2 (2026-10-18, Ruling CJ)**
//!   the run is fed the world's trade flows (`harness_flows`, the app's
//!   `trade_network` match) and its hashes are pinned by `M2_BELIEF`, a
//!   deliberate re-baseline; the other three columns still compare to M0.
//!   The same file holds R2-2's M2 bars (`fh_r2_m2_coexistence_bars`) and its
//!   cost measurement (`fh_r2_m2_belief_cost`).
//! - **urban** -- `run_layout` for the three most populous settlements (ties
//!   by index), every `Building` by `Debug` (kind, district, polygon, ridge,
//!   age). This is where a faith site's rite shows: since 2026-10-25 (Ruling
//!   CY item 1) the layout takes the app's faith (`CivData::town_faith`), as
//!   `urban_layouts` does, so the rite's cleared houses move this column
//!   where a top-three town's faction has a rite other than the default.
//!
//! Worlds: five seeds x {800 km plain, 40 075 km wrapped} x {6 factions (the
//! app default, `cartalith_engine::WorldParams::defaults()`'s `civ.factions`),
//! 24 factions (the parameter's maximum, `params.rs`'s `civ.factions` row)}.
//! The 24-faction worlds are the ones that matter: only a faction index >= 7
//! of an unthemed culture could land on an appended religion if the exclusion
//! were missing (`FH_R2_PLAN.md` §4.1). Grid 384 wide, height by
//! `reference_grid_h`: a labelled judgement. 192 wide (`bugsweep_seed707`'s
//! width) placed only 8 settlements per world, too few to exercise the belief
//! run; 384 places 28 (800 km) and 22 (40 075 km), and the 20 worlds run in
//! about 17 s (one run, 2026-10-12, not a benchmark).
//!
//! What it must never do: re-record its own literals in passing. The table
//! below is the M0 baseline; changing it is a re-baseline and needs an owner
//! ruling (`MISTAKES.md`, "Change generated output").
//!
//! Run (alone, one cargo at a time):
//! `cargo test -p cartalith-godot --lib fh_r2_baseline -j 2 -- --ignored --nocapture`

use cartalith_civ::relations::{civ_faction_relations, FactionRelationsInput};
use cartalith_civ::trade::{trade_flows, TradeFlow, TradeInput};
use cartalith_civ::urban_adapter::{run_layout, um_place_context, UrbanWorld};

/// The world's trade flows, built exactly as `WorldGen::trade_network` builds
/// them for the `civ_belief_run` `#[func]` (same `TradeInput` members, the
/// roster's tariff rows), so the belief run here is the app's. The traced
/// `river_polys` in `uw` are not consulted by `trade_flows` (its own caller's
/// comment in `civ_trade_bridge.rs`), so passing them changes nothing.
fn harness_flows(civ: &crate::CivData, uw: &UrbanWorld) -> Vec<TradeFlow> {
    let tariffs = civ.faction_roster.tariff_rows();
    let input = TradeInput {
        settlements: &civ.settlements,
        balances: &civ.trade_balances,
        ways: &civ.ways,
        map_width_km: uw.map_width_km,
        gw: uw.gw,
        tariffs: &tariffs,
    };
    trade_flows(&input, uw).flows
}

/// Grid width for every harness world (see the module doc).
const GW: usize = 384;

/// The five seeds: four already used by this repository's harnesses and
/// probes (`12345`/`314159` by GF-13's bars, `404`/`707` by the Ruling CD
/// probe and `bugsweep_seed707`), plus `24601` from the river comb captures.
/// A labelled choice, not a sample design.
const SEEDS: [i32; 5] = [12345, 314159, 404, 707, 24601];

/// One recorded world: `(seed, width_km, factions, roster, relations, belief,
/// urban)`. The four hashes were recorded at commit `8e5be475` on 2026-10-12,
/// before any R2-1 code change, by running this test with an empty table.
type Row = (i32, u32, i32, u64, u64, u64, u64);

/// The M0 baseline. See the module doc: never edit a value in passing.
/// Kept as the historical record and read by no test since 2026-10-24 (see
/// `AT_CURRENT`), hence the `allow`.
#[allow(dead_code)]
const M0: [Row; 20] = [
    (12345, 800, 6, 0x12ad5407c90b2d2f, 0xab1f17fb0d785229, 0xd019daca6ee60c84, 0x8178f309faf8f7cd), // 28 settlements
    (12345, 800, 24, 0x83fc2cde5ba874d6, 0x03a079dd4e952fa8, 0xc6b267da00dd815a, 0xeedc4dfd08ed6b39), // 28 settlements
    (12345, 40075, 6, 0x12ad5407c90b2d2f, 0xe6a27c902142fd6d, 0xb840116a07e00732, 0xa0a94dfc0d6141d8), // 22 settlements
    (12345, 40075, 24, 0x83fc2cde5ba874d6, 0x8c411caacf6adbe0, 0xc57f15bef9f2112b, 0xbbbca4ed2411c7a4), // 22 settlements
    (314159, 800, 6, 0x12ad5407c90b2d2f, 0xfe09868af3a03810, 0xe6f99b77944d6f4a, 0xe1fc839176491f3d), // 28 settlements
    (314159, 800, 24, 0x83fc2cde5ba874d6, 0xdb8258de1905bcc8, 0x091233403d7f5883, 0xec6362dd4cf3e60f), // 28 settlements
    (314159, 40075, 6, 0x12ad5407c90b2d2f, 0xfbb32547284c426f, 0x2fdf851701a0ba74, 0x09f120130daf1645), // 22 settlements
    (314159, 40075, 24, 0x83fc2cde5ba874d6, 0x6f9ee5d930551d02, 0x49881b58cd8d4f16, 0x0655b38cb85ad0f5), // 22 settlements
    (404, 800, 6, 0x12ad5407c90b2d2f, 0x90d8436e3df4f695, 0x3d004e55ae482930, 0x35327a9b8ab7c017), // 28 settlements
    (404, 800, 24, 0x83fc2cde5ba874d6, 0x0b5f850938724c15, 0xd0d808a29f65e726, 0x017eb0ca7c7b1781), // 28 settlements
    (404, 40075, 6, 0x12ad5407c90b2d2f, 0x92685f1dd76a2e87, 0x61699fa797a43be0, 0x56cc6d5a8683ba8c), // 22 settlements
    (404, 40075, 24, 0x83fc2cde5ba874d6, 0xe16312e12dc2baa1, 0xf0c094c4637f8535, 0x9b8f65e2d737be2d), // 22 settlements
    (707, 800, 6, 0x12ad5407c90b2d2f, 0x02d35a629c08de38, 0x07c1a044c129cb85, 0xc577368bf4274eff), // 28 settlements
    (707, 800, 24, 0x83fc2cde5ba874d6, 0xe3d2d364306c6668, 0xea17b82042d1490b, 0x619401557b5f5810), // 28 settlements
    (707, 40075, 6, 0x12ad5407c90b2d2f, 0xd45702da336f998e, 0xb7180f53ce9df9c7, 0x0d9c2544948e6ec0), // 22 settlements
    (707, 40075, 24, 0x83fc2cde5ba874d6, 0x6adbba17e77282d2, 0x28c543fa25fc8d13, 0xbc1de39c1750f081), // 22 settlements
    (24601, 800, 6, 0x12ad5407c90b2d2f, 0xcfab231de5e42de1, 0x4b564592bd5bdfcc, 0x361d625320f62e47), // 28 settlements
    (24601, 800, 24, 0x83fc2cde5ba874d6, 0x7d50d270de4d9614, 0x67664ae2df2f9e93, 0x2125f79a12163cd8), // 28 settlements
    (24601, 40075, 6, 0x12ad5407c90b2d2f, 0x56b2cbe09310bcf1, 0xea826840a949e010, 0x2e4de7ca282a081e), // 22 settlements
    (24601, 40075, 24, 0x83fc2cde5ba874d6, 0xf0de3bb7c3378c4d, 0x87a829e2db67aa25, 0xdcff73937261b189), // 22 settlements
];

/// 64-bit FNV-1a, the hash `Fnv` uses elsewhere in Rust tooling; chosen
/// because it is a dozen lines with no dependency and is stable across
/// platforms and Rust versions (unlike `DefaultHasher`, whose algorithm the
/// standard library does not promise).
#[derive(Clone, Copy)]
struct Fnv(u64);

impl Fnv {
    /// The FNV-1a 64-bit offset basis (Fowler-Noll-Vo, published constant).
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    /// Folds `bytes` in, using the FNV-1a 64-bit prime (published constant).
    fn eat(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    /// Folds a string in followed by a separator byte, so `["ab","c"]` and
    /// `["a","bc"]` hash differently.
    fn eat_str(&mut self, s: &str) {
        self.eat(s.as_bytes());
        self.eat(&[0x1f]);
    }
}

/// The four fingerprints of one world, plus counts that prove the fixture
/// reached the code under test (a world with no settlements or no relations
/// would hash "nothing" identically before and after and prove nothing).
struct Print {
    roster: u64,
    relations: u64,
    belief: u64,
    urban: u64,
    factions: usize,
    pairs: usize,
    settlements: usize,
    towns: usize,
    buildings: usize,
}

/// Generates one world through the app's parameters (`params::defaults()`,
/// CPU path), `gw` cells wide: the terrain, its civilisation layer, the grid
/// height and the parameters used.
fn generate_world(
    seed: i32,
    width_km: f64,
    factions: i32,
    gw: usize,
) -> (cartalith_engine::WorldState, crate::CivData, usize, cartalith_engine::WorldParams) {
    let world = width_km >= 40_075.0;
    let gh = crate::reference_grid_h(gw, world);
    let mut p = crate::with_call_args(crate::params::defaults(), seed, width_km, gw, gh);
    p.world = world;
    p.civ.factions = factions;
    p.use_gpu = false;
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
    let civ = crate::compute_civilisation(
        &ws, gw, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false,
        &mut Vec::new(), None,
    );
    (ws, civ, gh, p)
}

/// Generates one world through the app's parameters and fingerprints it.
fn fingerprint(seed: i32, width_km: f64, factions: i32) -> Print {
    let (ws, mut civ, gh, p) = generate_world(seed, width_km, factions, GW);

    // roster
    let mut h = Fnv::new();
    for e in civ.faction_roster.0.iter() {
        h.eat_str(&e.name);
        h.eat_str(&e.culture);
        h.eat_str(&e.religion);
        h.eat_str(&e.government);
    }
    let roster = h.0;

    // relations
    let cultures: Vec<&str> = civ.faction_roster.0.iter().map(|e| e.culture.as_str()).collect();
    let religions: Vec<&str> = civ.faction_roster.engine_religions(&civ.religions);
    let input = FactionRelationsInput {
        faction_count: civ.faction_roster.0.len(),
        gw: GW,
        gh,
        territory: Some(&civ.territory),
        cultures: &cultures,
        religions: &religions,
        wrap_x: p.world,
    };
    let empty = cartalith_civ::FactionAggregates {
        by_faction: Vec::new(),
        max_pop: 0.0,
        max_trade_volume: 0.0,
        max_territory_km2: 0.0,
        max_settlement_count: 0,
        world_mean_resource: Default::default(),
        world_mean_terrain: Default::default(),
    };
    let rel = civ_faction_relations(&input, &empty);
    let mut h = Fnv::new();
    for r in rel.pairs.iter() {
        h.eat_str(&format!("{r:?}"));
    }
    let relations = h.0;
    let pairs = rel.pairs.len();

    // urban (before the belief run, which does not touch settlements anyway)
    let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
        (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, GW, gh, 1),
        _ => Vec::new(),
    };
    let uw = UrbanWorld {
        field: &ws.field,
        flow: &ws.flow_discharge,
        water_bodies: &civ.water_bodies,
        order: ws.stream_order.as_deref(),
        river_polys: &river_polys,
        gw: GW,
        gh,
        sea_level: ws.sea_level,
        map_width_km: p.map_width_km,
        flow_thresh: cartalith_hydrology::river_flow_thresh(GW, gh, GW, p.map_width_km),
        world_seed: seed,
    };
    let mut order: Vec<usize> = (0..civ.settlements.len()).collect();
    order.sort_by(|&a, &b| civ.settlements[b].pop.cmp(&civ.settlements[a].pop).then(a.cmp(&b)));
    let mut h = Fnv::new();
    let (mut towns, mut buildings) = (0usize, 0usize);
    for &i in order.iter().take(3) {
        // Since 2026-10-25 (Ruling CY item 1) with the app's faith, so this
        // column measures what `urban_layouts` lays out.
        let s = &civ.settlements[i];
        let o = cartalith_civ::urban_adapter::PlaceOverrides { faith: civ.town_faith(s), ..Default::default() };
        let ctx = cartalith_civ::urban_adapter::um_place_context_with(&uw, s, &civ.ways, &o);
        match run_layout(&ctx, None) {
            Some(layout) => {
                towns += 1;
                buildings += layout.buildings.len();
                for b in layout.buildings.iter() {
                    h.eat_str(&format!("{b:?}"));
                }
            }
            None => h.eat_str("no layout"),
        }
    }
    let urban = h.0;

    // belief -- since FH-R2 R2-2 a run reads the world's trade flows (the
    // influx term), computed the way `WorldGen::trade_network` does, so the
    // harness measures what the app runs.
    let flows = harness_flows(&civ, &uw);
    civ.civ_belief_run(100, &flows);
    let mut h = Fnv::new();
    for s in civ.belief.iter() {
        for (r, v) in s.share.iter().enumerate() {
            if r < 8 {
                h.eat(&v.to_bits().to_le_bytes());
            } else {
                assert_eq!(*v, 0.0, "seed {seed}: an appended religion slot {r} holds a share");
            }
        }
    }
    let belief = h.0;

    Print {
        roster,
        relations,
        belief,
        urban,
        factions: civ.faction_roster.count(),
        pairs,
        settlements: civ.settlements.len(),
        towns,
        buildings,
    }
}

/// The belief column after FH-R2 R2-2, one per [`M0`] row in the same order.
///
/// RE-BASELINED 2026-10-18 under Ruling CJ (which approved `FH_R2_PLAN.md` Q4,
/// S2's coexistence terms, and M2's bars): R2-2 changes what every belief run
/// produces by design, so all 20 belief hashes moved, M0 -> this table, while
/// the roster, relations and urban columns stay on [`M0`] byte for byte. The
/// M0 belief literals are kept in [`M0`] as the pre-R2-2 record. Recorded by
/// this test's own printout on the R2-2 tree (every weight at its committed
/// value, trade flows from [`harness_flows`]). Before recording, every R2-2
/// weight was set to `0.0` and all 20 belief hashes reproduced [`M0`]
/// (STATUS.md's R2-2 entry): the terms vanish by arithmetic at weight zero, so
/// this move is the terms' and nothing else's.
#[allow(dead_code)] // historical since 2026-10-24, see `AT_CURRENT`
const M2_BELIEF: [u64; 20] = [
    0x97467380a4a39565, // 12345 800 6     (M0 0xd019daca6ee60c84)
    0xe5363e50d332b709, // 12345 800 24    (M0 0xc6b267da00dd815a)
    0xbf122e5cc156bb2a, // 12345 40075 6   (M0 0xb840116a07e00732)
    0xa77dc6df2bb72045, // 12345 40075 24  (M0 0xc57f15bef9f2112b)
    0x50ee36e2ed6a55b4, // 314159 800 6    (M0 0xe6f99b77944d6f4a)
    0x97dece3e0f56467e, // 314159 800 24   (M0 0x091233403d7f5883)
    0xd5354a151d119921, // 314159 40075 6  (M0 0x2fdf851701a0ba74)
    0xb668cae81cdddf5e, // 314159 40075 24 (M0 0x49881b58cd8d4f16)
    0x0aad938cf041c1b3, // 404 800 6       (M0 0x3d004e55ae482930)
    0x48dd5f0e9b822783, // 404 800 24      (M0 0xd0d808a29f65e726)
    0xe550ba63c7f10ed6, // 404 40075 6     (M0 0x61699fa797a43be0)
    0xa098d8b994dd413b, // 404 40075 24    (M0 0xf0c094c4637f8535)
    0x1e3de270b41aa491, // 707 800 6       (M0 0x07c1a044c129cb85)
    0xea076a454f1ef412, // 707 800 24      (M0 0xea17b82042d1490b)
    0x96c3916cdae4b33c, // 707 40075 6     (M0 0xb7180f53ce9df9c7)
    0x450a6f629450882f, // 707 40075 24    (M0 0x28c543fa25fc8d13)
    0x019604d20d730ba5, // 24601 800 6     (M0 0x4b564592bd5bdfcc)
    0xab5d184ee2a15f01, // 24601 800 24    (M0 0x67664ae2df2f9e93)
    0x651045b294da1d9f, // 24601 40075 6   (M0 0xea826840a949e010)
    0xb7e5e03852df7746, // 24601 40075 24  (M0 0x87a829e2db67aa25)
];

/// The whole harness re-recorded at `0801fbce` (2026-10-24), the tree FH-R2
/// R2-3 started from, one row per [`M0`] row in the same order.
///
/// RE-BASELINED 2026-10-24, and disclosed rather than ruled: **the harness was
/// already red at `0801fbce`, before any R2-3 change.** Run there, 20 of 20
/// worlds differ from [`M0`]/[`M2_BELIEF`] in the relations, belief and urban
/// columns (the roster column is unchanged: same religion and government
/// picks). Every one of those columns reads the generated world itself
/// (territory, settlements, town sites), and between R2-2 (`820e0acd`) and
/// `0801fbce` several ruled app-side terrain changes landed in
/// `params::defaults()` -- Ruling CM's B-spline rain up-sample (`26ac5156`),
/// Ruling CO's volcano pit fill (`21874074`, `2cfce96c`), Ruling CU's orogen
/// ranges (`782ec8c7`) -- each of which moves app worlds by its own ruling.
/// Which of them moved these worlds was **not bisected**; it is a candidate
/// list, not a finding. The rows were first printed by this test on a clean
/// `0801fbce` worktree; [`M0`] and [`M2_BELIEF`] stay as the historical
/// record.
///
/// RE-BASELINED AGAIN 2026-10-25, after Ruling CX item 1 (`45ce3d29`, VA-5
/// volcano morphometry ON in the app's `params::defaults()`) moved app
/// worlds: 12 of 20 rows moved (relations 9, belief 12, urban 4, roster 0),
/// each moved row carrying its `0801fbce` value in its comment. Re-printed on
/// a clean `ea75a87e` worktree and identical on the tree with FH-R2 R2-3, R2-6
/// and R2-4 applied, so none of the move is FH-R2's. From R2-3 on, a stage's "0 bytes moved" bar is measured against this
/// table, and a stage that changes a column re-baselines it here with the old
/// value beside the new.
///
/// RE-BASELINED 2026-10-25 under Ruling CY item 1 (FH-R2 S3a switched on): the
/// urban column now lays towns out with the app's faith (`CivData::town_faith`,
/// as `urban_layouts` does), and all 20 rows' urban hashes moved; each carries
/// its pre-switch value in its comment, and the building counts are the new
/// ones. Roster, relations and belief are byte-identical. Re-printed on the
/// tree rebased onto `884e7d27` (Ruling CX items 2-3) and identical to the run
/// before that rebase, so none of the move is the volcano work's.
///
/// RE-BASELINED AGAIN 2026-10-25, after rebasing onto `dbdb9b37` (Ruling CX
/// item 4, the grid-aware scoria-cone law ON in the app), which moved app
/// worlds: measured with this harness's urban column forced back to no
/// faith, CX item 4 alone moved relations on 13 rows, belief on 20 and urban
/// on 10 (roster 0); the S3 switch then moves urban on 20 of 20. Rows whose
/// relations or belief moved carry their previous values in the comment. Which of a
/// row's three towns moved was not broken down; `fh_r2_m3_rite_only` shows
/// that every house a rite moves sits under the new building.
const AT_CURRENT: [Row; 20] = [
    (12345, 800, 6, 0x12ad5407c90b2d2f, 0x0d1a231ae44944af, 0xebec72f720a760e8, 0xfd7bfb7ce66a5c4b), // 28 settlements, 15 pairs, 3 towns, 9600 buildings; was (12345, 800, 6, 0x12ad5407c90b2d2f, 0x5c2004de10c6265f, 0x6ef29e9bcdae1ca9, 0xbcb3da126476c843) at 0801fbce; urban was 0xbcb3da126476c843 before Ruling CY's rite switch-on; relations/belief were 0x0d1a231ae44944af/0xc2bf20dd2d1c1a07 before Ruling CX item 4
    (12345, 800, 24, 0x83fc2cde5ba874d6, 0x4e02807c519c4bb9, 0xd30c1bb2d9e9b400, 0xc1976b0e8f11e87d), // 28 settlements, 276 pairs, 3 towns, 12157 buildings; was (12345, 800, 24, 0x83fc2cde5ba874d6, 0x60b724792206c53b, 0xda772cea5a60f87b, 0xbbe3a8e71eb7a26c) at 0801fbce; urban was 0xbbe3a8e71eb7a26c before Ruling CY's rite switch-on; relations/belief were 0x3b274aea47b357fc/0x4273ed93cbb870b9 before Ruling CX item 4
    (12345, 40075, 6, 0x12ad5407c90b2d2f, 0x95c289cde0c804f7, 0x718ec2e4009a0414, 0xd68ac4500da544cc), // 22 settlements, 15 pairs, 3 towns, 12018 buildings; urban was 0xc472ed74011cea76 before Ruling CY's rite switch-on; relations/belief were 0x95c289cde0c804f7/0x4e5ec3cf050eed16 before Ruling CX item 4
    (12345, 40075, 24, 0x83fc2cde5ba874d6, 0xf2b266e1a38b4043, 0x0dc971fc84218e6c, 0xde9ae7219cc3f6b6), // 22 settlements, 276 pairs, 3 towns, 13015 buildings; was (12345, 40075, 24, 0x83fc2cde5ba874d6, 0x7d6ea6524e9aa563, 0xf6f43dba29d80ac0, 0xc224165ac8ab2daa) at 0801fbce; urban was 0xc224165ac8ab2daa before Ruling CY's rite switch-on; relations/belief were 0x7d6ea6524e9aa563/0xb95e53a1e3bf5400 before Ruling CX item 4
    (314159, 800, 6, 0x12ad5407c90b2d2f, 0x2a1a393c17267114, 0x6245c5984dc80257, 0x081a049d34370c5f), // 28 settlements, 15 pairs, 3 towns, 11347 buildings; was (314159, 800, 6, 0x12ad5407c90b2d2f, 0xf0e680f532a2af86, 0xc8665b1a229e8bf0, 0xd7cadf5474989e2d) at 0801fbce; urban was 0x53aa8c9f1be57867 before Ruling CY's rite switch-on; relations/belief were 0x2a1a393c17267114/0x60694227da60a6e4 before Ruling CX item 4
    (314159, 800, 24, 0x83fc2cde5ba874d6, 0x77a862c948cfc4b1, 0x3e206b6b41070523, 0xe8dd36aa36a8e532), // 28 settlements, 276 pairs, 3 towns, 11255 buildings; was (314159, 800, 24, 0x83fc2cde5ba874d6, 0xab8459c721f3aa24, 0x5ff9ad8480196a8b, 0x2dbc7beffcd5c5b2) at 0801fbce; urban was 0xd7e1e98e14b60bc4 before Ruling CY's rite switch-on; relations/belief were 0xab8459c721f3aa24/0x0eabaeb491f1e8cf before Ruling CX item 4
    (314159, 40075, 6, 0x12ad5407c90b2d2f, 0x275abc9ea5deb8a1, 0x4a9b4555bd2f1462, 0xddb6edc862de69ad), // 22 settlements, 15 pairs, 3 towns, 13200 buildings; urban was 0x1a21bca108e23078 before Ruling CY's rite switch-on; relations/belief were 0x75596bb70531fc30/0xaa708efde9b802f3 before Ruling CX item 4
    (314159, 40075, 24, 0x83fc2cde5ba874d6, 0x28e9de78c8ff5474, 0x93c117b628b76927, 0xe4723c5aec1cd8b8), // 22 settlements, 276 pairs, 3 towns, 11793 buildings; urban was 0xb8cff3caea5af249 before Ruling CY's rite switch-on; relations/belief were 0xa6c3dfc12981ca11/0xbd742606c68ee9b8 before Ruling CX item 4
    (404, 800, 6, 0x12ad5407c90b2d2f, 0xe72a483510747a94, 0x5f1159474ea63b17, 0x076bad9e9a3789b0), // 28 settlements, 15 pairs, 3 towns, 11998 buildings; was (404, 800, 6, 0x12ad5407c90b2d2f, 0x6d489371e183bfe5, 0x08aa0d83b485a732, 0xe9eeb4ee8cd67eae) at 0801fbce; urban was 0xe9eeb4ee8cd67eae before Ruling CY's rite switch-on; relations/belief were 0xe72a483510747a94/0x21f13aaf362114ea before Ruling CX item 4
    (404, 800, 24, 0x83fc2cde5ba874d6, 0xfe1bfbeab5210a87, 0x7caff4f91a38da6f, 0xb17ebea2c1b9fea9), // 28 settlements, 276 pairs, 3 towns, 13152 buildings; was (404, 800, 24, 0x83fc2cde5ba874d6, 0xd6cc372330159c4e, 0x55b6fc8b0c4f4641, 0x4a8d9f8fa5757d3a) at 0801fbce; urban was 0x4a8d9f8fa5757d3a before Ruling CY's rite switch-on; relations/belief were 0xfe1bfbeab5210a87/0x64a03eba1b0d7a80 before Ruling CX item 4
    (404, 40075, 6, 0x12ad5407c90b2d2f, 0x4a0254ee296843b7, 0xb36bd3a43a922f94, 0x5eef6cc888ee49ff), // 22 settlements, 15 pairs, 3 towns, 9979 buildings; urban was 0xc1fb1a8707c483e2 before Ruling CY's rite switch-on; relations/belief were 0x02594796df689f67/0xff7ffc1933569655 before Ruling CX item 4
    (404, 40075, 24, 0x83fc2cde5ba874d6, 0xb338b6f48a89b478, 0x4396f2c78c54cc99, 0xc94b487f7a9c25da), // 22 settlements, 276 pairs, 3 towns, 11247 buildings; was (404, 40075, 24, 0x83fc2cde5ba874d6, 0xe4d590af380630db, 0x72e75e14a35ef867, 0x4c458d53a4752bed) at 0801fbce; urban was 0x4c458d53a4752bed before Ruling CY's rite switch-on; relations/belief were 0x4afb198c85fe8b2f/0xc24cf86a34076751 before Ruling CX item 4
    (707, 800, 6, 0x12ad5407c90b2d2f, 0x645fd8c818683126, 0x7d27f7924c6752ad, 0x65c95cb220485e42), // 28 settlements, 15 pairs, 3 towns, 11711 buildings; was (707, 800, 6, 0x12ad5407c90b2d2f, 0x8b390478b554281e, 0x8e310cdb39d8269f, 0x6f78d2d6bac38e4b) at 0801fbce; urban was 0x6f78d2d6bac38e4b before Ruling CY's rite switch-on; relations/belief were 0xa9667ca05bb9740f/0xb5700fa6df9af237 before Ruling CX item 4
    (707, 800, 24, 0x83fc2cde5ba874d6, 0xb9b0611bb23e2e5a, 0xfc538ebd7a37e6aa, 0x4a5741ec00a1d5d8), // 28 settlements, 276 pairs, 3 towns, 13349 buildings; was (707, 800, 24, 0x83fc2cde5ba874d6, 0xcedf35dd0eb2a5ae, 0xe518464aa3c95cd2, 0xdad31b17be184141) at 0801fbce; urban was 0xdad31b17be184141 before Ruling CY's rite switch-on; relations/belief were 0xcedf35dd0eb2a5ae/0xe88ef1ddfd8fb89e before Ruling CX item 4
    (707, 40075, 6, 0x12ad5407c90b2d2f, 0x423e55d58efe8710, 0xba99d6473a7da81d, 0xa4b581be2526466e), // 22 settlements, 15 pairs, 3 towns, 12548 buildings; urban was 0x20c886252478cac2 before Ruling CY's rite switch-on; relations/belief were 0x423e55d58efe8710/0x9f49f53f1b3df459 before Ruling CX item 4
    (707, 40075, 24, 0x83fc2cde5ba874d6, 0xad2e2e0b81d07e01, 0xecdab5f5a35d34e2, 0xd6f2d571cefe4841), // 22 settlements, 276 pairs, 3 towns, 14077 buildings; urban was 0xcef0e234416caaa9 before Ruling CY's rite switch-on; relations/belief were 0xad2e2e0b81d07e01/0x4654513f79d40be8 before Ruling CX item 4
    (24601, 800, 6, 0x12ad5407c90b2d2f, 0x8d371cd21f96d1d0, 0xd788b7866dd5661c, 0x3a7e32981925c6a8), // 28 settlements, 15 pairs, 3 towns, 11825 buildings; was (24601, 800, 6, 0x12ad5407c90b2d2f, 0xbefe95773005588d, 0x72b0cabcba4b567f, 0x22a8f50c77d0cf82) at 0801fbce; urban was 0x9cd714fae1664329 before Ruling CY's rite switch-on; relations/belief were 0x13d97ff356d62bab/0x07c3d0d1dbe0bd48 before Ruling CX item 4
    (24601, 800, 24, 0x83fc2cde5ba874d6, 0xf3ff9e3c33a6830e, 0x96e8d875d499e50e, 0x4c24a92a623b85cb), // 28 settlements, 276 pairs, 3 towns, 12159 buildings; was (24601, 800, 24, 0x83fc2cde5ba874d6, 0x06380ec2d44607f8, 0x1118b2972ae33d37, 0x0a0accc42b23eb4e) at 0801fbce; urban was 0xa0d2e78175d2deea before Ruling CY's rite switch-on; relations/belief were 0x42c1cb98b9eed7a2/0x817ef18fec84b2ba before Ruling CX item 4
    (24601, 40075, 6, 0x12ad5407c90b2d2f, 0xeb486706c835f8a9, 0xd022c33f9e00d80d, 0x223c45302667d212), // 22 settlements, 15 pairs, 3 towns, 11857 buildings; urban was 0x528600d735c84d91 before Ruling CY's rite switch-on; relations/belief were 0xce0f10490895e492/0x1a72a4dd95c5a5e6 before Ruling CX item 4
    (24601, 40075, 24, 0x83fc2cde5ba874d6, 0x64e07d5a5f22394d, 0x33cd92fa46de2f7b, 0x76edbdb5a9e6180d), // 22 settlements, 276 pairs, 3 towns, 12643 buildings; urban was 0x962e576450dcf773 before Ruling CY's rite switch-on; relations/belief were 0xc42b13d64411c63f/0x0ff4132f780ff84a before Ruling CX item 4
];

/// Protects: R2-1 (`FH_R2_PLAN.md` M1) moves zero bytes of the roster's
/// religion/government picks, the relations they feed and the three largest
/// towns' buildings on 20 real worlds against the M0 literals recorded before
/// the change, and (since R2-2) that a 100-year belief run equals the
/// re-baselined [`M2_BELIEF`] column. The name is kept because STATUS.md
/// cites it; its belief half is now an M2 pin. Ignored: it generates 20 worlds.
#[test]
#[ignore = "M0/M1 measurement over 20 generated worlds; run alone with --ignored"]
fn fh_r2_m1_moves_zero_bytes_against_m0() {
    let mut rows: Vec<Row> = Vec::new();
    for &seed in SEEDS.iter() {
        for &width in [800u32, 40_075].iter() {
            for &factions in [6i32, 24].iter() {
                let f = fingerprint(seed, f64::from(width), factions);
                // The fixture must reach the code under test.
                assert!(f.settlements > 0, "seed {seed} {width} km: no settlements");
                assert!(f.pairs > 0, "seed {seed} {width} km: no relation pairs");
                assert!(f.towns > 0 && f.buildings > 0, "seed {seed} {width} km: no town layout");
                assert_eq!(f.factions, factions as usize);
                println!(
                    "FHR2 ({seed}, {width}, {factions}, 0x{:016x}, 0x{:016x}, 0x{:016x}, 0x{:016x}), \
                     // {} settlements, {} pairs, {} towns, {} buildings",
                    f.roster, f.relations, f.belief, f.urban, f.settlements, f.pairs, f.towns, f.buildings
                );
                rows.push((seed, width, factions, f.roster, f.relations, f.belief, f.urban));
            }
        }
    }
    let mut moved = Vec::new();
    // Compared with `AT_CURRENT` since R2-3 (2026-10-24): see that table's
    // doc for why `M0`/`M2_BELIEF` no longer describe the app's worlds.
    for (got, want) in rows.iter().zip(AT_CURRENT.iter()) {
        if got != want {
            moved.push(format!("got {got:?} want {want:?}"));
        }
    }
    assert!(moved.is_empty(), "M1/M2: {} of 20 worlds moved:\n{}", moved.len(), moved.join("\n"));
}

/// One world's `UrbanWorld` for `trade_flows`, with no traced river polylines
/// (`trade_flows` does not consult them; `WorldGen::trade_network` passes none
/// either).
fn trade_world<'a>(
    ws: &'a cartalith_engine::WorldState,
    civ: &'a crate::CivData,
    gw: usize,
    gh: usize,
    p: &cartalith_engine::WorldParams,
    seed: i32,
    no_polys: &'a [Vec<(f64, f64)>],
) -> UrbanWorld<'a> {
    UrbanWorld {
        field: &ws.field,
        flow: &ws.flow_discharge,
        water_bodies: &civ.water_bodies,
        order: ws.stream_order.as_deref(),
        river_polys: no_polys,
        gw,
        gh,
        sea_level: ws.sea_level,
        map_width_km: p.map_width_km,
        flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
        world_seed: seed,
    }
}

/// FH-R2 R2-2's **M2 coexistence bars** (`FH_R2_PLAN.md` §6, approved by
/// Ruling CJ), measured on the twenty M0 worlds after a 200-year
/// `civ_belief_run` fed by each world's own trade flows:
///
/// - *Hubs hold minorities*: of the settlements in each world's top decile of
///   trade volume (`ceil(n / 10)` by volume, both flow ends counted, ties by
///   index, zero-volume settlements never hubs), **at least 50 %** hold two
///   or more faiths (the unaffiliated slot excluded) each at >= 5 %.
/// - *Isolated villages still converge*: of the settlements with at most one
///   distinct road neighbour and no trade flow, **at least 90 %** hold one
///   affiliation (the unaffiliated slot counted, since an isolated Unclaimed
///   village is wholly unaffiliated) at >= 95 %.
///
/// Both thresholds are the plan's labelled judgements. The populations are
/// printed and asserted non-empty, so a vacuous pass is a failure.
///
/// Protects: that the calibrated R2-2 weights meet the approved bars on real
/// worlds, not only on fixtures.
#[test]
#[ignore = "M2 measurement over 20 generated worlds; run alone with --ignored"]
fn fh_r2_m2_coexistence_bars() {
    use cartalith_civ::belief::RELIGION_NONE;
    let (mut hubs, mut hubs_plural, mut hubs_plural_with_none) = (0usize, 0usize, 0usize);
    let (mut iso, mut iso_fixed) = (0usize, 0usize);
    let (mut all, mut all_plural) = (0usize, 0usize);
    for &seed in SEEDS.iter() {
        for &width in [800u32, 40_075].iter() {
            for &factions in [6i32, 24].iter() {
                let (ws, mut civ, gh, p) = generate_world(seed, f64::from(width), factions, GW);
                let no_polys: Vec<Vec<(f64, f64)>> = Vec::new();
                let flows = {
                    let uw = trade_world(&ws, &civ, GW, gh, &p, seed, &no_polys);
                    harness_flows(&civ, &uw)
                };
                civ.civ_belief_run(200, &flows);
                let n = civ.settlements.len();
                assert_eq!(civ.belief.len(), n, "the run covers every settlement");
                let mut vol = vec![0.0f64; n];
                for f in flows.iter() {
                    if f.from < n && f.to < n && f.from != f.to && f.volume > 0.0 {
                        vol[f.from] += f.volume;
                        vol[f.to] += f.volume;
                    }
                }
                let mut nb: Vec<std::collections::BTreeSet<usize>> = vec![Default::default(); n];
                for w in civ.ways.iter() {
                    if w.a_idx != w.b_idx && w.a_idx < n && w.b_idx < n {
                        nb[w.a_idx].insert(w.b_idx);
                        nb[w.b_idx].insert(w.a_idx);
                    }
                }
                let faiths = |i: usize, with_none: bool| -> usize {
                    civ.belief[i]
                        .share
                        .iter()
                        .enumerate()
                        .filter(|&(r, &v)| (with_none || r != RELIGION_NONE) && v >= 0.05)
                        .count()
                };
                let mut order: Vec<usize> = (0..n).filter(|&i| vol[i] > 0.0).collect();
                order.sort_by(|&a, &b| vol[b].partial_cmp(&vol[a]).unwrap().then(a.cmp(&b)));
                let k = n.div_ceil(10).min(order.len());
                let (mut wh, mut whp) = (0usize, 0usize);
                for &i in order.iter().take(k) {
                    wh += 1;
                    if faiths(i, false) >= 2 {
                        whp += 1;
                    }
                    if faiths(i, true) >= 2 {
                        hubs_plural_with_none += 1;
                    }
                }
                let (mut wi, mut wif) = (0usize, 0usize);
                for i in 0..n {
                    if faiths(i, false) >= 2 {
                        all_plural += 1;
                    }
                    if nb[i].len() <= 1 && vol[i] == 0.0 {
                        wi += 1;
                        let top = civ.belief[i].share.iter().copied().fold(0.0f64, f64::max);
                        if top >= 0.95 {
                            wif += 1;
                        }
                    }
                }
                all += n;
                println!(
                    "M2 seed {seed} {width} km {factions} f: {n} settlements, {} flows; hubs {whp}/{wh} \
                     plural; isolated {wif}/{wi} converged",
                    flows.len()
                );
                hubs += wh;
                hubs_plural += whp;
                iso += wi;
                iso_fixed += wif;
            }
        }
    }
    let hub_pct = 100.0 * hubs_plural as f64 / hubs.max(1) as f64;
    let iso_pct = 100.0 * iso_fixed as f64 / iso.max(1) as f64;
    println!(
        "M2 TOTAL: hubs {hubs_plural}/{hubs} = {hub_pct:.1} % plural (>= 2 faiths at >= 5 %; \
         {hubs_plural_with_none}/{hubs} counting the unaffiliated slot); isolated {iso_fixed}/{iso} = \
         {iso_pct:.1} % converged (>= 95 %); all settlements plural {all_plural}/{all}"
    );
    assert!(hubs > 0 && iso > 0, "both populations must be non-empty: hubs {hubs}, isolated {iso}");
    assert!(hub_pct >= 50.0, "hub bar: {hub_pct:.1} % < 50 %");
    assert!(iso_pct >= 90.0, "isolated-village bar: {iso_pct:.1} % < 90 %");
}

/// FH-R2 R2-2's **M2 cost bar** (`FH_R2_PLAN.md` §6): a 100-year
/// `civ_belief_run` (seed plus 100 steps, re-seeded each run by clearing the
/// layer), median with min..max over five runs, run alone, against the
/// pre-R2-2 median measured on the same worlds at `b769ad0f` (STATUS.md's R2-2
/// entry). Prints the trade-flow match's own cost separately, because the
/// `#[func]` now computes it once per run. No assertion: a timing is a
/// measurement to read, not a pin (`MISTAKES.md`, "Quote a timing").
#[test]
#[ignore = "M2 timing; run alone with --ignored --release"]
fn fh_r2_m2_belief_cost() {
    for &gw in [GW, 1024].iter() {
        let (ws, mut civ, gh, p) = generate_world(12345, 800.0, 24, gw);
        let no_polys: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut tf = Vec::new();
        let mut flows = Vec::new();
        {
            let uw = trade_world(&ws, &civ, gw, gh, &p, 12345, &no_polys);
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                flows = harness_flows(&civ, &uw);
                tf.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        let mut tb = Vec::new();
        for _ in 0..5 {
            civ.belief.clear();
            let t0 = std::time::Instant::now();
            civ.civ_belief_run(100, &flows);
            tb.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        tf.sort_by(|a, b| a.partial_cmp(b).unwrap());
        tb.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "COST gw {gw}: {} settlements, {} flows; belief run median {:.3} ms ({:.3}..{:.3}); \
             trade_flows median {:.3} ms ({:.3}..{:.3})",
            civ.settlements.len(),
            flows.len(),
            tb[2],
            tb[0],
            tb[4],
            tf[2],
            tf[0],
            tf[4]
        );
    }
}

/// The relations and belief fingerprints of one civ as it stands (roster,
/// custom religions and all): what FH-R2 R2-3's M5 bar compares.
fn relations_and_belief(civ: &mut crate::CivData, gh: usize, wrap: bool, flows: &[TradeFlow]) -> (u64, u64) {
    let cultures: Vec<&str> = civ.faction_roster.0.iter().map(|e| e.culture.as_str()).collect();
    let religions: Vec<&str> = civ.faction_roster.engine_religions(&civ.religions);
    let input = FactionRelationsInput {
        faction_count: civ.faction_roster.0.len(),
        gw: GW,
        gh,
        territory: Some(&civ.territory),
        cultures: &cultures,
        religions: &religions,
        wrap_x: wrap,
    };
    let empty = cartalith_civ::FactionAggregates {
        by_faction: Vec::new(),
        max_pop: 0.0,
        max_trade_volume: 0.0,
        max_territory_km2: 0.0,
        max_settlement_count: 0,
        world_mean_resource: Default::default(),
        world_mean_terrain: Default::default(),
    };
    let mut h = Fnv::new();
    for r in civ_faction_relations(&input, &empty).pairs.iter() {
        h.eat_str(&format!("{r:?}"));
    }
    let rel = h.0;
    civ.belief.clear();
    civ.civ_belief_run(100, flows);
    let mut h = Fnv::new();
    for s in civ.belief.iter() {
        for v in s.share.iter() {
            h.eat(&v.to_bits().to_le_bytes());
        }
    }
    (rel, h.0)
}

/// FH-R2 R2-3's **M5 bar** (`FH_R2_PLAN.md` §6): a custom religion whose
/// axis values equal its base's profile is byte-identical to its base in the
/// belief and relations fingerprints, and changing one belief-read axis moves
/// the belief run (and nothing the axes do not feed: relations). Over the
/// five seeds at 800 km with 24 factions, faction 1 is re-pointed at a custom
/// religion based on its own default religion, three ways: unedited; with
/// exclusivity explicitly set to the base's own value (a table is built and
/// must still equal the base bit for bit); with membership set to `birth`.
/// The recorded-only axes (object, locus, institution) are set on the third
/// variant's twin to show they move nothing. Ignored: it generates 5 worlds.
///
/// Protects: Q6's "base plus overrides" default is identity for an unedited
/// or base-equal custom religion, and the override actually reaches the run.
#[test]
#[ignore = "M5 measurement over 5 generated worlds; run alone with --ignored"]
fn fh_r2_m5_custom_axes() {
    use cartalith_civ::religion_library::custom_religion_key;
    use cartalith_civ::religion_profile::{
        religion_profile, AxisOverrides, Institution, Locus, Membership, WorshipObject,
    };
    let (mut moved, mut worlds) = (0usize, 0usize);
    for &seed in SEEDS.iter() {
        let (ws, mut civ, gh, p) = generate_world(seed, 800.0, 24, GW);
        let no_polys: Vec<Vec<(f64, f64)>> = Vec::new();
        let flows = {
            let uw = trade_world(&ws, &civ, GW, gh, &p, seed, &no_polys);
            harness_flows(&civ, &uw)
        };
        let base_key = civ.faction_roster.0[1].religion.clone();
        let base = religion_profile(&base_key).expect("faction 1 holds a faith with a profile");
        let held = civ.settlements.iter().filter(|s| s.placement.faction == 1).count();
        assert!(held > 0, "seed {seed}: faction 1 holds no settlement, the fixture proves nothing");
        let plain = relations_and_belief(&mut civ, gh, p.world, &flows);
        let id = civ.religions.create("Harness Variant", &base_key, (1, 2, 3), "").unwrap();
        let key = custom_religion_key(id);
        {
            let crate::CivData { faction_roster, religions, .. } = &mut civ;
            assert!(faction_roster.set_religion(1, &key, religions));
        }
        let unedited = relations_and_belief(&mut civ, gh, p.world, &flows);
        civ.religions
            .set_axes(id, AxisOverrides { exclusivity: Some(base.exclusivity), ..Default::default() })
            .unwrap();
        let same_value = relations_and_belief(&mut civ, gh, p.world, &flows);
        let recorded_only = AxisOverrides {
            object: Some(WorshipObject::Animism),
            locus: Some(Locus::Portable),
            institution: Some(Institution::Specialist),
            ..Default::default()
        };
        civ.religions.set_axes(id, recorded_only).unwrap();
        let recorded = relations_and_belief(&mut civ, gh, p.world, &flows);
        // A membership the base does not already have.
        let m = if base.membership == Some(Membership::Birth) { Membership::Universal } else { Membership::Birth };
        civ.religions.set_axes(id, AxisOverrides { membership: Some(m), ..Default::default() }).unwrap();
        let changed = relations_and_belief(&mut civ, gh, p.world, &flows);
        println!(
            "M5 seed {seed} base {base_key} ({held} settlements): plain {:016x}/{:016x} unedited {:016x} \
             same-value {:016x} recorded-only {:016x} membership={m:?} {:016x}",
            plain.0, plain.1, unedited.1, same_value.1, recorded.1, changed.1
        );
        assert_eq!(unedited, plain, "seed {seed}: an unedited custom religion moved");
        assert_eq!(same_value, plain, "seed {seed}: an override equal to the base moved");
        assert_eq!(recorded, plain, "seed {seed}: a recorded-only axis moved");
        assert_eq!(changed.0, plain.0, "seed {seed}: relations read no axis");
        worlds += 1;
        if changed.1 != plain.1 {
            moved += 1;
        }
    }
    println!("M5 TOTAL: membership override moved the belief run on {moved} of {worlds} worlds");
    assert_eq!(moved, worlds, "the override must reach every world's run");
}

/// FH-R2 R2-6's measurement (`FH_R2_PLAN.md` R2-6: "belief runs only"). The
/// plan names no numeric bar for S6; the bars below are **labelled
/// judgements** on the research document's direction ("persecution raises
/// minority out-flow, tolerant regimes lower it"): with every faction given
/// the same policy, the mean minority share after 200 years -- per settlement,
/// the share held by faiths other than its ruler's state religion and the
/// unaffiliated, over settlements whose ruler has a state religion -- must
/// order persecuting < exclusive establishment < plural. Tolerant
/// establishment is printed, not ordered (its two halves push opposite ways).
/// The all-plural case is the M1 harness's belief column (0 bytes moved, by
/// control flow: no table is built). Five seeds, 800 km, 24 factions.
///
/// Protects: the policy reaches real worlds' runs, in the stated direction.
#[test]
#[ignore = "R2-6 measurement over 5 generated worlds; run alone with --ignored"]
fn fh_r2_m6_policy_bars() {
    use cartalith_civ::belief::{religion_index, RELIGION_NONE};
    use cartalith_civ::religious_policy::CIV_RELIGIOUS_POLICIES;
    let mut means = Vec::new();
    for (key, _) in CIV_RELIGIOUS_POLICIES {
        let (mut sum, mut n) = (0.0f64, 0usize);
        for &seed in SEEDS.iter() {
            let (ws, mut civ, gh, p) = generate_world(seed, 800.0, 24, GW);
            let no_polys: Vec<Vec<(f64, f64)>> = Vec::new();
            let flows = {
                let uw = trade_world(&ws, &civ, GW, gh, &p, seed, &no_polys);
                harness_flows(&civ, &uw)
            };
            for f in 1..civ.faction_roster.0.len() {
                assert!(civ.faction_roster.set_field(f, "religious_policy", key));
            }
            civ.civ_belief_run(200, &flows);
            let states: Vec<Option<usize>> = civ
                .settlements
                .iter()
                .map(|s| {
                    let e = civ.faction_roster.0.get(s.placement.faction.max(0) as usize)?;
                    religion_index(civ.religions.engine_key(&e.religion)).filter(|&r| r != RELIGION_NONE)
                })
                .collect();
            for (i, st) in states.iter().enumerate() {
                let Some(state) = *st else { continue };
                let share = &civ.belief[i].share;
                let minority: f64 =
                    share.iter().enumerate().filter(|&(r, _)| r != state && r != RELIGION_NONE).map(|(_, v)| v).sum();
                sum += minority;
                n += 1;
            }
        }
        assert!(n > 0, "no settlement with a state religion");
        let mean = sum / n as f64;
        println!("M6 policy {key}: mean minority share {mean:.5} over {n} settlements (5 seeds)");
        means.push(mean);
    }
    let (plural, _tolerant, exclusive, persecuting) = (means[0], means[1], means[2], means[3]);
    assert!(persecuting < exclusive && exclusive < plural, "{means:?}");
}

/// FH-R2 S3a's **M3 bars** (`FH_R2_PLAN.md` §6), first measured for R2-4
/// with the rite built but not switched on, and re-run on 2026-10-25 for the
/// switch itself (Ruling CY item 1: `urban_layouts` now passes
/// `CivData::town_faith`). Over **every** settlement of five seeds x
/// {800 km, 40 075 km} x 24 factions, each town is laid out twice, with no
/// faith (the pre-switch call) and with the app's faith, and compared.
///
/// The no-rite and the four-rite populations are built here, since fresh
/// rosters only reach the temple and shrine religions: faction 1 is set to no
/// religion, 2 to `flame_creed` (a religion whose form is S4's), 3 to
/// `revealed_law` (mosque) and 4 to `one_god` (church, the medieval default).
///
/// - A town whose faction religion maps to no rite of its own (unaffiliated,
///   Unclaimed, or a religion without a building form yet) must be
///   **byte-identical** in buildings and faith sites -- it takes the same
///   call by control flow.
/// - Every other town's faith sites may differ **only in rite and its
///   consequences** (name, built form): the same count (still
///   `clamp(round(households/500), 1, 4)`), the same centres, ids and
///   churchyard parcels, and **byte-identical parcels**. Its buildings may
///   move, and every building that appears or vanishes must have its centroid
///   inside a site's building or open precinct polygon, before or after: the
///   only place `build_faith_sites` clears a house besides the (unchanged)
///   churchyard parcels. The count of moved houses not so explained is
///   asserted zero, and the count of moved houses non-zero (or the check
///   would measure nothing).
///
/// The JS-captured urban goldens are the third M3 bar and live in
/// `cartalith-urban` / `golden_parity_urban_adapter.rs`, which never set a
/// faith; the workspace suite running them unchanged is that bar.
///
/// Protects: S3a changes the rite and nothing else about a town.
#[test]
#[ignore = "M3 measurement over 10 generated worlds; run alone with --ignored"]
fn fh_r2_m3_rite_only() {
    use cartalith_civ::urban_adapter::{um_place_context_with, Building, PlaceOverrides, UrbanLayout, Vec2};
    // Area centroid and even-odd point-in-polygon: the two tests
    // `build_faith_sites` clears houses by (`poly_centroid`, `point_in_poly`
    // in `cartalith-urban`'s `geom`, which this crate does not depend on).
    fn centroid(p: &[Vec2]) -> Vec2 {
        let (mut a, mut cx, mut cy) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..p.len() {
            let (u, v) = (p[i], p[(i + 1) % p.len()]);
            let k = u.x * v.y - v.x * u.y;
            a += k;
            cx += (u.x + v.x) * k;
            cy += (u.y + v.y) * k;
        }
        if a.abs() < 1e-12 {
            let n = p.len().max(1) as f64;
            return Vec2::new(p.iter().map(|q| q.x).sum::<f64>() / n, p.iter().map(|q| q.y).sum::<f64>() / n);
        }
        Vec2::new(cx / (3.0 * a), cy / (3.0 * a))
    }
    fn inside(c: Vec2, p: &[Vec2]) -> bool {
        let mut hit = false;
        let mut j = p.len().wrapping_sub(1);
        for i in 0..p.len() {
            let (a, b) = (p[i], p[j]);
            if (a.y > c.y) != (b.y > c.y) && c.x < (b.x - a.x) * (c.y - a.y) / (b.y - a.y) + a.x {
                hit = !hit;
            }
            j = i;
        }
        hit
    }
    let under_a_site = |c: Vec2, l: &UrbanLayout| {
        l.faith_sites.iter().any(|s| s.polys.iter().chain(s.open.iter()).any(|p| inside(c, p)))
    };
    let (mut same, mut changed, mut sites_changed, mut builds_moved) = (0usize, 0usize, 0usize, 0usize);
    let (mut houses_moved, mut unexplained) = (0usize, 0usize);
    let mut rites: std::collections::BTreeMap<String, usize> = Default::default();
    let mut moved_by_rite: std::collections::BTreeMap<String, usize> = Default::default();
    for &seed in SEEDS.iter() {
        for &width in [800u32, 40_075].iter() {
            let (ws, mut civ, gh, p) = generate_world(seed, f64::from(width), 24, GW);
            assert!(civ.faction_roster.set_field(1, "religion", "none"));
            assert!(civ.faction_roster.set_field(2, "religion", "flame_creed"));
            assert!(civ.faction_roster.set_field(3, "religion", "revealed_law"));
            assert!(civ.faction_roster.set_field(4, "religion", "one_god"));
            let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
                (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, GW, gh, 1),
                _ => Vec::new(),
            };
            let uw = UrbanWorld {
                field: &ws.field,
                flow: &ws.flow_discharge,
                water_bodies: &civ.water_bodies,
                order: ws.stream_order.as_deref(),
                river_polys: &river_polys,
                gw: GW,
                gh,
                sea_level: ws.sea_level,
                map_width_km: p.map_width_km,
                flow_thresh: cartalith_hydrology::river_flow_thresh(GW, gh, GW, p.map_width_km),
                world_seed: seed,
            };
            for s in civ.settlements.iter() {
                let faith = civ.town_faith(s);
                let plain = run_layout(&um_place_context(&uw, s, &civ.ways), None);
                let app = run_layout(&um_place_context_with(&uw, s, &civ.ways, &PlaceOverrides { faith, ..Default::default() }), None);
                let (Some(a), Some(b)) = (plain, app) else { continue };
                let bld = |l: &UrbanLayout| format!("{:?}", l.buildings);
                match faith {
                    None => {
                        assert_eq!(bld(&a), bld(&b), "seed {seed} {width}: an unaffiliated town moved");
                        assert_eq!(format!("{:?}", a.faith_sites), format!("{:?}", b.faith_sites));
                        same += 1;
                    }
                    Some(r) => {
                        changed += 1;
                        *rites.entry(r.to_string()).or_default() += 1;
                        assert_eq!(a.faith_sites.len(), b.faith_sites.len(), "seed {seed}: site count moved");
                        // `LayoutParcel` has no `Debug`; every field it carries is compared.
                        let par = |l: &UrbanLayout| {
                            l.parcels.iter().map(|q| format!("{:?} {} {} {} {}", q.poly, q.area.to_bits(), q.edge_cls, q.tone.to_bits(), q.district)).collect::<Vec<_>>()
                        };
                        assert_eq!(par(&a), par(&b), "seed {seed}: parcels moved");
                        for (x, y) in a.faith_sites.iter().zip(b.faith_sites.iter()) {
                            assert_eq!((x.center, &x.yard, &x.id), (y.center, &y.yard, &y.id), "seed {seed}: a site moved");
                            assert_eq!(y.faith, r);
                            if x.faith != y.faith {
                                sites_changed += 1;
                            }
                        }
                        if bld(&a) != bld(&b) {
                            builds_moved += 1;
                            *moved_by_rite.entry(r.to_string()).or_default() += 1;
                            let key = |x: &Building| format!("{:?}", x.poly);
                            let ka: std::collections::HashSet<String> = a.buildings.iter().map(key).collect();
                            let kb: std::collections::HashSet<String> = b.buildings.iter().map(key).collect();
                            let only = a
                                .buildings
                                .iter()
                                .filter(|x| !kb.contains(&key(x)))
                                .chain(b.buildings.iter().filter(|x| !ka.contains(&key(x))));
                            for h in only {
                                houses_moved += 1;
                                let c = centroid(&h.poly);
                                if !under_a_site(c, &a) && !under_a_site(c, &b) {
                                    unexplained += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "M3: {same} towns with no rite of their own identical; {changed} towns given a rite {rites:?}; \
         {sites_changed} faith sites changed rite; {builds_moved} of those towns' building lists moved \
         {moved_by_rite:?}; {houses_moved} houses appeared or vanished, {unexplained} not under a site"
    );
    assert!(same > 0 && changed > 0, "both populations must be non-empty");
    assert!(houses_moved > 0, "the fixture must move a house, or the explained-by check measures nothing");
    assert_eq!(unexplained, 0, "every moved house must sit under a place of worship");
}
