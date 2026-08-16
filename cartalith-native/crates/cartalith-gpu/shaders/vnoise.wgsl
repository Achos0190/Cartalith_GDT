// GPU port of cartalith-noise's hash/vnoise (see crates/cartalith-noise/src/lib.rs).
//
// This is the "natural", portable f32 port GPU_COMPUTE_PILOT_SCOPE.md asks for --
// NOT a reformulation of the algorithm, a direct translation of the same formula.
// It is expected (and, per the pilot, measured rather than assumed) to diverge from
// the CPU f64 reference: `hash`'s own doc comment notes its middle product reaches
// ~2^61, past f64's own exact-integer range (2^53) -- f32 (24-bit mantissa) loses far
// more precision at that magnitude than f64 does, and WGSL's float->u32 conversion for
// out-of-range floats is implementation-defined/saturating, not the wrap-on-truncate
// that Rust's `(x as i64) as u32` guarantees. Both effects compound; see this crate's
// `f32_hash_diverges_from_cpu_reference` test and CHANGELOG.md for the measured result.

struct Params {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> out_values: array<f32>;

fn hash_f32(x: i32, y: i32, s: i32) -> f32 {
    let h0 = f32(x) * 374761393.0 + f32(y) * 668265263.0 + f32(s) * 362437.0;
    let h1 = u32(h0);
    let h2_bits = h1 ^ (h1 >> 13u);
    // Reference HTML/Rust: the 32-bit XOR result is re-interpreted as SIGNED
    // before the next multiply (a real sign subtlety cartalith-noise's own doc
    // comment calls out) -- mirrored here via bitcast<i32>.
    let h3 = f32(bitcast<i32>(h2_bits)) * 1274126177.0;
    let h4 = u32(h3);
    let h5 = h4 ^ (h4 >> 16u);
    return f32(h5) / 4294967295.0;
}

fn smoothstep_component(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

fn vnoise_f32(x: f32, y: f32, s: i32) -> f32 {
    let xi = floor(x);
    let yi = floor(y);
    let xf = x - xi;
    let yf = y - yi;
    let u = smoothstep_component(xf);
    let v = smoothstep_component(yf);
    let xii = i32(xi);
    let yii = i32(yi);
    let a = hash_f32(xii, yii, s);
    let b = hash_f32(xii + 1, yii, s);
    let c = hash_f32(xii, yii + 1, s);
    let d = hash_f32(xii + 1, yii + 1, s);
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
    out_values[idx] = vnoise_f32(x, y, params.seed);
}
