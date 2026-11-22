//! Sea lanes that call at ports, hug a coast where that is no great detour, and
//! share trunks (owner findings F16 and F39, 2026-11-21; `civ.sea_lanes`).
//!
//! **What shipped before.** `civ_sea_routes` picks lane *pairs* (a Prim tree plus
//! each port's nearest neighbour) and draws every pair's own least-time path. Two
//! consequences, both seen on the owner's seed 83744 (2048 x 1024, 200 x 100 km):
//! (1) F39: the least-time path of a long pair ignores a port it sails right past,
//! so a lane "skips" a coastal settlement; and every open-water cell costs the
//! same, so nothing keeps a lane near the shore. (2) F16: every pair is traced
//! alone, so lanes heading the same way run as a bundle of parallel wires, the
//! thing `way_merge` fixed for roads (Ruling EU item 1).
//!
//! **What this does, in order.** (a) *Calls*: a lane whose path passes within
//! [`CALL_RADIUS_CELLS`] of a port that is not one of its ends is split at that
//! port (a -> p -> b), the cabotage pattern: a ship passing a harbour on its way
//! calls there. (b) *Coast*: the water within [`COAST_BAND_CELLS`] of land costs
//! [`COAST_FACTOR`] of open water, so a lane follows a shore where that is a
//! modest detour. (c) *Trunks*: every lane is re-traced [`PASSES`] times, longest
//! trade first, with the cells earlier lanes use costing [`REUSE_FACTOR`], the
//! same trail-reinforcement `way_merge` uses (Helbing, Keltsch and Moln\u{e1}r 1997).
//!
//! **What it must never do.** Cross land (the search reads the same cost grid,
//! where land is infinite); price a lane worse than [`DETOUR_TOL`] over the pair's
//! own least-time path under the unchanged wind and current cost (the guard keeps
//! the original instead); change which pairs exist beyond splitting at calls; or
//! run when `civ.sea_lanes` is off, so every golden keeps the reference's lanes.

use crate::way_merge::{Key, Window, window_for};
use crate::civ_trace_path;
use std::collections::{BinaryHeap, HashSet, VecDeque};

/// A lane that passes within this many routing cells of a port that is not an
/// end calls there. A labelled judgement: about the radius of a harbour's own
/// water (the port is snapped to the nearest ocean cell within 10), and small
/// enough that a lane across open water is not pulled into a distant bay.
pub const CALL_RADIUS_CELLS: f64 = 6.0;

/// Water within this many routing cells of land is "coastal". A labelled
/// judgement: coasting is sailing in sight of land, a few cells of the 1024-wide
/// routing grid at most.
pub const COAST_BAND_CELLS: u32 = 3;

/// The cost of coastal water relative to open water: a lane may take a coast
/// path up to about 1 / 0.85 = 18 % longer in sailing time before open water
/// wins. A labelled judgement (cabotage was the dominant medieval trade pattern),
/// bounded by [`DETOUR_TOL`]; swept in `STATUS.md` (2026-11-22).
pub const COAST_FACTOR: f64 = 0.85;

/// Cost multiplier on a cell an earlier lane already uses: `way_merge`'s own
/// value, the reference's `_CIV_EXISTING_WAY_DISCOUNT`.
pub const REUSE_FACTOR: f64 = crate::way_merge::REUSE_FACTOR;

/// How much worse than the pair's own least-time path (same cost, no coast or
/// reuse discount) a re-traced lane may be and still be kept. A labelled
/// judgement, looser than the roads' 0.15 because coasting is a deliberate
/// detour; the census reports the realised detour.
pub const DETOUR_TOL: f64 = 0.25;

/// Re-trace passes, as `way_merge::PASSES`.
pub const PASSES: usize = 3;

/// Inputs for [`lane_paths`], all on the routing grid.
pub struct LaneIn<'a> {
    /// Navigable-ocean cost grid: finite = sea, infinite = land.
    pub cost: &'a [f32],
    pub rw: usize,
    pub rh: usize,
    pub world: bool,
    /// Each port's routing cell (snapped onto the sea).
    pub rp: &'a [usize],
    /// Each port's Dijkstra `prev` tree (the port is the source).
    pub prev: &'a [&'a [i32]],
    /// Each port's weight (population), for the trunk-laying order.
    pub weight: &'a [f64],
    /// The unchanged sailing-time step (wind and current), infinite on land.
    pub base: &'a (dyn Fn(usize, usize, isize, isize) -> f64 + Sync),
}

/// The signed step from cell `a` to the adjacent cell `b`, wrap-aware.
fn delta(a: usize, b: usize, rw: usize, world: bool) -> (isize, isize) {
    let mut dx = (b % rw) as isize - (a % rw) as isize;
    if world && dx.unsigned_abs() > rw / 2 {
        dx -= dx.signum() * rw as isize;
    }
    (dx, (b / rw) as isize - (a / rw) as isize)
}

fn path_base_cost(p: &[usize], inp: &LaneIn) -> f64 {
    p.windows(2)
        .map(|w| {
            let (dx, dy) = delta(w[0], w[1], inp.rw, inp.world);
            (inp.base)(w[0], w[1], dx, dy)
        })
        .sum()
}

/// Per routing cell, [`COAST_FACTOR`] where the cell is sea within
/// [`COAST_BAND_CELLS`] of land, else 1. A multi-source 8-neighbour BFS from every
/// land cell.
pub fn coast_factor(cost: &[f32], rw: usize, rh: usize, world: bool) -> Vec<f32> {
    let mut d = vec![u32::MAX; rw * rh];
    let mut q = VecDeque::new();
    for (i, c) in cost.iter().enumerate() {
        if !c.is_finite() {
            d[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        if d[i] >= COAST_BAND_CELLS {
            continue;
        }
        let (x, y) = ((i % rw) as isize, (i / rw) as isize);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = if world { (x + dx).rem_euclid(rw as isize) } else { x + dx };
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= rw as isize || ny >= rh as isize {
                    continue;
                }
                let j = ny as usize * rw + nx as usize;
                if d[j] == u32::MAX {
                    d[j] = d[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    d.iter().map(|&v| if v != 0 && v <= COAST_BAND_CELLS { COAST_FACTOR as f32 } else { 1.0 }).collect()
}

/// The ports (not `a`, not `b`; a port on an end's own cell counts as that end) a routing-cell `path` passes within
/// [`CALL_RADIUS_CELLS`] of, in path order, each once.
fn calls(path: &[usize], a: usize, b: usize, inp: &LaneIn) -> Vec<usize> {
    let rw = inp.rw;
    let mut hit: Vec<(usize, usize)> = Vec::new();
    for (p, &rc) in inp.rp.iter().enumerate() {
        if rc == inp.rp[a] || rc == inp.rp[b] {
            continue;
        }
        let (px, py) = ((rc % rw) as f64, (rc / rw) as f64);
        let mut best: Option<(f64, usize)> = None;
        for (k, &c) in path.iter().enumerate() {
            let mut dx = ((c % rw) as f64 - px).abs();
            if inp.world {
                dx = dx.min(rw as f64 - dx);
            }
            let dy = (c / rw) as f64 - py;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= CALL_RADIUS_CELLS && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, k));
            }
        }
        if let Some((_, k)) = best {
            hit.push((k, p));
        }
    }
    hit.sort();
    hit.into_iter().map(|(_, p)| p).collect()
}

/// Windowed least-cost search from `s` to `t`: the step is the unchanged
/// sailing-time step times the mean of the two cells' multipliers (coast, and
/// [`REUSE_FACTOR`] on cells in `laid`).
fn search(inp: &LaneIn, coast: &[f32], laid: &[u16], win: Window, s: usize, t: usize) -> Option<Vec<usize>> {
    let rw = inp.rw;
    let n = win.w * win.h;
    let local = |g: usize| (g / rw - win.y0) * win.w + (g % rw - win.x0);
    let global = |l: usize| (win.y0 + l / win.w) * rw + win.x0 + l % win.w;
    let mult = |g: usize| -> f64 {
        let m = coast[g] as f64;
        if laid[g] > 0 { m * REUSE_FACTOR } else { m }
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
                let gj = global(j);
                let step = (inp.base)(gi, gj, dx, dy);
                if !step.is_finite() {
                    continue;
                }
                let nd = d + step * 0.5 * (mult(gi) + mult(gj));
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

/// The lanes for `edges` (port pairs, source first) as routing-cell paths
/// source..=target: split at calls, then re-traced onto coasts and shared trunks.
/// A pair whose path cannot be traced is dropped, as the reference's loop drops it.
pub fn lane_paths(edges: &[(usize, usize)], inp: &LaneIn) -> Vec<(usize, usize, Vec<usize>)> {
    let trace = |x: usize, y: usize| civ_trace_path(inp.prev[x], inp.rp[x], inp.rp[y]);
    // (a) calls.
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let mut push = |x: usize, y: usize, pairs: &mut Vec<(usize, usize)>| {
        if seen.insert((x.min(y), x.max(y))) {
            pairs.push((x, y));
        }
    };
    for &(a, b) in edges {
        let p = trace(a, b);
        let mut chain = vec![a];
        chain.extend(calls(&p, a, b, inp));
        chain.push(b);
        for w in chain.windows(2) {
            push(w[0], w[1], &mut pairs);
        }
    }
    let mut lanes: Vec<(usize, usize, Vec<usize>)> = pairs
        .into_iter()
        .map(|(a, b)| (a, b, trace(a, b)))
        .filter(|(a, b, p)| p.len() >= 2 && p[0] == inp.rp[*a] && p[p.len() - 1] == inp.rp[*b])
        .collect();
    // (b) + (c): coast cost and trunk reuse, busiest trade first.
    let coast = coast_factor(inp.cost, inp.rw, inp.rh, inp.world);
    let orig: Vec<f64> = lanes.iter().map(|l| path_base_cost(&l.2, inp)).collect();
    let mut order: Vec<usize> = (0..lanes.len()).collect();
    order.sort_by(|&i, &j| {
        let w = |k: usize| inp.weight[lanes[k].0] + inp.weight[lanes[k].1];
        w(j).total_cmp(&w(i))
    });
    let mut laid = vec![0u16; inp.rw * inp.rh];
    for pass in 0..PASSES {
        for &k in &order {
            if pass > 0 {
                for &c in &lanes[k].2 {
                    laid[c] = laid[c].saturating_sub(1);
                }
            }
            let win = window_for(&lanes[k].2, inp.rw, inp.rh, inp.world);
            let (s, t) = (lanes[k].2[0], *lanes[k].2.last().unwrap());
            if let Some(p) = search(inp, &coast, &laid, win, s, t)
                && path_base_cost(&p, inp) <= orig[k] * (1.0 + DETOUR_TOL)
            {
                lanes[k].2 = p;
            }
            for &c in &lanes[k].2 {
                laid[c] = laid[c].saturating_add(1);
            }
        }
    }
    lanes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w` x `h` sea whose top `land_rows` rows are land, with ports at the
    /// given `(x, y)` cells, and the lanes for `edges` from [`lane_paths`].
    fn lanes(w: usize, h: usize, land_rows: usize, ports: &[(usize, usize)], edges: &[(usize, usize)]) -> Vec<(usize, usize, Vec<usize>)> {
        let cost: Vec<f32> = (0..w * h).map(|i| if i / w < land_rows { f32::INFINITY } else { 1.0 }).collect();
        let rp: Vec<usize> = ports.iter().map(|&(x, y)| y * w + x).collect();
        let prevs: Vec<Vec<i32>> = rp.iter().map(|&r| crate::road_dijkstra(&cost, w, h, r % w, r / w, false, None, true).1).collect();
        let prev: Vec<&[i32]> = prevs.iter().map(|v| v.as_slice()).collect();
        let weight = vec![1.0; ports.len()];
        let c = cost.clone();
        let base = move |i: usize, j: usize, dx: isize, dy: isize| -> f64 {
            if !c[i].is_finite() || !c[j].is_finite() { f64::INFINITY } else if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 }
        };
        lane_paths(edges, &LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &rp, prev: &prev, weight: &weight, base: &base })
    }

    /// Protects F39: a lane that sails within the call radius of a port that is
    /// not an end is split there (a -> p -> b); a port well off the lane is not.
    /// Mutating `CALL_RADIUS_CELLS` to 0.5 or 60.0 turns it red.
    #[test]
    fn a_lane_calls_at_a_port_it_passes_and_not_at_a_far_one() {
        let ports = [(2, 6), (37, 6), (20, 9), (20, 30)];
        let l = lanes(40, 34, 0, &ports, &[(0, 1)]);
        let pairs: Vec<(usize, usize)> = l.iter().map(|x| (x.0.min(x.1), x.0.max(x.1))).collect();
        assert_eq!(pairs, vec![(0, 2), (1, 2)], "a -> p -> b, the far port left out");
    }

    /// Protects F16: a lane seven rows (just outside the call radius) from an earlier one joins its cells
    /// (trail reinforcement). Mutating `REUSE_FACTOR` to 1.0 turns it red.
    #[test]
    fn a_parallel_lane_joins_the_trunk() {
        let ports = [(2, 10), (87, 10), (2, 17), (87, 17)];
        let l = lanes(90, 28, 0, &ports, &[(0, 1), (2, 3)]);
        let trunk: std::collections::HashSet<usize> = l[0].2.iter().copied().collect();
        let shared = l[1].2.iter().filter(|c| trunk.contains(c)).count();
        assert!(shared >= 60, "shared cells {shared}");
    }

    /// Protects F39's coast half and the detour guard together: ports 4 rows off a
    /// coast (land rows 0 and 1, coastal water rows 2-4) take the coastal row, a
    /// small detour. Mutating `COAST_FACTOR` to 1.0 (no pull) or `DETOUR_TOL` to
    /// 0.0 (the detour refused) leaves the lane on row 5 and turns it red.
    #[test]
    fn a_lane_follows_the_coast_when_the_detour_is_small() {
        let l = lanes(40, 12, 2, &[(2, 5), (37, 5)], &[(0, 1)]);
        let on_coast = l[0].2.iter().filter(|&&c| c / 40 <= 4).count();
        assert!(on_coast * 2 > l[0].2.len(), "{on_coast} of {} cells on the coast", l[0].2.len());
    }

    /// Never crosses land: with a land wall between, every lane cell is sea.
    #[test]
    fn a_lane_never_crosses_land() {
        let l = lanes(40, 12, 2, &[(2, 5), (37, 5)], &[(0, 1)]);
        assert!(l[0].2.iter().all(|&c| c / 40 >= 2));
    }
}
