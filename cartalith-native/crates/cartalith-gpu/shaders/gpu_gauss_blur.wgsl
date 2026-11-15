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
    // 1 / (2 * radius + 1), computed on the CPU in f32 and uploaded (Ruling EZ
    // item 3, 2026-11-15; `blur_norm` in lib.rs). WGSL lets the driver
    // evaluate f32 division to 2.5 ULP, and the in-shader division this
    // replaces was the kernel's only operation not correctly rounded: the
    // flexure blur's radius came out differently on the Adreno 630 and the
    // RX 7800 XT (`gpu_divergence_bench`, PERFORMANCE_BENCHMARKS.md 12.8).
    // Now the kernel is adds and one multiply, correctly rounded in WGSL and
    // with no multiply-add to fuse, so every device gives the CPU mirror's bits.
    norm: f32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
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

// Entry point 1 of 2: horizontal pass. `radius`/`wrap` come from `params`
// (set per-pass by `dispatch_gpu_gauss_blur` -- box_v's own BlurParams
// always has wrap=0, matching the CPU box_v's no-wrap behaviour).
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
    out_field[y * w + x] = sum * params.norm;
}

// Entry point 2 of 2: vertical pass, always clamp-to-edge (no world wrap --
// there is no pole-to-pole wrap to apply, matching the CPU box_v exactly).
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
    out_field[y * w + x] = sum * params.norm;
}
