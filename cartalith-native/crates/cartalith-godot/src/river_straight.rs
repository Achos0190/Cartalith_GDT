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
}
