//! **Not the reference** -- Ruling FN (owner, 2026-11-16): *"river towns may
//! stretch along their river beyond the square town box where terrain
//! allows"*. A large river town is laid out in a box elongated to fit its
//! river, and grows in a river-following frame: the market's distance (the
//! growth front, the Clark density gradient) is read along the river and
//! across it at different scales, so the town reaches farther along its banks
//! and less far inland.
//!
//! # Why
//!
//! `TOWN_GENERATOR_JUDGEMENT.md` change 5's bar asks for 15 % of a town's
//! houses within two street widths of its bank. The town leftovers batch
//! (`0e4af349`) found it out of reach for a compact town: one row on the water
//! is about 580 houses (6.4 km of bank) for a town of 20 000, while a town
//! grown round its market reaches at most `max_rf` (720 m) along the river
//! each way. The share of a fully lined river front is about the bank row's
//! depth over the town's depth across the river, whatever the town's size,
//! so a town reaches the bar by being shallower across its river and longer
//! along it, not by being bigger.
//!
//! # What
//!
//! - [`plan_river_stretch`] sizes the stretch from the town's population and
//!   the bank it can build on: a target depth across the river
//!   ([`STRETCH_ACROSS_M`]), the along reach that keeps the town's area
//!   (`max_rf^2 / across`), capped at [`STRETCH_ALONG_MAX_RF`] times `max_rf`,
//!   then cut on each side of the market where walking the centreline finds
//!   the river entering open water, both banks unbuildable for
//!   [`STRETCH_GAP_M`], another settlement's reach, or the half-way line to it.
//! - [`StretchFrame::dist`] is the distance the growth reads in place of
//!   `o.dist(market)`: `max_rf * hypot(s / along, t / across)`, with `s` the
//!   arc length along the centreline from the market's foot (`along` the
//!   reach on that side) and `t` the distance from the centreline. With both
//!   reaches and the depth equal to `max_rf` it is the plain distance near
//!   the river.
//! - [`stretch_half_box`] is the box the host lays the town out in: the
//!   stretched region's extent from the market plus [`STRETCH_BOX_MARGIN_M`].
//!
//! # What it must never do
//!
//! Reach a town that is not a river town of [`STRETCH_MIN_POP`] and up (the
//! host decides; villages are untouched), cross into another settlement's
//! reach or past the half-way line to it, or run into open water. With
//! `GenOpts::river_stretch` `None` nothing here is read, so every JS-captured
//! golden is unmoved by control flow.

use crate::districts::TERRAIN_BUILD_MIN;
use crate::geom::Vec2;
use crate::site::{Site, terrain_suitability};

/// The smallest town that stretches: the chartered-town line
/// ([`crate::plaza::PLAZA_MARKET_POP`]), the judgement's own class for the
/// 15 % bar ("towns of 1 500 and up"). Villages keep their street-village
/// layout (Ruling FE item 2).
pub const STRETCH_MIN_POP: f64 = crate::plaza::PLAZA_MARKET_POP;

/// The target half-depth across the river, metres: how far inland of its
/// river a stretched town reaches. **A labelled judgement**, set by
/// measurement against the owner's "must not look silly" bar: the share of a
/// fully lined river front is about the bank row's depth over the town's
/// depth, so a shallower town reaches more of the 15 % bar, but 300 m (with a
/// 2 x `max_rf` cap) drew ribbon suburbs and a noodle on the census, 6.7 %
/// riverside; 450 m (with 1.5 x) drew elongated towns in the City Viewer and
/// 5.1 % (`cartalith-native/docs/evidence/river-stretch-2026-11-16/`). Never
/// deeper than the town's own `max_rf` (a small town is not widened), so in
/// practice only towns of about 5 000 and up stretch.
pub const STRETCH_ACROSS_M: f64 = 450.0;

/// The longest reach along the river on either side of the market, as a
/// multiple of `max_rf`. **A labelled judgement** against the owner's bar that
/// a stretched town must not "look silly" (a noodle): with
/// [`STRETCH_ACROSS_M`] it holds a town of 20 000 to about 2.2 km along its
/// river by about 1 km across it (2.0 was measured as a noodle; see
/// [`STRETCH_ACROSS_M`]).
pub const STRETCH_ALONG_MAX_RF: f64 = 1.5;

/// The step, metres, at which [`plan_river_stretch`] walks the centreline.
/// About two lots' frontage: finer than any gap that matters to it.
pub const STRETCH_STEP_M: f64 = 20.0;

/// How far beyond half the river's width, metres, the walk probes each bank
/// for buildable ground: the reference's own flood margin
/// (`river_w / 2 + 30`, `terrain_suitability`'s `margin`) plus a lot depth,
/// so the probe stands where the first row of houses would.
pub const STRETCH_BANK_PROBE_M: f64 = 45.0;

/// How long a run of the river with neither bank buildable ends the stretch
/// on that side, metres. **A labelled judgement**: a short gorge or marsh
/// stays inside a town (a street bridges it); one longer than about three
/// blocks would leave a detached suburb, so the town stops before it.
pub const STRETCH_GAP_M: f64 = 120.0;

/// How far short of another settlement's reach the stretch stops, metres.
/// **A labelled judgement**: about a street block, so two towns never meet.
pub const STRETCH_NEIGHBOUR_GAP_M: f64 = 120.0;

/// The margin, metres, added round the stretched region for its layout box
/// ([`stretch_half_box`]). The same 150 m the reference's own box leaves
/// between its 720 m largest `max_rf` and its 850 m half-width, rounded.
pub const STRETCH_BOX_MARGIN_M: f64 = 150.0;

/// A stretch is used only when the mean along reach exceeds `max_rf` by this
/// factor; a smaller one would change a town's shape by less than a block.
pub const STRETCH_MIN_GAIN: f64 = 1.15;

/// How a river town is stretched (Ruling FN): its reach along the river in
/// each direction from the market's foot on the centreline, and its depth
/// across it, metres. `ahead_m` is toward the centreline's last vertex (the
/// traced river runs downstream, so downstream), `behind_m` toward its first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StretchPlan {
    pub ahead_m: f64,
    pub behind_m: f64,
    pub across_m: f64,
}

/// Another settlement, for [`plan_river_stretch`]: its position in the same
/// local box metres as the site, and its own reach (its `max_rf`,
/// [`crate::generate::town_reach_m`]), metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StretchNeighbour {
    pub at: Vec2,
    pub reach_m: f64,
}

/// The river-following frame a stretched town grows in ([`StretchPlan`] over
/// the site's centreline). Built once per layout by [`StretchFrame::new`].
#[derive(Debug, Clone, PartialEq)]
pub struct StretchFrame {
    river: Vec<Vec2>,
    /// Arc length at each vertex of `river`.
    cum: Vec<f64>,
    /// Arc length of the market's foot.
    s0: f64,
    plan: StretchPlan,
    max_rf: f64,
}

/// The foot of `p` on polyline `river` (at least two vertices): `(arc length
/// of the foot, distance to it, unit tangent there)`. Ties keep the first
/// segment, so the answer is deterministic.
fn foot(river: &[Vec2], cum: &[f64], p: Vec2) -> (f64, f64, Vec2) {
    let mut best = (f64::INFINITY, 0.0, Vec2::new(1.0, 0.0));
    for k in 0..river.len() - 1 {
        let (a, b) = (river[k], river[k + 1]);
        let ab = b - a;
        let l2 = ab.dot(ab);
        if l2 <= 1e-12 {
            continue;
        }
        let t = ((p - a).dot(ab) / l2).clamp(0.0, 1.0);
        let q = a + ab * t;
        let d = q.dist(p);
        if d < best.0 {
            best = (d, cum[k] + t * l2.sqrt(), ab * (1.0 / l2.sqrt()));
        }
    }
    (best.1, best.0, best.2)
}

/// Cumulative arc length of a polyline.
fn arc(river: &[Vec2]) -> Vec<f64> {
    let mut cum = Vec::with_capacity(river.len());
    let mut s = 0.0;
    for (k, p) in river.iter().enumerate() {
        if k > 0 {
            s += river[k - 1].dist(*p);
        }
        cum.push(s);
    }
    cum
}

/// The point at arc length `s` on `river` (clamped to its ends), and the unit
/// tangent there.
fn at_arc(river: &[Vec2], cum: &[f64], s: f64) -> (Vec2, Vec2) {
    let n = river.len();
    let s = s.clamp(0.0, cum[n - 1]);
    let mut k = 0;
    while k + 2 < n && cum[k + 1] < s {
        k += 1;
    }
    let (a, b) = (river[k], river[k + 1]);
    let l = cum[k + 1] - cum[k];
    let t = if l > 1e-12 { (s - cum[k]) / l } else { 0.0 };
    let dir = if l > 1e-12 { (b - a) * (1.0 / l) } else { Vec2::new(1.0, 0.0) };
    (a.lerp(b, t), dir)
}

impl StretchFrame {
    /// The frame for `plan` over `river` (the site's centreline), with the
    /// market `market` and the town's `max_rf`. `None` for a river of fewer
    /// than two vertices or zero length.
    pub fn new(river: &[Vec2], market: Vec2, plan: StretchPlan, max_rf: f64) -> Option<Self> {
        if river.len() < 2 {
            return None;
        }
        let cum = arc(river);
        if !(cum[cum.len() - 1] > 0.0) {
            return None;
        }
        let (s0, _, _) = foot(river, &cum, market);
        Some(Self { river: river.to_vec(), cum, s0, plan, max_rf })
    }

    /// The stretched distance of `p` from the market, in metres of the
    /// unstretched town (`max_rf` is the edge of the stretched region either
    /// way): `max_rf * hypot(s / reach, t / across)`, where `s` is the arc
    /// length from the market's foot to `p`'s foot (its reach the plan's
    /// `ahead_m` or `behind_m` by sign) and `t` is `p`'s distance from the
    /// centreline. Read by `crate::growth::grow` and `grow_far_bank` in place
    /// of the distance to the market.
    pub fn dist(&self, p: Vec2) -> f64 {
        let (s, t, _) = foot(&self.river, &self.cum, p);
        let ds = s - self.s0;
        let reach = if ds >= 0.0 { self.plan.ahead_m } else { self.plan.behind_m };
        self.max_rf * (ds / reach).hypot(t / self.plan.across_m)
    }

    /// The plan this frame was built from.
    pub fn plan(&self) -> StretchPlan {
        self.plan
    }
}

/// Is the ground at `p` buildable for the walk: dry and at least
/// [`TERRAIN_BUILD_MIN`] on the reference's own suitability (slope and flood,
/// the bar a lot is left vacant under).
fn buildable(site: &Site, p: Vec2) -> bool {
    !site.is_water(p) && terrain_suitability(site, p) >= TERRAIN_BUILD_MIN
}

/// **Ruling FN**: how a river town of population target `pop_target` and
/// growth radius `max_rf`, its market at `market` on `site`, is stretched
/// along its river, or `None` when it is not (a site with no centreline of two
/// vertices, a town under [`STRETCH_MIN_POP`], one already no deeper than
/// [`STRETCH_ACROSS_M`], or one whose bank allows no gain of
/// [`STRETCH_MIN_GAIN`]).
///
/// The target: depth `across = min(max_rf, STRETCH_ACROSS_M)` and reach
/// `max_rf^2 / across` on each side (the town's area kept), at most
/// [`STRETCH_ALONG_MAX_RF`]` * max_rf`. Then each side is walked along the
/// centreline every [`STRETCH_STEP_M`] from the market's foot and cut where
///
/// - the centreline leaves the box less [`STRETCH_BOX_MARGIN_M`],
/// - the centreline point is open water (a lake or the sea the river runs
///   into; the channel itself is not open water),
/// - neither bank, [`STRETCH_BANK_PROBE_M`] past half the river's width, is
///   [`buildable`] for more than [`STRETCH_GAP_M`] running,
/// - the point is within a neighbour's reach plus
///   [`STRETCH_NEIGHBOUR_GAP_M`], or nearer that neighbour than the market.
///
/// A side cut short is never shorter than `across` (the town's end stays
/// round), and the depth is widened back toward `max_rf` to keep the area
/// when the sides are cut (`max_rf^2 / mean reach`).
pub fn plan_river_stretch(site: &Site, market: Vec2, max_rf: f64, pop_target: f64, neighbours: &[StretchNeighbour]) -> Option<StretchPlan> {
    let river = &site.river;
    if river.len() < 2 || pop_target < STRETCH_MIN_POP || !(max_rf > STRETCH_ACROSS_M) {
        return None;
    }
    let cum = arc(river);
    let total = cum[cum.len() - 1];
    if !(total > 0.0) {
        return None;
    }
    let (s0, _, _) = foot(river, &cum, market);
    let across0 = STRETCH_ACROSS_M;
    let want = (max_rf * max_rf / across0).min(STRETCH_ALONG_MAX_RF * max_rf);
    let probe = site.river_w / 2.0 + STRETCH_BANK_PROBE_M;
    let in_box = |p: Vec2| {
        p.x >= STRETCH_BOX_MARGIN_M && p.y >= STRETCH_BOX_MARGIN_M && p.x <= site.wm - STRETCH_BOX_MARGIN_M && p.y <= site.hm - STRETCH_BOX_MARGIN_M
    };
    let open_water = |p: Vec2| site.open_water_at(p);
    let crowded = |p: Vec2| {
        neighbours.iter().any(|n| p.dist(n.at) < n.reach_m + STRETCH_NEIGHBOUR_GAP_M || p.dist(n.at) < p.dist(market))
    };
    let walk = |sign: f64| -> f64 {
        let mut gap = 0.0;
        let mut last_ok = 0.0;
        let mut d = STRETCH_STEP_M;
        while d <= want {
            let s = s0 + sign * d;
            if s < 0.0 || s > total {
                break;
            }
            let (c, dir) = at_arc(river, &cum, s);
            if !in_box(c) || open_water(c) || crowded(c) {
                break;
            }
            let nl = dir.rot90();
            if buildable(site, c + nl * probe) || buildable(site, c + nl * -probe) {
                gap = 0.0;
                last_ok = d;
            } else {
                gap += STRETCH_STEP_M;
                if gap > STRETCH_GAP_M {
                    break;
                }
            }
            d += STRETCH_STEP_M;
        }
        last_ok
    };
    let (fwd, back) = (walk(1.0), walk(-1.0));
    let mean = (fwd + back) / 2.0;
    if !(mean > max_rf * STRETCH_MIN_GAIN) {
        return None;
    }
    let across = (max_rf * max_rf / mean).clamp(across0, max_rf);
    Some(StretchPlan { ahead_m: fwd.max(across), behind_m: back.max(across), across_m: across })
}

/// How far from the channel's edge, metres, [`lay_river_spine`] lays its
/// street: the middle of the waterfront back street's own band
/// (`crate::riverbank::BACK_STREET_M`, 35 to 45 m), so the waterfront pass
/// (`crate::riverbank::lay_back_street`) takes the spine as its back line and
/// lays the water lanes and the embankment in front of it.
pub const SPINE_BANK_M: f64 = 40.0;

/// The spine's sample spacing along the river, metres. **A labelled
/// judgement**: about two lots' frontage, so the street follows the river's
/// bends without short segments (the growth skips edges under 38 m as
/// branch origins, M-NET-4).
pub const SPINE_STEP_M: f64 = 40.0;

/// The spine's width, metres: the reference's own primary width
/// (`crate::routes::build_primaries`, 7 m), since it is the town's road along
/// its river.
pub const SPINE_W: f64 = 7.0;

/// Provenance of [`lay_river_spine`]'s street.
pub const SPINE_PROV: &str = "River road: a stretched river town's street along its bank (not the reference; Ruling FN, 2026-11-16).";

/// **Ruling FN**: lay a stretched river town's road along its river, on the
/// market's bank, before the town grows, and return the edges laid.
///
/// The line runs [`SPINE_BANK_M`] back from the channel's edge (half `river_w`
/// plus that from the centreline), sampled every [`SPINE_STEP_M`] of the
/// centreline from `behind_m` to `ahead_m` of the market's foot, and is cut to
/// the run of points round the foot that are dry, buildable (as the plan's
/// walk tests) and on the market's bank, and that stay inside the box
/// (`STRETCH_BOX_MARGIN_M` in). It is a `primary` of [`SPINE_W`].
///
/// **Why.** The growth (`crate::growth::grow`) is a branching tree off the
/// streets that exist; a disc grows round the market's primaries on its own,
/// but a town reaches along its river only along a street that runs there,
/// and once walled its extramural growth clings to the primaries. A linear
/// river town is a road along its river with lanes off it, and the waterfront
/// pass needs a back line along the bank to close riverside blocks against
/// the water (measured on the fixture `stretch::tests`, a 300 m deep, 1 300 m
/// reach plan: without the spine the stretched town housed 6 776 people
/// against the unstretched town's 10 795, 63 %).
///
/// Must never lay an edge in the water, across the river, or outside the box,
/// and lays nothing on a site with no stretch frame.
pub fn lay_river_spine(site: &Site, g: &mut crate::graph::Graph, market: Vec2) -> Vec<usize> {
    let Some(f) = site.stretch() else { return Vec::new() };
    let side = site.bank_side(market);
    let off = site.river_w / 2.0 + SPINE_BANK_M;
    let ok = |p: Vec2| {
        p.x >= STRETCH_BOX_MARGIN_M
            && p.y >= STRETCH_BOX_MARGIN_M
            && p.x <= site.wm - STRETCH_BOX_MARGIN_M
            && p.y <= site.hm - STRETCH_BOX_MARGIN_M
            && buildable(site, p)
            && site.bank_side(p) == side
    };
    let point = |d: f64| -> Vec2 {
        let (c, dir) = at_arc(&f.river, &f.cum, f.s0 + d);
        let nl = dir.rot90();
        let a = c + nl * off;
        if site.bank_side(a) == side { a } else { c + nl * -off }
    };
    // Outward from the foot each way, stopping at the first point refused.
    let mut back: Vec<Vec2> = Vec::new();
    let mut d = 0.0;
    while d <= f.plan.behind_m {
        let p = point(-d);
        if !ok(p) {
            break;
        }
        back.push(p);
        d += SPINE_STEP_M;
    }
    let mut pts: Vec<Vec2> = back.into_iter().rev().collect();
    let mut d = SPINE_STEP_M;
    while d <= f.plan.ahead_m {
        let p = point(d);
        if !ok(p) {
            break;
        }
        pts.push(p);
        d += SPINE_STEP_M;
    }
    // Every segment dry: a bend's chord must not cut the channel.
    let dry = |a: Vec2, b: Vec2| (1..4).all(|k| !site.is_water(a.lerp(b, k as f64 / 4.0)));
    if pts.len() < 2 || !pts.windows(2).all(|w| dry(w[0], w[1])) {
        return Vec::new();
    }
    g.add_polyline_street(&pts, "primary", SPINE_W, 0, SPINE_PROV)
}

/// Half the width and height, metres, of the box a town stretched by `plan`
/// from `market` along `river` is laid out in: the farthest the stretched
/// region (the centreline from `behind_m` to `ahead_m` of the market's foot,
/// `across_m` either side of it, and a circle of `across_m` round the market)
/// reaches from the market in x and in y, plus [`STRETCH_BOX_MARGIN_M`].
/// Never smaller than half the reference's box ([`crate::generate::SITE_WM`]
/// by [`crate::generate::SITE_HM`]), so a stretched town's box only grows.
pub fn stretch_half_box(river: &[Vec2], market: Vec2, plan: StretchPlan) -> (f64, f64) {
    let (mut hx, mut hy) = (plan.across_m, plan.across_m);
    if river.len() >= 2 {
        let cum = arc(river);
        let (s0, _, _) = foot(river, &cum, market);
        let mut d = -plan.behind_m;
        while d <= plan.ahead_m {
            let (c, dir) = at_arc(river, &cum, s0 + d);
            let nl = dir.rot90();
            for q in [c + nl * plan.across_m, c + nl * -plan.across_m] {
                hx = hx.max((q.x - market.x).abs());
                hy = hy.max((q.y - market.y).abs());
            }
            d += STRETCH_STEP_M;
        }
    }
    (
        (hx + STRETCH_BOX_MARGIN_M).max(crate::generate::SITE_WM / 2.0),
        (hy + STRETCH_BOX_MARGIN_M).max(crate::generate::SITE_HM / 2.0),
    )
}

#[cfg(test)]
mod tests;
