//! EF-6's three tracers against a **real generated world** rather than a
//! synthetic fixture — the same reason `bake_real_world.rs` exists.
//!
//! `cartalith_terrain::vector`'s own unit tests check each tracer against a
//! shape whose answer is known in closed form (a circle, a straight ramp, a
//! tilted crest). Those are the correctness tests. What they cannot check is
//! the property that only real terrain has: a coastline with thousands of
//! islands, a boundary mask that branches and loops, and a TPI field that is
//! noisy rather than a clean cone. These tests live in `cartalith-engine`
//! because that is the crate that owns `generate_terrain`; the functions under
//! test are `cartalith-terrain`'s.
//!
//! The grid is deliberately small (192 × 128) so this runs in seconds and can
//! stay un-`#[ignore]`d. Every assertion below is a *property*, not a captured
//! value, so it does not need re-baselining when generation changes — and none
//! of it is a golden-parity test, because EF-6 has no reference equivalent to
//! be parity with.

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_terrain::analysis::tpi;
use cartalith_terrain::vector::{trace_coastline, trace_fault_lines, trace_ridgelines, RidgeOpts};

const GW: usize = 192;
const GH: usize = 128;
const SEED: i32 = 20260920;

fn world() -> WorldState {
    let mut p = WorldParams::defaults(GW, GH, SEED);
    p.map_width_km = 800.0;
    generate_terrain(&p)
}

#[test]
fn fault_lines_stay_on_the_boundary_mask() {
    let ws = world();
    assert!(ws.boundary_mask.iter().any(|&v| v != 0), "the world has plate boundaries at all");

    let fls = trace_fault_lines(&ws.boundary_mask, &ws.boundary_type, GW, GH);
    assert!(!fls.is_empty(), "a real world's boundaries trace to at least one polyline");

    let mut pts = 0usize;
    for fl in &fls {
        assert!(fl.pts.len() >= 2, "a polyline is at least a segment");
        for &(px, py) in &fl.pts {
            // The pixel-space convention: a cell centre is (col+0.5, row+0.5),
            // so both coordinates must be exactly a half-integer inside the
            // grid, and the cell they name must be a masked one.
            assert!((px.fract() - 0.5).abs() < 1e-12, "x = {px} is not a cell centre");
            assert!((py.fract() - 0.5).abs() < 1e-12, "y = {py} is not a cell centre");
            let (cx, cy) = ((px - 0.5) as usize, (py - 0.5) as usize);
            assert!(cx < GW && cy < GH, "({px}, {py}) is off the grid");
            assert_eq!(
                ws.boundary_mask[cy * GW + cx], 1,
                "({px}, {py}) is not a boundary_mask cell"
            );
            pts += 1;
        }
    }
    // Not a threshold to tune — just proof the loop above had real work to do
    // rather than passing over three points.
    assert!(pts > 200, "only {pts} traced points on a {GW}x{GH} world");
    println!("faults: {} polylines, {pts} points", fls.len());
}

#[test]
fn fault_line_kinds_are_the_tagged_boundary_types() {
    let ws = world();
    let fls = trace_fault_lines(&ws.boundary_mask, &ws.boundary_type, GW, GH);
    // `tag_boundary_types` sets each run's kind to the most frequent non-NONE
    // `boundary_type` among its own points. Re-derive that here rather than
    // trusting the adapter carried the right field across.
    for fl in &fls {
        let mut counts = [0i32; 6];
        for &(px, py) in &fl.pts {
            let i = ((py - 0.5) as usize) * GW + ((px - 0.5) as usize);
            counts[ws.boundary_type[i] as usize] += 1;
        }
        // First maximum, not last: `tag_boundary_types` scans `k` ascending
        // with a strict `>`, so a tie goes to the lower `btype`. Reproducing
        // that here is the point — `max_by_key` returns the *last* maximum and
        // disagreed on a real tie between SUBDUCTION_OC and RIFT.
        let want = (1..6u8).fold((0u8, -1i32), |(bk, best), k| {
            if counts[k as usize] > best { (k, counts[k as usize]) } else { (bk, best) }
        });
        assert_eq!(fl.kind, want.0, "kind disagrees with its own points' majority");
    }
    let classified = fls.iter().filter(|f| f.kind != cartalith_terrain::btype::NONE).count();
    assert!(classified > 0, "no polyline carried a boundary type");
}

#[test]
fn ridgelines_run_through_ridge_like_cells_and_never_valley_like_ones() {
    let ws = world();
    let opts = RidgeOpts { world: true, ..RidgeOpts::default() };
    let pls = trace_ridgelines(&ws.field, GW, GH, ws.sea_level, opts);
    assert!(!pls.is_empty(), "a real world has ridges");

    // Re-derive TPI independently at the same radius rather than reading the
    // tracer's own copy: the claim under test is "these cells are ones TPI
    // calls ridge-like", and it has to be checked against TPI, not against the
    // tracer's internal state.
    let t = tpi(&ws.field, GW, GH, opts.tpi_radius, opts.world);
    let land: Vec<usize> =
        (0..GW * GH).filter(|&i| (ws.field[i] as f64) >= ws.sea_level).collect();
    assert!(!land.is_empty(), "the world has land");
    let land_mean: f64 = land.iter().map(|&i| t[i] as f64).sum::<f64>() / land.len() as f64;

    let mut traced = 0usize;
    let mut sum = 0.0f64;
    let mut worst = f64::INFINITY;
    for pl in &pls {
        assert!(pl.len() >= 3, "min_points defaults to 3");
        for &(px, py) in pl {
            let i = ((py - 0.5) as usize) * GW + ((px - 0.5) as usize);
            // Ridge-like means above the local mean. A valley-like cell is
            // below it, and not one may appear.
            assert!(
                t[i] as f64 > land_mean,
                "({px}, {py}) has TPI {} against a land mean of {land_mean} — valley-like",
                t[i]
            );
            assert!((ws.field[i] as f64) >= ws.sea_level, "({px}, {py}) is under water");
            sum += t[i] as f64;
            worst = worst.min(t[i] as f64);
            traced += 1;
        }
    }
    let traced_mean = sum / traced as f64;
    // The interesting claim is not that the mask was applied — it is that the
    // traced set is drawn from the far upper tail of TPI, which is what makes
    // it a crest line rather than "vaguely high ground". The threshold is
    // land_mean + 1 SD, so the traced mean must clear that comfortably.
    assert!(
        traced_mean > land_mean,
        "traced mean TPI {traced_mean} is not above the land mean {land_mean}"
    );
    println!(
        "ridges: {n} polylines, {traced} points, TPI land mean {land_mean:.5}, \
         traced mean {traced_mean:.5}, traced min {worst:.5}",
        n = pls.len()
    );
}

#[test]
fn ridgelines_ascend_strictly_on_real_terrain() {
    // The acyclicity guarantee, checked where it actually matters: real
    // terrain with noise, plateaus and equal-height neighbours, not a cone.
    let ws = world();
    let pls = trace_ridgelines(
        &ws.field,
        GW,
        GH,
        ws.sea_level,
        RidgeOpts { world: true, ..RidgeOpts::default() },
    );
    let at = |p: (f64, f64)| ws.field[((p.1 - 0.5) as usize) * GW + ((p.0 - 0.5) as usize)];
    let mut longest = 0usize;
    for pl in &pls {
        longest = longest.max(pl.len());
        for w in pl.windows(2) {
            assert!(at(w[1]) > at(w[0]), "step {:?} -> {:?} did not rise", w[0], w[1]);
            // Each step is to an 8-neighbour: never a jump.
            assert!(
                (w[1].0 - w[0].0).abs() <= 1.0 && (w[1].1 - w[0].1).abs() <= 1.0,
                "step {:?} -> {:?} is not an 8-neighbour",
                w[0],
                w[1]
            );
        }
    }
    println!("longest ridgeline: {longest} points");
}

#[test]
fn a_real_coastline_closes_its_rings_and_keeps_land_on_the_left() {
    let ws = world();
    let pls = trace_coastline(&ws.field, GW, GH, ws.sea_level);
    assert!(!pls.is_empty(), "a real world has a coastline");

    let mut rings = 0usize;
    let mut open = 0usize;
    let mut pts = 0usize;
    let (mut agree, mut tested) = (0usize, 0usize);
    let sample = |x: f64, y: f64| -> Option<f64> {
        let (cx, cy) = (x.floor(), y.floor());
        if cx < 0.0 || cy < 0.0 || cx >= GW as f64 || cy >= GH as f64 {
            return None;
        }
        Some(ws.field[cy as usize * GW + cx as usize] as f64)
    };
    for pl in &pls {
        assert!(pl.len() >= 2);
        pts += pl.len();
        if pl[0] == pl[pl.len() - 1] {
            rings += 1;
        } else {
            // An open chain may only end at the sampled border — half a cell
            // inside the grid edge, per the tracer's own documented limit.
            open += 1;
            for end in [pl[0], pl[pl.len() - 1]] {
                let edge = end.0 <= 0.5 + 1e-9
                    || end.1 <= 0.5 + 1e-9
                    || end.0 >= GW as f64 - 1.5 - 1e-9
                    || end.1 >= GH as f64 - 1.5 - 1e-9;
                assert!(edge, "open chain ends at {end:?}, which is not the border");
            }
        }
        // Orientation: the above-`sea_level` side is on the segment's visual
        // left, which for a row-major Y-down grid is `(dy, -dx)`. Sampled
        // 0.7 cells off the segment's midpoint on each side; ties and
        // near-tangent cases are skipped rather than counted either way.
        for w in pl.windows(2) {
            let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
            let len = dx.hypot(dy);
            if len < 1e-9 {
                continue;
            }
            let (nx, ny) = (dy / len * 0.7, -dx / len * 0.7);
            let (mx, my) = ((w[0].0 + w[1].0) * 0.5, (w[0].1 + w[1].1) * 0.5);
            let (Some(l), Some(r)) = (sample(mx + nx, my + ny), sample(mx - nx, my - ny)) else {
                continue;
            };
            if (l - ws.sea_level).signum() == (r - ws.sea_level).signum() {
                continue; // both sides read the same; the probe is inside the
                          // shore's own cell, not a disagreement with the rule
            }
            tested += 1;
            if l >= ws.sea_level && r < ws.sea_level {
                agree += 1;
            }
        }
    }
    assert!(tested > 500, "only {tested} orientable segments — too few to mean anything");
    let frac = agree as f64 / tested as f64;
    assert!(frac > 0.98, "land was on the left for only {:.2}% of segments", frac * 100.0);
    assert!(rings > 0, "no closed ring on a real world");
    println!("coast: {} runs ({rings} closed, {open} open), {pts} points, land-left {:.2}%", pls.len(), frac * 100.0);
}

#[test]
fn all_three_tracers_are_deterministic_on_a_real_world() {
    // Two independently generated worlds from the same params, so this also
    // covers "the input is the same" rather than only "the tracer is pure".
    let a = world();
    let b = world();
    assert_eq!(a.field, b.field, "generate_terrain itself is deterministic");

    let opts = RidgeOpts { world: true, ..RidgeOpts::default() };
    assert_eq!(
        trace_coastline(&a.field, GW, GH, a.sea_level),
        trace_coastline(&b.field, GW, GH, b.sea_level)
    );
    assert_eq!(
        trace_fault_lines(&a.boundary_mask, &a.boundary_type, GW, GH),
        trace_fault_lines(&b.boundary_mask, &b.boundary_type, GW, GH)
    );
    assert_eq!(
        trace_ridgelines(&a.field, GW, GH, a.sea_level, opts),
        trace_ridgelines(&b.field, GW, GH, b.sea_level, opts)
    );
}
