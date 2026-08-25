//! The project archive, end to end over a **real** generated world
//! (`SAVEFILE_COMPAT.md`, `DECISIONS.md` §7h).
//!
//! `cartalith-io`'s own tests prove the container, the slot registry and
//! the number handling; `project_bridge.rs`'s own tests prove the document
//! schemas round-trip. Neither exercises the seam between them — the
//! parameter block being *split* into `params.json`'s two views on the way
//! out and *rejoined* on the way in, over parameters a live generator
//! actually produced.
//!
//! That seam is where a save quietly stops regenerating the world it
//! describes, so this asserts the strongest available statement: the
//! restored parameters regenerate the same world bit-for-bit.
//!
//! `params.rs` comes in by `#[path]` — this crate is a `cdylib`, so there
//! is no rlib to link — the same pattern `save_round_trip.rs`,
//! `params_mapping.rs` and `golden_parity_render.rs` already use. The split
//! itself is written out again here rather than called, because
//! `project_bridge.rs` is `godot`-dependent throughout; `project_save`'s
//! own body is the one place it has to match.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_io::project::{ProjectWrite, Raster};

fn fields_of(ws: &WorldState, n: usize) -> cartalith_io::SaveFields {
    cartalith_io::SaveFields {
        heightmap: ws.field.clone(),
        temperature: ws.temperature.clone(),
        rainfall: ws.rainfall.clone(),
        volcanic_field: ws.volcanic_field.clone(),
        impact_field: ws.impact_field.clone(),
        strahler_order: match ws.stream_order.as_ref() {
            Some(order) => order.iter().map(|&o| o.clamp(0, 255) as u8).collect(),
            None => vec![0u8; n],
        },
    }
}

/// A non-square world with every stage really run — the shape a real export
/// has, and the one a row-major bug would survive on a square grid.
fn a_real_world() -> (WorldParams, WorldState) {
    let mut p = WorldParams::defaults(24, 15, 24601);
    p.map_width_km = 640.0;
    p.climate.lat_n = 62.0;
    p.climate.equator_temp = 27.0;
    p.tect.plates = 11;
    p.tect.blur_r = 21.0;
    p.volc.count = 24;
    p.peak_m = 5200.0;
    let ws = generate_terrain(&p);
    (p, ws)
}

/// `project_save`'s own split of `params::save_state` into the two views
/// `SAVEFILE_COMPAT.md` §13.1 defines.
fn split_params(p: &WorldParams) -> (serde_json::Value, serde_json::Value) {
    let mut state = params::save_state(p);
    let cartalith = state
        .as_object_mut()
        .and_then(|o| o.remove("cartalith"))
        .unwrap_or(serde_json::Value::Null);
    (cartalith, state)
}

#[test]
fn a_real_world_survives_the_tree_and_regenerates_bit_for_bit() {
    let (p, ws) = a_real_world();
    let n = p.gw * p.gh;
    let fields = fields_of(&ws, n);
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
    };

    let mut write = ProjectWrite::new(&sp, &fields);
    let (cartalith, reference) = split_params(&p);
    write.cartalith_params = cartalith;
    write.reference_params = reference;
    write.readme = Some(cartalith_io::DEFAULT_README.to_string());
    // A civ raster, so the extra-raster path is exercised over a real grid
    // rather than only over the synthetic ones in the unit tests.
    write.raster("rasters/territory.i32", Raster::I32((0..n).map(|i| (i % 7) as i32).collect()));
    write.document("entities/settlements.json", r#"{"next_id":1,"settlements":[]}"#);

    let mut buf = Vec::new();
    cartalith_io::write_project(std::io::Cursor::new(&mut buf), &write).expect("a real world should save");

    let back = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("a saved project should reopen");
    assert_eq!(back.layout, cartalith_io::Layout::Tree);
    assert_eq!(back.format_version, cartalith_io::PROJECT_FORMAT_VERSION);
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert!(back.foreign_entries.is_empty(), "{:?}", back.foreign_entries);

    // -- the world the renderer reads -------------------------------------
    assert_eq!(back.save.params, sp);
    // Bit-exact: a raster entry is a byte reinterpretation, not a second
    // computation, so anything short of equality is a format-handling bug
    // rather than floating-point drift.
    assert_eq!(back.save.fields.heightmap, ws.field, "heightmap");
    assert_eq!(back.save.fields.temperature, ws.temperature, "temperature");
    assert_eq!(back.save.fields.rainfall, ws.rainfall, "rainfall");
    assert_eq!(back.save.fields.volcanic_field, ws.volcanic_field, "volcanic_field");
    assert_eq!(back.save.fields.impact_field, ws.impact_field, "impact_field");
    assert_eq!(back.save.fields.strahler_order, fields.strahler_order, "strahler_order");
    assert_eq!(
        back.raster("rasters/territory.i32"),
        Some(&Raster::I32((0..n).map(|i| (i % 7) as i32).collect()))
    );

    // "Watch for silently-empty golden output" (`CLAUDE.md`): a writer that
    // emitted `gw*gh` zeros would satisfy every equality above if the
    // generator had also produced zeros. It did not.
    assert!(back.save.fields.heightmap.iter().any(|&v| v > 0.0), "the heightmap should have real relief");
    assert!(back.save.fields.temperature.iter().any(|&v| v != 0.0), "temperature should be real");
    assert!(back.save.fields.rainfall.iter().any(|&v| v > 0.0), "rainfall should be real");
    assert!(back.save.fields.strahler_order.iter().any(|&v| v > 0), "strahler_order should have real channels");
    assert!(back.save.fields.heightmap.iter().all(|v| v.is_finite()), "no NaN may reach the file");

    // -- the settings, split into two views and rejoined -------------------
    let mut restored = params::defaults();
    let applied = params::apply_saved_state(&mut restored, &back.save.state);
    assert_eq!(applied, params::PARAMS.len(), "every parameter must survive the split and the rejoin");
    restored.gw = back.save.params.gw;
    restored.gh = back.save.params.gh;
    restored.tect.seed = back.save.params.seed;
    restored.map_width_km = back.save.params.map_width_km;
    assert_eq!(restored, p, "the restored parameters must equal the ones the world was generated from");
    assert_eq!(generate_terrain(&restored).field, ws.field, "the saved parameters must regenerate the same world");

    // Both views really are present and really are separate (§13.1).
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&buf)).unwrap();
    let params_json: serde_json::Value = {
        use std::io::Read;
        let mut e = archive.by_name("params.json").expect("params.json must be written");
        let mut s = String::new();
        e.read_to_string(&mut s).unwrap();
        serde_json::from_str(&s).unwrap()
    };
    assert!(params_json["cartalith"].is_object(), "the port's own dotted keys");
    assert!(params_json["reference"].is_object(), "the reference-named block");
    // ...and the grid is not repeated there. It lives in project.json alone.
    assert!(params_json.get("GW").is_none());
    assert!(params_json["reference"].get("cartalith").is_none(), "the two views must not nest");
}

#[test]
fn the_tree_is_the_tree_the_specification_publishes() {
    // The entry names are the contract a second implementation is written
    // against, so they are asserted rather than assumed.
    let (p, ws) = a_real_world();
    let n = p.gw * p.gh;
    let fields = fields_of(&ws, n);
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
    };
    let mut write = ProjectWrite::new(&sp, &fields);
    write.readme = Some(cartalith_io::DEFAULT_README.to_string());
    let mut buf = Vec::new();
    cartalith_io::write_project(std::io::Cursor::new(&mut buf), &write).unwrap();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&buf)).unwrap();
    let mut names: Vec<String> = (0..archive.len()).map(|i| archive.by_index(i).unwrap().name().to_string()).collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "README.md",
            "project.json",
            "rasters/heightmap.f32",
            "rasters/impact_field.f32",
            "rasters/rainfall.f32",
            "rasters/strahler_order.u8",
            "rasters/temperature.f32",
            "rasters/volcanic_field.f32",
        ]
    );
    // Every raster entry is exactly gw*gh*element bytes -- no header, no
    // length prefix (§8).
    assert_eq!(archive.by_name("rasters/heightmap.f32").unwrap().size(), (n * 4) as u64);
    assert_eq!(archive.by_name("rasters/strahler_order.u8").unwrap().size(), n as u64);
    // `project.json` is first, so a truncated transfer is diagnosable (§3.1).
    assert_eq!(archive.by_index(0).unwrap().name(), "project.json");
}

#[test]
fn a_flat_legacy_export_still_opens_through_the_project_reader() {
    // The owner's ruling: read both layouts, write only the tree. This is
    // the half that must never regress -- every `Cartalith Gen1` export
    // that exists is flat.
    let (p, ws) = a_real_world();
    let n = p.gw * p.gh;
    let fields = fields_of(&ws, n);
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
    };
    let mut buf = Vec::new();
    cartalith_io::write_save(
        std::io::Cursor::new(&mut buf),
        &cartalith_io::SaveWrite { params: &sp, state: params::save_state(&p), fields: &fields },
    )
    .unwrap();

    let back = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("a flat archive must still open");
    assert_eq!(back.layout, cartalith_io::Layout::Flat);
    assert_eq!(back.save.params, sp);
    assert_eq!(back.save.fields.heightmap, ws.field);
    assert!(back.documents.is_empty(), "a flat archive carries no project layer, and says so by being empty");

    // The parameters survive the flat path too -- unchanged behaviour, held
    // here because `load_save` now dispatches through the project reader.
    let mut restored = params::defaults();
    assert_eq!(params::apply_saved_state(&mut restored, &back.save.state), params::PARAMS.len());
    restored.gw = back.save.params.gw;
    restored.gh = back.save.params.gh;
    restored.tect.seed = back.save.params.seed;
    restored.map_width_km = back.save.params.map_width_km;
    assert_eq!(restored, p);
}

/// The real fixture, not a synthetic one: `cartalith-io`'s own golden
/// fixture is a genuine HTML-app export, and the project reader must open
/// it exactly as `load_save` always has.
#[test]
fn the_real_html_app_export_opens_as_a_flat_project() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../cartalith-io/tests/fixtures/real_export_seed24601.zip");
    let file = std::fs::File::open(path).expect("the golden fixture must be there");
    let back = cartalith_io::read_project(std::io::BufReader::new(file)).expect("a real export must open");
    assert_eq!(back.layout, cartalith_io::Layout::Flat);
    assert!(back.save.params.gw > 0 && back.save.params.gh > 0);
    assert_eq!(back.save.fields.heightmap.len(), back.save.params.gw * back.save.params.gh);
    assert!(back.save.fields.heightmap.iter().any(|&v| v > 0.0), "a real export has real relief");
}
