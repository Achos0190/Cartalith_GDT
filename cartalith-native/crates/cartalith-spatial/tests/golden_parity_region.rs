//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region
//! *selection* geometry: `normRegion` (reference line 11569) and `tileDims`
//! (11536).
//!
//! Both are functions the reference itself flagged as pure and headless-
//! testable, so there is nothing to stub and nothing to transcribe: the
//! harness calls them directly under Node `vm.runInContext` over whole
//! `<script>` blocks (#1 2084-14556, #2 14563-26720), delimiters asserted
//! against the real `<script>`/`</script>` tags — see
//! `cartalith-terrain/tests/golden_parity_amplify.rs` for the harness write-up
//! and the two balance-check false positives it produced.
//!
//! Fixture shape, chosen so the branches are actually reachable: reversed drag
//! corners, negative overshoot, far-edge overshoot, a below-minimum drag, an
//! explicitly *smaller* minimum, a selection wider than the grid, a zero-area
//! tap, fractional corners (which is the only case that distinguishes the
//! `floor` on the origin from the `ceil` on the extent), and a minimum larger
//! than the room left at the far corner. Before any golden was written down
//! the extraction asserted that all ten rectangles are non-empty and all seven
//! tile-dim results are at least 2px.

use cartalith_spatial::{norm_region, tile_dims, Region};

#[test]
fn norm_region_matches_the_reference_on_every_recorded_drag() {
    // (x0, y0, x1, y1, W, H, minW, minH) -> (x, y, w, h)
    #[allow(clippy::type_complexity)]
    let want: &[((f64, f64, f64, f64, usize, usize, Option<usize>, Option<usize>),
                 (usize, usize, usize, usize))] = &[
        ((3.0, 4.0, 20.0, 18.0, 64, 48, None, None), (3, 4, 17, 14)),
        ((20.0, 18.0, 3.0, 4.0, 64, 48, None, None), (3, 4, 17, 14)),
        ((-9.0, -4.0, 12.0, 9.0, 64, 48, None, None), (0, 0, 21, 13)),
        ((58.0, 44.0, 90.0, 70.0, 64, 48, None, None), (32, 22, 32, 26)),
        ((10.0, 10.0, 11.0, 11.0, 64, 48, None, None), (10, 10, 8, 8)),
        ((10.0, 10.0, 12.0, 13.0, 64, 48, Some(3), Some(3)), (10, 10, 3, 3)),
        ((0.0, 0.0, 200.0, 200.0, 64, 48, None, None), (0, 0, 64, 48)),
        ((5.0, 5.0, 5.0, 5.0, 64, 48, None, None), (5, 5, 8, 8)),
        ((1.7, 2.2, 9.4, 8.9, 64, 48, None, None), (1, 2, 8, 8)),
        ((63.0, 47.0, 64.0, 48.0, 64, 48, Some(16), Some(16)), (48, 32, 16, 16)),
        // A fractional drag big enough that the 8-cell minimum does not mask
        // the `ceil` on the extent: ceil(18.7) = 19, floor would give 18 and
        // the minimum would not rescue it. Added after mutation testing found
        // the original fractional case could not distinguish the two.
        ((1.7, 2.2, 20.4, 18.9, 64, 48, None, None), (1, 2, 19, 17)),
        // An explicit ZERO minimum. JS `minW = minW || 8` treats it as 8.
        ((10.0, 10.0, 30.0, 30.0, 64, 48, Some(0), Some(0)), (10, 10, 20, 20)),
        // ...and an explicit minimum of ONE, which is emphatically not 8 --
        // the pair is what pins the falsy-zero rule rather than an off-by-one.
        ((10.0, 10.0, 11.0, 11.0, 64, 48, Some(1), Some(1)), (10, 10, 1, 1)),
    ];
    for &((x0, y0, x1, y1, gw, gh, mw, mh), (x, y, w, h)) in want {
        let got = norm_region(x0, y0, x1, y1, gw, gh, mw, mh);
        assert_eq!(got, Region { x, y, w, h }, "normRegion({x0},{y0},{x1},{y1},{gw},{gh},{mw:?},{mh:?})");
        // shape, re-asserted from the harness: no drag yields an empty rect
        assert!(got.w > 0 && got.h > 0);
    }
}

#[test]
fn tile_dims_matches_the_reference_including_both_extremes() {
    // (w, h, cols, rows, ts) -> (tileW, tileH)
    type TileDimCase = ((usize, usize, usize, usize, usize), (usize, usize));
    let want: &[TileDimCase] = &[
        ((64, 48, 2, 2, 1024), (1024, 768)),
        ((64, 48, 1, 1, 512), (512, 384)),
        ((100, 20, 2, 1, 1024), (1024, 410)),   // very wide
        ((20, 100, 1, 2, 1024), (410, 1024)),   // very tall
        ((33, 33, 3, 3, 256), (256, 256)),      // exactly square -> aspect 1 branch
        ((1000, 3, 1, 1, 8), (8, 2)),           // the max(2, ..) floor, wide
        ((3, 1000, 1, 1, 8), (2, 8)),           // and tall
        // Aspect exactly 1 with a tile size *below* the 2px floor. Added after
        // mutation testing: at any sane tile size the `aspect >= 1` and
        // `aspect > 1` branches compute the same pair, so nothing could see
        // the comparator. Below the floor they diverge -- one floors the
        // height, the other the width -- and the reference floors the height.
        ((10, 10, 1, 1, 1), (1, 2)),
    ];
    for &((w, h, cols, rows, ts), (tw, th)) in want {
        let d = tile_dims(&Region { x: 0, y: 0, w, h }, cols, rows, ts);
        assert_eq!((d.w, d.h), (tw, th), "tileDims({w}x{h}, {cols}x{rows}, {ts})");
    }
}
