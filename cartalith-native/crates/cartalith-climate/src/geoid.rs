//! G2 geoid — `buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview`
//! (reference HTML lines 4967-5015).
//!
//! The geoid offsets **local** sea level: a cell is water iff
//! `field[i] < seaLevel + geoid[i]`. Every consumer in the reference reads
//! it through `geoAt(i)`, which returns `0` while the field is `null`, so
//! the whole feature collapses to the legacy comparison when it is off —
//! bit-identical by construction. That is why this port's
//! [`crate::compute_temperature`] has always taken `geo_field:
//! Option<&[f32]>`: the parameter was there waiting for this module.
//!
//! Three components, summed per cell:
//!
//! * **J2 rotational bulge** — sea stands higher at the equator, `∝ Ω²R/g`,
//!   which is where `rot_k` comes from ([`geoid_rot_k`]).
//! * **Low-degree harmonics** — four seeded `(m, l)` pairs standing in for a
//!   lumpy mass distribution.
//! * **Mantle noise** — low-frequency `fbm`, seam-blended in X for a
//!   wrapping world.
//!
//! The result is then re-centred to zero mean and rescaled so the peak
//! absolute offset is exactly `amp`.

use cartalith_jsmath::js_cos;
use cartalith_noise::{fbm, hash};

/// `buildGeoid`'s own `Object.assign` defaults (reference line 4974).
/// Every field is the reference's literal; `Default` exists so a caller
/// that only wants to move one knob writes `..GeoidOpts::default()` rather
/// than repeating the other seven.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoidOpts {
    pub seed: i32,
    pub rot_k: f64,
    pub harm_k: f64,
    pub mantle_k: f64,
    pub amp: f64,
    pub lat0: f64,
    pub lat1: f64,
    pub wrap_x: bool,
}

impl Default for GeoidOpts {
    fn default() -> Self {
        Self {
            seed: 7,
            rot_k: 1.0,
            harm_k: 0.6,
            mantle_k: 0.5,
            amp: 0.015,
            lat0: 90.0,
            lat1: -90.0,
            wrap_x: false,
        }
    }
}

/// The four `(lon order m, lat order l)` low-degree pairs (reference line
/// 4976). Order is load-bearing: `ph[k]` is seeded by `k`, and each term is
/// divided by `k + 1`.
const DEG: [(f64, f64); 4] = [(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (3.0, 2.0)];

/// `buildGeoid` (reference HTML lines 4973-4995) — pure, exactly as the
/// reference's own comment claims ("only pure fbm/hash from module scope").
///
/// `out` is a `Float32Array` in the reference, and the narrowing genuinely
/// participates: the zero-mean pass reads back each cell's already-narrowed
/// value to find the peak magnitude, and the rescale pass reads it a third
/// time. Kept as `Vec<f32>` with every intermediate at `f64`, which is what
/// JS does with an auto-promoted typed-array read.
pub fn build_geoid(w: usize, h: usize, o: &GeoidOpts) -> Vec<f32> {
    let n = w * h;
    let mut out = vec![0f32; n];
    let ph: [f64; 4] = std::array::from_fn(|k| hash(k as i32, 17, o.seed) * std::f64::consts::PI * 2.0);

    let denom = (h.max(2) - 1) as f64; // JS: Math.max(1, H-1)
    for y in 0..h {
        let lat = (o.lat0 + (y as f64 / denom) * (o.lat1 - o.lat0)) * std::f64::consts::PI / 180.0;
        let clat = js_cos(lat);
        let j2 = o.rot_k * (clat * clat - 2.0 / 3.0);
        for x in 0..w {
            let lon = x as f64 / w as f64 * std::f64::consts::PI * 2.0;
            let mut harm = 0.0;
            for (k, (m, l)) in DEG.iter().enumerate() {
                harm += js_cos(lon * m + ph[k]) * js_cos(lat * l + ph[k] * 0.7) / (k as f64 + 1.0);
            }
            let mut mantle = fbm(x as f64 / w as f64 * 4.0, y as f64 / h as f64 * 4.0, o.seed + 99);
            if o.wrap_x {
                // Seam-blend for world wrap: the same fbm sampled one full
                // period to the left, cross-faded across the row.
                let t = x as f64 / w as f64;
                let left = fbm((x as f64 - w as f64) / w as f64 * 4.0, y as f64 / h as f64 * 4.0, o.seed + 99);
                mantle = mantle * (1.0 - t) + left * t;
            }
            out[y * w + x] = (j2 + o.harm_k * harm * 0.35 + o.mantle_k * (mantle - 0.5) * 1.2) as f32;
        }
    }

    let mut mean = 0.0f64;
    for v in out.iter() {
        mean += *v as f64;
    }
    mean /= out.len() as f64;
    let mut mxa = 1e-9f64;
    for v in out.iter_mut() {
        *v = (*v as f64 - mean) as f32;
        let a = (*v as f64).abs();
        if a > mxa {
            mxa = a;
        }
    }
    let k = o.amp / mxa;
    for v in out.iter_mut() {
        *v = (*v as f64 * k) as f32; // zero-mean, peak |offset| = amp
    }
    out
}

/// `refreshGeoid`'s own `rotK` expression (reference line 5000):
/// `(24/rotationHours)² · radiusRel / g`, i.e. the centrifugal bulge scaled
/// against Earth's own day and gravity.
pub fn geoid_rot_k(rotation_hours: f64, radius_rel: f64, g: f64) -> f64 {
    (24.0 / rotation_hours.max(1.0)).powi(2) * radius_rel / g.max(0.05)
}

/// `refreshGeoid` (reference HTML lines 4996-5002). `None` is the
/// reference's own `geoidField = null` — the state every `geoAt()` caller
/// already collapses to zero on, and `state.planet.geoid.enabled`'s own
/// default.
///
/// `lat0`/`lat1` are `latAt(0)`/`latAt(GH-1)`, which the caller already has;
/// they are passed rather than recomputed so this function needs no view of
/// `world`/`latN`/`latS`.
#[allow(clippy::too_many_arguments)]
pub fn refresh_geoid(
    gw: usize,
    gh: usize,
    enabled: bool,
    amp: f64,
    seed: i32,
    rotation_hours: f64,
    radius_rel: f64,
    g: f64,
    lat0: f64,
    lat1: f64,
    world: bool,
) -> Option<Vec<f32>> {
    // JS: `!pg || !pg.enabled || !(pg.amp>0)`. `!(amp > 0)` rather than
    // `amp <= 0` so a NaN amplitude reads as off, matching JS (NaN>0 is
    // false) — this port's own NaN convention.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !enabled || !(amp > 0.0) {
        return None;
    }
    Some(build_geoid(
        gw,
        gh,
        &GeoidOpts {
            seed,
            rot_k: geoid_rot_k(rotation_hours, radius_rel, g),
            amp,
            lat0,
            lat1,
            wrap_x: world,
            ..GeoidOpts::default()
        },
    ))
}

/// `currentGeoidPreview` (reference HTML lines 5005-5015): the Geoid debug
/// view draws the field **even while the toggle is off**, falling back to
/// the `0.015` default amplitude. Returns `(field, amp)`; the view divides
/// by `amp` to get its `[-1, 1]` diverging ramp.
///
/// The reference caches this on a key built from every input. This port
/// does not: the debug rasters here are built on demand per pick, the same
/// choice `current_wind_field` already documents, and a 48×32-through-
/// 2048×1311 `build_geoid` is one pass of cheap arithmetic.
#[allow(clippy::too_many_arguments)]
pub fn current_geoid_preview(
    gw: usize,
    gh: usize,
    live: Option<&[f32]>,
    amp: f64,
    seed: i32,
    rotation_hours: f64,
    radius_rel: f64,
    g: f64,
    lat0: f64,
    lat1: f64,
    world: bool,
) -> (Vec<f32>, f64) {
    let amp = if amp > 0.0 { amp } else { 0.015 };
    if let Some(f) = live {
        return (f.to_vec(), amp);
    }
    (
        build_geoid(
            gw,
            gh,
            &GeoidOpts {
                seed,
                rot_k: geoid_rot_k(rotation_hours, radius_rel, g),
                amp,
                lat0,
                lat1,
                wrap_x: world,
                ..GeoidOpts::default()
            },
        ),
        amp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_reference_literals() {
        let d = GeoidOpts::default();
        assert_eq!((d.seed, d.rot_k, d.harm_k, d.mantle_k), (7, 1.0, 0.6, 0.5));
        assert_eq!((d.amp, d.lat0, d.lat1, d.wrap_x), (0.015, 90.0, -90.0, false));
    }

    #[test]
    fn the_field_is_zero_mean_and_peaks_at_amp() {
        let f = build_geoid(
            24,
            16,
            &GeoidOpts {
                amp: 0.02,
                ..GeoidOpts::default()
            },
        );
        let mean: f64 = f.iter().map(|v| *v as f64).sum::<f64>() / f.len() as f64;
        assert!(mean.abs() < 1e-7, "mean {mean} should be ~0");
        let peak = f.iter().map(|v| (*v as f64).abs()).fold(0.0, f64::max);
        assert!((peak - 0.02).abs() < 1e-7, "peak {peak} should be amp");
    }

    #[test]
    fn refresh_is_none_when_off_or_amplitude_is_not_positive() {
        assert!(refresh_geoid(8, 8, false, 0.015, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true).is_none());
        assert!(refresh_geoid(8, 8, true, 0.0, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true).is_none());
        assert!(refresh_geoid(8, 8, true, f64::NAN, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true).is_none());
        assert!(refresh_geoid(8, 8, true, 0.015, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true).is_some());
    }

    #[test]
    fn rot_k_clamps_its_two_divisors_the_way_the_reference_does() {
        // JS: Math.max(1, rotationHours) and Math.max(0.05, g).
        assert_eq!(geoid_rot_k(0.0, 1.0, 1.0), geoid_rot_k(1.0, 1.0, 1.0));
        assert_eq!(geoid_rot_k(24.0, 1.0, 0.0), geoid_rot_k(24.0, 1.0, 0.05));
        assert_eq!(geoid_rot_k(24.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn the_preview_falls_back_to_the_default_amplitude_but_prefers_a_live_field() {
        let (_, amp) = current_geoid_preview(8, 8, None, 0.0, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true);
        assert_eq!(amp, 0.015);
        let live = vec![0.5f32; 64];
        let (f, amp) = current_geoid_preview(8, 8, Some(&live), 0.03, 1, 24.0, 1.0, 1.0, 90.0, -90.0, true);
        assert_eq!(f, live);
        assert_eq!(amp, 0.03);
    }
}
