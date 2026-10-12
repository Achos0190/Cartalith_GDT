//! A zero-length route is recognised, not planned as a 0-day journey
//! (found-bug row of 2026-10-11, `OUTSTANDING_WORK.md` §2.3: "A zero-length
//! route plans as 0 days with a 0 km stage and no flag").
//!
//! `cartalith_civ::jp_route_is_zero_length` is the test the interactive planner
//! (`cartalith-godot`'s `jp_compute`) refuses with. These tests pin what it
//! accepts, that a route whose two stops coincide really does come out of the
//! router as such a route, and that `jp_plan` on it is the 0 km / 0 day answer
//! the refusal exists to keep off the screen (and that the saved-journey
//! timeline already turns into `NoTimeline::NoTravel`).

use std::collections::HashMap;

use cartalith_civ::journey_progress::{JourneyTimeline, NoTimeline};
use cartalith_civ::tools::{self, RouteContext, RouteMode};
use cartalith_civ::{JpPlan, JpWorld, jp_route_is_zero_length};

const W: usize = 40;
const H: usize = 20;

/// A flat, all-land world with mild weather, so a land route plans.
struct Flat {
    field: Vec<f32>,
    wb: Vec<u8>,
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

fn flat() -> Flat {
    let n = W * H;
    Flat {
        field: vec![0.6; n],
        wb: vec![0; n],
        // Biome 5 / terrain 1: an ordinary land cell (any land code serves;
        // the tests compare zero against non-zero, not a pinned day count).
        biome: vec![5; n],
        terrain: vec![1; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
    }
}

fn route_ctx(f: &Flat) -> RouteContext<'_> {
    RouteContext {
        field: &f.field,
        water_bodies: &f.wb,
        biome: None,
        river_order: None,
        places: &[],
        ways: &[],
        gw: W,
        gh: H,
        sea: 0.42,
        world: false,
        map_width_km: W as f64 * 12.0,
        corridors: None,
        flow: None,
        flow_thresh: 0.0,
    }
}

fn jp_world<'a>(f: &'a Flat, road: &'a HashMap<(i64, i64), cartalith_civ::JpRoadCell>) -> JpWorld<'a> {
    JpWorld {
        gw: W,
        gh: H,
        world: false,
        map_width_km: W as f64 * 12.0,
        sea_level: 0.42,
        peak_m: 4000.0,
        field: &f.field,
        cart_biome: &f.biome,
        cart_terrain: &f.terrain,
        temp: &f.temp,
        rain: &f.rain,
        flow_field: None,
        flow_thresh: 0.0,
        water_bodies: Some(&f.wb),
        territory: None,
        places: &[],
        road_cells: road,
        ocean_field: None,
        wind_field: None,
        river_net: None,
        measure_slope: false,
    }
}

/// Protects: the predicate's contract -- all points one point (including the
/// degenerate empty and one-point routes) is zero-length; any second distinct
/// point, however close, is not.
#[test]
fn the_predicate_accepts_only_routes_with_every_point_the_same() {
    assert!(jp_route_is_zero_length(&[]));
    assert!(jp_route_is_zero_length(&[(4.0, 7.0)]));
    assert!(jp_route_is_zero_length(&[(4.0, 7.0), (4.0, 7.0)]));
    assert!(jp_route_is_zero_length(&[(4.0, 7.0), (4.0, 7.0), (4.0, 7.0)]));
    assert!(!jp_route_is_zero_length(&[(4.0, 7.0), (5.0, 7.0)]));
    assert!(!jp_route_is_zero_length(&[(4.0, 7.0), (4.0, 7.0), (4.0, 7.25)]));
    // A route that leaves and comes back has length even though its ends meet.
    assert!(!jp_route_is_zero_length(&[(4.0, 7.0), (9.0, 7.0), (4.0, 7.0)]));
}

/// Protects: the premise of the refusal -- the router, given two coinciding
/// stops in every mode, returns a route of identical points (its fallback),
/// with `km` 0, so the planner really does receive one.
#[test]
fn coinciding_stops_route_to_a_zero_length_route() {
    let f = flat();
    let c = route_ctx(&f);
    for mode in [RouteMode::Land, RouteMode::Mixed] {
        let p = tools::civ_dijkstra_path(&c, 12.0, 9.0, 12.0, 9.0, mode);
        assert!(p.pts.len() >= 2, "{mode:?}: a drawable route of {} points", p.pts.len());
        assert!(jp_route_is_zero_length(&p.pts), "{mode:?}: {:?}", p.pts);
        assert_eq!(p.km, 0.0, "{mode:?}");
        // Positive control: one cell apart is a real route.
        let q = tools::civ_dijkstra_path(&c, 12.0, 9.0, 13.0, 9.0, mode);
        assert!(!jp_route_is_zero_length(&q.pts), "{mode:?}: {:?}", q.pts);
        assert!(q.km > 0.0, "{mode:?}");
    }
}

/// Protects: why the shell refusal is needed and why `jp_plan` itself is left
/// planning it -- the planner turns a zero-length route into 0 km and 0 travel
/// days (the plausible-looking non-answer), and the saved-journey timeline
/// already reads that as `NoTravel`. A one-cell route plans real, non-zero days.
#[test]
fn the_planner_answers_a_zero_length_route_with_zero_and_the_timeline_says_no_travel() {
    let f = flat();
    let road = HashMap::new();
    let w = jp_world(&f, &road);
    let plan = JpPlan::default();
    let z = cartalith_civ::jp_plan(&w, &[(12.0, 9.0), (12.0, 9.0)], &plan, &HashMap::new(), &|_, _| 1.0)
        .expect("the planner derives a stage for it");
    assert_eq!((z.km, z.days), (0.0, 0.0));
    assert_eq!(JourneyTimeline::from_plan(&z).err(), Some(NoTimeline::NoTravel));

    let one = cartalith_civ::jp_plan(&w, &[(12.0, 9.0), (13.0, 9.0)], &plan, &HashMap::new(), &|_, _| 1.0)
        .expect("a one-cell route plans");
    assert_eq!(one.km, 12.0, "one cell of a 40-cell, 480 km map");
    assert!(one.days > 0.0, "{}", one.days);
}
