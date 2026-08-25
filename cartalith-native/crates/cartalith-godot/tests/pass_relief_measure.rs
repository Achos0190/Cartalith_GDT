//! `DECISIONS.md` §7i, measured on **real generated terrain** rather than a
//! synthetic ridge.
//!
//! The unit tests in `cartalith-civ::tools` prove the saddle test fires on a
//! saddle and nowhere else. What they cannot answer is the question that
//! decides whether the term is worth having: on a heightmap this generator
//! actually produces, how *often* does it fire, and does a route notice?
//!
//! This test answers both, and asserts the answer rather than printing it —
//! so a future retune of the terrain pipeline that quietly makes the term
//! dead (or makes it fire on half the map, which would be worse) fails here
//! instead of being discovered by eye.

use cartalith_engine::{generate_terrain, WorldParams};

fn world(seed: i32, gw: usize, gh: usize) -> cartalith_engine::WorldState {
    generate_terrain(&WorldParams::defaults(gw, gh, seed))
}

#[test]
fn the_corridor_relief_is_sparse_and_reaches_real_routes() {
    let ws = world(24_601, 512, 384);
    let (gw, gh) = (512usize, 384usize);
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, false, Some(&ws.rainfall));
    let slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, false);
    let corr = cartalith_civ::build_route_corridors(
        &ws.field,
        &slope,
        Some(&ws.flow_discharge),
        gw,
        gh,
        ws.sea_level,
        false,
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, 5120.0),
    );

    // The detector must be SPARSE -- the reference's own calibration rule for
    // a term of this shape ("~zero almost everywhere and spike only where the
    // thing it names actually is"; reference line 5940). A term that fired on
    // a third of the map would be a broad discount on hilly ground, not pass
    // detection, and would quietly reshape every route.
    let land = (0..gw * gh).filter(|&i| wb.classification[i] == 0).count();
    let hits = (0..gw * gh).filter(|&i| wb.classification[i] == 0 && corr[i] > 0.0).count();
    let share = hits as f64 / land.max(1) as f64;
    let mean: f64 = (0..gw * gh).filter(|&i| wb.classification[i] == 0).map(|i| corr[i] as f64).sum::<f64>() / land.max(1) as f64;
    let strong = (0..gw * gh).filter(|&i| wb.classification[i] == 0 && corr[i] > 0.5).count() as f64 / land.max(1) as f64;
    // Measured on this fixture, 2026-08-26: 169 558 land cells, 30.8% carrying
    // SOME corridor value, mean 0.064, and 1.02% above half strength. That is
    // the shape the term needs: the average land cell gets a 4% slope discount
    // (1 - 0.60 x 0.064), which changes nothing, while the ~1% that are real
    // pinch points get 30-60% off. The bounds below are deliberately loose
    // around those numbers -- they exist to catch the field going dead or
    // going broad after a terrain-pipeline retune, not to pin a value.
    assert!(mean > 0.005, "the corridor field found essentially nothing: mean {mean:.4}");
    assert!(mean < 0.15, "the corridor field is a broad discount, not pass detection: mean {mean:.4}");
    assert!(strong > 0.001, "no genuine pinch points at all: {strong:.4} of land above half strength");
    assert!(strong < 0.05, "too much of the map is a strong pass: {strong:.4}");
    assert!(share < 0.50, "over half of all land carries a corridor value: {share:.4}");

    let plain = cartalith_civ::tools::RouteContext {
        field: &ws.field,
        water_bodies: &wb.classification,
        biome: None,
        river_order: None,
        places: &[],
        ways: &[],
        gw,
        gh,
        sea: ws.sea_level,
        world: false,
        map_width_km: 5120.0,
        corridors: None,
    };
    let aware = cartalith_civ::tools::RouteContext { corridors: Some(&corr), ..plain };

    // Four long land crossings on one world. Endpoints are chosen as the
    // land cells nearest four fixed fractions of the map, so this is a fixed
    // scenario, not a search that could quietly find nothing to do.
    let land_near = |fx: f64, fy: f64| -> (f64, f64) {
        let (tx, ty) = (fx * gw as f64, fy * gh as f64);
        let mut best = None;
        let mut bd = f64::INFINITY;
        for y in 0..gh {
            for x in 0..gw {
                if wb.classification[y * gw + x] != 0 {
                    continue;
                }
                let d = (x as f64 - tx).powi(2) + (y as f64 - ty).powi(2);
                if d < bd {
                    bd = d;
                    best = Some((x as f64, y as f64));
                }
            }
        }
        best.expect("a generated world has land")
    };
    let pairs = [
        (land_near(0.10, 0.20), land_near(0.90, 0.80)),
        (land_near(0.10, 0.80), land_near(0.90, 0.20)),
        (land_near(0.50, 0.10), land_near(0.50, 0.90)),
        (land_near(0.15, 0.50), land_near(0.85, 0.50)),
    ];

    let mut moved = 0;
    let mut reachable = 0;
    for (s, e) in pairs {
        let a = cartalith_civ::tools::civ_dijkstra_path(&plain, s.0, s.1, e.0, e.1, cartalith_civ::tools::RouteMode::Mixed);
        let b = cartalith_civ::tools::civ_dijkstra_path(&aware, s.0, s.1, e.0, e.1, cartalith_civ::tools::RouteMode::Mixed);
        assert_eq!(a.reachable, b.reachable, "the relief must never change WHETHER a route exists");
        if a.reachable {
            reachable += 1;
        }
        if a.pts != b.pts {
            moved += 1;
        }
    }
    assert!(reachable >= 3, "the fixture must actually route: only {reachable}/4 connected");
    assert!(
        moved >= 1,
        "the saddle term never reached a single route on real terrain -- it is dead weight, \
         and §7i's claim that routes are now pass-aware would be false"
    );
}
