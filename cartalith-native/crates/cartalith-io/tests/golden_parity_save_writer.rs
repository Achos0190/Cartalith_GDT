//! The writer, checked against the same **real** HTML-app export
//! `golden_parity_real_export.rs` reads (`SAVEFILE_COMPAT.md`).
//!
//! `save.rs`'s own unit tests round-trip synthetic data through
//! `write_save` -> `load_save`, which proves the two agree with each other
//! and nothing more. This file is the check that matters for a file the
//! owner will trust with real work: take a genuine export produced by the
//! unmodified reference engine, re-write it through this port's writer
//! preserving its `params.json` `state` verbatim, and assert that what
//! comes back matches the **independent** value capture
//! (`real_export_seed24601_captured.json`) rather than merely matching
//! itself.
//!
//! If the writer laid a header, a length prefix, a byte order or an entry
//! name down differently from `f32bytes`/`exportZip`, this is where it
//! shows: the re-written archive would still open, and the numbers coming
//! out of it would be wrong.

use std::io::Cursor;

const ZIP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/real_export_seed24601.zip");
const CAPTURED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/real_export_seed24601_captured.json");

/// The `state` object out of the real export's own `params.json` — 200+
/// keys the reference wrote, none of which this crate models. Round-tripping
/// it verbatim is the point: a writer that silently dropped the parts it does
/// not understand would lose the whole civ/UI payload of any save it touched.
fn real_state() -> serde_json::Value {
    let file = std::fs::File::open(ZIP).expect("real export fixture should open");
    let mut archive = zip::ZipArchive::new(file).expect("fixture should be a zip");
    let entry = archive.by_name("params.json").expect("fixture should carry params.json");
    let parsed: serde_json::Value = serde_json::from_reader(entry).expect("params.json should parse");
    parsed["state"].clone()
}

#[test]
fn rewriting_a_real_export_preserves_every_value() {
    let original = cartalith_io::load_save(std::fs::File::open(ZIP).unwrap()).expect("fixture should load");

    let mut buf = Vec::new();
    cartalith_io::write_save(
        Cursor::new(&mut buf),
        &cartalith_io::SaveWrite { params: &original.params, state: real_state(), fields: &original.fields },
    )
    .expect("write_save should succeed on a real export's contents");

    let back = cartalith_io::load_save(Cursor::new(&buf)).expect("our writer's output must load");

    // Against the INDEPENDENT capture, not against `original` -- comparing a
    // re-write to the thing it was written from only proves the writer is
    // consistent with the reader.
    let captured: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(CAPTURED).expect("capture should read")).expect("capture parses");
    let as_f32 = |key: &str| -> Vec<f32> {
        captured[key].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect()
    };
    let as_u8 =
        |key: &str| -> Vec<u8> { captured[key].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect() };

    assert_eq!(back.params.gw, captured["gw"].as_u64().unwrap() as usize);
    assert_eq!(back.params.gh, captured["gh"].as_u64().unwrap() as usize);
    assert_eq!(back.params.seed, captured["seed"].as_i64().unwrap() as i32);
    assert_eq!(back.params.world, captured["world"].as_bool().unwrap());
    assert_eq!(back.params.map_width_km, captured["mapWidthKm"].as_f64().unwrap());
    assert_eq!(back.params.sea_level, captured["seaLevel"].as_f64().unwrap());

    assert_eq!(back.fields.heightmap, as_f32("heightmap"), "heightmap");
    assert_eq!(back.fields.temperature, as_f32("temperature"), "temperature");
    assert_eq!(back.fields.rainfall, as_f32("rainfall"), "rainfall");
    assert_eq!(back.fields.volcanic_field, as_f32("volcanic_field"), "volcanic_field");
    assert_eq!(back.fields.impact_field, as_f32("impact_field"), "impact_field");
    assert_eq!(back.fields.strahler_order, as_u8("strahler_order"), "strahler_order");

    // Same non-emptiness guard the sibling golden test carries, for the same
    // reason (`CLAUDE.md`: "watch for silently-empty golden output").
    assert!(back.fields.volcanic_field.iter().any(|&v| v > 0.0), "volcanic_field should have real volcanism");
    assert!(back.fields.strahler_order.iter().any(|&v| v > 0), "strahler_order should have real channels");
}

#[test]
fn the_unmodelled_half_of_state_survives_the_trip() {
    let original = cartalith_io::load_save(std::fs::File::open(ZIP).unwrap()).expect("fixture should load");
    let state = real_state();

    let mut buf = Vec::new();
    cartalith_io::write_save(
        Cursor::new(&mut buf),
        &cartalith_io::SaveWrite { params: &original.params, state: state.clone(), fields: &original.fields },
    )
    .unwrap();

    let mut archive = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
    let written: serde_json::Value = serde_json::from_reader(archive.by_name("params.json").unwrap()).unwrap();

    // The four keys the writer owns are re-derived from `SaveParams`; every
    // other key -- `places`, `labels`, `mapIcons`, `viz`, `cartoPaint`, the
    // whole erosion/glacial/coastal/planet block -- must come back byte-equal.
    let mut expected = state.as_object().unwrap().clone();
    let mut skipped = 0;
    for key in ["world", "seaLevel", "mapWidthKm"] {
        assert!(expected.remove(key).is_some(), "the real export should carry state.{key}");
        skipped += 1;
    }
    assert_eq!(skipped, 3);
    for (key, value) in &expected {
        assert_eq!(&written["state"][key], value, "state.{key} must survive verbatim");
    }
    assert!(expected.len() > 20, "the real export's state should be large, got {} keys", expected.len());

    // `tect` is the one nested block the writer reaches into, and only for
    // `seed`; its siblings must be untouched.
    assert_eq!(written["state"]["tect"]["seed"], original.params.seed);
    assert_eq!(written["state"]["tect"]["plates"], state["tect"]["plates"]);
}
