//! Perimeter blocks around an open courtyard, on the town's outermost ring —
//! **not a reference port.** A deliberate departure under Ruling H
//! (`LARGE_ITEM_RULINGS.md`, 2026-09-12: *"change the ported urban algorithm
//! toward the owner's town plan — golden re-baseline authorised"*), sited by
//! Ruling AD (2026-09-23): *"the outermost ring by distance from market.
//! Matches how other density gradients already keyed on market distance work
//! in this engine"*. Reasoned against the owner's target plan
//! `design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`.
//!
//! # What the plan shows, and why this is a new subdivision
//!
//! The plan's outer quarters — the suburbs south-east and north-west of the
//! curtain above all — are **perimeter blocks**: one continuous ring of
//! building footprints following every street the block fronts, each lot
//! running from the street back to a small shared open court, usually with a
//! tree in it. Measured off the plan (the curtain is ~470 px across a town the
//! citadel's own header scales at ~450 m, so ~1 m/px): blocks ~45-85 m across,
//! a built ring ~15-20 m deep, courts ~10-25% of the block, frontages ~8-12 m,
//! and no gaps along the ring.
//!
//! [`crate::blocks::build_parcels`] cannot produce that. It plats strip lots
//! whose depth is a block-uniform `logn(22|30)` clamped to 14-46 m and capped
//! at 0.42 of the waist, so their backs do not share a line, and
//! [`crate::districts::build_buildings`] then puts a main range on the front
//! 7-11 m of each with wings and outbuildings behind — gardens, not a court.
//! Its one courtyard plan (M-BLD-3) is a *per-lot* typology: a ring of ranges
//! inside a single grand market/burgher plot. What the plan draws is a *block*
//! typology: the court belongs to the whole block.
//!
//! So [`build_courtyard_rings`] re-plats a qualifying block: the block's own
//! interior, inset uniformly by one ring depth, is the court; the band between
//! the street line and the court is cut into one trapezoid per block edge —
//! mitred along each vertex's bisector, so the trapezoids tile the band with no
//! gap and no overlap — and each trapezoid into lots along its street. Every
//! lot is the same `[P0, P1, Q1, Q0]` quad `build_parcels` makes (street
//! frontage first, back line on the court), flagged
//! [`Parcel::courtyard_ring`], and `build_buildings` fills each one with a
//! single range from street to court with no eaves gap. The court is left
//! unplatted, which is exactly what `build_details`' garden-tree pass already
//! plants in, so the plan's tree in the yard needs nothing new.
//!
//! # Which blocks — Ruling AD's outermost ring, and "high density"
//!
//! **Market distance** is the one [`crate::blocks::build_parcels`] already
//! keys its block-uniform depth on (M-PAR-2): the block interior's centroid to
//! `anchors.market`, Euclidean. It is the same `dM` the Clark demand gradient
//! in [`crate::growth::grow`] reads, measured there from a street origin. The
//! outermost ring is every built block whose `dM` is at least
//! [`OUTER_RING_FRAC`] of the largest built block's — a band relative to this
//! town's own extent, so a hamlet and a city both have one.
//!
//! **High density** is the second test, and it is what the plan's courts show:
//! a court no larger than [`COURT_MAX_FRAC`] of the block. A block too large
//! for one ring depth to reach that keeps its strip plat — a 12 000 m² field
//! behind a row of houses is not a perimeter block, and a ring 60 m deep would
//! not be a building. A block too small to leave [`COURT_MIN_AREA`] of court
//! keeps it too.
//!
//! A block is converted whole or not at all: every lot corner must be dry, and
//! every lot a simple, correctly-wound quad. A partial ring would be a strip
//! plat with a hole in it.
//!
//! # Not on the radial plan
//!
//! The Venus grammar's own outermost band is its logistics-warehouse belt
//! (M-VEN-5), a designed ring this would overwrite. The caller gates on the
//! organic plan, as the citadel does. Ponytail: extend when the owner rules on
//! it.
//!
//! # Golden consequence
//!
//! Draws come from a per-block `"courtyard/<blk.id>"` substream and a
//! `"courtyard/tone"` stream, never from `parcels/…` or `roof-tone`, and the
//! stage runs after every street-platted and wall lot exists. A converted
//! block's strip lots are removed and its ring lots appended, ids `cyd{n}`, so
//! every other lot keeps its id, geometry and `'bld'` stream. What moves is
//! disclosed case by case in `generate/tests/golden.rs`'s header.

use crate::blocks::{Block, Parcel};
use crate::geom::{Vec2, inset_poly, js_max, poly_area, poly_centroid, poly_self_intersects};
use crate::graph::Graph;
use crate::rng::{fnv1a, stream};
use crate::site::Site;
use crate::wallside::WallBacking;

#[cfg(test)]
mod tests;

/// Ruling AD's ring: a block qualifies when its market distance is at least
/// this share of the farthest built block's.
pub const OUTER_RING_FRAC: f64 = 0.7;
/// Depth of the built ring, street line to court, metres: from a single-pile
/// range (M-BLD-2's 7 m floor, plus a metre) to the plan's deepest
/// double-pile ring. Within it the depth is solved per block for the court
/// share drawn from [`COURT_FRAC`].
pub const RING_DEPTH: (f64, f64) = (8.0, 22.0);
/// The court's target share of the block interior, drawn once per block. The
/// plan's courts run 10-25%.
pub const COURT_FRAC: (f64, f64) = (0.12, 0.28);
/// Largest court share accepted when even [`RING_DEPTH`]'s deepest ring
/// cannot reach the target — the block is too big to be a perimeter block.
pub const COURT_MAX_FRAC: f64 = 0.35;
/// Smallest court, m² — below this the ring closes into a solid block.
pub const COURT_MIN_AREA: f64 = 80.0;
/// Lot frontage median and spread, metres (`logn`), clamped to
/// [`FRONTAGE_CLAMP`]. The plan's ring lots are 8-12 m.
pub const FRONTAGE: (f64, f64) = (9.0, 0.2);
pub const FRONTAGE_CLAMP: (f64, f64) = (6.0, 14.0);
/// Smallest lot, m² — `build_parcels`' own floor.
pub const MIN_LOT_AREA: f64 = 26.0;

/// The block inset uniformly by `depth`, or [`None`] when there is no court.
///
/// `inset_poly` alone is not that test. Inset a block past its own half-width
/// and the offset lines cross over: the result is the block's **mirror**,
/// correctly wound and with a positive area (a 24.5 m square inset 22 m comes
/// back as a 19.5 m square, 380 m²). Every edge of it runs backwards against
/// the edge it was offset from, so that is what is refused.
pub fn court_of(poly: &[Vec2], depth: f64) -> Option<Vec<Vec2>> {
    let court = inset_poly(poly, &[depth])?;
    let n = poly.len();
    (0..n)
        .all(|i| (poly[(i + 1) % n] - poly[i]).dot(court[(i + 1) % n] - court[i]) > 0.0)
        .then_some(court)
}

/// The ring depth whose court is `target` of the block, clamped to
/// [`RING_DEPTH`] — or [`None`] when the clamped ring's court is not a dense
/// one: over [`COURT_MAX_FRAC`] (too big a block) or under [`COURT_MIN_AREA`]
/// (too small). The court shrinks as the ring deepens, so a bisection finds it;
/// an inset that collapses counts as no court at all.
pub fn ring_depth(poly: &[Vec2], target: f64) -> Option<f64> {
    let area = poly_area(poly);
    let court = |d: f64| court_of(poly, d).map_or(0.0, |c| poly_area(&c));
    let (mut lo, mut hi) = RING_DEPTH;
    let d = if court(hi) >= area * target {
        hi
    } else if court(lo) <= area * target {
        lo
    } else {
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if court(mid) > area * target { lo = mid } else { hi = mid }
        }
        lo
    };
    let c = court(d);
    (c >= COURT_MIN_AREA && c <= area * COURT_MAX_FRAC).then_some(d)
}

/// A court and its ring lots, each `(edge index, [P0, P1, Q1, Q0])`.
pub type RingPlat = (Vec<Vec2>, Vec<(usize, Vec<Vec2>)>);

/// The court and ring lots for one block at ring depth `depth`, or [`None`]
/// when there is no court ([`court_of`]) or any lot would be a bowtie. `poly` is CCW (a
/// [`Block::poly`]). Returns `(court, lots)`, each lot `(edge index,
/// [P0, P1, Q1, Q0])`.
pub fn ring_plat(poly: &[Vec2], depth: f64, frontage: &mut dyn FnMut() -> f64) -> Option<RingPlat> {
    let n = poly.len();
    let court = court_of(poly, depth)?;
    let mut lots = Vec::new();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let (qa, qb) = (court[i], court[(i + 1) % n]);
        let k = js_max(1.0, (a.dist(b) / frontage()).round()) as usize;
        for j in 0..k {
            let (f0, f1) = (j as f64 / k as f64, (j + 1) as f64 / k as f64);
            let quad = vec![a.lerp(b, f0), a.lerp(b, f1), qa.lerp(qb, f1), qa.lerp(qb, f0)];
            if poly_self_intersects(&quad) || poly_area(&quad) <= 0.0 {
                return None;
            }
            // A sliver at a sharp corner is left open, as `build_parcels`
            // leaves one.
            if poly_area(&quad) >= MIN_LOT_AREA {
                lots.push((i, quad));
            }
        }
    }
    Some((court, lots))
}

/// Re-plat the outermost ring's dense blocks as courtyard perimeter blocks.
/// Removes each converted block's strip lots from `parcels` and appends its
/// ring lots; returns the converted blocks' ids, in block order.
pub fn build_courtyard_rings(
    seed: u32,
    g: &Graph,
    blocks: &[Block],
    parcels: &mut Vec<Parcel>,
    market: Vec2,
    epochs: i32,
    site: &Site,
) -> Vec<String> {
    let built = |b: &&Block| !b.plaza && parcels.iter().any(|p| p.block == b.id);
    let d_m = |b: &Block| poly_centroid(&b.poly).dist(market);
    let max_d = blocks.iter().filter(built).map(d_m).fold(0.0, f64::max);
    let ring: Vec<&Block> =
        blocks.iter().filter(built).filter(|b| d_m(b) >= max_d * OUTER_RING_FRAC).collect();

    let margin = if site.kind == "river" { site.river_w / 2.0 + 1.0 } else { 3.0 };
    let dry = |q: &Vec2| !site.is_water(*q) && site.river_dist(*q) >= margin;
    let mut tone = stream(seed, "courtyard/tone");
    let mut converted = Vec::new();
    let mut new_lots = Vec::new();
    for blk in ring {
        let mut r = stream(fnv1a(&blk.id), &format!("courtyard/{}", blk.id));
        let Some(depth) = ring_depth(&blk.poly, r.range(COURT_FRAC.0, COURT_FRAC.1)) else { continue };
        let mut frontage = || r.logn(FRONTAGE.0, FRONTAGE.1).clamp(FRONTAGE_CLAMP.0, FRONTAGE_CLAMP.1);
        let Some((_, lots)) = ring_plat(&blk.poly, depth, &mut frontage) else { continue };
        if lots.is_empty() || !lots.iter().all(|(_, q)| q.iter().all(dry)) {
            continue;
        }
        let n = blk.face_ids.len();
        for (i, quad) in lots {
            let eref = g.edge_between(blk.face_ids[i], blk.face_ids[(i + 1) % n]);
            new_lots.push(Parcel {
                id: String::new(), // numbered below, once the order is final
                frontage: quad[0].dist(quad[1]),
                depth: (quad[0].dist(quad[3]) + quad[1].dist(quad[2])) / 2.0,
                area: poly_area(&quad),
                poly: quad,
                block: blk.id.clone(),
                age: eref.map_or(epochs as f64, |eid| js_max(0.0, (epochs - g.edges[eid].epoch) as f64)),
                edge_cls: eref.map_or("street", |eid| g.edges[eid].cls),
                tone: tone.u(),
                wall_backing: WallBacking::No,
                courtyard_ring: true,
                gate_quality: None,
            });
        }
        converted.push(blk.id.clone());
    }
    parcels.retain(|p| !converted.contains(&p.block));
    for (k, mut p) in new_lots.into_iter().enumerate() {
        p.id = format!("cyd{k}");
        parcels.push(p);
    }
    converted
}
