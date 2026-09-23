//! The four Phase 2 affordance kernels (`src/affordance.rs`) against the real
//! `cartalith-civ` functions, on real generated worlds. Skips when this
//! machine opens no GPU. A non-multiple-of-4 cell count is included on
//! purpose: every u8 field is packed four cells to a word, and the tail word
//! is where a packing bug would live.
use cartalith_civ as civ;
use cartalith_engine::{generate_terrain, WorldParams};

fn max_abs(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(&x, &y)| (x as f64 - y as f64).abs()).fold(0.0, f64::max)
}

fn bits_eq(a: &[f32], b: &[f32]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

#[test]
fn affordance_kernels_match_cpu_on_real_worlds() {
    let Some(set) = cartalith_gpu::init_gpu_device_set().ok() else {
        eprintln!("no GPU available -- skipping");
        return;
    };
    let gpu = set.primary();

    // 251*171 = 42 921 cells, 1 past a multiple of 4.
    for (gw, gh) in [(256usize, 256usize), (251, 171)] {
        let p = WorldParams::defaults(gw, gh, 12345);
        let ws = generate_terrain(&p);
        let (sea, world) = (ws.sea_level, p.world);
        let wb = civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
        let slope_n = civ::build_slope_field(&ws.field, gw, gh, world);
        let lith = civ::build_lithology(
            &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea,
        );
        let soil = civ::build_soil_fertility(&lith, &ws.temperature, &ws.rainfall, &slope_n, &ws.age_field);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
        let water = civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
        let wet = civ::build_wetland_mask(&wb.classification, &ws.field, &ws.rainfall, &slope_n, sea);

        // Biome: bit-identical, and not vacuously so -- land, ocean and lake
        // classes must all be present.
        let biome = civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
        let biome_gpu = cartalith_gpu::biome_raster_grid_gpu_with(gpu, &wb.classification, &ws.temperature, &ws.rainfall).unwrap();
        assert_eq!(biome, biome_gpu, "{gw}x{gh}: biome must be bit-identical");
        assert!(biome.contains(&0) && biome.iter().any(|&b| (1..=12).contains(&b)), "{gw}x{gh}: degenerate world");

        for (bk, wm) in [(0.0, None), (1.0, Some(wet.as_slice()))] {
            let cpu = civ::build_carrying_capacity(&soil, &water, Some(&biome), &ws.temperature, &ws.field, sea, bk, wm);
            let run = || {
                cartalith_gpu::carrying_capacity_grid_gpu_with(gpu, &soil, &water, &biome, &ws.temperature, &ws.field, sea, bk, wm)
                    .unwrap()
            };
            let (g1, g2) = (run(), run());
            let d = max_abs(&cpu, &g1);
            assert!(d <= cartalith_gpu::CARRYING_TOLERANCE, "{gw}x{gh} k={bk}: carrying |d| {d:e}");
            assert!(bits_eq(&g1, &g2), "{gw}x{gh}: carrying not deterministic across dispatches");
            assert!(cpu.iter().any(|&v| v > 0.1), "{gw}x{gh}: carrying capacity all ~0");
        }
        let k = civ::build_carrying_capacity(&soil, &water, Some(&biome), &ws.temperature, &ws.field, sea, 0.0, None);

        // Resources: the kernel, then the CPU scarcity tail on top of it.
        let res_cpu = |scarcity| {
            civ::build_resource_potentials(
                &lith, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome), &ws.field,
                &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), scarcity, false,
            )
        };
        let cu_dist = civ::resource_copper_dist(Some(&ws.boundary_type), gw, gh);
        let fields = cartalith_gpu::resource_potentials_grid_gpu_with(
            gpu,
            &cartalith_gpu::ResourceGpuInputs {
                lith: &lith,
                boundary_type: Some(&ws.boundary_type),
                shear_field: Some(&ws.shear_field),
                flow: Some(&ws.flow_discharge),
                biome: Some(&biome),
                field: &ws.field,
                rain: &ws.rainfall,
                age: &ws.age_field,
                volcanic: Some(&ws.volcanic_field),
                cu_dist: &cu_dist,
                flow_max: civ::resource_flow_max(Some(&ws.flow_discharge)),
                gw,
                sea,
            },
        )
        .unwrap();
        let as15 = |r: &civ::ResourcePotentials| -> [Vec<f32>; 15] {
            [
                &r.copper, &r.tin, &r.iron, &r.gold, &r.salt, &r.timber, &r.lead, &r.silver, &r.clay, &r.buildstone,
                &r.flint, &r.obsidian, &r.gems, &r.sulfur, &r.alum,
            ]
            .map(|v| v.clone())
        };
        let raw = as15(&res_cpu(false));
        for (key, (c, g)) in civ::RESOURCE_KEYS.iter().zip(raw.iter().zip(fields.iter())) {
            let d = max_abs(c, g);
            assert!(d <= cartalith_gpu::RESOURCES_TOLERANCE, "{gw}x{gh} {key}: |d| {d:e}");
        }
        let res_gpu = civ::finish_resource_potentials(fields, &ws.field, sea, true, false);
        let res = res_cpu(true);
        for (key, (c, g)) in civ::RESOURCE_KEYS.iter().zip(as15(&res).iter().zip(as15(&res_gpu).iter())) {
            let d = max_abs(c, g);
            assert!(d <= cartalith_gpu::RESOURCES_TOLERANCE, "{gw}x{gh} {key} after scarcity: |d| {d:e}");
        }

        // Suitability, production context.
        let raw_slope = civ::build_raw_slope_field(&ws.field, gw, gh, world);
        let corridors = civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
        let landmass = civ::build_landmass_quality(&ws.field, Some(&k), gw, gh, sea, world);
        let coast_reach = civ::build_coast_reach(
            &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea),
            &wb.classification,
            gw,
            gh,
        );
        let flood = civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
        let (order, polys) = civ::fresh_river_network(
            &ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, p.map_width_km, ws.integrated_drainage,
        );
        let river_reach = civ::build_river_reach(&polys, &order, gw, gh);
        let ctx = civ::SuitabilityCtx {
            water_bodies: Some(&wb.classification),
            corridor: Some(&corridors),
            landmass: Some(&landmass.quality),
            flow: Some(&ws.flow_discharge),
            river_reach: Some(&river_reach),
            coast_reach: Some(&coast_reach),
            resources: Some(&res),
            rain: Some(&ws.rainfall),
            flood: Some(&flood),
            slope_raw: Some(&raw_slope),
            flow_thresh,
        };
        let cpu = civ::build_settlement_suitability(&soil, &water, &k, &ws.field, &slope_n, gw, gh, sea, Some(&ctx));
        let inp = cartalith_gpu::SuitabilityGpuInputs {
            soil: &soil,
            water: &water,
            carrying_cap: &k,
            field: &ws.field,
            slope_n: &slope_n,
            water_bodies: &wb.classification,
            corridor: &corridors,
            landmass: &landmass.quality,
            flow: &ws.flow_discharge,
            river_reach: &river_reach,
            coast_reach: &coast_reach,
            resources: [&res.copper, &res.tin, &res.iron, &res.gold, &res.salt, &res.timber, &res.lead, &res.silver, &res.gems],
            rain: &ws.rainfall,
            flood: &flood,
            slope_raw: &raw_slope,
            flow_thresh,
            gw,
            gh,
            sea,
        };
        let wt = cartalith_gpu::SuitabilityWeights {
            k: civ::SUIT_W_FULL_K,
            w: civ::SUIT_W_FULL_W,
            a: civ::SUIT_W_FULL_A,
            d: civ::SUIT_W_FULL_D,
            agri: civ::SUIT_W_FULL_AGRI,
            build: civ::SUIT_W_FULL_BUILD,
            coast: civ::SUIT_W_FULL_COAST,
            river: civ::SUIT_W_FULL_RIVER,
            lake: civ::SUIT_W_FULL_LAKE,
            mineral: civ::SUIT_W_FULL_MINERAL,
            corridor: civ::SUIT_W_FULL_CORRIDOR,
            flood: civ::SUIT_W_FULL_FLOOD,
            islet: civ::SUIT_W_FULL_ISLET,
            islet_knee: civ::ISLET_KNEE,
        };
        let g1 = cartalith_gpu::settlement_suitability_grid_gpu_with(gpu, &inp, &wt).unwrap();
        let g2 = cartalith_gpu::settlement_suitability_grid_gpu_with(gpu, &inp, &wt).unwrap();
        let d = max_abs(&cpu, &g1);
        assert!(d <= cartalith_gpu::SUITABILITY_TOLERANCE, "{gw}x{gh}: suitability |d| {d:e}");
        assert!(bits_eq(&g1, &g2), "{gw}x{gh}: suitability not deterministic across dispatches");
        assert!(cpu.iter().any(|&v| v > civ::SETTLE_SEED_THRESH as f32), "{gw}x{gh}: no seedable cell");
    }
}
