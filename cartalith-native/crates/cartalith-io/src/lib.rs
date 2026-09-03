//! save/load — the HTML app's .zip save format (SAVEFILE_COMPAT.md)
//!
//! `MVP_SCOPE.md` point 12 scoped this to **reading only**, and that was
//! the whole of this crate for Phases 1-5. [`save`] is the other half,
//! authorised by the owner (2026-08-23) once `GUI_GAP_REGISTER.md` FI-01,
//! DM-04, JP-06/JP-08 and MEA-07 had all queued up behind the same missing
//! writer. `SAVEFILE_COMPAT.md`'s own "Deferred" section is updated to
//! match.

pub mod atlas;
pub mod gzip;
pub mod project;
pub mod save;
pub mod tiles;

pub use atlas::{
    atlas_chunk_file, atlas_key_str, atlas_meta_key, build_atlas_manifest, decode_chunk,
    encode_chunk, fnv1a32_hex, world_key, AtlasChunk, AtlasChunkDesc, AtlasChunkRecord, AtlasKey,
    AtlasManifest, AtlasMeta, AtlasStore, ATLAS_KIND, ATLAS_MANIFEST,
};
pub use gzip::{gunzip_bytes, gzip_bytes};
// `project::manifest_json` is deliberately NOT re-exported here: `tiles`
// already owns that name at the crate root and two `manifest_json`s in one
// namespace is the kind of collision this format exists to avoid. Call it
// as `project::manifest_json`.
pub use project::{
    coerce_integral_floats, read_project, write_project, Element, Layout, ProjectData, ProjectWrite, Raster,
    RasterSlot, CORE_RASTERS, DEFAULT_README, DOCUMENT_SLOTS, HISTORY_TERRITORY_PREFIX, PROJECT_FORMAT,
    PROJECT_FORMAT_VERSION, PROJECT_MANIFEST, RASTER_SLOTS,
};
pub use save::{params_json, write_save, SaveError, SaveWrite, SAVE_VERSION};
pub use tiles::{
    build_tile_manifest, js_num, json_string, manifest_json, pack_height16, unpack_height16, CoarseBounds, TileManifest,
    TileManifestOpts, TileRecord,
};

use std::io::Read;

/// `params.json`'s subset this port's terrain pipeline actually reads
/// (`SAVEFILE_COMPAT.md`'s own "two workable approaches" — this is
/// approach 1, `serde_json::Value` plus manual field pulls, chosen as
/// "fastest to MVP" since `state` is large and mostly civ/UI data this
/// port has nothing to deserialize into yet).
#[derive(Debug, Clone, PartialEq)]
pub struct SaveParams {
    pub gw: usize,
    pub gh: usize,
    pub seed: i32,
    pub map_width_km: f64,
    pub sea_level: f64,
    pub world: bool,
    /// **How the height field was produced**, verbatim from the archive —
    /// `SAVEFILE_COMPAT.md` §7's `world.origin`, whose three defined values
    /// are `"gen"`, `"import"` and `"region"` (`cartalith_godot::
    /// bake_bridge`'s `ORIGIN_*` constants are the same strings). This crate
    /// deliberately does not police it: an unrecognised value from a newer
    /// writer is carried through unchanged rather than folded into a known
    /// one, per §14.3's unknown-member rule.
    ///
    /// **`None` means the archive did not say, and nothing else.** Every
    /// archive written before this member existed is that case, and it is
    /// not the same fact as `Some("gen")` — which is exactly why this is an
    /// `Option` and not a `String` defaulting to `"gen"`. A caller that
    /// re-saves a `None` writes `None` again rather than inventing a
    /// provenance the file never carried.
    ///
    /// What a *consumer* does with `None` is its own decision, and
    /// `cartalith-godot`'s is documented at its `world_key()`: the atlas key
    /// substitutes `"gen"` there, because giving unknown-provenance archives
    /// their own namespace would orphan the baked atlas of every project
    /// saved before this member existed. That substitution is one line in
    /// one consumer; it is not this type's answer.
    pub origin: Option<String>,
}

/// The terrain fields a save carries that this port reads
/// (`SAVEFILE_COMPAT.md`'s entry table). `strahler_order` is `u8` in the
/// save (`0` = non-channel) — matches the file format exactly; this
/// port's own `strahler_from_receivers` produces `i16` internally for a
/// wider per-cell order range, but nothing about the *save format* itself
/// needs anything wider than a byte.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveFields {
    pub heightmap: Vec<f32>,
    pub temperature: Vec<f32>,
    pub rainfall: Vec<f32>,
    pub volcanic_field: Vec<f32>,
    pub impact_field: Vec<f32>,
    pub strahler_order: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveData {
    pub params: SaveParams,
    pub fields: SaveFields,
    /// `params.json`'s whole `state` object, exactly as it was read.
    ///
    /// [`SaveParams`] above is the handful of values this port's terrain
    /// pipeline needs; this is everything else the file carried, kept so a
    /// caller can pull what it models out of it — `cartalith-godot`'s
    /// `params::apply_saved_state` reads the generation-parameter block
    /// [`save::write_save`] wrote — without this crate having to grow a
    /// struct for 200+ keys of civ and UI state it has nothing to
    /// deserialize into (`SAVEFILE_COMPAT.md`'s own reasoning for approach
    /// 1 over approach 2).
    ///
    /// `Value::Null` when the file had no `state` object at all. Reading it
    /// never fails, so a save whose `state` is unrecognisable still loads
    /// its terrain.
    pub state: serde_json::Value,
}

#[derive(Debug)]
pub enum LoadError {
    Zip(zip::result::ZipError),
    Io(std::io::Error),
    MissingEntry(&'static str),
    Json(serde_json::Error),
    MissingField(&'static str),
    /// `project.json` exists but its `format` is not this format's
    /// (`SAVEFILE_COMPAT.md` §4). Carries whatever it said — empty when the
    /// member was absent entirely. Refused rather than guessed: §4 makes
    /// that entry's *presence* the layout test, so an unrelated file of the
    /// same name must not be read as a world.
    NotAProject(String),
    /// A raster's byte length disagrees with the grid the manifest
    /// declares. A raster entry carries no length of its own, so this is
    /// the only place a truncated world can be caught (`SAVEFILE_COMPAT.md`
    /// §8). Only fatal for the heightmap; every other raster is skipped
    /// with a warning (§6.4).
    RasterLength(String),
    /// A numeric member is inside `SAVEFILE_COMPAT.md` §14.1's safe-integer
    /// range but outside what this implementation can represent.
    ///
    /// Today that is `world.seed` alone: the format allows +/-(2^53 - 1),
    /// this engine's RNG is seeded with an `i32`, and the read used to be a
    /// bare `as i32` — which in Rust **saturates silently**. A conforming
    /// archive with a large seed therefore loaded with a different seed, and
    /// nothing said so. The terrain itself still came back correct (it is
    /// restored from `rasters/heightmap.f32`, not regenerated), so the damage
    /// was narrow and nasty: pressing Generate afterwards produced a world
    /// that was not the one on screen, and `save_round_trip`'s own
    /// "regenerate from restored parameters and assert bit-identical" promise
    /// quietly stopped holding.
    ///
    /// Refusing is the honest answer. §6.4a's damage ladder puts a value with
    /// no representable meaning at the archive level when the value decides
    /// what every later regeneration produces.
    OutOfRange {
        field: &'static str,
        value: f64,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Zip(e) => write!(f, "zip error: {e}"),
            LoadError::Io(e) => write!(f, "io error: {e}"),
            LoadError::MissingEntry(name) => write!(f, "missing zip entry: {name}"),
            LoadError::Json(e) => write!(f, "params.json parse error: {e}"),
            LoadError::MissingField(name) => write!(f, "params.json missing field: {name}"),
            LoadError::NotAProject(found) if found.is_empty() => {
                write!(f, "project.json has no \"format\" member; this is not a Cartalith project")
            }
            LoadError::NotAProject(found) => {
                write!(f, "project.json says format \"{found}\"; this is not a Cartalith project")
            }
            LoadError::RasterLength(message) => write!(f, "{message}"),
            LoadError::OutOfRange { field, value } => write!(
                f,
                "{field} is {value}, which this build cannot represent \n(the format allows the full safe-integer range; this engine seeds with a 32-bit value)"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<zip::result::ZipError> for LoadError {
    fn from(e: zip::result::ZipError) -> Self {
        LoadError::Zip(e)
    }
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::Io(e)
    }
}

/// `f32bytes(a)` is a bare little-endian byte dump of a `Float32Array` —
/// no header, no length prefix (`SAVEFILE_COMPAT.md`). Reads explicitly
/// rather than casting: the `Vec<u8>` a zip entry decompresses into is
/// allocator-aligned, not guaranteed `f32`-aligned.
fn read_f32_entries(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()
}

fn read_entry(archive: &mut zip::ZipArchive<impl Read + std::io::Seek>, name: &'static str) -> Result<Vec<u8>, LoadError> {
    let mut entry = archive.by_name(name).map_err(|_| LoadError::MissingEntry(name))?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

fn json_num(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for &seg in path {
        cur = cur.get(seg)?;
    }
    cur.as_f64()
}

fn json_bool(v: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut cur = v;
    for &seg in path {
        cur = cur.get(seg)?;
    }
    cur.as_bool()
}

/// Reads a save (`SAVEFILE_COMPAT.md`) from any seekable byte source —
/// `zip::ZipArchive` accepts a `File`, an in-memory `Cursor<Vec<u8>>`, or
/// anything else `Read + Seek`, so this isn't tied to a filesystem path.
///
/// **Ignores unknown entries; never errors on them** — a real export
/// always carries more than this reader wants (biome/lithology rasters,
/// civ data, a baked atlas, `map.png`, a README), and that's normal, not
/// corruption (`SAVEFILE_COMPAT.md`'s own explicit instruction).
///
/// **Reads both layouts** (`SAVEFILE_COMPAT.md` §1, owner decision
/// 2026-08-25): a flat legacy archive, and the tree a
/// [`project::write_project`] writes. It returns only the terrain half of a
/// tree archive — the entities, history and annotations need
/// [`project::read_project`] — so every caller that only ever wanted a
/// world keeps working unchanged against either.
pub fn load_save<R: Read + std::io::Seek>(reader: R) -> Result<SaveData, LoadError> {
    project::read_project(reader).map(|p| p.save)
}

/// The flat legacy layout's reader (`SAVEFILE_COMPAT.md` §15), over an
/// already-opened archive. Split out of [`load_save`] so
/// [`project::read_project`] can dispatch to it after §4's layout test
/// without opening the archive twice.
pub(crate) fn load_from_archive(
    archive: &mut zip::ZipArchive<impl Read + std::io::Seek>,
) -> Result<SaveData, LoadError> {
    let params_bytes = read_entry(archive, "params.json")?;
    let params_json: serde_json::Value = serde_json::from_slice(&params_bytes).map_err(LoadError::Json)?;

    let gw = json_num(&params_json, &["GW"]).ok_or(LoadError::MissingField("GW"))? as usize;
    let gh = json_num(&params_json, &["GH"]).ok_or(LoadError::MissingField("GH"))? as usize;
    let seed = json_num(&params_json, &["state", "tect", "seed"]).ok_or(LoadError::MissingField("state.tect.seed"))? as i32;
    let map_width_km =
        json_num(&params_json, &["state", "mapWidthKm"]).ok_or(LoadError::MissingField("state.mapWidthKm"))?;
    let sea_level = json_num(&params_json, &["state", "seaLevel"]).ok_or(LoadError::MissingField("state.seaLevel"))?;
    let world = json_bool(&params_json, &["state", "world"]).unwrap_or(false);
    // A member of `params.json` itself, not of `state`: `state` is the reference
    // app's own vocabulary and `loadZip()` merges the whole object into its
    // live state, so a member of this port's invention does not belong
    // there. `None` when the key is absent, which is every archive the
    // reference itself has ever written (SAVEFILE_COMPAT.md 15).
    let origin = params_json.get("origin").and_then(|v| v.as_str()).map(str::to_string);

    let heightmap = read_f32_entries(&read_entry(archive, "heightmap.f32")?);
    let temperature = read_f32_entries(&read_entry(archive, "temperature.f32")?);
    let rainfall = read_f32_entries(&read_entry(archive, "rainfall.f32")?);
    let volcanic_field = read_f32_entries(&read_entry(archive, "volcanic_field.f32")?);
    let impact_field = read_f32_entries(&read_entry(archive, "impact_field.f32")?);
    let strahler_order = read_entry(archive, "strahler_order.bin")?;

    Ok(SaveData {
        params: SaveParams { gw, gh, seed, map_width_km, sea_level, world, origin },
        fields: SaveFields { heightmap, temperature, rainfall, volcanic_field, impact_field, strahler_order },
        state: params_json.get("state").cloned().unwrap_or(serde_json::Value::Null),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    /// Builds a synthetic save `.zip` matching `SAVEFILE_COMPAT.md`'s
    /// documented entry list — the closest available stand-in for a real
    /// HTML-app export in this environment (no browser to produce one).
    /// Verifies `load_save`'s own reading logic is correct; it is *not* a
    /// substitute for testing against a real export, which
    /// `SAVEFILE_COMPAT.md` itself flags as the thing to confirm early.
    /// Uses STORE (method 0), matching pre-v1.90 saves and the simpler of
    /// the two methods the reference file's own `zipStore()` writes.
    fn build_test_zip(gw: usize, gh: usize, seed: i32, map_width_km: f64, sea_level: f64, world: bool) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            let params = serde_json::json!({
                "v": 210,
                "GW": gw,
                "GH": gh,
                "state": {
                    "world": world,
                    "seaLevel": sea_level,
                    "mapWidthKm": map_width_km,
                    "tect": { "seed": seed },
                }
            });
            writer.start_file("params.json", opts).unwrap();
            writer.write_all(serde_json::to_string(&params).unwrap().as_bytes()).unwrap();

            let n = gw * gh;
            let write_f32 = |writer: &mut zip::ZipWriter<Cursor<&mut Vec<u8>>>, name: &str, vals: &[f32]| {
                writer.start_file(name, opts).unwrap();
                for &v in vals {
                    writer.write_all(&v.to_le_bytes()).unwrap();
                }
            };
            let heightmap: Vec<f32> = (0..n).map(|i| (i as f32) / (n as f32)).collect();
            let temperature: Vec<f32> = (0..n).map(|i| 20.0 - (i as f32) * 0.1).collect();
            let rainfall: Vec<f32> = (0..n).map(|i| ((i % 7) as f32) / 7.0).collect();
            let volcanic_field: Vec<f32> = vec![0.0; n];
            let impact_field: Vec<f32> = vec![0.0; n];
            write_f32(&mut writer, "heightmap.f32", &heightmap);
            write_f32(&mut writer, "temperature.f32", &temperature);
            write_f32(&mut writer, "rainfall.f32", &rainfall);
            write_f32(&mut writer, "volcanic_field.f32", &volcanic_field);
            write_f32(&mut writer, "impact_field.f32", &impact_field);

            let strahler: Vec<u8> = (0..n).map(|i| (i % 4) as u8).collect();
            writer.start_file("strahler_order.bin", opts).unwrap();
            writer.write_all(&strahler).unwrap();

            // An entry this reader doesn't know about -- must be ignored,
            // not error (SAVEFILE_COMPAT.md's own explicit instruction).
            writer.start_file("map.png", opts).unwrap();
            writer.write_all(b"not a real png, just an unknown entry").unwrap();

            writer.finish().unwrap();
        }
        buf
    }

    /// `build_test_zip` is modelled on a genuine `Cartalith Gen1` export,
    /// which has no `origin` member and never will — so this is the reading
    /// of an archive from the HTML app itself, not of one this port wrote.
    /// It must be `None`, not `Some("gen")`: the file did not say.
    #[test]
    fn a_reference_export_records_no_origin() {
        let save = load_save(Cursor::new(build_test_zip(4, 4, 7, 800.0, 0.42, false)))
            .expect("load_save should succeed");
        assert_eq!(save.params.origin, None);
    }

    #[test]
    fn load_save_round_trip_region() {
        let gw = 10;
        let gh = 8;
        let zip_bytes = build_test_zip(gw, gh, 12345, 800.0, 0.42, false);
        let save = load_save(Cursor::new(zip_bytes)).expect("load_save should succeed");

        assert_eq!(save.params.gw, gw);
        assert_eq!(save.params.gh, gh);
        assert_eq!(save.params.seed, 12345);
        assert_eq!(save.params.map_width_km, 800.0);
        assert_eq!(save.params.sea_level, 0.42);
        assert!(!save.params.world);

        let n = gw * gh;
        assert_eq!(save.fields.heightmap.len(), n);
        assert_eq!(save.fields.temperature.len(), n);
        assert_eq!(save.fields.rainfall.len(), n);
        assert_eq!(save.fields.volcanic_field.len(), n);
        assert_eq!(save.fields.impact_field.len(), n);
        assert_eq!(save.fields.strahler_order.len(), n);

        let expected_heightmap: Vec<f32> = (0..n).map(|i| (i as f32) / (n as f32)).collect();
        assert_eq!(save.fields.heightmap, expected_heightmap);
        let expected_temperature: Vec<f32> = (0..n).map(|i| 20.0 - (i as f32) * 0.1).collect();
        assert_eq!(save.fields.temperature, expected_temperature);
        let expected_strahler: Vec<u8> = (0..n).map(|i| (i % 4) as u8).collect();
        assert_eq!(save.fields.strahler_order, expected_strahler);
    }

    #[test]
    fn load_save_round_trip_world() {
        let gw = 12;
        let gh = 6;
        let zip_bytes = build_test_zip(gw, gh, 999, 40000.0, 0.35, true);
        let save = load_save(Cursor::new(zip_bytes)).expect("load_save should succeed");

        assert!(save.params.world);
        assert_eq!(save.params.map_width_km, 40000.0);
        assert_eq!(save.params.sea_level, 0.35);
    }

    #[test]
    fn load_save_missing_entry_errors_cleanly() {
        // A zip with only params.json -- no heightmap.f32 -- should
        // return a clear MissingEntry error, not panic.
        let mut buf = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            let params = serde_json::json!({"GW": 4, "GH": 4, "state": {"seaLevel": 0.42, "mapWidthKm": 800.0, "tect": {"seed": 1}}});
            writer.start_file("params.json", opts).unwrap();
            writer.write_all(serde_json::to_string(&params).unwrap().as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        let result = load_save(Cursor::new(buf));
        assert!(matches!(result, Err(LoadError::MissingEntry("heightmap.f32"))));
    }
}
