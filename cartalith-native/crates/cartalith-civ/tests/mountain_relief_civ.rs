//! **Ruling CU item 1** -- the regression check the owner asked for on the
//! orogen relief term (`TectonicParams::orogen_relief`): rivers, settlement
//! siting, landmarks and lakes all derive from height, so each is measured on
//! the app's own worlds with the switch off (the app before the ruling) and on
//! (the app after), at 2048 x 1311 on `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md`'s
//! worlds. Recorded in that document's §7.
//!
//! Per world and arm it prints one `CIV` line:
//! - **rivers** -- RV-1's three B8 guards (`gf6_carve.rs`' walk, copied): ocean
//!   cells on river paths (bar: 0), lake share of path cells, and 1-3-cell
//!   lakes; plus the lake count;
//! - **settlements** -- the app's default auto-populate chain (as
//!   `ef9_refinement_gain.rs`' `build_world` runs it): how many, their median
//!   elevation, and (on the `on` arm) the share sited within 3 cells of an
//!   `off` settlement and the median distance to the nearest one;
//! - **landmarks** -- the full default pass, placed count per kind.
//!
//! `cargo test -j 2 -p cartalith-civ --test mountain_relief_civ -- --ignored --nocapture`
//! (`MRC_WORLDS` filters by tag prefix.)
//!
//! Protects: nothing by itself (a measurement, `#[ignore]`d); it asserts only
//! that each world has river cells to walk and settlements to compare.
#![allow(dead_code)]

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, LandmarkInputs, LandmarkSettings};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};

const EARTH: [f64; 5] = [0.30, 0.50, 0.60, 0.60, 0.20];
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];
const SUPERCONTINENT: [f64; 5] = [0.60, 0.10, 0.50, 0.70, 0.10];

/// The investigation's nine worlds: (tag, seed, archetype, width km).
const WORLDS: [(&str, i32, Option<[f64; 5]>, f64); 9] = [
    ("s314159_earth_800", 314159, Some(EARTH), 800.0),
    ("s24601_archipelago_800", 24601, Some(ARCHIPELAGO), 800.0),
    ("s24601_earth_3000", 24601, Some(EARTH), 3000.0),
    ("s483920_classic_800", 483920, None, 800.0),
    ("s71077345_classic_800", 71077345, None, 800.0),
    ("s246371_earth_800", 246371, Some(EARTH), 800.0),
    ("s24601_classic_800", 24601, None, 800.0),
    ("s314159_classic_3000", 314159, None, 3000.0),
    ("s483920_supercontinent_3000", 483920, Some(SUPERCONTINENT), 3000.0),
];

fn world_params(seed: i32, arche: Option<[f64; 5]>, km: f64, on: bool) -> WorldParams {
    let mut p = params::defaults();
    p.gw = 2048;
    p.gh = 1311;
    p.tect.seed = seed;
    p.map_width_km = km;
    p.sea_level = 0.42;
    p.tect.orogen_relief = on;
    if let Some(k) = arche {
        p.world_structure = cartalith_engine::WorldStructureParams {
            enabled: true,
            continentality: k[0],
            fragmentation: k[1],
            tectonic_energy: k[2],
            ocean_depth: k[3],
            hotspot_density: k[4],
        };
    }
    p
}

/// RV-1's three B8 guards (`gf6_carve.rs`' `b8`, copied):
/// `(ocean cells on river paths, % of path cells that are lake, 1-3-cell lakes, path cells, lakes)`.
fn b8(ws: &WorldState, class: &[u8], gw: usize, gh: usize, km: f64) -> (usize, f64, usize, usize, usize) {
    let (Some(order), Some(ch)) = (ws.stream_order.as_ref(), ws.channels.as_ref()) else {
        return (0, f64::NAN, 0, 0, 0);
    };
    let rivers = cartalith_hydrology::river_entities(
        order,
        &ch.recv,
        &ws.flow_discharge,
        &ws.field,
        gw,
        gh,
        1,
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, km),
        cartalith_hydrology::river_width_scale_k(km),
        false,
    );
    let (mut cells, mut ocean, mut lake) = (0usize, 0usize, 0usize);
    for r in &rivers {
        for &(x, y) in r.pts.iter().take(r.pts.len().saturating_sub(1)) {
            let i = y as usize * gw + x as usize;
            cells += 1;
            match class[i] {
                1 => ocean += 1,
                2 => lake += 1,
                _ => {}
            }
        }
    }
    let mut seen = vec![false; gw * gh];
    let (mut small, mut lakes, mut stack) = (0usize, 0usize, Vec::new());
    for s0 in 0..gw * gh {
        if class[s0] != 2 || seen[s0] {
            continue;
        }
        seen[s0] = true;
        stack.push(s0);
        let mut k = 0usize;
        while let Some(i) = stack.pop() {
            k += 1;
            let (x, y) = (i % gw, i / gw);
            for j in [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)]
                .into_iter()
                .flatten()
            {
                if class[j] == 2 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        lakes += 1;
        if k <= 3 {
            small += 1;
        }
    }
    (ocean, 100.0 * lake as f64 / cells.max(1) as f64, small, cells, lakes)
}

/// The app's default auto-populate chain to placed settlements
/// (`ef9_refinement_gain.rs`' `build_world`, copied up to placement).
fn settlements(ws: &WorldState, p: &WorldParams, wb: &cartalith_civ::WaterBodies) -> Vec<(usize, usize)> {
    let (gw, gh, world, sea, km) = (p.gw, p.gh, p.world, ws.sea_level, p.map_width_km);
    let opts = &p.civ;
    let want = opts.want_counts();
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea);
    let soil = cartalith_civ::build_soil_fertility(&lith, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
    let thresh_flow = cartalith_hydrology::river_flow_thresh(gw, gh, gw, km);
    let water = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, thresh_flow);
    let cap = cartalith_civ::build_carrying_capacity(&soil, &water, Some(&biome), &ws.temperature, &ws.field, sea, 0.0, None);
    let resources = cartalith_civ::build_resource_potentials(
        &lith, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome), &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea,
        Some(&ws.volcanic_field), true, false,
    );
    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, thresh_flow);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&cap), gw, gh, sea, world);
    let coast_reach = cartalith_civ::build_coast_reach(&cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea), &wb.classification, gw, gh);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
    let (river_order, river_polys) =
        cartalith_civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, km, ws.integrated_drainage);
    let river_reach = cartalith_civ::build_river_reach(&river_polys, &river_order, gw, gh);
    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_reach: Some(&river_reach),
        coast_reach: Some(&coast_reach),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh: thresh_flow,
    };
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water, &cap, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx));
    let (thresh, supp_r) = match want {
        Some(w) => cartalith_civ::civ_want_counts_seed_params(gw, w.iter().sum()),
        None => (opts.seed_thresh, (gw as f64 / opts.seed_suppress_div.max(1.0)).floor().max(6.0)),
    };
    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, thresh, supp_r);
    let placements = cartalith_civ::place_settlements_with_counts(
        &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea, world, opts.factions.max(1), &flood, &ws.flow_discharge, thresh_flow, km, want,
    );
    placements.iter().map(|s| (s.x, s.y)).collect()
}

/// Placed landmark count per kind, the full default pass (as
/// `gf6_identity_hashes.rs` runs it).
fn landmarks(ws: &WorldState, p: &WorldParams, wb: &cartalith_civ::WaterBodies) -> std::collections::BTreeMap<String, usize> {
    let mut inp = LandmarkInputs::new(&ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km);
    inp.peak_m = p.peak_m;
    inp.flow = Some(&ws.flow_discharge);
    inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
    inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
    inp.order = ws.stream_order.as_deref();
    inp.water = Some(&wb.classification);
    inp.volcanism = Some(&ws.volcanic_field);
    let rock = landmark::RockView::for_world(ws, p).ok();
    inp.rock = rock.as_ref();
    let r = landmark::generate(&inp, &LandmarkSettings::default(), p.tect.seed as u64);
    r.funnels.iter().filter(|f| f.placed > 0).map(|f| (f.kind.to_string(), f.placed)).collect()
}

#[test]
#[ignore = "generates eighteen 2048x1311 worlds and runs the civ chain; run explicitly"]
fn mountain_relief_civ_regression() {
    let only = std::env::var("MRC_WORLDS").ok();
    for &(tag, seed, arche, km) in &WORLDS {
        if let Some(o) = &only
            && !o.split(',').any(|t| tag.starts_with(t))
        {
            continue;
        }
        let mut prev: Option<Vec<(usize, usize)>> = None;
        let mut prev_lm = None;
        for on in [false, true] {
            let p = world_params(seed, arche, km, on);
            let ws = generate_terrain(&p);
            let (gw, gh) = (p.gw, p.gh);
            let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, p.world, Some(&ws.rainfall));
            let (ocean, lake_pct, small, cells, lakes) = b8(&ws, &wb.classification, gw, gh, km);
            // `MRC_DUMP=<dir>`: the water classification, for locating lakes.
            if let Ok(d) = std::env::var("MRC_DUMP") {
                std::fs::write(format!("{d}/{tag}_{}_class.bin", if on { "on" } else { "off" }), &wb.classification).unwrap();
            }
            assert!(cells > 0, "{tag}: B8 walked no river cells");
            let st = settlements(&ws, &p, &wb);
            assert!(!st.is_empty(), "{tag}: no settlements");
            let denom = 1.0 - ws.sea_level;
            let mut elev: Vec<f64> = st.iter().map(|&(x, y)| (ws.field[y * gw + x] as f64 - ws.sea_level) / denom * p.peak_m).collect();
            elev.sort_by(|a, b| a.total_cmp(b));
            let med_elev = elev[elev.len() / 2];
            let moved = match &prev {
                Some(off) => {
                    let mut d: Vec<f64> = st
                        .iter()
                        .map(|&(x, y)| off.iter().map(|&(a, b)| (x as f64 - a as f64).hypot(y as f64 - b as f64)).fold(f64::INFINITY, f64::min))
                        .collect();
                    let near = d.iter().filter(|&&v| v <= 3.0).count() as f64 / d.len() as f64;
                    d.sort_by(|a, b| a.total_cmp(b));
                    format!(" | within3 {:.3} median_nn {:.1}", near, d[d.len() / 2])
                }
                None => String::new(),
            };
            let lm = landmarks(&ws, &p, &wb);
            let lm_line = match &prev_lm {
                Some(off) => {
                    let off: &std::collections::BTreeMap<String, usize> = off;
                    let keys: std::collections::BTreeSet<&String> = off.keys().chain(lm.keys()).collect();
                    keys.iter()
                        .filter(|k| off.get(**k) != lm.get(**k))
                        .map(|k| format!("{}:{}->{}", k, off.get(*k).copied().unwrap_or(0), lm.get(*k).copied().unwrap_or(0)))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                None => lm.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "),
            };
            println!(
                "CIV {tag} {}: B8 ocean {ocean} lake% {lake_pct:.3} small {small} (path {cells}) lakes {lakes} | settlements {} median_elev {:.0}{moved} | landmarks {} total | {}",
                if on { "on " } else { "off" },
                st.len(),
                med_elev,
                lm.values().sum::<usize>(),
                lm_line
            );
            if !on {
                prev = Some(st);
                prev_lm = Some(lm);
            }
        }
    }
}
