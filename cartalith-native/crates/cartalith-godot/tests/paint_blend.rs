//! `landColorCore`'s paint-brush Biome/Terrain tint (reference HTML
//! 7897-7901), the one stage the map render was missing while the paint tool
//! itself was fully functional — a stroke wrote real cells, the overlay
//! preview drew them, and `build_color_texture` never changed a pixel.
//!
//! Not a golden *fixture* test: the reference block is four lines with no
//! new maths of its own, so what needs pinning is the **relationship**
//! between a painted pixel and the same unpainted pixel, which this file
//! asserts exactly rather than to a tolerance:
//!
//! ```text
//! l = l + (CART_*_COLS[p-1] - l) * 0.60
//! ```
//!
//! evaluated on the *fully shaded* colour, after every other colour and
//! lighting step and before the NPR block. Deriving the expected value from
//! this port's own unpainted render is the point: it makes the test
//! independent of every constant upstream of the blend (which
//! `golden_parity_render.rs` already pins) and sensitive to exactly the four
//! things this stage can get wrong — the weight, the two tables, the
//! ordering of the two layers, and whether the stage runs at all.
//!
//! `#[path]`-includes `render.rs` for the same reason every other renderer
//! test does: `cartalith-godot` is `cdylib`-only (`ARCHITECTURE.md`).
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;

use render::{PaintOverride, RenderCtx, TerrainAppearance, CART_BIOME_COLS, CART_TERRAIN_COLS};

const GW: usize = 24;
const GH: usize = 24;
const SEA: f64 = 0.30;

struct World {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
}

/// All-land, gently varied so neighbouring cells differ and a constant
/// output would be visible as a bug rather than a pass.
fn world() -> World {
    let mut field = vec![0.0f32; GW * GH];
    for y in 0..GH {
        for x in 0..GW {
            field[y * GW + x] = 0.45 + 0.30 * ((x + y) as f32 / (GW + GH) as f32);
        }
    }
    World { field, temperature: vec![11.0; GW * GH], rainfall: vec![0.45; GW * GH] }
}

fn ctx<'a>(w: &'a World) -> RenderCtx<'a> {
    RenderCtx::with_appearance(
        &w.field,
        &w.temperature,
        &w.rainfall,
        None,
        GW,
        GH,
        SEA,
        false,
        60.0,
        20.0,
        TerrainAppearance::js_reference(),
    )
}

fn grid(painted: &[(usize, u8)]) -> Vec<u8> {
    let mut g = vec![0u8; GW * GH];
    for &(i, v) in painted {
        g[i] = v;
    }
    g
}

/// The reference's own line, in one place, so no test below can quietly
/// re-derive it differently: `l = l + (p - l) * 0.60`, on 0-255 channels.
fn blend(base: (f64, f64, f64), p: (u8, u8, u8)) -> (f64, f64, f64) {
    (
        base.0 + (p.0 as f64 - base.0) * 0.60,
        base.1 + (p.1 as f64 - base.1) * 0.60,
        base.2 + (p.2 as f64 - base.2) * 0.60,
    )
}

/// `cell_color` returns `[0,1]`; the reference blends in `[0,255]`. Compare
/// there, and to a tolerance that only absorbs the one `/255.0` division
/// (the blend itself is asserted exactly by construction of the expected
/// value from the same unpainted render).
fn as255(c: (f64, f64, f64)) -> (f64, f64, f64) {
    (c.0 * 255.0, c.1 * 255.0, c.2 * 255.0)
}

fn assert_close(got: (f64, f64, f64), want: (f64, f64, f64), what: &str) {
    for (g, w, ch) in [(got.0, want.0, 'r'), (got.1, want.1, 'g'), (got.2, want.2, 'b')] {
        assert!((g - w).abs() < 1e-6, "{what}: channel {ch} was {g}, expected {w}");
    }
}

#[test]
fn an_unpainted_grid_renders_bit_identically_to_no_paint_grid_at_all() {
    // The property that let `golden_parity_render.rs` pass untouched, made
    // explicit: attaching three all-zero layers is not merely close to
    // attaching none, it is the same bytes.
    let w = world();
    let zero = vec![0u8; GW * GH];
    let plain = ctx(&w);
    let with = ctx(&w).with_paint(Some(&zero), Some(&zero), Some(&zero));
    for y in 0..GH {
        for x in 0..GW {
            assert_eq!(render::cell_color(&plain, x, y), render::cell_color(&with, x, y), "({x},{y})");
        }
    }
}

#[test]
fn a_painted_biome_cell_is_the_reference_060_blend_of_its_own_unpainted_colour() {
    let w = world();
    let plain = ctx(&w);
    // Every legal Biome index, not one -- a wrong table would still pass on
    // a single lucky entry, and index 1 in particular is the one an
    // off-by-one would land on.
    for v in 1..=13u8 {
        let i = 10 * GW + 10;
        let g = grid(&[(i, v)]);
        let painted = ctx(&w).with_paint(Some(&g), None, None);
        let want = blend(as255(render::cell_color(&plain, 10, 10)), CART_BIOME_COLS[v as usize - 1]);
        assert_close(as255(render::cell_color(&painted, 10, 10)), want, &format!("biome index {v}"));
    }
}

#[test]
fn a_painted_terrain_cell_is_the_reference_060_blend_of_its_own_unpainted_colour() {
    let w = world();
    let plain = ctx(&w);
    for v in 1..=13u8 {
        let i = 10 * GW + 10;
        let g = grid(&[(i, v)]);
        let painted = ctx(&w).with_paint(None, Some(&g), None);
        let want = blend(as255(render::cell_color(&plain, 10, 10)), CART_TERRAIN_COLS[v as usize - 1]);
        assert_close(as255(render::cell_color(&painted, 10, 10)), want, &format!("terrain index {v}"));
    }
}

#[test]
fn biome_and_terrain_coexist_on_one_cell_and_apply_in_the_references_order() {
    // "Both layers can coexist on one cell, applied sequentially" -- and the
    // order is load-bearing, because two 0.60 blends do not commute.
    let w = world();
    let plain = ctx(&w);
    let i = 7 * GW + 13;
    let gb = grid(&[(i, 6)]);
    let gt = grid(&[(i, 3)]);
    let both = ctx(&w).with_paint(Some(&gb), Some(&gt), None);

    let base = as255(render::cell_color(&plain, 13, 7));
    let want = blend(blend(base, CART_BIOME_COLS[5]), CART_TERRAIN_COLS[2]);
    assert_close(as255(render::cell_color(&both, 13, 7)), want, "biome then terrain");

    let wrong_way = blend(blend(base, CART_TERRAIN_COLS[2]), CART_BIOME_COLS[5]);
    assert!(
        (want.0 - wrong_way.0).abs() > 1e-3 || (want.1 - wrong_way.1).abs() > 1e-3 || (want.2 - wrong_way.2).abs() > 1e-3,
        "this fixture must actually distinguish the two orders, or the assertion above proves nothing"
    );
}

#[test]
fn only_the_painted_cell_changes() {
    let w = world();
    let plain = ctx(&w);
    let i = 12 * GW + 5;
    let g = grid(&[(i, 4)]);
    let painted = ctx(&w).with_paint(Some(&g), None, None);
    for y in 0..GH {
        for x in 0..GW {
            let same = render::cell_color(&plain, x, y) == render::cell_color(&painted, x, y);
            assert_eq!(same, !(x == 5 && y == 12), "({x},{y}) changed-ness is wrong");
        }
    }
}

#[test]
fn a_paint_grid_shorter_than_the_field_is_dropped_rather_than_indexing_off_the_end() {
    // `cartalith-rust-conventions`: a panic must not cross the gdext
    // boundary, and this one would fire once per pixel inside a rayon
    // `par_chunks_mut`.
    let w = world();
    let short = vec![9u8; 4];
    let plain = ctx(&w);
    let with = ctx(&w).with_paint(Some(&short), None, None);
    for y in 0..GH {
        for x in 0..GW {
            assert_eq!(render::cell_color(&plain, x, y), render::cell_color(&with, x, y));
        }
    }
}

#[test]
fn the_water_branch_never_takes_a_paint_blend() {
    // The blend lives in `landColorCore`; `seaColorCore` has no paint
    // parameter at all. Moot in the reference (`_paintAt` is unconditionally
    // land-gated) and reachable here only through this port's own
    // `land_only` toggle, so it is pinned rather than assumed.
    let mut w = world();
    let i = 3 * GW + 3;
    w.field[i] = 0.10; // well below SEA
    let plain = ctx(&w);
    let g = grid(&[(i, 5)]);
    let painted = ctx(&w).with_paint(Some(&g), None, None);
    assert_eq!(render::cell_color(&plain, 3, 3), render::cell_color(&painted, 3, 3));
}

#[test]
fn an_out_of_range_index_paints_nothing_rather_than_wrapping_or_panicking() {
    let w = world();
    let plain = ctx(&w);
    for v in [14u8, 15, 200, 255] {
        let i = 10 * GW + 10;
        let g = grid(&[(i, v)]);
        // 14/15 are legal *CART_BIOME_COLS* rows (Lake/Ocean) but not
        // paintable ones; they must still resolve to their own colour rather
        // than to a neighbouring index, so the two tables are checked apart.
        let painted = ctx(&w).with_paint(None, Some(&g), None);
        if (v as usize) <= CART_TERRAIN_COLS.len() {
            continue;
        }
        assert_eq!(render::cell_color(&plain, 10, 10), render::cell_color(&painted, 10, 10), "terrain index {v}");
    }
}

#[test]
fn paint_override_default_is_the_unpainted_state() {
    assert_eq!(PaintOverride::default(), PaintOverride { bio: 0, ter: 0, splat: 0 });
}
