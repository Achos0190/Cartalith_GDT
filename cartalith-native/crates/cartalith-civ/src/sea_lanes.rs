//! Sea lanes that call at ports, stop running side by side and share trunks
//! (owner findings F16 and F39, 2026-11-21, and Rulings GR and GS, 2026-11-22;
//! `civ.sea_lanes`).
//!
//! **What shipped before.** `civ_sea_routes` picks lane *pairs* (a Prim tree plus
//! each port's nearest neighbour) and draws every pair's own least-time path. Two
//! consequences, both seen on the owner's seed 83744 (2048 x 1024, 200 x 100 km):
//! (1) F39: the least-time path of a long pair ignores a port it sails right past,
//! so a lane "skips" a coastal settlement, and a big arc swings far out to sea.
//! (2) F16: every pair is traced alone, so lanes heading the same way run as a
//! bundle of parallel wires, the thing `way_merge` fixed for roads (Ruling EU
//! item 1).
//!
//! **What this does, in order.** (a) *Calls*: a lane whose path passes within
//! [`CALL_RADIUS_CELLS`] of a port that is not one of its ends is split at that
//! port (a -> p -> b), the cabotage pattern: a ship passing a harbour on its way
//! calls there. (b) *Trunks*: every lane is re-traced [`PASSES`] times, busiest
//! ports first, with the cells earlier lanes use costing [`REUSE_FACTOR`], the
//! same trail-reinforcement `way_merge` uses (Helbing, Keltsch and Moln\u{e1}r
//! 1997), but only where the cell is at least [`REUSE_OFFSHORE_SHARE`] of the
//! lane's own median distance from land (Ruling GU), so a long lane is not drawn
//! onto a coastal trunk. (c) *Threading*: a cell beside another lane costs
//! [`THREAD_FACTOR`] times, and [`snap_parallel`] merges near-parallel lanes after
//! smoothing.
//!
//! **Distance from the coast is left as it was** (owner, Ruling GS, 2026-11-22:
//! *"just keep it as it originally was for the distance from the coast"*). The
//! first build pulled lanes onto the coast (a 0.85 discount) and the second kept
//! them off it (a 2.5 km standoff cost); a third, a swing cost on cells far from a
//! lane's own chord, also pulled the seed-83744 arc onto the coast (Ruling GU: *"keep
//! the long lane's own open-sea course"*); the owner rejected all three, so no term
//! exists: no cell is priced for being near land.
//!
//! **What it must never do.** Cross land (the search reads the same cost grid,
//! where land is infinite); price a lane worse than [`DETOUR_TOL`] over the pair's
//! own least-time path under the unchanged wind and current cost (the guard keeps
//! the original instead); attract a lane toward, or push it away from, a coast;
//! change which pairs exist beyond splitting at calls; or run when
//! `civ.sea_lanes` is off, so every golden keeps the reference's lanes.

use crate::way_merge::{Key, Window, window_for};
use crate::civ_trace_path;
use std::collections::{BinaryHeap, HashSet};

/// A lane that passes within this many routing cells of a port that is not an
/// end calls there. A labelled judgement: about the radius of a harbour's own
/// water (the port is snapped to the nearest ocean cell within 10), and small
/// enough that a lane across open water is not pulled into a distant bay.
pub const CALL_RADIUS_CELLS: f64 = 6.0;

/// Cost multiplier on a cell an earlier lane already uses: `way_merge`'s own
/// value, the reference's `_CIV_EXISTING_WAY_DISCOUNT`.
pub const REUSE_FACTOR: f64 = crate::way_merge::REUSE_FACTOR;

/// How much worse than the pair's own least-time path (same cost, no coast or
/// reuse discount) a re-traced lane may be and still be kept. A labelled
/// judgement, looser than the roads' 0.15 because coasting is a deliberate
/// detour; the census reports the realised detour.
pub const DETOUR_TOL: f64 = 0.25;

/// A cell this many routing cells or fewer from another lane's cells, but not on
/// them, is in that lane's *thread zone* (owner, Ruling GS addendum 2026-11-22:
/// *"make sure the threading doesn't happen anymore"*): lanes running side by side
/// a few cells apart instead of sharing a trunk. A labelled judgement, the owner's
/// "a small distance" read as 1 to 4 cells.
pub const THREAD_RADIUS_CELLS: usize = 4;

/// The cost of a cell in another lane's thread zone, so a lane either joins the
/// trunk (at [`REUSE_FACTOR`]) or stays clear of it, never runs beside it. A
/// labelled judgement: doubles the cost, so only a real saving justifies riding
/// beside a lane.
pub const THREAD_FACTOR: f64 = 2.0;

/// The thread zone and the shared trunk are not applied this many routing cells
/// round a port, where lanes leaving one harbour must be close. A labelled
/// judgement, the radius the call split uses plus two.
pub const THREAD_JUNCTION_CELLS: f64 = 8.0;

/// After smoothing, a run of a lane's vertices within this many routing cells of an
/// earlier lane (and not on it) is replaced by that lane's own course (see
/// [`snap_parallel`]). A labelled judgement: each lane is smoothed alone
/// (`civ_smooth_path`, RDP tolerance 1.5 cells), so two lanes that shared their
/// cells come out 1 to 2 cells apart; 3 closes that and stays inside
/// [`THREAD_RADIUS_CELLS`].
pub const SNAP_CELLS: f64 = 3.0;

/// A lane's trunk discount applies only to cells at least this share of the lane's
/// own median distance from land (Ruling GU, owner 2026-11-22: *"keep the long
/// lane's own open-sea course"*). A long lane that sails well offshore is not
/// pulled onto a coastal trunk, because the coastal cells are closer to land than
/// half its own course and so earn no discount; a lane that already hugs the coast
/// has a small median, so nothing is withheld from it. A labelled judgement: half.
pub const REUSE_OFFSHORE_SHARE: f64 = 0.5;

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
/// sailing-time step times the mean of the two cells' multipliers
/// ([`REUSE_FACTOR`] on cells in `laid` and [`THREAD_FACTOR`] on cells in another
/// lane's `zone` away from ports, `free`).
#[allow(clippy::too_many_arguments)]
fn search(inp: &LaneIn, laid: &[u16], zone: &[u16], free: &[bool], land: &[u16], reuse_min: u16, win: Window, s: usize, t: usize) -> Option<Vec<usize>> {
    let rw = inp.rw;
    let n = win.w * win.h;
    let local = |g: usize| (g / rw - win.y0) * win.w + (g % rw - win.x0);
    let global = |l: usize| (win.y0 + l / win.w) * rw + win.x0 + l % win.w;
    let mult = |g: usize| -> f64 {
        if laid[g] > 0 && land[g] >= reuse_min {
            REUSE_FACTOR
        } else if zone[g] > 0 && !free[g] {
            THREAD_FACTOR
        } else {
            1.0
        }
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

/// Adds (`add`) or removes a lane path's cells from `laid` and its
/// [`THREAD_RADIUS_CELLS`] neighbourhood from `zone`.
fn stamp_path(laid: &mut [u16], zone: &mut [u16], path: &[usize], inp: &LaneIn, add: bool) {
    let r = THREAD_RADIUS_CELLS as isize;
    let bump = |v: &mut u16| *v = if add { v.saturating_add(1) } else { v.saturating_sub(1) };
    for &c in path {
        bump(&mut laid[c]);
        let (x, y) = ((c % inp.rw) as isize, (c / inp.rw) as isize);
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = if inp.world { (x + dx).rem_euclid(inp.rw as isize) } else { x + dx };
                let ny = y + dy;
                if nx >= 0 && ny >= 0 && nx < inp.rw as isize && ny < inp.rh as isize {
                    bump(&mut zone[ny as usize * inp.rw + nx as usize]);
                }
            }
        }
    }
}

/// The lowest land distance at which a lane's trunk discount applies: [`REUSE_OFFSHORE_SHARE`]
/// of the median land distance of its own `path`.
fn reuse_floor(path: &[usize], land: &[u16]) -> u16 {
    let mut d: Vec<u16> = path.iter().map(|&c| land[c]).collect();
    d.sort_unstable();
    (d[d.len() / 2] as f64 * REUSE_OFFSHORE_SHARE) as u16
}

/// Per routing cell, the 8-neighbour step distance to the nearest land cell (a cell
/// of infinite cost), capped at 4095. A multi-source breadth-first search; land is 0.
fn land_distance(inp: &LaneIn) -> Vec<u16> {
    let (rw, rh) = (inp.rw, inp.rh);
    let mut d = vec![u16::MAX; rw * rh];
    let mut q = std::collections::VecDeque::new();
    for (i, c) in inp.cost.iter().enumerate() {
        if !c.is_finite() {
            d[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        let (x, y) = ((i % rw) as isize, (i / rw) as isize);
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = if inp.world { (x + dx).rem_euclid(rw as isize) } else { x + dx };
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= rw as isize || ny >= rh as isize {
                    continue;
                }
                let j = ny as usize * rw + nx as usize;
                if d[j] == u16::MAX && d[i] < 4095 {
                    d[j] = d[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    // A map with no land has no distance: treat every cell as far.
    d.iter().map(|&v| if v == u16::MAX { 4095 } else { v }).collect()
}

/// Per routing cell, whether it lies within [`THREAD_JUNCTION_CELLS`] of a port.
fn junction_cells(inp: &LaneIn) -> Vec<bool> {
    let mut f = vec![false; inp.rw * inp.rh];
    let r = THREAD_JUNCTION_CELLS.ceil() as isize;
    for &p in inp.rp {
        let (px, py) = ((p % inp.rw) as isize, (p / inp.rw) as isize);
        for dy in -r..=r {
            for dx in -r..=r {
                if ((dx * dx + dy * dy) as f64).sqrt() > THREAD_JUNCTION_CELLS {
                    continue;
                }
                let nx = if inp.world { (px + dx).rem_euclid(inp.rw as isize) } else { px + dx };
                let ny = py + dy;
                if nx >= 0 && ny >= 0 && nx < inp.rw as isize && ny < inp.rh as isize {
                    f[ny as usize * inp.rw + nx as usize] = true;
                }
            }
        }
    }
    f
}

/// The lanes for `edges` (port pairs, source first) as routing-cell paths
/// source..=target: split at calls, then re-traced onto shared trunks and kept clear of each other.
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
    // (b) + (c): trunk reuse and the thread zone, busiest trade first.
    let orig: Vec<f64> = lanes.iter().map(|l| path_base_cost(&l.2, inp)).collect();
    let mut order: Vec<usize> = (0..lanes.len()).collect();
    order.sort_by(|&i, &j| {
        let w = |k: usize| inp.weight[lanes[k].0] + inp.weight[lanes[k].1];
        w(j).total_cmp(&w(i))
    });
    let mut laid = vec![0u16; inp.rw * inp.rh];
    let mut zone = vec![0u16; inp.rw * inp.rh];
    let free = junction_cells(inp);
    let land = land_distance(inp);
    // Each lane's trunk discount floor: a share of the median land distance of its own
    // original course (`REUSE_OFFSHORE_SHARE`).
    let reuse_min: Vec<u16> = lanes.iter().map(|l| reuse_floor(&l.2, &land)).collect();
    for pass in 0..PASSES {
        for &k in &order {
            if pass > 0 {
                stamp_path(&mut laid, &mut zone, &lanes[k].2, inp, false);
            }
            let win = window_for(&lanes[k].2, inp.rw, inp.rh, inp.world);
            let (s, t) = (lanes[k].2[0], *lanes[k].2.last().unwrap());
            if let Some(p) = search(inp, &laid, &zone, &free, &land, reuse_min[k], win, s, t)
                && path_base_cost(&p, inp) <= orig[k] * (1.0 + DETOUR_TOL)
            {
                lanes[k].2 = p;
            }
            stamp_path(&mut laid, &mut zone, &lanes[k].2, inp, true);
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

    /// Protects Ruling GU's floor: half the median land distance of the lane's own
    /// course (median 20 here, floor 10; a coast-hugging lane's floor is 0). Mutating
    /// `REUSE_OFFSHORE_SHARE`, or taking the minimum instead of the median, turns it red.
    #[test]
    fn the_trunk_discount_floor_is_half_the_lanes_median_land_distance() {
        let land: Vec<u16> = vec![1, 10, 20, 30, 40];
        assert_eq!(reuse_floor(&[0, 1, 2, 3, 4], &land), 10);
        assert_eq!(reuse_floor(&[0, 0, 1], &land), 0, "a coast-hugging lane keeps every trunk");
    }

    /// Protects the floor's use in the search: a laid trunk along the coast (row 3, land
    /// rows 0-2) pulls a lane from (2,20) to (87,20) onto it with floor 0, and does not
    /// with floor 8 (the trunk's cells are 1 from land). Dropping the floor test in the
    /// search turns the second half red.
    #[test]
    fn a_lane_far_offshore_is_not_pulled_onto_a_coastal_trunk() {
        let (w, h) = (90usize, 40usize);
        let cost: Vec<f32> = (0..w * h).map(|i| if i / w < 3 { f32::INFINITY } else { 1.0 }).collect();
        let rp = [20 * w + 2, 20 * w + 87];
        let prevs: Vec<Vec<i32>> = rp.iter().map(|&r| crate::road_dijkstra(&cost, w, h, r % w, r / w, false, None, true).1).collect();
        let prev: Vec<&[i32]> = prevs.iter().map(|v| v.as_slice()).collect();
        let c = cost.clone();
        let base = move |i: usize, j: usize, dx: isize, dy: isize| -> f64 {
            if !c[i].is_finite() || !c[j].is_finite() { f64::INFINITY } else if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 }
        };
        let inp = LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &rp, prev: &prev, weight: &[1.0, 1.0], base: &base };
        let land = land_distance(&inp);
        let laid: Vec<u16> = (0..w * h).map(|c| (c / w == 3) as u16).collect();
        let zone = vec![0u16; w * h];
        let free = vec![false; w * h];
        let win = window_for(&[20 * w + 2, 20 * w + 87], w, h, false);
        let mid = |floor: u16| search(&inp, &laid, &zone, &free, &land, floor, win, rp[0], rp[1]).unwrap().into_iter().find(|c| c % w == 45).unwrap() / w;
        assert_eq!(mid(0), 3, "with no floor the lane joins the coastal trunk");
        assert_eq!(mid(8), 20, "with the floor it keeps its open-sea course");
    }

    /// Protects the land distance: the steps to the nearest land cell, 0 on land.
    #[test]
    fn land_distance_counts_steps_to_the_nearest_land() {
        let (w, h) = (10usize, 6usize);
        let cost: Vec<f32> = (0..w * h).map(|i| if i / w == 0 { f32::INFINITY } else { 1.0 }).collect();
        let prev: Vec<&[i32]> = vec![];
        let base = |_: usize, _: usize, _: isize, _: isize| 1.0;
        let inp = LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &[], prev: &prev, weight: &[], base: &base };
        let d = land_distance(&inp);
        assert_eq!((d[3], d[w + 3], d[3 * w + 3], d[5 * w + 3]), (0, 1, 3, 5));
    }

    /// A `w` x `h` all-sea grid with the given ports, for the thread-zone tests.
    fn open_sea(w: usize, h: usize, rp: &[usize]) -> (Vec<f32>, Vec<usize>) {
        (vec![1.0; w * h], rp.to_vec())
    }

    /// Protects the stamp: a lane path marks its own cells and every cell within
    /// `THREAD_RADIUS_CELLS` (4) of them, and removing it clears both. Mutating
    /// `THREAD_RADIUS_CELLS` to 0 or 5 turns it red.
    #[test]
    fn a_lane_stamps_a_four_cell_thread_zone_and_removes_it() {
        let (w, h) = (30usize, 30usize);
        let (cost, rp) = open_sea(w, h, &[15 * w + 2]);
        let prev: Vec<&[i32]> = vec![&[]];
        let base = |_: usize, _: usize, _: isize, _: isize| 1.0;
        let inp = LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &rp, prev: &prev, weight: &[1.0], base: &base };
        let (mut laid, mut zone) = (vec![0u16; w * h], vec![0u16; w * h]);
        stamp_path(&mut laid, &mut zone, &[15 * w + 15], &inp, true);
        assert_eq!(laid[15 * w + 15], 1);
        assert!(zone[15 * w + 19] > 0 && zone[19 * w + 15] > 0, "4 cells away is in the zone");
        assert_eq!(zone[15 * w + 20], 0, "5 cells away is not");
        stamp_path(&mut laid, &mut zone, &[15 * w + 15], &inp, false);
        assert!(laid.iter().chain(zone.iter()).all(|&v| v == 0));
    }

    /// Protects the port exemption's reach: cells within 8 of a port are free, 9 not.
    /// Mutating `THREAD_JUNCTION_CELLS` to 6 or 10 turns it red.
    #[test]
    fn the_port_exemption_reaches_eight_cells() {
        let (w, h) = (40usize, 40usize);
        let (cost, rp) = open_sea(w, h, &[20 * w + 10]);
        let prev: Vec<&[i32]> = vec![&[]];
        let base = |_: usize, _: usize, _: isize, _: isize| 1.0;
        let inp = LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &rp, prev: &prev, weight: &[1.0], base: &base };
        let f = junction_cells(&inp);
        assert!(f[20 * w + 18] && !f[20 * w + 19]);
    }

    /// Protects the thread penalty: a lane from (2,22) to (87,22) with a thread zone
    /// over rows 18-24 (another lane's) leaves the zone to keep clear of it, where
    /// ports are far away; with the whole map marked free (the port exemption) it
    /// stays on row 22. Mutating `THREAD_FACTOR` to 1.0 or dropping the `free`
    /// test turns it red.
    #[test]
    fn a_lane_keeps_clear_of_a_thread_zone_except_around_ports() {
        let (w, h) = (90usize, 50usize);
        let (cost, rp) = open_sea(w, h, &[22 * w + 2, 22 * w + 87]);
        let prevs: Vec<Vec<i32>> = rp.iter().map(|&r| crate::road_dijkstra(&cost, w, h, r % w, r / w, false, None, true).1).collect();
        let prev: Vec<&[i32]> = prevs.iter().map(|v| v.as_slice()).collect();
        let base = |_: usize, _: usize, dx: isize, dy: isize| if dx != 0 && dy != 0 { std::f64::consts::SQRT_2 } else { 1.0 };
        let inp = LaneIn { cost: &cost, rw: w, rh: h, world: false, rp: &rp, prev: &prev, weight: &[1.0, 1.0], base: &base };
        let laid = vec![0u16; w * h];
        let zone: Vec<u16> = (0..w * h).map(|c| ((18..=24).contains(&(c / w))) as u16).collect();
        let win = window_for(&[22 * w + 2, 22 * w + 87], w, h, false);
        let rows = |free: bool| -> Vec<usize> {
            let f = vec![free; w * h];
            search(&inp, &laid, &zone, &f, &vec![4095; w * h], 0, win, rp[0], rp[1]).unwrap().iter().filter(|&&c| (30..60).contains(&(c % w))).map(|c| c / w).collect()
        };
        assert!(rows(false).iter().all(|&r| !(18..=24).contains(&r)), "{:?}", rows(false));
        assert!(rows(true).iter().all(|&r| r == 22), "free cells are not penalised");
    }

    /// Never crosses land: with a land wall between, every lane cell is sea.
    #[test]
    fn a_lane_never_crosses_land() {
        let l = lanes(40, 12, 2, &[(2, 5), (37, 5)], &[(0, 1)]);
        assert!(l[0].2.iter().all(|&c| c / 40 >= 2));
    }

    /// Protects the merge: lane B runs 1.5 to 2 cells off lane A for a stretch; its
    /// interior vertices there are replaced by A's points, so B's middle lies exactly
    /// on A (y == 0) and B's ends (ports) do not move. Mutating `snap` below the gap
    /// (as `SNAP_CELLS` 3 to 1) leaves B off A and turns it red.
    #[test]
    fn near_parallel_lanes_are_merged_onto_one_trunk() {
        let a = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
        let b = vec![(0.0, 2.5), (20.0, 2.0), (40.0, 1.5), (60.0, 2.0), (80.0, 1.5), (100.0, 2.5)];
        let mut lanes = vec![a, b.clone()];
        let n = snap_parallel(&mut lanes, 3.0, &|_, _| true);
        assert_eq!(n, 1);
        assert_eq!((lanes[1][0], *lanes[1].last().unwrap()), (b[0], *b.last().unwrap()), "ports stay");
        assert!(lanes[1][1..lanes[1].len() - 1].iter().all(|p| p.1 == 0.0), "{:?}", lanes[1]);
        let mut far = vec![vec![(0.0, 0.0), (100.0, 0.0)], b.clone()];
        assert_eq!(snap_parallel(&mut far, 1.0, &|_, _| true), 0, "a gap wider than `snap` is left");
        assert_eq!(far[1], b);
    }

    /// Protects the land guard: the same lanes with land between them (nothing is
    /// ocean) are left as they were.
    #[test]
    fn a_merge_never_crosses_land() {
        let a = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
        let b = vec![(0.0, 2.5), (20.0, 2.0), (40.0, 1.5), (60.0, 2.0), (80.0, 1.5), (100.0, 2.5)];
        let mut lanes = vec![a, b.clone()];
        assert_eq!(snap_parallel(&mut lanes, 3.0, &|_, _| false), 0);
        assert_eq!(lanes[1], b);
    }

    /// Protects the connector test: the new segment from the lane's last vertex before
    /// the run to the trunk, and back from it, must be open water; here the midpoints are land, so nothing
    /// merges. Dropping the connector test merges it and turns this red.
    #[test]
    fn a_merge_whose_connector_crosses_land_is_refused() {
        let a = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
        let b = vec![(0.0, 2.5), (5.0, 10.0), (20.0, 2.0), (40.0, 1.5), (60.0, 2.0), (80.0, 1.5), (100.0, 2.5)];
        let wet = |_: f64, y: f64| y < 1.0 || y > 12.0;
        let mut lanes = vec![a.clone(), b.clone()];
        assert_eq!(snap_parallel(&mut lanes, 3.0, &wet), 0);
        assert_eq!(snap_parallel(&mut vec![a, b], 3.0, &|_, _| true), 1);
        // And the connector from the trunk back to the lane's next vertex.
        let c = vec![(0.0, 2.5), (20.0, 2.0), (40.0, 1.5), (60.0, 2.0), (80.0, 1.5), (95.0, 10.0), (100.0, 2.5)];
        let a = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
        assert_eq!(snap_parallel(&mut vec![a.clone(), c.clone()], 3.0, &wet), 0);
        assert_eq!(snap_parallel(&mut vec![a, c], 3.0, &|_, _| true), 1);
    }

    /// Protects the direction rule: a lane running along another and then doubling back
    /// is merged only for the forward run; the vertex where it turns back is left as
    /// it was. Dropping the direction break replaces the turn-back too and turns it red.
    #[test]
    fn a_run_ends_where_the_lane_doubles_back_along_another() {
        let a = vec![(0.0, 0.0), (100.0, 0.0)];
        let b = vec![(0.0, 8.0), (30.0, 2.0), (70.0, 2.0), (50.0, 1.0), (40.0, 8.0)];
        let mut lanes = vec![a, b.clone()];
        assert_eq!(snap_parallel(&mut lanes, 3.0, &|_, _| true), 1);
        assert_eq!(lanes[1][1].1, 0.0, "the forward run is on the trunk");
        assert!(lanes[1].contains(&(50.0, 1.0)), "the turn-back vertex stays: {:?}", lanes[1]);
    }
}

/// The nearest point of polyline `line` to `p`: its distance, the segment index and the
/// parameter along that segment (0..=1).
fn nearest_on(line: &[(f64, f64)], p: (f64, f64)) -> (f64, usize, f64) {
    let mut best = (f64::INFINITY, 0usize, 0.0);
    for (k, w) in line.windows(2).enumerate() {
        let (vx, vy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let l2 = vx * vx + vy * vy;
        let t = if l2 > 0.0 { (((p.0 - w[0].0) * vx + (p.1 - w[0].1) * vy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let d = (p.0 - w[0].0 - t * vx).hypot(p.1 - w[0].1 - t * vy);
        if d < best.0 {
            best = (d, k, t);
        }
    }
    best
}

/// The point `t` along segment `k` of `line`.
fn at_param(line: &[(f64, f64)], k: usize, t: f64) -> (f64, f64) {
    let (a, b) = (line[k], line[k + 1]);
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

/// Merges near-parallel lanes into one trunk **after** tracing and smoothing (owner,
/// Ruling GS addendum, 2026-11-22: *"make sure the threading doesn't happen anymore
/// for sea lanes"*). Each lane is smoothed alone, so lanes that shared their cells
/// still run 1 to 2 cells apart. For every lane after the first, every maximal run
/// of its interior vertices lying within `snap` map cells of an earlier lane's
/// polyline (not on it) whose nearest points advance in one direction along that
/// polyline is replaced by the earlier lane's own vertices between the run's two
/// projections, so the two lanes then share points exactly. A replacement is made
/// only when the two connecting segments' midpoints are open water (`is_ocean`), so a merge
/// never crosses land; endpoints (the ports) never move. Lanes with pen breaks
/// (a seam crossing) are left alone. Returns how many runs were merged.
///
/// `lanes` are polylines in map cells, in trunk order (the busiest first).
pub fn snap_parallel(lanes: &mut [Vec<(f64, f64)>], snap: f64, is_ocean: &dyn Fn(f64, f64) -> bool) -> usize {
    let mut merged = 0;
    for j in 1..lanes.len() {
        for i in 0..j {
            let (head, tail) = lanes.split_at_mut(j);
            let (other, mine) = (&head[i], &mut tail[0]);
            if other.len() < 2 || mine.len() < 3 {
                continue;
            }
            let near: Vec<Option<(usize, f64)>> = mine
                .iter()
                .enumerate()
                .map(|(v, &p)| {
                    let (d, k, t) = nearest_on(other, p);
                    (v > 0 && v + 1 < mine.len() && d <= snap).then_some((k, t))
                })
                .collect();
            let mut out: Vec<(f64, f64)> = Vec::with_capacity(mine.len());
            let mut v = 0;
            while v < mine.len() {
                if near[v].is_none() {
                    out.push(mine[v]);
                    v += 1;
                    continue;
                }
                // The run ends where the nearest points stop advancing one way along `other`
                // (a lane that doubles back or leaves is not running alongside any more).
                let key = |q: (usize, f64)| q.0 as f64 + q.1;
                let mut e = v;
                let mut dir = 0.0f64;
                while e + 1 < mine.len() && near[e + 1].is_some() {
                    let step = key(near[e + 1].unwrap()) - key(near[e].unwrap());
                    if step != 0.0 && dir != 0.0 && step.signum() != dir {
                        break;
                    }
                    if step != 0.0 {
                        dir = step.signum();
                    }
                    e += 1;
                }
                let (a, b) = (near[v].unwrap(), near[e].unwrap());
                let mut piece = vec![at_param(other, a.0, a.1)];
                if key(b) >= key(a) {
                    piece.extend((a.0 + 1..=b.0).map(|k| other[k]));
                } else {
                    piece.extend((b.0 + 1..=a.0).rev().map(|k| other[k]));
                }
                piece.push(at_param(other, b.0, b.1));
                // The piece is the earlier lane's own course, already water; only the two
                // connectors (from the lane to the piece and back) are new segments.
                let free = |p: (f64, f64), q: (f64, f64)| is_ocean((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
                // A lane's ends are its ports, which stand on land, so a connector to an end is
                // not tested.
                let wet = is_ocean(piece[0].0, piece[0].1)
                    && (v == 1 || out.last().is_none_or(|&l| free(l, piece[0])))
                    && (e + 2 == mine.len() || mine.get(e + 1).is_none_or(|&n| free(*piece.last().unwrap(), n)));
                if e > v && wet {
                    out.extend(piece);
                    merged += 1;
                } else {
                    out.extend_from_slice(&mine[v..=e]);
                }
                v = e + 1;
            }
            *mine = out;
        }
    }
    merged
}
