//! The Asset Library's mutable, in-memory session: `GUI_GAP_REGISTER.md`
//! rows AS-01 through AS-08/AS-13 and DM-05 — every one of them a real
//! `cartalith-assets` function with no `#[func]` in front of it (verified by
//! reading `library.rs`/`raster.rs`/`archive.rs` directly, the same "read
//! before wiring" discipline `travel_bridge.rs` and `pack.rs` both document).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `travel_bridge.rs`/`journey_bridge.rs`/`timeline_bridge.rs` already
//! establish: this module owns everything expressible without one, and
//! `lib.rs`'s `#[godot_api(secondary)]` block owns the thin `Variant`<->Rust
//! conversion.
//!
//! # Why a parallel pixel store, not `AssetDB` alone
//!
//! `cartalith_assets::AssetDB` (`library.rs`) is deliberately pixel-free —
//! its own module doc says so: `LibraryItem` carries a `hash: String` but no
//! `img`/`w`/`h`. Every real operation this session needs (thumbnails, a
//! pack export's baked slot image, `Apply to map`) needs the actual decoded
//! pixels behind an item, so [`AssetLibrarySession`] keeps a second store,
//! [`AssetLibrarySession::images`], index-parallel to `AssetDB`'s own
//! `store[uid]` — same uid, same order, same length, always. Every method
//! below that mutates one mutates the other in the same call, so the two
//! never drift.
//!
//! # What this session does and does not restore
//!
//! This is the *authoring* session — `AssetImporter.intake`/`AssetBrowserUI`'s
//! batch operations/`PackManifestBuilder.build`/`applyToMap`/`exportPack` in
//! reference terms. It does not read a saved `assetlib/library.json` project
//! section back in (`AssetDB::apply_library_file_with_items`, a different,
//! save-format-adjacent feature `SAVEFILE_COMPAT.md` territory owns and this
//! dispatch was not asked to wire).

use std::collections::HashMap;

use cartalith_assets::{
    AssetDB, Anchor, DecodedImage, Family, ItemTransform, LibraryItem, PackEntries, PackManifest,
    archive, decode_png, encode_png, fit_to_bottom, item_hash, library, render_item,
};

/// The live Asset Library: `AssetDB`'s metadata registry plus the decoded
/// pixels behind every stored item.
pub struct AssetLibrarySession {
    pub db: AssetDB,
    /// uid -> decoded variants, index-parallel to `db`'s own `items(uid)`.
    images: HashMap<String, Vec<DecodedImage>>,
}

impl Default for AssetLibrarySession {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLibrarySession {
    pub fn new() -> Self {
        AssetLibrarySession { db: AssetDB::new(), images: HashMap::new() }
    }

    /// The decoded pixels behind one stored item, if both `uid` and `index`
    /// resolve on both sides of the parallel store.
    pub fn image(&self, uid: &str, index: usize) -> Option<&DecodedImage> {
        self.images.get(uid)?.get(index)
    }

    // -- Import ---------------------------------------------------------

    /// Decode `bytes` as a PNG and add it as a new item on `uid` — the
    /// reference's `AssetImporter.intake` (per-file half): computes a real
    /// [`item_hash`], and, for a bottom-anchored family (feature icons),
    /// applies [`fit_to_bottom`] on intake exactly as the reference does.
    /// `Err` for an unknown `uid` or bytes that don't decode as a PNG —
    /// never a panic across the boundary.
    pub fn import_item(&mut self, uid: &str, name: String, bytes: &[u8]) -> Result<(), String> {
        let Some(slot) = self.db.get(uid) else {
            return Err(format!("no such slot: {uid}"));
        };
        let family = slot.family;
        let decoded = decode_png(bytes).map_err(|e| e.to_string())?;
        let mut transform = ItemTransform::default();
        if family.anchor() == Anchor::Bottom {
            fit_to_bottom(&mut transform, decoded.w, decoded.h, family.size());
        }
        let hash = item_hash(&decoded);
        let item = LibraryItem::new(name, hash).with_transform(transform);
        if !self.db.add_item(uid, item) {
            return Err(format!("no such slot: {uid}"));
        }
        self.images.entry(uid.to_string()).or_default().push(decoded);
        Ok(())
    }

    /// Add (or return the existing) custom slot — `AssetDB::add_custom_slot`,
    /// exposed so a caller can land a loose image somewhere real before
    /// [`import_item`] has a uid to target.
    pub fn add_custom_slot(&mut self, name: &str, set_name: Option<&str>) -> String {
        self.db.add_custom_slot(name, set_name).uid.clone()
    }

    // -- Removal / clearing ----------------------------------------------

    /// Remove one item, keeping both stores in lockstep.
    pub fn remove_item(&mut self, uid: &str, index: usize) -> bool {
        let removed = self.db.remove_item(uid, index).is_some();
        if removed
            && let Some(v) = self.images.get_mut(uid)
            && index < v.len()
        {
            v.remove(index);
        }
        removed
    }

    /// Drop every item from a slot without removing the slot itself — the
    /// reference's `store[uid]=[]`, used by batch-delete on a *frozen* slot
    /// (a frozen slot cannot be removed, only emptied; see
    /// [`AssetLibrarySession::remove_custom_slot`] for the custom half).
    fn clear_slot_items(&mut self, uid: &str) {
        while self.db.remove_item(uid, 0).is_some() {}
        self.images.remove(uid);
    }

    /// Remove a custom slot entirely (and its images) — `false` for an
    /// unknown or frozen `uid`.
    pub fn remove_custom_slot(&mut self, uid: &str) -> bool {
        let ok = self.db.remove_custom_slot(uid);
        if ok {
            self.images.remove(uid);
        }
        ok
    }

    /// Reset to a freshly bootstrapped, empty library — `AssetDB::clear`
    /// plus dropping every decoded pixel.
    pub fn clear(&mut self) {
        self.db.clear();
        self.images.clear();
    }

    // -- Validation --------------------------------------------------------

    /// `AssetValidator.run()` — real warning strings, ordered exactly as the
    /// reference produces them.
    pub fn validate(&self) -> Vec<String> {
        library::run(&self.db)
    }

    // -- Thumbnails ----------------------------------------------------------

    /// A `render_item`-baked, PNG-encoded thumbnail for one stored item —
    /// the shared render core the reference itself uses for grid
    /// thumbnails, the inspector preview, and export bake alike. `None` when
    /// the uid/index/pixels don't all resolve, or if re-encoding fails.
    pub fn thumbnail_png(&self, uid: &str, index: usize, size: u32) -> Option<Vec<u8>> {
        let slot = self.db.get(uid)?;
        let item = self.db.items(uid).get(index)?;
        let img = self.image(uid, index)?;
        let baked = render_item(img, &item.transform, size, slot.family.opaque());
        encode_png(&baked).ok()
    }

    // -- Batch operations (`AssetBrowserUI.init`'s `alBatch*` handlers) -----

    /// `alBatchTag`: append every tag in `tags` (deduplicated per slot) to
    /// each of `uids`' `SlotMeta::tags`. Unknown uids are skipped, not an
    /// error — a batch op over a stale selection should apply to whatever
    /// still exists.
    pub fn batch_tag(&mut self, uids: &[String], tags: &[String]) -> usize {
        let mut tagged = 0usize;
        for uid in uids {
            let Some(meta) = self.db.slot_meta_mut(uid) else { continue };
            for t in tags {
                if !t.is_empty() && !meta.tags.contains(t) {
                    meta.tags.push(t.clone());
                }
            }
            tagged += 1;
        }
        tagged
    }

    /// `alBatchColl`: add every uid in `uids` to collection `name` —
    /// `AssetCollections::add`'s own blank-name-is-a-no-op and
    /// no-duplicate-membership rules apply unchanged.
    pub fn batch_collect(&mut self, name: &str, uids: &[String]) {
        self.db.collections.add(name, uids);
    }

    /// `alBatchRename`: `uids` become `{base}_01`, `{base}_02`, … in
    /// selection order. A custom slot is renamed via
    /// `AssetDB::rename_custom_slot` (one `_NN` per slot); a frozen slot
    /// instead renames each of its *item variants* in place via
    /// [`cartalith_assets::AssetDB::item_mut`] (`_NN` advances by
    /// `max(1, item_count)`), matching the reference's own
    /// `store[uid].forEach((it,i)=>{it.name=...})` — frozen slots are not
    /// nameable at all (`slot_title` is a constant), only their variants
    /// are. Returns `(renamed_slot_count, remap)`, where `remap` carries
    /// `old_uid -> new_uid` for every custom slot whose uid changed (a
    /// caller's selection set needs to follow it).
    pub fn batch_rename(&mut self, uids: &[String], base: &str) -> (usize, Vec<(String, String)>) {
        let mut n = 1usize;
        let mut renamed = 0usize;
        let mut remap = Vec::new();
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if slot.family == Family::Custom {
                let new_name = format!("{base}_{n:02}");
                let nuid = self.db.rename_custom_slot(uid, &new_name);
                if &nuid != uid {
                    if let Some(v) = self.images.remove(uid) {
                        self.images.insert(nuid.clone(), v);
                    }
                    remap.push((uid.clone(), nuid));
                }
                n += 1;
            } else {
                let count = self.db.items(uid).len().max(1);
                for i in 0..self.db.items(uid).len() {
                    let name = format!("{base}_{:02}", n + i);
                    if let Some(item) = self.db.item_mut(uid, i) {
                        item.name = name;
                    }
                }
                n += count;
            }
            renamed += 1;
        }
        (renamed, remap)
    }

    /// `alBatchDup`: for each of `uids` carrying at least one item, a new
    /// custom slot named `"{slot name} copy"` under the `"Duplicates"` set,
    /// with every source item (and its decoded pixels) cloned in. Returns
    /// how many slots were duplicated.
    pub fn batch_duplicate(&mut self, uids: &[String]) -> usize {
        let mut made = 0usize;
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if self.db.items(uid).is_empty() {
                continue;
            }
            let new_name = format!("{} copy", slot.name);
            let new_uid = self.db.add_custom_slot(&new_name, Some("Duplicates")).uid.clone();
            let items: Vec<LibraryItem> = self.db.items(uid).to_vec();
            let pixels: Vec<DecodedImage> = self.images.get(uid).cloned().unwrap_or_default();
            for item in items {
                self.db.add_item(&new_uid, item);
            }
            self.images.entry(new_uid).or_default().extend(pixels);
            made += 1;
        }
        made
    }

    /// `alBatchDel`: for each of `uids`, remove the whole slot if it's
    /// custom, or just empty its items if it's frozen (a frozen slot can
    /// never be removed). Returns how many slots were affected.
    pub fn batch_delete(&mut self, uids: &[String]) -> usize {
        let mut deleted = 0usize;
        for uid in uids {
            let Some(slot) = self.db.get(uid) else { continue };
            if slot.family == Family::Custom {
                if self.remove_custom_slot(uid) {
                    deleted += 1;
                }
            } else {
                self.clear_slot_items(uid);
                deleted += 1;
            }
        }
        deleted
    }

    // -- Export / apply ------------------------------------------------------

    /// `PackManifestBuilder.build()` + `zipStore`: bake every stored item at
    /// its family's own size/opacity (`render_item`), PNG-encode it, place
    /// it at the exporter's own path convention (`Family::asset_path`), and
    /// write the whole thing out as a pack `.zip` via
    /// `archive::write_pack`. `Err("Library is empty.")` when nothing is
    /// stored at all — the reference's own `AssetDB.totalItems()===0` guard
    /// on both `exportPack` and `applyToMap`. Pack name/author/license fall
    /// back to the reference's own defaults (`'Custom Asset Pack'`/
    /// `'Cartalith Gen1'`/`'CC0'`) when blank. Returns `(pack_name, bytes)`.
    pub fn export_pack_bytes(&mut self) -> Result<(String, Vec<u8>), String> {
        if self.db.total_items() == 0 {
            return Err("Library is empty.".to_string());
        }
        let name = if self.db.pack.name.trim().is_empty() {
            "Custom Asset Pack".to_string()
        } else {
            self.db.pack.name.trim().to_string()
        };
        let author = if self.db.pack.author.trim().is_empty() {
            "Cartalith Gen1".to_string()
        } else {
            self.db.pack.author.trim().to_string()
        };
        let license = if self.db.pack.license.trim().is_empty() {
            "CC0".to_string()
        } else {
            self.db.pack.license.trim().to_string()
        };

        let mut manifest = PackManifest { name: name.clone(), author, license, ..Default::default() };
        let mut images = PackEntries::new();

        for uid in self.db.uids_in_order() {
            let Some(slot) = self.db.get(&uid).cloned() else { continue };
            let items = self.db.items(&uid);
            if items.is_empty() {
                continue;
            }
            let family = slot.family;
            let set_id = slot.set_id.clone().unwrap_or_default();
            let mut paths = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let Some(pixels) = self.image(&uid, i) else { continue };
                let baked = render_item(pixels, &item.transform, family.size(), family.opaque());
                let png = encode_png(&baked).map_err(|e| e.to_string())?;
                let path = family.asset_path(&slot.id, &set_id, i);
                images.insert(path.clone(), png);
                paths.push(path);
            }
            if paths.is_empty() {
                continue;
            }
            match family {
                Family::Textures => {
                    manifest.textures.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Biomes => {
                    manifest.biomes.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Terrains => {
                    manifest.terrains.insert(slot.id.clone(), paths.into_iter().next().expect("non-empty"));
                }
                Family::Icons => {
                    manifest.icons.insert(slot.id.clone(), paths);
                }
                Family::Settlement => {
                    manifest.structures.settlement.insert(slot.id.clone(), paths);
                }
                Family::Trait => {
                    manifest.structures.traits.insert(slot.id.clone(), paths);
                }
                Family::Poi => {
                    manifest.structures.poi.insert(slot.id.clone(), paths);
                }
                Family::Custom => {
                    let set_name = slot.set.clone().unwrap_or_default();
                    if manifest.custom.get(&set_name).is_none() {
                        manifest.custom.insert(set_name.clone(), Default::default());
                    }
                    manifest.custom.get_mut(&set_name).expect("just inserted").insert(slot.id.clone(), paths);
                }
            }
        }

        let mut buf = Vec::new();
        archive::write_pack(std::io::Cursor::new(&mut buf), &manifest, &images).map_err(|e| e.to_string())?;
        Ok((name, buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        let img = DecodedImage::new(w, h, data).unwrap();
        encode_png(&img).unwrap()
    }

    #[test]
    fn import_item_into_a_frozen_slot_fits_to_bottom_and_stores_pixels() {
        let mut s = AssetLibrarySession::new();
        // Wider than tall, so `w` (not `h`) is the longer side: `fit_to_bottom`
        // then leaves genuine vertical slack to centre away from zero.
        s.import_item("icons:mountain", "m1.png".to_string(), &png_bytes(16, 8, [1, 2, 3, 255])).unwrap();
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert!(s.image("icons:mountain", 0).is_some());
        // fit_to_bottom mutates pan_y away from the identity default for a
        // non-square item on a bottom-anchored family.
        assert_ne!(s.db.items("icons:mountain")[0].transform.pan_y, 0.0);
    }

    #[test]
    fn import_item_into_settlement_family_does_not_fit_to_bottom() {
        let mut s = AssetLibrarySession::new();
        s.import_item("settlement:hamlet", "h.png".to_string(), &png_bytes(16, 8, [1, 2, 3, 255])).unwrap();
        assert_eq!(s.db.items("settlement:hamlet")[0].transform.pan_y, 0.0);
    }

    #[test]
    fn import_item_rejects_unknown_uid_and_bad_bytes_without_panicking() {
        let mut s = AssetLibrarySession::new();
        assert!(s.import_item("nope:nope", "x.png".to_string(), &png_bytes(4, 4, [0, 0, 0, 255])).is_err());
        assert!(s.import_item("icons:mountain", "x.png".to_string(), b"not a png").is_err());
    }

    #[test]
    fn remove_item_keeps_both_stores_in_lockstep() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        s.import_item("icons:mountain", "b.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();
        assert!(s.remove_item("icons:mountain", 0));
        assert_eq!(s.db.items("icons:mountain").len(), 1);
        assert_eq!(s.db.items("icons:mountain")[0].name, "b.png");
        assert_eq!(s.image("icons:mountain", 0).unwrap().rgba[1], 1);
        assert!(!s.remove_item("icons:mountain", 5));
    }

    #[test]
    fn batch_delete_empties_a_frozen_slot_but_removes_a_custom_one() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();

        let n = s.batch_delete(&[cuid.clone(), "icons:mountain".to_string()]);
        assert_eq!(n, 2);
        assert!(s.db.get(&cuid).is_none(), "custom slot removed entirely");
        assert!(s.db.get("icons:mountain").is_some(), "frozen slot survives");
        assert!(s.db.items("icons:mountain").is_empty(), "but its items are gone");
        assert!(s.image("icons:mountain", 0).is_none());
    }

    #[test]
    fn batch_rename_renames_custom_slots_and_frozen_item_variants_differently() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        s.import_item("icons:mountain", "b.png".to_string(), &png_bytes(4, 4, [1, 0, 0, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [0, 1, 0, 255])).unwrap();

        let (n, remap) = s.batch_rename(&["icons:mountain".to_string(), cuid.clone()], "Base");
        assert_eq!(n, 2);
        assert_eq!(s.db.items("icons:mountain")[0].name, "Base_01");
        assert_eq!(s.db.items("icons:mountain")[1].name, "Base_02");
        assert_eq!(remap.len(), 1);
        let new_uid = &remap[0].1;
        assert_eq!(s.db.get(new_uid).unwrap().name, "Base_03");
        assert!(s.image(new_uid, 0).is_some(), "pixel store followed the uid change");
    }

    #[test]
    fn batch_duplicate_clones_items_and_pixels_into_a_new_custom_slot() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "a.png".to_string(), &png_bytes(4, 4, [9, 9, 9, 255])).unwrap();
        let made = s.batch_duplicate(&["icons:mountain".to_string()]);
        assert_eq!(made, 1);
        let dup_uid = s
            .db
            .slots_in_family(Family::Custom)
            .into_iter()
            .find(|s| s.name == "Mountain copy")
            .unwrap()
            .uid
            .clone();
        assert_eq!(s.db.items(&dup_uid).len(), 1);
        assert_eq!(s.image(&dup_uid, 0).unwrap().rgba[0], 9);
    }

    #[test]
    fn batch_tag_deduplicates_per_slot() {
        let mut s = AssetLibrarySession::new();
        let n = s.batch_tag(
            &["icons:mountain".to_string()],
            &["tall".to_string(), "tall".to_string(), "rocky".to_string()],
        );
        assert_eq!(n, 1);
        assert_eq!(s.db.get("icons:mountain").unwrap().meta.tags, vec!["tall", "rocky"]);
    }

    #[test]
    fn export_pack_bytes_fails_honestly_on_an_empty_library() {
        let mut s = AssetLibrarySession::new();
        assert!(s.export_pack_bytes().is_err());
    }

    #[test]
    fn export_pack_bytes_round_trips_through_read_pack() {
        let mut s = AssetLibrarySession::new();
        s.db.pack.name = "My Pack".to_string();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(8, 8, [10, 20, 30, 255])).unwrap();
        s.import_item("textures:grass", "g.png".to_string(), &png_bytes(8, 8, [40, 50, 60, 255])).unwrap();
        let (name, bytes) = s.export_pack_bytes().unwrap();
        assert_eq!(name, "My Pack");

        let (manifest, _entries) = cartalith_assets::read_pack(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(manifest.name, "My Pack");
        assert_eq!(manifest.icons.get("mountain").unwrap().len(), 1);
        assert_eq!(manifest.textures.get("grass").unwrap(), "textures/grass.png");
        assert!(manifest.warnings.is_empty());
    }

    #[test]
    fn thumbnail_png_bakes_a_real_size_by_size_image() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(8, 8, [1, 2, 3, 255])).unwrap();
        let png = s.thumbnail_png("icons:mountain", 0, 32).unwrap();
        let decoded = decode_png(&png).unwrap();
        assert_eq!((decoded.w, decoded.h), (32, 32));
    }

    #[test]
    fn clear_drops_every_item_and_every_decoded_pixel() {
        let mut s = AssetLibrarySession::new();
        s.import_item("icons:mountain", "m.png".to_string(), &png_bytes(4, 4, [1, 2, 3, 255])).unwrap();
        let cuid = s.add_custom_slot("Lighthouse", Some("Naval"));
        s.import_item(&cuid, "l.png".to_string(), &png_bytes(4, 4, [1, 2, 3, 255])).unwrap();
        s.clear();
        assert_eq!(s.db.total_items(), 0);
        assert!(s.db.get(&cuid).is_none());
        assert!(s.image("icons:mountain", 0).is_none());
    }
}
