//! Global heightmap undo — the reference's `pushUndo`/`undoLast`/
//! `updateUndoUI` (reference HTML lines 9548-9565), `PARITY_AUDIT.md` §3.1's
//! "Global heightmap undo (3 functions) — absent", register `ED-01`/`PR-11`.
//!
//! **Not** the Sculpt draft's stamp-history undo. That one is real, older and
//! entirely separate (`cartalith_spatial::PassBuffer::undo`, exposed as
//! `sculpt_undo`/`sculpt_redo`), and the reference itself keeps the two apart
//! in the same words — its own comment at line 9294 calls the stamp history
//! *"draft-scoped — separate from the field-level undoStack/pushUndo"*. A
//! draft undo pops a stamp that was never written to the field; this pops a
//! whole *committed* height field back off a stack.
//!
//! ## What the reference actually does, read rather than remembered
//!
//! ```js
//! const MAX_UNDO=5; const undoStack=[];
//! function pushUndo(){
//!   undoStack.push(field.slice());
//!   if(undoStack.length>MAX_UNDO) undoStack.shift();
//!   updateUndoUI();
//! }
//! function undoLast(){
//!   if(!undoStack.length) return;
//!   field.set(undoStack.pop()); updateUndoUI();
//!   computeFlow(true); refreshClimate(); renderNow();
//! }
//! ```
//!
//! Three facts worth stating because each of them settled a design question
//! here, and one of them corrects this repository's own index:
//!
//! 1. **It snapshots `field` and nothing else.** Not `riverMask`, not
//!    `riverFloor`, not `lakeMask`, not climate, not civ. One
//!    `Float32Array.slice()`.
//! 2. **The depth is 5, not 1.** `reference/FUNCTION_INDEX.md` line 61 says
//!    *"one level per destructive op"*; `MAX_UNDO=5` and the shipped label
//!    reads "Up to 5 steps saved in memory". The index is wrong on that
//!    detail — noted here rather than silently ported around.
//! 3. **Fifteen call sites, all destructive height ops**: the ten erosion /
//!    coastal / glacial / fjord buttons, and `sculptCommit`. Notably *not*
//!    `centerLandmasses`, which mutates the field and is deliberately not
//!    undoable.
//!
//! ## Where this port diverges, and why
//!
//! The reference is a browser app whose grid tops out far below this one's.
//! `MEMORY_OPTIMIZATION_SCOPE.md` measured this port at ~680 MB steady-state
//! at 2048², with the resolution control offering up to 8192². One `f32`
//! height field is **16 MB at 2048², 64 MB at 4096², 256 MB at 8192²**, so a
//! flat "5 deep" rule would quietly commit to **1.25 GB of undo buffer** on
//! the largest world the UI will let a user pick — more than the whole
//! generated world costs.
//!
//! So the bound is **a byte budget first, the reference's step count
//! second**: evict oldest until the stack is within *both*
//! `budget_bytes` (default [`DEFAULT_BUDGET_BYTES`], 256 MiB) and
//! [`MAX_STEPS`] (5, the reference's own). One step is always kept if one was
//! pushed, even when it alone exceeds the budget — an undo that silently
//! isn't there is worse than a big one you were told about. In practice:
//!
//! | Grid | Bytes/step | Steps kept |
//! |---|---:|---:|
//! | 1024² | 4 MB | 5 (count-bound) |
//! | 2048² | 16 MB | 5 (count-bound) |
//! | 4096² | 64 MB | 4 (budget-bound) |
//! | 8192² | 256 MB | 1 (budget floor) |
//!
//! Two smaller divergences, both forced rather than chosen:
//!
//! - **The stack is cleared on every generate/load.** The reference does not
//!   clear it, which is survivable there because its grid never changes size
//!   mid-session; here `generate_sized` can, and restoring a 2048² field over
//!   a 4096² world would be a length mismatch, not an undo. [`Self::restore`]
//!   also refuses a length mismatch outright, so the clear is a policy and
//!   the length check is the guard.
//! - **Undo marks flow/climate stale instead of re-running them.** The
//!   reference's `undoLast` calls `computeFlow(true); refreshClimate();
//!   renderNow()` inline. `WorldGen::undo_last`/`redo_last` mark
//!   `PipelineStage::Height` changed over the whole map and return; the
//!   recompute is `recompute_stale_stages`' to run.
//!
//!   This bullet used to say the port "defers those" because "this port's
//!   whole commit path already defers" them, citing `sculpt_commit`. **That
//!   citation contradicted the claim**: `sculpt_commit` ends in
//!   `mark_and_recompute` (`lib.rs`), and so do `carve_fjords`, the erosion
//!   ops and the paint commit. What undo actually did was neither defer nor
//!   recompute — it wrote `ws.field` and marked *nothing*, so the graph
//!   reported a world that was current while every derived stage still held
//!   values computed from the field the undo had just thrown away. Whole-map
//!   marking is the fix, because a snapshot swap cannot say which tiles
//!   differ; not recomputing stays deliberate, since this runs synchronously
//!   on the main thread with no progress channel.
//!
//! **What it does not revert, exactly as in the reference:** `river_mask` /
//! `river_floor` locks written by a Sculpt commit's water hooks. Reverting
//! the height without the locks leaves a later commit's
//! `enforce_river_channels` re-clamping cells to a floor whose channel is no
//! longer carved. That is the reference's own behaviour, and matching it
//! costs 0 MB where diverging costs +130 % per step (a `u8` mask plus a
//! second `f32` field). Stated rather than fixed.

use std::collections::VecDeque;

/// The reference's `MAX_UNDO`. Kept literally; the byte budget is what
/// actually binds on a large world.
pub const MAX_STEPS: usize = 5;

/// 256 MiB. Chosen against `MEMORY_OPTIMIZATION_SCOPE.md`'s measured
/// ~680 MB steady-state at 2048²: it buys the reference's full five steps at
/// every resolution up to and including 2048² (80 MB) while capping the
/// 8192² case at one step instead of five.
pub const DEFAULT_BUDGET_BYTES: usize = 256 * 1024 * 1024;

/// One reverted-to height field, with the name of the operation that was
/// about to overwrite it.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoStep {
    /// Short operation label (`"Sculpt commit"`, `"Carve fjords"`) — what the
    /// Edit menu shows after "Undo", the way every DCC labels the row.
    pub label: String,
    /// The height field as it was *before* that operation.
    pub field: Vec<f32>,
}

impl UndoStep {
    fn bytes(&self) -> usize {
        self.field.len() * std::mem::size_of::<f32>()
    }
}

/// A bounded stack of pre-operation height fields. Oldest at the front.
#[derive(Debug)]
pub struct HeightUndo {
    stack: VecDeque<UndoStep>,
    /// Live sum of `stack`'s step bytes — maintained on push/pop rather than
    /// recomputed, so `stats()` is free enough for a menu's `about_to_popup`.
    bytes: usize,
    budget_bytes: usize,
}

impl Default for HeightUndo {
    fn default() -> Self {
        Self::new()
    }
}

impl HeightUndo {
    pub fn new() -> Self {
        Self { stack: VecDeque::new(), bytes: 0, budget_bytes: DEFAULT_BUDGET_BYTES }
    }

    /// Snapshot `field` before a destructive operation. `label` names the
    /// operation, not the state.
    ///
    /// A zero-length field is not pushed: there is no world to revert to, and
    /// an empty step would occupy a slot that a real one could use.
    pub fn push(&mut self, label: &str, field: &[f32]) {
        if field.is_empty() {
            return;
        }
        let step = UndoStep { label: label.to_string(), field: field.to_vec() };
        self.bytes += step.bytes();
        self.stack.push_back(step);
        self.evict();
    }

    /// Drop oldest steps until the stack satisfies both bounds — but never
    /// below one step, so a single snapshot larger than the whole budget is
    /// still an undo rather than a silent no-op.
    fn evict(&mut self) {
        while self.stack.len() > 1 && (self.stack.len() > MAX_STEPS || self.bytes > self.budget_bytes) {
            if let Some(dropped) = self.stack.pop_front() {
                self.bytes -= dropped.bytes();
            }
        }
    }

    /// Pop the newest step and write it back over `field`.
    ///
    /// Returns the step's label on success. `None` when the stack is empty,
    /// or when the snapshot's length no longer matches the live field — the
    /// grid was resized under it, and half-restoring a mismatched field would
    /// be worse than refusing. A refused step is still popped: it can never
    /// become valid again, and leaving it would wedge the stack.
    pub fn restore(&mut self, field: &mut [f32]) -> Option<String> {
        let step = self.stack.pop_back()?;
        self.bytes -= step.bytes();
        if step.field.len() != field.len() {
            return None;
        }
        field.copy_from_slice(&step.field);
        Some(step.label)
    }

    /// Drop everything. Called on generate/load — see the module doc.
    pub fn clear(&mut self) {
        self.stack.clear();
        self.bytes = 0;
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Bytes currently held by the stack. This is the number the Preferences
    /// ▸ Memory row and the reference's own `#undoMem` readout report.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    /// Label of the step [`Self::restore`] would pop next, for the Edit
    /// menu's "Undo <operation>" row.
    pub fn next_label(&self) -> Option<&str> {
        self.stack.back().map(|s| s.label.as_str())
    }

    /// Every held step's label, **newest first** — the order
    /// [`Self::restore`] hands them back in, so the first element is always
    /// [`Self::next_label`].
    ///
    /// The stack has held these labels since it was written; only the newest
    /// one was reachable, so a caller could name the step it was about to
    /// pop and nothing else. Exposing the rest costs no memory, retains
    /// nothing extra and moves no bound — [`Self::evict`] still decides what
    /// is in the stack, and this only reads it. Which is the point: a panel
    /// listing undone steps does not need undo to keep more, it needs to see
    /// what undo already keeps.
    pub fn labels(&self) -> impl ExactSizeIterator<Item = &str> {
        self.stack.iter().rev().map(|s| s.label.as_str())
    }

    /// Re-budget, evicting immediately if the new budget is smaller.
    /// Clamped to at least one 1024² field (4 MiB) so a caller cannot set a
    /// budget that makes every push a one-step stack by accident.
    pub fn set_budget_bytes(&mut self, bytes: usize) {
        self.budget_bytes = bytes.max(4 * 1024 * 1024);
        self.evict();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(n: usize, v: f32) -> Vec<f32> {
        vec![v; n]
    }

    #[test]
    fn empty_stack_restores_nothing() {
        let mut u = HeightUndo::new();
        let mut f = field(4, 1.0);
        assert_eq!(u.restore(&mut f), None);
        assert_eq!(f, field(4, 1.0));
        assert_eq!(u.depth(), 0);
        assert_eq!(u.bytes(), 0);
        assert_eq!(u.next_label(), None);
    }

    #[test]
    fn push_then_restore_reverts_the_field() {
        let mut u = HeightUndo::new();
        let mut f = field(4, 1.0);
        u.push("Carve fjords", &f);
        f.iter_mut().for_each(|v| *v = 9.0);
        assert_eq!(u.restore(&mut f).as_deref(), Some("Carve fjords"));
        assert_eq!(f, field(4, 1.0));
        assert!(u.is_empty());
    }

    #[test]
    fn restores_in_reverse_order() {
        let mut u = HeightUndo::new();
        let mut f = field(2, 0.0);
        for step in 1..=3 {
            u.push(&format!("op{step}"), &f);
            f.iter_mut().for_each(|v| *v = step as f32);
        }
        assert_eq!(u.depth(), 3);
        assert_eq!(u.next_label(), Some("op3"));
        assert_eq!(u.restore(&mut f).as_deref(), Some("op3"));
        assert_eq!(f, field(2, 2.0));
        assert_eq!(u.restore(&mut f).as_deref(), Some("op2"));
        assert_eq!(f, field(2, 1.0));
        assert_eq!(u.restore(&mut f).as_deref(), Some("op1"));
        assert_eq!(f, field(2, 0.0));
        assert_eq!(u.restore(&mut f), None);
    }

    /// The reference's `MAX_UNDO=5` with `undoStack.shift()`: the sixth push
    /// evicts the first, and the oldest reachable state is the fifth-from-last.
    #[test]
    fn count_bound_evicts_oldest() {
        let mut u = HeightUndo::new();
        let mut f = field(2, 0.0);
        for step in 0..8 {
            u.push(&format!("op{step}"), &f);
            f.iter_mut().for_each(|v| *v = (step + 1) as f32);
        }
        assert_eq!(u.depth(), MAX_STEPS);
        assert_eq!(u.next_label(), Some("op7"));
        // Five survivors: op3..op7, holding the pre-op values 3..7.
        for expected in (3..=7).rev() {
            assert_eq!(u.restore(&mut f).as_deref(), Some(format!("op{expected}").as_str()));
            assert_eq!(f, field(2, expected as f32));
        }
        assert!(u.is_empty());
    }

    /// `labels()` is `next_label()` extended to the whole stack, in the same
    /// direction: newest first, first element identical to `next_label()`,
    /// and it shortens as steps are popped.
    #[test]
    fn labels_list_the_whole_stack_newest_first() {
        let mut u = HeightUndo::new();
        let mut f = field(2, 0.0);
        assert_eq!(u.labels().len(), 0, "an empty stack names nothing");
        assert_eq!(u.labels().next(), None);
        for step in 1..=3 {
            u.push(&format!("op{step}"), &f);
            f.iter_mut().for_each(|v| *v = step as f32);
        }
        assert_eq!(u.labels().collect::<Vec<_>>(), vec!["op3", "op2", "op1"]);
        assert_eq!(u.labels().len(), u.depth());
        assert_eq!(u.labels().next(), u.next_label(), "the head must be the next pop");
        u.restore(&mut f);
        assert_eq!(u.labels().collect::<Vec<_>>(), vec!["op2", "op1"]);
        u.clear();
        assert_eq!(u.labels().len(), 0);
    }

    /// The list is a view of what the stack **actually holds**, not of what
    /// was pushed: an evicted step is not named, exactly as it is not
    /// counted by `depth()` and not restorable. Eight pushes, five survivors.
    #[test]
    fn labels_do_not_name_evicted_steps() {
        let mut u = HeightUndo::new();
        let f = field(2, 0.0);
        for step in 0..8 {
            u.push(&format!("op{step}"), &f);
        }
        assert_eq!(u.labels().collect::<Vec<_>>(), vec!["op7", "op6", "op5", "op4", "op3"]);

        // ...and the budget bound drops names the same way the count bound
        // does. 4 MiB steps against a 9 MiB budget: two survive, two names.
        let mut b = HeightUndo::new();
        let big = field(1024 * 1024, 0.0);
        for step in 0..5 {
            b.push(&format!("op{step}"), &big);
        }
        assert_eq!(b.labels().len(), 5);
        b.set_budget_bytes(9 * 1024 * 1024);
        assert_eq!(b.labels().collect::<Vec<_>>(), vec!["op4", "op3"]);
        assert_eq!(b.labels().len(), b.depth(), "the list and the depth cannot disagree");
    }

    #[test]
    fn bytes_tracks_pushes_and_pops() {
        let mut u = HeightUndo::new();
        let mut f = field(1000, 0.0);
        u.push("a", &f);
        assert_eq!(u.bytes(), 4000);
        u.push("b", &f);
        assert_eq!(u.bytes(), 8000);
        u.restore(&mut f);
        assert_eq!(u.bytes(), 4000);
        u.clear();
        assert_eq!(u.bytes(), 0);
        assert_eq!(u.depth(), 0);
    }

    /// The budget, not the count, is what binds on a big world. 10 MiB of
    /// budget against 4 MiB steps holds two, not five.
    #[test]
    fn budget_bound_evicts_before_the_count_bound() {
        let mut u = HeightUndo::new();
        u.set_budget_bytes(10 * 1024 * 1024);
        let f = field(1024 * 1024, 0.0); // 4 MiB
        for step in 0..5 {
            u.push(&format!("op{step}"), &f);
        }
        assert_eq!(u.depth(), 2);
        assert!(u.bytes() <= u.budget_bytes());
        assert_eq!(u.next_label(), Some("op4"));
    }

    /// A single step larger than the whole budget is still kept: one big undo
    /// beats a silently absent one. This is the 8192² case.
    #[test]
    fn one_step_survives_an_impossible_budget() {
        let mut u = HeightUndo::new();
        u.set_budget_bytes(4 * 1024 * 1024);
        let mut f = field(4 * 1024 * 1024, 0.5); // 16 MiB, > budget
        u.push("Sculpt commit", &f);
        u.push("Sculpt commit", &f);
        assert_eq!(u.depth(), 1);
        assert!(u.bytes() > u.budget_bytes());
        f[0] = 9.0;
        assert!(u.restore(&mut f).is_some());
        assert_eq!(f[0], 0.5);
    }

    #[test]
    fn shrinking_the_budget_evicts_immediately() {
        let mut u = HeightUndo::new();
        let f = field(1024 * 1024, 0.0); // 4 MiB
        for step in 0..5 {
            u.push(&format!("op{step}"), &f);
        }
        assert_eq!(u.depth(), 5);
        u.set_budget_bytes(9 * 1024 * 1024);
        assert_eq!(u.depth(), 2);
        assert_eq!(u.bytes(), 8 * 1024 * 1024);
    }

    /// The floor stops a caller from setting a budget that would make undo
    /// useless at every resolution.
    #[test]
    fn budget_has_a_floor() {
        let mut u = HeightUndo::new();
        u.set_budget_bytes(0);
        assert_eq!(u.budget_bytes(), 4 * 1024 * 1024);
    }

    /// A resized grid: the snapshot is refused *and* dropped, so the next
    /// undo reaches a step that might still fit rather than jamming on this
    /// one forever.
    #[test]
    fn length_mismatch_is_refused_and_dropped() {
        let mut u = HeightUndo::new();
        u.push("op", &field(4, 1.0));
        let mut resized = field(9, 7.0);
        assert_eq!(u.restore(&mut resized), None);
        assert_eq!(resized, field(9, 7.0), "a refused restore must not partially write");
        assert!(u.is_empty(), "a permanently-unusable step is dropped, not left to wedge the stack");
    }

    #[test]
    fn empty_field_is_not_pushed() {
        let mut u = HeightUndo::new();
        u.push("op", &[]);
        assert!(u.is_empty());
        assert_eq!(u.bytes(), 0);
    }

    #[test]
    fn clear_resets_everything() {
        let mut u = HeightUndo::new();
        let mut f = field(16, 1.0);
        u.push("a", &f);
        u.push("b", &f);
        u.clear();
        assert!(u.is_empty());
        assert_eq!(u.bytes(), 0);
        assert_eq!(u.next_label(), None);
        assert_eq!(u.restore(&mut f), None);
    }
}

// ============================================================== the ledger ==

/// What kind of thing a [`LedgerEntry`] is, and therefore what can be done
/// about it.
///
/// | Kind | What it means |
/// |---|---|
/// | [`EntryKind::HeightSnapshot`] | A pre-operation height field is held for it. Reverting is real. |
/// | [`EntryKind::Recorded`] | It happened; no snapshot exists, and the row carries the reason. |
/// | [`EntryKind::Floor`] | A generate or a load. History starts here; nothing before it survives. |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A height snapshot was pushed for this entry, in the same call.
    HeightSnapshot,
    /// It happened and cannot be walked back. The `&'static str` is the
    /// reason, shown on the row -- never "not implemented".
    Recorded(&'static str),
    /// A generate or a load: everything before it is gone.
    Floor,
}

/// One row of the ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    /// Monotonic, never reused within a session. What the shell passes back
    /// to [`HistoryLedger::steps_to_revert_to`].
    pub seq: u64,
    /// `height`, `paint`, `civ`, `world` -- the icon column, and the field
    /// per-subsystem reversal will key on when it lands.
    pub subsystem: &'static str,
    /// The operation, in the same words the Edit menu uses.
    pub label: String,
    /// What it touched. Free text, built by the call site, because only the
    /// call site knows what the interesting extent is.
    pub detail: String,
    /// Milliseconds since the Unix epoch. `0` when the clock refused.
    pub at_ms: u64,
    pub kind: EntryKind,
}

/// How many rows are kept. Generous because a `Recorded` row is a label and
/// a timestamp -- the memory that matters is [`HeightUndo`]'s, and that has
/// its own byte budget.
pub const MAX_LEDGER: usize = 200;

/// The history **ledger** -- `GUI_GAP_REGISTER.md` **ED-02**,
/// `DCC_SHELL_SPEC.md` section 7.1's proposals 1 and 3.
///
/// ## Why this is not a five-row list of [`HeightUndo`]'s labels
///
/// A previous pass declined to build that list, and the reason it gave is
/// the design constraint here: 7.1 asks for a *ledger with per-subsystem
/// reversal*, and shipping the flat list would have answered the easy half
/// of ED-02 while foreclosing the hard one. The hard half is that this
/// application has seven edit domains with three commit models, and a
/// history panel that shows one of them is a history that lies by omission.
///
/// So the ledger **records every commit** and **reverses the ones it can**,
/// and says per row which it is (see [`EntryKind`]). That is strictly more
/// honest than the list, and it is the shape per-subsystem reversal drops
/// into later: a `Recorded` row already knows its subsystem, so turning one
/// on is a kind change rather than a redesign.
///
/// ## What makes a row reversible is [`HeightUndo`], not this type
///
/// The two are deliberately **not** cross-wired. A `HeightSnapshot` row is
/// reversible exactly while its snapshot is still on the stack, and the
/// stack evicts on its own byte budget -- so [`HistoryLedger::rows`] takes
/// the live depth and marks the newest `depth` height rows reversible and
/// every older one not. One source of truth for "is there a snapshot",
/// asked at read time, rather than two structures that can disagree.
///
/// ## Linear, and only linear
///
/// Photoshop's non-linear history is a documented source of user confusion
/// and this engine has no cheap way to re-apply a divergent branch over a
/// regenerated world (7.1 proposal 3's own conclusion). Reverting to a row
/// pops everything above it, and the row reverted to leaves with them -- the
/// snapshot *is* the state before that operation, so once it is restored the
/// operation is not part of the history any more.
///
/// ## The `COMMITTED` boundary lives here, and it is a mark, not a row
///
/// `design/proposed-2026-09-05/Main.dc.html` draws a labelled rule across the
/// step list -- rows at or below it are inside the project file on disk, rows
/// above it are not -- and its footnote's third clause is *"reverting above
/// COMMITTED asks first"*. The shell derived that boundary itself until
/// 2026-09-06, from `EngineBridge.project_saved`, which is correct within one
/// session and **blank for a project saved in an earlier one**.
///
/// **Adding a `saved` field to a [`LedgerEntry`] would not have fixed that,
/// and the measurement is the reason this is a mark instead.** The ledger
/// does not survive a reload at all: `WorldGen::load_save` records an
/// [`EntryKind::Floor`], and [`Self::record`]'s first statement is
/// `self.entries.clear()` for that kind. Every row written before the save is
/// gone by the time the panel asks the question, so a per-row flag would be a
/// field that is never read back.
///
/// So the boundary is three scalars on the ledger itself:
///
/// | | |
/// |---|---|
/// | [`Self::saved_seq`] | highest `seq` known to be in the file. `None` = no such point |
/// | [`Self::saved_at_ms`] | when the file was written, Unix ms. `None` is a real answer -- placeable rule, unknown age |
/// | [`Self::saved_reverted_past`] | a revert unwound past the mark. A file exists and no row here is in it |
///
/// **`None` and "seq 0" are different claims and are spelled differently.**
/// A world that has never been saved has `saved_seq == None`, and the panel
/// draws no rule rather than one at the bottom of the list.
///
/// ### The save format did not move for this
///
/// The cross-session half is [`WorldGen::load_save`] calling
/// [`Self::mark_saved`] with [`file_written_at_ms`] of the archive it just
/// read -- the file's own last-write time, which every archive has and no
/// writer had to start emitting. An archive written by any earlier build of
/// this port opens with a correctly placed rule and a correct age, because
/// nothing inside the archive is consulted for either.
///
/// [`WorldGen::load_save`]: crate::WorldGen
#[derive(Debug, Default)]
pub struct HistoryLedger {
    entries: VecDeque<LedgerEntry>,
    next_seq: u64,
    /// Highest `seq` inside the project file on disk. See the type's own doc
    /// for why this is a mark and not a column on [`LedgerEntry`].
    saved_seq: Option<u64>,
    /// Unix-epoch milliseconds the file was written, when that could be
    /// established at all.
    saved_at_ms: Option<u64>,
    /// Set by [`Self::truncate_to`] / [`Self::pop_newest_height`] when the
    /// undone rows reach the mark -- the state on screen is now *older* than
    /// the file, which is neither "saved" nor "never saved".
    saved_reverted_past: bool,
}

impl HistoryLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Unix-epoch milliseconds, or `None` when the clock refuses. Callers
    /// that must produce a number use [`Self::now_ms`]; callers recording a
    /// *claim* about when something happened keep the `None`.
    fn now_ms_opt() -> Option<u64> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis() as u64)
    }

    fn now_ms() -> u64 {
        Self::now_ms_opt().unwrap_or(0)
    }

    /// Append one entry and return its `seq`.
    ///
    /// A [`EntryKind::Floor`] entry **clears everything before it** rather
    /// than merely marking a boundary, because that is what actually
    /// happens: `HeightUndo::clear` runs on every generate and load, so a row
    /// above a floor could never be reverted to and drawing it would be an
    /// offer the engine cannot keep.
    pub fn record(
        &mut self,
        subsystem: &'static str,
        label: impl Into<String>,
        detail: impl Into<String>,
        kind: EntryKind,
    ) -> u64 {
        if kind == EntryKind::Floor {
            self.entries.clear();
            // ...and the saved mark with them. A generate produces a world
            // that is on no disk anywhere; a load produces one that is, and
            // `WorldGen::load_save` re-marks immediately after this call
            // rather than this method guessing which floor it is holding.
            self.saved_seq = None;
            self.saved_at_ms = None;
            self.saved_reverted_past = false;
        }
        self.next_seq += 1;
        self.entries.push_back(LedgerEntry {
            seq: self.next_seq,
            subsystem,
            label: label.into(),
            detail: detail.into(),
            at_ms: Self::now_ms(),
            kind,
        });
        while self.entries.len() > MAX_LEDGER {
            self.entries.pop_front();
        }
        self.next_seq
    }

    /// How many rows are held. **Test-only**, and gated so it says so:
    /// `rows()` is what every production reader uses, and this and a
    /// matching `is_empty()` sat next to it unconditionally with no caller
    /// outside `#[cfg(test)]` — `cargo check` reported both as dead while
    /// `cargo test` still compiled them, so the warning was permanent and
    /// ignorable. `is_empty()` had no caller at all and is gone.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every row, oldest first, each paired with whether a snapshot is still
    /// held for it.
    ///
    /// `height_depth` is [`HeightUndo::depth`]: the newest that many
    /// `HeightSnapshot` rows are reversible and every older one is not,
    /// because the stack evicts oldest-first. This is the only place the two
    /// structures meet, and they meet at read time so they cannot drift.
    pub fn rows(&self, height_depth: usize) -> Vec<(&LedgerEntry, bool)> {
        let mut seen = 0usize;
        let mut flags: Vec<bool> = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter().rev() {
            let live = e.kind == EntryKind::HeightSnapshot && seen < height_depth;
            if e.kind == EntryKind::HeightSnapshot {
                seen += 1;
            }
            flags.push(live);
        }
        flags.reverse();
        self.entries.iter().zip(flags).collect()
    }

    /// How many [`HeightUndo::restore`] calls it takes to get back to `seq`,
    /// or `None` when that row is not a live snapshot.
    ///
    /// The count is *"this row plus every height snapshot above it"* -- the
    /// linear rule stated as arithmetic. `Recorded` rows above it are not
    /// counted because there is nothing to pop for them; they are dropped
    /// from the ledger by [`Self::truncate_to`] all the same, since claiming
    /// an operation is still in effect after the field under it was rolled
    /// back would be the worse lie.
    pub fn steps_to_revert_to(&self, seq: u64, height_depth: usize) -> Option<usize> {
        let rows = self.rows(height_depth);
        let idx = rows.iter().position(|(e, _)| e.seq == seq)?;
        if !rows[idx].1 {
            return None;
        }
        Some(rows[idx..].iter().filter(|(e, _)| e.kind == EntryKind::HeightSnapshot).count())
    }

    /// Drop `seq` and everything after it -- what a successful revert leaves
    /// behind.
    ///
    /// Reverting *to* a row at or below the saved mark throws away work that
    /// is in the file, so the boundary stops being placeable and says which
    /// way it went. The panel's own confirmation already warns about exactly
    /// this gesture (*"this reverts past the COMMITTED rule"*); leaving the
    /// mark standing afterwards would draw the rule below every remaining row
    /// and claim the file matches a state it does not.
    pub fn truncate_to(&mut self, seq: u64) {
        while self.entries.back().is_some_and(|e| e.seq >= seq) {
            self.entries.pop_back();
        }
        self.invalidate_mark_at_or_below(seq);
    }

    /// Drop the newest height row -- for the plain `Edit > Undo`, which pops
    /// one snapshot without going through a row.
    pub fn pop_newest_height(&mut self) {
        if let Some(pos) = self.entries.iter().rposition(|e| e.kind == EntryKind::HeightSnapshot) {
            let seq = self.entries[pos].seq;
            self.entries.remove(pos);
            // The newest *height* row is not always the newest row: a paint
            // or civ commit above it is `Recorded` and is not popped here. So
            // this asks the removed row's own seq rather than assuming the
            // undo happened above the mark.
            self.invalidate_mark_at_or_below(seq);
        }
    }

    /// Shared by the two removal paths: a row at or below [`Self::saved_seq`]
    /// has just been undone, so the file on disk no longer matches anything
    /// still listed.
    ///
    /// Eviction by [`MAX_LEDGER`] deliberately does **not** call this -- those
    /// rows are older than the save and are still in the file; forgetting
    /// their names is not unwinding them.
    fn invalidate_mark_at_or_below(&mut self, seq: u64) {
        if self.saved_seq.is_some_and(|s| seq <= s) {
            self.saved_seq = None;
            self.saved_at_ms = None;
            self.saved_reverted_past = true;
        }
    }

    /// Everything at or below the newest `seq` issued is now in the project
    /// file on disk. `at_ms` is when that file was written, Unix ms, and
    /// **stays `None` when it is not known** -- a placeable rule whose age is
    /// unknown is a different thing from one drawn at "just now".
    ///
    /// Called from two places and they are the two ways a world can equal a
    /// file: `project_save_with_documents` after the bytes land, and
    /// `load_save` after the archive's own floor row.
    pub fn mark_saved(&mut self, at_ms: Option<u64>) {
        self.saved_seq = Some(self.next_seq);
        self.saved_at_ms = at_ms;
        self.saved_reverted_past = false;
    }

    /// [`Self::mark_saved`] with this machine's clock -- the save side, where
    /// the write just happened here.
    pub fn mark_saved_now(&mut self) {
        self.mark_saved(Self::now_ms_opt());
    }

    /// Highest `seq` that is inside the file on disk, or `None` when no such
    /// point is known.
    pub fn saved_seq(&self) -> Option<u64> {
        self.saved_seq
    }

    /// When the file was written, Unix ms. `None` even while
    /// [`Self::saved_seq`] is `Some` is a legitimate pair.
    pub fn saved_at_ms(&self) -> Option<u64> {
        self.saved_at_ms
    }

    /// Whether the mark was lost to a revert rather than never set. The panel
    /// says different things for the two, because they are different facts.
    pub fn saved_reverted_past(&self) -> bool {
        self.saved_reverted_past
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        // `clear_undo()` throws the snapshots away, so no row here could be
        // reverted to anyway. The mark goes with them rather than surviving
        // as a boundary across an empty list.
        self.saved_seq = None;
        self.saved_at_ms = None;
        self.saved_reverted_past = false;
    }
}

/// Unix-epoch milliseconds of `path`'s last write, or `None` when the
/// filesystem will not say.
///
/// This is the whole of the cross-session `COMMITTED` boundary's age, and it
/// is why the save format did not have to change: a project archive's own
/// mtime is when it was last written, which is what a save is, and every
/// archive ever written by this port has one. A stamp inside the archive
/// would have been a format addition whose absence in older files had to be
/// spelled "unknown" — this reaches the same answer for those files instead
/// of the same gap.
///
/// `None` on any refusal (the file was moved between the read and this call,
/// a filesystem with no mtime, a platform error). The caller keeps the `None`
/// rather than substituting `now`, which would report a years-old project as
/// saved this minute.
pub fn file_written_at_ms(path: &str) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

#[cfg(test)]
mod ledger_tests {
    use super::*;

    fn ledger() -> HistoryLedger {
        let mut l = HistoryLedger::new();
        l.record("world", "Generate world", "seed 1", EntryKind::Floor);
        l.record("civ", "Settlement dropped", "Sedge Ford", EntryKind::Recorded("no snapshot"));
        l.record("height", "Carve fjords", "512 sq", EntryKind::HeightSnapshot);
        l.record("paint", "Paint commit", "412 cells", EntryKind::Recorded("no snapshot"));
        l.record("height", "Sculpt commit", "512 sq", EntryKind::HeightSnapshot);
        l
    }

    #[test]
    fn a_floor_clears_everything_before_it() {
        let mut l = ledger();
        assert_eq!(l.len(), 5);
        l.record("world", "Generate world", "seed 2", EntryKind::Floor);
        assert_eq!(l.len(), 1, "a generate is a floor, not a divider");
        assert_eq!(l.rows(0)[0].0.label, "Generate world");
    }

    /// The one place the two structures meet: with two snapshots on the
    /// stack both height rows are live; with one, only the newer is.
    #[test]
    fn reversibility_follows_the_live_stack_depth() {
        let l = ledger();
        let live: Vec<bool> = l.rows(2).iter().map(|(_, b)| *b).collect();
        assert_eq!(live, vec![false, false, true, false, true]);
        let live1: Vec<bool> = l.rows(1).iter().map(|(_, b)| *b).collect();
        assert_eq!(live1, vec![false, false, false, false, true], "an evicted row is not offered");
        let live0: Vec<bool> = l.rows(0).iter().map(|(_, b)| *b).collect();
        assert!(live0.iter().all(|b| !b), "an empty stack offers nothing");
    }

    #[test]
    fn a_recorded_row_is_never_reversible_however_deep_the_stack() {
        let l = ledger();
        for depth in 0..6 {
            for (e, live) in l.rows(depth) {
                if e.kind != EntryKind::HeightSnapshot {
                    assert!(!live, "{} was offered as reversible", e.label);
                }
            }
        }
    }

    #[test]
    fn reverting_counts_this_row_and_every_snapshot_above_it() {
        let l = ledger();
        let rows = l.rows(2);
        let fjords = rows.iter().find(|(e, _)| e.label == "Carve fjords").unwrap().0.seq;
        let sculpt = rows.iter().find(|(e, _)| e.label == "Sculpt commit").unwrap().0.seq;
        assert_eq!(l.steps_to_revert_to(sculpt, 2), Some(1));
        assert_eq!(l.steps_to_revert_to(fjords, 2), Some(2));
        // with only one snapshot left, the older row is not an offer at all
        assert_eq!(l.steps_to_revert_to(fjords, 1), None);
        // and neither is a recorded row, ever
        let paint = rows.iter().find(|(e, _)| e.label == "Paint commit").unwrap().0.seq;
        assert_eq!(l.steps_to_revert_to(paint, 2), None);
        assert_eq!(l.steps_to_revert_to(9999, 2), None, "an unknown seq is not a panic");
    }

    #[test]
    fn a_revert_takes_everything_above_it_with_it() {
        let mut l = ledger();
        let seqs: Vec<u64> = l.rows(2).iter().map(|(e, _)| e.seq).collect();
        l.truncate_to(seqs[2]);
        let left: Vec<&str> = l.rows(0).iter().map(|(e, _)| e.label.as_str()).collect();
        assert_eq!(left, vec!["Generate world", "Settlement dropped"]);
    }

    #[test]
    fn a_plain_undo_removes_the_newest_snapshot_and_not_a_recorded_row() {
        let mut l = ledger();
        l.pop_newest_height();
        let left: Vec<&str> = l.rows(0).iter().map(|(e, _)| e.label.as_str()).collect();
        assert_eq!(
            left,
            vec!["Generate world", "Settlement dropped", "Carve fjords", "Paint commit"]
        );
        l.pop_newest_height();
        let left2: Vec<&str> = l.rows(0).iter().map(|(e, _)| e.label.as_str()).collect();
        assert_eq!(left2, vec!["Generate world", "Settlement dropped", "Paint commit"]);
        // with no snapshots left it is a no-op, not a pop of something else
        l.pop_newest_height();
        assert_eq!(l.len(), 3);
    }

    #[test]
    fn the_ledger_is_bounded_and_drops_the_oldest() {
        let mut l = HistoryLedger::new();
        for i in 0..(MAX_LEDGER + 20) {
            l.record("civ", format!("op{i}"), "", EntryKind::Recorded("r"));
        }
        assert_eq!(l.len(), MAX_LEDGER);
        assert_eq!(l.rows(0)[0].0.label, "op20", "the oldest twenty were dropped");
    }

    /// `seq` is what the shell round-trips, so it must never repeat within a
    /// session -- including across a floor, which clears the rows but not the
    /// counter.
    // -- the `COMMITTED` boundary ------------------------------------------
    //
    // A wall-clock instant used as a literal, so the assertions below pin the
    // value that was handed in rather than whatever `now_ms_opt()` returns:
    // 2023-11-14T22:13:20Z.
    const WRITTEN_AT: u64 = 1_700_000_000_000;

    /// The never-saved case, which the panel must draw as *no rule* rather
    /// than a rule at seq 0. All three answers are absent, and absent is a
    /// different value from zero.
    #[test]
    fn a_world_that_was_never_saved_has_no_boundary() {
        let l = ledger();
        assert_eq!(l.saved_seq(), None);
        assert_eq!(l.saved_at_ms(), None);
        assert!(!l.saved_reverted_past(), "never saved is not the same fact as reverted past");
    }

    /// The mark sits at the newest row, stated two ways: against the row's own
    /// `seq` (the relationship the panel needs -- everything at or below it is
    /// in the file) and against the literal `5`, since `ledger()` records five
    /// rows into a fresh counter.
    #[test]
    fn mark_saved_puts_the_boundary_at_the_newest_row() {
        let mut l = ledger();
        l.mark_saved(Some(WRITTEN_AT));
        let rows = l.rows(0);
        let newest = rows[rows.len() - 1].0.seq;
        assert_eq!(l.saved_seq(), Some(newest));
        assert_eq!(l.saved_seq(), Some(5));
        assert_eq!(l.saved_at_ms(), Some(WRITTEN_AT));
        assert!(!l.saved_reverted_past());
    }

    /// A file whose write time the filesystem would not give up: the rule is
    /// still placeable and its age is still unknown. Two facts, and the second
    /// is not allowed to become "just now".
    #[test]
    fn an_unknown_write_time_stays_unknown() {
        let mut l = ledger();
        l.mark_saved(None);
        assert_eq!(l.saved_seq(), Some(5), "the boundary is placeable without a time");
        assert_eq!(l.saved_at_ms(), None);
    }

    /// The generate case. A floor clears the rows, and the mark goes with them
    /// -- a freshly generated world is on no disk anywhere.
    #[test]
    fn a_generate_floor_clears_the_boundary() {
        let mut l = ledger();
        l.mark_saved(Some(WRITTEN_AT));
        assert_eq!(l.saved_seq(), Some(5));
        l.record("world", "Generate world", "seed 2", EntryKind::Floor);
        assert_eq!(l.saved_seq(), None);
        assert_eq!(l.saved_at_ms(), None);
        assert!(!l.saved_reverted_past());
    }

    /// The cross-session case at this type's own level, in the order
    /// `WorldGen::load_save` runs it: the load's floor clears everything, and
    /// the mark is set on the floor immediately after. The boundary is then
    /// the only row, and every later edit lands above it.
    #[test]
    fn a_load_marks_the_floor_it_just_recorded() {
        let mut l = ledger();
        let floor = l.record("world", "Open project", "512 x 512", EntryKind::Floor);
        l.mark_saved(Some(WRITTEN_AT));
        assert_eq!(l.saved_seq(), Some(floor));
        assert_eq!(l.saved_at_ms(), Some(WRITTEN_AT));
        assert_eq!(l.len(), 1, "the floor cleared the rows a previous session left");

        let after = l.record("height", "Carve fjords", "512 sq", EntryKind::HeightSnapshot);
        assert!(after > l.saved_seq().unwrap(), "an edit after the load is above the rule");
        assert_eq!(l.saved_seq(), Some(floor), "and does not move it");
    }

    /// Reverting *to* the saved row, or below it, unwinds work that is in the
    /// file -- so the boundary stops being placeable and says which of the two
    /// reasons it is. Reverting above it leaves the mark alone.
    #[test]
    fn reverting_at_or_below_the_boundary_replaces_it_with_a_reason() {
        // Five rows, saved (mark at 5), then two edits since the save.
        fn saved_then_edited() -> HistoryLedger {
            let mut l = ledger();
            l.mark_saved(Some(WRITTEN_AT));
            l.record("civ", "Rename", "Sevjuniana", EntryKind::Recorded("no snapshot"));
            l.record("height", "Sculpt commit", "512 sq", EntryKind::HeightSnapshot);
            l
        }
        assert_eq!(saved_then_edited().saved_seq(), Some(5));

        // Above the rule: rows 6 and 7 are unsaved work, and unwinding them
        // leaves the file's own state exactly where it was.
        let mut above = saved_then_edited();
        above.truncate_to(6);
        assert_eq!(above.saved_seq(), Some(5));
        assert_eq!(above.saved_at_ms(), Some(WRITTEN_AT));
        assert!(!above.saved_reverted_past());

        // Exactly at it -- the boundary row is itself unwound.
        let mut at = saved_then_edited();
        at.truncate_to(5);
        assert_eq!(at.saved_seq(), None, "the saved row itself was unwound");
        assert!(at.saved_reverted_past());
        assert_eq!(at.saved_at_ms(), None, "no age for a boundary that is not there");

        // Below it.
        let mut below = saved_then_edited();
        below.truncate_to(3);
        assert_eq!(below.saved_seq(), None);
        assert!(below.saved_reverted_past());
    }

    /// `Edit > Undo` pops the newest *height* row, which is not always the
    /// newest row -- so the invalidation asks that row's own `seq`. Here the
    /// mark is above the height row being popped, and the boundary survives;
    /// mark below it and it does not.
    #[test]
    fn a_plain_undo_invalidates_only_when_it_reaches_the_boundary() {
        // `ledger()`'s newest height row is seq 5 (Sculpt commit).
        let mut kept = HistoryLedger::new();
        kept.record("height", "Carve fjords", "", EntryKind::HeightSnapshot);
        kept.mark_saved(Some(WRITTEN_AT));
        kept.record("height", "Sculpt commit", "", EntryKind::HeightSnapshot);
        kept.pop_newest_height();
        assert_eq!(kept.saved_seq(), Some(1), "the undone row was above the rule");
        assert!(!kept.saved_reverted_past());

        let mut lost = ledger();
        lost.mark_saved(Some(WRITTEN_AT));
        lost.pop_newest_height();
        assert_eq!(lost.saved_seq(), None, "seq 5 was the rule and it was just undone");
        assert!(lost.saved_reverted_past());
    }

    #[test]
    fn clearing_the_ledger_drops_the_boundary_without_blaming_a_revert() {
        let mut l = ledger();
        l.mark_saved(Some(WRITTEN_AT));
        l.clear();
        assert_eq!(l.saved_seq(), None);
        assert_eq!(l.saved_at_ms(), None);
        assert!(!l.saved_reverted_past());
    }

    /// `MAX_LEDGER` eviction forgets old rows' *names*; it does not unwind
    /// them, and they are still in the file. The boundary must not move.
    #[test]
    fn eviction_does_not_disturb_the_boundary() {
        let mut l = HistoryLedger::new();
        l.record("world", "Generate world", "", EntryKind::Floor);
        l.mark_saved(Some(WRITTEN_AT));
        for i in 0..(MAX_LEDGER + 20) {
            l.record("civ", format!("op{i}"), "", EntryKind::Recorded("r"));
        }
        assert_eq!(l.len(), MAX_LEDGER);
        assert_eq!(l.saved_seq(), Some(1), "seq 1 is evicted from the list, not from the file");
        assert_eq!(l.saved_at_ms(), Some(WRITTEN_AT));
        assert!(!l.saved_reverted_past());
    }

    /// [`file_written_at_ms`] over a file this test just wrote, and over one
    /// that is not there. The window is deliberately wide -- the assertion is
    /// that a real epoch-milliseconds figure comes back, not a timing.
    #[test]
    fn file_written_at_ms_reads_a_real_mtime_and_refuses_a_missing_path() {
        let dir = std::env::temp_dir().join("cartalith_undo_mtime_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("probe.zip");
        std::fs::write(&path, b"not really a zip").unwrap();

        let at = file_written_at_ms(path.to_str().unwrap()).expect("a file has an mtime");
        let now = HistoryLedger::now_ms_opt().expect("this platform has a clock");
        // 2001-09-09T01:46:40Z: any plausible mtime is far above it, and a
        // seconds-instead-of-milliseconds bug lands far below it.
        assert!(at > 1_000_000_000_000, "epoch milliseconds, not seconds: {at}");
        assert!(at <= now + 60_000, "an mtime cannot be a minute in the future: {at} vs {now}");
        assert!(now - at.min(now) < 600_000, "written seconds ago, not hours: {at} vs {now}");

        std::fs::remove_file(&path).unwrap();
        assert_eq!(file_written_at_ms(path.to_str().unwrap()), None);
        assert_eq!(file_written_at_ms(""), None);
    }

    #[test]
    fn seq_never_repeats_across_a_floor() {
        let mut l = HistoryLedger::new();
        let a = l.record("height", "a", "", EntryKind::HeightSnapshot);
        let b = l.record("world", "Generate", "", EntryKind::Floor);
        let c = l.record("height", "c", "", EntryKind::HeightSnapshot);
        assert!(a < b && b < c);
    }
}
