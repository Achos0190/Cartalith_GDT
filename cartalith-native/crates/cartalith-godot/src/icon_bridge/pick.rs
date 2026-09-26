//! The map context request's read-only icon pick (`MAP_CONTEXT_SCOPE.md`
//! CM-1) — [`super::IconEditor::pick_all`] across the gdext boundary.
//!
//! A child module for `brush.rs`'s and `generate.rs`'s own reason, and so
//! `lib.rs` is not touched by this milestone.

use godot::prelude::*;

use crate::WorldGen;

#[godot_api(secondary)]
impl WorldGen {
    /// Every placed icon whose box contains `(gx, gy)`, topmost first,
    /// **without selecting anything** — the multi-hit pick `map_overlay.gd`'s
    /// `context_requested` carries, which `icon_hit_test` cannot be (it
    /// selects its hit and answers one). The box is `icon_hit_test_mode`'s
    /// exactly, `zoom_scale: 1.0` placeholder included, deliberately: the two
    /// must agree on what is under the pointer, so the first entry is what a
    /// plain click would select. Empty on a miss or before any `generate()`.
    #[func]
    fn icon_pick_all(&self, gx: f64, gy: f64) -> PackedInt64Array {
        let grid_w = self.gw as usize;
        let Some(icons) = self.icons.as_ref() else { return PackedInt64Array::new() };
        let env = cartalith_assets::manual::IconViewEnv { grid_w, zoom_scale: 1.0, icon_scale: 1.0 };
        icons.pick_all(gx, gy, &env).into_iter().map(|i| i as i64).collect()
    }
}
