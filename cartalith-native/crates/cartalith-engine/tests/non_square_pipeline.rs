//! The full pipeline at non-square grid dimensions.
//!
//! `WorldParams` has always carried independent `gw`/`gh`, and the
//! golden-parity fixtures this crate and its subsystems are verified against
//! are themselves non-square (`golden_parity_pipeline.rs` runs 24×18 and
//! 20×14; `golden_parity_carve.rs` 14×11 and 16×12) — so JS parity at
//! non-square dimensions is already established and is *not* what this file
//! re-checks.
//!
//! What it checks is the part those fixtures cannot: that the pipeline holds
//! up at **large, strongly asymmetric** shapes, in both orientations, at a
//! non-power-of-two size, and in world (X-wrapping) mode — the sizes at which
//! resolution-derived radii (`smooth_sea_h`'s `gw/200`, `build_weather_grid`'s
//! `min(gw,240)` coarse grid, the blur radii derived from `blur_r`) become
//! large relative to the *shorter* axis, which is exactly where a
//! square-assuming bound would go out of range.
//!
//! Deliberately not a golden test: there is no captured JS run at these
//! dimensions to diff against, and inventing one would not add information
//! that the existing non-square fixtures do not already carry.

use cartalith_engine::{WorldParams, generate_terrain};

fn check(gw: usize, gh: usize, seed: i32, world: bool) {
    let mut p = WorldParams::defaults(gw, gh, seed);
    p.use_gpu = false;
    p.world = world;
    let ws = generate_terrain(&p);

    let n = gw * gh;
    assert_eq!(ws.field.len(), n, "{gw}x{gh}: height field");
    assert_eq!(ws.temperature.len(), n, "{gw}x{gh}: temperature");
    assert_eq!(ws.rainfall.len(), n, "{gw}x{gh}: rainfall");
    assert_eq!(ws.flow_discharge.len(), n, "{gw}x{gh}: flow");
    assert_eq!(ws.age_field.len(), n, "{gw}x{gh}: age");
    assert_eq!(ws.resistance_field.len(), n, "{gw}x{gh}: resistance");
    assert_eq!(ws.volcanic_field.len(), n, "{gw}x{gh}: volcanic");
    assert_eq!(ws.impact_field.len(), n, "{gw}x{gh}: impacts");
    if let Some(ch) = ws.channels.as_ref() {
        assert_eq!(ch.chan.len(), n, "{gw}x{gh}: channel mask");
    }

    assert!(ws.field.iter().all(|v| v.is_finite()), "{gw}x{gh}: NaN/Inf in the height field");
    assert!(ws.field.iter().all(|&v| (0.0..=1.0).contains(&v)), "{gw}x{gh}: height field left [0,1]");
    assert!(ws.temperature.iter().all(|v| v.is_finite()), "{gw}x{gh}: NaN/Inf in temperature");
    assert!(ws.rainfall.iter().all(|v| v.is_finite()), "{gw}x{gh}: NaN/Inf in rainfall");

    // A world that is all sea or all land at these defaults would mean the
    // shape broke normalization or the sea-level anchor, not that the seed
    // was unlucky.
    let land = ws.field.iter().filter(|&&v| (v as f64) >= ws.sea_level).count();
    assert!(land > n / 100 && land < n * 99 / 100, "{gw}x{gh}: degenerate land fraction {land}/{n}");
}

#[test]
fn two_to_one_wide() {
    check(256, 128, 12345, false);
}

#[test]
fn one_to_two_tall() {
    check(128, 256, 12345, false);
}

/// Neither dimension a power of two, and neither a multiple of the other.
#[test]
fn non_power_of_two_five_to_three() {
    check(250, 150, 4242, false);
}

/// The reference app's own region shape (`gridH(gw) = round(gw*0.64)`,
/// reference HTML line 5049) at a real working resolution.
#[test]
fn the_reference_apps_own_region_aspect() {
    check(256, 164, 24601, false);
}

/// The reference app's own world shape — 2:1 equirectangular, with the
/// toroidal X wrap active.
#[test]
fn the_reference_apps_own_world_aspect() {
    check(256, 128, 987654, true);
}

/// A shape asymmetric enough that resolution-derived radii (`gw/200` blur
/// windows, `min(gw,240)` weather grid) exceed the shorter axis outright.
#[test]
fn extremely_wide_shorter_than_its_own_blur_radii() {
    check(512, 32, 7, false);
}

/// World Structure on, non-square: the continentality field, the derived
/// plate/volcano counts and the land-fraction sea-level re-anchor all run
/// over the same grid.
#[test]
fn world_structure_archipelago_two_to_one() {
    let (gw, gh) = (192usize, 96usize);
    let mut p = WorldParams::defaults(gw, gh, 12345);
    p.use_gpu = false;
    p.world_structure.enabled = true;
    p.world_structure.continentality = 0.15;
    p.world_structure.fragmentation = 0.90;
    p.world_structure.tectonic_energy = 0.80;
    p.world_structure.ocean_depth = 0.30;
    p.world_structure.hotspot_density = 0.50;
    let ws = generate_terrain(&p);
    assert_eq!(ws.field.len(), gw * gh);
    assert!(ws.field.iter().all(|v| v.is_finite()));
}
