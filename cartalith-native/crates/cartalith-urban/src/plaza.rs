//! Milestone 8, part one — `buildPlaza` (reference lines 28941-28963).
//!
//! The market place: a rectangle carved out of the town by widening the
//! principal street nearest the market anchor, away from the river (M-DEN-6,
//! "additive plaza mode"). Three streets are laid — the two ends and the far
//! side — and the fourth side is the primary street that was already there, so
//! the widened band becomes a **face of the street graph** in its own right.
//! [`crate::blocks::build_blocks`] then finds that face by
//! `point_in_poly(plaza.center, face)` and flags it, and
//! [`crate::blocks::build_parcels`] skips every flagged block. That chain is
//! what keeps the square open: nothing here marks anything unbuilt directly.
//!
//! ## Why this landed ahead of the rest of milestone 8
//!
//! Milestone 8 is `buildRadialStreets`, `buildWaterway` and `buildPlaza`. The
//! first two serve the *radial* (Venus) planning mode only; `buildPlaza` runs
//! on **both** branches of `generate()` (reference lines 31018 and 31024), and
//! without it every drawn town — organic ones included — plats a block straight
//! over its own market anchor. Milestone 12 recorded that as the most visible
//! gap it shipped with. The other two functions are unaffected and still
//! outstanding.
//!
//! ## Where it runs, which is not where it might look like it runs
//!
//! On the organic branch the call sits **between `buildPrimaries` and `grow`**
//! (line 31024), not after growth: the three plaza streets are in the graph
//! before the epoch loop starts, so the town grows *around* the square rather
//! than having one punched through it afterwards. Anything that reproduces
//! `generate()`'s order has to put it there —
//! `cartalith_civ::urban_adapter::run_layout` does.
//!
//! ## The one draw pair
//!
//! `stream(seed, 'plaza')` is its own labelled substream and takes exactly two
//! numbers, `range(55, 80)` then `range(26, 40)`, in that order (JS evaluates
//! the `const L=…,Wd=…` declarators left to right). Nothing else in the
//! subsystem reads that substream, so adding this stage cannot perturb any
//! other milestone's sequence — only the *graph* changes, which is the point.
//!
//! ## v2.73 — market place or village green
//!
//! A village green and a chartered town's market place are the same widened
//! bay; what differs is what the space is *for*, decided by market right, not
//! size. [`build_civic`](crate::amenities::build_civic) already draws the
//! chartered line at [`PLAZA_MARKET_POP`] ("a civic hall appears once a place
//! is a chartered town"), and v2.73 reuses that exact number rather than
//! inventing a second threshold for the same distinction (v2.63's rule) — so
//! this module owns the constant and `build_civic` reads it from here.
//!
//! The geometry is untouched: verified byte-for-byte against the reference's
//! own v2.71→v2.73 diff, the same three `addStreet` calls run in the same
//! order with the same widths regardless of `kind`, so blocks, parcels and
//! buildings stay bit-identical and `hashModel` (which covers
//! graph/blocks/parcels/buildings and NOT plaza or details) cannot move. Only
//! [`PlazaKind`] and the edge `prov` text are new. [`crate::hinterland`]'s
//! market-cross detail is the one downstream consumer: it is gated on
//! `kind == Market`, since a village green carries no market right and so no
//! cross stands in it.

use crate::geom::{Vec2, dist_pt_seg, poly_centroid};
use crate::graph::Graph;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::Site;

/// The population at which a settlement is a chartered town rather than a
/// village (v2.73's own `const PLAZA_MARKET_POP=1500`) — the same number
/// [`build_civic`](crate::amenities::build_civic) has gated its civic hall on
/// since milestone 14, reused rather than restated.
pub const PLAZA_MARKET_POP: f64 = 1500.0;

/// The provenance string the reference writes on the plaza and on all three of
/// the streets it lays, when `kind` is [`PlazaKind::Market`] (line 28960
/// pre-v2.73; v2.73's own market-branch literal is unchanged), verbatim.
///
/// A constant rather than a field on [`Plaza`]: there is exactly one of these,
/// where `Anchors::prov` is one of three chosen by site kind. Golden-asserted
/// against the reference's own literal all the same.
pub const PROV: &str =
    "Market place formed by widening the principal street (additive plaza mode, M-DEN-6); \
     stall encroachment shapes later frontages.";

/// The provenance string for a village green ([`PlazaKind::Green`]), v2.73's
/// other branch, verbatim.
pub const PROV_GREEN: &str =
    "Village green: the common open centre the settlement grew around — grazing, assembly \
     and the pond. Below the chartered-town threshold there is no market right, so no cross \
     stands in it (M-DEN-6).";

/// What the widened bay off the principal street *is for* — v2.73's addition.
///
/// Decided by market right, not size: a chartered town's plaza is commercial
/// (a market cross stands in it); a village's green is common land (grazing,
/// assembly, the pond — no cross, because there is no right to mark).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlazaKind {
    Market,
    Green,
}

/// `buildPlaza`'s return value — the market square, or the village green.
///
/// `poly` is the reference's own `[p1, p2, q2, q1]` winding: `p1`/`p2` are the
/// two ends *on* the widened street and `q1`/`q2` the far side, so `p1 → p2` is
/// the existing carriageway and the other three sides are the streets this
/// function laid. `center` is `polyCentroid` of that quad, and is the point
/// [`crate::blocks::build_blocks`] tests faces against.
#[derive(Debug, Clone, PartialEq)]
pub struct Plaza {
    pub center: Vec2,
    pub poly: Vec<Vec2>,
    pub kind: PlazaKind,
}

/// `buildPlaza` (line 28942) — the market place, by widening the principal
/// street nearest the market on the side away from the river.
///
/// Returns [`None`] when the graph holds no live `'primary'` edge at all, which
/// is the reference's own `if(!be)return null` and is a real state rather than
/// an error: `buildPrimaries` returns without laying anything when `astar`
/// finds no path (an all-NaN cost field does exactly that), and the radial
/// branch reaches here with whatever `buildRadialStreets` produced.
///
/// **`g` is mutated before the return value exists.** The three streets go in
/// through [`Graph::add_street`], so the graph is planar-corrected — crossings
/// split, near-collinear nodes promoted to junctions — and the quad this
/// returns is built from the *pre-snap* points, exactly as the reference builds
/// it. The two are not the same rectangle when `add_street` moved an endpoint,
/// and the reference uses the un-moved one for both the centroid and the
/// polygon; reproduced.
///
/// `pop` is v2.73's addition — the reference's own `popTarget`, the same value
/// [`build_civic`](crate::amenities::build_civic) is gated on, not the later
/// realised `pop` [`crate::hinterland::build_details`] reads. It decides
/// [`PlazaKind`] only; the geometry above this point does not read it.
pub fn build_plaza(seed: u32, site: &Site, anchors: &Anchors, g: &mut Graph, pop: f64) -> Option<Plaza> {
    let mut r = stream(seed, "plaza");

    // Nearest live primary edge to the market. Strict `<`, so a tie keeps the
    // lowest-indexed edge -- the same first-index-wins the rest of this
    // subsystem is built on.
    let mut be: Option<usize> = None;
    let mut bd = f64::INFINITY;
    for (i, e) in g.edges.iter().enumerate() {
        if !e.alive || e.cls != "primary" {
            continue;
        }
        let d = dist_pt_seg(anchors.market, g.nodes[e.a].pt(), g.nodes[e.b].pt());
        if d < bd {
            bd = d;
            be = Some(i);
        }
    }
    let be = be?;

    let a = g.nodes[g.edges[be].a].pt();
    let b = g.nodes[g.edges[be].b].pt();
    let dir = (b - a).norm();
    let mid = a.lerp(b, 0.5);
    // European market place band (M-DEN-6). Two draws, in declaration order.
    let l = r.range(55.0, 80.0);
    let wd = r.range(26.0, 40.0);

    // Widen on the side away from the river: probe 20 m either way along the
    // edge normal and keep whichever is further from the channel. `river_dist`
    // is `buildSite`'s, so a landlocked site answers this too (it has no
    // channel, and the probe simply picks a side deterministically).
    let nl = dir.rot90();
    let side_p = mid + nl * 20.0;
    let side = if site.river_dist(side_p) > site.river_dist(mid + nl * -20.0) {
        1.0
    } else {
        -1.0
    };

    let p1 = mid + dir * (-l / 2.0);
    let p2 = mid + dir * (l / 2.0);
    let q1 = p1 + nl * (side * wd);
    let q2 = p2 + nl * (side * wd);

    // v2.73: market right, not size, decides what the space is for. Reuses
    // `build_civic`'s own chartered-town threshold rather than a second one.
    let kind = if pop >= PLAZA_MARKET_POP { PlazaKind::Market } else { PlazaKind::Green };
    let prov = if kind == PlazaKind::Market { PROV } else { PROV_GREEN };

    // Three sides only: `p1 → p2` is the street being widened and is already in
    // the graph. Width 5, epoch 0 -- the plaza's edges are as old as the
    // primaries they hang off, which is what `build_parcels`' age gate reads
    // for any lot that ends up fronting one.
    g.add_street(p1.x, p1.y, q1.x, q1.y, "street", 5.0, 0, prov);
    g.add_street(q1.x, q1.y, q2.x, q2.y, "street", 5.0, 0, prov);
    g.add_street(q2.x, q2.y, p2.x, p2.y, "street", 5.0, 0, prov);

    let poly = vec![p1, p2, q2, q1];
    Some(Plaza { center: poly_centroid(&poly), poly, kind })
}

#[cfg(test)]
mod tests;
