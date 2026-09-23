//! CPU vs GPU for the four Phase 2 affordance fields (`OUTSTANDING_WORK.md`
//! §2.6), on real generated worlds: numeric agreement, GPU run-to-run
//! determinism, the settlement list the suitability field produces, and
//! timing.
//!
//! `cargo run --release -p cartalith-gpu --example affordance_gpu_compare -- [sizes...]`
//! (default 512 1024 2048). Run it alone -- timings under a parallel build or
//! test run measure the machine, not the code (`MISTAKES.md`).
//!
//! Every CPU call here is the real `cartalith-civ` function, and the upstream
//! chain is `compute_civilisation`'s (`cartalith-godot/src/lib.rs`), so a
//! constant restated in a shader that drifted from civ shows up as a
//! disagreement.
use std::time::{Duration, Instant};

use cartalith_civ as civ;
use cartalith_engine::{generate_terrain, WorldParams};

const ROUNDS: usize = 5;

fn timed<T>(mut f: impl FnMut() -> T) -> (T, Duration, Duration, Duration) {
    let mut out = f(); // warm-up, discarded from the samples
    let mut s = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let t = Instant::now();
        out = f();
        s.push(t.elapsed());
    }
    s.sort();
    (out, s[ROUNDS / 2], s[0], s[ROUNDS - 1])
}

fn fmt(t: (Duration, Duration, Duration)) -> String {
    format!("{:.1} ms ({:.1}..{:.1})", t.0.as_secs_f64() * 1e3, t.1.as_secs_f64() * 1e3, t.2.as_secs_f64() * 1e3)
}

fn max_abs(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(&x, &y)| (x as f64 - y as f64).abs()).fold(0.0, f64::max)
}

fn differing(a: &[f32], b: &[f32]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
}

fn res_fields(r: &civ::ResourcePotentials) -> [&[f32]; 15] {
    [
        &r.copper, &r.tin, &r.iron, &r.gold, &r.salt, &r.timber, &r.lead, &r.silver, &r.clay, &r.buildstone, &r.flint,
        &r.obsidian, &r.gems, &r.sulfur, &r.alum,
    ]
}

fn suit_res(r: &civ::ResourcePotentials) -> [&[f32]; 9] {
    [&r.copper, &r.tin, &r.iron, &r.gold, &r.salt, &r.timber, &r.lead, &r.silver, &r.gems]
}

fn weights() -> cartalith_gpu::SuitabilityWeights {
    cartalith_gpu::SuitabilityWeights {
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
    }
}

fn main() {
    let sizes: Vec<usize> = {
        let a: Vec<usize> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
        if a.is_empty() { vec![512, 1024, 2048] } else { a }
    };
    let set = cartalith_gpu::init_gpu_device_set().expect("no GPU device -- this comparison needs real hardware");
    let gpu = set.primary();
    println!("GPU: {} ({:?})", gpu.adapter_name, gpu.adapter_backend);

    for &size in &sizes {
        let (gw, gh) = (size, size);
        let mut p = WorldParams::defaults(gw, gh, 12345);
        p.climate.w_iters = 12;
        let ws = generate_terrain(&p);
        let sea = ws.sea_level;
        let world = p.world;
        println!("\n=== {gw}x{gh} (seed 12345, sea {sea:.4}) ===");

        // compute_civilisation's upstream chain, verbatim.
        let wb = civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
        let soil_slope = civ::build_slope_field(&ws.field, gw, gh, world);
        let lith = civ::build_lithology(
            &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea,
        );
        let soil = civ::build_soil_fertility(&lith, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
        let water = civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
        let wet = civ::build_wetland_mask(&wb.classification, &ws.field, &ws.rainfall, &soil_slope, sea);

        // -- biome
        let (b_cpu, t0, t1, t2) = timed(|| civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall));
        let bc = (t0, t1, t2);
        let (b_gpu, t0, t1, t2) =
            timed(|| cartalith_gpu::biome_raster_grid_gpu_with(gpu, &wb.classification, &ws.temperature, &ws.rainfall).expect("gpu biome"));
        let bg = (t0, t1, t2);
        let b_diff = b_cpu.iter().zip(&b_gpu).filter(|(a, b)| a != b).count();
        println!("biome        cells differing: {b_diff}   CPU {}   GPU {}", fmt(bc), fmt(bg));

        // -- carrying capacity: both biome_k settings (the residual table is only read at 1.0)
        for (bk, wm) in [(0.0, None), (1.0, Some(wet.as_slice()))] {
            let (k_cpu, t0, t1, t2) = timed(|| {
                civ::build_carrying_capacity(&soil, &water, Some(&b_cpu), &ws.temperature, &ws.field, sea, bk, wm)
            });
            let kc = (t0, t1, t2);
            let (k_gpu, t0, t1, t2) = timed(|| {
                cartalith_gpu::carrying_capacity_grid_gpu_with(gpu, &soil, &water, &b_cpu, &ws.temperature, &ws.field, sea, bk, wm)
                    .expect("gpu carrying")
            });
            let kg = (t0, t1, t2);
            let again = cartalith_gpu::carrying_capacity_grid_gpu_with(gpu, &soil, &water, &b_cpu, &ws.temperature, &ws.field, sea, bk, wm).unwrap();
            println!(
                "carrying k={bk}  max|d| {:.3e}  cells differing {}  gpu-vs-gpu differing {}   CPU {}   GPU {}",
                max_abs(&k_cpu, &k_gpu),
                differing(&k_cpu, &k_gpu),
                differing(&k_gpu, &again),
                fmt(kc),
                fmt(kg)
            );
        }
        let k_cpu = civ::build_carrying_capacity(&soil, &water, Some(&b_cpu), &ws.temperature, &ws.field, sea, 0.0, None);

        // -- resources: the kernel alone (scarcity off), then with the CPU scarcity tail
        let res_cpu_call = |scarcity: bool| {
            civ::build_resource_potentials(
                &lith, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&b_cpu), &ws.field,
                &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), scarcity, false,
            )
        };
        let res_gpu_call = || {
            let cu_dist = civ::resource_copper_dist(Some(&ws.boundary_type), gw, gh);
            let inp = cartalith_gpu::ResourceGpuInputs {
                lith: &lith,
                boundary_type: Some(&ws.boundary_type),
                shear_field: Some(&ws.shear_field),
                flow: Some(&ws.flow_discharge),
                biome: Some(&b_cpu),
                field: &ws.field,
                rain: &ws.rainfall,
                age: &ws.age_field,
                volcanic: Some(&ws.volcanic_field),
                cu_dist: &cu_dist,
                flow_max: civ::resource_flow_max(Some(&ws.flow_discharge)),
                gw,
                sea,
            };
            cartalith_gpu::resource_potentials_grid_gpu_with(gpu, &inp).expect("gpu resources")
        };
        let raw_cpu = res_cpu_call(false);
        let raw_gpu = res_gpu_call();
        let names = civ::RESOURCE_KEYS;
        let mut worst = (0.0f64, "");
        let mut ndiff = 0usize;
        for (k, (c, g)) in res_fields(&raw_cpu).iter().zip(raw_gpu.iter()).enumerate() {
            let d = max_abs(c, g);
            ndiff += differing(c, g);
            if d > worst.0 {
                worst = (d, names[k]);
            }
        }
        println!("resources kernel  max|d| {:.3e} (worst field {})  cells differing (all 15) {ndiff}", worst.0, worst.1);

        let (res_cpu, t0, t1, t2) = timed(|| res_cpu_call(true));
        let rc = (t0, t1, t2);
        let (res_gpu, t0, t1, t2) = timed(|| civ::finish_resource_potentials(res_gpu_call(), &ws.field, sea, true, false));
        let rg = (t0, t1, t2);
        let mut sc_worst = 0.0f64;
        let mut sc_flip = 0usize;
        for (c, g) in res_fields(&res_cpu).iter().zip(res_fields(&res_gpu)) {
            sc_worst = sc_worst.max(max_abs(c, g));
            sc_flip += c.iter().zip(g).filter(|(a, b)| (**a == 0.0) != (**b == 0.0)).count();
        }
        println!(
            "resources +scarcity  max|d| {sc_worst:.3e}  cells kept/cut differently {sc_flip}   CPU {}   GPU(+CPU pre/post) {}",
            fmt(rc),
            fmt(rg)
        );

        // -- suitability, production context
        let raw_slope = civ::build_raw_slope_field(&ws.field, gw, gh, world);
        let corridors = civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
        let landmass = civ::build_landmass_quality(&ws.field, Some(&k_cpu), gw, gh, sea, world);
        let coast_reach = civ::build_coast_reach(
            &cartalith_terrain::vector::trace_coastline(&ws.field, gw, gh, sea),
            &wb.classification,
            gw,
            gh,
        );
        let flood = civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
        let (river_order, river_polys) = civ::fresh_river_network(
            &ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, p.map_width_km, ws.integrated_drainage,
        );
        let river_reach = civ::build_river_reach(&river_polys, &river_order, gw, gh);

        let suit_cpu_with = |k: &[f32], res: &civ::ResourcePotentials| {
            let ctx = civ::SuitabilityCtx {
                water_bodies: Some(&wb.classification),
                corridor: Some(&corridors),
                landmass: Some(&landmass.quality),
                flow: Some(&ws.flow_discharge),
                river_reach: Some(&river_reach),
                coast_reach: Some(&coast_reach),
                resources: Some(res),
                rain: Some(&ws.rainfall),
                flood: Some(&flood),
                slope_raw: Some(&raw_slope),
                flow_thresh,
            };
            civ::build_settlement_suitability(&soil, &water, k, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx))
        };
        let suit_gpu_with = |k: &[f32], res: &civ::ResourcePotentials| {
            let inp = cartalith_gpu::SuitabilityGpuInputs {
                soil: &soil,
                water: &water,
                carrying_cap: k,
                field: &ws.field,
                slope_n: &soil_slope,
                water_bodies: &wb.classification,
                corridor: &corridors,
                landmass: &landmass.quality,
                flow: &ws.flow_discharge,
                river_reach: &river_reach,
                coast_reach: &coast_reach,
                resources: suit_res(res),
                rain: &ws.rainfall,
                flood: &flood,
                slope_raw: &raw_slope,
                flow_thresh,
                gw,
                gh,
                sea,
            };
            cartalith_gpu::settlement_suitability_grid_gpu_with(gpu, &inp, &weights()).expect("gpu suitability")
        };
        let (s_cpu, t0, t1, t2) = timed(|| suit_cpu_with(&k_cpu, &res_cpu));
        let sc = (t0, t1, t2);
        let (s_gpu, t0, t1, t2) = timed(|| suit_gpu_with(&k_cpu, &res_cpu));
        let sg = (t0, t1, t2);
        let s_again = suit_gpu_with(&k_cpu, &res_cpu);
        println!(
            "suitability (same inputs)  max|d| {:.3e}  cells differing {}  gpu-vs-gpu differing {}   CPU {}   GPU {}",
            max_abs(&s_cpu, &s_gpu),
            differing(&s_cpu, &s_gpu),
            differing(&s_gpu, &s_again),
            fmt(sc),
            fmt(sg)
        );

        // -- the whole GPU chain against the whole CPU chain, and what it places
        let k_gpu = cartalith_gpu::carrying_capacity_grid_gpu_with(gpu, &soil, &water, &b_gpu, &ws.temperature, &ws.field, sea, 0.0, None).unwrap();
        let s_chain = suit_gpu_with(&k_gpu, &res_gpu);
        println!("suitability (full GPU chain)  max|d| {:.3e}", max_abs(&s_cpu, &s_chain));
        let supp_r = (gw as f64 / p.civ.seed_suppress_div.max(1.0)).floor().max(6.0);
        let place = |s: &[f32]| {
            let seeds = civ::find_settlement_seeds(s, gw, gh, p.civ.seed_thresh, supp_r);
            let pl = civ::place_settlements_with_counts(
                &seeds, s, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea, world, p.civ.factions.max(1),
                &flood, &ws.flow_discharge, flow_thresh, p.map_width_km, None,
            );
            (seeds.len(), pl.iter().map(|q| (q.x, q.y, q.kind, q.faction)).collect::<Vec<_>>())
        };
        let (ns_c, pl_c) = place(&s_cpu);
        let (ns_g, pl_g) = place(&s_chain);
        let same = pl_c == pl_g;
        let moved = pl_c.iter().filter(|q| !pl_g.contains(q)).count();
        println!(
            "placement  seeds CPU {ns_c} / GPU chain {ns_g}   settlements CPU {} / GPU chain {}   identical: {same}   CPU settlements absent from GPU list: {moved}",
            pl_c.len(),
            pl_g.len()
        );
    }
}
