//! The colour-paint **fill helper** (`COLOUR_PAINT_SCOPE.md` §13, milestone
//! CP-5): click inside a crater and get its outline as a closed ring in map
//! kilometres, ready to become an `Area` mark.
//!
//! # What it is
//!
//! Pure functions over a height field window: no render context, no colour,
//! no world mutation. The shell clicks a cell, asks [`fill_from_click`] for a
//! preview, moves the level / cap sliders (each move asks again) and commits
//! the ring as an area mark (plus an optional hard "grow" stroke along it).
//!
//! # The two modes, and why both
//!
//! The scope's §13 measured both on two real worlds: a **closed fill to the
//! pour point** (grow from the lowest cell the way water fills, stop at the
//! first boundary cell lower than the level reached, trace the level set) and
//! a **capped fill** for breached craters (every cell below the rim level
//! less a margin, connected to the click and within a radius cap, smoothed
//! and traced at one half). *Every crater found on those worlds was breached
//! or too small at grid resolution for the closed fill*, so the closed fill
//! is tried first and the capped fill is the usual answer. The tool is
//! **click-seeded, never automatic**: an automatic detector took a river
//! gorge's rimmed pits for a crater on 71077345.
//!
//! # Provenance
//!
//! The algorithm is the prototype in `cartalith-godot/tests/colour_paint_frames.rs`
//! (`crater_fill`, `capped_fill`, `ring_around`, `rays`, written 2026-10-18),
//! moved here and made total: every out-of-range window returns an error, never
//! a panic or a wrapped index. The constants below are that prototype's
//! measured judgements, each named where it is defined.
//!
//! # What it must never do
//!
//! - Read or write anything but the field it is lent.
//! - Return a ring it did not trace from a real contour: "no ring" is an
//!   [`FillError`], never an empty or default ring.
//! - Take unbounded time: the flood stops at [`MAX_CLOSED_CELLS`] and the
//!   capped fill is bounded by its radius cap ([`MAX_CAP_CELLS`]).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::contour::contour_polylines;

/// How many cells around the click the closed fill looks for the lowest cell
/// to start from, so a click on the crater wall near the floor still seeds the
/// floor. Labelled judgement: a user's tap lands a few cells off at the zoom a
/// crater (5-10 km across, §13) is filled at.
pub const SEEK_CELLS: usize = 4;

/// The closed fill gives up after this many cells (the prototype's 20 000: a
/// region this large is not a crater, and the flood must stay cheap).
pub const MAX_CLOSED_CELLS: usize = 20_000;

/// A closed fill smaller than this many cells is "no depression" and the
/// capped fill takes over (the prototype's `>= 20` filter): one- and
/// two-cell pits in a noisy field are not craters.
pub const MIN_CLOSED_CELLS: usize = 20;

/// The window half-width, in cells, the closed fill's level set is traced in.
/// 90 cells reaches a 20 000-cell region (about 80 cells across) with room.
pub const RING_WINDOW_CELLS: usize = 90;

/// The rays that estimate the rim: 16 directions to [`RIM_RAY_CELLS`], the
/// prototype's own, whose median maximum is the rim height and whose median
/// distance to it is the rim distance.
const RIM_RAYS: usize = 16;

/// How far the rim rays reach, in cells (the prototype's 14).
pub const RIM_RAY_CELLS: usize = 14;

/// The capped fill's automatic level is the rim height less this much
/// normalised height (the prototype's `rim - 0.01`). Labelled judgement: just
/// under the rim, so the fill stops at the crest and does not spill onto the
/// flank.
pub const RIM_MARGIN: f32 = 0.01;

/// The capped fill's automatic radius cap, as a multiple of the median rim
/// distance (the prototype's 1.3): wide enough to take the whole bowl, tight
/// enough that a breach closes with an arc instead of a lobe down the gorge.
pub const CAP_OF_RIM_DISTANCE: f64 = 1.3;

/// The largest radius cap a caller may ask for, in cells. Bounds the capped
/// fill's window (`(2 cap + 7)^2` cells). A labelled judgement: 60 cells is
/// 4 x the rim rays, far past any crater and still a few tens of thousands of
/// cells.
pub const MAX_CAP_CELLS: f64 = 60.0;

/// The level set the closed fill traces sits just under the spill level so the
/// ring is inside the pour point (the prototype's `level - 1e-5`).
const POUR_EPS: f32 = 1e-5;

/// A click whose rim estimate does not rise above the floor by at least this
/// much normalised height is not in a depression. Labelled judgement: the
/// prototype's craters rose 0.06 or more on at least 15 of 16 rays; a fifth
/// of that is a deliberately forgiving floor, because the user, not a
/// detector, decided it is a crater.
pub const MIN_RIM_RISE: f32 = 0.012;

/// Rings are thinned to this many points at most (the engine's per-mark cap,
/// `colour_paint::MAX_POINTS_PER_MARK`, restated so this module has no
/// dependency on a private constant).
const MAX_RING_POINTS: usize = 4_096;

/// Which of the two algorithms produced the ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    /// The level set at the pour point (a closed crater).
    PourPoint,
    /// The capped fill below the rim (a breached crater).
    Capped,
}

impl FillMode {
    /// The key the shell shows and a probe asserts on.
    pub fn key(self) -> &'static str {
        match self {
            FillMode::PourPoint => "pour_point",
            FillMode::Capped => "capped",
        }
    }
}

/// What a caller may move. `None` means "the automatic value" -- never a
/// sentinel number.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FillParams {
    /// Added to the automatic level, in normalised height (the field's own
    /// units; the bridge converts the shell's metres). Positive fills higher.
    pub level_delta: f32,
    /// The radius cap in cells, or `None` for the automatic one
    /// ([`CAP_OF_RIM_DISTANCE`] x the rim distance). Used by the capped fill
    /// only; a closed fill has no cap.
    pub cap_cells: Option<f64>,
}

/// A traced fill.
#[derive(Clone, Debug)]
pub struct Fill {
    /// The closed ring, `[x_km, y_km]` from the map's top-left edge, last
    /// point not repeated.
    pub ring_km: Vec<[f64; 2]>,
    pub mode: FillMode,
    /// Cells inside the flood (before smoothing).
    pub cells: usize,
    /// The level the ring was traced at, normalised height (automatic level
    /// plus `level_delta`).
    pub level: f32,
    /// The automatic level for this click, so a shell can show "+20 m".
    pub auto_level: f32,
    /// The radius cap used, in cells (`None` for a closed fill).
    pub cap_cells: Option<f64>,
    /// The automatic radius cap, in cells (always computed: the capped fill's
    /// own default, shown even while the closed fill answered).
    pub auto_cap_cells: f64,
    /// The cell the flood started from (the lowest near the click).
    pub seed: (usize, usize),
}

/// Why no ring came back. Every variant is something the shell can say in a
/// sentence.
#[derive(Clone, Debug, PartialEq)]
pub enum FillError {
    /// The field is empty, ragged, or the grid or scale is not usable.
    BadField,
    /// The click is outside the grid.
    OutsideGrid,
    /// The click is too near the map edge for the rim rays and the window.
    TooNearEdge,
    /// The click is not in a depression: the rim does not rise above it.
    NotInDepression,
    /// The level moved so far that nothing (or the whole window) is under it.
    NoRingAtLevel,
    /// A radius cap that is not finite, not positive, or above [`MAX_CAP_CELLS`].
    BadCap,
    /// A level that is not finite.
    BadLevel,
}

impl std::fmt::Display for FillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FillError::BadField => write!(f, "this world has no height field to fill from"),
            FillError::OutsideGrid => write!(f, "that point is outside the map"),
            FillError::TooNearEdge => write!(f, "that point is too near the map edge to outline a crater"),
            FillError::NotInDepression => write!(f, "that point is not inside a crater or a bowl: the ground around it does not rise"),
            FillError::NoRingAtLevel => write!(f, "no outline closes around that point at this level; lower the level or raise the cap"),
            FillError::BadCap => write!(f, "the cap must be a positive size no larger than {MAX_CAP_CELLS} cells"),
            FillError::BadLevel => write!(f, "the level is not a number"),
        }
    }
}

/// Outlines the crater under the click cell `(cx, cy)`.
///
/// `field` is `gw * gh` normalised heights; `kpc` is kilometres per cell.
/// Tries the closed fill from the lowest cell within [`SEEK_CELLS`] of the
/// click; when that is not a closed region of at least [`MIN_CLOSED_CELLS`]
/// (a breached crater, the usual case) the capped fill answers. `params` moves
/// the level and the cap from their automatic values.
pub fn fill_from_click(field: &[f32], gw: usize, gh: usize, cx: usize, cy: usize, kpc: f64, params: FillParams) -> Result<Fill, FillError> {
    if gw < 2 || gh < 2 || field.len() < gw * gh || !kpc.is_finite() || kpc <= 0.0 {
        return Err(FillError::BadField);
    }
    if cx >= gw || cy >= gh {
        return Err(FillError::OutsideGrid);
    }
    if !params.level_delta.is_finite() {
        return Err(FillError::BadLevel);
    }
    if let Some(c) = params.cap_cells {
        if !c.is_finite() || c <= 0.0 || c > MAX_CAP_CELLS {
            return Err(FillError::BadCap);
        }
    }
    // The rim rays reach RIM_RAY_CELLS and the capped window reaches the
    // largest cap + 3; a click nearer the edge than the rays is refused rather
    // than sampled outside the grid.
    let m = RIM_RAY_CELLS + 1;
    if cx < m || cy < m || cx + m >= gw || cy + m >= gh {
        return Err(FillError::TooNearEdge);
    }
    let floor = field[cy * gw + cx];
    let (rim, rim_d) = rim_of(field, gw, cx, cy);
    if rim - floor < MIN_RIM_RISE {
        return Err(FillError::NotInDepression);
    }
    let auto_cap = CAP_OF_RIM_DISTANCE * rim_d;
    let closed = pour_point(field, gw, gh, cx, cy);
    if let Some((spill, cells, seed)) = closed.filter(|c| c.1 >= MIN_CLOSED_CELLS) {
        let auto_level = spill - POUR_EPS;
        let level = auto_level + params.level_delta;
        if let Some(ring) = ring_around(field, gw, gh, seed, level as f64, RING_WINDOW_CELLS, kpc) {
            return Ok(Fill { ring_km: thin(ring), mode: FillMode::PourPoint, cells, level, auto_level, cap_cells: None, auto_cap_cells: auto_cap, seed });
        }
    }
    // Capped fill. Its window must lie inside the grid: refuse a click whose
    // cap would leave it, instead of wrapping.
    let cap = params.cap_cells.unwrap_or(auto_cap).min(MAX_CAP_CELLS);
    let half = cap.ceil() as usize + 3;
    if cx < half || cy < half || cx + half >= gw || cy + half >= gh {
        return Err(FillError::TooNearEdge);
    }
    let auto_level = rim - RIM_MARGIN;
    let level = auto_level + params.level_delta;
    let (ring, cells) = capped(field, gw, (cx, cy), level, cap, kpc).ok_or(FillError::NoRingAtLevel)?;
    Ok(Fill { ring_km: thin(ring), mode: FillMode::Capped, cells, level, auto_level, cap_cells: Some(cap), auto_cap_cells: auto_cap, seed: (cx, cy) })
}

/// Thins a ring to at most [`MAX_RING_POINTS`] by taking every k-th point.
fn thin(ring: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if ring.len() <= MAX_RING_POINTS {
        return ring;
    }
    let k = ring.len().div_ceil(MAX_RING_POINTS);
    ring.into_iter().step_by(k).collect()
}

/// The rim height (median of 16 rays' maxima) and distance (median distance
/// to them, cells) around `(x, y)`. The caller has checked the rays stay in
/// the grid.
fn rim_of(f: &[f32], gw: usize, x: usize, y: usize) -> (f32, f64) {
    let mut maxes: Vec<(f32, f64)> = Vec::with_capacity(RIM_RAYS);
    for k in 0..RIM_RAYS {
        let a = k as f64 * std::f64::consts::TAU / RIM_RAYS as f64;
        let mut best = (f32::MIN, 0.0);
        for r in 1..=RIM_RAY_CELLS {
            let (qx, qy) = ((x as f64 + r as f64 * a.cos()).round() as usize, (y as f64 + r as f64 * a.sin()).round() as usize);
            let v = f[qy * gw + qx];
            if v > best.0 {
                best = (v, r as f64);
            }
        }
        maxes.push(best);
    }
    let mut hs: Vec<f32> = maxes.iter().map(|b| b.0).collect();
    hs.sort_by(f32::total_cmp);
    let mut ds: Vec<f64> = maxes.iter().map(|b| b.1).collect();
    ds.sort_by(f64::total_cmp);
    (hs[RIM_RAYS / 2], ds[RIM_RAYS / 2])
}

/// A cell in the flood's min-heap (lowest first; ties by index so the result
/// is deterministic).
#[derive(PartialEq)]
struct HeapCell(f32, usize);
impl Eq for HeapCell {}
impl PartialOrd for HeapCell {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for HeapCell {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0).then(o.1.cmp(&self.1))
    }
}

/// The closed fill: from the lowest cell within [`SEEK_CELLS`] of `(cx, cy)`,
/// raise the level the way water fills the basin, stopping at the pour point
/// (the first boundary cell lower than the level reached). Returns the spill
/// level, the cell count and the seed; `None` when the region reaches the map
/// edge or [`MAX_CLOSED_CELLS`] (not a closed depression within reach) or is
/// a single cell.
fn pour_point(field: &[f32], gw: usize, gh: usize, cx: usize, cy: usize) -> Option<(f32, usize, (usize, usize))> {
    let mut seed = (cx, cy);
    for y in cy.saturating_sub(SEEK_CELLS)..(cy + SEEK_CELLS).min(gh) {
        for x in cx.saturating_sub(SEEK_CELLS)..(cx + SEEK_CELLS).min(gw) {
            if (x as f64 - cx as f64).hypot(y as f64 - cy as f64) <= SEEK_CELLS as f64 && field[y * gw + x] < field[seed.1 * gw + seed.0] {
                seed = (x, y);
            }
        }
    }
    // The field is normalised and clipped at its maximum: a summit rim at the
    // ceiling is a flat wall the flood could only cross by spreading over the
    // whole clipped plateau, so a ceiling cell is a wall (a crater whose rim
    // is all ceiling is closed at the ceiling; seen on 71077345's snow-rimmed
    // cone, `colour_paint_frames.rs`).
    let ceiling = field[..gw * gh].iter().copied().fold(f32::MIN, f32::max);
    let mut walled = false;
    let mut seen = vec![false; gw * gh];
    let mut heap = BinaryHeap::new();
    let s = seed.1 * gw + seed.0;
    heap.push(HeapCell(field[s], s));
    seen[s] = true;
    let mut level = field[s];
    let mut count = 0usize;
    while let Some(HeapCell(h, i)) = heap.pop() {
        if h < level {
            break; // the pour point: lower than the water already reached
        }
        level = h;
        count += 1;
        if count >= MAX_CLOSED_CELLS {
            return None;
        }
        let (x, y) = (i % gw, i / gw);
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x as i64 + dx, y as i64 + dy);
            if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 {
                return None; // reached the map edge
            }
            let j = ny as usize * gw + nx as usize;
            if !seen[j] {
                seen[j] = true;
                if field[j] >= ceiling {
                    walled = true;
                    continue;
                }
                heap.push(HeapCell(field[j], j));
            }
        }
    }
    if count <= 1 {
        return None;
    }
    let level = if heap.is_empty() && walled { ceiling - 1e-4 } else { level };
    Some((level, count, seed))
}

/// The closed level-set ring of `field` at `level` that encloses `seed`,
/// traced in a window of `half` cells around it, in km. Polylines from
/// `contour_polylines` are in cell-centre pixel space (`col + 0.5`), which is
/// exactly the colour-paint kilometre convention times `kpc`.
fn ring_around(field: &[f32], gw: usize, gh: usize, seed: (usize, usize), level: f64, half: usize, kpc: f64) -> Option<Vec<[f64; 2]>> {
    let (x0, y0) = (seed.0.saturating_sub(half), seed.1.saturating_sub(half));
    let (x1, y1) = ((seed.0 + half).min(gw), (seed.1 + half).min(gh));
    let (w, h) = (x1 - x0, y1 - y0);
    let mut win = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            win[y * w + x] = field[(y0 + y) * gw + x0 + x];
        }
    }
    let p = [(seed.0 - x0) as f64 + 0.5, (seed.1 - y0) as f64 + 0.5];
    let mut best: Option<Vec<(f64, f64)>> = None;
    for pl in contour_polylines(&win, w, h, level) {
        let closed = pl.len() > 3 && (pl[0].0 - pl[pl.len() - 1].0).abs() < 1e-9 && (pl[0].1 - pl[pl.len() - 1].1).abs() < 1e-9;
        if !closed {
            continue;
        }
        let ring: Vec<(f64, f64)> = pl[..pl.len() - 1].to_vec();
        if inside(&ring, p) && best.as_ref().is_none_or(|b| ring.len() < b.len()) {
            best = Some(ring);
        }
    }
    best.map(|r| r.into_iter().map(|(x, y)| [(x + x0 as f64) * kpc, (y + y0 as f64) * kpc]).collect())
}

/// The capped fill: every cell below `level` connected to the seed and within
/// `cap` cells of it, as a mask, box-blurred twice so its contour at one half
/// is a smooth curve rather than a cell staircase, traced in km. The caller
/// has checked the window `(2 * ceil(cap) + 7)^2` lies inside the grid.
fn capped(field: &[f32], gw: usize, seed: (usize, usize), level: f32, cap: f64, kpc: f64) -> Option<(Vec<[f64; 2]>, usize)> {
    let half = cap.ceil() as usize + 3;
    let (x0, y0) = (seed.0 - half, seed.1 - half);
    let w = 2 * half + 1;
    let mut mask = vec![0f32; w * w];
    let mut stack = vec![(half, half)];
    mask[half * w + half] = 1.0;
    let mut count = 0;
    while let Some((x, y)) = stack.pop() {
        count += 1;
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = ((x as i64 + dx) as usize, (y as i64 + dy) as usize);
            if nx >= w || ny >= w || mask[ny * w + nx] > 0.0 {
                continue;
            }
            if (nx as f64 - half as f64).hypot(ny as f64 - half as f64) > cap || field[(y0 + ny) * gw + x0 + nx] >= level {
                continue;
            }
            mask[ny * w + nx] = 1.0;
            stack.push((nx, ny));
        }
    }
    for _ in 0..2 {
        let src = mask.clone();
        for y in 1..w - 1 {
            for x in 1..w - 1 {
                let mut acc = 0.0;
                for dy in 0..3 {
                    for dx in 0..3 {
                        acc += src[(y + dy - 1) * w + x + dx - 1];
                    }
                }
                mask[y * w + x] = acc / 9.0;
            }
        }
    }
    let p = (half as f64 + 0.5, half as f64 + 0.5);
    let ring = contour_polylines(&mask, w, w, 0.5)
        .into_iter()
        .filter(|pl| pl.len() > 3 && pl[0] == pl[pl.len() - 1])
        .map(|pl| pl[..pl.len() - 1].to_vec())
        .filter(|r| inside(r, [p.0, p.1]))
        .min_by_key(|r| r.len())?;
    Some((ring.into_iter().map(|(x, y)| [(x + x0 as f64) * kpc, (y + y0 as f64) * kpc]).collect(), count))
}

/// Even-odd point in polygon for a fractional ring.
fn inside(ring: &[(f64, f64)], p: [f64; 2]) -> bool {
    let mut c = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[j]);
        if (a.1 > p[1]) != (b.1 > p[1]) && p[0] < a.0 + (p[1] - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            c = !c;
        }
        j = i;
    }
    c
}

/// Signed shoelace area of a ring, km^2 (a diagnostic the bridge reports).
pub fn ring_area_km2(r: &[[f64; 2]]) -> f64 {
    let n = r.len();
    if n < 3 {
        return 0.0;
    }
    ((0..n).map(|i| r[i][0] * r[(i + 1) % n][1] - r[(i + 1) % n][0] * r[i][1]).sum::<f64>() / 2.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bowl: floor 0.2 at the centre rising to a flat rim of 0.6 (the field's own maximum, so a wall, as a
    /// summit rim clipped at the ceiling is, `pour_point`) 3 cells wide from radius `r`, and falling away to 0.3 outside, on a 120 x 100 grid. `breach`, when set, cuts a 5-cell
    /// channel from the floor out to the +x edge at the floor's own height, so
    /// the flood has no pour point and reaches the map edge.
    fn bowl(r: f64, breach: bool) -> (Vec<f32>, usize, usize, usize, usize) {
        let (gw, gh) = (120usize, 100usize);
        let (cx, cy) = (60usize, 50usize);
        let mut f = vec![0.3f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let d = (x as f64 - cx as f64).hypot(y as f64 - cy as f64);
                f[y * gw + x] = if d < r { (0.2 + 0.4 * (d / r)) as f32 } else if d < r + 3.0 { 0.6 } else { (0.6 - 0.3 * ((d - r - 3.0) / 12.0).min(1.0)) as f32 };
            }
        }
        if breach {
            for y in cy - 2..=cy + 2 {
                for x in cx..gw {
                    f[y * gw + x] = f[y * gw + x].min(0.2);
                }
            }
        }
        (f, gw, gh, cx, cy)
    }

    /// Protects: a closed bowl is outlined by the pour-point mode, as a ring
    /// round the click whose equivalent radius is the rim's (to within one
    /// cell), in km (so `kpc` scales it), and the result is stable.
    #[test]
    fn a_closed_bowl_is_outlined_at_its_pour_point() {
        let (f, gw, gh, cx, cy) = bowl(12.0, false);
        let kpc = 0.5;
        let a = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams::default()).unwrap();
        assert_eq!(a.mode, FillMode::PourPoint);
        assert!(a.cells >= MIN_CLOSED_CELLS);
        let eq_r_cells = (ring_area_km2(&a.ring_km) / (kpc * kpc) / std::f64::consts::PI).sqrt();
        assert!((eq_r_cells - 12.0).abs() < 1.5, "equivalent radius {eq_r_cells} cells");
        // Around the click: centroid within a cell of it, in km.
        let n = a.ring_km.len() as f64;
        let (mx, my) = (a.ring_km.iter().map(|p| p[0]).sum::<f64>() / n, a.ring_km.iter().map(|p| p[1]).sum::<f64>() / n);
        assert!((mx / kpc - (cx as f64 + 0.5)).abs() < 1.0 && (my / kpc - (cy as f64 + 0.5)).abs() < 1.0, "centroid ({mx}, {my})");
        let b = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams::default()).unwrap();
        assert_eq!(a.ring_km, b.ring_km, "deterministic");
    }

    /// Protects: a breached bowl falls through to the capped fill, whose ring
    /// is bounded by the automatic cap (1.3 x the rim distance) and encloses
    /// the click; and an explicit smaller cap makes a smaller ring.
    #[test]
    fn a_breached_bowl_takes_the_capped_fill_and_the_cap_bounds_it() {
        let (f, gw, gh, cx, cy) = bowl(12.0, true);
        let kpc = 0.5;
        let a = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams::default()).unwrap();
        assert_eq!(a.mode, FillMode::Capped);
        let cap = a.cap_cells.unwrap();
        assert!((cap - a.auto_cap_cells).abs() < 1e-9);
        let area_auto = ring_area_km2(&a.ring_km) / (kpc * kpc);
        assert!(area_auto <= std::f64::consts::PI * (cap + 1.0).powi(2), "{area_auto} cells^2 within the cap {cap}");
        assert!(area_auto > 300.0, "a bowl of radius 12 is about 450 cells^2: {area_auto}");
        let small = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams { level_delta: 0.0, cap_cells: Some(6.0) }).unwrap();
        let area_small = ring_area_km2(&small.ring_km) / (kpc * kpc);
        assert!(area_small < area_auto * 0.6, "cap 6 gives {area_small}, auto {area_auto}");
        assert_eq!(small.cap_cells, Some(6.0));
    }

    /// Protects: the capped fill's automatic level sits just UNDER the rim (`RIM_MARGIN`), so a shelf a hair below the
    /// rim does not float the fill out of the bowl. A breached bowl whose rim crest (0.60) is ringed by a shelf at 0.595
    /// (0.005 under it): with the margin the level is about 0.59 and the fill stays inside the bowl; with no margin it would
    /// be about 0.598 and flood the shelf out to the cap, more than a third as large again. Mutating the margin away must
    /// fail this.
    #[test]
    fn the_automatic_level_stays_under_the_rim_so_a_shelf_is_not_flooded() {
        let (mut f, gw, gh, cx, cy) = bowl(12.0, true);
        for y in 0..gh {
            for x in 0..gw {
                let d = (x as f64 - cx as f64).hypot(y as f64 - cy as f64);
                if (12.0..22.0).contains(&d) && !(y.abs_diff(cy) <= 2 && x >= cx) {
                    f[y * gw + x] = 0.595;
                }
            }
        }
        let a = fill_from_click(&f, gw, gh, cx, cy, 1.0, FillParams::default()).unwrap();
        assert_eq!(a.mode, FillMode::Capped);
        assert!(a.auto_level > 0.5, "a level near the rim, not the floor: {}", a.auto_level);
        assert!(a.auto_level < 0.595, "the automatic level {} must be under the shelf", a.auto_level);
        let area = ring_area_km2(&a.ring_km);
        assert!(area < 520.0, "the fill stays in the bowl: {area} cells^2 (a flooded shelf is about 760)");
    }

    /// Protects: the level moves the ring. Raising it to the rim's own height
    /// widens a capped fill; lowering it past the floor leaves no ring, which
    /// is an error and never an empty ring.
    #[test]
    fn the_level_moves_the_ring_and_a_level_below_the_floor_is_an_error() {
        let (f, gw, gh, cx, cy) = bowl(12.0, true);
        let kpc = 1.0;
        let base = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams::default()).unwrap();
        let low = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams { level_delta: -0.2, cap_cells: None }).unwrap();
        assert!(ring_area_km2(&low.ring_km) < ring_area_km2(&base.ring_km) * 0.5, "a lower level fills less");
        let err = fill_from_click(&f, gw, gh, cx, cy, kpc, FillParams { level_delta: -0.5, cap_cells: None }).unwrap_err();
        assert_eq!(err, FillError::NoRingAtLevel);
    }

    /// Protects: every refusal is its own error, never a panic or a wrapped
    /// index: flat ground is not a depression, the edge, a bad cap, a bad
    /// level, outside the grid, a short field.
    #[test]
    fn refusals_are_errors_not_panics() {
        let (gw, gh) = (120usize, 100usize);
        let flat = vec![0.5f32; gw * gh];
        assert_eq!(fill_from_click(&flat, gw, gh, 60, 50, 1.0, FillParams::default()).unwrap_err(), FillError::NotInDepression);
        let (f, gw, gh, _, _) = bowl(12.0, false);
        assert_eq!(fill_from_click(&f, gw, gh, 3, 50, 1.0, FillParams::default()).unwrap_err(), FillError::TooNearEdge);
        assert_eq!(fill_from_click(&f, gw, gh, 500, 50, 1.0, FillParams::default()).unwrap_err(), FillError::OutsideGrid);
        assert_eq!(fill_from_click(&f, gw, gh, 60, 50, 1.0, FillParams { level_delta: f32::NAN, cap_cells: None }).unwrap_err(), FillError::BadLevel);
        for bad in [0.0, -1.0, f64::NAN, MAX_CAP_CELLS + 1.0] {
            assert_eq!(fill_from_click(&f, gw, gh, 60, 50, 1.0, FillParams { level_delta: 0.0, cap_cells: Some(bad) }).unwrap_err(), FillError::BadCap, "cap {bad}");
        }
        assert_eq!(fill_from_click(&f[..100], gw, gh, 60, 50, 1.0, FillParams::default()).unwrap_err(), FillError::BadField);
        assert_eq!(fill_from_click(&f, gw, gh, 60, 50, 0.0, FillParams::default()).unwrap_err(), FillError::BadField);
        // A cap whose window leaves the grid: the bowl sits 50 cells from the
        // top edge, so cap 60 (window 63 cells) leaves it.
        let (fb, gw, gh, cx, cy) = bowl(12.0, true);
        assert_eq!(fill_from_click(&fb, gw, gh, cx, cy, 1.0, FillParams { level_delta: 0.0, cap_cells: Some(55.0) }).unwrap_err(), FillError::TooNearEdge);
    }
}
