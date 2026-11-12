//! Shared trunks for the generated road network (owner request 2026-11-12,
//! `LARGE_ITEM_RULINGS.md` Ruling EU item 1): routes that come close to each
//! other on the way to the same place merge into one way instead of being drawn
//! as a bundle of parallel wires.
//!
//! **What it changes.** Only the cell path of each [`RoadEdge`] and the network's
//! per-cell `usage_count`. The edge *pairs* (which settlements are joined) stay
//! exactly as `civ_hierarchical_network_topology`'s three passes chose them, so
//! the centrality loop, the metropolis pick and every consumer that reads pairs
//! is unchanged by construction. The caller runs this once, after
//! [`crate::civ_iterative_network`] returns, never inside its loop (re-tiering
//! there reads usage order and could flip a near-tie).
//!
//! **Why the bundles happen without it.** Pass 1 of the topology is a Prim tree
//! over a cost grid with no reuse at all, and every pass computes all of its
//! Dijkstra trees up front, so no edge is attracted to an edge laid in the same
//! pass. The only attraction anywhere is pass 2-3's `ROAD_REUSE_K` 0.55 on the
//! *exact* cells pass 1 used; a path one cell off a corridor gets nothing.
//! Consolidation (`civ_consolidate_and_smooth_ways`) then merges only cells that
//! are literally shared, so two corridors one to four routing cells apart are
//! drawn as two strokes.
//!
//! **What it does instead** (Stage A of the plan in this commit's ruling): the
//! edges are re-traced one at a time, busiest places first, over the pass-1 cost
//! grid in which every cell an earlier re-traced edge already uses costs
//! [`REUSE_FACTOR`] as much. That is the trail-reinforcement mechanism of
//! Helbing, Keltsch and Molnár (1997, *Nature* 388:47-50, "Modelling the
//! evolution of human trail systems"): an existing path lowers the cost of
//! walking it, so later walkers join it. The village connector already grows
//! its tracks this way (`tools::civ_connect_village_addons`, the reference's
//! `_CIV_EXISTING_WAY_DISCOUNT`).
//!
//! **What it must never do.** It must never make a road worse to walk than the
//! road it replaces by more than [`DETOUR_TOL`]: a re-trace is kept only when its
//! *undiscounted* pass-1 cost is within that share of the original path's, so a
//! trunk can never drag a route over a ridge or round a long detour. It must
//! never cross water (the search runs over the same grid, where water is
//! infinite). And it is never called when `CivParams::merge_ways` is off, so the
//! reference-parity network and every golden are untouched by control flow.

use crate::{
    HierarchicalNetworkResult, NamedSettlement, RoadEdge, SettlementKind, SettlementPlacement, Way,
    civ_apply_settlement_gravity, civ_catmull_rom_sample, civ_enhanced_travel_cost, civ_routing_grid,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// The cost multiplier on a routing cell an earlier re-traced edge already uses.
///
/// The reference's own flat "this cell is on a way" router discount,
/// `_CIV_EXISTING_WAY_DISCOUNT` (`Cartalith Gen1 v2.11.html` line 22241, carried
/// here as [`crate::way_class::REFERENCE_WAY_DISCOUNT`]), which its village
/// connector uses for exactly this job: a sibling "forks off it instead of
/// cutting a parallel line". The sweep in `cartalith-native/docs/STATUS.md`
/// (2026-11-12 entry; 0.55, 0.40 and 0.25 on two worlds) found no bundle left at
/// the opening view at any of the three once shared trunks are drawn once, and no
/// consistent difference between them otherwise, so the reference's own value is
/// kept rather than a tuned one. Applied to the exact cells only, not a
/// 3x3 neighbourhood: a neighbourhood discount makes the cell beside a trunk as
/// cheap as the trunk, which is the parallel line this module exists to remove.
pub const REUSE_FACTOR: f64 = crate::way_class::REFERENCE_WAY_DISCOUNT as f64;

/// How much more a re-traced path may cost than the path it replaces, both
/// measured on the undiscounted pass-1 grid, before the original is kept.
///
/// A labelled judgement (owner request 2026-11-12: "it's often more logical to
/// take that path"): sharing a trunk is worth up to 15 % more effort on the
/// ground, about what a traveller accepts to stay on a made road. Measured in the
/// 2026-11-12 `STATUS.md` entry (the W6 detour figures), not tuned to a target.
pub const DETOUR_TOL: f64 = 0.15;

/// How many times every edge is re-traced. The first pass lays the trunks in
/// tier order, so an edge laid early never sees the edges laid after it; a second
/// pass re-traces each edge against every *other* edge's cells, so an early edge
/// can join a trunk a later one laid beside it; a third does the same against the
/// second's result. Chosen by the sweep in the 2026-11-12 `STATUS.md` entry: on
/// both swept worlds three passes left no convergent bundle at four times the
/// opening view and no junction end off its trunk, where one and two each left
/// one on the owner's world, for about 0.2-0.4 s on its 2048-wide network. The
/// guard is always measured against the topology's own original path, so later
/// passes cannot accumulate detours.
pub const PASSES: usize = 3;

/// The search window's margin around the original path's bounding box, as a
/// share of the box's longer side, so a route can swing out to a trunk that runs
/// beside it. A labelled judgement; [`MIN_MARGIN_CELLS`] floors it. A path that
/// would need to leave the window is a detour the guard would refuse anyway.
pub const MARGIN_SHARE: f64 = 0.25;

/// The search window's minimum margin, in routing cells. A labelled judgement:
/// enough room for a short edge between two close towns to reach a trunk one
/// settlement-gravity radius away (`civ_apply_settlement_gravity`'s radius is
/// `max(3, rw / 80)` cells, 12.8 at the 1024-cell cap).
pub const MIN_MARGIN_CELLS: usize = 16;

/// The weight of each settlement tier in the re-trace order: the reference's own
/// minimum-degree table for `_civHierarchicalNetwork` (v2.11 line 21532,
/// `{metropolis:5,capital:5,city:4,town:3,village:2,hamlet:1}`), reused as a
/// ranking so trunks between the biggest places are laid first and the smaller
/// places' roads join them, not the reverse.
fn tier_weight(k: SettlementKind) -> u32 {
    match k {
        SettlementKind::Metropolis | SettlementKind::Capital => 5,
        SettlementKind::City => 4,
        SettlementKind::Town => 3,
        SettlementKind::Village => 2,
        SettlementKind::Hamlet => 1,
    }
}

/// Settings for [`civ_merge_network_paths`]. `Default` is the shipped setting;
/// the census in `cartalith-godot` sweeps the other values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WayMergeOpts {
    /// See [`REUSE_FACTOR`].
    pub reuse_factor: f64,
    /// See [`DETOUR_TOL`].
    pub detour_tol: f64,
    /// How many times every edge is re-traced; see [`PASSES`].
    pub passes: usize,
}

impl Default for WayMergeOpts {
    /// The shipped values: [`REUSE_FACTOR`] and [`DETOUR_TOL`].
    fn default() -> Self {
        WayMergeOpts { reuse_factor: REUSE_FACTOR, detour_tol: DETOUR_TOL, passes: PASSES }
    }
}

/// What happened to each edge, in the topology's own edge order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WayMergeReport {
    /// Per edge: the original path's undiscounted pass-1 cost.
    pub original_cost: Vec<f64>,
    /// Per edge: the kept path's undiscounted pass-1 cost.
    pub kept_cost: Vec<f64>,
    /// Per edge: whether the re-traced path was kept (`false`: the guard kept
    /// the original, or the search found nothing).
    pub rerouted: Vec<bool>,
}

/// An `f64` heap key ordered for a min-heap, ties broken by the cell index so
/// the search is deterministic (the topology's own heap is in `crate`, keyed
/// for whole-grid sources; this one serves a window).
#[derive(Clone, Copy, PartialEq)]
struct Key(f64, usize);
impl Eq for Key {}
impl Ord for Key {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0).then_with(|| o.1.cmp(&self.1))
    }
}
impl PartialOrd for Key {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// A rectangle of the routing grid to search in: columns `x0..x0 + w` and rows
/// `y0..y0 + h`. `wrap` is set only when the window is the full width of a
/// wrapping world, the one case where a step may cross the seam.
#[derive(Clone, Copy, Debug)]
struct Window {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    wrap: bool,
}

/// The step cost between two 8-adjacent routing cells: `len * 0.5 * (c_i + c_j)`,
/// `len` `sqrt(2)` for a diagonal. The topology's own `road_dijkstra_multi`
/// formula, so a cost measured here is a cost on the grid the network was built
/// on.
fn step(ci: f64, cj: f64, diag: bool) -> f64 {
    (if diag { std::f64::consts::SQRT_2 } else { 1.0 }) * 0.5 * (ci + cj)
}

/// Whether two routing cells `a`, `b` (indices on a grid `rw` wide) are a
/// diagonal step, wrap-aware when `world`.
fn is_diag(a: usize, b: usize, rw: usize, world: bool) -> bool {
    let (ax, bx) = (a % rw, b % rw);
    let dx = ax.abs_diff(bx);
    let dx = if world { dx.min(rw - dx) } else { dx };
    dx != 0 && a / rw != b / rw
}

/// A path's cost on `cost` by [`step`]. Infinite when any cell is.
pub fn path_cost(cost: &[f32], path: &[usize], rw: usize, world: bool) -> f64 {
    path.windows(2).map(|p| step(cost[p[0]] as f64, cost[p[1]] as f64, is_diag(p[0], p[1], rw, world))).sum()
}

/// The search window for a path: its bounding box plus a margin
/// ([`MARGIN_SHARE`], floored at [`MIN_MARGIN_CELLS`]), clamped to the grid. A
/// path that crosses a wrapping world's seam takes the full width (the box of
/// its columns would be the wrong side of the world).
fn window_for(path: &[usize], rw: usize, rh: usize, world: bool) -> Window {
    let crosses_seam = world && path.windows(2).any(|p| (p[0] % rw).abs_diff(p[1] % rw) > 1);
    let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0, usize::MAX, 0);
    for &c in path {
        let (x, y) = (c % rw, c / rw);
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    let side = (x1 - x0).max(y1 - y0) as f64;
    let m = ((side * MARGIN_SHARE).ceil() as usize).max(MIN_MARGIN_CELLS);
    let (ya, yb) = (y0.saturating_sub(m), (y1 + m).min(rh - 1));
    if crosses_seam {
        return Window { x0: 0, y0: ya, w: rw, h: yb - ya + 1, wrap: true };
    }
    let (xa, xb) = (x0.saturating_sub(m), (x1 + m).min(rw - 1));
    Window { x0: xa, y0: ya, w: xb - xa + 1, h: yb - ya + 1, wrap: false }
}

/// Least-cost path from routing cell `s` to `t` inside `win`, over `cost` with
/// every cell whose `laid` count is above zero multiplied by `factor`, stopping as soon as `t` is
/// settled. Returns the path `s..=t`, or `None` when `t` is unreachable inside
/// the window. Steps never leave the window, so a cell outside it is never read.
fn search(
    cost: &[f32],
    laid: &[u16],
    factor: f64,
    rw: usize,
    win: Window,
    s: usize,
    t: usize,
) -> Option<Vec<usize>> {
    let n = win.w * win.h;
    let local = |g: usize| -> usize {
        let (x, y) = (g % rw, g / rw);
        (y - win.y0) * win.w + (x - win.x0)
    };
    let global = |l: usize| -> usize { (win.y0 + l / win.w) * rw + win.x0 + l % win.w };
    let eff = |g: usize| -> f64 {
        let c = cost[g] as f64;
        if laid[g] > 0 { c * factor } else { c }
    };
    let (ls, lt) = (local(s), local(t));
    let mut dist = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut done = vec![false; n];
    let mut heap = BinaryHeap::new();
    dist[ls] = 0.0;
    heap.push(Key(0.0, ls));
    while let Some(Key(d, i)) = heap.pop() {
        if done[i] {
            continue;
        }
        done[i] = true;
        if i == lt {
            break;
        }
        let gi = global(i);
        let ci = eff(gi);
        if !ci.is_finite() {
            continue;
        }
        let (x, y) = ((i % win.w) as isize, (i / win.w) as isize);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = if win.wrap { (x + dx).rem_euclid(win.w as isize) } else { x + dx };
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= win.w as isize || ny >= win.h as isize {
                    continue;
                }
                let j = ny as usize * win.w + nx as usize;
                if done[j] {
                    continue;
                }
                let cj = eff(global(j));
                if !cj.is_finite() {
                    continue;
                }
                let nd = d + step(ci, cj, dx != 0 && dy != 0);
                if nd < dist[j] {
                    dist[j] = nd;
                    prev[j] = i;
                    heap.push(Key(nd, j));
                }
            }
        }
    }
    if !dist[lt].is_finite() {
        return None;
    }
    let mut out = vec![t];
    let mut c = lt;
    while c != ls {
        c = prev[c];
        if c == usize::MAX {
            return None;
        }
        out.push(global(c));
    }
    out.reverse();
    Some(out)
}

/// One edge's re-trace and its guard: the least-cost path between the path's
/// two end cells over `cost` with `laid` cells at `opts.reuse_factor`, kept only
/// when its undiscounted cost is at most `(1 + opts.detour_tol)` times
/// `orig_cost` (the original path's, by [`path_cost`]). Returns the kept path and
/// its undiscounted cost, or `None` to keep the original (a path of fewer than two
/// cells, an infinite original, nothing reachable in the window, or the guard).
#[allow(clippy::too_many_arguments)]
fn retrace(
    cost: &[f32],
    laid: &[u16],
    rw: usize,
    rh: usize,
    world: bool,
    path: &[usize],
    orig_cost: f64,
    opts: &WayMergeOpts,
) -> Option<(Vec<usize>, f64)> {
    let n = path.len();
    if n < 2 || !orig_cost.is_finite() {
        return None;
    }
    let win = window_for(path, rw, rh, world);
    let p = search(cost, laid, opts.reuse_factor, rw, win, path[0], path[n - 1])?;
    let c = path_cost(cost, &p, rw, world);
    (c <= orig_cost * (1.0 + opts.detour_tol)).then_some((p, c))
}

/// The pass-1 cost grid the topology routes its tree over: `civ_enhanced_travel_cost`
/// with no reuse term, then settlement gravity -- the same two calls, with the
/// same arguments, as `civ_hierarchical_network_topology`'s pass 1. Returns the
/// grid and its width, height and wrap flag.
#[allow(clippy::too_many_arguments)]
pub fn civ_pass1_cost(
    places: &[SettlementPlacement],
    gw: usize,
    gh: usize,
    sea: f64,
    field: &[f32],
    flow: &[f32],
    river_order: &[i16],
    biome: &[u8],
    water_bodies: &[u8],
    world: bool,
    map_width_km: f64,
) -> (Vec<f32>, usize, usize) {
    let grid = civ_routing_grid(field, gw, gh);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let mut cost = civ_enhanced_travel_cost(
        &grid.dfld, grid.rw, grid.rh, sea, None, gw, gh, Some(water_bodies), Some(flow), flow_thresh,
        Some(river_order), Some(biome),
    );
    civ_apply_settlement_gravity(&mut cost, grid.rw, grid.rh, grid.sc, places, world);
    (cost, grid.rw, grid.rh)
}

/// Re-traces every edge of `topology` so later roads join the trunks earlier
/// ones laid (this module's doc comment says why and what it must never do).
///
/// `places` is the network's own settlement list, the one the topology's pass 1
/// applied settlement gravity over, so the cost grid here is that pass's grid.
/// `kinds` is each settlement's tier in the index space `topology`'s `a`/`b` use
/// (the same list on the auto-populate path; the kept settlement list on the
/// SG-02 keep path, where the edges were remapped onto it), read after the
/// metropolis promotion. Edges are taken by descending
/// [`tier_weight`]`(a) + tier_weight(b)`, ties in the topology's own edge order;
/// an index past `kinds` weighs 0.
/// Each edge keeps its two end cells (`path[0]` stays `a`'s routing cell and the
/// last stays `b`'s, the orientation consolidation reads) and its place in
/// `topology.edges`; only the cells between change, and only when the guard
/// accepts. `usage_count` is then recounted from the kept paths, so a shared
/// trunk counts every edge riding it (`civ_classify_way` reads each edge's
/// peak). That peak can also fall: on the owner's world (2048 x 1311, seed
/// 246371) the reference network's only cell used by 8 edges, the one that
/// made those 8 edges highways (7 drawn), is used by 7 once one edge no longer
/// passes through it, so those ways draw as regional roads (`STATUS.md`,
/// 2026-11-12, measured from the saved `road_edges`).
///
/// Sequential by design: each re-trace reads the cells the previous ones laid.
#[allow(clippy::too_many_arguments)]
pub fn civ_merge_network_paths(
    topology: &mut HierarchicalNetworkResult,
    places: &[SettlementPlacement],
    kinds: &[SettlementKind],
    gw: usize,
    gh: usize,
    sea: f64,
    field: &[f32],
    flow: &[f32],
    river_order: &[i16],
    biome: &[u8],
    water_bodies: &[u8],
    world: bool,
    map_width_km: f64,
    opts: &WayMergeOpts,
) -> WayMergeReport {
    let n_edges = topology.edges.len();
    let mut report = WayMergeReport {
        original_cost: vec![f64::NAN; n_edges],
        kept_cost: vec![f64::NAN; n_edges],
        rerouted: vec![false; n_edges],
    };
    if n_edges == 0 || gw == 0 || gh == 0 {
        return report;
    }
    let (cost, rw, rh) =
        civ_pass1_cost(places, gw, gh, sea, field, flow, river_order, biome, water_bodies, world, map_width_km);
    let weight = |i: usize| kinds.get(i).map(|&k| tier_weight(k)).unwrap_or(0);
    let mut order: Vec<usize> = (0..n_edges).collect();
    // Stable, so equal tier sums keep the topology's own order (pass 1's tree
    // first, then the degree fill, then the shortcuts).
    order.sort_by_key(|&k| std::cmp::Reverse(weight(topology.edges[k].a) + weight(topology.edges[k].b)));

    for (k, e) in topology.edges.iter().enumerate() {
        let c = path_cost(&cost, &e.path, rw, world);
        report.original_cost[k] = c;
        report.kept_cost[k] = c;
    }
    let originals: Vec<Vec<usize>> = topology.edges.iter().map(|e| e.path.clone()).collect();
    // How many laid edges use each cell. Pass 1 starts empty and adds each edge
    // as it is laid; later passes start from every edge and take the edge being
    // re-traced out first, so it is attracted only to the others.
    let mut laid = vec![0u16; rw * rh];
    for pass in 0..opts.passes.max(1) {
        for &k in &order {
            if pass > 0 {
                for &c in &topology.edges[k].path {
                    laid[c] -= 1;
                }
            }
            let orig_cost = report.original_cost[k];
            if let Some((p, c)) = retrace(&cost, &laid, rw, rh, world, &topology.edges[k].path, orig_cost, opts) {
                report.kept_cost[k] = c;
                topology.edges[k].path = p;
            }
            for &c in &topology.edges[k].path {
                laid[c] = laid[c].saturating_add(1);
            }
        }
    }
    for (k, orig) in originals.iter().enumerate() {
        report.rerouted[k] = topology.edges[k].path != *orig;
    }
    topology.usage_count = vec![0u16; rw * rh];
    for e in &topology.edges {
        for &c in &e.path {
            topology.usage_count[c] = topology.usage_count[c].saturating_add(1);
        }
    }
    report
}

/// How far, in map cells, a way's junction end may be moved onto the drawn line
/// of the way it joins. A labelled judgement: consolidation cuts a branch at a
/// cell of the busier way's routed path, but that way is drawn smoothed
/// (`civ_smooth_path`'s RDP tolerance is 1.5 cells, then a Catmull-Rom), so the
/// drawn trunk can pass up to about that far from the cut; 3 cells is twice
/// that, and short of the distance at which two real roads sit side by side.
pub const JUNCTION_SNAP_CELLS: f64 = 3.0;

/// The render step `cartalith-godot`'s `way_render_polyline` samples a way's
/// curve at (`WAY_RENDER_STEP_CELLS`, 0.25 cells), so a junction end is snapped
/// onto the curve the overlay actually strokes.
const RENDER_STEP_CELLS: f64 = 0.25;

/// A way's drawn curve: each pen-down run Catmull-Rom sampled at
/// [`RENDER_STEP_CELLS`], as the overlay draws it.
fn drawn_runs(w: &Way) -> Vec<Vec<(f64, f64)>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for &cut in w.brks.iter().chain(std::iter::once(&w.pts.len())) {
        let cut = cut.min(w.pts.len());
        if cut > start {
            let run = &w.pts[start..cut];
            let curve = civ_catmull_rom_sample(run, RENDER_STEP_CELLS);
            out.push(if curve.len() >= run.len() { curve } else { run.to_vec() });
        }
        start = start.max(cut);
    }
    out
}

/// Joins every way's junction ends onto the drawn line of the way they meet, so
/// a branch reaches its trunk on the screen instead of stopping a smoothing
/// tolerance short of it (owner Ruling EU item 1: "merge and become a single
/// way").
///
/// A **junction end** is a way's first or last point that is not on one of its
/// own two settlements (more than 1.5 cells from both, consolidation's own
/// `EXTEND_T2` reach). Each is moved to the nearest point of any *other* visible
/// way's drawn curve within [`JUNCTION_SNAP_CELLS`] (for a road, any other
/// road: never a village track); an end with nothing that
/// close is left where it is, and a candidate across a world-wrap seam (more than
/// half the map away in x) is ignored. Ways are taken in list order
/// (consolidation emits the busiest first), and a moved way's curve is redrawn
/// before later ways snap onto it. Moving a way's own junction end bends the
/// last stretch of its curve, which can leave an end already snapped onto that
/// stretch a fraction of a cell off, so the whole pass repeats until no end moves, at most
/// [`SNAP_ROUNDS`] times. `km` is not re-measured: it stays the consolidation's
/// own figure. Hidden ways are neither moved nor snapped onto.
///
/// Called only with `CivParams::merge_ways` on; it must never move a way's
/// settlement end.
pub fn civ_snap_junction_ends(ways: &mut [Way], places: &[NamedSettlement], gw: usize) {
    for _ in 0..SNAP_ROUNDS {
        if !snap_round(ways, places, gw) {
            break;
        }
    }
}

/// The most rounds [`civ_snap_junction_ends`] runs. A labelled judgement: each
/// round only re-joins ends a previous round's moves nudged, by a fraction of a
/// cell, so the passes settle within two or three; the cap keeps a pathological
/// pair of mutually snapping ends from looping.
pub const SNAP_ROUNDS: usize = 6;

/// One round of [`civ_snap_junction_ends`]; `true` when any end moved.
fn snap_round(ways: &mut [Way], places: &[NamedSettlement], gw: usize) -> bool {
    let mut any = false;
    let mut curves: Vec<Vec<Vec<(f64, f64)>>> =
        ways.iter().map(|w| if w.hidden || w.pts.len() < 2 { Vec::new() } else { drawn_runs(w) }).collect();
    let own = |w: &Way, p: (f64, f64)| {
        [w.a_idx, w.b_idx].iter().any(|&i| {
            places.get(i).is_some_and(|s| (p.0 - s.placement.x as f64).hypot(p.1 - s.placement.y as f64) <= 1.5)
        })
    };
    for i in 0..ways.len() {
        if curves[i].is_empty() {
            continue;
        }
        let mut moved = false;
        for end in [0usize, 1] {
            let k = if end == 0 { 0 } else { ways[i].pts.len() - 1 };
            let p = ways[i].pts[k];
            if own(&ways[i], p) {
                continue;
            }
            let mut best: Option<((f64, f64), f64)> = None;
            for (j, runs) in curves.iter().enumerate() {
                // A road never ends on a village track: the tracks are cut back
                // onto the roads afterwards (`civ_join_tracks_to_network`), which
                // would leave such an end hanging (measured on the 256 x 192 app
                // fixture before this rule: 0.21 cells off).
                let track = ways[j].way_type == crate::WayType::Ancient;
                if j == i || (track && ways[i].way_type != crate::WayType::Ancient) {
                    continue;
                }
                for run in runs {
                    for seg in run.windows(2) {
                        let (a, b) = (seg[0], seg[1]);
                        if (a.0 - p.0).abs() > gw as f64 / 2.0 {
                            continue;
                        }
                        let (sx, sy) = (b.0 - a.0, b.1 - a.1);
                        let l2 = sx * sx + sy * sy;
                        let t = if l2 > 0.0 { (((p.0 - a.0) * sx + (p.1 - a.1) * sy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                        let q = (a.0 + sx * t, a.1 + sy * t);
                        let d = (p.0 - q.0).hypot(p.1 - q.1);
                        if d <= JUNCTION_SNAP_CELLS && best.is_none_or(|(_, bd)| d < bd) {
                            best = Some((q, d));
                        }
                    }
                }
            }
            if let Some((q, d)) = best
                && d > 1e-9
            {
                ways[i].pts[k] = q;
                moved = true;
            }
        }
        if moved {
            curves[i] = drawn_runs(&ways[i]);
            any = true;
        }
    }
    any
}

/// The nearest point to `p` on any of `curves` (drawn runs), and its distance,
/// within `r` cells; candidates more than half the map away in x (across a
/// wrap seam) are ignored.
fn nearest_on(curves: &[&Vec<Vec<(f64, f64)>>], p: (f64, f64), r: f64, gw: usize) -> Option<((f64, f64), f64)> {
    let mut best: Option<((f64, f64), f64)> = None;
    for runs in curves {
        for run in runs.iter() {
            for seg in run.windows(2) {
                let (a, b) = (seg[0], seg[1]);
                if (a.0 - p.0).abs() > gw as f64 / 2.0 {
                    continue;
                }
                let (sx, sy) = (b.0 - a.0, b.1 - a.1);
                let l2 = sx * sx + sy * sy;
                let t = if l2 > 0.0 { (((p.0 - a.0) * sx + (p.1 - a.1) * sy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                let q = (a.0 + sx * t, a.1 + sy * t);
                let d = (p.0 - q.0).hypot(p.1 - q.1);
                if d <= r && best.is_none_or(|(_, bd)| d < bd) {
                    best = Some((q, d));
                }
            }
        }
    }
    best
}

/// Ends every village track where it first reaches the road network or an
/// earlier village track, joined onto that line (owner Ruling EU item 1).
///
/// The village connector (`tools::civ_connect_village_addons`, the reference's
/// `_civConnectVillageAddons`) routes each track from its village to the nearest
/// settlement, over a cost grid where every cell within one routing cell of a
/// way is cheaper -- so the last stretch of a track runs *beside* the road into
/// town, a second stroke a cell or two off the first (measured: most of the
/// convergent bundles left at four times the opening view on the census worlds
/// once the network itself is merged; `STATUS.md`, 2026-11-12). Here a track
/// (`WayType::Ancient`, not hidden) is cut at its first control point, walking
/// from the village end, within [`JUNCTION_SNAP_CELLS`] of a visible non-track
/// way's drawn curve, and that point is moved onto the curve.
///
/// Tracks are taken in order, and each track's kept line (cut or whole) is
/// added to the lines later tracks join, so two tracks running side by side
/// into the same town become one: the later one ends where it first comes
/// within [`JUNCTION_SNAP_CELLS`] of the earlier one. Measured on the census's
/// 18 small worlds with villages before this was added
/// (`way_census::diagnose_residual_bundle_pairs`, 2026-11-12): every
/// convergent bundle left at the opening view (16 of 16) and 64 of the 72 at
/// x4 ran between two tracks, none between a track and a road. A track joined
/// onto another track still reaches the network through it (the earlier track
/// is final when it is added), and the order is the connector's own, so the
/// result is deterministic. A track that never comes that close to a road or
/// an earlier track is left whole. A village already
/// on a road keeps a two-point track from its pin to the road; one whose pin is
/// exactly on the line is hidden (nothing to draw). `km` is scaled by the share
/// of the control polyline kept, a labelled approximation of the consolidation's
/// sample-based figure. `b_idx` stays the place the track was routed to.
///
/// Called only with `CivParams::merge_ways` on; it must never move a track's
/// village end, and never touch a non-track way.
pub fn civ_join_tracks_to_network(ways: &mut [Way], gw: usize) {
    // The lines a track may join: every visible road first, then each track's
    // kept line as it is finished (see the doc).
    let mut lines: Vec<Vec<Vec<(f64, f64)>>> = ways
        .iter()
        .filter(|w| !w.hidden && w.pts.len() >= 2 && w.way_type != crate::WayType::Ancient)
        .map(drawn_runs)
        .collect();
    let length = |pts: &[(f64, f64)], brks: &[usize]| -> f64 {
        pts.windows(2)
            .enumerate()
            .filter(|(i, _)| !brks.contains(&(i + 1)))
            .map(|(_, s)| (s[1].0 - s[0].0).hypot(s[1].1 - s[0].1))
            .sum()
    };
    for w in ways.iter_mut() {
        if w.hidden || w.pts.len() < 2 || w.way_type != crate::WayType::Ancient {
            continue;
        }
        let refs: Vec<&Vec<Vec<(f64, f64)>>> = lines.iter().collect();
        let hit = w
            .pts
            .iter()
            .enumerate()
            .find_map(|(k, &p)| nearest_on(&refs, p, JUNCTION_SNAP_CELLS, gw).map(|hit| (k, hit)));
        if let Some((k, (q, d))) = hit {
            let before = length(&w.pts, &w.brks);
            if k == 0 {
                if d <= 0.0 {
                    w.hidden = true;
                    continue;
                }
                w.pts = vec![w.pts[0], q];
                w.brks.clear();
            } else {
                w.pts.truncate(k + 1);
                w.pts[k] = q;
                w.brks.retain(|&b| b <= k);
            }
            if before > 0.0 {
                w.km *= length(&w.pts, &w.brks) / before;
            }
        }
        lines.push(drawn_runs(w));
    }
}

/// The least undiscounted pass-1 cost from `edge`'s first cell to its last,
/// searched in the same window the merge uses: the "direct" cost a detour ratio
/// is measured against (W6 in the 2026-11-12 census). `None` when unreachable in
/// the window. Measurement only; generation never calls it.
pub fn civ_direct_edge_cost(cost: &[f32], edge: &RoadEdge, rw: usize, rh: usize, world: bool) -> Option<f64> {
    let n = edge.path.len();
    if n < 2 {
        return None;
    }
    let none = vec![0u16; cost.len()];
    let win = window_for(&edge.path, rw, rh, world);
    search(cost, &none, 1.0, rw, win, edge.path[0], edge.path[n - 1]).map(|p| path_cost(cost, &p, rw, world))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat open grid `w x h` of unit cost (no water).
    fn flat(w: usize, h: usize) -> Vec<f32> {
        vec![1.0; w * h]
    }

    /// A straight row path from `(x0, y)` to `(x1, y)` on a grid `w` wide.
    fn row(y: usize, x0: usize, x1: usize, w: usize) -> Vec<usize> {
        (x0..=x1).map(|x| y * w + x).collect()
    }

    /// Protects: the step formula is the topology's (`len * 0.5 * (ci + cj)`,
    /// sqrt 2 on a diagonal), so a merged path's cost is comparable with the
    /// original's. Literals: a 3-cell straight row of unit cost costs 2.0; one
    /// diagonal step between cost-1 and cost-3 cells costs 2 sqrt 2.
    #[test]
    fn path_cost_is_the_topologys_step_formula() {
        let c = flat(8, 8);
        assert_eq!(path_cost(&c, &row(2, 1, 3, 8), 8, false), 2.0);
        let mut c2 = flat(8, 8);
        c2[3 * 8 + 3] = 3.0;
        let d = path_cost(&c2, &[2 * 8 + 2, 3 * 8 + 3], 8, false);
        assert!((d - 2.0 * std::f64::consts::SQRT_2).abs() < 1e-12, "{d}");
        // Wrap-aware diagonal: column 0 to column w-1 on the next row is one diagonal step.
        assert!((path_cost(&c, &[0, 8 + 7], 8, true) - std::f64::consts::SQRT_2).abs() < 1e-12);
    }

    /// Protects: the reuse discount draws a later route onto a laid trunk two
    /// rows away rather than letting it run parallel (the owner's "bundle of
    /// wires"), and the guard accepts it because the undiscounted detour is small.
    /// Fixture: a 40 x 12 flat grid; the trunk is row 5 from x 2 to 37; the second
    /// route runs from (4, 7) to (35, 7). Without merging it is row 7, two rows off
    /// the trunk; merged, most of its cells are trunk cells.
    #[test]
    fn a_later_route_joins_a_trunk_two_rows_away() {
        let (w, h) = (40, 12);
        let cost = flat(w, h);
        let mut laid = vec![0u16; w * h];
        for c in row(5, 2, 37, w) {
            laid[c] = 1;
        }
        let second = row(7, 4, 35, w);
        let win = window_for(&second, w, h, false);
        let p = search(&cost, &laid, REUSE_FACTOR, w, win, second[0], *second.last().unwrap()).unwrap();
        let on_trunk = p.iter().filter(|&&c| laid[c] > 0).count();
        assert!(on_trunk >= 20, "only {on_trunk} of {} cells ride the trunk", p.len());
        let (orig, new) = (path_cost(&cost, &second, w, false), path_cost(&cost, &p, w, false));
        assert!(new <= orig * (1.0 + DETOUR_TOL), "detour {new} vs {orig}");
        // Negative control: with no reuse discount the search keeps the straight row.
        let p0 = search(&cost, &laid, 1.0, w, win, second[0], *second.last().unwrap()).unwrap();
        assert_eq!(path_cost(&cost, &p0, w, false), orig);
        assert!(p0.iter().all(|&c| laid[c] == 0), "an undiscounted search must not touch the trunk");
    }

    /// Protects: the detour guard. The trunk is a ridge road: its cells cost 2
    /// (steep ground), so at the shipped reuse factor riding it is cheaper on the
    /// discounted grid (2 x 0.25 = 0.5 a cell, against 1 on the flat) but about
    /// twice the effort on the ground (under 2.15, so a guard loosened to
    /// `2 + tol` would accept it). `retrace` must keep the original.
    /// The control: the same trunk on flat ground (cost 1) is accepted.
    /// Fixture: 50 x 12 grid, route row 8 from x 5 to 44, trunk row 6 from x 0 to 49.
    #[test]
    fn the_guard_refuses_a_trunk_over_a_ridge() {
        let (w, h) = (50, 12);
        let mut cost = flat(w, h);
        let mut laid = vec![0u16; w * h];
        for c in row(6, 0, 49, w) {
            laid[c] = 1;
            cost[c] = 2.0;
        }
        let route = row(8, 5, 44, w);
        let orig = path_cost(&cost, &route, w, false);
        let win = window_for(&route, w, h, false);
        let p = search(&cost, &laid, REUSE_FACTOR, w, win, route[0], *route.last().unwrap()).unwrap();
        assert!(p.iter().filter(|&&c| laid[c] > 0).count() > 20, "premise: the discounted search rides the ridge");
        let ride = path_cost(&cost, &p, w, false);
        assert!(ride > orig * 1.5 && ride < orig * 2.15, "premise: the ride is a large detour on the ground: {ride} vs {orig}");
        assert_eq!(retrace(&cost, &laid, w, h, false, &route, orig, &WayMergeOpts::default()), None);
        // Control: the same trunk on flat ground passes the guard.
        for c in row(6, 0, 49, w) {
            cost[c] = 1.0;
        }
        let kept = retrace(&cost, &laid, w, h, false, &route, orig, &WayMergeOpts::default());
        let (kp, kc) = kept.expect("a flat trunk two rows off is within 15 %");
        assert!(kp.iter().filter(|&&c| laid[c] > 0).count() > 20);
        assert!(kc <= orig * 1.15);
        // And the literal: 15 % is the shipped tolerance.
        assert_eq!(DETOUR_TOL, 0.15);
    }

    /// Protects: water stays impassable. A lake (infinite cost) between start and
    /// end forces the path round it; no cell of the result is infinite.
    #[test]
    fn a_search_never_crosses_water() {
        let (w, h) = (30, 20);
        let mut cost = flat(w, h);
        for y in 0..16 {
            cost[y * w + 15] = f32::INFINITY;
        }
        let win = Window { x0: 0, y0: 0, w, h, wrap: false };
        let p = search(&cost, &vec![0u16; w * h], 1.0, w, win, 2 * w + 5, 2 * w + 25).unwrap();
        assert!(p.iter().all(|&c| cost[c].is_finite()));
        assert!(p.iter().any(|&c| c / w >= 16), "the path must go round the lake's open end");
        // And an unreachable target is None, not a straight line.
        for y in 0..h {
            cost[y * w + 15] = f32::INFINITY;
        }
        assert!(search(&cost, &vec![0u16; w * h], 1.0, w, win, 2 * w + 5, 2 * w + 25).is_none());
    }

    /// Protects: a junction end is moved onto the drawn line it meets, and a
    /// settlement end never moves. Fixture: a trunk along y = 10 from x 0 to 40
    /// (settlements 0 and 1 at its ends) and a branch from settlement 2 at (20, 30)
    /// up to (20, 12), two cells short of the trunk (its junction end). Negative
    /// control: an end 4 cells off (past `JUNCTION_SNAP_CELLS`, 3) stays put.
    #[test]
    fn a_branch_end_is_joined_onto_its_trunk() {
        let place = |x, y| NamedSettlement {
            tid: 0,
            placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Town, coastal: false },
            name: String::new(),
            pop: 0,
        };
        let places = vec![place(0, 10), place(40, 10), place(20, 30), place(30, 30)];
        let way = |pts: Vec<(f64, f64)>, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 0.0, name: String::new(), way_type: crate::WayType::Road, a_idx: a, b_idx: b, hidden: false,
        };
        let trunk = way(vec![(0.0, 10.0), (10.0, 10.0), (20.0, 10.0), (30.0, 10.0), (40.0, 10.0)], 0, 1);
        let branch = way(vec![(20.0, 30.0), (20.0, 20.0), (20.0, 12.0)], 2, 1);
        let far = way(vec![(30.0, 30.0), (30.0, 20.0), (30.0, 14.0)], 3, 1);
        let mut ways = vec![trunk.clone(), branch, far];
        civ_snap_junction_ends(&mut ways, &places, 64);
        assert_eq!(ways[0].pts, trunk.pts, "the trunk's ends are its own settlements");
        let end = *ways[1].pts.last().unwrap();
        assert!((end.0 - 20.0).abs() < 1e-9 && (end.1 - 10.0).abs() < 1e-9, "{end:?}");
        assert_eq!(ways[1].pts[0], (20.0, 30.0), "the settlement end never moves");
        assert_eq!(*ways[2].pts.last().unwrap(), (30.0, 14.0), "4 cells is past the snap reach");
        assert_eq!(JUNCTION_SNAP_CELLS, 3.0);
    }

    /// Protects: a village track that runs beside a road into town is cut where it
    /// first comes within 3 cells of the road and joined onto it; its village end
    /// never moves; a track that never nears a road is left whole; a non-track way
    /// is never touched. Fixture: a road along y = 10; a track from a village at
    /// (20, 30) down to (20, 12) then beside the road (y = 11.5) to (35, 11.5).
    #[test]
    fn a_village_track_ends_where_it_reaches_the_road() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(vec![(0.0, 10.0), (20.0, 10.0), (40.0, 10.0)], crate::WayType::Road, 0, 1);
        let track = way(
            vec![(20.0, 30.0), (20.0, 20.0), (20.0, 14.0), (20.0, 11.5), (28.0, 11.5), (35.0, 11.5)],
            crate::WayType::Ancient, 2, 1,
        );
        let lone = way(vec![(5.0, 40.0), (15.0, 40.0)], crate::WayType::Ancient, 3, 2);
        let mut ways = vec![road.clone(), track, lone.clone()];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[0], road, "a road is never touched");
        // (20, 14) is 4 cells off the road; (20, 11.5), 1.5 off, is the first within 3.
        assert_eq!(ways[1].pts.len(), 4, "cut at (20, 11.5): {:?}", ways[1].pts);
        assert_eq!(ways[1].pts[0], (20.0, 30.0));
        let end = ways[1].pts[3];
        assert!((end.0 - 20.0).abs() < 1e-9 && (end.1 - 10.0).abs() < 1e-9, "joined onto the road: {end:?}");
        assert!(ways[1].km < 10.0 && ways[1].km > 0.0);
        assert_eq!(ways[2], lone, "a track that never nears a road is left whole");
    }

    /// Protects: a village track that runs beside an earlier village track into
    /// the same town is cut where it first comes within 3 cells of that track's
    /// kept line, and joined onto it; the earlier track is untouched; the order
    /// is the slice's (the earlier track never joins the later one). Fixture:
    /// no road at all; track A from a village at (0, 10) east along y = 10 to a
    /// town at (40, 10); track B from a village at (0, 30) to (10, 14.5), then
    /// beside A at y = 12 to the same town. Negative control: (10, 14.5) is 4.5
    /// cells off A, past the reach; (20, 12), 2 off, is the first within it.
    #[test]
    fn a_village_track_ends_where_it_reaches_an_earlier_track() {
        let way = |pts: Vec<(f64, f64)>, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: crate::WayType::Ancient, a_idx: a, b_idx: b,
            hidden: false,
        };
        let a = way(vec![(0.0, 10.0), (10.0, 10.0), (20.0, 10.0), (30.0, 10.0), (40.0, 10.0)], 0, 2);
        let b = way(vec![(0.0, 30.0), (10.0, 14.5), (20.0, 12.0), (30.0, 12.0), (40.0, 10.0)], 1, 2);
        let mut ways = vec![a.clone(), b];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[0], a, "the earlier track is left whole");
        assert_eq!(ways[1].pts.len(), 3, "cut at (20, 12): {:?}", ways[1].pts);
        assert_eq!(ways[1].pts[0], (0.0, 30.0), "the village end never moves");
        let end = ways[1].pts[2];
        assert!((end.0 - 20.0).abs() < 1e-6 && (end.1 - 10.0).abs() < 1e-6, "joined onto track A: {end:?}");
        assert!(ways[1].km < 10.0 && ways[1].km > 0.0);
    }

    /// Protects: the window. Its margin is `max(16, ceil(0.25 * longer side))`
    /// cells (literals), clamped to the grid; a seam-crossing path on a wrapping
    /// world takes the full width with wrap on.
    #[test]
    fn the_window_is_the_box_plus_a_floored_margin() {
        let w = 200;
        let p = row(50, 60, 140, w); // 80 cells long: margin ceil(20) = 20
        let win = window_for(&p, w, 100, false);
        assert_eq!((win.x0, win.w, win.y0, win.h, win.wrap), (40, 121, 30, 41, false));
        let short = row(50, 100, 104, w); // margin floored at 16
        let win = window_for(&short, w, 100, false);
        assert_eq!((win.x0, win.w, win.y0, win.h), (84, 37, 34, 33));
        let seam = vec![50 * w + 198, 50 * w + 199, 50 * w, 50 * w + 1];
        let win = window_for(&seam, w, 100, true);
        assert_eq!((win.x0, win.w, win.wrap), (0, w, true));
    }

    /// Protects: the re-trace order and the recount. Two edges on a flat grid: the
    /// town-town edge (tier sum 6) is laid before the hamlet-hamlet edge (sum 2)
    /// even though it comes second in the list; the hamlet edge then joins it, and
    /// `usage_count` counts 2 on the shared cells. End cells never move. The rows
    /// are four apart, past the settlement-gravity radius (3 cells on this grid),
    /// so the town road's own optimum is its straight row. Long (120 cells), so
    /// the hamlet road's two four-cell climbs to the trunk stay well inside the
    /// 15 % guard (a 50-cell version measured 54.9 against a 54.6 limit and was
    /// rightly refused: a strong discount rides a trunk as far as it can).
    #[test]
    fn bigger_places_lay_the_trunk_and_usage_is_recounted() {
        let (w, h) = (120, 20);
        let field = vec![0.6f32; w * h];
        let zeros_f = vec![0.0f32; w * h];
        let zeros_o = vec![0i16; w * h];
        let zeros_b = vec![7u8; w * h];
        let zeros_w = vec![0u8; w * h];
        let place = |x, y, kind| SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind, coastal: false };
        let places = vec![
            place(5, 10, SettlementKind::Hamlet),
            place(114, 10, SettlementKind::Hamlet),
            place(3, 6, SettlementKind::Town),
            place(116, 6, SettlementKind::Town),
        ];
        let hamlets = row(10, 5, 114, w);
        let towns = row(6, 3, 116, w);
        let mut topo = HierarchicalNetworkResult {
            edges: vec![RoadEdge { a: 0, b: 1, path: hamlets.clone() }, RoadEdge { a: 2, b: 3, path: towns.clone() }],
            usage_count: vec![0; w * h],
            degree_of: vec![1; 4],
        };
        let kinds: Vec<SettlementKind> = places.iter().map(|p| p.kind).collect();
        let rep = civ_merge_network_paths(
            &mut topo, &places, &kinds, w, h, 0.4, &field, &zeros_f, &zeros_o, &zeros_b, &zeros_w, false, 600.0,
            &WayMergeOpts::default(),
        );
        assert!(!rep.rerouted[1], "the trunk is laid first, on an empty grid, as it was routed");
        assert!(rep.rerouted[0], "the hamlet road joins the trunk");
        let e = &topo.edges[0];
        assert_eq!((e.path[0], *e.path.last().unwrap()), (hamlets[0], *hamlets.last().unwrap()));
        let shared = e.path.iter().filter(|c| topo.edges[1].path.contains(c)).count();
        assert!(shared >= 90, "only {shared} shared cells");
        assert!(e.path.iter().filter(|c| topo.edges[1].path.contains(c)).all(|&c| topo.usage_count[c] == 2));
        assert!(rep.kept_cost[0] <= rep.original_cost[0] * 1.15);
        // The shipped settings are the measured ones (`STATUS.md`, 2026-11-12):
        // literals, so a change to either fails here and has to be re-measured.
        assert_eq!((WayMergeOpts::default().passes, PASSES), (3, 3));
        assert_eq!(WayMergeOpts::default().reuse_factor, 0.25);
    }
}
