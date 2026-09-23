// GPU build_settlement_suitability (OUTSTANDING_WORK.md §2.6 "Phase 2 per-cell
// affordance fields"). Port of `cartalith_civ::build_settlement_suitability`
// for the FULL-context case only -- every `SuitabilityCtx` field present,
// which is the one shape production (`compute_civilisation`) calls it with.
// A partial context stays on the CPU.
//
// Per-cell plus one fixed neighbourhood: the lake term scans a
// (2*lake_r+1)^2 window of the water-body classification, lake_r =
// max(2, round(gw/170)) -- 25x25 at 2048 wide, which is where the CPU spends
// its time and why this kernel exists. Integer work, so exact.
//
// Every BRANCH takes the CPU's side exactly (host-computed f32 cuts, see
// gpu_biome.wgsl's header). The weighted sum and the sigmoid are f32 here
// against the CPU's f64-then-round; the measured bound is
// `affordance::SUITABILITY_TOLERANCE`.
//
// `fin` is 22 planar f32 planes of n cells: 0 soil, 1 water, 2 carrying
// capacity, 3 field, 4 slope_n, 5 coast_reach, 6 flow, 7 river_reach,
// 8 rain, 9 flood, 10 slope_raw, 11 corridor, 12 landmass quality, 13..21 the
// nine SUIT_RESOURCE_KEYS fields in that order (copper, tin, iron, gold,
// salt, timber, lead, silver, gems). `wb` is the water-body class, u8
// packed four per u32.

struct SuitParams {
    n: u32,
    gw: u32,
    gh: u32,
    row_threads: u32,
    lake_r: i32,
    _pad0: u32,
    sea: f32,
    denom: f32,
    sea_cut: f32,      // field < sea_cut        <=> (field as f64) < sea
    estuary_cut: f32,  // flow >= estuary_cut    <=> (flow as f64) > flow_thresh*3
    rr_lt030: f32,
    rr_lt060: f32,
    rr_lt085: f32,
    gw_f: f32,
    islet_knee: f32,
    w_k: f32,
    w_w: f32,
    w_a: f32,
    w_d: f32,
    w_agri: f32,
    w_build: f32,
    w_coast: f32,
    w_river: f32,
    w_lake: f32,
    w_mineral: f32,
    w_corridor: f32,
    w_flood: f32,
    w_islet: f32,
}

@group(0) @binding(0) var<uniform> params: SuitParams;
@group(0) @binding(1) var<storage, read> fin: array<f32>;
@group(0) @binding(2) var<storage, read> wb: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_suit: array<f32>;

fn wb_at(i: u32) -> u32 {
    return (wb[i >> 2u] >> (8u * (i & 3u))) & 0xffu;
}

fn plane(p: u32, i: u32) -> f32 {
    return fin[p * params.n + i];
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.y * params.row_threads + gid.x;
    if i >= params.n {
        return;
    }
    let fh = plane(3u, i);
    if fh < params.sea_cut || wb_at(i) != 0u {
        out_suit[i] = 0.0;
        return;
    }
    let gw = params.gw;
    let gh = params.gh;
    let x = i % gw;
    let y = i / gw;
    let slope_max = 4.0;

    let k = plane(2u, i);
    let wa = plane(1u, i);
    let a = max(1.0 - plane(4u, i) / slope_max, 0.0);
    let r = (fh - params.sea) / params.denom;
    let d = max(1.0 - 4.0 * abs(r - 0.35), 0.0); // terrain_ruggedness_d
    var z = params.w_k * k + params.w_w * wa + params.w_a * a + params.w_d * d;

    var coast = plane(5u, i);
    if coast > 0.0 && plane(6u, i) >= params.estuary_cut {
        coast = min(coast + 0.6, 1.0);
    }
    let river = plane(7u, i);

    var lake = 0.0;
    let lr = params.lake_r;
    for (var dy = -lr; dy <= lr; dy = dy + 1) {
        let ny = i32(y) + dy;
        if ny < 0 || ny >= i32(gh) { continue; }
        var hit = false;
        for (var dx = -lr; dx <= lr; dx = dx + 1) {
            let nx = i32(x) + dx;
            if nx < 0 || nx >= i32(gw) { continue; }
            if wb_at(u32(ny) * gw + u32(nx)) == 2u {
                hit = true;
                break;
            }
        }
        if hit {
            lake = 0.55;
            break;
        }
    }
    let l_n = y > 0u && wb_at(i - gw) == 2u;
    let l_s = y < gh - 1u && wb_at(i + gw) == 2u;
    let l_e = x < gw - 1u && wb_at(i + 1u) == 2u;
    let l_w = x > 0u && wb_at(i - 1u) == 2u;
    if (l_n && l_s) || (l_e && l_w) {
        lake = 1.0;
    }

    var s = 0.0;
    for (var p = 13u; p < 22u; p = p + 1u) {
        s = s + plane(p, i);
    }
    let mineral = min(s / 3.0, 1.0); // SUIT_RESOURCE_KEYS.len() / 3

    let rr = plane(8u, i);
    var r_bell = 0.0;
    if rr < params.rr_lt030 {
        r_bell = rr / 0.30;
    } else if rr < params.rr_lt060 {
        r_bell = 1.0;
    } else if rr < params.rr_lt085 {
        r_bell = (0.85 - rr) / 0.25;
    }
    let agri = clamp(plane(0u, i) * r_bell, 0.0, 1.0);

    let fl = plane(9u, i);
    let build = clamp((1.0 - min(plane(10u, i) * params.gw_f / slope_max, 1.0)) * (1.0 - fl), 0.0, 1.0);
    let corr = plane(11u, i);
    let islet = max(1.0 - plane(12u, i) / params.islet_knee, 0.0);

    z = z + params.w_coast * coast
          + params.w_river * river
          + params.w_lake * lake
          + params.w_mineral * mineral
          + params.w_corridor * corr
          + params.w_agri * agri
          + params.w_build * build
          - params.w_flood * fl
          - params.w_islet * islet;

    out_suit[i] = clamp(1.0 / (1.0 + exp(-6.0 * (z - 0.5))), 0.0, 1.0);
}
