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
//! ## Recorded as v2.73 — market place or village green
//!
//! **v2.73 is not a real source version.** Owner, 2026-09-28: v2.72 and v2.73
//! do not exist; v2.71 is the latest. Recorded by `6aa1ff3` (2026-09-20) from
//! an unverified source. What this module builds stands as a port-original
//! improvement, not a v2.73 port -- see `STATUS.md`.
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

/// How far a riverbank town's square keeps from the river's channel edge,
/// metres ([`riverbank_square`]). **A labelled judgement**: an embankment
/// quay and one row of riverside lots (`crate::riverbank::EMBANKMENT_W` 4 m,
/// the platting's shallowest lot of 6 m and its frontage street, about 15 m),
/// so the square fronts the town, not the water.
pub const SQUARE_RIVER_CLEAR_M: f64 = 15.0;
/// The step [`riverbank_square`] slides a square along its street, metres.
/// A labelled judgement: a third of the narrowest square (`l` from 55 m).
const SQUARE_SLIDE_M: f64 = 5.0;

/// **Not the reference** (2026-11-15, the coordinator's defect (a), riverbank
/// sites only): where a riverbank town's market square goes. The reference
/// widens the nearest primary **at its middle**, which on a real river,
/// where the market is pinned at the bridge foot on the bank, puts the square
/// a primary's half-length from the market and often across a river from the
/// town (measured: the census's walled towns at 384, 12 of 32 squares outside
/// their wall, among them seed 483920 #7, whose square stood on the far side
/// of a confluence). Here each live primary is tried from the point nearest
/// the market outward in [`SQUARE_SLIDE_M`] steps, both ways, keeping the
/// `l` by `wd` square (widened away from the river, as the reference widens)
/// whose four corners are dry, on the market's bank and at least
/// [`SQUARE_RIVER_CLEAR_M`] from the channel's edge; of those the one whose
/// centre is nearest the market. `None` (the reference's placement stands)
/// without a river or where no primary has room. Returns the street's
/// direction and the square's mid-point on it. Draws nothing from the random
/// stream, so the reference's draws are unchanged. Must never place a square
/// on the other bank or over the water.
pub fn riverbank_square(site: &Site, anchors: &Anchors, g: &Graph, l: f64, wd: f64) -> Option<(Vec2, Vec2)> {
    if site.river.len() < 2 {
        return None;
    }
    square_near_market(site, anchors, g, l, wd, SQUARE_RIVER_CLEAR_M)
}

/// How far a street village's green keeps from the river's channel edge,
/// metres ([`square_near_market`] from [`build_plaza_with`]). **A labelled
/// judgement**: an embankment's width (`crate::riverbank::EMBANKMENT_W`, 4 m),
/// so the green may come down to the landing (Ruling FE item 2: *"green at the
/// landing"*), where a chartered town's square keeps a row of lots between it
/// and the water ([`SQUARE_RIVER_CLEAR_M`]). Measured on the census at 384
/// with the town's 15 m: 287 of 647 greens found no room by a village road
/// that runs along its bank and fell back 300 m away.
pub const GREEN_RIVER_CLEAR_M: f64 = 4.0;
/// How near a live primary a village green's road-side corner must stand for
/// a short primary edge to carry it ([`square_near`] with `village`), metres.
/// **A labelled judgement**: half the 6-7 m primary's width, so the corner is
/// on the carriageway of a road that runs on straight past the edge's end.
pub const ROAD_ON_M: f64 = 3.0;
/// How near the market point a street village's green should stand, metres
/// (centre to point): `TOWN_GENERATOR_JUDGEMENT.md` change 2's own bar (*"green
/// within 60 m of the market point"*), which Ruling FE item 2 adopted as the
/// design (*"green at the landing"*). [`build_plaza_with`] shrinks a green
/// that cannot stand this near at its drawn size ([`GREEN_SHRINK`]).
pub const GREEN_NEAR_M: f64 = 60.0;
/// The smaller sizes a street village's green tries, as fractions of its
/// drawn size, in order, when the drawn size cannot stand within
/// [`GREEN_NEAR_M`] of the market point (2026-11-16). **A labelled
/// judgement**: down to two-thirds, about 37-53 m by 17-27 m (0.06-0.14 ha),
/// still a green a village gathers on; a village whose market point is at the
/// water's edge on a road that bends away (seed 483920 #140 and #207 at
/// 384 x 288, a 78 m green 82 m off) fits the smaller one by the landing.
pub const GREEN_SHRINK: [f64; 2] = [0.85, 0.67];

/// The centre of the green [`build_plaza_with`] would lay on a street of
/// direction `dir` with its road side's middle at `mid`, `l` long and `wd`
/// deep, widened away from the river as `build_plaza_with` widens it. Draws
/// nothing; must agree with the quad `build_plaza_with` builds.
fn green_centre(site: &Site, dir: Vec2, mid: Vec2, l: f64, wd: f64) -> Vec2 {
    let nl = dir.rot90();
    let side = if site.river_dist(mid + nl * 20.0) > site.river_dist(mid + nl * -20.0) { 1.0 } else { -1.0 };
    let (p1, p2) = (mid + dir * (-l / 2.0), mid + dir * (l / 2.0));
    poly_centroid(&[p1, p2, p2 + nl * (side * wd), p1 + nl * (side * wd)])
}

/// [`riverbank_square`]'s search on any site (2026-11-15, Ruling FE item 2's
/// street village, `crate::village`): the square nearest the market point on
/// a live primary, its corners dry and, where the site has a river or shore
/// line (two or more points), on the market's side of it and `clear_m` off
/// the channel's edge. Without one only the
/// dryness test applies (a landlocked site's water is its lakes). Draws
/// nothing. Must never place a square over water or across the river.
pub fn square_near_market(site: &Site, anchors: &Anchors, g: &Graph, l: f64, wd: f64, clear_m: f64) -> Option<(Vec2, Vec2)> {
    square_near(site, anchors, g, l, wd, clear_m, true, false)
}

/// [`square_near_market`], with `same_bank` false letting the square stand on
/// either bank (still dry and `clear_m` off the channel): a street village
/// whose market point is the end of its road across the river from its
/// houses (the real-water market is pinned at the box centre, 2026-11-15,
/// seed 246371 #104 at 384) has its green at the bridge's other foot, not
/// 300 m up the road. Since 2026-11-16 such a market is usually moved to its
/// roads' bank first (`crate::village::road_bank_market`), so this fallback
/// is left for markets abeam of no road. Draws nothing.
///
/// `village` (a street village's green, Ruling FE item 2) relaxes two tests
/// written for a town's square: the road-side corners need only be dry and on
/// the bank (the road is already there; the far corners keep `clear_m`), and
/// a primary edge shorter than the square counts where the road runs on
/// straight past its ends (both road-side corners within [`ROAD_ON_M`] of a
/// live primary). Why: a village road along its bank is cut into short edges
/// by the lanes joining it and stands near the water, and every green on such
/// a road fell back 300-400 m away (seed 483920 #15 at 384 x 288, 2026-11-15).
#[allow(clippy::too_many_arguments)]
pub fn square_near(site: &Site, anchors: &Anchors, g: &Graph, l: f64, wd: f64, clear_m: f64, same_bank: bool, village: bool) -> Option<(Vec2, Vec2)> {
    let has_line = site.river.len() >= 2;
    let mkt_side = if has_line { site.bank_side(anchors.market) } else { 1.0 };
    let clear = site.river_w / 2.0 + clear_m;
    // Off the channel too, whatever `clear_m`: on a site the riverbank
    // geometry did not take, `is_water` reads the stamped mask, which can call
    // a bridge deck dry, and a village green with one corner on each bank
    // laid its far street across the river (census at 384, 2026-11-15).
    let in_channel = |p: Vec2| has_line && site.river_dist(p) < site.river_w / 2.0 + 1.0;
    let bank_ok = |p: Vec2| !site.is_water(p) && !in_channel(p) && (!has_line || !same_bank || site.bank_side(p) == mkt_side);
    let ok = |p: Vec2| bank_ok(p) && (!has_line || site.river_dist(p) >= clear);
    let on_road = |p: Vec2| {
        g.edges.iter().any(|e| e.alive && e.cls == "primary" && dist_pt_seg(p, g.nodes[e.a].pt(), g.nodes[e.b].pt()) <= ROAD_ON_M)
    };
    let mut best: Option<(f64, Vec2, Vec2)> = None;
    for e in g.edges.iter().filter(|e| e.alive && e.cls == "primary") {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        let len = a.dist(b);
        if len < l && !(village && len >= SQUARE_SLIDE_M) {
            continue;
        }
        let dir = (b - a).norm();
        let nl = dir.rot90();
        // A short edge (village only) slides its square's middle along the
        // edge itself; the road-side corners are then held to the road.
        let (t_lo, t_hi) = if len >= l { (l / 2.0, len - l / 2.0) } else { (0.0, len) };
        let t0 = (anchors.market - a).dot(dir).clamp(t_lo, t_hi);
        let steps = (len / SQUARE_SLIDE_M).ceil() as i64;
        for k in 0..=steps {
            let mut found = false;
            for t in [t0 + k as f64 * SQUARE_SLIDE_M, t0 - k as f64 * SQUARE_SLIDE_M] {
                if t < t_lo || t > t_hi {
                    continue;
                }
                let mid = a + dir * t;
                let side = if site.river_dist(mid + nl * 20.0) > site.river_dist(mid + nl * -20.0) { 1.0 } else { -1.0 };
                let (p1, p2) = (mid + dir * (-l / 2.0), mid + dir * (l / 2.0));
                let (q1, q2) = (p1 + nl * (side * wd), p2 + nl * (side * wd));
                let fits = if village {
                    [p1, p2, mid].into_iter().all(bank_ok) && [q1, q2].into_iter().all(ok) && (len >= l || (on_road(p1) && on_road(p2)))
                } else {
                    [p1, p2, q1, q2, mid].into_iter().all(ok)
                };
                if fits {
                    let c = poly_centroid(&[p1, p2, q2, q1]);
                    let d = c.dist(anchors.market);
                    if best.is_none_or(|(bd, _, _)| d < bd) {
                        best = Some((d, dir, mid));
                    }
                    found = true;
                }
            }
            if found {
                break;
            }
        }
    }
    best.map(|(_, d, m)| (d, m))
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
    build_plaza_with(seed, site, anchors, g, pop, false, false)
}

/// The research's settled-area law, `settled ha = a * N^b`: Cesaretti et al.
/// on 173 cities (`MEDIEVAL_TOWN_GROWTH_RESEARCH.md` rule 1, which the
/// judgement rates **strong**), `a` = 0.12, `b` = 0.714. [`square_scale`]
/// sizes the market square as a share of it, so the square grows as a town's
/// settled area grows, not as its head count.
pub const SETTLED_LAW: (f64, f64) = (0.12, 0.714);

/// A chartered town's market square as a share of [`SETTLED_LAW`]'s settled
/// area for its `pop_target` ([`square_scale`]). **A labelled judgement**
/// (`TOWN_GENERATOR_JUDGEMENT.md` change 6: *"about 0.5 to 1 % of the built
/// area for an organic town, and I do not claim a source for it"*;
/// `MEDIEVAL_TOWN_GROWTH_RESEARCH.md` section 7 found no sourced ratio). The
/// generator's towns settle about 0.4 to 0.6 of the law's area (the
/// judgement's measure), so 0.4 % of the law is about 0.7 to 1 % of what a
/// town really covers: 0.09 ha at 1 500 people, 0.21 ha at 5 000 (the
/// reference's fixed draw), 0.56 ha at 20 000.
pub const SQUARE_LAW_SHARE: f64 = 0.004;

/// The mean area of the reference's two draws, `range(55, 80)` by
/// `range(26, 40)`: 67.5 x 33 m. **The reference's** numbers; [`square_scale`]
/// scales both draws by one factor, so the mean square is the target area and
/// each town keeps its own drawn proportions.
const DRAW_MEAN_AREA_M2: f64 = 67.5 * 33.0;

/// How much smaller each next try is when a scaled square finds no room
/// ([`build_plaza_with`]). **A labelled judgement**: about one seventh off the
/// area a step, so at most four steps bring the largest city's square back to
/// the reference's size.
const SQUARE_SHRINK: f64 = 0.85;

/// **Not the reference** (`TOWN_GENERATOR_JUDGEMENT.md` change 6, 2026-11-15,
/// behind `GenOpts::market_scale`): the factor both of `build_plaza`'s draws
/// are multiplied by for a chartered town, so the square's area is
/// [`SQUARE_LAW_SHARE`] of [`SETTLED_LAW`]'s area for `pop`, on average over
/// the draws. Why: the reference's square is one size, about 0.2 ha, from a
/// hamlet of 400 to a city of 20 000, which the judgement measured at 0.28 %
/// of a large town's built area. `pop` is the `pop_target` (`build_civic`'s,
/// not the realised count). Must never be read with the switch off, nor for a
/// village (below).
///
/// **A village keeps its green** ([`build_plaza_with`] does not scale it):
/// the street village's green of `9cd4c7c7` (Ruling FE item 2) is kept. A
/// green scaled the same way (1 % of the law, 0.09 ha at 400 people) met the
/// judgement's 4 % bar (23 of 648 villages over it against 160) but left 115
/// villages under 80 % of their target against 91 on the census at 384
/// (three seeds, `docs/evidence/waterfront-square-2026-11-15/`): fewer back
/// lanes grew off the smaller green's streets. Routed, not shipped.
///
/// **Nor is a square shrunk**: a town whose factor is under 1 (under about
/// 5 300 people) keeps the reference's square, size and place.
/// `TOWN_GENERATOR_JUDGEMENT.md`'s bar is set for towns of 5 000 and up, the
/// reference's square is already 0.8 to 2 % of a smaller town's settled area,
/// and a smaller square moved where such a town grew, by chance worse in the
/// one frame of it looked at (seed 12345 #0 at 384 x 288, 2 039 people: more
/// of its kept wall's interior left unbuilt).
pub fn square_scale(pop: f64) -> f64 {
    let law_m2 = SETTLED_LAW.0 * pop.max(1.0).powf(SETTLED_LAW.1) * 1e4;
    (SQUARE_LAW_SHARE * law_m2 / DRAW_MEAN_AREA_M2).sqrt()
}

/// The factors [`build_plaza_with`] tries for a square scaled by `k`, in
/// order: `k` itself, then [`SQUARE_SHRINK`] times the last while that is
/// still over 1, then 1 (the reference's size). A square at or under the
/// reference's size is tried once: a smaller square fits wherever a larger
/// one would.
fn scale_tries(k: f64) -> Vec<f64> {
    let mut out = vec![k];
    let mut s = k;
    while s > 1.0 {
        s = (s * SQUARE_SHRINK).max(1.0);
        out.push(s);
    }
    out
}

/// [`build_plaza`], with `green_at_market` (**not the reference**; Ruling FE
/// item 2, 2026-11-15, `crate::village`): a street village's green is widened
/// on the primary at the point nearest the market point
/// ([`square_near_market`]), the landing or the crossroads, instead of at the
/// nearest primary's middle. `false` is [`build_plaza`] exactly. The two draws
/// are the same either way, so no other stream moves.
///
/// `scale` (**not the reference**; `TOWN_GENERATOR_JUDGEMENT.md` change 6,
/// 2026-11-15, `GenOpts::market_scale`): for a chartered town whose
/// [`square_scale`] is over 1 (a village's green, and a smaller town's square,
/// are never scaled, see [`square_scale`]) both draws are multiplied by [`square_scale`]`(pop)`,
/// and the square is placed by the searches below,
/// each tried at [`scale_tries`]' sizes from the largest down, the first that
/// fits winning. A chartered town on a site neither search above takes (no
/// riverbank geometry) is placed by [`square_near_market`] too, nearest the
/// market point on a live primary, because a square larger than the
/// reference's no longer fits on any edge the reference's middle-of-the-edge
/// rule picks; if none fits, the reference's placement stands at the
/// reference's size. The draws are still exactly
/// two, so no other stream moves. `false` changes nothing.
pub fn build_plaza_with(seed: u32, site: &Site, anchors: &Anchors, g: &mut Graph, pop: f64, green_at_market: bool, scale: bool) -> Option<Plaza> {
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
    let mut dir = (b - a).norm();
    let mut mid = a.lerp(b, 0.5);
    // European market place band (M-DEN-6). Two draws, in declaration order.
    let l0 = r.range(55.0, 80.0);
    let wd0 = r.range(26.0, 40.0);
    // Change 6 (see the doc comment): the sizes to try, largest first. With
    // the switch off, the reference's one size.
    // A village keeps the reference's green, and a town whose law-sized
    // square would be smaller than the reference's keeps the reference's
    // (see `square_scale`).
    let scale = scale && pop >= PLAZA_MARKET_POP && square_scale(pop) > 1.0;
    let k = if scale { square_scale(pop) } else { 1.0 };
    let tries = if scale { scale_tries(k) } else { vec![1.0] };
    let (mut l, mut wd) = (l0 * tries[0], wd0 * tries[0]);
    // Whether a search below placed the square; only with `scale` does the
    // unplaced case change size (to `min(k, 1)` at the reference's place).
    let mut placed = false;
    // **Not the reference** (2026-11-15, riverbank sites only): a chartered
    // town's market square beside the market, on its bank, rather than at
    // the middle of the nearest primary (`riverbank_square`). Not a village
    // green: a village has no wall to keep it in, and moving its green moved
    // where 24 more villages grew, under 80 % of their target (the census at
    // 2 048, 3 seeds, measured with the green moved too).
    if site.riverbank() && pop >= PLAZA_MARKET_POP {
        for &t in &tries {
            if let Some((d, m)) = riverbank_square(site, anchors, g, l0 * t, wd0 * t) {
                (dir, mid, l, wd, placed) = (d, m, l0 * t, wd0 * t, true);
                break;
            }
        }
    }
    // Ruling FE item 2 (see `build_plaza_with`): a street village's green at
    // the market point. The street budget changes with it
    // (`crate::village::STREET_PER_HEAD_M`) and the tofts fill to the target
    // (`crate::village::plat_tofts`), which is what the recorded attempt above
    // lacked: it moved the green alone and the old growth followed it.
    if green_at_market && pop < PLAZA_MARKET_POP {
        // 2026-11-16 (the town leftovers): the reference's green first; where
        // it can stand no nearer the market point than `GREEN_NEAR_M`, the
        // smaller sizes of `GREEN_SHRINK` in turn, taking the first that can;
        // if none can, the reference's size where it fitted.
        let mut first: Option<(Vec2, Vec2, f64, f64)> = None;
        for &t in tries.iter().chain(GREEN_SHRINK.iter()) {
            let (lt, wt) = (l0 * t, wd0 * t);
            if let Some((d, m)) = square_near(site, anchors, g, lt, wt, GREEN_RIVER_CLEAR_M, true, true)
                .or_else(|| square_near(site, anchors, g, lt, wt, GREEN_RIVER_CLEAR_M, false, true))
            {
                let near = green_centre(site, d, m, lt, wt).dist(anchors.market) <= GREEN_NEAR_M;
                if near {
                    (dir, mid, l, wd, placed) = (d, m, lt, wt, true);
                    first = None;
                    break;
                }
                first.get_or_insert((d, m, lt, wt));
            }
        }
        if let Some((d, m, lt, wt)) = first {
            (dir, mid, l, wd, placed) = (d, m, lt, wt, true);
        }
    }
    // Change 6: a scaled chartered town's square that neither search above
    // placed (a site without the riverbank geometry) stands nearest the market
    // point on a primary it fits on; else at the reference's place and size.
    if scale && !placed && pop >= PLAZA_MARKET_POP && !site.riverbank() {
        for &t in &tries {
            if let Some((d, m)) = square_near_market(site, anchors, g, l0 * t, wd0 * t, SQUARE_RIVER_CLEAR_M) {
                (dir, mid, l, wd, placed) = (d, m, l0 * t, wd0 * t, true);
                break;
            }
        }
    }
    if scale && !placed {
        (l, wd) = (l0, wd0);
    }

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

/// Milestone 8a's golden and unit tests — see `plaza/tests.rs`'s own module
/// doc for the scenario catalogue.
#[cfg(test)]
mod tests;
