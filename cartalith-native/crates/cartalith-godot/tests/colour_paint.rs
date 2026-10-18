//! Ruling CN's colour-paint layer in the renderer (`COLOUR_PAINT_SCOPE.md`
//! CP-1): `cartalith_spatial::colour_paint` composited by `render.rs` into the
//! screen raster (`cell_color`), the export bake (`BakeFields` /
//! `bake_and_finish` / `bake_export_band`) and the deep-zoom tile
//! (`render_biome_tile_rgba`).
//!
//! What each test protects is on the test. The shape of the file:
//!
//! - **Identity**: no layer, a hidden layer and a layer whose marks reach
//!   nothing draw the same bytes on all three paths (the scope's bar 1; the
//!   literal-hash censuses in `export_raster.rs` hold the absent case against
//!   HEAD).
//! - **Kilometres**: a hard stroke's edge sits at the same kilometre at two
//!   tile resolutions and on two grid sizes of one map (bars 2 and 3).
//! - **Consistency**: banded export equals monolithic with marks (bar 5);
//!   two tiles agree at their shared column (bar 4).
//! - **Relief survives** every mode, because the paint goes in before the
//!   light (bar 6) -- the owner's "paint the inside of a volcano red".
//! - **Water**: `land_only` per mark (the crater-lake case).
//!
//! Compiled standalone through the `#[path]` trick `tests/export_bands.rs`
//! uses, so it runs under plain `cargo test` with no Godot present.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use cartalith_spatial::colour_paint::{ColourBlend, ColourMark, ColourPaint, ColourPaintDoc, MarkShape, DOC_VERSION};
use render::{BakeFields, ExportBandPlan, ExportRivers, RenderCtx, TerrainAppearance, TileBounds, TileFields};

/// Sea level of every fixture: the project's usual 0.42, so land and sea both run.
const SEA: f64 = 0.42;

/// The fixture world at any grid size, from closed-form functions of the
/// position as a fraction of the map, so two grid sizes are the same map at
/// two resolutions (bar 3). A plateau above sea over most of the map, a round
/// sea basin centred at (0.3, 0.7) of the map (cell (18, 30) at 61 x 43,
/// clear of the plate frame's 10-cell margin), and rolling relief so the
/// light varies -- unless `flat`, where the plateau is level so the light is
/// uniform and a stroke's effect is proportional to its coverage (bars 2-3
/// read edges at half coverage).
fn fixture_with(gw: usize, gh: usize, flat: bool) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let n = gw * gh;
    let (mut field, mut temp, mut rain, mut flow) = (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for y in 0..gh {
        for x in 0..gw {
            let (u, v) = ((x as f64 + 0.5) / gw as f64, (y as f64 + 0.5) / gh as f64);
            let i = y * gw + x;
            let sea = (0.22 - ((u - 0.3).powi(2) + (v - 0.7).powi(2)).sqrt()).max(0.0) * 2.0;
            let relief = if flat { 0.0 } else { 0.08 * ((u * 9.0).sin() * (v * 7.0).cos()) + 0.04 * ((u * 23.0 + v * 17.0).sin()) };
            field[i] = (0.62 + relief - sea) as f32;
            temp[i] = (18.0 - 10.0 * v) as f32;
            rain[i] = (0.4 + 0.3 * (u * 5.0).sin()).clamp(0.0, 1.0) as f32;
            flow[i] = 1.0;
        }
    }
    (field, temp, rain, flow)
}

/// [`fixture_with`] with relief.
fn fixture(gw: usize, gh: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    fixture_with(gw, gh, false)
}

/// The map width every fixture is drawn at: 10 km a cell at 61 wide.
const KM: f64 = 610.0;

fn doc(marks: Vec<ColourMark>) -> ColourPaintDoc {
    ColourPaintDoc { version: DOC_VERSION, visible: true, marks }
}

fn line(pts: &[[f64; 2]], radius_km: f64, rgb: [u8; 3], blend: ColourBlend, hardness: f64) -> ColourMark {
    ColourMark { shape: MarkShape::Line { points_km: pts.to_vec(), radius_km }, rgb, opacity: 1.0, hardness, blend, land_only: true }
}

/// A horizontal hard stroke across the north of the map, 40 km in radius:
/// its edges are at y = 215 +- 40 km (cells 17.0..25.0 in centre
/// coordinates at 10 km a cell), clear of the plate frame and the sea.
fn band_stroke(blend: ColourBlend) -> ColourMark {
    line(&[[100.0, 215.0], [520.0, 215.0]], 40.0, [255, 0, 255], blend, 1.0)
}

fn appearance() -> TerrainAppearance {
    TerrainAppearance::default()
}

/// The whole screen raster (`cell_color`), as bytes.
fn screen(ctx: &RenderCtx) -> Vec<u8> {
    let mut out = Vec::with_capacity(ctx.gw * ctx.gh * 3);
    for y in 0..ctx.gh {
        for x in 0..ctx.gw {
            let (r, g, b) = render::cell_color(ctx, x, y);
            // `cell_color` answers on 0..1; `build_color_texture` truncates.
            out.extend([r, g, b].map(|v| (v * 255.0) as u8));
        }
    }
    assert!(out.iter().any(|&v| v != out[0]), "an empty screen render");
    out
}

/// One tile over `bounds` at `w x h` texels, with the halo the shipped tile
/// path carries (`tile_halo_px`, so no shading stencil clamps at the edge),
/// its height sampled bilinearly from the grid (no pyramid: this file tests
/// the colouriser, not the height synthesis).
fn tile(ctx: &RenderCtx, tf: &TileFields, bounds: TileBounds, w: usize, h: usize) -> Vec<u8> {
    let (cx, cy) = (bounds.w / (w - 1) as f64, bounds.h / (h - 1) as f64);
    let pad = render::tile_halo_px(ctx, w, h, bounds);
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let mut hts = vec![0f32; pw * ph];
    for y in 0..ph {
        for x in 0..pw {
            let (gx, gy) = (bounds.x + (x as f64 - pad as f64) * cx, bounds.y + (y as f64 - pad as f64) * cy);
            hts[y * pw + x] = bilinear(ctx.field, ctx.gw, ctx.gh, gx, gy) as f32;
        }
    }
    let rgba = render::render_biome_tile_rgba_padded(ctx, &hts, w, h, pad, bounds, tf);
    assert_eq!(rgba.len(), w * h * 4, "the colouriser refused the tile");
    rgba
}

fn bilinear(f: &[f32], gw: usize, gh: usize, x: f64, y: f64) -> f64 {
    let x = x.clamp(0.0, (gw - 1) as f64);
    let y = y.clamp(0.0, (gh - 1) as f64);
    let (x0, y0) = (x as usize, y as usize);
    let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
    let (tx, ty) = (x - x0 as f64, y - y0 as f64);
    let p = |x: usize, y: usize| f[y * gw + x] as f64;
    (p(x0, y0) * (1.0 - tx) + p(x1, y0) * tx) * (1.0 - ty) + (p(x0, y1) * (1.0 - tx) + p(x1, y1) * tx) * ty
}

fn export(ctx: &RenderCtx, w: usize) -> Vec<u8> {
    let (w, h) = render::bake_dims(w, ctx.gw, ctx.gh);
    let bf = BakeFields::new(ctx);
    let inf = render::build_grade_influence(ctx, w, h);
    render::bake_and_finish(ctx, &bf, ExportRivers::default(), w, h, &inf, render::ColorSpace::Srgb)
}

/// Protects bar 1: with no layer, a hidden layer, an empty layer and a
/// visible layer whose only mark is off the map, the screen, the export and a
/// tile are the same bytes. The first three by `with_colour_paint`'s inert
/// filter; the fourth by `ColourPaint::apply`'s early return.
#[test]
fn absent_hidden_empty_and_unreached_layers_draw_the_same_bytes() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let base = || RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
    let mut hidden = doc(vec![band_stroke(ColourBlend::Replace)]);
    hidden.visible = false;
    let layers = [
        ColourPaint::new(hidden, 20.0).unwrap(),
        ColourPaint::new(doc(vec![]), 20.0).unwrap(),
        ColourPaint::new(doc(vec![line(&[[5000.0, 5000.0]], 10.0, [255, 0, 0], ColourBlend::Replace, 1.0)]), 20.0).unwrap(),
    ];
    let b0 = base();
    let tf0 = TileFields::new(&b0, None);
    let tb = TileBounds { x: 10.0, y: 10.0, w: 20.0, h: 20.0 };
    let (s0, e0, t0) = (screen(&b0), export(&b0, 300), tile(&b0, &tf0, tb, 96, 96));
    for (k, l) in layers.iter().enumerate() {
        let c = base().with_colour_paint(Some(l));
        let tf = TileFields::new(&c, None);
        assert_eq!(screen(&c), s0, "layer {k}: screen moved");
        assert_eq!(export(&c, 300), e0, "layer {k}: export moved");
        assert_eq!(tile(&c, &tf, tb, 96, 96), t0, "layer {k}: tile moved");
    }
    // Load-bearing: the visible stroke does move all three.
    let l = ColourPaint::new(doc(vec![band_stroke(ColourBlend::Replace)]), 20.0).unwrap();
    let c = base().with_colour_paint(Some(&l));
    let tf = TileFields::new(&c, None);
    assert_ne!(screen(&c), s0);
    assert_ne!(export(&c, 300), e0);
    let tb2 = TileBounds { x: 20.0, y: 20.0, w: 20.0, h: 20.0 };
    assert_ne!(tile(&c, &tf, tb2, 96, 96), tile(&b0, &tf0, tb2, 96, 96));
}

/// The rows of column `x` of an RGBA (`ch = 4`) or RGB (`ch = 3`) image whose
/// colour differs from the unpainted one by more than half the stroke's
/// largest difference on that column: the painted footprint, as a row range.
fn painted_rows(p: &[u8], u: &[u8], w: usize, h: usize, ch: usize, x: usize) -> (usize, usize) {
    let e: Vec<u32> = (0..h).map(|y| (0..3).map(|c| p[(y * w + x) * ch + c].abs_diff(u[(y * w + x) * ch + c]) as u32).sum()).collect();
    let mx = *e.iter().max().unwrap();
    assert!(mx > 60, "premise: the stroke is visible on this column ({mx})");
    let first = e.iter().position(|&v| v * 2 > mx).unwrap();
    let last = e.iter().rposition(|&v| v * 2 > mx).unwrap();
    (first, last)
}

/// Protects bar 2: a hard stroke's two edges are at the same kilometre in a
/// tile drawn at 48 and at 384 texels over the same ground (8x the zoom), and
/// at the kilometre the mark says (305 +- 40 km), within one texel of the
/// coarser tile. Also that the deep tile's edge is sharp: at most two texels
/// of its 384 are in the transition (the antialiasing is one pixel wide).
#[test]
fn a_stroke_edge_is_at_the_same_kilometre_at_every_zoom() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture_with(gw, gh, true);
    let l = ColourPaint::new(doc(vec![band_stroke(ColourBlend::Replace)]), 20.0).unwrap();
    let un = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
    let pa = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM).with_colour_paint(Some(&l));
    let (tfu, tfp) = (TileFields::new(&un, None), TileFields::new(&pa, None));
    // Cells 13..29 vertically span the stroke (y 215 +- 40 km = cells
    // 17.0..25.0 in centre coordinates: km / 10 - 0.5).
    let tb = TileBounds { x: 24.0, y: 13.0, w: 16.0, h: 16.0 };
    let mut edges = vec![];
    for n in [48usize, 384] {
        let (u, p) = (tile(&un, &tfu, tb, n, n), tile(&pa, &tfp, tb, n, n));
        let (a, b) = painted_rows(&p, &u, n, n, 4, n / 2);
        let cpx = tb.h / (n - 1) as f64;
        let km = |row: usize| ((tb.y + row as f64 * cpx) + 0.5) * 10.0;
        edges.push((km(a), km(b), cpx * 10.0));
        if n == 384 {
            // Sharpness: the top edge's 10 % row and its 50 % row are at most
            // two texels apart (the light varies inside the stroke, so the
            // plateau is not flat enough for a "partial" count).
            let e: Vec<u32> = (0..n).map(|y| (0..3).map(|c| p[(y * n + n / 2) * 4 + c].abs_diff(u[(y * n + n / 2) * 4 + c]) as u32).sum()).collect();
            let mx = *e.iter().max().unwrap();
            let r10 = e.iter().position(|&v| v * 10 > mx).unwrap();
            assert!(a - r10 <= 2, "the deep tile's top edge takes {} texels from 10 % to 50 %", a - r10);
        }
    }
    println!("edges (km): {edges:?}");
    let tol = edges[0].2;
    for (lo, hi, _) in &edges {
        assert!((lo - 175.0).abs() <= tol, "top edge at {lo} km, mark says 175");
        assert!((hi - 255.0).abs() <= tol, "bottom edge at {hi} km, mark says 255");
    }
    assert!((edges[0].0 - edges[1].0).abs() <= tol && (edges[0].1 - edges[1].1).abs() <= tol);
}

/// Protects bar 3: the same kilometre stroke on a 61 x 43 and a 122 x 86 grid
/// of the same map paints the same ground: the export's painted rows, in km,
/// agree within one export pixel.
#[test]
fn the_same_kilometres_paint_the_same_ground_at_two_grid_sizes() {
    let l = ColourPaint::new(doc(vec![band_stroke(ColourBlend::Replace)]), 40.0).unwrap();
    let mut spans = vec![];
    for (gw, gh) in [(61usize, 43usize), (122, 86)] {
        let (field, temp, rain, flow) = fixture_with(gw, gh, true);
        let mk = || RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
        let (u, p) = (export(&mk(), 488), export(&mk().with_colour_paint(Some(&l)), 488));
        let (w, h) = render::bake_dims(488, gw, gh);
        let (a, b) = painted_rows(&p, &u, w, h, 3, w / 2);
        // Sharp at the export's own pixel: the top edge goes from 10 % to
        // 90 % within two export pixels (the antialiasing is one export
        // pixel wide, never one grid cell -- 4 to 8 pixels here). The
        // fixture is flat, so the painted plateau is level.
        let e: Vec<u32> = (0..h).map(|y| (0..3).map(|c| p[(y * w + w / 2) * 3 + c].abs_diff(u[(y * w + w / 2) * 3 + c]) as u32).sum()).collect();
        let mx = *e.iter().max().unwrap();
        let r10 = e.iter().position(|&v| v * 10 > mx).unwrap();
        let r90 = e.iter().position(|&v| v * 10 > mx * 9).unwrap();
        assert!(r90 - r10 <= 2, "{gw}x{gh}: the export's edge takes {} pixels from 10 % to 90 %", r90 - r10);
        // An export pixel `py` samples grid `py * (gh - 1) / (h - 1)`
        // (`bake_steps`); km is (grid + 0.5) * km per cell.
        let kpc = KM / gw as f64;
        let km = |py: usize| (py as f64 * (gh - 1) as f64 / (h - 1) as f64 + 0.5) * kpc;
        spans.push((km(a), km(b), KM / w as f64));
    }
    println!("spans (km): {spans:?}");
    let tol = spans[0].2.max(spans[1].2) * 1.5;
    assert!((spans[0].0 - spans[1].0).abs() <= tol && (spans[0].1 - spans[1].1).abs() <= tol, "{spans:?}");
    for (lo, hi, _) in &spans {
        assert!((lo - 175.0).abs() <= tol && (hi - 255.0).abs() <= tol, "{spans:?}: the mark says 175..255 km");
    }
}

/// Protects bar 5: with marks attached, the banded export is byte-identical
/// to the monolithic one, so the paint is a pure function of the pixel's
/// position and never of its band.
#[test]
fn banded_export_with_paint_equals_monolithic() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let l = ColourPaint::new(doc(vec![band_stroke(ColourBlend::Colour), line(&[[60.0, 380.0], [300.0, 420.0]], 25.0, [40, 90, 200], ColourBlend::Multiply, 0.3)]), 20.0).unwrap();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM).with_colour_paint(Some(&l));
    let (w, h) = render::bake_dims(400, gw, gh);
    let whole = export(&ctx, 400);
    let bf = BakeFields::new(&ctx);
    let cells = render::build_grade_influence_cells(&ctx);
    for rows in [h, 37, 5] {
        let plan = ExportBandPlan::with_rows(ctx.appearance(), w, h, rows);
        let mut out = vec![];
        for band in plan.bands() {
            out.extend(render::bake_export_band(&ctx, &bf, ExportRivers::default(), &plan, band, &cells, render::ColorSpace::Srgb));
        }
        assert_eq!(out, whole, "{rows} rows per band");
    }
}

/// Protects bar 4: two side-by-side tiles share their seam column (the last
/// texel of the left one and the first of the right one sample the same
/// ground), and the paint does not open a seam there: the painted pair
/// differs across the seam by no more than the unpainted pair does, plus one
/// level per channel.
#[test]
fn adjacent_tiles_agree_at_the_seam() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    // A soft diagonal stroke crossing the seam at x = 30 cells.
    let l = ColourPaint::new(doc(vec![line(&[[150.0, 200.0], [450.0, 400.0]], 50.0, [255, 30, 0], ColourBlend::Replace, 0.4)]), 20.0).unwrap();
    let un = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
    let pa = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM).with_colour_paint(Some(&l));
    let (tfu, tfp) = (TileFields::new(&un, None), TileFields::new(&pa, None));
    let n = 128;
    let (left, right) = (TileBounds { x: 18.0, y: 15.0, w: 12.0, h: 12.0 }, TileBounds { x: 30.0, y: 15.0, w: 12.0, h: 12.0 });
    let seam = |c: &RenderCtx, tf: &TileFields| -> Vec<u32> {
        let (a, b) = (tile(c, tf, left, n, n), tile(c, tf, right, n, n));
        (0..n).map(|y| (0..3).map(|k| a[(y * n + n - 1) * 4 + k].abs_diff(b[(y * n) * 4 + k]) as u32).max().unwrap()).collect()
    };
    let (su, sp) = (seam(&un, &tfu), seam(&pa, &tfp));
    // Premise: the stroke crosses the seam.
    let (a, u) = (tile(&pa, &tfp, left, n, n), tile(&un, &tfu, left, n, n));
    let painted = (0..n).filter(|y| (0..3).any(|k| a[(y * n + n - 1) * 4 + k].abs_diff(u[(y * n + n - 1) * 4 + k]) > 20)).count();
    assert!(painted > 20, "premise: the stroke crosses the seam ({painted} rows)");
    for y in 0..n {
        assert!(sp[y] <= su[y] + 1, "row {y}: painted seam step {} vs unpainted {}", sp[y], su[y]);
    }
}

/// Mean absolute luminance change of `pix` between two renders.
fn luma_change(a: &[u8], b: &[u8], pix: &[usize]) -> f64 {
    let l = |v: &[u8], i: usize| 0.3 * v[i * 3] as f64 + 0.59 * v[i * 3 + 1] as f64 + 0.11 * v[i * 3 + 2] as f64;
    pix.iter().map(|&i| (l(a, i) - l(b, i)).abs()).sum::<f64>() / pix.len() as f64
}

/// Protects bar 6's mechanism: the paint goes in BEFORE the light, so the
/// hillshade still acts on painted ground. Turning the sun from the
/// north-west to the south-east moves the painted pixels' luminance by at
/// least half as much (relative to their mean) as it moves the unpainted
/// ones -- for Replace too, which would be flat under the hillshade if it
/// were applied after the light.
#[test]
fn the_sun_still_lights_painted_ground() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let pix: Vec<usize> = (18..25).flat_map(|y| (12..50).map(move |x| y * gw + x)).filter(|&i| field[i] >= SEA as f32).collect();
    let mean = |b: &[u8]| pix.iter().map(|&i| 0.3 * b[i * 3] as f64 + 0.59 * b[i * 3 + 1] as f64 + 0.11 * b[i * 3 + 2] as f64).sum::<f64>() / pix.len() as f64;
    let render_at = |az: f64, l: Option<&ColourPaint>| {
        let mut a = appearance();
        a.sun_az_deg = az;
        screen(&RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, a).with_map_scale(KM).with_colour_paint(l))
    };
    let (u1, u2) = (render_at(315.0, None), render_at(135.0, None));
    let base = luma_change(&u1, &u2, &pix) / mean(&u1);
    assert!(base > 0.05, "premise: the sun moves the unpainted relief ({base:.3})");
    for blend in ColourBlend::ALL {
        let l = ColourPaint::new(doc(vec![line(&[[100.0, 215.0], [520.0, 215.0]], 40.0, [230, 20, 20], blend, 1.0)]), 20.0).unwrap();
        let (p1, p2) = (render_at(315.0, Some(&l)), render_at(135.0, Some(&l)));
        let moved = luma_change(&p1, &p2, &pix) / mean(&p1);
        println!("{blend:?}: relative luminance change under the sun flip {moved:.3} vs unpainted {base:.3}");
        assert!(moved >= 0.5 * base, "{blend:?}: the light no longer reaches painted ground ({moved:.3} vs {base:.3})");
    }
}

/// Protects bar 6, the owner's volcano case: a Replace, a Colour and a
/// Multiply stroke over relief still shows the relief. The painted pixels'
/// relative luminance spread (std / mean) is at least 60 % of the unpainted
/// pixels' on the same ground -- it would collapse to near zero for Replace if the paint
/// were applied after the light. And Replace makes a red that reads red on
/// light ground (red channel at least twice green), which Colour need not.
#[test]
fn relief_survives_every_mode() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let un = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
    let u = screen(&un);
    // Inside the band stroke, rows 18..25, columns 12..50: all land, clear
    // of the frame and the sea.
    // Selected by the INPUT (the height), never by the colour under test:
    // the sea basin's north shore reaches row 24.
    let pix: Vec<usize> = (18..25).flat_map(|y| (12..50).map(move |x| y * gw + x)).filter(|&i| field[i] >= SEA as f32).collect();
    let luma = |b: &[u8], i: usize| 0.3 * b[i * 3] as f64 + 0.59 * b[i * 3 + 1] as f64 + 0.11 * b[i * 3 + 2] as f64;
    let spread = |b: &[u8]| {
        let v: Vec<f64> = pix.iter().map(|&i| luma(b, i)).collect();
        let m = v.iter().sum::<f64>() / v.len() as f64;
        // Relative (standard deviation over mean): the light MULTIPLIES the
        // colour, so a darker paint scales the spread with its mean, and the
        // relief is the relative variation, not the absolute one.
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt() / m
    };
    let su = spread(&u);
    assert!(su > 0.03, "premise: the fixture's relief shows ({su})");
    for blend in ColourBlend::ALL {
        let l = ColourPaint::new(doc(vec![line(&[[100.0, 215.0], [520.0, 215.0]], 40.0, [230, 20, 20], blend, 1.0)]), 20.0).unwrap();
        let p = screen(&RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM).with_colour_paint(Some(&l)));
        let sp = spread(&p);
        println!("{blend:?}: relative luma spread {sp:.3} vs unpainted {su:.3}");
        assert!(sp >= 0.6 * su, "{blend:?}: relief lost ({sp:.2} vs {su:.2})");
        // "Reads red": red at least 60 levels above both green and blue.
        let red = pix.iter().filter(|&&i| p[i * 3] as i32 - 60 >= (p[i * 3 + 1] as i32).max(p[i * 3 + 2] as i32)).count();
        println!("{blend:?}: {red} of {} painted pixels read red", pix.len());
        if blend == ColourBlend::Replace {
            assert_eq!(red, pix.len(), "Replace: not every painted pixel reads red");
        }
    }
}

/// Protects `land_only` per mark (the crater-lake case): on a sea pixel a
/// `land_only` mark changes nothing and a water-painting mark changes it,
/// on the screen, in the export and in a tile.
#[test]
fn land_only_is_honoured_on_every_path() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    // The sea basin's centre, cell (18, 30), is deep water.
    let (sx, sy) = (18usize, 30usize);
    assert!(field[sy * gw + sx] < SEA as f32, "premise: a sea cell");
    let mk = |land_only: bool| {
        let mut m = line(&[[(sx as f64 + 0.5) * 10.0, (sy as f64 + 0.5) * 10.0]], 30.0, [255, 0, 0], ColourBlend::Replace, 1.0);
        m.land_only = land_only;
        ColourPaint::new(doc(vec![m]), 20.0).unwrap()
    };
    let base = || RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_map_scale(KM);
    let (dry, wet) = (mk(true), mk(false));
    let px = |c: &RenderCtx| render::cell_color(c, sx, sy);
    assert_eq!(px(&base().with_colour_paint(Some(&dry))), px(&base()));
    let w = px(&base().with_colour_paint(Some(&wet)));
    // `cell_color` answers on 0..1.
    assert!(w.0 > 0.78 && w.1 < 0.25, "the water-painting mark reads red on the sea: {w:?}");
    // Export: the same cell's pixel.
    let (ew, eh) = render::bake_dims(61, gw, gh);
    let (e0, ed, ewet) = (export(&base(), 61), export(&base().with_colour_paint(Some(&dry)), 61), export(&base().with_colour_paint(Some(&wet)), 61));
    let o = (sy * ew + sx) * 3;
    assert_eq!(eh, gh);
    assert_eq!(&ed[o..o + 3], &e0[o..o + 3]);
    assert!(ewet[o] > 200 && ewet[o + 1] < 60, "{:?}", &ewet[o..o + 3]);
    // Tile.
    let tb = TileBounds { x: 6.0, y: 22.0, w: 16.0, h: 16.0 };
    let tile_px = |c: &RenderCtx| {
        let tf = TileFields::new(c, None);
        let t = tile(c, &tf, tb, 33, 33);
        let o = (16 * 33 + 24) * 4; // grid (6 + 24 * 0.5, 22 + 16 * 0.5) = (18, 30)
        [t[o], t[o + 1], t[o + 2]]
    };
    assert_eq!(tile_px(&base().with_colour_paint(Some(&dry))), tile_px(&base()));
    let t = tile_px(&base().with_colour_paint(Some(&wet)));
    assert!(t[0] > 200 && t[1] < 60, "{t:?}");
}

/// Protects: a context with no map width paints nothing (marks are in km;
/// a guessed scale would put them on the wrong ground).
#[test]
fn no_map_width_paints_nothing() {
    let (gw, gh) = (61, 43);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let l = ColourPaint::new(doc(vec![band_stroke(ColourBlend::Replace)]), 20.0).unwrap();
    let a = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance());
    let b = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, appearance()).with_colour_paint(Some(&l));
    assert_eq!(screen(&a), screen(&b));
}
