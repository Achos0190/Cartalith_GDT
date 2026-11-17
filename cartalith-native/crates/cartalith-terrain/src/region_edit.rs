//! Region edits for the assistant's edit actions (`LLM_ASSISTANT_SCOPE.md` s.5c, milestone
//! LLM-7; owner Rulings DD Q10, DI, and the "region edit" ruling recorded beside them): raise,
//! lower or smooth the ground inside a region the USER painted or boxed, by a bounded amount.
//!
//! ## What it is, and how it relates to the sculpt tools
//!
//! The sculpt editor's Freehand `Raise`, `Lower` and `Smooth` sub-modes (`sculpt.rs`,
//! `SculptStamp::apply_into`) weigh a cell by its distance to a STROKE. A painted region is not a
//! stroke, so this module weighs a cell by its distance to the region's EDGE instead, and keeps
//! everything else the sculpt tools do:
//!
//! * the ramp shapes are the sculpt tools' own [`Falloff`] (`Smooth`, `Linear`, `Sharp`,
//!   `Constant`), through the same `Falloff::coverage` call;
//! * a raise or lower adds `weight x delta` and is clamped by `sculpt::clamp_height` (the stamp's
//!   own rule: never below 0, never above `max(1, h0)`);
//! * a smooth is the stamp's own four-neighbour blur over a stable snapshot
//!   (`new = old + (average - old) x weight x strength`), edges replicated.
//!
//! It is NOT a sculpt stamp and does not use the draft stack: a stamp is a recipe re-evaluated at
//! commit with noise-warped edges, and the owner's bar for edit actions is the opposite, an edit
//! that is exactly bounded to the painted cells and whose preview is bit-for-bit what Apply does.
//! The same function ([`apply_ops`]) produces both, over a window copy, so preview and apply cannot
//! drift (`tests` pin it). Rivers, lakes and paint are NOT touched here; the engine bridge
//! (`region_edit_bridge.rs`) records the undo step, re-runs hydrology and climate exactly as a
//! sculpt commit does, and refuses when committed paint lies under the region.
//!
//! ## Why a distance to the edge gives a ridge
//!
//! A cell's weight is `Falloff::coverage(d / feather)` where `d` is the chamfer distance to the
//! nearest cell outside the region (1 for a cell on the edge) and `feather = max(1.5, softness x
//! deepest)`, `deepest` being the largest `d` anywhere in the region. With `softness` 1 the weight
//! reaches its maximum only at the deepest cells, so raising an elongated painted stroke yields a
//! ridge along the stroke's medial line, and raising a blob yields a dome. With `softness` near 0
//! the region is raised as a flat-topped block with a one-to-two-cell shoulder.
//!
//! ## Limits (every one a labelled judgement; the model-facing twins are in
//! `cartalith_llm::edit_actions`, and `cartalith-godot/tests/region_edit_limits.rs` pins that
//! the two sets are equal, because the LLM crate cannot depend on this one)
//!
//! MUST NEVER: write a cell outside the region mask; write a non-finite height; accept a region
//! above [`MAX_REGION_CELLS`] or an amount above [`MAX_AMOUNT_M`]; clamp or round a caller's
//! number instead of refusing it (the caller validated already; this is the second wall).

use crate::sculpt::{Falloff, FreehandMode, Point, clamp_height, freehand_shape, nearest_on_stroke};

/// Most cells one edit's region may hold. A labelled judgement (no measurement behind it): 250 000
/// cells is a 500 x 500 block, about 9 percent of the default 2048 x 1311 grid, so one proposal
/// cannot rewrite a continent; the window copy of it is 1 MB of `f32`, cheap to preview. The owner
/// may move it (Ruling DQ question E8).
pub const MAX_REGION_CELLS: usize = 250_000;
/// Fewest cells a region may hold: below it there is no area to describe or smooth (a labelled
/// judgement; a 4 x 4 block).
pub const MIN_REGION_CELLS: usize = 16;
/// Most operations in one proposal. Labelled judgement: "raise it, then smooth it" is two; three
/// leaves room for "lower the middle first". Refused whole above this, never truncated.
pub const MAX_EDIT_OPS: usize = 3;
/// Most metres one raise or lower may move the ground. Labelled judgement: 500 m is 12.5 percent of
/// the default 4 000 m peak, enough for "a ridge about 300 m high" with headroom and far below a
/// whole mountain range.
pub const MAX_AMOUNT_M: f64 = 500.0;
/// Most metres all raises and lowers of one proposal may move together (a labelled judgement: the
/// per-operation cap plus one third). **It is also the most any ONE CELL may move in one edit, whatever
/// the operations** ([`apply_ops`]'s last pass holds a cell to it and counts it in `Applied::limited`):
/// raises and lowers can sum to no more than this, but a smooth moves a cell toward its neighbours, and
/// beside a cliff that can be hundreds of metres (measured on three seeds, a 300 m raise then a smooth
/// moved a cell 431.8 m). This is the scope's "predicted height change per cell is bounded".
pub const MAX_TOTAL_AMOUNT_M: f64 = 600.0;
/// Most smoothing passes in one operation. Labelled judgement: eight blurs of weight 1 flatten
/// detail finer than about three cells, which is the most a "smooth this" request needs.
pub const MAX_SMOOTH_PASSES: u32 = 8;
/// Smallest and largest smoothing strength (the stamp's own `intensity` control is 0..1.5; above 1
/// a blur overshoots its neighbourhood average, which is not smoothing).
pub const STRENGTH_RANGE: (f64, f64) = (0.05, 1.0);
/// Most paint dabs one region may be made of (a labelled judgement: a hand-painted blob of about
/// 40 strokes at 12 dabs each; keeps the spec a few KB).
pub const MAX_DABS: usize = 512;
/// A dab's radius range in cells: the sculpt brush's own slider range (`SculptGlobals::brush_size`,
/// "Slider range 6..200, step 1 (cells)").
pub const DAB_RADIUS_RANGE: (f64, f64) = (6.0, 200.0);
/// Largest bounding box of a region as a fraction of nothing: a bounding box over this many cells is
/// refused before a mask is allocated (a labelled judgement: sixteen times the cell cap, 4 million,
/// above the default 2.7 million grid so a legal region is never refused here, and a hostile spec
/// cannot make a multi-gigabyte mask).
pub const MAX_BBOX_CELLS: usize = 4_000_000;

/// Why a region or an edit was refused, in words a person reads.
#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// The spec holds no shape.
    Empty,
    /// More than [`MAX_DABS`] dabs.
    TooManyDabs(usize),
    /// A number in the spec is not finite or a dab radius is outside [`DAB_RADIUS_RANGE`].
    BadNumber(String),
    /// The shape lies entirely outside the world.
    OutsideWorld,
    /// Fewer than [`MIN_REGION_CELLS`] cells.
    TooSmall(usize),
    /// More than [`MAX_REGION_CELLS`] cells.
    TooLarge(usize),
    /// The world has no usable height scale (`peak_m <= 0` or `sea >= 1`).
    NoScale,
    /// The operation list breaks a limit.
    Ops(String),
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::Empty => write!(f, "the region holds no cells"),
            EditError::TooManyDabs(n) => write!(f, "the region is made of {n} paint dabs; at most {MAX_DABS} are accepted"),
            EditError::BadNumber(w) => write!(f, "{w}"),
            EditError::OutsideWorld => write!(f, "the region lies outside the world"),
            EditError::TooSmall(n) => write!(f, "the region holds {n} cells; at least {MIN_REGION_CELLS} are needed"),
            EditError::TooLarge(n) => write!(f, "the region holds {n} cells; at most {MAX_REGION_CELLS} may be edited at once"),
            EditError::NoScale => write!(f, "the world has no usable height scale"),
            EditError::Ops(w) => write!(f, "{w}"),
        }
    }
}

/// A region as the user made it: an optional rectangle (the Region select marquee) and any number
/// of round dabs (the paint brush), in grid cells. The union is the region.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RegionSpec {
    /// `[x, y, w, h]` in cells (the marquee). `None` when there is none.
    pub rect: Option<[f64; 4]>,
    /// `[cx, cy, radius]` in cells, one per dab.
    pub dabs: Vec<[f64; 3]>,
}

/// A rasterised region: a boolean window over its bounding box, clipped to the world.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionMask {
    /// Left column of the bounding box.
    pub x0: usize,
    /// Top row of the bounding box.
    pub y0: usize,
    /// Bounding box width.
    pub w: usize,
    /// Bounding box height.
    pub h: usize,
    /// Row-major `w * h`; `true` is in the region.
    pub cells: Vec<bool>,
    /// How many cells are `true`.
    pub count: usize,
    /// True when part of the user's shape lay outside the world and was cut off.
    pub clipped: bool,
    /// The painted dabs' centres `[x, y]` in cell coordinates, in the order they were painted: the
    /// polyline a path-following stamp (ridge, cliff, canyon) runs along (Ruling FW item 1). Empty for a
    /// pure box. Not clipped to the world, as the user painted it.
    pub path: Vec<[f64; 2]>,
    /// The largest dab radius in cells (0 when there are no dabs): the brush radius `r` of a path stamp.
    pub path_r: f64,
    /// True when the region includes a drawn rectangle (the Region select box). A box has no path, so a
    /// path-following stamp refuses it.
    pub boxed: bool,
}

fn finite(label: &str, v: f64) -> Result<f64, EditError> {
    if v.is_finite() { Ok(v) } else { Err(EditError::BadNumber(format!("{label} is not a finite number"))) }
}

/// Rasterise `spec` over a `gw x gh` world. Pure; allocates one bounding-box mask.
///
/// A cell is in the region when its CENTRE (`x + 0.5`, `y + 0.5`) lies inside the rectangle or
/// within a dab's radius. The shape is clipped to the world and `clipped` says so. Refuses an empty
/// shape, a non-finite number, a dab radius outside [`DAB_RADIUS_RANGE`], more than [`MAX_DABS`]
/// dabs, a shape entirely outside the world, and a region under [`MIN_REGION_CELLS`] or over
/// [`MAX_REGION_CELLS`] cells.
pub fn rasterise(spec: &RegionSpec, gw: usize, gh: usize) -> Result<RegionMask, EditError> {
    if spec.rect.is_none() && spec.dabs.is_empty() {
        return Err(EditError::Empty);
    }
    if spec.dabs.len() > MAX_DABS {
        return Err(EditError::TooManyDabs(spec.dabs.len()));
    }
    if gw == 0 || gh == 0 {
        return Err(EditError::OutsideWorld);
    }
    // Shape extents in continuous cell coordinates (before clipping).
    let (mut lx, mut ly, mut hx, mut hy) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    if let Some(r) = spec.rect {
        let (x, y, w, h) = (finite("the box x", r[0])?, finite("the box y", r[1])?, finite("the box width", r[2])?, finite("the box height", r[3])?);
        if w <= 0.0 || h <= 0.0 {
            return Err(EditError::BadNumber("the box has no area".into()));
        }
        lx = lx.min(x);
        ly = ly.min(y);
        hx = hx.max(x + w);
        hy = hy.max(y + h);
    }
    for d in &spec.dabs {
        let (cx, cy, r) = (finite("a dab x", d[0])?, finite("a dab y", d[1])?, finite("a dab radius", d[2])?);
        if r < DAB_RADIUS_RANGE.0 || r > DAB_RADIUS_RANGE.1 {
            return Err(EditError::BadNumber(format!("a dab radius of {r} is outside {}..{} cells", DAB_RADIUS_RANGE.0, DAB_RADIUS_RANGE.1)));
        }
        lx = lx.min(cx - r);
        ly = ly.min(cy - r);
        hx = hx.max(cx + r);
        hy = hy.max(cy + r);
    }
    // Clip the extents to the world; a shape wholly outside has an empty clipped box.
    let x0 = lx.floor().max(0.0);
    let y0 = ly.floor().max(0.0);
    let x1 = hx.ceil().min(gw as f64);
    let y1 = hy.ceil().min(gh as f64);
    if !(x1 > x0 && y1 > y0) {
        return Err(EditError::OutsideWorld);
    }
    let (x0, y0, x1, y1) = (x0 as usize, y0 as usize, x1 as usize, y1 as usize);
    let (w, h) = (x1 - x0, y1 - y0);
    if w.saturating_mul(h) > MAX_BBOX_CELLS {
        return Err(EditError::TooLarge(w.saturating_mul(h)));
    }
    let mut cells = vec![false; w * h];
    for (row, line) in cells.chunks_mut(w).enumerate() {
        let cy = (y0 + row) as f64 + 0.5;
        for (col, c) in line.iter_mut().enumerate() {
            let cx = (x0 + col) as f64 + 0.5;
            let in_rect = spec.rect.is_some_and(|r| cx >= r[0] && cx < r[0] + r[2] && cy >= r[1] && cy < r[1] + r[3]);
            *c = in_rect
                || spec.dabs.iter().any(|d| {
                    let (dx, dy) = (cx - d[0], cy - d[1]);
                    dx * dx + dy * dy <= d[2] * d[2]
                });
        }
    }
    let count = cells.iter().filter(|c| **c).count();
    if count == 0 {
        return Err(EditError::OutsideWorld);
    }
    if count < MIN_REGION_CELLS {
        return Err(EditError::TooSmall(count));
    }
    if count > MAX_REGION_CELLS {
        return Err(EditError::TooLarge(count));
    }
    let clipped = lx < 0.0 || ly < 0.0 || hx > gw as f64 || hy > gh as f64;
    let path: Vec<[f64; 2]> = spec.dabs.iter().map(|d| [d[0], d[1]]).collect();
    let path_r = spec.dabs.iter().map(|d| d[2]).fold(0.0f64, f64::max);
    Ok(RegionMask { x0, y0, w, h, cells, count, clipped, path, path_r, boxed: spec.rect.is_some() })
}

/// Height units to metres above sea (`metersPerUnit`'s anchoring, the formula `world_summary.rs`
/// and `sample_bridge` share): `(h - sea) / (1 - sea) x peak_m`.
pub fn to_metres(h: f64, sea: f64, peak_m: f64) -> f64 {
    (h - sea) / (1.0 - sea) * peak_m
}

/// What the app can say about a region's ground. Every figure is a measurement of the field passed
/// in; `mean_slope_m_per_km` is `None` when the world has no cell size.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionStats {
    /// Cells in the region.
    pub count: usize,
    /// Lowest cell, metres above sea (negative below).
    pub min_m: f64,
    /// Mean over the region, metres above sea.
    pub mean_m: f64,
    /// Highest cell, metres above sea.
    pub max_m: f64,
    /// Share of the region's cells at or above sea level, 0..1.
    pub land_fraction: f64,
    /// Mean gradient magnitude in metres of height per kilometre of ground; `None` without a cell size.
    pub mean_slope_m_per_km: Option<f64>,
}

/// Measure `mask` over `field` (`gw x gh`). `None` when the field is not `gw * gh` long or the
/// world has no height scale.
pub fn stats(mask: &RegionMask, field: &[f32], gw: usize, gh: usize, sea: f64, peak_m: f64, cell_km: Option<f64>) -> Option<RegionStats> {
    if field.len() != gw * gh || !(peak_m > 0.0) || !(sea < 1.0) || mask.count == 0 {
        return None;
    }
    let (mut lo, mut hi, mut sum, mut land) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64, 0usize);
    let (mut slope_sum, mut slope_n) = (0.0f64, 0usize);
    let m_per_unit = peak_m / (1.0 - sea);
    for row in 0..mask.h {
        for col in 0..mask.w {
            if !mask.cells[row * mask.w + col] {
                continue;
            }
            let (x, y) = (mask.x0 + col, mask.y0 + row);
            let hv = field[y * gw + x] as f64;
            let m = to_metres(hv, sea, peak_m);
            lo = lo.min(m);
            hi = hi.max(m);
            sum += m;
            if hv >= sea {
                land += 1;
            }
            if let Some(km) = cell_km.filter(|k| *k > 0.0) {
                let at = |xx: usize, yy: usize| field[yy * gw + xx] as f64;
                let (xl, xr) = (x.saturating_sub(1), (x + 1).min(gw - 1));
                let (yu, yd) = (y.saturating_sub(1), (y + 1).min(gh - 1));
                let dx = (at(xr, y) - at(xl, y)) / ((xr - xl).max(1) as f64);
                let dy = (at(x, yd) - at(x, yu)) / ((yd - yu).max(1) as f64);
                slope_sum += (dx * dx + dy * dy).sqrt() * m_per_unit / km;
                slope_n += 1;
            }
        }
    }
    Some(RegionStats {
        count: mask.count,
        min_m: lo,
        mean_m: sum / mask.count as f64,
        max_m: hi,
        land_fraction: land as f64 / mask.count as f64,
        mean_slope_m_per_km: (slope_n > 0).then(|| slope_sum / slope_n as f64),
    })
}

/// One operation of an edit, as the validator produced it. Every number was range-checked by the
/// caller and is re-checked by [`check_ops`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// Add `amount_m` metres at full weight.
    Raise {
        /// Metres, `0 < amount_m <= MAX_AMOUNT_M`.
        amount_m: f64,
    },
    /// Remove `amount_m` metres at full weight.
    Lower {
        /// Metres, `0 < amount_m <= MAX_AMOUNT_M`.
        amount_m: f64,
    },
    /// Blur toward the four-neighbour average, `passes` times at `strength`.
    Smooth {
        /// Within [`STRENGTH_RANGE`].
        strength: f64,
        /// 1 to [`MAX_SMOOTH_PASSES`].
        passes: u32,
    },
    /// One of the sculpt panel's five shaped Freehand sub-modes (Ruling FW item 1), evaluated by the
    /// sculpt stamp's own formula ([`freehand_shape`]) over a path derived from the painted region and
    /// weighted by the region's edge ramp so it is exactly bounded to the painted cells.
    Stamp {
        /// Which shape.
        kind: StampKind,
        /// The largest change the stamp makes, metres, `0 < amount_m <= MAX_AMOUNT_M` (the formulas'
        /// own multiples are divided out, see [`StampKind::peak_factor`]).
        amount_m: f64,
    },
}

/// The five stamps (`FreehandMode`'s shaped sub-modes; `Feature` has no Mesa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampKind {
    /// One crest along the painted path.
    Ridge,
    /// A one-sided step across the painted path (the high side is left of the stroke).
    Cliff,
    /// A carved channel along the painted path.
    Canyon,
    /// A flat-topped rise centred on the region's deepest cell.
    Mesa,
    /// A cone centred on the region's deepest cell.
    Volcano,
}

impl StampKind {
    /// Every kind, in the model-facing order.
    pub const ALL: [StampKind; 5] = [StampKind::Ridge, StampKind::Cliff, StampKind::Canyon, StampKind::Mesa, StampKind::Volcano];

    /// The sculpt Freehand sub-mode that evaluates it.
    pub fn mode(self) -> FreehandMode {
        match self {
            StampKind::Ridge => FreehandMode::Ridge,
            StampKind::Cliff => FreehandMode::Cliff,
            StampKind::Canyon => FreehandMode::Canyon,
            StampKind::Mesa => FreehandMode::Mesa,
            StampKind::Volcano => FreehandMode::Volcano,
        }
    }

    /// The model-facing name (also the sculpt sub-mode's key).
    pub fn key(self) -> &'static str {
        self.mode().key()
    }

    /// The reverse of [`StampKind::key`].
    pub fn from_key(k: &str) -> Option<StampKind> {
        StampKind::ALL.iter().copied().find(|s| s.key() == k)
    }

    /// True for the three that follow the painted path (and so refuse a box); mesa and volcano are
    /// radial and centre on the deepest cell.
    pub fn follows_path(self) -> bool {
        matches!(self, StampKind::Ridge | StampKind::Cliff | StampKind::Canyon)
    }

    /// How many times the formula's `amount` its strongest change is: ridge 1 (the crest), cliff 1
    /// (the step's two ends are +-amount), canyon 1.5 (the floor), mesa 1.25 (the top), volcano 1.5
    /// (the peak), read off `freehand_shape`. The model's `amount_m` is the strongest change, so the
    /// engine divides this out; a test pins each value by measuring the formula.
    pub fn peak_factor(self) -> f64 {
        match self {
            StampKind::Ridge | StampKind::Cliff => 1.0,
            StampKind::Canyon | StampKind::Volcano => 1.5,
            StampKind::Mesa => 1.25,
        }
    }
}

/// An operation with its shape controls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditOp {
    /// What it does.
    pub op: Op,
    /// The ramp's shape (the sculpt tools' own).
    pub falloff: Falloff,
    /// 0..1: the ramp's width as a share of the region's depth (see the module docs).
    pub softness: f64,
}

/// The second wall: re-check an operation list against the limits. The model-facing validator
/// already did; this is what the engine call runs, so a caller that skipped the validator still
/// cannot exceed a limit. Returns the first breach in words.
pub fn check_ops(ops: &[EditOp]) -> Result<(), EditError> {
    let bad = |w: String| Err(EditError::Ops(w));
    if ops.is_empty() {
        return bad("there is no operation to apply".into());
    }
    if ops.len() > MAX_EDIT_OPS {
        return bad(format!("{} operations is more than the {MAX_EDIT_OPS} one edit may hold", ops.len()));
    }
    let mut total = 0.0;
    for o in ops {
        if !o.softness.is_finite() || !(0.0..=1.0).contains(&o.softness) {
            return bad("softness must be a number from 0 to 1".into());
        }
        match o.op {
            Op::Raise { amount_m } | Op::Lower { amount_m } | Op::Stamp { amount_m, .. } => {
                if !amount_m.is_finite() || amount_m <= 0.0 || amount_m > MAX_AMOUNT_M {
                    return bad(format!("an amount must be above 0 and at most {MAX_AMOUNT_M} m"));
                }
                total += amount_m;
            }
            Op::Smooth { strength, passes } => {
                if !strength.is_finite() || strength < STRENGTH_RANGE.0 || strength > STRENGTH_RANGE.1 {
                    return bad(format!("smoothing strength must be from {} to {}", STRENGTH_RANGE.0, STRENGTH_RANGE.1));
                }
                if passes == 0 || passes > MAX_SMOOTH_PASSES {
                    return bad(format!("smoothing passes must be from 1 to {MAX_SMOOTH_PASSES}"));
                }
            }
        }
    }
    if total > MAX_TOTAL_AMOUNT_M {
        return bad(format!("the operations move {total} m in all; at most {MAX_TOTAL_AMOUNT_M} m may be moved at once"));
    }
    Ok(())
}

/// The weight of every mask cell for one operation's shape: `Falloff::coverage(d / feather)` with
/// `d` the chamfer distance to the nearest cell outside the region (see the module docs). Cells
/// outside the region weigh 0.
fn weights(mask: &RegionMask, falloff: Falloff, softness: f64) -> Vec<f32> {
    // Pad by one cell so the bounding box's own border counts as "outside".
    let (pw, ph) = (mask.w + 2, mask.h + 2);
    let mut outside = vec![1u8; pw * ph];
    for row in 0..mask.h {
        for col in 0..mask.w {
            if mask.cells[row * mask.w + col] {
                outside[(row + 1) * pw + col + 1] = 0;
            }
        }
    }
    let d = crate::infer::chamfer_dist(&outside, pw, ph);
    let deepest = (0..mask.h)
        .flat_map(|r| (0..mask.w).map(move |c| (r, c)))
        .filter(|&(r, c)| mask.cells[r * mask.w + c])
        .map(|(r, c)| d[(r + 1) * pw + c + 1] as f64)
        .fold(0.0f64, f64::max);
    let feather = 1.5f64.max(softness * deepest);
    let mut out = vec![0f32; mask.w * mask.h];
    for row in 0..mask.h {
        for col in 0..mask.w {
            if mask.cells[row * mask.w + col] {
                let dist = d[(row + 1) * pw + col + 1] as f64;
                out[row * mask.w + col] = falloff.coverage(dist / feather) as f32;
            }
        }
    }
    out
}

/// A window of the field: the region's bounding box padded by one cell (so a blur has its
/// neighbours), clipped to the world.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// Left column in the world.
    pub x0: usize,
    /// Top row in the world.
    pub y0: usize,
    /// Width.
    pub w: usize,
    /// Height.
    pub h: usize,
    /// Row-major heights.
    pub data: Vec<f32>,
}

/// Copy the region's padded window out of `field`.
pub fn window(mask: &RegionMask, field: &[f32], gw: usize, gh: usize) -> Window {
    let x0 = mask.x0.saturating_sub(1);
    let y0 = mask.y0.saturating_sub(1);
    let x1 = (mask.x0 + mask.w + 1).min(gw);
    let y1 = (mask.y0 + mask.h + 1).min(gh);
    let (w, h) = (x1 - x0, y1 - y0);
    let mut data = Vec::with_capacity(w * h);
    for y in y0..y1 {
        data.extend_from_slice(&field[y * gw + x0..y * gw + x1]);
    }
    Window { x0, y0, w, h, data }
}

/// What [`apply_ops`] did, for the preview and the bridge's reply.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    /// Cells whose height changed (by at least one `f32` step).
    pub changed: usize,
    /// Largest rise, metres (0 when nothing rose).
    pub max_up_m: f64,
    /// Largest fall, metres, as a positive number (0 when nothing fell).
    pub max_down_m: f64,
    /// Cells that were land and are now below sea level.
    pub became_sea: usize,
    /// Cells that were below sea level and are now land.
    pub became_land: usize,
    /// Cells a raise wanted to lift above the stamp's ceiling and the clamp stopped (so the result is lower than asked).
    pub capped: usize,
    /// Cells a lower wanted to push below 0 and the clamp stopped.
    pub floored: usize,
    /// Cells the per-cell limit ([`MAX_TOTAL_AMOUNT_M`]) held to that many metres of change.
    pub limited: usize,
}

/// Apply `ops` in order to `win` inside `mask`. The ONE function both the preview and the apply
/// call. Writes only mask cells. Refuses what [`check_ops`] refuses and a world with no height
/// scale; on a refusal `win` is untouched.
pub fn apply_ops(win: &mut Window, mask: &RegionMask, ops: &[EditOp], sea: f64, peak_m: f64) -> Result<Applied, EditError> {
    check_ops(ops)?;
    if !(peak_m > 0.0) || !(sea < 1.0) || !peak_m.is_finite() || !sea.is_finite() {
        return Err(EditError::NoScale);
    }
    // Every stamp's geometry is derived BEFORE anything is written, so a region that cannot carry one
    // (a box under a ridge) refuses with the window untouched.
    let mut geos: Vec<Option<StampGeo>> = Vec::with_capacity(ops.len());
    for o in ops {
        geos.push(match o.op {
            Op::Stamp { kind, .. } => Some(stamp_geometry(mask, kind)?),
            _ => None,
        });
    }
    let before = win.data.clone();
    let m_per_unit = peak_m / (1.0 - sea);
    let (mut capped, mut floored) = (0usize, 0usize);
    // Offset of the mask's box inside the window.
    let (ox, oy) = (mask.x0 - win.x0, mask.y0 - win.y0);
    for (oi, o) in ops.iter().enumerate() {
        let wt = weights(mask, o.falloff, o.softness);
        match o.op {
            Op::Stamp { kind, amount_m } => {
                let Some(geo) = geos[oi].as_ref() else { continue };
                let amount = amount_m / m_per_unit / kind.peak_factor();
                let (pts, r): (&[Point], f64) = match geo {
                    StampGeo::Path { pts, r } => (pts, *r),
                    StampGeo::Radial { centre, r } => (std::slice::from_ref(centre), *r),
                };
                for row in 0..mask.h {
                    for col in 0..mask.w {
                        let k = wt[row * mask.w + col] as f64;
                        if k <= 0.0 {
                            continue;
                        }
                        let i = (row + oy) * win.w + col + ox;
                        let h0 = win.data[i] as f64;
                        // The cell's centre, the frame the dabs and the rasteriser use.
                        let (px, py) = ((mask.x0 + col) as f64 + 0.5, (mask.y0 + row) as f64 + 0.5);
                        let hit = nearest_on_stroke(px, py, pts);
                        let Some((is_set, v)) = freehand_shape(kind.mode(), amount, r, hit.sd, hit.dist, h0) else { continue };
                        // `k` is the REGION's edge ramp, so the stamp's change is exactly 0 outside the
                        // mask (the loop never visits those cells) and fades to 0 over the mask's edge.
                        let want = if is_set { h0 + k * (v - h0) } else { h0 + k * v };
                        let nh = clamp_height(want, h0);
                        if want > nh {
                            capped += 1;
                        } else if want < nh {
                            floored += 1;
                        }
                        win.data[i] = nh as f32;
                    }
                }
            }
            Op::Raise { amount_m } | Op::Lower { amount_m } => {
                let sign = if matches!(o.op, Op::Raise { .. }) { 1.0 } else { -1.0 };
                let delta = sign * amount_m / m_per_unit;
                for row in 0..mask.h {
                    for col in 0..mask.w {
                        let k = wt[row * mask.w + col] as f64;
                        if k <= 0.0 {
                            continue;
                        }
                        let i = (row + oy) * win.w + col + ox;
                        let h0 = win.data[i] as f64;
                        let want = h0 + k * delta;
                        let nh = clamp_height(want, h0);
                        if want > nh {
                            capped += 1;
                        } else if want < nh {
                            floored += 1;
                        }
                        win.data[i] = nh as f32;
                    }
                }
            }
            Op::Smooth { strength, passes } => {
                for _ in 0..passes {
                    let snap = win.data.clone();
                    for row in 0..mask.h {
                        for col in 0..mask.w {
                            let k = wt[row * mask.w + col] as f64 * strength;
                            if k <= 0.0 {
                                continue;
                            }
                            let (x, y) = (col + ox, row + oy);
                            let i = y * win.w + x;
                            let c = snap[i];
                            let l = if x > 0 { snap[i - 1] } else { c };
                            let r = if x + 1 < win.w { snap[i + 1] } else { c };
                            let u = if y > 0 { snap[i - win.w] } else { c };
                            let d = if y + 1 < win.h { snap[i + win.w] } else { c };
                            let avg = (l as f64 + r as f64 + u as f64 + d as f64) * 0.25;
                            win.data[i] = clamp_height(c as f64 + (avg - c as f64) * k, c as f64) as f32;
                        }
                    }
                }
            }
        }
    }
    // The per-cell limit: no cell ends further than MAX_TOTAL_AMOUNT_M from where it began.
    let limit = MAX_TOTAL_AMOUNT_M / m_per_unit;
    let mut limited = 0usize;
    for row in 0..mask.h {
        for col in 0..mask.w {
            if !mask.cells[row * mask.w + col] {
                continue;
            }
            let i = (row + oy) * win.w + col + ox;
            let d = win.data[i] as f64 - before[i] as f64;
            if d.abs() > limit {
                win.data[i] = (before[i] as f64 + d.signum() * limit) as f32;
                limited += 1;
            }
        }
    }
    let mut rep = Applied { changed: 0, max_up_m: 0.0, max_down_m: 0.0, became_sea: 0, became_land: 0, capped, floored, limited };
    for row in 0..mask.h {
        for col in 0..mask.w {
            if !mask.cells[row * mask.w + col] {
                continue;
            }
            let i = (row + oy) * win.w + col + ox;
            let (a, b) = (before[i], win.data[i]);
            if a != b {
                rep.changed += 1;
                let dm = (b as f64 - a as f64) * m_per_unit;
                if dm > 0.0 {
                    rep.max_up_m = rep.max_up_m.max(dm);
                } else {
                    rep.max_down_m = rep.max_down_m.max(-dm);
                }
            }
            let (was_land, is_land) = ((a as f64) >= sea, (b as f64) >= sea);
            if was_land && !is_land {
                rep.became_sea += 1;
            } else if !was_land && is_land {
                rep.became_land += 1;
            }
        }
    }
    Ok(rep)
}

/// The geometry a stamp runs on, derived from the region alone (no randomness anywhere).
#[derive(Debug, Clone, PartialEq)]
enum StampGeo {
    /// Ridge, cliff, canyon: the painted dabs' centres in order, with the largest dab radius as `r`.
    Path {
        /// The polyline.
        pts: Vec<Point>,
        /// Brush radius in cells.
        r: f64,
    },
    /// Mesa, volcano: the region's deepest cell, with that depth as `r`.
    Radial {
        /// The centre (a cell centre).
        centre: Point,
        /// Radius in cells.
        r: f64,
    },
}

/// The deepest cell of the mask: the largest chamfer distance to a cell outside it (the box's own
/// border counts as outside), first in row-major order on a tie. Returns `(col, row, depth)` in the
/// mask's own box. Always a mask cell.
fn deepest_cell(mask: &RegionMask) -> (usize, usize, f64) {
    let (pw, ph) = (mask.w + 2, mask.h + 2);
    let mut outside = vec![1u8; pw * ph];
    for row in 0..mask.h {
        for col in 0..mask.w {
            if mask.cells[row * mask.w + col] {
                outside[(row + 1) * pw + col + 1] = 0;
            }
        }
    }
    let d = crate::infer::chamfer_dist(&outside, pw, ph);
    let mut best = (0usize, 0usize, -1.0f64);
    for row in 0..mask.h {
        for col in 0..mask.w {
            if mask.cells[row * mask.w + col] {
                let v = d[(row + 1) * pw + col + 1] as f64;
                if v > best.2 {
                    best = (col, row, v);
                }
            }
        }
    }
    best
}

/// The geometry `kind` runs on for this region, or why the region cannot carry it.
fn stamp_geometry(mask: &RegionMask, kind: StampKind) -> Result<StampGeo, EditError> {
    if kind.follows_path() {
        if mask.boxed || mask.path.is_empty() {
            return Err(EditError::Ops(format!(
                "a {} follows a painted stroke; a drawn box has no path, so paint the region with the brush instead",
                kind.key()
            )));
        }
        return Ok(StampGeo::Path { pts: mask.path.iter().map(|p| Point { x: p[0], y: p[1] }).collect(), r: mask.path_r });
    }
    let (col, row, depth) = deepest_cell(mask);
    Ok(StampGeo::Radial {
        centre: Point { x: (mask.x0 + col) as f64 + 0.5, y: (mask.y0 + row) as f64 + 0.5 },
        // At least 1.5 so a one-cell-deep sliver still has a footprint; at most the depth, so the footprint fits.
        r: depth.max(1.5),
    })
}

/// A digest (FNV-1a 64, 16 hex digits) of what a path stamp is derived from: the painted path's
/// points and brush radius (as bit patterns) and whether a box is involved. The region's id hashes the
/// CELLS only, so the same cells painted in another order give another path under one id; the preview
/// returns this and Apply recomputes it and refuses a mismatch, so the ridge the user saw is the ridge
/// applied. Radial stamps depend on the cells alone and are covered by the id.
pub fn path_digest(mask: &RegionMask) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: u8| h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    eat(u8::from(mask.boxed));
    for b in mask.path_r.to_bits().to_le_bytes() {
        eat(b);
    }
    for p in &mask.path {
        for v in p {
            for b in v.to_bits().to_le_bytes() {
                eat(b);
            }
        }
    }
    format!("{h:016x}")
}

/// Most pieces (4-connected groups of cells) a colour paint may cover: one `Area` mark each, against
/// `colour_paint::MAX_MARKS` 5 000. A labelled judgement (a painted region is one or two blobs; 64
/// means a scatter of dabs is refused with a reason rather than flooding the document).
pub const MAX_PAINT_PIECES: usize = 64;
/// Most vertices of one traced ring, the colour-paint document's own `MAX_POINTS_PER_MARK`
/// (`cartalith_spatial::colour_paint`), pinned equal by a test in the bridge crate.
pub const MAX_RING_POINTS: usize = 4_096;

/// The outline of each 4-connected piece of the mask, traced along cell EDGES, as closed rings in
/// grid-edge coordinates (the corner of cell `(x, y)` is `(x, y)`; multiply by km per cell for the
/// colour-paint document). Collinear vertices are dropped. This is what makes a colour mark exactly
/// the region: a cell centre is half a cell from the ring, so it is inside (coverage 1) or outside
/// (coverage 0) at every zoom, with the antialias the mark's own one pixel.
///
/// **Refuses, with the reason, what one exact ring per piece cannot express:** a piece with a hole (or
/// pinched where two cells touch only at a corner, which the trace reads as a second loop), more than
/// [`MAX_PAINT_PIECES`] pieces, or a ring over [`MAX_RING_POINTS`] vertices. Pieces come in row-major
/// order of their first cell, so the result is deterministic.
pub fn mask_rings(mask: &RegionMask) -> Result<Vec<Vec<[f64; 2]>>, EditError> {
    use std::collections::{BTreeMap, VecDeque};
    let bad = |w: String| Err(EditError::Ops(w));
    let (w, h) = (mask.w, mask.h);
    let mut label = vec![0usize; w * h];
    let mut pieces = 0usize;
    let mut rings: Vec<Vec<[f64; 2]>> = Vec::new();
    for start in 0..w * h {
        if !mask.cells[start] || label[start] != 0 {
            continue;
        }
        pieces += 1;
        if pieces > MAX_PAINT_PIECES {
            return bad(format!("the region is more than {MAX_PAINT_PIECES} separate pieces; paint fewer, larger areas"));
        }
        // Flood the piece (4-connected).
        let mut cells = Vec::new();
        let mut q = VecDeque::from([start]);
        label[start] = pieces;
        while let Some(i) = q.pop_front() {
            cells.push(i);
            let (x, y) = (i % w, i / w);
            let nb = [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)];
            for j in nb.into_iter().flatten() {
                if mask.cells[j] && label[j] == 0 {
                    label[j] = pieces;
                    q.push_back(j);
                }
            }
        }
        // Directed boundary edges with the piece on the RIGHT (clockwise on a y-down grid), keyed by
        // their start vertex; a vertex may start two edges where two cells touch at a corner.
        let inside = |x: i64, y: i64| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && label[y as usize * w + x as usize] == pieces;
        let mut out: BTreeMap<(i64, i64), Vec<(i64, i64)>> = BTreeMap::new();
        for &i in &cells {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            if !inside(x, y - 1) {
                out.entry((x, y)).or_default().push((x + 1, y));
            }
            if !inside(x + 1, y) {
                out.entry((x + 1, y)).or_default().push((x + 1, y + 1));
            }
            if !inside(x, y + 1) {
                out.entry((x + 1, y + 1)).or_default().push((x, y + 1));
            }
            if !inside(x - 1, y) {
                out.entry((x, y + 1)).or_default().push((x, y));
            }
        }
        let total_edges: usize = out.values().map(Vec::len).sum();
        // Trace the first loop from the smallest start vertex, turning RIGHT first at a corner contact
        // (hugging the piece), so a second loop means a hole or a pinch.
        let first = *out.keys().next().expect("a piece has boundary edges");
        let mut ring: Vec<(i64, i64)> = vec![first];
        let mut prev_dir = (0i64, 0i64);
        let mut cur = first;
        let mut used = 0usize;
        loop {
            let Some(cands) = out.get_mut(&cur) else { return bad("the region's outline could not be traced".into()) };
            let pick = (0..cands.len())
                .max_by_key(|&k| {
                    let d = (cands[k].0 - cur.0, cands[k].1 - cur.1);
                    prev_dir.0 * d.1 - prev_dir.1 * d.0
                })
                .expect("a vertex on the trace has an outgoing edge");
            let next = cands.swap_remove(pick);
            used += 1;
            prev_dir = (next.0 - cur.0, next.1 - cur.1);
            cur = next;
            if cur == first {
                break;
            }
            ring.push(cur);
            if used > total_edges {
                return bad("the region's outline could not be traced".into());
            }
        }
        if used != total_edges {
            return bad("the region has a hole, or two parts that touch only at a corner; one exact outline cannot express that, so paint a solid area".into());
        }
        // Drop collinear vertices.
        let n = ring.len();
        let mut pts: Vec<[f64; 2]> = Vec::new();
        for k in 0..n {
            let (a, b, c) = (ring[(k + n - 1) % n], ring[k], ring[(k + 1) % n]);
            if (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0) != 0 {
                pts.push([(mask.x0 as i64 + b.0) as f64, (mask.y0 as i64 + b.1) as f64]);
            }
        }
        if pts.len() > MAX_RING_POINTS {
            return bad(format!("the region's outline has {} corners; at most {MAX_RING_POINTS} are accepted, so paint a simpler shape", pts.len()));
        }
        rings.push(pts);
    }
    Ok(rings)
}

/// Write the window's mask cells back into `field`. Only mask cells are written (the one-cell
/// padding is never touched), so what changes outside the region is exactly nothing.
pub fn write_back(win: &Window, mask: &RegionMask, field: &mut [f32], gw: usize) {
    let (ox, oy) = (mask.x0 - win.x0, mask.y0 - win.y0);
    for row in 0..mask.h {
        for col in 0..mask.w {
            if mask.cells[row * mask.w + col] {
                field[(mask.y0 + row) * gw + mask.x0 + col] = win.data[(row + oy) * win.w + col + ox];
            }
        }
    }
}

/// The shape of the region as run-length rows, for the model (numbers only). Every
/// `ceil(h / max_rows)`-th cell row of the bounding box is described; each described row lists the
/// x ranges `(first, last)` (inclusive, in world cells) of its region cells. A row with more than
/// `max_runs` runs has its smallest gaps closed until it fits, and `approximate` is true then (a
/// lossy description that says so). Returns `(row_step, rows, approximate)`; rows with no region cell
/// are skipped, so a sparse region may list fewer than `max_rows` rows.
pub fn run_rows(mask: &RegionMask, max_rows: usize, max_runs: usize) -> (usize, Vec<(usize, Vec<(usize, usize)>)>, bool) {
    let step = mask.h.div_ceil(max_rows.max(1)).max(1);
    let mut approximate = false;
    let mut rows = Vec::new();
    let mut row = 0;
    while row < mask.h {
        let line = &mask.cells[row * mask.w..(row + 1) * mask.w];
        let mut runs: Vec<(usize, usize)> = Vec::new();
        let mut start: Option<usize> = None;
        for (c, &on) in line.iter().enumerate() {
            match (on, start) {
                (true, None) => start = Some(c),
                (false, Some(s)) => {
                    runs.push((mask.x0 + s, mask.x0 + c - 1));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            runs.push((mask.x0 + s, mask.x0 + mask.w - 1));
        }
        while runs.len() > max_runs.max(1) {
            // Close the smallest gap (the first of equals), merging the two runs around it.
            let (i, _) = runs.windows(2).enumerate().map(|(i, w)| (i, w[1].0 - w[0].1)).min_by_key(|&(_, g)| g).unwrap_or((0, 0));
            let merged = (runs[i].0, runs[i + 1].1);
            runs[i] = merged;
            runs.remove(i + 1);
            approximate = true;
        }
        if !runs.is_empty() {
            rows.push((mask.y0 + row, runs));
        }
        row += step;
    }
    (step, rows, approximate)
}

/// The 64-pixel tile indexes (`ty * tiles_x + tx`, `tiles_x = ceil(gw / 64)`) the region touches,
/// for the staleness graph (the sculpt commit marks tiles the same way; every producer of tile ids
/// tiles at 64 px, see `WorldGen::mark_and_recompute`).
pub fn tiles_touched(mask: &RegionMask, gw: usize) -> Vec<usize> {
    const TILE: usize = 64;
    let tiles_x = gw.div_ceil(TILE);
    let mut out = Vec::new();
    for ty in mask.y0 / TILE..=(mask.y0 + mask.h - 1) / TILE {
        for tx in mask.x0 / TILE..=(mask.x0 + mask.w - 1) / TILE {
            out.push(ty * tiles_x + tx);
        }
    }
    out
}

/// A small RGB thumbnail of one window state: height as a grey ramp over `[lo, hi]` (shared by the
/// before and after pictures so they are comparable), cells below sea tinted blue, cells outside
/// the region dimmed. At most `max_side` pixels on the long side, nearest-sample. Returns
/// `(width, height, rgb)`.
pub fn thumbnail(win: &Window, mask: &RegionMask, sea: f64, lo: f32, hi: f32, max_side: usize) -> (usize, usize, Vec<u8>) {
    thumbnail_with(win, mask, sea, lo, hi, max_side, &mut |_, _, _, c| c)
}

/// [`thumbnail`] with a hook that recolours each REGION cell's pixel before the outside is dimmed:
/// `shade(world_x, world_y, is_water, [r, g, b]) -> [r, g, b]`. The colour-paint preview passes the
/// real compositor here (Ruling FW item 2), so the "after" picture is what the blend does, not a
/// second implementation of it. Outside cells are never passed to `shade`: nothing outside paints.
pub fn thumbnail_with(
    win: &Window,
    mask: &RegionMask,
    sea: f64,
    lo: f32,
    hi: f32,
    max_side: usize,
    shade: &mut dyn FnMut(usize, usize, bool, [u8; 3]) -> [u8; 3],
) -> (usize, usize, Vec<u8>) {
    let (bw, bh) = (mask.w, mask.h);
    let scale = (bw.max(bh) as f64 / max_side.max(1) as f64).max(1.0);
    let (tw, th) = (((bw as f64 / scale).ceil() as usize).max(1), ((bh as f64 / scale).ceil() as usize).max(1));
    let (ox, oy) = (mask.x0 - win.x0, mask.y0 - win.y0);
    let span = (hi - lo).max(1e-9);
    let mut rgb = Vec::with_capacity(tw * th * 3);
    for ty in 0..th {
        for tx in 0..tw {
            let col = ((tx as f64 * scale) as usize).min(bw - 1);
            let row = ((ty as f64 * scale) as usize).min(bh - 1);
            let v = win.data[(row + oy) * win.w + col + ox];
            let g = (((v - lo) / span).clamp(0.0, 1.0) * 200.0 + 40.0) as u8;
            let water = (v as f64) < sea;
            let (mut r, mut gr, mut b) = if water { (g / 3, g / 2, g) } else { (g, g, g) };
            if mask.cells[row * bw + col] {
                [r, gr, b] = shade(mask.x0 + col, mask.y0 + row, water, [r, gr, b]);
            } else {
                r /= 2;
                gr /= 2;
                b /= 2;
            }
            rgb.extend_from_slice(&[r, gr, b]);
        }
    }
    (tw, th, rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(gw: usize, gh: usize, v: f32) -> Vec<f32> {
        vec![v; gw * gh]
    }

    fn rect_spec(x: f64, y: f64, w: f64, h: f64) -> RegionSpec {
        RegionSpec { rect: Some([x, y, w, h]), dabs: vec![] }
    }

    /// Protects the rasteriser's counting and clipping with literals: a 10 x 10 box is exactly 100
    /// cells, a box hanging off the corner is clipped and says so, a shape wholly outside is refused.
    #[test]
    fn a_box_rasterises_to_its_exact_cell_count_and_clips_to_the_world() {
        let m = rasterise(&rect_spec(5.0, 5.0, 10.0, 10.0), 64, 64).unwrap();
        assert_eq!((m.x0, m.y0, m.w, m.h, m.count, m.clipped), (5, 5, 10, 10, 100, false));
        let c = rasterise(&rect_spec(-5.0, -5.0, 10.0, 10.0), 64, 64).unwrap();
        assert_eq!((c.x0, c.y0, c.w, c.h, c.count, c.clipped), (0, 0, 5, 5, 25, true));
        assert_eq!(rasterise(&rect_spec(100.0, 100.0, 10.0, 10.0), 64, 64), Err(EditError::OutsideWorld));
        assert_eq!(rasterise(&RegionSpec::default(), 64, 64), Err(EditError::Empty));
    }

    /// Protects the size limits at their edges with literals. MUTATION: change `MAX_REGION_CELLS`
    /// or `MIN_REGION_CELLS` and one of the four assertions goes red.
    #[test]
    fn the_cell_limits_hold_at_their_edges() {
        assert_eq!(MAX_REGION_CELLS, 250_000);
        assert_eq!(MIN_REGION_CELLS, 16);
        assert!(rasterise(&rect_spec(0.0, 0.0, 4.0, 4.0), 64, 64).is_ok(), "16 cells is the smallest legal region");
        assert_eq!(rasterise(&rect_spec(0.0, 0.0, 15.0, 1.0), 64, 64), Err(EditError::TooSmall(15)));
        assert!(rasterise(&rect_spec(0.0, 0.0, 500.0, 500.0), 600, 600).is_ok(), "250 000 cells is the largest legal region");
        assert_eq!(rasterise(&rect_spec(0.0, 0.0, 501.0, 500.0), 600, 600), Err(EditError::TooLarge(250_500)));
    }

    /// Protects the dab rules: the radius range is the sculpt brush's (6..200), non-finite numbers
    /// and too many dabs are refused, never repaired.
    #[test]
    fn dabs_are_bounded_and_never_repaired() {
        let dab = |x: f64, y: f64, r: f64| RegionSpec { rect: None, dabs: vec![[x, y, r]] };
        assert!(rasterise(&dab(50.0, 50.0, 6.0), 200, 200).is_ok());
        assert!(rasterise(&dab(50.0, 50.0, 200.0), 600, 600).is_ok());
        for r in [5.99, 200.01, f64::NAN, f64::INFINITY, -8.0] {
            assert!(matches!(rasterise(&dab(50.0, 50.0, r), 600, 600), Err(EditError::BadNumber(_))), "radius {r}");
        }
        assert!(matches!(rasterise(&dab(f64::NAN, 5.0, 8.0), 600, 600), Err(EditError::BadNumber(_))));
        let many = RegionSpec { rect: None, dabs: vec![[10.0, 10.0, 6.0]; MAX_DABS + 1] };
        assert_eq!(rasterise(&many, 64, 64), Err(EditError::TooManyDabs(MAX_DABS + 1)));
        let ok = RegionSpec { rect: None, dabs: vec![[10.0, 10.0, 6.0]; MAX_DABS] };
        assert!(rasterise(&ok, 64, 64).is_ok());
    }

    /// A disc of radius 10 holds about pi r^2 = 314 cell centres; the exact counts are pinned as literals so a
    /// change to the centre rule (`+ 0.5`) or the inclusive edge turns it red. A dab centred ON a cell centre
    /// (32.5, 32.5) has lattice points at exactly distance 10 ((+-10, 0), (0, +-10), (+-6, +-8), (+-8, +-6): 12 of
    /// them), so only that case can tell `<=` from `<`: Gauss's count N(10) = 317 inclusive, 305 exclusive.
    #[test]
    fn a_dab_uses_cell_centres_and_an_inclusive_edge() {
        let m = rasterise(&RegionSpec { rect: None, dabs: vec![[32.0, 32.0, 10.0]] }, 64, 64).unwrap();
        assert_eq!(m.count, 316);
        let on_centre = rasterise(&RegionSpec { rect: None, dabs: vec![[32.5, 32.5, 10.0]] }, 64, 64).unwrap();
        assert_eq!(on_centre.count, 317, "the edge is inclusive: the 12 cells exactly 10 away are in");
    }

    fn base_world() -> (Vec<f32>, usize, usize) {
        (flat(64, 64, 0.5), 64, 64)
    }

    /// The core literal: sea 0, peak 1000 m, so 0.5 is 500 m. Raising a box by 100 m at constant
    /// full weight moves every region cell by exactly 0.1 and no other cell at all.
    #[test]
    fn a_constant_raise_moves_exactly_the_region_by_the_stated_metres() {
        let (field, gw, gh) = base_world();
        let mask = rasterise(&rect_spec(10.0, 10.0, 20.0, 20.0), gw, gh).unwrap();
        let mut w = window(&mask, &field, gw, gh);
        let op = EditOp { op: Op::Raise { amount_m: 100.0 }, falloff: Falloff::Constant, softness: 0.0 };
        let rep = apply_ops(&mut w, &mask, &[op], 0.0, 1000.0).unwrap();
        let mut after = field.clone();
        write_back(&w, &mask, &mut after, gw);
        let mut inside = 0;
        for y in 0..gh {
            for x in 0..gw {
                let v = after[y * gw + x];
                if (10..30).contains(&x) && (10..30).contains(&y) {
                    assert!((v - 0.6).abs() < 1e-6, "({x},{y}) = {v}");
                    inside += 1;
                } else {
                    assert_eq!(v, 0.5, "({x},{y}) outside the region changed");
                }
            }
        }
        assert_eq!(inside, 400);
        assert_eq!(rep.changed, 400);
        assert!((rep.max_up_m - 100.0).abs() < 0.01 && rep.max_down_m == 0.0, "{rep:?}");
        assert_eq!((rep.capped, rep.floored, rep.became_sea, rep.became_land), (0, 0, 0, 0));
    }

    /// The metres scale follows the world's own `metersPerUnit`: with sea 0.25 and peak 3000 m one
    /// metre is 0.75/3000 height units, so a 400 m raise adds 0.1 exactly.
    #[test]
    fn the_height_scale_is_the_worlds_own_anchoring() {
        let field = flat(64, 64, 0.5);
        let mask = rasterise(&rect_spec(10.0, 10.0, 20.0, 20.0), 64, 64).unwrap();
        let mut w = window(&mask, &field, 64, 64);
        let op = EditOp { op: Op::Raise { amount_m: 400.0 }, falloff: Falloff::Constant, softness: 0.0 };
        apply_ops(&mut w, &mask, &[op], 0.25, 3000.0).unwrap();
        let i = (20 - w.y0) * w.w + 20 - w.x0;
        assert!((w.data[i] - 0.6).abs() < 1e-6, "{}", w.data[i]);
        assert!((to_metres(0.5, 0.25, 3000.0) - 1000.0).abs() < 1e-9);
    }

    /// A softness of 1 puts the maximum weight only at the deepest cells, so a raised long strip is a
    /// RIDGE: the centre line rises the full amount, the edges rise almost nothing. Literals: the strip
    /// is 40 x 10, the deepest cells are at distance 5.
    #[test]
    fn full_softness_over_a_strip_makes_a_ridge_along_its_middle() {
        let (field, gw, gh) = base_world();
        let mask = rasterise(&rect_spec(10.0, 20.0, 40.0, 10.0), gw, gh).unwrap();
        let mut w = window(&mask, &field, gw, gh);
        let op = EditOp { op: Op::Raise { amount_m: 300.0 }, falloff: Falloff::Linear, softness: 1.0 };
        let rep = apply_ops(&mut w, &mask, &[op], 0.0, 1000.0).unwrap();
        let at = |x: usize, y: usize| w.data[(y - w.y0) * w.w + x - w.x0] as f64 - 0.5;
        assert!(at(30, 24) > 0.2 && at(30, 25) > 0.2, "the crest rows rise about 0.3: {} {}", at(30, 24), at(30, 25));
        assert!(at(30, 20) < 0.07, "the long edge barely moves: {}", at(30, 20));
        assert!(at(30, 24) > 2.0 * at(30, 21), "crest 1.0 against 0.4 of the amount");
        assert!(rep.max_up_m <= 300.0 + 1e-3 && rep.max_up_m > 299.9, "{rep:?}");
    }

    /// Protects the per-cell limit with literals: a spike of 1.0 over 0.0 at peak 1000 m smoothed hard would
    /// fall 1000 m; the limit holds it to 600 m (0.6 units), and the report counts it. MUTATION: remove the
    /// limit pass and the spike falls to 0.0 (or the count is 0).
    #[test]
    fn no_cell_moves_more_than_the_total_cap_whatever_the_operations() {
        let mut field = flat(64, 64, 0.0);
        field[20 * 64 + 20] = 1.0;
        let mask = rasterise(&rect_spec(15.0, 15.0, 10.0, 10.0), 64, 64).unwrap();
        let op = EditOp { op: Op::Smooth { strength: 1.0, passes: 1 }, falloff: Falloff::Constant, softness: 0.0 };
        // At peak 100 m the same smooth falls 100 m, inside the limit, and the limit does nothing.
        let mut small = window(&mask, &field, 64, 64);
        let rep_small = apply_ops(&mut small, &mask, &[op], 0.0, 100.0).unwrap();
        assert_eq!((small.data[(20 - small.y0) * small.w + 20 - small.x0], rep_small.limited), (0.0, 0), "100 m of fall is under the 600 m limit");
        // At peak 1000 m one full smooth pass would take the spike to its neighbours' average, 1000 m of fall;
        // the limit lets it fall only 600 m = 0.6 units.
        let mut w = window(&mask, &field, 64, 64);
        let rep = apply_ops(&mut w, &mask, &[op], 0.0, 1000.0).unwrap();
        let v = w.data[(20 - w.y0) * w.w + 20 - w.x0];
        assert!((v - 0.4).abs() < 1e-6, "the spike fell 0.6, not 1.0: {v}");
        assert_eq!(rep.limited, 1, "{rep:?}");
        assert!((rep.max_down_m - 600.0).abs() < 0.01, "{rep:?}");
    }

    /// Protects the sculpt tools' clamp: a raise cannot pass the stamp's ceiling (`max(1, h0)`), and
    /// the report counts the cells it stopped; a lower cannot pass 0 and counts too.
    #[test]
    fn the_sculpt_clamp_stops_a_raise_at_the_ceiling_and_a_lower_at_zero_and_says_so() {
        let field = flat(64, 64, 0.95);
        let mask = rasterise(&rect_spec(10.0, 10.0, 20.0, 20.0), 64, 64).unwrap();
        let mut w = window(&mask, &field, 64, 64);
        let up = EditOp { op: Op::Raise { amount_m: 500.0 }, falloff: Falloff::Constant, softness: 0.0 };
        let rep = apply_ops(&mut w, &mask, &[up], 0.0, 1000.0).unwrap();
        assert!(w.data.iter().all(|v| *v <= 1.0 + 1e-6));
        assert_eq!(rep.capped, 400);
        let field = flat(64, 64, 0.05);
        let mut w = window(&mask, &field, 64, 64);
        let down = EditOp { op: Op::Lower { amount_m: 500.0 }, falloff: Falloff::Constant, softness: 0.0 };
        let rep = apply_ops(&mut w, &mask, &[down], 0.0, 1000.0).unwrap();
        assert!(w.data.iter().all(|v| *v >= 0.0));
        assert_eq!(rep.floored, 400);
    }

    /// Protects the smoothing: it moves a single spike toward its neighbours, never raises the
    /// maximum, writes only mask cells, and is the stamp's own blur (one pass at full weight on a
    /// spike of 1.0 over 0.0 leaves 0.0 + (1 - 0.0)... the centre becomes the neighbour average 0).
    #[test]
    fn smoothing_pulls_a_spike_toward_its_neighbours_inside_the_region_only() {
        let mut field = flat(64, 64, 0.0);
        field[20 * 64 + 20] = 1.0;
        field[20 * 64 + 40] = 1.0; // a second spike OUTSIDE the region
        let mask = rasterise(&rect_spec(15.0, 15.0, 10.0, 10.0), 64, 64).unwrap();
        let mut w = window(&mask, &field, 64, 64);
        let op = EditOp { op: Op::Smooth { strength: 1.0, passes: 1 }, falloff: Falloff::Constant, softness: 0.0 };
        apply_ops(&mut w, &mask, &[op], 0.0, 100.0).unwrap();
        let mut after = field.clone();
        write_back(&w, &mask, &mut after, 64);
        assert_eq!(after[20 * 64 + 20], 0.0, "the spike took its neighbours' average");
        assert_eq!(after[20 * 64 + 19], 0.25, "a neighbour took a quarter of it");
        assert_eq!(after[20 * 64 + 40], 1.0, "a spike outside the region is untouched");
    }

    /// Protects "preview is what Apply does": the window path and a second run over a fresh window
    /// give bit-identical heights, and write-back changes no cell outside the mask.
    #[test]
    fn preview_and_apply_are_the_same_function_and_leave_the_outside_alone() {
        let mut field = flat(80, 60, 0.4);
        for (i, v) in field.iter_mut().enumerate() {
            *v += ((i * 2654435761) % 1000) as f32 / 20000.0;
        }
        let mask = rasterise(&RegionSpec { rect: Some([20.0, 10.0, 30.0, 25.0]), dabs: vec![[60.0, 40.0, 9.0]] }, 80, 60).unwrap();
        let ops = [
            EditOp { op: Op::Raise { amount_m: 200.0 }, falloff: Falloff::Smooth, softness: 0.6 },
            EditOp { op: Op::Smooth { strength: 0.5, passes: 2 }, falloff: Falloff::Linear, softness: 0.3 },
        ];
        let mut a = window(&mask, &field, 80, 60);
        let mut b = window(&mask, &field, 80, 60);
        let ra = apply_ops(&mut a, &mask, &ops, 0.2, 2000.0).unwrap();
        let rb = apply_ops(&mut b, &mask, &ops, 0.2, 2000.0).unwrap();
        assert_eq!(a, b);
        assert_eq!(ra, rb);
        let mut after = field.clone();
        write_back(&a, &mask, &mut after, 80);
        let mut changed = 0;
        for y in 0..60 {
            for x in 0..80 {
                let in_mask = x >= mask.x0 && x < mask.x0 + mask.w && y >= mask.y0 && y < mask.y0 + mask.h && mask.cells[(y - mask.y0) * mask.w + x - mask.x0];
                if !in_mask {
                    assert_eq!(after[y * 80 + x], field[y * 80 + x], "({x},{y}) outside the mask changed");
                } else if after[y * 80 + x] != field[y * 80 + x] {
                    changed += 1;
                }
            }
        }
        assert_eq!(changed, ra.changed, "the report counts exactly the changed cells");
        assert!(changed > 0, "the edit must change something");
    }

    /// Protects every operation limit with literals, at the boundary and just above it.
    /// MUTATION: change any one constant and an assertion goes red.
    #[test]
    fn the_operation_limits_hold_at_their_edges() {
        let r = |a: f64| EditOp { op: Op::Raise { amount_m: a }, falloff: Falloff::Smooth, softness: 0.5 };
        assert_eq!((MAX_EDIT_OPS, MAX_AMOUNT_M, MAX_TOTAL_AMOUNT_M, MAX_SMOOTH_PASSES), (3, 500.0, 600.0, 8));
        assert!(check_ops(&[r(500.0)]).is_ok());
        assert!(check_ops(&[r(500.1)]).is_err());
        assert!(check_ops(&[r(0.0)]).is_err());
        assert!(check_ops(&[r(-1.0)]).is_err());
        assert!(check_ops(&[r(f64::NAN)]).is_err());
        assert!(check_ops(&[r(300.0), r(300.0)]).is_ok());
        assert!(check_ops(&[r(300.0), r(300.1)]).is_err(), "600.1 m in all");
        assert!(check_ops(&[r(1.0), r(1.0), r(1.0)]).is_ok());
        assert!(check_ops(&[r(1.0), r(1.0), r(1.0), r(1.0)]).is_err());
        assert!(check_ops(&[]).is_err());
        let s = |st: f64, p: u32| EditOp { op: Op::Smooth { strength: st, passes: p }, falloff: Falloff::Smooth, softness: 0.5 };
        assert!(check_ops(&[s(0.05, 1)]).is_ok() && check_ops(&[s(1.0, 8)]).is_ok());
        assert!(check_ops(&[s(0.049, 1)]).is_err() && check_ops(&[s(1.01, 1)]).is_err());
        assert!(check_ops(&[s(0.5, 0)]).is_err() && check_ops(&[s(0.5, 9)]).is_err());
        let soft = |v: f64| EditOp { softness: v, ..r(10.0) };
        assert!(check_ops(&[soft(0.0)]).is_ok() && check_ops(&[soft(1.0)]).is_ok());
        assert!(check_ops(&[soft(1.01)]).is_err() && check_ops(&[soft(-0.01)]).is_err() && check_ops(&[soft(f64::NAN)]).is_err());
    }

    /// A refused edit leaves the window untouched (the engine call's second wall).
    #[test]
    fn a_refused_edit_changes_nothing() {
        let field = flat(64, 64, 0.5);
        let mask = rasterise(&rect_spec(10.0, 10.0, 20.0, 20.0), 64, 64).unwrap();
        let mut w = window(&mask, &field, 64, 64);
        let keep = w.clone();
        let bad = EditOp { op: Op::Raise { amount_m: 501.0 }, falloff: Falloff::Smooth, softness: 0.5 };
        assert!(apply_ops(&mut w, &mask, &[bad], 0.0, 1000.0).is_err());
        assert_eq!(w, keep);
        let ok = EditOp { op: Op::Raise { amount_m: 5.0 }, falloff: Falloff::Smooth, softness: 0.5 };
        assert_eq!(apply_ops(&mut w, &mask, &[ok], 0.0, 0.0), Err(EditError::NoScale));
        assert_eq!(apply_ops(&mut w, &mask, &[ok], 1.0, 1000.0), Err(EditError::NoScale));
        assert_eq!(w, keep);
    }

    /// Protects `write_back`'s own mask rule (defence in depth: `apply_ops` already writes only mask cells, so a
    /// window altered OUTSIDE the mask is the only way to see it). A stray value planted outside the mask in the
    /// window must not reach the field; a value inside must. MUTATION: write every cell and the first assertion
    /// goes red.
    #[test]
    fn write_back_never_writes_outside_the_mask() {
        let field = flat(64, 64, 0.5);
        // A disc: its bounding box has corner cells that are NOT in the region.
        let mask = rasterise(&RegionSpec { rect: None, dabs: vec![[32.0, 32.0, 10.0]] }, 64, 64).unwrap();
        assert!(!mask.cells[0], "the box's top-left corner is outside the disc");
        let mut w = window(&mask, &field, 64, 64);
        let corner = (mask.y0 - w.y0) * w.w + (mask.x0 - w.x0); // a window cell inside the box, outside the mask
        let pad = 0; // the padding cell at the window's own corner (outside the box)
        let inside = (32 - w.y0) * w.w + 32 - w.x0;
        w.data[corner] = 0.9;
        w.data[pad] = 0.8;
        w.data[inside] = 0.7;
        let mut f = field.clone();
        write_back(&w, &mask, &mut f, 64);
        assert_eq!(f[mask.y0 * 64 + mask.x0], 0.5, "a box cell outside the mask is not the region's");
        assert_eq!(f[w.y0 * 64 + w.x0], 0.5, "the padding is not the region's");
        assert_eq!(f[32 * 64 + 32], 0.7);
    }

    /// Protects `softness`: with Linear and softness 0 a cell two deep is already at full weight (the ramp is
    /// 1.5 cells wide); with softness 1 the same cell, in a region 5 deep, gets 2/5 of the amount. MUTATION:
    /// ignore softness and the two cases become equal.
    #[test]
    fn softness_sets_the_ramp_width_as_a_share_of_the_region_depth() {
        let field = flat(64, 64, 0.5);
        let mask = rasterise(&rect_spec(10.0, 20.0, 40.0, 10.0), 64, 64).unwrap();
        let rise_at_depth_two = |soft: f64| {
            let mut w = window(&mask, &field, 64, 64);
            let op = EditOp { op: Op::Raise { amount_m: 100.0 }, falloff: Falloff::Linear, softness: soft };
            apply_ops(&mut w, &mask, &[op], 0.0, 1000.0).unwrap();
            (w.data[(21 - w.y0) * w.w + 30 - w.x0] as f64 - 0.5) * 1000.0
        };
        assert!((rise_at_depth_two(0.0) - 100.0).abs() < 0.01, "{}", rise_at_depth_two(0.0));
        assert!((rise_at_depth_two(1.0) - 40.0).abs() < 0.01, "{}", rise_at_depth_two(1.0));
    }

    /// The statistics are literal on a built field: a region whose left half is 0.4 and right half 0.6
    /// with sea 0.5 and peak 1000 m has min -200 m, max 200 m, mean 0, land fraction one half.
    #[test]
    fn stats_report_metres_land_fraction_and_slope() {
        let mut field = flat(64, 64, 0.4);
        for y in 0..64 {
            for x in 32..64 {
                field[y * 64 + x] = 0.6;
            }
        }
        let mask = rasterise(&rect_spec(24.0, 10.0, 16.0, 16.0), 64, 64).unwrap();
        let s = stats(&mask, &field, 64, 64, 0.5, 1000.0, Some(2.0)).unwrap();
        assert_eq!(s.count, 256);
        assert!((s.min_m + 200.0).abs() < 1e-3 && (s.max_m - 200.0).abs() < 1e-3 && s.mean_m.abs() < 1e-3, "{s:?}");
        assert_eq!(s.land_fraction, 0.5);
        // The step is 0.2 units over two cells at x = 31/32: each of the two columns beside the step reads 0.1 per cell
        // = 0.1 x 2000 m/unit = 200 m per cell, over 2 km a cell = 100 m/km, in 2 of 16 columns.
        let slope = s.mean_slope_m_per_km.unwrap();
        assert!((slope - 100.0 * 2.0 / 16.0).abs() < 1e-3, "{slope}");
        assert!(stats(&mask, &field, 64, 64, 0.5, 1000.0, None).unwrap().mean_slope_m_per_km.is_none(), "no cell size, no slope");
        assert!(stats(&mask, &field[..10], 64, 64, 0.5, 1000.0, None).is_none());
    }

    /// Protects the model-facing description: rows are sampled every `ceil(h / max_rows)` cells, runs are
    /// literal inclusive ranges, a gap-ridden row is merged to the run cap and says it is approximate.
    #[test]
    fn run_rows_describe_the_shape_and_admit_when_lossy() {
        let m = rasterise(&RegionSpec { rect: Some([10.0, 20.0, 10.0, 6.0]), dabs: vec![] }, 64, 64).unwrap();
        let (step, rows, approx) = run_rows(&m, 24, 6);
        assert_eq!((step, approx, rows.len()), (1, false, 6));
        assert_eq!(rows[0], (20, vec![(10, 19)]));
        let (step, rows, _) = run_rows(&m, 3, 6);
        assert_eq!((step, rows.iter().map(|r| r.0).collect::<Vec<_>>()), (2, vec![20, 22, 24]));
        // A comb: eight one-cell teeth with one-cell gaps, runs capped at 3 -> merged and approximate.
        let comb = RegionSpec { rect: None, dabs: vec![[30.0, 30.0, 6.0], [42.0, 30.0, 6.0], [54.0, 30.0, 6.0], [66.0, 30.0, 6.0]] };
        let m2 = rasterise(&comb, 100, 100).unwrap();
        let (_, full, a_full) = run_rows(&m2, 24, 6);
        assert!(!a_full && full.iter().any(|r| r.1.len() == 4), "{full:?}");
        let (_, cut, a_cut) = run_rows(&m2, 24, 2);
        assert!(a_cut && cut.iter().all(|r| r.1.len() <= 2));
        // The merge only ever FILLS gaps: every original cell stays covered.
        for (y, runs) in &cut {
            let orig = &full.iter().find(|r| r.0 == *y).unwrap().1;
            for (a, b) in orig {
                assert!(runs.iter().any(|(x0, x1)| x0 <= a && b <= x1), "row {y}: {a}-{b} lost");
            }
        }
    }

    /// Protects the tile ids: a region straddling the 64 px boundary touches every tile it overlaps.
    #[test]
    fn tiles_touched_lists_every_overlapped_tile() {
        let m = rasterise(&rect_spec(60.0, 60.0, 10.0, 10.0), 200, 200).unwrap();
        // tiles_x = ceil(200 / 64) = 4; the box spans tiles x 0..=1, y 0..=1.
        assert_eq!(tiles_touched(&m, 200), vec![0, 1, 4, 5]);
    }

    /// The thumbnail is bounded and the same scale for both states.
    #[test]
    fn a_thumbnail_is_bounded_and_dims_the_outside() {
        let field = flat(200, 200, 0.5);
        let m = rasterise(&rect_spec(10.0, 10.0, 150.0, 80.0), 200, 200).unwrap();
        let w = window(&m, &field, 200, 200);
        let (tw, th, rgb) = thumbnail(&w, &m, 0.0, 0.0, 1.0, 96);
        assert_eq!((tw, th), (96, 52));
        assert_eq!(rgb.len(), tw * th * 3);
    }

    // ---------------------------------------------------------------- Ruling FW item 1: the stamps

    /// A mask from ASCII art (`#` is in the region) with the given painted path. Literal, so a test can
    /// state exactly which cells are inside; the notch and the L make the edge non-trivial.
    fn art_mask(x0: usize, y0: usize, rows: &[&str], path: &[[f64; 2]], path_r: f64) -> RegionMask {
        let w = rows[0].len();
        let h = rows.len();
        let cells: Vec<bool> = rows.iter().flat_map(|r| r.bytes().map(|b| b == b'#')).collect();
        let count = cells.iter().filter(|c| **c).count();
        RegionMask { x0, y0, w, h, cells, count, clipped: false, path: path.to_vec(), path_r, boxed: false }
    }

    /// 40 x 24 cells at (10, 10): a wide band with a notch cut from its top edge and a corner missing,
    /// the path running through the middle of it.
    fn notched_band() -> RegionMask {
        let mut rows: Vec<String> = Vec::new();
        for y in 0..24 {
            let mut line = String::new();
            for x in 0..40 {
                let notch = (14..20).contains(&x) && y < 9;
                let corner = x >= 34 && y >= 18;
                line.push(if notch || corner { '.' } else { '#' });
            }
            rows.push(line);
        }
        let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        art_mask(10, 10, &refs, &[[14.5, 22.5], [30.5, 22.5], [46.5, 22.5]], 11.0)
    }

    fn hills(gw: usize, gh: usize) -> Vec<f32> {
        (0..gw * gh).map(|i| 0.30 + 0.25 * (((i % gw) as f32 * 0.13).sin() * ((i / gw) as f32 * 0.09).cos()).abs()).collect()
    }

    fn stamp_op(kind: StampKind, amount_m: f64) -> EditOp {
        EditOp { op: Op::Stamp { kind, amount_m }, falloff: Falloff::Smooth, softness: 0.5 }
    }

    /// Protects Ruling FW item 1's bound: EVERY cell outside a literal mask is bit-identical after each of
    /// the five stamps (compared as bit patterns, over the whole world, padding ring included), and the
    /// stamp really changes cells inside. MUTATION: make the stamp branch write every window cell, or
    /// multiply by 1 instead of the region's ramp `k`: the cliff (+-amount at the far edge) then moves
    /// cells outside.
    #[test]
    fn no_stamp_changes_a_cell_outside_the_literal_mask() {
        let (gw, gh) = (80, 60);
        let field = hills(gw, gh);
        let mask = notched_band();
        for kind in StampKind::ALL {
            let mut win = window(&mask, &field, gw, gh);
            let rep = apply_ops(&mut win, &mask, &[stamp_op(kind, 300.0)], 0.0, 1000.0).unwrap();
            let mut after = field.clone();
            write_back(&win, &mask, &mut after, gw);
            let mut inside_changed = 0;
            for y in 0..gh {
                for x in 0..gw {
                    let inside = x >= mask.x0 && x < mask.x0 + mask.w && y >= mask.y0 && y < mask.y0 + mask.h && mask.cells[(y - mask.y0) * mask.w + (x - mask.x0)];
                    let (a, b) = (field[y * gw + x], after[y * gw + x]);
                    if inside {
                        inside_changed += usize::from(a.to_bits() != b.to_bits());
                    } else {
                        assert_eq!(a.to_bits(), b.to_bits(), "{kind:?}: the cell ({x},{y}) OUTSIDE the mask changed");
                    }
                }
            }
            assert!(inside_changed > 50 && rep.changed == inside_changed, "{kind:?}: {inside_changed} inside cells changed, report says {}", rep.changed);
            // The window's one-cell padding ring is not written either.
            for y in 0..win.h {
                for x in 0..win.w {
                    let (wx, wy) = (win.x0 + x, win.y0 + y);
                    let inside = wx >= mask.x0 && wx < mask.x0 + mask.w && wy >= mask.y0 && wy < mask.y0 + mask.h && mask.cells[(wy - mask.y0) * mask.w + (wx - mask.x0)];
                    if !inside {
                        assert_eq!(win.data[y * win.w + x].to_bits(), field[wy * gw + wx].to_bits(), "{kind:?}: window cell ({wx},{wy}) outside the mask moved");
                    }
                }
            }
        }
    }

    /// Protects "the preview is bit for bit the result": two independent evaluations of the same stamp
    /// over fresh window copies agree to the bit, and the stamp has no random input. MUTATION: seed
    /// anything from a clock or a counter and the second run differs.
    #[test]
    fn a_stamp_previews_exactly_what_it_applies() {
        let (gw, gh) = (80, 60);
        let field = hills(gw, gh);
        let mask = notched_band();
        for kind in StampKind::ALL {
            let mut a = window(&mask, &field, gw, gh);
            let mut b = window(&mask, &field, gw, gh);
            let ra = apply_ops(&mut a, &mask, &[stamp_op(kind, 200.0)], 0.0, 1000.0).unwrap();
            let rb = apply_ops(&mut b, &mask, &[stamp_op(kind, 200.0)], 0.0, 1000.0).unwrap();
            assert_eq!(ra, rb, "{kind:?}");
            assert!(a.data.iter().zip(&b.data).all(|(x, y)| x.to_bits() == y.to_bits()), "{kind:?}");
        }
    }

    /// Protects `StampKind::peak_factor`: the model's `amount_m` is the strongest change, so the factor
    /// must be what the sculpt formula's strongest change is, MEASURED from `freehand_shape` itself with
    /// `amount` 1. MUTATION: change any factor in `peak_factor` or any constant of an arm.
    #[test]
    fn the_peak_factors_are_what_the_formulas_measure() {
        let m = |k: StampKind, sd: f64, d: f64, h0: f64| freehand_shape(k.mode(), 1.0, 10.0, sd, d, h0).unwrap();
        assert!((m(StampKind::Ridge, 0.0, 0.0, 0.5).1 - 1.0).abs() < 1e-12);
        assert!((m(StampKind::Cliff, 1.0e9, 1.0e9, 0.5).1 - 1.0).abs() < 1e-9);
        assert!((m(StampKind::Cliff, -1.0e9, 1.0e9, 0.5).1 + 1.0).abs() < 1e-9);
        assert!((m(StampKind::Canyon, 0.0, 0.0, 0.5).1 + 1.5).abs() < 1e-12);
        let (set, top) = m(StampKind::Mesa, 0.0, 0.0, 0.5);
        assert!(set && (top - 0.5 - 1.25).abs() < 1e-12);
        assert!((m(StampKind::Volcano, 0.0, 0.0, 0.5).1 - 1.5).abs() < 1e-12);
        let factors: Vec<f64> = StampKind::ALL.iter().map(|k| k.peak_factor()).collect();
        assert_eq!(factors, [1.0, 1.0, 1.5, 1.25, 1.5]);
    }

    /// Protects the stamp's metres: a ridge of 100 m over a flat world (sea 0, peak 1000 m) raises its
    /// crest by 100 m within one f32 step and no more, and a volcano of 100 m peaks at 100 m, not 150.
    #[test]
    fn a_stamp_amount_is_the_strongest_change_in_metres() {
        let (gw, gh) = (80, 60);
        let field = flat(gw, gh, 0.2);
        let mask = notched_band();
        for kind in [StampKind::Ridge, StampKind::Volcano, StampKind::Mesa] {
            let mut w = window(&mask, &field, gw, gh);
            let rep = apply_ops(&mut w, &mask, &[EditOp { op: Op::Stamp { kind, amount_m: 100.0 }, falloff: Falloff::Constant, softness: 0.0 }], 0.0, 1000.0).unwrap();
            assert!(rep.max_up_m > 99.0 && rep.max_up_m < 100.0001, "{kind:?}: strongest rise {} m", rep.max_up_m);
        }
    }

    /// Protects the refusals: a drawn box has no path, so ridge, cliff and canyon refuse it with the
    /// window untouched; mesa and volcano accept it and centre on a cell INSIDE the region. MUTATION:
    /// let `follows_path` return false, or drop the `boxed` test.
    #[test]
    fn a_box_refuses_the_path_stamps_and_carries_the_radial_ones() {
        let (gw, gh) = (64, 64);
        let field = hills(gw, gh);
        let mask = rasterise(&rect_spec(10.0, 10.0, 24.0, 24.0), gw, gh).unwrap();
        assert!(mask.boxed && mask.path.is_empty());
        for kind in [StampKind::Ridge, StampKind::Cliff, StampKind::Canyon] {
            let mut w = window(&mask, &field, gw, gh);
            let before = w.clone();
            let e = apply_ops(&mut w, &mask, &[stamp_op(kind, 100.0)], 0.0, 1000.0).unwrap_err();
            assert!(e.to_string().contains("drawn box has no path"), "{e}");
            assert_eq!(w, before, "{kind:?} wrote before refusing");
        }
        for kind in [StampKind::Mesa, StampKind::Volcano] {
            let StampGeo::Radial { centre, r } = stamp_geometry(&mask, kind).unwrap() else { panic!("radial") };
            let (cx, cy) = (centre.x.floor() as usize, centre.y.floor() as usize);
            assert!(mask.cells[(cy - mask.y0) * mask.w + (cx - mask.x0)], "the centre is a region cell");
            assert!((r - 12.0).abs() < 1.5, "a 24 x 24 box is about 12 deep, got {r}");
            let mut w = window(&mask, &field, gw, gh);
            assert!(apply_ops(&mut w, &mask, &[stamp_op(kind, 100.0)], 0.0, 1000.0).unwrap().changed > 0);
        }
    }

    /// Protects the path digest: the same dabs in the same order give one digest, the same cells painted
    /// in the opposite order another (the region id alone cannot tell them apart). MUTATION: hash the
    /// cells instead of the points, or sort the points.
    #[test]
    fn the_path_digest_follows_the_painted_order() {
        let dabs = vec![[20.0, 30.0, 8.0], [28.0, 30.0, 8.0], [36.0, 30.0, 8.0]];
        let a = rasterise(&RegionSpec { rect: None, dabs: dabs.clone() }, 80, 80).unwrap();
        let b = rasterise(&RegionSpec { rect: None, dabs: dabs.clone() }, 80, 80).unwrap();
        let mut rev = dabs;
        rev.reverse();
        let c = rasterise(&RegionSpec { rect: None, dabs: rev }, 80, 80).unwrap();
        assert_eq!(a.cells, c.cells, "the cells are the same either way");
        assert_eq!(path_digest(&a), path_digest(&b));
        assert_ne!(path_digest(&a), path_digest(&c));
        assert_eq!(path_digest(&a).len(), 16);
    }

    // ---------------------------------------------------------------- Ruling FW item 2: the outline

    /// Protects the trace with literal masks: a 3 x 2 block is one ring of its four corners, an L is six
    /// corners, two blocks are two rings, in grid-edge coordinates offset by the mask's corner. MUTATION:
    /// keep collinear vertices, or drop the `x0`/`y0` offset.
    #[test]
    fn rings_are_the_literal_cell_edge_outlines() {
        let block = art_mask(5, 7, &["###", "###"], &[], 0.0);
        assert_eq!(mask_rings(&block).unwrap(), vec![vec![[5.0, 7.0], [8.0, 7.0], [8.0, 9.0], [5.0, 9.0]]]);
        let ell = art_mask(0, 0, &["#..", "#..", "###"], &[], 0.0);
        let rings = mask_rings(&ell).unwrap();
        assert_eq!(rings.len(), 1);
        let mut got = rings[0].clone();
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut want = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [3.0, 2.0], [3.0, 3.0], [0.0, 3.0]];
        want.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(got, want);
        let two = art_mask(0, 0, &["##.##", "##.##"], &[], 0.0);
        assert_eq!(mask_rings(&two).unwrap().len(), 2);
    }

    /// Protects the refusals: a hole, and a piece pinched at a corner, are named and refused, never
    /// traced into a ring that would paint the hole. MUTATION: skip the `used != total_edges` check.
    #[test]
    fn a_hole_is_refused_with_its_reason() {
        let ring = art_mask(0, 0, &["###", "#.#", "###"], &[], 0.0);
        let e = mask_rings(&ring).unwrap_err().to_string();
        assert!(e.contains("hole"), "{e}");
    }

    /// Protects the piece cap with a literal: 64 pieces trace, 65 are refused. MUTATION: change 64.
    #[test]
    fn the_piece_cap_is_sixty_four() {
        let row = |n: usize| (0..n).map(|i| if i % 2 == 0 { '#' } else { '.' }).collect::<String>();
        let ok = art_mask(0, 0, &[row(127).as_str()], &[], 0.0);
        assert_eq!(ok.cells.iter().filter(|c| **c).count(), 64);
        assert_eq!(mask_rings(&ok).unwrap().len(), 64);
        let over = art_mask(0, 0, &[row(129).as_str()], &[], 0.0);
        assert!(mask_rings(&over).unwrap_err().to_string().contains("64 separate pieces"));
        assert_eq!(MAX_PAINT_PIECES, 64);
    }

    /// Protects the clip-with-a-ramp rule (design s.5i item 2): a cliff is +-amount everywhere, so a hard
    /// cut at the mask edge would leave a wall. Over a flat world the change at cells touching the outside
    /// is a small fraction of the largest change, for every ramp that has a slope. MUTATION: drop `k` from
    /// the stamp branch's `want`.
    #[test]
    fn the_region_ramp_fades_a_stamp_to_nothing_at_the_mask_edge() {
        let (gw, gh) = (80, 60);
        let field = flat(gw, gh, 0.4);
        let mask = notched_band();
        let mut w = window(&mask, &field, gw, gh);
        apply_ops(&mut w, &mask, &[stamp_op(StampKind::Cliff, 300.0)], 0.0, 1000.0).unwrap();
        let (mut edge_max, mut all_max) = (0.0f64, 0.0f64);
        for row in 0..mask.h {
            for col in 0..mask.w {
                if !mask.cells[row * mask.w + col] {
                    continue;
                }
                let d = (w.data[(row + (mask.y0 - w.y0)) * w.w + col + (mask.x0 - w.x0)] as f64 - 0.4).abs();
                all_max = all_max.max(d);
                let on_edge = [(0i64, 1i64), (0, -1), (1, 0), (-1, 0)].iter().any(|(dx, dy)| {
                    let (c, r) = (col as i64 + dx, row as i64 + dy);
                    c < 0 || r < 0 || c >= mask.w as i64 || r >= mask.h as i64 || !mask.cells[r as usize * mask.w + c as usize]
                });
                if on_edge {
                    edge_max = edge_max.max(d);
                }
            }
        }
        assert!(all_max > 0.2, "the cliff is strong inside ({all_max})");
        assert!(edge_max < 0.2 * all_max, "the change at the edge ({edge_max}) is not a small fraction of the largest ({all_max}): a wall");
    }

    /// Protects the box rule for a region that is BOTH a box and painted: the union has a path in `path`
    /// but is still a drawn box, so the path stamps refuse it. MUTATION: drop `mask.boxed ||`.
    #[test]
    fn a_box_plus_paint_still_refuses_the_path_stamps() {
        let (gw, gh) = (64, 64);
        let spec = RegionSpec { rect: Some([10.0, 10.0, 20.0, 20.0]), dabs: vec![[40.0, 40.0, 8.0]] };
        let mask = rasterise(&spec, gw, gh).unwrap();
        assert!(mask.boxed && !mask.path.is_empty());
        let e = stamp_geometry(&mask, StampKind::Ridge).unwrap_err().to_string();
        assert!(e.contains("drawn box has no path"), "{e}");
    }

    /// Protects the radial centre's tie rule with a literal: a 6 x 4 block's deepest cells (depth 2) are
    /// the inner ring-free middle; the FIRST in row-major order wins, column 1, row 1 of the block. MUTATION:
    /// `>=` for `>` in `deepest_cell`.
    #[test]
    fn the_radial_centre_is_the_first_deepest_cell() {
        let block = art_mask(10, 20, &["######", "######", "######", "######"], &[], 0.0);
        let (col, row, depth) = deepest_cell(&block);
        assert_eq!((col, row), (1, 1), "depth {depth}");
        let StampGeo::Radial { centre, .. } = stamp_geometry(&block, StampKind::Volcano).unwrap() else { panic!("radial") };
        assert_eq!((centre.x, centre.y), (11.5, 21.5));
    }
}
