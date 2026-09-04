//! `PassBuffer<S>` — the non-destructive draft/commit/discard layer
//! (`UNIFIED_TOOL_PLAN.md` milestone A).
//!
//! This is a direct port of the *behaviour* of the reference HTML engine's
//! Sculpt editor draft layer (`reference/Cartalith Gen1 v2.10.html`,
//! `sculptStamps[]`/`sculptCommit`/`sculptDiscard`, lines ~9089-9360), with a
//! Rust-native storage substrate. The reference's own comment at the draft
//! layer is the whole contract in one line: *"nothing here touches `field` or
//! triggers any recompute"*.
//!
//! ## What a stamp is, as the reference actually stores it
//!
//! `{type, seed, pts, g:{...}, f:{...}, hidden, _cx, _cy}` — a feature key, a
//! seed, the captured stroke polyline in grid coordinates, two flat parameter
//! bags (global brush/noise params and the per-feature control values), a
//! hide flag, and a cached centroid for radial features. Critically it stores
//! **no pixel data at all**: a stamp is a *recipe*, re-evaluated over its own
//! padded bounding box every time it is drawn or baked. That is what makes
//! the draft cheap enough to keep as plain object state, snapshot for undo,
//! reorder, and throw away.
//!
//! This module therefore does **not** define a concrete `Stamp` struct. The
//! recipe (which landform, which parameters, which noise) is
//! Cartalith-terrain-specific and belongs with the feature registry that
//! Milestone B ports; the *stack semantics* around it are generic and belong
//! here, next to [`crate::DirtyTracker`] and [`crate::TiledField`]. So
//! [`Stamp`] is a trait with exactly the two operations the stack semantics
//! need — "what do you touch" and "write yourself into this caller-supplied
//! destination" — and a biome-paint disc, a territory-paint disc, and a
//! 13-feature landform stamp can all implement it without this crate learning
//! what a biome is (the same "stay generic, no Cartalith semantics baked in"
//! precedent [`crate::QuadTree`]'s caller-defined flag bitmask already set).
//!
//! ## Why `apply` writes into a caller-supplied slice
//!
//! Because the reference's does, for exactly the reason this milestone needs:
//! `sculptApplyStamp` *"writes directly into caller-supplied H/W arrays
//! (never `field`/module globals) so both the draft preview (a scratch
//! buffer) and commit (field itself) reuse the identical code path"*. One
//! apply function, two destinations, chosen by the caller — preview and
//! commit cannot drift because they are the same code. [`Stamp::apply`] keeps
//! that contract verbatim.
//!
//! Note also that `apply` reads the destination it writes (the reference's
//! `c.h0 = H[i]`): stamps compose over the accumulating buffer, which is why
//! stack **order** is load-bearing and why commit bakes the whole stack in
//! one ordered pass rather than each stamp against the original field.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{DirtyTracker, Region};

/// Draft-scoped undo depth, matching the reference's own `SCULPT_HIST_MAX`.
pub const HISTORY_MAX: usize = 30;

/// One entry in a [`PassBuffer`]'s stack: a recipe that knows its own
/// footprint and how to write itself into a destination field.
///
/// Implementors stay pure — no interior mutability, no globals — so that
/// preview and commit are genuinely the same computation against different
/// destinations.
pub trait Stamp {
    /// The field cell type this stamp writes (`f32` for height, `u8` for a
    /// categorical override layer, and so on).
    type Cell;

    /// The stamp's padded footprint in field-cell coordinates, already
    /// clipped to `width`/`height` — the reference's `sculptStampBBox`,
    /// which pads by radius + feather + edge-noise amplitude precisely so
    /// that domain-warped edges can't spill outside the box.
    ///
    /// Returning a zero-area [`Region`] means "touches nothing"; a
    /// [`PassBuffer`] treats such a stamp as marking no tiles.
    fn bounds(&self, width: usize, height: usize) -> Region;

    /// Write this stamp into `dst`, a full field-sized, row-major slice
    /// (`dst[y * width + x]`). The stamp must only write cells inside its own
    /// [`Stamp::bounds`], and may freely read `dst` (stamps compose over the
    /// accumulated result, in stack order).
    fn apply(&self, dst: &mut [Self::Cell], width: usize, height: usize);
}

/// A stamp plus its draft-scoped visibility. `hidden` lives here rather than
/// on the [`Stamp`] itself because hiding is a *stack* edit (one of the four
/// structural edits the reference's draft undo tracks: add, delete, reorder,
/// hide), not a property of the recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassEntry<S> {
    pub stamp: S,
    pub hidden: bool,
}

/// What a [`PassBuffer::commit`] actually did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSummary {
    /// Visible stamps baked into the field, in stack order.
    pub stamps_applied: usize,
    /// Hidden stamps skipped (they are dropped with the rest of the draft).
    pub stamps_skipped: usize,
    /// Tiles marked dirty, ascending. Exactly one version bump per tile per
    /// commit however many strokes touched it — `UI_SHELL_DESIGN.md`'s
    /// *"undo granularity is one committed pass, not one stroke"*, enforced
    /// here rather than left to the caller.
    pub tiles_marked: Vec<usize>,
}

/// An append-only stack of uncommitted [`Stamp`]s over a field that the
/// buffer never writes to until [`PassBuffer::commit`].
///
/// The buffer holds field *dimensions*, not the field itself: the live data
/// stays owned by whoever owns the pipeline (a `Vec<f32>`, a
/// [`crate::TiledField`]'s backing store), and is passed in by reference at
/// preview/commit time. That keeps the non-destructive guarantee visible in
/// the type system — [`PassBuffer::preview_into`] takes the field as `&[_]`
/// and so *cannot* mutate it, whatever a stamp implementation does.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassBuffer<S> {
    width: usize,
    height: usize,
    tile_size: usize,
    entries: Vec<PassEntry<S>>,
    /// Union of every entry's tile footprint, hidden entries included —
    /// hiding a stamp still requires repainting its footprint, so this is the
    /// *render* scope. Commit's dirty-marking uses the narrower visible-only
    /// set instead.
    touched: BTreeSet<usize>,
    history: Vec<Vec<PassEntry<S>>>,
    redo: Vec<Vec<PassEntry<S>>>,
}

impl<S: Stamp + Clone> PassBuffer<S> {
    /// `tile_size` matches the [`crate::TiledField`] whose tile indexing the
    /// paired [`DirtyTracker`] uses (`ty * tiles_x + tx`), so that
    /// [`PassBuffer::tile_count`] can size that tracker directly.
    pub fn new(width: usize, height: usize, tile_size: usize) -> Self {
        assert!(tile_size > 0, "tile_size must be positive");
        Self {
            width,
            height,
            tile_size,
            entries: Vec::new(),
            touched: BTreeSet::new(),
            history: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_size(&self) -> usize {
        self.tile_size
    }

    pub fn tiles_x(&self) -> usize {
        self.width.div_ceil(self.tile_size)
    }

    pub fn tiles_y(&self) -> usize {
        self.height.div_ceil(self.tile_size)
    }

    /// The tile count a paired [`DirtyTracker`] must be constructed with.
    pub fn tile_count(&self) -> usize {
        self.tiles_x() * self.tiles_y()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[PassEntry<S>] {
        &self.entries
    }

    pub fn get(&self, index: usize) -> Option<&PassEntry<S>> {
        self.entries.get(index)
    }

    /// Tiles the draft currently affects, ascending — the exact set a
    /// renderer needs to re-upload for a preview, instead of the whole map.
    pub fn touched_tiles(&self) -> impl Iterator<Item = usize> + '_ {
        self.touched.iter().copied()
    }

    /// The union of every entry's footprint as one rectangle, or `None` when
    /// the draft is empty (or touches nothing).
    pub fn touched_bounds(&self) -> Option<Region> {
        let mut acc: Option<(usize, usize, usize, usize)> = None;
        for e in &self.entries {
            let b = e.stamp.bounds(self.width, self.height);
            if b.w == 0 || b.h == 0 {
                continue;
            }
            let (x0, y0, x1, y1) = (b.x, b.y, b.x + b.w, b.y + b.h);
            acc = Some(match acc {
                None => (x0, y0, x1, y1),
                Some((ax0, ay0, ax1, ay1)) => (ax0.min(x0), ay0.min(y0), ax1.max(x1), ay1.max(y1)),
            });
        }
        acc.map(|(x0, y0, x1, y1)| Region::new(x0, y0, x1 - x0, y1 - y0))
    }

    // ---- structural edits (each one records draft-scoped undo) ----

    /// Pushes a finished stroke onto the stack. Returns its index.
    pub fn push(&mut self, stamp: S) -> usize {
        self.push_history();
        let tiles = self.tiles_of(stamp.bounds(self.width, self.height));
        self.touched.extend(tiles);
        self.entries.push(PassEntry {
            stamp,
            hidden: false,
        });
        self.entries.len() - 1
    }

    /// Removes the stamp at `index`, returning it.
    pub fn remove(&mut self, index: usize) -> S {
        self.push_history();
        let e = self.entries.remove(index);
        self.recompute_touched();
        e.stamp
    }

    /// Hides or shows a stamp. A hidden stamp is skipped by preview and by
    /// commit, but still counts toward [`PassBuffer::touched_tiles`] — the
    /// act of hiding it is itself a reason to repaint its footprint.
    pub fn set_hidden(&mut self, index: usize, hidden: bool) {
        self.push_history();
        self.entries[index].hidden = hidden;
    }

    /// Moves a stamp one place earlier in the stack. No-op at index 0.
    /// Stack order matters: stamps compose over each other's output.
    pub fn move_up(&mut self, index: usize) -> bool {
        if index == 0 || index >= self.entries.len() {
            return false;
        }
        self.push_history();
        self.entries.swap(index - 1, index);
        true
    }

    /// Moves a stamp one place later in the stack. No-op at the top.
    pub fn move_down(&mut self, index: usize) -> bool {
        if index + 1 >= self.entries.len() {
            return false;
        }
        self.push_history();
        self.entries.swap(index, index + 1);
        true
    }

    // ---- draft-scoped undo/redo ----

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Reverts the last structural edit. Draft-scoped only: this never
    /// touches the field, because nothing in the draft ever did.
    pub fn undo(&mut self) -> bool {
        let Some(prev) = self.history.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.entries, prev));
        self.recompute_touched();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.history.push(std::mem::replace(&mut self.entries, next));
        self.recompute_touched();
        true
    }

    fn push_history(&mut self) {
        self.history.push(self.entries.clone());
        if self.history.len() > HISTORY_MAX {
            self.history.remove(0);
        }
        // Same rule as the reference: any new structural edit invalidates the
        // redo branch.
        self.redo.clear();
    }

    // ---- preview / commit / discard ----

    /// Composites the draft over a **read** of `base` into `scratch`,
    /// leaving `base` untouched — `base` is `&[_]`, so that guarantee is the
    /// borrow checker's, not a convention.
    ///
    /// `scratch` must be field-sized. Copying the whole base each call is the
    /// simple, obviously-correct primitive; when a caller wants the bounded
    /// one, [`PassBuffer::preview_touched_into`] is the same composite
    /// restricted to [`PassBuffer::touched_bounds`] and is the method to
    /// reach for. This one stays whole-grid on purpose — it is what
    /// [`PassBuffer::commit`] is checked against, and what the bounded
    /// variant's own tests use as their oracle.
    ///
    /// **Wired-caller history, since two earlier revisions of this paragraph
    /// were both stale when read.** It said "with no renderer wired yet"
    /// from 2026-08-18 01:53 until 2026-09-04, by which time
    /// `cartalith-godot`'s `build_sculpt_preview_texture` (20:28 the same
    /// day) and `build_paint_preview_texture` (22:43) had both wired one.
    /// It then said both *"upload the whole grid on every call — neither
    /// reads `touched_tiles` or [`PassBuffer::touched_bounds`] at all"*,
    /// which stood for one batch: `cartalith-godot`'s
    /// `build_paint_preview_patch` now composites through
    /// [`PassBuffer::preview_touched_into`] and uploads the window only.
    /// **The sculpt one still uploads the whole grid, and still declines
    /// deliberately** — `RenderCtx::with_appearance` runs six whole-grid
    /// passes on construction with no window parameter, which is
    /// `SCULPT_LIVE_SCOPE.md` L1's work, not this crate's.
    pub fn preview_into(&self, base: &[S::Cell], scratch: &mut [S::Cell])
    where
        S::Cell: Clone,
    {
        assert_eq!(
            base.len(),
            self.width * self.height,
            "base length must equal width * height"
        );
        assert_eq!(
            scratch.len(),
            base.len(),
            "scratch must be the same size as base"
        );
        scratch.clone_from_slice(base);
        for e in &self.entries {
            if !e.hidden {
                e.stamp.apply(scratch, self.width, self.height);
            }
        }
    }

    /// The bounded half of [`PassBuffer::preview_into`]: composites the draft
    /// over `base` into `scratch` for the cells inside
    /// [`PassBuffer::touched_bounds`] **only**, and returns that window.
    /// Every cell of `scratch` outside it is left exactly as the caller
    /// passed it in — this method neither reads nor writes there, so a
    /// caller may keep one scratch allocation alive across frames and pay
    /// only the window.
    ///
    /// Inside the window the result is byte-identical to
    /// [`PassBuffer::preview_into`]'s, and that is a property of the code
    /// rather than a hope: it copies the same rows of `base`, then makes the
    /// same [`Stamp::apply`] calls in the same stack order, skipping the same
    /// hidden entries, against a `scratch` of the same dimensions — the
    /// indices a stamp computes are identical because `width`/`height` are.
    /// `pass::tests::preview_touched_into_matches_preview_into_in_the_window`
    /// asserts it against `preview_into` as an oracle rather than against a
    /// second transcription of this loop.
    ///
    /// Bounding it is sound because the window is, by construction, the union
    /// of every entry's [`Stamp::bounds`] — hidden ones included, since
    /// [`PassBuffer::touched_bounds`] does not filter them — and
    /// [`Stamp::apply`]'s contract is that a stamp writes only inside its own
    /// bounds. A stamp that also confines its *reads* there (`PaintStamp`,
    /// the only caller today, performs no reads of `dst` at all) therefore
    /// cannot observe or disturb a single cell outside the window.
    ///
    /// **`None` means the draft touched nothing. It does not mean "nothing
    /// needs drawing", and it is the exact opposite of "everything is
    /// dirty".** A committed layer with an empty draft returns `None` here
    /// and still owes its consumer the whole grid; so does a draft whose
    /// every stamp is off-grid or zero-radius. Nothing is written to
    /// `scratch` in that case — deliberately, so that a caller which
    /// mistook `None` for "an empty window, draw nothing" renders a blank
    /// rather than silently-stale pixels. Loud beats quiet for this one.
    pub fn preview_touched_into(&self, base: &[S::Cell], scratch: &mut [S::Cell]) -> Option<Region>
    where
        S::Cell: Clone,
    {
        assert_eq!(
            base.len(),
            self.width * self.height,
            "base length must equal width * height"
        );
        assert_eq!(
            scratch.len(),
            base.len(),
            "scratch must be the same size as base"
        );
        let win = self.touched_bounds()?;
        for y in win.y..win.y + win.h {
            let row = y * self.width + win.x;
            scratch[row..row + win.w].clone_from_slice(&base[row..row + win.w]);
        }
        for e in &self.entries {
            if !e.hidden {
                e.stamp.apply(scratch, self.width, self.height);
            }
        }
        Some(win)
    }

    /// Bakes the whole stack into `field` in stack order, marks every
    /// affected tile dirty in `tracker` with `reason`, and empties the draft.
    ///
    /// Atomic in the only sense that matters here: every precondition is
    /// checked before the first write, and [`Stamp::apply`] is infallible, so
    /// there is no partial-commit state to recover from. Either the whole
    /// stack lands or the call panics before touching a cell.
    ///
    /// Marking is **not** cascaded to downstream stages — see
    /// [`crate::staleness`]. That is the deferred half of the design, and it
    /// mirrors `sculptCommit` itself, which runs exactly one `computeFlow`/
    /// `refreshClimate` per commit and never eagerly re-runs settlements,
    /// roads or territory.
    pub fn commit(
        &mut self,
        field: &mut [S::Cell],
        tracker: &mut DirtyTracker,
        reason: &str,
    ) -> CommitSummary {
        assert_eq!(
            field.len(),
            self.width * self.height,
            "field length must equal width * height"
        );
        assert_eq!(
            tracker.tile_count(),
            self.tile_count(),
            "tracker must have one entry per tile of this buffer's grid"
        );

        let mut applied = 0usize;
        let mut skipped = 0usize;
        let mut marked: BTreeSet<usize> = BTreeSet::new();
        for e in &self.entries {
            if e.hidden {
                skipped += 1;
                continue;
            }
            e.stamp.apply(field, self.width, self.height);
            applied += 1;
            let b = e.stamp.bounds(self.width, self.height);
            marked.extend(Self::tiles_in(
                b,
                self.width,
                self.height,
                self.tile_size,
                self.tiles_x(),
            ));
        }
        for &t in &marked {
            tracker.mark_dirty(t, reason);
        }

        self.clear_draft();
        CommitSummary {
            stamps_applied: applied,
            stamps_skipped: skipped,
            tiles_marked: marked.into_iter().collect(),
        }
    }

    /// Drops the whole draft. Returns how many stamps were dropped.
    ///
    /// Nothing was ever written to the field, so there is nothing to undo
    /// there — the reference needed no per-stroke field undo for exactly this
    /// reason, and neither does this.
    pub fn discard(&mut self) -> usize {
        let n = self.entries.len();
        self.clear_draft();
        n
    }

    fn clear_draft(&mut self) {
        self.entries.clear();
        self.touched.clear();
        self.history.clear();
        self.redo.clear();
    }

    // ---- tile bookkeeping ----

    fn recompute_touched(&mut self) {
        let (w, h, ts, tx) = (self.width, self.height, self.tile_size, self.tiles_x());
        let mut set = BTreeSet::new();
        for e in &self.entries {
            set.extend(Self::tiles_in(e.stamp.bounds(w, h), w, h, ts, tx));
        }
        self.touched = set;
    }

    fn tiles_of(&self, bounds: Region) -> Vec<usize> {
        Self::tiles_in(
            bounds,
            self.width,
            self.height,
            self.tile_size,
            self.tiles_x(),
        )
    }

    fn tiles_in(
        bounds: Region,
        width: usize,
        height: usize,
        tile_size: usize,
        tiles_x: usize,
    ) -> Vec<usize> {
        if bounds.w == 0 || bounds.h == 0 || width == 0 || height == 0 {
            return Vec::new();
        }
        // Clamp to the field: a stamp near an edge legitimately reports a
        // padded box, and tile indices must stay inside the tracker's range.
        let x0 = bounds.x.min(width - 1);
        let y0 = bounds.y.min(height - 1);
        let x1 = (bounds.x + bounds.w - 1).min(width - 1);
        let y1 = (bounds.y + bounds.h - 1).min(height - 1);
        let mut out = Vec::new();
        for ty in (y0 / tile_size)..=(y1 / tile_size) {
            for tx in (x0 / tile_size)..=(x1 / tile_size) {
                out.push(ty * tiles_x + tx);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deliberately trivial stand-in for Milestone B's real landform
    /// stamps: add `amount` to every cell of an axis-aligned box. It has the
    /// two properties the pass-buffer machinery actually depends on — a
    /// declared footprint, and an `apply` that reads the destination it
    /// writes so that stacked stamps compose in order.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct AddBox {
        area: Region,
        amount: f32,
    }

    impl Stamp for AddBox {
        type Cell = f32;

        fn bounds(&self, width: usize, height: usize) -> Region {
            let x = self.area.x.min(width);
            let y = self.area.y.min(height);
            Region::new(
                x,
                y,
                self.area.w.min(width.saturating_sub(x)),
                self.area.h.min(height.saturating_sub(y)),
            )
        }

        fn apply(&self, dst: &mut [f32], width: usize, height: usize) {
            let b = self.bounds(width, height);
            for y in b.y..b.y + b.h {
                for x in b.x..b.x + b.w {
                    dst[y * width + x] += self.amount;
                }
            }
        }
    }

    fn stamp(x: usize, y: usize, w: usize, h: usize, amount: f32) -> AddBox {
        AddBox {
            area: Region::new(x, y, w, h),
            amount,
        }
    }

    fn buffer() -> PassBuffer<AddBox> {
        // 8x8 field, tile_size 4 -> 2x2 = 4 tiles.
        PassBuffer::new(8, 8, 4)
    }

    #[test]
    fn tile_count_matches_a_paired_dirty_tracker() {
        let buf = buffer();
        assert_eq!(buf.tiles_x(), 2);
        assert_eq!(buf.tiles_y(), 2);
        assert_eq!(buf.tile_count(), 4);
    }

    // ---- the headline guarantee: preview never mutates ----

    #[test]
    fn preview_composites_without_mutating_the_field() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        buf.push(stamp(1, 1, 2, 2, 0.5));

        let base = vec![0.0f32; 64];
        let before = base.clone();
        let mut scratch = vec![f32::NAN; 64];
        buf.preview_into(&base, &mut scratch);

        // The field is bit-identical -- nothing in the draft path writes it.
        assert_eq!(base, before);
        // The preview shows both stamps, composited in stack order.
        assert_eq!(scratch[0], 1.0); // first stamp only
        assert_eq!(scratch[9], 1.5); // overlap of both
        assert_eq!(scratch[18], 0.5); // second stamp only
        assert_eq!(scratch[63], 0.0); // untouched, copied from base
    }

    #[test]
    fn preview_is_idempotent_across_repeated_calls() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 3, 3, 2.0));
        let base = vec![1.0f32; 64];
        let mut a = vec![0.0f32; 64];
        let mut b = vec![0.0f32; 64];
        buf.preview_into(&base, &mut a);
        buf.preview_into(&base, &mut b);
        // Re-previewing must not accumulate -- the scratch is rebuilt from
        // base each time, it is not a running total.
        assert_eq!(a, b);
        assert_eq!(a[0], 3.0);
    }

    #[test]
    fn preview_skips_hidden_stamps() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        let i = buf.push(stamp(0, 0, 2, 2, 4.0));
        buf.set_hidden(i, true);
        let base = vec![0.0f32; 64];
        let mut scratch = vec![0.0f32; 64];
        buf.preview_into(&base, &mut scratch);
        assert_eq!(scratch[0], 1.0);
    }

    // ---- commit ----

    #[test]
    fn commit_applies_the_whole_stack_in_order_and_empties_the_draft() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        buf.push(stamp(1, 1, 2, 2, 0.5));

        let mut field = vec![0.0f32; 64];
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(&mut field, &mut tracker, "height_edited");

        assert_eq!(summary.stamps_applied, 2);
        assert_eq!(summary.stamps_skipped, 0);
        assert_eq!(field[0], 1.0);
        assert_eq!(field[9], 1.5);
        assert_eq!(field[18], 0.5);
        assert!(buf.is_empty());
        assert!(!buf.can_undo(), "committing clears the draft-scoped history");
    }

    #[test]
    fn commit_result_is_identical_to_the_preview_it_replaced() {
        // Preview and commit are the same code path against different
        // destinations (the reference's own sculptApplyStamp contract) --
        // this is the test that would catch them drifting apart.
        let mut buf = buffer();
        buf.push(stamp(0, 0, 5, 5, 0.25));
        buf.push(stamp(3, 3, 4, 4, -0.75));

        let base = vec![0.5f32; 64];
        let mut preview = vec![0.0f32; 64];
        buf.preview_into(&base, &mut preview);

        let mut field = base.clone();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        buf.commit(&mut field, &mut tracker, "height_edited");

        assert_eq!(field, preview);
    }

    #[test]
    fn commit_skips_hidden_stamps_but_still_drops_them() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        let i = buf.push(stamp(0, 0, 2, 2, 9.0));
        buf.set_hidden(i, true);

        let mut field = vec![0.0f32; 64];
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(&mut field, &mut tracker, "height_edited");
        assert_eq!(summary.stamps_applied, 1);
        assert_eq!(summary.stamps_skipped, 1);
        assert_eq!(field[0], 1.0);
        assert!(buf.is_empty());
    }

    #[test]
    fn commit_marks_exactly_the_touched_tiles_with_the_given_reason() {
        let mut buf = buffer();
        // Entirely inside tile (0,0) -> index 0.
        buf.push(stamp(0, 0, 3, 3, 1.0));
        // Spans the vertical seam -> tiles 0 and 1.
        buf.push(stamp(3, 0, 2, 2, 1.0));

        let mut field = vec![0.0f32; 64];
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(&mut field, &mut tracker, "height_edited");

        assert_eq!(summary.tiles_marked, vec![0, 1]);
        assert!(tracker.is_dirty(0));
        assert!(tracker.is_dirty(1));
        assert!(!tracker.is_dirty(2), "tile 2 was never touched");
        assert!(!tracker.is_dirty(3));
        assert_eq!(tracker.reason(0), Some("height_edited"));
    }

    #[test]
    fn one_commit_bumps_each_touched_tile_exactly_once_however_many_strokes() {
        // "Undo granularity is one committed pass, not one stroke" -- five
        // strokes over the same tile must be one version bump, not five.
        let mut buf = buffer();
        for _ in 0..5 {
            buf.push(stamp(0, 0, 2, 2, 0.1));
        }
        let mut field = vec![0.0f32; 64];
        let mut tracker = DirtyTracker::new(buf.tile_count());
        buf.commit(&mut field, &mut tracker, "height_edited");
        assert_eq!(tracker.version(0), 1);
        assert_eq!(tracker.version(1), 0);
    }

    #[test]
    fn an_empty_commit_writes_nothing_and_marks_nothing() {
        let mut buf = buffer();
        let mut field = vec![0.25f32; 64];
        let before = field.clone();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(&mut field, &mut tracker, "height_edited");
        assert_eq!(summary.stamps_applied, 0);
        assert!(summary.tiles_marked.is_empty());
        assert_eq!(field, before);
        assert_eq!(tracker.version(0), 0);
    }

    // ---- discard ----

    #[test]
    fn discard_leaves_the_field_bit_identical() {
        let mut buf = buffer();
        let field: Vec<f32> = (0..64).map(|i| i as f32 * 0.125).collect();
        let before = field.clone();

        buf.push(stamp(0, 0, 4, 4, 1.0));
        buf.push(stamp(2, 2, 4, 4, -2.0));
        let mut scratch = vec![0.0f32; 64];
        buf.preview_into(&field, &mut scratch);
        assert_ne!(scratch, before, "the preview really did show a change");

        let dropped = buf.discard();
        assert_eq!(dropped, 2);
        assert!(buf.is_empty());
        // Bit-identical, not merely equal: compare the raw bit patterns so a
        // -0.0/0.0 or NaN-payload difference would still fail.
        let bits_before: Vec<u32> = before.iter().map(|v| v.to_bits()).collect();
        let bits_after: Vec<u32> = field.iter().map(|v| v.to_bits()).collect();
        assert_eq!(bits_after, bits_before);
    }

    #[test]
    fn discard_leaves_no_dirty_marks_behind() {
        let mut buf = buffer();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        buf.push(stamp(0, 0, 4, 4, 1.0));
        buf.discard();
        assert_eq!(tracker.dirty_tiles().count(), 0);
        assert_eq!(tracker.version(0), 0);
        // And a later commit of a fresh draft still starts from version 0+1.
        buf.push(stamp(0, 0, 2, 2, 1.0));
        let mut field = vec![0.0f32; 64];
        buf.commit(&mut field, &mut tracker, "height_edited");
        assert_eq!(tracker.version(0), 1);
    }

    #[test]
    fn commit_discard_cycles_bump_versions_only_on_commit() {
        let mut buf = buffer();
        let mut field = vec![0.0f32; 64];
        let mut tracker = DirtyTracker::new(buf.tile_count());
        for round in 1..=3 {
            // A discarded pass contributes nothing to the version counter.
            buf.push(stamp(0, 0, 2, 2, 100.0));
            buf.discard();
            // A committed pass contributes exactly one.
            buf.push(stamp(0, 0, 2, 2, 1.0));
            buf.commit(&mut field, &mut tracker, "height_edited");
            assert_eq!(tracker.version(0), round);
            assert_eq!(field[0], round as f32);
        }
    }

    // ---- touched tiles ----

    #[test]
    fn touched_tiles_is_the_union_and_shrinks_when_a_stamp_is_removed() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0)); // tile 0
        let i = buf.push(stamp(5, 5, 2, 2, 1.0)); // tile 3
        assert_eq!(buf.touched_tiles().collect::<Vec<_>>(), vec![0, 3]);
        buf.remove(i);
        assert_eq!(buf.touched_tiles().collect::<Vec<_>>(), vec![0]);
    }

    #[test]
    fn touched_tiles_keeps_a_hidden_stamps_footprint() {
        // Hiding a stamp is itself a reason to repaint where it was, so the
        // render scope keeps it even though preview/commit skip it.
        let mut buf = buffer();
        let i = buf.push(stamp(5, 5, 2, 2, 1.0));
        buf.set_hidden(i, true);
        assert_eq!(buf.touched_tiles().collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn a_stamp_touching_nothing_marks_no_tiles() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 0, 0, 1.0));
        assert_eq!(buf.touched_tiles().count(), 0);
        assert_eq!(buf.touched_bounds(), None);
    }

    #[test]
    fn touched_bounds_unions_every_entry() {
        let mut buf = buffer();
        buf.push(stamp(1, 1, 2, 2, 1.0));
        buf.push(stamp(5, 6, 2, 2, 1.0));
        assert_eq!(buf.touched_bounds(), Some(Region::new(1, 1, 6, 7)));
    }

    // ---- the bounded composite ----

    /// A non-uniform base, so a wrong row offset inside
    /// `preview_touched_into` shows up as a value mismatch rather than
    /// hiding in a field of identical numbers.
    fn ramp_base() -> Vec<f32> {
        (0..64).map(|i| i as f32 * 0.25).collect()
    }

    /// The property the whole bounded upload rests on, asserted against
    /// `preview_into` as an oracle rather than a second transcription:
    /// **inside the window the two composites are identical**, and outside
    /// it `preview_touched_into` has not touched a single cell.
    #[test]
    fn preview_touched_into_matches_preview_into_in_the_window() {
        let mut buf = buffer();
        buf.push(stamp(1, 1, 2, 2, 1.0));
        buf.push(stamp(2, 2, 3, 3, 0.5));
        let hidden = buf.push(stamp(5, 5, 2, 2, 4.0));
        buf.set_hidden(hidden, true);

        let base = ramp_base();
        let mut full = vec![f32::NAN; 64];
        buf.preview_into(&base, &mut full);

        // A sentinel no stamp and no base cell can produce, so "untouched"
        // is provable rather than merely plausible.
        const SENTINEL: f32 = -999.0;
        let mut bounded = vec![SENTINEL; 64];
        let win = buf
            .preview_touched_into(&base, &mut bounded)
            .expect("three stamps must touch something");

        // The window is genuinely a window. Without this the test would
        // pass for a `touched_bounds` that gave up and returned the field.
        assert_eq!(win, Region::new(1, 1, 6, 6));
        assert!(win.w < 8 && win.h < 8);

        for y in 0..8 {
            for x in 0..8 {
                let i = y * 8 + x;
                let inside =
                    x >= win.x && x < win.x + win.w && y >= win.y && y < win.y + win.h;
                if inside {
                    assert_eq!(bounded[i], full[i], "cell ({x}, {y}) differs from preview_into");
                } else {
                    assert_eq!(bounded[i], SENTINEL, "cell ({x}, {y}) was written outside the window");
                }
            }
        }

        // And the composite is real, not a copy of the base: two literals
        // an oracle-only test could not distinguish from a no-op.
        assert_eq!(full[9], 3.25); // base 9*0.25, + 1.0 from the first stamp only
        assert_eq!(full[18], 6.0); // base 18*0.25, + 1.0 + 0.5 from both visible stamps
        assert_ne!(bounded[9], base[9]);
    }

    /// The window covers a **hidden** stamp's footprint even though the
    /// stamp is not applied — hiding one is itself a reason to repaint
    /// where it was, and a caller that only re-uploaded the visible union
    /// would leave the hidden stamp on screen forever.
    #[test]
    fn the_window_covers_a_hidden_stamp_that_is_not_applied() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        let hidden = buf.push(stamp(6, 6, 2, 2, 3.0));
        buf.set_hidden(hidden, true);

        let base = ramp_base();
        let mut bounded = vec![f32::NAN; 64];
        let win = buf.preview_touched_into(&base, &mut bounded).unwrap();

        assert_eq!(win, Region::new(0, 0, 8, 8), "the union must include the hidden footprint");
        // ... and inside it the hidden stamp contributes nothing.
        let i = 6 * 8 + 6;
        assert_eq!(bounded[i], base[i]);
    }

    /// `None` here means *"the draft touched nothing"*, which is neither
    /// "nothing needs drawing" nor "everything is dirty". The method writes
    /// **no** cell in that case, so a caller that misread it as an empty
    /// window renders a blank instead of stale pixels.
    #[test]
    fn none_from_preview_touched_into_writes_nothing_at_all() {
        let base = ramp_base();
        const SENTINEL: f32 = -999.0;

        // (a) An empty draft over a base that emphatically does need drawing.
        let buf = buffer();
        let mut scratch = vec![SENTINEL; 64];
        assert_eq!(buf.preview_touched_into(&base, &mut scratch), None);
        assert!(scratch.iter().all(|&v| v == SENTINEL));
        assert!(base.iter().any(|&v| v != 0.0), "the base is not empty; None did not mean 'nothing to draw'");

        // (b) A non-empty draft whose only stamp has a zero-area footprint.
        let mut buf = buffer();
        buf.push(stamp(0, 0, 0, 0, 1.0));
        assert!(!buf.is_empty());
        let mut scratch = vec![SENTINEL; 64];
        assert_eq!(buf.preview_touched_into(&base, &mut scratch), None);
        assert!(scratch.iter().all(|&v| v == SENTINEL));
    }

    // ---- stack order and structural edits ----

    #[test]
    fn stack_order_is_load_bearing_for_order_dependent_stamps() {
        // A "set to a constant" stamp shows what an add-only stamp can't:
        // reordering changes the result, which is why commit bakes in order.
        #[derive(Debug, Clone)]
        struct SetBox {
            area: Region,
            value: f32,
        }
        impl Stamp for SetBox {
            type Cell = f32;
            fn bounds(&self, _w: usize, _h: usize) -> Region {
                self.area
            }
            fn apply(&self, dst: &mut [f32], width: usize, _h: usize) {
                for y in self.area.y..self.area.y + self.area.h {
                    for x in self.area.x..self.area.x + self.area.w {
                        dst[y * width + x] = self.value;
                    }
                }
            }
        }

        let mut buf: PassBuffer<SetBox> = PassBuffer::new(8, 8, 4);
        buf.push(SetBox {
            area: Region::new(0, 0, 2, 2),
            value: 1.0,
        });
        buf.push(SetBox {
            area: Region::new(0, 0, 2, 2),
            value: 2.0,
        });
        let base = vec![0.0f32; 64];
        let mut scratch = vec![0.0f32; 64];
        buf.preview_into(&base, &mut scratch);
        assert_eq!(scratch[0], 2.0, "last stamp in the stack wins");

        buf.move_up(1);
        buf.preview_into(&base, &mut scratch);
        assert_eq!(scratch[0], 1.0, "reordering really changes the result");
    }

    #[test]
    fn move_up_and_down_reject_out_of_range_moves() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 1, 1, 1.0));
        buf.push(stamp(1, 1, 1, 1, 1.0));
        assert!(!buf.move_up(0));
        assert!(!buf.move_down(1));
        assert!(buf.move_down(0));
        assert!(buf.move_up(1));
    }

    // ---- draft-scoped undo/redo ----

    #[test]
    fn undo_reverts_a_structural_edit_without_touching_the_field() {
        let mut buf = buffer();
        let field = vec![0.0f32; 64];
        buf.push(stamp(0, 0, 2, 2, 1.0));
        buf.push(stamp(4, 4, 2, 2, 1.0));
        assert_eq!(buf.len(), 2);

        assert!(buf.undo());
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.touched_tiles().collect::<Vec<_>>(), vec![0]);

        assert!(buf.redo());
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.touched_tiles().collect::<Vec<_>>(), vec![0, 3]);

        assert_eq!(field, vec![0.0f32; 64], "undo/redo is draft-scoped only");
    }

    #[test]
    fn undo_covers_delete_hide_and_reorder_not_just_add() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        buf.push(stamp(4, 4, 2, 2, 1.0));

        buf.set_hidden(0, true);
        assert!(buf.entries()[0].hidden);
        assert!(buf.undo());
        assert!(!buf.entries()[0].hidden);

        buf.move_down(0);
        assert!(buf.undo());

        buf.remove(0);
        assert_eq!(buf.len(), 1);
        assert!(buf.undo());
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn a_new_edit_clears_the_redo_branch() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        buf.push(stamp(4, 4, 2, 2, 1.0));
        buf.undo();
        assert!(buf.can_redo());
        buf.push(stamp(0, 4, 2, 2, 1.0));
        assert!(!buf.can_redo());
    }

    #[test]
    fn undo_and_redo_are_no_ops_on_an_empty_history() {
        let mut buf = buffer();
        assert!(!buf.undo());
        assert!(!buf.redo());
    }

    #[test]
    fn draft_history_is_capped() {
        let mut buf = buffer();
        for _ in 0..(HISTORY_MAX + 10) {
            buf.push(stamp(0, 0, 1, 1, 1.0));
        }
        // Undo as far as it goes: exactly HISTORY_MAX steps survive.
        let mut steps = 0;
        while buf.undo() {
            steps += 1;
        }
        assert_eq!(steps, HISTORY_MAX);
    }

    #[test]
    fn discard_clears_the_draft_history_too() {
        let mut buf = buffer();
        buf.push(stamp(0, 0, 2, 2, 1.0));
        assert!(buf.can_undo());
        buf.discard();
        assert!(!buf.can_undo());
        assert!(!buf.can_redo());
    }

    #[test]
    fn pass_buffer_round_trips_through_json() {
        let mut buf = buffer();
        buf.push(stamp(1, 1, 2, 2, 0.75));
        let json = serde_json::to_string(&buf).unwrap();
        let back: PassBuffer<AddBox> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.touched_tiles().collect::<Vec<_>>(), vec![0]);
        assert_eq!(back.entries()[0].stamp.amount, 0.75);
    }
}
