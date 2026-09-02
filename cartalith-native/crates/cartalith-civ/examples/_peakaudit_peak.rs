//! `_peakaudit_peak` — generation-peak allocation audit. **Throwaway probe.**
//!
//! Written for the generation-peak audit recorded in
//! `MEMORY_OPTIMIZATION_SCOPE.md`. Not shipping code, not a test, not called
//! by anything: a `#[global_allocator]` wrapper plus a sampling thread, so
//! that the pipeline's heap high-water mark can be read *per stage* rather
//! than inferred from `dumpsys` or Windows private bytes after the fact.
//!
//! Lives in `cartalith-civ` because that is the only crate in the workspace
//! that can reach **both** halves of the pipeline: it depends on
//! `cartalith-engine` (so `generate_terrain` is callable) and it *is* the civ
//! layer (so every `build_*` stage `compute_civilisation` calls is callable).
//! `cartalith-godot` is a `cdylib` and cannot host an example at all.
//!
//! ```text
//! cargo run --release -p cartalith-civ --example _peakaudit_peak -- <gw> <gh> [seed]
//! cargo run --release -p cartalith-civ --example _peakaudit_peak -- trace <gw> <gh> [seed]
//! ```
//!
//! `trace` adds the 2 ms sampler over the whole run and dumps the heap
//! time-series, which is the only way to see *inside* `generate_terrain`
//! without editing it (this audit is forbidden from touching `.rs` files
//! outside this probe).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Tracking allocator
// ---------------------------------------------------------------------------

static LIVE: AtomicUsize = AtomicUsize::new(0);
/// Reset at every checkpoint, so each row reports its own stage's high-water.
static PEAK: AtomicUsize = AtomicUsize::new(0);
/// Never reset: the run's true high-water mark, which is the audit's headline.
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

fn live() -> f64 {
    LIVE.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0)
}
fn peak() -> f64 {
    PEAK.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0)
}
fn reset_peak() {
    PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
}

/// One checkpoint row: what is live now, and how high the heap went since the
/// previous checkpoint. The second column is the number that matters — a
/// stage whose own scratch is freed before it returns is invisible in `live`
/// and fully visible here.
fn cp(label: &str, t0: &Instant) {
    println!("{:<44} live {:9.2} MiB   peak-since {:9.2} MiB   t {:7.2} s", label, live(), peak(), t0.elapsed().as_secs_f64());
    reset_peak();
}

// ---------------------------------------------------------------------------

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// The OS's own high-water RSS, on Android/Linux only. The allocator counter
/// above measures *requested* bytes; this measures what the kernel actually
/// charged the process, so the gap between the two is the allocator's own
/// overhead (Scudo on Android, which is what the app pays).
fn rss_hwm_kb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest.trim().trim_end_matches(" kB").trim().parse().ok();
        }
    }
    None
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let trace = args.first().map(|s| s == "trace").unwrap_or(false);
    if trace {
        args.remove(0);
    }
    let gw: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(2048);
    let gh: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1311);
    let seed: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(483920);

    let n = gw * gh;
    println!("=== _peakaudit_peak  {gw}x{gh} = {n} cells, seed {seed} (FIXED) ===");

    // `regen` reproduces `WorldGen::generate_sized`'s ordering: the previous
    // world is still owned by `self.source`/`self.civ` when `generate_terrain`
    // is called for the new one, so the second generate's peak carries a whole
    // extra world. Measured rather than argued.
    if std::env::var("PEAKAUDIT_REGEN").is_ok() {
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
        p.map_width_km = 1200.0;
        let first = cartalith_engine::generate_terrain(&p);
        let held = first.field.len();
        reset_peak();
        GMAX.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
        println!("held previous WorldState: live {:.2} MiB ({held} cells)", live());
        let second = cartalith_engine::generate_terrain(&p);
        println!("generate #2 with #1 still held: peak {:.2} MiB, live {:.2} MiB", mib(GMAX.load(Ordering::Relaxed)), live());
        drop(first);
        println!("after dropping #1: live {:.2} MiB", live());
        drop(second);
        return;
    }
    println!("one f32 grid = {:.2} MiB, one u8 grid = {:.2} MiB, one usize grid = {:.2} MiB\n", mib(n * 4), mib(n), mib(n * 8));

    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let samples = std::sync::Arc::new(std::sync::Mutex::new(Vec::<(f64, usize)>::new()));
    let t0 = Instant::now();

    let sampler = if trace {
        let stop = stop.clone();
        let samples = samples.clone();
        let t0 = t0;
        Some(std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let v = LIVE.load(Ordering::Relaxed);
                samples.lock().unwrap().push((t0.elapsed().as_secs_f64(), v));
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }))
    } else {
        None
    };

    reset_peak();
    let base = live();
    println!("baseline live: {base:.2} MiB\n");

    // ---- TERRAIN ----------------------------------------------------------
    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.map_width_km = 1200.0;
    let ws = cartalith_engine::generate_terrain(&p);
    cp("generate_terrain (whole)", &t0);

    println!("\n--- WorldState resident fields ---");
    let mut resident = 0usize;
    let row = |name: &str, bytes: usize, resident: &mut usize| {
        *resident += bytes;
        println!("  {name:<22} {:9.2} MiB", mib(bytes));
    };
    row("field f32", ws.field.len() * 4, &mut resident);
    row("plate_id usize", ws.plate_id.len() * std::mem::size_of::<usize>(), &mut resident);
    row("boundary_mask u8", ws.boundary_mask.len(), &mut resident);
    row("stress_field f32", ws.stress_field.len() * 4, &mut resident);
    row("age_field f32", ws.age_field.len() * 4, &mut resident);
    row("resistance_field f32", ws.resistance_field.len() * 4, &mut resident);
    row("crust_field f32", ws.crust_field.len() * 4, &mut resident);
    row("boundary_type u8", ws.boundary_type.len(), &mut resident);
    row("shear_field f32", ws.shear_field.len() * 4, &mut resident);
    row("volcanic_field f32", ws.volcanic_field.len() * 4, &mut resident);
    row("impact_field f32", ws.impact_field.len() * 4, &mut resident);
    row("temperature f32", ws.temperature.len() * 4, &mut resident);
    row("rainfall f32", ws.rainfall.len() * 4, &mut resident);
    row("flow_discharge f32", ws.flow_discharge.len() * 4, &mut resident);
    if let Some(c) = ws.channels.as_ref() {
        row("channels.recv i32", c.recv.len() * 4, &mut resident);
        row("channels.chan u8", c.chan.len(), &mut resident);
        // `channels.slope` was a fourth row here until R2 released it.
        row("channels.slope f32", c.slope.len() * 4, &mut resident);
    }
    if let Some(o) = ws.stream_order.as_ref() {
        row("stream_order i16", o.len() * 2, &mut resident);
    }
    if let Some(m) = ws.river_mask.as_ref() {
        row("river_mask u8", m.len(), &mut resident);
    }
    if let Some(f) = ws.river_floor.as_ref() {
        row("river_floor f32", f.len() * 4, &mut resident);
    }
    println!("  {:<22} {:9.2} MiB   ({:.1} bytes/cell)", "TOTAL WorldState", mib(resident), resident as f64 / n as f64);
    println!("  (allocator says live = {:.2} MiB)\n", live());
    reset_peak();

    // ---- CIVILISATION -----------------------------------------------------
    // `compute_civilisation` (cartalith-godot/src/lib.rs), call for call, at
    // its shipping defaults (villages/metropolis/biome_k off, recovery
    // Stable). Reproduced rather than called because that function is private
    // and lives in a cdylib.
    let sea = ws.sea_level;
    let world = p.world;
    let map_width_km = p.map_width_km;

    println!("--- compute_civilisation, stage by stage ---");
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
    cp("build_water_bodies", &t0);
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    cp("build_biome_raster", &t0);
    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    cp("build_slope_field", &t0);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea,
    );
    cp("build_lithology", &t0);
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
    cp("build_soil_fertility", &t0);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
    cp("build_water_access", &t0);
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea, 0.0, None,
    );
    cp("build_carrying_capacity", &t0);
    let mut resources = cartalith_civ::build_resource_potentials(
        &lithology, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome),
        &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), true, false,
    );
    cp("build_resource_potentials (15 f32 grids)", &t0);
    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    cp("build_raw_slope_field", &t0);
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
    cp("build_route_corridors", &t0);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea, world);
    cp("build_landmass_quality", &t0);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, sea);
    cp("build_coast_sdf", &t0);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
    cp("build_flood_field", &t0);
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, map_width_km);
    cp("fresh_river_order", &t0);

    // The high-water point the scope document names: everything above is
    // alive simultaneously here.
    println!("\n  >>> SuitabilityCtx point: live {:.2} MiB ({:.1} B/cell over the whole run)", live(), (LIVE.load(Ordering::Relaxed) as f64) / n as f64);
    println!("      civ-side fields alive here:");
    let mut civ_live = 0usize;
    let crow = |name: &str, bytes: usize, acc: &mut usize| {
        *acc += bytes;
        println!("        {name:<28} {:8.2} MiB", mib(bytes));
    };
    crow("wb.classification", wb.classification.len() * std::mem::size_of_val(&wb.classification[0]), &mut civ_live);
    crow("wb.fill_level", wb.fill_level.len() * std::mem::size_of_val(&wb.fill_level[0]), &mut civ_live);
    crow("biome", biome.len() * std::mem::size_of_val(&biome[0]), &mut civ_live);
    crow("soil_slope", soil_slope.len() * 4, &mut civ_live);
    crow("lithology", lithology.len() * std::mem::size_of_val(&lithology[0]), &mut civ_live);
    crow("soil", soil.len() * 4, &mut civ_live);
    crow("water_access", water_access.len() * 4, &mut civ_live);
    crow("carrying_cap", carrying_cap.len() * 4, &mut civ_live);
    let res_bytes = 15 * resources.copper.len() * 4;
    crow("resources (15 x f32)", res_bytes, &mut civ_live);
    crow("raw_slope", raw_slope.len() * 4, &mut civ_live);
    crow("corridors", corridors.len() * 4, &mut civ_live);
    crow("landmass.quality", landmass.quality.len() * 4, &mut civ_live);
    crow("coast_sdf", coast_sdf.len() * 4, &mut civ_live);
    crow("flood", flood.len() * 4, &mut civ_live);
    crow("river_order", river_order.len() * std::mem::size_of_val(&river_order[0]), &mut civ_live);
    println!("        {:<28} {:8.2} MiB  ({:.1} B/cell)", "civ subtotal", mib(civ_live), civ_live as f64 / n as f64);
    println!("        {:<28} {:8.2} MiB", "+ WorldState resident", mib(resident));
    println!("        {:<28} {:8.2} MiB\n", "= census total", mib(civ_live + resident));
    reset_peak();

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_order: Some(&river_order),
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx));
    cp("build_settlement_suitability", &t0);
    drop(ctx);

    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.42, (gw as f64 / 22.0).floor().max(6.0));
    let placements = cartalith_civ::place_settlements_with_water_edge_snap(
        &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea, world, 6,
        &flood, &ws.flow_discharge, flow_thresh, map_width_km,
    );
    cp(&format!("seeds+placement ({} places)", placements.len()), &t0);

    let topology = cartalith_civ::civ_hierarchical_network_topology(
        &placements, gw, gh, sea, &ws.field, &ws.flow_discharge, &river_order, &biome, &wb.classification, world, map_width_km,
    );
    cp("civ_hierarchical_network_topology", &t0);

    let cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, sea);
    cp("build_travel_cost", &t0);

    let mut rng = cartalith_civ::civ_name_rng();
    let settlements = cartalith_civ::name_and_populate_settlements_with_rng(&placements, &mut rng);
    cp("name_and_populate", &t0);

    let _world_mean = cartalith_civ::civ_world_mean_resources(&resources, &ws.field, sea);
    cp("civ_world_mean_resources", &t0);

    // The six unused resource fields are emptied here in the real function.
    let freed = 6 * resources.clay.len() * 4;
    resources.clay = Vec::new();
    resources.buildstone = Vec::new();
    resources.flint = Vec::new();
    resources.obsidian = Vec::new();
    resources.sulfur = Vec::new();
    resources.alum = Vec::new();
    println!("  [the 2026-08-16 fix frees {:.2} MiB here]", mib(freed));
    cp("resources: 6 unused fields emptied", &t0);

    let territory = cartalith_civ::assign_territory(&settlements, &cost, gw, gh, world);
    cp("assign_territory", &t0);
    let (provinces, _plist) = cartalith_civ::civ_generate_provinces(&settlements, &territory, gw, gh);
    cp("civ_generate_provinces", &t0);
    let continents = cartalith_civ::civ_continents(&landmass, gw, gh, 400, Some(&territory));
    cp("civ_continents", &t0);
    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&topology, &settlements, &ws.field, &wb.classification, gw, gh, map_width_km);
    cp("civ_consolidate_and_smooth_ways", &t0);

    println!("\n--- CivData resident grids ---");
    println!("  territory  i32  {:8.2} MiB", mib(territory.len() * 4));
    println!("  provinces  i32  {:8.2} MiB", mib(provinces.len() * 4));
    println!("  water_bodies    {:8.2} MiB", mib(wb.classification.len() * std::mem::size_of_val(&wb.classification[0])));
    println!("  ways/settlements/continents: {} ways, {} settlements, {} continents", ways.len(), settlements.len(), continents.len());

    // Everything compute_civilisation drops on return.
    drop(suit);
    drop(cost);
    drop(soil);
    drop(soil_slope);
    drop(raw_slope);
    drop(corridors);
    drop(landmass);
    drop(coast_sdf);
    drop(flood);
    drop(river_order);
    drop(carrying_cap);
    drop(water_access);
    drop(lithology);
    drop(biome);
    drop(resources);
    drop(topology);
    drop(placements);
    drop(seeds);
    println!("\nafter compute_civilisation's own frees: live {:.2} MiB", live());
    println!("(WorldState + CivData is what the session then holds)");

    stop.store(true, Ordering::Relaxed);
    if let Some(h) = sampler {
        let _ = h.join();
        let s = samples.lock().unwrap();
        println!("\n--- heap trace, {} samples ---", s.len());
        // Print a downsampled trace plus the true maximum.
        let mut mx = (0.0f64, 0usize);
        for &(t, v) in s.iter() {
            if v > mx.1 {
                mx = (t, v);
            }
        }
        println!("trace maximum: {:.2} MiB at t = {:.2} s", mib(mx.1), mx.0);
        let step = (s.len() / 120).max(1);
        for (i, &(t, v)) in s.iter().enumerate() {
            if i % step == 0 {
                println!("  {t:7.3} s  {:9.2} MiB", mib(v));
            }
        }
    }
    println!("\nOVERALL allocator peak (never reset): {:.2} MiB", mib(GMAX.load(Ordering::Relaxed)));
    if let Some(kb) = rss_hwm_kb() {
        println!("OS VmHWM (real RSS high-water):        {:.2} MiB", kb as f64 / 1024.0);
    }
    println!("threads: {}", std::thread::available_parallelism().map(|v| v.get()).unwrap_or(0));
}
