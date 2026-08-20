//! Anchors and primary routes — reference lines **28743-28833**, three
//! functions.
//!
//! This is the first milestone that produces a real street graph. [`Site`]
//! (milestone 5) says where the water and the relief are; [`place_anchors`]
//! picks the one point the whole town is organised around, and the two
//! `build_primaries*` functions lay the arterial backbone into the
//! [`Graph`](crate::graph::Graph) that milestone 7's growth, milestone 10's
//! enceinte and milestone 12's blocks all accrete onto. Everything downstream
//! is measured from the market and grown off the primaries, so a single
//! differently-chosen cell here moves every block, parcel and building in the
//! town.
//!
//! # The two ways a town gets its backbone
//!
//! [`build_primaries`] **synthesises** it: rasterise the site into 8 m cells
//! whose cost is a Tobler-flavoured slope penalty plus water and bank terms,
//! then run [`astar`](crate::astar::astar) from each external route endpoint to
//! the market, reinforcing already-used cells so later routes braid onto
//! earlier ones.
//!
//! [`build_primaries_from_paths`] **injects** it: the host app hands over the
//! real inter-settlement roads reaching this settlement, as polylines of metre
//! offsets from the settlement, and the town is grown around those instead.
//! `generate()` prefers this whenever `opts.primaryPaths` is non-empty.
//!
//! # Three things this milestone establishes for the rest of the subsystem
//!
//! 1. **Neither route builder draws a random number.** `buildPrimaries` and
//!    `buildPrimariesFromPaths` both take a `seed` parameter and neither reads
//!    it — verified by grep over both bodies, and reproduced here as
//!    `_seed`. Only [`place_anchors`] consumes RNG, and it consumes exactly one
//!    substream (`stream(seed, 'anchors')`, 800 draws). Milestone 16 needs that
//!    to hold when it reasons about `generate()`'s overall draw order.
//! 2. **Both return values are dead.** `generate()` calls both for their effect
//!    on `g` and discards the routes (lines 31021-31022). They are returned
//!    here anyway, because they are what the reference returns and because they
//!    make a far stricter golden than the graph alone.
//! 3. **`Graph::from_paths` exists now.** Milestone 2 deliberately left the
//!    field out — nothing set it. `buildPrimariesFromPaths` sets
//!    `g._fromPaths = true` (line 28830) and milestone 10's `builtMassHull`
//!    reads it (line 29709) to discount the bare degree-2 vertices a resampled
//!    real road drags in. Without it the enceinte over-encloses along
//!    arterials, exactly as the reference's own v1.01 note describes.

use crate::astar::astar;
use crate::geom::{Vec2, chaikin, js_cos, js_max, js_min, js_round, js_sin, simplify};
use crate::graph::Graph;
use crate::rng::stream;
use crate::site::Site;
use std::collections::HashSet;
use std::f64::consts::PI;

#[cfg(test)]
mod tests;

/// The town's anchor points. The reference returns `{market, prov}` and nothing
/// else has ever been added to it, but every later milestone reads
/// `anchors.market` — twenty-odd call sites across `grow`, `buildWall`,
/// `buildBlocks`, `buildMarkets`, `buildCivic`, `assignDistricts` — so it is
/// the single most-read value the engine produces.
#[derive(Debug, Clone, PartialEq)]
pub struct Anchors {
    pub market: Vec2,
    /// One of three fixed strings, chosen by site kind. `&'static str` because
    /// there are exactly three and none is built by concatenation.
    pub prov: &'static str,
}

/// One primary route: the smoothed polyline that was laid into the graph.
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub pts: Vec<Vec2>,
    /// The index into `site.routeEnds` this route came from.
    ///
    /// `None` for [`build_primaries_from_paths`], which pushes `{pts}` with no
    /// `i` at all where [`build_primaries`] pushes `{pts, i}`. Nothing reads
    /// either, so the asymmetry is invisible to the engine; it is reproduced
    /// because erasing it would be a silent decision about a field a later
    /// milestone might yet want.
    pub i: Option<usize>,
}

const PROV_LAND_MARKET: &str = "Market sited on flat, dry ground where the land routes converge — an inland market town with no water frontage (M-REG-1, lit. review §4).";
const PROV_RIVER_MARKET: &str = "Market sited on flat land above the flood band, close to the bridge crossing (route convergence). Refs: M-REG-6, lit. review §4.";
const PROV_QUAY_MARKET: &str = "Market sited on the shore flat just behind the quay: goods change mode at the break-of-bulk point (harbour-city family, lit. review §4-5).";

const PROV_PRIMARY: &str = "Primary route: least-cost path over slope/water cost field with trail reinforcement (Helbing 1997, Tobler kernel M-REG-5); immutable backbone (M-GRW-3).";
const PROV_PRIMARY_FROM_PATHS: &str = "Primary route: the real inter-settlement road the town grew along (host road network; M-REG-1, M-GRW-3).";

/// `placeAnchors(seed, site)` — reference line 28744.
///
/// Sites the market square by rejection sampling: 400 seeded candidates on the
/// landward side of the break-of-bulk point, each scored against slope,
/// distance from that point and distance out of the flood band, keeping the
/// best. The whole search consumes exactly **800 draws** from the `'anchors'`
/// substream — two per candidate, drawn *before* any of the four rejection
/// tests, so a rejected candidate costs the same draws as an accepted one and
/// the sequence is independent of the site's shape.
///
/// # The break-of-bulk point, and its third arm
///
/// `site.bridgePt || (site.harbour && site.harbour.pt) || {x: Wm*0.52, y: Hm*0.42}`.
/// **The literal is live, not defensive** — a landlocked site has no bridge
/// *and* no quay, so the town centres on that fixed fraction of the box. That
/// is the finding milestone 5 wrote forward to this one, and `landlocked*` in
/// the goldens is the fixture for it.
///
/// # Two asymmetries between the two "river" kinds
///
/// `riverthrough` shares `river`'s wider `[60, 240]` candidate band but **not**
/// its `120 m` preferred distance — the score's ternary tests `'river'` alone,
/// so a bisecting river prefers a market at `100 m` like a coastal town while
/// still being allowed to sit out to 240 m. Both branches are pinned
/// separately; a fixture set with only one of the two kinds cannot see the
/// difference.
///
/// # `Math.cos` / `Math.sin` are V8's, not the platform's
///
/// Every candidate's position comes out of [`js_cos`] / [`js_sin`]. Those are
/// the third and fourth measured V8 libm divergences in this port (see
/// [`js_sin`] for the numbers) and the reason milestone 6 measured before
/// trusting rather than after a golden failed.
pub fn place_anchors(seed: u32, site: &Site) -> Anchors {
    let mut r = stream(seed, "anchors");
    let reference = site
        .bridge_pt
        .or(site.harbour.pt)
        .unwrap_or_else(|| Vec2::new(site.wm * 0.52, site.hm * 0.42));
    let river_kind = site.kind == "river" || site.kind == "riverthrough";
    let d_band = if river_kind { [60.0, 240.0] } else { [60.0, 180.0] };

    let mut best: Option<Vec2> = None;
    let mut bs = f64::NEG_INFINITY;
    for _ in 0..400 {
        // Landward side: the reference's comment says "water lies south", and
        // the half-circle it draws from is the upper one in screen coordinates.
        let ang = r.range(-PI, 0.0);
        let d = r.range(d_band[0], d_band[1]);
        let p = Vec2::new(reference.x + js_cos(ang) * d, reference.y + js_sin(ang) * d);
        if p.x < 80.0 || p.y < 80.0 || p.x > site.wm - 80.0 || p.y > site.hm - 80.0 {
            continue;
        }
        if site.is_water(p) {
            continue;
        }
        let rd = site.river_dist(p);
        if rd < site.river_w / 2.0 + 30.0 {
            continue; // flood band
        }
        let s = site.slope(p);
        // `Math.max(0, rd - 260)` through `js_max`, not `f64::max`: a NaN
        // `riverDist` propagates in JS and is absorbed in Rust, and a NaN score
        // fails `score > bs` where the absorbed one need not. Reachable — the
        // site model returns NaN for an out-of-bounds raster probe (milestone 5,
        // finding 8).
        let score = -(s * 4.0)
            - (d - (if site.kind == "river" { 120.0 } else { 100.0 })).abs() / 60.0
            - js_max(0.0, rd - 260.0) / 120.0;
        if score > bs {
            bs = score;
            best = Some(p);
        }
    }

    let market = best.unwrap_or_else(|| Vec2::new(reference.x, reference.y - 120.0));
    let prov = if site.no_water {
        PROV_LAND_MARKET
    } else if river_kind {
        PROV_RIVER_MARKET
    } else {
        PROV_QUAY_MARKET
    };
    Anchors { market, prov }
}

/// The primary-route cost raster's cell size, in metres (reference `CS`).
const CS: f64 = 8.0;

/// `buildPrimaries(seed, site, anchors, g)` — reference line 28771.
///
/// # The cost field
///
/// One `f64` per 8 m cell, sampled at the cell centre:
///
/// - base `1 + (slope * 3.2)^2`, the Tobler-flavoured slope penalty;
/// - **in water**, `+1.5` within 14 m of the bridge point and `+240` otherwise
///   — so a channel is crossable only at the bridge and the open sea is merely
///   ruinously expensive rather than impassable;
/// - **on land within `riverW/2 + 22` m of the channel**, `+3` for marshy bank
///   and shore flat;
/// - all of it scaled by `CS`, so a cell's cost is in metres-equivalent.
///
/// `Math.pow(x, 2)` is written `x * x` here. That is not a liberty: V8's
/// `Math.pow` with an exponent of exactly 2 was measured bit-identical to both
/// `x * x` and `f64::powf(2.0)` on 60,000 arguments, so the three are the same
/// function and the multiply is the one that cannot be mistaken for a general
/// `pow` port.
///
/// # Reinforcement is order-dependent on purpose
///
/// Each route runs over a **copy** of the raster with every previously-used
/// cell multiplied by `0.45`, and then adds its own cells to that set. So route
/// *n* sees the union of routes 0..n-1 and routes braid onto each other in
/// `site.routeEnds` order. This is Helbing's trail-formation model and it is
/// deliberately path-dependent — the reference's own citation says so. Nothing
/// here may be reordered, parallelised or de-duplicated.
///
/// The used-cell set is a `HashSet`, matching the reference's `Set`: each
/// distinct cell is multiplied exactly once per route, and since the factor is
/// applied to disjoint indices the iteration order cannot affect the result.
///
/// # `to_cell`'s clamp is this function's responsibility
///
/// `astar` in this port takes `(usize, usize)` and panics out of range — a
/// deliberate divergence from the reference, which reads past its typed arrays
/// and gets `undefined` (milestone 3). The clamp that makes that safe is
/// `max(1, min(W-2, round(p.x/CS)))`, and it lives here, exactly as the
/// reference writes it. Note it clamps to `[1, W-2]`, not `[0, W-1]`: the
/// border ring of cells is never a route endpoint.
///
/// # Panics
///
/// If the site box is under three cells on either axis (`Wm < 24` or
/// `Hm < 24`), the reference's clamp produces an index past the end of its own
/// raster and reads `undefined`; here `astar` asserts instead. The engine's box
/// is a fixed 1700 x 1250 m, i.e. 213 x 157 cells.
pub fn build_primaries(_seed: u32, site: &Site, anchors: &Anchors, g: &mut Graph) -> Vec<Route> {
    let w = (site.wm / CS).ceil() as usize;
    let h = (site.hm / CS).ceil() as usize;
    let mut cost = vec![0.0f64; w * h];
    let bp = site.bridge_pt;

    for y in 0..h {
        for x in 0..w {
            let p = Vec2::new((x as f64 + 0.5) * CS, (y as f64 + 0.5) * CS);
            let sp = site.slope(p) * 3.2;
            let mut c = 1.0 + sp * sp;
            if site.is_water(p) {
                let db = match bp {
                    Some(bp) => p.dist(bp),
                    None => f64::INFINITY,
                };
                // crossing only at the bridge (M-REG-6); sea impassable
                c = if db < 14.0 { c + 1.5 } else { c + 240.0 };
            } else if site.river_dist(p) < site.river_w / 2.0 + 22.0 {
                c += 3.0; // marshy banks / shore flat
            }
            cost[y * w + x] = c * CS;
        }
    }

    let to_cell = |p: Vec2| -> (usize, usize) {
        (
            js_max(1.0, js_min(w as f64 - 2.0, js_round(p.x / CS))) as usize,
            js_max(1.0, js_min(h as f64 - 2.0, js_round(p.y / CS))) as usize,
        )
    };
    let mk = to_cell(anchors.market);

    let mut used: HashSet<usize> = HashSet::new();
    let mut routes: Vec<Route> = Vec::new();
    for (i, &end) in site.route_ends.iter().enumerate() {
        let mut c2 = cost.clone();
        for &k in &used {
            c2[k] *= 0.45; // trail reinforcement (Helbing 1997; M-GRW-1)
        }
        // The reference's `return` inside a `forEach` callback: this route is
        // dropped, the remaining ones still run, and `used` is left untouched.
        let Some(path) = astar(&c2, w, h, to_cell(end), mk) else { continue };
        for &(cx, cy) in &path {
            used.insert(cy * w + cx);
        }
        let pts: Vec<Vec2> = path
            .iter()
            .map(|&(cx, cy)| Vec2::new((cx as f64 + 0.5) * CS, (cy as f64 + 0.5) * CS))
            .collect();
        let pts = chaikin(&simplify(&pts, 7.0), false);
        let pts = simplify(&pts, 1.2);
        routes.push(Route { pts, i: Some(i) });
    }

    // Laid into the graph only after every route is traced, so the reinforcement
    // above sees no graph state at all. width M-NET-8
    for rt in &routes {
        g.add_polyline_street(&rt.pts, "primary", 7.0, 0, PROV_PRIMARY);
    }
    routes
}

/// `buildPrimariesFromPaths(seed, site, anchors, g, paths)` — reference line
/// 28811.
///
/// The v0.97 alternative to [`build_primaries`], and the one the host app
/// actually takes: instead of synthesising arterials from `routeEnds`, take the
/// **real** inter-settlement roads that reach this settlement — polylines of
/// metre offsets from the settlement, in the layout's local un-rotated frame —
/// translate each to the market, and grow the town around them. Roads
/// converging on the market is the classic market-town form, so nothing
/// downstream needs to change.
///
/// # The in-box run is taken from the market end and stops at the first exit
///
/// `inBox` is the site box widened by 6 m on all four sides, and the loop
/// **breaks** rather than continuing on the first point outside it. The
/// reference's comment explains why: the road enters the town from outside, so
/// it leaves the box exactly once, and keeping the contiguous run from the
/// market end is what "the part of this road that is inside the town" means. A
/// path whose *first* point is already outside contributes nothing.
///
/// # `sm.len() < 2` is dead, and is kept
///
/// `pts` has at least 2 entries by the guard above; `simplify` is the identity
/// below three points and never drops an endpoint; and `chaikin` on an open
/// 2-point line returns 4 points. So the smoothed polyline always has at least
/// 2 and the final guard cannot fire. Reproduced as written — the same call
/// milestone 3 made about `astar`'s dead `Infinity` check.
///
/// # `seed` and `site.riverW`/relief are unread
///
/// `seed` is taken and never used, and the site is read only for `Wm`/`Hm`.
/// This function is therefore pure geometry over the host's own roads: no RNG,
/// no cost raster, no terrain.
pub fn build_primaries_from_paths(
    _seed: u32,
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    paths: &[Vec<Vec2>],
) -> Vec<Route> {
    let m = anchors.market;
    let mut routes: Vec<Route> = Vec::new();
    let in_box =
        |q: Vec2| q.x >= -6.0 && q.x <= site.wm + 6.0 && q.y >= -6.0 && q.y <= site.hm + 6.0;

    for path in paths {
        if path.len() < 2 {
            continue;
        }
        let mut pts: Vec<Vec2> = Vec::new();
        for o in path {
            let q = Vec2::new(m.x + o.x, m.y + o.y);
            if in_box(q) {
                pts.push(q);
            } else {
                break;
            }
        }
        if pts.len() < 2 {
            continue;
        }
        if pts[0].dist(m) > 1.0 {
            // anchor the inner end on the market
            pts.insert(0, Vec2::new(m.x, m.y));
        }
        let sm = chaikin(&simplify(&pts, 7.0), false);
        let sm = simplify(&sm, 1.2);
        if sm.len() < 2 {
            continue;
        }
        g.add_polyline_street(&sm, "primary", 7.0, 0, PROV_PRIMARY_FROM_PATHS);
        // v1.01: marks injected real-road primaries — `builtMassHull`
        // (milestone 10) discounts their bare vertices. Set inside the loop, so
        // a call that contributes no usable path leaves the flag alone.
        g.from_paths = true;
        routes.push(Route { pts: sm, i: None });
    }
    routes
}
