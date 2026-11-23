//! **Not the reference** -- a river town builds to its riverbank (2026-11-15,
//! under Ruling EX's follow-ups; owner, 2026-11-14: *"per town placement is
//! critical. If it misses it just looks silly"*). Behind
//! [`crate::GenOpts::riverbank`], off by default, so every JS-captured golden
//! is unmoved by control flow (`DECISIONS.md` section 7p).
//!
//! **What was wrong.** Measured over the app's worlds (`cartalith-godot`'s
//! `riverbank_gap_audit`, `docs/evidence/riverbank-2026-11-15/`): on every
//! judged river town the nearest house stood further from the river's bank
//! than the town's widest street. Two causes, both the reference's:
//!
//! 1. **The real-water path counts the river's half-width twice.** `_umWaterCtx`
//!    stamps the river into a 22 m mask as a band `riverW/2 + cellM/2` wide each
//!    side and `buildSite`'s real-water `riverDist` is the distance to that
//!    band's edge, while every margin that reads it (`riverW/2 + 1`, `riverW/2
//!    + 1.5`, `riverW/2 + 30`, ...) was written for the synthetic site, whose
//!    `riverDist` is the distance to the centreline. The reference's own river
//!    quay (`buildHarbour`, `riverW/2 + 4` from the centreline) lies inside the
//!    stamped band and is dropped as wet from order 2 up. Measured, the gap
//!    does not grow with the width, so the 22 m quantisation and cause 2
//!    dominate; the doubling is real in the code all the same. Fixed by
//!    [`crate::site::Site::use_riverbank`], which measures the river from its
//!    channel.
//! 2. **Nothing closes the faces against the river.** Blocks are the faces of
//!    the street graph ([`crate::build_blocks`]); the streets that run toward
//!    the river stop short of it and no street runs along it, so the ground
//!    between the last cross street and the water is never a face and is never
//!    built on. [`build_embankment`] lays a riverside street (an embankment
//!    quay) along each bank the town reaches and ties the streets that end near
//!    it onto it, so those faces close and are platted like any other block.
//!
//! A third, found while measuring: a traced river reaches the box as vertices
//! a map cell apart, and the reference's stages that walk the centreline (the
//! river port, the harbour's shore) were written for ~30 m vertices. Once the
//! quay was no longer dropped as wet it ran the whole 4 km path; so a riverbank
//! site's centreline is clipped and resampled first ([`fine_river_path`]).

use crate::districts::TERRAIN_BUILD_MIN;
use crate::geom::{Vec2, dist_pt_seg, js_or, point_in_poly, seg_int, simplify};
use crate::graph::Graph;
use crate::plaza::PLAZA_MARKET_POP;
use crate::rng::stream;
use crate::routes::Anchors;
use crate::site::{Site, terrain_suitability};

/// The embankment's width, metres. **The reference's** narrowest street laid
/// along a shore: `buildHarbour`'s back street and herringbone stubs (4.0), a
/// lane-scale riverside street rather than a harbour's 7 m quay.
pub const EMBANKMENT_W: f64 = 4.0;

/// The gap between the channel's edge and the embankment's water side,
/// metres. **The reference's** own lot margin from the water, `riverW/2 + 1`
/// ([`crate::blocks::wet_margin`]), applied to the street that fronts it.
pub const EMBANKMENT_CLEAR: f64 = 1.0;

/// How far from a bank point the town's own streets may end and still make
/// that stretch of bank part of the town, metres; also the longest tie the
/// embankment makes to a street that ends near it. **The reference's** block
/// depth along a shore: `buildHarbour` lays its back street 36-46 m inland of
/// the quay, and this is the deep end of that draw.
pub const EMBANKMENT_REACH: f64 = 46.0;

/// The width of a [`BANK_CLASS`] edge, metres: the line the riverside houses
/// stand on where they come down to the water with no street between
/// (Ruling EZ item 1). **A labelled judgement**: it is not a carriageway but
/// the revetment's crest, a metre wide, so the block inset
/// (`w/2 + 1.4`, `crate::blocks::build_blocks`) puts the house front about
/// 3.4 m off the channel's edge, inside one street width of it.
pub const BANK_W: f64 = 1.0;

/// The edge class of a riverside stretch where the houses come down to the
/// water (Ruling EZ item 1: *"Riverside houses are fine, as well as having a
/// street in between the river and house. So both conditions can exist"*).
/// **Not a street**: it closes the riverside faces into blocks exactly as the
/// embankment quay does, and the lots platted along it front the water, but
/// no renderer draws it (`urban_layout_draw.gd`'s `DRAW_ORDER` names the
/// street classes it draws, and `cartalith-godot`'s `urban_bridge` leaves this
/// class out of the exported streets).
pub const BANK_CLASS: &str = "bank";

/// The shortest and longest riverside stretch laid in one form before the
/// frontage turns to the other (houses on the water, or houses behind an
/// embankment street), metres. **A labelled judgement** of the owner's
/// Amsterdam reference, where both forms alternate along one water over a
/// block or two: the engine's blocks are 60-150 m along a street (`grow`'s
/// 30-125 m segments, `parallelStreetSpacing`), so 90-220 m is one to two
/// blocks of each.
pub const FRONTAGE_STRETCH_M: (f64, f64) = (90.0, 220.0);

/// The smallest `pop_target` that grows across its river (Ruling FB: *"Big
/// river towns can grow around both banks when the terrain allows it"*).
/// **The urban crate's own size class**, [`PLAZA_MARKET_POP`]: the line
/// between a village (a green, no market right) and a chartered town. A
/// village keeps the reference's one-bank layout and bridgehead, so the
/// change cannot add to the villages' over-build.
pub const BOTH_BANKS_MIN_POP: f64 = PLAZA_MARKET_POP;

/// How far from a town bridge's near end the street it is joined to may be,
/// metres. **The reference's** own bridge approach reach, `addRiverBridges`'
/// `Math.min(end.dist(market), 120)` stub, doubled for a joined street rather
/// than a stub, so a bridge 200 m from the nearest street is not joined
/// across open ground (`crate::water::add_river_bridges`).
pub const BRIDGE_JOIN_M: f64 = 240.0;

/// The length of the street a town bridge's far end carries on, away from
/// the river, metres: the stub `crate::growth::grow_far_bank` first taps on the far bank. **The
/// reference's** shortest street segment kept clear of the short-segment
/// tail is 38 m (`grow`'s mid-edge tap floor); 60 m is that plus room for
/// one tap either side of the middle, and half `addRiverBridges`' 120 m
/// approach cap. A labelled judgement within those two figures.
pub const BRIDGE_FAR_STUB_M: f64 = 60.0;

/// The narrowest a **river** is laid out, metres: the reference's own floor
/// (`um_water_ctx`'s `max(12, min(46, 10 + 7 * order))`). A real watercourse
/// laid out narrower than this is a **stream** (RR-T2, Ruling HE): no quay
/// street ([`build_embankment_with`]) and no river port (`build_harbour`).
pub const RIVER_MIN_W_M: f64 = 12.0;

/// The width a stream is laid out at, metres (RR-T2, Ruling HE). **A labelled
/// judgement**: headwater streams are a few metres across (order-1 channels of
/// 1 to 5 m; the exact figure is UNCONFIRMED, not sourced), well under
/// [`RIVER_MIN_W_M`], so the town draws a stream a street can step across.
pub const STREAM_WIDTH_M: f64 = 5.0;

/// The step a real river's centreline is resampled at ([`fine_river_path`]),
/// metres. **The reference's** plot frontage, ~8 m (`generate`'s M-DEN-1/2,
/// M-PAR-1 note), so no lot's frontage falls between two bank points.
pub const RIVER_STEP: f64 = 8.0;

/// How far beyond the town box a real river's centreline is kept
/// ([`fine_river_path`]), metres. **A labelled judgement** set from the
/// reference's own reach: the furthest any stage reads `riverDist` from is
/// `placeAnchors`' flood score (`rd - 260`, `crate::routes`), so 300 m keeps
/// every distance a stage takes inside the box the same as on the whole path.
pub const RIVER_CLIP_MARGIN: f64 = 300.0;

/// A real river's centreline as the riverbank geometry lays a town on it:
/// every original vertex kept, and the stretch of each segment inside the town
/// box grown by [`RIVER_CLIP_MARGIN`] resampled every [`RIVER_STEP`] (so the
/// line itself does not move, and a river that leaves the grown box and comes
/// back keeps its real course between: no chord is drawn across the gap).
///
/// **Why.** A traced river reaches the box as a handful of vertices one map
/// cell apart (2 km at 384 cells), several of them kilometres outside the box,
/// while every reference stage that walks the centreline was written for the
/// synthetic river's ~30 m vertices: `buildSite` puts the river port "a little
/// downstream of the bridge" by walking vertices until 95 m, and
/// `buildHarbour` gathers 115-150 m of shore the same way, so on a coarse path
/// one step is a whole map cell and the quay ran the length of the path, 4 km,
/// outside the box (seed 483920 at 384 x 288, settlement #36: 1 220 harbour
/// houses for a town of 400). The reference never showed it only because its
/// quay was dropped as wet on those rivers ([`crate::site::Site::use_riverbank`]).
///
/// Returns `None` (keep the path as given) when no segment comes within the
/// grown box: no river runs through or near it. Must never move or drop a
/// vertex of the traced path.
pub fn fine_river_path(path: &[Vec2], wm: f64, hm: f64) -> Option<Vec<Vec2>> {
    let (x0, y0, x1, y1) = (-RIVER_CLIP_MARGIN, -RIVER_CLIP_MARGIN, wm + RIVER_CLIP_MARGIN, hm + RIVER_CLIP_MARGIN);
    let mut out: Vec<Vec2> = Vec::new();
    let mut near = false;
    let mut push = |out: &mut Vec<Vec2>, p: Vec2| {
        if out.last().is_none_or(|l: &Vec2| l.dist(p) > 1e-9) {
            out.push(p);
        }
    };
    for w in path.windows(2) {
        push(&mut out, w[0]);
        // Liang-Barsky: the part of the segment inside the grown box.
        let (a, d) = (w[0], w[1] - w[0]);
        let (mut t0, mut t1) = (0.0f64, 1.0f64);
        let mut inside = true;
        for (p, q) in [(-d.x, a.x - x0), (d.x, x1 - a.x), (-d.y, a.y - y0), (d.y, y1 - a.y)] {
            if p == 0.0 {
                if q < 0.0 {
                    inside = false;
                }
            } else {
                let t = q / p;
                if p < 0.0 {
                    t0 = t0.max(t);
                } else {
                    t1 = t1.min(t);
                }
            }
        }
        if !inside || t0 > t1 {
            continue;
        }
        near = true;
        let (s, e) = (a + d * t0, a + d * t1);
        let k = (s.dist(e) / RIVER_STEP).ceil().max(1.0) as usize;
        for j in 0..=k {
            push(&mut out, s.lerp(e, j as f64 / k as f64));
        }
    }
    if let Some(&last) = path.last() {
        push(&mut out, last);
    }
    (near && out.len() >= 2).then_some(out)
}

/// The provenance string every embankment edge carries.
const EMBANKMENT_PROV: &str = "Riverbank embankment (not the reference; 2026-11-15, Ruling EX's follow-ups): \
     the river town's street along its bank, so the riverside faces close into blocks.";

/// The provenance string of a [`BANK_CLASS`] edge.
const BANK_PROV: &str = "Riverside frontage (not the reference; 2026-11-15, Ruling EZ item 1): the      houses come down to the water here, with no street between them and the river.";

/// The provenance string of a bridge [`lay_town_bridge`] lays.
const TOWN_BRIDGE_PROV: &str = "Town bridge (not the reference; 2026-11-15, Ruling FB): a big river      town spans its river at the bridge point, so it can grow on both banks.";

/// The provenance string of a town bridge's approach streets.
const TOWN_BRIDGE_APPROACH_PROV: &str = "Town bridge approach (not the reference; 2026-11-15, Ruling FB):      the bridge joined to the town's streets on its own bank and carried onto the far bank.";

/// The provenance string of a tie from a street's end to the embankment.
const TIE_PROV: &str = "Riverbank tie (not the reference; 2026-11-15): a street that ended near the \
     bank, carried onto the embankment.";

/// What [`build_embankment`] laid: each bank run's polyline (before the
/// graph's planar snapping, as `buildHarbour` returns its quay) and how many
/// street ends it tied on. Empty when the town reaches neither bank.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Embankment {
    /// Each embankment quay stretch as laid, simplified, before `add_street`
    /// snapped it.
    pub runs: Vec<Vec<Vec2>>,
    /// Each stretch where the houses come down to the water ([`BANK_CLASS`]),
    /// as laid. Empty when `build_embankment` was asked for quays only.
    pub banks: Vec<Vec<Vec2>>,
    /// Street ends tied onto the embankment.
    pub ties: usize,
    /// Houses `generate` swept because they reached into the water
    /// ([`crate::site::Site::poly_in_water`]); filled in after the last house
    /// stage, 0 from [`build_embankment`] itself.
    pub wet_swept: usize,
    /// Field and pasture strips `generate` swept for crossing the water; 0
    /// from [`build_embankment`] itself.
    pub fields_swept: usize,
    /// Each waterfront back street [`lay_back_street`] laid (change 5 of
    /// `TOWN_GENERATOR_JUDGEMENT.md`), as laid, simplified. Empty from
    /// [`build_embankment`] itself and for a town it is not asked for.
    pub back_streets: Vec<Vec<Vec2>>,
    /// Water lanes [`lay_back_street`] laid from its back streets down to the
    /// bank.
    pub water_lanes: usize,
}

/// Which side of the centreline `p` lies on: `+1.0` left of the nearest
/// segment's direction, `-1.0` right, `0.0` on the line (or for a NaN point).
/// Must never be called on a centreline of fewer than two points (the caller
/// returns first).
fn side_of(site: &Site, p: Vec2) -> f64 {
    let r = &site.river;
    let i = site.nearest_river_seg(p);
    let (a, b) = (r[i], r[(i + 1).min(r.len() - 1)]);
    let c = (b - a).cross(p - a);
    if c > 0.0 {
        1.0
    } else if c < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// The nearest point to `p` on segment `a`-`b` (`a` for a zero-length one).
/// Must never return a point off the segment.
fn closest_on_seg(p: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let d = b - a;
    let l2 = d.dot(d);
    if l2 <= 0.0 {
        return a;
    }
    let t = ((p - a).dot(d) / l2).clamp(0.0, 1.0);
    a.lerp(b, t)
}

/// Is the segment `a`-`b` dry land on bank `side`, inside the box, and far
/// enough from the channel that a street of [`EMBANKMENT_W`] laid on it keeps
/// its paved edge [`EMBANKMENT_CLEAR`] off the water? Six probes, as the
/// reference's `dry()` (`crate::water`) takes them. The 1e-6 m slack lets the
/// embankment itself, laid at exactly that offset, pass on a straight run.
/// Must never accept a probe on the other bank (the side check, tested) or in
/// the water.
fn dry_on_side(site: &Site, a: Vec2, b: Vec2, side: f64) -> bool {
    dry_on_side_w(site, a, b, side, EMBANKMENT_W)
}

/// [`dry_on_side`] for an edge of width `w` (the embankment's, or a
/// [`BANK_CLASS`] edge's [`BANK_W`]): its paved edge must keep
/// [`EMBANKMENT_CLEAR`] off the water.
fn dry_on_side_w(site: &Site, a: Vec2, b: Vec2, side: f64, w: f64) -> bool {
    let clear = site.river_w / 2.0 + EMBANKMENT_CLEAR + w / 2.0 - 1e-6;
    (0..=5).all(|k| {
        let p = a.lerp(b, k as f64 / 5.0);
        p.x >= 0.0
            && p.y >= 0.0
            && p.x <= site.wm
            && p.y <= site.hm
            && !site.is_water(p)
            && site.river_dist(p) >= clear
            && side_of(site, p) == side
    })
}

/// Does segment `a`-`b` cross the closed ring `wall`?
fn crosses_wall(a: Vec2, b: Vec2, wall: Option<&[Vec2]>) -> bool {
    wall.is_some_and(|r| (0..r.len()).any(|k| seg_int(a, b, r[k], r[(k + 1) % r.len()]).is_some()))
}

/// Lay the river town's embankment and tie the streets that end near it onto
/// it ([`crate::riverbank`] for why).
///
/// For each bank (`side` +1 and -1), every centreline vertex (one every
/// [`RIVER_STEP`] on a riverbank site) gets a bank point
/// `riverW/2 + EMBANKMENT_CLEAR + EMBANKMENT_W/2` out along its normal; a bank
/// point belongs to the town when a live `street` or `lane` node on that bank
/// lies within [`EMBANKMENT_REACH`] of it. Each run of consecutive town bank
/// points becomes a riverside edge through the graph's own
/// [`Graph::add_street`], segment by segment, keeping only dry segments on
/// their own bank. Then every `street` or `lane` dead end on a bank within
/// [`EMBANKMENT_REACH`] of that bank's run is tied to its nearest point with a
/// `street` of the embankment's width, if the tie is dry, and each end of the
/// run is tied the same way to the nearest street, lane or primary node on its
/// bank within that reach, so the run closes the faces it borders.
///
/// **The two riverside forms** (Ruling EZ item 1, owner 2026-11-15:
/// *"Riverside houses are fine, as well as having a street in between the
/// river and house. So both conditions can exist (look at for example
/// Amsterdam)"*). With `frontage` `None` every run is an embankment `quay`
/// ([`EMBANKMENT_W`]), the houses behind it. With `Some(seed)` each run is cut
/// into stretches of [`FRONTAGE_STRETCH_M`] (drawn from the town's own
/// `riverbank/frontage` substream, so no reference draw moves) that alternate
/// between that quay and a [`BANK_CLASS`] edge ([`BANK_W`], laid
/// [`EMBANKMENT_CLEAR`] off the channel), along which the lots front the
/// water itself; the first form of each run is a coin flip. Both close the
/// riverside faces the same way. A stretch joins the next at the shared
/// centreline vertex, by a short step between the two offsets that belongs to
/// the bank stretch (so every quay vertex lies at the quay's offset), and a
/// run stays one connected line.
///
/// **Must only be called on a riverbank site** ([`Site::riverbank`]): it reads
/// the channel through [`Site::river_dist`] and [`Site::is_water`], which are
/// the channel only there. It must never lay an edge in the water, across the
/// river, or outside the box, and it adds edges only (it kills none), so every
/// street the reference grew is still there. It must never cross the town
/// wall (`wall`, the circuit `grow` built): a riverside or tie segment that
/// would is not laid, so no street passes the curtain without a gate (the
/// embankment is a `quay`, which the rampart sweep `clear_fort_zone` exempts).
pub fn build_embankment(site: &Site, g: &mut Graph, wall: Option<&[Vec2]>, frontage: Option<u32>) -> Embankment {
    build_embankment_with(site, g, wall, frontage, &[])
}

/// [`build_embankment`], with `waterfront`: embankment points
/// ([`Waterfront::water_pts`], change 5 of `TOWN_GENERATOR_JUDGEMENT.md`) that
/// belong to the town whatever its streets' reach, so the quay and bank
/// stretches run along the whole waterfront [`lay_back_street`] made. A bank
/// point within [`RIVER_STEP`]` / 2` of one counts. Empty is
/// [`build_embankment`] exactly.
pub fn build_embankment_with(site: &Site, g: &mut Graph, wall: Option<&[Vec2]>, frontage: Option<u32>, waterfront: &[Vec2]) -> Embankment {
    let mut out = Embankment::default();
    let r = &site.river;
    // RR-T2 (Ruling HE): a stream has no quay street. `river_w` is at least `RIVER_MIN_W_M` on every real river.
    if !site.riverbank() || r.len() < 2 || (site.uses_real_water && site.river_w < RIVER_MIN_W_M) {
        return out;
    }
    let off = site.river_w / 2.0 + EMBANKMENT_CLEAR + EMBANKMENT_W / 2.0;
    let off_bank = site.river_w / 2.0 + EMBANKMENT_CLEAR + BANK_W / 2.0;
    let mut rng = frontage.map(|seed| stream(seed, "riverbank/frontage"));
    // The town's street and lane nodes, with the bank each is on.
    let mut town: Vec<(Vec2, f64)> = Vec::new();
    for e in g.edges.iter().filter(|e| e.alive && (e.cls == "street" || e.cls == "lane")) {
        for nid in [e.a, e.b] {
            let p = g.nodes[nid].pt();
            town.push((p, side_of(site, p)));
        }
    }
    // Each centreline vertex (already every `RIVER_STEP`: `fine_river_path`)
    // with the left normal of the segment it starts (the last, of the one it
    // ends).
    let fine: Vec<(Vec2, Vec2)> = (0..r.len())
        .map(|i| {
            let (a, b) = if i + 1 < r.len() { (r[i], r[i + 1]) } else { (r[i - 1], r[i]) };
            (r[i], (b - a).norm().rot90())
        })
        .collect();
    for side in [1.0, -1.0] {
        // Bank points (with the centreline vertex and normal they came from),
        // `None` where the town does not reach that stretch.
        let bank: Vec<Option<(Vec2, Vec2, Vec2)>> = fine
            .iter()
            .map(|&(c, nl)| {
                let q = c + nl * (off * side);
                (town.iter().any(|&(p, s)| s == side && p.dist(q) <= EMBANKMENT_REACH) || waterfront.iter().any(|w| w.dist(q) <= RIVER_STEP / 2.0))
                    .then_some((q, c, nl))
            })
            .collect();
        let mut run: Vec<(Vec2, Vec2, Vec2)> = Vec::new();
        let mut runs: Vec<Vec<(Vec2, Vec2, Vec2)>> = Vec::new();
        for q in bank.iter().copied().chain(std::iter::once(None)) {
            match q {
                Some(q) if run.last().is_none_or(|l| dry_on_side(site, l.0, q.0, side)) => run.push(q),
                Some(q) => {
                    runs.push(std::mem::take(&mut run));
                    run.push(q);
                }
                None => runs.push(std::mem::take(&mut run)),
            }
        }
        for run in runs.into_iter().filter(|r| r.len() >= 2) {
            // Cut the run into stretches of one form each: (is a bank edge,
            // points). Quays only when no frontage seed was given.
            let mut stretches: Vec<(bool, Vec<Vec2>)> = Vec::new();
            match rng.as_mut() {
                None => stretches.push((false, run.iter().map(|t| t.0).collect())),
                Some(rg) => {
                    let mut is_bank = rg.chance(0.5);
                    let mut left = rg.range(FRONTAGE_STRETCH_M.0, FRONTAGE_STRETCH_M.1);
                    let at = |t: &(Vec2, Vec2, Vec2), b: bool| if b { t.1 + t.2 * (off_bank * side) } else { t.0 };
                    let mut cur: Vec<Vec2> = vec![at(&run[0], is_bank)];
                    for k in 1..run.len() {
                        left -= run[k - 1].1.dist(run[k].1);
                        cur.push(at(&run[k], is_bank));
                        if left <= 0.0 && k + 1 < run.len() {
                            // The short step between the two forms' offsets at
                            // this vertex belongs to the bank stretch, so every
                            // quay vertex stays at the quay's offset.
                            if is_bank {
                                cur.push(at(&run[k], false));
                            }
                            let last = *cur.last().expect("at least the vertex just pushed");
                            stretches.push((is_bank, std::mem::take(&mut cur)));
                            is_bank = !is_bank;
                            left = rg.range(FRONTAGE_STRETCH_M.0, FRONTAGE_STRETCH_M.1);
                            cur.push(last);
                            if is_bank {
                                cur.push(at(&run[k], true));
                            }
                        }
                    }
                    stretches.push((is_bank, cur));
                }
            }
            let mut segs: Vec<(Vec2, Vec2)> = Vec::new();
            for (is_bank, pts) in stretches.into_iter().filter(|s| s.1.len() >= 2) {
                let s = simplify(&pts, 0.8);
                let (cls, w, prov) = if is_bank { (BANK_CLASS, BANK_W, BANK_PROV) } else { ("quay", EMBANKMENT_W, EMBANKMENT_PROV) };
                for w2 in s.windows(2) {
                    if dry_on_side_w(site, w2[0], w2[1], side, w) && !crosses_wall(w2[0], w2[1], wall) {
                        g.add_street(w2[0].x, w2[0].y, w2[1].x, w2[1].y, cls, w, 0, prov);
                    }
                    segs.push((w2[0], w2[1]));
                }
                if is_bank { out.banks.push(s) } else { out.runs.push(s) }
            }
            // Tie the street ends on this bank within reach of this run.
            let ends: Vec<usize> = (0..g.nodes.len())
                .filter(|&n| g.live_degree(n) == 1)
                .filter(|&n| g.live_adj(n).any(|e| matches!(g.edges[e].cls, "street" | "lane")))
                .collect();
            for n in ends {
                let p = g.nodes[n].pt();
                if side_of(site, p) != side {
                    continue;
                }
                let (mut best, mut bd) = (None, EMBANKMENT_REACH);
                for &(a, b) in &segs {
                    let d = dist_pt_seg(p, a, b);
                    if d < bd {
                        bd = d;
                        best = Some(closest_on_seg(p, a, b));
                    }
                }
                // A street end already on the riverside edge needs no tie.
                if let Some(q) = best.filter(|_| bd > 1.0) {
                    if dry_on_side_w(site, p, q, side, BANK_W) && !crosses_wall(p, q, wall) {
                        g.add_street(p.x, p.y, q.x, q.y, "street", EMBANKMENT_W, 0, TIE_PROV);
                        out.ties += 1;
                    }
                }
            }
            // Each end of the run is carried to the nearest street, lane or
            // primary node on its bank within reach (2026-11-15): a run whose
            // ends hang free closes no face, so the riverside it was laid for
            // is never platted (seen on villages whose only nearby street
            // runs past the run's end rather than ending at it).
            let ends = [segs.first().map(|s| s.0), segs.last().map(|s| s.1)];
            for q in ends.into_iter().flatten() {
                let mut best: Option<(Vec2, f64)> = None;
                for e in g.edges.iter().filter(|e| e.alive && matches!(e.cls, "street" | "lane" | "primary")) {
                    for nid in [e.a, e.b] {
                        let p = g.nodes[nid].pt();
                        let d = p.dist(q);
                        if d > 1.0 && d <= EMBANKMENT_REACH && side_of(site, p) == side && best.is_none_or(|(_, bd)| d < bd) {
                            best = Some((p, d));
                        }
                    }
                }
                if let Some((p, _)) = best
                    && dry_on_side_w(site, p, q, side, BANK_W)
                    && !crosses_wall(p, q, wall)
                {
                    g.add_street(p.x, p.y, q.x, q.y, "street", EMBANKMENT_W, 0, TIE_PROV);
                    out.ties += 1;
                }
            }
        }
    }
    out
}

/// How far from the channel's edge a chartered river town's waterfront back
/// street runs ([`lay_back_street`]), metres: one depth drawn per bank.
/// **A labelled judgement** inside `TOWN_GENERATOR_JUDGEMENT.md` change 5's
/// *"35 to 50 m back from the water"*, capped at 45 m so the back street
/// stands within [`EMBANKMENT_REACH`] (46 m) of the embankment's line and the
/// embankment is laid along all of it. It is also the reference's own harbour
/// block: `buildHarbour` lays its back street 36-46 m inland of the quay.
pub const BACK_STREET_M: (f64, f64) = (35.0, 45.0);

/// The back street's width, metres. **The reference's** harbour back street
/// (`buildHarbour`, 4.0), the street this one is modelled on.
pub const BACK_STREET_W: f64 = 4.0;

/// The angle, degrees, under which an existing street counts as running
/// along the river ([`BACK_EXISTING_M`]). **A labelled judgement**:
/// a street within 30 degrees of the bank's line is a riverside street; one
/// crossing the back street's line at a steeper angle is a cross street that
/// the back street meets at a junction.
pub const BACK_PARALLEL_DEG: f64 = 30.0;

/// The shortest back street [`lay_back_street`] lays, metres. **A labelled
/// judgement**: one block of the engine's 60-150 m (`grow`'s street segments,
/// `parallelStreetSpacing`); a shorter piece closes no waterfront block.
pub const BACK_RUN_MIN_M: f64 = 60.0;

/// The spacing of the water lanes down from a back street to the bank,
/// metres: the shortest and longest gap, one drawn per lane. **A labelled
/// judgement**: the engine's own block length along a street (60-150 m,
/// above), shortened to 60-120 m because a waterfront block is reached from
/// the water lane as well as the back street (King's Lynn's lanes to the
/// river, `MEDIEVAL_TOWN_GROWTH_RESEARCH.md` rule 19, are its pattern).
pub const WATER_LANE_GAP_M: (f64, f64) = (60.0, 120.0);

/// How far from the channel's edge an existing street running along the
/// river serves as a waterfront's back line ([`lay_back_street`]), metres:
/// nearer than the first, the town already has a street at the water (the
/// embankment's own reach, [`EMBANKMENT_REACH`], closes that bank); between
/// the two, the street stays and only the water lanes are added; beyond the
/// second, a new back street is laid at [`BACK_STREET_M`]. **A labelled
/// judgement**: [`BACK_STREET_M`] widened by 15 m below (about a lot's
/// shallowest depth beyond the half-widths, so a new street never runs a
/// sliver away from an old one) and 25 m above, so a street that already stands about where the back
/// street would go is used rather than doubled (the measured case: seed
/// 483920 #18 at 384 x 246, a riverside street 55-60 m from the water left
/// the bank between unbuilt, because it stood beyond the embankment's reach
/// and too near for a second street).
pub const BACK_EXISTING_M: (f64, f64) = (20.0, 70.0);

/// What [`lay_back_street`] laid (change 5 of `TOWN_GENERATOR_JUDGEMENT.md`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Waterfront {
    /// Each new back street, as laid, simplified.
    pub streets: Vec<Vec<Vec2>>,
    /// Water lanes laid from a back line down to the bank.
    pub lanes: usize,
    /// The embankment points of every centreline vertex the waterfront
    /// covers: [`build_embankment`] lays its quay or bank stretch there
    /// whether or not a town street comes within its reach.
    pub water_pts: Vec<Vec2>,
}

/// How many centreline vertices ([`RIVER_STEP`], 8 m apart) a back street
/// leaves out on each side of a segment it refuses ([`lay_back_street`]).
/// With no drop the two run ends stood 8 m apart, inside `Graph::add_street`'s
/// 11 m node snap (`attach_point`'s `SNAP`, the reference's), and the wall
/// fixture caught a snapped end laying the street through the curtain. Two
/// puts them 40 m apart. **A labelled judgement**: one (24 m) would also
/// clear the snap; two leaves a margin for a centreline whose vertices are
/// closer than [`RIVER_STEP`] at a bend.
pub const BACK_BREAK_DROP: usize = 2;

/// The water lane's width, metres. **The reference's** lane width
/// (`lanePass`, 2.6 m, `crate::cleanup::lane_pass`).
pub const WATER_LANE_W: f64 = 2.6;

/// The provenance string of a back street [`lay_back_street`] lays.
const BACK_STREET_PROV: &str = "Waterfront back street (not the reference; 2026-11-15, TOWN_GENERATOR_JUDGEMENT.md \
     change 5, Ruling EZ item 1): the street behind the riverside plots, which run from it to the water.";

/// The provenance string of a water lane [`lay_back_street`] lays.
const WATER_LANE_PROV: &str = "Water lane (not the reference; 2026-11-15, TOWN_GENERATOR_JUDGEMENT.md change 5): \
     a lane from the waterfront back street down to the bank.";

/// **Not the reference** -- `TOWN_GENERATOR_JUDGEMENT.md` change 5 (2026-11-15;
/// owner Ruling EZ item 1: *"Riverside houses are fine, as well as having a
/// street in between the river and house. So both conditions can exist"*): a
/// chartered river town's waterfront. Along each bank the town reaches, a back
/// street [`BACK_STREET_M`] from the channel's edge, water lanes from it down
/// to the bank every [`WATER_LANE_GAP_M`], and ties from its ends into the
/// town, so that [`build_embankment`] (called next) lays its quay and bank
/// stretches along the whole of it and the strip between becomes waterfront
/// blocks: plots from the back street and from the water, houses on the water
/// where the embankment lays a [`BANK_CLASS`] stretch and behind a quay where
/// it lays a quay.
///
/// **Why.** Measured on the census at 384 (`town_river_census.rs::both_banks_audit`,
/// three seeds, `docs/evidence/waterfront-2026-11-15/`): 2.4 % of a chartered
/// river town's houses stood within two street widths of the bank and 39 % of
/// the bank its town reaches had no house or quay on it. The embankment is laid
/// only where a town street comes within [`EMBANKMENT_REACH`] of the water, and
/// a riverside face closes into a block only where streets meet it, so a town
/// whose streets stop 50-90 m from the water kept an open band along its river.
///
/// **What it does.** For each bank, each centreline vertex (one every
/// [`RIVER_STEP`]) looks out across its bank for a live street, lane, primary
/// or ring road running along the river ([`BACK_PARALLEL_DEG`]). One nearer the
/// channel than [`BACK_EXISTING_M`]`.0` means the town is already at the water
/// there (nothing to do); one within [`BACK_EXISTING_M`] is the back line; with
/// none, a back-street point is taken [`BACK_STREET_M`] (one depth per bank)
/// from the channel's edge, kept when one of the town's own streets (a
/// street, lane or ring road; not a primary, which runs out into the country
/// and over the bridge) stands within [`EMBANKMENT_REACH`] of it (the town
/// reaches it). Points off the market
/// square (`plaza`) and inside the town's wall (`wall`, when it has one: a
/// waterfront outside it lost its streets to the rampart sweep's
/// largest-piece prune and kept its houses) whose joining segments are dry on their bank
/// ([`dry_on_side_w`]) and cross no wall make a run; each run of at least
/// [`BACK_RUN_MIN_M`] has its new points laid as a `street` of
/// [`BACK_STREET_W`] (joined to the existing back line where the two meet), a
/// `lane` down to the embankment's line at each end and every
/// [`WATER_LANE_GAP_M`] between (drawn from its own `riverbank/waterfront`
/// substream, so no reference draw moves), and each new end tied to the
/// nearest town node on its bank within [`EMBANKMENT_REACH`]. A run that
/// neither meets an existing street nor can be tied at an end is not laid: it
/// would be a block standing alone by the river. The run's
/// embankment points are returned ([`Waterfront::water_pts`]) for
/// [`build_embankment_with`], which lays the waterfront along all of them.
///
/// **Must only be called on a riverbank site** ([`Site::riverbank`]), and by
/// `generate` only for a town of [`BOTH_BANKS_MIN_POP`] and up on the organic
/// plan. It must never lay an edge in the water, across the river, outside the
/// box, across the town wall (`wall`) or through the square, and it adds edges
/// only.
pub fn lay_back_street(site: &Site, g: &mut Graph, wall: Option<&[Vec2]>, plaza: Option<&[Vec2]>, seed: u32) -> Waterfront {
    let mut out = Waterfront::default();
    let r = &site.river;
    if !site.riverbank() || r.len() < 2 {
        return out;
    }
    let mut rng = stream(seed, "riverbank/waterfront");
    // Every street the town has, for the back line; the town's own streets
    // (not the primaries, which run out into the country and across bridges)
    // for whether the town reaches a stretch of bank.
    let town_cls = |c: &str| matches!(c, "street" | "lane" | "primary" | "ringroad");
    let town: Vec<(Vec2, Vec2, bool)> = g
        .edges
        .iter()
        .filter(|e| e.alive && town_cls(e.cls))
        .map(|e| (g.nodes[e.a].pt(), g.nodes[e.b].pt(), e.cls != "primary"))
        .collect();
    let cos_par = BACK_PARALLEL_DEG.to_radians().cos();
    let in_plaza = |p: Vec2| plaza.is_some_and(|z| z.len() >= 3 && point_in_poly(p, z));
    // Inside the town's wall, when it has one: the rampart sweep
    // (`cleanup::clear_fort_zone`) kills every street crossing the curtain
    // without a gate and then keeps only the largest connected piece of the
    // street graph (`prune_largest`), so a waterfront outside the wall, tied
    // to the town through it, lost every street while its lots and houses
    // stayed: a block standing alone by the river (seed 483920 #18 at
    // 384 x 288 and #24 at 2 048 x 1 311, 2026-11-15).
    let in_wall = |p: Vec2| wall.is_none_or(|r| r.len() < 3 || point_in_poly(p, r));
    let crosses_plaza = |a: Vec2, b: Vec2| plaza.is_some_and(|z| z.len() >= 3 && (0..z.len()).any(|k| seg_int(a, b, z[k], z[(k + 1) % z.len()]).is_some()));
    let water_off = site.river_w / 2.0 + EMBANKMENT_CLEAR + EMBANKMENT_W / 2.0;
    let fine: Vec<(Vec2, Vec2)> = (0..r.len())
        .map(|i| {
            let (a, b) = if i + 1 < r.len() { (r[i], r[i + 1]) } else { (r[i - 1], r[i]) };
            (r[i], (b - a).norm())
        })
        .collect();
    for side in [1.0, -1.0] {
        let back_off = site.river_w / 2.0 + rng.range(BACK_STREET_M.0, BACK_STREET_M.1);
        // Each vertex's back line point, whether it is on a street the town
        // already has (`true`) or a new back street (`false`), and the
        // embankment point it looks down to; `None` where the vertex has no
        // waterfront to make.
        let pts: Vec<Option<(Vec2, bool, Vec2)>> = fine
            .iter()
            .map(|&(c, dir)| {
                let nl = dir.rot90() * side;
                let w = c + nl * water_off;
                // The nearest existing street running along the river on
                // this bank, out to `BACK_EXISTING_M.1` from the channel.
                let (o, far) = (c + nl * (site.river_w / 2.0), c + nl * (site.river_w / 2.0 + BACK_EXISTING_M.1));
                let mut hit: Option<(f64, Vec2)> = None;
                for &(a, e, _) in &town {
                    let l = a.dist(e);
                    if l <= 1e-9 || ((e - a).dot(dir) / l).abs() < cos_par {
                        continue;
                    }
                    if let Some(h) = seg_int(o, far, a, e)
                        && hit.is_none_or(|(t, _)| h.t < t)
                    {
                        hit = Some((h.t, h.pt));
                    }
                }
                match hit {
                    // A riverside street already at the water: the
                    // embankment's own reach covers it.
                    Some((t, _)) if t * BACK_EXISTING_M.1 < BACK_EXISTING_M.0 => None,
                    // A riverside street the right depth back: it is the back
                    // line, and only the water lanes are added.
                    Some((_, p)) => (!in_plaza(p) && in_wall(p)).then_some((p, true, w)),
                    None => {
                        let b = c + nl * back_off;
                        let reached = town.iter().any(|&(a, e, own)| own && dist_pt_seg(b, a, e) <= EMBANKMENT_REACH);
                        (reached && side_of(site, b) == side && !in_plaza(b) && in_wall(b)).then_some((b, false, w))
                    }
                }
            })
            .collect();
        let seg_ok = |a: Vec2, b: Vec2| dry_on_side_w(site, a, b, side, BACK_STREET_W) && !crosses_wall(a, b, wall) && !crosses_plaza(a, b);
        let mut runs: Vec<Vec<(Vec2, bool, Vec2)>> = Vec::new();
        let mut run: Vec<(Vec2, bool, Vec2)> = Vec::new();
        // Vertices still to drop after a refused segment (below).
        let mut skip = 0usize;
        for q in pts.into_iter().chain(std::iter::once(None)) {
            match q {
                Some(_) if skip > 0 => skip -= 1,
                // Two existing-street points need no segment between them
                // (the street is there); any pair with a new point does.
                Some(q) if run.last().is_none_or(|l| (l.1 && q.1) || seg_ok(l.0, q.0)) => run.push(q),
                // A refused segment (wet, across the wall or the square): the
                // run stops [`BACK_BREAK_DROP`] vertices short of it and the
                // next starts as many past it, so `Graph::add_street`'s 11 m
                // node snap cannot join the two ends across what refused it.
                Some(_) => {
                    let mut done = std::mem::take(&mut run);
                    done.truncate(done.len().saturating_sub(BACK_BREAK_DROP));
                    runs.push(done);
                    skip = BACK_BREAK_DROP - 1;
                }
                None => {
                    runs.push(std::mem::take(&mut run));
                    skip = 0;
                }
            }
        }
        for run in runs {
            let len: f64 = run.windows(2).map(|w| w[0].2.dist(w[1].2)).sum();
            if run.len() < 2 || len < BACK_RUN_MIN_M {
                continue;
            }
            // Each new end's tie into the town on its bank: the nearest town
            // node within `EMBANKMENT_REACH` whose tie is dry, crosses no wall
            // and misses the square.
            let tie = |q: Vec2| {
                let mut best: Option<(Vec2, f64)> = None;
                for &(a, e, _) in &town {
                    for p in [a, e] {
                        let d = p.dist(q);
                        if d > 1.0 && d <= EMBANKMENT_REACH && side_of(site, p) == side && best.is_none_or(|(_, bd)| d < bd) {
                            best = Some((p, d));
                        }
                    }
                }
                best.map(|(p, _)| p).filter(|&p| seg_ok(p, q))
            };
            let ties: Vec<(Vec2, Vec2)> =
                [run[0], run[run.len() - 1]].into_iter().filter(|t| !t.1).filter_map(|t| tie(t.0).map(|p| (p, t.0))).collect();
            // A run joined to the town by neither an existing street nor a
            // tie would stand alone by the river, a block with no way into the
            // town (seen at seed 483920 #24 at 2 048 before this rule): not laid.
            if ties.is_empty() && !run.iter().any(|t| t.1) {
                continue;
            }
            // The new back street, piece by piece: every stretch of
            // consecutive points with a new one in each pair.
            let mut piece: Vec<Vec2> = Vec::new();
            for k in 0..run.len() {
                let joins = k + 1 < run.len() && !(run[k].1 && run[k + 1].1);
                if joins || (k > 0 && !(run[k - 1].1 && run[k].1)) {
                    piece.push(run[k].0);
                }
                if !joins && piece.len() >= 2 {
                    let line = simplify(&piece, 0.8);
                    for w in line.windows(2) {
                        g.add_street(w[0].x, w[0].y, w[1].x, w[1].y, "street", BACK_STREET_W, 0, BACK_STREET_PROV);
                    }
                    out.streets.push(line);
                }
                if !joins {
                    piece.clear();
                }
            }
            // Water lanes: at the run's first vertex, then every drawn gap,
            // and at its last.
            let mut at: Vec<usize> = vec![0];
            let (mut walked, mut next) = (0.0, rng.range(WATER_LANE_GAP_M.0, WATER_LANE_GAP_M.1));
            for k in 1..run.len() {
                walked += run[k - 1].2.dist(run[k].2);
                if walked >= next && len - walked >= WATER_LANE_GAP_M.0 / 2.0 {
                    at.push(k);
                    walked = 0.0;
                    next = rng.range(WATER_LANE_GAP_M.0, WATER_LANE_GAP_M.1);
                }
            }
            at.push(run.len() - 1);
            for k in at {
                let (b, _, w) = run[k];
                if dry_on_side_w(site, b, w, side, WATER_LANE_W) && !crosses_wall(b, w, wall) && !crosses_plaza(b, w) {
                    g.add_street(b.x, b.y, w.x, w.y, "lane", WATER_LANE_W, 0, WATER_LANE_PROV);
                    out.lanes += 1;
                }
            }
            // Each new end tied into the town on its bank.
            for (p, q) in ties {
                g.add_street(p.x, p.y, q.x, q.y, "street", BACK_STREET_W, 0, BACK_STREET_PROV);
            }
            out.water_pts.extend(run.iter().map(|t| t.2));
        }
    }
    out
}

/// What [`lay_town_bridge`] found or laid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TownBridge {
    /// A live `primary` already crosses the river at the bridge point (within
    /// the span's half-length; a real road the host supplied,
    /// `build_primaries_from_paths`): no bridge is added.
    Existing,
    /// A bridge was laid at the site's bridge point, with its approaches.
    Laid,
    /// No crossing: no bridge point, one too far from the market, or a bridge
    /// whose ends would stand in the water, outside the box or on ground the
    /// town could not build on.
    None,
}

/// Which layout of a river town [`crate::generate`] is making (2026-11-15).
/// Every pass but `First` exists only with `urban_riverbank` on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiverbankPass {
    /// The reference's layout (with the riverbank geometry when it is on).
    First,
    /// Ruling FB's second layout of a both-banks town that housed under its
    /// target: the market's bank as in `First`, then a bridge and
    /// `crate::growth::grow_far_bank` with a street budget of `share` times
    /// the market bank's street length.
    FarBank { share: f64 },
    /// A riverbank village that the first layout over-built: grown again
    /// from `street_scale` times its street budget ([`fit_village`]), with
    /// [`trim_overbuilt`] run only when `trim` (the chosen layout, still over
    /// the cap).
    Village { street_scale: f64, trim: bool },
}

/// How much more than its target a town with a grain (`Town::parallel`'s field) may house on the
/// market's bank and still be given the far bank ([`second_pass`]). **A labelled judgement**: the
/// upper quartile of housed(switch on) / housed(off) over the 35 census towns is 1.22 (median 1.06,
/// `parallel-streets-ps2-2026-11-18/step0/towns_s0b_384.csv`), so 1.25 gives the far bank to a town
/// whose off layout would have taken it. Reached only with `urban_parallel_streets` on.
pub const PARALLEL_FAR_BANK_SLACK: f64 = 1.25;

/// Whether [`crate::generate`] lays this town out again, and how, from its
/// first layout `t` (2026-11-15; `None` for a second layout, with the switch
/// off, or when the first is right).
///
/// - **A both-banks town** (`Town::both_banks`) housing under its
///   `pop_target`: [`RiverbankPass::FarBank`] with `share` the shortfall as a
///   fraction of what it housed (`pop_target / pop - 1`), at most
///   [`FAR_BANK_MAX_RATIO`]. Why the shortfall: the owner's bar is that the
///   town's own target governs its size (coordinator, 2026-11-15: *"Growing
///   the far bank must not shrink the near bank below what it housed
///   before"*), and street length is what the platting turns into houses at
///   the same rate on either bank. A town that houses its target on one bank
///   stays on it, as the ruling's *"can"* allows.
/// - **A village** (under [`BOTH_BANKS_MIN_POP`]) housing over
///   [`RIVER_HOUSING_CAP`] of its target: [`RiverbankPass::Village`], which
///   `generate` hands to [`fit_village`] to search the street scale, so the
///   village is grown to about its target and the lots it does not need are
///   never platted; [`trim_overbuilt`] then empties only what that leaves
///   over the cap. Why: the trim alone
///   left a ring of emptied plots round about half of riverbank villages
///   (`docs/evidence/both-banks-2026-11-15/`).
///
/// Must never answer for a town laid out with the switch off, so the
/// reference is laid out once, byte for byte.
pub fn second_pass(t: &crate::Town) -> Option<RiverbankPass> {
    if !(t.riverbank || t.street_village) || t.riverbank_pass != Some(RiverbankPass::First) {
        return None;
    }
    // **Not the reference** (PS-2 step 0b, Ruling GH, `PARALLEL_FAR_BANK_SLACK`): a town with a
    // grain houses more on the market's bank than today's layout does, so judged against its bare
    // target it skipped the far bank today's layout takes. Its target is raised by the slack.
    let target = t.pop_target * if t.parallel.as_ref().is_some_and(|p| p.field.is_some()) { PARALLEL_FAR_BANK_SLACK } else { 1.0 };
    if t.both_banks && t.pop < target {
        let max = far_bank_max_ratio(t.bank_sites);
        let share = if t.pop > 0.0 { target / t.pop - 1.0 } else { max };
        return Some(RiverbankPass::FarBank { share: share.min(max) });
    }
    if t.pop_target < BOTH_BANKS_MIN_POP && t.pop > t.pop_target * RIVER_HOUSING_CAP {
        return Some(RiverbankPass::Village { street_scale: t.pop_target / t.pop, trim: false });
    }
    // **Ruling FE item 2** (`crate::village`): a street village housing under
    // [`VILLAGE_LOW_SHARE`] of its target, whose road frontage ran out (a road
    // along its bank has one side; a short road has little), is laid out again
    // with more lanes by its road, whose frontage takes more tofts.
    if t.street_village && t.pop < t.pop_target * village_low_share(t) {
        let scale = if t.pop > 0.0 { (t.pop_target / t.pop).clamp(1.0, VILLAGE_MAX_SCALE) } else { VILLAGE_MAX_SCALE };
        return Some(RiverbankPass::Village { street_scale: scale, trim: false });
    }
    None
}

/// [`VILLAGE_LOW_SHARE`] for a town on the map's wide water
/// (`GenOpts::bank_sites`, Ruling HN, owner 2026-11-23). **A labelled
/// judgement**, 0.97: the ruling's bar is that a town houses on the bank what
/// it housed in the river, and a river village on the reference's narrow
/// channel over-built and was fitted down to its target, while on the bank
/// its first layout landed in the 80 to 100 % it was accepted in (seed 532187
/// at 2 048 x 1 311: villages at 81, 89 and 92 % that had housed 100 %). Three
/// points under the target is the fit's own noise (one toft in 30).
pub const BANK_SITES_LOW_SHARE: f64 = 0.97;

/// The least share of its target a street village's layout is accepted at:
/// [`BANK_SITES_LOW_SHARE`] for a town laid out with `GenOpts::bank_sites`,
/// else [`VILLAGE_LOW_SHARE`].
pub fn village_low_share(t: &crate::Town) -> f64 {
    if t.bank_sites { BANK_SITES_LOW_SHARE } else { VILLAGE_LOW_SHARE }
}

/// The share of its target a riverbank village's refitted layout may house
/// at the least ([`fit_village`]). **A labelled judgement**: the village
/// census's own "under 80 %" bar (`town_river_census.rs::both_banks_audit`),
/// so a refit never makes a village the audit would flag as short.
pub const VILLAGE_LOW_SHARE: f64 = 0.8;
/// What [`fit_village`] aims at, as a share of the target. **A labelled
/// judgement**: inside the band from [`VILLAGE_LOW_SHARE`] to
/// [`RIVER_HOUSING_CAP`], nearer the cap since the households come in lots of
/// 5.2 and the search stops at the first layout inside the band.
pub const VILLAGE_AIM_SHARE: f64 = 1.1;
/// How many refitted layouts [`fit_village`] tries at most. **A labelled
/// judgement**: a village's layout is a few milliseconds, and the measured
/// villages settle in two or three (the street budget does not turn into
/// houses in proportion: the primaries' and the embankment's lots come
/// whatever the budget).
pub const VILLAGE_FITS: usize = 4;
/// The least street scale [`fit_village`] tries. **A labelled judgement**: a
/// tenth of the budget is a crossroads and a lane; below it the village is
/// its primaries alone.
pub const VILLAGE_MIN_SCALE: f64 = 0.1;
/// The most street scale [`fit_village`] tries for a street village short of
/// its target (Ruling FE item 2). **A labelled judgement**: eight times
/// `crate::village::STREET_PER_HEAD_M` is 2 m a head, about a town's own
/// 2.1 m (M-DEN-1/2), so a short village is never given more lane per head
/// than a town.
pub const VILLAGE_MAX_SCALE: f64 = 8.0;

/// Lay a riverbank village out again from a scaled street budget until it
/// houses between [`VILLAGE_LOW_SHARE`] and [`RIVER_HOUSING_CAP`] of its
/// target (2026-11-15, the coordinator's item 4: *"Prefer not creating those
/// lots to emptying them"*). `first` is the first layout (scale 1, over the
/// cap); `lay(scale, trim)` makes one layout. Each next scale is the secant
/// through the last two layouts' head counts aimed at [`VILLAGE_AIM_SHARE`]
/// (the first step `target / pop`), clamped to [`VILLAGE_MIN_SCALE`]..1; at
/// most [`VILLAGE_FITS`] layouts. The first layout inside the band is taken
/// as it stands. If none is, the over-cap layout housing the fewest (the
/// first one among them) is laid out once more with [`trim_overbuilt`]
/// (`trim`), which then empties only the few lots its streets could not
/// avoid: never a layout under the band, since a village rescaled from 125 %
/// to 31 % of its target (measured on 48 villages of the census at 384 when
/// the nearest to the aim was taken) is the worse look. **Must never** return
/// a layout over the cap untrimmed, one housing less than the first layout
/// trimmed would, or run with the switch off.
pub fn fit_village(first: crate::Town, lay: impl Fn(f64, bool) -> crate::Town) -> crate::Town {
    let target = first.pop_target;
    let (lo, hi, aim) = (target * village_low_share(&first), target * RIVER_HOUSING_CAP, target * VILLAGE_AIM_SHARE);
    if first.pop < lo {
        return fit_short_village(first, lay);
    }
    let mut pts: Vec<(f64, f64)> = vec![(1.0, first.pop)];
    // The over-cap layout housing the fewest, by its scale (the first layout
    // is one).
    let mut least_over = (1.0, first.pop);
    let mut scale = (target / first.pop.max(1.0)).clamp(VILLAGE_MIN_SCALE, 1.0);
    for _ in 0..VILLAGE_FITS {
        let t = lay(scale, false);
        if t.pop >= lo && t.pop <= hi {
            return t;
        }
        if t.pop > hi && t.pop < least_over.1 {
            least_over = (scale, t.pop);
        }
        pts.push((scale, t.pop));
        let (s1, p1) = pts[pts.len() - 2];
        let (s2, p2) = pts[pts.len() - 1];
        let next = if (p2 - p1).abs() > 1e-9 { s2 + (aim - p2) * (s2 - s1) / (p2 - p1) } else { s2 * 0.5 };
        let next = next.clamp(VILLAGE_MIN_SCALE, 1.0);
        if (next - scale).abs() < 1e-3 {
            break;
        }
        scale = next;
    }
    lay(least_over.0, true)
}

/// **Ruling FE item 2** (`crate::village`): [`fit_village`] for a street
/// village whose first layout housed under [`VILLAGE_LOW_SHARE`] of its
/// target: each next street scale is the secant aimed at
/// [`VILLAGE_AIM_SHARE`], from `target / pop`, clamped to 1..
/// [`VILLAGE_MAX_SCALE`]; at most [`VILLAGE_FITS`] layouts. The first layout
/// inside the band is taken; otherwise the one housing the most without
/// passing [`RIVER_HOUSING_CAP`] (the first layout among them, so a village
/// no scale helps is laid out as it was). **Must never** return a layout
/// housing fewer than `first`, or one over the cap.
pub fn fit_short_village(first: crate::Town, lay: impl Fn(f64, bool) -> crate::Town) -> crate::Town {
    let target = first.pop_target;
    let (lo, hi, aim) = (target * village_low_share(&first), target * RIVER_HOUSING_CAP, target * VILLAGE_AIM_SHARE);
    let mut pts: Vec<(f64, f64)> = vec![(1.0, first.pop)];
    let mut best = first;
    let mut scale = (target / best.pop.max(1.0)).clamp(1.0, VILLAGE_MAX_SCALE);
    for _ in 0..VILLAGE_FITS {
        let t = lay(scale, false);
        if t.pop >= lo && t.pop <= hi {
            return t;
        }
        pts.push((scale, t.pop));
        if t.pop <= hi && t.pop > best.pop {
            best = t;
        }
        let (s1, p1) = pts[pts.len() - 2];
        let (s2, p2) = pts[pts.len() - 1];
        let next = if (p2 - p1).abs() > 1e-9 { s2 + (aim - p2) * (s2 - s1) / (p2 - p1) } else { s2 * 2.0 };
        let next = next.clamp(1.0, VILLAGE_MAX_SCALE);
        if (next - scale).abs() < 1e-3 {
            break;
        }
        scale = next;
    }
    best
}

/// The far end of the town bridge [`lay_town_bridge`] lays (or would lay) at
/// the site's bridge point: the point `riverW/2 + 16` m from it across the
/// river, on the bank the market is not on. The far bank's centre for
/// `crate::growth::grow_far_bank`. `None` without a bridge point or a river.
pub fn far_bridge_end(site: &Site, market: Vec2) -> Option<Vec2> {
    let (bp, dir) = (site.bridge_pt?, site.bridge_dir?);
    if site.river.len() < 2 {
        return None;
    }
    let nl = dir.rot90();
    let half = site.river_w / 2.0 + 16.0;
    let (e1, e2) = (bp + nl * half, bp + nl * -half);
    Some(if site.bank_side(e1) == site.bank_side(market) { e2 } else { e1 })
}

/// Ruling FB's *"when the terrain allows it"*: may a both-banks town build at
/// `p` on its far bank? Its `terrain_suitability` must reach
/// [`TERRAIN_BUILD_MIN`], the bar the building stage leaves a lot vacant
/// under, so the far bank grows only where it would be built: not a gorge
/// (the slope term) or a marsh bank (the flood term, near the water).
/// Read by `crate::growth::grow_far_bank` for every far-bank origin and by [`lay_town_bridge`] for
/// the bridge's far end.
pub fn far_bank_buildable(site: &Site, p: Vec2) -> bool {
    terrain_suitability(site, p) >= TERRAIN_BUILD_MIN
}

/// The most street a both-banks town's far bank may hold, as a multiple of
/// its market's bank (Ruling FB). **A labelled judgement**: 1.0, so the far
/// bank grows up to the size of the market's own and no further. The market's
/// bank is the town's centre (its square, its primaries); a far bank allowed
/// to outgrow it turns the town inside out, and where the two banks are joined
/// only by a crossing the real-water sweep removes, `prune_largest` keeps the
/// larger and deletes the market's (measured: seed 483920 #4 at 384 x 246 fell
/// from 89 % to 44 % of its target that way before this cap and the bridge
/// rule in [`lay_town_bridge`]).
pub const FAR_BANK_MAX_RATIO: f64 = 1.0;

/// [`FAR_BANK_MAX_RATIO`] for a town on the map's wide water
/// (`GenOpts::bank_sites`, **Ruling HQ**, owner 2026-11-23: *"raise the
/// far-bank cap for bank-site towns"*). **A labelled judgement**, 2.0: on the
/// map's river (the owner's town, seed 532187 at cell 1478, 1159: 399 m wide)
/// the market's bank keeps less ground than on the reference's 12 to 46 m
/// channel, and the far bank, grown from the bridge's far end, is the
/// town's other half rather than its suburb. Measured on that town
/// (`docs/evidence/town-on-bank-hq-2026-11-23/`): with
/// [`crate::growth::BANK_SITES_FAR_BANK_TRIES`] its far bank stops at 1.41
/// times the market bank's streets, so 2.0 never binds there; 1.0 stopped it
/// at 1.0 (78 % housed). Still a cap, so the market's bank stays the town's
/// centre where the far bank has far more ground. The share it also bounds is
/// the shortfall (`second_pass`), so a town is never grown past its target by it.
pub const BANK_SITES_FAR_BANK_MAX_RATIO: f64 = 2.0;

/// The far bank's cap as a multiple of the market bank's streets: Ruling HQ's
/// [`BANK_SITES_FAR_BANK_MAX_RATIO`] for a town laid out with
/// `GenOpts::bank_sites`, else Ruling FB's [`FAR_BANK_MAX_RATIO`] (so a town
/// with the switch off is laid out as before). Read by [`second_pass`] and
/// `crate::growth::grow_far_bank`.
pub fn far_bank_max_ratio(bank_sites: bool) -> f64 {
    if bank_sites { BANK_SITES_FAR_BANK_MAX_RATIO } else { FAR_BANK_MAX_RATIO }
}

/// The live street length on each bank of a river site, metres: `(the
/// market's bank, the other)`, by each edge's midpoint
/// ([`Site::bank_side`]). Every live edge but the embankment and bank edges,
/// which run along the water. Read by `crate::growth::grow_far_bank` for
/// its budget and its [`FAR_BANK_MAX_RATIO`] cap.
pub fn bank_street_lengths(site: &Site, g: &Graph, mkt_side: f64) -> (f64, f64) {
    let (mut near, mut far) = (0.0, 0.0);
    for e in g.edges.iter().filter(|e| e.alive && e.cls != "quay" && e.cls != BANK_CLASS) {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        if site.bank_side(a.lerp(b, 0.5)) == mkt_side {
            near += a.dist(b);
        } else {
            far += a.dist(b);
        }
    }
    (near, far)
}

/// The most a river town on a riverbank site may house, as a multiple of its
/// `pop_target` ([`trim_overbuilt`]). **A labelled judgement**: a quarter
/// over, room for the 5.2-person household's granularity and for the empty
/// house every real town has, and under the 139-146 % median the riverbank
/// villages reached (`docs/evidence/both-banks-2026-11-15/`).
pub const RIVER_HOUSING_CAP: f64 = 1.25;

/// Leave a riverbank town's outermost inland lots vacant until it houses at
/// most [`RIVER_HOUSING_CAP`] times `pop_target` (5.2 people a built lot, the
/// head count `generate` takes).
///
/// **Why** (2026-11-15). Villages on a riverbank site were over-built: a
/// median of 139 % of their target at 384 and 146 % at 2 048, 117 and 143 of
/// them over 200 %. Two causes, measured in the evidence above: the
/// reference's street budget has a 1 600 m floor (`generate`'s `target_len`),
/// which is the street of a 762-person town, so every village under that is
/// asked for more street than its people need (pre-existing, every site); and
/// the embankment and bank edges close the riverside faces into blocks that
/// were open ground before, so the same streets plat more lots (this code).
/// The street floor is the reference's and is left alone; the lots the river
/// town gains are paid for here, from its fringe, not its riverside.
///
/// **What it does.** Built lots are emptied (`built` false, `empty` true) and
/// their buildings removed until the head count is within the cap: those
/// outside the town wall `ring` first, then the rest, each group farthest
/// from the market first, skipping lots that front the riverside (`quay` or
/// [`BANK_CLASS`]) and churchyards. Outside the wall first because an emptied
/// lot inside it is the empty walled interior Ruling FB forbids (seen on the
/// harness town at (108,71) of seed 12345 before this order). `generate`
/// calls it for villages only (under [`BOTH_BANKS_MIN_POP`]), and since
/// 2026-11-15 only in a village's second layout
/// ([`RiverbankPass::Village`]), whose street budget was already scaled to
/// its target, so it empties only the remainder. **Must never** empty a lot when
/// the town is within the cap, touch a riverside lot, empty a lot inside the
/// wall while one outside it is still built, or leave a building on an
/// emptied lot.
pub fn trim_overbuilt(
    lots: &mut [crate::districts::Lot<'_>],
    buildings: &mut Vec<crate::districts::Building>,
    market: Vec2,
    pop_target: f64,
    ring: Option<&[Vec2]>,
) {
    let cap = (pop_target * RIVER_HOUSING_CAP / 5.2).floor() as usize;
    let built = lots.iter().filter(|l| l.built && !l.churchyard).count();
    if built <= cap {
        return;
    }
    let mut order: Vec<usize> = (0..lots.len())
        .filter(|&i| lots[i].built && !lots[i].churchyard && lots[i].par.edge_cls != "quay" && lots[i].par.edge_cls != BANK_CLASS)
        .collect();
    let d = |i: usize| crate::geom::poly_centroid(&lots[i].par.poly).dist(market);
    let inside = |i: usize| ring.is_some_and(|r| crate::geom::point_in_poly(crate::geom::poly_centroid(&lots[i].par.poly), r));
    order.sort_by(|&a, &b| inside(a).cmp(&inside(b)).then(d(b).total_cmp(&d(a))).then(a.cmp(&b)));
    let mut drop: std::collections::HashSet<String> = std::collections::HashSet::new();
    for &i in order.iter().take(built - cap) {
        lots[i].built = false;
        lots[i].empty = true;
        drop.insert(lots[i].par.id.clone());
    }
    buildings.retain(|b| !drop.contains(&b.parcel));
}

/// Is every one of ten samples along `a`-`b` dry ground (off the channel by
/// half the river's width and a half metre, the reference's own
/// `removeWaterCrossings` test, and off open water), inside the box?
fn dry_line(site: &Site, a: Vec2, b: Vec2) -> bool {
    (0..=10).all(|k| {
        let p = a.lerp(b, k as f64 / 10.0);
        p.x >= 0.0
            && p.y >= 0.0
            && p.x <= site.wm
            && p.y <= site.hm
            && !site.is_water(p)
            && site.river_dist(p) >= site.river_w / 2.0 + 0.5
    })
}

/// **Not the reference** -- Ruling FB (owner, 2026-11-15): *"Big river towns
/// can grow around both banks when the terrain allows it."* Give a big river
/// town ([`BOTH_BANKS_MIN_POP`] and up, on a riverbank site) a bridge joined
/// to its streets, so `crate::growth::grow_far_bank` can extend the town onto
/// the far bank from the bridge's far end. Laid after `grow` (2026-11-15:
/// before it, the bridge's edges changed what `grow` drew on the market's
/// bank), in a both-banks town's second layout only
/// ([`RiverbankPass::FarBank`]).
///
/// **Why.** On real water the reference lays no bridge at all
/// (`addRiverBridges` returns when `usesRealWater`), and a crossing is only
/// recorded where a real road happens to cross (`detectRiverCrossings`):
/// measured on 60 harness towns, 40 had none (`town-housing-2026-11-15`), so
/// the far bank had no street to grow from.
///
/// **What it does.** If a live `primary` already crosses the centreline
/// within `remove_water_crossings`' bridge radius of the bridge point and
/// within `max_r` of the market, nothing ([`TownBridge::Existing`]): that
/// crossing survives the real-water sweep.
/// Otherwise a `primary` span (6 m, `addRiverBridges`' own width and
/// `riverW/2 + 16` half-length) across the site's bridge point, which
/// `build_site` chose as the flattest approach in the middle of the river, and
/// which is the one place the reference's real-water sweep
/// (`remove_water_crossings`) lets a road cross. The near end is joined to the
/// nearest street node on the market's bank within [`BRIDGE_JOIN_M`] by a
/// dry street; the far end carries a [`BRIDGE_FAR_STUB_M`] approach street straight on, away
/// from the river, if that is dry.
///
/// **What it must never do.** Lay a bridge whose far end stands on ground the
/// town could not build on (`terrain_suitability` under
/// [`TERRAIN_BUILD_MIN`], the bar the building stage itself uses: a gorge or
/// a marsh bank), in the water or within 40 m of the box edge (`grow`'s own
/// edge rule), or more than `max_r` from the market; lay any street through
/// the water but the span itself; or remove anything.
pub fn lay_town_bridge(site: &Site, anchors: &Anchors, g: &mut Graph, max_r: f64) -> TownBridge {
    lay_town_bridge_search(site, anchors, g, max_r, false).0
}

/// How many centreline vertices [`lay_town_bridge_search`] may try for a
/// crossing. **A labelled judgement** (Ruling GB, 2026-11-17): enough to walk
/// a V-bend's two limbs and both approaches (24 vertices 25 m apart span
/// 600 m of river), few enough that the search, which runs only in a
/// both-banks town's second layout, stays cheap.
pub const BRIDGE_SEARCH_MAX_TRIES: usize = 24;

/// The least spacing between two vertices [`lay_town_bridge_search`] tries,
/// metres. **A labelled judgement** (Ruling GB): the resampled centreline is a
/// vertex every 8 m (`fine_river_path`), so without a spacing the 24 tries
/// would all be neighbours of one bend and test the same crossing 24 times.
pub const BRIDGE_SEARCH_SPACING_M: f64 = 25.0;

/// [`lay_town_bridge`] with Ruling GB's search (`GenOpts::town_bridge_search`):
/// the bridge's outcome, its far end (the far bank's centre for
/// `crate::growth::grow_far_bank`, [`far_bridge_end`] at the site's own bridge
/// point) and, when the search laid it, the point it crossed at.
///
/// **Why.** The site's bridge point is one fixed place, `build_site`'s flattest
/// approach in the middle of the river. Where the river bends in a V round the
/// market that point's span can have both ends on one bank or one in the
/// channel, `lay_town_bridge` gave up (seed 912345 #10 at 2 048 x 1 311: bridge
/// point 50 m from the market, refused at the ends' test), `grow_far_bank` never
/// ran and the far bank, 42 % of the growth disc, held 55 buildings against
/// about 1 000 in the healthy controls.
///
/// **What it does.** With `search` off, exactly [`lay_town_bridge`]'s result
/// (the second field is [`far_bridge_end`]'s, the third `None`). With it on, and
/// only where the site's own point failed, the centreline's vertices within
/// `max_r` of the market, nearest first and at least
/// [`BRIDGE_SEARCH_SPACING_M`] apart, [`BRIDGE_SEARCH_MAX_TRIES`] at most, are
/// tried with the same span, ends test, buildability test and join as the
/// site's point; the first that holds is laid.
///
/// **What it must never do.** Change a town whose own point lays or already
/// holds a crossing, lay a span whose ends fail any test the site's point must
/// pass, or touch the graph for a vertex that fails. The sweep
/// (`crate::cleanup::remove_water_crossings`) removes a span over open water
/// away from `Site::bridge_pt`, so the caller must hand the returned point to
/// `Site::alt_bridge_pt`.
pub fn lay_town_bridge_search(
    site: &Site,
    anchors: &Anchors,
    g: &mut Graph,
    max_r: f64,
    search: bool,
) -> (TownBridge, Option<Vec2>, Option<Vec2>) {
    let r = &site.river;
    if !site.riverbank() || r.len() < 2 {
        return (TownBridge::None, None, None);
    }
    let (Some(bp), Some(dir)) = (site.bridge_pt, site.bridge_dir) else { return (TownBridge::None, None, None) };
    // A road that already crosses counts only where it is this bridge: a
    // `primary` (the base sweep culls any other street through the channel)
    // crossing within the span's own half-length (`riverW/2 + 16`) of the
    // bridge point, which the real-water sweep keeps (its own test is
    // repeated below). Any other crossing may be removed after growth, and a
    // far bank grown from it is cut off and one bank pruned (`prune_largest`
    // keeps one component): measured on seed 483920 #0 at 2 048 x 1 311, which
    // fell from 78 % to 58 % of its target that way while crossings anywhere
    // within the sweep's radius counted.
    let bridge_r = js_or(site.river_w, 20.0) * 1.5 + 34.0;
    for e in g.edges.iter().filter(|e| e.alive && e.cls == "primary") {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        // The sweep's own test: two of nine interior samples in the water
        // beyond `bridge_r` of the bridge point kill the edge.
        let open_wet = (1..10).filter(|&i| {
            let p = a.lerp(b, i as f64 / 10.0);
            site.is_water(p) && p.dist(bp) > bridge_r
        });
        if open_wet.count() >= 2 {
            continue;
        }
        for w in r.windows(2) {
            if let Some(h) = seg_int(a, b, w[0], w[1])
                && h.pt.dist(bp) <= js_or(site.river_w, 20.0) / 2.0 + 16.0
                && h.pt.dist(anchors.market) <= max_r
            {
                return (TownBridge::Existing, far_bridge_end(site, anchors.market), None);
            }
        }
    }
    if bp.dist(anchors.market) <= max_r
        && let Some(far) = lay_span(site, anchors, g, bp, dir)
    {
        return (TownBridge::Laid, Some(far), None);
    }
    if search {
        let mut cands: Vec<(f64, usize)> =
            r.iter().enumerate().map(|(i, p)| (p.dist(anchors.market), i)).filter(|&(d, _)| d <= max_r).collect();
        cands.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut tried: Vec<Vec2> = Vec::new();
        for (_, i) in cands {
            if tried.len() >= BRIDGE_SEARCH_MAX_TRIES {
                break;
            }
            let p = r[i];
            if tried.iter().any(|q| q.dist(p) < BRIDGE_SEARCH_SPACING_M) {
                continue;
            }
            tried.push(p);
            let (a, b) = (r[i.saturating_sub(1)], r[(i + 1).min(r.len() - 1)]);
            if a.dist(b) < 1e-6 {
                continue;
            }
            if let Some(far) = lay_span(site, anchors, g, p, (b - a).norm()) {
                return (TownBridge::Laid, Some(far), Some(p));
            }
        }
    }
    (TownBridge::None, None, None)
}

/// One crossing of [`lay_town_bridge_search`]: a 6 m `primary` span at `bp`
/// across a river running along `dir`, joined to the market's bank, with the
/// far stub. Returns the span's far end, and touches `g` only when it lays.
/// The tests are the ones `lay_town_bridge`'s doc names (ends dry, in the box,
/// on opposite banks, the far end buildable, a join within [`BRIDGE_JOIN_M`]).
fn lay_span(site: &Site, anchors: &Anchors, g: &mut Graph, bp: Vec2, dir: Vec2) -> Option<Vec2> {
    let nl = dir.rot90();
    let half = site.river_w / 2.0 + 16.0;
    let mkt_side = site.bank_side(anchors.market);
    let (e1, e2) = (bp + nl * half, bp + nl * -half);
    let (near, far, out_dir) = if site.bank_side(e1) == mkt_side { (e1, e2, nl * -1.0) } else { (e2, e1, nl) };
    let ok_end = |p: Vec2| {
        p.x >= 40.0
            && p.y >= 40.0
            && p.x <= site.wm - 40.0
            && p.y <= site.hm - 40.0
            && !site.is_water(p)
            && site.river_dist(p) >= site.river_w / 2.0 + 0.5
    };
    if !ok_end(near) || !ok_end(far) || site.bank_side(near) == site.bank_side(far) {
        return None;
    }
    if !far_bank_buildable(site, far) {
        return None;
    }
    // The near end's join: the nearest live street, lane or primary node on
    // the market's bank, joined by a dry straight street.
    let mut join: Option<(Vec2, f64)> = None;
    for e in g.edges.iter().filter(|e| e.alive && matches!(e.cls, "street" | "lane" | "primary")) {
        for nid in [e.a, e.b] {
            let p = g.nodes[nid].pt();
            let d = p.dist(near);
            if d <= BRIDGE_JOIN_M && site.bank_side(p) == mkt_side && join.is_none_or(|(_, bd)| d < bd) && dry_line(site, near, p) {
                join = Some((p, d));
            }
        }
    }
    let Some((jp, _)) = join else { return None };
    g.add_street(near.x, near.y, far.x, far.y, "primary", 6.0, 0, TOWN_BRIDGE_PROV);
    if jp.dist(near) > 1.0 {
        g.add_street(near.x, near.y, jp.x, jp.y, "street", 4.0, 0, TOWN_BRIDGE_APPROACH_PROV);
    }
    let stub = far + out_dir * BRIDGE_FAR_STUB_M;
    if ok_end(stub) && dry_line(site, far, stub) {
        g.add_street(far.x, far.y, stub.x, stub.y, "street", 4.0, 0, TOWN_BRIDGE_APPROACH_PROV);
    }
    Some(far)
}

#[cfg(test)]
mod tests;
