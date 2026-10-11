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
