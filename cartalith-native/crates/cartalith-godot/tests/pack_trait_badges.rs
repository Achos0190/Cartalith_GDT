//! `OUTSTANDING_WORK.md` §2.5, sprite half: compositing an imported pack's own
//! **trait** art (`structures.trait`), the family `pack.rs` parsed and never
//! drew.
//!
//! Same `#[path]`-include technique as `pack_compositing.rs` (this crate is
//! `cdylib`-only, so an integration test cannot link against it).
//!
//! **Two packs, for two different jobs, and the split is deliberate.** The
//! real fixture — `cartalith-assets/tests/fixtures/reference_pack.zip`, the
//! one milestone 2 verified against the reference's own exporter — proves the
//! real load path decodes `structures.trait` and paints from it. The
//! *exact-pixel* assertions run against a purpose-built pack whose art is a
//! single flat colour, because the fixture's `port_01.png` is **not** uniform
//! (measured: 65 536 texels in exactly two alphas, 32 768 at `255` and 32 768
//! at `128`), so a bilinear sample of it is a position-dependent mix and there
//! is no constant to assert. A flat source makes every sample identical and
//! the destination value pure arithmetic.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
#[path = "../src/pack.rs"]
mod pack;

use cartalith_assets::{DecodedImage, encode_png, trait_badge_layout, trait_badge_radius, zip_store_bytes};
use pack::{TraitArtMiss, composite_trait_badges, load_pack_from_bytes, resolve_trait_badges};
use std::fs;

fn fixture_bytes() -> Vec<u8> {
    fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture (milestone 2) must exist")
}

/// A real `.zip` pack declaring one `structures.trait` slot whose only variant
/// is a flat `rgba` image — written with the crate's own writer and read back
/// through the real `read_pack`, so nothing about the load path is stubbed.
fn solid_trait_pack(slot: &str, rgba: [u8; 4]) -> Vec<u8> {
    let (w, h) = (16u32, 16u32);
    let img = DecodedImage::new(w, h, rgba.iter().copied().cycle().take((w * h * 4) as usize).collect()).expect("flat image");
    let png = encode_png(&img).expect("encode");
    let manifest = format!(
        r#"{{"schema":2,"name":"flat","structures":{{"trait":{{"{slot}":["structures/trait/{slot}_01.png"]}}}}}}"#
    );
    let path = format!("structures/trait/{slot}_01.png");
    zip_store_bytes(&[("pack.json", manifest.as_bytes()), (path.as_str(), png.as_slice())]).expect("zip_store_bytes must write a readable archive")
}

/// A pack whose manifest declares trait art that cannot be decoded — the
/// `ArtFailedToDecode` case, which no real fixture can express (a fixture with
/// a broken PNG in it is a broken fixture).
fn pack_with_undecodable_trait_art() -> Vec<u8> {
    let manifest = br#"{"schema":2,"name":"broken","structures":{"trait":{"mining":["structures/trait/mining_01.png"]}}}"#;
    zip_store_bytes(&[("pack.json", manifest.as_slice()), ("structures/trait/mining_01.png", b"not a png at all".as_slice())])
        .expect("zip_store_bytes must write a readable archive")
}

fn px(bytes: &[u8], gw: usize, x: usize, y: usize) -> [u8; 3] {
    let i = (y * gw + x) * 3;
    [bytes[i], bytes[i + 1], bytes[i + 2]]
}

const WHITE: [u8; 3] = [255, 255, 255];

/// A pin at (32, 20), `sz = 10`, `sc = 2`. `trait_badge_radius(10) =
/// max(2.2, 4.2) = 4.2`, so a lone badge sits at `(32, 20 + 10 + 4.2 + 1.2*2)
/// = (32, 36.6)` and its square sprite spans `4.2*2 = 8.4` px: x in
/// [27.8, 36.2), y in [32.4, 40.8).
const PIN: (f64, f64, f64, f64) = (32.0, 20.0, 10.0, 2.0);
/// Four points comfortably inside that box, one per quadrant.
const INSIDE: [(usize, usize); 4] = [(29, 34), (35, 34), (29, 39), (35, 39)];
/// Four points comfortably outside it, one past each edge.
const OUTSIDE: [(usize, usize); 4] = [(26, 36), (38, 36), (32, 30), (32, 43)];

fn keys(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_real_fixture_pack_decodes_its_trait_family() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("real reference-exported pack must load");
    // One slot, one variant — the fixture's own `pack.json`.
    assert_eq!(loaded.traits.get("port").map(Vec::len), Some(1));
    let img = &loaded.traits["port"][0];
    assert_eq!((img.w, img.h), (256, 256));
    // Every other `PACK_TRAIT_SLOTS` member is absent, not present-and-empty:
    // the pack never declared them. That difference is the whole point of the
    // map's shape.
    assert!(!loaded.traits.contains_key("mining"));
    assert!(!loaded.traits.contains_key("fortified"));
}

/// The real pack, the real load path, real pixels — non-uniform art, so the
/// claim is confined to what is true of it: the badge box is painted from the
/// pack and everything outside the box is untouched.
#[test]
fn the_real_fixture_packs_trait_art_paints_inside_the_badge_box_and_nowhere_else() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("real reference-exported pack must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &keys(&["port"]), PIN.2, PIN.3, 7, &loaded);

    assert!(misses.is_empty(), "the fixture pack HAS port art; nothing should have missed");
    for (x, y) in INSIDE {
        assert_ne!(px(&bytes, gw, x, y), WHITE, "({x},{y}) is inside the sprite box and must be painted");
    }
    for (x, y) in OUTSIDE {
        assert_eq!(px(&bytes, gw, x, y), WHITE, "({x},{y}) is outside the sprite box and must not have been painted");
    }
    // The colour really came from the pack: `port_01.png` is a solid blue
    // (60, 90, 150) at two alphas, so every painted texel must be bluer than
    // it is red whatever the alpha mix, which the white ground is not.
    let c = px(&bytes, gw, 32, 36);
    assert!(c[2] > c[0], "painted pixel {c:?} must carry the pack sprite's blue, not the white ground");
}

/// The exact-pixel drawing test. A fully opaque flat sprite must land its own
/// bytes, unchanged, at exactly the box `trait_badge_layout` +
/// `trait_sprite_rect` describe — and nowhere else.
#[test]
fn an_opaque_pack_sprite_lands_its_exact_bytes_at_the_ported_geometry() {
    let loaded = load_pack_from_bytes(solid_trait_pack("port", [200, 40, 90, 255])).expect("flat pack must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    let traits = keys(&["port"]);
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &loaded);
    assert!(misses.is_empty());

    // The geometry is `trait_badge_layout`'s, not this test's guess.
    let badge = &trait_badge_layout(PIN.0, PIN.1, &traits, PIN.2, PIN.3)[0];
    assert!((badge.r - trait_badge_radius(PIN.2)).abs() < 1e-12);
    assert!((badge.cy - 36.6).abs() < 1e-12, "{}", badge.cy);
    assert_eq!(badge.cx, PIN.0, "a lone badge is centred on the pin");

    assert_eq!(px(&bytes, gw, badge.cx.round() as usize, badge.cy.round() as usize), [200, 40, 90], "badge centre must carry the pack's own colour");
    for (x, y) in INSIDE {
        assert_eq!(px(&bytes, gw, x, y), [200, 40, 90], "({x},{y}) is inside the sprite box");
    }
    for (x, y) in OUTSIDE {
        assert_eq!(px(&bytes, gw, x, y), WHITE, "({x},{y}) is outside the sprite box");
    }
}

/// The same, half-transparent: `Canvas::blend_px`' straight-alpha source-over,
/// `round(src*a + 255*(1-a))` with `a = 128/255 = 0.501961` —
///   r: 60*0.501961 + 255*0.498039 = 30.118 + 127.0 = 157.118 -> 157
///   g: 90*0.501961 + 127.0 = 172.176 -> 172
///   b: 150*0.501961 + 127.0 = 202.294 -> 202
/// Worked by hand from the blend, not read off a run.
#[test]
fn a_half_transparent_pack_sprite_blends_against_what_is_under_it() {
    let loaded = load_pack_from_bytes(solid_trait_pack("port", [60, 90, 150, 128])).expect("flat pack must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    assert!(composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &keys(&["port"]), PIN.2, PIN.3, 7, &loaded).is_empty());
    for (x, y) in INSIDE {
        assert_eq!(px(&bytes, gw, x, y), [157, 172, 202], "({x},{y})");
    }
    for (x, y) in OUTSIDE {
        assert_eq!(px(&bytes, gw, x, y), WHITE, "({x},{y})");
    }
}

/// Four badges, one of which the pack has art for: the pack-art one is
/// painted, the three misses are reported, and **nothing is painted where a
/// miss is**. A dark disc drawn there would be a plausible-looking badge that
/// cannot say which trait it is — the reference's fallback is that disc *plus*
/// the trait's text glyph, and text is what this rasterizer cannot draw.
#[test]
fn a_slot_with_no_art_paints_nothing_and_is_reported_with_its_glyph() {
    let loaded = load_pack_from_bytes(solid_trait_pack("port", [200, 40, 90, 255])).expect("flat pack must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];

    let traits = keys(&["mining", "port", "military", "religious"]);
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &loaded);

    assert_eq!(misses.iter().map(|m| m.key.as_str()).collect::<Vec<_>>(), ["mining", "military", "religious"]);
    for m in &misses {
        assert_eq!(m.miss, TraitArtMiss::NoArtInPack);
        // The glyph the caller must draw, from CIV_TRAITS — present, so the
        // fallback can say which trait it is.
        assert!(m.glyph.is_some_and(|g| !g.is_empty()), "{} is a real CIV_TRAITS key", m.key);
        assert_eq!(
            px(&bytes, gw, m.cx.round() as usize, m.cy.round() as usize),
            WHITE,
            "a miss must paint nothing, not a blank disc"
        );
    }

    // ...while the one slot that does have art was painted, in the same call.
    let badges = trait_badge_layout(PIN.0, PIN.1, &traits, PIN.2, PIN.3);
    let port = badges.iter().find(|b| b.key == "port").expect("port is among the shown four");
    assert_eq!(px(&bytes, gw, port.cx.round() as usize, port.cy.round() as usize), [200, 40, 90]);
}

/// Declared-but-undecodable is a **different** reason from never-declared,
/// and both paint nothing. Collapsing them would tell a pack author with a
/// broken PNG that they simply have no art.
#[test]
fn declared_trait_art_that_fails_to_decode_is_a_distinct_reason() {
    let loaded = load_pack_from_bytes(pack_with_undecodable_trait_art()).expect("a pack with one broken PNG must still load");
    // The slot IS in the map — declared — and holds no variants.
    assert_eq!(loaded.traits.get("mining").map(Vec::len), Some(0));

    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    let traits = keys(&["mining"]);
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &loaded);

    assert_eq!(misses.len(), 1);
    assert_eq!(misses[0].miss, TraitArtMiss::ArtFailedToDecode);
    assert!(bytes.iter().all(|&b| b == 255), "an undecodable sprite must paint nothing at all");

    // And the same key against a pack that never mentioned it reports the
    // other reason — the two are genuinely distinguishable, for the same
    // trait, from the same call shape.
    let fixture = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let mut b2 = vec![255u8; gw * gh * 3];
    let other = composite_trait_badges(&mut b2, gw, gh, PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &fixture);
    assert_eq!(other[0].miss, TraitArtMiss::NoArtInPack);
}

/// A trait key that is not in `CIV_TRAITS` carries no glyph, because the
/// reference draws **nothing at all** for it — its fallback sits inside
/// `const t=CIV_TRAITS.find(...); if(t){...}` (v2.11 line 15592). A caller
/// must skip such a badge rather than draw a blank disc for it.
#[test]
fn an_unknown_trait_key_reports_no_glyph_because_the_reference_draws_none() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &keys(&["not_a_real_trait"]), PIN.2, PIN.3, 7, &loaded);
    assert_eq!(misses.len(), 1);
    assert!(misses[0].glyph.is_none());
    assert!(bytes.iter().all(|&b| b == 255));
}

/// The reference's `slice(0,4)` cap is the layout's, and the compositor must
/// not re-derive it: a settlement carrying all seven traits produces at most
/// four badges, art or miss.
#[test]
fn the_compositor_honours_the_layouts_four_badge_cap() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let (gw, gh) = (64usize, 64usize);
    let mut bytes = vec![255u8; gw * gh * 3];
    let traits: Vec<String> = cartalith_assets::PACK_TRAIT_SLOTS.iter().map(|s| s.to_string()).collect();
    assert_eq!(traits.len(), 7);
    let misses = composite_trait_badges(&mut bytes, gw, gh, PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &loaded);
    // Four shown — `PACK_TRAIT_SLOTS`' first four are fortified, mining, port,
    // administrative — so exactly one of them has art and three miss.
    assert_eq!(misses.iter().map(|m| m.key.as_str()).collect::<Vec<_>>(), ["fortified", "mining", "administrative"]);
}

/// An empty grid is a no-op rather than a panic — the same guard
/// `composite_map_icons` carries.
#[test]
fn an_empty_grid_paints_nothing_and_reports_nothing() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let mut bytes: Vec<u8> = Vec::new();
    assert!(composite_trait_badges(&mut bytes, 0, 0, 0.0, 0.0, &keys(&["port"]), PIN.2, PIN.3, 7, &loaded).is_empty());
}

// ---------------------------------------------------------------------------
// `resolve_trait_badges` — the decisions, without the raster
// ---------------------------------------------------------------------------

/// The two miss reasons cross the gdext boundary as strings, so the strings
/// are the contract. Asserted as **literals**, not against the constants that
/// produce them: `assert_eq!(x, THE_CONSTANT)` holds for every value of the
/// constant (`MISTAKES.md`), and a rename here silently breaks
/// `map_overlay.gd` and every readout keyed on these names.
#[test]
fn the_two_miss_reasons_have_stable_names() {
    assert_eq!(TraitArtMiss::NoArtInPack.key(), "no_art_in_pack");
    assert_eq!(TraitArtMiss::ArtFailedToDecode.key(), "art_failed_to_decode");
    assert_ne!(TraitArtMiss::NoArtInPack.key(), TraitArtMiss::ArtFailedToDecode.key());
}

/// The resolver answers with the pack's own variant and the reference's own
/// destination box, and `composite_trait_badges` paints exactly that — which
/// is the whole reason the two share one function. Pins the box against
/// arithmetic written out here rather than against `trait_sprite_rect`
/// itself: `port_01.png` is 256x256, so `dw = dh = r*2` and the box is
/// centred on the badge.
#[test]
fn resolve_reports_the_packs_variant_at_the_references_centre_anchored_box() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let badges = resolve_trait_badges(PIN.0, PIN.1, &keys(&["port", "mining"]), PIN.2, PIN.3, 7, &loaded);
    assert_eq!(badges.len(), 2);

    let r = trait_badge_radius(PIN.2);
    assert_eq!(r, 4.2, "sz=10 -> max(2.2, 4.2)");
    match badges[0].art {
        pack::TraitBadgeArt::Sprite { variant, rect } => {
            assert_eq!(variant, 0, "the fixture declares exactly one `port` variant");
            assert_eq!((rect.dw, rect.dh), (r * 2.0, r * 2.0));
            assert_eq!(rect.dx + rect.dw / 2.0, badges[0].cx);
            assert_eq!(rect.dy + rect.dh / 2.0, badges[0].cy);
        }
        pack::TraitBadgeArt::Miss(m) => panic!("the fixture HAS port art; got {m:?}"),
    }
    assert_eq!(badges[1].art, pack::TraitBadgeArt::Miss(TraitArtMiss::NoArtInPack));
    assert_eq!(badges[1].glyph, Some("⚒"), "a miss still carries the glyph its fallback draws");
}

/// The resolver's own layout is `trait_badge_layout`'s and is not re-derived,
/// checked against the reference's three expressions written out
/// independently (`_civDrawTraitBadges`, v2.11 lines 15586-15588).
#[test]
fn resolve_lays_the_row_out_at_the_references_own_geometry() {
    let loaded = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let badges = resolve_trait_badges(PIN.0, PIN.1, &keys(&["port", "mining", "military"]), PIN.2, PIN.3, 7, &loaded);
    assert_eq!(badges.len(), 3);
    let r = 4.2_f64; // max(2.2, sz*0.42) at sz = 10
    assert_eq!(badges[0].r, r);
    assert_eq!(badges[1].cx - badges[0].cx, r * 2.35);
    assert_eq!((badges[0].cx + badges[2].cx) / 2.0, PIN.0, "centred on the pin");
    assert_eq!(badges[0].cy, PIN.1 + PIN.2 + r + 1.2 * PIN.3);
    // No traits is an empty row, not a row of zero-radius badges.
    assert!(resolve_trait_badges(PIN.0, PIN.1, &[], PIN.2, PIN.3, 7, &loaded).is_empty());
}

/// Declared-but-undecodable reaches the resolver as its own reason, so the
/// `#[func]` that hands GDScript a badge row can report it per badge without
/// re-deriving anything.
#[test]
fn resolve_keeps_the_undecodable_reason_apart_from_the_never_declared_one() {
    let broken = load_pack_from_bytes(pack_with_undecodable_trait_art()).expect("broken pack must load");
    let fixture = load_pack_from_bytes(fixture_bytes()).expect("fixture must load");
    let traits = keys(&["mining"]);
    assert_eq!(
        resolve_trait_badges(PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &broken)[0].art,
        pack::TraitBadgeArt::Miss(TraitArtMiss::ArtFailedToDecode)
    );
    assert_eq!(
        resolve_trait_badges(PIN.0, PIN.1, &traits, PIN.2, PIN.3, 7, &fixture)[0].art,
        pack::TraitBadgeArt::Miss(TraitArtMiss::NoArtInPack)
    );
}
