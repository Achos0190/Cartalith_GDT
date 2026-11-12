//! **Ruling ES / ET follow-ups, measured: the playa tint, RR-5's lake tone by
//! depth and the wet bank's targets** (`RIVER_REALISM_SCOPE.md` RR-2 part 3,
//! RR-5; `LARGE_ITEM_RULINGS.md` Ruling ET's open playa follow-up).
//!
//! Per world (the owner world, seed 483920) it prints:
//! * the playa: cells, 4-connected components, the five largest, and the share
//!   of the old rule's lake cells the balance turned into playa;
//! * RR-5: the lake-tone distribution (`lake_depth_axis` over every lake cell),
//!   and, through `render::cell_color` on a real context, the colour spread
//!   between the deepest and the shallowest cell of the 20 largest lakes (the
//!   scope's bar: at least 12 levels for a lake deeper than 2 `LAKE_DEPTH`) and
//!   the largest per-channel move on the small ponds (the bar: within 4 levels
//!   of today's);
//! * and writes the probe's `--targets` file (`targets_<label>.json`): the
//!   largest playa, the largest lake's deepest cell and the widest trunk, for
//!   `_riverzoom_probe.gd --fu-frames`.
//!
//! **What it must never do.** Change generated output or pin a value; its
//! assertions only guard that the fixture reached the code (a playa, a lake and
//! a trunk exist). Never un-ignore it: it generates two large worlds.
//!
//! ```text
//! FU_DIR=<dir> cargo test --release -p cartalith-godot --test river_lake_followup_measurement -- --ignored --nocapture --test-threads=1
//! ```
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/render.rs"]
mod render;

use cartalith_civ::LAKE_DEPTH;
use cartalith_engine::generate_terrain;
use render::{LakeBalanceInputs, RenderCtx, TerrainAppearance};

/// `(seed, gw, gh, km, density, label)`: the owner world and seed 483920, as
/// `river_realism_measurement.rs` and `lake_balance_harness.rs` have them.
const WORLDS: [(i32, usize, usize, f64, f64, &str); 2] = [(246_371, 2048, 1311, 40_075.0, 1.55, "owner"), (483_920, 1024, 656, 1_200.0, 1.0, "483920")];

/// 4-connected components of the cells where `m` is set, largest first, each as
/// its cell list. Wrap is ignored (a measurement, not a rule).
fn components(m: &[bool], gw: usize, gh: usize) -> Vec<Vec<usize>> {
    let mut seen = vec![false; m.len()];
    let mut out = Vec::new();
    for s in 0..m.len() {
        if seen[s] || !m[s] {
            continue;
        }
        let mut cells = vec![s];
        seen[s] = true;
        let mut k = 0;
        while k < cells.len() {
            let i = cells[k];
            k += 1;
            let (x, y) = (i % gw, i / gw);
            let mut push = |j: usize| {
                if !seen[j] && m[j] {
                    seen[j] = true;
                    cells.push(j);
                }
            };
            if x > 0 {
                push(i - 1);
            }
            if x + 1 < gw {
                push(i + 1);
            }
            if y > 0 {
                push(i - gw);
            }
            if y + 1 < gh {
                push(i + gw);
            }
        }
        out.push(cells);
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.len()));
    out
}

/// The cell of `cells` nearest their centroid (so a ring-shaped component is
/// framed on one of its own cells).
fn central(cells: &[usize], gw: usize) -> usize {
    let (sx, sy) = cells.iter().fold((0.0, 0.0), |(a, b), &i| (a + (i % gw) as f64, b + (i / gw) as f64));
    let (cx, cy) = (sx / cells.len() as f64, sy / cells.len() as f64);
    *cells.iter().min_by(|&&a, &&b| ((a % gw) as f64 - cx).hypot((a / gw) as f64 - cy).total_cmp(&((b % gw) as f64 - cx).hypot((b / gw) as f64 - cy))).unwrap()
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

#[test]
#[ignore]
fn river_lake_followup_measurement() {
    let out = std::env::var("FU_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir().join("fu"));
    std::fs::create_dir_all(&out).expect("create output dir");
    let mut targets = serde_json::Map::new();
    for &(seed, gw, gh, km, density, label) in WORLDS.iter() {
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = km;
        p.river_density = density;
        p.use_gpu = false;
        assert!(p.lake_balance, "the app default has the lake balance on");
        let ws = generate_terrain(&p);
        let (sea, wrap, n) = (ws.sea_level, p.world, gw * gh);
        let table = ws.volcanoes.as_deref().map(|v| v.as_slice());
        let bal = LakeBalanceInputs::when(true, Some(&ws.flow_discharge), &ws.temperature);
        let wb = render::water_bodies_floored_balanced(&ws.field, &ws.rainfall, gw, gh, sea, wrap, None, table, bal);
        let old = render::water_bodies_floored_balanced(&ws.field, &ws.rainfall, gw, gh, sea, wrap, None, table, None);
        let lake_cells = wb.classification.iter().filter(|&&c| c == 2).count();
        let old_lake_cells = old.classification.iter().filter(|&&c| c == 2).count();
        let playa_cells = wb.playa.iter().filter(|&&c| c != 0).count();
        assert_eq!(wb.playa.len(), n, "{label}: the balanced classification carries a playa mask");
        assert!(old.playa.is_empty(), "{label}: the unbalanced one carries none");
        let pc = components(&wb.playa.iter().map(|&c| c != 0).collect::<Vec<_>>(), gw, gh);
        println!("== {label} {gw}x{gh}: lake cells old {old_lake_cells} balanced {lake_cells}; playa cells {playa_cells} ({:.1} % of the old lake cells), {} components, largest {:?}", 100.0 * playa_cells as f64 / old_lake_cells.max(1) as f64, pc.len(), pc.iter().take(5).map(|c| c.len()).collect::<Vec<_>>());
        assert!(playa_cells > 0 && !pc.is_empty(), "{label}: no playa -- the fixture did not reach the code");

        // RR-5: the tone over every lake cell, and the lakes.
        // Above-sea lakes only: a below-sea lake (an inland sea) is drawn by the sea branch, which RR-5 does not touch.
        let lakes = components(&(0..n).map(|i| wb.classification[i] == 2 && f64::from(ws.field[i]) > sea).collect::<Vec<_>>(), gw, gh);
        assert!(!lakes.is_empty(), "{label}: no lake");
        let u = |i: usize| (f64::from(wb.fill_level[i]) - f64::from(ws.field[i])) / LAKE_DEPTH;
        let mut tones: Vec<f64> = (0..n).filter(|&i| wb.classification[i] == 2 && f64::from(ws.field[i]) > sea).map(|i| render::lake_depth_axis(u(i))).collect();
        println!("   lake tone axis over {} lake cells: p5 {:.3} p25 {:.3} p50 {:.3} p75 {:.3} p95 {:.3} (today's constant 0.300)", tones.len(), pct(&mut tones, 0.05), pct(&mut tones, 0.25), pct(&mut tones, 0.5), pct(&mut tones, 0.75), pct(&mut tones, 0.95));

        // Real colours through cell_color.
        let on = TerrainAppearance::default();
        let off = TerrainAppearance { lake_depth_tone: 0.0, ..TerrainAppearance::default() };
        let mk = |a: TerrainAppearance| RenderCtx::with_appearance(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, sea, wrap, 55.0, 5.0, a).with_lakes(&wb.classification).with_water_detail(&wb.fill_level, &wb.playa);
        let (c_on, c_off) = (mk(on), mk(off));
        let lv = |c: (f64, f64, f64)| (c.0 * 255.0, c.1 * 255.0, c.2 * 255.0);
        let sad = |a: (f64, f64, f64), b: (f64, f64, f64)| (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs();
        let mut spreads = Vec::new();
        for l in lakes.iter().take(20) {
            let deepest = *l.iter().max_by(|&&a, &&b| u(a).total_cmp(&u(b))).unwrap();
            let shallow = *l.iter().min_by(|&&a, &&b| u(a).total_cmp(&u(b))).unwrap();
            if u(deepest) > 2.0 {
                let (d, s) = (lv(render::cell_color(&c_on, deepest % gw, deepest / gw)), lv(render::cell_color(&c_on, shallow % gw, shallow / gw)));
                spreads.push((sad(d, s), l.len(), u(deepest), u(shallow)));
            }
        }
        println!("   deepest vs shallowest cell of the largest lakes (summed RGB levels, cells, u deep, u shallow): {:?}", spreads.iter().map(|s| (s.0.round(), s.1, s.2.round(), s.3.round())).collect::<Vec<_>>());
        let mut max_pond = 0.0f64;
        let mut ponds = 0usize;
        for l in lakes.iter().filter(|l| l.len() <= 4 && l.iter().all(|&i| u(i) <= 4.0)) {
            ponds += 1;
            for &i in l {
                let (a, b) = (lv(render::cell_color(&c_on, i % gw, i / gw)), lv(render::cell_color(&c_off, i % gw, i / gw)));
                max_pond = max_pond.max((a.0 - b.0).abs().max((a.1 - b.1).abs()).max((a.2 - b.2).abs()));
            }
        }
        println!("   ponds of 1-4 cells with u <= 4: {ponds}; largest per-channel move against today's constant: {max_pond:.1} levels (bar: within 4)");

        // Targets for the probe, each framed where the effect shows: a playa cell on the playa's edge nearest its centroid
        // (the contour), the largest lake's cell whose depth is nearest 8 `LAKE_DEPTH` on its shoreward side (the tone
        // gradient), and the highest-discharge channel cell at least 10 % of the grid from every edge (not on the plate frame).
        let big = &pc[0];
        let cen = central(big, gw);
        let edge_pl: Vec<usize> = big.iter().copied().filter(|&i| { let (x, y) = (i % gw, i / gw); [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)].iter().any(|&(a, b)| a < gw && b < gh && wb.playa[b * gw + a] == 0) }).collect();
        let pl = *edge_pl.iter().min_by(|&&a, &&b| ((a % gw) as f64 - (cen % gw) as f64).hypot((a / gw) as f64 - (cen / gw) as f64).total_cmp(&((b % gw) as f64 - (cen % gw) as f64).hypot((b / gw) as f64 - (cen / gw) as f64))).unwrap_or(&cen);
        // The lake of 50 cells or more with the deepest bed (the owner world's biggest lake is shallow and shows no tone).
        let deep_lake = lakes.iter().filter(|l| l.len() >= 50).max_by(|a, b| a.iter().map(|&i| u(i)).fold(0.0, f64::max).total_cmp(&b.iter().map(|&i| u(i)).fold(0.0, f64::max))).unwrap_or(&lakes[0]);
        let lk = *deep_lake.iter().min_by(|&&a, &&b| (u(a) - 8.0).abs().total_cmp(&(u(b) - 8.0).abs())).unwrap();
        let chan = ws.channels.as_ref().map(|c| c.chan.clone()).unwrap_or_default();
        let inside = |i: usize| { let (x, y) = (i % gw, i / gw); x * 10 >= gw && x * 10 <= gw * 9 && y * 10 >= gh && y * 10 <= gh * 9 };
        let trunk = (0..n).filter(|&i| ws.field[i] as f64 > sea && inside(i) && chan.get(i).is_some_and(|&c| c != 0)).max_by(|&a, &b| ws.flow_discharge[a].total_cmp(&ws.flow_discharge[b])).expect("a channel cell");
        let cell = |i: usize| serde_json::json!({"cell": [(i % gw) as f64 + 0.5, (i / gw) as f64 + 0.5]});
        println!("   targets: playa {} cells at ({}, {}); largest lake {} cells, target cell ({}, {}) u {:.0}; trunk ({}, {})", big.len(), pl % gw, pl / gw, lakes[0].len(), lk % gw, lk / gw, u(lk), trunk % gw, trunk / gw);
        targets.insert(seed.to_string(), serde_json::json!({"targets": {"playa": cell(pl), "lake": cell(lk), "trunk": cell(trunk)}}));
    }
    std::fs::write(out.join("targets.json"), serde_json::to_string_pretty(&serde_json::Value::Object(targets)).unwrap()).expect("write targets");
}
