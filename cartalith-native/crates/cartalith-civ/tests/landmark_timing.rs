//! Ad-hoc timing harness for `landmark::generate` — diagnostic only.
//! Run with: cargo test -p cartalith-civ --test landmark_timing --release -- --ignored --nocapture
//! (and again without --release to see the profile the Godot editor actually loads).

use cartalith_civ::landmark::{self, LandmarkInputs, LandmarkSettings, LandmarkSite};
use std::time::Instant;

const SEA: f64 = 0.42;
const PEAK_M: f64 = 4000.0;

fn test_field(gw: usize, gh: usize) -> Vec<f32> {
    let mut f = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let fx = x as f64 / gw as f64;
            let fy = y as f64 / gh as f64;
            let mut h = 0.80 - 0.633 * fx;
            h += 0.14 * ((0.30 - fx) * 400.0).tanh();
            let mask = ((0.35 - fx) * 4.0).clamp(0.0, 1.0);
            let rough = (x as f64 * 0.55).sin() * (y as f64 * 0.47).cos()
                + 0.5 * (x as f64 * 1.3).sin() * (y as f64 * 1.1).cos();
            h += mask * 0.030 * rough;
            h -= 0.050 * (fy * std::f64::consts::PI * 6.0).sin().abs();
            for (px, py, amp, sig) in [
                (0.10f64, 0.22f64, 0.14f64, 0.030f64),
                (0.18, 0.62, 0.11, 0.028),
                (0.07, 0.86, 0.09, 0.025),
            ] {
                let d2 = (fx - px).powi(2) + (fy - py).powi(2);
                h += amp * (-d2 / (2.0 * sig * sig)).exp();
            }
            let d2 = (fx - 0.20f64).powi(2) + (fy - 0.50f64).powi(2);
            h -= 0.20 * (-d2 / (2.0 * 0.040f64 * 0.040)).exp();
            f[y * gw + x] = h.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

fn blob(gw: usize, gh: usize, px: f64, py: f64, sig: f64) -> Vec<f32> {
    let mut v = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let d2 = (x as f64 / gw as f64 - px).powi(2) + (y as f64 / gh as f64 - py).powi(2);
            v[y * gw + x] = (-d2 / (2.0 * sig * sig)).exp() as f32;
        }
    }
    v
}

struct World {
    gw: usize,
    gh: usize,
    width_km: f64,
    field: Vec<f32>,
    flow: Vec<f32>,
    chan: Vec<u8>,
    recv: Vec<i32>,
    order: Vec<i16>,
    water: Vec<u8>,
    corridors: Vec<f32>,
    iron: Vec<f32>,
    stone: Vec<f32>,
    timber: Vec<f32>,
    settlements: Vec<LandmarkSite>,
}

fn world(gw: usize, gh: usize, width_km: f64) -> (World, f64) {
    let field = test_field(gw, gh);
    let flow = cartalith_hydrology::compute_flow(gw, gh, &field, None, false, false);
    let ch =
        cartalith_hydrology::build_channels(&field, &flow, gw, gh, SEA, false, 1.0, width_km);
    let order = cartalith_hydrology::strahler_from_receivers(&ch.recv, &flow, &ch.chan);
    let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SEA, false, None);

    // This half is what `landmark_geology_inputs()` in cartalith-godot/src/lib.rs
    // redoes on every `landmark_run()`; time the corridor pass on its own.
    let t = Instant::now();
    let raw_slope = cartalith_civ::build_raw_slope_field(&field, gw, gh, false);
    let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
    let corridors = cartalith_civ::build_route_corridors(
        &field, &raw_slope, Some(&flow), gw, gh, SEA, false, thresh,
    );
    let corridor_s = t.elapsed().as_secs_f64();

    let mk = |seeds: &[(f64, f64)]| -> Vec<f32> {
        let mut v = vec![0f32; gw * gh];
        for (px, py) in seeds {
            let b = blob(gw, gh, *px, *py, 0.028);
            for i in 0..v.len() {
                if b[i] > v[i] {
                    v[i] = b[i];
                }
            }
        }
        v
    };

    (
        World {
            gw,
            gh,
            width_km,
            field,
            flow,
            chan: ch.chan,
            recv: ch.recv,
            order,
            water: wb.classification,
            corridors,
            iron: mk(&[(0.24, 0.20), (0.26, 0.72), (0.13, 0.10)]),
            stone: mk(&[(0.23, 0.45), (0.26, 0.92)]),
            timber: mk(&[(0.30, 0.30), (0.10, 0.55)]),
            settlements: vec![
                LandmarkSite { x: gw / 4, y: gh / 3, population: 12_000.0 },
                LandmarkSite { x: gw / 3, y: (gh * 2) / 3, population: 4_000.0 },
            ],
        },
        corridor_s,
    )
}

fn run_at(gw: usize, gh: usize, width_km: f64, settlements: usize) {
    let t_all = Instant::now();
    let (mut w, corridor_s) = world(gw, gh, width_km);
    let prep_s = t_all.elapsed().as_secs_f64();

    // Scale the settlement list — `Ctx::influence()` is O(candidates x settlements).
    let base = w.settlements.clone();
    w.settlements.clear();
    for k in 0..settlements {
        let b = &base[k % base.len()];
        w.settlements.push(LandmarkSite {
            x: (b.x + k * 7) % gw,
            y: (b.y + k * 11) % gh,
            population: b.population,
        });
    }

    let res: Vec<(&str, &[f32])> = vec![
        ("iron", w.iron.as_slice()),
        ("buildstone", w.stone.as_slice()),
        ("timber", w.timber.as_slice()),
    ];
    let mut inp = LandmarkInputs::new(&w.field, w.gw, w.gh, SEA, false, w.width_km);
    inp.peak_m = PEAK_M;
    inp.flow = Some(&w.flow);
    inp.channel = Some(&w.chan);
    inp.recv = Some(&w.recv);
    inp.order = Some(&w.order);
    inp.water = Some(&w.water);
    inp.corridors = Some(&w.corridors);
    inp.resources = &res;
    inp.settlements = &w.settlements;

    let s = LandmarkSettings::default();
    let t = Instant::now();
    let r = landmark::generate(&inp, &s, 4242);
    let gen_s = t.elapsed().as_secs_f64();

    let cells = gw * gh;
    println!(
        "{:>5}x{:<5} ({:>9} cells, cell {:.3} km, {:>3} settlements)  \
         corridors {:>7.2}s | fixture prep {:>7.2}s | generate {:>7.2}s | {:>5} placed | {:.3} us/cell",
        gw,
        gh,
        cells,
        width_km / gw as f64,
        settlements,
        corridor_s,
        prep_s,
        gen_s,
        r.landmarks.len(),
        gen_s * 1e6 / cells as f64,
    );
    let mut worst: Vec<_> = r.funnels.iter().filter(|f| f.candidates > 0).collect();
    worst.sort_by_key(|f| std::cmp::Reverse(f.candidates));
    for f in worst.iter().take(5) {
        println!("        {:<28} candidates {:>9}", f.kind, f.candidates);
    }
}

#[test]
#[ignore = "diagnostic timing harness, minutes long at the top size"]
fn landmark_generate_scaling() {
    println!();
    // Constant cell size (0.39 km, the shell's own 2048 @ 800 km default), so
    // the analysis radii clamp identically at every size and the only variable
    // is cell count.
    for (gw, gh) in [(256usize, 164usize), (512, 328), (1024, 656), (2048, 1311)] {
        run_at(gw, gh, gw as f64 * 0.390625, 2);
    }
}

#[test]
#[ignore = "diagnostic timing harness"]
fn landmark_generate_settlement_scaling() {
    println!();
    for k in [2usize, 50, 200, 800] {
        run_at(512, 328, 200.0, k);
    }
}

#[test]
#[ignore = "diagnostic timing harness — the shell's real default world"]
fn landmark_generate_at_the_shipping_default() {
    println!();
    run_at(2048, 1311, 800.0, 2);
}

/// `landmark_geology_inputs()` (cartalith-godot/src/lib.rs) redoes the whole
/// lithology + biome + resource-potential + slope + corridor chain on every
/// `landmark_run()`. Time that half on its own at the shipping default.
#[test]
#[ignore = "diagnostic timing harness"]
fn geology_inputs_cost() {
    println!();
    for (gw, gh) in [(512usize, 328usize), (2048, 1311)] {
        let n = gw * gh;
        let width_km = gw as f64 * 0.390625;
        let field = test_field(gw, gh);
        let noise = |k: f64| -> Vec<f32> {
            (0..n)
                .map(|i| {
                    let (x, y) = ((i % gw) as f64, (i / gw) as f64);
                    (0.5 + 0.5 * ((x * k).sin() * (y * k * 1.3).cos())) as f32
                })
                .collect()
        };
        let age = noise(0.011);
        let volc = noise(0.023);
        let crust = noise(0.007);
        let resist = noise(0.017);
        let rain = noise(0.005);
        let temp = noise(0.003);
        let shear = noise(0.031);
        let boundary: Vec<u8> = (0..n).map(|i| (i % 7) as u8).collect();
        let flow = cartalith_hydrology::compute_flow(gw, gh, &field, None, false, false);
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SEA, false, None);

        let t = std::time::Instant::now();
        let lith = cartalith_civ::build_lithology(&field, &age, &volc, &crust, &resist, &rain, SEA);
        let t_lith = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let biome = cartalith_civ::build_biome_raster(&wb.classification, &temp, &rain);
        let t_biome = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let _rp = cartalith_civ::build_resource_potentials(
            &lith, Some(&boundary), Some(&shear), Some(&flow), Some(&biome), &field, &rain,
            &age, gw, gh, SEA, Some(&volc), true, false,
        );
        let t_res = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let raw_slope = cartalith_civ::build_raw_slope_field(&field, gw, gh, false);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let _cor = cartalith_civ::build_route_corridors(
            &field, &raw_slope, Some(&flow), gw, gh, SEA, false, thresh,
        );
        let t_cor = t.elapsed().as_secs_f64();
        println!(
            "{:>5}x{:<5} lithology {:.3}s | biome {:.3}s | resource potentials {:.3}s | slope+corridors {:.3}s | TOTAL {:.3}s",
            gw, gh, t_lith, t_biome, t_res, t_cor,
            t_lith + t_biome + t_res + t_cor
        );
    }
}

/// The fixture above is analytically smooth; a really generated heightfield is
/// not, and candidate counts are what a rough field changes. Same size, same
/// cell, fractal noise added — how much does `generate()` cost then?
#[test]
#[ignore = "diagnostic timing harness"]
fn landmark_generate_on_a_rough_field() {
    println!();
    for amp in [0.0f64, 0.004, 0.012, 0.030] {
        let (gw, gh) = (2048usize, 1311usize);
        let width_km = 800.0;
        let mut field = test_field(gw, gh);
        if amp > 0.0 {
            for y in 0..gh {
                for x in 0..gw {
                    let (fx, fy) = (x as f64, y as f64);
                    let mut v = 0.0;
                    let mut a = 1.0;
                    let mut f = 0.09;
                    for _ in 0..5 {
                        v += a * ((fx * f).sin() * (fy * f * 1.37 + 1.1).cos()
                            + (fx * f * 0.61 + 2.3).cos() * (fy * f * 0.83).sin());
                        a *= 0.5;
                        f *= 2.03;
                    }
                    field[y * gw + x] = (field[y * gw + x] as f64 + amp * v).clamp(0.0, 1.0) as f32;
                }
            }
        }
        let flow = cartalith_hydrology::compute_flow(gw, gh, &field, None, false, false);
        let ch = cartalith_hydrology::build_channels(
            &field, &flow, gw, gh, SEA, false, 1.0, width_km,
        );
        let order = cartalith_hydrology::strahler_from_receivers(&ch.recv, &flow, &ch.chan);
        let wb = cartalith_civ::build_water_bodies(&field, gw, gh, SEA, false, None);
        let raw_slope = cartalith_civ::build_raw_slope_field(&field, gw, gh, false);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let corridors = cartalith_civ::build_route_corridors(
            &field, &raw_slope, Some(&flow), gw, gh, SEA, false, thresh,
        );
        let iron = blob(gw, gh, 0.24, 0.20, 0.028);
        let stone = blob(gw, gh, 0.23, 0.45, 0.028);
        let timber = blob(gw, gh, 0.30, 0.30, 0.028);
        let sites = vec![
            LandmarkSite { x: gw / 4, y: gh / 3, population: 12_000.0 },
            LandmarkSite { x: gw / 3, y: (gh * 2) / 3, population: 4_000.0 },
        ];
        let res: Vec<(&str, &[f32])> = vec![
            ("iron", iron.as_slice()),
            ("buildstone", stone.as_slice()),
            ("timber", timber.as_slice()),
        ];
        let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
        inp.peak_m = PEAK_M;
        inp.flow = Some(&flow);
        inp.channel = Some(&ch.chan);
        inp.recv = Some(&ch.recv);
        inp.order = Some(&order);
        inp.water = Some(&wb.classification);
        inp.corridors = Some(&corridors);
        inp.resources = &res;
        inp.settlements = &sites;
        let s = LandmarkSettings::default();
        let t = Instant::now();
        let r = landmark::generate(&inp, &s, 4242);
        let gen_s = t.elapsed().as_secs_f64();
        let total: u64 = r.funnels.iter().map(|f| f.candidates as u64).sum();
        println!(
            "roughness {:.3}  generate {:>7.2}s  {:>6} placed  {:>10} candidates total",
            amp, gen_s, r.landmarks.len(), total
        );
        let mut worst: Vec<_> = r.funnels.iter().filter(|f| f.candidates > 0).collect();
        worst.sort_by_key(|f| std::cmp::Reverse(f.candidates));
        for f in worst.iter().take(6) {
            println!("        {:<28} candidates {:>9}  placed {:>4}", f.kind, f.candidates, f.placed);
        }
    }
}

/// Where the 4 seconds actually go: `Derived::build`'s four analysis fields,
/// timed directly. `r_fine`/`r_broad` are what `to_cells()` produces at the
/// shell's own default (2048 columns @ 800 km -> 0.39 km cells): 3 km -> 8
/// cells, 25 km -> 64 cells clamped to SCALE_MAX_CELLS = 40.
#[test]
#[ignore = "diagnostic timing harness"]
fn derived_build_component_cost() {
    use cartalith_terrain::analysis;
    println!();
    let (gw, gh) = (2048usize, 1311usize);
    let field = test_field(gw, gh);
    let (r_fine, r_broad) = (8i64, 40i64);
    let t = Instant::now();
    let _ = analysis::slope(&field, gw, gh);
    println!("slope()                        {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let _ = analysis::curvature_at(&field, gw, gh, r_fine, false);
    println!("curvature_at(r_fine=8)         {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let _ = analysis::tpi(&field, gw, gh, r_fine, false);
    println!("tpi(r_fine=8)                  {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let _ = analysis::tpi(&field, gw, gh, r_broad, false);
    println!("tpi(r_broad=40)                {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let _ = analysis::local_relief(&field, gw, gh, r_broad, false);
    println!("local_relief(r_broad=40)       {:.3}s   <- same shape as sep_min_max()", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let _ = analysis::local_relief(&field, gw, gh, r_fine, false);
    println!("local_relief(r_fine=8)         {:.3}s   <- x4, inside pool_ridge + 3x pool_resource", t.elapsed().as_secs_f64());
}
