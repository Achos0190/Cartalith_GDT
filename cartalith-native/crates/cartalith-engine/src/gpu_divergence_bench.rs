//! Which GPU kernel makes the GPU path's world differ from one device to
//! the next, and which kernel is faster than its CPU stage (owner,
//! 2026-11-15: *"See where the gpu gives a benefit"*;
//! `PERFORMANCE_BENCHMARKS.md` §12.6). Run on the PC and on a handset, then
//! compare the printed hashes.
//!
//! **Method.** The CPU path's own tectonic substrate is built once; every GPU
//! kernel `generate_terrain` could run (warp, plate assignment, stress, the
//! flexure and base-field blurs, heterogeneity, weather, plain-D8 flow) is
//! then fed **the same CPU-made inputs**, so a difference in one kernel's
//! output cannot be inherited from an earlier GPU kernel. For each it prints
//! the FNV-1a hash of the GPU output's bits, the hash of the CPU stage it
//! replaces (equal on every machine: the CPU path is deterministic), how many
//! cells differ from the CPU and by how much, and the median wall time of
//! both. Two devices whose `GPU` hash differs for a kernel compute that
//! kernel differently. The flexure blur, its f32 CPU mirror, flow and the
//! flow pipeline's build time (`flow_init`) were added 2026-11-15 for Ruling
//! EZ item 3: these lines are the measurements `GPU_PATH_KERNELS` rests on,
//! and re-running them on a new device is how that list is re-checked.
//!
//! **Not the harness world.** This runs `WorldParams::defaults()` (the golden
//! path) at the harness's grid and seed, not `cartalith_godot::params::defaults()`,
//! because the kernels are reached here through crate-private functions. The
//! kernels and their arithmetic are the same either way; only their inputs
//! differ.
//!
//! **Must never:** gate anything. `#[ignore]`d: it is a measurement whose
//! numbers depend on the GPU and driver.
//!
//! `cargo test --release -p cartalith-engine --lib gpu_divergence_bench -- --ignored --nocapture --test-threads 1`
//! (environment: `SNAP_GW`, default 2048; `SNAP_SEED`, default 12345;
//! `SNAP_REPS`, timed repetitions per kernel, default 3).

use super::*;
use std::time::Instant;

/// Reads an integer environment variable, or `default`.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// FNV-1a 64 over each value's exact bits (offset basis and prime are the
/// published constants), the same digest `device_stage_bench` prints.
fn hash_bits<T: Copy>(v: &[T], bits: impl Fn(T) -> u64) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &x in v {
        for b in bits(x).to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// Hash of an `f32` field by bit pattern.
fn hf(v: &[f32]) -> u64 {
    hash_bits(v, |x| u64::from(x.to_bits()))
}

/// Cells that differ, and the largest absolute difference.
fn diff(a: &[f32], b: &[f32]) -> (usize, f64) {
    assert_eq!(a.len(), b.len(), "compared fields must be the same size");
    a.iter().zip(b).fold((0, 0.0), |(n, m), (&x, &y)| {
        if x.to_bits() == y.to_bits() {
            (n, m)
        } else {
            (n + 1, m.max((x as f64 - y as f64).abs()))
        }
    })
}

/// Median wall time of `reps` calls of `f`, in milliseconds, and the last
/// result.
fn timed<R>(reps: usize, mut f: impl FnMut() -> R) -> (f64, R) {
    let mut ts = Vec::new();
    let mut last = None;
    for _ in 0..reps.max(1) {
        let t = Instant::now();
        last = Some(f());
        ts.push(t.elapsed().as_secs_f64() * 1e3);
    }
    ts.sort_by(f64::total_cmp);
    (ts[ts.len() / 2], last.expect("at least one repetition ran"))
}

/// Prints one `KERNEL` line.
fn report(name: &str, gpu: Option<&[f32]>, cpu: &[f32], t_gpu: f64, t_cpu: f64) {
    match gpu {
        Some(g) => {
            let (n, m) = diff(g, cpu);
            println!(
                "KERNEL {name:<16} gpu={:016x} cpu={:016x} differing={n} of {} max_abs={m:.3e} gpu_ms={t_gpu:.1} cpu_ms={t_cpu:.1}",
                hf(g),
                hf(cpu),
                cpu.len()
            );
        }
        None => println!("KERNEL {name:<16} gpu=unavailable cpu={:016x} cpu_ms={t_cpu:.1}", hf(cpu)),
    }
}

/// See the module comment. Protects nothing.
#[test]
#[ignore = "a measurement for PERFORMANCE_BENCHMARKS.md §12.6; needs a GPU"]
fn gpu_divergence_bench() {
    let gw = env_usize("SNAP_GW", 2048);
    let gh = (gw as f64 * 1311.0 / 2048.0).round() as usize;
    let seed = env_usize("SNAP_SEED", 12345) as i32;
    let reps = env_usize("SNAP_REPS", 3);
    let mut p = WorldParams::defaults(gw, gh, seed);
    p.use_gpu = true;
    ensure_thread_pool();
    let set = open_gpu_set(&p);
    let Some(gpu) = set.as_ref().map(|s| s.primary()) else {
        println!("GPU none: open_gpu_set returned no device");
        return;
    };
    println!("CONFIG gw={gw} gh={gh} seed={seed} reps={reps} rayon_threads={} adapter={:?} backend={:?}", rayon::current_num_threads(), gpu.adapter_name, gpu.adapter_backend);
    let world = p.world;
    let (tect_plates, tect_vel, _) = world_structure_derived(&p);

    // Warp: the GPU kernel against the CPU path's f64 `compute_warp`.
    let amp = (p.tect.warp * 0.18 * gw as f64) as f32;
    let wf = (if world { 3.0 } else { 2.5 } / gw as f64) as f32;
    let (t_cpu, warp) = timed(reps, || compute_warp(gw, gh, p.tect.seed, p.tect.warp, world).expect("default warp is on"));
    let (t_gpu, gw_out) = timed(reps, || cartalith_gpu::warp_grid_gpu_with(gpu, gw as u32, gh as u32, p.tect.seed, wf, amp, world));
    report("warp_x", gw_out.as_ref().map(|w| w.0.as_slice()), &warp.0, t_gpu, t_cpu);
    report("warp_y", gw_out.as_ref().map(|w| w.1.as_slice()), &warp.1, t_gpu, t_cpu);

    // Plate assignment over the CPU warp.
    let plates = build_plates(gw, gh, p.tect.seed as u32, tect_plates, p.tect.lloyd, world, None);
    let (t_cpu, plate_id) = timed(reps, || assign_plates(gw, gh, world, &plates, Some(&warp.0), Some(&warp.1)));
    let px: Vec<f32> = plates.iter().map(|pl| pl.x as f32).collect();
    let py: Vec<f32> = plates.iter().map(|pl| pl.y as f32).collect();
    let (t_gpu, ids) = timed(reps, || cartalith_gpu::assign_plates_grid_gpu_with(gpu, gw as u32, gh as u32, &px, &py, Some(&warp.0), Some(&warp.1), world));
    let as_f = |v: &[u16]| v.iter().map(|&x| x as f32).collect::<Vec<f32>>();
    let ids_f: Option<Vec<f32>> = ids.map(|v| v.iter().map(|&x| x as f32).collect());
    report("plate_assignment", ids_f.as_deref(), &as_f(&plate_id), t_gpu, t_cpu);

    // Stress over the CPU plate ids.
    let (t_cpu, stress) = timed(reps, || compute_stress(gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r));
    let (t_gpu, sg) = timed(reps, || compute_stress_gpu(gpu, gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r));
    let mask_f = |v: &[u8]| v.iter().map(|&x| x as f32).collect::<Vec<f32>>();
    report("stress_mask", sg.as_ref().map(|s| mask_f(&s.boundary_mask)).as_deref(), &mask_f(&stress.boundary_mask), t_gpu, t_cpu);
    report("stress_field", sg.as_ref().map(|s| s.stress_field.as_slice()), &stress.stress_field, t_gpu, t_cpu);
    report("stress_shear", sg.as_ref().map(|s| s.shear_field.as_slice()), &stress.shear_field, t_gpu, t_cpu);

    // The flexure blur (the same blur kernel at `blur_r * 3`) over the CPU
    // stress, normalised as `tectonic_substrate` normalises it (added
    // 2026-11-15 for Ruling EZ item 3: it is a separate dispatch with its own
    // radius, so its speed is measured, not assumed from the base blur's).
    let (t_cpu, flex) = timed(reps, || compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world));
    let (t_gpu, fg) = timed(reps, || flexure_gpu(gpu, gw, gh, world, &stress, p.tect.blur_r));
    report("flexure_blur", fg.as_deref(), &flex, t_gpu, t_cpu);
    // The same through the blur's f32 CPU mirror, normalised as
    // `flexure_gpu` does: a device whose GPU line equals this one computes
    // the kernel exactly as IEEE f32 does.
    let masked: Vec<f32> = stress.boundary_mask.iter().zip(&stress.stress_field).map(|(&m, &v)| if m != 0 { v } else { 0.0 }).collect();
    let fm = cartalith_gpu::gpu_gauss_blur_grid_cpu(&masked, p.tect.blur_r * 3.0, gw as u32, gh as u32, world);
    let fmx = fm.iter().fold(1e-6f64, |m, &v| m.max((v as f64).abs()));
    let fm: Vec<f32> = fm.iter().map(|&v| (v as f64 / fmx) as f32).collect();
    report("flexure_vs_f32_cpu", fg.as_deref(), &fm, t_gpu, 0.0);

    // The base field's blur over the CPU's plate base heights.
    let base_raw: Vec<f32> = plate_id.iter().map(|&pid| plates[pid as usize].base as f32).collect();
    let r = plate_base_blur_r(&p.tect);
    let (t_cpu, base) = timed(reps, || gauss_blur(&base_raw, r, gw, gh, world));
    let (t_gpu, bg) = timed(reps, || cartalith_gpu::gauss_blur_grid_gpu_with(gpu, &base_raw, r, gw as u32, gh as u32, world));
    report("base_field_blur", bg.as_deref(), &base, t_gpu, t_cpu);
    let mirror = cartalith_gpu::gpu_gauss_blur_grid_cpu(&base_raw, r, gw as u32, gh as u32, world);
    report("blur_vs_f32_cpu", bg.as_deref(), &mirror, t_gpu, 0.0);

    // Heterogeneity over the CPU age field and warp; the GPU's raw output is
    // normalised exactly as `tectonic_substrate` normalises it.
    let age = build_age_field(gw, gh, &stress.boundary_mask);
    let (t_cpu, het) = timed(reps, || compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age, Some(&warp.0), Some(&warp.1)));
    let hf64 = 1.5 * cartalith_terrain::terrain_detail_k(gw, p.map_width_km);
    let p_x = (js_round(hf64).max(2.0)) as i32;
    let (t_gpu, hg) = timed(reps, || {
        cartalith_gpu::heterogeneity_grid_gpu_with(gpu, gw as u32, gh as u32, p.tect.seed ^ 0x44bb, hf64 as f32 / gw as f32, world, p_x, &age, &warp.0, &warp.1).map(|mut out| {
            let mx = out.iter().fold(1e-6f64, |m, &v| m.max((v as f64).abs()));
            for v in &mut out {
                *v = (*v as f64 / mx) as f32;
            }
            out
        })
    });
    report("heterogeneity", hg.as_deref(), &het, t_gpu, t_cpu);

    // Weather over the CPU substrate's height field.
    let sub = tectonic_substrate(&WorldParams { use_gpu: false, ..p.clone() }, tect_plates, tect_vel, None, None, &mut Vec::new());
    let wp = weather_params_for(&p, p.sea_level);
    let (t_cpu, rain) = timed(reps, || simulate_weather_with(gw, gh, &sub.field, p.climate.w_iters, 0.0, &wp, p.climate.rain_upsample()));
    let (t_gpu, rg) = timed(reps, || {
        let grid = cartalith_climate::build_weather_grid(gw, gh, &sub.field, 0.0, &wp);
        cartalith_gpu::simulate_weather_loop_gpu_with(
            gpu,
            &grid.eh,
            &grid.tc,
            &grid.sst_evap,
            &grid.wx,
            &grid.wy,
            &grid.w_init,
            grid.ww as u32,
            grid.wh as u32,
            p.climate.w_iters,
            grid.sea as f32,
            grid.ocean_hum as f32,
            grid.evap as f32,
            grid.ocean as f32,
            grid.rain_k as f32,
            grid.dry as f32,
            grid.step as f32,
            grid.bulk_evap,
            grid.wrap_x,
        )
        .map(|(_w, r)| cartalith_climate::finish_weather_grid_with(&grid.eh, r, grid.ww, grid.wh, grid.wrap_x, grid.sea, gw, gh, p.climate.rain_upsample()))
    });
    report("weather", rg.as_deref(), &rain, t_gpu, t_cpu);

    // D8 flow accumulation over the CPU substrate's height field, plain D8
    // (no roughness), as `generate_terrain_inner`'s `flow_on_gpu` runs it.
    // The pipeline build is timed on its own: generation pays it once per
    // world whenever a device opened (added 2026-11-15, Ruling EZ item 3).
    let (t_init, ctx) = timed(1, || cartalith_gpu::init_gpu_flow_with(gpu));
    println!("KERNEL flow_init       gpu_ms={t_init:.1}");
    let (t_cpu, acc) = timed(reps, || cartalith_hydrology::compute_flow_with(gw, gh, &sub.field, None, false, world, None));
    let (t_gpu, fa) = timed(reps, || cartalith_gpu::dispatch_gpu_flow(&ctx, gw, gh, &sub.field, None, false, world).map(|r| r.acc));
    report("flow", fa.as_deref(), &acc, t_gpu, t_cpu);
}
