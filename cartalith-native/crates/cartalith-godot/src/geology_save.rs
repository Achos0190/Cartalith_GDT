//! The geology column in a project archive (`SAVEFILE_COMPAT.md` §8.4,
//! `GEOLOGY_FIRST_SCOPE.md` §2.6; GF-8's save slice, owner Rulings BH and BJ):
//! `cartalith_terrain::geology::GeologyColumn`, written beside the world
//! substrate (§8.3) so a reopened project keeps its rock, the regolith erosion
//! left on it, and every rock the user painted.
//!
//! # Why this exists
//!
//! Before it, a reopened project rebuilt its `WorldState` from the substrate
//! with `Geology::Absent(GeologyAbsent::Restored)`: every rock readout (the
//! Sample panel's rock keys, rock-styled LOD detail, rock-reading civ and
//! landmarks) went dark and a committed Rock paint was lost on reopen. Ruling
//! BJ requires the painted layer to persist and the change to be backward
//! compatible.
//!
//! # Every `GeologyColumn` member, accounted for
//!
//! | Member | Here |
//! |---|---|
//! | `rock_top`, `rock_sub`, `volcanic_setting` | stored, `u8`, range-checked on read |
//! | `contact` | stored, `f32`; NaN under every single-layer cell (the writer keeps the column's own NaN bits; the reader replaces any non-NaN value there with NaN, which is §8.4's "MUST ignore it there") |
//! | `regolith` | stored, `f32`; refused if negative or not finite |
//! | `painted` | stored as `rock_painted.u8` **only when `Some`**; the manifest's `painted` records which, so an absent mask reads back absent, never as zeros |
//! | the world's volcano table (`WorldState::volcanoes`, VA-1) | the manifest's optional `volcanoes` array, one row per edifice; absent (not empty) when the world has no table. Added 2026-10-24 under Ruling CW without a version bump: an older build ignores the unknown key, and an archive without it reloads with the table absent |
//! | `construction` | the manifest's `construction` (`"off"`, `"skipped"` with `construction_skip`, `"built"`); `Built`'s structural surface is `structural_surface.f32`. Not in §2.6's first table: added here because GF-8's re-erosion reads it (`rock_paint::reerode`'s `constructed`), so a column without it would re-erode a painted window differently after a reopen |
//!
//! # Honest refusal
//!
//! No member: `Ok(None)`, the normal answer for every archive written before
//! 2026-10-18, and the caller keeps `GeologyAbsent::Restored`. A member this
//! build cannot honour (unknown `version`, another grid size, a promised
//! raster missing or out of range), or geology rasters with no member (what an
//! older build's re-save leaves): `Err(reason)`, and the caller keeps
//! `Restored` and reports the reason. Never a partial column.
//!
//! Free of `godot`, so `tests/project_round_trip.rs` includes it by path.

use cartalith_engine::{Geology, WorldState};
use cartalith_io::{ProjectData, ProjectWrite, Raster};
use cartalith_terrain::geology::{setting_code, Construction, ConstructionSkip, GeologyColumn, NO_LAYER, ROCK_COUNT};
use serde::{Deserialize, Serialize};

/// The manifest member's own `version`. A reader refuses any other
/// (`SAVEFILE_COMPAT.md` §8.4): a newer writer that changed what the set means
/// must not be read as this one.
pub(crate) const GEOLOGY_VERSION: i64 = 1;

/// Archive member path, a format name (`SAVEFILE_COMPAT.md` §8.4), not a tunable: writer and reader share it, and changing it orphans every archive already saved. `GeologyColumn::rock_top`, `u8`, `0 ..= 10`.
const ROCK_TOP: &str = "rasters/rock_top.u8";
/// Archive member path (§8.4). `GeologyColumn::rock_sub`, `u8`, `0 ..= 10` or `255` (`NO_LAYER`).
const ROCK_SUB: &str = "rasters/rock_sub.u8";
/// Archive member path (§8.4). `GeologyColumn::contact`, `f32`, NaN under single-layer cells.
const CONTACT: &str = "rasters/rock_contact.f32";
/// Archive member path (§8.4). `GeologyColumn::regolith`, `f32`, `>= 0`.
const REGOLITH: &str = "rasters/regolith.f32";
/// Archive member path (§8.4). `GeologyColumn::volcanic_setting`, `u8`, `0 ..= 4` (`setting_code`).
const SETTING: &str = "rasters/volcanic_setting.u8";
/// Archive member path (§8.4). `GeologyColumn::painted`, `u8`, `0`/`1`; present only when the manifest says `painted`.
const PAINTED: &str = "rasters/rock_painted.u8";
/// Archive member path (§8.4). `Construction::Built`'s structural surface, `f32`; present only when the manifest says `construction: "built"`.
const STRUCTURAL: &str = "rasters/structural_surface.f32";

/// `project.json`'s `geology` member (`SAVEFILE_COMPAT.md` §8.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct GeologyManifest {
    pub version: i64,
    /// The grid the set was written for. Redundant with `world.grid_width` /
    /// `grid_height` in a well-formed archive; checked so a set spliced into
    /// another world's archive is refused by name rather than by a length
    /// mismatch further down.
    pub grid_width: usize,
    pub grid_height: usize,
    /// The geological clock τ (`GEOLOGY_FIRST_SCOPE.md` §4.12) of the
    /// parameters this world was saved with. Provenance for these grids, the
    /// reason `integrated_drainage` sits in the substrate member; `params.json`
    /// still carries the dial itself.
    pub age: f64,
    /// `rock_painted.u8` is present (`GeologyColumn::painted` was `Some`).
    pub painted: bool,
    /// `"off"`, `"skipped"` or `"built"` -- `GeologyColumn::construction`.
    pub construction: String,
    /// `ConstructionSkip`'s key, present exactly when `construction` is
    /// `"skipped"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction_skip: Option<String>,
    /// The world's volcanic edifices (`WorldState::volcanoes`, VA-1, Ruling
    /// CW), present exactly when the world had a table. Optional and
    /// additive: archives written before 2026-10-24 lack it and reload with no
    /// table (every volcano-colour consumer then falls back to the plain
    /// render by control flow), and a build from before ignores it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volcanoes: Option<Vec<VolcanoRow>>,
}

/// One row of the manifest's `volcanoes` table: a
/// `cartalith_terrain::PlacedEdifice`, with its setting and kind spelled as
/// the format keys of `cartalith_terrain::volcano_attr`. `kind` is `null` for
/// an edifice the reference model drew (it has no morphology).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct VolcanoRow {
    pub setting: String,
    pub kind: Option<String>,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub h: f64,
    pub age: f64,
    /// VA-5's own summit crater, `(floor, rim)` in radius fractions, when the
    /// edifice has one (`PlacedEdifice::crater`); absent otherwise and in
    /// every table written before VA-5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crater: Option<(f64, f64)>,
    /// The edifice's age in years under Ruling CX item 2's re-spread map
    /// (`PlacedEdifice::years`), for every edifice stamped since; absent in
    /// every table written before, which then reloads under Ruling CW's map
    /// (`volcano_attr::years`), so an old project keeps the ages, activity
    /// and colours it was saved with. Optional, so no version bump: an older
    /// reader ignores the key and reads the draw, as it always did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub years: Option<f64>,
    /// The rift's trend at the edifice, radians in `0..PI`
    /// (`PlacedEdifice::rift_bearing`, owner Ruling DB item 1, 2026-10-25):
    /// present for a rift edifice generated since the ruling, absent for
    /// every other row and in every table written before, which then reopens
    /// as it was saved (no fissure field). Optional, so no version bump: an
    /// older reader ignores the key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rift_bearing: Option<f64>,
}

/// The rows for `edifices`, in placement order (the order decides owner ties).
fn volcano_rows(edifices: &[cartalith_terrain::PlacedEdifice]) -> Vec<VolcanoRow> {
    use cartalith_terrain::volcano_attr::setting_key;
    edifices
        .iter()
        .map(|e| VolcanoRow {
            setting: setting_key(e.setting).to_string(),
            kind: e.kind.map(|k| k.key().to_string()),
            cx: e.cx,
            cy: e.cy,
            r: e.r,
            h: e.h,
            age: e.age,
            crater: e.crater,
            years: e.years,
            rift_bearing: e.rift_bearing,
        })
        .collect()
}

/// The table back from its rows, or why it is refused: an unknown setting or
/// kind, a non-finite number, a radius that is not positive, an age outside
/// `0..=1`, or an empty table (a writer never writes one). All or nothing --
/// a table missing a row would re-assign that row's cells to a neighbour.
pub(crate) fn volcanoes_from_rows(rows: &[VolcanoRow]) -> Result<Vec<cartalith_terrain::PlacedEdifice>, String> {
    use cartalith_terrain::volcano_attr::setting_from_key;
    if rows.is_empty() {
        return Err("the volcano table is empty, which no writer produces".to_string());
    }
    rows.iter()
        .enumerate()
        .map(|(i, r)| {
            let setting = setting_from_key(&r.setting).ok_or_else(|| format!("volcano {i}: unknown setting {:?}", r.setting))?;
            let kind = match r.kind.as_deref() {
                None => None,
                Some(k) => Some(cartalith_terrain::EdificeKind::from_key(k).ok_or_else(|| format!("volcano {i}: unknown kind {k:?}"))?),
            };
            if ![r.cx, r.cy, r.r, r.h, r.age].iter().all(|v| v.is_finite()) || r.r <= 0.0 || !(0.0..=1.0).contains(&r.age) {
                return Err(format!("volcano {i}: a position, radius, height or age that is not usable"));
            }
            if let Some((f, rm)) = r.crater
                && !(f.is_finite() && rm.is_finite() && 0.0 <= f && f <= rm && rm <= 1.0)
            {
                return Err(format!("volcano {i}: a crater that is not usable"));
            }
            if let Some(y) = r.years
                && !(y.is_finite() && y > 0.0)
            {
                return Err(format!("volcano {i}: an age in years that is not usable"));
            }
            if let Some(b) = r.rift_bearing
                && !(b.is_finite() && (0.0..std::f64::consts::PI).contains(&b))
            {
                return Err(format!("volcano {i}: a rift bearing that is not usable"));
            }
            Ok(cartalith_terrain::PlacedEdifice { setting, cx: r.cx, cy: r.cy, r: r.r, h: r.h, kind, age: r.age, years: r.years, crater: r.crater, rift_bearing: r.rift_bearing })
        })
        .collect()
}

/// The stored key for one `ConstructionSkip` -- a format name, so it is
/// spelled here once and never derived from `Debug`.
fn skip_key(s: ConstructionSkip) -> &'static str {
    match s {
        ConstructionSkip::NoLand => "no_land",
        ConstructionSkip::NoTwoLayerLand => "no_two_layer_land",
        ConstructionSkip::WorldStructureOn => "world_structure_on",
    }
}

/// The inverse of [`skip_key`]; `None` for a key this build does not know.
fn skip_of(key: &str) -> Option<ConstructionSkip> {
    [ConstructionSkip::NoLand, ConstructionSkip::NoTwoLayerLand, ConstructionSkip::WorldStructureOn]
        .into_iter()
        .find(|&s| skip_key(s) == key)
}

/// The geology rasters of `col` and the manifest member describing them, or
/// why they cannot be written. `Err` writes nothing: a partial set under a
/// manifest claiming it whole is worse than no set.
pub(crate) fn geology_rasters(
    col: &GeologyColumn,
    gw: usize,
    gh: usize,
    age: f64,
    volcanoes: Option<&[cartalith_terrain::PlacedEdifice]>,
) -> Result<(Vec<(&'static str, Raster)>, GeologyManifest), String> {
    let n = gw * gh;
    let mut out: Vec<(&'static str, Raster)> = Vec::new();
    let mut put = |path: &'static str, r: Raster| -> Result<(), String> {
        if r.len() != n {
            return Err(format!("{path}: {} cells for a {n}-cell grid", r.len()));
        }
        out.push((path, r));
        Ok(())
    };
    put(ROCK_TOP, Raster::U8(col.rock_top.clone()))?;
    put(ROCK_SUB, Raster::U8(col.rock_sub.clone()))?;
    put(CONTACT, Raster::F32(col.contact.clone()))?;
    put(REGOLITH, Raster::F32(col.regolith.clone()))?;
    put(SETTING, Raster::U8(col.volcanic_setting.clone()))?;
    if let Some(m) = col.painted.as_ref() {
        put(PAINTED, Raster::U8(m.clone()))?;
    }
    let (construction, construction_skip) = match &col.construction {
        Construction::Off => ("off", None),
        Construction::Skipped(s) => ("skipped", Some(skip_key(*s).to_string())),
        Construction::Built { structural_surface } => {
            put(STRUCTURAL, Raster::F32(structural_surface.clone()))?;
            ("built", None)
        }
    };
    let manifest = GeologyManifest {
        version: GEOLOGY_VERSION,
        grid_width: gw,
        grid_height: gh,
        age,
        painted: col.painted.is_some(),
        construction: construction.to_string(),
        construction_skip,
        volcanoes: volcanoes.filter(|v| !v.is_empty()).map(volcano_rows),
    };
    Ok((out, manifest))
}

/// Adds `ws`'s geology column to `write`. `Ok(false)` when the world has no
/// column (it writes nothing: §8.4's "a writer does not invent a column");
/// `Err` when it has one that cannot be written, which also writes nothing.
pub(crate) fn write_geology(ws: &WorldState, gw: usize, gh: usize, age: f64, write: &mut ProjectWrite<'_>) -> Result<bool, String> {
    let Some(col) = ws.geology.column() else { return Ok(false) };
    let (rasters, manifest) = geology_rasters(col, gw, gh, age, ws.volcanoes.as_deref().map(|v| v.as_slice()))?;
    let member = serde_json::to_value(manifest).map_err(|e| e.to_string())?;
    for (path, r) in rasters {
        write.raster(path, r);
    }
    write.geology = member;
    Ok(true)
}

/// The manifest member, if the archive has one this build can read.
fn manifest_of(data: &ProjectData) -> Result<Option<GeologyManifest>, String> {
    if data.geology.is_null() {
        // An older build's re-save carries the rasters (§6.2) and drops the
        // member. Name it rather than read the absence as "never had one".
        let stray: Vec<&str> = cartalith_io::GEOLOGY_RASTERS.iter().copied().filter(|p| data.raster(p).is_some()).collect();
        if !stray.is_empty() {
            return Err(format!(
                "the archive carries geology rasters ({}) but no geology member in project.json -- \
                 the shape an older build's re-save leaves -- so they were not read",
                stray.join(", ")
            ));
        }
        return Ok(None);
    }
    let m: GeologyManifest = serde_json::from_value(data.geology.clone())
        .map_err(|e| format!("project.json's geology member is unreadable ({e})"))?;
    if m.version != GEOLOGY_VERSION {
        return Err(format!("project.json's geology member is version {}; this build reads {GEOLOGY_VERSION}", m.version));
    }
    Ok(Some(m))
}

/// `true` when `path` holds a raster of the right element type.
fn has_u8(data: &ProjectData, path: &str) -> bool {
    matches!(data.raster(path), Some(Raster::U8(_)))
}
/// `has_u8` for an `f32` raster.
fn has_f32(data: &ProjectData, path: &str) -> bool {
    matches!(data.raster(path), Some(Raster::F32(_)))
}
/// Removes `path` as a `u8` raster; only called after [`has_u8`] held.
fn take_u8(data: &mut ProjectData, path: &str) -> Vec<u8> {
    match data.rasters.remove(path) {
        Some(Raster::U8(v)) => v,
        _ => unreachable!("validated before taking: {path}"),
    }
}
/// Removes `path` as an `f32` raster; only called after [`has_f32`] held.
fn take_f32(data: &mut ProjectData, path: &str) -> Vec<f32> {
    match data.rasters.remove(path) {
        Some(Raster::F32(v)) => v,
        _ => unreachable!("validated before taking: {path}"),
    }
}

/// Rebuilds the saved column from `data`, for a world of `gw × gh`.
///
/// `Ok(None)`: no member and no geology rasters -- every archive written
/// before 2026-10-18. `Err(reason)`: a set this build refuses; `data.rasters`
/// is left as it was read. `Ok(Some)`: the column, with every value in range.
///
/// Must never return a column it had to guess any part of.
pub(crate) fn column_from_project(data: &mut ProjectData, gw: usize, gh: usize) -> Result<Option<GeologyColumn>, String> {
    let Some(m) = manifest_of(data)? else { return Ok(None) };
    if (m.grid_width, m.grid_height) != (gw, gh) {
        return Err(format!(
            "the geology set was written for a {}x{} grid and this world is {gw}x{gh}",
            m.grid_width, m.grid_height
        ));
    }
    if !m.age.is_finite() {
        return Err("the geology member's age is not a number".to_string());
    }
    let construction_kind = match (m.construction.as_str(), m.construction_skip.as_deref()) {
        ("off", None) => 0u8,
        ("built", None) => 2,
        ("skipped", Some(k)) if skip_of(k).is_some() => 1,
        (c, s) => return Err(format!("the geology member's construction ({c:?}, {s:?}) is not one this build knows")),
    };
    let mut missing: Vec<&str> = Vec::new();
    for p in [ROCK_TOP, ROCK_SUB, SETTING] {
        if !has_u8(data, p) {
            missing.push(p);
        }
    }
    for p in [CONTACT, REGOLITH] {
        if !has_f32(data, p) {
            missing.push(p);
        }
    }
    if m.painted && !has_u8(data, PAINTED) {
        missing.push(PAINTED);
    }
    if construction_kind == 2 && !has_f32(data, STRUCTURAL) {
        missing.push(STRUCTURAL);
    }
    if !missing.is_empty() {
        return Err(format!("the saved geology column is incomplete: {} missing or damaged", missing.join(", ")));
    }
    // Range checks, all before any take, so a refusal leaves `data` as read.
    let u8s = |p: &str| match data.raster(p) {
        Some(Raster::U8(v)) => v,
        _ => unreachable!("checked above"),
    };
    let f32s = |p: &str| match data.raster(p) {
        Some(Raster::F32(v)) => v,
        _ => unreachable!("checked above"),
    };
    let max_rock = (ROCK_COUNT - 1) as u8;
    if let Some(bad) = u8s(ROCK_TOP).iter().find(|&&v| v > max_rock) {
        return Err(format!("{ROCK_TOP} holds {bad}, which is not a rock index (0..={max_rock})"));
    }
    if let Some(bad) = u8s(ROCK_SUB).iter().find(|&&v| v > max_rock && v != NO_LAYER) {
        return Err(format!("{ROCK_SUB} holds {bad}, which is neither a rock index nor {NO_LAYER} (no second layer)"));
    }
    if let Some(bad) = u8s(SETTING).iter().find(|&&v| v > setting_code::UNCLASSIFIED) {
        return Err(format!("{SETTING} holds {bad}, which is not a volcanic-setting code (0..={})", setting_code::UNCLASSIFIED));
    }
    if m.painted {
        if let Some(bad) = u8s(PAINTED).iter().find(|&&v| v > 1) {
            return Err(format!("{PAINTED} holds {bad}; the mask is 0 or 1"));
        }
    }
    {
        let sub = u8s(ROCK_SUB);
        if let Some(i) = f32s(CONTACT).iter().zip(sub).position(|(c, &s)| s != NO_LAYER && !c.is_finite()) {
            return Err(format!("{CONTACT} is not a number at cell {i}, which has a second layer"));
        }
    }
    if let Some(bad) = f32s(REGOLITH).iter().find(|v| !v.is_finite() || **v < 0.0) {
        return Err(format!("{REGOLITH} holds {bad}; regolith is a thickness, finite and >= 0"));
    }
    if construction_kind == 2 && f32s(STRUCTURAL).iter().any(|v| !v.is_finite()) {
        return Err(format!("{STRUCTURAL} holds a value that is not a number"));
    }

    let rock_sub = take_u8(data, ROCK_SUB);
    let mut contact = take_f32(data, CONTACT);
    // §8.4: a reader MUST ignore the contact under a single-layer cell. The
    // column's own invariant is NaN there, so a foreign writer's number is
    // replaced; this build's own NaN (whatever its payload bits) is kept, so
    // a self-written archive restores bit for bit.
    for (c, &s) in contact.iter_mut().zip(&rock_sub) {
        if s == NO_LAYER && !c.is_nan() {
            *c = f32::NAN;
        }
    }
    let construction = match construction_kind {
        0 => Construction::Off,
        1 => Construction::Skipped(skip_of(m.construction_skip.as_deref().unwrap_or_default()).expect("checked above")),
        _ => Construction::Built { structural_surface: take_f32(data, STRUCTURAL) },
    };
    Ok(Some(GeologyColumn {
        rock_top: take_u8(data, ROCK_TOP),
        rock_sub,
        contact,
        regolith: take_f32(data, REGOLITH),
        volcanic_setting: take_u8(data, SETTING),
        construction,
        painted: if m.painted { Some(take_u8(data, PAINTED)) } else { None },
    }))
}

/// Installs the archive's column on a world just rebuilt from its substrate,
/// and says what happened: `"complete"` (the column is restored), `"absent"`
/// (the archive has none: `ws.geology` stays `Absent(Restored)`) or
/// `"incomplete"` (refused, with the reason in `warning`).
///
/// The volcano table (VA-1) is read from the same member after the column is
/// accepted: present, it is installed as `ws.volcanoes`; absent (every archive
/// before 2026-10-24) it stays `None`; refused, the column is still restored
/// and the refusal is the warning -- a damaged table costs the volcano
/// colours, never the rock.
pub(crate) fn restore_into(ws: &mut WorldState, data: &mut ProjectData, gw: usize, gh: usize) -> (&'static str, Option<String>) {
    // Read before `column_from_project`, which only takes rasters: the member
    // itself is left in `data` either way.
    let rows: Option<Vec<VolcanoRow>> = manifest_of(data).ok().flatten().and_then(|m| m.volcanoes);
    match column_from_project(data, gw, gh) {
        Ok(Some(col)) => {
            ws.geology = Geology::Column(Box::new(col));
            match rows.as_deref().map(volcanoes_from_rows) {
                None => ("complete", None),
                Some(Ok(v)) => {
                    ws.volcanoes = Some(std::sync::Arc::new(v));
                    ("complete", None)
                }
                Some(Err(e)) => ("complete", Some(format!("{e} -- the world opened without its volcano table, so volcano colours are off on it"))),
            }
        }
        Ok(None) => ("absent", None),
        Err(e) => (
            "incomplete",
            Some(format!("{e} -- the world opened without its geology column, so rock readouts and Rock paint are off on it")),
        ),
    }
}
