//! `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9's real headline measurement.
//!
//! Flow accumulation is the first GPU kernel in this project that is NOT a leaf computation.
//! Noise, blur, hillshade and the rest either terminate in a rendered pixel or feed a field that is
//! itself only compared against its own CPU twin. Flow accumulation feeds rivers, rivers feed
//! settlement suitability, suitability feeds roads and territory -- so "the numbers agree to 1e-4"
//! is not by itself a sufficient answer. What matters is whether the *civilisation layer* comes out
//! in the same place.
//!
//! This example measures exactly that, and isolates it properly:
//!
//!   1. Generate one real world on the pure CPU path (`use_gpu=false`), so terrain, climate and the
//!      carved river valleys are all the untouched reference pipeline.
//!   2. Recompute flow accumulation over that world's OWN final height/rainfall fields, twice --
//!      once with the real `cartalith_hydrology::compute_flow`, once with the GPU kernel.
//!   3. Run the settlement chain (`compute_civilisation`'s own sequence, transcribed) twice, with
//!      every input identical except `flow_discharge`.
//!   4. Compare the resulting settlement seeds position for position.
//!
//! Holding the terrain fixed is the point: running `generate_terrain` twice with `use_gpu` flipped
//! would also swap the noise, plate assignment and weather kernels, and the resulting settlement
//! differences would say nothing about flow accumulation specifically.
//!
//! Run with:  cargo run --release -p cartalith-gpu --example flow_downstream_settlements

use cartalith_civ as civ;
use cartalith_engine::{WorldParams, WorldState};

fn settlement_seeds(
    ws: &WorldState,
    flow: &[f32],
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
) -> (Vec<civ::SettlementSeed>, Vec<f32>) {
    let sea = ws.sea_level;
    let wb = civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
    let biome = civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let soil_slope = civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = civ::build_lithology(
        &ws.field,
        &ws.age_field,
        &ws.volcanic_field,
        &ws.crust_field,
        &ws.resistance_field,
        &ws.rainfall,
        sea,
    );
    let soil = civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = civ::build_water_access(flow, &ws.field, gw, gh, sea, flow_thresh);
    let carrying_cap = civ::build_carrying_capacity(
        &soil,
        &water_access,
        Some(&biome),
        &ws.temperature,
        &ws.field,
        sea,
        0.0,
        None,
    );
    let resources = civ::build_resource_potentials(
        &lithology,
        Some(&ws.boundary_type),
        Some(&ws.shear_field),
        Some(flow),
        Some(&biome),
        &ws.field,
        &ws.rainfall,
        &ws.age_field,
        gw,
        gh,
        sea,
        Some(&ws.volcanic_field),
        true,
        false,
    );
    let raw_slope = civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = civ::build_route_corridors(&ws.field, &raw_slope, Some(flow), gw, gh, sea, world, flow_thresh);
    let landmass = civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea, world);
    let coast_sdf = civ::build_coast_sdf(&ws.field, gw, gh, sea);
    let flood = civ::build_flood_field(&ws.field, flow, &raw_slope, gw, gh, sea);
    let river_order = civ::fresh_river_order(&ws.field, flow, gw, gh, sea, world, river_density, map_width_km);

    let ctx = civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(flow),
        river_order: Some(&river_order),
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };
    let slope_n = civ::build_slope_field(&ws.field, gw, gh, world);
    let suit = civ::build_settlement_suitability(
        &soil,
        &water_access,
        &carrying_cap,
        &ws.field,
        &slope_n,
        gw,
        gh,
        sea,
        Some(&ctx),
    );
    let seeds = civ::find_settlement_seeds(&suit, gw, gh, 0.42, (gw as f64 / 22.0).floor().max(6.0));
    (seeds, suit)
}

fn main() {
    for size in [512usize, 1024] {
        run(size);
    }
}

fn run(size: usize) {
    let (gw, gh) = (size, size);
    let mut p = WorldParams::defaults(gw, gh, 20250817);
    p.use_gpu = false;
    let world = p.world;
    let (km, density) = (p.map_width_km, p.river_density);

    eprintln!("generating one reference world on the pure CPU path ({gw}x{gh}, seed {})...", p.tect.seed);
    let ws = cartalith_engine::generate_terrain(&p);

    let Ok(gpu) = cartalith_gpu::init_gpu_shared_device() else {
        eprintln!("no GPU adapter available -- nothing to compare");
        return;
    };
    let ctx = cartalith_gpu::init_gpu_flow_with(&gpu);

    // Step 2: the same accumulation, two algorithms, over this world's own final fields.
    let flow_cpu = cartalith_hydrology::compute_flow(gw, gh, &ws.field, Some(&ws.rainfall), true, world);
    let out = cartalith_gpu::dispatch_gpu_flow(&ctx, gw, gh, &ws.field, Some(&ws.rainfall), true, world);
    let flow_gpu = out.acc;

    let mut max_rel = 0.0f64;
    let mut max_abs = 0.0f64;
    let mut cpu_max = 0.0f64;
    for (g, c) in flow_gpu.iter().zip(flow_cpu.iter()) {
        let (g, c) = (*g as f64, *c as f64);
        cpu_max = cpu_max.max(c);
        max_abs = max_abs.max((g - c).abs());
        max_rel = max_rel.max((g - c).abs() / c.abs().max(1.0));
    }
    println!("flow accumulation: max_abs={max_abs:.6}, max_rel={max_rel:.3e}, cpu_max_acc={cpu_max:.1}");
    // Where does the worst relative error actually live?
    let mut worst = (0usize, 0.0f64);
    for (i, (g, c)) in flow_gpu.iter().zip(flow_cpu.iter()).enumerate() {
        let r = ((*g as f64) - (*c as f64)).abs() / (*c as f64).abs().max(1.0);
        if r > worst.1 { worst = (i, r); }
    }
    println!(
        "  worst rel cell {}: cpu={} gpu={} (channel-initiation threshold is {:.1}, fixed-point step {:.3e})",
        worst.0,
        flow_cpu[worst.0],
        flow_gpu[worst.0],
        cartalith_hydrology::river_flow_thresh(gw, gh, gw, km),
        1.0 / out.fixed_point_scale
    );
    for &floor in &[1.0f64, 10.0, 100.0, 1000.0, cartalith_hydrology::river_flow_thresh(gw, gh, gw, km)] {
        let mut m = 0.0f64; let mut cnt = 0usize;
        for (g, c) in flow_gpu.iter().zip(flow_cpu.iter()) {
            if (*c as f64) < floor { continue; }
            cnt += 1;
            m = m.max(((*g as f64) - (*c as f64)).abs() / (*c as f64));
        }
        println!("  cells with cpu>={floor}: {cnt}, max rel error {m:.3e}");
    }

    // Step 3/4: identical everything but the flow field.
    let (seeds_cpu, suit_cpu) = settlement_seeds(&ws, &flow_cpu, gw, gh, world, km, density);
    let (seeds_gpu, suit_gpu) = settlement_seeds(&ws, &flow_gpu, gw, gh, world, km, density);

    let suit_diff = suit_gpu.iter().zip(suit_cpu.iter()).filter(|(a, b)| a != b).count();
    let suit_max = suit_gpu
        .iter()
        .zip(suit_cpu.iter())
        .map(|(a, b)| ((*a as f64) - (*b as f64)).abs())
        .fold(0.0f64, f64::max);
    println!(
        "settlement suitability raster: {suit_diff}/{} cells differ at all, max abs difference {suit_max:.3e}",
        suit_cpu.len()
    );

    println!("settlement seeds: CPU={} GPU={}", seeds_cpu.len(), seeds_gpu.len());
    let common = seeds_cpu.len().min(seeds_gpu.len());
    let moved = (0..common).filter(|&i| seeds_cpu[i].x != seeds_gpu[i].x || seeds_cpu[i].y != seeds_gpu[i].y).count();
    println!("settlement seeds in a different position (rank for rank, first {common}): {moved}");
    for i in 0..common {
        let (a, b) = (&seeds_cpu[i], &seeds_gpu[i]);
        if a.x != b.x || a.y != b.y {
            println!("  rank {i}: CPU ({},{}) score {:.6} vs GPU ({},{}) score {:.6}", a.x, a.y, a.score, b.x, b.y, b.score);
        }
    }
}
