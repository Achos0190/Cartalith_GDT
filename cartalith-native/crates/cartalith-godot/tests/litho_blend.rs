//! The lithology corner blend (Ruling CG addendum 2, owner, 2026-10-12;
//! `GEOLOGY_FIRST_SCOPE.md` §7 "Follow-up: what the remaining artefacts are"
//! (a)): under magnification the export bake and the deep-zoom tiles blend
//! the colour terms of the four jittered corner rocks by bilinear weight,
//! instead of reading the nearest cell and drawing every contact as a
//! one-cell staircase.
//!
//! What this file pins, each against an independent statement rather than
//! the code's own constants:
//! * the filter switches on strictly below one cell per pixel, both axes;
//! * a contact with known corner weights gives exactly the weighted colour
//!   mix (ramp and microtexture), and four agreeing corners collapse to the
//!   single-rock path;
//! * at every magnified sample whose four corners agree, the blended read is
//!   the nearest read (byte identity by control flow away from a contact),
//!   and the contact fixture does reach the blend (non-vacuous);
//! * a magnified bake draws contact pixels that are neither rock's pure
//!   colour (the blend reaches the export), while a grid-resolution bake
//!   draws only pure-rock pixels (nearest).

/// The renderer under test, compiled straight from `src/render.rs` (via `#[path]`) so this integration target can reach its `pub(crate)` items without the Godot-facing `#[func]` layer.
#[path = "../src/render.rs"]
mod render;

use render::{LithoSample, RenderCtx, TerrainAppearance};

/// Fixture grid width in cells. Judgement: wide enough for a 45° contact to cross the jitter's ~11-cell wavelength (`vnoise(x * 0.09)`) several times; not square so a stride mix-up shows.
const GW: usize = 40;
/// Fixture grid height in cells; see [`GW`].
const GH: usize = 30;
/// Limestone's `LITHO_PALETTE_ORDER` index (`render::LITHO_PALETTE_ORDER[3]`).
const LIMESTONE: u8 = 3;
/// Shale's `LITHO_PALETTE_ORDER` index (`render::LITHO_PALETTE_ORDER[5]`).
const SHALE: u8 = 5;

/// Land everywhere, gentle slope, temperate and moderately wet, so every pixel takes `land_color`. Lithology: limestone below the 45° line `x + y = 35`, shale above it -- the shape of the τ 4 breach contact the follow-up found.
fn fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u8>) {
    let n = GW * GH;
    let field: Vec<f32> = (0..n).map(|i| 0.55 + 0.004 * ((i % GW) as f32) + 0.002 * ((i / GW) as f32)).collect();
    let temp = vec![12.0f32; n];
    let rain = vec![0.45f32; n];
    let lith: Vec<u8> = (0..n).map(|i| if (i % GW) + (i / GW) < 35 { LIMESTONE } else { SHALE }).collect();
    assert!(lith.contains(&LIMESTONE) && lith.contains(&SHALE), "the contact fixture has one rock only");
    (field, temp, rain, lith)
}

/// Protects: the blend engages strictly below one cell per pixel on BOTH axes; exactly one cell per pixel (the screen's own sampling, a grid-resolution export) keeps the nearest read.
#[test]
fn magnification_gate_is_strict_and_two_axis() {
    assert!(!render::litho_magnified(1.0, 1.0), "one cell per pixel must stay nearest");
    assert!(render::litho_magnified(0.125, 0.125));
    assert!(render::litho_magnified(0.999, 0.999));
    assert!(!render::litho_magnified(0.5, 1.0), "a raster that does not magnify on Y must stay nearest");
    assert!(!render::litho_magnified(2.0, 0.5));
}

/// Protects: four agreeing corners collapse to `One` (the single-rock path), and disagreeing corners merge to one weight per distinct rock with the bilinear weights summed.
#[test]
fn corners_collapse_and_merge() {
    assert_eq!(LithoSample::from_corners([(SHALE, 0.1), (SHALE, 0.2), (SHALE, 0.3), (SHALE, 0.4)]), LithoSample::One(SHALE));
    match LithoSample::from_corners([(LIMESTONE, 0.5), (SHALE, 0.25), (LIMESTONE, 0.125), (SHALE, 0.125)]) {
        LithoSample::Blend(p) => {
            assert_eq!(p[0], (LIMESTONE, 0.625));
            assert_eq!(p[1], (SHALE, 0.375));
            assert_eq!(p[2].1, 0.0);
            assert_eq!(p[3].1, 0.0);
        }
        other => panic!("two rocks must blend, got {other:?}"),
    }
}

/// Protects: a contact with known weights gives exactly the weighted mix of each rock's ramp colour and microtexture (the bilinear mix of the colour terms, not of the category).
#[test]
fn a_contact_with_known_weights_gives_the_bilinear_mix() {
    let a = TerrainAppearance::default();
    let tt = 0.37;
    let lime = LithoSample::One(LIMESTONE).ramp(&a, tt);
    let shale = LithoSample::One(SHALE).ramp(&a, tt);
    assert_ne!(lime, shale, "the two palettes must differ or the mix proves nothing");
    let mixed = LithoSample::from_corners([(LIMESTONE, 0.75), (SHALE, 0.25), (SHALE, 0.0), (LIMESTONE, 0.0)]).ramp(&a, tt);
    let want = (lime.0 * 0.75 + shale.0 * 0.25, lime.1 * 0.75 + shale.1 * 0.25, lime.2 * 0.75 + shale.2 * 0.25);
    for (g, w) in [(mixed.0, want.0), (mixed.1, want.1), (mixed.2, want.2)] {
        assert!((g - w).abs() < 1e-9, "ramp mix {mixed:?} != {want:?}");
    }
    let (fx, fy, r) = (0.31, 0.17, 0.42);
    let mt_l = LithoSample::One(LIMESTONE).microtexture(fx, fy, r);
    let mt_s = LithoSample::One(SHALE).microtexture(fx, fy, r);
    assert_ne!(mt_l, mt_s, "the two microtextures must differ at this point");
    let mt = LithoSample::from_corners([(LIMESTONE, 0.75), (SHALE, 0.25), (SHALE, 0.0), (LIMESTONE, 0.0)]).microtexture(fx, fy, r);
    assert!((mt - (0.75 * mt_l + 0.25 * mt_s)).abs() < 1e-12, "microtexture mix {mt}");
}

/// Protects: wherever a magnified sample's four corners agree, the blended read IS the nearest read (no change away from a contact); the 45° contact does reach the blend; and each blend's weights sum to 1.
#[test]
fn no_change_where_corners_agree_and_the_contact_blends() {
    let (field, temp, rain, lith) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default()).with_lithology(&lith);
    let (mut ones, mut blends) = (0usize, 0usize);
    // Eight samples per cell, the deep-zoom density the follow-up measured.
    for sy in 0..(GH - 1) * 8 {
        for sx in 0..(GW - 1) * 8 {
            let (x, y) = (sx as f64 / 8.0, sy as f64 / 8.0);
            let near = ctx.litho_at_f(x, y, false).expect("lithology attached and weighted");
            let LithoSample::One(near_rock) = near else { panic!("the nearest read must be One") };
            match ctx.litho_at_f(x, y, true).expect("lithology attached") {
                LithoSample::One(r) => {
                    ones += 1;
                    assert_eq!(r, near_rock, "agreeing corners at ({x}, {y}) changed the rock");
                }
                LithoSample::Blend(p) => {
                    blends += 1;
                    let s: f64 = p.iter().map(|q| q.1).sum();
                    assert!((s - 1.0).abs() < 1e-12, "weights sum to {s} at ({x}, {y})");
                    assert!(p.iter().filter(|q| q.1 > 0.0).count() >= 2, "a Blend with one rock at ({x}, {y})");
                }
            }
        }
    }
    assert!(blends > 0, "the contact fixture never reached the blend");
    assert!(ones > blends, "most of the fixture is away from the contact ({ones} vs {blends})");
}

/// Protects: the blend reaches `bake_rect`'s pixels (attaching a capability is not a consumer reading it): a magnified bake of the contact draws pixels that match neither an all-limestone nor an all-shale bake, which a nearest read cannot; a grid-resolution bake draws only pixels matching one of the two.
#[test]
fn the_magnified_bake_reads_the_blend() {
    let (field, temp, rain, lith) = fixture();
    let a = TerrainAppearance::default();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, a.clone()).with_lithology(&lith);
    let bf = render::BakeFields::new(&ctx);
    let (ow, oh) = (GW * 8, GH * 8);
    let px = render::bake_rect(&ctx, &bf, None, ow, oh, 0, 0, ow, oh);
    // Independent of the filter's code: every pixel of a nearest read equals
    // the same pixel of the all-limestone or the all-shale bake (every other
    // input is identical), so a pixel matching neither is a blend.
    let all_l = vec![LIMESTONE; GW * GH];
    let all_s = vec![SHALE; GW * GH];
    let bake_with = |l: &[u8]| {
        let c = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, a.clone()).with_lithology(l);
        let b = render::BakeFields::new(&c);
        render::bake_rect(&c, &b, None, ow, oh, 0, 0, ow, oh)
    };
    let (pl, ps) = (bake_with(&all_l), bake_with(&all_s));
    let mut in_neither = 0;
    for i in 0..ow * oh {
        let o = i * 3;
        let p = &px[o..o + 3];
        if p != &pl[o..o + 3] && p != &ps[o..o + 3] {
            in_neither += 1;
        }
    }
    assert!(in_neither > 0, "no magnified pixel is a blend of the two rocks: the bake does not read the corner blend");
    // And at one pixel per cell the bake still equals one of the two pure
    // renders at every pixel (nearest read).
    let px1 = render::bake_rect(&ctx, &bf, None, GW, GH, 0, 0, GW, GH);
    let (pl1, ps1) = (
        {
            let c = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, a.clone()).with_lithology(&all_l);
            render::bake_rect(&c, &render::BakeFields::new(&c), None, GW, GH, 0, 0, GW, GH)
        },
        {
            let c = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, a.clone()).with_lithology(&all_s);
            render::bake_rect(&c, &render::BakeFields::new(&c), None, GW, GH, 0, 0, GW, GH)
        },
    );
    for i in 0..GW * GH {
        let o = i * 3;
        assert!(px1[o..o + 3] == pl1[o..o + 3] || px1[o..o + 3] == ps1[o..o + 3], "grid-resolution bake pixel {i} is a blend");
    }
}
