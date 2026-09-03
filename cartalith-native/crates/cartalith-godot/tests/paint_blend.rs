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
//! The second half of the file (from `attaching_tiles_changes_nothing`
//! onward) covers the same stage's **v1.28 pack-tile branch**
//! (`_paintedTex`, reference 12187-12196): a loaded pack's `biomes`/
//! `terrains` ground tile is blended as true colour *instead of* the flat
//! swatch, at the same weight and position. Same method — the expected value
//! is derived from this port's own unpainted render — so what those tests
//! pin is the sampler (wrap, texel offset, positional index) and the
//! true-colour/inverse-mean asymmetry, not any constant already pinned
//! above.
//!
//! `#[path]`-includes `render.rs` for the same reason every other renderer
//! test does: `cartalith-godot` is `cdylib`-only (`ARCHITECTURE.md`), and
//! `pack.rs` alongside it so the last test can load the real fixture pack.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
#[path = "../src/pack.rs"]
mod pack;

use render::{GroundTile, GroundTiles, PaintOverride, RenderCtx, TerrainAppearance, CART_BIOME_COLS, CART_TERRAIN_COLS};

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

// ---------------------------------------------------------------------------
// The pack-tile half of the same blend (`_paintedTex`, reference v1.28)
//
// Everything above pins `_p = CART_*_COLS[p-1]`. Everything below pins the
// `_t ||` in front of it: when a loaded pack supplies a ground tile for the
// painted index, that tile's **true colour** is what gets blended, at the
// same 0.60 weight and the same position. The flat-swatch tests above are
// therefore also the fallback tests for this half.
// ---------------------------------------------------------------------------

/// A tile of one colour, `w` by `h`. Solid on purpose: see
/// `two_solid_tiles_of_different_colours_blend_differently` for the mistake
/// that makes discriminable.
fn solid(w: u32, h: u32, c: (u8, u8, u8)) -> GroundTile {
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        rgba.extend_from_slice(&[c.0, c.1, c.2, 255]);
    }
    GroundTile { w, h, rgba }
}

/// A positional family table `n` long with one tile at `at` (0-based, i.e.
/// painted index `at + 1`) — the shape `LoadedPack::biomes`/`::terrains` has.
fn table(n: usize, at: usize, t: GroundTile) -> Vec<Option<GroundTile>> {
    let mut v: Vec<Option<GroundTile>> = (0..n).map(|_| None).collect();
    v[at] = Some(t);
    v
}

fn biome_tiles(t: &[Option<GroundTile>]) -> GroundTiles<'_> {
    GroundTiles { biomes: t, terrains: &[] }
}

fn terrain_tiles(t: &[Option<GroundTile>]) -> GroundTiles<'_> {
    GroundTiles { biomes: &[], terrains: t }
}

#[test]
fn attaching_tiles_changes_nothing_until_a_cell_is_actually_painted() {
    // The property the default render depends on: this port ships no pack,
    // but loading one must not move a pixel by itself either. Every cell
    // here is unpainted and every cell must be byte-identical.
    let w = world();
    let t = table(15, 5, solid(4, 4, (255, 0, 255)));
    let plain = ctx(&w);
    let with = ctx(&w).with_ground_tiles(biome_tiles(&t));
    for y in 0..GH {
        for x in 0..GW {
            assert_eq!(render::cell_color(&plain, x, y), render::cell_color(&with, x, y), "({x},{y})");
        }
    }
}

#[test]
fn an_empty_ground_table_is_the_flat_swatch_branch_exactly() {
    // `GroundTiles::default()` (no pack) and an all-`None` table of the real
    // length (a pack with no art for any painted index) are the same picture
    // as never calling the builder — the reference's `_t || CART_*_COLS`
    // fallback, and what keeps `golden_parity_render.rs` out of this branch.
    let w = world();
    let i = 10 * GW + 10;
    let g = grid(&[(i, 6)]);
    let none: Vec<Option<GroundTile>> = (0..15).map(|_| None).collect();
    let plain = ctx(&w).with_paint(Some(&g), None, None);
    for ground in [GroundTiles::default(), biome_tiles(&none)] {
        let with = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(ground);
        assert_eq!(render::cell_color(&plain, 10, 10), render::cell_color(&with, 10, 10));
    }
}

#[test]
fn a_pack_tile_replaces_the_flat_swatch_at_the_same_060_weight() {
    let w = world();
    let plain = ctx(&w);
    let i = 10 * GW + 10;
    let g = grid(&[(i, 6)]);
    let tile = (40, 100, 45);
    let t = table(15, 5, solid(8, 8, tile));
    let painted = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&t));

    let base = as255(render::cell_color(&plain, 10, 10));
    assert_close(as255(render::cell_color(&painted, 10, 10)), blend(base, tile), "biome 6 with a tile");
    // And demonstrably *not* the swatch it replaced, or the assertion above
    // would pass on a renderer that ignored the tile entirely.
    let swatch = blend(base, CART_BIOME_COLS[5]);
    assert!(
        (blend(base, tile).0 - swatch.0).abs() > 1e-3 || (blend(base, tile).2 - swatch.2).abs() > 1e-3,
        "fixture colours must differ from the swatch, or this test proves nothing"
    );
}

#[test]
fn two_solid_tiles_of_different_colours_blend_differently() {
    // **The `SplatChannel` mistake, pinned.** A splat channel is sampled as
    // `texel * inv_mean` — for a *solid* tile that ratio is 1.0 whatever the
    // colour, so a paint path that borrowed splat's normalisation would give
    // these two tiles the identical result. True colour gives two.
    let w = world();
    let i = 10 * GW + 10;
    let g = grid(&[(i, 6)]);
    let dark = table(15, 5, solid(4, 4, (20, 30, 40)));
    let light = table(15, 5, solid(4, 4, (220, 210, 200)));
    let a = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&dark));
    let b = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&light));
    assert_ne!(render::cell_color(&a, 10, 10), render::cell_color(&b, 10, 10));

    let base = as255(render::cell_color(&ctx(&w), 10, 10));
    assert_close(as255(render::cell_color(&a, 10, 10)), blend(base, (20, 30, 40)), "dark tile");
    assert_close(as255(render::cell_color(&b, 10, 10)), blend(base, (220, 210, 200)), "light tile");
}

#[test]
fn a_tile_is_one_texel_per_cell_wrapped_the_way_the_splat_path_samples() {
    // A 2x2 tile with four distinct texels: pins the `%` wrap, the
    // `(sy * tw + sx) * 4` offset, and that x and y are not transposed.
    let w = world();
    let plain = ctx(&w);
    let px = [(200u8, 10, 10), (10, 200, 10), (10, 10, 200), (200, 200, 10)];
    let mut rgba = Vec::new();
    for c in px {
        rgba.extend_from_slice(&[c.0, c.1, c.2, 255]);
    }
    let t = table(15, 5, GroundTile { w: 2, h: 2, rgba });

    // (10,10) -> texel (0,0); (11,10) -> (1,0); (10,11) -> (0,1);
    // (11,11) -> (1,1); (12,10) wraps back to (0,0).
    for (x, y, want) in [(10, 10, px[0]), (11, 10, px[1]), (10, 11, px[2]), (11, 11, px[3]), (12, 10, px[0])] {
        let g = grid(&[(y * GW + x, 6)]);
        let painted = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&t));
        let base = as255(render::cell_color(&plain, x, y));
        assert_close(as255(render::cell_color(&painted, x, y)), blend(base, want), &format!("cell ({x},{y})"));
    }
}

#[test]
fn a_ground_table_is_positional_and_only_its_own_painted_index_reads_it() {
    // `PACK_BIOME_SLOTS[n]` is painted index `n + 1`. An off-by-one here
    // silently re-points every tile in every pack ever authored, and would
    // still render *something* — which is why the neighbours are asserted
    // rather than only the hit.
    let w = world();
    let plain = ctx(&w);
    let tile = (255, 0, 255);
    let t = table(15, 5, solid(4, 4, tile));
    for v in 5..=7u8 {
        let i = 10 * GW + 10;
        let g = grid(&[(i, v)]);
        let painted = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&t));
        let base = as255(render::cell_color(&plain, 10, 10));
        let want = if v == 6 { blend(base, tile) } else { blend(base, CART_BIOME_COLS[v as usize - 1]) };
        assert_close(as255(render::cell_color(&painted, 10, 10)), want, &format!("biome index {v}"));
    }
}

#[test]
fn the_two_families_read_their_own_table_and_never_each_others() {
    // One tile, installed in the biome table only. A terrain cell painted at
    // the same index must still take its own flat swatch.
    let w = world();
    let plain = ctx(&w);
    let tile = (255, 0, 255);
    let t = table(15, 5, solid(4, 4, tile));
    let i = 10 * GW + 10;
    let g = grid(&[(i, 6)]);
    let base = as255(render::cell_color(&plain, 10, 10));

    let bio = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&t));
    assert_close(as255(render::cell_color(&bio, 10, 10)), blend(base, tile), "biome reads the biome table");

    let ter = ctx(&w).with_paint(None, Some(&g), None).with_ground_tiles(biome_tiles(&t));
    assert_close(as255(render::cell_color(&ter, 10, 10)), blend(base, CART_TERRAIN_COLS[5]), "terrain must not read it");

    // And the mirror: a terrain-table tile is invisible to a biome cell.
    let tt = table(13, 5, solid(4, 4, tile));
    let bio2 = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(terrain_tiles(&tt));
    assert_close(as255(render::cell_color(&bio2, 10, 10)), blend(base, CART_BIOME_COLS[5]), "biome must not read the terrain table");
}

#[test]
fn a_malformed_tile_falls_back_to_the_swatch_rather_than_panicking() {
    // `cartalith-rust-conventions`: `%` by zero panics in Rust where JS
    // yields NaN, and a truncated buffer would index off the end — either
    // one crosses the gdext boundary from inside a rayon `par_chunks_mut`.
    let w = world();
    let plain = ctx(&w);
    let i = 10 * GW + 10;
    let g = grid(&[(i, 6)]);
    let base = as255(render::cell_color(&plain, 10, 10));
    let broken = [
        GroundTile { w: 0, h: 0, rgba: Vec::new() },
        GroundTile { w: 4, h: 0, rgba: Vec::new() },
        // Declares 4x4 RGBA (64 bytes) and carries one pixel.
        GroundTile { w: 4, h: 4, rgba: vec![9, 9, 9, 255] },
    ];
    for (n, t) in broken.into_iter().enumerate() {
        let tb = table(15, 5, t);
        let painted = ctx(&w).with_paint(Some(&g), None, None).with_ground_tiles(biome_tiles(&tb));
        assert_close(as255(render::cell_color(&painted, 10, 10)), blend(base, CART_BIOME_COLS[5]), &format!("broken tile {n}"));
    }
}

#[test]
fn the_real_fixture_packs_ground_tiles_are_decoded_and_reach_the_map() {
    // End to end on the real reference-exported pack
    // (`cartalith-assets/tests/fixtures/reference_pack.zip`), whose
    // `pack.json` has carried `biomes/jungle.png` and `terrains/paved.png`
    // since milestone 2 — two files `load_pack_from_bytes` dropped on the
    // floor until this decode landed.
    let bytes = std::fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture must exist");
    let loaded = pack::load_pack_from_bytes(bytes).expect("real reference-exported pack must load");

    // Positional, full length, exactly one filled slot each: `jungle` is
    // `PACK_BIOME_SLOTS[5]` (painted index 6), `paved` is
    // `PACK_TERRAIN_SLOTS[0]` (painted index 1).
    assert_eq!(loaded.biomes.len(), 15);
    assert_eq!(loaded.terrains.len(), 13);
    assert_eq!(loaded.biomes.iter().filter(|t| t.is_some()).count(), 1);
    assert_eq!(loaded.terrains.iter().filter(|t| t.is_some()).count(), 1);
    let jungle = loaded.biomes[5].as_ref().expect("biomes/jungle.png must decode");
    let paved = loaded.terrains[0].as_ref().expect("terrains/paved.png must decode");
    assert_eq!((jungle.w, jungle.h), (512, 512));
    assert_eq!(jungle.rgba.len(), 512 * 512 * 4);
    // The fixture's own two solid colours, read straight out of the PNGs.
    assert_eq!(&jungle.rgba[..4], &[40, 100, 45, 255]);
    assert_eq!(&paved.rgba[..4], &[130, 130, 135, 255]);

    // ...and they change the map, at the reference's weight, on a cell the
    // brush painted with that index.
    let w = world();
    let plain = ctx(&w);
    let i = 10 * GW + 10;
    let gb = grid(&[(i, 6)]);
    let gt = grid(&[(i, 1)]);
    let ground = GroundTiles { biomes: &loaded.biomes, terrains: &loaded.terrains };
    let painted = ctx(&w).with_paint(Some(&gb), Some(&gt), None).with_ground_tiles(ground);
    let base = as255(render::cell_color(&plain, 10, 10));
    let want = blend(blend(base, (40, 100, 45)), (130, 130, 135));
    assert_close(as255(render::cell_color(&painted, 10, 10)), want, "fixture jungle then paved");

    // The same two cells with no pack attached take the flat swatches, which
    // are different colours — so this test fails if the decode regresses.
    let swatched = ctx(&w).with_paint(Some(&gb), Some(&gt), None);
    assert_ne!(render::cell_color(&painted, 10, 10), render::cell_color(&swatched, 10, 10));
}
