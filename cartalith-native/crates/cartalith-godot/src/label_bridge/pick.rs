//! The map context request's read-only label pick (`MAP_CONTEXT_SCOPE.md`
//! CM-1) — [`super::LabelBridge::pick_all`] across the gdext boundary.
//!
//! A child module for `generate.rs`'s own reason: `label_bridge.rs` stays free
//! of any `godot` dependency, and `lib.rs` is not this milestone's file.

use godot::prelude::*;

use crate::WorldGen;

#[godot_api(secondary)]
impl WorldGen {
    /// Every label whose box contains `(gx, gy)`, topmost first, **without
    /// selecting anything** — the multi-hit pick `map_overlay.gd`'s
    /// `context_requested` carries. `label_hit_test` cannot be that pick: it
    /// selects its hit and answers only the topmost one. Same `px_per_cell`
    /// box model (pass `ViewportHost::label_px_per_cell()`), so the first
    /// entry is what a plain click would select. Empty on a miss or before
    /// any `generate()` call.
    #[func]
    fn label_pick_all(&self, gx: f64, gy: f64, px_per_cell: f64) -> PackedInt64Array {
        let Some(labels) = self.labels.as_ref() else { return PackedInt64Array::new() };
        labels.pick_all(gx, gy, px_per_cell).into_iter().map(|i| i as i64).collect()
    }
}
