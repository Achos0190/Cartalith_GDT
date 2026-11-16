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
    // Riverside tofts cut back at the bank ([`bank_behind`]): a riverbank
    // site with a river line only, where the channel is exactly known.
    let bank_cut = site.riverbank() && (site.kind == "river" || site.kind == "riverthrough") && site.river.len() >= 2;
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
                // A road along the bank (riverbank river sites): the toft on
                // its river side is cut back at the water ([`bank_behind`]),
                // so the village fronts its river instead of leaving the strip
                // between the road and the water empty.
                let depth = if bank_cut { [f0, f1].into_iter().map(|f| bank_behind(site, f, nn, depth)).fold(depth, f64::min) } else { depth };
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

/// How far from the channel's edge a street village's market point stands
/// after [`landing_market`] moved it to its river, metres. **A labelled
/// judgement**: a toft's depth ([`TOFT_DEPTH_M`]) and a verge, so the green
/// (a road-side square about 26-40 m deep, `plaza::build_plaza_with`) and the
/// first tofts reach down to the water, the judgement's *"green where the road
/// meets the water"*.
pub const LANDING_M: f64 = 32.0;

/// **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 5's *"no river
/// town with every house over two widths away"*, for street villages; rides
/// `urban_street_village`): where a street village's market point goes when
/// the box centre stands inland of the village's river, or `None` when it
/// stays.
///
/// **Why.** The real-water market is pinned at the box centre, and on a fine
/// grid (2 048 cells across 800 km, 390 m a cell) a village the siting proxy
/// calls a river village can have its centre 200-500 m from the river in its
/// own 1.7 km box: its tofts, taken nearest the market first, then never come
/// near the water (census at 2 048, 2026-11-16: 80 river villages with every
/// house over two street widths from the bank, median 290 m).
///
/// **What.** The nearest point of the river's centreline (`site.river`) is
/// found; when `m`'s distance from the channel's edge (that distance less half
/// `river_w`) is more than [`LANDING_M`], the market moves straight toward
/// that point until it stands [`LANDING_M`] from the edge, on its own bank.
/// The caller's ring search then sets it on dry ground a dry line joins to a
/// road, as it does for Ruling FK's harbour front. Never closer than
/// [`LANDING_M`]; `None` without a river line or when already that close.
/// Draws nothing. **Must never** move a market across the river (the line
/// runs toward the nearest centreline point, so it stops on `m`'s side) or
/// further from the river than it was.
pub fn landing_market(site: &Site, m: Vec2) -> Option<Vec2> {
    if site.river.len() < 2 {
        return None;
    }
    let mut best: Option<(f64, Vec2)> = None;
    for w in site.river.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let l2 = ab.dot(ab);
        let t = if l2 > 0.0 { ((m - a).dot(ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let q = a + ab * t;
        let d = q.dist(m);
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, q));
        }
    }
    let (d, q) = best?;
    let stand = site.river_w / 2.0 + LANDING_M;
    if d <= stand {
        return None;
    }
    Some(q + (m - q).norm() * stand)
}

/// How far from the market point the host's roads are weighed by
/// [`road_bank_market`], as a multiple of `generate`'s `max_rf` (about 200 m
/// for 400 people). **A labelled judgement**: the urbanisation front itself,
/// the ground the village's first houses take ([`plat_tofts`] goes nearest
/// the market first), so a long road on the far side of the box does not
/// outvote the road the village actually stands on.
pub const ROAD_BANK_REACH_RF: f64 = 1.0;

/// **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 2's green bar,
/// the leftover of Ruling FE item 2; rides `urban_street_village`): where a
/// street village's market point goes when the box centre stands across the
/// river from its roads.
///
/// **Why.** The real-water market is pinned at the box centre (v0.98), and a
/// river village's map cell can put the centre on the bank across the bridge
/// from the road the village lines: the market point was then the road's end
/// over the river, its houses all on the other bank, and the green at the
/// bridge's far foot 60-120 m from it (census at 384, 2026-11-15: about a
/// quarter of villages missed the judgement's 60 m bar this way; seed 912345
/// #136 and 246371 #44 at 384 x 288 are the plain cases). Or, with no road on
/// its bank at all, the village barely grew (246371 #157: 31 of 400).
///
/// **What.** The host's roads (`paths`, offsets from `origin`, the map pin:
/// the market as the reference placed it; their points inside the box, as
/// `crate::routes::build_primaries_from_paths` lays them) are measured within
/// [`ROAD_BANK_REACH_RF`]` * max_rf` of `m`, segment by segment on the bank
/// of each segment's midpoint, channel segments left out. When the other
/// bank holds more road there than the market's own, the market goes to the
/// nearest ring point (every 30 m out, 24 bearings, the reference's own
/// channel-nudge search) on that bank that is dry, clear of the channel and
/// joined by a dry straight line to one of the roads
/// (`crate::routes::dry_join`), so it lands at the road's landing on the
/// village's side. `None` (the market stays) on a site with no river line,
/// where the market's bank holds as much road as the other, or where no such
/// point is found within the reach. Draws nothing.
///
/// **Must never** move a market onto the water or into the channel, onto the
/// bank holding less road, or for a town (`generate_pass` calls it only for a
/// street village on a real-water river site).
pub fn road_bank_market(site: &Site, m: Vec2, origin: Vec2, paths: &[Vec<Vec2>], max_rf: f64) -> Option<Vec2> {
    if site.river.len() < 2 {
        return None;
    }
    // Across the river only where the river runs past: beyond either end of
    // the line "the other bank" is the side of its extension, not water to
    // cross (seed 483920 #105 and #193 in the app's world at 384 x 288,
    // 2026-11-16: a market moved round a river's end, its green 197 m off and
    // a hamlet housing 10 of 400).
    if !abeam(site, m) {
        return None;
    }
    let reach = ROAD_BANK_REACH_RF * max_rf;
    let in_box = |p: Vec2| p.x >= -6.0 && p.x <= site.wm + 6.0 && p.y >= -6.0 && p.y <= site.hm + 6.0;
    let wet = |p: Vec2| site.is_water(p) || site.river_dist(p) < site.river_w / 2.0 + 1.0;
    let mkt_side = site.bank_side(m);
    let roads: Vec<Vec<Vec2>> = paths
        .iter()
        .map(|path| path.iter().map(|o| Vec2::new(origin.x + o.x, origin.y + o.y)).take_while(|&p| in_box(p)).collect::<Vec<_>>())
        .filter(|pts| pts.len() >= 2)
        .collect();
    let (mut own, mut other) = (0.0, 0.0);
    for w in roads.iter().flat_map(|pts| pts.windows(2)) {
        let mid = w[0].lerp(w[1], 0.5);
        if mid.dist(m) > reach || wet(mid) || !abeam(site, mid) {
            continue;
        }
        if site.bank_side(mid) == mkt_side {
            own += w[0].dist(w[1]);
        } else {
            other += w[0].dist(w[1]);
        }
    }
    if other <= own {
        return None;
    }
    // The roads' dry points on the other bank, for the join test.
    let far_roads: Vec<Vec<Vec2>> =
        roads.iter().map(|pts| pts.iter().copied().filter(|&p| !wet(p) && site.bank_side(p) != mkt_side).collect::<Vec<_>>()).filter(|p| !p.is_empty()).collect();
    let mut rr = 30.0;
    while rr <= reach {
        for a in 0..24 {
            let ang = a as f64 / 24.0 * std::f64::consts::PI * 2.0;
            let q = Vec2::new(m.x + crate::geom::js_cos(ang) * rr, m.y + crate::geom::js_sin(ang) * rr);
            if q.x < 25.0 || q.y < 25.0 || q.x > site.wm - 25.0 || q.y > site.hm - 25.0 {
                continue;
            }
            if wet(q) || site.bank_side(q) == mkt_side || !abeam(site, q) {
                continue;
            }
            if far_roads.iter().any(|pts| crate::routes::dry_join(site, q, pts).is_some()) {
                return Some(q);
            }
        }
        rr += 30.0;
    }
    None
}

/// How far a riverside toft's back stays from the channel's edge, metres
/// ([`bank_behind`]). **A labelled judgement**: a metre, so the plot ends at
/// the water's edge (a house with its yard to the river, Ruling EZ item 1:
/// *"Riverside houses are fine"*) and clears `blocks::lot_is_wet`'s own
/// margin on a river site (half the width plus a metre), which [`accepts`]
/// applies to every toft.
pub const TOFT_BANK_CLEAR_M: f64 = 1.0;

/// **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 5's *"no river
/// town with every house over two widths away"*, for street villages; rides
/// `urban_street_village`): the depth a toft fronting at `o` and running
/// along `dir` may keep before it reaches [`TOFT_BANK_CLEAR_M`] off the
/// channel or open water, at most `depth`; `depth` itself where the water is
/// further. Stepped a metre at a time: the last whole metre still clear, so
/// the back line stands at least [`TOFT_BANK_CLEAR_M`] off the edge. Why: a village whose road runs along
/// its bank had every toft on the road's land side, since the full-depth toft
/// on the river side stood in the water and was refused (census at 384 and
/// 2 048, 2026-11-15: 6 and 77 river villages with every house over two
/// street widths from the bank). Must never return more than `depth`.
fn bank_behind(site: &Site, o: Vec2, dir: Vec2, depth: f64) -> f64 {
    let clear = site.river_w / 2.0 + TOFT_BANK_CLEAR_M;
    let mut t = 0.0;
    while t <= depth {
        let p = o + dir * t;
        if site.is_water(p) || site.river_dist(p) < clear {
            // The last step that was still clear (0 when the front itself is
            // not, which `plat_tofts`' depth floor then refuses).
            return (t - 1.0).max(0.0);
        }
        t += 1.0;
    }
    depth
}

/// Whether `p` stands abeam of the site's river line: its nearest point on
/// the line is not one of the line's two ends (a projection past the first
/// segment's start or the last one's end). Where it is an end, `p` lies
/// beyond the river and [`Site::bank_side`] reads the side of the line's
/// extension, not a bank. For [`road_bank_market`]; `false` with fewer than
/// two points.
fn abeam(site: &Site, p: Vec2) -> bool {
    let r = &site.river;
    if r.len() < 2 {
        return false;
    }
    let mut best = (f64::INFINITY, 0.0);
    let last = r.len() - 2;
    for (k, w) in r.windows(2).enumerate() {
        let ab = w[1] - w[0];
        let l2 = ab.dot(ab);
        let t = if l2 > 0.0 { (p - w[0]).dot(ab) / l2 } else { 0.0 };
        let q = w[0] + ab * t.clamp(0.0, 1.0);
        let d = q.dist(p);
        if d < best.0 {
            // How far past an end of the whole line the projection falls
            // (0 when it falls on the line).
            let past = if k == 0 && t < 0.0 { -t } else if k == last && t > 1.0 { t - 1.0 } else { 0.0 };
            best = (d, past);
        }
    }
    best.1 == 0.0
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

/// The road-bank market, the landing and the riverside toft cut (2026-11-16,
/// the town leftovers); see its own module doc.
#[cfg(test)]
mod landing_tests;
