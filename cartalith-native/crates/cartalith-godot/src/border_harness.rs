//! Owner Ruling GO's measurement (`BORDER_FORMATION_RESEARCH.md` §1.4 and §4
//! R0): the research's scratch straightness and terrain scripts, turned into an
//! ignored test over the real pipeline that **pins nothing**. It prints, per
//! seed, grid width and `civ.border_friction` strength:
//!
//! - **linear share**: of every third border pixel (a land cell 4-adjacent to
//!   a cell of another owning faction), the fraction whose radius-12 neighbourhood
//!   of border pixels fits a line with RMS residual under 0.8 px. The research's
//!   own measure and constants (`analyze.py`'s `straight_share`, radius 12,
//!   tolerance 0.8, at least 12 neighbours), chosen once there and not tuned here.
//! - **ridge share**: border cells above the mean of their local window, against
//!   the same share over owned land (the research's `tpi > 0`, window radius 12
//!   cells at 1024 wide, scaled with the grid);
//! - **river share**: border cells within `4 * gw / 1024` cells of a river of
//!   order >= 3, against owned land (the research's "within one stride-4 sample");
//! - **unowned land**, **roads** (the generated ways, sea routes and road edges
//!   hashed, which must not move with the strength) and the largest per-faction
//!   area ratio against strength 0;
//! - the territory sweep's time at each strength (one run each, not a benchmark).
//!
//! Worlds: the research's three seeds, 4 000 km wide, the app's parameters
//! (`params::defaults()`) with only `civ.border_friction` varied. Set
//! `BORDER_HARNESS_OUT` to a directory to also write a quick composite PNG per
//! world (grey relief, the wash, rivers in blue, the border line in black):
//! scratch for looking, not the shell's rendering.
//!
//! One small test beside it does run with the suite,
//! `border_friction_moves_territory_not_roads` (see its doc).
//!
//! Run (alone): `cargo test -p cartalith-godot --lib border_harness -- --ignored --nocapture`
//! (`BORDER_GW=384,2048` and `BORDER_S=0,0.6` choose the grids and strengths).

use crate::fh_r2_baseline::Fnv;

/// The research's seeds (§1.4).
const SEEDS: [i32; 3] = [483920, 912345, 246371];

/// The research's map width.
const WIDTH_KM: f64 = 4000.0;

fn generate(seed: i32, gw: usize, strength: f64) -> (cartalith_engine::WorldState, crate::CivData, usize, f64) {
    let gh = crate::reference_grid_h(gw, false);
    let mut p = crate::with_call_args(crate::params::defaults(), seed, WIDTH_KM, gw, gh);
    p.civ.border_friction = strength;
    p.use_gpu = false;
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
    let t0 = std::time::Instant::now();
    let civ = crate::compute_civilisation(
        &ws, gw, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), None, p.lake_balance,
    );
    let secs = t0.elapsed().as_secs_f64();
    (ws, civ, gh, secs)
}

/// Land cells 4-adjacent to a cell owned by another faction (both owned).
fn border_mask(owner: &[i32], gw: usize, gh: usize) -> Vec<bool> {
    let mut b = vec![false; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            for j in [(x + 1 < gw).then(|| i + 1), (y + 1 < gh).then(|| i + gw)].into_iter().flatten() {
                if owner[i] > 0 && owner[j] > 0 && owner[i] != owner[j] {
                    b[i] = true;
                    b[j] = true;
                }
            }
        }
    }
    b
}

/// `analyze.py::straight_share`: every third border pixel, its radius-`r`
/// neighbours (at least `r`), smallest principal deviation under `tol`.
fn linear_share(b: &[bool], gw: usize, r: i64, tol: f64) -> (f64, usize) {
    let pts: Vec<(i64, i64)> = b.iter().enumerate().filter(|(_, v)| **v).map(|(i, _)| ((i % gw) as i64, (i / gw) as i64)).collect();
    let mut grid: std::collections::HashMap<(i64, i64), Vec<usize>> = Default::default();
    for (k, &(x, y)) in pts.iter().enumerate() {
        grid.entry((x / r, y / r)).or_default().push(k);
    }
    let (mut ok, mut tried) = (0usize, 0usize);
    for k in (0..pts.len()).step_by(3) {
        tried += 1;
        let (cx, cy) = pts[k];
        let mut nb = Vec::new();
        for a in -1..=1 {
            for c in -1..=1 {
                if let Some(v) = grid.get(&(cx / r + a, cy / r + c)) {
                    for &j in v {
                        let (dx, dy) = (pts[j].0 - cx, pts[j].1 - cy);
                        if dx * dx + dy * dy <= r * r {
                            nb.push(j);
                        }
                    }
                }
            }
        }
        if (nb.len() as i64) < r {
            continue;
        }
        let n = nb.len() as f64;
        let (mx, my) = nb.iter().fold((0.0, 0.0), |a, &j| (a.0 + pts[j].0 as f64, a.1 + pts[j].1 as f64));
        let (mx, my) = (mx / n, my / n);
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for &j in &nb {
            let (dx, dy) = (pts[j].0 as f64 - mx, pts[j].1 as f64 - my);
            sxx += dx * dx;
            syy += dy * dy;
            sxy += dx * dy;
        }
        let (sxx, syy, sxy) = (sxx / n, syy / n, sxy / n);
        let tr = sxx + syy;
        let det = sxx * syy - sxy * sxy;
        let ev0 = tr / 2.0 - ((tr * tr / 4.0 - det).max(0.0)).sqrt();
        if ev0.max(0.0).sqrt() < tol {
            ok += 1;
        }
    }
    (ok as f64 / tried.max(1) as f64, pts.len())
}

/// Box mean of `a` over a `(2r+1)^2` window clamped at the edges.
fn box_mean(a: &[f32], gw: usize, gh: usize, r: usize) -> Vec<f64> {
    let w1 = gw + 1;
    let mut sat = vec![0f64; w1 * (gh + 1)];
    for y in 0..gh {
        let mut row = 0f64;
        for x in 0..gw {
            row += a[y * gw + x] as f64;
            sat[(y + 1) * w1 + x + 1] = sat[y * w1 + x + 1] + row;
        }
    }
    (0..gw * gh)
        .map(|i| {
            let (x, y) = (i % gw, i / gw);
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(gw));
            let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(gh));
            (sat[y1 * w1 + x1] - sat[y0 * w1 + x1] - sat[y1 * w1 + x0] + sat[y0 * w1 + x0]) / ((x1 - x0) * (y1 - y0)) as f64
        })
        .collect()
}

/// One world's numbers.
struct Row {
    linear: f64,
    border_px: usize,
    high_b: f64,
    high_l: f64,
    ridge_b: f64,
    ridge_l: f64,
    river_b: f64,
    river_l: f64,
    unowned_land: f64,
    roads: u64,
    areas: Vec<usize>,
    secs: f64,
}

fn measure(seed: i32, gw: usize, strength: f64, out: Option<&str>) -> Row {
    let (ws, civ, gh, secs) = generate(seed, gw, strength);
    let n = gw * gh;
    let owner = &civ.territory;
    assert_eq!(owner.len(), n);
    let land: Vec<bool> = ws.field.iter().map(|&h| h as f64 >= ws.sea_level).collect();
    let b = border_mask(owner, gw, gh);
    let (linear, border_px) = linear_share(&b, gw, 12, 0.8);
    let mean = box_mean(&ws.field, gw, gh, (12 * gw / 1024).max(2));
    let order = ws.stream_order.as_deref().expect("a generated world has a stream order");
    let rr = (4 * gw / 1024).max(1);
    let near_river = |i: usize| {
        let (x, y) = (i % gw, i / gw);
        (y.saturating_sub(rr)..(y + rr + 1).min(gh)).any(|yy| (x.saturating_sub(rr)..(x + rr + 1).min(gw)).any(|xx| order[yy * gw + xx] >= 3))
    };
    // High ground at range scale: the land's top quintile of height above the
    // mean of a radius `gw / 48` window (about 85 km on these 4 000 km worlds),
    // so its land baseline is 0.20 by construction.
    let wide = box_mean(&ws.field, gw, gh, (gw / 48).max(2));
    let mut tl: Vec<f64> = (0..n).filter(|&i| land[i]).map(|i| ws.field[i] as f64 - wide[i]).collect();
    tl.sort_by(f64::total_cmp);
    let hi_cut = tl[(tl.len() - 1) * 4 / 5];
    let (mut hb, mut hl) = (0usize, 0usize);
    let (mut nb, mut nl, mut rb, mut rl, mut vb, mut vl) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut land_n, mut unowned) = (0usize, 0usize);
    for i in 0..n {
        if !land[i] {
            continue;
        }
        land_n += 1;
        if owner[i] == 0 {
            unowned += 1;
            continue;
        }
        let ridge = ws.field[i] as f64 > mean[i];
        let riv = near_river(i);
        let high = ws.field[i] as f64 - wide[i] >= hi_cut;
        nl += 1;
        hl += high as usize;
        rl += ridge as usize;
        vl += riv as usize;
        if b[i] {
            nb += 1;
            hb += high as usize;
            rb += ridge as usize;
            vb += riv as usize;
        }
    }
    assert!(nb > 0 && nl > 0, "seed {seed} gw {gw}: no border or no owned land");
    // Where to look: the border cell with the most river-side (resp. high
    // ground) border cells within `gw / 32` cells, for the close-up frames.
    let pick = |hit: &dyn Fn(usize) -> bool| {
        let r = (gw / 32) as i64;
        // Away from the sheet edge, so a close-up can centre on it.
        let (mx, my) = (gw / 6, gh / 6);
        let inside = |i: usize| (mx..gw - mx).contains(&(i % gw)) && (my..gh - my).contains(&(i / gw));
        let cells: Vec<usize> = (0..n).filter(|&i| b[i] && hit(i) && inside(i)).collect();
        let mut best = (0usize, 0usize);
        for &c in cells.iter().step_by(7) {
            let (cx, cy) = ((c % gw) as i64, (c / gw) as i64);
            let k = cells.iter().filter(|&&j| ((j % gw) as i64 - cx).abs() <= r && ((j / gw) as i64 - cy).abs() <= r).count();
            if k > best.0 {
                best = (k, c);
            }
        }
        (best.1 % gw, best.1 / gw, best.0)
    };
    let on_river = |i: usize| {
        let (x, y) = (i % gw, i / gw);
        order[i] >= 3 || (x > 0 && order[i - 1] >= 3) || (x + 1 < gw && order[i + 1] >= 3) || (y > 0 && order[i - gw] >= 3) || (y + 1 < gh && order[i + gw] >= 3)
    };
    let (rx, ry, rk) = pick(&on_river);
    let (hx, hy, hk) = pick(&|i: usize| land[i] && ws.field[i] as f64 - wide[i] >= hi_cut);
    println!("LOOK seed {seed} gw {gw} s {strength:.2}: river seam at ({rx}, {ry}) [{rk} cells], high-ground seam at ({hx}, {hy}) [{hk} cells]");
    let mut h = Fnv::new();
    for w in &civ.ways {
        h.eat_str(&format!("{w:?}"));
    }
    for r in &civ.sea_routes {
        h.eat_str(&format!("{r:?}"));
    }
    for e in &civ.road_edges {
        h.eat_str(&format!("{e:?}"));
    }
    let nf = owner.iter().copied().max().unwrap_or(0).max(0) as usize;
    let mut areas = vec![0usize; nf + 1];
    for &o in owner {
        if o > 0 {
            areas[o as usize] += 1;
        }
    }
    if strength == 0.0 {
        // The relief inputs' land distribution, which the friction's
        // `STEEP_SLOPE_N` and `TPI_REF` are read against.
        let slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, false);
        let mw = box_mean(&ws.field, gw, gh, (gw / 48).max(2));
        let mut sl: Vec<f32> = (0..n).filter(|&i| land[i]).map(|i| slope[i]).collect();
        let mut tp: Vec<f64> = (0..n).filter(|&i| land[i]).map(|i| (ws.field[i] as f64 - mw[i]).abs()).collect();
        sl.sort_by(f32::total_cmp);
        tp.sort_by(f64::total_cmp);
        let q = |k: f64| ((sl.len() - 1) as f64 * k) as usize;
        println!(
            "RELIEF seed {seed} gw {gw}: slope_n p50 {:.2} p90 {:.2} p97 {:.2} p99 {:.2}; |tpi| p50 {:.4} p90 {:.4} p97 {:.4} p99 {:.4}",
            sl[q(0.5)], sl[q(0.9)], sl[q(0.97)], sl[q(0.99)], tp[q(0.5)], tp[q(0.9)], tp[q(0.97)], tp[q(0.99)]
        );
    }
    if let Some(dir) = out {
        write_png(dir, seed, gw, gh, strength, &ws, owner, order, &land);
    }
    Row {
        linear,
        border_px,
        high_b: hb as f64 / nb as f64,
        high_l: hl as f64 / nl as f64,
        ridge_b: rb as f64 / nb as f64,
        ridge_l: rl as f64 / nl as f64,
        river_b: vb as f64 / nb as f64,
        river_l: vl as f64 / nl as f64,
        unowned_land: unowned as f64 / land_n.max(1) as f64,
        roads: h.0,
        areas,
        secs,
    }
}

/// Scratch composite: grey relief (light from the north-west), each owned
/// cell tinted by faction, order >= 3 rivers blue, border cells black.
#[allow(clippy::too_many_arguments)]
fn write_png(dir: &str, seed: i32, gw: usize, gh: usize, s: f64, ws: &cartalith_engine::WorldState, owner: &[i32], order: &[i16], land: &[bool]) {
    const TINT: [[u8; 3]; 8] = [[200, 60, 60], [60, 140, 200], [220, 180, 40], [80, 170, 90], [160, 90, 190], [230, 120, 40], [90, 190, 190], [200, 110, 150]];
    let b = border_mask(owner, gw, gh);
    let mut px = vec![0u8; gw * gh * 3];
    for i in 0..gw * gh {
        let (x, y) = (i % gw, i / gw);
        let c = if !land[i] {
            [40, 60, 110]
        } else if b[i] {
            [0, 0, 0]
        } else if order[i] >= 3 {
            [30, 90, 230]
        } else {
            let h = ws.field[i] as f64;
            let hl = ws.field[y.saturating_sub(1) * gw + x.saturating_sub(1)] as f64;
            let shade = (0.6 + (h - hl) * 40.0 * gw as f64 / 1024.0).clamp(0.2, 1.0);
            let base = (110.0 + (h - ws.sea_level) * 300.0).clamp(60.0, 255.0) * shade;
            let g = base as u8;
            if owner[i] > 0 {
                let t = TINT[(owner[i] as usize - 1) % 8];
                [((g as u32 + t[0] as u32) / 2) as u8, ((g as u32 + t[1] as u32) / 2) as u8, ((g as u32 + t[2] as u32) / 2) as u8]
            } else {
                [g, g, g]
            }
        };
        px[i * 3..i * 3 + 3].copy_from_slice(&c);
    }
    let path = format!("{dir}/harness_{seed}_{gw}_s{s:.2}.png");
    let f = std::fs::File::create(&path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), gw as u32, gh as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
}

fn list<T: std::str::FromStr>(var: &str, default: &[T]) -> Vec<T>
where
    T: Clone,
{
    match std::env::var(var) {
        Ok(v) => v.split(',').map(|s| s.trim().parse().ok().expect("a number")).collect(),
        Err(_) => default.to_vec(),
    }
}

/// Protects nothing (pins nothing): prints Ruling GO's bars over the three
/// research seeds. Asserts only that each world reached the code (a border and
/// owned land exist) and that the roads did not move with the strength, which
/// is the ruling's own "roads bit-identical" bar.
#[test]
#[ignore = "Ruling GO measurement over generated worlds; run alone with --ignored"]
fn border_harness() {
    let gws: Vec<usize> = list("BORDER_GW", &[384, 2048]);
    let ss: Vec<f64> = list("BORDER_S", &[0.0, 0.6]);
    let out = std::env::var("BORDER_HARNESS_OUT").ok();
    for &gw in &gws {
        for &seed in &SEEDS {
            let mut base: Option<Row> = None;
            for &s in &ss {
                let r = measure(seed, gw, s, out.as_deref());
                let (area_ratio, roads_same) = match &base {
                    Some(b0) => {
                        let worst = b0
                            .areas
                            .iter()
                            .zip(&r.areas)
                            .skip(1)
                            .filter(|(a, _)| **a > 0)
                            .map(|(&a, &c)| { let q = c as f64 / a as f64; q.max(1.0 / q.max(1e-9)) })
                            .fold(1.0f64, f64::max);
                        (worst, b0.roads == r.roads)
                    }
                    None => (1.0, true),
                };
                println!(
                    "BORDER seed {seed} gw {gw} s {s:.2}: linear {:.3} (border px {}), high ground {:.3}/{:.3} = x{:.2}, ridge {:.3}/{:.3} = x{:.2}, river {:.3}/{:.3} = x{:.2}, unowned land {:.4}, worst faction area ratio {:.2}, roads 0x{:016x} same {}, civ {:.1} s",
                    r.linear, r.border_px, r.high_b, r.high_l, r.high_b / r.high_l, r.ridge_b, r.ridge_l, r.ridge_b / r.ridge_l, r.river_b, r.river_l, r.river_b / r.river_l,
                    r.unowned_land, area_ratio, r.roads, roads_same, r.secs
                );
                assert!(roads_same, "seed {seed} gw {gw}: the roads moved with border_friction {s}");
                if base.is_none() {
                    base = Some(r);
                }
            }
        }
    }
}

/// Protects: `compute_civilisation` reads `civ.border_friction` (a world at
/// 0.6 owns its land differently from the same world at 0) and the ruling's
/// "roads bit-identical" bar (ways, sea routes and road edges hash the same at
/// both strengths). A small grid, so it runs with the suite.
#[test]
fn border_friction_moves_territory_not_roads() {
    let (_, a, _, _) = generate(483920, 160, 0.0);
    let (_, b, _, _) = generate(483920, 160, 0.6);
    let owned = a.territory.iter().filter(|&&o| o > 0).count();
    assert!(owned > 1000, "the fixture must own land ({owned} cells)");
    let moved = a.territory.iter().zip(&b.territory).filter(|(x, y)| x != y).count();
    assert!(moved > 0, "border_friction 0.6 moved no territory");
    let hash = |c: &crate::CivData| {
        let mut h = Fnv::new();
        for w in &c.ways {
            h.eat_str(&format!("{w:?}"));
        }
        for r in &c.sea_routes {
            h.eat_str(&format!("{r:?}"));
        }
        for e in &c.road_edges {
            h.eat_str(&format!("{e:?}"));
        }
        h.0
    };
    assert!(!a.ways.is_empty(), "the fixture must have roads");
    assert_eq!(hash(&a), hash(&b), "the roads moved with border_friction");
}
