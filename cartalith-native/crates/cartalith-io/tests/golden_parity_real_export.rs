//! Golden-parity test against a **real** HTML-app export, not a synthetic
//! fixture (`MVP_SCOPE.md` criterion 7: "opens a real HTML-app `.zip` and
//! renders that save's terrain... not merely 'it did not crash'").
//!
//! `real_export_seed24601.zip` is a genuine export produced by running the
//! actual, unmodified reference engine (`reference/Cartalith Gen1 v2.10.html`,
//! script tag #1 only -- this avoids a real name collision with the unrelated
//! urban-morphology block's own `function generate(seed,opts)` in a later
//! script tag, which would otherwise be the last `function generate`
//! declaration in the shared global scope and shadow the real terrain
//! `generate()`) under Node's `vm.runInContext`, the same DOM/timer-stub
//! harness technique `cartalith-native/docs/CHANGELOG.md`'s 2026-08-15
//! "extraction harness upgrade" entry established, extended with a
//! permissive Proxy-based DOM-element stub (this file's top-level script
//! wires a lot of UI at load time that a headless extraction doesn't care
//! about, it only needs to not crash). Config matches
//! `golden_parity_carve.rs`'s case 0 (`gw=14 gh=11 seed=24601 world=false`)
//! deliberately, and `field[0..5]` from this independent extraction run
//! matched that fixture's `expected_field[0..5]` exactly on the first
//! attempt -- real cross-validation that the harness reconstruction is
//! faithful, not a coincidentally-similar but differently-configured run.
//!
//! `real_export_seed24601_captured.json` holds the SAME values, captured
//! directly from the JS sandbox's own `field`/`tempField`/`rainField`/
//! `volcanicField`/`impactField` typed arrays (plus the computed Strahler
//! raster) at export time -- independently of the `.zip`'s own bytes, not
//! by re-opening the `.zip` and re-reading it back (that would only test
//! `load_save` against itself). Comparing `cartalith_io::load_save`'s
//! parsed output against this independent capture is the actual criterion-7
//! check: does the real save format round-trip through this port's loader
//! byte-for-byte, not just through its own synthetic round-trip tests.
//!
//! f32 values are asserted for EXACT equality, not tolerance: `heightmap.f32`
//! etc. are a bare little-endian `Float32Array` dump (`SAVEFILE_COMPAT.md`),
//! so reading them back is a lossless byte reinterpretation, not a second
//! computation -- any mismatch here would mean the zip/entry-reading path
//! itself is wrong, not ordinary floating-point drift.

use std::fs::File;

#[test]
fn load_save_matches_real_html_app_export() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/real_export_seed24601.zip");
    let file = File::open(path).expect("real export fixture should open");
    let save = cartalith_io::load_save(file).expect("load_save should parse a real HTML-app export");

    let captured_json = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/real_export_seed24601_captured.json"
    ))
    .expect("captured.json fixture should read");
    let captured: serde_json::Value = serde_json::from_str(&captured_json).expect("captured.json should parse");

    let as_f32_vec = |key: &str| -> Vec<f32> {
        captured[key].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect()
    };
    let as_u8_vec = |key: &str| -> Vec<u8> {
        captured[key].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect()
    };

    let gw = captured["gw"].as_u64().unwrap() as usize;
    let gh = captured["gh"].as_u64().unwrap() as usize;
    assert_eq!(save.params.gw, gw);
    assert_eq!(save.params.gh, gh);
    assert_eq!(save.params.seed, captured["seed"].as_i64().unwrap() as i32);
    assert_eq!(save.params.world, captured["world"].as_bool().unwrap());
    assert_eq!(save.params.map_width_km, captured["mapWidthKm"].as_f64().unwrap());
    assert_eq!(save.params.sea_level, captured["seaLevel"].as_f64().unwrap());

    let n = gw * gh;
    assert_eq!(save.fields.heightmap.len(), n);

    // Bit-exact: a lossless byte round-trip, not a second computation.
    assert_eq!(save.fields.heightmap, as_f32_vec("heightmap"), "heightmap");
    assert_eq!(save.fields.temperature, as_f32_vec("temperature"), "temperature");
    assert_eq!(save.fields.rainfall, as_f32_vec("rainfall"), "rainfall");
    assert_eq!(save.fields.volcanic_field, as_f32_vec("volcanic_field"), "volcanic_field");
    assert_eq!(save.fields.impact_field, as_f32_vec("impact_field"), "impact_field");
    assert_eq!(save.fields.strahler_order, as_u8_vec("strahler_order"), "strahler_order");

    // Sanity: this is real generation output, not all-zero/placeholder data --
    // volcanic_field and strahler_order both have genuine non-zero variation.
    assert!(save.fields.volcanic_field.iter().any(|&v| v > 0.0), "volcanic_field should have real volcanism");
    assert!(save.fields.strahler_order.iter().any(|&v| v > 0), "strahler_order should have real channels");
}
