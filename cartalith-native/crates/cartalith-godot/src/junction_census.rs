//! The census for owner finding F27 / Ruling GL (2026-11-22): settlements founded where roads
//! converge. Measurement only; `#[ignore]`d (it generates 2 048-cell worlds). Run alone, release:
//! `cargo test --release -p cartalith-godot --lib junction_census -- --ignored --nocapture --test-threads 1`.
//! `JUNCTION_DUMP=<dir>` writes each world's settlements (founded ones flagged), ways and rivers as JSON
//! for the frame script (`docs/evidence/junctions-2026-11-22/frames.py`).
//!
//! One row per world: `civ.junction_settlements` off then on, everything else the app's defaults.
//! The founded set is the on-run's settlements absent from the off-run (the pass has its own stream
//! and runs after the village pass, so nothing else moves; a mismatch in `moved` would say otherwise).

use cartalith_civ::junction::{civ_junctions, midness};
use cartalith_civ::{NamedSettlement, SettlementKind};

fn world(seed: i32, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = crate::params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = seed as _;
    p.map_width_km = 800.0;
    p.use_gpu = false;
    p.civ.villages = true;
    p
}

fn larger(s: &NamedSettlement) -> bool {
    matches!(s.placement.kind, SettlementKind::Metropolis | SettlementKind::Capital | SettlementKind::City | SettlementKind::Town)
}

#[test]
#[ignore = "measurement: generates 2048-cell worlds"]
fn junction_census() {
    for (seed, gw, gh) in [(483_920, 384, 288), (912_345, 384, 288), (246_371, 384, 288), (483_920, 2048, 1311), (912_345, 2048, 1311), (246_371, 2048, 1311)] {
        let mut p = world(seed, gw, gh);
        let ws = cartalith_engine::generate_terrain(&p);
        p.civ.junction_settlements = false;
        let off = crate::way_census::civ_for(&ws, &p);
        p.civ.junction_settlements = true;
        let on = crate::way_census::civ_for(&ws, &p);
        let cell_km = p.map_width_km / gw as f64;
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
        let reach = (cartalith_civ::junction::JUNCTION_RIVER_REACH_KM / cell_km).ceil().max(2.0) as i64;
        let near_river = |x: f64, y: f64| {
            let (cx, cy) = (x.round() as i64, y.round() as i64);
            (-reach..=reach).any(|dy| (-reach..=reach).any(|dx| {
                let (nx, ny) = (cx + dx, cy + dy);
                nx >= 0 && ny >= 0 && (nx as usize) < gw && (ny as usize) < gh && ws.flow_discharge[ny as usize * gw + nx as usize] as f64 >= thresh
            }))
        };
        // The junctions the pass sees: the base network's ways (the village tracks are no road
        // network) against the base places (villages have `pop` 0), as at generation.
        let base_ways: Vec<_> = off.ways.iter().filter(|w| w.way_type != cartalith_civ::WayType::Ancient).cloned().collect();
        let base_places: Vec<_> = off.settlements.iter().filter(|s| s.pop > 0).cloned().collect();
        let js = civ_junctions(&base_ways, &base_places);
        let j_river = js.iter().filter(|j| near_river(j.x, j.y)).count();
        let known: std::collections::HashSet<(usize, usize)> = off.settlements.iter().map(|s| (s.placement.x, s.placement.y)).collect();
        let founded: Vec<&NamedSettlement> = on.settlements.iter().filter(|s| !known.contains(&(s.placement.x, s.placement.y))).collect();
        let moved = off.settlements.iter().filter(|s| !on.settlements.iter().any(|t| t.placement.x == s.placement.x && t.placement.y == s.placement.y)).count();
        let f_river = founded.iter().filter(|s| near_river(s.placement.x as f64, s.placement.y as f64)).count();
        // Distances to the two nearest larger settlements (km) and the midness, over the founded set.
        let (mut d1, mut mid) = (Vec::new(), Vec::new());
        for s in &founded {
            let mut d: Vec<f64> = on.settlements.iter().filter(|t| larger(t)).map(|t| (t.placement.x as f64 - s.placement.x as f64).hypot(t.placement.y as f64 - s.placement.y as f64) * cell_km).collect();
            d.sort_by(|a, b| a.total_cmp(b));
            d1.push(d.first().copied().unwrap_or(f64::NAN));
            mid.push(midness(s.placement.x as f64, s.placement.y as f64, &on.settlements, cell_km));
        }
        let mean = |v: &[f64]| if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 };
        let all_mid = mean(&js.iter().map(|j| midness(j.x, j.y, &base_places, cell_km)).collect::<Vec<_>>());
        // Spacing violations: any founded hamlet closer than the village spacing to another settlement.
        let sp = (cartalith_civ::VILLAGE_SPACING_KM / cell_km).round().max(4.0);
        let viol = founded.iter().filter(|s| on.settlements.iter().any(|t| !std::ptr::eq(**s, t) && ((t.placement.x as f64 - s.placement.x as f64).powi(2) + (t.placement.y as f64 - s.placement.y as f64).powi(2)) < sp * sp)).count();
        println!(
            "JUNC {seed}@{gw} junctions={} near_river={} ({:.0}%) | founded={} near_river={} ({:.0}%) | nearest_larger_km mean={:.1} min={:.1} | midness founded={:.2} all_junctions={:.2} | spacing_violations={viol} moved={moved} | settlements off={} on={}",
            js.len(), j_river, 100.0 * j_river as f64 / js.len().max(1) as f64, founded.len(), f_river, 100.0 * f_river as f64 / founded.len().max(1) as f64,
            mean(&d1), d1.iter().copied().fold(f64::INFINITY, f64::min), mean(&mid), all_mid, off.settlements.len(), on.settlements.len()
        );
        if let Ok(dir) = std::env::var("JUNCTION_DUMP") {
            let _ = std::fs::create_dir_all(&dir);
            let j = serde_json::json!({
                "gw": gw, "gh": gh, "km": p.map_width_km,
                "settlements": off.settlements.iter().map(|s| [s.placement.x as f64, s.placement.y as f64, 0.0]).collect::<Vec<_>>(),
                "founded": founded.iter().map(|s| [s.placement.x as f64, s.placement.y as f64, 1.0]).collect::<Vec<_>>(),
                "junctions": js.iter().map(|j| [j.x, j.y]).collect::<Vec<_>>(),
                "ways": off.ways.iter().filter(|w| !w.hidden).map(|w| w.pts.clone()).collect::<Vec<_>>(),
                "wb": off.water_bodies,
                "river": (0..gw * gh).filter(|&i| ws.flow_discharge[i] as f64 >= thresh).collect::<Vec<_>>(),
            });
            std::fs::write(format!("{dir}/{seed}_{gw}.json"), j.to_string()).unwrap();
        }
    }
}
