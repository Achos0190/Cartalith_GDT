#![cfg(feature = "zip")]

//! Golden-parity test for the pack **archive** layer against a pack the
//! reference itself exported — not a synthetic fixture.
//!
//! `fixtures/reference_pack.zip` was produced by running the reference's own
//! `PackManifestBuilder.build()` (`Cartalith Gen1 v2.10.html` line 26964) and
//! its own `zipStore()` (line 12009) headlessly under Node's
//! `vm.runInContext`, the same extraction technique
//! `cartalith-native/docs/CHANGELOG.md`'s 2026-08-15 "extraction harness
//! upgrade" entry established and `cartalith-io`'s
//! `golden_parity_real_export.rs` uses for world saves. The vocabulary table
//! (`FAMILIES`), the slot registry (`AssetDB`), the manifest builder, the ZIP
//! writer, `unzipAny`, `parsePackManifest` and `packSummary` were all lifted
//! verbatim by line range out of the frozen HTML.
//!
//! Two things in that run are **not** the reference's own code, and both are
//! stated here rather than glossed:
//!
//! - `renderToBlob` is a canvas rasteriser; headlessly there is no canvas, so
//!   the harness substituted a real PNG encoder producing genuine, valid PNGs
//!   at the family's own bake size (512² opaque for ground textures, 256²
//!   RGBA for sprites). The archive layer is indifferent to what a PNG
//!   *depicts*; what it is not indifferent to is that the payload really is
//!   PNG bytes at a realistic size, and it is.
//! - `E('alPackName'|'alPackAuthor'|'alPackLicense')` are three DOM text
//!   inputs, stubbed with real values so the manifest carries a name, an
//!   author and a licence.
//!
//! Everything else — which files exist, what they are named, what order they
//! are written in, which are STORED and which DEFLATED, the frozen 1980-01-01
//! timestamps, the manifest's exact JSON text, the CRC-32 of every entry —
//! came out of the reference's own code and is asserted below.
//!
//! `fixtures/reference_pack_captured.json` records that run's own view of the
//! archive: what its `unzipAny()` read back out, what its `parsePackManifest`
//! and `packSummary` then said about it, and the raw central-directory record
//! of every entry. Comparing this port against that capture — rather than
//! against a re-read of the `.zip` by this port's own reader — is what makes
//! the test a parity check instead of a self-consistency check.

use cartalith_assets::{read_pack, read_pack_entries, write_pack};
use serde_json::Value;
use std::io::Cursor;

const FIXTURE_ZIP: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reference_pack.zip"));

fn capture() -> Value {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/reference_pack_captured.json"
    ))
    .expect("capture fixture should open");
    serde_json::from_str(&text).expect("capture fixture should be JSON")
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|s| s.as_str().expect("string").to_string())
        .collect()
}

/// The reference's own `crc32` (line 12005), ported so the bytes this port
/// reads out can be checked against the checksums the reference wrote into the
/// archive's central directory. Ten lines beats a dependency, and it keeps the
/// comparison anchored to a value the reference computed rather than one this
/// port's `zip` crate computed.
fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c = table[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

// ===========================================================================
// Reading a real reference-exported pack.
// ===========================================================================

#[test]
fn read_pack_entries_matches_the_reference_unzip_any() {
    let cap = capture();
    let got = read_pack_entries(Cursor::new(FIXTURE_ZIP)).expect("reference pack should open");

    let expected = cap["readBack"].as_array().expect("readBack array");
    assert_eq!(
        got.len(),
        expected.len(),
        "entry count must match what the reference's own unzipAny() read back"
    );
    for e in expected {
        let name = e["name"].as_str().expect("name");
        let bytes = got
            .get(name)
            .unwrap_or_else(|| panic!("entry {name} missing from this port's read"));
        assert_eq!(
            bytes.len() as u64,
            e["len"].as_u64().expect("len"),
            "length mismatch for {name}"
        );
    }

    // Byte-level check against the checksums the reference itself wrote.
    for e in cap["centralDirectory"].as_array().expect("cd array") {
        let name = e["name"].as_str().expect("name");
        assert_eq!(
            u64::from(crc32(&got[name])),
            e["crc"].as_u64().expect("crc"),
            "CRC-32 mismatch for {name}"
        );
    }
}

#[test]
fn parsing_a_real_reference_pack_matches_the_reference_parser() {
    let cap = capture();
    let (manifest, _entries) = read_pack(Cursor::new(FIXTURE_ZIP)).expect("pack should parse");

    assert_eq!(manifest.name, cap["manifestName"].as_str().expect("name"));
    assert_eq!(manifest.author, cap["manifestAuthor"].as_str().expect("author"));
    assert_eq!(manifest.license, cap["manifestLicense"].as_str().expect("license"));
    assert_eq!(
        cartalith_assets::pack_summary(&manifest),
        cap["summary"].as_str().expect("summary")
    );
    // One warning, and it is the "not yet used by the live map" notice for the
    // trait/biomes/terrains sections -- a real pack the reference exported
    // carries art in three families its own renderer does not consume yet.
    assert_eq!(manifest.warnings, strs(&cap["warnings"]));
}

/// The manifest this port re-emits must be the *same text* the reference's own
/// exporter wrote — same key order (`textures`, `icons`, then only the
/// non-empty `biomes`/`terrains`/`structures`/`custom`), same two-space
/// indent, same one-element-array-not-bare-string shape for icon variants.
#[test]
fn to_pack_json_reproduces_the_reference_exporters_own_manifest_text() {
    let cap = capture();
    let (manifest, _) = read_pack(Cursor::new(FIXTURE_ZIP)).expect("pack should parse");
    assert_eq!(manifest.to_pack_json(), cap["packJson"].as_str().expect("packJson"));
}

// ===========================================================================
// Writing one back.
// ===========================================================================

/// Round-trip: read the reference's pack, write it back out, and check the
/// result against the reference's own archive record entry for entry.
///
/// Exact byte equality is not the bar and could not be: the one DEFLATED entry
/// (`pack.json`) is compressed by `miniz_oxide` here and by the browser's
/// zlib there, and two conforming deflate encoders need not agree on a bit
/// stream. Everything a reader can observe is asserted instead — order,
/// method, CRC-32, uncompressed size, timestamp — plus the payloads
/// themselves via a re-read.
#[test]
fn write_pack_reproduces_the_reference_exporters_archive() {
    let cap = capture();
    let (manifest, entries) = read_pack(Cursor::new(FIXTURE_ZIP)).expect("pack should parse");

    let mut buf = Vec::new();
    write_pack(Cursor::new(&mut buf), &manifest, &entries).expect("write_pack should succeed");

    let mut archive = zip::ZipArchive::new(Cursor::new(&buf)).expect("our own output must be a zip");
    let cd = cap["centralDirectory"].as_array().expect("cd array");
    assert_eq!(archive.len(), cd.len(), "entry count");

    for (i, want) in cd.iter().enumerate() {
        let file = archive.by_index_raw(i).expect("entry");
        let name = want["name"].as_str().expect("name");
        // Order: the exporter's family walk, with pack.json appended last.
        assert_eq!(file.name(), name, "entry {i} out of order");
        assert_eq!(
            u64::from(file.crc32()),
            want["crc"].as_u64().expect("crc"),
            "CRC-32 mismatch for {name}"
        );
        assert_eq!(file.size(), want["usize"].as_u64().expect("usize"), "size for {name}");
        // .png STORED (0), pack.json DEFLATED (8) -- the reference's own rule.
        let method = if want["method"].as_u64() == Some(0) {
            zip::CompressionMethod::Stored
        } else {
            zip::CompressionMethod::Deflated
        };
        assert_eq!(file.compression(), method, "compression method for {name}");
        // zipStore hardcodes DOS date 0x0021 / time 0x0000 = 1980-01-01
        // 00:00:00, so exports are reproducible. `zip`'s own default is the
        // wall clock, which would not be.
        assert_eq!(want["mdate"].as_u64(), Some(33), "capture sanity: DOS date word");
        assert_eq!(want["mtime"].as_u64(), Some(0), "capture sanity: DOS time word");
        let dt = file.last_modified().expect("a timestamp");
        assert_eq!(
            (dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()),
            (1980, 1, 1, 0, 0, 0),
            "timestamp for {name}"
        );
    }
    drop(archive);

    // The entry ORDER the reference's exporter itself produced.
    let mut archive = zip::ZipArchive::new(Cursor::new(&buf)).expect("zip");
    let ours: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index_raw(i).expect("entry").name().to_string())
        .collect();
    assert_eq!(ours, strs(&cap["exporterEntryOrder"]));
    drop(archive);

    // And the payloads survive: reading our own archive back must give the
    // reference's entries byte for byte.
    let reread = read_pack_entries(Cursor::new(&buf)).expect("re-read");
    assert_eq!(reread, entries, "round-tripped payloads must be identical");
}
