//! Seasons and Köppen–Geiger classification — `computeTempInto`/
//! `computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor`
//! (reference HTML lines 7491-7562).
//!
//! The annual `tempField`/`rainField` pair the rest of this crate produces
//! cannot classify a climate: Köppen is defined on the *seasonal* extremes.
//! `computeSeasons` therefore runs the temperature and weather models twice
//! more, once at each solstice (`±axialTiltDeg` solar declination), and the
//! classifier reads the resulting Jul/Jan pair.
//!
//! `KOPPEN_KEYS` is a **frozen, append-only** order in the reference — the
//! raster it produces is an exported index layer (`koppen_index.json`), so
//! reordering it would silently reinterpret every previously exported map.
//! Kept verbatim.

use crate::{lat_at, ClimateParams, WeatherParams};
use cartalith_jsmath::{js_max, js_min};

/// `KOPPEN_KEYS` (reference HTML lines 7513-7514) — **frozen append-only
/// order**; index 0 in the raster is ocean, so key `k` maps to raster index
/// `k + 1`.
pub const KOPPEN_KEYS: [&str; 30] = [
    "Af", "Am", "Aw", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc",
    "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF",
];

/// `KOPPEN_COL` (reference HTML lines 7517-7522) — the standard Peel et al.
/// 2007 palette, abbreviated. Parallel to [`KOPPEN_KEYS`].
pub const KOPPEN_COL: [(u8, u8, u8); 30] = [
    (0, 0, 254),
    (0, 119, 255),
    (70, 169, 250),
    (254, 0, 0),
    (254, 150, 149),
    (245, 163, 1),
    (255, 219, 99),
    (255, 255, 0),
    (198, 199, 0),
    (150, 150, 0),
    (150, 255, 150),
    (99, 199, 99),
    (50, 150, 50),
    (198, 255, 78),
    (102, 255, 51),
    (51, 199, 1),
    (255, 0, 254),
    (198, 0, 199),
    (150, 50, 149),
    (150, 100, 149),
    (171, 177, 255),
    (90, 120, 220),
    (76, 81, 181),
    (50, 0, 135),
    (0, 255, 255),
    (56, 199, 255),
    (0, 126, 125),
    (0, 69, 94),
    (178, 178, 178),
    (104, 104, 104),
];

/// `KOPPEN_INDEX[code]` (reference line 7515): the 1-based raster index of
/// a code, or `None` for a code the frozen key list does not contain.
pub fn koppen_index(code: &str) -> Option<u8> {
    KOPPEN_KEYS.iter().position(|k| *k == code).map(|i| i as u8 + 1)
}

/// `koppenColor` (reference HTML line 7560). Index `0` (ocean /
/// unclassified) is the reference's own dark blue; an index past the frozen
/// key list is its own grey fallback.
pub fn koppen_color(idx: u8) -> (u8, u8, u8) {
    if idx == 0 {
        return (18, 34, 64);
    }
    KOPPEN_COL.get(idx as usize - 1).copied().unwrap_or((128, 128, 128))
}

/// The four seasonal fields `computeSeasons` fills, plus the raster it
/// derives from them.
pub struct Seasons {
    pub temp_jul: Vec<f32>,
    pub temp_jan: Vec<f32>,
    pub rain_jul: Vec<f32>,
    pub rain_jan: Vec<f32>,
    pub koppen: Vec<u8>,
}

/// `computeTempInto` (reference HTML lines 7491-7500): the temperature
/// model with the thermal equator shifted by solar declination `decl`
/// (degrees).
///
/// This is [`crate::compute_temperature`]'s own body at `decl = 0`, minus
/// the cryosphere-albedo relaxation — which `computeTempInto` genuinely
/// does not apply, so the seasonal pair is the *un-relaxed* field even when
/// `state.climate.albedo` is on. That asymmetry is the reference's, not
/// this port's.
pub fn compute_temp_into(gw: usize, gh: usize, field: &[f32], geo_field: Option<&[f32]>, decl: f64, p: &ClimateParams) -> Vec<f32> {
    let mpu = crate::meters_per_unit(p.peak_m, p.sea_level);
    let eq_eff = crate::clim_effective_equator_temp(p.equator_temp, p.pole_temp, p.tilt_deg, p.rotation_hours);
    let decl_r = decl * std::f64::consts::PI / 180.0;
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        let lat = lat_at(y, gh, p.world, p.lat_n, p.lat_s) * std::f64::consts::PI / 180.0;
        let t_sea = p.pole_temp + (eq_eff - p.pole_temp) * (lat - decl_r).cos().max(0.0);
        for x in 0..gw {
            let i = y * gw + x;
            let geo = geo_field.map_or(0.0, |g| g[i] as f64);
            let above_sea = ((field[i] as f64 - geo - p.sea_level).max(0.0)) * mpu;
            out[i] = (t_sea - p.lapse_rate * p.g * (above_sea / 1000.0)) as f32;
        }
    }
    out
}

/// Everything `classifyKoppen` reads that is not per-cell state.
pub struct KoppenParams {
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    pub sea_level: f64,
    /// `state.climate.maxRainMm || 3000` — the reference's own `||`, so a
    /// caller passing `0` gets `3000`, not a world with no rain.
    pub max_rain_mm: f64,
}

/// `classifyKoppen` (reference HTML lines 7524-7555) for one cell. Returns
/// the code string, or `None` over water.
///
/// `geo` is `geoAt(i)` — the geoid's local sea-level offset, `0.0` when the
/// geoid is off.
#[allow(clippy::too_many_arguments)]
pub fn classify_koppen(
    height: f64,
    geo: f64,
    temp_jul: f64,
    temp_jan: f64,
    rain_jul: f64,
    rain_jan: f64,
    lat: f64,
    p: &KoppenParams,
) -> Option<&'static str> {
    if height - geo < p.sea_level {
        return None;
    }
    let th = js_max(temp_jul, temp_jan);
    let tc = js_min(temp_jul, temp_jan);
    let mat = (th + tc) * 0.5;
    let mm = if p.max_rain_mm != 0.0 && !p.max_rain_mm.is_nan() {
        p.max_rain_mm
    } else {
        3000.0
    };
    let north = lat >= 0.0;
    // Local-summer / local-winter precipitation rates.
    let p_sum01 = if north { rain_jul } else { rain_jan };
    let p_win01 = if north { rain_jan } else { rain_jul };
    let map = ((p_sum01 + p_win01) * 0.5) * mm; // annual total (mm)
    let sum_mm = p_sum01 * mm;
    let win_mm = p_win01 * mm;
    let p_summer_share = sum_mm / (sum_mm + win_mm + 1e-6);
    let pdry_m = js_min(sum_mm, win_mm) / 6.0;
    let sum_dry_m = sum_mm / 6.0;
    let win_dry_m = win_mm / 6.0;

    // E -- polar
    if th < 10.0 {
        return Some(if th < 0.0 { "EF" } else { "ET" });
    }
    // B -- arid (threshold depends on which half-year carries the rain)
    let pth = 20.0 * mat
        + if p_summer_share >= 0.7 {
            280.0
        } else if p_summer_share <= 0.3 {
            0.0
        } else {
            140.0
        };
    if map < pth {
        return Some(match (map < 0.5 * pth, mat >= 18.0) {
            (true, true) => "BWh",
            (true, false) => "BWk",
            (false, true) => "BSh",
            (false, false) => "BSk",
        });
    }
    // third letter from warmest-season temperature (2-season approximation)
    let third = if th >= 22.0 {
        'a'
    } else if th >= 10.0 {
        'b'
    } else {
        'c'
    };
    // second letter: dry-summer (s) / dry-winter (w) / no dry season (f)
    let second = if sum_dry_m < 40.0 && sum_dry_m < win_dry_m / 3.0 {
        's'
    } else if win_dry_m < sum_dry_m / 10.0 {
        'w'
    } else {
        'f'
    };
    // A -- tropical
    if tc >= 18.0 {
        if pdry_m >= 60.0 {
            return Some("Af");
        }
        if pdry_m >= 100.0 - map / 25.0 {
            return Some("Am");
        }
        return Some("Aw");
    }
    // C -- temperate (coldest 0..18); D -- continental (coldest < 0)
    let grp = if tc >= 0.0 { 'C' } else { 'D' };
    let mut letter = third;
    if grp == 'D' && tc < -38.0 {
        letter = 'd';
    }
    let code: String = [grp, second, letter].iter().collect();
    // The reference's own "guard to a defined code". Its fallback
    // (`grp+second+(third==='c'?'c':third)`) recomposes the *same* string
    // whenever `third` was used, so the only real effect is on the `'d'`
    // branch; `build_koppen` maps an unknown code to 0 either way.
    KOPPEN_KEYS.iter().find(|k| **k == code).copied().or_else(|| {
        let alt: String = [grp, second, third].iter().collect();
        KOPPEN_KEYS.iter().find(|k| **k == alt).copied()
    })
}

/// `buildKoppen` (reference HTML lines 7556-7558): the whole raster,
/// `0` over water and over any code the frozen key list does not contain.
#[allow(clippy::too_many_arguments)]
pub fn build_koppen(
    gw: usize,
    gh: usize,
    field: &[f32],
    geo_field: Option<&[f32]>,
    temp_jul: &[f32],
    temp_jan: &[f32],
    rain_jul: &[f32],
    rain_jan: &[f32],
    p: &KoppenParams,
) -> Vec<u8> {
    let mut out = vec![0u8; gw * gh];
    for y in 0..gh {
        let lat = lat_at(y, gh, p.world, p.lat_n, p.lat_s);
        for x in 0..gw {
            let i = y * gw + x;
            let geo = geo_field.map_or(0.0, |g| g[i] as f64);
            let code = classify_koppen(
                field[i] as f64,
                geo,
                temp_jul[i] as f64,
                temp_jan[i] as f64,
                rain_jul[i] as f64,
                rain_jan[i] as f64,
                lat,
                p,
            );
            out[i] = code.and_then(koppen_index).unwrap_or(0);
        }
    }
    out
}

/// `computeSeasons` (reference HTML lines 7501-7511) — the orchestrator:
/// two more temperature solves and two more weather simulations at
/// `±axialTiltDeg` declination, then the classifier.
///
/// **What differs from the reference, and why.** The temperature half and
/// the classifier are bit-exact (`golden_parity_koppen.rs` pins both against
/// a real reference run). The *rain* half is [`crate::simulate_weather`],
/// which carries this port's three long-standing, already-disclosed
/// deferrals — terrain wind deflection, ocean-current SST folding and
/// world-structure interior dryness (see `WeatherParams`' own doc comments).
/// Those deferrals move seasonal rainfall exactly as much as they already
/// move annual rainfall, and the Köppen classifier is downstream of them, so
/// a raster built from *this* function's rain is only as close to the
/// reference's as `simulate_weather` itself is. The golden suite therefore
/// feeds the classifier the reference's own captured seasonal rain rather
/// than re-deriving it here, which is what makes it a test of the
/// classifier instead of a third copy of the weather test.
///
/// The reference mutates `rainField` in place and restores the annual field
/// afterwards; nothing is mutated here, so that dance has no equivalent.
#[allow(clippy::too_many_arguments)]
pub fn compute_seasons(
    gw: usize,
    gh: usize,
    field: &[f32],
    geo_field: Option<&[f32]>,
    tilt_deg: f64,
    w_iters: i32,
    cp: &ClimateParams,
    wp: &WeatherParams,
    kp: &KoppenParams,
) -> Seasons {
    let temp_jul = compute_temp_into(gw, gh, field, geo_field, tilt_deg, cp);
    let temp_jan = compute_temp_into(gw, gh, field, geo_field, -tilt_deg, cp);
    let rain_jul = crate::simulate_weather(gw, gh, field, w_iters, tilt_deg, wp);
    let rain_jan = crate::simulate_weather(gw, gh, field, w_iters, -tilt_deg, wp);
    let koppen = build_koppen(gw, gh, field, geo_field, &temp_jul, &temp_jan, &rain_jul, &rain_jan, kp);
    Seasons {
        temp_jul,
        temp_jan,
        rain_jul,
        rain_jan,
        koppen,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kp() -> KoppenParams {
        KoppenParams {
            world: true,
            lat_n: 90.0,
            lat_s: -90.0,
            sea_level: 0.42,
            max_rain_mm: 3000.0,
        }
    }

    #[test]
    fn the_frozen_key_order_and_its_palette_stay_parallel() {
        assert_eq!(KOPPEN_KEYS.len(), KOPPEN_COL.len());
        assert_eq!(KOPPEN_KEYS[0], "Af");
        assert_eq!(KOPPEN_KEYS[29], "EF");
        assert_eq!(koppen_index("Af"), Some(1));
        assert_eq!(koppen_index("EF"), Some(30));
        assert_eq!(koppen_index("Xx"), None);
        assert_eq!(koppen_color(0), (18, 34, 64));
        assert_eq!(koppen_color(1), (0, 0, 254));
        assert_eq!(koppen_color(31), (128, 128, 128));
    }

    #[test]
    fn water_is_unclassified() {
        assert_eq!(classify_koppen(0.1, 0.0, 20.0, 20.0, 0.5, 0.5, 10.0, &kp()), None);
        // ...and the geoid raises local sea level, so a cell just above the
        // global waterline can still be water.
        assert_eq!(classify_koppen(0.43, 0.02, 20.0, 20.0, 0.5, 0.5, 10.0, &kp()), None);
        assert!(classify_koppen(0.43, 0.0, 20.0, 20.0, 0.5, 0.5, 10.0, &kp()).is_some());
    }

    #[test]
    fn the_polar_branch_splits_at_zero_and_pre_empts_everything_else() {
        assert_eq!(classify_koppen(0.5, 0.0, 9.9, -40.0, 0.9, 0.9, 45.0, &kp()), Some("ET"));
        assert_eq!(classify_koppen(0.5, 0.0, -0.1, -40.0, 0.9, 0.9, 45.0, &kp()), Some("EF"));
        // Th exactly 10 is NOT polar (the test is `< 10`).
        assert_ne!(classify_koppen(0.5, 0.0, 10.0, -40.0, 0.9, 0.9, 45.0, &kp()), Some("ET"));
    }

    #[test]
    fn the_arid_branch_splits_on_both_map_and_mat() {
        // Near-zero rain: MAP < Pth for any positive MAT, and BW (not BS)
        // because MAP < 0.5 * Pth too.
        assert_eq!(classify_koppen(0.5, 0.0, 30.0, 25.0, 0.0, 0.0, 10.0, &kp()), Some("BWh"));
        // MAT below 18 flips h -> k. Th must stay >= 10 to clear the polar
        // branch, so a 14/8 pair gives MAT = 11.
        assert_eq!(classify_koppen(0.5, 0.0, 14.0, 8.0, 0.0, 0.0, 10.0, &kp()), Some("BWk"));
    }

    #[test]
    fn the_southern_hemisphere_swaps_which_solstice_is_summer() {
        // Rain in July only. North of the equator that is a wet summer /
        // dry winter (w); south of it, the reverse (s).
        let n = classify_koppen(0.5, 0.0, 25.0, 5.0, 0.6, 0.0, 40.0, &kp());
        let s = classify_koppen(0.5, 0.0, 25.0, 5.0, 0.6, 0.0, -40.0, &kp());
        assert_eq!(n, Some("Cwa"));
        assert_eq!(s, Some("Csa"));
    }

    #[test]
    fn the_coldest_month_selects_the_group() {
        // Tc >= 18 -> A, 0..18 -> C, < 0 -> D, < -38 -> the d third letter.
        assert_eq!(classify_koppen(0.5, 0.0, 30.0, 20.0, 0.5, 0.5, 5.0, &kp()), Some("Af"));
        assert_eq!(classify_koppen(0.5, 0.0, 25.0, 5.0, 0.5, 0.5, 45.0, &kp()), Some("Cfa"));
        assert_eq!(classify_koppen(0.5, 0.0, 25.0, -5.0, 0.5, 0.5, 55.0, &kp()), Some("Dfa"));
        assert_eq!(classify_koppen(0.5, 0.0, 25.0, -40.0, 0.5, 0.5, 65.0, &kp()), Some("Dfd"));
    }

    #[test]
    fn max_rain_mm_falls_back_to_three_thousand_on_zero() {
        let mut p = kp();
        p.max_rain_mm = 0.0;
        // With mm = 0 every precipitation term would be 0 and every land
        // cell would read as desert; the `|| 3000` is what stops that.
        assert_eq!(classify_koppen(0.5, 0.0, 25.0, 20.0, 0.5, 0.5, 5.0, &p), Some("Af"));
    }

    #[test]
    fn build_koppen_zeroes_water_and_fills_land() {
        let field = vec![0.1f32, 0.9, 0.9, 0.1];
        let tj = vec![25.0f32; 4];
        let ta = vec![20.0f32; 4];
        let r = vec![0.5f32; 4];
        let out = build_koppen(2, 2, &field, None, &tj, &ta, &r, &r, &kp());
        assert_eq!(out[0], 0);
        assert_eq!(out[3], 0);
        assert_eq!(out[1], koppen_index("Af").unwrap());
    }
}
