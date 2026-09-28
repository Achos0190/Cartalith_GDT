//! The **interoperability export** — the flat legacy layout
//! (`SAVEFILE_COMPAT.md` §15), written for one purpose only: handing a file
//! to an unmodified pre-upgrade `Cartalith Gen1` build.
//!
//! **This is no longer the save path.** Owner decision 2026-08-25
//! (`DECISIONS.md` §7h): readers accept both layouts, writers produce only
//! the tree, and the tree's writer is [`crate::project::write_project`].
//! What survives here is §1.1's labelled export, which is lossy by
//! construction — it writes no settlement, no faction, no label, no recorded
//! year and no vault link. The flat layout does have a place for most of
//! them — inside `state`, in the HTML app's own vocabulary (§15.1), which is
//! where [`crate::legacy`] reads them from — but this writer does not
//! translate them back into it.
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

/// Everything that can stop [`write_save`] from writing a save — see each
/// variant for what it means.
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

    let mut root = serde_json::Map::new();
    root.insert("v".into(), serde_json::json!(SAVE_VERSION));
    root.insert("GW".into(), serde_json::json!(params.gw));
    root.insert("GH".into(), serde_json::json!(params.gh));
    // Provenance, and **only when the caller has some**: an absent key is
    // how this format says "unknown", so writing `"gen"` for a
    // `SaveParams::origin` of `None` would turn a world whose history the
    // archive never recorded into one that claims to have been generated.
    // That is the same defect the atlas key had before an origin element
    // existed, moved one layer out.
    //
    // A member of `params.json` itself, not of `state`, for the reason
    // `crate::load_from_archive` states at the reading end: `state` is the
    // reference app's object and `loadZip()` merges all of it into its live
    // state. An unknown *top-level* key is ignored by `loadZip()`, which is
    // what makes this safe to add to an interoperability export.
    if let Some(origin) = &params.origin {
        root.insert("origin".into(), serde_json::json!(origin));
    }
    // Same MAY, absent-means-unknown shape as `origin` immediately above,
    // for the world's own generated display name (`SAVEFILE_COMPAT.md` §15
    // / §7's `world.name`, added for `OUTSTANDING_WORK.md`'s Recent-worlds
    // row).
    if let Some(name) = &params.name {
        root.insert("name".into(), serde_json::json!(name));
    }
    root.insert("state".into(), state);
    serde_json::Value::Object(root)
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
        // `.as_slice()` on every entry, uniformly: `heightmap`/`temperature`/
        // `rainfall` are `Arc<Vec<f32>>` and `volcanic_field`/`impact_field`
        // are plain `Vec<f32>` (`SaveFields`'s own doc), so a literal array
        // of `&f.<name>` would mix two reference types and fail to compile.
        ("heightmap.f32", f.heightmap.as_slice()),
        ("temperature.f32", f.temperature.as_slice()),
        ("rainfall.f32", f.rainfall.as_slice()),
        ("volcanic_field.f32", f.volcanic_field.as_slice()),
        ("impact_field.f32", f.impact_field.as_slice()),
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
    // 16K values (64 KiB of f32) per DEFLATE call -- a size judged to keep
    // per-call overhead negligible while chunks stay small.
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

/// Unit tests for [`write_save`]/[`params_json`]'s reference-format entries
/// and its `origin`/`name` provenance members.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_save;
    use std::io::Cursor;
    use std::sync::Arc;

    /// A minimal `SaveParams`/`SaveFields` pair for the tests below, at a
    /// caller-chosen grid size. `origin`/`name` start `None` deliberately —
    /// the pre-provenance, pre-naming shape most of the tests below mutate
    /// from.
    fn sample(gw: usize, gh: usize) -> (SaveParams, SaveFields) {
        let n = gw * gh;
        // `origin: None`/`name: None` deliberately -- see `project.rs`'s own
        // `sample`.
        let params = SaveParams {
            gw,
            gh,
            seed: 4242,
            map_width_km: 1234.5,
            sea_level: 0.37,
            world: true,
            origin: None,
            name: None,
        };
        let fields = SaveFields {
            // Values chosen to survive an f64 -> f32 -> f64 trip exactly and
            // to differ per index, so a swapped or truncated entry cannot
            // pass by accident.
            heightmap: Arc::new((0..n).map(|i| i as f32 * 0.25).collect()),
            temperature: Arc::new((0..n).map(|i| 30.0 - i as f32 * 0.5).collect()),
            rainfall: Arc::new((0..n).map(|i| (i % 13) as f32 * 0.125).collect()),
            volcanic_field: (0..n).map(|i| (i % 5) as f32 * 0.5).collect(),
            impact_field: (0..n).map(|i| (i % 3) as f32 * 0.75).collect(),
            strahler_order: (0..n).map(|i| (i % 251) as u8).collect(),
        };
        (params, fields)
    }

    #[test]
    fn write_then_load_round_trips_every_field() {
        // Protects: a value written into every one of the six raw fields
        // (all f32 grids plus the strahler byte grid) comes back unchanged
        // after a real zip round trip through this crate's own reader --
        // the basic write/read contract the rest of this module's guards
        // sit on top of.
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

    /// The flat writer's half of `SaveParams::origin` (`SAVEFILE_COMPAT.md`
    /// §15's `params.json`), and the case that matters most: **absent means
    /// absent.** A `None` written as `"gen"` would be a world claiming a
    /// provenance its file never carried, which is the defect the member
    /// exists to close rather than to relocate.
    ///
    /// Protects: an absent `SaveParams::origin` stays absent from both
    /// `params.json`'s top level and `state`, never fabricated as `"gen"`.
    #[test]
    fn an_unknown_origin_is_an_absent_key_not_a_written_gen() {
        let (params, _) = sample(4, 4);
        assert_eq!(params.origin, None, "the sample is the pre-provenance shape");
        let json = params_json(&params, &serde_json::json!({}));
        assert!(json.get("origin").is_none(), "wrote an origin for a caller that had none: {json}");
        // ...and not smuggled into `state` either, where a reference build's
        // `Object.assign` would merge it into its own live state.
        assert!(json["state"].get("origin").is_none());
    }

    #[test]
    fn a_known_origin_is_written_beside_the_grid_not_inside_state() {
        // Protects: a known origin lands at `params.json`'s top level, not
        // inside `state` where `loadZip()`'s `Object.assign` would merge it
        // into the reference app's own live state -- and the four keys
        // this writer owns (`v`, `GW`, `GH`, `state.seaLevel`) are untouched
        // by adding it.
        let (mut params, _) = sample(4, 4);
        params.origin = Some("import".to_string());
        let json = params_json(&params, &serde_json::json!({}));
        assert_eq!(json["origin"], "import");
        assert!(json["state"].get("origin").is_none());
        // The four members §15 fixes are untouched by the addition.
        assert_eq!(json["v"], SAVE_VERSION);
        assert_eq!(json["GW"], 4);
        assert_eq!(json["GH"], 4);
        assert_eq!(json["state"]["seaLevel"], 0.37);
    }

    /// Round-trips through a real archive, for each of the three origins this
    /// port writes, an unrecognised fourth, and the absent case — the reader
    /// is `crate::load_from_archive` by way of `load_save`.
    ///
    /// Protects: every value `SaveParams::origin` can hold (`gen`, `import`,
    /// `region`, `sculpt`, an unrecognised string, and `None`) survives a
    /// real zip write/read cycle exactly, not just the JSON-level check
    /// above.
    #[test]
    fn every_origin_survives_a_flat_round_trip_including_absence() {
        for origin in [None, Some("gen"), Some("import"), Some("region"), Some("sculpt")] {
            let (mut params, fields) = sample(4, 4);
            params.origin = origin.map(str::to_string);
            let mut buf = Vec::new();
            write_save(
                Cursor::new(&mut buf),
                &SaveWrite { params: &params, state: serde_json::json!({}), fields: &fields },
            )
            .expect("write_save should succeed");
            let back = crate::load_save(Cursor::new(&buf)).expect("read back");
            assert_eq!(back.params.origin.as_deref(), origin, "origin did not survive");
        }
    }

    /// The flat writer's half of `SaveParams::name` — same MAY, absent-means-
    /// absent shape as `origin` above, for `OUTSTANDING_WORK.md`'s Recent-
    /// worlds "name" gap.
    ///
    /// Protects: an absent `SaveParams::name` stays absent, never written as
    /// an empty or placeholder string a Recent-worlds list would then show.
    #[test]
    fn an_unknown_name_is_an_absent_key_not_a_fabricated_one() {
        let (params, _) = sample(4, 4);
        assert_eq!(params.name, None, "the sample is the pre-name shape");
        let json = params_json(&params, &serde_json::json!({}));
        assert!(json.get("name").is_none(), "wrote a name for a caller that had none: {json}");
        assert!(json["state"].get("name").is_none());
    }

    #[test]
    fn a_known_name_is_written_beside_the_grid_not_inside_state() {
        // Protects: a known world name lands at `params.json`'s top level,
        // not inside `state`, mirroring the origin case above.
        let (mut params, _) = sample(4, 4);
        params.name = Some("The Vharen Reach".to_string());
        let json = params_json(&params, &serde_json::json!({}));
        assert_eq!(json["name"], "The Vharen Reach");
        assert!(json["state"].get("name").is_none());
    }

    /// Round-trips through a real flat archive: a named world, and absence.
    ///
    /// Protects: a world name and the absent case both survive a real zip
    /// write/read cycle, not just the JSON-level check above.
    #[test]
    fn a_world_name_survives_a_flat_round_trip_including_absence() {
        for name in [None, Some("The Vharen Reach"), Some("Kessa")] {
            let (mut params, fields) = sample(4, 4);
            params.name = name.map(str::to_string);
            let mut buf = Vec::new();
            write_save(
                Cursor::new(&mut buf),
                &SaveWrite { params: &params, state: serde_json::json!({}), fields: &fields },
            )
            .expect("write_save should succeed");
            let back = crate::load_save(Cursor::new(&buf)).expect("read back");
            assert_eq!(back.params.name.as_deref(), name, "world name did not survive");
        }
    }

    #[test]
    fn writer_owns_the_keys_the_reader_requires() {
        // Protects: the four keys this writer owns (GW, GH, state.world,
        // state.seaLevel, state.mapWidthKm, state.tect.seed) win over
        // contradicting values a caller wrote into `state`, so a save can
        // never disagree with its own `fields` -- while everything else the
        // caller put in `tect` survives untouched.
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
        // Protects: `SaveWrite::state`'s "a non-object Value... is treated
        // as an empty object rather than rejected" guarantee -- Null, a
        // bare number, and an empty object all still get the four required
        // keys filled in rather than panicking or producing a state with no
        // `tect.seed`.
        let (params, _) = sample(4, 4);
        for state in [serde_json::Value::Null, serde_json::json!(3), serde_json::json!({})] {
            let json = params_json(&params, &state);
            assert_eq!(json["state"]["tect"]["seed"], 4242);
            assert_eq!(json["state"]["mapWidthKm"], 1234.5);
        }
    }

    #[test]
    fn a_short_field_is_refused_not_truncated() {
        // Protects: the module doc's guard 1 -- a field shorter than gw*gh
        // is a hard `SaveError::FieldLength`, not a write that would leave
        // a silently truncated climate on disk with no length prefix to
        // catch it on read-back.
        let (params, mut fields) = sample(6, 5);
        Arc::make_mut(&mut fields.rainfall).pop();
        let mut buf = Vec::new();
        let err = write_save(Cursor::new(&mut buf), &SaveWrite { params: &params, state: serde_json::json!({}), fields: &fields })
            .expect_err("a short field must not be written");
        assert!(matches!(err, SaveError::FieldLength { entry: "rainfall.f32", expected: 30, got: 29 }));
    }

    #[test]
    fn entries_are_the_documented_set_in_the_reference_order() {
        // Protects: the archive's entry names and their write order match
        // the reference's own `exportZip()` exactly, and a `.f32` entry is
        // exactly gw*gh*4 bytes with no header or length prefix -- the
        // shape `crate::load_save` depends on.
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
