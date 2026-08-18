//! A\* over the site cost raster — reference lines 28514-28547, one function.
//! (The scope doc's plan said 28556; the last nine of those lines are
//! milestone 5's own site-model header comments.)
//!
//! `buildPrimaries` (milestone 6) rasterises the site into 8 m cells whose cost
//! is a Tobler-flavoured slope penalty plus water and bank terms, then runs this
//! search from each external route endpoint to the market anchor, multiplying
//! already-used cells by 0.45 so later routes braid onto earlier ones. The path
//! it returns is simplified, Chaikin-smoothed and simplified again before
//! becoming `'primary'` streets, so a single differently-chosen cell early in a
//! route survives as a differently-shaped arterial, and every block, parcel and
//! building that grows against that arterial moves with it.
//!
//! **The heap is ported literally rather than swapped for [`std::collections::
//! BinaryHeap`].** This is not stylistic. The reference's search is not an
//! optimal one — the heuristic is `0.9 ·` Euclidean *in cells* while a step's
//! cost is the trapezoidal average of two raster values that are metres-scaled
//! (`c * CS`, so on the order of 8-2000), which makes the heuristic wildly
//! *under*-weighted in the normal case and *over*-weighted wherever the raster
//! is cheap; there is no closed set and no stale-entry check, so cells are
//! re-expanded; and `if (i === gi) break` stops on the first *pop* of the goal,
//! which under an inadmissible heuristic is not necessarily its cheapest path.
//! What makes the result reproducible is therefore the exact order the open list
//! hands cells back, which is decided by this specific binary heap's tie-break:
//! sift-up stops on `<=` (so an equal-`f` newcomer stays *below* its parent) and
//! sift-down uses a strict `<` (so a tie prefers the left child, i.e. the
//! current node keeps its place). `BinaryHeap` has neither property, and on a
//! flat cost field — which is most of a real site away from the river — ties are
//! the common case, not the exotic one. Golden-tested; see `tests.rs`.
//!
//! Reachable from the reference's own `UME._test` export, so every value in
//! `tests/golden.rs` is the reference's, not this port's.

use crate::geom::js_hypot;
use std::f64::consts::SQRT_2;

#[cfg(test)]
mod tests;

/// The 8-connected neighbourhood, in the reference's order and with its step
/// lengths. **The order is load-bearing**: every one of these pushes at the same
/// `f` lands in the open list in this sequence, and the heap preserves that
/// sequence among equals.
const DIRS: [(isize, isize, f64); 8] = [
    (1, 0, 1.0),
    (-1, 0, 1.0),
    (0, 1, 1.0),
    (0, -1, 1.0),
    (1, 1, SQRT_2),
    (1, -1, SQRT_2),
    (-1, 1, SQRT_2),
    (-1, -1, SQRT_2),
];

/// `push(i, f)` — sift-up. Stops on `open[p].f <= open[c].f`, so an entry that
/// ties its parent stays where it is.
fn push(open: &mut Vec<(usize, f64)>, i: usize, f: f64) {
    open.push((i, f));
    let mut c = open.len() - 1;
    while c > 0 {
        let p = (c - 1) >> 1;
        if open[p].1 <= open[c].1 {
            break;
        }
        open.swap(p, c);
        c = p;
    }
}

/// `pop()` — returns the root and sifts the former last element down. Both child
/// comparisons are strict `<`, so a tie leaves `m == c` and the left child wins
/// over the right.
///
/// Note the reference's own shape here: it reads `open[0]` *before* popping the
/// tail, so a one-element heap returns that element and leaves an empty list
/// without ever entering the sift loop.
fn pop(open: &mut Vec<(usize, f64)>) -> (usize, f64) {
    let t = open[0];
    let l = open.pop().expect("pop is only called on a non-empty open list");
    if !open.is_empty() {
        open[0] = l;
        let mut c = 0usize;
        loop {
            let mut m = c;
            let (a, b) = (2 * c + 1, 2 * c + 2);
            if a < open.len() && open[a].1 < open[m].1 {
                m = a;
            }
            if b < open.len() && open[b].1 < open[m].1 {
                m = b;
            }
            if m == c {
                break;
            }
            open.swap(m, c);
            c = m;
        }
    }
    t
}

/// `astar(cost, W, H, start, goal)` — least-cost 8-connected path over a
/// row-major `w * h` cost raster, returning the cells from `start` to `goal`
/// inclusive, or `None` where the reference returns `null`.
///
/// A step from `i` to `ni` costs `dl * 0.5 * (cost[i] + cost[ni])` — the
/// trapezoidal average of the two cells, scaled by the step length (1 or √2).
/// The frontier is ordered by `g + 0.9 * hypot(dx, dy)`, with the distance
/// measured in **cells** and through [`js_hypot`], because every distance in
/// this engine goes through V8's `Math.hypot` and not Rust's.
///
/// # Non-finite cost is how the reference expresses "impassable"
///
/// There is no obstacle mask: an 8-connected full grid has no unreachable cell,
/// so `None` can only arise from arithmetic. Both routes to it are reproduced
/// exactly, and both are reachable from `buildPrimaries`' own raster:
///
/// - **`f64::INFINITY`** — the tentative cost is `INFINITY`, and
///   `INFINITY < INFINITY` is false, so the cell is never relaxed. (The
///   reference's sea cells cost `240 * CS`, not infinity, so sea is merely
///   ruinously expensive there; but the raster is host-supplied and a caller
///   can hand in infinities.)
/// - **`f64::NAN`** — every comparison against NaN is false in Rust exactly as
///   in JS, so a NaN cell is likewise never relaxed *and* never poisons `g0`.
///   This is one of the few places where JS and Rust NaN semantics agreeing is
///   load-bearing rather than incidental (`cartalith-rust-conventions`), so it
///   is pinned by the `nanBand` and `nanSeals` goldens rather than assumed.
///
/// # Panics
///
/// If `start` or `goal` lies outside `w * h`, or `cost` is shorter than
/// `w * h`. The reference silently produces garbage in that case (a typed-array
/// read out of range yields `undefined`, and `undefined === Infinity` is false,
/// so its guard does not fire); its only caller clamps to `[1, w-2] x [1, h-2]`
/// before calling, so the branch is unreachable in practice and a panic is the
/// honest port of "cannot happen".
pub fn astar(
    cost: &[f64],
    w: usize,
    h: usize,
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    assert!(cost.len() >= w * h, "cost raster is {} cells, need {}", cost.len(), w * h);
    assert!(start.0 < w && start.1 < h, "start {start:?} is outside the {w}x{h} raster");
    assert!(goal.0 < w && goal.1 < h, "goal {goal:?} is outside the {w}x{h} raster");

    let idx = |x: usize, y: usize| y * w + x;
    let mut open: Vec<(usize, f64)> = Vec::new();
    let mut g0 = vec![f64::INFINITY; w * h];
    let mut came = vec![-1i32; w * h];

    let (si, gi) = (idx(start.0, start.1), idx(goal.0, goal.1));
    g0[si] = 0.0;
    push(&mut open, si, 0.0);

    while !open.is_empty() {
        let (i, _) = pop(&mut open);
        if i == gi {
            break;
        }
        let (x, y) = (i % w, i / w);
        // Dead in the reference too — `g0[i]` is written before every `push`, so
        // a popped cell always has a finite `g`. Kept because it is written, and
        // because removing it would be a silent behavioural bet.
        if g0[i] == f64::INFINITY {
            continue;
        }
        for &(dx, dy, dl) in &DIRS {
            let (nx, ny) = (x as isize + dx, y as isize + dy);
            if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let ni = idx(nx as usize, ny as usize);
            let c = g0[i] + dl * 0.5 * (cost[i] + cost[ni]);
            if c < g0[ni] {
                g0[ni] = c;
                came[ni] = i as i32;
                let hh = js_hypot(nx as f64 - goal.0 as f64, ny as f64 - goal.1 as f64);
                push(&mut open, ni, c + hh * 0.9);
            }
        }
    }

    if came[gi] < 0 && gi != si {
        return None;
    }
    let mut path = Vec::new();
    let mut i = gi as i32;
    // The reference's own loop condition. If the predecessor chain ever ended
    // before reaching the start it would return a partial path rather than
    // `None`; it cannot, since `came` is only written when a cell is relaxed
    // from an already-relaxed one, but the shape is reproduced as written.
    while i >= 0 {
        let iu = i as usize;
        path.push((iu % w, iu / w));
        if iu == si {
            break;
        }
        i = came[iu];
    }
    path.reverse();
    Some(path)
}
