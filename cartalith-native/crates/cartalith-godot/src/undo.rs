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
//! - **Undo does not re-run flow/climate.** The reference's `undoLast` calls
//!   `computeFlow(true); refreshClimate(); renderNow()`. This port's whole
//!   commit path already defers those (`UNIFIED_TOOL_PLAN.md` milestone A's
//!   deferred staleness; `sculpt_commit`'s own doc comment says the same),
//!   so undo is consistent with the commit it reverses rather than with the
//!   reference's inline recompute.
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
