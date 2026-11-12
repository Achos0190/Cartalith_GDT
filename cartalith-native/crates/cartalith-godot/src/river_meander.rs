//! **The display meander: a bounded wander on the drawn river for close zoom**
//! (owner Ruling ER, 2026-11-11, part B; `DELTA_GEOMETRY_RESEARCH.md` section
//! 13).
//!
//! **Why.** Even with rough routing (part A) a river is traced through cell
//! centres and drawn as a smooth spline through them, so at x16 and beyond,
//! where a cell is 7 to 30 pixels on the owner's world, a stretch of a few
//! cells still reads as a drawn ruler line. A real channel wanders inside its
//! valley at every scale. [`display_meander`] adds that wander to the DRAWN
//! curve only, bounded so it never leaves the cells the river was traced
//! through.
//!
//! **How it fades in by zoom.** The wander has a fixed size on the ground
//! ([`MEANDER_AMP_CELLS`]), so on screen it grows with the zoom: 0.14 pixel at
//! the owner world's opening view (0.46 pixel a cell), 0.55 at x4, 2.2 at x16
//! and 8.8 at x64. It is invisible until about x16 by construction, and it
//! needs no second law: it is part of the one drawn geometry
//! (`river_stroke::RiverGeometry`) that the screen's field, the deep-zoom
//! tiles, the export and picking all read, so they agree at every zoom, and
//! the visibility law (`river_stroke::river_visibility`) is untouched (no
//! shader is changed).
//!
//! **Must never** move a drawn point out of the set of cells its run was
//! traced through (it shrinks the offset until the point is inside one, or
//! drops it); move a pinned point (a junction, a bridge) or a run's two ends,
//! where the curve must meet another run or the shore exactly (the wander is
//! faded to zero over half a wavelength round each); read or write anything a
//! simulation, a save or a golden reads (render only); or depend on anything
//! but the curve, the traced cells and the run's own first cell (deterministic,
//! the same on every path).

use std::collections::HashSet;

/// The wander's amplitude on the ground, in grid cells. **Labelled
/// judgement** (section 13): under half a cell, so a point on a cell's
/// centreline stays inside that cell (the bound is enforced point by point
/// anyway, [`display_meander`]); 0.3 leaves a fifth of a cell of margin for
/// the spline's own offset from the cell centres. At x16 on the owner world
/// that is 2.2 pixels each side, at x64 8.8.
pub const MEANDER_AMP_CELLS: f64 = 0.3;

/// The wander's wavelength along the river, in grid cells. **Labelled
/// judgement** (section 13): real meander wavelengths are about 11 channel
/// widths (Leopold & Wolman 1960, the figure `river_delta`'s
/// `MEANDER_WAVELENGTH_WIDTHS` cites), and the drawn rivers are 0.1 to 0.6
/// cells wide, which would be one to seven cells; 4 cells is in that range
/// and long enough that the spline's quarter-cell points draw each bend with
/// sixteen, so a bend never reads as a kink. With [`MEANDER_AMP_CELLS`] the
/// sinuosity this adds is about `1 + (pi * 0.3 / 4)^2` = 1.06, the low end of
/// a real channel's.
pub const MEANDER_WAVELENGTH_CELLS: f64 = 4.0;

/// **RR-8 (Ruling ES, render only): a wide channel bends in long arcs.** The
/// wavelength at a point is `max(MEANDER_WAVELENGTH_CELLS,
/// river_delta::MEANDER_WAVELENGTH_WIDTHS * width)`, `width` the drawn full
/// width in cells there (RR-1's discharge law), so a trunk wanders in bends of
/// about eleven of its own widths (Leopold & Wolman 1960, 7-11 widths, cited
/// UNCONFIRMED in `river_delta.rs`) and a trickle keeps the 4-cell floor.
/// The phase is the integral of `2 pi / wavelength` along the arc, so the curve
/// stays smooth where the width changes. The amplitude is unchanged (still
/// inside the traced cells).
pub fn wavelength_cells(full_width_cells: f64) -> f64 {
    if full_width_cells.is_finite() {
        MEANDER_WAVELENGTH_CELLS.max(crate::river_delta::MEANDER_WAVELENGTH_WIDTHS as f64 * full_width_cells)
    } else {
        MEANDER_WAVELENGTH_CELLS
    }
}

/// How often the offset is halved before a point that will not fit inside a
/// traced cell is left where it was: 0.3, 0.15, 0.075, then none.
const SHRINK_STEPS: usize = 3;

/// SplitMix64's finaliser: the run's phase from its first traced cell.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// **Displaces the drawn curve `rp` of one run** sideways by a sine of
/// [`MEANDER_AMP_CELLS`] and [`MEANDER_WAVELENGTH_CELLS`] along its arc
/// length, with a phase drawn from the run's first traced cell, faded to zero
/// over half a wavelength round each end and each point `pinned` says must not
/// move (smoothstep), and clamped point by point so every displaced point
/// lies in a cell of `traced` (the run's traced cell centres, river space: a
/// cell's centre is `x + 0.5`). A run of fewer than three points, or with no
/// traced cells, is left as it is.
///
/// `widths` (RR-8): the drawn full width in cells at each traced point (read
/// at the traced point an arc-length fraction along the run), which sets the
/// local wavelength ([`wavelength_cells`]); empty keeps every bend at
/// [`MEANDER_WAVELENGTH_CELLS`], the pre-RR-8 curve exactly.
pub fn display_meander(rp: &mut [(f64, f64)], traced: &[(f64, f64)], widths: &[f64], pinned: impl Fn((f64, f64)) -> bool) {
    let n = rp.len();
    if n < 3 || traced.is_empty() {
        return;
    }
    let cell = |p: (f64, f64)| (p.0.floor() as i64, p.1.floor() as i64);
    let cells: HashSet<(i64, i64)> = traced.iter().map(|&p| cell(p)).collect();
    let mut s = vec![0.0f64; n];
    for k in 1..n {
        s[k] = s[k - 1] + ((rp[k].0 - rp[k - 1].0).powi(2) + (rp[k].1 - rp[k - 1].1).powi(2)).sqrt();
    }
    // Arc length to the nearest fixed point (an end or a pinned point), both ways.
    let fixed: Vec<bool> = (0..n).map(|k| k == 0 || k == n - 1 || pinned(rp[k])).collect();
    let mut to_fixed = vec![f64::INFINITY; n];
    let mut last: Option<f64> = None;
    for k in 0..n {
        if fixed[k] {
            last = Some(s[k]);
        }
        if let Some(l) = last {
            to_fixed[k] = s[k] - l;
        }
    }
    last = None;
    for k in (0..n).rev() {
        if fixed[k] {
            last = Some(s[k]);
        }
        if let Some(l) = last {
            to_fixed[k] = to_fixed[k].min(l - s[k]);
        }
    }
    let (c0x, c0y) = cell(traced[0]);
    let phase = (mix((c0x as u64) << 32 ^ (c0y as u64 & 0xFFFF_FFFF)) >> 11) as f64 / (1u64 << 53) as f64 * std::f64::consts::TAU;
    let total = s[n - 1].max(1e-9);
    let lam = |k: usize| -> f64 {
        if widths.is_empty() {
            return MEANDER_WAVELENGTH_CELLS;
        }
        let t = ((s[k] / total) * (widths.len() - 1) as f64).round() as usize;
        wavelength_cells(widths[t.min(widths.len() - 1)])
    };
    // The phase integral along the arc (RR-8); with no widths, `2 pi s / 4` exactly.
    let mut theta = vec![0.0f64; n];
    for k in 1..n {
        theta[k] = if widths.is_empty() { std::f64::consts::TAU * s[k] / MEANDER_WAVELENGTH_CELLS } else { theta[k - 1] + std::f64::consts::TAU * (s[k] - s[k - 1]) / lam(k) };
    }
    let orig: Vec<(f64, f64)> = rp.to_vec();
    for k in 1..n - 1 {
        let half = 0.5 * lam(k);
        let t = (to_fixed[k] / half).clamp(0.0, 1.0);
        let fade = t * t * (3.0 - 2.0 * t);
        if fade <= 0.0 {
            continue;
        }
        let (tx, ty) = (orig[k + 1].0 - orig[k - 1].0, orig[k + 1].1 - orig[k - 1].1);
        let tl = (tx * tx + ty * ty).sqrt();
        if tl <= 0.0 {
            continue;
        }
        let (nx, ny) = (-ty / tl, tx / tl);
        let mut off = MEANDER_AMP_CELLS * fade * (theta[k] + phase).sin();
        for _ in 0..=SHRINK_STEPS {
            let q = (orig[k].0 + nx * off, orig[k].1 + ny * off);
            if cells.contains(&cell(q)) {
                rp[k] = q;
                break;
            }
            off *= 0.5;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight traced run east along row 10 (cell centres 0.5 .. 39.5), and
    /// its drawn curve at quarter-cell steps (what the spline gives a straight
    /// run).
    fn straight() -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let traced: Vec<(f64, f64)> = (0..40).map(|x| (x as f64 + 0.5, 10.5)).collect();
        let rp: Vec<(f64, f64)> = (0..=156).map(|j| (0.5 + j as f64 * 0.25, 10.5)).collect();
        (traced, rp)
    }

    /// Protects RR-8 (Ruling ES): the wavelength is eleven widths above the
    /// 4-cell floor, the floor for a trickle and for a missing width, and a
    /// run given wide widths bends less often than the same run at the floor.
    #[test]
    fn a_wide_channel_bends_in_longer_arcs() {
        assert_eq!(wavelength_cells(0.1), 4.0);
        assert_eq!(wavelength_cells(1.0), 11.0);
        assert_eq!(wavelength_cells(f64::NAN), 4.0);
        let (traced, rp0) = straight();
        let crossings = |w: &[f64]| {
            let mut rp = rp0.clone();
            display_meander(&mut rp, &traced, w, |_| false);
            rp.windows(2).filter(|p| (p[0].1 - 10.5).signum() != (p[1].1 - 10.5).signum() && (p[1].1 - 10.5).abs() > 1e-9).count()
        };
        let narrow = crossings(&[]);
        let wide = crossings(&vec![1.0; traced.len()]);
        assert!(narrow > 0 && wide < narrow, "a 1-cell channel crosses its axis less often ({wide} vs {narrow})");
    }

    /// Protects: the two constants at LITERAL values (a retune must be on
    /// purpose and re-judged at x16 and x64).
    #[test]
    fn the_meander_constants_are_pinned_by_literals() {
        assert_eq!((MEANDER_AMP_CELLS, MEANDER_WAVELENGTH_CELLS, SHRINK_STEPS), (0.3, 4.0, 3));
    }

    /// Protects: the point of it and its bound. A straight run gains a wander
    /// of at most 0.3 cells that does reach close to it mid-run (more than
    /// 0.25), every displaced point stays in a traced cell, the two ends do
    /// not move, and the wander is zero within a fraction of a cell of the
    /// ends (the fade).
    #[test]
    fn a_straight_run_wanders_inside_its_cells_and_keeps_its_ends() {
        let (traced, mut rp) = straight();
        let orig = rp.clone();
        display_meander(&mut rp, &traced, &[], |_| false);
        assert_eq!(rp[0], orig[0]);
        assert_eq!(rp[rp.len() - 1], orig[orig.len() - 1]);
        let dev: Vec<f64> = rp.iter().map(|p| (p.1 - 10.5).abs()).collect();
        assert!(dev.iter().all(|&d| d <= 0.3 + 1e-12), "{:?}", dev.iter().cloned().fold(0.0, f64::max));
        assert!(dev.iter().cloned().fold(0.0, f64::max) > 0.25, "it wanders");
        assert!(rp.iter().all(|p| p.1.floor() == 10.0), "inside the traced row");
        assert!(dev[1] < 0.05 && dev[dev.len() - 2] < 0.05, "faded at the ends: {} {}", dev[1], dev[dev.len() - 2]);
    }

    /// Protects: the bound when the cells do not allow the full wander. A run
    /// whose traced row is row 10 but whose curve sits 0.4 cells off-centre
    /// (as an RDP-simplified spline can) is only displaced as far as the row
    /// allows: no point leaves row 10.
    #[test]
    fn a_point_that_would_leave_the_traced_cells_is_held_inside() {
        let (traced, rp0) = straight();
        let mut rp: Vec<(f64, f64)> = rp0.iter().map(|p| (p.0, 10.9)).collect();
        display_meander(&mut rp, &traced, &[], |_| false);
        assert!(rp.iter().all(|p| p.1.floor() == 10.0), "a point left the traced cells");
        assert!(rp.iter().any(|p| (p.1 - 10.9).abs() > 0.05), "the inward half of the wander is still drawn");
    }

    /// Protects: a pinned point (a junction) does not move and the curve is
    /// faded round it; determinism (same input, same output); and the phase
    /// differs between runs that start in different cells.
    #[test]
    fn pinned_points_hold_and_the_wander_is_deterministic() {
        let (traced, rp0) = straight();
        let pin = rp0[80];
        let mut a = rp0.clone();
        display_meander(&mut a, &traced, &[], |p| p == pin);
        assert_eq!(a[80], pin);
        assert!((a[79].1 - 10.5).abs() < 0.02 && (a[81].1 - 10.5).abs() < 0.02);
        let mut b = rp0.clone();
        display_meander(&mut b, &traced, &[], |p| p == pin);
        assert_eq!(a, b);
        let traced2: Vec<(f64, f64)> = traced.iter().map(|p| (p.0, p.1 + 5.0)).collect();
        let mut c: Vec<(f64, f64)> = rp0.iter().map(|p| (p.0, p.1 + 5.0)).collect();
        display_meander(&mut c, &traced2, &[], |_| false);
        let mut a2 = rp0.clone();
        display_meander(&mut a2, &traced, &[], |_| false);
        assert!(a2.iter().zip(&c).any(|(p, q)| ((p.1 - 10.5) - (q.1 - 15.5)).abs() > 0.05), "another run, another phase");
    }

    /// Protects: a run too short to bend is left alone.
    #[test]
    fn a_two_point_run_is_unchanged() {
        let traced = vec![(0.5, 0.5), (1.5, 0.5)];
        let mut rp = traced.clone();
        display_meander(&mut rp, &traced, &[], |_| false);
        assert_eq!(rp, traced);
    }
}
