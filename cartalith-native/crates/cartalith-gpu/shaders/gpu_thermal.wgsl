// One pass of `cartalith_erosion::erode_thermal` (reference `erodeThermalCPU`,
// HTML lines 3856-3865), rewritten from SCATTER to GATHER.
//
// The CPU pass lets each over-steep cell push `move` out of itself and into
// the 4-neighbours it is steeper than, split in proportion to each one's
// excess over the talus angle. Several uphill cells can push into the same
// downhill cell in one pass, so the CPU form is a multi-writer scatter and
// cannot run one thread per cell as written.
//
// The gather form is the same arithmetic read from the receiving side. Every
// quantity a donor's push depends on -- its own height, its excess, its
// `move` -- is a function of the FROZEN start-of-pass field and the donor's
// own 4-neighbourhood, so a cell can recompute any neighbour's push itself:
//
//   out[i] = clamp(h[i] - move(i) + sum_k move(k) * (h[k]-h[i]-talus) / excess(k), 0, 1)
//
// over the in-bounds 4-neighbours k with h[k]-h[i] > talus. That reads a
// radius-2 diamond (13 cells) per output cell and writes only its own cell --
// no atomics, no reduction, so a dispatch is deterministic by construction.
//
// Boundaries: out-of-grid neighbours are skipped, never wrapped, exactly as
// the CPU pass skips them. `erode_thermal` takes no `world` flag, so there is
// no X-wrap to port.
//
// Precision: WGSL has no f64 (naga rejects `enable f64;`). The CPU pass
// computes `dh`, `excess` and `move` in f64 and rounds each `delta[j] +=`
// to f32 in scan order; this pass does all of it in f32 and sums a cell's
// inflows in neighbour order. The two are principled-equivalent, not
// bit-identical (`DECISIONS.md` §7a) -- the measured deviation and the
// tolerance derived from it are in `cartalith-engine`'s erode_op tests.
//
// The per-pass `[0,1]` clamp is fused in: the CPU pass clamps after every
// pass too, before the next one reads the field.

struct ThermalParams {
    width: u32,
    height: u32,
    talus: f32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: ThermalParams;
@group(0) @binding(1) var<storage, read> in_field: array<f32>;
@group(0) @binding(2) var<storage, read_write> out_field: array<f32>;

fn in_grid(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn h_at(x: i32, y: i32) -> f32 {
    return in_field[u32(y) * params.width + u32(x)];
}

// `erode_thermal`'s own neighbour order: -x, +x, -y, +y. A function rather
// than a module-scope `const` array, so the runtime index never depends on a
// backend's support for dynamically indexing a value array.
fn nb(k: i32) -> vec2<i32> {
    if k == 0 {
        return vec2<i32>(-1, 0);
    }
    if k == 1 {
        return vec2<i32>(1, 0);
    }
    if k == 2 {
        return vec2<i32>(0, -1);
    }
    return vec2<i32>(0, 1);
}

// Sum over in-bounds 4-neighbours of (h - h_n - talus) where that is positive.
fn excess_at(x: i32, y: i32) -> f32 {
    let hh = h_at(x, y);
    var e: f32 = 0.0;
    for (var k: i32 = 0; k < 4; k = k + 1) {
        let n = vec2<i32>(x, y) + nb(k);
        if !in_grid(n.x, n.y) {
            continue;
        }
        let dh = hh - h_at(n.x, n.y);
        if dh > params.talus {
            e += dh - params.talus;
        }
    }
    return e;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let hh = h_at(x, y);

    var d: f32 = 0.0;
    // Outflow: `move_amt = hh.min(excess * 0.5 * 0.25)`.
    let e_self = excess_at(x, y);
    if e_self > 0.0 {
        d -= min(hh, e_self * 0.5 * 0.25);
    }
    // Inflow: every neighbour steeper than this cell by more than talus
    // pushes its share `move_k * (e / excess_k)` here. `excess_k > 0` is
    // guaranteed whenever this branch runs -- this cell is one of its terms.
    for (var k: i32 = 0; k < 4; k = k + 1) {
        let n = vec2<i32>(x, y) + nb(k);
        if !in_grid(n.x, n.y) {
            continue;
        }
        let hk = h_at(n.x, n.y);
        let dk = hk - hh;
        if dk > params.talus {
            let ek = excess_at(n.x, n.y);
            let mk = min(hk, ek * 0.5 * 0.25);
            d += mk * ((dk - params.talus) / ek);
        }
    }
    out_field[u32(y) * params.width + u32(x)] = clamp(hh + d, 0.0, 1.0);
}
