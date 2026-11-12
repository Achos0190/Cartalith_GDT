//! Ruling ET's playa follow-up and RR-5 (Ruling ES): the salt-flat tint of a
//! basin the lake balance dried, and the lake's tone by depth.
//!
//! Both are render only. This file protects: the mask's definition at
//! literals; that the mask and the fill levels reach the screen (`cell_color`),
//! the export bake (`BakeFields::pixel`) and the tiles (`TileFields::new`
//! through `render_biome_tile_rgba`) -- one funnel, three consumers; that
//! every new term is inert under the `js_reference` look and without the
//! attachment; and the depth law's literal anchors.
//!
//! `#[path]`-includes `render.rs` for the reason `base_lake_stamp.rs` gives:
//! `cartalith-godot` is `cdylib`-only.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;

use render::{BakeFields, LakeBalanceInputs, RenderCtx, TerrainAppearance};

/// Fixture width: a judgement -- room for an 8x8 bowl inside a plateau rim.
const GW: usize = 24;
/// Fixture height; see [`GW`].
const GH: usize = 24;
/// The project's default sea level; the bowl and plateau sit above it.
const SEA: f64 = 0.42;
/// The bowl's centre cell, inside rows/columns 8..16.
const CENTRE: usize = 12 * GW + 12;
/// A plateau corner, far from the bowl.
const CORNER: usize = 2 * GW + 2;

/// A 0.70 plateau with a closed 0.50 bowl (the `base_lake_stamp.rs` shape):
/// the flood pools it into a lake, 0.20 deep, which is far over `LAKE_DEPTH`.
fn bowl() -> Vec<f32> {
    let mut f = vec![0.70f32; GW * GH];
    for y in 8..16 {
        for x in 8..16 {
            f[y * GW + x] = 0.50;
        }
    }
    f
}

/// The balanced classification of the bowl world with `inflow` discharge in
/// every cell, at 25 C (evaporation 2.0 a cell): inflow 0 dries the bowl (a
/// playa), inflow 1000 keeps it a lake. `balanced == false` is the old rule.
fn water(inflow: f32, balanced: bool) -> cartalith_civ::WaterBodies {
    let field = bowl();
    let rain = vec![1.0f32; GW * GH];
    let flow = vec![inflow; GW * GH];
    let temp = vec![25.0f32; GW * GH];
    let b = LakeBalanceInputs::when(balanced, Some(&flow), &temp);
    render::water_bodies_floored_balanced(&field, &rain, GW, GH, SEA, false, None, None, b)
}

/// Protects the mask's definition at literals: old lake and not now lake and
/// above sea. A below-sea cell, a still-lake cell, a never-lake cell and a
/// length mismatch (which is "no information", an empty mask) are all 0 / empty.
#[test]
fn the_playa_mask_is_old_lake_now_dry_above_sea() {
    let old = [2u8, 2, 0, 2, 1];
    let now = [0u8, 2, 0, 0, 0];
    let field = [0.6f32, 0.6, 0.6, 0.1, 0.6];
    assert_eq!(render::playa_mask(&old, &now, &field, SEA), vec![1, 0, 0, 0, 0]);
    assert!(render::playa_mask(&old, &now[..4], &field, SEA).is_empty());
}

/// Protects the funnel: a starved bowl is dry in the classification and a
/// playa in the mask; the same bowl with inflow stays lake with no playa; and
/// with the balance off the mask is absent (empty), not all zero.
#[test]
fn the_balance_leaves_a_playa_where_it_dried_a_lake() {
    let dry = water(0.0, true);
    assert_eq!(dry.classification[CENTRE], 0, "premise: the starved bowl is dry");
    assert_eq!(dry.playa.len(), GW * GH);
    assert_eq!(dry.playa[CENTRE], 1);
    assert_eq!(dry.playa[CORNER], 0);
    assert_eq!(dry.playa.iter().filter(|&&p| p != 0).count(), dry.playa.iter().zip(&water(0.0, false).classification).filter(|&(&p, &c)| p != 0 && c == 2).count(), "every playa cell was a lake under the old rule");

    let wet = water(1000.0, true);
    assert_eq!(wet.classification[CENTRE], 2);
    assert!(wet.playa.iter().all(|&p| p == 0), "a lake the balance keeps is no playa");

    let old = water(0.0, false);
    assert_eq!(old.classification[CENTRE], 2, "the old rule keeps the lake");
    assert!(old.playa.is_empty(), "no balance, no information: an empty mask, never an all-zero one");
}

/// A context over the bowl world with `a`; `lakes`/`fill`/`playa` attached
/// when given.
fn ctx<'a>(field: &'a [f32], temp: &'a [f32], rain: &'a [f32], a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(field, temp, rain, None, GW, GH, SEA, false, 55.0, 5.0, a)
}

/// L1 distance between a `0..=1` pixel `c` and a `0..=255` colour `t`, in levels.
fn dist(c: (f64, f64, f64), t: (f64, f64, f64)) -> f64 {
    (c.0 * 255.0 - t.0).abs() + (c.1 * 255.0 - t.1).abs() + (c.2 * 255.0 - t.2).abs()
}

/// Protects the salt flat on `cell_color`: a dry-bowl cell with the playa
/// attached moves toward `PLAYA_SALT` under the app look, stays byte-identical
/// under `js_reference` and under `playa_tint = 0`, and a cell off the playa
/// never moves.
#[test]
fn a_playa_cell_reads_as_salt_and_the_reference_look_is_untouched() {
    let (field, rain, temp) = (bowl(), vec![1.0f32; GW * GH], vec![25.0f32; GW * GH]);
    let w = water(0.0, true);
    assert_eq!(w.playa[CENTRE], 1, "premise: the fixture has a playa");
    let plain = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&w.classification);
    let salt = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&w.classification).with_water_detail(&w.fill_level, &w.playa);
    let (p, s) = (render::cell_color(&plain, 12, 12), render::cell_color(&salt, 12, 12));
    assert!(dist(s, render::PLAYA_SALT) < dist(p, render::PLAYA_SALT) - 20.0, "the playa must move toward the salt colour: {p:?} -> {s:?}");
    assert_eq!(render::cell_color(&plain, 2, 2), render::cell_color(&salt, 2, 2), "land off the playa must not move");

    for look in [TerrainAppearance::js_reference(), TerrainAppearance { playa_tint: 0.0, ..TerrainAppearance::default() }] {
        let a = ctx(&field, &temp, &rain, look.clone()).with_lakes(&w.classification);
        let b = ctx(&field, &temp, &rain, look).with_lakes(&w.classification).with_water_detail(&w.fill_level, &w.playa);
        assert_eq!(render::cell_color(&a, 12, 12), render::cell_color(&b, 12, 12), "tint off must be the old pixel");
    }
}

/// Protects the export bake's seam: `BakeFields::pixel` at an integer
/// coordinate agrees with `cell_color` (within the bake's own f32 bound) for
/// the playa cell and for a lake cell with the depth tone, so the PNG shows
/// the screen's salt flat and lake tone.
#[test]
fn the_export_bake_draws_the_same_salt_flat_and_lake_tone() {
    let (field, rain, temp) = (bowl(), vec![1.0f32; GW * GH], vec![25.0f32; GW * GH]);
    // The playa world.
    let w = water(0.0, true);
    let c = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&w.classification).with_water_detail(&w.fill_level, &w.playa);
    let bf = BakeFields::new(&c);
    let (sx, sy, sz) = render::cell_color(&c, 12, 12);
    let (bx, by, bz) = bf.pixel(&c, 12.0, 12.0);
    for (a, b) in [(sx, bx), (sy, by), (sz, bz)] {
        assert!((a - b).abs() < 2.0 / 255.0, "playa pixel: screen {a} vs bake {}", b * 255.0);
    }
    // The lake world (inflow keeps the lake): the bake's lake pixel carries the depth tone too.
    let l = water(1000.0, true);
    let c = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&l.classification).with_water_detail(&l.fill_level, &l.playa);
    let bf = BakeFields::new(&c);
    let (sx, sy, sz) = render::cell_color(&c, 12, 12);
    let (bx, by, bz) = bf.pixel(&c, 12.0, 12.0);
    for (a, b) in [(sx, bx), (sy, by), (sz, bz)] {
        assert!((a - b).abs() < 2.0 / 255.0, "lake pixel: screen {a} vs bake {}", b * 255.0);
    }
}

/// Protects the tiles' seam: a tile over the dry bowl, built from
/// `TileFields::new` of a context whose look carries `lake_balance`, draws the
/// salt flat (the pixel differs from the same tile with `playa_tint = 0` and is
/// nearer the salt colour) and a tile outside the bowl does not move.
#[test]
fn the_tiles_draw_the_salt_flat_from_the_same_funnel() {
    let (field, rain, temp) = (bowl(), vec![1.0f32; GW * GH], vec![25.0f32; GW * GH]);
    let flow = vec![0.0f32; GW * GH];
    let draw = |tint: f64, bounds: render::TileBounds| -> Vec<u8> {
        let a = TerrainAppearance { lake_balance: true, playa_tint: tint, ..TerrainAppearance::default() };
        let c = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a);
        let (tw, th) = (24usize, 24usize);
        let mut tile = vec![0f32; tw * th];
        for y in 0..th {
            for x in 0..tw {
                let (wx, wy) = (bounds.x + x as f64 * bounds.w / (tw - 1) as f64, bounds.y + y as f64 * bounds.h / (th - 1) as f64);
                tile[y * tw + x] = field[(wy as usize).min(GH - 1) * GW + (wx as usize).min(GW - 1)];
            }
        }
        let tf = render::TileFields::new(&c, None);
        render::render_biome_tile_rgba(&c, &tile, tw, th, bounds, &tf)
    };
    let inside = render::TileBounds { x: 9.0, y: 9.0, w: 5.0, h: 5.0 };
    let (on, off) = (draw(1.0, inside), draw(0.0, inside));
    assert_eq!(on.len(), 24 * 24 * 4, "the tile came back empty");
    let o = (12 * 24 + 12) * 4;
    let px = |b: &[u8]| (f64::from(b[o]) / 255.0, f64::from(b[o + 1]) / 255.0, f64::from(b[o + 2]) / 255.0);
    assert!(dist(px(&on), render::PLAYA_SALT) < dist(px(&off), render::PLAYA_SALT) - 20.0, "tile playa: {:?} vs {:?}", px(&off), px(&on));
    let outside = render::TileBounds { x: 0.0, y: 0.0, w: 5.0, h: 5.0 };
    assert_eq!(draw(1.0, outside), draw(0.0, outside), "a tile off the playa must not move");
}

/// Protects RR-5's law at literal anchors: today's 0.30 up to four
/// `LAKE_DEPTH` (so ponds and shelves keep their tone), 0.62 at sixteen and
/// beyond, the smoothstep midpoint 0.46 at ten, rising monotonically between.
#[test]
fn the_lake_depth_axis_has_its_literal_anchors() {
    assert_eq!(render::lake_depth_axis(1.0), 0.30);
    assert_eq!(render::lake_depth_axis(3.9), 0.30);
    assert_eq!(render::lake_depth_axis(4.0), 0.30);
    let t = 0.3 / 12.0;
    assert!((render::lake_depth_axis(4.3) - (0.30 + 0.32 * t * t * (3.0 - 2.0 * t))).abs() < 1e-12, "the rise starts at four");
    assert!((render::lake_depth_axis(16.0) - 0.62).abs() < 1e-12);
    assert!((render::lake_depth_axis(100.0) - 0.62).abs() < 1e-12);
    assert!((render::lake_depth_axis(10.0) - 0.46).abs() < 1e-12, "the smoothstep midpoint");
    let mut last = 0.0;
    for k in 10..200 {
        let v = render::lake_depth_axis(f64::from(k) / 10.0);
        assert!(v >= last, "the axis must not fall with depth");
        last = v;
    }
}

/// Protects the lake tone on `cell_color`: with the fill levels attached a deep
/// cell is darker than a shelf cell under the app look; under `js_reference`
/// (and with the fill levels absent) every lake cell is the constant-depth
/// colour, byte for byte.
#[test]
fn a_deep_lake_is_darker_than_a_shelf_and_the_reference_look_is_untouched() {
    let (field, rain, temp) = (bowl(), vec![1.0f32; GW * GH], vec![25.0f32; GW * GH]);
    let l = water(1000.0, true);
    let shelf = 11 * GW + 11;
    assert_eq!(l.classification[shelf], 2, "premise: the shelf cell is lake");
    // Make a genuinely shallow cell: raise its bed to 0.5 below the fill (a shelf).
    let mut fill = l.fill_level.clone();
    fill[shelf] = field[shelf] + 4.0 * cartalith_civ::LAKE_DEPTH as f32;
    let on = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&l.classification).with_water_detail(&fill, &l.playa);
    let lum = |c: (f64, f64, f64)| 0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2;
    let (deep, shallow) = (lum(render::cell_color(&on, 12, 12)), lum(render::cell_color(&on, 11, 11)));
    assert!(shallow > deep + 0.05, "the shelf must be paler than the deep: {shallow} vs {deep}");

    let plain = ctx(&field, &temp, &rain, TerrainAppearance::js_reference()).with_lakes(&l.classification);
    let reference = ctx(&field, &temp, &rain, TerrainAppearance::js_reference()).with_lakes(&l.classification).with_water_detail(&fill, &l.playa);
    for &(x, y) in &[(12usize, 12usize), (11, 11)] {
        assert_eq!(render::cell_color(&plain, x, y), render::cell_color(&reference, x, y), "js_reference must ignore the fill levels");
    }
    let none = ctx(&field, &temp, &rain, TerrainAppearance::default()).with_lakes(&l.classification);
    let zero_tone = TerrainAppearance { lake_depth_tone: 0.0, ..TerrainAppearance::default() };
    let tone0 = ctx(&field, &temp, &rain, zero_tone).with_lakes(&l.classification).with_water_detail(&fill, &l.playa);
    for &(x, y) in &[(12usize, 12usize), (11, 11)] {
        assert_eq!(render::cell_color(&none, x, y), render::cell_color(&tone0, x, y), "tone 0 with fill levels is the no-fill pixel");
    }
}

/// Protects the look keys' defaults at literals: on in the app look, off in
/// the reference pin, tunable 0..1.
#[test]
fn the_new_look_keys_are_on_in_the_app_and_off_in_the_reference() {
    let (d, r) = (TerrainAppearance::default(), TerrainAppearance::js_reference());
    assert_eq!((d.playa_tint, d.lake_depth_tone, d.river_wet_bank), (1.0, 1.0, 1.0));
    assert_eq!((r.playa_tint, r.lake_depth_tone, r.river_wet_bank), (0.0, 0.0, 0.0));
    for k in ["playa_tint", "lake_depth_tone", "river_wet_bank"] {
        let t = TerrainAppearance::TUNABLE.iter().find(|t| t.0 == k).copied().unwrap_or_else(|| panic!("{k} is not a tunable"));
        assert_eq!((t.1, t.2), (0.0, 1.0));
    }
}

/// Protects the salt-flat arithmetic at literals: on a full mask a grey (100,
/// 100, 100) at grain 0.5 (crust factor exactly 1) goes half way to
/// `PLAYA_SALT` (214, 206, 186); `playa_tint` 0.5 goes a quarter; an off-playa
/// pixel, no mask and a wrong-size mask return the colour untouched.
#[test]
fn the_salt_flat_mix_is_pinned_at_literals() {
    let full = vec![1u8; GW * GH];
    let a = TerrainAppearance::default();
    let c = (100.0, 100.0, 100.0);
    let got = render::playa_apply(&a, Some(&full), c, 5.0, 5.0, GW, GH, 0.5);
    for (g, w) in [(got.0, 157.0), (got.1, 153.0), (got.2, 143.0)] {
        assert!((g - w).abs() < 1e-9, "{got:?}");
    }
    let half = TerrainAppearance { playa_tint: 0.5, ..TerrainAppearance::default() };
    let got = render::playa_apply(&half, Some(&full), c, 5.0, 5.0, GW, GH, 0.5);
    assert!((got.0 - 128.5).abs() < 1e-9, "{got:?}");
    let none = vec![0u8; GW * GH];
    assert_eq!(render::playa_apply(&a, Some(&none), c, 5.0, 5.0, GW, GH, 0.5), c);
    assert_eq!(render::playa_apply(&a, None, c, 5.0, 5.0, GW, GH, 0.5), c);
    assert_eq!(render::playa_apply(&a, Some(&full[..10]), c, 5.0, 5.0, GW, GH, 0.5), c);
    // The crust factor: grain 0 is 0.8 of the mix, grain 1 is 1.2.
    let lo = render::playa_apply(&a, Some(&full), c, 5.0, 5.0, GW, GH, 0.0);
    let hi = render::playa_apply(&a, Some(&full), c, 5.0, 5.0, GW, GH, 1.0);
    assert!((lo.0 - (100.0 + 114.0 * 0.4)).abs() < 1e-9 && (hi.0 - (100.0 + 114.0 * 0.6)).abs() < 1e-9, "{lo:?} {hi:?}");
}

/// Protects the cover's softness at literals: one set cell is exactly 1 at its
/// centre; half way to a clear neighbour the bilinear is 0.5 and the smoothstep
/// over 0.3..0.7 leaves 0.5; a third of the way out (0.7) is 1, two thirds (0.3
/// bilinear) is 0.
#[test]
fn the_playa_cover_is_a_soft_contour_over_the_bilinear() {
    let mut m = vec![0u8; GW * GH];
    m[5 * GW + 5] = 1;
    assert_eq!(render::playa_cover(&m, 5.0, 5.0, GW, GH), 1.0);
    assert!((render::playa_cover(&m, 5.5, 5.0, GW, GH) - 0.5).abs() < 1e-12);
    assert_eq!(render::playa_cover(&m, 5.3, 5.0, GW, GH), 1.0);
    assert_eq!(render::playa_cover(&m, 5.7, 5.0, GW, GH), 0.0);
    assert_eq!(render::playa_cover(&m, 9.0, 9.0, GW, GH), 0.0);
}

/// Protects `lake_depth_for` at literals: the tone strength scales the move
/// from today's 0.30, absent or non-finite depth is today's 0.30.
#[test]
fn the_lake_depth_blend_is_pinned_at_literals() {
    let on = TerrainAppearance::default();
    let half = TerrainAppearance { lake_depth_tone: 0.5, ..TerrainAppearance::default() };
    let ld = cartalith_civ::LAKE_DEPTH;
    assert_eq!(render::lake_depth_for(&on, None), 0.30);
    assert_eq!(render::lake_depth_for(&on, Some(f64::NAN)), 0.30);
    assert!((render::lake_depth_for(&on, Some(2.0 * ld)) - 0.30).abs() < 1e-12);
    assert!((render::lake_depth_for(&on, Some(16.0 * ld)) - 0.62).abs() < 1e-12);
    assert!((render::lake_depth_for(&half, Some(16.0 * ld)) - 0.46).abs() < 1e-12);
    assert!((render::lake_depth_for(&on, Some(ld)) - 0.30).abs() < 1e-12);
    assert!((render::lake_depth_for(&on, Some(10.0 * ld)) - 0.46).abs() < 1e-12);
}
