// GPU build_carrying_capacity (OUTSTANDING_WORK.md §2.6 "Phase 2 per-cell
// affordance fields"). Port of `cartalith_civ::build_carrying_capacity`:
// K = clamp(soil * exp(-(t-18)^2/800) * (0.25 + 0.75*water) * bM, 0, 1),
// zero below sea level and on biome 0 (ocean).
//
// The CPU computes in f64 and rounds once to f32; this computes in f32, so
// values differ by a few f32 ulps (DECISIONS.md §7a principled equivalence;
// the measured bound is `affordance::CARRYING_TOLERANCE`). The one BRANCH
// with a float input -- `field < sea` -- uses a host-computed cut so it takes
// the CPU's branch on every cell (see gpu_biome.wgsl's header).
//
// `biome`/`wet` are u8 fields packed four per u32. `has_wet == 0` is the
// CPU's `wet_mask: None`.

struct CarryParams {
    n: u32,
    row_threads: u32,
    has_wet: u32,
    _pad0: u32,
    sea_cut: f32,   // field < sea_cut  <=>  (field as f64) < sea
    biome_k: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> params: CarryParams;
@group(0) @binding(1) var<storage, read> soil: array<f32>;
@group(0) @binding(2) var<storage, read> water: array<f32>;
@group(0) @binding(3) var<storage, read> biome: array<u32>;
@group(0) @binding(4) var<storage, read> temp: array<f32>;
@group(0) @binding(5) var<storage, read> field: array<f32>;
@group(0) @binding(6) var<storage, read> wet: array<u32>;
@group(0) @binding(7) var<storage, read_write> out_k: array<f32>;

// cartalith_civ::BIOME_DENSITY_RESIDUAL, indexed by biome - 1.
const RESID = array<f32, 13>(0.60, 0.65, 0.85, 0.85, 1.00, 0.90, 0.90, 0.95, 0.55, 0.80, 0.75, 0.55, 0.00);

fn byte_at(i: u32, which: u32) -> u32 {
    var w = 0u;
    if which == 0u { w = biome[i >> 2u]; } else { w = wet[i >> 2u]; }
    return (w >> (8u * (i & 3u))) & 0xffu;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.y * params.row_threads + gid.x;
    if i >= params.n {
        return;
    }
    if field[i] < params.sea_cut {
        out_k[i] = 0.0;
        return;
    }
    let b = byte_at(i, 0u);
    if b == 0u {
        out_k[i] = 0.0;
        return;
    }
    let t = temp[i];
    let dt = t - 18.0;
    let t_f = exp(-(dt * dt) / 800.0);
    let w_mod = 0.25 + 0.75 * water[i];
    var resid = 0.9; // biome_density_residual's out-of-table fallback
    if b <= 13u {
        var tbl = RESID; // a `var` so the runtime index is legal on every backend
        resid = tbl[b - 1u];
    }
    if params.has_wet != 0u && byte_at(i, 1u) != 0u {
        resid = 0.70; // WETLAND_DENSITY_RESIDUAL
    }
    var b_m = 1.0;
    if params.biome_k != 0.0 {
        b_m = 1.0 - params.biome_k + params.biome_k * resid;
    }
    out_k[i] = clamp(soil[i] * t_f * w_mod * b_m, 0.0, 1.0);
}
