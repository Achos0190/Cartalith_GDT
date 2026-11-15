//! A town wall that hugs the built-up area -- **not a reference port.** Owner
//! Ruling EW item 1 (2026-11-13, `LARGE_ITEM_RULINGS.md`): *"walls were
//! expensive so a lot of free space between the wall and the first city on the
//! inside is unrealistic"*; recorded under `DECISIONS.md` section 7p.
//!
//! # Why the reference's wall stands off, measured
//!
//! Two causes, found by drawing the finished towns (`CITY_MAP_LOOK_SCOPE.md`'s
//! 2026-11-13 wall-hug note has the frames) and by measuring them
//! ([`crate::wallgap`]):
//!
//! 1. [`crate::fortify::built_mass_hull`] puts the curtain at the hull of the
//!    built *street nodes*, inflated 10 % about its centroid and then 16 m (a
//!    deliberate growth reserve, the reference's M-NET-9), at epoch
//!    `max(3, floor(0.6 * epochs))`, before the last epochs of street growth.
//! 2. More important: the street network is a *growth tree*, not a closed
//!    mesh. Its outermost streets are dead-end lanes and open branches that
//!    close no face, so [`crate::blocks::build_blocks`] cuts no block and
//!    `build_parcels` plats no lot there: the ground between the last platted
//!    lots and a wall that is drawn round the street tips is **open, with
//!    streets running through it and nothing built beside them**. A wall
//!    refitted to the street nodes alone would still stand 50-100 m from the
//!    first house on those sides, so the hull must be fitted to the *lots*.
//!
//! # What this does instead
//!
//! After the lots are platted and **before** the wall lots, courtyards,
//! districts, buildings, farmland and the citadel (`generate()`, straight after
//! `build_parcels`), the wall is **refitted to the platted town**, by the
//! reference's own circuit code ([`build_wall_from_mass`]) fed a different hull:
//!
//! 1. The built mass is every lot the building stage will build on (frontage
//!    and depth at `build_buildings`' own floors) whose centroid is inside the
//!    current circuit, plus the street edges those lots front.
//! 2. Its outline is the star-shaped envelope of those points ([`envelope`]):
//!    one radius per direction about the mass's centre, so the wall follows a
//!    bay or notch in the town and cannot fold over itself, grown by
//!    [`CLEARANCE_M`]. **On a riverbank site** (`urban_riverbank`, 2026-11-15)
//!    it is [`outline`] instead, the built area with gaps under twice
//!    [`CLOSE_R_M`] closed over, because a star walls the open ground inside
//!    a U- or L-shaped town; a circuit across the river keeps that shape
//!    rather than the reference's convex hull, the market square counts as
//!    mass, and a circuit that cannot be drawn one way (across the river, or
//!    round the market's bank alone) is tried the other way before the
//!    original is kept.
//! 3. The reference's smoothing (two Chaikin passes, a corner cut, a 2 m
//!    simplify) rounds corners **inward**, so the result is checked: every mass
//!    point must stand at least [`MIN_MARGIN_M`] inside the ring. A shortfall
//!    widens the clearance by that shortfall and the circuit is built again (at
//!    most [`MAX_REFITS`] times); if that still fails, or the result is not
//!    smaller than the circuit it replaces, **the original circuit is kept**
//!    ([`HugOutcome::Kept`]) -- never a wall that cuts through a lot.
//! 4. Street edges that ran only into the freed band -- a dead-end lane or open
//!    branch that fronts no lot and now leaves the new ring -- are removed, so
//!    no street crosses the wall except at a gate. Primaries are never removed:
//!    they cross the ring and **gates are re-derived by the same code from the
//!    primaries that cross the new ring**, so every gate still stands on a
//!    street. Edges wholly outside the old circuit (ribbon suburbs) are not
//!    touched.
//! 5. Wall lots, the faubourg, farmland, details and the citadel are built
//!    afterwards from the refitted ring, so they follow it: the citadel stays
//!    astride the curtain, and fields start at the new wall.
//!
//! No lot is added or removed by this stage and the street-platted buildings
//! are the same ones, so the population is the same; what moves is the
//! circuit, the wall lots platted from it, and the stray streets. Terrain
//! deflection of the hull (up to 60 m onto higher ground) is switched off for
//! the refit, because a 60 m outward shift is the band being removed. A
//! bastioned trace is left alone: its bastions are sized to their own hull and
//! the ring is the gorge polygon through their throats.

/// Unit and property tests; see its own module doc.
#[cfg(test)]
mod tests;

use crate::blocks::Parcel;
use crate::cleanup::kill_edge;
use crate::districts::TERRAIN_BUILD_MIN;
use crate::fortify::{BuiltMass, FortOpts, build_wall_from_mass};
use crate::geom::{Vec2, convex_hull, dist_pt_seg, point_in_poly, poly_area, seg_int, simplify, poly_centroid};
use crate::graph::Graph;
use crate::growth::{HarbourFront, WallState, ring_crossings};
use crate::routes::Anchors;
use crate::districts::lot_suitability;
use crate::site::Site;
use std::f64::consts::PI;

/// Distance from the lots' outer edge to the wall, metres: the room for a wall
/// street (the Roman *pomerium*, the medieval *intervallum*: the lane kept
/// clear along the inner face for the garrison and the wall-walk's stairs) and
/// the rampart's footing. **A labelled judgement**, not a citation of a width:
/// it is about one street width plus the footing (the engine's streets are
/// 3.4-5.4 m wide, `grow`), the "few metres" the owner asked for, and under
/// the 15 m rampart strip the reference keeps clear on the *outside*
/// (`clear_fort_zone`).
pub const CLEARANCE_M: f64 = 8.0;
/// The least distance a mass point may stand inside the finished ring, metres.
/// A labelled judgement: over one street half-width (about 2.7 m) so a lot's
/// frontage street is never under the curtain, and below [`CLEARANCE_M`] so the
/// smoothing's inward corner rounding costs one retry at most.
pub const MIN_MARGIN_M: f64 = 5.0;
/// How many times the circuit is rebuilt wider after a margin shortfall. A
/// labelled judgement: each retry adds the measured shortfall, so one or two
/// settle it; four is a ceiling, not an expectation.
pub const MAX_REFITS: usize = 4;
/// The fewest buildable lots a town needs for a refit. A labelled judgement:
/// eight lots is two short rows, the same "a hamlet declines a wall" scale as
/// `built_mass_hull`'s eight nodes.
pub const MIN_LOTS: usize = 8;
/// Directions of the star-shaped envelope ([`envelope`]). 72 is one every 5
/// degrees: a 300 m town gets a vertex every 26 m, finer than a block, coarser
/// than a building. A labelled judgement.
const SECTORS: usize = 180;
/// A sector also takes the farthest point of its `WINDOW` neighbours on each
/// side, so the outline cannot slip between two directions to a lot lying
/// across them. 2 sectors is 10 degrees, past the half-sector a chord between
/// two vertices can lose. A labelled judgement.
const WINDOW: usize = 3;
/// Spacing of the samples along a street, metres: under the narrowest lot
/// frontage the platting allows (`build_parcels`' 4.5 m floor, doubled), so a
/// fronted street cannot slip between two samples. A labelled judgement.
const EDGE_SAMPLE_M: f64 = 10.0;
/// A street counts as fronting a lot when a sample on it is within this many
/// metres of one of the lot's corners, metres. A lot's frontage corners lie on
/// the street line and lots are 4.5-15 m wide, so a fronted street always has a
/// corner within half a lot width; 12 m is that plus slack. A labelled judgement.
const FRONT_REACH_M: f64 = 12.0;
/// The least share of the near bank's lots the far bank must hold for a
/// both-banks town's refitted circuit to span the river (Ruling FB). **The
/// reference's** own far-bank share, `builtMassHull`'s `max(20, 32 %)` of the
/// near bank's street nodes, applied to lots instead of nodes.
pub const FAR_BANK_LOT_SHARE: f64 = 0.32;
/// How close to the water side of the ring a point must be to be left out of the
/// margin test, metres ([`least_margin`]). A labelled judgement: the bank wall
/// is `town_bank`'s 5 m off the shore line, and a shore street's samples fall
/// within about half that of it.
const SHORE_TOL_M: f64 = 3.0;
/// How much nearer the ring than the land arc a point must be to count as
/// "nearest the water side", metres. A labelled judgement: a half metre is
/// beyond the 0.05 m float noise of two distances to one ring.
const SHORE_SIDE_M: f64 = 0.5;
/// A circuit must be at most this fraction of the area of the one it replaces to
/// be taken. A labelled judgement: it only rejects a refit that changed nothing
/// worth the churn in gates and downstream stages.
const MAX_AREA_RATIO: f64 = 0.995;

/// What [`hug_wall`] did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HugOutcome {
    /// The circuit was rebuilt. `clearance_m` is the offset the accepted build
    /// used (at least [`CLEARANCE_M`]); `area_ratio` is new area over old;
    /// `streets_removed` counts the stray street edges taken out.
    Refit { clearance_m: f64, area_ratio: f64, streets_removed: usize },
    /// The original circuit stands, and why.
    Kept(KeptReason),
}

/// Why [`hug_wall`] left the circuit alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeptReason {
    /// The town has no circuit.
    NoWall,
    /// A bastioned trace is its own geometry.
    Bastioned,
    /// Fewer than [`MIN_LOTS`] buildable lots inside the circuit.
    TooFewLots,
    /// The circuit builder refused the hull (under six points, or no land).
    Refused,
    /// A mass point still stood within [`MIN_MARGIN_M`] of the wall (or
    /// outside it) after [`MAX_REFITS`] widenings.
    MarginNotMet,
    /// The refitted ring was not smaller than the one it replaces.
    NotSmaller,
}

/// Ruling FB's span test on a both-banks town's lots (see [`hug_wall`]):
/// `sides` is each buildable lot's bank (`Site::bank_side`, +1 or -1) and
/// `mkt_side` the market's. `Some(mkt_side)` when the other bank holds too few
/// lots to be walled (under [`MIN_LOTS`], or under [`FAR_BANK_LOT_SHARE`] of
/// the market bank's), so the circuit is refitted round the market's bank
/// alone; `None` when it should span the river.
///
/// Reworked 2026-11-15 (the coordinator's defect (a)): the first version
/// answered the *larger* bank, and refitted the circuit round the far bank
/// when it held more lots, leaving the market and its square outside the
/// wall. A far bank larger than the market's is now walled with it.
/// Must never answer the bank the market is not on.
pub fn one_bank_by_lots(sides: &[f64], mkt_side: f64) -> Option<f64> {
    let near = sides.iter().filter(|&&s| s == mkt_side).count();
    let far = sides.len() - near;
    if (far as f64) < (MIN_LOTS as f64).max(near as f64 * FAR_BANK_LOT_SHARE) {
        return Some(mkt_side);
    }
    None
}

/// Samples of one street edge: both ends and a point every [`EDGE_SAMPLE_M`].
fn edge_samples(a: Vec2, b: Vec2) -> Vec<Vec2> {
    let n = ((a.dist(b) / EDGE_SAMPLE_M).ceil() as usize).max(1);
    (0..=n).map(|k| a + (b - a) * (k as f64 / n as f64)).collect()
}

/// The star-shaped outline of `pts` about `c0`, grown by `pad`: for each of
/// [`SECTORS`] directions the radius of the farthest point within [`WINDOW`]
/// sectors of it, plus `pad`. Unlike a convex hull it follows a bay or a notch
/// in the town; unlike a free-form concave hull it cannot fold over itself or
/// spike, because it is one radius per direction. A direction with no point in
/// its window takes the linear interpolation of the nearest directions that
/// have one (a bank, a gap in the town), so no sector is left at radius 0.
/// Empty for no points.
pub fn envelope(c0: Vec2, pts: &[Vec2], pad: f64) -> Vec<Vec2> {
    let mut far = vec![f64::NAN; SECTORS];
    for p in pts {
        let (dx, dy) = (p.x - c0.x, p.y - c0.y);
        let r = dx.hypot(dy);
        let s = ((dy.atan2(dx) / (2.0 * PI)).rem_euclid(1.0) * SECTORS as f64).floor() as usize % SECTORS;
        for w in 0..=2 * WINDOW {
            let k = (s + SECTORS + w - WINDOW) % SECTORS;
            if far[k].is_nan() || r > far[k] {
                far[k] = r;
            }
        }
    }
    if far.iter().all(|r| r.is_nan()) {
        return Vec::new();
    }
    // Fill any empty sector from its nearest filled neighbours either side.
    let filled = far.clone();
    for k in 0..SECTORS {
        if filled[k].is_nan() {
            let (mut dp, mut dn) = (1, 1);
            while filled[(k + SECTORS - dp) % SECTORS].is_nan() {
                dp += 1;
            }
            while filled[(k + dn) % SECTORS].is_nan() {
                dn += 1;
            }
            let (rp, rn) = (filled[(k + SECTORS - dp) % SECTORS], filled[(k + dn) % SECTORS]);
            far[k] = (rp * dn as f64 + rn * dp as f64) / (dp + dn) as f64;
        }
    }
    (0..SECTORS)
        .map(|k| {
            let a = (k as f64 + 0.5) * 2.0 * PI / SECTORS as f64;
            let r = far[k] + pad;
            Vec2::new(c0.x + r * a.cos(), c0.y + r * a.sin())
        })
        .collect()
}

/// Grid step of [`outline`], metres. A labelled judgement: under the
/// narrowest lot frontage the platting allows (`build_parcels`' 4.5 m floor),
/// so no lot slips between cells, and coarse enough that a 1.5 km town is a
/// few hundred thousand cells.
pub const OUTLINE_CELL_M: f64 = 4.0;
/// The gap [`outline`] bridges, as a radius, metres: any opening in the built
/// area narrower than twice this is closed over (a street, a churchyard, a
/// back-garden row), any wider one is left out of the wall. **A labelled
/// judgement**, matched to the audit's unbuilt bar
/// (`town_river_census.rs::UNBUILT_REACH_M`, 40 m: a deep burgage plot's
/// yard), so the wall encloses no ground farther than a plot from a lot.
pub const CLOSE_R_M: f64 = 40.0;

/// How near the river's bank a mass point may stand and owe the wall no
/// clearance, metres ([`hug_wall`], riverbank sites). **A labelled
/// judgement**: two of the riverbank street widths the census uses (5.4 m,
/// `town_river_census.rs::BANK_STREET_W`) rounded down, the depth of the
/// riverside frontage row that fronts the water itself.
pub const BANK_EXEMPT_M: f64 = 10.0;

/// The most of a town's occupied cells [`outline`] may leave outside its loop
/// in detached pieces, as a share. **A labelled judgement**: a stray lot or
/// two (seed 483920 #0 at 2 048 x 1 311: 8 cells against 8 836, under 0.1 %)
/// is a suburb; a twentieth of the town is a quarter, and the loop is not
/// drawn.
pub const OUTLINE_MINOR_SHARE: f64 = 0.05;

/// The opening [`outline`] applies to its shape, in cells: two cells (8 m) at
/// [`OUTLINE_CELL_M`]. **A labelled judgement**: the least that removes a
/// one-cell neck or spur from the traced loop with a cell to spare, and well
/// under a lot depth, so no lot's outline is cut.
pub const OUTLINE_OPEN_CELLS: u32 = 2;

/// **Not the reference** (2026-11-15, the coordinator's defect (b) on seed
/// 12345: a refitted wall round a large unbuilt patch between two lobes of a
/// river town). The outline of the built mass grown by `pad`, with every gap
/// narrower than `2 * CLOSE_R_M` closed over and every wider one left out: a
/// morphological closing on an [`OUTLINE_CELL_M`] grid (lots filled, street
/// samples as points; dilate by [`CLOSE_R_M`], fill the holes, erode back to
/// `pad`), traced as one loop (Moore neighbours) and simplified to 2 m. With
/// the shoelace orientation of [`envelope`]'s output.
///
/// **Why not [`envelope`]**: a star about the mass's centre keeps the
/// farthest point in each direction, so a town shaped like a U or an L walls
/// the open ground inside its bend; a closing follows the bend. **Why not a
/// convex hull** (the reference's spanning circuit): it walls every bay.
///
/// Returns the loop and the centres of the occupied cells it leaves out: the
/// minor pieces when the closed mass falls apart (a stray lot or two more
/// than `2 * CLOSE_R_M` from the town, together at most
/// [`OUTLINE_MINOR_SHARE`] of its occupied cells), which stay outside the
/// wall as a suburb does. `None` when the mass is empty, when `pad` is not
/// under [`CLOSE_R_M`], when the town falls apart into quarters larger than
/// that (one loop cannot enclose both without walling the gap), or when
/// `keep` (the market square's centre) is not in the loop's piece. Must never
/// return a loop that leaves out `keep` or a lot of the largest piece.
pub fn outline(polys: &[&[Vec2]], pts: &[Vec2], pad: f64, keep: Option<Vec2>) -> Option<(Vec<Vec2>, Vec<Vec2>)> {
    if pad >= CLOSE_R_M || (polys.is_empty() && pts.is_empty()) {
        return None;
    }
    let all = polys.iter().flat_map(|p| p.iter()).chain(pts.iter());
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in all {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    let m = CLOSE_R_M + 3.0 * OUTLINE_CELL_M;
    let (x0, y0) = (x0 - m, y0 - m);
    let nx = ((x1 + m - x0) / OUTLINE_CELL_M).ceil() as usize + 1;
    let ny = ((y1 + m - y0) / OUTLINE_CELL_M).ceil() as usize + 1;
    let ctr = |i: usize, j: usize| Vec2::new(x0 + (i as f64 + 0.5) * OUTLINE_CELL_M, y0 + (j as f64 + 0.5) * OUTLINE_CELL_M);
    let cell = |p: Vec2| -> (usize, usize) {
        let i = ((p.x - x0) / OUTLINE_CELL_M).floor().max(0.0) as usize;
        let j = ((p.y - y0) / OUTLINE_CELL_M).floor().max(0.0) as usize;
        (i.min(nx - 1), j.min(ny - 1))
    };
    // The occupied cells: every lot (its cells and its corners) and every point.
    let mut occ = vec![false; nx * ny];
    for poly in polys.iter().filter(|p| p.len() >= 3) {
        let (mut a, mut b) = (cell(poly[0]), cell(poly[0]));
        for q in poly.iter() {
            let (i, j) = cell(*q);
            occ[j * nx + i] = true;
            a = (a.0.min(i), a.1.min(j));
            b = (b.0.max(i), b.1.max(j));
        }
        for j in a.1..=b.1 {
            for i in a.0..=b.0 {
                if point_in_poly(ctr(i, j), poly) {
                    occ[j * nx + i] = true;
                }
            }
        }
    }
    for q in pts {
        let (i, j) = cell(*q);
        occ[j * nx + i] = true;
    }
    // A two-pass 3-4 chamfer distance from `seed` cells, in thirds of a cell
    // (within about 8 % of Euclidean, ample at a 40 m radius).
    let chamfer = |seed: &dyn Fn(usize) -> bool| -> Vec<u32> {
        const INF: u32 = u32::MAX / 2;
        let mut d: Vec<u32> = (0..nx * ny).map(|k| if seed(k) { 0 } else { INF }).collect();
        for j in 0..ny {
            for i in 0..nx {
                let k = j * nx + i;
                let mut v = d[k];
                if i > 0 { v = v.min(d[k - 1] + 3) }
                if j > 0 {
                    v = v.min(d[k - nx] + 3);
                    if i > 0 { v = v.min(d[k - nx - 1] + 4) }
                    if i + 1 < nx { v = v.min(d[k - nx + 1] + 4) }
                }
                d[k] = v;
            }
        }
        for j in (0..ny).rev() {
            for i in (0..nx).rev() {
                let k = j * nx + i;
                let mut v = d[k];
                if i + 1 < nx { v = v.min(d[k + 1] + 3) }
                if j + 1 < ny {
                    v = v.min(d[k + nx] + 3);
                    if i + 1 < nx { v = v.min(d[k + nx + 1] + 4) }
                    if i > 0 { v = v.min(d[k + nx - 1] + 4) }
                }
                d[k] = v;
            }
        }
        d
    };
    let units = |metres: f64| (metres / OUTLINE_CELL_M * 3.0).round() as u32;
    let d_occ = chamfer(&|k| occ[k]);
    let dilated: Vec<bool> = d_occ.iter().map(|&v| v <= units(CLOSE_R_M)).collect();
    // Fill the holes: outside is what the box border reaches through open cells.
    let mut outside = vec![false; nx * ny];
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..nx {
        stack.push(i);
        stack.push((ny - 1) * nx + i);
    }
    for j in 0..ny {
        stack.push(j * nx);
        stack.push(j * nx + nx - 1);
    }
    while let Some(k) = stack.pop() {
        if outside[k] || dilated[k] {
            continue;
        }
        outside[k] = true;
        let (i, j) = (k % nx, k / nx);
        if i > 0 { stack.push(k - 1) }
        if i + 1 < nx { stack.push(k + 1) }
        if j > 0 { stack.push(k - nx) }
        if j + 1 < ny { stack.push(k + nx) }
    }
    // Erode back to `pad` beyond the mass.
    let d_out = chamfer(&|k| outside[k]);
    let keep_u = units(CLOSE_R_M - pad);
    let eroded: Vec<bool> = (0..nx * ny).map(|k| !outside[k] && d_out[k] > keep_u).collect();
    // An opening by [`OUTLINE_OPEN_CELLS`]: no neck or spur thinner than
    // that survives, so the traced loop never runs down one side of a
    // one-cell corridor and back up the other (a loop touching itself, which
    // the circuit's smoothing turns into a ring that crosses itself).
    let open_u = 3 * OUTLINE_OPEN_CELLS;
    let d_in = chamfer(&|k| !eroded[k]);
    let core: Vec<bool> = (0..nx * ny).map(|k| d_in[k] > open_u).collect();
    let d_core = chamfer(&|k| core[k]);
    let shape: Vec<bool> = (0..nx * ny).map(|k| d_core[k] <= open_u).collect();
    // Its 4-connected pieces; the one with the most occupied cells is the town.
    let mut comp = vec![usize::MAX; nx * ny];
    let mut occ_in: Vec<usize> = Vec::new();
    for k0 in 0..nx * ny {
        if !shape[k0] || comp[k0] != usize::MAX {
            continue;
        }
        let id = occ_in.len();
        occ_in.push(0);
        let mut st = vec![k0];
        comp[k0] = id;
        while let Some(k) = st.pop() {
            occ_in[id] += usize::from(occ[k]);
            let (i, j) = (k % nx, k / nx);
            let mut push = |kk: usize| {
                if shape[kk] && comp[kk] == usize::MAX {
                    comp[kk] = id;
                    st.push(kk);
                }
            };
            if i > 0 { push(k - 1) }
            if i + 1 < nx { push(k + 1) }
            if j > 0 { push(k - nx) }
            if j + 1 < ny { push(k + nx) }
        }
    }
    let best = (0..occ_in.len()).max_by_key(|&c| (occ_in[c], usize::MAX - c))?;
    // `keep` (the market square) must be in the loop, never a dropped piece.
    if let Some(k) = keep {
        let (i, j) = cell(k);
        if comp[j * nx + i] != best {
            return None;
        }
    }
    let total: usize = occ_in.iter().sum();
    if (total - occ_in[best]) as f64 > OUTLINE_MINOR_SHARE * total as f64 {
        return None;
    }
    // The occupied cells of the minor pieces, left outside the loop.
    let dropped: Vec<Vec2> = (0..nx * ny)
        .filter(|&k| occ[k] && shape[k] && comp[k] != best)
        .map(|k| ctr(k % nx, k / nx))
        .collect();
    let inside = |i: i64, j: i64| i >= 0 && j >= 0 && (i as usize) < nx && (j as usize) < ny && comp[j as usize * nx + i as usize] == best;
    // Moore-neighbour trace of the piece's outer boundary, from its first
    // cell in raster order (its west neighbour is outside), clockwise on the
    // screen (y down), stopping on Jacob's criterion.
    let start = (0..nx * ny).find(|&k| comp[k] == best)?;
    let (si, sj) = ((start % nx) as i64, (start / nx) as i64);
    const DIRS: [(i64, i64); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
    let mut loop_cells: Vec<(i64, i64)> = vec![(si, sj)];
    let (mut ci, mut cj, mut back) = (si, sj, 4usize);
    let start_back = back;
    for _ in 0..4 * nx * ny {
        let mut next = None;
        for t in 1..=8 {
            let k = (back + t) % 8;
            let (ni, nj) = (ci + DIRS[k].0, cj + DIRS[k].1);
            if inside(ni, nj) {
                // The cell checked just before, seen from the new cell.
                let (pi, pj) = (ci + DIRS[(k + 7) % 8].0, cj + DIRS[(k + 7) % 8].1);
                let nb = DIRS.iter().position(|&dd| dd == (pi - ni, pj - nj)).unwrap_or(4);
                next = Some((ni, nj, nb));
                break;
            }
        }
        let Some((ni, nj, nb)) = next else { break };
        if (ni, nj) == (si, sj) && nb == start_back {
            break;
        }
        ci = ni;
        cj = nj;
        back = nb;
        if (ci, cj) == (si, sj) {
            continue;
        }
        loop_cells.push((ci, cj));
    }
    if loop_cells.len() < 6 {
        return None;
    }
    let mut ring: Vec<Vec2> = loop_cells.iter().map(|&(i, j)| ctr(i as usize, j as usize)).collect();
    ring = simplify(&ring, 2.0);
    if ring.len() < 6 {
        return None;
    }
    if poly_area(&ring) < 0.0 {
        ring.reverse();
    }
    Some((ring, dropped))
}

/// The first proper crossing of the closed loop `ring` with itself: edges
/// `a < b`, not adjacent, whose interiors cross (a touch at an end, as at a
/// repeated vertex, is not a crossing), and the crossing point.
fn first_crossing(ring: &[Vec2]) -> Option<(usize, usize, Vec2)> {
    let n = ring.len();
    for a in 0..n {
        for b in (a + 2)..n {
            if a == 0 && b == n - 1 {
                continue;
            }
            if let Some(h) = seg_int(ring[a], ring[(a + 1) % n], ring[b], ring[(b + 1) % n])
                && h.t > 1e-6
                && h.t < 1.0 - 1e-6
                && h.u > 1e-6
                && h.u < 1.0 - 1e-6
            {
                return Some((a, b, h.pt));
            }
        }
    }
    None
}

/// Whether the closed loop `ring` crosses itself ([`first_crossing`]).
pub fn ring_self_crosses(ring: &[Vec2]) -> bool {
    first_crossing(ring).is_some()
}

/// The most of a ring's area one loop [`untangle`] may cut off, as a share.
/// **A labelled judgement**: the crossings the circuit builder makes where a
/// land arc meets its bank walk are slivers of a few square metres (seen on
/// seed 12345 at 384 x 246, 2026-11-15); a loop of 2 % of a town's area is a
/// fold, not a sliver, and the refit is refused instead.
pub const UNTANGLE_MAX_SHARE: f64 = 0.02;

/// `ring` with each small self-crossing loop cut off at the crossing: the
/// shorter side of the loop (by vertex count) is replaced by the crossing
/// point, until none is left. `None` when a loop to cut is over
/// [`UNTANGLE_MAX_SHARE`] of the ring's area, or after `ring.len()` cuts.
/// Used by [`hug_wall`] on a riverbank site (2026-11-15). Must never cut a
/// loop larger than that share, or return a ring that still crosses itself.
pub fn untangle(ring: &[Vec2]) -> Option<Vec<Vec2>> {
    let mut r = ring.to_vec();
    let area = poly_area(ring).abs();
    for _ in 0..ring.len() {
        let Some((a, b, x)) = first_crossing(&r) else { return Some(r) };
        // Inner loop: vertices a+1..=b; outer: the rest.
        let inner: Vec<Vec2> = std::iter::once(x).chain(r[a + 1..=b].iter().copied()).collect();
        let outer: Vec<Vec2> = std::iter::once(x).chain(r[b + 1..].iter().copied()).chain(r[..=a].iter().copied()).collect();
        let (cut, keep) = if inner.len() <= outer.len() { (inner, outer) } else { (outer, inner) };
        if poly_area(&cut).abs() > UNTANGLE_MAX_SHARE * area {
            return None;
        }
        r = keep;
        if r.len() < 6 {
            return None;
        }
    }
    None
}

/// How far inside `ring` the least-enclosed point of `pts` stands, metres:
/// negative when one is outside it. `f64::INFINITY` for no points.
///
/// `arc` is the circuit's drawn land-facing arc ([`WallState::land_arc`], an
/// open polyline). When given, a point **inside** the ring is measured against
/// the arc only: the ring also runs along a shore, where a street may stand at
/// the water's edge, and no clearance is owed to the water. A point **outside**
/// the ring is always measured against the whole ring (a shore street outside
/// it is a real failure).
pub fn least_margin(pts: &[Vec2], ring: &[Vec2], arc: Option<&[Vec2]>) -> f64 {
    let dist_to = |p: Vec2, line: &[Vec2], closed: bool| -> f64 {
        let n = if closed { line.len() } else { line.len().saturating_sub(1) };
        (0..n).map(|i| dist_pt_seg(p, line[i], line[(i + 1) % line.len()])).fold(f64::INFINITY, f64::min)
    };
    let mut worst = f64::INFINITY;
    for p in pts {
        // A street laid along a shore stands within a metre or two of the water
        // wall, which follows the bank 5 m off the shore line (`town_bank`), and
        // may fall either side of it. No clearance is owed there, so a point
        // whose nearest boundary is the water side, and within `SHORE_TOL_M` of
        // it, is not measured.
        if let Some(a) = arc.filter(|a| a.len() >= 2) {
            let (d_ring, d_arc) = (dist_to(*p, ring, true), dist_to(*p, a, false));
            if d_ring <= SHORE_TOL_M && d_arc - d_ring > SHORE_SIDE_M {
                continue;
            }
        }
        let m = if point_in_poly(*p, ring) {
            match arc {
                Some(a) if a.len() >= 2 => dist_to(*p, a, false),
                _ => dist_to(*p, ring, true),
            }
        } else {
            -dist_to(*p, ring, true)
        };
        worst = worst.min(m);
    }
    worst
}

/// Refit `wall_state`'s circuit to the platted town -- the module doc says how
/// and why. Mutates `wall_state` and `g` (stray street edges) only on
/// [`HugOutcome::Refit`]; every other outcome leaves both exactly as they were.
///
/// The circuit's recorded epoch is the original's, so provenance and wall
/// generations read as they did.
///
/// `both_banks` is `generate`'s Ruling FB eligibility (the span tests run
/// only for such a town); `plaza` is the town's market square (`generate`
/// passes a chartered town's market only, never a village green, which may
/// stand outside its wall as the reference's does), read only on a riverbank
/// site (it holds the wall out and must end inside it). Both are
/// not the reference; with `urban_riverbank` off neither changes anything.
#[allow(clippy::too_many_arguments)]
pub fn hug_wall(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    parcels: &[Parcel],
    terrain_aware: bool,
    wall_state: &mut WallState,
    harbour: Option<&HarbourFront>,
    opts: &FortOpts,
    both_banks: bool,
    plaza: Option<&[Vec2]>,
) -> HugOutcome {
    let Some(old_ring) = wall_state.ring.clone() else {
        return HugOutcome::Kept(KeptReason::NoWall);
    };
    if wall_state.style == "bastioned" {
        return HugOutcome::Kept(KeptReason::Bastioned);
    }

    // The mass: lots `build_buildings` will build on (its own floors, so a lot
    // it would leave empty does not hold the wall out), inside the old circuit.
    let mut mass: Vec<Vec2> = Vec::new();
    // Each counted lot's centroid and footprint, for the far-bank test below.
    let mut lot_polys: Vec<(Vec2, &Vec<Vec2>)> = Vec::new();
    let mut lots = 0usize;
    // **Not the reference** (Ruling FB, riverbank sites only): a both-banks
    // town whose circuit rose round the market's bank alone grows its far
    // bank after the wall (`crate::growth::grow_far_bank`), so those lots are
    // outside the circuit; they are gathered here, within the old circuit's
    // own radius of the bridge's far end, for the span test below.
    let rb = site.riverbank();
    let rk_site = (site.kind == "river" || site.kind == "riverthrough") && site.river.len() >= 2;
    let upgrade = rb && both_banks && rk_site && !wall_state.spans_water;
    let mkt_side = site.bank_side(anchors.market);
    let far_end = if upgrade { crate::riverbank::far_bridge_end(site, anchors.market) } else { None };
    let ring_c = poly_centroid(&old_ring);
    let ring_r = old_ring.iter().map(|q| q.dist(ring_c)).fold(0.0, f64::max);
    let mut far_out: Vec<(Vec2, &Vec<Vec2>)> = Vec::new();
    for p in parcels {
        if p.frontage < 3.2 || p.depth < 6.0 || p.poly.len() < 3 {
            continue;
        }
        let c = poly_centroid(&p.poly);
        if !point_in_poly(c, &old_ring) {
            if let Some(fe) = far_end
                && c.dist(fe) <= ring_r
                && site.bank_side(c) != mkt_side
                && !(terrain_aware && lot_suitability(site, p) < TERRAIN_BUILD_MIN)
            {
                far_out.push((c, &p.poly));
            }
            continue;
        }
        // `terrain_aware` leaves a lot below the suitability bar vacant, so it
        // does not hold the wall out either (steep or flood-prone ground).
        if terrain_aware && lot_suitability(site, p) < TERRAIN_BUILD_MIN {
            continue;
        }
        lots += 1;
        lot_polys.push((c, &p.poly));
        mass.extend(p.poly.iter().copied());
    }
    if lots < MIN_LOTS {
        return HugOutcome::Kept(KeptReason::TooFewLots);
    }
    // **Not the reference** (Ruling FB, owner 2026-11-15: the wall must follow
    // the built-up area and not enclose an empty river). The reference decides
    // that a circuit spans the river from the far bank's *street nodes* when
    // the wall rises (`built_mass_hull`), before the far bank has been platted,
    // and always on a `riverthrough` site; on a both-banks town (`both_banks`,
    // `crate::riverbank::BOTH_BANKS_MIN_POP` and up) the refit asks the lots
    // instead ([`one_bank_by_lots`]): the circuit is refitted round the
    // market's bank alone when the other bank holds too few lots to be worth
    // walling, and those few lots stay a suburb outside it. It is never
    // refitted round the other bank alone: the market and its square are the
    // town's heart and stay inside (coordinator, 2026-11-15, defect (a): the
    // first version walled the bigger bank and left the square outside).
    let mut spans_water = wall_state.spans_water;
    // Every counted lot, before the span test below may drop one bank's:
    // the fallback circuit across the river (riverbank sites) is fitted to them.
    let all_lots: Vec<(Vec2, &Vec<Vec2>)> = lot_polys.clone();
    if both_banks && spans_water && rk_site {
        let sides: Vec<f64> = lot_polys.iter().map(|(c, _)| site.bank_side(*c)).collect();
        if let Some(side) = one_bank_by_lots(&sides, mkt_side) {
            spans_water = false;
            lot_polys.retain(|(c, _)| site.bank_side(*c) == side);
            mass = lot_polys.iter().flat_map(|(_, p)| p.iter().copied()).collect();
        }
    }
    // The other way (riverbank sites, both-banks towns): a far bank grown
    // after a one-bank circuit, holding enough lots by the same test, is
    // walled with the market's bank, so the town's two halves share one
    // circuit as a walled city on both banks does.
    if upgrade && !far_out.is_empty() {
        let sides: Vec<f64> = lot_polys.iter().map(|(c, _)| site.bank_side(*c)).chain(far_out.iter().map(|_| -mkt_side)).collect();
        if one_bank_by_lots(&sides, mkt_side).is_none() {
            spans_water = true;
            for (c, p) in &far_out {
                lot_polys.push((*c, p));
                mass.extend(p.iter().copied());
            }
        }
    }
    // **Not the reference** (2026-11-15, defect (a), riverbank sites only):
    // the market square holds the wall out like a lot, so the refit never
    // leaves the town's square outside its own circuit.
    // Whether or not the circuit the town grew held it: a square left
    // outside that circuit (the reference's wall rises from the street nodes
    // at one epoch and can miss a square on the bank) is taken in by the
    // refit, or the refit is given up (`outline`'s `keep`).
    let plaza = plaza.filter(|p| rb && p.len() >= 3);
    let rk = site.kind == "river" || site.kind == "riverthrough";
    let old_area = poly_area(&old_ring).abs();
    // The circuits to try, in order: the one the span tests chose, and, on a
    // riverbank river site whose choice was one bank, a circuit across the
    // river round every lot (2026-11-15: where the river is a stub at the box
    // edge, the bank split cuts the town in two and no one-bank circuit fits
    // it; seed 483920 #0 at 2 048 x 1 311 kept the reference's circuit round
    // empty ground that way).
    // The other way round too: a circuit the span tests sent across the
    // river that cannot be drawn there (the two quarters face each other
    // nowhere, so `outline` sees two towns) falls back to the market's bank
    // alone (seed 483920 #11 at 384 x 288: `MarginNotMet` and the grown
    // circuit kept round an unbuilt bay before this).
    let mut cands: Vec<(bool, Vec<(Vec2, &Vec<Vec2>)>)> = vec![(spans_water, lot_polys.clone())];
    if rb && rk_site && !spans_water {
        let mut every = all_lots.clone();
        every.extend(far_out.iter().map(|(c, p)| (*c, *p)));
        cands.push((true, every));
    } else if rb && rk_site && spans_water {
        cands.push((false, all_lots.iter().copied().filter(|(c, _)| site.bank_side(*c) == mkt_side).collect()));
    }
    let mut last = KeptReason::MarginNotMet;
    'cand: for (spans, lots_c) in cands {
        let mut mass: Vec<Vec2> = if spans == spans_water { mass.clone() } else { lots_c.iter().flat_map(|(_, p)| p.iter().copied()).collect() };
        // A circuit across the river (riverbank sites): the water between the
        // two banks' quarters joins their outlines, so the wall crosses the
        // river at two water gates and encloses the reach the town faces
        // across, as a walled river town's does; elsewhere the river stays
        // outside. A centreline vertex counts where both banks hold a lot
        // within `river_w / 2 + CLOSE_R_M` of it; it and the channel's two
        // edges there join the outline (water, so not mass). Without it the
        // two quarters were joined only along the bridge, a 16 m neck the
        // wall drew as a loop into the river (seed 483920 #11 at 384 x 288),
        // and a bridge where the quarters do not face each other still drew
        // one; so the bridge itself joins nothing, and quarters that face
        // each other nowhere are not one walled town.
        let mut connect: Vec<Vec2> = Vec::new();
        if rb && spans && rk_site {
            let reach = site.river_w / 2.0 + CLOSE_R_M;
            let sides: Vec<(Vec2, f64)> = lots_c.iter().map(|(c, _)| (*c, site.bank_side(*c))).collect();
            for w in site.river.windows(2) {
                let (a, b) = (w[0], w[1]);
                let n = (b - a).rot90().norm();
                for c in edge_samples(a, b) {
                    let faces = |side: f64| sides.iter().any(|(p, sd)| *sd == side && p.dist(c) <= reach);
                    if faces(1.0) && faces(-1.0) {
                        connect.extend([c, c + n * (site.river_w / 2.0), c + n * (-site.river_w / 2.0)]);
                    }
                }
            }
        }
        if let Some(pz) = plaza {
            mass.extend(pz.iter().copied());
        }

        // Street edges those lots front join the mass; every other non-primary edge
        // inside the old circuit is a candidate for removal.
        let lot_corners = mass.clone();
        let near_corner = |p: Vec2| lot_corners.iter().any(|m| m.dist(p) <= FRONT_REACH_M);
        let mut stray: Vec<usize> = Vec::new();
        for e in &g.edges {
            if !e.alive || e.cls == "primary" {
                continue;
            }
            let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
            // Edges with an end outside the old circuit are the extramural network.
            if !(point_in_poly(a, &old_ring) && point_in_poly(b, &old_ring)) {
                continue;
            }
            let samples = edge_samples(a, b);
            if samples.iter().any(|s| near_corner(*s)) {
                mass.extend(samples);
            } else {
                stray.push(e.id);
            }
        }

        // A street sample on a bridge, quay or ford stands in the water, which no wall
        // encloses (the circuit follows the bank): drop those points, with the same
        // water tests `build_wall`'s land classification uses.
        mass.retain(|p| if rk { site.river_dist(*p) > site.river_w / 2.0 + 1.0 } else { !site.is_water(*p) });
        // **Not the reference** (2026-11-15, riverbank sites only): the points
        // within [`BANK_EXEMPT_M`] of the bank owe the wall no clearance. Ruling
        // EZ item 1's houses stand on the water there, and the circuit's water
        // side is the bank itself; where the land arc meets the bank walk such a
        // point fell a metre either side of the ring and refused the refit of a
        // town that otherwise fitted (seed 12345 at 384 x 246, the town at
        // (108,71): `MarginNotMet` on points 3-5 m from the bank).
        let inland: Vec<Vec2> = if rb && rk { mass.iter().copied().filter(|p| site.river_dist(*p) > site.river_w / 2.0 + BANK_EXEMPT_M).collect() } else { Vec::new() };
        let c0 = poly_centroid(&convex_hull(&mass));
        let mut pad = CLEARANCE_M;
        // **Not the reference** (2026-11-15, defect (b), riverbank sites only):
        // the circuit follows [`outline`], the built area with its gaps under
        // `2 * CLOSE_R_M` closed over, not the star [`envelope`], and a circuit
        // across the river keeps that shape instead of the reference's convex
        // hull (`BuiltMass::exact`). No star there: where the outline cannot
        // be drawn (the town in separate quarters, or a clearance grown to
        // `CLOSE_R_M`) this circuit is given up.
        let mut out_polys: Vec<&[Vec2]> = lots_c.iter().map(|(_, p)| p.as_slice()).collect();
        if let Some(pz) = plaza {
            out_polys.push(pz);
        }
        // The points the margin is measured on: the mass (on a riverbank river
        // site its inland part), less any detached lot [`outline`] leaves out.
        let mut margin_pts: Vec<Vec2> = if rb && rk { inland.clone() } else { mass.clone() };
        for _ in 0..=MAX_REFITS {
            let mut trial = wall_state.clone();
            let hull = if rb {
                match outline(&out_polys, &[mass.as_slice(), connect.as_slice()].concat(), pad, plaza.map(poly_centroid)) {
                    Some((h, dropped)) => {
                        // A detached lot the loop leaves out owes it no margin.
                        let near = |p: &Vec2| dropped.iter().any(|d| d.dist(*p) <= 1.5 * OUTLINE_CELL_M);
                        margin_pts = if rb && rk { inland.clone() } else { mass.clone() };
                        margin_pts.retain(|p| !near(p));
                        h
                    }
                    None => continue 'cand,
                }
            } else {
                envelope(c0, &mass, pad)
            };
            let built = BuiltMass { hull, spans_water: spans, exact: rb };
            build_wall_from_mass(seed, site, anchors, g, &mut trial, wall_state.epoch, harbour, opts, built, false);
            let Some(ring) = trial.ring.clone() else {
                last = KeptReason::Refused;
                continue 'cand;
            };
            let ring = ring.as_slice();
            if trial.ring == wall_state.ring {
                last = KeptReason::Refused;
                continue 'cand;
            }
            // **Not the reference** (2026-11-15, riverbank sites only): where
            // the circuit builder's land arc meets its bank walk the ring can
            // cross itself in a sliver; [`untangle`] cuts those off, and a ring
            // folded over the town (a loop over `UNTANGLE_MAX_SHARE` of its area)
            // is no wall (seed 12345 at 384 x 246, the town at (375,134),
            // before this: a refit across the town).
            let untangled;
            let ring = if rb && ring_self_crosses(ring) {
                let Some(u) = untangle(ring) else {
                    last = KeptReason::Refused;
                    continue 'cand;
                };
                trial.ring = Some(u.clone());
                untangled = u;
                untangled.as_slice()
            } else {
                ring
            };
            let margin = least_margin(&margin_pts, ring, trial.land_arc.as_deref());
            // The market square must be inside the circuit taken (defect (a)):
            // where the bank walk or `untangle` cut the square's corner of
            // the town off, this circuit is given up and the next one tried
            // (the census at 384: 12 squares outside on 3 seeds before this).
            if plaza.is_some_and(|pz| !point_in_poly(poly_centroid(pz), ring)) {
                last = KeptReason::Refused;
                continue 'cand;
            }
            if margin >= MIN_MARGIN_M {
                let area_ratio = poly_area(ring).abs() / old_area;
                // A circuit widened across the river to take in the far bank
                // (Ruling FB, above) is larger than the one it replaces by design.
                if area_ratio > MAX_AREA_RATIO && !(spans && !wall_state.spans_water) {
                    last = KeptReason::NotSmaller;
                    continue 'cand;
                }
                // Take the stray streets that now leave the ring (an end outside it,
                // or a crossing of it); the ones still wholly inside stay.
                let mut removed = 0usize;
                for &id in &stray {
                    let (a, b) = (g.nodes[g.edges[id].a].pt(), g.nodes[g.edges[id].b].pt());
                    if !point_in_poly(a, ring) || !point_in_poly(b, ring) || !ring_crossings(ring, a, b).is_empty() {
                        kill_edge(g, id);
                        removed += 1;
                    }
                }
                *wall_state = trial;
                return HugOutcome::Refit { clearance_m: pad, area_ratio, streets_removed: removed };
            }
            // Widen by what was missing (a point outside the ring has a negative
            // margin, which this also covers), plus a metre so the next build
            // clears the bar instead of meeting it.
            pad += MIN_MARGIN_M - margin + 1.0;
            last = KeptReason::MarginNotMet;
        }
    }
    HugOutcome::Kept(last)
}
