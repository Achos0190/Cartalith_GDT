//! Landmass centering — the reference's `#centerBtn` (HTML lines 3150-3199).
//!
//! World mode is a cylinder: it wraps in X (longitude) only, never in Y.
//! A continent straddling the `x=0`/`x=GW` seam therefore shows as slivers
//! on both edges. The fix is a pure circular shift in X that moves the
//! emptiest meridian to the edge — **never** a Y shift, which would
//! teleport the poles and corrupt the climate.
//!
//! Three pure kernels live here; the orchestration that applies them to
//! every retained grid at once is `cartalith_engine::center::center_landmasses`
//! (this crate owns no `WorldState`).

/// `bestEmptyColumn` (reference line 3156) — the X column with the least
/// land, i.e. the least destructive place to put the seam.
///
/// `geo` is the reference's optional geoid field (`geo?geo[i]:0`). This
/// port has no geoid (`cartalith-climate`'s own note, `GUI_GAP_REGISTER.md`
/// WW-07), so every caller currently passes `None` — the parameter is kept
/// because the reference's own call site passes `geoidField`, and dropping
/// it would make a future geoid a silent behaviour change rather than a
/// one-line wiring.
///
/// Ties go to the **lowest** column: the reference's `cnt<bestV` is strict,
/// so the first column reaching a given count wins.
pub fn best_empty_column(fld: &[f32], geo: Option<&[f32]>, w: usize, h: usize, sea: f64) -> usize {
    let mut best_x = 0usize;
    let mut best_v = usize::MAX;
    for x in 0..w {
        let mut cnt = 0usize;
        for y in 0..h {
            let i = y * w + x;
            let g = geo.map_or(0.0, |g| g[i] as f64);
            if fld[i] as f64 - g > sea {
                cnt += 1;
            }
        }
        if cnt < best_v {
            best_v = cnt;
            best_x = x;
        }
    }
    best_x
}

/// `shiftGridX` (reference line 3161) — circular-shift one grid array in X
/// by `off` columns, in place. Rows are untouched.
///
/// New column `x` reads old column `(x + off) % W`, so old column `off`
/// (the emptiest one) lands at new column 0. `off` is reduced modulo `W`
/// the way JS's `((off%W)+W)%W` does, so negative and over-wide offsets
/// are legal and `off == 0` is a no-op.
///
/// Generic over the element type because the reference shifts
/// `Float32Array`, `Uint8Array` and `Int32Array` grids through this same
/// function (`new arr.constructor(W)` is how it allocates its row buffer).
pub fn shift_grid_x<T: Copy + Default>(arr: &mut [T], w: usize, h: usize, off: isize) {
    if w == 0 || h == 0 {
        return;
    }
    let off = (off.rem_euclid(w as isize)) as usize;
    if off == 0 {
        return;
    }
    let mut buf: Vec<T> = vec![T::default(); w];
    for y in 0..h {
        let r = y * w;
        for (x, b) in buf.iter_mut().enumerate() {
            *b = arr[r + (x + off) % w];
        }
        arr[r..r + w].copy_from_slice(&buf);
    }
}

/// `featherSeamX` (reference line 3171) — a wrap-aware horizontal box
/// smooth of a narrow column band around `col`, dissolving the seam
/// discontinuity the shift relocated into the map interior.
///
/// The world is only *approximately* periodic in X (the reference's own
/// Invariant 9: seam wrap-delta < 0.12), so after a shift the original
/// `x=0 ↔ x=W-1` join sits at roughly column `W - off` and reads as a
/// straight vertical line. This blurs it away.
///
/// `half_w == 0` is substituted with `2`, matching the reference's
/// `halfW=halfW||2` JS truthiness — the same substitution `smoothstep`'s
/// `||1e-6` needs and gets in `sculpt.rs`.
///
/// Sums accumulate in `f64` (the reference's own `Float64Array buf`) and
/// round to `f32` only at the point of storage, which is where JS's
/// `Float32Array` assignment rounds too.
pub fn feather_seam_x(arr: &mut [f32], w: usize, h: usize, col: usize, half_w: usize) {
    if w == 0 || h == 0 {
        return;
    }
    let half_w = if half_w == 0 { 2 } else { half_w } as isize;
    let span = 2 * half_w + 1;
    let wi = w as isize;
    let col = col as isize;
    let mut buf = vec![0f64; span as usize];
    for y in 0..h {
        let r = y * w;
        for k in -half_w..=half_w {
            let mut s = 0f64;
            for j in -half_w..=half_w {
                let xx = (col + k + j).rem_euclid(wi) as usize;
                s += arr[r + xx] as f64;
            }
            buf[(k + half_w) as usize] = s / span as f64;
        }
        for k in -half_w..=half_w {
            let x = (col + k).rem_euclid(wi) as usize;
            arr[r + x] = buf[(k + half_w) as usize] as f32;
        }
    }
}

/// The column the relocated seam lands on after shifting by `off`
/// (reference line 3192's `sc=((GW-off)%GW+GW)%GW`) — the one
/// `feather_seam_x` must be pointed at.
pub fn seam_column(w: usize, off: usize) -> usize {
    if w == 0 {
        return 0;
    }
    (w as isize - off as isize).rem_euclid(w as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_by_zero_and_by_a_full_width_are_both_no_ops() {
        let src: Vec<f32> = (0..12).map(|i| i as f32).collect();
        for off in [0isize, 4, -4, 8] {
            let mut a = src.clone();
            shift_grid_x(&mut a, 4, 3, off);
            if off.rem_euclid(4) == 0 {
                assert_eq!(a, src, "off {off} reduces to 0 and must not move anything");
            } else {
                assert_ne!(a, src);
            }
        }
    }

    #[test]
    fn negative_and_oversized_offsets_reduce_the_same_way_js_does() {
        let src: Vec<f32> = (0..8).map(|i| i as f32).collect();
        let mut a = src.clone();
        let mut b = src.clone();
        shift_grid_x(&mut a, 4, 2, -1);
        shift_grid_x(&mut b, 4, 2, 3);
        assert_eq!(a, b, "-1 and +3 are the same offset modulo 4");
        let mut c = src.clone();
        shift_grid_x(&mut c, 4, 2, 7);
        assert_eq!(c, b, "7 and 3 are the same offset modulo 4");
    }

    #[test]
    fn seam_column_is_the_edge_when_nothing_moved() {
        assert_eq!(seam_column(48, 0), 0);
        assert_eq!(seam_column(48, 27), 21);
        assert_eq!(seam_column(48, 48), 0);
    }

    #[test]
    fn a_zero_half_width_feathers_two_columns_either_side_not_none() {
        let mut a = vec![0f32, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let mut b = a.clone();
        feather_seam_x(&mut a, 8, 1, 3, 0);
        feather_seam_x(&mut b, 8, 1, 3, 2);
        assert_eq!(a, b, "half_w 0 must substitute 2, matching `halfW||2`");
        assert_ne!(a, vec![0f32, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    }
}
