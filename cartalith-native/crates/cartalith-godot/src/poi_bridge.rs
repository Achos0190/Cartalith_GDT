//! `WorldGen`'s user-placed points of interest (`CITY_MAP_LOOK_SCOPE.md` CL-11, Ruling EW item 4: *"Yes, user-placed
//! POIs"*, which reverses CV-01): the `#[func]` surface the shell's Place-POI tool, the card's Move / Rename / Delete rows,
//! the Note binding and Find read and write.
//!
//! A second `impl WorldGen` file like `annotation_bridge.rs`, and it **uses that ledger**: every mutating call goes through
//! [`WorldGen::anno_scope`] with [`AnnoStore::Pois`], so a placement, a move, a rename, a kind change and a delete are each
//! **one undo step** and a refused call (an off-grid point, an unknown kind, a bad index) records nothing.
//!
//! ## What it must never do
//!
//! Feed generation, settlement siting, the economy, a golden or a landmark pass: a POI is the user's own mark on the map.
//! It must never turn an unknown kind key into `other` (the call is refused), invent a name for an unnamed POI, or reissue
//! a retired id: [`WorldGen::poi_next_id`] only grows and is saved (`annotations/pois.json`).
//!
//! ## Identity
//!
//! A POI is addressed by its **stored id** ([`cartalith_civ::poi::Poi::id`]) for the vault (`EntityKind::Poi`, entity id =
//! the id), and by its **index** in [`WorldGen::pois`] for the shell's calls (like a label or an icon), the index shifting
//! on a delete or an undo. The shell re-reads `poi_list()` after every edit.

use crate::{annotation_bridge, annotation_undo::AnnoStore, WorldGen};
use cartalith_civ::poi::{clean_free_label, is_valid_kind_name, issue_id, kind_label_for, Poi, PoiKind, FREE_MARKS};
use godot::prelude::*;

/// One POI as a flat `Dictionary` (shared by `poi_get` and `poi_list`): `{id, x, y, kind, kind_label}` plus `name`,
/// **omitted when the user gave none** (an unnamed POI is shown by its kind; an empty string is not a name), and
/// `free_label`, **omitted when the pin has none**. `kind` is the kind **name** (the icon's name, Ruling FD);
/// `kind_label` is the user's free label when there is one, else the words for the kind (`kind_label_for`), so every
/// reader that shows `kind_label` (the card, Find, the vault) shows the user's own words for a free-text kind.
pub(crate) fn poi_dict(p: &Poi) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("id", i64::from(p.id));
    d.set("x", p.x);
    d.set("y", p.y);
    d.set("kind", p.kind.as_str());
    if p.free_label.is_empty() {
        d.set("kind_label", kind_label_for(&p.kind).as_str());
    } else {
        d.set("kind_label", p.free_label.as_str());
        d.set("free_label", p.free_label.as_str());
    }
    if !p.name.trim().is_empty() {
        d.set("name", p.name.as_str());
    }
    d
}

/// The history row's noun, and the name a row shows for a POI with none (its kind).
const NOUN: &str = "point of interest";

impl WorldGen {
    /// Whether a world exists and `(gx, gy)` is a finite point inside its grid. The one gate every placement and move
    /// shares, so a POI can never sit off the map (a saved file would then hold a point no pin can be drawn at).
    fn poi_point_ok(&self, gx: f64, gy: f64) -> bool {
        self.gw > 0 && self.gh > 0 && gx.is_finite() && gy.is_finite() && gx >= 0.0 && gy >= 0.0 && gx < f64::from(self.gw) && gy < f64::from(self.gh)
    }

    /// The name of POI `index` for a history row; `None` for an index the store does not hold.
    fn anno_poi_name(&self, index: i64) -> Option<String> {
        let i = usize::try_from(index).ok()?;
        self.pois.get(i).map(|p| p.name.clone())
    }

    /// Places a POI and returns its index, or `-1` (nothing recorded) for no world, an off-grid or non-finite point, an
    /// unknown kind name, a free label on a kind that is not a [`FREE_MARKS`] mark, or an exhausted id space. `free_label`
    /// is trimmed and capped (`clean_free_label`); an empty one is no label. Inside [`WorldGen::anno_scope`], so it is one
    /// undo step.
    pub(crate) fn poi_add_inner(&mut self, gx: f64, gy: f64, kind: &str, name: &str, free_label: &str) -> i64 {
        if !is_valid_kind_name(kind) {
            return -1;
        }
        let free = clean_free_label(free_label).unwrap_or_default();
        if !free.is_empty() && !FREE_MARKS.contains(&kind) {
            return -1;
        }
        if !self.poi_point_ok(gx, gy) {
            return -1;
        }
        let Some(id) = issue_id(&self.pois, self.poi_next_id) else { return -1 };
        self.poi_next_id = id.saturating_add(1);
        self.pois.push(Poi { id, x: gx, y: gy, kind: kind.to_string(), free_label: free, name: name.trim().to_string() });
        (self.pois.len() - 1) as i64
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The **legacy** kinds (the original twelve plus `inn`, 2026-11-15), in their picker order: `[{key, label}]`. The same list for every world. The pin
    /// palette (Ruling FD) offers every icon name instead (`poi_kind_names`); this stays for the phone's Change kind sheet
    /// and for readers that want the owner's original list.
    #[func]
    fn poi_kinds(&self) -> Array<VarDictionary> {
        PoiKind::ALL
            .iter()
            .map(|k| {
                let mut d = VarDictionary::new();
                d.set("key", k.key());
                d.set("label", k.label());
                d
            })
            .collect()
    }

    /// Every kind name a pin may carry (`cartalith_civ::poi::KIND_NAMES`, the icon sets' union, sorted) as a
    /// `PackedStringArray`; the palette's table is tested against it.
    #[func]
    fn poi_kind_names(&self) -> PackedStringArray {
        cartalith_civ::poi::KIND_NAMES.iter().map(|n| GString::from(*n)).collect()
    }

    /// Places a point of interest of kind `kind` (any kind name, see `poi_kind_names`; a legacy `poi_kinds` key is one)
    /// named `name` (may be empty) at grid `(gx, gy)`. Returns its index in `poi_list()`, or `-1` when refused (no world,
    /// off-grid, unknown kind). One undo step.
    #[func]
    fn poi_add(&mut self, gx: f64, gy: f64, kind: GString, name: GString) -> i64 {
        let (k, n) = (kind.to_string(), name.to_string());
        let label = annotation_bridge::row_label("Place", NOUN, &n);
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| s.poi_add_inner(gx, gy, &k, &n, ""))
    }

    /// As `poi_add` for a **free-text kind**: `mark` is `"other"` (a question mark) or `"notice"` (an exclamation mark) and
    /// `free_label` the user's words for the kind. Refused (`-1`, nothing recorded) for any other mark or an empty label.
    /// One undo step.
    #[func]
    fn poi_add_free(&mut self, gx: f64, gy: f64, mark: GString, free_label: GString, name: GString) -> i64 {
        let (m, f, n) = (mark.to_string(), free_label.to_string(), name.to_string());
        if clean_free_label(&f).is_none() {
            return -1;
        }
        let label = annotation_bridge::row_label("Place", NOUN, &n);
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| s.poi_add_inner(gx, gy, &m, &n, &f))
    }

    /// POI `index` as a `Dictionary` (see [`poi_dict`]), **empty** for an index the store does not hold.
    #[func]
    fn poi_get(&self, index: i64) -> VarDictionary {
        usize::try_from(index).ok().and_then(|i| self.pois.get(i)).map(poi_dict).unwrap_or_default()
    }

    /// Every POI in store order, each [`poi_dict`] plus its `index`.
    #[func]
    fn poi_list(&self) -> Array<VarDictionary> {
        self.pois
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut d = poi_dict(p);
                d.set("index", i as i64);
                d
            })
            .collect()
    }

    /// Moves POI `index` to `(gx, gy)`. `false` (nothing recorded) for a bad index or an off-grid point. Its note stays
    /// bound: the id does not change. One undo step.
    #[func]
    fn poi_move(&mut self, index: i64, gx: f64, gy: f64) -> bool {
        let label = annotation_bridge::row_label("Move", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        let key = Some(format!("move:poi:{index}"));
        self.anno_scope(AnnoStore::Pois, label, key, None, |s| {
            let Ok(i) = usize::try_from(index) else { return false };
            if i >= s.pois.len() || !s.poi_point_ok(gx, gy) {
                return false;
            }
            s.pois[i].x = gx;
            s.pois[i].y = gy;
            true
        })
    }

    /// Renames POI `index` (an empty name makes it unnamed). `false` for a bad index; a rename to the same text records
    /// nothing. One undo step.
    #[func]
    fn poi_rename(&mut self, index: i64, name: GString) -> bool {
        let n = name.to_string().trim().to_string();
        let label = annotation_bridge::row_label("Rename", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| {
            let Some(p) = usize::try_from(index).ok().and_then(|i| s.pois.get_mut(i)) else { return false };
            p.name = n;
            true
        })
    }

    /// Changes POI `index`'s kind to `kind` (any kind name). The pin's free label is cleared: a named kind replaces the
    /// user's own words. `false` for a bad index or an unknown name (never `other`). One undo step.
    #[func]
    fn poi_set_kind(&mut self, index: i64, kind: GString) -> bool {
        let k = kind.to_string();
        if !is_valid_kind_name(&k) {
            return false;
        }
        let label = annotation_bridge::row_label("Change kind of", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| {
            let Some(p) = usize::try_from(index).ok().and_then(|i| s.pois.get_mut(i)) else { return false };
            p.kind = k;
            p.free_label.clear();
            true
        })
    }

    /// Gives POI `index` a free-text kind: `mark` (`"other"` or `"notice"`) is its icon and `free_label` its words.
    /// `false` for a bad index, another mark or an empty label. One undo step.
    #[func]
    fn poi_set_free_kind(&mut self, index: i64, mark: GString, free_label: GString) -> bool {
        let m = mark.to_string();
        let Some(f) = clean_free_label(&free_label.to_string()) else { return false };
        if !FREE_MARKS.contains(&m.as_str()) {
            return false;
        }
        let label = annotation_bridge::row_label("Change kind of", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| {
            let Some(p) = usize::try_from(index).ok().and_then(|i| s.pois.get_mut(i)) else { return false };
            p.kind = m;
            p.free_label = f;
            true
        })
    }

    /// Deletes POI `index`. `false` for a bad index. Its note link stays in the vault as an absent entity (the id is
    /// retired, never reissued), so undo restores the POI with its note still bound. One undo step.
    #[func]
    fn poi_delete(&mut self, index: i64) -> bool {
        let label = annotation_bridge::row_label("Delete", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| {
            let Ok(i) = usize::try_from(index) else { return false };
            if i >= s.pois.len() {
                return false;
            }
            s.pois.remove(i);
            true
        })
    }

    /// The vault binding of POI `index`: `{ok, entity_id, label}` (the entity id **is** the stored id; the label is its
    /// name, else its kind and cell), or `{ok: false, error}` for an index the store does not hold. The vault's own kind is
    /// `"poi"`.
    #[func]
    fn vault_poi_entity_id(&self, index: i64) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(p) = usize::try_from(index).ok().and_then(|i| self.pois.get(i)) else {
            d.set("ok", false);
            d.set("error", "No point of interest at that index.");
            return d;
        };
        d.set("ok", true);
        d.set("entity_id", i64::from(p.id));
        let name = p.name.trim();
        let words = if p.free_label.is_empty() { kind_label_for(&p.kind) } else { p.free_label.clone() };
        let text = if name.is_empty() { format!("{} ({}, {})", words, p.x.round(), p.y.round()) } else { name.to_string() };
        d.set("label", text.as_str());
        d
    }
}


// ---------------------------------------------------------------------------------------------------------------------
// `annotations/pois.json` (SAVEFILE_COMPAT.md, "annotations/pois.json"): the saved form, as pure functions so the round
// trip and the refusals are unit-tested without a live `WorldGen` (a cdylib `GodotClass`; see `MISTAKES.md`).
// ---------------------------------------------------------------------------------------------------------------------

/// `annotations/pois.json`: `{"next_id": n, "pois": [{"id", "x", "y", "kind", "kind_name"?, "free_label"?, "name"?}]}`. **Optional**: written only when
/// there is a POI or the id counter has moved past its start (so a deleted POI's id stays retired across a save); a world
/// that never had one writes no such member, and so every save written before this slot existed re-serialises byte for byte.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct PoisDoc {
    /// The counter [`WorldGen::poi_next_id`] resumes from. Absent or `0` reads as "from the start" (the writer never writes 0).
    #[serde(default)]
    pub next_id: u32,
    /// The points of interest, in placement order.
    #[serde(default)]
    pub pois: Vec<PoiDto>,
}

/// One row of `annotations/pois.json`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct PoiDto {
    /// The stored id (`>= 1`).
    #[serde(default)]
    pub id: u32,
    /// Grid x, continuous cell units.
    #[serde(default)]
    pub x: f64,
    /// Grid y.
    #[serde(default)]
    pub y: f64,
    /// A legacy `PoiKind::key()`. For a pin whose kind is one of the original twelve (not `inn`, see `PoiKind::is_original_wire_key`) this IS its kind (an old save has only this);
    /// for any other kind name it is `other` (so a build from before Ruling FD still opens the row, as a question mark)
    /// and the real name is `kind_name`. Unknown keys are refused on read, never turned into `other`.
    #[serde(default)]
    pub kind: String,
    /// The pin's kind name when it is not one of the twelve legacy keys (`waterfall`, `capital`...); **omitted otherwise**,
    /// so a world whose pins are all legacy kinds re-serialises byte for byte (Ruling FD item 2). Wins over `kind`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind_name: String,
    /// The user's own words for the kind (a free-text kind, whose icon is `other` or `notice`); **omitted when none**.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub free_label: String,
    /// The user's name; **omitted when none** (an empty string is not a name).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
}

/// The document to save for `pois` and counter `next_id`, or `None` when there is nothing to say (no POI and the counter
/// never moved), so the member is omitted and an old save is unchanged.
pub(crate) fn pois_to_doc(pois: &[Poi], next_id: u32) -> Option<PoisDoc> {
    if pois.is_empty() && next_id <= 1 {
        return None;
    }
    Some(PoisDoc {
        next_id,
        pois: pois
            .iter()
            .map(|p| {
                let legacy = PoiKind::is_original_wire_key(&p.kind);
                PoiDto {
                    id: p.id,
                    x: p.x,
                    y: p.y,
                    kind: if legacy { p.kind.clone() } else { "other".to_string() },
                    kind_name: if legacy { String::new() } else { p.kind.clone() },
                    free_label: p.free_label.clone(),
                    name: p.name.clone(),
                }
            })
            .collect(),
    })
}

/// What a saved document restores to: the POIs, the counter, and how many rows were dropped (reported by the caller as a
/// warning, `SAVEFILE_COMPAT.md` section 6.4a: skip the element, keep the rest, report it).
pub(crate) struct PoisRestored {
    /// The rows that resolved.
    pub pois: Vec<Poi>,
    /// The counter to resume from: the saved one, lifted past the largest restored id by [`issue_id`]'s rule.
    pub next_id: u32,
    /// Rows skipped: an unknown kind, a non-finite or off-grid point, id `0`, or an id already seen (the first wins).
    pub dropped: usize,
}

/// Resolves a saved document over a `gw` x `gh` grid. Never invents: a row that cannot be read is dropped and counted.
pub(crate) fn pois_from_doc(doc: &PoisDoc, gw: i32, gh: i32) -> PoisRestored {
    let mut pois: Vec<Poi> = Vec::new();
    let mut dropped = 0usize;
    for d in &doc.pois {
        // `kind_name` wins when present; an old save has only `kind`, whose key is already a valid name.
        let name = if d.kind_name.is_empty() { d.kind.as_str() } else { d.kind_name.as_str() };
        let free = clean_free_label(&d.free_label).unwrap_or_default();
        // A free label belongs to a mark kind only: a row that pairs one with a picture kind is unreadable, not repaired.
        let kind = (is_valid_kind_name(name) && (free.is_empty() || FREE_MARKS.contains(&name))).then(|| name.to_string());
        let on_grid = d.x.is_finite() && d.y.is_finite() && d.x >= 0.0 && d.y >= 0.0 && d.x < f64::from(gw) && d.y < f64::from(gh);
        match kind {
            Some(kind) if on_grid && d.id >= 1 && !pois.iter().any(|p| p.id == d.id) => {
                pois.push(Poi { id: d.id, x: d.x, y: d.y, kind, free_label: free, name: d.name.trim().to_string() });
            }
            _ => dropped += 1,
        }
    }
    // A saved counter lifted past the restored ids; `None` (an id of u32::MAX restored) keeps the saved counter as is.
    let next_id = issue_id(&pois, doc.next_id).unwrap_or(doc.next_id.max(1));
    PoisRestored { pois, next_id, dropped }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poi(id: u32, kind: &str, name: &str) -> Poi {
        Poi { id, x: 10.5 + f64::from(id), y: 20.25, kind: kind.to_string(), free_label: String::new(), name: name.to_string() }
    }

    fn row(id: u32, x: f64, y: f64, kind: &str) -> PoiDto {
        PoiDto { id, x, y, kind: kind.into(), kind_name: String::new(), free_label: String::new(), name: String::new() }
    }

    /// Protects: a world that never had a POI writes NO member (so every older save re-serialises unchanged), and one that
    /// had one but deleted it still writes the counter (a retired id is never reissued after a reload).
    #[test]
    fn the_member_is_omitted_until_a_poi_was_ever_placed() {
        assert!(pois_to_doc(&[], 1).is_none(), "a fresh world writes no pois.json");
        let kept = pois_to_doc(&[], 4).expect("a deleted POI's counter must be saved");
        assert!(kept.pois.is_empty());
        assert_eq!(kept.next_id, 4);
    }

    /// Protects: save, read, save again gives the same text (byte-identical round trip), the name is omitted when blank, and
    /// every field survives.
    #[test]
    fn a_saved_document_round_trips_byte_for_byte() {
        let pois = vec![poi(1, "tavern", "The Gilded Mug"), poi(3, "well", "")];
        let doc = pois_to_doc(&pois, 5).unwrap();
        let text = serde_json::to_string_pretty(&doc).unwrap();
        assert!(!text.contains("\"name\": \"\""), "a blank name must be omitted: {text}");
        let back: PoisDoc = serde_json::from_str(&text).unwrap();
        let r = pois_from_doc(&back, 512, 512);
        assert_eq!(r.dropped, 0);
        assert_eq!(r.pois, pois);
        assert_eq!(r.next_id, 5, "the saved counter (ahead of the largest id 3) is kept");
        let text2 = serde_json::to_string_pretty(&pois_to_doc(&r.pois, r.next_id).unwrap()).unwrap();
        assert_eq!(text, text2);
    }

    /// Protects: every refusal drops the ROW and counts it, never substituting a plausible value (an unknown kind is not
    /// `other`, a bad point is not clamped, id 0 and a repeated id are not renumbered).
    #[test]
    fn unreadable_rows_are_dropped_and_counted_not_repaired() {
        let doc = PoisDoc {
            next_id: 0,
            pois: vec![
                row(1, 5.0, 5.0, "tavern"),
                row(2, 5.0, 5.0, "brothel"),
                row(3, -1.0, 5.0, "shop"),
                row(4, 5.0, 600.0, "shop"),
                row(5, f64::NAN, 5.0, "shop"),
                row(0, 5.0, 5.0, "shop"),
                row(1, 6.0, 6.0, "gate"),
                row(7, 9.0, 9.0, "other"),
            ],
        };
        let r = pois_from_doc(&doc, 512, 512);
        assert_eq!(r.pois.iter().map(|p| p.id).collect::<Vec<_>>(), vec![1, 7]);
        assert_eq!(r.dropped, 6);
        assert_eq!(r.pois[0].kind, "tavern", "the first of a repeated id wins");
        assert_eq!(r.next_id, 8, "a counter of 0 is lifted past the largest restored id");
    }

    /// Protects (the old-save round trip, Ruling FD item 2): a document written by the build BEFORE kind names (a literal,
    /// not a fixture of empty collections) opens with every pin's kind as the same string, and saving it again gives
    /// the same text byte for byte: no `kind_name` and no `free_label` appear for legacy kinds.
    #[test]
    fn a_save_from_before_kind_names_opens_unchanged_and_re_saves_byte_for_byte() {
        let old = r#"{
  "next_id": 4,
  "pois": [
    {
      "id": 1,
      "x": 11.5,
      "y": 20.25,
      "kind": "tavern",
      "name": "The Gilded Mug"
    },
    {
      "id": 3,
      "x": 13.5,
      "y": 20.25,
      "kind": "shop"
    },
    {
      "id": 2,
      "x": 12.5,
      "y": 20.25,
      "kind": "other",
      "name": "Odd place"
    }
  ]
}"#;
        let doc: PoisDoc = serde_json::from_str(old).unwrap();
        let r = pois_from_doc(&doc, 512, 512);
        assert_eq!(r.dropped, 0);
        let kinds: Vec<&str> = r.pois.iter().map(|p| p.kind.as_str()).collect();
        assert_eq!(kinds, ["tavern", "shop", "other"], "an old save's PoiKind loads as its own name");
        assert!(r.pois.iter().all(|p| p.free_label.is_empty()), "an old pin has no free label");
        assert_eq!(r.next_id, 4);
        let again = serde_json::to_string_pretty(&pois_to_doc(&r.pois, r.next_id).unwrap()).unwrap();
        assert_eq!(again, old, "re-saving an old document must not add a member");
    }

    /// Protects (PP-7): an inn saved by `cf430ea4`'s kind-name path (a literal: `kind: "other"`, `kind_name: "inn"`) loads as
    /// the inn, the writer still emits exactly that form (so an older build opens the row and a re-save is byte-identical),
    /// and a hand-written `kind: "inn"` row is read as an inn too. Also: `PoiKind::Inn` is a legacy kind with its own words.
    #[test]
    fn an_inn_saved_by_the_kind_name_path_loads_as_inn_and_re_saves_byte_for_byte() {
        let old = r#"{
  "next_id": 3,
  "pois": [
    {
      "id": 1,
      "x": 11.5,
      "y": 20.25,
      "kind": "other",
      "kind_name": "inn",
      "name": "The Lantern"
    },
    {
      "id": 2,
      "x": 12.5,
      "y": 20.25,
      "kind": "tavern"
    }
  ]
}"#;
        let doc: PoisDoc = serde_json::from_str(old).unwrap();
        let r = pois_from_doc(&doc, 512, 512);
        assert_eq!(r.dropped, 0);
        assert_eq!(r.pois[0].kind, "inn");
        assert_eq!(PoiKind::from_key(&r.pois[0].kind), Some(PoiKind::Inn));
        assert_eq!(kind_label_for("inn"), "Inn");
        let again = serde_json::to_string_pretty(&pois_to_doc(&r.pois, r.next_id).unwrap()).unwrap();
        assert_eq!(again, old, "an inn keeps the wire form an older build already reads");
        let hand: PoisDoc = serde_json::from_str(r#"{"next_id":2,"pois":[{"id":1,"x":1.0,"y":1.0,"kind":"inn"}]}"#).unwrap();
        assert_eq!(pois_from_doc(&hand, 512, 512).pois[0].kind, "inn", "a bare `kind: inn` is read, not refused");
    }

    /// Protects: a new kind name and a free-text kind survive a save and a load, an older build can still open the row (its
    /// `kind` is the legacy `other`), and a free label on a picture kind is a dropped row, not a repaired one.
    #[test]
    fn a_kind_name_and_a_free_label_round_trip() {
        let mut free = poi(2, "notice", "Hive field");
        free.free_label = "Beehives".to_string();
        let pois = vec![poi(1, "waterfall", ""), free.clone(), poi(3, "well", "")];
        let doc = pois_to_doc(&pois, 4).unwrap();
        assert_eq!(doc.pois[0].kind, "other", "a pre-FD build reads a non-legacy name as the stated fallback");
        assert_eq!(doc.pois[0].kind_name, "waterfall");
        assert_eq!(doc.pois[2].kind_name, "", "a legacy kind writes no kind_name");
        let text = serde_json::to_string_pretty(&doc).unwrap();
        assert!(text.contains("\"free_label\": \"Beehives\""), "{text}");
        let back: PoisDoc = serde_json::from_str(&text).unwrap();
        let r = pois_from_doc(&back, 512, 512);
        assert_eq!(r.dropped, 0);
        assert_eq!(r.pois, pois);
        assert_eq!(r.pois[1].free_label, "Beehives");
        // A free label on a picture kind, an unknown kind_name and an over-long-but-valid label.
        let mut bad = row(5, 5.0, 5.0, "tavern");
        bad.free_label = "Not a mark".into();
        let mut unknown = row(6, 5.0, 5.0, "other");
        unknown.kind_name = "brothel".into();
        let r2 = pois_from_doc(&PoisDoc { next_id: 1, pois: vec![bad, unknown] }, 512, 512);
        assert!(r2.pois.is_empty());
        assert_eq!(r2.dropped, 2);
    }
}
