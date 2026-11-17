//! Ruling FP at the session level: the Pins category of the library (add, edit, catalog), the normalised icon, and the pack
//! round trip, on real PNG bytes. A child module of `asset_bridge`, so it reads the session's private stores like its siblings.

use super::*;
use cartalith_assets::{DecodedImage, encode_png};

/// A `w` by `h` PNG whose opaque part is a `bw` by `bh` block at `(bx, by)` of colour `rgba`, the rest transparent.
fn art(w: u32, h: u32, bx: u32, by: u32, bw: u32, bh: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut v = vec![0u8; (w * h * 4) as usize];
    for y in by..by + bh {
        for x in bx..bx + bw {
            let i = ((y * w + x) * 4) as usize;
            v[i..i + 4].copy_from_slice(&rgba);
        }
    }
    encode_png(&DecodedImage::new(w, h, v).unwrap()).unwrap()
}

fn red() -> Vec<u8> {
    art(40, 60, 5, 8, 20, 40, [200, 30, 30, 255])
}

/// Protects: the normalised icon is 128 x 128, bottom-aligned and centred, fills its longer side, and is **idempotent**
/// (a pack carries the normalised picture and must re-import to the same kind id). Literals for the geometry.
#[test]
fn the_icon_is_normalised_bottom_aligned_and_idempotent() {
    let img = decode_png(&red()).unwrap();
    let n = normalise_pin_icon(&img).unwrap();
    assert_eq!((n.w, n.h), (128, 128));
    // The block was 20 x 40; the longer side (40) scales to 128, so the body is 64 x 128, centred: columns 32..96.
    let alpha = |x: u32, y: u32| n.rgba[((y * 128 + x) * 4 + 3) as usize];
    assert_eq!(alpha(64, 127), 255, "bottom-centre pixel is opaque (the tip)");
    assert_eq!(alpha(64, 0), 255, "the top row is the body");
    assert_eq!(alpha(10, 64), 0, "left of the body is clear");
    assert_eq!(alpha(120, 64), 0);
    assert_eq!(normalise_pin_icon(&n).unwrap(), n, "normalising twice changes nothing");
    // A wide, short picture is bottom-aligned, not centred vertically.
    let wide = normalise_pin_icon(&decode_png(&art(64, 64, 0, 0, 64, 16, [0, 200, 0, 255])).unwrap()).unwrap();
    let a = |x: u32, y: u32| wide.rgba[((y * 128 + x) * 4 + 3) as usize];
    assert_eq!((a(64, 127), a(64, 96), a(64, 90)), (255, 255, 0), "a 4:1 picture sits in the bottom 32 rows");
    assert!(normalise_pin_icon(&decode_png(&art(8, 8, 0, 0, 0, 0, [0, 0, 0, 0])).unwrap()).is_err(), "fully transparent is refused");
}

/// Protects: add, catalog, update. A pin is a slot of the Pins set; its kind id has the engine's shape; the id changes with
/// the name, the colour and the picture and with nothing else; a taken name and bad input are refused with nothing created.
#[test]
fn a_pin_is_added_listed_and_edited_and_its_id_follows_its_content() {
    let mut s = AssetLibrarySession::new();
    let uid = s.pin_add("Beehive", "#C0A040", &red()).unwrap();
    assert_eq!(uid, "custom:pins/beehive");
    let cat = s.pin_catalog();
    assert_eq!(cat.len(), 1);
    let e = &cat[0];
    assert_eq!((e.name.as_str(), e.color.as_str(), e.slot_id.as_str()), ("Beehive", "#c0a040", "beehive"));
    assert!(cartalith_civ::poi::is_custom_kind_id(&e.kind), "{}", e.kind);
    assert!(e.kind.starts_with("custom-beehive-"));
    assert_eq!(s.pin_catalog()[0].kind, e.kind, "catalog is deterministic");
    let before = e.kind.clone();
    // Recolour: same slot, new id.
    let uid = s.pin_update(&uid, "Beehive", "#2040c0").unwrap();
    assert_eq!(uid, "custom:pins/beehive");
    let after = s.pin_catalog()[0].kind.clone();
    assert_ne!(after, before, "a new colour is a new id, so placed pins keep the old copy");
    // Rename: the uid follows, the picture follows.
    let uid2 = s.pin_update(&uid, "Wild hive", "#2040c0").unwrap();
    assert_eq!(uid2, "custom:pins/wild_hive");
    assert!(s.image(&uid2, 0).is_some() && s.image(&uid, 0).is_none(), "the pixel store moved with the uid");
    assert!(s.pin_catalog()[0].kind.starts_with("custom-wild_hive-"));
    // Refusals: nothing created.
    assert!(s.pin_add("Wild hive", "#000000", &red()).unwrap_err().contains("already exists"));
    assert!(s.pin_add("   ", "#000000", &red()).is_err());
    assert!(s.pin_add("Bad colour", "red", &red()).is_err());
    assert!(s.pin_add("Not a png", "#000000", b"GIF89a").is_err());
    assert!(s.pin_add("Blank", "#000000", &art(8, 8, 0, 0, 0, 0, [0, 0, 0, 0])).is_err());
    assert_eq!(s.db.slots_in_family(Family::Custom).len(), 1, "no slot was created by a refusal");
    let second = s.pin_add("Forge", "#000000", &red()).unwrap();
    assert!(s.pin_update(&second, "Wild hive", "#000000").unwrap_err().contains("already exists"), "a rename onto a taken name is refused");
    assert_eq!(s.pin_update("icons:mountain", "x", "#000000").unwrap_err(), "icons:mountain is not a pin");
}

/// Protects: a pin is a Pins-set slot and a custom icon of another set is not offered as a pin.
#[test]
fn only_the_pins_set_is_a_pin() {
    let mut s = AssetLibrarySession::new();
    let u = s.add_custom_slot("Lighthouse", Some("Naval"));
    s.import_item(&u, "l.png".into(), &red()).unwrap();
    assert!(s.pin_catalog().is_empty());
}

/// Protects the round trip (Ruling FP item 2): export carries the pin as its normalised icon plus `pins` name and colour;
/// a fresh library imports it with the same name, colour, slot and **the same kind id**; the manifest carries the new key
/// only when a pin exists.
#[test]
fn a_pack_round_trip_keeps_name_colour_and_kind_id() {
    let mut a = AssetLibrarySession::new();
    a.pin_add("Beehive", "#c0a040", &red()).unwrap();
    a.pin_add("Forge", "#aa3322", &art(30, 30, 2, 2, 26, 26, [10, 120, 200, 255])).unwrap();
    let want: Vec<_> = a.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    let (_, bytes) = a.export_pack_bytes().unwrap();
    let entries = archive::read_pack_entries(std::io::Cursor::new(&bytes)).unwrap();
    let manifest = String::from_utf8(entries["pack.json"].clone()).unwrap();
    assert!(manifest.contains("\"pins\"") && manifest.contains("\"#c0a040\"") && manifest.contains("\"Beehive\""), "{manifest}");
    let mut b = AssetLibrarySession::new();
    let out = b.import_pack(&bytes, "pins.zip", PackImportMode::KeepExisting).unwrap();
    assert_eq!(out.count(PackRowStatus::Imported), 2, "{:?}", out.rows);
    let got: Vec<_> = b.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    assert_eq!(got, want, "same ids, names and colours after the trip");
    // A library with no pin writes no `pins` key.
    let mut c = AssetLibrarySession::new();
    let u = c.add_custom_slot("Lighthouse", Some("Naval"));
    c.import_item(&u, "l.png".into(), &red()).unwrap();
    let (_, plain) = c.export_pack_bytes().unwrap();
    let m = String::from_utf8(archive::read_pack_entries(std::io::Cursor::new(&plain)).unwrap()["pack.json"].clone()).unwrap();
    assert!(!m.contains("pins"), "{m}");
}

/// Protects the hostile-pack rules: a `pins` entry with a bad colour or an empty name or no art leaves the art an ordinary
/// custom icon (named by its id), never a repaired pin; a good entry beside them still imports.
#[test]
fn a_pack_with_bad_pin_entries_is_refused_per_entry_not_repaired() {
    let png = art(20, 20, 0, 0, 20, 20, [1, 2, 3, 255]);
    let z = archive::zip_store_bytes(&[
        ("a.png", &png),
        ("b.png", &png),
        ("c.png", &png),
        (
            "pack.json",
            br##"{"custom":{"Pins":{"good":["a.png"],"badcolour":["b.png"],"noname":["c.png"]}},
                "pins":{"good":{"name":"Good one","color":"#102030"},"badcolour":{"name":"X","color":"red"},"noname":{"name":"  ","color":"#102030"},"ghost":{"name":"G","color":"#102030"}}}"##,
        ),
    ])
    .unwrap();
    let mut s = AssetLibrarySession::new();
    s.import_pack(&z, "x.zip", PackImportMode::KeepExisting).unwrap();
    let cat = s.pin_catalog();
    let by_slot = |id: &str| cat.iter().find(|e| e.slot_id == id).map(|e| (e.name.clone(), e.color.clone()));
    assert_eq!(by_slot("good_one"), Some(("Good one".into(), "#102030".into())));
    // The refused entries keep the id as the name and the neutral grey: art with no valid pin data is still shown, plainly.
    assert_eq!(by_slot("badcolour"), Some(("badcolour".into(), "#8a8f98".into())));
    assert_eq!(by_slot("noname"), Some(("noname".into(), "#8a8f98".into())));
    assert!(cat.iter().all(|e| e.slot_id != "ghost" && e.slot_id != "g"));
}

/// Protects Ruling FR at the session level: opening a world MERGES its pixel-less record into the library (superseding Ruling FP's
/// snapshot, clear, apply, restore of the pins). Nothing the library holds is replaced or dropped: both pins and the Naval
/// picture keep their ids, colours and pixels even where the record names the same pin slot with another colour; a slot only the
/// record names is created empty; the record's collection is unioned in.
#[test]
fn a_worlds_record_merges_into_the_library_and_drops_nothing() {
    let mut s = AssetLibrarySession::new();
    s.pin_add("Beehive", "#c0a040", &red()).unwrap();
    s.pin_add("Forge", "#aa3322", &art(30, 30, 2, 2, 26, 26, [10, 120, 200, 255])).unwrap();
    let u = s.add_custom_slot("Lighthouse", Some("Naval"));
    s.import_item(&u, "l.png".into(), &red()).unwrap();
    let want: Vec<_> = s.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    let items_before = s.db.total_items();
    // The record a world carries: no pixels; ONE pin slot under another colour, a slot this library lacks, a collection.
    let mut record_lib = AssetLibrarySession::new();
    record_lib.pin_add("Beehive", "#000001", &red()).unwrap();
    let other = record_lib.add_custom_slot("Windmill", Some("Rural"));
    record_lib.import_item(&other, "w.png".into(), &red()).unwrap();
    record_lib.db.collections.add("Favourites of that world", &[other.clone()]);
    let record = record_lib.db.to_library_json().expect("a library with a picture has a record");
    let created = s.db.merge_library_file(&record);
    assert_eq!(created, 1, "only the Windmill slot was new");
    assert!(s.db.get(&other).is_some(), "the record's own slot exists, empty");
    assert!(s.db.items(&other).is_empty());
    assert_eq!(s.db.total_items(), items_before, "no item was added or dropped");
    let got: Vec<_> = s.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    assert_eq!(got, want, "ids, names and colours survive, the record's #000001 did not recolour Beehive");
    assert!(s.image(&u, 0).is_some(), "the Naval picture's pixels are still there");
    assert!(s.db.collections.as_map().get("Favourites of that world").is_some(), "collections are unioned in");
}
