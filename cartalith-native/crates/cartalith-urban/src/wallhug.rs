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
//!    [`CLEARANCE_M`].
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
use crate::geom::{Vec2, convex_hull, dist_pt_seg, point_in_poly, poly_area, poly_centroid};
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
/// `mkt_side` the market's. `None` when the circuit should span the river
/// (the smaller bank holds at least [`FAR_BANK_LOT_SHARE`] of the larger
/// bank's lots and at least [`MIN_LOTS`]); otherwise `Some` of the larger
/// bank, the one the circuit is refitted round (the market's on a tie).
/// Must never answer a bank that holds no lots while the other holds some.
pub fn one_bank_by_lots(sides: &[f64], mkt_side: f64) -> Option<f64> {
    let near = sides.iter().filter(|&&s| s == mkt_side).count();
    let far = sides.len() - near;
    if (near.min(far) as f64) >= (MIN_LOTS as f64).max(near.max(far) as f64 * FAR_BANK_LOT_SHARE) {
        return None;
    }
    Some(if near >= far { mkt_side } else { -mkt_side })
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
    for p in parcels {
        if p.frontage < 3.2 || p.depth < 6.0 || p.poly.len() < 3 {
            continue;
        }
        let c = poly_centroid(&p.poly);
        if !point_in_poly(c, &old_ring) {
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
    // instead. The circuit spans the river only when its smaller bank holds at
    // least [`FAR_BANK_LOT_SHARE`] of the larger bank's lots (and
    // [`MIN_LOTS`]); otherwise it is refitted round the larger bank alone, and
    // the other bank's few lots stay a suburb outside it. The larger bank need
    // not be the market's (a market pinned on a bank the town could not build
    // on): the circuit builder decides which bank is "land" from the market
    // (`build_wall_from_mass`'s `is_land`), so it is handed `home`, the lot
    // nearest the larger bank's lots' mean, in the market's place.
    let mut spans_water = wall_state.spans_water;
    let mut home = anchors.clone();
    if both_banks && spans_water && (site.kind == "river" || site.kind == "riverthrough") && site.river.len() >= 2 {
        let mkt_side = site.bank_side(anchors.market);
        let sides: Vec<f64> = lot_polys.iter().map(|(c, _)| site.bank_side(*c)).collect();
        if let Some(big_side) = one_bank_by_lots(&sides, mkt_side) {
            spans_water = false;
            let kept: Vec<Vec2> = lot_polys.iter().filter(|(c, _)| site.bank_side(*c) == big_side).map(|(c, _)| *c).collect();
            mass = lot_polys.iter().filter(|(c, _)| site.bank_side(*c) == big_side).flat_map(|(_, p)| p.iter().copied()).collect();
            if big_side != mkt_side {
                let mean = kept.iter().fold(Vec2::new(0.0, 0.0), |a, c| a + *c) * (1.0 / kept.len().max(1) as f64);
                if let Some(c) = kept.iter().copied().min_by(|a, b| a.dist(mean).total_cmp(&b.dist(mean))) {
                    home.market = c;
                }
            }
        }
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
    let rk = site.kind == "river" || site.kind == "riverthrough";
    mass.retain(|p| if rk { site.river_dist(*p) > site.river_w / 2.0 + 1.0 } else { !site.is_water(*p) });
    let c0 = poly_centroid(&convex_hull(&mass));
    let old_area = poly_area(&old_ring).abs();
    let mut pad = CLEARANCE_M;
    // Ruling FB: a circuit the lots no longer let span the river.
    let downgraded = spans_water != wall_state.spans_water;
    for attempt in 0..=MAX_REFITS {
        let mut trial = wall_state.clone();
        let hull = envelope(c0, &mass, pad);
        let built = BuiltMass { hull, spans_water };
        build_wall_from_mass(seed, site, &home, g, &mut trial, wall_state.epoch, harbour, opts, built, false);
        let Some(ring) = trial.ring.clone() else {
            return HugOutcome::Kept(KeptReason::Refused);
        };
        let ring = ring.as_slice();
        if trial.ring == wall_state.ring {
            return HugOutcome::Kept(KeptReason::Refused);
        }
        let margin = least_margin(&mass, ring, trial.land_arc.as_deref());
        // **Not the reference** (Ruling FB): when the span test refitted a
        // spanning circuit to one bank, the last widening is taken even if a
        // lot still stands within the margin, because the alternative this
        // function would otherwise keep is the reference circuit round an
        // empty bank and across the river twice, the defect the ruling forbids
        // (seed 483920 #0 at 2 048 x 1 311: `MarginNotMet`, the wall round the
        // empty far bank).
        if margin >= MIN_MARGIN_M || (downgraded && attempt == MAX_REFITS) {
            let area_ratio = poly_area(ring).abs() / old_area;
            if area_ratio > MAX_AREA_RATIO {
                return HugOutcome::Kept(KeptReason::NotSmaller);
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
    }
    HugOutcome::Kept(KeptReason::MarginNotMet)
}
