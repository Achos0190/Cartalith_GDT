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
    /// One per piece: `(starts on a shore, ends on a shore)` -- whether the
    /// piece opens at a lake outlet's shoreline crossing and whether it closes
    /// at an inlet's (or a mouth's) shoreline crossing. [`extend_shore_ends`]
    /// carries exactly these ends on into the water; nothing else reads them.
    /// A caller that moves an end onto a shore itself ([`coast_end`]) sets the
    /// flag for it. Must never be set for an end that meets another river or
    /// stops inland: those must not be extended.
    pub shore: Vec<(bool, bool)>,
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

    let mut out = StrokePieces { pts: Vec::new(), u: Vec::new(), pieces: Vec::new(), shore: Vec::new() };
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
            // A shoreline crossing opened / closed this piece exactly when
            // one was handed in: `open` is an outlet, `close` an inlet.
            out.shore.push((open.is_some(), close.is_some()));
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

// ---------------------------------------------------------------------------
// River mouths: the stroke carried on into the water it meets
//
// Owner, 2026-09-27: *"When rivers end into the ocean or lake they should be
// drawn a bit longer to make sure they actually end in the ocean/lake. And the
// ocean texture should be drawn above the river graphic."* Before this, a
// stroke ended ON the shoreline (a lake inlet, [`stroke_pieces`]) or at the
// field's sea-level crossing ([`coast_end`]), and `_riverzoom_probe` measured
// 76-100 free ends per world stopping short of the water beside them (median
// 0.31-0.37 cells): with a flat end cap, any end that meets the shore at an
// angle, or a shore drawn a little further out than the cut, leaves a sliver
// of land between the river and the sea.
//
// The fix has two halves. Here, every end that meets water is carried on
// into it, far enough that the stroke's WHOLE end -- its width and its
// antialiasing fringe -- lies on water on every drawing path. And every path
// draws water above the river, so the part carried into the water is never
// seen: the deep-zoom tiles colour a water pixel as water whatever the river
// layer holds (`render::render_biome_tile_rgba_rivers`), and the base view
// discards the stroke on water pixels (`map_overlay.gd::_draw_rivers`, with
// [`WorldGen::river_water_mask`]).
//
// **Where "on water on every path" is.** The paths draw the shoreline at
// different sub-cell positions: the base view draws the zero line of RV-4's
// shore field (`render::shore_field`, bilinear between cell centres, positive
// on every water cell -- `map_shore.gdshader`; or, for a look without it, whole
// nearest-filtered cells, `drawn_water_classification != 0`); a tile draws the
// sea where its field is below sea level (the amplifier never moves a sample
// across it -- `cartalith_terrain::amplify`'s `clamp_toward_sea`), and a lake
// where all four surrounding cell centres are lake, or by the same shore
// field in the band between (the reference look: a marching-squares rule,
// `render::is_lake_pixel`). Every one of them draws water wherever **all four
// cell centres around a point are water**: a convex combination of four
// positive shore-field values, or of four below-sea values, is positive or
// below sea; four lake centres is the tile's "lake outright" case; and the
// point's own cell is one of the four. So that region -- [`REGION_OPEN`] -- is
// where an end is safe on every path, and the one-cell band between it and
// the drawn cells is exactly the sub-cell uncertainty of the shoreline. A
// small lake that has no such region falls back to its cells
// ([`REGION_DRAWN`]) -- which the smooth shoreline draws a little smaller than
// the cells (a one-cell lake as a rounded blob), so there a stroke end can
// stop a fraction of a cell past the water's edge: known, not yet handled
// (RV-4, 2026-09-27).

/// The grid offset of the squares [`REGION_DRAWN`] is made of: whole cells,
/// `[i, i+1)` in river space.
const REGION_DRAWN: f64 = 0.0;
/// The grid offset of the squares [`REGION_OPEN`] is made of: the squares
/// BETWEEN cell centres, `[i + 0.5, i + 1.5)`, each member when all four of
/// its corner cells are water.
const REGION_OPEN: f64 = 0.5;

/// How far along a ray from `p` (river space, unit direction `d`) the ray
/// stays inside a region made of unit squares on the grid offset by `origin`
/// (`square(i, j)`: whether square `[i+origin, i+1+origin) x [j+origin,
/// j+1+origin)` is a member), up to `limit`. `None` when `p` itself is not
/// inside. Exact: membership can change only where the ray crosses a grid
/// line, and it walks exactly those crossings (a grid DDA). A ray through a
/// square's corner steps into the diagonal square too, which can only end the
/// run earlier -- the conservative side.
fn inside_run(p: (f64, f64), d: (f64, f64), limit: f64, origin: f64, square: &impl Fn(i64, i64) -> bool) -> Option<f64> {
    let (x, y) = (p.0 - origin, p.1 - origin);
    let (mut i, mut j) = (x.floor() as i64, y.floor() as i64);
    if !square(i, j) {
        return None;
    }
    // Per axis: which way the ray steps, the ray length per whole square, and
    // the ray length to the first line it crosses. An axis the ray does not
    // move along never crosses a line (`INFINITY`).
    let axis = |pos: f64, cell: i64, dir: f64| -> (i64, f64, f64) {
        if dir > 0.0 {
            (1, 1.0 / dir, ((cell + 1) as f64 - pos) / dir)
        } else if dir < 0.0 {
            (-1, -1.0 / dir, (pos - cell as f64) / -dir)
        } else {
            (0, f64::INFINITY, f64::INFINITY)
        }
    };
    let (si, di, mut ti) = axis(x, i, d.0);
    let (sj, dj, mut tj) = axis(y, j, d.1);
    loop {
        let t = ti.min(tj);
        if t >= limit {
            return Some(limit);
        }
        if ti < tj {
            i += si;
            ti += di;
        } else {
            j += sj;
            tj += dj;
        }
        if !square(i, j) {
            return Some(t);
        }
    }
}

/// The half-length of the widest cap, centred on `p` across the unit normal
/// `n`, that lies wholly inside the region (see [`inside_run`]), up to
/// `limit`; `None` when `p` is not inside. The narrower of the two sides.
fn cap_fit(p: (f64, f64), n: (f64, f64), limit: f64, origin: f64, square: &impl Fn(i64, i64) -> bool) -> Option<f64> {
    let a = inside_run(p, n, limit, origin, square)?;
    let b = inside_run(p, (-n.0, -n.1), limit, origin, square)?;
    Some(a.min(b))
}

/// The march step along an end's direction, in cells. **Labelled judgement**:
/// a sixteenth of the one-cell band the shoreline's sub-cell position varies
/// in. It sets only how finely the least overshoot is found; a coarser step
/// can only carry an end further into water, which every path hides. At the
/// deepest zoom tested (~25 px per cell at z16) one step is ~1.6 px of that
/// hidden overshoot.
pub const REACH_STEP_CELLS: f64 = 1.0 / 16.0;

/// How far one river end must be carried past where it stops to put a cap of
/// a given half-extent wholly on water. Built once per end by [`shore_reach`]
/// (grid space, independent of zoom and preset); read per raster by
/// [`ShoreReach::extra`] with the cap that raster draws. Must never be
/// attached to an end that does not meet water ([`StrokePieces::shore`]).
#[derive(Clone, Debug, Default)]
pub struct ShoreReach {
    /// Unit direction the end is carried in, river space: the stroke's own
    /// last segment, continued.
    pub dir: (f64, f64),
    /// `(distance, widest cap that fits)` pairs, both in cells: the first
    /// distance past the drawn end at which a cap of that half-extent lies
    /// wholly in [`REGION_OPEN`] (water on every path). The fit only ever
    /// rises down the list -- each entry is where the running best improved.
    pub open: Vec<(f32, f32)>,
    /// The same against [`REGION_DRAWN`] (the base view's whole cells): the
    /// fallback for a lake too small to hold any open region.
    pub drawn: Vec<(f32, f32)>,
}

impl ShoreReach {
    /// The distance, in cells, to carry this end past the drawn one so that
    /// a cap of half-extent `h` cells (the raster's half-width plus fringe,
    /// in cells) lies wholly on water: the first distance at which it fits in
    /// the open region; failing that, in the drawn cells; failing both --
    /// a lake too small, or a shore met at a grazing angle within the march
    /// ([`shore_reach`]) -- the distance of the widest fit found, the best
    /// this water allows (a floor the probe counts rather than hides).
    /// `0.0` for an end with no table. Never negative.
    pub fn extra(&self, h: f32) -> f32 {
        let first = |t: &[(f32, f32)]| t.iter().find(|e| e.1 >= h).map(|e| e.0);
        let best = || {
            // The widest fit of either table; the drawn cells' on a tie,
            // since the open region is a subset of them.
            let o = self.open.last().copied();
            let d = self.drawn.last().copied();
            match (o, d) {
                (Some(o), Some(d)) => Some(if o.1 > d.1 { o.0 } else { d.0 }),
                (a, b) => a.or(b).map(|e| e.0),
            }
        };
        first(&self.open).or_else(|| first(&self.drawn)).or_else(best).unwrap_or(0.0).max(0.0)
    }
}

/// Builds the [`ShoreReach`] of one end: marches from `end` along the unit
/// `dir` in [`REACH_STEP_CELLS`], recording at each step the widest cap
/// (across `dir`) that fits in each region, and keeps each step where the
/// running best improves.
///
/// - `limit_h`: the widest cap half-extent any raster will ask about at this
///   end, in cells; the march stops once the open region holds it.
/// - `wet(i, j)`: whether grid cell `(i, j)` is drawn water; out of the grid
///   is not (a stroke is never carried off the map).
///
/// **Where the march stops, and why** (every bound derived, one labelled):
/// - before the centreline has reached a drawn water cell, after `sqrt(2)`
///   cells -- the farthest any point of a cell lies from a neighbouring cell,
///   so an end that has not met water by then was not beside any;
/// - once it has, the moment the centreline leaves drawn water again: past
///   that is the far shore of a lake, and a stroke carried there would draw on
///   land;
/// - after `2 * (limit_h + 1)` cells in all. **Labelled judgement**: a cap of
///   half-extent `H` crossing a straight shore at an incidence `phi` clears
///   it `H * tan(phi)` past the centreline's crossing, plus the one-cell band
///   the shoreline's sub-cell position varies in; the bound admits every
///   incidence up to `tan(phi) = 2` (63 deg). A river meeting the shore more
///   obliquely than that runs along the coast rather than into it, and its
///   end takes the widest fit found ([`ShoreReach::extra`]).
pub fn shore_reach(end: (f64, f64), dir: (f64, f64), limit_h: f64, wet: impl Fn(i64, i64) -> bool) -> ShoreReach {
    let mut r = ShoreReach { dir, ..ShoreReach::default() };
    let n = (-dir.1, dir.0);
    let open_sq = |i: i64, j: i64| wet(i, j) && wet(i + 1, j) && wet(i, j + 1) && wet(i + 1, j + 1);
    let limit_h = limit_h.max(0.0);
    let s_max = 2.0 * (limit_h + 1.0);
    let (mut best_o, mut best_d) = (-1.0f64, -1.0f64);
    let mut entered = false;
    let mut k = 0usize;
    loop {
        let s = k as f64 * REACH_STEP_CELLS;
        if s > s_max {
            break;
        }
        let p = (end.0 + dir.0 * s, end.1 + dir.1 * s);
        let on_water = wet(p.0.floor() as i64, p.1.floor() as i64);
        if on_water {
            entered = true;
        } else if entered || s > std::f64::consts::SQRT_2 {
            break;
        }
        if let Some(h) = cap_fit(p, n, limit_h, REGION_DRAWN, &wet) {
            if h > best_d {
                best_d = h;
                r.drawn.push((s as f32, h as f32));
            }
        }
        if let Some(h) = cap_fit(p, n, limit_h, REGION_OPEN, &open_sq) {
            if h > best_o {
                best_o = h;
                r.open.push((s as f32, h as f32));
            }
            if h >= limit_h {
                break;
            }
        }
        k += 1;
    }
    r
}

/// Carries every shore end of `s` ([`StrokePieces::shore`]) on to where its
/// centreline first stands on water on every path -- the zero-width cap's
/// [`ShoreReach::extra`] -- as one more drawn point, and returns, per piece,
/// `(head, tail)` reaches measured from those new ends, for a raster to carry
/// its own cap the rest of the way ([`for_each_span`]).
///
/// The new point takes the `u` of the end it extends, so it is drawn at the
/// river's width and colour there. A head is carried backwards (a lake
/// outlet's stroke starts inside the lake); a tail forwards (an inlet or a
/// mouth ends inside the water).
///
/// `limit_h(u)`: the widest cap half-extent, in cells, any raster draws at
/// the point with that `u` ([`reach_limit_cells`]). `wet` as in
/// [`shore_reach`]. Pieces keep their count and order; only ends that meet
/// water move, so a confluence and an inland end are untouched. Must never
/// run on pieces whose `shore` flags were not set from real shoreline
/// crossings.
pub fn extend_shore_ends(
    s: &StrokePieces,
    limit_h: impl Fn(f64) -> f64,
    wet: impl Fn(i64, i64) -> bool,
) -> (StrokePieces, Vec<(Option<ShoreReach>, Option<ShoreReach>)>) {
    let mut out = StrokePieces { pts: Vec::new(), u: Vec::new(), pieces: Vec::new(), shore: s.shore.clone() };
    let mut reach = Vec::with_capacity(s.pieces.len());
    let unit = |a: (f64, f64), b: (f64, f64)| -> Option<(f64, f64)> {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt();
        (l > 0.0).then(|| (dx / l, dy / l))
    };
    // One end: its reach table, re-based at the zero-width fit, and the point
    // that fit is at (`None` when the end does not move).
    let end_reach = |p: (f64, f64), dir: (f64, f64), uu: f64| -> (ShoreReach, Option<(f64, f64)>) {
        let mut r = shore_reach(p, dir, limit_h(uu), &wet);
        let s0 = r.extra(0.0) as f64;
        for e in r.open.iter_mut().chain(r.drawn.iter_mut()) {
            e.0 = (e.0 - s0 as f32).max(0.0);
        }
        (r, (s0 > 0.0).then(|| (p.0 + dir.0 * s0, p.1 + dir.1 * s0)))
    };
    // The direction a piece's end points in: from the nearest point before it
    // that is not the same point. A shoreline crossing at `t == 0` lands on
    // the dry point it started from, so a piece can end on a repeated point;
    // its last step then has no direction, and the one before it does.
    // Measured: 5 ends on one world stayed short for exactly this reason.
    let end_dir = |range: &mut dyn Iterator<Item = usize>, end: (f64, f64)| {
        for j in range {
            if let Some(d) = unit(s.pts[j], end) {
                return Some(d);
            }
        }
        None
    };
    for (k, &(a, b)) in s.pieces.iter().enumerate() {
        let (head, tail) = s.shore.get(k).copied().unwrap_or((false, false));
        let start = out.pts.len();
        let mut hr = None;
        // A head on a shore: carried backwards, against the piece's first step.
        if head && b >= a + 2 {
            if let Some(dir) = end_dir(&mut (a + 1..b), s.pts[a]) {
                let (r, q) = end_reach(s.pts[a], dir, s.u[a]);
                if let Some(q) = q {
                    out.pts.push(q);
                    out.u.push(s.u[a]);
                }
                hr = Some(r);
            }
        }
        out.pts.extend_from_slice(&s.pts[a..b]);
        out.u.extend_from_slice(&s.u[a..b]);
        let mut tr = None;
        if tail && b >= a + 2 {
            if let Some(dir) = end_dir(&mut (a..b - 1).rev(), s.pts[b - 1]) {
                let (r, q) = end_reach(s.pts[b - 1], dir, s.u[b - 1]);
                if let Some(q) = q {
                    out.pts.push(q);
                    out.u.push(s.u[b - 1]);
                }
                tr = Some(r);
            }
        }
        out.pieces.push((start, out.pts.len()));
        reach.push((hr, tr));
    }
    (out, reach)
}

/// The widest cap half-extent, in cells, that any raster draws at a river
/// point of full width `width_cells`, on a `gw`-cell-wide map -- the bound
/// [`shore_reach`] marches to. The widest raster is the base view's colour
/// field ([`rasterize_colour_field`]): one pixel per cell, the stroke widened
/// by [`colour_field_pad_cells`] each side, plus the fringe; its width is the
/// seam's largest ([`RIVER_WIDTH_GUARD`] times the width, floored at
/// [`MIN_STROKE_PX`]). Every other raster draws at a higher density, so a
/// narrower cap in cells -- down to the lowest screen density the pad is
/// derived for ([`FIELD_VIEW_PX`]); below that, an end takes the widest fit
/// its table holds.
pub fn reach_limit_cells(width_cells: f64, gw: usize) -> f64 {
    let full = (width_cells * RIVER_WIDTH_GUARD as f64).max(MIN_STROKE_PX as f64);
    0.5 * full + colour_field_pad_cells(gw) as f64 + EDGE_FRINGE_PX as f64
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
    /// Straight RGBA in `0..=1` at each point: the per-Strahler-order palette,
    /// read at the order blended along the course ([`blended_orders`],
    /// [`palette_at`]) since 2026-09-28.
    pub colors: Vec<[f32; 4]>,
    /// Strahler order of the traced cell nearest each point.
    pub orders: Vec<i16>,
    /// `WorldState::flow_discharge` of the traced cell nearest each point
    /// (NaN where the field had no reading -- never a plausible zero).
    pub discharge: Vec<f32>,
    /// `[start, end)` into `pts`, one per drawn piece.
    pub pieces: Vec<(usize, usize)>,
    /// Per piece, `(head, tail)`: how much further each end meeting water is
    /// carried into it at a given raster's cap ([`ShoreReach`], built by
    /// [`extend_shore_ends`]); `None` for an end that meets no water. Shorter
    /// than `pieces` (a test's hand-built run) reads as no reach at all.
    pub reach: Vec<(Option<ShoreReach>, Option<ShoreReach>)>,
    /// The run's highest order excluding the junction cell it ends on, raised
    /// to its continuation group's highest ([`carry_orders`]) --
    /// `get_rivers()`' `own_order`; `1` is a headwater trickle.
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
    let k = px_per_cell * wm * river_width_factor(a);
    Some(((p.width_cells * k).max(MIN_STROKE_PX), am))
}

/// The preset's river width multiplier as [`river_px_width`] applies it,
/// clamped to `0..=`[`RIVER_WIDTH_GUARD`]. One function so the painted river
/// (`river_field::build`, `map_shore.gdshader`'s `river_width`) and the stroke
/// scale by exactly the same number; a second `clamp` written elsewhere is how
/// they would drift.
pub fn river_width_factor(a: &render::TerrainAppearance) -> f32 {
    a.river_width.clamp(0.0, RIVER_WIDTH_GUARD as f64) as f32
}

/// The largest preset width multiplier [`river_px_width`] honours: a guard
/// against a hand-edited look file, far above the `river_width` tunable's own
/// 3.0 ceiling (`render.rs`'s `tunables!`) -- labelled judgement, unchanged
/// since RV-2 (it was the literal `16.0` there). Named since 2026-09-27
/// because [`reach_limit_cells`] must bound the same widest stroke the seam
/// can return; one constant keeps the two from disagreeing.
pub const RIVER_WIDTH_GUARD: f32 = 16.0;

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

// ---------------------------------------------------------------------------
// Colour along one course (OUTSTANDING_WORK.md, "Rivers change colour
// abruptly mid-course", found 2026-09-27 on RV-2's before/after sheets).
//
// Two defects, both in how a river's order became its drawn colour and alpha:
// 1. The per-point palette (`lib.rs::RIVER_ORDER_RGB`) was read at the order
//    of the nearest traced cell, so where a tributary raised the order the
//    colour jumped a whole palette step (26-46 levels a channel) between two
//    render points, inside one cell. `_rivcolour_probe.gd`: 343-438 such
//    jumps per 1024x656 world.
// 2. A run carried on past a land pit onto the head of the next run (a
//    `river_draw_plan` continuation; [`settle_join_widths`]' `k == 0`) is one
//    river, but the next run restarts the D8 order at its head: it began at
//    the headwater colour and, when its own order stayed 1, at the order-1
//    de-emphasis alpha ([`o1_deemphasis`]), so one visible course read as two
//    differently coloured pieces. 100/105/79 continuations per world stepped
//    colour, 132/144/105 stepped alpha.
// The fix blends the colour along the course and carries the order across a
// continuation. The hierarchy stays: colour still only darkens downstream, and
// a tributary still meets its trunk in its own colour.
// ---------------------------------------------------------------------------

/// Cells over which a river's colour blends from one Strahler order to the
/// next, downstream of the cell where its order rises, on a `gw`-cell-wide
/// grid: `gw / 128`, at least [`COLOUR_RAMP_MIN_CELLS`].
///
/// **Labelled judgement.** At the fit view a map `gw` cells wide fills roughly
/// the screen's width, so `gw / 128` is about 1/128 of the view -- 12 px on a
/// 1600 px view, 8 cells at 1024 wide: long enough that the change reads as a
/// blend at fit (one cell, the old step, is ~1.5 px), short beside a river's
/// own length (the median drawn trunk runs hundreds of cells), so each reach
/// still reads in its own order's colour. Scaled with `gw` rather than fixed
/// in cells so the blend looks the same at fit on every grid size.
pub fn colour_ramp_cells(gw: usize) -> f64 {
    (gw as f64 / 128.0).max(COLOUR_RAMP_MIN_CELLS)
}

/// The shortest blend [`colour_ramp_cells`] returns -- labelled judgement: on
/// a small grid (384 wide gives 3 cells) the ramp would again sit inside a
/// couple of screen pixels at fit; 4 cells keeps it visible.
pub const COLOUR_RAMP_MIN_CELLS: f64 = 4.0;

/// Each render point's **blended order**: the mean of the traced per-cell
/// order `po` over the stretch of course `[u - ramp, u]` upstream of the
/// point, where `u` is the point's traced parameter ([`render_params`]:
/// traced point `k` owns `[k - 0.5, k + 0.5)`). A step from order `a` to `b`
/// therefore becomes a straight ramp from `a` to `b` over `ramp` cells
/// downstream of the step. The window is clipped to the run's own points, so
/// the head reads its own order.
///
/// Why trailing, not centred: water upstream of a confluence does not yet
/// carry the tributary, so the blend starts where the order rises and never
/// darkens a reach above it. Why a mean of the step function: it stays within
/// `[min, max]` of the orders it covers and is monotone wherever `po` is (a
/// traced run's Strahler order never falls downstream), so the order hierarchy
/// is kept -- nothing is drawn darker than its own order's colour, and a
/// fully-ramped reach is exactly its order's colour.
///
/// `incoming` is the blended order the course arrives with at this run's
/// head when the run is a continuation ([`carry_orders`]): the part of the
/// window upstream of the head reads that constant instead of being clipped,
/// so the blend runs on across the bridge from the colour the incoming run
/// was drawn in at its end, instead of restarting at this run's own head
/// order. `None` (a true head, or a run nothing is carried onto) clips.
///
/// Must never extrapolate: an empty `po` returns an empty vector (no colour is
/// invented), and `ramp <= 0` returns each point's nearest order unchanged
/// (the pre-2026-09-28 step, kept reachable for a test).
pub fn blended_orders(po: &[i16], u: &[f64], ramp: f64, incoming: Option<f64>) -> Vec<f64> {
    let n = po.len();
    if n == 0 {
        return Vec::new();
    }
    let nearest = |t: f64| po[(t.round().max(0.0) as usize).min(n - 1)] as f64;
    // cum[m] = sum of po[..m]: the integral of the step function over
    // [-0.5, m - 0.5).
    let mut cum = vec![0.0f64; n + 1];
    for k in 0..n {
        cum[k + 1] = cum[k] + po[k] as f64;
    }
    let (lo, hi) = (-0.5f64, n as f64 - 0.5);
    let integral = |t: f64| -> f64 {
        let t = t.clamp(lo, hi);
        let m = ((t + 0.5).floor() as usize).min(n - 1);
        cum[m] + po[m] as f64 * (t + 0.5 - m as f64)
    };
    u.iter()
        .map(|&t| {
            // No blend. `ramp == 0` would also reach `nearest` through the
            // empty window below (so `<=` vs `<` is an equivalent mutant);
            // the branch exists for a negative ramp, where `clamp(0.0, ramp)`
            // would panic.
            if ramp <= 0.0 {
                return nearest(t);
            }
            let (a, b) = ((t - ramp).clamp(lo, hi), t.clamp(lo, hi));
            // The stretch of the window upstream of the head, read at the
            // incoming course's order (continuations only).
            let before = incoming.map_or(0.0, |_| (lo - (t - ramp)).clamp(0.0, ramp));
            let (sum, len) = (integral(b) - integral(a) + incoming.unwrap_or(0.0) * before, (b - a) + before);
            if len <= 1e-9 {
                nearest(t)
            } else {
                sum / len
            }
        })
        .collect()
}

/// Per run, the blended order the course arrives with at its head
/// ([`blended_orders`]' `incoming`): for a run that others are carried onto
/// (`joins[i] = Some((j, 0))`, a continuation), the largest blended order any
/// of them is drawn with at its own last point (`(len - 1)`, the bridge
/// cell); `None` for every other run. Settled along chains of continuations,
/// so a three-run river blends through both bridges. `orders[i]` are run
/// `i`'s traced orders already raised by [`carry_orders`]' floor.
///
/// Must never be read for a confluence (`k > 0`): a tributary's colour does
/// not flow into its trunk's. Must never invent a history: a run nothing is
/// carried onto stays `None`.
pub fn incoming_orders(orders: &[Vec<i16>], joins: &[Option<(usize, usize)>], ramp: f64) -> Vec<Option<f64>> {
    let n = orders.len().min(joins.len());
    let mut inc: Vec<Option<f64>> = vec![None; n];
    // Each pass settles one more link of every chain; `n` passes bound any
    // chain, and a pass that changes nothing ends early.
    for _ in 0..n.max(1) {
        let mut changed = false;
        for i in 0..n {
            let Some((j, 0)) = joins[i] else { continue };
            if j == i || j >= n || orders[i].is_empty() {
                continue;
            }
            let end = (orders[i].len() - 1) as f64;
            let Some(&v) = blended_orders(&orders[i], &[end], ramp, inc[i]).first() else { continue };
            if inc[j].is_none_or(|w| v > w + 1e-12) {
                inc[j] = Some(v);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    inc
}

/// A palette read at a fractional order: order `o` between `k` and `k + 1`
/// is the straight RGB mix of `palette[k - 1]` and `palette[k]`, each channel
/// rounded. Below 1 is order 1; at or past the last entry is the last entry --
/// `lib.rs::river_order_rgb`'s own clamps, so an integral order reads exactly
/// what it did before. Must never index past the palette; an empty palette is
/// a caller bug and panics in debug only through the `debug_assert`, returning
/// black in release rather than a plausible blue.
pub fn palette_at(palette: &[(u8, u8, u8)], o: f64) -> (u8, u8, u8) {
    debug_assert!(!palette.is_empty());
    let Some(&last) = palette.last() else { return (0, 0, 0) };
    let x = (o - 1.0).max(0.0);
    let i = x.floor() as usize;
    if i + 1 >= palette.len() {
        return last;
    }
    let f = x - i as f64;
    let (a, b) = (palette[i], palette[i + 1]);
    let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * f).round().clamp(0.0, 255.0) as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// Carries Strahler order across every continuation: `joins[i] = Some((j, 0))`
/// means run `i` is carried onto the head of run `j` ([`settle_join_widths`]'
/// `k == 0`), which is one river continuing, not a tributary. Returns, per
/// run, `(floor, drawn_own_order)`:
///
/// - `floor`: the order run `j`'s points are raised to at least -- the
///   highest end order of any run carried onto it, itself raised by *its*
///   floor, settled along chains of continuations. So `j` begins in the colour
///   the incoming river ended in, instead of restarting at the headwater's.
///   `end_order[i]` is run `i`'s last traced order (the junction-free one
///   `lib.rs::river_draws` colours its end with); `None` for a run with no
///   points, which carries nothing.
/// - `drawn_own_order`: the highest `own_order` over the
///   whole group of runs linked by continuations. [`river_px_width`] keys the
///   order-1 de-emphasis on it, so a continuation group is de-emphasised all
///   together or not at all -- as a single traced main stem is (its own
///   order-1 headwater arm draws at full alpha because the run's `own_order`
///   is its maximum). Without it an order-1 run carried onto an order-2 one
///   stepped from 0.4 to full alpha at the bridge.
///
/// Confluences (`k > 0`) are not touched: a tributary keeps its own colour and
/// alpha where it meets its trunk -- that step IS the hierarchy. Must never
/// lower an order, and must never change a run's traced orders themselves
/// (`get_rivers()`' `orders` stays the traced cell's).
pub fn carry_orders(end_order: &[Option<i16>], own_order: &[i16], joins: &[Option<(usize, usize)>]) -> Vec<(i16, i16)> {
    let n = end_order.len().min(own_order.len()).min(joins.len());
    let cont = |i: usize| joins[i].filter(|&(j, k)| k == 0 && j != i && j < n).map(|(j, _)| j);
    let mut floor = vec![0i16; n];
    // Settles along chains; `n` passes bound any chain (a cycle cannot raise
    // past the largest end order, so it settles too).
    for _ in 0..n.max(1) {
        let mut changed = false;
        for i in 0..n {
            let (Some(j), Some(e)) = (cont(i), end_order[i]) else { continue };
            let carried = e.max(floor[i]);
            if carried > floor[j] {
                floor[j] = carried;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Union-find over continuation edges: each group takes its largest
    // (floor-raised) own order.
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for i in 0..n {
        if let Some(j) = cont(i) {
            let (a, b) = (root(&mut parent, i), root(&mut parent, j));
            parent[a] = b;
        }
    }
    let mut best = vec![i16::MIN; n];
    for i in 0..n {
        let r = root(&mut parent, i);
        // `own_order[i]` alone: a floor never exceeds its group's largest
        // own order, since every carried end order is some member's last
        // traced order and a run's own order is the max over its points.
        best[r] = best[r].max(own_order[i]);
    }
    (0..n).map(|i| (floor[i], best[root(&mut parent, i)])).collect()
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
/// stroke is only ever drawn between points drawn at this scale. A piece end
/// that meets water ([`DrawnRun::reach`]) gets one more point, carried on
/// until this raster's own cap lies on water (river mouths, 2026-09-27) --
/// the point where this raster's density is known.
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
    sink: impl FnMut(StripMesh),
) {
    for_each_span_in(geom, 0..geom.runs.len(), a, ppc, widen, to_raster, style, view, sink);
}

/// [`for_each_span`] over the runs `runs` only (a sub-range of
/// `geom.runs`, clamped to it), still in draw order. Added for the base
/// view's chunked stroke (`map_overlay.gd`'s layer cache, 2026-09-28): the
/// overlay rebuilds the stroke a few run ranges a frame after a zoom, and
/// consecutive ranges covering `0..len` walk exactly the spans one whole call
/// walks, in the same order -- a run's spans depend on that run alone. Must
/// never reorder runs within the range.
#[allow(clippy::too_many_arguments)]
fn for_each_span_in(
    geom: &RiverGeometry,
    runs: std::ops::Range<usize>,
    a: &render::TerrainAppearance,
    ppc: f32,
    widen: f32,
    to_raster: impl Fn((f32, f32)) -> (f32, f32),
    style: impl Fn([f32; 4], f32) -> [f32; 4],
    view: Option<(f32, f32, f32, f32)>,
    mut sink: impl FnMut(StripMesh),
) {
    let len = geom.runs.len();
    let (lo, hi) = (runs.start.min(len), runs.end.min(len));
    for run in geom.runs.get(lo..hi.max(lo)).unwrap_or(&[]) {
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
        for (pk, &(s0, e0)) in run.pieces.iter().enumerate() {
            let e0 = e0.min(n);
            let (head, tail) = run.reach.get(pk).map_or((None, None), |r| (r.0.as_ref(), r.1.as_ref()));
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
                    let mut sp: Vec<(f32, f32)> = run.pts[s..e].iter().map(|&q| to_raster(q)).collect();
                    // Every entry in `s..e` is `Some` by construction of the
                    // span, so the `flatten` drops nothing.
                    let drawn: Vec<(f32, f32)> = seam[s..e].iter().flatten().copied().collect();
                    let mut hw: Vec<f32> = drawn.iter().map(|&(wpx, _)| (wpx + widen) * 0.5).collect();
                    let mut cc: Vec<[f32; 4]> = run.colors[s..e].iter().zip(&drawn).map(|(k, &(_, am))| style(*k, am)).collect();
                    // **River mouths.** An end that meets water is carried on
                    // until THIS raster's cap -- its half-width plus the fringe,
                    // in cells at this density -- lies wholly on water
                    // ([`ShoreReach::extra`]). Only at the piece's own ends: a
                    // span the seam split mid-piece meets no shore there.
                    let cells = |h_px: f32| if ppc > 0.0 { (h_px + EDGE_FRINGE_PX) / ppc } else { 0.0 };
                    let carried = |from: (f32, f32), r: &ShoreReach, h_px: f32| -> Option<(f32, f32)> {
                        let x = r.extra(cells(h_px));
                        (x > 0.0).then(|| to_raster((from.0 + r.dir.0 as f32 * x, from.1 + r.dir.1 as f32 * x)))
                    };
                    if let Some(q) = head.filter(|_| s == s0).and_then(|r| carried(run.pts[s], r, hw[0])) {
                        sp.insert(0, q);
                        hw.insert(0, hw[0]);
                        cc.insert(0, cc[0]);
                    }
                    let last = hw.len() - 1;
                    if let Some(q) = tail.filter(|_| e == e0).and_then(|r| carried(run.pts[e - 1], r, hw[last])) {
                        sp.push(q);
                        hw.push(hw[last]);
                        cc.push(cc[last]);
                    }
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
        self.river_view_mesh_range(scale, offset, view, 0..usize::MAX)
    }

    /// [`Self::river_view_mesh`] for the runs `first .. first + count` of the
    /// network only (clamped to it; negative arguments read as 0), in draw
    /// order -- one CHUNK of the base view's stroke. `map_overlay.gd`'s layer
    /// cache (2026-09-28, Ruling BP's 16.7 ms bar) rebuilds the stroke after a
    /// zoom a few chunks a frame, each chunk its own canvas item drawn in
    /// chunk order, instead of the whole ~24 ms mesh inside one frame.
    /// Consecutive ranges covering [`Self::river_run_count`] give exactly the
    /// triangles of one whole call, in the same order (each chunk's indices
    /// are its own, from 0). Same refusals and empties as the whole call.
    #[func]
    fn river_view_mesh_runs(&self, scale: Vector2, offset: Vector2, view: Rect2, first: i64, count: i64) -> VarDictionary {
        let lo = first.max(0) as usize;
        let hi = lo.saturating_add(count.max(0) as usize);
        self.river_view_mesh_range(scale, offset, view, lo..hi)
    }

    /// How many runs the base view's stroke has ([`RiverGeometry::runs`]),
    /// the range [`Self::river_view_mesh_runs`] chunks over. 0 with no
    /// network (before any world, a loaded save). Reads only.
    #[func]
    fn river_run_count(&self) -> i64 {
        self.river_geometry().map_or(0, |g| g.runs.len() as i64)
    }
}

/// The base view's stroke over the runs `runs` as flat arrays -- the whole of
/// `WorldGen::river_view_mesh`'s work, kept free of Godot types so a unit test
/// can compare a chunked build with a whole one. `points` in screen pixels,
/// `colors` white at the seam's alpha, `uvs` into the `gw x gh` colour
/// texture, `indices` from 0 into this call's own `points`.
pub struct ViewMesh {
    pub points: Vec<(f32, f32)>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<(f32, f32)>,
    pub indices: Vec<i32>,
}

/// See [`ViewMesh`]. `scale`/`offset`/`view` are `WorldGen::river_view_mesh`'s
/// own arguments, as tuples.
#[allow(clippy::too_many_arguments)]
pub fn view_mesh(
    geom: &RiverGeometry,
    a: &render::TerrainAppearance,
    gw: usize,
    gh: usize,
    scale: (f32, f32),
    offset: (f32, f32),
    view: (f32, f32, f32, f32),
    runs: std::ops::Range<usize>,
) -> ViewMesh {
    let mut out = ViewMesh { points: Vec::new(), colors: Vec::new(), uvs: Vec::new(), indices: Vec::new() };
    let (gw, gh) = (gw.max(1) as f32, gh.max(1) as f32);
    let ppc = (scale.0 * scale.1).abs().sqrt();
    let to = |q: (f32, f32)| (offset.0 + q.0 * scale.0, offset.1 + q.1 * scale.1);
    let (sx, sy) = (if scale.0 != 0.0 { scale.0 } else { 1.0 }, if scale.1 != 0.0 { scale.1 } else { 1.0 });
    for_each_span_in(geom, runs, a, ppc, 0.0, to, |_, am| [1.0, 1.0, 1.0, am], Some(view), |m| {
        let b = out.points.len() as i32;
        for q in &m.pts {
            out.points.push(*q);
            // Back to river space, then to the texture's UV: texel
            // `x` spans `[x, x+1]` of river space.
            out.uvs.push(((q.0 - offset.0) / sx / gw, (q.1 - offset.1) / sy / gh));
        }
        out.colors.extend(m.colors.iter().copied());
        out.indices.extend(m.indices.iter().map(|&i| b + i));
    });
    out
}

impl WorldGen {
    /// The shared body of [`Self::river_view_mesh`] and
    /// [`Self::river_view_mesh_runs`]: [`view_mesh`] into Godot arrays.
    fn river_view_mesh_range(&self, scale: Vector2, offset: Vector2, view: Rect2, runs: std::ops::Range<usize>) -> VarDictionary {
        let m = match self.river_geometry() {
            Some(g) => view_mesh(&g, &self.appearance(), self.gw as usize, self.gh as usize, (scale.x, scale.y), (offset.x, offset.y),
                (view.position.x, view.position.y, view.size.x, view.size.y), runs),
            None => ViewMesh { points: Vec::new(), colors: Vec::new(), uvs: Vec::new(), indices: Vec::new() },
        };
        let p: Vec<Vector2> = m.points.iter().map(|q| Vector2::new(q.0, q.1)).collect();
        let uv: Vec<Vector2> = m.uvs.iter().map(|q| Vector2::new(q.0, q.1)).collect();
        let c: Vec<Color> = m.colors.iter().map(|k| Color::from_rgba(k[0], k[1], k[2], k[3])).collect();
        vdict! {
            "points" => &PackedVector2Array::from(p.as_slice()),
            "colors" => &PackedColorArray::from(c.as_slice()),
            "uvs" => &PackedVector2Array::from(uv.as_slice()),
            "indices" => &PackedInt32Array::from(m.indices.as_slice()),
        }
    }
}

#[godot_api(secondary)]
impl WorldGen {

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

    /// **Water above rivers, in the base view** (owner, 2026-09-27: *"the
    /// ocean texture should be drawn above the river graphic"*): a `gw x gh`
    /// `L8` texture, 255 on every cell the map texture draws as water
    /// (`drawn_water_classification() != 0` -- the classification
    /// `render::cell_color` draws its ocean and lakes from), 0 on land. Built
    /// by the last `build_color_texture` beside [`Self::river_color_texture`].
    ///
    /// `map_overlay.gd::_draw_rivers` hands it to the stroke's shader, which
    /// samples it NEAREST at the stroke's own UV -- the same texel the base
    /// map's nearest-filtered `TextureRect` shows at that screen pixel -- and
    /// discards the stroke there. So the river is hidden exactly on the pixels
    /// drawn as water, and the end carried into the water
    /// ([`extend_shore_ends`]) never shows. `null` whenever the river colour
    /// texture is (a loaded save, before any world). Must never be drawn
    /// itself.
    ///
    /// **Since RV-4 (2026-09-27) the stroke's shader uses this only as the
    /// fallback**: when the map draws its smooth shoreline
    /// ([`Self::shore_field_texture`]) the shader hides the stroke under that
    /// water instead, and this mask stays the classification the probes read
    /// open water from (`_rivstyle_probe.gd` section M, `_shorestep_probe.gd`).
    #[func]
    fn river_water_mask(&self) -> Option<Gd<godot::classes::ImageTexture>> {
        self.river_water_mask_tex.borrow().clone()
    }

    /// **RV-4: the base map's smooth shoreline** -- a `gw x gh` half-float
    /// (`RH`) texture of `render::shore_field`: per cell, the water surface
    /// minus the ground, positive on every cell the map draws as water and
    /// negative on every land cell, so its bilinear interpolation crosses zero
    /// on a sub-cell shoreline (the height field's own coastline at sea level,
    /// a lake's pooled surface on a lake shore). Built by the last
    /// `build_color_texture`, beside the map texture it describes.
    ///
    /// `viewport_host.gd::_apply_shore_field` gives it to the base map's
    /// `map_shore.gdshader`, which redraws only the pixels between a water and
    /// a land cell centre along that contour, antialiased; the river stroke's
    /// shader (`river_under_water.gdshader`) reads the same field so the water
    /// still sits above the river. `null` for a look drawing the reference's
    /// cell coast (`TerrainAppearance::smooth_shores` false) and before any
    /// world. Must never be drawn itself, nor read as a classification.
    #[func]
    fn shore_field_texture(&self) -> Option<Gd<godot::classes::ImageTexture>> {
        self.shore_field_tex.borrow().clone()
    }

    /// **RIM-1: the river as water in the base map's own pixels** -- a
    /// half-float RGBA texture (`river_field::build`), `gw * scale` by
    /// `gh * scale` texels, `scale` 2 or 1 (see `river_field`'s module doc for
    /// the channels): per texel the distance in cells to the nearest drawn
    /// river centreline, that river's half-width, and its order-1 weight.
    /// Built by the last `build_color_texture` beside the shore field it
    /// merges into. `map_shore.gdshader` paints the river from it with the
    /// shore's own antialiased edge (`river_color_texture` supplies the
    /// colour), `viewport_host.gd::_apply_river_paint` hands it over.
    ///
    /// `null` when the look keeps the stroke (`rivers_as_water` or
    /// `smooth_shores` off), for a loaded save, before any world, and for a
    /// grid over `river_field::MAX_TEXELS` -- in every such case the vector
    /// stroke is drawn, as before ([`Self::rivers_painted`]). Must never be
    /// drawn itself.
    #[func]
    fn river_field_texture(&self) -> Option<Gd<godot::classes::ImageTexture>> {
        self.river_field_tex.borrow().clone()
    }

    /// Whether the base map paints the rivers itself (a river field and the
    /// shore field both built), which is the one condition under which
    /// `map_overlay.gd` stops drawing the vector stroke (RIM-6). Hit-testing
    /// (`get_rivers`, `river_catchment`, the picks) never reads this: it is
    /// geometry, not drawing. The Rivers layer switch is not an input -- off,
    /// the shader paints nothing and the stroke is empty either way.
    #[func]
    fn rivers_painted(&self) -> bool {
        self.river_field_tex.borrow().is_some() && self.shore_field_tex.borrow().is_some()
    }

    /// The shader's river uniforms in one read: `{width}`, the preset's river
    /// width multiplier the field was built for (`river_width_factor`).
    /// `{}` when no field. The shader takes the SAME number the build used,
    /// so a width slider rebuilds the field with the colour texture rather
    /// than leaving the two to disagree.
    #[func]
    fn river_paint_params(&self) -> VarDictionary {
        if self.river_field_tex.borrow().is_none() {
            return VarDictionary::new();
        }
        vdict! { "width" => self.river_field_width.get() as f64 }
    }

    /// **Diagnostic, probe-only**: `on = false` forces `rivers_as_water` off in
    /// every appearance this object builds, so the stroke path renders the
    /// same world and a probe can compare the two (coverage, seams, cost). The
    /// next repaint (`generate`, a width or look change) rebuilds the map; this
    /// does not repaint by itself. Nothing in the shell calls it.
    #[func]
    fn debug_set_river_paint(&self, on: bool) {
        self.river_paint_off.set(!on);
    }

    /// The last uncached river-field build, for a probe: `{built, ms, w, h,
    /// scale, segments, bytes}`. `ms` includes fetching the drawn network when
    /// its cache was cold. Diagnostic; nothing in the shell reads it.
    #[func]
    fn river_paint_stats(&self) -> VarDictionary {
        let (ms, w, h, scale, segs) = self.river_field_build.get();
        vdict! { "built" => self.river_field_tex.borrow().is_some(), "ms" => ms, "w" => w as i64, "h" => h as i64,
            "scale" => scale as i64, "segments" => segs as i64, "bytes" => (w * h * crate::river_field::CHANNELS * 2) as i64 }
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
        // Protects: the shore flags `extend_shore_ends` reads -- a mouth's
        // end, and (below) an outlet's start, are marked; nothing else is.
        assert_eq!(s.shore, vec![(false, true)]);
        let lake = |p: (f64, f64)| p.0 < 2.0;
        let tw: Vec<bool> = pts.iter().map(|&p| lake(p)).collect();
        let s = stroke_pieces(&rp, &u, &tw, lake);
        assert_eq!(s.pieces.len(), 1);
        assert!((s.pts[0].0 - 2.0).abs() < 1e-9, "{:?}", s.pts[0]);
        assert_eq!(s.shore, vec![(true, false)]);
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
                reach: Vec::new(),
                own_order: order,
            }],
        }
    }

    /// Five runs of different widths, orders and colours, for the chunked
    /// view mesh: enough runs that a chunking splits them several ways.
    fn five_runs() -> RiverGeometry {
        let mut g = RiverGeometry { runs: Vec::new() };
        for r in 0..5 {
            let y = 4.0 + 6.0 * r as f32;
            let pts: Vec<(f32, f32)> = (0..12).map(|i| (2.0 + i as f32 * 1.5, y + (i as f32 * 0.7).sin())).collect();
            let mut one = one_run(pts, 0.6 + 0.4 * r as f32, [0.1 * r as f32, 0.4, 0.8], r % 2 == 0);
            g.runs.append(&mut one.runs);
        }
        g
    }

    /// Glues chunk meshes back into one: points/colours/uvs appended, each
    /// chunk's indices shifted by the points before it -- what drawing the
    /// chunks as consecutive canvas items amounts to.
    fn glue(parts: &[ViewMesh]) -> ViewMesh {
        let mut out = ViewMesh { points: Vec::new(), colors: Vec::new(), uvs: Vec::new(), indices: Vec::new() };
        for m in parts {
            let b = out.points.len() as i32;
            out.points.extend_from_slice(&m.points);
            out.colors.extend_from_slice(&m.colors);
            out.uvs.extend_from_slice(&m.uvs);
            out.indices.extend(m.indices.iter().map(|&i| b + i));
        }
        out
    }

    // Protects: the overlay's chunked river rebuild (`map_overlay.gd` layer
    // cache) drawing the SAME stroke as the whole call -- every chunking of the
    // runs, glued in order, is bit-identical to `0..len`, including a range
    // running past the end and an empty one.
    #[test]
    fn chunked_view_mesh_is_the_whole_mesh_in_order() {
        let g = five_runs();
        let a = render::TerrainAppearance::default();
        let (scale, offset, view) = ((12.0, 12.0), (3.0, -2.0), (-50.0, -50.0, 1000.0, 1000.0));
        let whole = view_mesh(&g, &a, 64, 48, scale, offset, view, 0..g.runs.len());
        assert!(!whole.indices.is_empty(), "the fixture must draw something");
        for cuts in [vec![0, 5], vec![0, 1, 5], vec![0, 2, 3, 5], vec![0, 1, 2, 3, 4, 5], vec![0, 0, 3, 3, 99]] {
            let parts: Vec<ViewMesh> = cuts.windows(2).map(|w| view_mesh(&g, &a, 64, 48, scale, offset, view, w[0]..w[1])).collect();
            let glued = glue(&parts);
            assert_eq!(glued.points, whole.points, "cuts {cuts:?}");
            assert_eq!(glued.colors, whole.colors, "cuts {cuts:?}");
            assert_eq!(glued.uvs, whole.uvs, "cuts {cuts:?}");
            assert_eq!(glued.indices, whole.indices, "cuts {cuts:?}");
        }
    }

    // Protects: a chunk draws only its own runs -- a middle chunk is not the
    // whole network, and a range wholly past the end draws nothing (the
    // overlay asks for `n*idx/count ..` ranges, which can be empty).
    #[test]
    fn a_chunk_draws_only_its_own_runs() {
        let g = five_runs();
        let a = render::TerrainAppearance::default();
        let (scale, offset, view) = ((12.0, 12.0), (0.0, 0.0), (-50.0, -50.0, 1000.0, 1000.0));
        let whole = view_mesh(&g, &a, 64, 48, scale, offset, view, 0..5);
        let mid = view_mesh(&g, &a, 64, 48, scale, offset, view, 2..3);
        assert!(!mid.points.is_empty() && mid.points.len() < whole.points.len());
        let past = view_mesh(&g, &a, 64, 48, scale, offset, view, 7..9);
        assert!(past.points.is_empty() && past.indices.is_empty());
        assert!(mid.indices.iter().all(|&i| (i as usize) < mid.points.len()), "a chunk's indices are its own, from 0");
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

    // ---- river mouths (2026-09-27) ----------------------------------------

    /// Cell predicate for the mouth tests: water where `f(x, y)`, inside a
    /// 40 x 20 grid; off the grid is dry (as `river_draws`' own is).
    fn cells(f: impl Fn(i64, i64) -> bool) -> impl Fn(i64, i64) -> bool {
        move |x, y| (0..40).contains(&x) && (0..20).contains(&y) && f(x, y)
    }

    /// The region walk is exact: it stops on the grid line where membership
    /// changes, on either grid (cells, or the squares between centres).
    ///
    /// Protects: `inside_run`'s DDA (step, first-line distance, the
    /// open-region offset).
    #[test]
    fn the_region_walk_stops_on_the_changing_grid_line() {
        let wet = cells(|x, _| x >= 10);
        // From x = 12.25 walking -x, the drawn cells end at x = 10.
        assert_eq!(inside_run((12.25, 5.5), (-1.0, 0.0), 50.0, REGION_DRAWN, &wet), Some(2.25));
        // The open region (all four centres wet) ends half a cell further in.
        let open = |i: i64, j: i64| wet(i, j) && wet(i + 1, j) && wet(i, j + 1) && wet(i + 1, j + 1);
        assert_eq!(inside_run((12.25, 5.5), (-1.0, 0.0), 50.0, REGION_OPEN, &open), Some(1.75));
        // Capped at the limit, and `None` from outside.
        assert_eq!(inside_run((12.25, 5.5), (1.0, 0.0), 3.0, REGION_DRAWN, &wet), Some(3.0));
        // The limit is where the walk stops looking, even with dry ground
        // just past it: water x in [10, 15), limit 2 from x = 12.25.
        let strip = cells(|x, _| (10..15).contains(&x));
        assert_eq!(inside_run((12.25, 5.5), (1.0, 0.0), 2.0, REGION_DRAWN, &strip), Some(2.0));
        assert_eq!(inside_run((9.5, 5.5), (1.0, 0.0), 3.0, REGION_DRAWN, &wet), None);
        // Along y, to the grid's own edge (row 20 is off it).
        assert_eq!(inside_run((12.5, 18.5), (0.0, 1.0), 50.0, REGION_DRAWN, &wet), Some(1.5));
    }

    /// Straight into a straight coast: the centreline stands on water on every
    /// path half a cell past the drawn shore (the open region), and a cap
    /// needs no more there. Met at 45 degrees, a cap of half-extent `H` needs
    /// `H * tan(45 deg) = H` more -- plus that half cell over `cos(45 deg)` --
    /// found to within one march step.
    ///
    /// Protects: the march, the cap fit, and the geometry the margin is
    /// derived from (half-width + fringe, and the shoreline band).
    #[test]
    fn a_mouth_is_carried_until_its_whole_cap_is_on_water() {
        let wet = cells(|x, _| x >= 10);
        let r = shore_reach((10.0, 5.5), (1.0, 0.0), 3.0, &wet);
        assert_eq!(r.extra(0.0), 0.5, "centreline on open water half a cell past the shore");
        assert_eq!(r.extra(2.0), 0.5, "a straight-on cap fits as soon as the centreline does");
        let d = std::f64::consts::FRAC_1_SQRT_2;
        let r = shore_reach((10.0, 5.5), (d, d), 3.0, &wet);
        // Corner at P - H n, n = (-d, d): x = 10 + s d - H d >= 10.5.
        let want = |h: f64| 0.5 / d + h;
        for h in [0.0f32, 1.0, 2.0] {
            let got = r.extra(h) as f64;
            let lo = want(h as f64);
            assert!(got >= lo - 1e-6 && got <= lo + REACH_STEP_CELLS + 1e-6, "H {h}: {got} vs {lo}");
        }
    }

    /// A bay: the open region is narrower than the drawn cells, so a wide cap
    /// falls back to the drawn cells, and one wider than both takes the widest
    /// fit found -- the floor, never a guess.
    ///
    /// Protects: `ShoreReach::extra`'s three answers, in order.
    #[test]
    fn a_cap_too_wide_for_open_water_falls_back_to_the_drawn_cells() {
        // Water: x >= 10, rows 3..=7 (y in [3, 8)); centre y = 5.5.
        let wet = cells(|x, y| x >= 10 && (3..8).contains(&y));
        let r = shore_reach((10.0, 5.5), (1.0, 0.0), 4.0, &wet);
        // Open rows: y in [3.5, 7.5) -> a cap of 2.0 fits.
        assert_eq!(r.extra(2.0), 0.5);
        // Drawn rows: y in [3, 8) -> 2.5 fits at once, on the shore.
        assert_eq!(r.extra(2.5), 0.0);
        // Nothing holds 3.0: the widest fit found (drawn, 2.5) is where it goes.
        assert_eq!(r.extra(3.0), 0.0);
        assert_eq!(r.open.last().map(|e| e.1), Some(2.0));
        assert_eq!(r.drawn.last().map(|e| e.1), Some(2.5));
        // Off-centre, near the bay's top row (rows 3..=8, centre y = 7.5): the
        // open region ends at y = 8.5 (its squares need the row BELOW them
        // too), half a cell short of the drawn cells' 9.
        let bay = cells(|x, y| x >= 10 && (3..9).contains(&y));
        let r = shore_reach((10.0, 7.5), (1.0, 0.0), 4.0, &bay);
        assert_eq!(r.open.last().map(|e| e.1), Some(1.0));
        assert_eq!(r.drawn.last().map(|e| e.1), Some(1.5));
    }

    /// A lake two cells across: the march stops where the centreline would
    /// reach its far shore, so no stroke is ever carried onto the land beyond.
    ///
    /// Protects: the march's stop on leaving water after entering it.
    #[test]
    fn a_stroke_is_never_carried_across_a_lake_to_the_far_shore() {
        let wet = cells(|x, _| (10..12).contains(&x));
        let r = shore_reach((10.0, 5.5), (1.0, 0.0), 40.0, &wet);
        let far = r.open.iter().chain(&r.drawn).map(|e| e.0).fold(0.0f32, f32::max);
        assert!(far < 2.0, "never past x = 12: {r:?}");
        assert!(r.extra(30.0) < 2.0, "the floor stays in the lake too");
        // A lake a tenth of a cell deep along the walk, then one dry cell, then
        // a wide water body: the march stops on leaving the first, even though
        // the second is closer than the sqrt(2) allowed for FINDING water.
        let two = cells(|x, y| (x == 10 && (9..12).contains(&y)) || x >= 12);
        let r = shore_reach((10.9, 10.5), (1.0, 0.0), 6.0, &two);
        assert!(r.drawn.iter().chain(&r.open).all(|e| e.0 < 0.2), "{r:?}");
        // An end with no water ahead is not moved at all.
        let r = shore_reach((5.0, 5.5), (-1.0, 0.0), 3.0, &wet);
        assert!(r.open.is_empty() && r.drawn.is_empty() && r.extra(0.0) == 0.0, "{r:?}");
    }

    /// Only ends flagged as shore ends move; each moves along its own last
    /// step (a head backwards), keeps its `u`, and a piece that ends on a
    /// repeated point takes the step before it -- which is what left five
    /// ends on one world short before it was handled.
    ///
    /// Protects: `extend_shore_ends`' flags, directions and `u`.
    #[test]
    fn only_shore_ends_are_carried_and_each_along_its_own_step() {
        let wet = cells(|x, _| x >= 10 || x < 2);
        let s = StrokePieces {
            // Piece 0: 2 -> 10 (a mouth at x = 10, last point repeated).
            // Piece 1: 5 -> 8 (an inland stretch, no shore flags).
            pts: vec![(2.0, 5.5), (6.0, 5.5), (10.0, 5.5), (10.0, 5.5), (5.0, 9.5), (8.0, 9.5)],
            u: vec![0.0, 1.0, 2.0, 2.0, 5.0, 6.0],
            pieces: vec![(0, 4), (4, 6)],
            shore: vec![(true, true), (false, false)],
        };
        let (out, reach) = extend_shore_ends(&s, |_| 3.0, &wet);
        assert_eq!(out.pieces, vec![(0, 6), (6, 8)], "one point each end of piece 0, none on piece 1");
        // Open water behind the head is x < 1.5 (x = 1.5 itself belongs to the
        // square [1.5, 2.5), whose right-hand cell is dry): the first march
        // step past it, 1/16 cell further.
        assert_eq!(out.pts[0], (1.4375, 5.5), "the head goes back into the water behind it");
        assert_eq!(out.pts[5], (10.5, 5.5), "the tail on, past the repeated point");
        assert_eq!((out.u[0], out.u[5]), (0.0, 2.0), "each new point keeps its end's u");
        assert!(reach[0].0.is_some() && reach[0].1.is_some() && reach[1].0.is_none() && reach[1].1.is_none());
        // The tables are measured from the NEW ends: a zero-width cap needs
        // nothing more there.
        assert_eq!(reach[0].1.as_ref().map(|r| r.extra(0.0)), Some(0.0));
        assert_eq!(reach[0].0.as_ref().map(|r| r.extra(0.0)), Some(0.0));
        assert_eq!(&out.pts[6..], &s.pts[4..], "the inland piece is untouched");
    }

    /// The widest cap any raster asks about: the colour field's, at one pixel
    /// per cell -- the widest seam width, the pad either side, the fringe.
    ///
    /// Protects: `reach_limit_cells`' terms.
    #[test]
    fn the_reach_limit_is_the_colour_fields_cap() {
        // 1 cell x 16 = 16 wide; pad 3 (800 cells); fringe 1.
        assert_eq!(reach_limit_cells(1.0, 800), 8.0 + 3.0 + 1.0);
        // A trickle is floored at one pixel: 0.5 + 6 (1600 cells) + 1.
        assert_eq!(reach_limit_cells(0.01, 1600), 0.5 + 6.0 + 1.0);
    }

    /// A raster carries a shore end on by its OWN cap: the grid raster (one
    /// pixel per cell) draws past the end exactly as far as the table says for
    /// a cap of half-width plus fringe, and not at all without a table.
    ///
    /// Protects: `for_each_span`'s use of the reach, at the piece's end.
    #[test]
    fn a_raster_carries_the_end_by_its_own_cap() {
        let a = render::TerrainAppearance::default();
        // Ends at x = 10.0.
        let pts: Vec<(f32, f32)> = (0..=30).map(|i| (2.5 + i as f32 * 0.25, 12.5)).collect();
        let mut g = one_run(pts, 1.0, [0.2, 0.4, 0.8], false);
        let bare = rasterize(&g, &a, 32, 32, RasterMap::grid());
        // Pixel 10 samples river x = 10.5: past the bare end, not drawn.
        assert!(bare.at(10, 12).is_none(), "{:?}", bare.at(10, 12));
        // 1 px wide at 1 px/cell: cap = 0.5 + 1 fringe = 1.5 cells.
        // The table fits 0.75 at 2 cells on and 1.5 at 3: the fringe is what
        // makes the cap 1.5, so the end goes 3 cells on, to x = 13.
        let reach = ShoreReach { dir: (1.0, 0.0), open: vec![(0.0, 0.25), (2.0, 0.75), (3.0, 1.5), (6.0, 4.0)], drawn: Vec::new() };
        g.runs[0].reach = vec![(None, Some(reach))];
        let carried = rasterize(&g, &a, 32, 32, RasterMap::grid());
        let p = carried.at(12, 12).expect("carried three cells on");
        assert!((p[3] - 1.0).abs() < 1e-6, "solid there: {p:?}");
        assert!(carried.at(14, 12).is_none(), "and no further than the cap needs");
    }

    /// A step in traced order becomes a straight ramp of exactly `ramp`
    /// cells, starting at the step and running downstream; upstream of the
    /// step nothing moves, and past the ramp the new order holds exactly.
    ///
    /// Protects: `blended_orders`' trailing window (a centred one would
    /// darken the reach above the confluence), its clip at the head, and its
    /// length.
    #[test]
    fn an_order_step_becomes_a_ramp_downstream_of_it() {
        // Traced orders 1 x 10 then 3 x 20; point k owns [k-0.5, k+0.5), so
        // the step is at t = 9.5.
        let po: Vec<i16> = [vec![1i16; 10], vec![3i16; 20]].concat();
        let u: Vec<f64> = vec![0.0, 5.0, 9.5, 11.5, 13.5, 17.5, 25.0];
        let o = blended_orders(&po, &u, 8.0, None);
        assert_eq!(o[0], 1.0, "the head reads its own order");
        assert_eq!(o[1], 1.0, "upstream of the step is untouched");
        assert_eq!(o[2], 1.0, "the ramp starts AT the step, not before it");
        assert!((o[3] - 1.5).abs() < 1e-12, "2 of 8 cells past: 1 + 2 x 2/8, got {}", o[3]);
        assert!((o[4] - 2.0).abs() < 1e-12, "half way: {}", o[4]);
        assert_eq!(o[5], 3.0, "8 cells past the step the new order holds exactly");
        assert_eq!(o[6], 3.0);
        // Monotone wherever the traced order is.
        let fine: Vec<f64> = (0..300).map(|i| i as f64 * 0.1).collect();
        let b = blended_orders(&po, &fine, 8.0, None);
        assert!(b.windows(2).all(|w| w[1] >= w[0] - 1e-12), "the hierarchy: never lightens downstream");
        // No blend asked for: each point's nearest traced order, the old step.
        assert_eq!(blended_orders(&po, &[9.4, 9.6], 0.0, None), vec![1.0, 3.0]);
        assert!(blended_orders(&[], &[1.0], 8.0, Some(2.0)).is_empty(), "no order, no colour invented");
        // A negative ramp is no blend, not a panic.
        assert_eq!(blended_orders(&po, &[9.4, 9.6], -1.0, Some(2.0)), vec![1.0, 3.0]);
    }

    /// A continuation's blend runs on from the colour the incoming run ended
    /// in: at its head the window reads that incoming order, and it ramps to
    /// the run's own order over `ramp` cells -- no step at the bridge.
    ///
    /// Protects: `blended_orders`' `incoming` history (without it the head
    /// restarts at its own order, the step the probe counted at 19-39
    /// continuations per world).
    #[test]
    fn a_continuation_blends_on_from_the_incoming_colour() {
        let po = vec![3i16; 20];
        // The head, window [-8, 0]: 7.5 cells upstream of the head at 1.5,
        // 0.5 cell of the run's own 3.
        let o = blended_orders(&po, &[0.0, 3.5, 7.5, 12.0], 8.0, Some(1.5));
        assert!((o[0] - (7.5 * 1.5 + 0.5 * 3.0) / 8.0).abs() < 1e-12, "{}", o[0]);
        assert!((o[1] - (4.0 * 1.5 + 4.0 * 3.0) / 8.0).abs() < 1e-12, "{}", o[1]);
        assert_eq!(o[2], 3.0, "a whole ramp past the head, the run's own order");
        assert_eq!(o[3], 3.0);
        // The same run as a true head: clipped, its own order from the start.
        assert_eq!(blended_orders(&po, &[0.0], 8.0, None), vec![3.0]);
    }

    /// The history a continuation arrives with is the blended order the run
    /// carried onto it ends with, settled along a chain; confluences and true
    /// heads get none.
    ///
    /// Protects: `incoming_orders`' `k == 0` test, its end point, and its
    /// settling along a chain.
    #[test]
    fn the_incoming_history_is_the_carried_runs_blended_end() {
        // 0: orders 1 x 4 then 2 x 2 (the rise 2 cells before its end), carried
        // onto 1's head; 1 carried onto 2's head; 3 joins 2 at point 5.
        let orders = vec![vec![1i16, 1, 1, 1, 2, 2], vec![2i16; 3], vec![2i16; 10], vec![1i16; 4]];
        let joins = [Some((1, 0)), Some((2, 0)), None, Some((2, 5))];
        let inc = incoming_orders(&orders, &joins, 4.0);
        // Run 0's end u = 5, window [1, 5]: cells [1, 3.5) at order 1 and
        // [3.5, 5] at order 2, so (2.5 x 1 + 1.5 x 2) / 4.
        let e0 = (2.5 * 1.0 + 1.5 * 2.0) / 4.0;
        assert_eq!(inc[0], None, "a true head has no history");
        assert!((inc[1].unwrap() - e0).abs() < 1e-12, "{:?}", inc[1]);
        // Run 1's end u = 2, window [-2, 2]: 1.5 cells of history at e0 and
        // 2.5 cells of its own 2 -- the chain's second link.
        let e1 = (1.5 * e0 + 2.5 * 2.0) / 4.0;
        assert!((inc[2].unwrap() - e1).abs() < 1e-12, "{:?}", inc[2]);
        assert_eq!(inc[3], None, "a tributary's colour never flows into its trunk's");
        // Two runs carried onto one head: the darker arrival wins, whichever
        // comes first.
        let orders = vec![vec![1i16; 5], vec![3i16; 5], vec![3i16; 5]];
        assert_eq!(incoming_orders(&orders, &[Some((2, 0)), Some((2, 0)), None], 4.0)[2], Some(3.0));
        // A darker tributary joining mid-run (k = 2) gives the trunk's head
        // no history.
        let trib = vec![vec![4i16; 5], vec![1i16; 5]];
        assert_eq!(incoming_orders(&trib, &[Some((1, 2)), None], 4.0), vec![None, None]);
        let rev = vec![vec![3i16; 5], vec![1i16; 5], vec![3i16; 5]];
        assert_eq!(incoming_orders(&rev, &[Some((2, 0)), Some((2, 0)), None], 4.0)[2], Some(3.0));
    }

    /// The palette at integral orders is each entry exactly (the old
    /// per-order colour), at a fractional order the rounded mix of its two
    /// neighbours, and it clamps both ends as `river_order_rgb` did.
    ///
    /// Protects: `palette_at`'s index, mix and clamps.
    #[test]
    fn the_palette_mixes_between_orders_and_clamps_its_ends() {
        let pal = [(100u8, 200u8, 0u8), (0, 100, 200), (10, 20, 30)];
        assert_eq!(palette_at(&pal, 1.0), (100, 200, 0));
        assert_eq!(palette_at(&pal, 2.0), (0, 100, 200));
        assert_eq!(palette_at(&pal, 1.5), (50, 150, 100));
        assert_eq!(palette_at(&pal, 1.25), (75, 175, 50));
        assert_eq!(palette_at(&pal, 0.0), (100, 200, 0), "below order 1 is order 1");
        assert_eq!(palette_at(&pal, 3.0), (10, 20, 30));
        assert_eq!(palette_at(&pal, 9.0), (10, 20, 30), "past the last entry is the last entry");
        // Rounded, not truncated: 0.6 of the way from 0 to 1 is 1.
        assert_eq!(palette_at(&[(0, 0, 0), (1, 1, 1)], 1.6), (1, 1, 1));
        assert_eq!(palette_at(&[(0, 0, 0), (1, 1, 1)], 1.4), (0, 0, 0));
    }

    /// A chain of continuations carries the order it ended in, and every run
    /// linked by continuations shares one de-emphasis order; a confluence
    /// (`k > 0`) carries nothing.
    ///
    /// Protects: `carry_orders`' floor (settled along a chain, never lowered),
    /// its group maximum, and its `k == 0` test.
    #[test]
    fn a_continuation_carries_its_order_and_a_confluence_does_not() {
        // 0 (ends at order 3) -> head of 1 (own 1) -> head of 2 (own 2);
        // 3 (own 1, ends 1) joins 2 at point 4: a tributary.
        // 4 (own 1) -> head of 5 (own 1): an order-1 pair stays order 1.
        let end = [Some(3i16), Some(1), Some(2), Some(1), Some(1), Some(1)];
        let own = [3i16, 1, 2, 1, 1, 1];
        let joins = [Some((1, 0)), Some((2, 0)), None, Some((2, 4)), Some((5, 0)), None];
        let c = carry_orders(&end, &own, &joins);
        assert_eq!(c[0], (0, 3), "nothing carried onto the first run");
        assert_eq!(c[1], (3, 3), "run 1 starts in run 0's end order");
        assert_eq!(c[2], (3, 3), "and passes it on through run 1 (a chain), never lowered to run 1's own 1");
        assert_eq!(c[3], (0, 1), "a tributary keeps its own order and its de-emphasis");
        assert_eq!(c[4], (0, 1));
        assert_eq!(c[5], (1, 1), "an order-1 river carried on stays order 1");
        // A run carried onto itself, or past the slice, is ignored.
        assert_eq!(carry_orders(&[Some(2)], &[2], &[Some((0, 0))]), vec![(0, 2)]);
        assert_eq!(carry_orders(&[Some(2)], &[2], &[Some((7, 0))]), vec![(0, 2)]);
        // A run with no points carries nothing (no invented order).
        assert_eq!(carry_orders(&[None, Some(1)], &[1, 1], &[Some((1, 0)), None]), vec![(0, 1), (0, 1)]);
    }

    /// The blend's length follows the grid (1/128 of its width) with a floor.
    ///
    /// Protects: `colour_ramp_cells`' scale and `COLOUR_RAMP_MIN_CELLS`.
    #[test]
    fn the_colour_ramp_scales_with_the_grid_above_a_floor() {
        assert_eq!(colour_ramp_cells(1024), 8.0);
        assert_eq!(colour_ramp_cells(2048), 16.0);
        assert_eq!(colour_ramp_cells(384), 4.0, "3 cells would be under the floor");
        assert_eq!(colour_ramp_cells(0), 4.0);
    }
}
