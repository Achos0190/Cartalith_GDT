//! Non-square grids, end to end: generation, rendering, and sprite
//! compositing at genuinely different aspect ratios.
//!
//! ## Why this test exists
//!
//! `cartalith_engine::WorldParams` has always had independent `gw`/`gh`, and
//! **every golden-parity fixture in this workspace is already non-square**
//! (14×11, 16×12, 24×18, 20×14, 48×40, 10×8) — the engine and the civ layer
//! are JS-verified at non-square dimensions and have been since they were
//! ported. The square-ness was entirely in this crate: `WorldGen::generate`
//! took one `resolution` and `call_params` wrote `p.gh = gw`.
//!
//! The two surfaces that had never been *exercised* non-square are the ones
//! this crate owns and that no golden fixture covers: `render.rs`'s per-pixel
//! renderer (`golden_parity_render.rs` runs at 10×10 and 12×12) and
//! `pack.rs`'s sprite compositing (`pack_compositing.rs` runs at 24×24). So
//! this file drives both at 2:1, 1:2 and a non-power-of-two 5:3, on real
//! generated worlds, and checks the output is a correctly *shaped* image
//! rather than a stretched, transposed or truncated one.
//!
//! `#[path]`-includes `render.rs`/`pack.rs` for the same reason
//! `pack_compositing.rs` does: `cartalith-godot` is `cdylib`-only
//! (`ARCHITECTURE.md`), so there is no `rlib` an integration test could link.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
#[path = "../src/pack.rs"]
mod pack;

use cartalith_engine::{WorldParams, generate_terrain};

/// The aspect ratios under test. Deliberately includes both orientations
/// (a `gh`-for-`gw` substitution that happens to be symmetric would survive
/// only one of them) and one case where neither dimension is a power of two
/// and neither divides the other.
const SHAPES: [(usize, usize); 4] = [(192, 96), (96, 192), (150, 90), (128, 128)];

fn world(gw: usize, gh: usize, seed: i32, world_mode: bool) -> cartalith_engine::WorldState {
    let mut p = WorldParams::defaults(gw, gh, seed);
    // CPU only: the GPU path is principled-equivalent, not bit-reproducible
    // (`DECISIONS.md` §7c), and this test is about geometry, not shading.
    p.use_gpu = false;
    p.world = world_mode;
    generate_terrain(&p)
}

fn ctx<'a>(
    ws: &'a cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world_mode: bool,
    appearance: render::TerrainAppearance,
) -> render::RenderCtx<'a> {
    render::RenderCtx::with_appearance(
        &ws.field,
        &ws.temperature,
        &ws.rainfall,
        Some(&ws.flow_discharge),
        gw,
        gh,
        ws.sea_level,
        world_mode,
        55.0,
        5.0,
        appearance,
    )
}

/// Every cell of every shape renders, with no panic and no index-out-of-
/// bounds — the failure mode a `y * gw + x` index whose `y` bound came from
/// `gw` would produce.
#[test]
fn every_cell_of_every_aspect_ratio_renders_in_range() {
    for (gw, gh) in SHAPES {
        let ws = world(gw, gh, 12345, false);
        assert_eq!(ws.field.len(), gw * gh, "{gw}x{gh}: engine allocated the wrong field size");
        let c = ctx(&ws, gw, gh, false, render::TerrainAppearance::default());
        let mut seen = 0usize;
        for y in 0..gh {
            for x in 0..gw {
                let (r, g, b) = render::cell_color(&c, x, y);
                assert!(
                    (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b),
                    "{gw}x{gh}: cell ({x},{y}) rendered out of range: {r},{g},{b}"
                );
                seen += 1;
            }
        }
        assert_eq!(seen, gw * gh);
    }
}

/// World mode (toroidal X wrap) at 2:1 — the reference's own equirectangular
/// shape (`gridH(gw)` is `gw*0.5` there), and the case where the X-wrapping
/// blur/slope paths run against a `gh` that is not `gw`.
#[test]
fn world_mode_renders_at_the_reference_two_to_one_shape() {
    let (gw, gh) = (192usize, 96usize);
    let ws = world(gw, gh, 987654, true);
    let c = ctx(&ws, gw, gh, true, render::TerrainAppearance::default());
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(&c, x, y);
            assert!((0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b));
        }
    }
}

/// The real shape check: the rendered image's own sea/land pattern must
/// still agree with the height field it was rendered from.
///
/// This is what separates "did not panic" from "is the right picture". A
/// renderer that read a row with the wrong stride, transposed the axes, or
/// clamped `y` to `gw` would still produce `gw*gh` finite pixels — but its
/// water would no longer sit where the field's water is. Sea colours here are
/// strongly blue-dominant (`w_shelf` .. `w_abyss`) and land colours are not,
/// so `b > r` is a direct readout of which branch `cell_color` took.
///
/// Cells at/below 2 °C are excluded: `snow_glac`/`snow_perm` are *also*
/// blue-dominant, so freezing land would legitimately read as "blue" and
/// weaken a check that is about geometry, not palette.
#[test]
fn rendered_water_still_lands_where_the_field_says_it_does() {
    for (gw, gh) in SHAPES {
        let ws = world(gw, gh, 4242, false);
        // The JS-reference appearance: no paper wash, no plate frame, so the
        // pixel is the material colour and nothing else.
        let c = ctx(&ws, gw, gh, false, render::TerrainAppearance::js_reference());
        let (mut agree, mut total) = (0usize, 0usize);
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                if ws.temperature[i] <= 2.0 {
                    continue;
                }
                let (r, _g, b) = render::cell_color(&c, x, y);
                let is_water = (ws.field[i] as f64) < ws.sea_level;
                if is_water == (b > r) {
                    agree += 1;
                }
                total += 1;
            }
        }
        assert!(total > gw * gh / 4, "{gw}x{gh}: too few non-freezing cells to judge ({total})");
        let frac = agree as f64 / total as f64;
        assert!(frac > 0.95, "{gw}x{gh}: rendered water/land agrees with the field only {:.1}% of the time", frac * 100.0);
    }
}

/// The plate frame (milestone 4) on a non-square sheet: a uniform margin of
/// the same cell width on all four sides, and a map interior that is not
/// margin. A frame keyed to the wrong dimension shows up here as either a
/// missing top/bottom margin or a plate that is entirely margin.
#[test]
fn the_plate_frame_is_a_uniform_margin_on_a_non_square_sheet() {
    let a = render::TerrainAppearance::default();
    for (gw, gh) in SHAPES {
        let w = render::border_width_cells(&a, gw, gh);
        assert!(w > 0.0, "{gw}x{gh}: default appearance must have a frame");
        // Fully covered at the very edge of all four sides...
        for (x, y) in [(0usize, gh / 2), (gw - 1, gh / 2), (gw / 2, 0), (gw / 2, gh - 1)] {
            assert_eq!(render::border_cover(&a, x, y, gw, gh), 1.0, "{gw}x{gh}: edge ({x},{y}) should be under the margin");
        }
        // ...and untouched in the middle of the plate.
        assert_eq!(render::border_cover(&a, gw / 2, gh / 2, gw, gh), 0.0, "{gw}x{gh}: plate centre must not be margin");
        // The margin must never swallow the sheet.
        assert!(w < gh as f64 * 0.5, "{gw}x{gh}: margin {w} cells would cover the whole height");
    }
}

/// The one-sided `gh` guard in `border_width_cells` must be exactly that:
/// square and tall grids keep the width they had before non-square
/// generation existed, so no square render can shift by a byte.
#[test]
fn the_border_guard_never_fires_on_a_square_or_tall_grid() {
    let a = render::TerrainAppearance::default();
    for n in [4usize, 16, 32, 64, 256, 512, 2048] {
        let before = (a.border_width_frac * n as f64).max(10.0);
        assert_eq!(render::border_width_cells(&a, n, n), before, "square {n}x{n} border width changed");
        assert_eq!(render::border_width_cells(&a, n, n * 3), before, "tall {n}x{} border width changed", n * 3);
    }
    // It does fire on a pathologically wide plate, which is the whole point:
    // 0.014*2048 = 28.7 cells of margin on a 40-cell-tall sheet would leave
    // no map at all.
    assert!(render::border_width_cells(&a, 2048, 40) < 28.0);
}

/// Sprite compositing (`pack.rs`, Asset Library milestone 7) on a non-square
/// buffer. The scatter engine, the biome/wetland raster it reads, and the
/// software rasterizer all index the same `gw*gh` buffer; a `gh`-for-`gw`
/// substitution here is an out-of-bounds panic or a buffer-length mismatch,
/// both of which this catches.
#[test]
fn sprite_compositing_runs_on_a_non_square_buffer() {
    let bytes = std::fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip")
        .expect("reference_pack.zip fixture (milestone 2) must exist");
    let loaded = pack::load_pack_from_bytes(bytes).expect("real reference-exported pack must load");

    for (gw, gh) in [(96usize, 48usize), (48usize, 96usize)] {
        let n = gw * gh;
        // A synthetic world in the same spirit as `pack_compositing.rs`'s:
        // a raised ridge across the middle rows so relief rules have real
        // candidates, boreal/conifer elsewhere.
        let mut field = vec![0.5f32; n];
        for y in (gh / 2 - 2)..(gh / 2 + 2) {
            for x in 0..gw {
                field[y * gw + x] = 0.95;
            }
        }
        let temperature = vec![8.0f32; n];
        let rainfall = vec![0.4f32; n];
        let mut buf = vec![255u8; n * 3];
        pack::composite_map_icons(&mut buf, &field, &temperature, &rainfall, gw, gh, 0.42, 7, &loaded);
        assert_eq!(buf.len(), n * 3, "{gw}x{gh}: compositing must not resize the buffer");
        assert!(buf.iter().any(|&b| b != 255), "{gw}x{gh}: expected the pack to paint something");
    }
}

/// A real image of a real 2:1 world, written as a PNG for eyeball
/// verification — the same "look at the actual pixels" discipline
/// `appearance_ab_dump.rs` established. `#[ignore]` because it generates
/// full-size worlds and has no business slowing `cargo test --workspace`:
///
/// ```text
/// cargo test -p cartalith-godot --test nonsquare -- --ignored --nocapture
/// ```
///
/// Output: `target/nonsquare/<w>x<h>.png`.
#[test]
#[ignore = "generates real worlds and writes PNGs; run explicitly with --ignored"]
fn dump_non_square_pngs() {
    for (gw, gh, world_mode, seed) in [(512usize, 256usize, false, 12345i32), (256, 512, false, 12345), (512, 256, true, 987654), (512, 512, false, 12345)] {
        let ws = world(gw, gh, seed, world_mode);
        let c = ctx(&ws, gw, gh, world_mode, render::TerrainAppearance::default());
        let mut rgba = Vec::with_capacity(gw * gh * 4);
        for y in 0..gh {
            for x in 0..gw {
                let (r, g, b) = render::cell_color(&c, x, y);
                rgba.push((r * 255.0).round().clamp(0.0, 255.0) as u8);
                rgba.push((g * 255.0).round().clamp(0.0, 255.0) as u8);
                rgba.push((b * 255.0).round().clamp(0.0, 255.0) as u8);
                rgba.push(255);
            }
        }
        let img = cartalith_assets::DecodedImage::new(gw as u32, gh as u32, rgba).expect("rgba buffer matches gw*gh*4");
        let png = cartalith_assets::encode_png(&img).expect("png encode");
        std::fs::create_dir_all("../../target/nonsquare").unwrap();
        let suffix = if world_mode { "_world" } else { "" };
        let path = format!("../../target/nonsquare/{gw}x{gh}{suffix}.png");
        std::fs::write(&path, png).unwrap();
        println!("wrote {path} ({gw}x{gh}, {} bytes of field)", ws.field.len());
    }
}
