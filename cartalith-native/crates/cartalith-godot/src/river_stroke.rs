//! RV-2 -- the drawn river stroke's geometry (`OUTSTANDING_WORK.md`, "RV-2,
//! RV-4, RV-5: smooth rivers and lakes at every zoom"). Owner, 2026-09-29:
//! rivers must always be smooth, never *"the pixilated depressions that are
//! either yes or no connected all the way to nonsensical partial lines."*
//!
//! `_riverzoom_probe.gd` measured three causes in the one drawn river, the
//! vector stroke `map_overlay.gd::_draw_rivers` draws from `get_rivers(1)`:
//!
//! 1. **A per-point lake mask cut strokes into pieces.** Every render point
//!    was tested against the lake raster on its own, so a spline that grazed a
//!    lake cell's corner, or a run through a bead of 1-3 cell lakes, fell apart
//!    into 3 or more pieces on 65-101 rivers per world (one into 29), and a
//!    piece of one point was dropped outright. [`stroke_pieces`] replaces it:
//!    a stroke is cut only where the **traced** run itself crosses water, once
//!    per crossing, and ends exactly on the shoreline.
//! 2. **Width was one number per run, read at the mouth.**
//!    `cartalith_hydrology::river_half_width_profile` now gives one per traced
//!    point; [`render_params`] carries it onto the curve and [`stroke_mesh`]
//!    draws it, tapering, as one triangle strip.
//! 3. **Headwaters were sub-pixel at fit zoom.** [`stroke_mesh`] never draws a
//!    stroke narrower than [`MIN_STROKE_PX`].
//!
//! Everything here is presentation: nothing reads or writes the height field,
//! the channel rasters or the traced runs, so no generation golden can move.

use godot::prelude::*;

use crate::WorldGen;

/// The narrowest a river stroke is ever drawn, in **screen** pixels. The
/// owner's bar for RV-2 ("headwaters never draw thinner than 1 px") restated,
/// not a tuning choice: below one pixel an antialiased stroke is drawn as a
/// fraction of a pixel's coverage, which a light headwater colour over land
/// turns into the dotted, on-off line the probe counted.
pub const MIN_STROKE_PX: f32 = 1.0;

/// The antialiasing fringe outside each edge of a stroke, in screen pixels:
/// the stroke is solid to its geometric edge and its alpha falls to zero over
/// this distance beyond it. One pixel is the width of the thing being
/// antialiased.
///
/// **Outside the edge, not centred on it -- chosen by looking, and the first
/// build was the other way.** A fringe centred on the edge keeps the drawn
/// coverage exactly equal to the stroke's width, which sounds right and read
/// wrong: a 1 px stroke became a 2 px tent peaking at full alpha only on its
/// centreline, so wherever the line fell between two pixel centres both got
/// half alpha, and at the opening view a dark-blue trunk read as a pale,
/// washed band (`_riverzoom_probe`, seed 483920, x1, before/after PNGs). A
/// solid core keeps every stroke at its colour; this is also how Godot's own
/// antialiased `draw_polyline`, which drew these rivers before, feathers.
pub const EDGE_FRINGE_PX: f32 = 1.0;

/// For each render point, its position along the traced run as a fractional
/// index into `pts` (`2.25` = a quarter of the way from `pts[2]` to `pts[3]`).
///
/// The render curve is a spline through an RDP-simplified subset of `pts`
/// (`river_render_polyline`), so there is no index correspondence to read
/// off. Each render point is projected onto the traced segments either side of
/// its nearest traced point, walking forward only -- both lists run head to
/// mouth -- and the result is clamped to never go backwards, so anything
/// interpolated along it (the width) keeps the traced run's ordering.
pub fn render_params(rp: &[(f64, f64)], pts: &[(f64, f64)]) -> Vec<f64> {
    let n = pts.len();
    if n == 0 {
        return vec![0.0; rp.len()];
    }
    let d2 = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2);
    // Projection of `p` onto segment `k..k+1`, as `(squared distance, u)`.
    let proj = |p: (f64, f64), k: usize| -> (f64, f64) {
        let (a, b) = (pts[k], pts[k + 1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        (d2(p, (a.0 + dx * t, a.1 + dy * t)), k as f64 + t)
    };
    let mut near = 0usize;
    let mut last = 0.0f64;
    rp.iter()
        .map(|&p| {
            while near + 1 < n && d2(pts[near + 1], p) <= d2(pts[near], p) {
                near += 1;
            }
            let mut best = (d2(p, pts[near]), near as f64);
            if near + 1 < n {
                let c = proj(p, near);
                if c.0 < best.0 {
                    best = c;
                }
            }
            if near > 0 {
                let c = proj(p, near - 1);
                if c.0 < best.0 {
                    best = c;
                }
            }
            last = last.max(best.1);
            last
        })
        .collect()
}

/// Linear interpolation of a per-traced-point value at fractional index `u`.
pub fn sample_at(v: &[f64], u: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let u = u.clamp(0.0, (v.len() - 1) as f64);
    let k = u.floor() as usize;
    if k + 1 >= v.len() {
        return v[v.len() - 1];
    }
    let t = u - k as f64;
    v[k] + (v[k + 1] - v[k]) * t
}

/// The point on `a -> b` where `wet` changes from false (at `a`) to true (at
/// `b`), by bisection -- exact to far below a pixel at any zoom this map has,
/// since `wet` is a per-cell test and so a step function along the segment.
/// Returns the fraction along `a -> b`. `wet(a)` must be false and `wet(b)`
/// true; if not, the caller has no crossing here and gets `1.0`.
pub fn crossing(a: (f64, f64), b: (f64, f64), wet: &impl Fn((f64, f64)) -> bool) -> f64 {
    if wet(a) || !wet(b) {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    // 40 halvings of a segment a quarter-cell long is 2e-13 cells.
    for _ in 0..40 {
        let m = 0.5 * (lo + hi);
        if wet((a.0 + (b.0 - a.0) * m, a.1 + (b.1 - a.1) * m)) {
            hi = m;
        } else {
            lo = m;
        }
    }
    lo
}

/// Where a run that ends on dry land meets the water beside it.
///
/// A traced run stops at its last channel cell, whose centre is up to half a
/// cell inland of the shore, so its stroke stopped short of the sea or lake it
/// drains into -- a gap that grows with zoom. This is the point on the way
/// from that cell's centre to the water cell it drains into where the shore
/// is:
///
/// - the water cell is `recv` of the mouth when that is a water neighbour,
///   otherwise the water neighbour most nearly straight on from the run's last
///   step (none behind it: a river does not turn back to reach the sea);
/// - on the **sea**, where the field crosses `sea_level` between the two cell
///   centres, linearly -- the coast the interpolated deep-zoom terrain draws;
/// - on a **lake**, the lake cell's edge, since a lake is drawn as whole cells
///   (RV-4 will give it a contour, and this will want to follow it).
///
/// `water` is the drawn classification (0 land, 1 ocean, 2 lake). `None` when
/// the mouth is itself water (the caller's shoreline cut handles it), when no
/// water touches it (an inland end), or when the arrays are short.
#[allow(clippy::too_many_arguments)]
pub fn coast_end(
    pts: &[(f64, f64)],
    water: &[u8],
    fld: &[f32],
    sea_level: f64,
    recv: Option<&[i32]>,
    w: usize,
    h: usize,
) -> Option<(f64, f64)> {
    let n = w * h;
    if n == 0 || water.len() < n || fld.len() < n {
        return None;
    }
    let last = *pts.last()?;
    let (mx, my) = ((last.0.floor().max(0.0) as usize).min(w - 1), (last.1.floor().max(0.0) as usize).min(h - 1));
    let m = my * w + mx;
    if water[m] != 0 {
        return None;
    }
    let centre = |c: usize| ((c % w) as f64 + 0.5, (c / w) as f64 + 0.5);
    let wet_nbs: Vec<usize> = (-1i64..=1)
        .flat_map(|dy| (-1i64..=1).map(move |dx| (dx, dy)))
        .filter(|&d| d != (0, 0))
        .map(|(dx, dy)| (mx as i64 + dx, my as i64 + dy))
        .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h)
        .map(|(x, y)| y as usize * w + x as usize)
        .filter(|&c| water[c] != 0)
        .collect();
    let by_recv = recv.and_then(|r| r.get(m)).and_then(|&r| usize::try_from(r).ok()).filter(|r| wet_nbs.contains(r));
    let nb = by_recv.or_else(|| {
        let prev = *pts.get(pts.len().checked_sub(2)?)?;
        let (dx, dy) = (last.0 - prev.0, last.1 - prev.1);
        let cos = |c: usize| {
            let q = centre(c);
            let (ex, ey) = (q.0 - last.0, q.1 - last.1);
            (dx * ex + dy * ey) / ((dx * dx + dy * dy).sqrt() * (ex * ex + ey * ey).sqrt())
        };
        wet_nbs.iter().copied().filter(|&c| cos(c) > 0.0).max_by(|&a, &b| cos(a).total_cmp(&cos(b)))
    })?;
    let (a, b) = (centre(m), centre(nb));
    let (fm, fb) = (fld[m] as f64, fld[nb] as f64);
    let t = if water[nb] == 1 && fb < sea_level && sea_level < fm {
        (fm - sea_level) / (fm - fb)
    } else {
        let wet = |p: (f64, f64)| {
            let c = (p.1.floor().max(0.0) as usize).min(h - 1) * w + (p.0.floor().max(0.0) as usize).min(w - 1);
            water[c] != 0
        };
        crossing(a, b, &wet)
    };
    Some((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t))
}

/// Settles each run's width profile against the run its stroke ends on:
/// `joins[i] = Some((j, k))` means run `i`'s stroke ends on point `k` of run
/// `j`. Two different things end that way, and they are treated differently:
///
/// - **`k > 0`, a confluence.** The tributary is capped at its trunk's width
///   there, so its end sits inside the trunk's stroke -- no tab of the
///   narrower river's colour spilling past the wider one's banks, which is
///   what two equal-width bands meeting looked like before
///   (`River::half_width_cells`' own history).
/// - **`k == 0`, a continuation.** The run ends on the *first* point of `j` --
///   a `river_draw_plan` bridge from a land pit onto the head of the next run.
///   That is one river carrying on, not a tributary: capping would shrink a
///   trunk to a headwater's width. Instead `j` is raised to at least the
///   incoming run's end width, so the width keeps growing downstream across
///   the join.
///
/// Both keep a monotone profile monotone (a cap or a floor by a constant).
/// Raises settle first, to a fixed point along chains of continuations; then
/// caps, to a fixed point along every join -- a continuation included, so that
/// when the run carried on *into* is itself capped by the trunk it joins, the
/// incoming run is capped to match rather than stepping down at the bridge
/// (measured before this: 22 / 19 / 9 continuations per world narrowed there,
/// `_riverzoom_probe`). Doing both in one loop could oscillate on a run that is
/// both continued into and capped.
pub fn settle_join_widths(profiles: &mut [Option<Vec<f64>>], joins: &[Option<(usize, usize)>]) {
    let n = profiles.len().min(joins.len());
    let at = |profiles: &[Option<Vec<f64>>], j: usize, k: usize| -> Option<f64> {
        profiles.get(j)?.as_ref().and_then(|p| p.get(k.min(p.len().saturating_sub(1))).copied())
    };
    for raise in [true, false] {
        for _ in 0..n {
            let mut changed = false;
            for i in 0..n {
                let Some((j, k)) = joins[i] else { continue };
                // Raises run along continuations only; caps along every
                // edge, so a run bridged onto a head that its own trunk then
                // capped is capped with it, and the two still meet at one
                // width.
                if (raise && k != 0) || j == i {
                    continue;
                }
                if raise {
                    let Some(floor) = profiles[i].as_ref().and_then(|p| p.last().copied()) else { continue };
                    if let Some(p) = profiles[j].as_mut() {
                        for v in p.iter_mut().filter(|v| **v < floor) {
                            *v = floor;
                            changed = true;
                        }
                    }
                } else {
                    let Some(cap) = at(profiles, j, k) else { continue };
                    if let Some(p) = profiles[i].as_mut() {
                        for v in p.iter_mut().filter(|v| **v > cap) {
                            *v = cap;
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
}

/// The drawn stroke of one run, cut where the run crosses open water.
pub struct StrokePieces {
    /// Every drawn point, pieces concatenated in order.
    pub pts: Vec<(f64, f64)>,
    /// Each point's [`render_params`] value (for its width and colour).
    pub u: Vec<f64>,
    /// `[start, end)` into `pts`, one per piece, each at least two points.
    pub pieces: Vec<(usize, usize)>,
}

/// Cut the render curve `rp` (with its [`render_params`] `u`) only where the
/// traced run crosses water, and end each piece exactly on the shoreline.
///
/// `traced_wet[k]` says whether traced point `k` lies on water (a lake or the
/// sea, as the map draws them); `wet(p)` is the same test at any point.
///
/// **Why the traced run, not every render point.** A stroke that tests each
/// render point on its own cuts wherever its spline grazes a water cell's
/// corner, and then drops any piece of one point -- the fragmentation
/// `_riverzoom_probe` counted. Here a gap exists only for a *span* of
/// consecutive wet traced points, i.e. where the river actually runs through
/// a lake. Within the render points belonging to that span (`u` within one
/// traced step of it), the gap runs from the first to the last point that is
/// really on water, widened outwards while the curve is still on water, and
/// its two ends are the exact shoreline crossings ([`crossing`]). A span whose
/// curve never touches water (the spline cut the corner around a one-cell
/// lake) makes no gap: the stroke is continuous there, as it looks.
///
/// So a river entering a lake stops at the inlet and resumes at the outlet;
/// one ending in a lake or the sea stops at its shore; and nothing else cuts
/// it.
pub fn stroke_pieces(rp: &[(f64, f64)], u: &[f64], traced_wet: &[bool], wet: impl Fn((f64, f64)) -> bool) -> StrokePieces {
    let n = rp.len().min(u.len());
    // Gaps as inclusive render-index ranges of points NOT drawn.
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    let mut k = 0usize;
    while k < traced_wet.len() {
        if !traced_wet[k] {
            k += 1;
            continue;
        }
        let k0 = k;
        while k < traced_wet.len() && traced_wet[k] {
            k += 1;
        }
        let k1 = k - 1;
        let (lo, hi) = (k0 as f64 - 1.0, k1 as f64 + 1.0);
        let window: Vec<usize> = (0..n).filter(|&j| u[j] > lo && u[j] < hi).collect();
        let first = window.iter().copied().find(|&j| wet(rp[j]));
        let last = window.iter().rev().copied().find(|&j| wet(rp[j]));
        if let (Some(mut a), Some(mut b)) = (first, last) {
            while a > 0 && wet(rp[a - 1]) {
                a -= 1;
            }
            while b + 1 < n && wet(rp[b + 1]) {
                b += 1;
            }
            gaps.push((a, b));
        }
    }
    gaps.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for g in gaps {
        match merged.last_mut() {
            Some(m) if g.0 <= m.1 + 1 => m.1 = m.1.max(g.1),
            _ => merged.push(g),
        }
    }

    let mut out = StrokePieces { pts: Vec::new(), u: Vec::new(), pieces: Vec::new() };
    let lerp = |a: (f64, f64), b: (f64, f64), t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    // Dry intervals between gaps, as inclusive ranges, with the shoreline
    // point (if any) that opens and closes each.
    let mut start = 0usize;
    let mut open: Option<((f64, f64), f64)> = None;
    let mut emit = |from: usize, to: Option<usize>, open: Option<((f64, f64), f64)>, close: Option<((f64, f64), f64)>| {
        let s = out.pts.len();
        if let Some((p, uu)) = open {
            out.pts.push(p);
            out.u.push(uu);
        }
        if let Some(to) = to {
            for j in from..=to {
                out.pts.push(rp[j]);
                out.u.push(u[j]);
            }
        }
        if let Some((p, uu)) = close {
            out.pts.push(p);
            out.u.push(uu);
        }
        if out.pts.len() - s >= 2 {
            out.pieces.push((s, out.pts.len()));
        } else {
            out.pts.truncate(s);
            out.u.truncate(s);
        }
    };
    for &(a, b) in &merged {
        // Dry points start..a-1, then the inlet crossing between a-1 and a.
        let close = (a > 0).then(|| {
            let t = crossing(rp[a - 1], rp[a], &wet);
            (lerp(rp[a - 1], rp[a], t), u[a - 1] + (u[a] - u[a - 1]) * t)
        });
        if a > start || open.is_some() {
            emit(start, (a > start).then(|| a - 1), open, close);
        }
        // The outlet crossing between b and b+1 opens the next piece.
        open = (b + 1 < n).then(|| {
            // `crossing` wants dry -> wet, so walk it from the dry side.
            let t = crossing(rp[b + 1], rp[b], &wet);
            (lerp(rp[b + 1], rp[b], t), u[b + 1] + (u[b] - u[b + 1]) * t)
        });
        start = b + 1;
    }
    if start < n {
        emit(start, Some(n - 1), open, None);
    }
    out
}

/// A tapered, antialiased stroke as one indexed triangle list, ready for
/// `RenderingServer.canvas_item_add_triangle_array`.
///
/// `pts` are screen pixels, `half_w` the stroke's half-width at each point in
/// screen pixels, `colors` its colour. Godot's `draw_polyline` takes **one**
/// width per call, which is why the old stroke could only be one width per
/// run; this builds the strip itself.
///
/// Four vertices per point across the stroke -- outer fringe (alpha 0), inner
/// edge, inner edge, outer fringe -- offset along the point's normal (the
/// perpendicular of the chord to its neighbours, so a join has no gap and no
/// overlap wedge on the outside of a bend). The offset is not miter-scaled:
/// the render curve is a spline sampled every quarter cell
/// (`WAY_RENDER_STEP_CELLS`), so the turn between neighbours is small and the
/// width lost on a bend is `1 - cos(half the turn)`.
///
/// `view` culls: a segment whose box, grown by its width, misses `view` is
/// not emitted, and the strip restarts on the far side. The index array is
/// what is culled; vertices are emitted only for points a kept segment uses.
pub struct StripMesh {
    pub pts: Vec<(f32, f32)>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<i32>,
}

pub fn stroke_mesh(
    pts: &[(f32, f32)],
    half_w: &[f32],
    colors: &[[f32; 4]],
    view: Option<(f32, f32, f32, f32)>,
) -> StripMesh {
    let n = pts.len().min(half_w.len()).min(colors.len());
    let mut m = StripMesh { pts: Vec::new(), colors: Vec::new(), indices: Vec::new() };
    if n < 2 {
        return m;
    }
    let f = EDGE_FRINGE_PX;
    let visible = |i: usize| -> bool {
        let Some((vx, vy, vw, vh)) = view else { return true };
        let pad = half_w[i].max(half_w[i + 1]) + f;
        let (a, b) = (pts[i], pts[i + 1]);
        let (x0, x1) = (a.0.min(b.0) - pad, a.0.max(b.0) + pad);
        let (y0, y1) = (a.1.min(b.1) - pad, a.1.max(b.1) + pad);
        x1 >= vx && x0 <= vx + vw && y1 >= vy && y0 <= vy + vh
    };
    // Unit normal at point i: perpendicular of the chord between the nearest
    // distinct neighbours either side.
    let normal = |i: usize| -> (f32, f32) {
        let mut a = i;
        let mut b = i;
        let same = |p: (f32, f32), q: (f32, f32)| p.0 == q.0 && p.1 == q.1;
        while a > 0 && same(pts[a], pts[i]) {
            a -= 1;
        }
        while b + 1 < n && same(pts[b], pts[i]) {
            b += 1;
        }
        let (dx, dy) = (pts[b].0 - pts[a].0, pts[b].1 - pts[a].1);
        let l = (dx * dx + dy * dy).sqrt();
        if l > 0.0 { (-dy / l, dx / l) } else { (0.0, 0.0) }
    };
    let mut base: Vec<i32> = vec![-1; n];
    let mut vert = |i: usize, m: &mut StripMesh| -> i32 {
        if base[i] >= 0 {
            return base[i];
        }
        let (nx, ny) = normal(i);
        let p = pts[i];
        let hw = half_w[i];
        let inner = hw;
        let outer = hw + f;
        let c = colors[i];
        let clear = [c[0], c[1], c[2], 0.0];
        let at = |d: f32| (p.0 + nx * d, p.1 + ny * d);
        let b = m.pts.len() as i32;
        m.pts.extend_from_slice(&[at(outer), at(inner), at(-inner), at(-outer)]);
        m.colors.extend_from_slice(&[clear, c, c, clear]);
        base[i] = b;
        b
    };
    for i in 0..n - 1 {
        if !visible(i) {
            continue;
        }
        let a = vert(i, &mut m);
        let b = vert(i + 1, &mut m);
        for s in 0..3 {
            let (a0, a1, b0, b1) = (a + s, a + s + 1, b + s, b + s + 1);
            m.indices.extend_from_slice(&[a0, a1, b0, a1, b1, b0]);
        }
    }
    m
}

/// Each run's place in the draw order (`get_rivers()`' `draw_rank`): every
/// tributary before the run it joins, from the same `joins` that
/// [`settle_join_widths`] reads (`Some((j, k))` with `k > 0`: run `i` ends on
/// point `k` of run `j`).
///
/// A tributary's stroke ends on its trunk's centreline, so whichever of the
/// two is drawn second paints the join. Drawn second, the trunk covers the
/// tributary's end with its own bank; drawn first, the tributary's lighter
/// colour lies across the trunk as a tab -- half the trunk's width, which is
/// 100+ px at deep zoom. The list's own order drew about 94% of confluences
/// the wrong way round (`_riverzoom_probe`, three worlds, before RV-2).
///
/// **Topological, not by Strahler order -- the first build sorted by
/// `own_order` and the probe refuted it:** on seed 71077345 an order-3 run
/// ends on an order-1 one, because `trace_river_polylines` follows
/// `build_channels`' receiver tree while `stream_order` was computed on
/// another, so a trace's order says nothing reliable about which run is the
/// trunk. The join itself does.
///
/// Kahn's algorithm with edges tributary -> trunk; among runs ready together,
/// the later-traced first. A continuation (`k == 0`) is no edge: its runs
/// meet end to head, and neither covers the other. Anything left in a cycle
/// (bridges can in principle close one) follows in the same later-first order.
pub fn draw_ranks(joins: &[Option<(usize, usize)>]) -> Vec<usize> {
    let n = joins.len();
    let trunk = |i: usize| joins[i].filter(|&(j, k)| k > 0 && j != i && j < n).map(|(j, _)| j);
    let mut waiting = vec![0usize; n];
    for i in 0..n {
        if let Some(j) = trunk(i) {
            waiting[j] += 1;
        }
    }
    let mut ready: std::collections::BinaryHeap<usize> = (0..n).filter(|&i| waiting[i] == 0).collect();
    let mut rank = vec![usize::MAX; n];
    let mut next = 0usize;
    while let Some(i) = ready.pop() {
        rank[i] = next;
        next += 1;
        if let Some(j) = trunk(i) {
            waiting[j] -= 1;
            if waiting[j] == 0 {
                ready.push(j);
            }
        }
    }
    for i in (0..n).rev() {
        if rank[i] == usize::MAX {
            rank[i] = next;
            next += 1;
        }
    }
    rank
}

/// Each point's drawn half-width in screen pixels: its full width in cells
/// times `px_per_cell`, floored at [`MIN_STROKE_PX`] before halving.
pub fn stroke_half_widths(widths_cells: &[f32], px_per_cell: f32) -> Vec<f32> {
    widths_cells.iter().map(|&x| (x * px_per_cell).max(MIN_STROKE_PX) * 0.5).collect()
}

#[godot_api(secondary)]
impl WorldGen {
    /// RV-2: every drawn river's stroke as **one** triangle mesh in screen
    /// pixels, for `map_overlay.gd::_draw_rivers` to hand straight to
    /// `RenderingServer.canvas_item_add_triangle_array`. Static -- it reads no
    /// world state, only its arguments. One call for the whole network rather
    /// than one per river: a world draws ~1 000 runs, and a GDScript loop
    /// marshalling each one across the boundary on every redraw was the cost.
    ///
    /// - `rivers`: `get_rivers()`' array, typed or untyped. A run without `widths`, or carrying
    ///   `parallel_of`, is skipped (`get_rivers()`' doc); each other run's
    ///   `render_points`, `widths` (full width in grid cells), `colors` and
    ///   `pieces` are drawn, in ascending `draw_rank` ([`draw_ranks`]), so a
    ///   trunk is drawn over the ends of the tributaries that join it. A run
    ///   without `draw_rank` is drawn first.
    /// - `scale`, `offset`: grid to screen, `screen = offset + point * scale`.
    /// - `px_per_cell`: screen pixels per grid cell of width.
    /// - `o1_width`, `o1_alpha`: `drawRiverWays`' order-1 de-emphasis, applied
    ///   to a run whose `own_order` is 1 (`1.0` and `1.0` switch it off).
    /// - `view`: the visible screen rectangle; segments missing it are culled.
    ///
    /// Each point's half-width is `max(width * px_per_cell * w, MIN_STROKE_PX)
    /// / 2`. Returns `points` (`PackedVector2Array`), `colors`
    /// (`PackedColorArray`) and `indices` (`PackedInt32Array`), all empty when
    /// nothing is visible. A run whose arrays disagree in length is skipped.
    #[func]
    #[allow(clippy::too_many_arguments)]
    fn river_strokes_mesh(
        rivers: Variant,
        scale: Vector2,
        offset: Vector2,
        px_per_cell: f32,
        o1_width: f32,
        o1_alpha: f32,
        view: Rect2,
    ) -> VarDictionary {
        let vr = Some((view.position.x, view.position.y, view.size.x, view.size.y));
        // `get_rivers()` returns a typed `Array[Dictionary]`, and a shell
        // variable holding it may be typed or not; gdext converts neither into
        // the other implicitly, so take either.
        let list: Vec<VarDictionary> = if let Ok(a) = rivers.try_to::<Array<VarDictionary>>() {
            a.iter_shared().collect()
        } else if let Ok(a) = rivers.try_to::<VarArray>() {
            a.iter_shared().filter_map(|v| v.try_to::<VarDictionary>().ok()).collect()
        } else {
            Vec::new()
        };
        let mut out_p: Vec<Vector2> = Vec::new();
        let mut out_c: Vec<Color> = Vec::new();
        let mut out_i: Vec<i32> = Vec::new();
        let own_order = |r: &VarDictionary| r.get("own_order").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(2);
        let rank = |r: &VarDictionary| r.get("draw_rank").and_then(|v| v.try_to::<i64>().ok());
        let mut order: Vec<usize> = (0..list.len()).collect();
        order.sort_by_key(|&i| rank(&list[i]));
        for r in order.into_iter().map(|i| &list[i]) {
            if r.contains_key("parallel_of") {
                continue;
            }
            let get = |k: &str| r.get(k);
            let (Some(p), Some(w), Some(c), Some(pc)) = (
                get("render_points").and_then(|v| v.try_to::<PackedVector2Array>().ok()),
                get("widths").and_then(|v| v.try_to::<PackedFloat32Array>().ok()),
                get("colors").and_then(|v| v.try_to::<PackedColorArray>().ok()),
                get("pieces").and_then(|v| v.try_to::<PackedInt32Array>().ok()),
            ) else {
                continue;
            };
            let (p, w, c, pc) = (p.as_slice(), w.as_slice(), c.as_slice(), pc.as_slice());
            if p.len() != w.len() || p.len() != c.len() {
                continue;
            }
            let (wm, am) = if own_order(r) <= 1 { (o1_width, o1_alpha) } else { (1.0, 1.0) };
            for pair in pc.chunks_exact(2) {
                let (s, e) = (pair[0].max(0) as usize, (pair[1].max(0) as usize).min(p.len()));
                if e < s + 2 {
                    continue;
                }
                let sp: Vec<(f32, f32)> =
                    p[s..e].iter().map(|q| (offset.x + q.x * scale.x, offset.y + q.y * scale.y)).collect();
                let hw = stroke_half_widths(&w[s..e], px_per_cell * wm);
                let cc: Vec<[f32; 4]> = c[s..e].iter().map(|k| [k.r, k.g, k.b, k.a * am]).collect();
                let mesh = stroke_mesh(&sp, &hw, &cc, vr);
                let b = out_p.len() as i32;
                out_p.extend(mesh.pts.iter().map(|q| Vector2::new(q.0, q.1)));
                out_c.extend(mesh.colors.iter().map(|k| Color::from_rgba(k[0], k[1], k[2], k[3])));
                out_i.extend(mesh.indices.iter().map(|&i| b + i));
            }
        }
        vdict! {
            "points" => &PackedVector2Array::from(out_p.as_slice()),
            "colors" => &PackedColorArray::from(out_c.as_slice()),
            "indices" => &PackedInt32Array::from(out_i.as_slice()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize) -> Vec<(f64, f64)> {
        (0..n).map(|x| (x as f64 + 0.5, 0.5)).collect()
    }

    #[test]
    fn render_params_follow_the_traced_run_and_never_go_back() {
        let pts = line(5);
        let rp: Vec<(f64, f64)> = (0..17).map(|j| (0.5 + j as f64 * 0.25, 0.5 + 0.1 * (j as f64).sin())).collect();
        let u = render_params(&rp, &pts);
        assert_eq!(u[0], 0.0);
        assert!((u[16] - 4.0).abs() < 1e-12, "{u:?}");
        assert!((u[6] - 1.5).abs() < 1e-12, "a quarter-cell step is a quarter index: {u:?}");
        assert!(u.windows(2).all(|v| v[1] >= v[0]));
        assert_eq!(sample_at(&[1.0, 3.0, 5.0], 1.25), 3.5);
        assert_eq!(sample_at(&[1.0, 3.0, 5.0], 9.0), 5.0);
    }

    /// The fragmentation the probe counted: a curve that grazes a lake cell it
    /// does not run through must not be cut at all.
    #[test]
    fn a_curve_grazing_a_lake_it_does_not_cross_stays_whole() {
        let pts = line(9);
        let rp: Vec<(f64, f64)> = (0..33).map(|j| (0.5 + j as f64 * 0.25, 0.5)).collect();
        let u = render_params(&rp, &pts);
        // Lake cells scattered on the curve's own row, but the TRACE is dry.
        let wet = |p: (f64, f64)| matches!(p.0.floor() as i64, 2 | 5);
        let s = stroke_pieces(&rp, &u, &vec![false; pts.len()], wet);
        assert_eq!(s.pieces, vec![(0, rp.len())], "one piece, no cut");
    }

    /// A traced cell classed as lake that the drawn curve never actually
    /// crosses (the spline cut the corner round a one-cell lake) makes no gap:
    /// the stroke is continuous where it looks continuous.
    #[test]
    fn a_traced_lake_cell_the_curve_misses_makes_no_gap() {
        let pts = line(5);
        let rp: Vec<(f64, f64)> = (0..17).map(|j| (0.5 + j as f64 * 0.25, 0.5)).collect();
        let u = render_params(&rp, &pts);
        let tw = [false, false, true, false, false];
        let s = stroke_pieces(&rp, &u, &tw, |_| false);
        assert_eq!(s.pieces, vec![(0, rp.len())]);
    }

    /// A render curve that steps back along the run for a moment (a spline
    /// overshoot at a tight bend) still gets a position that never decreases,
    /// so the width interpolated along it never narrows.
    #[test]
    fn render_params_never_decrease_through_an_overshoot() {
        let pts = line(4);
        let rp = [(0.5, 0.5), (1.2, 0.5), (1.6, 0.5), (1.4, 0.6), (2.0, 0.5), (3.5, 0.5)];
        let u = render_params(&rp, &pts);
        assert!(u.windows(2).all(|v| v[1] >= v[0]), "{u:?}");
        assert!((u[2] - 1.1).abs() < 1e-12 && (u[3] - 1.1).abs() < 1e-12, "held, not stepped back: {u:?}");
    }

    /// A river through a lake stops at the inlet shore and resumes at the
    /// outlet shore -- exactly on the lake cells' edges -- in two pieces.
    #[test]
    fn a_river_through_a_lake_stops_at_the_inlet_and_resumes_at_the_outlet() {
        let pts = line(12);
        let rp: Vec<(f64, f64)> = (0..45).map(|j| (0.5 + j as f64 * 0.25, 0.5)).collect();
        let u = render_params(&rp, &pts);
        // Lake: cells x = 4..=6.
        let wet = |p: (f64, f64)| (4.0..7.0).contains(&p.0);
        let tw: Vec<bool> = pts.iter().map(|&p| wet(p)).collect();
        let s = stroke_pieces(&rp, &u, &tw, wet);
        assert_eq!(s.pieces.len(), 2, "{:?}", s.pieces);
        let (a0, a1) = s.pieces[0];
        let (b0, b1) = s.pieces[1];
        assert_eq!(s.pts[a0], rp[0]);
        assert!((s.pts[a1 - 1].0 - 4.0).abs() < 1e-9, "ends on the inlet shore: {:?}", s.pts[a1 - 1]);
        assert!((s.pts[b0].0 - 7.0).abs() < 1e-9, "resumes on the outlet shore: {:?}", s.pts[b0]);
        assert_eq!(s.pts[b1 - 1], rp[rp.len() - 1]);
        assert!(s.pts.iter().all(|&p| !(p.0 > 4.0 + 1e-9 && p.0 < 7.0 - 1e-9)), "nothing drawn on the water");
        // The shoreline points carry the interpolated position along the run.
        assert!((s.u[a1 - 1] - 3.5).abs() < 1e-9 && (s.u[b0] - 6.5).abs() < 1e-9);
    }

    /// A river whose last traced cells are water ends on the shore; one that
    /// starts in water (a lake outlet) starts on it.
    #[test]
    fn a_stroke_ends_and_starts_on_the_shore() {
        let pts = line(8);
        let rp: Vec<(f64, f64)> = (0..29).map(|j| (0.5 + j as f64 * 0.25, 0.5)).collect();
        let u = render_params(&rp, &pts);
        let sea = |p: (f64, f64)| p.0 >= 6.0;
        let tw: Vec<bool> = pts.iter().map(|&p| sea(p)).collect();
        let s = stroke_pieces(&rp, &u, &tw, sea);
        assert_eq!(s.pieces.len(), 1);
        assert!((s.pts[s.pts.len() - 1].0 - 6.0).abs() < 1e-9);
        let lake = |p: (f64, f64)| p.0 < 2.0;
        let tw: Vec<bool> = pts.iter().map(|&p| lake(p)).collect();
        let s = stroke_pieces(&rp, &u, &tw, lake);
        assert_eq!(s.pieces.len(), 1);
        assert!((s.pts[0].0 - 2.0).abs() < 1e-9, "{:?}", s.pts[0]);
    }

    #[test]
    fn the_mesh_tapers_with_the_width_and_fringes_to_clear() {
        let pts: Vec<(f32, f32)> = (0..4).map(|i| (i as f32 * 10.0, 0.0)).collect();
        let hw = [0.5f32, 1.0, 2.0, 4.0];
        let c = [[0.1f32, 0.2, 0.3, 1.0]; 4];
        let m = stroke_mesh(&pts, &hw, &c, None);
        assert_eq!(m.pts.len(), 16);
        assert_eq!(m.indices.len(), 3 * 6 * 3, "six triangles per segment");
        for (i, &h) in hw.iter().enumerate() {
            let v = &m.pts[i * 4..i * 4 + 4];
            // Horizontal line: the normal is vertical; solid to the edge at h,
            // fringe out to h + 1 px.
            assert_eq!(v[0].1.abs(), h + 1.0, "outer fringe at point {i}");
            assert_eq!(v[1].1.abs(), h, "solid to the edge at point {i}");
            assert_eq!(m.colors[i * 4][3], 0.0);
            assert_eq!(m.colors[i * 4 + 1][3], 1.0);
        }
        // Culling: a view that sees only the last segment emits only it.
        let m = stroke_mesh(&pts, &hw, &c, Some((25.0, -5.0, 10.0, 10.0)));
        assert_eq!(m.indices.len(), 18);
        assert_eq!(m.pts.len(), 8);
    }

    /// A mouth on land ends on the shore: at the field's sea-level crossing
    /// toward the sea cell it drains into, at the cell edge toward a lake.
    #[test]
    fn a_coastal_mouth_is_carried_to_the_shore() {
        let (w, h) = (6usize, 3usize);
        let mut water = vec![0u8; w * h];
        let mut fld = vec![0.6f32; w * h];
        // Sea at x >= 4 on row 1; the mouth is (3,1) at 0.5, the sea at 0.2.
        for x in 4..w {
            water[w + x] = 1;
            fld[w + x] = 0.2;
        }
        fld[w + 3] = 0.5;
        let pts = [(1.5, 1.5), (2.5, 1.5), (3.5, 1.5)];
        let p = coast_end(&pts, &water, &fld, 0.4, None, w, h).unwrap();
        // (0.5 - 0.4) / (0.5 - 0.2) = 1/3 of the way from 3.5 to 4.5.
        assert!((p.0 - (3.5 + 1.0 / 3.0)).abs() < 1e-6 && (p.1 - 1.5).abs() < 1e-12, "{p:?}");
        // The same shore as a lake: its cell edge, x = 4.
        for x in 4..w {
            water[w + x] = 2;
        }
        let p = coast_end(&pts, &water, &fld, 0.4, None, w, h).unwrap();
        assert!((p.0 - 4.0).abs() < 1e-9, "{p:?}");
        // Water only BEHIND the run is not a shore it reaches.
        // The run flows +x into (3,1); the only water is up-left of it.
        let mut behind = vec![0u8; w * h];
        behind[2] = 2;
        assert_eq!(coast_end(&pts, &behind, &fld, 0.4, None, w, h), None);
        // Positive control for that: the same water straight on is reached.
        behind[4] = 2;
        assert!(coast_end(&pts, &behind, &fld, 0.4, None, w, h).is_some());
        // recv wins over the straight-on neighbour: drain diagonally.
        let mut diag = vec![0u8; w * h];
        diag[2 * w + 4] = 1;
        diag[w + 4] = 1;
        fld[2 * w + 4] = 0.2;
        let mut recv = vec![-1i32; w * h];
        recv[w + 3] = (2 * w + 4) as i32;
        let p = coast_end(&pts, &diag, &fld, 0.4, Some(&recv), w, h).unwrap();
        assert!(p.1 > 1.5 && p.0 > 3.5, "follows recv to the diagonal: {p:?}");
        // No water beside the mouth: no extension.
        assert_eq!(coast_end(&pts, &vec![0u8; w * h], &fld, 0.4, None, w, h), None);
    }

    /// A tributary is capped at its trunk's width where it joins, and the cap
    /// runs down a chain: C joins B joins A, and A is the narrowest there.
    #[test]
    fn a_tributary_is_never_wider_than_its_trunk_where_it_joins() {
        // C is listed BEFORE the B it joins, so one pass caps C by B's
        // uncapped 1.4 and only the fixed-point repeat brings it to 0.8.
        let mut p = vec![
            Some(vec![0.5, 0.8, 1.0, 1.2]), // A: the trunk
            Some(vec![0.6, 0.9, 1.3]),      // C: joins B at B's point 2
            Some(vec![0.7, 1.1, 1.4]),      // B: joins A at A's point 1 (0.8)
            None,                           // D: no width at all
            Some(vec![0.2, 0.3]),           // E: joins A at point 3 and is narrower
        ];
        let joins = [None, Some((2, 2)), Some((0, 1)), Some((0, 2)), Some((0, 3))];
        settle_join_widths(&mut p, &joins);
        assert_eq!(p[0], Some(vec![0.5, 0.8, 1.0, 1.2]), "the trunk is untouched");
        assert_eq!(p[2], Some(vec![0.7, 0.8, 0.8]));
        assert_eq!(p[1], Some(vec![0.6, 0.8, 0.8]), "capped down the chain");
        assert_eq!(p[3], None, "no width stays no width");
        assert_eq!(p[4], Some(vec![0.2, 0.3]), "a narrower tributary keeps its own width");
    }

    /// A run bridged onto the HEAD of another is one river carrying on: the
    /// next run is widened to it, never the other way round -- and a chain of
    /// two continuations carries the width through both.
    #[test]
    fn a_continuation_widens_the_run_it_carries_on_as() {
        // Y is listed before the X that feeds it, so one pass raises Z from
        // Y's un-raised end and only the fixed-point repeat carries X's width
        // through to Z.
        let mut p = vec![
            Some(vec![0.5, 0.6, 0.7]), // Y: bridged from X at its head
            Some(vec![1.0, 1.5, 2.0]), // X: a trunk ending on a land pit
            Some(vec![0.5, 0.5]),      // Z: bridged from Y at its head
        ];
        let joins = [Some((2, 0)), Some((0, 0)), None];
        settle_join_widths(&mut p, &joins);
        assert_eq!(p[1], Some(vec![1.0, 1.5, 2.0]), "the incoming run keeps its width");
        assert_eq!(p[0], Some(vec![2.0, 2.0, 2.0]), "raised to X's end");
        assert_eq!(p[2], Some(vec![2.0, 2.0]), "and on through Y to Z");

        // Now Y itself joins a trunk T that is narrower where Y meets it: Y is
        // capped, and X -- carried on as Y -- is capped with it, so the two
        // still meet at one width instead of stepping down at the bridge.
        let mut p = vec![
            Some(vec![0.5, 0.6, 0.7]), // Y
            Some(vec![1.0, 1.5, 2.0]), // X, bridged onto Y's head
            Some(vec![0.9, 1.2, 1.4]), // T: Y joins it at point 1 (1.2)
        ];
        let joins = [Some((2, 1)), Some((0, 0)), None];
        settle_join_widths(&mut p, &joins);
        assert_eq!(p[0], Some(vec![1.2, 1.2, 1.2]), "raised to 2.0, then capped at T's 1.2");
        assert_eq!(p[1], Some(vec![1.0, 1.2, 1.2]), "X capped at Y's head");
    }

    /// Every tributary is drawn before the run it joins, whatever the
    /// indices; a continuation imposes no order; ready runs go later-first.
    #[test]
    fn trunks_are_drawn_after_the_tributaries_that_join_them() {
        // 0 <- 1 <- 3 (a chain: 3 joins 1, 1 joins 0); 4 joins 0; 2 is alone;
        // 5 continues onto 2's head (k == 0: no edge). Index order would draw
        // 0 first, under everything that joins it.
        let joins = [None, Some((0, 4)), None, Some((1, 2)), Some((0, 7)), Some((2, 0))];
        let r = draw_ranks(&joins);
        assert!(r[3] < r[1] && r[1] < r[0] && r[4] < r[0], "{r:?}");
        // Ready at the start: 2, 3, 4, 5 -- later-traced first.
        assert_eq!((r[5], r[4], r[3], r[2]), (0, 1, 2, 3), "{r:?}");
        assert_eq!((r[1], r[0]), (4, 5), "{r:?}");
        // A cycle cannot strand a run without a rank.
        let r = draw_ranks(&[Some((1, 1)), Some((0, 1))]);
        let mut s = r.clone();
        s.sort();
        assert_eq!(s, vec![0, 1]);
    }

    /// The owner's floor: a stroke narrower than a pixel is drawn at one.
    #[test]
    fn a_sub_pixel_stroke_is_drawn_one_pixel_wide() {
        // 0.3 cells at 0.9 px/cell is 0.27 px on screen; 4 cells is 3.6 px.
        let hw = stroke_half_widths(&[0.3, 4.0], 0.9);
        assert_eq!(hw[0], 0.5, "floored at one pixel");
        assert!((hw[1] - 1.8).abs() < 1e-6, "a wide stroke is not touched: {hw:?}");
        // And the floored stroke is a solid pixel with its fringe outside it.
        let m = stroke_mesh(&[(0.0, 0.0), (10.0, 0.0)], &hw[..1].repeat(2), &[[0.0, 0.0, 0.0, 1.0]; 2], None);
        assert_eq!((m.pts[0].1.abs(), m.pts[1].1.abs()), (1.5, 0.5));
    }
}
