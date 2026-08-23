//! Golden-parity test for fjord masking and carving (`PARITY_TESTING.md`)
//! — reference HTML lines 3208-3238 (`LITH_COMPETENCE`, `buildFjordMask`,
//! `carveFjords`).
//!
//! `fjord_captured.json` was captured under the same Node
//! `vm.runInContext` harness `golden_parity_center.rs` documents, from a
//! real `generate()` at `gw=48 gh=32 seed=24601 world=true
//! mapWidthKm=4000`. Every input is what the real pipeline produced:
//! `lith` is the sandbox's own `currentLithology()`, `coast_d` its own
//! `chamferDist(seaMask, GW, GH)` — the exact chain `currentFjordMask()`
//! (reference line 3240) runs.
//!
//! Assertions are exact. Both functions are fixed-order `f64` arithmetic
//! stored through `f32`, with `Math.min` covered by `js_min`; there is no
//! transcendental in either, so a tolerance would only hide a real
//! mismatch.
//!
//! # Fixture shapes, and what each is for
//!
//! - `real`: the default `{}` opts. 97 of 1 536 cells carry a non-zero
//!   mask and 36 cells actually carve — asserted below, because a
//!   silently-empty golden is this project's own most-repeated failure
//!   mode.
//! - `opts`: every one of the seven `opts.x != null` branches taken at
//!   once (`coastBuffer` 3, `paleoAnomaly` 4, `reliefR` 1, `reliefMin`
//!   0.02, `reliefRange` 0.3, `overDeep` 0.3, `maskFull` 0.1). Without
//!   this, the defaults could be hard-coded and the suite would pass.
//! - `cold`: the same world with every temperature 12 °C lower, which
//!   slides the paleoclimate band onto entirely different coastline. This
//!   is the mutation test for the `7 / 6 / -2 / -22 / -12` constants: a
//!   wrong one still matches `real` on a lucky world but cannot match two
//!   worlds 12 degrees apart.
//! - `lith_sweep`: lithology forced to `i % 7`, so all seven
//!   `LITH_COMPETENCE` entries are read. The real run happens to contain
//!   all seven classes too, but not necessarily on fjord-eligible cells.

use cartalith_terrain::fjord::{CarveFjordsOpts, FjordMaskOpts, LITH_COMPETENCE, build_fjord_mask, carve_fjords};

struct Fx {
    v: serde_json::Value,
    gw: usize,
    gh: usize,
    sea: f64,
    field: Vec<f32>,
    temp: Vec<f32>,
    lith: Vec<u8>,
    coast_d: Vec<f32>,
}

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn fixture() -> Fx {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fjord_captured.json"))
        .expect("fjord_captured.json fixture should read");
    let v: serde_json::Value = serde_json::from_str(&s).expect("fixture should parse");
    let gw = v["gw"].as_u64().unwrap() as usize;
    let gh = v["gh"].as_u64().unwrap() as usize;
    let sea = v["sea"].as_f64().unwrap();
    let field = f32s(&v["field"]);
    let temp = f32s(&v["temp"]);
    let lith = v["real"]["lith"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect();
    let coast_d = f32s(&v["real"]["coast_d"]);
    assert_eq!(field.len(), gw * gh);
    Fx { v, gw, gh, sea, field, temp, lith, coast_d }
}

#[test]
fn the_lith_competence_table_matches_the_reference() {
    let f = fixture();
    let expected: Vec<f64> = f.v["lith_competence"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
    assert_eq!(LITH_COMPETENCE.to_vec(), expected);
}

#[test]
fn build_fjord_mask_matches_the_reference_at_the_defaults() {
    let f = fixture();
    let got = build_fjord_mask(
        &f.field,
        &f.temp,
        &f.lith,
        &f.coast_d,
        f.gw,
        f.gh,
        f.sea,
        FjordMaskOpts::for_width(f.gw),
    );
    assert_eq!(got, f32s(&f.v["real"]["mask"]));
    assert_eq!(
        got.iter().filter(|&&v| v > 0.0).count(),
        f.v["real"]["nonzero"].as_u64().unwrap() as usize,
        "the fixture must produce a real, non-empty mask"
    );
}

#[test]
fn carve_fjords_matches_the_reference_at_the_defaults() {
    let f = fixture();
    let mask = f32s(&f.v["real"]["mask"]);
    let got = carve_fjords(&f.field, &mask, f.gw, f.gh, f.sea, CarveFjordsOpts::default());
    assert_eq!(got, f32s(&f.v["real"]["carved"]));
    let changed = got.iter().zip(f.field.iter()).filter(|(a, b)| a != b).count();
    assert_eq!(changed, f.v["real"]["changed"].as_u64().unwrap() as usize);
    assert!(changed > 0, "the fixture must actually carve something");
}

#[test]
fn every_opts_override_is_read() {
    let f = fixture();
    let opts = FjordMaskOpts {
        coast_buffer: 3.0,
        paleo_anomaly: 4.0,
        relief_r: 1,
        relief_min: 0.02,
        relief_range: 0.3,
    };
    let mask = build_fjord_mask(&f.field, &f.temp, &f.lith, &f.coast_d, f.gw, f.gh, f.sea, opts);
    assert_eq!(mask, f32s(&f.v["opts"]["mask"]));
    assert_eq!(mask.iter().filter(|&&v| v > 0.0).count(), f.v["opts"]["nonzero"].as_u64().unwrap() as usize);
    assert_ne!(mask, f32s(&f.v["real"]["mask"]), "the overrides must change the answer");

    let carved = carve_fjords(&f.field, &mask, f.gw, f.gh, f.sea, CarveFjordsOpts { over_deep: 0.3, mask_full: 0.1 });
    assert_eq!(carved, f32s(&f.v["opts"]["carved"]));
    assert_eq!(
        carved.iter().zip(f.field.iter()).filter(|(a, b)| a != b).count(),
        f.v["opts"]["changed"].as_u64().unwrap() as usize
    );
}

/// The paleoclimate band's five constants cannot all be right on one
/// world by luck; a second world 12 °C colder pins them.
#[test]
fn the_paleoclimate_band_matches_on_a_second_world_twelve_degrees_colder() {
    let f = fixture();
    let cold = f32s(&f.v["cold"]["temp"]);
    let got =
        build_fjord_mask(&f.field, &cold, &f.lith, &f.coast_d, f.gw, f.gh, f.sea, FjordMaskOpts::for_width(f.gw));
    assert_eq!(got, f32s(&f.v["cold"]["mask"]));
    assert_eq!(got.iter().filter(|&&v| v > 0.0).count(), f.v["cold"]["nonzero"].as_u64().unwrap() as usize);
    assert_ne!(got, f32s(&f.v["real"]["mask"]), "a 12-degree shift must move the band");
}

#[test]
fn all_seven_lithology_competences_are_applied() {
    let f = fixture();
    let cold = f32s(&f.v["cold"]["temp"]);
    let sweep: Vec<u8> = (0..f.gw * f.gh).map(|i| (i % 7) as u8).collect();
    let got =
        build_fjord_mask(&f.field, &cold, &sweep, &f.coast_d, f.gw, f.gh, f.sea, FjordMaskOpts::for_width(f.gw));
    assert_eq!(got, f32s(&f.v["lith_sweep"]["mask"]));
    assert_eq!(got.iter().filter(|&&v| v > 0.0).count(), f.v["lith_sweep"]["nonzero"].as_u64().unwrap() as usize);
    assert_ne!(got, f32s(&f.v["cold"]["mask"]), "rewriting lithology must change the mask");
}
