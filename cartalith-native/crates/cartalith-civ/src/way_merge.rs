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
/// peak), and **floored per cell at the count the topology came in with**: a
/// cell never carries less traffic than the reference network routed through
/// it. Without the floor the peak could fall: on the owner's world (2048 x
/// 1311, seed 246371) the reference network's only cell used by 8 edges, the
/// one that made those 8 edges highways (7 drawn), was used by 7 once one edge
/// no longer passed through it, and every highway drew as a regional road,
/// thinner and dimmer, on the routes the merge had just made busier (verifier,
/// 2026-11-12; `STATUS.md`). The ruling asked for routes to merge, not for
/// their class to drop. An edge that still passes a cell keeps at least the
/// class that cell gave it; one re-traced off every busy cell is classed by its
/// new route, by the reference's own rule.
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
    // The recount, floored per cell at the topology's own count (see the doc:
    // a cell never carries less traffic than the reference network gave it).
    // An empty or short incoming `usage_count` (a caller that built the
    // topology by hand) floors nothing.
    let reference = std::mem::take(&mut topology.usage_count);
    topology.usage_count = vec![0u16; rw * rh];
    for e in &topology.edges {
        for &c in &e.path {
            topology.usage_count[c] = topology.usage_count[c].saturating_add(1);
        }
    }
    for (u, &r) in topology.usage_count.iter_mut().zip(&reference) {
        *u = (*u).max(r);
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
/// own two settlements' pins ([`OWN_PIN_CELLS`]). Each is moved to the nearest point of any *other* visible
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

/// How close, in map cells, a way's end must be to one of its own settlements'
/// pins to count as that settlement's end, not a junction. Consolidation and its
/// endpoint snap put a settlement end exactly on the pin, so this is a float
/// tolerance, not a reach. It was 1.5 cells (consolidation's `EXTEND_T2`)
/// until 2026-11-13, when `civ_pipeline_tests::
/// recovery_keeps_every_road_on_the_settlements_it_joined` found a cut a cell
/// from its own town, 0.28 cells off the trunk it meets, left unjoined
/// because it counted as on the town.
pub const OWN_PIN_CELLS: f64 = 1e-6;

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
            places.get(i).is_some_and(|s| (p.0 - s.placement.x as f64).hypot(p.1 - s.placement.y as f64) <= OWN_PIN_CELLS)
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

/// One line a village track may join: a visible way's drawn runs and their
/// bounding box (min x, min y, max x, max y), so a far line is skipped cheaply.
struct Line {
    runs: Vec<Vec<(f64, f64)>>,
    bbox: (f64, f64, f64, f64),
}

impl Line {
    /// The line of `w` as the overlay draws it ([`drawn_runs`]).
    fn of(w: &Way) -> Line {
        let runs = drawn_runs(w);
        let mut bbox = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for &(x, y) in runs.iter().flatten() {
            bbox = (bbox.0.min(x), bbox.1.min(y), bbox.2.max(x), bbox.3.max(y));
        }
        Line { runs, bbox }
    }

    /// The first and last point of each drawn run: where the line can meet another.
    fn ends(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.runs.iter().filter(|r| !r.is_empty()).flat_map(|r| [r[0], r[r.len() - 1]])
    }

    /// The nearest point to `p` on this line and its distance, when within `r`
    /// cells. A segment more than half the map away in x (across a wrap seam) is
    /// ignored; a line whose box lies more than `r` from `p` is not walked at
    /// all, unless the box spans more than half the map (a line across the
    /// seam, which the box cannot describe).
    fn nearest(&self, p: (f64, f64), r: f64, gw: usize) -> Option<((f64, f64), f64)> {
        let (x0, y0, x1, y1) = self.bbox;
        let spans_seam = x1 - x0 > gw as f64 / 2.0;
        if !spans_seam && (p.0 < x0 - r || p.0 > x1 + r || p.1 < y0 - r || p.1 > y1 + r) {
            return None;
        }
        let mut best: Option<((f64, f64), f64)> = None;
        for run in &self.runs {
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
        best
    }
}

/// How close, in map cells, one line's end must lie to another line for the
/// two to **meet** (a junction a traveller can turn at). A labelled judgement:
/// `civ_snap_junction_ends` and this pass put a joined end exactly on the
/// drawn curve, and two ways leaving one settlement both end on its pin, so
/// half a cell admits every real junction while two lines that merely pass
/// near each other do not meet.
const MEET_CELLS: f64 = 0.5;

/// How close, in map cells, a line's end must lie to a track's destination
/// for the line to **reach** it (a line whose curve passes within
/// [`MEET_CELLS`] of it reaches it too). 1.5 cells is consolidation's own
/// `EXTEND_T2` reach, the distance at which a way's end counts as on its
/// settlement.
const REACH_PLACE_CELLS: f64 = 1.5;

/// The spacing, in map cells, at which [`joinable_tail`] samples a track's
/// pen-down segments. A labelled judgement: a sixth of [`JUNCTION_SNAP_CELLS`],
/// fine enough that a stretch leaving the reach between two control points
/// cannot be stepped over.
const TAIL_STEP_CELLS: f64 = 0.5;

/// Where a track may be cut, and onto which lines: the first control point `k`
/// of the track (`pts`, `brks`), walking from its village end, from which the
/// whole rest of it, `pts[k..]` to its destination end, is followed by **one
/// connected chain of lines that reaches the destination**; with the lines of
/// that chain near `pts[k]`. `None` when no line reaches the destination.
///
/// The chain is found walking back from the destination. It starts as every
/// line that reaches the destination (an end within [`REACH_PLACE_CELLS`] of
/// it, or its curve within [`MEET_CELLS`]). At each sample further back
/// (every [`TAIL_STEP_CELLS`] along a pen-down segment; the two ends only of a
/// pen-up one) a chain line stays in the chain while it is within
/// [`JUNCTION_SNAP_CELLS`] of the sample, and another line joins the chain when
/// it is that near the sample and meets a chain line ([`MEET_CELLS`]) within
/// that distance of the sample. The walk stops at the first sample no chain
/// line is near.
///
/// Why: a track may be cut onto the network only where the network itself goes
/// on to the track's destination. Cutting at the first point that merely came
/// near a line (as this pass first did) cut tracks onto a highway that ran the
/// other way, and a village's walk to the place its track was routed to went
/// round the network (verifier, 2026-11-12: 43 of 3 599 tracks on the census's
/// small worlds walked more than twice the straight distance along the drawn
/// ways, the worst about 15 times, against 1 with the merge off). Requiring
/// only that the rest lie near *some* line was not enough either (measured with
/// `way_census::measure_track_detours`: 60 above twice): a highway passing three
/// cells from a hamlet it never meets covered the last stretch, and two lines
/// side by side that never meet covered a tail between them. The chain is the
/// owner's own wording, "merge and become a single way **to the same
/// destination**".
fn joinable_tail(
    pts: &[(f64, f64)],
    brks: &[usize],
    lines: &[Line],
    meets: &[Vec<(usize, (f64, f64))>],
    gw: usize,
) -> Option<(usize, Vec<usize>)> {
    let dest = *pts.last()?;
    let mut chain: Vec<usize> = (0..lines.len())
        .filter(|&i| {
            lines[i].nearest(dest, MEET_CELLS, gw).is_some()
                || lines[i].ends().any(|e| (e.0 - dest.0).hypot(e.1 - dest.1) <= REACH_PLACE_CELLS)
        })
        .collect();
    if chain.is_empty() {
        return None;
    }
    // One step of the walk: the chain at sample `p`, or `None` when no chain
    // line is near it.
    let step = |chain: &[usize], p: (f64, f64)| -> Option<Vec<usize>> {
        let next: Vec<usize> = (0..lines.len())
            .filter(|&i| lines[i].nearest(p, JUNCTION_SNAP_CELLS, gw).is_some())
            .filter(|i| {
                chain.contains(i)
                    || meets[*i]
                        .iter()
                        .any(|&(j, m)| chain.contains(&j) && (m.0 - p.0).hypot(m.1 - p.1) <= JUNCTION_SNAP_CELLS)
            })
            .collect();
        (!next.is_empty()).then_some(next)
    };
    let mut start = None;
    for k in (0..pts.len()).rev() {
        let a = pts[k];
        let samples: Vec<(f64, f64)> = match pts.get(k + 1) {
            Some(&b) if !brks.contains(&(k + 1)) => {
                let steps = ((a.0 - b.0).hypot(a.1 - b.1) / TAIL_STEP_CELLS).ceil().max(1.0) as usize;
                (0..steps)
                    .rev()
                    .map(|s| {
                        let f = s as f64 / steps as f64;
                        (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
                    })
                    .collect()
            }
            _ => vec![a],
        };
        for p in samples {
            match step(&chain, p) {
                Some(c) => chain = c,
                None => return start,
            }
        }
        start = Some((k, chain.clone()));
    }
    start
}

/// Records in `meets` every junction between line `i` and the lines before it:
/// each end of either within [`MEET_CELLS`] of the other's curve, at that end.
fn record_meets(lines: &[Line], meets: &mut Vec<Vec<(usize, (f64, f64))>>, i: usize, gw: usize) {
    meets.resize(lines.len(), Vec::new());
    for j in 0..i {
        let mut at: Vec<(f64, f64)> = lines[i].ends().filter(|&e| lines[j].nearest(e, MEET_CELLS, gw).is_some()).collect();
        at.extend(lines[j].ends().filter(|&e| lines[i].nearest(e, MEET_CELLS, gw).is_some()));
        for m in at {
            meets[i].push((j, m));
            meets[j].push((i, m));
        }
    }
}

/// Ends every village track where the road network or an earlier village track
/// takes it on to its destination, joined onto that line (owner Ruling EU item 1).
///
/// The village connector (`tools::civ_connect_village_addons`, the reference's
/// `_civConnectVillageAddons`) routes each track from its village to the nearest
/// settlement, over a cost grid where every cell within one routing cell of a
/// way is cheaper -- so the last stretch of a track runs *beside* the road into
/// town, a second stroke a cell or two off the first (measured: most of the
/// convergent bundles left at four times the opening view on the census worlds
/// once the network itself is merged; `STATUS.md`, 2026-11-12). Here a track
/// (`WayType::Ancient`, not hidden) is cut at the first control point, walking
/// from the village end, from which **the whole rest of the track** stays within
/// [`JUNCTION_SNAP_CELLS`] of one connected chain of lines that reaches its
/// destination ([`joinable_tail`]), and that point is moved onto the nearest
/// line of that chain. A line that merely passes near the track, runs the other
/// way, or never meets the lines that reach the destination never takes it.
///
/// Tracks are taken in order, and each track's kept line (cut or whole) is
/// added to the lines later tracks join, so two tracks running side by side
/// into the same town become one: the later one ends where the earlier one
/// carries the rest of its way. Measured on the census's
/// 18 small worlds with villages before this was added
/// (`way_census::diagnose_residual_bundle_pairs`, 2026-11-12): every
/// convergent bundle left at the opening view (16 of 16) and 64 of the 72 at
/// x4 ran between two tracks, none between a track and a road. A track joined
/// onto another track still reaches the network through it (the earlier track
/// is final when it is added), and the order is the connector's own, so the
/// result is deterministic. A track whose destination end is not on a line
/// (a village routed to another village that has no kept line yet) is left
/// whole. A village whose whole track runs along the lines keeps a two-point
/// track from its pin to the nearest line; one whose pin is exactly on the line
/// is hidden (nothing to draw). `km` is scaled by the share
/// of the control polyline kept, a labelled approximation of the consolidation's
/// sample-based figure. `b_idx` stays the place the track was routed to.
///
/// Called only with `CivParams::merge_ways` on; it must never move a track's
/// village end, and never touch a non-track way.
pub fn civ_join_tracks_to_network(ways: &mut [Way], gw: usize) {
    // The lines a track may join: every visible road first, then each track's
    // kept line as it is finished (see the doc), with the junctions between them.
    let mut lines: Vec<Line> = ways
        .iter()
        .filter(|w| !w.hidden && w.pts.len() >= 2 && w.way_type != crate::WayType::Ancient)
        .map(Line::of)
        .collect();
    let mut meets: Vec<Vec<(usize, (f64, f64))>> = Vec::new();
    for i in 0..lines.len() {
        record_meets(&lines, &mut meets, i, gw);
    }
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
        let hit = joinable_tail(&w.pts, &w.brks, &lines, &meets, gw).and_then(|(k, chain)| {
            chain
                .iter()
                .filter_map(|&i| lines[i].nearest(w.pts[k], JUNCTION_SNAP_CELLS, gw))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|hit| (k, hit))
        });
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
        lines.push(Line::of(w));
        record_meets(&lines, &mut meets, lines.len() - 1, gw);
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

    /// Protects: a village track that runs beside a road into town (the road
    /// reaches the town) is cut where it first comes within 3 cells of the road
    /// for the rest of its way, and joined onto it; its village end
    /// never moves; a track that never nears a road is left whole; a non-track way
    /// is never touched. Fixture: a road along y = 10; a track from a village at
    /// (20, 30) down to (20, 12) then beside the road (y = 11.5) to (35, 11.5)
    /// and into the town at the road's end, (40, 10).
    #[test]
    fn a_village_track_ends_where_it_reaches_the_road() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(vec![(0.0, 10.0), (20.0, 10.0), (40.0, 10.0)], crate::WayType::Road, 0, 1);
        let track = way(
            vec![(20.0, 30.0), (20.0, 20.0), (20.0, 14.0), (20.0, 11.5), (28.0, 11.5), (35.0, 11.5), (40.0, 10.0)],
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
    /// kept line for the rest of its way, and joined onto it; the earlier track is untouched; the order
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

    /// Protects: a track is never cut onto a line that does not carry it on to
    /// its destination (verifier finding, 2026-11-12: a village pin 3 cells from
    /// a highway running the other way became a stub onto it, and its walk to
    /// the hamlet it was routed to went from 6.1 cells to 94.1). Fixture: a road
    /// along x = 10 from y 0 to 40; a village at (12, 20), 2 cells from it,
    /// whose track runs east to a hamlet at (30, 20). The track must stay
    /// whole. Control: the same village with a track that runs along the road
    /// to a town on it, (12, 20) to (12, 38) to (10, 40), is cut at its pin
    /// (a two-point track onto the road), so the rule did not just switch the
    /// join off.
    #[test]
    fn a_track_is_not_cut_onto_a_line_running_away_from_its_destination() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(vec![(10.0, 0.0), (10.0, 20.0), (10.0, 40.0)], crate::WayType::Road, 0, 1);
        let away = way(vec![(12.0, 20.0), (20.0, 20.0), (30.0, 20.0)], crate::WayType::Ancient, 2, 3);
        let mut ways = vec![road.clone(), away.clone()];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[1], away, "a road running across the track's way does not take it east");
        let along = way(vec![(12.0, 20.0), (12.0, 38.0), (10.0, 40.0)], crate::WayType::Ancient, 2, 1);
        let mut ways = vec![road, along];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[1].pts.len(), 2, "{:?}", ways[1].pts);
        assert_eq!(ways[1].pts[0], (12.0, 20.0), "the village end never moves");
        let end = ways[1].pts[1];
        assert!((end.0 - 10.0).abs() < 1e-9 && (end.1 - 20.0).abs() < 1e-9, "joined onto the road: {end:?}");
        assert!(!ways[1].hidden);
    }

    /// Protects: a line that passes near a track's destination but never meets
    /// the lines that reach it does not take the track (the verifier's case on
    /// the 256 x 192 world of seed 12345: a village 2 cells from a highway,
    /// routed to a hamlet 3 cells from the same highway, became a stub onto it,
    /// and its walk to the hamlet along the drawn ways went from 6 cells to
    /// about 100). Fixture: a road along x = 110; a hamlet at (107, 93) whose
    /// own earlier track runs west, (107, 93) to (95, 93); a village at
    /// (112, 97) routed to the hamlet: (112, 97), (110.5, 95), (108, 93.5),
    /// (107, 93). The whole track lies within 3 cells of the road or the
    /// hamlet's track, but the road meets neither the hamlet nor its track, so
    /// the track is cut only where the hamlet's track carries it: at
    /// (108, 93.5), onto that track.
    #[test]
    fn a_line_that_never_meets_the_destination_does_not_take_the_track() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(vec![(110.0, 60.0), (110.0, 93.0), (110.0, 130.0)], crate::WayType::Road, 0, 1);
        let hamlet = way(vec![(107.0, 93.0), (101.0, 93.0), (95.0, 93.0)], crate::WayType::Ancient, 2, 3);
        let village = way(vec![(112.0, 97.0), (110.5, 95.0), (108.0, 93.5), (107.0, 93.0)], crate::WayType::Ancient, 4, 2);
        let mut ways = vec![road, hamlet.clone(), village];
        civ_join_tracks_to_network(&mut ways, 256);
        assert_eq!(ways[1], hamlet, "premise: the hamlet's own track meets no line and stays whole");
        assert_eq!(ways[2].pts.len(), 3, "cut at (108, 93.5): {:?}", ways[2].pts);
        assert_eq!(ways[2].pts[1], (110.5, 95.0));
        let end = ways[2].pts[2];
        assert!((end.1 - 93.0).abs() < 1e-6 && end.0 <= 107.0 + 1e-6, "joined onto the hamlet's track: {end:?}");
        // The same with the road ending at (110, 93), 3 cells from the hamlet:
        // an end that close is still not on the hamlet (`REACH_PLACE_CELLS`,
        // 1.5), so the road still does not take the track.
        let road = way(vec![(110.0, 60.0), (110.0, 76.0), (110.0, 93.0)], crate::WayType::Road, 0, 1);
        let village = way(vec![(112.0, 97.0), (110.5, 95.0), (108.0, 93.5), (107.0, 93.0)], crate::WayType::Ancient, 4, 2);
        let mut ways = vec![road, hamlet, village];
        civ_join_tracks_to_network(&mut ways, 256);
        assert_eq!(ways[2].pts.len(), 3, "cut at (108, 93.5): {:?}", ways[2].pts);
        assert_eq!(ways[2].pts[1], (110.5, 95.0));
    }

    /// Protects: the chain may pass from one line to another where the two
    /// meet, so a track beside two roads that join is taken by both. Fixture:
    /// road A along y = 10 from x 0 to 30; road B from A's end (30, 10) south to
    /// a town at (30, 40); a track from a village at (5, 25) to (5, 11.5), beside
    /// A to (28.5, 11.5), beside B to (28.5, 38), into the town. It is cut at
    /// (5, 11.5), onto A. Were lines not allowed to hand over where they meet,
    /// the cut would be at (28.5, 11.5), leaving a track beside A.
    #[test]
    fn a_track_beside_two_roads_that_meet_is_taken_by_both() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let a = way(vec![(0.0, 10.0), (15.0, 10.0), (30.0, 10.0)], crate::WayType::Road, 0, 1);
        let b = way(vec![(30.0, 10.0), (30.0, 25.0), (30.0, 40.0)], crate::WayType::Road, 1, 2);
        let track = way(
            vec![(5.0, 25.0), (5.0, 11.5), (28.5, 11.5), (28.5, 38.0), (30.0, 40.0)],
            crate::WayType::Ancient, 3, 2,
        );
        let mut ways = vec![a, b, track];
        civ_join_tracks_to_network(&mut ways, 128);
        assert_eq!(ways[2].pts.len(), 2, "cut at (5, 11.5): {:?}", ways[2].pts);
        let end = ways[2].pts[1];
        assert!((end.0 - 5.0).abs() < 1e-6 && (end.1 - 10.0).abs() < 1e-6, "joined onto road A: {end:?}");
    }

    /// Protects: the rest of a track is judged along each segment, not only at
    /// its control points. Fixture: a road along y = 10 that dips to (30, 20)
    /// between x 20 and 40, to a town at (60, 10); a track from a village at
    /// (20, 30) to (20, 11.5), then straight across the dip to (40, 11.5)
    /// (both ends 1.5 cells off the road, its middle about 6 off), then beside
    /// the road to the town. It is cut at (40, 11.5), where the road starts to
    /// carry it, not at (20, 11.5): the 20-cell shortcut leaves the road.
    #[test]
    fn a_track_that_cuts_across_a_bend_is_cut_after_the_bend() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(
            vec![(0.0, 10.0), (20.0, 10.0), (30.0, 20.0), (40.0, 10.0), (60.0, 10.0)],
            crate::WayType::Road, 0, 1,
        );
        let track = way(
            vec![(20.0, 30.0), (20.0, 11.5), (40.0, 11.5), (60.0, 11.5), (60.0, 10.0)],
            crate::WayType::Ancient, 2, 1,
        );
        let mut ways = vec![road, track];
        civ_join_tracks_to_network(&mut ways, 128);
        assert_eq!(ways[1].pts.len(), 3, "cut at (40, 11.5): {:?}", ways[1].pts);
        assert_eq!(ways[1].pts[1], (20.0, 11.5));
        let end = ways[1].pts[2];
        assert!((end.0 - 40.0).hypot(end.1 - 10.0) < 1.5, "joined onto the road: {end:?}");
    }

    /// Protects: a pen-up segment (a gap the track does not draw: the jump
    /// across a wrapping world's seam) is judged by its two ends, not by the
    /// straight line between them across the map. Fixture, a 64-wide world: a
    /// road along y = 10 from x 58 to 63, a seam break, then x 0 to 5 (a town
    /// at (5, 10)); a track from a village at (58, 25) to (58, 11.5), beside the
    /// road to (63, 11.5), a seam break, then (0, 11.5) and the town. It is cut
    /// at (58, 11.5), where the road carries the rest of it across the seam.
    #[test]
    fn a_track_crossing_the_seam_beside_a_road_is_cut_before_it() {
        let road = Way {
            tid: 0, pts: vec![(58.0, 10.0), (63.0, 10.0), (0.0, 10.0), (5.0, 10.0)], brks: vec![2], km: 10.0,
            name: String::new(), way_type: crate::WayType::Road, a_idx: 0, b_idx: 1, hidden: false,
        };
        let track = Way {
            tid: 0, pts: vec![(58.0, 25.0), (58.0, 11.5), (63.0, 11.5), (0.0, 11.5), (5.0, 10.0)], brks: vec![3],
            km: 10.0, name: String::new(), way_type: crate::WayType::Ancient, a_idx: 2, b_idx: 1, hidden: false,
        };
        let mut ways = vec![road.clone(), track];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[1].pts.len(), 2, "cut at (58, 11.5): {:?}", ways[1].pts);
        assert!(ways[1].brks.is_empty(), "the seam break went with the cut: {:?}", ways[1].brks);
        assert_eq!(ways[1].pts[1], (58.0, 10.0));
        // A pen-up segment whose near end is far from every line stops the walk:
        // a track that reaches the seam at (63, 20), 10 cells off the road, is
        // cut only on the far side, at (0, 11.5), and keeps its seam break.
        let far = Way {
            tid: 0, pts: vec![(58.0, 25.0), (63.0, 20.0), (0.0, 11.5), (5.0, 10.0)], brks: vec![2], km: 10.0,
            name: String::new(), way_type: crate::WayType::Ancient, a_idx: 2, b_idx: 1, hidden: false,
        };
        let mut ways = vec![road, far];
        civ_join_tracks_to_network(&mut ways, 64);
        assert_eq!(ways[1].pts.len(), 3, "cut at (0, 11.5): {:?}", ways[1].pts);
        assert_eq!(ways[1].brks, vec![2]);
        assert_eq!(ways[1].pts[2], (0.0, 10.0));
    }

    /// Protects: a village whose pin is exactly on a line that carries its whole
    /// track is hidden (there is nothing left to draw), not kept as a zero-length
    /// stroke. Fixture: a road along y = 10 from x 0 to 40 (town at (40, 10)); a
    /// village on it at (20, 10) whose track runs along it to the town.
    #[test]
    fn a_track_whose_village_is_on_the_line_is_hidden() {
        let way = |pts: Vec<(f64, f64)>, t, a, b| Way {
            tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false,
        };
        let road = way(vec![(0.0, 10.0), (20.0, 10.0), (40.0, 10.0)], crate::WayType::Road, 0, 1);
        let on = way(vec![(20.0, 10.0), (30.0, 10.0), (40.0, 10.0)], crate::WayType::Ancient, 2, 1);
        let mut ways = vec![road, on];
        civ_join_tracks_to_network(&mut ways, 64);
        assert!(ways[1].hidden, "{:?}", ways[1]);
    }

    /// Protects: passes after the first take the edge being re-traced out of
    /// the laid cells, so it is drawn only to the *other* edges' cells (the
    /// reason for `PASSES`). Fixture, on a flat 120 x 24 grid: a town-town edge
    /// along row 8 from x 20 to 100 is laid first (tier sum 6); four short
    /// hamlet-hamlet edges (sum 2) along row 13, x 20-40, 40-60, 60-80 and
    /// 80-100, are laid after it. Each hamlet edge is too short to climb five
    /// rows to the town road within the 15 % guard, so after pass 1 row 13
    /// carries a parallel line. In pass 2 the town road, re-traced against
    /// the hamlet edges only, moves onto row 13. Were its own row-8 cells still
    /// counted as laid it would stay on them (they are discounted as much as
    /// row 13 and need no climb), which is what this test catches. Control:
    /// with one pass the town road stays on row 8.
    #[test]
    fn a_later_pass_moves_an_early_edge_onto_a_trunk_laid_after_it() {
        let (w, h) = (120, 24);
        let field = vec![0.6f32; w * h];
        let zeros_f = vec![0.0f32; w * h];
        let zeros_o = vec![0i16; w * h];
        let zeros_b = vec![7u8; w * h];
        let zeros_w = vec![0u8; w * h];
        let place = |x, y, kind| SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind, coastal: false };
        let mut places = vec![place(20, 8, SettlementKind::Town), place(100, 8, SettlementKind::Town)];
        for x in [20, 40, 60, 80, 100] {
            places.push(place(x, 13, SettlementKind::Hamlet));
        }
        let mut edges = vec![RoadEdge { a: 0, b: 1, path: row(8, 20, 100, w) }];
        for k in 0..4 {
            edges.push(RoadEdge { a: 2 + k, b: 3 + k, path: row(13, 20 + 20 * k, 40 + 20 * k, w) });
        }
        // Settlement gravity reads `places`: only the two towns are given, so the
        // hamlets do not bend the routes towards themselves and the fixture tests
        // the pass mechanics alone; the tiers still come from all seven (`kinds`).
        let kinds: Vec<SettlementKind> = places.iter().map(|p| p.kind).collect();
        let run = |passes: usize| {
            let mut topo = HierarchicalNetworkResult { edges: edges.clone(), usage_count: vec![0; w * h], degree_of: vec![1; 7] };
            let rep = civ_merge_network_paths(
                &mut topo, &places[..2], &kinds, w, h, 0.4, &field, &zeros_f, &zeros_o, &zeros_b, &zeros_w, false, 600.0,
                &WayMergeOpts { passes, ..WayMergeOpts::default() },
            );
            (topo, rep)
        };
        let (one, _) = run(1);
        let near_town_road = |t: &HierarchicalNetworkResult| {
            t.edges[1..].iter().flat_map(|e| e.path.iter()).filter(|&&c| c / w <= 10).count()
        };
        assert_eq!(near_town_road(&one), 0, "premise: no hamlet edge climbs towards the town road in pass 1");
        assert_eq!(one.edges[0].path, edges[0].path, "control: with one pass the town road keeps row 8");
        let (three, rep3) = run(3);
        assert!(rep3.rerouted[0], "the town road moves in a later pass");
        let on13 = three.edges[0].path.iter().filter(|&&c| c / w == 13).count();
        assert!(on13 >= 60, "only {on13} of {} cells on row 13", three.edges[0].path.len());
        assert!(rep3.kept_cost[0] <= rep3.original_cost[0] * (1.0 + DETOUR_TOL));
    }

    /// Protects: the recount never drops a cell below the count the topology
    /// came in with (verifier finding, 2026-11-12: on the owner's world the one
    /// cell used by 8 edges fell to 7 and every highway drew as a regional
    /// road). Fixture: one hamlet-hamlet edge along row 5 of a flat 40 x 12 grid
    /// whose incoming `usage_count` says 8 edges used its middle cell (the
    /// reference network's count, which this lone edge cannot reproduce). After
    /// the merge that cell still reads 8 (literal), so `civ_classify_way` still
    /// makes the edge a highway; the next cell, which came in at 0, reads 1.
    #[test]
    fn the_recount_keeps_the_reference_networks_traffic_as_a_floor() {
        let (w, h) = (40, 12);
        let field = vec![0.6f32; w * h];
        let zeros_f = vec![0.0f32; w * h];
        let zeros_o = vec![0i16; w * h];
        let zeros_b = vec![7u8; w * h];
        let zeros_w = vec![0u8; w * h];
        let place = |x, y| SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Hamlet, coastal: false };
        let places = vec![place(4, 5), place(35, 5)];
        let path = row(5, 4, 35, w);
        let mid = path[15];
        let mut usage = vec![0u16; w * h];
        usage[mid] = 8;
        let mut topo = HierarchicalNetworkResult { edges: vec![RoadEdge { a: 0, b: 1, path }], usage_count: usage, degree_of: vec![1; 2] };
        let kinds = [SettlementKind::Hamlet, SettlementKind::Hamlet];
        civ_merge_network_paths(
            &mut topo, &places, &kinds, w, h, 0.4, &field, &zeros_f, &zeros_o, &zeros_b, &zeros_w, false, 600.0,
            &WayMergeOpts::default(),
        );
        assert!(topo.edges[0].path.contains(&mid), "premise: the lone edge keeps its straight row");
        assert_eq!(topo.usage_count[mid], 8);
        assert_eq!(topo.usage_count[mid + 1], 1);
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

    fn gap_way(pts: Vec<(f64, f64)>, t: crate::WayType, a: usize, b: usize) -> Way {
        Way { tid: 0, pts, brks: Vec::new(), km: 10.0, name: String::new(), way_type: t, a_idx: a, b_idx: b, hidden: false }
    }

    /// Runs the branch-and-trunk fixture shared by the junction tests.
    /// Trunk along y = 10 (x 0..40, town at (40, 10)); a branch road along
    /// x = 20 from y 40 down to `(20, 10 + gap)`; a track from a village at
    /// (21.5, 38) beside the branch, then beside the trunk, into the town.
    /// `branch_first` lists the branch BEFORE the trunk. Returns the track's points.
    fn branch_trunk_track(gap: f64, branch_first: bool) -> Vec<(f64, f64)> {
        let trunk = gap_way(vec![(0.0, 10.0), (20.0, 10.0), (40.0, 10.0)], crate::WayType::Road, 0, 1);
        let branch = gap_way(vec![(20.0, 40.0), (20.0, 20.0), (20.0, 10.0 + gap)], crate::WayType::Road, 2, 0);
        let track = gap_way(
            vec![(21.5, 38.0), (21.5, 11.5), (38.0, 11.5), (40.0, 10.0)],
            crate::WayType::Ancient, 3, 1,
        );
        let mut ways = if branch_first { vec![branch, trunk, track] } else { vec![trunk, branch, track] };
        civ_join_tracks_to_network(&mut ways, 128);
        ways[2].pts.clone()
    }

    /// Protects: `record_meets` records a junction whichever line is listed
    /// first, i.e. the SECOND half (the earlier line's end on the later line's
    /// curve). Fixture: a branch listed BEFORE the trunk it ends on, and a track
    /// beside the branch whose whole length is carried only through that junction
    /// (branch -> trunk -> town); it must be cut at its village (two points,
    /// onto the branch). Without the second half the branch never joins the
    /// chain and the cut falls at the corner (21.5, 11.5) instead. Control: the
    /// forward order gives the same cut.
    #[test]
    fn a_branch_listed_before_its_trunk_still_meets_it() {
        for first in [true, false] {
            let pts = branch_trunk_track(0.0, first);
            assert_eq!(pts.len(), 2, "branch_first={first}: {pts:?}");
            assert_eq!(pts[0], (21.5, 38.0), "the village end never moves");
            assert!((pts[1].0 - 20.0).abs() < 1e-9 && (pts[1].1 - 38.0).abs() < 1e-9, "onto the branch: {:?}", pts[1]);
        }
    }

    /// Protects: the lower side of `MEET_CELLS` -- a junction whose end is just
    /// under half a cell off the trunk (float noise from the snap pass, or a
    /// slightly short branch) is still a junction (0.49 off), while one just over
    /// (0.51) is not. Lowering the reach to 0 would drop the first; raising it
    /// the second. Same fixture as above.
    #[test]
    fn a_junction_just_under_half_a_cell_off_still_meets() {
        let met = branch_trunk_track(0.49, true);
        assert_eq!(met.len(), 2, "0.49 cells off is a junction: {met:?}");
        assert!((met[1].0 - 20.0).abs() < 1e-9 && (met[1].1 - 38.0).abs() < 1e-9, "onto the branch: {met:?}");
        let missed = branch_trunk_track(0.51, true);
        assert_eq!(missed.len(), 2, "{missed:?}");
        assert!((missed[1].1 - 10.0).abs() < 1e-9, "0.51 cells off is no junction, cut onto the trunk at the corner: {missed:?}");
    }

    /// Protects: of the chain lines near a cut, the NEAREST takes the track.
    /// Fixture: roads A (y = 10) and B (y = 13, bending to the town), both
    /// ending at the town (40, 10), and a track from (10, 11), 1 cell off A and
    /// 2 off B, to the town: it must be joined onto A at (10, 10). Picking the
    /// farthest would join B at (10, 13).
    #[test]
    fn a_track_joins_the_nearer_of_two_chain_lines() {
        let a = gap_way(vec![(0.0, 10.0), (20.0, 10.0), (40.0, 10.0)], crate::WayType::Road, 0, 1);
        let b = gap_way(vec![(0.0, 13.0), (30.0, 13.0), (40.0, 10.0)], crate::WayType::Road, 2, 1);
        let track = gap_way(vec![(10.0, 11.0), (25.0, 11.0), (40.0, 10.0)], crate::WayType::Ancient, 3, 1);
        let mut ways = vec![a, b, track];
        civ_join_tracks_to_network(&mut ways, 128);
        assert_eq!(ways[2].pts.len(), 2, "{:?}", ways[2].pts);
        let end = ways[2].pts[1];
        assert!((end.0 - 10.0).abs() < 1e-9 && (end.1 - 10.0).abs() < 1e-9, "joined onto the nearer road A: {end:?}");
    }

    /// Protects: the lower side of `REACH_PLACE_CELLS` -- a road whose end stops
    /// 1.0 cell short of the town (its curve is not within `MEET_CELLS` of it)
    /// still reaches the town, so a track beside it is joined. Fixture: a road
    /// along y = 10 from x 0 to 39, a town at (40, 10), a track from (10, 12)
    /// beside the road into the town. Were the reach under 1 cell, no line would
    /// reach the destination and the track would stay whole (3 points).
    #[test]
    fn a_road_ending_a_cell_short_of_the_town_still_reaches_it() {
        let road = gap_way(vec![(0.0, 10.0), (20.0, 10.0), (39.0, 10.0)], crate::WayType::Road, 0, 1);
        let track = gap_way(vec![(10.0, 12.0), (30.0, 11.5), (40.0, 10.0)], crate::WayType::Ancient, 2, 1);
        let mut ways = vec![road, track];
        civ_join_tracks_to_network(&mut ways, 128);
        assert_eq!(ways[1].pts.len(), 2, "{:?}", ways[1].pts);
        assert!((ways[1].pts[1].1 - 10.0).abs() < 1e-9, "joined onto the road: {:?}", ways[1].pts[1]);
    }
}
