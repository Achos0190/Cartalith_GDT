//! How much empty ground a town wall encloses -- the measurement behind Ruling
//! EW's "walls were expensive, so a lot of free space between the wall and the
//! first city on the inside is unrealistic" (`LARGE_ITEM_RULINGS.md`, item 1,
//! last point). **Not a reference port and not a generation stage**: it reads a
//! finished town and changes nothing, so it can run before and after
//! [`crate::wallhug`] and be compared.
//!
//! Two numbers, both over the *land* arc of the circuit (a water wall follows a
//! bank and has no "inside" to leave empty):
//!
//! - **the gap**: for sample points every [`SAMPLE_STEP_M`] metres along the
//!   wall, the distance to the nearest building standing inside the circuit.
//!   [`WallGap::median_m`] and [`WallGap::p90_m`] are its quantiles. The
//!   2026-09-22 measurement in [`crate::wallside`]'s header reads the same
//!   quantity (*"the median gap from the wall to the nearest intramural
//!   building was 38-81 m"*), so the two are comparable.
//! - **the free share**: the fraction of the walled area farther than
//!   [`FREE_R_M`] from every building -- ground a street frontage does not
//!   reach, i.e. the "free space" the owner means.
//!
//! A town with no building inside its circuit, or a circuit with no land arc,
//! returns [`None`]: there is nothing to measure, and `0.0` would read as a
//! perfect fit (MISTAKES.md: never encode "no value" as a plausible value).

use crate::geom::{Vec2, dist_pt_seg, point_in_poly, poly_area, poly_centroid};

/// Spacing of the sample points along the wall, metres. A labelled judgement:
/// finer than the smallest building (about 4 m) would only repeat samples.
pub const SAMPLE_STEP_M: f64 = 4.0;
/// Grid spacing used to estimate the free share, metres. A labelled judgement:
/// 6 m resolves a lot (8 m frontage, `build_parcels`) at a few thousand points
/// for a 700 m town.
pub const AREA_STEP_M: f64 = 6.0;
/// A point farther than this from every building counts as free, metres. A
/// labelled judgement: below `build_parcels`' own 46 m lot-depth ceiling (a lot
/// plus its building cannot put a building farther than that from its street)
/// and above a back garden, so ground this far from any building is a gap in
/// the plat and not a yard.
pub const FREE_R_M: f64 = 30.0;
/// Cell edge of the segment grid the nearest-building query uses, metres.
const CELL_M: f64 = 40.0;
/// The query gives up beyond this distance and returns it as the answer, metres.
/// Larger than any town's radius (`max_rf` is at most 720), so a cap hit means
/// "the wall has no building within 400 m", which is a finding, not a bound.
const QUERY_CAP_M: f64 = 400.0;

/// The wall-to-building gap and the free share of one walled town.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallGap {
    /// Median over the wall samples of the distance to the nearest building, metres.
    pub median_m: f64,
    /// The 90th percentile of the same, metres.
    pub p90_m: f64,
    /// Fraction of the walled area farther than [`FREE_R_M`] from every building.
    pub free_share: f64,
    /// Walled area, square metres (for weighting and for the report).
    pub area_m2: f64,
    /// Number of wall samples the quantiles are over.
    pub samples: usize,
}

/// Every building segment, bucketed by [`CELL_M`], for nearest-distance queries.
struct SegGrid {
    cells: std::collections::HashMap<(i32, i32), Vec<(Vec2, Vec2)>>,
}

impl SegGrid {
    fn new(polys: &[&[Vec2]]) -> Self {
        let mut cells: std::collections::HashMap<(i32, i32), Vec<(Vec2, Vec2)>> =
            std::collections::HashMap::new();
        for poly in polys {
            for i in 0..poly.len() {
                let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
                let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
                for cx in (x0 / CELL_M).floor() as i32..=(x1 / CELL_M).floor() as i32 {
                    for cy in (y0 / CELL_M).floor() as i32..=(y1 / CELL_M).floor() as i32 {
                        cells.entry((cx, cy)).or_default().push((a, b));
                    }
                }
            }
        }
        SegGrid { cells }
    }

    /// Distance from `p` to the nearest segment, exact up to [`QUERY_CAP_M`].
    fn nearest(&self, p: Vec2) -> f64 {
        let (cx, cy) = ((p.x / CELL_M).floor() as i32, (p.y / CELL_M).floor() as i32);
        let mut best = f64::INFINITY;
        let mut r = 0i32;
        loop {
            // Ring `r` of cells: anything in it is at least `(r - 1) * CELL_M`
            // away, so once `best` is within that the search is exact.
            for dx in -r..=r {
                for dy in -r..=r {
                    if dx.abs().max(dy.abs()) != r {
                        continue;
                    }
                    if let Some(v) = self.cells.get(&(cx + dx, cy + dy)) {
                        for (a, b) in v {
                            best = best.min(dist_pt_seg(p, *a, *b));
                        }
                    }
                }
            }
            if best <= f64::from(r) * CELL_M || f64::from(r) * CELL_M >= QUERY_CAP_M {
                return best.min(QUERY_CAP_M);
            }
            r += 1;
        }
    }
}

/// `q`-quantile (nearest-rank) of an ascending, non-empty slice.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    let i = ((sorted.len() as f64) * q).ceil() as usize;
    sorted[i.clamp(1, sorted.len()) - 1]
}

/// Measure one circuit. `ring` is the containment ring ([`crate::WallState::ring`]),
/// `land_arc` the drawn land-facing arc ([`crate::WallState::land_arc`], `None`
/// to take the whole ring), `buildings` every building footprint (those outside
/// the ring are ignored, so a faubourg does not count as inside).
pub fn wall_gap(ring: &[Vec2], land_arc: Option<&[Vec2]>, buildings: &[&[Vec2]]) -> Option<WallGap> {
    if ring.len() < 3 {
        return None;
    }
    let inside: Vec<&[Vec2]> = buildings
        .iter()
        .copied()
        .filter(|b| b.len() >= 3 && point_in_poly(poly_centroid(b), ring))
        .collect();
    if inside.is_empty() {
        return None;
    }
    let grid = SegGrid::new(&inside);

    // Wall samples: the land arc as an open polyline (or the ring as a closed one).
    let line: Vec<Vec2> = match land_arc {
        Some(a) if a.len() >= 2 => a.to_vec(),
        Some(_) => return None,
        None => {
            let mut l = ring.to_vec();
            l.push(ring[0]);
            l
        }
    };
    let mut gaps: Vec<f64> = Vec::new();
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.dist(b);
        let n = ((len / SAMPLE_STEP_M).ceil() as usize).max(1);
        for k in 0..n {
            let t = k as f64 / n as f64;
            gaps.push(grid.nearest(a + (b - a) * t));
        }
    }
    if gaps.is_empty() {
        return None;
    }
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Free share: grid points inside the ring, farther than FREE_R_M from a building.
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for p in ring {
        x0 = x0.min(p.x);
        x1 = x1.max(p.x);
        y0 = y0.min(p.y);
        y1 = y1.max(p.y);
    }
    let (mut inside_pts, mut free_pts) = (0usize, 0usize);
    let mut y = y0 + AREA_STEP_M / 2.0;
    while y < y1 {
        let mut x = x0 + AREA_STEP_M / 2.0;
        while x < x1 {
            let p = Vec2::new(x, y);
            if point_in_poly(p, ring) {
                inside_pts += 1;
                if grid.nearest(p) > FREE_R_M {
                    free_pts += 1;
                }
            }
            x += AREA_STEP_M;
        }
        y += AREA_STEP_M;
    }
    if inside_pts == 0 {
        return None;
    }
    Some(WallGap {
        median_m: quantile(&gaps, 0.5),
        p90_m: quantile(&gaps, 0.9),
        free_share: free_pts as f64 / inside_pts as f64,
        area_m2: poly_area(ring).abs(),
        samples: gaps.len(),
    })
}
