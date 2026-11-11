//! **Ruling ER: which flow routing breaks the straight river runs?**
//! (`DELTA_GEOMETRY_RESEARCH.md` section 13; the owner, 2026-11-11: *"3, and
//! would it change if we had 32 instead of 8 directions?"*).
//!
//! Section 11 measured why rivers are straight: single-direction D8 routing
//! over planar slopes gives runs of one repeated grid step, 32-58 % of every
//! network's length. This harness measures the candidates on the SAME ground:
//!
//! - `d8`: the shipped steepest-descent receiver (the baseline);
//! - `tie`: D8 with a seeded break among near-ties (drops within
//!   [`TIE_FRAC`] of the steepest);
//! - `d16`, `d32`: steepest descent over 16 and 32 neighbours (the knight
//!   moves, then the (1,3)/(2,3) moves): angle steps of about 27 and 11
//!   degrees instead of 45;
//! - `ltd`: a D-infinity direction (the aspect of the routing surface) made
//!   single-receiver by Orlandini et al. (2003)'s least transverse deviation
//!   (D8-LTD): of the two D8 neighbours either side of the aspect, the one
//!   that keeps the path's accumulated sideways error smallest;
//! - `rough(k, L)`: `cartalith_hydrology::rough`, D8 among strictly lower
//!   neighbours with a seeded value-noise perturbation of strength `k` (a
//!   fraction of the local slope) and lattice spacing `L` cells.
//!
//! **The ground.** Each world is generated with the app's parameters and
//! `carve_rivers` OFF, so the field is the eroded terrain before any river is
//! carved into it (a carved field would hold the shipped D8 channels as
//! trenches and bias every candidate toward them). Each candidate routes over
//! that field's depression-filled routing surface, accumulates the world's own
//! rainfall, channelises at the app's own threshold (`river_flow_thresh /
//! density`, the integrated path's rule), orders by Strahler and traces
//! (`trace_river_polylines`). Straightness is `river_straight::straight_stats`
//! (min length 8 cells, tolerance 0.25 cells, as section 11) on the traced
//! path (densified to one point a cell, so a 16/32-neighbour jump is not one
//! long straight segment) and on the drawn curve's proxy: the app's
//! `river_render_polyline` steps (`civ_rdp_simplify` at 0.5 cells, then
//! `civ_catmull_rom_sample` at 0.25), without the shore and junction pins.
//!
//! **Plausibility.** `uphill`: channel steps whose receiver is higher on the
//! real field, between two cells the depression fill did not raise (inside a
//! filled lake the routing surface, not the bed, is what water follows);
//! `uphill_route`: on the routing surface (must be 0); `dry_ends`:
//! traced rivers whose last cell is on land, not beside drawn water (sea or a
//! lake) and not on the map's edge -- broken drainage; `km`: total traced
//! length in cells; `rivers`: traced polylines. Lakes are a function of the
//! field, which no candidate changes here; the full pipeline's lake count is
//! checked for the built candidate by `generation_with_rough_routing`.
//!
//! **Must never** pin a value or change generated output: it only reports.
//! Never un-ignore it: it generates several large worlds.
//!
//! ```text
//! cargo test --release -p cartalith-godot --test river_routing_measurement -- --ignored --nocapture --test-threads=1
//! ```
//! `RR_WORLDS=owner,483920,912345` (default all three) picks the worlds.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/river_straight.rs"]
mod river_straight;

use std::time::Instant;

use cartalith_engine::generate_terrain;
use cartalith_hydrology::{Roughness, river_flow_thresh, routing_view, strahler_from_receivers, trace_river_polylines};

/// Near-tie fraction for `tie`: a lower neighbour whose drop per distance is
/// at least `1 - TIE_FRAC` of the steepest is a tie. **Labelled judgement**:
/// 5 %, larger than any rounding and small enough to stay "steepest".
const TIE_FRAC: f64 = 0.05;
/// The straightness metric's minimum stretch and tolerance, cells: section 11's.
const MIN_LEN: f64 = 8.0;
/// See [`MIN_LEN`].
const TOL: f64 = 0.25;

/// One routing candidate.
#[derive(Clone, Copy, Debug)]
enum Cand {
    D8,
    Tie,
    D16,
    D32,
    Ltd,
    Rough(f64, f64),
}

impl Cand {
    fn name(self) -> String {
        match self {
            Cand::D8 => "d8".into(),
            Cand::Tie => "tie".into(),
            Cand::D16 => "d16".into(),
            Cand::D32 => "d32".into(),
            Cand::Ltd => "ltd".into(),
            Cand::Rough(k, l) => format!("rough(k={k},L={l})"),
        }
    }
}

/// The neighbour offsets of an `n`-direction scheme (8, 16 or 32).
fn offsets(n: usize) -> Vec<(i64, i64)> {
    let mut v: Vec<(i64, i64)> = Vec::new();
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx != 0 || dy != 0 {
                v.push((dx, dy));
            }
        }
    }
    let mut add = |a: i64, b: i64| {
        for (sx, sy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            v.push((a * sx, b * sy));
            v.push((b * sx, a * sy));
        }
    };
    if n >= 16 {
        add(1, 2);
    }
    if n >= 32 {
        add(1, 3);
        add(2, 3);
    }
    v
}

/// SplitMix64's finaliser, for the tie break's per-cell choice.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Cell index of `(x, y)` with `x` wrapped on a world map, `None` off the grid.
fn at(x: i64, y: i64, gw: usize, gh: usize, wrap: bool) -> Option<usize> {
    if y < 0 || y >= gh as i64 {
        return None;
    }
    let x = if wrap {
        x.rem_euclid(gw as i64)
    } else if x < 0 || x >= gw as i64 {
        return None;
    } else {
        x
    };
    Some(y as usize * gw + x as usize)
}

/// Routes `route` by candidate `c`: `(accumulated flow, receiver per cell)`.
/// Every candidate picks only strictly lower cells, so cells are accumulated
/// in descending height order (ties by ascending index, `compute_flow`'s).
fn route_by(c: Cand, route: &[f32], rain: &[f32], gw: usize, gh: usize, wrap: bool, seed: u64) -> (Vec<f32>, Vec<i32>) {
    let n = gw * gh;
    if let Cand::D8 | Cand::Rough(..) = c {
        let r = match c {
            Cand::Rough(k, l) => Some(Roughness::with_params(gw, gh, seed, wrap, k, l)),
            _ => None,
        };
        let acc = cartalith_hydrology::compute_flow_with(gw, gh, route, Some(rain), true, wrap, r.as_ref());
        let recv = cartalith_hydrology::flow_receivers_with(route, gw, gh, wrap, r.as_ref());
        return (acc, recv);
    }
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_by(|&a, &b| route[b as usize].total_cmp(&route[a as usize]).then(a.cmp(&b)));
    // Rain seeding as compute_flow's: floor 0.05, mean-normalised.
    let sm: f64 = rain.iter().map(|&r| (r as f64).max(0.05)).sum();
    let kq = n as f64 / sm.max(1e-6);
    let mut acc: Vec<f32> = rain.iter().map(|&r| ((r as f64).max(0.05) * kq) as f32).collect();
    let mut recv = vec![-1i32; n];
    let nb8 = offsets(8);
    let nbs = match c {
        Cand::D16 => offsets(16),
        Cand::D32 => offsets(32),
        _ => nb8.clone(),
    };
    // D8-LTD state: each cell's accumulated sideways deviation (cells) and the
    // discharge of the donor it was inherited from.
    let mut dev = vec![0f64; n];
    let mut dev_q = vec![-1f32; n];
    for &i in &order {
        let i = i as usize;
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        let h = route[i] as f64;
        let steepest = |set: &[(i64, i64)]| -> (i64, f64) {
            let mut best = (-1i64, 0.0f64);
            for &(dx, dy) in set {
                if let Some(j) = at(x + dx, y + dy, gw, gh, wrap) {
                    let d = ((dx * dx + dy * dy) as f64).sqrt();
                    let s = (h - route[j] as f64) / d;
                    if s > best.1 {
                        best = (j as i64, s);
                    }
                }
            }
            best
        };
        let r: i64 = match c {
            Cand::D16 | Cand::D32 => steepest(&nbs).0,
            Cand::Tie => {
                let (b, s) = steepest(&nb8);
                if b < 0 {
                    -1
                } else {
                    let ties: Vec<usize> = nb8
                        .iter()
                        .filter_map(|&(dx, dy)| at(x + dx, y + dy, gw, gh, wrap).map(|j| (j, ((dx * dx + dy * dy) as f64).sqrt())))
                        .filter(|&(j, d)| (h - route[j] as f64) / d >= (1.0 - TIE_FRAC) * s && (route[j] as f64) < h)
                        .map(|(j, _)| j)
                        .collect();
                    ties[(mix(seed ^ i as u64) % ties.len() as u64) as usize] as i64
                }
            }
            Cand::Ltd => {
                // The aspect of the routing surface (central differences);
                // on a flat (the fill's ULP tilt) it has no direction and
                // the cell takes plain D8.
                let hx = |dx: i64| at(x + dx, y, gw, gh, wrap).map_or(h, |j| route[j] as f64);
                let hy = |dy: i64| at(x, y + dy, gw, gh, wrap).map_or(h, |j| route[j] as f64);
                let (gx, gy) = ((hx(1) - hx(-1)) * 0.5, (hy(1) - hy(-1)) * 0.5);
                let (b8, _) = steepest(&nb8);
                if b8 < 0 || (gx == 0.0 && gy == 0.0) {
                    b8
                } else {
                    let a = (-gy).atan2(-gx); // downhill direction, radians, grid coords
                    let k0 = (a / std::f64::consts::FRAC_PI_4).floor() as i64;
                    let mut best = (b8, f64::INFINITY, 0.0f64);
                    for k in [k0, k0 + 1] {
                        let b = k as f64 * std::f64::consts::FRAC_PI_4;
                        let (dx, dy) = (b.cos().round() as i64, b.sin().round() as i64);
                        if let Some(j) = at(x + dx, y + dy, gw, gh, wrap) {
                            if (route[j] as f64) < h {
                                let d = ((dx * dx + dy * dy) as f64).sqrt();
                                let side = dev[i] + d * (b - a).sin();
                                if side.abs() < best.1 {
                                    best = (j as i64, side.abs(), side);
                                }
                            }
                        }
                    }
                    if best.1.is_finite() {
                        let j = best.0 as usize;
                        if acc[i] > dev_q[j] {
                            dev_q[j] = acc[i];
                            dev[j] = best.2;
                        }
                    }
                    best.0
                }
            }
            Cand::D8 | Cand::Rough(..) => unreachable!(),
        };
        if r >= 0 {
            let j = r as usize;
            assert!(route[j] < route[i], "{c:?} picked a cell not strictly lower");
            acc[j] = (acc[j] as f64 + acc[i] as f64) as f32;
            recv[i] = r as i32;
        }
    }
    (acc, recv)
}

/// The traced path with one point a cell (a 16/32-neighbour jump is
/// filled in linearly, so it is measured as the line it draws).
fn densify(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out = vec![p[0]];
    for w in p.windows(2) {
        let d = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
        let k = d.ceil().max(1.0) as usize;
        for t in 1..=k {
            let f = t as f64 / k as f64;
            out.push((w[0].0 + (w[1].0 - w[0].0) * f, w[0].1 + (w[1].1 - w[0].1) * f));
        }
    }
    out
}

/// One candidate's row on one world.
struct Row {
    traced: river_straight::StraightStats,
    drawn: river_straight::StraightStats,
    uphill: usize,
    uphill_route: usize,
    dry_ends: usize,
    rivers: usize,
    ms: f64,
}

#[test]
#[ignore = "Ruling ER measurement: generates large worlds; run alone in release (see the module doc)"]
fn river_routing_measurement() {
    let pick = std::env::var("RR_WORLDS").unwrap_or_else(|_| "owner,483920,912345".into());
    // (name, seed, gw, gh, km, density): the owner's world as section 11 and
    // the delta work measured it; the two seeds at section 11's 1024 x 656.
    let worlds: Vec<(&str, i32, usize, usize, f64, f64)> = [
        ("owner", 246_371, 2048, 1311, 40_075.0, 1.55),
        ("483920", 483_920, 1024, 656, 40_075.0, params::defaults().river_density),
        ("912345", 912_345, 1024, 656, 40_075.0, params::defaults().river_density),
    ]
    .into_iter()
    .filter(|w| pick.split(',').any(|p| p.trim() == w.0))
    .collect();
    let cands = [
        Cand::D8,
        Cand::Tie,
        Cand::D16,
        Cand::D32,
        Cand::Ltd,
        Cand::Rough(1.0, 8.0),
        Cand::Rough(4.0, 8.0),
        Cand::Rough(8.0, 8.0),
        Cand::Rough(2.0, 4.0),
        Cand::Rough(3.0, 4.0),
        Cand::Rough(4.0, 4.0),
        Cand::Rough(4.0, 6.0),
        Cand::Rough(4.0, 16.0),
    ];
    println!("| world | candidate | traced straight | axis share | longest | drawn-curve straight | rivers | length | uphill (field) | uphill (route) | dry ends | route ms |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    for (name, seed, gw, gh, km, density) in worlds {
        let mut pr = params::defaults();
        pr.gw = gw;
        pr.gh = gh;
        pr.tect.seed = seed;
        pr.map_width_km = km;
        pr.use_gpu = false;
        pr.river_density = density;
        pr.carve_rivers = false;
        let ws = generate_terrain(&pr);
        let wrap = pr.world;
        let sea = ws.sea_level;
        let field: &[f32] = &ws.field;
        let route = routing_view(field, gw, gh, sea, wrap, true).into_owned();
        let water = cartalith_civ::build_water_bodies(field, gw, gh, sea, wrap, Some(&ws.rainfall)).classification;
        let thresh = river_flow_thresh(gw, gh, gw, km) / density;
        for c in cands {
            let t0 = Instant::now();
            let (acc, recv) = route_by(c, &route, &ws.rainfall, gw, gh, wrap, seed as u64);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let chan: Vec<u8> = (0..gw * gh).map(|i| ((field[i] as f64) >= sea && acc[i] as f64 > thresh) as u8).collect();
            let order = strahler_from_receivers(&recv, &acc, &chan);
            let polys = trace_river_polylines(&order, &recv, gw, gh, 1);
            assert!(!polys.is_empty(), "{name} {c:?}: no rivers traced -- the fixture measured nothing");
            let dense: Vec<Vec<(f64, f64)>> = polys.iter().map(|p| densify(p)).collect();
            let drawn: Vec<Vec<(f64, f64)>> = polys
                .iter()
                .map(|p| cartalith_civ::civ_catmull_rom_sample(&cartalith_civ::civ_rdp_simplify(p, 0.5), 0.25))
                .collect();
            let (mut uphill, mut uphill_route) = (0usize, 0usize);
            for i in 0..gw * gh {
                if chan[i] != 0 && recv[i] >= 0 {
                    let j = recv[i] as usize;
                    // On ground the fill did not raise (both cells): a step up there is real uphill flow.
                    uphill += (route[i] == field[i] && route[j] == field[j] && field[j] > field[i]) as usize;
                    uphill_route += (route[j] >= route[i]) as usize;
                }
            }
            let cell = |p: (f64, f64)| (p.1 as usize).min(gh - 1) * gw + (p.0 as usize).min(gw - 1);
            let dry_ends = polys
                .iter()
                .filter(|p| {
                    let e = cell(*p.last().unwrap());
                    let (x, y) = ((e % gw) as i64, (e / gw) as i64);
                    let edge = y == 0 || y + 1 == gh as i64 || (!wrap && (x == 0 || x + 1 == gw as i64));
                    let near_water = (-1..=1).any(|dy| {
                        (-1..=1).any(|dx| at(x + dx, y + dy, gw, gh, wrap).is_some_and(|q| (field[q] as f64) < sea || water[q] != 0))
                    });
                    // A trace also ends where it meets a river already traced (a confluence): not dry.
                    let joins = recv[e] >= 0 && order[recv[e] as usize] >= 1;
                    !edge && !near_water && !joins
                })
                .count();
            let row = Row {
                traced: river_straight::straight_stats(dense.iter().map(|v| v.as_slice()), MIN_LEN, TOL),
                drawn: river_straight::straight_stats(drawn.iter().map(|v| v.as_slice()), MIN_LEN, TOL),
                uphill,
                uphill_route,
                dry_ends,
                rivers: polys.len(),
                ms,
            };
            let pc = |s: &river_straight::StraightStats| 100.0 * s.straight / s.total.max(1e-9);
            println!(
                "| {name} | {} | {:.1} % | {:.0} % | {:.0} | {:.1} % | {} | {:.0} | {} | {} | {} | {:.0} |",
                c.name(),
                pc(&row.traced),
                100.0 * row.traced.axis / row.traced.straight.max(1e-9),
                row.traced.longest,
                pc(&row.drawn),
                row.rivers,
                row.traced.total,
                row.uphill,
                row.uphill_route,
                row.dry_ends,
                row.ms
            );
        }
    }
}
