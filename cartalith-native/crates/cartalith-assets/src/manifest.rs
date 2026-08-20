//! Asset-pack manifest: the `pack.json` / `pack.csv` data model, its parser,
//! its validation warnings, and schema-2 serialization.
//!
//! A direct port of the reference's own `parsePackCsv` (line 12093),
//! `parsePackManifest` (line 12113) and `packSummary` (line 12200), plus the
//! manifest half of `PackManifestBuilder.build()` (line ~26968). Everything
//! here is pure: no images are decoded, no ZIP is opened, nothing touches a
//! filesystem. Callers hand in the manifest text and the set of file names the
//! pack contains; that is exactly the information the reference's own parser
//! reads out of its in-memory `zip` object.

use crate::ordered_map::OrderedMap;
use crate::slots::{Family, PACK_BIOME_SLOTS, PACK_ICON_SLOTS, PACK_TERRAIN_SLOTS, PACK_TEX_SLOTS};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The manifest name the parser looks for first. JSON wins when both are
/// present.
pub const MANIFEST_JSON: &str = "pack.json";
/// The spreadsheet-friendly alternative manifest name.
pub const MANIFEST_CSV: &str = "pack.csv";

/// What went wrong reading a pack.
#[derive(Debug)]
pub enum PackError {
    /// Neither `pack.json` nor `pack.csv` is present.
    NoManifest,
    /// `pack.json` is not valid JSON, or is not shaped like a manifest.
    Json(serde_json::Error),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Verbatim the reference's own thrown message, so a UI that
            // surfaces it reads identically to the HTML app's.
            PackError::NoManifest => f.write_str("pack has no pack.json or pack.csv"),
            PackError::Json(e) => write!(f, "pack.json is not valid: {e}"),
        }
    }
}

impl std::error::Error for PackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PackError::Json(e) => Some(e),
            PackError::NoManifest => None,
        }
    }
}

impl From<serde_json::Error> for PackError {
    fn from(e: serde_json::Error) -> Self {
        PackError::Json(e)
    }
}

/// One path, or several. The manifest accepts a bare string wherever a variant
/// list is expected (`"hill": "icons/hill_01.png"`), which the reference
/// normalizes with `if(typeof v==='string') v=[v]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Paths {
    /// A single path written as a bare string.
    One(String),
    /// An explicit variant list.
    Many(Vec<String>),
}

impl Paths {
    /// The paths, normalized to a slice.
    pub fn as_slice(&self) -> &[String] {
        match self {
            Paths::One(p) => std::slice::from_ref(p),
            Paths::Many(v) => v,
        }
    }
}

/// The three families nested under a manifest's `structures` object, exactly
/// as written on disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawStructures {
    /// `structures.settlement`.
    #[serde(default, skip_serializing_if = "OrderedMap::is_empty")]
    pub settlement: OrderedMap<Option<Paths>>,
    /// `structures.trait` (`trait` is a Rust keyword, hence the field name).
    #[serde(
        default,
        rename = "trait",
        skip_serializing_if = "OrderedMap::is_empty"
    )]
    pub traits: OrderedMap<Option<Paths>>,
    /// `structures.poi`.
    #[serde(default, skip_serializing_if = "OrderedMap::is_empty")]
    pub poi: OrderedMap<Option<Paths>>,
}

impl RawStructures {
    /// Whether all three families are absent.
    pub fn is_empty(&self) -> bool {
        self.settlement.is_empty() && self.traits.is_empty() && self.poi.is_empty()
    }

    fn family(&self, fam: Family) -> &OrderedMap<Option<Paths>> {
        match fam {
            Family::Settlement => &self.settlement,
            Family::Trait => &self.traits,
            Family::Poi => &self.poi,
            other => panic!("{} is not a structures family", other.key()),
        }
    }
}

/// A manifest exactly as authored — unvalidated, key order preserved, unknown
/// slots still present.
///
/// Deserialize this from `pack.json`, or build it from `pack.csv` with
/// [`parse_pack_csv`]; then run [`parse_pack_manifest`] to get the validated
/// [`PackManifest`] the renderer consumes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawManifest {
    /// Format version: 1 (textures + icons) or 2 (the compiler superset).
    /// Advisory only — the parser reads whatever sections are present, which
    /// is what lets a schema-2 pack load unchanged in a schema-1 consumer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<u32>,
    /// Pack name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Pack author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Pack licence, as free text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// `textures` — one image per splat channel.
    #[serde(default)]
    pub textures: OrderedMap<Option<String>>,
    /// `icons` — 1..N variants per feature slot.
    #[serde(default)]
    pub icons: OrderedMap<Option<Paths>>,
    /// `biomes` — one ground tile per painted biome index.
    #[serde(default, skip_serializing_if = "OrderedMap::is_empty")]
    pub biomes: OrderedMap<Option<String>>,
    /// `terrains` — one ground tile per painted terrain index.
    #[serde(default, skip_serializing_if = "OrderedMap::is_empty")]
    pub terrains: OrderedMap<Option<String>>,
    /// `structures` — settlement / trait / POI symbols.
    #[serde(default, skip_serializing_if = "RawStructures::is_empty")]
    pub structures: RawStructures,
    /// `custom` — free-form icon sets, `set -> slot -> [paths]`.
    #[serde(default, skip_serializing_if = "OrderedMap::is_empty")]
    pub custom: OrderedMap<OrderedMap<Option<Paths>>>,
}

impl RawManifest {
    fn single_section(&self, fam: Family) -> &OrderedMap<Option<String>> {
        match fam {
            Family::Textures => &self.textures,
            Family::Biomes => &self.biomes,
            Family::Terrains => &self.terrains,
            other => panic!("{} is not a single-image family", other.key()),
        }
    }
}

/// The validated `structures` section of a [`PackManifest`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Structures {
    /// Settlement pins that resolved to a real file.
    pub settlement: OrderedMap<Vec<String>>,
    /// Trait overlays that resolved to a real file.
    pub traits: OrderedMap<Vec<String>>,
    /// POI markers that resolved to a real file.
    pub poi: OrderedMap<Vec<String>>,
}

impl Structures {
    /// Total number of structure slots carrying art.
    pub fn len(&self) -> usize {
        self.settlement.len() + self.traits.len() + self.poi.len()
    }

    /// Whether no structure family carries art.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The map for one of [`Family::STRUCTURES`].
    ///
    /// # Panics
    /// If `fam` is not a structures family.
    pub fn family(&self, fam: Family) -> &OrderedMap<Vec<String>> {
        match fam {
            Family::Settlement => &self.settlement,
            Family::Trait => &self.traits,
            Family::Poi => &self.poi,
            other => panic!("{} is not a structures family", other.key()),
        }
    }

    fn family_mut(&mut self, fam: Family) -> &mut OrderedMap<Vec<String>> {
        match fam {
            Family::Settlement => &mut self.settlement,
            Family::Trait => &mut self.traits,
            Family::Poi => &mut self.poi,
            other => panic!("{} is not a structures family", other.key()),
        }
    }
}

/// A manifest after validation: only slots in the frozen vocabulary whose files
/// actually exist in the pack, plus the [`warnings`](PackManifest::warnings)
/// explaining everything that was dropped.
///
/// This is the reference's `parsePackManifest` return value, field for field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackManifest {
    /// Pack name, defaulted to `"Asset pack"`.
    pub name: String,
    /// Pack author, defaulted to empty.
    pub author: String,
    /// Pack licence, defaulted to empty.
    pub license: String,
    /// Splat-channel textures, keyed by slot, in [`PACK_TEX_SLOTS`] order.
    pub textures: OrderedMap<String>,
    /// Feature-icon variants, keyed by slot, in [`PACK_ICON_SLOTS`] order.
    pub icons: OrderedMap<Vec<String>>,
    /// Painted-biome ground tiles, in [`PACK_BIOME_SLOTS`] order.
    pub biomes: OrderedMap<String>,
    /// Painted-terrain ground tiles, in [`PACK_TERRAIN_SLOTS`] order.
    pub terrains: OrderedMap<String>,
    /// Settlement / trait / POI symbols.
    pub structures: Structures,
    /// Free-form icon sets, in manifest document order.
    pub custom: OrderedMap<OrderedMap<Vec<String>>>,
    /// Everything the parser dropped or could not use, in the order it was
    /// found. **Not fatal** — a pack with warnings still loads; the reference
    /// surfaces the count next to the import summary rather than refusing.
    pub warnings: Vec<String>,
}

impl PackManifest {
    /// Whether the pack carries no usable art at all.
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
            && self.icons.is_empty()
            && self.biomes.is_empty()
            && self.terrains.is_empty()
            && self.structures.is_empty()
            && self.custom.is_empty()
    }

    /// The variant paths for a frozen-vocabulary slot, or `None` when the pack
    /// has no art there (so the caller falls back to procedural art for that
    /// slot only — the format's per-slot fallback rule).
    ///
    /// # Panics
    /// If `fam` is [`Family::Custom`]; use [`custom_paths`](Self::custom_paths).
    pub fn slot_paths(&self, fam: Family, slot_id: &str) -> Option<&[String]> {
        match fam {
            Family::Textures => self.textures.get(slot_id).map(std::slice::from_ref),
            Family::Biomes => self.biomes.get(slot_id).map(std::slice::from_ref),
            Family::Terrains => self.terrains.get(slot_id).map(std::slice::from_ref),
            Family::Icons => self.icons.get(slot_id).map(Vec::as_slice),
            Family::Settlement | Family::Trait | Family::Poi => {
                self.structures.family(fam).get(slot_id).map(Vec::as_slice)
            }
            Family::Custom => panic!("use custom_paths() for the custom family"),
        }
    }

    /// The variant paths for a custom-set asset.
    pub fn custom_paths(&self, set_name: &str, slot_id: &str) -> Option<&[String]> {
        self.custom.get(set_name)?.get(slot_id).map(Vec::as_slice)
    }

    /// Every distinct file the pack's manifest points at, deduplicated and
    /// sorted — what a loader needs to decode.
    pub fn referenced_files(&self) -> BTreeSet<&str> {
        let mut out = BTreeSet::new();
        out.extend(self.textures.values().map(String::as_str));
        out.extend(self.biomes.values().map(String::as_str));
        out.extend(self.terrains.values().map(String::as_str));
        for v in self.icons.values() {
            out.extend(v.iter().map(String::as_str));
        }
        for fam in Family::STRUCTURES {
            for v in self.structures.family(fam).values() {
                out.extend(v.iter().map(String::as_str));
            }
        }
        for set in self.custom.values() {
            for v in set.values() {
                out.extend(v.iter().map(String::as_str));
            }
        }
        out
    }

    /// Re-emit this manifest as a schema-2 [`RawManifest`], ready to serialize
    /// back into a `pack.json`.
    ///
    /// The result is what an export writes: only resolved slots, only real
    /// paths, no unknown keys. Round-tripping a parsed manifest through this
    /// and back is therefore lossless *for the art that survived validation* —
    /// warnings and dropped slots are, by design, not carried over.
    pub fn to_raw(&self) -> RawManifest {
        let many = |m: &OrderedMap<Vec<String>>| -> OrderedMap<Option<Paths>> {
            m.iter()
                .map(|(k, v)| (k.to_string(), Some(Paths::Many(v.clone()))))
                .collect()
        };
        let single = |m: &OrderedMap<String>| -> OrderedMap<Option<String>> {
            m.iter()
                .map(|(k, v)| (k.to_string(), Some(v.clone())))
                .collect()
        };
        RawManifest {
            schema: Some(2),
            name: Some(self.name.clone()),
            author: Some(self.author.clone()),
            license: Some(self.license.clone()),
            textures: single(&self.textures),
            icons: many(&self.icons),
            biomes: single(&self.biomes),
            terrains: single(&self.terrains),
            structures: RawStructures {
                settlement: many(&self.structures.settlement),
                traits: many(&self.structures.traits),
                poi: many(&self.structures.poi),
            },
            custom: self
                .custom
                .iter()
                .map(|(set, slots)| (set.to_string(), many(slots)))
                .collect(),
        }
    }

    /// The manifest as `pack.json` text, formatted the way the reference's own
    /// exporter writes it (`JSON.stringify(manifest, null, 2)`).
    pub fn to_pack_json(&self) -> String {
        serde_json::to_string_pretty(&self.to_raw()).expect("manifest serialization cannot fail")
    }
}

/// The one-line human summary the reference shows after an import
/// (`packSummary`, line 12200).
pub fn pack_summary(p: &PackManifest) -> String {
    let t = p.textures.len();
    let ic = PACK_ICON_SLOTS
        .iter()
        .filter_map(|s| {
            p.icons
                .get(s)
                .map(|v| format!("{}×{}", s.replacen("tree_", "", 1), v.len()))
        })
        .collect::<Vec<_>>()
        .join(" ");
    let struct_n = p.structures.len();
    let ground_n = p.biomes.len() + p.terrains.len();
    let custom_slot_n: usize = p.custom.values().map(OrderedMap::len).sum();

    let name = if p.name.is_empty() { "Pack" } else { &p.name };
    let license = if p.license.is_empty() {
        "license?"
    } else {
        &p.license
    };
    let mut out = format!("{name} · {license} — {t} textures");
    if ground_n > 0 {
        out.push_str(&format!(" · {ground_n} biome/terrain ground"));
    }
    if !ic.is_empty() {
        out.push_str(&format!(" · {ic}"));
    }
    if struct_n > 0 {
        out.push_str(&format!(
            " · {struct_n} structure sprite{}",
            if struct_n > 1 { "s" } else { "" }
        ));
    }
    if custom_slot_n > 0 {
        out.push_str(&format!(
            " · {custom_slot_n} custom icon{}",
            if custom_slot_n > 1 { "s" } else { "" }
        ));
    }
    out
}

/// `parseFloat` as JavaScript defines it: the longest numeric prefix, or `None`
/// where JS would return `NaN`. Trailing junk is ignored (`"2x"` -> `2`), which
/// a plain `str::parse` would reject.
fn js_parse_float(s: &str) -> Option<f64> {
    let s = s.trim_start();
    let mut best = None;
    for (i, _) in s.char_indices().skip(1) {
        if let Ok(v) = s[..i].parse::<f64>() {
            best = Some(v);
        }
    }
    if let Ok(v) = s.parse::<f64>() {
        best = Some(v);
    }
    best.filter(|v| !v.is_nan())
}

/// Parse the spreadsheet-friendly `pack.csv` form into a [`RawManifest`]
/// (reference `parsePackCsv`, line 12093).
///
/// Header row, blank lines, CRLF and surrounding whitespace are all tolerated.
/// Columns are `type,slot,file,variant`; `type` is one of `texture` / `icon` /
/// `biome` / `terrain`, and `variant` orders icon variants (rows with no usable
/// variant number sort last, keeping their relative order). Rows naming a slot
/// outside the frozen vocabulary are dropped **silently** — unlike the JSON
/// path, which warns; that asymmetry is the reference's own and is preserved
/// here rather than tidied.
///
/// Note the CSV format predates `structures` and `custom` and cannot express
/// them, nor a pack name/author/licence.
pub fn parse_pack_csv(text: &str) -> RawManifest {
    let mut out = RawManifest::default();
    // slot -> [(variant, file)], in first-appearance order
    let mut ico_tmp: Vec<(String, Vec<(f64, String)>)> = Vec::new();

    for raw_line in text.split('\n') {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        let ty = cells[0].to_lowercase();
        if ty == "type" || ty.is_empty() {
            continue; // header / blank
        }
        let slot = cells.get(1).copied().unwrap_or("");
        let file = cells.get(2).copied().unwrap_or("");
        if slot.is_empty() || file.is_empty() {
            continue;
        }
        // A row naming a slot outside its family's frozen vocabulary fails its
        // guard and falls through to the catch-all -- dropped silently, which
        // is what the reference does here (only the JSON path warns).
        match ty.as_str() {
            "texture" if PACK_TEX_SLOTS.contains(&slot) => {
                out.textures.insert(slot, Some(file.to_string()));
            }
            "biome" if PACK_BIOME_SLOTS.contains(&slot) => {
                out.biomes.insert(slot, Some(file.to_string()));
            }
            "terrain" if PACK_TERRAIN_SLOTS.contains(&slot) => {
                out.terrains.insert(slot, Some(file.to_string()));
            }
            "icon" if PACK_ICON_SLOTS.contains(&slot) => {
                let v = cells.get(3).and_then(|c| js_parse_float(c)).unwrap_or(1e9);
                match ico_tmp.iter_mut().find(|(k, _)| k == slot) {
                    Some((_, list)) => list.push((v, file.to_string())),
                    None => ico_tmp.push((slot.to_string(), vec![(v, file.to_string())])),
                }
            }
            _ => {}
        }
    }

    for (slot, mut list) in ico_tmp {
        // Stable sort, matching `Array.prototype.sort`'s guaranteed stability:
        // two rows with no variant number keep the order they were written in.
        list.sort_by(|a, b| a.0.total_cmp(&b.0));
        out.icons.insert(
            slot,
            Some(Paths::Many(list.into_iter().map(|(_, f)| f).collect())),
        );
    }
    out
}

/// Validate an authored manifest against the frozen vocabulary and the pack's
/// actual file list (reference `parsePackManifest`, line 12113).
///
/// `files` is the set of entry names the pack contains, ZIP-root-relative — the
/// reference reads this straight off its in-memory `zip` object. A slot whose
/// declared file is absent is dropped with a warning; a slot outside the frozen
/// vocabulary is dropped with a warning; a slot the manifest simply omits is
/// silently absent, which is the format's per-slot procedural fallback.
pub fn parse_pack_manifest(m: &RawManifest, files: &BTreeSet<String>) -> PackManifest {
    let mut out = PackManifest {
        name: non_empty(m.name.as_deref(), "Asset pack"),
        author: non_empty(m.author.as_deref(), ""),
        license: non_empty(m.license.as_deref(), ""),
        ..Default::default()
    };
    // `has` in the reference: a falsy (empty) path never resolves.
    let has = |p: &str| !p.is_empty() && files.contains(p);

    // ---- single-image families: textures, then the two painted-layer grounds.
    for fam in [Family::Textures, Family::Biomes, Family::Terrains] {
        let raw = m.single_section(fam);
        // The section label the warnings use is the manifest key itself
        // ("texture" singular for the splat channels, matching the reference).
        let label = if fam == Family::Textures {
            "texture"
        } else {
            fam.key()
        };
        for slot in fam.slots() {
            let Some(Some(p)) = raw.get(slot) else {
                continue;
            };
            if has(p) {
                match fam {
                    Family::Textures => out.textures.insert(*slot, p.clone()),
                    Family::Biomes => out.biomes.insert(*slot, p.clone()),
                    _ => out.terrains.insert(*slot, p.clone()),
                }
            } else {
                out.warnings
                    .push(format!("{label} {slot}: file missing ({p})"));
            }
        }
        for k in raw.keys() {
            if !fam.has_slot(k) {
                out.warnings.push(format!("unknown {label} slot: {k}"));
            }
        }
    }

    // ---- feature icons: 1..N variants, missing variants dropped individually.
    for slot in PACK_ICON_SLOTS {
        let Some(Some(v)) = m.icons.get(slot) else {
            continue;
        };
        let kept = keep_existing(v, &has, &format!("icon {slot}"), &mut out.warnings);
        if !kept.is_empty() {
            out.icons.insert(slot, kept);
        }
    }
    for k in m.icons.keys() {
        if !Family::Icons.has_slot(k) {
            out.warnings.push(format!("unknown icon slot: {k}"));
        }
    }

    // ---- structure sprites. Note the iteration order here is the reference's
    // own settlement/poi/trait, which is *not* the settlement/trait/poi order
    // the exporter writes them in; warning order follows this one.
    for fam in [Family::Settlement, Family::Poi, Family::Trait] {
        let raw = m.structures.family(fam);
        for slot in fam.slots() {
            let Some(Some(v)) = raw.get(slot) else {
                continue;
            };
            let kept = keep_existing(v, &has, &format!("{} {slot}", fam.key()), &mut out.warnings);
            if !kept.is_empty() {
                out.structures.family_mut(fam).insert(*slot, kept);
            }
        }
        for k in raw.keys() {
            if !fam.has_slot(k) {
                out.warnings
                    .push(format!("unknown {} slot: {k}", fam.key()));
            }
        }
    }

    // ---- free-form custom sets: open vocabulary, so nothing is "unknown".
    for (set_name, raw_set) in m.custom.iter() {
        let mut out_set: OrderedMap<Vec<String>> = OrderedMap::new();
        for (slot_id, v) in raw_set.iter() {
            let Some(v) = v else { continue };
            let kept = keep_existing(
                v,
                &has,
                &format!("custom {set_name}/{slot_id}"),
                &mut out.warnings,
            );
            if !kept.is_empty() {
                out_set.insert(slot_id, kept);
            }
        }
        if !out_set.is_empty() {
            out.custom.insert(set_name, out_set);
        }
    }

    // ---- families the live renderer does not consume yet. Reported rather
    // than silently dropped, so "Import asset pack" cannot claim an
    // unconditional success it did not deliver.
    let mut unused: Vec<&str> = Vec::new();
    if !m.structures.traits.is_empty() {
        unused.push("trait");
    }
    if !m.biomes.is_empty() {
        unused.push("biomes");
    }
    if !m.terrains.is_empty() {
        unused.push("terrains");
    }
    if !unused.is_empty() {
        out.warnings.push(format!(
            "{} pack section(s) not yet used by the live map ({})",
            unused.len(),
            unused.join(", ")
        ));
    }
    out
}

/// Read a whole pack's entries the way the reference's importer does: take
/// `pack.json` if present, else `pack.csv`, then validate against the entry
/// names.
///
/// This is the direct equivalent of `parsePackManifest(zip)`; the ZIP itself is
/// somebody else's problem, which is what keeps this crate free of an archive
/// dependency.
pub fn parse_pack_entries(entries: &BTreeMap<String, Vec<u8>>) -> Result<PackManifest, PackError> {
    let names: BTreeSet<String> = entries.keys().cloned().collect();
    let raw = if let Some(bytes) = entries.get(MANIFEST_JSON) {
        serde_json::from_str::<RawManifest>(&String::from_utf8_lossy(bytes))?
    } else if let Some(bytes) = entries.get(MANIFEST_CSV) {
        parse_pack_csv(&String::from_utf8_lossy(bytes))
    } else {
        return Err(PackError::NoManifest);
    };
    Ok(parse_pack_manifest(&raw, &names))
}

fn non_empty(v: Option<&str>, fallback: &str) -> String {
    match v {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => fallback.to_string(),
    }
}

fn keep_existing(
    v: &Paths,
    has: &dyn Fn(&str) -> bool,
    label: &str,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    v.as_slice()
        .iter()
        .filter(|p| {
            if has(p) {
                true
            } else {
                warnings.push(format!("{label}: file missing ({p})"));
                false
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn js_parse_float_matches_javascript() {
        assert_eq!(js_parse_float("1"), Some(1.0));
        assert_eq!(js_parse_float(" 2.5 "), Some(2.5));
        assert_eq!(js_parse_float("3px"), Some(3.0)); // str::parse would reject
        assert_eq!(js_parse_float("-4"), Some(-4.0));
        assert_eq!(js_parse_float(""), None);
        assert_eq!(js_parse_float("abc"), None);
    }

    #[test]
    fn an_omitted_slot_is_silently_absent() {
        // The per-slot procedural fallback: an icons-only pack is legal, and
        // says nothing about the six textures it does not carry.
        let raw: RawManifest =
            serde_json::from_str(r#"{"icons":{"shrub":["icons/s.png"]}}"#).unwrap();
        let m = parse_pack_manifest(&raw, &files(&["icons/s.png"]));
        assert!(m.warnings.is_empty());
        assert!(m.textures.is_empty());
        assert_eq!(m.slot_paths(Family::Icons, "shrub").unwrap().len(), 1);
        assert!(m.slot_paths(Family::Icons, "cactus").is_none());
    }

    #[test]
    fn a_null_valued_slot_is_skipped_without_a_warning() {
        // `if(p==null) continue` in the reference: an explicit null is "no art
        // here", not a broken reference.
        let raw: RawManifest =
            serde_json::from_str(r#"{"textures":{"grass":null},"icons":{"hill":null}}"#).unwrap();
        let m = parse_pack_manifest(&raw, &files(&[]));
        assert!(m.warnings.is_empty());
        assert!(m.is_empty());
    }

    #[test]
    fn referenced_files_lists_every_family() {
        let raw: RawManifest = serde_json::from_str(
            r#"{"textures":{"grass":"t/g.png"},"biomes":{"jungle":"b/j.png"},
                "terrains":{"paved":"r/p.png"},"icons":{"hill":["i/h.png"]},
                "structures":{"settlement":{"town":["s/t.png"]},"trait":{"port":["s/p.png"]},
                              "poi":{"cave":["s/c.png"]}},
                "custom":{"Naval":{"anchor":["c/a.png"]}}}"#,
        )
        .unwrap();
        let all = [
            "t/g.png", "b/j.png", "r/p.png", "i/h.png", "s/t.png", "s/p.png", "s/c.png", "c/a.png",
        ];
        let m = parse_pack_manifest(&raw, &files(&all));
        assert_eq!(
            m.referenced_files(),
            files(&all).iter().map(String::as_str).collect()
        );
        assert_eq!(m.structures.len(), 3);
        assert_eq!(m.custom_paths("Naval", "anchor").unwrap(), ["c/a.png"]);
        assert!(m.custom_paths("Naval", "lighthouse").is_none());
        assert!(m.custom_paths("Mining", "anchor").is_none());
    }

    #[test]
    fn to_raw_round_trips_through_json() {
        let raw: RawManifest = serde_json::from_str(
            r#"{"schema":2,"name":"P","author":"a","license":"CC0",
                "textures":{"grass":"t/g.png"},"icons":{"hill":"i/h.png"},
                "biomes":{"jungle":"b/j.png"},
                "structures":{"trait":{"port":["s/p.png"]}},
                "custom":{"Naval":{"anchor":["c/a.png","c/a2.png"]}}}"#,
        )
        .unwrap();
        let all = [
            "t/g.png", "i/h.png", "b/j.png", "s/p.png", "c/a.png", "c/a2.png",
        ];
        let first = parse_pack_manifest(&raw, &files(&all));

        // Serialize the validated manifest, parse it back, and require the
        // result to be identical -- what export -> re-import must guarantee.
        let json = first.to_pack_json();
        let reparsed: RawManifest = serde_json::from_str(&json).unwrap();
        let second = parse_pack_manifest(&reparsed, &files(&all));
        assert_eq!(first, second);
        // The bare-string `icons.hill` came back as a one-element list.
        assert_eq!(second.icons.get("hill").unwrap(), &["i/h.png".to_string()]);
        assert!(json.contains("\"schema\": 2"));
    }

    #[test]
    fn empty_sections_are_omitted_but_textures_and_icons_are_always_written() {
        // Mirrors PackManifestBuilder, which seeds `textures:{}` and `icons:{}`
        // unconditionally and adds the other sections only when used.
        let json = PackManifest {
            name: "P".into(),
            ..Default::default()
        }
        .to_pack_json();
        assert!(json.contains("\"textures\""));
        assert!(json.contains("\"icons\""));
        assert!(!json.contains("\"biomes\""));
        assert!(!json.contains("\"structures\""));
        assert!(!json.contains("\"custom\""));
    }

    #[test]
    fn parse_pack_entries_picks_json_over_csv_and_reports_neither() {
        let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        entries.insert("art.png".into(), vec![1]);
        assert!(matches!(
            parse_pack_entries(&entries),
            Err(PackError::NoManifest)
        ));
        assert_eq!(
            PackError::NoManifest.to_string(),
            "pack has no pack.json or pack.csv"
        );

        entries.insert(
            "pack.csv".into(),
            b"type,slot,file,variant\ntexture,grass,art.png,".to_vec(),
        );
        assert_eq!(parse_pack_entries(&entries).unwrap().textures.len(), 1);

        entries.insert(
            "pack.json".into(),
            br#"{"name":"J","icons":{"hill":["art.png"]}}"#.to_vec(),
        );
        let m = parse_pack_entries(&entries).unwrap();
        assert_eq!(m.name, "J");
        assert!(m.textures.is_empty(), "JSON must win over CSV");
    }

    #[test]
    fn malformed_pack_json_is_an_error_not_a_panic() {
        let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        entries.insert("pack.json".into(), b"{not json".to_vec());
        assert!(matches!(
            parse_pack_entries(&entries),
            Err(PackError::Json(_))
        ));
    }

    #[test]
    fn csv_cannot_express_structures_or_custom() {
        let raw = parse_pack_csv("icon,mountain,i/m.png,1\nstructure,hamlet,s/h.png,1");
        assert!(raw.structures.is_empty());
        assert!(raw.custom.is_empty());
        assert!(raw.name.is_none());
        assert_eq!(raw.icons.len(), 1);
    }
}
