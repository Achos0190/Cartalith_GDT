//! v2.25's `tileShadeExag(bounds, W)` (`RC_ENGINE_CHANGES.md` §4, followed
//! under Ruling AP): a zoomed LOD tile's hillshade exaggeration is scaled by
//! tile pixels per coarse cell, so its relief contrast tracks the main map's
//! instead of flattening as the view comes in.
//!
//! Three things are held here:
//!
//! - **The function**, against hand-computed literals — including the clamp
//!   and the reference's *"bounds omitted ⇒ the previous value exactly"*.
//! - **The split**: `js_reference()` keeps v2.11's bare `exag` (the golden in
//!   `golden_parity_tile_biome.rs` pins that path byte for byte), `default()`
//!   ships the v2.25 scaling, and at one tile pixel per coarse cell the two
//!   render byte-identical tiles (the clamp makes the factor exactly `1`).
//! - **The measurement the row asks for**: a deep tile's relief contrast
//!   against the base render over the same ground, before and after.
//!
//! # How "relief contrast" is measured, and why this way
//!
//! A pixel's luma carries its material colour as well as its shading, and the
//! material term does not depend on `exag` at all. So the relief signal is
//! taken as a **difference**: luma rendered at the shipped `exag`, minus luma
//! rendered at `exag = 0` (every normal `(0, 0, 1)`), at the same pixel. Its
//! standard deviation over **every** pixel of the tile, and over **every**
//! cell of the map inside the tile's footprint, is the contrast. No pixel is
//! selected by the value under test; the footprint is fixed before rendering
//! and the world is all land by construction, which is asserted.

#[path = "../src/render.rs"]
mod render;

use render::{RenderCtx, TerrainAppearance, TileBounds, TileFields};

const N: usize = 256;
const SEA: f64 = 0.42;
const LAT_N: f64 = 45.0;
const LAT_S: f64 = 35.0;

/// All land, with relief at two scales so the macro and meso normals both
/// have something to read. Uniform climate, so biome colour does not vary
/// across the footprint and the only thing `exag` can move is the shading.
fn world() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut field = vec![0f32; N * N];
    for y in 0..N {
        for x in 0..N {
            // Wavelengths of ~20 and ~7 cells: per-cell slopes of a few
            // hundredths, the order a real world's hills have, so the map's
            // own hillshade has a signal to compare against.
            let (u, v) = (x as f64, y as f64);
            let h = 0.66 + 0.12 * (u * 0.31 + 0.4).sin() * (v * 0.27).cos() + 0.05 * (u * 0.9).sin() * (v * 0.77 + 1.0).cos();
            field[y * N + x] = h as f32;
        }
    }
    (field, vec![12f32; N * N], vec![0.5f32; N * N])
}

/// Bilinear read of the grid at a coarse coordinate — the tile's height with
/// no amplified detail added, so the test measures the shading gain alone and
/// not new relief.
fn bilinear(f: &[f32], x: f64, y: f64) -> f32 {
    let x0 = (x.floor() as usize).min(N - 1);
    let y0 = (y.floor() as usize).min(N - 1);
    let x1 = (x0 + 1).min(N - 1);
    let y1 = (y0 + 1).min(N - 1);
    let (tx, ty) = (x - x0 as f64, y - y0 as f64);
    let a = f[y0 * N + x0] as f64 * (1.0 - tx) + f[y0 * N + x1] as f64 * tx;
    let b = f[y1 * N + x0] as f64 * (1.0 - tx) + f[y1 * N + x1] as f64 * tx;
    (a * (1.0 - ty) + b * ty) as f32
}

fn luma(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn std_dev(v: &[f64]) -> f64 {
    assert!(v.len() > 16, "too few samples ({}) for a spread", v.len());
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
}

fn ctx<'a>(f: &'a [f32], t: &'a [f32], r: &'a [f32], a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(f, t, r, None, N, N, SEA, false, LAT_N, LAT_S, a)
}

/// Luma of a tile (0..255) rendered with `a`, one value per pixel.
fn tile_luma(field: &[f32], t: &[f32], r: &[f32], a: TerrainAppearance, b: TileBounds, w: usize) -> Vec<f64> {
    let c = ctx(field, t, r, a);
    let tf = TileFields::new(&c, None).without_lakes();
    let cx = b.w / (w - 1) as f64;
    let cy = b.h / (w - 1) as f64;
    let mut tile = vec![0f32; w * w];
    for y in 0..w {
        for x in 0..w {
            tile[y * w + x] = bilinear(field, b.x + x as f64 * cx, b.y + y as f64 * cy);
        }
    }
    let rgba = render::render_biome_tile_rgba(&c, &tile, w, w, b, &tf);
    assert_eq!(rgba.len(), w * w * 4, "the tile came back empty or mis-sized");
    rgba.chunks(4).map(|p| luma(p[0] as f64, p[1] as f64, p[2] as f64)).collect()
}

/// Luma of the main map (0..255) over the cells `[x0, x0 + k] x [y0, y0 + k]`.
fn map_luma(field: &[f32], t: &[f32], r: &[f32], a: TerrainAppearance, x0: usize, y0: usize, k: usize) -> Vec<f64> {
    let c = ctx(field, t, r, a);
    let mut out = Vec::new();
    for y in y0..=y0 + k {
        for x in x0..=x0 + k {
            let (cr, cg, cb) = render::cell_color(&c, x, y);
            out.push(luma(cr, cg, cb) * 255.0);
        }
    }
    out
}

fn relief(shaded: &[f64], flat: &[f64]) -> f64 {
    assert_eq!(shaded.len(), flat.len());
    let d: Vec<f64> = shaded.iter().zip(flat).map(|(s, f)| s - f).collect();
    std_dev(&d)
}

#[test]
fn tile_shade_exag_is_the_v2_25_expression() {
    // 128 px over 16 cells: 8 px per cell, so 3.4 * 8.
    assert_eq!(render::tile_shade_exag(3.4, 16.0, 129), 27.2);
    // Coarser than the grid (0.5 px per cell): clamped at 1, never reduced.
    assert_eq!(render::tile_shade_exag(3.4, 256.0, 129), 3.4);
    // Exactly one px per cell: the factor is exactly 1.
    assert_eq!(render::tile_shade_exag(3.4, 255.0, 256), 3.4);
    // Bounds that cannot be divided by: the previous value exactly.
    assert_eq!(render::tile_shade_exag(3.4, 0.0, 129), 3.4);
    assert_eq!(render::tile_shade_exag(3.4, f64::NAN, 129), 3.4);
    assert_eq!(render::tile_shade_exag(3.4, -4.0, 129), 3.4);
}

#[test]
fn the_parity_path_keeps_v2_11_and_the_app_ships_v2_25() {
    assert!(!TerrainAppearance::js_reference().tile_shade_exag_scaled);
    assert!(TerrainAppearance::default().tile_shade_exag_scaled);
}

/// At one tile pixel per coarse cell the factor is exactly `1`, so the flag
/// must not move a single byte — the property that keeps LOD entry, the
/// screen-identity check and the main map untouched.
#[test]
fn at_one_pixel_per_cell_the_flag_changes_nothing() {
    let (f, t, r) = world();
    let whole = TileBounds { x: 0.0, y: 0.0, w: (N - 1) as f64, h: (N - 1) as f64 };
    let on = TerrainAppearance::default();
    let off = TerrainAppearance { tile_shade_exag_scaled: false, ..TerrainAppearance::default() };
    let a = tile_luma(&f, &t, &r, on, whole, N);
    let b = tile_luma(&f, &t, &r, off, whole, N);
    assert_eq!(a, b, "the flag moved a tile drawn at one pixel per cell");
}

/// The row's acceptance: a deep tile's relief contrast against the base
/// render over the same ground, before (v2.11 bare `exag`) and after (v2.25
/// `tileShadeExag`), at 1, 2, 4 and 8 tile pixels per coarse cell.
///
/// # What "tracks" means here, measured rather than assumed
///
/// The first run of this test asserted `tile / map` near `1` after the change
/// and failed at `2.57`. The reason is not this change: **at one pixel per
/// cell, where the flag is provably inert, the tile already carries `2.20x`
/// the map's relief signal on this fixture** (`1.44x` under `js_reference()`).
/// That is `render.rs`'s documented departure 2 — the tile's meso shade has
/// no `/ ms` normaliser, the map's `shadeFactor2` has one — plus the two
/// paths' different light models, and it is what
/// `golden_parity_tile_biome.rs`'s `screen_residual_at_lod_entry_*` measures.
/// `tileShadeExag` does not touch it and should not.
///
/// What `tileShadeExag` fixes is the **zoom dependence**: the ratio of the
/// tile's relief to the map's, taken at depth and divided by the same ratio
/// at LOD entry. Measured 2026-09-24 on this fixture (map 8.583 luma levels):
///
/// | px/cell | before: tile, ratio to LOD entry | after: tile, ratio to LOD entry |
/// |---|---|---|
/// | 1 | 18.898, 1.000 | 18.898, 1.000 |
/// | 2 | 10.746, 0.569 | 20.702, 1.095 |
/// | 4 | 5.676, 0.300 | 21.516, 1.139 |
/// | 8 | 2.938, 0.155 | 22.098, 1.169 |
///
/// Before, relief falls as `1 / (px per cell)` — halved per zoom octave —
/// which is RC_ENGINE_CHANGES §4's *"relief flattened exactly where the LOD
/// viewer exists to show it"*. After, it holds within 17% of its LOD-entry
/// value over three octaves; the residual rise is the tile's own meso and
/// micro bands reading a smoother (bilinear) surface at a finer step.
#[test]
fn a_zoomed_tiles_relief_contrast_tracks_the_main_maps() {
    let (f, t, r) = world();
    let (x0, y0, k) = (96usize, 96usize, 16usize);
    for y in y0..=y0 + k {
        for x in x0..=x0 + k {
            assert!(f[y * N + x] as f64 > SEA + 0.02, "the footprint is meant to be all land");
        }
    }
    let b = TileBounds { x: x0 as f64, y: y0 as f64, w: k as f64, h: k as f64 };
    let shipped = TerrainAppearance::default();
    let flat = TerrainAppearance { exag: 0.0, ..TerrainAppearance::default() };
    let before = TerrainAppearance { tile_shade_exag_scaled: false, ..TerrainAppearance::default() };

    let map = relief(&map_luma(&f, &t, &r, shipped.clone(), x0, y0, k), &map_luma(&f, &t, &r, flat.clone(), x0, y0, k));
    assert!(map > 1.0, "the map's own relief signal is {map:.4} luma levels -- the fixture has no relief to compare");

    // (tile relief / map relief) at `ppc` pixels per cell, for one appearance.
    let ratio = |a: &TerrainAppearance, ppc: usize| -> f64 {
        let w = k * ppc + 1;
        let t0 = tile_luma(&f, &t, &r, flat.clone(), b, w);
        relief(&tile_luma(&f, &t, &r, a.clone(), b, w), &t0) / map
    };
    let entry_before = ratio(&before, 1);
    let entry_after = ratio(&shipped, 1);
    assert_eq!(entry_before, entry_after, "at LOD entry the flag must be inert");
    println!("map relief {map:.4}; tile/map at LOD entry {entry_after:.4}");
    let mut prev_before = 1.0;
    for ppc in [2usize, 4, 8] {
        let tb = ratio(&before, ppc) / entry_before;
        let ta = ratio(&shipped, ppc) / entry_after;
        println!("{ppc} px/cell: tracking before {tb:.4}, after {ta:.4}");
        // Before: relief collapses with zoom, roughly as 1 / ppc.
        assert!(tb < prev_before, "before the change relief should fall with every octave ({ppc} px/cell: {tb:.4})");
        assert!(tb < 1.25 / ppc as f64, "before the change relief should fall about as 1/ppc ({ppc} px/cell: {tb:.4})");
        prev_before = tb;
        // After: it holds near its LOD-entry value.
        assert!((0.85..=1.35).contains(&ta), "after the change a deep tile should keep the map's relief; {ppc} px/cell: {ta:.4}");
    }
}
