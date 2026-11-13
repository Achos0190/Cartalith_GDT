//! Ruling EU item 2 (owner, 2026-11-12): **a planned route joins a way heading
//! toward its destination**, and the stretch it rides is drawn on the way's own
//! line.
//!
//! The owner, verbatim: *"Same for the planner; when a planned route comes close
//! to a way its often more logical to take that path completely to the
//! destination."* Two faults were behind it, both read at the symbols before this
//! module was written:
//!
//! 1. **Corner-cutting.** The router ([`tools::civ_dijkstra_path_guided`]) rides
//!    a way only while its class discount ([`crate::way_class::router_discount`])
//!    beats the shortcut, so a route leaves a way at every bend it can cut
//!    (`PLANNER_AUDIT_2026-10-11.md` Q1: 88 % of a route on the way while it cut
//!    off 42 % of the way's own detour).
//! 2. **A second stroke beside the way.** The discount covers each way cell's
//!    3x3 neighbourhood (`civ_mark_way_neighborhood`), so the cell beside a way is
//!    as cheap as the way, and after smoothing the route is drawn a cell or two
//!    off the way's curve: the "parallel wires" of the ruling. `way_merge`'s own
//!    note on `REUSE_FACTOR` names the same effect for generated roads.
//!
//! **What this does, per leg** ([`civ_dijkstra_path_joined`]), on the router's
//! own grid ([`tools::civ_route_cost_grid`]) and kernel (`road_dijkstra`) -- one
//! router, no second cost model:
//!
//! 1. Route the leg as today; its cost `C*` (summed by [`path_cost`] over the
//!    router's own step rule) is the budget's base.
//! 2. Re-route with the cells a way owns ([`tools::civ_way_owner_on_grid`])
//!    made stickier, strongest first ([`STICK_LEVELS`]). The first stickier path
//!    whose cost **on the unmodified grid** is within `(1 + margin) * C*`, and
//!    which rides at least [`MIN_RIDE_CELLS`] more way cells than the direct path,
//!    replaces it. A way that leads away from the destination is only taken when
//!    the whole detour fits the margin; one beyond it never is. This is a
//!    deliberate replacement for the read-only map's three-tree "join cell"
//!    search: it yields a real least-cost path under one cost model, rides
//!    through junctions onto further ways for free, and needs no new kernel.
//! 3. Draw every run of the chosen path that rides one drawn way for at least
//!    [`MIN_RIDE_CELLS`] cells on **that way's own points** (the splice), so the
//!    route renders on the way's curve instead of beside it. Only the off-way
//!    pieces are smoothed ([`civ_smooth_path`]). A spliced point that fails the
//!    mode's terrain test (a way sample drawn on water: Ruling EU item 1's
//!    residue) keeps the routed cells for that run. A stop within reach of the
//!    first or last ridden way is reached along that way over at most
//!    [`MIN_RIDE_CELLS`] routed cells, then by one straight step, so the line
//!    does not hook beside the road at a town (`extend_stretch`).
//!
//! **What it must never do:** change [`tools::civ_dijkstra_path`] or
//! [`tools::civ_join_dijkstra_segs`] (every golden, every way commit and the
//! village connector call those, and stay byte-identical by control flow);
//! accept a join that costs more than the margin; splice onto a hidden way, a sea
//! lane or a credited edge that is not drawn.

use std::collections::HashSet;

use crate::tools::{
    self, DijkstraPath, JoinedPath, RouteContext, RouteMode, civ_route_chain, civ_route_cost_grid, civ_route_end_cells,
    civ_route_trace, civ_sea_lane_cells, civ_way_owner_on_grid,
};
use crate::{TerrainValid, civ_smooth_path, js_hypot, road_dijkstra, way_merge};

/// How much dearer, in router cost, a joined leg may be than the direct one:
/// `way_merge::DETOUR_TOL` (0.15), so the planner and the generated network agree
/// on how much extra effort riding a way is worth. A labelled judgement (owner
/// question Q2), measured in router cost rather than planned days because the
/// router cost tracks planner speed through `way_class` and planning inside the
/// router would mean running the planner per candidate.
pub const JOIN_MARGIN: f64 = way_merge::DETOUR_TOL;

/// The shortest ride, in routing cells, that counts as joining a way and that is
/// drawn on the way's own line: `2 x way_merge::JUNCTION_SNAP_CELLS` (6). A
/// labelled judgement (owner question Q2): twice the reach at which generation
/// already treats two road ends as one junction, so a route brushing a way's
/// neighbourhood for a cell or two is not called a join.
pub const MIN_RIDE_CELLS: usize = (2.0 * way_merge::JUNCTION_SNAP_CELLS) as usize;

/// Multipliers tried on way-owned cells, strongest first, until one yields a path
/// within the margin. A labelled judgement: `0.25` is the reference's own
/// way discount applied once more (`way_class::REFERENCE_WAY_DISCOUNT`), so a way
/// cell weighs a sixteenth of open ground and a route stays on the way through
/// every bend the margin allows; `0.5` is the half-step tried when that detour is
/// too dear. Any value below `1.0` only ever lengthens the ride; the margin, not
/// these, bounds the cost.
pub const STICK_LEVELS: [f32; 2] = [crate::way_class::REFERENCE_WAY_DISCOUNT, 0.5];

/// How far a run's end may lie from the way it is spliced onto, in full-res
/// cells, before the splice is refused: the 3x3 ownership neighbourhood's reach
/// (one routing cell each side, `1 / sc` full-res cells) plus a cell for the
/// routing cell's drawn centre. A labelled judgement; a run farther off is not
/// riding that way and keeps its routed cells.
const SPLICE_REACH_CELLS: f64 = 2.0;

/// The way join's settings. `drawn_ways` is how many leading entries of
/// `RouteContext::ways` are drawn ways (generated plus hand-drawn, the order
/// [`tools::civ_network_way_refs`] lists them); later entries are credited edges,
/// which discount and count as a ride but are never spliced onto, because no line
/// of theirs is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WayJoin {
    pub margin: f64,
    pub min_ride_cells: usize,
    pub drawn_ways: usize,
}

impl WayJoin {
    /// The shipped settings over a network whose first `drawn_ways` refs are drawn.
    pub fn new(drawn_ways: usize) -> Self {
        WayJoin { margin: JOIN_MARGIN, min_ride_cells: MIN_RIDE_CELLS, drawn_ways }
    }
}

/// One stretch of a joined route drawn on a way's own points: `way` indexes
/// `RouteContext::ways`, `i0..=i1` the route's points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WayRideSpan {
    pub way: usize,
    pub i0: usize,
    pub i1: usize,
}

/// What the join did on one leg. `cost_ratio` is the chosen path's router cost
/// over the direct path's (`1.0` when the direct path was kept); it is `None` when
/// the leg was not routed through the join at all (a water leg, an unreachable
/// leg, a zero-length leg), never a plausible `1.0` standing in for "not
/// measured".
#[derive(Debug, Clone, PartialEq)]
pub struct LegJoin {
    /// A stickier path replaced the direct one.
    pub joined: bool,
    pub cost_ratio: Option<f64>,
    /// Routing cells of the direct and the chosen path that a way owns.
    pub direct_way_cells: usize,
    pub chosen_way_cells: usize,
    /// Runs drawn on a way's own points.
    pub spliced_runs: usize,
}

/// [`civ_join_dijkstra_segs_joined`]'s result: the joined line, a report per leg,
/// and the spliced stretches (indices into `path.pts`).
#[derive(Debug, Clone, PartialEq)]
pub struct JoinedRoute {
    pub path: JoinedPath,
    pub legs: Vec<LegJoin>,
    pub spans: Vec<WayRideSpan>,
}

/// The router's cost of walking `cells` on `cost`: the sum over each step of
/// `len x 0.5 x (c_i + c_j)`, `len` being `sqrt 2` on a diagonal -- `road_dijkstra`'s
/// own step rule, in `f64`. A step across the x seam of a wrapped world counts as
/// the one-cell step it is. Infinite when any cell is impassable.
pub fn path_cost(cost: &[f32], rw: usize, cells: &[usize]) -> f64 {
    let mut acc = 0.0f64;
    for w in cells.windows(2) {
        let (a, b) = (w[0], w[1]);
        let dx = (a % rw).abs_diff(b % rw);
        let dx = if dx > 1 { 1 } else { dx }; // a wrap step is one cell
        let dy = (a / rw).abs_diff(b / rw);
        let len = if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 };
        acc += len * 0.5 * (cost[a] as f64 + cost[b] as f64);
    }
    acc
}

/// The nearest point to `(gx, gy)` on polyline `pts` as `(segment k, t, x, y,
/// distance)`: the point lies on the segment `pts[k] -> pts[k + 1]` at fraction
/// `t`. `None` for fewer than two points.
fn project(pts: &[(f64, f64)], gx: f64, gy: f64) -> Option<(usize, f64, f64, f64, f64)> {
    let mut best: Option<(usize, f64, f64, f64, f64)> = None;
    for k in 0..pts.len().saturating_sub(1) {
        let (ax, ay) = pts[k];
        let (bx, by) = pts[k + 1];
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 1e-12 { (((gx - ax) * dx + (gy - ay) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let (px, py) = (ax + dx * t, ay + dy * t);
        let d = (px - gx).hypot(py - gy);
        if best.is_none_or(|b| d < b.4) {
            best = Some((k, t, px, py, d));
        }
    }
    best
}

/// A way's own points between the projections of `a` and `b`, in travel order
/// (`a` first): the projected start, every way point strictly between, the
/// projected end. `None` when either end lies farther than `reach` from the way,
/// when the stretch crosses one of the way's pen lifts (`brks`) or a jump wider
/// than half the map (the x seam), or when it is degenerate (one point).
fn way_stretch(
    pts: &[(f64, f64)],
    brks: &[usize],
    a: (f64, f64),
    b: (f64, f64),
    reach: f64,
    gw: usize,
) -> Option<Vec<(f64, f64)>> {
    let pa = project(pts, a.0, a.1)?;
    let pb = project(pts, b.0, b.1)?;
    if pa.4 > reach || pb.4 > reach {
        return None;
    }
    let forward = (pa.0, pa.1) <= (pb.0, pb.1);
    let (lo, hi) = if forward { (pa.0, pb.0) } else { (pb.0, pa.0) };
    // Way points `lo + 1 ..= hi` lie strictly inside the stretch.
    if brks.iter().any(|&k| k > lo && k <= hi) {
        return None;
    }
    for k in lo..hi {
        if (pts[k + 1].0 - pts[k].0).abs() > gw as f64 / 2.0 {
            return None;
        }
    }
    let mut out = vec![(pa.2, pa.3)];
    if forward {
        out.extend_from_slice(&pts[pa.0 + 1..=pb.0]);
    } else {
        out.extend(pts[pb.0 + 1..=pa.0].iter().rev().copied());
    }
    out.push((pb.2, pb.3));
    out.dedup();
    (out.len() >= 2).then_some(out)
}

/// `st` (a way stretch, in travel order) extended along its own way so that its
/// last point (`at_end`) or its first point (not `at_end`) is the projection of
/// `stop`, when `stop` lies within `reach` of the way and beyond that end of the
/// stretch in the stretch's own direction (never folding back along the way), and
/// the way gets there about as directly as the straight line does: the length it
/// adds is at most `(1 + JOIN_MARGIN)` times the straight distance from the
/// stretch's end to the stop plus `reach`, so a stop beside a winding road is
/// never reached by riding the road's whole detour. The other end of the stretch
/// stays where it is. The caller extends only over the last few routed cells
/// before a stop (see `civ_dijkstra_path_joined`): this tidies the step onto or
/// off a way and must never re-route a leg past the join's cost test.
fn extend_stretch(
    pts: &[(f64, f64)],
    brks: &[usize],
    st: &[(f64, f64)],
    stop: (f64, f64),
    at_end: bool,
    reach: f64,
    gw: usize,
) -> Option<Vec<(f64, f64)>> {
    let (fixed, far) = if at_end { (st[0], *st.last()?) } else { (*st.last()?, st[0]) };
    let pf = project(pts, fixed.0, fixed.1)?;
    let pe = project(pts, far.0, far.1)?;
    let ps = project(pts, stop.0, stop.1)?;
    let key = |p: &(usize, f64, f64, f64, f64)| p.0 as f64 + p.1;
    let (kf, ke, ks) = (key(&pf), key(&pe), key(&ps));
    // The stop must lie past the stretch's far end, on the far side of `fixed`.
    if ps.4 > reach || (ke - kf) * (ks - ke) <= 0.0 {
        return None;
    }
    let ext = way_stretch(pts, brks, fixed, stop, reach, gw)?;
    let len = |l: &[(f64, f64)]| l.windows(2).map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1)).sum::<f64>();
    let straight = (stop.0 - far.0).hypot(stop.1 - far.1);
    if len(&ext) - len(st) > (straight + reach) * (1.0 + JOIN_MARGIN) {
        return None;
    }
    Some(if at_end { ext } else { ext.into_iter().rev().collect() })
}

/// Whether every point of `st`, and every point along its segments at most half
/// a cell apart, passes `ok`: a way stretch is tested along its line, not only
/// at its sparse control points, so a segment drawn across a lake fails.
fn stretch_is_valid(st: &[(f64, f64)], ok: &dyn Fn(f64, f64) -> bool) -> bool {
    for w in st.windows(2) {
        let (a, b) = (w[0], w[1]);
        let n = ((b.0 - a.0).hypot(b.1 - a.1) / 0.5).ceil().max(1.0) as usize;
        for k in 0..=n {
            let t = k as f64 / n as f64;
            if !ok(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t) {
                return false;
            }
        }
    }
    true
}

/// The km of polyline `pts`, each step measured the short way across the x seam
/// (`min(|dx|, gw - |dx|)`), as [`civ_smooth_path`] counts it.
fn polyline_km(pts: &[(f64, f64)], gw: usize, map_width_km: f64) -> f64 {
    let mut km = 0.0;
    for w in pts.windows(2) {
        let adx = (w[1].0 - w[0].0).abs();
        km += js_hypot(adx.min(gw as f64 - adx), w[1].1 - w[0].1);
    }
    km * map_width_km / gw as f64
}

/// The runs of `owners` (one entry per raw route point; `-1` for none) that ride
/// one way, as `(way, first, last)` point indices, longest first not required:
/// in route order. One point owned by another way or by none, between two points
/// of the same way, is absorbed, so ownership flickering where two ways overlap
/// does not split a ride (the read-only map's hysteresis risk). The route's own
/// first and last points are never inside a run: they are the user's stops.
fn way_runs(owners: &[i32], min_len: usize, drawn_ways: usize) -> Vec<(usize, usize, usize)> {
    let n = owners.len();
    let mut out = Vec::new();
    let mut k = 1usize;
    while k + 1 < n {
        let w = owners[k];
        if w < 0 || w as usize >= drawn_ways {
            k += 1;
            continue;
        }
        let start = k;
        let mut end = k;
        let mut j = k + 1;
        while j + 1 < n {
            if owners[j] == w {
                end = j;
                j += 1;
            } else if j + 2 < n && owners[j + 1] == w {
                // a one-point gap inside the ride
                end = j + 1;
                j += 2;
            } else {
                break;
            }
        }
        if end + 1 - start >= min_len {
            out.push((w as usize, start, end));
        }
        k = end + 1;
    }
    out
}

/// One leg through the join: the leg's line, its report, and its spliced
/// stretches (indices into the returned `pts`). See the module doc for the
/// three steps. A water leg (`RouteMode::Water`: no land way to ride) and an
/// unreachable leg are [`tools::civ_dijkstra_path`]'s own result, unchanged.
pub fn civ_dijkstra_path_joined(
    ctx: &RouteContext,
    sx: f64,
    sy: f64,
    ex: f64,
    ey: f64,
    mode: RouteMode,
    opts: &WayJoin,
) -> (DijkstraPath, LegJoin, Vec<WayRideSpan>) {
    let not_routed = LegJoin { joined: false, cost_ratio: None, direct_way_cells: 0, chosen_way_cells: 0, spliced_runs: 0 };
    if mode == RouteMode::Water {
        return (tools::civ_dijkstra_path(ctx, sx, sy, ex, ey, mode), not_routed, Vec::new());
    }
    let grid = civ_route_cost_grid(ctx, mode, None);
    let (rw, rh) = (grid.rw, grid.rh);
    let cells = civ_route_end_cells(&grid, sx, sy, ex, ey);
    let (si, ti) = (cells.1 * rw + cells.0, cells.3 * rw + cells.2);
    let (_dist, prev) = road_dijkstra(&grid.cost, rw, rh, cells.0, cells.1, ctx.world, None, true);
    let direct_ok = ti != si && prev[ti] >= 0;
    if !direct_ok {
        return (civ_route_trace(ctx, &grid, &prev, cells, (sx, sy, ex, ey), mode), not_routed, Vec::new());
    }
    let owner = civ_way_owner_on_grid(ctx.ways, rw, rh, grid.sc, ctx.gw);
    let direct = civ_route_chain(&prev, si, ti, rw * rh);
    let c_star = path_cost(&grid.cost, rw, &direct);
    let on_way = |p: &[usize]| p.iter().filter(|&&c| owner[c] >= 0).count();
    let direct_on = on_way(&direct);

    // Step 2: the stickiest path within the margin, if any rides further.
    let mut chosen: Option<(Vec<i32>, Vec<usize>, f64)> = None;
    for &f in &STICK_LEVELS {
        let mut sticky = grid.cost.clone();
        for (i, c) in sticky.iter_mut().enumerate() {
            if owner[i] >= 0 && c.is_finite() {
                *c *= f;
            }
        }
        let (_d2, prev2) = road_dijkstra(&sticky, rw, rh, cells.0, cells.1, ctx.world, None, true);
        if prev2[ti] < 0 {
            continue;
        }
        let path = civ_route_chain(&prev2, si, ti, rw * rh);
        let cost = path_cost(&grid.cost, rw, &path);
        if cost <= (1.0 + opts.margin) * c_star && on_way(&path) >= direct_on + opts.min_ride_cells {
            chosen = Some((prev2, path, cost));
            break;
        }
    }
    let joined = chosen.is_some();
    let (prev_c, path_c, cost_c) = chosen.unwrap_or((prev, direct, c_star));
    let mut report = LegJoin {
        joined,
        cost_ratio: Some(if c_star > 0.0 { cost_c / c_star } else { 1.0 }),
        direct_way_cells: direct_on,
        chosen_way_cells: on_way(&path_c),
        spliced_runs: 0,
    };

    // Step 3: the splice. Raw points as `civ_route_trace` builds them: the
    // caller's start, every later routing cell's drawn centre, the caller's end.
    let sc = grid.sc;
    let mut raw: Vec<(f64, f64)> = vec![(sx, sy)];
    let mut owners: Vec<i32> = vec![owner[path_c[0]]];
    for &c in &path_c[1..] {
        raw.push((((c % rw) as f64 + 0.5) / sc, ((c / rw) as f64 + 0.5) / sc));
        owners.push(owner[c]);
    }
    if raw.len() < 2 || raw[raw.len() - 1] != (ex, ey) {
        raw.push((ex, ey));
        owners.push(owner[ti]);
    }
    let lane_cells;
    let valid = match mode {
        RouteMode::Water | RouteMode::Mixed => TerrainValid::Unchecked,
        RouteMode::Land => {
            lane_cells = civ_sea_lane_cells(ctx.ways, ctx.gw, ctx.gh);
            TerrainValid::Land(Some(&lane_cells))
        }
    };
    let reach = SPLICE_REACH_CELLS + 1.0 / sc;
    let checked = !matches!(valid, TerrainValid::Unchecked);
    let ok_at = |x: f64, y: f64| !checked || valid.check(x, y, ctx.gw, ctx.gh, ctx.water_bodies);
    // A raw point whose nearest point on its own way fails the leg's terrain
    // test (a way sample drawn on water) rides no way for the splice, so a ride
    // that passes such a stretch splits around it and only the dry parts are
    // drawn on the way.
    for k in 0..raw.len() {
        let w = owners[k];
        if w >= 0 && (w as usize) < opts.drawn_ways {
            let wr = &ctx.ways[w as usize];
            if project(wr.pts, raw[k].0, raw[k].1).is_some_and(|q| !ok_at(q.2, q.3)) {
                owners[k] = -1;
            }
        }
    }
    let mut splices: Vec<(usize, usize, usize, Vec<(f64, f64)>)> = Vec::new();
    for (w, a, b) in way_runs(&owners, opts.min_ride_cells, opts.drawn_ways) {
        let wr = &ctx.ways[w];
        let Some(st) = way_stretch(wr.pts, wr.brks, raw[a], raw[b], reach, ctx.gw) else { continue };
        if !checked || stretch_is_valid(&st, &ok_at) {
            splices.push((w, a, b, st));
        }
    }
    if splices.is_empty() {
        return (civ_route_trace(ctx, &grid, &prev_c, cells, (sx, sy, ex, ey), mode), report, Vec::new());
    }
    // A stop within reach of the first or last ridden way is reached along that
    // way, then by a straight step: otherwise the few routed cells between the
    // ride's end and the stop are smoothed into a hook beside the road (seen at
    // x16 on seed 912345, `_jpwayriver_probe`, 2026-11-13).
    // Only over at most `min_ride_cells` routed cells before or after the stop:
    // a longer stretch between the stop and the ride is a route the cost test
    // chose, and is kept.
    let last_raw = raw.len() - 1;
    if let Some(first) = splices.first_mut().filter(|f| f.1 <= opts.min_ride_cells) {
        let (w, _, _, st) = first;
        let wr = &ctx.ways[*w];
        if let Some(ext) = extend_stretch(wr.pts, wr.brks, st, raw[0], false, reach, ctx.gw)
            && (!checked || stretch_is_valid(&ext, &ok_at))
        {
            *st = ext;
            first.1 = 1;
        }
    }
    if let Some(last) = splices.last_mut().filter(|l| last_raw - l.2 <= opts.min_ride_cells + 1) {
        let (w, _, _, st) = last;
        let wr = &ctx.ways[*w];
        if let Some(ext) = extend_stretch(wr.pts, wr.brks, st, raw[last_raw], true, reach, ctx.gw)
            && (!checked || stretch_is_valid(&ext, &ok_at))
        {
            *st = ext;
            last.2 = last_raw - 1;
        }
    }
    report.spliced_runs = splices.len();
    let (pts, brks, km, spans) = assemble(ctx, &raw, &splices, &valid);
    (DijkstraPath { pts, brks, km, reachable: true }, report, spans)
}

/// Appends `piece` (with its own pen lifts `piece_brks`) to `pts`/`brks`: the
/// first point is dropped when it repeats the last one emitted, and a pen lift is
/// pushed when the two do not meet. Returns the index in `pts` where `piece[0]`
/// now sits (the shared point's index when it was dropped).
fn push_piece(piece: &[(f64, f64)], piece_brks: &[usize], pts: &mut Vec<(f64, f64)>, brks: &mut Vec<usize>) -> usize {
    if piece.is_empty() {
        return pts.len();
    }
    let drop = usize::from(pts.last() == Some(&piece[0]));
    if !pts.is_empty() && drop == 0 {
        brks.push(pts.len());
    }
    let off = pts.len();
    for &b in piece_brks {
        if b >= drop {
            let bb = off + b - drop;
            if bb > off && bb < off + piece.len() - drop {
                brks.push(bb);
            }
        }
    }
    pts.extend_from_slice(&piece[drop..]);
    off - drop
}

/// Builds the leg's line from the routed raw points and the splices: each
/// off-way piece (from the previous splice's end, or the start, through the raw
/// points, to the next splice's start, or the end) is smoothed by
/// [`civ_smooth_path`] under the leg's terrain test; each splice is the way's own
/// points. Junction points the pieces share are emitted once.
#[allow(clippy::type_complexity)]
fn assemble(
    ctx: &RouteContext,
    raw: &[(f64, f64)],
    splices: &[(usize, usize, usize, Vec<(f64, f64)>)],
    valid: &TerrainValid,
) -> (Vec<(f64, f64)>, Vec<usize>, f64, Vec<WayRideSpan>) {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut brks: Vec<usize> = Vec::new();
    let mut km = 0.0f64;
    let mut spans = Vec::new();
    let checked = !matches!(valid, TerrainValid::Unchecked);
    let ok_at = |x: f64, y: f64| !checked || valid.check(x, y, ctx.gw, ctx.gh, ctx.water_bodies);
    let smooth_piece = |piece: &[(f64, f64)], pts: &mut Vec<(f64, f64)>, brks: &mut Vec<usize>, km: &mut f64| {
        if piece.len() < 2 {
            return;
        }
        // A two-point piece (a stop's straight step onto or off a way) has
        // nothing to smooth; it is drawn as the straight step it is when that
        // step passes the leg's terrain test, so smoothing adds no wobble at it.
        if piece.len() == 2 && stretch_is_valid(piece, &ok_at) {
            *km += polyline_km(piece, ctx.gw, ctx.map_width_km);
            push_piece(piece, &[], pts, brks);
            return;
        }
        match civ_smooth_path(piece, ctx.gw, ctx.gh, ctx.water_bodies, ctx.map_width_km, valid) {
            Some(sm) => {
                *km += sm.km;
                push_piece(&sm.pts, &sm.brks, pts, brks);
            }
            None => {
                *km += polyline_km(piece, ctx.gw, ctx.map_width_km);
                push_piece(piece, &[], pts, brks);
            }
        }
    };
    let mut anchor = raw[0];
    let mut next_raw = 1usize;
    for (w, a, b, st) in splices {
        let mut off: Vec<(f64, f64)> = vec![anchor];
        off.extend_from_slice(&raw[next_raw..*a]);
        off.push(st[0]);
        off.dedup();
        smooth_piece(&off, &mut pts, &mut brks, &mut km);
        km += polyline_km(st, ctx.gw, ctx.map_width_km);
        let i0 = push_piece(st, &[], &mut pts, &mut brks);
        spans.push(WayRideSpan { way: *w, i0, i1: pts.len() - 1 });
        anchor = *st.last().expect("a stretch has at least two points");
        next_raw = b + 1;
    }
    let mut tail: Vec<(f64, f64)> = vec![anchor];
    tail.extend_from_slice(&raw[next_raw..]);
    tail.dedup();
    smooth_piece(&tail, &mut pts, &mut brks, &mut km);
    (pts, brks, km, spans)
}

/// [`tools::civ_join_dijkstra_segs`] with the way join on every leg: the same
/// chaining (the junction point two legs share drawn once, a pen lift where they
/// do not meet), each leg through [`civ_dijkstra_path_joined`]. The Route tool's
/// commit, the planner's re-route and the regenerate resnap call this when the
/// join is on; way commits, drag-guided legs and the village connector never do.
pub fn civ_join_dijkstra_segs_joined(ctx: &RouteContext, wps: &[(f64, f64)], mode: RouteMode, opts: &WayJoin) -> JoinedRoute {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut brks: Vec<usize> = Vec::new();
    let mut km = 0.0f64;
    let mut unreachable_legs = 0usize;
    let mut legs = Vec::new();
    let mut spans = Vec::new();
    for k in 0..wps.len().saturating_sub(1) {
        let (sx, sy) = wps[k];
        let (ex, ey) = wps[k + 1];
        let (sm, rep, leg_spans) = civ_dijkstra_path_joined(ctx, sx, sy, ex, ey, mode, opts);
        legs.push(rep);
        if sm.pts.is_empty() {
            continue;
        }
        if !sm.reachable {
            unreachable_legs += 1;
        }
        let mut seg = sm.pts.clone();
        let mut dropped = 0usize;
        if !pts.is_empty() && *pts.last().unwrap() == seg[0] {
            seg.remove(0);
            dropped = 1;
        } else if !pts.is_empty() {
            brks.push(pts.len());
        }
        let off = pts.len();
        for b in &sm.brks {
            let bb = off + b - dropped;
            if bb > 0 && bb < off + seg.len() {
                brks.push(bb);
            }
        }
        for s in leg_spans {
            spans.push(WayRideSpan { way: s.way, i0: (off + s.i0).saturating_sub(dropped), i1: off + s.i1 - dropped });
        }
        pts.extend_from_slice(&seg);
        km += sm.km;
    }
    JoinedRoute { path: JoinedPath { pts, brks, km, unreachable_legs }, legs, spans }
}

/// The share of `pts` (by index) inside a spliced span: what the planner's
/// commit status reports as "on the way". `None` for an empty route.
pub fn spliced_share(pts_len: usize, spans: &[WayRideSpan]) -> Option<f64> {
    if pts_len == 0 {
        return None;
    }
    let mut on: HashSet<usize> = HashSet::new();
    for s in spans {
        on.extend(s.i0..=s.i1.min(pts_len - 1));
    }
    Some(on.len() as f64 / pts_len as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WayRef, civ_dijkstra_path, civ_join_dijkstra_segs};
    use crate::way_class::WayClass;

    /// Test fixture: a flat, all-land `gw x gh` world (every cell 0.6 over a 0.42
    /// sea, so every land cell costs exactly 1 before way discounts), 4 km a cell.
    /// The sizes and the 4 km are test-local choices, not reference constants.
    fn flat(gw: usize, gh: usize) -> (Vec<f32>, Vec<u8>) {
        (vec![0.6f32; gw * gh], vec![0u8; gw * gh])
    }

    /// Test fixture: a [`RouteContext`] over a flat world with every optional
    /// terrain term off, so a test is about the ways alone.
    fn ctx<'a>(field: &'a [f32], wb: &'a [u8], ways: &'a [WayRef<'a>], gw: usize, gh: usize) -> RouteContext<'a> {
        RouteContext {
            field,
            water_bodies: wb,
            biome: None,
            river_order: None,
            places: &[],
            ways,
            gw,
            gh,
            sea: 0.42,
            corridors: None,
            world: false,
            map_width_km: 4.0 * gw as f64,
            flow: None,
            flow_thresh: 0.0,
        }
    }

    /// Test fixture: a plain road from (10, 20) east to (30, 20), a loop `depth`
    /// cells south between x = 30 and x = 40, then east to (50, 20): a way whose
    /// own line makes a detour the router's discount alone cuts across.
    fn loop_way(depth: f64) -> Vec<(f64, f64)> {
        vec![(10.0, 20.0), (30.0, 20.0), (30.0, 20.0 + depth), (40.0, 20.0 + depth), (40.0, 20.0), (50.0, 20.0)]
    }

    /// Test fixture: a plain road over `pts`.
    fn road(pts: &[(f64, f64)]) -> WayRef<'_> {
        WayRef { pts, brks: &[], sea: false, hidden: false, class: WayClass::Road }
    }

    /// Distance from `p` to polyline `pts`.
    fn dist_to(pts: &[(f64, f64)], p: (f64, f64)) -> f64 {
        project(pts, p.0, p.1).map_or(f64::INFINITY, |b| b.4)
    }

    /// Protects: [`path_cost`] is the router's own step rule -- on the direct path
    /// it equals the distance `road_dijkstra` relaxed to the target (an independent
    /// sum, so this is not the function checked against itself).
    #[test]
    fn path_cost_equals_the_kernels_own_distance() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        let w = loop_way(14.0);
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let grid = civ_route_cost_grid(&c, RouteMode::Land, None);
        let cells = civ_route_end_cells(&grid, 5.0, 20.0, 55.0, 20.0);
        let (dist, prev) = road_dijkstra(&grid.cost, grid.rw, grid.rh, cells.0, cells.1, false, None, true);
        let ti = cells.3 * grid.rw + cells.2;
        let path = civ_route_chain(&prev, cells.1 * grid.rw + cells.0, ti, grid.rw * grid.rh);
        let pc = path_cost(&grid.cost, grid.rw, &path);
        assert!((pc - dist[ti] as f64).abs() < 1e-3, "path_cost {pc} vs dijkstra {}", dist[ti]);
        assert!(pc > 0.0);
    }

    /// Protects the join itself (Ruling EU item 2): a way whose loop the direct
    /// router cuts across is ridden round the loop when that costs at most the
    /// margin more, and the ridden stretch is drawn on the way's own points.
    #[test]
    fn a_route_rides_a_looping_way_within_the_margin_and_is_drawn_on_it() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        // Depth 20 measured 2026-11-13: the direct route cuts the loop, riding it
        // costs 1.12x (inside 0.15, outside a 0.10 margin), so a smaller margin fails.
        let w = loop_way(20.0);
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let direct = civ_dijkstra_path(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land);
        // The direct route cuts across the top of the loop: no point near its floor.
        assert!(direct.pts.iter().all(|p| p.1 < 30.0), "the direct route already rode the loop: {:?}", direct.pts);
        let (p, rep, spans) = civ_dijkstra_path_joined(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land, &WayJoin::new(1));
        assert!(rep.joined, "{rep:?}");
        let r = rep.cost_ratio.expect("a routed leg reports its ratio");
        assert!(r > 1.0 && r <= 1.0 + JOIN_MARGIN, "ratio {r}");
        // The loop's two corners are way points, so a spliced route carries them exactly.
        assert!(p.pts.contains(&(30.0, 40.0)) && p.pts.contains(&(40.0, 40.0)), "{:?}", p.pts);
        assert!(!spans.is_empty());
        for s in &spans {
            for k in s.i0..=s.i1 {
                assert!(dist_to(&w, p.pts[k]) < 1e-9, "spliced point {:?} is off the way", p.pts[k]);
            }
        }
        assert_eq!((p.pts[0], *p.pts.last().unwrap()), ((5.0, 20.0), (55.0, 20.0)), "the user's stops are kept");
        assert!(p.reachable);
    }

    /// Protects the margin: the same loop made deep enough that riding it costs
    /// more than the margin over the direct route is not ridden.
    #[test]
    fn a_loop_dearer_than_the_margin_is_not_ridden() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        // Depth 22, one step past the joined depth 20: riding costs more than 15 %.
        let w = loop_way(22.0);
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, rep, _) = civ_dijkstra_path_joined(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land, &WayJoin::new(1));
        assert!(!rep.joined, "{rep:?}");
        assert_eq!(rep.cost_ratio, Some(1.0));
        assert!(p.pts.iter().all(|q| q.1 < 31.0), "the route rode the loop: {:?}", p.pts);
    }

    /// Protects: a hidden way is never joined or spliced onto, so the leg is
    /// exactly the unjoined router's (byte-identical, by control flow).
    #[test]
    fn a_hidden_way_is_never_joined() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        let w = loop_way(14.0);
        let ways = [WayRef { hidden: true, ..road(&w) }];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, rep, spans) = civ_dijkstra_path_joined(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land, &WayJoin::new(1));
        assert!(!rep.joined && spans.is_empty());
        assert_eq!(p, civ_dijkstra_path(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land));
    }

    /// Protects `drawn_ways`: a credited edge (a ref past the drawn ones) still
    /// counts as a ride but is never spliced onto, because no line of it is drawn.
    #[test]
    fn an_undrawn_edge_is_ridden_but_not_spliced() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        let w = loop_way(20.0);
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, rep, spans) = civ_dijkstra_path_joined(&c, 5.0, 20.0, 55.0, 20.0, RouteMode::Land, &WayJoin::new(0));
        assert!(rep.joined);
        assert!(spans.is_empty() && rep.spliced_runs == 0);
        assert!(!p.pts.contains(&(30.0, 40.0)));
    }

    /// Protects: a water leg has no land way to ride and is the router's own leg.
    #[test]
    fn a_water_leg_is_the_routers_own() {
        let (gw, gh) = (30, 20);
        let field = vec![0.2f32; gw * gh];
        let wb = vec![1u8; gw * gh];
        let c = ctx(&field, &wb, &[], gw, gh);
        let (p, rep, _) = civ_dijkstra_path_joined(&c, 2.0, 2.0, 25.0, 15.0, RouteMode::Water, &WayJoin::new(0));
        assert_eq!(rep.cost_ratio, None, "a water leg is not measured, not reported as 1.0");
        assert_eq!(p, civ_dijkstra_path(&c, 2.0, 2.0, 25.0, 15.0, RouteMode::Water));
    }

    /// Protects the parallel stroke (Ruling EU's "bundle of wires"): a route that
    /// already follows a straight road is drawn on the road's own line, where the
    /// unjoined router draws it on routed cell centres half a cell beside it.
    #[test]
    fn a_route_along_a_road_is_drawn_on_the_road_not_beside_it() {
        let (gw, gh) = (60, 30);
        let (field, wb) = flat(gw, gh);
        // A diagonal road (slope 14/44), so routed cell centres cannot lie on it,
        // in steps shorter than half the map (a longer step reads as the x seam).
        let w: Vec<(f64, f64)> = (0..=4).map(|k| (8.0 + 11.0 * k as f64, 6.0 + 3.5 * k as f64)).collect();
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let direct = civ_dijkstra_path(&c, 4.0, 5.0, 56.0, 21.0, RouteMode::Land);
        let off = direct.pts.iter().filter(|q| q.0 > 10.0 && q.0 < 50.0 && dist_to(&w, **q) > 0.25).count();
        assert!(off > 0, "the fixture must show the unjoined stroke beside the road: {:?}", direct.pts);
        let (p, _, spans) = civ_dijkstra_path_joined(&c, 4.0, 5.0, 56.0, 21.0, RouteMode::Land, &WayJoin::new(1));
        assert_eq!(spans.len(), 1);
        let on = p.pts.iter().filter(|q| q.0 > 10.0 && q.0 < 50.0 && dist_to(&w, **q) > 1e-9).count();
        assert_eq!(on, 0, "{:?}", p.pts);
    }

    /// Protects the stop step: a destination two cells past the end of the road it
    /// rides is reached along the road to its end, then by one straight step, never
    /// by routed cells smoothed into a hook beside the road; the same at the start.
    #[test]
    fn a_stop_beside_the_road_is_reached_along_it() {
        let (gw, gh) = (60, 30);
        let (field, wb) = flat(gw, gh);
        // Steps shorter than half the map: a longer step reads as the x seam.
        let w: Vec<(f64, f64)> = (0..=4).map(|k| (8.0 + 11.0 * k as f64, 12.0)).collect();
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, _, spans) = civ_dijkstra_path_joined(&c, 6.5, 13.5, 53.5, 13.5, RouteMode::Land, &WayJoin::new(1));
        assert!(!spans.is_empty());
        let n = p.pts.len();
        assert_eq!((p.pts[0], p.pts[n - 1]), ((6.5, 13.5), (53.5, 13.5)));
        // Every point lies on the road or on the straight step from a stop to the
        // road's end ((8, 12) and (52, 12)), never beside the road.
        let steps = [vec![(6.5, 13.5), (8.0, 12.0)], vec![(52.0, 12.0), (53.5, 13.5)]];
        for q in &p.pts {
            let off = dist_to(&w, *q).min(dist_to(&steps[0], *q)).min(dist_to(&steps[1], *q));
            assert!(off < 1e-9, "{q:?} is off the road: {:?}", p.pts);
        }
        assert!(p.pts.contains(&(8.0, 12.0)) && p.pts.contains(&(52.0, 12.0)), "{:?}", p.pts);
    }

    /// Protects the stop step where the ride is cut short: a highway crossing the
    /// road just before the stop owns the cells there (it has the higher planner
    /// priority), so the road's run ends before the crossing; the stop, beside the
    /// road two cells past it, is still reached along the road's own line and one
    /// straight step, where the routed cells through the crossing would draw a
    /// hook half a cell beside the road.
    #[test]
    fn a_ride_cut_short_by_a_crossing_still_reaches_the_stop_along_the_road() {
        let (gw, gh) = (60, 30);
        let (field, wb) = flat(gw, gh);
        // Steps shorter than half the map: a longer step reads as the x seam.
        let w: Vec<(f64, f64)> = (0..=4).map(|k| (8.0 + 11.0 * k as f64, 12.0)).collect();
        let hw = [(46.0, 2.0), (46.0, 12.0), (46.0, 22.0)];
        let ways = [road(&w), WayRef { pts: &hw, brks: &[], sea: false, hidden: false, class: WayClass::Highway }];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, _, spans) = civ_dijkstra_path_joined(&c, 6.5, 13.5, 48.5, 13.5, RouteMode::Land, &WayJoin::new(2));
        assert!(!spans.is_empty());
        let steps = [vec![(6.5, 13.5), (8.0, 12.0)], vec![(48.5, 12.0), (48.5, 13.5)]];
        for q in &p.pts {
            let off = dist_to(&w, *q).min(dist_to(&steps[0], *q)).min(dist_to(&steps[1], *q));
            assert!(off < 1e-9, "{q:?} is off the road: {:?}", p.pts);
        }
    }

    /// Protects the terrain check on a splice: a way drawn across a lake (Ruling EU
    /// item 1's residue, a way sample on water) is not spliced on a land leg where
    /// it crosses the water; the routed cells are kept, which stay on land.
    #[test]
    fn a_way_drawn_over_water_is_not_spliced_on_land() {
        let (gw, gh) = (60, 30);
        let (field, mut wb) = flat(gw, gh);
        for x in 25..35 {
            wb[12 * gw + x] = 2;
        }
        // Steps shorter than half the map: a longer step reads as the x seam.
        let w: Vec<(f64, f64)> = (0..=4).map(|k| (8.0 + 11.0 * k as f64, 12.0)).collect();
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let (p, _, spans) = civ_dijkstra_path_joined(&c, 4.0, 12.0, 56.0, 12.0, RouteMode::Land, &WayJoin::new(1));
        assert!(!spans.is_empty(), "the dry stretches either side of the lake are still spliced");
        for s in &spans {
            for k in s.i0..=s.i1 {
                let (x, y) = p.pts[k];
                assert_eq!(wb[(y.round() as usize) * gw + x.round() as usize], 0, "spliced point {:?} on water", p.pts[k]);
            }
        }
    }

    /// Protects the run finder's hysteresis and limits with literal inputs: one
    /// point of another owner inside a ride is absorbed, two end it; a run shorter
    /// than the minimum is dropped; the route's own first and last points are never
    /// inside a run; an owner at or past `drawn_ways` is not a run.
    #[test]
    fn way_runs_absorb_one_point_gaps_and_respect_the_minimum() {
        let o = [0, 0, 0, 0, 1, 0, 0, 0, 0];
        assert_eq!(way_runs(&o, 6, 2), vec![(0, 1, 7)]);
        let o = [0, 0, 0, 1, 1, 0, 0, 0, 0];
        assert_eq!(way_runs(&o, 2, 2), vec![(0, 1, 2), (1, 3, 4), (0, 5, 7)]);
        assert_eq!(way_runs(&o, 3, 2), vec![(0, 5, 7)]);
        assert_eq!(way_runs(&o, 4, 2), Vec::<(usize, usize, usize)>::new());
        let o = [-1, 0, 0, 0, -1];
        assert_eq!(way_runs(&o, 3, 1), vec![(0, 1, 3)]);
        assert_eq!(way_runs(&o, 3, 0), Vec::<(usize, usize, usize)>::new());
    }

    /// Protects the joined chain: two legs share their junction point once, the
    /// spans index the joined line, and the total km stays close to the plain join.
    #[test]
    fn joined_legs_chain_like_the_unjoined_join() {
        let (gw, gh) = (60, 30);
        let (field, wb) = flat(gw, gh);
        // Steps shorter than half the map: a longer step reads as the x seam.
        let w: Vec<(f64, f64)> = (0..=4).map(|k| (8.0 + 11.0 * k as f64, 12.0)).collect();
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let wps = [(4.0, 12.0), (30.0, 20.0), (56.0, 12.0)];
        let j = civ_join_dijkstra_segs_joined(&c, &wps, RouteMode::Land, &WayJoin::new(1));
        assert_eq!(j.legs.len(), 2);
        assert!(!j.spans.is_empty(), "both legs ride the road");
        assert_eq!(j.path.unreachable_legs, 0);
        assert_eq!(j.path.pts.iter().filter(|p| **p == (30.0, 20.0)).count(), 1);
        for s in &j.spans {
            assert!(s.i0 <= s.i1 && s.i1 < j.path.pts.len());
            for k in s.i0..=s.i1 {
                assert!(dist_to(&w, j.path.pts[k]) < 1e-9);
            }
        }
        let plain = civ_join_dijkstra_segs(&c, &wps, RouteMode::Land);
        assert!((j.path.km - plain.km).abs() / plain.km < 0.25, "joined {} vs plain {}", j.path.km, plain.km);
    }

    /// Protects determinism: the same request joins to the same line, report and
    /// spans twice over (no iteration order, cache or clock reaches the result).
    #[test]
    fn the_join_is_deterministic() {
        let (gw, gh) = (60, 50);
        let (field, wb) = flat(gw, gh);
        let w = loop_way(20.0);
        let ways = [road(&w)];
        let c = ctx(&field, &wb, &ways, gw, gh);
        let wps = [(5.0, 20.0), (55.0, 20.0)];
        let a = civ_join_dijkstra_segs_joined(&c, &wps, RouteMode::Land, &WayJoin::new(1));
        let b = civ_join_dijkstra_segs_joined(&c, &wps, RouteMode::Land, &WayJoin::new(1));
        assert_eq!(a, b);
    }

}
