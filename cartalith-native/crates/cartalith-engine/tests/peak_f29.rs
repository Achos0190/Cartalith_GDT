//! Ruling GL / F29: the derived peak altitude (`derived_peak_m`, `resolve_peak`).
//!
//! Protects: the law's anchors and monotonicity (a literal table, so a changed constant goes red),
//! that `peak_auto` is off in `WorldParams::defaults` (the goldens' baseline) and a no-op when off,
//! that resolution is deterministic and resolution-independent, and that collision energy responds
//! monotonically to `tect.vel`. `measure_f29` (ignored) prints the report numbers.
use cartalith_engine::*;

fn p(seed: i32, plates: usize, vel: f64, km: f64) -> WorldParams {
    let mut p = WorldParams::defaults(384, 256, seed);
    p.tect.plates = plates;
    p.tect.vel = vel;
    p.map_width_km = km;
    p.peak_auto = true;
    p
}

#[test]
fn law_anchors_are_literal_and_monotone() {
    // (cm/yr, km) -> metres: Alps-like 1 cm/yr and Himalaya-like 5 cm/yr on a continent-sized map.
    let near = |a: f64, b: f64| (a - b).abs() < 1.0;
    assert!(near(peak_from_energy(0.0, 1e9), 2000.0));
    assert!((3150.0..3250.0).contains(&peak_from_energy(1.0, 1e9)), "slow, old belt ~3.2 km");
    assert!((7200.0..7400.0).contains(&peak_from_energy(5.0, 1e9)), "fast collision ~7.3 km");
    assert!(peak_from_energy(1e6, 1e9) < 9000.5, "saturates near 9 km");
    assert!(peak_from_energy(5.0, 200.0) < peak_from_energy(5.0, 800.0) && peak_from_energy(5.0, 800.0) < peak_from_energy(5.0, 8000.0));
    let mut last = 0.0;
    for e in 0..40 {
        let v = peak_from_energy(e as f64 * 0.5, 3000.0);
        assert!(v >= last);
        last = v;
    }
}

#[test]
fn parity_baseline_is_off_and_resolve_is_a_noop_when_off() {
    let base = WorldParams::defaults(64, 64, 1);
    assert!(!base.peak_auto);
    let mut q = base.clone();
    resolve_peak(&mut q);
    assert_eq!(q, base);
}

#[test]
fn derived_peak_is_deterministic_and_grid_independent_and_responds_to_velocity() {
    let a = derived_peak_m(&p(483920, 14, 1.0, 3000.0));
    assert_eq!(a, derived_peak_m(&p(483920, 14, 1.0, 3000.0)));
    let mut big = p(483920, 14, 1.0, 3000.0);
    big.gw = 1536;
    big.gh = 1024;
    assert_eq!(a, derived_peak_m(&big), "proxy grid: resolution does not move the peak");
    let mut last = 0.0;
    for vel in [0.0, 0.25, 0.5, 1.0, 1.5, 2.0] {
        let v = derived_peak_m(&p(912345, 14, vel, 3000.0));
        assert!(v >= last, "vel {vel}: {v} < {last}");
        last = v;
    }
    assert_eq!(derived_peak_m(&p(912345, 14, 0.0, 3000.0)), PEAK_BASE_M);
}

#[test]
#[ignore]
fn measure_f29() {
    for seed in [483920, 912345, 246371] {
        for (plates, vel, km) in [(8, 1.0, 3000.0), (14, 0.5, 3000.0), (14, 1.0, 800.0), (14, 1.0, 3000.0), (14, 2.0, 3000.0), (14, 2.0, 800.0), (14, 3.0, 800.0), (30, 1.0, 3000.0), (14, 1.0, 12000.0)] {
            let q = p(seed, plates, vel, km);
            let e = collision_energy_cm_yr(&q);
            println!("PEAK seed {seed} plates {plates} vel {vel} km {km}: E {e:.2} cm/yr -> {:.0} m", derived_peak_m(&q));
        }
        // before / after shares on a land-and-climate world at the app-like size
        for (auto, km) in [(false, 800.0), (true, 800.0), (false, 3000.0), (true, 3000.0)] {
            let mut q = p(seed, 14, 1.0, km);
            q.peak_auto = auto;
            resolve_peak(&mut q);
            let ws = generate_terrain(&q);
            let (mut land, mut sub0, mut sub5, mut subm10) = (0usize, 0usize, 0usize, 0usize);
            let mut snow = 0usize;
            let (mut ice_t, mut high_m) = (0usize, 0f64);
            for i in 0..ws.field.len() {
                if (ws.field[i] as f64) <= ws.sea_level { continue }
                land += 1;
                let t = ws.temperature[i] as f64;
                if t < 0.0 { sub0 += 1 }
                if t < 5.0 { sub5 += 1 }
                if t < -10.0 { subm10 += 1 }
                if (ws.field[i] as f64) > ws.sea_level + (1.0 - ws.sea_level) * q.passes.glacial_snowline { snow += 1 }
                high_m = high_m.max((ws.field[i] as f64 - ws.sea_level) / (1.0 - ws.sea_level) * q.peak_m);
                let _ = &mut ice_t;
            }
            let f = |n: usize| 100.0 * n as f64 / land as f64;
            println!("SHARE seed {seed} km {km} auto {auto} peak {:.0} m: land {:.1}% | T<0 {:.1}% T<5 {:.1}% T<-10 {:.1}% | above 0.65 snowline {:.1}% | top {:.0} m", q.peak_m, 100.0 * land as f64 / ws.field.len() as f64, f(sub0), f(sub5), f(subm10), f(snow), high_m);
        }
    }
}

#[test]
fn resolve_writes_the_derived_peak_when_on_and_pins_a_literal_world() {
    let mut q = p(483920, 14, 1.0, 3000.0);
    resolve_peak(&mut q);
    assert!((4900.0..5020.0).contains(&q.peak_m), "483920 at 14 plates, drift 1, 3000 km: {}", q.peak_m);
    let mut off = p(483920, 14, 1.0, 3000.0);
    off.peak_auto = false;
    off.peak_m = 4321.0;
    resolve_peak(&mut off);
    assert_eq!(off.peak_m, 4321.0, "the override is used as typed");
}

/// Ruling GQ's fit data: land share below 0 C against a fixed peak, per seed (ignored; prints).
#[test]
#[ignore]
fn sweep_cold_share() {
    for seed in [483920, 912345, 246371] {
        for pk in [3500.0, 4000.0, 4250.0, 4500.0, 4750.0, 5000.0, 5500.0] {
            let mut q = p(seed, 14, 1.0, 3000.0);
            q.peak_auto = false;
            q.peak_m = pk;
            let ws = generate_terrain(&q);
            let (mut land, mut cold) = (0usize, 0usize);
            for i in 0..ws.field.len() {
                if (ws.field[i] as f64) > ws.sea_level {
                    land += 1;
                    if (ws.temperature[i] as f64) < 0.0 { cold += 1 }
                }
            }
            println!("COLD seed {seed} peak {pk}: {:.1}%", 100.0 * cold as f64 / land as f64);
        }
    }
}
