//! Street villages -- **not a reference port.** Owner Ruling FE item 2
//! (2026-11-15, `LARGE_ITEM_RULINGS.md`): *"Villages: rows along the road.
//! Villages become street villages (houses lining the through road, green at
//! the landing), not the towns' courtyard blocks"*, answering
//! `TOWN_GENERATOR_JUDGEMENT.md` change 2; recorded under `DECISIONS.md`
//! section 7p behind `urban_street_village`.
//!
//! # Why the reference's village looks like a small town with houses missing
//!
//! Measured on the judgement's census (`town_river_census.rs::both_banks_audit`,
//! 384 cells, three seeds, 607 villages with houses): a median of 33 m of street
//! per building against 8 m in a town, the green over 150 m from the market
//! point in most villages, and 15 % of village lots unbuilt. Three causes, read
//! at the symbols:
//!
//! 1. `generate_pass`'s street budget has a 1 600 m floor (`target_len`), the
//!    street of a 762-person town, so every village under that is asked for
//!    more lanes than its houses need.
//! 2. `grow` spreads those lanes over the whole urbanisation front, and lots
//!    are platted only where lanes close a face into a block
//!    (`blocks::build_blocks`), so a lane on the fringe fronts nothing.
//! 3. `plaza::build_plaza` widens the nearest primary **at its middle**, which on
//!    a real map is a primary's half-length from the market point.
//!
//! # What a street village does instead (`generate_pass`, with the switch on)
//!
//! For a settlement under `plaza::PLAZA_MARKET_POP` on the organic plan:
//!
//! 1. the street budget is [`STREET_PER_HEAD_M`] a head, with no floor, and
//!    `grow` starts a lane only within [`ROAD_REACH_M`] of a primary (the
//!    approach and through roads), so the few back lanes it lays stay by the
//!    road;
//! 2. the green is widened on the primary at the point nearest the market
//!    point (`plaza::square_near_market`), not at the primary's middle;
//! 3. after the block lots are platted, [`plat_tofts`] lines the roads with
//!    tofts -- a house plot on open road frontage, [`TOFT_DEPTH_M`] deep --
//!    nearest the market first, until the village houses its target.
//!
//! The block lots (round the green, between a road and a back lane, beside
//! the embankment) are platted as before; tofts go only where no block lot
//! stands, so the two never overlap.

use crate::blocks::{Block, Parcel, lot_is_wet};
use crate::districts::{TERRAIN_BUILD_MIN, lot_suitability};
use crate::geom::{Vec2, point_in_poly, poly_area, poly_centroid, poly_self_intersects, seg_int};
use crate::graph::Graph;
use crate::rng::stream;
use crate::site::Site;
use crate::wallside::WallBacking;

/// Metres of lane a street village's growth is given per head of its target
/// (`generate_pass`'s `target_len` with the switch on). **A labelled
/// judgement**: a quarter of the 2.1 m a head a town gets (M-DEN-1/2, the
/// reference's), because a street village's houses front the through road
/// itself and lanes are only the few back lanes and a way to the green; 400
/// people get 100 m, 1 400 get 350 m. With no floor (Ruling FE item 2, the
/// judgement's *"without the 1 600 m floor"*).
pub const STREET_PER_HEAD_M: f64 = 0.25;

/// How far from a primary a street village's lane may start, metres (`grow`'s
/// `GrowOpts::road_reach`). **A labelled judgement**: one toft depth
/// ([`TOFT_DEPTH_M`]) and a lane's half-width either side of the road's own,
/// so a lane begins on the road or at the back of its plots, never out in the
/// fields: the judgement's *"grow along the approach and through road"*.
pub const ROAD_REACH_M: f64 = 40.0;

/// Depth of a toft, metres. **A labelled judgement**, `build_parcels`' own
/// fringe target depth (30 m, the reference's, M-PAR-1): a house on the road
/// and its yard behind. Cut back to a street that stands behind it.
pub const TOFT_DEPTH_M: f64 = 30.0;

/// A toft's frontage: the median and the log-normal spread `logn` is drawn
/// with, metres. **The reference's** platting frontage (`build_parcels`:
/// log-normal about 11 m), the same plot width the town's own lots take.
pub const TOFT_FRONTAGE: (f64, f64) = (11.0, 0.22);

/// The least depth a toft cut back by a street behind it may keep, metres.
/// `build_buildings`' own floor for building on a lot (6 m), so a toft is
/// never platted to stand empty.
const TOFT_MIN_DEPTH_M: f64 = 6.0;

/// The verge between a road's edge and a toft's front line, metres:
/// `build_blocks`' own 1.4 m inset, so a toft's frontage stands where a block
/// lot's would.
const VERGE_M: f64 = 1.4;

/// How far from the market point tofts may be platted along the roads, as a
/// multiple of `generate`'s `max_rf`. **A labelled judgement**: three times
/// the urbanisation front (about 600 m for 400 people), so a village squeezed
/// by water, slope or its own side lanes can string out along its road (a
/// street village is long, not round), and no further. Twice was measured
/// short: on the census at 384, 132 villages under 80 % of target against
/// 108 for the reference's (seed 246371 #104 stopped at 64 % with its road
/// running on past the reach).
pub const TOFT_REACH_RF: f64 = 3.0;

/// Head count per built lot (`generate`'s 5.2, the reference's household).
const HOUSEHOLD: f64 = 5.2;

/// **Not the reference** (Ruling FE item 2): the tofts lining a street
/// village's roads. `parcels` are the lots `build_parcels` platted (never
/// overlapped); `blocks` give the green to keep clear; `market` and `max_rf`
/// bound where tofts go; `pop_target` is how many people the village must
/// house, at 5.2 a lot, counting the block lots that will be built
/// (`build_buildings`' floors and, with `terrain_aware`, its suitability bar).
///
/// Every live street (and, on a riverbank site only, the embankment's quay and
/// riverside frontage, `riverbank::BANK_CLASS`: change 5 of
/// `TOWN_GENERATOR_JUDGEMENT.md`, 2026-11-15; elsewhere a quay is a harbour's
/// and stays unplatted, as before) is cut into frontages of [`TOFT_FRONTAGE`] from its first node; each frontage
/// on each side is a candidate toft [`TOFT_DEPTH_M`] deep (cut back to a street
/// behind it), kept when it is dry (`blocks::lot_is_wet`), inside the box,
/// clear of the green, of every street but its own and of every lot so far, and
/// buildable. Candidates are taken nearest the market first until the village
/// houses its target. Draws only from its own `'village/toft'` substream, so
/// no other stage's sequence moves. Ids are `toft{k}`, each its own one-lot
/// block, as `wallside`'s lots are.
///
/// **Must never** overlap a lot, cross a street, stand in water, plat a toft a
/// village does not need, or run for a town (`generate_pass` calls it only with
/// the switch on, for a village).
#[allow(clippy::too_many_arguments)]
pub fn plat_tofts(
    seed: u32,
    g: &Graph,
    site: &Site,
    blocks: &[Block],
    parcels: &[Parcel],
    market: Vec2,
    max_rf: f64,
    pop_target: f64,
    epochs: i32,
    terrain_aware: bool,
) -> Vec<Parcel> {
    let buildable = |p: &Parcel| p.frontage >= 3.2 && p.depth >= TOFT_MIN_DEPTH_M && !(terrain_aware && lot_suitability(site, p) < TERRAIN_BUILD_MIN);
    let have = parcels.iter().filter(|p| buildable(p)).count();
    let need = (pop_target / HOUSEHOLD).ceil() as usize;
    if have >= need {
        return Vec::new();
    }
    let greens: Vec<&[Vec2]> = blocks.iter().filter(|b| b.plaza).map(|b| b.face_poly.as_slice()).collect();
    let reach = TOFT_REACH_RF * max_rf;
    let mut r = stream(seed, "village/toft");
    // Every candidate: its distance to the market, the quad (front edge
    // first, `poly[0]`-`poly[1]` on the street), and the street's class.
    let mut cands: Vec<(f64, Vec<Vec2>, &'static str)> = Vec::new();
    // The riverside edges count as frontage only on a riverbank site
    // (`TOWN_GENERATOR_JUDGEMENT.md` change 5, Ruling EZ item 1): there the
    // embankment's quay and bank stretches are the village's waterfront, and a
    // toft on them is a house on the quay or on the water. Elsewhere a `quay`
    // is a harbour's, and the reference's village never fronted it.
    let waterside = |c: &str| c == "quay" || c == crate::riverbank::BANK_CLASS;
    for e in g.edges.iter().filter(|e| e.alive && (site.riverbank() || !waterside(e.cls))) {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        let len = a.dist(b);
        if len < 6.0 || crate::geom::dist_pt_seg(market, a, b) > reach {
            continue;
        }
        let dir = (b - a).norm();
        let n = dir.rot90();
        let off = e.w / 2.0 + VERGE_M;
        let mut s = 0.0;
        while s < len - 4.0 {
            let w = r.logn(TOFT_FRONTAGE.0, TOFT_FRONTAGE.1).clamp(6.0, 16.0);
            let (s0, s1) = (s, (s + w).min(len));
            s += w;
            if s1 - s0 < 6.0 {
                continue;
            }
            let (p0, p1) = (a + dir * s0, a + dir * s1);
            for side in [1.0, -1.0] {
                let nn = n * side;
                let (f0, f1) = (p0 + nn * off, p1 + nn * off);
                let depth = [f0, f1]
                    .into_iter()
                    .map(|f| street_behind(g, e.id, f, nn, TOFT_DEPTH_M + 8.0).unwrap_or(TOFT_DEPTH_M).min(TOFT_DEPTH_M))
                    .fold(f64::INFINITY, f64::min);
                if depth < TOFT_MIN_DEPTH_M {
                    continue;
                }
                // Laid out as `wallside`'s lots are: the frontage `poly[0]`-
                // `poly[1]` on the street, the back `poly[3]`-`poly[2]`.
                let quad = vec![f0, f1, f1 + nn * depth, f0 + nn * depth];
                let d = poly_centroid(&quad).dist(market);
                if d <= reach {
                    cands.push((d, quad, e.cls));
                }
            }
        }
    }
    cands.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut taken: Vec<(Vec<Vec2>, [f64; 4])> = parcels.iter().map(|p| (p.poly.clone(), bbox(&p.poly))).collect();
    let mut out: Vec<Parcel> = Vec::new();
    let mut tone = stream(seed, "village/tone");
    for (_, quad, cls) in cands {
        if have + out.len() >= need {
            break;
        }
        if !accepts(g, site, &greens, &taken, &quad) {
            continue;
        }
        let k = out.len();
        let p = Parcel {
            id: format!("toft{k}"),
            block: format!("toft{k}"),
            frontage: quad[0].dist(quad[1]),
            depth: (quad[0].dist(quad[3]) + quad[1].dist(quad[2])) / 2.0,
            area: poly_area(&quad).abs(),
            poly: quad,
            age: epochs as f64,
            edge_cls: cls,
            tone: tone.u(),
            wall_backing: WallBacking::No,
            courtyard_ring: false,
            gate_quality: None,
        };
        if !buildable(&p) {
            continue;
        }
        taken.push((p.poly.clone(), bbox(&p.poly)));
        out.push(p);
    }
    out
}

/// Distance along `dir` from `o` to the first live street other than `own`
/// within `reach`, less its half-width and [`VERGE_M`]: the depth a toft may
/// keep before it meets a street behind it. `None` when there is none.
fn street_behind(g: &Graph, own: usize, o: Vec2, dir: Vec2, reach: f64) -> Option<f64> {
    let far = o + dir * reach;
    let mut best: Option<(f64, f64)> = None;
    for eid in g.edges_near(o, far) {
        let e = &g.edges[eid];
        if !e.alive || eid == own {
            continue;
        }
        if let Some(h) = seg_int(o, far, g.nodes[e.a].pt(), g.nodes[e.b].pt())
            && best.is_none_or(|(t, _)| h.t < t)
        {
            best = Some((h.t, e.w));
        }
    }
    best.map(|(t, w)| t * reach - w / 2.0 - VERGE_M)
}

/// A point set's axis-aligned bounding box, `[minx, miny, maxx, maxy]`.
fn bbox(p: &[Vec2]) -> [f64; 4] {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for q in p {
        b = [b[0].min(q.x), b[1].min(q.y), b[2].max(q.x), b[3].max(q.y)];
    }
    b
}

/// Every test a toft must pass: a sane area, inside the box, dry by
/// `build_parcels`' own footprint rule (`lot_is_wet`), off the green, crossed
/// by no street, and overlapping no lot already taken (`wallside`'s own
/// tests, which the wall lots pass).
fn accepts(g: &Graph, site: &Site, greens: &[&[Vec2]], taken: &[(Vec<Vec2>, [f64; 4])], quad: &[Vec2]) -> bool {
    let area = poly_area(quad).abs();
    if !(26.0..=2600.0).contains(&area) || poly_self_intersects(quad) {
        return false;
    }
    if quad.iter().any(|q| q.x < 0.0 || q.y < 0.0 || q.x > site.wm || q.y > site.hm) {
        return false;
    }
    if lot_is_wet(site, quad) {
        return false;
    }
    // The channel as `cleanup::remove_water_crossings` reads it on a river
    // site (centreline within half the width and half a metre): with the
    // riverbank geometry off, `lot_is_wet` reads the stamped mask, and tofts
    // stood in the channel (the census's housing audit, `urban_riverbank` off:
    // 6 buildings at 384, 15 at 2 048, 2026-11-15).
    let rk = (site.kind == "river" || site.kind == "riverthrough") && site.river.len() >= 2;
    if rk {
        let n = quad.len();
        let c = poly_centroid(quad);
        let mut samples: Vec<Vec2> = quad.to_vec();
        samples.extend((0..n).map(|i| quad[i].lerp(quad[(i + 1) % n], 0.5)));
        samples.push(c);
        if samples.iter().any(|p| site.river_dist(*p) < site.river_w / 2.0 + 0.5) {
            return false;
        }
    }
    let c = poly_centroid(quad);
    if greens.iter().any(|p| point_in_poly(c, p) || quad.iter().any(|q| point_in_poly(*q, p))) {
        return false;
    }
    for i in 0..4 {
        let (a, b) = (quad[i], quad[(i + 1) % 4]);
        if g.edges_near(a, b).into_iter().any(|eid| {
            let e = &g.edges[eid];
            e.alive && seg_int(a, b, g.nodes[e.a].pt(), g.nodes[e.b].pt()).is_some()
        }) {
            return false;
        }
    }
    let bb = bbox(quad);
    for (p, pb) in taken {
        if pb[0] > bb[2] || pb[2] < bb[0] || pb[1] > bb[3] || pb[3] < bb[1] {
            continue;
        }
        if quad.iter().chain(std::iter::once(&c)).any(|q| point_in_poly(*q, p)) || point_in_poly(poly_centroid(p), quad) {
            return false;
        }
        // Two quads can overlap with no corner of either inside the other
        // (a cross); an edge crossing settles that case.
        let n = p.len();
        for i in 0..4 {
            for j in 0..n {
                if let Some(h) = seg_int(quad[i], quad[(i + 1) % 4], p[j], p[(j + 1) % n])
                    && h.t > 1e-6
                    && h.t < 1.0 - 1e-6
                    && h.u > 1e-6
                    && h.u < 1.0 - 1e-6
                {
                    return false;
                }
            }
        }
    }
    true
}

/// Unit tests; see its own module doc.
#[cfg(test)]
mod tests;
