//! save/load — reads the HTML app's .zip saves (SAVEFILE_COMPAT.md)
//!
//! `MVP_SCOPE.md` point 12: **reading only**, one specific thing — not a
//! general save/load licence. Writing a save is explicitly out of scope
//! (`SAVEFILE_COMPAT.md`'s own "Deferred" section).

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
}

#[derive(Debug)]
pub enum LoadError {
    Zip(zip::result::ZipError),
    Io(std::io::Error),
    MissingEntry(&'static str),
    Json(serde_json::Error),
    MissingField(&'static str),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Zip(e) => write!(f, "zip error: {e}"),
            LoadError::Io(e) => write!(f, "io error: {e}"),
            LoadError::MissingEntry(name) => write!(f, "missing zip entry: {name}"),
            LoadError::Json(e) => write!(f, "params.json parse error: {e}"),
            LoadError::MissingField(name) => write!(f, "params.json missing field: {name}"),
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
pub fn load_save<R: Read + std::io::Seek>(reader: R) -> Result<SaveData, LoadError> {
    let mut archive = zip::ZipArchive::new(reader)?;

    let params_bytes = read_entry(&mut archive, "params.json")?;
    let params_json: serde_json::Value = serde_json::from_slice(&params_bytes).map_err(LoadError::Json)?;

    let gw = json_num(&params_json, &["GW"]).ok_or(LoadError::MissingField("GW"))? as usize;
    let gh = json_num(&params_json, &["GH"]).ok_or(LoadError::MissingField("GH"))? as usize;
    let seed = json_num(&params_json, &["state", "tect", "seed"]).ok_or(LoadError::MissingField("state.tect.seed"))? as i32;
    let map_width_km =
        json_num(&params_json, &["state", "mapWidthKm"]).ok_or(LoadError::MissingField("state.mapWidthKm"))?;
    let sea_level = json_num(&params_json, &["state", "seaLevel"]).ok_or(LoadError::MissingField("state.seaLevel"))?;
    let world = json_bool(&params_json, &["state", "world"]).unwrap_or(false);

    let heightmap = read_f32_entries(&read_entry(&mut archive, "heightmap.f32")?);
    let temperature = read_f32_entries(&read_entry(&mut archive, "temperature.f32")?);
    let rainfall = read_f32_entries(&read_entry(&mut archive, "rainfall.f32")?);
    let volcanic_field = read_f32_entries(&read_entry(&mut archive, "volcanic_field.f32")?);
    let impact_field = read_f32_entries(&read_entry(&mut archive, "impact_field.f32")?);
    let strahler_order = read_entry(&mut archive, "strahler_order.bin")?;

    Ok(SaveData {
        params: SaveParams { gw, gh, seed, map_width_km, sea_level, world },
        fields: SaveFields { heightmap, temperature, rainfall, volcanic_field, impact_field, strahler_order },
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
