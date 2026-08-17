// GPU-safe climate wind/rain loop (GPU_LAYER_INTEGRATION_SCOPE.md milestone 7).
//
// Ports `cartalith_climate::simulate_weather`'s inner `for _ in 0..iters`
// loop, one iteration = three gather-shaped passes, sequenced (no cell ever
// writes another cell's output within a pass, but pass 2 needs pass 1's
// COMPLETE output and pass 3 needs pass 2's -- three separate dispatches per
// iteration, not one fused kernel, since WGSL has no cross-workgroup
// barrier mid-dispatch). The CPU function computes in f64 throughout
// (JS-parity); WGSL has no real f64 (naga rejects `enable f64;`, confirmed
// by the GPU-compute pilot) -- this kernel is an f32 port, verified against
// the untouched CPU function directly at a measured tolerance (this
// project's own three-way/GPU-vs-CPU-twin discipline, `DECISIONS.md` §7a/7c).
//
// `evap_main`: per-cell evaporation into `w`, in place -- also folds in the
// non-wrap ocean-boundary humidity reset (CPU's own `if !wrap_x {...}`
// block), since that only touches border cells based on their OWN
// just-evaporated value, no neighbour read, safe to fuse into the same pass.
// `advect_main`: `w2[i] = bil_c(w, x - wx[i], y - wy[i])` -- a gather from
// the PREVIOUS pass's complete `w`, written to a separate buffer (never
// in-place, avoids any read-after-write hazard across invocations).
// `deposit_main`: reads `w2` + static `eh`, writes `w` (next iteration's
// input) and accumulates `rain`.

struct WeatherParams {
    ww: u32,
    wh: u32,
    wrap_x: u32,
    bulk_evap: u32,
    sea: f32,
    ocean_hum: f32,
    evap_c: f32,
    ocean_c: f32,
    rain_k: f32,
    dry: f32,
    step: f32,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> params: WeatherParams;
@group(0) @binding(1) var<storage, read> eh: array<f32>;
@group(0) @binding(2) var<storage, read> tc: array<f32>;
@group(0) @binding(3) var<storage, read> sst_evap: array<f32>;
@group(0) @binding(4) var<storage, read> wx: array<f32>;
@group(0) @binding(5) var<storage, read> wy: array<f32>;
@group(0) @binding(6) var<storage, read_write> w: array<f32>;
@group(0) @binding(7) var<storage, read_write> w2: array<f32>;
@group(0) @binding(8) var<storage, read_write> rain: array<f32>;

fn sat_cap(t: f32) -> f32 {
    return 0.16 * exp(0.058 * t);
}

// `bilC()` (reference HTML line 5537) on the coarse weather grid --
// x-wrap optional, y always clamps. Reads whichever storage array is
// passed via the two array-of-f32 params below (WGSL has no function
// pointers/generics over storage bindings, so this takes explicit
// `array<f32>` bindings by value is not possible either -- inlined at
// each call site instead, matching `box_h_main`/`box_v_main`'s own
// per-kernel-inlined-logic convention in `gpu_gauss_blur.wgsl`).
fn bil_sample(idx_base: u32, arr_is_w: bool, fx_in: f32, fy_in: f32) -> f32 {
    let ww = i32(params.ww);
    let wh = i32(params.wh);
    var fx = fx_in;
    if params.wrap_x != 0u {
        fx = (fx % f32(ww) + f32(ww)) % f32(ww);
    } else {
        fx = clamp(fx, 0.0, f32(ww) - 1.0);
    }
    let fy = clamp(fy_in, 0.0, f32(wh) - 1.0);
    let x0 = i32(floor(fx));
    let y0 = i32(floor(fy));
    var x1 = x0 + 1;
    if x1 >= ww {
        if params.wrap_x != 0u {
            x1 = 0;
        } else {
            x1 = ww - 1;
        }
    }
    let y1 = min(y0 + 1, wh - 1);
    let tx = fx - f32(x0);
    let ty = fy - f32(y0);
    let i00 = u32(y0 * ww + x0);
    let i01 = u32(y0 * ww + x1);
    let i10 = u32(y1 * ww + x0);
    let i11 = u32(y1 * ww + x1);
    var v00: f32; var v01: f32; var v10: f32; var v11: f32;
    if arr_is_w {
        v00 = w[i00]; v01 = w[i01]; v10 = w[i10]; v11 = w[i11];
    } else {
        v00 = eh[i00]; v01 = eh[i01]; v10 = eh[i10]; v11 = eh[i11];
    }
    let top = v00 * (1.0 - tx) + v01 * tx;
    let bot = v10 * (1.0 - tx) + v11 * tx;
    return top * (1.0 - ty) + bot * ty;
}

@compute @workgroup_size(8, 8, 1)
fn evap_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.ww || gid.y >= params.wh {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let i = u32(y) * params.ww + u32(x);

    if eh[i] < params.sea {
        let cap = max(params.ocean_hum, sat_cap(tc[i])) * 1.2;
        var e = params.evap_c * sst_evap[i] * params.ocean_c;
        if params.bulk_evap != 0u {
            let u = length(vec2<f32>(wx[i], wy[i])) / params.step;
            e *= (0.4 + 0.6 * u) * max(0.0, 1.0 - w[i] / cap);
        }
        w[i] = min(w[i] + e, cap);
    }

    // Non-wrap ocean-boundary humidity reset (CPU's `if !wrap_x {...}`) --
    // border cells only, based on this cell's own just-evaporated `w[i]`
    // and fixed wind direction, no neighbour read.
    if params.wrap_x == 0u {
        let th = 0.15 * params.step;
        let wwi = i32(params.ww);
        let whi = i32(params.wh);
        if x == 0 && wx[i] > th && w[i] < params.ocean_hum {
            w[i] = params.ocean_hum;
        }
        if x == wwi - 1 && wx[i] < -th && w[i] < params.ocean_hum {
            w[i] = params.ocean_hum;
        }
        if y == 0 && wy[i] > th && w[i] < params.ocean_hum {
            w[i] = params.ocean_hum;
        }
        if y == whi - 1 && wy[i] < -th && w[i] < params.ocean_hum {
            w[i] = params.ocean_hum;
        }
    }
}

@compute @workgroup_size(8, 8, 1)
fn advect_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.ww || gid.y >= params.wh {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let i = u32(y) * params.ww + u32(x);
    w2[i] = bil_sample(i, true, f32(x) - wx[i], f32(y) - wy[i]);
}

@compute @workgroup_size(8, 8, 1)
fn deposit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.ww || gid.y >= params.wh {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let i = u32(y) * params.ww + u32(x);

    if eh[i] < params.sea {
        w[i] = w2[i];
        return;
    }
    let ux = wx[i];
    let uy = wy[i];
    var l = length(vec2<f32>(ux, uy));
    if l == 0.0 {
        l = 1.0;
    }
    let eh_up = bil_sample(i, false, f32(x) - ux / l, f32(y) - uy / l);
    let oro = w2[i] * max(0.0, eh[i] - eh_up) * params.rain_k * 9.0;
    let excess = max(0.0, w2[i] - sat_cap(tc[i]));
    let conv = w2[i] * 0.05;
    var pr = (oro + excess * 0.6 + conv) * params.dry;
    pr = min(pr, w2[i]);
    w[i] = w2[i] - pr;
    rain[i] = rain[i] * 0.55 + pr * 0.45;
}
