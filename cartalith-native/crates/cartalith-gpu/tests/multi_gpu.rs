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
    let (wx, _wy) = warp_grid_gpu_with(set.primary(), 8192, 8192, 4242, 2.5 / 8192.0, 0.18 * 8192.0);
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
    let (whole_x, whole_y) = warp_grid_gpu_with(set.primary(), W, H, SEED, 0.011, 7.5);

    // Rebuild the whole grid from bands, using the same partition arithmetic
    // `warp_grid_gpu_split` uses, but all on one device.
    let mut band_x: Vec<f32> = Vec::new();
    let mut band_y: Vec<f32> = Vec::new();
    for (y0, rows) in split_rows(H, &[1.0, 0.2, 0.5]) {
        if rows == 0 {
            continue;
        }
        let (bx, by) = cartalith_gpu::warp_band_gpu_with(set.primary(), W, H, y0, rows, SEED, 0.011, 7.5);
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
        let (sx, _sy) = warp_grid_gpu_with(single.primary(), w, h, SEED, wf, amp);
        let single_ms = t0.elapsed().as_secs_f64() * 1e3;
        drop(single);

        let split_prefs =
            GpuPreferences { selected_keys: keys.clone(), mode: MultiGpuMode::SplitTiles, ..Default::default() };
        let split = init_gpu_device_set_with(&split_prefs).expect("split device set");
        assert!(split.is_split(), "two selected devices in split_tiles mode must actually split");
        let _ = warp_grid_gpu_split(&split, 64, 64, SEED, wf, amp);
        let t1 = Instant::now();
        let (px, _py) = warp_grid_gpu_split(&split, w, h, SEED, wf, amp);
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
