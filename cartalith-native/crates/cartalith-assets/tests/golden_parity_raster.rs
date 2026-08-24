//! Golden-parity tests for the two milestone-6 (`ASSET_LIBRARY_SCOPE.md`)
//! functions that touch **no** DOM API at all — `fitToBottom` (reference line
//! 26769) and `finalizePackTexture` (line 12196), both pure arithmetic over
//! plain numbers/arrays.
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — the same technique every earlier milestone's golden tests use) that
//! lifts both functions straight out of the frozen HTML by line range and
//! calls them on the fixtures below. **The expected values here are that
//! run's output verbatim.**
//!
//! Every other pixel-touching function this milestone ships — `itemHash`,
//! `drawItemOnly`/`renderItem` (`render_item`), `encodeItemPng`/
//! `AssetImporter.decodeBytes`/`decodePackImage` (`encode_png`/`decode_png`)
//! — needs a live `HTMLCanvasElement`/`Image`/`Blob`, none of which exist in
//! this headless harness, so those are real unit tests instead
//! (`src/raster.rs`'s own `#[cfg(test)]` module documents why, function by
//! function).

use cartalith_assets::{ItemTransform, fit_to_bottom, finalize_pack_texture_inv_mean};

// ============================================================================
// fitToBottom
// ============================================================================

struct FitCase {
    w: u32,
    h: u32,
    size: u32,
    scale: f64,
    pan_x: f64,
    pan_y_before: f64,
    pan_y_after: f64,
}

// Captured verbatim from the reference's own `fitToBottom(item,size)`.
const FIT_CASES: &[FitCase] = &[
    FitCase { w: 100, h: 200, size: 256, scale: 1.0, pan_x: 0.0, pan_y_before: 0.0, pan_y_after: 0.0 },
    FitCase { w: 200, h: 100, size: 256, scale: 1.0, pan_x: 0.0, pan_y_before: 0.0, pan_y_after: 64.0 },
    FitCase { w: 100, h: 100, size: 256, scale: 1.0, pan_x: 5.0, pan_y_before: -5.0, pan_y_after: 0.0 },
    FitCase { w: 50, h: 150, size: 256, scale: 2.0, pan_x: 0.0, pan_y_before: 0.0, pan_y_after: -128.0 },
    FitCase { w: 150, h: 50, size: 256, scale: 0.5, pan_x: 10.0, pan_y_before: 10.0, pan_y_after: 106.66666666666666 },
    FitCase { w: 1, h: 1, size: 32, scale: 1.0, pan_x: 0.0, pan_y_before: 0.0, pan_y_after: 0.0 },
    FitCase { w: 300, h: 100, size: 512, scale: 1.3, pan_x: -20.0, pan_y_before: 40.0, pan_y_after: 145.06666666666666 },
];

#[test]
fn fit_to_bottom_matches_the_reference_on_every_fixture() {
    for c in FIT_CASES {
        let mut t = ItemTransform { scale: c.scale, pan_x: c.pan_x, pan_y: c.pan_y_before };
        fit_to_bottom(&mut t, c.w, c.h, c.size);
        assert_eq!(
            t.pan_y, c.pan_y_after,
            "w={} h={} size={} scale={} panX={}",
            c.w, c.h, c.size, c.scale, c.pan_x
        );
        // panX/scale are untouched -- fitToBottom only ever assigns panY.
        assert_eq!(t.pan_x, c.pan_x);
        assert_eq!(t.scale, c.scale);
    }
}

// ============================================================================
// finalizePackTexture's inverse means
// ============================================================================

/// Build a flat RGBA buffer the way the harness's `makeData` did, calling
/// `f(x,y) -> [r,g,b,a]` for every pixel.
fn make_data(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            data.extend_from_slice(&f(x, y));
        }
    }
    data
}

struct TexCase {
    name: &'static str,
    w: u32,
    h: u32,
    data: fn(u32, u32) -> Vec<u8>,
    inv: [f64; 3],
}

#[test]
fn finalize_pack_texture_inv_mean_matches_the_reference_on_every_fixture() {
    let cases: &[TexCase] = &[
        TexCase {
            name: "uniform_mid",
            w: 4,
            h: 4,
            data: |w, h| make_data(w, h, |_, _| [128, 64, 32, 255]),
            inv: [0.0078125, 0.015625, 0.03125],
        },
        TexCase {
            name: "near_black_clamped",
            w: 4,
            h: 4,
            data: |w, h| make_data(w, h, |_, _| [0, 0, 0, 255]),
            inv: [1.0, 1.0, 1.0],
        },
        TexCase {
            name: "mean_below_1_clamped",
            w: 4,
            h: 4,
            data: |w, h| {
                make_data(w, h, |x, y| if x == 0 && y == 0 { [16, 8, 4, 255] } else { [0, 0, 0, 255] })
            },
            inv: [1.0, 1.0, 1.0],
        },
        TexCase {
            name: "varying",
            w: 3,
            h: 2,
            data: |w, h| make_data(w, h, |x, y| [(x * 50 + 10) as u8, (y * 100 + 20) as u8, ((x + y) * 30) as u8, 255]),
            inv: [0.016666666666666666, 0.014285714285714285, 0.022222222222222223],
        },
        TexCase {
            name: "full_white",
            w: 2,
            h: 2,
            data: |w, h| make_data(w, h, |_, _| [255, 255, 255, 255]),
            inv: [0.00392156862745098, 0.00392156862745098, 0.00392156862745098],
        },
        TexCase { name: "zero_dim", w: 0, h: 0, data: |_, _| Vec::new(), inv: [0.0, 0.0, 0.0] },
    ];

    for c in cases {
        let data = (c.data)(c.w, c.h);
        let inv = finalize_pack_texture_inv_mean(c.w, c.h, &data);
        assert_eq!(inv, c.inv, "case {}", c.name);
    }
}
