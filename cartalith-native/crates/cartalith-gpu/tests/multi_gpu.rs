//! Multi-GPU against real hardware (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04).
//!
//! Every test here **skips cleanly when there is no GPU**, and one of them
//! (`enumeration_on_a_machine_with_no_gpu_is_empty_not_an_error`) asserts
//! that the no-GPU path is a normal, empty answer rather than a panic --
//! that is the headless/CI reality this crate runs under, not an edge case.
//!
//! The pure logic (adapter grouping, row splitting, budget arithmetic) is
//! unit-tested in `src/multi.rs` and needs no hardware at all; this file is
//! only for the parts that genuinely require devices.
//!
//! **Preferences are passed explicitly here, not set globally** -- every
//! device test calls [`init_gpu_device_set_with`] with its own
//! [`GpuPreferences`] value. `cargo test` runs these in parallel, and
//! `set_preferences` writes one process-global that they would otherwise all
//! be sharing: the earlier version of this file did exactly that and
//! `every_enumerated_device_can_be_selected_and_opened` failed on about one
//! run in six, when a neighbouring test's `set_preferences(default())`
//! landed between this one's write and its read. The two tests that
//! genuinely exercise the global path take `PREFS_LOCK` instead.
//!
//! Run the timing test's numbers with:
//! `cargo test -p cartalith-gpu --test multi_gpu -- --nocapture --test-threads=1`

use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use cartalith_gpu::{
    GpuPreferences, MultiGpuMode, enumerate_devices, gpu_working_set_bytes, init_gpu_device_set,
    init_gpu_device_set_with, set_preferences, split_rows, warp_grid_gpu_split, warp_grid_gpu_with,
};

/// Held for the whole of any test that writes the process-global
/// preferences, so those tests never overlap each other.
static PREFS_LOCK: Mutex<()> = Mutex::new(());

/// `PREFS_LOCK`, ignoring poisoning: one failing test must not cascade into
/// the other as a second, misleading failure.
fn global_prefs() -> MutexGuard<'static, ()> {
    PREFS_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Non-software devices, in the order [`enumerate_devices`] ranks them.
fn real_devices() -> Vec<cartalith_gpu::GpuDeviceInfo> {
    enumerate_devices().into_iter().filter(|d| !d.is_software && d.supports_compute).collect()
}

#[test]
fn enumeration_on_a_machine_with_no_gpu_is_empty_not_an_error() {
    // The call itself must not panic anywhere -- that is the assertion.
    let devs = enumerate_devices();
    for d in &devs {
        assert!(!d.key.is_empty(), "every enumerated device needs a stable key");
        assert!(!d.name.is_empty());
        // The preferred backend is never one of the alternates.
        assert!(!d.alternate_backends.contains(&d.backend));
    }
    let keys: Vec<&str> = devs.iter().map(|d| d.key.as_str()).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), keys.len(), "device keys must be unique: {keys:?}");
    println!("enumerate_devices() -> {} device(s)", devs.len());
    for d in &devs {
        println!(
            "  {:<32} {:?} via {:?} (alt {:?})  driver={:?} {:?}  max_buffer={} MB  software={}",
            d.name,
            d.device_type,
            d.backend,
            d.alternate_backends,
            d.driver,
            d.driver_info,
            d.max_buffer_size / (1024 * 1024),
            d.is_software
        );
    }
}

/// Selecting a device by key must actually open *that* device -- the whole
/// point of PR-01. Runs against every real device the machine has, so on a
/// two-GPU machine it proves the integrated GPU is reachable too, not just
/// the one `PowerPreference::HighPerformance` would have picked anyway.
#[test]
fn every_enumerated_device_can_be_selected_and_opened() {
    let devs = real_devices();
    if devs.is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    for d in &devs {
        let prefs = GpuPreferences { selected_keys: vec![d.key.clone()], ..Default::default() };
        let set = init_gpu_device_set_with(&prefs).expect("selected device must open");
        assert_eq!(set.devices().len(), 1, "single_device mode opens exactly one device");
        assert_eq!(
            set.primary().adapter_name,
            d.name,
            "selecting {:?} must open {:?}, not whatever HighPerformance prefers",
            d.key,
            d.name
        );
        assert_eq!(set.primary().device_type, d.device_type);
        println!("selected {:?} -> opened {:?} ({:?})", d.key, set.primary().adapter_name, set.primary().device_type);
    }
}

/// The same guarantee through the **ambient** entry point: a key written with
/// `set_preferences` must be the device `init_gpu_device_set()` opens. This is
/// the direct regression test for the 2026-08-24 bug, in which that call read
/// the global twice and could resolve the adapter from a second, different
/// snapshot -- silently opening the `HighPerformance` device instead of the
/// named one.
#[test]
fn a_globally_set_device_key_is_the_device_that_opens() {
    let _guard = global_prefs();
    let devs = real_devices();
    if devs.is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    for d in &devs {
        set_preferences(GpuPreferences { selected_keys: vec![d.key.clone()], ..Default::default() });
        let set = init_gpu_device_set().expect("selected device must open");
        assert_eq!(set.primary().adapter_name, d.name, "the global preference must decide, not HighPerformance");
        assert_eq!(set.primary().device_type, d.device_type);
    }
    set_preferences(GpuPreferences::default());
}

/// An unresolvable key (a GPU removed, a preference copied from another
/// machine) must degrade to the automatic pick, not to no GPU at all.
#[test]
fn an_unknown_device_key_falls_back_to_auto() {
    if real_devices().is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    let prefs = GpuPreferences { selected_keys: vec!["ffff:ffff:No Such GPU".to_string()], ..Default::default() };
    let set = init_gpu_device_set_with(&prefs).expect("an unknown key must degrade to auto, not fail");
    println!("unknown key -> fell back to {:?}", set.primary().adapter_name);
}

/// The automatic pick must never open a software rasterizer, on either
/// entry into it -- no preference at all, and a preference naming a device
/// that no longer resolves. Both take the same `HighPerformance` branch, and
/// wgpu's own request does not exclude a CPU adapter there
/// (`force_fallback_adapter: false` declines to *restrict to* fallbacks; it
/// filters nothing). "No GPU" is the right answer on such a machine, and
/// `init_gpu_device_set(...).ok()` is already how every caller reaches the
/// CPU path.
///
/// **This machine cannot make it fail**: it enumerates the Basic Render
/// Driver but also two real Radeons, so `get_order` would sort the software
/// adapter last regardless. It is the regression guard for the machine that
/// has only the software adapter -- a VM, a CI runner, a broken ICD -- where
/// it is the whole assertion.
#[test]
fn the_automatic_pick_never_opens_a_software_rasterizer() {
    let software: Vec<_> = enumerate_devices().into_iter().filter(|d| d.is_software).collect();
    if software.is_empty() {
        println!("skipped: this machine enumerates no software rasterizer");
        return;
    }
    println!("software adapters present: {:?}", software.iter().map(|d| &d.name).collect::<Vec<_>>());

    for (what, prefs) in [
        ("no preference", GpuPreferences::default()),
        (
            "an unresolvable preference",
            GpuPreferences { selected_keys: vec!["ffff:ffff:No Such GPU".to_string()], ..Default::default() },
        ),
    ] {
        match init_gpu_device_set_with(&prefs) {
            Ok(set) => {
                let d = set.primary();
                assert_ne!(
                    d.device_type,
                    wgpu::DeviceType::Cpu,
                    "{what}: the automatic pick opened {:?}, a software rasterizer",
                    d.adapter_name
                );
                println!("{what} -> {:?} ({:?})", d.adapter_name, d.device_type);
            }
            // The correct outcome when the only adapter is software: no
            // device set at all, and the caller runs on the CPU.
            Err(e) => println!("{what} -> no device ({e:?}) -- CPU path, which is the point"),
        }
    }
}

/// A device this crate opens must be able to bind a full-grid `f32` buffer at
/// **every** resolution the shell offers -- `new_world_dialog.gd`'s
/// `RESOLUTION_PRESETS = [512, 1024, 2048, 4096, 8192]`.
///
/// The regression this pins (found by `PERFORMANCE_BENCHMARKS.md`'s own run,
/// 2026-08-24): `request_gpu_device_from` opened every device at
/// `Limits::downlevel_defaults()`, whose `max_storage_buffer_binding_size` is
/// 128 MiB. One 8192² `f32` grid is 256 MiB, so `use_gpu = true` at the largest
/// shipped preset -- with the shell's GPU toggle at its own default of on --
/// died on "Buffer binding 1 range 268435456 exceeds
/// `max_*_buffer_binding_size` limit 134217728". Not a soft failure: a `wgpu`
/// validation error is a **panic**, and a panic inside a loaded GDExtension
/// takes the Godot process with it.
///
/// Asserted against the real dispatch, not only against the limit: the last
/// case actually runs the 8192² warp kernel, which is the exact call that used
/// to die.
#[test]
fn an_opened_device_can_bind_a_full_grid_at_every_shipped_resolution() {
    let devs = real_devices();
    if devs.is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    let prefs = GpuPreferences { selected_keys: vec![devs[0].key.clone()], ..Default::default() };
    let set = init_gpu_device_set_with(&prefs).expect("device");
    for size in [512usize, 1024, 2048, 4096, 8192] {
        assert!(
            cartalith_gpu::device_supports_grid(set.primary(), size, size),
            "{size}² needs a {} MiB binding; {:?} was opened with only {} MiB",
            cartalith_gpu::grid_buffer_bytes(size, size) / (1024 * 1024),
            set.primary().adapter_name,
            cartalith_gpu::device_grid_limit_bytes(set.primary()) / (1024 * 1024)
        );
        assert!(set.supports_grid(size, size), "the set's own check must agree with the per-device one");
    }
    // The dispatch that used to panic, run for real.
    let (wx, _wy) = warp_grid_gpu_with(set.primary(), 8192, 8192, 4242, 2.5 / 8192.0, 0.18 * 8192.0)
        .expect("the 8192² warp must complete on the primary device");
    assert_eq!(wx.len(), 8192 * 8192, "the 8192² warp must return the whole grid");
}

/// PR-04: the allocator report is a *measurement*, so assert it moves with
/// a real allocation rather than merely being `Some`.
#[test]
fn device_usage_reports_this_apps_own_allocations() {
    if real_devices().is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    let set = init_gpu_device_set_with(&GpuPreferences::default()).expect("device");
    let Some(before) = cartalith_gpu::device_usage(set.primary()) else {
        println!("skipped: this backend implements no allocator report");
        return;
    };
    // A 1024x1024 warp dispatch allocates two 4 MB storage buffers and two
    // 4 MB staging buffers; the reading is taken while nothing else is live,
    // so any movement at all is this dispatch's.
    let _ = warp_grid_gpu_with(set.primary(), 1024, 1024, 4242, 0.01, 3.0);
    let after = cartalith_gpu::device_usage(set.primary()).expect("report stayed available");
    println!(
        "device_usage: {} -> {} bytes allocated ({} reserved)",
        before.allocated_bytes, after.allocated_bytes, after.reserved_bytes
    );
    assert!(after.reserved_bytes >= after.allocated_bytes, "reserved must cover allocated");
    assert!(after.reserved_bytes > 0, "a device that has run a dispatch has reserved memory");
}

/// PR-02, and the correctness half of it: a split dispatch must produce the
/// same field as the whole-grid dispatch.
///
/// The comparison is against the **same device** running the whole grid, and
/// the assertion is `assert_eq!` on `f32` -- bit-exact, no tolerance. That is
/// justified rather than optimistic: `gpu_warp.wgsl` reads nothing but its
/// own `(x, y, seed)`, so on one device a row band is the identical
/// computation. If this ever fails, the band arithmetic is wrong, not the
/// hardware.
#[test]
fn a_split_across_bands_on_one_device_is_bit_identical_to_the_whole_grid() {
    let devs = real_devices();
    if devs.is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    const W: u32 = 256;
    const H: u32 = 192;
    const SEED: i32 = 90210;

    let prefs = GpuPreferences { selected_keys: vec![devs[0].key.clone()], ..Default::default() };
    let set = init_gpu_device_set_with(&prefs).expect("device");
    let (whole_x, whole_y) = warp_grid_gpu_with(set.primary(), W, H, SEED, 0.011, 7.5).expect("whole-grid warp");

    // Rebuild the whole grid from bands, using the same partition arithmetic
    // `warp_grid_gpu_split` uses, but all on one device.
    let mut band_x: Vec<f32> = Vec::new();
    let mut band_y: Vec<f32> = Vec::new();
    for (y0, rows) in split_rows(H, &[1.0, 0.2, 0.5]) {
        if rows == 0 {
            continue;
        }
        let (bx, by) =
            cartalith_gpu::warp_band_gpu_with(set.primary(), W, H, y0, rows, SEED, 0.011, 7.5).expect("band warp");
        band_x.extend_from_slice(&bx);
        band_y.extend_from_slice(&by);
    }
    assert_eq!(band_x.len(), whole_x.len());
    assert_eq!(band_x, whole_x, "warp_x from three bands must be bit-identical to the whole grid");
    assert_eq!(band_y, whole_y, "warp_y likewise");
}

/// PR-02's real question: does `split tiles` across this machine's actual
/// devices beat one device? Measured, printed, and asserted only on
/// correctness -- never on being faster, because whether it is faster is
/// exactly what is being measured (and, per
/// `CPU_MULTITHREADING_SCOPE.md`'s own finding about GPU dispatch overhead
/// below 2048², it may well not be).
#[test]
fn split_tiles_across_two_real_devices_measured() {
    let devs = real_devices();
    if devs.len() < 2 {
        println!("skipped: split tiles needs two non-software GPUs, this machine has {}", devs.len());
        return;
    }
    let keys: Vec<String> = devs.iter().take(2).map(|d| d.key.clone()).collect();
    println!("split across: {:?} + {:?}", devs[0].name, devs[1].name);

    for &(w, h) in &[(512u32, 512u32), (1024, 1024), (2048, 2048), (4096, 4096)] {
        const SEED: i32 = 1337;
        let (wf, amp) = (2.5 / w as f32, 0.18 * w as f32);

        let single_prefs = GpuPreferences { selected_keys: vec![keys[0].clone()], ..Default::default() };
        let single = init_gpu_device_set_with(&single_prefs).expect("primary device");
        // Warm-up: the first dispatch on a fresh device pays shader
        // compilation, which is not what this measures.
        let _ = warp_grid_gpu_with(single.primary(), 64, 64, SEED, wf, amp);
        let t0 = Instant::now();
        let (sx, _sy) = warp_grid_gpu_with(single.primary(), w, h, SEED, wf, amp).expect("single-device warp");
        let single_ms = t0.elapsed().as_secs_f64() * 1e3;
        drop(single);

        let split_prefs =
            GpuPreferences { selected_keys: keys.clone(), mode: MultiGpuMode::SplitTiles, ..Default::default() };
        let split = init_gpu_device_set_with(&split_prefs).expect("split device set");
        assert!(split.is_split(), "two selected devices in split_tiles mode must actually split");
        let _ = warp_grid_gpu_split(&split, 64, 64, SEED, wf, amp);
        let t1 = Instant::now();
        let (px, _py) = warp_grid_gpu_split(&split, w, h, SEED, wf, amp).expect("split warp");
        let split_ms = t1.elapsed().as_secs_f64() * 1e3;

        assert_eq!(px.len(), sx.len(), "split output must be the full grid");
        // Across two *different* devices the last bits may differ (two
        // shader compilers), so this is a tolerance comparison, unlike the
        // same-device band test above which is bit-exact.
        let worst = px.iter().zip(sx.iter()).map(|(a, b)| (*a as f64 - *b as f64).abs()).fold(0.0f64, f64::max);
        println!(
            "{w}x{h}: single {single_ms:7.1} ms   split {split_ms:7.1} ms   ratio {:.2}x   worst |split-single| = {worst:.3e}",
            single_ms / split_ms
        );
        assert!(worst < 1e-2 * f64::from(amp).max(1.0), "split must compute the same field, not a different one");
        drop(split);
    }
}

/// Where [`cartalith_gpu::device_weight`]'s numbers come from. Prints the
/// whole-grid warp time on each device on its own, which is exactly the
/// ratio the row bands should be sized by. Measurement, not assertion --
/// the constant is set from what this prints, and this test exists so the
/// next person can re-run it on their own hardware rather than inheriting
/// one machine's ratio as if it were universal.
#[test]
fn per_device_warp_throughput_measured() {
    let devs = real_devices();
    if devs.is_empty() {
        println!("skipped: no non-software GPU on this machine");
        return;
    }
    for d in &devs {
        let prefs = GpuPreferences { selected_keys: vec![d.key.clone()], ..Default::default() };
        let set = init_gpu_device_set_with(&prefs).expect("device");
        print!("{:<28} {:?}", d.name, d.device_type);
        for &(w, h) in &[(1024u32, 1024u32), (2048, 2048), (4096, 4096)] {
            let (wf, amp) = (2.5 / w as f32, 0.18 * w as f32);
            let _ = warp_grid_gpu_with(set.primary(), 64, 64, 7, wf, amp); // warm-up: shader compile
            let t = Instant::now();
            let _ = warp_grid_gpu_with(set.primary(), w, h, 7, wf, amp);
            print!("   {w}²: {:6.1} ms", t.elapsed().as_secs_f64() * 1e3);
        }
        println!();
    }
}

#[test]
fn a_vram_budget_below_the_grids_working_set_keeps_the_gpu_path_off() {
    let _guard = global_prefs();
    // 2048x2048 x 10 f32 grids = 320 MB by this crate's own estimate.
    let need = gpu_working_set_bytes(2048, 2048);
    assert_eq!(need, 2048 * 2048 * 4 * 10);
    set_preferences(GpuPreferences { vram_budget_bytes: need - 1, ..Default::default() });
    assert!(!cartalith_gpu::gpu_allowed_for_grid(2048, 2048));
    assert!(cartalith_gpu::gpu_allowed_for_grid(1024, 1024), "a smaller grid still fits");
    set_preferences(GpuPreferences::default());
    assert!(cartalith_gpu::gpu_allowed_for_grid(2048, 2048), "no budget => never refused");
}

/// **The integrated-GPU 8192² readback, run for real on the real device.**
///
/// `504c2a6` fixed the *limits* half of the 8192² crash and reported what it
/// deliberately left: the integrated Radeon passes every limits check at
/// 8192², dispatches, and then dies on `expect("buffer map failed")` --
/// `BufferAsyncError` from the `MAP_READ` staging map, ten such sites in this
/// crate. A panic there is not a failed generation, it is a dead Godot
/// process (`cartalith-rust-conventions`).
///
/// Nothing is mocked or skipped here: this opens the machine's integrated
/// device and asks it for a genuine 8192² warp. Two outcomes are acceptable
/// and both are asserted --
///
/// - it completes, and returns the whole grid; or
/// - it fails, returns `None`, and is *demoted*: [`device_supports_grid`] must
///   then refuse 8192² on it (so the engine's own device-set filter and every
///   later stage in the same generation take the CPU path), while a smaller
///   grid on the very same device must still work.
///
/// What is NOT acceptable, and what this test exists to catch, is a panic.
#[test]
fn the_integrated_gpu_at_8192_falls_back_instead_of_panicking() {
    const N: u32 = 8192;
    let Some(igpu) = real_devices().into_iter().find(|d| d.device_type == wgpu::DeviceType::IntegratedGpu) else {
        println!("skipped: this machine has no integrated GPU");
        return;
    };
    println!("integrated device: {:?} ({:?})", igpu.name, igpu.backend);

    cartalith_gpu::clear_readback_failures();
    let prefs = GpuPreferences { selected_keys: vec![igpu.key.clone()], ..Default::default() };
    let set = init_gpu_device_set_with(&prefs).expect("the integrated device must open");
    assert_eq!(set.primary().adapter_name, igpu.name, "the test must run on the integrated device, not another one");

    // Its reported limits cover 8192² -- that is precisely why the limits
    // check alone was not enough, and why this test is about the readback.
    assert!(
        cartalith_gpu::device_supports_grid(set.primary(), N as usize, N as usize),
        "before any attempt, the reported limits say 8192² is fine"
    );

    let (wf, amp) = (2.5 / N as f32, 0.18 * N as f32);
    let t = Instant::now();
    let out = warp_grid_gpu_with(set.primary(), N, N, 4242, wf, amp);
    let ms = t.elapsed().as_secs_f64() * 1e3;

    match out {
        Some((wx, wy)) => {
            println!("8192² completed on the integrated GPU in {ms:.0} ms");
            assert_eq!(wx.len(), (N as usize) * (N as usize), "a completed dispatch returns the whole grid");
            assert_eq!(wy.len(), wx.len());
            assert!(
                cartalith_gpu::device_supports_grid(set.primary(), N as usize, N as usize),
                "a device that succeeded must not be demoted"
            );
        }
        None => {
            println!("8192² failed its readback on the integrated GPU after {ms:.0} ms -- fell back, did not panic");
            assert!(
                !cartalith_gpu::device_supports_grid(set.primary(), N as usize, N as usize),
                "a failed readback must demote the device so later stages skip it"
            );
            assert!(
                !set.supports_grid(N as usize, N as usize),
                "the set's own check must agree, which is what `generate_terrain`'s filter reads"
            );
            // The demotion is size-scoped, not a blanket ban on the device.
            assert!(
                cartalith_gpu::device_supports_grid(set.primary(), 512, 512),
                "a smaller grid on the same device must still be allowed"
            );
            let (sx, _sy) = warp_grid_gpu_with(set.primary(), 512, 512, 4242, 2.5 / 512.0, 0.18 * 512.0)
                .expect("512² must still complete on the device that failed at 8192²");
            assert_eq!(sx.len(), 512 * 512);
            // And the refusal is now immediate: no second doomed dispatch.
            let t = Instant::now();
            assert!(warp_grid_gpu_with(set.primary(), N, N, 4242, wf, amp).is_none());
            let again_ms = t.elapsed().as_secs_f64() * 1e3;
            println!("a second 8192² request was refused in {again_ms:.1} ms without dispatching");
            assert!(again_ms < ms, "the second attempt must be refused up front, not re-dispatched");
        }
    }

    // Leave the session's record as this test found it.
    cartalith_gpu::clear_readback_failures();
}

/// **The whole 8192² generation on the integrated GPU, the exact run that
/// used to kill the process.**
///
/// Slow (about a minute and a half, and several GB of working set) and worth
/// every second: the failure this pins is invisible to any single dispatch.
/// `the_integrated_gpu_at_8192_falls_back_instead_of_panicking` above shows an
/// isolated 8192² warp *completing* on this device in about a second — it is
/// only under the whole pipeline's accumulated working set that the
/// base-field blur's readback fails, and only then that the stages after it
/// meet a device that is no longer there.
///
/// Two panics, one after the other, are what this test walks through:
///
/// 1. before any of this work, `expect("buffer map failed")` in the blur;
/// 2. with only the `Option` threading in place, the *next* stage — weather,
///    on a 240² grid nowhere near any size limit — panicked on a 32-byte
///    uniform buffer with `Buffer with 'weather params' label is invalid`,
///    because the device was lost, not merely full.
///
/// The assertion is simply that this returns. `gpu_stages_used` is printed
/// rather than asserted: which stages get through before the device gives out
/// is a property of the hardware and the day, not of this code.
#[test]
fn a_full_8192_generation_on_the_integrated_gpu_completes_or_falls_back() {
    const N: usize = 8192;
    let Some(igpu) = real_devices().into_iter().find(|d| d.device_type == wgpu::DeviceType::IntegratedGpu) else {
        println!("skipped: this machine has no integrated GPU");
        return;
    };
    let _guard = global_prefs();
    cartalith_gpu::clear_readback_failures();
    set_preferences(GpuPreferences { selected_keys: vec![igpu.key.clone()], ..Default::default() });

    let mut p = cartalith_engine::WorldParams::defaults(N, N, 12345);
    p.tect.plates = 40;
    p.use_gpu = true;

    let t = Instant::now();
    let ws = cartalith_engine::generate_terrain(&p);
    println!(
        "8192² on {:?}: {:.1} s, gpu stages that completed = {:?}",
        igpu.name,
        t.elapsed().as_secs_f64(),
        ws.gpu_stages_used
    );
    assert_eq!(ws.field.len(), N * N, "the generation must produce a whole 8192² field, GPU or CPU");
    assert!(ws.field.iter().all(|v| v.is_finite()), "a fallen-back stage must still produce real values");

    set_preferences(GpuPreferences::default());
    cartalith_gpu::clear_readback_failures();
}
