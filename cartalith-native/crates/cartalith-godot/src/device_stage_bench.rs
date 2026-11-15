//! Per-stage New World generation timing, run on the PC and on Android
//! handsets over `adb` (owner, 2026-11-14: *"we should also check
//! optimisation on snapdragon cpu (like the pad3 and 6t ... generation in the
//! pad feels as slow"*). Results are in `PERFORMANCE_BENCHMARKS.md` §12.
//!
//! **Why here, in `cartalith-godot`, and not in `cartalith-engine`'s
//! examples.** The world the app makes is `generate_terrain` over
//! `params::defaults()` (every ruled divergence on) followed by
//! `compute_civilisation`; the engine's own `WorldParams::defaults()` is the
//! golden-parity path with the divergences off and no civilisation layer, so
//! timing it would time a world nobody generates. Same reasoning as
//! `way_census.rs`.
//!
//! **What it measures.** For each repetition: the wall time of each of the
//! ten `cartalith_engine::progress` stages (read by polling
//! `progress::snapshot()` from a second thread every millisecond, so a stage
//! boundary is resolved to about 1 ms and no engine code is touched), the
//! civilisation layer, and the total. It also prints rayon's thread count, the
//! GPU stages `generate_terrain` reports it used, and a digest of the
//! generated world, so a build-flag change can be checked to move no output.
//! On Linux/Android it samples `/proc/self/task/*/stat` every ~20 ms and
//! attributes each thread's CPU ticks to the core it was last seen on, which
//! is how big-versus-little core use is read.
//!
//! **Must never:** gate anything. The test is `#[ignore]`d: it is a
//! measurement that runs for minutes, and its numbers depend on the machine.
//!
//! Run alone, release (MISTAKES.md: never under a parallel suite):
//! `cargo test --release -p cartalith-godot --lib device_stage_bench -- --ignored --nocapture --test-threads 1`.
//! On a handset: build the test binary with `cargo ndk -t arm64-v8a test
//! --release -p cartalith-godot --lib --no-run`, `adb push` it to
//! `/data/local/tmp`, and run it there with the same arguments.
//!
//! Environment (all optional): `SNAP_REPS` (timed repetitions, default 5),
//! `SNAP_GW` (grid width, default 2048, the New World dialog's
//! `RESOLUTION_DEFAULT_INDEX`), `SNAP_SEED` (default 12345), `SNAP_GPU`
//! (`1` = `use_gpu` on, as the shell turns it on at boot in
//! `engine_bridge.gd`; default `0`), `SNAP_WARM` (untimed warm-up runs,
//! default 1).

use crate::{coarse_ocean_wind_fields, compute_civilisation};
use cartalith_engine::progress;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Reads an integer environment variable, falling back to `default` when it
/// is unset or does not parse.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// FNV-1a 64 (Fowler-Noll-Vo; offset basis and prime are the published
/// constants) over raw bytes. Used for the world digest only.
fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Folds an `f32` slice in by its exact bit patterns, so any change to a
/// single value (including a last-bit rounding change from a codegen flag)
/// moves the digest.
fn fnv_f32(h: u64, v: &[f32]) -> u64 {
    let mut h = h;
    for x in v {
        h = fnv(h, &x.to_bits().to_le_bytes());
    }
    h
}

/// Per-core CPU ticks gathered from `/proc/self/task/*/stat`. Linux/Android
/// only; empty elsewhere.
#[derive(Default)]
struct CoreTicks {
    /// Last seen `utime + stime` per thread id.
    last: std::collections::HashMap<u32, u64>,
    /// Ticks attributed to each core index.
    per_core: Vec<u64>,
}

impl CoreTicks {
    /// One sample: each thread's tick delta since the previous sample goes to
    /// the core it is on now (`stat` field 39, `processor`). That attribution
    /// is approximate -- a thread can migrate between samples -- which is
    /// why it is read as a share, not an exact split.
    fn sample(&mut self) {
        let Ok(dir) = std::fs::read_dir("/proc/self/task") else { return };
        for e in dir.flatten() {
            let Ok(tid) = e.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(s) = std::fs::read_to_string(e.path().join("stat")) else { continue };
            // Fields after the `comm` field's closing parenthesis start at
            // field 3 (`state`), so field N is index N - 3 here (proc(5)).
            let Some(rest) = s.rsplit_once(')').map(|(_, r)| r) else { continue };
            let f: Vec<&str> = rest.split_whitespace().collect();
            if f.len() < 37 {
                continue;
            }
            let ticks = f[11].parse::<u64>().unwrap_or(0) + f[12].parse::<u64>().unwrap_or(0);
            let cpu = f[36].parse::<usize>().unwrap_or(0);
            let prev = self.last.insert(tid, ticks).unwrap_or(ticks);
            if self.per_core.len() <= cpu {
                self.per_core.resize(cpu + 1, 0);
            }
            self.per_core[cpu] += ticks.saturating_sub(prev);
        }
    }
}

/// The whole process's CPU time so far, `utime + stime` in clock ticks
/// (`/proc/self/stat` fields 14 and 15, proc(5)); `None` off Linux/Android.
/// Read at each stage boundary, so a stage's CPU time over its wall time is
/// the number of cores it kept busy on average -- how a serial stage is told
/// from a parallel one without a profiler (`simpleperf` is refused to the
/// `adb shell` user on both handsets: "Event type 'cpu-clock' is not
/// supported on the device", measured 2026-11-14).
fn proc_ticks() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/stat").ok()?;
    let f: Vec<&str> = s.rsplit_once(')')?.1.split_whitespace().collect();
    Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
}

/// `USER_HZ`, the unit of [`proc_ticks`]: 100 on every Linux and Android ABI
/// this runs on (the kernel's fixed userspace tick, `sysconf(_SC_CLK_TCK)`).
const USER_HZ: f64 = 100.0;

/// Formats a stage's CPU time and average busy cores, or nothing when the
/// platform has no `/proc` (Windows).
fn cpu_note(t0: Option<u64>, t1: Option<u64>, wall_s: f64) -> String {
    match (t0, t1) {
        (Some(a), Some(b)) if wall_s > 0.0 => {
            let cpu = b.saturating_sub(a) as f64 / USER_HZ;
            format!("  cpu {cpu:7.1} s  busy_cores {:4.2}", cpu / wall_s)
        }
        _ => String::new(),
    }
}

/// Each core's `cpuinfo_max_freq` in kHz, which is how a big core is told
/// from a little one (the kernel exposes no core-type flag to an app).
fn core_max_freqs() -> Vec<String> {
    (0..16)
        .map_while(|i| {
            std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{i}/cpufreq/cpuinfo_max_freq"))
                .ok()
                .map(|s| s.trim().to_string())
        })
        .collect()
}

/// Times `reps` New World generations at the shell's default size and prints
/// one `STAGE` line per stage per rep, a `TOTAL` line per rep, the digest and
/// the per-core tick shares. Protects nothing; see the module comment.
#[test]
#[ignore = "a measurement for PERFORMANCE_BENCHMARKS.md §12, minutes long"]
fn device_stage_bench() {
    let reps = env_usize("SNAP_REPS", 5);
    let warm = env_usize("SNAP_WARM", 1);
    let gw = env_usize("SNAP_GW", 2048);
    let seed = env_usize("SNAP_SEED", 12345) as i32;
    let use_gpu = env_usize("SNAP_GPU", 0) == 1;
    // 800 km and the region frame are the New World dialog's defaults
    // (`width_input`'s 800.0, `ASPECT_DEFAULT_INDEX` = region).
    let width_km = 800.0;
    let gh = crate::reference_grid_h(gw, false);
    let mut p = crate::with_call_args(crate::params::defaults(), seed, width_km, gw, gh);
    p.use_gpu = use_gpu;
    // Build the pool the way generation does before anything below touches
    // rayon: `rayon::current_num_threads()` in the CONFIG line would otherwise
    // build the global pool implicitly at rayon's default and bypass
    // `auto_thread_count` (found 2026-11-15: the 6T then ran 6 workers).
    cartalith_engine::ensure_thread_pool();

    println!(
        "CONFIG gw={gw} gh={gh} seed={seed} use_gpu={use_gpu} reps={reps} warm={warm} rayon_threads={} available_parallelism={:?}",
        rayon::current_num_threads(),
        std::thread::available_parallelism().map(|n| n.get()).ok()
    );
    println!("CORE_MAX_FREQ_KHZ {}", core_max_freqs().join(","));

    let ticks = Arc::new(Mutex::new(CoreTicks::default()));
    for rep in 0..(warm + reps) {
        let timed = rep >= warm;
        let stop = Arc::new(AtomicBool::new(false));
        let marks: Arc<Mutex<Vec<(usize, Instant, Option<u64>)>>> = Arc::new(Mutex::new(Vec::new()));
        let token0 = progress::snapshot().0;
        let poller = {
            let (stop, marks, ticks) = (stop.clone(), marks.clone(), ticks.clone());
            std::thread::spawn(move || {
                let mut last = usize::MAX;
                let mut n = 0u32;
                while !stop.load(Ordering::Relaxed) {
                    let (tok, st) = progress::snapshot();
                    if tok > token0 && st != last {
                        marks.lock().unwrap().push((st, Instant::now(), proc_ticks()));
                        last = st;
                    }
                    n += 1;
                    if timed && n % 20 == 0 {
                        ticks.lock().unwrap().sample();
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            })
        };
        let (t0, k0) = (Instant::now(), proc_ticks());
        let ws = cartalith_engine::generate_terrain(&p);
        let (t_terrain, k_terrain) = (Instant::now(), proc_ticks());
        let (o, w) = coarse_ocean_wind_fields(&ws.field, gw, gh, p.world, ws.sea_level, &p);
        let civ = compute_civilisation(
            &ws, gw, gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false,
            &mut Vec::new(), None, p.lake_balance,
        );
        let (t_end, k_end) = (Instant::now(), proc_ticks());
        stop.store(true, Ordering::Relaxed);
        poller.join().unwrap();

        // Stage s runs from its first sighting to the next sighting of a
        // later stage (or the end of `generate_terrain`). Stages skipped
        // between two samples (fetch_max can jump several) get no line.
        let m = marks.lock().unwrap().clone();
        if timed {
            for (i, &(st, at, k)) in m.iter().enumerate() {
                let (end, k_next) = m
                    .get(i + 1)
                    .map(|x| (x.1.min(t_terrain), x.2))
                    .unwrap_or((t_terrain, k_terrain));
                let name = progress::STAGE_NAMES.get(st).copied().unwrap_or("finished");
                if st < progress::STAGE_COUNT {
                    let wall = (end - at).as_secs_f64();
                    println!("STAGE rep={} {:>2} {:<22} {:>9.1} ms{}", rep - warm, st, name, wall * 1e3, cpu_note(k, k_next, wall));
                }
            }
            let wall = (t_terrain - t0).as_secs_f64();
            println!("STAGE rep={} -- {:<22} {:>9.1} ms{}", rep - warm, "terrain total", wall * 1e3, cpu_note(k0, k_terrain, wall));
            let wall = (t_end - t_terrain).as_secs_f64();
            println!("STAGE rep={} -- {:<22} {:>9.1} ms{}", rep - warm, "civilisation", wall * 1e3, cpu_note(k_terrain, k_end, wall));
            println!("TOTAL rep={} {:.1} ms", rep - warm, (t_end - t0).as_secs_f64() * 1e3);
        }
        if rep == warm {
            let mut h = 0xcbf2_9ce4_8422_2325u64;
            for f in [&*ws.field, &*ws.temperature, &*ws.rainfall, &*ws.flow_discharge] {
                h = fnv_f32(h, f);
            }
            if let Some(rm) = ws.river_mask.as_ref() {
                h = fnv(h, rm);
            }
            let mut hc = 0xcbf2_9ce4_8422_2325u64;
            for s in civ.settlements.iter() {
                hc = fnv(hc, format!("{s:?}").as_bytes());
            }
            for wy in civ.ways.iter() {
                hc = fnv(hc, format!("{wy:?}").as_bytes());
            }
            println!(
                "DIGEST terrain={h:016x} civ={hc:016x} settlements={} ways={} gpu_stages_used={:?}",
                civ.settlements.len(),
                civ.ways.len(),
                ws.gpu_stages_used
            );
        }
    }
    let t = ticks.lock().unwrap();
    let total: u64 = t.per_core.iter().sum();
    if total > 0 {
        let shares: Vec<String> = t
            .per_core
            .iter()
            .enumerate()
            .map(|(i, &v)| format!("cpu{i}={:.1}%", 100.0 * v as f64 / total as f64))
            .collect();
        println!("CORE_TICK_SHARE {}", shares.join(" "));
    }
}
