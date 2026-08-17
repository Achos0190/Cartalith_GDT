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
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: domain warp and crustal
/// heterogeneity, built on `gpu_hash`/`gpu_vnoise` above (via a WGSL
/// `gpu_fbm` combinator duplicated into each shader file -- no cross-file
/// WGSL module include here, see each shader's own header comment).
const SHADER_SRC_GPU_WARP: &str = include_str!("../shaders/gpu_warp.wgsl");
const SHADER_SRC_GPU_HETEROGENEITY: &str = include_str!("../shaders/gpu_heterogeneity.wgsl");
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3: the height formula
/// itself, treating `stress`/`flex`/`hetero`/`age`/`base_field`/`oro` as
/// opaque input buffers (see that document's own "deliberately scoped
/// narrow" note -- plate assignment/stress/flexure/orogeny's own GPU
/// portability is explicitly out of scope here).
const SHADER_SRC_GPU_HEIGHT: &str = include_str!("../shaders/gpu_height.wgsl");
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: separable box blur
/// (`gauss_blur`'s GPU-native direct-sum twin, two entry points --
/// `box_h_main`/`box_v_main`, see that shader's own header for why a
/// direct sum rather than a running-sum port) and `compute_resistance`
/// (trivial, no noise). Neither touches noise, so -- unlike milestones
/// 1-3 -- these may reach genuine three-way JS/CPU/GPU verification, not
/// just an internally-consistent GPU-vs-CPU-twin pair; see the tests
/// below for which this pair actually achieved.
const SHADER_SRC_GPU_GAUSS_BLUR: &str = include_str!("../shaders/gpu_gauss_blur.wgsl");
const SHADER_SRC_GPU_RESISTANCE: &str = include_str!("../shaders/gpu_resistance.wgsl");
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 5: plate assignment via a
/// standard double-buffered Jump Flooding Algorithm -- NOT a port of
/// `cartalith_terrain::assign_plates`'s in-place-mutation variant (see the
/// shader's own header comment for why those are different algorithms, not
/// different implementations of the same one). Verified against
/// brute-force exact-nearest ground truth, not the CPU function directly.
const SHADER_SRC_GPU_JFA_PLATES: &str = include_str!("../shaders/gpu_jfa_plates.wgsl");

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

/// Tolerance for `gpu_compute_warp`'s GPU-vs-CPU pair specifically
/// (`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2) -- looser than
/// [`GPU_SAFE_NOISE_TOLERANCE`] for a real, measured, structural reason:
/// `compute_warp` chains TWO nested `gpu_fbm` evaluations per axis (`qx`/
/// `qy` first, then `wx`/`wy` sampled at `xf + 4*qx, yf + 4*qy`), unlike
/// `gpu_heterogeneity`'s single evaluation (which matches within
/// [`GPU_SAFE_NOISE_TOLERANCE`] exactly, confirming the base `gpu_fbm`
/// combinator itself is not the source). Sub-epsilon residual float-
/// scheduling differences (FMA contraction, etc. -- the same category
/// [`GPU_SAFE_NOISE_TOLERANCE`]'s own doc comment already names) in the
/// first evaluation become a coordinate perturbation feeding the second,
/// full 6-octave evaluation, which can amplify it. Measured directly, not
/// assumed: at 512x512, max observed absolute difference was
/// `1.18e-4` — this tolerance is set above that measured value with a
/// small margin, not widened further to make a differently-sized failure
/// disappear (`PARITY_TESTING.md`'s rule, applied here to a GPU/GPU pair
/// instead of a JS/Rust one).
pub const WARP_TOLERANCE: f64 = 2e-4;

/// Tolerance for `gpu_compute_height`'s GPU-vs-CPU pair
/// (`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3). `compute_height` has
/// only ONE `gpu_fbm`/`gpu_ridged` evaluation per cell (unlike `compute_
/// warp`'s two nested ones) -- the same single-evaluation shape as
/// [`gpu_heterogeneity`], not the compounding shape that earned
/// [`WARP_TOLERANCE`] its looser value. Measured directly at 512x512
/// (both `ridged=false` and `ridged=true`): max observed absolute
/// difference was `1.19e-7` -- essentially `f32`'s own machine epsilon,
/// i.e. bit-exact modulo final rounding. Set at [`GPU_SAFE_NOISE_TOLERANCE`]
/// (the tightest tolerance this crate uses, already proven sufficient for
/// the base noise primitive) rather than reusing [`WARP_TOLERANCE`] just
/// because it was the most recently added constant -- borrowing a looser
/// tolerance than what's actually measured would hide, not honestly
/// report, this kernel's real precision.
pub const HEIGHT_TOLERANCE: f64 = GPU_SAFE_NOISE_TOLERANCE;

/// Tolerance for `gpu_gauss_blur`'s GPU-vs-CPU comparison
/// (`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4) -- and, remarkably,
/// against the REAL, untouched, JS-matching `cartalith_terrain::gauss_blur`
/// directly, not a GPU-specific twin (`gpu_gauss_blur_matches_real_cpu_
/// gauss_blur`, a genuine `cartalith-terrain` dev-dependency comparison).
/// The theoretical concern going in was real -- the CPU side accumulates
/// its sliding-window sum in `f64` (only rounding to `f32` on write),
/// while WGSL has no `f64` at all, so this kernel's direct-sum is `f32`-
/// throughout -- but measured directly at 512x512 across three radius/
/// wrap configurations (including a 48-cell-radius window, `2*48+1=97`
/// summed values), the worst observed divergence was `7.15e-7`,
/// essentially `f32` machine epsilon: a bounded linear sum over a modest
/// window turns out not to compound the way milestones 1-3's chaotic,
/// coordinate-perturbing noise evaluations did. Set just above that
/// measured value, not the far looser bound this doc comment originally
/// guessed before running the actual test.
pub const BLUR_TOLERANCE: f64 = 2e-6;

/// Tolerance for `gpu_compute_resistance`'s GPU-vs-CPU comparison --
/// also against the REAL `cartalith_terrain::compute_resistance` directly
/// (`gpu_compute_resistance_matches_real_cpu_compute_resistance`).
/// `compute_resistance` has no noise, no transcendental functions, and no
/// accumulation of any kind -- a single multiply-add-min per cell.
/// Measured at 512x512: worst observed divergence `5.96e-8`, essentially
/// `f32` machine epsilon -- genuine three-way JS/CPU/GPU parity. Set with
/// a comfortable margin (~8x) over that measured value, matching this
/// crate's own convention for a stable, FMA-contraction-scale residual
/// ([`GPU_SAFE_NOISE_TOLERANCE`]'s own doc comment names the same source)
/// rather than the thin ~1.7x margin a first pass gave this constant.
pub const RESISTANCE_TOLERANCE: f64 = 5e-7;

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

/// Matches `gpu_warp.wgsl`'s `WarpParams` field-for-field, including the
/// explicit padding -- WGSL uniform-address-space structs are commonly
/// required to round to 16-byte multiples by real backends even where the
/// spec's minimum requirement is looser; padding explicitly here avoids
/// relying on that being true only in the strict-minimum case.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct WarpParams {
    seed: i32,
    width: u32,
    height: u32,
    wf: f32,
    amp: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

/// Matches `gpu_heterogeneity.wgsl`'s `HeteroParams` field-for-field.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct HeteroParams {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32,
}

/// Matches `gpu_height.wgsl`'s `HeightParams` field-for-field. 12 `f32`-
/// sized fields = 48 bytes = 3x16, already a multiple of the common
/// uniform-buffer alignment real backends round up to (see `WarpParams`'
/// own comment on why this matters) -- no explicit padding field needed
/// beyond `_pad0`, which exists only to keep the field count even/clean.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct HeightGpuParams {
    seed: i32,
    width: u32,
    height: u32,
    nf: f32,
    a: f32,
    b: f32,
    age_inf: f32,
    fwt: f32,
    hwt: f32,
    ridged: u32,
    has_oro: u32,
    _pad0: f32,
}

/// Matches `gpu_gauss_blur.wgsl`'s `BlurParams` field-for-field -- already
/// a multiple of 16 bytes (4 x u32/i32), no explicit padding needed.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct BlurParams {
    width: u32,
    height: u32,
    radius: i32,
    wrap: u32,
}

/// Matches `gpu_resistance.wgsl`'s `ResistanceParams` field-for-field,
/// including its explicit padding to a 16-byte multiple.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ResistanceParams {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct JfaParams {
    width: u32,
    height: u32,
    step: i32,
    _pad0: u32,
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
    init_gpu_with(SHADER_SRC, wgpu::Features::empty(), "vnoise (f32)", &ONE_STORAGE_OUT_LAYOUT)
}

/// Secondary pilot experiment: identical setup, but requests
/// `Features::SHADER_F64` and uses the `f64`-arithmetic shader
/// (`vnoise_f64.wgsl`) -- tests whether the CPU reference's
/// f64-rounding-dependent `hash` formula is exactly reproducible on GPU
/// when the (optional, Vulkan-only, native-only) feature is available.
/// Returns `Err` cleanly if the adapter doesn't support it -- callers
/// should treat that the same as "no GPU" (`HARDWARE_ACCELERATION.md` §27).
pub fn init_gpu_f64() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_F64, wgpu::Features::SHADER_F64, "vnoise (f64)", &ONE_STORAGE_OUT_LAYOUT)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1: the GPU-safe noise
/// primitive's device/pipeline. Same setup discipline as [`init_gpu`]
/// (conservative limits, no fallback adapter) -- reuses [`Params`]'s exact
/// bind-group layout (uniform Params + one storage `f32` buffer), so
/// [`dispatch_gpu`] works unmodified against this context too.
pub fn init_gpu_safe_noise() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_NOISE, wgpu::Features::empty(), "gpu_noise (f32, PCG3D)", &ONE_STORAGE_OUT_LAYOUT)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: domain warp needs two
/// storage outputs (warp_x, warp_y) computed together in one dispatch
/// (matching `compute_warp`'s own shape -- `qx`/`qy` are shared between
/// the `wx`/`wy` paths, so splitting into two dispatches would recompute
/// them twice for no benefit).
pub fn init_gpu_warp() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_WARP, wgpu::Features::empty(), "gpu_warp (f32, PCG3D fbm)", &TWO_STORAGE_OUT_LAYOUT)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: crustal heterogeneity
/// reads three inputs (age, required; warp_x/warp_y, zero-filled when the
/// caller has no warp field -- see the shader's own header comment) and
/// writes one output (the pre-normalize heterogeneity value; the
/// max-reduce normalize pass runs on CPU after readback).
pub fn init_gpu_heterogeneity() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(
        SHADER_SRC_GPU_HETEROGENEITY,
        wgpu::Features::empty(),
        "gpu_heterogeneity (f32, PCG3D fbm)",
        &HETEROGENEITY_LAYOUT,
    )
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3: the height formula reads
/// 8 input fields (base, stress, flex, hetero, age, warp_x, warp_y, oro)
/// and writes 1 output -- more storage buffers than any prior kernel in
/// this crate needed, past `Limits::downlevel_defaults()`'s conservative
/// baseline (see [`init_gpu_with`]'s own limit-derivation for how this is
/// requested without hand-picking a number).
pub fn init_gpu_height() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_HEIGHT, wgpu::Features::empty(), "gpu_height (f32, PCG3D fbm/ridged)", &HEIGHT_LAYOUT)
}

const ONE_STORAGE_OUT_LAYOUT: [wgpu::BindGroupLayoutEntry; 2] =
    [uniform_entry(0), storage_entry(1, false)];
const TWO_STORAGE_OUT_LAYOUT: [wgpu::BindGroupLayoutEntry; 3] =
    [uniform_entry(0), storage_entry(1, false), storage_entry(2, false)];
const HETEROGENEITY_LAYOUT: [wgpu::BindGroupLayoutEntry; 5] = [
    uniform_entry(0),
    storage_entry(1, true),
    storage_entry(2, true),
    storage_entry(3, true),
    storage_entry(4, false),
];
const HEIGHT_LAYOUT: [wgpu::BindGroupLayoutEntry; 10] = [
    uniform_entry(0),
    storage_entry(1, true), // base
    storage_entry(2, true), // stress
    storage_entry(3, true), // flex
    storage_entry(4, true), // hetero
    storage_entry(5, true), // age
    storage_entry(6, true), // warp_x
    storage_entry(7, true), // warp_y
    storage_entry(8, true), // oro
    storage_entry(9, false), // out
];
const RESISTANCE_LAYOUT: [wgpu::BindGroupLayoutEntry; 5] = [
    uniform_entry(0),
    storage_entry(1, true), // plate_id
    storage_entry(2, true), // age
    storage_entry(3, true), // crustal_per_plate
    storage_entry(4, false), // out
];
/// Shared by both `box_h_main` and `box_v_main` -- same buffer signature
/// (params, one input, one output), only the shader entry point differs.
const BLUR_LAYOUT: [wgpu::BindGroupLayoutEntry; 3] =
    [uniform_entry(0), storage_entry(1, true), storage_entry(2, false)];
/// `gpu_jfa_plates.wgsl`'s `main` -- one pipeline, dispatched once per
/// JFA step with alternating in/out bind groups (double-buffered), same
/// "single pipeline, many bind groups" shape [`init_gpu_resistance`] uses,
/// not [`GpuBlurContext`]'s two-pipeline shape (JFA has only one shader
/// entry point, unlike blur's `box_h`/`box_v` pair).
const JFA_LAYOUT: [wgpu::BindGroupLayoutEntry; 9] = [
    uniform_entry(0),
    storage_entry(1, true),  // nearest_in
    storage_entry(2, true),  // best_d2_in
    storage_entry(3, false), // nearest_out
    storage_entry(4, false), // best_d2_out
    storage_entry(5, true),  // plate_x
    storage_entry(6, true),  // plate_y
    storage_entry(7, true),  // warp_x
    storage_entry(8, true),  // warp_y
];

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: `compute_resistance`,
/// trivial per-cell formula, single pipeline -- reuses [`init_gpu_with`]
/// unchanged, same as every kernel before this one that only needed one
/// shader entry point.
pub fn init_gpu_resistance() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_RESISTANCE, wgpu::Features::empty(), "gpu_resistance (f32)", &RESISTANCE_LAYOUT)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 5: plate assignment (JFA).
/// One pipeline, `log2(max(width,height))`-ish dispatches per call
/// (see [`dispatch_gpu_assign_plates`]), all issued within one encoder --
/// same "many passes, one submit" shape [`dispatch_gpu_gauss_blur`]
/// already established.
pub fn init_gpu_jfa_plates() -> Result<GpuContext, GpuInitError> {
    init_gpu_with(SHADER_SRC_GPU_JFA_PLATES, wgpu::Features::empty(), "gpu_jfa_plates (f32, double-buffered JFA)", &JFA_LAYOUT)
}

/// Two pipelines (`box_h_main`, `box_v_main`) sharing one device/queue/
/// bind-group-layout -- `gauss_blur`'s three-pass horizontal-then-
/// vertical structure needs both kernels able to write into buffers the
/// other reads, which a single-pipeline [`GpuContext`] (built for exactly
/// one shader entry point) can't express. A dedicated context type rather
/// than overloading [`GpuContext`] itself, so every existing single-
/// pipeline caller stays untouched.
pub struct GpuBlurContext {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
    box_h_pipeline: wgpu::ComputePipeline,
    box_v_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: `gauss_blur`'s GPU path.
/// Duplicates [`init_gpu_with`]'s adapter/device/queue setup rather than
/// refactoring it to support multiple entry points -- this is the only
/// kernel in this crate needing two pipelines from one shader module, and
/// a one-off dedicated function is a smaller, clearer diff than
/// generalizing the shared helper for a single caller.
pub fn init_gpu_gauss_blur() -> Result<GpuBlurContext, GpuInitError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .map_err(|_| GpuInitError::NoAdapter)?;

    let info = adapter.get_info();
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits = limits.using_resolution(adapter.limits());
    let storage_buffers_needed = BLUR_LAYOUT
        .iter()
        .filter(|e| matches!(e.ty, wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { .. }, .. }))
        .count() as u32;
    limits.max_storage_buffers_per_shader_stage =
        limits.max_storage_buffers_per_shader_stage.max(storage_buffers_needed);

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cartalith-gpu blur device"),
        required_features: wgpu::Features::empty(),
        required_limits: limits,
        ..Default::default()
    }))
    .map_err(GpuInitError::RequestDevice)?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpu_gauss_blur (f32, direct-sum box blur)"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC_GPU_GAUSS_BLUR.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cartalith-gpu blur bind group layout"),
        entries: &BLUR_LAYOUT,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gauss blur pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let box_h_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("box_h pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("box_h_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let box_v_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("box_v pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("box_v_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    Ok(GpuBlurContext {
        adapter_name: info.name,
        adapter_vendor: info.vendor,
        adapter_backend: info.backend,
        device_type: info.device_type,
        device,
        queue,
        box_h_pipeline,
        box_v_pipeline,
        bind_group_layout,
    })
}

const fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

const fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// The adapter+device+queue triple, before any pipeline is built on top of
/// it -- factored out of what was `init_gpu_with`'s own inline body so
/// [`init_gpu_shared_device`] (milestone 8, GPU context reuse) can request
/// this exact same handshake once and hand it to several pipeline builders,
/// instead of every kernel repeating its own `request_adapter`/
/// `request_device` round trip.
struct RawGpuDevice {
    adapter_name: String,
    adapter_vendor: u32,
    adapter_backend: wgpu::Backend,
    device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// `HARDWARE_ACCELERATION.md` §10: request the minimum actually needed,
/// not `Limits::unlimited()` -- but "minimum needed" depends on how many
/// storage buffers the kernel(s) this device will back actually declare.
/// Still capped by the adapter's own real limit (`using_resolution` below),
/// never requesting more than the hardware actually reports.
fn request_gpu_device(
    required_features: wgpu::Features,
    min_storage_buffers: u32,
    device_label: &str,
) -> Result<RawGpuDevice, GpuInitError> {
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
    limits.max_storage_buffers_per_shader_stage =
        limits.max_storage_buffers_per_shader_stage.max(min_storage_buffers);

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some(device_label),
        required_features,
        required_limits: limits,
        ..Default::default()
    }))
    .map_err(GpuInitError::RequestDevice)?;

    Ok(RawGpuDevice {
        adapter_name: info.name,
        adapter_vendor: info.vendor,
        adapter_backend: info.backend,
        device_type: info.device_type,
        device,
        queue,
    })
}

fn count_storage_buffers(layout_entries: &[wgpu::BindGroupLayoutEntry]) -> u32 {
    layout_entries
        .iter()
        .filter(|e| matches!(e.ty, wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { .. }, .. }))
        .count() as u32
}

fn build_pipeline(
    raw: RawGpuDevice,
    shader_src: &str,
    label: &str,
    layout_entries: &[wgpu::BindGroupLayoutEntry],
) -> GpuContext {
    let shader = raw.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let bind_group_layout = raw.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cartalith-gpu bind group layout"),
        entries: layout_entries,
    });

    let pipeline_layout = raw.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("vnoise pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let pipeline = raw.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("vnoise pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    GpuContext {
        adapter_name: raw.adapter_name,
        adapter_vendor: raw.adapter_vendor,
        adapter_backend: raw.adapter_backend,
        device_type: raw.device_type,
        device: raw.device,
        queue: raw.queue,
        pipeline,
        bind_group_layout,
    }
}

fn init_gpu_with(
    shader_src: &str,
    required_features: wgpu::Features,
    label: &str,
    layout_entries: &[wgpu::BindGroupLayoutEntry],
) -> Result<GpuContext, GpuInitError> {
    let storage_buffers_needed = count_storage_buffers(layout_entries);
    let raw = request_gpu_device(required_features, storage_buffers_needed, "cartalith-gpu pilot device")?;
    Ok(build_pipeline(raw, shader_src, label, layout_entries))
}

/// A shared adapter+device+queue, reusable across several pipeline
/// contexts. `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6 measured
/// `instance.request_adapter`/`adapter.request_device` (both synchronous
/// driver calls) at ~1.3-1.4s **per stage**, flat regardless of grid size
/// or which shader follows -- the dominant cost at every size
/// `generate_terrain` ships at by default below 2048x2048, since each of
/// its five GPU dispatches (warp, heterogeneity, plate assignment, and two
/// separate `gauss_blur` calls) was paying that handshake independently.
/// `wgpu::Device`/`wgpu::Queue` are cheap `Clone` handles (Arc-backed
/// internally, confirmed by reading `wgpu` 30's own source, not assumed),
/// so building each stage's own pipeline from one shared `GpuDevice`
/// keeps the expensive part paid once per `generate_terrain` call instead
/// of once per stage, without needing `unsafe` or a custom ref-counting
/// scheme.
pub struct GpuDevice {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// Sized for the largest bind group among the kernels `generate_terrain`'s
/// `use_gpu` path reuses this device across -- JFA plate assignment's own
/// [`JFA_LAYOUT`] (8 storage buffers), the highest of the four reused
/// kernels (warp needs 2, heterogeneity 4, blur 2). `wgpu` limits can't be
/// raised after device creation, so this has to be decided up front rather
/// than derived per-pipeline the way [`init_gpu_with`] derives it for a
/// single-use device.
const REUSED_STAGE_MAX_STORAGE_BUFFERS: u32 = 8;

/// Create the shared device milestone 8's `_with` pipeline builders below
/// (e.g. [`init_gpu_warp_with`]) each build a pipeline on top of, instead
/// of each independently requesting its own adapter/device.
pub fn init_gpu_shared_device() -> Result<GpuDevice, GpuInitError> {
    let raw = request_gpu_device(
        wgpu::Features::empty(),
        REUSED_STAGE_MAX_STORAGE_BUFFERS,
        "cartalith-gpu shared device",
    )?;
    Ok(GpuDevice {
        adapter_name: raw.adapter_name,
        adapter_vendor: raw.adapter_vendor,
        adapter_backend: raw.adapter_backend,
        device_type: raw.device_type,
        device: raw.device,
        queue: raw.queue,
    })
}

fn build_pipeline_shared(
    gpu: &GpuDevice,
    shader_src: &str,
    label: &str,
    layout_entries: &[wgpu::BindGroupLayoutEntry],
) -> GpuContext {
    let raw = RawGpuDevice {
        adapter_name: gpu.adapter_name.clone(),
        adapter_vendor: gpu.adapter_vendor,
        adapter_backend: gpu.adapter_backend,
        device_type: gpu.device_type,
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
    };
    build_pipeline(raw, shader_src, label, layout_entries)
}

/// `_with` sibling of [`init_gpu_warp`] -- builds the same pipeline on an
/// already-created [`GpuDevice`] instead of requesting a new adapter/device.
pub fn init_gpu_warp_with(gpu: &GpuDevice) -> GpuContext {
    build_pipeline_shared(gpu, SHADER_SRC_GPU_WARP, "gpu_warp (f32, PCG3D fbm)", &TWO_STORAGE_OUT_LAYOUT)
}

/// `_with` sibling of [`init_gpu_heterogeneity`].
pub fn init_gpu_heterogeneity_with(gpu: &GpuDevice) -> GpuContext {
    build_pipeline_shared(
        gpu,
        SHADER_SRC_GPU_HETEROGENEITY,
        "gpu_heterogeneity (f32, PCG3D fbm)",
        &HETEROGENEITY_LAYOUT,
    )
}

/// `_with` sibling of [`init_gpu_jfa_plates`].
pub fn init_gpu_jfa_plates_with(gpu: &GpuDevice) -> GpuContext {
    build_pipeline_shared(
        gpu,
        SHADER_SRC_GPU_JFA_PLATES,
        "gpu_jfa_plates (f32, double-buffered JFA)",
        &JFA_LAYOUT,
    )
}

/// `_with` sibling of [`init_gpu_gauss_blur`] -- same two-pipeline
/// (`box_h_main`/`box_v_main`) shape, built on a shared device instead of
/// requesting its own.
pub fn init_gpu_gauss_blur_with(gpu: &GpuDevice) -> GpuBlurContext {
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpu_gauss_blur (f32, direct-sum box blur)"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC_GPU_GAUSS_BLUR.into()),
    });

    let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cartalith-gpu blur bind group layout"),
        entries: &BLUR_LAYOUT,
    });

    let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gauss blur pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let box_h_pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("box_h pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("box_h_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let box_v_pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("box_v pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("box_v_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    GpuBlurContext {
        adapter_name: gpu.adapter_name.clone(),
        adapter_vendor: gpu.adapter_vendor,
        adapter_backend: gpu.adapter_backend,
        device_type: gpu.device_type,
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
        box_h_pipeline,
        box_v_pipeline,
        bind_group_layout,
    }
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

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: dispatch `gpu_warp.wgsl`,
/// returning `(warp_x, warp_y)` -- one dispatch computes both, matching
/// `compute_warp`'s own shape (see [`init_gpu_warp`]'s doc comment).
fn dispatch_gpu_warp(ctx: &GpuContext, width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> (Vec<f32>, Vec<f32>) {
    let count = (width * height) as usize;
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = WarpParams { seed, width, height, wf, amp, _pad0: 0.0, _pad1: 0.0, _pad2: 0.0 };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("warp params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let make_storage = |label: &str| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    };
    let out_x = make_storage("warp_x (storage)");
    let out_y = make_storage("warp_y (storage)");
    let make_staging = |label: &str| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    };
    let staging_x = make_staging("warp_x (staging)");
    let staging_y = make_staging("warp_y (staging)");

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("warp bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: out_x.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: out_y.as_entire_binding() },
        ],
    });

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("warp encoder") });
    {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("warp pass"), timestamp_writes: None });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out_x, 0, &staging_x, 0, byte_len);
    encoder.copy_buffer_to_buffer(&out_y, 0, &staging_y, 0, byte_len);
    ctx.queue.submit(Some(encoder.finish()));

    let read_back = |staging: &wgpu::Buffer| -> Vec<f32> {
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        ctx.device.poll(wgpu::PollType::wait_indefinitely()).expect("device poll failed");
        rx.recv().expect("map_async channel closed").expect("buffer map failed");
        let data = slice.get_mapped_range().expect("get_mapped_range failed");
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        result
    };
    (read_back(&staging_x), read_back(&staging_y))
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: dispatch
/// `gpu_heterogeneity.wgsl`. `age` is required; `warp_x`/`warp_y` should be
/// zero-filled by the caller when there's no real warp field (mirrors
/// `compute_heterogeneity`'s `Option<&[f32]>` -- see the shader's own
/// header comment). Returns the PRE-normalize heterogeneity field --
/// callers do the max-reduce normalize pass on CPU (see
/// [`normalize_by_max_abs`]).
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_heterogeneity(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    hetero_seed: i32,
    scale: f32,
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
) -> Vec<f32> {
    let count = (width * height) as usize;
    assert_eq!(age.len(), count);
    assert_eq!(warp_x.len(), count);
    assert_eq!(warp_y.len(), count);
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = HeteroParams { seed: hetero_seed, width, height, scale };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hetero params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let make_input = |label: &str, data: &[f32]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE,
        })
    };
    let age_buf = make_input("hetero age (storage)", age);
    let warp_x_buf = make_input("hetero warp_x (storage)", warp_x);
    let warp_y_buf = make_input("hetero warp_y (storage)", warp_y);
    let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hetero out (storage)"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hetero out (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("hetero bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: age_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: warp_x_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: warp_y_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: out_buf.as_entire_binding() },
        ],
    });

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("hetero encoder") });
    {
        let mut pass = encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("hetero pass"), timestamp_writes: None });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out_buf, 0, &staging_buf, 0, byte_len);
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

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3: dispatch `gpu_height.wgsl`.
/// `warp_x`/`warp_y` should be zero-filled by the caller when there's no
/// real warp field (matches `compute_height`'s `Option<&[f32]>` the same
/// way [`dispatch_gpu_heterogeneity`] already does). `oro` is different --
/// its ABSENCE changes which formula runs, not just an additive no-op --
/// so `has_oro` is a real parameter, and the `oro` buffer content is
/// ignored by the shader when `has_oro` is `false` (any same-length slice
/// works, including a zero-filled dummy).
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_height(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    seed: i32,
    nf: f32,
    a: f32,
    b: f32,
    age_inf: f32,
    fwt: f32,
    hwt: f32,
    ridged: bool,
    has_oro: bool,
    base_field: &[f32],
    stress: &[f32],
    flex: &[f32],
    hetero: &[f32],
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
    oro: &[f32],
) -> Vec<f32> {
    let count = (width * height) as usize;
    assert_eq!(base_field.len(), count);
    assert_eq!(stress.len(), count);
    assert_eq!(flex.len(), count);
    assert_eq!(hetero.len(), count);
    assert_eq!(age.len(), count);
    assert_eq!(warp_x.len(), count);
    assert_eq!(warp_y.len(), count);
    assert_eq!(oro.len(), count);
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = HeightGpuParams {
        seed,
        width,
        height,
        nf,
        a,
        b,
        age_inf,
        fwt,
        hwt,
        ridged: ridged as u32,
        has_oro: has_oro as u32,
        _pad0: 0.0,
    };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("height params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let make_input = |label: &str, data: &[f32]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE,
        })
    };
    let base_buf = make_input("height base (storage)", base_field);
    let stress_buf = make_input("height stress (storage)", stress);
    let flex_buf = make_input("height flex (storage)", flex);
    let hetero_buf = make_input("height hetero (storage)", hetero);
    let age_buf = make_input("height age (storage)", age);
    let warp_x_buf = make_input("height warp_x (storage)", warp_x);
    let warp_y_buf = make_input("height warp_y (storage)", warp_y);
    let oro_buf = make_input("height oro (storage)", oro);
    let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("height out (storage)"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("height out (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("height bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: base_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: stress_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: flex_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: hetero_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: age_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 6, resource: warp_x_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 7, resource: warp_y_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 8, resource: oro_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 9, resource: out_buf.as_entire_binding() },
        ],
    });

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("height encoder") });
    {
        let mut pass = encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("height pass"), timestamp_writes: None });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out_buf, 0, &staging_buf, 0, byte_len);
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

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: dispatch `gpu_resistance.wgsl`.
/// `crustal_per_plate` is precomputed by the caller as `plates[k].base.
/// max(0.0)` for each plate -- a tiny (`num_plates`-length) CPU-side step,
/// not the per-cell workload this kernel accelerates (see the shader's
/// own header comment).
fn dispatch_gpu_resistance(ctx: &GpuContext, width: u32, height: u32, plate_id: &[u32], age: &[f32], crustal_per_plate: &[f32]) -> Vec<f32> {
    let count = (width * height) as usize;
    assert_eq!(plate_id.len(), count);
    assert_eq!(age.len(), count);
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = ResistanceParams { width, height, _pad0: 0, _pad1: 0 };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("resistance params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let plate_id_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("resistance plate_id (storage)"),
        contents: bytemuck::cast_slice(plate_id),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let age_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("resistance age (storage)"),
        contents: bytemuck::cast_slice(age),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let crustal_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("resistance crustal_per_plate (storage)"),
        contents: bytemuck::cast_slice(crustal_per_plate),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resistance out (storage)"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resistance out (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("resistance bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: plate_id_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: age_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: crustal_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: out_buf.as_entire_binding() },
        ],
    });

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("resistance encoder") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("resistance pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out_buf, 0, &staging_buf, 0, byte_len);
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

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: dispatch `gpu_gauss_blur.
/// wgsl`'s two entry points, three passes each (box_h then box_v), all
/// within ONE command encoder -- wgpu sequences compute passes within a
/// single encoder in submission order with an implicit barrier between
/// them, so each pass's writes are visible to the next pass's reads
/// without a CPU round-trip between iterations. Mirrors `gauss_blur`'s own
/// `r<1.0` early-return-copy and `pr=round(r/1.6).max(1.0)` radius
/// derivation exactly (JS `Math.round`/this project's own `js_round` and
/// Rust `f64::round()` agree for the always-positive `r` this function
/// receives -- both round halfway-from-zero, i.e. up, for positive
/// values).
fn dispatch_gpu_gauss_blur(ctx: &GpuBlurContext, src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Vec<f32> {
    if radius < 1.0 {
        return src.to_vec();
    }
    let count = (width * height) as usize;
    assert_eq!(src.len(), count);
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;
    let pr = (radius / 1.6).round().max(1.0) as i32;

    let params_h = BlurParams { width, height, radius: pr, wrap: wrap_x as u32 };
    let params_v = BlurParams { width, height, radius: pr, wrap: 0 }; // box_v never wraps, matching the CPU box_v
    let params_h_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("blur params (h)"),
        contents: bytemuck::bytes_of(&params_h),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let params_v_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("blur params (v)"),
        contents: bytemuck::bytes_of(&params_v),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // Ping-pong between two storage buffers across the 6 dispatches (3x
    // box_h, 3x box_v), same alternation `gauss_blur` itself does with its
    // own `a`/`b` CPU arrays.
    let make_rw = |label: &str, contents: Option<&[f32]>| {
        if let Some(c) = contents {
            ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(c),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            })
        } else {
            ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: byte_len,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    };
    let buf_a = make_rw("blur buf a", Some(src));
    let buf_b = make_rw("blur buf b", None);
    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("blur out (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let make_bind_group = |label: &str, params_buf: &wgpu::Buffer, in_buf: &wgpu::Buffer, out_buf: &wgpu::Buffer| {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &ctx.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: in_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: out_buf.as_entire_binding() },
            ],
        })
    };
    // a -> b (box_h), b -> a (box_v), repeated 3 times: a holds the result.
    let bg_h = make_bind_group("blur bg h (a->b)", &params_h_buf, &buf_a, &buf_b);
    let bg_v = make_bind_group("blur bg v (b->a)", &params_v_buf, &buf_b, &buf_a);

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("blur encoder") });
    for _ in 0..3 {
        {
            let mut pass = encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("box_h pass"), timestamp_writes: None });
            pass.set_pipeline(&ctx.box_h_pipeline);
            pass.set_bind_group(0, &bg_h, &[]);
            pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
        }
        {
            let mut pass = encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("box_v pass"), timestamp_writes: None });
            pass.set_pipeline(&ctx.box_v_pipeline);
            pass.set_bind_group(0, &bg_v, &[]);
            pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
        }
    }
    encoder.copy_buffer_to_buffer(&buf_a, 0, &staging_buf, 0, byte_len);
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

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 5: double-buffered JFA plate
/// assignment. `world`-mode x-wrap is deliberately unimplemented (matching
/// every GPU milestone so far) -- callers must pass `world=false` data.
///
/// Seeding (`nearest[home_cell]=p, best_d2=0`) and the CPU-side fallback
/// fill for any cell JFA never reached (`nearest<0`, filled by brute-force
/// nearest-plate search, mirroring `assign_plates`'s own fallback loop)
/// both run on CPU -- seeding is O(plate count), the fallback is expected
/// near-zero-cost in practice (a full JFA sweep down to step=1 reaches
/// essentially every cell), so neither is worth its own GPU kernel.
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_assign_plates(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    plate_x: &[f32],
    plate_y: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Vec<i32> {
    let w = width as usize;
    let h = height as usize;
    let n = w * h;
    let np = plate_x.len();
    assert_eq!(plate_y.len(), np);
    let byte_len_i32 = (n * std::mem::size_of::<i32>()) as u64;
    let byte_len_f32 = (n * std::mem::size_of::<f32>()) as u64;

    // Seed: each plate's own home cell starts at distance 0, matching
    // `assign_plates`'s own seeding loop (clamped y, skip out-of-range x).
    let mut nearest0 = vec![-1i32; n];
    let mut best_d20 = vec![1e30f32; n];
    for (p, (&px, &py)) in plate_x.iter().zip(plate_y.iter()).enumerate() {
        let cx = px as i32;
        if cx < 0 || cx >= width as i32 {
            continue;
        }
        let cy = (py as i32).clamp(0, height as i32 - 1) as usize;
        let i = cy * w + cx as usize;
        nearest0[i] = p as i32;
        best_d20[i] = 0.0;
    }

    let warp_x_owned;
    let warp_x = match warp_x {
        Some(w) => w,
        None => {
            warp_x_owned = vec![0f32; n];
            &warp_x_owned
        }
    };
    let warp_y_owned;
    let warp_y = match warp_y {
        Some(w) => w,
        None => {
            warp_y_owned = vec![0f32; n];
            &warp_y_owned
        }
    };

    let max_dim = w.max(h) as f64;
    let max_step = 1u32 << (max_dim.log2().ceil() as u32);
    let mut steps = Vec::new();
    let mut step_u = max_step >> 1;
    loop {
        steps.push(step_u as i32);
        if step_u == 1 {
            break;
        }
        step_u >>= 1;
    }

    let make_i32 = |label: &str, contents: &[i32]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(contents),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        })
    };
    let make_f32 = |label: &str, contents: &[f32]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(contents),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        })
    };
    let make_i32_empty = |label: &str| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len_i32,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    };
    let make_f32_empty = |label: &str| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len_f32,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    };

    let nearest_a = make_i32("jfa nearest a", &nearest0);
    let nearest_b = make_i32_empty("jfa nearest b");
    let best_d2_a = make_f32("jfa best_d2 a", &best_d20);
    let best_d2_b = make_f32_empty("jfa best_d2 b");
    let plate_x_buf = make_f32("jfa plate_x", plate_x);
    let plate_y_buf = make_f32("jfa plate_y", plate_y);
    let warp_x_buf = make_f32("jfa warp_x", warp_x);
    let warp_y_buf = make_f32("jfa warp_y", warp_y);

    let staging_nearest = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("jfa nearest (staging)"),
        size: byte_len_i32,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let param_bufs: Vec<wgpu::Buffer> = steps
        .iter()
        .map(|&step| {
            let params = JfaParams { width, height, step, _pad0: 0 };
            ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("jfa params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            })
        })
        .collect();

    let make_bind_group = |label: &str,
                            params_buf: &wgpu::Buffer,
                            nearest_in: &wgpu::Buffer,
                            best_d2_in: &wgpu::Buffer,
                            nearest_out: &wgpu::Buffer,
                            best_d2_out: &wgpu::Buffer| {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &ctx.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: nearest_in.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: best_d2_in.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: nearest_out.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: best_d2_out.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: plate_x_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: plate_y_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 7, resource: warp_x_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 8, resource: warp_y_buf.as_entire_binding() },
            ],
        })
    };

    // One fresh bind group per pass (each has a distinct `step` uniform),
    // alternating a->b / b->a direction -- same ping-pong shape
    // `dispatch_gpu_gauss_blur` uses, generalized from a fixed 3x2 to N
    // distinct-param passes (N = steps.len(), unknown until `width`/
    // `height` are known, so passes can't be split into two reused bind
    // groups the way blur's fixed 3-iteration loop can).
    let bind_groups: Vec<wgpu::BindGroup> = param_bufs
        .iter()
        .enumerate()
        .map(|(idx, params_buf)| {
            if idx % 2 == 0 {
                make_bind_group("jfa bg a->b", params_buf, &nearest_a, &best_d2_a, &nearest_b, &best_d2_b)
            } else {
                make_bind_group("jfa bg b->a", params_buf, &nearest_b, &best_d2_b, &nearest_a, &best_d2_a)
            }
        })
        .collect();

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("jfa encoder") });
    for bg in &bind_groups {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("jfa pass"), timestamp_writes: None });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // After an even number of passes (0-indexed: 1,3,5.. => odd count)
    // result lands back in `a`; after an odd pass count it's in `b`.
    let result_in_a = steps.len() % 2 == 0;
    let final_buf = if result_in_a { &nearest_a } else { &nearest_b };
    encoder.copy_buffer_to_buffer(final_buf, 0, &staging_nearest, 0, byte_len_i32);
    ctx.queue.submit(Some(encoder.finish()));

    let slice = staging_nearest.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    ctx.device.poll(wgpu::PollType::wait_indefinitely()).expect("device poll failed");
    rx.recv().expect("map_async channel closed").expect("buffer map failed");
    let data = slice.get_mapped_range().expect("get_mapped_range failed");
    let mut nearest: Vec<i32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging_nearest.unmap();

    // Fallback: any cell JFA never reached (rare/none in practice) gets a
    // brute-force nearest-plate search, matching `assign_plates`'s own
    // fallback loop.
    for i in 0..n {
        if nearest[i] >= 0 {
            continue;
        }
        let x = (i % w) as f32;
        let y = (i / w) as f32;
        let ax = x + warp_x[i];
        let ay = y + warp_y[i];
        let mut best = 0i32;
        let mut bd = f32::INFINITY;
        for p in 0..np {
            let dx = ax - plate_x[p];
            let dy = ay - plate_y[p];
            let d = dx * dx + dy * dy;
            if d < bd {
                bd = d;
                best = p as i32;
            }
        }
        nearest[i] = best;
    }
    nearest
}

/// Brute-force exact nearest-plate ground truth (no JFA at all) -- what
/// both `assign_plates` (CPU, in-place JFA) and [`dispatch_gpu_assign_plates`]
/// (GPU, double-buffered JFA) are each an *approximation* of. Used to
/// characterize both JFA variants' real mismatch rate against the true
/// answer, since the two JFA variants are not expected to match each other
/// exactly (see `gpu_jfa_plates.wgsl`'s header comment). `world`-mode
/// unimplemented, matching the GPU kernel it's verifying.
pub fn brute_force_nearest_plate(
    width: u32,
    height: u32,
    plate_x: &[f32],
    plate_y: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Vec<i32> {
    let w = width as usize;
    let h = height as usize;
    let np = plate_x.len();
    let mut out = vec![0i32; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let ax = x as f32 + warp_x.map_or(0.0, |v| v[i]);
            let ay = y as f32 + warp_y.map_or(0.0, |v| v[i]);
            let mut best = 0i32;
            let mut bd = f32::INFINITY;
            for p in 0..np {
                let dx = ax - plate_x[p];
                let dy = ay - plate_y[p];
                let d = dx * dx + dy * dy;
                if d < bd {
                    bd = d;
                    best = p as i32;
                }
            }
            out[i] = best;
        }
    }
    out
}

/// `compute_heterogeneity`'s trailing max-reduce normalize pass, factored
/// out so both the CPU and GPU paths apply the exact same post-process to
/// their (different, per `DECISIONS.md` §7c) per-cell outputs.
fn normalize_by_max_abs(values: &mut [f32]) {
    let mut mx = 1e-6f64;
    for &v in values.iter() {
        let v = (v as f64).abs();
        if v > mx {
            mx = v;
        }
    }
    for v in values.iter_mut() {
        *v = (*v as f64 / mx) as f32;
    }
}

/// CPU reference for [`dispatch_gpu_warp`] -- calls `cartalith_noise::gpu_fbm`
/// directly (all-`f32`, same shape as [`gpu_safe_noise_grid_cpu`]), not a
/// second reimplementation. Non-world case only, matching the GPU kernel.
pub fn gpu_warp_grid_cpu(width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> (Vec<f32>, Vec<f32>) {
    let n = (width * height) as usize;
    let mut warp_x = vec![0.0f32; n];
    let mut warp_y = vec![0.0f32; n];
    for gy in 0..height {
        for gx in 0..width {
            let i = (gy * width + gx) as usize;
            let xf = gx as f32 * wf;
            let yf = gy as f32 * wf;
            let qx = cartalith_noise::gpu_fbm(xf, yf, seed + 17);
            let qy = cartalith_noise::gpu_fbm(xf, yf, seed + 101);
            let wx = cartalith_noise::gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 213) - 0.5;
            let wy = cartalith_noise::gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 331) - 0.5;
            warp_x[i] = wx * 2.0 * amp;
            warp_y[i] = wy * 2.0 * amp;
        }
    }
    (warp_x, warp_y)
}

/// CPU reference for [`dispatch_gpu_heterogeneity`], including the
/// normalize pass -- directly comparable to the GPU path's own
/// dispatch-then-[`normalize_by_max_abs`] sequence.
pub fn gpu_heterogeneity_grid_cpu(
    width: u32,
    height: u32,
    hetero_seed: i32,
    scale: f32,
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
) -> Vec<f32> {
    let n = (width * height) as usize;
    let mut out = vec![0.0f32; n];
    for gy in 0..height {
        for gx in 0..width {
            let i = (gy * width + gx) as usize;
            let wx = gx as f32 + warp_x[i];
            let wy = gy as f32 + warp_y[i];
            let low_n = cartalith_noise::gpu_fbm(wx * scale, wy * scale, hetero_seed) - 0.5;
            out[i] = low_n * (0.3 + 0.7 * age[i]);
        }
    }
    normalize_by_max_abs(&mut out);
    out
}

/// CPU reference for [`dispatch_gpu_height`] -- calls `cartalith_noise::
/// gpu_fbm`/`gpu_ridged` directly, same shape as the other `*_grid_cpu`
/// twins above. Non-`world` case only, matching the GPU kernel. `oro`'s
/// absence is expressed as `has_oro: false` (the `oro` slice content is
/// then ignored, mirroring the shader's own `select()`), not by passing
/// an `Option` -- keeps this function's signature directly comparable to
/// [`dispatch_gpu_height`]'s own flat parameter list.
#[allow(clippy::too_many_arguments)]
pub fn gpu_height_grid_cpu(
    width: u32,
    height: u32,
    seed: i32,
    nf: f32,
    a: f32,
    b: f32,
    age_inf: f32,
    fwt: f32,
    hwt: f32,
    ridged: bool,
    has_oro: bool,
    base_field: &[f32],
    stress: &[f32],
    flex: &[f32],
    hetero: &[f32],
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
    oro: &[f32],
) -> Vec<f32> {
    let n = (width * height) as usize;
    let mut out = vec![0.0f32; n];
    for gy in 0..height {
        for gx in 0..width {
            let i = (gy * width + gx) as usize;
            let sf = stress[i];
            let t = if has_oro { oro[i] + sf.min(0.0) } else { sf };
            let bs = base_field[i];
            let rug = (-age[i] * (1.0 + age_inf * 6.0)).exp();
            let wx = gx as f32 + warp_x[i];
            let wy = gy as f32 + warp_y[i];
            let nx = wx * nf / width as f32;
            let ny = wy * nf / width as f32;
            let n_val = (if ridged { cartalith_noise::gpu_ridged(nx, ny, seed) } else { cartalith_noise::gpu_fbm(nx, ny, seed) }) - 0.5;
            out[i] = 0.5 + a * (0.40 * bs + 0.50 * t) + fwt * flex[i] + hwt * hetero[i] + b * n_val * (0.25 + 0.75 * rug);
        }
    }
    out
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: GPU-shape CPU twin for
/// [`dispatch_gpu_gauss_blur`] -- the same direct-sum-per-cell evaluation
/// the shader performs (not the CPU production `box_h`/`box_v`'s running-
/// sum-in-f64 optimization), so this is directly comparable to the GPU
/// kernel's own arithmetic. The REAL parity question (does this whole
/// approach match `cartalith_terrain::gauss_blur`, the untouched JS-
/// matching function) is answered by a dedicated test importing that
/// crate directly (`cartalith-terrain` as a dev-dependency here), not by
/// this function -- this one exists only so GPU-side determinism/
/// regression tests have a same-shape CPU comparison independent of the
/// three-way question.
pub fn gpu_gauss_blur_grid_cpu(src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Vec<f32> {
    if radius < 1.0 {
        return src.to_vec();
    }
    let w = width as i32;
    let h = height as i32;
    let pr = (radius / 1.6).round().max(1.0) as i32;
    let norm = 1.0 / (2.0 * pr as f32 + 1.0);

    let box_h = |input: &[f32], wrap: bool| -> Vec<f32> {
        let mut out = vec![0.0f32; input.len()];
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0.0f32;
                for k in -pr..=pr {
                    let idx = if wrap { ((x + k).rem_euclid(w)) as usize } else { (x + k).clamp(0, w - 1) as usize };
                    sum += input[(y * w) as usize + idx];
                }
                out[(y * w + x) as usize] = sum * norm;
            }
        }
        out
    };
    let box_v = |input: &[f32]| -> Vec<f32> {
        let mut out = vec![0.0f32; input.len()];
        for x in 0..w {
            for y in 0..h {
                let mut sum = 0.0f32;
                for k in -pr..=pr {
                    let idx = (y + k).clamp(0, h - 1);
                    sum += input[(idx * w + x) as usize];
                }
                out[(y * w + x) as usize] = sum * norm;
            }
        }
        out
    };

    let mut a = src.to_vec();
    for _ in 0..3 {
        let b = box_h(&a, wrap_x);
        a = box_v(&b);
    }
    a
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: CPU twin for
/// [`dispatch_gpu_resistance`] -- identical trivial formula, no precision
/// concerns either side.
pub fn gpu_resistance_grid_cpu(width: u32, height: u32, plate_id: &[u32], age: &[f32], crustal_per_plate: &[f32]) -> Vec<f32> {
    let n = (width * height) as usize;
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let crustal = crustal_per_plate[plate_id[i] as usize];
        out[i] = (crustal * 0.6 + age[i] * 0.4).min(1.0);
    }
    out
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

// ===================== GPU_LAYER_INTEGRATION_SCOPE.md milestone 6 =====================
//
// Public grid-level entry points for the four kernels a caller outside
// this crate (`cartalith-engine`) actually needs to invoke end-to-end:
// domain warp, crustal heterogeneity, flexure/base-field blur, and plate
// assignment. Each kernel's own *numerical correctness* was already
// separately verified in its own milestone's tests (2, 4, 5) -- what these
// wrappers add is the runtime `HARDWARE_ACCELERATION.md` §27 gate: `init_
// gpu_*()`'s own `Result` IS the self-test here (adapter/device/pipeline
// creation failing on THIS machine is exactly the failure mode a
// per-kernel self-test would also catch; re-deriving a bespoke numerical
// self-test per kernel would just re-prove what milestone 2/4/5's own
// tests already established). `None` means "GPU unavailable for this
// kernel right now" -- the caller falls back to the CPU function, never a
// panic (§27: "GPU failure must never crash Cartalith").
//
// A fresh `GpuContext` is created per call rather than cached across
// `generate_terrain()` invocations -- acceptable for a one-shot batch
// generation (`HARDWARE_ACCELERATION.md`'s own static-generation scope
// correction: no per-frame budget to protect), unlike a real-time app
// where context churn would matter far more.

/// Domain warp (`compute_warp`'s GPU sibling, milestone 2's `gpu_compute_
/// warp`/`gpu_fbm`). Returns `None` if GPU init fails for any reason.
pub fn warp_grid_gpu(width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> Option<(Vec<f32>, Vec<f32>)> {
    let ctx = init_gpu_warp().ok()?;
    Some(dispatch_gpu_warp(&ctx, width, height, seed, wf, amp))
}

/// Crustal heterogeneity's pre-normalize value (`compute_heterogeneity`'s
/// GPU sibling, milestone 2). The caller must still run the CPU max-reduce
/// normalize pass on the result -- this kernel only computes the raw
/// per-cell value, matching `compute_heterogeneity`'s own two-phase shape
/// (loop, then a separate normalize pass over the whole field).
pub fn heterogeneity_grid_gpu(
    width: u32,
    height: u32,
    hetero_seed: i32,
    scale: f32,
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
) -> Option<Vec<f32>> {
    let ctx = init_gpu_heterogeneity().ok()?;
    Some(dispatch_gpu_heterogeneity(&ctx, width, height, hetero_seed, scale, age, warp_x, warp_y))
}

/// `gauss_blur`'s GPU sibling (milestone 4) -- used for both `base_field`
/// and, via the caller wrapping `compute_flexure`'s own thin-blur logic,
/// `flexure_field`.
pub fn gauss_blur_grid_gpu(src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Option<Vec<f32>> {
    let ctx = init_gpu_gauss_blur().ok()?;
    Some(dispatch_gpu_gauss_blur(&ctx, src, radius, width, height, wrap_x))
}

/// Plate assignment via JFA (`assign_plates`'s GPU sibling, milestone 5).
/// **Not a port of the CPU function** -- the GPU path is double-buffered
/// (frozen-read/separate-write per pass), while the CPU function mutates
/// in-place mid-scan; milestone 5's own verification found the GPU
/// variant is actually *more* accurate against brute-force ground truth,
/// not just different. Returns plate indices as `i32`, matching `dispatch_
/// gpu_assign_plates`'s own output type (the caller casts to `usize` as
/// `assign_plates`'s own return type requires, after checking for `-1`/
/// unassigned cells the same way the CPU path's callers already must).
pub fn assign_plates_grid_gpu(
    width: u32,
    height: u32,
    plate_x: &[f32],
    plate_y: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Option<Vec<i32>> {
    let ctx = init_gpu_jfa_plates().ok()?;
    Some(dispatch_gpu_assign_plates(&ctx, width, height, plate_x, plate_y, warp_x, warp_y))
}

/// `_with` sibling of [`warp_grid_gpu`] -- builds its pipeline on an
/// already-created [`GpuDevice`] (milestone 8, context reuse across
/// `generate_terrain`'s several GPU stages) instead of requesting its own
/// adapter/device. Infallible past device creation, which the caller
/// already handled by holding a `GpuDevice` in the first place.
pub fn warp_grid_gpu_with(gpu: &GpuDevice, width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> (Vec<f32>, Vec<f32>) {
    let ctx = init_gpu_warp_with(gpu);
    dispatch_gpu_warp(&ctx, width, height, seed, wf, amp)
}

/// `_with` sibling of [`heterogeneity_grid_gpu`].
#[allow(clippy::too_many_arguments)]
pub fn heterogeneity_grid_gpu_with(
    gpu: &GpuDevice,
    width: u32,
    height: u32,
    hetero_seed: i32,
    scale: f32,
    age: &[f32],
    warp_x: &[f32],
    warp_y: &[f32],
) -> Vec<f32> {
    let ctx = init_gpu_heterogeneity_with(gpu);
    dispatch_gpu_heterogeneity(&ctx, width, height, hetero_seed, scale, age, warp_x, warp_y)
}

/// `_with` sibling of [`gauss_blur_grid_gpu`].
pub fn gauss_blur_grid_gpu_with(gpu: &GpuDevice, src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Vec<f32> {
    let ctx = init_gpu_gauss_blur_with(gpu);
    dispatch_gpu_gauss_blur(&ctx, src, radius, width, height, wrap_x)
}

/// `_with` sibling of [`assign_plates_grid_gpu`].
pub fn assign_plates_grid_gpu_with(
    gpu: &GpuDevice,
    width: u32,
    height: u32,
    plate_x: &[f32],
    plate_y: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Vec<i32> {
    let ctx = init_gpu_jfa_plates_with(gpu);
    dispatch_gpu_assign_plates(&ctx, width, height, plate_x, plate_y, warp_x, warp_y)
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

    // ===================== GPU_LAYER_INTEGRATION_SCOPE.md milestone 2 =====================
    // Domain warp + crustal heterogeneity on GPU. No JS-reference comparison
    // (there isn't one, per `DECISIONS.md` §7a/§7c) -- verified by GPU-side
    // determinism, statistical sanity, and (for warp) a written debug image.

    fn try_gpu_warp() -> Option<GpuContext> {
        init_gpu_warp().ok()
    }

    fn try_gpu_heterogeneity() -> Option<GpuContext> {
        init_gpu_heterogeneity().ok()
    }

    fn assert_finite_and_bounded(values: &[f32], lo: f32, hi: f32, what: &str) {
        for &v in values {
            assert!(v.is_finite(), "{what}: found non-finite value {v}");
            assert!((lo..=hi).contains(&v), "{what}: value {v} out of expected range [{lo},{hi}]");
        }
    }

    #[test]
    fn gpu_warp_matches_cpu_reference_at_real_field_size() {
        let Some(ctx) = try_gpu_warp() else {
            eprintln!("no GPU available -- skipping (requires real hardware)");
            return;
        };
        let (w, h) = (512u32, 512u32);
        let seed = 24601;
        let wf = 2.5 / w as f32;
        let amp = 40.0f32;
        let (gx, gy) = dispatch_gpu_warp(&ctx, w, h, seed, wf, amp);
        let (cx, cy) = gpu_warp_grid_cpu(w, h, seed, wf, amp);
        let mut max_abs_diff = 0.0f64;
        let mut mismatches = 0usize;
        for (g, c) in gx.iter().chain(gy.iter()).zip(cx.iter().chain(cy.iter())) {
            let d = ((*g as f64) - (*c as f64)).abs();
            if d > WARP_TOLERANCE {
                mismatches += 1;
            }
            if d > max_abs_diff {
                max_abs_diff = d;
            }
        }
        eprintln!(
            "gpu_warp GPU vs CPU at {w}x{h}: {mismatches}/{} cells (both x,y) exceed tol={WARP_TOLERANCE}, max_abs_diff={max_abs_diff}",
            (w * h * 2)
        );
        assert_eq!(mismatches, 0, "gpu_warp GPU/CPU diverged beyond {WARP_TOLERANCE} -- see max_abs_diff above");
        assert_finite_and_bounded(&gx, -2.0 * amp, 2.0 * amp, "warp_x");
        assert_finite_and_bounded(&gy, -2.0 * amp, 2.0 * amp, "warp_y");
    }

    #[test]
    fn gpu_warp_deterministic_across_runs() {
        let Some(ctx) = try_gpu_warp() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (a_x, a_y) = dispatch_gpu_warp(&ctx, 64, 64, 42, 0.05, 20.0);
        let (b_x, b_y) = dispatch_gpu_warp(&ctx, 64, 64, 42, 0.05, 20.0);
        assert_eq!(a_x, b_x, "gpu_warp not deterministic across runs (x)");
        assert_eq!(a_y, b_y, "gpu_warp not deterministic across runs (y)");
    }

    /// Qualitative sanity check (`DECISIONS.md` §7a: "judged by looking at
    /// it") -- writes a small grayscale PGM of the GPU warp_x field so it can
    /// actually be viewed, not just statistically summarized. PGM (not PNG)
    /// deliberately: trivial ASCII-header + raw-bytes format, no extra
    /// dependency for a one-off debug dump.
    #[test]
    fn gpu_warp_debug_image_written_for_visual_check() {
        let Some(ctx) = try_gpu_warp() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (256u32, 256u32);
        let (warp_x, _) = dispatch_gpu_warp(&ctx, w, h, 24601, 2.5 / w as f32, 40.0);
        let mut mn = f32::INFINITY;
        let mut mx = f32::NEG_INFINITY;
        for &v in &warp_x {
            mn = mn.min(v);
            mx = mx.max(v);
        }
        let range = (mx - mn).max(1e-6);
        let mut bytes = Vec::with_capacity((w * h) as usize);
        for &v in &warp_x {
            bytes.push((((v - mn) / range) * 255.0) as u8);
        }
        let path = std::env::temp_dir().join("cartalith_gpu_warp_debug.pgm");
        let mut file_contents = format!("P5\n{w} {h}\n255\n").into_bytes();
        file_contents.extend_from_slice(&bytes);
        std::fs::write(&path, &file_contents).expect("failed to write debug PGM");
        eprintln!("wrote GPU warp_x debug image to {} (range [{mn},{mx}]) -- open it to visually confirm no banding/lattice artifacts", path.display());
    }

    #[test]
    fn gpu_heterogeneity_matches_cpu_reference_at_real_field_size() {
        let Some(ctx) = try_gpu_heterogeneity() else {
            eprintln!("no GPU available -- skipping (requires real hardware)");
            return;
        };
        let (w, h) = (512u32, 512u32);
        let n = (w * h) as usize;
        let seed = 24601;
        let hetero_seed = seed ^ 0x44bb;
        let scale = 1.5 * 12.0 / w as f32; // representative hf/gw, hf ~ 1.5*terrain_detail_k
        // Real-shaped, deterministic synthetic age field (not all-zero/all-one,
        // which wouldn't exercise the `0.3 + 0.7*age` term meaningfully).
        let age: Vec<f32> = (0..n).map(|i| ((i * 2654435761u32 as usize) % 1000) as f32 / 1000.0).collect();
        let warp_x = vec![0.0f32; n]; // no-warp case: zero-filled, matches Option::None on CPU
        let warp_y = vec![0.0f32; n];

        let mut gpu = dispatch_gpu_heterogeneity(&ctx, w, h, hetero_seed, scale, &age, &warp_x, &warp_y);
        normalize_by_max_abs(&mut gpu);
        let cpu = gpu_heterogeneity_grid_cpu(w, h, hetero_seed, scale, &age, &warp_x, &warp_y);

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
            "gpu_heterogeneity GPU vs CPU at {w}x{h}: {mismatches}/{n} cells exceed tol={GPU_SAFE_NOISE_TOLERANCE}, max_abs_diff={max_abs_diff}"
        );
        assert_eq!(mismatches, 0, "gpu_heterogeneity GPU/CPU diverged beyond {GPU_SAFE_NOISE_TOLERANCE} -- see max_abs_diff above");
        assert_finite_and_bounded(&gpu, -1.0, 1.0, "heterogeneity (post-normalize)");
    }

    #[test]
    fn gpu_heterogeneity_deterministic_across_runs() {
        let Some(ctx) = try_gpu_heterogeneity() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let n = 32 * 32;
        let age = vec![0.5f32; n];
        let warp_x = vec![0.0f32; n];
        let warp_y = vec![0.0f32; n];
        let mut a = dispatch_gpu_heterogeneity(&ctx, 32, 32, 42, 0.1, &age, &warp_x, &warp_y);
        let mut b = dispatch_gpu_heterogeneity(&ctx, 32, 32, 42, 0.1, &age, &warp_x, &warp_y);
        normalize_by_max_abs(&mut a);
        normalize_by_max_abs(&mut b);
        assert_eq!(a, b, "gpu_heterogeneity not deterministic across runs");
    }

    /// Real timing at the pilot's own tested sizes -- milestone 1's numbers
    /// were for the bare noise kernel; these two functions do meaningfully
    /// more per-cell work (up to 4 `gpu_fbm` calls per warp cell, each 6
    /// octaves), so the ratios are not assumed to carry over unchanged.
    #[test]
    fn measured_gpu_warp_vs_cpu_timing() {
        let Some(ctx) = try_gpu_warp() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        let _ = dispatch_gpu_warp(&ctx, 8, 8, 1, 0.5, 10.0); // warm up
        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let wf = 2.5 / w as f32;
            let amp = 40.0f32;

            let t0 = Instant::now();
            let _ = dispatch_gpu_warp(&ctx, w, h, seed, wf, amp);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = gpu_warp_grid_cpu(w, h, seed, wf, amp);
            let cpu_time = t1.elapsed();

            eprintln!(
                "gpu_warp {w}x{h} ({} cells): GPU dispatch+readback = {:?}, CPU (single-thread) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    #[test]
    fn measured_gpu_heterogeneity_vs_cpu_timing() {
        let Some(ctx) = try_gpu_heterogeneity() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        let warm_n = 64;
        let warm_age = vec![0.5f32; warm_n];
        let warm_zero = vec![0.0f32; warm_n];
        let _ = dispatch_gpu_heterogeneity(&ctx, 8, 8, 1, 0.5, &warm_age, &warm_zero, &warm_zero); // warm up

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let hetero_seed = 24601 ^ 0x44bb;
            let scale = 1.5 * 12.0 / w as f32;
            let age = vec![0.6f32; n];
            let warp_x = vec![0.0f32; n];
            let warp_y = vec![0.0f32; n];

            let t0 = Instant::now();
            let _ = dispatch_gpu_heterogeneity(&ctx, w, h, hetero_seed, scale, &age, &warp_x, &warp_y);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = gpu_heterogeneity_grid_cpu(w, h, hetero_seed, scale, &age, &warp_x, &warp_y);
            let cpu_time = t1.elapsed();

            eprintln!(
                "gpu_heterogeneity {w}x{h} ({} cells): GPU dispatch+readback = {:?}, CPU (single-thread) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    // GPU_LAYER_INTEGRATION_SCOPE.md milestone 3: the height formula
    // itself. Same verification shape as warp/heterogeneity above -- no
    // JS-reference comparison (§7c), GPU-side determinism + statistical
    // sanity + a debug image + real timing.

    fn try_gpu_height() -> Option<GpuContext> {
        init_gpu_height().ok()
    }

    /// `(base, stress, flex, hetero, age)` -- named to avoid a 5-tuple of
    /// `Vec<f32>` tripping `clippy::type_complexity` at every call site.
    type HeightTestFields = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>);

    /// Deterministic synthetic input fields sized `w*h`, distinct per field
    /// so a mis-wired binding (e.g. `stress` accidentally reading `flex`'s
    /// buffer) would show up as a wrong-shaped result rather than silently
    /// matching by coincidence -- same reasoning `gpu_heterogeneity`'s own
    /// non-trivial synthetic `age` field already applied.
    fn synthetic_height_inputs(n: usize) -> HeightTestFields {
        let base: Vec<f32> = (0..n).map(|i| ((i * 2654435761u32 as usize) % 1000) as f32 / 1000.0 - 0.5).collect();
        let stress: Vec<f32> = (0..n).map(|i| ((i * 40503u32 as usize) % 1000) as f32 / 1000.0 - 0.5).collect();
        let flex: Vec<f32> = (0..n).map(|i| ((i * 2246822519u32 as usize) % 1000) as f32 / 1000.0 * 0.2).collect();
        let hetero: Vec<f32> = (0..n).map(|i| ((i * 3266489917u32 as usize) % 1000) as f32 / 1000.0 - 0.5).collect();
        let age: Vec<f32> = (0..n).map(|i| ((i * 668265263u32 as usize) % 1000) as f32 / 1000.0).collect();
        (base, stress, flex, hetero, age)
    }

    #[test]
    fn gpu_height_matches_cpu_reference_at_real_field_size() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping (requires real hardware)");
            return;
        };
        let (w, h) = (512u32, 512u32);
        let n = (w * h) as usize;
        let seed = 24601;
        let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
        let warp_x = vec![0.0f32; n];
        let warp_y = vec![0.0f32; n];
        let oro = vec![0.0f32; n]; // has_oro=false below -- content ignored, matches shader's select()
        let (nf, a, b, age_inf, fwt, hwt) = (5.0f32, 0.5f32, 0.3f32, 0.5f32, 0.15f32, 0.1f32);

        for ridged in [false, true] {
            let gpu = dispatch_gpu_height(
                &ctx, w, h, seed, nf, a, b, age_inf, fwt, hwt, ridged, false, &base, &stress, &flex, &hetero, &age,
                &warp_x, &warp_y, &oro,
            );
            let cpu = gpu_height_grid_cpu(
                w, h, seed, nf, a, b, age_inf, fwt, hwt, ridged, false, &base, &stress, &flex, &hetero, &age, &warp_x,
                &warp_y, &oro,
            );
            let mut max_abs_diff = 0.0f64;
            let mut mismatches = 0usize;
            for (g, c) in gpu.iter().zip(cpu.iter()) {
                let d = ((*g as f64) - (*c as f64)).abs();
                if d > HEIGHT_TOLERANCE {
                    mismatches += 1;
                }
                if d > max_abs_diff {
                    max_abs_diff = d;
                }
            }
            eprintln!(
                "gpu_height (ridged={ridged}) GPU vs CPU at {w}x{h}: {mismatches}/{n} cells exceed tol={HEIGHT_TOLERANCE}, max_abs_diff={max_abs_diff}"
            );
            assert_eq!(
                mismatches, 0,
                "gpu_height (ridged={ridged}) GPU/CPU diverged beyond {HEIGHT_TOLERANCE} -- see max_abs_diff above"
            );
            assert_finite_and_bounded(&gpu, -10.0, 10.0, "height");
        }
    }

    #[test]
    fn gpu_height_has_oro_true_changes_the_formula() {
        // Regression guard for the has_oro branch specifically: an oro field
        // that actually differs from `min(stress, 0)` must change the output,
        // proving the shader's `select()` genuinely branches rather than
        // silently ignoring the oro buffer either way.
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (16u32, 16u32);
        let n = (w * h) as usize;
        let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
        let warp_x = vec![0.0f32; n];
        let warp_y = vec![0.0f32; n];
        let oro_zero = vec![0.0f32; n];
        let oro_large = vec![5.0f32; n]; // deliberately far from min(stress,0)
        let (nf, a, b, age_inf, fwt, hwt) = (5.0f32, 0.5f32, 0.3f32, 0.5f32, 0.15f32, 0.1f32);

        let without_oro = dispatch_gpu_height(
            &ctx, w, h, 1, nf, a, b, age_inf, fwt, hwt, false, false, &base, &stress, &flex, &hetero, &age, &warp_x,
            &warp_y, &oro_zero,
        );
        let with_oro = dispatch_gpu_height(
            &ctx, w, h, 1, nf, a, b, age_inf, fwt, hwt, false, true, &base, &stress, &flex, &hetero, &age, &warp_x,
            &warp_y, &oro_large,
        );
        assert_ne!(without_oro, with_oro, "has_oro=true with a distinctly different oro field produced identical output -- select() branch not exercised");
    }

    #[test]
    fn gpu_height_deterministic_across_runs() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let n = 32 * 32;
        let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
        let warp_x = vec![0.0f32; n];
        let warp_y = vec![0.0f32; n];
        let oro = vec![0.0f32; n];
        let a = dispatch_gpu_height(
            &ctx, 32, 32, 42, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
            &warp_x, &warp_y, &oro,
        );
        let b = dispatch_gpu_height(
            &ctx, 32, 32, 42, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
            &warp_x, &warp_y, &oro,
        );
        assert_eq!(a, b, "gpu_height not deterministic across runs");
    }

    /// Qualitative sanity check (`DECISIONS.md` §7a) -- writes a small
    /// grayscale PGM of a real GPU height field.
    #[test]
    fn gpu_height_debug_image_written_for_visual_check() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (256u32, 256u32);
        let n = (w * h) as usize;
        let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
        let warp_x = vec![0.0f32; n];
        let warp_y = vec![0.0f32; n];
        let oro = vec![0.0f32; n];
        let height = dispatch_gpu_height(
            &ctx, w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
            &warp_x, &warp_y, &oro,
        );
        let mut mn = f32::INFINITY;
        let mut mx = f32::NEG_INFINITY;
        for &v in &height {
            mn = mn.min(v);
            mx = mx.max(v);
        }
        let range = (mx - mn).max(1e-6);
        let mut bytes = Vec::with_capacity(n);
        for &v in &height {
            bytes.push((((v - mn) / range) * 255.0) as u8);
        }
        let path = std::env::temp_dir().join("cartalith_gpu_height_debug.pgm");
        let mut file_contents = format!("P5\n{w} {h}\n255\n").into_bytes();
        file_contents.extend_from_slice(&bytes);
        std::fs::write(&path, &file_contents).expect("failed to write debug PGM");
        eprintln!("wrote GPU height debug image to {} (range [{mn},{mx}]) -- open it to visually confirm no banding/lattice artifacts", path.display());
    }

    #[test]
    fn measured_gpu_height_vs_cpu_timing() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        let (warm_base, warm_stress, warm_flex, warm_hetero, warm_age) = synthetic_height_inputs(64);
        let warm_zero = vec![0.0f32; 64];
        let _ = dispatch_gpu_height(
            &ctx, 8, 8, 1, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &warm_base, &warm_stress, &warm_flex,
            &warm_hetero, &warm_age, &warm_zero, &warm_zero, &warm_zero,
        ); // warm up

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
            let warp_x = vec![0.0f32; n];
            let warp_y = vec![0.0f32; n];
            let oro = vec![0.0f32; n];

            let t0 = Instant::now();
            let _ = dispatch_gpu_height(
                &ctx, w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
                &warp_x, &warp_y, &oro,
            );
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = gpu_height_grid_cpu(
                w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
                &warp_x, &warp_y, &oro,
            );
            let cpu_time = t1.elapsed();

            eprintln!(
                "gpu_height {w}x{h} ({} cells): GPU dispatch+readback = {:?}, CPU (single-thread) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    // ===================== GPU_LAYER_INTEGRATION_SCOPE.md milestone 4 =====================
    // gauss_blur + compute_resistance. Neither touches noise -- the
    // headline question each test below answers is whether that reaches
    // genuine three-way JS/CPU/GPU parity (comparing directly against the
    // real, untouched cartalith_terrain functions) or needs its own
    // GPU-vs-CPU-twin carve-out like milestones 1-3.

    fn try_gpu_resistance() -> Option<GpuContext> {
        init_gpu_resistance().ok()
    }

    fn try_gpu_blur() -> Option<GpuBlurContext> {
        init_gpu_gauss_blur().ok()
    }

    /// Deterministic pseudo-random-looking f32 field in [0,1] -- not real
    /// noise (no dependency on `cartalith-noise` needed here), just enough
    /// spatial variation that a mis-wired buffer or a swapped x/y index
    /// would show up as a real mismatch rather than passing by coincidence
    /// on a flat/uniform field.
    fn synthetic_field(n: usize, salt: u32) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let x = (i as u32).wrapping_mul(2654435761).wrapping_add(salt);
                ((x >> 8) & 0xFFFF) as f32 / 65535.0
            })
            .collect()
    }

    #[test]
    fn gpu_gauss_blur_matches_real_cpu_gauss_blur() {
        let Some(ctx) = try_gpu_blur() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (512u32, 512u32);
        let n = (w * h) as usize;
        let src = synthetic_field(n, 7);

        for &(radius, wrap) in &[(12.0f64, false), (12.0, true), (48.0, false)] {
            let gpu = dispatch_gpu_gauss_blur(&ctx, &src, radius, w, h, wrap);
            // The REAL comparison: the untouched, JS-matching CPU function,
            // not a GPU-shaped twin.
            let real_cpu = cartalith_terrain::gauss_blur(&src, radius, w as usize, h as usize, wrap);

            let mut max_abs_diff = 0.0f64;
            let mut mismatches = 0usize;
            for (g, c) in gpu.iter().zip(real_cpu.iter()) {
                let d = ((*g as f64) - (*c as f64)).abs();
                if d > BLUR_TOLERANCE {
                    mismatches += 1;
                }
                if d > max_abs_diff {
                    max_abs_diff = d;
                }
            }
            eprintln!(
                "gpu_gauss_blur vs REAL cartalith_terrain::gauss_blur, radius={radius} wrap={wrap}, {w}x{h}: {mismatches}/{n} cells exceed tol={BLUR_TOLERANCE}, max_abs_diff={max_abs_diff}"
            );
            assert_eq!(
                mismatches, 0,
                "gpu_gauss_blur diverged from the REAL CPU gauss_blur beyond {BLUR_TOLERANCE} at radius={radius} wrap={wrap} -- see max_abs_diff above"
            );
        }
    }

    #[test]
    fn gpu_gauss_blur_matches_gpu_shaped_cpu_twin() {
        // Independent of the three-way question above: confirms the GPU
        // kernel itself is internally correct against a same-shape CPU
        // reimplementation, so a three-way regression (if the real CPU
        // gauss_blur's own behaviour ever legitimately changes) doesn't
        // silently take this kernel's own correctness signal down with it.
        let Some(ctx) = try_gpu_blur() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (256u32, 256u32);
        let n = (w * h) as usize;
        let src = synthetic_field(n, 11);
        let gpu = dispatch_gpu_gauss_blur(&ctx, &src, 10.0, w, h, false);
        let twin = gpu_gauss_blur_grid_cpu(&src, 10.0, w, h, false);
        for (g, t) in gpu.iter().zip(twin.iter()) {
            assert!((g - t).abs() < 1e-4, "gpu vs same-shape CPU twin diverged: {g} vs {t}");
        }
    }

    #[test]
    fn gpu_gauss_blur_r_below_one_is_unmodified_copy() {
        let Some(ctx) = try_gpu_blur() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let src = synthetic_field(64, 3);
        let out = dispatch_gpu_gauss_blur(&ctx, &src, 0.5, 8, 8, false);
        assert_eq!(out, src, "radius<1 must be an unmodified copy, matching cartalith_terrain::gauss_blur's own early exit");
    }

    #[test]
    fn gpu_gauss_blur_deterministic_across_runs() {
        let Some(ctx) = try_gpu_blur() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let src = synthetic_field(64 * 64, 5);
        let a = dispatch_gpu_gauss_blur(&ctx, &src, 12.0, 64, 64, false);
        let b = dispatch_gpu_gauss_blur(&ctx, &src, 12.0, 64, 64, false);
        assert_eq!(a, b, "same input must produce identical GPU output across runs");
    }

    #[test]
    fn gpu_compute_resistance_matches_real_cpu_compute_resistance() {
        let Some(ctx) = try_gpu_resistance() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (512u32, 512u32);
        let n = (w * h) as usize;
        let num_plates = 9usize;
        let plate_id_usize: Vec<usize> = (0..n).map(|i| i % num_plates).collect();
        let plate_id_u32: Vec<u32> = plate_id_usize.iter().map(|&p| p as u32).collect();
        let age = synthetic_field(n, 13);
        // Real Plate structs, some with negative base (oceanic crust) --
        // compute_resistance's `.max(0.0)` clamp only matters if a real
        // test exercises a negative value, not just positive ones.
        let plates: Vec<cartalith_terrain::Plate> = (0..num_plates)
            .map(|k| cartalith_terrain::Plate {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                base: (k as f64 - 4.0) * 0.3, // spans negative and positive
            })
            .collect();
        let crustal_per_plate: Vec<f32> = plates.iter().map(|p| p.base.max(0.0) as f32).collect();

        let gpu = dispatch_gpu_resistance(&ctx, w, h, &plate_id_u32, &age, &crustal_per_plate);
        let real_cpu = cartalith_terrain::compute_resistance(w as usize, h as usize, &plate_id_usize, &plates, &age);

        let mut max_abs_diff = 0.0f64;
        let mut mismatches = 0usize;
        for (g, c) in gpu.iter().zip(real_cpu.iter()) {
            let d = ((*g as f64) - (*c as f64)).abs();
            if d > RESISTANCE_TOLERANCE {
                mismatches += 1;
            }
            if d > max_abs_diff {
                max_abs_diff = d;
            }
        }
        eprintln!(
            "gpu_compute_resistance vs REAL cartalith_terrain::compute_resistance, {w}x{h}: {mismatches}/{n} cells exceed tol={RESISTANCE_TOLERANCE}, max_abs_diff={max_abs_diff}"
        );
        assert_eq!(mismatches, 0, "gpu_compute_resistance diverged from the REAL CPU function beyond {RESISTANCE_TOLERANCE}");
    }

    fn try_gpu_jfa_plates() -> Option<GpuContext> {
        init_gpu_jfa_plates().ok()
    }

    /// Deterministic scattered plate positions across a `w`x`h` grid --
    /// real spatial spread (not all-at-origin like the resistance test's
    /// simpler setup), needed for JFA's nearest-seed behaviour to actually
    /// be exercised.
    fn scattered_plates(np: usize, w: u32, h: u32, salt: u32) -> (Vec<f32>, Vec<f32>) {
        let mut px = Vec::with_capacity(np);
        let mut py = Vec::with_capacity(np);
        for p in 0..np {
            let hx = (p as u32).wrapping_mul(2654435761).wrapping_add(salt);
            let hy = (p as u32).wrapping_mul(40503).wrapping_add(salt).wrapping_mul(2246822519);
            px.push(((hx >> 8) & 0xFFFF) as f32 / 65535.0 * w as f32);
            py.push(((hy >> 8) & 0xFFFF) as f32 / 65535.0 * h as f32);
        }
        (px, py)
    }

    /// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 5's actual headline
    /// question: `assign_plates` (CPU, in-place-mutation JFA) and
    /// [`dispatch_gpu_assign_plates`] (GPU, double-buffered JFA) are two
    /// DIFFERENT completions of the jump-flood algorithm -- three-way
    /// JS/CPU/GPU exact parity (milestone 4's result for `gauss_blur`/
    /// `compute_resistance`) is not expected here, and this test doesn't
    /// attempt it. Instead: measure each variant's real mismatch rate
    /// against brute-force exact-nearest-plate ground truth, and measure
    /// GPU-vs-CPU directly, all with real numbers rather than an assumed
    /// "GPU-vs-CPU-twin, ignore CPU" framing.
    #[test]
    fn gpu_jfa_plates_vs_cpu_jfa_vs_brute_force_ground_truth() {
        let Some(ctx) = try_gpu_jfa_plates() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        for (w, h, np, salt) in [(512u32, 512u32, 14usize, 71u32), (512, 512, 40, 137), (1024, 768, 22, 5)] {
            let n = (w * h) as usize;
            let (px, py) = scattered_plates(np, w, h, salt);

            let gpu = dispatch_gpu_assign_plates(&ctx, w, h, &px, &py, None, None);
            let truth = brute_force_nearest_plate(w, h, &px, &py, None, None);

            let plates: Vec<cartalith_terrain::Plate> =
                px.iter().zip(py.iter()).map(|(&x, &y)| cartalith_terrain::Plate {
                    x: x as f64,
                    y: y as f64,
                    vx: 0.0,
                    vy: 0.0,
                    base: 0.0,
                }).collect();
            let cpu = cartalith_terrain::assign_plates(w as usize, h as usize, false, &plates, None, None);

            let mut gpu_vs_truth = 0usize;
            let mut cpu_vs_truth = 0usize;
            let mut gpu_vs_cpu = 0usize;
            for i in 0..n {
                if gpu[i] != truth[i] {
                    gpu_vs_truth += 1;
                }
                if cpu[i] as i32 != truth[i] {
                    cpu_vs_truth += 1;
                }
                if gpu[i] != cpu[i] as i32 {
                    gpu_vs_cpu += 1;
                }
            }
            eprintln!(
                "gpu_jfa_plates {w}x{h} ({np} plates, salt={salt}): gpu_vs_brute_force={gpu_vs_truth}/{n} ({:.4}%), cpu_vs_brute_force={cpu_vs_truth}/{n} ({:.4}%), gpu_vs_cpu={gpu_vs_cpu}/{n} ({:.4}%)",
                100.0 * gpu_vs_truth as f64 / n as f64,
                100.0 * cpu_vs_truth as f64 / n as f64,
                100.0 * gpu_vs_cpu as f64 / n as f64,
            );
            // Both JFA variants are *approximations* of brute-force nearest
            // -- a small, real mismatch rate (boundary cells equidistant or
            // near-equidistant between two plates) is expected and correct
            // behaviour, not a bug. JFA's own literature puts this at a
            // fraction of a percent for reasonable seed counts/grid sizes;
            // assert a generous ceiling that would catch a genuinely broken
            // implementation (e.g. a mis-wired buffer producing near-random
            // output) without false-failing on JFA's known, real
            // approximation error.
            assert!(
                (gpu_vs_truth as f64 / n as f64) < 0.05,
                "GPU JFA mismatch rate against brute-force truth ({:.4}%) at {w}x{h}/{np} plates is far higher than JFA's known approximation error -- likely a real bug, not expected imprecision",
                100.0 * gpu_vs_truth as f64 / n as f64
            );
        }
    }

    #[test]
    fn gpu_jfa_plates_determinism() {
        let Some(ctx) = try_gpu_jfa_plates() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (w, h) = (256u32, 256u32);
        let (px, py) = scattered_plates(9, w, h, 5);
        let a = dispatch_gpu_assign_plates(&ctx, w, h, &px, &py, None, None);
        let b = dispatch_gpu_assign_plates(&ctx, w, h, &px, &py, None, None);
        assert_eq!(a, b, "same input must produce identical GPU JFA output across runs");
    }

    /// Real timing, same honest methodology every prior milestone used.
    /// JFA's pass count scales with `log2(max(w,h))`, not O(1) like the
    /// single-pass kernels milestones 1-4 measured -- report what's
    /// actually measured, don't assume the earlier ratios carry over.
    #[test]
    fn measured_gpu_jfa_plates_vs_cpu_timing() {
        let Some(ctx) = try_gpu_jfa_plates() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        // Warm up: first dispatch pays one-time pipeline/driver JIT cost.
        let (wx, wy) = scattered_plates(8, 64, 64, 1);
        let _ = dispatch_gpu_assign_plates(&ctx, 64, 64, &wx, &wy, None, None);

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let np = 24usize;
            let (px, py) = scattered_plates(np, w, h, 999);
            let plates: Vec<cartalith_terrain::Plate> =
                px.iter().zip(py.iter()).map(|(&x, &y)| cartalith_terrain::Plate {
                    x: x as f64,
                    y: y as f64,
                    vx: 0.0,
                    vy: 0.0,
                    base: 0.0,
                }).collect();

            let t0 = Instant::now();
            let _ = dispatch_gpu_assign_plates(&ctx, w, h, &px, &py, None, None);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = cartalith_terrain::assign_plates(w as usize, h as usize, false, &plates, None, None);
            let cpu_time = t1.elapsed();

            let max_dim = w.max(h) as f64;
            let passes = max_dim.log2().ceil() as u32;
            eprintln!(
                "{w}x{h} ({} cells, {passes} JFA passes, {np} plates): GPU dispatch+readback = {:?}, CPU (single-thread, in-place JFA) = {:?}, ratio (CPU/GPU) = {:.2}x",
                w * h,
                gpu_time,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
    }

    #[test]
    fn gpu_compute_resistance_deterministic_across_runs() {
        let Some(ctx) = try_gpu_resistance() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let n = 64 * 64;
        let plate_id: Vec<u32> = (0..n).map(|i| (i % 5) as u32).collect();
        let age = synthetic_field(n, 17);
        let crustal_per_plate = vec![0.2f32, 0.5, 0.0, 0.9, 0.35];
        let a = dispatch_gpu_resistance(&ctx, 64, 64, &plate_id, &age, &crustal_per_plate);
        let b = dispatch_gpu_resistance(&ctx, 64, 64, &plate_id, &age, &crustal_per_plate);
        assert_eq!(a, b, "same input must produce identical GPU output across runs");
    }

    #[test]
    fn measured_gpu_blur_and_resistance_timing() {
        let (Some(blur_ctx), Some(res_ctx)) = (try_gpu_blur(), try_gpu_resistance()) else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let src = synthetic_field(n, 23);

            let t0 = Instant::now();
            let _ = dispatch_gpu_gauss_blur(&blur_ctx, &src, 12.0, w, h, false);
            let gpu_blur_time = t0.elapsed();
            let t1 = Instant::now();
            let _ = cartalith_terrain::gauss_blur(&src, 12.0, w as usize, h as usize, false);
            let cpu_blur_time = t1.elapsed();
            eprintln!(
                "gpu_gauss_blur {w}x{h}: GPU = {:?}, CPU (real, running-sum) = {:?}, ratio (CPU/GPU) = {:.2}x",
                gpu_blur_time,
                cpu_blur_time,
                cpu_blur_time.as_secs_f64() / gpu_blur_time.as_secs_f64().max(1e-9)
            );

            let num_plates = 9usize;
            let plate_id_u32: Vec<u32> = (0..n).map(|i| (i % num_plates) as u32).collect();
            let plate_id_usize: Vec<usize> = plate_id_u32.iter().map(|&p| p as usize).collect();
            let age = synthetic_field(n, 29);
            let plates: Vec<cartalith_terrain::Plate> = (0..num_plates)
                .map(|k| cartalith_terrain::Plate { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, base: (k as f64 - 4.0) * 0.3 })
                .collect();
            let crustal_per_plate: Vec<f32> = plates.iter().map(|p| p.base.max(0.0) as f32).collect();

            let t2 = Instant::now();
            let _ = dispatch_gpu_resistance(&res_ctx, w, h, &plate_id_u32, &age, &crustal_per_plate);
            let gpu_res_time = t2.elapsed();
            let t3 = Instant::now();
            let _ = cartalith_terrain::compute_resistance(w as usize, h as usize, &plate_id_usize, &plates, &age);
            let cpu_res_time = t3.elapsed();
            eprintln!(
                "gpu_compute_resistance {w}x{h}: GPU = {:?}, CPU (real) = {:?}, ratio (CPU/GPU) = {:.2}x",
                gpu_res_time,
                cpu_res_time,
                cpu_res_time.as_secs_f64() / gpu_res_time.as_secs_f64().max(1e-9)
            );
        }
    }
}
