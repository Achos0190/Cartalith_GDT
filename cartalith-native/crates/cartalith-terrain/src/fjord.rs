//! Fjords — constrained glacial-coastal incision (reference HTML lines
//! 3201-3249; the source doc's "Better terrain", Section 1).
//!
//! Fjords are not generic coastal indents: they are overdeepened glacial
//! troughs, strictly bound to cold, steep, hard-rock coasts (Norway,
//! Patagonia, NZ South Island, Alaska, Greenland). The composite mask
//! enforces three physical constraints so they never appear on tropical,
//! low-relief or weak-rock coasts:
//!
//! ```text
//! Fjord = I_glacial(paleoclimate thermal band)
//!       × H_relief(coastal orographic steepness)
//!       × B_crystalline(competent lithology)
//! ```
//!
//! Carving then overdeepens *pre-existing coastal valley floors* below sea
//! level (drowned U-valleys) while leaving ridges high, producing steep
//! fjord walls. Refs in the source doc: Holtedahl 1993, Montgomery 2001,
//! Benn & Evans 2014.
//!
//! `lith` is `cartalith_civ::build_lithology`'s output and `coast_d` is
//! [`crate::infer::chamfer_dist`] over the sea mask — both taken as plain
//! slices, exactly as the reference passes them, so this module needs no
//! dependency on either producer.

use crate::sculpt::{clamp01, smoothstep};
use cartalith_jsmath::js_min;

/// `LITH_COMPETENCE` (reference line 3208) — granite, basalt, andesite,
/// limestone, sandstone, shale, metamorphic (gneiss). Crystalline rock is
/// competent; a fjord wall has to stand up.
pub const LITH_COMPETENCE: [f64; 7] = [1.0, 0.9, 0.6, 0.2, 0.2, 0.15, 1.0];

/// `buildFjordMask`'s `opts` (reference line 3209).
///
/// [`FjordMaskOpts::for_width`] is the reference's own `{}` default, whose
/// `coastBuffer` is `Math.max(4, W/30)` and therefore grid-dependent —
/// which is why this is a constructor rather than a `Default` impl.
#[derive(Clone, Copy, Debug)]
pub struct FjordMaskOpts {
    /// Coastal fringe depth in cells — fjords penetrate inland.
    pub coast_buffer: f64,
    /// Pleistocene cold offset applied to the temperature field.
    pub paleo_anomaly: f64,
    /// Relief-window radius, in cells.
    pub relief_r: i64,
    /// Relief floor: below this local range, no fjord.
    pub relief_min: f64,
    /// Relief span the `[relief_min, relief_min + relief_range]` window
    /// normalises over.
    pub relief_range: f64,
}

impl FjordMaskOpts {
    /// The reference's `buildFjordMask(..., {})` defaults.
    pub fn for_width(w: usize) -> Self {
        Self {
            coast_buffer: (w as f64 / 30.0).max(4.0),
            paleo_anomaly: 7.0,
            relief_r: 2,
            relief_min: 0.06,
            relief_range: 0.18,
        }
    }
}

/// `buildFjordMask` (reference line 3209) — `[0,1]` per cell, non-zero
/// only on cold, rugged, crystalline coastal land.
#[allow(clippy::too_many_arguments)]
pub fn build_fjord_mask(
    fld: &[f32],
    temp_c: &[f32],
    lith: &[u8],
    coast_d: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    opts: FjordMaskOpts,
) -> Vec<f32> {
    let mut out = vec![0f32; w * h];
    let buf = opts.coast_buffer;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if (fld[i] as f64) < sea {
                continue; // land only
            }
            let cd = coast_d[i] as f64;
            if cd > buf {
                continue; // coastal fringe only
            }
            let tp = temp_c[i] as f64 - opts.paleo_anomaly; // paleoclimate-adjusted
            // cold-but-not-frozen-out marine-glacier band
            let iglac = smoothstep(6.0, -2.0, tp) * smoothstep(-22.0, -12.0, tp);
            if iglac <= 0.0 {
                continue;
            }
            // NEIGHBOURHOOD relief — high even at a flat valley floor
            // between steep walls, which is exactly where a fjord is.
            let mut mx = -1e9f64;
            let mut mn = 1e9f64;
            for dy in -opts.relief_r..=opts.relief_r {
                for dx in -opts.relief_r..=opts.relief_r {
                    let nx = x as i64 + dx;
                    let ny = y as i64 + dy;
                    if nx < 0 || nx >= w as i64 || ny < 0 || ny >= h as i64 {
                        continue;
                    }
                    let hh = fld[ny as usize * w + nx as usize] as f64;
                    if hh > mx {
                        mx = hh;
                    }
                    if hh < mn {
                        mn = hh;
                    }
                }
            }
            let hrel = clamp01((mx - mn - opts.relief_min) / opts.relief_range);
            if hrel <= 0.0 {
                continue;
            }
            let bc = LITH_COMPETENCE.get(lith[i] as usize).copied().unwrap_or(0.3);
            // ×fringe: strongest at the shore, still present inland.
            out[i] = (iglac * hrel * bc * (1.0 - 0.5 * cd / buf)) as f32;
        }
    }
    out
}

/// `carveFjords`' `opts` (reference line 3229).
#[derive(Clone, Copy, Debug)]
pub struct CarveFjordsOpts {
    /// How far below sea level a fully-masked valley floor is pulled.
    pub over_deep: f64,
    /// Mask value at which the pull reaches full strength.
    pub mask_full: f64,
}

impl Default for CarveFjordsOpts {
    fn default() -> Self {
        Self { over_deep: 0.16, mask_full: 0.25 }
    }
}

/// `carveFjords` (reference line 3229) — overdeepen valley floors inside
/// the mask zone below sea level; ridges are untouched, which is what
/// makes the sides steep. Returns a carved **copy**; the input is not
/// modified.
///
/// `sea` enters only through `target = sea - over_deep`.
pub fn carve_fjords(
    fld: &[f32],
    mask: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    opts: CarveFjordsOpts,
) -> Vec<f32> {
    let mut out = fld.to_vec();
    let target = sea - opts.over_deep;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let m = mask[i] as f64;
            if m <= 0.02 {
                continue;
            }
            let mut s = 0f64;
            let mut c = 0f64;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let nx = x as i64 + dx;
                    let ny = y as i64 + dy;
                    if nx < 0 || nx >= w as i64 || ny < 0 || ny >= h as i64 {
                        continue;
                    }
                    s += fld[ny as usize * w + nx as usize] as f64;
                    c += 1.0;
                }
            }
            if s / c - fld[i] as f64 <= 0.0 {
                continue; // ridges / walls stay high -> steep sides
            }
            // Pull the valley floor toward a FIXED overdeepened (sub-sea)
            // bed, so even moderate-mask corridors drown into visible inlets.
            let wgt = js_min(1.0, m / opts.mask_full);
            let carved = fld[i] as f64 + (target - fld[i] as f64) * wgt;
            if carved < out[i] as f64 {
                out[i] = carved as f32; // only deepen
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference's own default is grid-dependent through `W/30` and
    /// floored at 4 — both halves matter, and a constant would pass a
    /// golden test taken at one width while being wrong at every other.
    #[test]
    fn the_coast_buffer_default_is_the_max_of_four_and_a_thirtieth_of_the_width() {
        assert_eq!(FjordMaskOpts::for_width(48).coast_buffer, 4.0);
        assert_eq!(FjordMaskOpts::for_width(120).coast_buffer, 4.0);
        assert_eq!(FjordMaskOpts::for_width(600).coast_buffer, 20.0);
    }

    #[test]
    fn carving_never_raises_a_cell() {
        let w = 8;
        let h = 8;
        let fld: Vec<f32> = (0..w * h).map(|i| 0.30 + (i % 5) as f32 * 0.02).collect();
        let mask = vec![1.0f32; w * h];
        let out = carve_fjords(&fld, &mask, w, h, 0.42, CarveFjordsOpts::default());
        for (a, b) in out.iter().zip(fld.iter()) {
            assert!(a <= b, "carve_fjords is deepen-only");
        }
    }

    #[test]
    fn a_zero_mask_carves_nothing() {
        let w = 6;
        let h = 6;
        let fld: Vec<f32> = (0..w * h).map(|i| 0.2 + i as f32 * 0.01).collect();
        let out = carve_fjords(&fld, &vec![0f32; w * h], w, h, 0.42, CarveFjordsOpts::default());
        assert_eq!(out, fld);
    }

    /// The mask is a product of three terms; each must independently be
    /// able to zero it, or one of them is not actually being applied.
    #[test]
    fn each_of_the_three_constraints_can_zero_the_mask_alone() {
        let w = 9;
        let h = 9;
        // A steep coastal ridge: half ocean, half a rising wall.
        let fld: Vec<f32> = (0..w * h).map(|i| if (i % w) < 3 { 0.10 } else { 0.42 + ((i % w) - 3) as f32 * 0.12 }).collect();
        let coast_d: Vec<f32> = (0..w * h).map(|i| ((i % w) as f32 - 2.0).max(0.0)).collect();
        let cold = vec![-5.0f32; w * h]; // Tp = -12, inside the band
        let granite = vec![0u8; w * h];
        let o = FjordMaskOpts::for_width(w);

        let base = build_fjord_mask(&fld, &cold, &granite, &coast_d, w, h, 0.42, o);
        assert!(base.iter().any(|&v| v > 0.0), "the fixture must reach the code it tests");

        let warm = vec![30.0f32; w * h];
        assert!(
            build_fjord_mask(&fld, &warm, &granite, &coast_d, w, h, 0.42, o).iter().all(|&v| v == 0.0),
            "thermal band alone must be able to zero it"
        );
        let flat = vec![0.5f32; w * h];
        assert!(
            build_fjord_mask(&flat, &cold, &granite, &coast_d, w, h, 0.42, o).iter().all(|&v| v == 0.0),
            "relief alone must be able to zero it"
        );
        // Lithology 5 (shale) is 0.15, not 0 -- it scales rather than zeroes,
        // which is the honest statement of what that term does.
        let shale = vec![5u8; w * h];
        let weak = build_fjord_mask(&fld, &cold, &shale, &coast_d, w, h, 0.42, o);
        for (a, b) in weak.iter().zip(base.iter()) {
            assert!(a <= b, "shale must never out-mask granite");
        }
        assert!(weak.iter().zip(base.iter()).any(|(a, b)| a < b), "lithology must actually be read");
    }
}
