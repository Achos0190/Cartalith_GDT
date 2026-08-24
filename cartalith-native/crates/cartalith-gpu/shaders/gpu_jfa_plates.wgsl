// GPU-safe plate assignment via Jump Flooding Algorithm (GPU_LAYER_INTEGRATION_SCOPE.md
// milestone 5).
//
// `cartalith_terrain::assign_plates` implements JFA with IN-PLACE mutation: it scans the
// grid row-major and updates `nearest[i]`/`best_d2[i]` directly, so a cell processed later
// in the SAME pass can see another cell's update from EARLIER in that same pass (not just
// the previous pass's final state). That is a real, order-dependent algorithm variant, not
// an implementation detail like `gauss_blur`'s running-sum -- it changes which specific
// nearest-seed answer a cell converges to in ambiguous/boundary cases.
//
// This kernel is a standard DOUBLE-BUFFERED JFA instead: every invocation reads only the
// frozen previous-pass state (`nearest_in`/`best_d2_in`) and writes its own cell of
// `nearest_out`/`best_d2_out` -- the textbook, race-free GPU formulation, and the one JFA is
// actually known for. It is NOT expected to reproduce the CPU function's specific in-place
// answer cell-for-cell; both are valid completions of the same jump-flood *algorithm*, not
// the same *procedure*. See the Rust-side test suite for how this is actually verified
// (against brute-force exact-nearest ground truth, not against the CPU function directly).
//
// World-wrap (`world` mode's x-axis wraparound) is deliberately NOT implemented here,
// matching every GPU milestone so far (1-4) -- the non-wrapping case is the one exercised
// and verified.

struct JfaParams {
    width: u32,
    height: u32,
    step: i32,
}

@group(0) @binding(0) var<uniform> params: JfaParams;
@group(0) @binding(1) var<storage, read> nearest_in: array<i32>;
@group(0) @binding(2) var<storage, read> best_d2_in: array<f32>;
@group(0) @binding(3) var<storage, read_write> nearest_out: array<i32>;
@group(0) @binding(4) var<storage, read_write> best_d2_out: array<f32>;
@group(0) @binding(5) var<storage, read> plate_x: array<f32>;
@group(0) @binding(6) var<storage, read> plate_y: array<f32>;
@group(0) @binding(7) var<storage, read> warp_x: array<f32>;
@group(0) @binding(8) var<storage, read> warp_y: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let w = i32(params.width);
    let h = i32(params.height);
    let x = i32(gid.x);
    let y = i32(gid.y);
    let i = y * w + x;
    let step = params.step;

    var best_nearest: i32 = nearest_in[i];
    var best_d2: f32 = best_d2_in[i];

    let ax = f32(x) + warp_x[i];
    let ay = f32(y) + warp_y[i];

    // JS/CPU visits exactly the three offsets {-step, 0, step} per axis
    // (not a full [-step,step] range) -- nine total, minus the (0,0) skip.
    let offsets = array<i32, 3>(-step, 0, step);
    for (var oy: i32 = 0; oy < 3; oy = oy + 1) {
        for (var ox: i32 = 0; ox < 3; ox = ox + 1) {
            let dx = offsets[ox];
            let dy = offsets[oy];
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if nx < 0 || nx >= w || ny < 0 || ny >= h {
                continue;
            }
            let j = ny * w + nx;
            let p = nearest_in[j];
            if p < 0 {
                continue;
            }
            let ddx = ax - plate_x[p];
            let ddy = ay - plate_y[p];
            let d2 = ddx * ddx + ddy * ddy;
            if d2 < best_d2 {
                best_d2 = d2;
                best_nearest = p;
            }
        }
    }

    nearest_out[i] = best_nearest;
    best_d2_out[i] = best_d2;
}
