//! Found-bug sweep 2026-10-11, item R8: an **ignored measurement**, not a
//! contract.
//!
//! What it is for. The Ruling CD end-to-end probe (`STATUS.md`, 2026-10-11)
//! reported that on seed 707 "a committed sea route there came out about 0.1
//! day long", against a committed 462.5 km sea route of 4.752 days on seed 404,
//! and filed it as "not investigated; possibly snapping". This harness replays
//! the shell's path for that measurement in Rust so the cause can be read off
//! numbers instead of guessed: the app's parameters (`params::defaults`), a
//! 192 x 128 world at 800 km (the probe's stated setting), the real
//! `compute_civilisation`, the longest ocean row, the Route tool's own stop
//! snap (`civ_snap_point` at `civ_snap_radius`, what `route_append_stop` does),
//! a Water-mode `civ_join_dijkstra_segs` over `RouteInputs::build`'s context
//! (what `route_commit` does), then `jp_plan` over a `JpWorld` assembled the
//! way `story_bridge::WorldGen::jp_world_parts`/`jp_world` assemble it.
//!
//! What it must never do: assert a measured number. It asserts only that the
//! fixture reached the code under test (an ocean row exists, the route has
//! points), and prints everything else.
//!
//! Run (alone, one cargo at a time):
//! `cargo test -p cartalith-godot --lib bugsweep_seed707 -- --ignored --nocapture`

use std::collections::HashMap;

use cartalith_civ::tools::{self, RouteContext, RouteMode, WayRef};
use cartalith_civ::{JpPlan, JpWorld};

use crate::infra_tools_bridge::RouteInputs;
use crate::journey_bridge::JourneyWorld;

/// The probe's setting: 192 x 128 cells over 800 km (`STATUS.md`'s seed-404
/// line, "800 km, 192x128"). A judgement that seed 707 ran at the same size,
/// since the probe ran one script over several seeds.
const GW: usize = 192;
const GH: usize = 128;
const WIDTH_KM: f64 = 800.0;

/// The longest horizontal run of ocean cells (`water_bodies == 1`) on any row:
/// `(row, first x, last x)`. `None` for a world with no ocean.
fn longest_ocean_row(wb: &[u8]) -> Option<(usize, usize, usize)> {
    let mut best: Option<(usize, usize, usize)> = None;
    for y in 0..GH {
        let mut x = 0;
        while x < GW {
            if wb[y * GW + x] != 1 {
                x += 1;
                continue;
            }
            let x0 = x;
            while x < GW && wb[y * GW + x] == 1 {
                x += 1;
            }
            let x1 = x - 1;
            if best.is_none_or(|(_, a, b)| x1 - x0 > b - a) {
                best = Some((y, x0, x1));
            }
        }
    }
    best
}

/// One seed, end to end, printed. See the module doc for each step's source.
/// `villages` sets `civ.villages` (the probe's own setting is not recorded; both
/// are run).
fn replay(seed: i32, villages: bool) {
    let mut p = crate::params::defaults();
    p.civ.villages = villages;
    println!("R8 ---- seed {seed}, villages {villages} ----");
    p.gw = GW;
    p.gh = GH;
    p.tect.seed = seed;
    p.map_width_km = WIDTH_KM;
    p.use_gpu = false;
    let ws = cartalith_engine::generate_terrain(&p);
    let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, GW, GH, p.world, ws.sea_level, &p);
    let civ = crate::compute_civilisation(
        &ws, GW, GH, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false, &mut Vec::new(), None,
    );
    let inputs = RouteInputs::build(&ws, GW, GH, p.world, p.map_width_km, p.river_density, RouteMode::Water, None);
    let (row, x0, x1) = longest_ocean_row(&inputs.water_bodies).expect("the fixture world has ocean");
    println!(
        "R8 seed {seed}: sea level {:.3}, {} settlements, {} ways, {} sea lanes; longest ocean row y={row} x={x0}..={x1} ({} cells)",
        ws.sea_level,
        civ.settlements.len(),
        civ.ways.len(),
        civ.sea_routes.len(),
        x1 - x0 + 1
    );

    // `snap_point`: generated ways plus (none yet) hand-drawn ways, settlements.
    let snap_ways: Vec<WayRef> = civ.ways.iter().map(WayRef::from).collect();
    let radius = tools::civ_snap_radius(GW);
    let raw = [(x0 as f64, row as f64), (x1 as f64, row as f64)];
    let snapped: Vec<(f64, f64)> = raw.iter().map(|&(x, y)| tools::civ_snap_point(&civ.settlements, &snap_ways, x, y, radius)).collect();
    println!("R8 seed {seed}: snap radius {radius:.2}; raw stops {raw:?} -> snapped {snapped:?}");
    for (k, &(sx, sy)) in snapped.iter().enumerate() {
        let i = (sy.round() as usize).min(GH - 1) * GW + (sx.round() as usize).min(GW - 1);
        println!("R8 seed {seed}:   stop {k} lands on class {} (0 land, 1 ocean, 2 lake)", inputs.water_bodies[i]);
    }

    // `route_commit`'s context (credited edges included, as there).
    let credited = tools::civ_credited_road_edges(&civ.ways, &civ.road_edges);
    let edge_polys = tools::civ_road_edge_polylines(&credited, GW);
    let way_refs: Vec<WayRef> = tools::civ_network_way_refs(&civ.ways, &[], &edge_polys);
    let ctx = RouteContext {
        field: &ws.field,
        water_bodies: &inputs.water_bodies,
        biome: inputs.biome.as_deref(),
        river_order: inputs.river_order.as_deref(),
        places: &civ.settlements,
        ways: &way_refs,
        gw: GW,
        gh: GH,
        sea: ws.sea_level,
        world: p.world,
        map_width_km: p.map_width_km,
        corridors: inputs.corridors.as_deref(),
        flow: Some(&ws.flow_discharge),
        flow_thresh: cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km),
    };

    // The planner's world, as `jp_world_parts` + `jp_world` build it.
    let mut jw = JourneyWorld::build(&ws.field, &civ.water_bodies, &ws.temperature, &ws.rainfall, GW, GH, p.world, ws.sea_level, &civ.ways, &civ.settlements);
    jw.road_cells = cartalith_civ::jp_road_cells(&civ.ways, &[], &civ.road_edges, GW);
    let (river_order, river_recv) = cartalith_civ::fresh_river_channels(
        &ws.field, &ws.flow_discharge, GW, GH, ws.sea_level, p.world, p.river_density, p.map_width_km, ws.integrated_drainage,
    );
    let slope = cartalith_civ::build_raw_slope_field(&ws.field, GW, GH, p.world);
    let world = JpWorld {
        gw: GW,
        gh: GH,
        world: p.world,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        peak_m: p.peak_m,
        field: &ws.field,
        cart_biome: &jw.cart_biome,
        cart_terrain: &jw.cart_terrain,
        temp: &ws.temperature,
        rain: &ws.rainfall,
        flow_field: Some(&ws.flow_discharge),
        flow_thresh: cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km),
        water_bodies: Some(&civ.water_bodies),
        territory: Some(&civ.territory),
        places: &jw.places,
        road_cells: &jw.road_cells,
        ocean_field: Some(&o),
        wind_field: Some(&w),
        river_net: Some(cartalith_civ::JpRiverNet { order: &river_order, recv: &river_recv }),
        slope_field: Some(&slope),
    };
    let plan = JpPlan { transport: "Sea Faring".into(), vessel: "Cog".into(), ..JpPlan::default() };

    // The candidate mechanism: two stops that snap to the SAME target (a
    // coastal settlement pin within the radius of both clicks) commit a route
    // of near-zero length. What does a plan over such a route read?
    let degenerate = vec![snapped[0], snapped[0]];
    // And: how many ocean cells of the row lie within the snap radius of a
    // settlement, i.e. would be pulled onto land by `route_append_stop`?
    let pulled = (x0..=x1)
        .filter(|&x| tools::civ_snap_point(&civ.settlements, &snap_ways, x as f64, row as f64, radius) != (x as f64, row as f64))
        .count();
    println!("R8 seed {seed}: {pulled} of the row's {} ocean cells snap away from where they were clicked", x1 - x0 + 1);
    for (label, stops) in [
        ("snapped (what route_append_stop stores)", snapped.clone()),
        ("raw (no snap)", raw.to_vec()),
        ("degenerate (both stops on one point)", degenerate),
    ] {
        let j = tools::civ_join_dijkstra_segs(&ctx, &stops, RouteMode::Water);
        assert!(!j.pts.is_empty(), "a committed route has points");
        let first = j.pts.first().copied().unwrap_or_default();
        let last = j.pts.last().copied().unwrap_or_default();
        println!(
            "R8 seed {seed} {label}: route {} pts, {:.1} km, unreachable legs {}, first {first:?} last {last:?}",
            j.pts.len(),
            j.km,
            j.unreachable_legs
        );
        match cartalith_civ::jp_plan(&world, &j.pts, &plan, &HashMap::new(), &|_, _| 1.0) {
            Some(jp) => {
                let st: Vec<String> = jp.stages.iter().map(|s| format!("[{} | {} | {:.1} km]", s.cat, s.terrain, s.km)).collect();
                println!(
                    "R8 seed {seed} {label}: plan {:.2} km, total {:?} days, travel {:.3} days, blocked {:?}; stages {}",
                    jp.km,
                    jp.total_days,
                    jp.travel_days,
                    jp.blocked_idx,
                    st.join(" ")
                );
            }
            None => println!("R8 seed {seed} {label}: the planner returned no plan"),
        }
    }
}

/// Protects: nothing by assertion -- an ignored measurement for item R8 (see
/// the module doc); it prints the route and plan for seed 707 against the
/// seed-404 control.
#[test]
#[ignore = "R8 measurement: generates two worlds and prints; pins nothing"]
fn bugsweep_seed707_sea_route() {
    for seed in [404, 707] {
        for villages in [false, true] {
            replay(seed, villages);
        }
    }
}
