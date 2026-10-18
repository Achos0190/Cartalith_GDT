//! **Variant (b)** of the volcano-edifice follow-up
//! (`EDIFICE_AND_STREAKS_2026-10-18.md` at the repository root; owner, Ruling
//! CM item 1: "measure first, no change yet"): radial erosion furrows on
//! edifice flanks, synthesised **at tile time** by the deep-zoom path rather
//! than at generation.
//!
//! # Why
//!
//! The generation grid can hold a gully no finer than about two cells. The
//! shipped flank cuts them at four cells, so at x16-x64 the tile path's
//! bilinear up-sample shows them as soft radial bands with no gully inside,
//! and the kilometre zoom-detail law (`amplify::KmZoomDetail`) adds isotropic
//! value noise there, not furrows. This module adds the furrows the grid could
//! not hold, octave by octave, as the tile's own resolution can carry them:
//! the same "complete the spectrum at the scale that can hold it" rule the
//! zoom law follows, with oriented (radial) noise instead of isotropic.
//!
//! # What it reads, and why only that
//!
//! Only `volcanic_field` -- which `WorldState` keeps and the LOD path already
//! has -- because the per-edifice records the stamps make are transient
//! (`VolcanoTrace`, dropped after the geology stage). The stamp writes
//! `volcanic_field = max over edifices of (1 - t)(1 - age)`, a cone per
//! edifice, so [`EdificeFrame::from_volcanic_field`] recovers each edifice's
//! centre (the cone's peak), and its radius from `t = 1 - v / v_peak` and the
//! distance to the centre. That recovery is approximate (the centre to the
//! nearest cell, the radius as a median), which is fine for **new** detail: it
//! fixes where furrows run (radially, from the recovered centre), not the
//! generated gullies' exact phase.
//!
//! # What it must never do
//!
//! - Run on any shipped or parity path. Nothing in the app calls it; the only
//!   caller is `cartalith-godot`'s `tests/zoom_detail_ladder.rs` under
//!   `ZD_FURROWS`. Wiring it into `lod_bridge` is the owner's call.
//! - Move the coastline: water texels are skipped and an excursion toward sea
//!   level is capped by `amplify`'s own `clamp_toward_sea`.
//! - Depend on anything but the texel's coarse coordinate, the frame and the
//!   options, which keeps two tiles seam-exact on a shared edge.

use cartalith_noise::pvnoise;
use cartalith_spatial::FloatRegion;

use crate::edifice_pits::EdificeFrame;
use crate::{edifice_resolved_weight, EDIFICE_RIDGE_SQ_MEAN};

/// The furrow law's knobs. Every field is required; there is no `Default`,
/// because each one is a scale the caller must state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FurrowOpts {
    /// Kilometres per coarse cell.
    pub km_per_cell: f64,
    /// Gully spacing at mid-flank, km (variant (a)'s
    /// [`crate::EDIFICE_GULLY_SPACING_KM_PROPOSED`]).
    pub spacing_km: f64,
    /// `WorldParams::peak_m`, the height that normalised 1.0 stands for.
    pub peak_m: f64,
    /// Sea level, normalised.
    pub sea: f64,
}

/// Furrow relief per unit spacing: an octave with spacing `s` km has relief
/// amplitude `FURROW_DEPTH_PER_SPACING * s`. Depth proportional to spacing is
/// the self-similar (`H = 1`) spectrum the zoom law ships
/// (`amplify::ZOOM_DETAIL_KM_OCTAVE_GAIN`), so every octave adds the same
/// slope. `0.1` puts a 1.5 km gully at 150 m, near the shipped gully term's
/// own ~117 m on the reference's mid-class stratocone (0.45 x 4 000 m x 0.10
/// rough x 0.65). **A labelled judgement**.
pub const FURROW_DEPTH_PER_SPACING: f64 = 0.1;

/// The finest octave this law adds, counted from the mid-flank spacing: 2^-8
/// of 1.5 km is about 6 m. A cost cap, like `KmZoomDetail::max_octaves`.
pub const FURROW_MAX_OCTAVES: i32 = 9;

/// The flank band furrows live in, as a fraction of the recovered radius: 0
/// inside `t < FURROW_T_TOP` (the summit and its depression; the recovered
/// frame does not know the caldera's own rim) and at the toe, peaking between.
/// **A labelled judgement**: between the shipped shapes' rims (0.12-0.38).
pub const FURROW_T_TOP: f64 = 0.25;

/// The furrow height at coarse coordinate `(cx, cy)` for a tile whose texels
/// are `texel_cells` coarse cells apart. Zero off every recovered edifice,
/// on the summit and at the toe.
///
/// Octave `k` (spacing `s_k = spacing_km / 2^k`) is weighted by
/// `max(0, w_tile − w_grid)`: the share the tile can hold beyond what the
/// generation grid already holds, both from [`edifice_resolved_weight`] on the octave's
/// noise-lattice spacing (the smaller of its angular and radial spacing, in
/// texels and in cells). So an octave the generated field already carries is
/// never added twice, and one finer than the tile is never drawn.
pub fn edifice_furrow_at(frame: &EdificeFrame, cx: f64, cy: f64, texel_cells: f64, o: &FurrowOpts) -> f64 {
    let Some(e) = frame.owner_near(cx, cy) else { return 0.0 };
    if !(o.km_per_cell > 0.0 && o.spacing_km > 0.0 && o.peak_m > 0.0 && texel_cells > 0.0) {
        return 0.0;
    }
    let (dx, dy) = (cx - e.cx, cy - e.cy);
    let d = dx.hypot(dy);
    let t = d / e.r;
    if !(FURROW_T_TOP..1.0).contains(&t) {
        return 0.0;
    }
    let u = (t - FURROW_T_TOP) / (1.0 - FURROW_T_TOP);
    let envelope = 4.0 * u * (1.0 - u);
    let sp_cells = o.spacing_km / o.km_per_cell;
    let g = ((std::f64::consts::PI * e.r / sp_cells).round().clamp(6.0, 1.0e6)) as i32;
    let ang = (dy.atan2(dx) * std::f64::consts::FRAC_1_PI * 0.5 + 0.5) * g as f64;
    let mut sum = 0.0;
    for k in 0..FURROW_MAX_OCTAVES {
        let f = f64::powi(2.0, k);
        let lat = (std::f64::consts::TAU * d / (g as f64 * f)).min(e.r / (2.2 * f));
        let w = (edifice_resolved_weight(lat / texel_cells) - edifice_resolved_weight(lat)).max(0.0);
        if w <= 0.0 {
            continue;
        }
        let n = pvnoise(ang * f, t * 2.2 * f, e.salt.wrapping_add(k.wrapping_mul(131)), (g as f64 * f).min(i32::MAX as f64) as i32);
        let ridge = 1.0 - (2.0 * n - 1.0).abs();
        // Incised, like the shipped gully term (`1 - 2·pridged`): a high
        // ridge is a furrow floor. Zero-mean, so the flank's mean height
        // does not move.
        let relief_m = FURROW_DEPTH_PER_SPACING * (o.spacing_km / f) * 1000.0;
        sum += -(ridge * ridge - EDIFICE_RIDGE_SQ_MEAN) * 2.0 * relief_m / o.peak_m * w;
    }
    sum * envelope
}

/// [`edifice_furrow_at`] over a tile carrying `amplify::amplify_region_padded`'s
/// halo, in place, with `add_zoom_detail_padded`'s own texel-to-coarse mapping
/// (`b` is the tile's coarse footprint, `w`/`h` the tile without the halo).
/// Water texels are skipped; an excursion toward sea level is capped at half
/// the headroom (`amplify::clamp_toward_sea`).
#[allow(clippy::too_many_arguments)]
pub fn add_edifice_furrows_padded(data: &mut [f32], w: usize, h: usize, pad: usize, frame: &EdificeFrame, b: &FloatRegion, o: &FurrowOpts) {
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    assert!(data.len() >= pw * ph, "add_edifice_furrows_padded: data is too short");
    let texel_cells = if w > 1 { b.w / (w as f64 - 1.0) } else { b.w };
    let padf = pad as f64;
    for oy in 0..ph {
        let cy = b.y + if h > 1 { (oy as f64 - padf) / (h as f64 - 1.0) * b.h } else { 0.0 };
        for ox in 0..pw {
            let i = oy * pw + ox;
            let base = data[i] as f64;
            if base < o.sea {
                continue;
            }
            let cx = b.x + if w > 1 { (ox as f64 - padf) / (w as f64 - 1.0) * b.w } else { 0.0 };
            let s = edifice_furrow_at(frame, cx, cy, texel_cells, o);
            if s != 0.0 {
                data[i] = (base + crate::amplify::clamp_toward_sea(base, s, o.sea)) as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic `volcanic_field`: two cones, `(1 - t)(1 - age)`, the
    /// stamp's own mark.
    fn two_cones(gw: usize, gh: usize) -> Vec<f32> {
        let mut v = vec![0f32; gw * gh];
        for (cx, cy, r, age) in [(40.0, 40.0, 30.0, 0.2), (110.0, 60.0, 20.0, 0.0)] {
            for y in 0..gh {
                for x in 0..gw {
                    let d = (x as f64 - cx).hypot(y as f64 - cy);
                    if d <= r {
                        let c = ((1.0 - d / r) * (1.0 - age)) as f32;
                        let i = y * gw + x;
                        v[i] = v[i].max(c);
                    }
                }
            }
        }
        v
    }

    fn opts() -> FurrowOpts {
        FurrowOpts { km_per_cell: 800.0 / 2048.0, spacing_km: 1.5, peak_m: 4000.0, sea: 0.42 }
    }

    /// Protects: the band rule. At a texel spacing equal to the coarse cell
    /// (x1-ish) nothing is added -- the grid already holds what that texel
    /// could -- and at a fine texel (1/16 cell) furrows appear, zero-mean
    /// round the flank, and nothing is added on the summit or off the cone.
    #[test]
    fn furrows_fill_only_what_the_grid_could_not_hold() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let ring = |texel: f64| -> Vec<f64> {
            (0..720).map(|k| {
                let a = k as f64 / 720.0 * std::f64::consts::TAU;
                edifice_furrow_at(&f, 40.0 + 18.0 * a.cos(), 40.0 + 18.0 * a.sin(), texel, &o)
            }).collect()
        };
        assert!(ring(1.0).iter().all(|&s| s == 0.0), "a cell-sized texel must add nothing");
        let fine = ring(1.0 / 16.0);
        let mean = fine.iter().sum::<f64>() / fine.len() as f64;
        let rms = (fine.iter().map(|s| s * s).sum::<f64>() / fine.len() as f64).sqrt();
        assert!(rms > 1e-4, "fine texels should carry furrows, rms {rms}");
        assert!(mean.abs() < 0.25 * rms, "furrows should be about zero-mean: mean {mean}, rms {rms}");
        assert_eq!(edifice_furrow_at(&f, 40.0, 41.0, 1.0 / 16.0, &o), 0.0, "summit");
        assert_eq!(edifice_furrow_at(&f, 150.0, 5.0, 1.0 / 16.0, &o), 0.0, "off every cone");
    }

    /// Protects: the angular noise closes across the bearing's +/-pi seam
    /// (due west of the centre), like the shipped gully term's `pridged`
    /// period: no radial scar where `atan2` wraps. The noise's period must be
    /// on the angular axis for this to hold.
    #[test]
    fn furrows_close_across_the_seam() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let (mut jump, mut step) = (0.0f64, 0.0f64);
        for k in 0..40 {
            let r = 12.0 + k as f64 * 0.3;
            let e = 1e-6;
            let p = |a: f64| edifice_furrow_at(&f, 40.0 + r * a.cos(), 40.0 + r * a.sin(), 1.0 / 16.0, &o);
            jump = jump.max((p(std::f64::consts::PI - e) - p(-std::f64::consts::PI + e)).abs());
            step = step.max((p(1.0) - p(1.0 + 2.0 * e)).abs());
        }
        // Across the seam the change must be no bigger than an ordinary step
        // of the same angular size elsewhere.
        assert!(jump <= 2.0 * step + 1e-12, "a scar at the seam: {jump} (an ordinary step of the same size is {step})");
    }

    /// Protects: the furrows run radially -- along a ray the furrow height
    /// varies far less than across the ray at the same radius.
    #[test]
    fn furrows_are_radial() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let tex = 1.0 / 16.0;
        let (mut along, mut across) = (0.0, 0.0);
        for k in 0..400 {
            let a = k as f64 * 0.0157;
            let r0 = 17.0;
            let p = |r: f64, da: f64| {
                let (cc, ss) = ((a + da).cos(), (a + da).sin());
                edifice_furrow_at(&f, 40.0 + r * cc, 40.0 + r * ss, tex, &o)
            };
            along += (p(r0 + 0.05, 0.0) - p(r0, 0.0)).powi(2);
            across += (p(r0, 0.05 / r0) - p(r0, 0.0)).powi(2);
        }
        assert!(across > 4.0 * along, "across {across} vs along {along}");
    }
}
