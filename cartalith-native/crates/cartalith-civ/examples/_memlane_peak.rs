//! `_memlane_peak` -- generation-peak allocation audit, 2026-09-27 re-run of
//! `MEMORY_OPTIMIZATION_SCOPE.md`'s `_peakaudit_peak` (d195ed6 / dd2f386),
//! updated to today's `compute_civilisation`. **Throwaway probe.**
//!
//! ```text
//! cargo run --release -p cartalith-civ --example _memlane_peak -- <gw> <gh> [seed] [km]
//! MEMLANE_HASH=1 ...   # also print FNV fingerprints of every output
//! MEMLANE_REF=1 ...    # WorldParams::defaults (the goldens' baseline) instead of the app's
//! ```
//!
//! Reproduces `cartalith-godot::compute_civilisation`'s default auto-populate
//! path (keep None, fixed counts off, villages/metropolis/biome_k off,
//! recovery Stable, CPU) call for call. Terrain parameters are the shipped
//! app's `cartalith_godot::params::defaults()` (its six divergence flags on)
//! unless MEMLANE_REF is set. Sea routes get no ocean/wind field (a <=240-wide
//! coarse grid, negligible memory).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static GMAX: AtomicUsize = AtomicUsize::new(0);

struct Tracking;

#[inline]
fn bump(n: usize) {
    let cur = LIVE.fetch_add(n, Ordering::Relaxed) + n;
    PEAK.fetch_max(cur, Ordering::Relaxed);
    GMAX.fetch_max(cur, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for Tracking {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            bump(l.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            bump(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let np = unsafe { System.realloc(p, l, new) };
        if !np.is_null() {
            if new >= l.size() {
                bump(new - l.size());
            } else {
                LIVE.fetch_sub(l.size() - new, Ordering::Relaxed);
            }
        }
        np
    }
}

#[global_allocator]
static A: Tracking = Tracking;

fn mib(b: usize) -> f64 {
    b as f64 / (1024.0 * 1024.0)
}
fn live() -> f64 {
    mib(LIVE.load(Ordering::Relaxed))
}
fn reset_peak() {
    PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
}
fn cp(label: &str, t0: &Instant) {
    println!(
        "STAGE {:<40} live {:9.2}  ceiling {:9.2}  t {:7.2}",
        label,
        live(),
        mib(PEAK.load(Ordering::Relaxed)),
        t0.elapsed().as_secs_f64()
    );
    reset_peak();
}

// ---- FNV-1a fingerprints ---------------------------------------------------
fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
fn fbytes<T: Copy>(v: &[T]) -> u64 {
    let n = std::mem::size_of_val(v);
    fnv(unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, n) })
}
fn fdbg<T: std::fmt::Debug>(v: &T) -> u64 {
    fnv(format!("{v:?}").as_bytes())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let gw: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(2048);
    let gh: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1311);
    let seed: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(483920);
    let km: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(800.0);
    let reference = std::env::var("MEMLANE_REF").is_ok();
    let n = gw * gh;
    println!("=== _memlane_peak {gw}x{gh} = {n} cells, seed {seed}, {km} km, {} params ===", if reference { "REFERENCE" } else { "APP" });

    let t0 = Instant::now();
    reset_peak();
    GMAX.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
    println!("baseline live {:.2} MiB", live());

    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    if !reference {
        // cartalith_godot::params::defaults()' six divergence flags.
        p.crater.physical_model = true;
        p.volc.exclude_transform = true;
        p.volc.edifice_model = true;
        p.integrate_drainage = true;
        p.tect.narrow_plate_base_blur = true;
        p.passes.glacial = true;
    }
    p.map_width_km = km;
    let ws = cartalith_engine::generate_terrain(&p);
    cp("generate_terrain", &t0);

    // ---- WorldState census --------------------------------------------------
    let mut resident = 0usize;
    let mut row = |name: &str, bytes: usize| {
        resident += bytes;
        println!("  WS {name:<22} {:9.2} MiB", mib(bytes));
    };
    row("field f32", ws.field.len() * 4);
    row("plate_id u16", ws.plate_id.len() * 2);
    row("boundary_mask u8", ws.boundary_mask.len());
    row("stress_field f32", ws.stress_field.len() * 4);
    row("age_field f32", ws.age_field.len() * 4);
    row("resistance_field f32", ws.resistance_field.len() * 4);
    row("crust_field f32", ws.crust_field.len() * 4);
    row("boundary_type u8", ws.boundary_type.len());
    row("shear_field f32", ws.shear_field.len() * 4);
    row("volcanic_field f32", ws.volcanic_field.len() * 4);
    row("impact_field f32", ws.impact_field.len() * 4);
    row("temperature f32", ws.temperature.len() * 4);
    row("rainfall f32", ws.rainfall.len() * 4);
    row("flow_discharge f32", ws.flow_discharge.len() * 4);
    if let Some(c) = ws.channels.as_ref() {
        row("channels.recv", std::mem::size_of_val(&c.recv[..]));
        row("channels.chan", std::mem::size_of_val(&c.chan[..]));
        row("channels.slope", std::mem::size_of_val(&c.slope[..]));
    }
    if let Some(o) = ws.stream_order.as_ref() {
        row("stream_order i16", o.len() * 2);
    }
    if let Some(m) = ws.river_mask.as_ref() {
        row("river_mask u8", m.len());
    }
    if let Some(f) = ws.river_floor.as_ref() {
        row("river_floor f32", f.len() * 4);
    }
    println!("  WS TOTAL {:9.2} MiB ({:.1} B/cell); allocator live {:.2}", mib(resident), resident as f64 / n as f64, live());
    reset_peak();

    // ---- compute_civilisation, default auto-populate path -------------------
    let opts = &p.civ;
    let want = opts.want_counts();
    let world = p.world;
    let sea_level = ws.sea_level;
    let map_width_km = p.map_width_km;

    let mut wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea_level, world, Some(&ws.rainfall));
    cp("build_water_bodies", &t0);
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    cp("build_biome_raster", &t0);
    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    cp("build_slope_field", &t0);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea_level,
    );
    cp("build_lithology", &t0);
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
    cp("build_soil_fertility", &t0);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea_level, flow_thresh);
    cp("build_water_access", &t0);
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea_level, 0.0, None,
    );
    cp("build_carrying_capacity", &t0);
    let mut resources = cartalith_civ::build_resource_potentials(
        &lithology, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome),
        &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea_level, Some(&ws.volcanic_field), true, false,
    );
    cp("build_resource_potentials", &t0);
    drop(lithology);
    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    cp("build_raw_slope_field", &t0);
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea_level, world, flow_thresh);
    cp("build_route_corridors", &t0);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea_level, world);
    cp("build_landmass_quality", &t0);
    let coast_reach = cartalith_civ::build_coast_reach(
        &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea_level),
        &wb.classification,
        gw,
        gh,
    );
    cp("trace_coastline + build_coast_reach", &t0);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea_level);
    cp("build_flood_field", &t0);
    let (river_order, river_polys) =
        cartalith_civ::fresh_river_network(&ws.field, &ws.flow_discharge, gw, gh, sea_level, world, p.river_density, map_width_km, ws.integrated_drainage);
    cp("fresh_river_network", &t0);
    let river_reach = cartalith_civ::build_river_reach(&river_polys, &river_order, gw, gh);
    drop(river_polys);
    cp("build_river_reach", &t0);

    println!(">>> SuitabilityCtx point: live {:.2} MiB", live());
    let mut civ_live = 0usize;
    let mut crow = |name: &str, bytes: usize| {
        civ_live += bytes;
        println!("  CIV {name:<22} {:8.2} MiB", mib(bytes));
    };
    crow("wb.classification", std::mem::size_of_val(&wb.classification[..]));
    crow("wb.fill_level", std::mem::size_of_val(&wb.fill_level[..]));
    crow("biome", std::mem::size_of_val(&biome[..]));
    crow("soil_slope", std::mem::size_of_val(&soil_slope[..]));
    crow("soil", std::mem::size_of_val(&soil[..]));
    crow("water_access", std::mem::size_of_val(&water_access[..]));
    crow("carrying_cap", std::mem::size_of_val(&carrying_cap[..]));
    crow("resources (15)", 15 * std::mem::size_of_val(&resources.copper[..]));
    crow("raw_slope", std::mem::size_of_val(&raw_slope[..]));
    crow("corridors", std::mem::size_of_val(&corridors[..]));
    crow("landmass.quality", std::mem::size_of_val(&landmass.quality[..]));
    crow("landmass.comp", std::mem::size_of_val(&landmass.comp[..]));
    crow("coast_reach", std::mem::size_of_val(&coast_reach[..]));
    crow("flood", std::mem::size_of_val(&flood[..]));
    crow("river_order", std::mem::size_of_val(&river_order[..]));
    crow("river_reach", std::mem::size_of_val(&river_reach[..]));
    println!("  CIV subtotal {:8.2} MiB ({:.1} B/cell); + WS = {:.2}", mib(civ_live), civ_live as f64 / n as f64, mib(civ_live + resident));
    reset_peak();

    macro_rules! mk_ctx {
        () => {
            cartalith_civ::SuitabilityCtx {
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
                flow_thresh,
            }
        };
    }
    let ctx = mk_ctx!();
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &soil_slope, gw, gh, sea_level, Some(&ctx));
    cp("build_settlement_suitability", &t0);

    let (thresh, supp_r) = match want {
        Some(w) => cartalith_civ::civ_want_counts_seed_params(gw, w.iter().sum()),
        None => (opts.seed_thresh, (gw as f64 / opts.seed_suppress_div.max(1.0)).floor().max(6.0)),
    };
    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, thresh, supp_r);
    let mut placements = cartalith_civ::place_settlements_with_counts(
        &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea_level, world, opts.factions.max(1),
        &flood, &ws.flow_discharge, flow_thresh, map_width_km, want,
    );
    cp(&format!("seeds+placement ({})", placements.len()), &t0);
    drop(suit);
    wb.fill_level = Vec::new();
    cp("suit + fill_level freed (villages off)", &t0);

    let passes = if want.is_some() { 1 } else { cartalith_civ::CIV_AUTO_WORLD_PASSES };
    let topology = cartalith_civ::civ_iterative_network(
        &mut placements, passes, gw, gh, sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &wb.classification, world, map_width_km,
    );
    cp("civ_iterative_network", &t0);
    let cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, sea_level);
    cp("build_travel_cost", &t0);
    let mut rng = cartalith_civ::civ_name_rng();
    let settlements = cartalith_civ::name_and_populate_settlements_with_rng(&placements, &mut rng, &[]);
    cp("name_and_populate", &t0);
    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&topology, &settlements, &ws.field, &wb.classification, gw, gh, map_width_km);
    cp("civ_consolidate_and_smooth_ways", &t0);

    let world_mean = cartalith_civ::civ_world_mean_resources(&resources, &ws.field, sea_level);
    let trade: Vec<cartalith_civ::TradeBalance> = settlements
        .iter()
        .map(|s| {
            let cat_km2 = cartalith_civ::civ_catchment_km2(s.placement.kind);
            let radius = cartalith_civ::civ_catchment_radius_cells(cat_km2, map_width_km, gw);
            let m = cartalith_civ::civ_place_resource_context(&resources, &ws.field, gw, gh, sea_level, s.placement.x as i64, s.placement.y as i64, radius, world);
            cartalith_civ::civ_resource_trade_balance(&m, &world_mean)
        })
        .collect();
    cp("world_mean + trade balances", &t0);
    resources.clay = Vec::new();
    resources.buildstone = Vec::new();
    resources.flint = Vec::new();
    resources.obsidian = Vec::new();
    resources.sulfur = Vec::new();
    resources.alum = Vec::new();
    cp("6 resource fields freed", &t0);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, sea_level);
    cp("build_coast_sdf (deferred)", &t0);
    let ctx = mk_ctx!();
    let explain: Vec<String> = settlements
        .iter()
        .map(|s| {
            let e = cartalith_civ::explain_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &soil_slope, gw, gh, sea_level, Some(&ctx), s.placement.x, s.placement.y);
            format!("{e:?}")
        })
        .collect();
    cp("explanations", &t0);
    drop(ctx);
    let _ = coast_sdf.len();
    drop((soil, soil_slope, raw_slope, corridors, coast_sdf, coast_reach, flood, river_reach, river_order, resources));
    cp("explanation-only rasters released", &t0);

    let territory = cartalith_civ::assign_territory(&settlements, &cost, gw, gh, world);
    cp("assign_territory", &t0);
    let (provinces, _province_list) = cartalith_civ::civ_generate_provinces(&settlements, &territory, gw, gh);
    cp("civ_generate_provinces", &t0);
    let continents = cartalith_civ::civ_continents_with_cultures(&landmass, gw, gh, 400, Some(&territory), &[]);
    cp("civ_continents_with_cultures", &t0);
    let ports: Vec<cartalith_civ::NamedSettlement> = settlements.iter().filter(|s| s.placement.coastal).cloned().collect();
    let sea_routes = cartalith_civ::civ_sea_routes(&ports, &ws.field, &wb.classification, gw, gh, world, map_width_km, None, None);
    cp("civ_sea_routes", &t0);
    let dens = cartalith_civ::timeline::civ_current_agrarian_density(&carrying_cap, &water_access, Some(&biome), &ws.rainfall, &ws.field, sea_level);
    cp("civ_current_agrarian_density", &t0);

    let civ_grids = territory.len() * 4 + provinces.len() * 4 + wb.classification.len() + dens.len() * 4;
    drop((cost, wb.fill_level));
    drop((carrying_cap, water_access, biome, seeds, placements, explain, trade));
    drop(landmass);
    let resident_after = live();
    println!("CivData grids {:.2} MiB; live after compute_civilisation's frees {:.2} MiB ({:.1} B/cell)",
        mib(civ_grids), resident_after, LIVE.load(Ordering::Relaxed) as f64 / n as f64);
    drop((topology, ways, territory, provinces, continents, sea_routes, dens, ws));
    let g = GMAX.load(Ordering::Relaxed);
    println!("live after dropping everything {:.2} MiB", live());
    println!("PEAK {:.2} MiB  ({:.1} B/cell)  total {:.2} s", mib(g), g as f64 / n as f64, t0.elapsed().as_secs_f64());
}
