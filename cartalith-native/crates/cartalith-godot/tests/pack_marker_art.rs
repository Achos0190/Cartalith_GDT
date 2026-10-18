//! AP-3 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ Q2): the engine half of
//! drawing a pack's settlement, POI, sea-mark and custom art on placed map
//! icons — decoding it into `LoadedPack::markers`, resolving one icon to a
//! variant (`resolve_marker_sprite`), and pre-filtering it to the size it is
//! drawn at (`marker_raster_for`, `area_downscale`).
//!
//! Same `#[path]`-include technique as `pack_trait_badges.rs` (this crate is
//! `cdylib`-only, so an integration test cannot link against it). The Godot
//! half — `WorldGen::icon_marker_art` and `map_overlay.gd`'s draw — needs a
//! running engine and is `_markerart_probe.tscn`'s job.
//!
//! The reference pack fixture is the real-load-path check; purpose-built
//! packs (written with the crate's own `zip_store_bytes`) carry the cases the
//! fixture cannot: sea marks, a broken PNG, a non-square sprite.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
/// The pack loader and marker resolver under test, compiled straight from `src/pack.rs`.
#[path = "../src/pack.rs"]
mod pack;

use cartalith_assets::{DecodedImage, encode_png, pick_icon_variant, zip_store_bytes};
use pack::{area_downscale, load_pack_from_bytes, marker_art_key, marker_raster_for, resolve_marker_sprite};

/// Bytes of `cartalith-assets/tests/fixtures/reference_pack.zip`.
fn fixture_bytes() -> Vec<u8> {
    std::fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture must exist")
}

/// A flat `w`×`h` image of `rgba`.
fn flat(w: u32, h: u32, rgba: [u8; 4]) -> DecodedImage {
    DecodedImage::new(w, h, rgba.iter().copied().cycle().take((w * h * 4) as usize).collect()).expect("flat image")
}

/// A real `.zip` with `manifest` and each `(path, image)` PNG-encoded.
fn pack_zip(manifest: &str, images: &[(&str, DecodedImage)]) -> Vec<u8> {
    let pngs: Vec<(String, Vec<u8>)> = images.iter().map(|(p, i)| (p.to_string(), encode_png(i).expect("encode"))).collect();
    let mut entries: Vec<(&str, &[u8])> = vec![("pack.json", manifest.as_bytes())];
    for (p, b) in &pngs {
        entries.push((p.as_str(), b.as_slice()));
    }
    zip_store_bytes(&entries).expect("zip_store_bytes must write a readable archive")
}

/// Protects: the reference pack's settlement, POI and custom art is decoded
/// into exactly the four marker keys its manifest declares, with the variant
/// counts it declares, and nothing else (its trait and icon art go to their
/// own tables; it has no sea marks).
#[test]
fn the_reference_pack_decodes_exactly_its_four_marker_slots() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture loads");
    let mut keys: Vec<(String, usize)> = loaded.markers.iter().map(|(k, v)| (k.replace('\u{1f}', "|"), v.len())).collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            ("custom|Naval|lighthouse".to_string(), 2),
            ("poi|ruin".to_string(), 1),
            ("settlement|city".to_string(), 2),
            ("settlement|village".to_string(), 1),
        ]
    );
}

/// Protects: the four families resolve and `feature` (AP-5's) and anything
/// unknown never do; an empty slot and a non-finite coordinate give `None`.
#[test]
fn only_the_four_families_with_art_resolve() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture loads");
    assert!(resolve_marker_sprite("settlement", "city", "", 10.0, 10.0, 7, &loaded).is_some());
    assert!(resolve_marker_sprite("poi", "ruin", "", 10.0, 10.0, 7, &loaded).is_some());
    assert!(resolve_marker_sprite("custom", "lighthouse", "Naval", 10.0, 10.0, 7, &loaded).is_some());
    // `custom` is two-level: the right slot in the wrong set has no art.
    assert!(resolve_marker_sprite("custom", "lighthouse", "naval", 10.0, 10.0, 7, &loaded).is_none());
    // `feature` art is the pack's `icons` family (mountain is filled) and
    // is NOT served here.
    assert!(resolve_marker_sprite("feature", "mountain", "", 10.0, 10.0, 7, &loaded).is_none());
    assert!(resolve_marker_sprite("settlement", "town", "", 10.0, 10.0, 7, &loaded).is_none(), "an empty slot");
    assert!(resolve_marker_sprite("bogus", "city", "", 10.0, 10.0, 7, &loaded).is_none());
    assert!(resolve_marker_sprite("settlement", "city", "", f64::NAN, 10.0, 7, &loaded).is_none());
    assert_eq!(marker_art_key("feature", "mountain", ""), None);
    // The separator keeps a `/` in a set name from colliding.
    assert_ne!(marker_art_key("custom", "c", "a/b"), marker_art_key("custom", "b/c", "a"));
}

/// Protects: the variant hash is the reference's fit-view one,
/// `pickIconVariant((x+0.5)|0, (y+0.5)|0, seed, n)` (`civToScreen` with no
/// view rect), not the bare `x|0` -- found by searching for a point where the
/// two disagree, so dropping the `+ 0.5` turns this red -- and it varies over
/// the map (both of `city`'s variants are drawn somewhere).
#[test]
fn the_variant_is_the_reference_fit_view_hash() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture loads");
    let seed = 1234;
    let mut found = None;
    'outer: for gy in 0..64 {
        for gx in 0..64 {
            let (x, y) = (gx as f64 + 0.7, gy as f64 + 0.7);
            let with_half = pick_icon_variant((x + 0.5) as i32, (y + 0.5) as i32, seed, 2);
            let without = pick_icon_variant(x as i32, y as i32, seed, 2);
            if with_half != without {
                found = Some((x, y, with_half));
                break 'outer;
            }
        }
    }
    let (x, y, want) = found.expect("some point in 64x64 distinguishes the two hashes");
    assert_eq!(resolve_marker_sprite("settlement", "city", "", x, y, seed, &loaded).unwrap().variant, want);
    let mut seen = [false; 2];
    for gx in 0..200 {
        seen[resolve_marker_sprite("settlement", "city", "", gx as f64, 3.0, seed, &loaded).unwrap().variant] = true;
    }
    assert_eq!(seen, [true, true], "both variants of a two-variant slot are drawn somewhere");
}

/// Protects: sea marks (a port-only section) decode and resolve, a slot whose
/// only PNG is broken is absent rather than present-and-empty, and the aspect
/// is the variant's own `w / max(1, h)`.
#[test]
fn sea_marks_broken_art_and_aspect() {
    let manifest = r#"{"schema":2,"name":"t",
        "seamarks":{"buoy":["seamarks/buoy_01.png"],"reef":["seamarks/reef_01.png"]},
        "structures":{"poi":{"cave":["structures/poi/cave_01.png"]}}}"#;
    let mut entries = vec![("seamarks/buoy_01.png", flat(32, 16, [200, 30, 30, 255])), ("structures/poi/cave_01.png", flat(16, 16, [1, 2, 3, 255]))];
    let mut bytes_entries: Vec<(&str, Vec<u8>)> =
        entries.drain(..).map(|(p, i)| (p, encode_png(&i).expect("encode"))).collect();
    bytes_entries.push(("seamarks/reef_01.png", b"not a png".to_vec()));
    let mut zipped: Vec<(&str, &[u8])> = vec![("pack.json", manifest.as_bytes())];
    for (p, b) in &bytes_entries {
        zipped.push((p, b.as_slice()));
    }
    let loaded = load_pack_from_bytes(zip_store_bytes(&zipped).expect("zip")).expect("pack loads");
    let buoy = resolve_marker_sprite("seamarks", "buoy", "", 5.0, 5.0, 0, &loaded).expect("sea mark art resolves");
    assert_eq!(buoy.aspect, 2.0);
    assert_eq!(resolve_marker_sprite("poi", "cave", "", 5.0, 5.0, 0, &loaded).unwrap().aspect, 1.0);
    assert!(resolve_marker_sprite("seamarks", "reef", "", 5.0, 5.0, 0, &loaded).is_none());
    assert!(!loaded.markers.contains_key(&marker_art_key("seamarks", "reef", "").unwrap()));
}

/// Protects: the raster ladder's choice as literals: the smallest bucket at
/// or above the on-screen size, capped at the source's long side, and the
/// smallest bucket for a degenerate size.
#[test]
fn the_raster_is_the_smallest_bucket_at_or_above_screen_size() {
    assert_eq!(marker_raster_for(11.0, 256), 12);
    assert_eq!(marker_raster_for(12.0, 256), 12);
    assert_eq!(marker_raster_for(12.1, 256), 16);
    assert_eq!(marker_raster_for(97.0, 256), 128);
    assert_eq!(marker_raster_for(300.0, 256), 256);
    assert_eq!(marker_raster_for(100.0, 64), 64, "never above the source");
    assert_eq!(marker_raster_for(0.0, 256), 12);
    assert_eq!(marker_raster_for(f64::NAN, 256), 12);
    assert_eq!(marker_raster_for(-5.0, 256), 12);
}

/// Protects: the area filter keeps a thin line's ink mass (the reason it is
/// used instead of one bilinear sample), keeps a flat colour exact, and is a
/// clone when asked not to shrink.
#[test]
fn area_downscale_keeps_ink_mass_and_flat_colour() {
    // A 2 px black line on transparent, 256 -> 16 (a 16x shrink).
    let mut rgba = vec![0u8; 256 * 256 * 4];
    for y in 0..256 {
        for x in 100..102 {
            rgba[(y * 256 + x) * 4 + 3] = 255;
        }
    }
    let line = DecodedImage::new(256, 256, rgba).unwrap();
    let small = area_downscale(&line, 16);
    assert_eq!((small.w, small.h), (16, 16));
    let mass_in: f64 = line.rgba.chunks(4).map(|p| f64::from(p[3])).sum();
    let mass_out: f64 = small.rgba.chunks(4).map(|p| f64::from(p[3])).sum::<f64>() * 256.0;
    assert!((mass_out / mass_in - 1.0).abs() < 0.02, "ink mass ratio {}", mass_out / mass_in);
    // Column 6 holds source columns 96..112, so the line is 2/16 of it.
    assert_eq!(small.rgba[(3 * 16 + 6) * 4 + 3], 32, "255 * 2/16 = 31.9, rounded");
    let f = area_downscale(&flat(256, 256, [10, 200, 30, 255]), 24);
    assert!(f.rgba.chunks(4).all(|p| p == [10, 200, 30, 255]));
    assert_eq!(area_downscale(&line, 256), line);
    assert_eq!(area_downscale(&line, 400), line);
    // Non-square: the short side scales with the long one.
    let wide = area_downscale(&flat(256, 128, [1, 1, 1, 255]), 64);
    assert_eq!((wide.w, wide.h), (64, 32));
}

/// Protects: the filter is premultiplied -- a transparent texel's RGB (here
/// pure green, as some exporters leave it) must not bleed into an edge pixel
/// that mixes opaque red with it. A straight-alpha average gives that edge
/// pixel G = 127.
#[test]
fn area_downscale_does_not_bleed_transparent_rgb() {
    let img = DecodedImage::new(4, 1, vec![255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0]).unwrap();
    let small = area_downscale(&img, 2);
    assert_eq!(&small.rgba[0..4], &[255, 0, 0, 255]);
    let edge = area_downscale(&DecodedImage::new(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 0]).unwrap(), 1);
    assert_eq!(edge.rgba, vec![255, 0, 0, 128], "red keeps its hue at half alpha; no green");
}
