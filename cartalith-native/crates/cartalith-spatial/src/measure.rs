//! The Measure tool — `UNIFIED_TOOL_PLAN.md` milestone E.
//!
//! **This is an addition, not a port** (`DECISIONS.md` §7d). The plan recorded
//! *"zero reference precedent — grepped broadly (`ruler`, `measureDist`,
//! `distanceTool`) and found nothing"*, and re-checking the reference for this
//! milestone confirms it: the only distance readout anywhere is
//! `updateScaleBar` (line 14024), a passive scale bar, not an interactive
//! measure. There is therefore **no golden-parity test for this module** and
//! there cannot be one; it is unit-tested against its own stated contract
//! instead, and disclosed as new rather than presented as parity.
//!
//! What it *is* faithful to is the km scale, which is not invented: every
//! length in this port already comes from the same expression
//! `hypot(dx, dy) * map_width_km / grid_w` — `civ_smooth_path`'s own `km`
//! accumulation (`cartalith-civ`, golden-verified against `_civSmoothPath`),
//! `civ_catchment_radius_cells`' `cell_km`, and `_geoCellKm`'s GeoJSON
//! coordinate scale all use it. Measuring with a different one would report a
//! distance that disagreed with the route lengths shown beside it.
//!
//! Scope, deliberately minimal: a straight-line reading between two grid
//! points, wrap-aware in world mode. The plan floats a terrain-cost variant
//! ("real travel distance" via `road_dijkstra`) as *"a nice-to-have, not
//! required for a first version"* — that is not built here, and would belong
//! in `cartalith-civ` beside `civ_dijkstra_path` if it ever is, not in this
//! crate.

/// Kilometres per grid cell — `state.mapWidthKm / GW`.
///
/// The single expression every length in this port is derived from. Kept as a
/// named function so a future change has one place to make it.
#[inline]
pub fn cell_km(map_width_km: f64, grid_w: usize) -> f64 {
    map_width_km / grid_w as f64
}

/// A completed measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// Straight-line separation in grid cells.
    pub cells: f64,
    /// The same distance in kilometres.
    pub km: f64,
    /// Signed x separation actually used, after wrap resolution. Differs from
    /// `b.x - a.x` only when the short way round crosses the seam.
    pub dx: f64,
    /// Signed y separation (never wraps: the map is a cylinder, not a torus).
    pub dy: f64,
    /// Whether the shortest path between the two points crosses the seam.
    pub wrapped: bool,
}

/// Straight-line distance between two grid points.
///
/// In `world` mode the x axis wraps, so two points either side of the
/// antimeridian measure the short way round — the same `|Δx| > GW/2` seam test
/// `civ_smooth_path` uses to split a wrapped route into runs. `world = false`
/// never wraps.
pub fn measure(
    a: (f64, f64),
    b: (f64, f64),
    grid_w: usize,
    map_width_km: f64,
    world: bool,
) -> Measurement {
    let gw = grid_w as f64;
    let raw_dx = b.0 - a.0;
    let (dx, wrapped) = if world && gw > 0.0 && raw_dx.abs() > gw / 2.0 {
        (raw_dx - raw_dx.signum() * gw, true)
    } else {
        (raw_dx, false)
    };
    let dy = b.1 - a.1;
    let cells = dx.hypot(dy);
    Measurement { cells, km: cells * cell_km(map_width_km, grid_w), dx, dy, wrapped }
}

/// Total length of a multi-point measuring chain (click, click, click…),
/// summed leg by leg with the same wrap rule as [`measure`].
///
/// Returns `0.0` for fewer than two points rather than erroring — a chain
/// under construction is a normal state, not a failure.
pub fn measure_path(
    pts: &[(f64, f64)],
    grid_w: usize,
    map_width_km: f64,
    world: bool,
) -> Measurement {
    let mut cells = 0.0;
    let mut wrapped = false;
    for pair in pts.windows(2) {
        let m = measure(pair[0], pair[1], grid_w, map_width_km, world);
        cells += m.cells;
        wrapped |= m.wrapped;
    }
    let (dx, dy) = match (pts.first(), pts.last()) {
        (Some(f), Some(l)) if pts.len() >= 2 => (l.0 - f.0, l.1 - f.1),
        _ => (0.0, 0.0),
    };
    Measurement { cells, km: cells * cell_km(map_width_km, grid_w), dx, dy, wrapped }
}

// ===================== Polygon primitives (the Area tool) =====================
//
// `design/Cartalith Measurement Toolbar.dc.html` state 3 adds an **Area** tool
// (polygon / rectangle / freehand) whose right dock reads projected area,
// perimeter, centroid and bounding box. Unlike the ruler above, the three
// functions it needs are **not** new: the reference has all three, and this
// port already carries two of them in other shapes.
//
// - `polyArea` (reference line 28290) and `polyCentroid` (28291) live in
//   `cartalith-urban::geom` as `poly_area`/`poly_centroid`, over that crate's
//   own `Vec2`.
// - `_geoRingArea` (12526) and `_geoPointInRing` (12527) live *in this crate*
//   as `geo::ring_area`/`geo::point_in_ring`, over integer cell corners.
//
// Neither is reusable here as-is, and the reason is a real semantic
// difference, not a type annoyance: the `_geo*` pair takes an **explicitly
// closed** ring (last point == first, and both iterate `i < len - 1`), while
// `polyArea`/`polyCentroid`/`pointInPoly` take an **implicitly closed** one
// (`(i + 1) % n`). A user-drawn measuring ring is the second kind — the tool
// closes it for you rather than making you click the first vertex twice — so
// these are ports of the `poly*` family, at `f64`, and the tests below pin
// them against `geo::ring_area`/`geo::point_in_ring` on a ring that is legal
// under both conventions, so the two copies in this one crate cannot drift.

/// `polyArea` (reference line 28290) — signed shoelace area of an implicitly
/// closed polygon, in whatever unit the coordinates are in (grid cells, for
/// every caller here). Positive for counter-clockwise in a y-down grid;
/// callers that only want magnitude take `.abs()`.
///
/// Zero for fewer than three points, which is what the reference's own loop
/// produces rather than a special case.
pub fn polygon_area(p: &[(f64, f64)]) -> f64 {
    let n = p.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        s += a.0 * b.1 - b.0 * a.1;
    }
    s / 2.0
}

/// `polyCentroid` (reference line 28291) — the area-weighted centroid, with
/// the reference's own `|2A| < 1e-9` degenerate fallback to the plain vertex
/// mean.
///
/// An empty slice returns `(NaN, NaN)`, exactly as the reference does
/// (`mx / p.length` with `p.length == 0`) rather than a fabricated origin.
pub fn polygon_centroid(p: &[(f64, f64)]) -> (f64, f64) {
    let n = p.len();
    let (mut sx, mut sy, mut sa) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        let c = a.0 * b.1 - b.0 * a.1;
        sa += c;
        sx += (a.0 + b.0) * c;
        sy += (a.1 + b.1) * c;
    }
    if sa.abs() < 1e-9 {
        let (mut mx, mut my) = (0.0, 0.0);
        for q in p {
            mx += q.0;
            my += q.1;
        }
        return (mx / n as f64, my / n as f64);
    }
    (sx / (3.0 * sa), sy / (3.0 * sa))
}

/// `pointInPoly` (reference line 28295) — the crossing-number test over an
/// implicitly closed ring.
///
/// The empty case is guarded rather than transcribed: the reference's `j =
/// p.length - 1` is `-1` on an empty array and its loop simply never runs,
/// while the same expression on a `usize` underflows. Same answer (`false`),
/// reached without a panic.
pub fn point_in_polygon(pt: (f64, f64), p: &[(f64, f64)]) -> bool {
    if p.is_empty() {
        return false;
    }
    let mut inside = false;
    let mut j = p.len() - 1;
    for i in 0..p.len() {
        let (xi, yi) = p[i];
        let (xj, yj) = p[j];
        if ((yi > pt.1) != (yj > pt.1)) && (pt.0 < (xj - xi) * (pt.1 - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// The ring's own perimeter in kilometres, closed back to the first point,
/// summed leg by leg through [`measure`] so it inherits the same wrap rule
/// (and therefore the same km scale) every other length in this port uses.
///
/// **New, not a port** — the reference computes no polygon perimeter
/// anywhere. Fewer than two points measures `0.0`, same convention as
/// [`measure_path`].
pub fn polygon_perimeter_km(
    p: &[(f64, f64)],
    grid_w: usize,
    map_width_km: f64,
    world: bool,
) -> f64 {
    if p.len() < 2 {
        return 0.0;
    }
    let mut km = 0.0;
    for i in 0..p.len() {
        km += measure(p[i], p[(i + 1) % p.len()], grid_w, map_width_km, world).km;
    }
    km
}

/// Tests for the Measure tool (`UNIFIED_TOOL_PLAN.md` milestone E) — this
/// port's own addition, with no reference precedent to check against (see
/// the module doc comment above), so these tests are the only spec.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_km_is_map_width_over_grid_width() {
        // Protects: cell_km() being exactly `map_width_km / grid_w`, the one
        // expression every length in this port scales by.
        assert_eq!(cell_km(4000.0, 512), 4000.0 / 512.0);
    }

    #[test]
    fn a_horizontal_span_measures_its_share_of_the_map_width() {
        // Protects: measure() converting a cell-space distance to km via
        // cell_km(), and reporting no wrap in flat mode.
        // Half the grid across == half the map width, by construction.
        let m = measure((0.0, 10.0), (256.0, 10.0), 512, 4000.0, false);
        assert_eq!(m.cells, 256.0);
        assert_eq!(m.km, 2000.0);
        assert!(!m.wrapped);
    }

    #[test]
    fn a_zero_length_measurement_is_zero_not_an_error() {
        // Protects: measure() on two coincident points returning zero
        // rather than panicking or dividing by a zero span.
        let m = measure((7.0, 7.0), (7.0, 7.0), 512, 4000.0, false);
        assert_eq!(m.cells, 0.0);
        assert_eq!(m.km, 0.0);
    }

    #[test]
    fn flat_mode_never_wraps_however_far_apart_the_points_are() {
        // Protects: measure() with `world = false` never taking the seam
        // shortcut, even when the raw separation exceeds half the grid.
        let m = measure((1.0, 0.0), (511.0, 0.0), 512, 4000.0, false);
        assert_eq!(m.cells, 510.0);
        assert!(!m.wrapped);
    }

    #[test]
    fn world_mode_takes_the_short_way_round_the_seam() {
        // Protects: measure() with `world = true` splitting a >GW/2 raw
        // separation across the antimeridian, matching civ_smooth_path's
        // own seam test.
        let m = measure((1.0, 0.0), (511.0, 0.0), 512, 4000.0, true);
        assert!(m.wrapped);
        assert_eq!(m.cells, 2.0); // 1 -> 0 -> 511, not 510 the long way
        assert_eq!(m.dx, -2.0);
    }

    #[test]
    fn world_mode_leaves_a_short_span_alone() {
        // Protects: measure() with `world = true` not wrapping a separation
        // that is already the short way round.
        let m = measure((10.0, 0.0), (20.0, 0.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 10.0);
    }

    #[test]
    fn exactly_half_the_grid_apart_does_not_wrap() {
        // Protects: measure()'s seam test being strict `>`, not `>=` --
        // exactly half the grid apart does not wrap.
        // The test is `> gw/2`, not `>=`, matching civ_smooth_path's seam split.
        let m = measure((0.0, 0.0), (256.0, 0.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 256.0);
    }

    #[test]
    fn y_never_wraps_even_in_world_mode() {
        // Protects: measure() never applying the seam wrap to the y axis --
        // the map is a cylinder, not a torus, even in world mode.
        let m = measure((0.0, 1.0), (0.0, 400.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 399.0);
    }

    #[test]
    fn a_chain_sums_its_legs() {
        // Protects: measure_path() summing each leg's cells/km rather than
        // measuring only the first-to-last endpoints.
        let pts = [(0.0, 0.0), (3.0, 4.0), (3.0, 8.0)];
        let m = measure_path(&pts, 512, 512.0, false);
        assert_eq!(m.cells, 9.0); // 5 + 4
        assert_eq!(m.km, 9.0); // 1 km per cell here
    }

    #[test]
    fn a_chain_under_two_points_measures_zero() {
        // Protects: measure_path() returning 0.0 for an empty or
        // single-point chain rather than erroring on a chain still being
        // built.
        assert_eq!(measure_path(&[], 512, 4000.0, false).cells, 0.0);
        assert_eq!(measure_path(&[(1.0, 1.0)], 512, 4000.0, false).cells, 0.0);
    }

    #[test]
    fn a_chain_reports_wrapped_when_any_leg_crosses_the_seam() {
        // Protects: measure_path()'s `wrapped` flag being the OR of every
        // leg's own wrap, not just the last leg's.
        let pts = [(10.0, 0.0), (20.0, 0.0), (500.0, 0.0)];
        let m = measure_path(&pts, 512, 512.0, true);
        assert!(m.wrapped);
        assert_eq!(m.cells, 10.0 + 32.0);
    }

    #[test]
    fn the_km_scale_agrees_with_the_route_length_expression_civ_uses() {
        // Protects: measure()'s km scale being bit-identical to
        // civ_smooth_path's own `hypot(dx,dy) * map_width_km / gw` --
        // this module's header claim, pinned rather than just asserted.
        // civ_smooth_path accumulates `hypot(dx,dy) * map_width_km / gw`.
        let (gw, map_km) = (317usize, 4321.0f64);
        let (dx, dy) = (13.5f64, -7.25f64);
        let expected = dx.hypot(dy) * map_km / gw as f64;
        let m = measure((0.0, 0.0), (dx, dy), gw, map_km, false);
        assert_eq!(m.km.to_bits(), expected.to_bits());
    }

    // -- Polygon primitives ------------------------------------------------

    /// A 6x4 rectangle, implicitly closed (four vertices, not five).
    const RECT: [(f64, f64); 4] = [(2.0, 3.0), (8.0, 3.0), (8.0, 7.0), (2.0, 7.0)];

    #[test]
    fn a_rectangle_measures_its_own_width_times_height() {
        // Protects: polygon_area() on an axis-aligned rectangle equalling
        // its width * height.
        assert_eq!(polygon_area(&RECT).abs(), 24.0);
    }

    #[test]
    fn winding_flips_the_sign_but_not_the_magnitude() {
        // Protects: polygon_area()'s sign tracking winding direction while
        // the magnitude stays the same for the reversed ring.
        let mut rev = RECT.to_vec();
        rev.reverse();
        assert_eq!(polygon_area(&RECT), -polygon_area(&rev));
    }

    #[test]
    fn a_degenerate_polygon_measures_zero_rather_than_erroring() {
        // Protects: polygon_area() on 0/1/2-point input returning 0.0, the
        // same degenerate answer the reference's own loop produces without
        // a special case.
        assert_eq!(polygon_area(&[]), 0.0);
        assert_eq!(polygon_area(&[(1.0, 1.0)]), 0.0);
        assert_eq!(polygon_area(&[(1.0, 1.0), (4.0, 4.0)]), 0.0);
    }

    #[test]
    fn a_rectangles_centroid_is_its_middle() {
        // Protects: polygon_centroid() on a rectangle landing on its
        // geometric middle.
        let (cx, cy) = polygon_centroid(&RECT);
        assert!((cx - 5.0).abs() < 1e-12, "{cx}");
        assert!((cy - 5.0).abs() < 1e-12, "{cy}");
    }

    /// The reference's own `Math.abs(sa) < 1e-9` branch: a collinear ring has
    /// no area to weight by, so it falls back to the plain vertex mean.
    #[test]
    fn a_zero_area_polygon_falls_back_to_the_vertex_mean() {
        // Protects: polygon_centroid()'s `|2A| < 1e-9` collinear fallback to
        // the plain vertex mean, the reference's own degenerate case.
        let line = [(0.0, 0.0), (2.0, 0.0), (4.0, 0.0)];
        let (cx, cy) = polygon_centroid(&line);
        assert_eq!((cx, cy), (2.0, 0.0));
    }

    #[test]
    fn point_in_polygon_answers_inside_outside_and_never_panics_when_empty() {
        // Protects: point_in_polygon() answering inside/outside correctly
        // for a rectangle and returning false (not panicking) for an empty
        // ring.
        assert!(point_in_polygon((5.0, 5.0), &RECT));
        assert!(!point_in_polygon((0.0, 0.0), &RECT));
        assert!(!point_in_polygon((9.0, 5.0), &RECT));
        assert!(!point_in_polygon((5.0, 5.0), &[]));
    }

    /// A concave ring is where a bounding-box test would disagree with a real
    /// crossing-number one, so it is the fixture that proves this is the
    /// latter: the notch's interior point is *outside* the polygon.
    #[test]
    fn a_concave_ring_excludes_its_own_notch() {
        // Protects: point_in_polygon() using a real crossing-number test,
        // not a bounding-box shortcut that would miss the notch.
        // A "U": tall left and right arms with a gap bitten out of the top.
        let u = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (7.0, 10.0),
            (7.0, 3.0),
            (3.0, 3.0),
            (3.0, 10.0),
            (0.0, 10.0),
        ];
        assert!(point_in_polygon((5.0, 1.0), &u), "the base is inside");
        assert!(!point_in_polygon((5.0, 6.0), &u), "the notch is outside");
        assert!(point_in_polygon((8.5, 6.0), &u), "the right arm is inside");
    }

    /// The anti-drift pin this module's own header promises: on a ring that is
    /// legal under *both* conventions (explicitly closed, so `_geoRingArea`'s
    /// `i < len - 1` walks every real edge and `polyArea`'s `(i + 1) % n`
    /// contributes a zero-length closing edge) the two implementations in this
    /// one crate must agree exactly.
    #[test]
    fn the_implicit_and_explicit_shoelace_agree_on_a_closed_ring() {
        // Protects: polygon_area() and geo::ring_area() agreeing exactly on
        // a ring legal under both conventions -- the anti-drift pin this
        // module's header promises.
        let closed_i: [(i32, i32); 5] = [(2, 3), (8, 3), (8, 7), (2, 7), (2, 3)];
        let closed_f: Vec<(f64, f64)> =
            closed_i.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
        assert_eq!(polygon_area(&closed_f), crate::geo::ring_area(&closed_i));
    }

    #[test]
    fn the_two_point_in_ring_tests_agree_on_a_closed_ring() {
        // Protects: point_in_polygon() and geo::point_in_ring() agreeing on
        // every probe against a ring legal under both conventions.
        let closed_i: [(i32, i32); 5] = [(2, 3), (8, 3), (8, 7), (2, 7), (2, 3)];
        let closed_f: Vec<(f64, f64)> =
            closed_i.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
        for (px, py) in [(5.0, 5.0), (0.5, 5.0), (5.0, 0.5), (7.9, 6.9), (8.5, 5.0)] {
            assert_eq!(
                point_in_polygon((px, py), &closed_f),
                crate::geo::point_in_ring(px, py, &closed_i),
                "({px}, {py})"
            );
        }
    }

    #[test]
    fn a_rectangles_perimeter_sums_all_four_sides_including_the_closing_one() {
        // Protects: polygon_perimeter_km() closing the ring back to its
        // first point and summing all four legs via measure()'s km scale.
        // 1 km per cell here (512 km over 512 cells), so cells == km.
        assert_eq!(polygon_perimeter_km(&RECT, 512, 512.0, false), 20.0);
    }

    #[test]
    fn a_perimeter_under_two_points_measures_zero() {
        // Protects: polygon_perimeter_km() on 0/1-point input returning
        // 0.0, matching measure_path()'s own convention.
        assert_eq!(polygon_perimeter_km(&[], 512, 512.0, false), 0.0);
        assert_eq!(polygon_perimeter_km(&[(1.0, 1.0)], 512, 512.0, false), 0.0);
    }
}
