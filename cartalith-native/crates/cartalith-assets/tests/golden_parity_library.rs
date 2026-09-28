//! Golden-parity tests for `cartalith-assets`' Library model against the
//! **real** reference implementation — `mkSlots`/`FAMILIES`/`AssetDB`/
//! `AssetCollections`/`AssetValidator` (`Cartalith Gen1 v2.10.html` lines
//! 26781-26961) and `window._alExportEntries`'s **shape** (lines 27879-27899).
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — the same technique `tests/golden_parity_pack_manifest.rs` and
//! `tests/golden_parity_scatter_rules.rs` use) that loads those blocks
//! straight out of the frozen HTML by line range, stubs only the two things a
//! headless run genuinely cannot have (the three `E('alPackName'|…)` DOM
//! fields, and `encodeItemPng`'s canvas rasterisation — replaced with a fixed
//! marker byte, since no test here inspects PNG bytes), and calls the real
//! `AssetDB`/`AssetCollections`/`AssetValidator`/`_alExportEntries` on the
//! fixtures below. **The expected values here are that run's output
//! verbatim.**
//!
//! Two real findings this run confirmed, both worth flagging because they are
//! easy to get wrong by *not* checking:
//!
//! - `AssetValidator.run()`'s "Identical images" message renders each item's
//!   **slot display name** (`Mountain`, `Hill`), not its id (`mountain`,
//!   `hill`) — confirmed by `"Identical images: Mountain#1 = Hill#1"` in the
//!   captured output. This is why [`cartalith_assets::slot_title`] exists at
//!   all despite `ASSET_LIBRARY_SCOPE.md` filing per-slot titles as UI-only.
//! - "Collection references a missing asset" cannot fire via
//!   `AssetDB.removeCustomSlot` (it cleans up membership first); the captured
//!   run confirms the only real trigger is an *unchecked* collections
//!   assignment, exactly what `_alImportProject`'s
//!   `AssetCollections.map=lib.collections||{}` is.

use cartalith_assets::{
    AssetCollections, AssetDB, ItemRecord, ItemTransform, LibraryFile, LibraryItem, PackInfo,
    SlotMeta, SlotRecord,
};

/// Shorthand for building a fixture [`LibraryItem`] with the identity
/// transform, used throughout this file's fixtures.
fn item(name: &str, hash: &str) -> LibraryItem {
    LibraryItem::new(name, hash)
}

// ---------------------------------------------------------------------------
// AssetValidator.run()
// ---------------------------------------------------------------------------

#[test]
fn validator_empty_library_empty_pack_name() {
    // Protects: run() emits both warnings, blank-pack-name first, for a totally empty library.
    let db = AssetDB::new();
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Pack name is empty.", "Library is empty — nothing to export."]
    );
}

#[test]
fn validator_empty_library_named_pack() {
    // Protects: run() drops the blank-pack-name warning once a pack name is set, but keeps the empty-library one.
    let mut db = AssetDB::new();
    db.pack.name = "My Pack".to_string();
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Library is empty — nothing to export."]
    );
}

#[test]
fn validator_one_item_no_warnings() {
    // Protects: A named pack with one item passes run() with no warnings at all.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("icons:mountain", item("m1.png", "hashA"));
    assert!(cartalith_assets::library::run(&db).is_empty());
}

#[test]
fn validator_identical_images_across_two_slots_names_by_slot_title() {
    // Protects: the "Identical images" message names entries by slot display title ("Mountain", "Hill"), not by slot id — a real reference finding (module docs).
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("icons:mountain", item("m1.png", "hashA"));
    db.add_item("icons:hill", item("h1.png", "hashA"));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Identical images: Mountain#1 = Hill#1"]
    );
}

#[test]
fn validator_identical_images_three_way_group_in_store_order() {
    // Protects: a three-way hash collision is reported as one group, in store order, not sorted.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("icons:mountain", item("m1.png", "hashA"));
    db.add_item("icons:mountain", item("m2.png", "hashA"));
    db.add_item("icons:hill", item("h1.png", "hashA"));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Identical images: Mountain#1 = Mountain#2 = Hill#1"]
    );
}

#[test]
fn validator_grass_hint_fires_when_textures_present_without_grass() {
    // Protects: the splat-blend hint fires once any texture slot is filled but "grass" is not.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("textures:rock", item("r.png", "hashR"));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Splat channels present but no \"grass\" — the engine splat blends grass most."]
    );
}

#[test]
fn validator_grass_hint_silent_once_grass_is_present() {
    // Protects: the splat-blend hint is silent once "grass" itself is filled, even with other textures present.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("textures:rock", item("r.png", "hashR"));
    db.add_item("textures:grass", item("g.png", "hashG"));
    assert!(cartalith_assets::library::run(&db).is_empty());
}

#[test]
fn validator_empty_custom_slot_warns_by_display_name() {
    // Protects: an empty custom slot is reported by its display name ("Lighthouse"), not its id.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_custom_slot("Lighthouse", Some("Naval"));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Library is empty — nothing to export.", "Empty custom slot (no variants): Lighthouse"]
    );
}

#[test]
fn validator_custom_slot_with_a_variant_has_no_empty_slot_warning() {
    // Protects: a custom slot with at least one item never triggers the empty-slot warning.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    db.add_item(&uid, item("l1.png", "hashL"));
    assert!(cartalith_assets::library::run(&db).is_empty());
}

#[test]
fn validator_removing_a_custom_slot_cleans_up_its_own_collection_membership_first() {
    // Protects: remove_custom_slot's own drop_uid call means this path can never leave a stale collection reference for run() to catch — a real finding (module docs).
    // Confirms a real finding: this path can NEVER produce a "references a
    // missing asset" warning, because `remove_custom_slot` already calls
    // `AssetCollections::drop_uid` before the validator could ever see it.
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    db.collections.add("Coastal", std::slice::from_ref(&uid));
    assert!(db.remove_custom_slot(&uid));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Library is empty — nothing to export."]
    );
}

#[test]
fn validator_stale_collection_only_fires_via_an_unchecked_assignment() {
    // Protects: the only real path to "references a missing asset" is AssetCollections::from_map bypassing the live registry, exactly as apply_library_file uses it.
    // The only real trigger: `AssetCollections::from_map` (what
    // `apply_library_file` uses) assigns a parsed project's collections
    // without checking them against the live registry, exactly like the
    // reference's `AssetCollections.map=lib.collections||{}`.
    use cartalith_assets::OrderedMap;
    let mut db = AssetDB::new();
    db.pack.name = "P".to_string();
    db.add_item("icons:mountain", item("m1.png", "hashA"));
    let mut raw = OrderedMap::new();
    raw.insert("Stale", vec!["custom:nope/nope".to_string()]);
    db.collections = AssetCollections::from_map(raw);
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec!["Collection \"Stale\" references a missing asset."]
    );
}

#[test]
fn validator_kitchen_sink_matches_the_references_own_warning_order() {
    // Protects: every check fires together in the reference's own fixed warning order, not alphabetical or insertion order.
    let mut db = AssetDB::new();
    // No pack name set -> "Pack name is empty." stays first.
    db.add_custom_slot("Lighthouse", Some("Naval"));
    let buoy = db.add_custom_slot("Buoy", Some("Naval")).uid.clone();
    db.add_item(&buoy, item("b1.png", "hashDup"));
    db.add_item("icons:hill", item("h1.png", "hashDup"));
    db.add_item("textures:rock", item("r.png", "hashR"));
    assert_eq!(
        cartalith_assets::library::run(&db),
        vec![
            "Pack name is empty.",
            "Empty custom slot (no variants): Lighthouse",
            "Identical images: Hill#1 = Buoy#1",
            "Splat channels present but no \"grass\" — the engine splat blends grass most.",
        ]
    );
}

// ---------------------------------------------------------------------------
// AssetDB::add_custom_slot / rename_custom_slot / remove_custom_slot
// ---------------------------------------------------------------------------

#[test]
fn add_custom_slot_is_idempotent_on_repeated_identical_name_and_set() {
    // Protects: calling add_custom_slot twice with the same name/set returns the same slot, not a duplicate.
    let mut db = AssetDB::new();
    let a1 = db.add_custom_slot("Lighthouse", Some("Naval")).clone();
    let a2 = db.add_custom_slot("Lighthouse", Some("Naval")).clone();
    assert_eq!(a1, a2);
    assert_eq!(a1.uid, "custom:naval/lighthouse");
    assert_eq!(a1.name, "Lighthouse");
    assert_eq!(a1.set.as_deref(), Some("Naval"));
    assert_eq!(a1.set_id.as_deref(), Some("naval"));
}

#[test]
fn add_custom_slot_display_names_slugging_to_the_same_id_collide_onto_the_first() {
    // Protects: two display names that slug to the same id collide onto the first slot created, keeping its original display name.
    let mut db = AssetDB::new();
    let b1 = db.add_custom_slot("Wind Mill!!", Some("Naval")).clone();
    let b2 = db.add_custom_slot("wind   mill", Some("Naval")).clone();
    assert_eq!(b1, b2);
    assert_eq!(b1.name, "Wind Mill!!"); // first writer wins; second is a no-op
    assert_eq!(b1.id, "wind_mill");
}

#[test]
fn add_custom_slot_blank_name_and_set_fall_back_the_same_way_as_the_reference() {
    // Protects: a blank name/set falls back to the reference's own defaults ("icon" id, "Default" set) rather than producing an empty or invalid slot.
    let mut db = AssetDB::new();
    let slot = db.add_custom_slot("   ", Some("   ")).clone();
    assert_eq!(slot.id, "icon");
    assert_eq!(slot.name, "");
    assert_eq!(slot.set.as_deref(), Some("Default"));
    assert_eq!(slot.set_id.as_deref(), Some("default"));
    assert_eq!(slot.uid, "custom:default/icon");
}

#[test]
fn rename_custom_slot_changes_id_and_uid() {
    // Protects: renaming a custom slot updates its id and uid, and the old uid no longer resolves.
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let renamed = db.rename_custom_slot(&uid, "Old Lighthouse");
    assert_eq!(renamed, "custom:naval/old_lighthouse");
    assert!(db.get(&uid).is_none());
    let slot = db.get(&renamed).unwrap();
    assert_eq!(slot.id, "old_lighthouse");
    assert_eq!(slot.name, "Old Lighthouse");
}

#[test]
fn rename_custom_slot_in_place_when_the_slug_is_unchanged_keeps_the_uid() {
    // Protects: renaming to a spelling that slugs the same keeps the original uid, only updating the display name.
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let result = db.rename_custom_slot(&uid, "LIGHTHOUSE");
    assert_eq!(result, uid);
    assert_eq!(db.get(&uid).unwrap().name, "LIGHTHOUSE");
    assert_eq!(db.get(&uid).unwrap().id, "lighthouse");
}

#[test]
fn rename_custom_slot_collision_keeps_the_old_uid_and_does_not_clobber_the_target() {
    // Protects: a rename that would collide with an existing slot is refused: the renaming slot keeps its old identity and the target slot is untouched.
    let mut db = AssetDB::new();
    let uid1 = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let uid2 = db.add_custom_slot("Buoy", Some("Naval")).uid.clone();
    let result = db.rename_custom_slot(&uid1, "Buoy");
    assert_eq!(result, uid1, "rename must be refused, not silently redirected");
    assert_eq!(db.get(&uid1).unwrap().name, "Lighthouse", "the renaming slot is untouched");
    assert_eq!(db.get(&uid2).unwrap().name, "Buoy", "the existing target slot is untouched");
}

#[test]
fn rename_custom_slot_is_a_no_op_for_an_unknown_uid() {
    // Protects: renaming an unknown uid is a no-op that returns the uid unchanged, not a panic or an inserted slot.
    let mut db = AssetDB::new();
    assert_eq!(db.rename_custom_slot("custom:nope/nope", "X"), "custom:nope/nope");
}

#[test]
fn rename_custom_slot_is_a_no_op_for_a_frozen_slot() {
    // Protects: a frozen (non-custom) slot cannot be renamed at all.
    let mut db = AssetDB::new();
    assert_eq!(db.rename_custom_slot("icons:mountain", "X"), "icons:mountain");
    assert_eq!(db.get("icons:mountain").unwrap().name, "Mountain");
}

#[test]
fn remove_custom_slot_removes_slot_store_and_collection_membership() {
    // Protects: removing a custom slot drops its registry entry, its item store and every collection membership together.
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    db.collections.add("Coastal", std::slice::from_ref(&uid));
    assert!(db.remove_custom_slot(&uid));
    assert!(db.get(&uid).is_none());
    assert!(db.collections.names().is_empty());
}

#[test]
fn remove_custom_slot_refuses_a_frozen_slot() {
    // Protects: a frozen slot cannot be removed via remove_custom_slot.
    let mut db = AssetDB::new();
    assert!(!db.remove_custom_slot("icons:mountain"));
    assert!(db.get("icons:mountain").is_some());
}

// ---------------------------------------------------------------------------
// AssetCollections
// ---------------------------------------------------------------------------

#[test]
fn collections_add_remove_drop_uid_rename_uid_membership_match_the_reference() {
    // Protects: add/remove/drop_uid/rename_uid and membership() compose correctly across a realistic sequence, matching the reference's own AssetCollections behaviour.
    let mut c = AssetCollections::new();
    c.add(
        "Coastal",
        &["icons:mountain".to_string(), "icons:hill".to_string(), "icons:mountain".to_string()],
    );
    assert_eq!(c.as_map().get("Coastal").unwrap(), &["icons:mountain", "icons:hill"]);

    c.remove("Coastal", "icons:hill");
    assert_eq!(c.as_map().get("Coastal").unwrap(), &["icons:mountain"]);

    c.add("Peaks", &["icons:mountain".to_string()]);
    assert_eq!(c.names(), vec!["Coastal", "Peaks"]);
    assert_eq!(c.membership("icons:mountain"), vec!["Coastal", "Peaks"]);

    c.rename_uid("icons:mountain", "icons:hill");
    assert_eq!(c.as_map().get("Coastal").unwrap(), &["icons:hill"]);
    assert_eq!(c.as_map().get("Peaks").unwrap(), &["icons:hill"]);

    c.drop_uid("icons:hill");
    assert!(c.names().is_empty());
}

#[test]
fn collections_removing_the_last_uid_drops_the_collection_entirely() {
    // Protects: removing a collection's last member drops the whole collection, not an empty array.
    let mut c = AssetCollections::new();
    c.add("Solo", &["icons:mountain".to_string()]);
    c.remove("Solo", "icons:mountain");
    assert!(c.names().is_empty());
}

#[test]
fn collections_blank_name_is_a_no_op_on_add() {
    // Protects: adding to a blank (post-trim) collection name is a no-op; no collection is created.
    let mut c = AssetCollections::new();
    c.add("   ", &["icons:mountain".to_string()]);
    assert!(c.names().is_empty());
}

// ---------------------------------------------------------------------------
// AssetDB counting
// ---------------------------------------------------------------------------

#[test]
fn filled_count_and_total_items_match_the_reference() {
    // Protects: filled_count and total_items report per-family and whole-registry counts correctly.
    let mut db = AssetDB::new();
    db.add_item("icons:mountain", item("m1", "h1"));
    db.add_item("icons:mountain", item("m2", "h2"));
    db.add_item("icons:hill", item("h1", "h3"));
    assert_eq!(db.filled_count(cartalith_assets::Family::Icons), 2);
    assert_eq!(db.filled_count(cartalith_assets::Family::Textures), 0);
    assert_eq!(db.total_items(), 3);
}

// ---------------------------------------------------------------------------
// library.json record shape, via the real reference `_alExportEntries`
// ---------------------------------------------------------------------------

#[test]
fn export_minimal_one_icon_item_with_pack_fields_set() {
    // Protects: to_library_json produces the exact reference record shape for a minimal one-item, pack-fields-set library.
    let mut db = AssetDB::new();
    db.pack = PackInfo {
        name: "My Pack".to_string(),
        author: "Author A".to_string(),
        license: "CC0".to_string(),
    };
    db.add_item(
        "icons:mountain",
        item("m1.png", "hashA").with_transform(ItemTransform {
            scale: 1.2,
            pan_x: 0.1,
            pan_y: -0.2,
        }),
    );

    let file = db.to_library_json().unwrap();
    let mountain_rules = db.get("icons:mountain").unwrap().rules.clone().unwrap();
    let expected = LibraryFile {
        version: 1,
        kind: "cartalith-assetlib".to_string(),
        pack: Some(db.pack.clone()),
        collections: Default::default(),
        slots: vec![SlotRecord {
            fam: "icons".to_string(),
            id: "mountain".to_string(),
            name: "Mountain".to_string(),
            meta: SlotMeta::default(),
            items: vec![ItemRecord {
                img: 0,
                name: "m1.png".to_string(),
                t: ItemTransform {
                    scale: 1.2,
                    pan_x: 0.1,
                    pan_y: -0.2,
                },
            }],
            set: None,
            rules: Some(mountain_rules),
        }],
    };
    assert_eq!(file, expected);
}

#[test]
fn export_custom_slot_with_tags_and_no_items_is_still_included() {
    // Protects: a custom slot always survives the export inclusion filter, even with zero items, because fam.custom always does.
    // fam.custom always survives the inclusion filter, even with zero items.
    let mut db = AssetDB::new();
    db.add_item("icons:mountain", item("m1.png", "hashA")); // keep totalItems()>0
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    let meta = db.slot_meta_mut(&uid).unwrap();
    meta.tags = vec!["coastal".to_string(), "landmark".to_string()];
    meta.author = "Jane".to_string();

    let file = db.to_library_json().unwrap();
    let lighthouse = file.slots.iter().find(|s| s.id == "lighthouse").expect("custom slot must be present");
    assert!(lighthouse.items.is_empty());
    assert_eq!(lighthouse.set.as_deref(), Some("Naval"));
    assert!(lighthouse.rules.is_some());
    assert_eq!(lighthouse.meta.author, "Jane");
    assert_eq!(lighthouse.meta.tags, vec!["coastal", "landmark"]);
}

#[test]
fn export_frozen_slot_with_tags_but_no_items_is_included_because_of_the_tags() {
    // Protects: a frozen slot with tags but no items is still included in the export, because it carries tags.
    let mut db = AssetDB::new();
    db.add_item("icons:mountain", item("m1.png", "hashA")); // keep totalItems()>0
    db.slot_meta_mut("icons:shrub").unwrap().tags = vec!["sparse".to_string()];

    let file = db.to_library_json().unwrap();
    let shrub = file.slots.iter().find(|s| s.id == "shrub").expect("tagged frozen slot must be present");
    assert!(shrub.items.is_empty());
    assert_eq!(shrub.meta.tags, vec!["sparse"]);
}

#[test]
fn export_frozen_slot_with_no_items_and_no_tags_is_excluded_entirely() {
    // Protects: a frozen slot with neither items nor tags is dropped from the export entirely.
    let mut db = AssetDB::new();
    db.add_item("icons:mountain", item("m1.png", "hashA"));
    // icons:hill carries nothing -> must not appear.
    let file = db.to_library_json().unwrap();
    assert_eq!(file.slots.len(), 1);
    assert_eq!(file.slots[0].id, "mountain");
    assert!(!file.slots.iter().any(|s| s.id == "hill"));
}

#[test]
fn export_collections_travel_verbatim() {
    // Protects: collections are carried into the exported record unchanged.
    let mut db = AssetDB::new();
    let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
    db.collections.add("Coastal", std::slice::from_ref(&uid));
    db.add_item(&uid, item("l1.png", "hashL"));
    let file = db.to_library_json().unwrap();
    assert_eq!(file.collections.get("Coastal").unwrap(), &[uid]);
}

#[test]
fn export_of_a_totally_empty_library_is_none() {
    // Protects: to_library_json returns None for a library with zero total items, matching the reference's own null-return guard.
    let mut db = AssetDB::new();
    assert!(db.to_library_json().is_none());
}
