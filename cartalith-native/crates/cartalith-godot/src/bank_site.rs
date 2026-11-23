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
/// Where no point on the market's bank is clear (with the gap, then
/// without), the search is run once more for any clear point: out of the
/// water on the wrong bank is the older ruling (HG) and still holds.
///
/// Must never return a point in open water or inside a drawn stroke.
pub fn bank_point(q: &BankQuery, centre: (f64, f64), market: Option<(f64, f64)>, prefer: Option<(f64, f64)>) -> Option<(f64, f64)> {
    let two = |q: &BankQuery| bank_point_at_gap(q, centre, market, prefer).or_else(|| if q.gap > 0.0 { bank_point_at_gap(&BankQuery { gap: 0.0, ..*q }, centre, market, prefer) } else { None });
    two(q).or_else(|| if q.same_bank && market.is_some() { two(&BankQuery { same_bank: false, ..*q }) } else { None })
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

/// **Ruling HM: the bank a settlement's roads are on**, as a point one cell
/// out from the stroke at `hit` on that side (a side for [`bank_point`]'s
/// `market`, not a place), or `None` when the roads favour neither bank.
///
/// The roads are the ones the town is laid out with
/// (`urban_adapter::um_primary_paths`: every way ending at the cell `at`,
/// sampled every 55 m out to the town box's long side), and each sample
/// counts one for the side of the stroke it is on. A town is laid out from
/// its market along these roads, and a market put across a river as wide as
/// the map draws it from all of them houses almost nobody (measured on seed
/// 483920 at 384 cells before this rule: hamlets at 1 to 13 % of their
/// target). **A labelled judgement**: a majority of samples, any margin.
fn roads_bank(hit: &Hit, centre: (f64, f64), ways: &[cartalith_civ::Way], at: (f64, f64), gw: usize, map_width_km: f64) -> Option<(f64, f64)> {
    let gpm = gw as f64 / (map_width_km * 1000.0);
    let paths = cartalith_civ::urban_adapter::um_primary_paths(ways, at.0, at.1, gw, map_width_km, 0.0)?;
    let sum: f64 = paths
        .iter()
        .flatten()
        .filter_map(|o| side_of(hit, (centre.0 + o.x * gpm, centre.1 + o.y * gpm)))
        .map(|n| (n.0 * hit.n.0 + n.1 * hit.n.1).signum())
        .sum();
    (sum != 0.0).then(|| (hit.foot.0 + hit.n.0 * sum.signum(), hit.foot.1 + hit.n.1 * sum.signum()))
}

/// [`WorldGen::town_origin`]'s rule, apart from the world it reads: the town
/// point for a settlement whose cell centre is `c` (traced frame, cells) on a
/// grid of `cell_m` metres a cell, and its distance from the nearest drawn
/// centreline in cells. `roads` names the bank the settlement's roads are on,
/// given the stroke covering `c` ([`roads_bank`]). `None` where `c` is
/// already clear or no dry clear point is in reach.
pub fn origin_point(
    geom: &RiverGeometry,
    index: &StrokeIndex,
    dry: &dyn Fn(f64, f64) -> bool,
    cell_m: f64,
    c: (f64, f64),
    roads: &dyn Fn(&Hit) -> Option<(f64, f64)>,
) -> Option<((f64, f64), f64)> {
    // `a` and `ppc` are not read on the ground.
    let a = TerrainAppearance::default();
    let min = ORIGIN_MIN_CLEAR_M / cell_m;
    let wide = widest_ground_half(geom);
    // The widest the river is drawn anywhere the town's box can reach along it: the points within
    // the adapter's own stem reach (`um_water_ctx_box_smooth`: 1.8 cells, or three quarters of the
    // box's long side, here the box grown for the widest river), half as far again since the
    // adapter's reach is straight-line and this one is along the run.
    let long_m = cartalith_civ::urban_adapter::SITE_WM + 2.0 * (wide * cell_m + ORIGIN_BANK_GAP_M + ORIGIN_MIN_CLEAR_M);
    let k = (0.75 * long_m / cell_m).max(1.8) * 1.5;
    let q = BankQuery { geom, index, a: &a, ppc: 1.0, gap: ORIGIN_BANK_GAP_M / cell_m, dry, wide, ground: Some(min), reach_widths: Some(index.reach_widths(geom, k)), same_bank: false };
    // The bank the settlement's roads are on, else the centre's own.
    let side = q.covering(c).and_then(|(hit, _)| roads(&hit));
    let p = bank_point(&q, c, side, Some(MARKET_FIRST_BANK))?;
    let river = index.nearest(geom, p, (wide.max(min) + q.gap) * (1.0 + SEARCH_REACH) + 1.0)?;
    Some((p, river.d))
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
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0)
            }
        };
        let q = BankQuery { geom: &geom, index: &index, a: &a, ppc, gap: gap_px.max(0.0) / f64::from(ppc), dry: &dry, wide: widest_half(&geom, &a), ground: None, reach_widths: None, same_bank: true };
        // Ruling HM: each marker on its market's bank (`market_points`).
        let markets = self.market_points();
        Some(civ.settlements.iter().zip(markets).map(|(s, m)| bank_point(&q, (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), m, Some(MARKET_FIRST_BANK))).collect())
    }

    /// **Ruling HM: where settlement `s`'s town is laid out**, when that is
    /// not its cell: the nearest dry point clear of every drawn river at its
    /// width on the ground ([`BankQuery::ground`], never nearer the centreline
    /// than [`ORIGIN_MIN_CLEAR_M`]) by [`ORIGIN_BANK_GAP_M`], on the bank its roads are on
    /// ([`roads_bank`]), else the bank the cell centre is on ([`MARKET_FIRST_BANK`] for a centre on
    /// the line). The same rule as the
    /// marker's ([`bank_point`]) at the one density a generated town can use:
    /// none. `None` for a settlement already clear (its town stays round its
    /// cell), with the `civ.bank_sites` switch off, with no drawn network (a
    /// loaded save), and where no dry clear point is in reach.
    ///
    /// Must depend on nothing a view can change (zoom, window, look): a
    /// layout is cached, hashed and frozen in a save.
    pub(crate) fn town_origin(&self, s: &cartalith_civ::NamedSettlement) -> Option<cartalith_civ::urban_adapter::BankOrigin> {
        if !self.params.civ.bank_sites {
            return None;
        }
        let (Some(civ), Some(crate::WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else { return None };
        let (geom, index) = self.bank_index()?;
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let cell_m = self.map_width_km * 1000.0 / gw.max(1) as f64;
        if !(cell_m > 0.0) {
            return None;
        }
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0)
            }
        };
        let c = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
        let (p, river_cells) = origin_point(&geom, &index, &dry, cell_m, c, &|hit| roads_bank(hit, c, &civ.ways, (s.placement.x as f64, s.placement.y as f64), gw, self.map_width_km))?;
        // The adapter's frame puts cell `c` at `c`; the traced frame at `c + 0.5`.
        Some(cartalith_civ::urban_adapter::BankOrigin { at: (p.0 - 0.5, p.1 - 0.5), river_m: river_cells * cell_m })
    }

    /// **Ruling HM: where each settlement's market stands on the map**
    /// (traced frame, cells), in `get_settlements()` order, for the marker to
    /// follow: a frozen town's own market (its saved layout, which is what
    /// the City Viewer shows whatever the rules have become), else the point
    /// its town is laid round ([`Self::town_origin`], or the cell centre for
    /// a town that stays round its cell: where the layout puts the market
    /// unless one of its own rules moves it; `town_bank_census` counts the
    /// towns where one did).
    fn market_points(&self) -> Vec<Option<(f64, f64)>> {
        let Some(civ) = self.civ.as_ref() else { return Vec::new() };
        civ.settlements
            .iter()
            .map(|s| match self.town_layouts.rows.get(&s.tid).and_then(|r| r.layout.as_ref()) {
                Some(l) => Some(self.town_grid(l, s.placement.x, s.placement.y, l.market)),
                None => Some(self.town_origin(s).map_or((s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5), |o| (o.at.0 + 0.5, o.at.1 + 0.5))),
            })
            .collect()
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

    /// **The Ruling HM census** (a measurement; `_bankcensus_probe.gd --towns`
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
    ///   `pop`, `pop_target`;
    /// - `over_b`, `over_w`, `over_s` and `of_b`, `of_w`, `of_s`: of the
    ///   town's footprint points carried onto the map (every building corner
    ///   and centre; every wall ring vertex; every street's two ends and
    ///   middle), how many lie inside a river stroke the map draws at its
    ///   ground width (the default look's; [`BankQuery::ground`] with no
    ///   minimum), and how many were tested. A street that bridges the river
    ///   is over it by design, so the streets are counted apart;
    /// - `mx`, `my`: the market on the map, cells from the cell centre;
    /// - `side_x1`, `side_x4`: `1` the marker and the market stand on one
    ///   bank, `-1` the centreline of a river drawn at that zoom runs
    ///   between them ([`StrokeIndex::crosses`]); absent with no drawn river
    ///   within two cells of the cell.
    #[func]
    fn town_bank_census(&self, indices: PackedInt32Array, view_w: f64, view_h: f64) -> Array<VarDictionary> {
        use crate::town_measures::{bank_frontage, laid_ctx, line_dist, wet_buildings};
        let mut out = Array::new();
        let Some((geom, index)) = self.bank_index() else { return out };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let fit = crate::river_stroke::fit_ppc_in((view_w as f32, view_h as f32), gw, gh);
        let marks: Vec<PackedVector2Array> = {
            let _law = crate::river_stroke::ScaleGuard::hold(Some((fit, crate::river_stroke::start_share(self.map_width_km as f32))));
            [fit, fit * 4.0].iter().map(|&ppc| self.settlement_bank_offsets(f64::from(ppc), -1.0)).collect()
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
            let Some(l) = self.stretch_layout(ctx, rules) else { return d };
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
            use cartalith_civ::urban_adapter::Vec2 as V;
            let mut kinds: [Vec<V>; 3] = [Vec::new(), Vec::new(), Vec::new()];
            for b in &l.buildings {
                let n = b.poly.len().max(1) as f64;
                kinds[0].extend(b.poly.iter().copied());
                kinds[0].push(V::new(b.poly.iter().map(|v| v.x).sum::<f64>() / n, b.poly.iter().map(|v| v.y).sum::<f64>() / n));
            }
            kinds[1].extend(l.wall.ring.iter().flatten().copied());
            for e in &l.edges {
                kinds[2].extend([e.a, e.b, V::new((e.a.x + e.b.x) / 2.0, (e.a.y + e.b.y) / 2.0)]);
            }
            for (pts, k) in kinds.iter().zip(["b", "w", "s"]) {
                let over = pts.iter().filter(|&&v| strokes.covering(self.town_grid(&l, s.placement.x, s.placement.y, v)).is_some()).count();
                d.set(format!("over_{k}"), over as i64);
                d.set(format!("of_{k}"), pts.len() as i64);
            }
            let c = (s.placement.x as f64 + 0.5, s.placement.y as f64 + 0.5);
            let m = self.town_grid(&l, s.placement.x, s.placement.y, l.market);
            d.set("mx", m.0 - c.0);
            d.set("my", m.1 - c.1);
            if index.nearest(&geom, c, 2.0).is_some() {
                for (k, name) in ["x1", "x4"].iter().enumerate() {
                    let off = marks[k].get(i).unwrap_or(Vector2::ZERO);
                    let p = (c.0 + f64::from(off.x), c.1 + f64::from(off.y));
                    let ppc = fit * [1.0, 4.0][k];
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
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0)
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
        let dry = |x: f64, y: f64| {
            x >= 0.0 && y >= 0.0 && (x as usize) < gw && (y as usize) < gh && {
                let k = y as usize * gw + x as usize;
                f64::from(ws.field[k]) >= self.sea_level && civ.water_bodies.get(k).is_none_or(|&c| c == 0)
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

    /// Protects [`roads_bank`]: a settlement on the line whose one road runs
    /// off to the south is given the south bank, one whose road runs north
    /// the north bank, and one with a road each way neither.
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
        let side = |ways: &[cartalith_civ::Way]| roads_bank(&hit, c, ways, (8.0, 10.0), 64, 64.0).map(|p| (p.1 - 10.5).signum());
        assert_eq!(side(&[way((8.0, 13.0))]), Some(1.0), "south");
        assert_eq!(side(&[way((8.0, 7.0))]), Some(-1.0), "north");
        assert_eq!(side(&[way((8.0, 13.0)), way((8.0, 7.0))]), None, "a road each way: neither");
        assert_eq!(side(&[]), None, "no road: neither");
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
        let (p, d) = origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 10.5), &|_| None).expect("moved");
        assert!((p.1 - (10.5 - 0.9)).abs() < 1e-6 && (d - 0.9).abs() < 1e-6, "90 m, north: {p:?} {d}");
        let (p, _) = origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 10.5), &|h| Some((h.foot.0, h.foot.1 + 1.0))).expect("moved");
        assert!((p.1 - (10.5 + 0.9)).abs() < 1e-6, "its roads' bank, south: {p:?}");
        assert_eq!(origin_point(&narrow, &index, &|_, _| true, 100.0, (8.5, 11.45), &|_| None), None, "95 m off: clear");
        let wide = RiverGeometry { runs: vec![run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 4.0)] };
        let index = StrokeIndex::new(&wide);
        let (p, d) = origin_point(&wide, &index, &|_, _| true, 100.0, (8.5, 10.5), &|_| None).expect("moved");
        assert!((p.1 - (10.5 - 2.3)).abs() < 1e-6 && (d - 2.3).abs() < 1e-6, "230 m: {p:?} {d}");
        // The narrow river widening to 4 cells two points downstream: the town clears the wide width.
        let mut r = run(&[(2.5, 10.5), (6.5, 10.5), (10.5, 10.5), (14.5, 10.5), (18.5, 10.5)], 0.4);
        r.widths[3] = 4.0;
        let widening = RiverGeometry { runs: vec![r] };
        let index = StrokeIndex::new(&widening);
        let (p, _) = origin_point(&widening, &index, &|_, _| true, 100.0, (8.5, 10.5), &|_| None).expect("moved");
        assert!((p.1 - (10.5 - 2.3)).abs() < 1e-6, "the widest width within its reach: {p:?}");
    }

}
