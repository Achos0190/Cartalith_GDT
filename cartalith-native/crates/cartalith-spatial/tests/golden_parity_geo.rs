//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E2's raster→vector
//! half: `_geoXY` (reference line 12491), `_geoTraceMaskRings` (12500),
//! `_geoRingArea` (12526), `_geoPointInRing` (12527) and
//! `_geoMaskOutlineCoords` (12540).
//!
//! # The harness
//!
//! Node `vm.runInContext` over whole `<script>` blocks, delimiters asserted
//! against the real `<script>`/`</script>` tags (block #1, lines 2084-14556;
//! block #2, 14563-26720). The block-comment balance assertion ran on both and
//! passed clean — 1203 and 187 open comments — with milestone E's two fixes to
//! the template-literal and regex-literal skippers still in place.
//!
//! All five functions are pure and headless: the masks below were handed to
//! the reference as plain JS closures on a 12x9 grid with `mapWidthKm = 600`
//! (so `cellKm` is exactly `50`), and the rings came straight back.
//!
//! # The fixtures, and what each one is *for*
//!
//! Six masks, chosen so every branch has something to decide:
//!
//! - **a** — a 6x5 block with a 2x2 hole, plus a disjoint 2x2 blob. Shell,
//!   hole and second shell in one trace; the case that pins hole nesting.
//! - **b** — the checkerboard pinch (two diagonal cells, the other two not).
//!   The reference explicitly *"doesn't disambiguate"* it, and what that looks
//!   like from the outside is an **unclosed ring**, because the second cell's
//!   up-edge overwrites the first cell's down-edge in the JS `Map` and the walk
//!   runs into an already-visited key. Asserted, not tolerated.
//! - **c** — a single cell. The smallest ring that survives the `>= 4` filter.
//! - **d** — empty. The `None` path.
//! - **e** — the whole grid. The outline runs along the grid border, which is
//!   only reachable because the mask answers `false` outside it.
//! - **f** — a block with a hole that has an *island* in it. The island traces
//!   positive, so it becomes its own polygon rather than a nested ring: the
//!   staircase-level simplification, pinned so nobody "fixes" it silently.
//!
//! # Emptiness and shape, asserted before any golden was written down
//!
//! Every non-empty mask produced at least one ring, every ring at least four
//! points, and the areas came back non-zero and correctly signed. The
//! coordinate list is compared point for point rather than by count.

use cartalith_spatial::geo::{
    geo_xy, js_to_fixed, mask_outline_coords, point_in_ring, ring_area, trace_mask_rings,
};

const GW: usize = 12;
const GH: usize = 9;
const CELL_KM: f64 = 50.0; // 600 km / 12 cells, as the harness set it

fn mask_a(x: i32, y: i32) -> bool {
    ((1..=6).contains(&x) && (1..=5).contains(&y) && !((3..=4).contains(&x) && (2..=3).contains(&y)))
        || ((9..=10).contains(&x) && (6..=7).contains(&y))
}
fn mask_b(x: i32, y: i32) -> bool {
    (x == 2 && y == 2) || (x == 3 && y == 3)
}
fn mask_c(x: i32, y: i32) -> bool {
    x == 5 && y == 4
}
fn mask_e(x: i32, y: i32) -> bool {
    (0..12).contains(&x) && (0..9).contains(&y)
}
fn mask_f(x: i32, y: i32) -> bool {
    ((1..=7).contains(&x) && (1..=7).contains(&y) && !((3..=5).contains(&x) && (3..=5).contains(&y)))
        || (x == 4 && y == 4)
}

#[test]
fn geo_xy_matches_the_reference_including_the_north_up_flip() {
    let want: &[((f64, f64), [f64; 2])] = &[
        ((0.0, 0.0), [0.0, 450.0]),
        ((1.0, 1.0), [50.0, 400.0]),
        ((3.0, 7.0), [150.0, 100.0]),
        ((12.0, 9.0), [600.0, 0.0]),
        ((5.5, 2.25), [275.0, 337.5]),
        ((11.0, 0.0), [550.0, 450.0]),
        ((0.0, 9.0), [0.0, 0.0]),
        ((7.0, 3.0), [350.0, 300.0]),
    ];
    for &((gx, gy), p) in want {
        assert_eq!(geo_xy(gx, gy, GH, CELL_KM).map(f64::to_bits), p.map(f64::to_bits), "_geoXY({gx},{gy})");
    }
    // ...and the rounding really is toFixed, not Rust's tie-to-even. 800 km
    // over a 12800-cell grid gives cellKm == 0.0625, an exact tie at three
    // decimals, which is where the two rules visibly disagree.
    assert_eq!(js_to_fixed(1.0 * 0.0625, 3), 0.063);
}

#[test]
fn trace_mask_rings_matches_the_reference_ring_for_ring() {
    #[allow(clippy::type_complexity)]
    let want: &[(&str, fn(i32, i32) -> bool, &[(f64, &[(i32, i32)])])] = &[
        ("a", mask_a, &[
            (30.0, &[(1,1),(2,1),(3,1),(4,1),(5,1),(6,1),(7,1),(7,2),(7,3),(7,4),(7,5),(7,6),
                     (6,6),(5,6),(4,6),(3,6),(2,6),(1,6),(1,5),(1,4),(1,3),(1,2),(1,1)]),
            (-4.0, &[(4,2),(3,2),(3,3),(3,4),(4,4),(5,4),(5,3),(5,2),(4,2)]),
            (4.0, &[(9,6),(10,6),(11,6),(11,7),(11,8),(10,8),(9,8),(9,7),(9,6)]),
        ]),
        // The pinch: ONE ring, six points, and deliberately not closed.
        ("b", mask_b, &[(3.0, &[(2,2),(3,2),(3,3),(4,3),(4,4),(3,4)])]),
        ("c", mask_c, &[(1.0, &[(5,4),(6,4),(6,5),(5,5),(5,4)])]),
        ("e", mask_e, &[
            (108.0, &[(0,0),(1,0),(2,0),(3,0),(4,0),(5,0),(6,0),(7,0),(8,0),(9,0),(10,0),(11,0),(12,0),
                      (12,1),(12,2),(12,3),(12,4),(12,5),(12,6),(12,7),(12,8),(12,9),
                      (11,9),(10,9),(9,9),(8,9),(7,9),(6,9),(5,9),(4,9),(3,9),(2,9),(1,9),(0,9),
                      (0,8),(0,7),(0,6),(0,5),(0,4),(0,3),(0,2),(0,1),(0,0)]),
        ]),
        ("f", mask_f, &[
            (49.0, &[(1,1),(2,1),(3,1),(4,1),(5,1),(6,1),(7,1),(8,1),(8,2),(8,3),(8,4),(8,5),(8,6),(8,7),(8,8),
                     (7,8),(6,8),(5,8),(4,8),(3,8),(2,8),(1,8),(1,7),(1,6),(1,5),(1,4),(1,3),(1,2),(1,1)]),
            (-9.0, &[(4,3),(3,3),(3,4),(3,5),(3,6),(4,6),(5,6),(6,6),(6,5),(6,4),(6,3),(5,3),(4,3)]),
            (1.0, &[(4,4),(5,4),(5,5),(4,5),(4,4)]),
        ]),
    ];
    for &(label, m, rings) in want {
        let got = trace_mask_rings(&m, 0, 0, GW as i32, GH as i32);
        assert_eq!(got.len(), rings.len(), "mask {label}: ring count");
        for (i, &(area, pts)) in rings.iter().enumerate() {
            assert_eq!(got[i].as_slice(), pts, "mask {label} ring {i}: points");
            assert_eq!(ring_area(&got[i]), area, "mask {label} ring {i}: area");
            // Shape, re-asserted from the extraction.
            assert!(got[i].len() >= 4, "mask {label} ring {i} survived the >= 4 filter");
            assert_ne!(area, 0.0);
        }
    }
    // Empty in, empty out -- and nothing in between.
    assert!(trace_mask_rings(&|_, _| false, 0, 0, GW as i32, GH as i32).is_empty());
}

#[test]
fn the_pinch_ring_is_the_one_that_does_not_close() {
    // Split out from the table above because it is the single most surprising
    // recorded behaviour here and deserves to fail by name.
    let rings = trace_mask_rings(&mask_b, 0, 0, GW as i32, GH as i32);
    assert_eq!(rings.len(), 1);
    assert_ne!(rings[0].first(), rings[0].last());
    // Every OTHER traced ring in the fixture set does close.
    for m in [mask_a as fn(i32, i32) -> bool, mask_c, mask_e, mask_f] {
        for r in trace_mask_rings(&m, 0, 0, GW as i32, GH as i32) {
            assert_eq!(r.first(), r.last(), "a well-formed boundary ring must close");
        }
    }
}

#[test]
fn point_in_ring_matches_the_reference_on_a_traced_shell() {
    let rings = trace_mask_rings(&mask_a, 0, 0, GW as i32, GH as i32);
    let shell = rings.iter().find(|r| ring_area(r) > 0.0).expect("a shell");
    let want: &[((f64, f64), bool)] = &[
        ((0.0, 0.0), false),
        ((3.0, 3.0), true),
        ((2.0, 2.0), true),
        ((1.0, 1.0), true),
        ((7.0, 6.0), false),
        ((6.5, 3.5), true),
        ((3.5, 2.5), true),
    ];
    for &((px, py), inside) in want {
        assert_eq!(point_in_ring(px, py, shell), inside, "_geoPointInRing({px},{py})");
    }
}

#[test]
fn mask_outline_coords_matches_the_reference_coordinate_for_coordinate() {
    // Shorthand: a km pair.
    fn p(x: f64, y: f64) -> [f64; 2] {
        [x, y]
    }
    let a_shell = vec![p(50.,400.),p(100.,400.),p(150.,400.),p(200.,400.),p(250.,400.),p(300.,400.),
        p(350.,400.),p(350.,350.),p(350.,300.),p(350.,250.),p(350.,200.),p(350.,150.),p(300.,150.),
        p(250.,150.),p(200.,150.),p(150.,150.),p(100.,150.),p(50.,150.),p(50.,200.),p(50.,250.),
        p(50.,300.),p(50.,350.),p(50.,400.)];
    let a_hole = vec![p(200.,350.),p(150.,350.),p(150.,300.),p(150.,250.),p(200.,250.),p(250.,250.),
        p(250.,300.),p(250.,350.),p(200.,350.)];
    let a_blob = vec![p(450.,150.),p(500.,150.),p(550.,150.),p(550.,100.),p(550.,50.),p(500.,50.),
        p(450.,50.),p(450.,100.),p(450.,150.)];
    let got = mask_outline_coords(&mask_a, GW, GH, CELL_KM).expect("mask a is not empty");
    assert_eq!(got, vec![vec![a_shell, a_hole], vec![a_blob]]);

    // The pinch, carried all the way through to coordinates: still one polygon,
    // still six points, still unclosed.
    let got_b = mask_outline_coords(&mask_b, GW, GH, CELL_KM).expect("mask b is not empty");
    assert_eq!(got_b, vec![vec![vec![p(100.,350.),p(150.,350.),p(150.,300.),p(200.,300.),
                                     p(200.,250.),p(150.,250.)]]]);

    let got_c = mask_outline_coords(&mask_c, GW, GH, CELL_KM).expect("mask c is not empty");
    assert_eq!(got_c, vec![vec![vec![p(250.,250.),p(300.,250.),p(300.,200.),p(250.,200.),p(250.,250.)]]]);

    assert!(mask_outline_coords(&|_, _| false, GW, GH, CELL_KM).is_none(), "empty mask -> None");

    // The island: two polygons, and the island is a SHELL, not a nested ring.
    let got_f = mask_outline_coords(&mask_f, GW, GH, CELL_KM).expect("mask f is not empty");
    assert_eq!(got_f.len(), 2);
    assert_eq!(got_f[0].len(), 2);
    assert_eq!(got_f[1], vec![vec![p(200.,250.),p(250.,250.),p(250.,200.),p(200.,200.),p(200.,250.)]]);
}

// ---------------------------------------------------------------------------
// Second pass: the fixtures the FIRST mutation sweep proved were missing.
//
// Four mutations survived the sweep above, and none of them was equivalent —
// every one was a fixture that could not see the constant it was meant to pin:
//
//   * `_geoXY`'s three decimals, because every coordinate in the 12x9 fixture
//     is a whole number of kilometres or a clean `.5`;
//   * the tracer's `ring.length >= 4` filter in BOTH directions, because the
//     fixture masks only ever produce rings of length 5, 6, 9, 13, 23, 29 and
//     43 — nothing at the boundary;
//   * the shell/hole split's `area > 0`, because no fixture ring has area
//     exactly zero.
//
// Whether the boundary cases were even *reachable* was settled by brute force
// rather than argued: all 65 536 masks on a 4x4 grid were run through the
// reference's own `_geoTraceMaskRings`, and the answer is that length-4 rings
// occur for 1 695 of them, length-3 rings (which the filter drops) for 8 760,
// and zero-area rings do occur. So all three are real behaviour, and the masks
// below are three of the reference's own examples.
// ---------------------------------------------------------------------------

/// Build a mask from an explicit cell list, on a 6x6 grid at 1 km per cell.
fn cells_mask(cells: &'static [(i32, i32)]) -> impl Fn(i32, i32) -> bool {
    move |x, y| cells.contains(&(x, y))
}

#[test]
fn geo_xy_matches_the_reference_when_the_cell_size_is_not_round() {
    // GW=2048, GH=1311, mapWidthKm=800 -> cellKm = 0.390625, so every
    // coordinate really uses all three decimals `toFixed(3)` keeps.
    const K: f64 = 0.390625;
    let want: &[((f64, f64), [f64; 2])] = &[
        ((0.0, 0.0), [0.0, 512.109]),
        ((1.0, 1.0), [0.391, 511.719]),
        ((3.0, 7.0), [1.172, 509.375]),
        ((17.0, 29.0), [6.641, 500.781]),
        ((2048.0, 1311.0), [800.0, 0.0]),
        ((1023.0, 655.0), [399.609, 256.25]),
        ((7.0, 1310.0), [2.734, 0.391]),
        ((1.0, 0.0), [0.391, 512.109]),
    ];
    for &((gx, gy), p) in want {
        assert_eq!(geo_xy(gx, gy, 1311, K).map(f64::to_bits), p.map(f64::to_bits), "_geoXY({gx},{gy})");
    }
}

#[test]
fn a_length_four_ring_is_kept_and_a_length_three_ring_is_dropped() {
    // Mask 151 of the 4x4 sweep: cells (0,0) (1,0) (2,0) (0,1) (3,1). The
    // reference keeps TWO rings here, the second of which is exactly four
    // points long and unclosed -- the `>= 4` boundary, from below.
    let m151 = cells_mask(&[(0, 0), (1, 0), (2, 0), (0, 1), (3, 1)]);
    let rings = trace_mask_rings(&m151, 0, 0, 6, 6);
    assert_eq!(rings.len(), 2);
    assert_eq!(rings[0].as_slice(), &[(0,0),(1,0),(2,0),(3,0),(3,1),(4,1),(4,2),(3,2)]);
    assert_eq!(ring_area(&rings[0]), 4.0);
    assert_eq!(rings[1].as_slice(), &[(2,1),(1,1),(1,2),(0,2)]);
    assert_eq!(ring_area(&rings[1]), 2.0);

    // Mask 37: cells (0,0) (2,0) (1,1). The walk produces a THREE-point ring
    // as well, and the reference drops it -- so exactly one ring survives.
    let m37 = cells_mask(&[(0, 0), (2, 0), (1, 1)]);
    let rings = trace_mask_rings(&m37, 0, 0, 6, 6);
    assert_eq!(rings.len(), 1, "the length-3 ring must be dropped, not kept");
    assert_eq!(rings[0].as_slice(), &[(0,0),(1,0),(1,1),(2,1),(2,2),(1,2)]);
    assert_eq!(ring_area(&rings[0]), 2.0);

    // And through to coordinates, at 1 km per cell on a 6-row grid.
    assert_eq!(
        mask_outline_coords(&m151, 6, 6, 1.0).expect("non-empty"),
        vec![
            vec![vec![[0.,6.],[1.,6.],[2.,6.],[3.,6.],[3.,5.],[4.,5.],[4.,4.],[3.,4.]]],
            vec![vec![[2.,5.],[1.,5.],[1.,4.],[0.,4.]]],
        ]
    );
    assert_eq!(
        mask_outline_coords(&m37, 6, 6, 1.0).expect("non-empty"),
        vec![vec![vec![[0.,6.],[1.,6.],[1.,5.],[2.,5.],[2.,4.],[1.,4.]]]]
    );
}

#[test]
fn a_ring_of_exactly_zero_area_is_treated_as_a_hole() {
    // Mask 1943: cells (0,0) (1,0) (2,0) (0,1) (3,1) (0,2) (1,2) (2,2). The
    // pinch inside it traces a four-point ring whose shoelace area is exactly
    // 0 -- and the reference's `a > 0 ? shells : holes` therefore files it as a
    // HOLE, which nests it inside the enclosing shell. `a >= 0` would make it a
    // second polygon instead.
    let m = cells_mask(&[(0,0),(1,0),(2,0),(0,1),(3,1),(0,2),(1,2),(2,2)]);
    let rings = trace_mask_rings(&m, 0, 0, 6, 6);
    assert_eq!(rings.len(), 2);
    assert_eq!(ring_area(&rings[0]), 10.0);
    assert_eq!(rings[1].as_slice(), &[(2,1),(1,1),(1,2),(2,2)]);
    assert_eq!(ring_area(&rings[1]), 0.0, "exactly zero, not merely small");

    let out = mask_outline_coords(&m, 6, 6, 1.0).expect("non-empty");
    assert_eq!(out.len(), 1, "ONE polygon: the zero-area ring is a hole, not a shell");
    assert_eq!(out[0].len(), 2, "shell + the zero-area hole");
    assert_eq!(out[0][1], vec![[2.,5.],[1.,5.],[1.,4.],[2.,4.]]);
}
