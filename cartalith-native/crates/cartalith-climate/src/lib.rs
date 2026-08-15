//! temperature, wind, rainfall
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

/// `smoothstep()` (reference HTML line 7569).
fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let denom = b - a;
    let denom = if denom == 0.0 { 1e-6 } else { denom };
    let t = ((x - a) / denom).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// This file's own Earth-default axial tilt (reference HTML line 5099) —
/// `insolationContrastK` is normalized to `1.0` here so `equatorTemp`/
/// `poleTemp` keep their exact existing meaning at Earth's default tilt.
const OBLIQUITY_REF_DEG: f64 = 23.4;

fn obliquity_s2(tilt_deg: f64) -> f64 {
    let s = (tilt_deg * std::f64::consts::PI / 180.0).sin();
    3.0 * s * s - 2.0
}

/// `insolationContrastK()` (reference HTML line 5101): the P2
/// energy-balance-model approximation of annual-mean insolation contrast
/// vs. axial tilt (North & Coakley 1979), normalized to `1.0` at Earth's
/// own 23.4° default.
fn insolation_contrast_k(tilt_deg: f64) -> f64 {
    obliquity_s2(tilt_deg) / obliquity_s2(OBLIQUITY_REF_DEG)
}

/// `rotationContrastK()` (reference HTML line 5105): a slower rotator's
/// weaker Coriolis constraint flattens the pole-equator gradient.
fn rotation_contrast_k(rotation_hours: f64) -> f64 {
    (24.0 / rotation_hours.max(1.0)).powf(0.25)
}

/// `climEffectiveEquatorTemp()` (reference HTML line 5115): the one place
/// `equatorTemp`/`poleTemp`'s *contrast* is scaled by planet params —
/// `poleTemp` stays the fixed anchor, only the spread above it scales.
fn clim_effective_equator_temp(equator_temp: f64, pole_temp: f64, tilt_deg: f64, rotation_hours: f64) -> f64 {
    pole_temp + (equator_temp - pole_temp) * insolation_contrast_k(tilt_deg) * rotation_contrast_k(rotation_hours)
}

/// `metersPerUnit()` (reference HTML line 4951): converts the `[0,1]`
/// height field into real meters, anchored so `1.0 - seaLevel` (the
/// above-sea fraction of the field's range) maps to `peakM`.
fn meters_per_unit(peak_m: f64, sea_level: f64) -> f64 {
    let denom = 1.0 - sea_level;
    let denom = if denom == 0.0 { 1e-6 } else { denom };
    peak_m / denom
}

/// `latAt()` (reference HTML line 4965): world mode spans the whole
/// planet pole-to-pole; a region uses the configured `latN`/`latS` band.
fn lat_at(y: usize, gh: usize, world: bool, lat_n: f64, lat_s: f64) -> f64 {
    let denom = (gh.max(2) - 1) as f64;
    if world {
        90.0 - (y as f64 / denom) * 180.0
    } else {
        lat_n + (y as f64 / denom) * (lat_s - lat_n)
    }
}

/// Max additional cooling (°C) at full ice cover, strength 1 (reference
/// HTML line 5054).
const ALB_COOL: f64 = 9.0;

/// `applyCryosphereAlbedo()` (reference HTML lines 5055-5062): a 6-pass
/// damped relaxation toward a colder equilibrium wherever ice forms.
/// No-op when `k` isn't positive (`state.climate.albedo` defaults to `0`
/// — the app's own default path never runs this loop).
///
/// `temp` stays `f32` throughout, matching JS's `Float32Array` in place —
/// each pass's `temp[i]=...` rounds to `f32` on store, and the *next*
/// pass's `smoothstep(1,-6,temp[i])` reads that rounded value back, not
/// a full-precision running `f64`. `base` is captured once before any
/// pass runs (JS: `new Float32Array(temp)`, a snapshot) and never
/// re-rounded.
fn apply_cryosphere_albedo(temp: &mut [f32], k: f64) {
    // `!(k > 0.0)`, not `k <= 0.0`: JS's `!(k>0)` is true for NaN too
    // (NaN>0 is false), matching this project's NaN-safety convention
    // (cartalith-rust-conventions) of treating NaN as "off" rather than
    // panicking or silently running with it. `k <= 0.0` would flip that
    // — NaN<=0.0 is also false, so it would NOT early-return.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(k > 0.0) {
        return;
    }
    let base: Vec<f32> = temp.to_vec();
    for _ in 0..6 {
        for i in 0..temp.len() {
            let ice = smoothstep(1.0, -6.0, temp[i] as f64);
            let v = temp[i] as f64 * 0.5 + (base[i] as f64 - k * ALB_COOL * ice) * 0.5;
            temp[i] = v as f32;
        }
    }
}

/// The climate/planet parameters `computeTemperature()` reads off
/// `state.climate`/`state.planet`/`state.seaLevel` — bundled since
/// they're the formula's real tuning knobs, the same reasoning
/// `HeightParams` in `cartalith-terrain` uses.
pub struct ClimateParams {
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    pub pole_temp: f64,
    pub equator_temp: f64,
    pub tilt_deg: f64,
    pub rotation_hours: f64,
    pub lapse_rate: f64,
    pub g: f64,
    pub sea_level: f64,
    pub peak_m: f64,
    pub albedo_k: f64,
}

/// `computeTemperature()` (reference HTML lines 5119-5143), CPU path only
/// — GPU is unavailable headless and JS falls back to this exact code
/// when it is, or when a geoid/albedo is active (neither the shader
/// supports). Latitude-band base temperature, cooled by altitude above
/// sea level (lapse rate scaled by gravity), then optionally relaxed
/// toward a colder cryosphere equilibrium.
///
/// `geo_field` (the geoid's per-cell sea-level offset) is `None` here —
/// `buildGeoid` itself isn't ported yet, but `state.planet.geoid.enabled`
/// defaults to `false`, where JS's own `geoAt()` always returns `0`
/// anyway, so `None` matches the app's own default path exactly rather
/// than approximating it.
pub fn compute_temperature(
    gw: usize,
    gh: usize,
    field: &[f32],
    geo_field: Option<&[f32]>,
    p: &ClimateParams,
) -> Vec<f32> {
    let mpu = meters_per_unit(p.peak_m, p.sea_level);
    let eq_eff = clim_effective_equator_temp(p.equator_temp, p.pole_temp, p.tilt_deg, p.rotation_hours);
    let mut temp = vec![0f32; gw * gh];
    for y in 0..gh {
        let lat = lat_at(y, gh, p.world, p.lat_n, p.lat_s) * std::f64::consts::PI / 180.0;
        let t_sea = p.pole_temp + (eq_eff - p.pole_temp) * lat.cos().max(0.0);
        for x in 0..gw {
            let i = y * gw + x;
            let geo = geo_field.map_or(0.0, |g| g[i] as f64);
            let above_sea = ((field[i] as f64 - geo - p.sea_level).max(0.0)) * mpu;
            temp[i] = (t_sea - p.lapse_rate * p.g * (above_sea / 1000.0)) as f32;
        }
    }
    apply_cryosphere_albedo(&mut temp, p.albedo_k);
    temp
}

/// Mirrors JS `Math.round` (ties toward `+Infinity`) — same trap
/// `cartalith-terrain::js_round` exists for; duplicated here (one line)
/// rather than adding a dependency purely for it.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `circulationCells()` (reference HTML lines 5299-5302): planet
/// rotation/size/gravity set how many latitude wind bands the planet
/// gets (a faster/smaller/lighter-gravity planet has stronger Coriolis
/// constraint → more, narrower cells). JS's `p.rotationHours||24` etc.
/// fall back on `0` (falsy in JS); mirrored here for `0.0` specifically.
fn circulation_cells(rotation_hours: f64, radius_rel: f64, g: f64) -> i32 {
    let rh = if rotation_hours != 0.0 { rotation_hours } else { 24.0 };
    let rr = if radius_rel != 0.0 { radius_rel } else { 1.0 };
    let gg = if g != 0.0 { g } else { 1.0 };
    let rel = (24.0 / rh) * rr / gg.sqrt();
    (js_round(3.0 * rel.sqrt()) as i32).clamp(1, 8)
}

/// `satCap()` (reference HTML line 5295): Clausius-Clapeyron-ish warm-air
/// moisture-holding capacity.
fn sat_cap(t: f64) -> f64 {
    0.16 * (0.058 * t).exp()
}

/// `bilC()` (reference HTML line 5537): bilinear sample on the coarse
/// weather grid, with optional x-wrap.
fn bil_c(a: &[f32], fx: f64, fy: f64, ww: usize, wh: usize, wrap_x: bool) -> f64 {
    let fx = if wrap_x {
        ((fx % ww as f64) + ww as f64) % ww as f64
    } else {
        fx.clamp(0.0, ww as f64 - 1.0)
    };
    let fy = fy.clamp(0.0, wh as f64 - 1.0);
    let x0 = fx as i64;
    let y0 = fy as i64;
    let x1 = if x0 + 1 >= ww as i64 {
        if wrap_x { 0 } else { ww as i64 - 1 }
    } else {
        x0 + 1
    };
    let y1 = (y0 + 1).min(wh as i64 - 1);
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let (x0, x1, y0, y1) = (x0 as usize, x1 as usize, y0 as usize, y1 as usize);
    let top = a[y0 * ww + x0] as f64 * (1.0 - tx) + a[y0 * ww + x1] as f64 * tx;
    let bot = a[y1 * ww + x0] as f64 * (1.0 - tx) + a[y1 * ww + x1] as f64 * tx;
    top * (1.0 - ty) + bot * ty
}

/// `sampleArr()` (reference HTML line 10242): bilinear sample on the
/// full-resolution grid, clamp-only (no wrap — used for reading the
/// height field at a coarse weather-grid position).
fn sample_arr(a: &[f32], fx: f64, fy: f64, gw: usize, gh: usize) -> f64 {
    let fx = fx.clamp(0.0, gw as f64 - 1.0);
    let fy = fy.clamp(0.0, gh as f64 - 1.0);
    let x0 = fx as i64;
    let y0 = fy as i64;
    let x1 = if x0 < gw as i64 - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < gh as i64 - 1 { y0 + 1 } else { y0 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let (x0, x1, y0, y1) = (x0 as usize, x1 as usize, y0 as usize, y1 as usize);
    let top = a[y0 * gw + x0] as f64 * (1.0 - tx) + a[y0 * gw + x1] as f64 * tx;
    let bot = a[y1 * gw + x0] as f64 * (1.0 - tx) + a[y1 * gw + x1] as f64 * tx;
    top * (1.0 - ty) + bot * ty
}

/// `blurCoarse()` (reference HTML line 5543): separable 3-tap box blur —
/// kills row/column banding without flattening large rain shadows the
/// way a wider Gaussian would. Mutates `a` in place, `passes` times.
fn blur_coarse(a: &mut [f32], ww: usize, wh: usize, wrap_x: bool, passes: i32) {
    let mut t = vec![0f32; a.len()];
    for _ in 0..passes {
        for y in 0..wh {
            for x in 0..ww {
                let xl = if wrap_x {
                    (x + ww - 1) % ww
                } else if x > 0 {
                    x - 1
                } else {
                    0
                };
                let xr = if wrap_x {
                    (x + 1) % ww
                } else if x < ww - 1 {
                    x + 1
                } else {
                    ww - 1
                };
                t[y * ww + x] =
                    ((a[y * ww + xl] as f64 + a[y * ww + x] as f64 + a[y * ww + xr] as f64) / 3.0) as f32;
            }
        }
        for y in 0..wh {
            let yu = if y > 0 { y - 1 } else { 0 };
            let yd = if y < wh - 1 { y + 1 } else { wh - 1 };
            for x in 0..ww {
                a[y * ww + x] =
                    ((t[yu * ww + x] as f64 + t[y * ww + x] as f64 + t[yd * ww + x] as f64) / 3.0) as f32;
            }
        }
    }
}

/// Tuning knobs `deflectFlow()`'s own `opts` bag takes (reference HTML
/// line 5315-5319) — JS defaults noted per field; both call sites this
/// port has (`build_wind`'s terrain deflection) override them.
pub struct DeflectFlowParams {
    pub strength: f64,
    pub k1: f64,
    pub k2: f64,
    pub gap_k: f64,
    pub iterations: i32,
    pub block_blur: i32,
}

/// `deflectFlow()` (reference HTML lines 5315-5357): the component of
/// `(u,v)` pointing INTO rising `block` is reduced and redirected
/// tangentially along the block field's local contour, iterated with
/// light blending so the deflection propagates upstream of a ridge/
/// coastline rather than only appearing on top of it (linearised
/// hill-flow theory, Jackson & Hunt 1975). Gap/strait acceleration comes
/// from the block field's own Laplacian. Pure; shared by `build_wind`'s
/// terrain deflection and (not yet ported) `computeOceanCurrent`'s hard
/// coastline.
///
/// Every intermediate value here is `f64` arithmetic over `f32`-stored
/// inputs, rounded to `f32` only at each `Float32Array` write point
/// (`nu`/`nv`/`u`/`v`) — matching JS's own read-as-full-precision,
/// write-rounds-to-f32 typed-array semantics, the same discipline
/// `stamp_one_volcano` documents for the same reason.
pub fn deflect_flow(
    u0: &[f32],
    v0: &[f32],
    block0: &[f32],
    ww: usize,
    wh: usize,
    wrap_x: bool,
    p: &DeflectFlowParams,
) -> (Vec<f32>, Vec<f32>) {
    let n = ww * wh;
    let k1 = p.k1 * p.strength;
    let k2 = p.k2 * p.strength;
    let gap_k = p.gap_k * p.strength;

    let mut u: Vec<f32> = u0.to_vec();
    let mut v: Vec<f32> = v0.to_vec();
    let mut b: Vec<f32> = block0.to_vec();
    blur_coarse(&mut b, ww, wh, wrap_x, p.block_blur);

    let mut bgx = vec![0f32; n];
    let mut bgy = vec![0f32; n];
    let mut lap = vec![0f32; n];
    for y in 0..wh {
        for x in 0..ww {
            let i = y * ww + x;
            let xl = if wrap_x { (x + ww - 1) % ww } else if x > 0 { x - 1 } else { 0 };
            let xr = if wrap_x { (x + 1) % ww } else if x + 1 < ww { x + 1 } else { ww - 1 };
            let yu = if y > 0 { y - 1 } else { 0 };
            let yd = if y + 1 < wh { y + 1 } else { wh - 1 };
            bgx[i] = ((b[y * ww + xr] as f64 - b[y * ww + xl] as f64) * 0.5) as f32;
            bgy[i] = ((b[yd * ww + x] as f64 - b[yu * ww + x] as f64) * 0.5) as f32;
            lap[i] = (b[y * ww + xl] as f64 + b[y * ww + xr] as f64 + b[yu * ww + x] as f64 + b[yd * ww + x] as f64
                - 4.0 * b[i] as f64) as f32;
        }
    }

    if p.strength > 0.0 {
        for _ in 0..p.iterations {
            let mut nu = vec![0f32; n];
            let mut nv = vec![0f32; n];
            for i in 0..n {
                let dot = u[i] as f64 * bgx[i] as f64 + v[i] as f64 * bgy[i] as f64;
                if dot > 0.0 {
                    let gn = (bgx[i] as f64).hypot(bgy[i] as f64) + 1e-6;
                    let nx = bgx[i] as f64 / gn;
                    let ny = bgy[i] as f64 / gn;
                    let rem = (k1 * dot).min(0.9 * dot);
                    let mut uu = u[i] as f64 - rem * nx;
                    let mut vv = v[i] as f64 - rem * ny;
                    let px = -ny;
                    let py = nx;
                    let sign = if u[i] as f64 * px + v[i] as f64 * py >= 0.0 { 1.0 } else { -1.0 };
                    let tang = k2 * dot;
                    uu += sign * tang * px;
                    vv += sign * tang * py;
                    nu[i] = uu as f32;
                    nv[i] = vv as f32;
                } else {
                    nu[i] = u[i];
                    nv[i] = v[i];
                }
            }
            let mut bu = nu.clone();
            let mut bv = nv.clone();
            blur_coarse(&mut bu, ww, wh, wrap_x, 1);
            blur_coarse(&mut bv, ww, wh, wrap_x, 1);
            for i in 0..n {
                u[i] = (nu[i] as f64 * 0.7 + bu[i] as f64 * 0.3) as f32;
                v[i] = (nv[i] as f64 * 0.7 + bv[i] as f64 * 0.3) as f32;
            }
        }
    }

    for i in 0..n {
        let m = (1.0 + gap_k * lap[i] as f64 * 3.0).clamp(0.45, 2.4);
        let sp = (u[i] as f64).hypot(v[i] as f64);
        if sp > 1e-6 {
            let t = sp * m;
            u[i] = (u[i] as f64 / sp * t) as f32;
            v[i] = (v[i] as f64 / sp * t) as f32;
        }
    }

    (u, v)
}

/// `buildWind()` (reference HTML lines 5464-5535). Prevailing latitude-band
/// winds (band count from `circulation_cells`), an optional pressure-
/// gradient + Coriolis perturbation from a smoothed temperature proxy
/// (warm = thermal low) when `tc` is supplied and `press_k > 0`, then —
/// when `elev` is supplied — `deflectFlow`'s terrain-deflection block
/// (mountains block/split flow, gaps/straits accelerate it) plus a
/// high-altitude thin-air damping term. `elev`/`sea_level` mirror JS's
/// `opts.elev`/`state.seaLevel`; every caller in this port supplies both
/// (`simulate_weather`'s own `eh` coarse elevation array), matching JS's
/// own v1.78 "no longer a toggle" default -- the only caller in the
/// reference that omits `opts.elev` is `currentWindField` (a debug-view
/// helper this port hasn't ported).
#[allow(clippy::too_many_arguments)]
fn build_wind(
    ww: usize,
    wh: usize,
    step: f64,
    tc: Option<&[f32]>,
    decl: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    wind_manual: bool,
    wind_dir_deg: f64,
    press_k: f64,
    rotation_hours: f64,
    elev: Option<(&[f32], f64)>,
) -> (Vec<f32>, Vec<f32>) {
    let n = ww * wh;
    let mut wx = vec![0f32; n];
    let mut wy = vec![0f32; n];
    let lat_of = |y: usize| -> f64 {
        if world {
            90.0 - (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * 180.0
        } else {
            lat_n + (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * (lat_s - lat_n)
        }
    };
    if !world && wind_manual {
        let rad = wind_dir_deg * std::f64::consts::PI / 180.0;
        let ux = (rad.cos() * step) as f32;
        let uy = (rad.sin() * step) as f32;
        wx.fill(ux);
        wy.fill(uy);
    } else {
        let cells = circulation_cells(rotation_hours, 1.0, 1.0);
        let band_w = 90.0 / cells as f64;
        for y in 0..wh {
            let lat = lat_of(y);
            let a = (lat - decl).abs();
            let band = ((a / band_w).floor() as i32).min(cells - 1);
            let zx: f64 = if band % 2 == 0 { -1.0 } else { 1.0 };
            let tilt = if band == cells - 1 && band % 2 == 0 && cells > 1 {
                0.15
            } else {
                0.35
            };
            let my = if band % 2 == 0 {
                (if lat > 0.0 { 1.0 } else { -1.0 }) * tilt
            } else {
                (if lat > 0.0 { -1.0 } else { 1.0 }) * tilt
            };
            let l = zx.hypot(my);
            let l = if l == 0.0 { 1.0 } else { l };
            let ux = (zx / l * step) as f32;
            let uy = (my / l * step) as f32;
            for x in 0..ww {
                wx[y * ww + x] = ux;
                wy[y * ww + x] = uy;
            }
        }
    }
    let kp = press_k;
    if let Some(tc) = tc
        && kp > 0.0
    {
        let wrap_x = world;
        let p = cartalith_terrain::gauss_blur(tc, 2.0, ww, wh, wrap_x);
        let mut gx = vec![0f64; n];
        let mut gy = vec![0f64; n];
        let omega = 24.0 / if rotation_hours != 0.0 { rotation_hours } else { 24.0 };
        let mut mx = 1e-9f64;
        for y in 0..wh {
            for x in 0..ww {
                let i = y * ww + x;
                let xm = if wrap_x { (x + ww - 1) % ww } else { x.saturating_sub(1) };
                let xp = if wrap_x { (x + 1) % ww } else { (x + 1).min(ww - 1) };
                let ym = y.saturating_sub(1);
                let yp = (y + 1).min(wh - 1);
                let d_pdx = -(p[y * ww + xp] as f64 - p[y * ww + xm] as f64) * 0.5;
                let d_pdy_s = -(p[yp * ww + x] as f64 - p[ym * ww + x] as f64) * 0.5;
                let lat_r = lat_of(y) * std::f64::consts::PI / 180.0;
                let f = lat_r.sin() * omega;
                let geo_w = (((lat_r.abs() * 180.0 / std::f64::consts::PI) - 5.0) / 10.0).clamp(0.0, 1.0);
                let (mut ux, mut uy) = (0.0f64, 0.0f64);
                if geo_w > 0.0 {
                    let inv = (if f < 0.0 { -1.0 } else { 1.0 }) / f.abs().max(0.25);
                    ux = inv * d_pdy_s * geo_w;
                    uy = -inv * d_pdx * geo_w;
                }
                let dgw = (1.0 - geo_w) * 1.5;
                ux += -d_pdx * dgw;
                uy += -d_pdy_s * dgw;
                gx[i] = ux;
                gy[i] = uy;
                let m = ux.hypot(uy);
                if m > mx {
                    mx = m;
                }
            }
        }
        let sc = (step * 0.8) / mx * kp;
        let cap = step * 1.8;
        for i in 0..n {
            let mut ux = wx[i] as f64 + gx[i] * sc;
            let mut uy = wy[i] as f64 + gy[i] * sc;
            let m = ux.hypot(uy);
            if m > cap {
                ux *= cap / m;
                uy *= cap / m;
            }
            wx[i] = ux as f32;
            wy[i] = uy as f32;
        }
    }
    // deflectFlow terrain-deflection block (reference HTML lines 5521-5535):
    // mountains block/split flow, gaps/straits accelerate it -- the one
    // physical mechanism the pressure/Coriolis step above never modeled
    // (it reacts to temperature, not real elevation).
    if let Some((elev, sea)) = elev {
        let mut block = vec![0f32; n];
        for i in 0..n {
            let h = elev[i] as f64;
            let land = (((h - (sea - 0.02)) / 0.04).clamp(0.0, 1.0)) * 0.12;
            let mtn = ((h - (sea - 0.03)) / 0.43).clamp(0.0, 1.0);
            block[i] = (land + mtn).min(1.0) as f32;
        }
        let deflect_params = DeflectFlowParams { strength: 1.0, k1: 0.6, k2: 0.65, gap_k: 0.32, iterations: 16, block_blur: 2 };
        let (du, dv) = deflect_flow(&wx, &wy, &block, ww, wh, world, &deflect_params);
        for i in 0..n {
            // elevation-band damping: thin high-altitude air slows/
            // simplifies near-surface flow.
            let damp = 0.55 * (((elev[i] as f64 - (sea + 0.30)) / 0.32).clamp(0.0, 1.0));
            wx[i] = (du[i] as f64 * (1.0 - damp)) as f32;
            wy[i] = (dv[i] as f64 * (1.0 - damp)) as f32;
        }
    }
    (wx, wy)
}

/// Tuning knobs `computeOceanCurrent()`'s own `opts` bag takes (reference
/// HTML lines 5368-5369) — this port's one call site (`ocean_sst_anomaly`)
/// passes JS's own defaults (an empty `{}` in JS resolves to
/// `gap_k: 0.4, iterations: 20, bend_k: 0.9, western: true`).
pub struct OceanCurrentParams {
    pub gap_k: f64,
    pub iterations: i32,
    pub bend_k: f64,
    pub western: bool,
}

/// A 2D ocean-current vector field (`u`/`v` zero on land) plus the ocean
/// mask `computeOceanCurrent` derived it against.
pub struct OceanCurrentResult {
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub ocean: Vec<u8>,
}

/// `computeOceanCurrent()` (reference HTML lines 5368-5462): a genuine 2D
/// ocean-current vector field — Ekman-rotated (~25° right of wind in the
/// N hemisphere, left in the S; Ekman 1905) from the (terrain-deflected)
/// wind, run through `deflect_flow` again against a HARD coastline (vs.
/// wind's soft elevation ramp), then a continental-shelf friction term
/// (shallow water damps flow) and a western-intensification heuristic
/// (subtropical gyres pile transport on a basin's western edge — Sverdrup
/// 1947 / Stommel 1948). The heuristic is a distance-to-coast proxy, NOT a
/// solved beta-plane model — disclosed, not oversold, matching the
/// reference's own method notes. `u`/`v` are zero on land.
#[allow(clippy::too_many_arguments)]
pub fn compute_ocean_current(
    wx: &[f32],
    wy: &[f32],
    elev_c: &[f32],
    ww: usize,
    wh: usize,
    wrap_x: bool,
    sea: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    p: &OceanCurrentParams,
) -> OceanCurrentResult {
    let lat_of = |y: usize| -> f64 {
        if world {
            90.0 - (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * 180.0
        } else {
            lat_n + (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * (lat_s - lat_n)
        }
    };
    let n = ww * wh;
    let ang = 25.0 * std::f64::consts::PI / 180.0;
    let mut cu = vec![0f32; n];
    let mut cv = vec![0f32; n];
    let mut ocean = vec![0u8; n];
    let mut block = vec![0f32; n];
    for y in 0..wh {
        let lat = lat_of(y);
        let sgn = if lat >= 0.0 { 1.0 } else { -1.0 };
        let c_a = (ang * sgn).cos();
        let s_a = (ang * sgn).sin();
        for x in 0..ww {
            let i = y * ww + x;
            let is_ocean = (elev_c[i] as f64) < sea;
            ocean[i] = if is_ocean { 1 } else { 0 };
            block[i] = if is_ocean { 0.0 } else { 1.0 };
            if !is_ocean {
                continue;
            }
            let ux = wx[i] as f64;
            let uy = wy[i] as f64;
            cu[i] = ((ux * c_a - uy * s_a) * 0.55) as f32;
            cv[i] = ((ux * s_a + uy * c_a) * 0.55) as f32;
        }
    }

    let deflect_params =
        DeflectFlowParams { strength: 1.0, k1: 0.7, k2: 0.85, gap_k: p.gap_k, iterations: p.iterations, block_blur: 6 };
    let (mut u_c, mut v_c) = deflect_flow(&cu, &cv, &block, ww, wh, wrap_x, &deflect_params);

    for i in 0..n {
        if ocean[i] == 0 {
            u_c[i] = 0.0;
            v_c[i] = 0.0;
            continue;
        }
        let depth = sea - elev_c[i] as f64;
        let shelf = (depth / 0.12).clamp(0.0, 1.0);
        u_c[i] = (u_c[i] as f64 * shelf) as f32;
        v_c[i] = (v_c[i] as f64 * shelf) as f32;
    }

    // Western-intensification heuristic: per-row west/east coast-distance
    // scan, a speed boost on whichever side is closer to open ocean, and a
    // poleward/equatorward bend derived from the same proximity terms.
    if p.western {
        let search_r = (ww as f64 * 0.28).max(10.0);
        for y in 0..wh {
            let row_off = y * ww;
            let mut west_dist = vec![0f32; ww];
            let mut acc = search_r + 1.0;
            let passes = if wrap_x { 2 } else { 1 };
            for _ in 0..passes {
                #[allow(clippy::needless_range_loop)]
                for x in 0..ww {
                    let i = row_off + x;
                    acc = if (elev_c[i] as f64) >= sea { 0.0 } else { (acc + 1.0).min(search_r + 1.0) };
                    west_dist[x] = acc as f32;
                }
            }
            let lat_here = lat_of(y);
            let lat_next = lat_of((y + 1).min(wh - 1));
            let pole_sign = if lat_next.abs() >= lat_here.abs() { 1.0 } else { -1.0 };
            #[allow(clippy::needless_range_loop)]
            for x in 0..ww {
                let i = row_off + x;
                if ocean[i] == 0 {
                    continue;
                }
                let mut east_dist = search_r + 1.0;
                let mut s = 1usize;
                while (s as f64) <= search_r {
                    let xx = if wrap_x { (x + s) % ww } else { x + s };
                    if !wrap_x && xx >= ww {
                        break;
                    }
                    if (elev_c[row_off + xx] as f64) >= sea {
                        east_dist = s as f64;
                        break;
                    }
                    s += 1;
                }
                let wd = west_dist[x] as f64;
                if wd < east_dist {
                    let boost =
                        1.0 + 0.9 * (1.0 - wd / search_r).clamp(0.0, 1.0) * (east_dist / search_r).clamp(0.0, 1.0);
                    u_c[i] = (u_c[i] as f64 * boost) as f32;
                    v_c[i] = (v_c[i] as f64 * boost) as f32;
                }
                let w_bend = (1.0 - wd / search_r).clamp(0.0, 1.0) * (east_dist / search_r).clamp(0.0, 1.0);
                let e_bend = (1.0 - east_dist / search_r).clamp(0.0, 1.0) * (wd / search_r).clamp(0.0, 1.0);
                if w_bend > 0.0 || e_bend > 0.0 {
                    let sp = (u_c[i] as f64).hypot(v_c[i] as f64);
                    v_c[i] = (v_c[i] as f64 + pole_sign * p.bend_k * (w_bend - 0.45 * e_bend) * sp) as f32;
                }
            }
        }
    }

    OceanCurrentResult { u: u_c, v: v_c, ocean }
}

/// `oceanSSTAnomaly()` (reference HTML lines 5246-5268): wind-driven
/// ocean-current SST anomaly on the coarse weather grid — poleward
/// surface currents carry warm water poleward (warm anomaly), equatorward
/// flow brings cold upwelling (cold anomaly, Benguela/Peru → Atacama/
/// Namib). Returned as a coarse field so `simulate_weather` can fold it
/// into sea temperature BEFORE building winds (closing the real loop:
/// currents → SST → pressure/evaporation → winds → rainfall) and so
/// `apply_ocean_currents` can also use it as a post-hoc correction.
/// `geoidField` omitted, matching `compute_temperature`'s own reasoning —
/// `state.planet.geoid.enabled` defaults `false`.
#[allow(clippy::too_many_arguments)]
pub fn ocean_sst_anomaly(
    gw: usize,
    gh: usize,
    field: &[f32],
    ww: usize,
    wh: usize,
    wrap_x: bool,
    step: f64,
    sea: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    equator_temp: f64,
    pole_temp: f64,
    tilt_deg: f64,
    rotation_hours: f64,
    wind_manual: bool,
    wind_dir_deg: f64,
    press_k: f64,
    current_k: f64,
) -> Vec<f32> {
    let lat_of = |y: usize| -> f64 {
        if world {
            90.0 - (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * 180.0
        } else {
            lat_n + (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * (lat_s - lat_n)
        }
    };
    let nc = ww * wh;
    let eq_eff = clim_effective_equator_temp(equator_temp, pole_temp, tilt_deg, rotation_hours);
    let t_sea_at =
        |lat: f64| -> f64 { pole_temp + (eq_eff - pole_temp) * (lat * std::f64::consts::PI / 180.0).cos().max(0.0) };

    let mut tc = vec![0f32; nc];
    for y in 0..wh {
        let ts = t_sea_at(lat_of(y)) as f32;
        for x in 0..ww {
            tc[y * ww + x] = ts;
        }
    }

    let mut elev_c = vec![0f32; nc];
    for y in 0..wh {
        for x in 0..ww {
            let fx = x as f64 / (ww as f64 - 1.0) * (gw as f64 - 1.0);
            let fy = y as f64 / (wh as f64 - 1.0) * (gh as f64 - 1.0);
            elev_c[y * ww + x] = sample_arr(field, fx, fy, gw, gh) as f32;
        }
    }

    let (wx, wy) = build_wind(
        ww,
        wh,
        step,
        Some(&tc),
        0.0,
        world,
        lat_n,
        lat_s,
        wind_manual,
        wind_dir_deg,
        press_k,
        rotation_hours,
        Some((&elev_c, sea)),
    );

    let cur_params = OceanCurrentParams { gap_k: 0.4, iterations: 20, bend_k: 0.9, western: true };
    let cur = compute_ocean_current(&wx, &wy, &elev_c, ww, wh, wrap_x, sea, world, lat_n, lat_s, &cur_params);

    let mut sst = vec![0f32; nc];
    for y in 0..wh {
        let lat = lat_of(y);
        let a_pole = lat.abs();
        let d_warm = t_sea_at((a_pole - 12.0).max(0.0)) - t_sea_at(lat);
        for x in 0..ww {
            let i = y * ww + x;
            if cur.ocean[i] == 0 {
                sst[i] = 0.0;
                continue;
            }
            let vp = if lat >= 0.0 { -(cur.v[i] as f64) } else { cur.v[i] as f64 };
            let a = (current_k * (vp / step) * d_warm).clamp(-8.0, 8.0);
            sst[i] = a as f32;
        }
    }

    cartalith_terrain::gauss_blur(&sst, js_round(ww as f64 * 0.04).max(2.0), ww, wh, wrap_x)
}

/// `applyOceanCurrents()` (reference HTML lines 5270-5288): the post-hoc
/// half of ocean-current coupling — folds `ocean_sst_anomaly`'s field
/// into `temperature` directly over ocean, and into `temperature`/
/// `rainfall` (coast-proximity-weighted, cold current → fog-dry coast,
/// warm → slightly wetter) over nearby land. Distinct from
/// `simulate_weather`'s own (optional) fold of the same anomaly into `tc`
/// *before* building winds — this one runs after, as
/// `refreshClimate()`'s own separate step.
#[allow(clippy::too_many_arguments)]
pub fn apply_ocean_currents(
    gw: usize,
    gh: usize,
    field: &[f32],
    temperature: &mut [f32],
    rainfall: &mut [f32],
    sea: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    equator_temp: f64,
    pole_temp: f64,
    tilt_deg: f64,
    rotation_hours: f64,
    wind_manual: bool,
    wind_dir_deg: f64,
    press_k: f64,
    current_k: f64,
) {
    let ww = gw.min(240);
    let wh = (js_round(ww as f64 * gh as f64 / gw as f64) as usize).max(2);
    let wrap_x = world;
    let step = 3.0;

    let sst_b = ocean_sst_anomaly(
        gw,
        gh,
        field,
        ww,
        wh,
        wrap_x,
        step,
        sea,
        world,
        lat_n,
        lat_s,
        equator_temp,
        pole_temp,
        tilt_deg,
        rotation_hours,
        wind_manual,
        wind_dir_deg,
        press_k,
        current_k,
    );

    let mut c_mask = vec![0f32; ww * wh];
    for y in 0..wh {
        for x in 0..ww {
            let fx = x as f64 / (ww as f64 - 1.0) * (gw as f64 - 1.0);
            let fy = y as f64 / (wh as f64 - 1.0) * (gh as f64 - 1.0);
            if sample_arr(field, fx, fy, gw, gh) < sea {
                c_mask[y * ww + x] = 1.0;
            }
        }
    }
    let c_blur = cartalith_terrain::gauss_blur(&c_mask, js_round(ww as f64 * 0.05).max(2.0), ww, wh, wrap_x);

    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let a = bil_c(
                &sst_b,
                x as f64 / (gw as f64 - 1.0) * (ww as f64 - 1.0),
                y as f64 / (gh as f64 - 1.0) * (wh as f64 - 1.0),
                ww,
                wh,
                wrap_x,
            );
            if (field[i] as f64) < sea {
                temperature[i] = (temperature[i] as f64 + a) as f32;
            } else {
                let prox = bil_c(
                    &c_blur,
                    x as f64 / (gw as f64 - 1.0) * (ww as f64 - 1.0),
                    y as f64 / (gh as f64 - 1.0) * (wh as f64 - 1.0),
                    ww,
                    wh,
                    wrap_x,
                );
                temperature[i] = (temperature[i] as f64 + a * prox * 0.6) as f32;
                let dr = if a < 0.0 { a * prox * 0.10 } else { a * prox * 0.04 };
                rainfall[i] = (rainfall[i] as f64 + dr).clamp(0.0, 1.0) as f32;
            }
        }
    }
}

/// The climate parameters `simulate_weather` reads off
/// `state.climate`/`state.planet`/`state.seaLevel` — the formula's real
/// tuning knobs, same reasoning as `ClimateParams`/`HeightParams`.
pub struct WeatherParams {
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    pub pole_temp: f64,
    pub equator_temp: f64,
    pub tilt_deg: f64,
    pub rotation_hours: f64,
    pub lapse_rate: f64,
    pub sea_level: f64,
    pub peak_m: f64,
    pub wind_manual: bool,
    pub wind_dir_deg: f64,
    pub press_k: f64,
    pub ocean_hum: f64,
    pub evap: f64,
    pub ocean: f64,
    pub rain_k: f64,
    pub rain_dep: f64,
    pub bulk_evap: bool,
    /// Gates `build_wind`'s `deflectFlow` terrain-deflection block. JS has
    /// no equivalent flag -- v1.78 made this unconditional ("wind and
    /// current should always be coupled to terrain") -- so `false` here is
    /// this port's own deliberate deviation, not a JS default being
    /// mirrored. `deflect_flow` itself is now golden-verified
    /// (`golden_parity_deflect_flow.rs`, bit-exact against real JS output,
    /// three cases including custom knobs and world-wrap), and the wiring
    /// around it in `build_wind` (the `block` field, the `DeflectFlowParams`
    /// constants, the elevation-band damping combine) was checked
    /// line-for-line against reference HTML lines 5521-5535 and matches
    /// exactly. Still `false` regardless: it's a substantial iterative
    /// algorithm (16 blur+redirect passes) that reshapes wind everywhere
    /// terrain exists, cascading through every downstream term in this
    /// function (evaporation, advection, orographic rain) -- flipping it
    /// would invalidate `golden_parity_weather.rs` and everything built on
    /// `simulate_weather` unless those are also re-extracted, which hasn't
    /// happened yet (same reasoning `generate_terrain`'s own doc comment
    /// gives for `stampVolcanoesProvinces`).
    pub terrain_wind_deflection: bool,
    /// Gates folding `ocean_sst_anomaly` into `tc`/`sst_evap` before
    /// `build_wind` runs (reference HTML: `if(c.currents){...}` in
    /// `simulateWeather`'s own loop 2). JS's own default is `true`
    /// ("ocean currents ON by default... integrated into the weather sim
    /// before buildWind"); this port defaults to `false`. Unlike
    /// `terrain_wind_deflection`, not yet golden-verified even at the
    /// kernel level: `ocean_sst_anomaly` calls `build_wind` (now verified)
    /// and `compute_ocean_current` (`deflect_flow` again, plus a
    /// western-intensification heuristic that hasn't been checked against
    /// JS), so there's real remaining work here, not just a
    /// leave-off-to-protect-fixtures decision like the other two.
    pub currents: bool,
    pub current_k: f64,
}

/// `simulateWeather()` (reference HTML lines 5670-5719) — `MVP_SCOPE.md`
/// point 6's rainfall half: evaporate over sea, advect moisture along
/// wind (semi-Lagrangian backtrace), precipitate on orographic lift +
/// convective excess, iterated `iters` times, then normalized against
/// the 82nd-percentile land rainfall.
///
/// Runs on a coarse grid (`min(gw,240)` wide) like the JS original, then
/// bilinear-samples back up to full resolution.
///
/// **Deferred, matching this port's established pattern** (documented,
/// not silently dropped — see `cartalith-native/docs/CHANGELOG.md`):
/// world-structure continental-interior dryness (consistent with every
/// other world-structure deferral in this port so far). Terrain wind
/// deflection (`build_wind`'s `deflectFlow` block) and ocean-current SST
/// folding (`ocean_sst_anomaly`) are both ported and reachable via
/// `p.terrain_wind_deflection`/`p.currents`, but both default to `false`
/// here, not JS's own defaults (unconditional, and `true`, respectively)
/// — see their own doc comments (`WeatherParams::terrain_wind_deflection`/
/// `WeatherParams::currents`) for why. `geoidField` is
/// also omitted, matching `compute_temperature`'s own reasoning — the
/// default `state.planet.geoid.enabled=false` never reads it either.
pub fn simulate_weather(
    gw: usize,
    gh: usize,
    field: &[f32],
    iters: i32,
    decl: f64,
    p: &WeatherParams,
) -> Vec<f32> {
    let sea = p.sea_level;
    let mpu = meters_per_unit(p.peak_m, p.sea_level);
    let decl_r = decl * std::f64::consts::PI / 180.0;
    let ww = gw.min(240);
    let wh = ((ww as f64 * gh as f64 / gw as f64).round() as usize).max(2);
    let wrap_x = p.world;
    let n = ww * wh;
    let step = 3.0f64;

    let mut eh = vec![0f32; n];
    let mut tc = vec![0f32; n];
    let mut sst_evap = vec![0f32; n];
    let eq_eff = clim_effective_equator_temp(p.equator_temp, p.pole_temp, p.tilt_deg, p.rotation_hours);
    for y in 0..wh {
        let lat = (if p.world {
            90.0 - (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * 180.0
        } else {
            p.lat_n + (y as f64 / (wh.max(1) as f64 - 1.0).max(1.0)) * (p.lat_s - p.lat_n)
        }) * std::f64::consts::PI
            / 180.0;
        let t_sea = p.pole_temp + (eq_eff - p.pole_temp) * (lat - decl_r).cos().max(0.0);
        let ev_f = 0.2 + 0.8 * ((t_sea + 2.0) / 30.0).clamp(0.0, 1.0);
        for x in 0..ww {
            let i = y * ww + x;
            let fx = x as f64 / (ww as f64 - 1.0) * (gw as f64 - 1.0);
            let fy = y as f64 / (wh as f64 - 1.0) * (gh as f64 - 1.0);
            let h = sample_arr(field, fx, fy, gw, gh);
            eh[i] = h as f32;
            tc[i] = (t_sea - p.lapse_rate * ((h - sea).max(0.0) * mpu / 1000.0)) as f32;
            sst_evap[i] = ev_f as f32;
        }
    }

    // Loop 2 (docs/research/system-coupling-audit.md §2): fold the
    // wind-driven SST anomaly into the sea temperature BEFORE building
    // winds, so warm currents -> lower pressure + more evaporation ->
    // wetter downwind, cold currents -> drier.
    if p.currents {
        let an = ocean_sst_anomaly(
            gw,
            gh,
            field,
            ww,
            wh,
            wrap_x,
            step,
            sea,
            p.world,
            p.lat_n,
            p.lat_s,
            p.equator_temp,
            p.pole_temp,
            p.tilt_deg,
            p.rotation_hours,
            p.wind_manual,
            p.wind_dir_deg,
            p.press_k,
            p.current_k,
        );
        for i in 0..n {
            if (eh[i] as f64) < sea {
                tc[i] = (tc[i] as f64 + an[i] as f64) as f32;
                sst_evap[i] = (0.2 + 0.8 * ((tc[i] as f64 + 2.0) / 30.0).clamp(0.0, 1.0)) as f32;
            }
        }
    }

    let (wx, wy) = build_wind(
        ww,
        wh,
        step,
        Some(&tc),
        decl,
        p.world,
        p.lat_n,
        p.lat_s,
        p.wind_manual,
        p.wind_dir_deg,
        p.press_k,
        p.rotation_hours,
        if p.terrain_wind_deflection { Some((&eh, sea)) } else { None },
    );

    let mut w = vec![0f32; n];
    for i in 0..n {
        w[i] = if (eh[i] as f64) < sea { p.ocean_hum as f32 } else { 0.10 };
    }
    let mut rain = vec![0f32; n];
    let dry = 0.4 + p.rain_dep;

    for _ in 0..iters {
        for i in 0..n {
            if (eh[i] as f64) < sea {
                let cap = p.ocean_hum.max(sat_cap(tc[i] as f64)) * 1.2;
                let mut e = p.evap * sst_evap[i] as f64 * p.ocean;
                if p.bulk_evap {
                    let u = (wx[i] as f64).hypot(wy[i] as f64) / step;
                    e *= (0.4 + 0.6 * u) * (1.0 - w[i] as f64 / cap).max(0.0);
                }
                w[i] = (w[i] as f64 + e).min(cap) as f32;
            }
        }
        if !wrap_x {
            let th = 0.15 * step;
            for y in 0..wh {
                let mut i = y * ww;
                if wx[i] as f64 > th && (w[i] as f64) < p.ocean_hum {
                    w[i] = p.ocean_hum as f32;
                }
                i = y * ww + ww - 1;
                if (wx[i] as f64) < -th && (w[i] as f64) < p.ocean_hum {
                    w[i] = p.ocean_hum as f32;
                }
            }
            for x in 0..ww {
                let mut i = x;
                if wy[i] as f64 > th && (w[i] as f64) < p.ocean_hum {
                    w[i] = p.ocean_hum as f32;
                }
                i = (wh - 1) * ww + x;
                if (wy[i] as f64) < -th && (w[i] as f64) < p.ocean_hum {
                    w[i] = p.ocean_hum as f32;
                }
            }
        }
        let mut w2 = vec![0f32; n];
        for y in 0..wh {
            for x in 0..ww {
                let i = y * ww + x;
                w2[i] = bil_c(&w, x as f64 - wx[i] as f64, y as f64 - wy[i] as f64, ww, wh, wrap_x) as f32;
            }
        }
        for y in 0..wh {
            for x in 0..ww {
                let i = y * ww + x;
                if (eh[i] as f64) < sea {
                    w[i] = w2[i];
                    continue;
                }
                let ux = wx[i] as f64;
                let uy = wy[i] as f64;
                let l = ux.hypot(uy);
                let l = if l == 0.0 { 1.0 } else { l };
                let eh_up = bil_c(&eh, x as f64 - ux / l, y as f64 - uy / l, ww, wh, wrap_x);
                let oro = w2[i] as f64 * (eh[i] as f64 - eh_up).max(0.0) * p.rain_k * 9.0;
                let excess = (w2[i] as f64 - sat_cap(tc[i] as f64)).max(0.0);
                let conv = w2[i] as f64 * 0.05;
                let mut pr = (oro + excess * 0.6 + conv) * dry;
                if pr > w2[i] as f64 {
                    pr = w2[i] as f64;
                }
                w[i] = (w2[i] as f64 - pr) as f32;
                rain[i] = (rain[i] as f64 * 0.55 + pr * 0.45) as f32;
            }
        }
    }

    blur_coarse(&mut rain, ww, wh, wrap_x, 3);

    let mut land: Vec<f32> = (0..n).filter(|&i| (eh[i] as f64) >= sea).map(|i| rain[i]).collect();
    // NaN-safety: rain values are bounded, physically-derived quantities
    // (never NaN in practice); total_cmp is the panic-free choice anyway.
    land.sort_by(|a, b| a.total_cmp(b));
    let reference = if land.is_empty() {
        1e-6
    } else {
        let idx = ((land.len() as f64 * 0.82).floor() as usize).min(land.len() - 1);
        (land[idx] as f64).max(1e-6)
    };

    let mut rain_field = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let r = bil_c(
                &rain,
                x as f64 / (gw as f64 - 1.0) * (ww as f64 - 1.0),
                y as f64 / (gh as f64 - 1.0) * (wh as f64 - 1.0),
                ww,
                wh,
                wrap_x,
            );
            let moisture = (r / reference).min(1.0);
            rain_field[y * gw + x] = moisture as f32;
        }
    }
    rain_field
}

/// `applyClimateMoistureCorrectors()` (reference HTML lines 5188-5225) —
/// **unconditional**, unlike `applyOceanCurrents`/`computeSeasons`
/// (both gated on `state.climate.currents`/`.seasons`, off by default and
/// not yet ported): `refreshClimate()` always runs this after
/// `simulateWeather()`, so it's part of the MVP's default rainfall path,
/// not a stretch-goal deferral.
///
/// Three sequential, in-place corrections to `rain` (each sees the
/// previous one's already-written values, same as JS mutating the one
/// `rainField` array through all three passes):
/// 1. **Coastal proximity** — blur an ocean mask on the coarse weather
///    grid, add up to +0.38 near coastlines (wind-driven advection alone
///    under-wets the immediate coast).
/// 2. **River moisture corridors** — max-pool `flow_field` onto the same
///    coarse grid (a bilinear sample alone would miss narrow single-cell
///    rivers), blur, add up to +0.38 along them.
/// 3. **Latitude climate zones** — sharpens the ITCZ wet belt (±15°) and
///    subtropical dry belt (28°±13°) beyond what 2-D wind advection
///    reaches on its own (no vertical subsidence in this model).
///
/// `geo_field` omitted for the same reason `compute_temperature` omits
/// it: `state.planet.geoid.enabled` defaults `false`, where JS's
/// `geoAt()` always returns `0` anyway.
#[allow(clippy::too_many_arguments)]
pub fn apply_climate_moisture_correctors(
    gw: usize,
    gh: usize,
    field: &[f32],
    flow_field: &[f32],
    rain: &mut [f32],
    sea: f64,
    world: bool,
    lat_n: f64,
    lat_s: f64,
    zonal_k: f64,
) {
    let n = gw * gh;
    let ww = gw.min(240);
    let wh = (js_round(ww as f64 * gh as f64 / gw as f64) as usize).max(2);
    let nc = ww * wh;
    let wrap_x = world;

    let field_c = |x: usize, y: usize| -> f64 {
        sample_arr(
            field,
            x as f64 / (ww as f64 - 1.0) * (gw as f64 - 1.0),
            y as f64 / (wh as f64 - 1.0) * (gh as f64 - 1.0),
            gw,
            gh,
        )
    };

    // 1. coastal proximity
    let mut c_mask = vec![0f32; nc];
    for y in 0..wh {
        for x in 0..ww {
            if field_c(x, y) < sea {
                c_mask[y * ww + x] = 1.0;
            }
        }
    }
    let c_blur = cartalith_terrain::gauss_blur(&c_mask, js_round(ww as f64 * 0.06), ww, wh, wrap_x);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                continue;
            }
            let boost = bil_c(
                &c_blur,
                x as f64 / (gw as f64 - 1.0) * (ww as f64 - 1.0),
                y as f64 / (gh as f64 - 1.0) * (wh as f64 - 1.0),
                ww,
                wh,
                wrap_x,
            ) * 0.38;
            rain[i] = ((rain[i] as f64 + boost).min(1.0)) as f32;
        }
    }

    // 2. river moisture corridors
    let f_th = n as f64 * 0.0004;
    let mut f_max = 1e-6f64;
    for &f in flow_field {
        if f as f64 > f_max {
            f_max = f as f64;
        }
    }
    let step = (1usize).max((gw as f64 / ww as f64).ceil() as usize);
    let mut r_mask = vec![0f32; nc];
    for y in 0..wh {
        for x in 0..ww {
            let x0 = (x as f64 / ww as f64 * (gw as f64 - 1.0)) as usize;
            let y0 = (y as f64 / wh as f64 * (gh as f64 - 1.0)) as usize;
            let mut mx = 0f64;
            for dy in 0..step {
                for dx in 0..step {
                    let nx_ = (x0 + dx).min(gw - 1);
                    let ny_ = (y0 + dy).min(gh - 1);
                    let fl = flow_field[ny_ * gw + nx_] as f64;
                    if fl > mx {
                        mx = fl;
                    }
                }
            }
            if mx > f_th {
                r_mask[y * ww + x] = ((mx - f_th) / (f_max * 0.04)).min(1.0) as f32;
            }
        }
    }
    let r_blur = cartalith_terrain::gauss_blur(&r_mask, js_round(ww as f64 * 0.012).max(2.0), ww, wh, false);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                continue;
            }
            let boost = bil_c(
                &r_blur,
                x as f64 / (gw as f64 - 1.0) * (ww as f64 - 1.0),
                y as f64 / (gh as f64 - 1.0) * (wh as f64 - 1.0),
                ww,
                wh,
                false,
            ) * 0.38;
            rain[i] = ((rain[i] as f64 + boost).min(1.0)) as f32;
        }
    }

    // 3. latitude climate zones
    let (lat_top, lat_bot) = if world { (90.0, -90.0) } else { (lat_n, lat_s) };
    for y in 0..gh {
        let lat = lat_top - (lat_top - lat_bot) * y as f64 / (gh as f64 - 1.0);
        let abs_lat = lat.abs();
        let itcz = (1.0 - abs_lat / 15.0).max(0.0) * 0.22 * zonal_k;
        let sub_dry = (1.0 - (abs_lat - 28.0).abs() / 13.0).max(0.0) * 0.30 * zonal_k;
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                continue;
            }
            rain[i] = ((rain[i] as f64 + itcz - sub_dry).clamp(0.0, 1.0)) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}

