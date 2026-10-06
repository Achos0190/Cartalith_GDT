//! **EF-9.2** -- the subdivision rule: which pyramid tiles of a view to draw,
//! and in what order to synthesise them, given per-tile *importance*
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.3, milestone EF-9.2).
//!
//! This is the *engine-side* rule only. **Nothing calls it**: no viewport, no
//! bridge `#[func]`, no generation code (EF-9.4 wires it). It reads scores
//! (`crate::importance`, EF-9.1) and returns a [`Selection`]; it synthesises no
//! texel and moves no golden.
//!
//! # What it replaces, and what "today" is
//!
//! Today's display (`godot-project/shell/viewport_host.gd`, `_update_lod`)
//! draws **one pyramid level `z` for the whole view**:
//!
//! * `z = lod_level_for_zoom(...)` (`cartalith_godot::lod_bridge::level_for_zoom`,
//!   which is `cartalith_spatial::pyramid::pyramid_level_for_zoom`), then
//!   clamped by `lod_max_level()` and by LOD-D6's per-tier `max_level`;
//! * the wanted set is every tile `(z, col, row)` with `col in c0..=c1`,
//!   `row in r0..=r1`, inserted **row-major** (rows outer);
//! * when more tiles are missing than the tier's `tiles_per_update`, `_nearest_tiles`
//!   trims to the closest to the view centre `((c0 + c1) * 0.5, (r0 + r1) * 0.5)`
//!   by squared index distance (at one level the `prefer` group and the
//!   `f` factor are constant). [`centre_distance_order`] is that rule.
//!
//! [`zoom_only_selection`] is that set and that order. Everything below is
//! defined as *today, plus a strictly additive refinement*.
//!
//! # The rule
//!
//! Q1's default (§8.5): **promote only, at most one level, never below `z0`.**
//! A tile at `z0` may be replaced by its four children at `z0 + 1`
//! ([`cartalith_spatial::pyramid::chunk_children`]). Promotions are taken in
//! importance order. Three limits apply, in this order of authority:
//!
//! 1. **`z_cap`** (I3): a promotion is allowed only when `z0 + 1 <= z_cap`.
//!    `z_cap` is the caller's `min(lod_max_level, tier max_level)`.
//! 2. **The threshold** ([`SelectRule`], defaults [`PROMOTE_PERCENT`] and
//!    [`SCORE_FLOOR`]): only the top fraction of the view's *scored* tiles, and
//!    only those strictly above the floor, are eligible. See [`SelectRule`] for
//!    the justification and for which numbers are measured and which are
//!    judgements.
//! 3. **`budget`**: the maximum number of promoted tiles. See
//!    [`select_tiles`].
//!
//! # Why the quadtree is balanced by construction (and what would break that)
//!
//! A selection holds tiles at `z0` and `z0 + 1` only, so any two neighbours
//! differ by at most one level: the 2:1 restriction needs **no adjustment
//! pass**, and none is implemented. The tests prove it on the real output
//! rather than assume it. **If Q1 is ever changed to allow demotion or `+2`,
//! this stops being true and a balance pass becomes mandatory.**
//!
//! # Absent importance is not zero importance
//!
//! `MISTAKES.md`: never encode "no value" as a plausible value. The behaviour,
//! by case, is:
//!
//! * `importance == None` (EF-9 disabled, or no importance computed):
//!   [`Basis::Disabled`], today's selection and order, exactly.
//! * a [`ViewImportance`] built for a different view: [`Basis::Mismatch`], the
//!   same fallback (fail safe = today's behaviour, which is never a regression
//!   against A1).
//! * every score `None` ([`Basis::Absent`]) or every present score equal
//!   ([`Basis::Flat`]): no tile is ranked above another, so nothing is promoted,
//!   and the order is today's -- **unless policy ranks vary** (Q2), which still
//!   raise order.
//! * some scores `None`, some present: a `None` tile is **never promoted** and
//!   is ordered **after** every scored tile (within its policy rank). It is
//!   never treated as score 0.

use crate::importance::{
    importance, policy_rank, synthesis_order, tile_features, ImportanceWeights, PolicyInputs, WorldImportanceField,
};
use cartalith_spatial::pyramid::{chunk_children, pyramid_tile_bounds, ChunkId};

// ------------------------------------------------------------------ constants

/// Percentage of a view's *scored* tiles that may be promoted: **25**.
///
/// **Source: EF-9.0's working point, not a fitted optimum.** The harness
/// (`crates/cartalith-civ/tests/ef9_refinement_gain.rs`) and EF-9.1's fit are
/// both scored at the 25 % budget (Q3's gate), which is where the measured
/// capture (fit 0.579 held out against today's centre-first 0.232) was
/// recorded. Why a fraction at all, when `budget` already caps the count:
/// promoting *every* tile is just a deeper level for the whole view -- that is
/// `level_for_zoom`'s decision, not an importance decision. **Labelled
/// judgement:** the right fraction at other levels or views is unmeasured.
pub const PROMOTE_PERCENT: u32 = 25;

/// A tile is promotable only if its score is **strictly above** this: **0.0**.
///
/// **Labelled judgement, with its basis stated.** [`crate::importance::FITTED_WEIGHTS`]
/// scores a tile as the fitted *centred* `ln G_scr` prediction (a
/// within-(world, level) fixed effect was removed in the fit), so `0.0` reads
/// "predicted refinement gain at the training average". The floor exists
/// because a view of nothing but open sea would otherwise still promote its
/// top 25 %, spending budget where EF-9.0 predicts gain near zero (sea tiles
/// score about `-3` under the shipped weights: `land_fraction` carries most of
/// the weight). **Not measured as a cut-off**: the `> 0` choice has not been
/// swept; only the scoring itself was fitted. It is in the score's own units, so
/// it is only meaningful for scores from `FITTED_WEIGHTS`; a caller using other
/// weights passes its own [`SelectRule`].
pub const SCORE_FLOOR: f64 = 0.0;

/// Highest level a tile may be *promoted to*: **31**. `pyramid_dims` clamps
/// levels to 31; a parent at level 30 has a column below `2^30`, so its
/// children's `col * 2 + 1` fits a `u32`. Above that, promoting could wrap an
/// index, so it is refused (`z_cap` is far lower in practice: `MAX_LEVEL` is 10).
const MAX_TARGET_LEVEL: u32 = 31;

// ---------------------------------------------------------------------- types

/// The tiles a view touches **at `z0`**, as an inclusive index rectangle:
/// `col in c0..=c1`, `row in r0..=r1`. The same quantity `_update_lod` computes
/// (`c0, c1, r0, r1`) and `cartalith_spatial::pyramid::TilesInView` carries.
///
/// A reversed rectangle (`c1 < c0` or `r1 < r0`) is **empty**, not an error.
/// The caller must have clamped it into `[0, 2^z0 - 1]`: nothing here knows the
/// grid, and an index past the level's edge is the caller's bug, not repaired.
/// Must never be used with a view of millions of tiles: the selection is a
/// materialised `Vec`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewRange {
    /// First column (inclusive).
    pub c0: u32,
    /// Last column (inclusive).
    pub c1: u32,
    /// First row (inclusive).
    pub r0: u32,
    /// Last row (inclusive).
    pub r1: u32,
}

impl ViewRange {
    /// Columns in the rectangle; `0` when reversed.
    #[must_use]
    pub fn cols(&self) -> usize {
        if self.c1 >= self.c0 { (self.c1 - self.c0) as usize + 1 } else { 0 }
    }

    /// Rows in the rectangle; `0` when reversed.
    #[must_use]
    pub fn rows(&self) -> usize {
        if self.r1 >= self.r0 { (self.r1 - self.r0) as usize + 1 } else { 0 }
    }

    /// Tile count (`cols * rows`); `0` for an empty view.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cols() * self.rows()
    }

    /// Whether the view holds no tile.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The view centre `((c0 + c1) * 0.5, (r0 + r1) * 0.5)` in `z0` tile-index
    /// space: exactly the `centre` `_update_lod` hands `_nearest_tiles`.
    #[must_use]
    fn centre(&self) -> (f64, f64) {
        ((f64::from(self.c0) + f64::from(self.c1)) * 0.5, (f64::from(self.r0) + f64::from(self.r1)) * 0.5)
    }

    /// The view's `z0` tiles in today's insertion order: rows outer, columns
    /// inner (`for row in r0..=r1: for col in c0..=c1`).
    fn tiles_at(&self, z0: u32) -> Vec<ChunkId> {
        let mut v = Vec::with_capacity(self.len());
        if !self.is_empty() {
            for row in self.r0..=self.r1 {
                for col in self.c0..=self.c1 {
                    v.push(ChunkId::new(z0, col, row));
                }
            }
        }
        v
    }
}

/// One view's importance, **one entry per `z0` tile in row-major order** (the
/// order of [`ViewRange::tiles_at`]), plus the optional Q2 policy ranks.
///
/// `scores[i] == None` means "no importance could be computed for that tile"
/// (EF-9.1's [`crate::importance::Importance::score`] is `None` when every
/// fitted term is absent); it is **never** `Some(0.0)` in disguise and
/// [`select_tiles`] never reads it as zero. `ranks` is parallel; `None` means
/// "no policy information" and is ordered as not-raised, never as raised
/// ([`crate::importance::policy_rank`]).
///
/// Built only through [`Self::new`] / [`Self::from_world`], which fix the
/// lengths, so a half-filled vector cannot reach the rule. It records the view
/// it was built for, so [`select_tiles`] can refuse a stale one.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewImportance {
    view: ViewRange,
    scores: Vec<Option<f64>>,
    ranks: Vec<Option<u8>>,
}

impl ViewImportance {
    /// Wraps scores (and optionally ranks) for `view`.
    ///
    /// Returns `None` -- not a truncated or padded value -- when `scores.len()`
    /// is not `view.len()`, or when `ranks` is `Some` with a different length.
    /// `ranks: None` means the world has no policy layer; every rank is then
    /// `None`.
    #[must_use]
    pub fn new(view: ViewRange, scores: Vec<Option<f64>>, ranks: Option<Vec<Option<u8>>>) -> Option<Self> {
        let n = view.len();
        let ranks = match ranks {
            Some(r) if r.len() == n => r,
            Some(_) => return None,
            None => vec![None; n],
        };
        (scores.len() == n).then_some(Self { view, scores, ranks })
    }

    /// Scores every `z0` tile of `view` from a world's prepared rasters:
    /// `importance(tile_features(src, pyramid_tile_bounds(gw, gh, z0, col, row)), weights)`,
    /// row-major. `policy`, when given, must hold one [`PolicyInputs`] per
    /// tile in the same order and becomes the Q2 ranks via
    /// [`crate::importance::policy_rank`]; it can never alter a score.
    ///
    /// `None` on a length mismatch (`policy`) or an empty grid, as
    /// [`Self::new`].
    #[must_use]
    pub fn from_world(
        src: &WorldImportanceField,
        weights: &ImportanceWeights,
        view: ViewRange,
        z0: u32,
        policy: Option<&[PolicyInputs]>,
    ) -> Option<Self> {
        if src.gw() == 0 || src.gh() == 0 || z0 > MAX_TARGET_LEVEL {
            return None;
        }
        let tiles = view.tiles_at(z0);
        let scores = tiles
            .iter()
            .map(|t| {
                let b = pyramid_tile_bounds(src.gw(), src.gh(), z0 as i32, t.col, t.row);
                importance(&tile_features(src, &b), weights).score
            })
            .collect();
        let ranks = policy.map(|p| p.iter().map(policy_rank).collect());
        Self::new(view, scores, ranks)
    }

    /// The scores, one per `z0` tile, row-major.
    #[must_use]
    pub fn scores(&self) -> &[Option<f64>] {
        &self.scores
    }

    /// The policy ranks, one per `z0` tile, row-major.
    #[must_use]
    pub fn ranks(&self) -> &[Option<u8>] {
        &self.ranks
    }
}

/// The two numbers of the promotion threshold, separated from [`select_tiles`]
/// so tests (and a future owner decision) can vary them without a second code
/// path. [`SelectRule::DEFAULT`] is what [`select_tiles`] uses.
///
/// A tile is **eligible** iff its score is finite and `> score_floor`; of the
/// eligible tiles at most `ceil(top_percent % of the scored tiles)` are taken,
/// highest score first (ties: nearer the view centre, then lower index).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectRule {
    /// Percent of the view's scored tiles that may be promoted.
    pub top_percent: u32,
    /// Strict lower bound on a promotable score.
    pub score_floor: f64,
}

impl SelectRule {
    /// The shipped rule: [`PROMOTE_PERCENT`] and [`SCORE_FLOOR`].
    pub const DEFAULT: SelectRule = SelectRule { top_percent: PROMOTE_PERCENT, score_floor: SCORE_FLOOR };
}

/// Why a [`Selection`] has the shape it has: which branch of the rule ran.
/// Reported so callers and tests can tell *identity* from *refinement*, rather
/// than infer it from the tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    /// `importance` was `None`: today's selection and order.
    Disabled,
    /// A [`ViewImportance`] for a different view was supplied: today's.
    Mismatch,
    /// Every score was `None` (no term could be scored): nothing ranked.
    Absent,
    /// Every present score was equal (uniform importance): nothing ranked.
    Flat,
    /// Scores discriminate between tiles: the rule ran.
    Ranked,
}

/// The result of [`select_tiles`].
///
/// `tiles` is the **exact cover** of the view: each ground point lies in exactly
/// one tile. Tiles are listed row-major over the `z0` tiles, a promoted tile
/// replaced in place by its four children (NW, NE, SW, SE). `order` is a
/// **permutation of `0..tiles.len()`**: `order[0]` is the index in `tiles` to
/// synthesise first. It is complete, not truncated -- cutting it to the tier's
/// `tiles_per_update` (I3) is the caller's, and EF-9.4's, job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The tiles to draw, covering the view exactly once.
    pub tiles: Vec<ChunkId>,
    /// Synthesis priority: indices into `tiles`, first = most urgent.
    pub order: Vec<usize>,
    /// Which branch produced this selection.
    pub basis: Basis,
    /// How many `z0` tiles were replaced by four children each
    /// (`tiles.len() == view.len() + 3 * promoted`).
    pub promoted: usize,
}

// ------------------------------------------------------ shared today-helpers

/// **Today's order, shared**: indices into `pos`, ascending squared distance to
/// `centre`, ties by position in the input (`a.cmp(&b)`).
///
/// This is `viewport_host.gd::_nearest_tiles` at a single level, where the
/// `prefer` group is constant and the `f` factor is 1. It is the *one*
/// definition: EF-9.0's harness (`crates/cartalith-civ/tests/ef9_refinement_gain.rs`)
/// calls it through [`centre_distance_order`] rather than keeping a copy.
/// Godot's `sort_custom` is not guaranteed stable, so tie order in the shell is
/// unspecified; the input-order tie-break here is the deterministic reading.
#[must_use]
pub fn centre_distance_order_at(pos: &[(f64, f64)], centre: (f64, f64)) -> Vec<usize> {
    let d = |i: usize| {
        let (dx, dy) = (pos[i].0 - centre.0, pos[i].1 - centre.1);
        dx * dx + dy * dy
    };
    let mut idx: Vec<usize> = (0..pos.len()).collect();
    idx.sort_by(|&a, &b| d(a).partial_cmp(&d(b)).expect("finite positions").then(a.cmp(&b)));
    idx
}

/// [`centre_distance_order_at`] over integer tile indices `(col, row)`: today's
/// order at one level, in that level's index space.
#[must_use]
pub fn centre_distance_order(tiles: &[(u32, u32)], centre: (f64, f64)) -> Vec<usize> {
    let pos: Vec<(f64, f64)> = tiles.iter().map(|&(c, r)| (f64::from(c), f64::from(r))).collect();
    centre_distance_order_at(&pos, centre)
}

/// Number of tiles a `pct` % budget covers out of `n`: `ceil(n * pct / 100)` in
/// integer arithmetic, so a non-empty set never rounds to zero tiles. Shared
/// with EF-9.0's harness (which measures capture at the same budgets) rather
/// than restated there.
#[must_use]
pub fn budget_count(n: usize, pct: u32) -> usize {
    (n * pct as usize).div_ceil(100)
}

/// **Today's selection** for `view` at `z0`: every tile of the view at `z0`,
/// row-major, in today's centre-distance synthesis order. No importance, no
/// promotion; [`Basis::Disabled`]. What [`select_tiles`] returns whenever
/// EF-9 is off or has nothing to say.
#[must_use]
pub fn zoom_only_selection(view: ViewRange, z0: u32) -> Selection {
    let tiles = view.tiles_at(z0);
    let idx: Vec<(u32, u32)> = tiles.iter().map(|t| (t.col, t.row)).collect();
    let order = centre_distance_order(&idx, view.centre());
    Selection { tiles, order, basis: Basis::Disabled, promoted: 0 }
}

// ------------------------------------------------------------- the rule itself

/// **The subdivision rule** with the shipped threshold ([`SelectRule::DEFAULT`]).
///
/// * `view` -- the `z0` tile rectangle in view (see [`ViewRange`]).
/// * `z0` -- today's level (`level_for_zoom`, already clamped as today clamps
///   it). **Never lowered**: no returned tile is shallower than `z0` (I1).
/// * `z_cap` -- `min(lod_max_level, tier max_level)`. No returned tile is
///   deeper than `max(z0, z_cap)`: a promotion needs `z0 + 1 <= z_cap`, and a
///   `z0` already at or past the cap simply gets no promotion (the caller's
///   own clamp is what keeps `z0 <= z_cap` in the first place).
/// * `budget` -- the **maximum number of promoted tiles** (each promotion adds
///   three tiles, so `tiles.len() <= view.len() + 3 * budget`). This is the
///   quantity EF-9.0 measures (a "25 % budget" is a count of refined tiles).
///   It is *not* `tiles_per_update`: that bounds synthesis *per call* (I3), and
///   the caller enforces it by truncating `order`. `0` allows none.
/// * `importance` -- `None` disables EF-9 and returns [`zoom_only_selection`].
///
/// See the module header for the absent-importance cases. Deterministic: the
/// output is a pure function of the arguments.
#[must_use]
pub fn select_tiles(view: ViewRange, z0: u32, z_cap: u32, budget: usize, importance: Option<&ViewImportance>) -> Selection {
    select_with(&SelectRule::DEFAULT, view, z0, z_cap, budget, importance)
}

/// [`select_tiles`] with an explicit [`SelectRule`]. The body of the rule; kept
/// separate so tests can pin the shipped numbers and exercise others.
#[must_use]
pub fn select_with(
    rule: &SelectRule,
    view: ViewRange,
    z0: u32,
    z_cap: u32,
    budget: usize,
    importance: Option<&ViewImportance>,
) -> Selection {
    let base = zoom_only_selection(view, z0);
    // Identity by control flow: these returns are *today's* selection, built by
    // `zoom_only_selection` and touched by nothing below. Nothing about them
    // is computed from importance.
    let Some(imp) = importance else { return base };
    if imp.view != view || imp.scores.len() != view.len() {
        return Selection { basis: Basis::Mismatch, ..base };
    }
    let finite: Vec<f64> = imp.scores.iter().filter_map(|s| s.filter(|v| v.is_finite())).collect();
    let has_spread = finite.iter().any(|&v| v != finite[0]);
    let ranks_vary = imp.ranks.iter().any(|r| r.unwrap_or(0) != imp.ranks[0].unwrap_or(0));
    if !has_spread && !ranks_vary {
        let basis = if finite.is_empty() { Basis::Absent } else { Basis::Flat };
        return Selection { basis, ..base };
    }

    // ---- which z0 tiles get promoted (scores only: ranks never promote, Q2).
    let n = base.tiles.len();
    let promoted = if has_spread && z0 < z_cap.min(MAX_TARGET_LEVEL) {
        let mut cd_rank = vec![0usize; n];
        for (r, &i) in base.order.iter().enumerate() {
            cd_rank[i] = r;
        }
        let mut cand: Vec<usize> = (0..n).filter(|&i| imp.scores[i].is_some_and(|s| s.is_finite() && s > rule.score_floor)).collect();
        cand.sort_by(|&a, &b| {
            let (sa, sb) = (imp.scores[a].expect("candidate"), imp.scores[b].expect("candidate"));
            // `cd_rank` is a permutation, so this last key is already a total order:
            // an index tie-break would be dead code (a mutation test showed it
            // unreachable), and none is written.
            sb.partial_cmp(&sa).expect("finite").then(cd_rank[a].cmp(&cd_rank[b]))
        });
        let k = budget.min(budget_count(finite.len(), rule.top_percent)).min(cand.len());
        let mut set = vec![false; n];
        for &i in &cand[..k] {
            set[i] = true;
        }
        set
    } else {
        vec![false; n]
    };

    // ---- the exact cover: a promoted tile is replaced in place by its four
    // children; everything else stays at z0. Parallel arrays carry what the
    // order needs per final tile.
    let (mut tiles, mut scores, mut ranks, mut pos) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (i, t) in base.tiles.iter().enumerate() {
        if promoted[i] {
            for k in chunk_children(*t) {
                tiles.push(k);
                scores.push(imp.scores[i]);
                ranks.push(imp.ranks[i]);
                pos.push((f64::from(k.col), f64::from(k.row)));
            }
        } else {
            tiles.push(*t);
            scores.push(imp.scores[i]);
            ranks.push(imp.ranks[i]);
            // a z0 tile's centre in the z0 + 1 index space, where it spans
            // `2c ..= 2c + 1` and so sits at `2c + 0.5` (`_nearest_tiles`'s own
            // `idx * f + (f - 1) / 2` with `f = 2`).
            pos.push((f64::from(t.col) * 2.0 + 0.5, f64::from(t.row) * 2.0 + 0.5));
        }
    }
    // the view centre mapped into the same space (affine, exact for these sizes)
    let (cx, cy) = view.centre();
    let centre = (cx * 2.0 + 0.5, cy * 2.0 + 0.5);

    // ---- the order: policy rank, then importance (None last), then nearer the
    // view centre, then index. `synthesis_order` is the one consumer of ranks
    // and breaks its last tie by index, so the list is first put in
    // centre-distance order and the result mapped back: an index tie in the
    // permuted list *is* a centre-distance tie.
    let perm = centre_distance_order_at(&pos, centre);
    let scores_p: Vec<Option<f64>> = perm.iter().map(|&i| scores[i]).collect();
    let ranks_p: Vec<Option<u8>> = perm.iter().map(|&i| ranks[i]).collect();
    let order = synthesis_order(&scores_p, &ranks_p).into_iter().map(|k| perm[k]).collect();

    let n_promoted = promoted.iter().filter(|&&p| p).count();
    Selection { tiles, order, basis: Basis::Ranked, promoted: n_promoted }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::pyramid::baked_cover;

    /// A view over `cols x rows` tiles starting at `(c0, r0)`.
    fn view(c0: u32, r0: u32, cols: u32, rows: u32) -> ViewRange {
        ViewRange { c0, c1: c0 + cols - 1, r0, r1: r0 + rows - 1 }
    }

    /// Row-major scores for `v` from a function of `(col, row)`.
    fn scored(v: ViewRange, f: impl Fn(u32, u32) -> Option<f64>) -> ViewImportance {
        let mut s = Vec::new();
        for r in v.r0..=v.r1 {
            for c in v.c0..=v.c1 {
                s.push(f(c, r));
            }
        }
        ViewImportance::new(v, s, None).expect("lengths match")
    }

    /// A 6x5 view with all-distinct positive scores (so promotions exist),
    /// strictly increasing with column then row so the best tile is the
    /// bottom-right one: score `1 + col + 10 * row`.
    fn ramp(v: ViewRange) -> ViewImportance {
        scored(v, |c, r| Some(1.0 + f64::from(c) + 10.0 * f64::from(r)))
    }

    /// The tiles of a selection that sit at `z`.
    fn at_level(s: &Selection, z: u32) -> Vec<ChunkId> {
        s.tiles.iter().copied().filter(|t| t.z == z).collect()
    }

    /// Independent restatement of today's wanted set: three nested loops, no
    /// call into the module's own helper.
    fn expected_today(v: ViewRange, z: u32) -> Vec<ChunkId> {
        let mut out = Vec::new();
        for row in v.r0..=v.r1 {
            for col in v.c0..=v.c1 {
                out.push(ChunkId::new(z, col, row));
            }
        }
        out
    }

    /// Independent restatement of today's order: a hand-rolled selection sort on
    /// integer-doubled squared distance (`(2c - (c0 + c1))^2 + ...`), ties by
    /// position. No floats, no `sort_by`, no call into the module.
    fn expected_order(v: ViewRange, tiles: &[ChunkId]) -> Vec<usize> {
        let d = |i: usize| {
            let dx = 2 * i64::from(tiles[i].col) - (i64::from(v.c0) + i64::from(v.c1));
            let dy = 2 * i64::from(tiles[i].row) - (i64::from(v.r0) + i64::from(v.r1));
            dx * dx + dy * dy
        };
        let mut left: Vec<usize> = (0..tiles.len()).collect();
        let mut out = Vec::new();
        while !left.is_empty() {
            let mut best = 0;
            for k in 1..left.len() {
                if d(left[k]) < d(left[best]) {
                    best = k;
                }
            }
            out.push(left.remove(best));
        }
        out
    }

    /// Counts finest-level cells not covered by exactly one selected tile, via
    /// `baked_cover`, plus any area mismatch. `0` means an exact cover of the
    /// view (`finest` = the deepest level present; the view's cells at it are
    /// `c0 << d ..= ((c1 + 1) << d) - 1`).
    fn cover_defects(v: ViewRange, z0: u32, tiles: &[ChunkId]) -> usize {
        let finest = tiles.iter().map(|t| t.z).max().unwrap_or(z0).max(z0);
        let d = finest - z0;
        let mut bad = 0usize;
        for row in (v.r0 << d)..((v.r1 + 1) << d) {
            for col in (v.c0 << d)..((v.c1 + 1) << d) {
                let cell = ChunkId::new(finest, col, row);
                let owners = tiles.iter().filter(|&&t| baked_cover(cell, |k| k == t)).count();
                if owners != 1 {
                    bad += 1;
                }
            }
        }
        // a tile outside the view would add area the cell walk cannot see
        let area: u64 = tiles.iter().map(|t| 4u64.pow(finest - t.z)).sum();
        let want = v.len() as u64 * 4u64.pow(d);
        bad + usize::from(area != want)
    }

    /// Counts 4-neighbour cell pairs (at the finest level) owned by different
    /// tiles whose levels differ by more than one.
    fn balance_violations(v: ViewRange, z0: u32, tiles: &[ChunkId]) -> usize {
        let finest = tiles.iter().map(|t| t.z).max().unwrap_or(z0).max(z0);
        let d = finest - z0;
        let (c0, r0) = (v.c0 << d, v.r0 << d);
        let (w, h) = (v.cols() << d, v.rows() << d);
        let owner_of = |cell: ChunkId| tiles.iter().position(|&t| baked_cover(cell, |k| k == t));
        let mut grid = vec![None; w * h];
        for y in 0..h {
            for x in 0..w {
                grid[y * w + x] = owner_of(ChunkId::new(finest, c0 + x as u32, r0 + y as u32));
            }
        }
        let mut bad = 0;
        for y in 0..h {
            for x in 0..w {
                for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                    if nx < w && ny < h {
                        let (a, b) = (grid[y * w + x].expect("covered"), grid[ny * w + nx].expect("covered"));
                        if a != b && tiles[a].z.abs_diff(tiles[b].z) > 1 {
                            bad += 1;
                        }
                    }
                }
            }
        }
        bad
    }

    /// Protects: the shipped threshold numbers as literals -- 25 % and a `0.0`
    /// floor. The behavioural tests below pin what each does; this one stops a
    /// silent edit to either constant that a lucky fixture would still pass.
    #[test]
    fn the_shipped_threshold_is_the_recorded_literals() {
        assert_eq!(PROMOTE_PERCENT, 25);
        assert_eq!(SCORE_FLOOR, 0.0);
        assert_eq!(SelectRule::DEFAULT, SelectRule { top_percent: 25, score_floor: 0.0 });
        assert_eq!(MAX_TARGET_LEVEL, 31);
    }

    /// Protects: `budget_count` -- `ceil(n * pct / 100)`, never zero for a
    /// non-empty set -- since it sizes both this rule's fraction and the
    /// harness's capture budget (one shared definition).
    #[test]
    fn budget_count_is_a_ceiling_percentage() {
        assert_eq!([10, 25, 50].map(|p| budget_count(5, p)), [1, 2, 3]);
        assert_eq!(budget_count(30, 25), 8);
        assert_eq!(budget_count(4, 25), 1);
        assert_eq!(budget_count(1, 25), 1);
        assert_eq!(budget_count(0, 25), 0);
    }

    /// Protects: `centre_distance_order` being `_nearest_tiles`'s rule at one
    /// level -- the four middle tiles of a 4x4 first (tied, so in input order),
    /// the far corner last -- on a hand-computed case.
    #[test]
    fn centre_distance_order_is_the_nearest_tiles_rule() {
        let tiles: Vec<(u32, u32)> = (0..4).flat_map(|r| (0..4).map(move |c| (c, r))).collect();
        let o = centre_distance_order(&tiles, (1.5, 1.5));
        assert_eq!(o[..4].iter().map(|&i| tiles[i]).collect::<Vec<_>>(), vec![(1, 1), (2, 1), (1, 2), (2, 2)]);
        assert_eq!(tiles[*o.last().unwrap()], (3, 3));
    }

    /// Protects: **identity by control flow** (§8.3). With EF-9 disabled
    /// (`None`), the selection's tiles and order are *equal* to today's
    /// zoom-only set and centre-distance order, each built here by independent
    /// loops (`expected_today`, `expected_order`), over square, wide and
    /// one-row views at several levels, and the branch reported is
    /// `Basis::Disabled` with nothing promoted -- even with a generous budget
    /// and a cap that would allow promotion.
    #[test]
    fn disabled_is_exactly_todays_selection_and_order() {
        for (v, z0) in [(view(2, 3, 5, 4), 3u32), (view(0, 0, 1, 1), 0), (view(7, 1, 9, 1), 4), (view(1, 1, 3, 6), 5)] {
            let s = select_tiles(v, z0, 10, 1000, None);
            let want_tiles = expected_today(v, z0);
            assert_eq!(s.tiles, want_tiles, "tiles {v:?} z{z0}");
            assert_eq!(s.order, expected_order(v, &want_tiles), "order {v:?} z{z0}");
            assert_eq!((s.basis, s.promoted), (Basis::Disabled, 0));
        }
    }

    /// Protects: identity under **uniform importance**: every score equal
    /// (any value, positive or not) and no policy ranks gives today's set and
    /// order, tiles and order *equal* to the independent construction, branch
    /// `Flat`; and an all-`None` importance (the absent case) does the same,
    /// branch `Absent` -- absent is not read as zero and not as "promote".
    #[test]
    fn uniform_or_absent_importance_is_exactly_todays_selection_and_order() {
        let v = view(2, 3, 5, 4);
        let want_tiles = expected_today(v, 3);
        let want_order = expected_order(v, &want_tiles);
        for flat in [5.0, 0.0, -3.0, 1e9] {
            let s = select_tiles(v, 3, 10, 1000, Some(&scored(v, |_, _| Some(flat))));
            assert_eq!((s.tiles.clone(), s.order.clone()), (want_tiles.clone(), want_order.clone()), "flat {flat}");
            assert_eq!((s.basis, s.promoted), (Basis::Flat, 0));
        }
        let none = select_tiles(v, 3, 10, 1000, Some(&scored(v, |_, _| None)));
        assert_eq!((none.tiles, none.order, none.basis), (want_tiles, want_order, Basis::Absent));
    }

    /// Protects: a stale or malformed importance falling back to today's
    /// selection with `Basis::Mismatch` rather than panicking or indexing past
    /// a short vector: scores built for a different view are refused, and
    /// `ViewImportance::new` returns `None` for a wrong-length vector.
    #[test]
    fn importance_for_another_view_falls_back_to_todays_selection() {
        let (a, b) = (view(0, 0, 4, 4), view(1, 1, 4, 4));
        let s = select_tiles(b, 2, 9, 99, Some(&ramp(a)));
        assert_eq!(s.basis, Basis::Mismatch);
        assert_eq!(s.tiles, expected_today(b, 2));
        assert_eq!(s.order, expected_order(b, &expected_today(b, 2)));
        assert!(ViewImportance::new(a, vec![Some(1.0); 3], None).is_none());
        assert!(ViewImportance::new(a, vec![Some(1.0); 16], Some(vec![None; 15])).is_none());
    }

    /// Protects: the rule's *positive* path (so the identity tests above are not
    /// vacuous): on a 6x5 view of distinct positive scores, with budget to
    /// spare, exactly `ceil(25 % of 30) = 8` tiles are promoted, they are the
    /// 8 highest-scoring ones (the bottom rows, rightmost), each replaced by its
    /// four children one level deeper, and the branch is `Ranked`.
    #[test]
    fn the_top_quarter_by_score_is_promoted_one_level() {
        let v = view(0, 0, 6, 5);
        let s = select_tiles(v, 3, 10, 1000, Some(&ramp(v)));
        assert_eq!((s.basis, s.promoted), (Basis::Ranked, 8));
        assert_eq!(s.tiles.len(), 30 + 3 * 8);
        assert_eq!(at_level(&s, 3).len(), 22);
        let kids = at_level(&s, 4);
        assert_eq!(kids.len(), 32);
        // score 1 + col + 10 * row: the top 8 are row 4 (cols 0..=5) and row 3's
        // cols 5 and 4; their children are the (2c.., 2r..) tiles at z4.
        let mut parents: Vec<(u32, u32)> = kids.iter().map(|k| (k.col >> 1, k.row >> 1)).collect();
        parents.sort_unstable();
        parents.dedup();
        let mut want = vec![(0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (5, 3), (4, 3)];
        want.sort_unstable();
        assert_eq!(parents, want);
    }

    /// Protects: **never below `z0`** and **never deeper than `z0 + 1`** (Q1,
    /// I1): over many views, caps, budgets and score patterns every returned
    /// tile's level is `z0` or `z0 + 1`.
    #[test]
    fn levels_are_z0_or_one_deeper_and_never_shallower() {
        for (v, z0) in [(view(0, 0, 6, 5), 3u32), (view(2, 2, 3, 3), 1), (view(0, 0, 1, 1), 0)] {
            for budget in [0usize, 1, 3, 1000] {
                for cap in [0u32, z0, z0 + 1, z0 + 5] {
                    let s = select_tiles(v, z0, cap, budget, Some(&ramp(v)));
                    assert!(s.tiles.iter().all(|t| t.z == z0 || t.z == z0 + 1), "{v:?} z{z0} cap{cap} budget{budget}");
                    assert!(s.tiles.iter().all(|t| t.z >= z0), "never below z0");
                }
            }
        }
    }

    /// Protects: **I3, the budget**: `budget` is the maximum number of
    /// promoted tiles. With the 8-tile quarter above, budget 0 promotes none,
    /// 3 promotes exactly the 3 best, 8 and 1000 promote 8; and the tile count
    /// follows `view + 3 * promoted`.
    #[test]
    fn the_budget_caps_the_promotions() {
        let v = view(0, 0, 6, 5);
        for (budget, want) in [(0usize, 0usize), (1, 1), (3, 3), (8, 8), (1000, 8)] {
            let s = select_tiles(v, 3, 10, budget, Some(&ramp(v)));
            assert_eq!(s.promoted, want, "budget {budget}");
            assert_eq!(s.tiles.len(), 30 + 3 * want);
        }
        // the three that survive budget 3 are the three best scores
        let s = select_tiles(v, 3, 10, 3, Some(&ramp(v)));
        let mut parents: Vec<(u32, u32)> = at_level(&s, 4).iter().map(|k| (k.col >> 1, k.row >> 1)).collect();
        parents.sort_unstable();
        parents.dedup();
        assert_eq!(parents, vec![(3, 4), (4, 4), (5, 4)]);
    }

    /// Protects: **I3, `z_cap`**: a promotion needs `z0 + 1 <= z_cap`. At
    /// `z_cap == z0` (and below it, where the caller's own clamp failed) there is
    /// no promotion and no tile deeper than `z0`; at `z_cap == z0 + 1` there is;
    /// and a cap of 31 does not wrap a column index (the `MAX_TARGET_LEVEL`
    /// guard: a view at level 31 promotes nothing).
    #[test]
    fn z_cap_forbids_promotion_beyond_it() {
        let v = view(0, 0, 6, 5);
        for cap in [0u32, 2, 3] {
            let s = select_tiles(v, 3, cap, 1000, Some(&ramp(v)));
            assert_eq!(s.promoted, 0, "cap {cap}");
            assert!(s.tiles.iter().all(|t| t.z == 3));
            assert_eq!(s.basis, Basis::Ranked, "the order is still importance order");
        }
        assert_eq!(select_tiles(v, 3, 4, 1000, Some(&ramp(v))).promoted, 8);
        assert_eq!(select_tiles(v, 3, 100, 1000, Some(&ramp(v))).promoted, 8);
        // the top of the u32 index range: no promotion, no overflow panic
        let hi = ViewRange { c0: (1 << 31) - 3, c1: (1 << 31) - 1, r0: 5, r1: 6 };
        let s = select_tiles(hi, 31, 100, 1000, Some(&ramp(hi)));
        assert_eq!(s.promoted, 0);
        assert!(s.tiles.iter().all(|t| t.z == 31));
    }

    /// Protects: the **threshold's floor**: a tile whose score is exactly the
    /// floor, or below it, is never promoted even when budget and the 25 %
    /// fraction would allow it -- strictly above, not at-or-above. A view of
    /// non-positive scores promotes nothing; one with three positive tiles
    /// promotes exactly those three.
    #[test]
    fn a_tile_must_be_strictly_above_the_floor_to_be_promoted() {
        let v = view(0, 0, 4, 4);
        // all non-positive but distinct (spread exists, nothing clears)
        let low = scored(v, |c, r| Some(-f64::from(c + 4 * r)));
        let s = select_tiles(v, 2, 9, 1000, Some(&low));
        assert_eq!((s.promoted, s.basis), (0, Basis::Ranked), "score 0 for tile (0,0) is not above the floor");
        // three tiles above the floor, one exactly at it, the rest below
        let some = scored(v, |c, r| Some(match (c, r) { (3, 3) => 9.0, (2, 3) => 5.0, (1, 3) => 0.5, (0, 3) => 0.0, _ => -1.0 }));
        let s = select_tiles(v, 2, 9, 1000, Some(&some));
        assert_eq!(s.promoted, 3);
        let mut parents: Vec<(u32, u32)> = at_level(&s, 3).iter().map(|k| (k.col >> 1, k.row >> 1)).collect();
        parents.sort_unstable();
        parents.dedup();
        assert_eq!(parents, vec![(1, 3), (2, 3), (3, 3)]);
        // a custom rule's floor moves the boundary: floor -0.5 admits the 0.0 tile
        let lowfloor = SelectRule { top_percent: 100, score_floor: -0.5 };
        assert_eq!(select_with(&lowfloor, v, 2, 9, 1000, Some(&some)).promoted, 4);
    }

    /// Protects: the **top-fraction half of the threshold**: 16 tiles all above
    /// the floor promote only `ceil(25 %) = 4` (not all 16), `top_percent`
    /// 100 promotes all 16, 50 promotes 8 -- and tie scores break to the tile
    /// nearer the view centre, then to the lower index, so a plateau cannot be
    /// resolved arbitrarily.
    #[test]
    fn only_the_top_fraction_is_eligible_and_ties_go_to_the_centre() {
        let v = view(0, 0, 4, 4);
        let flat_but_two = scored(v, |c, r| Some(if (c, r) == (0, 0) { 2.0 } else { 1.0 }));
        let s = select_tiles(v, 2, 9, 1000, Some(&flat_but_two));
        assert_eq!(s.promoted, 4);
        let mut parents: Vec<(u32, u32)> = at_level(&s, 3).iter().map(|k| (k.col >> 1, k.row >> 1)).collect();
        parents.sort_unstable();
        parents.dedup();
        // (0,0) is highest; then the 1.0 plateau: nearest the centre (1.5, 1.5)
        // are (1,1), (2,1), (1,2), (2,2) -- equidistant, lowest indices first.
        assert_eq!(parents, vec![(0, 0), (1, 1), (1, 2), (2, 1)]); // sorted; the set is {(0,0),(1,1),(2,1),(1,2)}
        for (pct, want) in [(25u32, 4usize), (50, 8), (100, 16)] {
            let r = SelectRule { top_percent: pct, score_floor: 0.0 };
            assert_eq!(select_with(&r, v, 2, 9, 1000, Some(&flat_but_two)).promoted, want, "{pct} %");
        }
    }

    /// Protects: **exact cover** (§8.3): for several views, caps, budgets and
    /// score patterns -- including the identity branches -- the selection
    /// covers the view exactly once: every finest-level cell has exactly one
    /// covering tile (`baked_cover`), and the summed tile area equals the
    /// view's. A positive control feeds a hand-made overlap and a gap and
    /// requires the checker to flag both.
    #[test]
    fn the_selection_covers_the_view_exactly_once() {
        for (v, z0) in [(view(0, 0, 6, 5), 3u32), (view(3, 2, 4, 4), 3), (view(1, 0, 1, 7), 4), (view(0, 0, 1, 1), 0)] {
            for budget in [0usize, 2, 5, 1000] {
                for imp in [Some(ramp(v)), Some(scored(v, |c, r| Some(f64::from((c * 7 + r * 3) % 5) - 1.0))), None] {
                    let s = select_tiles(v, z0, z0 + 1, budget, imp.as_ref());
                    assert_eq!(cover_defects(v, z0, &s.tiles), 0, "{v:?} z{z0} budget{budget}");
                }
            }
        }
        // positive control: a duplicated tile, and a missing one
        let v = view(0, 0, 2, 2);
        let mut dup = expected_today(v, 1);
        dup.push(dup[0]);
        assert!(cover_defects(v, 1, &dup) > 0, "an overlap must be flagged");
        let mut gap = expected_today(v, 1);
        gap.pop();
        assert!(cover_defects(v, 1, &gap) > 0, "a gap must be flagged");
        // and a tile outside the view
        let mut stray = expected_today(v, 1);
        stray.push(ChunkId::new(1, 5, 5));
        assert!(cover_defects(v, 1, &stray) > 0, "a tile outside the view must be flagged");
    }

    /// Protects: the **2:1 balance** (§8.3): in every selection no two edge-
    /// neighbouring tiles differ by more than one level, over views and score
    /// patterns that promote an isolated tile, a block, and a ring. A positive
    /// control (a `z0` tile beside `z0 + 2` children) must be flagged, so the
    /// checker cannot pass vacuously. No balance *pass* exists to test: a
    /// tile's level is `z0` or `z0 + 1`, so none is needed.
    #[test]
    fn neighbouring_tiles_never_differ_by_more_than_one_level() {
        let v = view(0, 0, 6, 5);
        let patterns: Vec<ViewImportance> = vec![
            ramp(v),
            scored(v, |c, r| Some(if (c, r) == (2, 2) { 9.0 } else { 1.0 })),
            scored(v, |c, r| Some(if c == 0 || r == 0 || c == 5 || r == 4 { 7.0 } else { 0.5 })),
            scored(v, |c, r| Some(f64::from((c * 7 + r * 3) % 5) + 0.25)),
        ];
        for imp in &patterns {
            for budget in [1usize, 4, 1000] {
                let s = select_tiles(v, 3, 9, budget, Some(imp));
                assert_eq!(balance_violations(v, 3, &s.tiles), 0, "budget {budget}");
            }
        }
        // positive control: z1 tile (1,0) next to z3 tiles filling (0,0) at z1
        let bad: Vec<ChunkId> = std::iter::once(ChunkId::new(1, 1, 0))
            .chain(std::iter::once(ChunkId::new(1, 0, 1)))
            .chain(std::iter::once(ChunkId::new(1, 1, 1)))
            .chain(chunk_children(ChunkId::new(1, 0, 0)).into_iter().flat_map(chunk_children))
            .collect();
        assert!(balance_violations(view(0, 0, 2, 2), 1, &bad) > 0);
    }

    /// Protects: **determinism**: the same arguments give the same selection,
    /// order and basis on repeated calls, including over tied scores (where
    /// only a defined tie-break keeps it stable), and `order` is always a
    /// permutation of `0..tiles.len()`.
    #[test]
    fn the_selection_is_deterministic_and_the_order_a_permutation() {
        let v = view(0, 0, 6, 5);
        let tied = scored(v, |c, _| Some(1.0 + f64::from(c % 2)));
        for imp in [&ramp(v), &tied] {
            let a = select_tiles(v, 3, 9, 6, Some(imp));
            assert_eq!(a, select_tiles(v, 3, 9, 6, Some(imp)));
            let mut sorted = a.order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..a.tiles.len()).collect::<Vec<_>>());
        }
    }

    /// Protects: the **order is importance order** (not today's): among tiles
    /// at one level, a higher score is synthesised before a lower one even when
    /// it is farther from the centre; a promoted tile's four children inherit its
    /// score so they come out first; and within equal scores the nearer-the-
    /// centre tile is first. Here every score differs, so the order must be the
    /// scores sorted descending.
    #[test]
    fn synthesis_follows_importance_not_distance() {
        let v = view(0, 0, 6, 5);
        let imp = ramp(v);
        let s = select_tiles(v, 3, 3, 1000, Some(&imp)); // cap forbids promotion: pure ordering
        let by_score: Vec<ChunkId> = s.order.iter().map(|&i| s.tiles[i]).collect();
        let mut want = expected_today(v, 3);
        want.sort_by_key(|t| std::cmp::Reverse(t.col + 10 * t.row));
        assert_eq!(by_score, want);
        // with promotion, the 4 children of the best tile are the first four
        let p = select_tiles(v, 3, 9, 1000, Some(&imp));
        let first4: Vec<ChunkId> = p.order[..4].iter().map(|&i| p.tiles[i]).collect();
        assert!(first4.iter().all(|t| t.z == 4 && (t.col >> 1, t.row >> 1) == (5, 4)), "{first4:?}");
        // every promoted child precedes every unpromoted z0 tile (it has a higher score)
        let pos = |z: u32| p.order.iter().enumerate().filter(|&(_, &i)| p.tiles[i].z == z).map(|(k, _)| k).collect::<Vec<_>>();
        assert!(pos(4).iter().max() < pos(3).iter().min());
    }

    /// Protects: the **order among tied scores is nearer-the-centre first, across
    /// levels** (the order key's last term). On a 4x4 view with one best tile and a
    /// 1.0 plateau, with promotion allowed, the whole `order` equals an
    /// independent computation: sort by (score descending, squared distance from
    /// the view centre, index) in the `z0 + 1` index space, with every coordinate
    /// doubled to stay in integers (a `z0` tile `(c, r)` sits at `(4c + 1, 4r + 1)`,
    /// a child `(col, row)` at `(2col, 2row)`, the centre at
    /// `(2(c0 + c1) + 1, 2(r0 + r1) + 1)`). Moves if the centre pre-permutation,
    /// the `+ 0.5` offsets or the centre mapping change.
    #[test]
    fn tied_scores_are_ordered_by_distance_from_the_centre_across_levels() {
        let v = view(0, 0, 4, 4);
        let imp = scored(v, |c, r| Some(if (c, r) == (0, 0) { 2.0 } else { 1.0 }));
        let s = select_tiles(v, 2, 9, 1000, Some(&imp));
        assert_eq!(s.promoted, 4);
        let key = |t: &ChunkId| {
            let (px, py) = if t.z == 3 { (2 * i64::from(t.col), 2 * i64::from(t.row)) } else { (4 * i64::from(t.col) + 1, 4 * i64::from(t.row) + 1) };
            let (z0c, z0r) = if t.z == 3 { (t.col >> 1, t.row >> 1) } else { (t.col, t.row) };
            let score = if (z0c, z0r) == (0, 0) { 2 } else { 1 };
            let (dx, dy) = (px - (2 * (i64::from(v.c0) + i64::from(v.c1)) + 1), py - (2 * (i64::from(v.r0) + i64::from(v.r1)) + 1));
            (-score, dx * dx + dy * dy)
        };
        let mut want: Vec<usize> = (0..s.tiles.len()).collect();
        want.sort_by_key(|&i| (key(&s.tiles[i]), i));
        assert_eq!(s.order, want);
    }

    /// Protects: **Q2 -- policy raises order, never level.** Giving a
    /// low-importance tile policy rank 1 moves it to the front of the order, but
    /// the *set* of promoted tiles and every tile's level are byte-identical to
    /// the run without ranks. Ranks are the only thing that differs between the
    /// two runs.
    #[test]
    fn policy_ranks_reorder_but_never_promote() {
        let v = view(0, 0, 6, 5);
        let plain = ramp(v);
        let mut ranks = vec![None; v.len()];
        ranks[0] = Some(1); // tile (0, 0): the lowest score of all
        let with = ViewImportance::new(v, plain.scores().to_vec(), Some(ranks)).unwrap();
        let a = select_tiles(v, 3, 9, 1000, Some(&plain));
        let b = select_tiles(v, 3, 9, 1000, Some(&with));
        assert_eq!(a.tiles, b.tiles, "same tiles, same levels");
        assert_eq!(a.promoted, b.promoted);
        assert_ne!(a.order, b.order, "but a different order");
        assert_eq!(b.tiles[b.order[0]], ChunkId::new(3, 0, 0), "the policy tile is synthesised first");
        // with uniform scores, ranks alone still order (no identity shortcut),
        // and nothing is promoted
        let mut r2 = vec![None; v.len()];
        r2[7] = Some(1);
        let flat = ViewImportance::new(v, vec![Some(4.0); v.len()], Some(r2)).unwrap();
        let c = select_tiles(v, 3, 9, 1000, Some(&flat));
        assert_eq!((c.promoted, c.tiles[c.order[0]]), (0, ChunkId::new(3, 1, 1)));
    }

    /// Protects: **a `None` score is never zero**: in a view where some tiles
    /// have no score, those tiles are never promoted -- even when every present
    /// score is below the floor and a `Some(0.0)` neighbour would have
    /// been, by contrast, treated as a plain below-floor tile -- and they sort
    /// after every scored tile.
    #[test]
    fn a_tile_without_a_score_is_never_promoted_and_sorts_last() {
        let v = view(0, 0, 4, 2);
        let imp = scored(v, |c, r| if (c + r) % 2 == 0 { None } else { Some(f64::from(c) + 1.0) });
        let s = select_tiles(v, 2, 9, 1000, Some(&imp));
        assert_eq!(s.basis, Basis::Ranked);
        assert!(s.promoted > 0);
        for t in at_level(&s, 3) {
            let (c, r) = (t.col >> 1, t.row >> 1);
            assert!((c + r) % 2 == 1, "tile ({c},{r}) has no score and was promoted");
        }
        let scored_last = s.order.iter().rposition(|&i| {
            let t = s.tiles[i];
            let (c, r) = if t.z == 3 { (t.col >> 1, t.row >> 1) } else { (t.col, t.row) };
            (c + r) % 2 == 1
        });
        let unscored_first = s.order.iter().position(|&i| {
            let t = s.tiles[i];
            let (c, r) = if t.z == 3 { (t.col >> 1, t.row >> 1) } else { (t.col, t.row) };
            (c + r) % 2 == 0
        });
        assert!(scored_last < unscored_first, "every scored tile precedes every unscored one");
    }

    /// Protects: an empty (reversed) view returning an empty selection, not a
    /// panic or a one-tile fallback -- the one case where `ViewRange`'s
    /// reversed-is-empty rule is observable.
    #[test]
    fn an_empty_view_selects_nothing() {
        let v = ViewRange { c0: 3, c1: 2, r0: 0, r1: 4 };
        assert!(v.is_empty());
        let s = select_tiles(v, 3, 9, 10, None);
        assert!(s.tiles.is_empty() && s.order.is_empty());
        let imp = ViewImportance::new(v, Vec::new(), None).unwrap();
        let s = select_tiles(v, 3, 9, 10, Some(&imp));
        assert!(s.tiles.is_empty() && s.order.is_empty() && s.promoted == 0);
    }

    /// Protects: the end-to-end path from a world's rasters to a promotion, with
    /// the shipped `FITTED_WEIGHTS`: a 16x16 world whose west third is sea, with
    /// a steep ridge in the north-east and a gentle plain elsewhere. At `z0 = 2`
    /// (4x4 tiles) no sea tile may be promoted (their fitted score is far
    /// below the floor), at least one ridge tile must be, and **every promoted
    /// tile lies on land**. It also pins that `from_world` scores
    /// `tile_features` over the right tile bounds: a plain tile and a ridge
    /// tile get different scores.
    #[test]
    fn a_ridge_is_promoted_and_open_sea_is_not() {
        use crate::importance::FITTED_WEIGHTS;
        let n = 16usize;
        let mut f = vec![0.6f32; n * n];
        for y in 0..n {
            for x in 0..n {
                if x < 5 {
                    f[y * n + x] = 0.2; // sea, sea level 0.42
                }
                if x >= 10 && y < 6 {
                    // a steep ridge: height saw-toothing by column in the NE
                    f[y * n + x] = 0.6 + 0.3 * ((x + y) % 2) as f32;
                }
            }
        }
        let src = WorldImportanceField::build(&f, n, n, 0.42, false, 800.0, None, &[], &[]);
        let v = view(0, 0, 4, 4);
        let imp = ViewImportance::from_world(&src, &FITTED_WEIGHTS, v, 2, None).expect("grid is non-empty");
        assert_eq!(imp.scores().len(), 16);
        assert!(imp.scores().iter().all(Option::is_some));
        let s = select_tiles(v, 2, 8, 1000, Some(&imp));
        assert_eq!(s.basis, Basis::Ranked);
        let promoted: Vec<(u32, u32)> = {
            let mut p: Vec<(u32, u32)> = at_level(&s, 3).iter().map(|k| (k.col >> 1, k.row >> 1)).collect();
            p.sort_unstable();
            p.dedup();
            p
        };
        assert!(!promoted.is_empty());
        // tile (c, r) at z2 spans cells [3.75c, 3.75(c+1)): column 0 is all sea
        // and column 1 is mostly sea (cells 3.75..7.5, sea is x < 5)
        assert!(promoted.iter().all(|&(c, _)| c >= 2), "no sea tile may be promoted: {promoted:?}");
        assert!(promoted.iter().any(|&(c, r)| c >= 2 && r <= 1), "the ridge (NE) must be promoted: {promoted:?}");
        assert_ne!(imp.scores()[3], imp.scores()[15], "ridge tile and plain tile score differently");
        // a policy slice of the wrong length is refused, not truncated
        assert!(ViewImportance::from_world(&src, &FITTED_WEIGHTS, v, 2, Some(&[PolicyInputs::default(); 3])).is_none());
    }
}
