//! The move-to-slot verb (`AssetLibrarySession::move_item`, `move_targets`) and the session's `bake_filter`
//! reaching the pack report, at the session level on real PNG bytes. A child module of `asset_bridge`, so it
//! reads the session's private stores like its siblings.
//!
//! Why the verb exists: a folder import parks every `textures/` PNG that matches no ground slot's name in the
//! custom set `GROUND_CANDIDATES_SET` and, until 2026-11-17, nothing could move one into the slot it was for.

use super::*;
use cartalith_assets::{DecodedImage, encode_png};

/// A `w` by `h` opaque PNG of one colour.
fn flat(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    encode_png(&DecodedImage::new(w, h, rgba.repeat((w * h) as usize)).unwrap()).unwrap()
}

/// A session holding one ground candidate (a 40 x 40 tile) in a `Ground candidates` slot, the way a folder
/// import leaves it.
fn with_candidate() -> (AssetLibrarySession, String) {
    let mut s = AssetLibrarySession::new();
    let uid = s.add_custom_slot("moss", Some(GROUND_CANDIDATES_SET));
    s.import_item(&uid, "moss.png".to_string(), &flat(40, 40, [30, 120, 40, 255])).unwrap();
    (s, uid)
}

/// Protects: the verb itself. A candidate moves to a frozen ground slot with its name, content hash and
/// pixels, becomes that slot's newest variant, and its now-empty placeholder slot is removed so the waiting set
/// shrinks. Literals for the counts; the pixel check reads the destination's stored image, not the record.
#[test]
fn a_ground_candidate_moves_into_a_ground_slot_and_its_placeholder_goes() {
    let (mut s, uid) = with_candidate();
    let before_hash = s.db.items(&uid)[0].hash.clone();
    let before_px = s.image(&uid, 0).unwrap().clone();
    assert!(s.db.items("textures:grass").is_empty(), "the target starts empty");
    let (idx, dropped) = s.move_item(&uid, 0, "textures:grass").unwrap();
    assert_eq!((idx, dropped), (0, true));
    assert_eq!(s.db.items("textures:grass").len(), 1);
    assert_eq!(s.db.items("textures:grass")[0].name, "moss.png");
    assert_eq!(s.db.items("textures:grass")[0].hash, before_hash);
    assert_eq!(s.image("textures:grass", 0).unwrap(), &before_px, "the pixels moved with the record");
    assert!(s.db.get(&uid).is_none(), "the empty candidate slot is removed");
    assert!(s.image(&uid, 0).is_none(), "and so are its pixels");
    // A second candidate appends as the next variant of the same slot.
    let uid2 = s.add_custom_slot("fern", Some(GROUND_CANDIDATES_SET));
    s.import_item(&uid2, "fern.png".to_string(), &flat(40, 40, [10, 90, 20, 255])).unwrap();
    assert_eq!(s.move_item(&uid2, 0, "textures:grass").unwrap().0, 1);
    assert_eq!(s.db.items("textures:grass").len(), 2);
}

/// Protects: the placeholder rule is narrow. A candidate slot holding two images keeps itself after one
/// moves, and an emptied custom slot of any OTHER set is never removed (positive control for the removal
/// above: it is the set that licenses it).
#[test]
fn only_an_emptied_ground_candidates_slot_is_removed() {
    let (mut s, uid) = with_candidate();
    s.import_item(&uid, "second.png".to_string(), &flat(40, 40, [1, 2, 3, 255])).unwrap();
    let (_, dropped) = s.move_item(&uid, 0, "textures:rock").unwrap();
    assert!(!dropped && s.db.get(&uid).is_some() && s.db.items(&uid).len() == 1);
    let mut t = AssetLibrarySession::new();
    let other = t.add_custom_slot("moss", Some("My set"));
    t.import_item(&other, "m.png".to_string(), &flat(40, 40, [30, 120, 40, 255])).unwrap();
    // A custom slot outside the candidates set is a sprite slot: it cannot move into ground, so use a sprite target.
    let (_, dropped) = t.move_item(&other, 0, "icons:hill").unwrap();
    assert!(!dropped, "a slot of another set stays when emptied");
    assert!(t.db.get(&other).is_some() && t.db.items(&other).is_empty());
}

/// Protects: the refusals, each with its words and with nothing changed (a snapshot of both stores before and
/// after): ground to a sprite slot, a sprite to a ground slot, the same slot, a missing variant, a pin.
#[test]
fn refused_moves_say_why_and_change_nothing() {
    let (mut s, uid) = with_candidate();
    s.import_item("icons:hill", "h.png".to_string(), &flat(30, 30, [9, 9, 9, 255])).unwrap();
    let pin = s.pin_add("Flag", "#336699", &encode_png(&DecodedImage::new(8, 8, [200u8, 10, 10, 255].repeat(64)).unwrap()).unwrap()).unwrap();
    let snap = |s: &AssetLibrarySession| (s.db.total_items(), s.db.uids_in_order(), s.db.items(&uid).to_vec(), s.db.items("icons:hill").to_vec());
    let before = snap(&s);
    let cases: [(&str, usize, &str, &str); 6] = [
        (&uid, 0, "icons:hill", "holds sprites"),
        ("icons:hill", 0, "textures:grass", "holds ground tiles"),
        (&uid, 0, &uid, "already in that slot"),
        (&uid, 3, "textures:grass", "has no variant 4"),
        (&pin, 0, "icons:hill", "Pin section"),
        ("icons:hill", 0, &pin, "Pin section"),
    ];
    for (from, i, to, word) in cases {
        let e = s.move_item(from, i, to).unwrap_err();
        assert!(e.contains(word), "{from} -> {to}: {e}");
    }
    assert!(s.move_item(&uid, 0, "no:such").unwrap_err().contains("no such slot"));
    assert_eq!(snap(&s), before, "no refused move changed anything");
}

/// Protects: the transform rule. Within one family the authored transform is kept; across families it is
/// the destination's intake default, which for a bottom-anchored family is `fit_to_bottom`'s pan.
#[test]
fn the_transform_is_kept_within_a_family_and_reset_across_families() {
    let mut s = AssetLibrarySession::new();
    s.import_item("icons:hill", "h.png".to_string(), &flat(60, 20, [9, 9, 9, 255])).unwrap();
    assert!(s.db.item_mut("icons:hill", 0).map(|it| it.transform = ItemTransform { scale: 0.7, pan_x: 5.0, pan_y: -4.0 }).is_some());
    s.move_item("icons:hill", 0, "icons:mountain").unwrap();
    let t = &s.db.items("icons:mountain")[0].transform;
    assert_eq!((t.scale, t.pan_x, t.pan_y), (0.7, 5.0, -4.0), "same family: kept");
    // Across families: Icons -> Poi. Both sprite families; the Poi default must be what a fresh import gets.
    let mut fresh = AssetLibrarySession::new();
    fresh.import_item("poi:ruin", "h.png".to_string(), &flat(60, 20, [9, 9, 9, 255])).unwrap();
    let want = fresh.db.items("poi:ruin")[0].transform.clone();
    s.move_item("icons:mountain", 0, "poi:ruin").unwrap();
    assert_eq!(s.db.items("poi:ruin")[0].transform, want, "different family: the destination's intake default");
}

/// Protects: the target list the inspector shows. For a candidate it is exactly the ground slots (every
/// frozen texture, biome and terrain slot) and no sprite slot, not the candidate's own slot, and each label
/// names the family; for a sprite it holds no ground slot.
#[test]
fn the_target_list_is_the_compatible_slots_with_family_labels() {
    let (s, uid) = with_candidate();
    let t = s.move_targets(&uid, 0);
    let ground: usize = [Family::Textures, Family::Biomes, Family::Terrains].iter().map(|f| s.db.slots_in_family(*f).len()).sum();
    assert_eq!(t.len(), ground);
    assert!(t.iter().all(|(u, _)| u.starts_with("textures:") || u.starts_with("biomes:") || u.starts_with("terrains:")), "{t:?}");
    assert!(t.iter().any(|(u, l)| u == "textures:grass" && l == "Splat channels · Grass"), "{t:?}");
    assert!(s.move_targets(&uid, 9).is_empty());
    let mut p = AssetLibrarySession::new();
    p.import_item("icons:hill", "h.png".to_string(), &flat(30, 30, [9, 9, 9, 255])).unwrap();
    let st = p.move_targets("icons:hill", 0);
    assert!(!st.is_empty() && st.iter().all(|(u, _)| !u.starts_with("textures:") && !u.starts_with("biomes:") && !u.starts_with("terrains:") && u != "icons:hill"));
}

/// Protects: `AssetLibrarySession::bake_filter` reaches the pack report and the line-art working image. Two
/// striped drawings identical but for the RGB under their transparent texels are one near-duplicate group
/// when the session bakes premultiplied and none when it bakes straight (the default), so a session that
/// ignored its own filter in either report would keep the straight answer.
#[test]
fn the_session_bake_filter_reaches_the_pack_report() {
    let stripes = |under: [u8; 3]| {
        let mut v = Vec::with_capacity(512 * 512 * 4);
        for y in 0..512u32 {
            for x in 0..512u32 {
                let r = (f64::from(x) + 0.5 - 256.0).hypot(f64::from(y) + 0.5 - 256.0);
                if (150.0..190.0).contains(&r) && x % 2 == 0 {
                    v.extend_from_slice(&[200, 60, 60, 255]);
                } else {
                    v.extend_from_slice(&[under[0], under[1], under[2], 0]);
                }
            }
        }
        encode_png(&DecodedImage::new(512, 512, v).unwrap()).unwrap()
    };
    let near = |s: &AssetLibrarySession| s.pack_report().iter().filter(|(f, _)| f.kind == FindingKind::NearDuplicate).count();
    let mut s = AssetLibrarySession::new();
    s.import_item("icons:hill", "a.png".to_string(), &stripes([0, 0, 0])).unwrap();
    s.import_item("icons:hill", "b.png".to_string(), &stripes([0, 255, 0])).unwrap();
    assert_eq!(near(&s), 0, "the default (straight) session splits the pair");
    s.bake_filter = BakeFilter::Premultiplied;
    assert_eq!(near(&s), 1, "a premultiplying session groups it");
}

/// Protects: the session's `bake_filter` reaches the line-art working image. A soft black ring (ink with a
/// radial alpha falloff, white RGB under the transparent texels) converted at half scale gives different
/// pixels when the session bakes premultiplied than when it bakes straight: straight drags the white under
/// the soft edge into the ink and lightens it. A `line_art_variant` that drew its working image straight
/// whatever the session's filter would give the same bytes both ways.
#[test]
fn the_session_bake_filter_reaches_the_line_art_working_image() {
    let mut v = Vec::with_capacity(512 * 512 * 4);
    for y in 0..512u32 {
        for x in 0..512u32 {
            let r = (f64::from(x) + 0.5 - 256.0).hypot(f64::from(y) + 0.5 - 256.0);
            let a = ((1.0 - (r - 150.0).abs() / 9.0) * 255.0).clamp(0.0, 255.0) as u8;
            v.extend_from_slice(&if a > 0 { [10, 10, 10, a] } else { [255, 255, 255, 0] });
        }
    }
    let png = encode_png(&DecodedImage::new(512, 512, v).unwrap()).unwrap();
    let made = |filter: BakeFilter| {
        let mut s = AssetLibrarySession::new();
        s.bake_filter = filter;
        s.import_item("icons:hill", "ring.png".to_string(), &png).unwrap();
        s.db.item_mut("icons:hill", 0).unwrap().transform = ItemTransform { scale: 0.5, pan_x: 0.0, pan_y: 0.0 };
        let (_, i) = s.line_art_variant("icons:hill", 0, &LineArtOptions::default()).unwrap();
        s.image("icons:hill", i).unwrap().clone()
    };
    assert_ne!(made(BakeFilter::Premultiplied), made(BakeFilter::Straight));
    assert_eq!(made(BakeFilter::Straight), made(BakeFilter::Straight), "positive control: the conversion is deterministic");
}
