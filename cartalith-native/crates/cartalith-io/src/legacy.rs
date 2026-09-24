//! The flat layout's project records, normalised — owner **Ruling AU**
//! (`LARGE_ITEM_RULINGS.md`, 2026-09-24): *"Legacy flat `.zip` archives import
//! their settlements, labels and icons, mapped onto today's settlements,
//! annotations and icons, with a report of anything that did not map."*
//!
//! A flat archive (`SAVEFILE_COMPAT.md` §15) carries its project layer inside
//! `params.json`'s `state` object, in the HTML app's own vocabulary (§15.1).
//! This module reads the three record kinds the ruling names — plus the
//! faction roster and territory the settlements refer to — into plain,
//! engine-neutral records, and says in [`LegacyProject::unmapped`] what it
//! saw and could not carry. It makes every substitution here, so that the
//! report is written in one place; the caller (`cartalith-godot`'s
//! `legacy_import`) only has to translate these records into its own types.
//!
//! **What this does not do, and says so.** Ways, journeys, recorded years,
//! the older `state.roads` network, hand-painted cartography cells and the
//! region marquee are outside the ruling; each one the archive actually
//! carries gets a line in `unmapped` rather than being dropped in silence.
//! Points of interest share `state.places` with settlements and this build
//! has no hand-placed point-of-interest record, so they are reported too:
//! §15.1 forbids inventing a settlement from a point that is not one.
//!
//! The shapes read here are the ones the reference writes: the Settlement
//! tool's `{x,y,name,kind,faction,pop,traits}` (`_civDropPlace`, v2.11 line
//! 16534), Auto-populate's `{…,klass,category:'settlement',…}` (25961,
//! 26173), the POI tool's (16558), the bare `{x,y}` of "designate places"
//! (9801); labels `{x,y,name,angle,arc,size}` plus the editor's `sizeMode`,
//! `font`, `color` (9810, 17297-17319); icons `{x,y,fam,slot,scale}` and
//! `custom`'s `set` (9822, 15560); and `_civSyncToState`'s `state.civ`
//! (26633). `_treeRead` (12751) is the reference's own mapping in the other
//! direction, and every default below is the one it uses.

use serde_json::Value;
use std::collections::BTreeMap;

/// §9.1's closed settlement-tier set — this port's `SettlementKind`.
pub const SETTLEMENT_KINDS: [&str; 6] = ["metropolis", "capital", "city", "town", "village", "hamlet"];

/// The four entries of the reference's `CIV_SETTLEMENT_CLASSES` (v2.11 line
/// 15157) that are settlements there and have no tier here. Read as `town`
/// (§9.1's rule for an unrecognised tier) and reported.
pub const REFERENCE_ONLY_CLASSES: [&str; 4] = ["monastery", "fortress", "university", "industrial"];

/// §11.2's icon families — the only `fam` values the reference writes.
pub const ICON_FAMILIES: [&str; 4] = ["settlement", "feature", "poi", "custom"];

/// The reference's roster length when a save has no `factionNames`: its base
/// `CIV_FACTIONS` literal, "Unclaimed" plus six. A settlement's `faction` is
/// range-checked against this in that case.
pub const REFERENCE_DEFAULT_ROSTER_LEN: usize = 7;

const PLACE_MEMBERS: [&str; 18] = [
    "x", "y", "tid", "name", "kind", "klass", "category", "faction", "pop", "traits", "history",
    "specialisation", "umAge", "umWalls", "capital", "coastal", "suit", "villageAddon",
];
const LABEL_MEMBERS: [&str; 9] = ["x", "y", "name", "angle", "arc", "size", "sizeMode", "font", "color"];
const ICON_MEMBERS: [&str; 6] = ["x", "y", "fam", "slot", "set", "scale"];
/// `state.civ` members this module reads, or reports by name below.
const CIV_MEMBERS: [&str; 11] = [
    "territory", "timeline", "ways", "routes", "journeys", "year", "factionNames", "factionCulture",
    "factionReligion", "factionGovernment", "factionAgTech",
];

/// One settlement, in this port's vocabulary. Every member is a value the
/// archive stated, or the documented substitute for one it did not — each
/// substitution is counted into [`LegacyProject::unmapped`].
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySettlement {
    /// The archive's `tid` when it carried a unique positive one; otherwise
    /// assigned past every id present (the reference assigns lazily too).
    pub tid: u64,
    pub x: usize,
    pub y: usize,
    pub name: String,
    pub population: u32,
    /// Range-checked against the roster; out of range reads as `0`
    /// (Unclaimed), §9.1's clamp.
    pub faction: i32,
    /// One of [`SETTLEMENT_KINDS`].
    pub kind: &'static str,
    pub capital: bool,
    pub coastal: bool,
    pub suitability: f64,
    pub village_seeded: bool,
    pub traits: Vec<String>,
    pub history: Option<String>,
    pub specialisation: Option<String>,
    pub age: Option<u32>,
    pub walls: Option<bool>,
}

/// One roster row. `None` in a column means the archive's parallel array did
/// not reach this index — the caller supplies its own default and the
/// report already says so. There is no colour: the flat layout has none
/// (§15.3).
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFaction {
    pub name: String,
    pub culture: Option<String>,
    pub religion: Option<String>,
    pub government: Option<String>,
    pub ag_tech: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLabel {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub angle: f64,
    pub arc: f64,
    pub size: f64,
    /// Absent is the renderer's default, not a concrete font (§11.1).
    pub font: Option<String>,
    pub color: Option<String>,
    /// `sizeMode: 'fixed'`; anything else is `zoom`, the reference's default.
    pub fixed_size: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIcon {
    pub x: f64,
    pub y: f64,
    /// One of [`ICON_FAMILIES`].
    pub family: &'static str,
    pub slot: String,
    /// `custom` only.
    pub set: Option<String>,
    pub scale: f64,
}

/// Everything [`read_legacy`] carried out of a flat archive's `state`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LegacyProject {
    pub settlements: Vec<LegacySettlement>,
    /// Empty when the archive carried no `factionNames` — the reference then
    /// uses its default roster, and so does the caller.
    pub factions: Vec<LegacyFaction>,
    /// `state.civ.territory` decoded from its sparse `[i, faction, …]` pairs,
    /// or `None` when the archive carried no claim at all.
    pub territory: Option<Vec<i32>>,
    pub labels: Vec<LegacyLabel>,
    pub icons: Vec<LegacyIcon>,
    /// Ruling AU's report: one line per kind of thing that was present and
    /// did not map, or was mapped by substitution. Empty for an archive with
    /// nothing to report — which is every archive that carries no records.
    pub unmapped: Vec<String>,
}

impl LegacyProject {
    /// Whether the archive carried anything this module imports.
    pub fn is_empty(&self) -> bool {
        self.settlements.is_empty() && self.territory.is_none() && self.labels.is_empty() && self.icons.is_empty()
    }
}

fn int(v: Option<&Value>) -> Option<i64> {
    let f = v?.as_f64()?;
    (f.is_finite() && f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_991.0).then_some(f as i64)
}

fn finite(v: Option<&Value>) -> Option<f64> {
    v?.as_f64().filter(|f| f.is_finite())
}

fn string(v: Option<&Value>) -> Option<String> {
    v?.as_str().map(str::to_string)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Members found on records of one kind that this module does not read,
/// with how many records carried each — one report line per member.
fn unknown_members(records: &[&serde_json::Map<String, Value>], known: &[&str]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for r in records {
        for k in r.keys() {
            if !known.contains(&k.as_str()) {
                *out.entry(k.clone()).or_insert(0) += 1;
            }
        }
    }
    out
}

/// Reads the settlements, faction roster, territory, labels and icons out of
/// a flat archive's `state` object. Never fails: a malformed element costs
/// that element and a line in the report (§14.3), and a `state` that is not
/// an object yields an empty result.
pub fn read_legacy(state: &Value, gw: usize, gh: usize) -> LegacyProject {
    let mut out = LegacyProject::default();
    let Some(state) = state.as_object() else { return out };
    let n = gw * gh;
    let report = &mut out.unmapped;
    let civ = state.get("civ").and_then(Value::as_object);

    // ---- the faction roster (state.civ's five parallel arrays) ----
    let names: Vec<Option<String>> = civ
        .and_then(|c| c.get("factionNames"))
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| string(Some(v))).collect())
        .unwrap_or_default();
    let column = |key: &str| -> Vec<Option<String>> {
        civ.and_then(|c| c.get(key))
            .and_then(Value::as_array)
            .map(|a| a.iter().map(|v| string(Some(v))).collect())
            .unwrap_or_default()
    };
    let (cul, rel, gov, ag) =
        (column("factionCulture"), column("factionReligion"), column("factionGovernment"), column("factionAgTech"));
    if !names.is_empty() {
        let mut unnamed = 0;
        for (i, nm) in names.iter().enumerate() {
            let name = match nm {
                Some(s) => s.clone(),
                // `_treeRead`'s own fallback for a roster row with no name.
                None => {
                    unnamed += 1;
                    format!("Faction {i}")
                }
            };
            let at = |c: &Vec<Option<String>>| c.get(i).cloned().flatten();
            out.factions.push(LegacyFaction {
                name,
                culture: at(&cul),
                religion: at(&rel),
                government: at(&gov),
                ag_tech: at(&ag),
            });
        }
        report.push(format!(
            "state.civ: {} -- faction colours are not stored in the flat layout, so this build's \
             palette supplies them (SAVEFILE_COMPAT.md 15.3)",
            plural(names.len(), "faction row", "faction rows")
        ));
        if unnamed > 0 {
            report.push(format!("state.civ.factionNames: {} had no name and read as \"Faction <n>\"", plural(unnamed, "row", "rows")));
        }
        for (key, col, what) in [
            ("factionCulture", &cul, "culture"),
            ("factionReligion", &rel, "religion"),
            ("factionGovernment", &gov, "government"),
            ("factionAgTech", &ag, "agricultural technology"),
        ] {
            let missing = (0..names.len()).filter(|&i| col.get(i).cloned().flatten().is_none()).count();
            if missing > 0 {
                report.push(format!(
                    "state.civ.{key}: absent for {} -- this build's default {what} used",
                    plural(missing, "faction", "factions")
                ));
            }
        }
    }
    let roster_len = if names.is_empty() { REFERENCE_DEFAULT_ROSTER_LEN } else { names.len() };

    // ---- settlements, out of state.places ----
    let places: Vec<&serde_json::Map<String, Value>> = state
        .get("places")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_object).collect())
        .unwrap_or_default();
    let not_objects = state.get("places").and_then(Value::as_array).map_or(0, |a| a.len()) - places.len();
    if not_objects > 0 {
        report.push(format!("state.places: {} not an object, skipped", plural(not_objects, "entry is", "entries are")));
    }

    let mut settlement_records: Vec<&serde_json::Map<String, Value>> = Vec::new();
    let mut poi: BTreeMap<String, usize> = BTreeMap::new();
    let (mut kindless, mut off_grid, mut rounded, mut no_pop, mut no_faction, mut clamped) = (0, 0, 0, 0, 0, 0);
    let mut reclassed: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut pending: Vec<(Option<u64>, LegacySettlement)> = Vec::new();

    for p in &places {
        let kind = p.get("kind").and_then(Value::as_str);
        let is_settlement_category = p.get("category").and_then(Value::as_str) == Some("settlement");
        let tier: &'static str = match kind {
            Some(k) if SETTLEMENT_KINDS.contains(&k) => SETTLEMENT_KINDS.iter().find(|s| **s == k).copied().unwrap_or("town"),
            Some(k) if REFERENCE_ONLY_CLASSES.contains(&k) => {
                *reclassed.entry(k.to_string()).or_insert(0) += 1;
                "town"
            }
            Some(k) if is_settlement_category => {
                *unknown_kind.entry(k.to_string()).or_insert(0) += 1;
                "town"
            }
            Some(k) => {
                // Not a settlement tier and not marked a settlement: a point of
                // interest (the reference's `CIV_POI_TYPES`, v2.11 line 15169)
                // or something newer. §15.1 forbids inventing a settlement.
                *poi.entry(k.to_string()).or_insert(0) += 1;
                continue;
            }
            None => {
                kindless += 1;
                continue;
            }
        };
        let (Some(fx), Some(fy)) = (finite(p.get("x")), finite(p.get("y"))) else {
            off_grid += 1;
            continue;
        };
        // The civ tools round a click to a whole cell before placing
        // (reference 26336); an older save placed before that fix can hold a
        // fractional one, and this port's settlements sit on cells.
        let (rx, ry) = (fx.round(), fy.round());
        if rx != fx || ry != fy {
            rounded += 1;
        }
        if rx < 0.0 || ry < 0.0 || rx >= gw as f64 || ry >= gh as f64 {
            off_grid += 1;
            continue;
        }
        let population = match int(p.get("pop")) {
            Some(v) if v >= 0 => v.min(u32::MAX as i64) as u32,
            _ => {
                no_pop += 1;
                0
            }
        };
        let faction = match int(p.get("faction")) {
            Some(f) if f >= 0 && (f as usize) < roster_len => f as i32,
            Some(_) => {
                clamped += 1;
                0
            }
            None => {
                no_faction += 1;
                0
            }
        };
        let traits: Vec<String> = p
            .get("traits")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|t| t.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let coastal = p.get("coastal").and_then(Value::as_bool) == Some(true) || traits.iter().any(|t| t == "port");
        let tid = int(p.get("tid")).filter(|&t| t >= 1).map(|t| t as u64);
        settlement_records.push(p);
        pending.push((
            tid,
            LegacySettlement {
                tid: 0,
                x: rx as usize,
                y: ry as usize,
                name: string(p.get("name")).unwrap_or_default(),
                population,
                faction,
                kind: tier,
                capital: p.get("capital").and_then(Value::as_bool).unwrap_or(tier == "capital"),
                coastal,
                // §9.1: "Absent: 0" -- a display-only diagnostic.
                suitability: finite(p.get("suit")).unwrap_or(0.0),
                village_seeded: p.get("villageAddon").and_then(Value::as_bool) == Some(true),
                traits,
                history: string(p.get("history")),
                specialisation: string(p.get("specialisation")),
                age: int(p.get("umAge")).filter(|&a| a >= 0).map(|a| a.min(u32::MAX as i64) as u32),
                walls: p.get("umWalls").and_then(Value::as_bool),
            },
        ));
    }
    // Stable ids: an archive's own `tid` wherever it is unique, and fresh
    // ones past the highest for the rest.
    let mut seen = std::collections::HashSet::new();
    let mut next = pending.iter().filter_map(|(t, _)| *t).max().unwrap_or(0) + 1;
    for (tid, mut s) in pending {
        s.tid = match tid {
            Some(t) if seen.insert(t) => t,
            _ => {
                let t = next;
                next += 1;
                seen.insert(t);
                t
            }
        };
        out.settlements.push(s);
    }

    for (k, c) in &poi {
        report.push(format!(
            "state.places: {} of kind \"{k}\" not imported -- this build has no hand-placed point-of-interest record",
            plural(*c, "point", "points")
        ));
    }
    if kindless > 0 {
        report.push(format!(
            "state.places: {} with no kind (the \"designate places\" tool) not imported -- not a settlement",
            plural(kindless, "place", "places")
        ));
    }
    for (k, c) in &reclassed {
        report.push(format!(
            "state.places: {} of class \"{k}\" read as \"town\" -- this build has no such tier",
            plural(*c, "settlement", "settlements")
        ));
    }
    for (k, c) in &unknown_kind {
        report.push(format!("state.places: {} of unknown kind \"{k}\" read as \"town\"", plural(*c, "settlement", "settlements")));
    }
    if off_grid > 0 {
        report.push(format!("state.places: {} outside the grid, not imported", plural(off_grid, "settlement", "settlements")));
    }
    if rounded > 0 {
        report.push(format!("state.places: {} at a fractional cell, rounded to the nearest", plural(rounded, "settlement", "settlements")));
    }
    if no_pop > 0 {
        report.push(format!("state.places: {} carried no population, read as 0", plural(no_pop, "settlement", "settlements")));
    }
    if no_faction > 0 {
        report.push(format!("state.places: {} carried no faction, read as Unclaimed", plural(no_faction, "settlement", "settlements")));
    }
    if clamped > 0 {
        report.push(format!(
            "state.places: {} named a faction outside the roster, read as Unclaimed",
            plural(clamped, "settlement", "settlements")
        ));
    }
    for (m, c) in unknown_members(&settlement_records, &PLACE_MEMBERS) {
        report.push(format!(
            "state.places[].{m}: carried by {}, no home in this build, not imported",
            plural(c, "settlement", "settlements")
        ));
    }

    // ---- territory ----
    if let Some(pairs) = civ.and_then(|c| c.get("territory")).and_then(Value::as_array).filter(|a| !a.is_empty()) {
        let mut grid = vec![0i32; n];
        let (mut bad_cell, mut bad_faction) = (0, 0);
        for pair in pairs.chunks(2) {
            let (Some(i), Some(f)) = (int(pair.first()), int(pair.get(1))) else {
                bad_cell += 1;
                continue;
            };
            if i < 0 || i as usize >= n {
                bad_cell += 1;
                continue;
            }
            if f < 0 || f as usize >= roster_len {
                bad_faction += 1;
                continue;
            }
            grid[i as usize] = f as i32;
        }
        if bad_cell > 0 {
            report.push(format!("state.civ.territory: {} outside the grid or malformed, skipped", plural(bad_cell, "claim", "claims")));
        }
        if bad_faction > 0 {
            report.push(format!(
                "state.civ.territory: {} named a faction outside the roster, left unclaimed",
                plural(bad_faction, "claim", "claims")
            ));
        }
        out.territory = Some(grid);
    }

    // ---- what the ruling does not cover, where the archive has any ----
    if let Some(c) = civ {
        let count = |k: &str| c.get(k).and_then(Value::as_array).map_or(0, Vec::len);
        let ways = count("ways") + count("routes");
        if ways > 0 {
            report.push(format!(
                "state.civ.ways: {} not imported -- Ruling AU covers settlements, labels and icons, and a way's \
                 endpoints index the places array, which this import filters",
                plural(ways, "way", "ways")
            ));
        }
        if count("journeys") > 0 {
            report.push(format!("state.civ.journeys: {} not imported", plural(count("journeys"), "journey", "journeys")));
        }
        if count("timeline") > 0 {
            report.push(format!(
                "state.civ.timeline: {} not imported (the year cursor with them)",
                plural(count("timeline"), "recorded year", "recorded years")
            ));
        } else if int(c.get("year")).is_some_and(|y| y != 0) {
            report.push("state.civ.year: the timeline cursor, not imported".to_string());
        }
        let extra: Vec<&str> = c.keys().map(String::as_str).filter(|k| !CIV_MEMBERS.contains(k)).collect();
        if !extra.is_empty() {
            report.push(format!("state.civ: {} not imported (no home in this build)", extra.join(", ")));
        }
    }
    if state.get("roads").is_some_and(|r| !r.is_null()) {
        report.push("state.roads: the older auto-built road network, not imported".to_string());
    }
    if let Some(cp) = state.get("cartoPaint").and_then(Value::as_object) {
        let painted: Vec<&str> = cp
            .iter()
            .filter(|(_, v)| v.as_array().is_some_and(|a| !a.is_empty()))
            .map(|(k, _)| k.as_str())
            .collect();
        if !painted.is_empty() {
            report.push(format!("state.cartoPaint: hand-painted {} cells not imported", painted.join("/")));
        }
    }
    if state.get("region").is_some_and(|r| !r.is_null()) {
        report.push("state.region: the region-of-interest marquee, not imported".to_string());
    }

    // ---- labels (§11.1) ----
    let label_arr: Vec<&Value> = state.get("labels").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
    let mut label_records = Vec::new();
    let mut bad_labels = 0;
    for l in label_arr {
        let Some(o) = l.as_object() else {
            bad_labels += 1;
            continue;
        };
        let (Some(x), Some(y)) = (finite(o.get("x")), finite(o.get("y"))) else {
            bad_labels += 1;
            continue;
        };
        label_records.push(o);
        out.labels.push(LegacyLabel {
            x,
            y,
            name: string(o.get("name")).unwrap_or_default(),
            // `_treeRead`'s defaults, which are the click handler's literal.
            angle: finite(o.get("angle")).unwrap_or(0.0),
            arc: finite(o.get("arc")).unwrap_or(0.0),
            size: finite(o.get("size")).unwrap_or(16.0),
            font: string(o.get("font")),
            color: string(o.get("color")),
            fixed_size: o.get("sizeMode").and_then(Value::as_str) == Some("fixed"),
        });
    }
    if bad_labels > 0 {
        report.push(format!("state.labels: {} without a finite position, skipped", plural(bad_labels, "label", "labels")));
    }
    for (m, c) in unknown_members(&label_records, &LABEL_MEMBERS) {
        report.push(format!("state.labels[].{m}: carried by {}, no home in this build, not imported", plural(c, "label", "labels")));
    }

    // ---- icons (§11.2) ----
    let icon_arr: Vec<&Value> = state.get("mapIcons").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
    let mut icon_records = Vec::new();
    let (mut bad_icons, mut no_scale) = (0, 0);
    let mut bad_family: BTreeMap<String, usize> = BTreeMap::new();
    for ic in icon_arr {
        let Some(o) = ic.as_object() else {
            bad_icons += 1;
            continue;
        };
        let (Some(x), Some(y)) = (finite(o.get("x")), finite(o.get("y"))) else {
            bad_icons += 1;
            continue;
        };
        let fam = o.get("fam").and_then(Value::as_str);
        let Some(family) = fam.and_then(|f| ICON_FAMILIES.iter().find(|k| **k == f).copied()) else {
            *bad_family.entry(fam.unwrap_or("(none)").to_string()).or_insert(0) += 1;
            continue;
        };
        let scale = match finite(o.get("scale")) {
            Some(s) if s > 0.0 => s,
            _ => {
                no_scale += 1;
                1.0
            }
        };
        icon_records.push(o);
        out.icons.push(LegacyIcon {
            x,
            y,
            family,
            slot: string(o.get("slot")).unwrap_or_default(),
            set: if family == "custom" { string(o.get("set")) } else { None },
            scale,
        });
    }
    if bad_icons > 0 {
        report.push(format!("state.mapIcons: {} without a finite position, skipped", plural(bad_icons, "icon", "icons")));
    }
    for (f, c) in &bad_family {
        report.push(format!("state.mapIcons: {} of unknown family \"{f}\" not imported", plural(*c, "icon", "icons")));
    }
    if no_scale > 0 {
        report.push(format!("state.mapIcons: {} with no usable scale, read as 1", plural(no_scale, "icon", "icons")));
    }
    for (m, c) in unknown_members(&icon_records, &ICON_MEMBERS) {
        report.push(format!("state.mapIcons[].{m}: carried by {}, no home in this build, not imported", plural(c, "icon", "icons")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_state_with_no_records_imports_nothing_and_reports_nothing() {
        for state in [
            Value::Null,
            json!({}),
            json!({"places": [], "labels": [], "mapIcons": [], "civ": null, "roads": null,
                   "cartoPaint": {"biome": [], "splat": [], "terrain": []}}),
        ] {
            let got = read_legacy(&state, 8, 6);
            assert!(got.is_empty(), "{state}");
            assert!(got.unmapped.is_empty(), "{state}: {:?}", got.unmapped);
        }
    }

    #[test]
    fn settlements_are_filtered_by_kind_and_everything_else_is_reported() {
        let state = json!({
            "places": [
                {"x": 2, "y": 3, "name": "Kessa", "kind": "city", "faction": 2, "pop": 5400, "traits": ["port"], "tid": 9},
                {"x": 4, "y": 1, "name": "", "kind": "shrine", "faction": 0, "traits": []},
                {"x": 1.4, "y": 2.6},
                {"x": 5, "y": 5, "name": "Oda", "kind": "monastery", "category": "settlement", "faction": 1, "pop": 80, "tradePop": 3},
                {"x": 50, "y": 1, "name": "Far", "kind": "town", "faction": 1, "pop": 10}
            ]
        });
        let got = read_legacy(&state, 8, 6);
        assert_eq!(got.settlements.len(), 2);
        let k = &got.settlements[0];
        assert_eq!((k.tid, k.x, k.y, k.name.as_str(), k.kind, k.population, k.faction), (9, 2, 3, "Kessa", "city", 5400, 2));
        assert!(k.coastal, "the `port` trait is the flat layout's coastal flag");
        assert!(!k.capital);
        let o = &got.settlements[1];
        assert_eq!((o.tid, o.kind), (10, "town"), "fresh id past the highest; monastery has no tier");
        let r = got.unmapped.join("\n");
        for needle in [
            "1 point of kind \"shrine\" not imported",
            "1 place with no kind",
            "1 settlement of class \"monastery\" read as \"town\"",
            "1 settlement outside the grid",
            "state.places[].tradePop: carried by 1 settlement",
        ] {
            assert!(r.contains(needle), "missing {needle:?} in:\n{r}");
        }
    }

    #[test]
    fn a_missing_value_is_reported_not_silently_defaulted() {
        let state = json!({"places": [{"x": 1, "y": 1, "kind": "town"}]});
        let got = read_legacy(&state, 4, 4);
        assert_eq!(got.settlements[0].population, 0);
        let r = got.unmapped.join("\n");
        assert!(r.contains("1 settlement carried no population"), "{r}");
        assert!(r.contains("1 settlement carried no faction"), "{r}");
    }

    #[test]
    fn territory_is_decoded_from_sparse_pairs_and_colours_are_reported() {
        let state = json!({"civ": {
            "territory": [0, 1, 5, 2, 99, 1, 3, 9],
            "factionNames": ["Unclaimed", "Aurelia", "Kessa"],
            "factionCulture": ["common", "imperial"],
            "ways": [{"pts": []}], "year": 0, "regionalPop": 12
        }});
        let got = read_legacy(&state, 4, 3);
        assert_eq!(got.territory.as_deref(), Some(&[1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0][..]));
        assert_eq!(got.factions.len(), 3);
        assert_eq!(got.factions[1].culture.as_deref(), Some("imperial"));
        assert_eq!(got.factions[2].culture, None, "past the array's end is absent, not a guess");
        let r = got.unmapped.join("\n");
        for needle in [
            "3 faction rows -- faction colours are not stored",
            "state.civ.factionCulture: absent for 1 faction",
            "1 claim outside the grid",
            "1 claim named a faction outside the roster",
            "state.civ.ways: 1 way not imported",
            "state.civ: regionalPop not imported",
        ] {
            assert!(r.contains(needle), "missing {needle:?} in:\n{r}");
        }
    }
}
