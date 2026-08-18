//! Regional amplification — `UNIFIED_TOOL_PLAN.md` milestone E, the compute
//! half of the Region select/export tool.
//!
//! Ports `amplifyRegion` (reference line 10265) and `refineTile` (10305), the
//! two pure functions behind both of the reference's region operations: the
//! tiled `Refine & export` (`exportRegionTiles` → `refineTile`) and
//! `regionNewWorldBtn`'s "replace the world with a higher-resolution version
//! of this region" (`amplifyRegion` directly). The reference's own header
//! calls them *"Pure function: takes src explicitly (no globals) so it is
//! worker-ready"* and *"All headless-testable"* — which is exactly why they
//! port cleanly while the `.zip`/PNG assembly around them does not.
//!
//! **Why `cartalith-terrain`.** Milestone B's third placement category:
//! subsystem-domain math belongs to the crate that owns the field. This is an
//! upsample of a height field plus world-space fBm detail tapered by local
//! relief and faded out underwater — height formula, start to finish, over the
//! same `fbm`/`ridged` this crate already generates terrain with.
//! `cartalith-engine` would be wrong for milestone B's reason
//! (*"cartalith-engine orchestrates; it does not compute"*), and
//! `cartalith-spatial` for milestone C's (a sea-level-aware relief taper is
//! not generic machinery). The *selection* rectangle it consumes is generic
//! and does live in `cartalith-spatial` ([`FloatRegion`]).
//!
//! # The seam property, and why the region is fractional
//!
//! Both the upsample and the detail are pure functions of the shared **coarse**
//! coordinate `(cx, cy)`, so two tiles that overlap by exactly one coarse
//! column agree bit-for-bit along that shared edge. `refine_tile` reproduces
//! that overlap (`w: stepX + 1`), and `stepX = region.w / cols` is generally
//! **not** an integer — rounding the sub-bounds to whole cells would break the
//! agreement outright, which is why [`FloatRegion`] exists.
//!
//! # A real division by zero, ported rather than fixed
//!
//! `cy = rh > 1 ? ry + (oy/(outH-1))*(rh-1) : ry` divides by `outH - 1`. With
//! `outH == 1` **and** `rh > 1` that is `0/0`, and the entire output comes back
//! `NaN` — verified against the reference, not inferred. No shipped caller
//! reaches it, because `tile_dims` floors both edges at 2px and
//! `regionNewWorldBtn` goes through `tile_dims` too. Ported as written
//! (`DECISIONS.md`'s rule: the reference's behaviour is the specification), and
//! pinned by a golden case so nobody later "fixes" the port into disagreeing.

use cartalith_spatial::FloatRegion;

use crate::sculpt::js_hypot;

/// `amplifyRegion`'s `opts` bag, with the reference's own defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmplifyOpts {
    /// `opts.seed`, `|0`-truncated by the reference. Default 1234.
    pub seed: i32,
    /// Noise cycles per coarse cell at octave 0. Default 1.0.
    pub detail_freq: f64,
    /// Maximum added relief in normalised height. Default 0.14.
    pub detail_amp: f64,
    /// Sea level, below which detail is faded out. Default 0.42.
    pub sea: f64,
    /// Use `ridged` instead of `fbm` for the detail term. Default false.
    pub ridged: bool,
}

impl Default for AmplifyOpts {
    fn default() -> Self {
        AmplifyOpts { seed: 1234, detail_freq: 1.0, detail_amp: 0.14, sea: 0.42, ridged: false }
    }
}

/// `Math.min` semantics, which differ from Rust's `f64::min` on NaN: JS
/// propagates it, Rust returns the other operand. `amplifyRegion`'s
/// `Math.min(1, hypot(gx,gy)*8)` is reached with NaN inputs by the `outW == 1`
/// case above, and `f64::min` there would silently turn an all-NaN tile into a
/// plausible-looking one.
#[inline]
pub(crate) fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}

/// `Math.max` with the same NaN propagation as [`js_min`].
#[inline]
pub(crate) fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else {
        b
    }
}

/// The reference's inline bilinear sampler over the coarse source, clamped at
/// every edge. Reads `f32` storage, computes in `f64` — the same split as the
/// rest of this crate.
#[inline]
fn samp(src: &[f32], src_w: usize, src_h: usize, fx: f64, fy: f64) -> f64 {
    let (sw, sh) = (src_w as f64, src_h as f64);
    let fx = fx.clamp(0.0, sw - 1.0);
    let fy = fy.clamp(0.0, sh - 1.0);
    // `fx|0` truncates toward zero; fx is non-negative here after the clamp,
    // and NaN truncates to 0 exactly as `NaN|0` does in JS.
    let x0 = fx as i64 as usize;
    let y0 = fy as i64 as usize;
    let x1 = if x0 < src_w - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < src_h - 1 { y0 + 1 } else { y0 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let (a, b) = (src[y0 * src_w + x0] as f64, src[y0 * src_w + x1] as f64);
    let (c, d) = (src[y1 * src_w + x0] as f64, src[y1 * src_w + x1] as f64);
    (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (c * (1.0 - tx) + d * tx) * ty
}

/// `amplifyRegion(src, srcW, srcH, region, outW, outH, opts)` (reference line
/// 10265).
///
/// Refines `region` of the coarse field `src` to `out_w × out_h`: bilinear
/// upsample of the coarse constraint (which keeps continents and ranges),
/// plus world-space high-frequency detail, tapered by local relief so plains
/// and ocean floors stay smooth, and faded out below the shelf.
///
/// # Panics
///
/// Panics if `src.len() < src_w * src_h`, or if either source dimension is
/// zero — the reference indexes `src` unchecked and would read `undefined`.
// The reference's own signature, argument for argument: grouping them into a
// struct would put a coordinate frame and a noise configuration in one bag and
// make the port harder to check against `amplifyRegion` line by line.
#[allow(clippy::too_many_arguments)]
pub fn amplify_region(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    out_w: usize,
    out_h: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    assert!(src_w > 0 && src_h > 0, "amplify_region needs a non-empty source field");
    assert!(
        src.len() >= src_w * src_h,
        "amplify_region source is {} cells, needs {}",
        src.len(),
        src_w * src_h
    );
    let FloatRegion { x: rx, y: ry, w: rw, h: rh } = *region;
    let mut out = vec![0.0f32; out_w * out_h];
    for oy in 0..out_h {
        let cy = if rh > 1.0 {
            ry + (oy as f64 / (out_h as f64 - 1.0)) * (rh - 1.0)
        } else {
            ry
        };
        for ox in 0..out_w {
            let cx = if rw > 1.0 {
                rx + (ox as f64 / (out_w as f64 - 1.0)) * (rw - 1.0)
            } else {
                rx
            };
            let base = samp(src, src_w, src_h, cx, cy);
            // local relief from the coarse field -> detail amplitude
            let e = 1.0;
            let gx = (samp(src, src_w, src_h, cx + e, cy) - samp(src, src_w, src_h, cx - e, cy)) * 0.5;
            let gy = (samp(src, src_w, src_h, cx, cy + e) - samp(src, src_w, src_h, cx, cy - e)) * 0.5;
            let relief = js_min(1.0, js_hypot(gx, gy) * 8.0);
            // fade detail out below the shelf. `NaN < sea` is false in both
            // languages, so a NaN base takes the `0` branch identically.
            let underwater = if base < opts.sea {
                js_max(0.0, (opts.sea - base) / 0.06)
            } else {
                0.0
            };
            let taper = relief * js_max(0.0, 1.0 - underwater);
            // detail sampled in continuous coarse coords -> seamless across
            // tiles sharing (cx, cy)
            let d = if opts.ridged {
                cartalith_noise::ridged(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
            } else {
                cartalith_noise::fbm(cx * opts.detail_freq, cy * opts.detail_freq, opts.seed)
            } - 0.5;
            let v = base + d * opts.detail_amp * taper;
            // The reference's own `v<0?0:v>1?1:v`. `f64::clamp` is that
            // expression exactly, NaN included: NaN fails both comparisons and
            // falls through unchanged, which is what makes the `out_w == 1`
            // case observable at all.
            out[oy * out_w + ox] = v.clamp(0.0, 1.0) as f32;
        }
    }
    out
}

/// `refineTile(src, srcW, srcH, region, cols, rows, col, row, tileW, tileH,
/// opts)` (reference line 10305).
///
/// Splits `region` into a `cols × rows` grid and amplifies one tile of it.
/// Each tile's coarse sub-bounds **overlap its neighbour by exactly one coarse
/// column/row** (`w: stepX + 1`), so the shared edge pixels map to the same
/// coarse coordinate and, because the detail is sampled in continuous coarse
/// coords, adjacent tiles agree exactly — seam delta zero, asserted by test.
#[allow(clippy::too_many_arguments)]
pub fn refine_tile(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    region: &FloatRegion,
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
    tile_w: usize,
    tile_h: usize,
    opts: &AmplifyOpts,
) -> Vec<f32> {
    let step_x = region.w / cols as f64;
    let step_y = region.h / rows as f64;
    let sub = FloatRegion {
        x: region.x + col as f64 * step_x,
        y: region.y + row as f64 * step_y,
        w: step_x + 1.0,
        h: step_y + 1.0,
    };
    amplify_region(src, src_w, src_h, &sub, tile_w, tile_h, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::Region;

    /// The same synthetic field the golden harness builds, in the same order:
    /// pure arithmetic (no `sin`/`cos`/`exp`) so V8 and Rust cannot disagree,
    /// with a deliberately **quantised** `% 11` term.
    pub(crate) fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
        let mut f = vec![0.0f32; gw * gh];
        let cx = gw as f64 * 0.42;
        let cy = gh as f64 * 0.55;
        let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
        for y in 0..gh {
            for x in 0..gw {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
                let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
                v += 0.05 * ((q as f64 / 10.0) - 0.5);
                v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
                f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    #[test]
    fn output_is_exactly_out_w_times_out_h() {
        let src = synthetic_field(16, 12, 0);
        let o = amplify_region(&src, 16, 12, &Region { x: 2, y: 2, w: 8, h: 6 }.to_float(), 20, 15,
                               &AmplifyOpts::default());
        assert_eq!(o.len(), 300);
    }

    #[test]
    fn every_sample_stays_inside_the_unit_range() {
        let src = synthetic_field(24, 18, 3);
        let o = amplify_region(&src, 24, 18, &Region { x: 0, y: 0, w: 24, h: 18 }.to_float(), 40, 30,
                               &AmplifyOpts { detail_amp: 5.0, ..Default::default() });
        assert!(o.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    #[test]
    fn zero_detail_amplitude_is_a_pure_upsample() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let with = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts { detail_amp: 0.0, ..Default::default() });
        let flat = amplify_region(&src, 16, 12, &reg, 20, 16,
                                  &AmplifyOpts { detail_amp: 0.0, ridged: true, seed: 999, ..Default::default() });
        // With no amplitude the noise family and seed cannot matter at all.
        assert_eq!(with, flat);
    }

    #[test]
    fn the_detail_actually_changes_the_result() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let plain = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        let none = amplify_region(&src, 16, 12, &reg, 20, 16,
                                  &AmplifyOpts { detail_amp: 0.0, ..Default::default() });
        assert_ne!(plain, none, "detail_amp 0.14 produced the same field as 0.0");
    }

    #[test]
    fn ridged_and_fbm_detail_differ() {
        let src = synthetic_field(16, 12, 1);
        let reg = Region { x: 1, y: 1, w: 10, h: 8 }.to_float();
        let a = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        let b = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts { ridged: true, ..Default::default() });
        assert_ne!(a, b);
    }

    #[test]
    fn adjacent_tiles_agree_exactly_on_their_shared_edge() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (tw, th) = (16, 12);
        let left = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 0, tw, th, &o);
        let right = refine_tile(&src, 48, 32, &reg, 2, 2, 1, 0, tw, th, &o);
        for y in 0..th {
            assert_eq!(
                left[y * tw + (tw - 1)].to_bits(),
                right[y * tw].to_bits(),
                "seam mismatch at row {y}"
            );
        }
    }

    #[test]
    fn vertically_adjacent_tiles_agree_too() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts { seed: 4242, ..Default::default() };
        let (tw, th) = (16, 12);
        let top = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 0, tw, th, &o);
        let bottom = refine_tile(&src, 48, 32, &reg, 2, 2, 0, 1, tw, th, &o);
        for x in 0..tw {
            assert_eq!(top[(th - 1) * tw + x].to_bits(), bottom[x].to_bits(), "seam mismatch at col {x}");
        }
    }

    #[test]
    fn a_collapsed_region_samples_one_coarse_point_everywhere() {
        let src = synthetic_field(16, 12, 2);
        let o = amplify_region(&src, 16, 12, &FloatRegion { x: 5.0, y: 5.0, w: 1.0, h: 1.0 }, 6, 6,
                               &AmplifyOpts::default());
        // rw <= 1 and rh <= 1 both take the `: rx` / `: ry` branch, so every
        // output cell reads the same coarse coordinate -- constant, not NaN.
        assert!(o.iter().all(|v| v.to_bits() == o[0].to_bits()));
        assert!(o[0].is_finite());
    }

    #[test]
    fn a_single_pixel_output_over_a_real_region_is_all_nan_like_the_reference() {
        let src = synthetic_field(16, 12, 2);
        let o = amplify_region(&src, 16, 12, &Region { x: 2, y: 2, w: 10, h: 10 }.to_float(), 1, 1,
                               &AmplifyOpts::default());
        // 0/0 in the coordinate mapping. tile_dims' max(2, ..) floor is why no
        // shipped caller reaches this; pinned so it is not silently "fixed".
        assert_eq!(o.len(), 1);
        assert!(o[0].is_nan());
    }

    #[test]
    fn js_min_propagates_nan_where_rusts_own_min_would_not() {
        assert!(js_min(1.0, f64::NAN).is_nan());
        assert_eq!(1.0f64.min(f64::NAN), 1.0, "Rust's f64::min still swallows NaN");
        assert_eq!(js_min(1.0, 0.5), 0.5);
    }

    #[test]
    fn js_max_propagates_nan_too() {
        assert!(js_max(0.0, f64::NAN).is_nan());
        assert_eq!(js_max(0.0, 3.0), 3.0);
    }

    #[test]
    fn the_sampler_clamps_rather_than_wrapping_at_every_edge() {
        let src = synthetic_field(8, 6, 0);
        assert_eq!(samp(&src, 8, 6, -5.0, -5.0), src[0] as f64);
        assert_eq!(samp(&src, 8, 6, 99.0, 99.0), src[5 * 8 + 7] as f64);
    }

    #[test]
    fn underwater_cells_lose_their_detail() {
        // A field entirely below sea level gets `underwater` >= 1 everywhere,
        // so taper is 0 and the output is the pure upsample.
        let src = vec![0.10f32; 16 * 12];
        let reg = Region { x: 0, y: 0, w: 16, h: 12 }.to_float();
        let o = amplify_region(&src, 16, 12, &reg, 20, 16, &AmplifyOpts::default());
        assert!(o.iter().all(|v| (*v - 0.10).abs() < 1e-7));
    }

    #[test]
    fn refine_tile_covers_the_whole_region_across_its_grid() {
        let src = synthetic_field(48, 32, 5);
        let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
        let o = AmplifyOpts::default();
        // The last tile's sub-bounds must end one coarse cell past the region's
        // own far edge minus the overlap -- i.e. reach region.x + region.w.
        let step_x = reg.w / 3.0;
        let last_x = reg.x + 2.0 * step_x + (step_x + 1.0);
        assert_eq!(last_x, reg.x + reg.w + 1.0);
        let t = refine_tile(&src, 48, 32, &reg, 3, 3, 2, 2, 8, 8, &o);
        assert_eq!(t.len(), 64);
    }

    #[test]
    #[should_panic(expected = "non-empty source field")]
    fn an_empty_source_is_rejected_rather_than_read_out_of_bounds() {
        amplify_region(&[], 0, 0, &FloatRegion { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, 4, 4,
                       &AmplifyOpts::default());
    }

    #[test]
    fn the_defaults_are_the_references_own() {
        let d = AmplifyOpts::default();
        assert_eq!(d.seed, 1234);
        assert_eq!(d.detail_freq, 1.0);
        assert_eq!(d.detail_amp, 0.14);
        assert_eq!(d.sea, 0.42);
        assert!(!d.ridged);
    }
}
