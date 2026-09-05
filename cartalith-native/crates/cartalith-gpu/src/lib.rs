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

/// Multi-GPU enumeration, selection, VRAM budgeting and split-tiles
/// dispatch (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04/PR-05). Re-exported
/// flat, like everything else this crate offers.
mod multi;

/// Shared by this crate's unit tests and by `tests/multi_gpu.rs`, which pulls
/// the same file in with `#[path]`. Test-only, so it never reaches the crate's
/// public surface.
#[cfg(test)]
mod timing_harness;
pub use multi::*;

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
const SHADER_SRC_GPU_WEATHER: &str = include_str!("../shaders/gpu_weather.wgsl");
/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9: D8 flow accumulation,
/// the first genuinely *sequential* CPU algorithm in this pipeline to get
/// a parallel redesign rather than a port. `cartalith_hydrology::
/// compute_flow` sorts all `n` cells by descending height and walks that
/// order; this kernel replaces both the sort and the walk with (1) a
/// per-cell flow-direction pass and (2) `ceil(log2(n))` pointer-doubling
/// rounds over the resulting receiver forest. See the shader's own header
/// for the literature this follows and [`dispatch_gpu_flow`] for the
/// fixed-point accumulation choice that keeps it deterministic.
const SHADER_SRC_GPU_FLOW: &str = include_str!("../shaders/gpu_flow.wgsl");

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

/// **Relative** tolerance for GPU flow accumulation against the real
/// `cartalith_hydrology::compute_flow` -- the only tolerance in this crate
/// that is relative rather than absolute, and the only one comparing two
/// genuinely *different algorithms* rather than two implementations of the
/// same one (see `gpu_flow.wgsl`'s header). Accumulations here span six
/// orders of magnitude within a single grid (1.0 at a ridge crest, ~4.2e6
/// at a 2048x2048 outlet), so an absolute bound would be meaningless at one
/// end or vacuous at the other; the comparison is `|gpu-cpu| / max(cpu,1)`.
///
/// Two error sources, and they have opposite shapes -- which is why this
/// constant is defined against a *threshold* rather than against every
/// cell. The CPU carries a long chain of per-write `f32` roundings
/// (`acc[best] = (acc[best] as f64 + acc[i] as f64) as f32`, thousands deep
/// at a major outlet), so its error grows with accumulation. The GPU rounds
/// each seed once into fixed point and is exact from then on (integer
/// addition), so its absolute error is bounded by half a quantization step
/// per contributing cell -- which makes its *relative* error worst at tiny
/// accumulations built from many small seeds, and shrink as accumulation
/// grows.
///
/// Measured on real generated worlds
/// (`examples/flow_downstream_settlements`): the worst relative error in the
/// whole grid lands on cells accumulating ~1.2 units (2.3e-4 at 512x512,
/// 2.6e-3 at 1024x1024), while at and above the channel-initiation
/// threshold `river_flow_thresh` -- the only regime any downstream consumer
/// actually distinguishes -- it is 1.3e-4 and 3.3e-4 respectively. This
/// constant bounds that second, load-bearing regime;
/// [`FLOW_ANY_CELL_TOLERANCE`] is the loose guard over everything else.
pub const FLOW_TOLERANCE: f64 = 1e-3;

/// Guard on the sub-threshold cells [`FLOW_TOLERANCE`] deliberately
/// excludes -- accumulations of a few units, where a handful of
/// individually-rounded seeds is the entire value, so quantization is the
/// entire error. Not a hydrologically meaningful regime (nothing in this
/// pipeline distinguishes an accumulation of 1.27 from one of 1.28), but
/// bounded rather than unchecked, so a wrong fixed-point scale or a lost
/// doubling round still fails a test.
pub const FLOW_ANY_CELL_TOLERANCE: f64 = 5e-3;

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
    /// Multi-GPU split-tiles: world row of this band's first row. `0` for
    /// every single-device caller, which is bit-identical to the
    /// pre-split kernel -- see the shader's own comment.
    y_offset: u32,
    /// Rows this dispatch actually writes (`height` for the whole grid).
    band_rows: u32,
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

/// Matches `gpu_flow.wgsl`'s `FlowParams` field-for-field -- 4 x u32 = 16
/// bytes, already a multiple of the common uniform-buffer alignment (see
/// [`WarpParams`]' own comment on why that matters).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct FlowGpuParams {
    width: u32,
    height: u32,
    world: u32,
    _pad0: u32,
}

/// Matches `gpu_weather.wgsl`'s `WeatherParams` field-for-field, including
/// the trailing `_pad` (12 fields, already a multiple of 4 x 4 bytes = 48
/// bytes -- no extra padding needed beyond the one explicit `f32` field).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct WeatherParams {
    ww: u32,
    wh: u32,
    wrap_x: u32,
    bulk_evap: u32,
    sea: f32,
    ocean_hum: f32,
    evap_c: f32,
    ocean_c: f32,
    rain_k: f32,
    dry: f32,
    step: f32,
    _pad: f32,
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
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
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

/// `gpu_weather.wgsl`'s three entry points (`evap_main`/`advect_main`/
/// `deposit_main`) -- one shared bind group layout across all three,
/// [`GpuBlurContext`]'s "one shader module, many pipelines" shape
/// generalized from two pipelines to three. `w`/`w2`/`rain` are
/// `read_write` even though a given pass only ever touches a subset (e.g.
/// `advect_main` only writes `w2`) -- the layout is fixed once and shared
/// by every pipeline built from it, so it has to satisfy the neediest pass.
/// `gpu_flow.wgsl`'s three entry points (`dir_main`/`scatter_main`/
/// `merge_main`) -- same "one shader module, one layout, several
/// pipelines" shape as [`WEATHER_LAYOUT`]. `acc`/`delta` are declared
/// `atomic<u32>` on the shader side; a bind group layout has no way to
/// express that (an atomic storage array is just a storage buffer as far
/// as the layout is concerned), so they appear here as ordinary
/// `read_write` storage entries.
const FLOW_LAYOUT: [wgpu::BindGroupLayoutEntry; 7] = [
    uniform_entry(0),
    storage_entry(1, true),  // field (heights, read-only throughout)
    storage_entry(2, false), // recv (steepest-descent receiver, written once by dir_main)
    storage_entry(3, false), // ptr (the doubling pointer)
    storage_entry(4, false), // ptr_next (next round's pointer, written by scatter_main)
    storage_entry(5, false), // acc (atomic<u32> fixed-point accumulation)
    storage_entry(6, false), // delta (atomic<u32> per-round deliveries)
];

const WEATHER_LAYOUT: [wgpu::BindGroupLayoutEntry; 9] = [
    uniform_entry(0),
    storage_entry(1, true),  // eh (static elevation, coarse grid)
    storage_entry(2, true),  // tc (static temperature, coarse grid)
    storage_entry(3, true),  // sst_evap (static)
    storage_entry(4, true),  // wx (static wind, frozen for the whole call)
    storage_entry(5, true),  // wy
    storage_entry(6, false), // w (evap: read_write in place; advect: read; deposit: write)
    storage_entry(7, false), // w2 (advect: write; deposit: read)
    storage_entry(8, false), // rain (deposit: read_write accumulate)
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
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    box_h_pipeline: wgpu::ComputePipeline,
    box_v_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7: `simulate_weather`'s
/// inner loop -- three pipelines (`evap_main`/`advect_main`/`deposit_main`)
/// from one shared bind group layout, `GpuBlurContext`'s two-pipeline shape
/// generalized to three. Built only via [`init_gpu_weather_with`] (a
/// shared [`GpuDevice`]) -- this kernel runs up to `iters` (70 by default)
/// x 2 dispatches per `generate_terrain` call, so paying a per-call
/// adapter/device handshake independently would have been even more costly
/// here than for the four milestone-8 kernels; there is deliberately no
/// milestone-6-style standalone `init_gpu_weather`. (Milestone 6 put that
/// handshake at ~1.3-1.4 s and it no longer reproduces at that size --
/// [`GpuDevice`] carries the current figure and its range -- which changes
/// how large the saving is, not whether there is one.)
pub struct GpuWeatherContext {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    evap_pipeline: wgpu::ComputePipeline,
    advect_pipeline: wgpu::ComputePipeline,
    deposit_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

/// `_with` sibling pattern (milestone 8): builds on an already-created
/// [`GpuDevice`] instead of requesting a new adapter/device.
pub fn init_gpu_weather_with(gpu: &GpuDevice) -> GpuWeatherContext {
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpu_weather (f32, gather-shaped wind/rain loop)"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC_GPU_WEATHER.into()),
    });

    let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cartalith-gpu weather bind group layout"),
        entries: &WEATHER_LAYOUT,
    });

    let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("weather pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let make_pipeline = |label: &str, entry_point: &str| {
        gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    GpuWeatherContext {
        adapter_name: gpu.adapter_name.clone(),
        adapter_vendor: gpu.adapter_vendor,
        adapter_backend: gpu.adapter_backend,
        device_type: gpu.device_type,
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
        lost: std::sync::Arc::clone(&gpu.lost),
        evap_pipeline: make_pipeline("evap pipeline", "evap_main"),
        advect_pipeline: make_pipeline("advect pipeline", "advect_main"),
        deposit_pipeline: make_pipeline("deposit pipeline", "deposit_main"),
        bind_group_layout,
    }
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9: D8 flow accumulation --
/// three pipelines (`dir_main`/`scatter_main`/`merge_main`) from one shader
/// module, [`GpuWeatherContext`]'s shape reused. Built only via
/// [`init_gpu_flow_with`] (a shared [`GpuDevice`]), never a standalone
/// per-call adapter/device handshake: `generate_terrain` calls flow
/// accumulation up to four times per generation, so milestone 6's
/// per-call-context mistake would cost four handshakes here on top of
/// everything else. Hold one of these across all four call sites and the
/// shader is compiled once per generation too, not once per call.
pub struct GpuFlowContext {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    dir_pipeline: wgpu::ComputePipeline,
    scatter_pipeline: wgpu::ComputePipeline,
    merge_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

/// `_with` sibling pattern (milestone 8): builds on an already-created
/// [`GpuDevice`] instead of requesting a new adapter/device.
pub fn init_gpu_flow_with(gpu: &GpuDevice) -> GpuFlowContext {
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpu_flow (D8 direction + pointer-doubling accumulation)"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC_GPU_FLOW.into()),
    });

    let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cartalith-gpu flow bind group layout"),
        entries: &FLOW_LAYOUT,
    });

    let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("flow pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        ..Default::default()
    });

    let make_pipeline = |label: &str, entry_point: &str| {
        gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    GpuFlowContext {
        adapter_name: gpu.adapter_name.clone(),
        adapter_vendor: gpu.adapter_vendor,
        adapter_backend: gpu.adapter_backend,
        device_type: gpu.device_type,
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
        lost: std::sync::Arc::clone(&gpu.lost),
        dir_pipeline: make_pipeline("flow dir pipeline", "dir_main"),
        scatter_pipeline: make_pipeline("flow scatter pipeline", "scatter_main"),
        merge_pipeline: make_pipeline("flow merge pipeline", "merge_main"),
        bind_group_layout,
    }
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: `gauss_blur`'s GPU path.
/// Duplicates [`init_gpu_with`]'s adapter/device/queue setup rather than
/// refactoring it to support multiple entry points -- this is the only
/// kernel in this crate needing two pipelines from one shader module, and
/// a one-off dedicated function is a smaller, clearer diff than
/// generalizing the shared helper for a single caller.
pub fn init_gpu_gauss_blur() -> Result<GpuBlurContext, GpuInitError> {
    let instance = multi::compute_instance();

    // Was its own raw `request_adapter(HighPerformance)`, which is the one
    // adapter request in this crate that neither honoured the device
    // preference nor excluded a software rasterizer (2026-09-02 --
    // `multi::GpuDeviceInfo::is_software` has the wgpu-core reading).
    // `pick_primary_adapter` is that same request plus both rules, so this
    // is a deletion rather than a second copy of them.
    let adapter = multi::pick_primary_adapter(&instance).ok_or(GpuInitError::NoAdapter)?;

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
        lost: std::sync::Arc::default(),
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
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
    let instance = multi::compute_instance();

    // PR-01: the adapter is now *chosen* rather than always the first
    // `HighPerformance` match -- but only when a selection exists. With no
    // preference set (the default), `pick_primary_adapter` makes the exact
    // same `request_adapter(HighPerformance)` call this function always did.
    let adapter = multi::pick_primary_adapter(&instance).ok_or(GpuInitError::NoAdapter)?;
    request_gpu_device_from(adapter, required_features, min_storage_buffers, device_label)
}

/// [`request_gpu_device`]'s body, for an adapter the caller already chose --
/// which is what split-tiles needs: `init_gpu_device_set` resolves several
/// adapters by key and opens a device on each, and none of them is "the
/// `HighPerformance` one".
fn request_gpu_device_from(
    adapter: wgpu::Adapter,
    required_features: wgpu::Features,
    min_storage_buffers: u32,
    device_label: &str,
) -> Result<RawGpuDevice, GpuInitError> {
    if !adapter.features().contains(required_features) {
        return Err(GpuInitError::NoAdapter);
    }

    let info = adapter.get_info();
    let adapter_limits = adapter.limits();
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits = limits.using_resolution(adapter_limits.clone());
    limits.max_storage_buffers_per_shader_stage =
        limits.max_storage_buffers_per_shader_stage.max(min_storage_buffers);
    // `using_resolution` raises only the three `max_texture_dimension_*`
    // fields; the two *buffer* ceilings stay at `downlevel_defaults()`'s
    // 128 MiB binding / 256 MiB allocation no matter what the card can do.
    // One full-grid `f32` buffer is `w*h*4` bytes, so 128 MiB caps the GPU
    // path at 5792² -- and past it `create_bind_group` does not fall back,
    // it raises a wgpu validation error, which is a **panic** in the Godot
    // process (`cartalith-rust-conventions`: no panic crosses the gdext
    // boundary). Measured, not theorised: `use_gpu = true` at 8192² -- a
    // resolution `new_world_dialog.gd`'s own `RESOLUTION_PRESETS` offers, with
    // the GPU toggle at the shell's default of on -- died with
    // "Buffer binding 1 range 268435456 exceeds `max_*_buffer_binding_size`
    // limit 134217728" while `PERFORMANCE_BENCHMARKS.md` was being measured.
    //
    // Taking the adapter's own reported ceilings keeps
    // `HARDWARE_ACCELERATION.md` §10's rule intact -- this still never asks
    // for more than the hardware reports, it just stops asking for less than
    // the pipeline needs. On this machine both adapters report 2047 MB /
    // 2048 MB, which covers every size in `RESOLUTION_PRESETS`.
    limits.max_storage_buffer_binding_size =
        limits.max_storage_buffer_binding_size.max(adapter_limits.max_storage_buffer_binding_size);
    limits.max_buffer_size = limits.max_buffer_size.max(adapter_limits.max_buffer_size);

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
        lost: std::sync::Arc::default(),
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
        lost: raw.lost,
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
///
/// **The ~1.3-1.4 s above is milestone 6's number and no longer reproduces.**
/// Re-measured by `measured_device_handshake_and_per_stage_pipeline_build`:
/// **a handshake costs a couple of hundred milliseconds**, and the first one
/// of a process costs somewhat more than the ones after it -- by less than the
/// margin the old "416 ms then ~198 ms" implied. Figures below.
///
/// **Why the figures below all carry a range.** The first version of this
/// comment said "416 ms cold, then ~198 ms". An independent re-run on the same
/// hardware measured **730 ms cold** -- 1.75x the number, and outside the range
/// this comment had already been cited for in two other places. Both were
/// single samples, and this device is demonstrably noisy: a `512x512` GPU
/// timing quoted from a parallel `cargo test` run spanned 596 ms to 2.98 s,
/// while the same measurement run alone spanned 499-506 ms. The test now draws
/// `TIMING_ROUNDS` samples and prints median with min..max, and refuses to
/// print anything at all unless it was started with `--test-threads=1`.
///
/// Re-measured 2026-09-05 by
/// `cargo test --release -p cartalith-gpu measured_device_handshake --`
/// `--ignored --test-threads=1 --nocapture`, run alone, **on one machine**
/// (AMD Radeon RX 7800 XT, Vulkan, discrete -- these are figures about that
/// box and not about GPUs):
///
/// - **Cold handshake: 235 ms.** One sample by construction -- a process has
///   exactly one first handshake, so this is the one number here that cannot
///   be given a spread, and it is reported separately for that reason rather
///   than pooled with the warm ones.
/// - **Warm handshake: 190 ms median (189.7-202.2 ms, n=5).**
/// - **Pipeline builds: medians summing to ~3 ms for all six**, the dearest
///   stage `weather` and the cheapest `jfa_plates` about 4x apart.
///
///   The per-stage figures carried spreads in the test output and lost them
///   here — a verifier caught eight such bare estimates in this pass, in the
///   very prose written to retire single-sample timings (corrected
///   2026-09-05). Run `measured_pipeline_build_cost` for the numbers with
///   their brackets; the conclusion below needs only the order of magnitude,
///   and quoting three significant figures for a sub-millisecond timing on a
///   noisy device implies a precision the harness does not have.
///
/// Sharing the device is still worth it -- ~190 ms once per stage that would
/// otherwise open its own is a large fraction of a second per generation --
/// and it is far cheaper than milestone 6's 1.3-1.4 s. Two
/// `OUTSTANDING_WORK.md` §2.6 rows lean on the old number and should be read
/// against these:
///
/// - *"Per-pipeline caching across repeated `generate_terrain` calls."*
///   `generate_terrain` holds this device in a local and drops it at the end,
///   so **call two rebuilds one handshake plus every pipeline** -- each
///   `*_grid_gpu_with` entry point calls its own `init_gpu_*_with` inside the
///   dispatch rather than accepting a built context (`gpu_flow` is the lone
///   exception, hoisted by milestone 9 because one call uses it four times).
///   The two halves are nothing like equal: ~3 ms of pipeline against ~190 ms
///   of handshake, so caching *pipelines* -- the thing the row asks for -- is
///   the smaller half by roughly two orders of magnitude. Caching the *device* is where that row's value
///   actually is, and that conclusion is the one thing here that does not turn
///   on the exact figures: the ranges do not come close to overlapping.
/// - *"Hardware capability cache (§30)"*, re-opened because the handshake was
///   thought to be 1.3-1.4 s. At ~190 ms the original deferral ("nothing
///   expensive enough to cache") is much closer to right than the row
///   supposes. **Re-measure before building anything**, which is what that
///   row asked for and what this note records.
pub struct GpuDevice {
    pub adapter_name: String,
    pub adapter_vendor: u32,
    pub adapter_backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// Set once this device has failed a readback: a `wgpu` device that
    /// loses a `map_async` is *gone*, not merely out of room at that size --
    /// measured, the very next `create_buffer_init` on it panics with
    /// "Buffer ... is invalid". Shared (`Arc`) with every context built from
    /// the same device, so one stage's failure closes the GPU path for all of
    /// them. See [`read_back`].
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// Sized for the largest bind group among the kernels `generate_terrain`'s
/// `use_gpu` path reuses this device across -- JFA plate assignment's own
/// [`JFA_LAYOUT`] (8 storage buffers), the highest of the four reused
/// kernels (warp needs 2, heterogeneity 4, blur 2). `wgpu` limits can't be
/// raised after device creation, so this has to be decided up front rather
/// than derived per-pipeline the way [`init_gpu_with`] derives it for a
/// single-use device.
///
/// **This 8 is also what keeps `gpu_height` off the shared device**:
/// [`HEIGHT_LAYOUT`] needs 9, which is why it is the one milestone-1-to-5
/// kernel with no `init_gpu_height_with` sibling. Raising this to 9 is not a
/// free widening -- it raises the request for *every* stage's device, and an
/// adapter that cannot meet it fails at `create_bind_group` with a `wgpu`
/// validation error, which panics, which takes Godot down. See
/// [`dispatch_gpu_height`]'s own section for the decision that leaves it
/// unwired and the measurements behind it.
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
        lost: raw.lost,
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
        lost: std::sync::Arc::clone(&gpu.lost),
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
        lost: std::sync::Arc::clone(&gpu.lost),
        box_h_pipeline,
        box_v_pipeline,
        bind_group_layout,
    }
}

// -- Readback ------------------------------------------------------------------
//
// Every dispatch below ends the same way: copy a storage buffer into a
// `MAP_READ` staging buffer, map it, copy the bytes out, unmap. That tail used
// to be written out ten times with `.expect("buffer map failed")` at the end of
// each, which made a device that cannot complete a readback a *panic* -- and a
// panic inside a loaded GDExtension takes the Godot process with it
// (`cartalith-rust-conventions`). Measured, not theorised: this machine's
// integrated Radeon at 8192² passes every limits check, dispatches, and then
// returns `BufferAsyncError` from the map.
//
// So the tail lives in one place and returns `Option`. A failure does two
// things, and BOTH were needed -- the second was found by re-running the real
// 8192² integrated-GPU generation after only the first was in place:
//
// 1. It records the size against the adapter (`multi::note_readback_failure`),
//    so a *later* generation at that size skips this device up front rather
//    than re-discovering the same wall.
// 2. It marks the live device **lost**. A `wgpu` device that loses a
//    `map_async` is gone, not merely out of room: with only (1) in place, the
//    base-field blur failed gracefully at 8192² and the next stage -- weather,
//    on a 240² coarse grid, far under any size ban -- panicked immediately on
//    a 32-byte uniform buffer with `Buffer with 'weather params' label is
//    invalid`. The flag is shared by `Arc` with every context built from the
//    same device, at every size, for the life of that device. A *new* device
//    (the next `generate_terrain` opens its own) starts clean, so this is not
//    a permanent verdict on the hardware.
//
// `HARDWARE_ACCELERATION.md` §27: a GPU failure transitions to CPU, it does
// not crash.

/// What a dispatch needs from whichever context type it was handed: the live
/// device to poll, the adapter identity to blame if the readback fails, and
/// the shared lost flag. Implemented for all four context types and for
/// [`GpuDevice`] itself -- they all already carry these fields, this only
/// names them once.
trait DispatchDevice {
    fn wgpu_device(&self) -> &wgpu::Device;
    fn identity(&self) -> (&str, u32, wgpu::Backend);
    fn lost(&self) -> &std::sync::atomic::AtomicBool;
}

macro_rules! impl_dispatch_device {
    ($($t:ty),+ $(,)?) => {$(
        impl DispatchDevice for $t {
            fn wgpu_device(&self) -> &wgpu::Device {
                &self.device
            }
            fn identity(&self) -> (&str, u32, wgpu::Backend) {
                (&self.adapter_name, self.adapter_vendor, self.adapter_backend)
            }
            fn lost(&self) -> &std::sync::atomic::AtomicBool {
                &self.lost
            }
        }
    )+};
}
impl_dispatch_device!(GpuContext, GpuBlurContext, GpuWeatherContext, GpuFlowContext, GpuDevice);

/// Whether this device has already failed a readback (in this session, at this
/// size or smaller) or been lost outright. The one gate every entry point
/// checks before touching the device at all -- including before building a
/// pipeline on it, since a lost device fails those too.
fn device_is_unusable(ctx: &impl DispatchDevice, cells: u64) -> bool {
    if ctx.lost().load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
    let (name, vendor, backend) = ctx.identity();
    multi::readback_failure_cells(name, vendor, backend).is_some_and(|at| cells >= at)
}

/// Map `staging`, hand its bytes to `f`, unmap, and return `f`'s result --
/// or `None` if the device could not complete the readback, having first
/// recorded the size and marked the device lost.
///
/// `cells` is the grid the *dispatch* was for, not the length of this
/// particular buffer: a stage reading two half-width buffers still fails at
/// the size it was asked to compute, and that is the size a later stage needs
/// to avoid.
fn read_back<R>(
    ctx: &impl DispatchDevice,
    staging: &wgpu::Buffer,
    cells: u64,
    f: impl FnOnce(&[u8]) -> R,
) -> Option<R> {
    let (name, vendor, backend) = ctx.identity();
    let fail = |what: &str| {
        eprintln!(
            "cartalith-gpu: {what} on {name} at {cells} cells -- this device is done for this run; \
             falling back to CPU (HARDWARE_ACCELERATION.md §27)"
        );
        multi::note_readback_failure(name, vendor, backend, cells);
        ctx.lost().store(true, std::sync::atomic::Ordering::Relaxed);
        None::<()>
    };

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    if let Err(e) = ctx.wgpu_device().poll(wgpu::PollType::wait_indefinitely()) {
        fail(&format!("device poll failed ({e:?})"))?;
    }
    match rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => fail(&format!("buffer map failed ({e:?})"))?,
        Err(e) => fail(&format!("map_async channel closed ({e})"))?,
    }
    let data = match slice.get_mapped_range() {
        Ok(d) => d,
        Err(e) => {
            fail(&format!("get_mapped_range failed ({e:?})"))?;
            return None;
        }
    };
    let out = f(&data);
    drop(data);
    staging.unmap();
    Some(out)
}

/// [`read_back`] for the common case: a whole staging buffer of `T`, copied
/// out as a `Vec<T>`.
fn read_back_vec<T: Pod>(ctx: &impl DispatchDevice, staging: &wgpu::Buffer, cells: u64) -> Option<Vec<T>> {
    read_back(ctx, staging, cells, |bytes| bytemuck::cast_slice::<u8, T>(bytes).to_vec())
}

/// Run one whole-grid dispatch on `gpu`, refusing it up front -- *before the
/// pipeline is even built* -- if the device is lost or already known not to
/// reach this size ([`device_supports_grid`], which now answers on measured
/// readback failures as well as on reported limits).
///
/// The `dispatch` closure builds its own context on purpose: shader-module and
/// pipeline creation are device calls too, and on a lost device they are as
/// unsafe as the dispatch itself.
///
/// The pre-check matters inside a single `generate_terrain` as much as across
/// calls: the engine resolves its device set once, at the top, so without this
/// every one of the six GPU stages would separately build, dispatch, wait and
/// fail on a device the first stage already proved cannot do it.
fn on_grid<T>(gpu: &GpuDevice, width: u32, height: u32, dispatch: impl FnOnce() -> Option<T>) -> Option<T> {
    if device_is_unusable(gpu, u64::from(width) * u64::from(height))
        || !device_supports_grid(gpu, width as usize, height as usize)
    {
        return None;
    }
    dispatch()
}

/// Dispatch the GPU kernel over a `width`x`height` grid, sampling
/// `vnoise(x*scale, y*scale, seed)` at each cell -- read back and return
/// as a plain `Vec<f32>`, row-major (matches `cartalith-noise`'s own
/// convention elsewhere in this workspace).
///
/// `None` if the readback failed; see [`read_back`].
fn dispatch_gpu(ctx: &GpuContext, width: u32, height: u32, seed: i32, scale: f32) -> Option<Vec<f32>> {
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

    read_back_vec::<f32>(ctx, &staging_buf, count as u64)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: dispatch `gpu_warp.wgsl`,
/// returning `(warp_x, warp_y)` -- one dispatch computes both, matching
/// `compute_warp`'s own shape (see [`init_gpu_warp`]'s doc comment).
fn dispatch_gpu_warp(ctx: &GpuContext, width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> Option<(Vec<f32>, Vec<f32>)> {
    dispatch_gpu_warp_band(ctx, width, height, 0, height, seed, wf, amp)
}

/// The row-band form of [`dispatch_gpu_warp`]: computes rows
/// `y_offset .. y_offset + band_rows` of a `width` x `height` grid and
/// returns just that band, `width * band_rows` values per output.
///
/// `warp`'s kernel is the one GPU stage in this pipeline with **no**
/// cross-cell dependency at all -- every output is a pure function of its
/// own `(x, y, seed)`, with no input buffers and no neighbour reads -- so a
/// row band computed alone is bit-identical, on the same device, to the
/// same rows computed as part of the whole grid. That is what makes
/// [`warp_grid_gpu_split`] a real partition rather than an approximation.
/// (Across *different* devices the usual GPU-vs-GPU caveat applies; see
/// that function's own doc comment.)
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_warp_band(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    y_offset: u32,
    band_rows: u32,
    seed: i32,
    wf: f32,
    amp: f32,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let count = (width * band_rows) as usize;
    let mut bx = vec![0f32; count];
    let mut by = vec![0f32; count];
    dispatch_gpu_warp_band_into(ctx, width, height, y_offset, band_rows, seed, wf, amp, &mut bx, &mut by)?;
    Some((bx, by))
}

/// [`dispatch_gpu_warp_band`] writing straight into caller-owned slices.
///
/// This exists for a measured reason, not for style: assembling
/// [`warp_grid_gpu_split`]'s result by concatenating per-band `Vec`s made
/// the split path pay two extra whole-grid memcpys (~134 MB at 4096x4096)
/// that the single-device path does not, which is overhead charged to
/// splitting rather than a property of it. Writing each band's readback
/// directly into its slice of the final buffer removes that asymmetry, so
/// the timing comparison measures the dispatch and not the bookkeeping.
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_warp_band_into(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    y_offset: u32,
    band_rows: u32,
    seed: i32,
    wf: f32,
    amp: f32,
    out_warp_x: &mut [f32],
    out_warp_y: &mut [f32],
) -> Option<()> {
    let count = (width * band_rows) as usize;
    assert_eq!(out_warp_x.len(), count);
    assert_eq!(out_warp_y.len(), count);
    let byte_len = (count * std::mem::size_of::<f32>()) as u64;

    let params = WarpParams { seed, width, height, wf, amp, y_offset, band_rows, _pad2: 0.0 };
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
        pass.dispatch_workgroups(width.div_ceil(8), band_rows.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out_x, 0, &staging_x, 0, byte_len);
    encoder.copy_buffer_to_buffer(&out_y, 0, &staging_y, 0, byte_len);
    ctx.queue.submit(Some(encoder.finish()));

    // A band's failure is a failure at the size of the grid it is a band OF --
    // that is the size a later stage has to steer around, not the band's own
    // row count.
    let cells = u64::from(width) * u64::from(height);
    let into = |staging: &wgpu::Buffer, dst: &mut [f32]| {
        read_back(ctx, staging, cells, |bytes| dst.copy_from_slice(bytemuck::cast_slice(bytes)))
    };
    into(&staging_x, out_warp_x)?;
    into(&staging_y, out_warp_y)?;
    Some(())
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
) -> Option<Vec<f32>> {
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

    read_back_vec::<f32>(ctx, &staging_buf, count as u64)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3: dispatch `gpu_height.wgsl`.
/// `warp_x`/`warp_y` should be zero-filled by the caller when there's no
/// real warp field (matches `compute_height`'s `Option<&[f32]>` the same
/// way [`dispatch_gpu_heterogeneity`] already does). `oro` is different --
/// its ABSENCE changes which formula runs, not just an additive no-op --
/// so `has_oro` is a real parameter, and the `oro` buffer content is
/// ignored by the shader when `has_oro` is `false` (any same-length slice
/// works, including a zero-filled dummy).
///
/// # Why nothing calls this, and why that is correct
///
/// `OUTSTANDING_WORK.md` §2.6 carried this as *"built, verified, and **never
/// called** ... either a real gap or an undocumented decision -- the docs do
/// not say which"*. **Settled 2026-09-03: an undocumented decision, and the
/// right one.** There is no `if p.use_gpu` branch around
/// `cartalith_terrain::compute_height` and there should not be one. Three
/// findings, in the order that decides it:
///
/// 1. **The shared device structurally cannot host this pipeline.**
///    [`HEIGHT_LAYOUT`] binds **9 storage buffers**;
///    [`REUSED_STAGE_MAX_STORAGE_BUFFERS`] -- the limit
///    [`init_gpu_shared_device`] opens every reused stage's device at -- is
///    **8**, sized for JFA, the widest kernel `generate_terrain` actually
///    shares. That is why this is the one milestone-1-to-5 kernel with no
///    `init_gpu_height_with` sibling: it never had a device it could be built
///    on. Wiring it means either raising that limit for *every* stage (a
///    device-request change on a path whose failure mode is a `wgpu`
///    validation error, i.e. a panic, i.e. the Godot process --
///    `cartalith-rust-conventions`, and the measured incident recorded at
///    `generate_terrain`'s own `gpu_allowed_for_grid` check), or giving
///    height its own device and paying a second handshake of **roughly
///    200 ms** ([`GpuDevice`]'s re-measurement, which carries the range).
/// 2. **The speedup on record is against the wrong baseline.** 5.17× / 8.13× /
///    4.84× (milestone 3) compare this kernel to [`gpu_height_grid_cpu`], a
///    **single-threaded `f32` twin written to match the shader**.
///    `generate_terrain` calls `cartalith_terrain::compute_height`, which is
///    `f64` and already `par_chunks_mut` across every core. Against *that*
///    baseline -- `measured_gpu_height_vs_the_real_compute_height`, run alone
///    2026-09-05, medians of five with the range each bracket allows -- the
///    GPU wins **1.95× (1.52..2.01×) at 1024²**.
///
///    **The 2048² figure was refuted and is withdrawn (2026-09-05).** This
///    said "1.06× (1.00..1.08×) at 2048², saving 2.2 ms". A verifier re-ran
///    the same test serially three times, medians of five each, and measured
///    **1.00× (0.98..1.04), 1.00× (0.97..1.03), 1.02× (0.96..1.03)** — the
///    claimed median sits outside all three brackets, and the "saving" spanned
///    +0.10 to -0.17 ms, i.e. it changed sign. **At 2048² no difference is
///    established between this kernel and the function that ships.** Quote no
///    number here; there is not one.
///
///    That this was written by the very pass sent to replace single-sample
///    timings with medians is the point of the rule, not an aside: a median of
///    five on a noisy device is still one sample of the median. A few
///    milliseconds would not buy a handshake two orders of magnitude larger in
///    any case, and it is well inside the run-to-run spread of one generation.
/// 3. **It is the widest bind group in the crate, and that is what limits
///    it.** See `measured_gpu_height_is_bandwidth_bound_at_nine_buffers`: the
///    eight input uploads are **63% (2048²) to 82% (1024²)** of the dispatch
///    median, and this kernel's per-cell cost *rises* **2.19×** from 1024² to
///    2048² (3.92 -> 8.58 ns/cell) while every narrower kernel's falls. Its
///    advantage shrinks exactly where a bigger grid would make it matter.
///
/// So this stays a verified, tested, uncalled kernel -- the same standing as
/// `dispatch_gpu_resistance`, which `GPU_LAYER_INTEGRATION_SCOPE.md` already
/// documents as deliberately unwired at 0.38× (re-measured alone 2026-09-05 by
/// `measured_gpu_blur_and_resistance_timing` at **0.24× at 1024² and 0.13× at
/// 2048²**, so the scope document's figure is optimistic and its conclusion is
/// not). **What would overturn it**: a generation that runs the height stage
/// many times (live sculpt, a parameter sweep) so one handshake amortises, or
/// a kernel reshaped to bind fewer than nine storage buffers. Re-run the two
/// `measured_*` tests named above before acting on either.
///
/// Every figure in this comment is from one machine (AMD Radeon RX 7800 XT,
/// Vulkan, discrete). Re-run before carrying any of them onto another.
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
) -> Option<Vec<f32>> {
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

    read_back_vec::<f32>(ctx, &staging_buf, count as u64)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 4: dispatch `gpu_resistance.wgsl`.
/// `crustal_per_plate` is precomputed by the caller as `plates[k].base.
/// max(0.0)` for each plate -- a tiny (`num_plates`-length) CPU-side step,
/// not the per-cell workload this kernel accelerates (see the shader's
/// own header comment).
fn dispatch_gpu_resistance(
    ctx: &GpuContext,
    width: u32,
    height: u32,
    plate_id: &[u32],
    age: &[f32],
    crustal_per_plate: &[f32],
) -> Option<Vec<f32>> {
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

    read_back_vec::<f32>(ctx, &staging_buf, count as u64)
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
fn dispatch_gpu_gauss_blur(
    ctx: &GpuBlurContext,
    src: &[f32],
    radius: f64,
    width: u32,
    height: u32,
    wrap_x: bool,
) -> Option<Vec<f32>> {
    if radius < 1.0 {
        return Some(src.to_vec());
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

    read_back_vec::<f32>(ctx, &staging_buf, count as u64)
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7: `simulate_weather`'s inner
/// loop. `eh`/`tc`/`sst_evap`/`wx`/`wy` are frozen for the whole call
/// (uploaded once); `w` starts pre-initialized by the caller (CPU's own
/// `w[i] = if eh[i]<sea { ocean_hum } else { 0.10 }`, cheap enough on this
/// coarse `ww`x`wh` grid that mirroring it as a fourth WGSL entry point
/// would be pure overhead). Every one of `iters` x 2 dispatches (evap+
/// boundary fused into one, advect, deposit) is encoded into ONE command
/// encoder and submitted ONCE -- matching `gpu_jfa_plates`'s own multi-pass
/// convention -- so the GPU driver can pipeline the whole sequence without
/// a CPU-side sync point between iterations; the only readback is the
/// final `w`/`rain` state after the loop completes.
#[allow(clippy::too_many_arguments)]
fn dispatch_gpu_weather(
    ctx: &GpuWeatherContext,
    eh: &[f32],
    tc: &[f32],
    sst_evap: &[f32],
    wx: &[f32],
    wy: &[f32],
    w_init: &[f32],
    ww: u32,
    wh: u32,
    iters: i32,
    sea: f32,
    ocean_hum: f32,
    evap_c: f32,
    ocean_c: f32,
    rain_k: f32,
    dry: f32,
    step: f32,
    bulk_evap: bool,
    wrap_x: bool,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let n = (ww * wh) as usize;
    assert_eq!(eh.len(), n);
    assert_eq!(w_init.len(), n);
    let byte_len = (n * std::mem::size_of::<f32>()) as u64;

    let params = WeatherParams {
        ww,
        wh,
        wrap_x: wrap_x as u32,
        bulk_evap: bulk_evap as u32,
        sea,
        ocean_hum,
        evap_c,
        ocean_c,
        rain_k,
        dry,
        step,
        _pad: 0.0,
    };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("weather params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let make_ro = |label: &str, contents: &[f32]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(contents),
            usage: wgpu::BufferUsages::STORAGE,
        })
    };
    let eh_buf = make_ro("weather eh", eh);
    let tc_buf = make_ro("weather tc", tc);
    let sst_evap_buf = make_ro("weather sst_evap", sst_evap);
    let wx_buf = make_ro("weather wx", wx);
    let wy_buf = make_ro("weather wy", wy);

    let w_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("weather w"),
        contents: bytemuck::cast_slice(w_init),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let w2_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("weather w2"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let rain_init = vec![0f32; n];
    let rain_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("weather rain"),
        contents: bytemuck::cast_slice(&rain_init),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let staging_w = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("weather w (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let staging_rain = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("weather rain (staging)"),
        size: byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("weather bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: eh_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: tc_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: sst_evap_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: wx_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: wy_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 6, resource: w_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 7, resource: w2_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 8, resource: rain_buf.as_entire_binding() },
        ],
    });

    let wg_x = ww.div_ceil(8);
    let wg_y = wh.div_ceil(8);
    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("weather encoder") });
    for _ in 0..iters {
        {
            let mut pass = encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("evap pass"), timestamp_writes: None });
            pass.set_pipeline(&ctx.evap_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("advect pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.advect_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("deposit pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.deposit_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
    }
    encoder.copy_buffer_to_buffer(&w_buf, 0, &staging_w, 0, byte_len);
    encoder.copy_buffer_to_buffer(&rain_buf, 0, &staging_rain, 0, byte_len);
    ctx.queue.submit(Some(encoder.finish()));

    let w_out = read_back_vec::<f32>(ctx, &staging_w, n as u64)?;
    let rain_out = read_back_vec::<f32>(ctx, &staging_rain, n as u64)?;
    Some((w_out, rain_out))
}

/// Public `_with` wrapper (milestone 8's shared-device convention, applied
/// here from the start per this milestone's own directive -- this kernel's
/// `iters` (default 70) x 2 dispatches make context reuse matter even more
/// than it did for milestone 8's original four kernels). Returns `(w,
/// rain)`, both length `ww*wh`, matching `simulate_weather`'s own internal
/// coarse-grid state -- callers are responsible for the final blur-then-
/// upsample-to-`gw`x`gh` step (cheap, CPU-only, not part of this kernel;
/// see `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7 for why).
#[allow(clippy::too_many_arguments)]
pub fn simulate_weather_loop_gpu_with(
    gpu: &GpuDevice,
    eh: &[f32],
    tc: &[f32],
    sst_evap: &[f32],
    wx: &[f32],
    wy: &[f32],
    w_init: &[f32],
    ww: u32,
    wh: u32,
    iters: i32,
    sea: f32,
    ocean_hum: f32,
    evap_c: f32,
    ocean_c: f32,
    rain_k: f32,
    dry: f32,
    step: f32,
    bulk_evap: bool,
    wrap_x: bool,
) -> Option<(Vec<f32>, Vec<f32>)> {
    // `ww`x`wh` is the coarse weather grid, not the world grid -- the size
    // checked and, on failure, recorded is the one this dispatch actually
    // asked the device for. (The *lost* check inside `on_grid` is what stops
    // this stage after an earlier whole-grid stage killed the device; the size
    // ban alone would wave a 240² grid straight through.)
    on_grid(gpu, ww, wh, || {
        let ctx = init_gpu_weather_with(gpu);
        dispatch_gpu_weather(
            &ctx, eh, tc, sst_evap, wx, wy, w_init, ww, wh, iters, sea, ocean_hum, evap_c, ocean_c, rain_k, dry,
            step, bulk_evap, wrap_x,
        )
    })
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
) -> Option<Vec<i32>> {
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

    let mut nearest = read_back_vec::<i32>(ctx, &staging_nearest, n as u64)?;

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
    Some(nearest)
}

/// The per-cell seed `compute_flow` starts its accumulation from, computed
/// exactly as `cartalith_hydrology::compute_flow`'s own opening block does
/// (`use_rain=false` => 1.0 everywhere; `use_rain=true` => `max(rain,0.05)`
/// rescaled so the seeds sum to `n`). Deliberately duplicated here rather
/// than imported: `cartalith-gpu` depends on `cartalith-hydrology` only as
/// a dev-dependency (milestone 4's convention -- the library never pulls a
/// subsystem crate in, only the test suite does), and this is six lines of
/// arithmetic whose *whole point* is being bit-identical to the CPU
/// function's, so any drift shows up immediately as a divergence in
/// `gpu_flow_matches_real_cpu_compute_flow`.
fn flow_seed(n: usize, rain: Option<&[f32]>, use_rain: bool) -> Vec<f32> {
    let mut acc = vec![0f32; n];
    if use_rain {
        let rain = rain.expect("rain field required when use_rain is true");
        let mut sm = 0.0f64;
        for (v, &r) in acc.iter_mut().zip(rain.iter()) {
            let r = (r as f64).max(0.05);
            *v = r as f32;
            sm += r;
        }
        let k = n as f64 / sm.max(1e-6);
        for v in acc.iter_mut() {
            *v = (*v as f64 * k) as f32;
        }
    } else {
        acc.fill(1.0);
    }
    acc
}

/// Headroom the fixed-point accumulator keeps below `u32::MAX`. The largest
/// value any cell can ever hold is the full seed total (an outlet draining
/// the whole grid), so `total * scale` must fit; capping at 2^31 leaves a
/// full factor-of-two margin for the per-seed rounding that can push the
/// quantized total slightly above the real one.
const FLOW_FIXED_POINT_CEILING: f64 = 2_147_483_648.0;
/// Upper bound on the fixed-point scale, so a tiny grid doesn't pick an
/// absurd shift. 2^24 is where an `f32` stops representing consecutive
/// integers anyway -- finer quantization than that buys nothing once the
/// result is converted back to `f32`.
const FLOW_MAX_FIXED_POINT_SHIFT: u32 = 24;

/// Largest power-of-two fixed-point scale whose worst-case accumulation
/// still fits the `u32` atomics, given the real seed total.
fn flow_fixed_point_scale(total: f64) -> f64 {
    let total = total.max(1.0);
    let mut shift = 0u32;
    while shift < FLOW_MAX_FIXED_POINT_SHIFT && total * ((1u64 << (shift + 1)) as f64) <= FLOW_FIXED_POINT_CEILING {
        shift += 1;
    }
    (1u64 << shift) as f64
}

/// Result of one flow dispatch: the accumulation itself plus the D8
/// receiver field it was computed over (`-1` = pit/outlet). The receivers
/// are the interesting half for verification -- they are the *only* place
/// this kernel can diverge from the CPU function for a reason other than
/// summation order, so the tests read them back directly rather than
/// inferring divergence from the accumulation alone.
pub struct GpuFlowResult {
    pub acc: Vec<f32>,
    pub recv: Vec<i32>,
    /// The fixed-point scale actually chosen for this grid (see
    /// [`flow_fixed_point_scale`]); reported so a caller/test can state the
    /// real quantization step rather than guess at it.
    pub fixed_point_scale: f64,
}

/// D8 flow accumulation on GPU. See `gpu_flow.wgsl`'s header for the
/// algorithm and its literature; this function owns the two decisions that
/// live on the Rust side:
///
/// **Round count.** Pointer doubling reaches every ancestor once `2^k`
/// exceeds the longest flow path. The longest possible path in an `n`-cell
/// single-receiver forest is `n` cells (every edge strictly descends, so no
/// cycles), hence `ceil(log2(n))` rounds is a hard upper bound -- 22 at
/// 2048x2048. No convergence read-back: checking whether every pointer has
/// gone `-1` would cost a map/unmap round trip per round, which is more
/// expensive than simply running the bound (each round is two cheap
/// dispatches), and it would make the dispatch count data-dependent, i.e.
/// the timing unreproducible.
///
/// **Fixed-point accumulation.** WGSL has no atomic float add. Emulating
/// one with a compare-exchange loop would make the answer depend on which
/// thread wins each race, i.e. non-deterministic run to run -- something
/// every GPU milestone in this project has had to rule out. Integer
/// addition is exactly associative and commutative, so `u32` fixed point
/// makes the scatter order-independent *and* bit-reproducible. The cost is
/// quantization: each seed is rounded to the nearest `1/scale`, where
/// `scale` is the largest power of two whose worst-case total still fits
/// (see [`flow_fixed_point_scale`]). At 2048x2048 that is 1/1024 per seed
/// against accumulations reaching ~4.2e6, i.e. a worst-case relative error
/// of ~5e-4 and a typical (random-sign, sqrt-cancelling) one far below
/// that -- comparable to, and at large accumulations better than, the
/// error the CPU's own long chain of `f32` additions already carries.
/// `None` -- CPU fallback, never a panic -- when the readback fails, or when
/// this device already failed one at this size ([`device_supports_grid`]).
/// This is the one dispatch the engine calls through its own context rather
/// than through a `_with` wrapper, so it carries the check itself.
pub fn dispatch_gpu_flow(
    ctx: &GpuFlowContext,
    gw: usize,
    gh: usize,
    field: &[f32],
    rain: Option<&[f32]>,
    use_rain: bool,
    world: bool,
) -> Option<GpuFlowResult> {
    let n = gw * gh;
    assert_eq!(field.len(), n, "field length must be gw*gh");
    if device_is_unusable(ctx, n as u64) {
        return None;
    }
    let width = gw as u32;
    let height = gh as u32;

    let seed = flow_seed(n, rain, use_rain);
    let total: f64 = seed.iter().map(|&v| v as f64).sum();
    let scale = flow_fixed_point_scale(total);
    let acc0: Vec<u32> = seed
        .iter()
        .map(|&v| ((v as f64) * scale).round().clamp(0.0, u32::MAX as f64) as u32)
        .collect();

    let byte_len_u32 = (n * std::mem::size_of::<u32>()) as u64;

    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
    let make_init = |label: &str, contents: &[u8]| {
        ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents, usage: storage })
    };
    let make_empty = |label: &str| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len_u32,
            usage: storage,
            mapped_at_creation: false,
        })
    };

    let field_buf = make_init("flow field", bytemuck::cast_slice(field));
    // `recv`/`ptr` are both written by `dir_main` before anything reads
    // them; `delta` must start at zero, which `wgpu` guarantees for a
    // freshly-created buffer.
    let recv_buf = make_empty("flow recv");
    let ptr_buf = make_empty("flow ptr");
    let ptr_next_buf = make_empty("flow ptr_next");
    let acc_buf = make_init("flow acc", bytemuck::cast_slice(&acc0));
    let delta_buf = make_init("flow delta", bytemuck::cast_slice(&vec![0u32; n]));

    let params = FlowGpuParams { width, height, world: u32::from(world), _pad0: 0 };
    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("flow params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("flow bind group"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: field_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: recv_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: ptr_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: ptr_next_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: acc_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 6, resource: delta_buf.as_entire_binding() },
        ],
    });

    let staging_acc = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flow acc (staging)"),
        size: byte_len_u32,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let staging_recv = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flow recv (staging)"),
        size: byte_len_u32,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let rounds = (n as f64).log2().ceil().max(1.0) as u32;
    let (gx, gy) = (width.div_ceil(8), height.div_ceil(8));

    let mut encoder =
        ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("flow encoder") });
    {
        let mut pass = encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("flow dir"), timestamp_writes: None });
        pass.set_pipeline(&ctx.dir_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(gx, gy, 1);
    }
    // One bind group for every round -- unlike JFA (a distinct `step`
    // uniform per pass) nothing per-round is uniform data here, and unlike
    // blur nothing ping-pongs between buffers: `merge_main` copies
    // `ptr_next` back into `ptr` itself. So all `2 * rounds` passes go into
    // one encoder and one submit.
    for _ in 0..rounds {
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("flow scatter"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.scatter_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        {
            let mut pass = encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("flow merge"), timestamp_writes: None });
            pass.set_pipeline(&ctx.merge_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
    }
    encoder.copy_buffer_to_buffer(&acc_buf, 0, &staging_acc, 0, byte_len_u32);
    encoder.copy_buffer_to_buffer(&recv_buf, 0, &staging_recv, 0, byte_len_u32);
    ctx.queue.submit(Some(encoder.finish()));

    // One buffer at a time rather than both maps in flight against a single
    // poll: the submit has already happened, so the second map's wait is
    // essentially free, and `read_back` is where the failure bookkeeping lives.
    let acc = read_back(ctx, &staging_acc, n as u64, |bytes| {
        bytemuck::cast_slice::<u8, u32>(bytes).iter().map(|&q| (q as f64 / scale) as f32).collect::<Vec<f32>>()
    })?;
    let recv = read_back_vec::<i32>(ctx, &staging_recv, n as u64)?;

    Some(GpuFlowResult { acc, recv, fixed_point_scale: scale })
}

/// Convenience wrapper: build a flow pipeline on `gpu` and run one
/// accumulation. Callers issuing several accumulations per generation
/// (`generate_terrain` issues up to four) should hold one
/// [`init_gpu_flow_with`] context and call [`dispatch_gpu_flow`] instead,
/// so the shader is compiled once rather than once per call.
pub fn flow_accumulation_gpu_with(
    gpu: &GpuDevice,
    gw: usize,
    gh: usize,
    field: &[f32],
    rain: Option<&[f32]>,
    use_rain: bool,
    world: bool,
) -> Option<Vec<f32>> {
    on_grid(gpu, gw as u32, gh as u32, || {
        let ctx = init_gpu_flow_with(gpu);
        dispatch_gpu_flow(&ctx, gw, gh, field, rain, use_rain, world)
    })
    .map(|r| r.acc)
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
    dispatch_gpu_warp(&ctx, width, height, seed, wf, amp)
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
    dispatch_gpu_heterogeneity(&ctx, width, height, hetero_seed, scale, age, warp_x, warp_y)
}

/// `gauss_blur`'s GPU sibling (milestone 4) -- used for both `base_field`
/// and, via the caller wrapping `compute_flexure`'s own thin-blur logic,
/// `flexure_field`.
pub fn gauss_blur_grid_gpu(src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Option<Vec<f32>> {
    let ctx = init_gpu_gauss_blur().ok()?;
    dispatch_gpu_gauss_blur(&ctx, src, radius, width, height, wrap_x)
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
    dispatch_gpu_assign_plates(&ctx, width, height, plate_x, plate_y, warp_x, warp_y)
}

/// `_with` sibling of [`warp_grid_gpu`] -- builds its pipeline on an
/// already-created [`GpuDevice`] (milestone 8, context reuse across
/// `generate_terrain`'s several GPU stages) instead of requesting its own
/// adapter/device.
///
/// `None` when the device cannot reach this grid -- either because its
/// reported limits say so, or because a readback on it has already failed at
/// this size ([`device_supports_grid`]), or because this dispatch's own
/// readback failed. Every caller falls back to the CPU function
/// (`HARDWARE_ACCELERATION.md` §27); none of them may panic.
pub fn warp_grid_gpu_with(
    gpu: &GpuDevice,
    width: u32,
    height: u32,
    seed: i32,
    wf: f32,
    amp: f32,
) -> Option<(Vec<f32>, Vec<f32>)> {
    on_grid(gpu, width, height, || {
        let ctx = init_gpu_warp_with(gpu);
        dispatch_gpu_warp(&ctx, width, height, seed, wf, amp)
    })
}

/// One row band of [`warp_grid_gpu_with`], on one device: rows
/// `y_offset .. y_offset + rows` of a `width` x `height` grid, returned as
/// `width * rows` values per output.
///
/// The building block [`warp_grid_gpu_split`] dispatches concurrently, and
/// exposed on its own so the band arithmetic can be verified against the
/// whole-grid result on a *single* device, where the two are required to be
/// bit-identical.
#[allow(clippy::too_many_arguments)]
pub fn warp_band_gpu_with(
    gpu: &GpuDevice,
    width: u32,
    height: u32,
    y_offset: u32,
    rows: u32,
    seed: i32,
    wf: f32,
    amp: f32,
) -> Option<(Vec<f32>, Vec<f32>)> {
    on_grid(gpu, width, height, || {
        let ctx = init_gpu_warp_with(gpu);
        dispatch_gpu_warp_band(&ctx, width, height, y_offset, rows, seed, wf, amp)
    })
}

/// One device's share of a [`warp_grid_gpu_split`] call: which device, which
/// rows, and the disjoint slices of the final buffers it writes into.
struct WarpBandJob<'a> {
    gpu: &'a GpuDevice,
    y_offset: u32,
    rows: u32,
    out_x: &'a mut [f32],
    out_y: &'a mut [f32],
}

/// The `split tiles` multi-GPU mode, for the one stage where a partition is
/// exact (`DCC_SHELL_SPEC.md` §2.5, `GUI_GAP_REGISTER.md` PR-02).
///
/// **Why warp and not another stage.** Splitting a grid across devices is
/// only sound where a cell's output depends on nothing outside its own
/// band. Auditing this pipeline's GPU stages against that test:
///
/// | Stage | Splittable |
/// |---|---|
/// | `gpu_warp` | **yes** — a pure function of `(x, y, seed)`; no input buffers, no neighbour reads at all |
/// | `gpu_heterogeneity`, `gpu_height`, `gpu_resistance` | per-cell, but every one reads several full input fields, so each band would re-upload data it does not use |
/// | `gpu_gauss_blur` | no — a radius-wide halo, and `blur_r * 3` at map scale is a large fraction of the grid |
/// | `gpu_jfa_plates` | no — jump flooding reads at ever-halving strides across the whole grid |
/// | `gpu_flow` | no — pointer doubling walks a receiver forest that spans the grid by construction |
/// | `gpu_weather` | no — advection carries moisture across the whole domain |
///
/// So warp is not the *convenient* choice, it is the only stage in the
/// current pipeline where this mode is a real partition rather than an
/// approximation, and this function says so rather than implying the mode
/// covers generation as a whole.
///
/// **What determinism this keeps, and what it does not.** On one device,
/// a band computed alone is bit-identical to the same rows computed with
/// the whole grid — the kernel reads nothing but its own coordinates. Across
/// *different* devices the ordinary GPU-vs-GPU caveat applies (two shader
/// compilers may contract multiply-adds differently), so a world generated
/// with a given device set is reproducible on that same set, and may differ
/// in the last bits from the same seed on a different set. That is the same
/// class of difference `DECISIONS.md` §7a already accepts between the CPU
/// and GPU paths, now one level finer; nothing about the `use_gpu = false`
/// path is affected.
///
/// Bands are sized by [`split_rows`] from [`set_weights`] and dispatched
/// concurrently, one thread per device — `wgpu`'s `Device`/`Queue` are
/// `Send + Sync`, and each band's readback blocks its own thread, so
/// without threads the devices would simply run one after another.
pub fn warp_grid_gpu_split(
    set: &GpuDeviceSet,
    width: u32,
    height: u32,
    seed: i32,
    wf: f32,
    amp: f32,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let devices = set.devices();
    let bands = split_rows(height, &set_weights(set));
    let n = (width as usize) * (height as usize);
    let mut out_x = vec![0f32; n];
    let mut out_y = vec![0f32; n];

    // Hand each thread a disjoint `&mut` slice of the final buffers, so a
    // band's readback lands where it belongs with no concatenation pass.
    // `split_at_mut` in a loop is what proves disjointness to the borrow
    // checker; nothing here needs a lock, and nothing needs `unsafe`.
    let mut rest_x = out_x.as_mut_slice();
    let mut rest_y = out_y.as_mut_slice();
    let mut jobs: Vec<WarpBandJob<'_>> = Vec::with_capacity(devices.len());
    for (gpu, &(y0, rows)) in devices.iter().zip(bands.iter()) {
        let take = (width as usize) * (rows as usize);
        let (mine_x, tail_x) = rest_x.split_at_mut(take);
        let (mine_y, tail_y) = rest_y.split_at_mut(take);
        rest_x = tail_x;
        rest_y = tail_y;
        jobs.push(WarpBandJob { gpu, y_offset: y0, rows, out_x: mine_x, out_y: mine_y });
    }

    // One band failing is the whole split failing: the bands that did land are
    // a partial grid, and a partial grid is not an answer. `None` here, and
    // the caller runs the CPU warp over the whole thing.
    let every_band_landed = std::thread::scope(|s| {
        let handles: Vec<_> = jobs
            .into_iter()
            .filter(|job| job.rows != 0)
            .map(|job| {
                s.spawn(move || {
                    let WarpBandJob { gpu, y_offset, rows, out_x, out_y } = job;
                    on_grid(gpu, width, height, || {
                        let ctx = init_gpu_warp_with(gpu);
                        dispatch_gpu_warp_band_into(&ctx, width, height, y_offset, rows, seed, wf, amp, out_x, out_y)
                    })
                    .is_some()
                })
            })
            .collect();
        // Collected, not short-circuited: every thread is joined before the
        // verdict, so a later band's failure is recorded too.
        handles.into_iter().map(|h| h.join().unwrap_or(false)).collect::<Vec<bool>>().into_iter().all(|ok| ok)
    });

    every_band_landed.then_some((out_x, out_y))
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
) -> Option<Vec<f32>> {
    on_grid(gpu, width, height, || {
        let ctx = init_gpu_heterogeneity_with(gpu);
        dispatch_gpu_heterogeneity(&ctx, width, height, hetero_seed, scale, age, warp_x, warp_y)
    })
}

/// `_with` sibling of [`gauss_blur_grid_gpu`].
pub fn gauss_blur_grid_gpu_with(
    gpu: &GpuDevice,
    src: &[f32],
    radius: f64,
    width: u32,
    height: u32,
    wrap_x: bool,
) -> Option<Vec<f32>> {
    on_grid(gpu, width, height, || {
        let ctx = init_gpu_gauss_blur_with(gpu);
        dispatch_gpu_gauss_blur(&ctx, src, radius, width, height, wrap_x)
    })
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
) -> Option<Vec<i32>> {
    on_grid(gpu, width, height, || {
        let ctx = init_gpu_jfa_plates_with(gpu);
        dispatch_gpu_assign_plates(&ctx, width, height, plate_x, plate_y, warp_x, warp_y)
    })
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

    // A device that cannot even read back an 8x8 grid fails the gate, which
    // is exactly what a self-test is for.
    let Some(gpu) = dispatch_gpu(ctx, W, H, SEED, SCALE) else {
        return false;
    };
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
        // A failed readback falls THROUGH to the CPU path below rather than
        // returning -- §27's "transition to CPU", and the reported
        // `ComputePath` stays honest about which path produced the values.
        if let Some(values) = dispatch_gpu(ctx, width, height, seed, scale) {
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
    }
    let t0 = Instant::now();
    let values = vnoise_grid_cpu(width, height, seed, scale);
    let cpu_time = t0.elapsed();
    VnoiseResult { values, path: ComputePath::Cpu, gpu_dispatch_and_readback: None, cpu_duration: cpu_time }
}

#[cfg(test)]
#[allow(clippy::excessive_precision)] // milestone 7's weather test reuses a real f32 fixture verbatim, matching cartalith-climate's own golden_parity_weather.rs convention
mod tests {
    use super::*;

    // -- Readback shims --------------------------------------------------
    //
    // Every `dispatch_gpu_*` returns `Option` now, because the shipped path
    // has to fall back to CPU when a device cannot complete a readback. A
    // *test* must do the opposite: a readback failure here means the kernel
    // under test never ran, and silently comparing nothing against nothing is
    // exactly the "silently-empty golden output" failure this project has
    // already been bitten by four times (root `CLAUDE.md`). These same-named
    // wrappers shadow the glob import above (an item in the module wins over
    // a glob), so each test body reads unchanged and any failure is loud.
    macro_rules! unwrapping_dispatch {
        ($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
            #[allow(clippy::too_many_arguments)]
            fn $name($($arg: $ty),*) -> $ret {
                super::$name($($arg),*).expect(concat!(stringify!($name), ": GPU readback failed in a test"))
            }
        };
    }
    unwrapping_dispatch!(dispatch_gpu(ctx: &GpuContext, width: u32, height: u32, seed: i32, scale: f32) -> Vec<f32>);
    unwrapping_dispatch!(dispatch_gpu_warp(ctx: &GpuContext, width: u32, height: u32, seed: i32, wf: f32, amp: f32) -> (Vec<f32>, Vec<f32>));
    unwrapping_dispatch!(dispatch_gpu_heterogeneity(ctx: &GpuContext, width: u32, height: u32, hetero_seed: i32, scale: f32, age: &[f32], warp_x: &[f32], warp_y: &[f32]) -> Vec<f32>);
    unwrapping_dispatch!(dispatch_gpu_height(ctx: &GpuContext, width: u32, height: u32, seed: i32, nf: f32, a: f32, b: f32, age_inf: f32, fwt: f32, hwt: f32, ridged: bool, has_oro: bool, base_field: &[f32], stress: &[f32], flex: &[f32], hetero: &[f32], age: &[f32], warp_x: &[f32], warp_y: &[f32], oro: &[f32]) -> Vec<f32>);
    unwrapping_dispatch!(dispatch_gpu_gauss_blur(ctx: &GpuBlurContext, src: &[f32], radius: f64, width: u32, height: u32, wrap_x: bool) -> Vec<f32>);
    unwrapping_dispatch!(dispatch_gpu_resistance(ctx: &GpuContext, width: u32, height: u32, plate_id: &[u32], age: &[f32], crustal_per_plate: &[f32]) -> Vec<f32>);
    unwrapping_dispatch!(dispatch_gpu_assign_plates(ctx: &GpuContext, width: u32, height: u32, plate_x: &[f32], plate_y: &[f32], warp_x: Option<&[f32]>, warp_y: Option<&[f32]>) -> Vec<i32>);
    unwrapping_dispatch!(dispatch_gpu_flow(ctx: &GpuFlowContext, gw: usize, gh: usize, field: &[f32], rain: Option<&[f32]>, use_rain: bool, world: bool) -> GpuFlowResult);

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
        let quote = timings_quotable("measured_gpu_vs_cpu_timing");
        if quote {
            eprintln!("vnoise pilot kernel {}", device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type));
        }

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let scale = 0.02f32;
            let n = (w * h) as usize;

            let (gpu_t, gpu) = timed_for(quote, || dispatch_gpu(&ctx, w, h, seed, scale));
            let (cpu_t, cpu) = timed_for(quote, || vnoise_grid_cpu(w, h, seed, scale));

            assert_eq!(gpu.len(), n, "GPU field is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu.len(), n, "CPU field is the wrong length -- the call that was timed produced nothing usable");

            if quote {
                eprintln!(
                    "{w}x{h} ({n} cells): GPU dispatch+readback = {gpu_t}, CPU (single-thread) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
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
        let quote = timings_quotable("measured_gpu_safe_noise_vs_cpu_timing");
        if quote {
            eprintln!("gpu-safe noise {}", device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type));
        }

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let scale = 0.02f32;
            let n = (w * h) as usize;

            let (gpu_t, gpu) = timed_for(quote, || dispatch_gpu(&ctx, w, h, seed, scale));
            let (cpu_t, cpu) = timed_for(quote, || gpu_safe_noise_grid_cpu(w, h, seed, scale));

            assert_eq!(gpu.len(), n, "GPU field is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu.len(), n, "CPU field is the wrong length -- the call that was timed produced nothing usable");

            if quote {
                eprintln!(
                    "gpu-safe noise {w}x{h} ({n} cells): GPU dispatch+readback = {gpu_t}, CPU (single-thread) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
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
        let instance = multi::compute_instance();
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
        let quote = timings_quotable("measured_gpu_warp_vs_cpu_timing");
        if quote {
            eprintln!("gpu_warp {}", device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type));
        }
        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let seed = 24601;
            let wf = 2.5 / w as f32;
            let amp = 40.0f32;
            let n = (w * h) as usize;

            let (gpu_t, gpu) = timed_for(quote, || dispatch_gpu_warp(&ctx, w, h, seed, wf, amp));
            let (cpu_t, cpu) = timed_for(quote, || gpu_warp_grid_cpu(w, h, seed, wf, amp));

            assert_eq!((gpu.0.len(), gpu.1.len()), (n, n), "GPU warp field is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!((cpu.0.len(), cpu.1.len()), (n, n), "CPU warp field is the wrong length -- the call that was timed produced nothing usable");

            if quote {
                eprintln!(
                    "gpu_warp {w}x{h} ({n} cells): GPU dispatch+readback = {gpu_t}, CPU (single-thread) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
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
        let quote = timings_quotable("measured_gpu_heterogeneity_vs_cpu_timing");
        if quote {
            eprintln!("gpu_heterogeneity {}", device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type));
        }

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let hetero_seed = 24601 ^ 0x44bb;
            let scale = 1.5 * 12.0 / w as f32;
            let age = vec![0.6f32; n];
            let warp_x = vec![0.0f32; n];
            let warp_y = vec![0.0f32; n];

            let (gpu_t, gpu) =
                timed_for(quote, || dispatch_gpu_heterogeneity(&ctx, w, h, hetero_seed, scale, &age, &warp_x, &warp_y));
            let (cpu_t, cpu) =
                timed_for(quote, || gpu_heterogeneity_grid_cpu(w, h, hetero_seed, scale, &age, &warp_x, &warp_y));

            assert_eq!(gpu.len(), n, "GPU field is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu.len(), n, "CPU field is the wrong length -- the call that was timed produced nothing usable");

            if quote {
                eprintln!(
                    "gpu_heterogeneity {w}x{h} ({n} cells): GPU dispatch+readback = {gpu_t}, CPU (single-thread) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
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

    // The timing harness every `measured_*` test below uses -- `Timing`,
    // `timed`, `timed_for`, `ratio`, `timings_quotable`, `device_note` --
    // lives in `src/timing_harness.rs` so `tests/multi_gpu.rs` can pull the
    // identical one in with `#[path]`. Two definitions of "how a timing is
    // taken" is how two of them drift apart.
    use crate::timing_harness::*;

    /// The timing harness's own tests. They live here rather than beside it
    /// because `tests/multi_gpu.rs` compiles that file a second time via
    /// `#[path]`, where `cfg(test)` is also set -- so a test module inside it
    /// would run twice, in two binaries, for no extra coverage.
    mod harness_tests {
        use std::time::Duration;

        use crate::timing_harness::*;

        /// The property the doc comment claims, asserted independently of the
        /// value: odd so the median is a real sample, and above one so there is a
        /// range to report. `TIMING_ROUNDS = 1` or `= 4` must fail here.
        #[test]
        fn timing_rounds_is_an_odd_number_above_one() {
            assert_eq!(TIMING_ROUNDS % 2, 1, "an even sample count makes the median an average of two samples");
            assert!(TIMING_ROUNDS > 1, "one sample has no range, which is the whole defect this harness exists to fix");
        }

        #[test]
        fn timed_draws_exactly_the_rounds_asked_for_and_orders_them() {
            let mut calls = 0usize;
            let (t, last) = timed(3, || {
                calls += 1;
                calls
            });
            assert_eq!(calls, 3, "timed must call f once per round");
            assert_eq!(last, 3, "timed must return the last value produced, not the first");
            assert_eq!(t.rounds, 3);
            assert!(t.min <= t.median && t.median <= t.max, "min <= median <= max");
        }

        #[test]
        fn timed_for_draws_one_sample_when_it_may_not_quote() {
            let mut calls = 0usize;
            let (t, ()) = timed_for(false, || calls += 1);
            assert_eq!(calls, 1, "a run that cannot print a figure must not pay for five samples");
            assert_eq!(t.rounds, 1);
        }

        /// A one-sample `Timing` must never render as though it were measured --
        /// that is the exact failure this file exists to prevent.
        #[test]
        fn a_single_sample_says_so_in_both_renderings() {
            let one = Timing { median: Duration::from_millis(10), min: Duration::from_millis(10), max: Duration::from_millis(10), rounds: 1 };
            let five = Timing { median: Duration::from_millis(10), min: Duration::from_millis(8), max: Duration::from_millis(20), rounds: 5 };
            assert!(one.to_string().contains("not a measurement"), "got {one}");
            assert!(ratio(five, one).contains("not a measurement"), "a ratio is only as sampled as its worst operand");
            assert!(!five.to_string().contains("not a measurement"), "got {five}");
        }

        #[test]
        fn a_ratio_carries_the_bracket_its_two_ranges_permit() {
            let num = Timing { median: Duration::from_millis(100), min: Duration::from_millis(90), max: Duration::from_millis(110), rounds: 5 };
            let den = Timing { median: Duration::from_millis(50), min: Duration::from_millis(40), max: Duration::from_millis(60), rounds: 5 };
            // 100/50 = 2.00; low = 90/60 = 1.50; high = 110/40 = 2.75.
            assert_eq!(ratio(num, den), "2.00x (1.50..2.75x)");
        }

        #[test]
        fn spread_is_max_over_min() {
            let t = Timing { median: Duration::from_millis(10), min: Duration::from_millis(8), max: Duration::from_millis(20), rounds: 5 };
            assert!((t.spread() - 2.5).abs() < 1e-9, "got {}", t.spread());
            assert!((t.ms() - 10.0).abs() < 1e-9);
            // 10 ms is 1e7 ns; over 1e6 cells that is 10 ns/cell.
            assert!((t.ns_per_cell(1_000_000) - 10.0).abs() < 1e-9, "got {}", t.ns_per_cell(1_000_000));
        }

        fn argv(parts: &[&str]) -> std::vec::IntoIter<String> {
            parts.iter().map(|s| (*s).to_string()).collect::<Vec<_>>().into_iter()
        }

        /// All three shapes libtest accepts, plus the ones that must NOT count as
        /// serialised. A detector that answers `true` too readily is worse than
        /// none: it re-opens the door this harness closes.
        #[test]
        fn every_way_of_asking_for_one_thread_is_recognised() {
            assert!(serialised_from(argv(&["bin", "--test-threads=1"]), None));
            assert!(serialised_from(argv(&["bin", "--test-threads", "1"]), None));
            assert!(serialised_from(argv(&["bin", "--nocapture"]), Some("1")));

            assert!(!serialised_from(argv(&["bin", "--nocapture"]), None));
            assert!(!serialised_from(argv(&["bin", "--test-threads=8"]), None));
            assert!(!serialised_from(argv(&["bin", "--test-threads", "8"]), None));
            assert!(!serialised_from(argv(&["bin"]), Some("8")));
            // A bare trailing `--test-threads` with no value is not a request for one.
            assert!(!serialised_from(argv(&["bin", "--test-threads"]), None));
            // The flag wins over the variable, in both directions.
            assert!(!serialised_from(argv(&["bin", "--test-threads=8"]), Some("1")));
            assert!(serialised_from(argv(&["bin", "--test-threads=1"]), Some("8")));
            // A filter that merely contains the text is not the flag.
            assert!(!serialised_from(argv(&["bin", "my_test_threads_1"]), None));
        }
    }

    /// One tiny dispatch, so the first *measured* one is not also paying the
    /// driver's first-use costs. Shared by the three `gpu_height` timing
    /// tests below so all of their numbers are comparable to each other.
    fn warm_up_height(ctx: &GpuContext) {
        let (base, stress, flex, hetero, age) = synthetic_height_inputs(64);
        let zero = vec![0.0f32; 64];
        let _ = dispatch_gpu_height(
            ctx, 8, 8, 1, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age, &zero,
            &zero, &zero,
        );
    }

    #[test]
    fn measured_gpu_height_vs_cpu_timing() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping timing measurement");
            return;
        };
        warm_up_height(&ctx);
        let quote = timings_quotable("measured_gpu_height_vs_cpu_timing");
        if quote {
            eprintln!(
                "gpu_height vs its single-threaded f32 twin {}",
                device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type)
            );
        }

        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
            let warp_x = vec![0.0f32; n];
            let warp_y = vec![0.0f32; n];
            let oro = vec![0.0f32; n];

            let (gpu_t, gpu) = timed_for(quote, || {
                dispatch_gpu_height(
                    &ctx, w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero,
                    &age, &warp_x, &warp_y, &oro,
                )
            });

            let (cpu_t, cpu) = timed_for(quote, || {
                gpu_height_grid_cpu(
                    w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero, &age,
                    &warp_x, &warp_y, &oro,
                )
            });

            assert_eq!(gpu.len(), n, "GPU field is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu.len(), n, "CPU field is the wrong length -- the call that was timed produced nothing usable");

            if quote {
                eprintln!(
                    "gpu_height {w}x{h} ({n} cells): GPU dispatch+readback = {gpu_t}, CPU (single-thread) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
        }
    }

    /// `OUTSTANDING_WORK.md` §2.6, *"investigate the `gpu_height` throughput
    /// drop from 1024² (8.13×) to 2048² (4.84×)"*. The cause on record --
    /// **memory-bandwidth-bound at 9 buffers** -- was stated and untested.
    /// **Tested 2026-09-03, and it holds.** This test is the whole
    /// experiment, self-contained so its three kernels are timed in one run
    /// on one device rather than assembled out of numbers from three others.
    ///
    /// It measures two things at 1024² and 2048²:
    ///
    /// 1. *The cross-kernel control.* `gpu_warp` (2 storage buffers, 8 B/cell
    ///    moved), `gpu_heterogeneity` (4 buffers, 16 B/cell) and `gpu_height`
    ///    (9 buffers, 36 B/cell) all dispatch and read back through the same
    ///    code path. If the drop were about grid size, it would show in all
    ///    three; if it is about bytes moved, only the widest bind group turns
    ///    around. Re-measured alone 2026-09-05, ns/cell medians: the two narrow
    ///    kernels get **cheaper** per cell as the grid grows (`gpu_warp` 1.84
    ///    -> 1.66, `gpu_heterogeneity` 2.23 -> 1.85 -- fixed dispatch overhead
    ///    amortising over 4× the cells) while the nine-buffer one gets
    ///    **2.19× dearer** (3.92 -> 8.58). The direction is what the argument
    ///    rests on and it is not close; the factor is one box's.
    ///
    /// 2. *The direct half.* The eight host→device input uploads timed on
    ///    their own against the full dispatch. They are the majority of it at
    ///    both sizes -- **82% at 1024², 63% at 2048²** of the dispatch median,
    ///    re-measured alone 2026-09-05, at 9.22 and 5.51 GiB/s.
    ///    `create_buffer_init` maps at creation and unmaps, so a buffer's cost
    ///    can partly defer to the next submit -- the flush below makes this a
    ///    **floor** on the upload cost, not an estimate of all of it, which
    ///    only strengthens the conclusion.
    ///
    /// Every figure is a **median of [`TIMING_ROUNDS`] printed with the range
    /// its samples spanned**, because a single GPU dispatch on this hardware
    /// varied 38-78 ms at 2048² across runs while this row was being answered
    /// -- `OUTSTANDING_WORK.md` §2.6's own *"single-run variance is
    /// indistinguishable from a result"*, in miniature. Three timing tests
    /// sharing one GPU is itself a source of spread, so [`timings_quotable`]
    /// prints nothing at all unless the binary was started with
    /// `--test-threads=1`.
    ///
    /// The consequence worth carrying forward: this kernel's cost is set by
    /// its **bind group**, not its formula. That is why `gpu_height` cannot be
    /// pointed at a bigger grid and expected to keep its 1024² win against the
    /// f32 twin, and it is one of the two reasons the height stage stays
    /// unwired -- see [`dispatch_gpu_height`] for the decision and the other
    /// reason.
    #[test]
    fn measured_gpu_height_is_bandwidth_bound_at_nine_buffers() {
        let (Some(hctx), Some(wctx), Some(xctx)) = (try_gpu_height(), try_gpu_warp(), try_gpu_heterogeneity()) else {
            eprintln!("no GPU available -- skipping bandwidth measurement");
            return;
        };
        warm_up_height(&hctx);
        let quote = timings_quotable("measured_gpu_height_is_bandwidth_bound_at_nine_buffers");
        if quote {
            eprintln!(
                "bind-group width vs per-cell cost {}",
                device_note(&hctx.adapter_name, hctx.adapter_backend, hctx.device_type)
            );
        }
        for &(w, h) in &[(1024u32, 1024u32), (2048, 2048)] {
            let n = (w * h) as usize;
            let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
            let zero = vec![0.0f32; n];

            let (warp_t, _) = timed_for(quote, || dispatch_gpu_warp(&wctx, w, h, 24601, 2.5 / w as f32, 40.0));
            let (hetero_t, _) = timed_for(quote, || {
                dispatch_gpu_heterogeneity(&xctx, w, h, 24601 ^ 0x44bb, 18.0 / w as f32, &age, &zero, &zero)
            });
            let (height_t, out) = timed_for(quote, || {
                dispatch_gpu_height(
                    &hctx, w, h, 24601, 5.0, 0.5, 0.3, 0.5, 0.15, 0.1, false, false, &base, &stress, &flex, &hetero,
                    &age, &zero, &zero, &zero,
                )
            });

            // The eight input uploads that dispatch performed, created exactly
            // as `dispatch_gpu_height` creates them (same usage, same path).
            let inputs: [&[f32]; 8] = [&base, &stress, &flex, &hetero, &age, &zero, &zero, &zero];
            let (upload_t, uploaded_bytes) = timed_for(quote, || {
                let bufs: Vec<wgpu::Buffer> = inputs
                    .iter()
                    .map(|data| {
                        hctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("height upload probe (storage)"),
                            contents: bytemuck::cast_slice(data),
                            usage: wgpu::BufferUsages::STORAGE,
                        })
                    })
                    .collect();
                hctx.queue.submit(std::iter::empty());
                let _ = hctx.device.poll(wgpu::PollType::wait_indefinitely());
                bufs.iter().map(|b| b.size()).sum::<u64>()
            });

            if quote {
                // Per-cell figures are medians; the bracket beside each is the
                // same statistic taken from that timing's own min and max, so
                // a reader can see whether two kernels' costs actually
                // separate or merely have separated medians.
                let per_cell = |t: Timing| {
                    format!(
                        "{:.2} ({:.2}..{:.2}, {:.2}x spread)",
                        t.ns_per_cell(n),
                        t.min.as_secs_f64() * 1e9 / n as f64,
                        t.max.as_secs_f64() * 1e9 / n as f64,
                        t.spread()
                    )
                };
                eprintln!(
                    "{w}x{h} ns/cell (median of {TIMING_ROUNDS}, range in brackets): gpu_warp[2 buf, 8 B/cell] = {}, \
                     gpu_heterogeneity[4 buf, 16 B/cell] = {}, gpu_height[9 buf, 36 B/cell] = {} \
                     -- of which {:.1} MiB of input upload = {upload_t} ({:.0}% of the dispatch median, {:.2} GiB/s at the median)",
                    per_cell(warp_t),
                    per_cell(hetero_t),
                    per_cell(height_t),
                    uploaded_bytes as f64 / (1024.0 * 1024.0),
                    100.0 * upload_t.secs() / height_t.secs().max(1e-9),
                    uploaded_bytes as f64 / (1024.0 * 1024.0 * 1024.0) / upload_t.secs().max(1e-9),
                );
            }

            // Non-vacuity, per root `CLAUDE.md`'s "watch for silently-empty
            // golden output": the probe must have moved the bytes it claims
            // (8 buffers x 4 B x n) and the dispatch must have produced a
            // full, finite field rather than nothing at all.
            assert_eq!(uploaded_bytes, 8 * 4 * n as u64, "upload probe did not bind this kernel's real input footprint");
            assert_eq!(out.len(), n);
            assert!(out.iter().all(|v| v.is_finite()), "no NaN/Inf in the GPU height field");
        }
    }

    /// `OUTSTANDING_WORK.md` §2.6, *"decide `gpu_compute_height`'s status --
    /// built, verified, and never called"*. **Decided 2026-09-03: an
    /// undocumented consequence of milestone 8's shared device, plus a speed
    /// case that does not survive contact with the real CPU function. It is
    /// not a forgotten gap, and it should stay unwired.** The full reasoning
    /// is on [`dispatch_gpu_height`]; this test is the second half's
    /// evidence, and re-running it is how a later pass would overturn the
    /// decision.
    ///
    /// Every published `gpu_height` speedup -- 5.17× at 512², 8.13× at 1024²,
    /// 4.84× at 2048² (`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 3) -- is
    /// measured against [`gpu_height_grid_cpu`], which is a **single-threaded
    /// `f32` twin written to match the shader**. The function
    /// `generate_terrain` actually calls is `cartalith_terrain::compute_height`,
    /// which is `f64` throughout and already `par_chunks_mut` across every
    /// core (`CPU_MULTITHREADING_SCOPE.md`). Those are not the same baseline,
    /// and the ratio against the one that ships is the only one that decides
    /// whether wiring the stage would make a generation faster.
    #[test]
    fn measured_gpu_height_vs_the_real_compute_height() {
        let Some(ctx) = try_gpu_height() else {
            eprintln!("no GPU available -- skipping production-baseline measurement");
            return;
        };
        warm_up_height(&ctx);
        let quote = timings_quotable("measured_gpu_height_vs_the_real_compute_height");
        if quote {
            eprintln!(
                "gpu_height vs the shipped f64 rayon compute_height {}",
                device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type)
            );
        }
        for &(w, h) in &[(1024u32, 1024u32), (2048, 2048)] {
            let n = (w * h) as usize;
            let (base, stress, flex, hetero, age) = synthetic_height_inputs(n);
            let zero = vec![0.0f32; n];
            let (nf, a, b, age_inf, fwt, hwt) = (5.0f32, 0.5f32, 0.3f32, 0.5f32, 0.15f32, 0.1f32);

            let (gpu_time, gpu) = timed_for(quote, || {
                dispatch_gpu_height(
                    &ctx, w, h, 24601, nf, a, b, age_inf, fwt, hwt, false, false, &base, &stress, &flex, &hetero, &age,
                    &zero, &zero, &zero,
                )
            });

            let (real_time, real) = timed_for(quote, || {
                cartalith_terrain::compute_height(
                    w as usize,
                    h as usize,
                    &base,
                    &stress,
                    &flex,
                    &hetero,
                    &age,
                    None,
                    None,
                    None,
                    &cartalith_terrain::HeightParams {
                        nf: nf as f64,
                        seed: 24601,
                        a: a as f64,
                        b: b as f64,
                        age_inf: age_inf as f64,
                        fwt: fwt as f64,
                        hwt: hwt as f64,
                        world: false,
                        ridged: false,
                    },
                )
            });

            // Not a tolerance check: `compute_height` evaluates `fbm` in f64
            // and the shader evaluates `gpu_fbm` in f32, which is a different
            // noise regime by design (`DECISIONS.md` §7c) -- the divergence is
            // reported so the size of what `use_gpu` would change is on the
            // record next to the speed it would buy, not asserted on.
            let max_abs_diff = gpu
                .iter()
                .zip(real.iter())
                .map(|(g, r)| ((*g as f64) - (*r as f64)).abs())
                .fold(0.0f64, f64::max);

            if quote {
                eprintln!(
                    "gpu_height {w}x{h}: GPU = {gpu_time}, \
                     REAL cartalith_terrain::compute_height (f64, rayon) = {real_time}, ratio (CPU/GPU) = {}, \
                     max_abs_diff vs the shipped field = {max_abs_diff:.4}",
                    ratio(real_time, gpu_time),
                );
            }

            assert_eq!(gpu.len(), n);
            assert_eq!(real.len(), n);
            assert!(max_abs_diff > 0.0, "GPU and f64 CPU height came back identical -- one of the two never ran");
        }
    }

    /// `OUTSTANDING_WORK.md` §2.6, the two rows that both turn on one number:
    /// *"per-pipeline caching across repeated `generate_terrain` calls"* and
    /// *"hardware capability cache (§30) -- re-measure the handshake before
    /// building anything"*. This is that measurement, and it is what a second
    /// `generate_terrain` call pays before it computes anything.
    ///
    /// `generate_terrain` opens its device set once per call
    /// (`cartalith-engine/src/lib.rs`, `init_gpu_device_set`) and drops it at
    /// the end, so **call two rebuilds all of it**: one adapter/device
    /// handshake, plus one pipeline per stage, since every `*_grid_gpu_with`
    /// entry point calls its own `init_gpu_*_with` inside the dispatch rather
    /// than taking a built context. (`gpu_flow` is the one exception already
    /// fixed -- milestone 9 hoisted it because it is called four times within
    /// a single call.)
    ///
    /// `#[ignore]`d: it opens fresh adapters in a loop, which is slow and is
    /// exactly the cost being measured. Run it with
    /// `cargo test --release -p cartalith-gpu measured_device_handshake -- --ignored --test-threads=1 --nocapture`.
    ///
    /// **This one panics rather than suppressing.** The other `measured_*`
    /// tests run inside the default suite, so [`timings_quotable`] lets them
    /// fall back to assertions and print nothing; this one runs only when
    /// somebody typed `--ignored`, so the only reason to be here is the
    /// figure, and producing a contended one is worse than failing.
    ///
    /// The **first** handshake of the process is separated out, because there
    /// is only ever one of it: it cannot be given a spread, it measures dearer
    /// than the ones after it, and it is the one a fresh `generate_terrain` in
    /// a fresh process actually pays. Pooling it with its successors is how
    /// "416 ms" was published and then re-measured at 730 ms.
    #[test]
    #[ignore = "opens fresh adapters in a loop (that is the cost being measured), and refuses to run without --test-threads=1"]
    fn measured_device_handshake_and_per_stage_pipeline_build() {
        assert!(
            timing_is_serialised(),
            "measured_device_handshake_and_per_stage_pipeline_build refuses to run under a parallel test binary -- \
             a handshake timed against other tests' GPU work is a figure about contention, which is how a 416 ms \
             handshake was published and re-measured at 730 ms. Re-run with: cargo test --release -p cartalith-gpu \
             measured_device_handshake -- --ignored --test-threads=1 --nocapture"
        );

        // The cold one, on its own: it is not a sample of the same population
        // as the warm ones and must not be pooled with them.
        let t0 = Instant::now();
        let first = init_gpu_shared_device();
        let cold = t0.elapsed();
        if first.is_err() {
            eprintln!("no GPU available -- skipping handshake measurement");
            return;
        }
        drop(first);

        let (warm, dev) = timed(TIMING_ROUNDS, init_gpu_shared_device);
        let Ok(gpu) = dev else {
            eprintln!("no GPU available -- skipping handshake measurement");
            return;
        };
        eprintln!("handshake {}", device_note(&gpu.adapter_name, gpu.adapter_backend, gpu.device_type));
        eprintln!(
            "init_gpu_shared_device: first-of-process = {cold:?} [1 sample by definition -- there is only one cold \
             handshake per process], then warm = {warm}"
        );

        let mut pipeline_total = std::time::Duration::ZERO;
        let mut pipeline_min = std::time::Duration::MAX;
        let mut pipeline_max = std::time::Duration::ZERO;
        for (label, build) in [
            ("warp", &(|g: &GpuDevice| { init_gpu_warp_with(g); }) as &dyn Fn(&GpuDevice)),
            ("heterogeneity", &|g: &GpuDevice| { init_gpu_heterogeneity_with(g); }),
            ("jfa_plates", &|g: &GpuDevice| { init_gpu_jfa_plates_with(g); }),
            ("gauss_blur", &|g: &GpuDevice| { init_gpu_gauss_blur_with(g); }),
            ("flow", &|g: &GpuDevice| { init_gpu_flow_with(g); }),
            ("weather", &|g: &GpuDevice| { init_gpu_weather_with(g); }),
        ] {
            let (t, ()) = timed(TIMING_ROUNDS, || build(&gpu));
            pipeline_total += t.median;
            pipeline_min = pipeline_min.min(t.median);
            pipeline_max = pipeline_max.max(t.median);
            eprintln!("  init_gpu_{label}_with (shader compile + pipeline) = {t}");
        }
        eprintln!(
            "a second generate_terrain call in the same process rebuilds: 1 warm handshake ({warm}) + every pipeline \
             (medians sum to {pipeline_total:?} for all six; the dearest single stage's median is {pipeline_max:?}, \
             the cheapest {pipeline_min:?})"
        );

        assert!(cold > std::time::Duration::ZERO, "the cold handshake took no measurable time -- nothing ran");
        assert!(warm.rounds > 1 && warm.min <= warm.max, "the warm handshake did not draw a sample set with a range");
        assert!(pipeline_total > std::time::Duration::ZERO, "pipeline builds took no measurable time -- nothing ran");
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
        let plate_id_u16: Vec<u16> = (0..n).map(|i| (i % num_plates) as u16).collect();
        let plate_id_u32: Vec<u32> = plate_id_u16.iter().map(|&p| u32::from(p)).collect();
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
        let real_cpu = cartalith_terrain::compute_resistance(w as usize, h as usize, &plate_id_u16, &plates, &age);

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
        let quote = timings_quotable("measured_gpu_jfa_plates_vs_cpu_timing");
        if quote {
            eprintln!("gpu_jfa_plates {}", device_note(&ctx.adapter_name, ctx.adapter_backend, ctx.device_type));
        }

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

            let n = (w * h) as usize;
            let (gpu_t, gpu) = timed_for(quote, || dispatch_gpu_assign_plates(&ctx, w, h, &px, &py, None, None));
            let (cpu_t, cpu) =
                timed_for(quote, || cartalith_terrain::assign_plates(w as usize, h as usize, false, &plates, None, None));

            assert_eq!(gpu.len(), n, "GPU plate map is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu.len(), n, "CPU plate map is the wrong length -- the call that was timed produced nothing usable");

            let max_dim = w.max(h) as f64;
            let passes = max_dim.log2().ceil() as u32;
            if quote {
                eprintln!(
                    "{w}x{h} ({n} cells, {passes} JFA passes, {np} plates): GPU dispatch+readback = {gpu_t}, CPU (single-thread, in-place JFA) = {cpu_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_t, gpu_t)
                );
            }
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
        let quote = timings_quotable("measured_gpu_blur_and_resistance_timing");
        if quote {
            eprintln!(
                "gpu_gauss_blur + gpu_compute_resistance {}",
                device_note(&blur_ctx.adapter_name, blur_ctx.adapter_backend, blur_ctx.device_type)
            );
        }
        for &(w, h) in &[(128u32, 128u32), (512, 512), (1024, 1024), (2048, 2048)] {
            let n = (w * h) as usize;
            let src = synthetic_field(n, 23);

            let (gpu_blur_t, gpu_blur) =
                timed_for(quote, || dispatch_gpu_gauss_blur(&blur_ctx, &src, 12.0, w, h, false));
            let (cpu_blur_t, cpu_blur) =
                timed_for(quote, || cartalith_terrain::gauss_blur(&src, 12.0, w as usize, h as usize, false));
            assert_eq!(gpu_blur.len(), n, "GPU blur is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu_blur.len(), n, "CPU blur is the wrong length -- the call that was timed produced nothing usable");
            if quote {
                eprintln!(
                    "gpu_gauss_blur {w}x{h}: GPU = {gpu_blur_t}, CPU (real, running-sum) = {cpu_blur_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_blur_t, gpu_blur_t)
                );
            }

            let num_plates = 9usize;
            let plate_id_u32: Vec<u32> = (0..n).map(|i| (i % num_plates) as u32).collect();
            let plate_id_u16: Vec<u16> = plate_id_u32.iter().map(|&p| p as u16).collect();
            let age = synthetic_field(n, 29);
            let plates: Vec<cartalith_terrain::Plate> = (0..num_plates)
                .map(|k| cartalith_terrain::Plate { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, base: (k as f64 - 4.0) * 0.3 })
                .collect();
            let crustal_per_plate: Vec<f32> = plates.iter().map(|p| p.base.max(0.0) as f32).collect();

            let (gpu_res_t, gpu_res) =
                timed_for(quote, || dispatch_gpu_resistance(&res_ctx, w, h, &plate_id_u32, &age, &crustal_per_plate));
            let (cpu_res_t, cpu_res) = timed_for(quote, || {
                cartalith_terrain::compute_resistance(w as usize, h as usize, &plate_id_u16, &plates, &age)
            });
            assert_eq!(gpu_res.len(), n, "GPU resistance is the wrong length -- the dispatch that was timed produced nothing usable");
            assert_eq!(cpu_res.len(), n, "CPU resistance is the wrong length -- the call that was timed produced nothing usable");
            if quote {
                eprintln!(
                    "gpu_compute_resistance {w}x{h}: GPU = {gpu_res_t}, CPU (real, rayon) = {cpu_res_t}, ratio (CPU/GPU) = {}",
                    ratio(cpu_res_t, gpu_res_t)
                );
            }
        }
    }

    // GPU_LAYER_INTEGRATION_SCOPE.md milestone 7: simulate_weather's inner
    // loop. No noise dependency (grepped: cartalith-climate doesn't import
    // cartalith-noise), so verified directly against the REAL, untouched
    // cartalith_climate::simulate_weather -- same discipline milestone 4
    // used for gauss_blur/compute_resistance, not a GPU-vs-CPU-twin
    // carve-out.

    fn weather_test_field_and_params() -> (usize, usize, Vec<f32>, cartalith_climate::WeatherParams) {
        // Same field/params as cartalith-climate's own
        // golden_parity_weather.rs::simulate_weather_case_0 -- a real,
        // already-trusted input, not synthesized fresh for this test.
        let field: Vec<f32> = vec![
            0.30000001192092896, 0.41398876905441284, 0.5219740271568298, 0.618268609046936, 0.6978008151054382,
            0.7563819885253906, 0.7909267544746399, 0.7996158003807068, 0.7819914817810059, 0.7389820218086243,
            0.672852635383606, 0.5870860815048218, 0.4861995279788971, 0.37550634145736694, 0.33916351199150085,
            0.45177075266838074, 0.5563846826553345, 0.6474955081939697, 0.7203047275543213, 0.7709776163101196,
            0.7968454957008362, 0.7965459227561951, 0.770094633102417, 0.7188847661018372, 0.6456133723258972,
            0.5541395545005798, 0.44928085803985596, 0.3365600109100342, 0.3780863881111145, 0.48862019181251526,
            0.589219868183136, 0.6745871901512146, 0.7402260303497314, 0.7826793789863586, 0.7997113466262817,
            0.7904249429702759, 0.7553091645240784, 0.6962136030197144, 0.6162505149841309, 0.5196314454078674,
            0.4114449620246887, 0.30261099338531494, 0.41652944684028625, 0.5243106484413147, 0.620278000831604,
            0.699377179145813, 0.7574422955513, 0.7914152145385742, 0.799506664276123, 0.7812904715538025,
            0.737726092338562, 0.6711078882217407, 0.5849444270133972, 0.48377376794815063, 0.372924268245697,
            0.34176596999168396, 0.45425650477409363, 0.5586227774620056, 0.6493681073188782, 0.7217131853103638,
            0.7718478441238403, 0.7971315383911133, 0.7962327599525452, 0.7691987752914429, 0.7174533605575562,
            0.643721878528595, 0.5518875122070312, 0.44678691029548645, 0.33395546674728394, 0.38066428899765015,
            0.49103569984436035, 0.5913457870483398, 0.6763115525245667, 0.741457998752594, 0.7833540439605713,
            0.7997932434082031, 0.7899097204208374, 0.7542240023612976, 0.6946155428886414, 0.614223837852478,
        ];
        let p = cartalith_climate::WeatherParams {
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
            pole_temp: -25.0,
            equator_temp: 30.0,
            tilt_deg: 23.4,
            rotation_hours: 24.0,
            lapse_rate: 6.5,
            sea_level: 0.42,
            peak_m: 4000.0,
            wind_manual: false,
            wind_dir_deg: 0.0,
            press_k: 0.6,
            ocean_hum: 1.0,
            evap: 0.12,
            ocean: 1.0,
            rain_k: 1.0,
            rain_dep: 0.35,
            bulk_evap: true,
            terrain_wind_deflection: false,
            currents: false,
            current_k: 1.0,
        };
        (10, 8, field, p)
    }

    #[test]
    fn gpu_weather_loop_matches_real_cpu_simulate_weather() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let (gw, gh, field, p) = weather_test_field_and_params();
        // Real production default (cartalith-engine's WorldParams::default,
        // `w_iters: 70`), not the golden test's own iters=5 -- exercises the
        // full compounding-over-iterations case this milestone cares about.
        let iters = 70;

        let grid = cartalith_climate::build_weather_grid(gw, gh, &field, 0.0, &p);
        let (_w, rain) = simulate_weather_loop_gpu_with(
            &gpu,
            &grid.eh,
            &grid.tc,
            &grid.sst_evap,
            &grid.wx,
            &grid.wy,
            &grid.w_init,
            grid.ww as u32,
            grid.wh as u32,
            iters,
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
        .expect("the weather loop must complete on this device");
        let gpu_result =
            cartalith_climate::finish_weather_grid(&grid.eh, rain, grid.ww, grid.wh, grid.wrap_x, grid.sea, gw, gh);

        // The REAL comparison: the untouched CPU function, not a GPU-shaped
        // twin -- same discipline milestone 4 used.
        let cpu_result = cartalith_climate::simulate_weather(gw, gh, &field, iters, 0.0, &p);

        let mut max_abs_diff = 0.0f64;
        let mut mismatches = 0usize;
        // Measured, not guessed (same discipline as every tolerance in this
        // crate): real max_abs_diff observed here is ~1.8e-7 -- essentially
        // f32 machine epsilon, milestone 3's HEIGHT_TOLERANCE territory, not
        // milestone 2's WARP_TOLERANCE compounding-noise territory. Bounded,
        // non-chaotic arithmetic (gather/advect/deposit, no nested noise
        // evaluations) turns out not to compound meaningfully across 70
        // iterations, the same finding milestone 4 made for gauss_blur's
        // direct-sum-in-f32 vs running-sum-in-f64 gap. ~50x headroom over
        // the observed value.
        const WEATHER_TOLERANCE: f64 = 1e-5;
        for (i, (g, c)) in gpu_result.iter().zip(cpu_result.iter()).enumerate() {
            let d = ((*g as f64) - (*c as f64)).abs();
            if d > WEATHER_TOLERANCE {
                mismatches += 1;
                if mismatches <= 5 {
                    eprintln!("  mismatch at {i}: gpu={g} cpu={c} diff={d}");
                }
            }
            if d > max_abs_diff {
                max_abs_diff = d;
            }
        }
        eprintln!(
            "gpu_weather_loop vs REAL cartalith_climate::simulate_weather, {gw}x{gh} (coarse {}x{}), iters={iters}: {mismatches}/{} cells exceed tol={WEATHER_TOLERANCE}, max_abs_diff={max_abs_diff}",
            grid.ww, grid.wh, gpu_result.len()
        );
        assert_eq!(
            mismatches, 0,
            "gpu_weather_loop diverged from the REAL CPU simulate_weather beyond {WEATHER_TOLERANCE} -- see max_abs_diff above"
        );
    }

    #[test]
    fn gpu_weather_loop_real_timing() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        // A production-scale source map (2048x2048, this session's own
        // default resolution), NOT the tiny 10x8 correctness-test field --
        // `simulate_weather`'s coarse grid is capped at min(gw,240), so a
        // small source field would understate the real per-iteration
        // working-set size this kernel actually runs at in practice.
        let (gw, gh) = (2048usize, 2048usize);
        let field = synthetic_field(gw * gh, 41);
        let (_, _, _, p) = weather_test_field_and_params();
        let iters = 70;
        let grid = cartalith_climate::build_weather_grid(gw, gh, &field, 0.0, &p);

        let t0 = Instant::now();
        let _ = simulate_weather_loop_gpu_with(
            &gpu,
            &grid.eh,
            &grid.tc,
            &grid.sst_evap,
            &grid.wx,
            &grid.wy,
            &grid.w_init,
            grid.ww as u32,
            grid.wh as u32,
            iters,
            grid.sea as f32,
            grid.ocean_hum as f32,
            grid.evap as f32,
            grid.ocean as f32,
            grid.rain_k as f32,
            grid.dry as f32,
            grid.step as f32,
            grid.bulk_evap,
            grid.wrap_x,
        );
        let gpu_time = t0.elapsed();

        let t1 = Instant::now();
        let _ = cartalith_climate::simulate_weather(gw, gh, &field, iters, 0.0, &p);
        let cpu_time = t1.elapsed();

        // Real, honest finding: unlike every prior GPU milestone, this
        // kernel's own working grid (ww x wh) is capped at min(gw,240) --
        // for any square map at gw>=240 (every size this port's own
        // resolution presets offer, 512 through 8192), the coarse loop's
        // actual per-iteration work is CONSTANT regardless of map
        // resolution. Testing at 128/512/1024/2048 (this session's usual
        // sweep) would not show meaningfully different LOOP timing across
        // those sizes -- the coarse grid barely changes size once gw
        // exceeds 240 -- so this single real measurement at the actual
        // working size is the meaningful data point, not four repeats of
        // essentially the same number. `generate_terrain`'s own end-to-end
        // timing DOES still vary with gw/gh, via every other GPU-wired
        // stage plus finish_weather_grid's O(gw*gh) upsample, just not via
        // this kernel's own loop.
        eprintln!(
            "gpu_weather_loop coarse grid {}x{}, iters={iters} (source map {gw}x{gh}): GPU = {:?}, CPU (real) = {:?}, ratio (CPU/GPU) = {:.2}x",
            grid.ww,
            grid.wh,
            gpu_time,
            cpu_time,
            cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
        );
    }

    // ---- GPU_LAYER_INTEGRATION_SCOPE.md milestone 9: flow accumulation ----

    /// A smoothed height field with real drainage structure.
    /// [`synthetic_field`]'s raw per-cell hash is white noise: almost every
    /// cell is a local pit or one step from one, so flow paths are ~1 cell
    /// long and NEITHER the CPU function's descending walk NOR the GPU
    /// kernel's pointer doubling gets exercised at all. Blurring produces
    /// basins with flow paths hundreds of cells long -- the case that
    /// actually distinguishes the two algorithms.
    ///
    /// `fine_weight` sets how pitted the result is, and that turns out to
    /// be the variable that actually matters here: a rough field drains in
    /// short hops into thousands of local pits (short pointer chains, small
    /// accumulations), a smooth one drains across most of the grid into a
    /// handful of outlets (chains hundreds of cells long, accumulations
    /// four orders of magnitude larger, and correspondingly deeper `f32`
    /// summation chains on the CPU side). Both regimes are tested, because
    /// only the second one really stresses either algorithm.
    fn drainage_test_field(w: usize, h: usize, salt: u32, fine_weight: f32) -> Vec<f32> {
        let broad = cartalith_terrain::gauss_blur(&synthetic_field(w * h, salt), 64.0, w, h, false);
        let fine = cartalith_terrain::gauss_blur(&synthetic_field(w * h, salt ^ 0x9E37), 5.0, w, h, false);
        let mut f: Vec<f32> = broad.iter().zip(fine.iter()).map(|(&b, &s)| b + fine_weight * s).collect();
        let mn = f.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = f.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (mx - mn).max(1e-6);
        for v in f.iter_mut() {
            *v = (*v - mn) / range;
        }
        f
    }

    /// `compute_flow`'s D8 receiver choice, extracted as a reference so the
    /// GPU direction kernel can be checked against it *directly* rather
    /// than only through the accumulation. Deliberately a literal transcript
    /// of the CPU function's inner double loop (same visiting order, same
    /// `f64` arithmetic, same strict `>` first-max-wins tie-break) -- if it
    /// ever drifts from the real one, `gpu_flow_matches_real_cpu_compute_flow`
    /// (which uses the real function, not this) is what catches it.
    fn cpu_receivers(gw: usize, gh: usize, field: &[f32], world: bool) -> Vec<i32> {
        let mut d8 = [0f64; 9];
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
            }
        }
        (0..gw * gh)
            .map(|i| {
                let x = (i % gw) as i64;
                let y = (i / gw) as i64;
                let h = field[i] as f64;
                let mut best: i64 = -1;
                let mut best_drop = 0.0f64;
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let mut nx = x + dx;
                        let ny = y + dy;
                        if world {
                            nx = ((nx % gw as i64) + gw as i64) % gw as i64;
                        } else if nx < 0 || nx >= gw as i64 {
                            continue;
                        }
                        if ny < 0 || ny >= gh as i64 {
                            continue;
                        }
                        let j = ny * gw as i64 + nx;
                        let drop = (h - field[j as usize] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                        if drop > best_drop {
                            best_drop = drop;
                            best = j;
                        }
                    }
                }
                best as i32
            })
            .collect()
    }

    /// The flow-direction half of the redesign: embarrassingly parallel and
    /// a faithful transcript of the CPU inner loop, so this is where the
    /// two algorithms *could* agree exactly. The only thing standing in the
    /// way is that the CPU computes `drop` in `f64` and WGSL has no `f64`
    /// -- so a receiver flips only when two candidate neighbours' drops are
    /// within `f32` rounding of each other. This test measures how often
    /// that really happens rather than assuming it away.
    #[test]
    fn gpu_flow_directions_match_cpu_receivers() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let ctx = init_gpu_flow_with(&gpu);
        let (gw, gh) = (512usize, 512usize);

        for &(fine, world) in &[(0.25f32, false), (0.25, true), (0.02, false)] {
            let field = drainage_test_field(gw, gh, 11, fine);
            let out = dispatch_gpu_flow(&ctx, gw, gh, &field, None, false, world);
            let cpu = cpu_receivers(gw, gh, &field, world);
            let mismatches = out.recv.iter().zip(cpu.iter()).filter(|(a, b)| a != b).count();
            let pits = cpu.iter().filter(|&&r| r < 0).count();
            eprintln!(
                "gpu_flow directions {gw}x{gh} fine={fine} world={world}: {mismatches}/{} receivers differ from CPU ({pits} pits)",
                cpu.len()
            );
            assert_eq!(
                mismatches, 0,
                "flow direction is a pure per-cell function of the height field -- any mismatch is a real \
                 f32-vs-f64 near-tie, worth investigating rather than tolerating"
            );
        }
    }

    /// The headline correctness check: the GPU accumulation against the
    /// **real, untouched** `cartalith_hydrology::compute_flow`, not a
    /// GPU-shaped CPU twin (milestone 4's discipline). This is a
    /// different-algorithm comparison -- a global descending-height walk
    /// summing in `f32` versus `ceil(log2(n))` pointer-doubling rounds
    /// summing in `u32` fixed point -- so exact equality is not expected;
    /// what is expected, and asserted, is that the two agree to within the
    /// fixed-point quantization the Rust side deliberately chose.
    #[test]
    fn gpu_flow_matches_real_cpu_compute_flow() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let ctx = init_gpu_flow_with(&gpu);
        let (gw, gh) = (512usize, 512usize);
        let rain = synthetic_field(gw * gh, 23);

        // The third row is the long-chain regime: a nearly pit-free field
        // where a single outlet drains most of the grid, so the CPU's own
        // f32 accumulation chain is ~1e5 additions deep. That is where the
        // two algorithms' error behaviour actually differs.
        for &(use_rain, world, fine) in &[(false, false, 0.25f32), (true, false, 0.25), (true, true, 0.25), (false, false, 0.02), (true, false, 0.02)] {
            let field = drainage_test_field(gw, gh, 11, fine);
            let out =
                dispatch_gpu_flow(&ctx, gw, gh, &field, if use_rain { Some(&rain) } else { None }, use_rain, world);
            let cpu = cartalith_hydrology::compute_flow(
                gw,
                gh,
                &field,
                if use_rain { Some(&rain) } else { None },
                use_rain,
                world,
            );

            // The channel-initiation threshold splits the two error
            // regimes described on FLOW_TOLERANCE: at and above it, every
            // downstream consumer (channels, water access, route corridors,
            // flood) actually reads the value; below it, the accumulation is
            // a handful of individually-quantized seeds and nothing in the
            // pipeline distinguishes one from the next.
            let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, 1000.0);
            let mut max_abs = 0.0f64;
            let mut max_rel_any = 0.0f64;
            let mut max_rel_channel = 0.0f64;
            let mut over_tol = 0usize;
            let mut cpu_max = 0.0f64;
            for (g, c) in out.acc.iter().zip(cpu.iter()) {
                let (g, c) = (*g as f64, *c as f64);
                cpu_max = cpu_max.max(c);
                let abs = (g - c).abs();
                // Floored at 1.0 (one cell's worth of seed): below that a
                // "relative" error says more about the denominator than
                // about the algorithms.
                let rel = abs / c.abs().max(1.0);
                max_abs = max_abs.max(abs);
                max_rel_any = max_rel_any.max(rel);
                if c >= thresh {
                    max_rel_channel = max_rel_channel.max(rel);
                    if rel > FLOW_TOLERANCE {
                        over_tol += 1;
                    }
                }
            }
            eprintln!(
                "gpu_flow vs REAL compute_flow {gw}x{gh} use_rain={use_rain} world={world} fine={fine}: max_abs={max_abs:.6}, max_rel(all)={max_rel_any:.3e}, max_rel(>=thresh {thresh:.0})={max_rel_channel:.3e}, {over_tol} channel cells over tol={FLOW_TOLERANCE}, cpu_max_acc={cpu_max:.1}, fixed_point_step={:.3e}",
                1.0 / out.fixed_point_scale
            );
            assert_eq!(
                over_tol, 0,
                "GPU flow accumulation diverged from the real CPU compute_flow beyond tolerance where it matters"
            );
            assert!(
                max_rel_any <= FLOW_ANY_CELL_TOLERANCE,
                "even the sub-threshold cells moved more than quantization can explain: {max_rel_any:.3e}"
            );
        }
    }

    /// Fixed-point integer accumulation is exactly order-independent, so
    /// unlike a compare-exchange float-atomic emulation this kernel is
    /// bit-reproducible -- asserted, not assumed, because it is the whole
    /// reason the fixed-point choice was made.
    #[test]
    fn gpu_flow_is_bit_reproducible() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let ctx = init_gpu_flow_with(&gpu);
        let (gw, gh) = (256usize, 256usize);
        let field = drainage_test_field(gw, gh, 5, 0.05);
        let rain = synthetic_field(gw * gh, 6);
        let a = dispatch_gpu_flow(&ctx, gw, gh, &field, Some(&rain), true, false);
        let b = dispatch_gpu_flow(&ctx, gw, gh, &field, Some(&rain), true, false);
        assert_eq!(a.acc, b.acc, "GPU flow accumulation must be bit-identical run to run");
        assert_eq!(a.recv, b.recv, "GPU flow directions must be bit-identical run to run");
    }

    /// The measurement `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9 exists
    /// to produce: flow accumulation is not a leaf computation, it feeds
    /// the river network, so "how close is the number" matters less than
    /// "does the river network come out the same". Both accumulations are
    /// run through the real `build_channels`/`strahler_from_receivers` and
    /// the resulting channel masks compared cell for cell.
    #[test]
    fn gpu_flow_downstream_river_network_divergence() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let ctx = init_gpu_flow_with(&gpu);
        let (gw, gh) = (512usize, 512usize);
        let rain = synthetic_field(gw * gh, 23);
        let (sea, density, km) = (0.35f64, 1.0f64, 1000.0f64);
        for &fine in &[0.25f32, 0.02] {
        let field = drainage_test_field(gw, gh, 11, fine);

        let gpu_acc = dispatch_gpu_flow(&ctx, gw, gh, &field, Some(&rain), true, false).acc;
        let cpu_acc = cartalith_hydrology::compute_flow(gw, gh, &field, Some(&rain), true, false);

        let ch_gpu = cartalith_hydrology::build_channels(&field, &gpu_acc, gw, gh, sea, false, density, km);
        let ch_cpu = cartalith_hydrology::build_channels(&field, &cpu_acc, gw, gh, sea, false, density, km);
        let n_gpu = ch_gpu.chan.iter().filter(|&&c| c != 0).count();
        let n_cpu = ch_cpu.chan.iter().filter(|&&c| c != 0).count();
        let chan_diff = ch_gpu.chan.iter().zip(ch_cpu.chan.iter()).filter(|(a, b)| a != b).count();
        let recv_diff = ch_gpu.recv.iter().zip(ch_cpu.recv.iter()).filter(|(a, b)| a != b).count();

        let ord_gpu = cartalith_hydrology::strahler_from_receivers(&ch_gpu.recv, &gpu_acc, &ch_gpu.chan);
        let ord_cpu = cartalith_hydrology::strahler_from_receivers(&ch_cpu.recv, &cpu_acc, &ch_cpu.chan);
        let ord_diff = ord_gpu.iter().zip(ord_cpu.iter()).filter(|(a, b)| a != b).count();
        let max_gpu = ord_gpu.iter().copied().max().unwrap_or(0);
        let max_cpu = ord_cpu.iter().copied().max().unwrap_or(0);

        eprintln!(
            "gpu_flow downstream {gw}x{gh} fine={fine}: river cells GPU={n_gpu} CPU={n_cpu} (delta {}), \
             channel-mask cells differing={chan_diff}, channel receivers differing={recv_diff}, \
             Strahler order cells differing={ord_diff}, max order GPU={max_gpu} CPU={max_cpu}",
            n_gpu as i64 - n_cpu as i64
        );

        // A hard ceiling on how far the river network may move, so a future
        // regression (a wrong scale, a lost round, a broken tie-break) fails
        // here instead of quietly reshaping every map. Set well above the
        // real measured divergence -- see the milestone entry for the actual
        // numbers.
        let rel = (n_gpu as f64 - n_cpu as f64).abs() / (n_cpu.max(1) as f64);
        assert!(rel < 0.02, "river-cell count moved by {:.3}% -- far beyond quantization", rel * 100.0);
        assert_eq!(max_gpu, max_cpu, "the river network's maximum Strahler order must not change");
        }
    }

    #[test]
    fn gpu_flow_real_timing() {
        let Some(gpu) = init_gpu_shared_device().ok() else {
            eprintln!("no GPU available -- skipping");
            return;
        };
        let ctx = init_gpu_flow_with(&gpu);
        for &size in &[128usize, 512, 1024, 2048] {
            let field = drainage_test_field(size, size, 11, 0.02);
            let rain = synthetic_field(size * size, 23);

            let t0 = Instant::now();
            let out = dispatch_gpu_flow(&ctx, size, size, &field, Some(&rain), true, false);
            let gpu_time = t0.elapsed();

            let t1 = Instant::now();
            let _ = cartalith_hydrology::compute_flow(size, size, &field, Some(&rain), true, false);
            let cpu_time = t1.elapsed();

            eprintln!(
                "gpu_flow {size}x{size}: GPU = {:?} ({} doubling rounds), CPU (real) = {:?}, ratio (CPU/GPU) = {:.2}x",
                gpu_time,
                ((size * size) as f64).log2().ceil() as u32,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
            assert!(out.acc.iter().all(|v| v.is_finite()), "no NaN/Inf in the GPU accumulation");
        }
    }
}
