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

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}
