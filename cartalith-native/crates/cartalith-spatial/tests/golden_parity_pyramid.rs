//! Golden-parity tests for the LOD pyramid's addressing — `pyramidDims`
//! (reference 10461), `pyramidTileBounds` (10594), `pyramidLevelForZoom`
//! (10600) and `tilesInView` (10637).
//!
//! # The harness
//!
//! Node `vm.runInContext`, this project's established practice, not checked in.
//! **Whole `<script>` blocks, not line slices** — block #1 (2084-14556), with
//! the harness asserting that the line before the slice *is* `<script>` and the
//! line after *is* `</script>`, so the boundary is the real delimiter rather
//! than an inferred top-level one.
//!
//! One harness detail is worth recording because it cost real time and is the
//! same hazard `CLAUDE.md` already names ("host-side assignment shadowing
//! `let`-declared reference globals — lexical bindings, not `vm` context
//! properties"): `GW`, `GH`, `state` and `VERSION` are `let`/`const` in the
//! block, so they are **not** context properties and `ctx.GW` reads
//! `undefined`. The probe is therefore *appended to the block's own source* and
//! run as one script, which is the only way it shares that lexical scope.
//!
//! The other is that block #1's boot (line 14554) reads
//! `typeof indexedDB==='undefined'` and, headless, **auto-generates a full
//! world** — minutes at the default `state.resW` of 2048. The harness supplies
//! a truthy `indexedDB` stub so boot takes the browser branch instead. None of
//! the functions under test read it.
//!
//! # Emptiness and shape assertions
//!
//! Per `CLAUDE.md`'s "watch for silently-empty golden output" rule, the
//! extraction asserted before any golden was written down: that every level's
//! tiles tile the inset field with zero gap and zero overlap, that an
//! off-grid view still names exactly one tile rather than an empty range, and
//! that `pyramidLevelForZoom` is monotonic in scale across fourteen octaves.
//! All three are re-asserted here against the port (the first and third in
//! `pyramid.rs`'s own unit tests, the second below).

use cartalith_spatial::pyramid::{
    pyramid_dims, pyramid_level_for_zoom, pyramid_tile_bounds, tiles_in_view,
};

/// The extraction's fixture dimensions: a 48x32 coarse field.
const CW: usize = 48;
const CH: usize = 32;

#[test]
fn pyramid_dims_matches_the_reference() {
    // [z, cols, rows], straight from the harness.
    for &(z, cols, rows) in
        &[(0, 1, 1), (1, 2, 2), (2, 4, 4), (3, 8, 8), (4, 16, 16), (-1, 1, 1)]
    {
        let d = pyramid_dims(z);
        assert_eq!((d.cols, d.rows), (cols, rows), "z={z}");
    }
}

#[test]
fn pyramid_tile_bounds_matches_the_reference() {
    // [z, col, row, x, y, w, h]. The fractional steps are the point: 47/2,
    // 47/4, 47/8 and their vertical twins are exactly what a rounding port
    // would get wrong.
    let cases: &[(i32, u32, u32, f64, f64, f64, f64)] = &[
        (0, 0, 0, 0.0, 0.0, 47.0, 31.0),
        (1, 0, 0, 0.0, 0.0, 23.5, 15.5),
        (1, 1, 1, 23.5, 15.5, 23.5, 15.5),
        (2, 3, 2, 35.25, 15.5, 11.75, 7.75),
        (3, 5, 7, 29.375, 27.125, 5.875, 3.875),
    ];
    for &(z, col, row, x, y, w, h) in cases {
        let b = pyramid_tile_bounds(CW, CH, z, col, row);
        assert_eq!((b.x, b.y, b.w, b.h), (x, y, w, h), "z={z} col={col} row={row}");
    }
}

#[test]
fn tiles_in_view_matches_the_reference() {
    // [z, vx0, vy0, vx1, vy1] -> [cols, rows, c0, c1, r0, r1, count]
    let cases: &[(i32, f64, f64, f64, f64, u32, u32, u32, u32, u32, u32, i64)] = &[
        (0, 0.0, 0.0, 46.0, 30.0, 1, 1, 0, 0, 0, 0, 1),
        (2, 0.0, 0.0, 46.0, 30.0, 4, 4, 0, 3, 0, 3, 16),
        (3, 10.0, 5.0, 20.0, 12.0, 8, 8, 1, 3, 1, 3, 9),
        (4, 46.0, 30.0, 47.0, 31.0, 16, 16, 15, 15, 15, 15, 1),
        // The last case is the one a "clamp the rect first" port gets wrong:
        // a view entirely off the north-west still names tile (0, 0).
        (2, -5.0, -5.0, 3.0, 3.0, 4, 4, 0, 0, 0, 0, 1),
    ];
    for &(z, x0, y0, x1, y1, cols, rows, c0, c1, r0, r1, count) in cases {
        let t = tiles_in_view(z, x0, y0, x1, y1, CW, CH);
        assert_eq!(
            (t.cols, t.rows, t.c0, t.c1, t.r0, t.r1, t.count),
            (cols, rows, c0, c1, r0, r1, count),
            "z={z} view=({x0},{y0})-({x1},{y1})"
        );
    }
}

#[test]
fn pyramid_level_for_zoom_matches_the_reference() {
    // [scale, baseW, tileSize, maxLevel] -> level.
    let cases: &[(f64, f64, f64, Option<i32>, i32)] = &[
        (1.0, 2048.0, 1024.0, Some(8), 1),
        (4.0, 2048.0, 1024.0, Some(8), 3),
        (64.0, 2048.0, 1024.0, Some(8), 7),
        // Past the cap: min(8, 11).
        (1024.0, 2048.0, 1024.0, Some(8), 8),
        // Below the floor: `Math.max(0.01, scale)` then `Math.max(1, want)`.
        (0.001, 2048.0, 1024.0, Some(8), 0),
        // Not a power of two -- log2(20) = 4.32, which rounds to 4.
        (8.0, 1280.0, 512.0, Some(6), 4),
        (1.0, 1024.0, 1024.0, None, 0),
    ];
    for &(scale, base_w, ts, max_level, want) in cases {
        assert_eq!(
            pyramid_level_for_zoom(scale, base_w, ts, max_level),
            want,
            "scale={scale} baseW={base_w} ts={ts} max={max_level:?}"
        );
    }
}
