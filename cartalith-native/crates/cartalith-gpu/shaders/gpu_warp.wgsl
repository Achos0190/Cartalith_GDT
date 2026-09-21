// GPU-safe domain warp (GPU_LAYER_INTEGRATION_SCOPE.md milestone 2, world-wrap
// added per OUTSTANDING_WORK.md §2.9's "World-wrap support for the
// milestone 1-5 kernels" row).
//
// Ports `compute_warp`'s NON-world branch via gpu_fbm, and its world=true
// branch (period-wrapped `pfbm`, fixed p_x=3 like compute_warp's own
// hardcoded local) via gpu_pfbm below -- mirrors
// cartalith_noise::gpu_pfbm/gpu_pvnoise operation-for-operation. Duplicates
// gpu_hash/gpu_vnoise/gpu_pvnoise from gpu_noise.wgsl (WGSL has no
// cross-file module include via include_str! here) -- mirror all copies
// operation-for-operation if any changes, matching that file's own
// note about staying in lockstep with cartalith-noise's Rust side.

// `band_rows`/`y_offset` are the multi-GPU split-tiles addition
// (`HARDWARE_ACCELERATION.md`, 2026-08-20 section). The kernel writes a
// CONTIGUOUS ROW BAND of `band_rows` rows starting at world row
// `y_offset`, into a buffer sized `width * band_rows` -- so the output
// index is band-local while the noise coordinate stays world-absolute.
// `y_offset = 0, band_rows = height` is exactly the whole-grid case, and
// is bit-identical to the pre-split kernel: `f32(gid.y + 0u)` is
// `f32(gid.y)`, and `gid.y * width + gid.x` is unchanged. Every
// single-device caller passes those values, so the existing path's
// numbers do not move.
struct WarpParams {
    seed: i32,
    width: u32,
    height: u32,
    wf: f32,
    amp: f32,
    y_offset: u32,
    band_rows: u32,
    // Was `_pad2: f32` -- reused in place (same offset, same 32-byte total
    // size) rather than growing the struct, since a bool-shaped flag fits
    // the slot padding already reserved.
    world: u32,
}

@group(0) @binding(0) var<uniform> params: WarpParams;
@group(0) @binding(1) var<storage, read_write> out_warp_x: array<f32>;
@group(0) @binding(2) var<storage, read_write> out_warp_y: array<f32>;

fn pcg3d(v_in: vec3<u32>) -> vec3<u32> {
    var v = v_in * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u, 16u, 16u));
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return v;
}

fn gpu_hash(x: i32, y: i32, s: i32) -> u32 {
    let v = pcg3d(vec3<u32>(bitcast<u32>(x), bitcast<u32>(y), bitcast<u32>(s)));
    return v.x;
}

fn gpu_hash_to_unit_f32(h: u32) -> f32 {
    return f32(h) / 4294967295.0;
}

fn smoothstep_component(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

fn gpu_vnoise(x: f32, y: f32, s: i32) -> f32 {
    let xi = floor(x);
    let yi = floor(y);
    let xf = x - xi;
    let yf = y - yi;
    let u = smoothstep_component(xf);
    let v = smoothstep_component(yf);
    let xii = i32(xi);
    let yii = i32(yi);
    let a = gpu_hash_to_unit_f32(gpu_hash(xii, yii, s));
    let b = gpu_hash_to_unit_f32(gpu_hash(xii + 1, yii, s));
    let c = gpu_hash_to_unit_f32(gpu_hash(xii, yii + 1, s));
    let d = gpu_hash_to_unit_f32(gpu_hash(xii + 1, yii + 1, s));
    return a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v;
}

// Mirrors cartalith_noise::gpu_fbm exactly: 6 octaves, amp/freq
// halving/doubling, `s + o*131` per-octave seed offset.
fn gpu_fbm(x: f32, y: f32, s: i32) -> f32 {
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var sum: f32 = 0.0;
    var nrm: f32 = 0.0;
    for (var o: i32 = 0; o < 6; o = o + 1) {
        sum += amp * gpu_vnoise(x * freq, y * freq, s + o * 131);
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    return sum / nrm;
}

// `gpu_vnoise`'s periodic sibling -- mirrors cartalith_noise::gpu_pvnoise's
// x-lattice Euclidean-mod wrap exactly.
fn gpu_pvnoise(x: f32, y: f32, s: i32, p_x_in: i32) -> f32 {
    let xi = floor(x);
    let yi = floor(y);
    let xf = x - xi;
    let yf = y - yi;
    let u = smoothstep_component(xf);
    let v = smoothstep_component(yf);
    let xii = i32(xi);
    let yii = i32(yi);
    let p_x = max(p_x_in, 2);
    let px = ((xii % p_x) + p_x) % p_x;
    let px1 = (px + 1) % p_x;
    let a = gpu_hash_to_unit_f32(gpu_hash(px, yii, s));
    let b = gpu_hash_to_unit_f32(gpu_hash(px1, yii, s));
    let c = gpu_hash_to_unit_f32(gpu_hash(px, yii + 1, s));
    let d = gpu_hash_to_unit_f32(gpu_hash(px1, yii + 1, s));
    return a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v;
}

// Mirrors cartalith_noise::gpu_pfbm exactly: same octave loop as gpu_fbm,
// plus the period `p` doubling alongside `freq` each octave (floored at 2).
fn gpu_pfbm(x: f32, y: f32, s: i32, p_x_in: i32) -> f32 {
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var sum: f32 = 0.0;
    var nrm: f32 = 0.0;
    var p: i32 = max(p_x_in, 2);
    for (var o: i32 = 0; o < 6; o = o + 1) {
        sum += amp * gpu_pvnoise(x * freq, y * freq, s + o * 131, p);
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
        p = max(p * 2, 2);
    }
    return sum / nrm;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.band_rows {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let xf = f32(gid.x) * params.wf;
    let yf = f32(gid.y + params.y_offset) * params.wf;

    var qx: f32;
    var qy: f32;
    var wx: f32;
    var wy: f32;
    // Fixed p_x=3, matching `compute_warp`'s own hardcoded local of the
    // same name -- warp's world-wrap period is not derived from grid
    // width, unlike heterogeneity's (see gpu_heterogeneity.wgsl).
    if params.world != 0u {
        qx = gpu_pfbm(xf, yf, params.seed + 17, 3);
        qy = gpu_pfbm(xf, yf, params.seed + 101, 3);
        wx = gpu_pfbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 213, 3) - 0.5;
        wy = gpu_pfbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 331, 3) - 0.5;
    } else {
        qx = gpu_fbm(xf, yf, params.seed + 17);
        qy = gpu_fbm(xf, yf, params.seed + 101);
        wx = gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 213) - 0.5;
        wy = gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 331) - 0.5;
    }

    out_warp_x[idx] = wx * 2.0 * params.amp;
    out_warp_y[idx] = wy * 2.0 * params.amp;
}
