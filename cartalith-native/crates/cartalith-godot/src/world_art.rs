//! Ruling GW (`LARGE_ITEM_RULINGS.md`, owner 2026-11-22; `SAVEFILE_COMPAT.md` §16.5): a save carries the art its world uses.
//!
//! The device library (`user://asset_library/`, Ruling FR) holds the pictures; a world's `library/assets.json` holds only the
//! records. So a world opened on another device drew from a library that lacked its images. Now the writer embeds, under
//! `library/art/<key>.png`, **only the pictures of slots the world actually uses** -- a slot a hand-placed or generated icon
//! names (`annotations/icons.json`), or a custom pin a point of interest uses (`annotations/pois.json`) -- and tags each
//! embedded item in `library/assets.json` with `"h": "<key>"`, the same per-item field the device store's `index.json` uses.
//! `key` is the store's own [`content_key`] (`<16 hex content hash>-<w>x<h>`), so equal keys are equal pixels.
//!
//! # Opening
//!
//! [`restore_art`] runs after the record is merged ([`AssetDB::merge_library_file`](cartalith_assets::AssetDB)). It **only
//! adds**: an item is added to its slot when the slot has no item with that key. It never replaces, edits or removes a device
//! item, so there is nothing to ask about; because a key is a hash of the pixels, "an existing key with different bytes" cannot
//! arise except by a 64-bit collision or a tampered file, and a picture whose bytes do not hash to the key it is filed under is
//! refused and reported, never added. A pin slot the device already has (it holds a picture) is left as the device's own: the
//! world still draws its pin from the copy in `annotations/pois.json`. No `library/art/` means [`restore_art`] does nothing.

use std::collections::{BTreeMap, HashMap, HashSet};

use cartalith_assets::pins::PINS_SET;
use cartalith_assets::{Family, decode_png, encode_png, parse_library_json};
use cartalith_civ::poi::Poi;

use crate::asset_bridge::AssetLibrarySession;
use crate::asset_bridge::store::content_key;

/// Where an embedded picture lives in the archive: `library/art/<key>.png`.
pub const ART_PREFIX: &str = "library/art/";

/// The library slots a world draws from: each slot a placed icon names (family, slot id, and for the open `custom` family the set
/// name), and each slot whose pin a point of interest uses. Never includes a slot nothing places.
pub fn used_slots(session: &AssetLibrarySession, icons: &[cartalith_assets::manual::ManualIcon], pois: &[Poi]) -> HashSet<String> {
    let mut used = HashSet::new();
    for icon in icons {
        let family = icon.family.pack_family();
        for slot in session.db.slots_in_family(family) {
            if slot.id == icon.slot && (family != Family::Custom || slot.set == icon.set) {
                used.insert(slot.uid.clone());
            }
        }
    }
    if pois.iter().any(|p| p.kind.starts_with("custom")) {
        for pin in session.pin_catalog() {
            if pois.iter().any(|p| p.kind == pin.kind) {
                used.insert(pin.uid);
            }
        }
    }
    used
}

/// What the writer embeds for `used`: the `img` number (the running counter `AssetDB::to_library_file` gives every item) of
/// each embedded item mapped to its key, and the PNGs by key. An item without a decoded picture is skipped, not invented.
pub fn art_of(session: &AssetLibrarySession, used: &HashSet<String>) -> (HashMap<usize, String>, BTreeMap<String, Vec<u8>>) {
    let mut key_of_img = HashMap::new();
    let mut art = BTreeMap::new();
    let mut n = 0usize;
    for uid in session.db.uids_in_order() {
        for i in 0..session.db.items(&uid).len() {
            if used.contains(&uid)
                && let Some(img) = session.image(&uid, i)
                && let Ok(png) = encode_png(img)
            {
                let key = content_key(img);
                key_of_img.insert(n, key.clone());
                art.insert(key, png);
            }
            n += 1;
        }
    }
    (key_of_img, art)
}

/// `record` (the serialised `LibraryFile`) with `"h": "<key>"` added to every item [`art_of`] embedded. The same text when none was.
pub fn tag_record(record: serde_json::Value, key_of_img: &HashMap<usize, String>) -> serde_json::Value {
    let mut v = record;
    for slot in v.get_mut("slots").and_then(serde_json::Value::as_array_mut).into_iter().flatten() {
        for it in slot.get_mut("items").and_then(serde_json::Value::as_array_mut).into_iter().flatten() {
            let Some(img) = it.get("img").and_then(serde_json::Value::as_u64) else { continue };
            if let (Some(k), Some(obj)) = (key_of_img.get(&(img as usize)), it.as_object_mut()) {
                obj.insert("h".to_string(), serde_json::Value::String(k.clone()));
            }
        }
    }
    v
}

/// What [`restore_art`] did, for the open's notes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ArtReport {
    /// Pictures added to the device library.
    pub added: usize,
    /// Pictures the device already had (same slot, same key), or a pin slot the device already fills.
    pub present: usize,
    /// One sentence per embedded picture that was refused (unreadable, or its bytes do not hash to its key).
    pub refused: Vec<String>,
}

/// Adds the embedded pictures `art` (`library/art/<key>.png` entries, key -> PNG bytes) that the device library lacks, using
/// the `h`-tagged record `record_text` to say which slot each belongs to and under what name and transform. Call after the
/// record has been merged so its custom slots exist. See the module doc for the rules; nothing is ever replaced.
pub fn restore_art(session: &mut AssetLibrarySession, record_text: &str, art: &BTreeMap<String, Vec<u8>>) -> ArtReport {
    let mut report = ArtReport::default();
    if art.is_empty() {
        return report;
    }
    let Ok(file) = parse_library_json(record_text.as_bytes()) else { return report };
    let raw: serde_json::Value = serde_json::from_str(record_text).unwrap_or_default();
    let mut key_of_img: HashMap<usize, String> = HashMap::new();
    for slot in raw.get("slots").and_then(serde_json::Value::as_array).into_iter().flatten() {
        for it in slot.get("items").and_then(serde_json::Value::as_array).into_iter().flatten() {
            if let (Some(img), Some(h)) = (it.get("img").and_then(serde_json::Value::as_u64), it.get("h").and_then(serde_json::Value::as_str)) {
                key_of_img.insert(img as usize, h.to_string());
            }
        }
    }
    for rec in &file.slots {
        let Some(family) = Family::from_key(&rec.fam) else { continue };
        let Some(uid) = session.db.slots_in_family(family).into_iter().find(|s| s.id == rec.id && s.set == rec.set).map(|s| s.uid.clone()) else {
            continue;
        };
        let is_pin = family == Family::Custom && rec.set.as_deref() == Some(PINS_SET);
        for item in &rec.items {
            let Some(key) = key_of_img.get(&item.img) else { continue };
            let Some(bytes) = art.get(key) else { continue };
            let n = session.db.items(&uid).len();
            let have = (0..n).any(|i| session.image(&uid, i).is_some_and(|img| &content_key(img) == key));
            if have || (is_pin && n > 0) {
                report.present += 1;
                continue;
            }
            match decode_png(bytes) {
                Ok(img) if &content_key(&img) == key => match session.insert_decoded(&uid, item.name.clone(), img) {
                    Ok(_) => {
                        if let Some(it) = session.db.item_mut(&uid, n) {
                            it.transform = item.t.clone();
                        }
                        report.added += 1;
                    }
                    Err(e) => report.refused.push(format!("{ART_PREFIX}{key}.png: {e}")),
                },
                Ok(_) => report.refused.push(format!("{ART_PREFIX}{key}.png does not hold the picture its name says; not added")),
                Err(e) => report.refused.push(format!("{ART_PREFIX}{key}.png is not a readable PNG ({e}); not added")),
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_assets::DecodedImage;

    /// A small opaque PNG whose pixels depend on `seed`, so two seeds are two different pictures.
    fn png(seed: u8) -> Vec<u8> {
        let rgba: Vec<u8> = (0..16 * 16).flat_map(|i| [seed.wrapping_mul(7).wrapping_add(i as u8), seed, 200, 255]).collect();
        encode_png(&DecodedImage { w: 16, h: 16, rgba }).unwrap()
    }

    /// Builds device A: one custom pin and one custom icon slot with two pictures, only one icon of which the world places.
    fn device_a() -> (AssetLibrarySession, HashSet<String>) {
        let mut a = AssetLibrarySession::new();
        let pin = a.pin_add("Beacon", "#ff0000", &png(1)).unwrap();
        let slot = a.add_custom_slot("Oak", Some("Trees"));
        a.import_item(&slot, "oak1".into(), &png(2)).unwrap();
        a.import_item(&slot, "oak2".into(), &png(3)).unwrap();
        let unused = a.add_custom_slot("Unused", Some("Trees"));
        a.import_item(&unused, "u".into(), &png(4)).unwrap();
        (a, HashSet::from([pin, slot]))
    }

    /// Protects the round trip GW exists for: the used slots' pictures (and only those) are written, and an empty device B
    /// ends up with the same pictures under the same names, transforms and slots after the record is merged and the art restored.
    #[test]
    fn art_written_from_device_a_arrives_on_an_empty_device_b() {
        let (mut a, used) = device_a();
        let oak_a = used.iter().find(|u| u.ends_with("/oak")).unwrap().clone();
        a.db.item_mut(&oak_a, 1).unwrap().transform.scale = 1.75; // a transform the default would not give
        let (key_of_img, art) = art_of(&a, &used);
        assert_eq!(art.len(), 3, "pin + two oak pictures, not the unused slot's");
        let file = a.db.to_library_file();
        let record = tag_record(serde_json::to_value(&file).unwrap(), &key_of_img).to_string();

        let mut b = AssetLibrarySession::new();
        let parsed = parse_library_json(record.as_bytes()).unwrap();
        b.db.merge_library_file(&parsed);
        assert_eq!(b.db.total_items(), 0, "the merge alone adds no picture (Ruling FR)");
        let r = restore_art(&mut b, &record, &art);
        assert_eq!(r, ArtReport { added: 3, present: 0, refused: vec![] });
        assert_eq!(b.db.total_items(), 3, "the unused slot's picture was never embedded");
        let oak = b.db.slots_in_family(Family::Custom).into_iter().find(|s| s.id == "oak").unwrap().uid.clone();
        assert_eq!(b.db.items(&oak).iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["oak1", "oak2"]);
        for i in 0..2 {
            assert_eq!(content_key(b.image(&oak, i).unwrap()), content_key(a.image(&used.iter().find(|u| u.ends_with("/oak")).unwrap().clone(), i).unwrap()));
        }
        assert_eq!(b.db.items(&oak)[1].transform.scale, 1.75, "the item's transform came with it");
        assert_eq!(b.pin_catalog().len(), 1, "the pin is now in B's palette");
        assert_eq!(b.pin_catalog()[0].kind, a.pin_catalog()[0].kind, "and under the same kind id");

        // Opening again changes nothing: nothing is added twice.
        let again = restore_art(&mut b, &record, &art);
        assert_eq!((again.added, again.present), (0, 3));
        assert_eq!(b.db.total_items(), 3);
    }

    /// Protects "never replace": a device that already holds a different picture in the slot keeps it, and the world's picture
    /// is added beside it only when it is not already there; a pin slot the device fills is left alone.
    #[test]
    fn a_device_picture_is_never_replaced_and_a_filled_pin_slot_is_left_alone() {
        let (mut a, used) = device_a();
        let (key_of_img, art) = art_of(&a, &used);
        let record = tag_record(serde_json::to_value(a.db.to_library_file()).unwrap(), &key_of_img).to_string();
        let mut b = AssetLibrarySession::new();
        b.pin_add("Beacon", "#00ff00", &png(9)).unwrap();
        let slot = b.add_custom_slot("Oak", Some("Trees"));
        b.import_item(&slot, "mine".into(), &png(5)).unwrap();
        b.db.merge_library_file(&parse_library_json(record.as_bytes()).unwrap());
        let before = content_key(b.image(&slot, 0).unwrap());
        let r = restore_art(&mut b, &record, &art);
        assert_eq!(content_key(b.image(&slot, 0).unwrap()), before, "the device's own picture is untouched");
        assert_eq!(r.added, 2, "the two oak pictures are added beside it");
        assert!(r.present >= 1, "the pin slot the device fills is left as its own");
        assert_eq!(b.pin_catalog().len(), 1);
        assert_eq!(b.pin_catalog()[0].color, "#00ff00", "the device's pin keeps its colour");
    }

    /// Protects the integrity rule: bytes filed under a key they do not hash to are refused and reported, not added.
    #[test]
    fn a_picture_that_does_not_match_its_key_is_refused() {
        let (mut a, used) = device_a();
        let (key_of_img, mut art) = art_of(&a, &used);
        let record = tag_record(serde_json::to_value(a.db.to_library_file()).unwrap(), &key_of_img).to_string();
        let k = art.keys().next().unwrap().clone();
        art.insert(k, png(77));
        let mut b = AssetLibrarySession::new();
        b.db.merge_library_file(&parse_library_json(record.as_bytes()).unwrap());
        let r = restore_art(&mut b, &record, &art);
        assert_eq!((r.added, r.refused.len()), (2, 1));
    }

    /// Protects what "the world uses" means: a slot a placed icon names (matched on family, slot id and, for `custom`, the set) and a
    /// pin a point of interest uses are used; a same-named slot in another set, an unplaced slot and an unused pin are not.
    #[test]
    fn used_slots_are_those_an_icon_or_a_pin_places() {
        use cartalith_assets::manual::{IconOrigin, ManualIcon, ManualIconFamily};
        let mut a = AssetLibrarySession::new();
        let oak = a.add_custom_slot("Oak", Some("Trees"));
        let oak_other = a.add_custom_slot("Oak", Some("Elsewhere"));
        let _idle = a.add_custom_slot("Idle", Some("Trees"));
        let pin = a.pin_add("Beacon", "#ff0000", &png(1)).unwrap();
        let _unused_pin = a.pin_add("Spare", "#00ff00", &png(2)).unwrap();
        let icon = ManualIcon { x: 1.0, y: 1.0, family: ManualIconFamily::Custom, slot: "oak".into(), set: Some("Trees".into()), scale: 1.0, origin: IconOrigin::Manual };
        let kind = a.pin_catalog().into_iter().find(|p| p.uid == pin).unwrap().kind;
        let poi = Poi { id: 1, x: 1.0, y: 1.0, kind, free_label: String::new(), name: String::new() };
        let plain_poi = Poi { id: 2, x: 2.0, y: 2.0, kind: "inn".into(), free_label: String::new(), name: String::new() };
        assert_eq!(used_slots(&a, &[icon], &[poi, plain_poi]), HashSet::from([oak.clone(), pin]));
        assert!(!used_slots(&a, &[], &[]).contains(&oak) && !used_slots(&a, &[], &[]).contains(&oak_other));
    }

    /// Protects the archive leg end to end (GW's test, with the real format-3 writer and reader): device A's used art goes into a
    /// real `.zip` as `library/art/<key>.png` entries beside a `h`-tagged `library/assets.json`, and an empty device B that reads that
    /// archive ends with the same pictures; a save that embeds nothing carries no `library/art/` entry.
    #[test]
    fn a_real_archive_carries_the_art_to_an_empty_device() {
        use cartalith_io::project::{ProjectWrite, read_project, write_project};
        use cartalith_io::{SaveFields, SaveParams};
        use std::io::Cursor;
        let (mut a, used) = device_a();
        let (keys, art) = art_of(&a, &used);
        let record = tag_record(serde_json::to_value(a.db.to_library_file()).unwrap(), &keys).to_string();
        let n = 16;
        let params = SaveParams { gw: 4, gh: 4, seed: 1, map_width_km: 100.0, sea_level: 0.4, world: false, origin: None, name: None };
        let fields = SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut w = ProjectWrite::new(&params, &fields);
        w.documents.insert("library/assets.json".to_string(), record.clone());
        for (k, png) in &art {
            w.foreign.insert(format!("{ART_PREFIX}{k}.png"), png.clone());
        }
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), &w).unwrap();

        let data = read_project(Cursor::new(&buf)).unwrap();
        assert_eq!(data.format_version, 3);
        let from_zip: BTreeMap<String, Vec<u8>> =
            data.foreign.iter().filter_map(|(n, b)| Some((n.strip_prefix(ART_PREFIX)?.strip_suffix(".png")?.to_string(), b.clone()))).collect();
        assert_eq!(from_zip, art, "the three pictures came back byte for byte");
        let mut b = AssetLibrarySession::new();
        b.db.merge_library_file(&parse_library_json(data.text_of("library/assets.json").unwrap().as_bytes()).unwrap());
        assert_eq!(restore_art(&mut b, data.text_of("library/assets.json").unwrap(), &from_zip).added, 3);
        assert_eq!(b.db.total_items(), 3);

        let mut none = ProjectWrite::new(&params, &fields);
        none.documents.insert("library/assets.json".to_string(), "{}".to_string());
        let mut buf2 = Vec::new();
        write_project(Cursor::new(&mut buf2), &none).unwrap();
        assert!(read_project(Cursor::new(&buf2)).unwrap().foreign.keys().all(|k| !k.starts_with(ART_PREFIX)));
    }

    /// Protects "a save without `library/art/` opens as today": no art, no change.
    #[test]
    fn no_art_changes_nothing() {
        let mut b = AssetLibrarySession::new();
        assert_eq!(restore_art(&mut b, "{}", &BTreeMap::new()), ArtReport::default());
    }
}
