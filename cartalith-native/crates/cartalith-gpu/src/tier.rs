//! `HARDWARE_ACCELERATION.md` §4: the hardware capability tier.
//!
//! `GPU_COMPUTE_PILOT_SCOPE.md` deferred this "once ≥2 real kernels exist and
//! their actual differing requirements are known". They do now, so every
//! threshold below is **what this crate's own kernels and device request
//! need**, not a guess about GPUs in general:
//!
//! | Input §4 names | Used here as |
//! |---|---|
//! | GPU availability, device type | no device / software rasterizer → [`ComputeTier::CpuOnly`]; discrete vs. everything else splits High from Standard |
//! | supported compute features | `DownlevelFlags::COMPUTE_SHADERS` ([`GpuDeviceInfo::supports_compute`]). Every production kernel requests `Features::empty()`, so no optional feature is a requirement |
//! | storage-buffer capabilities | `max_storage_buffers_per_shader_stage` ≥ `REUSED_STAGE_MAX_STORAGE_BUFFERS` (8), what the shared device is opened with — below it `request_device` fails and the engine takes the CPU path anyway |
//! | workgroup limits | [`KERNEL_WORKGROUP_X`] / [`KERNEL_WORKGROUP_Y`] / [`KERNEL_WORKGROUP_INVOCATIONS`], the largest `@workgroup_size` any shader in `shaders/` declares (a test re-derives them from the files) |
//! | maximum buffer sizes | whether one full-grid `f32` buffer at [`LARGEST_PRESET_GRID`]² fits `min(max_storage_buffer_binding_size, max_buffer_size)` — [`crate::device_grid_limit_bytes`]'s arithmetic, from enumeration data instead of a live device |
//! | memory pressure | the only real signal `wgpu` 30 gives: a readback failure this session ([`crate::readback_failure_cells`]). There is no VRAM query (see `multi.rs`'s module doc) |
//! | mobile vs desktop | a mobile build is capped at [`ComputeTier::GpuBasic`] (§21: "not miniature desktop computers") |
//! | backend, vendor, device name | **not used**, deliberately: §29 forbids branching on backend and §4 on model-name matching. They stay visible on [`GpuDeviceInfo`] |
//! | texture/storage-texture caps | **not used**: no kernel in `shaders/` binds a texture |
//! | CPU core count, system memory | **not used** — see [`classify`] |

use crate::{GpuDeviceInfo, REUSED_STAGE_MAX_STORAGE_BUFFERS, enumerate_devices, preferences, readback_failure_cells};

/// Ordered worst to best, so `>=` reads as "at least this capable".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComputeTier {
    /// No device the GPU path can run on. Everything takes the CPU/Rayon path.
    CpuOnly,
    /// Runs every kernel, but will not reach the largest grid the shell
    /// offers (limits too small, or a readback already failed below it), or
    /// is a mobile build.
    GpuBasic,
    /// Reaches the largest grid, on a device that is not discrete — shares
    /// system memory, so the limit it reports is a promise rather than
    /// dedicated VRAM (this machine's integrated Radeon reports 2047 MiB and
    /// still failed a readback at 8192²; see [`crate::device_supports_grid`]).
    GpuStandard,
    /// Discrete, desktop, reaches the largest grid.
    GpuHigh,
}

/// Largest `@workgroup_size` X any kernel in `shaders/` declares (the four
/// `64, 1, 1` affordance kernels).
pub const KERNEL_WORKGROUP_X: u32 = 64;
/// Largest `@workgroup_size` Y (every `8, 8, 1` kernel).
pub const KERNEL_WORKGROUP_Y: u32 = 8;
/// Largest `@workgroup_size` product (both shapes are 64).
pub const KERNEL_WORKGROUP_INVOCATIONS: u32 = 64;

/// The largest grid side `new_world_dialog.gd` offers (`GRID_MAX`, and the
/// top of `RESOLUTION_PRESETS`). A test reads that file so the two cannot drift.
pub const LARGEST_PRESET_GRID: usize = 8192;

/// Whether the GPU path can run on this device at all.
fn runs_every_kernel(d: &GpuDeviceInfo) -> bool {
    let l = &d.limits;
    !d.is_software
        && d.supports_compute
        && l.max_storage_buffers_per_shader_stage >= REUSED_STAGE_MAX_STORAGE_BUFFERS
        && l.max_compute_workgroup_size_x >= KERNEL_WORKGROUP_X
        && l.max_compute_workgroup_size_y >= KERNEL_WORKGROUP_Y
        && l.max_compute_invocations_per_workgroup >= KERNEL_WORKGROUP_INVOCATIONS
}

/// Whether one full-grid buffer at `side`² fits the device's limits and no
/// readback has failed at or below that size this session.
fn reaches_grid(d: &GpuDeviceInfo, side: usize) -> bool {
    let cells = (side as u64) * (side as u64);
    let fits = crate::grid_buffer_bytes(side, side)
        <= d.limits.max_storage_buffer_binding_size.min(d.limits.max_buffer_size);
    fits && readback_failure_cells(&d.name, d.vendor, d.backend).is_none_or(|failed_at| cells < failed_at)
}

/// Classify one device (`None` = no device). Pure apart from the readback
/// record, so it is testable with no GPU.
///
/// CPU core count and system memory are not inputs: they cannot move a
/// device between GPU tiers, and the CPU path's parallelism is already set by
/// `cartalith-engine`'s thread pool. System memory also has no std query —
/// reading it would need a new dependency for a value nothing here would use.
#[must_use]
pub fn classify(dev: Option<&GpuDeviceInfo>, mobile: bool) -> ComputeTier {
    let Some(d) = dev.filter(|d| runs_every_kernel(d)) else {
        return ComputeTier::CpuOnly;
    };
    if mobile || !reaches_grid(d, LARGEST_PRESET_GRID) {
        ComputeTier::GpuBasic
    } else if d.device_type == wgpu::DeviceType::DiscreteGpu {
        ComputeTier::GpuHigh
    } else {
        ComputeTier::GpuStandard
    }
}

/// Whether this build targets a mobile OS.
pub const IS_MOBILE: bool = cfg!(any(target_os = "android", target_os = "ios"));

/// The tier of the device the GPU path would open: the first selected key
/// that still enumerates, otherwise the first non-software device in
/// [`enumerate_devices`]' order (discrete first — the same class order
/// `PowerPreference::HighPerformance` resolves by).
#[must_use]
pub fn compute_tier() -> ComputeTier {
    let devs = enumerate_devices();
    let chosen = preferences()
        .selected_keys
        .first()
        .and_then(|k| devs.iter().find(|d| &d.key == k))
        .or_else(|| devs.iter().find(|d| !d.is_software));
    classify(chosen, IS_MOBILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1024 * 1024;

    fn dev(t: wgpu::DeviceType, binding: u64) -> GpuDeviceInfo {
        let limits =
            wgpu::Limits { max_storage_buffer_binding_size: binding, max_buffer_size: binding, ..Default::default() };
        GpuDeviceInfo {
            key: String::new(),
            name: "cartalith tier pseudo-adapter".into(),
            vendor: 0x7e57,
            device_id: 1,
            device_type: t,
            backend: wgpu::Backend::Noop,
            alternate_backends: Vec::new(),
            driver: String::new(),
            driver_info: String::new(),
            max_buffer_size: binding,
            max_storage_buffer_binding_size: binding,
            supports_compute: true,
            is_software: t == wgpu::DeviceType::Cpu,
            limits,
        }
    }

    use wgpu::DeviceType::{Cpu, DiscreteGpu, IntegratedGpu, Other, VirtualGpu};

    #[test]
    fn tiers_by_device_class_and_mobility() {
        let big = 2048 * MIB;
        assert_eq!(classify(None, false), ComputeTier::CpuOnly);
        assert_eq!(classify(Some(&dev(Cpu, big)), false), ComputeTier::CpuOnly);
        assert_eq!(classify(Some(&dev(DiscreteGpu, big)), false), ComputeTier::GpuHigh);
        assert_eq!(classify(Some(&dev(DiscreteGpu, big)), true), ComputeTier::GpuBasic);
        for t in [IntegratedGpu, VirtualGpu, Other] {
            assert_eq!(classify(Some(&dev(t, big)), false), ComputeTier::GpuStandard, "{t:?}");
        }
        assert!(ComputeTier::CpuOnly < ComputeTier::GpuBasic && ComputeTier::GpuStandard < ComputeTier::GpuHigh);
    }

    /// The largest grid is 8192², one f32 buffer of which is exactly 256 MiB.
    /// One byte less and the device falls to Basic.
    #[test]
    fn buffer_ceiling_boundary_is_one_8192_squared_f32_grid() {
        assert_eq!(classify(Some(&dev(DiscreteGpu, 256 * MIB)), false), ComputeTier::GpuHigh);
        assert_eq!(classify(Some(&dev(DiscreteGpu, 256 * MIB - 1)), false), ComputeTier::GpuBasic);
        // `downlevel_defaults()`' 128 MiB binding is a Basic device.
        assert_eq!(classify(Some(&dev(DiscreteGpu, 128 * MIB)), false), ComputeTier::GpuBasic);
        // min of the two limits binds, not either alone.
        let mut d = dev(DiscreteGpu, 2048 * MIB);
        d.limits.max_buffer_size = 256 * MIB - 1;
        assert_eq!(classify(Some(&d), false), ComputeTier::GpuBasic);
    }

    #[test]
    fn each_kernel_requirement_is_a_hard_floor() {
        let ok = dev(DiscreteGpu, 2048 * MIB);
        type Break = fn(&mut GpuDeviceInfo);
        let cases: [(&str, Break); 5] = [
            ("no compute", |d| d.supports_compute = false),
            ("7 storage buffers", |d| d.limits.max_storage_buffers_per_shader_stage = 7),
            ("workgroup x 63", |d| d.limits.max_compute_workgroup_size_x = 63),
            ("workgroup y 7", |d| d.limits.max_compute_workgroup_size_y = 7),
            ("63 invocations", |d| d.limits.max_compute_invocations_per_workgroup = 63),
        ];
        for (why, f) in cases {
            let mut d = ok.clone();
            f(&mut d);
            assert_eq!(classify(Some(&d), false), ComputeTier::CpuOnly, "{why}");
        }
        // Exactly at each floor still runs.
        let mut d = ok;
        d.limits.max_storage_buffers_per_shader_stage = 8;
        d.limits.max_compute_workgroup_size_x = 64;
        d.limits.max_compute_workgroup_size_y = 8;
        d.limits.max_compute_invocations_per_workgroup = 64;
        assert_eq!(classify(Some(&d), false), ComputeTier::GpuHigh);
    }

    /// Memory pressure: a readback that failed at the largest grid demotes the
    /// device; one that failed only above it does not.
    #[test]
    fn a_readback_failure_at_the_largest_grid_demotes() {
        let _g = crate::multi::tests::readback_test_guard();
        let mut d = dev(DiscreteGpu, 2048 * MIB);
        d.vendor = 0x7e58; // own key, so no other test's record can reach it
        let side = LARGEST_PRESET_GRID as u64;
        crate::note_readback_failure(&d.name, d.vendor, d.backend, side * side + 1);
        assert_eq!(classify(Some(&d), false), ComputeTier::GpuHigh, "failed only above the largest grid");
        crate::note_readback_failure(&d.name, d.vendor, d.backend, side * side);
        assert_eq!(classify(Some(&d), false), ComputeTier::GpuBasic);
        crate::clear_readback_failures();
    }

    /// The constants against the files they mirror, not against themselves.
    #[test]
    fn constants_match_the_shaders_and_the_new_world_dialog() {
        let (mut x, mut y, mut inv) = (0, 0, 0);
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/shaders");
        for e in std::fs::read_dir(dir).expect("shaders dir") {
            let src = std::fs::read_to_string(e.expect("entry").path()).expect("shader");
            for part in src.split("@workgroup_size(").skip(1) {
                let dims: Vec<u32> =
                    part.split(')').next().unwrap().split(',').map(|s| s.trim().parse().unwrap()).collect();
                let get = |i: usize| dims.get(i).copied().unwrap_or(1);
                x = x.max(get(0));
                y = y.max(get(1));
                inv = inv.max(get(0) * get(1) * get(2));
            }
        }
        assert_eq!((x, y, inv), (64, 8, 64), "a shader's workgroup changed: update the constants");
        assert_eq!((KERNEL_WORKGROUP_X, KERNEL_WORKGROUP_Y, KERNEL_WORKGROUP_INVOCATIONS), (x, y, inv));

        let gd = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../godot-project/shell/new_world_dialog.gd"
        ))
        .expect("new_world_dialog.gd");
        assert!(gd.contains("const GRID_MAX := 8192"), "GRID_MAX moved: update LARGEST_PRESET_GRID");
        assert_eq!(LARGEST_PRESET_GRID, 8192);
        assert_eq!(REUSED_STAGE_MAX_STORAGE_BUFFERS, 8);
    }

    /// Runs with or without a GPU: prints what this machine is, and holds the
    /// one invariant that needs no hardware knowledge.
    #[test]
    fn compute_tier_on_this_machine() {
        let tier = compute_tier();
        println!("compute tier: {tier:?}");
        if enumerate_devices().iter().all(|d| d.is_software) {
            assert_eq!(tier, ComputeTier::CpuOnly);
        }
    }
}
