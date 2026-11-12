//! **RR-0: river realism measured before anything is pinned**
//! (`RIVER_REALISM_SCOPE.md` §4 RR-0; owner Ruling ES, 2026-11-12: "RR-0
//! (measure first) comes first, as the scope says").
//!
//! Per world it prints, over the drawn network:
//! * the distribution of `Q / Q_max` over drawn length, and the full widths in
//!   pixels it implies at x1, x4, x16 and x64 under RR-1's law
//!   (`river_realism::hydraulic_full_width`) at `W_MAX_CELLS` 1 and 2, beside
//!   the old `channel_disc` profile's -- each as its **width classes**
//!   (distinct whole-pixel widths, after the 1 px floor, holding at least
//!   [`CLASS_MIN_SHARE`] of drawn length): RR-1's bar (a) counts these;
//! * the along-course gradient distribution, the share of drawn length RR-4
//!   marks as white water, and the falls it draws by cause (slope, knickpoint,
//!   hanging valley), plus how many of the landmark detector's waterfall
//!   candidates (`waterfall_constraints_hold` at a channel cell) lie on a drawn
//!   fall point -- RR-4's "every placement on a FALL point" bar;
//! * the cell gradient and rainfall at drawn points (the turbidity inputs) and
//!   the mean turbidity, which `river_realism::TURBID_CLEAR_SLOPE` was read from;
//! * the §2.1 km conversion (cell size).
//!
//! **What it is a proxy for, honestly** (as `river_zoom_measurement.rs`): it
//! reads `River` entities (`river_entities` + `river_draw_plan`), not the
//! app's `DrawnRun`s (built inside the cdylib's `WorldGen`). Points are traced
//! cells; a run's discharge is its own running maximum (the app also pushes a
//! tributary's into its trunk, which can only raise a trunk). The RR laws are
//! the shipped ones: `src/river_realism.rs` is included by path.
//!
//! **What it must never do.** Change generated output or pin a value; its
//! assertions only guard that the fixture reached the code (non-empty tables).
//! Never un-ignore it: it generates several large worlds.
//!
//! ```text
//! cargo test --release -p cartalith-godot --test river_realism_measurement -- --ignored --nocapture --test-threads=1
//! ```
//! `RR_FAST=1` skips the owner world (2048x1311).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/river_realism.rs"]
mod river_realism;

use cartalith_engine::generate_terrain;
use cartalith_hydrology::{River, river_draw_plan, river_entities, river_flow_thresh, river_half_width_profile, river_width_scale_k};

/// The fit density of the probe's 1600 x 1000 window (950 x 608 px of map,
/// measured: 0.464 px a cell on 2048 x 1311, 0.928 on 1024 x 656), so the
/// pixel widths below are the frames' own.
const VIEW: (f64, f64) = (950.0, 608.0);
/// A width class must hold at least this share of drawn length to count, so a
/// handful of cells at an odd width is not a "class". **Labelled judgement.**
const CLASS_MIN_SHARE: f64 = 0.005;
const ZOOMS: [f64; 4] = [1.0, 4.0, 16.0, 64.0];

/// `(seed, gw, gh, km, density, label)`: the owner world and the two probe
/// worlds of Ruling ER's measurement (`DELTA_GEOMETRY_RESEARCH.md` §13).
const WORLDS: [(i32, usize, usize, f64, f64, &str); 3] = [
    (246_371, 2048, 1311, 40_075.0, 1.55, "owner"),
    (483_920, 1024, 656, 1_200.0, 1.0, "483920"),
    (912_345, 1024, 656, 1_200.0, 1.0, "912345"),
];

fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// Width classes: whole-pixel widths (floored at 1 px) holding at least
/// [`CLASS_MIN_SHARE`] of the drawn length, weighted by `len`.
fn classes(w_px: &[(f64, f64)]) -> (usize, f64) {
    let total: f64 = w_px.iter().map(|x| x.1).sum();
    let mut by: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    let mut widest: f64 = 0.0;
    for &(w, len) in w_px {
        let px = w.max(1.0);
        widest = widest.max(px);
        *by.entry(px.round() as i64).or_default() += len;
    }
    (by.values().filter(|&&l| l >= CLASS_MIN_SHARE * total).count(), widest)
}

#[test]
#[ignore]
fn river_realism_measurement() {
    let fast = std::env::var("RR_FAST").is_ok();
    for &(seed, gw, gh, km, density, label) in WORLDS.iter() {
        if fast && label == "owner" {
            continue;
        }
        let t0 = std::time::Instant::now();
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = km;
        p.river_density = density;
        p.use_gpu = false;
        let ws = generate_terrain(&p);
        let wrap = p.world;
        let n = gw * gh;
        let order = ws.stream_order.as_ref().expect("river extraction ran");
        let ch = ws.channels.as_ref().expect("channel tree retained");
        let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
        let route = cartalith_hydrology::routing_view(&ws.field, gw, gh, ws.sea_level, wrap, ws.integrated_drainage);
        let d8 = cartalith_hydrology::flow_receivers_with(&route, gw, gh, wrap, cartalith_engine::routing_roughness_of(ws.routing_roughness, gw, gh, wrap).as_ref());
        let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { d8[i] }).collect();
        let thresh = river_flow_thresh(gw, gh, gw, km);
        let wk = river_width_scale_k(km);
        let rivers: Vec<River> = river_entities(order, &ch.recv, &ws.flow_discharge, &ws.field, gw, gh, 1, thresh, wk, wrap);
        let plan = river_draw_plan(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
        let cell = |q: (f64, f64)| (q.1.max(0.0) as usize).min(gh - 1) * gw + (q.0.max(0.0) as usize).min(gw - 1);
        let fit = (VIEW.0 / gw as f64).min(VIEW.1 / gh as f64);
        let inputs = river_realism::NetworkInputs::new(gw, gh, ws.sea_level, p.peak_m, &ws.field, &ws.rainfall, &ws.flow_discharge, &ws, &p, km).expect("inputs");

        // Per drawn run: running-max discharge per point, step lengths.
        let drawn: Vec<usize> = (0..rivers.len()).filter(|&i| plan.parallel_of[i].is_none() && rivers[i].pts.len() >= 2).collect();
        let mut q_run: Vec<Vec<f32>> = Vec::new();
        let mut steps: Vec<Vec<f64>> = Vec::new();
        for &i in &drawn {
            let r = &rivers[i];
            let mut m = f32::NAN;
            q_run.push(
                r.pts
                    .iter()
                    .map(|&pt| {
                        let q = ws.flow_discharge[cell(pt)];
                        if q.is_finite() && !(m >= q) {
                            m = q;
                        }
                        m
                    })
                    .collect(),
            );
            steps.push((0..r.pts.len()).map(|k| if k == 0 { 0.5 } else { ((r.pts[k].0 - r.pts[k - 1].0).powi(2) + (r.pts[k].1 - r.pts[k - 1].1).powi(2)).sqrt() }).collect());
        }
        let q_max = river_realism::network_q_max(&q_run).expect("a discharge reading");
        let total: f64 = steps.iter().flatten().sum();
        assert!(total > 0.0 && !drawn.is_empty(), "{label}: no drawn network");
        println!("\n=== {label}: seed {seed}, {gw}x{gh}, {km} km ({:.2} km a cell), density {density}; drawn runs {}, drawn length {total:.0} cells, Q_max {q_max:.1} cell-units, fit {fit:.3} px/cell", km / gw as f64, drawn.len());

        // Q / Q_max over drawn length.
        let mut share: Vec<f64> = Vec::new();
        let mut weighted: Vec<(f64, f64)> = Vec::new();
        for (qr, st) in q_run.iter().zip(&steps) {
            for (q, l) in qr.iter().zip(st) {
                if q.is_finite() {
                    share.push(*q as f64 / q_max);
                    weighted.push((*q as f64 / q_max, *l));
                }
            }
        }
        let mut s2 = share.clone();
        println!("  Q/Q_max over drawn points: p10 {:.5} p50 {:.5} p90 {:.4} p99 {:.3} max {:.3}", pct(&mut s2, 0.1), pct(&mut s2, 0.5), pct(&mut s2, 0.9), pct(&mut s2, 0.99), pct(&mut s2, 1.0));
        let mut sq: Vec<f64> = share.iter().map(|s| s.sqrt()).collect();
        println!("  sqrt(Q/Q_max) (= width at W_MAX 1, cells): p10 {:.3} p50 {:.3} p90 {:.3} p99 {:.3}", pct(&mut sq, 0.1), pct(&mut sq, 0.5), pct(&mut sq, 0.9), pct(&mut sq, 0.99));

        // Width classes per zoom: old law, RR-1 at 1 and 2.
        let mut old_w: Vec<(f64, f64)> = Vec::new();
        for (j, &i) in drawn.iter().enumerate() {
            let r = &rivers[i];
            if let Some(pr) = river_half_width_profile(&r.pts, r.pts.len(), &ws.field, &ws.flow_discharge, order, gw, gh, wrap, thresh, wk) {
                for (k, hw) in pr.iter().enumerate() {
                    old_w.push((2.0 * hw, steps[j][k]));
                }
            }
        }
        for z in ZOOMS {
            let ppc = fit * z;
            let o = classes(&old_w.iter().map(|&(w, l)| (w * ppc, l)).collect::<Vec<_>>());
            let a: Vec<(f64, f64)> = weighted.iter().map(|&(s, l)| (river_realism::hydraulic_full_width(s, 1.0, 1.0).unwrap_or(0.0) * ppc, l)).collect();
            let b: Vec<(f64, f64)> = weighted.iter().map(|&(s, l)| (river_realism::hydraulic_full_width(s, 1.0, 2.0).unwrap_or(0.0) * ppc, l)).collect();
            let (ca, wa) = classes(&a);
            let (cb, wb) = classes(&b);
            println!(
                "  x{z:>2} ({ppc:.2} px/cell): width classes old {} (widest {:.1} px) | RR-1 W1 {ca} (widest {wa:.1} px, {:.1}x the floor) | W2 {cb} (widest {wb:.1} px)",
                o.0, o.1, wa
            );
        }

        // RR-4 and RR-2 per run.
        let (mut white_len, mut falls, mut knick, mut hang) = (0.0f64, 0usize, 0usize, 0usize);
        let mut grads: Vec<f64> = Vec::new();
        let mut cell_g: Vec<f64> = Vec::new();
        let mut rains: Vec<f64> = Vec::new();
        let mut turb: Vec<f64> = Vec::new();
        let mut fall_cells: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut drawn_cells: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mpu = p.peak_m / (1.0 - ws.sea_level);
        for (j, &i) in drawn.iter().enumerate() {
            let r = &rivers[i];
            let wet: Vec<bool> = r.pts.iter().map(|&pt| water[cell(pt)] != 0).collect();
            let rr = inputs.run(&r.pts, r.pts.len(), true, true, &wet);
            for k in 0..r.pts.len() {
                if rr.white[k] > 0.0 {
                    white_len += steps[j][k];
                }
                if rr.lip[k] > 0.0 {
                    fall_cells.insert(cell(r.pts[k]));
                }
                drawn_cells.insert(cell(r.pts[k]));
                let c = cell(r.pts[k]);
                if water[c] == 0 {
                    cell_g.push(inputs.grad_at(c) as f64);
                    rains.push(ws.rainfall[c] as f64);
                    if rr.turbidity[k].is_finite() {
                        turb.push(rr.turbidity[k] as f64);
                    }
                }
                if k + 1 < r.pts.len() {
                    let run_m = steps[j][k + 1] * km * 1000.0 / gw as f64;
                    let g = (ws.field[c] as f64 - ws.field[cell(r.pts[k + 1])] as f64) * mpu / run_m;
                    if g.is_finite() {
                        grads.push(g);
                    }
                }
            }
            falls += rr.counts.falls;
            knick += rr.counts.falls_knick;
            hang += rr.counts.falls_hanging;
        }
        let ng = grads.len().max(1) as f64;
        let over = |t: f64| grads.iter().filter(|&&g| g >= t).count() as f64 / ng * 100.0;
        let mut g2 = grads.clone();
        println!(
            "  along-course gradient: p50 {:.5} p90 {:.4} p99 {:.4} max {:.3}; steps >= 0.015 {:.2} %, >= 0.03 (rapids) {:.2} %, >= 0.065 (cascade) {:.2} %",
            pct(&mut g2, 0.5),
            pct(&mut g2, 0.9),
            pct(&mut g2, 0.99),
            pct(&mut g2, 1.0),
            over(0.015),
            over(0.03),
            over(0.065)
        );
        println!("  RR-4: white water {:.2} % of drawn length (bar 1-8 %); falls drawn {falls} (on a knickpoint {knick}, hanging valley {hang}); rock column {}", white_len / total * 100.0, inputs.has_rock());
        // The landmark detector's candidates (its constraint block at a land channel cell, to its channel receiver).
        let (mut cand, mut on_fall, mut on_drawn) = (0usize, 0usize, 0usize);
        for c in 0..n {
            // `pool_waterfall`'s domain: a channel cell the water mask calls land.
            if ch.chan[c] == 0 || water[c] != 0 {
                continue;
            }
            let rv = ch.recv[c];
            let drop_m = if rv >= 0 { (ws.field[c] as f64 - ws.field[rv as usize] as f64) * mpu } else { 0.0 };
            if cartalith_civ::landmark::waterfall_constraints_hold(drop_m, inputs.grad_at(c) as f64, ws.flow_discharge[c] as f64, thresh) {
                cand += 1;
                on_fall += usize::from(fall_cells.contains(&c));
                on_drawn += usize::from(drawn_cells.contains(&c));
            }
        }
        println!("  waterfall candidates (landmark constraints) {cand}, on a drawn run {on_drawn}, on a drawn fall {on_fall}");
        assert_eq!(on_fall, on_drawn, "{label}: every waterfall candidate on a drawn run must be a drawn fall (one predicate)");
        let (mut cg, mut rn, mut tb) = (cell_g.clone(), rains.clone(), turb.clone());
        println!(
            "  turbidity inputs at drawn land points: cell gradient p25 {:.4} p50 {:.4} p75 {:.4} p90 {:.4}; rain p10 {:.3} p50 {:.3} p90 {:.3}; turbidity p10 {:.2} p50 {:.2} p90 {:.2}",
            pct(&mut cg, 0.25),
            pct(&mut cg, 0.5),
            pct(&mut cg, 0.75),
            pct(&mut cg.clone(), 0.9),
            pct(&mut rn, 0.1),
            pct(&mut rn, 0.5),
            pct(&mut rn, 0.9),
            pct(&mut tb, 0.1),
            pct(&mut tb, 0.5),
            pct(&mut tb, 0.9)
        );
        assert!(!cell_g.is_empty() && !grads.is_empty(), "{label}: the tables must not be empty");
        println!("  ({:.1} s)", t0.elapsed().as_secs_f64());
    }
}
