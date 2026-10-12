//! The Route/Way router's returned points stay on the map (found-bug row of
//! 2026-10-11, `OUTSTANDING_WORK.md` §2.3: "The router's smoothed points can
//! leave the map").
//!
//! `civ_dijkstra_path` emits each routing cell at its centre, `(c + 0.5) / sc`,
//! then `civ_smooth_path` splines those points (`civ_catmull_rom_sample`) and
//! rounds every sample. Two things put a sample off the grid: the centripetal
//! Catmull-Rom curve overshoots its control points where the path turns (the
//! cause of the large excursions), and a sample at the last routing row's
//! centre, `gh - 0.5`, rounds up to `gh`. The reference `_civSmoothPath` (v2.11
//! line 22382) does not clamp, so this is an inherited defect; the port now
//! clamps every returned sampled point into the grid, `[0, gw - 1] x
//! [0, gh - 1]`, the same box `JpWorld::clamp_cell` and `TerrainValid::check`
//! already read a point through.
//!
//! Since Ruling CF item 4 (owner, 2026-10-12) each run's two restored endpoints
//! are clamped into the same box too (a run that ends at a world-wrap seam
//! ended on a raw router point in the last half cell, measured (1023.5, 0.5)
//! and (0.5, 23.5) on the 1024 x 24 world-mode map), and the km is measured
//! over the clamped samples; `route_km_on_map.rs` tests the km. So every
//! returned point, endpoint or not, is held to the strict box here.

use cartalith_civ::tools::{self, RouteContext, RouteMode};

/// A flat, all-land world: every cell the same height above sea, so the router's
/// path is decided by distance alone and the only way a point can leave the map
/// is the smoother.
struct Flat {
    field: Vec<f32>,
    wb: Vec<u8>,
    gw: usize,
    gh: usize,
}

fn flat(gw: usize, gh: usize) -> Flat {
    Flat { field: vec![0.6; gw * gh], wb: vec![0; gw * gh], gw, gh }
}

fn ctx(f: &Flat, world: bool) -> RouteContext<'_> {
    RouteContext {
        field: &f.field,
        water_bodies: &f.wb,
        biome: None,
        river_order: None,
        places: &[],
        ways: &[],
        gw: f.gw,
        gh: f.gh,
        sea: 0.42,
        world,
        map_width_km: f.gw as f64 * 12.0,
        corridors: None,
        flow: None,
        flow_thresh: 0.0,
    }
}

/// Routes every endpoint pair of a small grid of rows and columns, in every
/// mode and both wrap settings, and returns the extreme coordinates of the
/// returned sampled (non-endpoint) points (min x, max x, min y, max y), how
/// many routes ran, and a description of every point outside the grid box,
/// for the failure message.
fn sweep(gw: usize, gh: usize) -> (f64, f64, f64, f64, usize, Vec<String>) {
    let f = flat(gw, gh);
    let ys = [0.0, 1.0, 2.0, (gh / 2) as f64, (gh - 3) as f64, (gh - 2) as f64, (gh - 1) as f64];
    let xs = [0.0, 3.0, (gw / 3) as f64, (gw / 2) as f64, (gw - 1) as f64];
    let (mut lo_x, mut hi_x, mut lo_y, mut hi_y) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    let mut n = 0;
    let mut off = Vec::new();
    for world in [false, true] {
        let c = ctx(&f, world);
        for mode in [RouteMode::Land, RouteMode::Mixed] {
            for &sx in &xs {
                for &ex in &xs {
                    for &sy in &ys {
                        for &ey in &[0.0, (gh - 1) as f64, (gh / 2) as f64] {
                            if sx == ex && sy == ey {
                                continue;
                            }
                            let p = tools::civ_dijkstra_path(&c, sx, sy, ex, ey, mode);
                            n += 1;
                            // A run endpoint: the first or last point, or
                            // either side of a seam break.
                            let is_end = |k: usize| {
                                k == 0 || k + 1 == p.pts.len() || p.brks.contains(&k) || p.brks.contains(&(k + 1))
                            };
                            for (k, &(x, y)) in p.pts.iter().enumerate() {
                                let (mx, my) = ((gw - 1) as f64, (gh - 1) as f64);
                                if x < 0.0 || y < 0.0 || x > mx || y > my {
                                    off.push(format!(
                                        "({x}, {y}) at {k} on {mode:?} world={world} ({sx},{sy})->({ex},{ey})"
                                    ));
                                }
                                if !is_end(k) {
                                    lo_x = lo_x.min(x);
                                    hi_x = hi_x.max(x);
                                    lo_y = lo_y.min(y);
                                    hi_y = hi_y.max(y);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (lo_x, hi_x, lo_y, hi_y, n, off)
}

/// Asserts every returned point lies inside `[0, gw - 1] x [0, gh - 1]`, and
/// that sampled (non-endpoint) points
/// actually reached all four edges (so an all-empty result cannot pass).
fn assert_in_bounds(gw: usize, gh: usize) {
    let (lo_x, hi_x, lo_y, hi_y, n, off) = sweep(gw, gh);
    assert!(n > 100, "the sweep ran {n} routes");
    assert!(
        off.is_empty(),
        "{gw}x{gh}: {} off-grid points (x {lo_x}..{hi_x}, y {lo_y}..{hi_y}), first: {:?}",
        off.len(),
        &off[..off.len().min(4)]
    );
    assert!(lo_y >= 0.0, "{gw}x{gh}: a route point lies above the top edge, y = {lo_y}");
    assert!(hi_y <= (gh - 1) as f64, "{gw}x{gh}: a route point lies below the bottom edge, y = {hi_y}");
    assert!(lo_x >= 0.0, "{gw}x{gh}: a route point lies left of the map, x = {lo_x}");
    assert!(hi_x <= (gw - 1) as f64, "{gw}x{gh}: a route point lies right of the map, x = {hi_x}");
    // Positive control: routes do run along both edges, so the bounds above
    // were reached rather than vacuously satisfied.
    assert_eq!((lo_x, lo_y), (0.0, 0.0), "{gw}x{gh}: the sweep never touched the top-left edges");
    assert_eq!(
        (hi_x, hi_y),
        ((gw - 1) as f64, (gh - 1) as f64),
        "{gw}x{gh}: the sweep never touched the bottom-right edges"
    );
}

/// Protects: at 1:1 routing scale (a map up to 1024 cells wide) every point
/// `civ_dijkstra_path` returns lies on the grid. Measured before the fix
/// (2026-10-12, this sweep with the clamp reverted): sampled points with y from
/// -15 to 40 on this 1024 x 24 flat map, 50 124 off-grid points.
#[test]
fn routed_points_stay_on_a_1024_wide_map() {
    assert_in_bounds(1024, 24);
}

/// Protects: the same on a map wider than 1024 cells, where the routing grid
/// is coarser than the map (`sc = 0.5`) and the overshoot is larger. Measured
/// before the fix (same method): y from -23 to 51 on this 2048 x 24 flat map,
/// 141 556 off-grid points.
#[test]
fn routed_points_stay_on_a_2048_wide_map() {
    assert_in_bounds(2048, 24);
}

/// Protects: a small 1:1 map, where the bottom row's centre `gh - 0.5` rounds
/// to `gh` (one past the last row) without any overshoot at all. Measured
/// before the fix (same method): y from -3 to 26, 1 414 off-grid points.
#[test]
fn routed_points_stay_on_a_small_map() {
    assert_in_bounds(96, 24);
}

/// Protects: the x half of the clamp. The wide maps above run their routes
/// along x, so their overshoot is in y; this tall map runs them along y, so
/// they overshoot past the left and right edges instead. Measured before the
/// fix (same method): x from -15 to 40 and y up to 1024 on this 24 x 1024 map,
/// 69 892 off-grid points.
#[test]
fn routed_points_stay_on_a_tall_map() {
    assert_in_bounds(24, 1024);
}
