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
//!   so at one pixel per cell all three agree.
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

/// Protects: `coarse_slope_f` -- the grid-scale slope the canopy look's tile
/// materials read -- at an integer position is the plain central difference
/// of the coarse field over one cell either side, halved, worked here by hand
/// from the fixture's own heights (0.001 per cell in x, 0.0007 in y), and at
/// a half-cell it is the same plane's slope (bilinear sampling of a plane).
#[test]
fn coarse_slope_is_the_grid_central_difference() {
    let w = world();
    let c = ctx(&w, canopy_on(), true);
    let want = (0.001f64).hypot(0.0007);
    for (x, y) in [(10.0, 10.0), (20.0, 30.0), (40.5, 12.5)] {
        let got = render::coarse_slope_f(&c, x, y);
        assert!((got - want).abs() < 1e-6, "slope at ({x}, {y}) is {got}, the plane's is {want}");
    }
}

/// Protects: in the canopy look, a deep-zoom tile's MATERIALS read the
/// grid-scale slope, so sub-cell relief (what the kilometre zoom-detail law
/// adds) cannot turn a forest into rock as the view zooms in -- the measured
/// failure behind `coarse_slope_f`. A tile 8x magnified over the forest half,
/// rendered twice: its heights the coarse plane, and the plane plus a steep
/// ripple (0.004 height units over an eighth of a cell, far past
/// `material_weights`' rock knee). Shading is off (`exag 0`) and so is the
/// LOD-D5 micro band (`detail_scale_strength 0`), so only the materials can
/// tell the two apart: under the canopy look they agree to a fraction of a
/// level; with the canopy off (the tile-slope path) the ripple reads as rock
/// and the two differ by many levels -- the control that shows the ripple is
/// steep enough to be a test.
#[test]
fn canopy_tile_materials_ignore_sub_cell_relief() {
    let w = world();
    let (tw, th) = (64usize, 64usize);
    let bounds = TileBounds { x: 8.0, y: 16.0, w: 8.0, h: 8.0 };
    let heights = |amp: f64| -> Vec<f32> {
        let mut tile = vec![0f32; tw * th];
        for j in 0..th {
            for i in 0..tw {
                let (gx, gy) = (bounds.x + i as f64 / (tw - 1) as f64 * bounds.w, bounds.y + j as f64 / (th - 1) as f64 * bounds.h);
                let plane = 0.50 + 0.001 * gx + 0.0007 * gy;
                let ripple = amp * ((i as f64) * std::f64::consts::PI * 0.5).sin() * ((j as f64) * std::f64::consts::PI * 0.5).cos();
                tile[j * tw + i] = (plane + ripple) as f32;
            }
        }
        tile
    };
    let (flat, rippled) = (heights(0.0), heights(0.004));
    let mean_diff = |a: TerrainAppearance| -> f64 {
        let c = ctx(&w, a, true);
        let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
        let p0 = render::render_biome_tile_rgba(&c, &flat, tw, th, bounds, &tf);
        let p1 = render::render_biome_tile_rgba(&c, &rippled, tw, th, bounds, &tf);
        assert_eq!(p0.len(), tw * th * 4);
        p0.iter().zip(&p1).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / p0.len() as f64
    };
    let on = mean_diff(TerrainAppearance { exag: 0.0, detail_scale_strength: 0.0, ..canopy_on() });
    let off = mean_diff(TerrainAppearance { exag: 0.0, detail_scale_strength: 0.0, ..plain() });
    println!("mean |rippled - flat| per channel: canopy look {on:.3}, tile-slope path {off:.3}");
    assert!(on < 0.5, "the canopy look's materials moved under sub-cell relief ({on:.3} levels)");
    assert!(off > 3.0, "the ripple is not steep enough to be a test ({off:.3} levels)");
}
