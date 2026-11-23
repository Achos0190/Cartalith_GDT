//! CS-1's tests of the hazard test (Ruling HS): [`super::land_bits`], [`super::cliff_top`], [`super::fine_grid`] and
//! [`super::exposure`] at their boundaries, and [`super::setback_ctx`] on a hand-built world with an exposed and a
//! sheltered shore. Every boundary is a literal, so a changed constant or a flipped comparison turns one red.

use super::*;

fn cell() -> LandCell {
    LandCell { sea_m: 500.0, ground_m: 50.0, exposed: true, h_surge_m: H_SURGE_M, cliff_m: f64::INFINITY, flood: 0.0, wetland: false, landform: 0 }
}

#[test]
fn surge_is_low_ground_within_a_hundred_metres_of_an_exposed_sea() {
    let low = LandCell { sea_m: 100.0, ground_m: 3.99, ..cell() };
    assert_eq!(land_bits(&low), SURGE);
    assert_eq!(land_bits(&LandCell { sea_m: 100.01, ..low }), 0, "past 100 m");
    assert_eq!(land_bits(&LandCell { ground_m: 4.0, ..low }), 0, "4 m is above the band");
    assert_eq!(land_bits(&LandCell { exposed: false, ..low }), 0, "a sheltered shore has no surge band (Ruling HS)");
}

#[test]
fn cliff_marsh_floodplain_and_wetland_bits() {
    assert_eq!(land_bits(&LandCell { cliff_m: 50.0, ..cell() }), CLIFF);
    assert_eq!(land_bits(&LandCell { cliff_m: 50.5, ..cell() }), 0);
    assert_eq!(land_bits(&LandCell { flood: 0.56, ..cell() }), MARSH | FLOODPLAIN, "flood over 0.55");
    assert_eq!(land_bits(&LandCell { flood: 0.55, ..cell() }), 0, "0.55 itself is the line, not over it");
    assert_eq!(land_bits(&LandCell { wetland: true, ..cell() }), MARSH | WETLAND);
    assert_eq!(land_bits(&LandCell { landform: 6, ..cell() }), FLOODPLAIN);
    assert_eq!(land_bits(&LandCell { landform: 1, ..cell() }), 0, "a cliff class alone is no reason away from the sea");
    assert!(cliff_top(22.0, 15.0, 0, 22.0) && cliff_top(22.0, 0.0, 1, 22.0));
    assert!(!cliff_top(22.0, 14.9, 0, 22.0) && !cliff_top(22.5, 15.0, 0, 22.0));
}

#[test]
fn a_grid_of_625_metre_cells_is_fine_and_coarser_is_not() {
    assert!(fine_grid(625.0) && fine_grid(800_000.0 / 2048.0));
    assert!(!fine_grid(625.5) && !fine_grid(800_000.0 / 384.0));
}

#[test]
fn exposure_is_the_wet_share_of_the_half_disc_in_front() {
    // Water everywhere east of x = 10: the half-disc facing east from (10, 10) is all water, facing west none.
    let wet = |i: i64, _: i64| i >= 10;
    assert_eq!(exposure((10.0, 10.0), (1.0, 0.0), 3.0, &wet), Some(1.0));
    assert_eq!(exposure((10.0, 10.0), (-1.0, 0.0), 3.0, &wet), Some(0.0));
    assert_eq!(exposure((10.0, 10.0), (1.0, 0.0), 0.5, &wet), None, "no cell in a half-disc under one cell");
}

/// A 64 x 64 world of 625 m cells (40 km), sea from row 39 down (0.30 under a 0.42 sea), land flat 0.5 m above it
/// at 1 000 m a unit; the ocean classified (exposed) or not (no ocean to face: sheltered).
fn world(ocean: bool) -> (Vec<f32>, Vec<u8>) {
    let (mut field, mut wb) = (vec![0.4205f32; 64 * 64], vec![0u8; 64 * 64]);
    for y in 39..64 {
        for x in 0..64 {
            field[y * 64 + x] = 0.30;
            wb[y * 64 + x] = u8::from(ocean);
        }
    }
    (field, wb)
}

fn ctx(ocean: bool) -> SetbackCtx {
    let (field, wb) = world(ocean);
    let w = UrbanWorld {
        field: &field,
        flow: &[],
        water_bodies: &wb,
        order: None,
        river_polys: &[],
        gw: 64,
        gh: 64,
        sea_level: 0.42,
        map_width_km: 40.0,
        flow_thresh: 1.0,
        world_seed: 1,
        river_kind_from_map: false,
        map_water: None,
    };
    let f = SetbackFields { coast: true, river: true, flood: vec![0.0; 64 * 64], wetland: vec![0; 64 * 64], landform: vec![0; 64 * 64], metres_per_unit: 1000.0, h_surge_m: H_SURGE_M };
    // The town's own raster: 77 x 57 cells of 22 m, sea from row 40 (as the host's water context would say).
    let mut open = vec![0u8; 77 * 57];
    open[40 * 77..].iter_mut().for_each(|v| *v = 1);
    let water = WaterCtx { mw: 77, mh: 57, cell_m: 22.0, open_water: Some(open), ..Default::default() };
    setback_ctx(&w, &f, &water, 32.0, 37.0, 1700.0, 1250.0, true)
}

#[test]
fn the_raster_reads_the_surge_band_on_an_exposed_shore_and_none_on_a_sheltered_one() {
    let sc = ctx(true);
    assert!(sc.fine && sc.coast);
    let at = |j: usize| sc.bits[j * 77 + 38];
    assert_eq!(at(40) & SEA, SEA);
    // Row 39 is 11 m from the sea's edge, row 35 99 m, row 34 121 m.
    assert_eq!(sc.sea_m[39 * 77 + 38], 11.0);
    assert_eq!(at(35), SURGE);
    assert_eq!(at(34), 0);
    assert!((sc.ground_m[30 * 77 + 38] - 0.5).abs() < 1e-3, "flat land 0.5 m up: {}", sc.ground_m[30 * 77 + 38]);
    let sheltered = ctx(false);
    assert!(sheltered.bits.iter().all(|&b| b & SURGE == 0), "no ocean in front: no surge band");
}
