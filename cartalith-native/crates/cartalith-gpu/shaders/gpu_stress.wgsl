// The boundary loop of `cartalith_terrain::compute_stress` (reference
// `computeStress`, HTML lines 2819-2848), rewritten from SCATTER to GATHER.
// Only the loop: the two Gaussian blurs that follow it run on the existing
// `gpu_gauss_blur.wgsl`, and the max-normalise on the CPU.
//
// The CPU loop visits every cell `i` in row-major order and, for each of its
// right / down / (world only, x == gw-1) row-wrap neighbours `j` on another
// plate, adds the edge's convergence `c` and shear `s` into BOTH `raw[i]` and
// `raw[j]`, and offers the edge's magnitude to both cells' running
// dominant-type pick. Two scanners can hit one cell in the same sweep (its
// upper and its left neighbour), so as written it is a multi-writer scatter.
//
// The gather rests on one observation: an edge's whole contribution --
// `c`, `s`, `mag = |c|+|s|` and its boundary type -- is a function of the
// PAIR (scanner plate `a`, neighbour plate `b`) and nothing else. So the host
// tabulates it per pair in f64, exactly as the CPU loop computes it
// (`cartalith_terrain::stress_edge`), and each cell here enumerates the edges
// it takes part in and reads their contributions from the table. The cell's
// edges, in the order the CPU sweep reaches them (by scanner index):
//
//   1. from its UP neighbour's scan    (k-gw  -> k, a = plate[k-gw])
//   2. from its LEFT neighbour's scan  (k-1   -> k, a = plate[k-1])
//   3. its own scan: RIGHT             (k -> k+1,   a = own plate)
//   4. its own scan: DOWN              (k -> k+gw,  a = own plate)
//   5. its own scan: ROW-WRAP, world && x == gw-1 (k -> y*gw)
//   6. the row's last cell's ROW-WRAP scan, world && x == 0 (y*gw+gw-1 -> k)
//
// Each cell writes only itself: no atomics, no reduction, deterministic by
// construction. Summing in that order also keeps the dominant-type tie rule
// (`>=`, so a later edge wins a tie) meaning what it means on the CPU.
//
// Precision: WGSL has no f64. The CPU adds each f64 `c` to the f32 cell and
// rounds once (`(raw as f64 + c) as f32`); this adds `f32(c)` in f32, which
// can differ by an ulp per step -- principled equivalence (`DECISIONS.md`
// §7a), measured in `cartalith-engine`'s stress tests. The dominant-type pick
// does NOT lose anything: the CPU compares the f64 `mag` against the STORED
// f32 `dom`, and `mag >= D` for an f32 `D` is exactly
// `hi > D || (hi == D && !(mag < hi))` where `hi = f32(mag)` (round-to-nearest
// is monotone and fixes every f32). The host ships `hi` and that one bit, so
// the mask and the boundary type are bit-identical to the CPU's.

struct StressParams {
    width: u32,
    height: u32,
    world: u32,
    n_plates: u32,
}

// One (scanner plate, neighbour plate) edge, `stress_edge`'s output packed
// for f32: `mag_below` is `mag < f32(mag)` evaluated in f64 on the host.
struct Pair {
    c: f32,
    s: f32,
    mag: f32,
    mag_below: u32,
    bt: u32,
}

@group(0) @binding(0) var<uniform> params: StressParams;
@group(0) @binding(1) var<storage, read> plate_id: array<u32>;
@group(0) @binding(2) var<storage, read> pairs: array<Pair>;
// Three planes of `width*height`: raw, raw_s (f32 bits), then
// mask | (type << 8).
@group(0) @binding(3) var<storage, read_write> out: array<u32>;

var<private> raw: f32;
var<private> raw_s: f32;
var<private> dom: f32;
var<private> btype: u32;
var<private> mask: u32;

// One edge's contribution to THIS cell, `a` being the scanning cell's plate.
fn visit(a: u32, b: u32) {
    if a == b {
        return;
    }
    mask = 1u;
    let p = pairs[a * params.n_plates + b];
    raw = raw + p.c;
    raw_s = raw_s + p.s;
    if p.mag > dom || (p.mag == dom && p.mag_below == 0u) {
        dom = p.mag;
        btype = p.bt;
    }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = params.width;
    let h = params.height;
    if gid.x >= w || gid.y >= h {
        return;
    }
    let x = gid.x;
    let y = gid.y;
    let k = y * w + x;
    let me = plate_id[k];
    raw = 0.0;
    raw_s = 0.0;
    dom = 0.0;
    btype = 0u;
    mask = 0u;

    if y > 0u {
        visit(plate_id[k - w], me);
    }
    if x > 0u {
        visit(plate_id[k - 1u], me);
    }
    if x + 1u < w {
        visit(me, plate_id[k + 1u]);
    }
    if y + 1u < h {
        visit(me, plate_id[k + w]);
    }
    if params.world != 0u && x == w - 1u {
        visit(me, plate_id[y * w]);
    }
    if params.world != 0u && x == 0u {
        visit(plate_id[y * w + w - 1u], me);
    }

    let n = w * h;
    out[k] = bitcast<u32>(raw);
    out[n + k] = bitcast<u32>(raw_s);
    out[2u * n + k] = mask | (btype << 8u);
}
