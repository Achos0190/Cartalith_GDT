// GPU-safe separable box blur (GPU_LAYER_INTEGRATION_SCOPE.md milestone 4).
//
// `cartalith_terrain::gauss_blur` runs three passes of horizontal-then-
// vertical box blur (`box_h`/`box_v`) to approximate a Gaussian, using a
// SLIDING-WINDOW RUNNING SUM accumulated in f64 (only rounded to f32 on
// write) for O(w) work per row instead of O(w*window). That running-sum
// optimization is a CPU implementation detail, not part of the box
// filter's mathematical definition (which only specifies "the average of
// this window"), and it isn't even expressible on this GPU toolchain --
// WGSL has no f64 at all (naga rejects `enable f64;`, confirmed by the
// GPU-compute pilot). This kernel instead computes a direct per-output-
// cell local-window sum in f32: the same window, the same normalization,
// a different (GPU-native, embarrassingly parallel) evaluation order.
//
// Verified against the untouched CPU `cartalith_terrain::gauss_blur`
// directly (a real `cartalith-gpu` test with `cartalith-terrain` as a
// dev-dependency) -- see that test for the measured tolerance this
// equivalence actually holds to, and whether it reaches three-way
// JS/CPU/GPU parity or needs its own GPU-vs-CPU-twin carve-out.

struct BlurParams {
    width: u32,
    height: u32,
    radius: i32,
    wrap: u32, // box_h only: 0 = clamp-to-edge, 1 = world x-wrap. box_v always clamps (no pole-to-pole wrap, matching the CPU box_v).
}

@group(0) @binding(0) var<uniform> params: BlurParams;
@group(0) @binding(1) var<storage, read> in_field: array<f32>;
@group(0) @binding(2) var<storage, read_write> out_field: array<f32>;

fn wrap_index(i: i32, n: i32) -> i32 {
    return ((i % n) + n) % n;
}

fn clamp_index(i: i32, n: i32) -> i32 {
    return clamp(i, 0, n - 1);
}

@compute @workgroup_size(8, 8, 1)
fn box_h_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let w = i32(params.width);
    let y = i32(gid.y);
    let x = i32(gid.x);
    let r = params.radius;
    var sum: f32 = 0.0;
    for (var k: i32 = -r; k <= r; k = k + 1) {
        var idx: i32;
        if params.wrap != 0u {
            idx = wrap_index(x + k, w);
        } else {
            idx = clamp_index(x + k, w);
        }
        sum += in_field[y * w + idx];
    }
    let norm = 1.0 / (2.0 * f32(r) + 1.0);
    out_field[y * w + x] = sum * norm;
}

@compute @workgroup_size(8, 8, 1)
fn box_v_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let w = i32(params.width);
    let h = i32(params.height);
    let x = i32(gid.x);
    let y = i32(gid.y);
    let r = params.radius;
    var sum: f32 = 0.0;
    for (var k: i32 = -r; k <= r; k = k + 1) {
        let idx = clamp_index(y + k, h);
        sum += in_field[idx * w + x];
    }
    let norm = 1.0 / (2.0 * f32(r) + 1.0);
    out_field[y * w + x] = sum * norm;
}
