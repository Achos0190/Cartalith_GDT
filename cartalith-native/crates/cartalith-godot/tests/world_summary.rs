//! LLM-8's `get_world_summary` figures against an independent recount
//! (`LLM_ASSISTANT_SCOPE.md` s.5b.5 and the LLM-8 bar: "`get_world_summary` figures equal an
//! independent recount from the same field arrays on 3 seeds").
//!
//! `world_summary.rs` is godot-free and pulled in by `#[path]` (the `tests/params_mapping.rs`
//! pattern). The recount below restates the definitions with different code: an integer ceil for
//! the rank, a metres-first sort, a reversed summation order, so a shared bug in one formula
//! cannot hide in both.
#![allow(dead_code)]

#[path = "../src/world_summary.rs"]
mod world_summary;

use cartalith_engine::{WorldParams, generate_terrain};
use world_summary::{PERCENTILES, summarise};

/// One generated world, CPU only (the GPU path is principled-equivalent, not bit-exact, and this
/// is arithmetic, not shading). 128 x 80 keeps the test fast; the three seeds differ in land
/// fraction (asserted below, so the fixture cannot silently become three copies of one world).
fn world(seed: i32) -> (cartalith_engine::WorldState, usize, usize, f64) {
    let (gw, gh) = (128usize, 80usize);
    let mut p = WorldParams::defaults(gw, gh, seed);
    p.use_gpu = false;
    let peak = p.peak_m;
    (generate_terrain(&p), gw, gh, peak)
}

/// Independent recount: (land, fraction, percentile metres, max metres, rain mean, temp mean).
fn recount(field: &[f32], temp: &[f32], rain: &[f32], sea: f64, peak: f64, gw: usize, gh: usize) -> (usize, f64, Vec<f64>, f64, f64, f64) {
    // Metres first, then sort the metres (the module sorts heights, then converts).
    let mut metres: Vec<f64> = Vec::new();
    let (mut tsum, mut rsum) = (0f64, 0f64);
    // Rows last-to-first and columns last-to-first: a different summation order from the module.
    for y in (0..gh).rev() {
        for x in (0..gw).rev() {
            let i = y * gw + x;
            if (field[i] as f64) >= sea {
                metres.push((field[i] as f64 - sea) / (1.0 - sea) * peak);
                tsum += temp[i] as f64;
                rsum += rain[i] as f64;
            }
        }
    }
    metres.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = metres.len();
    let ps = PERCENTILES
        .iter()
        .map(|&p| {
            let rank = (p as usize * n).div_ceil(100).max(1); // integer ceil, not f64::ceil
            metres[rank - 1]
        })
        .collect();
    (n, n as f64 / (gw * gh) as f64, ps, metres[n - 1], rsum / n as f64, tsum / n as f64)
}

/// Protects the LLM-8 bar on three real worlds: land count, fraction, all five percentiles, the
/// maximum and both means equal the recount. The fixture is asserted to have both land and sea
/// and to differ between seeds (a test that passes on empty worlds proves nothing). MUTATION:
/// change `<` to `<=` in the module, a percentile's rank rule, or the metres formula: red.
#[test]
fn summary_equals_an_independent_recount_on_three_seeds() {
    let mut fractions = Vec::new();
    for seed in [9137, 24601, 71077345] {
        let (ws, gw, gh, peak) = world(seed);
        let s = summarise(&ws.field, &ws.temperature, &ws.rainfall, ws.sea_level, peak, gw, gh).expect("a generated world has a summary");
        let (n, frac, ps, max, rain, temp) = recount(&ws.field, &ws.temperature, &ws.rainfall, ws.sea_level, peak, gw, gh);
        assert!(n > 100 && n < gw * gh, "seed {seed}: the fixture must have both land and sea, got {n}");
        assert_eq!(s.land_cells, n, "seed {seed}");
        assert!((s.land_fraction - frac).abs() < 1e-12, "seed {seed}");
        let got = s.elevation_m.as_ref().unwrap();
        for (k, (p, m)) in got.iter().enumerate() {
            assert_eq!(*p, PERCENTILES[k]);
            assert!((m - ps[k]).abs() < 1e-9, "seed {seed} p{p}: {m} vs {}", ps[k]);
        }
        assert!((s.elevation_max_m.unwrap() - max).abs() < 1e-9, "seed {seed}");
        assert!((s.land_rainfall_mean.unwrap() - rain).abs() < 1e-9, "seed {seed}");
        assert!((s.land_temperature_mean_c.unwrap() - temp).abs() < 1e-9, "seed {seed}");
        assert_eq!((s.grid_width, s.grid_height), (gw, gh));
        assert!(got.windows(2).all(|w| w[0].1 <= w[1].1) && got[4].1 <= s.elevation_max_m.unwrap());
        fractions.push(frac);
    }
    assert!(fractions.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-6), "three seeds, three different worlds: {fractions:?}");
}

/// Protects the definitions on a hand-made grid whose answer is known without running anything: a
/// cell exactly at sea level IS land (`>=`), NaN is not land, ranks are nearest-rank (ten land
/// cells give p05 = 1st, p25 = 3rd, p50 = 5th, p75 = 8th, p95 = 10th, not interpolated values),
/// and metres are `(h - sea) / (1 - sea) * peak`. MUTATION: `>=` to `>`, or interpolate.
#[test]
fn definitions_on_a_known_grid() {
    // sea 0.5, peak 1000: h = 0.5 + k * 0.05 for k = 0..9 gives metres k * 100 (k = 0 sits ON sea level).
    let mut field: Vec<f32> = (0..10).map(|k| 0.5 + k as f32 * 0.05).collect();
    field.push(0.2); // sea
    field.push(f32::NAN); // not a cell
    let n = field.len();
    let t: Vec<f32> = vec![10.0; n];
    let r: Vec<f32> = vec![0.25; n];
    let s = summarise(&field, &t, &r, 0.5, 1000.0, n, 1).unwrap();
    assert_eq!(s.land_cells, 10, "the cell exactly at sea level is land; NaN and 0.2 are not");
    assert!((s.land_fraction - 10.0 / 12.0).abs() < 1e-12);
    let m: Vec<f64> = s.elevation_m.unwrap().iter().map(|x| x.1).collect();
    // ranks over 10 land cells: ceil(0.5)=1, ceil(2.5)=3, 5, ceil(7.5)=8, ceil(9.5)=10 -> k = 0, 2, 4, 7, 9
    let want = [0.0, 200.0, 400.0, 700.0, 900.0];
    for (g, w) in m.iter().zip(want) {
        assert!((g - w).abs() < 1e-3, "{m:?}"); // f32 steps of 0.05 are not exact
    }
    assert!((s.elevation_max_m.unwrap() - 900.0).abs() < 1e-3);
    assert_eq!(s.land_rainfall_mean, Some(0.25));
    assert_eq!(s.land_temperature_mean_c, Some(10.0));
}

/// Protects absent-means-absent: an all-sea world has a real land fraction of 0 and NO percentile,
/// maximum or mean; a raster of the wrong length drops its mean only; a field of the wrong length
/// is not a world at all.
#[test]
fn no_land_and_wrong_lengths_omit_instead_of_zero() {
    let field = vec![0.1f32; 12];
    let s = summarise(&field, &[1.0; 12], &[0.5; 12], 0.5, 4000.0, 4, 3).unwrap();
    assert_eq!(s.land_cells, 0);
    assert_eq!(s.land_fraction, 0.0);
    assert!(s.elevation_m.is_none() && s.elevation_max_m.is_none() && s.land_rainfall_mean.is_none() && s.land_temperature_mean_c.is_none());
    let land = vec![0.9f32; 12];
    let s = summarise(&land, &[1.0; 12], &[0.5; 3], 0.5, 4000.0, 4, 3).unwrap();
    assert!(s.land_rainfall_mean.is_none(), "short rainfall raster: no mean, not a mean of a prefix");
    assert_eq!(s.land_temperature_mean_c, Some(1.0));
    assert!(summarise(&land, &[], &[], 0.5, 4000.0, 5, 3).is_none(), "a field that is not gw x gh is not a world");
    assert!(summarise(&[], &[], &[], 0.5, 4000.0, 0, 0).is_none());
    assert!(summarise(&land, &[], &[], f64::NAN, 4000.0, 4, 3).is_none());
}

/// Protects the sea-level-at-1 guard: `sea == 1` must not divide by zero (the engine's
/// `elevation_m` uses `1e-6`), so such a world yields finite metres.
#[test]
fn sea_level_one_is_finite() {
    let s = summarise(&[1.0, 1.0], &[0.0, 0.0], &[0.0, 0.0], 1.0, 4000.0, 2, 1).unwrap();
    assert!(s.elevation_m.unwrap().iter().all(|x| x.1.is_finite()));
}

/// Protects the two copies of the percentile list: this module's must equal `cartalith-llm`'s (the
/// tool reports exactly the percentiles the summary computes).
#[test]
fn percentile_lists_agree() {
    assert_eq!(PERCENTILES, cartalith_llm::query::SUMMARY_PERCENTILES);
}
