//! **Straight river runs, measured** (owner, 2026-11-11). A diagnostic for the probe (`WorldGen::river_straightness`);
//! nothing draws from it.

/// **Straight river runs, measured** (owner, 2026-11-11: *"Investigate why sometimes we still get straight
/// lines for rivers"*; `DELTA_GEOMETRY_RESEARCH.md` section 11). Over each polyline, the maximal stretches
/// whose every point lies within `tol` cells of the stretch's own chord, at least `min_len` cells long.
/// A pure measurement: never read by drawing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StraightStats {
    /// Total polyline length measured, cells.
    pub total: f64,
    /// Length lying in straight stretches, cells.
    pub straight: f64,
    /// Straight stretches found.
    pub count: usize,
    /// Of `straight`, the length whose chord is within [`AXIS_TOL_DEG`] of a grid axis or diagonal
    /// (a multiple of 45 degrees): the D8 signature.
    pub axis: f64,
    /// The longest stretch, cells.
    pub longest: f64,
}

/// A straight stretch's chord counts as grid-aligned within this many degrees of a multiple of 45.
/// **Labelled judgement**: a D8 run's chord is exactly on a multiple of 45; 3 degrees admits a run that
/// steps once off its axis over twenty cells (atan(1/20) = 2.9 degrees).
pub const AXIS_TOL_DEG: f64 = 3.0;

/// Measures [`StraightStats`] over `lines` (grid cells). `O(n^2)` per line in the worst case, bounded by
/// stopping a stretch at the first point outside `tol`.
pub fn straight_stats<'a>(lines: impl IntoIterator<Item = &'a [(f64, f64)]>, min_len: f64, tol: f64) -> StraightStats {
    let mut st = StraightStats::default();
    let d = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    for p in lines {
        let n = p.len();
        st.total += p.windows(2).map(|w| d(w[0], w[1])).sum::<f64>();
        let mut i = 0;
        while i + 1 < n {
            // Grow j while every point of i..=j stays within tol of the chord p[i] -> p[j].
            let mut best = i + 1;
            let mut j = i + 2;
            while j < n {
                let (a, b) = (p[i], p[j]);
                let l = d(a, b);
                if l < 1e-9 {
                    j += 1;
                    continue;
                }
                let ok = (i + 1..j).all(|k| ((b.0 - a.0) * (a.1 - p[k].1) - (a.0 - p[k].0) * (b.1 - a.1)).abs() / l <= tol);
                if !ok {
                    break;
                }
                best = j;
                j += 1;
            }
            let len: f64 = p[i..=best].windows(2).map(|w| d(w[0], w[1])).sum();
            if len >= min_len {
                st.count += 1;
                st.straight += len;
                st.longest = st.longest.max(len);
                let ang = (p[best].1 - p[i].1).atan2(p[best].0 - p[i].0).to_degrees().rem_euclid(45.0);
                if ang.min(45.0 - ang) <= AXIS_TOL_DEG {
                    st.axis += len;
                }
                i = best;
            } else {
                i += 1;
            }
        }
    }
    st
}

/// **Parallel river combs, measured** (owner, 2026-11-13: *"Still see some heavy combing"*;
/// `DELTA_GEOMETRY_RESEARCH.md` section 14): the length of the network that runs beside ANOTHER river,
/// close and parallel, for a sustained stretch -- the teeth of a comb, which [`straight_stats`] cannot see
/// (a comb of gently wavy teeth is not straight, and one straight trunk is no comb). A pure measurement:
/// never read by drawing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CombStats {
    /// Total polyline length measured, cells (resampled at [`COMB_STEP_CELLS`]).
    pub total: f64,
    /// Length lying in comb runs, cells.
    pub comb: f64,
    /// Comb runs found.
    pub runs: usize,
    /// The longest comb run, cells.
    pub longest: f64,
}

/// A neighbour river counts when it lies between [`COMB_NEAR_CELLS`] and this many cells to either side,
/// across the river's own direction. **Labelled judgement** (lane C of the 2026-11-13 combing report): a D8
/// comb's teeth stand one channel-head spacing apart, measured 2-5 cells on the owner's world; nearer than 2
/// is a confluence or a braid, farther than 5 is ordinary valley spacing.
pub const COMB_FAR_CELLS: f64 = 5.0;

/// See [`COMB_FAR_CELLS`]: the near edge of the band, in cells.
pub const COMB_NEAR_CELLS: f64 = 2.0;

/// The neighbour's own direction must lie within this many degrees of the river's (either sense).
/// **Labelled judgement**: parallel teeth of a comb agree within a few degrees; 20 admits gently wavy teeth
/// (rough routing's) and refuses a tributary meeting its trunk at a normal confluence angle.
pub const COMB_ANGLE_DEG: f64 = 20.0;

/// A comb run must last at least this many cells of the river's length. **Labelled judgement**: a
/// tributary approaching its trunk runs beside it for a few cells before it joins; a tooth of a comb runs
/// beside its neighbour for tens to hundreds.
pub const COMB_MIN_RUN_CELLS: f64 = 12.0;

/// Every line is resampled at this spacing (cells) before it is measured, so the result does not depend on
/// how densely a stage of the drawing stores its points. One cell: the grid the network is traced on.
pub const COMB_STEP_CELLS: f64 = 1.0;

/// The direction at a resampled point is the chord from this many samples back to this many ahead, so a
/// one-cell D8 zig-zag reads as its mean direction. Three cells: a judgement, half of [`COMB_FAR_CELLS`]
/// rounded up.
const COMB_TANGENT_SAMPLES: usize = 3;

/// Measures [`CombStats`] over `lines` (grid cells, planar: a caller on a wrapped world splits its lines
/// at the seam first). Every line is resampled at [`COMB_STEP_CELLS`]; a sample is **in a comb** when a
/// sample of a DIFFERENT line lies [`COMB_NEAR_CELLS`]..[`COMB_FAR_CELLS`] cells to either side of it,
/// perpendicular to its direction, heading within [`COMB_ANGLE_DEG`] of it; a run of consecutive such
/// samples at least [`COMB_MIN_RUN_CELLS`] long is a comb run. `O(samples)` with a cell hash. Must never
/// count a river against itself (a meander folding back is not a comb).
pub fn comb_stats<'a>(lines: impl IntoIterator<Item = &'a [(f64, f64)]>) -> CombStats {
    use std::collections::HashMap;
    let resampled: Vec<Vec<(f64, f64)>> = lines.into_iter().map(resample).filter(|v| v.len() >= 2).collect();
    let tangent = |v: &[(f64, f64)], k: usize| -> (f64, f64) {
        let a = v[k.saturating_sub(COMB_TANGENT_SAMPLES)];
        let b = v[(k + COMB_TANGENT_SAMPLES).min(v.len() - 1)];
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(1e-12);
        (dx / l, dy / l)
    };
    let key = |p: (f64, f64)| (p.0.floor() as i64, p.1.floor() as i64);
    // Every cell each line passes through, with the line's direction there: the segment after each sample is
    // walked in quarter-cell steps, so a line crossing a cell only at its corner still marks it and a probe
    // into the band can never step between two samples of a neighbour.
    let mut grid: HashMap<(i64, i64), Vec<(u32, (f64, f64))>> = HashMap::new();
    for (li, v) in resampled.iter().enumerate() {
        for k in 0..v.len() {
            let t = tangent(v, k);
            let b = v[(k + 1).min(v.len() - 1)];
            for q in 0..4 {
                let f = q as f64 * 0.25;
                let c = key((v[k].0 + (b.0 - v[k].0) * f, v[k].1 + (b.1 - v[k].1) * f));
                let e = grid.entry(c).or_default();
                if e.last().is_none_or(|&(l, _)| l != li as u32) {
                    e.push((li as u32, t));
                }
            }
        }
    }
    let cos_max = COMB_ANGLE_DEG.to_radians().cos();
    // Offsets across the river, every half cell through the band, so no cell of it is skipped.
    let offsets: Vec<f64> = {
        let mut o = Vec::new();
        let mut s = COMB_NEAR_CELLS;
        while s <= COMB_FAR_CELLS + 1e-9 {
            o.push(s);
            s += 0.5;
        }
        o
    };
    let mut st = CombStats::default();
    for (li, v) in resampled.iter().enumerate() {
        st.total += COMB_STEP_CELLS * (v.len() - 1) as f64;
        let mut run = 0usize;
        let close = |run: &mut usize, st: &mut CombStats| {
            let len = COMB_STEP_CELLS * run.saturating_sub(1) as f64;
            if len >= COMB_MIN_RUN_CELLS {
                st.runs += 1;
                st.comb += len;
                st.longest = st.longest.max(len);
            }
            *run = 0;
        };
        for k in 0..v.len() {
            let t = tangent(v, k);
            let n = (-t.1, t.0);
            let beside = offsets.iter().any(|&s| {
                [-1.0f64, 1.0].iter().any(|&sg| {
                    let q = (v[k].0 + sg * s * n.0, v[k].1 + sg * s * n.1);
                    grid.get(&key(q)).is_some_and(|hits| {
                        hits.iter().any(|&(lj, tj)| lj as usize != li && (tj.0 * t.0 + tj.1 * t.1).abs() >= cos_max)
                    })
                })
            });
            if beside {
                run += 1;
            } else {
                close(&mut run, &mut st);
            }
        }
        close(&mut run, &mut st);
    }
    st
}

/// `p` resampled at [`COMB_STEP_CELLS`] along its arc length (the first and last points kept).
fn resample(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    let Some(&first) = p.first() else { return out };
    out.push(first);
    let mut carry = 0.0f64;
    for w in p.windows(2) {
        let (a, b) = (w[0], w[1]);
        let l = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if l < 1e-12 {
            continue;
        }
        let mut s = COMB_STEP_CELLS - carry;
        while s <= l + 1e-12 {
            let f = s / l;
            out.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            s += COMB_STEP_CELLS;
        }
        carry = l - (s - COMB_STEP_CELLS);
    }
    if let (Some(&last), Some(&end)) = (out.last(), p.last()) {
        if ((last.0 - end.0).powi(2) + (last.1 - end.1).powi(2)).sqrt() > 1e-9 {
            out.push(end);
        }
    }
    out
}

/// Tests of the straight-run measure: what counts as straight, the axis share,
/// and that a meander is not counted.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: a 20-cell east run counts as one straight, grid-aligned stretch
    /// of 20 cells; a 1:3 slanted line (18.4 degrees) is straight but not
    /// aligned; a sine meander of amplitude 1 cell is not straight at
    /// tolerance 0.25; and a stretch under `min_len` is not counted. Literals.
    #[test]
    fn straight_stretches_are_found_and_classed() {
        let east: Vec<(f64, f64)> = (0..21).map(|i| (i as f64, 0.0)).collect();
        let s = straight_stats([east.as_slice()], 8.0, 0.25);
        assert_eq!((s.count, s.straight, s.axis, s.longest, s.total), (1, 20.0, 20.0, 20.0, 20.0));
        let slant: Vec<(f64, f64)> = (0..21).map(|i| (i as f64 * 3.0, i as f64)).collect();
        let s = straight_stats([slant.as_slice()], 8.0, 0.25);
        assert_eq!(s.count, 1);
        assert_eq!(s.axis, 0.0, "18.4 degrees is no grid axis");
        let wavy: Vec<(f64, f64)> = (0..81).map(|i| (i as f64 * 0.25, (i as f64 * 0.25 * std::f64::consts::TAU / 6.0).sin())).collect();
        assert_eq!(straight_stats([wavy.as_slice()], 8.0, 0.25).count, 0, "a meandering line is not straight");
        let short: Vec<(f64, f64)> = (0..6).map(|i| (i as f64, 0.0)).collect();
        assert_eq!(straight_stats([short.as_slice()], 8.0, 0.25).count, 0);
        assert_eq!(AXIS_TOL_DEG, 3.0);
    }

    /// `n` lines `len` cells long heading south, `gap` cells apart, each displaced sideways by
    /// `amp * sin(2 pi y / 10)` (all in phase).
    fn teeth(n: usize, gap: f64, len: usize, amp: f64) -> Vec<Vec<(f64, f64)>> {
        (0..n)
            .map(|k| (0..=len).map(|y| (k as f64 * gap + amp * (y as f64 * std::f64::consts::TAU / 10.0).sin(), y as f64)).collect())
            .collect()
    }

    /// Protects the comb measure's calibration against a KNOWN BAD and KNOWN GOOD: five south-running teeth
    /// 3 cells apart are a comb along their whole length (share 1.0, the owner's 2026-11-13 picture), the same
    /// teeth gently meandering in phase still are (rough routing's wavy comb is still a comb), teeth 12 cells
    /// apart are not (ordinary valley spacing), one line alone is not, a line crossing another at a right
    /// angle is not, and a tributary joining a trunk at 30 degrees is not. Also pins the band, angle, run and
    /// step as literals. Widening the band to 12 cells, narrowing the angle below the wavy teeth's spread,
    /// or comparing a line with itself makes a case fail.
    #[test]
    fn combs_are_found_and_ordinary_networks_are_not() {
        assert_eq!((COMB_NEAR_CELLS, COMB_FAR_CELLS, COMB_ANGLE_DEG, COMB_MIN_RUN_CELLS, COMB_STEP_CELLS), (2.0, 5.0, 20.0, 12.0, 1.0));
        let share = |ls: &[Vec<(f64, f64)>]| {
            let s = comb_stats(ls.iter().map(|v| v.as_slice()));
            s.comb / s.total
        };
        let comb = teeth(5, 3.0, 60, 0.0);
        let s = comb_stats(comb.iter().map(|v| v.as_slice()));
        assert_eq!((s.total, s.comb, s.runs, s.longest), (300.0, 300.0, 5, 60.0), "five straight teeth 3 cells apart");
        assert!(share(&teeth(5, 3.0, 60, 0.6)) > 0.95, "wavy teeth in phase are still a comb: {}", share(&teeth(5, 3.0, 60, 0.6)));
        assert_eq!(share(&teeth(5, 12.0, 60, 0.0)), 0.0, "teeth 12 cells apart are valleys, not a comb");
        assert_eq!(share(&teeth(1, 3.0, 60, 0.0)), 0.0, "one line is no comb");
        // A meander folding back on itself within the band is one river, never its own neighbour.
        let hairpin: Vec<(f64, f64)> = (0..=40).map(|y| (0.0, y as f64)).chain((0..=40).rev().map(|y| (3.0, y as f64))).collect();
        assert_eq!(share(&[hairpin]), 0.0, "a river is never its own neighbour");
        let cross = vec![(0..=60).map(|y| (30.0, y as f64)).collect::<Vec<_>>(), (0..=60).map(|x| (x as f64, 30.0)).collect()];
        assert_eq!(share(&cross), 0.0, "a crossing at a right angle is no comb");
        let (ang, len) = (30f64.to_radians(), 40.0);
        let trunk: Vec<(f64, f64)> = (0..=80).map(|y| (40.0, y as f64)).collect();
        let trib: Vec<(f64, f64)> = (0..=len as usize).map(|t| (40.0 - (len - t as f64) * ang.sin(), 60.0 - (len - t as f64) * ang.cos())).collect();
        assert_eq!(share(&[trunk, trib]), 0.0, "a tributary at 30 degrees is no comb");
    }

    /// Protects `resample`: the measure does not depend on how densely a line stores its points -- the
    /// five-tooth comb stored at one point per 4 cells, and at four points a cell, measures the same.
    #[test]
    fn the_comb_measure_does_not_depend_on_point_spacing() {
        let sparse: Vec<Vec<(f64, f64)>> = (0..5).map(|k| (0..=15).map(|y| (k as f64 * 3.0, y as f64 * 4.0)).collect()).collect();
        let dense: Vec<Vec<(f64, f64)>> = (0..5).map(|k| (0..=240).map(|y| (k as f64 * 3.0, y as f64 * 0.25)).collect()).collect();
        let a = comb_stats(sparse.iter().map(|v| v.as_slice()));
        let b = comb_stats(dense.iter().map(|v| v.as_slice()));
        assert_eq!(a, b);
        assert_eq!((a.total, a.comb), (300.0, 300.0));
    }
}
