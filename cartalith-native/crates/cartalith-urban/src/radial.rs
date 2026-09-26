//! Milestone 8 — `buildRadialStreets` and `buildWaterway` (reference lines
//! 28835-28939), the **radial** (Venus Project) planning mode.
//!
//! The second of the reference's two growth models and the single biggest
//! culture fork in the subsystem: where `'organic'` epoch-loops least-cost
//! routes into an accreted tangle ([`crate::growth`]), `'radial'` lays a hub,
//! five concentric ring streets and twelve straight spokes in **one shot, with
//! no epoch loop at all**, and leans on the unchanged planar-face detector to
//! turn ring × spoke crossings into annular-wedge blocks. Only the layout rule
//! differs; [`crate::blocks`] and everything downstream needs no branch.
//!
//! ## The range was verified against the reference before anything was sliced
//!
//! `URBAN_MORPHOLOGY_SCOPE.md` claims 28835-28939 for this milestone, and four
//! consecutive urban milestones have found their claimed range wrong — twice at
//! both ends — so it was checked rather than trusted. It is right this time:
//! 28835 opens the block comment, `buildRadialStreets` is 28844, `buildWaterway`
//! 28928 and its last line 28939, with `buildPlaza` (milestone 8a, already
//! landed) starting at 28942.
//!
//! ## And so was the drift, which is the check the freeze now needs
//!
//! `reference/` is pinned at v2.10 while the source has moved twelve mainline
//! versions and a whole DCC line past it, so a port written against the freeze
//! can be porting a definition that no longer exists. Both functions were
//! diffed between v2.10 and the source's current head (v2.73): **byte-identical**.
//! The only text that moved in that whole span is the `PLAZA_MARKET_POP`
//! constant v2.73 added *after* `buildWaterway`, which belongs to milestone 8a.
//!
//! ## Two things a reader will look for and not find
//!
//! - **Nothing here consumes `seed` except one substream.** `build_waterway`
//!   takes `seed` and never draws — it is pure geometry off the market anchor —
//!   and [`RADIAL_SUBSTREAM`] is the only label `build_radial_streets` opens.
//!   Both are asserted, because a stray draw in either would silently shift
//!   every later value out of that substream.
//! - **The reference's only call site discards the return value**
//!   (line 31012, `buildRadialStreets(seed,site,anchors,g,{maxRF});`). The
//!   struct is still returned, because it is the cheapest honest way to test
//!   ring radii and hub size, and because a later milestone wanting the rings
//!   should not have to re-derive them.
//!
//! ## Two of thirty-seven mutations survive, and neither is a gap
//!
//! The sweep runs 37 single-constant mutations of this module against the
//! fixture set. **Thirty-five die.** The two that live are not untested code —
//! they are the two places where a mutation cannot change behaviour at all, and
//! saying so here is cheaper than rediscovering it every time the sweep runs.
//!
//! - **`run.len() > 1` -> `> 0` is an EQUIVALENT MUTANT.** A run of one point
//!   reaches [`Graph::add_polyline_street`](crate::graph::Graph::add_polyline_street),
//!   finds no pair to walk, and returns having touched nothing — so the guard
//!   only says what the call already does. That is not left as an argument:
//!   `a_one_point_polyline_lays_nothing` asserts it, which is what keeps the
//!   mutant equivalent if that function ever changes.
//! - **`>` -> `>=` on the river guard is a MEASURE-ZERO tie.** `river_dist` is
//!   a continuous distance to a polyline and `river_guard` a fixed float, so
//!   only an exact `f64` equality separates the two forms. Measured over the
//!   45 scenarios the capture sweeps — **17 737 evaluated points — the closest
//!   any sample came to the guard is 3.1 mm**, and it would need 0.0. The
//!   guard's *value* is a different question and is pinned: `+8 -> +9` dies.
//!
//! Six others survived the first sweep and were **fixture limits, now closed**;
//! the four scenarios that close them are described at the top of the tests.
//! The lesson they carried is worth keeping: `max_rf * 0.38` lands at 100-274 m
//! across the natural cross-product, so the 90 m floor never binds; and a disc
//! of at most 274 m about a market near the middle of a 1700 x 1250 box never
//! comes within 25 m of an edge, so neither land margin is ever the clause that
//! rejects a point. **A constant whose branch the fixture set cannot reach is an
//! untested constant however many scenarios pass.**
//!
//! ## The draw order is load-bearing and is not the order it reads in
//!
//! Four numbers come out of `'radial-organic'` before any geometry
//! (`wobPhase1`, `wobPhase2`, `wobFreq1`, `wobFreq2` — JS evaluates a
//! multi-declarator `const` left to right), then **one per spoke and one per
//! cross-spoke, 24 more, drawn whether or not the spoke is laid**: the jitter
//! is inside the `a = …` expression, which is evaluated before the `landSeg`
//! test that may reject the street. A port that draws only for accepted spokes
//! is bit-identical on an all-dry site and diverges on every wet one.

use crate::geom::{Vec2, js_cos, js_max, js_min, js_round, js_sin};
use crate::graph::Graph;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::Site;

use std::f64::consts::PI;

/// The one substream this module opens. `build_waterway` opens none.
pub const RADIAL_SUBSTREAM: &str = "radial-organic";

/// Provenance on every ring-street polyline (line 28877), verbatim.
pub const PROV_RING: &str = "Ring street: one of several concentric circuits at regular intervals \
     linking the radial spokes, each ring a regular polygon standing in for a circle at city \
     scale (M-VEN-1).";

/// Provenance on every primary spoke (line 28878), verbatim.
pub const PROV_SPOKE: &str = "Radial spoke: a straight route from the central Resource Management \
     hub outward to the residential ring — the spine of the concentric-zone city (M-VEN-1).";

/// Provenance on every intermediate cross-spoke (line 28879), verbatim.
pub const PROV_CROSS: &str = "Cross-spoke: an intermediate radial in the wider outer band, added \
     where the larger circumference wants more crossings than the inner spokes alone provide \
     (M-VEN-1).";

/// Provenance on the circular irrigation canal (line 28938), verbatim.
pub const PROV_WATERWAY: &str = "Circular irrigation waterway: a fully-closed canal encircling \
     the built city outside the residential ring, drawing on and returning to the agricultural \
     belt it borders; the same ring supplies the star fort's wet moat when the town is fortified \
     (M-VEN-3).";

/// Evenly-spaced primary radial streets — the spine of the concentric-zone city.
const SPOKES: usize = 12;
/// Fresco's plans show several concentric circuits, not just two.
const N_RINGS: usize = 5;
/// ±5.5% radius variation: small enough that consecutive rings never cross.
const WOB_AMP: f64 = 0.055;

/// `buildRadialStreets`' return value.
///
/// `rings` is the hub ring followed by `N_RINGS` evenly spaced circuits out
/// to `outer_r`, so it has `N_RINGS + 1` entries and `rings[0] == hub_r`.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialPlan {
    pub center: Vec2,
    pub rings: Vec<f64>,
    pub spokes: usize,
    pub outer_r: f64,
}

/// One decorative detail: the closed irrigation ring.
///
/// The reference returns `[{kind, poly, prov}]` — an array of exactly zero or
/// one — and its caller spreads it into `details`. That is an [`Option`], and
/// saying so here makes the "radius too small, no canal" case impossible to
/// drop silently. Milestone 15 owns the general `details` vocabulary; `kind` is
/// carried as a field so this slots into it unchanged when that lands.
#[derive(Debug, Clone, PartialEq)]
pub struct Waterway {
    pub kind: &'static str,
    pub poly: Vec<Vec2>,
    pub prov: &'static str,
}

/// `buildRadialStreets` (line 28844) — hub, concentric rings and radial spokes,
/// laid into `g` in one pass.
///
/// **Why the rings wobble.** Perfectly compass-drawn circles read as the one
/// artificial thing on a map whose every other profile is noisy and accretive,
/// so each ring radius is modulated by two summed sine terms whose phase drifts
/// ring to ring like tree-ring eccentricity. It is a *look* decision the
/// reference documents as such, and it is reproduced exactly because it moves
/// every vertex.
///
/// **Only the twelve primary spokes are tagged `'primary'`.** The rings stay
/// `'street'` on purpose: `buildWall`'s gate loop creates a land gate for
/// `cls == "primary"` edges only, and an earlier version of the reference left
/// the spokes untagged, which gave a fortified Venus town **zero land gates**.
/// The cross-spokes stop one ring short of the outermost circuit, so they can
/// never become a through-route candidate and never need a gate of their own.
///
/// **A spoke is sampled along its whole length, not at its endpoints.** Both
/// ends can sit on dry land with open water in between; the reference samples
/// 13 points (`k = 0..=12`) and rejects the whole segment if any is wet.
pub fn build_radial_streets(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    max_rf: f64,
) -> RadialPlan {
    let c = anchors.market;
    let mut r_org = stream(seed, RADIAL_SUBSTREAM);

    // Four draws, in declarator order -- JS evaluates `const a=…,b=…` left to
    // right, and these are two separate `const` lines of two declarators each.
    let wob_phase1 = r_org.range(0.0, PI * 2.0);
    let wob_phase2 = r_org.range(0.0, PI * 2.0);
    let wob_freq1 = r_org.int(3, 5) as f64;
    let wob_freq2 = r_org.int(6, 9) as f64;
    let wobble = |ang: f64, ring_idx: f64| {
        1.0 + WOB_AMP
            * (js_sin(ang * wob_freq1 + wob_phase1 + ring_idx * 0.35) * 0.6
                + js_sin(ang * wob_freq2 + wob_phase2 + ring_idx * 0.5) * 0.4)
    };

    // A denser mesh needs a SMALLER footprint for the same target population,
    // not a bigger one: five rings plus cross-spokes pack far more buildable
    // frontage into a given radius than the original two-ring layout did
    // (empirically tuned to ~90-105% realisation, M-VEN-1).
    let outer_r = js_max(90.0, max_rf * 0.38);
    let hub_r = js_max(24.0, outer_r * 0.13);

    // `site.riverW ? site.riverW/2+8 : 0` -- a JS truthiness test, so 0, -0 and
    // NaN all take the else branch. Rust's `!= 0.0` is true for NaN, so the NaN
    // case is spelled out rather than inherited. Hoisted because `site.riverW`
    // cannot change inside this call; bit-identical to evaluating it per point.
    let river_guard = if site.river_w != 0.0 && !site.river_w.is_nan() {
        site.river_w / 2.0 + 8.0
    } else {
        0.0
    };
    let land = |p: Vec2| {
        !site.is_water(p)
            && site.river_dist(p) > river_guard
            && p.x > 25.0
            && p.y > 25.0
            && p.x < site.wm - 25.0
            && p.y < site.hm - 25.0
    };
    let land_seg = |p0: Vec2, p1: Vec2| {
        let n = 12;
        for k in 0..=n {
            if !land(p0.lerp(p1, f64::from(k) / f64::from(n))) {
                return false;
            }
        }
        true
    };

    // The hub ring, then N_RINGS evenly spaced out to the residential ring.
    let mut rings = vec![hub_r];
    for i in 1..=N_RINGS {
        rings.push(hub_r + (outer_r - hub_r) * (i as f64) / (N_RINGS as f64));
    }

    // `g` is passed in rather than captured, so the ring pass and the two spoke
    // passes below can each take it mutably without fighting `land`'s borrow of
    // `site`.
    let draw_ring = |g: &mut Graph, radius: f64, ring_idx: f64, cls: &'static str, w: f64| {
        // A vertex about every 8 m, so the circle reads smooth at city scale.
        let sides = js_max(24.0, js_round(radius / 8.0));
        let n = sides as i64;
        let mut pts = Vec::with_capacity((n + 1) as usize);
        for k in 0..=n {
            let a = 2.0 * PI * (k as f64) / sides;
            let rad = radius * wobble(a, ring_idx);
            pts.push(Vec2::new(c.x + js_cos(a) * rad, c.y + js_sin(a) * rad));
        }
        // Split into contiguous on-land runs, so a ring is never drawn uncapped
        // across water. A run of one point lays nothing -- `addPolylineStreet`
        // walks pairs, so a single point has no pair, and the reference's own
        // `run.length > 1` guard says the same thing explicitly.
        let mut run: Vec<Vec2> = Vec::new();
        for p in pts {
            if land(p) {
                run.push(p);
            } else {
                if run.len() > 1 {
                    g.add_polyline_street(&run, cls, w, 0, PROV_RING);
                }
                run.clear();
            }
        }
        if run.len() > 1 {
            g.add_polyline_street(&run, cls, w, 0, PROV_RING);
        }
    };

    // Every ring stays 'street'-class: internal mesh, not a through-route.
    // The outermost is drawn wider (6.5) because it is the residential circuit.
    let last = rings.len() - 1;
    for (idx, &radius) in rings.iter().enumerate() {
        draw_ring(g, radius, idx as f64, "street", if idx == last { 6.5 } else { 4.5 });
    }

    // The twelve primary spokes -- the only radial streets that reach the
    // outermost ring, and therefore the only ones tagged so a wall can gate them.
    for i in 0..SPOKES {
        // The jitter draw happens BEFORE the land test, so it is consumed even
        // when the spoke is rejected. See the module note.
        let a = 2.0 * PI * (i as f64) / (SPOKES as f64) + r_org.range(-0.045, 0.045);
        let p0 = Vec2::new(c.x + js_cos(a) * hub_r, c.y + js_sin(a) * hub_r);
        let p1 = Vec2::new(c.x + js_cos(a) * outer_r, c.y + js_sin(a) * outer_r);
        if land_seg(p0, p1) {
            g.add_street(p0.x, p0.y, p1.x, p1.y, "primary", 5.0, 0, PROV_SPOKE);
        }
    }

    // Intermediate cross-spokes, offset half a sector, spanning only the wider
    // outer band and stopping ONE RING SHORT of the outermost circuit.
    let mid_r = rings[(N_RINGS / 2).max(1)];
    let cross_outer = rings[rings.len() - 2];
    for i in 0..SPOKES {
        let a =
            2.0 * PI * (i as f64 + 0.5) / (SPOKES as f64) + r_org.range(-0.045, 0.045);
        let p0 = Vec2::new(c.x + js_cos(a) * mid_r, c.y + js_sin(a) * mid_r);
        let p1 = Vec2::new(c.x + js_cos(a) * cross_outer, c.y + js_sin(a) * cross_outer);
        if land_seg(p0, p1) {
            g.add_street(p0.x, p0.y, p1.x, p1.y, "street", 4.5, 0, PROV_CROSS);
        }
    }

    RadialPlan { center: c, rings, spokes: SPOKES, outer_r }
}

/// `buildWaterway` (line 28928) — the closed irrigation canal, drawn beyond the
/// outermost built ring.
///
/// **It cannot overlap a building or a parcel by construction**, which is the
/// whole design: no block, parcel or building geometry is ever generated out
/// past the street network's reach in the first place. That is the same
/// "generate it where geometry cannot reach" pattern the opt-in ruined-decay
/// pass relies on, and it is why this ring carries none of the must-not-sit-in-
/// water invariant that applies to built fabric — sitting past the built city
/// is the point.
///
/// **The radius cap is a fix, not a tidy-up.** Without it the ring ran off the
/// map edge and got cut flat there, and a fully-closed circle that terminates
/// in a straight edge is exactly what a closed canal must never look like
/// (M-VEN-3). Under 40 m after the cap there is no canal at all.
///
/// `seed` is taken and unused: the reference's signature has it and this draws
/// nothing. Keeping the parameter keeps the call sites honest about which
/// stages are seeded.
pub fn build_waterway(_seed: u32, site: &Site, anchors: &Anchors, radius: f64) -> Option<Waterway> {
    let c = anchors.market;
    let sides = 64.0_f64;
    let edge_r = js_min(js_min(c.x, c.y), js_min(site.wm - c.x, site.hm - c.y)) - 12.0;
    let radius = js_min(radius, edge_r);
    if radius < 40.0 {
        return None;
    }
    let n = sides as i64;
    let mut poly = Vec::with_capacity((n + 1) as usize);
    for k in 0..=n {
        let a = 2.0 * PI * (k as f64) / sides;
        poly.push(Vec2::new(c.x + js_cos(a) * radius, c.y + js_sin(a) * radius));
    }
    Some(Waterway { kind: "waterway", poly, prov: PROV_WATERWAY })
}

#[cfg(test)]
mod tests;
