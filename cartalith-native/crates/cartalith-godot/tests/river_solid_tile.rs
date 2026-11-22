//! **F23 (owner finding, 2026-11-21; fixed 2026-11-22): a wide river at x64 is solid water.** The ground's mottle showed
//! inside it. Ablated in the real app on seed 483920 (1 024 x 656, 1 200 km, x64, `_small3_probe.gd --mode riverabl`):
//! the river's own alpha was 1 (not coverage), and two ground terms acted on a river pixel after the river was laid,
//! the crest stroke (`apply_crest`, `crest_strength`) and the local-contrast delta (`local_contrast`, from a detail
//! band built from the terrain alone): switching both off took the luma variance inside the river from 7.709 to 2.152.
//! The tile now weighs each by one minus the river's coverage on land pixels.
//!
//! What this protects, on a 64 x 64 fixture with bumpy ground, the west half land and the east half (block column 1 of
//! the river layer) an opaque river: (1) the river's pixels are one colour with the shipped `local_contrast` and
//! `crest_strength` on (the interior's plane-removed luma variance, columns 34..46, rows 14..50 clear of the east edge's vignette, is 0.000 after the fix and 189.946 before it, at a `local_contrast` of 3.0 so the sea patch below carries a delta the byte quantiser cannot hide; the shipped 0.55 reads 21.132 before); (2) the same
//! two terms still texture the land (variance 1224.8, bar 40, a positive control: a fix that switched the
//! terms off everywhere would read 0); (3) the sea at a river mouth (the layer covers water pixels there) is byte-identical with and without the layer; (4) a land pixel with the river layer attached is byte-identical to one rendered
//! without it, 20 columns clear of the river (the fix moves no land).
#![allow(dead_code)]

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use render::{RenderCtx, RiverLayer, TerrainAppearance, TileBounds, TileFields};

const GW: usize = 64;
const GH: usize = 64;
const SEA: f64 = 0.42;
const MAP_KM: f64 = 25.0;

fn appearance() -> TerrainAppearance {
    TerrainAppearance { border_width_frac: 0.0, paper_strength: 0.0, haze_strength: 0.0, local_contrast: 3.0, crest_strength: 0.12, ..TerrainAppearance::default() }
}

/// Bumpy temperate land everywhere (sine ridges, so crests and a detail band exist on both halves).
fn field() -> Vec<f32> {
    (0..GW * GH).map(|i| {
        let (x, y) = ((i % GW) as f64, (i / GW) as f64);
        let ground = 0.56 + 0.05 * (x * 0.9).sin() * (y * 0.7).cos() + 0.03 * (x * 0.31 + y * 0.43).sin();
        // A sea patch inside the river layer's block (a river mouth: the layer covers water there too).
        let sea = (40.0..58.0).contains(&x) && (4.0..12.0).contains(&y);
        (if sea { 0.30 + 0.04 * (x * 0.8).sin() * (y * 0.9).cos() } else { ground }) as f32
    }).collect()
}

fn river_layer() -> RiverLayer {
    let mut l = RiverLayer::new(GW, GH);
    let b = render::RIVER_BLOCK;
    for by in 0..GH.div_ceil(b) {
        // Opaque, premultiplied, `land_color`'s 0..=255 scale: (60, 90, 140) at alpha 1.
        l.put_block(1, by, Some(vec![[60.0, 90.0, 140.0, 1.0]; b * b].into_boxed_slice()), None);
    }
    l
}

fn tile(river: Option<&RiverLayer>) -> Vec<u8> {
    let (f, t, r) = (field(), vec![14.0f32; GW * GH], vec![0.7f32; GW * GH]);
    let c = RenderCtx::with_appearance(&f, &t, &r, None, GW, GH, SEA, false, 55.0, 5.0, appearance()).with_map_scale(MAP_KM);
    let mut screen = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (cr, cg, cb) = render::cell_color(&c, x, y);
            let o = (y * GW + x) * 3;
            screen[o] = (cr.clamp(0.0, 1.0) * 255.0) as u8;
            screen[o + 1] = (cg.clamp(0.0, 1.0) * 255.0) as u8;
            screen[o + 2] = (cb.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    assert!(screen.iter().any(|&v| v != 0), "an all-black render proves nothing");
    let tf = TileFields::new(&c, Some(&screen)).without_lakes();
    let bounds = TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 };
    let out = render::render_biome_tile_rgba_rivers(&c, &f, GW, GH, 0, bounds, &tf, river);
    assert_eq!(out.len(), GW * GH * 4);
    out
}

fn luma(t: &[u8], x: usize, y: usize) -> f64 {
    let o = (y * GW + x) * 4;
    0.299 * t[o] as f64 + 0.587 * t[o + 1] as f64 + 0.114 * t[o + 2] as f64
}

/// Luma variance left after removing the best-fit plane (a smooth vignette or tilt is lighting, not mottle).
fn variance(t: &[u8], xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> f64 {
    let v: Vec<(f64, f64, f64)> = ys.flat_map(|y| xs.clone().map(move |x| (x, y))).map(|(x, y)| (luma(t, x, y), x as f64, y as f64)).collect();
    let n = v.len() as f64;
    let (mut sl, mut sx, mut sy, mut sxx, mut syy, mut sxl, mut syl) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(l, x, y) in &v {
        sl += l; sx += x; sy += y; sxx += x * x; syy += y * y; sxl += x * l; syl += y * l;
    }
    let (mx, my, ml) = (sx / n, sy / n, sl / n);
    let (bx, by) = ((sxl / n - mx * ml) / (sxx / n - mx * mx), (syl / n - my * ml) / (syy / n - my * my));
    v.iter().map(|&(l, x, y)| { let r = l - (ml + bx * (x - mx) + by * (y - my)); r * r }).sum::<f64>() / n
}

#[test]
fn a_wide_river_is_solid_and_the_land_beside_it_is_untouched() {
    let layer = river_layer();
    let with = tile(Some(&layer));
    let river_var = variance(&with, 34..46, 14..50);
    let land_var = variance(&with, 4..28, 14..50);
    println!("F23 river_var={river_var:.3} land_var={land_var:.3}");
    assert!(land_var > 40.0, "positive control: the land must carry texture ({land_var})");
    assert!(river_var < 0.5, "the river must be one colour, not the ground's relief showing through ({river_var})");
    let without = tile(None);
    // (4) the sea at a river mouth keeps its local contrast: the layer covers those water pixels, and the fix is
    // land-only. Rows 5..11, columns 42..56 are sea; they must equal the render without the layer.
    for y in 5..11 {
        for x in 42..56 {
            let o = (y * GW + x) * 4;
            assert_eq!(&with[o..o + 3], &without[o..o + 3], "sea pixel ({x}, {y}) was damped by the river layer");
        }
    }
    let sea_var = variance(&with, 42..56, 5..11);
    assert!(sea_var > 0.5, "positive control: the sea patch must carry contrast ({sea_var})");
    for y in 0..GH {
        for x in 0..12 {
            let o = (y * GW + x) * 4;
            assert_eq!(&with[o..o + 3], &without[o..o + 3], "land pixel ({x}, {y}) moved when the river layer was attached");
        }
    }
}
