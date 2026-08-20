//! Real integration test for `ASSET_LIBRARY_SCOPE.md` milestone 7's sprite
//! compositing, run against the real fixture pack milestone 2 verified
//! against the reference's own exporter (`cartalith-assets/tests/fixtures/
//! reference_pack.zip` — reused rather than inventing a new fixture, per the
//! milestone's own instruction). Not a golden-parity test (there is no
//! reference execution path for this port's own new rendering feature to
//! diff against — `pack.rs`'s own doc comment explains why); this proves the
//! real code path actually loads a real pack and actually paints real
//! pixels, both the sprite-blit branch and the procedural-glyph-fallback
//! branch.
//!
//! `#[path]`-includes `render.rs` and `pack.rs` directly, the same technique
//! `golden_parity_render.rs` already uses, since `cartalith-godot` is
//! `cdylib`-only (`ARCHITECTURE.md`) and has no `rlib` target a normal
//! integration test could link against. This test only exercises the
//! compositing surface (`pack.rs`), not the colour renderer itself
//! (`golden_parity_render.rs` already covers that) -- `render.rs` is pulled
//! in only for the `SplatChannel`/`SplatTextures` types `pack.rs` needs,
//! hence the blanket `dead_code` allow below for everything else in it.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
#[path = "../src/pack.rs"]
mod pack;

use std::fs;

fn fixture_bytes() -> Vec<u8> {
    fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture (milestone 2) must exist")
}

#[test]
fn loads_the_real_fixture_pack_and_decodes_its_two_composited_families() {
    let loaded = pack::load_pack_from_bytes(fixture_bytes()).expect("real reference-exported pack must load");

    // `icons`: mountain (3 variants) and tree_conifer (2 variants), per the
    // fixture's own `pack.json` (milestone 2's own manifest).
    assert_eq!(loaded.icons.get("mountain").map(Vec::len), Some(3));
    assert_eq!(loaded.icons.get("tree_conifer").map(Vec::len), Some(2));
    // Every other `PACK_ICON_SLOTS` member has no art in this fixture --
    // exercised below as the procedural-glyph-fallback path.
    assert!(!loaded.icons.contains_key("shrub"));

    // `splat`: the fixture's `textures` section has grass/rock/sand only.
    assert!(loaded.splat.contains_key("grass"));
    assert!(loaded.splat.contains_key("rock"));
    assert!(loaded.splat.contains_key("sand"));
    assert!(!loaded.splat.contains_key("snow"));
    assert!(!loaded.splat.contains_key("wetland"));
    assert!(!loaded.splat.contains_key("canopy"));
}

/// A small synthetic world built for compositing, not generation --
/// deliberately not a real `WorldState` (this test is about the rendering
/// integration, not re-verifying tectonics/climate). One elevated peak (a
/// real-sprite `mountain` icon via the Relief scatter mode), a broad
/// boreal/conifer region (real-sprite `tree_conifer` icons via the Scatter
/// mode), and a broad dry-grass corner (the `shrub` preset's own biome
/// list, which this fixture pack has no art for at all -- the procedural
/// glyph fallback).
struct World {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    gw: usize,
    gh: usize,
    sea_level: f64,
}

fn build_world() -> World {
    let (gw, gh) = (24usize, 24usize);
    let sea_level = 0.30;
    let mut field = vec![0.5f32; gw * gh];
    // BOREAL/CONIFER by default (`classify_biome`: t in [0,12), m>=0.20 land
    // -- t=8, m=0.4 lands in CONIFER (3<=t<12, 0.30<=m<0.60)), matching the
    // `tree_conifer` preset's own `biomes:[3,4]`.
    let mut temperature = vec![8.0f32; gw * gh];
    let mut rainfall = vec![0.4f32; gw * gh];

    // A real elevated peak: r = (h-sea)/(1-sea) must clear the `mountain`
    // preset's `elev_min: 0.58`. h=0.95, sea=0.30 -> r ≈ 0.93.
    let (mx, my) = (12i32, 12i32);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (x, y) = ((mx + dx) as usize, (my + dy) as usize);
            field[y * gw + x] = 0.95;
        }
    }

    // A dry-grass corner (`classify_biome(8.0, 0.10)` -> GRASS, biome index
    // 7): the `shrub` preset's own `biomes:[7,8]`, and this fixture pack has
    // no `shrub` art at all -- guaranteed procedural-glyph fallback.
    for y in 0..gh {
        for x in (gw - 5)..gw {
            temperature[y * gw + x] = 8.0;
            rainfall[y * gw + x] = 0.10;
        }
    }

    World { field, temperature, rainfall, gw, gh, sea_level }
}

/// `bytes[y0..y1, x0..x1]` (RGB8) differs from an all-white background
/// somewhere in the box -- the compositing test's own "something was
/// actually painted here" predicate.
fn region_changed(bytes: &[u8], gw: usize, x0: usize, x1: usize, y0: usize, y1: usize) -> bool {
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * gw + x) * 3;
            if bytes[i] != 255 || bytes[i + 1] != 255 || bytes[i + 2] != 255 {
                return true;
            }
        }
    }
    false
}

#[test]
fn composites_real_sprites_and_falls_back_to_glyphs_on_a_real_test_pack() {
    let loaded = pack::load_pack_from_bytes(fixture_bytes()).expect("real reference-exported pack must load");
    let world = build_world();
    let mut bytes = vec![255u8; world.gw * world.gh * 3];

    pack::composite_map_icons(&mut bytes, &world.field, &world.temperature, &world.rainfall, world.gw, world.gh, world.sea_level, 7, &loaded);

    // Something was painted at all -- the placement + compositing pipeline
    // actually ran, not a silent no-op.
    assert!(bytes.iter().any(|&b| b != 255), "composite_map_icons painted nothing at all onto a real pack + real placements");

    // The mountain peak's own footprint (bottom-anchored around (12,12),
    // sprite base scale `max(3.5, gw/110)` at this tiny resolution) --
    // real sprite art should have been blitted there.
    assert!(region_changed(&bytes, world.gw, 6, 19, 6, 14), "expected the real mountain sprite to paint pixels near the peak");

    // The dry-grass corner has real `tree_conifer`-style land elsewhere on
    // the map (proving real sprites composite in general, checked above);
    // this corner specifically forces `shrub`, which the fixture has no art
    // for -- proving the procedural glyph fallback fired instead of nothing.
    assert!(region_changed(&bytes, world.gw, world.gw - 5, world.gw, 0, world.gh), "expected the procedural glyph fallback to paint the shrub-only corner");
}

#[test]
fn a_pack_with_no_icon_art_at_all_places_nothing() {
    // A pack whose manifest has icons the reference vocabulary doesn't
    // resolve (`autopopulate_scatter_rules` only binds real `pack.icons`
    // slots to a rule) is the same "no rules configured" condition
    // `cartalith_assets::current_scatter_rules`'s own doc comment names as
    // what keeps a pack-less render bit-identical -- reproduced here with a
    // pack whose `icons` map is genuinely empty rather than merely
    // undecoded, so `composite_map_icons` takes its own early-return path.
    let mut loaded = pack::load_pack_from_bytes(fixture_bytes()).expect("real reference-exported pack must load");
    // `manifest.icons` (not `loaded.icons`) is what `autopopulate_scatter_rules`
    // reads -- clear the source the rule table is actually built from.
    for slot in cartalith_assets::PACK_ICON_SLOTS {
        loaded.manifest.icons.remove(slot);
    }
    loaded.icons.clear();

    let world = build_world();
    let mut bytes = vec![255u8; world.gw * world.gh * 3];
    pack::composite_map_icons(&mut bytes, &world.field, &world.temperature, &world.rainfall, world.gw, world.gh, world.sea_level, 7, &loaded);

    assert!(bytes.iter().all(|&b| b == 255), "no scatter rules configured must place nothing -- the pack-less-equivalent no-op");
}
