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

use crate::geom::{Vec2, dist_pt_seg, seg_int, simplify};
use crate::graph::Graph;
use crate::site::Site;

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

/// The provenance string of a tie from a street's end to the embankment.
const TIE_PROV: &str = "Riverbank tie (not the reference; 2026-11-15): a street that ended near the \
     bank, carried onto the embankment.";

/// What [`build_embankment`] laid: each bank run's polyline (before the
/// graph's planar snapping, as `buildHarbour` returns its quay) and how many
/// street ends it tied on. Empty when the town reaches neither bank.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Embankment {
    /// Each bank run as laid, simplified, before `add_street` snapped it.
    pub runs: Vec<Vec<Vec2>>,
    /// Street ends tied onto the embankment.
    pub ties: usize,
    /// Houses `generate` swept because they reached into the water
    /// ([`crate::site::Site::poly_in_water`]); filled in after the last house
    /// stage, 0 from [`build_embankment`] itself.
    pub wet_swept: usize,
    /// Field and pasture strips `generate` swept for crossing the water; 0
    /// from [`build_embankment`] itself.
    pub fields_swept: usize,
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
    let clear = site.river_w / 2.0 + EMBANKMENT_CLEAR + EMBANKMENT_W / 2.0 - 1e-6;
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
/// points becomes a `quay` street ([`EMBANKMENT_W`]) through the graph's own
/// [`Graph::add_street`], segment by segment, keeping only dry segments on
/// their own bank. Then every `street` or `lane` dead end on a bank within
/// [`EMBANKMENT_REACH`] of that bank's embankment is tied to its nearest point
/// with a `street` of the same width, if the tie is dry.
///
/// **Must only be called on a riverbank site** ([`Site::riverbank`]): it reads
/// the channel through [`Site::river_dist`] and [`Site::is_water`], which are
/// the channel only there. It must never lay a street in the water, across the
/// river, or outside the box, and it adds edges only (it kills none), so every
/// street the reference grew is still there. It must never cross the town
/// wall (`wall`, the circuit `grow` built): an embankment or tie segment that
/// would is not laid, so no street passes the curtain without a gate (the
/// embankment is a `quay`, which the rampart sweep `clear_fort_zone` exempts).
pub fn build_embankment(site: &Site, g: &mut Graph, wall: Option<&[Vec2]>) -> Embankment {
    let mut out = Embankment::default();
    let r = &site.river;
    if !site.riverbank() || r.len() < 2 {
        return out;
    }
    let off = site.river_w / 2.0 + EMBANKMENT_CLEAR + EMBANKMENT_W / 2.0;
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
        // Bank points, `None` where the town does not reach that stretch.
        let bank: Vec<Option<Vec2>> = fine
            .iter()
            .map(|&(c, nl)| {
                let q = c + nl * (off * side);
                town.iter().any(|&(p, s)| s == side && p.dist(q) <= EMBANKMENT_REACH).then_some(q)
            })
            .collect();
        let mut run: Vec<Vec2> = Vec::new();
        let mut runs: Vec<Vec<Vec2>> = Vec::new();
        for q in bank.iter().copied().chain(std::iter::once(None)) {
            match q {
                Some(q) if run.last().is_none_or(|&l| dry_on_side(site, l, q, side)) => run.push(q),
                Some(q) => {
                    runs.push(std::mem::take(&mut run));
                    run.push(q);
                }
                None => runs.push(std::mem::take(&mut run)),
            }
        }
        for run in runs.into_iter().filter(|r| r.len() >= 2) {
            let s = simplify(&run, 0.8);
            for w in s.windows(2) {
                if dry_on_side(site, w[0], w[1], side) && !crosses_wall(w[0], w[1], wall) {
                    g.add_street(w[0].x, w[0].y, w[1].x, w[1].y, "quay", EMBANKMENT_W, 0, EMBANKMENT_PROV);
                }
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
                for w in s.windows(2) {
                    let d = dist_pt_seg(p, w[0], w[1]);
                    if d < bd {
                        bd = d;
                        best = Some(closest_on_seg(p, w[0], w[1]));
                    }
                }
                // A street end already on the embankment needs no tie.
                if let Some(q) = best.filter(|_| bd > 1.0) {
                    if dry_on_side(site, p, q, side) && !crosses_wall(p, q, wall) {
                        g.add_street(p.x, p.y, q.x, q.y, "street", EMBANKMENT_W, 0, TIE_PROV);
                        out.ties += 1;
                    }
                }
            }
            out.runs.push(s);
        }
    }
    out
}

#[cfg(test)]
mod tests;
