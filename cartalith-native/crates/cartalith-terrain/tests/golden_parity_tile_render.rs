//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E2's pixel half:
//! `hypso` (reference line 8332) and `renderHeightTileRGBA` (11610), with the
//! four edge extrapolators (11606-11609) underneath them.
//!
//! # The harness
//!
//! Node `vm.runInContext`, the same one milestone E built and not checked in.
//! Whole `<script>` blocks, delimiters asserted against the real
//! `<script>`/`</script>` tags (block #1 is lines 2084-14556); the
//! block-comment balance assertion ran and passed clean this time, 1203 open
//! comments, after milestone E's two fixes to the template-literal and
//! regex-literal skippers.
//!
//! **One harness bug found, and it is worth recording because it looked
//! exactly like a reference bug.** With the DOM stubbed, block #1's own boot
//! code schedules a deferred first `generate()`/render pass on a timer — and
//! the reference's `microtask()` is literally `setTimeout(r, 0)`, which
//! `exportRegionTiles` awaits between tiles. So the boot work fired *between
//! tile 3 and tile 4* and overwrote `field` mid-loop, which read as
//! `amplifyRegion` returning different answers for identical arguments across
//! an `exportRegionTiles` call. It is not: `amplifyRegion` called twice in a
//! row is bit-identical. The harness now makes `requestAnimationFrame` inert
//! and drains pending macrotasks before installing any fixture, after which
//! `field` is byte-stable straight through an export. Recorded here because
//! "the reference is non-deterministic" is a conclusion worth being slow to
//! reach.
//!
//! # The fixture
//!
//! A synthetic tile from pure arithmetic — no `sin`/`cos`/`exp` — so V8's libm
//! and Rust's cannot disagree about the *input*. It carries a quantised `% 11`
//! term (urban M3's lesson that a continuous fixture can structurally fail to
//! reach the paths under test) and spans sea level, so both shading bands run.
//! Both sides FNV-1a-64 the raw `f32` bytes and this file re-derives the same
//! tile, so a fixture that drifted would fail loudly rather than quietly.
//!
//! `sin`/`cos` **are** reached inside the function under test, on the sun
//! azimuth — and the byte-exact match below is the evidence that V8's and
//! Rust's agree at these arguments. Four azimuths are covered (0, 45, 200,
//! 315) rather than one, so the agreement is not a single lucky argument.
//!
//! # Emptiness and shape, asserted before any golden was written down
//!
//! Every extracted raster was checked non-constant (73 to 164 distinct byte
//! values), full-range (min 9-13, max 255) and correctly sized. Silently-empty
//! output that passes every structural check has now bitten three subsystems
//! in this port, so it gets an explicit assertion rather than an implicit one.

use cartalith_terrain::tile_render::{hypso, render_height_tile_rgba};

fn fnv_u8(a: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in a {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

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

/// The harness's own `mkTile`, reproduced arithmetic-for-arithmetic.
fn mk_tile(w: usize, h: usize, k: i64) -> Vec<f32> {
    let mut t = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let q = ((x as i64 * 7 + y as i64 * 13 + k) % 11) as f64 / 10.0;
            let denom = ((w - 1) * (h - 1) * 2) as f64;
            let v = 0.05
                + 0.9 * ((x * (h - 1) + y * (w - 1)) as f64 / if denom == 0.0 { 1.0 } else { denom })
                + 0.08 * (q - 0.5);
            // The harness's own `if(v<0)v=0; if(v>1)v=1;` -- identical for a
            // fixture that cannot be NaN.
            t[y * w + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    t
}

#[test]
fn hypso_matches_the_reference_across_four_sea_levels() {
    // (sea, v) -> [r, g, b], straight out of the reference. The list spans:
    // both sea sub-ramps, every LAND stop boundary, the `sea <= 0` guard, the
    // `1 - sea <= 0` guard, and a v BELOW the palette (which extrapolates to
    // negative channels -- see the port's own doc comment).
    #[allow(clippy::type_complexity)]
    let want: &[((f64, f64), [f64; 3])] = &[
        ((0.42, -0.1), [2.3809523809523796, 0.3809523809523796, 1.2380952380952408]),
        ((0.42, 0.0), [10.0, 28.0, 46.0]),
        ((0.42, 0.05), [13.80952380952381, 41.80952380952381, 68.38095238095238]),
        ((0.42, 0.2), [25.23809523809524, 83.23809523809524, 135.52380952380955]),
        ((0.42, 0.41), [67.9047619047619, 137.42857142857142, 193.33333333333334]),
        ((0.42, 0.42), [47.0, 122.0, 68.0]),
        ((0.42, 0.45), [65.39080459770116, 131.19540229885058, 65.12643678160919]),
        ((0.42, 0.5), [96.04214559386973, 146.52107279693487, 60.337164750957854]),
        ((0.42, 0.6), [169.65517241379308, 169.64137931034483, 68.42758620689655]),
        ((0.42, 0.75), [152.8137931034483, 115.64137931034485, 72.1103448275862]),
        ((0.42, 0.9), [163.3605015673981, 163.3605015673981, 163.79310344827584]),
        ((0.42, 1.0), [248.0, 248.0, 250.0]),
        ((0.0, -0.1), [70.0, 140.0, 196.0]),
        ((0.0, 0.0), [47.0, 122.0, 68.0]),
        ((0.0, 0.05), [64.77777777777777, 130.88888888888889, 65.22222222222223]),
        ((0.0, 0.2), [120.0, 156.4, 59.6]),
        ((0.0, 0.41), [193.35, 168.10000000000002, 73.7]),
        ((0.0, 0.42), [190.8, 164.8, 73.6]),
        ((0.0, 0.45), [183.15, 154.9, 73.3]),
        ((0.0, 0.5), [170.4, 138.39999999999998, 72.8]),
        ((0.0, 0.6), [149.0, 114.8, 78.80000000000001]),
        ((0.0, 0.75), [141.5, 135.8, 129.79999999999998]),
        ((0.0, 0.9), [198.9090909090909, 198.9090909090909, 200.0]),
        ((0.0, 1.0), [248.0, 248.0, 250.0]),
        ((1.0, -0.1), [6.799999999999997, 16.39999999999999, 27.19999999999999]),
        ((1.0, 0.0), [10.0, 28.0, 46.0]),
        ((1.0, 0.05), [11.600000000000001, 33.800000000000004, 55.400000000000006]),
        ((1.0, 0.2), [16.4, 51.199999999999996, 83.6]),
        ((1.0, 0.41), [23.119999999999997, 75.55999999999999, 123.07999999999998]),
        ((1.0, 0.42), [23.439999999999998, 76.72, 124.95999999999998]),
        ((1.0, 0.45), [24.4, 80.19999999999999, 130.6]),
        ((1.0, 0.5), [26.0, 86.0, 140.0]),
        ((1.0, 0.6), [34.8, 96.8, 151.2]),
        ((1.0, 0.75), [48.0, 113.0, 168.0]),
        ((1.0, 0.9), [61.2, 129.2, 184.8]),
        ((1.0, 1.0), [47.0, 122.0, 68.0]),
        ((0.3, -0.1), [-0.6666666666666714, -10.666666666666686, -16.666666666666686]),
        ((0.3, 0.0), [10.0, 28.0, 46.0]),
        ((0.3, 0.05), [15.333333333333332, 47.33333333333333, 77.33333333333333]),
        ((0.3, 0.2), [40.66666666666667, 104.0, 158.66666666666669]),
        ((0.3, 0.41), [102.87301587301587, 149.93650793650795, 59.269841269841265]),
        ((0.3, 0.42), [107.95238095238096, 152.47619047619048, 58.476190476190474]),
        ((0.3, 0.45), [126.42857142857144, 158.11428571428573, 60.74285714285715]),
        ((0.3, 0.5), [158.57142857142858, 166.68571428571428, 66.45714285714286]),
        ((0.3, 0.6), [188.6142857142857, 161.97142857142856, 73.51428571428572]),
        ((0.3, 0.75), [146.85714285714286, 120.80000000000001, 93.3714285714286]),
        ((0.3, 0.9), [177.87012987012994, 177.87012987012994, 178.57142857142867]),
        ((0.3, 1.0), [248.0, 248.0, 250.0]),
    ];
    let mut saw_negative = false;
    for &((sea, v), c) in want {
        let got = hypso(v, sea);
        // Bit-exact, not epsilon: every operation here is +, -, * and /.
        assert_eq!(
            got.map(f64::to_bits),
            c.map(f64::to_bits),
            "hypso({v}) at sea {sea}: got {got:?}, want {c:?}"
        );
        if c.iter().any(|&x| x < 0.0) {
            saw_negative = true;
        }
    }
    assert!(saw_negative, "the fixture must actually reach the unclamped extrapolation");
}

#[test]
fn render_height_tile_rgba_matches_the_reference_byte_for_byte() {
    // (w, h, k, sea, sunAz, exag, rasterFnv, first 12 bytes, last 12 bytes),
    // plus the FNV of the f32 tile the raster was rendered from, so a drifted
    // fixture fails as a fixture rather than as a port.
    #[allow(clippy::type_complexity)]
    let want: &[(usize, usize, i64, f64, f64, f64, &str, &str, [u8; 12], [u8; 12], usize)] = &[
        (7, 5, 3, 0.42, 315.0, 3.4, "1fe86088feaadbb9", "3a52ad29ad651696",
         [13, 37, 61, 255, 22, 72, 118, 255, 25, 84, 137, 255],
         [140, 120, 98, 255, 141, 134, 127, 255, 233, 233, 234, 255], 73),
        (16, 11, 5, 0.42, 315.0, 3.4, "184830e09f9342b0", "fd2d98c48452d99e",
         [13, 40, 65, 255, 13, 40, 66, 255, 20, 64, 104, 255],
         [142, 142, 142, 255, 152, 152, 152, 255, 228, 228, 229, 255], 155),
        // Same tile, different appearance: this pair is what pins sea level,
        // azimuth and exaggeration as three separate inputs rather than one.
        (16, 11, 5, 0.60, 45.0, 1.0, "184830e09f9342b0", "b24fdfd56c4c0a10",
         [12, 35, 57, 255, 11, 34, 56, 255, 16, 49, 80, 255],
         [117, 111, 105, 255, 113, 107, 100, 255, 172, 172, 173, 255], 164),
        // One column wide: edgeL and edgeR both take their extrapolating
        // branch on every pixel, and min(1, W-1) is what keeps them in range.
        (1, 6, 2, 0.42, 315.0, 3.4, "5100a0d072bfac15", "5f80c04659753e88",
         [11, 33, 53, 255, 12, 37, 60, 255, 13, 41, 67, 255],
         [14, 45, 73, 255, 15, 47, 76, 255, 9, 28, 45, 255], 18),
        // One row tall: the same for edgeU and edgeD.
        (6, 1, 2, 0.42, 200.0, 8.0, "6437908bd1c86cc1", "fda3b8af9a4eb0b1",
         [11, 32, 53, 255, 15, 47, 76, 255, 12, 36, 59, 255],
         [10, 30, 50, 255, 14, 45, 73, 255, 11, 34, 56, 255], 18),
        // 2x2: every pixel is simultaneously a first and a last edge.
        (2, 2, 1, 0.42, 0.0, 3.4, "85d743c087377282", "d232f2b29f298cac",
         [10, 30, 49, 255, 87, 121, 46, 255, 68, 111, 49, 255],
         [87, 121, 46, 255, 68, 111, 49, 255, 188, 188, 190, 255], 11),
    ];
    for &(w, h, k, sea, az, exag, tile_fnv, raster_fnv, head, tail, distinct) in want {
        let t = mk_tile(w, h, k);
        assert_eq!(fnv_f32(&t), tile_fnv, "the {w}x{h} k={k} FIXTURE drifted, not the port");
        let px = render_height_tile_rgba(&t, w, h, sea, az, exag);
        assert_eq!(px.len(), w * h * 4);
        assert_eq!(&px[..12.min(px.len())], &head[..12.min(px.len())]);
        assert_eq!(&px[px.len() - 12..], &tail[..]);
        assert_eq!(fnv_u8(&px), raster_fnv, "{w}x{h} sea={sea} az={az} exag={exag}");
        // Shape, re-asserted from the extraction: the raster is not constant.
        let d: std::collections::HashSet<u8> = px.iter().copied().collect();
        assert_eq!(d.len(), distinct, "distinct byte count");
        assert_eq!(px.iter().copied().max().unwrap(), 255);
    }
}

// ---------------------------------------------------------------------------
// Second pass: the fixture the FIRST mutation sweep proved was missing.
//
// `let s = if v < sea {...} else {...}` survived being mutated to `v <= sea`,
// because no pixel in the six rasters above sits EXACTLY at sea level — and
// `hypso`'s own `v < sea` is a separate test, so the mutation only shows up in
// the shading multiplier (`0.75 + 0.25·sh` versus `0.4 + 0.6·sh`), which is a
// visible, several-tens-of-a-byte difference. Two more rasters, both with a
// pixel value bit-identical to `state.seaLevel`, close it.
// ---------------------------------------------------------------------------

#[test]
fn a_pixel_exactly_at_sea_level_takes_the_land_branch() {
    // Flat, entirely AT sea level. A single colour, and the reference's is the
    // LAND multiplier applied to LAND[0] -- `v < sea` is false at equality.
    let flat = vec![0.5f32; 9];
    let px = render_height_tile_rgba(&flat, 3, 3, 0.5, 315.0, 3.4);
    assert_eq!(px.len(), 36);
    assert_eq!(&px[..8], &[37, 96, 53, 255, 37, 96, 53, 255]);
    assert_eq!(&px[px.len() - 8..], &[37, 96, 53, 255, 37, 96, 53, 255]);
    assert_eq!(fnv_u8(&px), "94af154c13707170");
    let d: std::collections::HashSet<u8> = px.iter().copied().collect();
    assert_eq!(d.len(), 4, "one colour plus alpha");

    // ...and a raster that STEPS through sea level, so the two branches run
    // side by side on real gradients rather than on a constant.
    let mut step = vec![0.0f32; 16];
    for (i, v) in step.iter_mut().enumerate() {
        *v = match i % 3 {
            0 => 0.5f32,
            1 => 0.35f32,
            _ => 0.7f32,
        };
    }
    let px = render_height_tile_rgba(&step, 4, 4, 0.5, 45.0, 2.0);
    assert_eq!(px.len(), 64);
    assert_eq!(&px[..8], &[33, 85, 47, 255, 40, 99, 149, 255]);
    assert_eq!(&px[px.len() - 8..], &[162, 141, 61, 255, 31, 80, 45, 255]);
    assert_eq!(fnv_u8(&px), "b1185bbf55bbdb35");
    let d: std::collections::HashSet<u8> = px.iter().copied().collect();
    assert_eq!(d.len(), 37);
}
