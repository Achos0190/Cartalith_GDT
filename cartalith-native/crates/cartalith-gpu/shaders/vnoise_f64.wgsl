// Secondary pilot experiment: same kernel as vnoise.wgsl, but using WGSL's
// optional `f64` type (requires wgpu::Features::SHADER_F64, Vulkan-only,
// native-only -- confirmed present on this session's real adapter by
// `f64_probe::check_shader_f64_availability`). Tests whether the CPU
// reference's f64-rounding-dependent `hash` formula CAN be reproduced
// exactly on GPU, as the natural f32 port (vnoise.wgsl) could not.
enable f64;

struct Params {
    seed: i32,
    width: u32,
    height: u32,
    scale: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> out_values: array<f32>;

fn hash_f64(x: i32, y: i32, s: i32) -> f64 {
    let h0 = f64(x) * 374761393.0 + f64(y) * 668265263.0 + f64(s) * 362437.0;
    let h1 = u32(h0);
    let h2_bits = h1 ^ (h1 >> 13u);
    let h3 = f64(bitcast<i32>(h2_bits)) * 1274126177.0;
    let h4 = u32(h3);
    let h5 = h4 ^ (h4 >> 16u);
    return f64(h5) / 4294967295.0;
}

fn smoothstep_component_f64(t: f64) -> f64 {
    return t * t * (3.0 - 2.0 * t);
}

fn vnoise_f64(x: f64, y: f64, s: i32) -> f64 {
    let xi = floor(x);
    let yi = floor(y);
    let xf = x - xi;
    let yf = y - yi;
    let u = smoothstep_component_f64(xf);
    let v = smoothstep_component_f64(yf);
    let xii = i32(xi);
    let yii = i32(yi);
    let a = hash_f64(xii, yii, s);
    let b = hash_f64(xii + 1, yii, s);
    let c = hash_f64(xii, yii + 1, s);
    let d = hash_f64(xii + 1, yii + 1, s);
    return a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let x = f64(gid.x) * f64(params.scale);
    let y = f64(gid.y) * f64(params.scale);
    // storage output stays f32 (matches CPU reference's own f64->f32 cast
    // point in vnoise_grid_cpu) -- only the internal arithmetic is f64.
    out_values[idx] = f32(vnoise_f64(x, y, params.seed));
}
