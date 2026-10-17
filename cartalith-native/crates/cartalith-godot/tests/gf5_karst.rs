//! GF-5's measurement (`GEOLOGY_FIRST_SCOPE.md` §7 GF-5, bars B5 and B8 of
//! §5.2): does karst dissolution act only on soluble rock, does
//! permeability-reduced runoff leave hydrology unregressed, how does it move
//! river cells per rock class, and is the spring line non-empty?
//!
//! # What is measured, and on what
//!
//! Every world is `params::defaults()` at 800 km with `geology_processes` on
//! and `relief_model` off (construction stays off under Ruling CG, as GF-4's
//! bars were measured). **Control:** `geology_karst` off, which is the
//! pre-GF-5 processes-on world by control flow. **Treatment:** the same with
//! `geology_karst` on.
//!
//! - **B5, exact half**: the dissolution pass replayed alone
//!   (`cartalith_erosion::karst::karst_dissolution`, the engine's `k_k` and
//!   τ = 1) on the control world's final surface, column and rainfall. The
//!   pass's own input surface (after the threshold hillslope, inside
//!   `generate_terrain`) is not observable from a `WorldState`, so this is
//!   the kernel on a real world's rock and rain, as GF-4 measured B6 and B7.
//!   Every lowered cell must expose soluble rock; asserted, not just printed.
//! - **B5, depressions**: on the same replay, cells whose depression depth
//!   (priority-flood fill minus surface, `build_routing_surface`) exceeds the
//!   lake classifier's 0.004 after the pass and did not before; the share
//!   whose exposed rock was soluble must be at least 95 %. Fewer than 100
//!   such cells is "not measurable", never a pass (§5.1).
//! - **B8**: RV-1's three guards, treatment against control on generated
//!   worlds (the GF-4 harness's `b8` walk, copied).
//! - **River cells per rock class** (GF-5's "done means"): `river_mask`
//!   cells per exposed rock on each world, control -> treatment.
//! - **Spring line**: `spring_line_potential` cells on the treatment world.
//!
//! Run (release, alone; minutes at 2048 x 1311):
//! `cargo test -j 2 --release -p cartalith-godot --test gf5_karst -- --ignored --nocapture --test-threads 1`
//! `GF5_GW` overrides the grid width, `GF5_SEEDS` the seeds (comma-separated).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_erosion::karst::{karst_dissolution, spring_line_potential, KarstRock, KARST_DEPTH_M};
use cartalith_terrain::geology::{m_to_norm, GeologyColumn, Rock, R_EXPOSE_M};

/// `GEOLOGY_FIRST_SCOPE.md` §5.1's five seeds (the GF-0 harness's `SEEDS`).
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
/// §5.2: the bars apply at 800 km.
const KM: f64 = 800.0;
/// A population under this is "not measurable on this seed" (§5.1).
const MIN_POP: usize = 100;
/// §5.2 B5: the lake classifier's depression threshold, normalised units.
const LAKE_DEPTH: f64 = 0.004;

fn env_list<T: std::str::FromStr + Clone>(key: &str, default: &[T]) -> Vec<T> {
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse `{s}`"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// The app's grid: 2048 x round(2048 x 0.64) = 1311 (`reference_grid_h`).
fn grid() -> (usize, usize) {
    let gw = std::env::var("GF5_GW").map(|v| v.parse::<usize>().expect("GF5_GW")).unwrap_or(2048);
    (gw, ((gw as f64 * 0.64).round() as usize).max(4))
}

/// `params::defaults()` at a seed and grid, 800 km, processes on, relief off.
fn processes_on(seed: i32, gw: usize, gh: usize) -> WorldParams {
    let mut p = params::defaults();
    assert!(p.geology_model && !p.geology_processes && !p.relief_model && !p.geology_karst, "the app ships the model on, the rest off");
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed;
    p.map_width_km = KM;
    p.geology_processes = true;
    p
}

fn exposed(col: &GeologyColumn, field: &[f32], r_expose: f32) -> Vec<Rock> {
    (0..field.len()).map(|i| col.exposed(i, field[i], r_expose).expect("every column cell holds a rock")).collect()
}

/// RV-1's three B8 guards (the GF-4 harness's `b8`, the same walk):
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

/// River cells (`river_mask != 0`) per exposed rock on a world.
fn river_cells_by_rock(ws: &WorldState, r_expose: f32) -> [usize; 11] {
    let col = ws.geology.column().expect("processes-on world carries the column");
    let rock = exposed(col, &ws.field, r_expose);
    let mut out = [0usize; 11];
    if let Some(m) = ws.river_mask.as_ref() {
        for (i, &v) in m.iter().enumerate() {
            if v != 0 {
                out[rock[i] as usize] += 1;
            }
        }
    }
    out
}

/// B5, B8, river cells per rock and the spring line on every seed at 800 km.
/// Prints, with a verdict per bar; asserts B5's exact half, non-empty
/// populations, and that the switch moved the world.
#[test]
#[ignore = "GF-5 measurement: minutes at 2048x1311; run alone in release (module doc)"]
fn gf5_bars() {
    let (gw, gh) = grid();
    let seeds = env_list("GF5_SEEDS", &SEEDS);
    println!("GF-5 measurement at {KM} km, grid {gw}x{gh}; processes on, relief_model off (Ruling CG); k_k {KARST_DEPTH_M} m, phi {}", cartalith_erosion::karst::PERMEABLE_RUNOFF_LOSS);
    let mut springs_any = false;
    for &seed in &seeds {
        let p = processes_on(seed, gw, gh);
        let world = p.world;
        let ctrl = generate_terrain(&p);
        let treat = generate_terrain(&WorldParams { geology_karst: true, ..p.clone() });
        let sea = ctrl.sea_level;
        assert_eq!(sea, treat.sea_level, "seed {seed}: sea level is set before the processes");
        let mpu = p.peak_m / (1.0 - sea);
        let r_expose = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        println!("\n==== seed {seed} (sea {sea:.3}) ====");

        // ---- B5, the pass replayed on the control world ----
        let col = ctrl.geology.column().expect("column").clone();
        let rock = KarstRock { column: &col, r_expose, sea };
        let before = ctrl.field.to_vec();
        let mut after = before.clone();
        let depth = m_to_norm(KARST_DEPTH_M, sea, p.peak_m);
        let n_low = karst_dissolution(&mut after, &ctrl.rainfall, depth, &rock);
        let rock_in = exposed(&col, &before, r_expose);
        let lowered: Vec<usize> = (0..gw * gh).filter(|&i| after[i] < before[i]).collect();
        assert_eq!(lowered.len(), n_low, "seed {seed}: the pass's own count");
        let insoluble = lowered.iter().filter(|&&i| !rock_in[i].props().soluble).count();
        let land = (0..gw * gh).filter(|&i| before[i] as f64 > sea).count();
        let lime = (0..gw * gh).filter(|&i| before[i] as f64 > sea && rock_in[i] == Rock::Limestone).count();
        let mean_m = if lowered.is_empty() { f64::NAN } else { lowered.iter().map(|&i| (before[i] - after[i]) as f64 * mpu).sum::<f64>() / lowered.len() as f64 };
        let max_m = lowered.iter().map(|&i| (before[i] - after[i]) as f64 * mpu).fold(0.0, f64::max);
        println!(
            "B5 replay: land {land}, exposed limestone {lime}, lowered {} (mean {mean_m:.2} m, max {max_m:.2} m), lowered on insoluble rock {insoluble}  => exact half {}",
            lowered.len(),
            if insoluble == 0 { "PASS" } else { "FAIL" }
        );
        assert_eq!(insoluble, 0, "seed {seed}: B5 exact half: a lowered cell is insoluble");
        let fill0 = cartalith_hydrology::build_routing_surface(&before, gw, gh, sea, world);
        let fill1 = cartalith_hydrology::build_routing_surface(&after, gw, gh, sea, world);
        let new_dep: Vec<usize> = (0..gw * gh)
            .filter(|&i| (fill1[i] as f64 - after[i] as f64) > LAKE_DEPTH && (fill0[i] as f64 - before[i] as f64) <= LAKE_DEPTH)
            .collect();
        let dep_sol = new_dep.iter().filter(|&&i| rock_in[i].props().soluble).count();
        if new_dep.len() < MIN_POP {
            println!("B5 depressions: {} new cells deeper than {LAKE_DEPTH} ({dep_sol} soluble)  => not measurable on this seed (< {MIN_POP})", new_dep.len());
        } else {
            let share = dep_sol as f64 / new_dep.len() as f64;
            println!("B5 depressions: {} new cells deeper than {LAKE_DEPTH}, {:.2} % soluble  => {}", new_dep.len(), 100.0 * share, if share >= 0.95 { "PASS" } else { "FAIL" });
        }

        // ---- B8, treatment against control ----
        let ca = cartalith_civ::build_water_bodies(&ctrl.field, gw, gh, sea, world, Some(&ctrl.rainfall)).classification;
        let cf = cartalith_civ::build_water_bodies(&treat.field, gw, gh, sea, world, Some(&treat.rainfall)).classification;
        let (o0, l0, s0, n0) = b8(&ctrl, &ca, gw, gh);
        let (o1, l1, s1, n1) = b8(&treat, &cf, gw, gh);
        assert!(n0 > 0 && n1 > 0, "seed {seed}: B8 walked no river cells");
        let moved = ctrl.field.iter().zip(treat.field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
        assert!(moved > 0, "seed {seed}: the karst switch moved nothing");
        println!(
            "B8 karst off -> on: ocean on paths {o0} -> {o1}; lake % of path {:.3} ({n0}) -> {:.3} ({n1}); 1-3-cell lakes {s0} -> {s1}; field cells moved {moved}",
            l0.unwrap_or(f64::NAN),
            l1.unwrap_or(f64::NAN)
        );
        let ok8 = o1 == 0 && l1.zip(l0).is_some_and(|(b, a)| b <= a + 2.0) && (s1 as f64) <= 1.25 * s0 as f64;
        println!("   bar (ocean 0; lake % <= control + 2 pp; small lakes <= 1.25 x control) => {}", if ok8 { "PASS" } else { "FAIL" });

        // ---- River cells per exposed rock ----
        let r0 = river_cells_by_rock(&ctrl, r_expose);
        let r1 = river_cells_by_rock(&treat, r_expose);
        let mut line = String::from("river cells by exposed rock, off -> on:");
        for r in Rock::ALL {
            let k = r as usize;
            if r0[k] + r1[k] > 0 {
                line.push_str(&format!(" {} {} -> {} ({:+.1} %);", r.props().name, r0[k], r1[k], 100.0 * (r1[k] as f64 - r0[k] as f64) / (r0[k].max(1)) as f64));
            }
        }
        println!("{line}");

        // ---- Spring line on the treatment world ----
        let tcol = treat.geology.column().expect("column");
        let springs = spring_line_potential(&treat.field, gw, gh, world, &KarstRock { column: tcol, r_expose, sea });
        let n_sp = springs.iter().filter(|&&b| b).count();
        let two_layer_land = (0..gw * gh).filter(|&i| treat.field[i] as f64 > sea && tcol.substrate(i).is_some()).count();
        springs_any |= n_sp > 0;
        println!("spring line: {n_sp} cells (two-layer land {two_layer_land})");
    }
    println!("\nspring line non-empty on at least one seed: {}", if springs_any { "PASS" } else { "FAIL" });
}

/// The switch-off identity, printed: FNV-1a hashes of thirteen arrays for the
/// parity world, the app world and the processes-on world on two seeds at
/// 512 x 328. Run on HEAD and on the change; the lines must match.
#[test]
#[ignore = "GF-5 identity hashes: compare the printout across two builds"]
fn gf5_identity_hashes() {
    fn fnv(bytes: impl Iterator<Item = u8>) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
    fn f32s(v: &[f32]) -> u64 {
        fnv(v.iter().flat_map(|x| x.to_bits().to_le_bytes()))
    }
    let (gw, gh) = (512usize, 328usize);
    for seed in [483_920, 314_159] {
        let mut app = params::defaults();
        app.gw = gw;
        app.gh = gh;
        app.tect.seed = seed;
        let parity = WorldParams::defaults(gw, gh, seed);
        let on = WorldParams { geology_processes: true, ..app.clone() };
        for (label, p) in [("parity", parity), ("app", app), ("processes-on", on)] {
            let ws = generate_terrain(&p);
            let ch = ws.channels.as_ref();
            let col = ws.geology.column();
            let hs = [
                f32s(&ws.field),
                f32s(&ws.rainfall),
                f32s(&ws.temperature),
                f32s(&ws.flow_discharge),
                ws.stream_order.as_ref().map_or(0, |v| fnv(v.iter().flat_map(|x| x.to_le_bytes()))),
                ws.river_mask.as_ref().map_or(0, |v| fnv(v.iter().copied())),
                ws.river_floor.as_ref().map_or(0, |v| f32s(v)),
                ch.map_or(0, |c| fnv(c.chan.iter().flat_map(|x| x.to_le_bytes()))),
                ch.map_or(0, |c| fnv(c.recv.iter().flat_map(|x| x.to_le_bytes()))),
                ch.map_or(0, |c| f32s(&c.intensity)),
                col.map_or(0, |c| f32s(&c.regolith)),
                col.map_or(0, |c| f32s(&c.contact)),
                col.map_or(0, |c| fnv(c.rock_top.iter().copied())),
            ];
            println!("HASH seed {seed} {label}: {}", hs.iter().map(|h| format!("{h:016x}")).collect::<Vec<_>>().join(" "));
        }
    }
}
