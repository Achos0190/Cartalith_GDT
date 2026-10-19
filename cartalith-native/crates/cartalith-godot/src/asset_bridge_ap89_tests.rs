//! AP-8 (folder import) and AP-9 (pack diff and merge) at the session level:
//! real PNG bytes, real folders made for each test under the system temp
//! directory (no third-party art), and real pack archives. A child module of
//! `asset_bridge`, so it reads the session's private stores the way that
//! file's own tests do.

use super::*;
use cartalith_assets::{DecodedImage, GROUND_CANDIDATES_SET, SlotChange, encode_png};
use std::fs;
use std::path::{Path, PathBuf};

/// A `w` by `h` PNG of one colour.
fn png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        v.extend_from_slice(&rgba);
    }
    encode_png(&DecodedImage::new(w, h, v).unwrap()).unwrap()
}

/// A fresh folder under the temp dir, removed on drop. `name` is the
/// chosen folder's own name, so a test can assert the set it gives.
struct Tree {
    base: PathBuf,
    root: PathBuf,
}

impl Tree {
    fn new(tag: &str, name: &str) -> Tree {
        let base = std::env::temp_dir().join(format!("cartalith_ap89_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let root = base.join(name);
        fs::create_dir_all(&root).unwrap();
        Tree { base, root }
    }
    fn file(&self, rel: &str, bytes: &[u8]) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    fn path(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

/// Every custom slot as `(set, slot name, item count, tags)`, in add order,
/// asserted non-empty so an import that silently filled nothing cannot pass.
fn custom_sets(s: &AssetLibrarySession) -> Vec<(String, String, usize, Vec<String>)> {
    let out: Vec<_> = s
        .db
        .slots_in_family(Family::Custom)
        .into_iter()
        .map(|sl| (sl.set.clone().unwrap_or_default(), sl.name.clone(), s.db.items(&sl.uid).len(), sl.meta.tags.clone()))
        .collect();
    assert!(!out.is_empty(), "no custom slot was created");
    out
}

/// Protects: AP-8's loose-folder bar. Images directly in the chosen folder
/// form a set named after it, each subfolder its own set; numbered files are
/// variants of one slot; items are named after their files; the licence
/// text lands in every filled slot's `SlotMeta::license`; a text file is not
/// read; the library's pack licence is untouched.
#[test]
fn a_loose_folder_imports_into_custom_sets_with_its_licence() {
    let t = Tree::new("loose", "Loose Art");
    t.file("castle.png", &png(8, 8, [200, 10, 10, 255]));
    t.file("tower_01.png", &png(8, 8, [10, 200, 10, 255]));
    t.file("tower_02.png", &png(8, 8, [10, 10, 200, 255]));
    t.file("trees/oak_1.png", &png(8, 8, [1, 2, 3, 255]));
    t.file("trees/oak_2.png", &png(8, 8, [4, 5, 6, 255]));
    t.file("trees/pine.png", &png(8, 8, [7, 8, 9, 255]));
    t.file("notes.txt", b"not an image");
    t.file("LICENSE.txt", b"  CC BY 4.0, test author  \n");
    let mut s = AssetLibrarySession::new();
    let r = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(r.layout, FolderLayout::Loose);
    assert_eq!(r.folder, "Loose Art");
    assert_eq!(r.licence.as_ref().map(|l| l.text.as_str()), Some("CC BY 4.0, test author"));
    assert_eq!(
        custom_sets(&s),
        vec![
            ("Loose Art".into(), "castle".into(), 1, vec![]),
            ("Loose Art".into(), "tower".into(), 2, vec![]),
            ("trees".into(), "oak".into(), 2, vec![]),
            ("trees".into(), "pine".into(), 1, vec![]),
        ]
    );
    assert_eq!(r.outcome.count(PackRowStatus::Imported), 6);
    assert_eq!(r.outcome.count(PackRowStatus::Refused), 0);
    let tower = s.db.slots_in_family(Family::Custom)[1].uid.clone();
    let names: Vec<&str> = s.db.items(&tower).iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, vec!["tower_01", "tower_02"]);
    for sl in s.db.slots_in_family(Family::Custom) {
        assert_eq!(sl.meta.license, "CC BY 4.0, test author", "{}", sl.uid);
        assert!(sl.meta.source.starts_with("Loose Art ("), "{}", sl.meta.source);
    }
    assert_eq!(s.db.pack.license, "", "a folder is not a pack: the pack licence is untouched");
    assert!(r.outcome.notes.iter().any(|n| n.contains("1 file that is not a PNG image was not read")), "{:?}", r.outcome.notes);
}

/// Protects: AP-8's Wonderdraft bar on a synthetic package folder: sprite
/// folders below `sprites/` become sets named by their path, `textures/`
/// images become ground candidates (a slot-named one fills that slot, the
/// rest wait in the Ground candidates set), images elsewhere are left out
/// with a note, and `.wonderdraft_symbols` is not parsed.
#[test]
fn a_wonderdraft_folder_maps_sprites_to_sets_and_textures_to_ground() {
    let t = Tree::new("wd", "My Package");
    t.file("sprites/symbols/castles/keep_01.png", &png(8, 8, [9, 9, 9, 255]));
    t.file("sprites/symbols/castles/keep_02.png", &png(8, 8, [19, 9, 9, 255]));
    t.file("sprites/symbols/castles/.wonderdraft_symbols", br#"{"keep_01":{"radius":0.5}}"#);
    t.file("sprites/trees/conifer/fir.png", &png(8, 8, [29, 9, 9, 255]));
    t.file("textures/grass.png", &png(16, 16, [10, 120, 30, 255]));
    t.file("textures/cobbles.png", &png(16, 16, [90, 90, 90, 255]));
    t.file("brushes/soft.png", &png(8, 8, [0, 0, 0, 255]));
    let mut s = AssetLibrarySession::new();
    let r = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(r.layout, FolderLayout::Wonderdraft);
    assert_eq!(
        custom_sets(&s),
        vec![
            ("symbols / castles".into(), "keep".into(), 2, vec![]),
            ("trees / conifer".into(), "fir".into(), 1, vec![]),
            (GROUND_CANDIDATES_SET.into(), "cobbles".into(), 1, vec![]),
        ]
    );
    assert_eq!(s.db.items("textures:grass").len(), 1, "a slot-named ground image fills that slot");
    assert_eq!(r.outcome.count(PackRowStatus::Imported), 5);
    assert!(r.outcome.notes.iter().any(|n| n.starts_with("1 image was outside sprites/")), "{:?}", r.outcome.notes);
    assert!(r.outcome.notes.iter().any(|n| n.contains(".wonderdraft_symbols")), "{:?}", r.outcome.notes);
    assert!(r.licence.is_none());
}

/// Protects: AP-8's Dungeondraft bar on a synthetic unpacked tree: objects
/// become sets, terrain becomes ground candidates, the tag file's tags land
/// on the slots holding their paths, and the other texture folders and the
/// preview are left out with a note.
#[test]
fn an_unpacked_dungeondraft_tree_maps_objects_terrain_and_tags() {
    let t = Tree::new("dd", "Dungeon Bits");
    t.file("textures/objects/furniture/chair_1.png", &png(8, 8, [1, 1, 1, 255]));
    t.file("textures/objects/furniture/chair_2.png", &png(8, 8, [2, 1, 1, 255]));
    t.file("textures/objects/barrel.png", &png(8, 8, [3, 1, 1, 255]));
    t.file("textures/terrain/dirt.png", &png(16, 16, [120, 90, 60, 255]));
    t.file("textures/terrain/moss_stone.png", &png(16, 16, [60, 90, 60, 255]));
    t.file("textures/walls/brick.png", &png(8, 8, [4, 1, 1, 255]));
    t.file("preview.png", &png(8, 10, [5, 1, 1, 255]));
    t.file(
        "data/default.dungeondraft_tags",
        br#"{"tags":{"Furniture":["textures/objects/furniture/chair_1.png"],
                     "Outdoor":["textures/objects/barrel.png","textures/terrain/dirt.png"]},
             "sets":{"All":["Furniture","Outdoor"]}}"#,
    );
    let mut s = AssetLibrarySession::new();
    let r = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(r.layout, FolderLayout::Dungeondraft);
    assert_eq!(
        custom_sets(&s),
        vec![
            ("Dungeon Bits".into(), "barrel".into(), 1, vec!["Outdoor".to_string()]),
            ("furniture".into(), "chair".into(), 2, vec!["Furniture".to_string()]),
            (GROUND_CANDIDATES_SET.into(), "moss_stone".into(), 1, vec![]),
        ]
    );
    assert_eq!(s.db.items("terrains:dirt").len(), 1);
    assert_eq!(s.db.get("terrains:dirt").unwrap().meta.tags, vec!["Outdoor".to_string()]);
    assert_eq!(r.outcome.count(PackRowStatus::Imported), 5);
    assert!(r.outcome.notes.iter().any(|n| n.starts_with("2 images were outside textures/objects/")), "{:?}", r.outcome.notes);
    assert!(r.outcome.notes.iter().any(|n| n.contains("\"sets\"")), "{:?}", r.outcome.notes);
}

/// Protects: importing the same folder twice never doubles its art. The dry
/// run names the conflicts, keep skips them, replace empties and refills.
#[test]
fn a_folder_imported_twice_conflicts_until_replaced() {
    let t = Tree::new("twice", "Twice");
    t.file("a_1.png", &png(8, 8, [1, 1, 1, 255]));
    t.file("a_2.png", &png(8, 8, [2, 2, 2, 255]));
    let mut s = AssetLibrarySession::new();
    s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    let dry = s.import_folder(&t.path(), PackImportMode::DryRun).unwrap();
    assert_eq!(dry.outcome.conflicts.len(), 1, "{:?}", dry.outcome.conflicts);
    assert!(dry.outcome.rows.is_empty(), "a dry run writes no rows");
    let keep = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(keep.outcome.count(PackRowStatus::Skipped), 2);
    s.import_folder(&t.path(), PackImportMode::Replace).unwrap();
    assert_eq!(s.db.total_items(), 2);
}

/// Protects: an image that is not a PNG inside, or whose header is over the
/// cap, is a refused row with a reason, and the rest of the folder imports.
#[test]
fn a_bad_image_refuses_only_its_own_row() {
    let t = Tree::new("bad", "Bad");
    t.file("good.png", &png(8, 8, [1, 1, 1, 255]));
    t.file("fake.png", b"GIF89a this is not a png at all");
    let mut s = AssetLibrarySession::new();
    let r = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(r.outcome.count(PackRowStatus::Imported), 1);
    let refused: Vec<&PackImportRow> = r.outcome.rows.iter().filter(|x| x.status == PackRowStatus::Refused).collect();
    assert_eq!(refused.len(), 1);
    assert!(refused[0].reason.contains("not a PNG"), "{}", refused[0].reason);
}

/// Protects: "nothing outside the chosen folder is read" at the session
/// level: a link inside the folder to a folder of images outside it imports
/// none of them. When this machine can make no link at all, says so and
/// checks only that the outside images are absent.
#[test]
fn a_link_out_of_the_folder_imports_nothing_from_outside() {
    let outside = Tree::new("outside", "Elsewhere");
    outside.file("secret.png", &png(8, 8, [255, 0, 255, 255]));
    let t = Tree::new("linked", "Linked");
    t.file("inside.png", &png(8, 8, [1, 1, 1, 255]));
    let link = t.root.join("escape");
    let made = make_dir_link(&outside.root, &link);
    let mut s = AssetLibrarySession::new();
    let r = s.import_folder(&t.path(), PackImportMode::KeepExisting).unwrap();
    assert_eq!(s.db.total_items(), 1, "only the image inside the folder");
    assert!(r.outcome.rows.iter().all(|x| !x.path.contains("secret")));
    if made {
        assert!(r.outcome.notes.iter().any(|n| n.starts_with("skipped escape:")), "{:?}", r.outcome.notes);
    } else {
        eprintln!("no directory link could be made on this machine; the link branch was not exercised");
    }
}

/// A directory link (symlink, else a Windows junction). False if neither.
fn make_dir_link(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return true;
        }
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Protects: a path that is not a folder is an error in words, not a panic.
#[test]
fn a_missing_folder_is_an_error() {
    let mut s = AssetLibrarySession::new();
    let e = s.import_folder("Z:/no/such/folder/ap8", PackImportMode::KeepExisting).unwrap_err();
    assert!(e.contains("not a readable folder"), "{e}");
}

// -- AP-9 ----------------------------------------------------------------

/// Four sprites in three slots and a custom slot: the base library both
/// merge tests start from.
fn base_session() -> AssetLibrarySession {
    let mut s = AssetLibrarySession::new();
    s.db.pack.name = "Base".into();
    s.db.pack.license = "MIT".into();
    s.import_item("icons:hill", "a".into(), &png(32, 32, [10, 0, 0, 255])).unwrap();
    s.import_item("icons:hill", "b".into(), &png(32, 32, [20, 0, 0, 255])).unwrap();
    s.import_item("poi:ruin", "c".into(), &png(32, 32, [30, 0, 0, 255])).unwrap();
    let u = s.add_custom_slot("Windmill", Some("Farm"));
    s.import_item(&u, "d".into(), &png(32, 32, [40, 0, 0, 255])).unwrap();
    s
}

/// The base library's pack, opened back into a fresh library: B, the
/// library every merge test merges into. Opened rather than built, so its
/// items are the pack's own bakes and a pack exported from it diffs empty.
fn library_b() -> (AssetLibrarySession, Vec<u8>) {
    let bytes = base_session().export_pack_bytes().unwrap().1;
    let mut b = AssetLibrarySession::new();
    b.import_pack(&bytes, "b.zip", PackImportMode::Replace).unwrap();
    (b, bytes)
}

/// Pack A: B's pack with hill's second image changed, ruin's image changed,
/// author set, and (when `extra`) a sea mark B does not fill.
fn pack_a(extra: bool) -> Vec<u8> {
    let mut s = base_session();
    s.remove_item("icons:hill", 1);
    s.import_item("icons:hill", "b2".into(), &png(32, 32, [99, 0, 0, 255])).unwrap();
    s.remove_item("poi:ruin", 0);
    s.import_item("poi:ruin", "c2".into(), &png(32, 32, [77, 0, 0, 255])).unwrap();
    s.db.pack.author = "Other".into();
    if extra {
        s.import_item("seamarks:buoy", "e".into(), &png(32, 32, [5, 5, 5, 255])).unwrap();
    }
    s.export_pack_bytes().unwrap().1
}

/// Protects: AP-9's first bar. A pack diffed against a library opened from
/// itself is empty, for the reference fixture and the synthetic base pack;
/// and the base pack against an empty library is every slot "only in the
/// pack" (the diff is not vacuously empty).
#[test]
fn a_pack_diffed_against_itself_is_empty() {
    let refp = super::tests::reference_pack();
    let mut lib = AssetLibrarySession::new();
    lib.import_pack(&refp, "r.zip", PackImportMode::Replace).unwrap();
    let (d, refused) = lib.diff_pack(&refp, "r.zip").unwrap();
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(refused, 0);
    let (b, bytes) = library_b();
    assert!(b.diff_pack(&bytes, "b.zip").unwrap().0.is_empty());
    let (d, _) = AssetLibrarySession::new().diff_pack(&bytes, "b.zip").unwrap();
    assert_eq!(d.slots.len(), 3);
    assert!(d.slots.iter().all(|x| x.change == SlotChange::OnlyA));
}

/// Protects: AP-9's merge bar. Merging A into B with "keep library" where A
/// adds no slot leaves B byte-identical: its full snapshot and its exported
/// pack bytes. With A adding a slot, every slot B already filled is
/// byte-identical and only the new slot is added.
#[test]
fn keep_library_leaves_the_library_byte_identical() {
    let (mut b, _) = library_b();
    let before = super::tests::snapshot(&b);
    let before_bytes = b.export_pack_bytes().unwrap().1;
    let a = pack_a(false);
    let (d, _) = b.diff_pack(&a, "a.zip").unwrap();
    assert_eq!(d.slots.len(), 2, "the fixture really differs: {d:?}");
    let out = b.merge_pack(&a, "a.zip", MergePolicy::KeepB).unwrap();
    assert_eq!((out.added, out.replaced, out.appended, out.kept, out.items), (0, 0, 0, 2, 0));
    assert!(out.fields.is_empty());
    assert_eq!(super::tests::snapshot(&b), before);
    assert_eq!(b.export_pack_bytes().unwrap().1, before_bytes);

    let a2 = pack_a(true);
    b.merge_pack(&a2, "a2.zip", MergePolicy::KeepB).unwrap();
    let (info, after) = super::tests::snapshot(&b);
    assert_eq!(info, before.0);
    let kept: Vec<_> = after.iter().filter(|r| r.0 != "seamarks:buoy").cloned().collect();
    assert_eq!(kept, before.1, "every slot B filled is untouched");
    assert_eq!(b.db.items("seamarks:buoy").len(), 1, "the slot only A fills is added");
}

/// Protects: "take pack" makes every slot A fills equal to A's (the diff
/// afterwards has no differing or pack-only slot) and takes A's author;
/// "keep both" appends only the images B does not already hold.
#[test]
fn take_pack_and_keep_both_do_what_they_say() {
    let a = pack_a(true);
    let (mut b, _) = library_b();
    let out = b.merge_pack(&a, "a.zip", MergePolicy::TakeA).unwrap();
    assert_eq!((out.added, out.replaced), (1, 2));
    assert_eq!(out.fields, vec!["author".to_string()]);
    let (d, _) = b.diff_pack(&a, "a.zip").unwrap();
    assert!(d.is_empty(), "after taking the pack it diffs empty: {d:?}");
    assert_eq!(b.db.pack.author, "Other");

    let (mut b, _) = library_b();
    let out = b.merge_pack(&a, "a.zip", MergePolicy::KeepBoth).unwrap();
    // hill: one new image appended (its first is shared); ruin: one new
    // image appended; buoy added.
    assert_eq!((out.added, out.appended, out.items), (1, 2, 3));
    assert_eq!(b.db.items("icons:hill").len(), 3);
    assert_eq!(b.db.items("poi:ruin").len(), 2);
    // B's author is the exporter's default for a blank one, so it is filled
    // and keep both keeps it (the blank-fill branch is `pack_diff`'s test).
    assert_eq!(b.db.pack.author, "Cartalith Gen1");
    assert_eq!(b.db.pack.name, "Base");
}
