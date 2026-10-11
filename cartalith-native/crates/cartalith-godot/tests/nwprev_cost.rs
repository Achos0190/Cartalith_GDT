//! **New World seed preview: cost and faithfulness measurement**
//! (`NEW_WORLD_PREVIEW_PROPOSAL.md`, 2026-10-11).
//!
//! It measures; it changes no generated output, and nothing in the engine or the
//! shell calls it. Every test here is `#[ignore]`d. Run it alone, in release:
//!
//! ```text
//! cargo test --release -p cartalith-godot --test nwprev_cost -- --ignored --nocapture --test-threads=1
//! ```
//!
//! # What it answers
//!
//! 1. **Cost.** How long the pipeline's cheap early stages (warp, plates, plate
//!    assignment, stress, flexure, base blur, age, heterogeneity, height,
//!    normalise -- the CPU path of `generate_terrain_inner` up to the
//!    `VOLCANISM` banner) take on a 320-pixel-wide grid, and how long the whole
//!    of `generate_terrain` takes there, for comparison.
//! 2. **Faithfulness.** Whether a 320-wide land/ocean mask predicts the mask of
//!    the world the same seed gives at 1024 wide. Agreement is reported beside
//!    the agreement two *unrelated* masks with the same land fractions would
//!    reach by chance, and beside a different-seed negative control, because a
//!    bare percentage of a mostly-ocean map is not evidence of anything.
//!
//! # The arms
//!
//! - `early320 naive` -- the early stages at 320 wide with the app's own
//!   parameters, unchanged.
//! - `early320 scaled` -- the same, with the two cell-unit inputs re-expressed
//!   for the target width: `tect.blur_r` multiplied by `320 / 1024` (it is a
//!   radius in CELLS), and `map_width_km` multiplied by the same ratio where it
//!   feeds `terrain_detail_k` (so the noise frequency `nf` and the
//!   heterogeneity frequency are the target grid's, not the preview's).
//! - `plates320` -- the crudest faithful mask: the sign of each cell's plate
//!   `base` after `assign_plates`, nothing else.
//! - `full320` -- `generate_terrain` at 320 wide.
//! - references at the target width (1024, or `NWPREV_TARGET`): `earlyT` and `fullT` (the world a user
//!   would actually get at that width from the same seed and parameters).
//!
//! The CPU path throughout (`use_gpu` is `false` in `params::defaults()`); the
//! shipped shell turns the GPU on, whose stage outputs are principled-equivalent
//! rather than bit-identical, so this measures the CPU arm only.

#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use cartalith_engine::{generate_terrain, WorldParams};
use cartalith_terrain::{
    assign_plates, build_age_field, build_plates, compute_flexure, compute_height, compute_heterogeneity,
    compute_stress, compute_warp, gauss_blur, normalize_field, terrain_detail_k, HeightParams,
};
use std::time::Instant;

/// The preview's width in cells (the owner's "320,px resolution").
const PREVIEW_W: usize = 320;
/// The default comparison width: the smallest resolution preset above 512 (`RESOLUTION_PRESETS`).
/// `NWPREV_TARGET` overrides it for `nwprev_overlap` (e.g. 2048, the dialog's default); an
/// unparsable value panics rather than falling back.
const TARGET_W: usize = 1024;

/// The comparison width for this run: `NWPREV_TARGET` when set, else `TARGET_W`.
fn target_w() -> usize {
    match std::env::var("NWPREV_TARGET") {
        Ok(v) => v.parse::<usize>().unwrap_or_else(|_| panic!("NWPREV_TARGET: cannot parse `{v}`")),
        Err(_) => TARGET_W,
    }
}

/// Seeds measured. A labelled judgement: five arbitrary values, including the
/// seed the owner's own glass report quoted (51127).
const SEEDS: [i32; 5] = [1, 42, 1234, 51127, 987_654];

/// `reference_grid_h` in `cartalith-godot/src/lib.rs`: 2:1 for a world, 0.64 for a region.
fn grid_h(gw: usize, world: bool) -> usize {
    let k = if world { 0.5 } else { 0.64 };
    ((gw as f64 * k).round() as usize).max(4)
}

/// The app's own parameters (`params::defaults()`) for one preview case.
fn app_params(seed: i32, km: f64, world: bool, plates: usize, gw: usize) -> WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = grid_h(gw, world);
    p.world = world;
    p.tect.seed = seed;
    p.tect.plates = plates;
    p.map_width_km = km;
    p
}

/// The CPU path of `generate_terrain_inner` from the warp to `normalize_field`, World
/// Structure off (Classic). `blur_scale` multiplies `tect.blur_r` and `km_scale` the km
/// width fed to `terrain_detail_k`; both are 1.0 for the faithful arm. Returns the
/// normalised height field and the per-cell plate-base sign mask (1 = continental).
fn early(p: &WorldParams, blur_scale: f64, km_scale: f64) -> (Vec<f32>, Vec<u8>) {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    let blur_r = p.tect.blur_r * blur_scale;
    let km = p.map_width_km * km_scale;
    let warp = compute_warp(gw, gh, p.tect.seed, p.tect.warp, world);
    let (wx, wy) = match &warp {
        Some((a, b)) => (Some(a.as_slice()), Some(b.as_slice())),
        None => (None, None),
    };
    let plates = build_plates(gw, gh, p.tect.seed as u32, p.tect.plates, p.tect.lloyd, world, None);
    let pid = assign_plates(gw, gh, world, &plates, wx, wy);
    let stress = compute_stress(gw, gh, world, &pid, &plates, p.tect.vel, blur_r);
    let flex = compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, blur_r, world);
    let base_raw: Vec<f32> = pid.iter().map(|&i| plates[i as usize].base as f32).collect();
    // `plate_base_blur_r`: max(2, blur_r * K), K = 0.18 with narrow_plate_base_blur (the app).
    let k = if p.tect.narrow_plate_base_blur { 0.18 } else { 0.35 };
    let base = gauss_blur(&base_raw, (blur_r * k).max(2.0), gw, gh, world);
    let age = build_age_field(gw, gh, &stress.boundary_mask);
    let hetero = compute_heterogeneity(gw, gh, p.tect.seed, km, world, &age, wx, wy);
    let hp = HeightParams {
        nf: 5.0 * terrain_detail_k(gw, km),
        seed: p.tect.seed,
        a: p.tect.alpha,
        b: p.tect.beta,
        age_inf: p.tect.age_inf,
        fwt: p.tect.flexure,
        hwt: p.tect.hetero,
        world,
        ridged: p.tect.ridged,
    };
    let raw = compute_height(gw, gh, &base, &stress.stress_field, &flex, &hetero, &age, wx, wy, None, &hp);
    let sign: Vec<u8> = base_raw.iter().map(|&b| u8::from(b > 0.0)).collect();
    (normalize_field(&raw), sign)
}

/// Land (1) where `field > sea`.
fn mask(field: &[f32], sea: f64) -> Vec<u8> {
    field.iter().map(|&v| u8::from(v as f64 > sea)).collect()
}

/// Agreement between a `lw`-wide mask and a `hw`-wide one, sampled at each low cell's
/// centre. Returns (agreement %, chance agreement %, land % low, land % high, land IoU %).
/// Panics on an empty mask: a silently-empty comparison is the trap `MISTAKES.md` names.
fn compare(lo: &[u8], lw: usize, lh: usize, hi: &[u8], hw: usize, hh: usize) -> (f64, f64, f64, f64, f64) {
    assert!(!lo.is_empty() && !hi.is_empty(), "empty mask");
    assert_eq!(lo.len(), lw * lh);
    assert_eq!(hi.len(), hw * hh);
    let (mut agree, mut land_lo, mut land_hi, mut both, mut either) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for y in 0..lh {
        let hy = (((y as f64 + 0.5) * hh as f64 / lh as f64) as usize).min(hh - 1);
        for x in 0..lw {
            let hx = (((x as f64 + 0.5) * hw as f64 / lw as f64) as usize).min(hw - 1);
            let a = lo[y * lw + x];
            let b = hi[hy * hw + hx];
            agree += usize::from(a == b);
            land_lo += a as usize;
            land_hi += b as usize;
            both += usize::from(a == 1 && b == 1);
            either += usize::from(a == 1 || b == 1);
        }
    }
    let n = (lw * lh) as f64;
    let (pl, ph) = (land_lo as f64 / n, land_hi as f64 / n);
    let chance = pl * ph + (1.0 - pl) * (1.0 - ph);
    let iou = if either == 0 { f64::NAN } else { both as f64 / either as f64 };
    (100.0 * agree as f64 / n, 100.0 * chance, 100.0 * pl, 100.0 * ph, 100.0 * iou)
}

fn fmt_cmp(name: &str, c: (f64, f64, f64, f64, f64)) -> String {
    format!(
        "{name:<34} agree {:5.1}%  chance {:5.1}%  land {:4.1}% vs {:4.1}%  landIoU {:5.1}%",
        c.0, c.1, c.2, c.3, c.4
    )
}

/// Median, min and max of a set of millisecond samples.
fn spread(mut v: Vec<f64>) -> String {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    format!("median {:.1} ms ({:.1}..{:.1}, n={})", v[v.len() / 2], v[0], v[v.len() - 1], v.len())
}

/// The two cases the owner named: a 200 km region at 6 plates and an Earth-width world at 36.
const CASES: [(&str, f64, bool, usize); 2] = [("region 200 km, 6 plates", 200.0, false, 6), ("world 40 075 km, 36 plates", 40075.0, true, 36)];

/// Cost of the early stages and of the whole pipeline at 320 wide (and the early stages at
/// 1024 for scale). Protects nothing; prints medians with min..max. Run ALONE.
#[test]
#[ignore = "measurement: run alone in release (see the module doc)"]
fn nwprev_cost() {
    println!("rayon threads: {}", rayon::current_num_threads());
    for (name, km, world, plates) in CASES {
        let p = app_params(51127, km, world, plates, PREVIEW_W);
        let _ = early(&p, 1.0, 1.0); // warm-up (thread pool, page faults)
        let mut e = Vec::new();
        for s in SEEDS.iter().chain(SEEDS.iter()) {
            let p = app_params(*s, km, world, plates, PREVIEW_W);
            let t = Instant::now();
            let (f, _) = early(&p, PREVIEW_W as f64 / TARGET_W as f64, PREVIEW_W as f64 / TARGET_W as f64);
            assert!(!f.is_empty());
            e.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let mut full = Vec::new();
        for s in SEEDS {
            let p = app_params(s, km, world, plates, PREVIEW_W);
            let t = Instant::now();
            let ws = generate_terrain(&p);
            assert!(!ws.field.is_empty());
            full.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let mut e1024 = Vec::new();
        for s in SEEDS {
            let p = app_params(s, km, world, plates, TARGET_W);
            let t = Instant::now();
            let _ = early(&p, 1.0, 1.0);
            e1024.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        // What one wrong seed costs today: the whole CPU pipeline at the dialog's default 2K width.
        let mut full2k = Vec::new();
        for s in &SEEDS[..3] {
            let p = app_params(*s, km, world, plates, 2048);
            let t = Instant::now();
            let ws = generate_terrain(&p);
            assert!(!ws.field.is_empty());
            full2k.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!("{name}: full generate_terrain 2048x{} (CPU, the dialog default): {}", grid_h(2048, world), spread(full2k));
        println!("{name}: early stages {}x{}: {}", PREVIEW_W, grid_h(PREVIEW_W, world), spread(e));
        println!("{name}: full generate_terrain {}x{}: {}", PREVIEW_W, grid_h(PREVIEW_W, world), spread(full));
        println!("{name}: early stages {}x{}: {}", TARGET_W, grid_h(TARGET_W, world), spread(e1024));
    }
}

/// Mask agreement between every preview arm and the 1024-wide world, per seed and case, plus
/// a different-seed negative control. Protects nothing; prints. Run ALONE (it runs the full
/// pipeline at 1024 wide ten times).
#[test]
#[ignore = "measurement: run alone in release (see the module doc)"]
fn nwprev_overlap() {
    let target = target_w();
    let r = PREVIEW_W as f64 / target as f64;
    for (name, km, world, plates) in CASES {
        println!("== {name}  (preview {}x{}, target {}x{})", PREVIEW_W, grid_h(PREVIEW_W, world), target, grid_h(target, world));
        let (lw, lh, hw, hh) = (PREVIEW_W, grid_h(PREVIEW_W, world), target, grid_h(target, world));
        let mut prev_full: Option<(Vec<u8>, i32)> = None;
        for s in SEEDS {
            let pl = app_params(s, km, world, plates, PREVIEW_W);
            let ph = app_params(s, km, world, plates, target);
            let sea = pl.sea_level;
            let (naive, sign_lo) = early(&pl, 1.0, 1.0);
            let (scaled, _) = early(&pl, r, r);
            let (e_hi, sign_hi) = early(&ph, 1.0, 1.0);
            let full_lo = generate_terrain(&pl);
            let full_hi = generate_terrain(&ph);
            let m_naive = mask(&naive, sea);
            let m_scaled = mask(&scaled, sea);
            let m_e_hi = mask(&e_hi, sea);
            let m_full_lo = mask(&full_lo.field, full_lo.sea_level);
            let m_full_hi = mask(&full_hi.field, full_hi.sea_level);
            println!("-- seed {s} (sea level {sea})");
            println!("{}", fmt_cmp("early320 naive  vs fullT", compare(&m_naive, lw, lh, &m_full_hi, hw, hh)));
            println!("{}", fmt_cmp("early320 scaled vs fullT", compare(&m_scaled, lw, lh, &m_full_hi, hw, hh)));
            println!("{}", fmt_cmp("plates320 sign  vs fullT", compare(&sign_lo, lw, lh, &m_full_hi, hw, hh)));
            println!("{}", fmt_cmp("full320         vs fullT", compare(&m_full_lo, lw, lh, &m_full_hi, hw, hh)));
            println!("{}", fmt_cmp("early320 scaled vs earlyT", compare(&m_scaled, lw, lh, &m_e_hi, hw, hh)));
            println!("{}", fmt_cmp("plates320 sign  vs platesT", compare(&sign_lo, lw, lh, &sign_hi, hw, hh)));
            // Same resolution, early vs final: how far everything after tectonics moves the coast.
            println!("{}", fmt_cmp("earlyT       vs fullT", compare(&m_e_hi, hw, hh, &m_full_hi, hw, hh)));
            if let Some((other, os)) = &prev_full {
                println!("{}", fmt_cmp(&format!("NEG early320 scaled vs fullT seed {os}"), compare(&m_scaled, lw, lh, other, hw, hh)));
            }
            prev_full = Some((m_full_hi, s));
        }
    }
}
