//! The caller-owned document channel, end to end
//! (`SAVEFILE_COMPAT.md` §5, §9.6, §14; `PARITY_AUDIT.md` §23 F10).
//!
//! `project_save_with_documents` takes a shell-owned payload as JSON **text**
//! and `project_open`/`project_read_document` hand it back as JSON text. The
//! channel's whole value is that the second text is the first text, and that
//! is the one property neither end can check on its own: the writer sees only
//! what it was given, and the reader only what it found.
//!
//! So this asserts it across a real archive, over the payload most likely to
//! break it — `entities/journeys.json` (§9.6), the slot F10 was raised
//! about — carrying the two things a JSON round trip actually loses:
//!
//! * **An integer above 2^31.** §14.1 caps the format at 2^53 − 1 precisely
//!   because JavaScript and GDScript both type every JSON number as a double.
//!   Anything that reads a document into a number type and writes it back
//!   corrupts one of these; carrying the text does not.
//! * **A string with quotes and non-ASCII in it.** §14.4 says free text is
//!   arbitrary UTF-8, which means escaping, re-escaping and BOM handling all
//!   have to be exactly transparent rather than nearly so.
//!
//! The document is deliberately written *pretty-printed, with its object
//! members out of alphabetical order*. That is not decoration: it is what
//! makes "verbatim" a testable claim rather than a coincidence. Re-serializing
//! the parsed value — what this crate did before the reader existed — would
//! sort those members and drop that whitespace, and the test asserts that it
//! would, so a revert cannot pass quietly.
//!
//! `params.rs` comes in by `#[path]` for the reason `project_round_trip.rs`
//! gives: this crate is a `cdylib`, so there is no rlib to link against. The
//! Godot boundary itself (`project_read_document`, and the engine-owned
//! refusal it shares with the writer) cannot be reached from an integration
//! test for the same reason; its own unit tests in `project_bridge.rs` cover
//! the refusal, and everything below the boundary is exercised here.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_io::project::ProjectWrite;

const JOURNEYS: &str = "entities/journeys.json";
/// The sixth caller-owned slot (`SAVEFILE_COMPAT.md` §11.4, registered
/// 2026-09-03) and the second one GDScript writes for itself.
const MEASUREMENTS: &str = "annotations/measurements.json";

/// One id above 2^31 and one at §14.1's ceiling exactly. The first is what a
/// 32-bit reader loses; the second is what a double-typed one loses on the
/// very next increment, so the pair brackets the rule rather than sampling
/// inside it.
const ABOVE_2_31: i64 = 4_294_967_297;
const MAX_SAFE: i64 = 9_007_199_254_740_991;

/// Pretty-printed, members out of alphabetical order, and free text carrying
/// escaped quotes, a backslash, an em dash, a non-Latin script and an
/// astral-plane character.
fn journeys_document() -> String {
    format!(
        "{{\n  \
           \"next_id\": {ABOVE_2_31},\n  \
           \"journeys\": [\n    {{\n      \
             \"id\": {MAX_SAFE},\n      \
             \"name\": \"The \\\"salt\\\" road — Ærik\\u2019s ferry, \\\\ 城壁 🜚\",\n      \
             \"party_preset\": \"merchant_caravan\",\n      \
             \"start_year\": 412\n    \
           }}\n  ]\n\
         }}\n"
    )
}

fn a_small_world() -> (WorldParams, WorldState) {
    let mut p = WorldParams::defaults(16, 11, 24601);
    p.map_width_km = 400.0;
    let ws = generate_terrain(&p);
    (p, ws)
}

fn write_with_journeys(text: &str) -> Vec<u8> {
    write_with_documents(&[(JOURNEYS, text)], Default::default())
}

fn write_with_documents(
    docs: &[(&str, &str)],
    foreign: std::collections::BTreeMap<String, Vec<u8>>,
) -> Vec<u8> {
    let (p, ws) = a_small_world();
    let n = p.gw * p.gh;
    let fields = cartalith_io::SaveFields {
        heightmap: ws.field.clone(),
        temperature: ws.temperature.clone(),
        rainfall: ws.rainfall.clone(),
        volcanic_field: ws.volcanic_field.clone(),
        impact_field: ws.impact_field.clone(),
        strahler_order: vec![0u8; n],
    };
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
        // Pre-provenance fixture: the archive shape a user's existing
        // save has, so the assertions below cover the absent case.
        origin: None,
    };
    let mut write = ProjectWrite::new(&sp, &fields);
    write.readme = Some(cartalith_io::DEFAULT_README.to_string());
    for (slot, text) in docs {
        write.document(*slot, *text);
    }
    write.foreign = foreign;
    let mut buf = Vec::new();
    cartalith_io::write_project(std::io::Cursor::new(&mut buf), &write)
        .expect("a registered slot holding valid JSON must save");
    buf
}

#[test]
fn a_caller_owned_document_survives_the_archive_byte_for_byte() {
    let sent = journeys_document();
    let buf = write_with_journeys(&sent);

    let back = cartalith_io::read_project(std::io::Cursor::new(&buf))
        .expect("a saved project must reopen");
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());

    // The claim, stated once and unhedged.
    let got = back.text_of(JOURNEYS).expect("the slot must come back");
    assert_eq!(got, sent, "the text a caller saved is the text it gets back");

    // "Watch for silently-empty golden output" (`CLAUDE.md`): an accessor
    // returning the empty string would satisfy an equality against an empty
    // fixture. This fixture is not empty, and says so.
    assert!(sent.len() > 200, "the fixture must be a real document");
    assert!(sent.contains('🜚') && sent.contains("Ærik"), "non-ASCII must be in the fixture at all");

    // And the parsed view agrees on the values, so "verbatim" is not being
    // bought by keeping text nobody could read.
    let parsed = back.document(JOURNEYS).expect("the slot must parse too");
    assert_eq!(parsed["next_id"].as_i64(), Some(ABOVE_2_31));
    assert_eq!(parsed["journeys"][0]["id"].as_i64(), Some(MAX_SAFE));
    assert_eq!(
        parsed["journeys"][0]["name"].as_str(),
        Some("The \"salt\" road — Ærik\u{2019}s ferry, \\ 城壁 🜚")
    );

    // The mutation guard. `serde_json::to_string` of the parsed value is what
    // the bridge handed back before this reader existed; if it were still
    // equal to the input, the assertion above would prove nothing.
    let reserialized = serde_json::to_string(parsed).unwrap();
    assert_ne!(
        reserialized, sent,
        "if re-serializing were lossless there would be nothing to test"
    );
    // Specifically: it sorts the members and drops the whitespace.
    assert!(
        reserialized.find("\"journeys\"").unwrap() < reserialized.find("\"next_id\"").unwrap(),
        "re-serialization sorts object members; the archive's own text does not: {reserialized}"
    );
    assert!(!reserialized.contains('\n'));
}

/// The measurement store rides the channel above rather than being a second
/// persistence mechanism (owner ruling, 2026-09-03), and this is what "rides"
/// has to mean in practice:
///
/// * the slot is registered, so `write_project` accepts it instead of
///   answering `UnknownSlot` -- the failure a store bolted on beside the
///   channel would never see, because it would never have called this writer;
/// * it comes back **verbatim** through the same reader the shell's journeys
///   use, alongside them, so neither owner's document displaces the other's;
/// * and an entry this build does not model survives the whole cycle. That is
///   the one property adding a slot can quietly break: `is_own_entry` gates
///   both the read's `foreign` map and the write's re-emission of it, and a
///   name that appears in one list and not the other becomes either a lost
///   entry or a duplicate-name save failure.
#[test]
fn the_measurements_slot_rides_the_channel_and_leaves_a_foreign_entry_alone() {
    // Two decimals of a km figure and a fractional grid point, because the
    // store is canonical km over fractional cells and rounding either at the
    // boundary would be silent.
    let measurements = concat!(
        "{\"gw\":2048,\"gh\":1024,\"measurements\":[",
        "{\"mode\":\"distance\",\"unit\":\"km\",\"value\":120.25,",
        "\"points\":[[10.5,4.0],[88.0,12.25]]}]}"
    );
    let journeys = r#"{"next_id":1,"journeys":[]}"#;
    let alien: Vec<u8> = b"a payload from an implementation this one predates".to_vec();

    let buf = write_with_documents(
        &[(JOURNEYS, journeys), (MEASUREMENTS, measurements)],
        std::collections::BTreeMap::from([("extensions/somebody-elses.bin".to_string(), alien.clone())]),
    );

    let back = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("must reopen");
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.text_of(MEASUREMENTS), Some(measurements));
    assert_eq!(back.text_of(JOURNEYS), Some(journeys), "the two callers do not displace each other");
    assert_eq!(
        back.foreign.get("extensions/somebody-elses.bin"),
        Some(&alien),
        "an unmodelled entry must survive the read"
    );
    // It is a document slot, so it is emphatically NOT foreign -- the half of
    // `is_own_entry` that would otherwise re-emit it twice and fail the save.
    assert!(!back.foreign.contains_key(MEASUREMENTS));

    // Now the leg that only exists once both halves agree: read, carry, write.
    let again = write_with_documents(
        &[(JOURNEYS, journeys), (MEASUREMENTS, measurements)],
        back.foreign.clone(),
    );
    let twice = cartalith_io::read_project(std::io::Cursor::new(&again)).expect("must reopen twice");
    assert_eq!(twice.text_of(MEASUREMENTS), Some(measurements));
    assert_eq!(twice.foreign.get("extensions/somebody-elses.bin"), Some(&alien));

    // The values, so "verbatim" is not being bought with text nobody parses.
    let parsed = twice.document(MEASUREMENTS).expect("must parse");
    assert_eq!(parsed["gw"].as_i64(), Some(2048));
    assert_eq!(parsed["measurements"][0]["value"].as_f64(), Some(120.25));
    assert_eq!(parsed["measurements"][0]["points"][0][0].as_f64(), Some(10.5));
    assert_eq!(parsed["measurements"][0]["unit"].as_str(), Some("km"));
}

#[test]
fn the_targeted_reader_gives_the_same_text_without_decoding_the_world() {
    let sent = journeys_document();
    let buf = write_with_journeys(&sent);

    let got = cartalith_io::project::read_document(std::io::Cursor::new(&buf), JOURNEYS)
        .expect("a valid archive must not fail the read")
        .expect("the archive carries this slot");
    assert_eq!(got, sent);

    // A registered slot the archive does not carry, and a name that is not a
    // slot at all, are both "no document" here -- the caller tells them apart
    // by checking the name against `DOCUMENT_SLOTS` first, which is what
    // `project_read_document` does before it calls this.
    assert_eq!(
        cartalith_io::project::read_document(std::io::Cursor::new(&buf), "drafts/paint.json")
            .unwrap(),
        None
    );
    assert_eq!(
        cartalith_io::project::read_document(std::io::Cursor::new(&buf), "entities/journey.json")
            .unwrap(),
        None
    );
}

#[test]
fn a_flat_legacy_export_carries_no_documents_and_that_is_not_an_error() {
    // §15: the flat layout has no project layer at all. The targeted reader
    // must answer "absent" rather than refusing the archive, or a shell that
    // asks a legacy export for its journeys reports a broken file.
    let (p, ws) = a_small_world();
    let n = p.gw * p.gh;
    let fields = cartalith_io::SaveFields {
        heightmap: ws.field.clone(),
        temperature: ws.temperature.clone(),
        rainfall: ws.rainfall.clone(),
        volcanic_field: ws.volcanic_field.clone(),
        impact_field: ws.impact_field.clone(),
        strahler_order: vec![0u8; n],
    };
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
        // Pre-provenance fixture: the archive shape a user's existing
        // save has, so the assertions below cover the absent case.
        origin: None,
    };
    let mut buf = Vec::new();
    cartalith_io::write_save(
        std::io::Cursor::new(&mut buf),
        &cartalith_io::SaveWrite { params: &sp, state: params::save_state(&p), fields: &fields },
    )
    .unwrap();

    assert_eq!(
        cartalith_io::project::read_document(std::io::Cursor::new(&buf), JOURNEYS).unwrap(),
        None
    );
    let back = cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap();
    assert!(back.document_text.is_empty());
}

/// Rebuilds an archive with one entry's bytes replaced. The only way to
/// produce a document this port's own writer will not emit — see the BOM
/// test below, which needs exactly that.
fn with_entry_replaced(buf: &[u8], name: &str, bytes: &[u8]) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap();
    let names: Vec<String> = (0..src.len())
        .map(|i| src.by_index(i).unwrap().name().to_string())
        .collect();
    let mut out = Vec::new();
    {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for n in names {
            let mut data = Vec::new();
            src.by_name(&n).unwrap().read_to_end(&mut data).unwrap();
            w.start_file(n.as_str(), opts).unwrap();
            w.write_all(if n == name { bytes } else { &data }).unwrap();
        }
        w.finish().unwrap();
    }
    out
}

/// A document written by an implementation that emits integers as floats and
/// leads with a byte-order mark — the shape §14.2 exists for, and the shape
/// the HTML app's own JSON layer produces. The verbatim text keeps the
/// producer's bytes; the parsed view is the one that gets coerced. Both are
/// correct, about different questions.
#[test]
fn a_foreign_writers_bytes_are_kept_and_its_numbers_are_still_coerced() {
    let foreign = "{\"next_id\":1.0,\"journeys\":[{\"id\":2e0,\"length_km\":1.5}]}";
    let clean = write_with_journeys(r#"{"next_id":0,"journeys":[]}"#);
    let buf = with_entry_replaced(
        &clean,
        JOURNEYS,
        format!("\u{feff}{foreign}").as_bytes(),
    );

    let back = cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap();
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);

    // §14 says a reader MUST tolerate a BOM by skipping it. Handing it back
    // on the text would push that same obligation onto the caller's parser,
    // and `JSON.parse_string` does not have it.
    assert_eq!(back.text_of(JOURNEYS).unwrap(), foreign);
    assert_eq!(
        cartalith_io::project::read_document(std::io::Cursor::new(&buf), JOURNEYS).unwrap(),
        Some(foreign.to_string()),
        "both readers must strip it, or they disagree about the same file"
    );

    let parsed = back.document(JOURNEYS).unwrap();
    assert_eq!(parsed["next_id"].as_i64(), Some(1), "1.0 must read as the integer 1");
    assert_eq!(parsed["journeys"][0]["id"].as_i64(), Some(2), "2e0 must read as the integer 2");
    assert_eq!(
        parsed["journeys"][0]["length_km"].as_f64(),
        Some(1.5),
        "a genuine fraction must survive as one"
    );
}

/// This port will not *write* a byte-order mark, and refuses the save rather
/// than stripping one silently: §14 states the format is UTF-8 without a BOM,
/// and a writer that quietly edited a caller's document would break the
/// verbatim promise the rest of this file rests on.
#[test]
fn the_writer_refuses_a_document_it_would_have_to_edit() {
    let (p, ws) = a_small_world();
    let n = p.gw * p.gh;
    let fields = cartalith_io::SaveFields {
        heightmap: ws.field.clone(),
        temperature: ws.temperature.clone(),
        rainfall: ws.rainfall.clone(),
        volcanic_field: ws.volcanic_field.clone(),
        impact_field: ws.impact_field.clone(),
        strahler_order: vec![0u8; n],
    };
    let sp = cartalith_io::SaveParams {
        gw: p.gw,
        gh: p.gh,
        seed: p.tect.seed,
        map_width_km: p.map_width_km,
        sea_level: ws.sea_level,
        world: p.world,
        // Pre-provenance fixture: the archive shape a user's existing
        // save has, so the assertions below cover the absent case.
        origin: None,
    };
    for bad in ["\u{feff}{}", "{\"unterminated\": ", "not json at all"] {
        let mut write = ProjectWrite::new(&sp, &fields);
        write.document(JOURNEYS, bad);
        let mut buf = Vec::new();
        assert!(
            cartalith_io::write_project(std::io::Cursor::new(&mut buf), &write).is_err(),
            "{bad:?} must not reach the archive"
        );
    }
}
