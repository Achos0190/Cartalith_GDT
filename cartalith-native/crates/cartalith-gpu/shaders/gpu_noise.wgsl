// GPU-safe noise primitive (GPU_LAYER_INTEGRATION_SCOPE.md milestone 1).
//
// Deliberately NOT a port of cartalith-noise::hash -- that function's
// JS-matching semantics depend on f64 rounding at an intermediate magnitude
// (~2^61), unrepresentable in f32 and unreachable via WGSL at all (naga has
// no working `f64` support on this toolchain; see GPU_COMPUTE_PILOT_SCOPE.md
// and this crate's f64_wgsl_is_not_implemented_by_naga_even_though_the_gpu_feature_exists
// test). This is a fresh, GPU-native hash using ONLY u32 wrapping arithmetic
// (multiply/add/xor/shift), which Rust and WGSL both specify identically --
// u32 wraps on overflow by spec in both -- so this pair has NO
// cross-precision-regime gap the old hash/f32 pairing did.
//
// Construction: single-round PCG3D (Mark Jarzynski & Marc Olano, "Hash
// Functions for GPU Rendering," Journal of Computer Graphics Techniques,
// vol. 9, no. 3, 2020, https://www.jcgt.org/published/0009/03/02/) -- a hash
// designed specifically for GPU shaders. Mirror this file operation-for-
// operation against cartalith-noise's `pcg3d`/`gpu_hash`/`gpu_vnoise`
// (crates/cartalith-noise/src/lib.rs) if either ever changes -- they must
// stay in lockstep, that correspondence is this milestone's entire point.

struct Params {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> out_values: array<f32>;

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

// (x, y, s) reinterpreted bit-for-bit as u32 -- bitcast, not a
// value-preserving conversion, matching Rust's `as u32` cast on an i32
// exactly (both are two's-complement bit-pattern reinterprets).
fn gpu_hash(x: i32, y: i32, s: i32) -> u32 {
    let v = pcg3d(vec3<u32>(bitcast<u32>(x), bitcast<u32>(y), bitcast<u32>(s)));
    return v.x;
}

// u32 -> f32 is a standard, fully-specified IEEE-754 round-to-nearest
// conversion in both WGSL (`f32(...)`) and Rust (`as f32`) -- unlike the
// f32 -> u32 direction the original pilot hit (implementation-defined for
// out-of-range values), this direction has no platform-dependent behaviour.
fn gpu_hash_to_unit_f32(h: u32) -> f32 {
    return f32(h) / 4294967295.0; // u32::MAX as f32, matches Rust side exactly
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

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let x = f32(gid.x) * params.scale;
    let y = f32(gid.y) * params.scale;
    out_values[idx] = gpu_vnoise(x, y, params.seed);
}
