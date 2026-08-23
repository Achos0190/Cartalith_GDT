//! The refined-tile visual: hypsometric tint × hillshade — `UNIFIED_TOOL_PLAN.md`
//! milestone E2, the pixel half of `tilePngBytes`.
//!
//! Ported from `Cartalith Gen1 v2.10.html` block #1: `lerp`/`mix` (8304-8305),
//! the `SEA`/`LAND` palettes (8330-8331), `hypso` (8332), the four edge
//! extrapolators `edgeL`/`edgeR`/`edgeU`/`edgeD` (11606-11609) and
//! `renderHeightTileRGBA` (11610).
//!
//! # Why here
//!
//! `renderHeightTileRGBA` is a pure function of *a height tile* — plus three
//! scalars (sea level, sun azimuth, vertical exaggeration) that the reference
//! reads off `state` and this port takes as parameters. Its tint is a height
//! ramp and its shade is the normal-from-height formula `shadeFactor` already
//! uses, so it is a height formula start to finish: milestone E's own reason
//! for putting `amplify_region`/`refine_tile` in this crate, applied to the
//! next step of the same pipeline. Nothing here touches a canvas, an encoder or
//! Godot; PNG *containerisation* is a separate concern and lives with the
//! export composition (`cartalith_engine::region_export`), which reuses
//! `cartalith_assets::raster::encode_png` rather than growing a second one.
//!
//! # What the reference corrected
//!
//! **`Uint8ClampedArray` is not a cast.** `out[p]=c[0]*s` stores a *float* into
//! a clamped byte array, and the ECMA `ToUint8Clamp` conversion rounds to
//! nearest with **ties to even**, after clamping to `[0, 255]` and mapping NaN
//! to `0`. Since `c[0]*s` is fractional almost everywhere, a naive `as u8`
//! (which truncates) would be wrong in roughly half of all pixels. See
//! [`u8_clamped`].
//!
//! **`hypso` extrapolates past its own palette.** Below sea level the ramp
//! parameter `d = (sea - v) / sea` is not clamped, so a `v` far enough below
//! zero drives `mix` past `SEA[0]` and returns *negative* channel values —
//! e.g. `hypso(-0.1)` at `sea = 0.3` is `[-0.67, -10.67, -16.67]`. Verified
//! against the reference rather than assumed, and pinned by a golden. It is
//! harmless only because the clamped store catches it, which is one more
//! reason [`u8_clamped`] cannot be shortcut.
//!
//! **The border extrapolation is the seam fix, and it is load-bearing.** The
//! reference's own v1.29 note: every tile coloriser used to *clamp* the index
//! at the tile border, so the border column's central difference spanned one
//! cell instead of two and rendered at half its true slope — a 1px bright line
//! down every tile edge, measured at 5.05x the local mean colour discontinuity.
//! `edge_*` replace the clamp with a linear extrapolation of the missing
//! neighbour (`2·centre − inner`), which reproduces the interior's two-cell
//! scale exactly. Ported as the four separate functions the reference has,
//! because their asymmetries (`min(1, W-1)` on one side, `max(0, x-1)` on the
//! other) are easy to smooth over by accident.
//!
//! NaN flows through here for real: milestone E found `amplify_region` has a
//! genuine division by zero whose whole tile comes back NaN, so `js_max` — not
//! `f64::max` — guards the lambert term, and `v < sea` answers `false` for a
//! NaN `v`, sending it down the *land* branch. Both are the reference's
//! semantics, not a choice.

use crate::amplify::{js_max, js_min};

/// `SEA` (reference 8330): the bathymetric ramp, deepest first.
pub const SEA: [[f64; 3]; 3] = [[10.0, 28.0, 46.0], [26.0, 86.0, 140.0], [70.0, 140.0, 196.0]];

/// `LAND` (reference 8331): hypsometric stops as `(normalised height, rgb)`.
pub const LAND: [(f64, [f64; 3]); 6] = [
    (0.0, [47.0, 122.0, 68.0]),
    (0.18, [111.0, 154.0, 58.0]),
    (0.38, [201.0, 178.0, 74.0]),
    (0.58, [150.0, 112.0, 72.0]),
    (0.78, [140.0, 140.0, 140.0]),
    (1.0, [248.0, 248.0, 250.0]),
];

/// The sun's altitude, in degrees. Fixed in the reference (`alt = 40*PI/180`),
/// unlike the azimuth, which is a user setting.
pub const SUN_ALT_DEG: f64 = 40.0;

#[inline]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[inline]
fn mix(c1: [f64; 3], c2: [f64; 3], t: f64) -> [f64; 3] {
    [lerp(c1[0], c2[0], t), lerp(c1[1], c2[1], t), lerp(c1[2], c2[2], t)]
}

/// `hypso(v)` (reference 8332): a normalised height to its map colour.
///
/// Below `sea`, a two-segment ramp through [`SEA`] driven by relative depth;
/// above it, the [`LAND`] stops interpolated on height renormalised into
/// `[0, 1]`. Neither ramp parameter is clamped — see the module docs.
///
/// The two `|| 1`-style guards are the reference's: `sea <= 0` makes the depth
/// ramp read `0` (the shallowest sea colour), `1 - sea <= 0` makes the land
/// ramp read `0` (the lowest land colour), and a zero-width `LAND` interval
/// divides by `1` instead of by zero.
pub fn hypso(v: f64, sea: f64) -> [f64; 3] {
    if v < sea {
        let d = if sea <= 0.0 { 0.0 } else { (sea - v) / sea };
        return if d < 0.5 {
            mix(SEA[2], SEA[1], d / 0.5)
        } else {
            mix(SEA[1], SEA[0], (d - 0.5) / 0.5)
        };
    }
    let r = if (1.0 - sea) <= 0.0 { 0.0 } else { (v - sea) / (1.0 - sea) };
    for i in 0..LAND.len() - 1 {
        if r <= LAND[i + 1].0 {
            let span = LAND[i + 1].0 - LAND[i].0;
            let t = (r - LAND[i].0) / if span == 0.0 { 1.0 } else { span };
            return mix(LAND[i].1, LAND[i + 1].1, t);
        }
    }
    LAND[LAND.len() - 1].1
}

// ECMA-262 `ToUint8Clamp` -- what storing into a `Uint8ClampedArray` does,
// and the three-argument `Math.hypot` `renderHeightTileRGBA` calls. Both moved
// to `cartalith-jsmath`; re-exported because `u8_clamped` is this module's
// public API and `JS_SEMANTICS_AUDIT.md` §4.6 names it as the correct
// conversion the rest of the workspace is measured against.
pub use cartalith_jsmath::u8_clamped;
pub(crate) use cartalith_jsmath::js_hypot3;

/// `edgeL` (reference 11606). `ro` is the row offset `y * w`.
#[inline]
fn edge_l(t: &[f32], w: usize, x: usize, ro: usize) -> f64 {
    if x > 0 {
        t[ro + x - 1] as f64
    } else {
        2.0 * t[ro] as f64 - t[ro + js_min(1.0, (w - 1) as f64) as usize] as f64
    }
}

/// `edgeR` (reference 11607).
#[inline]
fn edge_r(t: &[f32], w: usize, x: usize, ro: usize) -> f64 {
    if x < w - 1 {
        t[ro + x + 1] as f64
    } else {
        2.0 * t[ro + x] as f64 - t[ro + js_max(0.0, x as f64 - 1.0) as usize] as f64
    }
}

/// `edgeU` (reference 11608).
#[inline]
fn edge_u(t: &[f32], w: usize, h: usize, x: usize, y: usize) -> f64 {
    if y > 0 {
        t[(y - 1) * w + x] as f64
    } else {
        2.0 * t[x] as f64 - t[js_min(1.0, (h - 1) as f64) as usize * w + x] as f64
    }
}

/// `edgeD` (reference 11609).
#[inline]
fn edge_d(t: &[f32], w: usize, h: usize, x: usize, y: usize) -> f64 {
    if y < h - 1 {
        t[(y + 1) * w + x] as f64
    } else {
        2.0 * t[y * w + x] as f64 - t[js_max(0.0, y as f64 - 1.0) as usize * w + x] as f64
    }
}

/// `renderHeightTileRGBA(tile, W, H)` (reference 11610): one refined tile as
/// RGBA8 pixels, row-major, four bytes per pixel, alpha always `255`.
///
/// `sea`, `sun_az_deg` and `exag` are `state.seaLevel`, `state.sunAz` and
/// `state.exag` — read off the global in the reference, passed in here so this
/// crate keeps having no opinion about application state.
///
/// The shade is a Lambert term against a light built from the azimuth and the
/// fixed [`SUN_ALT_DEG`] altitude, applied at different strengths on the two
/// sides of sea level (`0.75 + 0.25·sh` under water, `0.4 + 0.6·sh` above) so
/// the seabed stays readable.
///
/// # Panics
///
/// Panics if `tile` is shorter than `w * h`, or if either dimension is zero.
pub fn render_height_tile_rgba(
    tile: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    sun_az_deg: f64,
    exag: f64,
) -> Vec<u8> {
    assert!(w > 0 && h > 0, "render_height_tile_rgba needs a non-empty tile");
    assert!(tile.len() >= w * h, "tile is smaller than {w}x{h}");
    let mut out = vec![0u8; w * h * 4];
    let az = sun_az_deg * std::f64::consts::PI / 180.0;
    let alt = SUN_ALT_DEG * std::f64::consts::PI / 180.0;
    let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
    for y in 0..h {
        let ro = y * w;
        for x in 0..w {
            let i = ro + x;
            let v = tile[i] as f64;
            let l = edge_l(tile, w, x, ro);
            let r = edge_r(tile, w, x, ro);
            let u = edge_u(tile, w, h, x, y);
            let d = edge_d(tile, w, h, x, y);
            let (mut nx, mut ny, mut nz) = (-(r - l) * exag, -(d - u) * exag, 1.0_f64);
            let il = 1.0 / js_hypot3(nx, ny, nz);
            nx *= il;
            ny *= il;
            nz *= il;
            let sh = js_max(0.0, nx * lx + ny * ly + nz * lz);
            let c = hypso(v, sea);
            let s = if v < sea { 0.75 + 0.25 * sh } else { 0.4 + 0.6 * sh };
            let p = i * 4;
            out[p] = u8_clamped(c[0] * s);
            out[p + 1] = u8_clamped(c[1] * s);
            out[p + 2] = u8_clamped(c[2] * s);
            out[p + 3] = 255;
        }
    }
    out
}

/// The shade multiplier alone — `s` in [`render_height_tile_rgba`]'s own loop,
/// returned per pixel without the hypsometric tint it is normally multiplied
/// into.
///
/// # Why this exists
///
/// The reference picks its LOD tile coloriser off the view mode
/// (`_lodBuildTileRGBA`, reference 11148: `biome ? renderBiomeTileRGBA :
/// renderHeightTileRGBA`), so the height ramp is what *Relief* mode shows and
/// the full `landColorCore` look is what the default *Biome* mode shows. This
/// port has only the height-ramp half, and `cartalith-godot`'s deep-zoom
/// compositor needs the biome half: `lod_bridge` therefore composites the
/// *ratio* of this shade with and without `amplify_region`'s procedural detail
/// over the base map's own colours, which are already the biome look. See
/// `lod_bridge.rs`'s own module doc for the whole argument.
///
/// Deliberately **not** factored out of [`render_height_tile_rgba`], which is
/// pinned byte-for-byte against the reference by
/// `golden_parity_tile_render.rs`: the arithmetic below is that function's,
/// copied unchanged, so extracting it would buy nothing and risk a reordering
/// the goldens would then have to re-prove. `shade_tile(t, ..) [i]` equals the
/// `s` that call computes for pixel `i`, and the test below asserts exactly
/// that against the real function's output rather than restating the formula.
///
/// # Panics
///
/// Panics if `tile` is shorter than `w * h`, or if either dimension is zero —
/// the same preconditions [`render_height_tile_rgba`] has.
pub fn shade_tile(tile: &[f32], w: usize, h: usize, sea: f64, sun_az_deg: f64, exag: f64) -> Vec<f64> {
    assert!(w > 0 && h > 0, "shade_tile needs a non-empty tile");
    assert!(tile.len() >= w * h, "tile is smaller than {w}x{h}");
    let mut out = vec![0.0f64; w * h];
    let az = sun_az_deg * std::f64::consts::PI / 180.0;
    let alt = SUN_ALT_DEG * std::f64::consts::PI / 180.0;
    let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
    for y in 0..h {
        let ro = y * w;
        for x in 0..w {
            let i = ro + x;
            let v = tile[i] as f64;
            let l = edge_l(tile, w, x, ro);
            let r = edge_r(tile, w, x, ro);
            let u = edge_u(tile, w, h, x, y);
            let d = edge_d(tile, w, h, x, y);
            let (mut nx, mut ny, mut nz) = (-(r - l) * exag, -(d - u) * exag, 1.0_f64);
            let il = 1.0 / js_hypot3(nx, ny, nz);
            nx *= il;
            ny *= il;
            nz *= il;
            let sh = js_max(0.0, nx * lx + ny * ly + nz * lz);
            out[i] = if v < sea { 0.75 + 0.25 * sh } else { 0.4 + 0.6 * sh };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The harness's own `mkTile`, reproduced arithmetic-for-arithmetic: pure
    /// integer/rational arithmetic (no `sin`/`cos`/`exp`), with a deliberately
    /// quantised `% 11` term so the fixture reaches distinct colour bands
    /// rather than sliding smoothly through them.
    fn mk_tile(w: usize, h: usize, k: i64) -> Vec<f32> {
        let mut t = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let q = ((x as i64 * 7 + y as i64 * 13 + k) % 11) as f64 / 10.0;
                let denom = ((w - 1) * (h - 1) * 2) as f64;
                let v = 0.05
                    + 0.9 * ((x * (h - 1) + y * (w - 1)) as f64 / if denom == 0.0 { 1.0 } else { denom })
                    + 0.08 * (q - 0.5);
                // The harness's own `if(v<0)v=0; if(v>1)v=1;` -- identical for
                // a fixture that cannot be NaN.
                t[y * w + x] = v.clamp(0.0, 1.0) as f32;
            }
        }
        t
    }

    #[test]
    fn hypso_hits_its_palette_endpoints_exactly() {
        assert_eq!(hypso(0.0, 0.42), SEA[0]);
        assert_eq!(hypso(0.42, 0.42), LAND[0].1);
        assert_eq!(hypso(1.0, 0.42), LAND[5].1);
    }

    #[test]
    fn hypso_extrapolates_below_the_palette_rather_than_clamping() {
        // Real reference behaviour, not a bug being ported blind: the depth
        // ramp is unclamped, so a deep enough v drives mix() past SEA[0].
        let c = hypso(-0.1, 0.3);
        assert!(c[1] < 0.0 && c[2] < 0.0, "expected negative channels, got {c:?}");
        // ...and the clamped store is what makes it harmless.
        assert_eq!(u8_clamped(c[1]), 0);
    }

    #[test]
    fn hypso_survives_a_degenerate_sea_level_at_either_end() {
        // sea <= 0: nothing is below it except a negative v, which reads the
        // shallowest sea colour. 1 - sea <= 0: all land reads LAND[0].
        assert_eq!(hypso(-0.1, 0.0), SEA[2]);
        assert_eq!(hypso(1.0, 1.0), LAND[0].1);
    }

    #[test]
    fn render_emits_four_opaque_bytes_per_pixel() {
        let t = mk_tile(7, 5, 3);
        let px = render_height_tile_rgba(&t, 7, 5, 0.42, 315.0, 3.4);
        assert_eq!(px.len(), 7 * 5 * 4);
        assert!(px.iter().skip(3).step_by(4).all(|&a| a == 255));
    }

    #[test]
    fn render_is_not_flat() {
        // A silently-constant raster passes every structural check, so say it.
        let t = mk_tile(16, 11, 5);
        let px = render_height_tile_rgba(&t, 16, 11, 0.42, 315.0, 3.4);
        let distinct: std::collections::HashSet<u8> = px.iter().copied().collect();
        assert!(distinct.len() > 40, "only {} distinct byte values", distinct.len());
    }

    #[test]
    fn a_one_pixel_wide_tile_extrapolates_instead_of_indexing_out_of_range() {
        // edgeL/edgeR both fall to the extrapolating branch at W == 1, and
        // min(1, W-1) is what keeps them in range.
        let t = mk_tile(1, 6, 2);
        let px = render_height_tile_rgba(&t, 1, 6, 0.42, 315.0, 3.4);
        assert_eq!(px.len(), 24);
        let t = mk_tile(6, 1, 2);
        let px = render_height_tile_rgba(&t, 6, 1, 0.42, 200.0, 8.0);
        assert_eq!(px.len(), 24);
        let t = mk_tile(1, 1, 0);
        let px = render_height_tile_rgba(&t, 1, 1, 0.42, 315.0, 3.4);
        assert_eq!(px.len(), 4);
    }

    #[test]
    fn an_all_nan_tile_renders_black_and_opaque_rather_than_panicking() {
        // milestone E's amplifyRegion division by zero produces exactly this
        // tile. Math.max(0, NaN) is NaN, `v < sea` is false, and the clamped
        // store maps NaN to 0 -- so the reference draws a black square, and so
        // does this.
        let t = vec![f32::NAN; 9];
        let px = render_height_tile_rgba(&t, 3, 3, 0.42, 315.0, 3.4);
        for p in px.chunks(4) {
            assert_eq!(p, &[0, 0, 0, 255]);
        }
    }

    #[test]
    fn a_flat_tile_still_lights_from_the_azimuth() {
        // Zero gradient everywhere -> the normal is straight up -> sh == lz.
        let t = vec![0.7f32; 25];
        let px = render_height_tile_rgba(&t, 5, 5, 0.42, 315.0, 3.4);
        let first = &px[0..4];
        assert!(px.chunks(4).all(|p| p == first), "a flat tile must be one colour");
        assert_ne!(first, &[0, 0, 0, 255]);
    }

    #[test]
    fn the_sea_and_land_shade_bands_really_differ() {
        // 0.75+0.25*sh vs 0.4+0.6*sh: on a flat tile the same sh gives two
        // different multipliers, which is what stops a mutation swapping them
        // from going unnoticed.
        let below = render_height_tile_rgba(&[0.2f32; 4], 2, 2, 0.42, 315.0, 3.4);
        let above = render_height_tile_rgba(&[0.2f32; 4], 2, 2, 0.10, 315.0, 3.4);
        assert_ne!(below[0..3], above[0..3]);
    }

    #[test]
    fn exaggeration_changes_the_shading_but_not_the_tint_family() {
        let t = mk_tile(8, 8, 1);
        let a = render_height_tile_rgba(&t, 8, 8, 0.42, 315.0, 1.0);
        let b = render_height_tile_rgba(&t, 8, 8, 0.42, 315.0, 8.0);
        assert_ne!(a, b);
    }

    #[test]
    fn shade_tile_is_exactly_the_multiplier_render_height_tile_rgba_applies() {
        // The whole point of `shade_tile` existing separately: it must be the
        // same `s`, not a lookalike. Asserted by reconstruction -- tint the
        // hypso colour by this shade and the bytes must come back identical to
        // the pinned renderer's, for every pixel and every channel. A mutated
        // constant (0.4/0.6 <-> 0.75/0.25, a dropped `js_max`, a sign flip in
        // the light vector) breaks this immediately.
        for (w, h, sea) in [(7usize, 5usize, 0.42), (16, 11, 0.10), (5, 5, 0.9)] {
            let t = mk_tile(w, h, 3);
            let px = render_height_tile_rgba(&t, w, h, sea, 200.0, 3.4);
            let s = shade_tile(&t, w, h, sea, 200.0, 3.4);
            assert_eq!(s.len(), w * h);
            for i in 0..w * h {
                let c = hypso(t[i] as f64, sea);
                for k in 0..3 {
                    assert_eq!(
                        px[i * 4 + k],
                        u8_clamped(c[k] * s[i]),
                        "pixel {i} channel {k} at {w}x{h} sea {sea}"
                    );
                }
            }
        }
    }

    #[test]
    fn shade_tile_reaches_both_bands_and_is_not_flat() {
        // Non-emptiness and shape, stated rather than assumed (this port's own
        // "watch for silently-empty golden output" rule): a tile straddling sea
        // level must produce shades in both the underwater band (>= 0.75) and
        // the land band (< 0.75 wherever the Lambert term is small).
        let t = mk_tile(16, 16, 5);
        let s = shade_tile(&t, 16, 16, 0.5, 315.0, 3.4);
        assert!(t.iter().any(|v| (*v as f64) < 0.5), "fixture never goes underwater");
        assert!(t.iter().any(|v| (*v as f64) >= 0.5), "fixture never reaches land");
        for i in 0..s.len() {
            let floor = if (t[i] as f64) < 0.5 { 0.75 } else { 0.4 };
            assert!((floor..=1.0).contains(&s[i]), "pixel {i}: {} outside [{floor}, 1]", s[i]);
        }
        assert!(s.iter().any(|v| *v != s[0]), "a shade tile must not be flat");
    }
}
