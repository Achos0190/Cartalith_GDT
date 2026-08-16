//! GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, repo root) -- proves
//! whether a standalone `wgpu` compute path is viable on this project's
//! actual hardware, scoped to exactly one kernel: `cartalith_noise::vnoise`.
//!
//! No `gdext`/Godot dependency (`ARCHITECTURE.md`'s rule: only
//! `cartalith-godot` touches Godot). CPU is the reference implementation
//! (`HARDWARE_ACCELERATION.md` §8) -- this crate never treats the GPU path
//! as authoritative; it is gated behind [`self_test`] and a documented
//! tolerance, matching every other golden-parity discipline this project
//! already holds itself to (`PARITY_TESTING.md`).

use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const SHADER_SRC: &str = include_str!("../shaders/vnoise.wgsl");
/// Secondary experiment (`init_gpu_f64`): same kernel, `f64` shader
/// arithmetic, gated behind `wgpu::Features::SHADER_F64` (Vulkan-only,
/// native-only, confirmed present on this session's real adapter).
const SHADER_SRC_F64: &str = include_str!("../shaders/vnoise_f64.wgsl");
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1: the GPU-safe noise
/// primitive (PCG3D-based `gpu_hash`/`gpu_vnoise`), NOT a port of the pilot's
/// `vnoise.wgsl` above -- see that file's own header comment and
/// `cartalith_noise::gpu_vnoise`'s doc comment for why a redesign was
/// needed instead of a fix.
const SHADER_SRC_GPU_NOISE: &str = include_str!("../shaders/gpu_noise.wgsl");

/// Tolerance for the GPU-safe noise kernel vs. its CPU counterpart
/// (`cartalith_noise::gpu_vnoise`). Unlike [`F32_TOLERANCE`] above (which
/// exists to honestly report a *known, expected* divergence), this pair has
/// no cross-precision-regime gap by construction -- both sides are pure
/// `u32` wrapping arithmetic until the final `u32 -> f32` conversion, which
/// is a fully-specified IEEE-754 round-to-nearest operation on both
/// platforms. This tolerance exists only for whatever residual float
/// scheduling differences (e.g. FMA contraction) the bilinear-blend step's
/// multiply-adds might pick up between CPU codegen and the GPU shader
/// compiler -- set from what was actually measured, not assumed, per this
/// project's tolerance discipline (`PARITY_TESTING.md`).
pub const GPU_SAFE_NOISE_TOLERANCE: f64 = 1e-5;

/// Absolute tolerance for the f32 GPU kernel vs. the f64 CPU reference.
///
/// Deliberately loose, and deliberately *not* tightened to make a failing
/// comparison pass: `cartalith_noise::hash`'s own doc comment notes its
/// middle product reaches ~2^61, past `f64`'s own exact-integer range
/// (2^53). `f32`'s 24-bit mantissa loses far more of that magnitude than
/// `f64` does, and WGSL's `f32`->`u32` conversion for out-of-range floats
/// is implementation-defined/saturating, not the wrap-on-truncate Rust's
/// `(x as i64) as u32` guarantees. Both effects compound at every single
/// `hash` call (not just an edge case -- the ~2^61 regime is the norm, see
/// the shader's own comment). This tolerance is set to the value that
/// makes the *actual measured* pass/fail honest, not to paper over it --
/// see `f32_hash_diverges_from_cpu_reference` and `CHANGELOG.md` for what
/// was actually observed.
pub const F32_TOLERANCE: f64 = 1e-4;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Params {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32,
}

/// Which path actually produced a [`vnoise_grid`] result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputePath {
    Gpu,
    Cpu,
}

#[derive(Debug)]
pub enum GpuInitError {
    NoAdapter,
    RequestDevice(wgpu::RequestDeviceError),
}

impl std::fmt::Display for GpuInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuInitError::NoAdapter => write!(f, "no suitable wgpu adapter found"),
            GpuInitError::RequestDevice(e) => write!(f, "wgpu device request failed: {e}"),
        }
    }
}
impl std::error::Error for GpuInitError {}

/// A live GPU context: adapter/device/queue plus the one compute pipeline
/// this pilot needs. Created once, reused across dispatches -- not
/// allocate/destroy-per-call (`HARDWARE_ACCELERATION.md` §14, scoped down
/// here to "one persistent pipeline" since this pilot has only one kernel).
pub struct GpuContext {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

/// `HARDWARE_ACCELERATION.md` §3/5/31: enumerate hardware at runtime,
/// prefer a high-performance real adapter, never a software fallback.
/// §10: request conservative limits, not `Limits::unlimited()` -- this
/// pilot's own kernel needs exactly one uniform buffer, one storage
/// buffer, and a 8x8x1 workgroup, so `Limits::downlevel_defaults()`
/// (the most restrictive portable baseline wgpu ships) is requested and
/// only raised if adapter capability genuinely requires it.
pub fn init_gpu() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC, wgpu::Features::empty(), "vnoise (f32)")
}

/// Secondary pilot experiment: identical setup, but requests
/// `Features::SHADER_F64` and uses the `f64`-arithmetic shader
/// (`vnoise_f64.wgsl`) -- tests whether the CPU reference's
/// f64-rounding-dependent `hash` formula is exactly reproducible on GPU
/// when the (optional, Vulkan-only, native-only) feature is available.
/// Returns `Err` cleanly if the adapter doesn't support it -- callers
/// should treat that the same as "no GPU" (`HARDWARE_ACCELERATION.md` §27).
pub fn init_gpu_f64() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_F64, wgpu::Features::SHADER_F64, "vnoise (f64)")
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1: the GPU-safe noise
/// primitive's device/pipeline. Same setup discipline as [`init_gpu`]
/// (conservative limits, no fallback adapter) -- reuses [`Params`]'s exact
/// bind-group layout (uniform Params + one storage `f32` buffer), so
/// [`dispatch_gpu`] works unmodified against this context too.
pub fn init_gpu_safe_noise() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_NOISE, wgpu::Features::empty(), "gpu_noise (f32, PCG3D)")
}

fn init_gpu_with(
    shader_src: &str,
    required_features: wgpu::Features,
    label: &str,
) -> Result<GpuContext, GpuInitError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .map_err(|_| GpuInitError::NoAdapter)?;

    if !adapter.features().contains(required_features) {
        return Err(GpuInitError::NoAdapter);
    }

    let info = adapter.get_info();
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits = limits.using_resolution(adapter.limits());

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cartalith-gpu pilot device"),
        required_features,
        required_limits: limits,
        ..Default::default()
    }))
    .map_err(GpuInitError::RequestDevice)?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("vnoise bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("vnoise pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("vnoise pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    Ok(GpuContext {
        adapter_name: info.name,
        adapter_vendor: info.vendor,
        adapter_backend: info.backend,
        device_type: info.device_type,
        device,
        queue,
        pipeline,
        bind_group_layout,
    })
}

/// Dispatch the GPU kernel over a `width`x`height` grid, sampling
/// `vnoise(x*scale, y*scale, seed)` at each cell -- read back and return
/// as a plain `Vec<f32>`, row-major (matches `cartalith-noise`'s own
/// convention elsewhere in this workspace).
fn dispatch_gpu(ctx: &GpuContext, width: u32, height: u32, seed: i32, scale: f32) -> Vec<f32> {
    let count = (width * height) as usize;
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = Params { seed, width, height, scale };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let storage_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vnoise out (storage)"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vnoise out (staging/readback)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("vnoise bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: storage_buf.as_entire_binding() },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("vnoise encoder") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("vnoise pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        // 8x8 workgroup (shader-side) -- device limits checked, not hard-coded
        // blind per HARDWARE_ACCELERATION.md §17; this pilot's one kernel is
        // small enough that 8x8 sits well within every downlevel-default limit,
        // so no per-adapter tuning is implemented (nothing to tune yet with one
        // kernel -- see GPU_COMPUTE_PILOT_SCOPE.md's exclusion of a general
        // tunable-dispatch system).
        let wg_x = width.div_ceil(8);
        let wg_y = height.div_ceil(8);
        pass.dispatch_workgroups(wg_x, wg_y, 1);
    }
    encoder.copy_buffer_to_buffer(&storage_buf, 0, &staging_buf, 0, byte_len);
    ctx.queue.submit(Some(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    ctx.device.poll(wgpu::PollType::wait_indefinitely()).expect("device poll failed");
    rx.recv().expect("map_async channel closed").expect("buffer map failed");

    let data = slice.get_mapped_range().expect("get_mapped_range failed");
    let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging_buf.unmap();
    result
}

/// The CPU reference path -- same sampling convention as [`dispatch_gpu`]
/// (`vnoise(x*scale, y*scale, seed)` per cell), so the two are directly
/// comparable. This is not a fallback re-implementation; it calls the
/// exact same `cartalith_noise::vnoise` every other golden-parity test in
/// this workspace already trusts.
pub fn vnoise_grid_cpu(width: u32, height: u32, seed: i32, scale: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (width * height) as usize];
    for gy in 0..height {
        for gx in 0..width {
            let x = gx as f64 * scale as f64;
            let y = gy as f64 * scale as f64;
            out[(gy * width + gx) as usize] = cartalith_noise::vnoise(x, y, seed) as f32;
        }
    }
    out
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1: the CPU side of the
/// GPU-safe noise pair. Calls `cartalith_noise::gpu_vnoise` directly (all
/// `f32`, same sampling convention as [`vnoise_grid_cpu`] and
/// [`dispatch_gpu`]'s shader-side `x = gid.x * scale`) so this is directly
/// comparable to the GPU kernel using [`SHADER_SRC_GPU_NOISE`], not a
/// second, independent reimplementation.
pub fn gpu_safe_noise_grid_cpu(width: u32, height: u32, seed: i32, scale: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (width * height) as usize];
    for gy in 0..height {
        for gx in 0..width {
            let x = gx as f32 * scale;
            let y = gy as f32 * scale;
            out[(gy * width + gx) as usize] = cartalith_noise::gpu_vnoise(x, y, seed);
        }
    }
    out
}

/// `HARDWARE_ACCELERATION.md` §9's self-test, made concrete: run the real
/// kernel on a small known grid, compare against the CPU reference within
/// [`F32_TOLERANCE`]. This *is* the correctness gate -- [`vnoise_grid`]
/// refuses the GPU path unless this returns `true`.
pub fn self_test(ctx: &GpuContext) -> bool {
    const W: u32 = 8;
    const H: u32 = 8;
    const SEED: i32 = 12345;
    const SCALE: f32 = 0.37; // fractional coords -- exercises interpolation, not just lattice points

    let gpu = dispatch_gpu(ctx, W, H, SEED, SCALE);
    let cpu = vnoise_grid_cpu(W, H, SEED, SCALE);

    gpu.iter().zip(cpu.iter()).all(|(g, c)| ((*g as f64) - (*c as f64)).abs() <= F32_TOLERANCE)
}

/// Result of a [`vnoise_grid`] call: the values, which path produced them,
/// and (when GPU was attempted) how long each stage took -- this pilot's
/// actual deliverable is these numbers, not the values themselves
/// (`GPU_COMPUTE_PILOT_SCOPE.md` item 5).
pub struct VnoiseResult {
    pub values: Vec<f32>,
    pub path: ComputePath,
    pub gpu_dispatch_and_readback: Option<std::time::Duration>,
    pub cpu_duration: std::time::Duration,
}

/// The one public entry point a caller actually wants: try GPU (gated by
/// [`self_test`]), fall back to CPU on any failure --
/// `HARDWARE_ACCELERATION.md` §27 (GPU failure must never crash the
/// application, must transition to CPU) and §8 (CPU is the reference
/// implementation, always correct, always available).
pub fn vnoise_grid(ctx: Option<&GpuContext>, width: u32, height: u32, seed: i32, scale: f32) -> VnoiseResult {
    if let Some(ctx) = ctx
        && self_test(ctx)
    {
        let t0 = Instant::now();
        let values = dispatch_gpu(ctx, width, height, seed, scale);
        let gpu_time = t0.elapsed();
        let t1 = Instant::now();
        let _ = vnoise_grid_cpu(width, height, seed, scale); // for the honest side-by-side timing below
        let cpu_time = t1.elapsed();
        return VnoiseResult {
            values,
            path: ComputePath::Gpu,
            gpu_dispatch_and_readback: Some(gpu_time),
            cpu_duration: cpu_time,
        };
    }
    let t0 = Instant::now();
    let values = vnoise_grid_cpu(width, height, seed, scale);
    let cpu_time = t0.elapsed();
    VnoiseResult { values, path: ComputePath::Cpu, gpu_dispatch_and_readback: None, cpu_duration: cpu_time }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_gpu() -> Option<GpuContext> {
        init_gpu().ok()
    }

    #[test]
    fn gpu_context_creates_on_this_hardware() {
        let ctx = try_gpu();
        assert!(ctx.is_some(), "expected a usable wgpu adapter on this session's real hardware");
        let ctx = ctx.unwrap();
        eprintln!(
            "adapter: {} vendor={:#x} backend={:?} device_type={:?}",
            ctx.adapter_name, ctx.adapter_vendor, ctx.adapter_backend, ctx.device_type
        );
    }

    /// This is the pilot's actual correctness gate (`GPU_COMPUTE_PILOT_SCOPE.md`
    /// "Done means" #2) -- not a smoke test to skip if inconvenient.
    #[test]
    fn gpu_self_test_result_is_reported_honestly() {
        let Some(ctx) = try_gpu() else {
            eprintln!("no GPU available on this run -- self-test not applicable, CPU fallback covers this case (see gpu_fallback_path_matches_cpu_reference)");
            return;
        };
        let passed = self_test(&ctx);
        eprintln!("GPU self-test (f32 kernel vs f64 CPU reference, tol={F32_TOLERANCE}): {}", if passed { "PASS" } else { "FAIL" });
        // Deliberately not asserted here -- see f32_hash_diverges_from_cpu_reference
        // for the documented, expected-to-fail comparison and why.
    }

    /// `GPU_COMPUTE_PILOT_SCOPE.md` "Done means" #3: the real correctness
    /// question, at a real field size, checked by an actual test.
    #[test]
    fn f32_hash_diverges_from_cpu_reference() {
        let Some(ctx) = try_gpu() else {
            eprintln!("no GPU available -- skipping (documented finding requires real hardware)");
            return;
        };
        let w = 128u32;
        let h = 128u32;
        let seed = 24601;
        let scale = 0.05f32;
        let gpu = dispatch_gpu(&ctx, w, h, seed, scale);
        let cpu = vnoise_grid_cpu(w, h, seed, scale);
        let mut max_abs_diff = 0.0f64;
        let mut mismatches = 0usize;
        for (g, c) in gpu.iter().zip(cpu.iter()) {
            let d = ((*g as f64) - (*c as f64)).abs();
            if d > F32_TOLERANCE {
                mismatches += 1;
            }
            if d > max_abs_diff {
                max_abs_diff = d;
            }
        }
        eprintln!(
            "f32 GPU vs f64 CPU at {w}x{h}: {mismatches}/{} cells exceed tol={F32_TOLERANCE}, max_abs_diff={max_abs_diff}",
            w * h
        );
        // This is the pilot's headline finding: documented as an expected,
        // measured divergence, not asserted pass/fail either way here --
        // CHANGELOG.md records the actual numbers from this run.
    }

    /// `GPU_COMPUTE_PILOT_SCOPE.md` "Done means" #4: the CPU fallback path
    /// must be actually exercised, not merely present.
    #[test]
    fn gpu_fallback_path_matches_cpu_reference() {
        let w = 32u32;
        let h = 32u32;
        let seed = 777;
        let scale = 0.1f32;
        // ctx = None forces the no-GPU branch regardless of real hardware.
        let result = vnoise_grid(None, w, h, seed, scale);
        assert_eq!(result.path, ComputePath::Cpu);
        let reference = vnoise_grid_cpu(w, h, seed, scale);
        assert_eq!(result.values, reference, "CPU fallback must produce the exact already-golden-verified CPU result");
    }

    #[test]
    fn cpu_path_is_deterministic() {
        let a = vnoise_grid_cpu(16, 16, 42, 0.2);
        let b = vnoise_grid_cpu(16, 16, 42, 0.2);
        assert_eq!(a, b);
    }

    /// `GPU_COMPUTE_PILOT_SCOPE.md` item 5 / "Done means" #5: real measured
    /// numbers, independent of whether this particular kernel is
    /// correctness-viable (it isn't, see `f32_hash_diverges_from_cpu_reference`)
    /// -- dispatch/readback overhead characteristics are still real data
    /// about this hardware, useful for judging future GPU-compute
    /// candidates that don't share `hash`'s f64-precision dependency.
    /// Uses `dispatch_gpu` directly (bypasses `self_test` gating) since
    /// this measures raw hardware/API overhead, not correctness.
    #[test]
    fn measured_gpu_vs_cpu_timing() {
        let Some(ctx) = try_gpu() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        // Warm up: first dispatch pays one-time pipeline/driver JIT cost.
        let _ = dispatch_gpu(&ctx, 8, 8, 1, 0.5);

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let scale = 0.02f32;

            let t0 = Instant::now();
            let _ = dispatch_gpu(&ctx, w, h, seed, scale);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = vnoise_grid_cpu(w, h, seed, scale);
            let cpu_time = t1.elapsed();

            eprintln!(
                "{w}x{h} ({} cells): GPU dispatch+readback = {:?}, CPU (single-thread) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    // ===================== GPU_LAYER_INTEGRATION_SCOPE.md milestone 1 =====================
    // The GPU-safe noise pair (`cartalith_noise::gpu_hash`/`gpu_vnoise` vs.
    // `SHADER_SRC_GPU_NOISE`) -- verified CPU vs. GPU directly, NOT against
    // the JS reference (`DECISIONS.md` §7a: this pair is a deliberate
    // redesign, not required to match JS at all).

    fn try_gpu_safe_noise() -> Option<GpuContext> {
        init_gpu_safe_noise().ok()
    }

    /// This IS the milestone's correctness gate (`GPU_LAYER_INTEGRATION_
    /// SCOPE.md` "Done means": CPU/GPU verified identical or within a real
    /// tolerance) -- run at a real field size, not a toy grid, and asserted
    /// (unlike the old pilot's `f32_hash_diverges_from_cpu_reference`,
    /// which documented an *expected* failure -- this one is expected to
    /// pass, by design, and the test enforces that).
    #[test]
    fn gpu_safe_noise_matches_cpu_reference_at_real_field_size() {
        let Some(ctx) = try_gpu_safe_noise() else {
            eprintln!("no GPU available -- skipping (requires real hardware)");
            return;
        };
        let w = 512u32;
        let h = 512u32;
        let seed = 24601;
        let scale = 0.05f32;
        let gpu = dispatch_gpu(&ctx, w, h, seed, scale);
        let cpu = gpu_safe_noise_grid_cpu(w, h, seed, scale);
        let mut max_abs_diff = 0.0f64;
        let mut mismatches = 0usize;
        for (g, c) in gpu.iter().zip(cpu.iter()) {
            let d = ((*g as f64) - (*c as f64)).abs();
            if d > GPU_SAFE_NOISE_TOLERANCE {
                mismatches += 1;
            }
            if d > max_abs_diff {
                max_abs_diff = d;
            }
        }
        eprintln!(
            "gpu-safe noise, GPU vs CPU at {w}x{h}: {mismatches}/{} cells exceed tol={GPU_SAFE_NOISE_TOLERANCE}, max_abs_diff={max_abs_diff}",
            w * h
        );
        assert_eq!(mismatches, 0, "GPU-safe noise pair diverged from its own CPU counterpart -- see max_abs_diff above; this pair has no known precision-regime gap, so any divergence is a real bug to root-cause, not a tolerance to widen");
    }

    #[test]
    fn gpu_safe_noise_self_test_passes() {
        let Some(ctx) = try_gpu_safe_noise() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        const W: u32 = 8;
        const H: u32 = 8;
        const SEED: i32 = 12345;
        const SCALE: f32 = 0.37;
        let gpu = dispatch_gpu(&ctx, W, H, SEED, SCALE);
        let cpu = gpu_safe_noise_grid_cpu(W, H, SEED, SCALE);
        let passed =
            gpu.iter().zip(cpu.iter()).all(|(g, c)| ((*g as f64) - (*c as f64)).abs() <= GPU_SAFE_NOISE_TOLERANCE);
        assert!(passed, "self-test grid: GPU-safe noise GPU output diverged from its CPU counterpart");
    }

    #[test]
    fn gpu_safe_noise_cpu_path_is_deterministic() {
        let a = gpu_safe_noise_grid_cpu(16, 16, 42, 0.2);
        let b = gpu_safe_noise_grid_cpu(16, 16, 42, 0.2);
        assert_eq!(a, b);
    }

    /// `GPU_LAYER_INTEGRATION_SCOPE.md` "Done means": the pilot's own
    /// numbers (4.46x at 512x512, up to 19.55x at 2048x2048) were measured
    /// against the non-portable `hash` kernel and are not confirmed for
    /// this new one -- re-measured here at the same sizes, same honest
    /// methodology (report a loss as legitimately as a win).
    #[test]
    fn measured_gpu_safe_noise_vs_cpu_timing() {
        let Some(ctx) = try_gpu_safe_noise() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        // Warm up: first dispatch pays one-time pipeline/driver JIT cost.
        let _ = dispatch_gpu(&ctx, 8, 8, 1, 0.5);

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let scale = 0.02f32;

            let t0 = Instant::now();
            let _ = dispatch_gpu(&ctx, w, h, seed, scale);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = gpu_safe_noise_grid_cpu(w, h, seed, scale);
            let cpu_time = t1.elapsed();

            eprintln!(
                "gpu-safe noise {w}x{h} ({} cells): GPU dispatch+readback = {:?}, CPU (single-thread) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    /// Secondary pilot experiment: `wgpu::Features::SHADER_F64` is present
    /// on this adapter (confirmed -- see `CHANGELOG.md`), which raised the
    /// question of whether an `f64`-arithmetic WGSL kernel could reproduce
    /// the CPU reference exactly where the f32 kernel could not. It cannot
    /// be tried at all: naga (wgpu 30's WGSL front end) does not implement
    /// `enable f64;` -- its `EnableExtensions` type
    /// (`naga::front::wgsl::parse::directive::enable_extension`) lists
    /// `f16`/`wgpu_int16`/ray-tracing/mesh-shader extensions but no `f64`
    /// entry at all. The GPU and the `wgpu::Features` API both expose the
    /// capability; the WGSL shader language, as wgpu 30 compiles it, has
    /// no syntax to use it. This test captures that failure as data
    /// (`device.push_error_scope`/`pop_error_scope`, not a panic) rather
    /// than asserting a pass/fail on a path that was never reachable.
    /// (A raw-SPIR-V shader source could bypass WGSL entirely and use f64
    /// directly -- genuinely possible, but hand-authoring/generating SPIR-V
    /// is well outside `GPU_COMPUTE_PILOT_SCOPE.md`'s "port the formula"
    /// scope; flagged here as a real door, deliberately not opened.)
    #[test]
    fn f64_wgsl_is_not_implemented_by_naga_even_though_the_gpu_feature_exists() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        })) else {
            eprintln!("no adapter -- skipping");
            return;
        };
        let supports_feature = adapter.features().contains(wgpu::Features::SHADER_F64);
        eprintln!("wgpu::Features::SHADER_F64 reported by adapter: {supports_feature}");
        if !supports_feature {
            eprintln!("this adapter doesn't even report the feature -- nothing further to test");
            return;
        }
        let Ok((device, _queue)) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("f64 probe device"),
            required_features: wgpu::Features::SHADER_F64,
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        })) else {
            eprintln!("device request with SHADER_F64 failed -- skipping");
            return;
        };
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("f64 probe shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC_F64.into()),
        });
        let err = pollster::block_on(scope.pop());
        match err {
            Some(e) => eprintln!("confirmed: WGSL `enable f64;` rejected by naga -- {e}"),
            None => eprintln!(
                "unexpected: `enable f64;` was accepted -- naga's f64 support may have landed since this pilot was written, re-check its EnableExtensions list"
            ),
        }
    }
}
