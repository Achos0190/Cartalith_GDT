// GPU-safe height formula (GPU_LAYER_INTEGRATION_SCOPE.md milestone 3).
//
// Ports `compute_height`'s per-cell body -- one noise evaluation (fbm or
// ridged, selected by `params.ridged`) plus a weighted sum over
// already-materialized input fields. NON-world (`world=false`) branch
// only, matching milestone 2's gpu_warp.wgsl/gpu_heterogeneity.wgsl own
// scope -- no periodic (pfbm/pridged) combinator here.
//
// Duplicates gpu_hash/gpu_vnoise/gpu_fbm/gpu_ridged from the other shader
// files in this directory (no cross-file WGSL module include in this
// crate's convention) -- mirror all copies operation-for-operation if any
// changes.
//
// `warp_x`/`warp_y`/`oro` are always-bound storage buffers (WGSL can't
// express `Option<&[f32]>`) -- the caller passes a zero-filled buffer for
// warp_x/warp_y when absent (adding zero is a no-op, matching
// `warp_x.map_or(0.0, ...)`'s CPU behaviour exactly). `oro` is different:
// its ABSENCE changes which formula runs (`t = stress` vs.
// `t = oro[i] + min(stress, 0)`), not just an additive no-op -- `params
// .has_oro` selects between the two branches explicitly, matching
// `compute_height`'s own `match oro { Some(o) => ..., None => sf }`.

struct HeightParams {
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

@group(0) @binding(0) var<uniform> params: HeightParams;
@group(0) @binding(1) var<storage, read> in_base: array<f32>;
@group(0) @binding(2) var<storage, read> in_stress: array<f32>;
@group(0) @binding(3) var<storage, read> in_flex: array<f32>;
@group(0) @binding(4) var<storage, read> in_hetero: array<f32>;
@group(0) @binding(5) var<storage, read> in_age: array<f32>;
@group(0) @binding(6) var<storage, read> in_warp_x: array<f32>;
@group(0) @binding(7) var<storage, read> in_warp_y: array<f32>;
@group(0) @binding(8) var<storage, read> in_oro: array<f32>;
@group(0) @binding(9) var<storage, read_write> out_height: array<f32>;

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

fn gpu_ridged(x: f32, y: f32, s: i32) -> f32 {
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var sum: f32 = 0.0;
    var nrm: f32 = 0.0;
    for (var o: i32 = 0; o < 6; o = o + 1) {
        var n = gpu_vnoise(x * freq, y * freq, s + o * 131);
        n = 1.0 - abs(2.0 * n - 1.0);
        sum += amp * n * n;
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

    let sf = in_stress[idx];
    let t = select(sf, in_oro[idx] + min(sf, 0.0), params.has_oro != 0u);
    let bs = in_base[idx];
    let rug = exp(-in_age[idx] * (1.0 + params.age_inf * 6.0));

    let wx = f32(gid.x) + in_warp_x[idx];
    let wy = f32(gid.y) + in_warp_y[idx];
    // Both axes divide by WIDTH, not height -- matches compute_height's own
    // `nx = wx*nf/gw as f64; ny = wy*nf/gw as f64` (same convention
    // compute_warp/compute_heterogeneity already use elsewhere).
    let nx = wx * params.nf / f32(params.width);
    let ny = wy * params.nf / f32(params.width);

    var n_val: f32;
    if params.ridged != 0u {
        n_val = gpu_ridged(nx, ny, params.seed) - 0.5;
    } else {
        n_val = gpu_fbm(nx, ny, params.seed) - 0.5;
    }

    out_height[idx] = 0.5
        + params.a * (0.40 * bs + 0.50 * t)
        + params.fwt * in_flex[idx]
        + params.hwt * in_hetero[idx]
        + params.b * n_val * (0.25 + 0.75 * rug);
}
