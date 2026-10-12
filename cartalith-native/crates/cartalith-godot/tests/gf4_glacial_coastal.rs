//! GF-4's measurement (`GEOLOGY_FIRST_SCOPE.md` §7 GF-4, bars B6, B7, B8 and
//! B9 of §5.2): do the rock-reading glacial and coastal kernels
//! (`cartalith_erosion::glacial_kernel_rock`, `coastal_process_rock`) make
//! glacial lowering and cliff retreat track rock on real worlds, and what
//! would turning the coastal pass on in the app (Ruling BJ, §4.5) cost?
//!
//! # What is measured, and on what
//!
//! **B6 and B7 are the kernels alone.** `generate_terrain` runs glacial
//! rock-blind after the trace (its rock kernel is not wired, see
//! `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4 measured"); since GF-4 the coastal pass
//! runs ahead of the trace and reads rock with `geology_processes` on, where
//! its input surface is not observable from a `WorldState`. So B6 and B7 are
//! measured on the kernels **alone**, replayed on the inputs the pipeline
//! handed its legacy passes after the trace on a processes-on world
//! (`geology_processes` on, `relief_model` off: construction stays off under
//! Ruling CG, so GF-4's bars are measured without it, as the GF-13
//! assessment says):
//!
//! - **The input surface** is the same seed with `passes.glacial = false`.
//!   Glacial is the only pass `params::defaults()` turns on, so with it off
//!   `passes.any()` is false and that world's field, temperature, discharge
//!   and column are exactly what the passes block receives (the GF-0
//!   harness's argument, `gf0_geology_harness.rs`, module doc).
//! - **Control arm:** each rock kernel at contrast `c = 0`, asserted bit for
//!   bit equal to the legacy kernel, and the legacy glacial chain (glacial,
//!   strip, rebound, clamp) asserted to reproduce the generated world bit for
//!   bit -- so the control is the pass the app runs, not a stand-in. (The
//!   coastal chain was asserted the same way before the pass moved ahead of
//!   the trace; see the B7 block.)
//! - **Treatment arm:** the same kernels at the app's `c = tect.resist`
//!   (0.50, `WorldParams::defaults`; §9 Q5 repurposes it as `c`).
//!
//! These are the kernels' rock response on the post-trace inputs, not a
//! measurement of a moved stage. B8 is measured on generated worlds, with the
//! coastal pass where GF-4 put it.
//!
//! - **B6**, as §5.2 and §5.4 define it: ice cells are land at or above the
//!   snowline and below 0 °C (the kernel's own gate); discharge is GF-0's
//!   stand-in (unit upstream area over the depression-filled surface); ten
//!   equal-count discharge bands; per band the median lowering on weak
//!   (γ ≥ 1.5) over strong (γ ≤ 0.6) exposed rock, read on the input
//!   surface; the reading is the median of the per-band ratios over bands
//!   holding at least 10 cells of each group with a positive strong median.
//!   The lowering is the kernel's, before the rebound.
//! - **B7**: land cells lost to the sea per coastline cell, weak (ρ ≥ 1.5)
//!   over strong (ρ ≤ 0.3) exposed rock, over the default passes. The
//!   coastline is land 4-adjacent to ocean (`build_water_bodies` class 1) on
//!   the coastal pass's input surface.
//! - **B8**: RV-1's three guards on a world with the coastal pass on
//!   against the same world without it, twice: the app world (the
//!   processes off: Ruling BJ's flip as it would ship today, rock-blind)
//!   and the processes-on world (the pass reading rock).
//! - **B9** for the same flip: `gf4_coastal_cost`, run alone.
//!
//! Run (release, alone; minutes at 2048 x 1311):
//! `cargo test -j 2 --release -p cartalith-godot --test gf4_glacial_coastal -- --ignored --nocapture --test-threads 1`
//! `GF4_GW` overrides the grid width, `GF4_SEEDS` the seeds (comma-separated).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_terrain::geology::{m_to_norm, GeologyColumn, Rock, R_EXPOSE_M};

/// `GEOLOGY_FIRST_SCOPE.md` §5.1's five seeds (the GF-0 harness's `SEEDS`).
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
/// §5.2: the bars apply at 800 km.
const KM: f64 = 800.0;
/// A population under this is "not measurable on this seed" (§5.1).
const MIN_POP: usize = 100;
/// B6's weak and strong groups (§5.2: γ ≥ 1.5 and γ ≤ 0.6).
const B6_WEAK_GAMMA: f32 = 1.5;
const B6_STRONG_GAMMA: f32 = 0.6;
/// B7's weak and strong groups (§5.2: ρ ≥ 1.5 and ρ ≤ 0.3).
const B7_WEAK_RHO: f32 = 1.5;
const B7_STRONG_RHO: f32 = 0.3;

fn env_list<T: std::str::FromStr>(key: &str, default: &[T]) -> Vec<T>
where
    T: Clone,
{
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse `{s}`"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// The app's grid: 2048 x round(2048 x 0.64) = 1311 (`reference_grid_h`).
fn grid() -> (usize, usize) {
    let gw = std::env::var("GF4_GW").map(|v| v.parse::<usize>().expect("GF4_GW")).unwrap_or(2048);
    (gw, ((gw as f64 * 0.64).round() as usize).max(4))
}

/// `params::defaults()` at a seed and grid, 800 km.
fn app_params(seed: i32, gw: usize, gh: usize) -> WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = KM;
    p
}

fn quantile(v: &[f64], q: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let pos = q * (s.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    Some(s[lo] + (s[hi] - s[lo]) * (pos - lo as f64))
}

fn median(v: &[f64]) -> Option<f64> {
    quantile(v, 0.5)
}

/// The exposed rock on `field` per cell (§2.5).
fn exposed(col: &GeologyColumn, field: &[f32], r_expose: f32) -> Vec<Rock> {
    (0..field.len()).map(|i| col.exposed(i, field[i], r_expose).expect("every column cell holds a rock")).collect()
}

/// B6 on one arm: `(reading or reason, bands used, weak cells, strong cells)`.
fn b6(gamma: &[f32], lowering: &[f64], q: &[f32], ice: &[usize]) -> (Result<f64, String>, usize, usize, usize) {
    if ice.len() < MIN_POP {
        return (Err(format!("ice cells {} < {MIN_POP}", ice.len())), 0, 0, 0);
    }
    let mut ranked = ice.to_vec();
    ranked.sort_by(|&a, &b| q[a].total_cmp(&q[b]).then(a.cmp(&b)));
    let (mut ratios, mut nw, mut ns) = (Vec::new(), 0usize, 0usize);
    for d in 0..10 {
        let band = &ranked[d * ranked.len() / 10..(d + 1) * ranked.len() / 10];
        let weak: Vec<f64> = band.iter().filter(|&&i| gamma[i] >= B6_WEAK_GAMMA).map(|&i| lowering[i]).collect();
        let strong: Vec<f64> = band.iter().filter(|&&i| gamma[i] <= B6_STRONG_GAMMA).map(|&i| lowering[i]).collect();
        if weak.len() < 10 || strong.len() < 10 {
            continue;
        }
        let (mw, ms) = (median(&weak).unwrap(), median(&strong).unwrap());
        if ms <= 0.0 {
            continue;
        }
        nw += weak.len();
        ns += strong.len();
        ratios.push(mw / ms);
    }
    let used = ratios.len();
    match median(&ratios) {
        Some(r) if nw >= MIN_POP && ns >= MIN_POP => (Ok(r), used, nw, ns),
        Some(_) => (Err(format!("weak {nw} or strong {ns} in the used bands < {MIN_POP}")), used, nw, ns),
        None => (Err("no band had >= 10 weak and >= 10 strong cells with a positive strong median".into()), 0, nw, ns),
    }
}

/// B7 on one arm: `(ratio or reason, coast weak, coast strong, lost weak, lost strong)`.
fn b7(rho: &[f32], coast: &[usize], lost: &[usize]) -> (Result<f64, String>, usize, usize, usize, usize) {
    let cw = coast.iter().filter(|&&i| rho[i] >= B7_WEAK_RHO).count();
    let cs = coast.iter().filter(|&&i| rho[i] <= B7_STRONG_RHO).count();
    let lw = lost.iter().filter(|&&i| rho[i] >= B7_WEAK_RHO).count();
    let ls = lost.iter().filter(|&&i| rho[i] <= B7_STRONG_RHO).count();
    let r = if cw < MIN_POP || cs < MIN_POP {
        Err(format!("a coastline group is under {MIN_POP}"))
    } else if ls == 0 {
        Err("no strong-rock land lost, so the ratio is unbounded".into())
    } else {
        Ok((lw as f64 / cw as f64) / (ls as f64 / cs as f64))
    };
    (r, cw, cs, lw, ls)
}

/// RV-1's three B8 guards (the GF-0 harness's `b8`, the same walk):
/// `(ocean cells on river paths, % of path cells that are lake, 1-3-cell lakes, path cells)`.
fn b8(ws: &WorldState, class: &[u8], gw: usize, gh: usize) -> (usize, Option<f64>, usize, usize) {
    let (Some(order), Some(ch)) = (ws.stream_order.as_ref(), ws.channels.as_ref()) else {
        return (0, None, 0, 0);
    };
    let rivers = cartalith_hydrology::river_entities(
        order,
        &ch.recv,
        &ws.flow_discharge,
        &ws.field,
        gw,
        gh,
        1,
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, KM),
        cartalith_hydrology::river_width_scale_k(KM),
        false,
    );
    let (mut cells, mut ocean, mut lake) = (0usize, 0usize, 0usize);
    for r in &rivers {
        for &(x, y) in r.pts.iter().take(r.pts.len().saturating_sub(1)) {
            let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
            cells += 1;
            match class[i] {
                1 => ocean += 1,
                2 => lake += 1,
                _ => {}
            }
        }
    }
    let mut seen = vec![false; gw * gh];
    let (mut small, mut stack) = (0usize, Vec::new());
    for s0 in 0..gw * gh {
        if class[s0] != 2 || seen[s0] {
            continue;
        }
        seen[s0] = true;
        stack.push(s0);
        let mut k = 0usize;
        while let Some(i) = stack.pop() {
            k += 1;
            let (x, y) = (i % gw, i / gw);
            for j in [
                (x > 0).then(|| i - 1),
                (x + 1 < gw).then(|| i + 1),
                (y > 0).then(|| i - gw),
                (y + 1 < gh).then(|| i + gw),
            ]
            .into_iter()
            .flatten()
            {
                if class[j] == 2 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        if k <= 3 {
            small += 1;
        }
    }
    (ocean, (cells > 0).then(|| 100.0 * lake as f64 / cells as f64), small, cells)
}

/// The engine's two-statement clamp after the passes block.
#[allow(clippy::manual_clamp)]
fn clamp01(f: &mut [f32]) {
    for v in f.iter_mut() {
        if *v < 0.0 {
            *v = 0.0;
        } else if *v > 1.0 {
            *v = 1.0;
        }
    }
}

fn show(r: &Result<f64, String>) -> String {
    match r {
        Ok(v) => format!("{v:.4}"),
        Err(e) => format!("-- ({e})"),
    }
}

/// B6, B7 and B8 on every seed at 800 km. Prints, with a verdict per bar;
/// asserts only its own validity: that the `c = 0` kernels are the legacy
/// kernels bit for bit, that the legacy chain replayed from the
/// glacial-off world reproduces the generated processes-on world (with and
/// without the coastal pass) bit for bit, and that the populations it reads
/// are non-empty.
#[test]
#[ignore = "GF-4 measurement: minutes at 2048x1311; run alone in release (module doc)"]
fn gf4_bars() {
    let (gw, gh) = grid();
    let seeds = env_list("GF4_SEEDS", &SEEDS);
    println!("GF-4 measurement at {KM} km, grid {gw}x{gh}; processes on, relief_model off (Ruling CG); kernels replayed alone");
    for &seed in &seeds {
        let app = app_params(seed, gw, gh);
        assert!(app.geology_model && !app.geology_processes && !app.relief_model, "the app ships the model on and the processes and relief off");
        let p = WorldParams { geology_processes: true, ..app.clone() };
        let c = p.tect.resist;
        let world = p.world;
        // The passes block's input: the processes-on world with glacial off.
        let pre = generate_terrain(&WorldParams { passes: cartalith_engine::ErosionPassParams { glacial: false, ..p.passes.clone() }, ..p.clone() });
        let sea = pre.sea_level;
        let mpu = p.peak_m / (1.0 - sea);
        let r_expose = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        let col0 = pre.geology.column().expect("processes-on world carries the column").clone();
        let q = &p.passes;
        let gpar = cartalith_erosion::GlacialParams {
            kg: q.glacial_kg,
            mg: q.glacial_mg,
            snowline: q.glacial_snowline,
            u_factor: q.glacial_u_factor,
            passes: q.glacial_passes,
            g: p.planet.g,
            sea,
            world,
        };

        // ---- B6 ----
        let mut legacy = pre.field.to_vec();
        cartalith_erosion::glacial_kernel(&mut legacy, &pre.temperature, gw, gh, &gpar);
        let mut ctrl = pre.field.to_vec();
        cartalith_erosion::glacial_kernel_rock(&mut ctrl, &pre.temperature, gw, gh, &gpar, &cartalith_erosion::PassRock { column: &col0, contrast: 0.0, r_expose });
        assert!(ctrl == legacy, "seed {seed}: the c = 0 glacial kernel is not the legacy kernel");
        let mut treat = pre.field.to_vec();
        cartalith_erosion::glacial_kernel_rock(&mut treat, &pre.temperature, gw, gh, &gpar, &cartalith_erosion::PassRock { column: &col0, contrast: c, r_expose });

        // Validity: the legacy chain is the pipeline's (glacial, strip,
        // rebound, clamp), bit for bit against the generated world.
        let mut col = col0.clone();
        cartalith_erosion::account_regolith(&mut col, &pre.field, &legacy, false);
        let mut after_rb = legacy.clone();
        cartalith_erosion::isostatic_rebound(&mut after_rb, &pre.field, gw, gh, p.tect.blur_r, world);
        cartalith_erosion::lift_column(&mut col, &legacy, &after_rb);
        let mut replay = after_rb.clone();
        clamp01(&mut replay);
        let with_glacial = generate_terrain(&p);
        assert!(replay == *with_glacial.field, "seed {seed}: glacial + strip + rebound + clamp does not reproduce the processes-on world");

        let class0 = cartalith_civ::build_water_bodies(&pre.field, gw, gh, sea, world, Some(&pre.rainfall)).classification;
        let snow_el = sea + (1.0 - sea) * q.glacial_snowline;
        let ice: Vec<usize> =
            (0..gw * gh).filter(|&i| class0[i] == 0 && pre.field[i] as f64 >= snow_el && (pre.temperature[i] as f64) < 0.0).collect();
        let rock_in = exposed(&col0, &pre.field, r_expose);
        let gamma: Vec<f32> = rock_in.iter().map(|r| r.props().gamma).collect();
        let route = cartalith_hydrology::build_routing_surface(&pre.field, gw, gh, sea, world);
        let qd = cartalith_hydrology::compute_flow(gw, gh, &route, None, false, world);
        let low = |after: &[f32]| -> Vec<f64> { (0..gw * gh).map(|i| (pre.field[i] as f64 - after[i] as f64) * mpu).collect() };
        let (b6c, uc, wc, sc) = b6(&gamma, &low(&ctrl), &qd, &ice);
        let (b6t, ut, wt, st) = b6(&gamma, &low(&treat), &qd, &ice);
        println!("\n==== seed {seed} (sea {sea:.3}, c {c}) ====");
        println!("B6 glacial weak/strong  control {} (bands {uc}, weak {wc}, strong {sc})  treatment {} (bands {ut}, weak {wt}, strong {st})  ice cells {}", show(&b6c), show(&b6t), ice.len());
        match (&b6t, &b6c) {
            (Ok(t), Ok(cv)) => println!(
                "   bar: treatment >= 1.5 {}; >= 1.5 x control ({:.4}) {}  => {}",
                *t >= 1.5,
                1.5 * cv,
                *t >= 1.5 * cv,
                if *t >= 1.5 && *t >= 1.5 * cv { "PASS" } else { "FAIL" }
            ),
            _ => println!("   bar: not measurable on this seed"),
        }

        // ---- B7 ----
        // The coastal pass's input at its pre-GF-4 position (after the
        // trace, after glacial + rebound): the field after the rebound
        // (unclamped, as in the pipeline), the discharge the passes block
        // read (the glacial-off world's, which no pass refreshed), and the
        // column after glacial's strip and the rebound's lift. GF-4 moved
        // the pass ahead of the trace, where its input surface is not
        // observable from a `WorldState`; until 2026-10-12's move this
        // replay was asserted bit for bit against the generated coastal
        // world (it held on all five seeds at 2048 x 1311), so these are the
        // kernels' rock response on the inputs the pass then received.
        let cpar = cartalith_erosion::CoastalParams { wave_str: q.wave_str, estuary_depth: q.estuary_depth, marsh_band: q.marsh_band, passes: q.coastal_passes };
        let coastal_pair = |cp: &cartalith_erosion::CoastalParams| -> (Vec<f32>, Vec<f32>) {
            let mut c_legacy = after_rb.clone();
            cartalith_erosion::coastal_process(&mut c_legacy, &pre.flow_discharge, gw, gh, sea, world, p.planet.g, cp);
            let mut c_ctrl = after_rb.clone();
            cartalith_erosion::coastal_process_rock(&mut c_ctrl, &pre.flow_discharge, gw, gh, sea, world, p.planet.g, cp, &cartalith_erosion::PassRock { column: &col, contrast: 0.0, r_expose });
            assert!(c_ctrl == c_legacy, "seed {seed}: the c = 0 coastal kernel is not the legacy kernel");
            let mut c_treat = after_rb.clone();
            cartalith_erosion::coastal_process_rock(&mut c_treat, &pre.flow_discharge, gw, gh, sea, world, p.planet.g, cp, &cartalith_erosion::PassRock { column: &col, contrast: c, r_expose });
            (c_ctrl, c_treat)
        };
        let (c_ctrl, c_treat) = coastal_pair(&cpar);

        let mut in_c = after_rb.clone();
        clamp01(&mut in_c);
        let class1 = cartalith_civ::build_water_bodies(&in_c, gw, gh, sea, world, Some(&pre.rainfall)).classification;
        let coast: Vec<usize> = (0..gw * gh)
            .filter(|&i| {
                if class1[i] != 0 {
                    return false;
                }
                let (x, y) = (i % gw, i / gw);
                (x > 0 && class1[i - 1] == 1) || (x + 1 < gw && class1[i + 1] == 1) || (y > 0 && class1[i - gw] == 1) || (y + 1 < gh && class1[i + gw] == 1)
            })
            .collect();
        let rho: Vec<f32> = exposed(&col, &after_rb, r_expose).iter().map(|r| r.props().rho).collect();
        let lost_of = |after: &[f32]| -> Vec<usize> { (0..gw * gh).filter(|&i| class1[i] == 0 && (after[i] as f64) < sea).collect() };
        let (b7c, cw, cs, lwc, lsc) = b7(&rho, &coast, &lost_of(&c_ctrl));
        let (b7t, _, _, lwt, lst) = b7(&rho, &coast, &lost_of(&c_treat));
        println!(
            "B7 coastal lost/coast weak/strong  control {} (lost weak {lwc}, strong {lsc})  treatment {} (lost weak {lwt}, strong {lst})  coast weak {cw}, strong {cs}, all {}",
            show(&b7c),
            show(&b7t),
            coast.len()
        );
        match (&b7t, &b7c) {
            (Ok(t), Ok(cv)) => println!(
                "   bar: treatment >= 3 {}; >= 2 x control ({:.4}) {}  => {}",
                *t >= 3.0,
                2.0 * cv,
                *t >= 2.0 * cv,
                if *t >= 3.0 && *t >= 2.0 * cv { "PASS" } else { "FAIL" }
            ),
            (Ok(t), Err(e)) => println!("   bar: control not measurable ({e}); treatment >= 3 {}", *t >= 3.0),
            _ => println!("   bar: not measurable on this seed"),
        }
        // Diagnostic, not the bar: the same B7 with the tidal marsh off
        // (`marsh_band` 0) in both arms. The marsh's value-noise bump
        // (±0.55·marsh_band, `coastal_process`) reads no rock under §4.5 and
        // can lower a cell below sea on its own, so this isolates how much
        // of the land loss is the wave and estuary cut that §4.5 scales.
        let (m_ctrl, m_treat) = coastal_pair(&cartalith_erosion::CoastalParams { marsh_band: 0.0, ..cpar });
        let (m7c, _, _, mlwc, mlsc) = b7(&rho, &coast, &lost_of(&m_ctrl));
        let (m7t, _, _, mlwt, mlst) = b7(&rho, &coast, &lost_of(&m_treat));
        println!(
            "   diagnostic, marsh off: control {} (lost weak {mlwc}, strong {mlsc})  treatment {} (lost weak {mlwt}, strong {mlst})",
            show(&m7c),
            show(&m7t)
        );

        // ---- B8, with the coastal pass ahead of the trace (GF-4) ----
        // Two flips, each against its own control: the app world (the
        // processes off, so the pass is rock-blind), which is Ruling BJ's
        // flip as it would ship today; and the processes-on world, where the
        // pass reads rock.
        for (label, base) in [("app world (processes off)", &app), ("processes-on world", &p)] {
            let a_ws = generate_terrain(base);
            let f_ws = generate_terrain(&WorldParams { passes: cartalith_engine::ErosionPassParams { coastal: true, ..base.passes.clone() }, ..base.clone() });
            let ca = cartalith_civ::build_water_bodies(&a_ws.field, gw, gh, a_ws.sea_level, world, Some(&a_ws.rainfall)).classification;
            let cf = cartalith_civ::build_water_bodies(&f_ws.field, gw, gh, f_ws.sea_level, world, Some(&f_ws.rainfall)).classification;
            let (o0, l0, s0, n0) = b8(&a_ws, &ca, gw, gh);
            let (o1, l1, s1, n1) = b8(&f_ws, &cf, gw, gh);
            assert!(n0 > 0 && n1 > 0, "seed {seed}: B8 walked no river cells");
            let land = |c: &[u8]| c.iter().filter(|&&k| k == 0).count();
            let moved = a_ws.field.iter().zip(f_ws.field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
            assert!(moved > 0, "seed {seed}: the coastal pass moved nothing on the {label}");
            println!(
                "B8 {label}, coastal off -> on: ocean on paths {o0} -> {o1}; lake % of path {:.3} ({n0}) -> {:.3} ({n1}); 1-3-cell lakes {s0} -> {s1}; land cells {} -> {}; field cells moved {moved}",
                l0.unwrap_or(f64::NAN),
                l1.unwrap_or(f64::NAN),
                land(&ca),
                land(&cf)
            );
            let ok8 = o1 == 0 && l1.zip(l0).is_some_and(|(b, a)| b <= a + 2.0) && (s1 as f64) <= 1.25 * s0 as f64;
            println!("   bar (ocean 0; lake % <= control + 2 pp; small lakes <= 1.25 x control) => {}", if ok8 { "PASS" } else { "FAIL" });
        }
    }
}

/// B9 for the Ruling BJ flip: median of five whole generations of the app
/// world with the coastal pass off and on, interleaved, at 2048 x 1311, one
/// seed (483920). Prints median with min..max; asserts nothing about the
/// bar. Run alone: a parallel suite moves the number.
#[test]
#[ignore = "GF-4 cost: run alone in release (module doc)"]
fn gf4_coastal_cost() {
    let (gw, gh) = grid();
    let off = app_params(483_920, gw, gh);
    let on = WorldParams { passes: cartalith_engine::ErosionPassParams { coastal: true, ..off.clone().passes }, ..off.clone() };
    let _ = generate_terrain(&off); // warm-up, not timed
    let (mut t0, mut t1) = (Vec::new(), Vec::new());
    for _ in 0..5 {
        let s = std::time::Instant::now();
        let _ = generate_terrain(&off);
        t0.push(s.elapsed().as_secs_f64());
        let s = std::time::Instant::now();
        let _ = generate_terrain(&on);
        t1.push(s.elapsed().as_secs_f64());
    }
    let span = |v: &[f64]| (median(v).unwrap(), v.iter().cloned().fold(f64::INFINITY, f64::min), v.iter().cloned().fold(0.0, f64::max));
    let (m0, a0, b0) = span(&t0);
    let (m1, a1, b1) = span(&t1);
    println!("app world 483920 {gw}x{gh}: coastal off {m0:.3} s ({a0:.3}..{b0:.3}); on {m1:.3} s ({a1:.3}..{b1:.3}); ratio of medians {:.3}", m1 / m0);
}
