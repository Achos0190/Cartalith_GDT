//! AP-5 (`ASSET_PACK_TOOLS_PROPOSAL.md`, Ruling CQ Q3): the scattered feature
//! icons' split into decisions (`pack::place_feature_icons`) and drawing
//! (`pack::composite_placed_features`), the overlay switch's baked half, and
//! the two GPU texture paths (`pack::feature_texture`: AP-10's distance field
//! for one-ink art, a premultiplied area-filtered mip chain otherwise).
//!
//! What the screen does with these is measured by the windowed probe
//! `godot-project/_ap5_probe.gd`; this file pins the engine half.
//!
//! Same `#[path]`-include technique as `pack_compositing.rs` (this crate is a
//! cdylib).

#[path = "../src/render.rs"]
mod render;
/// The sprite-pack loader and compositor under test.
#[path = "../src/pack.rs"]
mod pack;

use cartalith_assets::{DecodedImage, sprite_draw_rect};
use pack::{FeatureArt, ONE_INK_TOLERANCE};

/// FNV-1a 64 over a byte buffer: the pin for a whole composited raster.
fn fnv1a(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in b {
        h ^= u64::from(*x);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The milestone-2 reference-exported pack (3 mountain and 2 tree_conifer
/// variants; no other icon art).
fn fixture_pack() -> pack::LoadedPack {
    let bytes = std::fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture must exist");
    pack::load_pack_from_bytes(bytes).expect("the reference pack loads")
}

/// A synthetic `n x n` world with relief and a climate gradient, so the
/// mountain and conifer rules both place many icons. Fixture values, not
/// measured data.
fn ridged_world(n: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut f = vec![0f32; n * n];
    let mut t = vec![0f32; n * n];
    let mut r = vec![0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 / n as f32, y as f32 / n as f32);
            f[y * n + x] = 0.45 + 0.5 * ((fx * 9.0).sin() * (fy * 7.0).cos()).abs();
            t[y * n + x] = 2.0 + 20.0 * fy;
            r[y * n + x] = 0.05 + 0.8 * fx;
        }
    }
    (f, t, r)
}

/// `pack_compositing.rs`'s 24 x 24 world: a peak (mountain sprites),
/// conifer land (tree sprites) and a dry-grass corner (the `shrub` preset's
/// biomes). With [`pack_with_undecodable_shrub`] the corner's shrubs draw the
/// procedural glyph fallback.
fn glyph_world() -> (Vec<f32>, Vec<f32>, Vec<f32>, usize) {
    let n = 24usize;
    let mut field = vec![0.5f32; n * n];
    let mut temperature = vec![8.0f32; n * n];
    let mut rainfall = vec![0.4f32; n * n];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            field[((12 + dy) as usize) * n + (12 + dx) as usize] = 0.95;
        }
    }
    for y in 0..n {
        for x in (n - 5)..n {
            temperature[y * n + x] = 8.0;
            rainfall[y * n + x] = 0.10;
        }
    }
    (field, temperature, rainfall, n)
}

/// Protects: the baked path's bytes did not move when AP-5 split
/// `composite_map_icons` in two. The hashes were produced by HEAD's
/// pre-split `pack.rs` (`88d9f4a2`), included beside the split one and
/// asserted byte-equal on these exact inputs before being pinned here
/// (2026-10-19), so they are the old implementation's output, not this one's.
#[test]
fn the_baked_path_is_byte_identical_to_the_pre_split_compositor() {
    let loaded = fixture_pack();
    let (f, t, r) = ridged_world(96);
    for (seed, want) in [(7, 0xebf7_be7b_e755_3292u64), (483_920, 0x3050_003b_6a3c_e9d4u64)] {
        let mut bytes = vec![200u8; 96 * 96 * 3];
        pack::composite_map_icons(&mut bytes, &f, &t, &r, 96, 96, 0.42, seed, &loaded);
        assert_eq!(fnv1a(&bytes), want, "seed {seed}: the baked feature icons moved");
    }
}

/// Protects: `place_feature_icons` is the baked path's own decisions: painter's
/// order by `y`, each sprite's box exactly `sprite_draw_rect` at the
/// reference's `base = max(gw / 110, 3.5)` for that variant's size, and a
/// glyph wherever the slot has no art.
#[test]
fn placements_are_y_sorted_and_carry_the_baked_box() {
    let loaded = fixture_pack();
    let (f, t, r) = ridged_world(96);
    let placed = pack::place_feature_icons(&f, &t, &r, 96, 96, 0.42, 7, &loaded);
    assert!(placed.len() > 100, "the ridged world places many icons ({})", placed.len());
    assert!(placed.windows(2).all(|w| w[0].item.y <= w[1].item.y), "painter's order is by y");
    let base = (96.0f64 / 110.0).max(3.5);
    assert_eq!(pack::feature_icon_base(96), 3.5, "a 96-wide grid's base is the 3.5 floor");
    assert!((pack::feature_icon_base(2048) - 2048.0 / 110.0).abs() < 1e-12);
    for p in &placed {
        let FeatureArt::Sprite { variant, rect } = p.art else { panic!("the fixture has art for every slot this world places") };
        let img = &loaded.icons[&p.slot][variant];
        let want = sprite_draw_rect(p.item.x as f64, p.item.y as f64, p.item.s, base, img.w as f64, img.h as f64);
        assert_eq!(rect, want);
    }
}

/// The fixture pack plus a `shrub` slot whose one variant never decoded: a
/// glyph fallback needs a scatter rule (`autopopulate_scatter_rules` reads
/// `manifest.icons`) and no decoded art (`LoadedPack::icons`). Measured
/// 2026-10-19: with the plain fixture the glyph world places 29 sprites and
/// 0 glyphs, because a slot with no manifest entry gets no rule at all.
fn pack_with_undecodable_shrub() -> pack::LoadedPack {
    let mut loaded = fixture_pack();
    loaded.manifest.icons.insert("shrub", vec!["icons/shrub_missing.png".to_string()]);
    loaded
}

/// Protects: the overlay switch's baked half. With `sprites: false` the
/// sprites leave the raster untouched and only glyph fallbacks are drawn;
/// with `true` it is the whole baked composite.
#[test]
fn the_overlay_switch_bakes_only_glyph_fallbacks() {
    let loaded = pack_with_undecodable_shrub();
    let (f, t, r, n) = glyph_world();
    let placed = pack::place_feature_icons(&f, &t, &r, n, n, 0.30, 7, &loaded);
    let glyphs = placed.iter().filter(|p| p.art == FeatureArt::Glyph).count();
    let sprites = placed.len() - glyphs;
    assert!(glyphs > 0 && sprites > 0, "the world has both kinds ({sprites} sprites, {glyphs} glyphs)");

    let mut all = vec![255u8; n * n * 3];
    pack::composite_map_icons(&mut all, &f, &t, &r, n, n, 0.30, 7, &loaded);
    let mut both = vec![255u8; n * n * 3];
    pack::composite_placed_features(&mut both, n, n, &placed, &loaded, true, pack::SpriteFilter::Bilinear);
    assert_eq!(all, both, "sprites: true is the baked composite");

    let mut glyph_only = vec![255u8; n * n * 3];
    pack::composite_placed_features(&mut glyph_only, n, n, &placed, &loaded, false, pack::SpriteFilter::Bilinear);
    assert_ne!(glyph_only, vec![255u8; n * n * 3], "the glyph fallbacks are still baked");
    // Only glyphs: the same as compositing a list with the sprites removed.
    let only: Vec<_> = placed.iter().filter(|p| p.art == FeatureArt::Glyph).cloned().collect();
    let mut want = vec![255u8; n * n * 3];
    pack::composite_placed_features(&mut want, n, n, &only, &loaded, true, pack::SpriteFilter::Bilinear);
    assert_eq!(glyph_only, want, "sprites: false draws exactly the glyphs");

    // A world of sprites only (the plain fixture: no rule without art):
    // switch on bakes nothing at all.
    let plain = fixture_pack();
    let (f2, t2, r2) = ridged_world(96);
    let p2 = pack::place_feature_icons(&f2, &t2, &r2, 96, 96, 0.42, 7, &plain);
    assert!(!p2.is_empty() && p2.iter().all(|p| p.art != FeatureArt::Glyph));
    let mut b2 = vec![123u8; 96 * 96 * 3];
    pack::composite_placed_features(&mut b2, 96, 96, &p2, &plain, false, pack::SpriteFilter::Bilinear);
    assert!(b2.iter().all(|v| *v == 123), "no glyph, so the switch leaves the raster untouched");
}

/// A `w x h` image filled by `px(x, y) -> [r, g, b, a]`.
fn image(w: u32, h: u32, px: impl Fn(u32, u32) -> [u8; 4]) -> DecodedImage {
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            rgba.extend_from_slice(&px(x, y));
        }
    }
    DecodedImage::new(w, h, rgba).expect("w*h*4 bytes")
}

/// A ring of ink `[200, 40, 90]` on transparency: outer radius 100, inner 84
/// (a 16-texel stroke), anti-aliased alpha from 4 x 4 supersampling, black
/// RGB under alpha 0 -- AP-10's output shape. `nudge` adds that many levels
/// to the red of one soft-edge pixel.
fn ring(nudge: u8) -> DecodedImage {
    let mut img = image(256, 256, |x, y| {
        let mut n = 0u32;
        for sy in 0..4 {
            for sx in 0..4 {
                let dx = f64::from(x) + (f64::from(sx) + 0.5) / 4.0 - 128.0;
                let dy = f64::from(y) + (f64::from(sy) + 0.5) / 4.0 - 128.0;
                let r = (dx * dx + dy * dy).sqrt();
                if (84.0..=100.0).contains(&r) {
                    n += 1;
                }
            }
        }
        let a = (n * 255 / 16) as u8;
        if a == 0 { [0, 0, 0, 0] } else { [200, 40, 90, a] }
    });
    if nudge > 0 {
        let i = img.rgba.chunks_exact(4).position(|p| p[3] > 0 && p[3] < 255).expect("a soft edge pixel");
        img.rgba[i * 4] += nudge;
    }
    img
}

/// Protects: `one_ink_colour` is the line-art rule AP-5 finds AP-10 variants
/// by: one RGB wherever alpha > 0, within `ONE_INK_TOLERANCE` (2 levels: the
/// bake's measured 1-level drift plus one), alpha-0 pixels ignored.
#[test]
fn one_ink_detection_follows_its_tolerance() {
    assert_eq!(ONE_INK_TOLERANCE, 2, "the tolerance is AP-1's measured 1-level bake drift plus one level");
    assert_eq!(pack::one_ink_colour(&ring(0)), Some([200, 40, 90]));
    assert_eq!(pack::one_ink_colour(&ring(2)), Some([200, 40, 90]), "a 2-level drift is still one ink");
    assert_eq!(pack::one_ink_colour(&ring(3)), None, "a 3-level difference is a second colour");
    // Alpha-0 pixels carry anything without breaking the rule.
    let mut junk = ring(0);
    let i = junk.rgba.chunks_exact(4).position(|p| p[3] == 0).expect("a transparent pixel");
    junk.rgba[i * 4..i * 4 + 3].copy_from_slice(&[9, 250, 9]);
    assert_eq!(pack::one_ink_colour(&junk), Some([200, 40, 90]));
    assert_eq!(pack::one_ink_colour(&image(4, 4, |_, _| [5, 5, 5, 0])), None, "fully transparent: no ink");
    let two = image(4, 4, |x, _| if x < 2 { [10, 10, 10, 255] } else { [240, 240, 240, 255] });
    assert_eq!(pack::one_ink_colour(&two), None);
}

/// Protects: the mip chain's sizes are Godot's (`Image::get_image_mipmap_size`:
/// each level `max(1, side >> i)`, down to 1 x 1), so
/// `Image::create_from_data(.., true, ..)` accepts the data.
#[test]
fn mip_sizes_are_godots() {
    assert_eq!(pack::mip_sizes(256, 256).len(), 9);
    assert_eq!(
        pack::mip_sizes(256, 180),
        vec![(256, 180), (128, 90), (64, 45), (32, 22), (16, 11), (8, 5), (4, 2), (2, 1), (1, 1)]
    );
    assert_eq!(pack::mip_sizes(1, 1), vec![(1, 1)]);
}

/// Protects: the raster path's texture is premultiplied and area-filtered at
/// every level: level 0 premultiplies exactly (round half up), the 1 x 1
/// level is the premultiplied mean of the whole sprite, and the data is the
/// whole chain.
#[test]
fn the_raster_path_is_a_premultiplied_area_mip_chain() {
    // Left half (200, 100, 50, 128); right half fully transparent with junk RGB.
    let img = image(8, 4, |x, _| if x < 4 { [200, 100, 50, 128] } else { [255, 255, 255, 0] });
    let ft = pack::feature_raster_texture(&img);
    assert!(!ft.sdf);
    let total: u32 = pack::mip_sizes(8, 4).iter().map(|(w, h)| w * h * 4).sum();
    assert_eq!(ft.data.len() as u32, total);
    // 200 * 128 / 255 = 100.39 -> 100; 100 * 128 / 255 = 50.2 -> 50; 50 * 128 / 255 = 25.1 -> 25.
    assert_eq!(&ft.data[0..4], &[100, 50, 25, 128]);
    assert_eq!(&ft.data[16..20], &[0, 0, 0, 0], "a transparent texel's junk RGB is premultiplied away");
    // Last level (1 x 1): half the area at the premultiplied values above.
    let last = &ft.data[ft.data.len() - 4..];
    assert_eq!(last, &[50, 25, 13, 64], "mean of (100.39, 50.2, 25.1, 128) and zero, rounded half up");
    assert_eq!((ft.extent, ft.stroke_top, ft.max_lod), ((1.0, 1.0), 0.0, 0));
    // An opaque texel premultiplies to itself (alpha 255 / 255 = 1 exactly).
    let opaque = pack::feature_raster_texture(&image(2, 2, |_, _| [255, 128, 1, 255]));
    assert_eq!(&opaque.data[0..4], &[255, 128, 1, 255]);
}

/// Protects: the distance field's mask is the sprite's alpha at 50 %
/// (`>= 128`): ink that never reaches half coverage has no inside, so there is
/// no field to build and the sprite is refused (`None`), not outlined.
#[test]
fn the_distance_field_mask_is_alpha_at_half() {
    let faint = image(32, 32, |x, y| if (8..24).contains(&x) && (8..24).contains(&y) { [10, 20, 30, 127] } else { [0, 0, 0, 0] });
    assert!(pack::feature_sdf_texture(&faint, [10, 20, 30]).is_none(), "alpha 127 everywhere is below the mask");
    let half = image(32, 32, |x, y| if (8..24).contains(&x) && (8..24).contains(&y) { [10, 20, 30, 128] } else { [0, 0, 0, 0] });
    assert!(pack::feature_sdf_texture(&half, [10, 20, 30]).is_some(), "alpha 128 is ink");
    // One-ink but faint: `feature_texture` falls back to the raster path.
    assert!(!pack::feature_texture(&faint).sdf);
}

/// Protects: a one-ink sprite takes the distance-field path. The texture is
/// AP-10's `SdfPyramid` of the sprite's alpha: RGB is the ink at every texel,
/// A is 128-is-the-edge (inside the stroke above, far outside 0), its first
/// levels are the pyramid's own and its length is Godot's whole chain; the
/// 16-texel ring reports a stroke near 16 and a painted (two-colour) sprite
/// takes the raster path instead.
#[test]
fn one_ink_art_takes_the_distance_field_path() {
    let ft = pack::feature_texture(&ring(0));
    assert!(ft.sdf);
    assert_eq!(ft.extent, (1.0, 1.0));
    let top = ft.w;
    assert!(top.is_power_of_two() && (128..=512).contains(&top), "top level {top}");
    let total: u32 = pack::mip_sizes(top, top).iter().map(|(w, h)| w * h * 4).sum();
    assert_eq!(ft.data.len() as u32, total);
    assert!(ft.data.chunks_exact(4).all(|p| p[..3] == [200, 40, 90]), "RGB is the ink everywhere");
    let a = |x: u32, y: u32| ft.data[((y * top + x) * 4 + 3) as usize];
    // Texel centres: mid-stroke (radius 92 of 256 -> 92/256 of top), the
    // hole's centre and a corner.
    let s = f64::from(top) / 256.0;
    let mid = ((128.0 - 92.0) * s) as u32;
    assert!(a(mid, top / 2) > 128, "inside the stroke is above the edge code ({})", a(mid, top / 2));
    assert!(a(top / 2, top / 2) < 128, "the ring's hole is outside");
    assert_eq!(a(0, 0), 0, "far outside saturates at 0");
    // `stroke_p2` uses the lab's 2e - 1 width, so a 16-texel stroke reads 15
    // give or take the ridge's sampling; in top texels that is * top / 256.
    let in_mask = ft.stroke_top * 256.0 / f64::from(top);
    assert!((13.0..=17.0).contains(&in_mask), "stroke {in_mask} mask px");
    assert!(ft.max_lod >= 1 && ft.max_lod as usize <= pack::mip_sizes(top, top).len() - 1);

    let painted = image(16, 16, |x, _| if x < 8 { [255, 0, 255, 255] } else { [128, 0, 128, 255] });
    assert!(!pack::feature_texture(&painted).sdf, "two colours: the raster path");
}

/// Protects: a non-square one-ink sprite's field sits in the top-left of the
/// pyramid's square and `extent` says how much of each side it uses.
#[test]
fn a_non_square_sprite_reports_its_extent() {
    let img = image(256, 128, |x, y| if (100..156).contains(&x) && (40..90).contains(&y) { [10, 20, 30, 255] } else { [0, 0, 0, 0] });
    let ft = pack::feature_texture(&img);
    assert!(ft.sdf);
    assert_eq!(ft.extent, (1.0, 0.5));
}
