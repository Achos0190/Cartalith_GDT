//! Measures of the drawn road network (owner request 2026-11-12, Ruling EU
//! item 1): where two drawn ways run side by side as a "bundle of wires", and
//! where a way is drawn wrong (over water, as a duplicate, with an end left
//! hanging). Measurement only: nothing in generation or drawing calls this.
//!
//! Every input is the **drawn** geometry -- the polylines the overlay strokes
//! (`cartalith-godot`'s `way_render_polyline` over each way's points and
//! breaks), in map cells -- so a figure here is about what the owner sees, not
//! about the routing grid underneath. Distances that are judged by eye are given
//! in **screen pixels** at a stated view, through `px_per_cell`, which the caller
//! measures (a windowed probe reads it off the viewport); nothing here assumes a
//! screen density.
//!
//! What it must never do: report a count of `0` for something it could not
//! measure. A pair that cannot be formed, a way list that is empty, is an empty
//! result, and the caller prints it as absent.

use std::collections::HashMap;

/// The widest separation, in screen pixels, at which two drawn ways read as one
/// bundle of parallel strokes rather than two roads. A labelled judgement: about
/// 3.5 times the widest road under-stroke (`map_overlay.gd`'s `WAY_STYLE`,
/// highway `under_w` 2.3 px), the distance at which two such strokes still read
/// as one band at a glance. To be confirmed by eye on the owner's frames.
pub const D_BUNDLE_PX: f64 = 8.0;

/// The shortest run, in screen pixels, counted as a stretch. A labelled
/// judgement: shorter side-by-side runs are the first few pixels of any fork,
/// where two roads leaving one junction at a modest angle are necessarily close.
pub const L_MIN_PX: f64 = 30.0;

/// The widest angle between two ways' directions for them to count as parallel.
/// A labelled judgement: two roads closer than [`D_BUNDLE_PX`] that cross at more
/// than this are a crossing or a fork, not a bundle.
pub const MAX_ANGLE_DEG: f64 = 20.0;

/// The sampling step along each drawn way, in screen pixels: half a pixel, so no
/// feature of a stroke at least one pixel wide is stepped over.
pub const SAMPLE_PX: f64 = 0.5;

/// How far past a stretch's end, as a multiple of the stretch's own length, two
/// ways may still meet for the stretch to count as **convergent** (heading to the
/// same place). A labelled judgement: a pair that meets within twice the length
/// it ran side by side was going to the same point.
pub const CONVERGE_REACH: f64 = 2.0;

/// One drawn way as the audit reads it.
#[derive(Clone, Copy, Debug)]
pub struct AuditWay<'a> {
    /// The drawn polyline, map cells.
    pub pts: &'a [(f64, f64)],
    /// Indices into `pts` where the pen lifts (a world-wrap seam).
    pub brks: &'a [usize],
    /// The way's under-stroke width in screen pixels (`WAY_STYLE`'s `under_w`).
    pub under_px: f64,
    /// The two settlements the way's edge joins (indices into the caller's
    /// settlement list).
    pub ends: (usize, usize),
}

/// The view the audit measures at.
#[derive(Clone, Copy, Debug)]
pub struct AuditView {
    /// Screen pixels per map cell at this view (measured by the caller).
    pub px_per_cell: f64,
}

/// One side-by-side run of two drawn ways.
#[derive(Clone, Debug, PartialEq)]
pub struct Stretch {
    /// The way the run was sampled along.
    pub p: usize,
    /// The way it ran beside.
    pub q: usize,
    /// The run's length along `p`, screen pixels.
    pub len_px: f64,
    /// Mean separation over the run, screen pixels.
    pub mean_sep_px: f64,
    /// `true` when the two strokes touch over the run (separation at most the
    /// mean of their under-stroke widths) -- drawn over each other.
    pub overlap: bool,
    /// `true` when, within [`CONVERGE_REACH`] times the run's length past either
    /// end, the two ways meet or end at the same settlement.
    pub convergent: bool,
}

/// A sample on a drawn way: position, unit direction, arc length (cells) and
/// which pen-down run of the way it is on.
#[derive(Clone, Copy, Debug)]
struct Sample {
    x: f64,
    y: f64,
    tx: f64,
    ty: f64,
    s: f64,
    run: usize,
}

/// The way's pen-down runs, each a slice of `pts` between breaks.
fn runs<'a>(pts: &'a [(f64, f64)], brks: &[usize]) -> Vec<&'a [(f64, f64)]> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for &cut in brks.iter().chain(std::iter::once(&pts.len())) {
        let cut = cut.min(pts.len());
        if cut > start + 1 {
            out.push(&pts[start..cut]);
        }
        start = start.max(cut);
    }
    out
}

/// Samples every `step` cells along a way's drawn runs, each with its segment's
/// direction. Zero-length segments are skipped (they have no direction).
fn sample_way(w: &AuditWay, step: f64) -> Vec<Sample> {
    let mut out = Vec::new();
    let mut s_total = 0.0;
    for (ri, r) in runs(w.pts, w.brks).into_iter().enumerate() {
        let mut carry = 0.0;
        for seg in r.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len = dx.hypot(dy);
            if len <= 0.0 {
                continue;
            }
            let (tx, ty) = (dx / len, dy / len);
            let mut t = carry;
            while t < len {
                out.push(Sample { x: a.0 + tx * t, y: a.1 + ty * t, tx, ty, s: s_total + t, run: ri });
                t += step;
            }
            carry = t - len;
            s_total += len;
        }
    }
    out
}

/// A uniform bucket grid over every drawn segment, so a point finds the segments
/// near it without scanning the whole network.
struct SegIndex {
    cell: f64,
    /// `(way, a, b, first, last)`: `first`/`last` mark a run's end segments.
    buckets: HashMap<(i64, i64), Vec<(usize, (f64, f64), (f64, f64), bool, bool)>>,
}

/// What [`SegIndex::near`] found for one other way: the way, the distance, its
/// segment's unit direction, and whether the nearest point is that way's own
/// pen-down end (the query point lies past the way, not beside it).
type Near = (usize, f64, (f64, f64), bool);

impl SegIndex {
    /// Buckets of side `cell` cells holding `(way, a, b)` for every segment.
    fn build(ways: &[AuditWay], cell: f64) -> Self {
        let mut buckets: HashMap<(i64, i64), Vec<_>> = HashMap::new();
        for (wi, w) in ways.iter().enumerate() {
            for r in runs(w.pts, w.brks) {
                let nseg = r.len() - 1;
                for (si, seg) in r.windows(2).enumerate() {
                    let (a, b) = (seg[0], seg[1]);
                    let (x0, x1) = ((a.0.min(b.0) / cell).floor() as i64, (a.0.max(b.0) / cell).floor() as i64);
                    let (y0, y1) = ((a.1.min(b.1) / cell).floor() as i64, (a.1.max(b.1) / cell).floor() as i64);
                    for bx in x0..=x1 {
                        for by in y0..=y1 {
                            buckets.entry((bx, by)).or_default().push((wi, a, b, si == 0, si + 1 == nseg));
                        }
                    }
                }
            }
        }
        SegIndex { cell, buckets }
    }

    /// For each way other than `skip` with a segment within `r` cells of
    /// `(x, y)` (`r` at most the bucket side): see [`Near`].
    fn near(&self, x: f64, y: f64, r: f64, skip: usize) -> Vec<Near> {
        let (bx, by) = ((x / self.cell).floor() as i64, (y / self.cell).floor() as i64);
        let mut best: HashMap<usize, (f64, (f64, f64), bool)> = HashMap::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                let Some(v) = self.buckets.get(&(bx + dx, by + dy)) else { continue };
                for &(wi, a, b, first, last) in v {
                    if wi == skip {
                        continue;
                    }
                    let (sx, sy) = (b.0 - a.0, b.1 - a.1);
                    let l2 = sx * sx + sy * sy;
                    if l2 <= 0.0 {
                        continue;
                    }
                    let raw = ((x - a.0) * sx + (y - a.1) * sy) / l2;
                    let t = raw.clamp(0.0, 1.0);
                    let d = (x - (a.0 + sx * t)).hypot(y - (a.1 + sy * t));
                    if d > r {
                        continue;
                    }
                    let past_end = (first && raw < 0.0) || (last && raw > 1.0);
                    let l = l2.sqrt();
                    let e = best.entry(wi).or_insert((f64::INFINITY, (0.0, 0.0), false));
                    if d < e.0 {
                        *e = (d, (sx / l, sy / l), past_end);
                    }
                }
            }
        }
        let mut out: Vec<Near> = best.into_iter().map(|(k, (d, t, e))| (k, d, t, e)).collect();
        out.sort_by_key(|e| e.0);
        out
    }
}

/// Every side-by-side stretch between two distinct drawn ways at `view`, from
/// the lower-indexed way's samples (`p < q`), with its overlap and convergence
/// flags. `places` gives each settlement's position in map cells, for the
/// shared-destination test. Uses the module's labelled thresholds.
pub fn bundle_stretches(ways: &[AuditWay], places: &[(f64, f64)], view: AuditView) -> Vec<Stretch> {
    bundle_stretches_with(ways, places, view, D_BUNDLE_PX, L_MIN_PX, MAX_ANGLE_DEG)
}

/// [`bundle_stretches`] with its three thresholds given, for the census's
/// sensitivity rows and the unit tests.
pub fn bundle_stretches_with(
    ways: &[AuditWay],
    places: &[(f64, f64)],
    view: AuditView,
    d_bundle_px: f64,
    l_min_px: f64,
    max_angle_deg: f64,
) -> Vec<Stretch> {
    let ppc = view.px_per_cell;
    if ways.len() < 2 || ppc <= 0.0 {
        return Vec::new();
    }
    let d_bundle = d_bundle_px / ppc;
    let l_min = l_min_px / ppc;
    let cos_max = max_angle_deg.to_radians().cos();
    let index = SegIndex::build(ways, d_bundle.max(1e-6));
    let mut out = Vec::new();
    for (p, wp) in ways.iter().enumerate() {
        let samples = sample_way(wp, SAMPLE_PX / ppc);
        // Per sample: the other ways within the bundle distance.
        let near: Vec<Vec<Near>> =
            samples.iter().map(|s| index.near(s.x, s.y, d_bundle, p)).collect();
        // Run tracking per partner q: (kind overlap?, start sample, sum sep, count).
        let mut open: HashMap<usize, (bool, usize, f64, usize)> = HashMap::new();
        let close = |q: usize, st: (bool, usize, f64, usize), end: usize, out: &mut Vec<Stretch>| {
            let (overlap, i0, sum, cnt) = st;
            let len = samples[end].s - samples[i0].s;
            if q <= p || len < l_min || cnt == 0 {
                return;
            }
            let reach = len * CONVERGE_REACH;
            let wq = &ways[q];
            let touch = (wp.under_px + wq.under_px) * 0.5 / ppc;
            let meets = |k: usize| near[k].iter().any(|&(qq, d, _, _)| qq == q && d <= touch);
            let (s0, s1) = (samples[i0].s, samples[end].s);
            let before = (0..i0).rev().take_while(|&k| s0 - samples[k].s <= reach).any(meets);
            let after = (end + 1..samples.len()).take_while(|&k| samples[k].s - s1 <= reach).any(meets);
            let mut convergent = before || after;
            if !convergent {
                let shared = [wp.ends.0, wp.ends.1].into_iter().filter(|e| *e == wq.ends.0 || *e == wq.ends.1);
                for e in shared {
                    let Some(&(ex, ey)) = places.get(e) else { continue };
                    let near_end = [i0, end].iter().any(|&k| (samples[k].x - ex).hypot(samples[k].y - ey) <= reach);
                    if near_end {
                        convergent = true;
                    }
                }
            }
            out.push(Stretch { p, q, len_px: len * ppc, mean_sep_px: sum / cnt as f64 * ppc, overlap, convergent });
        };
        for (k, s) in samples.iter().enumerate() {
            let mut seen: Vec<usize> = Vec::new();
            for &(q, d, (tx, ty), past_end) in &near[k] {
                let wq = &ways[q];
                let touch = (wp.under_px + wq.under_px) * 0.5 / ppc;
                // Beside, not past: a sample whose nearest point on `q` is `q`'s
                // own end lies beyond `q`, so it counts only when touching.
                let kind = if d <= touch {
                    Some(true)
                } else if !past_end && (s.tx * tx + s.ty * ty).abs() >= cos_max {
                    Some(false)
                } else {
                    None
                };
                let Some(kind) = kind else { continue };
                seen.push(q);
                let continues = open.get(&q).is_some_and(|st| st.0 == kind && samples[st.1].run == s.run);
                if continues {
                    let st = open.get_mut(&q).expect("checked above");
                    st.2 += d;
                    st.3 += 1;
                } else {
                    if let Some(st) = open.remove(&q) {
                        close(q, st, k.saturating_sub(1), &mut out);
                    }
                    open.insert(q, (kind, k, d, 1));
                }
            }
            let ended: Vec<usize> = open.keys().copied().filter(|q| !seen.contains(q)).collect();
            for q in ended {
                let st = open.remove(&q).unwrap();
                close(q, st, k.saturating_sub(1), &mut out);
            }
        }
        let last = samples.len().saturating_sub(1);
        let mut rest: Vec<_> = open.into_iter().collect();
        rest.sort_by_key(|e| e.0);
        for (q, st) in rest {
            close(q, st, last, &mut out);
        }
    }
    out
}

/// Pairs of drawn ways that are one stroke drawn twice: every sample of each lies
/// within the touch distance (the mean of their under-stroke widths) of the
/// other, i.e. their Hausdorff distance is at most that. Identical point lists
/// are the exact case. `p < q`.
pub fn near_duplicates(ways: &[AuditWay], view: AuditView) -> Vec<(usize, usize)> {
    let ppc = view.px_per_cell;
    if ways.len() < 2 || ppc <= 0.0 {
        return Vec::new();
    }
    let widest = ways.iter().map(|w| w.under_px).fold(0.0, f64::max) / ppc;
    let index = SegIndex::build(ways, widest.max(1e-6));
    // covered[(p, q)]: whether every sample of p is within touch of q.
    let mut covered: HashMap<(usize, usize), bool> = HashMap::new();
    for (p, wp) in ways.iter().enumerate() {
        let samples = sample_way(wp, SAMPLE_PX / ppc);
        if samples.is_empty() {
            continue;
        }
        let mut count: HashMap<usize, usize> = HashMap::new();
        for s in &samples {
            for (q, d, _, _) in index.near(s.x, s.y, widest, p) {
                if d <= (wp.under_px + ways[q].under_px) * 0.5 / ppc {
                    *count.entry(q).or_default() += 1;
                }
            }
        }
        for (q, c) in count {
            covered.insert((p, q), c == samples.len());
        }
    }
    let mut out: Vec<(usize, usize)> = covered
        .iter()
        .filter(|&(&(p, q), &all)| p < q && all && covered.get(&(q, p)).copied().unwrap_or(false))
        .map(|(&k, _)| k)
        .collect();
    out.sort();
    out
}

/// The distance, in map cells, from `pt` to the nearest drawn way other than
/// `skip`, searched out to `r` cells; `None` when none is that close.
pub fn nearest_other_way(ways: &[AuditWay], pt: (f64, f64), skip: usize, r: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for (wi, w) in ways.iter().enumerate() {
        if wi == skip {
            continue;
        }
        for run in runs(w.pts, w.brks) {
            for seg in run.windows(2) {
                let (a, b) = (seg[0], seg[1]);
                let (sx, sy) = (b.0 - a.0, b.1 - a.1);
                let l2 = sx * sx + sy * sy;
                let t = if l2 > 0.0 { (((pt.0 - a.0) * sx + (pt.1 - a.1) * sy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                let d = (pt.0 - (a.0 + sx * t)).hypot(pt.1 - (a.1 + sy * t));
                if d <= r && best.is_none_or(|b| d < b) {
                    best = Some(d);
                }
            }
        }
    }
    best
}

/// Drawn samples that land on water: every sample of every way at `step` cells
/// whose nearest cell's class is not `0` (land) in `classification`, skipping
/// samples within one cell of the way's own two ends (the pin cell, which a
/// harbour town may share with the shore). Returns the count of such samples.
pub fn water_samples(ways: &[AuditWay], classification: &[u8], gw: usize, gh: usize, step: f64) -> usize {
    let mut n = 0;
    for w in ways {
        let (Some(&first), Some(&last)) = (w.pts.first(), w.pts.last()) else { continue };
        for s in sample_way(w, step) {
            if (s.x - first.0).hypot(s.y - first.1) <= 1.0 || (s.x - last.0).hypot(s.y - last.1) <= 1.0 {
                continue;
            }
            let (cx, cy) = (s.x.round().clamp(0.0, gw as f64 - 1.0) as usize, s.y.round().clamp(0.0, gh as f64 - 1.0) as usize);
            if classification[cy * gw + cx] != 0 {
                n += 1;
            }
        }
    }
    n
}

/// River crossings along the drawn ways: each maximal run of samples whose cell
/// has a channel (`order >= 1`) is one crossing, rated by the highest order it
/// touches. Returns `(crossings, crossings of order >= bridge_order)`. `step` in
/// cells.
pub fn river_crossings(
    ways: &[AuditWay],
    order: &[i16],
    gw: usize,
    gh: usize,
    step: f64,
    bridge_order: i16,
) -> (usize, usize) {
    let (mut all, mut bridge) = (0, 0);
    for w in ways {
        let mut cur: Option<i16> = None;
        let mut last_run = usize::MAX;
        for s in sample_way(w, step) {
            let (cx, cy) = (s.x.round().clamp(0.0, gw as f64 - 1.0) as usize, s.y.round().clamp(0.0, gh as f64 - 1.0) as usize);
            let o = order[cy * gw + cx];
            if s.run != last_run {
                if let Some(m) = cur.take() {
                    all += 1;
                    bridge += usize::from(m >= bridge_order);
                }
                last_run = s.run;
            }
            if o >= 1 {
                cur = Some(cur.map_or(o, |m| m.max(o)));
            } else if let Some(m) = cur.take() {
                all += 1;
                bridge += usize::from(m >= bridge_order);
            }
        }
        if let Some(m) = cur {
            all += 1;
            bridge += usize::from(m >= bridge_order);
        }
    }
    (all, bridge)
}

/// Total drawn length of `ways`, map cells.
pub fn drawn_length(ways: &[AuditWay]) -> f64 {
    ways.iter()
        .flat_map(|w| runs(w.pts, w.brks))
        .flat_map(|r| r.windows(2).map(|s| (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1)))
        .sum()
}

/// The share of drawn length whose grade exceeds `grade` (rise over run), with
/// heights in metres from `height_m(x, y)` and `cell_m` metres per map cell.
/// Measured over each drawn segment between consecutive samples `step` cells
/// apart. `None` when there is no drawn length.
pub fn steep_share(ways: &[AuditWay], height_m: &dyn Fn(f64, f64) -> f64, cell_m: f64, step: f64, grade: f64) -> Option<f64> {
    let (mut steep, mut total) = (0.0, 0.0);
    for w in ways {
        let s = sample_way(w, step);
        for pair in s.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.run != b.run {
                continue;
            }
            let run = (b.s - a.s) * cell_m;
            if run <= 0.0 {
                continue;
            }
            total += run;
            if (height_m(b.x, b.y) - height_m(a.x, a.y)).abs() / run > grade {
                steep += run;
            }
        }
    }
    (total > 0.0).then(|| steep / total)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight polyline from `a` to `b` with points every 0.25 cells (the
    /// overlay's render step).
    fn line(a: (f64, f64), b: (f64, f64)) -> Vec<(f64, f64)> {
        let n = (((b.0 - a.0).hypot(b.1 - a.1)) / 0.25).ceil() as usize;
        (0..=n).map(|i| (a.0 + (b.0 - a.0) * i as f64 / n as f64, a.1 + (b.1 - a.1) * i as f64 / n as f64)).collect()
    }

    fn way<'a>(pts: &'a [(f64, f64)], ends: (usize, usize)) -> AuditWay<'a> {
        AuditWay { pts, brks: &[], under_px: 2.0, ends }
    }

    /// Protects: the bundle measure finds a side-by-side run of exactly the right
    /// length, flags it convergent when the two end at one settlement, and finds
    /// nothing when the pair is far apart (the negative control).
    /// Fixture at 1 px per cell: two horizontal lines 100 cells long, 4 cells
    /// apart (4 px: between touch 2 px and the 8 px bundle limit), both ending at
    /// settlement 2 at (100, 52).
    #[test]
    fn two_parallel_lines_are_one_stretch_of_their_length() {
        let a = line((0.0, 50.0), (100.0, 50.0));
        let b = line((0.0, 54.0), (100.0, 54.0));
        let places = [(0.0, 50.0), (0.0, 54.0), (100.0, 52.0)];
        let ways = [way(&a, (0, 2)), way(&b, (1, 2))];
        let st = bundle_stretches(&ways, &places, AuditView { px_per_cell: 1.0 });
        assert_eq!(st.len(), 1, "{st:?}");
        let s = &st[0];
        assert_eq!((s.p, s.q, s.overlap, s.convergent), (0, 1, false, true));
        // The stretch spans the whole line to within one sample (0.5 px).
        assert!((s.len_px - 100.0).abs() <= 0.5, "{}", s.len_px);
        assert!((s.mean_sep_px - 4.0).abs() < 1e-9, "{}", s.mean_sep_px);
        // Different destinations, no meeting: still a stretch, not convergent.
        let ways2 = [way(&a, (0, 3)), way(&b, (1, 4))];
        let st2 = bundle_stretches(&ways2, &[(0.0, 50.0), (0.0, 54.0), (100.0, 52.0), (200.0, 0.0), (200.0, 99.0)], AuditView { px_per_cell: 1.0 });
        assert_eq!(st2.len(), 1);
        assert!(!st2[0].convergent);
        // Negative control: 9 px apart is past D_BUNDLE_PX (8).
        let c = line((0.0, 59.0), (100.0, 59.0));
        let far = [way(&a, (0, 2)), way(&c, (1, 2))];
        assert!(bundle_stretches(&far, &places, AuditView { px_per_cell: 1.0 }).is_empty());
        // The same pair at 0.8 px per cell is 7.2 px apart: inside the limit again.
        let st3 = bundle_stretches(&far, &places, AuditView { px_per_cell: 0.8 });
        assert_eq!(st3.len(), 1, "9 cells at 0.8 px/cell is 7.2 px");
    }

    /// Protects: a fork at a wide angle is not a bundle (the angle test, 20
    /// degrees), and a run shorter than 30 px is not a stretch (the length test).
    #[test]
    fn a_wide_fork_and_a_short_run_are_not_stretches() {
        let a = line((0.0, 50.0), (100.0, 50.0));
        // Leaves (50, 50) at 45 degrees.
        let b = line((50.0, 50.0), (100.0, 100.0));
        let ways = [way(&a, (0, 1)), way(&b, (2, 3))];
        let st = bundle_stretches(&ways, &[(0.0, 0.0); 4], AuditView { px_per_cell: 1.0 });
        // The touching junction is a few px long and the rest is at 45 degrees.
        assert!(st.is_empty(), "{st:?}");
        // A parallel run of 20 cells (20 px) is below L_MIN_PX.
        let c = line((10.0, 54.0), (30.0, 54.0));
        let ways = [way(&a, (0, 1)), way(&c, (2, 3))];
        assert!(bundle_stretches(&ways, &[(0.0, 0.0); 4], AuditView { px_per_cell: 1.0 }).is_empty());
    }

    /// Protects: the angle test reads the local direction. A sawtooth (segments at
    /// +-30 degrees, 4-5.2 px off a straight line for 100 px) stays inside the
    /// bundle distance all the way but is never parallel at 20 degrees, so it is
    /// no stretch; at 50 degrees it would be one.
    #[test]
    fn a_zigzag_beside_a_line_is_not_parallel() {
        let a = line((0.0, 50.0), (100.0, 50.0));
        let rise = 2.0 * 30f64.to_radians().tan();
        let saw: Vec<(f64, f64)> = (0..=50).map(|i| (2.0 * i as f64, if i % 2 == 0 { 54.0 } else { 54.0 + rise })).collect();
        let ways = [way(&a, (0, 1)), way(&saw, (2, 3))];
        let v = AuditView { px_per_cell: 1.0 };
        assert!(bundle_stretches(&ways, &[(0.0, 0.0); 4], v).is_empty());
        assert_eq!(bundle_stretches_with(&ways, &[(0.0, 0.0); 4], v, D_BUNDLE_PX, L_MIN_PX, 50.0).len(), 1);
    }

    /// Protects: overlap (strokes touching) is told apart from a bundle, and a
    /// way drawn twice is a near-duplicate while a parallel neighbour is not.
    #[test]
    fn a_way_drawn_twice_is_a_near_duplicate() {
        let a = line((0.0, 50.0), (100.0, 50.0));
        let b = line((0.0, 50.5), (100.0, 50.5));
        let c = line((0.0, 54.0), (100.0, 54.0));
        let ways = [way(&a, (0, 1)), way(&b, (0, 1)), way(&c, (0, 1))];
        let v = AuditView { px_per_cell: 1.0 };
        assert_eq!(near_duplicates(&ways, v), vec![(0, 1)]);
        let st = bundle_stretches(&ways, &[(0.0, 50.0), (100.0, 50.0)], v);
        assert!(st.iter().any(|s| (s.p, s.q) == (0, 1) && s.overlap));
        assert!(st.iter().any(|s| (s.p, s.q) == (0, 2) && !s.overlap));
    }

    /// Protects: water and river crossings are counted on the drawn line, the pin
    /// cell is exempt, and a run over a channel is one crossing rated by its
    /// highest order.
    #[test]
    fn water_and_crossings_are_counted_on_the_drawn_line() {
        let (gw, gh) = (40, 10);
        let mut class = vec![0u8; gw * gh];
        let mut order = vec![0i16; gw * gh];
        // A lake cell at x 20 and a river at x 30 (orders 2 then 4 across it).
        class[5 * gw + 20] = 2;
        order[5 * gw + 29] = 2;
        order[5 * gw + 30] = 4;
        let a = line((2.0, 5.0), (37.0, 5.0));
        let ways = [way(&a, (0, 1))];
        assert!(water_samples(&ways, &class, gw, gh, 0.25) > 0);
        let (all, bridge) = river_crossings(&ways, &order, gw, gh, 0.25, 3);
        assert_eq!((all, bridge), (1, 1));
        // A way ending on the lake cell's neighbour: the pin cell is exempt.
        let b = line((2.0, 5.0), (19.0, 5.0));
        let ways = [way(&b, (0, 1))];
        assert_eq!(water_samples(&ways, &class, gw, gh, 0.25), 0);
    }

    /// Protects: the grade share. A 10-cell way with `cell_m` 100 and a 20 m rise
    /// per cell is a 20 % grade throughout: share 1.0 above 12 %, 0.0 above 25 %,
    /// and absent (not 0) with no drawn length.
    #[test]
    fn steep_share_reads_the_grade() {
        let a = line((0.0, 0.0), (10.0, 0.0));
        let ways = [way(&a, (0, 1))];
        let h = |x: f64, _y: f64| x * 20.0;
        assert!((steep_share(&ways, &h, 100.0, 0.5, 0.12).unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(steep_share(&ways, &h, 100.0, 0.5, 0.25), Some(0.0));
        assert_eq!(steep_share(&[], &h, 100.0, 0.5, 0.12), None);
    }
}
