//! The banded export renderer (`EXPORT_SCOPE.md` §4, milestone E1) —
//! `render::ExportBandPlan` and `render::bake_export_band`.
//!
//! # What is being proved, and why at this size
//!
//! A 16K/32K export cannot be held whole, and it cannot be compared against a
//! whole one either, because the whole one is the thing that does not fit. So
//! the claim is split in two:
//!
//! 1. [`ExportBandPlan`] derives every decision from the **full** `(w, h)` —
//!    the local-contrast radius, the plate frame's geometry, the grade
//!    influence's row mapping — so nothing it does depends on the band size.
//! 2. Given that, a banded render is measured **byte-identical** to the
//!    monolithic one (`export_raster_png`'s own four calls, transcribed below)
//!    at a width where both can be run.
//!
//! The identity is exact rather than close because of `EXPORT_SCOPE.md` §4.2:
//! the only neighbourhood stage is `apply_local_contrast`, its vertical reach is
//! exactly its radius, and a band carrying that many apron rows each side sees
//! the window the whole-raster pass sees.
//!
//! Compiled standalone through the same `#[path]` trick `tests/bake_raster.rs`
//! uses, so this runs under plain `cargo test` with no Godot present.

#[path = "../src/render.rs"]
mod render;

use render::{BakeFields, ExportBandPlan, RenderCtx, RiverInk, TerrainAppearance};

/// `EXPORT_SCOPE.md` §4.3's fixture: a 61 × 43 world rendered at 512 px wide.
const GW: usize = 61;
const GH: usize = 43;
const OUT_W: usize = 512;
const SEA: f64 = 0.42;

/// `tests/bake_raster.rs`'s closed-form world, at this file's grid size: two
/// ridges and a basin spanning `SEA`, temperatures across the material
/// switches, rainfall across the dry/wet ones.
fn fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let n = GW * GH;
    let (mut field, mut temp, mut rain, mut flow) = (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for y in 0..GH {
        for x in 0..GW {
            let (u, v) = (x as f64 / GW as f64, y as f64 / GH as f64);
            let i = y * GW + x;
            field[i] = (0.46 + 0.34 * ((u * 6.1).sin() * (v * 4.3).cos()) + 0.12 * ((u * 17.0 + v * 11.0).sin())) as f32;
            temp[i] = (28.0 - 44.0 * v + 6.0 * (u * 9.0).cos()) as f32;
            rain[i] = (0.5 + 0.5 * ((u * 5.0 + v * 3.0).sin())).clamp(0.0, 1.0) as f32;
            flow[i] = (1.0 + 40.0 * ((u * 13.0).sin() * (v * 7.0).sin()).abs()) as f32;
        }
    }
    (field, temp, rain, flow)
}

/// Every stage that can differ between a band and the whole raster, switched
/// on: local contrast (on at `default()`), the plate frame (on at `default()`),
/// a real grade, and two of the grade's field-influence weights, so the
/// influence's row half is in the path.
fn appearance() -> TerrainAppearance {
    let mut a = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
    a.grade_field_elevation = 0.5;
    a.grade_field_moisture = 0.3;
    a
}

/// `export_raster_png`'s closure, transcribed call for call: the shipped
/// monolithic path this milestone must not change.
fn monolithic(ctx: &RenderCtx, a: &TerrainAppearance, ink: Option<RiverInk<'_>>, w: usize, h: usize) -> Vec<u8> {
    let bf = BakeFields::new(ctx);
    let mut px = render::bake_rect(ctx, &bf, ink, w, h, 0, 0, w, h);
    render::apply_local_contrast(a, &mut px, w, h, ctx.world);
    let inf = render::build_grade_influence(ctx, w, h);
    render::apply_color_grade(a, &mut px, &inf);
    px
}

/// The banded path, stitched — asserting as it goes that the plan covers
/// every row exactly once and that every band is the size it claims.
fn banded(ctx: &RenderCtx, ink: Option<RiverInk<'_>>, plan: &ExportBandPlan) -> Vec<u8> {
    let bf = BakeFields::new(ctx);
    let cells = render::build_grade_influence_cells(ctx);
    let mut out = Vec::with_capacity(plan.w * plan.h * 3);
    for band in plan.bands() {
        assert_eq!(band.y0 * plan.w * 3, out.len(), "band at y0={} does not start where the last one ended", band.y0);
        let px = render::bake_export_band(ctx, &bf, ink, plan, band, &cells);
        assert_eq!(px.len(), band.rows * plan.w * 3, "band at y0={} returned the wrong row count", band.y0);
        out.extend_from_slice(&px);
    }
    assert_eq!(out.len(), plan.w * plan.h * 3, "the bands do not cover the raster");
    out
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Mean absolute byte delta between row `y - 1` and row `y`.
fn row_delta(px: &[u8], w: usize, y: usize) -> f64 {
    let (a, b) = (&px[(y - 1) * w * 3..y * w * 3], &px[y * w * 3..(y + 1) * w * 3]);
    a.iter().zip(b).map(|(p, q)| p.abs_diff(*q) as f64).sum::<f64>() / (w * 3) as f64
}

#[test]
fn a_banded_render_is_byte_identical_to_a_monolithic_one() {
    let (field, temp, rain, flow) = fixture();
    let (w, h) = render::bake_dims(OUT_W, GW, GH);
    // The default radius, then one five times wider so the apron is larger
    // than the smallest band rather than equal to it.
    for frac in [0.010, 0.05] {
        let mut a = appearance();
        a.local_contrast_radius_frac = frac;
        let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
        let whole = monolithic(&ctx, &a, None, w, h);
        let one = ExportBandPlan::with_rows(&a, w, h, h);
        assert_eq!((one.band_count(), one.apron), (1, 0));
        let single = banded(&ctx, None, &one);
        println!("frac {frac}: {w}x{h}, single band: {} differing bytes", differing(&single, &whole));
        assert_eq!(single, whole, "a single-band render is not the monolithic one (frac {frac})");
        for rows in [271usize, 128, 64, 37, 5] {
            let plan = ExportBandPlan::with_rows(&a, w, h, rows);
            let got = banded(&ctx, None, &plan);
            let d = differing(&got, &whole);
            println!("frac {frac}: {rows} rows/band, {} bands, apron {}: {d} differing bytes of {}", plan.band_count(), plan.apron, whole.len());
            assert_eq!(d, 0, "{rows} rows/band (frac {frac}) differs from the monolithic render in {d} bytes");
        }
    }
}

#[test]
fn river_tint_world_wrap_and_hachure_band_identically() {
    let (field, temp, rain, flow) = fixture();
    let mut a = appearance();
    a.npr.hachure = 0.6;
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, true, 55.0, 5.0, a.clone());
    let mask: Vec<u8> = (0..GW * GH).map(|i| u8::from((i % GW).is_multiple_of(3) || i % GW == i / GW)).collect();
    let ink = Some(RiverInk::Flag(&mask));
    let (w, h) = render::bake_dims(OUT_W, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    // Each switch is doing something, or its half of this test is vacuous.
    let a_plain = appearance();
    let ctx_plain = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a_plain.clone());
    let plain = monolithic(&ctx_plain, &a_plain, None, w, h);
    assert!(differing(&whole, &plain) > 0, "tint + wrap + hachure changed nothing");
    let no_ink = monolithic(&ctx, &a, None, w, h);
    assert!(differing(&whole, &no_ink) > 0, "the river ink moved no pixel");
    for rows in [97usize, 48, 16] {
        let plan = ExportBandPlan::with_rows(&a, w, h, rows);
        let got = banded(&ctx, ink, &plan);
        let d = differing(&got, &whole);
        println!("tint+wrap+hachure: {rows} rows/band, {} bands: {d} differing bytes of {}", plan.band_count(), whole.len());
        assert_eq!(d, 0, "{rows} rows/band differs from the monolithic render in {d} bytes");
    }
}

#[test]
fn there_is_no_step_at_a_band_boundary() {
    let (field, temp, rain, flow) = fixture();
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let (w, h) = render::bake_dims(OUT_W, GW, GH);
    let whole = monolithic(&ctx, &a, None, w, h);
    let plan = ExportBandPlan::with_rows(&a, w, h, 37);
    let got = banded(&ctx, None, &plan);
    let boundaries: Vec<usize> = plan.bands().skip(1).map(|b| b.y0).collect();
    assert!(boundaries.len() >= 4, "only {} boundaries -- too few to measure", boundaries.len());
    let mut worst = 0f64;
    for &y in &boundaries {
        let (db, dm) = (row_delta(&got, w, y), row_delta(&whole, w, y));
        worst = worst.max((db - dm).abs());
        assert!((db - dm).abs() < 1e-12, "row {y}: banded delta {db} vs monolithic {dm}");
    }
    let mean_seam = boundaries.iter().map(|&y| row_delta(&got, w, y)).sum::<f64>() / boundaries.len() as f64;
    println!("{} boundaries, worst |banded - monolithic| row delta {worst:e}, mean seam delta {mean_seam:.4} levels", boundaries.len());
}

#[test]
fn the_identity_is_not_vacuous() {
    let (field, temp, rain, flow) = fixture();
    let a = appearance();
    let (w, h) = render::bake_dims(OUT_W, GW, GH);
    assert!(a.local_contrast > 0.0, "local contrast is off -- the one neighbourhood stage is not in the path");
    assert!(render::border_width_cells(&a, w, h) > 0.0, "the plate frame is off");
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let cells = render::build_grade_influence_cells(&ctx);
    assert_eq!(cells.len(), GW * GH, "the grade influence is empty");
    assert!(cells.iter().any(|&m| m != 1.0), "the grade influence is flat 1.0 everywhere");
    let plan = ExportBandPlan::with_rows(&a, w, h, 37);
    // round(512 * 0.010) = 5, above the floor of 3 and below h / 4.
    assert_eq!(plan.apron, 5, "the apron at this size");
    let whole = monolithic(&ctx, &a, None, w, h);

    // Negative control 1: local contrast off changes the picture.
    let mut off = a.clone();
    off.local_contrast = 0.0;
    let ctx_off = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, off.clone());
    let d_off = differing(&monolithic(&ctx_off, &off, None, w, h), &whole);
    println!("local contrast off: {d_off} differing bytes");
    assert!(d_off > 0, "turning local contrast off changed nothing");

    // Negative control 2: the apron is load-bearing. The same bands with no
    // apron must differ -- otherwise the identity above proves nothing about it.
    let bare = ExportBandPlan { apron: 0, ..plan };
    let d_bare = differing(&banded(&ctx, None, &bare), &whole);
    println!("37 rows/band with a zero apron: {d_bare} differing bytes");
    assert!(d_bare > 0, "dropping the apron changed nothing -- the band identity is not testing it");
}

#[test]
fn the_plan_covers_every_row_once_and_the_shipped_sizes_stay_one_band() {
    let a = TerrainAppearance::default();
    // The app's own 2048 x 1311 grid (`export_raster.rs`'s measured worlds).
    for width in [2048usize, 4096, 8192, 16384, 32768] {
        let (w, h) = render::bake_dims(width, 2048, 1311);
        let whole = (w as u64) * (h as u64);

        // A budget of the whole raster (or more) is the shipped monolithic
        // path: exactly one band, zero apron, nothing clipped.
        for budget in [whole, whole + 1, whole * 4] {
            let p = ExportBandPlan::for_budget(&a, w, h, budget);
            let bands: Vec<_> = p.bands().collect();
            assert_eq!(bands.len(), 1, "{width}: budget {budget} split into {} bands", bands.len());
            assert_eq!(p.apron, 0, "{width}: a single band carries an apron");
            let b = bands[0];
            assert_eq!((b.y0, b.rows, b.top, b.bottom), (0, h, 0, 0), "{width}: the single band is not the whole raster");
        }

        // Real budgets: every row exactly once, and every band's rows plus
        // apron inside the budget.
        for budget_rows in [h - 1, h / 2, 1024, 700] {
            let budget = (w * budget_rows) as u64;
            let p = ExportBandPlan::for_budget(&a, w, h, budget);
            let mut seen = vec![0u8; h];
            for b in p.bands() {
                assert_eq!(b.top, p.apron.min(b.y0));
                assert_eq!(b.bottom, p.apron.min(h - b.y0 - b.rows));
                assert!((b.rows + b.top + b.bottom) as u64 * w as u64 <= budget, "{width}: a band exceeds the budget");
                for s in &mut seen[b.y0..b.y0 + b.rows] {
                    *s += 1;
                }
            }
            assert!(seen.iter().all(|&s| s == 1), "{width}/{budget_rows}: some row is covered {} times", seen.iter().find(|&&s| s != 1).unwrap());
            assert!(p.band_count() > 1, "{width}/{budget_rows}: a budget below the raster did not split it");
        }
    }
    // The radius is derived from the full width: 328 rows at 32K
    // (round(32768 * 0.010) = 328), EXPORT_SCOPE.md §4.1's figure.
    let (w, h) = render::bake_dims(32768, 2048, 1311);
    assert_eq!(ExportBandPlan::with_rows(&a, w, h, 1024).apron, 328);
}
