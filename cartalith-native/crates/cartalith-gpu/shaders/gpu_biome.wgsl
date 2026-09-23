// GPU build_biome_raster (GPU_LAYER_INTEGRATION_SCOPE.md, OUTSTANDING_WORK.md
// §2.6 "Phase 2 per-cell affordance fields"). Port of
// `cartalith_civ::build_biome_raster` + `classify_biome`: water-body class
// overrides climate (1 -> ocean 0, 2 -> lake 13), land -> the 12-way
// temperature/moisture ladder.
//
// **Bit-exact by construction, not by tolerance.** The CPU compares
// `f32 as f64` against `f64` literals; here every comparison is `f32 < cut`
// where `cut` is the smallest f32 for which the CPU's f64 comparison fails
// (computed on the host, `affordance::lt_cut`). For every finite f32 input
// the two comparisons agree, so the class chosen is the CPU's class.
// Temperature thresholds (-7, 0, 5, 12, 20) are exact in f32 and need no cut.
//
// u8 in and out, packed four per u32 (little-endian: cell i is byte i&3 of
// word i>>2). One invocation per output WORD, so no two threads write the
// same u32.

struct BiomeParams {
    n: u32,
    words: u32,
    row_threads: u32,
    _pad0: u32,
    // moisture cuts, in the order classify_biome tests them
    m020: f32,
    m030: f32,
    m060: f32,
    m012: f32,
    m028: f32,
    m055: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> params: BiomeParams;
@group(0) @binding(1) var<storage, read> wb: array<u32>;
@group(0) @binding(2) var<storage, read> temp: array<f32>;
@group(0) @binding(3) var<storage, read> rain: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_biome: array<u32>;

fn classify(t: f32, m: f32) -> u32 {
    if t < -7.0 { return 1u; }
    if t < 0.0 { return 2u; }
    if t < 5.0 {
        if m < params.m020 { return 2u; }
        return 3u;
    }
    if t < 12.0 {
        if m < params.m030 { return 7u; }
        if m < params.m060 { return 4u; }
        return 6u;
    }
    if t < 20.0 {
        if m < params.m012 { return 9u; }
        if m < params.m028 { return 8u; }
        if m < params.m055 { return 5u; }
        return 6u;
    }
    if m < params.m012 { return 9u; }
    if m < params.m030 { return 10u; }
    if m < params.m055 { return 11u; }
    return 12u;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = gid.y * params.row_threads + gid.x;
    if w >= params.words {
        return;
    }
    let wbw = wb[w];
    var packed = 0u;
    for (var k = 0u; k < 4u; k = k + 1u) {
        let i = w * 4u + k;
        if i >= params.n {
            break;
        }
        let c = (wbw >> (8u * k)) & 0xffu;
        var b = 0u;
        if c == 1u {
            b = 0u;
        } else if c == 2u {
            b = 13u;
        } else {
            b = classify(temp[i], rain[i]);
        }
        packed = packed | (b << (8u * k));
    }
    out_biome[w] = packed;
}
