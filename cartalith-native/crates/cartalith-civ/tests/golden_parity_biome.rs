//! Golden-parity tests for cartalith-civ's biome classification
//! (`classifyBiome`/`buildBiomeRaster`, reference HTML lines 5736/6798)
//! against the real reference engine. `PHASE2_SCOPE.md` milestone 3.
//! Generated from a Node `vm` extraction run (harness transient, not
//! checked in) that calls the reference's own `buildBiomeRaster()`
//! directly (not a hand-composed reimplementation), so the extraction
//! exercises the exact same `classifyBiome`+`currentWaterBodies()`
//! composition production code uses.
//!
//! Cases reuse `golden_parity_waterbodies.rs`'s exact fixture configs
//! (gw/gh/seed/world/w_iters=12). Cross-checked two ways before trusting
//! this data: (1) this harness's own `field[0..5]` matched
//! `golden_parity_waterbodies.rs`'s `expected_fill[0..5]` exactly for
//! both cases (same harness-seeding fix that milestone required:
//! `state.tect.seed`, not `state.seed`); (2) each case's biome category
//! counts sum exactly to that same file's already-verified ocean/lake/land
//! counts (case 0: 75 ocean + 79 land; case 1, pre-Ruling-Q: 13 ocean + 52
//! lake + 127 land, and post-Ruling-Q, matching `expected_biome` below: 14
//! ocean + 62 lake + 116 land) -- a biome raster with a real
//! classification bug would not reproduce those totals by coincidence.
//!
//! Both `classifyBiome` and `buildBiomeRaster`'s output are categorical
//! (`Uint8`) -- bit-exact match required.
//!
//! **`case_1_world_wrap`'s `expected_biome` was re-baselined 2026-09-21,
//! `LARGE_ITEM_RULINGS.md`'s Ruling Q.** `build_water_bodies` moved to a
//! topology-primary ocean/lake rule (`golden_parity_waterbodies.rs`'s own
//! header has the full account) -- a deliberate divergence from the
//! reference for classification only, so `expected_biome`'s
//! ocean(13)/lake(2) cells move wherever `classification` moved and are no
//! longer a reference-parity assertion on those cells specifically.
//! `case_0_region`'s water-body classification is unchanged, so its biome
//! raster needed no update. `case_1_world_wrap`'s new value is
//! `build_biome_raster`'s own actual output on the new classification,
//! captured 2026-09-21.

#[test]
fn biome_raster_case_0_region() {
    // case 0: region: gw=14 gh=11 seed=24601 world=false.
    let expected_biome: Vec<u8> = vec![
        1, 1, 2, 2, 2, 3, 6, 3, 6, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3, 6, 6, 6, 6, 0, 0, 0, 0, 0, 2, 3, 3, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 3, 3,
        6, 6, 6, 6, 0, 0, 0, 6, 0, 6, 6, 0, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 6, 6, 0, 6, 6, 6, 6, 6, 6, 0, 0, 12, 0, 0, 0, 0, 0, 6, 5, 5, 5,
        5, 6, 0, 0, 12, 0, 0, 0, 0, 0, 5, 5, 5, 11, 12, 12, 12, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 11, 12, 0, 0, 12, 0, 0, 0, 0, 0, 0, 5, 5, 11,
        11, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 11, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let wb = cartalith_civ::build_water_bodies(&ws.field, 14, 11, ws.sea_level, false, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    assert_eq!(biome, expected_biome);
}

#[test]
fn biome_raster_case_1_world_wrap() {
    // case 1: world_wrap: gw=16 gh=12 seed=314159 world=true. RE-BASELINED
    // 2026-09-21 (Ruling Q, see this file's own header).
    let expected_biome: Vec<u8> = vec![
        0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 13, 13, 1, 1, 13, 1, 1, 1, 1, 1, 1, 13, 2, 2, 13, 13, 13, 13, 2, 2,
        3, 13, 2, 3, 2, 2, 13, 13, 6, 13, 13, 13, 13, 13, 6, 6, 6, 6, 6, 6, 6, 13, 13, 6, 6, 13, 13, 13, 13, 13, 6, 6, 13, 13, 6, 12, 6, 6,
        6, 6, 12, 13, 13, 13, 12, 13, 13, 12, 13, 12, 12, 12, 13, 6, 6, 13, 12, 12, 13, 6, 13, 13, 13, 13, 13, 13, 6, 13, 13, 12, 13, 13,
        13, 6, 6, 6, 13, 13, 13, 13, 13, 13, 6, 13, 6, 6, 13, 13, 6, 6, 6, 6, 6, 6, 13, 13, 3, 2, 2, 13, 6, 6, 13, 13, 1, 3, 13, 3, 3, 3, 2,
        1, 3, 1, 1, 2, 0, 1, 3, 1, 1, 1, 13, 13, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1,
    ];

    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let wb = cartalith_civ::build_water_bodies(&ws.field, 16, 12, ws.sea_level, true, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    assert_eq!(biome, expected_biome);
}
