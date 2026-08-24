//! Compute-configuration benchmark: CPU/Rayon vs. each real GPU vs. the
//! two-GPU split, over the full `generate_terrain` pipeline, plus the
//! interactive LOD tile-synthesis cost that decides how the app *feels*.
//!
//! Written for `PERFORMANCE_BENCHMARKS.md`. `timing_bench.rs` next door
//! measures one thing (CPU wall clock at four sizes) and is kept as-is; this
//! adds the three axes that one cannot reach: which device actually ran,
//! per-tile synthesis distribution rather than a mean, and what happens to
//! frame times when generation and tile synthesis are genuinely concurrent.
//!
//! ```text
//! cargo run --release -p cartalith-engine --example compute_config_bench -- devices
//! cargo run --release -p cartalith-engine --example compute_config_bench -- gen  <cpu|gpu0|gpu1|gpusplit> <size> <plates> <reps>
//! cargo run --release -p cartalith-engine --example compute_config_bench -- tiles <cpu|gpu0|gpu1|gpusplit> <size> <plates>
//! cargo run --release -p cartalith-engine --example compute_config_bench -- interactive <cpu|gpu0|gpu1|gpusplit> <size> <plates>
//! ```
//!
//! One process per configuration on purpose: peak working set is read from
//! the OS for *this* process, so a run that never touched the GPU reports a
//! peak with no GPU driver allocations folded into it.

use std::time::{Duration, Instant};

use cartalith_engine::bake::pyramid_tile;
use cartalith_engine::{WorldParams, generate_terrain};
use cartalith_gpu::{GpuPreferences, MultiGpuMode};
use cartalith_spatial::pyramid::{ChunkId, pyramid_dims, pyramid_tile_bounds};
use cartalith_spatial::{Region, tile_dims};
use cartalith_terrain::amplify::{AmplifyOpts, refine_tile};
use cartalith_terrain::tile_render::{shade_tile, u8_clamped};

// --- lod_bridge.rs's own constants, mirrored ---------------------------------
// This file deliberately does not depend on `cartalith-godot` (a `cdylib`, so
// nothing can link it) -- it reproduces the exact call sequence
// `lod_bridge::synthesize_tile_rgba` makes, from the same committed engine
// functions, so the number measured here is the number that path pays.
const TILE_PX: usize = 256;
const REFERENCE_TILE_PX: usize = 1024;
const MAX_LEVEL: i32 = 10;
const SUN_AZ_DEG: f64 = 315.0;
const EXAG: f64 = 3.4;
const SHADE_RATIO_MID: f64 = 128.0;
const SHADE_RATIO_GAIN: f64 = 256.0;

fn z_base() -> i32 {
    AmplifyOpts::default().z_base + (REFERENCE_TILE_PX / TILE_PX).ilog2() as i32
}

fn tile_size_px(gw: usize, gh: usize, z: i32) -> (usize, usize) {
    let n = pyramid_dims(z.clamp(0, MAX_LEVEL)).cols as usize;
    let sel = Region { x: 0, y: 0, w: gw.saturating_sub(1), h: gh.saturating_sub(1) };
    let d = tile_dims(&sel, n, n, TILE_PX);
    (d.w, d.h)
}

/// `lod_bridge::synthesize_tile_rgba`, reproduced call for call.
#[allow(clippy::too_many_arguments)]
fn synth_tile(field: &[f32], gw: usize, gh: usize, z: i32, col: i32, row: i32, seed: i32, sea: f64) -> Vec<u8> {
    let opts = AmplifyOpts { seed, sea, z_base: z_base(), ..AmplifyOpts::default() };
    let tile = pyramid_tile(field, gw, gh, ChunkId::new(z as u32, col as u32, row as u32), TILE_PX, &opts);
    let (out_w, out_h) = tile_size_px(gw, gh, z);
    let detailed = tile.data;

    let n = pyramid_dims(z).cols as usize;
    let region = Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }.to_float();
    let plain_opts = AmplifyOpts { detail_amp: 0.0, ..opts };
    let plain =
        refine_tile(field, gw, gh, &region, n, n, col as usize, row as usize, out_w, out_h, &plain_opts);

    let bounds = pyramid_tile_bounds(gw, gh, z, col as u32, row as u32);
    let exag = EXAG * (out_w as f64 / bounds.w).max(1.0);
    let sd = shade_tile(&detailed, out_w, out_h, sea, SUN_AZ_DEG, exag);
    let sp = shade_tile(&plain, out_w, out_h, sea, SUN_AZ_DEG, exag);

    let mut rgba = vec![255u8; out_w * out_h * 4];
    for i in 0..out_w * out_h {
        let ratio = if sp[i] > 0.0 { sd[i] / sp[i] } else { 1.0 };
        let ratio = if ratio.is_nan() { 1.0 } else { ratio };
        let b = u8_clamped(SHADE_RATIO_MID + (ratio - 1.0) * SHADE_RATIO_GAIN);
        rgba[i * 4] = b;
        rgba[i * 4 + 1] = b;
        rgba[i * 4 + 2] = b;
    }
    rgba
}

// --- plumbing ----------------------------------------------------------------

/// Every real (non-software, compute-capable) GPU, in `enumerate_devices`
/// order: discrete first, then integrated.
fn real_devices() -> Vec<cartalith_gpu::GpuDeviceInfo> {
    cartalith_gpu::enumerate_devices().into_iter().filter(|d| !d.is_software && d.supports_compute).collect()
}

/// Apply the named configuration to the process-global GPU preferences and
/// return whether `use_gpu` should be set.
fn apply_config(cfg: &str) -> bool {
    let devs = real_devices();
    match cfg {
        "cpu" => {
            cartalith_gpu::set_preferences(GpuPreferences::default());
            false
        }
        "gpu0" | "gpu1" => {
            let i = if cfg == "gpu0" { 0 } else { 1 };
            let d = devs.get(i).unwrap_or_else(|| panic!("no device at index {i}"));
            println!("# requested device[{i}] = {:?} ({:?})", d.name, d.device_type);
            cartalith_gpu::set_preferences(GpuPreferences {
                selected_keys: vec![d.key.clone()],
                ..Default::default()
            });
            true
        }
        "gpusplit" => {
            assert!(devs.len() >= 2, "gpusplit needs two real GPUs");
            println!("# requested split across {:?} + {:?}", devs[0].name, devs[1].name);
            cartalith_gpu::set_preferences(GpuPreferences {
                selected_keys: devs.iter().take(2).map(|d| d.key.clone()).collect(),
                mode: MultiGpuMode::SplitTiles,
                ..Default::default()
            });
            true
        }
        other => panic!("unknown config {other:?}"),
    }
}

fn params(size: usize, plates: usize, use_gpu: bool) -> WorldParams {
    let mut p = WorldParams::defaults(size, size, 12345);
    p.tect.plates = plates;
    p.use_gpu = use_gpu;
    p
}

/// This process's peak working set, from the OS. No new dependency: the same
/// number Task Manager shows, asked for the way `MEMORY_OPTIMIZATION_SCOPE.md`
/// asked for it.
fn peak_working_set_mb() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).PeakWorkingSet64")])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().ok().map(|b| b / (1024.0 * 1024.0))
}

fn pct(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted_ms.len() - 1) as f64 * p).round() as usize;
    sorted_ms[i]
}

fn report(label: &str, mut v: Vec<f64>) {
    v.sort_by(f64::total_cmp);
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let sd = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64).sqrt();
    println!(
        "{label}: n={} mean={mean:.2}ms sd={sd:.2}ms p50={:.2} p95={:.2} p99={:.2} max={:.2} \
         over16.7ms={:.0}%",
        v.len(),
        pct(&v, 0.50),
        pct(&v, 0.95),
        pct(&v, 0.99),
        pct(&v, 1.0),
        100.0 * v.iter().filter(|x| **x > 16.7).count() as f64 / v.len() as f64,
    );
}

// --- modes -------------------------------------------------------------------

fn mode_devices() {
    for (i, d) in cartalith_gpu::enumerate_devices().iter().enumerate() {
        println!(
            "device[{i}] key={:?} name={:?} kind={} backend={} software={} \
             adapter_max_buffer={}MB adapter_max_storage_binding={}MB",
            d.key,
            d.name,
            d.kind_str(),
            d.backend_str(),
            d.is_software,
            d.max_buffer_size / (1024 * 1024),
            d.max_storage_buffer_binding_size / (1024 * 1024)
        );
    }
}

fn mode_gen(cfg: &str, size: usize, plates: usize, reps: usize) {
    let use_gpu = apply_config(cfg);
    let p = params(size, plates, use_gpu);
    let mut times = Vec::new();
    for r in 0..reps {
        let t = Instant::now();
        let ws = generate_terrain(&p);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        times.push(ms);
        if r == 0 {
            println!("# gpu_stages_used = {:?}", ws.gpu_stages_used);
            let used = cartalith_gpu::last_usage();
            for (name, u) in &used {
                println!(
                    "# device actually used: {name:?} allocated={}KB reserved={}MB",
                    u.allocated_bytes / 1024,
                    u.reserved_bytes / (1024 * 1024)
                );
            }
            if used.is_empty() {
                println!("# device actually used: none (pure CPU path)");
            }
            println!("# land fraction check: sea_level={:.3} field[0]={:.4}", ws.sea_level, ws.field[0]);
        }
        println!("rep{r} {ms:.1}ms");
    }
    let best = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    println!("RESULT gen cfg={cfg} size={size} plates={plates} best={best:.1}ms mean={mean:.1}ms");
    if let Some(mb) = peak_working_set_mb() {
        println!("RESULT peak_working_set_mb cfg={cfg} size={size} {mb:.0}");
    }
}

/// Which pyramid levels an interactive session actually reaches, and how many
/// tiles a 1920x1080 viewport wants at each. Levels are chosen so the same
/// ground is covered at increasing depth, which is what a zoom-in does.
const LEVELS: [i32; 5] = [4, 6, 7, 8, 9];

fn mode_tiles(cfg: &str, size: usize, plates: usize) {
    let use_gpu = apply_config(cfg);
    let p = params(size, plates, use_gpu);
    let t = Instant::now();
    let ws = generate_terrain(&p);
    println!("# generated in {:.1}ms (gpu stages {:?})", t.elapsed().as_secs_f64() * 1e3, ws.gpu_stages_used);
    let (field, seed, sea) = (&ws.field, p.tect.seed, ws.sea_level);

    for z in LEVELS {
        let n = pyramid_dims(z).cols as i32;
        // 48 tiles clustered at the middle of the level, which is what
        // `viewport_host.gd`'s closest-N-to-centre budget picks.
        let mid = n / 2;
        let mut v = Vec::new();
        let mut warm = 0usize;
        for k in 0..48 {
            let col = (mid + (k % 8) - 4).clamp(0, n - 1);
            let row = (mid + (k / 8) - 3).clamp(0, n - 1);
            let t = Instant::now();
            let rgba = synth_tile(field, size, size, z, col, row, seed, sea);
            v.push(t.elapsed().as_secs_f64() * 1e3);
            warm += rgba.len();
        }
        std::hint::black_box(warm);
        report(&format!("RESULT tile cfg={cfg} size={size} z={z}"), v);
    }

    // The real unit of stall: one `_update_lod()` call synthesising its whole
    // 48-tile budget inside one frame, then `_process()` draining 6 per frame.
    for z in [6, 8] {
        let n = pyramid_dims(z).cols as i32;
        let mid = n / 2;
        let t = Instant::now();
        for k in 0..48 {
            let col = (mid + (k % 8) - 4).clamp(0, n - 1);
            let row = (mid + (k / 8) - 3).clamp(0, n - 1);
            std::hint::black_box(synth_tile(field, size, size, z, col, row, seed, sea));
        }
        let burst = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        for k in 0..6 {
            let col = (mid + k).clamp(0, n - 1);
            std::hint::black_box(synth_tile(field, size, size, z, col, mid, seed, sea));
        }
        let catchup = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "RESULT stall cfg={cfg} size={size} z={z} update_burst_48={burst:.1}ms catchup_6={catchup:.1}ms"
        );
    }
    if let Some(mb) = peak_working_set_mb() {
        println!("RESULT peak_working_set_mb cfg={cfg} size={size} {mb:.0}");
    }
}

/// The one genuinely concurrent CPU+GPU measurement this harness can make
/// without inventing new pipeline architecture: generation running on a worker
/// thread (exactly what `engine_bridge.gd` does) while a simulated UI thread
/// synthesises LOD tiles at a 60 Hz cadence. Reports the UI thread's frame-time
/// distribution *while* generation is in flight against the same distribution
/// with the machine otherwise idle.
fn mode_interactive(cfg: &str, size: usize, plates: usize) {
    let use_gpu = apply_config(cfg);

    // A first world to have a field to synthesise tiles from -- the interactive
    // case is always "a world is on screen and another generate was started".
    let p0 = params(size, plates, use_gpu);
    let ws = generate_terrain(&p0);
    let (field, seed, sea) = (ws.field.clone(), p0.tect.seed, ws.sea_level);
    let z = 8;
    let n = pyramid_dims(z).cols as i32;
    let mid = n / 2;

    // Baseline: the UI thread alone.
    let mut idle = Vec::new();
    for k in 0..40 {
        let t = Instant::now();
        for j in 0..6 {
            let col = (mid + ((k + j) % 16) - 8).clamp(0, n - 1);
            let row = (mid + ((k + j) / 16) - 4).clamp(0, n - 1);
            std::hint::black_box(synth_tile(&field, size, size, z, col, row, seed, sea));
        }
        idle.push(t.elapsed().as_secs_f64() * 1e3);
    }
    report(&format!("RESULT ui_frame cfg={cfg} size={size} state=idle"), idle);

    // Concurrent: the same UI loop while a full generate runs on a worker.
    let p1 = params(size, plates, use_gpu);
    let gen_start = Instant::now();
    let worker = std::thread::spawn(move || {
        let t = Instant::now();
        let ws = generate_terrain(&p1);
        (t.elapsed().as_secs_f64() * 1e3, ws.gpu_stages_used)
    });

    let mut busy = Vec::new();
    while !worker.is_finished() {
        let t = Instant::now();
        for j in 0..6 {
            let k = busy.len() as i32;
            let col = (mid + ((k + j) % 16) - 8).clamp(0, n - 1);
            let row = (mid + ((k + j) / 16) - 4).clamp(0, n - 1);
            std::hint::black_box(synth_tile(&field, size, size, z, col, row, seed, sea));
        }
        busy.push(t.elapsed().as_secs_f64() * 1e3);
        // A real frame also sleeps; without this the "UI thread" is a spin
        // loop that would starve the generation it is measuring against.
        std::thread::sleep(Duration::from_millis(1));
    }
    let (gen_ms, stages) = worker.join().expect("generation thread");
    let wall = gen_start.elapsed().as_secs_f64() * 1e3;
    report(&format!("RESULT ui_frame cfg={cfg} size={size} state=generating"), busy);
    println!(
        "RESULT concurrent cfg={cfg} size={size} gen_ms={gen_ms:.1} wall_ms={wall:.1} stages={stages:?}"
    );
    if let Some(mb) = peak_working_set_mb() {
        println!("RESULT peak_working_set_mb cfg={cfg} size={size} {mb:.0}");
    }
}

/// Headroom, not a proposal: the same 48-tile `_update_lod()` burst dispatched
/// across the Rayon pool instead of one thread, and the same burst with only
/// the *detailed* half computed (the shade ratio's "plain" reference pass costs
/// a second `refine_tile` plus a second `shade_tile`). Both are measurements of
/// what a fix would be worth, made without changing the shipped path.
fn mode_tilepar(cfg: &str, size: usize, plates: usize) {
    use rayon::prelude::*;

    let use_gpu = apply_config(cfg);
    let p = params(size, plates, use_gpu);
    let ws = generate_terrain(&p);
    let (field, seed, sea) = (&ws.field, p.tect.seed, ws.sea_level);
    println!("# rayon threads = {}", rayon::current_num_threads());

    for z in [6i32, 8] {
        let n = pyramid_dims(z).cols as i32;
        let mid = n / 2;
        let idx: Vec<(i32, i32)> = (0..48)
            .map(|k| ((mid + (k % 8) - 4).clamp(0, n - 1), (mid + (k / 8) - 3).clamp(0, n - 1)))
            .collect();

        let t = Instant::now();
        for (col, row) in &idx {
            std::hint::black_box(synth_tile(field, size, size, z, *col, *row, seed, sea));
        }
        let seq = t.elapsed().as_secs_f64() * 1e3;

        let t = Instant::now();
        let out: Vec<Vec<u8>> =
            idx.par_iter().map(|(col, row)| synth_tile(field, size, size, z, *col, *row, seed, sea)).collect();
        let par = t.elapsed().as_secs_f64() * 1e3;
        std::hint::black_box(out);

        // The detailed half only -- what the tile would cost if the plain
        // reference pass were not recomputed per tile.
        let opts = AmplifyOpts { seed, sea, z_base: z_base(), ..AmplifyOpts::default() };
        let t = Instant::now();
        for (col, row) in &idx {
            std::hint::black_box(pyramid_tile(
                field,
                size,
                size,
                ChunkId::new(z as u32, *col as u32, *row as u32),
                TILE_PX,
                &opts,
            ));
        }
        let detail_only = t.elapsed().as_secs_f64() * 1e3;

        println!(
            "RESULT tilepar cfg={cfg} size={size} z={z} seq48={seq:.1}ms par48={par:.1}ms \
             speedup={:.2}x detail_only48={detail_only:.1}ms",
            seq / par
        );
    }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mode = a.first().map(String::as_str).unwrap_or("devices");
    let cfg = a.get(1).map(String::as_str).unwrap_or("cpu");
    let size = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(2048usize);
    let plates = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(40usize);
    let reps = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(3usize);
    match mode {
        "devices" => mode_devices(),
        "gen" => mode_gen(cfg, size, plates, reps),
        "tiles" => mode_tiles(cfg, size, plates),
        "tilepar" => mode_tilepar(cfg, size, plates),
        "interactive" => mode_interactive(cfg, size, plates),
        other => panic!("unknown mode {other:?}"),
    }
}
