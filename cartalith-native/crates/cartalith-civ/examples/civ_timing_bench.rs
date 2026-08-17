// Real wall-clock timing for cartalith-civ's own per-cell pipeline stages
// (CPU_MULTITHREADING_SCOPE.md milestone 2). Mirrors the real upstream
// chain `golden_parity_settlement_naming.rs`'s own `compute_named_settlements`
// helper already established (lithology -> soil/water access -> biome ->
// carrying capacity/NPP -> resource potentials -> corridors/landmass/coast
// SDF/flood -> settlement suitability -> travel cost), at this project's
// established sizes (128/512/1024/2048). `compute_civilisation()` itself
// (`cartalith-godot`) can't be benchmarked directly from here -- it's a
// private fn in a cdylib-only crate (`ARCHITECTURE.md`'s gdext boundary) --
// this covers the actual per-cell compute this milestone parallelized,
// which is the dominant cost of that function's own upstream half.
// `cargo run --release --example timing_bench -p cartalith-civ`.
use cartalith_engine::{generate_terrain, WorldParams};
use std::time::Instant;

fn run_civ_layer(ws: &cartalith_engine::WorldState, gw: usize, gh: usize, world: bool, map_width_km: f64) {
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level,
    );
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, ws.sea_level, flow_thresh);
    let wetland_mask = cartalith_civ::build_wetland_mask(&wb.classification, &ws.field, &ws.rainfall, &soil_slope, ws.sea_level);
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, ws.sea_level, 0.0, Some(&wetland_mask),
    );
    let npp = cartalith_civ::build_npp(&ws.temperature, &ws.rainfall, &ws.field, ws.sea_level, 3000.0);
    let _density = cartalith_civ::estimate_regional_density_km2(
        &carrying_cap, &water_access, Some(&biome), Some(&npp), &ws.field, ws.sea_level, Some(&wetland_mask),
    );

    let resources = cartalith_civ::build_resource_potentials(
        &lithology,
        Some(&ws.boundary_type),
        Some(&ws.shear_field),
        Some(&ws.flow_discharge),
        Some(&biome),
        &ws.field,
        &ws.rainfall,
        &ws.age_field,
        gw,
        gh,
        ws.sea_level,
        Some(&ws.volcanic_field),
        true,
        false,
    );

    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, ws.sea_level, world, flow_thresh);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, ws.sea_level, world);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, ws.sea_level);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, ws.sea_level);

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_order: None,
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };
    let slope_n = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let _suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &slope_n, gw, gh, ws.sea_level, Some(&ctx));
    let _travel_cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, ws.sea_level);
}

fn main() {
    for &size in &[128usize, 512, 1024, 2048] {
        let mut p = WorldParams::defaults(size, size, 12345);
        p.climate.w_iters = 12;
        let ws = generate_terrain(&p);

        // Warm up once, then best of 3 timed runs (same convention as
        // `cartalith-engine`'s own timing_bench).
        run_civ_layer(&ws, size, size, p.world, p.map_width_km);
        let mut best = f64::INFINITY;
        for _ in 0..3 {
            let start = Instant::now();
            run_civ_layer(&ws, size, size, p.world, p.map_width_km);
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed < best {
                best = elapsed;
            }
        }
        println!("{size}x{size}: {best:.4}s");
    }
}
