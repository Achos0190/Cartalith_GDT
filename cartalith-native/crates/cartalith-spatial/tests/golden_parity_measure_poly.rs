//! Golden-parity tests for the Area tool's three polygon primitives:
//! `polyArea` (reference `Cartalith Gen1 v2.10.html` line 28290),
//! `polyCentroid` (28291) and `pointInPoly` (28295), ported into
//! `cartalith_spatial::measure` as `polygon_area` / `polygon_centroid` /
//! `point_in_polygon`.
//!
//! # Why these are ports and the rest of `measure.rs` is not
//!
//! `measure.rs`'s own module doc records that the ruler has *"zero reference
//! precedent"* and therefore no golden test. That stands, and it does **not**
//! extend to the Area tool `design/Cartalith Measurement Toolbar.dc.html`
//! state 3 adds: the shoelace area, the area-weighted centroid and the
//! crossing-number containment test are all real reference functions, and two
//! of the three already exist in this workspace in a different shape
//! (`cartalith-urban::geom::poly_area`/`poly_centroid` over `Vec2`;
//! `cartalith_spatial::geo::ring_area`/`point_in_ring` over *explicitly
//! closed* integer rings). So these get parity treatment rather than the
//! "new, disclosed as new" treatment the ruler got.
//!
//! # The harness
//!
//! Node `vm.runInContext` over lines **28290-28297** of the frozen reference
//! — the three function declarations and nothing else; none of them touches
//! `V`, `GW`, or any other module-scope binding, which is what makes an
//! eight-line slice legal here where other captures need a whole `<script>`
//! block. The extractor asserted the slice's first line matched
//! `function polyArea(p){` and its last matched `inside=!inside;}return
//! inside;}` **before** evaluating it, and asserted all three names were
//! defined **inside the context** afterwards.
//!
//! The first run of that extractor failed exactly the way `CLAUDE.md`'s
//! "watch for silently-empty golden output" rule predicts: the fixtures were
//! being computed on the *host* side, where `function` declarations evaluated
//! by `runInContext` are not visible. It threw `polyArea is not defined`
//! rather than producing empty output, but the fix is the same one — every
//! call moved inside the context.
//!
//! # The fixtures, and what each is for
//!
//! Six rings, chosen so each branch has something to decide:
//!
//! - **rect** — a plain 6x4 rectangle, counter-clockwise in this port's
//!   y-down grid. The baseline: area 24, centroid dead centre.
//! - **rect_cw** — the same rectangle wound the other way. `polyArea` is
//!   *signed*, so this is the fixture that would survive dropping the sign
//!   only if nothing checked it. It also pins the fact that winding does
//!   **not** change `pointInPoly`'s answer (crossing number is
//!   winding-blind) or `polyCentroid`'s.
//! - **u** — a concave "U". A bounding-box containment test passes every
//!   convex fixture and fails this one; two probes sit in the notch, one
//!   0.0001 cells inside each of its walls.
//! - **frac** — five vertices on no cell corner at all. A measuring ring is
//!   drawn wherever the click landed, so an all-integer fixture set would
//!   never exercise that.
//! - **line** — three collinear points: `polyCentroid`'s own
//!   `Math.abs(sa) < 1e-9` fallback to the plain vertex mean, which is
//!   otherwise unreachable.
//! - **tri** — an odd vertex count with a real area, so the `(i + 1) % n`
//!   wrap is not always an even-length one.
//!
//! Fourteen probe points are run against every ring — inside, outside, in a
//! concave notch, exactly on a vertex, exactly on an edge, and off-grid
//! negative — giving 84 containment goldens rather than six.
//!
//! # Tolerance
//!
//! **None.** Every assertion is `assert_eq!` on `f64`: these are exact
//! shoelace sums over exactly-representable inputs, and V8 and Rust agree
//! bit-for-bit on `+`, `-`, `*` and `/`. Nothing here calls a libm function,
//! so none of `CLAUDE.md`'s `Math.hypot`/`Math.exp` divergences applies.

use cartalith_spatial::{point_in_polygon, polygon_area, polygon_centroid};

/// The fourteen probe points, in the capture's own order.
const PROBES: [(f64, f64); 14] = [
    (5.0, 5.0),
    (0.0, 0.0),
    (9.0, 5.0),
    (5.0, 1.0),
    (5.0, 6.0),
    (8.5, 6.0),
    (2.0, 3.0),
    (8.0, 3.0),
    (3.0001, 5.0),
    (6.9999, 5.0),
    (1.25, 2.75),
    (4.5, 4.5),
    (-1.0, -1.0),
    (3.0, 9.0),
];

struct Golden {
    label: &'static str,
    points: &'static [(f64, f64)],
    area: f64,
    centroid: (f64, f64),
    inside: [bool; 14],
}

const RECT: [(f64, f64); 4] = [(2.0, 3.0), (8.0, 3.0), (8.0, 7.0), (2.0, 7.0)];
const RECT_CW: [(f64, f64); 4] = [(2.0, 7.0), (8.0, 7.0), (8.0, 3.0), (2.0, 3.0)];
const U: [(f64, f64); 8] = [
    (0.0, 0.0),
    (10.0, 0.0),
    (10.0, 10.0),
    (7.0, 10.0),
    (7.0, 3.0),
    (3.0, 3.0),
    (3.0, 10.0),
    (0.0, 10.0),
];
const FRAC: [(f64, f64); 5] = [
    (1.25, 2.75),
    (9.5, 3.125),
    (7.875, 8.25),
    (2.5, 6.5),
    (0.75, 4.25),
];
const LINE: [(f64, f64); 3] = [(0.0, 0.0), (2.0, 0.0), (4.0, 0.0)];
const TRI: [(f64, f64); 3] = [(0.0, 0.0), (7.0, 1.0), (3.0, 9.0)];

/// Captured from the frozen reference; see the module doc for the harness.
const GOLDENS: [Golden; 6] = [
    Golden {
        label: "rect",
        points: &RECT,
        area: 24.0,
        centroid: (5.0, 5.0),
        inside: [
            true, false, false, false, true, false, true, false, true, true, false, true, false,
            false,
        ],
    },
    Golden {
        label: "rect_cw",
        points: &RECT_CW,
        area: -24.0,
        centroid: (5.0, 5.0),
        inside: [
            true, false, false, false, true, false, true, false, true, true, false, true, false,
            false,
        ],
    },
    Golden {
        label: "u",
        points: &U,
        area: 72.0,
        centroid: (5.0, 4.416_666_666_666_667),
        inside: [
            false, true, true, true, false, true, true, true, false, false, true, false, false,
            false,
        ],
    },
    Golden {
        label: "frac",
        points: &FRAC,
        area: 32.304_687_5,
        centroid: (5.286_124_546_553_809, 5.009_119_306_731_157),
        inside: [
            true, false, false, false, true, true, true, false, true, true, false, true, false,
            false,
        ],
    },
    Golden {
        label: "line",
        points: &LINE,
        area: 0.0,
        centroid: (2.0, 0.0),
        inside: [false; 14],
    },
    Golden {
        label: "tri",
        points: &TRI,
        area: 30.0,
        centroid: (3.333_333_333_333_333_5, 3.333_333_333_333_333_5),
        inside: [
            false, false, false, true, false, false, true, false, true, false, true, true, false,
            false,
        ],
    },
];

/// The fixtures are asserted non-degenerate *here*, in the test binary, and
/// not only in the extractor — a fixture table that silently lost its
/// contents in transcription would otherwise pass every comparison below
/// against an empty loop.
#[test]
fn the_fixture_table_is_non_empty_and_varied() {
    assert_eq!(GOLDENS.len(), 6);
    assert!(GOLDENS.iter().all(|g| g.points.len() >= 3), "every ring has a real polygon");
    assert!(GOLDENS.iter().any(|g| g.area > 0.0), "some ring winds positive");
    assert!(GOLDENS.iter().any(|g| g.area < 0.0), "some ring winds negative");
    assert!(GOLDENS.iter().any(|g| g.area == 0.0), "the degenerate ring is present");
    assert!(GOLDENS.iter().any(|g| g.inside.iter().any(|&b| b)), "some probe lands inside");
    assert!(GOLDENS.iter().any(|g| g.inside.iter().any(|&b| !b)), "some probe lands outside");
    assert!(
        GOLDENS.iter().any(|g| g.points.iter().any(|p| p.0.fract() != 0.0)),
        "some ring has sub-cell vertices"
    );
}

#[test]
fn polygon_area_matches_the_reference_exactly() {
    for g in &GOLDENS {
        assert_eq!(polygon_area(g.points), g.area, "{}", g.label);
    }
}

#[test]
fn polygon_centroid_matches_the_reference_exactly() {
    for g in &GOLDENS {
        let (cx, cy) = polygon_centroid(g.points);
        assert_eq!(cx, g.centroid.0, "{} x", g.label);
        assert_eq!(cy, g.centroid.1, "{} y", g.label);
    }
}

#[test]
fn point_in_polygon_matches_the_reference_on_every_probe() {
    for g in &GOLDENS {
        for (i, &probe) in PROBES.iter().enumerate() {
            assert_eq!(
                point_in_polygon(probe, g.points),
                g.inside[i],
                "{} probe {i} at {probe:?}",
                g.label
            );
        }
    }
}

/// Mutation guard, stated rather than implied: the `/2.0` in `polygon_area`,
/// the `3.0 *` in `polygon_centroid`'s divisor and the `1e-9` degeneracy
/// floor each have exactly one fixture that kills them, and this test names
/// which.
#[test]
fn the_three_constants_each_have_a_fixture_that_kills_them() {
    // `/2.0`: the rectangle's raw shoelace sum is 48, not 24.
    assert_eq!(polygon_area(&RECT), 24.0);
    // `3.0 *`: dropping it scales `tri`'s centroid by exactly 3.
    let (cx, _) = polygon_centroid(&TRI);
    assert_ne!(cx, 3.333_333_333_333_333_5 * 3.0);
    // `1e-9`: `line` has sa == 0 and reaches the mean fallback, which lands
    // on (2, 0) -- an area-weighted result there would be 0/0.
    let (lx, ly) = polygon_centroid(&LINE);
    assert!(lx.is_finite() && ly.is_finite());
    assert_eq!((lx, ly), (2.0, 0.0));
}
