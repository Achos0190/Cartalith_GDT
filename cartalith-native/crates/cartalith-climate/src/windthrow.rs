//! Wind-throw hazard (reference HTML lines 5599-5636).
//!
//! One of the reference's two "disturbance model" layers: a read-only
//! hazard field, debug-view only, adding no physics and nothing to the save
//! format. It reuses the W1 wind field the Wind view already derives and
//! the biome raster the civilisation layer already builds — the same
//! "derive when picked, keep nothing after" rule [`super::current_wind_field`]
//! and `sample_bridge`'s own debug rasters follow.
//!
//! Risk = prevailing wind speed × forest canopy density × slope exposure,
//! zero over water.

use super::{WindFieldResult, bil_c};
use cartalith_jsmath::{js_hypot, js_min};

/// `_CANOPY` (reference line 5602) — the closed-canopy biome classes, as
/// frozen `BIOME_KEYS` indices: boreal, conifer, temperate forest,
/// temperate rainforest, tropical wet. `cartalith_civ::build_biome_raster`
/// emits this exact indexing.
pub const CANOPY_BIOMES: [u8; 5] = [3, 4, 5, 6, 12];

/// Canopy density for a non-closed-canopy cell (reference line 5613's
/// `:0.15`) — grassland still loses the odd tree.
const OPEN_CANOPY: f64 = 0.15;

/// `buildWindThrowField` (reference line 5604) — `[0,1]` per cell, `0`
/// over water.
///
/// `biome` is `cartalith_civ::build_biome_raster`'s output, taken as a
/// plain slice so this crate needs no dependency on `cartalith-civ`. The
/// reference reads the *cached biome raster* here rather than
/// re-classifying `tempField`/`rainField` per cell, and its own v1.86
/// comment records why that is not merely an optimisation: a mountain lake
/// (above sea level, so the `vw < sea` test never catches it) was being
/// misclassified as an ordinary land biome instead of reading as water.
/// Passing a raw temp/rain re-classification here would reintroduce that.
///
/// The reference's `geoAt(i)` is `0` throughout this port (no geoid field
/// exists — `GUI_GAP_REGISTER.md` WW-07), so `vw` is just `field[i]`.
///
/// Slope is computed here rather than taken as a parameter: the reference
/// calls `slopeAt(x,y)` directly, at full `f64`, whereas
/// `cartalith_civ::build_raw_slope_field` rounds each value through `f32`
/// on the way into its `Vec<f32>`. Handing that array in would be a real
/// precision divergence, not a shortcut.
pub fn build_wind_throw_field(
    field: &[f32],
    biome: &[u8],
    wind: &WindFieldResult,
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
) -> Vec<f32> {
    let n = gw * gh;
    let mut out = vec![0f32; n];
    if n == 0 {
        return out;
    }
    let fx_k = (gw as f64 - 1.0).max(1.0);
    let fy_k = (gh as f64 - 1.0).max(1.0);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let vw = field[i] as f64;
            if vw < sea {
                out[i] = 0.0;
                continue;
            }
            let canopy = if CANOPY_BIOMES.contains(&biome[i]) { 1.0 } else { OPEN_CANOPY };
            let fx = x as f64 / fx_k * (wind.ww as f64 - 1.0);
            let fy = y as f64 / fy_k * (wind.wh as f64 - 1.0);
            let u = bil_c(&wind.u, fx, fy, wind.ww, wind.wh, world);
            let v = bil_c(&wind.v, fx, fy, wind.ww, wind.wh, world);
            let sp = js_hypot(u, v) / wind.max_speed;
            // Exposed slopes and ridges catch more wind.
            let expo = 0.4 + 0.6 * js_min(1.0, slope_at(field, gw, gh, world, x, y) * 4.0);
            out[i] = (sp * canopy * expo).clamp(0.0, 1.0) as f32;
        }
    }
    out
}

/// `slopeAt(x,y)` (reference line 7584) — wrap-aware in X only when the
/// world wraps; Y clamps at both poles. Returns the raw `f64`, never
/// rounded through `f32`.
fn slope_at(field: &[f32], gw: usize, gh: usize, world: bool, x: usize, y: usize) -> f64 {
    let (xl, xr) = if world {
        ((x + gw - 1) % gw, (x + 1) % gw)
    } else {
        (if x > 0 { x - 1 } else { x }, if x + 1 < gw { x + 1 } else { x })
    };
    let l = field[y * gw + xl] as f64;
    let r = field[y * gw + xr] as f64;
    let u = if y > 0 { field[(y - 1) * gw + x] } else { field[y * gw + x] } as f64;
    let d = if y + 1 < gh { field[(y + 1) * gw + x] } else { field[y * gw + x] } as f64;
    js_hypot((r - l) * 0.5, (d - u) * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_wind(ww: usize, wh: usize, u: f32, v: f32) -> WindFieldResult {
        WindFieldResult {
            u: vec![u; ww * wh],
            v: vec![v; ww * wh],
            ww,
            wh,
            max_speed: js_hypot(u as f64, v as f64),
        }
    }

    #[test]
    fn water_is_always_zero() {
        let (gw, gh) = (8usize, 6usize);
        let field = vec![0.1f32; gw * gh];
        let out = build_wind_throw_field(&field, &vec![5u8; gw * gh], &flat_wind(4, 3, 3.0, 4.0), gw, gh, 0.42, false);
        assert!(out.iter().all(|&v| v == 0.0));
    }

    /// The canopy term is the only difference between these two runs, so
    /// the ratio pins `_CANOPY`'s own `1 : 0.15` and not merely "forest is
    /// higher than grass".
    #[test]
    fn closed_canopy_is_exactly_the_open_canopy_value_over_zero_point_one_five() {
        let (gw, gh) = (8usize, 6usize);
        let field: Vec<f32> = (0..gw * gh).map(|i| 0.5 + (i % 3) as f32 * 0.05).collect();
        let wind = flat_wind(4, 3, 3.0, 4.0);
        let forest = build_wind_throw_field(&field, &vec![5u8; gw * gh], &wind, gw, gh, 0.42, false);
        let grass = build_wind_throw_field(&field, &vec![7u8; gw * gh], &wind, gw, gh, 0.42, false);
        assert!(forest.iter().any(|&v| v > 0.0), "the fixture must reach the code it tests");
        for (f, g) in forest.iter().zip(grass.iter()) {
            assert!((*g as f64 - *f as f64 * OPEN_CANOPY).abs() < 1e-6, "{g} should be {f} * 0.15");
        }
    }

    /// Every one of the five `_CANOPY` indices must actually be treated as
    /// closed canopy, and its neighbours must not be — a `Set` membership
    /// test is exactly the kind of constant a golden fixture can miss.
    #[test]
    fn only_the_five_canopy_indices_count_as_closed_canopy() {
        let (gw, gh) = (4usize, 4usize);
        let field: Vec<f32> = (0..gw * gh).map(|i| 0.5 + (i % 3) as f32 * 0.05).collect();
        let wind = flat_wind(3, 3, 3.0, 4.0);
        let level = |b: u8| build_wind_throw_field(&field, &vec![b; gw * gh], &wind, gw, gh, 0.42, false)[5];
        let closed = level(5);
        for b in 0u8..=13 {
            let expect_closed = CANOPY_BIOMES.contains(&b);
            let got = level(b);
            if expect_closed {
                assert_eq!(got, closed, "biome {b} should be closed canopy");
            } else {
                assert!(got < closed, "biome {b} should not be closed canopy");
            }
        }
        assert_eq!(CANOPY_BIOMES, [3, 4, 5, 6, 12]);
    }

    /// `expo` floors at 0.4 and saturates at 1.0 for slope ≥ 0.25, so a
    /// perfectly flat land cell under full wind reads 0.4, not 0.
    #[test]
    fn a_flat_sheltered_cell_still_carries_the_zero_point_four_exposure_floor() {
        let (gw, gh) = (6usize, 6usize);
        let field = vec![0.6f32; gw * gh];
        let out = build_wind_throw_field(&field, &vec![5u8; gw * gh], &flat_wind(3, 3, 3.0, 4.0), gw, gh, 0.42, false);
        for &v in &out {
            assert!((v as f64 - 0.4).abs() < 1e-6, "flat land at full wind should be exactly the 0.4 floor, got {v}");
        }
    }
}
