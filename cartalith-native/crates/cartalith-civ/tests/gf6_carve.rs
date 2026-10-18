//! GF-6's carve measurement (`GEOLOGY_FIRST_SCOPE.md` §4.7, §7 GF-6): the
//! switch-off identity of the rock-dependent valley width `ω`, and its bars.
//!
//! `gf6_carve_bars` measures, at 800 km on §5.1's five seeds, with
//! `geology_processes` on and `relief_model` off (Ruling CG, as GF-4's and
//! GF-5's bars were measured), control `geology_carve` off (the pre-GF-6
//! processes-on world by control flow) against treatment `geology_carve` on:
//! B8 (RV-1's three guards, the GF-5 harness's walk, copied; §7 GF-6's
//! done-means says B8 still passes); carved cells (`river_mask`) per exposed
//! rock on the control world's surface, pooled over weak (`ω > 1`) and strong
//! (`ω < 1`) rock; and `knickpoints` cells on both worlds (§4.7's waterfall
//! candidates).
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_carve gf6_carve_bars -- --ignored --nocapture --test-threads 1`
//! (`GF6_GW` overrides the grid width, default 2048; `GF6_SEEDS` the seeds).
//!
//! `gf6_carve_identity_hashes` uses only API that predates the switch, so the
//! same file builds against the commit before it: run it on both builds and
//! diff the `HASH` lines (`GEOLOGY_FIRST_SCOPE.md` §7 "GF-6 measured: the
//! carve half" records the result).
//!
//! `cargo test -j 2 --release -p cartalith-civ --test gf6_carve gf6_carve_identity_hashes -- --ignored --nocapture`
#![allow(dead_code)]

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_erosion::carve_width::{knickpoints, valley_width_factor};
use cartalith_terrain::geology::{m_to_norm, Rock, R_EXPOSE_M};

/// GEOLOGY_FIRST_SCOPE.md section 5.1's five seeds (the GF-0 harness's SEEDS).
const SEEDS: [i32; 5] = [483_920, 24_601, 71_077_345, 12_345, 314_159];
/// Section 5.2: the bars apply at 800 km.
const KM: f64 = 800.0;

/// FNV-1a over a byte stream: the GF-5 harness's identity hash, copied.
fn fnv(bytes: impl Iterator<Item = u8>) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// [`fnv`] over an `f32` slice's bit patterns.
fn f32s(v: &[f32]) -> u64 {
    fnv(v.iter().flat_map(|x| x.to_bits().to_le_bytes()))
}

/// The switch-off identity, printed: FNV-1a hashes of thirteen arrays for the
/// parity world, the app world and the processes-on world on two seeds at
/// 512 x 328 (the GF-5 harness's set). Protects the claim that adding the
/// carve switch, off, moves no array of any of the three; it asserts nothing
/// itself, because the comparison is between two builds.
#[test]
#[ignore = "GF-6 carve identity hashes: compare the printout across two builds"]
fn gf6_carve_identity_hashes() {
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

fn env_list<T: std::str::FromStr + Clone>(key: &str, default: &[T]) -> Vec<T> {
    match std::env::var(key) {
        Ok(v) => v.split(',').map(|s| s.trim().parse::<T>().unwrap_or_else(|_| panic!("{key}: cannot parse `{s}`"))).collect(),
        Err(_) => default.to_vec(),
    }
}

/// RV-1's three B8 guards (the GF-5 harness's `b8`, the same walk):
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

/// B8, carved width by rock and knickpoints on every seed at 800 km.
/// Prints, with a verdict for B8; asserts non-empty populations and that the
/// switch moved the world. It is the measurement `GEOLOGY_FIRST_SCOPE.md` §7
/// records, not a regression guard.
#[test]
#[ignore = "GF-6 carve measurement: minutes at 2048x1311; run alone in release (module doc)"]
fn gf6_carve_bars() {
    let gw = std::env::var("GF6_GW").map(|v| v.parse::<usize>().expect("GF6_GW")).unwrap_or(2048);
    let gh = ((gw as f64 * 0.64).round() as usize).max(4);
    let seeds = env_list("GF6_SEEDS", &SEEDS);
    println!("GF-6 carve measurement at {KM} km, grid {gw}x{gh}; processes on, relief_model off (Ruling CG)");
    for &seed in &seeds {
        let mut p = params::defaults();
        assert!(p.geology_model && !p.geology_processes && !p.relief_model && !p.geology_carve, "the app ships the model on, the rest off");
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = KM;
        p.geology_processes = true;
        let world = p.world;
        let ctrl = generate_terrain(&p);
        let treat = generate_terrain(&WorldParams { geology_carve: true, ..p.clone() });
        let sea = ctrl.sea_level;
        let r_expose = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        println!("\n==== seed {seed} (sea {sea:.3}) ====");

        // ---- B8 ----
        let ca = cartalith_civ::build_water_bodies(&ctrl.field, gw, gh, sea, world, Some(&ctrl.rainfall)).classification;
        let cf = cartalith_civ::build_water_bodies(&treat.field, gw, gh, sea, world, Some(&treat.rainfall)).classification;
        let (o0, l0, s0, n0) = b8(&ctrl, &ca, gw, gh);
        let (o1, l1, s1, n1) = b8(&treat, &cf, gw, gh);
        assert!(n0 > 0 && n1 > 0, "seed {seed}: B8 walked no river cells");
        let moved = ctrl.field.iter().zip(treat.field.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
        assert!(moved > 0, "seed {seed}: the carve switch moved nothing");
        println!(
            "B8 carve off -> on: ocean on paths {o0} -> {o1}; lake % of path {:.3} ({n0}) -> {:.3} ({n1}); 1-3-cell lakes {s0} -> {s1}; field cells moved {moved}",
            l0.unwrap_or(f64::NAN),
            l1.unwrap_or(f64::NAN)
        );
        let ok8 = o1 == 0 && l1.zip(l0).is_some_and(|(b, a)| b <= a + 2.0) && (s1 as f64) <= 1.25 * s0 as f64;
        println!("   bar (ocean 0; lake % <= control + 2 pp; small lakes <= 1.25 x control) => {}", if ok8 { "PASS" } else { "FAIL" });

        // ---- Carved width by rock, on the control world's exposed rock ----
        let col = ctrl.geology.column().expect("processes-on world carries the column");
        let (mo, mn) = (ctrl.river_mask.as_ref().unwrap(), treat.river_mask.as_ref().unwrap());
        let mut per = [(0usize, 0usize); 11];
        for i in 0..gw * gh {
            let Some(rock) = col.exposed(i, ctrl.field[i], r_expose) else { continue };
            per[rock as usize].0 += (mo[i] != 0) as usize;
            per[rock as usize].1 += (mn[i] != 0) as usize;
        }
        let (mut wk, mut st) = ((0usize, 0usize), (0usize, 0usize));
        let mut line = String::from("carved cells by exposed rock (omega), off -> on:");
        for r in Rock::ALL {
            let (a, b) = per[r as usize];
            if a + b == 0 {
                continue;
            }
            let w = valley_width_factor(r);
            line.push_str(&format!(" {} ({w:.2}) {a} -> {b};", r.props().name));
            if w > 1.0 + 1e-6 {
                wk = (wk.0 + a, wk.1 + b);
            } else if w < 1.0 - 1e-6 {
                st = (st.0 + a, st.1 + b);
            }
        }
        println!("{line}");
        println!(
            "   pooled: weak rock {} -> {} ({:+.1} %), strong rock {} -> {} ({:+.1} %)",
            wk.0,
            wk.1,
            100.0 * (wk.1 as f64 - wk.0 as f64) / wk.0.max(1) as f64,
            st.0,
            st.1,
            100.0 * (st.1 as f64 - st.0 as f64) / st.0.max(1) as f64
        );

        // ---- Knickpoints, on both worlds ----
        let tcol = treat.geology.column().expect("column");
        let ch = treat.channels.as_ref().expect("channels");
        let nk = knickpoints(&treat.field, &ch.chan, &ch.recv, tcol, sea, r_expose).iter().filter(|&&b| b).count();
        let chan_land = (0..gw * gh).filter(|&i| ch.chan[i] != 0 && treat.field[i] as f64 > sea).count();
        let kc = ctrl.channels.as_ref().expect("channels");
        let nk0 = knickpoints(&ctrl.field, &kc.chan, &kc.recv, col, sea, r_expose).iter().filter(|&&b| b).count();
        println!("knickpoints: {nk0} (carve off) -> {nk} (carve on), of {chan_land} land channel cells (on)");
    }
}
