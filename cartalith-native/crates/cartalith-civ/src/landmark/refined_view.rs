//! **Ruling AT (owner, 2026-09-24): the manual "refine viewshed for the
//! current view" action** — `OUTSTANDING_WORK.md`'s *"Ruling 16's manual
//! 'recompute and refine' viewshed action"* row.
//!
//! The coarse viewshed `Derived::build` makes on every landmark run is a
//! whole-map field at a cost-bounded fidelity: a global observer cap
//! ([`VIEW_OBSERVER_CAP`]), roads sampled every [`VIEW_WAY_SAMPLE_KM`], and a
//! horizon clamped in *cells* ([`VIEW_MAX_CELLS`]) so on a fine world it means
//! less than the stated 40 km. This module is the same line-of-sight pass
//! (`cartalith_terrain::analysis::visibility`, Category A, untouched) run **at
//! higher fidelity over the cells in view only**, kept as a value the owner
//! asked for explicitly, saved with the project, and **consumed by scoring
//! only while it is provably still true of the world**.
//!
//! ## What "higher fidelity" is here, each a labelled judgement
//!
//! * **The horizon is the stated 40 km** up to [`REFINE_VIEW_MAX_CELLS`], not
//!   the coarse pass's tighter cell clamp.
//! * **Roads are sampled every [`REFINE_WAY_SAMPLE_KM`]** instead of 10 km, so
//!   a road's contribution is a continuous band rather than overlapping discs.
//!   The per-sample weight is scaled by the same ratio
//!   ([`RefineModel::way_weight`]), so a kilometre of road carries **the same
//!   total weight** as in the coarse field. Without that the refined cells
//!   would read systematically higher than their coarse neighbours and the
//!   rectangle's edge would become a step in the landmark ranking.
//! * **The observer cap is spent only on observers that can reach the view**
//!   (within the horizon of the rectangle) and is [`REFINE_OBSERVER_CAP`],
//!   bounded by a step budget so one press stays interactive.
//!
//! Disclosed, not hidden: scoring normalises each landmark pool by its own
//! min/max, so refined and coarse magnitudes are *comparable* (the weight
//! scaling above) but not *identical*; a larger horizon on a fine world does
//! raise refined values where the coarse horizon was clamped.
//!
//! ## Staleness is a content key, not an event
//!
//! `StageGraph` (`cartalith-spatial::staleness`) already tracks "this stage is
//! older than its upstream", but its version counters live in one session and
//! cannot be saved, and the ruling requires the result to survive a project
//! round trip. The persistent precedent in this tree is
//! `cartalith_io::project::lod_source_key`: an opaque FNV-1a-64 digest of the
//! inputs, recomputed and compared. This follows it, split in two so the
//! status line can say *why*:
//!
//! * `terrain_key` — the height cells the pass read (the view rectangle plus a
//!   horizon-wide margin), the grid shape, wrapping, and the metres-per-cell
//!   and metres-per-unit anchoring. A **sculpt** inside the reach, or a
//!   **regenerate**, changes it; an edit far outside the reach does not,
//!   because it cannot change a cell of this result.
//! * `observer_key` — the observer set the pass built (settlement and
//!   road-sample cells and weights) and the physics it ran with (horizon,
//!   eye, target, Earth radius, sampling). A new town or a rerouted road
//!   changes it.
//!
//! [`RefinedViewshed::status`] recomputes both from the live inputs and
//! compares. Because it is a pure function of the inputs, a stale result can
//! never be read as fresh whichever path changed the world, and a project that
//! is saved and reopened is judged by the same test as one that never closed.
//!
//! ## What this must never do
//!
//! Change the coarse field when no result exists or it is stale
//! (`landmark::Derived::build` gates on `Fresh`); persist a result whose cells
//! disagree with its rectangle ([`RefinedViewshed::from_parts`] refuses); or
//! compute over the whole map when the owner asked for the current view
//! ([`REFINE_MAX_CELLS`] refuses an area the viewport should not be able to
//! produce at a useful zoom).

use super::*;

/// **Category C — roads sampled every 4 km** in the refined pass, against
/// [`VIEW_WAY_SAMPLE_KM`]'s 10. A tenth of the 40 km horizon: consecutive
/// samples are well inside each other's view, so a road reads as a band.
/// Not measured against anything; a labelled judgement.
pub const REFINE_WAY_SAMPLE_KM: f64 = 4.0;

/// The refined horizon's compute bound, in cells — **the number cost is
/// decided by** (`O(r²)` per observer). 416 keeps the stated 40 km true down
/// to a 0.096 km cell, i.e. an 8192-wide 800 km world, where the coarse
/// [`VIEW_MAX_CELLS`] means 10 km. Beyond that the clamp bites and the label
/// quotes the horizon actually in force.
pub const REFINE_VIEW_MAX_CELLS: i64 = 416;

/// **Category C, a cost ceiling.** The most observers one refine will cast,
/// further bounded by [`REFINE_STEP_BUDGET`]. Four times the coarse cap
/// because here the cap is spent on observers that can reach the view, not on
/// the whole map.
pub const REFINE_OBSERVER_CAP: usize = 1024;

/// **Category C, a cost ceiling.** Total ray steps one refine may take:
/// `observers * r * (8r + 4)` (see `visibility`'s own cost note), so a fine,
/// wide-horizon world casts fewer observers rather than taking minutes. 4e8
/// steps is a few seconds at the engine's measured step rate; a labelled
/// judgement, not a measurement of the shipping machine.
pub const REFINE_STEP_BUDGET: u64 = 400_000_000;

/// The largest rectangle (in cells) one refine will take: 2048². The result is
/// stored and serialised cell by cell, and "the current view" at a useful zoom
/// is far below this; a whole 8192² world is not a *view*. Refused with
/// [`RefineError::TooLarge`] so the shell can say "zoom in".
pub const REFINE_MAX_CELLS: usize = 2048 * 2048;

/// A rectangle of grid cells, half-open: columns `x..x+w`, rows `y..y+h`,
/// **inside the grid** (never wrapping across the seam — a view that spans the
/// seam is two rectangles, and the shell clamps to the one on screen).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl ViewRect {
    /// Number of cells.
    pub fn area(&self) -> usize {
        self.w.saturating_mul(self.h)
    }

    /// Whether grid cell `(x, y)` lies inside the rectangle.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    /// Non-empty and entirely inside a `gw x gh` grid.
    pub fn fits(&self, gw: usize, gh: usize) -> bool {
        self.w > 0
            && self.h > 0
            && self.x.checked_add(self.w).is_some_and(|e| e <= gw)
            && self.y.checked_add(self.h).is_some_and(|e| e <= gh)
    }

    /// Whether an observer at `(x, y)` is within Chebyshev distance `r` of any
    /// cell of the rectangle — the observers that can possibly light one of its
    /// cells. Chebyshev because `visibility` casts rays to the perimeter of an
    /// `r`-square. In a wrapping world the column test is circular; when the
    /// rectangle plus both margins covers the whole row, every column is in
    /// reach.
    pub(super) fn within_reach(
        &self,
        x: usize,
        y: usize,
        r: i64,
        gw: usize,
        _gh: usize,
        world: bool,
    ) -> bool {
        let (yi, y0, y1) = (y as i64, self.y as i64, (self.y + self.h) as i64 - 1);
        if yi < y0 - r || yi > y1 + r {
            return false;
        }
        let (xi, x0, x1) = (x as i64, self.x as i64, (self.x + self.w) as i64 - 1);
        if !world {
            return xi >= x0 - r && xi <= x1 + r;
        }
        let g = gw as i64;
        if self.w as i64 + 2 * r >= g {
            return true;
        }
        let d = (xi - x0).rem_euclid(g);
        d <= (self.w as i64 - 1) + r || d >= g - r
    }
}

/// Why a refine was refused. None of these touch the stored result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefineError {
    /// No usable grid: zero-sized, or the field is not `gw * gh` long.
    NoGrid,
    /// No real-world cell size (`width_km` unset), so a horizon in km means
    /// nothing.
    NoCellSize,
    /// The rectangle is empty or not inside the grid.
    BadView,
    /// The rectangle holds more than [`REFINE_MAX_CELLS`] cells.
    TooLarge { cells: usize, max: usize },
    /// The world has no observer at all (no settlement, no visible way): the
    /// `NoTerrain` condition the coarse field has, stated here as a refusal
    /// rather than a field of zeros that would read as "seen from nowhere".
    NoObservers,
}

impl RefineError {
    /// One sentence for the status line.
    pub fn message(&self) -> String {
        match self {
            RefineError::NoGrid => "no terrain grid to analyse".to_string(),
            RefineError::NoCellSize => "the map has no real-world width, so a horizon in km is undefined".to_string(),
            RefineError::BadView => "the view is empty or outside the map".to_string(),
            RefineError::TooLarge { cells, max } => {
                format!("the view is {cells} cells; zoom in to at most {max}")
            }
            RefineError::NoObservers => {
                "nothing to look from: place a settlement or a road first".to_string()
            }
        }
    }
}

/// Whether a stored result is still true of the world, and if not, of what.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefineStatus {
    /// Both keys match the live inputs: scoring may read it.
    Fresh,
    /// The terrain it read changed (sculpt, regenerate, a different grid).
    StaleTerrain,
    /// The observers it looked from changed (settlements, roads).
    StaleObservers,
    /// Both changed.
    StaleBoth,
}

impl RefineStatus {
    /// A stable lowercase token for the shell: `fresh`, `stale_terrain`,
    /// `stale_observers` or `stale_both`.
    pub fn key(&self) -> &'static str {
        match self {
            RefineStatus::Fresh => "fresh",
            RefineStatus::StaleTerrain => "stale_terrain",
            RefineStatus::StaleObservers => "stale_observers",
            RefineStatus::StaleBoth => "stale_both",
        }
    }
}

/// The tunables of one refined pass. `for_inputs` is the shipping model;
/// the fields are public so a test can pin the refined pass to the coarse
/// model and prove the crop-and-overlay machinery equals the whole-map result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefineModel {
    /// Horizon in cells.
    pub radius_cells: i64,
    /// Road sample spacing, km.
    pub way_sample_km: f64,
    /// Weight of one road sample.
    pub way_weight: f32,
    /// Observer cap after the reach filter.
    pub observer_cap: usize,
}

impl RefineModel {
    /// The shipping model for `inp`: [`VIEW_RADIUS_KM`] in cells clamped to
    /// `[VIEW_MIN_CELLS, REFINE_VIEW_MAX_CELLS]`, [`REFINE_WAY_SAMPLE_KM`]
    /// sampling at the coarse weight scaled by the sampling ratio (so a km of
    /// road weighs what it does in the coarse field), and an observer cap from
    /// the step budget.
    pub fn for_inputs(inp: &LandmarkInputs<'_>) -> RefineModel {
        let cell_km = inp.cell_km();
        let radius_cells = if cell_km > 0.0 {
            ((VIEW_RADIUS_KM / cell_km).round() as i64).clamp(VIEW_MIN_CELLS, REFINE_VIEW_MAX_CELLS)
        } else {
            VIEW_MIN_CELLS
        };
        let r = radius_cells.max(1) as u64;
        let per_observer = r * (8 * r + 4);
        let cap = (REFINE_STEP_BUDGET / per_observer).clamp(1, REFINE_OBSERVER_CAP as u64) as usize;
        RefineModel {
            radius_cells,
            way_sample_km: REFINE_WAY_SAMPLE_KM,
            way_weight: (VIEW_WAY_WEIGHT as f64 * REFINE_WAY_SAMPLE_KM / VIEW_WAY_SAMPLE_KM) as f32,
            observer_cap: cap,
        }
    }
}

/// **A refined viewshed over one rectangle**, plus the two content keys that
/// say whether it is still true. Held by `LandmarkStore::refined_view`, saved
/// as an optional member of `entities/landmarks.json`.
#[derive(Clone, Debug, PartialEq)]
pub struct RefinedViewshed {
    /// The view this covers.
    pub rect: ViewRect,
    /// The horizon actually in force, in cells — what [`landmark`'s causal
    /// text](super::Derived::view_reach_label) quotes.
    pub radius_cells: i64,
    /// Observers cast (after the reach filter, the cap and the budget).
    pub observers: usize,
    /// Digest of the terrain the pass read; see the module header.
    pub terrain_key: u64,
    /// Digest of the observers and physics the pass used.
    pub observer_key: u64,
    /// `rect.w * rect.h` raw accumulated observer weights, row-major in the
    /// rectangle — the same unnormalised quantity as `Derived::vis`.
    pub vis: Vec<f32>,
}

/// FNV-1a-64, the digest `cartalith_io::project::lod_source_key` and the
/// workspace's golden tests already use (standard offset basis and prime). Not
/// cryptographic; it defends against a *changed* world, not a forged one.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 ^= x as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn f32(&mut self, v: f32) {
        self.bytes(&v.to_bits().to_le_bytes());
    }
}

/// The window of the height field one result depends on: the rectangle grown
/// by the horizon on every side, clamped to the grid in y, and in x either
/// clamped (flat world) or wrapped (wrapping world). Cutting the field to it
/// is what keeps a refine at `O(view)` memory instead of `O(map)`.
struct Crop {
    /// Global column of sub-grid column 0 (may be negative in a wrapping world).
    x0: i64,
    y0: usize,
    cw: usize,
    ch: usize,
    /// Wrapping world and the window narrower than the row: columns are read
    /// modulo `gw`, and the sub-grid itself does not wrap.
    wrap_cols: bool,
    /// Whether the sub-grid wraps in x: only the full-row window of a wrapping
    /// world, where it *is* the whole row.
    sub_world: bool,
}

impl Crop {
    fn of(rect: &ViewRect, r: i64, gw: usize, gh: usize, world: bool) -> Crop {
        let y0 = (rect.y as i64 - r).max(0) as usize;
        let y1 = ((rect.y + rect.h) as i64 + r).min(gh as i64) as usize;
        let (x0, cw, wrap_cols, sub_world) = if !world {
            let x0 = (rect.x as i64 - r).max(0);
            let x1 = ((rect.x + rect.w) as i64 + r).min(gw as i64);
            (x0, (x1 - x0) as usize, false, false)
        } else if rect.w as i64 + 2 * r >= gw as i64 {
            (0, gw, false, true)
        } else {
            (rect.x as i64 - r, rect.w + 2 * r as usize, true, false)
        };
        Crop { x0, y0, cw, ch: y1 - y0, wrap_cols, sub_world }
    }

    /// Sub-grid column of global column `gx`, `None` if outside the window.
    fn col(&self, gx: usize, gw: usize) -> Option<usize> {
        let d = gx as i64 - self.x0;
        let d = if self.wrap_cols { d.rem_euclid(gw as i64) } else { d };
        (d >= 0 && (d as usize) < self.cw).then_some(d as usize)
    }

    /// The cut-out height field, row-major `cw x ch`.
    fn cut(&self, field: &[f32], gw: usize) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.cw * self.ch);
        for y in self.y0..self.y0 + self.ch {
            for c in 0..self.cw {
                let gx = if self.wrap_cols {
                    (self.x0 + c as i64).rem_euclid(gw as i64) as usize
                } else {
                    (self.x0 + c as i64) as usize
                };
                out.push(field[y * gw + gx]);
            }
        }
        out
    }
}

/// Everything a refine or a freshness check needs, derived once: the cut-out
/// field, the observers (global coordinates) and the two keys.
struct Plan {
    crop: Crop,
    sub: Vec<f32>,
    observers: Vec<analysis::ViewObserver>,
    terrain_key: u64,
    observer_key: u64,
}

/// Whether the world has any observer at all (the `NoTerrain` condition).
fn has_any_observer(inp: &LandmarkInputs<'_>) -> bool {
    inp.settlements.iter().any(|s| s.x < inp.gw && s.y < inp.gh)
        || inp.ways.iter().any(|w| !w.hidden && w.pts.len() >= 2)
}

/// Validate, then derive the plan for `rect` under `model`. Pure in `inp`;
/// both [`compute_with`] and [`RefinedViewshed::status_with`] go through it,
/// so "what was computed" and "what is checked" cannot disagree.
fn plan(
    inp: &LandmarkInputs<'_>,
    rect: &ViewRect,
    model: &RefineModel,
) -> Result<Plan, RefineError> {
    let (gw, gh) = (inp.gw, inp.gh);
    if gw == 0 || gh == 0 || inp.field.len() != gw * gh {
        return Err(RefineError::NoGrid);
    }
    let cell_km = inp.cell_km();
    if !(cell_km > 0.0) {
        return Err(RefineError::NoCellSize);
    }
    if !rect.fits(gw, gh) {
        return Err(RefineError::BadView);
    }
    if rect.area() > REFINE_MAX_CELLS {
        return Err(RefineError::TooLarge { cells: rect.area(), max: REFINE_MAX_CELLS });
    }
    if !has_any_observer(inp) {
        return Err(RefineError::NoObservers);
    }
    let r = model.radius_cells;
    let crop = Crop::of(rect, r, gw, gh, inp.world);
    let sub = crop.cut(inp.field, gw);
    let observers = view_observers_with(
        inp,
        model.way_sample_km,
        model.way_weight,
        model.observer_cap,
        Some((rect, r)),
    );

    // terrain_key: every input of the height read — the window, the shape, the
    // wrapping, and the two scalings that turn field units into metres.
    let mut t = Fnv::new();
    t.u64(gw as u64);
    t.u64(gh as u64);
    t.u64(inp.world as u64);
    t.f64(cell_km);
    t.f64(inp.mpu());
    t.u64(crop.x0 as u64);
    t.u64(crop.y0 as u64);
    t.u64(crop.cw as u64);
    t.u64(crop.ch as u64);
    for v in &sub {
        t.f32(*v);
    }

    // observer_key: the observer list and every physical parameter of the pass.
    let mut o = Fnv::new();
    o.u64(model.radius_cells as u64);
    o.f64(model.way_sample_km);
    o.f32(model.way_weight);
    o.u64(model.observer_cap as u64);
    o.f64(VIEW_EYE_M);
    o.f64(VIEW_TARGET_M);
    o.f64(VIEW_EARTH_RADIUS_M);
    o.u64(observers.len() as u64);
    for ob in &observers {
        o.u64(ob.x as u64);
        o.u64(ob.y as u64);
        o.f32(ob.weight);
    }
    Ok(Plan { crop, sub, observers, terrain_key: t.0, observer_key: o.0 })
}

/// Run the refined pass over `rect` with the shipping [`RefineModel`].
pub fn compute(inp: &LandmarkInputs<'_>, rect: ViewRect) -> Result<RefinedViewshed, RefineError> {
    compute_with(inp, rect, &RefineModel::for_inputs(inp))
}

/// [`compute`] with an explicit model (see [`RefineModel`]).
pub fn compute_with(
    inp: &LandmarkInputs<'_>,
    rect: ViewRect,
    model: &RefineModel,
) -> Result<RefinedViewshed, RefineError> {
    let p = plan(inp, &rect, model)?;
    let gw = inp.gw;
    // Observers into the sub-grid. `within_reach` already kept only those the
    // window holds; the `filter_map` is a guard, not a rule.
    let sub_obs: Vec<analysis::ViewObserver> = p
        .observers
        .iter()
        .filter_map(|o| {
            let c = p.crop.col(o.x, gw)?;
            let y = o.y.checked_sub(p.crop.y0).filter(|&y| y < p.crop.ch)?;
            Some(analysis::ViewObserver { x: c, y, weight: o.weight })
        })
        .collect();
    let cell_km = inp.cell_km();
    let full = analysis::visibility(
        &p.sub,
        p.crop.cw,
        p.crop.ch,
        p.crop.sub_world,
        &sub_obs,
        &analysis::ViewParams {
            radius_cells: model.radius_cells,
            cell_m: cell_km * 1000.0,
            m_per_unit: inp.mpu(),
            eye_m: VIEW_EYE_M,
            target_m: VIEW_TARGET_M,
            earth_radius_m: VIEW_EARTH_RADIUS_M,
        },
    );
    let mut vis = Vec::with_capacity(rect.area());
    for y in rect.y..rect.y + rect.h {
        let sy = y - p.crop.y0;
        for x in rect.x..rect.x + rect.w {
            // The window always holds the rectangle (it is the rectangle grown),
            // so `col` cannot be `None` here.
            let sx = p.crop.col(x, gw).expect("the crop holds the rectangle it was grown from");
            vis.push(full[sy * p.crop.cw + sx]);
        }
    }
    Ok(RefinedViewshed {
        rect,
        radius_cells: model.radius_cells,
        observers: sub_obs.len(),
        terrain_key: p.terrain_key,
        observer_key: p.observer_key,
        vis,
    })
}

impl RefinedViewshed {
    /// Rebuild from stored parts, refusing anything inconsistent: a cell count
    /// that is not `w * h`, a non-finite or negative weight, or a rectangle
    /// that is empty or over [`REFINE_MAX_CELLS`]. A document that fails this
    /// is dropped by the loader with a warning and the project opens as
    /// "never refined", never as a result that disagrees with its own
    /// rectangle.
    pub fn from_parts(
        rect: ViewRect,
        radius_cells: i64,
        observers: usize,
        terrain_key: u64,
        observer_key: u64,
        vis: Vec<f32>,
    ) -> Option<RefinedViewshed> {
        let ok = rect.w > 0
            && rect.h > 0
            && rect.area() <= REFINE_MAX_CELLS
            && radius_cells >= 1
            && vis.len() == rect.area()
            && vis.iter().all(|v| v.is_finite() && *v >= 0.0);
        ok.then_some(RefinedViewshed { rect, radius_cells, observers, terrain_key, observer_key, vis })
    }

    /// Is this still true of `inp`? The shipping check; see the module header.
    pub fn status(&self, inp: &LandmarkInputs<'_>) -> RefineStatus {
        self.status_with(inp, &RefineModel::for_inputs(inp))
    }

    /// [`status`](Self::status) under an explicit model.
    pub fn status_with(&self, inp: &LandmarkInputs<'_>, model: &RefineModel) -> RefineStatus {
        if self.vis.len() != self.rect.area() {
            return RefineStatus::StaleTerrain;
        }
        match plan(inp, &self.rect, model) {
            Ok(p) => match (p.terrain_key == self.terrain_key, p.observer_key == self.observer_key) {
                (true, true) => RefineStatus::Fresh,
                (false, true) => RefineStatus::StaleTerrain,
                (true, false) => RefineStatus::StaleObservers,
                (false, false) => RefineStatus::StaleBoth,
            },
            // The world has no observer left: what it looked from is gone.
            Err(RefineError::NoObservers) => RefineStatus::StaleObservers,
            // A grid that no longer exists or no longer holds the rectangle.
            Err(_) => RefineStatus::StaleTerrain,
        }
    }

    /// Copy the rectangle's cells into the whole-grid field `vis` (row stride
    /// `gw`). A no-op when the rectangle does not fit the field — a guard, as
    /// the caller has already established freshness.
    pub(super) fn overlay_onto(&self, vis: &mut [f32], gw: usize) {
        let r = &self.rect;
        if gw == 0 || r.x + r.w > gw || (r.y + r.h) * gw > vis.len() || self.vis.len() != r.area() {
            return;
        }
        for j in 0..r.h {
            let dst = (r.y + j) * gw + r.x;
            vis[dst..dst + r.w].copy_from_slice(&self.vis[j * r.w..(j + 1) * r.w]);
        }
    }

    /// The two keys as the 16-digit lowercase hex the save format carries
    /// (`SAVEFILE_COMPAT.md` §9.8): `(terrain_key, observer_key)`.
    pub fn keys_hex(&self) -> (String, String) {
        (format!("{:016x}", self.terrain_key), format!("{:016x}", self.observer_key))
    }

    /// Parse a key written by [`keys_hex`](Self::keys_hex); `None` for anything
    /// that is not exactly 16 hex digits.
    pub fn parse_key(s: &str) -> Option<u64> {
        (s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| u64::from_str_radix(s, 16).ok())
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sea level of every fixture (equal to `WorldParams::defaults().sea_level`).
    const SEA: f64 = 0.42;

    /// Deterministic rolling land, everywhere above sea, with enough relief that
    /// a viewshed over it is not uniform: a test that compared two constant
    /// fields would pass whatever the machinery did.
    fn bumpy(gw: usize, gh: usize) -> Vec<f32> {
        (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64, (i / gw) as f64);
                (0.55 + 0.06 * (x * 0.21).sin() * (y * 0.17).cos() + 0.04 * (x * 0.07 + y * 0.05).sin())
                    as f32
            })
            .collect()
    }

    fn way(pts: Vec<(f64, f64)>) -> crate::Way {
        crate::Way {
            tid: 0,
            pts,
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type: crate::WayType::Track,
            a_idx: 0,
            b_idx: 1,
            hidden: false,
        }
    }

    fn site(x: usize, y: usize) -> LandmarkSite {
        LandmarkSite { x, y, population: 5_000.0 }
    }

    /// The pass pinned to the coarse model: same horizon, same sampling, same
    /// weight, same cap — so the only thing left to differ is the crop.
    fn coarse_model(inp: &LandmarkInputs<'_>) -> RefineModel {
        let ctx = Ctx::build(inp, Needs::of("peak"));
        RefineModel {
            radius_cells: ctx.d.r_view,
            way_sample_km: VIEW_WAY_SAMPLE_KM,
            way_weight: VIEW_WAY_WEIGHT,
            observer_cap: VIEW_OBSERVER_CAP,
        }
    }

    /// **The crop-and-overlay machinery is exact.** Pinned to the coarse model,
    /// a refined viewshed over a rectangle equals the whole-map field's cells in
    /// that rectangle **bit for bit** — flat world, a wrapping world with the
    /// rectangle hugging each seam edge, and a rectangle wide enough that the
    /// window is the whole row. If the crop dropped an observer, shifted a
    /// column across the seam or clipped a ray that reaches the view, one cell
    /// would differ.
    #[test]
    fn pinned_to_the_coarse_model_a_refined_view_is_the_coarse_field_cropped() {
        // Protects: refined == coarse inside the rectangle, bit for bit, flat and wrapping
        // (seam-hugging rectangles, and a full-row window), with towns and a seam-crossing way.
        let (gw, gh) = (96usize, 64usize);
        let f = bumpy(gw, gh);
        let towns = [site(10, 12), site(40, 30), site(70, 50), site(90, 8), site(3, 40)];
        for world in [false, true] {
            let ways = [
                way(vec![(5.0, 5.0), (50.0, 20.0), (88.0, 40.0)]),
                if world {
                    // dx = -86: the short way round is across the seam.
                    way(vec![(90.0, 30.0), (4.0, 34.0)])
                } else {
                    way(vec![(10.0, 60.0), (80.0, 58.0)])
                },
            ];
            let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, world, 192.0);
            inp.settlements = &towns;
            inp.ways = &ways;
            let coarse = Ctx::build(&inp, Needs::of("peak")).d.vis;
            assert_eq!(coarse.len(), gw * gh, "positive control: the coarse field exists");
            let model = coarse_model(&inp);
            let rects: &[ViewRect] = if world {
                &[
                    ViewRect { x: 0, y: 5, w: 12, h: 30 },
                    ViewRect { x: 90, y: 5, w: 6, h: 30 },
                    ViewRect { x: 20, y: 5, w: 60, h: 30 },
                    ViewRect { x: 30, y: 10, w: 24, h: 20 },
                ]
            } else {
                &[
                    ViewRect { x: 0, y: 0, w: 12, h: 30 },
                    ViewRect { x: 90, y: 40, w: 6, h: 24 },
                    ViewRect { x: 30, y: 10, w: 24, h: 20 },
                ]
            };
            for rect in rects {
                let rv = compute_with(&inp, *rect, &model).expect("refine");
                assert_eq!(rv.vis.len(), rect.area());
                let mut lit = 0usize;
                for j in 0..rect.h {
                    for i in 0..rect.w {
                        let want = coarse[(rect.y + j) * gw + rect.x + i];
                        let got = rv.vis[j * rect.w + i];
                        assert_eq!(
                            got.to_bits(),
                            want.to_bits(),
                            "world={world} rect={rect:?} cell ({}, {}): {got} vs {want}",
                            rect.x + i,
                            rect.y + j
                        );
                        if want > 0.0 {
                            lit += 1;
                        }
                    }
                }
                assert!(lit > 0, "positive control: the rectangle {rect:?} sees something");
                assert_eq!(rv.status_with(&inp, &model), RefineStatus::Fresh);
            }
        }
    }

    /// One tall hill 24 km from the only town on a 0.2 km grid — where the
    /// coarse horizon is clamped to 104 cells (20.8 km) and the refined one
    /// is the stated 40 km (200 cells). Returns the field and the hill's cell.
    fn hill_world() -> (Vec<f32>, usize, usize, (usize, usize)) {
        let (gw, gh) = (256usize, 256usize);
        let hill = (248usize, 128usize);
        let f: Vec<f32> = (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64, (i / gw) as f64);
                let d2 = (x - hill.0 as f64).powi(2) + (y - hill.1 as f64).powi(2);
                (0.50 + 0.08 * (-d2 / (2.0 * 8.0 * 8.0)).exp()) as f32
            })
            .collect();
        (f, gw, gh, hill)
    }

    /// **The refined pass does what the coarse one cannot, and scoring reads it
    /// only for the covered cells.** The town is 120 cells from the hill; the
    /// coarse horizon is 104, so the coarse field says nobody sees the summit.
    /// A fresh refined view over the hill says the town does, `Derived` takes
    /// that for the cells inside the rectangle and nothing else, and quotes the
    /// horizon in force.
    #[test]
    fn a_fresh_refined_view_lights_what_the_coarse_horizon_cannot_and_only_inside_its_rectangle() {
        // Protects: Ruling AT: a fresh refined result overlays the coarse field for the covered
        // cells only (hill 24 km out: coarse 0.0, refined 1.0), outside is coarse bit for bit, and
        // the causal label says "(refined)" exactly there.
        let (f, gw, gh, hill) = hill_world();
        let towns = [site(128, 128)];
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 51.2);
        inp.settlements = &towns;
        let hi = hill.1 * gw + hill.0;

        let coarse = Ctx::build(&inp, Needs::of("peak"));
        assert_eq!(coarse.d.r_view, 104, "fixture: the coarse horizon is the clamp");
        assert_eq!(coarse.d.vis(hi), 0.0, "fixture: the coarse field cannot see the hill");
        assert!(coarse.d.refined.is_none());

        let rect = ViewRect { x: 200, y: 100, w: 56, h: 56 };
        let rv = compute(&inp, rect).expect("refine");
        assert_eq!(rv.radius_cells, 200, "the stated 40 km at 0.2 km a cell");
        assert_eq!(rv.status(&inp), RefineStatus::Fresh);

        let mut with = inp.clone();
        with.refined_view = Some(&rv);
        let fine = Ctx::build(&with, Needs::of("peak"));
        assert!(fine.d.vis(hi) > 0.0, "the refined horizon reaches the hill");
        assert_eq!(fine.d.vis(hi), 1.0, "one town, weight 1.0");
        for i in 0..gw * gh {
            if !rect.contains(i % gw, i / gw) {
                assert_eq!(
                    fine.d.vis[i].to_bits(),
                    coarse.d.vis[i].to_bits(),
                    "cell {i} is outside the rectangle and must stay coarse"
                );
            }
        }
        let c = inp.cell_km();
        assert_eq!(fine.d.view_reach_label(hi, gw, c), "40 km (refined)");
        assert_eq!(fine.d.view_reach_label(0, gw, c), coarse.d.view_reach_label(0, gw, c));
        assert!(!coarse.d.view_reach_label(hi, gw, c).contains("refined"));
    }

    /// **Stale falls back to coarse, and says why.** A sculpt of one cell the
    /// window reads is `StaleTerrain`; a sculpt beyond the horizon of the
    /// rectangle is not a change to this result and leaves it `Fresh`. A new
    /// town is `StaleObservers`, not terrain.
    #[test]
    fn a_sculpt_or_a_new_town_marks_it_stale_with_the_right_reason() {
        // Protects: Ruling AT: terrain change inside the reach -> StaleTerrain; outside the reach
        // -> still Fresh; a new observer -> StaleObservers; no observer -> StaleObservers.
        let (gw, gh) = (128usize, 40usize);
        let mut f = bumpy(gw, gh);
        let towns = [site(10, 20), site(60, 20)];
        let rect = ViewRect { x: 40, y: 10, w: 20, h: 20 };
        // A 10-cell horizon, so there is terrain beyond the reach.
        let model = RefineModel {
            radius_cells: 10,
            way_sample_km: REFINE_WAY_SAMPLE_KM,
            way_weight: 0.2,
            observer_cap: 64,
        };
        let rv = {
            let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 256.0);
            inp.settlements = &towns;
            compute_with(&inp, rect, &model).expect("refine")
        };
        let status = |f: &[f32], towns: &[LandmarkSite]| {
            let mut inp = LandmarkInputs::new(f, gw, gh, SEA, false, 256.0);
            inp.settlements = towns;
            rv.status_with(&inp, &model)
        };
        assert_eq!(status(&f, &towns), RefineStatus::Fresh);

        // Far outside the window (columns 30..70): column 120 cannot change it.
        let keep = f[20 * gw + 120];
        f[20 * gw + 120] += 0.2;
        assert_eq!(status(&f, &towns), RefineStatus::Fresh, "an edit beyond the reach is not a change");
        f[20 * gw + 120] = keep;
        // Inside the margin (column 33 is within 10 of the rectangle's left edge).
        f[20 * gw + 33] += 0.2;
        assert_eq!(status(&f, &towns), RefineStatus::StaleTerrain);
        f[20 * gw + 33] -= 0.2;
        assert_eq!(status(&f, &towns), RefineStatus::Fresh, "restoring the cell restores freshness");

        // One more town within reach: observers, not terrain.
        let more = [site(10, 20), site(60, 20), site(45, 15)];
        assert_eq!(status(&f, &more), RefineStatus::StaleObservers);
        // A town far outside the reach is not an observer of this result.
        let far = [site(10, 20), site(60, 20), site(120, 35)];
        assert_eq!(status(&f, &far), RefineStatus::Fresh);
        // No observers left at all.
        assert_eq!(status(&f, &[]), RefineStatus::StaleObservers);
    }

    /// Stale scoring is exactly never-refined scoring: `Derived` ignores a
    /// result whose status is not `Fresh`, field and label both.
    #[test]
    fn scoring_ignores_a_stale_refined_view_completely() {
        // Protects: Derived::build gates the overlay on Fresh: with a sculpted (stale) result the
        // vis field equals the never-refined field bit for bit and no label says refined.
        let (mut f, gw, gh, hill) = hill_world();
        let towns = [site(128, 128)];
        let rect = ViewRect { x: 200, y: 100, w: 56, h: 56 };
        let rv = {
            let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 51.2);
            inp.settlements = &towns;
            compute(&inp, rect).unwrap()
        };
        f[50 * gw + 50] += 0.01; // any sculpt: within 200 cells of the rectangle
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 51.2);
        inp.settlements = &towns;
        let never = Ctx::build(&inp, Needs::of("peak"));
        assert_eq!(rv.status(&inp), RefineStatus::StaleTerrain);
        let mut with = inp.clone();
        with.refined_view = Some(&rv);
        let stale = Ctx::build(&with, Needs::of("peak"));
        assert_eq!(stale.d.vis.len(), never.d.vis.len());
        assert!(stale.d.vis.iter().zip(&never.d.vis).all(|(a, b)| a.to_bits() == b.to_bits()));
        assert!(stale.d.refined.is_none());
        let hi = hill.1 * gw + hill.0;
        assert!(!stale.d.view_reach_label(hi, gw, inp.cell_km()).contains("refined"));
    }

    /// **End to end through the store, which is what the shell holds.** The
    /// hill is a peak; with a fresh refined view the chain says the town
    /// watches it (weight 1.0, "(refined)"), without it the coarse chain says
    /// 0.0; `invalidate()` keeps the stored result (Ruling AT says stale, not
    /// deleted); and after a sculpt the stored result is stale and the chain is
    /// coarse again.
    #[test]
    fn the_store_lends_a_fresh_result_keeps_it_across_invalidate_and_stops_using_it_when_stale() {
        // Protects: LandmarkStore::run lends refined_view to the pass; invalidate() does not clear
        // it; a stale one is not used (chain reverts to the coarse weight).
        let (mut f, gw, gh, hill) = hill_world();
        let towns = [site(128, 128)];
        let rect = ViewRect { x: 200, y: 100, w: 56, h: 56 };
        let mut settings = LandmarkSettings::default();
        for k in kinds() {
            settings.set_armed(k.key, k.key == "peak");
        }
        settings.set_cap("peak", 4);
        let chain = |store: &mut LandmarkStore, f: &[f32]| -> String {
            let mut inp = LandmarkInputs::new(f, gw, gh, SEA, false, 51.2);
            inp.settlements = &towns;
            let out = store.run(&inp, 7);
            out.landmarks
                .iter()
                .find(|l| l.kind == "peak" && (l.x, l.y) == hill)
                .unwrap_or_else(|| panic!("the hill is not a placed peak: {:?}", out.landmarks.len()))
                .causal
                .join(" | ")
        };
        let mut store = LandmarkStore { settings, ..Default::default() };
        let never = chain(&mut store, &f);
        assert!(never.contains("at observer weight 0.0"), "{never}");
        assert!(!never.contains("refined"), "{never}");

        store.refined_view = {
            let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 51.2);
            inp.settlements = &towns;
            Some(compute(&inp, rect).unwrap())
        };
        let fresh = chain(&mut store, &f);
        assert!(fresh.contains("at observer weight 1.0"), "{fresh}");
        assert!(fresh.contains("(refined)"), "{fresh}");

        store.invalidate();
        assert!(store.last.is_none());
        assert!(store.refined_view.is_some(), "invalidate must not delete the refined result");

        f[200 * gw + 20] += 0.01; // a sculpt inside the 200-cell reach
        let stale = chain(&mut store, &f);
        assert_eq!(stale, never, "a stale result must read exactly like never having refined");
    }

    /// Every refusal, and that none of them is a field of zeros.
    #[test]
    fn a_refine_that_cannot_be_honest_is_refused_not_zero_filled() {
        // Protects: compute refuses an empty/out-of-grid view, an oversized view, no cell size,
        // a mis-sized field and a world with no observer, rather than returning zeros.
        let (gw, gh) = (64usize, 64usize);
        let f = bumpy(gw, gh);
        let towns = [site(10, 10)];
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, 128.0);
        inp.settlements = &towns;
        let ok = ViewRect { x: 5, y: 5, w: 10, h: 10 };
        assert!(compute(&inp, ok).is_ok());
        for bad in [
            ViewRect { x: 5, y: 5, w: 0, h: 10 },
            ViewRect { x: 60, y: 5, w: 10, h: 10 },
            ViewRect { x: 5, y: 60, w: 10, h: 10 },
            ViewRect { x: usize::MAX, y: 0, w: 2, h: 2 },
        ] {
            assert_eq!(compute(&inp, bad).unwrap_err(), RefineError::BadView, "{bad:?}");
        }
        // Too large: a 4096-cell-wide grid, rectangle over the cap.
        let (bw, bh) = (4096usize, 1100usize);
        let big = vec![0.6f32; bw * bh];
        let mut binp = LandmarkInputs::new(&big, bw, bh, SEA, false, 800.0);
        binp.settlements = &towns;
        let over = ViewRect { x: 0, y: 0, w: bw, h: bh };
        assert!(matches!(compute(&binp, over), Err(RefineError::TooLarge { .. })));
        let mut nowidth = inp.clone();
        nowidth.width_km = 0.0;
        assert_eq!(compute(&nowidth, ok).unwrap_err(), RefineError::NoCellSize);
        let short = &f[..f.len() - 1];
        let mut bad_len = inp.clone();
        bad_len.field = short;
        assert_eq!(compute(&bad_len, ok).unwrap_err(), RefineError::NoGrid);
        let mut none = inp.clone();
        none.settlements = &[];
        assert_eq!(compute(&none, ok).unwrap_err(), RefineError::NoObservers);
        let hidden = [crate::Way { hidden: true, ..way(vec![(1.0, 1.0), (30.0, 30.0)]) }];
        none.ways = &hidden;
        assert_eq!(compute(&none, ok).unwrap_err(), RefineError::NoObservers, "a hidden way is not an observer");
    }

    /// `within_reach` against an independent oracle: for every observer cell on
    /// a small wrapping and a small flat grid, "is any cell of the rectangle
    /// within Chebyshev distance `r`" computed by brute force over the
    /// rectangle's cells with a circular column distance.
    #[test]
    fn reach_matches_a_brute_force_chebyshev_oracle() {
        // Protects: ViewRect::within_reach, including the seam arithmetic and the
        // "window covers the whole row" shortcut.
        let (gw, gh) = (24usize, 12usize);
        for world in [false, true] {
            for r in [2i64, 5, 9] {
                for rect in [
                    ViewRect { x: 0, y: 2, w: 3, h: 4 },
                    ViewRect { x: 21, y: 0, w: 3, h: 12 },
                    ViewRect { x: 10, y: 5, w: 6, h: 2 },
                    ViewRect { x: 2, y: 3, w: 20, h: 5 },
                ] {
                    for y in 0..gh {
                        for x in 0..gw {
                            let mut want = false;
                            for ry in rect.y..rect.y + rect.h {
                                for rx in rect.x..rect.x + rect.w {
                                    let mut dx = (x as i64 - rx as i64).abs();
                                    if world {
                                        dx = dx.min(gw as i64 - dx);
                                    }
                                    let dy = (y as i64 - ry as i64).abs();
                                    if dx <= r && dy <= r {
                                        want = true;
                                    }
                                }
                            }
                            assert_eq!(
                                rect.within_reach(x, y, r, gw, gh, world),
                                want,
                                "world={world} r={r} rect={rect:?} observer ({x}, {y})"
                            );
                        }
                    }
                }
            }
        }
    }

    /// The stored form is validated on the way in, and the hex key codec is
    /// exact.
    #[test]
    fn from_parts_refuses_what_disagrees_with_itself_and_keys_round_trip() {
        // Protects: RefinedViewshed::from_parts rejects a cell count != w*h, NaN/negative cells,
        // an empty rectangle or a zero radius; keys survive hex; overlay_onto copies rows to
        // stride gw and touches nothing outside the rectangle.
        let rect = ViewRect { x: 1, y: 1, w: 2, h: 2 };
        let ok = |v: Vec<f32>| RefinedViewshed::from_parts(rect, 5, 1, 1, 2, v);
        assert!(ok(vec![0.0, 1.0, 0.5, 0.0]).is_some());
        assert!(ok(vec![0.0, 1.0, 0.5]).is_none(), "short");
        assert!(ok(vec![0.0, 1.0, 0.5, 0.0, 0.0]).is_none(), "long");
        assert!(ok(vec![0.0, f32::NAN, 0.5, 0.0]).is_none());
        assert!(ok(vec![0.0, -1.0, 0.5, 0.0]).is_none());
        assert!(RefinedViewshed::from_parts(ViewRect { w: 0, ..rect }, 5, 1, 1, 2, vec![]).is_none());
        assert!(RefinedViewshed::from_parts(rect, 0, 1, 1, 2, vec![0.0; 4]).is_none(), "radius");

        let rv = ok(vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let mut grid = vec![0f32; 16];
        rv.overlay_onto(&mut grid, 4);
        assert_eq!(&grid[5..7], &[1.0, 2.0]);
        assert_eq!(&grid[9..11], &[3.0, 4.0]);
        assert_eq!(grid.iter().filter(|v| **v != 0.0).count(), 4, "nothing outside the rectangle");

        let (t, o) = RefinedViewshed::from_parts(rect, 5, 1, 0x0123_4567_89ab_cdef, 0xff, vec![0.0; 4])
            .unwrap()
            .keys_hex();
        assert_eq!((t.as_str(), o.as_str()), ("0123456789abcdef", "00000000000000ff"));
        assert_eq!(RefinedViewshed::parse_key(&t), Some(0x0123_4567_89ab_cdef));
        for bad in ["", "123", "0123456789abcdeg", "0123456789abcdef0", " 123456789abcdef"] {
            assert_eq!(RefinedViewshed::parse_key(bad), None, "{bad:?}");
        }
    }

    /// The shipping model is what it says: refined road samples carry the same
    /// weight per km as the coarse ones, and the cap respects the step budget.
    #[test]
    fn the_shipping_model_keeps_a_kilometre_of_road_worth_the_same_and_stays_inside_the_budget() {
        // Protects: RefineModel::for_inputs: way_weight / sample km == the coarse weight per km
        // (so refined and coarse cells share a scale), cap * per-observer steps <= budget, and the
        // stated 40 km is true where the coarse clamp makes it 20.
        let f = vec![0.6f32; 4];
        for (gw, width) in [(2048usize, 800.0), (8192, 800.0), (512, 40_000.0)] {
            let mut inp = LandmarkInputs::new(&f, gw, 1, SEA, false, width);
            inp.gw = gw;
            let m = RefineModel::for_inputs(&inp);
            let per_km_refined = m.way_weight as f64 / m.way_sample_km;
            let per_km_coarse = VIEW_WAY_WEIGHT as f64 / VIEW_WAY_SAMPLE_KM;
            assert!((per_km_refined - per_km_coarse).abs() < 1e-7, "{per_km_refined} vs {per_km_coarse}");
            let r = m.radius_cells as u64;
            assert!(m.observer_cap as u64 * r * (8 * r + 4) <= REFINE_STEP_BUDGET.max(r * (8 * r + 4)));
            assert!(m.radius_cells >= VIEW_MIN_CELLS && m.radius_cells <= REFINE_VIEW_MAX_CELLS);
        }
        let mut inp = LandmarkInputs::new(&f, 4096, 1, SEA, false, 800.0);
        inp.gw = 4096;
        assert_eq!(RefineModel::for_inputs(&inp).radius_cells, 205);
    }
}
