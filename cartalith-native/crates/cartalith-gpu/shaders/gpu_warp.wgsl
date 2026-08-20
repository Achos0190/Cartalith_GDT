// GPU-safe domain warp (GPU_LAYER_INTEGRATION_SCOPE.md milestone 2).
//
// Ports `compute_warp`'s NON-world (`world=false`) branch only --
// world-wrap (`pfbm`) is deliberately deferred, see the scope doc.
// Duplicates gpu_hash/gpu_vnoise from gpu_noise.wgsl (WGSL has no
// cross-file module include via include_str! here) -- mirror both copies
// operation-for-operation if either changes, matching that file's own
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
    _pad2: f32,
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

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.band_rows {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let xf = f32(gid.x) * params.wf;
    let yf = f32(gid.y + params.y_offset) * params.wf;

    let qx = gpu_fbm(xf, yf, params.seed + 17);
    let qy = gpu_fbm(xf, yf, params.seed + 101);
    let wx = gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 213) - 0.5;
    let wy = gpu_fbm(xf + 4.0 * qx, yf + 4.0 * qy, params.seed + 331) - 0.5;

    out_warp_x[idx] = wx * 2.0 * params.amp;
    out_warp_y[idx] = wy * 2.0 * params.amp;
}
