#![allow(clippy::excessive_precision)]
//! Golden-parity tests for `TIMELINE_SCOPE.md` milestone 1's population-
//! ceiling chain (`cartalith_civ::timeline`): `subsistenceModeAt`/
//! `agrarianDensityKm2` (reference lines 23369-23385), `currentAgrarianDensity`
//! (23441-23460), `_civCatchmentDensityMean` (23461-23469), `_civCatchmentPop`/
//! `_civSettlementPopulation` (23484-23511), and `_civTierForPopulation`
//! (24618).
//!
//! Generated from a Node `vm.runInContext` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` (harness itself transient, not
//! checked in, per this project's convention -- see `PARITY_TESTING.md`),
//! slicing lines 23313-23512 (verified before slicing: starts at the
//! `AGRARIAN_MAX_KM2`-introducing comment block, ends exactly at
//! `_civSettlementPopulation`'s closing brace, does not spill into
//! `_civAgrarianRegionalTotal`) and lines 24614-24618 (the tier table).
//!
//! `currentCarryingCapacity`/`currentWaterAccess`/`buildBiomeRaster` were
//! stubbed to return hand-picked arrays instead of running the whole
//! terrain pipeline -- legitimate here because `_civCatchmentDensityMean`/
//! `_civCatchmentPop`/`_civSettlementPopulation`/`agrarianDensityKm2` are
//! all "pure over the supplied per-cell field" per their own reference doc
//! comments, so feeding them a known-good input directly (rather than
//! deriving it from a full generated world) is exactly `PARITY_TESTING.md`'s
//! "one test per pipeline stage" guidance, not a shortcut around it. No
//! reference source was transcribed or reimplemented by hand; every number
//! below is the real reference's own output.
//!
//! `currentAgrarianDensity`'s own output is a `Float32Array` in the
//! reference, so the golden values are already f32-rounded; `1e-4`
//! absolute+relative tolerance (this crate's established convention) covers
//! any remaining f64-intermediate rounding-order difference.

use cartalith_civ::SettlementKind;
use cartalith_civ::timeline::{
    civ_agrarian_density_km2, civ_catchment_density_mean, civ_catchment_pop,
    civ_current_agrarian_density, civ_settlement_population, civ_subsistence_mode_at,
    civ_tier_for_population,
};

fn assert_close(actual: f64, expected: f64, label: &str) {
    const ATOL: f64 = 1e-4;
    const RTOL: f64 = 1e-4;
    let tol = ATOL + RTOL * expected.abs();
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: got {actual}, expected {expected} (diff {}, tol {tol})",
        (actual - expected).abs()
    );
}

// ---------- subsistenceModeAt / agrarianDensityKm2 ----------

#[test]
fn subsistence_mode_and_agrarian_density_match_the_reference() {
    // (k, water, biome, rain, want_mode, want_density) -- biome 7 is grass
    // (this crate's `BIOME_GRASS`), the reference's own `BIOME_KEYS` index.
    let cases: [(f64, f64, u8, f64, u8, f64); 11] = [
        (0.9, 0.9, 0, 0.9, 0, 1.8),                     // ocean -> gathering
        (0.9, 0.9, 1, 0.9, 0, 1.8),                     // ice -> gathering
        (0.9, 0.9, 2, 0.9, 0, 1.8),                     // tundra -> gathering
        (0.9, 0.9, 9, 0.9, 0, 1.8),                     // desert -> gathering
        (0.45, 0.35, 7, 0.25, 3, 72.0),                 // annual cultivation, exact boundary
        (0.45, 0.35, 7, 0.249999, 2, 17.55),            // just under rain -> short fallow
        (0.28, 0.199999, 7, 0.0, 1, 9.520000000000001), // just under short-fallow water -> bush fallow
        (0.099999, 0.0, 7, 0.0, 0, 0.199998),           // just under bush-fallow k -> gathering
        (0.5, 0.9, 7, 0.9, 3, 80.0),                    // mid annual cultivation
        (5.0, 0.9, 7, 0.9, 3, 160.0),                   // k>1 clamps to 1
        (f64::NAN, 0.9, 7, 0.9, 0, 0.0),                // NaN k -> falsy -> 0
    ];
    for (k, w, b, r, want_mode, want_density) in cases {
        let mode = civ_subsistence_mode_at(k, w, b, r);
        assert_eq!(mode, want_mode, "mode(k={k}, w={w}, b={b}, r={r})");
        let density = civ_agrarian_density_km2(k, w, b, r);
        assert_close(
            density,
            want_density,
            &format!("density(k={k}, w={w}, b={b}, r={r})"),
        );
    }
}

// ---------- currentAgrarianDensity ----------

#[test]
fn current_agrarian_density_matches_the_reference_on_a_mixed_land_sea_fixture() {
    let k = [0.9f32, 0.2, 0.9];
    let water = [0.9f32, 0.9, 0.9];
    let biome = [7u8, 7, 7];
    let rain = [0.9f32, 0.9, 0.9];
    let field = [0.6f32, 0.6, 0.3]; // cell 2 below sea (0.42)
    let sea = 0.42;
    let out = civ_current_agrarian_density(&k, &water, Some(&biome), &rain, &field, sea);
    let expected = [210.07957458496094f32, 9.920424461364746, 0.0];
    for i in 0..3 {
        assert_close(out[i] as f64, expected[i] as f64, &format!("dens[{i}]"));
    }
}

#[test]
fn current_agrarian_density_falls_back_to_norm_one_when_all_cells_are_sea() {
    let k = [0.9f32];
    let water = [0.9f32];
    let biome = [7u8];
    let rain = [0.9f32];
    let field = [0.1f32]; // below sea 0.42
    let out = civ_current_agrarian_density(&k, &water, Some(&biome), &rain, &field, 0.42);
    assert_eq!(out, vec![0.0f32]);
}

// ---------- _civCatchmentDensityMean ----------

#[test]
fn catchment_density_mean_matches_the_reference_with_a_sea_cell_excluded() {
    let gw = 5usize;
    let gh = 5usize;
    let mut field = vec![0.6f32; gw * gh];
    field[2 * gw + 3] = 0.1; // sea cell at (x=3, y=2)
    let dens: Vec<f32> = (0..gw * gh).map(|i| i as f32).collect();
    let mean = civ_catchment_density_mean(2, 2, 1, &dens, &field, gw, gh, 0.42, false);
    assert_close(mean, 11.75, "catchment_density_mean");
}

#[test]
fn catchment_density_mean_wrap_vs_no_wrap_matches_the_reference() {
    let gw = 4usize;
    let gh = 3usize;
    let field = vec![0.6f32; gw * gh];
    let dens: Vec<f32> = (0..gw * gh).map(|i| i as f32).collect();
    let mean_wrap = civ_catchment_density_mean(0, 1, 1, &dens, &field, gw, gh, 0.42, true);
    let mean_no_wrap = civ_catchment_density_mean(0, 1, 1, &dens, &field, gw, gh, 0.42, false);
    assert_close(mean_wrap, 4.8, "mean_wrap");
    assert_close(mean_no_wrap, 4.25, "mean_no_wrap");
}

#[test]
fn catchment_density_mean_is_zero_when_every_cell_in_range_is_sea() {
    let gw = 3usize;
    let gh = 3usize;
    let field = vec![0.1f32; gw * gh];
    let dens = vec![5.0f32; gw * gh];
    let mean = civ_catchment_density_mean(1, 1, 1, &dens, &field, gw, gh, 0.42, false);
    assert_eq!(mean, 0.0);
}

// ---------- _civCatchmentPop / _civSettlementPopulation ----------

/// All five settlement kinds, on a 10x10 all-land map with a uniform
/// density-10 field (map_width_km=800, so `cellKm=80` and every kind's
/// catchment radius rounds down to the minimum 1 cell -- the reference's
/// own `_civCatchmentRadiusCells` floor -- which is exactly why the mean
/// density stays a flat 10 for every kind and `catchmentPop` isolates the
/// per-tier catchment-area scaling).
#[test]
fn catchment_pop_and_settlement_population_match_the_reference_across_all_kinds() {
    let gw = 10usize;
    let gh = 10usize;
    let field = vec![0.6f32; gw * gh];
    let dens = vec![10.0f32; gw * gh];
    let map_width_km = 800.0;
    let sea = 0.42;
    let cases: [(SettlementKind, f64, f64, f64); 5] = [
        (SettlementKind::Hamlet, 60.0, 39.0, 48.75),
        (SettlementKind::Village, 250.0, 137.5, 206.25),
        (SettlementKind::Town, 1500.0, 240.0, 504.0),
        (SettlementKind::City, 8000.0, 960.0, 2592.0),
        (SettlementKind::Capital, 14000.0, 1540.0, 4466.0),
    ];
    for (kind, want_catchment_pop, want_pop_norm_b0, want_pop_norm_b1) in cases {
        let catchment_pop =
            civ_catchment_pop(5, 5, kind, &dens, &field, gw, gh, sea, false, map_width_km);
        assert_close(
            catchment_pop,
            want_catchment_pop,
            &format!("{kind:?} catchmentPop"),
        );
        let pop0 = civ_settlement_population(
            kind,
            5,
            5,
            &dens,
            &field,
            gw,
            gh,
            sea,
            false,
            map_width_km,
            0.0,
        );
        assert_close(
            pop0,
            want_pop_norm_b0,
            &format!("{kind:?} settlePop normB=0"),
        );
        let pop1 = civ_settlement_population(
            kind,
            5,
            5,
            &dens,
            &field,
            gw,
            gh,
            sea,
            false,
            map_width_km,
            1.0,
        );
        assert_close(
            pop1,
            want_pop_norm_b1,
            &format!("{kind:?} settlePop normB=1"),
        );
    }
}

#[test]
fn settlement_population_is_zero_for_a_nan_norm_b_over_an_all_sea_map() {
    let gw = 3usize;
    let gh = 3usize;
    let field = vec![0.1f32; gw * gh];
    let dens = vec![0.0f32; gw * gh];
    let pop = civ_settlement_population(
        SettlementKind::Hamlet,
        1,
        1,
        &dens,
        &field,
        gw,
        gh,
        0.42,
        false,
        800.0,
        f64::NAN,
    );
    assert_eq!(pop, 0.0);
}

// ---------- _civTierForPopulation ----------

/// The reference's own six-tier table has a `metropolis` entry above
/// `capital` (floor 150000). This test used to stop one tier short and
/// assert `Capital` on the last two rows -- the DOCUMENTED divergence
/// `TIMELINE_SCOPE.md` §9 recorded while `SettlementKind` had no
/// `Metropolis` variant. Porting `_civSelectMetropolises` (owner decision,
/// 2026-08-20) removed that divergence, so both rows now carry the
/// reference's own answer, re-extracted rather than hand-flipped:
/// `golden_parity_metropolis_recovery.rs`'s
/// `tier_for_population_matches_the_full_six_tier_reference_table` runs all
/// thirteen boundary samples straight out of the extraction harness, and
/// this test's two rows are two of them.
#[test]
fn tier_for_population_matches_the_full_reference_table_including_metropolis() {
    let cases: [(f64, SettlementKind); 9] = [
        (0.0, SettlementKind::Hamlet),
        (149.999, SettlementKind::Hamlet),
        (150.0, SettlementKind::Village),
        (799.999, SettlementKind::Village),
        (800.0, SettlementKind::Town),
        (4999.999, SettlementKind::Town),
        (5000.0, SettlementKind::City),
        (29999.999, SettlementKind::City),
        (30000.0, SettlementKind::Capital),
    ];
    for (pop, want) in cases {
        assert_eq!(civ_tier_for_population(pop), want, "pop={pop}");
    }
    // The reference answers "metropolis" for both of these, and so does this
    // port now.
    assert_eq!(
        civ_tier_for_population(149_999.999),
        SettlementKind::Capital
    );
    assert_eq!(
        civ_tier_for_population(150_000.0),
        SettlementKind::Metropolis
    );
    assert_eq!(
        civ_tier_for_population(5_000_000.0),
        SettlementKind::Metropolis
    );
}
