//! Ruling CO item 3 (owner, 2026-10-18): the deep-zoom tiles' volcano flank
//! furrows, as the renderer wires them (`render::add_tile_furrows`, called by
//! `lod_bridge::tile_heights` and the zoom-ladder harness).
//!
//! Held here:
//!
//! - **The split**: `TerrainAppearance::default()` ships the furrows at `1.0`
//!   and `js_reference()` pins `0.0`, so the parity tile goldens cannot see
//!   them (literals, not the field read back).
//! - **The gates**: a tile is untouched -- bit for bit -- with the look's
//!   strength at zero, without an edifice frame, or without a map width;
//!   with all three it moves land texels on the cone and nothing off it.
//! - **The frame guard**: a frame for another grid is not attached.

/// The renderer under test, compiled straight from `src/render.rs`.
#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use render::{RenderCtx, TerrainAppearance, TileBounds};

/// Grid edge in cells (a judgement: room for one 30-cell cone and margin).
const N: usize = 96;
/// The engine's default sea level; the fixture is land everywhere.
const SEA: f64 = 0.42;

/// A cone of radius 30 cells at the grid centre on a plain at 0.5, and the
/// matching `volcanic_field` mark `(1 - t)`, the stamp's own.
fn world() -> (Vec<f32>, Vec<f32>) {
    let mut field = vec![0.5f32; N * N];
    let mut volc = vec![0f32; N * N];
    for y in 0..N {
        for x in 0..N {
            let d = (x as f64 - 48.0).hypot(y as f64 - 48.0);
            if d <= 30.0 {
                let t = d / 30.0;
                field[y * N + x] = (0.5 + 0.3 * (1.0 - t)) as f32;
                volc[y * N + x] = (1.0 - t) as f32;
            }
        }
    }
    (field, volc)
}

/// A padded tile over the cone's east flank (`x` 66..70, `y` 46..50), filled
/// with the coarse field sampled bilinearly, as the amplifier's core would be.
fn tile(field: &[f32]) -> (Vec<f32>, usize, usize, usize, TileBounds) {
    let (w, h, pad) = (64usize, 64usize, 2usize);
    let b = TileBounds { x: 66.0, y: 46.0, w: 4.0, h: 4.0 };
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let mut d = vec![0f32; pw * ph];
    for oy in 0..ph {
        for ox in 0..pw {
            let cx = b.x + (ox as f64 - pad as f64) / (w as f64 - 1.0) * b.w;
            let cy = b.y + (oy as f64 - pad as f64) / (h as f64 - 1.0) * b.h;
            let (x0, y0) = (cx.floor() as usize, cy.floor() as usize);
            let (tx, ty) = (cx - x0 as f64, cy - y0 as f64);
            let v = |x: usize, y: usize| field[y * N + x] as f64;
            d[oy * pw + ox] = ((v(x0, y0) * (1.0 - tx) + v(x0 + 1, y0) * tx) * (1.0 - ty) + (v(x0, y0 + 1) * (1.0 - tx) + v(x0 + 1, y0 + 1) * tx) * ty) as f32;
        }
    }
    (d, w, h, pad, b)
}

/// Protects: the furrows ship on in the app's look and off on the parity
/// path. Literals, so a default moved to the parity value fails here.
#[test]
fn furrows_are_on_in_the_shipped_look_and_off_on_the_parity_path() {
    assert_eq!(TerrainAppearance::default().edifice_furrows, 1.0);
    assert_eq!(TerrainAppearance::js_reference().edifice_furrows, 0.0);
    let mut a = TerrainAppearance::default();
    assert!(a.set_tunable("edifice_furrows", 5.0), "the tunable row exists");
    assert_eq!(a.edifice_furrows, 2.0, "clamped to the row's maximum");
}

/// Protects: each gate of `add_tile_furrows` -- strength 0, no frame, no map
/// width -- leaves the tile bit-identical, and with all three present the
/// tile's land texels on the flank move while the cone's heights stay
/// within a few metres' worth of their source.
#[test]
fn every_gate_leaves_the_tile_untouched_and_all_three_move_it() {
    let (field, volc) = world();
    let (t, r) = (vec![12f32; N * N], vec![0.5f32; N * N]);
    let frame = cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&volc, N, N);
    assert_eq!(frame.edifices.len(), 1, "the fixture's cone must be recovered");
    let (base, w, h, pad, b) = tile(&field);
    let ctx_with = |a: TerrainAppearance, frame_on: bool, km: Option<f64>| {
        let mut c = RenderCtx::with_appearance(&field, &t, &r, None, N, N, SEA, false, 45.0, 35.0, a);
        if let Some(k) = km {
            c = c.with_map_scale(k);
        }
        if frame_on {
            c = c.with_edifice_frame(&frame, 4000.0);
        }
        c
    };
    let run = |c: &RenderCtx| {
        let mut d = base.clone();
        render::add_tile_furrows(c, &mut d, w, h, pad, b);
        d
    };
    let km = Some(800.0 * N as f64 / 2048.0);
    let off = TerrainAppearance { edifice_furrows: 0.0, ..TerrainAppearance::default() };
    assert_eq!(run(&ctx_with(off, true, km)), base, "strength 0 moved the tile");
    assert_eq!(run(&ctx_with(TerrainAppearance::default(), false, km)), base, "no frame, yet the tile moved");
    assert_eq!(run(&ctx_with(TerrainAppearance::default(), true, None)), base, "no map width, yet the tile moved");
    let on = run(&ctx_with(TerrainAppearance::default(), true, km));
    let moved = on.iter().zip(&base).filter(|(a, b)| a != b).count();
    assert!(moved > on.len() / 10, "furrows moved only {moved} of {} texels", on.len());
    let max = on.iter().zip(&base).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
    // 150 m at most for the coarsest octave at 4000 m / 0.58 per unit is
    // about 0.022; several octaves stay under 0.05.
    assert!(max < 0.05, "a furrow excursion of {max} is not a rill");
}

/// Protects: a frame recovered on another grid is never attached (it would
/// index out of bounds inside the tile loop), so such a context draws none.
#[test]
fn a_frame_for_another_grid_is_not_attached() {
    let (field, _) = world();
    let (t, r) = (vec![12f32; N * N], vec![0.5f32; N * N]);
    // A real cone on a 48 x 48 grid, centred so the test tile (x 66..70)
    // falls at t ~ 0.5 of it: attached, it would draw furrows (its
    // `owner_near` clamps into its own grid), so only the guard keeps the
    // tile untouched.
    let (_, small_volc) = {
        let mut v = vec![0f32; 48 * 48];
        for y in 0..48 {
            for x in 0..48 {
                let d = (x as f64 - 40.0).hypot(y as f64 - 40.0);
                if d <= 60.0 {
                    v[y * 48 + x] = (1.0 - d / 60.0) as f32;
                }
            }
        }
        ((), v)
    };
    let other = cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&small_volc, 48, 48);
    assert_eq!(other.edifices.len(), 1);
    let c = RenderCtx::with_appearance(&field, &t, &r, None, N, N, SEA, false, 45.0, 35.0, TerrainAppearance::default())
        .with_map_scale(37.5)
        .with_edifice_frame(&other, 4000.0);
    let (base, w, h, pad, b) = tile(&field);
    let mut d = base.clone();
    render::add_tile_furrows(&c, &mut d, w, h, pad, b);
    assert_eq!(d, base);
}
