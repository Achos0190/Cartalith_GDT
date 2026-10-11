//! **Ruling CC's lake basin-area floor, measured on the lake-candidates
//! harness's five worlds** (`lake_candidates_harness.rs`'s `CASES`), at the
//! SCALED floor the engine ships (`cartalith_civ::lake_area_floor_cells`), not
//! the harness's fixed 4/10/25 floors.
//!
//! Protects: the shipped pass (`cartalith_civ::apply_lake_area_floor`) on real
//! worlds, beyond the unit fixtures: it never adds a lake cell, never touches
//! ocean, `fill_level` or `basin`, never touches a lake body with a cell below
//! sea, never makes a traced river MORE broken (no river's count of drawn
//! pieces rises; a piece boundary is where its walk enters water), and leaves
//! the height field, and therefore the rivers' routing surface, bit-identical.
//! A world whose scaled floor is 1 must remove nothing; a world with 1-3-cell
//! lakes and a floor above 1 must remove something (positive control).
//!
//! ```text
//! cargo test -p cartalith-godot --release --test lake_floor_measurement -- --ignored --nocapture
//! ```
//!
//! Definitions are the harness's (its module doc): lake bodies are 4-connected
//! `classification == 2`; the river-path lake share is `b8`'s walk over
//! `river_entities(min order 1)` points, all but the last; a lake body is
//! Ruling-BY exempt when every cell lies in an existing depression of the
//! structural surface (erosion and carving off).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_civ::{apply_lake_area_floor, build_water_bodies, lake_area_floor_cells};
use cartalith_engine::{WorldState, generate_terrain};

/// One measured world (the harness's `Case`, without the unused humidity knob).
struct Case {
    seed: i32,
    gw: usize,
    km: f64,
}

/// The harness's five worlds, verbatim: three at 2048 x 1311 and 800 km
/// (`river_gap_harness.rs`), 246371 at 512 x 328 and 40 075 km and 483920 at
/// 384 x 246 and 800 km (`river_comb_harness.rs`).
const CASES: [Case; 5] = [
    Case { seed: 483_920, gw: 2048, km: 800.0 },
    Case { seed: 24_601, gw: 2048, km: 800.0 },
    Case { seed: 71_077_345, gw: 2048, km: 800.0 },
    Case { seed: 246_371, gw: 512, km: 40_075.0 },
    Case { seed: 483_920, gw: 384, km: 800.0 },
];

/// The app's world for `c` and its structural surface, as the harness builds
/// them (`worlds`), plus the wrap flag.
fn worlds(c: &Case) -> (WorldState, WorldState, bool, usize) {
    let mut p = params::defaults();
    p.gw = c.gw;
    p.gh = ((c.gw as f64 * 0.64).round() as usize).max(4);
    p.tect.seed = c.seed;
    p.map_width_km = c.km;
    p.use_gpu = false;
    let ws = generate_terrain(&p);
    let mut pp = p.clone();
    pp.carve_rivers = false;
    pp.passes = cartalith_engine::ErosionPassParams::off();
    let pre = generate_terrain(&pp);
    (ws, pre, p.world, p.gh)
}

/// 4-neighbours of `i`, x wrapping when `wrap` (the classifier's rule).
fn nb4(i: usize, gw: usize, gh: usize, wrap: bool) -> impl Iterator<Item = usize> {
    let (x, y) = ((i % gw) as i64, (i / gw) as i64);
    [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)].into_iter().filter_map(move |(dx, dy)| {
        let (mut nx, ny) = (x + dx, y + dy);
        if wrap {
            nx = nx.rem_euclid(gw as i64);
        }
        (nx >= 0 && ny >= 0 && (nx as usize) < gw && (ny as usize) < gh).then(|| ny as usize * gw + nx as usize)
    })
}

/// 4-connected bodies of `classification == 2`. Asserts it found every lake
/// cell (a silently-empty helper is the failure `MISTAKES.md` names).
fn bodies(class: &[u8], gw: usize, gh: usize, wrap: bool) -> Vec<Vec<usize>> {
    let n = gw * gh;
    let mut seen = vec![false; n];
    let mut out = Vec::new();
    for s in 0..n {
        if seen[s] || class[s] != 2 {
            continue;
        }
        seen[s] = true;
        let mut b = Vec::new();
        let mut st = vec![s];
        while let Some(i) = st.pop() {
            b.push(i);
            for j in nb4(i, gw, gh, wrap) {
                if !seen[j] && class[j] == 2 {
                    seen[j] = true;
                    st.push(j);
                }
            }
        }
        out.push(b);
    }
    assert_eq!(out.iter().map(Vec::len).sum::<usize>(), class.iter().filter(|&&k| k == 2).count(), "bodies() lost cells");
    out
}

/// FNV-1a over the bit patterns of an `f32` raster: a cheap identity check.
fn hash_f32(v: &[f32]) -> u64 {
    v.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, x| (h ^ x.to_bits() as u64).wrapping_mul(0x0100_0000_01b3))
}

/// Drawn pieces of one river walk under `wet`: maximal runs of dry points.
fn pieces(walk: &[usize], wet: &dyn Fn(usize) -> bool) -> usize {
    let mut k = 0;
    let mut in_dry = false;
    for &i in walk {
        let dry = !wet(i);
        if dry && !in_dry {
            k += 1;
        }
        in_dry = dry;
    }
    k
}

/// Protects: see the module doc. The measurement.
#[test]
#[ignore = "lake-floor measurement: two full generations per world, five worlds; run alone, --release"]
fn lake_floor_measurement() {
    for case in &CASES {
        let (ws, pre, wrap, gh) = worlds(case);
        let gw = case.gw;
        let n = gw * gh;
        let sea = ws.sea_level;
        let floor = lake_area_floor_cells(gw, gh);
        let mut wb = build_water_bodies(&ws.field, gw, gh, sea, wrap, Some(&ws.rainfall));
        let before = wb.classification.clone();
        let (fill0, basin0) = (hash_f32(&wb.fill_level), wb.basin.clone());
        let (field0, route0) = (hash_f32(&ws.field), hash_f32(&cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, sea, wrap)));
        let rep = apply_lake_area_floor(&mut wb, &ws.field, gw, gh, sea, wrap, floor);
        let after = wb.classification.clone();

        // Invariants.
        assert!((0..n).all(|i| after[i] == before[i] || (before[i] == 2 && after[i] == 0)), "only lake -> land may change");
        assert_eq!(hash_f32(&wb.fill_level), fill0, "fill_level untouched");
        assert_eq!(wb.basin, basin0, "basin untouched");
        assert_eq!(hash_f32(&ws.field), field0, "height field untouched");
        let route1 = hash_f32(&cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, sea, wrap));
        assert_eq!(route1, route0, "routing surface unchanged");
        let b0 = bodies(&before, gw, gh, wrap);
        assert!(!b0.is_empty(), "seed {}: no lakes at all", case.seed);
        for b in &b0 {
            let below = b.iter().any(|&i| (ws.field[i] as f64) < sea);
            if below {
                assert!(b.iter().all(|&i| after[i] == 2), "a lake with a below-sea cell was touched");
            }
            let lost = b.iter().filter(|&&i| after[i] != 2).count();
            assert!(lost == 0 || lost == b.len(), "a body was shrunk, not removed whole");
        }
        let b1 = bodies(&after, gw, gh, wrap);
        let small = |bs: &[Vec<usize>]| bs.iter().filter(|b| b.len() <= 3).count();
        if floor <= 1 {
            assert_eq!(rep.cells_removed, 0, "a 1-cell floor must remove nothing");
        } else if small(&b0) > 0 {
            assert!(rep.lakes_removed > 0, "positive control: floor {floor} removed nothing on a world with 1-3-cell lakes");
        }
        assert_eq!(b0.len() - b1.len(), rep.lakes_removed, "report counts the removed bodies");

        // Ruling BY exemption mask on the structural surface (the harness's).
        let wb0 = build_water_bodies(&pre.field, gw, gh, pre.sea_level, wrap, None);
        let removed_exempt = b0.iter().filter(|b| after[b[0]] != 2 && b.iter().all(|&i| wb0.fill_level[i] > pre.field[i])).count();

        // Rivers: b8's walk, per river.
        let order = ws.stream_order.as_ref().expect("river extraction ran");
        let ch = ws.channels.as_ref().expect("channel tree retained");
        let rivers = cartalith_hydrology::river_entities(
            order,
            &ch.recv,
            &ws.flow_discharge,
            &ws.field,
            gw,
            gh,
            1,
            cartalith_hydrology::river_flow_thresh(gw, gh, gw, case.km),
            cartalith_hydrology::river_width_scale_k(case.km),
            wrap,
        );
        let walks: Vec<Vec<usize>> = rivers
            .iter()
            .map(|r| r.pts.iter().take(r.pts.len().saturating_sub(1)).map(|&(x, y)| (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1)).collect())
            .collect();
        let walk_len: usize = walks.iter().map(Vec::len).sum();
        assert!(walk_len > 100, "a world with no rivers measures nothing");
        let share = |c: &[u8]| 100.0 * walks.iter().flatten().filter(|&&i| c[i] == 2).count() as f64 / walk_len as f64;
        let river_freed = walks.iter().flatten().filter(|&&i| before[i] == 2 && after[i] != 2).count();
        let (mut p0, mut p1, mut worse) = (0usize, 0usize, 0usize);
        for w in &walks {
            let a = pieces(w, &|i| before[i] != 0);
            let b = pieces(w, &|i| after[i] != 0);
            p0 += a;
            p1 += b;
            worse += (b > a) as usize;
        }
        assert_eq!(worse, 0, "a removed lake made a river more broken");

        println!(
            "seed {} {gw}x{gh} {} km: floor {floor} cells; removed {} lakes ({} cells), {} Ruling-BY exempt; lakes {} -> {}; 1-3-cell lakes {} -> {}; river-path lake share {:.2} % -> {:.2} % ({} river-path cells freed); drawn river pieces {p0} -> {p1}, rivers more broken {worse}; field/routing/fill/basin hashes unchanged",
            case.seed,
            case.km,
            rep.lakes_removed,
            rep.cells_removed,
            removed_exempt,
            b0.len(),
            b1.len(),
            small(&b0),
            small(&b1),
            share(&before),
            share(&after),
            river_freed,
        );
    }
}
