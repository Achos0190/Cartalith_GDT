//! **River comb thinning, measured** (Ruling BW item 3, 2026-10-10: draw-side
//! thinning of the coastal and lake-shore combs Ruling BV's option 1 left
//! behind -- `OUTSTANDING_WORK.md`, "Coastal and lake-shore river combs").
//!
//! Puts [`river_draw_plan_unthinned`] (the plan before the thinning) beside
//! [`river_draw_plan`] (the plan the app draws; map, deep-zoom tiles and export
//! all reach it through `WorldGen::river_draws`) on the SAME generated world
//! and reports, per seed and river density:
//!
//! * **streak runs** -- the metric Ruling BV's `DECISIONS.md` 7p paragraph
//!   quotes (762/228/518 -> 400/228/177): a DRAWN run that is order 1, has at
//!   least 4 cells, takes >= 90 % of its steps in ONE D8 direction and ends at
//!   water. The harness that produced those numbers was never committed; this
//!   is the definition `build_channels_routed`'s doc gives, re-implemented.
//! * **the guard side** -- drawn length of order >= 3 runs, of runs at or
//!   above [`COMB_DISCHARGE_FRAC`], and of every run with a tributary: none
//!   may drop AT ALL; total drawn length may drop only through comb teeth;
//!   the count of drawn fan-eligible mouths (order >= 3, big enough for a
//!   `river_delta` fan, on a shore) may not drop.
//!
//! ```text
//! cargo test --release -p cartalith-godot --test river_comb_harness -- --ignored --nocapture --test-threads=1 river_comb_measurement
//! ```
//!
//! `RC_SEEDS`, `RC_DENSITIES`, `RC_GW`, `RC_KM` override the defaults (the
//! owner's world: seed 246371, 2048 wide, 40 075 km). An unparsable value
//! panics rather than falling back.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{WorldState, generate_terrain};
use cartalith_hydrology::{
    COMB_DISCHARGE_FRAC, River, RiverDrawPlan, river_draw_plan, river_draw_plan_unthinned, river_entities, river_flow_thresh,
    river_width_scale_k,
};

/// What the plan decided about one world, in the two senses the module doc
/// names: how many comb strips remain, and what must not have moved.
#[derive(Default, Debug, Clone, PartialEq)]
struct CombStats {
    runs: usize,
    drawn: usize,
    streak_runs: usize,
    drawn_len: f64,
    /// Drawn length of runs of Strahler order >= 3.
    len_order3: f64,
    /// Drawn length of runs at or above the discharge cap.
    len_heavy: f64,
    /// Drawn length of runs that have at least one tributary.
    len_trunks: f64,
    /// Runs of order >= 3 or heavy or with tributaries that are drawn.
    protected_drawn: usize,
    /// Drawn runs whose outlet is on a shore AND that are big enough for a
    /// delta fan (order >= 3 and discharge >= 2e-4 of the grid).
    fan_mouths: usize,
}

/// A world at the app's parameters (integrated drainage on) with the given
/// river density.
fn world(seed: i32, density: f64, km: f64, gw: usize, gh: usize) -> (WorldState, bool) {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = km;
    p.use_gpu = false;
    p.river_density = density;
    assert!(p.integrate_drainage, "the app's defaults must have integrated drainage on");
    let wrap = p.world;
    let ws = generate_terrain(&p);
    assert!(ws.integrated_drainage);
    (ws, wrap)
}

/// Whether `c` or a neighbour is water or at/below sea level (the shore a
/// run's outlet must be on to be a streak or a fan mouth).
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

/// A run's streak test: >= 4 cells, >= 90 % of its steps one D8 vector.
fn is_straight_strip(r: &River) -> bool {
    if r.pts.len() < 4 {
        return false;
    }
    let steps: Vec<(i64, i64)> =
        r.pts.windows(2).map(|p| ((p[1].0.floor() - p[0].0.floor()) as i64, (p[1].1.floor() - p[0].1.floor()) as i64)).collect();
    let mut best = 0usize;
    for s in &steps {
        best = best.max(steps.iter().filter(|t| *t == s).count());
    }
    best as f64 >= 0.9 * steps.len() as f64
}

/// Everything [`CombStats`] holds for one plan.
fn stats(rivers: &[River], plan: &RiverDrawPlan, gw: usize, gh: usize, fld: &[f32], sea: f64, water: &[u8]) -> CombStats {
    let cap = COMB_DISCHARGE_FRAC * (gw * gh) as f32;
    let mut s = CombStats { runs: rivers.len(), ..Default::default() };
    for (i, r) in rivers.iter().enumerate() {
        if plan.parallel_of[i].is_some() {
            continue;
        }
        s.drawn += 1;
        s.drawn_len += r.length_cells;
        let at_shore = shore(r.mouth as usize, gw, gh, fld, sea, water);
        let heavy = r.discharge >= cap;
        if r.order >= 3 {
            s.len_order3 += r.length_cells;
        }
        if heavy {
            s.len_heavy += r.length_cells;
        }
        if r.tributaries > 0 {
            s.len_trunks += r.length_cells;
        }
        if r.order >= 3 || heavy || r.tributaries > 0 {
            s.protected_drawn += 1;
        }
        if r.order >= 3 && heavy && at_shore {
            s.fan_mouths += 1;
        }
        if r.order == 1 && at_shore && is_straight_strip(r) {
            s.streak_runs += 1;
        }
    }
    s
}

/// The thinned and unthinned stats of one world, plus the plans.
fn measure(ws: &WorldState, wrap: bool, km: f64, gw: usize, gh: usize) -> (CombStats, CombStats, Vec<River>, RiverDrawPlan, RiverDrawPlan) {
    let n = gw * gh;
    let order = ws.stream_order.as_ref().expect("river extraction ran");
    let ch = ws.channels.as_ref().expect("a generated world retains its channel tree");
    let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
    let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, ws.sea_level, wrap);
    let tree = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
    // `combined_receivers`, as `WorldGen::river_draws` hands the plan.
    let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { tree[i] }).collect();
    let rivers = river_entities(
        order,
        &ch.recv,
        &ws.flow_discharge,
        &ws.field,
        gw,
        gh,
        1,
        river_flow_thresh(gw, gh, gw, km),
        river_width_scale_k(km),
        wrap,
    );
    let raw = river_draw_plan_unthinned(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
    let thin = river_draw_plan(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
    let a = stats(&rivers, &raw, gw, gh, &ws.field, ws.sea_level, &water);
    let b = stats(&rivers, &thin, gw, gh, &ws.field, ws.sea_level, &water);
    (a, b, rivers, raw, thin)
}

/// Env override, or `default`; an unparsable value panics.
fn env_list<T: std::str::FromStr + Copy>(key: &str, default: &[T]) -> Vec<T> {
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse {s:?}"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// Asserts the guard side: nothing protected moved, and what was hidden was
/// only ever a comb tooth (so total length can only drop by teeth).
fn assert_guards(tag: &str, rivers: &[River], raw: &RiverDrawPlan, thin: &RiverDrawPlan, a: &CombStats, b: &CombStats, gw: usize, gh: usize) {
    assert_eq!(a.len_order3, b.len_order3, "{tag}: drawn length of order >= 3 runs moved");
    assert_eq!(a.len_heavy, b.len_heavy, "{tag}: drawn length of runs at/above the discharge cap moved");
    assert_eq!(a.len_trunks, b.len_trunks, "{tag}: drawn length of runs with tributaries moved");
    assert_eq!(a.protected_drawn, b.protected_drawn, "{tag}: a protected run was hidden");
    assert_eq!(a.fan_mouths, b.fan_mouths, "{tag}: a delta-fan mouth was hidden");
    let cap = COMB_DISCHARGE_FRAC * (gw * gh) as f32;
    for (i, r) in rivers.iter().enumerate() {
        if raw.parallel_of[i].is_none() && thin.parallel_of[i].is_some() {
            assert!(
                r.order <= 1 && r.tributaries == 0 && r.discharge < cap,
                "{tag}: run {i} (order {}, {} tributaries, discharge {}) was thinned but is not a comb tooth",
                r.order,
                r.tributaries,
                r.discharge
            );
        }
        // Thinning never un-hides.
        assert!(raw.parallel_of[i].is_none() || thin.parallel_of[i].is_some(), "{tag}: run {i} was hidden before and drawn after");
    }
    assert!(b.drawn_len <= a.drawn_len, "{tag}: drawn length grew");
}

/// Protects: the comb thinning's two sides on a small real world at the app's
/// parameters -- it hides at least one run when densified (a positive control:
/// a thinning that does nothing passes every "nothing protected moved" bar),
/// and never hides a trunk, an order >= 3 run, a heavy run or a fan mouth.
/// Mutation: disabling the thinning fails the positive control; widening the
/// cap to infinity (COMB_MAX_ORDER = i16::MAX, COMB_DISCHARGE_FRAC = inf) fails
/// `assert_guards`.
#[test]
fn comb_thinning_hides_teeth_and_only_teeth() {
    let (gw, gh, km) = (512usize, 328usize, 40_075.0);
    let (ws, wrap) = world(246_371, 1.55, km, gw, gh);
    let (a, b, rivers, raw, thin) = measure(&ws, wrap, km, gw, gh);
    println!("512x328 d1.55: before {a:?}\n               after  {b:?}");
    assert!(a.drawn > 50, "a world with no drawn rivers measures nothing");
    assert_guards("512x328 d1.55", &rivers, &raw, &thin, &a, &b, gw, gh);
    assert!(b.drawn < a.drawn, "the thinning hid no run at all on a dense coastal world");
    assert!(b.streak_runs <= a.streak_runs);
}

/// Protects: the plan is deterministic and the thinned plan differs from the
/// unthinned one only by added `parallel_of` entries -- checked on the real
/// world by `comb_thinning_hides_teeth_and_only_teeth`; this one pins that the
/// unthinned entry point is the pre-change plan on the default world (no
/// density change: seed 483920, 384 wide), where thinning may hide teeth but
/// every pre-existing hide is kept.
#[test]
fn thinned_plan_keeps_every_existing_hide() {
    let (gw, gh, km) = (384usize, 246usize, 800.0);
    let (ws, wrap) = world(483_920, 1.0, km, gw, gh);
    let (a, b, rivers, raw, thin) = measure(&ws, wrap, km, gw, gh);
    assert_guards("384 d1.0", &rivers, &raw, &thin, &a, &b, gw, gh);
}

/// Protects: nothing by assertion beyond `assert_guards` -- the comb-thinning measurement the module doc
/// describes. `#[ignore]`d: ~tens of seconds
/// to minutes per world at 2048x1311; run alone in release.
#[test]
#[ignore = "comb measurement at the owner's 2048x1311 world; run alone in release (see the module doc)"]
fn river_comb_measurement() {
    let gw = env_list::<usize>("RC_GW", &[2048])[0];
    let gh = ((gw as f64 * 0.64).round() as usize).max(4);
    let km = env_list::<f64>("RC_KM", &[40_075.0])[0];
    let seeds = env_list::<i32>("RC_SEEDS", &[246_371]);
    let dens = env_list::<f64>("RC_DENSITIES", &[1.55, 1.0]);
    for seed in seeds {
        for &d in &dens {
            let t0 = std::time::Instant::now();
            let (ws, wrap) = world(seed, d, km, gw, gh);
            let (a, b, rivers, raw, thin) = measure(&ws, wrap, km, gw, gh);
            println!("seed {seed} density {d} grid {gw}x{gh} {km} km ({:.0}s)", t0.elapsed().as_secs_f64());
            println!("  runs {}: drawn {} -> {}", a.runs, a.drawn, b.drawn);
            println!("  STREAK RUNS {} -> {}", a.streak_runs, b.streak_runs);
            println!("  drawn length (cells) {:.0} -> {:.0} ({:+.1} %)", a.drawn_len, b.drawn_len, 100.0 * (b.drawn_len / a.drawn_len - 1.0));
            println!(
                "  guard: order>=3 length {:.0} -> {:.0}; heavy length {:.0} -> {:.0}; trunk length {:.0} -> {:.0}; protected drawn {} -> {}; fan mouths {} -> {}",
                a.len_order3, b.len_order3, a.len_heavy, b.len_heavy, a.len_trunks, b.len_trunks, a.protected_drawn, b.protected_drawn, a.fan_mouths, b.fan_mouths
            );
            let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
            print_remaining_spacing(&rivers, &thin, gw, &ws.field, ws.sea_level, &water, gh);
            print_hotspots(&rivers, &raw, &thin, gw);
            assert_guards(&format!("seed {seed} d{d}"), &rivers, &raw, &thin, &a, &b, gw, gh);
        }
    }
}

/// Prints, for the streak runs the thinned plan still draws, their stream
/// length `L` and the distance from each one's outlet to the nearest OTHER
/// drawn outlet on a shore, as quartiles (min, 25 %, median, 75 %, max), next
/// to `0.5 L`. A comb the thinning left behind is one whose neighbour is
/// farther than `0.5 L`; this shows how far.
fn print_remaining_spacing(rivers: &[River], thin: &RiverDrawPlan, gw: usize, fld: &[f32], sea: f64, water: &[u8], gh: usize) {
    let mouths: Vec<(usize, (f64, f64))> = rivers
        .iter()
        .enumerate()
        .filter(|&(i, r)| thin.parallel_of[i].is_none() && shore(r.mouth as usize, gw, gh, fld, sea, water))
        .map(|(i, r)| (i, ((r.mouth as usize % gw) as f64, (r.mouth as usize / gw) as f64)))
        .collect();
    let mut ls = Vec::new();
    let mut ds = Vec::new();
    for &(i, p) in &mouths {
        let r = &rivers[i];
        if !(r.order == 1 && is_straight_strip(r)) {
            continue;
        }
        let d = mouths.iter().filter(|&&(j, _)| j != i).map(|&(_, q)| ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt()).fold(f64::MAX, f64::min);
        ls.push(r.length_cells);
        ds.push(d);
    }
    let q = |v: &mut Vec<f64>| -> String {
        v.sort_by(|a, b| a.total_cmp(b));
        if v.is_empty() {
            return "none".into();
        }
        let at = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
        format!("{:.1}/{:.1}/{:.1}/{:.1}/{:.1}", at(0.0), at(0.25), at(0.5), at(0.75), at(1.0))
    };
    println!("  remaining streaks: L (min/q1/med/q3/max) {}; nearest other outlet {}", q(&mut ls), q(&mut ds));
}

/// Prints the three 64x64-cell blocks where the thinning hid the most runs
/// (the places a before/after screenshot should look at): block centre in
/// grid cells and the count of runs the comb pass hid there.
fn print_hotspots(rivers: &[River], raw: &RiverDrawPlan, thin: &RiverDrawPlan, gw: usize) {
    let mut bins: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for (i, r) in rivers.iter().enumerate() {
        if raw.parallel_of[i].is_none() && thin.parallel_of[i].is_some() {
            let (x, y) = (r.mouth as usize % gw, r.mouth as usize / gw);
            *bins.entry((x / 64, y / 64)).or_insert(0) += 1;
        }
    }
    let mut v: Vec<_> = bins.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for ((bx, by), n) in v.into_iter().take(3) {
        println!("  hotspot: {n} runs thinned near cell {}:{}", bx * 64 + 32, by * 64 + 32);
    }
}

// ---------------------------------------------------------------------------
// Ruling CA item 4: BEFORE (Ruling BW: 0.5 x length, cap 24) vs AFTER (the
// shipped constants) on the harness worlds, with PNGs for the owner.
// ---------------------------------------------------------------------------

/// Per-plan numbers for the before/after table. `comb_hidden` is runs hidden by
/// the comb pass alone (hidden in this plan, drawn in the unthinned plan).
#[derive(Debug, Default, Clone)]
struct BaStats {
    drawn: usize,
    comb_hidden: usize,
    comb_groups: usize,
    drawn_len: f64,
    removed_len: f64,
}

/// Counts for one plan against the unthinned plan `raw`. A "comb group" is one
/// distinct host (`parallel_of` target) among the comb-hidden runs.
fn ba_stats(rivers: &[River], raw: &RiverDrawPlan, plan: &RiverDrawPlan) -> BaStats {
    let mut s = BaStats::default();
    let mut hosts = std::collections::HashSet::new();
    for (i, r) in rivers.iter().enumerate() {
        if plan.parallel_of[i].is_none() {
            s.drawn += 1;
            s.drawn_len += r.length_cells;
        } else if raw.parallel_of[i].is_none() {
            s.comb_hidden += 1;
            s.removed_len += r.length_cells;
            hosts.insert(plan.parallel_of[i].unwrap());
        }
    }
    s.comb_groups = hosts.len();
    s
}

/// Connectivity between two plans over the SAME `rivers` slice (so the traced
/// path set is identical by construction: neither plan can move a run's
/// `pts`). Returns `(dangling_hosts, lost_shore_rivers, orphaned_tributaries,
/// loose_before, loose_after)`:
/// * dangling hosts: runs hidden in AFTER whose host is itself hidden;
/// * lost shore rivers: runs that reach a shore, were drawn BEFORE, and are now
///   neither drawn nor hidden behind a drawn host that also reaches a shore;
/// * orphaned tributaries: runs drawn in AFTER that join (outlet cell owned by
///   another run) a run hidden in AFTER but drawn BEFORE;
/// * loose ends per plan: drawn runs whose outlet is not on a shore, does not
///   join a drawn run, and that have neither a bridge nor an extension.
fn ba_connectivity(
    rivers: &[River],
    before: &RiverDrawPlan,
    after: &RiverDrawPlan,
    gw: usize,
    gh: usize,
    fld: &[f32],
    sea: f64,
    water: &[u8],
) -> (usize, usize, usize, usize, usize) {
    let n = gw * gh;
    let cell = |p: (f64, f64)| (p.1 as usize).min(gh - 1) * gw + (p.0 as usize).min(gw - 1);
    let mut owner = vec![usize::MAX; n];
    for (i, r) in rivers.iter().enumerate() {
        for &p in &r.pts {
            let c = cell(p);
            if owner[c] == usize::MAX {
                owner[c] = i;
            }
        }
    }
    let on_shore: Vec<bool> = rivers.iter().map(|r| shore(r.mouth as usize, gw, gh, fld, sea, water)).collect();
    let (mut dangling_host, mut lost, mut orphan) = (0, 0, 0);
    for (i, r) in rivers.iter().enumerate() {
        if let Some(h) = after.parallel_of[i] {
            if after.parallel_of[h].is_some() {
                dangling_host += 1;
            }
        }
        if on_shore[i] && before.parallel_of[i].is_none() {
            let ok = match after.parallel_of[i] {
                None => true,
                Some(h) => after.parallel_of[h].is_none() && on_shore[h],
            };
            if !ok {
                lost += 1;
            }
        }
        let t = owner[r.mouth as usize];
        if after.parallel_of[i].is_none() && t != i && t != usize::MAX && after.parallel_of[t].is_some() && before.parallel_of[t].is_none() {
            orphan += 1;
        }
    }
    let loose = |p: &RiverDrawPlan| {
        (0..rivers.len())
            .filter(|&i| {
                let t = owner[rivers[i].mouth as usize];
                p.parallel_of[i].is_none()
                    && !on_shore[i]
                    && !(t != i && t != usize::MAX && p.parallel_of[t].is_none())
                    && p.bridge[i].is_none()
                    && p.extension[i].is_empty()
            })
            .count()
    };
    (dangling_host, lost, orphan, loose(before), loose(after))
}

/// Writes an RGB PNG with the `png` crate (already a dependency).
fn write_png(path: &std::path::Path, w: usize, h: usize, rgb: &[u8]) {
    let f = std::io::BufWriter::new(std::fs::File::create(path).expect("create png"));
    let mut enc = png::Encoder::new(f, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().expect("png header").write_image_data(rgb).expect("png data");
}

/// One panel: grey elevation (sea darker), the drawn runs of `plan` blue and
/// the runs flagged in `red` red, `scale` pixels per cell, over the crop
/// `(x0, y0, cw, ch)`.
fn render_panel(
    rivers: &[River],
    plan: &RiverDrawPlan,
    red: &[bool],
    fld: &[f32],
    sea: f64,
    gw: usize,
    (x0, y0, cw, ch): (usize, usize, usize, usize),
    scale: usize,
) -> Vec<u8> {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for y in y0..y0 + ch {
        for x in x0..x0 + cw {
            let v = fld[y * gw + x];
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    let (pw, ph) = (cw * scale, ch * scale);
    let mut buf = vec![0u8; pw * ph * 3];
    for py in 0..ph {
        for px in 0..pw {
            let v = fld[(y0 + py / scale) * gw + x0 + px / scale];
            let g = if v as f64 <= sea { 60 } else { 110 + (((v - lo) / (hi - lo).max(1e-6)) * 120.0) as u8 };
            let o = (py * pw + px) * 3;
            buf[o..o + 3].copy_from_slice(&[g, g, g]);
        }
    }
    let mut dot = |x: i64, y: i64, c: [u8; 3]| {
        if x < x0 as i64 || y < y0 as i64 || x >= (x0 + cw) as i64 || y >= (y0 + ch) as i64 {
            return;
        }
        for sy in 0..scale {
            for sx in 0..scale {
                let o = ((((y as usize - y0) * scale + sy) * pw) + (x as usize - x0) * scale + sx) * 3;
                buf[o..o + 3].copy_from_slice(&c);
            }
        }
    };
    for (i, r) in rivers.iter().enumerate() {
        let hidden = plan.parallel_of[i].is_some();
        let colour = if red[i] {
            [230, 30, 30]
        } else if !hidden {
            [20, 70, 235]
        } else {
            continue;
        };
        for p in r.pts.windows(2).flat_map(|w| {
            // Step between consecutive cell centres so no gap shows.
            let (a, b) = ((w[0].0 as i64, w[0].1 as i64), (w[1].0 as i64, w[1].1 as i64));
            let k = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
            (0..=k).map(move |t| (a.0 + (b.0 - a.0) * t / k, a.1 + (b.1 - a.1) * t / k))
        }) {
            dot(p.0, p.1, colour);
        }
        if !hidden {
            for &(x, y) in &plan.extension[i] {
                dot(x as i64, y as i64, colour);
            }
        }
    }
    buf
}

/// The crop `(count, x0, y0)` (cw x ch cells, stride 20) holding the most
/// mouths of runs for which `score` is true.
fn densest_crop(rivers: &[River], score: &dyn Fn(usize) -> bool, gw: usize, gh: usize, cw: usize, ch: usize) -> (usize, usize, usize) {
    let mut best = (0usize, 0usize, 0usize);
    let pts: Vec<(usize, usize)> =
        (0..rivers.len()).filter(|&i| score(i)).map(|i| (rivers[i].mouth as usize % gw, rivers[i].mouth as usize / gw)).collect();
    let mut y0 = 0;
    while y0 + ch <= gh {
        let mut x0 = 0;
        while x0 + cw <= gw {
            let c = pts.iter().filter(|&&(x, y)| x >= x0 && x < x0 + cw && y >= y0 && y < y0 + ch).count();
            if c > best.0 {
                best = (c, x0, y0);
            }
            x0 += 20;
        }
        y0 += 20;
    }
    best
}

/// Protects: nothing by value -- it is the measurement and the picture
/// generator for Ruling CA item 4's owner approval, and it asserts connectivity
/// (no shore river lost, no orphaned tributary, no hidden host that is itself
/// hidden) plus `assert_guards` between the two strengths. BEFORE is
/// [`cartalith_hydrology::COMB_RULING_BW`] (0.5 x, cap 24), AFTER the shipped
/// constants. Writes PNGs to `RC_OUT` (default: current dir). `#[ignore]`d.
///
/// ```text
/// RC_OUT=<dir> cargo test --release -p cartalith-godot --test river_comb_harness -- --ignored --nocapture --test-threads=1 river_comb_before_after
/// ```
#[test]
#[ignore = "before/after comb pictures at 2048x1311; run alone in release (see the doc)"]
fn river_comb_before_after() {
    let out = std::path::PathBuf::from(std::env::var("RC_OUT").unwrap_or_else(|_| ".".into()));
    std::fs::create_dir_all(&out).unwrap();
    let worlds: [(i32, usize, f64); 4] = [(483_920, 2048, 800.0), (24_601, 2048, 800.0), (71_077_345, 2048, 800.0), (246_371, 512, 40_075.0)];
    let density = params::defaults().river_density;
    for (seed, gw, km) in worlds {
        let gh = ((gw as f64 * 0.64).round() as usize).max(4);
        let n = gw * gh;
        let (ws, wrap) = world(seed, density, km, gw, gh);
        let order = ws.stream_order.as_ref().unwrap();
        let ch = ws.channels.as_ref().unwrap();
        let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
        let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, ws.sea_level, wrap);
        let tree = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
        let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { tree[i] }).collect();
        let rivers = river_entities(order, &ch.recv, &ws.flow_discharge, &ws.field, gw, gh, 1, river_flow_thresh(gw, gh, gw, km), river_width_scale_k(km), wrap);
        let raw = river_draw_plan_unthinned(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
        let bw = cartalith_hydrology::COMB_RULING_BW;
        let before = cartalith_hydrology::river_draw_plan_with_comb(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water, bw);
        let after = river_draw_plan(&rivers, &ws.flow_discharge, &ws.field, ws.sea_level, gw, gh, &down, &water);
        let (sb, sa) = (ba_stats(&rivers, &raw, &before), ba_stats(&rivers, &raw, &after));
        let removed_now: Vec<bool> = (0..rivers.len()).map(|i| before.parallel_of[i].is_none() && after.parallel_of[i].is_some()).collect();
        let newly = removed_now.iter().filter(|&&b| b).count();
        let newly_len: f64 = (0..rivers.len()).filter(|&i| removed_now[i]).map(|i| rivers[i].length_cells).sum();
        let (dang, lost, orphan, lb, la) = ba_connectivity(&rivers, &before, &after, gw, gh, &ws.field, ws.sea_level, &water);
        let tag = format!("seed {seed} {gw}x{gh} {km} km density {density}");
        println!("== {tag}: traced runs {} (one slice, identical for both plans); unthinned drawn {}", rivers.len(), rivers.len() - raw.parallel_of.iter().filter(|p| p.is_some()).count());
        println!("   BEFORE (0.5x, cap 24): drawn {} comb-hidden {} groups {} drawn_len {:.0} removed_len {:.0}", sb.drawn, sb.comb_hidden, sb.comb_groups, sb.drawn_len, sb.removed_len);
        println!("   AFTER  (shipped constants): drawn {} comb-hidden {} groups {} drawn_len {:.0} removed_len {:.0}", sa.drawn, sa.comb_hidden, sa.comb_groups, sa.drawn_len, sa.removed_len);
        println!(
            "   AFTER vs BEFORE: strands newly removed {newly} ({newly_len:.0} cells); drawn length -{:.2} % of BEFORE; AFTER removes {:.2} % of unthinned drawn length (BEFORE {:.2} %)",
            100.0 * (sb.drawn_len - sa.drawn_len) / sb.drawn_len,
            100.0 * sa.removed_len / (sa.drawn_len + sa.removed_len),
            100.0 * sb.removed_len / (sb.drawn_len + sb.removed_len)
        );
        println!("   connectivity: hidden-run hosts not drawn {dang}; shore rivers lost {lost}; tributaries orphaned {orphan}; loose ends before {lb} after {la}");
        let guards_a = stats(&rivers, &before, gw, gh, &ws.field, ws.sea_level, &water);
        let guards_b = stats(&rivers, &after, gw, gh, &ws.field, ws.sea_level, &water);
        assert_eq!((dang, lost, orphan), (0, 0, 0), "{tag}: connectivity broken");
        // The guard side of `assert_guards`, minus its "never un-hides" clause:
        // that clause compares the unthinned plan with a thinned one, and holds
        // there. Between two thinning STRENGTHS it need not (greedy, heaviest
        // first: a tooth hidden behind a host the stronger pass itself hides can
        // be drawn again when no kept run is within its own spacing), so the
        // count is reported instead.
        assert_eq!(guards_a.len_order3, guards_b.len_order3, "{tag}: order >= 3 length moved");
        assert_eq!(guards_a.len_heavy, guards_b.len_heavy, "{tag}: heavy length moved");
        assert_eq!(guards_a.len_trunks, guards_b.len_trunks, "{tag}: trunk length moved");
        assert_eq!(guards_a.protected_drawn, guards_b.protected_drawn, "{tag}: a protected run was hidden");
        assert_eq!(guards_a.fan_mouths, guards_b.fan_mouths, "{tag}: a fan mouth was hidden");
        let capq = COMB_DISCHARGE_FRAC * n as f32;
        for (i, r) in rivers.iter().enumerate() {
            if raw.parallel_of[i].is_none() && after.parallel_of[i].is_some() {
                assert!(r.order <= 1 && r.tributaries == 0 && r.discharge < capq, "{tag}: run {i} thinned but not a comb tooth");
            }
        }
        let unhidden = (0..rivers.len()).filter(|&i| before.parallel_of[i].is_some() && after.parallel_of[i].is_none()).count();
        println!("   runs hidden BEFORE but drawn AFTER (greedy re-draw): {unhidden}");
        // Pictures: "dense" = the crop with the most BEFORE comb candidates
        // (drawn order-1 shore runs); "removed" = where AFTER removes the most.
        let cand = |i: usize| before.parallel_of[i].is_none() && rivers[i].order == 1 && shore(rivers[i].mouth as usize, gw, gh, &ws.field, ws.sea_level, &water);
        let gone = |i: usize| removed_now[i];
        let (cw, chh) = (400usize.min(gw), 300usize.min(gh));
        for (name, sc) in [("dense", &cand as &dyn Fn(usize) -> bool), ("removed", &gone as &dyn Fn(usize) -> bool)] {
            let (cnt, x0, y0) = densest_crop(&rivers, sc, gw, gh, cw, chh);
            let none = vec![false; rivers.len()];
            let l = render_panel(&rivers, &before, &none, &ws.field, ws.sea_level, gw, (x0, y0, cw, chh), 2);
            let r = render_panel(&rivers, &after, &removed_now, &ws.field, ws.sea_level, gw, (x0, y0, cw, chh), 2);
            let (pw, ph) = (cw * 2, chh * 2);
            let gap = 8;
            let mut img = vec![255u8; (pw * 2 + gap) * ph * 3];
            for y in 0..ph {
                let o = y * (pw * 2 + gap) * 3;
                img[o..o + pw * 3].copy_from_slice(&l[y * pw * 3..(y + 1) * pw * 3]);
                let o2 = o + (pw + gap) * 3;
                img[o2..o2 + pw * 3].copy_from_slice(&r[y * pw * 3..(y + 1) * pw * 3]);
            }
            let path = out.join(format!("comb_{seed}_{name}.png"));
            write_png(&path, pw * 2 + gap, ph, &img);
            println!("   picture {name}: crop x0 {x0} y0 {y0} {cw}x{chh}, {cnt} scored mouths -> {}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Owner follow-up to Ruling CA item 4: a (factor, cap) GRID, to find where
// stronger comb thinning starts to cost real rivers. Measurement only; the
// shipped constants are not changed by anything here.
// ---------------------------------------------------------------------------

/// One generated world with everything the comb measurements need.
struct GridWorld {
    ws: WorldState,
    rivers: Vec<River>,
    water: Vec<u8>,
    down: Vec<i32>,
    gw: usize,
    gh: usize,
}

/// Builds a harness world exactly as `river_comb_before_after` does.
fn grid_world(seed: i32, gw: usize, km: f64) -> GridWorld {
    let gh = ((gw as f64 * 0.64).round() as usize).max(4);
    let n = gw * gh;
    let (ws, wrap) = world(seed, params::defaults().river_density, km, gw, gh);
    let order = ws.stream_order.as_ref().unwrap();
    let ch = ws.channels.as_ref().unwrap();
    let water = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, wrap, Some(&ws.rainfall)).classification;
    let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, ws.sea_level, wrap);
    let tree = cartalith_hydrology::flow_receivers(&route, gw, gh, wrap);
    let down: Vec<i32> = (0..n).map(|i| if ch.recv[i] >= 0 { ch.recv[i] } else { tree[i] }).collect();
    let rivers = river_entities(order, &ch.recv, &ws.flow_discharge, &ws.field, gw, gh, 1, river_flow_thresh(gw, gh, gw, km), river_width_scale_k(km), wrap);
    GridWorld { ws, rivers, water, down, gw, gh }
}

impl GridWorld {
    fn plan(&self, comb: Option<(f64, f64)>) -> RiverDrawPlan {
        let (ws, g) = (&self.ws, self);
        match comb {
            None => river_draw_plan_unthinned(&g.rivers, &ws.flow_discharge, &ws.field, ws.sea_level, g.gw, g.gh, &g.down, &g.water),
            Some(c) => cartalith_hydrology::river_draw_plan_with_comb(&g.rivers, &ws.flow_discharge, &ws.field, ws.sea_level, g.gw, g.gh, &g.down, &g.water, c),
        }
    }
}

/// Ground, in cells, within which two drawn one-cell strips count as
/// "parallel neighbours" for [`parallel_streaks`]. A **labelled judgement**,
/// fixed (independent of the comb spacing being tested) so every grid row is
/// judged by the same yardstick: 8 cells is a few strokes' width at the
/// owner's 2048 grid, near the hug reach (3) times a little over two.
const STREAK_NEIGHBOUR_CELLS: f64 = 8.0;

/// **Parallel streaks left** in a plan: drawn order-1 runs that are straight
/// strips ([`is_straight_strip`]) and have another drawn straight order-1
/// strip with at least [`cartalith_hydrology::PARALLEL_MIN_CELLS`] cells -- and
/// at least half of this run's cells -- within [`STREAK_NEIGHBOUR_CELLS`].
/// Counted over the WHOLE map (inland streaks included, since the owner's
/// complaint is not coastal only). Returns `(straight order-1 drawn, of which
/// with a parallel neighbour)`.
fn parallel_streaks(g: &GridWorld, plan: &RiverDrawPlan) -> (usize, usize) {
    let (gw, gh) = (g.gw, g.gh);
    let cell = |p: (f64, f64)| (p.1 as usize).min(gh - 1) * gw + (p.0 as usize).min(gw - 1);
    let strips: Vec<usize> =
        (0..g.rivers.len()).filter(|&i| plan.parallel_of[i].is_none() && g.rivers[i].order == 1 && is_straight_strip(&g.rivers[i])).collect();
    let mut at = vec![usize::MAX; gw * gh];
    for &i in &strips {
        for &p in &g.rivers[i].pts {
            let c = cell(p);
            if at[c] == usize::MAX {
                at[c] = i;
            }
        }
    }
    let k = STREAK_NEIGHBOUR_CELLS as i64;
    let mut with = 0;
    for &i in &strips {
        let cells: Vec<usize> = g.rivers[i].pts.iter().map(|&p| cell(p)).collect();
        let mut cnt: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for &c in &cells {
            let (x, y) = ((c % gw) as i64, (c / gw) as i64);
            let mut seen: Vec<usize> = Vec::new();
            for dy in -k..=k {
                for dx in -k..=k {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 || ((dx * dx + dy * dy) as f64).sqrt() > STREAK_NEIGHBOUR_CELLS {
                        continue;
                    }
                    let o = at[ny as usize * gw + nx as usize];
                    if o != usize::MAX && o != i && !seen.contains(&o) {
                        seen.push(o);
                        *cnt.entry(o).or_insert(0) += 1;
                    }
                }
            }
        }
        if cnt.values().any(|&v| v >= cartalith_hydrology::PARALLEL_MIN_CELLS && 2 * v >= cells.len()) {
            with += 1;
        }
    }
    (strips.len(), with)
}

/// The (factor, cap) grid: factors 1.0 (the shipped baseline), 2, 4, 8 crossed
/// with caps 32, 64, 100, 200 cells. `RC_GRID_WORLDS` limits the worlds.
///
/// Per pair and world prints: comb-hidden strands (vs the unthinned plan) and
/// how many of those are NOT straight strips (the ones that look like real
/// small rivers), their length and its share of the unthinned drawn length,
/// the connectivity counts (`ba_connectivity` against the shipped
/// plan), order >= 3 and trunk length moves, and [`parallel_streaks`].
#[test]
#[ignore = "comb (factor, cap) grid at 2048x1311; run alone in release"]
fn river_comb_grid() {
    let worlds: [(i32, usize, f64); 4] = [(483_920, 2048, 800.0), (24_601, 2048, 800.0), (71_077_345, 2048, 800.0), (246_371, 512, 40_075.0)];
    let only = env_list::<i32>("RC_GRID_SEEDS", &[]);
    for (seed, gw, km) in worlds {
        if !only.is_empty() && !only.contains(&seed) {
            continue;
        }
        let g = grid_world(seed, gw, km);
        let (gw, gh) = (g.gw, g.gh);
        let raw = g.plan(None);
        let base = g.plan(Some((cartalith_hydrology::COMB_SPACING_FACTOR, cartalith_hydrology::COMB_MAX_SPACING_CELLS)));
        let raw_len: f64 = (0..g.rivers.len()).filter(|&i| raw.parallel_of[i].is_none()).map(|i| g.rivers[i].length_cells).sum();
        let sa = stats(&g.rivers, &raw, gw, gh, &g.ws.field, g.ws.sea_level, &g.water);
        let (st0, par0) = parallel_streaks(&g, &raw);
        let (stb, parb) = parallel_streaks(&g, &base);
        println!("== seed {seed} {gw}x{gh} {km} km: runs {} unthinned drawn {} ; streaks (straight order-1 drawn / with parallel neighbour within {} cells): unthinned {st0}/{par0}, shipped constants {stb}/{parb}", g.rivers.len(), raw.parallel_of.iter().filter(|p| p.is_none()).count(), STREAK_NEIGHBOUR_CELLS);
        for &f in &[1.0f64, 2.0, 4.0, 8.0] {
            for &c in &[32.0f64, 64.0, 100.0, 200.0] {
                let p = g.plan(Some((f, c)));
                let hid: Vec<usize> = (0..g.rivers.len()).filter(|&i| p.parallel_of[i].is_some() && raw.parallel_of[i].is_none()).collect();
                let hid_len: f64 = hid.iter().map(|&i| g.rivers[i].length_cells).sum();
                let non_strip = hid.iter().filter(|&&i| !is_straight_strip(&g.rivers[i])).count();
                let max_len = hid.iter().map(|&i| g.rivers[i].length_cells).fold(0.0, f64::max);
                let (dang, lost, orphan, _lb, la) = ba_connectivity(&g.rivers, &raw, &p, gw, gh, &g.ws.field, g.ws.sea_level, &g.water);
                let (_, _, _, _, la_raw) = ba_connectivity(&g.rivers, &raw, &raw, gw, gh, &g.ws.field, g.ws.sea_level, &g.water);
                let sb = stats(&g.rivers, &p, gw, gh, &g.ws.field, g.ws.sea_level, &g.water);
                let (st, par) = parallel_streaks(&g, &p);
                println!(
                    "  f{f:.0} c{c:.0}: removed {} (non-strip {non_strip}, longest {max_len:.0}) len {hid_len:.0} ({:.2} % of unthinned) | lost {lost} orphan {orphan} dangling {dang} loose {la_raw}->{la} | ord3 {:+.0} trunk {:+.0} fan {} | streaks {st} with-neighbour {par}",
                    hid.len(),
                    100.0 * hid_len / raw_len,
                    sb.len_order3 - sa.len_order3,
                    sb.len_trunks - sa.len_trunks,
                    sb.fan_mouths as i64 - sa.fan_mouths as i64
                );
            }
        }
    }
}

/// Pictures for chosen pairs on the owner's world: `RC_PAIRS="f:c,f:c,..."`.
/// Left = the shipped plan, right = the pair, strands the pair hides
/// beyond the shipped plan in red; the crop is the same dense crop as
/// `river_comb_before_after`'s. Output `comb_246371_f<f>_c<c>.png` in `RC_OUT`.
#[test]
#[ignore = "pair pictures at the owner's world; run alone in release"]
fn river_comb_pair_pictures() {
    let out = std::path::PathBuf::from(std::env::var("RC_OUT").unwrap_or_else(|_| ".".into()));
    std::fs::create_dir_all(&out).unwrap();
    let pairs = std::env::var("RC_PAIRS").expect("RC_PAIRS=f:c,f:c");
    let g = grid_world(246_371, 512, 40_075.0);
    let (gw, gh) = (g.gw, g.gh);
    let bw = g.plan(Some(cartalith_hydrology::COMB_RULING_BW));
    let base = g.plan(Some((cartalith_hydrology::COMB_SPACING_FACTOR, cartalith_hydrology::COMB_MAX_SPACING_CELLS)));
    let cand = |i: usize| bw.parallel_of[i].is_none() && g.rivers[i].order == 1 && shore(g.rivers[i].mouth as usize, gw, gh, &g.ws.field, g.ws.sea_level, &g.water);
    let (cw, chh) = (400usize.min(gw), 300usize.min(gh));
    let (_, x0, y0) = densest_crop(&g.rivers, &cand, gw, gh, cw, chh);
    for pair in pairs.split(',') {
        let (f, c) = pair.split_once(':').expect("f:c");
        let (f, c): (f64, f64) = (f.trim().parse().unwrap(), c.trim().parse().unwrap());
        let p = g.plan(Some((f, c)));
        let red: Vec<bool> = (0..g.rivers.len()).map(|i| base.parallel_of[i].is_none() && p.parallel_of[i].is_some()).collect();
        let none = vec![false; g.rivers.len()];
        let l = render_panel(&g.rivers, &base, &none, &g.ws.field, g.ws.sea_level, gw, (x0, y0, cw, chh), 2);
        let r = render_panel(&g.rivers, &p, &red, &g.ws.field, g.ws.sea_level, gw, (x0, y0, cw, chh), 2);
        let (pw, ph, gap) = (cw * 2, chh * 2, 8);
        let mut img = vec![255u8; (pw * 2 + gap) * ph * 3];
        for y in 0..ph {
            let o = y * (pw * 2 + gap) * 3;
            img[o..o + pw * 3].copy_from_slice(&l[y * pw * 3..(y + 1) * pw * 3]);
            let o2 = o + (pw + gap) * 3;
            img[o2..o2 + pw * 3].copy_from_slice(&r[y * pw * 3..(y + 1) * pw * 3]);
        }
        let path = out.join(format!("comb_246371_f{f:.0}_c{c:.0}.png"));
        write_png(&path, pw * 2 + gap, ph, &img);
        println!("pair f{f} c{c}: {} newly hidden vs shipped -> {}", red.iter().filter(|&&b| b).count(), path.display());
    }
}

/// Diagnostic for the owner's world: of the runs DRAWN at the (8.0, 200) comb,
/// why is each not a comb tooth? Counts by the first failing comb guard
/// (order, tributary, discharge, joins another run, outlet not on a shore),
/// split by whether the run is a straight strip, plus drawn-length shares.
#[test]
#[ignore = "diagnostic on the owner's world; run alone in release"]
fn river_comb_why_not_thinned() {
    let g = grid_world(246_371, 512, 40_075.0);
    let (gw, gh) = (g.gw, g.gh);
    let p = g.plan(Some((8.0, 200.0)));
    let cap = COMB_DISCHARGE_FRAC * (gw * gh) as f32;
    let cell = |q: (f64, f64)| (q.1 as usize).min(gh - 1) * gw + (q.0 as usize).min(gw - 1);
    let mut owner = vec![usize::MAX; gw * gh];
    for (i, r) in g.rivers.iter().enumerate() {
        for &q in &r.pts {
            let c = cell(q);
            if owner[c] == usize::MAX {
                owner[c] = i;
            }
        }
    }
    let mut tab: std::collections::BTreeMap<String, (usize, f64)> = Default::default();
    for (i, r) in g.rivers.iter().enumerate() {
        if p.parallel_of[i].is_some() {
            continue;
        }
        let why = if r.order > 1 {
            if r.tributaries > 0 {
                "order>=2 with tributaries"
            } else if r.discharge >= cap {
                "order>=2 no tributaries, heavy"
            } else if owner[r.mouth as usize] != i {
                "order>=2 no tributaries, joins another run"
            } else if !shore(r.mouth as usize, gw, gh, &g.ws.field, g.ws.sea_level, &g.water) {
                "order>=2 no tributaries, inland outlet"
            } else {
                "order>=2 no tributaries, coastal, light (would be a tooth if the order cap allowed)"
            }
        } else if r.tributaries > 0 {
            "has tributary"
        } else if r.discharge >= cap {
            "heavy"
        } else if owner[r.mouth as usize] != i {
            "joins another run (tributary)"
        } else if !shore(r.mouth as usize, gw, gh, &g.ws.field, g.ws.sea_level, &g.water) {
            "outlet not on a shore (inland end)"
        } else {
            "ELIGIBLE but kept (no heavier kept run in reach)"
        };
        let e = tab.entry(format!("{why} | straight={}", is_straight_strip(r))).or_insert((0, 0.0));
        e.0 += 1;
        e.1 += r.length_cells;
    }
    for (k, (n, l)) in tab {
        println!("  {n:5} runs {l:8.0} cells  {k}");
    }
}
