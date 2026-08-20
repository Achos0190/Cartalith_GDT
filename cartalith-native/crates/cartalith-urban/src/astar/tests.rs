//! Milestone 3's tests. All but two are **golden**: `astar` is one of the
//! fifteen internals the reference hands out through `UME._test`, so every
//! expected path in `tests/golden.rs` came out of the reference engine itself,
//! sliced contiguously from the frozen HTML and run under a bare Node
//! `vm.runInContext` with no DOM.
//!
//! **The cost rasters are rebuilt here, not captured.** A raster is `w * h`
//! doubles and storing seventeen of them would bury the goldens in noise, so
//! each recipe below is deliberately trivial — a constant, an integer formula,
//! or one `stream().range()` draw per cell in row-major order — and is
//! reproduced verbatim on both sides. The RNG-driven ones are additionally a
//! golden over [`crate::rng`], since the reference's own exported `stream`
//! filled them.
//!
//! **The capture asserts its own output is not empty.** Three subsystems in this
//! project have now shipped a harness that produced silently empty results and
//! passed every structural check, so the capture script refuses to write a file
//! unless every path is non-empty, actually begins at its start cell, actually
//! ends at its goal cell, and the two deliberately-unreachable scenarios really
//! did return `null`.
//!
//! # Mutation testing, and the coverage hole it found
//!
//! The first seventeen scenarios here were hand-picked to look thorough:
//! degenerate strips, both rectangle orientations, barriers, non-finite cost,
//! a start-equals-goal case, two RNG-driven rasters, and a 30-goal sweep. They
//! reproduced the reference exactly. Then fifteen mutations of the ported
//! algorithm were run against them and **nine survived** — the heuristic
//! weight, the trapezoid factor, the `DIRS` order, all three heap-comparator
//! tie-breaks, `js_hypot` vs `f64::hypot`, the `i == gi` early break, and the
//! dead `INFINITY` guard. The goldens were, for most of what makes this
//! function's output reproducible, vacuous.
//!
//! The reason is one fact worth carrying into every later milestone:
//! **a continuously-valued cost raster never produces two frontier entries with
//! exactly equal `f`, so it cannot observe a tie-break at all.** An exhaustive
//! search over ~800,000 (raster family, size, endpoint) combinations found a
//! discriminator for every one of those mutations, and every tie-break
//! discriminator came from a **quantised** raster — costs drawn from
//! `{0.5, 1}`, `{1, 2}` or `{1, 2, 3, 4}` — which is also what a real 8 m site
//! raster looks like away from the river, where `1 + (slope * 3.2)^2` is flat.
//! The eight scenarios named `ties*`, `nearAdmissible`, `trapezoidal` and
//! `greedyTrap` are those discriminators, added for exactly that reason.
//!
//! With them, **fourteen of the fifteen mutations are killed**. The survivor is
//! the `g0[i] == INFINITY` early `continue`, which is provably unreachable in
//! the reference too; see [`the_dead_infinity_guard_is_dead_in_the_reference_too`].

use super::*;
use crate::rng::stream;

mod golden;

/// `fill(W, H, f)` from the capture script.
fn fill(w: usize, h: usize, f: impl Fn(usize, usize) -> f64) -> Vec<f64> {
    let mut c = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            c[y * w + x] = f(x, y);
        }
    }
    c
}

/// `rngFill(W, H, seed, label, a, b)` from the capture script — row-major draw
/// order, which is load-bearing.
fn rng_fill(w: usize, h: usize, seed: u32, label: &str, a: f64, b: f64) -> Vec<f64> {
    let mut r = stream(seed, label);
    let mut c = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            c[y * w + x] = r.range(a, b);
        }
    }
    c
}

/// `quantFill(...)` from the capture script — a raster drawn from a small set of
/// **repeating** values, which is the only kind that makes two frontier entries
/// tie on `f` exactly. See the module docs: no continuous raster does.
fn quant_fill(w: usize, h: usize, seed: u32, label: &str, lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let mut r = stream(seed, label);
    let mut c = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            c[y * w + x] = lo + step * ((hi - lo) / step * r.u()).floor();
        }
    }
    c
}

/// One scenario's raster and endpoints.
struct Case {
    cost: Vec<f64>,
    w: usize,
    h: usize,
    start: (usize, usize),
    goal: (usize, usize),
}

fn mk(cost: Vec<f64>, w: usize, h: usize, start: (usize, usize), goal: (usize, usize)) -> Case {
    Case { cost, w, h, start, goal }
}

/// Mirrors `SCEN` in the capture script one-for-one; the names are the goldens'
/// keys.
fn scenario(name: &str) -> Case {
    let cheb = |x: usize, y: usize| (x as isize - 6).abs().max((y as isize - 6).abs());
    match name {
        "strip" => mk(fill(9, 1, |_, _| 1.0), 9, 1, (0, 0), (8, 0)),
        "uniform" => mk(fill(11, 9, |_, _| 1.0), 11, 9, (0, 0), (10, 8)),
        "startIsGoal" => mk(fill(5, 5, |_, _| 1.0), 5, 5, (2, 2), (2, 2)),
        "adjacent" => mk(fill(5, 5, |_, _| 1.0), 5, 5, (1, 1), (2, 1)),
        "diagAdjacent" => mk(fill(5, 5, |_, _| 1.0), 5, 5, (1, 1), (2, 2)),
        "backwards" => mk(fill(11, 9, |_, _| 1.0), 11, 9, (10, 8), (0, 0)),
        "ramp" => mk(fill(12, 7, |x, _| 1.0 + x as f64), 12, 7, (0, 3), (11, 3)),
        "wall" => {
            let c = fill(15, 11, |x, y| if x == 7 && y != 8 { 500.0 } else { 1.0 });
            mk(c, 15, 11, (1, 5), (13, 5))
        }
        "moat" => {
            let c = fill(9, 9, |x, y| if cheb(x, y) == 1 { f64::INFINITY } else { 1.0 });
            mk(c, 9, 9, (1, 1), (6, 6))
        }
        "nanBand" => {
            let c = fill(13, 9, |x, y| if x == 6 && y != 2 { f64::NAN } else { 1.0 });
            mk(c, 13, 9, (0, 4), (12, 4))
        }
        "nanSeals" => {
            let c = fill(9, 9, |x, y| if cheb(x, y) == 1 { f64::NAN } else { 1.0 });
            mk(c, 9, 9, (1, 1), (6, 6))
        }
        "cheap" => mk(fill(13, 9, |_, _| 0.1), 13, 9, (0, 4), (12, 4)),
        "zero" => mk(fill(9, 7, |_, _| 0.0), 9, 7, (0, 0), (8, 6)),
        "nonSquare" => {
            mk(fill(17, 3, |x, y| 1.0 + ((x * 3 + y * 7) % 5) as f64), 17, 3, (0, 1), (16, 1))
        }
        "tall" => {
            mk(fill(3, 17, |x, y| 1.0 + ((x * 3 + y * 7) % 5) as f64), 3, 17, (1, 0), (1, 16))
        }
        "rngWide" => mk(rng_fill(24, 18, 7, "m3/astar", 1.0, 9.0), 24, 18, (1, 1), (22, 16)),
        "rngFlat" => {
            mk(rng_fill(31, 11, 20260818, "m3/astar2", 0.2, 4.0), 31, 11, (1, 5), (29, 5))
        }
        // The eight below were found by search, not by hand — see the module
        // docs. Each one kills a mutation that all seventeen hand-picked rasters
        // above survive.
        "tiesHalf" => {
            mk(quant_fill(24, 18, 1, "m3/search1", 0.5, 1.5, 0.5), 24, 18, (0, 9), (23, 9))
        }
        "tiesLeft" => {
            mk(quant_fill(31, 11, 1, "m3/search1", 0.5, 1.5, 0.5), 31, 11, (0, 5), (30, 5))
        }
        "tiesRight" => {
            mk(quant_fill(33, 25, 1, "m3/search1", 1.0, 3.0, 1.0), 33, 25, (0, 12), (32, 12))
        }
        "tiesWide" => {
            mk(quant_fill(64, 48, 23, "m3/search23", 1.0, 3.0, 1.0), 64, 48, (0, 0), (63, 47))
        }
        "tiesDiag" => {
            mk(quant_fill(33, 25, 38, "m3/search38", 1.0, 5.0, 1.0), 33, 25, (1, 23), (31, 1))
        }
        "nearAdmissible" => {
            mk(rng_fill(24, 18, 1, "m3/search1", 0.6, 1.2), 24, 18, (0, 9), (23, 9))
        }
        "trapezoidal" => mk(rng_fill(24, 18, 1, "m3/search1", 0.6, 1.2), 24, 18, (0, 0), (23, 17)),
        "greedyTrap" => mk(rng_fill(24, 18, 1, "m3/search1", 0.2, 4.0), 24, 18, (12, 9), (23, 0)),
        other => panic!("no raster defined for scenario {other}"),
    }
}

/// The sweep raster: one 6 x 5 field, every cell taken as the goal in turn.
fn sweep_raster() -> (Vec<f64>, usize, usize) {
    (rng_fill(6, 5, 3, "m3/sweep", 1.0, 5.0), 6, 5)
}

fn run(name: &str) -> Option<Vec<(usize, usize)>> {
    let c = scenario(name);
    astar(&c.cost, c.w, c.h, c.start, c.goal)
}

/// The distinct cost values in a scenario's raster, ascending.
fn distinct(name: &str) -> Vec<f64> {
    let mut v = scenario(name).cost;
    v.sort_by(|a, b| a.partial_cmp(b).expect("this raster has no NaN"));
    v.dedup();
    v
}

#[test]
fn golden_every_scenario_reproduces_the_reference_path_exactly() {
    for sc in golden::GOLDEN {
        let got = run(sc.name);
        let want = sc.path.map(<[(usize, usize)]>::to_vec);
        assert_eq!(got, want, "{}: path", sc.name);
    }
}

#[test]
fn golden_sweep_reproduces_every_goal_in_the_raster() {
    // A single hand-picked goal can hide an x/y transposition (it is symmetric
    // on a square raster reached along the diagonal) and can hide a backtrack
    // that drops or duplicates one end. Taking every cell in a small raster as
    // the goal in turn does not.
    let (cost, w, h) = sweep_raster();
    assert_eq!(golden::SWEEP.len(), w * h, "the sweep golden lost entries");
    for y in 0..h {
        for x in 0..w {
            let got = astar(&cost, w, h, (0, 0), (x, y));
            let want = golden::SWEEP[y * w + x].map(<[(usize, usize)]>::to_vec);
            assert_eq!(got, want, "sweep goal ({x},{y})");
        }
    }
}

#[test]
fn golden_scenarios_cover_every_branch_this_milestone_claims() {
    // The guard against goldens quietly becoming vacuous, in the shape milestone
    // 2 established: assert the properties each scenario exists to pin, so that
    // dropping or weakening one fails loudly here rather than silently there.
    let by = |n: &str| {
        golden::GOLDEN.iter().find(|s| s.name == n).unwrap_or_else(|| panic!("scenario {n} is gone"))
    };
    let p = |n: &str| by(n).path.expect("expected a path");

    // Endpoints are inclusive, and a zero-length search returns one cell, not
    // an empty vec — `buildPrimaries` maps the path straight into street points.
    assert_eq!(p("startIsGoal"), [(2, 2)]);
    assert_eq!(p("adjacent").len(), 2);
    assert_eq!(p("diagAdjacent"), [(1, 1), (2, 2)]);

    // Non-finite cost is the only route to `None`, and both kinds reach it.
    assert!(by("moat").path.is_none(), "an infinite ring must seal the goal off");
    assert!(by("nanSeals").path.is_none(), "a NaN ring must seal the goal off too");
    // ...but a NaN band with a gap is routed *around*, not through.
    let nb = p("nanBand");
    assert!(nb.iter().all(|&(x, y)| x != 6 || y == 2), "the path crossed a NaN cell");
    assert_eq!(nb.first(), Some(&(0, 4)));
    assert_eq!(nb.last(), Some(&(12, 4)));

    // The wall scenario really detours through its one gap at y == 8.
    let wl = p("wall");
    assert!(wl.contains(&(7, 8)), "the wall was crossed somewhere other than the gap");
    assert!(wl.iter().all(|&(x, y)| x != 7 || y == 8));

    // Rectangular rasters in both orientations, so an x/y swap cannot pass.
    assert_eq!(p("nonSquare").len(), 17);
    assert_eq!(p("tall").len(), 17);
    assert_ne!(p("nonSquare"), p("tall"), "the two orientations must not agree");

    // The RNG-driven cases are genuinely non-trivial: a straight run would be
    // 22 and 29 cells; both wander.
    assert!(p("rngWide").len() >= 26, "rngWide degenerated to a straight line");
    assert!(p("rngFlat").iter().any(|&(_, y)| y != 5), "rngFlat never left its row");

    // The eight search-found scenarios are the ones that make the tie-breaks and
    // the two weights observable at all (see the module docs). If any of them is
    // dropped, nine of fifteen mutations start surviving again — so their
    // presence is asserted, not assumed.
    for n in ["tiesHalf", "tiesLeft", "tiesRight", "tiesWide", "tiesDiag", "nearAdmissible", "trapezoidal", "greedyTrap"] {
        assert!(p(n).len() > 8, "{n}: the discriminating scenario went missing or degenerate");
    }
    // The quantised rasters really are quantised — that is the whole mechanism.
    assert_eq!(distinct("tiesHalf"), [0.5, 1.0], "tiesHalf must draw two repeating values");
    assert_eq!(distinct("tiesRight"), [1.0, 2.0]);
    assert_eq!(distinct("tiesDiag"), [1.0, 2.0, 3.0, 4.0]);
    // ...and the continuous ones really are not, which is why they observe the
    // two weights but no tie-break at all.
    assert_eq!(
        distinct("nearAdmissible").len(),
        24 * 18,
        "nearAdmissible must have no repeated cost value"
    );

    // Every path is a legal 8-connected walk with no repeats.
    for sc in golden::GOLDEN {
        let Some(path) = sc.path else { continue };
        for pair in path.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let (dx, dy) = ((a.0 as isize - b.0 as isize).abs(), (a.1 as isize - b.1 as isize).abs());
            assert!(dx <= 1 && dy <= 1 && dx + dy > 0, "{}: {a:?} -> {b:?} is not one step", sc.name);
        }
        let mut seen = std::collections::HashSet::new();
        assert!(path.iter().all(|c| seen.insert(*c)), "{}: path revisits a cell", sc.name);
    }
}

#[test]
fn the_search_is_reproducible_but_not_optimal_and_that_is_the_point() {
    // Recorded as a finding, not a defect. The heuristic is `0.9 x` the
    // Euclidean distance **in cells** while a step costs the trapezoidal mean of
    // two raster values, so on a cheap raster the heuristic dominates and the
    // search behaves like greedy best-first: it takes the straight row rather
    // than any cheaper wander, and `if (i == gi) break` accepts the first pop of
    // the goal. `cheap` is that case, at a uniform 0.1 per cell.
    let straight: Vec<(usize, usize)> = (0..13).map(|x| (x, 4)).collect();
    assert_eq!(run("cheap").unwrap(), straight);

    // The same raster shape at cost 1.0 per cell also runs straight — the point
    // is not that the greedy answer is wrong here, but that nothing in this
    // implementation *guarantees* optimality, so the golden path is the
    // specification and an "improved" search would break parity.
    let at_one = astar(&fill(13, 9, |_, _| 1.0), 13, 9, (0, 4), (12, 4)).unwrap();
    assert_eq!(at_one, straight);

    // And on a zero-cost raster every `g` is 0, so `f` is purely the heuristic
    // and the heap's tie-break alone decides the route.
    assert_eq!(run("zero").unwrap().len(), 9);
}

#[test]
fn the_dead_infinity_guard_is_dead_in_the_reference_too() {
    // The one mutation of fifteen that survives every golden: deleting
    // `if g0[i] == INFINITY { continue; }` from the expansion loop changes
    // nothing. That is not a coverage hole to be papered over with another
    // scenario — no scenario can reach it. `g0[ni]` is written on the line
    // before every `push`, and `g0[si]` is written before the start's own push,
    // so every index that can be popped already holds a finite `g`.
    //
    // The line is kept because it is what the reference writes, and removing it
    // would be a silent bet that this argument stays true as later milestones
    // hand in stranger rasters. Asserted here as the invariant it depends on:
    // even with infinities and NaNs in the raster, no relaxation ever leaves a
    // pushed cell at `INFINITY`.
    for name in ["moat", "nanBand", "nanSeals", "rngWide"] {
        let Case { cost, w, h, start, goal } = scenario(name);
        // Re-run the relaxation rule directly: any cell that would be pushed has
        // just had a finite `c` written into `g0`.
        let mut g0 = vec![f64::INFINITY; w * h];
        g0[start.1 * w + start.0] = 0.0;
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..w * h {
                if !g0[i].is_finite() {
                    continue;
                }
                let (x, y) = (i % w, i / w);
                for (dx, dy, dl) in DIRS {
                    let (nx, ny) = (x as isize + dx, y as isize + dy);
                    if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    let c = g0[i] + dl * 0.5 * (cost[i] + cost[ni]);
                    if c < g0[ni] {
                        assert!(c.is_finite(), "{name}: a relaxation wrote a non-finite g");
                        g0[ni] = c;
                        changed = true;
                    }
                }
            }
        }
        // ...and the two sealed scenarios really do leave their goal at INFINITY,
        // which is the only way `None` is ever returned.
        let sealed = matches!(name, "moat" | "nanSeals");
        assert_eq!(g0[goal.1 * w + goal.0].is_infinite(), sealed, "{name}: reachability");
    }
}

#[test]
fn js_hypot_is_used_for_the_heuristic_not_f64_hypot() {
    // Milestone 1 found the discrepancy and milestone 2 proved it structural
    // (V8 returns exactly 11 where `f64::hypot` returns 10.999999999999998, and
    // 11 is a node-snap threshold). Milestone 3's contribution is that it is not
    // exotic at all: over the 4,096 integer offsets a 64 x 64 raster produces,
    // the two disagree on **1,398** — better than a third. This is the direct
    // statement of the requirement; `tiesWide` is the golden that enforces it,
    // and it took a 64 x 48 quantised raster to find one, because the disagreement
    // is one ulp and only bites when it breaks or makes an exact `f` tie.
    let disagreements = (0..64)
        .flat_map(|dx| (0..64).map(move |dy| (dx as f64, dy as f64)))
        .filter(|&(dx, dy)| js_hypot(dx, dy) != dx.hypot(dy))
        .count();
    assert_eq!(disagreements, 1398, "the js_hypot / f64::hypot divergence changed");
    // The structural half, from milestone 2's captured pair:
    let d = 7.778174593052022_f64;
    assert_eq!(js_hypot(d, d), 11.0);
    assert!(d.hypot(d) < 11.0);
}

#[test]
#[should_panic(expected = "outside the")]
fn an_out_of_range_goal_panics_rather_than_reading_garbage() {
    // The reference reads past the end of its typed arrays and gets `undefined`,
    // whose comparisons are all false, so it silently produces nonsense. Its one
    // caller clamps to `[1, w-2] x [1, h-2]` first, so this is unreachable in
    // the engine; the port makes it loud instead of silent, and says so.
    let cost = fill(5, 5, |_, _| 1.0);
    let _ = astar(&cost, 5, 5, (0, 0), (5, 0));
}
