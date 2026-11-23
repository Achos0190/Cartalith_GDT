//! **Ruling HG (owner, 2026-11-22): a settlement sits on the river bank, not in
//! the river.** *"Only thing for on the map is that the settlements detect the
//! edge of a river and sit on the bank, not in it."*
//!
//! **What this is.** One answer to "where, beside the drawn river, does the
//! settlement at this cell stand?", read by everything that shows a settlement
//! on the map. The river is the one the map draws ([`RiverGeometry`], the
//! geometry `WorldGen::river_geometry_any` hands the strokes, the tiles and the
//! exports), and its edge is the drawn stroke's edge: half of
//! [`river_px_width`], the one seam that decides how wide a river is in a
//! raster, of every drawn stroke in reach. Not the cell centre and not the
//! flow field.
//!
//! **Why it is derived and not stored.** A settlement reaches the map by many
//! paths (generation, the river constraint, junction hamlets at generation and
//! in a recovery run, seeded villages, a hand drop with or without "Snap to
//! water", an import), and the drawn stroke's width in cells depends on the
//! zoom (a river is never drawn under one pixel wide,
//! `river_stroke::MIN_STROKE_PX`). A position nudged once on one of those
//! paths would miss the others and would be right at one zoom only. So the
//! settlement keeps its cell, which every generated layer (roads, territory,
//! the suitability terms, the layout's water context) was built on, and the
//! bank point is computed from that cell and the drawn river wherever a marker
//! is shown. Nothing is written to a save: a world shows its settlements on
//! the bank exactly when its `civ.bank_sites` switch is on.
//!
//! Must never move a settlement's cell, and must never answer a point in open
//! water or inside a drawn stroke.

use std::collections::HashMap;

use godot::prelude::*;

use crate::render::TerrainAppearance;
use crate::WorldGen;
use crate::river_stroke::{RiverGeometry, RiverPoint, river_px_width};

/// The nearest drawn river point to a position ([`StrokeIndex::nearest`]).
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    /// The foot of the perpendicular on the drawn centreline, grid cells in
    /// the traced frame (a cell's centre is `x + 0.5`).
    pub foot: (f64, f64),
    /// Distance from the position to `foot`, cells.
    pub d: f64,
    /// The unit vector from `foot` toward the position: the bank the position
    /// is on. Where the position is on the centreline itself (`d` under
    /// [`ON_LINE_EPS`]) it is the segment's left normal, a fixed choice so the
    /// answer never depends on float noise.
    pub n: (f64, f64),
    /// The drawn point at the foot, interpolated along its segment (width and
    /// rank linearly, the rest from the segment's first point).
    pub point: RiverPoint,
}

/// Under this distance, in cells, a position counts as on the centreline and
/// [`Hit::n`] is the segment's left normal. A labelled judgement: far under any
/// drawn width (the narrowest stroke at the deepest tile is some thousandths of
/// a cell) and far over `f64` noise on grid coordinates.
pub const ON_LINE_EPS: f64 = 1e-6;

/// Every drawn segment of a [`RiverGeometry`], bucketed by the grid cell it
/// passes through, so the nearest stroke to a settlement is a look at a few
/// cells and not at the whole network. Built once per drawn network.
///
/// Holds only what the map strokes: the segments inside each run's `pieces`
/// (a run is cut where it crosses a lake). The carried shore reach and the
/// delta fans are left out on purpose: both lie in or at open water, where no
/// settlement stands.
pub struct StrokeIndex {
    /// Cell -> `(run, first point of the segment)`.
    cells: HashMap<(i32, i32), Vec<(u32, u32)>>,
    /// [`Self::reach_widths`], built on first use.
    reach: std::sync::OnceLock<Vec<Vec<f32>>>,
}

impl StrokeIndex {
    /// Index `geom`. A segment is filed under every cell of its bounding box
    /// (a drawn segment is about a cell long, so that is one to four cells).
    pub fn new(geom: &RiverGeometry) -> StrokeIndex {
        let mut cells: HashMap<(i32, i32), Vec<(u32, u32)>> = HashMap::new();
        for (r, run) in geom.runs.iter().enumerate() {
            for &(a, b) in &run.pieces {
                for i in a..b.min(run.pts.len()).saturating_sub(1) {
                    let (p, q) = (run.pts[i], run.pts[i + 1]);
                    let (x0, x1) = (p.0.min(q.0).floor() as i32, p.0.max(q.0).floor() as i32);
                    let (y0, y1) = (p.1.min(q.1).floor() as i32, p.1.max(q.1).floor() as i32);
                    for y in y0..=y1 {
                        for x in x0..=x1 {
                            cells.entry((x, y)).or_default().push((r as u32, i as u32));
                        }
                    }
                }
            }
        }
        StrokeIndex { cells, reach: std::sync::OnceLock::new() }
    }

    /// The drawn point of `geom` nearest `p` (grid cells, the traced frame)
    /// among the segments passing within `reach` cells' worth of buckets
    /// (a square of `ceil(reach) + 1` cells each way), or `None` when no drawn
    /// segment is that near. `geom` must be the geometry the index was built
    /// from.
    pub fn nearest(&self, geom: &RiverGeometry, p: (f64, f64), reach: f64) -> Option<Hit> {
        let r = reach.max(0.0).ceil() as i32 + 1;
        let (cx, cy) = (p.0.floor() as i32, p.1.floor() as i32);
        let mut best: Option<Hit> = None;
        for y in cy - r..=cy + r {
            for x in cx - r..=cx + r {
                let Some(segs) = self.cells.get(&(x, y)) else { continue };
                for &(ri, i) in segs {
                    let hit = segment_hit(geom, ri, i, p);
                    if best.as_ref().is_none_or(|b| hit.d < b.d) {
                        best = Some(hit);
                    }
                }
            }
        }
        best
    }

    /// **Ruling HM: each run's widest width within `reach` cells of run
    /// length either way of every point**, cells (`geom`'s `widths`, dilated
    /// along the run; by length, since a run's points can be many to a cell). A
    /// town lays its river at one width, the widest the map draws it inside
    /// the town's box (`urban_adapter::um_water_ctx_box_smooth`), so the edge
    /// its origin must clear is that width's, not the width at the nearest
    /// point: with the local width a town a little upstream of a confluence
    /// had its box centre in its own channel and the layout moved its market
    /// to whichever bank came first. Built once (the first `reach` asked for
    /// is kept: a world has one), from the geometry the index was built from.
    pub fn reach_widths(&self, geom: &RiverGeometry, reach: f64) -> &[Vec<f32>] {
        self.reach.get_or_init(|| {
            geom.runs
                .iter()
                .map(|r| {
                    // Run length to each point, cells.
                    let mut at = vec![0.0f64; r.pts.len()];
                    for i in 1..r.pts.len() {
                        at[i] = at[i - 1] + f64::from(r.pts[i].0 - r.pts[i - 1].0).hypot(f64::from(r.pts[i].1 - r.pts[i - 1].1));
                    }
                    (0..r.widths.len().min(at.len()))
                        .map(|i| {
                            let lo = at.partition_point(|&d| d < at[i] - reach);
                            let hi = at.partition_point(|&d| d <= at[i] + reach).min(r.widths.len());
                            r.widths[lo..hi].iter().copied().fold(0.0f32, f32::max)
                        })
                        .collect()
                })
                .collect()
        })
    }

    /// Whether the straight line from `a` to `b` (traced frame, cells)
    /// crosses a drawn centreline of `geom`: the two points stand on
    /// opposite banks of some drawn river. A proper crossing only (a point
    /// exactly on a centreline is on neither bank and crosses nothing).
    /// `drawn` says whether the river at a point counts (a river the zoom
    /// hides has no banks on screen).
    pub fn crosses(&self, geom: &RiverGeometry, a: (f64, f64), b: (f64, f64), drawn: &dyn Fn(RiverPoint) -> bool) -> bool {
        let cross = |o: (f64, f64), p: (f64, f64), q: (f64, f64)| (p.0 - o.0) * (q.1 - o.1) - (p.1 - o.1) * (q.0 - o.0);
        let (x0, x1) = (a.0.min(b.0).floor() as i32 - 1, a.0.max(b.0).floor() as i32 + 1);
        let (y0, y1) = (a.1.min(b.1).floor() as i32 - 1, a.1.max(b.1).floor() as i32 + 1);
        (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter_map(|c| self.cells.get(&c)).flatten().any(|&(ri, i)| {
            let run = &geom.runs[ri as usize];
            let (p, q) = (run.pts[i as usize], run.pts[i as usize + 1]);
            let (p, q) = ((f64::from(p.0), f64::from(p.1)), (f64::from(q.0), f64::from(q.1)));
            cross(a, b, p) * cross(a, b, q) < 0.0 && cross(p, q, a) * cross(p, q, b) < 0.0 && drawn(segment_hit(geom, ri, i, a).point)
        })
    }
}

/// The [`Hit`] of `p` on one drawn segment: run `ri` of `geom`, points `i` and
/// `i + 1`.
fn segment_hit(geom: &RiverGeometry, ri: u32, i: u32, p: (f64, f64)) -> Hit {
    let run = &geom.runs[ri as usize];
    let i = i as usize;
    let (a, b) = (run.pts[i], run.pts[i + 1]);
    let (ax, ay, bx, by) = (f64::from(a.0), f64::from(a.1), f64::from(b.0), f64::from(b.1));
    let (dx, dy) = (bx - ax, by - ay);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.0 - ax) * dx + (p.1 - ay) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let foot = (ax + t * dx, ay + t * dy);
    let d = (p.0 - foot.0).hypot(p.1 - foot.1);
    let n = if d > ON_LINE_EPS {
        ((p.0 - foot.0) / d, (p.1 - foot.1) / d)
    } else {
        let l = l2.sqrt().max(1e-12);
        (-dy / l, dx / l)
    };
    let lerp = |u: f32, v: f32| u + (v - u) * t as f32;
    let ranked = !run.ranks.is_empty();
    let rank = |k: usize| if ranked { run.ranks.get(k).copied().unwrap_or(0.0) } else { 0.0 };
    let point = RiverPoint {
        width_cells: lerp(run.widths[i], run.widths[i + 1]),
        order: run.orders[i],
        own_order: run.own_order,
        discharge: run.discharge[i],
        rank: lerp(rank(i), rank(i + 1)),
        head: if ranked { run.detail_at(i)[2] } else { 1.0 },
    };
    Hit { foot, d, n, point }
}

/// Half the drawn stroke's width at `point`, in grid cells, in a raster of
/// `ppc` pixels per cell under look `a`: half of [`river_px_width`] over `ppc`.
/// `None` when the river is not drawn there at that density (hidden by the
/// zoom law, or fading under `river_stroke::PICK_MIN_VISIBILITY`, the alpha from
/// which a river counts as drawn for a click, so a marker and a pick agree). Read under the scale in force
/// (`river_stroke::scale`); a caller measuring another window holds a
/// `river_stroke::ScaleGuard`.
pub fn half_width_cells(point: RiverPoint, ppc: f32, a: &TerrainAppearance) -> Option<f64> {
    if !(ppc > 0.0) {
        return None;
    }
    let (w_px, alpha) = river_px_width(point, ppc, a)?;
    (alpha >= crate::river_stroke::PICK_MIN_VISIBILITY).then(|| f64::from(w_px) / 2.0 / f64::from(ppc))
}

/// How far the search for a bank point looks past the covering stroke's own
/// clearance, as a multiple of that clearance ([`bank_point`]). **A labelled
/// judgement**: twice the clearance reaches the far bank of the same stroke and
/// the first clear ground beside a second stroke of the same size at a
/// confluence; further than that the marker would no longer read as that
/// settlement's.
pub const SEARCH_REACH: f64 = 2.0;

/// The directions the ring search of [`bank_point`] tries at each radius. **A
/// labelled judgement**: sixteen leaves at most 22.5 degrees between a tried
/// direction and the true nearest clear one.
pub const SEARCH_DIRS: usize = 16;

/// What [`bank_point`] reads: the drawn river, the look and density it is drawn
/// at, how far past the edge a marker stands, and which ground is dry.
#[derive(Clone, Copy)]
pub struct BankQuery<'a> {
    /// The drawn network and its index ([`StrokeIndex::new`] of the same geometry).
    pub geom: &'a RiverGeometry,
    /// See `geom`.
    pub index: &'a StrokeIndex,
    /// The look the river is drawn under (its `river_width` scales every stroke).
    pub a: &'a TerrainAppearance,
    /// The raster's pixels per grid cell.
    pub ppc: f32,
    /// How far outside the stroke's edge the point stands, in cells (the
    /// caller's gap in pixels over `ppc`): the marker's own ink must clear the
    /// water, and only the caller knows how big its marker is.
    pub gap: f64,
    /// Whether the cell holding a point (traced frame) is dry land: not sea,
    /// not a lake. A bank point is never put in open water.
    pub dry: &'a dyn Fn(f64, f64) -> bool,
    /// [`widest_half`] of `geom` under `a`: how far a stroke's edge can be
    /// from its centreline before the one-pixel floor, so the search for a
    /// covering stroke looks far enough and no further.
    pub wide: f64,
    /// **Ruling HM, the town's own edge**: `Some(min)` measures every stroke
    /// at its width on the ground and not as one view draws it: half of
    /// `width_cells`, never under `min` cells, with no pixel floor, no look
    /// and no zoom law (`ppc` and `a` are then not read). A town's layout is
    /// generated once and may be frozen in a save, so its origin cannot
    /// follow the zoom, a marker size or the CARTO river-width slider; this
    /// is the edge every view's stroke converges on as the zoom deepens
    /// under the default look. `None`: the stroke as drawn at `ppc`.
    pub ground: Option<f64>,
    /// With [`Self::ground`]: the width to measure each run's points at
    /// ([`StrokeIndex::reach_widths`]) in place of the run's own; `None`
    /// reads the run's own width at the point.
    pub reach_widths: Option<&'a [Vec<f32>]>,
    /// **Ruling HM, the marker and its market on one bank.** With a market
    /// given to [`bank_point`], accept only a point with no drawn centreline
    /// between it and the market ([`StrokeIndex::crosses`]), so the ring
    /// search at a confluence cannot hand the marker to another bank. Off
    /// for [`WorldGen::town_origin`], whose `market` is only the side its
    /// roads are on, not a place.
    pub same_bank: bool,
}

/// Half the widest drawn width in `geom` under look `a`, in cells: the widest
/// `widths` entry times the look's `river_width`, halved. `0` for an empty
/// network.
pub fn widest_half(geom: &RiverGeometry, a: &TerrainAppearance) -> f64 {
    widest_ground_half(geom) * f64::from(crate::river_stroke::river_width_factor(a))
}

/// Half the widest width on the ground in `geom`, in cells: [`widest_half`]
/// without a look's multiplier (Ruling HM's [`BankQuery::ground`]).
pub fn widest_ground_half(geom: &RiverGeometry) -> f64 {
    f64::from(geom.runs.iter().flat_map(|r| r.widths.iter().copied()).fold(0.0f32, f32::max)) / 2.0
}

impl BankQuery<'_> {
    /// The drawn stroke `p` is deepest inside, counting the gap: the segment
    /// maximising `half + gap - d` over every drawn segment in reach, when
    /// that is positive. `None` when `p` is clear of every stroke. **Every
    /// stroke, not the nearest centreline**: at a confluence a point can clear
    /// a thin tributary and still stand inside the wider trunk.
    pub fn covering(&self, p: (f64, f64)) -> Option<(Hit, f64)> {
        // No drawn half-width exceeds the wider of the widest stroke's own and
        // the one-pixel floor's (`river_stroke::MIN_STROKE_PX`, halved).
        // On the ground the floor is the town's own minimum clearance.
        let floor = self.ground.unwrap_or_else(|| f64::from(crate::river_stroke::MIN_STROKE_PX) / 2.0 / f64::from(self.ppc.max(1e-6)));
        let reach = self.gap + self.wide.max(floor);
        let r = reach.ceil() as i32 + 1;
        let (cx, cy) = (p.0.floor() as i32, p.1.floor() as i32);
        let mut best: Option<(Hit, f64, f64)> = None;
        for y in cy - r..=cy + r {
            for x in cx - r..=cx + r {
                let Some(segs) = self.index.cells.get(&(x, y)) else { continue };
                for &(ri, i) in segs {
                    let hit = segment_hit(self.geom, ri, i, p);
                    let half = match (self.ground, self.reach_widths) {
                        (Some(min), Some(w)) => Some((f64::from(w[ri as usize][i as usize].max(w[ri as usize][i as usize + 1])) / 2.0).max(min)),
                        _ => self.half(hit.point),
                    };
                    let Some(half) = half else { continue };
                    let depth = half + self.gap - hit.d;
                    if depth > CLEAR_EPS && best.as_ref().is_none_or(|b| depth > b.2) {
                        best = Some((hit, half, depth));
                    }
                }
            }
        }
        best.map(|(h, half, _)| (h, half))
    }

    /// Half the width of the stroke at `point` as this query measures it, in
    /// cells: on the ground ([`Self::ground`]) or as drawn
    /// ([`half_width_cells`], `None` where the zoom hides the river).
    pub fn half(&self, point: RiverPoint) -> Option<f64> {
        match self.ground {
            Some(min) => Some((f64::from(point.width_cells) / 2.0).max(min)),
            None => half_width_cells(point, self.ppc, self.a),
        }
    }
}

/// A point this much inside a clearance still counts as clear, in cells: the
/// candidate [`bank_point`] builds stands exactly on the clearance and must
/// not be refused by its own rounding. Far under a thousandth of a pixel at
/// any density the map is drawn at.
pub const CLEAR_EPS: f64 = 1e-7;

/// The bank a settlement whose cell centre is on the centreline itself is
/// given, as a direction: west, and north where the river runs east to west.
/// **Why this one: it is where such a town's market most often stands,
/// measured, not derived.** The town layout moves a market that would stand
/// in its channel onto a bank (`cartalith_urban::generate`: the first dry
/// point of a ring, then the roads' bank), and on seeds 483920, 912345 and
/// 246371 at 384 x 288 the 41 such towns whose market left the centreline
/// have it west or north of the river 30 times and east or south 11 (9 to 5,
/// 7 to 4, 14 to 2; `_bankcensus_probe.gd --layouts`). **A weak lean, not a
/// law**: on seed 83744, which was not used to choose it, the count is 10 to
/// 8. Which hand of the flow it takes shows no preference (20 right, 21
/// left). The cause in the layout was not traced. **Since Ruling HM
/// (2026-11-23) this is only the last tie-break**: a marker follows its
/// market's bank ([`bank_point`]'s `market`) and a town's origin follows its
/// roads' bank ([`roads_bank`]); this direction decides for a centre on the
/// line with neither.
pub const MARKET_FIRST_BANK: (f64, f64) = (-1.0, -0.13);

/// **Where the settlement whose cell centre is `centre` stands beside the
/// drawn river** (traced frame, cells): the nearest dry point clear of every
/// drawn stroke by the gap, or `None` when the centre is already clear (the
/// settlement stays where it is) or no such point is found within
/// [`SEARCH_REACH`] (it stays, and the census counts it).
///
/// **Where the gap does not fit, the water's edge will do**: a settlement on a
/// strip of land narrower than the gap (between a river and the sea, or two
/// rivers) has no point the gap clear of every stroke, so the search is run
/// again with no gap. Out of the water without the margin is the ruling;
/// in the water for want of the margin is not. (Measured: 10 of 222 923
/// simulated hand drops at 2 048 cells at the opening view, where two pixels
/// of gap are over four cells.)
///
/// The first choice is the bank the centre is already on: straight out from
/// the covering stroke's centreline to its edge plus the gap. Failing that
/// (open water there, or a second stroke), rings of [`SEARCH_DIRS`]
/// directions round the centre at growing radius, nearest first and starting
/// toward its own bank: that is what finds dry ground at a confluence or
/// beside a lake, and the stroke's other bank last.
/// `prefer` (a unit vector) names the bank to try first where the centre is on
/// the centreline itself and has no bank of its own.
///
/// **Ruling HM: the marker follows the market's bank.** `market` is where the
/// settlement's town has its market (traced frame, cells; `None`: not known,
/// the centre's own bank). The point is put on the market's side of the
/// stroke whichever side the centre is on, so the map and the City Viewer
/// agree; and a centre that is clear of every stroke but stands **across the
/// river from its market** (a village whose market went over to its roads'
/// bank) is moved over to the market's bank too, which is the one case a
/// clear centre moves.
///
/// **Ruling HN (owner, 2026-11-23): on the market's bank in every town.**
/// Where the search round the centre finds no clear point on the market's
/// bank (with the gap, then without), the marker goes to the clear point
/// nearest the **market** ([`near_market`], with the gap, then without), and
/// failing that to the market itself when it is dry: on a tongue of land
/// between two strokes that the zoom's one-pixel floor closes over, no point
/// is clear, and the market's own ground (out of the river's real water, by
/// the town layout) is the bank. So the marker may stand inside a stroke's
/// pixel floor there; it is never across a drawn river from its market while
/// the market is dry. The cases, measured before this rule: a centre clear
/// of every stroke whose one candidate across the nearest river was refused
/// (the market lay beyond a second river), and a confluence at the opening
/// zoom (6 towns of about 1 230, 8 rows).
///
/// Only a market in the water still gives the marker any clear point: out
/// of the water on the wrong bank is the older ruling (HG).
///
/// Must never return a point in open water, and never one inside a drawn
/// stroke unless it is the market's own dry ground.
pub fn bank_point(q: &BankQuery, centre: (f64, f64), market: Option<(f64, f64)>, prefer: Option<(f64, f64)>) -> Option<(f64, f64)> {
    let two = |q: &BankQuery| bank_point_at_gap(q, centre, market, prefer).or_else(|| if q.gap > 0.0 { bank_point_at_gap(&BankQuery { gap: 0.0, ..*q }, centre, market, prefer) } else { None });
    let first = two(q);
    let Some(m) = market.filter(|_| q.same_bank) else { return first };
    // Found, or nothing to find: the centre is clear and on its market's bank already.
    if first.is_some() || (q.covering(centre).is_none() && !q.index.crosses(q.geom, centre, m, &|pt| q.half(pt).is_some())) {
        return first;
    }
    near_market(q, m)
        .or_else(|| if q.gap > 0.0 { near_market(&BankQuery { gap: 0.0, ..*q }, m) } else { None })
        .or_else(|| (q.dry)(m.0, m.1).then_some(m))
        .or_else(|| two(&BankQuery { same_bank: false, ..*q }))
}

/// **Ruling HN: the clear point nearest market `m`** on its own bank: `m`
/// itself, else the first dry point clear of every drawn stroke by the gap,
/// with no drawn centreline between it and `m`, on rings of [`SEARCH_DIRS`]
/// directions round `m` out to [`SEARCH_REACH`] times the clearance (the
/// stroke covering `m`, or the widest, plus the gap), nearest ring first.
/// `None` when there is none (see [`bank_point`]).
fn near_market(q: &BankQuery, m: (f64, f64)) -> Option<(f64, f64)> {
    let clear = |p: (f64, f64)| (q.dry)(p.0, p.1) && q.covering(p).is_none() && !q.index.crosses(q.geom, p, m, &|pt| q.half(pt).is_some());
    if clear(m) {
        return Some(m);
    }
    let c = q.covering(m).map_or(q.wide, |(_, half)| half.max(q.wide)) + q.gap;
    let steps = 8usize;
    for k in 1..=steps {
        let r = c * SEARCH_REACH * k as f64 / steps as f64;
        for j in 0..SEARCH_DIRS {
            let ang = j as f64 * std::f64::consts::TAU / SEARCH_DIRS as f64;
            let p = (m.0 + r * ang.cos(), m.1 + r * ang.sin());
            if clear(p) {
                return Some(p);
            }
        }
    }
    None
}

/// The side of the stroke at `hit` that `p` is on, as `hit.n` or its
/// opposite; `None` for a point on the line itself (within [`ON_LINE_EPS`]).
fn side_of(hit: &Hit, p: (f64, f64)) -> Option<(f64, f64)> {
    let s = (p.0 - hit.foot.0) * hit.n.0 + (p.1 - hit.foot.1) * hit.n.1;
    (s.abs() > ON_LINE_EPS).then(|| if s < 0.0 { (-hit.n.0, -hit.n.1) } else { hit.n })
}

/// [`bank_point`] at `q.gap` exactly, with no second try.
fn bank_point_at_gap(q: &BankQuery, centre: (f64, f64), market: Option<(f64, f64)>, prefer: Option<(f64, f64)>) -> Option<(f64, f64)> {
    let clear = |p: (f64, f64)| (q.dry)(p.0, p.1) && q.covering(p).is_none() && !(q.same_bank && market.is_some_and(|m| q.index.crosses(q.geom, p, m, &|pt| q.half(pt).is_some())));
    let Some((hit, half)) = q.covering(centre) else {
        // Clear of every stroke: it stays, unless its market is across the
        // nearest drawn river (Ruling HM). The search reaches as far as the
        // market is, plus the widest stroke.
        let m = market?;
        let hit = q.index.nearest(q.geom, centre, (m.0 - centre.0).hypot(m.1 - centre.1) + q.wide)?;
        let n = side_of(&hit, m).filter(|n| n.0 * hit.n.0 + n.1 * hit.n.1 < 0.0)?;
        let c = q.half(hit.point)? + q.gap;
        let p = (hit.foot.0 + n.0 * c, hit.foot.1 + n.1 * c);
        return clear(p).then_some(p);
    };
    let c = half + q.gap;
    let n = match (market.and_then(|m| side_of(&hit, m)), prefer) {
        // Ruling HM: the market's bank, whichever side the centre is on.
        (Some(n), _) => n,
        (None, Some(v)) if hit.d <= ON_LINE_EPS => {
            // The preferred bank, as a side of this stroke.
            if v.0 * hit.n.0 + v.1 * hit.n.1 < 0.0 { (-hit.n.0, -hit.n.1) } else { hit.n }
        }
        _ => hit.n,
    };
    let p = (hit.foot.0 + n.0 * c, hit.foot.1 + n.1 * c);
    if clear(p) {
        return Some(p);
    }
    let (steps, start) = (8usize, n.1.atan2(n.0));
    for k in 1..=steps {
        let r = c * SEARCH_REACH * k as f64 / steps as f64;
        for j in 0..SEARCH_DIRS {
            // Alternate either way from the centre's own bank: 0, +1, -1, +2, ...
            let turn = if j % 2 == 1 { j.div_ceil(2) as f64 } else { -((j / 2) as f64) };
            let ang = start + turn * std::f64::consts::TAU / SEARCH_DIRS as f64;
            let p = (centre.0 + r * ang.cos(), centre.1 + r * ang.sin());
            if clear(p) {
                return Some(p);
            }
        }
    }
    None
}

/// The gap, in screen pixels, the shell's settlement markers keep from the
/// drawn river's edge when the caller names none: the pin's tip, with the one
/// pixel of antialiased fringe either side of a stroke's edge. **A labelled
/// judgement** (owner Ruling HG asks for "on the bank, not in it"; the frames
/// in `docs/evidence/settlements-on-bank-2026-11-22/` are what it was judged on).
pub const MARKER_GAP_PX: f64 = 2.0;

/// **Ruling HM: the least a town's origin stands from its river's centreline,
/// metres**, whatever the drawn width ([`WorldGen::town_origin`]). On a fine
/// grid the drawn river is narrower than the channel the town itself lays
/// (12 to 46 m, `urban_adapter::um_water_ctx_box_smooth`), and a box centre
/// in that channel is moved by the layout to the first dry ring point,
/// whichever bank that is. **Source**: `cartalith_urban::routes::place_anchors`'
/// own nearest market distance from a river (its `d_band` floor, 60 m), which
/// clears half the widest channel (23 m) and the 30 m flood band beside it.
pub const ORIGIN_MIN_CLEAR_M: f64 = 60.0;

/// **Ruling HM: how far past the drawn river's edge a town's origin stands,
/// metres** ([`WorldGen::town_origin`]). The town lays its river as wide as
/// the map draws it (`urban_adapter::DrawnRiver::widths`), and its market,
/// which is the origin, must be on dry ground beside it. **Source**:
/// `cartalith_urban::routes::place_anchors`' flood band, the 30 m beside a
/// channel it never puts a market in; it also covers the half cell (11 m) by
/// which the town's 22 m water raster widens a stamped band.
pub const ORIGIN_BANK_GAP_M: f64 = 30.0;

/// **The settlement's roads as the town reads them**, for [`origin_point`]:
/// one list of points per road, walking outward from the settlement, in
/// the traced frame (cells). The roads are the ones the town is laid out
/// with (`urban_adapter::um_primary_paths_reach`: every shown way ending at
/// the cell `at`, sampled every 55 m of path out to `reach_m`), carried from
/// metre offsets round the cell centre `centre`. Empty with no road.
pub fn road_points(ways: &[cartalith_civ::Way], at: (f64, f64), centre: (f64, f64), gw: usize, map_width_km: f64, reach_m: f64) -> Vec<Vec<(f64, f64)>> {
    let gpm = gw as f64 / (map_width_km * 1000.0);
    cartalith_civ::urban_adapter::um_primary_paths_reach(ways, at.0, at.1, gw, map_width_km, 0.0, reach_m)
        .unwrap_or_default()
        .iter()
        .map(|path| path.iter().map(|o| (centre.0 + o.x * gpm, centre.1 + o.y * gpm)).collect())
        .collect()
}

/// **Ruling HM: the bank a settlement's roads are on**, as a point one cell
/// out from the stroke at `hit` on that side (a side for [`bank_point`]'s
/// `market`, not a place), or `None` when the roads favour neither bank.
///
/// `roads` are [`road_points`], and each sample counts one for the side of
/// the stroke it is on. A town is laid out from its market along these
/// roads, and a market put across a river as wide as the map draws it from
/// all of them houses almost nobody (measured on seed 483920 at 384 cells
/// before this rule: hamlets at 1 to 13 % of their target). **A labelled
/// judgement**: a majority of samples, any margin.
fn roads_bank(hit: &Hit, roads: &[Vec<(f64, f64)>]) -> Option<(f64, f64)> {
    let sum: f64 = roads.iter().flatten().filter_map(|&p| side_of(hit, p)).map(|n| (n.0 * hit.n.0 + n.1 * hit.n.1).signum()).sum();
    (sum != 0.0).then(|| (hit.foot.0 + hit.n.0 * sum.signum(), hit.foot.1 + hit.n.1 * sum.signum()))
}

/// How far along its roads a town's origin looks for the bank
/// ([`road_points`]' reach for [`origin_point`]), metres, on a grid of
/// `cell_m` metres a cell: the long side of the box a town on the widest
/// drawn river is laid out in (`urban_adapter::um_keep_river_box`), which is
/// as far as the town's own layout follows a road.
pub fn origin_road_reach_m(geom: &RiverGeometry, cell_m: f64) -> f64 {
    cartalith_civ::urban_adapter::SITE_WM + 2.0 * (widest_ground_half(geom) * cell_m + ORIGIN_BANK_GAP_M + ORIGIN_MIN_CLEAR_M)
}

/// [`WorldGen::town_origin`]'s rule, apart from the world it reads: the town
/// point for a settlement whose cell centre is `c` (traced frame, cells) on a
/// grid of `cell_m` metres a cell, and its distance from the nearest drawn
/// centreline in cells. `roads` are the settlement's roads ([`road_points`]).
/// `None` where `c` is already clear or no dry clear point is in reach.
///
/// **Ruling HN (owner, 2026-11-23): the town stands where its road reaches
/// the bank.** The first point along a road that is dry and clear of every
/// drawn river (by the gap, at the ground width), the nearest of them over
/// the roads, within the search's own reach of the centre
/// ([`SEARCH_REACH`] times the clearance). Why: the map routes a road to a
/// river settlement's cell, which is in the river, and the town is laid out
/// from its market along its roads. The older rule (the roads' bank of the
/// stroke under the centre, then the nearest clear point on any bank) lost
/// the roads' bank whenever its one candidate was refused: at a confluence
/// the stroke under the centre can be the tributary's while the road lands
/// on the trunk's far bank, and the town was put across the trunk from its
/// only road (seed 83744 #164 at 384 cells: 10 people of 400). A settlement
/// with no road, or whose roads stay in the water within that reach, keeps
/// the older rule.
pub fn origin_point(geom: &RiverGeometry, index: &StrokeIndex, dry: &dyn Fn(f64, f64) -> bool, cell_m: f64, c: (f64, f64), roads: &[Vec<(f64, f64)>]) -> Option<((f64, f64), f64)> {
    // `a` and `ppc` are not read on the ground.
    let a = TerrainAppearance::default();
    let min = ORIGIN_MIN_CLEAR_M / cell_m;
    let wide = widest_ground_half(geom);
    // The widest the river is drawn anywhere the town's box can reach along it: the points within
    // the adapter's own stem reach (`um_water_ctx_box_smooth`: 1.8 cells, or three quarters of the
    // box's long side, here the box grown for the widest river), half as far again since the
    // adapter's reach is straight-line and this one is along the run.
    let long_m = origin_road_reach_m(geom, cell_m);
    let k = (0.75 * long_m / cell_m).max(1.8) * 1.5;
    let q = BankQuery { geom, index, a: &a, ppc: 1.0, gap: ORIGIN_BANK_GAP_M / cell_m, dry, wide, ground: Some(min), reach_widths: Some(index.reach_widths(geom, k)), same_bank: false };
    let (hit, half) = q.covering(c)?;
    // Ruling HN: where a road reaches the bank.
    let reach = (half + q.gap) * SEARCH_REACH;
    let off = |p: (f64, f64)| (p.0 - c.0).hypot(p.1 - c.1);
    let landing = roads
        .iter()
        .filter_map(|r| r.iter().copied().find(|&p| dry(p.0, p.1) && q.covering(p).is_none()))
        .filter(|&p| off(p) <= reach)
        .min_by(|&x, &y| off(x).total_cmp(&off(y)));
    // Else the bank the settlement's roads are on, else the centre's own.
    let p = match landing {
        Some(p) => p,
        None => bank_point(&q, c, roads_bank(&hit, roads), Some(MARKET_FIRST_BANK))?,
    };
    let river = index.nearest(geom, p, (wide.max(min) + q.gap) * (1.0 + SEARCH_REACH) + 1.0)?;
    Some((p, river.d))
}

/// **Ruling HU: the markers' table being built off the main thread** (`WorldGen::market_points_marker`): the
/// `market_key` it is built for, and its thread, which answers the table and its milliseconds.
pub(crate) struct MarketJob {
    key: u64,
    handle: std::thread::JoinHandle<(Vec<Option<(f64, f64)>>, f64)>,
}

/// Everything [`market_table`] reads, owned (Ruling HU; `WorldGen::market_snap` fills it). Holds no Godot value
/// and nothing behind a `RefCell`, so it can be moved to another thread.
pub(crate) struct MarketSnap {
    field: std::sync::Arc<Vec<f32>>,
    flow: std::sync::Arc<Vec<f32>>,
    order: Option<Vec<i16>>,
    recv: Option<Vec<i32>>,
    water_bodies: Vec<u8>,
    settlements: Vec<cartalith_civ::NamedSettlement>,
    ways: Vec<cartalith_civ::Way>,
    gw: usize,
    gh: usize,
    sea_level: f64,
    map_width_km: f64,
    seed: i32,
    params: cartalith_engine::WorldParams,
    /// `WorldGen::town_map_water` (the town's water raster) and `WorldGen::town_water` (the origin's dry test).
    map_water: Option<std::sync::Arc<crate::town_water::DrawnWater>>,
    town_water: Option<std::sync::Arc<crate::town_water::DrawnWater>>,
    resources: Option<cartalith_civ::ResourcePotentials>,
    drawn: Vec<cartalith_civ::urban_adapter::DrawnRiver>,
    /// Each settlement's place-editor row and ruling rite, by `tid`.
    extras: HashMap<u64, crate::civ_roster_bridge::PlaceExtras>,
    faith: HashMap<u64, Option<&'static str>>,
    urban_rules: Option<cartalith_civ::urban_adapter::Rules>,
    bank: Option<(std::sync::Arc<RiverGeometry>, std::sync::Arc<StrokeIndex>)>,
    /// The stretched towns' verdicts as they stand (`WorldGen::stretch_choices`); a verdict reached here is not kept.
    stretch: HashMap<u64, cartalith_civ::urban_adapter::StretchChoice>,
    /// A frozen town's own market, per settlement (`None`: not frozen).
    frozen: Vec<Option<(f64, f64)>>,
}

/// **The markers' table** (`WorldGen::market_points`'s doc says what each entry is), built from a snapshot so the
/// same code runs on the main thread (the labels, the generation worker's warm-up) and on the markers' job (Ruling
/// HU). One builder of the town context with the drawn town (`urban_bridge::town_batch_in`).
pub(crate) fn market_table(m: &MarketSnap) -> Vec<Option<(f64, f64)>> {
    use cartalith_civ::urban_adapter as ua;
    let mut out = m.frozen.clone();
    let rest: Vec<usize> = (0..out.len()).filter(|&i| out[i].is_none()).collect();
    let mpc = m.map_width_km * 1000.0 / m.gw.max(1) as f64;
    let origin = |s: &cartalith_civ::NamedSettlement| {
        let (geom, index) = m.bank.as_ref()?;
        m.params.civ.bank_sites.then(|| origin_in(&OriginIn { field: &m.field, water_bodies: &m.water_bodies, ways: &m.ways, drawn: m.town_water.as_deref(), geom, index, gw: m.gw, gh: m.gh, sea_level: m.sea_level, map_width_km: m.map_width_km }, s)).flatten()
    };
    let b = crate::urban_bridge::TownInputs {
        field: &m.field,
        flow: &m.flow,
        order: m.order.as_deref(),
        recv: m.recv.as_deref(),
        water_bodies: &m.water_bodies,
        settlements: &m.settlements,
        ways: &m.ways,
        gw: m.gw,
        gh: m.gh,
        sea_level: m.sea_level,
        map_width_km: m.map_width_km,
        seed: m.seed,
        params: &m.params,
        map_water: m.map_water.as_deref(),
        resources: m.resources.as_ref(),
        drawn: &m.drawn,
        extras: &|s| m.extras.get(&s.tid).cloned().unwrap_or_default(),
        faith: &|s| m.faith.get(&s.tid).copied().flatten(),
        rules: &|e| crate::urban_bridge::own_rules_in(m.urban_rules, e),
        origin: &origin,
    };
    for (i, p) in crate::urban_bridge::town_batch_in(&b, &rest, |_, s, ctx, rules, o, _| {
        let at = o.origin.map_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), |b| (b.at.0 + 0.5, b.at.1 + 0.5));
        let cand: Vec<(f64, f64)> = ua::um_market_offsets(ctx, rules).iter().map(|v| (at.0 + v.x / mpc, at.1 + v.y / mpc)).collect();
        // Candidates on two banks of any drawn centreline: only the layout knows which is kept.
        let split = m.bank.as_ref().is_some_and(|(geom, index)| cand.iter().skip(1).any(|&q| index.crosses(geom, cand[0], q, &|_| true)));
        if split && let Some(l) = crate::urban_bridge::stretch_layout_in(&m.stretch, ctx, rules) {
            // `WorldGen::town_grid`: the layout's market as a grid point of its settlement's cell.
            let t = cartalith_civ::town_freeze::to_town(&l, l.market);
            return (s.placement.x as f64 + 0.5 + t.x / mpc.max(1e-9), s.placement.y as f64 + 0.5 + t.y / mpc.max(1e-9));
        }
        cand.first().copied().unwrap_or(at)
    }) {
        out[i] = Some(p);
    }
    for (o, s) in out.iter_mut().zip(&m.settlements) {
        o.get_or_insert((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5));
    }
    out
}

/// What [`origin_in`] reads (Ruling HU: from the live object, `WorldGen::town_origin_now`, or a [`MarketSnap`]).
pub(crate) struct OriginIn<'a> {
    pub field: &'a [f32],
    pub water_bodies: &'a [u8],
    pub ways: &'a [cartalith_civ::Way],
    pub drawn: Option<&'a crate::town_water::DrawnWater>,
    pub geom: &'a RiverGeometry,
    pub index: &'a StrokeIndex,
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub map_width_km: f64,
}

/// **Ruling HM: where settlement `s`'s town is laid out** (`WorldGen::town_origin`'s doc), computed afresh.
pub(crate) fn origin_in(w: &OriginIn, s: &cartalith_civ::NamedSettlement) -> Option<cartalith_civ::urban_adapter::BankOrigin> {
    let (gw, gh) = (w.gw, w.gh);
    let cell_m = w.map_width_km * 1000.0 / gw.max(1) as f64;
    if !(cell_m > 0.0) {
        return None;
    }
    // Ruling HN: and not where the map DRAWS lake or sea (`town_water::DrawnWater`, the traced frame less a
    // half): the cell test alone called a point dry that the smooth shore puts in the sea, and a capital at a
    // river's mouth was stood in it and not laid out (seed 483920 #4 at 2 048 x 1 536).
    let dry = |x: f64, y: f64| {
        x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
            let k = y as usize * gw + x as usize;
            f64::from(w.field[k]) >= w.sea_level && w.water_bodies.get(k).is_none_or(|&c| c == 0) && w.drawn.is_none_or(|d| d.class_at(w.field, x - 0.5, y - 0.5) == crate::town_water::LAND)
        }
    };
    let c = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
    let roads = road_points(w.ways, (s.placement.x as f64, s.placement.y as f64), c, gw, w.map_width_km, origin_road_reach_m(w.geom, cell_m));
    let (p, river_cells) = origin_point(w.geom, w.index, &dry, cell_m, c, &roads)?;
    // The adapter's frame puts cell `c` at `c`; the traced frame at `c + 0.5`.
    Some(cartalith_civ::urban_adapter::BankOrigin { at: (p.0 - 0.5, p.1 - 0.5), river_m: river_cells * cell_m })
}

#[godot_api(secondary)]
impl WorldGen {
    /// The index of the drawn river for [`bank_point`], built once per drawn
    /// network (cached on `river_network_key_str`, like the geometry itself).
    fn bank_index(&self) -> Option<(std::sync::Arc<RiverGeometry>, std::sync::Arc<StrokeIndex>)> {
        let geom = self.river_geometry_any()?;
        let key = self.river_network_key_str();
        if let Some((k, ix)) = self.bank_index_cache.borrow().as_ref()
            && *k == key
        {
            return Some((geom, ix.clone()));
        }
        let ix = std::sync::Arc::new(StrokeIndex::new(&geom));
        *self.bank_index_cache.borrow_mut() = Some((key, ix.clone()));
        Some((geom, ix))
    }

    /// Every settlement's bank point at `ppc` pixels per cell, `gap_px` pixels
    /// clear of the drawn edge, whatever the switch says: `None` per settlement
    /// that stays on its cell centre. `None` with no world, a non-positive
    /// density or no drawn network.
    fn bank_points_at(&self, ppc: f32, gap_px: f64) -> Option<Vec<Option<(f64, f64)>>> {
        let (Some(civ), Some(crate::WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else { return None };
        if !(ppc > 0.0) {
            return None;
        }
        let (geom, index) = self.bank_index()?;
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let a = self.appearance();
        // Ruling HN: and not where the map DRAWS lake or sea (`town_water::DrawnWater`, the traced frame less a
        // half): the cell test alone called a point dry that the smooth shore puts in the sea, and a capital at a
        // river's mouth was stood in it and not laid out (seed 483920 #4 at 2 048 x 1 536).
        let drawn = self.town_water();
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0) && drawn.as_ref().is_none_or(|w| w.class_at(&ws.field, x - 0.5, y - 0.5) == crate::town_water::LAND)
            }
        };
        let q = BankQuery { geom: &geom, index: &index, a: &a, ppc, gap: gap_px.max(0.0) / f64::from(ppc), dry: &dry, wide: widest_half(&geom, &a), ground: None, reach_widths: None, same_bank: true };
        // Ruling HM: each marker on its market's bank (`market_points`). Ruling HU: off the main thread when the shell
        // asked for that (`set_markers_async`); a probe's census reads the exact table.
        let markets = if self.markers_async.get() { self.market_points_marker() } else { self.market_points() };
        Some(civ.settlements.iter().zip(markets).map(|(s, m)| bank_point(&q, (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), m, Some(MARKET_FIRST_BANK))).collect())
    }

    /// **Ruling HM: where settlement `s`'s town is laid out**, when that is
    /// not its cell ([`origin_point`]): where its road reaches the bank
    /// (Ruling HN), else the nearest dry point clear of every drawn river at
    /// its width on the ground ([`BankQuery::ground`], never nearer the
    /// centreline than [`ORIGIN_MIN_CLEAR_M`]) by [`ORIGIN_BANK_GAP_M`], on
    /// the bank its roads are on ([`roads_bank`]), else the bank the cell
    /// centre is on ([`MARKET_FIRST_BANK`] for a centre on the line). "Dry"
    /// is land by the cell **and** by the map's drawn lake and sea
    /// (`town_water::DrawnWater`, Ruling HN). `None` for a settlement already
    /// clear (its town stays round its cell), with the `civ.bank_sites`
    /// switch off, with no drawn network (a loaded save), and where no dry
    /// clear point is in reach.
    ///
    /// Must depend on nothing a view can change (zoom, window, look): a
    /// layout is cached, hashed and frozen in a save. **The river look's
    /// width slider in particular moves no town** (Ruling HN): the widths
    /// read are the geometry's on the ground, never the look's multiple
    /// (`_towncensus_probe.gd --slider` holds every town still while the
    /// markers move).
    ///
    /// One answer per settlement and world state ([`Self::origin_key`]).
    pub(crate) fn town_origin(&self, s: &cartalith_civ::NamedSettlement) -> Option<cartalith_civ::urban_adapter::BankOrigin> {
        if !self.params.civ.bank_sites {
            return None;
        }
        // Ruling HN leftover (f): one answer per settlement and world state. Every layout context, every
        // `urban_layout_boxes` call and every `town_at` asks for each settlement's.
        let key = self.origin_key();
        let id = (s.tid, s.placement.x, s.placement.y);
        {
            let mut c = self.origin_cache.borrow_mut();
            if c.0 != key {
                *c = (key, std::collections::HashMap::new());
            }
            if let Some(o) = c.1.get(&id) {
                return *o;
            }
        }
        let o = self.town_origin_now(s);
        self.origin_cache.borrow_mut().1.insert(id, o);
        o
    }

    /// One hash of everything [`Self::town_origin_now`] reads: the drawn rivers
    /// (`river_network_key_str`), the terrain and sea level its dry test reads
    /// (`drawn_water_key`; `CivData::water_bodies` is built from the same
    /// two), the map width, and the settlements' cells and the ways
    /// ([`Self::civ_shape_hash`]). The switch is tested before the cache.
    /// In-process only.
    fn origin_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.river_network_key_str().hash(&mut h);
        self.drawn_water_key().hash(&mut h);
        self.map_width_km.to_bits().hash(&mut h);
        self.civ_shape_hash(&mut h);
        h.finish()
    }

    /// Feeds `h` where the settlements are and every way a town's roads are read from (`um_primary_paths`: the
    /// shown ways' points), for the caches keyed on them ([`Self::origin_key`], [`Self::market_key`]).
    fn civ_shape_hash(&self, h: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        let Some(civ) = self.civ.as_ref() else { return };
        for s in &civ.settlements {
            (s.tid, s.placement.x, s.placement.y).hash(h);
        }
        for w in &civ.ways {
            (w.hidden, w.pts.len()).hash(h);
            for p in &w.pts {
                (p.0.to_bits(), p.1.to_bits()).hash(h);
            }
        }
    }

    /// [`Self::town_origin`] computed afresh (no cache; the switch is the caller's test): [`origin_in`] on the live
    /// world.
    fn town_origin_now(&self, s: &cartalith_civ::NamedSettlement) -> Option<cartalith_civ::urban_adapter::BankOrigin> {
        let (Some(civ), Some(crate::WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else { return None };
        let (geom, index) = self.bank_index()?;
        let drawn = self.town_water();
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        origin_in(&OriginIn { field: &ws.field, water_bodies: &civ.water_bodies, ways: &civ.ways, drawn: drawn.as_deref(), geom: &geom, index: &index, gw, gh, sea_level: self.sea_level, map_width_km: self.map_width_km }, s)
    }

    /// **Where each settlement's market stands on the map** (traced frame,
    /// cells), in `get_settlements()` order, for the marker to follow
    /// (Ruling HM; made the laid-out market's own bank by Ruling HN,
    /// 2026-11-23):
    ///
    /// - a frozen town's own market (its saved layout, which is what the City
    ///   Viewer shows whatever the rules have become);
    /// - else the market the town's layout puts down, from the generator's
    ///   own market code without laying the town out
    ///   (`urban_adapter::um_market_offsets` round the town's point,
    ///   [`Self::town_origin`] or the cell centre). Where the layouts the
    ///   town could be given put their markets on different banks of a drawn
    ///   river (a housing comparison decides which is kept), the town is laid
    ///   out and its market read;
    /// - the cell centre where no context can be built (a loaded save).
    ///
    /// Until Ruling HN this was the town's point, and a layout that moved its
    /// market over the river by a rule of its own (a street village's
    /// landing, its roads' bank) left the marker across from it: 8 census
    /// rows on 6 towns of about 1 230.
    ///
    /// Cached under [`Self::market_key`]: it builds every settlement's layout
    /// context, and the marker asks on each zoom step.
    fn market_points(&self) -> Vec<Option<(f64, f64)>> {
        let key = self.market_key();
        if let Some((k, v)) = self.market_cache.borrow().as_ref()
            && *k == key
        {
            return v.clone();
        }
        let Some(snap) = self.market_snap() else { return self.market_frozen_or_cell() };
        let t0 = std::time::Instant::now();
        let out = market_table(&snap);
        self.market_landed(key, out.clone(), t0.elapsed().as_secs_f64() * 1000.0, godot::sys::is_main_thread());
        out
    }

    /// Stores a built markers' table under `key` and counts where it was built (`market_points_stats`).
    fn market_landed(&self, key: u64, table: Vec<Option<(f64, f64)>>, ms: f64, on_main: bool) {
        *self.market_cache.borrow_mut() = Some((key, table));
        let (main, off, _) = self.market_builds.get();
        self.market_builds.set(if on_main { (main + 1, off, ms) } else { (main, off + 1, ms) });
    }

    /// **Ruling HU: the markers' table for the map's markers, never built on the calling thread.** The table for the
    /// world as it stands when it is ready ([`Self::market_points`]'s cache); else the job building it is started, or
    /// left running ([`MarketJob`], one at a time: a key that moves while it runs is taken up when it lands, so a
    /// sea-level drag never stacks threads), and the last table is answered meanwhile, so
    /// a marker stays where it was drawn until the new table lands ([`Self::market_points_poll`], which the shell
    /// calls until it does, and then re-asks). A last table for another roster (a different count) is not answered:
    /// every marker then stands at its cell's bank point until the job lands. Before Ruling HU a live sea-level move,
    /// a sculpt, an open and a parameter write each rebuilt the table here, on the main thread (about 0.8 s at 236
    /// settlements, Ruling HQ's `marker_cost.txt`). The labels (`settlement_points`) still build it in place: a label
    /// is placed once and stored, so it must read the exact table.
    fn market_points_marker(&self) -> Vec<Option<(f64, f64)>> {
        let key = self.market_key();
        let n = self.civ.as_ref().map_or(0, |c| c.settlements.len());
        if let Some((k, v)) = self.market_cache.borrow().as_ref()
            && *k == key
        {
            return v.clone();
        }
        // One job at a time: a key that moves while one runs (a sea-level drag) is taken up when it lands
        // (`market_points_poll` starts the next for the world as it then stands).
        if self.market_job.borrow().is_none() {
            self.market_job_start(key);
        }
        match self.market_cache.borrow().as_ref() {
            Some((_, v)) if v.len() == n => v.clone(),
            _ => vec![None; n],
        }
    }

    /// **Ruling HU: the map's markers read their table without building it on the calling thread**
    /// ([`Self::market_points_marker`]). `EngineBridge.settlement_bank_offsets` sets it on every call: on for the live
    /// map's markers, off for every other caller (an export copy, a probe), which gets the exact table for the world
    /// as it stands, built in place if need be. Off by default, so a `WorldGen` no shell drives is always exact.
    #[func]
    fn set_markers_async(&self, on: bool) {
        self.markers_async.set(on);
    }

    /// Starts the markers' job for `key` on its own thread from a snapshot of the world ([`MarketSnap`]); a job for
    /// an older key is dropped (its thread finishes and its table is discarded). Nothing with no world.
    fn market_job_start(&self, key: u64) {
        let Some(snap) = self.market_snap() else {
            // No town context to build (a world without its substrate): the table is answered in place, at once.
            *self.market_job.borrow_mut() = None;
            *self.market_cache.borrow_mut() = Some((key, self.market_frozen_or_cell()));
            return;
        };
        let handle = std::thread::spawn(move || {
            let t0 = std::time::Instant::now();
            let table = market_table(&snap);
            (table, t0.elapsed().as_secs_f64() * 1000.0)
        });
        *self.market_job.borrow_mut() = Some(MarketJob { key, handle });
        self.market_jobs.set(self.market_jobs.get() + 1);
    }

    /// **Ruling HU: lands the markers' job when it has finished.** `1` when a table landed for the world as it stands
    /// (the shell then asks for the markers again), `0` while the job runs, `2` with no job. A job finished for a
    /// world that has since changed is discarded and one is started for the new state (`0`), so a table is never
    /// stored under a key it was not built for. Main thread only, like every call on this object.
    #[func]
    fn market_points_poll(&self) -> i64 {
        let done = match self.market_job.borrow().as_ref() {
            None => return 2,
            Some(j) => j.handle.is_finished(),
        };
        if !done {
            return 0;
        }
        let Some(job) = self.market_job.borrow_mut().take() else { return 2 };
        let Ok((table, ms)) = job.handle.join() else { return 2 };
        let key = self.market_key();
        // A table already built in place for the world as it stands (the warm-up, a label, an exact caller) stands.
        if self.market_cache.borrow().as_ref().is_some_and(|(k, _)| *k == key) {
            return 2;
        }
        if job.key == key {
            self.market_landed(job.key, table, ms, false);
            return 1;
        }
        // The world moved on while it ran: the job starts again for it.
        self.market_job_start(key);
        0
    }

    /// A frozen town's own market per settlement (its saved layout), `None` for a town not frozen.
    fn market_frozen(&self) -> Vec<Option<(f64, f64)>> {
        let Some(civ) = self.civ.as_ref() else { return Vec::new() };
        civ.settlements
            .iter()
            .map(|s| self.town_layouts.rows.get(&s.tid).and_then(|r| r.layout.as_ref()).map(|l| self.town_grid(l, s.placement.x, s.placement.y, l.market)))
            .collect()
    }

    /// The table where no town context can be built (a world without its substrate): a frozen town's market, else
    /// the cell centre.
    fn market_frozen_or_cell(&self) -> Vec<Option<(f64, f64)>> {
        let Some(civ) = self.civ.as_ref() else { return Vec::new() };
        self.market_frozen().into_iter().zip(&civ.settlements).map(|(f, s)| Some(f.unwrap_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5)))).collect()
    }

    /// Everything [`market_table`] reads, copied or shared out of the live object (Ruling HU), so the table can be
    /// built on another thread while the main thread goes on drawing. `None` before a world or without its
    /// substrate. The large rasters are shared (`Arc`) or copied once (the stream order, the receivers, the water
    /// bodies: a few megabytes at 2 048 cells); the ore rasters only when a mining town needs them, as
    /// `town_batch` builds them.
    fn market_snap(&self) -> Option<MarketSnap> {
        let (Some(civ), Some(crate::WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else { return None };
        let frozen = self.market_frozen();
        let wants_ore = civ.settlements.iter().zip(&frozen).any(|(s, f)| f.is_none() && civ.place_extras.get(s.tid).specialisation == "mining");
        Some(MarketSnap {
            field: ws.field.clone(),
            flow: ws.flow_discharge.clone(),
            order: ws.stream_order.clone(),
            recv: ws.channels.as_ref().map(|c| c.recv.clone()),
            water_bodies: civ.water_bodies.clone(),
            settlements: civ.settlements.clone(),
            ways: civ.ways.clone(),
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            seed: self.seed,
            params: self.params.clone(),
            map_water: self.town_map_water(),
            town_water: self.town_water(),
            resources: self.ore_potentials(ws, civ, wants_ore),
            drawn: self.drawn_river_runs(),
            extras: civ.settlements.iter().map(|s| (s.tid, civ.place_extras.get(s.tid))).collect(),
            faith: civ.settlements.iter().map(|s| (s.tid, civ.town_faith(s))).collect(),
            urban_rules: self.urban_rules,
            bank: self.bank_index(),
            stretch: self.stretch_choices.borrow().clone(),
            frozen,
        })
    }

    /// **Ruling HQ: where the markers' table was built** (`market_points`, the 0.75 s of Ruling HN's leftover
    /// (f)): `{main, worker, last_ms, jobs}`, the builds on the main thread and off it (a job's table counts when it
    /// lands) since this object was made, the last one's milliseconds, and the markers' jobs started (Ruling HU). Diagnostic for a probe (`_towncensus_probe.gd --time`); nothing in the shell
    /// reads it.
    #[func]
    fn market_points_stats(&self) -> VarDictionary {
        let (main, off, ms) = self.market_builds.get();
        vdict! { "main" => main as i64, "worker" => off as i64, "last_ms" => ms, "jobs" => i64::from(self.market_jobs.get()) }
    }

    /// One hash of everything [`Self::market_points`] reads, listed from what
    /// `town_batch` hands a layout: the terrain, sea level and drawn lakes
    /// (`drawn_water_key`), the drawn rivers (`river_network_key_str`), every
    /// parameter (the `urban_*` and `civ.*` switches among them), the world's
    /// active rules, seed and width, each settlement's cell, people, kind,
    /// faction's rite and place-editor row, every way, and the frozen towns'
    /// revision. In-process only (never stored), so the hasher need not be
    /// stable across builds. Must grow with `town_batch`'s inputs.
    fn market_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.drawn_water_key().hash(&mut h);
        self.river_network_key_str().hash(&mut h);
        format!("{:?}{:?}", self.params, self.urban_rules).hash(&mut h);
        (self.seed, self.map_width_km.to_bits(), self.town_rev).hash(&mut h);
        if let Some(civ) = self.civ.as_ref() {
            for s in &civ.settlements {
                (s.placement.x, s.placement.y, s.pop, s.tid, civ.town_faith(s)).hash(&mut h);
                format!("{:?}{:?}", s.placement.kind, civ.place_extras.get(s.tid)).hash(&mut h);
            }
        }
        self.civ_shape_hash(&mut h);
        h.finish()
    }

    /// **Ruling HM: where each settlement stands on the map for an
    /// engine-placed label** (`label_bridge`'s generated settlement and region
    /// labels), as a point per settlement: [`Self::market_points`]. Empty
    /// with the `civ.bank_sites` switch off, so those labels are anchored at
    /// the cell centre exactly as before the ruling. A label is placed once
    /// and stored, so it reads the town's point, which no view changes, not
    /// the marker's, which moves with the zoom.
    pub(crate) fn settlement_points(&self) -> Vec<(f64, f64)> {
        if !self.params.civ.bank_sites {
            return Vec::new();
        }
        let Some(civ) = self.civ.as_ref() else { return Vec::new() };
        self.market_points().into_iter().zip(&civ.settlements).map(|(m, s)| m.unwrap_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5))).collect()
    }

    /// **The Ruling HM and HN census** (a measurement; `_towncensus_probe.gd`
    /// prints it), here because the drawn river and the app's own layout
    /// context exist only on a live `WorldGen`. One row per settlement of
    /// `indices`, laid out exactly as `urban_layouts` lays it out under the
    /// switches in force, measured against the strokes a `view_w` x `view_h`
    /// map view draws at x1 and x4:
    ///
    /// - `i`; `laid` (a layout came back); `origin_m` (how far the box centre
    ///   is from the cell, metres) and `river_m` ([`Self::town_origin`]; both
    ///   absent for a town round its cell); `wm`, `hm` (the box);
    /// - `river` (the layout has a river line of two points or more);
    ///   `river_w`; `market_river_m`, the market's distance to that line
    ///   (absent with no river): the market is in the City Viewer's river
    ///   when it is under half `river_w`;
    /// - `wet` (`town_measures::wet_buildings`), `fronted`, `reach`
    ///   (`town_measures::bank_frontage`'s fronted and built-reach samples),
    ///   `pop`, `pop_target`, `kind` (the layout's site kind), `market_prov`;
    /// - the footprint over water (Ruling HN, `town_measures::footprint_over`,
    ///   lines sampled every 4 m): `over_b`, `over_w`, `over_s` are the
    ///   buildings, the wall-ring runs with no water gate and the street
    ///   edges that are no part of a bridge, standing in a river stroke the
    ///   map draws at its ground width (the default look's;
    ///   [`BankQuery::ground`] with no minimum); `gated_w` and `bridge_s` the
    ///   gated runs and bridge edges, `span_m` the longest bridge; `of_b`,
    ///   `of_w`, `of_s` how many were tested; `s_by` the classes of the
    ///   streets counted and `depth` how deep the deepest of each kind is.
    ///   `lake_over_b/w/s` and `sea_over_b/w/s` are the same over the map's
    ///   drawn lake and sea (`town_water::DrawnWater`), with no exemption;
    /// - `why`, for a settlement with no layout;
    /// - `mx`, `my`: the market on the map, cells from the cell centre;
    /// - `side_x1`, `side_x4`, `side_x16`, `side_x64`: `1` the marker and the
    ///   market stand on one bank, `-1` the centreline of a river drawn at that zoom runs
    ///   between them ([`StrokeIndex::crosses`]); absent with no drawn river
    ///   within two cells of the cell.
    #[func]
    fn town_bank_census(&self, indices: PackedInt32Array, view_w: f64, view_h: f64) -> Array<VarDictionary> {
        use crate::town_measures::{bank_frontage, footprint_over, laid_ctx, line_dist, wet_buildings};
        // The zooms the marker's side is read at, times the opening fit.
        const ZOOMS: [f32; 4] = [1.0, 4.0, 16.0, 64.0];
        let mut out = Array::new();
        let water = self.town_water();
        let field: &[f32] = match self.source.as_ref() {
            Some(crate::WorldSource::Generated(ws)) => &ws.field,
            _ => &[],
        };
        let Some((geom, index)) = self.bank_index() else { return out };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let fit = crate::river_stroke::fit_ppc_in((view_w as f32, view_h as f32), gw, gh);
        let marks: Vec<PackedVector2Array> = {
            let _law = crate::river_stroke::ScaleGuard::hold(Some((fit, crate::river_stroke::start_share(self.map_width_km as f32))));
            ZOOMS.iter().map(|&z| self.settlement_bank_offsets(f64::from(fit * z), -1.0)).collect()
        };
        let idx: Vec<usize> = indices.as_slice().iter().filter_map(|&i| usize::try_from(i).ok()).collect();
        let mpc = self.map_width_km * 1000.0 / gw.max(1) as f64;
        // The map's strokes at their ground width, for `over` (the default look's `river_width` is 1).
        let a = TerrainAppearance::default();
        let look = self.appearance();
        let strokes = BankQuery { geom: &geom, index: &index, a: &a, ppc: 1.0, gap: 0.0, dry: &|_, _| true, wide: widest_ground_half(&geom), ground: Some(0.0), reach_widths: None, same_bank: false };
        for (i, row) in self.town_batch(&idx, |i, s, ctx, rules, o, _| {
            let mut d = vdict! { "i" => i as i64, "laid" => false };
            if let Some(b) = o.origin {
                d.set("origin_m", (b.at.0 - s.placement.x as f64).hypot(b.at.1 - s.placement.y as f64) * mpc);
                d.set("river_m", b.river_m);
            }
            let Some(l) = self.stretch_layout(ctx, rules) else {
                // Why not: the layout refuses a settlement whose centre stands in water.
                d.set("why", if ctx.water.as_ref().is_some_and(|w| w.mostly_water) { "its centre is in the water" } else { "no layout" });
                return d;
            };
            d.set("laid", true);
            d.set("wm", l.wm);
            d.set("hm", l.hm);
            let rv: Vec<(f64, f64)> = l.river.iter().map(|v| (v.x, v.y)).collect();
            d.set("river", rv.len() >= 2);
            d.set("river_w", l.river_w);
            if let Some(m) = line_dist((l.market.x, l.market.y), &rv) {
                d.set("market_river_m", m);
            }
            let open = laid_ctx(ctx, &l).water.as_ref().and_then(|w| w.ctx.open_water.as_deref().map(|o| (o, w.ctx.mw, w.ctx.mh, w.ctx.cell_m)));
            d.set("wet", wet_buildings(&l, open) as i64);
            let (f, _, r) = bank_frontage(&l, &rv);
            d.set("fronted", f as i64);
            d.set("reach", r as i64);
            d.set("pop", l.pop);
            d.set("pop_target", l.pop_target);
            // Ruling HN: the footprint against the map's drawn river, lake and sea (`footprint_over`).
            let grid = |p: (f64, f64)| self.town_grid(&l, s.placement.x, s.placement.y, cartalith_civ::urban_adapter::Vec2::new(p.0, p.1));
            let river_o = footprint_over(&l, &|p| strokes.covering(grid(p)).map(|(hit, half)| (half - hit.d) * mpc), true);
            // The tiles' sample frame is the traced frame less a half (`town_water`'s module doc).
            let class = |p: (f64, f64)| {
                let g = grid(p);
                water.as_ref().map_or(crate::town_water::LAND, |w| w.class_at(field, g.0 - 0.5, g.1 - 0.5))
            };
            let lake_o = footprint_over(&l, &|p| (class(p) == crate::town_water::LAKE).then_some(0.0), false);
            let sea_o = footprint_over(&l, &|p| (class(p) == crate::town_water::SEA).then_some(0.0), false);
            for (k, o) in [("", &river_o), ("lake_", &lake_o), ("sea_", &sea_o)] {
                d.set(format!("{k}over_b"), o.b as i64);
                d.set(format!("{k}over_w"), o.w as i64);
                d.set(format!("{k}over_s"), o.s as i64);
                d.set(format!("{k}s_by"), o.s_by.as_str());
            }
            d.set("of_b", river_o.of_b as i64);
            d.set("of_w", river_o.of_w as i64);
            d.set("of_s", river_o.of_s as i64);
            d.set("gated_w", river_o.w_gated as i64);
            d.set("bridge_s", river_o.s_bridge as i64);
            d.set("span_m", river_o.span_m);
            d.set("wet_span_m", river_o.wet_span_m);
            // Ruling HQ: the ferries the layout put where a bridge would be longer than
            // `cleanup::BRIDGE_MAX_SPAN_M`, and the longest of them (landing to landing).
            let ferries: Vec<f64> = l.details.iter().filter(|d| d.kind == "ferry").filter_map(|d| match d.geom {
                cartalith_civ::urban_adapter::DetailGeom::Seg(a, b) => Some(a.dist(b)),
                _ => None,
            }).collect();
            d.set("ferries", ferries.len() as i64);
            d.set("ferry_m", ferries.iter().copied().fold(0.0, f64::max));
            // Ruling HU: and the shortest, for the census's gate that a ferry spans at least half its river.
            if !ferries.is_empty() {
                d.set("ferry_min_m", ferries.iter().copied().fold(f64::INFINITY, f64::min));
            }
            // Ruling HU: how far the deepest ferry landing stands inside the map's drawn river, metres (0 on its bank),
            // and how many landings stand in the drawn lake or sea: a landing is the bank's, never the water's.
            let landings: Vec<(f64, f64)> = l.details.iter().filter(|d| d.kind == "ferry").filter_map(|d| match d.geom {
                cartalith_civ::urban_adapter::DetailGeom::Seg(a, b) => Some([(a.x, a.y), (b.x, b.y)]),
                _ => None,
            }).flatten().collect();
            if !landings.is_empty() {
                let deep = landings.iter().filter_map(|&p| strokes.covering(grid(p)).map(|(hit, half)| (half - hit.d) * mpc)).fold(0.0, f64::max);
                d.set("ferry_wet_m", deep);
                d.set("ferry_open", landings.iter().filter(|&&p| class(p) != crate::town_water::LAND).count() as i64);
            }
            // Ruling HQ (HN's survivor U18): live streets with a point within `MARKET_STREET_NEAR_M` of the market.
            d.set("mkt_s", crate::town_measures::streets_near(&l, l.market, crate::town_measures::MARKET_STREET_NEAR_M) as i64);
            d.set("depth", format!("b {:.0} w {:.0} (run {:.0}) s {:.0} m", river_o.depth_m[0], river_o.depth_m[1], river_o.w_run_m, river_o.depth_m[2]).as_str());
            d.set("market_prov", l.market_prov);
            d.set("kind", l.site_kind.as_str());
            let c = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
            let m = self.town_grid(&l, s.placement.x, s.placement.y, l.market);
            d.set("mx", m.0 - c.0);
            d.set("my", m.1 - c.1);
            if index.nearest(&geom, c, 2.0).is_some() {
                for (k, name) in ["x1", "x4", "x16", "x64"].iter().enumerate() {
                    let off = marks[k].get(i).unwrap_or(Vector2::ZERO);
                    let p = (c.0 + f64::from(off.x), c.1 + f64::from(off.y));
                    let ppc = fit * ZOOMS[k];
                    let _law = crate::river_stroke::ScaleGuard::hold(Some((fit, crate::river_stroke::start_share(self.map_width_km as f32))));
                    d.set(format!("side_{name}"), if index.crosses(&geom, p, m, &|pt| half_width_cells(pt, ppc, &look).is_some()) { -1i64 } else { 1i64 });
                }
            }
            d
        }) {
            let _ = i;
            out.push(&row);
        }
        out
    }

    /// **A census aid (Ruling HN): one town and the map's water under it**, as JSON for a plot
    /// (`docs/evidence/town-on-bank-hn-2026-11-23/plot_town.py`): the town of settlement `index` laid out as
    /// `urban_layouts` lays it out, in its own metres (`wm`, `hm`, `market`, `pin`, `river`, `river_w`,
    /// `buildings`, `edges` as `[ax, ay, bx, by, class]`, `ring`, `gates` as `[x, y, water]`, `bridges`), and
    /// `grid`: what the map draws at each point of a `step_m` lattice over the box, row-major, `gw` by `gh`
    /// characters: `0` land, `1` sea, `2` lake, `3` a river stroke at its ground width, `4` the town's own water
    /// raster where the map draws land. Empty for an index with no layout. A measurement; nothing reads it back.
    #[func]
    fn town_debug_json(&self, index: i64, step_m: f64) -> GString {
        let Ok(idx) = usize::try_from(index) else { return GString::new() };
        let Some((geom, bank)) = self.bank_index() else { return GString::new() };
        let water = self.town_water();
        let field: &[f32] = match self.source.as_ref() {
            Some(crate::WorldSource::Generated(ws)) => &ws.field,
            _ => &[],
        };
        let a = TerrainAppearance::default();
        let strokes = BankQuery { geom: &geom, index: &bank, a: &a, ppc: 1.0, gap: 0.0, dry: &|_, _| true, wide: widest_ground_half(&geom), ground: Some(0.0), reach_widths: None, same_bank: false };
        let step = step_m.max(1.0);
        self.town_batch(&[idx], |_, s, ctx, rules, o, _| {
            let Some(l) = self.stretch_layout(ctx, rules) else {
                // A settlement the layout refuses: the map's water and the town's own raster over its box, no town.
                let w = ctx.water.as_ref()?;
                let (wm, hm) = ctx.site_box.unwrap_or((cartalith_civ::urban_adapter::SITE_WM, cartalith_civ::urban_adapter::SITE_HM));
                let at = o.origin.map_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), |b| (b.at.0 + 0.5, b.at.1 + 0.5));
                let mpc = self.map_width_km * 1000.0 / f64::from(self.gw.max(1));
                let (gw, gh) = ((wm / step).ceil() as usize, (hm / step).ceil() as usize);
                let mut grid = String::with_capacity(gw * gh);
                for j in 0..gh {
                    for i in 0..gw {
                        let (x, y) = ((i as f64 + 0.5) * step, (j as f64 + 0.5) * step);
                        let g = (at.0 + (x - wm / 2.0) / mpc, at.1 + (y - hm / 2.0) / mpc);
                        let (ci, cj) = ((x / w.ctx.cell_m) as usize, (y / w.ctx.cell_m) as usize);
                        let own = ci < w.ctx.mw && cj < w.ctx.mh && w.ctx.mask[cj * w.ctx.mw + ci] != 0;
                        grid.push(if strokes.covering(g).is_some() {
                            '3'
                        } else {
                            match water.as_ref().map_or(crate::town_water::LAND, |dw| dw.class_at(field, g.0 - 0.5, g.1 - 0.5)) {
                                crate::town_water::SEA => '1',
                                crate::town_water::LAKE => '2',
                                _ if own => '4',
                                _ => '0',
                            }
                        });
                    }
                }
                let river: Vec<_> = w.ctx.river_path.iter().flatten().map(|v| serde_json::json!([v.x, v.y])).collect();
                return Some(serde_json::json!({ "wm": wm, "hm": hm, "step": step, "gw": gw, "gh": gh, "grid": grid, "market": [wm / 2.0, hm / 2.0], "pin": [wm / 2.0, hm / 2.0],
                    "river_w": w.ctx.river_width_m, "site_kind": "NOT LAID OUT", "pop": 0, "pop_target": 0, "river": river, "buildings": [], "edges": [], "ring": [], "gates": [], "bridges": [], "over": [] }));
            };
            let open = crate::town_measures::laid_ctx(ctx, &l).water.as_ref().map(|w| (w.ctx.mask.clone(), w.ctx.mw, w.ctx.mh, w.ctx.cell_m));
            let (gw, gh) = ((l.wm / step).ceil() as usize, (l.hm / step).ceil() as usize);
            let mut grid = String::with_capacity(gw * gh);
            for j in 0..gh {
                for i in 0..gw {
                    let p = cartalith_civ::urban_adapter::Vec2::new((i as f64 + 0.5) * step, (j as f64 + 0.5) * step);
                    let g = self.town_grid(&l, s.placement.x, s.placement.y, p);
                    let own = open.as_ref().is_some_and(|(m, mw, mh, c)| {
                        let (ci, cj) = ((p.x / c) as usize, (p.y / c) as usize);
                        ci < *mw && cj < *mh && m[cj * mw + ci] != 0
                    });
                    let c = if strokes.covering(g).is_some() {
                        '3'
                    } else {
                        match water.as_ref().map_or(crate::town_water::LAND, |w| w.class_at(field, g.0 - 0.5, g.1 - 0.5)) {
                            crate::town_water::SEA => '1',
                            crate::town_water::LAKE => '2',
                            _ if own => '4',
                            _ => '0',
                        }
                    };
                    grid.push(c);
                }
            }
            let pt = |v: cartalith_civ::urban_adapter::Vec2| serde_json::json!([v.x, v.y]);
            // The buildings the census counts over a drawn river, by index (`footprint_over`'s own test).
            let over: Vec<usize> = (0..l.buildings.len())
                .filter(|&k| {
                    let poly: Vec<(f64, f64)> = l.buildings[k].poly.iter().map(|v| (v.x, v.y)).collect();
                    crate::town_measures::footprint_over_parts(&[poly], &[], &[], &[], &[], &|p| strokes.covering(self.town_grid(&l, s.placement.x, s.placement.y, cartalith_civ::urban_adapter::Vec2::new(p.0, p.1))).map(|_| 0.0), true).b > 0
                })
                .collect();
            Some(serde_json::json!({
                "over": over,
                "wm": l.wm, "hm": l.hm, "step": step, "gw": gw, "gh": gh, "grid": grid,
                "market": pt(l.market), "pin": pt(l.pin), "river_w": l.river_w, "site_kind": l.site_kind, "pop": l.pop, "pop_target": l.pop_target,
                "river": l.river.iter().map(|&v| pt(v)).collect::<Vec<_>>(),
                "buildings": l.buildings.iter().map(|b| b.poly.iter().map(|&v| pt(v)).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "edges": l.edges.iter().map(|e| serde_json::json!([e.a.x, e.a.y, e.b.x, e.b.y, e.cls])).collect::<Vec<_>>(),
                "ring": l.wall.ring.iter().flatten().map(|&v| pt(v)).collect::<Vec<_>>(),
                "gates": l.wall.gates.iter().map(|g| serde_json::json!([g.pt.x, g.pt.y, g.water])).collect::<Vec<_>>(),
                "bridges": l.bridges.iter().flatten().map(|b| pt(b.pt)).collect::<Vec<_>>(),
            }))
        })
        .into_iter()
        .next()
        .and_then(|(_, v)| v)
        .map_or_else(GString::new, |v| GString::from(&v.to_string()))
    }

    /// **Ruling HN (owner, 2026-11-23): whether to offer this world the
    /// update to towns on the bank**, as the key the shell records the answer
    /// under; empty when there is nothing to offer.
    ///
    /// Offered only when every one of these holds: the world was read from a
    /// save with no `civ.bank_sites` key (an explicit `false` is somebody's
    /// answer, and a new world has the switch on), the switch is still off,
    /// the world can lay towns out and has a drawn river for them to stand
    /// beside (a world opened with its substrate: a legacy flat archive has
    /// neither, so the switch would do nothing there), and it has a
    /// settlement.
    ///
    /// The key names the world, not the file, so a renamed or copied save is
    /// not asked twice: FNV-1a 64 over the seed, the grid, the map width, the
    /// sea level and every 61st height (`61`: a stride that samples a
    /// 384 x 288 grid 1 813 times and is prime, so no row or column is
    /// skipped; a labelled judgement). The answer itself is the shell's to
    /// keep (`DccSettings`): "no" writes no parameter here, so it is never
    /// taken as the user's rule for the next new world (`rules_chosen`,
    /// Ruling EV), and the save is not rewritten.
    #[func]
    fn bank_sites_offer(&self) -> GString {
        let (Some(civ), Some(crate::WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else { return GString::new() };
        if !self.bank_sites_absent || self.params.civ.bank_sites || civ.settlements.is_empty() || self.river_geometry_any().is_none() {
            return GString::new();
        }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        eat(self.seed as u64);
        eat(self.gw as u64);
        eat(self.gh as u64);
        eat(self.map_width_km.to_bits());
        eat(self.sea_level.to_bits());
        for v in ws.field.iter().step_by(61) {
            eat(u64::from(v.to_bits()));
        }
        GString::from(&format!("{h:016x}"))
    }

    /// **Ruling HG: where each settlement's marker stands**, as an offset in
    /// grid cells from its cell centre, in `get_settlements()` order, for a map
    /// drawn at `ppc` screen pixels per cell with the marker kept `gap_px`
    /// pixels off the drawn river's edge (negative: [`MARKER_GAP_PX`]). Zero
    /// for a settlement clear of every drawn river.
    ///
    /// **Empty** when the world's `civ.bank_sites` switch is off, before a
    /// world, and for a world with no drawn network (a loaded save): the
    /// caller then draws every marker at its cell centre, as before the
    /// ruling. It moves no settlement: `x` and `y` stay the cell.
    #[func]
    fn settlement_bank_offsets(&self, ppc: f64, gap_px: f64) -> PackedVector2Array {
        if !self.params.civ.bank_sites {
            return PackedVector2Array::new();
        }
        let gap = if gap_px < 0.0 { MARKER_GAP_PX } else { gap_px };
        let (Some(civ), Some(pts)) = (self.civ.as_ref(), self.bank_points_at(ppc as f32, gap)) else { return PackedVector2Array::new() };
        pts.iter()
            .zip(&civ.settlements)
            .map(|(p, s)| p.map_or(Vector2::ZERO, |p| Vector2::new((p.0 - (s.placement.x as f64 + 0.5)) as f32, (p.1 - (s.placement.y as f64 + 0.5)) as f32)))
            .collect()
    }

    /// **The Ruling HG census** (a measurement; `_bankcensus_probe.gd` prints
    /// it), here because the drawn river exists only on a live `WorldGen`.
    /// One row per settlement, in `get_settlements()` order, measured against
    /// the strokes a `view_w` x `view_h` pixel map view draws at x1 (the
    /// opening fit, `river_stroke::fit_ppc_in`) and at x4:
    ///
    /// - `i`, `kind`, `village` (the village `tid` flag: a seeded village or a
    ///   junction hamlet), `x`, `y`;
    /// - `d` to the nearest drawn centreline, `w` that stroke's full width
    ///   (`river_geometry_any` widths times the look's `river_width`), `rank`,
    ///   in cells; `nx`, `ny` the bank the centre is on: all **absent** with no
    ///   drawn river within two cells;
    /// - `in_w` (`d < w / 2`), `in_x1`, `in_x4` (inside ANY stroke drawn at
    ///   that zoom, [`BankQuery::covering`] with no gap);
    /// - `cell_x1`, `cell_x4`: whether one of the eight neighbour cells is
    ///   dry, clear of every stroke at that zoom, and within
    ///   `cartalith_civ::SUIT_RIVER_FRONT_M` of the river: could a whole-cell
    ///   move do it;
    /// - `shift_x1`, `shift_x4`: how far [`bank_point`] moves the marker at
    ///   that zoom with `gap_px`, cells (absent: it stays); `left_x1`,
    ///   `left_x4`: still inside a stroke after it.
    ///
    /// Empty before a world, and for a world with no drawn network.
    #[func]
    fn bank_census(&self, view_w: f64, view_h: f64, gap_px: f64) -> Array<VarDictionary> {
        let mut out = Array::new();
        let (Some(civ), Some(crate::WorldSource::Generated(ws)), Some((geom, index))) = (self.civ.as_ref(), self.source.as_ref(), self.bank_index()) else { return out };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let a = self.appearance();
        let fit = crate::river_stroke::fit_ppc_in((view_w as f32, view_h as f32), gw, gh);
        let _law = crate::river_stroke::ScaleGuard::hold(Some((fit, crate::river_stroke::start_share(self.map_width_km as f32))));
        let f = f64::from(crate::river_stroke::river_width_factor(&a));
        let on_cells = cartalith_civ::SUIT_RIVER_FRONT_M / (self.map_width_km * 1000.0 / gw.max(1) as f64);
        // Ruling HN: and not where the map DRAWS lake or sea (`town_water::DrawnWater`, the traced frame less a
        // half): the cell test alone called a point dry that the smooth shore puts in the sea, and a capital at a
        // river's mouth was stood in it and not laid out (seed 483920 #4 at 2 048 x 1 536).
        let drawn = self.town_water();
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0) && drawn.as_ref().is_none_or(|w| w.class_at(&ws.field, x - 0.5, y - 0.5) == crate::town_water::LAND)
            }
        };
        let wide = widest_half(&geom, &a);
        let at = |ppc: f32, gap_px: f64| BankQuery { geom: &geom, index: &index, a: &a, ppc, gap: gap_px / f64::from(ppc), dry: &dry, wide, ground: None, reach_widths: None, same_bank: false };
        let zooms = [("x1", fit), ("x4", fit * 4.0)];
        for (i, s) in civ.settlements.iter().enumerate() {
            let c = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
            let inside: Vec<bool> = zooms.iter().map(|&(_, ppc)| at(ppc, 0.0).covering(c).is_some()).collect();
            let mut d = vdict! {
                "i" => i as i64,
                "kind" => crate::journey_bridge::settlement_kind_key(s.placement.kind),
                "village" => civ.village_tids.contains(&s.tid),
                "x" => s.placement.x as i64,
                "y" => s.placement.y as i64,
                "in_w" => false,
            };
            if let Some(hit) = index.nearest(&geom, c, 2.0) {
                let w = f64::from(hit.point.width_cells) * f;
                d.set("d", hit.d);
                d.set("w", w);
                d.set("rank", f64::from(hit.point.rank));
                d.set("nx", hit.n.0);
                d.set("ny", hit.n.1);
                d.set("in_w", hit.d < w / 2.0);
            }
            for (k, &(name, ppc)) in zooms.iter().enumerate() {
                d.set(format!("in_{name}"), inside[k]);
                let bare = at(ppc, 0.0);
                let cell = (-1..=1i64).flat_map(|dy| (-1..=1i64).map(move |dx| (dx, dy))).filter(|&o| o != (0, 0)).any(|(dx, dy)| {
                    let n = (c.0 + dx as f64, c.1 + dy as f64);
                    dry(n.0, n.1) && bare.covering(n).is_none() && index.nearest(&geom, n, 2.0).is_some_and(|h| h.d <= on_cells)
                });
                d.set(format!("cell_{name}"), cell);
                if let Some(p) = bank_point(&at(ppc, gap_px), c, None, Some(MARKET_FIRST_BANK)) {
                    d.set(format!("shift_{name}"), (p.0 - c.0).hypot(p.1 - c.1));
                    d.set(format!("left_{name}"), bare.covering(p).is_some());
                } else {
                    d.set(format!("left_{name}"), inside[k]);
                }
            }
            out.push(&d);
        }
        out
    }

    /// The census for a hand drop with "Snap to water": for every `step`-th
    /// drawn river point, a click on the point's own cell and on the cells one
    /// and two away on either bank is snapped as `civ_drop_settlement` snaps it
    /// (`civ_tools_bridge::nearest_water_edge_cell`, the same radius and
    /// threshold) and the cell it lands on is measured like a settlement's in
    /// [`Self::bank_census`]: `clicks` (snapped clicks), `in_x1`, `in_x4`
    /// (its centre inside a drawn stroke), `left_x1`, `left_x4` (its
    /// [`bank_point`] at `gap_px` still inside one). Places nothing.
    #[func]
    fn bank_snap_census(&self, view_w: f64, view_h: f64, gap_px: f64, step: i64) -> VarDictionary {
        let mut out = VarDictionary::new();
        let (Some(civ), Some(crate::WorldSource::Generated(ws)), Some((geom, index))) = (self.civ.as_ref(), self.source.as_ref(), self.bank_index()) else { return out };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let a = self.appearance();
        let fit = crate::river_stroke::fit_ppc_in((view_w as f32, view_h as f32), gw, gh);
        let _law = crate::river_stroke::ScaleGuard::hold(Some((fit, crate::river_stroke::start_share(self.map_width_km as f32))));
        // Ruling HN: and not where the map DRAWS lake or sea (`town_water::DrawnWater`, the traced frame less a
        // half): the cell test alone called a point dry that the smooth shore puts in the sea, and a capital at a
        // river's mouth was stood in it and not laid out (seed 483920 #4 at 2 048 x 1 536).
        let drawn = self.town_water();
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0) && drawn.as_ref().is_none_or(|w| w.class_at(&ws.field, x - 0.5, y - 0.5) == crate::town_water::LAND)
            }
        };
        let wide = widest_half(&geom, &a);
        let at = |ppc: f32, gap_px: f64| BankQuery { geom: &geom, index: &index, a: &a, ppc, gap: gap_px / f64::from(ppc), dry: &dry, wide, ground: None, reach_widths: None, same_bank: false };
        let max_r = cartalith_civ::tools::civ_snap_radius(gw);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km);
        let mut clicks = 0i64;
        // Per zoom: inside at the centre, inside at the bank point.
        let mut n = [[0i64; 2]; 2];
        for run in &geom.runs {
            for w in run.pts.windows(2).step_by(step.max(1) as usize) {
                let (dx, dy) = (f64::from(w[1].0 - w[0].0), f64::from(w[1].1 - w[0].1));
                let l = dx.hypot(dy).max(1e-9);
                for off in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                    let (cx, cy) = ((f64::from(w[0].0) - dy / l * off).floor(), (f64::from(w[0].1) + dx / l * off).floor());
                    if cx < 0.0 || cy < 0.0 || cx as usize >= gw || cy as usize >= gh {
                        continue;
                    }
                    let Some((sx, sy)) = crate::civ_tools_bridge::nearest_water_edge_cell(cx as usize, cy as usize, gw, gh, &ws.field, &civ.water_bodies, &ws.flow_discharge, thresh, self.sea_level, max_r) else { continue };
                    clicks += 1;
                    let c = (sx as f64 + 0.5, sy as f64 + 0.5);
                    for (k, ppc) in [fit, fit * 4.0].into_iter().enumerate() {
                        let bare = at(ppc, 0.0);
                        n[k][0] += i64::from(bare.covering(c).is_some());
                        n[k][1] += i64::from(bare.covering(bank_point(&at(ppc, gap_px), c, None, Some(MARKET_FIRST_BANK)).unwrap_or(c)).is_some());
                    }
                }
            }
        }
        out.set("clicks", clicks);
        out.set("in_x1", n[0][0]);
        out.set("left_x1", n[0][1]);
        out.set("in_x4", n[1][0]);
        out.set("left_x4", n[1][1]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::river_stroke::{DrawnRun, ScaleGuard};

    /// A drawn run through `pts`, `width` cells wide all along, unranked (so
    /// drawn at every zoom: `DrawnRun::ranks` empty), one piece.
    fn run(pts: &[(f32, f32)], width: f32) -> DrawnRun {
        let n = pts.len();
        DrawnRun {
            pts: pts.to_vec(),
            widths: vec![width; n],
            colors: vec![[0.0, 0.0, 1.0, 1.0]; n],
            orders: vec![2; n],
            discharge: vec![1.0; n],
            ranks: Vec::new(),
            pieces: vec![(0, n)],
            reach: Vec::new(),
            own_order: 2,
            turbidity: Vec::new(),
            detail: Vec::new(),
        }
    }

    /// One river running west to east along `y = 10.5`, 0.4 cells wide.
    fn east_river() -> RiverGeometry {
        RiverGeometry { runs: vec![run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 0.4)] }
    }

    /// `bank_point` over `geom` at `ppc` with a `gap` in cells, all ground dry
    /// unless `dry` says otherwise, under a fixed zoom law.
    fn bank(geom: &RiverGeometry, ppc: f32, gap: f64, dry: &dyn Fn(f64, f64) -> bool, c: (f64, f64)) -> Option<(f64, f64)> {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let a = TerrainAppearance::default();
        let index = StrokeIndex::new(geom);
        bank_point(&BankQuery { geom, index: &index, a: &a, ppc, gap, dry, wide: widest_half(geom, &a), ground: None, reach_widths: None, same_bank: false }, c, None, Some(MARKET_FIRST_BANK))
    }

    /// Protects the rule itself: a centre inside the stroke moves straight out
    /// to the bank it is already on, to the stroke's edge plus the gap, and a
    /// centre already clear stays (`None`). The gap is 0.07 so the point is on
    /// no step of the ring search (steps of a quarter of the clearance): only
    /// the direct rule lands on it. MUTATION: `c = half + q.gap` to `half`, or
    /// `half + self.gap - hit.d` to `half - hit.d` in `covering` (the gap
    /// dropped), lands elsewhere than 10.77.
    #[test]
    fn a_centre_in_the_stroke_moves_to_its_own_bank() {
        let g = east_river();
        // 16 px per cell: the 0.4-cell stroke is 6.4 px wide, far over the one-pixel floor; half is 0.2 cells.
        let p = bank(&g, 16.0, 0.07, &|_, _| true, (8.5, 10.6)).expect("moved");
        assert!((p.0 - 8.5).abs() < 1e-9 && (p.1 - 10.77).abs() < 1e-6, "0.2 half + 0.07 gap south of y 10.5: {p:?}");
        let n = bank(&g, 16.0, 0.07, &|_, _| true, (8.5, 10.4)).expect("moved");
        assert!((n.1 - 10.23).abs() < 1e-6, "the north bank for a centre north of the line: {n:?}");
        assert_eq!(bank(&g, 16.0, 0.07, &|_, _| true, (8.5, 10.8)), None, "0.3 off the line is clear of 0.27");
        assert!(bank(&g, 16.0, 0.07, &|_, _| true, (8.5, 10.74)).is_some(), "0.24 off the line is clear of the water (0.2) and inside the gap: it moves");
    }

    /// Protects the one-pixel floor: at 1 px per cell the same 0.4-cell stroke
    /// is drawn one pixel wide (`river_stroke::MIN_STROKE_PX`), half a cell
    /// each side, so the marker stands 0.5 cells off the line, not 0.2.
    /// And the search for a covering stroke reaches as far as that floor: at
    /// an eighth of a pixel per cell the one-pixel stroke is four cells either
    /// side, so a centre 3.2 cells off the line is in it.
    /// MUTATION: reading the width without the floor (`width_cells / 2`) gives
    /// 10.7; `self.wide.max(floor)` to `self.wide` in `covering` (the floor
    /// left out of the reach) misses the far centre.
    #[test]
    fn the_edge_is_the_drawn_stroke_with_its_pixel_floor() {
        let g = east_river();
        let p = bank(&g, 1.0, 0.0, &|_, _| true, (8.5, 10.6)).expect("moved");
        assert!((p.1 - 11.0).abs() < 1e-6, "{p:?}");
        let far = bank(&g, 0.125, 0.0, &|_, _| true, (8.5, 13.7)).expect("3.2 cells off the line is inside a stroke four cells either side");
        assert!((far.1 - 14.5).abs() < 1e-6, "{far:?}");
    }

    /// Protects the tie-break: a centre on the centreline has no bank of its
    /// own and takes the north bank of an east-west river and the west bank of
    /// a north-south one ([`MARKET_FIRST_BANK`]). MUTATION: `MARKET_FIRST_BANK`
    /// to `(1.0, 0.13)` sends both the other way.
    #[test]
    fn a_centre_on_the_line_takes_the_markets_first_bank() {
        let p = bank(&east_river(), 16.0, 0.05, &|_, _| true, (8.5, 10.5)).expect("moved");
        assert!((p.1 - 10.25).abs() < 1e-6, "north: {p:?}");
        let south = RiverGeometry { runs: vec![run(&[(10.5, 2.5), (10.5, 6.5), (10.5, 10.5), (10.5, 14.5)], 0.4)] };
        let q = bank(&south, 16.0, 0.05, &|_, _| true, (10.5, 8.5)).expect("moved");
        assert!((q.0 - 10.25).abs() < 1e-6, "west: {q:?}");
    }

    /// Protects "dry": where the settlement's own bank is open water (a lake
    /// south of the river) the point is on the other bank, and where no ground
    /// is dry it stays. MUTATION: dropping `(q.dry)(p.0, p.1)` from `clear`
    /// leaves the first one in the lake at 10.75.
    #[test]
    fn a_bank_in_open_water_is_refused() {
        let g = east_river();
        let lake_south = |_: f64, y: f64| y < 10.5;
        let p = bank(&g, 16.0, 0.05, &lake_south, (8.5, 10.6)).expect("moved");
        assert!(p.1 <= 10.25 + 1e-6 && p.1 > 10.1, "the north bank, at or just past its edge plus the gap: {p:?}");
        assert_eq!(bank(&g, 16.0, 0.05, &|_, _| false, (8.5, 10.6)), None, "no dry ground: it stays");
    }

    /// Protects the narrow strip: where the only dry ground is nearer the
    /// stroke than the gap (a 0.2-cell strip south of the river, open water
    /// everywhere else), the point is on that strip, out of the water, and not
    /// left in the river for want of the margin. MUTATION: `bank_point`
    /// without its second try (`.or_else(..)` removed) returns `None`.
    #[test]
    fn where_the_gap_does_not_fit_the_point_is_still_out_of_the_water() {
        let g = east_river();
        // Dry only for 10.7 < y < 10.9: the stroke's south edge is at 10.7, a gap of 0.5 would need y >= 11.2.
        let strip = |_: f64, y: f64| y > 10.7 && y < 10.9;
        let p = bank(&g, 16.0, 0.5, &strip, (8.5, 10.6)).expect("moved onto the strip");
        assert!(p.1 > 10.7 && p.1 < 10.9, "{p:?}");
    }

    /// Protects the confluence case the first census missed: a centre in a
    /// thin tributary one edge-width from a wide trunk must clear BOTH, so
    /// the point that only clears the tributary (inside the trunk) is refused.
    /// MUTATION: `clear` testing only the covering stroke it started from (not
    /// `q.covering(p)`) accepts a point inside the other stroke.
    #[test]
    fn at_a_confluence_the_point_clears_every_stroke() {
        // The trunk: 1.2 cells wide along y = 10.5 (half 0.6). The tributary: 0.1 wide, coming down x = 8.5 from the north.
        let g = RiverGeometry { runs: vec![run(&[(2.5, 10.5), (8.5, 10.5), (14.5, 10.5)], 1.2), run(&[(8.5, 6.5), (8.5, 9.5), (8.5, 10.5)], 0.1)] };
        let c = (8.5, 9.95);
        let p = bank(&g, 16.0, 0.05, &|_, _| true, c).expect("moved");
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let a = TerrainAppearance::default();
        let index = StrokeIndex::new(&g);
        let bare = BankQuery { geom: &g, index: &index, a: &a, ppc: 16.0, gap: 0.0, dry: &|_, _| true, wide: widest_half(&g, &a), ground: None, reach_widths: None, same_bank: false };
        assert!(bare.covering(c).is_some(), "premise: the centre is in the water");
        assert!(bare.covering(p).is_none(), "the point is clear of the trunk and the tributary: {p:?}");
        assert!(p.1 < 10.5 - 0.6 - 0.049, "north of the trunk's edge plus the gap: {p:?}");
        assert!((p.0 - 8.5).abs() > 0.05 + 0.049, "off the tributary's edge plus the gap: {p:?}");
        assert!((p.0 - c.0).hypot(p.1 - c.1) <= (0.6 + 0.05) * SEARCH_REACH + 1e-9, "within the search reach");
    }

    /// Protects the zoom law's part: a river the map does not draw at this
    /// density (a headwater, rank 1, at the opening view) covers nothing, and
    /// the same river covers its centreline once the zoom shows it. A river
    /// fading in under half opacity is not yet drawn either (the pick rule's
    /// `PICK_MIN_VISIBILITY`), and from half opacity it is.
    /// MUTATION: `alpha >= PICK_MIN_VISIBILITY` to `alpha >= 0.0` makes the
    /// faint river cover.
    #[test]
    fn a_river_the_zoom_hides_moves_nothing() {
        let mut r = run(&[(2.5, 10.5), (8.5, 10.5), (14.5, 10.5)], 0.4);
        r.ranks = vec![1.0; 3];
        let g = RiverGeometry { runs: vec![r] };
        let a = TerrainAppearance::default();
        let index = StrokeIndex::new(&g);
        let q = |ppc: f32| BankQuery { geom: &g, index: &index, a: &a, ppc, gap: 0.0, dry: &|_, _| true, wide: widest_half(&g, &a), ground: None, reach_widths: None, same_bank: false };
        // Fit 1 px per cell, a whole-world start share: rank 1 is hidden at x1 and drawn from x64.
        let _law = ScaleGuard::hold(Some((1.0, 0.044)));
        assert!(q(1.0).covering((8.5, 10.5)).is_none(), "hidden at the opening view");
        assert!(q(64.0).covering((8.5, 10.5)).is_some(), "drawn at x64");
        // The fade: the first rank past the visible share whose alpha at the opening view is under a half, and
        // the last whose alpha is over it.
        let alpha = |u: f32| crate::river_stroke::river_visibility_at(u, 1.0, 1.0, 0.044);
        let ranks: Vec<f32> = (0..2000).map(|k| k as f32 / 2000.0).collect();
        let faint = *ranks.iter().find(|&&u| alpha(u) > 0.05 && alpha(u) < 0.45).expect("a rank inside the fade band");
        let solid = *ranks.iter().rev().find(|&&u| alpha(u) > 0.55).expect("a rank the opening view draws");
        let cover = |u: f32| {
            let mut r = run(&[(2.5, 10.5), (8.5, 10.5), (14.5, 10.5)], 0.4);
            r.ranks = vec![u; 3];
            let g = RiverGeometry { runs: vec![r] };
            let index = StrokeIndex::new(&g);
            BankQuery { geom: &g, index: &index, a: &a, ppc: 1.0, gap: 0.0, dry: &|_, _| true, wide: widest_half(&g, &a), ground: None, reach_widths: None, same_bank: false }.covering((8.5, 10.5)).is_some()
        };
        assert!(!cover(faint), "rank {faint} is at alpha {} at the opening view: not drawn yet", alpha(faint));
        assert!(cover(solid), "rank {solid} is at alpha {}: drawn", alpha(solid));
    }
    /// A query over `geom` for Ruling HM's tests: as drawn at `ppc` with `gap`
    /// cells, or on the ground with minimum `ground`; all ground dry.
    fn hm<'a>(geom: &'a RiverGeometry, index: &'a StrokeIndex, a: &'a TerrainAppearance, ppc: f32, gap: f64, ground: Option<f64>, same_bank: bool) -> BankQuery<'a> {
        BankQuery { geom, index, a, ppc, gap, dry: &|_, _| true, wide: widest_half(geom, a), ground, reach_widths: None, same_bank }
    }

    /// Protects Ruling HM's town edge ([`BankQuery::ground`]): on the ground a
    /// 0.4-cell river's edge is 0.2 cells from its centreline whatever the
    /// density (as drawn at one pixel per cell it is the pixel floor's 0.5),
    /// the gap is added to it, and a minimum wider than the stroke replaces
    /// it. A centre on the line takes the north bank ([`MARKET_FIRST_BANK`]).
    #[test]
    fn on_the_ground_the_edge_is_the_width_with_a_minimum_and_no_pixel_floor() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let (g, a) = (east_river(), TerrainAppearance::default());
        let index = StrokeIndex::new(&g);
        let at = |q: &BankQuery| bank_point(q, (8.5, 10.5), None, Some(MARKET_FIRST_BANK)).expect("moved");
        let p = at(&hm(&g, &index, &a, 1.0, 0.03, Some(0.05), false));
        assert!((p.0 - 8.5).abs() < 1e-9 && (p.1 - (10.5 - 0.23)).abs() < 1e-6, "ground: half the width plus the gap, north: {p:?}");
        let p = at(&hm(&g, &index, &a, 1.0, 0.03, None, false));
        assert!((p.1 - (10.5 - 0.53)).abs() < 1e-6, "as drawn at 1 px per cell: the pixel floor: {p:?}");
        let p = at(&hm(&g, &index, &a, 1.0, 0.03, Some(0.6), false));
        assert!((p.1 - (10.5 - 0.63)).abs() < 1e-6, "the minimum clearance replaces a narrower stroke: {p:?}");
        // The look does not reach the ground: a doubled river width moves nothing.
        let wide = TerrainAppearance { river_width: 2.0, ..TerrainAppearance::default() };
        let p = at(&hm(&g, &index, &wide, 1.0, 0.03, Some(0.05), false));
        assert!((p.1 - (10.5 - 0.23)).abs() < 1e-6, "the look's river width is not read on the ground: {p:?}");
    }

    /// Protects Ruling HM's marker rule: the marker stands on its market's
    /// bank. A centre just south of the line goes south on its own, north
    /// when its market is north; a centre clear of the stroke stays, unless
    /// its market is across the river, when it goes to the market's edge;
    /// and a market on the centre's own side moves a clear centre nowhere.
    #[test]
    fn the_marker_takes_its_markets_bank() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let (g, a) = (east_river(), TerrainAppearance::default());
        let index = StrokeIndex::new(&g);
        let q = hm(&g, &index, &a, 16.0, 0.07, None, true);
        let own = bank_point(&q, (8.5, 10.6), None, None).expect("moved");
        assert!((own.1 - 10.77).abs() < 1e-6, "its own bank, south: {own:?}");
        let north = bank_point(&q, (8.5, 10.6), Some((8.7, 9.0)), None).expect("moved");
        assert!((north.1 - 10.23).abs() < 1e-6 && (north.0 - 8.5).abs() < 1e-9, "the market's bank, north: {north:?}");
        assert_eq!(bank_point(&q, (8.5, 11.5), None, None), None, "a clear centre stays");
        assert_eq!(bank_point(&q, (8.5, 11.5), Some((8.5, 12.0)), None), None, "a market on its own bank moves nothing");
        let over = bank_point(&q, (8.5, 11.5), Some((8.5, 10.0)), None).expect("moved across");
        assert!((over.1 - 10.23).abs() < 1e-6, "across to the market's edge: {over:?}");
    }

    /// Protects [`StrokeIndex::crosses`] and the same-bank rule built on it:
    /// a line over the centreline crosses, one along a bank does not, and a
    /// river that is not drawn has no banks. Where the market's bank has no
    /// dry ground at all the marker still leaves the water, on the other
    /// bank (Ruling HG holds before Ruling HM).
    #[test]
    fn a_marker_is_never_handed_across_a_drawn_river_from_its_market() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let (g, a) = (east_river(), TerrainAppearance::default());
        let index = StrokeIndex::new(&g);
        assert!(index.crosses(&g, (8.5, 10.0), (8.7, 11.0), &|_| true));
        assert!(!index.crosses(&g, (8.5, 10.0), (12.0, 10.3), &|_| true));
        assert!(!index.crosses(&g, (8.5, 10.0), (8.7, 11.0), &|_| false), "a river not drawn has no banks");
        // North of the river is all open water: the market's bank cannot be reached.
        let dry = |_x: f64, y: f64| y > 10.5;
        let q = BankQuery { dry: &dry, ..hm(&g, &index, &a, 16.0, 0.07, None, true) };
        let p = bank_point(&q, (8.5, 10.5), Some((8.5, 9.0)), None).expect("out of the water");
        assert!(p.1 > 10.5 + 0.2, "on the south bank, the only dry one: {p:?}");
        // With both banks dry the same query never crosses to the south.
        let q = hm(&g, &index, &a, 16.0, 0.07, None, true);
        let p = bank_point(&q, (8.5, 10.5), Some((8.5, 9.0)), None).expect("moved");
        assert!(p.1 < 10.5 && !index.crosses(&g, p, (8.5, 9.0), &|_| true));
        // The market's bank is dry only east of x = 8.8, which the ring search reaches later than
        // the south bank: the marker still stays north of the line.
        let dry = |x: f64, y: f64| y > 10.5 || x > 8.8;
        let q = BankQuery { dry: &dry, ..hm(&g, &index, &a, 16.0, 0.07, None, true) };
        let p = bank_point(&q, (8.5, 10.5), Some((8.5, 9.0)), None).expect("moved");
        assert!(p.1 < 10.5 && p.0 > 8.8, "on the market's bank, further along: {p:?}");
        let free = BankQuery { same_bank: false, ..q };
        let p = bank_point(&free, (8.5, 10.5), Some((8.5, 9.0)), None).expect("moved");
        assert!(p.1 > 10.5, "the control: without the rule the nearer south bank is taken: {p:?}");
    }

    /// Protects Ruling HN's marker rule (`bank_point`'s market fallbacks) on two rivers running west to east, 1.2
    /// cells apart, with the market on the tongue between them and the settlement's centre south of both:
    /// (1) at a zoom that draws them narrow the centre is clear of every stroke, the one candidate across the
    /// nearest river is refused only where the tongue is not dry there, and the marker still comes to the tongue
    /// beside its market, never across a river from it; (2) at a zoom whose pixel floor closes the tongue over, no
    /// point is clear and the marker stands at the market itself; (3) a market in the water gets any clear point
    /// (the older ruling). MUTATION: dropping `near_market` leaves case 1 at `None`; dropping the dry market
    /// fallback sends case 2 across a river; dropping the `crosses` test in `near_market` lets case 1 land north of
    /// both rivers.
    #[test]
    fn the_marker_comes_to_its_markets_bank_where_the_search_round_the_centre_fails() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let a = TerrainAppearance::default();
        let g = RiverGeometry { runs: vec![run(&[(2.5, 10.0), (18.5, 10.0)], 0.2), run(&[(2.5, 11.2), (18.5, 11.2)], 0.2)] };
        let index = StrokeIndex::new(&g);
        let (centre, market) = ((8.5, 12.4), (8.5, 10.6));
        let on_tongue = |p: (f64, f64)| p.1 > 10.0 && p.1 < 11.2;
        // (1) The tongue is dry only east of x = 9.2 and at the market itself, so the point straight across from
        // the centre (8.5, 11.03) is refused.
        let dry = |x: f64, y: f64| !(y > 10.0 && y < 11.2) || x > 9.2 || ((x - 8.5).abs() < 0.05 && (y - 10.6).abs() < 0.05);
        let q = BankQuery { dry: &dry, ..hm(&g, &index, &a, 64.0, 0.07, None, true) };
        assert!(q.covering(centre).is_none(), "the control: the centre is clear of every stroke at this zoom");
        let p = bank_point(&q, centre, Some(market), None).expect("moved to the market's bank");
        assert!(on_tongue(p) && !index.crosses(&g, p, market, &|_| true), "beside its market: {p:?}");
        assert!((p.0 - market.0).hypot(p.1 - market.1) < 1e-9, "the market is itself clear, so it is the point: {p:?}");
        // (2) One pixel is 2 cells: the two floored strokes cover the whole tongue.
        let q = hm(&g, &index, &a, 0.5, 0.0, None, true);
        assert!(q.covering(market).is_some(), "the control: the pixel floor covers the market");
        let p = bank_point(&q, centre, Some(market), None).expect("moved");
        assert_eq!(p, market, "no point is clear: the market's own ground");
        // (3) The same, the market in open water: any clear point, which is across a river from it.
        let wet_tongue = |_x: f64, y: f64| !(y > 10.0 && y < 11.2);
        let q = BankQuery { dry: &wet_tongue, ..hm(&g, &index, &a, 64.0, 0.07, None, true) };
        let covered = (8.5, 11.2);
        let p = bank_point(&q, covered, Some(market), None).expect("out of the water");
        assert!(!on_tongue(p) && wet_tongue(p.0, p.1), "a dry point off the tongue: {p:?}");
    }

    /// Protects [`road_points`] and [`roads_bank`]: a settlement on the line whose one road runs
    /// off to the south is given the south bank, one whose road runs north
    /// the north bank, and one with a road each way neither; and a road's points walk outward from the
    /// settlement in the traced frame, 55 m apart.
    #[test]
    fn the_roads_bank_is_where_most_of_the_road_is() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let (g, a) = (east_river(), TerrainAppearance::default());
        let index = StrokeIndex::new(&g);
        let q = hm(&g, &index, &a, 16.0, 0.0, Some(0.0), false);
        let c = (8.5, 10.5);
        let hit = q.covering(c).expect("on the river").0;
        // A 64-cell-wide world over 64 km: 1 km cells, so a road of 3 cells is sampled about 30 times in the 1.7 km reach.
        let way = |to: (f64, f64)| cartalith_civ::Way { tid: 1, pts: vec![(8.0, 10.0), to], brks: Vec::new(), km: 3.0, name: String::new(), way_type: cartalith_civ::WayType::Road, a_idx: 0, b_idx: 1, hidden: false };
        let roads = |ways: &[cartalith_civ::Way]| road_points(ways, (8.0, 10.0), c, 64, 64.0, 1700.0);
        let side = |ways: &[cartalith_civ::Way]| roads_bank(&hit, &roads(ways)).map(|p| (p.1 - 10.5).signum());
        assert_eq!(side(&[way((8.0, 13.0))]), Some(1.0), "south");
        assert_eq!(side(&[way((8.0, 7.0))]), Some(-1.0), "north");
        assert_eq!(side(&[way((8.0, 13.0)), way((8.0, 7.0))]), None, "a road each way: neither");
        assert_eq!(side(&[]), None, "no road: neither");
        let r = &roads(&[way((8.0, 13.0))])[0];
        assert!((r[0].0 - 8.5).abs() < 1e-9 && (r[0].1 - 10.5).abs() < 1e-9, "from the settlement's centre: {:?}", r[0]);
        assert!((r[1].1 - r[0].1 - 0.055).abs() < 1e-6 && r[r.len() - 1].1 > r[0].1 + 1.6, "southward, 55 m a step, to the reach: {:?}", &r[..2]);
    }

    /// Protects [`StrokeIndex::reach_widths`] and its use on the ground: a
    /// run 0.2 cells wide that widens to 0.8 at its fourth point reads 0.8
    /// from 4.5 cells of run either side of it (its points are 4 apart), and a town two points upstream is
    /// kept clear of the wide width (0.4 from the line), not the local 0.1.
    #[test]
    fn the_towns_edge_is_the_widest_the_river_is_drawn_within_its_reach() {
        let _law = ScaleGuard::hold(Some((1.0, 1.0)));
        let mut r = run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 0.2);
        r.widths[3] = 0.8;
        let g = RiverGeometry { runs: vec![r] };
        let index = StrokeIndex::new(&g);
        assert_eq!(index.reach_widths(&g, 4.5), &[vec![0.2, 0.2, 0.8, 0.8, 0.8]]);
        let a = TerrainAppearance::default();
        let q = BankQuery { reach_widths: Some(index.reach_widths(&g, 4.5)), ..hm(&g, &index, &a, 1.0, 0.0, Some(0.0), false) };
        let p = bank_point(&q, (8.5, 10.5), None, Some(MARKET_FIRST_BANK)).expect("moved");
        assert!((p.1 - 10.1).abs() < 1e-6, "clear of the 0.8-cell width: {p:?}");
        let local = hm(&g, &index, &a, 1.0, 0.0, Some(0.0), false);
        let p = bank_point(&local, (8.5, 10.5), None, Some(MARKET_FIRST_BANK)).expect("moved");
        assert!(p.1 > 10.1 + 0.05, "without it, the interpolated local width: {p:?}");
    }

    /// Protects [`origin_point`]'s two metre constants as literals, on a
    /// 100 m grid: a town on the line of a 0.4-cell (40 m) river stands
    /// [`ORIGIN_MIN_CLEAR_M`] + [`ORIGIN_BANK_GAP_M`] = 90 m off it (the
    /// minimum, since half the width is 20 m), and on a 4-cell (400 m) river
    /// 200 + 30 = 230 m; its distance to the river is reported the same; its
    /// roads' bank is taken over its own; and a centre already 95 m off the
    /// narrow river is not moved.
    #[test]
    fn a_town_stands_thirty_metres_off_the_drawn_edge_and_never_under_ninety_from_the_line() {
        let narrow = east_river();
        let index = StrokeIndex::new(&narrow);
        let (p, d) = origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 10.5), &[]).expect("moved");
        assert!((p.1 - (10.5 - 0.9)).abs() < 1e-6 && (d - 0.9).abs() < 1e-6, "90 m, north: {p:?} {d}");
        // A road that stays in the water (it runs along the line) but lies a hair south: its bank, by the older rule.
        let along: Vec<(f64, f64)> = (0..20).map(|k| (8.5 + 0.55 * f64::from(k), 10.51)).collect();
        let (p, _) = origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 10.5), &[along]).expect("moved");
        assert!((p.1 - (10.5 + 0.9)).abs() < 1e-6 && (p.0 - 8.5).abs() < 1e-6, "its roads' bank, south, straight out: {p:?}");
        assert_eq!(origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 11.45), &[]), None, "95 m off: clear");
        let wide = RiverGeometry { runs: vec![run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 4.0)] };
        let index = StrokeIndex::new(&wide);
        let (p, d) = origin_point(&wide, &index, &|_, _| true, 100.0, (8.5, 10.5), &[]).expect("moved");
        assert!((p.1 - (10.5 - 2.3)).abs() < 1e-6 && (d - 2.3).abs() < 1e-6, "230 m: {p:?} {d}");
        // The narrow river widening to 4 cells two points downstream: the town clears the wide width.
        let mut r = run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 0.4);
        r.widths[3] = 4.0;
        let widening = RiverGeometry { runs: vec![r] };
        let index = StrokeIndex::new(&widening);
        let (p, _) = origin_point(&widening, &index, &|_, _| true, 100.0, (8.5, 10.5), &[]).expect("moved");
        assert!((p.1 - (10.5 - 2.3)).abs() < 1e-6, "the widest width within its reach: {p:?}");
    }

    /// Protects Ruling HN's origin rule on a 100 m grid: a town on the line of a 4-cell (400 m) river whose one
    /// road comes in from the south-east stands where that road reaches the bank (the first road point 230 m or
    /// more off the line, south), not straight out from the centre on the north bank the centre's tie-break names;
    /// a road whose landing is past the search's reach (2 x 230 m from the centre) is not followed; and with the
    /// tributary's stroke under the centre and the trunk beside it, the town lands on the road's bank of the
    /// trunk, where the older rule put it across the trunk from its road. MUTATION: dropping the landing puts case 1
    /// at (8.5, 8.2); dropping the reach test follows case 2's road.
    #[test]
    fn a_town_stands_where_its_road_reaches_the_bank() {
        let wide = RiverGeometry { runs: vec![run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 4.0)] };
        let index = StrokeIndex::new(&wide);
        let c = (8.5, 10.5);
        // South-east at 45 degrees, a point every 55 m of path (0.55 cells).
        let road: Vec<(f64, f64)> = (0..60).map(|k| (c.0 + 0.55 * f64::from(k) * 0.5f64.sqrt(), c.1 + 0.55 * f64::from(k) * 0.5f64.sqrt())).collect();
        let (p, d) = origin_point(&wide, &index, &|_, _| true, 100.0, c, &[road.clone()]).expect("moved");
        assert!(p.1 > 10.5 + 2.3 && p.1 < 10.5 + 2.3 + 0.55 && (p.0 - c.0 - (p.1 - c.1)).abs() < 1e-9, "on its road, just past the edge and the gap: {p:?}");
        assert!((d - (p.1 - 10.5)).abs() < 1e-9, "its distance from the line: {d}");
        // The same road running a long way down the river before it leaves it: past the reach, the older rule.
        let far: Vec<(f64, f64)> = (0..200).map(|k| (c.0 + 0.55 * f64::from(k), c.1 + 0.025 * f64::from(k))).collect();
        assert!(far.iter().any(|p| p.1 > 10.5 + 2.3), "the control: this road does leave the river");
        let (p, _) = origin_point(&wide, &index, &|_, _| true, 100.0, c, &[far]).expect("moved");
        assert!((p.0 - 8.5).abs() < 1e-6 && (p.1 - (10.5 + 2.3)).abs() < 1e-6, "straight out on its roads' bank: {p:?}");
        // A tributary from the north ending on the trunk's line at the centre, drawn second: the road still decides.
        let both = RiverGeometry { runs: vec![run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 4.0), run(&[(8.5, 2.5), (8.5, 6.5), (8.5, 10.5)], 0.4)] };
        let index = StrokeIndex::new(&both);
        let (p, _) = origin_point(&both, &index, &|_, _| true, 100.0, c, &[road]).expect("moved");
        assert!(p.1 > 10.5 + 2.3, "south of the trunk, on its road: {p:?}");
    }

}
