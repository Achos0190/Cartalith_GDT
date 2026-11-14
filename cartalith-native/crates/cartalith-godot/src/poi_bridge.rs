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
use cartalith_civ::poi::{issue_id, Poi, PoiKind};
use godot::prelude::*;

/// One POI as a flat `Dictionary` (shared by `poi_get` and `poi_list`): `{id, x, y, kind, kind_label}` plus `name`,
/// **omitted when the user gave none** (an unnamed POI is shown by its kind; an empty string is not a name).
pub(crate) fn poi_dict(p: &Poi) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("id", i64::from(p.id));
    d.set("x", p.x);
    d.set("y", p.y);
    d.set("kind", p.kind.key());
    d.set("kind_label", p.kind.label());
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
    /// unknown kind key, or an exhausted id space. Inside [`WorldGen::anno_scope`], so it is one undo step.
    pub(crate) fn poi_add_inner(&mut self, gx: f64, gy: f64, kind: &str, name: &str) -> i64 {
        let Some(kind) = PoiKind::from_key(kind) else { return -1 };
        if !self.poi_point_ok(gx, gy) {
            return -1;
        }
        let Some(id) = issue_id(&self.pois, self.poi_next_id) else { return -1 };
        self.poi_next_id = id.saturating_add(1);
        self.pois.push(Poi { id, x: gx, y: gy, kind, name: name.trim().to_string() });
        (self.pois.len() - 1) as i64
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The kinds the Place tool offers, in picker order: `[{key, label}]`. The same list for every world.
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

    /// Places a point of interest of kind `kind` (a key from `poi_kinds`) named `name` (may be empty) at grid `(gx, gy)`.
    /// Returns its index in `poi_list()`, or `-1` when refused (no world, off-grid, unknown kind). One undo step.
    #[func]
    fn poi_add(&mut self, gx: f64, gy: f64, kind: GString, name: GString) -> i64 {
        let (k, n) = (kind.to_string(), name.to_string());
        let label = annotation_bridge::row_label("Place", NOUN, &n);
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| s.poi_add_inner(gx, gy, &k, &n))
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

    /// Changes POI `index`'s kind to `kind` (a key from `poi_kinds`). `false` for a bad index or an unknown key (never
    /// `other`). One undo step.
    #[func]
    fn poi_set_kind(&mut self, index: i64, kind: GString) -> bool {
        let Some(k) = PoiKind::from_key(&kind.to_string()) else { return false };
        let label = annotation_bridge::row_label("Change kind of", NOUN, &self.anno_poi_name(index).unwrap_or_default());
        self.anno_scope(AnnoStore::Pois, label, None, None, |s| {
            let Some(p) = usize::try_from(index).ok().and_then(|i| s.pois.get_mut(i)) else { return false };
            p.kind = k;
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
        let text = if name.is_empty() { format!("{} ({}, {})", p.kind.label(), p.x.round(), p.y.round()) } else { name.to_string() };
        d.set("label", text.as_str());
        d
    }
}


// ---------------------------------------------------------------------------------------------------------------------
// `annotations/pois.json` (SAVEFILE_COMPAT.md, "annotations/pois.json"): the saved form, as pure functions so the round
// trip and the refusals are unit-tested without a live `WorldGen` (a cdylib `GodotClass`; see `MISTAKES.md`).
// ---------------------------------------------------------------------------------------------------------------------

/// `annotations/pois.json`: `{"next_id": n, "pois": [{"id", "x", "y", "kind", "name"?}]}`. **Optional**: written only when
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
    /// A `PoiKind::key()`. Unknown keys are refused on read, never turned into `other`.
    #[serde(default)]
    pub kind: String,
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
        pois: pois.iter().map(|p| PoiDto { id: p.id, x: p.x, y: p.y, kind: p.kind.key().to_string(), name: p.name.clone() }).collect(),
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
        let kind = PoiKind::from_key(&d.kind);
        let on_grid = d.x.is_finite() && d.y.is_finite() && d.x >= 0.0 && d.y >= 0.0 && d.x < f64::from(gw) && d.y < f64::from(gh);
        match kind {
            Some(kind) if on_grid && d.id >= 1 && !pois.iter().any(|p| p.id == d.id) => {
                pois.push(Poi { id: d.id, x: d.x, y: d.y, kind, name: d.name.trim().to_string() });
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

    fn poi(id: u32, kind: PoiKind, name: &str) -> Poi {
        Poi { id, x: 10.5 + f64::from(id), y: 20.25, kind, name: name.to_string() }
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
        let pois = vec![poi(1, PoiKind::Tavern, "The Gilded Mug"), poi(3, PoiKind::Well, "")];
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
        let row = |id: u32, x: f64, y: f64, kind: &str| PoiDto { id, x, y, kind: kind.into(), name: String::new() };
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
        assert_eq!(r.pois[0].kind, PoiKind::Tavern, "the first of a repeated id wins");
        assert_eq!(r.next_id, 8, "a counter of 0 is lifted past the largest restored id");
    }
}
