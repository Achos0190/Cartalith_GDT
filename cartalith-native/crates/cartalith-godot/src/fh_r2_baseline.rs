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
//!   after the vocabulary grows.
//! - **urban** -- `run_layout` for the three most populous settlements (ties
//!   by index), every `Building` by `Debug` (kind, district, polygon, ridge,
//!   age). This is where a faith site's rite would show; `run_layout` passes
//!   `faith: None` today, so it is predicted unmoved by construction and this
//!   measures that prediction.
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
use cartalith_civ::urban_adapter::{run_layout, um_place_context, UrbanWorld};

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

/// Generates one world through the app's parameters and fingerprints it.
fn fingerprint(seed: i32, width_km: f64, factions: i32) -> Print {
    let world = width_km >= 40_075.0;
    let gh = crate::reference_grid_h(GW, world);
    let mut p = crate::with_call_args(crate::params::defaults(), seed, width_km, GW, gh);
    p.world = world;
    p.civ.factions = factions;
    p.use_gpu = false;
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, GW, gh, p.world, ws.sea_level, &p);
    let mut civ = crate::compute_civilisation(
        &ws, GW, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false,
        &mut Vec::new(), None,
    );

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
        let ctx = um_place_context(&uw, &civ.settlements[i], &civ.ways);
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

    // belief
    civ.civ_belief_run(100);
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

/// Protects: R2-1 (`FH_R2_PLAN.md` M1) moves zero bytes of the roster's
/// religion/government picks, the relations they feed, a 100-year belief run,
/// and the three largest towns' buildings, on 20 real worlds, against the M0
/// literals recorded before the change. Ignored: it generates 20 worlds.
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
    for (got, want) in rows.iter().zip(M0.iter()) {
        if got != want {
            moved.push(format!("got {got:?} want {want:?}"));
        }
    }
    assert!(moved.is_empty(), "M1: {} of 20 worlds moved:\n{}", moved.len(), moved.join("\n"));
}
