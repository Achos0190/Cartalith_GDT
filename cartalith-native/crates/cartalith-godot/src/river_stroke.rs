//! RV-2 -- the drawn river stroke's geometry (`OUTSTANDING_WORK.md`, "RV-2,
//! RV-4, RV-5: smooth rivers and lakes at every zoom"). Owner, 2026-09-29:
//! rivers must always be smooth, never *"the pixilated depressions that are
//! either yes or no connected all the way to nonsensical partial lines."*
//!
//! `_riverzoom_probe.gd` measured three causes in the one drawn river, the
//! vector stroke drawn from `get_rivers(1)` (then by `map_overlay.gd` over the
//! finished map; since 2026-09-27 the style reaches it -- tiles rasterize it
//! ([`rasterize`]) and the base view textures it ([`rasterize_colour_field`],
//! `WorldGen::river_view_mesh`) -- so the
//! style preset reaches it -- owner: *"they're drawn on top of the style"*):
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
//! The raster half ([`rasterize`]) is reached only by the screen texture and
//! the deep-zoom tiles; `js_reference()`, every JS golden and every export
//! attach no river layer.

use godot::prelude::*;

use crate::render;
use crate::WorldGen;

/// The narrowest a river stroke is ever drawn, in **raster** pixels (the
/// screen texture's one pixel per cell, or a deep-zoom tile's own pixel; a
/// screen pixel when the overlay drew it). The
/// owner's bar for RV-2 ("headwaters never draw thinner than 1 px") restated,
/// not a tuning choice: below one pixel an antialiased stroke is drawn as a
/// fraction of a pixel's coverage, which a light headwater colour over land
/// turns into the dotted, on-off line the probe counted.
pub const MIN_STROKE_PX: f32 = 1.0;

/// The antialiasing fringe outside each edge of a stroke, in raster pixels:
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

/// A tapered, antialiased stroke as one indexed triangle list, which
/// [`rasterize`] fills into a `render::RiverLayer` (it was handed to
/// `RenderingServer.canvas_item_add_triangle_array` until 2026-09-27).
///
/// `pts` are raster pixels, `half_w` the stroke's half-width at each point in
/// raster pixels, `colors` its colour. Godot's `draw_polyline` takes **one**
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


/// One drawn river run, owned -- `get_rivers()`' `render_points`, `widths`,
/// `colors` and `pieces`, before marshalling (`WorldGen::river_geometry`).
/// Exists so the rasterizers read the SAME geometry `get_rivers()` reports
/// without a round trip through Godot types. Must never hold a run
/// `get_rivers()` documents as not drawn (`parallel_of`, no `widths`): the
/// builder filters those, and nothing here re-checks.
pub struct DrawnRun {
    /// Render points in grid-cell space (a cell's centre is `x + 0.5`).
    pub pts: Vec<(f32, f32)>,
    /// Full drawn width in grid cells at each point.
    pub widths: Vec<f32>,
    /// Straight RGBA in `0..=1` at each point: the per-Strahler-order palette.
    pub colors: Vec<[f32; 4]>,
    /// Strahler order of the traced cell nearest each point.
    pub orders: Vec<i16>,
    /// `WorldState::flow_discharge` of the traced cell nearest each point
    /// (NaN where the field had no reading -- never a plausible zero).
    pub discharge: Vec<f32>,
    /// `[start, end)` into `pts`, one per drawn piece.
    pub pieces: Vec<(usize, usize)>,
    /// The run's highest order excluding the junction cell it ends on
    /// (`get_rivers()`' `own_order`); `1` is a headwater trickle.
    pub own_order: i16,
}

/// One point of a river as [`river_px_width`] sees it. `order` and
/// `discharge` are carried for the zoom rule that will replace the seam's body
/// (coordinator, 2026-09-27: width and visibility must be a function of scale,
/// order and discharge) and are not read by today's. Must never be built with
/// a made-up discharge: NaN means "no reading" (`DrawnRun::discharge`).
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct RiverPoint {
    /// RV-2's full width in grid cells (`channel_disc`'s law, settled at
    /// confluences).
    pub width_cells: f32,
    /// Strahler order of the traced cell nearest the point.
    pub order: i16,
    /// The run's own order ([`DrawnRun::own_order`]).
    pub own_order: i16,
    /// Flow discharge at the traced cell nearest the point (NaN: none).
    pub discharge: f32,
}

/// **The one seam that decides how wide, how opaque and whether a river
/// point is drawn in a raster**: `Some((full width in raster pixels, alpha
/// multiplier))`, or `None` for "not drawn at this scale".
///
/// It takes everything a zoom-sensitive rule would key on -- the raster's
/// density (`px_per_cell`), the point's stream order and discharge, its RV-2
/// width, and the preset (`a`) -- so a future rule (select rivers by order per
/// zoom, grow from a fixed pixel width toward the real metre width, hand over
/// to a banked water polygon) replaces this body and nothing else. Today's
/// body is RV-2's look exactly: the width on the ground times the preset's
/// `river_width`, the order-1 de-emphasis ([`o1_deemphasis`]) by `own_order`,
/// floored at [`MIN_STROKE_PX`], and every point drawn. `order` and
/// `discharge` are carried and not yet read.
///
/// Must never return a width below [`MIN_STROKE_PX`] for a drawn point (the
/// owner's RV-2 floor) and must stay a pure function of its arguments: the
/// tiles and the base view call it at different densities and must agree at
/// equal ones.
pub fn river_px_width(p: RiverPoint, px_per_cell: f32, a: &render::TerrainAppearance) -> Option<(f32, f32)> {
    let (wm, am) = if p.own_order <= 1 { o1_deemphasis(px_per_cell) } else { (1.0, 1.0) };
    // `16`: a guard against a hand-edited look file, far above the tunable's
    // own 3.0 ceiling (`render.rs`'s `tunables!`) -- labelled judgement.
    let k = px_per_cell * wm * a.river_width.clamp(0.0, 16.0) as f32;
    Some(((p.width_cells * k).max(MIN_STROKE_PX), am))
}

/// The whole drawn network, **in draw order** (`draw_ranks`: every
/// tributary before the run it joins), so a rasterizer compositing in order
/// paints each trunk over its tributaries' ends as the GPU overlay did. Must
/// never be reordered by a consumer.
pub struct RiverGeometry {
    /// The runs, draw order.
    pub runs: Vec<DrawnRun>,
}

/// `drawRiverWays`' anti-barcode rule (reference 9512, v0.96 + v1.41), which
/// the first vector overlay shipped without: the network carries hundreds of
/// order-1 trickles, and on a smooth slope they run downhill side by side --
/// the reference owner's "green barcode" and this port's owner's "a lot of
/// parallel lines ... chaotic" (2026-09-23). At the opening view an order-1
/// run draws at this width and alpha; the de-emphasis fades out by
/// [`O1_FADE_ZOOM`], where a headwater stream IS the subject.
///
/// **Source: the reference's own numbers**, v2.11 lines 9577-9578:
/// `deEmph = 1 - (zk-1)/7` (gone by `zk = 8`), `o1Alpha = 0.4 + 0.6*(1-deEmph)`,
/// `o1Width = 0.55 + 0.45*(1-deEmph)` -- carried unchanged through
/// `map_overlay.gd` (13062e5) into here.
pub const O1_WIDTH_OPENING: f32 = 0.55;
/// The order-1 alpha at the opening view -- reference v2.11 line 9578 (see
/// [`O1_WIDTH_OPENING`]).
pub const O1_ALPHA_OPENING: f32 = 0.4;
/// Zoom (relative to one raster pixel per cell) at which the order-1
/// de-emphasis has faded out completely -- the reference's `zk = 8`, "the LOD
/// cap" (v2.11 line 9577).
pub const O1_FADE_ZOOM: f32 = 8.0;

/// The order-1 `(width, alpha)` multipliers at `zk`, the raster's pixels per
/// cell (`1` at the screen texture's one pixel per cell, the opening view's
/// counterpart). The map_overlay rule this replaces measured `zk` as camera
/// zoom over the opening view; a raster knows its own density instead, which
/// for a deep-zoom tile is its level's. Must never go below the opening
/// view's factors: `zk` under 1 is clamped to 1.
pub fn o1_deemphasis(zk: f32) -> (f32, f32) {
    let de = (1.0 - (zk.max(1.0) - 1.0) / (O1_FADE_ZOOM - 1.0)).clamp(0.0, 1.0);
    (O1_WIDTH_OPENING + (1.0 - O1_WIDTH_OPENING) * (1.0 - de), O1_ALPHA_OPENING + (1.0 - O1_ALPHA_OPENING) * (1.0 - de))
}

/// How a raster sees grid-cell space: `raster = offset + point * scale`, with
/// raster pixel `(x, y)` sampled at exactly `(x, y)`. One type for the two
/// sampling conventions this crate has (the grid texture's cell centres at
/// `x + 0.5`, the tile's sample coordinates at integers), so neither can be
/// half a pixel off the other. Must never be built by hand: use
/// [`Self::grid`] or [`Self::tile`], which `a_tile_places_a_stroke_at_its_sample_coordinate`
/// pins.
#[derive(Clone, Copy, Debug)]
pub struct RasterMap {
    /// Raster pixels per grid cell, per axis.
    pub scale: (f32, f32),
    /// Raster position of river-space `(0, 0)`.
    pub offset: (f32, f32),
}

impl RasterMap {
    /// The screen texture: one pixel per cell, pixel `x` on cell `x`'s centre
    /// (`x + 0.5` in river space).
    pub fn grid() -> Self {
        RasterMap { scale: (1.0, 1.0), offset: (-0.5, -0.5) }
    }

    /// A deep-zoom tile whose pixel `x` sits at sample coordinate
    /// `bx + x * cx` (`render::TileBounds`' convention: a cell's centre is
    /// its integer index there, `x + 0.5` in river space).
    pub fn tile(bx: f64, by: f64, cx: f64, cy: f64) -> Self {
        RasterMap {
            scale: ((1.0 / cx) as f32, (1.0 / cy) as f32),
            offset: (((-0.5 - bx) / cx) as f32, ((-0.5 - by) / cy) as f32),
        }
    }

    /// Raster pixels per grid cell of width: the geometric mean of the two
    /// axes, so an aspect-matched tile reads one number.
    pub fn px_per_cell(&self) -> f32 {
        (self.scale.0 * self.scale.1).abs().sqrt()
    }
}

/// Every drawn span of every run, through the one seam ([`river_px_width`]) at
/// `ppc` raster pixels per cell, as a [`stroke_mesh`] handed to `sink` in draw
/// order. `to_raster` maps a river-space point (a cell's centre is `x + 0.5`)
/// into the target's pixels; `style(palette colour, seam alpha)` is the
/// vertex colour; `widen` adds to every point's full width, in pixels, after
/// the seam. A point the seam declines (`None`) splits the piece there, so a
/// stroke is only ever drawn between points drawn at this scale.
///
/// Exists so the three consumers (tile raster, base-view colour field, base
/// view screen mesh) cannot walk the network three different ways. Must never
/// reorder runs, and must never bypass the seam.
#[allow(clippy::too_many_arguments)]
fn for_each_span(
    geom: &RiverGeometry,
    a: &render::TerrainAppearance,
    ppc: f32,
    widen: f32,
    to_raster: impl Fn((f32, f32)) -> (f32, f32),
    style: impl Fn([f32; 4], f32) -> [f32; 4],
    view: Option<(f32, f32, f32, f32)>,
    mut sink: impl FnMut(StripMesh),
) {
    for run in &geom.runs {
        let n = run.pts.len();
        if run.widths.len() != n || run.colors.len() != n || run.orders.len() != n || run.discharge.len() != n {
            continue;
        }
        let seam: Vec<Option<(f32, f32)>> = (0..n)
            .map(|i| {
                let p = RiverPoint { width_cells: run.widths[i], order: run.orders[i], own_order: run.own_order, discharge: run.discharge[i] };
                river_px_width(p, ppc, a)
            })
            .collect();
        for &(s0, e0) in &run.pieces {
            let e0 = e0.min(n);
            let mut s = s0;
            while s < e0 {
                while s < e0 && seam[s].is_none() {
                    s += 1;
                }
                let mut e = s;
                while e < e0 && seam[e].is_some() {
                    e += 1;
                }
                if e >= s + 2 {
                    let sp: Vec<(f32, f32)> = run.pts[s..e].iter().map(|&q| to_raster(q)).collect();
                    // Every entry in `s..e` is `Some` by construction of the
                    // span, so the `flatten` drops nothing.
                    let drawn: Vec<(f32, f32)> = seam[s..e].iter().flatten().copied().collect();
                    let hw: Vec<f32> = drawn.iter().map(|&(wpx, _)| (wpx + widen) * 0.5).collect();
                    let cc: Vec<[f32; 4]> = run.colors[s..e].iter().zip(&drawn).map(|(k, &(_, am))| style(*k, am)).collect();
                    sink(stroke_mesh(&sp, &hw, &cc, view));
                }
                s = e.max(s + 1);
            }
        }
    }
}

/// **The rivers into a raster** (owner, 2026-09-27: *"can we put the rivers
/// into the map again with the same vector approach?"*). RV-2's own strokes --
/// [`stroke_mesh`] over every piece of every run, in draw order, widths from
/// [`river_px_width`] with its 1 px floor -- filled into a
/// [`render::RiverLayer`] of `w x h`. `render::land_color` composites the
/// layer before the Painter styles, the paper and the grade, so a preset
/// reaches the rivers like any other map content. **The deep-zoom tiles** draw
/// their rivers this way, each at its own resolution
/// (`lod_bridge::synthesize_tile_rgba_rivers`).
///
/// The preset's own river treatment is applied here: its width multiplier
/// (`river_width`, inside [`river_px_width`]) before the floor, its colour and
/// opacity (`render::river_style_color`) on every vertex. The order-1
/// de-emphasis is [`o1_deemphasis`] at this raster's density, also inside
/// [`river_px_width`] -- the one seam a zoom rule replaces.
///
/// Culled against the raster's own rectangle; `stroke_mesh` grows each
/// segment's box by its width and fringe first, so a stroke whose centreline
/// runs just outside a tile still paints the pixels it reaches -- the tile
/// halo's job for strokes, and why two tiles either side of a boundary fill
/// their shared edge sample identically (the same segments, the same world
/// position).
///
/// Must never be attached to a render the JS goldens or the exports use:
/// those have no river layer by design (`js_reference()`, `BakeFields::pixel`).
pub fn rasterize(geom: &RiverGeometry, a: &render::TerrainAppearance, w: usize, h: usize, map: RasterMap) -> render::RiverLayer {
    let mut layer = render::RiverLayer::new(w, h);
    if w == 0 || h == 0 {
        return layer;
    }
    let view = Some((0.0f32, 0.0f32, (w - 1) as f32, (h - 1) as f32));
    let style = |k: [f32; 4], am: f32| {
        let c = render::river_style_color(a, k);
        [c[0], c[1], c[2], c[3] * am]
    };
    let to = |q: (f32, f32)| (map.offset.0 + q.0 * map.scale.0, map.offset.1 + q.1 * map.scale.1);
    for_each_span(geom, a, map.px_per_cell(), 0.0, to, style, view, |m| layer.fill_triangles(&m.pts, &m.colors, &m.indices));
    layer
}

/// The screen width a base-view river stroke can reach, as a width in cells
/// the colour field must cover around it: [`MIN_STROKE_PX`] plus the fringe
/// either side at the lowest screen density the base view is shown at. A
/// fit view puts a `gw`-cell map in ~[`FIELD_VIEW_PX`] screen pixels or
/// more, so a screen pixel is at most `gw / FIELD_VIEW_PX` cells; below 3
/// cells the pad is 3 regardless.
///
/// **Labelled judgement**: 800 px is a lower bound on the map area's width in
/// the shipped shell at 1366 wide (the narrowest desktop density) with both
/// docks open; a narrower view makes strokes at fit zoom reach past the band,
/// where they sample the terrain colour and read thinner, never wrong-coloured.
pub const FIELD_VIEW_PX: f32 = 800.0;

/// The colour field's pad, in cells, each side of a river -- see
/// [`FIELD_VIEW_PX`]. The `3.0` floor is a labelled judgement: it covers a
/// 1 px stroke plus fringes (3 px) at one screen pixel per cell. Pinned by
/// `the_colour_field_pad_covers_the_widest_base_view_stroke`.
pub fn colour_field_pad_cells(gw: usize) -> f32 {
    ((MIN_STROKE_PX + 2.0 * EDGE_FRINGE_PX) * gw as f32 / FIELD_VIEW_PX).max(3.0)
}

/// **The river colour field** for the base map (`WorldGen::river_color_texture`):
/// every river's styled colour and opacity at full coverage, over a band
/// [`colour_field_pad_cells`] wider than the river on each side, at one pixel
/// per cell. `build_color_texture` composites it through the whole style
/// (`render::land_color` and every stage after) into a second texture, and
/// the base view draws RV-2's vector stroke in screen pixels sampling THAT
/// texture: the stroke's shape and antialiasing are the vector's, at the
/// screen's own resolution, and its colour is the style's
/// (`WorldGen::river_view_mesh`). The band is wider than any stroke the base
/// view draws, so the stroke never samples past the river's own colour.
///
/// Opacity only -- no order-1 de-emphasis, which is a question of how the
/// stroke is drawn at a zoom, not of what colour the river is.
///
/// Must never be drawn on screen as it is: it is a colour lookup, far wider
/// than any river, and only the vector stroke decides where river shows.
pub fn rasterize_colour_field(geom: &RiverGeometry, a: &render::TerrainAppearance, w: usize, h: usize) -> render::RiverLayer {
    let mut layer = render::RiverLayer::new(w, h);
    if w == 0 || h == 0 {
        return layer;
    }
    let view = Some((0.0f32, 0.0f32, (w - 1) as f32, (h - 1) as f32));
    let map = RasterMap::grid();
    let pad = 2.0 * colour_field_pad_cells(w);
    // Opaque in both passes (the preset's ink, full alpha), and the preset's
    // opacity applied once to the finished layer below -- drawn with it, the
    // second pass would compound it over the first on every river's own
    // cells.
    let opacity = render::river_style_color(a, [0.0, 0.0, 0.0, 1.0])[3];
    let style = |k: [f32; 4], _am: f32| {
        let c = render::river_style_color(a, k);
        [c[0], c[1], c[2], 1.0]
    };
    let to = |q: (f32, f32)| (map.offset.0 + q.0 * map.scale.0, map.offset.1 + q.1 * map.scale.1);
    // One pixel per cell, as the raster is. An order-1 run's width here is
    // the opening view's de-emphasised one (0.55x) -- a headwater under a
    // cell wide, so the pad, not the width, is what the band's size is.
    //
    // **Two passes.** The padded bands first, then every river again at its
    // own width on top. With one pass a band drawn later in the order -- a
    // headwater running a few cells beside a trunk -- painted over the
    // trunk's own cells, and the base view's trunk took the headwater's
    // colour there: measured 21 levels/channel off the deep-zoom tile's
    // trunk at the same point, which is 1-2 levels when the trunk's own cells
    // are the trunk's (`_rivstyle_probe.gd`, z16).
    for_each_span(geom, a, 1.0, pad, to, style, view, |m| layer.fill_triangles(&m.pts, &m.colors, &m.indices));
    for_each_span(geom, a, 1.0, 0.0, to, style, view, |m| layer.fill_triangles(&m.pts, &m.colors, &m.indices));
    if opacity < 1.0 {
        layer.scale(opacity);
    }
    layer
}

#[godot_api(secondary)]
impl WorldGen {
    /// The Layers panel's Rivers switch: whether the base view draws its
    /// river stroke ([`Self::river_view_mesh`]) and the deep-zoom tiles
    /// rasterize theirs. The tile cache key carries it (`lod_cache_key`); the
    /// caller redraws the overlay and invalidates the tiles
    /// (`viewport_host.gd::set_layer_visible`). The base texture itself
    /// never holds a river, so it is not re-rendered. Must never change any
    /// generated data: it is a view switch.
    #[func]
    fn set_rivers_in_map(&mut self, shown: bool) {
        self.rivers_in_map = shown;
    }

    /// Read-back for [`Self::set_rivers_in_map`], for the Layers checkbox
    /// (`viewport_host.gd::layer_visible`). Reads only.
    #[func]
    fn rivers_in_map(&self) -> bool {
        self.rivers_in_map
    }

    /// **The base view's rivers** (below the deep-zoom switch): RV-2's
    /// stroke in SCREEN pixels, one triangle list, each vertex carrying the
    /// UV of its own ground position in the map texture, for
    /// `map_overlay.gd::_draw_rivers` to draw textured with
    /// [`Self::river_color_texture`]. So the shape, the width, the 1 px floor
    /// and the antialiasing are the vector's at the screen's resolution --
    /// smooth at fit zoom on any grid, which the one-texel-per-cell raster
    /// was not (coordinator, 2026-09-27) -- and the colour is the style's.
    ///
    /// - `scale`, `offset`: grid to screen, `screen = offset + point * scale`
    ///   (`map_overlay.gd`'s own `_point_to_screen` times its crisp `k`).
    /// - `view`: the visible screen rectangle; segments missing it are culled.
    ///
    /// Width and visibility go through [`river_px_width`] at the screen's own
    /// pixels per cell, exactly as a tile's go through it at the tile's.
    /// Vertex colour is white at the seam's alpha (the order-1 de-emphasis);
    /// the preset's opacity is already in the colour texture. Empty (all
    /// three arrays) with the Rivers layer off, before any world, and for a
    /// loaded save. Must never be called mid-generation (the bridge's
    /// wrapper refuses, for the `Gd<T>::bind()` reason `get_rivers()` has).
    #[func]
    fn river_view_mesh(&self, scale: Vector2, offset: Vector2, view: Rect2) -> VarDictionary {
        let mut out_p: Vec<Vector2> = Vec::new();
        let mut out_c: Vec<Color> = Vec::new();
        let mut out_uv: Vec<Vector2> = Vec::new();
        let mut out_i: Vec<i32> = Vec::new();
        let (gw, gh) = (self.gw.max(1) as f32, self.gh.max(1) as f32);
        if let Some(g) = self.river_geometry() {
            let a = self.appearance();
            let ppc = (scale.x * scale.y).abs().sqrt();
            let vr = Some((view.position.x, view.position.y, view.size.x, view.size.y));
            let to = |q: (f32, f32)| (offset.x + q.0 * scale.x, offset.y + q.1 * scale.y);
            let (sx, sy) = (if scale.x != 0.0 { scale.x } else { 1.0 }, if scale.y != 0.0 { scale.y } else { 1.0 });
            for_each_span(&g, &a, ppc, 0.0, to, |_, am| [1.0, 1.0, 1.0, am], vr, |m| {
                let b = out_p.len() as i32;
                for q in &m.pts {
                    out_p.push(Vector2::new(q.0, q.1));
                    // Back to river space, then to the texture's UV: texel
                    // `x` spans `[x, x+1]` of river space.
                    out_uv.push(Vector2::new((q.0 - offset.x) / sx / gw, (q.1 - offset.y) / sy / gh));
                }
                out_c.extend(m.colors.iter().map(|k| Color::from_rgba(k[0], k[1], k[2], k[3])));
                out_i.extend(m.indices.iter().map(|&i| b + i));
            });
        }
        vdict! {
            "points" => &PackedVector2Array::from(out_p.as_slice()),
            "colors" => &PackedColorArray::from(out_c.as_slice()),
            "uvs" => &PackedVector2Array::from(out_uv.as_slice()),
            "indices" => &PackedInt32Array::from(out_i.as_slice()),
        }
    }

    /// The last river colour field's size: `covered` pixels and the
    /// `allocated_bytes` its sparse blocks hold (`render::RiverLayer`). A
    /// probe's memory reading; nothing in the shell reads it.
    #[func]
    fn river_field_stats(&self) -> VarDictionary {
        let (c, b) = self.river_field_stats.get();
        vdict! { "covered" => c as i64, "allocated_bytes" => b as i64 }
    }

    /// The base map with every river drawn at full coverage in its styled
    /// colour ([`rasterize_colour_field`] through `render::land_color` and
    /// every stage after), built by the last `build_color_texture`. The base
    /// view's river stroke is textured with it ([`Self::river_view_mesh`]).
    /// `null` for a loaded save and before any world. Must never be shown
    /// on its own: every pixel near a river carries river colour.
    #[func]
    fn river_color_texture(&self) -> Option<Gd<godot::classes::ImageTexture>> {
        self.river_color_tex.borrow().clone()
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

    fn one_run(pts: Vec<(f32, f32)>, width: f32, rgb: [f32; 3], o1: bool) -> RiverGeometry {
        let n = pts.len();
        let order = if o1 { 1 } else { 3 };
        RiverGeometry {
            runs: vec![DrawnRun {
                pts,
                widths: vec![width; n],
                colors: vec![[rgb[0], rgb[1], rgb[2], 1.0]; n],
                orders: vec![order; n],
                discharge: vec![f32::NAN; n],
                pieces: vec![(0, n)],
                own_order: order,
            }],
        }
    }

    /// The colour-field pad: three cells, or the width a 1 px stroke and
    /// its two fringes reach at an 800-px fit of the grid, whichever is wider.
    #[test]
    fn the_colour_field_pad_covers_the_widest_base_view_stroke() {
        assert_eq!(colour_field_pad_cells(512), 3.0);
        assert_eq!(colour_field_pad_cells(800), 3.0);
        assert_eq!(colour_field_pad_cells(1600), 6.0);
        assert_eq!(colour_field_pad_cells(8000), 30.0);
    }

    /// The colour field is the river's styled colour at the preset's opacity
    /// over a band `pad` cells wider each side than the river, and it ignores
    /// the order-1 de-emphasis (a zoom question, not a colour one).
    #[test]
    fn the_colour_field_is_the_styled_river_over_a_padded_band() {
        let g = one_run((0..=100).map(|i| (2.5 + i as f32 * 0.25, 12.5)).collect(), 1.0, [0.2, 0.4, 0.8], true);
        let a = render::TerrainAppearance { river_opacity: 0.5, river_ink: 1.0, river_ink_r: 255.0, river_ink_g: 0.0, river_ink_b: 0.0, ..render::TerrainAppearance::default() };
        let l = rasterize_colour_field(&g, &a, 32, 32);
        // pad 3 cells: an order-1 run of width 1 cell at one px per cell is
        // floored at 1 px (0.55 below it), so the band is 1 + 2*3 = 7 cells:
        // rows 9..=15 solid (centre row 12), alpha = the opacity.
        for y in 9..=15 {
            let p = l.at(12, y).expect("inside the band");
            assert!((p[3] - 0.5).abs() < 1e-6, "row {y}: opacity, no order-1 fade: {p:?}");
            assert!((p[0] - 127.5).abs() < 1e-3 && p[1].abs() < 1e-3, "row {y}: the ink, premultiplied: {p:?}");
        }
        assert!(l.at(12, 17).is_none() && l.at(12, 7).is_none(), "nothing beyond band + fringe");
        // A second, lower-order run drawn LATER beside the first: its band
        // covers the first river's cells, but the first river's own cells
        // keep the first river's colour (the second pass).
        let mut g2 = one_run((0..=100).map(|i| (2.5 + i as f32 * 0.25, 12.5)).collect(), 1.0, [0.2, 0.4, 0.8], false);
        g2.runs.push(one_run((0..=100).map(|i| (2.5 + i as f32 * 0.25, 15.5)).collect(), 1.0, [0.9, 0.9, 0.1], false).runs.remove(0));
        let a2 = render::TerrainAppearance::default();
        let l2 = rasterize_colour_field(&g2, &a2, 32, 32);
        let own = l2.at(12, 12).unwrap();
        assert!((own[2] - 0.8 * 255.0).abs() < 1e-3 && (own[0] - 0.2 * 255.0).abs() < 1e-3, "the first river's own cell is its own colour: {own:?}");
        // Row 14 is 2 cells from the first river (past its 0.5 + 1 px
        // reach) and inside the second's band.
        let between = l2.at(12, 14).unwrap();
        assert!((between[0] - 0.9 * 255.0).abs() < 1e-3, "the band between them is the later run's: {between:?}");
        // The drawn stroke is far narrower: the raster at one px per cell
        // does not reach row 9.
        let r = rasterize(&g, &a, 32, 32, RasterMap::grid());
        assert!(r.at(12, 9).is_none());
    }

    /// The seam, at today's rule: RV-2's width on the ground, the preset's
    /// multiplier, the 1 px floor, and the order-1 de-emphasis by own order.
    #[test]
    fn the_width_seam_is_rv2s_rule() {
        let a = render::TerrainAppearance::default();
        let p = RiverPoint { width_cells: 2.0, order: 3, own_order: 3, discharge: f32::NAN };
        assert_eq!(river_px_width(p, 4.0, &a), Some((8.0, 1.0)));
        assert_eq!(river_px_width(RiverPoint { width_cells: 0.1, ..p }, 1.0, &a), Some((1.0, 1.0)), "floored at one pixel");
        let a2 = render::TerrainAppearance { river_width: 1.5, ..a.clone() };
        assert_eq!(river_px_width(p, 4.0, &a2), Some((12.0, 1.0)));
        let o1 = RiverPoint { own_order: 1, order: 1, ..p };
        assert_eq!(river_px_width(o1, 1.0, &a), Some((1.1, 0.4)));
    }

    /// Two triangles sharing a diagonal fill a half-alpha square: every pixel
    /// is filled exactly once -- 0.5, never the 0.75 a double fill gives.
    #[test]
    fn a_shared_edge_is_filled_once() {
        let mut l = render::RiverLayer::new(12, 12);
        let pts = [(1.0f32, 1.0f32), (9.0, 1.0), (9.0, 9.0), (1.0, 9.0)];
        l.fill_triangles(&pts, &[[1.0, 0.0, 0.0, 0.5]; 4], &[0, 1, 2, 0, 2, 3]);
        let mut n = 0;
        for y in 0..12 {
            for x in 0..12 {
                if let Some(p) = l.at(x, y) {
                    assert_eq!(p[3], 0.5, "({x},{y}) filled twice or partly: {p:?}");
                    assert_eq!(p[0], 127.5, "premultiplied red at ({x},{y})");
                    n += 1;
                }
            }
        }
        // A closed 8x8 square of sample points holds 9x9 minus the two
        // edges the top-left rule gives away: 8x8 = 64.
        assert_eq!(n, 64, "square coverage");
        assert_eq!(l.covered(), 64);
        // Untouched pixels are None, not a transparent colour.
        assert!(l.at(0, 0).is_none() && l.at(10, 10).is_none());
    }

    /// The screen texture's own mapping: a trunk-coloured 3-cell stroke along
    /// cell row 10's centre is solid on rows 9..=11, half-alpha fringe one
    /// pixel beyond, and nothing past that.
    #[test]
    fn the_grid_raster_draws_the_stroke_at_its_width_and_colour() {
        let g = one_run((0..=72).map(|i| (2.5 + i as f32 * 0.25, 10.5)).collect(), 3.0, [22.0 / 255.0, 36.0 / 255.0, 80.0 / 255.0], false);
        let l = rasterize(&g, &render::TerrainAppearance::default(), 32, 24, RasterMap::grid());
        for y in [9, 10, 11] {
            let p = l.at(10, y).expect("solid row");
            assert_eq!(p[3], 1.0, "row {y}");
            assert!((p[0] - 22.0).abs() < 1e-3 && (p[1] - 36.0).abs() < 1e-3 && (p[2] - 80.0).abs() < 1e-3, "row {y}: {p:?}");
        }
        for y in [8, 12] {
            let a = l.at(10, y).expect("fringe row")[3];
            assert!((a - 0.5).abs() < 1e-4, "fringe alpha at row {y}: {a}");
        }
        assert!(l.at(10, 7).is_none() && l.at(10, 13).is_none(), "nothing beyond the fringe");
        assert!(l.at(1, 10).is_none(), "nothing before the stroke starts");
    }

    /// Every river style key reaches the raster: width grows the covered
    /// area, ink and opacity change the colour and alpha -- each against a
    /// literal expectation, not against the constant itself.
    #[test]
    fn every_river_style_key_moves_the_raster() {
        let g = one_run((0..=72).map(|i| (2.5 + i as f32 * 0.25, 10.5)).collect(), 3.0, [22.0 / 255.0, 36.0 / 255.0, 80.0 / 255.0], false);
        let a = render::TerrainAppearance::default();
        let base = rasterize(&g, &a, 32, 24, RasterMap::grid());
        let wide = rasterize(&g, &render::TerrainAppearance { river_width: 2.0, ..a.clone() }, 32, 24, RasterMap::grid());
        // 3 cells -> 6 cells: solid rows 8..=12 instead of 9..=11.
        assert_eq!(wide.at(10, 8).unwrap()[3], 1.0);
        assert!(base.at(10, 8).unwrap()[3] < 1.0);
        assert!(wide.covered() > base.covered());
        let white = render::TerrainAppearance { river_ink: 1.0, river_ink_r: 255.0, river_ink_g: 255.0, river_ink_b: 255.0, ..a.clone() };
        let p = rasterize(&g, &white, 32, 24, RasterMap::grid()).at(10, 10).unwrap();
        assert!(p.iter().take(3).all(|&c| (c - 255.0).abs() < 1e-3), "full ink is the ink: {p:?}");
        let half = render::TerrainAppearance { river_ink: 0.5, river_ink_r: 222.0, river_ink_g: 236.0, river_ink_b: 80.0, ..a.clone() };
        let p = rasterize(&g, &half, 32, 24, RasterMap::grid()).at(10, 10).unwrap();
        assert!((p[0] - 122.0).abs() < 1e-3 && (p[1] - 136.0).abs() < 1e-3 && (p[2] - 80.0).abs() < 1e-3, "half-way to the ink: {p:?}");
        let faint = render::TerrainAppearance { river_opacity: 0.25, ..a.clone() };
        let p = rasterize(&g, &faint, 32, 24, RasterMap::grid()).at(10, 10).unwrap();
        assert!((p[3] - 0.25).abs() < 1e-6 && (p[2] - 20.0).abs() < 1e-3, "quarter alpha, premultiplied: {p:?}");
        // The default styling is the palette colour untouched.
        let c = [0.1f32, 0.2, 0.3, 1.0];
        assert_eq!(render::river_style_color(&a, c), c);
    }

    /// The order-1 de-emphasis, as the overlay drew it: 0.55 width / 0.4
    /// alpha at the opening view, gone by 8x, linear between.
    #[test]
    fn the_order_one_deemphasis_fades_by_eight_times() {
        assert_eq!(o1_deemphasis(1.0), (0.55, 0.4));
        assert_eq!(o1_deemphasis(0.5), (0.55, 0.4), "below one pixel per cell is the opening view");
        assert_eq!(o1_deemphasis(8.0), (1.0, 1.0));
        let (w, a) = o1_deemphasis(4.5);
        assert!((w - 0.775).abs() < 1e-6 && (a - 0.7).abs() < 1e-6, "{w} {a}");
        // And the rasterizer applies it: an order-1 run at one pixel per cell
        // draws at 0.4 alpha.
        let g = one_run((0..=72).map(|i| (2.5 + i as f32 * 0.25, 10.5)).collect(), 3.0, [0.5, 0.5, 0.5], true);
        let p = rasterize(&g, &render::TerrainAppearance::default(), 32, 24, RasterMap::grid()).at(10, 10).unwrap();
        assert!((p[3] - 0.4).abs() < 1e-6, "{p:?}");
    }

    /// A tile pixel sits at SAMPLE coordinate `bx + x * cx`, where a cell's
    /// centre is its integer index -- river space puts it at `x + 0.5`. A
    /// vertical stroke on the centre of cell 6 must therefore be centred on
    /// the tile column whose sample is 6.0, symmetric about it.
    #[test]
    fn a_tile_places_a_stroke_at_its_sample_coordinate() {
        let g = one_run((0..=80).map(|i| (6.5, 1.0 + i as f32 * 0.25)).collect(), 0.5, [0.2, 0.4, 0.8], false);
        let (bx, by, cx, cy) = (4.0, 2.0, 0.25, 0.25);
        let l = rasterize(&g, &render::TerrainAppearance::default(), 33, 33, RasterMap::tile(bx, by, cx, cy));
        let col = ((6.0 - bx) / cx) as usize; // 8
        let a = |x: usize| l.at(x, 16).map_or(0.0, |p| p[3]);
        assert_eq!(a(col), 1.0, "the centre column is solid");
        assert!((a(col - 1) - a(col + 1)).abs() < 1e-6, "symmetric about sample 6.0: {} vs {}", a(col - 1), a(col + 1));
        assert!(a(col - 3) == 0.0 && a(col + 3) == 0.0, "half a cell is 2 px here: nothing 3 px out");
        // The row mapping, the same way: a horizontal stroke on the centre of
        // cell row 5 sits on tile row (5 - by) / cy = 12.
        let g = one_run((0..=80).map(|i| (3.0 + i as f32 * 0.25, 5.5)).collect(), 0.5, [0.2, 0.4, 0.8], false);
        let l = rasterize(&g, &render::TerrainAppearance::default(), 33, 33, RasterMap::tile(bx, by, cx, cy));
        let row = ((5.0 - by) / cy) as usize;
        let b = |y: usize| l.at(16, y).map_or(0.0, |p| p[3]);
        assert_eq!(b(row), 1.0);
        assert!((b(row - 1) - b(row + 1)).abs() < 1e-6, "{} vs {}", b(row - 1), b(row + 1));
        // Width density on a non-square tile is the geometric mean of the
        // two axes' pixels per cell: 4 across and 1 down read as 2.
        assert_eq!(RasterMap::tile(0.0, 0.0, 0.25, 1.0).px_per_cell(), 2.0);
        assert_eq!(RasterMap::grid().px_per_cell(), 1.0);
    }

    /// Two deep-zoom tiles either side of a boundary sample the same world
    /// column there (`TileBounds`: pixel 0 of the right tile IS pixel `w-1`
    /// of the left), so a stroke crossing it must fill that column the same
    /// way in both -- including where the stroke's centreline is in the
    /// neighbour and only its width reaches in.
    #[test]
    fn adjacent_tiles_fill_their_shared_column_identically() {
        let g = one_run((0..=80).map(|i| (2.0 + i as f32 * 0.2, 5.0 + i as f32 * 0.1)).collect(), 1.2, [0.2, 0.4, 0.8], false);
        let a = render::TerrainAppearance::default();
        let (w, h) = (33usize, 41usize);
        // Left tile spans x 4..12, right 12..20, both y 2..12, at 0.25 cells per pixel.
        let (cx, cy) = (8.0 / (w - 1) as f64, 10.0 / (h - 1) as f64);
        let left = rasterize(&g, &a, w, h, RasterMap::tile(4.0, 2.0, cx, cy));
        let right = rasterize(&g, &a, w, h, RasterMap::tile(12.0, 2.0, cx, cy));
        let mut drawn = 0;
        for y in 0..h {
            let (p, q) = (left.at(w - 1, y), right.at(0, y));
            assert_eq!(p.is_some(), q.is_some(), "row {y}: {p:?} vs {q:?}");
            if let (Some(p), Some(q)) = (p, q) {
                drawn += 1;
                for k in 0..4 {
                    assert!((p[k] - q[k]).abs() < 1e-3, "row {y} channel {k}: {p:?} vs {q:?}");
                }
            }
        }
        assert!(drawn >= 3, "the stroke must actually cross the shared column ({drawn} rows)");
    }

    /// The owner's floor: a stroke narrower than a pixel is drawn at one.
    #[test]
    fn a_sub_pixel_stroke_is_drawn_one_pixel_wide() {
        // 0.3 cells at 0.9 px/cell is 0.27 px on screen; 4 cells is 3.6 px.
        let a = render::TerrainAppearance::default();
        let pt = |w: f32| RiverPoint { width_cells: w, order: 3, own_order: 3, discharge: f32::NAN };
        let hw: Vec<f32> = [0.3f32, 4.0].iter().map(|&w| river_px_width(pt(w), 0.9, &a).unwrap().0 * 0.5).collect();
        assert_eq!(hw[0], 0.5, "floored at one pixel");
        assert!((hw[1] - 1.8).abs() < 1e-6, "a wide stroke is not touched: {hw:?}");
        // And the floored stroke is a solid pixel with its fringe outside it.
        let m = stroke_mesh(&[(0.0, 0.0), (10.0, 0.0)], &hw[..1].repeat(2), &[[0.0, 0.0, 0.0, 1.0]; 2], None);
        assert_eq!((m.pts[0].1.abs(), m.pts[1].1.abs()), (1.5, 0.5));
    }
}
