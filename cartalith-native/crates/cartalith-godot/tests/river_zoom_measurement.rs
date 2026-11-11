//! **RZ-0: the three visibility keys, measured before anything is pinned**
//! (`RIVER_ZOOM_VISIBILITY_RESEARCH.md` §4 RZ-0; owner Ruling EN,
//! 2026-11-09). Compares, on the same drawn network, three ways of ranking a
//! river segment for "which part of the network is visible at a given share
//! `f` of the drawn length": **discharge** (running maximum downstream, the
//! proposed `u`), **Strahler order**, and **width** (`river_half_width_profile`,
//! the symbolic `channel_disc` law). Every key becomes a length-weighted rank
//! `u` (the share of drawn length whose key is at least this segment's, ties
//! sharing one rank), so the three are compared at EQUAL visible length.
//!
//! **What it is a proxy for, honestly.** Like `river_gate_measurement.rs` it
//! reads `River` entities (`river_entities` + `river_draw_plan`), not the app's
//! `DrawnRun`s (built inside the cdylib's `WorldGen`). Points are traced cells,
//! so a segment is one cell step; the app's smoothed curve, shore extension and
//! bridges are not counted. Connectivity is over cells: a tributary's last
//! point is its trunk's junction cell, so the graph joins there.
//!
//! **What it must never do.** Change generated output or pin a value; its
//! assertions only guard that the fixture reached the code (non-empty tables,
//! shore mouths exist). Never un-ignore it: it generates dozens of worlds.
//!
//! ```text
//! RZ_OWNER=1 cargo test --release -p cartalith-godot --test river_zoom_measurement -- --ignored --nocapture --test-threads=1
//! ```
//! `RZ_SEEDS` (comma list) overrides the twelve seeds; `RZ_OWNER=1` adds the
//! owner's world (seed 246371, 2048x1311, 40 075 km, density 1.55).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use std::collections::HashMap;
use std::time::Instant;

use cartalith_engine::{WorldState, generate_terrain};
use cartalith_hydrology::{River, river_draw_plan, river_entities, river_flow_thresh, river_half_width_profile, river_width_scale_k};

/// Twelve seeds, the same as `river_gate_measurement.rs` (chosen before any
/// result was seen there).
const SEEDS: [i32; 12] = [246_371, 483_920, 912_345, 24_601, 404, 505, 808, 707, 71_077_345, 1_001, 31_337, 123_456];
/// Same three shapes `(gw, gh, km)` as the gate harness.
const SHAPES: [(usize, usize, f64); 3] = [(512, 328, 40_075.0), (384, 246, 800.0), (1024, 656, 40_075.0)];
/// The shares `f` the research asks for, plus 0.5 (Ruling EN's opening view).
/// A segment counts "visible" when its rank `u <= f`.
const FS: [f64; 5] = [0.1, 0.24, 0.5, 0.96, 1.0];
const KEYS: [&str; 4] = ["discharge", "order", "width", "disch-net"];

fn env_list<T: std::str::FromStr + Copy>(key: &str, default: &[T]) -> Vec<T> {
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse {s:?}"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// One segment of the drawn network.
struct Seg {
    /// Cell indices of its two ends (graph nodes).
    a: usize,
    b: usize,
    len: f64,
    /// Keys in the order of [`KEYS`].
    key: [f64; 4],
    /// Which drawn run it belongs to.
    run: usize,
}

struct Net {
    segs: Vec<Seg>,
    /// Per drawn run: its running-max discharge, whether its mouth is on a shore, its last segment index.
    runs: Vec<(f64, bool, usize)>,
    total: f64,
}

fn shore(c: usize, gw: usize, gh: usize, fld: &[f32], sea: f64, water: &[u8]) -> bool {
    let (x, y) = ((c % gw) as i64, (c / gw) as i64);
    (-1..=1).any(|dy| {
        (-1..=1).any(|dx| {
            let (nx, ny) = (x + dx, y + dy);
            nx >= 0 && ny >= 0 && (nx as usize) < gw && (ny as usize) < gh && {
                let q = ny as usize * gw + nx as usize;
                fld[q] as f64 <= sea || water[q] != 0
            }
        })
    })
}

fn build(ws: &WorldState, wrap: bool, gw: usize, gh: usize, km: f64) -> Net {
    let n = gw * gh;
    let order = ws.stream_order.as_ref().expect("river extraction ran");
    let ch = ws.channels.as_ref().expect("channel tree retained");
    let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
    let route = cartalith_hydrology::routing_view(&ws.field, gw, gh, ws.sea_level, wrap, ws.integrated_drainage);
    let d8 = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
    let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { d8[i] }).collect();
    let thresh = river_flow_thresh(gw, gh, gw, km);
    let wk = river_width_scale_k(km);
    let rivers: Vec<River> = river_entities(order, &ch.recv, &ws.flow_discharge, &ws.field, gw, gh, 1, thresh, wk, wrap);
    let plan = river_draw_plan(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
    let cell = |p: (f64, f64)| (p.1.max(0.0) as usize).min(gh - 1) * gw + (p.0.max(0.0) as usize).min(gw - 1);
    let mut net = Net { segs: Vec::new(), runs: Vec::new(), total: 0.0 };
    for (i, r) in rivers.iter().enumerate() {
        if plan.parallel_of[i].is_some() || r.pts.len() < 2 {
            continue;
        }
        let prof = river_half_width_profile(&r.pts, r.pts.len(), &ws.field, &ws.flow_discharge, order, gw, gh, wrap, thresh, wk);
        // Running maximum of discharge from the head; a non-finite reading takes the previous one, and a run
        // with no reading at all gets +inf (always visible: hiding a river for lack of data is the worse failure).
        let mut run_q: Option<f64> = None;
        let run_idx = net.runs.len();
        let mut last_seg = 0usize;
        for k in 0..r.pts.len() {
            let c = cell(r.pts[k]);
            let q = ws.flow_discharge[c] as f64;
            if q.is_finite() {
                run_q = Some(run_q.map_or(q, |m| m.max(q)));
            }
            if k == 0 {
                continue;
            }
            let c0 = cell(r.pts[k - 1]);
            let (dx, dy) = (r.pts[k].0 - r.pts[k - 1].0, r.pts[k].1 - r.pts[k - 1].1);
            let len = (dx * dx + dy * dy).sqrt();
            let o = order[c].max(order[c0]) as f64;
            let w = prof.as_ref().and_then(|p| p.get(k).copied()).unwrap_or(0.0);
            last_seg = net.segs.len();
            net.segs.push(Seg { a: c0, b: c, len, key: [run_q.unwrap_or(f64::INFINITY), o, w, 0.0], run: run_idx });
            net.total += len;
        }
        let mouth = cell(*r.pts.last().unwrap());
        net.runs.push((run_q.unwrap_or(f64::INFINITY), shore(mouth, gw, gh, &ws.field, ws.sea_level, &water), last_seg));
    }
    // Key 3, "disch-net": the per-run running maximum pushed DOWNSTREAM THROUGH JUNCTIONS (Kahn order over the
    // cell graph a -> b), so a trunk is never ranked below a tributary that joins it. The per-run key alone lets a
    // visible tributary meet a hidden trunk (discharge is accumulated on a different tree than the traced one).
    let mut eff: HashMap<usize, f64> = HashMap::new();
    let mut indeg: HashMap<usize, usize> = HashMap::new();
    let mut out: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, s) in net.segs.iter().enumerate() {
        out.entry(s.a).or_default().push(i);
        *indeg.entry(s.b).or_default() += 1;
        indeg.entry(s.a).or_default();
        let e = eff.entry(s.b).or_insert(f64::NEG_INFINITY);
        *e = e.max(s.key[0]);
        eff.entry(s.a).or_insert(f64::NEG_INFINITY);
    }
    let mut stack: Vec<usize> = indeg.iter().filter(|&(_, &d)| d == 0).map(|(&c, _)| c).collect();
    while let Some(c) = stack.pop() {
        let ec = eff[&c];
        for &si in out.get(&c).map(|v| v.as_slice()).unwrap_or(&[]) {
            let b = net.segs[si].b;
            let e = eff.get_mut(&b).unwrap();
            *e = e.max(ec);
            let d = indeg.get_mut(&b).unwrap();
            *d -= 1;
            if *d == 0 {
                stack.push(b);
            }
        }
    }
    for s in net.segs.iter_mut() {
        s.key[3] = eff[&s.b].max(s.key[0]);
    }
    net
}

/// Length-weighted rank per segment for key `k`: the share of drawn length
/// whose key is `>=` this segment's (ties share a rank), in `(0, 1]`.
fn ranks(net: &Net, k: usize) -> Vec<f64> {
    let mut ix: Vec<usize> = (0..net.segs.len()).collect();
    ix.sort_by(|&a, &b| net.segs[b].key[k].total_cmp(&net.segs[a].key[k]));
    let mut u = vec![0.0; net.segs.len()];
    let (mut cum, mut i) = (0.0, 0);
    while i < ix.len() {
        let mut j = i;
        let mut grp = 0.0;
        while j < ix.len() && net.segs[ix[j]].key[k] == net.segs[ix[i]].key[k] {
            grp += net.segs[ix[j]].len;
            j += 1;
        }
        cum += grp;
        for &s in &ix[i..j] {
            u[s] = cum / net.total;
        }
        i = j;
    }
    u
}

fn root(p: &mut [usize], mut x: usize) -> usize {
    while p[x] != x {
        p[x] = p[p[x]];
        x = p[x];
    }
    x
}

/// Connected pieces of the visible segments, joined at shared cells.
fn components(net: &Net, vis: &[bool]) -> usize {
    let mut id: HashMap<usize, usize> = HashMap::new();
    let mut parent: Vec<usize> = Vec::new();
    for (i, s) in net.segs.iter().enumerate() {
        if !vis[i] {
            continue;
        }
        let mut ends = [0usize; 2];
        for (e, c) in [s.a, s.b].into_iter().enumerate() {
            ends[e] = *id.entry(c).or_insert_with(|| {
                parent.push(parent.len());
                parent.len() - 1
            });
        }
        let (ra, rb) = (root(&mut parent, ends[0]), root(&mut parent, ends[1]));
        parent[ra] = rb;
    }
    (0..parent.len()).filter(|&i| root(&mut parent, i) == i).count()
}

fn med(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.total_cmp(b));
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

#[derive(Default)]
struct Pool {
    n: Vec<f64>,
    total: Vec<f64>,
    o1_share: Vec<f64>,
    o1_top1: Vec<f64>,
    o1_top10: Vec<f64>,
    sort_ms: Vec<f64>,
    mono_breaks: Vec<f64>,
    /// `[f][key]`.
    comps: Vec<Vec<Vec<f64>>>,
    lost: Vec<Vec<Vec<f64>>>,
}

#[test]
#[ignore = "RZ-0 measurement: dozens of generated worlds; run alone in release (see the module doc)"]
fn river_zoom_measurement() {
    let seeds = env_list::<i32>("RZ_SEEDS", &SEEDS);
    let owner = std::env::var("RZ_OWNER").is_ok_and(|v| v == "1");
    let mut shapes: Vec<(usize, usize, f64, f64, Vec<i32>)> =
        SHAPES.iter().map(|&(w, h, k)| (w, h, k, params::defaults().river_density, seeds.clone())).collect();
    if owner {
        shapes.push((2048, 1311, 40_075.0, 1.55, vec![246_371]));
    }
    let t_all = Instant::now();
    let mut worlds = 0usize;
    for (gw, gh, km, density, seeds) in shapes {
        println!("\n=== {gw}x{gh}, {km} km, density {density}, {} seeds ===", seeds.len());
        let mut p = Pool { comps: vec![vec![vec![]; 4]; FS.len()], lost: vec![vec![vec![]; 4]; FS.len()], ..Default::default() };
        for &seed in &seeds {
            let mut pr = params::defaults();
            pr.gw = gw;
            pr.gh = gh;
            pr.tect.seed = seed;
            pr.map_width_km = km;
            pr.use_gpu = false;
            pr.river_density = density;
            let ws = generate_terrain(&pr);
            let net = build(&ws, pr.world, gw, gh, km);
            assert!(!net.segs.is_empty() && net.total > 0.0, "seed {seed}: no drawn segments -- the fixture measured nothing");
            worlds += 1;
            p.n.push(net.segs.len() as f64);
            p.total.push(net.total);
            let t = Instant::now();
            let u: Vec<Vec<f64>> = (0..4).map(|k| ranks(&net, k)).collect();
            p.sort_ms.push(t.elapsed().as_secs_f64() * 1e3 / 3.0);
            let o1: f64 = net.segs.iter().filter(|s| s.key[1] <= 1.0).map(|s| s.len).sum();
            p.o1_share.push(o1 / net.total);
            for (share, out) in [(0.01, &mut p.o1_top1), (0.10, &mut p.o1_top10)] {
                let (mut tot, mut o1l) = (0.0, 0.0);
                for (i, s) in net.segs.iter().enumerate() {
                    if u[0][i] <= share {
                        tot += s.len;
                        if s.key[1] <= 1.0 {
                            o1l += s.len;
                        }
                    }
                }
                out.push(if tot > 0.0 { o1l / tot } else { 0.0 });
            }
            // The discharge key is a running maximum, so it must never fall along a run.
            let mut breaks = 0usize;
            let mut prev: HashMap<usize, f64> = HashMap::new();
            for s in &net.segs {
                let e = prev.entry(s.run).or_insert(f64::NEG_INFINITY);
                if s.key[0] < *e {
                    breaks += 1;
                }
                *e = s.key[0];
            }
            p.mono_breaks.push(breaks as f64);
            // Top-decile shore outlets by discharge.
            let mut mouths: Vec<usize> = (0..net.runs.len()).filter(|&i| net.runs[i].1 && net.runs[i].0.is_finite()).collect();
            mouths.sort_by(|&a, &b| net.runs[b].0.total_cmp(&net.runs[a].0));
            assert!(!mouths.is_empty(), "seed {seed}: no shore mouths -- the fixture measured nothing");
            let top = (mouths.len() / 10).max(1);
            for (fi, &f) in FS.iter().enumerate() {
                for k in 0..4 {
                    let vis: Vec<bool> = u[k].iter().map(|&x| x <= f + 1e-9).collect();
                    p.comps[fi][k].push(components(&net, &vis) as f64);
                    let lost = mouths[..top].iter().filter(|&&m| !vis[net.runs[m].2]).count();
                    p.lost[fi][k].push(lost as f64 / top as f64);
                }
            }
        }
        let rows: Vec<(String, Vec<f64>)> = {
            let mut r = vec![
                ("drawn segments n".to_string(), p.n.clone()),
                ("drawn length (cells)".to_string(), p.total.clone()),
                ("order-1 share of length".to_string(), p.o1_share.clone()),
                ("order-1 share of top-1% Q length".to_string(), p.o1_top1.clone()),
                ("order-1 share of top-10% Q length".to_string(), p.o1_top10.clone()),
                ("rank sort per key (ms)".to_string(), p.sort_ms.clone()),
                ("Q running-max breaks (must be 0)".to_string(), p.mono_breaks.clone()),
            ];
            for (fi, &f) in FS.iter().enumerate() {
                for k in 0..4 {
                    r.push((format!("f={f:<4} {:<9} components", KEYS[k]), p.comps[fi][k].clone()));
                    r.push((format!("f={f:<4} {:<9} top-decile outlets lost", KEYS[k]), p.lost[fi][k].clone()));
                }
            }
            r
        };
        for (name, mut v) in rows {
            let (m, lo, hi) = med(&mut v);
            println!("  {name:<40} median {m:>10.3}  ({lo:.3}..{hi:.3})");
        }
    }
    println!("\n{worlds} worlds in {:.0}s", t_all.elapsed().as_secs_f64());
    assert!(worlds > 0);
}
