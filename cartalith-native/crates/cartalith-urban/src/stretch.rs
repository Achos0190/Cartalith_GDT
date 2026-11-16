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
//! - [`stretch_holds`] (Ruling FQ, owner 2026-11-16) judges a finished
//!   stretched layout against the ruling's bars ([`StretchMeasures`]); the
//!   host keeps it, tries [`StretchPlan::milder`], or lays the town out
//!   unstretched (`cartalith_civ::urban_adapter::run_layout`).
//!
//! # What it must never do
//!
//! Reach a town that is not a river town of [`STRETCH_MIN_POP`] and up (the
//! host decides; villages are untouched), cross into another settlement's
//! reach or past the half-way line to it, or run into open water; nor, once
//! judged ([`stretch_holds`]), leave a town worse on Ruling FQ's bars than
//! both their bounds and its own unstretched layout. With
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

/// The least share of its target, `pop / pop_target`, a stretched layout
/// must house for [`stretch_holds`]. **A labelled judgement** from Ruling FQ's
/// bar (owner, 2026-11-16: housed p10 and the towns under 80 % back to the
/// pre-stretch levels, 89 % p10 at 384 x 288 and 82 % at 2 048 x 1 311): a
/// floor above both p10s means a stretched town can never be one of the
/// towns that pulls either down. Measured on the prototype
/// (`cartalith-native/docs/evidence/river-stretch-2026-11-16/`): the towns
/// it made worse housed 54 to 82 %.
pub const STRETCH_HOUSED_MIN: f64 = 0.9;

/// The longest wall a stretched layout may have, as its length over its own
/// convex hull's ([`stretch_holds`]). **A labelled judgement** from Ruling
/// FQ's bar that the wall / hull p90 must not get worse: 1.12 is the
/// pre-stretch p90 at 384 x 288, the lower of the two sizes' (1.16 at
/// 2 048), so a stretched town is never one of the towns above either p90.
/// The prototype's worst were 1.22 (246371 #15 at 384, 483920 #5 at 2 048).
pub const STRETCH_WALL_HULL_MAX: f64 = 1.12;

/// The largest unbuilt patch, square metres, a stretched layout's wall may
/// enclose ([`stretch_holds`], [`unbuilt_patch_m2`]). **A labelled
/// judgement**: half of Ruling FQ's 1 ha bar ("no new unbuilt walled patch
/// over 1 ha"), so a stretched town never comes near the census's line. The
/// prototype's 483920 #12 at 384 walled 35.5 ha of empty ground.
pub const STRETCH_UNBUILT_MAX_M2: f64 = 5_000.0;

/// How far a stretched town's square (its market square's centre, else the
/// market) may stand outside its wall, metres ([`stretch_holds`]): the
/// census's own street width (`town_river_census.rs`, `BANK_STREET_W`, the
/// widest street the growth lays), since a river town's market is the
/// bridge foot, on the bank the water wall follows.
pub const STRETCH_SQUARE_OUT_M: f64 = 5.4;

/// The grid step of [`unbuilt_patch_m2`], metres: under the narrowest lot
/// frontage the platting allows (`build_parcels`' 4.5 m floor), so a row of
/// houses cannot slip between two cells. The same step as the census's
/// `unbuilt_inside_wall`, so the gate reads ground as the bar does.
pub const UNBUILT_CELL_M: f64 = 4.0;

/// How far from every building a point inside the wall must be to count as
/// unbuilt, metres ([`unbuilt_patch_m2`]): a deep burgage plot (about 40 m)
/// has its house at the street and its yard behind, so ground farther than
/// that from every house is open land, not a back garden. The census's
/// `unbuilt_inside_wall` reach.
pub const UNBUILT_REACH_M: f64 = 40.0;

/// What [`stretch_holds`] reads off a finished layout: the four measures of
/// Ruling FQ's bars, each as the census reads it
/// (`town_river_census.rs::both_banks_audit`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StretchMeasures {
    /// Housed people over the target, `pop / pop_target`.
    pub housed: f64,
    /// For a walled town (a ring of three points and up, with houses), its
    /// wall's length over its own convex hull's, its square's standing (`true`
    /// when the square, else the market, is outside the wall by more than
    /// [`STRETCH_SQUARE_OUT_M`]) and its largest unbuilt patch inside the
    /// wall, square metres ([`unbuilt_patch_m2`]). `None` for an unwalled
    /// town, which none of the three applies to.
    pub wall: Option<(f64, bool, f64)>,
}

impl StretchMeasures {
    /// The measures of the finished layout `t`.
    pub fn of(t: &crate::generate::Town) -> Self {
        let housed = if t.pop_target > 0.0 { t.pop / t.pop_target } else { 0.0 };
        let wall = t.wall.ring.as_ref().filter(|r| r.len() >= 3 && !t.buildings.is_empty()).map(|ring| {
            let wall_hull = loop_len(ring) / loop_len(&crate::geom::convex_hull(ring)).max(1e-9);
            let sq = t.plaza.as_ref().map_or(t.anchors.market, |z| z.center);
            let ring_d = (0..ring.len()).map(|k| crate::geom::dist_pt_seg(sq, ring[k], ring[(k + 1) % ring.len()])).fold(f64::INFINITY, f64::min);
            let out = !crate::geom::point_in_poly(sq, ring) && ring_d > STRETCH_SQUARE_OUT_M;
            let plaza: &[Vec2] = t.plaza.as_ref().map_or(&[], |z| z.poly.as_slice());
            (wall_hull, out, unbuilt_patch_m2(ring, &t.buildings, &t.site.river, t.site.river_w, &t.site.water_poly, plaza))
        });
        Self { housed, wall }
    }
}

/// **Ruling FQ** (owner, 2026-11-16): whether a stretched layout measured `s`
/// is kept, given the same town's unstretched layout measured `plain` where
/// its host has laid it out (`None` before it has). Each of Ruling FQ's four
/// bars is met when the stretched layout is inside the bar's own bound **or**
/// no worse than the unstretched layout on it:
///
/// - housed at least [`STRETCH_HOUSED_MIN`] of its target, or no less than
///   the unstretched layout;
/// - and if walled: a wall no longer than [`STRETCH_WALL_HULL_MAX`] times its
///   own convex hull, or than the unstretched wall's share (an unwalled
///   unstretched town allows only the bound); its square inside the wall
///   (within [`STRETCH_SQUARE_OUT_M`]), or the unstretched square outside
///   too; no unbuilt patch inside the wall over [`STRETCH_UNBUILT_MAX_M2`], or
///   none larger than the unstretched wall's.
///
/// **Why a test on the result, and why "or no worse".** The prototype's
/// census (Ruling FN, `496c7a2d`) found the stretch worse on a minority of
/// towns, each for a reason of its own site: a market at a sharp river bend
/// walls the inside of the bend (483920 #12 at 384), a town whose ground runs
/// out along the river houses short (246371 #2 at 2 048: 54 %), a town
/// wrapped round the outside of a bend gets a wall bent round it (246371 #15
/// at 384: 1.22). Ruling FQ's bars are on the census's whole population (the
/// housed p10 and the towns under 80 %, the wall / hull p90, the squares
/// outside their wall, the patches over 1 ha), and a town whose value is
/// either inside a bound tighter than the population's bar or no worse than
/// its own unstretched value can raise none of them: each order statistic of
/// the new population is bounded by the larger of the bound and the old one's
/// (the bounds sit at or inside the pre-stretch p10s and p90s, see each
/// constant). The host asks without `plain` first, so a town that clears the
/// bounds is laid out once.
///
/// Must never be asked for an unstretched town (its host does not), and
/// reads nothing but the two finished layouts' measures.
pub fn stretch_holds(s: &StretchMeasures, plain: Option<&StretchMeasures>) -> bool {
    let housed_ok = s.housed >= STRETCH_HOUSED_MIN || plain.is_some_and(|p| s.housed >= p.housed);
    if !housed_ok {
        return false;
    }
    let Some((wh, out, patch)) = s.wall else { return true };
    let pw = plain.and_then(|p| p.wall);
    let wall_ok = wh <= STRETCH_WALL_HULL_MAX || pw.is_some_and(|(pwh, _, _)| wh <= pwh);
    let square_ok = !out || pw.is_some_and(|(_, pout, _)| pout);
    let patch_ok = patch <= STRETCH_UNBUILT_MAX_M2 || pw.is_some_and(|(_, _, pp)| patch <= pp);
    wall_ok && square_ok && patch_ok
}

/// The share of a plan's reach beyond `max_rf` a second, milder try keeps
/// ([`StretchPlan::milder`]). **A labelled judgement** (Ruling FQ): half
/// way between the planned stretch and the unstretched town, so a town
/// whose full stretch fails [`stretch_holds`] (most often a wall bent round a
/// river bend, or houses run short along it) is tried once more nearer its
/// plain shape before it is laid out unstretched.
pub const STRETCH_RETRY_SHARE: f64 = 0.5;

impl StretchPlan {
    /// The milder plan of a town of reach `max_rf`: each side's reach beyond
    /// `max_rf` cut to [`STRETCH_RETRY_SHARE`] of it (a side already no
    /// longer than `max_rf` kept), and the depth widened back to keep the
    /// area (`max_rf^2 / mean reach`, between this plan's depth and
    /// `max_rf`; each side then at least that depth). `None` when the milder
    /// plan gains less than
    /// [`STRETCH_MIN_GAIN`], as [`plan_river_stretch`] drops such a plan.
    pub fn milder(&self, max_rf: f64) -> Option<StretchPlan> {
        let cut = |r: f64| if r > max_rf { max_rf + (r - max_rf) * STRETCH_RETRY_SHARE } else { r };
        let (ahead, behind) = (cut(self.ahead_m), cut(self.behind_m));
        let mean = (ahead + behind) / 2.0;
        if !(mean > max_rf * STRETCH_MIN_GAIN) {
            return None;
        }
        let across = (max_rf * max_rf / mean).clamp(self.across_m, max_rf.max(self.across_m));
        // As in `plan_river_stretch`: a side is never shorter than the depth,
        // so the town's end stays round.
        Some(StretchPlan { ahead_m: ahead.max(across), behind_m: behind.max(across), across_m: across })
    }
}

/// The length of the closed loop `ring`, metres.
fn loop_len(ring: &[Vec2]) -> f64 {
    (0..ring.len()).map(|k| ring[k].dist(ring[(k + 1) % ring.len()])).sum()
}

/// **Ruling FQ**: the largest 4-connected patch, square metres, of the
/// [`UNBUILT_CELL_M`] cells inside `ring` farther than [`UNBUILT_REACH_M`]
/// from every building footprint, leaving out the river channel (`river_w /
/// 2` of the centreline, plus a cell), `water` and the market square
/// (`plaza`, open by design). Distances by a two-pass 3-4 chamfer (within
/// about 8 % of Euclidean, ample at a hectare). The same reading as the
/// census's `unbuilt_inside_wall` (`town_river_census.rs`), which must stay
/// a measurement only; this copy is the generation side's own, read by
/// [`stretch_holds`] alone.
pub fn unbuilt_patch_m2(ring: &[Vec2], buildings: &[crate::districts::Building], river: &[Vec2], river_w: f64, water: &[Vec2], plaza: &[Vec2]) -> f64 {
    if ring.len() < 3 {
        return 0.0;
    }
    let x0 = ring.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - UNBUILT_REACH_M;
    let y0 = ring.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - UNBUILT_REACH_M;
    let x1 = ring.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + UNBUILT_REACH_M;
    let y1 = ring.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + UNBUILT_REACH_M;
    let nx = ((x1 - x0) / UNBUILT_CELL_M).ceil() as usize + 1;
    let ny = ((y1 - y0) / UNBUILT_CELL_M).ceil() as usize + 1;
    let ctr = |i: usize, j: usize| Vec2::new(x0 + (i as f64 + 0.5) * UNBUILT_CELL_M, y0 + (j as f64 + 0.5) * UNBUILT_CELL_M);
    let ci = |x: f64, n: usize, o: f64| (((x - o) / UNBUILT_CELL_M).floor().max(0.0) as usize).min(n - 1);
    const INF: u32 = u32::MAX / 2;
    let mut d = vec![INF; nx * ny];
    for b in buildings.iter().filter(|b| b.poly.len() >= 3) {
        let bx0 = b.poly.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let by0 = b.poly.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let bx1 = b.poly.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let by1 = b.poly.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        for j in ci(by0, ny, y0)..=ci(by1, ny, y0) {
            for i in ci(bx0, nx, x0)..=ci(bx1, nx, x0) {
                if crate::geom::point_in_poly(ctr(i, j), &b.poly) {
                    d[j * nx + i] = 0;
                }
            }
        }
        for p in &b.poly {
            d[ci(p.y, ny, y0) * nx + ci(p.x, nx, x0)] = 0;
        }
    }
    for j in 0..ny {
        for i in 0..nx {
            let mut v = d[j * nx + i];
            if i > 0 {
                v = v.min(d[j * nx + i - 1] + 3);
            }
            if j > 0 {
                v = v.min(d[(j - 1) * nx + i] + 3);
                if i > 0 {
                    v = v.min(d[(j - 1) * nx + i - 1] + 4);
                }
                if i + 1 < nx {
                    v = v.min(d[(j - 1) * nx + i + 1] + 4);
                }
            }
            d[j * nx + i] = v;
        }
    }
    for j in (0..ny).rev() {
        for i in (0..nx).rev() {
            let mut v = d[j * nx + i];
            if i + 1 < nx {
                v = v.min(d[j * nx + i + 1] + 3);
            }
            if j + 1 < ny {
                v = v.min(d[(j + 1) * nx + i] + 3);
                if i + 1 < nx {
                    v = v.min(d[(j + 1) * nx + i + 1] + 4);
                }
                if i > 0 {
                    v = v.min(d[(j + 1) * nx + i - 1] + 4);
                }
            }
            d[j * nx + i] = v;
        }
    }
    let reach = (UNBUILT_REACH_M / UNBUILT_CELL_M * 3.0) as u32;
    let river_d = |q: Vec2| (0..river.len().saturating_sub(1)).map(|k| crate::geom::dist_pt_seg(q, river[k], river[k + 1])).fold(f64::INFINITY, f64::min);
    let open: Vec<bool> = (0..nx * ny)
        .map(|k| {
            let q = ctr(k % nx, k / nx);
            d[k] > reach
                && crate::geom::point_in_poly(q, ring)
                && !(river.len() >= 2 && river_d(q) < river_w / 2.0 + UNBUILT_CELL_M)
                && !(water.len() >= 3 && crate::geom::point_in_poly(q, water))
                && !(plaza.len() >= 3 && crate::geom::point_in_poly(q, plaza))
        })
        .collect();
    let mut seen = vec![false; nx * ny];
    let mut best = 0usize;
    for k0 in 0..nx * ny {
        if !open[k0] || seen[k0] {
            continue;
        }
        let (mut n, mut stack) = (0usize, vec![k0]);
        seen[k0] = true;
        while let Some(k) = stack.pop() {
            n += 1;
            let (i, j) = (k % nx, k / nx);
            for (ok, kk) in [(i > 0, k.wrapping_sub(1)), (i + 1 < nx, k + 1), (j > 0, k.wrapping_sub(nx)), (j + 1 < ny, k + nx)] {
                if ok && open[kk] && !seen[kk] {
                    seen[kk] = true;
                    stack.push(kk);
                }
            }
        }
        best = best.max(n);
    }
    best as f64 * UNBUILT_CELL_M * UNBUILT_CELL_M
}

#[cfg(test)]
mod tests;
