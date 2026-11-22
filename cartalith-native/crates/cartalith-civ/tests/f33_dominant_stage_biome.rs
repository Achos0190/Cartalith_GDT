//! F33 (owner finding 2026-11-21: "Planner misidentified biome for desert when no such biome was crossed").
//!
//! The reference's `_jpDeriveStages` absorbs a too-short stage into a neighbour and keeps the **neighbour's**
//! biome name. When the absorbed stretch is long enough to lift the survivor past the sliver floor, a stage
//! of 0.5 km of desert and 1.5 km of grassland is named "Hot Desert". `JpWorld::dominant_stage_biome` names it
//! for the biome over most of its km instead. These tests pin both labels on one hand-built route.

use std::collections::HashMap;

use cartalith_civ::{JpPlan, JpWorld};

const W: usize = 40;
const H: usize = 20;

/// A flat all-land world, 0.5 km a cell (`map_width_km` 20 over 40 cells), so the sliver floor is its 2.0 km
/// minimum. Row 9 is grassland (cart biome 5) except cell x = 20, which is Hot Desert (cart biome 10).
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
    let mut biome = vec![5u8; n];
    biome[9 * W + 20] = 10;
    Flat { field: vec![0.6; n], wb: vec![0; n], biome, terrain: vec![1; n], temp: vec![15.0; n], rain: vec![0.5; n] }
}

fn world<'a>(f: &'a Flat, road: &'a HashMap<(i64, i64), cartalith_civ::JpRoadCell>, dominant: bool) -> JpWorld<'a> {
    JpWorld {
        gw: W,
        gh: H,
        world: false,
        map_width_km: 20.0,
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
        dominant_stage_biome: dominant,
    }
}

/// Protects F33. Route (0,9)..(23,9): points 0-19 grass, point 20 desert (0.5 km), points 21-23 grass (1.5 km).
/// The 1.5 km grass sliver is absorbed into the desert chunk, which then reaches the 2.0 km floor and survives.
/// Reference label: "Hot Desert" for a 2.0 km stage that is 1.5 km grass. With the switch: "Steppe / Grassland".
/// Both runs cut the same stages with the same km (the switch renames, never re-cuts), and the long first stage
/// is grassland either way. MUTATION: make the switch read `c.biome` always, or pick the smallest biome, and the
/// second assertion fails; make it ignore the flag and the first fails.
#[test]
fn a_merged_stage_is_named_for_the_biome_over_most_of_its_km_when_asked() {
    let f = flat();
    let road = HashMap::new();
    let pts: Vec<(f64, f64)> = (0..=23).map(|x| (x as f64, 9.0)).collect();
    let plan = JpPlan::default();
    let run = |dominant: bool| {
        let w = world(&f, &road, dominant);
        cartalith_civ::jp_plan(&w, &pts, &plan, &HashMap::new(), &|_, _| 1.0).expect("the route plans")
    };
    let (reference, honest) = (run(false), run(true));
    let names = |p: &cartalith_civ::JpJourneyPlan| p.stages.iter().map(|s| (s.biome.clone(), (s.km * 100.0).round() as i64)).collect::<Vec<_>>();
    assert_eq!(names(&reference), vec![("Steppe / Grassland".to_string(), 950), ("Hot Desert".to_string(), 200)], "the reference label");
    assert_eq!(names(&honest), vec![("Steppe / Grassland".to_string(), 950), ("Steppe / Grassland".to_string(), 200)], "the dominant label");
}
