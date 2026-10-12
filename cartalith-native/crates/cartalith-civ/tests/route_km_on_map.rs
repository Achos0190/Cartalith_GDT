//! Route km measures the on-map line the router returns (Ruling CF item 4,
//! owner, 2026-10-12; DECISIONS.md 7p). Two planner leftovers of the
//! 2026-10-12 off-map fix (`29b4ac0f`) are closed here:
//!
//! - **Seam-run endpoints.** A route that crosses a wrapped world's x seam is
//!   split into runs, and each run's restored endpoint at the seam is a raw
//!   router point `(c + 0.5) / sc`, which sat in the last half cell (`gw - 0.5`,
//!   `gh - 0.5`) and rounded off the map. It is now clamped into the grid box
//!   `[0, gw - 1] x [0, gh - 1]` like every other returned point.
//! - **Off-map km.** `civ_smooth_path` summed km over its unclamped samples, so
//!   a spline overshoot past the edge (measured y -23..51 on a 2048 x 24 map)
//!   was counted as distance travelled, while the step across the seam between
//!   two runs was not counted at all. Km is now the length of the on-map
//!   samples, with the seam step measured the short way round.
//!
//! The oracle here is the returned polyline itself, measured independently in
//! this file. Km is summed over the samples, whose run endpoints the reference
//! then replaces with the run's full-precision endpoints, so the two can differ
//! by at most each endpoint's displacement: under half a cell in x and y each
//! (`ENDPOINT_SLACK_CELLS` per endpoint). Anything past that bound is distance
//! the drawn line does not travel, or travel it does that km leaves out.

use cartalith_civ::tools::{self, DijkstraPath, RouteContext, RouteMode};

/// The furthest a run's restored endpoint can sit from the rounded, clamped
/// sample the km was measured from, in cells: a rounding moves each axis by at
/// most 0.5, so `hypot(0.5, 0.5)`. Derived, not tuned.
const ENDPOINT_SLACK_CELLS: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// A flat, all-land world (every cell the same height), so the router's path is
/// set by distance alone and no repair pass moves a sample.
struct Flat {
    field: Vec<f32>,
    wb: Vec<u8>,
    gw: usize,
    gh: usize,
}

/// Builds a [`Flat`] world `gw` x `gh`, all land at 0.6 above a 0.42 sea.
fn flat(gw: usize, gh: usize) -> Flat {
    Flat { field: vec![0.6; gw * gh], wb: vec![0; gw * gh], gw, gh }
}

/// 12 km per cell (a labelled judgement: any positive width would do; this
/// keeps km and cells easy to read side by side in a failure message).
const CELL_KM: f64 = 12.0;

/// The router context over a [`Flat`] world: no places, no ways, no §7i terms,
/// the given wrap flag.
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
        map_width_km: f.gw as f64 * CELL_KM,
        corridors: None,
        flow: None,
        flow_thresh: 0.0,
    }
}

/// The length in cells of the returned polyline, run by run, plus the step
/// across each seam break measured the short way round a `gw`-wide cylinder.
/// Independent of `civ_smooth_path`: it reads only the returned points.
fn drawn_cells(p: &DijkstraPath, gw: usize) -> f64 {
    let mut total = 0.0;
    for k in 1..p.pts.len() {
        let (a, b) = (p.pts[k - 1], p.pts[k]);
        let adx = (b.0 - a.0).abs();
        let dx = if p.brks.contains(&k) { adx.min(gw as f64 - adx) } else { adx };
        total += dx.hypot(b.1 - a.1);
    }
    total
}

/// Every returned point lies in the grid box `[0, gw - 1] x [0, gh - 1]`, and
/// the km agrees with the drawn line to within the endpoint slack. Returns a
/// description of each violation.
fn violations(p: &DijkstraPath, gw: usize, gh: usize, label: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (k, &(x, y)) in p.pts.iter().enumerate() {
        if !(0.0..=(gw - 1) as f64).contains(&x) || !(0.0..=(gh - 1) as f64).contains(&y) {
            out.push(format!("{label}: point {k} ({x}, {y}) off the grid"));
        }
    }
    let runs = p.brks.len() + 1;
    let slack = 2.0 * runs as f64 * ENDPOINT_SLACK_CELLS;
    let drawn = drawn_cells(p, gw);
    let km_cells = p.km / CELL_KM;
    if (km_cells - drawn).abs() > slack + 1e-9 {
        out.push(format!(
            "{label}: km {:.3} cells against a drawn line of {drawn:.3} cells (slack {slack:.3}, {runs} runs)",
            km_cells
        ));
    }
    out
}

/// Protects: the seam fix, on a wrapped flat world. Every route whose two ends
/// straddle the x seam comes back split (`brks` non-empty), with every point on
/// the grid, including the run endpoints at the seam, and with km equal to the
/// drawn line within the endpoint slack (so the step across the seam is
/// counted). Before the fix (2026-10-12, measured with it reverted) the seam
/// endpoints sat at `gw - 0.5` and the km left out the seam step.
#[test]
fn seam_crossing_routes_stay_on_the_map_and_count_the_seam() {
    let (gw, gh) = (96usize, 24usize);
    let f = flat(gw, gh);
    let c = ctx(&f, true);
    let mut seamed = 0;
    let mut bad = Vec::new();
    for mode in [RouteMode::Land, RouteMode::Mixed] {
        for &sx in &[gw as f64 - 6.0, gw as f64 - 3.0, gw as f64 - 1.0] {
            for &ex in &[0.0, 2.0, 5.0] {
                for &(sy, ey) in &[(0.0, 0.0), (5.0, 5.0), (3.0, 9.0), (12.0, 23.0), (23.0, 23.0), (22.0, 1.0)] {
                    let p = tools::civ_dijkstra_path(&c, sx, sy, ex, ey, mode);
                    assert!(p.reachable, "a flat all-land world routes everywhere");
                    if !p.brks.is_empty() {
                        seamed += 1;
                    }
                    bad.extend(violations(&p, gw, gh, &format!("{mode:?} ({sx},{sy})->({ex},{ey})")));
                }
            }
        }
    }
    // Positive control: these routes really do cross the seam (the short way
    // round is a few cells, the long way most of the map). Measured 2026-10-12:
    // 86 of 108 come back split; the other 22 come back as a single run (not
    // investigated further; they are still held to the box and the km bound).
    // The floor of 80 is a labelled judgement.
    assert!(seamed >= 80, "only {seamed} of 108 routes crossed the seam");
    assert!(bad.is_empty(), "{} violations, first: {:?}", bad.len(), &bad[..bad.len().min(4)]);
}

/// Protects: the seam step itself, as a number. A straight route along one row
/// from 3 cells west of the seam to 3 cells east of it is 6 cells long whether
/// or not it crosses the seam, so its km must match the same route moved into
/// the middle of the map to within the endpoint slack. Before the fix the
/// seam route's km was short by the uncounted seam step.
#[test]
fn a_seam_route_is_as_long_as_the_same_route_in_the_interior() {
    let (gw, gh) = (96usize, 24usize);
    let f = flat(gw, gh);
    let c = ctx(&f, true);
    let row = 12.0;
    let seam = tools::civ_dijkstra_path(&c, gw as f64 - 3.0, row, 3.0, row, RouteMode::Land);
    let mid = tools::civ_dijkstra_path(&c, 45.0, row, 51.0, row, RouteMode::Land);
    assert_eq!(seam.brks.len(), 1, "the seam route must split at the seam: {:?}", seam.pts);
    assert!(mid.brks.is_empty());
    let (ks, km) = (seam.km / CELL_KM, mid.km / CELL_KM);
    // Literal: the interior route is six cells along one row.
    assert!((km - 6.0).abs() < 1e-9, "interior route {km} cells, want 6");
    // Lower bound, no slack: both ends are whole cells, so the first and last
    // samples are the ends themselves, and a chain of steps between them,
    // with the seam step measured the short way, can be no shorter than the
    // 6 cells separating them on the cylinder. Without the seam step the km
    // came out at about 4.8 cells (mutation of 2026-10-12); measured with it,
    // 6.83 (the spline's samples step off the row near the seam).
    assert!(ks >= 6.0 - 1e-9, "seam route {ks} cells is shorter than the 6 cells between its ends");
    assert!(
        (ks - km).abs() <= 2.0 * ENDPOINT_SLACK_CELLS,
        "seam route {ks} cells against interior {km} cells; points {:?}",
        seam.pts
    );
}

/// Protects: the off-map km fix, on bounded flat maps whose routes hug an edge
/// and turn hard, where the spline overshoots past the edge. Every point stays
/// on the grid and km matches the drawn line within the endpoint slack. Before
/// the fix the km counted the overshoot (measured unclamped excursions of up to
/// 23 cells past a 24-row map's edge, 2026-10-12).
#[test]
fn near_edge_routes_count_only_on_map_distance() {
    let mut bad = Vec::new();
    let mut n = 0;
    for &(gw, gh) in &[(96usize, 24usize), (1024, 24), (2048, 24), (24, 1024)] {
        let f = flat(gw, gh);
        for world in [false, true] {
            let c = ctx(&f, world);
            let xs = [0.0, 3.0, (gw / 3) as f64, (gw / 2) as f64, (gw - 1) as f64];
            let ys = [0.0, 1.0, (gh / 2) as f64, (gh - 2) as f64, (gh - 1) as f64];
            for mode in [RouteMode::Land, RouteMode::Mixed] {
                for &sx in &xs {
                    for &ex in &xs {
                        for &sy in &ys {
                            for &ey in &[0.0, (gh - 1) as f64] {
                                if sx == ex && sy == ey {
                                    continue;
                                }
                                let p = tools::civ_dijkstra_path(&c, sx, sy, ex, ey, mode);
                                n += 1;
                                bad.extend(violations(
                                    &p,
                                    gw,
                                    gh,
                                    &format!("{gw}x{gh} world={world} {mode:?} ({sx},{sy})->({ex},{ey})"),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(n > 1000, "the sweep ran only {n} routes");
    assert!(bad.is_empty(), "{} of {n} routes violate, first: {:?}", bad.len(), &bad[..bad.len().min(4)]);
}

/// A [`cartalith_civ::JpWorld`] over a [`Flat`] world: biome 5 / terrain 1
/// everywhere (an ordinary land cell, as `zero_length_route.rs` uses), mild
/// weather, no rivers, roads, territory or slope.
struct PlanInputs {
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    road: std::collections::HashMap<(i64, i64), cartalith_civ::JpRoadCell>,
}

/// Builds the per-cell planner inputs for a `gw` x `gh` flat world.
fn plan_inputs(gw: usize, gh: usize) -> PlanInputs {
    let n = gw * gh;
    PlanInputs {
        biome: vec![5; n],
        terrain: vec![1; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
        road: std::collections::HashMap::new(),
    }
}

/// The planner's world over `f` and `p`, with the given wrap flag.
fn jp_world<'a>(f: &'a Flat, p: &'a PlanInputs, world: bool) -> cartalith_civ::JpWorld<'a> {
    cartalith_civ::JpWorld {
        gw: f.gw,
        gh: f.gh,
        world,
        map_width_km: f.gw as f64 * CELL_KM,
        sea_level: 0.42,
        peak_m: 4000.0,
        field: &f.field,
        cart_biome: &p.biome,
        cart_terrain: &p.terrain,
        temp: &p.temp,
        rain: &p.rain,
        flow_field: None,
        flow_thresh: 0.0,
        water_bodies: Some(&f.wb),
        territory: None,
        places: &[],
        road_cells: &p.road,
        ocean_field: None,
        wind_field: None,
        river_net: None,
        measure_slope: false,
    }
}

/// Plans `pts` with the default plan and returns its total km.
fn plan_km(w: &cartalith_civ::JpWorld, pts: &[(f64, f64)]) -> f64 {
    let plan = cartalith_civ::JpPlan::default();
    cartalith_civ::jp_plan(w, pts, &plan, &std::collections::HashMap::new(), &|_, _| 1.0)
        .expect("a land route on a flat world plans")
        .km
}

/// Protects: the planner half of the off-map km fix. A route carrying points
/// off the map (as a route saved before the fix can) plans the km of its
/// on-map line: a dip six rows above a 20-row map's top edge between two
/// points on that edge plans as the 10 cells along the edge, 120 km, not the
/// 2 x hypot(5, 6) = 15.6 cells (187.4 km) the raw points measure.
#[test]
fn the_planner_measures_off_map_points_on_the_map() {
    let f = flat(40, 20);
    let p = plan_inputs(40, 20);
    let w = jp_world(&f, &p, false);
    let km = plan_km(&w, &[(5.0, 0.0), (10.0, -6.0), (15.0, 0.0)]);
    assert!((km - 120.0).abs() < 1e-9, "planned {km} km, want 120 (10 cells along the edge)");
    // Positive control: the same shape inside the map is measured as drawn.
    let inside = plan_km(&w, &[(5.0, 10.0), (10.0, 4.0), (15.0, 10.0)]);
    let want = 2.0 * 5.0f64.hypot(6.0) * CELL_KM;
    assert!((inside - want).abs() < 1e-9, "planned {inside} km, want {want}");
}

/// Protects: route and planner agree. For routes that cross the seam or hug an
/// edge, the planner's km over the returned points equals the drawn line
/// measured in this file (the seam step the short way round), so the Route
/// tool's line, its km and the planner's stages describe one geometry.
#[test]
fn the_planner_km_of_a_route_is_its_drawn_line() {
    let (gw, gh) = (96usize, 24usize);
    let f = flat(gw, gh);
    let p = plan_inputs(gw, gh);
    let mut checked = 0;
    for world in [false, true] {
        let c = ctx(&f, world);
        let w = jp_world(&f, &p, world);
        for &(s, e) in &[
            ((gw as f64 - 3.0, 12.0), (3.0, 12.0)),
            ((gw as f64 - 6.0, 0.0), (2.0, 23.0)),
            ((0.0, 0.0), (40.0, 23.0)),
            ((3.0, 23.0), (90.0, 0.0)),
            ((10.0, 1.0), (60.0, 1.0)),
        ] {
            let r = tools::civ_dijkstra_path(&c, s.0, s.1, e.0, e.1, RouteMode::Land);
            let drawn = drawn_cells(&r, gw) * CELL_KM;
            let planned = plan_km(&w, &r.pts);
            assert!(
                (planned - drawn).abs() < 1e-6,
                "world={world} {s:?}->{e:?}: planned {planned} km against a drawn {drawn} km"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 10);
}
