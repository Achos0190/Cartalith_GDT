//! **Ruling CI, the owner's order after mountain relief** (2026-10-18): the
//! square land mottle, then the forest canopy texture.
//!
//! What these tests protect, and why each is shaped the way it is:
//! * `land_grain_warp` at `0.0` returns the reference's three jitter samples
//!   **bit for bit** (a dedicated branch), so `js_reference()` cannot move;
//!   above zero the samples keep their range and spread and lose only the
//!   axis-aligned lattice -- checked on the samples themselves, against the
//!   reference expressions written out here rather than called through the
//!   code under test (`MISTAKES.md`: never assert a constant against itself);
//! * the canopy texture is off by control flow wherever it should be (the
//!   field at `0.0`, a context without a map width, `js_reference()`), and
//!   when on it gains octaves as the output footprint shrinks and stops at its
//!   crown-scale floor -- the owner's "detail at every zoom level" requirement,
//!   as a property of the function;
//! * the screen (`cell_color`), the export (`BakeFields::pixel`) and a
//!   deep-zoom tile draw the canopy from one function at their own footprint,
//!   so at one pixel per cell all three agree;
//! * under the kilometre zoom-detail law a deep-zoom tile takes its material
//!   *category* from the grid-scale slope and draws the sub-cell relief as a
//!   zero-mean *tone* (`micro_relief_tone`, `apply_micro_tone`), in every
//!   look (the 2026-10-18 follow-up that generalised the canopy look's
//!   coarse-slope fix).
//!
//! Pixel pins of the shipped looks live where they always have
//! (`export_raster.rs`' census, `color_space.rs`); this file pins behaviour.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use cartalith_noise::vnoise;
use render::{BakeFields, CanopyScale, RenderCtx, TerrainAppearance, TileBounds, TileFields};

/// The app's default scale: 800 km over a 2048-cell grid.
const KM_PER_CELL: f64 = 800.0 / 2048.0;

fn scale(cells_per_px: f64) -> CanopyScale {
    CanopyScale { strength: 1.0, km_per_cell: KM_PER_CELL, px_km: cells_per_px * KM_PER_CELL, light: (-std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2) }
}

/// Mean and standard deviation of `canopy_relief` over an `n x n` patch of
/// output pixels `cells_per_px` cells apart.
fn spread(cells_per_px: f64, n: usize) -> (f64, f64) {
    let cs = scale(cells_per_px);
    let (mut s, mut s2) = (0.0, 0.0);
    for j in 0..n {
        for i in 0..n {
            let v = render::canopy_relief(300.0 + i as f64 * cells_per_px, 200.0 + j as f64 * cells_per_px, cs);
            s += v;
            s2 += v * v;
        }
    }
    let m = s / (n * n) as f64;
    (m, (s2 / (n * n) as f64 - m * m).sqrt())
}

/// Protects: `CANOPY_NORM` and `CANOPY_BILLOW_MEAN`'s measured values -- at
/// one pixel per cell on the app's default scale the texture is zero-mean
/// with a spread of about one half, so `CANOPY_K` means what its doc says.
/// Prints the measurement both constants rest on.
#[test]
fn canopy_relief_has_unit_scale_spread() {
    let (m, sd) = spread(1.0, 512);
    println!("canopy_relief at 1 px/cell: mean {m:.4} sd {sd:.4}");
    assert!(m.abs() < 0.08, "the texture is not zero-mean: {m}");
    assert!((0.30..0.60).contains(&sd), "spread {sd} outside 0.30..0.60");
}

/// Protects: the zoom law. Each smaller footprint draws every octave the
/// larger one drew plus finer ones (so the texture keeps changing as the view
/// zooms in), a footprint too coarse for the coarsest octave draws nothing,
/// and below the crown-scale floor (`CANOPY_MIN_KM`, 8 m) nothing more
/// arrives. Literal footprints, worked by hand from the constants' documented
/// values: at 2 m and 1 m per pixel every octave down to 4 km / 2^8 = 15.6 m
/// is fully visible (`2.5 px · 2 m · 2 = 10 m < 15.6 m`) and 4 km / 2^9 =
/// 7.8 m is below the 8 m floor, so the two must be identical.
#[test]
fn canopy_gains_octaves_with_zoom_and_stops_at_crown_scale() {
    let at = |px_km: f64, x: f64, y: f64| render::canopy_relief(x, y, CanopyScale { px_km, ..scale(1.0) });
    // Too coarse for the 4 km octave: 4 km / 2.5 px = 1.6 km per px or more.
    for k in 0..50 {
        let (x, y) = (k as f64 * 3.7, k as f64 * 1.3);
        assert_eq!(at(2.0, x, y), 0.0, "a 2 km pixel cannot resolve a 4 km octave");
    }
    // Below the floor: identical.
    for k in 0..200 {
        let (x, y) = (100.0 + k as f64 * 0.013, 50.0 + k as f64 * 0.007);
        assert_eq!(at(0.002, x, y), at(0.001, x, y), "an octave finer than the 8 m floor was drawn");
    }
    // Every halving of the footprint above the floor adds detail: the
    // difference between neighbouring footprints is never zero over a patch.
    let mut px = 0.8;
    while px > 0.004 {
        let mut diff = 0.0;
        for k in 0..400 {
            let (x, y) = (200.0 + (k % 20) as f64 * px / KM_PER_CELL, 300.0 + (k / 20) as f64 * px / KM_PER_CELL);
            diff += (at(px, x, y) - at(px * 2.0, x, y)).abs();
        }
        assert!(diff > 1e-3, "halving the footprint from {} km added nothing", px * 2.0);
        px *= 0.5;
    }
}

/// Protects: the colour step's hue ordering and its scale -- crowns lift red
/// most and blue least (`1.10 : 0.95 : 0.80` of `CANOPY_K = 0.55` per unit of
/// shaped relief), gaps sink the same way, and strength `0` is the identity.
/// The ratios are the literals, not the code's constants.
#[test]
fn apply_canopy_shifts_hue_in_the_stated_ratio() {
    let c = (60.0, 120.0, 50.0);
    let cs = scale(1.0);
    let mut seen = 0;
    for k in 0..400 {
        let (x, y) = (k as f64 * 0.73, k as f64 * 0.31);
        let out = render::apply_canopy(c, cs, x, y);
        let (fr, fg, fb) = (out.0 / c.0 - 1.0, out.1 / c.1 - 1.0, out.2 / c.2 - 1.0);
        if fr.abs() < 1e-6 {
            continue;
        }
        seen += 1;
        assert!((fg / fr - 0.95 / 1.10).abs() < 1e-9 && (fb / fr - 0.80 / 1.10).abs() < 1e-9, "channel ratio moved: {fr} {fg} {fb}");
        // Red moves by exactly 0.55 · shaped relief · 1.10, the shaping being
        // `sign(d)·|d|^0.8` of the relief clamped to ±1.5 -- pins `CANOPY_K`,
        // the clamp and the exponent as literals.
        let d = render::canopy_relief(x, y, cs).clamp(-1.5, 1.5);
        let want = 0.55 * d.signum() * d.abs().powf(0.8) * 1.10;
        assert!((fr - want).abs() < 1e-9, "red moved {fr}, the stated step is {want}");
        // |s| <= 0.55 · 1.5^0.8 · 1.10 on red.
        assert!(fr.abs() <= 0.55 * 1.5f64.powf(0.8) * 1.10 + 1e-9, "red moved {fr}");
        assert!(out.0 > 0.0 && out.1 > 0.0 && out.2 > 0.0);
    }
    assert!(seen > 300, "the texture was flat over the patch");
    let off = CanopyScale { strength: 0.0, ..cs };
    assert_eq!(render::apply_canopy(c, off, 3.0, 4.0), c);
}

/// The reference's three jitter expressions, written out (reference HTML
/// 7715-7719 and `landColorCore`'s `nLow`/`nHi`), not called through
/// `land_grain`.
fn reference_grain(x: f64, y: f64, gw: usize) -> (f64, f64, f64) {
    let g = gw as f64;
    (
        vnoise(x * 0.06, y * 0.06, 11),
        vnoise(x * 96.0 / g, y * 96.0 / g, 23),
        0.6 * vnoise(x / g * 44.0, y / g * 44.0, 31) + 0.4 * vnoise(x / g * 150.0, y / g * 150.0, 33),
    )
}

/// Protects: `land_grain_warp = 0` is the reference's jitter bit for bit (the
/// `js_reference()` path), and at `1` every sample is a different number at
/// almost every point -- the fix really is on in the shipped look.
#[test]
fn land_grain_is_the_reference_at_zero_and_moves_at_one() {
    let gw = 2048;
    let off = TerrainAppearance { land_grain_warp: 0.0, ..TerrainAppearance::default() };
    let on = TerrainAppearance::default();
    let mut moved = 0;
    for k in 0..500 {
        let (x, y) = (k as f64 * 3.17 + 0.5, k as f64 * 1.91 + 0.25);
        let r = reference_grain(x, y, gw);
        let a = render::land_grain(&off, x, y, gw);
        assert_eq!((a.0.to_bits(), a.1.to_bits(), a.2.to_bits()), (r.0.to_bits(), r.1.to_bits(), r.2.to_bits()), "warp 0 is not the reference at ({x}, {y})");
        let b = render::land_grain(&on, x, y, gw);
        if b.0 != r.0 && b.1 != r.1 && b.2 != r.2 {
            moved += 1;
        }
        for v in [b.0, b.1, b.2] {
            assert!((0.0..=1.0).contains(&v), "sample {v} outside value noise's range");
        }
    }
    assert!(moved > 480, "only {moved}/500 points moved under the warp");
}

/// Gradient-energy ratio of a field: energy within 10 degrees of the grid
/// axes over energy within 10 degrees of the diagonals. Isotropic noise reads
/// about 1; value noise on the grid's own lattice reads well above it.
fn axis_ratio(f: &[f64], n: usize) -> f64 {
    let (mut ax, mut dg) = (0.0, 0.0);
    for y in 1..n - 1 {
        for x in 1..n - 1 {
            let gx = (f[y * n + x + 1] - f[y * n + x - 1]) * 0.5;
            let gy = (f[(y + 1) * n + x] - f[(y - 1) * n + x]) * 0.5;
            let e = gx * gx + gy * gy;
            let th = gy.atan2(gx).to_degrees().rem_euclid(90.0);
            if !(10.0..=80.0).contains(&th) {
                ax += e;
            } else if (th - 45.0).abs() < 10.0 {
                dg += e;
            }
        }
    }
    ax / dg
}

/// Protects: the fix itself -- the mottle's finest lattice (`bio_jitter`'s
/// 150-per-width octave, 13.7 cells at 2048, the one the assessment's square
/// size matched) is axis-aligned under the reference and not under the warp.
/// Measured on the sample, so a rotation mutated back to `(1, 0)` fails here.
/// Bounds: on this 256-cell window the reference sample reads 2.54 and the
/// warped one 0.55 (printed; 2026-10-18); isotropic is 1.0, and the rotation
/// leans the warped lattice's remaining edges toward the diagonals.
#[test]
fn the_warped_lattice_is_not_axis_aligned() {
    let (gw, n) = (2048, 256);
    let field = |a: &TerrainAppearance| -> Vec<f64> {
        let mut f = vec![0.0; n * n];
        for y in 0..n {
            for x in 0..n {
                // `n_bio` minus its broad octave's share is dominated by the
                // 13.7-cell lattice at this window size; the whole sample is
                // what the colour sees, so measure the whole sample.
                f[y * n + x] = render::land_grain(a, x as f64 + 100.0, y as f64 + 100.0, gw).2;
            }
        }
        f
    };
    let old = axis_ratio(&field(&TerrainAppearance { land_grain_warp: 0.0, ..TerrainAppearance::default() }), n);
    let new = axis_ratio(&field(&TerrainAppearance::default()), n);
    println!("n_bio axis ratio: reference {old:.3}, warped {new:.3}");
    assert!(old > 1.3, "the reference lattice should read axis-aligned ({old})");
    assert!(new < 1.15, "the warped lattice still reads axis-aligned ({new})");
}

/// Protects: the shipped values, as literals -- the mottle fix on in
/// `default()` and so in every look built on it, off on the reference path;
/// the canopy texture on in Satellite relief only.
#[test]
fn the_shipped_values_are_the_stated_ones() {
    assert_eq!(TerrainAppearance::default().land_grain_warp, 1.0);
    assert_eq!(TerrainAppearance::js_reference().land_grain_warp, 0.0);
    for look in [render::LOOK_TIER, render::LOOK_VIBRANT, render::LOOK_ANTIQUE, render::LOOK_SATELLITE] {
        assert_eq!(TerrainAppearance::default().with_look(look).land_grain_warp, 1.0, "{look}");
    }
    assert_eq!(TerrainAppearance::default().canopy_texture, 0.0);
    assert_eq!(TerrainAppearance::js_reference().canopy_texture, 0.0);
    for look in [render::LOOK_TIER, render::LOOK_VIBRANT, render::LOOK_ANTIQUE] {
        assert_eq!(TerrainAppearance::default().with_look(look).canopy_texture, 0.0, "{look}");
    }
    assert_eq!(TerrainAppearance::default().with_look(render::LOOK_SATELLITE).canopy_texture, 1.0);
}

/// Protects: `canopy_texture` serializes only when on (so every look that
/// leaves it at `0.0` keeps its serialized form and tile-cache fingerprint),
/// `land_grain_warp` always serializes (its `0.0` on the reference path must
/// survive a round trip rather than read back as the `1.0` default), and an
/// older saved look without the key loads with the fix on.
#[test]
fn the_new_fields_serialize_as_stated() {
    let d = serde_json::to_string(&TerrainAppearance::default()).unwrap();
    assert!(!d.contains("\"canopy_texture\""), "default must not serialize canopy_texture");
    assert!(d.contains("\"land_grain_warp\":1.0"));
    let r = serde_json::to_string(&TerrainAppearance::js_reference()).unwrap();
    let back: TerrainAppearance = serde_json::from_str(&r).unwrap();
    assert_eq!(back.land_grain_warp, 0.0);
    let s = serde_json::to_string(&TerrainAppearance::default().with_look(render::LOOK_SATELLITE)).unwrap();
    let back: TerrainAppearance = serde_json::from_str(&s).unwrap();
    assert_eq!(back.canopy_texture, 1.0);
    let old = d.replace("\"land_grain_warp\":1.0,", "");
    assert!(!old.contains("land_grain_warp"));
    let back: TerrainAppearance = serde_json::from_str(&old).unwrap();
    assert_eq!(back.land_grain_warp, 1.0, "an older look loads with the fix on");
}

const GW: usize = 64;
const GH: usize = 48;
const SEA: f64 = 0.42;
/// 25 km over 64 cells: the app's default 0.39 km per cell.
const MAP_KM: f64 = 25.0;

/// The fixture: a gently tilted land block, temperate and wet in the west
/// (x < 32, forest) and a -20 C snowfield in the east, so one render holds
/// forest and ground the canopy must not touch (no canopy under full snow:
/// `material_weights` gives it no budget below -5 C, jitter included).
struct World {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

fn world() -> World {
    let n = GW * GH;
    let (mut field, mut temp, mut rain) = (vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            field[i] = (0.50 + 0.001 * x as f64 + 0.0007 * y as f64) as f32;
            let forest = x < 32;
            temp[i] = if forest { 14.0 } else { -20.0 };
            rain[i] = if forest { 0.85 } else { 0.5 };
        }
    }
    World { field, temp, rain }
}

fn ctx<'a>(w: &'a World, a: TerrainAppearance, scale: bool) -> RenderCtx<'a> {
    let c = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, None, GW, GH, SEA, false, 55.0, 5.0, a);
    if scale { c.with_map_scale(MAP_KM) } else { c }
}

fn screen(c: &RenderCtx) -> Vec<u8> {
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(c, x, y);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    assert!(out.iter().any(|&v| v != 0), "an all-black render proves nothing");
    out
}

/// The shipped default with its sheet stages off (frame, paper, edge haze),
/// which on a 64 x 48 fixture would cover most of the map and hide what is
/// under test.
fn plain() -> TerrainAppearance {
    TerrainAppearance { border_width_frac: 0.0, paper_strength: 0.0, haze_strength: 0.0, ..TerrainAppearance::default() }
}

fn canopy_on() -> TerrainAppearance {
    TerrainAppearance { canopy_texture: 1.0, ..plain() }
}

/// Protects: the fixture really has forest in the west and none in the east,
/// by `material_weights` itself -- otherwise the tests below prove nothing.
#[test]
fn the_fixture_has_forest_and_snow() {
    let wf = render::material_weights(14.0, 0.85, 0.001, 0.15, 0.0, 0.0, 0.0, 0.0);
    // The east at its warmest after the land jitter (+3.5 C, +1.25 C).
    let wd = render::material_weights(-15.25, 0.6, 0.001, 0.15, 0.0, 0.0, 0.0, 0.0);
    assert!(wf.canopy > 0.4, "west canopy {}", wf.canopy);
    assert_eq!(wd.canopy, 0.0, "east canopy {}", wd.canopy);
}

/// Protects: the texture is off by control flow at `0.0`, and also with the
/// field on but no map width (no kilometre lattice on a guessed scale): both
/// render byte-identical to the shipped default. On with a width, it moves
/// the forest and never the snowfield.
#[test]
fn canopy_is_off_without_strength_or_scale_and_touches_only_forest() {
    let w = world();
    let base = screen(&ctx(&w, plain(), true));
    assert_eq!(screen(&ctx(&w, canopy_on(), false)), base, "no map width must mean no texture");
    assert_eq!(screen(&ctx(&w, plain(), false)), base);
    let on = screen(&ctx(&w, canopy_on(), true));
    let (mut west, mut east) = (0, 0);
    for y in 0..GH {
        for x in 0..GW {
            let o = (y * GW + x) * 3;
            let d = (0..3).map(|k| on[o + k].abs_diff(base[o + k])).max().unwrap();
            if x < 30 && d > 0 {
                west += 1;
            } else if x > 33 && d > 0 {
                east += 1;
            }
        }
    }
    assert!(west > 30 * GH / 2, "the forest barely moved ({west} px)");
    assert_eq!(east, 0, "the canopy texture reached the snowfield");
    // The reference path stays the reference.
    assert_eq!(screen(&ctx(&w, TerrainAppearance::js_reference(), true)), screen(&ctx(&w, TerrainAppearance::js_reference(), false)));
}

/// Protects: the export draws the screen's canopy at one pixel per cell (the
/// prologue's f32 bound, as `bake_raster.rs` documents), and the export at
/// half a cell per pixel draws more than the screen does (a finer octave
/// arrives) -- the three paths share one function, keyed to each one's own
/// footprint.
#[test]
fn the_export_agrees_with_the_screen_and_adds_octaves_when_finer() {
    let w = world();
    let c = ctx(&w, canopy_on(), true);
    let bf = BakeFields::new(&c);
    let mut worst: f64 = 0.0;
    let mut finer = 0;
    for y in 2..GH - 2 {
        for x in 2..30 {
            let s = render::cell_color(&c, x, y);
            let e = bf.pixel(&c, x as f64, y as f64);
            worst = worst.max((s.0 - e.0).abs()).max((s.1 - e.1).abs()).max((s.2 - e.2).abs());
            let h = bf.pixel_at(&c, x as f64, y as f64, (0.5, 0.5), None);
            if (h.0 - e.0).abs() > 1e-6 {
                finer += 1;
            }
        }
    }
    assert!(worst < 1.0 / 255.0, "export vs screen worst {worst}");
    assert!(finer > 28 * (GH - 4) / 2, "a half-cell export added no octave ({finer} px)");
}

/// Protects: a deep-zoom tile at one pixel per cell draws the screen's canopy
/// within one quantization level (`exag 0`, as `satellite_relief.rs` holds the
/// meso shade equal), and the texture reaches the tile at all.
#[test]
fn a_tile_agrees_with_the_screen_under_the_canopy() {
    let w = world();
    let a = TerrainAppearance { exag: 0.0, ..canopy_on() };
    let c = ctx(&w, a.clone(), true);
    let mut finished = screen(&c);
    let infl = render::build_grade_influence(&c, GW, GH);
    render::finish_raster(&a, &mut finished, GW, GH, false, &infl, render::ColorSpace::Srgb);
    let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
    let bounds = TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 };
    let tile = render::render_biome_tile_rgba(&c, &w.field, GW, GH, bounds, &tf);
    assert_eq!(tile.len(), GW * GH * 4);
    let mut worst = 0u8;
    for (k, px) in tile.chunks(4).enumerate() {
        for ch in 0..3 {
            worst = worst.max(px[ch].abs_diff(finished[k * 3 + ch]));
        }
    }
    assert!(worst <= 1, "tile vs screen worst {worst} levels");
    let a0 = TerrainAppearance { exag: 0.0, ..plain() };
    let c0 = ctx(&w, a0, true);
    let tf0 = TileFields::new(&c0, Some(&screen(&c0))).without_lakes();
    assert_ne!(tile, render::render_biome_tile_rgba(&c0, &w.field, GW, GH, bounds, &tf0), "the canopy must reach the tile");
}

/// Protects: `coarse_grad_f` -- the grid-scale gradient a kilometre-law
/// tile's materials and snow facing read -- at an integer position is the
/// plain central difference of the coarse field over one cell either side,
/// halved, worked here by hand from the fixture's own heights (0.001 per cell
/// in x, 0.0007 in y), and at a half-cell it is the same plane's gradient
/// (bilinear sampling of a plane).
#[test]
fn coarse_grad_is_the_grid_central_difference() {
    let w = world();
    let c = ctx(&w, canopy_on(), true);
    for (x, y) in [(10.0, 10.0), (20.0, 30.0), (40.5, 12.5)] {
        let (gx, gy) = render::coarse_grad_f(&c, x, y);
        let got = gx.hypot(gy);
        let want = (0.001f64).hypot(0.0007);
        assert!((gx - 0.001).abs() < 1e-6 && (gy - 0.0007).abs() < 1e-6, "gradient at ({x}, {y}) is ({gx}, {gy}), the plane's is (0.001, 0.0007)");
        assert!((got - want).abs() < 1e-6, "slope at ({x}, {y}) is {got}, the plane's is {want}");
    }
}

/// The 8x-magnified tile over the fixture's forest half that the test below
/// renders: the coarse plane plus `amp` times a steep ripple of period four
/// pixels (`sin(i·π/2)·cos(j·π/2)`, values `0` and `±1` only, so exactly
/// zero-mean over the tile).
fn ripple_tile(amp: f64) -> (Vec<f32>, usize, usize, TileBounds) {
    let (tw, th) = (64usize, 64usize);
    let bounds = TileBounds { x: 8.0, y: 16.0, w: 8.0, h: 8.0 };
    let mut tile = vec![0f32; tw * th];
    for j in 0..th {
        for i in 0..tw {
            let (gx, gy) = (bounds.x + i as f64 / (tw - 1) as f64 * bounds.w, bounds.y + j as f64 / (th - 1) as f64 * bounds.h);
            let plane = 0.50 + 0.001 * gx + 0.0007 * gy;
            let ripple = amp * ((i as f64) * std::f64::consts::PI * 0.5).sin() * ((j as f64) * std::f64::consts::PI * 0.5).cos();
            tile[j * tw + i] = (plane + ripple) as f32;
        }
    }
    (tile, tw, th, bounds)
}

/// `(mean |rippled - flat| per colour byte, the largest per-channel shift of
/// the tile's mean colour)` for one appearance on [`ripple_tile`]: the first
/// is texture, the second a change of what the stand is on average.
fn ripple_effect(a: TerrainAppearance) -> (f64, f64) {
    let w = world();
    let (flat, tw, th, bounds) = ripple_tile(0.0);
    let (rippled, _, _, _) = ripple_tile(0.004);
    let c = ctx(&w, a, true);
    let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
    let p0 = render::render_biome_tile_rgba(&c, &flat, tw, th, bounds, &tf);
    let p1 = render::render_biome_tile_rgba(&c, &rippled, tw, th, bounds, &tf);
    assert_eq!(p0.len(), tw * th * 4);
    let n = (tw * th) as f64;
    let per_px = p0.iter().zip(&p1).enumerate().filter(|(k, _)| k % 4 != 3).map(|(_, (a, b))| a.abs_diff(*b) as f64).sum::<f64>() / (3.0 * n);
    let mean = |p: &[u8], ch: usize| p.chunks(4).map(|q| q[ch] as f64).sum::<f64>() / n;
    let shift = (0..3).map(|ch| (mean(&p1, ch) - mean(&p0, ch)).abs()).fold(0.0, f64::max);
    (per_px, shift)
}

/// Protects: **category from the grid, texture from the tile** (Ruling CI
/// follow-up, 2026-10-18), in the shipped default look and in the canopy
/// look alike, under the kilometre zoom-detail law. Sub-cell relief far past
/// `material_weights`' rock knee (0.004 height units over a quarter of a
/// cell) must not change what the ground IS: the rippled tile's mean colour
/// stays within half a level of the flat one's -- while the relief DOES
/// arrive, as tone (at least two levels per pixel on average). The control is
/// the reference schedule (`zoom_detail_km` 0, the tile's own slope feeding
/// the materials), where the same ripple reads as rock and moves the mean by
/// more than three levels, so the ripple is steep enough to be a test.
/// Shading (`exag 0`) and the LOD-D5 micro band (`detail_scale_strength 0`)
/// are off, so only the materials and the tone can tell the tiles apart.
/// Replaces `canopy_tile_materials_ignore_sub_cell_relief` (`de1a07ff`, canopy
/// look only), whose per-pixel bound the tone now exceeds by design.
#[test]
fn km_tile_materials_ignore_sub_cell_relief_in_every_look() {
    let quiet = |a: TerrainAppearance| TerrainAppearance { exag: 0.0, detail_scale_strength: 0.0, ..a };
    for (name, a) in [("default", plain()), ("canopy", canopy_on())] {
        assert!(a.zoom_detail_km > 0.0, "{name}: the fixture must be a kilometre-law tile");
        let (tex, shift) = ripple_effect(quiet(a.clone()));
        println!("{name}: km law: per-pixel {tex:.3}, mean shift {shift:.3} levels");
        assert!(shift < 0.5, "{name}: sub-cell relief moved the stand's mean colour by {shift:.3} levels");
        assert!(tex > 2.0, "{name}: the sub-cell relief did not arrive as texture ({tex:.3} levels per pixel)");
        let (_, ctl) = ripple_effect(quiet(TerrainAppearance { zoom_detail_km: 0.0, ..a }));
        println!("{name}: reference schedule: mean shift {ctl:.3} levels");
        assert!(ctl > 3.0, "{name}: the ripple is not steep enough to be a test ({ctl:.3} levels)");
    }
}

/// Protects: `micro_relief_tone`'s law against numbers worked by hand: zero
/// for no relief, a degenerate scale or a non-finite input; positive on
/// convex ground (a negative Laplacian), negative in hollows; exactly
/// `tanh(1)` where the normalised curvature equals the literal `0.04`
/// (`MICRO_TONE_REF`); never outside `[-1, 1]`. And its zoom law: under the
/// `H = 1` spectrum one octave at a fixed pixel wavelength has a pixel
/// Laplacian proportional to cells per pixel, so it gives the same tone at
/// any zoom; detail at a fixed GROUND wavelength (2 cells, amplitude 0.002,
/// read by a one-pixel stencil) loses tone as the view comes in, so it fades
/// rather than being magnified into blotches.
#[test]
fn micro_relief_tone_is_signed_bounded_and_keyed_to_zoom() {
    assert_eq!(render::micro_relief_tone(0.0, 0.5), 0.0);
    assert_eq!(render::micro_relief_tone(-1.0, 0.0), 0.0);
    assert_eq!(render::micro_relief_tone(f64::NAN, 0.5), 0.0);
    assert!(render::micro_relief_tone(-0.01, 0.5) > 0.0, "convex must be positive");
    assert!(render::micro_relief_tone(0.01, 0.5) < 0.0, "concave must be negative");
    for cpp in [0.5, 0.125, 0.03] {
        let g = render::micro_relief_tone(-0.04 * cpp, cpp);
        assert!((g - 1f64.tanh()).abs() < 1e-12, "at the reference scale the tone is tanh(1), got {g} at {cpp}");
        assert!(render::micro_relief_tone(-1e6, cpp) <= 1.0 && render::micro_relief_tone(1e6, cpp) >= -1.0);
    }
    let octave = |cpp: f64| render::micro_relief_tone(-0.05 * cpp, cpp);
    assert!((octave(0.5) - octave(0.03)).abs() < 1e-12, "one H = 1 octave must read the same at every zoom");
    // The one-pixel Laplacian of A·cos(2πx/L) at its crest is
    // A·(2cos(2π·cpp/L) - 2), negative (convex).
    let fixed = |cpp: f64| render::micro_relief_tone(0.002 * (2.0 * (2.0 * std::f64::consts::PI * cpp / 2.0).cos() - 2.0), cpp);
    let (far, near) = (fixed(0.25), fixed(0.03));
    println!("fixed-ground detail tone: {far:.4} at 0.25 cells/px, {near:.4} at 0.03");
    assert!(far > 0.0 && near > 0.0 && near < 0.5 * far, "fixed-ground detail must fade as the view comes in: {far} -> {near}");
}

/// Protects: `apply_micro_tone` against its stated literals -- `0.35` of the
/// way toward the rock colour and `8 %` of value per unit tone -- and its
/// linearity: `+g` and `-g` average back to the input exactly (a zero-mean
/// tone keeps a stand's mean colour), `g = 0` is the identity, and on bare
/// rock only the value half acts.
#[test]
fn apply_micro_tone_is_a_linear_bounded_tint() {
    let (c, rock) = ((60.0, 120.0, 50.0), (140.0, 130.0, 120.0));
    assert_eq!(render::apply_micro_tone(c, rock, 0.0), c);
    for g in [0.3, 0.75, 1.0] {
        let up = render::apply_micro_tone(c, rock, g);
        let dn = render::apply_micro_tone(c, rock, -g);
        let want = (c.0 + 0.35 * g * (rock.0 - c.0) + 0.08 * g * c.0, c.1 + 0.35 * g * (rock.1 - c.1) + 0.08 * g * c.1, c.2 + 0.35 * g * (rock.2 - c.2) + 0.08 * g * c.2);
        assert!((up.0 - want.0).abs() < 1e-9 && (up.1 - want.1).abs() < 1e-9 && (up.2 - want.2).abs() < 1e-9, "{up:?} vs {want:?}");
        assert!(((up.0 + dn.0) / 2.0 - c.0).abs() < 1e-9 && ((up.1 + dn.1) / 2.0 - c.1).abs() < 1e-9 && ((up.2 + dn.2) / 2.0 - c.2).abs() < 1e-9);
        let on_rock = render::apply_micro_tone(rock, rock, g);
        assert!((on_rock.0 - rock.0 * (1.0 + 0.08 * g)).abs() < 1e-9);
    }
}

/// Protects: `tile_micro_tone` reads only the relief the grid does NOT
/// carry. Over a curved coarse world (a bowl, `0.005·r²` in cells), a tile whose heights ARE the bilinear coarse field has
/// a large Laplacian of its own but a tone of zero, because the coarse
/// field's Laplacian at the same offsets cancels it; the same tile plus a
/// one-pixel bump on top has a positive (convex) tone there; and the
/// buffer's outermost pixel reads zero whatever the heights (the
/// unpadded-tile rule).
#[test]
fn tile_micro_tone_cancels_the_coarse_curvature() {
    let n = GW * GH;
    let (mut field, temp, rain) = (vec![0f32; n], vec![14f32; n], vec![0.85f32; n]);
    for y in 0..GH {
        for x in 0..GW {
            let (dx, dy) = (x as f64 - 32.0, y as f64 - 24.0);
            field[y * GW + x] = (0.5 + 0.005 * (dx * dx + dy * dy)) as f32;
        }
    }
    let c = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, plain()).with_map_scale(MAP_KM);
    // A 2x-magnified 33 x 33 buffer over cells 10..26 x 10..26, heights from
    // the coarse field itself (bilinear at each pixel's grid position).
    let (pw, ph, cxy) = (33usize, 33usize, 0.5f64);
    let at = |i: usize, j: usize| {
        let (gx, gy) = (10.0 + i as f64 * cxy, 10.0 + j as f64 * cxy);
        let (x0, y0) = (gx.floor() as usize, gy.floor() as usize);
        let (tx, ty) = (gx - x0 as f64, gy - y0 as f64);
        let f = |x: usize, y: usize| field[y * GW + x] as f64;
        (f(x0, y0) * (1.0 - tx) + f(x0 + 1, y0) * tx) * (1.0 - ty) + (f(x0, y0 + 1) * (1.0 - tx) + f(x0 + 1, y0 + 1) * tx) * ty
    };
    let mut tile: Vec<f32> = (0..pw * ph).map(|k| at(k % pw, k / pw) as f32).collect();
    let tone = |t: &[f32], i: usize, j: usize| render::tile_micro_tone(&c, t, pw, ph, i, j, 10.0 + i as f64 * cxy, 10.0 + j as f64 * cxy, cxy, cxy);
    let mut worst: f64 = 0.0;
    for j in 1..ph - 1 {
        for i in 1..pw - 1 {
            worst = worst.max(tone(&tile, i, j).abs());
        }
    }
    // The coarse bowl alone, uncancelled, reads tanh(-(4 * 0.005 * 0.5^2) /
    // 0.5 / 0.04) = -0.24 on this grid; the f32 heights leave far less.
    assert!(worst < 0.01, "the coarse field's own curvature leaked into the tone: {worst}");
    tile[16 * pw + 16] += 0.002;
    assert!(tone(&tile, 16, 16) > 0.3, "a convex bump must read as positive tone: {}", tone(&tile, 16, 16));
    assert_eq!(tone(&tile, 0, 16), 0.0);
    assert_eq!(tone(&tile, 16, ph - 1), 0.0);
}

/// Protects: under the kilometre law the snow line's facing (Ruling AP's
/// `snow_aspect_c`) is a material decision read from the grid's gradient,
/// not the tile's: sub-cell ripples must not flip snow on and off pixel by
/// pixel. A -1 C world (mid snow ramp) on a plane rising 0.01 per cell to
/// the south, an 8x tile carrying a ripple along y (`0.004·sin(j·π/2)`).
/// Rows `j ≡ 0` and `j ≡ 2 (mod 4)` have the same height, zero curvature
/// (so zero tone) and OPPOSITE tile gradients (+ and -), so with the facing
/// taken from the grid they draw the same colour; with the tile's facing
/// they would sit on opposite sides of the snow line. Light off (`exag 0`)
/// and the LOD-D5 micro band off, so only the materials can differ.
#[test]
fn km_tile_snow_facing_reads_the_grid_gradient() {
    let n = GW * GH;
    let (mut field, temp, rain) = (vec![0f32; n], vec![-1f32; n], vec![0.5f32; n]);
    for y in 0..GH {
        for x in 0..GW {
            field[y * GW + x] = (0.5 + 0.01 * y as f64) as f32;
        }
    }
    let a = TerrainAppearance { exag: 0.0, detail_scale_strength: 0.0, ..plain() };
    assert!(a.snow_aspect_c > 0.0, "the shipped look must carry the snow facing for this to test anything");
    let c = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, a).with_map_scale(MAP_KM);
    let (tw, th) = (64usize, 64usize);
    let bounds = TileBounds { x: 20.0, y: 10.0, w: 8.0, h: 8.0 };
    let mut tile = vec![0f32; tw * th];
    for j in 0..th {
        for i in 0..tw {
            let gy = bounds.y + j as f64 / (th - 1) as f64 * bounds.h;
            tile[j * tw + i] = (0.5 + 0.01 * gy + 0.004 * (j as f64 * std::f64::consts::PI * 0.5).sin()) as f32;
        }
    }
    let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
    let p = render::render_biome_tile_rgba(&c, &tile, tw, th, bounds, &tf);
    assert_eq!(p.len(), tw * th * 4);
    // Mean luma of the interior rows j = 4k (rising) and j = 4k + 2 (falling).
    let row_luma = |m: usize| {
        let (mut s, mut k) = (0.0, 0.0);
        for j in (4..th - 4).filter(|j| j % 4 == m) {
            for i in 2..tw - 2 {
                let q = &p[(j * tw + i) * 4..];
                s += 0.2126 * q[0] as f64 + 0.7152 * q[1] as f64 + 0.0722 * q[2] as f64;
                k += 1.0;
            }
        }
        s / k
    };
    let (rise, fall) = (row_luma(0), row_luma(2));
    println!("rows with a rising tile gradient {rise:.2}, falling {fall:.2}");
    assert!((rise - fall).abs() < 1.0, "the snow facing followed the sub-cell ripple: {rise:.2} vs {fall:.2}");
}
