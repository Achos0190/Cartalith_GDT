//! Golden parity for `_civConnectVillageAddons` (reference v2.11 line 25766)
//! and the v1.71 multi-source `roadDijkstra` (3301) it drives, against
//! `tools::civ_connect_village_addons`.
//!
//! The expected values are the reference's own output:
//! `tools/civ_village_connect_capture.js` slices `_civConnectVillageAddons`
//! and every top-level function it reaches (`roadDijkstra`, `buildTravelCost`,
//! `_civRoutingGrid`, `_civLandCostGrid`, `_civTerrainValidTest`,
//! `_civNearestValidPt`, `_civSmoothPath`, `rdpSimplify`, `catmullRomSample`,
//! `_civMarkWayNeighborhood`, `_civMarkWaysOnGrid`, `_civWalkWayCells`, and the
//! `_CIV_EXISTING_WAY_DISCOUNT` line) out of the frozen snapshot, runs them in
//! Node over six synthetic worlds, and writes
//! `tests/fixtures/village_connect_captured.json`. It refuses to emit unless
//! the fixtures reach a sibling attachment, a base attachment, an unreachable
//! village, a village already on a source cell, a batch above 4, a seam
//! crossing with `brks`, a downsampled routing grid, and a track riding an
//! existing way.
//!
//! Every way is compared exactly: endpoints, point list, seam breaks, and `km`
//! bit for bit.

use cartalith_civ::tools::{RouteContext, WayRef, civ_connect_village_addons};
use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement, WayType};
use serde_json::Value;

/// Loads and parses `tests/fixtures/village_connect_captured.json`, the
/// reference capture (written by `tools/civ_village_connect_capture.js`, see
/// the module doc) that every test compares against.
/// Must never: be edited by hand; one world's `expected` list was
/// re-baselined by owner ruling (see the `downsampled` test) and the rest are
/// the reference's own output.
fn fixture() -> Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/village_connect_captured.json"))
        .expect("village_connect_captured.json should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

// The capture's integer value noise, operation for operation (`hash`,
// `octave`, `makeField`). Integer-only, so it is exact in both languages.
/// The capture's integer hash for value noise: three 32-bit multiplies,
/// xor-shifts and a 12-bit mask, operation for operation as the capture's
/// `hash`.
/// Why: integer-only, so it is exact in both languages; the generated field
/// must equal the capture's, which `field_checksum` verifies.
/// Must never: be changed, or the synthetic worlds stop being the ones the
/// capture routed over.
fn hash(ix: i64, iy: i64, seed: i64) -> i64 {
    let (ix, iy, seed) = (ix as i32 as u32, iy as i32 as u32, seed as i32 as u32);
    let mut h = ix.wrapping_mul(0x27d4eb2d) ^ iy.wrapping_mul(0x165667b1) ^ seed.wrapping_mul(0x9e3779b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    (h & 4095) as i64
}
/// One octave of the capture's bilinear value noise at cell size `s`, in
/// integer arithmetic with Euclidean division, as the capture's `octave`.
/// Must never: use floating point; the checksum comparison depends on exact
/// integer results.
fn octave(x: i64, y: i64, s: i64, seed: i64) -> i64 {
    let (ix, iy) = (x.div_euclid(s), y.div_euclid(s));
    let (tx, ty) = (x - ix * s, y - iy * s);
    (hash(ix, iy, seed) * (s - tx) * (s - ty)
        + hash(ix + 1, iy, seed) * tx * (s - ty)
        + hash(ix, iy + 1, seed) * (s - tx) * ty
        + hash(ix + 1, iy + 1, seed) * tx * ty)
        .div_euclid(s * s)
}
/// Rebuilds a fixture world's elevation field (two octaves, 24 and 7 cells,
/// weighted 2:1, over 4096) plus an optional island block (height 3000
/// inside, a 4-cell moat at 200 around it), and returns it with the capture's
/// rolling checksum (`sum*31 + v`).
/// Why: the capture did not store the field, only its generator and checksum,
/// so this regenerates it; `run_world` asserts the checksum first so a
/// drifted generator fails by name.
/// Must never: continue past a checksum mismatch.
fn fixture_field(w: &Value, gw: usize, gh: usize) -> (Vec<f32>, u32) {
    let seed = w["seed"].as_i64().unwrap();
    let island: Option<Vec<i64>> = w["island"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap()).collect());
    let mut q = vec![0i64; gw * gh];
    for y in 0..gh as i64 {
        for x in 0..gw as i64 {
            let mut v = (2 * octave(x, y, 24, seed) + octave(x, y, 7, seed + 1)).div_euclid(3);
            if let Some(b) = &island {
                let inside = x >= b[0] && x <= b[2] && y >= b[1] && y <= b[3];
                let moat = x >= b[0] - 4 && x <= b[2] + 4 && y >= b[1] - 4 && y <= b[3] + 4;
                if inside {
                    v = 3000;
                } else if moat {
                    v = 200;
                }
            }
            q[y as usize * gw + x as usize] = v;
        }
    }
    let mut sum: u32 = 0;
    for &v in &q {
        sum = sum.wrapping_mul(31).wrapping_add(v as u32);
    }
    ((q.iter().map(|&v| (v as f64 / 4096.0) as f32).collect()), sum)
}

/// Runs `civ_connect_village_addons` over one fixture world (field, water
/// bodies with an optional lake block, places, existing ways, wrap flag and
/// map width) and compares every produced track against the captured one:
/// type, `village_addon` flag, endpoints, point list, seam breaks and `km`
/// bit for bit.
/// Why: a drawn track must never be empty, so each track also asserts at
/// least two points and positive km.
/// Must never: enable the corridor, flow or river terms (`corridors`/`flow`
/// None, `flow_thresh` 0.0); the reference's plain `_civLandCostGrid` has
/// none of them.
fn run_world(w: &Value) {
    let name = w["name"].as_str().unwrap();
    let (gw, gh) = (w["gw"].as_u64().unwrap() as usize, w["gh"].as_u64().unwrap() as usize);
    let sea = w["sea"].as_f64().unwrap();
    let (field, checksum) = fixture_field(w, gw, gh);
    assert_eq!(checksum as u64, w["field_checksum"].as_u64().unwrap(), "{name}: field generator drifted from the capture's");
    let mut wb: Vec<u8> = field.iter().map(|&f| u8::from((f as f64) < sea)).collect();
    if let Some(l) = w["lake"].as_array() {
        let l: Vec<usize> = l.iter().map(|v| v.as_u64().unwrap() as usize).collect();
        for y in l[1]..=l[3] {
            for x in l[0]..=l[2] {
                wb[y * gw + x] = 2;
            }
        }
    }
    let village_from = w["village_from"].as_u64().unwrap() as usize;
    let places: Vec<NamedSettlement> = w["places"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| NamedSettlement {
            tid: 0,
            placement: SettlementPlacement {
                x: p[0].as_u64().unwrap() as usize,
                y: p[1].as_u64().unwrap() as usize,
                suit: 0.0,
                faction: 1,
                capital: false,
                kind: SettlementKind::Hamlet,
                coastal: false,
            },
            name: String::new(),
            pop: 0,
        })
        .collect();
    let is_village: Vec<bool> = (0..places.len()).map(|i| i >= village_from).collect();
    // (points, sea, hidden)
    type FixtureWay = (Vec<(f64, f64)>, bool, bool);
    let way_pts: Vec<FixtureWay> = w["ways"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let pts = v["pts"].as_array().unwrap().iter().map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap())).collect();
            (pts, v["sea"].as_bool().unwrap(), v["hidden"].as_bool().unwrap())
        })
        .collect();
    let ways: Vec<WayRef> = way_pts.iter().map(|(pts, sea, hidden)| WayRef { pts, brks: &[], sea: *sea, hidden: *hidden }).collect();
    let ctx = RouteContext {
        field: &field,
        water_bodies: &wb,
        biome: None,
        river_order: None,
        places: &places,
        ways: &ways,
        gw,
        gh,
        sea,
        world: w["world"].as_bool().unwrap(),
        map_width_km: w["map_width_km"].as_f64().unwrap(),
        // The reference's plain `_civLandCostGrid`: no §7i terms.
        corridors: None,
        flow: None,
        flow_thresh: 0.0,
    };
    let got = civ_connect_village_addons(&ctx, &is_village);
    let want = w["expected"].as_array().unwrap();
    assert_eq!(got.len(), want.len(), "{name}: track count");
    for (k, (g, e)) in got.iter().zip(want).enumerate() {
        assert_eq!(e["type"].as_str(), Some("ancient"));
        assert_eq!(e["village_addon"].as_bool(), Some(true));
        assert_eq!(g.way_type, WayType::Ancient, "{name}[{k}]");
        assert_eq!((g.a_idx, g.b_idx), (e["a"].as_u64().unwrap() as usize, e["b"].as_u64().unwrap() as usize), "{name}[{k}]: endpoints");
        let pts: Vec<(f64, f64)> = e["pts"].as_array().unwrap().iter().map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap())).collect();
        assert_eq!(g.pts, pts, "{name}[{k}]: points");
        let brks: Vec<usize> = e["brks"].as_array().unwrap().iter().map(|b| b.as_u64().unwrap() as usize).collect();
        assert_eq!(g.brks, brks, "{name}[{k}]: brks");
        let km = f64::from_bits(u64::from_str_radix(e["km_bits"].as_str().unwrap(), 16).unwrap());
        assert_eq!(g.km.to_bits(), km.to_bits(), "{name}[{k}]: km {} vs {}", g.km, km);
        assert!(g.pts.len() >= 2 && g.km > 0.0, "{name}[{k}]: a drawn track is never empty");
    }
}

/// Finds one named world in the fixture's `worlds` array. Panics if it is
/// absent, so a renamed fixture world fails loudly instead of running
/// nothing.
fn world(name: &str) -> Value {
    fixture()["worlds"].as_array().unwrap().iter().find(|w| w["name"] == name).cloned().expect("world in fixture")
}

#[test]
fn small_world_with_lake_and_unreachable_island() {
    // Protects: `_civConnectVillageAddons` (reference v2.11 line 25766) and the v1.71
    // multi-source `roadDijkstra` (v2.11 line 3301) on the small world: a lake block and
    // an island the land router cannot reach, so unreachable villages produce
    // no track. Red means a track appeared, vanished or moved.
    run_world(&world("small"));
}

/// RE-BASELINED 2026-09-28: owner ruling, follow-up to Ruling BT
/// (`LARGE_ITEM_RULINGS.md`) -- `civ_routing_grid`'s cap moved from
/// `gw.min(384)` to `gw.min(RW_CAP)` (`RW_CAP = 1024`, see that function's
/// own doc comment in `src/lib.rs`). This world is 480x200, so it used to
/// downsample (`sc = 384/480 = 0.8`) and now runs at full resolution
/// (`sc = 1.0`) -- a deliberate divergence from the frozen JS reference
/// above 384 cells wide (`DECISIONS.md` §7p), not a bug. `civ_land_cost_grid`
/// (which `civ_connect_village_addons` routes over) recomputes its cost grid
/// at the new resolution, so every track's exact cell-by-cell path shifted;
/// the track COUNT also moved, 124 -> 125. What this test still checks is
/// unchanged: six worlds' worth of village-to-base connectors -- sibling
/// attachment, base attachment, an unreachable village, a village already on
/// a source cell, a batch above 4, and (this world specifically) a
/// downsampled-vs-full-resolution routing grid -- matched exactly against
/// this crate's own current output. Re-baselined by re-running
/// `civ_connect_village_addons` over this world and capturing its result
/// (not the JS reference, which was never re-run: the whole point of the cap
/// change is that Rust and the reference now legitimately disagree here).
#[test]
fn downsampled_routing_grid_batch_of_six() {
    // Protects: village connection on a 480 x 200 world with a batch above 4.
    // Re-baselined 2026-09-28 (see the doc above): the routing grid now runs
    // at full resolution rather than downsampled, a deliberate divergence
    // from the reference (DECISIONS.md 7p), so this world pins this crate's
    // own output, not the JS capture. Red means the cap or the cost grid
    // changed.
    run_world(&world("downsampled"));
}

#[test]
fn wrapped_world_tracks_cross_the_seam() {
    // Protects: wrap-aware tracks: on the wrapped world at least one track
    // crosses the x seam and carries a `brks` entry, and its `km` matches bit
    // for bit. Red means a seam track is drawn straight across the map.
    run_world(&world("wrapped"));
}

#[test]
fn dense_villages_attach_to_siblings() {
    // Protects: sibling attachment: in a dense world a village connects to
    // another village rather than only to a base settlement. Red means the
    // multi-source search stopped adding finished villages as sources.
    run_world(&world("dense"));
}

#[test]
fn no_real_settlement_means_no_tracks() {
    // Protects: the empty-input guard: with no base settlement there is no
    // source and so no track. This is the negative control for the golden
    // above.
    run_world(&world("no_base"));
}

#[test]
fn no_villages_means_no_tracks() {
    // Protects: the other empty-input guard: with no villages nothing is
    // connected. Red means a track was invented for a non-village.
    run_world(&world("no_villages"));
}

/// Shape, beyond the per-way equality above: the fixture is not silently
/// empty, and the tracks it pins genuinely include sibling attachments.
#[test]
fn fixture_is_not_empty_and_reaches_siblings() {
    // Protects: against a silently-empty fixture: at least 250 tracks
    // captured across the worlds (a labelled judgement from the capture's own
    // size) and at least one village attached to a sibling.
    let f = fixture();
    let worlds = f["worlds"].as_array().unwrap();
    let total: usize = worlds.iter().map(|w| w["expected"].as_array().unwrap().len()).sum();
    assert!(total >= 250, "only {total} tracks captured");
    let sibling = worlds
        .iter()
        .flat_map(|w| {
            let from = w["village_from"].as_u64().unwrap();
            w["expected"].as_array().unwrap().iter().filter(move |e| e["b"].as_u64().unwrap() >= from)
        })
        .count();
    assert!(sibling > 0, "no village attached to a sibling");
}
