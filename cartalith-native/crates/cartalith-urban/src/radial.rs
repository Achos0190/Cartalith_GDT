//! Milestone 8 — the radial (Venus) planning mode: `buildRadialStreets` and
//! `buildWaterway`, reference lines **28835-28939**.
//!
//! The engine's *second* way of laying a town out. `generate()` forks on
//! `profile.planning` (reference line 31011): `'organic'` runs
//! [`build_primaries`](crate::routes::build_primaries) and then the epoch loop
//! in [`grow`](crate::growth::grow), accreting a tangle; `'radial'` calls
//! [`build_radial_streets`] once and **never calls `grow` at all**. Concentric
//! ring streets, twelve straight spokes off a central hub, twelve intermediate
//! cross-spokes in the wider outer band — laid in one pass, and then handed to
//! exactly the same unchanged planar-face detector every other mode uses, which
//! turns ring × spoke crossings into annular-wedge blocks for free.
//!
//! [`build_waterway`] is the profile's one genuinely new piece of
//! infrastructure: a closed decorative canal drawn *outside* the outermost
//! built ring, where no parcel or building geometry is ever generated, so it
//! cannot overlap built fabric by construction.
//!
//! ## Milestone 8a took `buildPlaza` out of here
//!
//! `buildPlaza` sat between these two in the reference (lines 28941-28965) and
//! shipped separately as [`crate::plaza`], because it runs on **both** branches
//! of `generate()` while these two serve the radial branch only. Nothing in
//! this module duplicates it, and `generate()` calls it *after* `buildWall` on
//! this branch (line 31018) rather than before `grow` as on the organic one.
//!
//! ## The range was right
//!
//! Seven consecutive urban milestones found their stated line range wrong.
//! This one is correct as stated: 28835 is the radial section header, 28844 is
//! `buildRadialStreets`' `function` keyword, 28928 is `buildWaterway`'s, and
//! 28939 is the line that closes it — 28940 is blank and 28941 opens milestone
//! 8a's plaza header. All six boundaries are asserted by the golden capture,
//! not merely eyeballed.
//!
//! ## The RNG contract: twenty-eight draws, none of them conditional
//!
//! `stream(seed, 'radial-organic')` is this milestone's own labelled substream
//! and nothing else in the subsystem reads it. It takes exactly **28** draws:
//! two `range` for the wobble phases, two `int` for the wobble frequencies,
//! then one `range` per spoke and one per cross-spoke. The per-spoke draw is
//! taken **before** `landSeg` decides whether that spoke is laid at all, so a
//! site that rejects every spoke consumes the same 28 numbers as one that lays
//! them all. Golden-pinned across all 30 scenarios by a delegating counter the
//! capture wraps `rOrg` in.
//!
//! ## Where the "organic softening" comes from
//!
//! The reference's own post-review note (line 28847): mathematically exact
//! concentric circles read as too mechanical against a hand-drawn-cadastral
//! map, so every ring radius is modulated by two summed sine terms whose phase
//! drifts ring to ring, and every spoke angle gets ±0.045 rad of jitter. The
//! amplitude is 5.5% — deliberately small enough that consecutive rings, which
//! are `(outerR - hubR) / 5` apart, can never cross.

use crate::geom::{Vec2, js_cos, js_max, js_min, js_round, js_sin, js_truthy_num};
use crate::graph::Graph;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::Site;
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

/// Written on every ring-street polyline (reference line 28878), verbatim.
pub const PROV_RING: &str = "Ring street: one of several concentric circuits at regular intervals \
     linking the radial spokes, each ring a regular polygon standing in for a circle at city \
     scale (M-VEN-1).";

/// Written on every primary spoke (reference line 28879), verbatim.
pub const PROV_SPOKE: &str = "Radial spoke: a straight route from the central Resource Management \
     hub outward to the residential ring — the spine of the concentric-zone city (M-VEN-1).";

/// Written on every intermediate cross-spoke (reference line 28880), verbatim.
pub const PROV_CROSS: &str = "Cross-spoke: an intermediate radial in the wider outer band, added \
     where the larger circumference wants more crossings than the inner spokes alone provide \
     (M-VEN-1).";

/// [`build_waterway`]'s `kind` (reference line 28938).
pub const WATERWAY_KIND: &str = "waterway";

/// [`build_waterway`]'s provenance (reference line 28939), verbatim.
pub const PROV_WATERWAY: &str = "Circular irrigation waterway: a fully-closed canal encircling \
     the built city outside the residential ring, drawing on and returning to the agricultural \
     belt it borders; the same ring supplies the star fort's wet moat when the town is fortified \
     (M-VEN-3).";

/// `buildRadialStreets`' return value — the layout it just laid, so a later
/// stage can read the rings without re-deriving them.
///
/// **`generate()` discards it** (reference line 31012), exactly as it discards
/// both route builders' returns. It is reproduced because it is what the
/// reference returns and because it makes a far stricter golden than the graph
/// alone: `rings` pins both `Math.max` floors and the even spacing between
/// them without going through `add_street`'s planarity correction first.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialStreets {
    /// `anchors.market`, unchanged — the hub every ring is drawn about.
    pub center: Vec2,
    /// Six radii: the hub ring, then `nRings = 5` evenly spaced out to the
    /// residential ring. `rings[5] == outer_r`.
    pub rings: Vec<f64>,
    /// `spokes` — always 12; a field rather than a constant because the
    /// reference returns it.
    pub spokes: usize,
    pub outer_r: f64,
}

/// One entry of `generate()`'s `details` array: the circular canal.
///
/// The reference's record is `{kind, poly, prov}` and all three are kept.
/// `kind` is what a heterogeneous `details` list is dispatched on, so dropping
/// it because it is a constant here would be a decision milestone 15 has to
/// undo when it ports `buildDetails`.
#[derive(Debug, Clone, PartialEq)]
pub struct Waterway {
    pub kind: &'static str,
    pub poly: Vec<Vec2>,
    pub prov: &'static str,
}

/// `buildRadialStreets` (line 28844) — the concentric-ring city, laid into `g`.
///
/// # What lands in the graph, and what class each piece gets
///
/// Six rings at `'street'`, the outermost 6.5 m wide and the rest 4.5 m. Then
/// twelve spokes hub → outer at `'primary'`, 5 m. Then twelve cross-spokes at
/// `'street'`, 4.5 m, running only `rings[2] → rings[4]`.
///
/// **The class split is load-bearing and the reference says why** (line 28884):
/// `buildWall`'s gate loop only places a land gate where an edge crossing the
/// trace is `cls === 'primary'`. The rings are internal mesh and must not be
/// gated; the spokes are the only edges meant to reach the wall, so they are
/// the only ones tagged. An earlier version of the reference left them
/// untagged and produced a fortified Venus town with **zero** land gates. The
/// cross-spokes stop one ring short of the outermost precisely so they can
/// never become a through-route candidate at all.
///
/// # `landSeg` samples, it does not test endpoints
///
/// A straight spoke can have both ends on dry land and still cross open water
/// in between. The reference samples 13 points (`k = 0..=12`) along each spoke
/// and rejects it if any one is wet, out of the river's `riverW/2 + 8` margin,
/// or within 25 m of the box edge. Rings are handled the other way round: a
/// ring is split into contiguous on-land *runs*, so it is never drawn uncapped
/// across water, and a run of one point is dropped.
///
/// # Trig
///
/// Every position here comes out of [`js_sin`] / [`js_cos`], not the platform
/// libm, and [`js_round`] / [`js_max`] / [`js_min`] likewise. Milestone 6
/// measured `sin`/`cos` at 1,942 and 2,160 disagreements with V8 over 80,214
/// arguments; this function is the subsystem's most trig-saturated and would
/// have shown every one of them.
pub fn build_radial_streets(
    seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    max_rf: f64,
) -> RadialStreets {
    let c = anchors.market;

    let mut r_org = stream(seed, "radial-organic");
    // Declaration order, left to right, as JS evaluates a comma-separated
    // `const` list. Four draws before any geometry exists.
    let wob_phase1 = r_org.range(0.0, PI * 2.0);
    let wob_phase2 = r_org.range(0.0, PI * 2.0);
    let wob_freq1 = r_org.int(3, 5) as f64;
    let wob_freq2 = r_org.int(6, 9) as f64;
    // +-5.5% radius variation, small enough consecutive rings never cross.
    let wob_amp = 0.055;
    let wobble = |ang: f64, ring_idx: f64| {
        1.0 + wob_amp
            * (js_sin(ang * wob_freq1 + wob_phase1 + ring_idx * 0.35) * 0.6
                + js_sin(ang * wob_freq2 + wob_phase2 + ring_idx * 0.5) * 0.4)
    };

    let spokes = 12usize;
    let outer_r = js_max(90.0, max_rf * 0.38);
    let hub_r = js_max(24.0, outer_r * 0.13);
    let n_rings = 5usize;

    // `site.riverW ? site.riverW/2+8 : 0` — JS truthiness, so 0 and NaN both
    // fall to the `0` arm. A landlocked site has `riverW = 0` and therefore no
    // channel margin at all, which is right: it has no channel.
    let river_margin =
        if js_truthy_num(site.river_w) { site.river_w / 2.0 + 8.0 } else { 0.0 };
    let land = |p: Vec2| -> bool {
        !site.is_water(p)
            && site.river_dist(p) > river_margin
            && p.x > 25.0
            && p.y > 25.0
            && p.x < site.wm - 25.0
            && p.y < site.hm - 25.0
    };
    let land_seg = |p0: Vec2, p1: Vec2| -> bool {
        let n = 12;
        for k in 0..=n {
            if !land(p0.lerp(p1, f64::from(k) / f64::from(n))) {
                return false;
            }
        }
        true
    };

    // The hub ring, then `nRings` evenly spaced out to the residential ring.
    let mut ring_r = vec![hub_r];
    for i in 1..=n_rings {
        ring_r.push(hub_r + (outer_r - hub_r) * (i as f64) / (n_rings as f64));
    }

    // `ringR.forEach((rr, idx) => drawRing(rr, idx, 'street', idx === last ? 6.5 : 4.5))`,
    // inlined: `drawRing` needs `g` mutably and `land` immutably, and a Rust
    // closure holding both would fight the borrow checker for no gain.
    let last_ring = ring_r.len() - 1;
    for (idx, &radius) in ring_r.iter().enumerate() {
        let w = if idx == last_ring { 6.5 } else { 4.5 };
        // Vertex roughly every 8 m so the polygon reads as a circle.
        let sides = js_max(24.0, js_round(radius / 8.0));
        // `Math.max` propagates NaN and JS `k <= NaN` is false, so a NaN radius
        // draws nothing. A Rust `as i64` cast saturates NaN to 0, which would
        // draw a single point instead; `-1` keeps the reference's empty loop.
        let n_sides = if sides.is_nan() { -1i64 } else { sides as i64 };
        let mut pts: Vec<Vec2> = Vec::new();
        for k in 0..=n_sides {
            let a = 2.0 * PI * (k as f64) / sides;
            let rad = radius * wobble(a, idx as f64);
            pts.push(Vec2::new(c.x + js_cos(a) * rad, c.y + js_sin(a) * rad));
        }
        // Split into contiguous on-land runs, so a ring is never drawn
        // uncapped across water. `flush` lays a run only if it has 2+ points.
        let mut run: Vec<Vec2> = Vec::new();
        for p in pts {
            if land(p) {
                run.push(p);
            } else {
                if run.len() > 1 {
                    g.add_polyline_street(&run, "street", w, 0, PROV_RING);
                }
                run.clear();
            }
        }
        if run.len() > 1 {
            g.add_polyline_street(&run, "street", w, 0, PROV_RING);
        }
    }

    // Primary spokes — the only radials meant to reach the wall boundary, and
    // so the only ones `buildWall` will cut a land gate for. The jitter draw is
    // taken unconditionally, before `land_seg` gets a say.
    for i in 0..spokes {
        let a = 2.0 * PI * (i as f64) / (spokes as f64) + r_org.range(-0.045, 0.045);
        let p0 = Vec2::new(c.x + js_cos(a) * hub_r, c.y + js_sin(a) * hub_r);
        let p1 = Vec2::new(c.x + js_cos(a) * outer_r, c.y + js_sin(a) * outer_r);
        if land_seg(p0, p1) {
            g.add_street(p0.x, p0.y, p1.x, p1.y, "primary", 5.0, 0, PROV_SPOKE);
        }
    }

    // Intermediate cross-spokes, offset half a sector, spanning only the wider
    // outer band and stopping ONE RING SHORT of the outermost — so they add
    // interior mesh density without ever becoming a through-route candidate.
    let mid_r = ring_r[n_rings.div_euclid(2).max(1)];
    let cross_outer = ring_r[ring_r.len() - 2];
    for i in 0..spokes {
        let a = 2.0 * PI * (i as f64 + 0.5) / (spokes as f64) + r_org.range(-0.045, 0.045);
        let p0 = Vec2::new(c.x + js_cos(a) * mid_r, c.y + js_sin(a) * mid_r);
        let p1 = Vec2::new(c.x + js_cos(a) * cross_outer, c.y + js_sin(a) * cross_outer);
        if land_seg(p0, p1) {
            g.add_street(p0.x, p0.y, p1.x, p1.y, "street", 4.5, 0, PROV_CROSS);
        }
    }

    RadialStreets { center: c, rings: ring_r, spokes, outer_r }
}

/// `buildWaterway` (line 28928) — the closed circular irrigation canal.
///
/// A 64-sided closed ring (65 points, first repeated as last) about the market,
/// drawn *beyond* the outermost built ring. Returns an empty vector where the
/// reference returns `[]`; `generate()` spreads the result into `details`
/// (line 31064), so the vector is the faithful shape.
///
/// # Two things it does not do
///
/// It **draws no random numbers** — `seed` is read nowhere in the body, so the
/// parameter is kept for signature parity and ignored, the same way
/// [`crate::routes::build_primaries`] keeps its unread `seed`. And it touches
/// no water invariant: the canal is deliberately not built fabric, so the "must
/// not sit in water" rule that governs parcels and buildings does not apply.
///
/// # The two guards, in the reference's order
///
/// `edgeR` caps the radius so the whole circle sits 12 m inside the box — the
/// fix for an earlier version whose ring ran off the map edge and was cut flat
/// there, which a fully-closed circle must never be. Then `radius < 40` drops
/// it entirely. Both `Math.min`s go through [`js_min`], which propagates NaN
/// where Rust's `f64::min` absorbs it; a NaN radius then fails `radius < 40`
/// (JS and Rust agree: every comparison against NaN is false) and the canal is
/// returned with NaN vertices, exactly as the reference would.
pub fn build_waterway(_seed: u32, site: &Site, anchors: &Anchors, radius: f64) -> Vec<Waterway> {
    let c = anchors.market;
    let sides = 64;
    let edge_r = js_min(js_min(js_min(c.x, c.y), site.wm - c.x), site.hm - c.y) - 12.0;
    let radius = js_min(radius, edge_r);
    if radius < 40.0 {
        return Vec::new();
    }
    let mut pts = Vec::with_capacity(sides + 1);
    for k in 0..=sides {
        let a = 2.0 * PI * (k as f64) / (sides as f64);
        pts.push(Vec2::new(c.x + js_cos(a) * radius, c.y + js_sin(a) * radius));
    }
    vec![Waterway { kind: WATERWAY_KIND, poly: pts, prov: PROV_WATERWAY }]
}
