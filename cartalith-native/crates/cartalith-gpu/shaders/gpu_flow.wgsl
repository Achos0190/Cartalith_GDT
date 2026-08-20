// D8 flow accumulation, parallel formulation (GPU_LAYER_INTEGRATION_SCOPE.md milestone 9).
//
// `cartalith_hydrology::compute_flow` is sequential BY CONSTRUCTION: it sorts every cell into
// descending-height order, then walks that order pushing each cell's running total into its single
// steepest-descent receiver. The sort is a global data dependency and the push loop is a wavefront
// scatter -- neither survives a naive per-cell port.
//
// The redesign here is the decomposition the GIS literature already established (Qin & Zhan 2012,
// "Parallelizing flow-accumulation calculations on graphics processing units", Computers &
// Geosciences 43:7-16; "Parallel flow accumulation algorithms for GPUs with application to RUSLE",
// Computers & Geosciences 89:88-95, 2016; and HETEROGENEOUS_COMPUTE_RESEARCH.md's own §48-49):
//
//   1. FLOW DIRECTION is a pure function of the height field alone -- it never reads `acc`, so the
//      descending-height ordering is irrelevant to it. `dir_main` below is embarrassingly parallel
//      and reproduces the CPU function's inner loop exactly, including its visiting order and its
//      strict `>` first-max-wins tie-break.
//
//   2. ACCUMULATION over the resulting single-receiver forest is a subtree sum -- which is what the
//      descending-height walk actually computes, incidentally, not fundamentally. Subtree sums over
//      a pointer forest parallelize by POINTER DOUBLING (path doubling / dependency transfer): each
//      round every cell delivers its current total to the node its pointer names, then re-points at
//      that node's own pointer. The invariant, after round k:
//
//          acc[i] = sum of seeds of every cell u upstream of i (i included) with dist(u,i) < 2^k
//          ptr[i] = the cell exactly 2^k steps downstream of i (or -1 past an outlet)
//
//      which is exactly the final answer once 2^k exceeds the longest flow path. That needs
//      ceil(log2(n)) rounds in the absolute worst case -- 22 at 2048x2048 -- instead of the
//      longest-flow-path count a naive donor-gather-to-fixpoint iteration would need (thousands),
//      and instead of the global sort the CPU version pays. See the Rust side's own proof sketch.
//
// FIXED-POINT ACCUMULATION, not floats. WGSL has no atomic float add, and emulating one with a
// compare-exchange loop would make the result depend on the order threads happen to win the race --
// i.e. non-deterministic run to run, which every GPU milestone in this project has had to rule out.
// `acc`/`delta` are therefore `atomic<u32>` fixed-point: integer addition is exactly associative and
// commutative, so the scatter is order-independent AND bit-reproducible. The Rust side chooses the
// power-of-two scale from the real seed total so the largest possible accumulation still fits.
//
// World-wrap (`world` mode's x-axis wraparound) IS implemented here, unlike milestones 1-5's
// kernels -- for flow direction it is one extra modulo, not a structural change.

struct FlowParams {
    width: u32,
    height: u32,
    // 1 = x wraps (the CPU function's `world` argument), 0 = clamp/skip.
    world: u32,
    _pad0: u32,
}

@group(0) @binding(0) var<uniform> params: FlowParams;
@group(0) @binding(1) var<storage, read> field: array<f32>;
// Steepest-descent receiver per cell, -1 for a pit/outlet. Written by `dir_main`, kept read-only
// afterwards so a caller can read the raw flow directions back for verification.
@group(0) @binding(2) var<storage, read_write> recv: array<i32>;
// The doubling pointer: starts as a copy of `recv`, advances to 2^k steps downstream each round.
@group(0) @binding(3) var<storage, read_write> ptr: array<i32>;
@group(0) @binding(4) var<storage, read_write> ptr_next: array<i32>;
@group(0) @binding(5) var<storage, read_write> acc: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> delta: array<atomic<u32>>;

fn linear_index(gid: vec3<u32>) -> i32 {
    return i32(gid.y) * i32(params.width) + i32(gid.x);
}

fn in_bounds(gid: vec3<u32>) -> bool {
    return gid.x < params.width && gid.y < params.height;
}

// Pass 1 -- per-cell D8 steepest descent. Mirrors `compute_flow`'s inner double loop exactly:
// dy outer -1..1, dx inner -1..1, (0,0) skipped, `drop = (h - field[j]) / d8` with d8 the Euclidean
// step length, and a strict `>` against a `best_drop` that starts at 0 (so a cell with no strictly
// lower neighbour keeps receiver -1, and the FIRST of several equal-drop neighbours wins).
@compute @workgroup_size(8, 8, 1)
fn dir_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if !in_bounds(gid) {
        return;
    }
    let w = i32(params.width);
    let h = i32(params.height);
    let x = i32(gid.x);
    let y = i32(gid.y);
    let i = linear_index(gid);
    let hv = field[i];

    var best: i32 = -1;
    var best_drop: f32 = 0.0;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            if dx == 0 && dy == 0 {
                continue;
            }
            var nx = x + dx;
            let ny = y + dy;
            if params.world == 1u {
                nx = ((nx % w) + w) % w;
            } else if nx < 0 || nx >= w {
                continue;
            }
            if ny < 0 || ny >= h {
                continue;
            }
            let j = ny * w + nx;
            var d: f32 = 1.0;
            if dx != 0 && dy != 0 {
                d = sqrt(2.0);
            }
            let drop = (hv - field[j]) / d;
            if drop > best_drop {
                best_drop = drop;
                best = j;
            }
        }
    }
    recv[i] = best;
    ptr[i] = best;
}

// Pass 2a -- deliver this cell's current total to the cell its pointer names, and compute where the
// pointer will point next round. Reads only the round's frozen `ptr`/`acc`; writes only `delta`
// (atomically, since many cells can name the same target) and its own `ptr_next` slot.
@compute @workgroup_size(8, 8, 1)
fn scatter_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if !in_bounds(gid) {
        return;
    }
    let i = linear_index(gid);
    let p = ptr[i];
    if p < 0 {
        ptr_next[i] = -1;
        return;
    }
    let v = atomicLoad(&acc[i]);
    if v != 0u {
        atomicAdd(&delta[p], v);
    }
    // ptr[p] is this round's pointer too, never `ptr_next` -- doubling requires both reads to come
    // from the same frozen generation, which is exactly why `ptr_next` is a separate buffer.
    ptr_next[i] = ptr[p];
}

// Pass 2b -- fold the round's deliveries in and advance the pointers. One thread per cell, so the
// atomics here are uncontended; they exist only because `acc`/`delta` must be declared atomic for
// `scatter_main`'s sake.
@compute @workgroup_size(8, 8, 1)
fn merge_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if !in_bounds(gid) {
        return;
    }
    let i = linear_index(gid);
    let d = atomicExchange(&delta[i], 0u);
    if d != 0u {
        atomicAdd(&acc[i], d);
    }
    ptr[i] = ptr_next[i];
}
