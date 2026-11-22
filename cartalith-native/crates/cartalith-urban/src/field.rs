//! **Not the reference** -- the town-type rule, the local direction field and
//! the ground gate of `PARALLEL_STREETS_SCOPE.md` milestone PS-1 (owner Rulings
//! FY, GA, GC and GF, `LARGE_ITEM_RULINGS.md`).
//!
//! What it does: for an organic-plan town of [`MIN_POP`] and up it decides
//! which of the scope's section 4 types the town is ([`TownType`]), builds the
//! grain the later milestones grow streets along ([`DirectionField`], a
//! doubled-angle blend of the town's approach roads, its river and its market
//! spine, weak inside an organic core), and measures the dry ground the town
//! could grow into ([`Ground`], section 9.3's gate).
//!
//! Why: PS-0 (scope section 8) found that a field turns the streets as
//! designed, and that the ground for a deep-plot footprint decides which towns
//! may take a grain at all. PS-2 and PS-5 read the answer here, so both read
//! one rule.
//!
//! What it must never do: **change a layout.** PS-1 is "no growth change"
//! (scope section 6): nothing here draws from the RNG, writes to the [`Graph`]
//! or to the [`Site`], or lays a street. `generate_pass` computes a
//! [`ParallelPlan`] only with `GenOpts::parallel_streets` on, and only reports
//! it on `Town::parallel`; with the switch on, every street, lot and building
//! is the switch-off layout's (pinned by `field/tests.rs`). It must never type
//! a planted grid either: no world fact says which towns were planted (Q2,
//! restated by Ruling GC), so [`TownType`] has no such variant.

use crate::geom::{dist_pt_seg, Vec2};
use crate::graph::Graph;
use crate::site::Site;

#[cfg(test)]
mod tests;

/// The smallest town the rule types: the chartered-town size class, the same
/// line the market square and the census's "towns 1 500+" use
/// (`crate::plaza::PLAZA_MARKET_POP`). Scope section 4: "an organic-plan town
/// of 1 500 and up, not a street village".
pub const MIN_POP: f64 = crate::plaza::PLAZA_MARKET_POP;

/// Scope section 4, era and size: "a town under 5 000 gets one grain (one
/// street market or one riverside), a larger one up to its number of approach
/// roads". **A labelled judgement** in the scope; the same 5 000 is the line
/// the ground recount's go bar is counted over.
pub const MULTI_GRAIN_POP: f64 = 5000.0;

/// The radius of the circle round the market on which the approach roads are
/// read ([`approach_bearings`]), metres. **A labelled judgement**: a market
/// square (up to about 60 m across with `urban_market_scale`) plus about one
/// block, so a road is read where it enters the town's centre and not where
/// it bends far out. PS-0's prototype read "a primary within 60 m of the
/// market", which every host road meets (they all lead to the market), so it
/// could not tell a through street from a road that ends there.
pub const APPROACH_PROBE_M: f64 = 100.0;

/// Two crossings of [`APPROACH_PROBE_M`]'s circle closer than this, radians,
/// are one road (a road that wiggles across the circle twice, or two primary
/// edges meeting on it). **A labelled judgement**: 20 degrees, the same turn
/// below which PS-0's block-corner count merged two corners.
pub const APPROACH_MERGE_RAD: f64 = 20.0 * std::f64::consts::PI / 180.0;

/// A road "runs through the market square" (scope section 4's market-street
/// type) when two approach bearings are at least this far apart, radians: the
/// road enters on one side and leaves on the other, within 30 degrees of
/// straight. **A labelled judgement** (twice the branch jitter's sigma, 15
/// degrees, `DEFAULT_RULES`); a crossroads whose roads all bend more than
/// that is an organic-core town.
pub const THROUGH_MIN_RAD: f64 = 150.0 * std::f64::consts::PI / 180.0;

/// Scope section 4's river-front test, "its market is within two street widths
/// of the bank": two of the widest street `grow` lays (`r.range(3.4, 5.4)`,
/// M-NET-8), the census's own riverside reach (`BANK_STREET_W` x 2 in
/// `town_river_census.rs`), metres. Measured from the market square's outline
/// where the town has one (the market is a place, not its centre point), else
/// from the market point.
pub const RIVER_FRONT_BANK_M: f64 = 2.0 * 5.4;

/// The organic core's radius as a share of `max_rf` at the default settlement
/// age (300 years, `generate`'s `settlement_age` default): `grow`'s own
/// epoch-1 radius (scope section 3.4 item 5, Q3).
pub const CORE_FRAC: f64 = 0.38;

/// How much the core's share grows from the default age to the oldest town
/// (1 000 years, `generate`'s clamp), and shrinks below it at the same rate.
/// **A labelled judgement** for Q3's "the older the town, the larger its
/// organic core" (research 2.1's planned extensions round old cores): 0.38 at
/// 300 years, 0.50 at 1 000, clamped to [`CORE_FRAC_MIN`]..[`CORE_FRAC_MAX`].
pub const CORE_FRAC_PER_YEAR: f64 = 0.12 / 700.0;
/// The youngest town's core share (see [`CORE_FRAC_PER_YEAR`]).
pub const CORE_FRAC_MIN: f64 = 0.33;
/// The oldest town's core share (see [`CORE_FRAC_PER_YEAR`]).
pub const CORE_FRAC_MAX: f64 = 0.50;

/// The width of the ramp from no grain at the core's edge to the full grain,
/// as a share of `max_rf`: PS-0's prototype (`0.38` to `0.50 max_rf`, scope
/// section 8.3), kept.
pub const RAMP_FRAC: f64 = 0.12;

/// Each source's weight and reach, PS-0's prototype (scope section 8.3), kept:
/// an approach road weighs 1 within about 80 m of itself, the river 1.5 within
/// about 80 m of its centreline, the market spine 0.7 out to about 250 m. Both
/// reaches are the half-weight distance of `w / (1 + (d / reach)^2)`.
pub const ROAD_WEIGHT: f64 = 1.0;
/// See [`ROAD_WEIGHT`].
pub const ROAD_REACH_M: f64 = 80.0;
/// See [`ROAD_WEIGHT`].
pub const RIVER_WEIGHT: f64 = 1.5;
/// See [`ROAD_WEIGHT`].
pub const RIVER_REACH_M: f64 = 80.0;
/// See [`ROAD_WEIGHT`].
pub const SPINE_WEIGHT: f64 = 0.7;
/// See [`ROAD_WEIGHT`].
pub const SPINE_REACH_M: f64 = 250.0;

/// Section 9.3's growable cell: the sample spacing, metres (PS-0 item 2's own
/// 10 m cells, so this count and PS-0's are the same instrument).
pub const GROUND_CELL_M: f64 = 10.0;
/// Section 9.3: a cell inside the box's 40 m margin (PS-0 item 2; `grow`'s own
/// box test keeps streets 40 m in).
pub const GROUND_MARGIN_M: f64 = 40.0;
/// Section 9.3: a cell no steeper than this (`grow`'s own street slope test,
/// PS-0 item 2).
pub const GROUND_MAX_SLOPE: f64 = 0.34;

/// Section 9.3's footprint the deep plots need, the median case:
/// `F_deep / (1 + rho)` with PS-0's deep-plot median footprint 1.4 (scope
/// section 3.6) and `rho` 0.25 (the scope's labelled judgement) = 1.12. The
/// scope's "about 1.1" is this figure.
pub const F_REQ_MEDIAN: f64 = 1.4 / 1.25;
/// Section 9.3's p90 case: PS-0's p90 footprint 1.68 with `rho` at its 0.40
/// ceiling (Ruling GC, Q13) = 1.2.
pub const F_REQ_P90: f64 = 1.68 / 1.40;

/// The four types the rule assigns (scope section 4; Ruling GC's split). There
/// is deliberately no planted-grid type (Q2, Ruling GC: planted grids stay
/// out).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownType {
    /// A primary runs through the market square: the spine is that road.
    MarketStreet,
    /// A real river town that is stretched along its river (Ruling FN) or
    /// whose market square is within [`RIVER_FRONT_BANK_M`] of the bank.
    RiverFront,
    /// Neither, with at least one approach road: the roads set the grains
    /// round an organic core.
    OrganicCore,
    /// No grain: the radial plan, or a town with no source at all.
    NoGrain,
}

/// FS-2's order caps for a type (scope section 5.4; Ruling GC, Q12). `None`
/// in a field means that cap does not apply to the type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fs2Caps {
    /// Street-order phi per town at most this (market-street towns: 0.35).
    pub phi_per_town: Option<f64>,
    /// Median phi over the type's towns at most this (other types: 0.25).
    pub phi_median: Option<f64>,
    /// p90 phi over the type's towns at most this (other types: 0.45).
    pub phi_p90: Option<f64>,
    /// Per town, phi with the switch on minus off at most this (market-street
    /// 0.30, Q12 (a); other types 0.20).
    pub phi_delta: f64,
}

impl TownType {
    /// The key a host or a census prints: `market_street`, `river_front`,
    /// `organic_core`, `none`.
    pub fn key(self) -> &'static str {
        match self {
            Self::MarketStreet => "market_street",
            Self::RiverFront => "river_front",
            Self::OrganicCore => "organic_core",
            Self::NoGrain => "none",
        }
    }

    /// The FS-2 caps this type is judged by (Ruling GC's split): market-street
    /// towns phi per town 0.35 and a delta of 0.30; river-front and
    /// organic-core towns a median of 0.25, a p90 of 0.45 and a delta of 0.20;
    /// a town with no grain none (nothing in it is meant to change).
    pub fn fs2_caps(self) -> Option<Fs2Caps> {
        match self {
            Self::MarketStreet => Some(Fs2Caps { phi_per_town: Some(0.35), phi_median: None, phi_p90: None, phi_delta: 0.30 }),
            Self::RiverFront | Self::OrganicCore => Some(Fs2Caps { phi_per_town: None, phi_median: Some(0.25), phi_p90: Some(0.45), phi_delta: 0.20 }),
            Self::NoGrain => None,
        }
    }
}

/// The type rule on its inputs (scope section 4), in the section's order:
/// river-front first, then market-street, then organic-core. **The order is a
/// labelled judgement with a reason**: a town that is both a river-front and a
/// market-street town takes the river-front type and so the stricter FS-2 cap
/// (0.25, not 0.35); a cap is never loosened by the order of a test.
///
/// - `organic`: the town is on the organic plan (the radial plan never calls
///   `grow`, so it never takes a grain);
/// - `river_front`: [`is_river_front`]'s answer;
/// - `through`: [`through_pair`] found a road through the market;
/// - `approaches`: how many approach roads [`approach_bearings`] read.
pub fn town_type(organic: bool, river_front: bool, through: bool, approaches: usize) -> TownType {
    if !organic {
        TownType::NoGrain
    } else if river_front {
        TownType::RiverFront
    } else if through {
        TownType::MarketStreet
    } else if approaches > 0 {
        TownType::OrganicCore
    } else {
        TownType::NoGrain
    }
}

/// The organic core's radius, metres, for a town of reach `max_rf` and age
/// `age_years` (see [`CORE_FRAC_PER_YEAR`]).
pub fn core_radius(max_rf: f64, age_years: f64) -> f64 {
    let frac = (CORE_FRAC + (age_years - 300.0) * CORE_FRAC_PER_YEAR).clamp(CORE_FRAC_MIN, CORE_FRAC_MAX);
    frac * max_rf
}

/// The bearings (radians, `atan2` of the offset from the market) at which the
/// live `primary` edges of `g` cross the circle of radius `r` round `market`,
/// sorted ascending, with crossings within [`APPROACH_MERGE_RAD`] of the
/// previous kept one dropped (one road). Reads `g`, never writes it.
pub fn approach_bearings(g: &Graph, market: Vec2, r: f64) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for e in g.edges.iter().filter(|e| e.alive && e.cls == "primary") {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        // |a + t (b - a) - m| = r, t in [0, 1].
        let (d, f) = (b - a, a - market);
        let (qa, qb, qc) = (d.dot(d), 2.0 * f.dot(d), f.dot(f) - r * r);
        if qa <= 0.0 {
            continue;
        }
        let disc = qb * qb - 4.0 * qa * qc;
        if disc < 0.0 {
            continue;
        }
        let sq = disc.sqrt();
        for t in [(-qb - sq) / (2.0 * qa), (-qb + sq) / (2.0 * qa)] {
            if (0.0..=1.0).contains(&t) {
                let p = a + d * t;
                out.push((p.y - market.y).atan2(p.x - market.x));
            }
        }
    }
    out.sort_by(f64::total_cmp);
    let mut kept: Vec<f64> = Vec::new();
    for th in out {
        if kept.last().is_none_or(|&k| ang_diff(th, k) >= APPROACH_MERGE_RAD) {
            kept.push(th);
        }
    }
    // The circle wraps: the last kept bearing may be one road with the first.
    if kept.len() >= 2 && ang_diff(kept[0], kept[kept.len() - 1]) < APPROACH_MERGE_RAD {
        kept.pop();
    }
    kept
}

/// The unsigned angle between two bearings, 0..pi.
fn ang_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(2.0 * std::f64::consts::PI);
    d.min(2.0 * std::f64::consts::PI - d)
}

/// The pair of approach bearings most nearly opposite, if they are at least
/// [`THROUGH_MIN_RAD`] apart: a road through the market. Ties keep the first
/// pair in bearing order (deterministic).
pub fn through_pair(bearings: &[f64]) -> Option<(f64, f64)> {
    let mut best: Option<(f64, (f64, f64))> = None;
    for i in 0..bearings.len() {
        for j in i + 1..bearings.len() {
            let d = ang_diff(bearings[i], bearings[j]);
            if d >= THROUGH_MIN_RAD && best.is_none_or(|(bd, _)| d > bd) {
                best = Some((d, (bearings[i], bearings[j])));
            }
        }
    }
    best.map(|b| b.1)
}

/// The shortest distance from `p` to the polyline `line`, metres (infinite
/// for a line of fewer than two points).
fn dist_to_line(p: Vec2, line: &[Vec2]) -> f64 {
    line.windows(2).map(|w| dist_pt_seg(p, w[0], w[1])).fold(f64::INFINITY, f64::min)
}

/// Scope section 4's river-front test on `site`: a real river (`real_river`,
/// a `river` or `riverthrough` site with a centreline), and the town either
/// stretched along it (Ruling FN, `Site::stretch`) or with its market square
/// (`plaza`, else the market point) within [`RIVER_FRONT_BANK_M`] of the bank
/// (`river_w / 2` off the centreline).
pub fn is_river_front(site: &Site, market: Vec2, plaza: Option<&[Vec2]>) -> bool {
    if !(site.real_river && (site.kind == "river" || site.kind == "riverthrough") && site.river.len() >= 2) {
        return false;
    }
    if site.stretch().is_some() {
        return true;
    }
    let half = site.river_w / 2.0;
    let gap = |p: Vec2| dist_to_line(p, &site.river) - half;
    let near = match plaza {
        Some(poly) if poly.len() >= 3 => poly.iter().map(|&p| gap(p)).fold(f64::INFINITY, f64::min),
        _ => gap(market),
    };
    near <= RIVER_FRONT_BANK_M
}

/// What a field source is (for a reader and for the viewer's legend).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// One approach-road (primary) edge, or an organic-core town's one grain.
    Road,
    /// The river's centreline.
    River,
    /// The market spine: the through road, or a small town's one grain.
    Spine,
}

/// One source of the field: a polyline whose local bearing it lends to the
/// points near it, with weight `weight / (1 + (d / reach_m)^2)` at distance
/// `d` from its nearest segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub kind: SourceKind,
    pub line: Vec<Vec2>,
    pub weight: f64,
    pub reach_m: f64,
}

/// The grain at a point: `axis` a bearing in radians (an axis: `axis` and
/// `axis + pi` are the same grain, and the cross-direction is `axis + pi/2`),
/// and `strength` 0..1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grain {
    pub axis: f64,
    pub strength: f64,
}

/// The local direction field (scope section 3.4 item 1; Chen et al. 2008's
/// tensor field, blended by distance as PS-0's prototype did): every source
/// adds its nearest segment's orientation as a doubled angle, so opposite
/// bearings agree and crossing ones cancel.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectionField {
    pub sources: Vec<Source>,
    /// The organic core's radius, metres ([`core_radius`]): no grain inside it.
    pub core_m: f64,
    /// The ramp's width, metres ([`RAMP_FRAC`] of `max_rf`).
    pub ramp_m: f64,
}

impl DirectionField {
    /// The grain at `p`, whose distance from the market is `market_dist`
    /// (`Site::market_dist`, the stretch frame's where the town is
    /// stretched). Strength is the blend's coherence (the doubled-angle sum's
    /// length over its weight) times the total weight capped at 1, times a
    /// ramp from 0 at the core's edge to 1 at `core_m + ramp_m`. No sources
    /// (or none with weight) is no grain: strength 0, axis 0.
    pub fn at(&self, p: Vec2, market_dist: f64) -> Grain {
        let (mut c, mut s, mut wsum) = (0.0f64, 0.0f64, 0.0f64);
        for src in &self.sources {
            let mut best = (f64::INFINITY, 0usize);
            for (i, w) in src.line.windows(2).enumerate() {
                let d = dist_pt_seg(p, w[0], w[1]);
                if d < best.0 {
                    best = (d, i);
                }
            }
            if !best.0.is_finite() {
                continue;
            }
            let (a, b) = (src.line[best.1], src.line[best.1 + 1]);
            let th = (b.y - a.y).atan2(b.x - a.x);
            let w = src.weight / (1.0 + (best.0 / src.reach_m).powi(2));
            c += w * (2.0 * th).cos();
            s += w * (2.0 * th).sin();
            wsum += w;
        }
        if !(wsum > 0.0) {
            return Grain { axis: 0.0, strength: 0.0 };
        }
        let coherence = (c * c + s * s).sqrt() / wsum;
        let ramp = if self.ramp_m > 0.0 { ((market_dist - self.core_m) / self.ramp_m).clamp(0.0, 1.0) } else { f64::from(u8::from(market_dist >= self.core_m)) };
        Grain { axis: 0.5 * s.atan2(c), strength: (coherence * wsum.min(1.0) * ramp).clamp(0.0, 1.0) }
    }
}

/// The field's sources for a town of type `ty` (scope section 4): the river,
/// the spine or one road for a town under [`MULTI_GRAIN_POP`] (one grain);
/// that plus every approach-road edge for a larger town (up to its number of
/// approach roads). `None` for [`TownType::NoGrain`].
///
/// - `spine`: [`through_pair`]'s bearings (a market-street town);
/// - `bearings`: [`approach_bearings`] (a small organic-core town's one grain
///   is the approach road nearest the town's mean road orientation, below);
/// - `max_rf`: the spine and the one grain run this far each way.
pub fn sources_for(ty: TownType, pop: f64, site: &Site, g: &Graph, market: Vec2, spine: Option<(f64, f64)>, bearings: &[f64], max_rf: f64) -> Option<Vec<Source>> {
    let ray = |th: f64| market + Vec2::new(th.cos(), th.sin()) * max_rf;
    let roads = || -> Vec<Source> {
        g.edges
            .iter()
            .filter(|e| e.alive && e.cls == "primary")
            .map(|e| Source { kind: SourceKind::Road, line: vec![g.nodes[e.a].pt(), g.nodes[e.b].pt()], weight: ROAD_WEIGHT, reach_m: ROAD_REACH_M })
            .collect()
    };
    let mut out = match ty {
        TownType::NoGrain => return None,
        TownType::RiverFront => vec![Source { kind: SourceKind::River, line: site.river.clone(), weight: RIVER_WEIGHT, reach_m: RIVER_REACH_M }],
        TownType::MarketStreet => {
            let (a, b) = spine?;
            vec![Source { kind: SourceKind::Spine, line: vec![ray(a), market, ray(b)], weight: SPINE_WEIGHT, reach_m: SPINE_REACH_M }]
        }
        TownType::OrganicCore => {
            if pop >= MULTI_GRAIN_POP {
                Vec::new()
            } else {
                // One grain: the town's own mean road orientation (the
                // doubled-angle mean of its primaries within `max_rf`, by
                // length), laid as a straight line through the market along
                // the approach bearing nearest it. **A labelled judgement**
                // for "one grain" where no road runs through and no river
                // sets one.
                let (mut c, mut s) = (0.0, 0.0);
                for e in g.edges.iter().filter(|e| e.alive && e.cls == "primary") {
                    let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
                    if dist_pt_seg(market, a, b) > max_rf {
                        continue;
                    }
                    let th = (b.y - a.y).atan2(b.x - a.x);
                    let l = a.dist(b);
                    c += l * (2.0 * th).cos();
                    s += l * (2.0 * th).sin();
                }
                let mean = 0.5 * s.atan2(c);
                let th = bearings.iter().copied().min_by(|x, y| axis_diff(*x, mean).total_cmp(&axis_diff(*y, mean)))?;
                vec![Source { kind: SourceKind::Road, line: vec![market, ray(th)], weight: SPINE_WEIGHT, reach_m: SPINE_REACH_M }]
            }
        }
    };
    if pop >= MULTI_GRAIN_POP {
        out.extend(roads());
    }
    (!out.is_empty()).then_some(out)
}

/// A block outline with near-duplicate vertices (closer than 6 m) merged and
/// near-straight vertices (turn under `turn_deg`) dropped: the corners a
/// reader of the plan would count. PS-0's `ps_corners`
/// (`parallel-streets-ps0-2026-11-17/scratch_code/prototype_and_audit.diff.txt`),
/// kept verbatim so PS-1's block shapes are PS-0's instrument (moved here
/// from the census, 2026-11-21, so PS-2's `lane_pass_with` and the census
/// count one definition); the 6 m and
/// 20-degree thresholds are PS-0's labelled judgements.
pub fn block_outline_corners(poly: &[Vec2], turn_deg: f64) -> Vec<(f64, f64)> {
    let mut p: Vec<(f64, f64)> = Vec::new();
    for v in poly {
        if p.last().is_none_or(|l: &(f64, f64)| (l.0 - v.x).hypot(l.1 - v.y) >= 6.0) {
            p.push((v.x, v.y));
        }
    }
    while p.len() > 2 && {
        let (a, b) = (p[0], p[p.len() - 1]);
        (a.0 - b.0).hypot(a.1 - b.1) < 6.0
    } {
        p.pop();
    }
    loop {
        let n = p.len();
        if n <= 3 {
            return p;
        }
        let mut drop = 0usize;
        let mut least = f64::INFINITY;
        for i in 0..n {
            let (a, b, c) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n]);
            let t1 = (b.1 - a.1).atan2(b.0 - a.0);
            let t2 = (c.1 - b.1).atan2(c.0 - b.0);
            let mut d = (t2 - t1).abs() % (2.0 * std::f64::consts::PI);
            if d > std::f64::consts::PI {
                d = 2.0 * std::f64::consts::PI - d;
            }
            if d < least {
                least = d;
                drop = i;
            }
        }
        if least.to_degrees() < turn_deg {
            p.remove(drop);
        } else {
            return p;
        }
    }
}

/// Minimum-area bounding rectangle over the outline's edge directions:
/// `(long side, short side)`; the short side is a block's depth across.
/// PS-0's `ps_mabr`, verbatim (moved here with [`block_outline_corners`]).
pub fn block_mabr(poly: &[Vec2]) -> (f64, f64) {
    let mut best = (f64::INFINITY, 0.0, 0.0);
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let th = (b.y - a.y).atan2(b.x - a.x);
        let (c, s) = (th.cos(), th.sin());
        let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for v in poly {
            let (x, y) = (v.x * c + v.y * s, -v.x * s + v.y * c);
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        let (w, h) = (x1 - x0, y1 - y0);
        if w * h < best.0 {
            best = (w * h, w.max(h), w.min(h));
        }
    }
    (best.1, best.2)
}

/// The corner count PS-2's closed-block test reads: [`block_outline_corners`]
/// at PS-0's 20 degrees (the census's own setting).
pub fn block_corners(poly: &[Vec2]) -> usize {
    block_outline_corners(poly, 20.0).len()
}

/// Area over the minimum-area bounding rectangle's ([`block_mabr`]), 0..1:
/// PS-0's rectangularity, the census's `rect`. 0 for a degenerate outline.
pub fn rectangularity(poly: &[Vec2]) -> f64 {
    let (l, s) = block_mabr(poly);
    let a = crate::geom::poly_area(poly).abs();
    if l * s > 0.0 { a / (l * s) } else { 0.0 }
}

/// How far bearing `a` is from the nearest of the grain `axis`'s four
/// directions (the axis and its cross-direction, either way), 0..pi/4. PS-2
/// (`crate::growth`) reads it: a street "runs with the grain" when this is
/// small, whichever of the two directions it follows.
pub fn axis_off(a: f64, axis: f64) -> f64 {
    let d = (a - axis).rem_euclid(std::f64::consts::FRAC_PI_2);
    d.min(std::f64::consts::FRAC_PI_2 - d)
}

/// The unsigned angle between two axes (bearings modulo pi), 0..pi/2.
fn axis_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(std::f64::consts::PI);
    d.min(std::f64::consts::PI - d)
}

/// Section 9.3's ground gate for one town, measured on its [`Site`] before it
/// grows: the dry, gentle ground inside the box's margin and the growth front,
/// on the market's bank, plus the far bank inside the bridge strip
/// `crate::growth::grow_far_bank` serves.
#[derive(Debug, Clone, PartialEq)]
pub struct Ground {
    /// The market, and its distance rule's reach (`max_rf` as handed to
    /// `grow`, after Ruling GB's widening).
    pub market: Vec2,
    pub max_rf: f64,
    /// The market's bank side (`Site::bank_side`), `None` on a site without a
    /// river centreline (then every cell is "the market's bank").
    pub mkt_side: Option<f64>,
    /// The far end of the town bridge `crate::riverbank::lay_town_bridge`
    /// would lay (`crate::riverbank::far_bridge_end`), where the far bank
    /// counts at all: a both-banks town whose bridge end is buildable.
    pub far_end: Option<Vec2>,
    /// Growable area on the market's bank, m2.
    pub bank_m2: f64,
    /// Growable area on the far bank inside the bridge strip, m2, before the
    /// cap.
    pub far_raw_m2: f64,
    /// That, capped at `crate::riverbank::FAR_BANK_MAX_RATIO` times
    /// [`Self::bank_m2`]: `grow_far_bank` stops when the far bank's streets
    /// reach that multiple of the market bank's, so ground past it cannot be
    /// grown into. **A labelled judgement**: the street-length cap read as an
    /// area cap.
    pub far_m2: f64,
}

impl Ground {
    /// Is `p` in the gate's region (the region the settled area is counted in
    /// too): within the growth front (`Site::market_dist` for the market's
    /// bank; the stretch frame's distance, or the distance to [`Self::far_end`],
    /// for the far bank, as `grow_far_bank` measures it) and on the market's
    /// bank or the far bank's strip. No water or slope test: a settled cell is
    /// settled.
    pub fn in_region(&self, site: &Site, p: Vec2) -> bool {
        match self.mkt_side {
            None => site.market_dist(p, self.market) <= self.max_rf,
            Some(m) if site.bank_side(p) == m => site.market_dist(p, self.market) <= self.max_rf,
            Some(_) => self.far_end.is_some_and(|fe| far_dist(site, p, fe) <= self.max_rf),
        }
    }

    /// The growable area the gate counts: the market's bank plus the capped
    /// far bank, m2.
    pub fn growable_m2(&self) -> f64 {
        self.bank_m2 + self.far_m2
    }
}

/// `grow_far_bank`'s distance on the far bank: the stretch frame's where the
/// town is stretched, else from the bridge's far end.
fn far_dist(site: &Site, p: Vec2, far_end: Vec2) -> f64 {
    match site.stretch() {
        Some(f) => f.dist(p),
        None => p.dist(far_end),
    }
}

/// Section 9.3's growable-cell count over `site` for a town at `market` with
/// reach `max_rf` (the value handed to `grow`), `both_banks` its both-banks
/// eligibility (`generate`'s own flag). Cells every [`GROUND_CELL_M`] from
/// `GROUND_MARGIN_M + 5` (PS-0 item 2's lattice); a cell counts if it is
/// inside the margin, dry (`!Site::is_water`), no steeper than
/// [`GROUND_MAX_SLOPE`], and in [`Ground::in_region`]; a far-bank cell must
/// also be `crate::riverbank::far_bank_buildable`, the test `grow_far_bank`
/// puts on every far-bank origin. Pure: reads `site`, writes nothing.
pub fn ground_gate(site: &Site, market: Vec2, max_rf: f64, both_banks: bool) -> Ground {
    let mkt_side = (site.river_like() && site.river.len() >= 2).then(|| site.bank_side(market));
    let far_end = if both_banks && mkt_side.is_some() {
        crate::riverbank::far_bridge_end(site, market).filter(|&fe| crate::riverbank::far_bank_buildable(site, fe))
    } else {
        None
    };
    let mut g = Ground { market, max_rf, mkt_side, far_end, bank_m2: 0.0, far_raw_m2: 0.0, far_m2: 0.0 };
    let cell = GROUND_CELL_M * GROUND_CELL_M;
    let mut y = GROUND_MARGIN_M + 5.0;
    while y < site.hm - GROUND_MARGIN_M {
        let mut x = GROUND_MARGIN_M + 5.0;
        while x < site.wm - GROUND_MARGIN_M {
            let p = Vec2::new(x, y);
            if g.in_region(site, p) && !site.is_water(p) && site.slope(p) <= GROUND_MAX_SLOPE {
                if mkt_side.is_none_or(|m| site.bank_side(p) == m) {
                    g.bank_m2 += cell;
                } else if crate::riverbank::far_bank_buildable(site, p) {
                    g.far_raw_m2 += cell;
                }
            }
            x += GROUND_CELL_M;
        }
        y += GROUND_CELL_M;
    }
    g.far_m2 = g.far_raw_m2.min(crate::riverbank::FAR_BANK_MAX_RATIO * g.bank_m2);
    g
}

/// The settled-area cell, metres: the census's own (`SETTLED_CELL_M` in
/// `town_river_census.rs`, FS-1's measure), so the ground ratio's denominator
/// is the figure the census prints.
pub const SETTLED_CELL_M: f64 = 20.0;

/// Today's settled area inside the gate's region, m2: the census's FS-1
/// footprint (every [`SETTLED_CELL_M`] cell within one cell of a building's
/// vertex-mean centre), counted where the cell's centre is in
/// [`Ground::in_region`].
pub fn settled_in_region(ground: &Ground, site: &Site, buildings: &[&[Vec2]]) -> f64 {
    let mut cells = std::collections::BTreeSet::new();
    for b in buildings.iter().filter(|b| !b.is_empty()) {
        let n = b.len() as f64;
        let (cx, cy) = (b.iter().map(|p| p.x).sum::<f64>() / n, b.iter().map(|p| p.y).sum::<f64>() / n);
        let (i, j) = ((cx / SETTLED_CELL_M).floor() as i64, (cy / SETTLED_CELL_M).floor() as i64);
        for di in -1..=1 {
            for dj in -1..=1 {
                cells.insert((i + di, j + dj));
            }
        }
    }
    let inside = cells
        .iter()
        .filter(|&&(i, j)| ground.in_region(site, Vec2::new((i as f64 + 0.5) * SETTLED_CELL_M, (j as f64 + 0.5) * SETTLED_CELL_M)))
        .count();
    inside as f64 * SETTLED_CELL_M * SETTLED_CELL_M
}

/// Fill `plan`'s two settled areas from the finished layout's building
/// outlines ([`settled_in_region`]; the market's bank alone with the far end
/// left out).
pub fn fill_settled(plan: &mut ParallelPlan, site: &Site, buildings: &[&[Vec2]]) {
    plan.settled_m2 = settled_in_region(&plan.ground, site, buildings);
    let bank_only = Ground { far_end: None, ..plan.ground.clone() };
    plan.settled_bank_m2 = settled_in_region(&bank_only, site, buildings);
}

/// What PS-1 computes for one town of [`MIN_POP`] and up with the switch on,
/// reported on `Town::parallel` and changing nothing else.
#[derive(Debug, Clone, PartialEq)]
pub struct ParallelPlan {
    pub town_type: TownType,
    /// The approach roads read (their bearings), for a reader and the census.
    pub approaches: Vec<f64>,
    /// `None` for [`TownType::NoGrain`].
    pub field: Option<DirectionField>,
    pub ground: Ground,
    /// Today's settled area in the gate's region ([`settled_in_region`]),
    /// m2, from this layout's own buildings: PS-1 changes no layout, so this
    /// layout is today's.
    pub settled_m2: f64,
    /// The same, on the market's bank alone (the far-bank strip left out), m2:
    /// PS-0 item 2's like-with-like denominator, kept as the recount's
    /// control.
    pub settled_bank_m2: f64,
    /// The field sampled every [`FIELD_SAMPLE_M`] inside the growth front
    /// (points with strength over zero), for the City Viewer's overlay
    /// (`urban_bridge`'s `parallel_field`). Empty without a field.
    pub samples: Vec<(Vec2, Grain)>,
}

/// The overlay's sample spacing, metres. **A labelled judgement**: a tick
/// every 40 m reads as a grain at the viewer's opening frame (a 1 700 m box
/// in about 1 300 px) without hiding the streets.
pub const FIELD_SAMPLE_M: f64 = 40.0;

impl ParallelPlan {
    /// Section 9.3's ground ratio `g = growable / settled`, `None` for a town
    /// with nothing settled in the region (no ratio, not a plausible one).
    pub fn ground_ratio(&self) -> Option<f64> {
        (self.settled_m2 > 0.0).then(|| self.ground.growable_m2() / self.settled_m2)
    }

    /// Does the town fit section 9.3's rule at `f_req` ([`F_REQ_MEDIAN`] or
    /// [`F_REQ_P90`])? `g >= f_req`. A town with no ratio does not fit.
    pub fn fits(&self, f_req: f64) -> bool {
        self.ground_ratio().is_some_and(|g| g >= f_req)
    }

}

/// `field` sampled every `step` metres over the box inside the growth front
/// (`Site::market_dist` within `max_rf`), keeping the points with strength
/// over zero.
fn sample_field(field: &DirectionField, site: &Site, market: Vec2, max_rf: f64, step: f64) -> Vec<(Vec2, Grain)> {
    let mut out = Vec::new();
    let mut y = step / 2.0;
    while y < site.hm {
        let mut x = step / 2.0;
        while x < site.wm {
            let p = Vec2::new(x, y);
            let d = site.market_dist(p, market);
            if d <= max_rf {
                let gr = field.at(p, d);
                if gr.strength > 0.0 {
                    out.push((p, gr));
                }
            }
            x += step;
        }
        y += step;
    }
    out
}

/// The whole PS-1 pass for one town, before it grows: its type, its field and
/// its ground, with `settled_m2` and `settled_bank_m2` left at 0 for
/// `generate_pass` to fill from the finished layout ([`fill_settled`]). `organic` is false for the radial plan (no grain);
/// `plaza` the market square's outline if built. Reads `site` and `g`; writes
/// neither, and draws no random number.
#[allow(clippy::too_many_arguments)]
pub fn plan(site: &Site, g: &Graph, market: Vec2, plaza: Option<&[Vec2]>, pop: f64, organic: bool, max_rf: f64, age_years: f64, both_banks: bool) -> ParallelPlan {
    let approaches = if organic { approach_bearings(g, market, APPROACH_PROBE_M) } else { Vec::new() };
    let spine = through_pair(&approaches);
    let river_front = organic && is_river_front(site, market, plaza);
    let mut town_type = town_type(organic, river_front, spine.is_some(), approaches.len());
    let field = sources_for(town_type, pop, site, g, market, spine, &approaches, max_rf).map(|sources| DirectionField { sources, core_m: core_radius(max_rf, age_years), ramp_m: RAMP_FRAC * max_rf });
    if field.is_none() {
        town_type = TownType::NoGrain;
    }
    let samples = field.as_ref().map_or_else(Vec::new, |f| sample_field(f, site, market, max_rf, FIELD_SAMPLE_M));
    ParallelPlan { town_type, approaches, field, ground: ground_gate(site, market, max_rf, both_banks), settled_m2: 0.0, settled_bank_m2: 0.0, samples }
}

/// Scope section 9.2's access rule for a rear household (Ruling GF, Q13 (a)):
/// a rear dwelling needs a back lane (PS-3's back-street tier) within this
/// many metres, or a 2.5 m side passage. **The owner's** 30 m.
pub const REAR_ACCESS_M: f64 = 30.0;

/// Scope section 9.2: a rear household exists only on a lot more than this
/// deep, metres (Ruling GC's "about 30 m").
pub const REAR_LOT_DEPTH_M: f64 = 30.0;

/// PS-3's measure of section 9.2's access rule: the distance from the middle
/// of a four-cornered lot's rear edge to the nearest street centreline in
/// `streets`, metres. The lot is `[front a, front b, rear b, rear a]`, the
/// platting's order (`blocks::build_parcels_with`, read the same way by the
/// census). `None` for a lot that is not four-cornered, or with no street.
/// A lot is served from behind when this is at most [`REAR_ACCESS_M`]; for a
/// lot deeper than that, its own front street is never the nearest one
/// within reach, so a hit is a back lane or a side street. Reads only.
pub fn rear_access_m(lot: &[Vec2], streets: &[(Vec2, Vec2)]) -> Option<f64> {
    if lot.len() != 4 {
        return None;
    }
    let rear = lot[2].lerp(lot[3], 0.5);
    let d = streets.iter().map(|&(a, b)| dist_pt_seg(rear, a, b)).fold(f64::INFINITY, f64::min);
    d.is_finite().then_some(d)
}

/// Section 9.2's access rule on one lot, PS-3's measure of it: `None` unless
/// the lot is four-cornered and more than [`REAR_LOT_DEPTH_M`] deep (its two
/// sides' mean, the census's depth), else whether a street lies within
/// [`REAR_ACCESS_M`] of its rear edge ([`rear_access_m`]). The census counts
/// the `Some(true)` share; PS-5's rear households will read the same answer.
pub fn rear_served(lot: &[Vec2], streets: &[(Vec2, Vec2)]) -> Option<bool> {
    if lot.len() != 4 || (lot[0].dist(lot[3]) + lot[1].dist(lot[2])) / 2.0 <= REAR_LOT_DEPTH_M {
        return None;
    }
    Some(rear_access_m(lot, streets).is_some_and(|d| d <= REAR_ACCESS_M))
}
