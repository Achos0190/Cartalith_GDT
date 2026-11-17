//! Ruling FZ (owner, 2026-11-17): the Godot-facing side of **name styles from CSV**
//! (`cartalith_civ::name_style` owns the format, the parser, the generator and the export).
//!
//! # What lives here
//!
//! * [`NameStylesState`]: the world's own **copies** of the styles it uses and its default style. Styles live in
//!   the user's library (per device, kept by the shell like the asset library); each world saves a copy of
//!   the ones it uses (`entities/name_styles.json`, `SAVEFILE_COMPAT.md`), so a world opens and names
//!   the same on another device.
//! * [`NameStylesDoc`]: that save slot. Written only when there is something to say (a world this build
//!   generated writes `names_tracked`, so a world with no style still says which names were hand-given).
//! * The `ns_*` functions: parse, preview, export, assign, **Rename all** (one undoable step that
//!   never touches a hand-given name) and the names of the kinds the engine did not name.
//!
//! # What this must never do
//!
//! * Change anything when no style is chosen and `name_unnamed_kinds` is off: every built-in culture's names,
//!   the golden fixtures and the old saves stay byte for byte what they were.
//! * Touch a hand-given name. `PlaceExtras::name_given` records them; a world whose names were written before
//!   that record existed (`CivData::names_tracked == false`) has **no** name it may assume generated, so
//!   Rename all skips every one unless the caller says `include_unknown`.

use crate::*;
use cartalith_civ::name_style::{self, NameStyle};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One saved style copy: its library key and its canonical CSV (`name_style::export_style`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct StyleDto {
    pub key: String,
    pub csv: String,
}

/// `entities/name_styles.json` (`SAVEFILE_COMPAT.md`): the world's name-style state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct NameStylesDoc {
    #[serde(default)]
    pub version: u32,
    /// Whether this world records which settlement names were hand-given (`CivData::names_tracked`).
    #[serde(default)]
    pub names_tracked: bool,
    /// The world default: a built-in culture key or a saved style's key. Omitted when none is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_default: Option<String>,
    /// Copies of the styles this world uses (the default and every faction's), canonical CSV.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub styles: Vec<StyleDto>,
}

/// What a Rename all changed, kept for one undo. The `after` halves are what the edit left, so the undo
/// can refuse once a name changed again (never overwrite a newer edit).
#[derive(Debug, Clone, Default)]
pub(crate) struct RenameUndo {
    pub settlements: Vec<(u64, String, String)>,
    pub continents: Vec<(i32, String, String)>,
    pub provinces: Vec<(i32, String, String)>,
    pub world_name: Option<(Option<String>, String)>,
}

/// The world's name-style state.
#[derive(Debug, Clone, Default)]
pub(crate) struct NameStylesState {
    /// A built-in culture key or a `style:` key; `None` leaves every built-in behaviour untouched.
    pub world_default: Option<String>,
    /// The copies this world holds, by style key.
    pub styles: BTreeMap<String, NameStyle>,
    /// [`Self::world_default`] resolved to a style (a built-in culture becomes `culture_as_style`).
    pub resolved_default: Option<NameStyle>,
    /// The last Rename all, while it is still undoable.
    pub undo: Option<RenameUndo>,
}

impl NameStylesState {
    /// Resolve a key (built-in culture or `style:`) to a style: the world's copy, else the process registry.
    pub fn resolve(&self, key: &str) -> Option<NameStyle> {
        if let Some(c) = name_style::builtin_culture(key) {
            return Some(name_style::culture_as_style(c));
        }
        if let Some(s) = self.styles.get(key) {
            return Some(s.clone());
        }
        name_style::style_for_key(key).map(|a| (*a).clone())
    }

    /// Hold a copy of `key`'s style in this world (a built-in key holds nothing: it is always available).
    pub fn adopt(&mut self, key: &str) -> bool {
        if name_style::builtin_culture(key).is_some() {
            return true;
        }
        match name_style::style_for_key(key) {
            Some(a) => {
                self.styles.insert(key.to_string(), (*a).clone());
                true
            }
            None => self.styles.contains_key(key),
        }
    }

    /// Set (`Some`) or clear (`None`) the world default; `false` for a key that names nothing.
    pub fn set_default(&mut self, key: Option<&str>) -> bool {
        match key {
            None => {
                self.world_default = None;
                self.resolved_default = None;
                true
            }
            Some(k) => {
                if !self.adopt(k) {
                    return false;
                }
                let Some(st) = self.resolve(k) else { return false };
                self.world_default = Some(k.to_string());
                self.resolved_default = Some(st);
                true
            }
        }
    }

    /// The style that names the unnamed kinds: the world default, else the built-in culture the lake labels
    /// already speak in (`civ_default_culture(1)`).
    pub fn feature_style(&self) -> NameStyle {
        self.resolved_default.clone().unwrap_or_else(|| name_style::culture_as_style(cartalith_civ::civ_default_culture(1)))
    }
}

/// The save document, or `None` when there is nothing to record.
pub(crate) fn save_doc(state: &NameStylesState, names_tracked: bool, used_keys: &BTreeSet<String>) -> Option<NameStylesDoc> {
    let mut styles: Vec<StyleDto> = Vec::new();
    for (k, st) in &state.styles {
        if used_keys.contains(k) || state.world_default.as_deref() == Some(k.as_str()) {
            styles.push(StyleDto { key: k.clone(), csv: name_style::export_style(st) });
        }
    }
    if !names_tracked && styles.is_empty() && state.world_default.is_none() {
        return None;
    }
    Some(NameStylesDoc { version: 1, names_tracked, world_default: state.world_default.clone(), styles })
}

/// Rebuild the state from a saved document (registering each style under its saved key so the roster's
/// `style:` culture keys resolve). A style that does not parse is a warning and is dropped; whatever named it
/// keeps its stored key and names in the built-in culture until reassigned. Never repairs a CSV.
pub(crate) fn restore(doc: &NameStylesDoc, warnings: &mut Vec<String>) -> NameStylesState {
    let mut st = NameStylesState::default();
    for s in &doc.styles {
        let rep = name_style::parse_style_csv(&s.csv);
        match rep.style {
            Some(style) => {
                name_style::register_style_under(&s.key, style.clone());
                st.styles.insert(s.key.clone(), style);
            }
            None => warnings.push(format!(
                "entities/name_styles.json: the style {} could not be read ({}) -- what used it names in the built-in culture until it is reassigned",
                s.key,
                rep.issues.first().map(|i| i.message.as_str()).unwrap_or("unreadable")
            )),
        }
    }
    if let Some(d) = doc.world_default.as_deref() {
        if !st.set_default(Some(d)) {
            warnings.push(format!("entities/name_styles.json: the world default {d} is not in this file; the world has no default style"));
        }
    }
    st
}

/// A 32-bit seed for one feature: the world seed with the feature's own cell indices, never zero.
pub(crate) fn feature_seed(world_seed: i32, a: usize, b: usize) -> u32 {
    let h = (world_seed as u32).wrapping_mul(0x9E37_79B1) ^ (a as u32).wrapping_mul(0x85EB_CA6B) ^ (b as u32).rotate_left(13).wrapping_mul(0xC2B2_AE35);
    if h == 0 { 1 } else { h }
}

/// One name per feature, in order: each from its own stream (so a feature's name does not depend on how many
/// features precede it), made unique across the list.
pub(crate) fn names_for(style: &NameStyle, items: &[(u32, &str)]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    items
        .iter()
        .map(|(seed, section)| {
            let mut rng = cartalith_rng::Mulberry32::new(*seed);
            style.feature_name(&mut rng, section, &mut seen)
        })
        .collect()
}

/// The result of [`rename_all`], for the shell's summary line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RenameReport {
    pub renamed: usize,
    pub kept_hand: usize,
    pub kept_unknown: usize,
    pub continents: usize,
    pub provinces: usize,
}

/// Ruling FZ item 4: rename every **generated** settlement (and the continents and the provinces named from
/// them) in `style`, never a hand-given one. `include_unknown` treats the names of a world written before the
/// record (`names_tracked == false`) as generated; it changes nothing for a tracked world, where the record is
/// exact. Draws from `rng` (the manual-name stream, so repeated presses give new names) and returns what to
/// undo with. Uniqueness is per world: a new name never equals a name already on the map.
pub(crate) fn rename_all(
    civ: &mut CivData,
    style: &NameStyle,
    rng: &mut cartalith_rng::Mulberry32,
    include_unknown: bool,
) -> (RenameReport, RenameUndo) {
    let mut report = RenameReport::default();
    let mut undo = RenameUndo::default();
    let mut seen: BTreeSet<String> = civ.settlements.iter().map(|s| s.name.clone()).collect();
    let mut renamed_idx: Vec<usize> = Vec::new();
    for i in 0..civ.settlements.len() {
        let tid = civ.settlements[i].tid;
        let given = civ.place_extras.0.get(&tid).is_some_and(|e| e.name_given);
        if given {
            report.kept_hand += 1;
            continue;
        }
        if !civ.names_tracked && !include_unknown {
            report.kept_unknown += 1;
            continue;
        }
        let old = civ.settlements[i].name.clone();
        seen.remove(&old);
        let new = style.stem_bounded(rng, "Settlements", &mut seen);
        civ.settlements[i].name = new.clone();
        undo.settlements.push((tid, old, new));
        renamed_idx.push(i);
        report.renamed += 1;
    }
    // Provinces take their seat's new name (`"{stem} Province"` in the built-in form; the style's `[Provinces]` templates here).
    for p in civ.province_list.iter_mut() {
        if renamed_idx.contains(&p.capital_settlement_index) {
            let stem = civ.settlements[p.capital_settlement_index].name.clone();
            let new = style.decorate(&stem, "Provinces", rng);
            undo.provinces.push((p.id, p.name.clone(), new.clone()));
            p.name = new;
            report.provinces += 1;
        }
    }
    let mut cseen: BTreeSet<String> = BTreeSet::new();
    for c in civ.continents.iter_mut() {
        let new = style.feature_name(rng, "Continents", &mut cseen);
        undo.continents.push((c.id, c.name.clone(), new.clone()));
        c.name = new;
        report.continents += 1;
    }
    (report, undo)
}

/// Name the landmasses no faction holds from the world default style's `[Continents]` section (a landmass a
/// faction holds is named by that faction's own culture, in `civ_continents_with_styles`). A stream of its own
/// (`civ_continent_name_rng`), so the result is a pure function of the style and the continent list, and the
/// built-in path is never reached when no default style is set (the caller checks).
pub(crate) fn name_unowned_continents(civ: &mut CivData, style: &NameStyle) {
    let mut rng = cartalith_civ::civ_continent_name_rng();
    let mut seen: BTreeSet<String> = civ.continents.iter().filter(|c| c.faction != 0).map(|c| c.name.clone()).collect();
    for c in civ.continents.iter_mut().filter(|c| c.faction == 0) {
        c.name = style.feature_name(&mut rng, "Continents", &mut seen);
    }
}

/// Undo a [`rename_all`] if every name it wrote is still in place; otherwise change nothing and say so.
pub(crate) fn undo_rename(civ: &mut CivData, u: &RenameUndo) -> Result<(), String> {
    for (tid, _, after) in &u.settlements {
        if !civ.settlements.iter().any(|s| s.tid == *tid && &s.name == after) {
            return Err("a renamed place has changed since; undo would overwrite a newer name".into());
        }
    }
    for (tid, old, _) in &u.settlements {
        if let Some(s) = civ.settlements.iter_mut().find(|s| s.tid == *tid) {
            s.name = old.clone();
        }
    }
    for (id, old, _) in &u.provinces {
        if let Some(p) = civ.province_list.iter_mut().find(|p| p.id == *id) {
            p.name = old.clone();
        }
    }
    for (id, old, _) in &u.continents {
        if let Some(c) = civ.continents.iter_mut().find(|c| c.id == *id) {
            c.name = old.clone();
        }
    }
    Ok(())
}

// ===================== the shell's functions =====================

/// The `Dictionary` shape of one parse issue.
fn issue_dict(i: &name_style::StyleIssue) -> VarDictionary {
    vdict! { "line" => i.line as i64, "error" => i.error, "message" => i.message.as_str() }
}

#[godot_api(secondary)]
impl WorldGen {
    /// Ruling FZ: every section a style may carry, in file order: `[{key, label, templates:[{text, weight}]}]`.
    /// `templates` are the built-in ones the section falls back to. A design table, available before any world.
    #[func]
    fn ns_sections(&self) -> Array<VarDictionary> {
        name_style::SECTIONS
            .iter()
            .map(|d| {
                let t: Array<VarDictionary> = d.templates.iter().map(|(text, w)| vdict! { "text" => *text, "weight" => *w }).collect();
                vdict! { "key" => d.key, "label" => d.label, "templates" => &t }
            })
            .collect()
    }

    /// A built-in culture as an import-format CSV: the worked example (`name_style::export_culture`). Empty
    /// for a key that is not one of the seven.
    #[func]
    fn ns_builtin_csv(&self, key: GString) -> GString {
        match name_style::builtin_culture(&key.to_string()) {
            Some(c) => name_style::export_culture(c).as_str().into(),
            None => GString::new(),
        }
    }

    /// Parse a name-style CSV. `{ok, name, key, csv, issues:[{line, error, message}], errors, sections:[{key, fragments,
    /// templates}], min_syl, max_syl, max_len}`; `ok` is true when a style came out (rows with errors were skipped and are in
    /// `issues`, never repaired). With `register` the style is added to the process library so previews and the
    /// roster can name it by `key`; `csv` is the canonical form to store.
    #[func]
    fn ns_parse(&self, csv: GString, register: bool) -> VarDictionary {
        let rep = name_style::parse_style_csv(&csv.to_string());
        let issues: Array<VarDictionary> = rep.issues.iter().map(issue_dict).collect();
        let errors = rep.issues.iter().filter(|i| i.error).count() as i64;
        let mut d = vdict! { "ok" => rep.style.is_some(), "issues" => &issues, "errors" => errors };
        if let Some(st) = rep.style {
            let secs: Array<VarDictionary> = name_style::SECTIONS
                .iter()
                .filter_map(|def| st.sections.get(def.key).map(|s| (def, s)))
                .map(|(def, s)| {
                    let own_frag = !s.fragments.start.is_empty();
                    vdict! { "key" => def.key, "label" => def.label, "fragments" => own_frag, "templates" => s.templates.len() as i64 }
                })
                .collect();
            d.set("name", st.name.as_str());
            d.set("csv", name_style::export_style(&st).as_str());
            d.set("sections", &secs);
            d.set("min_syl", st.min_syl as i64);
            d.set("max_syl", st.max_syl as i64);
            if let Some(m) = st.max_len {
                d.set("max_len", m as i64);
            }
            let key = if register { name_style::register_style(st) } else { st.key() };
            d.set("key", key.as_str());
        }
        d
    }

    /// `count` sample names (2 to 40) for `section` in the style `key` (a built-in culture key, a library
    /// `style:` key or this world's copy), from `seed`: the same arguments always give the same names, a new
    /// seed gives new ones. Unique within the sample, built exactly as the engine builds them (a bounded stem
    /// then a template; a settlement is the bare stem). Empty for an unknown key or section.
    #[func]
    fn ns_preview(&self, key: GString, section: GString, seed: i64, count: i64) -> PackedStringArray {
        let Some(st) = self.name_styles.resolve(&key.to_string()) else { return PackedStringArray::new() };
        let sec = section.to_string();
        let Some(def) = name_style::section_def(&sec) else { return PackedStringArray::new() };
        let mut rng = cartalith_rng::Mulberry32::new(if seed as u32 == 0 { 1 } else { seed as u32 });
        let mut seen = BTreeSet::new();
        let mut out = PackedStringArray::new();
        for _ in 0..count.clamp(2, 40) {
            let n = if def.key == "Settlements" { st.stem_bounded(&mut rng, def.key, &mut seen) } else { st.feature_name(&mut rng, def.key, &mut seen) };
            out.push(n.as_str());
        }
        out
    }

    /// The canonical CSV of a style the process or this world holds (`Export .csv`); empty for a built-in key
    /// (use `ns_builtin_csv`) or an unknown one.
    #[func]
    fn ns_style_csv(&self, key: GString) -> GString {
        let k = key.to_string();
        match self.name_styles.styles.get(&k).cloned().or_else(|| name_style::style_for_key(&k).map(|a| (*a).clone())) {
            Some(st) => name_style::export_style(&st).as_str().into(),
            None => GString::new(),
        }
    }

    /// This world's name-style state: `{world_default, tracked, generated, hand_given, unknown, copies:[key], can_undo}`.
    /// `generated`, `hand_given` and `unknown` count settlements; `unknown` is every name of a world written before the
    /// record existed (`tracked` false), which Rename all skips unless told otherwise. `world_default` is omitted
    /// when none is set.
    #[func]
    fn ns_world_state(&self) -> VarDictionary {
        let (mut gen_, mut hand, mut unk, tracked) = (0i64, 0i64, 0i64, self.civ.as_ref().is_some_and(|c| c.names_tracked));
        if let Some(civ) = self.civ.as_ref() {
            for s in &civ.settlements {
                if civ.place_extras.0.get(&s.tid).is_some_and(|e| e.name_given) {
                    hand += 1;
                } else if civ.names_tracked {
                    gen_ += 1;
                } else {
                    unk += 1;
                }
            }
        }
        let copies: PackedStringArray = self.name_styles.styles.keys().map(|k| GString::from(k.as_str())).collect();
        let mut d = vdict! {
            "tracked" => tracked, "generated" => gen_, "hand_given" => hand, "unknown" => unk,
            "copies" => &copies, "can_undo" => self.name_styles.undo.is_some(),
        };
        if let Some(w) = self.name_styles.world_default.as_deref() {
            d.set("world_default", w);
        }
        d
    }

    /// Set the world default style (`""` clears it). A built-in culture key or a `style:` key the library holds;
    /// the world keeps a copy so it opens the same elsewhere. `false` (nothing changed) for a key that names
    /// nothing. The default names continents no faction holds, lakes, the world and the unnamed kinds from now on;
    /// it renames nothing already named (that is `ns_rename_all`).
    #[func]
    fn ns_set_world_default(&mut self, key: GString) -> bool {
        let k = key.to_string();
        let ok = if k.is_empty() { self.name_styles.set_default(None) } else { self.name_styles.set_default(Some(&k)) };
        if ok {
            self.name_styles.undo = None;
        }
        ok
    }

    /// Give faction `faction` the naming culture `key` (a built-in culture or a library style): the roster's
    /// culture column, so its next settlements, re-rolls and continents are named by it. The world keeps a
    /// copy of a style. `false` for an unassignable faction or a key that names nothing.
    #[func]
    fn ns_assign_faction(&mut self, faction: i64, key: GString) -> bool {
        let k = key.to_string();
        let Some(civ) = self.civ.as_mut() else { return false };
        if !civ.faction_roster.is_assignable(faction as i32) {
            return false;
        }
        if !self.name_styles.adopt(&k) {
            return false;
        }
        civ.faction_roster.set_field(faction as usize, "culture", &k)
    }

    /// Ruling FZ item 4, **Rename all**: rename every generated settlement, the provinces named from them and
    /// the continents in the style `key` (a built-in culture, a library style; `""` means the world default), and
    /// the world's own name; **never a hand-given name**. One step: `ns_rename_undo` puts all of it back, unless a
    /// renamed place changed again since. `include_unknown` treats the names of a world saved before the record
    /// as generated. Returns `{ok, renamed, kept_hand, kept_unknown, continents, provinces}` or `{ok:false, reason}`.
    #[func]
    fn ns_rename_all(&mut self, key: GString, include_unknown: bool) -> VarDictionary {
        let k = {
            let g = key.to_string();
            if g.is_empty() { self.name_styles.world_default.clone().unwrap_or_default() } else { g }
        };
        if k.is_empty() {
            return vdict! { "ok" => false, "reason" => "no style chosen and no world default is set" };
        }
        let Some(style) = self.name_styles.resolve(&k) else {
            return vdict! { "ok" => false, "reason" => "that style is not in the library or this world" };
        };
        let (Some(civ), Some(tools)) = (self.civ.as_mut(), self.civ_tools.as_mut()) else {
            return vdict! { "ok" => false, "reason" => "no world with places is open" };
        };
        let (rep, mut undo) = rename_all(civ, &style, &mut tools.name_rng, include_unknown);
        if self.seed != 0 || self.world_name.is_some() {
            let new = cartalith_civ::naming::world_name_styled(self.seed as u32, &style);
            undo.world_name = Some((self.world_name.clone(), new.clone()));
            self.world_name = Some(new);
        }
        self.ledger.record("civ", "Rename all", format!("{} place(s), {} province(s), {} continent(s) in {}", rep.renamed, rep.provinces, rep.continents, style.name), undo::EntryKind::Recorded("undo it with Data > Name styles > Undo rename all"));
        self.name_styles.undo = Some(undo);
        vdict! {
            "ok" => true, "renamed" => rep.renamed as i64, "kept_hand" => rep.kept_hand as i64,
            "kept_unknown" => rep.kept_unknown as i64, "continents" => rep.continents as i64, "provinces" => rep.provinces as i64,
        }
    }

    /// Undo the last Rename all (one step). `false` when there is nothing to undo or a renamed place has
    /// changed since (then nothing is touched).
    #[func]
    fn ns_rename_undo(&mut self) -> bool {
        let Some(u) = self.name_styles.undo.clone() else { return false };
        let Some(civ) = self.civ.as_mut() else { return false };
        if undo_rename(civ, &u).is_err() {
            return false;
        }
        if let Some((old, _)) = u.world_name {
            self.world_name = old;
        }
        self.name_styles.undo = None;
        true
    }

    /// Whether `ns_rename_undo` has a step to take back.
    #[func]
    fn ns_can_undo_rename(&self) -> bool {
        self.name_styles.undo.is_some()
    }
}

impl WorldGen {
    /// The names of `rivers` when `name_unnamed_kinds` is on (else `None`): one per river, in order, from the
    /// world's style, each seeded by the river's own head and mouth cells.
    pub(crate) fn river_style_names(&self, rivers: &[cartalith_hydrology::River]) -> Option<Vec<String>> {
        if !self.params.name_unnamed_kinds {
            return None;
        }
        let st = self.name_styles.feature_style();
        let items: Vec<(u32, &str)> = rivers.iter().map(|r| (feature_seed(self.seed, r.head as usize, r.mouth as usize), "Rivers")).collect();
        Some(names_for(&st, &items))
    }

    /// `landmark key -> style name` for every placed landmark whose kind has a name-style section, when
    /// `name_unnamed_kinds` is on. Empty otherwise. A user-given name is the caller's business (it wins).
    pub(crate) fn landmark_style_names(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        if !self.params.name_unnamed_kinds {
            return out;
        }
        let Some(run) = self.landmark_store.last.as_ref() else { return out };
        let st = self.name_styles.feature_style();
        let items: Vec<(u32, &str)> = run.landmarks.iter().map(|l| (feature_seed(self.seed, l.x as usize, l.y as usize), name_style::section_for_kind(&l.kind))).collect();
        for (l, n) in run.landmarks.iter().zip(names_for(&st, &items)) {
            out.insert(l.key(), n);
        }
        out
    }

    /// The culture keys the roster uses that name a library style (for the save's `styles`).
    pub(crate) fn name_style_used_keys(&self) -> BTreeSet<String> {
        self.civ.as_ref().map(|c| c.faction_roster.0.iter().map(|e| e.culture.clone()).filter(|k| k.starts_with(name_style::STYLE_KEY_PREFIX)).collect()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style(csv: &str) -> NameStyle {
        let r = name_style::parse_style_csv(csv);
        assert!(r.issues.iter().all(|i| !i.error), "{:?}", r.issues);
        r.style.unwrap()
    }

    /// Protects: the save document round-trips a world default and its style copy, writes nothing for a
    /// world with neither (untracked), writes only `names_tracked` for a plain new world, and omits styles
    /// no faction or default uses.
    #[test]
    fn the_document_round_trips_and_is_empty_when_there_is_nothing() {
        let mut s = NameStylesState::default();
        assert!(save_doc(&s, false, &BTreeSet::new()).is_none(), "an old save with nothing set writes no member");
        let d = save_doc(&s, true, &BTreeSet::new()).expect("a tracked world says so");
        assert!(d.styles.is_empty() && d.world_default.is_none() && d.names_tracked);
        let st = style("Name style,Rt\nSyllables,1,1\n[Settlements]\nStart\nzo\n");
        let key = name_style::register_style(st);
        assert!(s.set_default(Some(&key)));
        s.styles.insert("style:unused".into(), style("Name style,U\n[Settlements]\nStart\nq\n"));
        let doc = save_doc(&s, true, &BTreeSet::new()).unwrap();
        assert_eq!(doc.styles.len(), 1, "the unused copy is not written");
        let json = serde_json::to_string(&doc).unwrap();
        let back: NameStylesDoc = serde_json::from_str(&json).unwrap();
        let mut warnings = Vec::new();
        let s2 = restore(&back, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s2.world_default.as_deref(), Some(key.as_str()));
        assert_eq!(s2.resolved_default.as_ref().unwrap().name, "Rt");
        assert!(name_style::style_for_key(&key).is_some());
    }

    /// Protects: a document naming a style whose CSV cannot be read warns and drops it (never repairs), and a
    /// default that is not in the file warns and leaves the world with no default.
    #[test]
    fn an_unreadable_style_or_missing_default_warns() {
        let doc = NameStylesDoc {
            version: 1,
            names_tracked: true,
            world_default: Some("style:gone".into()),
            styles: vec![StyleDto { key: "style:bad".into(), csv: "[Settlements]\nStart\nx\n".into() }],
        };
        let mut w = Vec::new();
        let s = restore(&doc, &mut w);
        assert_eq!(w.len(), 2, "{w:?}");
        assert!(s.world_default.is_none() && s.styles.is_empty());
    }

    /// Protects: a built-in key resolves and never needs a copy; an unknown key is refused.
    #[test]
    fn builtin_keys_resolve_and_unknown_ones_do_not() {
        let mut s = NameStylesState::default();
        assert!(s.set_default(Some("highland")));
        assert!(s.styles.is_empty());
        assert_eq!(s.resolved_default.as_ref().unwrap().name, "highland");
        assert!(!s.set_default(Some("style:nonesuch")));
        assert_eq!(s.world_default.as_deref(), Some("highland"), "a refused change leaves the old default");
    }

    fn four_places() -> CivData {
        let mk = |tid: u64, name: &str| cartalith_civ::NamedSettlement {
            tid,
            placement: cartalith_civ::SettlementPlacement { x: tid as usize, y: 0, suit: 0.5, faction: 1, kind: cartalith_civ::SettlementKind::Town, capital: false, coastal: false },
            name: name.to_string(),
            pop: 100,
        };
        let mut civ = crate::civ_timeline_tests::empty_civ(vec![mk(1, "Hand Named"), mk(2, "Gen Two"), mk(3, "Gen Three"), mk(4, "Gen Four")], vec![0; 4]);
        civ.place_extras.set_name_given(1, true);
        civ.names_tracked = true;
        civ.continents.push(cartalith_civ::Continent { id: 1, name: "Old Land".into(), cells: 4, min_x: 0, min_y: 0, max_x: 3, max_y: 0, cx: 1.0, cy: 0.0, faction: 0 });
        civ
    }

    fn zo_style() -> NameStyle {
        style("Name style,Zo
Syllables,1,1
[Settlements]
Start,Start weight
zo,1
zu,1
za,1
ze,1
zi,1
zy,1
[Continents]
Template,Weight
Land of {name},1
")
    }

    /// Protects Ruling FZ item 4: Rename all renames every generated place and the continent, **never the
    /// hand-given one** (a literal name and a literal vocabulary), counts them, and one undo restores every
    /// old name exactly.
    #[test]
    fn rename_all_skips_hand_given_names_and_undoes_in_one_step() {
        let mut civ = four_places();
        let mut rng = cartalith_rng::Mulberry32::new(5);
        let (rep, undo) = rename_all(&mut civ, &zo_style(), &mut rng, false);
        assert_eq!((rep.renamed, rep.kept_hand, rep.kept_unknown, rep.continents), (3, 1, 0, 1));
        assert_eq!(civ.settlements[0].name, "Hand Named", "the hand-given name is untouched");
        for i in 1..4 {
            assert!(civ.settlements[i].name.starts_with('Z') && civ.settlements[i].name.len() == 2, "{}", civ.settlements[i].name);
        }
        assert!(civ.continents[0].name.starts_with("Land of Z"), "{}", civ.continents[0].name);
        let names: BTreeSet<&str> = civ.settlements.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names.len(), 4, "every name on the map is unique");
        undo_rename(&mut civ, &undo).unwrap();
        let back: Vec<&str> = civ.settlements.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(back, vec!["Hand Named", "Gen Two", "Gen Three", "Gen Four"]);
        assert_eq!(civ.continents[0].name, "Old Land");
    }

    /// Protects the old-save rule: a world written before the record (`names_tracked` false) renames
    /// **nothing** (every name is of unknown origin), unless `include_unknown`; and even then a name flagged
    /// hand-given stays.
    #[test]
    fn an_untracked_world_renames_nothing_unless_told_to() {
        let mut civ = four_places();
        civ.names_tracked = false;
        let mut rng = cartalith_rng::Mulberry32::new(5);
        let (rep, _) = rename_all(&mut civ, &zo_style(), &mut rng, false);
        assert_eq!((rep.renamed, rep.kept_hand, rep.kept_unknown), (0, 1, 3));
        assert_eq!(civ.settlements[1].name, "Gen Two");
        let (rep, _) = rename_all(&mut civ, &zo_style(), &mut rng, true);
        assert_eq!((rep.renamed, rep.kept_hand), (3, 1));
        assert_eq!(civ.settlements[0].name, "Hand Named");
    }

    /// Protects: an undo refuses (changing nothing) once a renamed place was named again after the rename,
    /// so it can never overwrite a newer edit.
    #[test]
    fn undo_refuses_after_a_newer_name() {
        let mut civ = four_places();
        let (_, undo) = rename_all(&mut civ, &zo_style(), &mut cartalith_rng::Mulberry32::new(9), false);
        civ.settlements[2].name = "Typed Later".into();
        assert!(undo_rename(&mut civ, &undo).is_err());
        assert_eq!(civ.settlements[2].name, "Typed Later");
        assert_ne!(civ.settlements[1].name, "Gen Two", "nothing was half-undone");
    }

    /// Protects: only landmasses no faction holds take the default style's names.
    #[test]
    fn unowned_continents_take_the_default_style() {
        let mut civ = four_places();
        civ.continents.push(cartalith_civ::Continent { id: 2, name: "Held".into(), cells: 4, min_x: 0, min_y: 0, max_x: 1, max_y: 1, cx: 0.0, cy: 0.0, faction: 3 });
        name_unowned_continents(&mut civ, &zo_style());
        assert!(civ.continents[0].name.starts_with("Land of Z"));
        assert_eq!(civ.continents[1].name, "Held");
    }

    /// Protects: a feature's name depends only on its own seed (permuting the list changes no name
    /// except where uniqueness bites), and the seed helper never returns 0.
    #[test]
    fn a_features_name_depends_on_its_own_seed() {
        // The seven-culture pool is large enough that no two of three names collide at these seeds.
        let st = name_style::culture_as_style(&cartalith_civ::CIV_CULTURES[0]);
        let a = names_for(&st, &[(11, "Rivers"), (22, "Rivers"), (33, "Cliffs")]);
        let b = names_for(&st, &[(33, "Cliffs"), (22, "Rivers")]);
        assert_eq!(a[2], b[0]);
        assert_eq!(a[1], b[1]);
        assert_ne!(feature_seed(1, 0, 0), 0);
    }
}
