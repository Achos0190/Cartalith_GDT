//! **Ruling ET: the lake water balance in the generation path**
//! (`cartalith_civ::lake_balance`, `render::water_bodies_floored_balanced`).
//!
//! Protects, on a real generated world (seed 483920 at 512 x 328, the app's
//! parameters):
//! * `balance: None` is exactly the plain `water_bodies_floored_keep`, so a
//!   world with the switch off (every golden, every old save) is untouched;
//! * with the balance, every lake cell the production funnel returns lies in
//!   a pool the shared `lake_pools` holds (one implementation, not two), and
//!   every cell `lake_pools` holds is lake there;
//! * the fixture reaches the code: the switch changes the classification;
//! * no cell outside a pool, and no ocean or below-sea cell, is changed.
//!
//! The grid is under the lake-area floor's 1 006 848 cells, so Ruling CC's floor
//! is a no-op and the comparison is exact.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use cartalith_civ::lake_balance::lake_pools;
use cartalith_engine::generate_terrain;
use render::LakeBalanceInputs;

#[test]
fn production_funnel_equals_the_shared_balance() {
    let (gw, gh) = (512usize, 328usize);
    let mut p = params::defaults();
    assert!(p.lake_balance, "the app default is ON (Ruling ET)");
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = 483_920;
    p.map_width_km = 1_200.0;
    p.use_gpu = false;
    let ws = generate_terrain(&p);
    let sea = ws.sea_level;

    let plain = render::water_bodies_floored_keep(&ws.field, &ws.rainfall, gw, gh, sea, p.world, None, None);
    let off = render::water_bodies_floored_balanced(&ws.field, &ws.rainfall, gw, gh, sea, p.world, None, None, None);
    assert_eq!(plain.classification, off.classification, "None is the plain path");
    assert_eq!(plain.fill_level, off.fill_level);

    let inputs = LakeBalanceInputs::when(true, Some(&ws.flow_discharge), &ws.temperature);
    assert!(inputs.is_some());
    assert!(LakeBalanceInputs::when(false, Some(&ws.flow_discharge), &ws.temperature).is_none(), "switch off carries nothing");
    assert!(LakeBalanceInputs::when(true, None, &ws.temperature).is_none(), "no flow field, no balance");
    let on = render::water_bodies_floored_balanced(&ws.field, &ws.rainfall, gw, gh, sea, p.world, None, None, inputs);

    let flood = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, p.world, Some(&ws.rainfall));
    let pools = lake_pools(&ws.field, &ws.flow_discharge, &ws.temperature, &flood.fill_level, gw, gh, sea, p.world, 1.0);
    assert!(!pools.is_empty(), "the fixture world has pools");
    let mut in_pool = vec![false; gw * gh];
    let mut held = vec![false; gw * gh];
    for q in &pools {
        for &i in &q.cells {
            in_pool[i] = true;
        }
        for &i in &q.balanced {
            held[i] = true;
        }
    }
    for i in 0..gw * gh {
        if in_pool[i] {
            assert_eq!(on.classification[i] == 2, held[i], "pool cell {i}: lake exactly when the shared balance holds it");
        } else {
            assert_eq!(on.classification[i], plain.classification[i], "cell {i} outside every pool is untouched");
        }
    }
    assert_ne!(on.classification, plain.classification, "the switch must move this world's lakes");
}

/// **The re-measurement** (Ruling ET): the production funnel, switch off then
/// on, on the owner world and seeds 483920 and 912345 at the harness's sizes.
/// Prints lakes per world (pools holding a lake cell, the harness's own unit),
/// lake cells, cells dried and flooded, and writes one before-over-after PNG
/// per world (1 px a cell; dark blue ocean, teal lake, orange a playa floor
/// (lake before, land after), green new lake, grey land by height).
///
/// ```text
/// LAKEBAL_DIR=<dir> cargo test --release -p cartalith-godot --test lake_balance_generation -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore]
fn lake_balance_remeasure() {
    let out = std::env::var("LAKEBAL_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir().join("lakebal_gen"));
    std::fs::create_dir_all(&out).expect("create output dir");
    for &(seed, gw, gh, km, density, label) in [(246_371, 2048usize, 1311usize, 40_075.0, 1.55, "owner"), (483_920, 1024, 656, 1_200.0, 1.0, "483920"), (912_345, 1024, 656, 1_200.0, 1.0, "912345")].iter() {
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = km;
        p.river_density = density;
        p.use_gpu = false;
        let ws = generate_terrain(&p);
        let sea = ws.sea_level;
        let before = render::water_bodies_floored_keep(&ws.field, &ws.rainfall, gw, gh, sea, p.world, None, None);
        let after = render::water_bodies_floored_balanced(&ws.field, &ws.rainfall, gw, gh, sea, p.world, None, None, LakeBalanceInputs::when(p.lake_balance, Some(&ws.flow_discharge), &ws.temperature));
        let flood = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, p.world, Some(&ws.rainfall));
        let pools = lake_pools(&ws.field, &ws.flow_discharge, &ws.temperature, &flood.fill_level, gw, gh, sea, p.world, 1.0);
        let lakes = |c: &[u8]| pools.iter().filter(|q| q.cells.iter().any(|&i| c[i] == 2)).count();
        let cells = |c: &[u8], f: &dyn Fn(usize) -> bool| (0..gw * gh).filter(|&i| c[i] == 2 && f(i) && ws.field[i] as f64 > sea).count();
        // The harness's own rows: before Ruling CC's lake-area floor (active from 1 006 848 cells), which is why
        // the owner world's floored count above is lower than the harness's 322 -> 356.
        let mut unfloored = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, p.world, Some(&ws.rainfall));
        let before_u = lakes(&unfloored.classification);
        cartalith_civ::lake_balance::apply_lake_balance(&mut unfloored, &ws.field, &ws.flow_discharge, &ws.temperature, gw, gh, sea, p.world);
        println!("  before the Ruling CC floor (the harness's rows): lakes {before_u} -> {}", lakes(&unfloored.classification));
        let dried = (0..gw * gh).filter(|&i| before.classification[i] == 2 && after.classification[i] != 2).count();
        let flooded = (0..gw * gh).filter(|&i| before.classification[i] != 2 && after.classification[i] == 2).count();
        println!(
            "{label}: seed {seed} {gw}x{gh}: lakes (pools with a lake cell) {} -> {}; above-sea lake cells {} -> {}; dried {dried}, flooded {flooded}",
            lakes(&before.classification),
            lakes(&after.classification),
            cells(&before.classification, &|_| true),
            cells(&after.classification, &|_| true)
        );
        let top = ws.field.iter().copied().fold(f32::MIN, f32::max) as f64;
        let mut rgb = vec![0u8; gw * gh * 2 * 3];
        for (half, c) in [(0usize, &before.classification), (1usize, &after.classification)] {
            for i in 0..gw * gh {
                let g = (90.0 + 150.0 * ((ws.field[i] as f64 - sea) / (top - sea)).clamp(0.0, 1.0)) as u8;
                let px: [u8; 3] = if c[i] == 1 || (c[i] == 2 && ws.field[i] as f64 <= sea) {
                    [28, 48, 82]
                } else if c[i] == 2 {
                    if half == 1 && before.classification[i] != 2 { [80, 255, 60] } else { [0, 200, 190] }
                } else if half == 1 && before.classification[i] == 2 {
                    [255, 140, 20]
                } else {
                    [g, g, g]
                };
                let o = (half * gw * gh + i) * 3;
                rgb[o..o + 3].copy_from_slice(&px);
            }
        }
        let path = out.join(format!("lake_balance_before_after_{label}.png"));
        let file = std::fs::File::create(&path).expect("create png");
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), gw as u32, (gh * 2) as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().expect("png header").write_image_data(&rgb).expect("png data");
        println!("  frame {}", path.display());
    }
}
