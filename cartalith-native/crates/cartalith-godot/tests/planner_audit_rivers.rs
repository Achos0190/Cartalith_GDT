//! Ruling CD batch 2 (2026-10-11): River Transport on a real traced river,
//! measured on the planner audit's own world. **Ignored** measurement harness.
//!
//! What it is for. `PLANNER_AUDIT_2026-10-11.md` Q3 traced a 57-cell river on
//! `generate_terrain(WorldParams::defaults(256, 192, 24601))` and found River
//! Transport planned it as 14 land stages, every leg a walking portage
//! (`planner_audit_journey.rs`, `q3_real_river_channel_is_planned_as_land`,
//! which still measures that path: its `JpWorld` carries no river network).
//! This file re-traces the same river the same way and plans it twice per
//! direction -- without a river network (the audit's path) and with the one
//! `story_bridge::WorldGen::jp_world_parts` now builds
//! (`cartalith_civ::fresh_river_channels`, same arguments) -- and prints the
//! stages, conditions and days, so the before/after of Ruling CD item 1 is one
//! command.
//!
//! What it must never do. It pins no value: its numbers are observations. Its
//! assertions only guard that the fixture reached the code under test (the
//! trace is a real river; with the network at least one stage is a river).
//! Must never be un-ignored into the default suite: it generates a world.
//!
//! Run: `cargo test -p cartalith-godot --test planner_audit_rivers -- --ignored --nocapture`.

use std::collections::HashMap;

use cartalith_civ::{JpLegCalc, JpPlan, JpRiverNet, JpRoadCell, JpWorld};
use cartalith_engine::{generate_terrain, WorldParams};

const SEED: i32 = 24_601;
const GW: usize = 256;
const GH: usize = 192;

/// Protects (measurement only): Ruling CD item 1 on the audit's real river.
/// Traces the longest downstream run from a river head (each step to the
/// 8-neighbour of most discharge, exactly as the audit's Q3 did), plans it as
/// River Transport + Keelboat downstream and upstream, with and without the
/// river network, and prints stage lines, legs and travel days. Guards that the
/// trace is on river cells and that the network-on plan has a river stage.
#[test]
#[ignore = "measurement harness for Ruling CD batch 2; prints, pins nothing"]
fn q3_real_river_with_the_river_network() {
    let p = WorldParams::defaults(GW, GH, SEED);
    let ws = generate_terrain(&p);
    let wb = cartalith_civ::build_water_bodies(&ws.field, GW, GH, ws.sea_level, false, Some(&ws.rainfall)).classification;
    let cart_biome = cartalith_civ::build_cart_biome(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let cart_terrain = cartalith_civ::build_cart_terrain(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km);
    let (order, recv) = cartalith_civ::fresh_river_channels(
        &ws.field, &ws.flow_discharge, GW, GH, ws.sea_level, false, p.river_density, p.map_width_km, ws.integrated_drainage,
    );
    let f = &ws.flow_discharge;

    // The audit's trace, verbatim in method.
    let trace_from = |s: usize| -> Vec<usize> {
        let mut path = vec![s];
        let mut c = s;
        loop {
            let (x, y) = ((c % GW) as i64, (c / GW) as i64);
            let mut best = c;
            for dy in -1..=1i64 {
                for dx in -1..=1i64 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= GW as i64 || ny >= GH as i64 {
                        continue;
                    }
                    let n = ny as usize * GW + nx as usize;
                    if f[n] > f[best] {
                        best = n;
                    }
                }
            }
            if best == c || wb[best] != 0 || path.len() > 4 * GW {
                break;
            }
            path.push(best);
            c = best;
        }
        path
    };
    let mut best: Vec<usize> = Vec::new();
    for i in 0..GW * GH {
        if wb[i] == 0 && (f[i] as f64) > flow_thresh && (f[i] as f64) < 1.5 * flow_thresh {
            let t = trace_from(i);
            if t.len() > best.len() {
                best = t;
            }
        }
    }
    assert!(best.len() >= 20, "the fixture must have a river at least 20 cells long, got {}", best.len());
    assert!(best.iter().all(|&i| (f[i] as f64) > flow_thresh), "every traced cell must be a river cell");
    let navigable = best.iter().filter(|&&i| order[i] >= cartalith_civ::river_class::NAVIGABLE_ORDER).count();
    let orders: Vec<i16> = best.iter().map(|&i| order[i]).collect();
    println!(
        "river: {} cells, flow {:.1} -> {:.1} (thresh {:.1}); {} cells at order >= {}; orders {:?}",
        best.len(), f[best[0]], f[best[best.len() - 1]], flow_thresh, navigable, cartalith_civ::river_class::NAVIGABLE_ORDER, orders
    );
    let down: Vec<(f64, f64)> = best.iter().map(|&i| ((i % GW) as f64, (i / GW) as f64)).collect();
    let up: Vec<(f64, f64)> = down.iter().rev().copied().collect();

    let road: HashMap<(i64, i64), JpRoadCell> = HashMap::new();
    let world = |net: bool| JpWorld {
        gw: GW,
        gh: GH,
        world: false,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        peak_m: p.peak_m,
        field: &ws.field,
        cart_biome: &cart_biome,
        cart_terrain: &cart_terrain,
        temp: &ws.temperature,
        rain: &ws.rainfall,
        flow_field: Some(&ws.flow_discharge),
        flow_thresh,
        water_bodies: Some(&wb),
        territory: None,
        places: &[],
        road_cells: &road,
        ocean_field: None,
        wind_field: None,
        river_net: net.then_some(JpRiverNet { order: &order, recv: &recv }),
    };
    let plan = JpPlan { transport: "River Transport".into(), vessel: "Keelboat".into(), ..JpPlan::default() };
    for net in [false, true] {
        let w = world(net);
        for (label, pts) in [("downstream", &down), ("upstream", &up)] {
            let jp = cartalith_civ::jp_plan(&w, pts, &plan, &HashMap::new(), &|_, _| 1.0).expect("the river plans");
            let stages: Vec<String> = jp
                .stages
                .iter()
                .map(|s| format!("[{} | {} | {} | {:.1} km]", s.cat, s.terrain, s.route_cond, s.km))
                .collect();
            let legs: Vec<String> = jp
                .results
                .iter()
                .map(|r| match &r.calc {
                    Ok(JpLegCalc::Land(l)) => format!("land {} portage={} {:.1} km/d", l.transport_label, l.portage, l.daily_km),
                    Ok(JpLegCalc::Water(wc)) => format!("water {} {:.1} km/d", wc.transport_label, wc.daily_km),
                    Err(e) => format!("blocked: {}", e.reason),
                })
                .collect();
            let n_river = jp.stages.iter().filter(|s| s.cat == "river").count();
            let n_land = jp.stages.iter().filter(|s| s.cat == "land").count();
            println!(
                "network {net} {label}: {} stages ({n_river} river, {n_land} land); total {:?} travel {:.3} days\n  {}\n  {:?}",
                jp.stages.len(), jp.total_days, jp.travel_days, stages.join(" "), legs
            );
            if net {
                assert!(n_river > 0, "with the network the traced river must plan at least one river stage");
            }
        }
    }
}
