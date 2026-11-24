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
//! **threading** (`thread_km`, owner Ruling GS addendum): km of lane drawn
//! side by side (within 20 degrees) with another lane across open sea, 0.75 to 4 routing cells apart, away from ports and from a fork
//! (within 8 routing cells of a point where the lane touches another, and within `FORK_PART_KM` (5 km) of
//! a point where the two have parted, more than 4 routing cells apart: one trunk about to part); and the
//! **minimum standoff** (reported, not enforced since Ruling GS): the least
//! distance to land at any sample of a lane farther than the call radius from every port.
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

const WORLDS: [World; 10] = [
    World { tag: "483920@384", seed: 483_920, gw: 384, gh: 288, km: 800.0 },
    World { tag: "912345@384", seed: 912_345, gw: 384, gh: 288, km: 800.0 },
    World { tag: "246371@384", seed: 246_371, gw: 384, gh: 288, km: 800.0 },
    World { tag: "483920@2048", seed: 483_920, gw: 2048, gh: 1311, km: 800.0 },
    World { tag: "912345@2048", seed: 912_345, gw: 2048, gh: 1311, km: 800.0 },
    World { tag: "246371@2048", seed: 246_371, gw: 2048, gh: 1311, km: 800.0 },
    // The whole-planet world the road census uses (many ports): the stress case for lane bundles.
    World { tag: "246371@planet", seed: 246_371, gw: 2048, gh: 1311, km: 40_075.0 },
    World { tag: "owner83744", seed: 83_744, gw: 2048, gh: 1024, km: 200.0 },
    // The owner's Pad world "Dungormkarcrag" (2026-11-23, v.26.11.23-001): app defaults, threading along a strait.
    World { tag: "51203@2048", seed: 51_203, gw: 2048, gh: 1311, km: 800.0 },
    // The same world as the owner made it (4 096 x 2 048, 800 km, his parameter list), read from the file
    // `SEA_CENSUS_PARAMS` names; skipped when it is unset (the list is the owner's and is not committed).
    World { tag: "owner51203", seed: 51_203, gw: 4096, gh: 2048, km: 800.0 },
];

/// How near a threaded sample the two lanes must part for the stretch to count as a fork, km of map
/// (2026-11-23, the owner's Pad report on seed 51203). A labelled judgement, checked by measurement, not derived:
/// 483920 at 2 048 with lanes on (the fork the exemption was written for) still reads 0.00 km with it. An ideal
/// straight fork is not fully exempt: at 18 degrees two lanes reach 4 routing cells (3.1 km at 800 km over
/// 1 024 routing cells) about 9.6 km from where they touch, so the samples between about 1.8 km (past `lo`) and
/// 4.6 km from the touch count, near 2.8 km per lane. Two lanes side by side for longer than that count wherever
/// they touch.
const FORK_PART_KM: f64 = 5.0;

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
    let file = if w.tag == "owner51203" { std::env::var("SEA_CENSUS_PARAMS").ok().and_then(|f| std::fs::read_to_string(f).ok()) } else { None };
    if w.tag == "owner83744" || file.is_some() {
        let mut skipped = 0;
        // The file opens with prose lines; only `key: value` lines whose key has a row are applied.
        for line in file.as_deref().unwrap_or(OWNER_PARAMS).lines() {
            let Some((k, v)) = line.split_once(": ") else { continue };
            let val = match v {
                "true" => crate::params::Value::Bool(true),
                "false" => crate::params::Value::Bool(false),
                n => match n.parse() {
                    Ok(x) => crate::params::Value::Num(x),
                    Err(_) => continue,
                },
            };
            if crate::params::spec(k).is_none() {
                skipped += 1;
                continue;
            }
            crate::params::set(&mut p, k, val);
            // A value the setter clamped or refused would measure another world: say so.
            if crate::params::get(&p, k) != Some(val) {
                println!("owner params: {k} did not apply as {v}");
            }
        }
        println!("owner params: {skipped} keys without a row skipped");
        p.gw = w.gw;
        p.gh = w.gh;
        p.map_width_km = w.km;
        p.tect.seed = w.seed as _;
        // The census measures on the CPU path, as every other world here.
        p.use_gpu = false;
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
    thread_km: f64,
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
    // The nearest approach to land is judged away from ports, so samples within the call
    // radius (in map cells) of any port are skipped.
    let exempt = cartalith_civ::sea_lanes::CALL_RADIUS_CELLS / sc;
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
    // Owner's "threading" (Ruling GS addendum, 2026-11-22): lane drawn side by side with another
    // lane, 0.75 to 4 routing cells apart (so not on the same cells), away from ports (8 routing
    // cells, the junction where lanes leave one harbour). Counted in km of lane.
    {
        let step = (0.5 / sc).max(0.25);
        let samples: Vec<(usize, f64, f64, f64, f64)> = c
            .sea_routes
            .iter()
            .enumerate()
            .flat_map(|(li, s)| {
                s.pts.windows(2).filter(|w| (w[1].0 - w[0].0).abs() < gw as f64 / 2.0).flat_map(move |w| {
                    let l = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
                    let n = ((l / step).ceil() as usize).max(1);
                    (0..n).map(move |k| {
                        let t = k as f64 / n as f64;
                        let (ux, uy) = ((w[1].0 - w[0].0) / l.max(1e-9), (w[1].1 - w[0].1) / l.max(1e-9));
                        (li, w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t, ux, uy)
                    })
                })
            })
            .collect();
        let (lo, hi, junction) = (0.75 / sc, 4.0 / sc, 8.0 / sc);
        // Per sample, the distance to the nearest sample of any other lane (no angle or water test): where
        // it exceeds `hi` the lane has parted from every other.
        let apart: Vec<f64> = samples
            .iter()
            .map(|a| samples.iter().filter(|o| o.0 != a.0).map(|o| (o.1 - a.1).hypot(o.2 - a.2)).fold(f64::INFINITY, f64::min))
            .collect();
        let mut n = 0usize;
        for &(li, x, y, tx, ty) in &samples {
            if ports.iter().any(|pp| (pp.0 - x).hypot(pp.1 - y) <= junction) {
                continue;
            }
            // Lanes either side of a thin spit of land are not threaded: every point of the straight line between the two samples must be ocean.
            // Parallel means within 20 degrees: two lanes converging on a trunk are not threaded.
            let cos_max = 20f64.to_radians().cos();
            let sea = |ox: f64, oy: f64| {
                (0..=8).all(|k| {
                    let t = k as f64 / 8.0;
                    let (mx, my) = (x + (ox - x) * t, y + (oy - y) * t);
                    c.water_bodies[(my as usize).min(gh - 1) * gw + (mx as usize).min(gw - 1)] == 1
                })
            };
            let d = samples
                .iter()
                .filter(|o| o.0 != li && (o.3 * tx + o.4 * ty).abs() >= cos_max && sea(o.1, o.2))
                .map(|o| (o.1 - x).hypot(o.2 - y))
                .fold(f64::INFINITY, f64::min);
            if d > lo && d <= hi {
                // A fork is not side by side: within `junction` of a point where this lane touches another
                // (a shared trunk's end, `lo` or less apart) the two lanes are one trunk about to part, as
                // they are beside a harbour. 483920 at 2 048: the whole residue (8 samples, 3.12 km) was the
                // first 3 km of one fork, lanes diverging 18 degrees off a shared trunk (2026-11-22).
                // 2026-11-23 (owner's Pad report, seed 51203): a fork must also PART. Two lanes threading a long
                // strait touch now and then, and every touch exempted 8 routing cells either side of it, so the
                // bar read 0 over real threading. Now the stretch is a fork only where this lane, within
                // `FORK_PART_KM` of the sample, also stands more than `hi` from every other lane (the two have
                // parted); threading that never parts counts.
                let touch = samples.iter().any(|q| q.0 == li && (q.1 - x).hypot(q.2 - y) <= junction && samples.iter().any(|o| o.0 != li && (o.1 - q.1).hypot(o.2 - q.2) <= lo));
                let part_r = FORK_PART_KM / km_cell;
                let fork = touch && samples.iter().zip(&apart).any(|(q, &far)| q.0 == li && far > hi && (q.1 - x).hypot(q.2 - y) <= part_r);
                if !fork {
                    n += 1;
                }
            }
        }
        r.thread_km = n as f64 * step * km_cell;
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
        if w.tag == "owner51203" && std::env::var("SEA_CENSUS_PARAMS").is_err() {
            continue;
        }
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
                "SEA {:<12} lanes={} lanes_n={:<3} km={:<8.1} bundles={:<3} conv={:<3} shared={:<3} ports={:<3} unlaned={:<2} skipped={:<2} skipped5km={:<2} off_coast_km={:.2} min_standoff_km={:.2} thread_km={:.2} ({} ms)",
                w.tag, if lanes { "on " } else { "off" }, r.lanes, r.km, r.bundles, r.bundles_conv, r.shared, r.ports, r.unlaned, r.skipped, r.skipped5km, r.off_coast_km, r.min_standoff_km, r.thread_km, ms
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
        if w.tag == "owner51203" && std::env::var("SEA_CENSUS_PARAMS").is_err() {
            continue;
        }
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
