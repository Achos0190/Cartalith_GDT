//! The Asset Library model: `AssetDB`'s slot registry, `AssetCollections`,
//! `AssetValidator`, and the `assetlib/library.json` record shape.
//!
//! Ported from `Cartalith Gen1 v2.10.html`:
//! - `mkSlots`/`FAMILIES`/`AssetDB`/`AssetCollections`/`itemHash`/
//!   `AssetValidator` (lines 26781-26961).
//! - `_alExportEntries`/`_alImportProject`'s **shape** (lines 27879-27928) —
//!   the record schema and the pack-info/collections/rules restoration logic.
//!   Image decode/encode (`encodeItemPng`, `AssetImporter.decodeBytes`) is
//!   milestone 6's job; nothing here touches a pixel.
//!
//! # What "pure data management; no images" means here
//!
//! [`LibraryItem`] carries a `hash: String` — the reference's `itemHash(img,
//! w, h)` — but this crate never computes one: it is always supplied by the
//! caller (a test fixture today, milestone 6's real decode step later). That
//! keeps [`run`]'s duplicate-image detection fully
//! implementable and golden-testable without decoding a single PNG.
//!
//! [`AssetDB::apply_library_file`] restores everything a parsed
//! [`LibraryFile`] carries **except** items: pack info, collections
//! (unvalidated — see its own docs), and per-slot metadata/scatter rules. Item
//! restoration needs real image bytes to compute a hash and is therefore left
//! to milestone 6, which can reuse [`SlotRecord::items`]' `img` indices to
//! find its bytes.
//!
//! # A real finding: the Library's own `poi` vocabulary is not the pack
//! vocabulary
//!
//! `AssetDB`'s `poi` family bootstraps from the Asset Library's own `FAMILIES`
//! table, which carries **ten** slots (`lake`/`bridge` included) — not
//! [`crate::PACK_POI_SLOTS`]' eight, which is the *pack-import* vocabulary
//! milestone 1 ported (`lake`/`bridge` have no engine POI kind to attach to,
//! so a pack can never carry art for them even though the Library lets you
//! author it). See [`LIBRARY_POI_SLOTS`].
//!
//! # A real finding: per-slot display names are functionally load-bearing
//! after all
//!
//! `ASSET_LIBRARY_SCOPE.md`'s milestone 1 filed per-slot titles (`mkSlots`'s
//! `name`/`desc`/`code` columns) as presentational UI text, out of scope.
//! That is true for `desc`/`code` — genuinely never read outside the browser
//! UI — but **not** for `name`: `AssetValidator.run()`'s "Identical images"
//! warning renders `slot.name`, not `slot.id`
//! (`SLOT_REG[e.uid].slot.name+'#'+(e.idx+1)`), and a golden run confirms it
//! (`"Identical images: Mountain#1 = Hill#1"`, not `mountain#1 = hill#1`).
//! Getting this port's frozen-slot titles wrong would silently break golden
//! parity on that one message. See [`slot_title`].

use crate::ordered_map::OrderedMap;
use crate::scatter::{ScatterRule, normalize_scatter_rule, scatter_rule_key};
use crate::slots::{Family, slug_id};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt;

// ---------------------------------------------------------------------------
// The Library's own `poi` vocabulary (ten slots; see the module docs)
// ---------------------------------------------------------------------------

/// The Asset Library's own `poi` slot list — ten slots, unlike the eight of
/// [`crate::PACK_POI_SLOTS`]. `lake` and `bridge` can be authored here (and
/// exported into a pack's manifest) but never resolve to art on the live map,
/// exactly as [`crate::PACK_POI_SLOTS`]' own doc comment already explains.
pub const LIBRARY_POI_SLOTS: [&str; 10] = [
    "ruin",
    "landmark",
    "mountain_peak",
    "lake",
    "named_forest",
    "battlefield",
    "shrine",
    "cave",
    "bridge",
    "other",
];

/// The frozen-vocabulary slot list `AssetDB` actually bootstraps a family
/// from — [`Family::slots`] for every family except [`Family::Poi`], where it
/// is [`LIBRARY_POI_SLOTS`] rather than the narrower pack-import list.
pub fn library_slot_ids(family: Family) -> &'static [&'static str] {
    if family == Family::Poi {
        &LIBRARY_POI_SLOTS
    } else {
        family.slots()
    }
}

/// The reference's `mkSlots` display title for a frozen slot — the second
/// column of each `FAMILIES` row (line 26784-26819). Functionally load-bearing
/// for [`run`]'s "Identical images" message; see the module
/// docs. Not meaningful for [`Family::Custom`], whose slots carry a
/// user-authored name instead.
///
/// # Panics
/// If `id` is not one of `family`'s [`library_slot_ids`]. Every call site in
/// this crate only ever calls this while bootstrapping the frozen vocabulary
/// itself, so the pair is always valid by construction.
pub fn slot_title(family: Family, id: &str) -> &'static str {
    match (family, id) {
        (Family::Textures, "grass") => "Grass",
        (Family::Textures, "rock") => "Rock",
        (Family::Textures, "sand") => "Sand",
        (Family::Textures, "snow") => "Snow",
        (Family::Textures, "wetland") => "Wetland",
        (Family::Textures, "canopy") => "Canopy",
        (Family::Textures, "parchment") => "Parchment",

        (Family::Biomes, "coastal") => "Coastal Lowland",
        (Family::Biomes, "temperate_forest") => "Temperate Forest",
        (Family::Biomes, "mediterranean") => "Mediterranean Scrub",
        (Family::Biomes, "wetlands") => "Wetlands / Marshes",
        (Family::Biomes, "steppe") => "Steppe / Grassland",
        (Family::Biomes, "jungle") => "Tropical Jungle",
        (Family::Biomes, "boreal") => "Boreal Taiga",
        (Family::Biomes, "mountain") => "Mountain Highland",
        (Family::Biomes, "cold_desert") => "Cold Desert / Badlands",
        (Family::Biomes, "hot_desert") => "Hot Desert",
        (Family::Biomes, "tundra") => "Tundra / Polar",
        (Family::Biomes, "ruined") => "Ruined Wastes",
        (Family::Biomes, "hills") => "Hills",
        (Family::Biomes, "lake_river") => "Lake / River",
        (Family::Biomes, "ocean") => "Ocean / Deep Water",

        (Family::Terrains, "paved") => "Paved Road",
        (Family::Terrains, "dirt") => "Dirt Track",
        (Family::Terrains, "hardpack") => "Desert Hardpack",
        (Family::Terrains, "plains") => "Open Plains",
        (Family::Terrains, "forest_path") => "Forest Path",
        (Family::Terrains, "hills") => "Hills",
        (Family::Terrains, "rocky") => "Rocky Terrain",
        (Family::Terrains, "mtn_pass") => "Mountain Pass",
        (Family::Terrains, "mtn_trail") => "Mountain Trails",
        (Family::Terrains, "swamp") => "Swamp / Marsh",
        (Family::Terrains, "deep_sand") => "Deep Sand",
        (Family::Terrains, "snow") => "Snow / Ice",
        (Family::Terrains, "ruins") => "Ruins / Debris",

        (Family::Icons, "mountain") => "Mountain",
        (Family::Icons, "hill") => "Hill",
        (Family::Icons, "tree_conifer") => "Conifer tree",
        (Family::Icons, "tree_broadleaf") => "Broadleaf tree",
        (Family::Icons, "tree_rainforest") => "Rainforest tree",
        (Family::Icons, "tree_savanna") => "Savanna tree",
        (Family::Icons, "tree_wetland") => "Wetland tree",
        (Family::Icons, "shrub") => "Shrub",
        (Family::Icons, "cactus") => "Cactus",
        (Family::Icons, "boulder") => "Boulder",

        (Family::Settlement, "hamlet") => "Hamlet",
        (Family::Settlement, "village") => "Village",
        (Family::Settlement, "town") => "Town",
        (Family::Settlement, "city") => "City",
        (Family::Settlement, "capital") => "Capital",
        (Family::Settlement, "monastery") => "Monastery",
        (Family::Settlement, "fortress") => "Fortress",
        (Family::Settlement, "university") => "University",
        (Family::Settlement, "industrial") => "Industrial",

        (Family::Trait, "fortified") => "Fortified",
        (Family::Trait, "mining") => "Mining",
        (Family::Trait, "port") => "Port",
        (Family::Trait, "administrative") => "Administrative",
        (Family::Trait, "trade_hub") => "Trade hub",
        (Family::Trait, "military") => "Military",
        (Family::Trait, "religious") => "Religious",

        (Family::Poi, "ruin") => "Ruin / old settlement",
        (Family::Poi, "landmark") => "Landmark",
        (Family::Poi, "mountain_peak") => "Mountain peak",
        (Family::Poi, "lake") => "Lake / spring",
        (Family::Poi, "named_forest") => "Named forest",
        (Family::Poi, "battlefield") => "Battlefield",
        (Family::Poi, "shrine") => "Shrine / temple",
        (Family::Poi, "cave") => "Cave / tunnel",
        (Family::Poi, "bridge") => "Bridge / ford",
        (Family::Poi, "other") => "Other",

        _ => panic!("no display title for {}:{id}", family.key()),
    }
}

/// Whether a family's assets can carry a [`ScatterRule`] at all — the
/// reference's `famScatters`. Only feature icons and the open custom
/// vocabulary scatter procedurally; settlements/traits/POIs are placed from
/// real civ data and ground textures are tiled fill, not sprites.
fn fam_scatters(family: Family) -> bool {
    matches!(family, Family::Icons | Family::Custom)
}

fn make_uid(family: Family, set_id: Option<&str>, id: &str) -> String {
    match set_id {
        Some(s) => format!("{}:{}/{}", family.key(), s, id),
        None => format!("{}:{}", family.key(), id),
    }
}

// ---------------------------------------------------------------------------
// Per-slot metadata and per-item transform
// ---------------------------------------------------------------------------

/// Free-form per-slot metadata — the reference's `defaultMeta()`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SlotMeta {
    pub author: String,
    pub copyright: String,
    pub license: String,
    pub source: String,
    pub notes: String,
    pub version: String,
    pub tags: Vec<String>,
}

/// Merge untrusted JSON onto [`SlotMeta::default`] — the reference's
/// `Object.assign(defaultMeta(), rec.meta||{})`, hardened rather than
/// transcribed: a key that is present but the wrong JSON type is dropped
/// (falls back to the default) instead of being carried through as garbage,
/// the same divergence [`crate::normalize_scatter_rule`] already documents
/// for `biomes`/`density`. `tags` keeps only the array's string elements.
fn normalize_meta(v: Option<&Value>) -> SlotMeta {
    let base = SlotMeta::default();
    let Some(obj) = v.and_then(Value::as_object) else {
        return base;
    };
    let s = |key: &str| {
        obj.get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_default()
    };
    let tags = obj
        .get("tags")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|t| t.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    SlotMeta {
        author: s("author"),
        copyright: s("copyright"),
        license: s("license"),
        source: s("source"),
        notes: s("notes"),
        version: s("version"),
        tags,
    }
}

/// Per-item scale/pan transform — the reference's `defaultTransform()`
/// (`{scale:1,panX:0,panY:0}`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemTransform {
    pub scale: f64,
    #[serde(rename = "panX")]
    pub pan_x: f64,
    #[serde(rename = "panY")]
    pub pan_y: f64,
}

impl Default for ItemTransform {
    fn default() -> Self {
        ItemTransform {
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}

/// Merge untrusted JSON onto [`ItemTransform::default`] — the reference's
/// `Object.assign(defaultTransform(), im.t||{})`. A present-but-non-numeric
/// field falls back to its default rather than propagating a non-`f64`
/// value, which the type system would not accept anyway.
fn normalize_transform(v: Option<&Value>) -> ItemTransform {
    let base = ItemTransform::default();
    let Some(obj) = v.and_then(Value::as_object) else {
        return base;
    };
    let num = |key: &str, dflt: f64| obj.get(key).and_then(Value::as_f64).unwrap_or(dflt);
    ItemTransform {
        scale: num("scale", base.scale),
        pan_x: num("panX", base.pan_x),
        pan_y: num("panY", base.pan_y),
    }
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

/// One imported/authored image bound to a slot — the reference's store item
/// (`{name,img,w,h,t,hash}`), minus the pixels: `img`/`w`/`h` are milestone
/// 6's job. `hash` is the reference's `itemHash(img,w,h)` — always supplied
/// by the caller here rather than computed, so [`run`]'s
/// duplicate-image detection is fully testable without an image decoder.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryItem {
    pub name: String,
    pub hash: String,
    pub transform: ItemTransform,
}

impl LibraryItem {
    /// A new item with the default (identity) transform.
    pub fn new(name: impl Into<String>, hash: impl Into<String>) -> Self {
        LibraryItem {
            name: name.into(),
            hash: hash.into(),
            transform: ItemTransform::default(),
        }
    }

    /// Builder-style: override the transform.
    pub fn with_transform(mut self, t: ItemTransform) -> Self {
        self.transform = t;
        self
    }
}

// ---------------------------------------------------------------------------
// Slots
// ---------------------------------------------------------------------------

/// One slot in the registry — a frozen-vocabulary entry or a user-defined
/// custom icon. The reference's slot object (`{id,name,uid,set?,setId?,
/// meta,rules?}`, plus the presentational `desc`/`code` this port does not
/// carry — see the module docs on why `name` still is).
#[derive(Debug, Clone, PartialEq)]
pub struct LibrarySlot {
    /// Filename-safe id: a frozen vocabulary entry, or [`slug_id`] of a
    /// custom slot's display name.
    pub id: String,
    /// Display name: the reference's pretty title for a frozen slot
    /// ([`slot_title`]), or the user's own text for a custom one.
    pub name: String,
    /// `family:id` (frozen) or `family:setId/id` (custom) — the reference's
    /// `slot.uid`, and the key both [`AssetDB`]'s store and
    /// [`AssetCollections`] address a slot by.
    pub uid: String,
    pub family: Family,
    /// Raw, user-typed set name — [`Family::Custom`] only. `None` for every
    /// frozen family.
    pub set: Option<String>,
    /// [`slug_id`] of `set` — [`Family::Custom`] only. Both the raw name and
    /// its slug are carried, per milestone 3's own finding: the exporter's
    /// path uses the slug (`custom/naval/…`) while the manifest key and this
    /// slot's own scatter-rule key use the raw text (`"Naval"`).
    pub set_id: Option<String>,
    pub meta: SlotMeta,
    /// `Some` only when [`fam_scatters`] is true for this slot's family, and
    /// even then only once something has read it — see [`AssetDB::slot_rules`].
    pub rules: Option<ScatterRule>,
}

impl LibrarySlot {
    /// The reference's `slotRuleKey`: a custom slot addresses its rule by
    /// `custom::<raw set name>::<id>`; a frozen scatterable slot by its bare
    /// id. `None` for a family that cannot scatter at all.
    pub fn rule_key(&self) -> Option<String> {
        if !fam_scatters(self.family) {
            return None;
        }
        Some(if self.family == Family::Custom {
            scatter_rule_key(&self.id, Some(self.set.as_deref().unwrap_or("Default")))
        } else {
            self.id.clone()
        })
    }
}

// ---------------------------------------------------------------------------
// AssetDB
// ---------------------------------------------------------------------------

/// The pack fields a Library carries alongside its slots — the reference's
/// `E('alPackName'|'alPackAuthor'|'alPackLicense')` DOM triple, promoted to
/// real data since this port has no DOM.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PackInfo {
    pub name: String,
    pub author: String,
    pub license: String,
}

/// The slot registry, item store, pack fields and collections — the
/// reference's `AssetDB` plus the three pack-field DOM globals it reads
/// through `E(...)`.
#[derive(Debug, Clone)]
pub struct AssetDB {
    slots: HashMap<String, LibrarySlot>,
    store: HashMap<String, Vec<LibraryItem>>,
    /// Every frozen-family uid, in `FAMILIES` bootstrap order. Immutable
    /// after construction — frozen family membership never changes.
    frozen_order: Vec<String>,
    /// Custom-family uids, in the order they were added. The only family
    /// whose membership is mutable at runtime.
    custom_order: Vec<String>,
    pub pack: PackInfo,
    pub collections: AssetCollections,
}

impl Default for AssetDB {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetDB {
    /// A freshly bootstrapped registry: every frozen-family slot present with
    /// empty metadata and an empty store, feature-icon slots already carrying
    /// their [`crate::preset_scatter_rule`] (the reference's own bootstrap
    /// `FAMILIES.forEach(fam=>fam.slots.forEach(slot=>{...slotRules(fam,slot)...}))`),
    /// and no custom slots yet.
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        let mut store = HashMap::new();
        let mut frozen_order = Vec::new();
        for family in Family::ALL {
            if family == Family::Custom {
                continue;
            }
            for &id in library_slot_ids(family) {
                let uid = make_uid(family, None, id);
                let rules = fam_scatters(family).then(|| crate::scatter::preset_scatter_rule(id));
                slots.insert(
                    uid.clone(),
                    LibrarySlot {
                        id: id.to_string(),
                        name: slot_title(family, id).to_string(),
                        uid: uid.clone(),
                        family,
                        set: None,
                        set_id: None,
                        meta: SlotMeta::default(),
                        rules,
                    },
                );
                store.insert(uid.clone(), Vec::new());
                frozen_order.push(uid);
            }
        }
        AssetDB {
            slots,
            store,
            frozen_order,
            custom_order: Vec::new(),
            pack: PackInfo::default(),
            collections: AssetCollections::new(),
        }
    }

    /// The slot registered under `uid`, if any — the reference's
    /// `AssetDB.get(uid)` (minus the `{fam,slot}` wrapper: [`Family`] is on
    /// [`LibrarySlot::family`] directly).
    pub fn get(&self, uid: &str) -> Option<&LibrarySlot> {
        self.slots.get(uid)
    }

    /// The items stored under `uid` — `&[]` if `uid` is unknown or empty,
    /// matching the reference's `store[uid]||[]`.
    pub fn items(&self, uid: &str) -> &[LibraryItem] {
        self.store.get(uid).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Mutable access to a slot's metadata (author/copyright/license/source/
    /// notes/version/tags) — there is no dedicated reference setter beyond
    /// direct field assignment on `slot.meta` (e.g. the Inspector UI's own
    /// bindings), so this is the equivalent surface for a UI-less port.
    pub fn slot_meta_mut(&mut self, uid: &str) -> Option<&mut SlotMeta> {
        self.slots.get_mut(uid).map(|s| &mut s.meta)
    }

    /// Every slot in `family`, in registry order (frozen vocabulary order, or
    /// custom add-order).
    pub fn slots_in_family(&self, family: Family) -> Vec<&LibrarySlot> {
        if family == Family::Custom {
            self.custom_order
                .iter()
                .filter_map(|u| self.slots.get(u.as_str()))
                .collect()
        } else {
            library_slot_ids(family)
                .iter()
                .filter_map(|id| self.slots.get(&make_uid(family, None, id)))
                .collect()
        }
    }

    /// Every uid in the order the reference's `for(const uid in store)`
    /// visits them: every frozen slot (bootstrap order), then every custom
    /// slot (add order). This is the order `AssetValidator`'s duplicate-image
    /// scan groups by.
    pub fn uids_in_order(&self) -> Vec<String> {
        self.frozen_order
            .iter()
            .chain(self.custom_order.iter())
            .cloned()
            .collect()
    }

    /// Number of slots in `family` carrying at least one item — the
    /// reference's `AssetDB.filledCount(famKey)`.
    pub fn filled_count(&self, family: Family) -> usize {
        self.slots_in_family(family)
            .iter()
            .filter(|s| !self.items(&s.uid).is_empty())
            .count()
    }

    /// Total items across every slot — the reference's `AssetDB.totalItems()`.
    pub fn total_items(&self) -> usize {
        self.store.values().map(Vec::len).sum()
    }

    /// Append an item to `uid`'s store. `false` if `uid` is not registered
    /// (there is no such slot to hold it).
    pub fn add_item(&mut self, uid: &str, item: LibraryItem) -> bool {
        match self.store.get_mut(uid) {
            Some(v) => {
                v.push(item);
                true
            }
            None => false,
        }
    }

    /// Remove and return the item at `index`, if both `uid` and `index` are
    /// valid.
    pub fn remove_item(&mut self, uid: &str, index: usize) -> Option<LibraryItem> {
        self.store.get_mut(uid).and_then(|v| {
            if index < v.len() {
                Some(v.remove(index))
            } else {
                None
            }
        })
    }

    /// Get-or-lazily-attach a slot's scatter rule — the reference's
    /// `slotRules(fam,slot)`. `None` for a family that cannot scatter
    /// ([`fam_scatters`]). On first call for a scatterable slot with no rule
    /// yet, attaches [`crate::preset_scatter_rule`] (frozen) or a
    /// disabled default (custom) — a real, mutating side effect the
    /// reference's own export path relies on (see [`AssetDB::to_library_json`]).
    pub fn slot_rules(&mut self, uid: &str) -> Option<&ScatterRule> {
        let slot = self.slots.get(uid)?;
        if !fam_scatters(slot.family) {
            return None;
        }
        if slot.rules.is_none() {
            let rule = if slot.family == Family::Custom {
                ScatterRule {
                    enabled: false,
                    ..ScatterRule::default()
                }
            } else {
                crate::scatter::preset_scatter_rule(&slot.id)
            };
            self.slots.get_mut(uid).unwrap().rules = Some(rule);
        }
        self.slots.get(uid).unwrap().rules.as_ref()
    }

    /// Add (or, on a name/set collision, return the existing) custom slot —
    /// the reference's `AssetDB.addCustomSlot(name,setName)`.
    ///
    /// **Idempotent by construction, not merely by convention**: `name` and
    /// `set_name` are independently slugged ([`slug_id`]) into the slot's
    /// `id`/`uid`, so two calls whose *display* text differs but whose slugs
    /// collide (`"Wind Mill!!"` then `"wind   mill"`) return the **same**
    /// slot — the first one created — rather than erroring or silently
    /// overwriting it. This is real untrusted-input hardening carried over
    /// from the reference (`const existing=fam.slots.find(s=>s.uid===uid); if(existing) return existing;`);
    /// see `tests/hardening_asset_db.rs`.
    pub fn add_custom_slot(&mut self, name: &str, set_name: Option<&str>) -> &LibrarySlot {
        let set_name = {
            let s = set_name.unwrap_or("Default").trim();
            if s.is_empty() {
                "Default".to_string()
            } else {
                s.to_string()
            }
        };
        let id = slug_id(name);
        let set_id = slug_id(&set_name);
        let uid = make_uid(Family::Custom, Some(&set_id), &id);
        if self.slots.contains_key(&uid) {
            return &self.slots[&uid];
        }
        let slot = LibrarySlot {
            id,
            name: name.trim().to_string(),
            uid: uid.clone(),
            family: Family::Custom,
            set: Some(set_name),
            set_id: Some(set_id),
            meta: SlotMeta::default(),
            rules: None,
        };
        self.slots.insert(uid.clone(), slot);
        self.custom_order.push(uid.clone());
        self.store.insert(uid.clone(), Vec::new());
        &self.slots[&uid]
    }

    /// Rename a custom slot's display name (and therefore its `id`/`uid`) —
    /// the reference's `AssetDB.renameCustomSlot(uid,newName)`. Returns the
    /// slot's (possibly new) uid.
    ///
    /// **Collision-safe.** If the new name slugs to a uid another slot
    /// already occupies, the rename is refused and the **old** uid is
    /// returned unchanged — it does not clobber the existing target, and it
    /// does not error. A no-op (returns `uid` unchanged) if `uid` is unknown
    /// or names a frozen (non-custom) slot, since frozen slots cannot be
    /// renamed at all. See `tests/hardening_asset_db.rs`.
    pub fn rename_custom_slot(&mut self, uid: &str, new_name: &str) -> String {
        let Some(slot) = self.slots.get(uid) else {
            return uid.to_string();
        };
        if slot.family != Family::Custom {
            return uid.to_string();
        }
        let set_id = slot.set_id.clone().expect("custom slot always carries a set_id");
        let nid = slug_id(new_name);
        let nuid = make_uid(Family::Custom, Some(&set_id), &nid);
        if nuid == uid {
            self.slots.get_mut(uid).unwrap().name = new_name.trim().to_string();
            return uid.to_string();
        }
        if self.slots.contains_key(&nuid) {
            return uid.to_string(); // collision — keep the old id
        }
        let mut slot = self.slots.remove(uid).unwrap();
        slot.id = nid;
        slot.name = new_name.trim().to_string();
        slot.uid = nuid.clone();
        self.slots.insert(nuid.clone(), slot);
        for u in self.custom_order.iter_mut() {
            if u == uid {
                *u = nuid.clone();
            }
        }
        if let Some(items) = self.store.remove(uid) {
            self.store.insert(nuid.clone(), items);
        }
        self.collections.rename_uid(uid, &nuid);
        nuid
    }

    /// Remove a custom slot, its store and its collection memberships —
    /// the reference's `AssetDB.removeCustomSlot(uid)`. `false` (no-op) if
    /// `uid` is unknown or names a frozen slot; frozen slots cannot be
    /// removed.
    pub fn remove_custom_slot(&mut self, uid: &str) -> bool {
        let Some(slot) = self.slots.get(uid) else {
            return false;
        };
        if slot.family != Family::Custom {
            return false;
        }
        self.slots.remove(uid);
        self.custom_order.retain(|u| u != uid);
        self.store.remove(uid);
        self.collections.drop_uid(uid);
        true
    }

    /// Reset to a freshly bootstrapped state: every custom slot is dropped,
    /// every frozen slot's store and metadata are cleared (its scatter rule
    /// is left as-is), and every collection is dropped — the reference's
    /// `AssetDB.clear()`. Pack info (name/author/license) is left untouched,
    /// matching the reference, which never touches those DOM fields here.
    pub fn clear(&mut self) {
        for uid in self.custom_order.drain(..) {
            self.slots.remove(&uid);
            self.store.remove(&uid);
        }
        for uid in &self.frozen_order {
            if let Some(items) = self.store.get_mut(uid) {
                items.clear();
            }
            if let Some(slot) = self.slots.get_mut(uid) {
                slot.meta = SlotMeta::default();
            }
        }
        self.collections.clear();
    }

    /// Build the `assetlib/library.json` record for this Library — the
    /// reference's `window._alExportEntries`, minus the actual
    /// `assetlib/img/N.png` bytes (milestone 6). `None` when the library is
    /// empty, matching the reference's own `if(AssetDB.totalItems()===0)
    /// return null`.
    ///
    /// Each item is assigned a monotonically increasing `img` index in
    /// exactly the reference's traversal order (frozen families first, in
    /// `FAMILIES` order, then custom slots in add-order; items within a slot
    /// in store order) — the index a future milestone 6 export would pair
    /// with `assetlib/img/<idx>.png`.
    ///
    /// **Mutates `self`**: reading a scatterable slot's rule for the first
    /// time lazily attaches its preset (see [`AssetDB::slot_rules`]), and the
    /// reference does this unconditionally for *every* scatterable slot
    /// during export — including ones the resulting record excludes. This
    /// port reproduces that real, if surprising, side effect rather than
    /// hiding it behind `&self`.
    pub fn to_library_json(&mut self) -> Option<LibraryFile> {
        if self.total_items() == 0 {
            return None;
        }
        let mut slots = Vec::new();
        let mut img_idx = 0usize;
        for uid in self.uids_in_order() {
            let mut items = Vec::new();
            for it in self.items(&uid) {
                items.push(ItemRecord {
                    img: img_idx,
                    name: it.name.clone(),
                    t: it.transform.clone(),
                });
                img_idx += 1;
            }
            let slot = self.get(&uid).expect("uid from uids_in_order always resolves").clone();
            let rules = if fam_scatters(slot.family) {
                self.slot_rules(&uid).cloned()
            } else {
                None
            };
            let has_tags = !slot.meta.tags.is_empty();
            let is_custom = slot.family == Family::Custom;
            if items.is_empty() && !has_tags && !is_custom {
                continue;
            }
            slots.push(SlotRecord {
                fam: slot.family.key().to_string(),
                id: slot.id.clone(),
                name: slot.name.clone(),
                meta: slot.meta.clone(),
                items,
                set: slot.set.clone(),
                rules,
            });
        }
        Some(LibraryFile {
            version: 1,
            kind: "cartalith-assetlib".to_string(),
            pack: Some(self.pack.clone()),
            collections: self.collections.as_map().clone(),
            slots,
        })
    }

    /// Restore a parsed `library.json`'s pack info, collections, and
    /// per-slot metadata/scatter rules onto this registry — the
    /// non-item-restoring core of the reference's `window._alImportProject`.
    ///
    /// Calls [`AssetDB::clear`] first, exactly as the reference does. Pack
    /// fields follow the reference's own (asymmetric) fallback rule: `name`
    /// is only overwritten when the file's own name is non-empty (an absent
    /// name leaves whatever was already there, e.g. from before an import),
    /// while `author`/`license` are always overwritten, falling back to `""`
    /// / `"CC0"` — already baked into [`LibraryFile::pack`] by
    /// [`parse_library_json`]. Custom slots are (re)created via
    /// [`AssetDB::add_custom_slot`], so id-slugging and uid-collision
    /// handling apply identically to a restored project as to one built by
    /// hand.
    ///
    /// **Does not restore items.** [`SlotRecord::items`] carries `img`
    /// indices, names and transforms — everything a real reader has *except*
    /// pixels, which need `assetlib/img/<idx>.png` decoded (milestone 6).
    pub fn apply_library_file(&mut self, file: &LibraryFile) {
        self.clear();
        if let Some(pack) = &file.pack {
            if !pack.name.is_empty() {
                self.pack.name = pack.name.clone();
            }
            self.pack.author = pack.author.clone();
            self.pack.license = pack.license.clone();
        }
        self.collections = AssetCollections::from_map(file.collections.clone());
        for rec in &file.slots {
            let uid = match Family::from_key(&rec.fam) {
                Some(Family::Custom) => self.add_custom_slot(&rec.name, rec.set.as_deref()).uid.clone(),
                Some(family) => make_uid(family, None, &rec.id),
                None => continue, // parse_library_json already drops these; defensive only
            };
            let Some(slot) = self.slots.get_mut(&uid) else {
                continue;
            };
            slot.meta = rec.meta.clone();
            if fam_scatters(slot.family)
                && let Some(r) = &rec.rules
            {
                slot.rules = Some(r.clone());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AssetCollections
// ---------------------------------------------------------------------------

/// Named groupings of uids — the reference's `AssetCollections`. Order
/// matters: `names()` follows creation order (JavaScript object-key order),
/// which is what makes [`run`]'s "missing asset" warnings
/// reproducible in the reference's own order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssetCollections {
    map: OrderedMap<Vec<String>>,
}

impl AssetCollections {
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap an already-built map — used when restoring from a parsed
    /// `library.json`, whose `collections` object the reference assigns
    /// wholesale and **without validating against the current registry**
    /// (`AssetCollections.map=lib.collections||{}`). A stale uid referencing
    /// a slot that no longer exists survives this call unchanged; that is
    /// exactly the (only real) path to [`run`]'s "references
    /// a missing asset" warning — `removeCustomSlot` cleans up any live
    /// membership itself, so that warning cannot fire through normal editing.
    pub fn from_map(map: OrderedMap<Vec<String>>) -> Self {
        AssetCollections { map }
    }

    pub fn as_map(&self) -> &OrderedMap<Vec<String>> {
        &self.map
    }

    pub fn names(&self) -> Vec<&str> {
        self.map.keys().collect()
    }

    /// Add `uids` to `name`, creating the collection if needed. A blank
    /// (post-trim) name is a no-op. Duplicate uids within one call, or
    /// across repeated calls, are not added twice.
    pub fn add(&mut self, name: &str, uids: &[String]) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        if self.map.get(name).is_none() {
            self.map.insert(name, Vec::new());
        }
        let arr = self.map.get_mut(name).unwrap();
        for u in uids {
            if !arr.contains(u) {
                arr.push(u.clone());
            }
        }
    }

    /// Remove one uid from `name`. The collection itself is dropped once it
    /// becomes empty, matching the reference.
    pub fn remove(&mut self, name: &str, uid: &str) {
        let Some(arr) = self.map.get_mut(name) else {
            return;
        };
        arr.retain(|u| u != uid);
        if arr.is_empty() {
            self.map.remove(name);
        }
    }

    /// Drop a whole collection by name.
    pub fn drop_collection(&mut self, name: &str) {
        self.map.remove(name);
    }

    /// Remove `uid` from every collection — the reference's `dropUid`, called
    /// by [`AssetDB::remove_custom_slot`].
    pub fn drop_uid(&mut self, uid: &str) {
        let names: Vec<String> = self.names().into_iter().map(String::from).collect();
        for n in names {
            self.remove(&n, uid);
        }
    }

    /// Replace every occurrence of `old` with `new`, across every
    /// collection — the reference's `renameUid`, called by
    /// [`AssetDB::rename_custom_slot`].
    pub fn rename_uid(&mut self, old: &str, new: &str) {
        for (_, arr) in self.map.iter_mut() {
            if let Some(pos) = arr.iter().position(|u| u == old) {
                arr[pos] = new.to_string();
            }
        }
    }

    /// Every collection name that currently lists `uid`.
    pub fn membership(&self, uid: &str) -> Vec<&str> {
        self.map
            .iter()
            .filter(|(_, arr)| arr.iter().any(|u| u == uid))
            .map(|(n, _)| n)
            .collect()
    }

    pub fn clear(&mut self) {
        self.map = OrderedMap::new();
    }
}

// ---------------------------------------------------------------------------
// AssetValidator
// ---------------------------------------------------------------------------

/// One member of a duplicate-image group — the reference's
/// `{uid,idx,name}` entry inside `AssetValidator.duplicateGroups()`.
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateEntry {
    pub uid: String,
    pub idx: usize,
    pub item_name: String,
}

/// Group items whose `hash` collides — the reference's
/// `AssetValidator.duplicateGroups()`. Groups (and entries within a group)
/// appear in [`AssetDB::uids_in_order`]/store order, i.e. the order the
/// reference's `for(const uid in store)` would encounter them, which is what
/// makes [`run`]'s "Identical images" message text
/// reproducible.
pub fn duplicate_groups(db: &AssetDB) -> Vec<Vec<DuplicateEntry>> {
    let mut by_hash: Vec<(String, Vec<DuplicateEntry>)> = Vec::new();
    for uid in db.uids_in_order() {
        for (idx, item) in db.items(&uid).iter().enumerate() {
            let entry = DuplicateEntry {
                uid: uid.clone(),
                idx,
                item_name: item.name.clone(),
            };
            match by_hash.iter_mut().find(|(h, _)| *h == item.hash) {
                Some((_, v)) => v.push(entry),
                None => by_hash.push((item.hash.clone(), vec![entry])),
            }
        }
    }
    by_hash.into_iter().map(|(_, v)| v).filter(|v| v.len() > 1).collect()
}

/// Whether `uid` participates in any duplicate-image group — the reference's
/// `AssetValidator.slotHasDupe(uid)`.
pub fn slot_has_dupe(db: &AssetDB, uid: &str) -> bool {
    duplicate_groups(db).iter().any(|g| g.iter().any(|e| e.uid == uid))
}

fn is_valid_custom_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The Library's own pre-export sanity checks — the reference's
/// `AssetValidator.run()`. Warnings, not errors: a library with warnings
/// still exports; a UI reports the count next to the summary.
///
/// Two of the six checks below are, by construction, unreachable through
/// this module's own public API in **both** languages (found by reading, not
/// assumed): "Duplicate identifier" can never fire because [`AssetDB`]'s
/// slot store is keyed by the very uid the check recomputes (a `HashMap` in
/// Rust cannot hold two values under one key; the reference's own
/// `SLOT_REG` object has the equivalent guarantee), and "Collection
/// references a missing asset" can never fire via
/// [`AssetDB::remove_custom_slot`], because that method already calls
/// [`AssetCollections::drop_uid`] before this could ever run. The only real
/// path to the latter is [`AssetCollections::from_map`] during
/// [`AssetDB::apply_library_file`], which — matching the reference's
/// `AssetCollections.map=lib.collections||{}` — assigns a parsed project's
/// collections **without** validating them against the current registry.
/// Both checks are ported anyway, faithfully, as real (if currently inert)
/// defence-in-depth the reference itself carries.
pub fn run(db: &AssetDB) -> Vec<String> {
    let mut warn = Vec::new();
    if db.pack.name.trim().is_empty() {
        warn.push("Pack name is empty.".to_string());
    }
    if db.total_items() == 0 {
        warn.push("Library is empty — nothing to export.".to_string());
    }

    let mut seen: HashSet<String> = HashSet::new();
    for family in Family::ALL {
        for slot in db.slots_in_family(family) {
            let key = match &slot.set_id {
                Some(set_id) => format!("{}:{set_id}/{}", family.key(), slot.id),
                None => format!("{}:{}", family.key(), slot.id),
            };
            if seen.contains(&key) {
                warn.push(format!("Duplicate identifier: {key}"));
            }
            seen.insert(key);
            if family == Family::Custom && !is_valid_custom_id(&slot.id) {
                warn.push(format!("Invalid filename id: {}", slot.id));
            }
            if family == Family::Custom && db.items(&slot.uid).is_empty() {
                warn.push(format!("Empty custom slot (no variants): {}", slot.name));
            }
        }
    }

    for group in duplicate_groups(db) {
        let names: Vec<String> = group
            .iter()
            .map(|e| {
                let slot_name = db.get(&e.uid).map(|s| s.name.as_str()).unwrap_or(e.uid.as_str());
                format!("{slot_name}#{}", e.idx + 1)
            })
            .collect();
        warn.push(format!("Identical images: {}", names.join(" = ")));
    }

    for name in db.collections.names() {
        for uid in db.collections.as_map().get(name).into_iter().flatten() {
            if db.get(uid).is_none() {
                warn.push(format!("Collection \"{name}\" references a missing asset."));
            }
        }
    }

    let filled: Vec<&str> = db
        .slots_in_family(Family::Textures)
        .into_iter()
        .filter(|s| !db.items(&s.uid).is_empty())
        .map(|s| s.id.as_str())
        .collect();
    if !filled.is_empty() && !filled.contains(&"grass") {
        warn.push(
            "Splat channels present but no \"grass\" — the engine splat blends grass most."
                .to_string(),
        );
    }

    warn
}

// ---------------------------------------------------------------------------
// The `assetlib/library.json` record shape
// ---------------------------------------------------------------------------

/// One item entry inside a [`SlotRecord`] — the reference's
/// `{img,name,t}` (`rec.items.push({img:idx,name:it.name,t:it.t})`). `img` is
/// the index of the paired `assetlib/img/<img>.png` entry; this crate never
/// reads or writes that file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemRecord {
    pub img: usize,
    pub name: String,
    pub t: ItemTransform,
}

fn parse_item_record(v: &Value) -> Option<ItemRecord> {
    let obj = v.as_object()?;
    let img = obj.get("img")?.as_u64()? as usize;
    let name = obj.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    let t = normalize_transform(obj.get("t"));
    Some(ItemRecord { img, name, t })
}

/// One slot's persisted record — the reference's `rec` object built inside
/// `_alExportEntries` (`{fam,id,name,meta,items,set?,rules?}`, field order
/// matching a real export byte for byte). `rules` is `Some` exactly when
/// [`fam_scatters`] is true for `fam`; `set` is `Some` only for
/// [`Family::Custom`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SlotRecord {
    pub fam: String,
    pub id: String,
    pub name: String,
    pub meta: SlotMeta,
    pub items: Vec<ItemRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<ScatterRule>,
}

/// Parse one `slots[]` entry, applying [`normalize_scatter_rule`] eagerly
/// (this milestone's "record shape ... with its normalizeScatterRule-on-load
/// behaviour"). Returns `None` for a record this port (and the reference)
/// cannot resolve to a real slot at all: an unknown `fam`, or — for a
/// non-custom family — an `id` outside that family's frozen vocabulary
/// ([`library_slot_ids`]). This mirrors the reference's own
/// `if(!uid) continue;`, which drops such a record's meta/rules/items
/// entirely rather than keeping any part of it.
fn parse_slot_record(v: &Value) -> Option<SlotRecord> {
    let obj = v.as_object()?;
    let fam = obj.get("fam").and_then(Value::as_str)?.to_string();
    let family = Family::from_key(&fam)?;
    let id = obj.get("id").and_then(Value::as_str)?.to_string();
    if family != Family::Custom && !library_slot_ids(family).contains(&id.as_str()) {
        return None;
    }
    let name_raw = obj.get("name").and_then(Value::as_str).filter(|s| !s.is_empty());
    let name = name_raw.unwrap_or(&id).to_string();
    let set = obj
        .get("set")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty());
    let meta = normalize_meta(obj.get("meta"));
    let rules = fam_scatters(family).then(|| {
        let key = if family == Family::Custom {
            scatter_rule_key(&id, Some(set.as_deref().unwrap_or("Default")))
        } else {
            id.clone()
        };
        let raw_rules = obj.get("rules").cloned().unwrap_or(Value::Null);
        normalize_scatter_rule(&raw_rules, &key)
    });
    let items = obj
        .get("items")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(parse_item_record).collect())
        .unwrap_or_default();
    Some(SlotRecord {
        fam,
        id,
        name,
        meta,
        items,
        set,
        rules,
    })
}

fn parse_pack_info(v: Option<&Value>) -> Option<PackInfo> {
    let v = v?;
    if v.is_null() {
        return None;
    }
    let obj = v.as_object();
    let get = |k: &str| obj.and_then(|o| o.get(k)).and_then(Value::as_str).unwrap_or("");
    let license = get("license");
    Some(PackInfo {
        name: get("name").to_string(),
        author: get("author").to_string(),
        license: if license.is_empty() {
            "CC0".to_string()
        } else {
            license.to_string()
        },
    })
}

/// A whole `assetlib/library.json` document — the reference's `lib` object
/// (`{version,kind,pack,collections,slots}`), field order matching a real
/// export exactly.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryFile {
    pub version: u32,
    pub kind: String,
    /// `None` only when the parsed document had no `pack` section at all (or
    /// an explicit `null`) — see [`AssetDB::apply_library_file`] for why that
    /// distinction (rather than just an empty [`PackInfo`]) matters.
    /// [`AssetDB::to_library_json`] always produces `Some`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackInfo>,
    pub collections: OrderedMap<Vec<String>>,
    pub slots: Vec<SlotRecord>,
}

/// What went wrong reading a `library.json`.
#[derive(Debug)]
pub enum LibraryError {
    /// Not valid JSON, or not shaped like a document at all (e.g. a bare
    /// array or string at the top level).
    Json(serde_json::Error),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::Json(e) => write!(f, "library.json is not valid: {e}"),
        }
    }
}

impl std::error::Error for LibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LibraryError::Json(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for LibraryError {
    fn from(e: serde_json::Error) -> Self {
        LibraryError::Json(e)
    }
}

/// Untyped top-level shape used only to route `collections` through
/// [`OrderedMap`] (preserving document order via serde's streaming
/// `MapAccess`, independent of whether `serde_json`'s `preserve_order`
/// feature is on) while every other section stays a [`Value`] for hand
/// coercion — the same "typed only where order matters, `Value` elsewhere"
/// split [`crate::manifest`] uses for `OrderedMap<Option<Paths>>`.
#[derive(Debug, Deserialize)]
struct RawLibraryFile {
    #[serde(default)]
    version: Option<u64>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    pack: Option<Value>,
    #[serde(default)]
    collections: OrderedMap<Value>,
    #[serde(default)]
    slots: Vec<Value>,
}

/// Parse an `assetlib/library.json` document — the reference's
/// `JSON.parse(new TextDecoder().decode(zip['assetlib/library.json']))` plus
/// every hardening step that follows it in `_alImportProject`
/// (`normalizeScatterRule`-on-load per scatterable slot, defaulted meta,
/// defaulted item transforms). Untrusted input throughout, per this crate's
/// established discipline: malformed JSON is a real [`LibraryError`]; a
/// malformed *field* is dropped or defaulted rather than propagated, never
/// causing the whole document to fail.
pub fn parse_library_json(bytes: &[u8]) -> Result<LibraryFile, LibraryError> {
    let raw: RawLibraryFile = serde_json::from_slice(bytes)?;
    let pack = parse_pack_info(raw.pack.as_ref());
    let mut collections = OrderedMap::new();
    for (name, val) in raw.collections.iter() {
        let uids: Vec<String> = val
            .as_array()
            .map(|a| a.iter().filter_map(|u| u.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        collections.insert(name, uids);
    }
    let slots = raw.slots.iter().filter_map(parse_slot_record).collect();
    Ok(LibraryFile {
        version: raw.version.unwrap_or(1) as u32,
        kind: raw.kind.unwrap_or_else(|| "cartalith-assetlib".to_string()),
        pack,
        collections,
        slots,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
//
// These are ordinary unit tests, not golden-parity ones: they pin this port's
// own design decisions (the `library.json` reader's leniency, the pack-name
// fallback asymmetry read directly out of `_alImportProject`, and the
// `slot_title` table's completeness) rather than a captured reference run.
// The golden-parity coverage lives in `tests/golden_parity_library.rs`.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn slot_title_covers_every_frozen_slot_without_panicking() {
        for family in Family::ALL {
            if family == Family::Custom {
                continue;
            }
            for &id in library_slot_ids(family) {
                let title = slot_title(family, id);
                assert!(!title.is_empty(), "{}:{id} has a blank title", family.key());
            }
        }
    }

    #[test]
    fn library_poi_slots_is_the_ten_slot_superset_of_the_eight_slot_pack_vocabulary() {
        assert_eq!(LIBRARY_POI_SLOTS.len(), 10);
        assert!(LIBRARY_POI_SLOTS.contains(&"lake"));
        assert!(LIBRARY_POI_SLOTS.contains(&"bridge"));
        for pack_slot in crate::PACK_POI_SLOTS {
            assert!(LIBRARY_POI_SLOTS.contains(&pack_slot));
        }
    }

    #[test]
    fn new_db_bootstraps_every_frozen_slot_with_no_custom_slots() {
        let db = AssetDB::new();
        for family in Family::ALL {
            if family == Family::Custom {
                assert!(db.slots_in_family(family).is_empty());
                continue;
            }
            assert_eq!(db.slots_in_family(family).len(), library_slot_ids(family).len());
        }
        assert_eq!(db.total_items(), 0);
    }

    #[test]
    fn frozen_icon_slots_carry_their_preset_rule_at_bootstrap() {
        let mut db = AssetDB::new();
        let r = db.slot_rules("icons:mountain").cloned().unwrap();
        assert_eq!(r, crate::scatter::preset_scatter_rule("mountain"));
    }

    #[test]
    fn non_scatterable_families_never_carry_a_rule() {
        let mut db = AssetDB::new();
        assert!(db.slot_rules("textures:grass").is_none());
        assert!(db.slot_rules("structures:hamlet").is_none() || db.get("settlement:hamlet").is_none());
        assert!(db.slot_rules("settlement:hamlet").is_none());
        assert!(db.slot_rules("poi:lake").is_none());
    }

    #[test]
    fn custom_slots_lazily_attach_a_disabled_rule_on_first_read() {
        let mut db = AssetDB::new();
        let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
        assert!(db.get(&uid).unwrap().rules.is_none(), "not attached until first read");
        let r = db.slot_rules(&uid).cloned().unwrap();
        assert!(!r.enabled);
        assert_eq!(r, ScatterRule { enabled: false, ..ScatterRule::default() });
        assert!(db.get(&uid).unwrap().rules.is_some(), "now attached as a side effect");
    }

    // -- parse_library_json / apply_library_file --------------------------

    #[test]
    fn export_then_reparse_then_apply_round_trips_meta_rules_pack_and_collections() {
        let mut db = AssetDB::new();
        db.pack = PackInfo {
            name: "My Pack".to_string(),
            author: "Author A".to_string(),
            license: "MIT".to_string(),
        };
        let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
        db.slot_meta_mut(&uid).unwrap().tags = vec!["coastal".to_string()];
        db.collections.add("Coastal", std::slice::from_ref(&uid));
        db.add_item(&uid, LibraryItem::new("l1.png", "hashL"));
        db.add_item("icons:mountain", LibraryItem::new("m1.png", "hashA"));

        let exported = db.to_library_json().unwrap();
        let text = serde_json::to_string(&exported).unwrap();

        let parsed = parse_library_json(text.as_bytes()).unwrap();
        assert_eq!(parsed.pack.as_ref().unwrap().name, "My Pack");
        assert_eq!(parsed.pack.as_ref().unwrap().license, "MIT");

        let mut fresh = AssetDB::new();
        fresh.apply_library_file(&parsed);

        assert_eq!(fresh.pack.name, "My Pack");
        assert_eq!(fresh.pack.author, "Author A");
        assert_eq!(fresh.pack.license, "MIT");
        assert_eq!(fresh.collections.as_map().get("Coastal").unwrap(), std::slice::from_ref(&uid));
        assert_eq!(fresh.get(&uid).unwrap().meta.tags, vec!["coastal"]);
        assert!(fresh.get(&uid).unwrap().rules.is_some());
        assert_eq!(
            fresh.get("icons:mountain").unwrap().rules,
            db.get("icons:mountain").unwrap().rules
        );
        // Items are NOT restored at this milestone (no image bytes here).
        assert!(fresh.items(&uid).is_empty());
        assert!(fresh.items("icons:mountain").is_empty());
    }

    #[test]
    fn apply_library_file_preserves_an_existing_pack_name_when_the_file_omits_it() {
        // The reference's own asymmetry: `if(E('alPackName')&&lib.pack.name)
        // E('alPackName').value=lib.pack.name;` -- name is only overwritten
        // when the incoming value is non-empty; author/license always are.
        let mut db = AssetDB::new();
        db.pack.name = "Original Name".to_string();
        db.pack.author = "Original Author".to_string();
        db.pack.license = "Original License".to_string();

        let file = LibraryFile {
            version: 1,
            kind: "cartalith-assetlib".to_string(),
            pack: Some(PackInfo {
                name: String::new(),
                author: String::new(),
                license: "CC0".to_string(),
            }),
            collections: OrderedMap::new(),
            slots: vec![],
        };
        db.apply_library_file(&file);
        assert_eq!(db.pack.name, "Original Name", "blank incoming name preserves the old one");
        assert_eq!(db.pack.author, "", "author always overwrites, falling back to empty");
        assert_eq!(db.pack.license, "CC0");
    }

    #[test]
    fn apply_library_file_leaves_pack_fields_untouched_when_the_file_has_no_pack_section() {
        let mut db = AssetDB::new();
        db.pack.name = "Untouched".to_string();
        let file = LibraryFile {
            version: 1,
            kind: "cartalith-assetlib".to_string(),
            pack: None,
            collections: OrderedMap::new(),
            slots: vec![],
        };
        db.apply_library_file(&file);
        assert_eq!(db.pack.name, "Untouched");
    }

    #[test]
    fn parse_library_json_drops_a_record_for_an_unknown_family_or_unresolvable_frozen_id() {
        let text = json!({
            "version": 1, "kind": "cartalith-assetlib",
            "collections": {},
            "slots": [
                {"fam": "nonsense", "id": "whatever", "meta": {}, "items": []},
                {"fam": "icons", "id": "not_a_real_icon_slot", "meta": {}, "items": []},
                {"fam": "icons", "id": "mountain", "meta": {}, "items": []},
            ]
        })
        .to_string();
        let parsed = parse_library_json(text.as_bytes()).unwrap();
        assert_eq!(parsed.slots.len(), 1);
        assert_eq!(parsed.slots[0].id, "mountain");
    }

    #[test]
    fn parse_library_json_normalizes_rules_on_load_for_scatterable_families_only() {
        let text = json!({
            "version": 1, "kind": "cartalith-assetlib",
            "collections": {},
            "slots": [
                {"fam": "icons", "id": "cactus", "meta": {}, "items": [],
                 "rules": {"density": "not a number"}},
                {"fam": "settlement", "id": "hamlet", "meta": {}, "items": [],
                 "rules": {"density": 99}},
            ]
        })
        .to_string();
        let parsed = parse_library_json(text.as_bytes()).unwrap();
        let cactus = parsed.slots.iter().find(|s| s.id == "cactus").unwrap();
        // A rejected density falls back to the literal 1 (v1.27's own documented
        // asymmetry, ported in milestone 3's normalize_scatter_rule).
        assert_eq!(cactus.rules.as_ref().unwrap().density, 1.0);

        let hamlet = parsed.slots.iter().find(|s| s.id == "hamlet").unwrap();
        assert!(hamlet.rules.is_none(), "settlements cannot scatter; no rule at all, even though the file carried one");
    }

    #[test]
    fn parse_library_json_collections_are_lenient_and_order_preserving() {
        // A hand-written literal, deliberately NOT built via the `json!` macro:
        // `serde_json::Value`'s own `Object` map is a `BTreeMap` in this
        // workspace (no `preserve_order` feature -- milestone 1's own finding),
        // so `json!({...}).to_string()` would already have sorted the keys
        // before parsing ever saw them. Real file bytes carry the author's
        // actual order; this is what a real read looks like.
        let text = r#"{"version":1,"kind":"cartalith-assetlib",
            "collections":{"Zebra":["a","b"],"Apple":"not an array","Mango":["c"]},
            "slots":[]}"#;
        let parsed = parse_library_json(text.as_bytes()).unwrap();
        assert_eq!(parsed.collections.keys().collect::<Vec<_>>(), ["Zebra", "Apple", "Mango"]);
        assert_eq!(parsed.collections.get("Zebra").unwrap(), &["a", "b"]);
        assert!(parsed.collections.get("Apple").unwrap().is_empty(), "malformed value -> empty, not a parse error");
    }

    #[test]
    fn parse_library_json_meta_and_transform_are_lenient_on_wrong_types() {
        let text = json!({
            "version": 1, "kind": "cartalith-assetlib",
            "collections": {},
            "slots": [{
                "fam": "icons", "id": "shrub",
                "meta": {"author": 42, "tags": ["ok", 5, "also-ok", null]},
                "items": [{"img": 0, "name": "x.png", "t": {"scale": "big", "panX": 2.5}}]
            }]
        })
        .to_string();
        let parsed = parse_library_json(text.as_bytes()).unwrap();
        let shrub = &parsed.slots[0];
        assert_eq!(shrub.meta.author, "", "wrong JSON type -> default, not propagated garbage");
        assert_eq!(shrub.meta.tags, vec!["ok", "also-ok"], "non-string tag entries are dropped");
        assert_eq!(shrub.items[0].t.scale, 1.0, "non-numeric scale falls back to the default");
        assert_eq!(shrub.items[0].t.pan_x, 2.5, "a well-typed sibling field is unaffected");
    }

    #[test]
    fn parse_library_json_rejects_malformed_json_as_an_error() {
        assert!(parse_library_json(b"{not json").is_err());
    }

    #[test]
    fn custom_slot_record_carries_both_the_raw_set_name_and_resolves_through_it() {
        // Milestone 3's finding, load-bearing here too: the manifest key is the
        // author's raw text, the exporter's path uses the slug. This record
        // shape keeps only the raw text (`set`); the slug is re-derivable via
        // `slug_id` wherever a path is needed (milestone 6).
        let mut db = AssetDB::new();
        db.add_custom_slot("Lighthouse", Some("Naval"));
        let uid = db.add_custom_slot("Anchor", Some("Naval")).uid.clone();
        db.add_item(&uid, LibraryItem::new("a1.png", "hashX"));
        let file = db.to_library_json().unwrap();
        let rec = file.slots.iter().find(|s| s.id == "anchor").unwrap();
        assert_eq!(rec.set.as_deref(), Some("Naval"));
        assert_eq!(slug_id(rec.set.as_deref().unwrap()), "naval");
    }
}
