// GPU build_resource_potentials, per-cell part (OUTSTANDING_WORK.md §2.6
// "Phase 2 per-cell affordance fields"). Port of the per-cell closure in
// `cartalith_civ::build_resource_potentials` -- the fifteen geological
// potentials. The function's three whole-raster steps stay on the CPU and
// are NOT here: the copper-source chamfer distance (a two-pass sweep,
// `resource_copper_dist`, handed in as plane 6), the flow maximum (a global
// reduction, `resource_flow_max`, handed in as `ln_den`), and the scarcity
// cut (a rank threshold over every land cell, `finish_resource_potentials`,
// run after readback).
//
// Every BRANCH takes the CPU's side exactly: each float threshold is a
// host-computed f32 cut (see gpu_biome.wgsl's header), including the three
// on the derived elevation fraction r = max((field - sea)/denom, 0), which is
// monotone in `field` and so reduces to a cut on `field` itself. Only the
// continuous values (exp, ln, the linear blends) differ from the CPU's
// f64-then-round, by a few f32 ulps.
//
// Inputs are planar: `fin` holds 7 f32 planes of n cells, `bytes` holds 3 u8
// planes of `words` u32 each (lith, boundary_type, biome). Output is 15
// planes of n cells in ResourcePotentials field order. An absent optional
// input is an all-zero plane, which every formula below treats exactly as
// the CPU treats `None`.

struct ResParams {
    n: u32,
    words: u32,
    row_threads: u32,
    _pad0: u32,
    cu_lam: f32,
    ln_den: f32,     // ln(1 + flow_max * 0.05), computed in f64 on the host
    age_gt060: f32,  // ai > 0.60     <=> ai >= cut
    rain_gt055: f32,
    rain_lt022: f32, // ri < 0.22     <=> ri < cut
    rain_lt012: f32,
    rain_gt050: f32,
    rain_lt030: f32,
    sh_gt025: f32,
    sh_gt030: f32,
    vv_gt045: f32,
    vv_gt035: f32,
    vv_gt030: f32,
    f_r_lt025: f32,  // r < 0.25      <=> field < cut
    f_r_lt012: f32,
    f_r_lt035: f32,
}

@group(0) @binding(0) var<uniform> params: ResParams;
@group(0) @binding(1) var<storage, read> fin: array<f32>;
@group(0) @binding(2) var<storage, read> bytes: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_res: array<f32>;

fn byte_plane(plane: u32, i: u32) -> u32 {
    let w = bytes[plane * params.words + (i >> 2u)];
    return (w >> (8u * (i & 3u))) & 0xffu;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.y * params.row_threads + gid.x;
    let n = params.n;
    if i >= n {
        return;
    }
    let li = byte_plane(0u, i);
    let bt = byte_plane(1u, i);
    let bv = byte_plane(2u, i);
    let sh = abs(fin[i]);
    let flow = fin[n + i];
    let fh = fin[2u * n + i];
    let ri = fin[3u * n + i];
    let ai = fin[4u * n + i];
    let vv = fin[5u * n + i];
    let cu_dist = fin[6u * n + i];

    let old = ai >= params.age_gt060;
    let r_lt025 = fh < params.f_r_lt025;
    let r_lt012 = fh < params.f_r_lt012;
    let r_lt035 = fh < params.f_r_lt035;

    // copper
    var cu_mult = 0.55;
    if li == 2u { cu_mult = 1.0; } else if li == 1u { cu_mult = 0.8; }
    let copper = min(exp(-cu_dist / params.cu_lam) * cu_mult, 1.0);

    // tin
    var tin = 0.0;
    if li == 0u && old { tin = 0.70; } else if li == 6u { tin = 0.45; } else if li == 0u { tin = 0.30; }

    // iron
    var iron = 0.0;
    if li == 0u && old && bt == 0u {
        iron = 0.65;
    } else if li == 5u && ri >= params.rain_gt055 && r_lt025 {
        iron = 0.55;
    } else if li == 3u {
        iron = 0.20;
    }

    // gold
    var gold = 0.0;
    if bt == 5u {
        gold = min(0.65 + 0.35 * sh, 1.0);
    } else if sh >= params.sh_gt025 && li == 0u {
        gold = min(0.20 + sh, 0.55);
    } else if li == 0u && old {
        gold = 0.12;
    }

    // salt
    var salt = 0.0;
    if r_lt025 && ri < params.rain_lt022 {
        if li == 3u || li == 4u {
            salt = min(0.50 + 0.40 * (0.22 - ri) / 0.22, 0.90);
        } else if r_lt012 && ri < params.rain_lt012 {
            salt = 0.40;
        }
    }

    // timber
    var timber = 0.0;
    if bv == 3u || bv == 4u || bv == 5u || bv == 6u || bv == 12u {
        timber = min(0.40 + 0.60 * min(ri * 1.5, 1.0), 1.0);
    }

    // lead
    var lead = 0.0;
    if li == 3u {
        var bonus = 0.0;
        if bt != 0u { bonus = 0.20; }
        lead = min(0.25 + 0.55 * min(sh * 2.2, 1.0) + bonus, 1.0);
    } else if li == 6u && sh >= params.sh_gt030 {
        lead = 0.25;
    }

    // silver
    var silver = 0.0;
    if lead > 0.0 { silver = lead * 0.55; }

    // clay (+ kaolin)
    var clay = 0.0;
    if r_lt035 {
        let wet = min(log(1.0 + flow) / params.ln_den, 1.0);
        var v = 0.30 + 0.50 * wet + 0.25 * min(ri * 1.6, 1.0);
        if li == 0u { v = v - 0.25; }
        clay = clamp(v, 0.0, 1.0);
    }
    if li == 0u && ri >= params.rain_gt050 && clay > 0.0 {
        clay = min(clay + 0.20, 1.0);
    }

    // building stone
    var buildstone = 0.15;
    if li == 3u { buildstone = 0.85; } else if li == 0u || li == 1u { buildstone = 0.70; } else if li == 4u { buildstone = 0.45; } else if li == 6u { buildstone = 0.40; }

    // flint
    var flint = 0.0;
    if li == 3u { flint = 0.60; }

    // obsidian
    var obsidian = 0.0;
    if vv >= params.vv_gt045 && (li == 2u || li == 1u) {
        obsidian = min(0.35 + 0.65 * vv, 1.0);
    } else if li == 2u && bt == 3u {
        obsidian = 0.30;
    }

    // gems
    var gems = 0.0;
    if li == 0u && old {
        gems = min(0.30 + 0.50 * min(sh * 2.0, 1.0), 1.0);
    } else if li == 6u {
        gems = min(0.20 + 0.55 * min(sh * 2.5, 1.0), 1.0);
    }

    // sulfur
    var sulfur = 0.0;
    if vv >= params.vv_gt035 { sulfur = min(0.25 + 0.75 * vv, 1.0); }

    // alum
    var alum = 0.0;
    if vv >= params.vv_gt030 {
        alum = min(0.20 + 0.60 * vv, 1.0);
    } else if r_lt025 && ri < params.rain_lt030 && (li == 4u || li == 5u) {
        alum = 0.45;
    }

    out_res[i] = copper;
    out_res[n + i] = tin;
    out_res[2u * n + i] = iron;
    out_res[3u * n + i] = gold;
    out_res[4u * n + i] = salt;
    out_res[5u * n + i] = timber;
    out_res[6u * n + i] = lead;
    out_res[7u * n + i] = silver;
    out_res[8u * n + i] = clay;
    out_res[9u * n + i] = buildstone;
    out_res[10u * n + i] = flint;
    out_res[11u * n + i] = obsidian;
    out_res[12u * n + i] = gems;
    out_res[13u * n + i] = sulfur;
    out_res[14u * n + i] = alum;
}
