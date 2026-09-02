//! The **interoperability export** — the flat legacy layout
//! (`SAVEFILE_COMPAT.md` §15), written for one purpose only: handing a file
//! to an unmodified pre-upgrade `Cartalith Gen1` build.
//!
//! **This is no longer the save path.** Owner decision 2026-08-25
//! (`DECISIONS.md` §7h): readers accept both layouts, writers produce only
//! the tree, and the tree's writer is [`crate::project::write_project`].
//! What survives here is §1.1's labelled export, which is lossy by
//! construction — it can carry no settlement, no faction, no label, no
//! recorded year and no vault link, because the flat layout has nowhere to
//! put them.
//!
//! Everything below this line predates that decision and describes the flat
//! layout as it was when it was the only one. It is still accurate about
//! that layout; it is no longer accurate about what "saving" means.
//!
//! The read side ([`crate::load_save`]) has existed since the MVP; this is
//! its mirror, and the thing `GUI_GAP_REGISTER.md` FI-01 (Save project),
//! DM-04 (Export ▸ World Data), JP-06/JP-08 (persisting a journey) and
//! `ROADMAP.md`'s "Options kept open, not scheduled" all named as the one
//! missing piece underneath them.
//!
//! ## The format is the reference's, not a new one
//!
//! `SAVEFILE_COMPAT.md` documents `exportZip()`/`zipStore()`/
//! `serializeState()`/`f32bytes()` from `Cartalith Gen1 v2.10.html`, and
//! this writes exactly that: a genuine PKZIP holding `params.json` plus the
//! six raw field dumps, in the reference's own entry order. Nothing here
//! invents a container, a header or a length prefix — a `.f32` entry is
//! `gw*gh*4` bytes of little-endian IEEE-754 and nothing else, which is
//! what `f32bytes(a)` produces and what [`crate::load_save`] already reads.
//!
//! DEFLATE rather than STORE, matching the reference from v1.90 onwards
//! (`CompressionStream('deflate-raw')`, method code 8). The reference's own
//! `unzipAny` reads both, and so does the `zip` crate.
//!
//! ## What this refuses to do
//!
//! Two guards, both about the failure mode this format makes easy — a file
//! that opens cleanly and is quietly wrong:
//!
//! 1. **Every field must be `gw*gh` long.** A `.f32` entry carries no
//!    length of its own, so a short `rainfall.f32` is not a parse error on
//!    the way back in; it is a silently truncated climate. [`write_save`]
//!    returns [`SaveError::FieldLength`] rather than write one.
//! 2. **The five values [`crate::load_save`] requires are written by this
//!    function, not by the caller.** `GW`, `GH`, `state.world`,
//!    `state.seaLevel`, `state.mapWidthKm` and `state.tect.seed` are
//!    injected into the caller's `state` object here, so a save this crate
//!    writes is readable by this crate's own reader *by construction*
//!    rather than by the caller having remembered.
//!
//! ## Everything else in `state` is the caller's
//!
//! This crate deliberately has no `WorldParams` (it cannot: `cartalith-io`
//! sits *below* `cartalith-engine`, which depends on it). The generation
//! parameters are the caller's to shape — `cartalith-godot`'s `params.rs`
//! owns that vocabulary and builds the object — and this module only
//! guarantees the four keys above and the container around it.

use crate::{SaveFields, SaveParams};
use std::io::{Seek, Write};

/// The `v` field of `params.json`. The reference writes its own `VERSION`
/// there; `loadZip()` never branches on it (every compatibility shim it has
/// tests for a missing *key*, not a version number), so this is provenance
/// metadata rather than a format selector. `210` is the frozen reference
/// snapshot this port is written against (`reference/Cartalith Gen1
/// v2.10.html`).
pub const SAVE_VERSION: i64 = 210;

/// One save, ready to be written. Borrowed rather than owned so a caller
/// can hand over the live world's fields without copying `gw*gh*4` bytes
/// six times on the way to a file that is about to hold them anyway.
pub struct SaveWrite<'a> {
    pub params: &'a SaveParams,
    /// The `state` object. Whatever the caller puts here is written
    /// verbatim, except for the four keys [`write_save`] owns (see the
    /// module doc). A non-object `Value` — including `Null` — is treated as
    /// an empty object rather than rejected, so a caller with nothing to
    /// add can pass `Value::Null`.
    pub state: serde_json::Value,
    pub fields: &'a SaveFields,
}

#[derive(Debug)]
pub enum SaveError {
    Zip(zip::result::ZipError),
    Io(std::io::Error),
    /// A field's length disagrees with `gw*gh`. Carries the entry name and
    /// both lengths, because "which of the six" is the whole diagnostic.
    FieldLength { entry: &'static str, expected: usize, got: usize },
    /// [`crate::project::write_project`] was handed a payload for a path
    /// the tree does not define (`SAVEFILE_COMPAT.md` §5), or a second copy
    /// of a raster `fields` already owns.
    ///
    /// A hard error rather than a silent pass-through, and that refusal is
    /// the whole point of the slot registry: an invented entry name is how
    /// one concept ends up with two homes.
    UnknownSlot(String),
    /// A registered raster's length disagrees with `gw*gh`. Same diagnostic
    /// as `FieldLength`, for a `String` path rather than one of the six.
    RasterLength { entry: String, expected: usize, got: usize },
    /// A raster was handed to a slot of a different element type — an
    /// `f32` for `rasters/territory.i32`. Caught here because the extension
    /// is what tells a reader the element width, so a mismatch produces an
    /// entry no reader can decode.
    RasterElement { entry: String, expected: crate::project::Element, got: crate::project::Element },
    /// A document's text is not valid JSON. Refused at write time so an
    /// archive never carries a document its own reader will have to skip.
    DocumentJson { entry: String, message: String },
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Zip(e) => write!(f, "zip error: {e}"),
            SaveError::Io(e) => write!(f, "io error: {e}"),
            SaveError::FieldLength { entry, expected, got } => {
                write!(f, "{entry}: expected {expected} values for this grid, got {got}")
            }
            SaveError::UnknownSlot(path) => {
                write!(f, "{path}: not a slot the project format defines (SAVEFILE_COMPAT.md section 5)")
            }
            SaveError::RasterLength { entry, expected, got } => {
                write!(f, "{entry}: expected {expected} values for this grid, got {got}")
            }
            SaveError::RasterElement { entry, expected, got } => {
                write!(f, "{entry}: slot holds {} values, got {}", expected.ext(), got.ext())
            }
            SaveError::DocumentJson { entry, message } => write!(f, "{entry}: not valid JSON ({message})"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<zip::result::ZipError> for SaveError {
    fn from(e: zip::result::ZipError) -> Self {
        SaveError::Zip(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

/// The `params.json` object for one save, with the keys
/// [`crate::load_save`] requires filled in from `params`.
///
/// Public so a caller can inspect (or test) exactly what would be written
/// without writing a file; [`write_save`] calls it.
pub fn params_json(params: &SaveParams, state: &serde_json::Value) -> serde_json::Value {
    let mut state = match state {
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()),
        _ => serde_json::json!({}),
    };
    let obj = state.as_object_mut().expect("just built as an object");
    obj.insert("world".into(), serde_json::json!(params.world));
    obj.insert("seaLevel".into(), serde_json::json!(params.sea_level));
    obj.insert("mapWidthKm".into(), serde_json::json!(params.map_width_km));
    // `tect` is a nested object and `loadZip()` merges `state` *shallowly*
    // (`Object.assign(state, pk.state)`), so a `tect` written here replaces
    // the reference's whole default block rather than merging into it --
    // which is why the caller is expected to write a complete one and this
    // only fills in the seed. Creating the object when the caller wrote
    // none keeps `load_save`'s `state.tect.seed` lookup satisfied either
    // way.
    let tect = obj.entry("tect").or_insert_with(|| serde_json::json!({}));
    if !tect.is_object() {
        *tect = serde_json::json!({});
    }
    tect.as_object_mut().expect("just ensured an object").insert("seed".into(), serde_json::json!(params.seed));

    serde_json::json!({
        "v": SAVE_VERSION,
        "GW": params.gw,
        "GH": params.gh,
        "state": state,
    })
}

/// Writes one save to any seekable sink — a `File`, or a
/// `Cursor<Vec<u8>>` for a round-trip test — in the reference's own entry
/// order.
pub fn write_save<W: Write + Seek>(sink: W, save: &SaveWrite<'_>) -> Result<(), SaveError> {
    let n = save.params.gw * save.params.gh;
    let f = save.fields;
    for (entry, got) in [
        ("heightmap.f32", f.heightmap.len()),
        ("temperature.f32", f.temperature.len()),
        ("rainfall.f32", f.rainfall.len()),
        ("volcanic_field.f32", f.volcanic_field.len()),
        ("impact_field.f32", f.impact_field.len()),
        ("strahler_order.bin", f.strahler_order.len()),
    ] {
        if got != n {
            return Err(SaveError::FieldLength { entry, expected: n, got });
        }
    }

    let mut writer = zip::ZipWriter::new(sink);
    // DEFLATE, the reference's own method from v1.90 (`SAVEFILE_COMPAT.md`).
    // `SimpleFileOptions::default()` is already `Deflated`; named here
    // because it is a format decision, not a default worth inheriting
    // silently.
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // `JSON.stringify(serializeState(), null, 2)` -- two-space indent, which
    // is `serde_json`'s own pretty default.
    writer.start_file("params.json", opts)?;
    let json = params_json(save.params, &save.state);
    writer.write_all(&serde_json::to_vec_pretty(&json).expect("a Value always serializes"))?;

    for (name, values) in [
        ("heightmap.f32", &f.heightmap),
        ("temperature.f32", &f.temperature),
        ("rainfall.f32", &f.rainfall),
        ("volcanic_field.f32", &f.volcanic_field),
        ("impact_field.f32", &f.impact_field),
    ] {
        writer.start_file(name, opts)?;
        write_f32_entries(&mut writer, values)?;
    }

    writer.start_file("strahler_order.bin", opts)?;
    writer.write_all(&f.strahler_order)?;

    writer.finish()?;
    Ok(())
}

/// The write half of `read_f32_entries`: a bare little-endian byte
/// dump, no header and no length prefix (`f32bytes`). Buffered in 64 KiB
/// chunks rather than one `write_all` per value -- at this port's 8192x8192
/// ceiling a field is 67 million values, and a per-value call into the
/// DEFLATE encoder is the whole cost of the export.
fn write_f32_entries<W: Write>(sink: &mut W, values: &[f32]) -> std::io::Result<()> {
    const CHUNK_VALUES: usize = 16 * 1024;
    let mut buf: Vec<u8> = Vec::with_capacity(CHUNK_VALUES * 4);
    for chunk in values.chunks(CHUNK_VALUES) {
        buf.clear();
        for &v in chunk {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        sink.write_all(&buf)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_save;
    use std::io::Cursor;

    fn sample(gw: usize, gh: usize) -> (SaveParams, SaveFields) {
        let n = gw * gh;
        let params = SaveParams { gw, gh, seed: 4242, map_width_km: 1234.5, sea_level: 0.37, world: true };
        let fields = SaveFields {
            // Values chosen to survive an f64 -> f32 -> f64 trip exactly and
            // to differ per index, so a swapped or truncated entry cannot
            // pass by accident.
            heightmap: (0..n).map(|i| i as f32 * 0.25).collect(),
            temperature: (0..n).map(|i| 30.0 - i as f32 * 0.5).collect(),
            rainfall: (0..n).map(|i| (i % 13) as f32 * 0.125).collect(),
            volcanic_field: (0..n).map(|i| (i % 5) as f32 * 0.5).collect(),
            impact_field: (0..n).map(|i| (i % 3) as f32 * 0.75).collect(),
            strahler_order: (0..n).map(|i| (i % 251) as u8).collect(),
        };
        (params, fields)
    }

    #[test]
    fn write_then_load_round_trips_every_field() {
        let (params, fields) = sample(17, 11);
        let mut buf = Vec::new();
        write_save(
            Cursor::new(&mut buf),
            &SaveWrite { params: &params, state: serde_json::json!({ "tect": { "plates": 9 } }), fields: &fields },
        )
        .expect("write_save should succeed");

        let back = load_save(Cursor::new(&buf)).expect("our own reader must read our own writer");
        assert_eq!(back.params, params);
        assert_eq!(back.fields, fields);
    }

    #[test]
    fn writer_owns_the_keys_the_reader_requires() {
        // A caller who writes a *contradicting* state must not be able to
        // produce a file whose params.json disagrees with its own fields.
        let (params, _) = sample(4, 4);
        let json = params_json(
            &params,
            &serde_json::json!({ "world": false, "seaLevel": 0.9, "mapWidthKm": 1.0, "tect": { "seed": 7, "plates": 9 } }),
        );
        assert_eq!(json["GW"], 4);
        assert_eq!(json["GH"], 4);
        assert_eq!(json["state"]["world"], true);
        assert_eq!(json["state"]["seaLevel"], 0.37);
        assert_eq!(json["state"]["mapWidthKm"], 1234.5);
        assert_eq!(json["state"]["tect"]["seed"], 4242);
        // ...while everything else the caller put in `tect` survives.
        assert_eq!(json["state"]["tect"]["plates"], 9);
    }

    #[test]
    fn params_json_tolerates_a_caller_with_nothing_to_add() {
        let (params, _) = sample(4, 4);
        for state in [serde_json::Value::Null, serde_json::json!(3), serde_json::json!({})] {
            let json = params_json(&params, &state);
            assert_eq!(json["state"]["tect"]["seed"], 4242);
            assert_eq!(json["state"]["mapWidthKm"], 1234.5);
        }
    }

    #[test]
    fn a_short_field_is_refused_not_truncated() {
        let (params, mut fields) = sample(6, 5);
        fields.rainfall.pop();
        let mut buf = Vec::new();
        let err = write_save(Cursor::new(&mut buf), &SaveWrite { params: &params, state: serde_json::json!({}), fields: &fields })
            .expect_err("a short field must not be written");
        assert!(matches!(err, SaveError::FieldLength { entry: "rainfall.f32", expected: 30, got: 29 }));
    }

    #[test]
    fn entries_are_the_documented_set_in_the_reference_order() {
        let (params, fields) = sample(3, 3);
        let mut buf = Vec::new();
        write_save(Cursor::new(&mut buf), &SaveWrite { params: &params, state: serde_json::json!({}), fields: &fields })
            .unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let names: Vec<String> = (0..archive.len()).map(|i| archive.by_index(i).unwrap().name().to_string()).collect();
        assert_eq!(
            names,
            vec![
                "params.json",
                "heightmap.f32",
                "temperature.f32",
                "rainfall.f32",
                "volcanic_field.f32",
                "impact_field.f32",
                "strahler_order.bin",
            ]
        );
        // A `.f32` entry is exactly gw*gh*4 bytes -- no header, no prefix.
        assert_eq!(archive.by_name("heightmap.f32").unwrap().size(), 9 * 4);
        assert_eq!(archive.by_name("strahler_order.bin").unwrap().size(), 9);
    }
}
