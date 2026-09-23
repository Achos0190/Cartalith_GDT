//! Phase 2 per-cell affordance fields on GPU (`OUTSTANDING_WORK.md` §2.6,
//! `GPU_LAYER_INTEGRATION_SCOPE.md`'s feasibility table: "directly comparable
//! to climate/erosion's per-cell case").
//!
//! Four `cartalith-civ` builders, and they do **not** port identically:
//!
//! | CPU function | Shape | Here |
//! |---|---|---|
//! | `build_biome_raster` | pure per-cell | [`biome_raster_grid_gpu_with`], **bit-exact** |
//! | `build_carrying_capacity` | pure per-cell | [`carrying_capacity_grid_gpu_with`] |
//! | `build_resource_potentials` | per-cell kernel **between three whole-raster steps** (chamfer distance, a flow max, a rank-threshold scarcity cut) | [`resource_potentials_grid_gpu_with`] runs the kernel only; the three whole-raster steps stay on the CPU (`cartalith_civ::resource_copper_dist` / `resource_flow_max` / `finish_resource_potentials`) |
//! | `build_settlement_suitability` | per-cell plus a fixed `(2*lake_r+1)^2` neighbourhood | [`settlement_suitability_grid_gpu_with`], full-context case only |
//!
//! Sequencing is the CPU's: biome feeds carrying capacity and resources;
//! carrying capacity and the *scarcity-cut* resources feed suitability. Each
//! is its own dispatch with its own readback, because the scarcity cut
//! between resources and suitability is a CPU sort.
//!
//! **Correctness bar** (`DECISIONS.md` §7a). Every *branch* on a float input
//! takes the CPU's side on every cell: the CPU compares `f32 as f64` (or a
//! monotone f64 function of one f32 input) against an f64 constant, and
//! [`first_true`] finds the f32 cut at which that comparison flips, so the
//! shader's `x < cut` is the same predicate. What remains is continuous
//! arithmetic done in f32 where the CPU used f64 and rounded once -- a few f32
//! ulps, bounded by the measured tolerances below.
//!
//! No `cartalith-civ` dependency (it would be a cycle: civ -> engine -> gpu),
//! so the per-cell formulas are restated in the shaders, exactly as
//! `gpu_resistance.wgsl` restates `compute_resistance`'s. The CPU-vs-GPU
//! comparison in `examples/affordance_gpu_compare.rs` runs against the real
//! `cartalith-civ` functions, which is what catches a drifted constant.

use wgpu::util::DeviceExt;

use super::{
    build_pipeline_shared, device_grid_limit_bytes, device_is_unusable, read_back_vec, storage_entry,
    uniform_entry, GpuContext, GpuDevice,
};

const SHADER_BIOME: &str = include_str!("../shaders/gpu_biome.wgsl");
const SHADER_CARRYING: &str = include_str!("../shaders/gpu_carrying.wgsl");
const SHADER_RESOURCES: &str = include_str!("../shaders/gpu_resources.wgsl");
const SHADER_SUITABILITY: &str = include_str!("../shaders/gpu_suitability.wgsl");

/// Max |GPU - CPU| allowed for carrying capacity. **Measured 2026-09-23**
/// (`examples/affordance_gpu_compare.rs`, seed 12345, AMD Radeon RX 7800 XT /
/// Vulkan): worst cell `5.96e-8` at every size from 512² to 4096², both
/// `biome_k` settings -- one f32 ulp near 0.5. Set ~3x above that.
pub const CARRYING_TOLERANCE: f64 = 2e-7;
/// Max |GPU - CPU| allowed over all fifteen resource fields. Measured the same
/// way: worst cell `1.19e-7` (the `clay` field, whose `ln` ratio is the only
/// transcendental in the kernel besides copper's `exp`), identical at every
/// size, both before the scarcity cut and after it -- the cut kept and dropped
/// exactly the same cells on every world measured. Set ~4x above.
pub const RESOURCES_TOLERANCE: f64 = 5e-7;
/// Max |GPU - CPU| allowed for settlement suitability. Measured the same way:
/// worst cell `2.38e-7` at every size, whether the GPU kernel is fed the CPU's
/// inputs or the GPU chain's own. Set ~4x above.
pub const SUITABILITY_TOLERANCE: f64 = 1e-6;

const BIOME_LAYOUT: [wgpu::BindGroupLayoutEntry; 5] =
    [uniform_entry(0), storage_entry(1, true), storage_entry(2, true), storage_entry(3, true), storage_entry(4, false)];
/// 7 storage buffers -- under the shared device's 8
/// (`REUSED_STAGE_MAX_STORAGE_BUFFERS`), so no device-limit change.
const CARRYING_LAYOUT: [wgpu::BindGroupLayoutEntry; 8] = [
    uniform_entry(0),
    storage_entry(1, true),
    storage_entry(2, true),
    storage_entry(3, true),
    storage_entry(4, true),
    storage_entry(5, true),
    storage_entry(6, true),
    storage_entry(7, false),
];
/// Planar: one f32 input buffer, one packed-u8 input buffer, one output.
/// Resources has 10 inputs and 15 outputs and suitability 23 inputs, far past
/// 8 bindings, so both concatenate their planes into one binding each.
pub(crate) const PLANAR_LAYOUT: [wgpu::BindGroupLayoutEntry; 4] =
    [uniform_entry(0), storage_entry(1, true), storage_entry(2, true), storage_entry(3, false)];

// -- exact branch cuts ----------------------------------------------------------

/// Total-order key of a non-NaN f32: increasing with the value, `-0.0` just
/// below `+0.0`.
fn key(x: f32) -> u32 {
    let b = x.to_bits();
    if b >> 31 == 1 { !b } else { b | 0x8000_0000 }
}

fn unkey(k: u32) -> f32 {
    f32::from_bits(if k >> 31 == 1 { k & 0x7fff_ffff } else { !k })
}

/// The smallest f32 (in `-inf..=+inf`) for which `pred` holds, where `pred`
/// is monotone (false, then true) over the f32 line. `+inf` when it never
/// holds on a finite value.
///
/// So for a CPU predicate `g(x as f64) < t` with `g` non-decreasing, the cut
/// `c = first_true(|x| !(g(x) < t))` makes `x < c` the same predicate for
/// every non-NaN f32 `x`; and for `g(x) > t`, `c = first_true(|x| g(x) > t)`
/// makes it `x >= c`. `±0` never straddles a cut: every `g` used here treats
/// them identically, as IEEE comparison and subtraction do.
fn first_true(pred: impl Fn(f32) -> bool) -> f32 {
    let (mut lo, mut hi) = (key(f32::NEG_INFINITY), key(f32::INFINITY));
    if pred(f32::NEG_INFINITY) {
        return f32::NEG_INFINITY;
    }
    if !pred(f32::INFINITY) {
        return f32::INFINITY;
    }
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if pred(unkey(mid)) { hi = mid } else { lo = mid }
    }
    unkey(hi)
}

/// Cut for `(x as f64) < c`: the shader tests `x < lt(c)`.
fn lt(c: f64) -> f32 {
    first_true(|x| (x as f64) >= c)
}

/// Cut for `(x as f64) > c`: the shader tests `x >= gt(c)`.
fn gt(c: f64) -> f32 {
    first_true(|x| (x as f64) > c)
}

// -- dispatch plumbing --------------------------------------------------------

/// Workgroups of 64 along x, at most 256 across, the rest down y -- a 1D
/// launch that stays under the 65 535-per-dimension limit at every grid size.
/// Returns `(groups_x, groups_y, threads_per_row)`.
fn lin_dispatch(items: u32) -> (u32, u32, u32) {
    let groups = items.div_ceil(64).max(1);
    let gx = groups.min(256);
    (gx, groups.div_ceil(gx), gx * 64)
}

fn words(n: usize) -> usize {
    n.div_ceil(4)
}

/// A u8 field as u32 words, four cells per word, little-endian -- the byte
/// order every shader here unpacks with `(w >> 8*(i&3)) & 0xff`.
fn pack_u8(src: &[u8]) -> Vec<u32> {
    let mut out = vec![0u32; words(src.len())];
    bytemuck::cast_slice_mut::<u32, u8>(&mut out)[..src.len()].copy_from_slice(src);
    out
}

fn storage_init(ctx: &GpuContext, label: &str, bytes: &[u8]) -> wgpu::Buffer {
    ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes,
        usage: wgpu::BufferUsages::STORAGE,
    })
}

/// One storage buffer holding `planes.len()` planes of `plane_bytes` each,
/// written plane by plane (no host-side concatenation). `None` planes stay
/// zero -- `wgpu` zero-initialises new buffers.
fn planar_buffer(ctx: &GpuContext, label: &str, plane_bytes: u64, planes: &[Option<&[u8]>]) -> wgpu::Buffer {
    let buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: plane_bytes * planes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for (k, p) in planes.iter().enumerate() {
        if let Some(bytes) = p {
            ctx.queue.write_buffer(&buf, k as u64 * plane_bytes, bytes);
        }
    }
    buf
}

fn out_buffer(ctx: &GpuContext, label: &str, bytes: u64) -> wgpu::Buffer {
    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

fn staging(ctx: &GpuContext, bytes: u64) -> wgpu::Buffer {
    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("affordance staging"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Encode one compute pass over `items` invocations and, if `copy` is given,
/// the copy of `(src, bytes)` into `dst` after it; submit.
fn run(
    ctx: &GpuContext,
    params: &wgpu::Buffer,
    storage: &[&wgpu::Buffer],
    items: u32,
    copy: Option<(&wgpu::Buffer, &wgpu::Buffer, u64)>,
) {
    let mut entries = vec![wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() }];
    for (k, b) in storage.iter().enumerate() {
        entries.push(wgpu::BindGroupEntry { binding: k as u32 + 1, resource: b.as_entire_binding() });
    }
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("affordance bind group"),
        layout: &ctx.bind_group_layout,
        entries: &entries,
    });
    let (gx, gy, _) = lin_dispatch(items);
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("affordance") });
    {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("affordance pass"), timestamp_writes: None });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(gx, gy, 1);
    }
    if let Some((src, dst, bytes)) = copy {
        encoder.copy_buffer_to_buffer(src, 0, dst, 0, bytes);
    }
    ctx.queue.submit(Some(encoder.finish()));
}

fn uniform<T: bytemuck::Pod>(ctx: &GpuContext, v: &T) -> wgpu::Buffer {
    ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("affordance params"),
        contents: bytemuck::bytes_of(v),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

/// [`super::on_grid`]'s gate, plus the one it cannot know: this stage's
/// largest single binding (a planar buffer is several grids in one) must fit
/// the device's binding limit. Over it, `create_bind_group` raises a `wgpu`
/// validation error -- a panic -- so the answer is CPU, not an attempt.
fn gate<T>(gpu: &GpuDevice, cells: usize, largest_binding: u64, dispatch: impl FnOnce() -> Option<T>) -> Option<T> {
    if cells == 0 || device_is_unusable(gpu, cells as u64) || largest_binding > device_grid_limit_bytes(gpu) {
        return None;
    }
    dispatch()
}

// -- biome --------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BiomeParams {
    n: u32,
    words: u32,
    row_threads: u32,
    _pad0: u32,
    m: [f32; 6],
    _pad1: [f32; 2],
}

/// GPU sibling of `cartalith_civ::build_biome_raster`. **Bit-identical**
/// output: pure integer class selection, every float comparison through an
/// exact cut. `None` on any device refusal or readback failure.
///
/// **Not called by `compute_civilisation`, on measurement.** The CPU function
/// is a rayon pass over three fields with no arithmetic to speak of, and
/// uploading those fields costs more than computing it: 2048² CPU 1.0 ms
/// (1.0..1.2) against GPU 6.0 ms (6.0..7.2), 4096² 3.7 ms against 42.1 ms
/// (`examples/affordance_gpu_compare.rs`, 2026-09-23, RX 7800 XT). Kept
/// because it is the verified port, and because on a device where the fields
/// already live in GPU memory the answer would be different.
pub fn biome_raster_grid_gpu_with(gpu: &GpuDevice, water_bodies: &[u8], temp: &[f32], rain: &[f32]) -> Option<Vec<u8>> {
    let n = water_bodies.len();
    assert_eq!(temp.len(), n);
    assert_eq!(rain.len(), n);
    gate(gpu, n, (n * 4) as u64, || {
        let ctx = build_pipeline_shared(gpu, SHADER_BIOME, "gpu_biome", &BIOME_LAYOUT);
        let w = words(n);
        let (_, _, row_threads) = lin_dispatch(w as u32);
        let params = BiomeParams {
            n: n as u32,
            words: w as u32,
            row_threads,
            _pad0: 0,
            // classify_biome's moisture thresholds, in the shader's order.
            m: [lt(0.20), lt(0.30), lt(0.60), lt(0.12), lt(0.28), lt(0.55)],
            _pad1: [0.0; 2],
        };
        let pbuf = uniform(&ctx, &params);
        let wb = storage_init(&ctx, "biome wb", bytemuck::cast_slice(&pack_u8(water_bodies)));
        let tb = storage_init(&ctx, "biome temp", bytemuck::cast_slice(temp));
        let rb = storage_init(&ctx, "biome rain", bytemuck::cast_slice(rain));
        let out_bytes = (w * 4) as u64;
        let out = out_buffer(&ctx, "biome out", out_bytes);
        let stage = staging(&ctx, out_bytes);
        run(&ctx, &pbuf, &[&wb, &tb, &rb, &out], w as u32, Some((&out, &stage, out_bytes)));
        let words: Vec<u32> = read_back_vec(&ctx, &stage, n as u64)?;
        Some(bytemuck::cast_slice::<u32, u8>(&words)[..n].to_vec())
    })
}

// -- carrying capacity --------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CarryParams {
    n: u32,
    row_threads: u32,
    has_wet: u32,
    _pad0: u32,
    sea_cut: f32,
    biome_k: f32,
    _pad1: [f32; 2],
}

/// GPU sibling of `cartalith_civ::build_carrying_capacity` with `biome`
/// present -- the only shape `compute_civilisation` calls it with. Within
/// [`CARRYING_TOLERANCE`] of the CPU.
///
/// **Not called by `compute_civilisation`, on measurement** -- the same
/// upload-bound result as [`biome_raster_grid_gpu_with`]: 2048² CPU 2.7 ms
/// (2.6..2.8) against GPU 10.6 ms (10.2..10.9), 4096² 10.3 ms against 82.2 ms.
#[allow(clippy::too_many_arguments)]
pub fn carrying_capacity_grid_gpu_with(
    gpu: &GpuDevice,
    soil: &[f32],
    water: &[f32],
    biome: &[u8],
    temp: &[f32],
    field: &[f32],
    sea: f64,
    biome_k: f64,
    wet_mask: Option<&[u8]>,
) -> Option<Vec<f32>> {
    let n = field.len();
    for len in [soil.len(), water.len(), biome.len(), temp.len()] {
        assert_eq!(len, n);
    }
    gate(gpu, n, (n * 4) as u64, || {
        let ctx = build_pipeline_shared(gpu, SHADER_CARRYING, "gpu_carrying", &CARRYING_LAYOUT);
        let (_, _, row_threads) = lin_dispatch(n as u32);
        let params = CarryParams {
            n: n as u32,
            row_threads,
            has_wet: u32::from(wet_mask.is_some()),
            _pad0: 0,
            sea_cut: lt(sea),
            biome_k: biome_k as f32,
            _pad1: [0.0; 2],
        };
        let pbuf = uniform(&ctx, &params);
        let sb = storage_init(&ctx, "carry soil", bytemuck::cast_slice(soil));
        let wab = storage_init(&ctx, "carry water", bytemuck::cast_slice(water));
        let bb = storage_init(&ctx, "carry biome", bytemuck::cast_slice(&pack_u8(biome)));
        let tb = storage_init(&ctx, "carry temp", bytemuck::cast_slice(temp));
        let fb = storage_init(&ctx, "carry field", bytemuck::cast_slice(field));
        // A binding cannot be empty; with no mask the shader never reads it.
        let wet = wet_mask.map_or_else(|| vec![0u32; 1], pack_u8);
        let wetb = storage_init(&ctx, "carry wet", bytemuck::cast_slice(&wet));
        let out_bytes = (n * 4) as u64;
        let out = out_buffer(&ctx, "carry out", out_bytes);
        let stage = staging(&ctx, out_bytes);
        run(&ctx, &pbuf, &[&sb, &wab, &bb, &tb, &fb, &wetb, &out], n as u32, Some((&out, &stage, out_bytes)));
        read_back_vec(&ctx, &stage, n as u64)
    })
}

// -- resource potentials ------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ResParams {
    n: u32,
    words: u32,
    row_threads: u32,
    _pad0: u32,
    cu_lam: f32,
    ln_den: f32,
    cuts: [f32; 14],
}

/// `build_resource_potentials`' per-cell inputs. `cu_dist` and `flow_max`
/// are its two whole-raster pre-steps, computed by the caller with
/// `cartalith_civ::resource_copper_dist` / `resource_flow_max`.
pub struct ResourceGpuInputs<'a> {
    pub lith: &'a [u8],
    pub boundary_type: Option<&'a [u8]>,
    pub shear_field: Option<&'a [f32]>,
    pub flow: Option<&'a [f32]>,
    pub biome: Option<&'a [u8]>,
    pub field: &'a [f32],
    pub rain: &'a [f32],
    pub age: &'a [f32],
    pub volcanic: Option<&'a [f32]>,
    pub cu_dist: &'a [f32],
    pub flow_max: f64,
    pub gw: usize,
    pub sea: f64,
}

/// GPU sibling of `build_resource_potentials`' per-cell kernel: the fifteen
/// fields in `ResourcePotentials` order, **before** the scarcity cut (pass
/// them to `cartalith_civ::finish_resource_potentials`). Within
/// [`RESOURCES_TOLERANCE`] of the CPU kernel.
///
/// Read back one plane at a time through one grid-sized staging buffer, so the
/// host never holds the 60 B/cell transient `MEMORY_OPTIMIZATION_SCOPE.md` R3
/// removed from the CPU path. The output buffer itself is 15 grids of device
/// memory for the length of the call.
pub fn resource_potentials_grid_gpu_with(gpu: &GpuDevice, inp: &ResourceGpuInputs) -> Option<[Vec<f32>; 15]> {
    let n = inp.field.len();
    for len in [inp.lith.len(), inp.rain.len(), inp.age.len(), inp.cu_dist.len()] {
        assert_eq!(len, n);
    }
    let plane = (n * 4) as u64;
    gate(gpu, n, plane * 15, || {
        let ctx = build_pipeline_shared(gpu, SHADER_RESOURCES, "gpu_resources", &PLANAR_LAYOUT);
        let w = words(n);
        let (_, _, row_threads) = lin_dispatch(n as u32);
        let sea = inp.sea;
        let denom = (1.0 - sea).max(1e-6);
        // `r = ((field - sea) / denom).max(0.0)`, exactly as the CPU writes it.
        let r = move |f: f32| ((f as f64 - sea) / denom).max(0.0);
        let r_lt = |t: f64| first_true(|f| r(f) >= t);
        let params = ResParams {
            n: n as u32,
            words: w as u32,
            row_threads,
            _pad0: 0,
            cu_lam: (inp.gw as f64 / 24.0).max(3.0) as f32,
            ln_den: (1.0 + inp.flow_max * 0.05).ln() as f32,
            cuts: [
                gt(0.60), // age_old
                gt(0.55),
                lt(0.22),
                lt(0.12),
                gt(0.5),
                lt(0.30),
                gt(0.25), // shear
                gt(0.30),
                gt(0.45), // volcanic
                gt(0.35),
                gt(0.30),
                r_lt(0.25),
                r_lt(0.12),
                r_lt(0.35),
            ],
        };
        let pbuf = uniform(&ctx, &params);
        fn f32b(s: Option<&[f32]>) -> Option<&[u8]> {
            s.map(bytemuck::cast_slice)
        }
        let fin = planar_buffer(
            &ctx,
            "res fin",
            plane,
            &[
                f32b(inp.shear_field),
                f32b(inp.flow),
                f32b(Some(inp.field)),
                f32b(Some(inp.rain)),
                f32b(Some(inp.age)),
                f32b(inp.volcanic),
                f32b(Some(inp.cu_dist)),
            ],
        );
        let packed: [Option<Vec<u32>>; 3] = [Some(pack_u8(inp.lith)), inp.boundary_type.map(pack_u8), inp.biome.map(pack_u8)];
        let byte_planes: Vec<Option<&[u8]>> = packed.iter().map(|p| p.as_deref().map(bytemuck::cast_slice)).collect();
        let bytes = planar_buffer(&ctx, "res bytes", (w * 4) as u64, &byte_planes);
        let out = out_buffer(&ctx, "res out", plane * 15);
        run(&ctx, &pbuf, &[&fin, &bytes, &out], n as u32, None);

        let stage = staging(&ctx, plane);
        let mut fields: [Vec<f32>; 15] = Default::default();
        for (k, dst) in fields.iter_mut().enumerate() {
            let mut enc = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("res readback") });
            enc.copy_buffer_to_buffer(&out, k as u64 * plane, &stage, 0, plane);
            ctx.queue.submit(Some(enc.finish()));
            *dst = read_back_vec(&ctx, &stage, n as u64)?;
        }
        Some(fields)
    })
}

// -- settlement suitability ---------------------------------------------------

/// `build_settlement_suitability`'s full-context weights -- the
/// `SUIT_W_FULL_*` constants and `ISLET_KNEE`, handed in by the caller so they
/// have one definition, in `cartalith-civ`.
pub struct SuitabilityWeights {
    pub k: f64,
    pub w: f64,
    pub a: f64,
    pub d: f64,
    pub agri: f64,
    pub build: f64,
    pub coast: f64,
    pub river: f64,
    pub lake: f64,
    pub mineral: f64,
    pub corridor: f64,
    pub flood: f64,
    pub islet: f64,
    pub islet_knee: f64,
}

/// Every input `build_settlement_suitability` reads when its `ctx` is fully
/// populated. `resources` are the nine `SUIT_RESOURCE_KEYS` fields in that
/// order (copper, tin, iron, gold, salt, timber, lead, silver, gems), after
/// the scarcity cut.
pub struct SuitabilityGpuInputs<'a> {
    pub soil: &'a [f32],
    pub water: &'a [f32],
    pub carrying_cap: &'a [f32],
    pub field: &'a [f32],
    pub slope_n: &'a [f32],
    pub water_bodies: &'a [u8],
    pub corridor: &'a [f32],
    pub landmass: &'a [f32],
    pub flow: &'a [f32],
    pub river_reach: &'a [f32],
    pub coast_reach: &'a [f32],
    pub resources: [&'a [f32]; 9],
    pub rain: &'a [f32],
    pub flood: &'a [f32],
    pub slope_raw: &'a [f32],
    pub flow_thresh: f64,
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SuitParams {
    n: u32,
    gw: u32,
    gh: u32,
    row_threads: u32,
    lake_r: i32,
    _pad0: u32,
    sea: f32,
    denom: f32,
    sea_cut: f32,
    estuary_cut: f32,
    rr_lt030: f32,
    rr_lt060: f32,
    rr_lt085: f32,
    gw_f: f32,
    islet_knee: f32,
    w: [f32; 13],
}

/// GPU sibling of `cartalith_civ::build_settlement_suitability` with a
/// fully-populated `SuitabilityCtx`. Within [`SUITABILITY_TOLERANCE`] of the
/// CPU.
pub fn settlement_suitability_grid_gpu_with(
    gpu: &GpuDevice,
    inp: &SuitabilityGpuInputs,
    wt: &SuitabilityWeights,
) -> Option<Vec<f32>> {
    let (gw, gh) = (inp.gw, inp.gh);
    let n = gw * gh;
    let f32_planes: [&[f32]; 22] = [
        inp.soil,
        inp.water,
        inp.carrying_cap,
        inp.field,
        inp.slope_n,
        inp.coast_reach,
        inp.flow,
        inp.river_reach,
        inp.rain,
        inp.flood,
        inp.slope_raw,
        inp.corridor,
        inp.landmass,
        inp.resources[0],
        inp.resources[1],
        inp.resources[2],
        inp.resources[3],
        inp.resources[4],
        inp.resources[5],
        inp.resources[6],
        inp.resources[7],
        inp.resources[8],
    ];
    assert_eq!(inp.water_bodies.len(), n);
    for p in &f32_planes {
        assert_eq!(p.len(), n);
    }
    let plane = (n * 4) as u64;
    gate(gpu, n, plane * 22, || {
        let ctx = build_pipeline_shared(gpu, SHADER_SUITABILITY, "gpu_suitability", &PLANAR_LAYOUT);
        let (_, _, row_threads) = lin_dispatch(n as u32);
        let sea = inp.sea;
        let f = |x: f64| x as f32;
        let params = SuitParams {
            n: n as u32,
            gw: gw as u32,
            gh: gh as u32,
            row_threads,
            lake_r: ((gw as f64 / 170.0).round() as i32).max(2),
            _pad0: 0,
            sea: f(sea),
            denom: f((1.0 - sea).max(1e-6)),
            sea_cut: lt(sea),
            estuary_cut: gt(inp.flow_thresh * 3.0),
            rr_lt030: lt(0.30),
            rr_lt060: lt(0.60),
            rr_lt085: lt(0.85),
            gw_f: f(gw as f64),
            islet_knee: f(wt.islet_knee),
            w: [
                wt.k, wt.w, wt.a, wt.d, wt.agri, wt.build, wt.coast, wt.river, wt.lake, wt.mineral, wt.corridor, wt.flood,
                wt.islet,
            ]
            .map(f),
        };
        let pbuf = uniform(&ctx, &params);
        let planes: Vec<Option<&[u8]>> = f32_planes.iter().map(|p| Some(bytemuck::cast_slice::<f32, u8>(p))).collect();
        let fin = planar_buffer(&ctx, "suit fin", plane, &planes);
        let wb = storage_init(&ctx, "suit wb", bytemuck::cast_slice(&pack_u8(inp.water_bodies)));
        let out = out_buffer(&ctx, "suit out", plane);
        let stage = staging(&ctx, plane);
        run(&ctx, &pbuf, &[&fin, &wb, &out], n as u32, Some((&out, &stage, plane)));
        read_back_vec(&ctx, &stage, n as u64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cut is exact at the f32 on each side of every threshold this
    /// module uses -- the property the whole "branches are the CPU's" claim
    /// rests on. Checked by brute force over the 64 f32 neighbours of each.
    #[test]
    fn cuts_reproduce_f64_comparisons_exactly() {
        for c in [0.20, 0.30, 0.60, 0.12, 0.28, 0.55, 0.22, 0.5, 0.25, 0.45, 0.35, 0.85, 0.4, -7.0, 1e-9] {
            let (l, g) = (lt(c), gt(c));
            let mut x = (c as f32).next_down();
            for _ in 0..32 {
                x = x.next_down();
            }
            for _ in 0..64 {
                assert_eq!(x < l, (x as f64) < c, "lt cut for {c} at {x:e}");
                assert_eq!(x >= g, (x as f64) > c, "gt cut for {c} at {x:e}");
                x = x.next_up();
            }
        }
    }

    /// A derived monotone predicate (the resource kernel's elevation
    /// fraction) reduces to a cut on the raw input.
    #[test]
    fn derived_cut_matches_elevation_fraction() {
        let sea = 0.4137_f64;
        let denom = (1.0 - sea).max(1e-6);
        let r = |f: f32| ((f as f64 - sea) / denom).max(0.0);
        let cut = first_true(|f| r(f) >= 0.25);
        let mut x = cut;
        for _ in 0..64 {
            x = x.next_down();
        }
        for _ in 0..128 {
            assert_eq!(x < cut, r(x) < 0.25, "at {x:e}");
            x = x.next_up();
        }
        // and the cut is not vacuous: it sits inside (sea, 1).
        assert!((cut as f64) > sea && cut < 1.0);
    }

    #[test]
    fn pack_u8_is_little_endian_four_per_word() {
        let w = pack_u8(&[1, 2, 3, 4, 5]);
        assert_eq!(w, vec![0x0403_0201, 0x0000_0005]);
    }
}
