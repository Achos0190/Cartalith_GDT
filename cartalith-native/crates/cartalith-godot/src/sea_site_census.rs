//! The census for owner findings F16, F39 and F36 (2026-11-21): sea lanes that
//! bundle, skip ports and run far off the coast, and settlement siting that
//! ignores river confluences and the coast. Measurement only; every test here is
//! `#[ignore]`d (it generates 2 048-cell worlds). Run alone, release:
//! `cargo test --release -p cartalith-godot --lib sea_site_census -- --ignored --nocapture --test-threads 1`.
//! `SEA_CENSUS_DUMP=<dir>` also writes each world's lane polylines, settlements
//! and coast as JSON for the frame script
//! (`docs/evidence/sea-routes-siting-2026-11-22/frames.py`).
//!
//! **Sea rows** (`sea_lane_census`), one world, `civ.sea_lanes` off then on, siting
//! terms held off in both so the ports are identical: lanes, total km, bundle
//! stretches (distinct strokes side by side at the opening view, and the
//! convergent ones; `shared` counts stretches where lanes touch, one trunk), ports
//! with no lane, **ports skipped** (a port a lane passes within
//! `sea_lanes::CALL_RADIUS_CELLS` routing cells of without ending there, and
//! `skipped5km`: the same within 5 km), and the
//! **mean distance off the coast**, length-weighted along every lane, and the
//! **minimum standoff** (Ruling GR): the least distance to land at any sample
//! of a lane farther than the standoff band plus the call radius from every port.
//!
//! **Site rows** (`site_census`), `civ.site_map_terms` off then on, lanes held
//! off: settlements, how many sit within the confluence reach of a confluence
//! of order 3 or more and how many of order 2 or more, against how many such
//! confluences the map has, and how many sit within 4 km of the ocean
//! (`PORT_REACH_KM`) against the harbour badge.

use crate::way_census::fit_px_per_cell;
use cartalith_civ::way_audit::{self, AuditView, AuditWay};

/// A world to measure: the app's defaults at this size.
#[derive(Clone, Copy)]
struct World {
    tag: &'static str,
    seed: i32,
    gw: usize,
    gh: usize,
    km: f64,
}

const WORLDS: [World; 8] = [
    World { tag: "483920@384", seed: 483_920, gw: 384, gh: 288, km: 800.0 },
    World { tag: "912345@384", seed: 912_345, gw: 384, gh: 288, km: 800.0 },
    World { tag: "246371@384", seed: 246_371, gw: 384, gh: 288, km: 800.0 },
    World { tag: "483920@2048", seed: 483_920, gw: 2048, gh: 1311, km: 800.0 },
    World { tag: "912345@2048", seed: 912_345, gw: 2048, gh: 1311, km: 800.0 },
    World { tag: "246371@2048", seed: 246_371, gw: 2048, gh: 1311, km: 800.0 },
    // The whole-planet world the road census uses (many ports): the stress case for lane bundles.
    World { tag: "246371@planet", seed: 246_371, gw: 2048, gh: 1311, km: 40_075.0 },
    World { tag: "owner83744", seed: 83_744, gw: 2048, gh: 1024, km: 200.0 },
];

/// The owner's generation parameters (`OWNER_FINDINGS_2026-11-21.md`, seed
/// 83744), as `key: value` lines, applied over the app's defaults through
/// `params::set`. Keys this build has no row for are skipped and counted.
const OWNER_PARAMS: &str = "basin_routing: true
carve_rivers: true
civ.base_ag_tech: 5
civ.biome_k: false
civ.factions: 2
civ.fixed_counts: false
civ.founding_year: true
civ.garrison_exposure_scale: 1.0
civ.harbour_box: true
civ.map_crossings: true
civ.map_rivers: true
civ.merge_ways: true
civ.metropolis: true
civ.n_capital: 0
civ.n_city: 0
civ.n_hamlet: 0
civ.n_town: 0
civ.n_village: 0
civ.port_reach: true
civ.port_shore: true
civ.recovery_phase: 3
civ.river_front: true
civ.seed_suppress_div: 22.0
civ.seed_thresh: 0.51
civ.villages: true
climate.albedo_k: 0.0
climate.bulk_evap: true
climate.current_k: 1.0
climate.currents: true
climate.equator_temp: 32.0
climate.evap: 0.2
climate.lapse_rate: 5.0
climate.lat_n: 55.0
climate.lat_s: 5.0
climate.ocean: 1.5
climate.ocean_hum: 1.8
climate.pole_temp: -27.0
climate.press_k: 0.6
climate.rain_bspline: true
climate.rain_dep: 0.38
climate.rain_k: 1.5
climate.terrain_wind_deflection: true
climate.w_iters: 90
climate.wind_dir_deg: 0.0
climate.wind_manual: false
climate.zonal_k: 0.5
crater.age: 0.5
crater.count: 40
crater.physical_model: true
crater.surface_age_myr: 100.0
geo.age: 1.0
geology_carve: false
geology_karst: false
geology_model: true
geology_processes: false
integrate_drainage: true
lake_balance: true
passes.coastal: false
passes.coastal_passes: 4
passes.diffuse_d: 0.15
passes.diffuse_passes: 6
passes.estuary_depth: 0.08
passes.evolve_cycles: 0
passes.glacial: true
passes.glacial_kg: 0.15
passes.glacial_mg: 0.4
passes.glacial_passes: 9
passes.glacial_snowline: 0.65
passes.glacial_u_factor: 0.6
passes.hillslope: false
passes.marsh_band: 0.03
passes.sediment_capacity: 6.0
passes.sediment_fill: false
passes.tidal_flats: false
passes.tidal_k: 0.45
passes.velo_iters: 60
passes.velo_meander: 0.6
passes.velo_strength: 0.51
passes.velocity: false
passes.wave_str: 0.5
peak_m: 4000.0
planet.axial_tilt_deg: 23.4
planet.g: 1.0
planet.rotation_hours: 24.0
relief_model: false
river_density: 2.5
rough_routing: true
sea_level: 0.42
stream.climate_k: 0.5
stream.deposit: 0.3
stream.iters: 15
stream.k: 0.012
stream.uplift: 0.0
tect.age_inf: 0.6
tect.alpha: 1.1
tect.beta: 0.58
tect.blur_r: 18.0
tect.dynamic_lithology: true
tect.flexure: 0.32
tect.hetero: 0.14
tect.high_ceiling: true
tect.lloyd: 2
tect.narrow_plate_base_blur: true
tect.orogen_relief: true
tect.plates: 4
tect.resist: 0.7
tect.ridged: true
tect.vel: 1.8
tect.warp: 0.95
urban_burgage: false
urban_harbour_front: true
urban_market_scale: true
urban_river_kind_map: true
urban_river_stretch: true
urban_riverbank: true
urban_street_village: true
urban_wall_hug: true
use_gpu: false
volc.age: 0.4
volc.cinder_grid_law: true
volc.count: 13
volc.dry_active_craters: true
volc.edifice_model: true
volc.exclude_transform: true
volc.fill_edifice_pits: true
volc.kind_caldera: true
volc.kind_dome: true
volc.kind_flood_basalt: true
volc.kind_maar: true
volc.kind_supervolcano: true
volc.lava_bench: true
volc.morphometry: true
volc.provinces: true
world: false
world_structure.continentality: 0.3
world_structure.enabled: false
world_structure.fragmentation: 0.5
world_structure.hotspot_density: 0.2
world_structure.ocean_depth: 0.6
world_structure.tectonic_energy: 0.6";

/// The world's parameters: the app's defaults at the world's size, or for the
/// owner's world the owner's list on top.
fn params_of(w: &World) -> cartalith_engine::WorldParams {
    let mut p = crate::params::defaults();
    p.gw = w.gw;
    p.gh = w.gh;
    p.tect.seed = w.seed as _;
    p.map_width_km = w.km;
    p.use_gpu = false;
    p.civ.villages = true;
    p.world = w.km >= 40_075.0;
    if w.tag == "owner83744" {
        let mut skipped = 0;
        for line in OWNER_PARAMS.lines() {
            let (k, v) = line.split_once(": ").expect("key: value");
            let val = match v {
                "true" => crate::params::Value::Bool(true),
                "false" => crate::params::Value::Bool(false),
                n => crate::params::Value::Num(n.parse().expect("number")),
            };
            if crate::params::spec(k).is_none() {
                skipped += 1;
                continue;
            }
            crate::params::set(&mut p, k, val);
        }
        println!("owner params: {skipped} keys without a row skipped");
        p.gw = w.gw;
        p.gh = w.gh;
        p.map_width_km = w.km;
        p.tect.seed = 83_744;
    }
    p
}

/// Distance in cells to the nearest cell of the ocean class's complement
/// (land or lake), a two-pass chamfer transform (1, sqrt 2), per cell.
fn land_distance(wb: &[u8], gw: usize, gh: usize) -> Vec<f32> {
    const D: f32 = std::f32::consts::SQRT_2;
    let mut d: Vec<f32> = wb.iter().map(|&c| if c == 1 { f32::INFINITY } else { 0.0 }).collect();
    let at = |d: &Vec<f32>, x: isize, y: isize| -> f32 {
        if x < 0 || y < 0 || x >= gw as isize || y >= gh as isize { f32::INFINITY } else { d[y as usize * gw + x as usize] }
    };
    for y in 0..gh as isize {
        for x in 0..gw as isize {
            let i = y as usize * gw + x as usize;
            let v = d[i].min(at(&d, x - 1, y) + 1.0).min(at(&d, x, y - 1) + 1.0).min(at(&d, x - 1, y - 1) + D).min(at(&d, x + 1, y - 1) + D);
            d[i] = v;
        }
    }
    for y in (0..gh as isize).rev() {
        for x in (0..gw as isize).rev() {
            let i = y as usize * gw + x as usize;
            let v = d[i].min(at(&d, x + 1, y) + 1.0).min(at(&d, x, y + 1) + 1.0).min(at(&d, x + 1, y + 1) + D).min(at(&d, x - 1, y + 1) + D);
            d[i] = v;
        }
    }
    d
}

fn seg_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let l2 = vx * vx + vy * vy;
    let t = if l2 > 0.0 { (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p.0 - a.0 - t * vx).hypot(p.1 - a.1 - t * vy)).abs()
}

/// One lane set's figures.
#[derive(Default, Debug)]
struct SeaRow {
    lanes: usize,
    km: f64,
    bundles: usize,
    bundles_conv: usize,
    shared: usize,
    ports: usize,
    unlaned: usize,
    skipped: usize,
    skipped5km: usize,
    off_coast_km: f64,
    min_standoff_km: f64,
}

fn sea_row(c: &crate::CivData, p: &cartalith_engine::WorldParams) -> SeaRow {
    let (gw, gh) = (p.gw, p.gh);
    let km_cell = p.map_width_km / gw as f64;
    let ports: Vec<(f64, f64)> = c.settlements.iter().filter(|s| s.placement.coastal).map(|s| (s.placement.x as f64, s.placement.y as f64)).collect();
    let (_, sc) = cartalith_civ::routing_grid_scale(gw);
    let call_r = cartalith_civ::sea_lanes::CALL_RADIUS_CELLS / sc;
    let mut r = SeaRow { lanes: c.sea_routes.len(), ports: ports.len(), ..Default::default() };
    r.km = c.sea_routes.iter().map(|s| s.km).sum();
    let ends = (usize::MAX, usize::MAX);
    let ways: Vec<AuditWay> = c.sea_routes.iter().map(|s| AuditWay { pts: &s.pts, brks: &s.brks, under_px: 1.5, ends }).collect();
    let view = AuditView { px_per_cell: fit_px_per_cell(gw, gh) };
    let st = way_audit::bundle_stretches(&ways, &ports, view);
    // A bundle is two lanes drawn side by side as distinct strokes; lanes that
    // share a trunk touch (`overlap`) and are counted apart as `shared`.
    r.bundles = st.iter().filter(|s| !s.overlap).count();
    r.bundles_conv = st.iter().filter(|s| !s.overlap && s.convergent).count();
    r.shared = st.iter().filter(|s| s.overlap).count();
    for &pp in &ports {
        let (mut near_any, mut skipped, mut skipped5) = (false, false, false);
        for s in &c.sea_routes {
            let (f, l) = (s.pts[0], *s.pts.last().unwrap());
            let is_end = (f.0 - pp.0).hypot(f.1 - pp.1) < 1.0 || (l.0 - pp.0).hypot(l.1 - pp.1) < 1.0;
            let d = s.pts.windows(2).map(|w| seg_dist(pp, w[0], w[1])).fold(f64::INFINITY, f64::min);
            if is_end {
                near_any = true;
            } else {
                skipped |= d <= call_r;
                skipped5 |= d * km_cell <= 5.0;
            }
        }
        r.unlaned += usize::from(!near_any);
        r.skipped += usize::from(skipped);
        r.skipped5km += usize::from(skipped5);
    }
    let ld = land_distance(&c.water_bodies, gw, gh);
    // Ruling GR: the standoff is judged away from ports, so samples inside the
    // exemption (the standoff band plus the call radius, in routing cells) of any port are skipped.
    let band = cartalith_civ::sea_lanes::standoff_cells(km_cell / sc);
    let exempt = (band as f64 + cartalith_civ::sea_lanes::CALL_RADIUS_CELLS) / sc;
    let at_ld = |x: f64, y: f64| ld[(y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1)] as f64 * km_cell;
    let mut min_standoff = f64::INFINITY;
    for s in &c.sea_routes {
        for w in s.pts.windows(2) {
            if (w[1].0 - w[0].0).abs() > gw as f64 / 2.0 {
                continue;
            }
            for q in [w[0], ((w[0].0 + w[1].0) / 2.0, (w[0].1 + w[1].1) / 2.0), w[1]] {
                if ports.iter().all(|pp| (pp.0 - q.0).hypot(pp.1 - q.1) > exempt) {
                    min_standoff = min_standoff.min(at_ld(q.0, q.1));
                }
            }
        }
    }
    r.min_standoff_km = if min_standoff.is_finite() { min_standoff } else { f64::NAN };
    let (mut sum, mut len) = (0.0, 0.0);
    for s in &c.sea_routes {
        for w in s.pts.windows(2) {
            let l = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            if l > gw as f64 / 2.0 {
                continue; // a seam jump
            }
            let (mx, my) = ((w[0].0 + w[1].0) / 2.0, (w[0].1 + w[1].1) / 2.0);
            let d = at_ld(mx, my);
            sum += d * l;
            len += l;
        }
    }
    r.off_coast_km = if len > 0.0 { sum / len } else { f64::NAN };
    r
}

fn dump(tag: &str, mode: &str, c: &crate::CivData, p: &cartalith_engine::WorldParams, rivers: &[Vec<(f64, f64)>]) {
    let Ok(dir) = std::env::var("SEA_CENSUS_DUMP") else { return };
    let _ = std::fs::create_dir_all(&dir);
    let j = serde_json::json!({
        "gw": p.gw, "gh": p.gh, "km": p.map_width_km,
        "lanes": c.sea_routes.iter().map(|s| s.pts.clone()).collect::<Vec<_>>(),
        "ports": c.settlements.iter().filter(|s| s.placement.coastal).map(|s| [s.placement.x, s.placement.y]).collect::<Vec<_>>(),
        "towns": c.settlements.iter().map(|s| [s.placement.x as f64, s.placement.y as f64, if s.placement.coastal { 1.0 } else { 0.0 }]).collect::<Vec<_>>(),
        "rivers": rivers,
        "wb": c.water_bodies,
    });
    std::fs::write(format!("{dir}/{tag}_{mode}.json"), j.to_string()).unwrap();
}

#[test]
#[ignore = "measurement: generates 2048-cell worlds"]
fn sea_lane_census() {
    let only = std::env::var("SEA_CENSUS_ONLY").ok();
    for w in WORLDS.iter().filter(|w| only.as_deref().is_none_or(|o| w.tag.contains(o))) {
        let mut p = params_of(w);
        p.civ.site_map_terms = false;
        let ws = cartalith_engine::generate_terrain(&p);
        for lanes in [false, true] {
            p.civ.sea_lanes = lanes;
            let t = std::time::Instant::now();
            let c = crate::way_census::civ_for(&ws, &p);
            let ms = t.elapsed().as_millis();
            let r = sea_row(&c, &p);
            println!(
                "SEA {:<12} lanes={} lanes_n={:<3} km={:<8.1} bundles={:<3} conv={:<3} shared={:<3} ports={:<3} unlaned={:<2} skipped={:<2} skipped5km={:<2} off_coast_km={:.2} min_standoff_km={:.2} ({} ms)",
                w.tag, if lanes { "on " } else { "off" }, r.lanes, r.km, r.bundles, r.bundles_conv, r.shared, r.ports, r.unlaned, r.skipped, r.skipped5km, r.off_coast_km, r.min_standoff_km, ms
            );
            dump(w.tag, if lanes { "lanes_on" } else { "lanes_off" }, &c, &p, &[]);
        }
    }
}

#[test]
#[ignore = "measurement: generates 2048-cell worlds"]
fn site_census() {
    let only = std::env::var("SEA_CENSUS_ONLY").ok();
    for w in WORLDS.iter().filter(|w| only.as_deref().is_none_or(|o| w.tag.contains(o))) {
        let mut p = params_of(w);
        p.civ.sea_lanes = false;
        let ws = cartalith_engine::generate_terrain(&p);
        let (gw, gh) = (p.gw, p.gh);
        // The confluences the map has.
        let net = crate::map_river_network(&ws, gw, gh).expect("the map has a river network");
        let conf = cartalith_civ::build_river_confluence(&net.0, net.1, gw, gh);
        let cell_km = p.map_width_km / gw as f64;
        for terms in [false, true] {
            p.civ.site_map_terms = terms;
            let c = crate::way_census::civ_for(&ws, &p);
            let reach_cells = (cartalith_civ::PORT_REACH_KM / cell_km).max(1.0) as f32;
            let (mut n3, mut n2, mut coast, mut badge) = (0, 0, 0, 0);
            for s in &c.settlements {
                let i = (s.placement.y).min(gh - 1) * gw + (s.placement.x).min(gw - 1);
                let v = conf[i];
                n2 += usize::from(v > 0.0);
                // 0.35 is above any order-2 confluence's value (0.3 at most), so
                // this counts settlements within half the reach of order >= 3.
                n3 += usize::from(v >= 0.35);
                coast += usize::from(near_ocean(&c.water_bodies, gw, gh, i, reach_cells as isize));
                badge += usize::from(s.placement.coastal);
            }
            let hi = conf.iter().filter(|&&v| v >= 0.999).count();
            println!(
                "SITE {:<12} terms={} settlements={:<3} near_confl_o2={:<3} near_confl_o3={:<3} (confluence cells order>=4: {}) within4km_ocean={:<3} harbours={:<3}",
                w.tag, if terms { "on " } else { "off" }, c.settlements.len(), n2, n3, hi, coast, badge
            );
            dump(w.tag, if terms { "site_on" } else { "site_off" }, &c, &p, &net.0);
        }
    }
}

/// Whether any ocean cell lies within `r` cells of cell `i` (for a land cell,
/// where `land_distance` is zero because it measures to the complement).
fn near_ocean(wb: &[u8], gw: usize, gh: usize, i: usize, r: isize) -> bool {
    let (x, y) = ((i % gw) as isize, (i / gw) as isize);
    for dy in -r..=r {
        for dx in -r..=r {
            let (nx, ny) = (x + dx, y + dy);
            if nx >= 0 && ny >= 0 && nx < gw as isize && ny < gh as isize && wb[ny as usize * gw + nx as usize] == 1 && dx * dx + dy * dy <= r * r {
                return true;
            }
        }
    }
    false
}
