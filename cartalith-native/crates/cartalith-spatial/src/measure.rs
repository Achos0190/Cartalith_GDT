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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_km_is_map_width_over_grid_width() {
        assert_eq!(cell_km(4000.0, 512), 4000.0 / 512.0);
    }

    #[test]
    fn a_horizontal_span_measures_its_share_of_the_map_width() {
        // Half the grid across == half the map width, by construction.
        let m = measure((0.0, 10.0), (256.0, 10.0), 512, 4000.0, false);
        assert_eq!(m.cells, 256.0);
        assert_eq!(m.km, 2000.0);
        assert!(!m.wrapped);
    }

    #[test]
    fn a_zero_length_measurement_is_zero_not_an_error() {
        let m = measure((7.0, 7.0), (7.0, 7.0), 512, 4000.0, false);
        assert_eq!(m.cells, 0.0);
        assert_eq!(m.km, 0.0);
    }

    #[test]
    fn flat_mode_never_wraps_however_far_apart_the_points_are() {
        let m = measure((1.0, 0.0), (511.0, 0.0), 512, 4000.0, false);
        assert_eq!(m.cells, 510.0);
        assert!(!m.wrapped);
    }

    #[test]
    fn world_mode_takes_the_short_way_round_the_seam() {
        let m = measure((1.0, 0.0), (511.0, 0.0), 512, 4000.0, true);
        assert!(m.wrapped);
        assert_eq!(m.cells, 2.0); // 1 -> 0 -> 511, not 510 the long way
        assert_eq!(m.dx, -2.0);
    }

    #[test]
    fn world_mode_leaves_a_short_span_alone() {
        let m = measure((10.0, 0.0), (20.0, 0.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 10.0);
    }

    #[test]
    fn exactly_half_the_grid_apart_does_not_wrap() {
        // The test is `> gw/2`, not `>=`, matching civ_smooth_path's seam split.
        let m = measure((0.0, 0.0), (256.0, 0.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 256.0);
    }

    #[test]
    fn y_never_wraps_even_in_world_mode() {
        let m = measure((0.0, 1.0), (0.0, 400.0), 512, 4000.0, true);
        assert!(!m.wrapped);
        assert_eq!(m.cells, 399.0);
    }

    #[test]
    fn a_chain_sums_its_legs() {
        let pts = [(0.0, 0.0), (3.0, 4.0), (3.0, 8.0)];
        let m = measure_path(&pts, 512, 512.0, false);
        assert_eq!(m.cells, 9.0); // 5 + 4
        assert_eq!(m.km, 9.0); // 1 km per cell here
    }

    #[test]
    fn a_chain_under_two_points_measures_zero() {
        assert_eq!(measure_path(&[], 512, 4000.0, false).cells, 0.0);
        assert_eq!(measure_path(&[(1.0, 1.0)], 512, 4000.0, false).cells, 0.0);
    }

    #[test]
    fn a_chain_reports_wrapped_when_any_leg_crosses_the_seam() {
        let pts = [(10.0, 0.0), (20.0, 0.0), (500.0, 0.0)];
        let m = measure_path(&pts, 512, 512.0, true);
        assert!(m.wrapped);
        assert_eq!(m.cells, 10.0 + 32.0);
    }

    #[test]
    fn the_km_scale_agrees_with_the_route_length_expression_civ_uses() {
        // civ_smooth_path accumulates `hypot(dx,dy) * map_width_km / gw`.
        let (gw, map_km) = (317usize, 4321.0f64);
        let (dx, dy) = (13.5f64, -7.25f64);
        let expected = dx.hypot(dy) * map_km / gw as f64;
        let m = measure((0.0, 0.0), (dx, dy), gw, map_km, false);
        assert_eq!(m.km.to_bits(), expected.to_bits());
    }
}
