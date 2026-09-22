//! Lots built against the town wall — **not a reference port.** A deliberate
//! departure under Ruling H (`LARGE_ITEM_RULINGS.md`, 2026-09-12: *"change the
//! ported urban algorithm toward the owner's town plan — golden re-baseline
//! authorised"*), requested by the owner on 2026-09-22: *"it's not uncommon for
//! a settlement to be built unto the wall from the inside and have a poor
//! district built up against the wall from the outside."*
//!
//! The reference (v2.11) has no such concept under any name, and neither did
//! this port before this module: `in_wall` in [`crate::districts`] is a
//! containment test, never a distance to the wall's surface.
//!
//! # Why a new stage rather than a bias on the existing blocks
//!
//! There is no "last ring of blocks against the wall" to bias. Blocks are the
//! inner faces of the **street** graph ([`crate::blocks::build_blocks`]) and
//! parcels are platted inward from street frontages only; the wall is not a
//! graph edge. [`crate::fortify::built_mass_hull`] puts the curtain at the
//! built-node hull inflated by 10% and then by 16 m, so the band between the
//! outermost streets and the wall is part of the graph's *outer* face, which
//! `build_blocks` skips. Measured 2026-09-22 on five towns before this change
//! (seed/pop/site 7/1 500/inland, 42/5 000/inland, 1234/12 000/inland,
//! 42/5 000/river, 99/9 000/coast), probing every 2 m of the land arc 1 m past
//! the drawn wall face: **0.2-1.6%** of the wall's length had a building
//! against its inner face and **0.0%** against its outer face, and the median
//! gap from the wall to the nearest intramural building was 38-81 m. Outside,
//! the curtain's 15 m rampart strip ([`crate::cleanup::clear_fort_zone`]) also
//! swept whatever reached it. After it, on the same five: 12-33% inside and
//! 6-30% outside, with every faubourg building touching the wall face — the
//! single-row version of 2026-09-22's first pass; the cluster that replaced it
//! keeps that first row and builds behind it.
//!
//! **Where the band stays open, that is measured too, not a gap in the pass.**
//! Most refused intramural stretches have no street within [`INNER_REACH`] —
//! 43, 83 and 145 of them on the first, second and fifth towns — i.e. growth
//! reserve the town never filled, which a lot deeper than `build_parcels`'
//! own 46 m ceiling would misrepresent.
//!
//! So both patterns are **plats of that band**, generated from the wall:
//!
//! 1. **Intramural** ([`WallBacking::Inside`]): from each stretch of the land
//!    arc, a lot runs inward from the wall's inner face to the first street it
//!    meets, fronting that street and using the curtain as its back line. No
//!    street within [`INNER_REACH`] — or one closer than [`MIN_DEPTH`] — and
//!    the band stays open there, as an intervallum did.
//! 2. **Extramural** ([`WallBacking::Outside`], [`WallBacking::OutsideRow`]): a
//!    *faubourg* — a cluster of narrow, shallow lots against the wall's outer
//!    face, district `"faubourg"`. It starts beside a land gate, as the
//!    historical ones did (the faubourg is named for the gate road it grew
//!    off), but it runs **along the wall**, into the gap between the roads —
//!    which is exactly what the road-anchored ribbon suburb in `grow` never
//!    does, since there every extramural origin must lie within 90 m of a
//!    primary. The two are additive: a faubourg lot that would overlap a ribbon
//!    parcel, or cross any street, is simply not platted.
//!
//!    **A cluster, not a line** (owner, 2026-09-22 follow-up: *"the shanty
//!    shouldn't just be a small line against the wall it should be a cluster
//!    against the wall, much like the reference image"*). The first row backs
//!    onto the wall face and the cluster is [`FAUBOURG_ROWS`] rows deep at its
//!    gate, counting that one; each row behind it is offset
//!    outward by the row in front plus a [`LANE`] (or none, [`BACK_TO_BACK`]),
//!    staggered along the wall, broken by cross-[`ALLEY`]s, and each reaching
//!    less far along the wall than the one in front — so the cluster is a wedge,
//!    deepest at its gate and thinning to a single row. **Parallel rows rather
//!    than a block subdivision**, because the reference's quarters read as rows
//!    following the curtain, and because a subdivision would need streets this
//!    ground does not have: blocks here are faces of the street graph
//!    ([`crate::blocks::build_blocks`]) and the faubourg's lanes are not graph
//!    edges — they are the gaps the rows leave, as the wall is the gap the
//!    intramural lots leave. This stage adds no street. The measurements behind
//!    each constant are on the constant.
//!
//! # What reads as "poor" here, and why no new visual language was invented
//!
//! This codebase has no wealth or status field. What it does have is a grammar
//! whose richness is keyed on district: courtyards only on market and burgher
//! plots, deeper main ranges there (`logn(11.5)` against `logn(9.5)`), and wings
//! and outbuildings with age. A faubourg lot gets the bottom of that scale on
//! every axis — frontage `logn(6.5)` against the town's `logn(11)`, depth
//! 6-12 m, one small range at the lot's wallward end and nothing else (a
//! lean-to on the curtain in the first row) — plus its own
//! parcel fill in `urban_layout_draw.gd`'s `DISTRICT_FILL`, derived from the
//! suburb fill rather than picked.
//!
//! # Scope, deliberately
//!
//! - **Curtain and palisade only.** A `ditch` has no masonry to build against,
//!   and a `bastioned` trace keeps its glacis clear — that clearance *is* the
//!   design (a field of fire), and [`crate::cleanup::clear_fort_zone`] still
//!   sweeps it.
//! - **Organic planning only.** The Venus profile's radial plan and building
//!   grammar are a designed city, not an accreted one.
//! - **Unconditional**, not a rule toggle: Ruling H's other changes are direct
//!   algorithm changes, and nothing here needs tuning a user would reach for.
//!
//! # Draws
//!
//! Four substreams of this module's own — `"wallside/in"`, `"faubourg"` (each
//! cluster's first row, against the wall), `"faubourg/rows"` (the rows behind
//! it, added by the 2026-09-22 follow-up in a separate stream so the first row
//! is exactly the single-row pass that preceded it) and `"wallside/tone"` (the
//! roof tone, kept apart for the reason [`Parcel::tone`] gives) — so no draw
//! any existing stage makes is shifted. Lots are appended after every
//! street-platted parcel, and every later consumer of the parcel list either
//! draws from a per-parcel stream (`build_buildings`) or walks the list in
//! order — so what these lots can change downstream is bounded to what reaches
//! them, and the goldens that moved are listed in `generate/tests/golden.rs`.

use crate::blocks::{Block, Parcel};
use crate::geom::{Vec2, point_in_poly, poly_area, poly_centroid, poly_self_intersects, seg_int};
use crate::graph::Graph;
use crate::growth::WallState;
use crate::rng::{Substream, stream};
use crate::site::Site;

/// Which face of the wall a lot backs onto. [`WallBacking::No`] is every lot
/// `build_parcels` platted off a street.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WallBacking {
    #[default]
    No,
    /// Inside the circuit, its back line on the wall's inner face.
    Inside,
    /// Outside the circuit, its back line on the wall's outer face: a
    /// faubourg lot in the cluster's first row.
    Outside,
    /// Outside the circuit, a faubourg lot in a row **behind** the first: its
    /// back line faces the wall across a lane or the row in front, and touches
    /// no wall. Kept apart from [`WallBacking::Outside`] so nothing it builds
    /// claims the curtain as its back wall.
    OutsideRow,
}

impl WallBacking {
    /// Any faubourg lot, first row or behind it — the district, the rampart
    /// sweep's exemption and the building grammar all key on this, not on
    /// [`WallBacking::Outside`] alone.
    pub fn is_faubourg(self) -> bool {
        matches!(self, WallBacking::Outside | WallBacking::OutsideRow)
    }
}

/// How far inward a lot may reach from the wall to find its street. Matches
/// `build_parcels`' own deepest plot (`depth_target` caps at 46 m) with the
/// street's half-width and verge on top.
pub const INNER_REACH: f64 = 52.0;
/// A lot shallower than this has no room for a house between wall and street.
pub const MIN_DEPTH: f64 = 6.0;
/// No lot within this distance of a gate: the gate passage and its causeway.
pub const GATE_CLEAR: f64 = 14.0;
/// The most the wall may bow away from a lot's straight back line. A lot is a
/// quad and the wall is a curve; across a sharp bend the chord cuts the corner,
/// and without this a back line was measured **10.5 m** off the wall inside and
/// 1.5 m under the drawn stroke outside, against a median of exactly the face.
pub const MAX_BOW: f64 = 1.0;

/// Rows in a faubourg cluster, at its gate end: 3-5, inclusive. Read off the
/// owner's reference plan (`design/owner-references-2026-09-12/
/// urban-town-plan-walled-market-town.jpg`, 900 x 1350 px): measured in that
/// plan's own lots (~20-25 px across, i.e. the ~11 m frontage of an ordinary
/// lot), its extramural quarters stand **2 lots deep** off the west curtain
/// (~55 px) and **5-7 deep** on the east (~100-180 px), against a wall ~470 px
/// across. At this module's 6-12 m faubourg depth plus a lane, 3-5 rows is
/// ~35-60 m — the reference's middle, not its extremes.
pub const FAUBOURG_ROWS: (i64, i64) = (3, 5);
/// A lane between two rows, in metres: the reference draws its lanes at about
/// a fifth of a lot's width (3-4 px against ~20 px), ~2-3.5 m at this scale.
pub const LANE: (f64, f64) = (2.0, 3.5);
/// The share of row pairs with no lane between them — built back to back, as
/// the reference's denser blocks are. Neither all lanes (a set of detached
/// lines) nor none (one undifferentiated mass).
pub const BACK_TO_BACK: f64 = 0.35;
/// Per lot, in a row behind the first, the chance of an alley — a gap of
/// [`LANE`] width — opening before it: the reference's lanes run across the
/// rows as well as along them.
pub const ALLEY: f64 = 0.12;
/// The shallowest lot a row behind the first is drawn at. Not [`MIN_DEPTH`]
/// itself: a lot is re-measured from its corners (`Parcel::depth`) and
/// `build_buildings` leaves anything under 6 m empty, so a lot drawn at exactly
/// 6.0 measured **5.99999999999992-5.99999999999999** and stood unbuilt — 26
/// of them across the goldens in a first draft. Half a metre clears the
/// round-trip by thirteen orders of magnitude and is below the lot-to-lot
/// spread of `logn(9.0, 0.2)`.
///
/// The first row is untouched and still clamps at `MIN_DEPTH`, so the same
/// defect stands there on the lots it reaches: three on the goldens
/// (`coastHarbourChain` `faub29`/`faub62`, `faithNoneBasilica` `faub19`),
/// pre-existing since the single-row pass; fixing it moves that row.
pub const ROW_DEPTH_FLOOR: f64 = MIN_DEPTH + 0.5;

/// Half the drawn stroke of each wall style that can be built against —
/// `urban_layout_draw.gd`'s `WALL_W` (`curtain` 4.5, `palisade` 2.2), halved,
/// so a lot's back line lands on the face the renderer draws. `None` for a
/// style nothing is built against (see the module header).
pub fn face_offset(style: &str) -> Option<f64> {
    match style {
        "curtain" => Some(2.25),
        "palisade" => Some(1.1),
        _ => None,
    }
}

/// The land arc as segments, with cumulative lengths; closed when it is the
/// whole ring (an all-land curtain), open when it stops at the water.
struct Arc {
    pts: Vec<Vec2>,
    cum: Vec<f64>,
    closed: bool,
}

impl Arc {
    fn new(arc: &[Vec2], closed: bool) -> Self {
        let mut pts = arc.to_vec();
        if closed {
            pts.push(arc[0]);
        }
        let mut cum = vec![0.0];
        for w in pts.windows(2) {
            cum.push(cum[cum.len() - 1] + w[0].dist(w[1]));
        }
        Arc { pts, cum, closed }
    }
    fn len(&self) -> f64 {
        self.cum[self.cum.len() - 1]
    }
    /// The point at arc length `s`, wrapped on a closed arc and clamped on an
    /// open one.
    fn at(&self, s: f64) -> Vec2 {
        let l = self.len();
        let s = if self.closed { s.rem_euclid(l) } else { s.clamp(0.0, l) };
        let i = self.cum.partition_point(|&c| c <= s).clamp(1, self.pts.len() - 1);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        let seg = self.cum[i] - self.cum[i - 1];
        if seg <= 0.0 { a } else { a.lerp(b, (s - self.cum[i - 1]) / seg) }
    }
    /// How far the arc strays from the straight chord between arc lengths `s0`
    /// and `s1`: its vertices in between, measured to the chord.
    fn bow(&self, s0: f64, s1: f64) -> f64 {
        let (a, b) = (self.at(s0), self.at(s1));
        let l = self.len();
        let mut worst = 0.0f64;
        for (i, &c) in self.cum.iter().enumerate() {
            // A closed arc is walked across its seam too, so a vertex is
            // tested at its own arc length and one lap either side.
            for off in [-l, 0.0, l] {
                let c = c + off;
                if c > s0 && c < s1 {
                    worst = worst.max(crate::geom::dist_pt_seg(self.pts[i], a, b));
                }
            }
        }
        worst
    }
    /// Arc length of the point nearest `p`.
    fn project(&self, p: Vec2) -> f64 {
        let mut best = (f64::INFINITY, 0.0);
        for i in 1..self.pts.len() {
            let (a, b) = (self.pts[i - 1], self.pts[i]);
            let ab = b - a;
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let t = if l2 > 0.0 { (((p - a).x * ab.x + (p - a).y * ab.y) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let d = p.dist(a.lerp(b, t));
            if d < best.0 {
                best = (d, self.cum[i - 1] + t * (self.cum[i] - self.cum[i - 1]));
            }
        }
        best.1
    }
}

/// Everything a candidate lot is tested against.
struct Ctx<'a> {
    g: &'a Graph,
    site: &'a Site,
    ring: &'a [Vec2],
    wall: &'a WallState,
    plazas: Vec<&'a [Vec2]>,
    /// Every accepted polygon so far — the street-platted parcels, then the
    /// wall lots as they are accepted — with its bounding box.
    taken: Vec<(Vec<Vec2>, [f64; 4])>,
}

fn bbox(p: &[Vec2]) -> [f64; 4] {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for q in p {
        b = [b[0].min(q.x), b[1].min(q.y), b[2].max(q.x), b[3].max(q.y)];
    }
    b
}

impl Ctx<'_> {
    /// Distance along `dir` from `o` to the first live street within `reach`,
    /// less that street's half-width and `build_blocks`' 1.4 m verge — i.e.
    /// the depth to the street's building line.
    /// Returns that depth and the street's class.
    fn street_face(&self, o: Vec2, dir: Vec2, reach: f64) -> Option<(f64, &'static str)> {
        let far = o + dir * reach;
        let mut best: Option<(f64, f64, &'static str)> = None;
        for eid in self.g.edges_near(o, far) {
            let e = &self.g.edges[eid];
            if !e.alive {
                continue;
            }
            if let Some(h) = seg_int(o, far, self.g.nodes[e.a].pt(), self.g.nodes[e.b].pt())
                && best.is_none_or(|(t, _, _)| h.t < t)
            {
                best = Some((h.t, e.w, e.cls));
            }
        }
        best.map(|(t, w, cls)| (t * reach - w / 2.0 - 1.4, cls))
    }

    /// Does any live street cross the segment `a`-`b`?
    fn crosses_street(&self, a: Vec2, b: Vec2) -> bool {
        self.g.edges_near(a, b).into_iter().any(|eid| {
            let e = &self.g.edges[eid];
            e.alive && seg_int(a, b, self.g.nodes[e.a].pt(), self.g.nodes[e.b].pt()).is_some()
        })
    }

    /// Every test a lot must pass besides its depth: inside the box, dry by
    /// `build_parcels`' own footprint rule, on the right side of the ring, clear
    /// of the gates, the plaza, every street and every lot already platted.
    fn accepts(&self, quad: &[Vec2], inside: bool) -> bool {
        let area = poly_area(quad).abs();
        if !(26.0..=2600.0).contains(&area) || poly_self_intersects(quad) {
            return false;
        }
        let c = poly_centroid(quad);
        let s = self.site;
        if quad.iter().any(|q| q.x < 0.0 || q.y < 0.0 || q.x > s.wm || q.y > s.hm) {
            return false;
        }
        let margin = if s.kind == "river" { s.river_w / 2.0 + 1.0 } else { 3.0 };
        if quad.iter().chain(std::iter::once(&c)).any(|q| s.is_water(*q) || s.river_dist(*q) < margin) {
            return false;
        }
        // Every corner, not the centroid: at the end of an open land arc the
        // ring turns away along the water, and a lot tested on its centroid
        // alone was measured straddling it (`unnavigableStem`, whose fort
        // sweep then cleared it).
        if quad.iter().any(|q| point_in_poly(*q, self.ring) != inside) {
            return false;
        }
        let back = (quad[3], quad[2]);
        if self.wall.gates.iter().any(|gt| crate::geom::dist_pt_seg(gt.pt, back.0, back.1) < GATE_CLEAR) {
            return false;
        }
        if self.plazas.iter().any(|p| point_in_poly(c, p)) {
            return false;
        }
        for i in 0..4 {
            if self.crosses_street(quad[i], quad[(i + 1) % 4]) {
                return false;
            }
        }
        let bb = bbox(quad);
        for (p, pb) in &self.taken {
            if pb[0] > bb[2] || pb[2] < bb[0] || pb[1] > bb[3] || pb[3] < bb[1] {
                continue;
            }
            if quad.iter().chain(std::iter::once(&c)).any(|q| point_in_poly(*q, p))
                || point_in_poly(poly_centroid(p), quad)
            {
                return false;
            }
        }
        true
    }
}

/// The outward unit normal of the chord `a`-`b`, oriented by the ring.
fn outward(ring: &[Vec2], a: Vec2, b: Vec2) -> Vec2 {
    let d = (b - a).norm();
    let n = Vec2::new(-d.y, d.x);
    if point_in_poly(a.lerp(b, 0.5) + n * 3.0, ring) { n * -1.0 } else { n }
}

/// The gap behind a faubourg row: none ([`BACK_TO_BACK`]) or a [`LANE`].
fn lane(r: &mut Substream) -> f64 {
    if r.chance(BACK_TO_BACK) { 0.0 } else { r.range(LANE.0, LANE.1) }
}

/// One faubourg lot between arc lengths `s0` and `s1` (either order), its back
/// line `off` metres out from the wall's drawn face and `depth` deep — cut back
/// to a street's building line in front — or `None` where it does not fit.
/// Returns the quad and the class of the street it fronts (`""`: none).
#[allow(clippy::too_many_arguments)]
fn faubourg_lot(
    ctx: &Ctx<'_>,
    arc: &Arc,
    ring: &[Vec2],
    hw: f64,
    s0: f64,
    s1: f64,
    off: f64,
    depth: f64,
) -> Option<(Vec<Vec2>, &'static str)> {
    // Walk order can run either way, but the quad is always wound the same way
    // round: `a` then `b` in increasing arc length.
    let (a, b) = (arc.at(s0.min(s1)), arc.at(s0.max(s1)));
    if a.dist(b) < 4.0 || arc.bow(s0.min(s1), s0.max(s1)) > MAX_BOW {
        return None;
    }
    let n_out = outward(ring, a, b);
    let (fa, fb) = (a + n_out * (hw + off), b + n_out * (hw + off));
    // A street in front cuts the lot back to its building line. `""` is
    // "fronts no street" -- open ground outside the wall. It is not a
    // plausible class: nothing compares against it.
    let (mut da, mut db, mut cls) = (depth, depth, "");
    if let Some((d, c)) = ctx.street_face(fa, n_out, depth + 8.0)
        && d < da
    {
        (da, cls) = (d, c);
    }
    if let Some((d, c)) = ctx.street_face(fb, n_out, depth + 8.0)
        && d < db
    {
        (db, cls) = (d, c);
    }
    if da < MIN_DEPTH || db < MIN_DEPTH {
        return None;
    }
    let quad = vec![fa + n_out * da, fb + n_out * db, fb, fa];
    ctx.accepts(&quad, false).then_some((quad, cls))
}

/// Plat the lots against the wall. Returns them in walk order, ids
/// `wallin{k}` / `faub{k}`, each its own one-lot "block" (see
/// [`Parcel::block`]) so `build_faith_sites` can never spread a churchyard
/// along a wall run.
///
/// `parcels` are the street-platted lots, which the new ones must not
/// overlap. `epochs` stamps `age` the way `build_parcels` does for a lot
/// whose street is unknown.
#[allow(clippy::too_many_arguments)]
pub fn build_wall_lots(
    seed: u32,
    g: &Graph,
    wall: &WallState,
    site: &Site,
    blocks: &[Block],
    parcels: &[Parcel],
    pop_target: f64,
    epochs: i32,
) -> Vec<Parcel> {
    let mut out = Vec::new();
    let (Some(ring), Some(land)) = (wall.ring.as_deref(), wall.land_arc.as_deref()) else {
        return out;
    };
    let Some(hw) = face_offset(&wall.style) else { return out };
    if land.len() < 2 {
        return out;
    }
    let arc = Arc::new(land, land == ring);
    let mut ctx = Ctx {
        g,
        site,
        ring,
        wall,
        plazas: blocks.iter().filter(|b| b.plaza).map(|b| b.face_poly.as_slice()).collect(),
        taken: parcels.iter().map(|p| (p.poly.clone(), bbox(&p.poly))).collect(),
    };
    let mut tone = stream(seed, "wallside/tone");
    let mut push = |ctx: &mut Ctx<'_>,
                    quad: Vec<Vec2>,
                    id: String,
                    backing: WallBacking,
                    edge_cls: &'static str,
                    tone: &mut Substream| {
        ctx.taken.push((quad.clone(), bbox(&quad)));
        out.push(Parcel {
            block: id.clone(),
            id,
            frontage: quad[0].dist(quad[1]),
            depth: (quad[0].dist(quad[3]) + quad[1].dist(quad[2])) / 2.0,
            area: poly_area(&quad).abs(),
            poly: quad,
            age: epochs as f64,
            edge_cls,
            tone: tone.u(),
            wall_backing: backing,
        });
    };

    // ---- 1. intramural: the whole land arc, lot by lot ----
    let mut r = stream(seed, "wallside/in");
    let total = arc.len();
    let mut s = 0.0;
    let mut k = 0usize;
    while s < total - 4.0 {
        let w = r.logn(9.5, 0.22).clamp(5.0, 15.0);
        let (a, b) = (arc.at(s), arc.at((s + w).min(total)));
        let s0 = s;
        s += w;
        if a.dist(b) < 4.0 || arc.bow(s0, s.min(total)) > MAX_BOW {
            continue;
        }
        let n_in = outward(ring, a, b) * -1.0;
        let (fa, fb) = (a + n_in * hw, b + n_in * hw);
        let (Some((da, cls)), Some((db, _))) =
            (ctx.street_face(fa, n_in, INNER_REACH), ctx.street_face(fb, n_in, INNER_REACH))
        else {
            continue;
        };
        // Two sides reaching their street more than 12 m apart in depth means
        // the rays found different streets, or one far across a junction: the
        // lot would be a wedge fronting neither.
        if da < MIN_DEPTH || db < MIN_DEPTH || (da - db).abs() > 12.0 {
            continue;
        }
        let quad = vec![fa + n_in * da, fb + n_in * db, fb, fa];
        if ctx.accepts(&quad, true) {
            push(&mut ctx, quad, format!("wallin{k}"), WallBacking::Inside, cls, &mut tone);
            k += 1;
        }
    }

    // ---- 2. extramural: faubourg clusters, each starting beside a land gate ----
    // 2a. Each cluster's FIRST row, against the wall face: exactly the
    // single-row pass this module shipped with (same stream, same draws), so
    // the cluster adds rows behind that line and never moves the line itself.
    let mut r = stream(seed, "faubourg");
    let runs = if pop_target < 4000.0 { 1 } else if pop_target < 10000.0 { 2 } else { 3 };
    let gates: Vec<f64> = wall.gates.iter().filter(|gt| !gt.water).map(|gt| arc.project(gt.pt)).collect();
    let mut k = 0usize;
    // Per run: its direction, start and length along the wall, and the
    // deepest lot drawn for its first row (the next row stands behind that).
    let mut clusters: Vec<(f64, f64, f64, f64)> = Vec::new();
    for _ in 0..runs {
        let dir = if r.chance(0.5) { 1.0 } else { -1.0 };
        let start = match r.pick(&gates) {
            Some(&sg) => sg + dir * GATE_CLEAR,
            None => r.range(0.0, total),
        };
        let run_len = total * r.range(0.10, 0.20);
        let mut deepest = MIN_DEPTH;
        let mut t = 0.0;
        while t < run_len {
            let w = r.logn(6.5, 0.18).clamp(4.5, 9.0);
            let depth = r.logn(9.0, 0.2).clamp(MIN_DEPTH, 12.0);
            deepest = deepest.max(depth);
            let (s0, s1) = (start + dir * t, start + dir * (t + w));
            t += w;
            if !arc.closed && (s0.min(s1) < 0.0 || s0.max(s1) > total) {
                break;
            }
            if let Some((quad, cls)) = faubourg_lot(&ctx, &arc, ring, hw, s0, s1, 0.0, depth) {
                push(&mut ctx, quad, format!("faub{k}"), WallBacking::Outside, cls, &mut tone);
                k += 1;
            }
        }
        clusters.push((dir, start, run_len, deepest));
    }

    // 2b. The rows behind, from a stream of their own so none of 2a's draws
    // shift. Each is offset outward by the row in front plus a lane (or none,
    // back to back), staggered, broken by alleys, and reaches less far along
    // the wall than the row in front: a wedge, deepest at its gate.
    let mut r = stream(seed, "faubourg/rows");
    for (dir, start, run_len, deepest) in clusters {
        let rows = r.int(FAUBOURG_ROWS.0, FAUBOURG_ROWS.1);
        let mut off = deepest + lane(&mut r);
        for j in 1..rows {
            let row_depth = r.logn(9.0, 0.2).clamp(MIN_DEPTH, 12.0);
            let row_len = run_len * (1.0 - j as f64 / rows as f64);
            // A stagger, so this row's plot lines do not continue the ones
            // of the row in front.
            let mut t = r.range(0.0, 6.0);
            while t < row_len {
                if r.chance(ALLEY) {
                    t += r.range(LANE.0, LANE.1);
                    continue;
                }
                let w = r.logn(6.5, 0.18).clamp(4.5, 9.0);
                let depth = (row_depth * r.range(0.8, 1.0)).max(ROW_DEPTH_FLOOR);
                let (s0, s1) = (start + dir * t, start + dir * (t + w));
                t += w;
                if !arc.closed && (s0.min(s1) < 0.0 || s0.max(s1) > total) {
                    break;
                }
                if let Some((quad, cls)) = faubourg_lot(&ctx, &arc, ring, hw, s0, s1, off, depth) {
                    push(&mut ctx, quad, format!("faub{k}"), WallBacking::OutsideRow, cls, &mut tone);
                    k += 1;
                }
            }
            off += row_depth + lane(&mut r);
        }
    }
    out
}

#[cfg(test)]
mod tests;
