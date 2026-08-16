// GPU-safe crustal heterogeneity (GPU_LAYER_INTEGRATION_SCOPE.md milestone 2).
//
// Ports `compute_heterogeneity`'s per-cell body only -- the global
// max-reduce normalize pass at the end is done on CPU after readback
// (a standard O(n) linear scan, not worth a second GPU kernel for this
// milestone; the per-cell noise+age evaluation is the actual workload
// being GPU-accelerated). NON-world (`world=false`, `fbm` not `pfbm`)
// branch only, matching gpu_warp.wgsl's own scope. Duplicates
// gpu_hash/gpu_vnoise/gpu_fbm from gpu_warp.wgsl/gpu_noise.wgsl -- mirror
// all three copies operation-for-operation if any changes.
//
// `age` is a required input (matches `compute_heterogeneity`'s `age: &[f32]`,
// not optional). `warp_x`/`warp_y` are always-bound storage buffers here
// (GPU shaders can't express `Option<&[f32]>`) -- the caller passes a
// zero-filled buffer for the no-warp case, which reproduces
// `warp_x.map_or(0.0, ...)`'s CPU behaviour exactly (adding zero is a no-op).

struct HeteroParams {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32, // hf / gw, matches compute_heterogeneity's `hf / gw as f64` factor
}

@group(0) @binding(0) var<uniform> params: HeteroParams;
@group(0) @binding(1) var<storage, read> in_age: array<f32>;
@group(0) @binding(2) var<storage, read> in_warp_x: array<f32>;
@group(0) @binding(3) var<storage, read> in_warp_y: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_hetero: array<f32>;

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
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let wx = f32(gid.x) + in_warp_x[idx];
    let wy = f32(gid.y) + in_warp_y[idx];

    let low_n = gpu_fbm(wx * params.scale, wy * params.scale, params.seed) - 0.5;
    out_hetero[idx] = low_n * (0.3 + 0.7 * in_age[idx]);
}
