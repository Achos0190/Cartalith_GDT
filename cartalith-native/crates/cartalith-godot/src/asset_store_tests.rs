//! Ruling FR: the per-device library store, on real files under the system temp directory and real PNG bytes. A child module of
//! `asset_store`, so it reads the store's private state like its siblings. Every test names what it protects.

use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

static N: AtomicU32 = AtomicU32::new(0);

/// A fresh, empty directory for one test (removed by the test's own `Scratch` drop).
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Scratch {
        let p = std::env::temp_dir().join(format!("cartalith_store_{}_{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        Scratch(p)
    }
    fn store(&self) -> PathBuf {
        self.0.join("lib")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A `w` by `h` PNG whose pixels depend on `seed`, so two seeds never share a content key.
fn png(w: u32, h: u32, seed: u8) -> Vec<u8> {
    let mut v = vec![0u8; (w * h * 4) as usize];
    for (i, px) in v.chunks_exact_mut(4).enumerate() {
        let x = (i as u32 % w) as u8;
        let y = (i as u32 / w) as u8;
        px.copy_from_slice(&[x.wrapping_add(seed), y.wrapping_mul(3).wrapping_add(seed), seed ^ 0x55, 255]);
    }
    encode_png(&DecodedImage::new(w, h, v).unwrap()).unwrap()
}

/// A library with a bit of everything: a frozen-family item, a custom slot with tags and a transform, two pins, a collection and
/// pack info.
fn rich() -> AssetLibrarySession {
    let mut s = AssetLibrarySession::new();
    s.import_item("icons:mountain", "m.png".into(), &png(24, 24, 1)).unwrap();
    let u = s.add_custom_slot("Lighthouse", Some("Naval"));
    s.import_item(&u, "l1.png".into(), &png(16, 20, 2)).unwrap();
    s.import_item(&u, "l2.png".into(), &png(16, 20, 3)).unwrap();
    s.db.slot_meta_mut(&u).unwrap().tags = vec!["coast".into(), "tall".into()];
    s.db.slot_meta_mut(&u).unwrap().license = "CC-BY".into();
    s.pin_add("Beehive", "#c0a040", &png(40, 60, 4)).unwrap();
    s.pin_add("Forge", "#aa3322", &png(30, 30, 5)).unwrap();
    s.db.collections.add("Harbour", &[u]);
    s.db.pack.name = "My pack".into();
    s.db.pack.author = "Me".into();
    s
}

/// The record's text and every picture's bytes, in store order: what two libraries must agree on to be the same library.
fn digest(s: &mut AssetLibrarySession) -> (String, Vec<Vec<u8>>) {
    let rec = serde_json::to_string(&s.db.to_library_file()).unwrap();
    let mut px = Vec::new();
    for uid in s.db.uids_in_order() {
        for i in 0..s.db.items(&uid).len() {
            px.push(s.image(&uid, i).expect("every item has pixels").rgba.clone());
        }
    }
    (rec, px)
}

fn img_files(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir.join("img")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

/// Protects the core promise: a library written by one process is read back by another with the same record, the same pixels,
/// the same pin kind ids. Literal expectations for the counts.
#[test]
fn a_synced_library_comes_back_identical_after_a_restart() {
    let sc = Scratch::new();
    let mut a = rich();
    let (mut store, rep) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    assert!(rep.fresh && rep.items == 0);
    let r = store.sync(&mut a, 0);
    assert!(r.error.is_none() && r.index_written && !r.pending, "{r:?}");
    assert_eq!(r.written, 5, "five distinct pictures (the pins are stored normalised, so they count too)");
    let pins_before: Vec<_> = a.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    // "Restart": a brand new session and a brand new store handle on the same directory.
    let mut b = AssetLibrarySession::new();
    let (_s2, rep2) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
    assert!(rep2.skipped.is_empty(), "{:?}", rep2.skipped);
    assert_eq!((rep2.items, rep2.fresh, rep2.index_corrupt), (5, false, false));
    assert_eq!(digest(&mut a), digest(&mut b), "the record and every pixel are the same");
    let pins_after: Vec<_> = b.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    assert_eq!(pins_before, pins_after);
    assert_eq!(pins_after.len(), 2);
    assert_eq!(b.db.collections.as_map().get("Harbour").map(Vec::len), Some(1));
    assert_eq!(b.db.pack.name, "My pack");
    assert_eq!(b.db.get("custom:naval/lighthouse").unwrap().meta.tags, vec!["coast".to_string(), "tall".to_string()]);
}

/// Protects "write incrementally": an unchanged library costs no write, adding one picture writes exactly that picture (no
/// existing file is rewritten), and removing one deletes exactly its file.
#[test]
fn a_sync_writes_only_what_changed() {
    let sc = Scratch::new();
    let mut a = rich();
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    store.sync(&mut a, 0);
    let before = img_files(&sc.store());
    let stamp = |n: &str| fs::metadata(sc.store().join("img").join(n)).unwrap().modified().unwrap();
    let times: Vec<_> = before.iter().map(|n| stamp(n)).collect();
    let r = store.sync(&mut a, 0);
    assert!(r.unchanged && r.written == 0 && !r.index_written, "{r:?}");
    a.import_item("icons:mountain", "m2.png".into(), &png(24, 24, 9)).unwrap();
    let r = store.sync(&mut a, 0);
    assert_eq!((r.written, r.removed, r.index_written), (1, 0, true), "{r:?}");
    let after = img_files(&sc.store());
    assert_eq!(after.len(), before.len() + 1);
    for (n, t) in before.iter().zip(times) {
        assert_eq!(stamp(n), t, "{n} was not rewritten");
    }
    // Remove it again: its file goes, nothing else does.
    assert!(a.remove_item("icons:mountain", 1));
    let r = store.sync(&mut a, 0);
    assert_eq!((r.written, r.removed), (0, 1), "{r:?}");
    assert_eq!(img_files(&sc.store()), before);
}

/// Protects the time budget: a budget too small for the work leaves it `pending` with the index NOT written (it would name
/// pictures that are not on disk), and repeating the call finishes it to the same store an unbounded sync writes.
#[test]
fn a_small_budget_spreads_the_work_and_never_writes_an_index_ahead_of_its_pictures() {
    let sc = Scratch::new();
    let mut a = AssetLibrarySession::new();
    for i in 0..12u8 {
        a.import_item("icons:mountain", format!("m{i}.png"), &png(400, 400, i)).unwrap();
    }
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    let mut rounds = 0;
    loop {
        let r = store.sync(&mut a, 1);
        rounds += 1;
        if r.pending {
            assert!(!r.index_written, "an index written mid-way would name pictures that are not on disk");
            assert!(!sc.store().join("index.json").exists());
        } else {
            assert!(r.index_written);
            break;
        }
        assert!(rounds < 100);
    }
    assert!(rounds > 1, "1 ms cannot encode twelve 400 x 400 pictures in one call");
    let mut b = AssetLibrarySession::new();
    let (_s, rep) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
    assert_eq!((rep.items, rep.skipped.len()), (12, 0));
}

/// Protects atomicity: a successful write leaves no `.tmp`; a write whose rename cannot happen (the target is a non-empty
/// directory) fails, removes its temporary and leaves the target as it was; a stale `.tmp` from a crash is swept at open and
/// never read, and the intact index beside it loads.
#[test]
fn writes_are_atomic_and_a_stale_temporary_is_swept() {
    let sc = Scratch::new();
    let f = sc.0.join("a.bin");
    write_atomic(&f, b"one").unwrap();
    write_atomic(&f, b"two").unwrap();
    assert_eq!(fs::read(&f).unwrap(), b"two");
    assert!(!sc.0.join("a.bin.tmp").exists());
    let d = sc.0.join("dir");
    fs::create_dir_all(d.join("inner")).unwrap();
    assert!(write_atomic(&d, b"x").is_err(), "cannot replace a directory");
    assert!(d.join("inner").is_dir() && !sc.0.join("dir.tmp").exists(), "the target is untouched and no temporary is left");
    // A crash between create and rename leaves a half-written temporary beside a good index.
    let mut a = rich();
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    store.sync(&mut a, 0);
    fs::write(sc.store().join("index.json.tmp"), b"{\"store_ver").unwrap();
    fs::write(sc.store().join("img").join("deadbeefdeadbeef-1x1.png.tmp"), b"\x89PNG half").unwrap();
    let mut b = AssetLibrarySession::new();
    let (_s, rep) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
    assert!(rep.skipped.is_empty() && rep.items == 5, "{rep:?}");
    assert!(!sc.store().join("index.json.tmp").exists() && img_files(&sc.store()).iter().all(|n| !n.ends_with(".tmp")));
}

/// Protects "a corrupt or partial entry is skipped with a warning, never silently repaired": a truncated picture, a picture
/// replaced by a different valid one (name and content disagree), a missing picture and a key that tries to leave the directory
/// are each reported by name and skipped; their slots and every other picture load; the damaged files are moved aside, not
/// deleted, and a later sync does not delete them.
#[test]
fn damaged_entries_are_skipped_reported_and_kept_aside() {
    let sc = Scratch::new();
    let mut a = rich();
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    store.sync(&mut a, 0);
    let lh1 = content_key(a.image("custom:naval/lighthouse", 0).unwrap());
    let lh2 = content_key(a.image("custom:naval/lighthouse", 1).unwrap());
    let mt = content_key(a.image("icons:mountain", 0).unwrap());
    let img = |k: &str| sc.store().join("img").join(format!("{k}.png"));
    // lighthouse 1: truncated. lighthouse 2: replaced by another valid picture. mountain: deleted.
    let full = fs::read(img(&lh1)).unwrap();
    fs::write(img(&lh1), &full[..full.len() / 2]).unwrap();
    fs::write(img(&lh2), png(16, 20, 77)).unwrap();
    fs::remove_file(img(&mt)).unwrap();
    // A hostile index entry naming a path outside the store.
    let idx = fs::read_to_string(sc.store().join("index.json")).unwrap();
    assert!(idx.contains(&format!("\"h\":\"{mt}\"")));
    fs::write(sc.store().join("index.json"), idx.replace(&format!("\"h\":\"{mt}\""), "\"h\":\"../../evil\"")).unwrap();
    let mut b = AssetLibrarySession::new();
    let (mut s2, rep) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
    assert_eq!(rep.skipped.len(), 3, "{:#?}", rep.skipped);
    assert!(rep.skipped.iter().any(|l| l.contains("l1.png") && l.contains("not a readable PNG")), "{:#?}", rep.skipped);
    assert!(rep.skipped.iter().any(|l| l.contains("l2.png") && l.contains("does not hold the picture its name says")), "{:#?}", rep.skipped);
    assert!(rep.skipped.iter().any(|l| l.contains("m.png") && l.contains("not a picture key")), "{:#?}", rep.skipped);
    assert_eq!(rep.items, 2, "the two pins are intact");
    assert_eq!(b.pin_catalog().len(), 2);
    assert!(b.db.get("custom:naval/lighthouse").is_some(), "the slot and its tags survive their pictures");
    assert!(b.db.items("custom:naval/lighthouse").is_empty());
    assert!(PathBuf::from(format!("{}.corrupt", img(&lh1).display())).exists() && PathBuf::from(format!("{}.corrupt", img(&lh2).display())).exists());
    // A sync after the load keeps the aside files (they are not `.png`), and the index no longer names the skipped pictures.
    s2.sync(&mut b, 0);
    assert!(PathBuf::from(format!("{}.corrupt", img(&lh1).display())).exists());
    let mut c = AssetLibrarySession::new();
    let (_s3, rep3) = LibraryStore::open(&mut c, &sc.store(), None).unwrap();
    assert_eq!(rep3.items, 2);
}

/// Protects the corrupt-index rule: an index that is not JSON, or has another store_version, is moved aside (not deleted), the
/// library starts empty, and garbage collection stays off so the pictures it named are still on disk after a sync.
#[test]
fn an_unreadable_index_is_moved_aside_and_deletes_nothing() {
    let sc = Scratch::new();
    let mut a = rich();
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    store.sync(&mut a, 0);
    let before = img_files(&sc.store());
    for bad in [&b"not json {"[..], &br#"{"store_version":99,"slots":[]}"#[..]] {
        fs::write(sc.store().join("index.json"), bad).unwrap();
        let mut b = AssetLibrarySession::new();
        let (mut s2, rep) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
        assert!(rep.index_corrupt && rep.items == 0, "{rep:?}");
        assert_eq!(b.db.total_items(), 0);
        let aside = fs::read_dir(sc.store()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with("index.json.corrupt-")).count();
        assert!(aside >= 1, "the index was moved aside");
        b.import_item("icons:mountain", "x.png".into(), &png(8, 8, 200)).unwrap();
        s2.sync(&mut b, 0);
        for n in &before {
            assert!(sc.store().join("img").join(n).exists(), "{n} was not garbage-collected");
        }
        // Put the good index back for the next round.
        let mut fresh = rich();
        let (mut s3, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
        s3.last_fp = None;
        s3.last_index = None;
        s3.sync(&mut fresh, 0);
    }
}

/// Protects the migration: a first start with Ruling FP's `custom_pins.zip` (a real pack written by this session's own exporter)
/// and no index imports its pins with the same kind ids, writes the store at once, and renames the zip to `.migrated`; the next
/// start loads the store alone and a leftover zip is ignored.
#[test]
fn a_legacy_custom_pins_zip_is_migrated_once_into_the_store() {
    let sc = Scratch::new();
    let mut old = AssetLibrarySession::new();
    old.pin_add("Beehive", "#c0a040", &png(40, 60, 4)).unwrap();
    old.pin_add("Forge", "#aa3322", &png(30, 30, 5)).unwrap();
    let want: Vec<_> = old.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect();
    let zip = sc.0.join("custom_pins.zip");
    fs::write(&zip, old.export_pack_bytes().unwrap().1).unwrap();
    let mut a = AssetLibrarySession::new();
    let (_s, rep) = LibraryStore::open(&mut a, &sc.store(), Some(&zip)).unwrap();
    assert_eq!((rep.fresh, rep.migrated_pins), (true, 2), "{rep:?}");
    assert_eq!(a.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect::<Vec<_>>(), want);
    assert!(!zip.exists() && sc.0.join("custom_pins.zip.migrated").exists(), "retired, not deleted");
    assert!(sc.store().join("index.json").exists(), "the store was written at once");
    // Next start: the store alone. A zip that reappears (an older build) is not read again.
    fs::write(&zip, b"stale").unwrap();
    let mut b = AssetLibrarySession::new();
    let (_s, rep2) = LibraryStore::open(&mut b, &sc.store(), Some(&zip)).unwrap();
    assert_eq!((rep2.fresh, rep2.migrated_pins, rep2.items), (false, 0, 2), "{rep2:?}");
    assert_eq!(b.pin_catalog().into_iter().map(|e| (e.kind, e.name, e.color)).collect::<Vec<_>>(), want);
    // An unreadable legacy file is reported and left where it is.
    let sc2 = Scratch::new();
    let bad = sc2.0.join("custom_pins.zip");
    fs::write(&bad, b"not a zip").unwrap();
    let (_s, rep3) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc2.store(), Some(&bad)).unwrap();
    assert_eq!(rep3.migrated_pins, 0);
    assert!(rep3.skipped.len() == 1 && bad.exists());
}

/// Protects "an emptied library is saved as emptied": clearing the library and syncing leaves a store that loads empty and holds
/// no picture file (a deleted import does not come back at the next start).
#[test]
fn a_cleared_library_stays_cleared() {
    let sc = Scratch::new();
    let mut a = rich();
    let (mut store, _) = LibraryStore::open(&mut AssetLibrarySession::new(), &sc.store(), None).unwrap();
    store.sync(&mut a, 0);
    a.clear();
    let r = store.sync(&mut a, 0);
    assert!(r.index_written && r.removed == 5, "{r:?}");
    assert!(img_files(&sc.store()).is_empty());
    let mut b = AssetLibrarySession::new();
    let (_s, rep) = LibraryStore::open(&mut b, &sc.store(), None).unwrap();
    assert_eq!((rep.items, b.db.total_items()), (0, 0));
    assert!(store.size_bytes() > 0, "the (small) index is still there");
}

/// Protects the content key: it names the pixels and the shape (equal bytes, different width, give different keys), and
/// `is_valid_key` accepts exactly the shape `content_key` writes.
#[test]
fn the_content_key_names_pixels_and_shape_and_validates() {
    let a = DecodedImage::new(2, 2, vec![7u8; 16]).unwrap();
    let b = DecodedImage::new(4, 1, vec![7u8; 16]).unwrap();
    assert_ne!(content_key(&a), content_key(&b));
    assert!(is_valid_key(&content_key(&a)));
    let mut c = a.clone();
    c.rgba[15] = 8;
    assert_ne!(content_key(&a), content_key(&c));
    for bad in ["", "../x", "0123456789abcdef", "0123456789abcdeg-1x1", "0123456789abcde-1x1", "0123456789abcdef-1x", "0123456789abcdef-1x1/..", "0123456789ABCDEF-1x1"] {
        assert!(!is_valid_key(bad), "{bad}");
    }
}
