//! The wall-gap census for owner Ruling EW item 1, last point (2026-11-13,
//! `LARGE_ITEM_RULINGS.md`): *"walls were expensive so a lot of free space
//! between the wall and the first city on the inside is unrealistic"*.
//!
//! **Why here.** The measurement must be of the towns the app lays out: a world
//! generated through `params::defaults()`, every settlement laid out through
//! `um_place_context_with` + `run_layout` exactly as `urban_layouts` does (with
//! the ruling faction's faith), so the switch under test is the one the app
//! reads.
//!
//! **What it prints.** For every walled town (a settlement whose layout has a
//! wall ring) of each seed, with `wall_hug` OFF then ON on the same settlement:
//! the building count, the walled area, the median and 90th-percentile
//! wall-to-nearest-building distance and the free share
//! (`cartalith_urban::wallgap`), and the layout's own population (`pop`, the
//! head count of built parcels). Both columns come from the same code path; only
//! the switch differs.
//!
//! `#[ignore]`d: a measurement, not a gate. Run alone, one thread:
//! `cargo test -p cartalith-godot --lib wall_gap_census -- --ignored --nocapture --test-threads 1`.
//! The gate is `wall_hug_tests` in `cartalith-urban`; this prints what to quote.

use crate::fh_r2_baseline::{GW, generate_world};
use cartalith_civ::urban_adapter::{PlaceOverrides, UrbanWorld, run_layout, um_place_context_with};
use cartalith_civ::urban_adapter::{Vec2, wallgap::wall_gap};

/// The seeds the owner's request names (`CITY_MAP_LOOK_SCOPE.md` task for
/// Ruling EW's wall hug), at the harness' usual 800 km width and 6 factions.
const SEEDS: [i32; 3] = [483920, 912345, 246371];

/// One town's numbers, for one switch state.
struct Row {
    buildings: usize,
    pop: f64,
    area_m2: f64,
    median_m: f64,
    p90_m: f64,
    free_share: f64,
}

fn measure(l: &cartalith_civ::urban_adapter::UrbanLayout) -> Option<Row> {
    let ring = l.wall.ring.as_deref()?;
    let polys: Vec<&[Vec2]> = l.buildings.iter().map(|b| b.poly.as_slice()).collect();
    let g = wall_gap(ring, l.wall.land_arc.as_deref(), &polys)?;
    Some(Row {
        buildings: l.buildings.len(),
        pop: l.pop,
        area_m2: g.area_m2,
        median_m: g.median_m,
        p90_m: g.p90_m,
        free_share: g.free_share,
    })
}

/// One layout as JSON (ring, land arc, gates, buildings, streets, citadel) for the frame script, to `dir`.
/// Hand-written because the census needs no serde and the shape is five lists of points.
fn dump(dir: &str, name: &str, l: &cartalith_civ::urban_adapter::UrbanLayout) {
    let pl = |v: &[Vec2]| -> String {
        let pts: Vec<String> = v.iter().map(|p| format!("[{:.1},{:.1}]", p.x, p.y)).collect();
        format!("[{}]", pts.join(","))
    };
    let ring = l.wall.ring.as_deref().map(&pl).unwrap_or_else(|| "[]".into());
    let arc = l.wall.land_arc.as_deref().map(&pl).unwrap_or_else(|| "[]".into());
    let gates: Vec<Vec2> = l.wall.gates.iter().map(|g| g.pt).collect();
    let blds: Vec<String> = l.buildings.iter().map(|b| pl(&b.poly)).collect();
    let edges: Vec<String> = l.edges.iter().map(|e| format!("[[{:.1},{:.1}],[{:.1},{:.1}],\"{}\"]", e.a.x, e.a.y, e.b.x, e.b.y, e.cls)).collect();
    let pars: Vec<String> = l.parcels.iter().map(|p| format!("[{},\"{}\"]", pl(&p.poly), p.district)).collect();
    let cit = l.citadel.as_ref().map(|c| pl(&c.wall)).unwrap_or_else(|| "[]".into());
    let json = format!(
        "{{\"ring\":{ring},\"arc\":{arc},\"gates\":{},\"buildings\":[{}],\"edges\":[{}],\"citadel\":{cit},\"parcels\":[{}]}}",
        pl(&gates), blds.join(","), edges.join(","), pars.join(","),
    );
    std::fs::write(format!("{dir}/{name}.json"), json).expect("dump");
}

#[test]
#[ignore = "measurement; see the module doc"]
fn wall_gap_before_and_after() {
    let mut all_off: Vec<Row> = Vec::new();
    let mut all_on: Vec<Row> = Vec::new();
    let (mut refit_off, mut refit_on): (Vec<Row>, Vec<Row>) = (Vec::new(), Vec::new());
    for seed in SEEDS {
        let (ws, civ, gh, p) = generate_world(seed, 800.0, 6, GW);
        let river_polys = match (ws.stream_order.as_ref(), ws.channels.as_ref()) {
            (Some(order), Some(ch)) => cartalith_hydrology::trace_river_polylines(order, &ch.recv, GW, gh, 1),
            _ => Vec::new(),
        };
        let uw = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &river_polys,
            gw: GW,
            gh,
            sea_level: ws.sea_level,
            map_width_km: p.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(GW, gh, GW, p.map_width_km),
            world_seed: seed,
            river_kind_from_map: p.urban_river_kind_map,
            map_water: None,
        };
        println!("seed {seed}: {} settlements", civ.settlements.len());
        println!("  idx  pop  sw | bld  area_ha | OFF med p90 free | ON med p90 free bld pop");
        for (i, s) in civ.settlements.iter().enumerate() {
            let mut o = PlaceOverrides { faith: civ.town_faith(s), ..Default::default() };
            o.wall_hug = false;
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let Some(off) = run_layout(&ctx, None) else { continue };
            let Some(r_off) = measure(&off) else { continue };
            o.wall_hug = true;
            let ctx = um_place_context_with(&uw, s, &civ.ways, &o);
            let on = run_layout(&ctx, None).expect("the same town lays out with the switch on");
            let r_on = measure(&on);
            match &r_on {
                Some(n) => println!(
                    "  {i:3} {:5} {:2} | {:4} {:6.1} | {:5.1} {:5.1} {:.3} | {:5.1} {:5.1} {:.3} {:4} {:.0}/{:.0} {}",
                    s.pop, s.placement.kind as u8, r_off.buildings, r_off.area_m2 / 1e4,
                    r_off.median_m, r_off.p90_m, r_off.free_share,
                    n.median_m, n.p90_m, n.free_share, n.buildings, n.pop, r_off.pop,
                    match on.wall_hug { Some(h) => format!("{h:?}"), None => "-".into() },
                ),
                None => println!("  {i:3} {:5} | OFF only (ON has no measurable wall)", s.pop),
            }
            if let Ok(dir) = std::env::var("WALLGAP_DUMP") {
                dump(&dir, &format!("{seed}_{i}_off"), &off);
                dump(&dir, &format!("{seed}_{i}_on"), &on);
            }
            let refit = matches!(on.wall_hug, Some(cartalith_civ::urban_adapter::HugOutcome::Refit { .. }));
            if refit {
                refit_off.push(Row { ..r_off });
                if let Some(n) = &r_on {
                    refit_on.push(Row { ..*n });
                }
            }
            all_off.push(r_off);
            if let Some(n) = r_on {
                all_on.push(n);
            }
        }
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
    };
    for (name, rows) in [("OFF all", &all_off), ("ON all", &all_on), ("OFF refit-only", &refit_off), ("ON refit-only", &refit_on)] {
        let mut m: Vec<f64> = rows.iter().map(|r| r.median_m).collect();
        let mut p: Vec<f64> = rows.iter().map(|r| r.p90_m).collect();
        let mut f: Vec<f64> = rows.iter().map(|r| r.free_share).collect();
        println!(
            "{name}: {} walled towns; median-of-medians {:.1} m, median p90 {:.1} m, median free share {:.3}, total pop {:.0}, buildings {}",
            rows.len(), med(&mut m), med(&mut p), med(&mut f),
            rows.iter().map(|r| r.pop).sum::<f64>(), rows.iter().map(|r| r.buildings).sum::<usize>(),
        );
    }
}
