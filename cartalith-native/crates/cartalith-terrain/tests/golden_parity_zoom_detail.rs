//! Golden-parity tests for `addZoomDetail` (reference line 10467) — the
//! pyramid's progressive (fractal) zoom detail, the pass that makes deep zoom
//! reveal *more* relief instead of a smoother surface.
//!
//! # The harness
//!
//! The same one `golden_parity_pyramid.rs` (in `cartalith-spatial`) documents:
//! Node `vm.runInContext` over the **whole** `<script>` block #1 (2084-14556),
//! delimiters asserted, the probe appended to the block's own source so it
//! shares the block's `let` scope, and a truthy `indexedDB` stub so the boot
//! line does not auto-generate a 2048-wide world first.
//!
//! # The fixture and why both sides hash it first
//!
//! The same synthetic field `golden_parity_amplify.rs` uses — pure arithmetic,
//! no `sin`/`cos`/`exp`, so V8's libm and Rust's cannot disagree about the
//! *input* before the function under test runs, with a deliberately quantised
//! `% 11` term so distinct tiles are actually distinct. Both sides FNV-1a-64
//! the raw `f32` bytes and this test asserts the fixture hash matches before
//! trusting any other value.
//!
//! # Emptiness and shape assertions
//!
//! The extraction asserted, before any golden was written down: that
//! `z == zBase` and `z < zBase` are **byte-identical no-ops** (hash unchanged),
//! that `z > zBase` really does change the tile, and that the base tile itself
//! is non-constant. All four are re-asserted here.
//!
//! The three no-op cases matter more than they look: `pyramid_tile` calls
//! `add_zoom_detail` unconditionally at every level, so "shallow levels are
//! untouched" is a correctness property of the whole bake, not a micro-detail.

use cartalith_spatial::pyramid::pyramid_tile_bounds;
use cartalith_terrain::amplify::{add_zoom_detail, refine_tile, AmplifyOpts};

fn fnv_f32(a: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in a {
        for &b in &v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

/// The harness's own `mkField`, arithmetic for arithmetic.
fn synthetic_field(cw: usize, ch: usize, k: i64) -> Vec<f32> {
    let mut f = vec![0.0f32; cw * ch];
    let (cx, cy) = (cw as f64 * 0.42, ch as f64 * 0.55);
    let r2 = (cw as f64 * 0.3) * (ch as f64 * 0.3);
    for y in 0..ch {
        for x in 0..cw {
            let (dx, dy) = (x as f64 - cx, y as f64 - cy);
            let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
            let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
            v += 0.05 * ((q as f64 / 10.0) - 0.5);
            v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - ch as f64 * 0.25).abs() / (ch as f64 * 0.12));
            f[y * cw + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

const CW: usize = 48;
const CH: usize = 32;
const W: usize = 24;
const H: usize = 20;
/// The base tile every case starts from, before any zoom detail.
const BASE_HASH: &str = "0edba777203501a2";

fn base_opts() -> AmplifyOpts {
    AmplifyOpts { seed: 4242, sea: 0.42, detail_amp: 0.12, detail_freq: 1.0, ..Default::default() }
}

fn base_tile(coarse: &[f32]) -> Vec<f32> {
    let region =
        cartalith_spatial::Region { x: 0, y: 0, w: CW - 1, h: CH - 1 }.to_float();
    refine_tile(coarse, CW, CH, &region, 4, 4, 1, 1, W, H, &base_opts())
}

#[test]
fn the_fixture_is_bit_identical_to_the_harnesss() {
    let f = synthetic_field(CW, CH, 5);
    assert_eq!(fnv_f32(&f), "e6a8f7dd46187082", "the fixture itself diverged");
    assert!(f.iter().any(|&v| v != f[0]), "the fixture is constant");
    assert!(f.iter().all(|&v| (0.0..=1.0).contains(&v)), "the fixture escaped [0,1]");
}

#[test]
fn the_base_tile_is_bit_identical_before_any_zoom_detail() {
    // Stage N-1 verified before stage N, per `PARITY_TESTING.md`: if this
    // fails, nothing below it means anything.
    let f = synthetic_field(CW, CH, 5);
    let t = base_tile(&f);
    assert_eq!(fnv_f32(&t), BASE_HASH);
    assert!(t.iter().any(|&v| v != t[0]), "the base tile is constant");
}

#[test]
fn add_zoom_detail_matches_the_reference() {
    // (z, z_base, zoom_detail_k) -> hash after the pass. The harness built `b`
    // as pyramidTileBounds(CW, CH, max(0, z-2), 1, 1) in every case.
    let cases: &[(i32, i32, f64, &str)] = &[
        // z <= zBase: byte-identical no-ops, hash unchanged from BASE_HASH.
        (2, 2, 1.0, BASE_HASH),
        (1, 2, 1.0, BASE_HASH),
        // One extra octave.
        (3, 2, 1.0, "17c5aa06fabad73c"),
        // Three.
        (5, 2, 1.0, "293fa4f8907e308d"),
        // Four, at 1.5x the user's zoom-detail amount.
        (6, 2, 1.5, "b575cd09c884d5c7"),
        // Six -- the `Math.min(6, z - zBase)` ceiling, reached at z = 8.
        (8, 2, 1.0, "1fa3a00da5f27909"),
    ];
    let f = synthetic_field(CW, CH, 5);
    for &(z, z_base, zk, want) in cases {
        let b = pyramid_tile_bounds(CW, CH, (z - 2).max(0), 1, 1);
        let mut data = base_tile(&f);
        let opts = AmplifyOpts { z_base, zoom_detail_k: zk, ..base_opts() };
        add_zoom_detail(&mut data, W, H, &f, CW, CH, &b, z, &opts);
        assert_eq!(fnv_f32(&data), want, "z={z} zBase={z_base} k={zk}");
    }
}

#[test]
fn a_shallow_level_is_a_byte_identical_no_op_and_a_deep_one_is_not() {
    // Stated separately from the hash table because it is the property, not a
    // value: `pyramid_tile` calls this unconditionally at every level.
    let f = synthetic_field(CW, CH, 5);
    let b = pyramid_tile_bounds(CW, CH, 1, 1, 1);
    let base = base_tile(&f);

    let mut shallow = base.clone();
    add_zoom_detail(&mut shallow, W, H, &f, CW, CH, &b, 2, &base_opts());
    assert_eq!(shallow, base, "z == z_base must not touch a single byte");

    let mut deep = base.clone();
    add_zoom_detail(&mut deep, W, H, &f, CW, CH, &b, 5, &base_opts());
    assert_ne!(deep, base, "z > z_base must add detail");
}

#[test]
fn the_octave_ceiling_really_binds() {
    // `Math.min(6, z - zBase)`: z = 8 and z = 40 must produce the *same*
    // tile, because both cap at six octaves. A port that dropped the min
    // would pass every other test here and then take unbounded time at depth.
    let f = synthetic_field(CW, CH, 5);
    let b = pyramid_tile_bounds(CW, CH, 6, 1, 1);
    let mut a = base_tile(&f);
    let mut c = base_tile(&f);
    add_zoom_detail(&mut a, W, H, &f, CW, CH, &b, 8, &base_opts());
    add_zoom_detail(&mut c, W, H, &f, CW, CH, &b, 40, &base_opts());
    assert_eq!(a, c, "the six-octave ceiling is not binding");
}

#[test]
fn the_write_back_is_unclamped_exactly_as_the_reference_is() {
    // Added after mutation testing: inserting a `[0,1]` clamp on the write-back
    // survived every other case in this file, because none of them pushes a
    // value out of range. So the claim was checked against the reference
    // directly rather than assumed -- a cliff fixture with a 9.0 detail
    // amplitude, run through the real `addZoomDetail`, comes back spanning
    // [-0.963, 2.825]. `amplifyRegion` clamps; this pass does not, and a port
    // that "tidied" that would silently flatten every peak a deep bake touches.
    let cw = 16usize;
    let ch = 16usize;
    let mut hi = vec![0.0f32; cw * ch];
    for y in 0..ch {
        for x in 0..cw {
            hi[y * cw + x] = if x < cw / 2 { 0.55 } else { 0.999 };
        }
    }
    let (w, h) = (8usize, 8usize);
    let mut data = vec![0.999f32; w * h];
    let b = cartalith_spatial::FloatRegion { x: 0.0, y: 0.0, w: cw as f64 - 1.0, h: ch as f64 - 1.0 };
    let opts = AmplifyOpts {
        seed: 7,
        sea: 0.42,
        detail_amp: 9.0,
        detail_freq: 1.0,
        z_base: 2,
        zoom_detail_k: 1.0,
        ridged: false,
    };
    add_zoom_detail(&mut data, w, h, &hi, cw, ch, &b, 8, &opts);
    assert_eq!(fnv_f32(&data), "89d0a40e18e0f704");
    assert_eq!(data.iter().copied().fold(f32::NEG_INFINITY, f32::max), 2.825_417_041_778_564_5);
    assert_eq!(data.iter().copied().fold(f32::INFINITY, f32::min), -0.963_324_189_186_096_2);
}

#[test]
fn nothing_below_sea_level_is_touched() {
    // The reference's hard `if(base<sea) continue`, which is a *different*
    // rule from `amplify_region`'s smooth `underwater` fade -- a port that
    // reused the fade here would pass a "the tile changed" test and quietly
    // roughen every seabed.
    let f = vec![0.10f32; CW * CH];
    let b = pyramid_tile_bounds(CW, CH, 1, 0, 0);
    let mut data = vec![0.10f32; W * H];
    add_zoom_detail(&mut data, W, H, &f, CW, CH, &b, 8, &base_opts());
    assert!(data.iter().all(|&v| v == 0.10), "detail leaked below sea level");
}
