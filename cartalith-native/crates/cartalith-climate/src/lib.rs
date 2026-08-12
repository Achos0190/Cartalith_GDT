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

/// `buildWind()` (reference HTML lines 5464-5530), minus the terrain-
/// deflection branch (`opts.elev`, gated on `deflectFlow`) — deferred
/// under the same reasoning `MVP_SCOPE.md` explicitly grants
/// ocean-current terrain coupling: real, but a later addition, tracked
/// in `cartalith-native/docs/CHANGELOG.md`, not silently dropped.
///
/// Prevailing latitude-band winds (band count from `circulation_cells`),
/// then an optional pressure-gradient + Coriolis perturbation from a
/// smoothed temperature proxy (warm = thermal low) when `tc` is supplied
/// and `press_k > 0`.
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
    (wx, wy)
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
/// ocean-current SST folding (`state.climate.currents`, explicitly named
/// a stretch goal by `MVP_SCOPE.md`), terrain wind deflection
/// (`build_wind`'s omitted `elev` branch, the same `deflectFlow`
/// mechanism `MVP_SCOPE.md` groups with ocean currents), and
/// world-structure continental-interior dryness (consistent with every
/// other world-structure deferral in this port so far). `geoidField` is
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

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}

