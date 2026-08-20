//! Golden-parity tests for the sprite-sheet slicer (`ASSET_LIBRARY_SCOPE.md`
//! milestone 8; `GUI_GAP_REGISTER.md` AS-09/AS-10/AS-11).
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — the same technique every earlier milestone's golden tests use) that
//! lifts the whole `SpriteSheetImporter` object literal straight out of the
//! frozen HTML by line range and calls its pure methods. **The expected
//! values below are that run's output verbatim.**
//!
//! # The line range, and why the lift works at all
//!
//! Lines **27465-27870** — `const SpriteSheetImporter={` through its closing
//! `};`. Verified at both ends by the harness itself before evaluating
//! anything (`CLAUDE.md`: "verify a scope document's line ranges against the
//! real reference before slicing" — a start that is too late does not fail to
//! parse, it silently omits a definition), plus a presence assertion for each
//! of the four functions under test. The literal touches no DOM at
//! *definition* time (every property initialiser is a primitive, an array, a
//! `Set` or an object literal), which is what makes a `vm` sandbox with four
//! stubs enough to run it.
//!
//! # What is golden here, and what deliberately is not
//!
//! - `computeCells` (line 27590) — the cell geometry, in full. Stubbed `E()`
//!   supplies the three DOM inputs.
//! - `cropCell` (line 27773) — its **source-rect rounding**, captured by
//!   stubbing `document.createElement('canvas')` and recording the arguments
//!   the reference passes to `ctx.drawImage`. The blit itself is a DOM API
//!   with no headless equivalent, so the pixel copy is covered by real unit
//!   tests in `src/slicer.rs` instead — the same carve-out `src/raster.rs`
//!   already draws for `renderItem`/`itemHash`.
//! - `isBlank` (line 27768) and `applyChroma` (line 27603) — in full, over a
//!   fake `ctx` whose `getImageData` hands back a plain array the reference's
//!   own loops then mutate.
//!
//! `trim_transparent_edges` has **no** fixtures here on purpose: the
//! reference slicer has no trim operation at all (see `src/slicer.rs`'s
//! module docs), so there is nothing to be golden against. It is unit-tested.

use cartalith_assets::{
    ChromaKey, DecodedImage, GridRect, SliceGrid, apply_chroma, cell_source_rect, compute_cells,
    is_blank,
};

// ============================================================================
// computeCells
// ============================================================================

struct GridCase {
    name: &'static str,
    rect: GridRect,
    cols: i64,
    rows: i64,
    spacing: f64,
    min_w: f64,
    min_h: f64,
    /// `(col, row, index, x, y, w, h)` per cell, in the reference's own order.
    cells: &'static [(u32, u32, usize, f64, f64, f64, f64)],
}

/// Captured verbatim from the reference's own `computeCells()`. The
/// fixtures are shaped to reach the code rather than to look plausible: an
/// odd spacing so the half-gutter lands on a `.5`, a span that does not
/// divide evenly so the float residue shows, a spacing wider than the cell so
/// the negative-extent branch is exercised, out-of-range counts so
/// `clampInt` is, and a non-numeric spacing so `parseFloat(...)||0` is.
const GRID_CASES: &[GridCase] = &[
    GridCase {
        name: "plain 4x4, no spacing, full 3072x2048 sheet",
        rect: GridRect { x: 0.0, y: 0.0, w: 3072.0, h: 2048.0 },
        cols: 4, rows: 4, spacing: 0.0,
        min_w: 768.0, min_h: 512.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 768.0, 512.0),
            (1, 0, 1, 768.0, 0.0, 768.0, 512.0),
            (2, 0, 2, 1536.0, 0.0, 768.0, 512.0),
            (3, 0, 3, 2304.0, 0.0, 768.0, 512.0),
            (0, 1, 4, 0.0, 512.0, 768.0, 512.0),
            (1, 1, 5, 768.0, 512.0, 768.0, 512.0),
            (2, 1, 6, 1536.0, 512.0, 768.0, 512.0),
            (3, 1, 7, 2304.0, 512.0, 768.0, 512.0),
            (0, 2, 8, 0.0, 1024.0, 768.0, 512.0),
            (1, 2, 9, 768.0, 1024.0, 768.0, 512.0),
            (2, 2, 10, 1536.0, 1024.0, 768.0, 512.0),
            (3, 2, 11, 2304.0, 1024.0, 768.0, 512.0),
            (0, 3, 12, 0.0, 1536.0, 768.0, 512.0),
            (1, 3, 13, 768.0, 1536.0, 768.0, 512.0),
            (2, 3, 14, 1536.0, 1536.0, 768.0, 512.0),
            (3, 3, 15, 2304.0, 1536.0, 768.0, 512.0),
        ],
    },
    GridCase {
        name: "6x4 with spacing 8",
        rect: GridRect { x: 0.0, y: 0.0, w: 3072.0, h: 2048.0 },
        cols: 6, rows: 4, spacing: 8.0,
        min_w: 504.0, min_h: 504.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 508.0, 508.0),
            (1, 0, 1, 516.0, 0.0, 504.0, 508.0),
            (2, 0, 2, 1028.0, 0.0, 504.0, 508.0),
            (3, 0, 3, 1540.0, 0.0, 504.0, 508.0),
            (4, 0, 4, 2052.0, 0.0, 504.0, 508.0),
            (5, 0, 5, 2564.0, 0.0, 508.0, 508.0),
            (0, 1, 6, 0.0, 516.0, 508.0, 504.0),
            (1, 1, 7, 516.0, 516.0, 504.0, 504.0),
            (2, 1, 8, 1028.0, 516.0, 504.0, 504.0),
            (3, 1, 9, 1540.0, 516.0, 504.0, 504.0),
            (4, 1, 10, 2052.0, 516.0, 504.0, 504.0),
            (5, 1, 11, 2564.0, 516.0, 508.0, 504.0),
            (0, 2, 12, 0.0, 1028.0, 508.0, 504.0),
            (1, 2, 13, 516.0, 1028.0, 504.0, 504.0),
            (2, 2, 14, 1028.0, 1028.0, 504.0, 504.0),
            (3, 2, 15, 1540.0, 1028.0, 504.0, 504.0),
            (4, 2, 16, 2052.0, 1028.0, 504.0, 504.0),
            (5, 2, 17, 2564.0, 1028.0, 508.0, 504.0),
            (0, 3, 18, 0.0, 1540.0, 508.0, 508.0),
            (1, 3, 19, 516.0, 1540.0, 504.0, 508.0),
            (2, 3, 20, 1028.0, 1540.0, 504.0, 508.0),
            (3, 3, 21, 1540.0, 1540.0, 504.0, 508.0),
            (4, 3, 22, 2052.0, 1540.0, 504.0, 508.0),
            (5, 3, 23, 2564.0, 1540.0, 508.0, 508.0),
        ],
    },
    GridCase {
        name: "3x2 inset by a 16px margin",
        rect: GridRect { x: 16.0, y: 16.0, w: 288.0, h: 224.0 },
        cols: 3, rows: 2, spacing: 0.0,
        min_w: 96.0, min_h: 112.0,
        cells: &[
            (0, 0, 0, 16.0, 16.0, 96.0, 112.0),
            (1, 0, 1, 112.0, 16.0, 96.0, 112.0),
            (2, 0, 2, 208.0, 16.0, 96.0, 112.0),
            (0, 1, 3, 16.0, 128.0, 96.0, 112.0),
            (1, 1, 4, 112.0, 128.0, 96.0, 112.0),
            (2, 1, 5, 208.0, 128.0, 96.0, 112.0),
        ],
    },
    GridCase {
        name: "3x2 inset by 16px margin, spacing 5 (odd -> half-gutter is .5)",
        rect: GridRect { x: 16.0, y: 16.0, w: 288.0, h: 224.0 },
        cols: 3, rows: 2, spacing: 5.0,
        min_w: 91.0, min_h: 109.5,
        cells: &[
            (0, 0, 0, 16.0, 16.0, 93.5, 109.5),
            (1, 0, 1, 114.5, 16.0, 91.0, 109.5),
            (2, 0, 2, 210.5, 16.0, 93.5, 109.5),
            (0, 1, 3, 16.0, 130.5, 93.5, 109.5),
            (1, 1, 4, 114.5, 130.5, 91.0, 109.5),
            (2, 1, 5, 210.5, 130.5, 93.5, 109.5),
        ],
    },
    GridCase {
        name: "1x1 degenerate (no interior edges at all)",
        rect: GridRect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 },
        cols: 1, rows: 1, spacing: 20.0,
        min_w: 100.0, min_h: 100.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 100.0, 100.0),
        ],
    },
    GridCase {
        name: "5x3 on a size that does not divide evenly",
        rect: GridRect { x: 7.0, y: 11.0, w: 101.0, h: 67.0 },
        cols: 5, rows: 3, spacing: 3.0,
        min_w: 17.19999999999999, min_h: 19.333333333333336,
        cells: &[
            (0, 0, 0, 7.0, 11.0, 18.700000000000003, 20.83333333333333),
            (1, 0, 1, 28.700000000000003, 11.0, 17.200000000000003, 20.83333333333333),
            (2, 0, 2, 48.900000000000006, 11.0, 17.19999999999999, 20.83333333333333),
            (3, 0, 3, 69.1, 11.0, 17.200000000000017, 20.83333333333333),
            (4, 0, 4, 89.30000000000001, 11.0, 18.69999999999999, 20.83333333333333),
            (0, 1, 5, 7.0, 34.83333333333333, 18.700000000000003, 19.333333333333336),
            (1, 1, 6, 28.700000000000003, 34.83333333333333, 17.200000000000003, 19.333333333333336),
            (2, 1, 7, 48.900000000000006, 34.83333333333333, 17.19999999999999, 19.333333333333336),
            (3, 1, 8, 69.1, 34.83333333333333, 17.200000000000017, 19.333333333333336),
            (4, 1, 9, 89.30000000000001, 34.83333333333333, 18.69999999999999, 19.333333333333336),
            (0, 2, 10, 7.0, 57.166666666666664, 18.700000000000003, 20.833333333333336),
            (1, 2, 11, 28.700000000000003, 57.166666666666664, 17.200000000000003, 20.833333333333336),
            (2, 2, 12, 48.900000000000006, 57.166666666666664, 17.19999999999999, 20.833333333333336),
            (3, 2, 13, 69.1, 57.166666666666664, 17.200000000000017, 20.833333333333336),
            (4, 2, 14, 89.30000000000001, 57.166666666666664, 18.69999999999999, 20.833333333333336),
        ],
    },
    GridCase {
        name: "spacing wider than the cell (negative w/h)",
        rect: GridRect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 },
        cols: 4, rows: 4, spacing: 200.0,
        min_w: -175.0, min_h: -175.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, -75.0, -75.0),
            (1, 0, 1, 125.0, 0.0, -175.0, -75.0),
            (2, 0, 2, 150.0, 0.0, -175.0, -75.0),
            (3, 0, 3, 175.0, 0.0, -75.0, -75.0),
            (0, 1, 4, 0.0, 125.0, -75.0, -175.0),
            (1, 1, 5, 125.0, 125.0, -175.0, -175.0),
            (2, 1, 6, 150.0, 125.0, -175.0, -175.0),
            (3, 1, 7, 175.0, 125.0, -75.0, -175.0),
            (0, 2, 8, 0.0, 150.0, -75.0, -175.0),
            (1, 2, 9, 125.0, 150.0, -175.0, -175.0),
            (2, 2, 10, 150.0, 150.0, -175.0, -175.0),
            (3, 2, 11, 175.0, 150.0, -75.0, -175.0),
            (0, 3, 12, 0.0, 175.0, -75.0, -75.0),
            (1, 3, 13, 125.0, 175.0, -175.0, -75.0),
            (2, 3, 14, 150.0, 175.0, -175.0, -75.0),
            (3, 3, 15, 175.0, 175.0, -75.0, -75.0),
        ],
    },
    GridCase {
        name: "clamped cols/rows (0 -> 1, 999 -> 128)",
        rect: GridRect { x: 0.0, y: 0.0, w: 128.0, h: 256.0 },
        cols: 0, rows: 999, spacing: 0.0,
        min_w: 128.0, min_h: 2.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 128.0, 2.0),
            (0, 1, 1, 0.0, 2.0, 128.0, 2.0),
            (0, 2, 2, 0.0, 4.0, 128.0, 2.0),
            (0, 3, 3, 0.0, 6.0, 128.0, 2.0),
            (0, 4, 4, 0.0, 8.0, 128.0, 2.0),
            (0, 5, 5, 0.0, 10.0, 128.0, 2.0),
            (0, 6, 6, 0.0, 12.0, 128.0, 2.0),
            (0, 7, 7, 0.0, 14.0, 128.0, 2.0),
            (0, 8, 8, 0.0, 16.0, 128.0, 2.0),
            (0, 9, 9, 0.0, 18.0, 128.0, 2.0),
            (0, 10, 10, 0.0, 20.0, 128.0, 2.0),
            (0, 11, 11, 0.0, 22.0, 128.0, 2.0),
            (0, 12, 12, 0.0, 24.0, 128.0, 2.0),
            (0, 13, 13, 0.0, 26.0, 128.0, 2.0),
            (0, 14, 14, 0.0, 28.0, 128.0, 2.0),
            (0, 15, 15, 0.0, 30.0, 128.0, 2.0),
            (0, 16, 16, 0.0, 32.0, 128.0, 2.0),
            (0, 17, 17, 0.0, 34.0, 128.0, 2.0),
            (0, 18, 18, 0.0, 36.0, 128.0, 2.0),
            (0, 19, 19, 0.0, 38.0, 128.0, 2.0),
            (0, 20, 20, 0.0, 40.0, 128.0, 2.0),
            (0, 21, 21, 0.0, 42.0, 128.0, 2.0),
            (0, 22, 22, 0.0, 44.0, 128.0, 2.0),
            (0, 23, 23, 0.0, 46.0, 128.0, 2.0),
            (0, 24, 24, 0.0, 48.0, 128.0, 2.0),
            (0, 25, 25, 0.0, 50.0, 128.0, 2.0),
            (0, 26, 26, 0.0, 52.0, 128.0, 2.0),
            (0, 27, 27, 0.0, 54.0, 128.0, 2.0),
            (0, 28, 28, 0.0, 56.0, 128.0, 2.0),
            (0, 29, 29, 0.0, 58.0, 128.0, 2.0),
            (0, 30, 30, 0.0, 60.0, 128.0, 2.0),
            (0, 31, 31, 0.0, 62.0, 128.0, 2.0),
            (0, 32, 32, 0.0, 64.0, 128.0, 2.0),
            (0, 33, 33, 0.0, 66.0, 128.0, 2.0),
            (0, 34, 34, 0.0, 68.0, 128.0, 2.0),
            (0, 35, 35, 0.0, 70.0, 128.0, 2.0),
            (0, 36, 36, 0.0, 72.0, 128.0, 2.0),
            (0, 37, 37, 0.0, 74.0, 128.0, 2.0),
            (0, 38, 38, 0.0, 76.0, 128.0, 2.0),
            (0, 39, 39, 0.0, 78.0, 128.0, 2.0),
            (0, 40, 40, 0.0, 80.0, 128.0, 2.0),
            (0, 41, 41, 0.0, 82.0, 128.0, 2.0),
            (0, 42, 42, 0.0, 84.0, 128.0, 2.0),
            (0, 43, 43, 0.0, 86.0, 128.0, 2.0),
            (0, 44, 44, 0.0, 88.0, 128.0, 2.0),
            (0, 45, 45, 0.0, 90.0, 128.0, 2.0),
            (0, 46, 46, 0.0, 92.0, 128.0, 2.0),
            (0, 47, 47, 0.0, 94.0, 128.0, 2.0),
            (0, 48, 48, 0.0, 96.0, 128.0, 2.0),
            (0, 49, 49, 0.0, 98.0, 128.0, 2.0),
            (0, 50, 50, 0.0, 100.0, 128.0, 2.0),
            (0, 51, 51, 0.0, 102.0, 128.0, 2.0),
            (0, 52, 52, 0.0, 104.0, 128.0, 2.0),
            (0, 53, 53, 0.0, 106.0, 128.0, 2.0),
            (0, 54, 54, 0.0, 108.0, 128.0, 2.0),
            (0, 55, 55, 0.0, 110.0, 128.0, 2.0),
            (0, 56, 56, 0.0, 112.0, 128.0, 2.0),
            (0, 57, 57, 0.0, 114.0, 128.0, 2.0),
            (0, 58, 58, 0.0, 116.0, 128.0, 2.0),
            (0, 59, 59, 0.0, 118.0, 128.0, 2.0),
            (0, 60, 60, 0.0, 120.0, 128.0, 2.0),
            (0, 61, 61, 0.0, 122.0, 128.0, 2.0),
            (0, 62, 62, 0.0, 124.0, 128.0, 2.0),
            (0, 63, 63, 0.0, 126.0, 128.0, 2.0),
            (0, 64, 64, 0.0, 128.0, 128.0, 2.0),
            (0, 65, 65, 0.0, 130.0, 128.0, 2.0),
            (0, 66, 66, 0.0, 132.0, 128.0, 2.0),
            (0, 67, 67, 0.0, 134.0, 128.0, 2.0),
            (0, 68, 68, 0.0, 136.0, 128.0, 2.0),
            (0, 69, 69, 0.0, 138.0, 128.0, 2.0),
            (0, 70, 70, 0.0, 140.0, 128.0, 2.0),
            (0, 71, 71, 0.0, 142.0, 128.0, 2.0),
            (0, 72, 72, 0.0, 144.0, 128.0, 2.0),
            (0, 73, 73, 0.0, 146.0, 128.0, 2.0),
            (0, 74, 74, 0.0, 148.0, 128.0, 2.0),
            (0, 75, 75, 0.0, 150.0, 128.0, 2.0),
            (0, 76, 76, 0.0, 152.0, 128.0, 2.0),
            (0, 77, 77, 0.0, 154.0, 128.0, 2.0),
            (0, 78, 78, 0.0, 156.0, 128.0, 2.0),
            (0, 79, 79, 0.0, 158.0, 128.0, 2.0),
            (0, 80, 80, 0.0, 160.0, 128.0, 2.0),
            (0, 81, 81, 0.0, 162.0, 128.0, 2.0),
            (0, 82, 82, 0.0, 164.0, 128.0, 2.0),
            (0, 83, 83, 0.0, 166.0, 128.0, 2.0),
            (0, 84, 84, 0.0, 168.0, 128.0, 2.0),
            (0, 85, 85, 0.0, 170.0, 128.0, 2.0),
            (0, 86, 86, 0.0, 172.0, 128.0, 2.0),
            (0, 87, 87, 0.0, 174.0, 128.0, 2.0),
            (0, 88, 88, 0.0, 176.0, 128.0, 2.0),
            (0, 89, 89, 0.0, 178.0, 128.0, 2.0),
            (0, 90, 90, 0.0, 180.0, 128.0, 2.0),
            (0, 91, 91, 0.0, 182.0, 128.0, 2.0),
            (0, 92, 92, 0.0, 184.0, 128.0, 2.0),
            (0, 93, 93, 0.0, 186.0, 128.0, 2.0),
            (0, 94, 94, 0.0, 188.0, 128.0, 2.0),
            (0, 95, 95, 0.0, 190.0, 128.0, 2.0),
            (0, 96, 96, 0.0, 192.0, 128.0, 2.0),
            (0, 97, 97, 0.0, 194.0, 128.0, 2.0),
            (0, 98, 98, 0.0, 196.0, 128.0, 2.0),
            (0, 99, 99, 0.0, 198.0, 128.0, 2.0),
            (0, 100, 100, 0.0, 200.0, 128.0, 2.0),
            (0, 101, 101, 0.0, 202.0, 128.0, 2.0),
            (0, 102, 102, 0.0, 204.0, 128.0, 2.0),
            (0, 103, 103, 0.0, 206.0, 128.0, 2.0),
            (0, 104, 104, 0.0, 208.0, 128.0, 2.0),
            (0, 105, 105, 0.0, 210.0, 128.0, 2.0),
            (0, 106, 106, 0.0, 212.0, 128.0, 2.0),
            (0, 107, 107, 0.0, 214.0, 128.0, 2.0),
            (0, 108, 108, 0.0, 216.0, 128.0, 2.0),
            (0, 109, 109, 0.0, 218.0, 128.0, 2.0),
            (0, 110, 110, 0.0, 220.0, 128.0, 2.0),
            (0, 111, 111, 0.0, 222.0, 128.0, 2.0),
            (0, 112, 112, 0.0, 224.0, 128.0, 2.0),
            (0, 113, 113, 0.0, 226.0, 128.0, 2.0),
            (0, 114, 114, 0.0, 228.0, 128.0, 2.0),
            (0, 115, 115, 0.0, 230.0, 128.0, 2.0),
            (0, 116, 116, 0.0, 232.0, 128.0, 2.0),
            (0, 117, 117, 0.0, 234.0, 128.0, 2.0),
            (0, 118, 118, 0.0, 236.0, 128.0, 2.0),
            (0, 119, 119, 0.0, 238.0, 128.0, 2.0),
            (0, 120, 120, 0.0, 240.0, 128.0, 2.0),
            (0, 121, 121, 0.0, 242.0, 128.0, 2.0),
            (0, 122, 122, 0.0, 244.0, 128.0, 2.0),
            (0, 123, 123, 0.0, 246.0, 128.0, 2.0),
            (0, 124, 124, 0.0, 248.0, 128.0, 2.0),
            (0, 125, 125, 0.0, 250.0, 128.0, 2.0),
            (0, 126, 126, 0.0, 252.0, 128.0, 2.0),
            (0, 127, 127, 0.0, 254.0, 128.0, 2.0),
        ],
    },
    GridCase {
        name: "NaN spacing -> 0",
        rect: GridRect { x: 0.0, y: 0.0, w: 64.0, h: 64.0 },
        cols: 2, rows: 2, spacing: f64::NAN,
        min_w: 32.0, min_h: 32.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 32.0, 32.0),
            (1, 0, 1, 32.0, 0.0, 32.0, 32.0),
            (0, 1, 2, 0.0, 32.0, 32.0, 32.0),
            (1, 1, 3, 32.0, 32.0, 32.0, 32.0),
        ],
    },
    GridCase {
        name: "negative spacing -> clamped to 0",
        rect: GridRect { x: 0.0, y: 0.0, w: 64.0, h: 64.0 },
        cols: 2, rows: 2, spacing: -10.0,
        min_w: 32.0, min_h: 32.0,
        cells: &[
            (0, 0, 0, 0.0, 0.0, 32.0, 32.0),
            (1, 0, 1, 32.0, 0.0, 32.0, 32.0),
            (0, 1, 2, 0.0, 32.0, 32.0, 32.0),
            (1, 1, 3, 32.0, 32.0, 32.0, 32.0),
        ],
    },
];

#[test]
fn compute_cells_matches_the_reference_on_every_fixture() {
    let mut total_cells = 0usize;
    for c in GRID_CASES {
        let grid = SliceGrid::new(c.rect, c.cols, c.rows, c.spacing);
        let got = compute_cells(&grid);
        assert_eq!(got.cells.len(), c.cells.len(), "{}: cell count", c.name);
        assert_eq!(got.min_w, c.min_w, "{}: min cell width (the reference's `cw`)", c.name);
        assert_eq!(got.min_h, c.min_h, "{}: min cell height (the reference's `ch`)", c.name);
        for (got_cell, want) in got.cells.iter().zip(c.cells) {
            let &(col, row, index, x, y, w, h) = want;
            assert_eq!((got_cell.col, got_cell.row, got_cell.index), (col, row, index), "{}: cell identity", c.name);
            assert_eq!(got_cell.x, x, "{} cell {}: x", c.name, index);
            assert_eq!(got_cell.y, y, "{} cell {}: y", c.name, index);
            assert_eq!(got_cell.w, w, "{} cell {}: w", c.name, index);
            assert_eq!(got_cell.h, h, "{} cell {}: h", c.name, index);
        }
        total_cells += got.cells.len();
    }
    // Guard against a silently-empty fixture table (`CLAUDE.md`'s
    // "watch for silently-empty golden output" rule): four subsystems in this
    // port have been bitten by a harness that produced nothing and a test
    // that passed anyway.
    assert_eq!(GRID_CASES.len(), 10);
    assert_eq!(total_cells, 220);
}

/// The half-gutter model, stated as its own assertion because it is the one
/// thing about `computeCells` a reimplementation gets wrong: spacing is *not*
/// a pitch. With `cols=6, spacing=8` over a 3072px span, the classic
/// equal-cell formula gives every cell `(3072 - 5*8)/6 = 505.33`; the
/// reference gives 508 for the two outer columns and 504 for the four
/// interior ones, because each interior *edge* eats `spacing/2` from the cell
/// on either side of it and the outer edges eat nothing.
#[test]
fn spacing_is_a_half_gutter_on_interior_edges_not_a_pitch() {
    let grid = SliceGrid::new(GridRect::whole(3072, 2048), 6, 4, 8.0);
    let got = compute_cells(&grid);
    let widths: Vec<f64> = got.cells[0..6].iter().map(|c| c.w).collect();
    assert_eq!(widths, vec![508.0, 504.0, 504.0, 504.0, 504.0, 508.0]);
    assert_eq!(got.min_w, 504.0);
    // The equal-cell formula a reimplementation would reach for first.
    let equal_cell = (3072.0 - 5.0 * 8.0) / 6.0;
    assert!(widths.iter().all(|&w| w != equal_cell));
}

// ============================================================================
// cropCell's source-rect rounding
// ============================================================================

struct CropCase {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    sx: u32,
    sy: u32,
    sw: u32,
    sh: u32,
}

/// Captured verbatim from the arguments the reference's own `cropCell` passes
/// to `ctx.drawImage`. `Math.round` is half-**up**, not half-away-from-zero,
/// which is why `12.5 -> 13` and `-0.4 -> -0 -> 0` are both here, alongside a
/// sub-pixel extent (the `Math.max(1,...)` floor) and a negative one.
const CROP_CASES: &[CropCase] = &[
    CropCase { x: 0_f64, y: 0_f64, w: 768_f64, h: 512_f64, sx: 0, sy: 0, sw: 768, sh: 512 },
    CropCase { x: 0.5_f64, y: 1.5_f64, w: 10.5_f64, h: 20.5_f64, sx: 1, sy: 2, sw: 11, sh: 21 },
    CropCase { x: -3.2_f64, y: -0.4_f64, w: 5.6_f64, h: 7.4_f64, sx: 0, sy: 0, sw: 6, sh: 7 },
    CropCase { x: 12.49999_f64, y: 12.5_f64, w: 0.4_f64, h: 0.5_f64, sx: 12, sy: 13, sw: 1, sh: 1 },
    CropCase { x: 7_f64, y: 11_f64, w: -4_f64, h: -4_f64, sx: 7, sy: 11, sw: 1, sh: 1 },
    CropCase { x: 33.333333_f64, y: 66.666666_f64, w: 33.333333_f64, h: 33.333333_f64, sx: 33, sy: 67, sw: 33, sh: 33 },
];

#[test]
fn cell_source_rect_matches_the_references_crop_cell_rounding() {
    for c in CROP_CASES {
        let cell = cartalith_assets::CellRect { col: 0, row: 0, index: 0, x: c.x, y: c.y, w: c.w, h: c.h };
        assert_eq!(
            cell_source_rect(&cell),
            (c.sx, c.sy, c.sw, c.sh),
            "cell x={} y={} w={} h={}",
            c.x, c.y, c.w, c.h
        );
    }
    assert_eq!(CROP_CASES.len(), 6);
}

// ============================================================================
// isBlank
// ============================================================================

struct BlankCase {
    name: &'static str,
    data: &'static [u8],
    blank: bool,
}

/// Captured verbatim from the reference's own `isBlank(ctx,w,h)`. The
/// threshold is `>8`, so alpha 8 is still blank — a quantised fixture pair
/// sits on either side of it, since a continuous one would hide the constant.
const BLANK_CASES: &[BlankCase] = &[
    BlankCase { name: "all alpha 0", data: &[0,0,0,0,1,2,3,0], blank: true },
    BlankCase { name: "alpha exactly 8 (the boundary -- still blank)", data: &[9,9,9,8,9,9,9,8], blank: true },
    BlankCase { name: "alpha 9 (one over -- not blank)", data: &[0,0,0,8,0,0,0,9], blank: false },
    BlankCase { name: "alpha 255", data: &[0,0,0,255], blank: false },
    BlankCase { name: "empty buffer", data: &[], blank: true },
    BlankCase { name: "alpha 1 only", data: &[0,0,0,1], blank: true },
];

#[test]
fn is_blank_matches_the_reference_on_every_fixture() {
    for c in BLANK_CASES {
        let px = c.data.len() / 4;
        let img = DecodedImage::new(1, px as u32, c.data.to_vec()).expect("fixture is a whole number of pixels");
        assert_eq!(is_blank(&img), c.blank, "{}", c.name);
    }
    assert_eq!(BLANK_CASES.len(), 6);
}

// ============================================================================
// applyChroma
// ============================================================================

struct ChromaCase {
    name: &'static str,
    color: [u8; 3],
    tol: f64,
    before: &'static [u8],
    after: &'static [u8],
}

/// Captured verbatim from the reference's own `applyChroma(ctx,w,h)`. The
/// comparison is `<=`, so a pixel at exactly the tolerance distance *is*
/// keyed out; the second fixture is quantised to straddle that boundary by
/// one unit, which `<` would fail and a continuous fixture would not catch.
const CHROMA_CASES: &[ChromaCase] = &[
    ChromaCase { name: "exact white match, tol 40", color: [255,255,255], tol: 40.0, before: &[255,255,255,255,0,0,0,255], after: &[255,255,255,0,0,0,0,255] },
    ChromaCase { name: "distance exactly == tol (<= keeps it keyed)", color: [100,100,100], tol: 10.0, before: &[110,100,100,255,111,100,100,255], after: &[110,100,100,0,111,100,100,255] },
    ChromaCase { name: "already-transparent pixel is skipped", color: [0,0,0], tol: 0.0, before: &[0,0,0,0,0,0,0,200], after: &[0,0,0,0,0,0,0,0] },
    ChromaCase { name: "tol 0 keys only an exact match", color: [12,34,56], tol: 0.0, before: &[12,34,56,255,12,34,57,255], after: &[12,34,56,0,12,34,57,255] },
    ChromaCase { name: "3-channel diagonal just inside tol 40 (23,23,23 -> d2=1587)", color: [0,0,0], tol: 40.0, before: &[23,23,23,255,24,24,24,255], after: &[23,23,23,0,24,24,24,255] },
];

#[test]
fn apply_chroma_matches_the_reference_on_every_fixture() {
    for c in CHROMA_CASES {
        let px = c.before.len() / 4;
        let mut img = DecodedImage::new(1, px as u32, c.before.to_vec()).expect("fixture is a whole number of pixels");
        apply_chroma(&mut img, &ChromaKey { color: c.color, tol: c.tol });
        assert_eq!(img.rgba, c.after, "{}", c.name);
    }
    assert_eq!(CHROMA_CASES.len(), 5);
}
