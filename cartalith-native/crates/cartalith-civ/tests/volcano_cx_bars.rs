//! **Ruling CX's regression bars** (owner, 2026-10-24): every Ruling CX item
//! that moves the height field (item 1, `volc.morphometry` on in the app;
//! item 4, a grid-aware cinder law; item 6, dry active craters) is measured
//! here on the five VA-0 census worlds, each arm against the first one named.
//!
//! Per world and arm it prints one `CX` line:
//! - **generation** -- seconds for `generate_terrain` (release; run alone);
//!   the field's FNV digest;
//! - **rivers** -- RV-1's three B8 guards (`mountain_relief_civ.rs`' walk,
//!   copied): ocean cells on river paths (bar: 0), lake share of path cells,
//!   1-3-cell lakes (bar: within 1.25x of the first arm), and the lake count;
//! - **watersheds** -- outlet agreement with the first arm on land
//!   (`terrain_streak_spike.rs`' measure, copied): exact outlet cell, and the
//!   outlet within 5 cells (a mouth moving one cell along a coast changes the
//!   exact outlet of its whole basin without moving the basin);
//! - **settlements** -- the app's default auto-populate chain: how many, and
//!   (against the first arm) the share within 3 cells of one of its
//!   settlements;
//! - **landmarks** -- the full default pass, total placed and the kinds whose
//!   count changed;
//! - **craters** -- per activity state, the lake cells inside the summit zone
//!   of the edifices in that state (`crater_zone_t`), so item 6's bar ("an
//!   active crater holds no water cell") is read here, and the volcano
//!   table's activity counts.
//!
//! The water classification is `build_water_bodies` as the earlier bars
//! (`mountain_relief_civ.rs`, Ruling CO's census) read it, without the
//! app's Ruling CC area floor, so the numbers compare with those.
//!
//! `CX_ARMS=app,no_morph cargo test --release -j 2 -p cartalith-civ --test
//! volcano_cx_bars -- --ignored --nocapture` (`CX_WORLDS` filters by tag
//! prefix). Arm names are [`arm`]'s match arms; an unknown name panics.
//!
//! Protects: nothing by itself (a measurement, `#[ignore]`d); it asserts only
//! that each world has river cells to walk, settlements and a volcano table.
#![allow(dead_code)]

#[path = "../../cartalith-godot/src/params.rs"]
mod params;

use cartalith_civ::landmark::{self, LandmarkInputs, LandmarkSettings};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_terrain::volcano_attr;

const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];

/// The VA-0 census worlds (`cartalith-godot/tests/volcano_appearance.rs`'
/// `WORLDS`): (tag, seed, archetype).
const WORLDS: [(&str, i32, Option<[f64; 5]>); 5] = [
    ("s71077345_classic", 71077345, None),
    ("s24601_archipelago", 24601, Some(ARCHIPELAGO)),
    ("s314159_classic", 314159, None),
    ("s483920_classic", 483920, None),
    ("s246371_classic", 246371, None),
];

/// The app's parameters for one census world (2048 x 1311, 800 km, sea 0.42,
/// as `volcano_appearance.rs` generates them), then the arm's change.
fn world_params(seed: i32, arche: Option<[f64; 5]>, arm_name: &str) -> WorldParams {
    let mut p = params::defaults();
    p.gw = 2048;
    p.gh = 1311;
    p.tect.seed = seed;
    p.map_width_km = 800.0;
    p.sea_level = 0.42;
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
    arm(&mut p, arm_name);
    p
}

/// One arm: the app's defaults (`app`) or the app with one Ruling CX switch
/// turned back. Panics on a name it does not know, so a typo cannot measure
/// the app twice under two names.
fn arm(p: &mut WorldParams, name: &str) {
    match name {
        "app" => {}
        // The app before Ruling CX item 1.
        "no_morph" => p.volc.morphometry = false,
        // The app before Ruling CX item 4 (the grid-aware cinder law). The
        // trial's two floors (2.5 and 2.0 cells) are recorded in `STATUS.md`.
        "no_cinder_law" => p.volc.cinder_grid_law = false,
        // The app before Ruling CX item 6 (active craters may pool).
        "wet_active" => p.volc.dry_active_craters = false,
        // The app before Ruling DB item 3 (the top back at 1).
        "old_ceiling" => p.tect.high_ceiling = false,
        // The app before Ruling DB item 2 (no lava benches).
        "no_bench" => p.volc.lava_bench = false,
        other => panic!("unknown arm {other:?}"),
    }
}

/// RV-1's three B8 guards (`mountain_relief_civ.rs`' `b8`, copied):
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

/// Every land cell's drainage outlet on the routing surface the app routes
/// over (`terrain_streak_spike.rs`' `outlets`, copied): follow the D8
/// receivers until a cell with none or a sea cell. `usize::MAX` for a sea
/// cell itself.
fn outlets(ws: &WorldState, gw: usize, gh: usize, world: bool) -> Vec<usize> {
    let sea = ws.sea_level;
    let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, sea, world);
    let recv = cartalith_hydrology::flow_receivers(&route, gw, gh, world);
    let n = gw * gh;
    let mut out = vec![usize::MAX - 1; n];
    let mut path = Vec::new();
    for s in 0..n {
        if (ws.field[s] as f64) < sea {
            out[s] = usize::MAX;
            continue;
        }
        if out[s] != usize::MAX - 1 {
            continue;
        }
        path.clear();
        let mut c = s;
        let end = loop {
            if out[c] != usize::MAX - 1 {
                break if out[c] == usize::MAX { c } else { out[c] };
            }
            path.push(c);
            let r = recv[c];
            if r < 0 {
                break c;
            }
            let r = r as usize;
            if (ws.field[r] as f64) < sea {
                break r;
            }
            c = r;
        };
        for &q in &path {
            out[q] = end;
        }
    }
    out
}

/// The app's default auto-populate chain to placed settlements
/// (`mountain_relief_civ.rs`' `settlements`, copied).
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

/// Placed landmark count per kind, the full default pass
/// (`mountain_relief_civ.rs`' `landmarks`, with the volcano table attached
/// as the shell attaches it).
fn landmarks(ws: &WorldState, p: &WorldParams, wb: &cartalith_civ::WaterBodies) -> std::collections::BTreeMap<String, usize> {
    let mut inp = LandmarkInputs::new(&ws.field, p.gw, p.gh, ws.sea_level, p.world, p.map_width_km);
    inp.peak_m = p.peak_m;
    inp.flow = Some(&ws.flow_discharge);
    inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
    inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
    inp.order = ws.stream_order.as_deref();
    inp.water = Some(&wb.classification);
    inp.volcanism = Some(&ws.volcanic_field);
    inp.volcanoes = ws.volcanoes.as_deref().map(|v| v.as_slice());
    let rock = landmark::RockView::for_world(ws, p).ok();
    inp.rock = rock.as_ref();
    let r = landmark::generate(&inp, &LandmarkSettings::default(), p.tect.seed as u64);
    r.funnels.iter().filter(|f| f.placed > 0).map(|f| (f.kind.to_string(), f.placed)).collect()
}

/// The summit zone of an edifice as a fraction of its radius: the region the
/// Ruling CO pit fill leaves alone (`edifice_pits::SUMMIT_T`) or the crater
/// rim with a quarter's margin, whichever is wider. A measurement region, the
/// same one item 6's dry-crater step and item 5's crater-lake colour use.
fn crater_zone_t(e: &cartalith_terrain::PlacedEdifice) -> f64 {
    let rim = volcano_attr::zones(e).map_or(0.0, |z| z.1);
    (rim * 1.25).max(cartalith_terrain::edifice_pits::SUMMIT_T)
}

/// Lake cells (`class == 2`) inside the summit zone of the edifices in each
/// activity state (active, dormant, extinct), each cell counted for the
/// edifice owning it by the stamp's rule (`volcano_attr::owner_of`); and the
/// count of edifices per state.
fn crater_lakes(vt: &[cartalith_terrain::PlacedEdifice], class: &[u8], gw: usize, gh: usize) -> ([usize; 3], [usize; 3]) {
    let (mut cells, mut count) = ([0usize; 3], [0usize; 3]);
    for e in vt {
        count[volcano_attr::activity(e) as usize] += 1;
    }
    let idx = volcano_attr::VolcanoIndex::new(vt, gw, gh);
    for i in 0..gw * gh {
        if class[i] != 2 {
            continue;
        }
        let Some(h) = idx.winner_at((i % gw) as f64, (i / gw) as f64) else { continue };
        if h.t < crater_zone_t(h.edifice) {
            cells[volcano_attr::activity(h.edifice) as usize] += 1;
        }
    }
    (cells, count)
}

/// The 1-3-cell lakes (4-connected, as [`b8`] counts them) by where their
/// first cell lies: `(in an edifice's summit zone, elsewhere on an edifice
/// disc, off every edifice)` -- so a change in the small-lake count can be
/// attributed to craters, flanks or the rest of the world.
fn small_lakes_by_place(vt: &[cartalith_terrain::PlacedEdifice], class: &[u8], gw: usize, gh: usize) -> (usize, usize, usize) {
    let idx = volcano_attr::VolcanoIndex::new(vt, gw, gh);
    let mut seen = vec![false; gw * gh];
    let (mut summit, mut flank, mut off, mut stack) = (0usize, 0usize, 0usize, Vec::new());
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
        if k > 3 {
            continue;
        }
        match idx.winner_at((s0 % gw) as f64, (s0 / gw) as f64) {
            Some(h) if h.t < crater_zone_t(h.edifice) => summit += 1,
            Some(_) => flank += 1,
            None => off += 1,
        }
    }
    (summit, flank, off)
}

fn fnv(v: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in v {
        for b in x.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}

#[test]
#[ignore = "generates ten 2048x1311 worlds and runs the civ chain; run explicitly with --release"]
fn volcano_cx_bars() {
    let arms_s = std::env::var("CX_ARMS").unwrap_or_else(|_| "no_morph,app".to_string());
    let arms: Vec<&str> = arms_s.split(',').collect();
    let only = std::env::var("CX_WORLDS").ok();
    for &(tag, seed, arche) in &WORLDS {
        if let Some(o) = &only
            && !o.split(',').any(|t| tag.starts_with(t))
        {
            continue;
        }
        let mut first: Option<(Vec<(usize, usize)>, std::collections::BTreeMap<String, usize>, Vec<usize>, usize)> = None;
        for &a in &arms {
            let p = world_params(seed, arche, a);
            let t0 = std::time::Instant::now();
            let ws = generate_terrain(&p);
            let gen_s = t0.elapsed().as_secs_f64();
            let (gw, gh) = (p.gw, p.gh);
            let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, p.world, Some(&ws.rainfall));
            let (ocean, lake_pct, small, cells, lakes) = b8(&ws, &wb.classification, gw, gh, p.map_width_km);
            assert!(cells > 0, "{tag} {a}: B8 walked no river cells");
            let st = settlements(&ws, &p, &wb);
            assert!(!st.is_empty(), "{tag} {a}: no settlements");
            let lm = landmarks(&ws, &p, &wb);
            let oc = outlets(&ws, gw, gh, p.world);
            let vt = ws.volcanoes.as_deref().expect("the app records a volcano table");
            let (cl, cnt) = crater_lakes(vt, &wb.classification, gw, gh);
            let (ss, sf, so) = small_lakes_by_place(vt, &wb.classification, gw, gh);
            let cmp = match &first {
                None => String::new(),
                Some((st0, lm0, oc0, small0)) => {
                    let near = st.iter().filter(|&&(x, y)| st0.iter().any(|&(a, b)| (x as f64 - a as f64).hypot(y as f64 - b as f64) <= 3.0)).count() as f64 / st.len() as f64;
                    let (mut same, mut nr, mut tot) = (0usize, 0usize, 0usize);
                    for i in 0..gw * gh {
                        if oc0[i] != usize::MAX && oc[i] != usize::MAX {
                            tot += 1;
                            same += usize::from(oc0[i] == oc[i]);
                            let (ax, ay, bx, by) = ((oc0[i] % gw) as i64, (oc0[i] / gw) as i64, (oc[i] % gw) as i64, (oc[i] / gw) as i64);
                            nr += usize::from((ax - bx).abs() <= 5 && (ay - by).abs() <= 5);
                        }
                    }
                    let keys: std::collections::BTreeSet<&String> = lm0.keys().chain(lm.keys()).collect();
                    let lmd = keys
                        .iter()
                        .filter(|k| lm0.get(**k) != lm.get(**k))
                        .map(|k| format!("{}:{}->{}", k, lm0.get(*k).copied().unwrap_or(0), lm.get(*k).copied().unwrap_or(0)))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!(
                        " | vs first: small x{:.2} outlets exact {:.2}% within5 {:.2}% settlements within3 {:.3} landmarks [{}]",
                        small as f64 / (*small0).max(1) as f64,
                        100.0 * same as f64 / tot.max(1) as f64,
                        100.0 * nr as f64 / tot.max(1) as f64,
                        near,
                        lmd
                    )
                }
            };
            println!(
                "CX {tag} {a:<10} gen {gen_s:.2}s fnv {:016x} | B8 ocean {ocean} lake% {lake_pct:.3} small {small} (path {cells}) lakes {lakes} (small summit/flank/off {ss}/{sf}/{so}) | settlements {} | landmarks {} | volcanoes act/dorm/ext {}/{}/{} crater-lake cells {}/{}/{}{cmp}",
                fnv(&ws.field),
                st.len(),
                lm.values().sum::<usize>(),
                cnt[0],
                cnt[1],
                cnt[2],
                cl[0],
                cl[1],
                cl[2]
            );
            if first.is_none() {
                first = Some((st, lm, oc, small));
            }
        }
    }
}
