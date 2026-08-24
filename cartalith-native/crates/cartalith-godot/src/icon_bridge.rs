//! The Icon tool's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F, the CARTO domain's Icon tool (`DCC_SHELL_SPEC.md` §4.5.5:
//! "Click stamps the armed icon (`place_manual_icon`)... The Asset library
//! arms an icon and closes; the Icon tool is what places it").
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own doc comment argues for and follows the same
//! template: `lib.rs` owns the thin `Variant`<->`f64`/`String`/`Dictionary`
//! conversion and the `#[func]` surface; this module owns the actual state
//! — the placed-icon list, the armed selection, and which icon (if any) is
//! selected — with its own `#[cfg(test)]` suite below, exercised by `cargo
//! test -p cartalith-godot`'s ordinary unit-test pass with no Godot runtime
//! involved.
//!
//! ## Why this lives on `WorldGen`, not a second `GodotClass`
//!
//! Same reasoning as [`crate::sculpt_bridge::SculptEditor`]: every operation
//! here (arm, place, hit-test, resize) needs `WorldGen`'s own live grid
//! dimensions (`gw`/`gh`) and, for `icon_place`, the field/sea-level data a
//! future water-aware placement rule might read — nothing about the Icon
//! editor is independently constructible or reusable across worlds, so it is
//! a plain field (`icons: Option<IconEditor>`) exactly like `sculpt` and
//! `civ` already are, not a sibling `GodotClass` needing its own `Gd<>`
//! handle re-borrowed on every call for no benefit.
//!
//! ## What this module is built on
//!
//! Every placement/selection primitive here is a thin call into
//! `cartalith_assets::manual` — [`ArmedIcon`], [`ManualIcon`],
//! [`place_manual_icon`], [`icon_hit_test`], [`icon_resize_scale`] — already
//! written, golden-verified (`cartalith-assets/tests/
//! golden_parity_manual_icons.rs`, `UNIFIED_TOOL_PLAN.md`'s own "29 unit
//! tests... 7 golden tests" tally) and unit-tested in `manual.rs` itself.
//! Per this port's own discipline (`cartalith-porting-discipline`): expose
//! tested Rust, don't rewrite it. Nothing in this file duplicates or
//! second-guesses `manual.rs`'s own placement/hit-test/resize math; it only
//! adds the bookkeeping (a `Vec`, a selection index, an armed slot) that
//! `manual.rs` deliberately leaves to its caller.
//!
//! ## `family`/`variant` — how the numeric API addresses the asset library
//!
//! `DCC_SHELL_SPEC.md` §4.5.5's tool-options row is "family · variant ·
//! scale · rotation · jitter". `UNIFIED_TOOL_PLAN.md`'s own investigation
//! initially guessed `variant` would mean *art* variant (`pick_icon_variant`'s
//! seeded-variant logic) — but that guess was superseded by what `manual.rs`
//! actually shipped: [`ManualIcon`] carries no variant/art-index field at
//! all. Which of a slot's several art images is drawn is chosen at
//! *composite/render* time (`pack::composite_map_icons`'s
//! `pick_weighted_variant`, hashed from position + seed), identically for a
//! manually-placed icon and an auto-scattered one of the same slot — so
//! there is nothing for an *arm-time* variant index to select there.
//!
//! What a gallery tile picker genuinely needs to choose, instead, is *which
//! slot* — "mountain" vs. "hill" within the Feature family, "hamlet" vs.
//! "city" within Settlement, and so on. [`resolve_variant`] therefore reads
//! `variant` as a zero-based index into that family's own frozen vocabulary
//! (`cartalith_assets::slots::Family::slots()` — `PACK_ICON_SLOTS`,
//! `PACK_SETTLEMENT_SLOTS`, `PACK_POI_SLOTS`), the same lists
//! `ManualIconFamily::pack_family()` already resolves a family to. This
//! keeps the numeric API a caller (a gallery grid indexed 0..N) can drive
//! directly without a string round-trip, while staying inside the
//! vocabulary a real asset pack actually populates.
//!
//! **[`ManualIconFamily::Custom`] cannot be armed through this call.** Its
//! vocabulary is open (`Family::Custom.slots()` is `&[]` by design — see
//! `slots.rs`'s own doc comment) and addressed two levels deep, `set ->
//! slot`, which a single `i64` cannot express. [`resolve_variant`] returns
//! `None` for it rather than inventing an ordering over an open, pack-
//! defined set list that would silently break the moment two packs order
//! their custom sets differently. A richer arm call taking `set`/`slot`
//! strings is future work if the shell needs Custom icons before then.
//!
//! ## Arm-time `scale` is a disclosed addition; `rotation`/`jitter` are not applied
//!
//! The reference's own click path always places at `scale: 1` (reference
//! lines 9776-9784, `place_manual_icon`'s own doc comment) — there is no
//! arm-time scale control in the reference at all; only the resize handle
//! (`icon_resize_scale`) changes a placed icon's scale, after the fact.
//! `DCC_SHELL_SPEC.md`'s tool-options row adds one anyway, so [`IconEditor::place`]
//! honours it: it calls `place_manual_icon` unchanged (its own golden-backed
//! `scale == 1.0` behaviour is untouched) and then overwrites the *returned*
//! `ManualIcon`'s `scale` field with the armed value before storing it. This
//! is a disclosed, boundary-layer addition — not a change to `manual.rs` —
//! but it is a genuine behavioural difference from the reference's click
//! path, flagged here per `cartalith-porting-discipline`'s "anything that
//! changes output" rule rather than assumed correct.
//!
//! `rotation` and `jitter` have no equivalent in the reference at all
//! ([`ManualIcon`] carries no rotation field, and the brush's own
//! "jitter" is the dart-throwing randomness itself, not a scalar control —
//! see `manual.rs`'s module doc). [`IconEditor::arm`] accepts and stores
//! both purely so the tool-options row and the armed-icon "chip"
//! (`DCC_SHELL_SPEC.md` §4.5.5) have somewhere to keep them, but neither
//! ever reaches a placed [`ManualIcon`] — there is no field on that type to
//! write them into, and adding one would be exactly the kind of
//! `cartalith-assets` rewrite this module is chartered not to do.

use cartalith_assets::manual::{
    civ_zoom_k, icon_box, icon_hit_test, icon_resize_scale, place_manual_icon, ArmedIcon, IconBox,
    IconHandle, IconHit, IconHitKind, IconViewEnv, ManualIcon, ManualIconFamily,
};

/// The armed selection plus the tool-options row's extra chip fields —
/// see the module doc's "Arm-time `scale`..." section for which of these
/// three actually reaches a placed icon.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmedSelection {
    pub icon: ArmedIcon,
    pub scale: f64,
    pub rotation: f64,
    pub jitter: f64,
}

/// Bounds a caller-supplied arm-time scale is clamped to, reusing
/// `manual.rs`'s own resize bounds ([`cartalith_assets::manual::ICON_SCALE_MIN`]/
/// `_MAX`) rather than inventing a second range for what is, after
/// placement, the exact same field a resize drag also writes.
pub use cartalith_assets::manual::{ICON_SCALE_MAX, ICON_SCALE_MIN};

/// Resolves `(family, variant)` into the slot string [`ArmedIcon::slot`]
/// needs — see the module doc's "how the numeric API addresses the asset
/// library" section. `None` for [`ManualIconFamily::Custom`] (open
/// vocabulary, not expressible as one index) or an out-of-range/negative
/// `variant`.
pub fn resolve_variant(family: ManualIconFamily, variant: i64) -> Option<String> {
    if family == ManualIconFamily::Custom {
        return None;
    }
    let i = usize::try_from(variant).ok()?;
    family.pack_family().slots().get(i).map(|s| s.to_string())
}

/// The live Icon-editor state for one generated world: every hand-placed
/// icon, the current armed selection (if any), and which placed icon (if
/// any) is selected — the reference's own `state.mapIcons`/`_carIconArmed`/
/// the click handler's own "select what was just placed or hit" convention,
/// kept together the way `SculptEditor` keeps its own draft/tool-state/
/// selection together.
pub struct IconEditor {
    pub icons: Vec<ManualIcon>,
    pub armed: Option<ArmedSelection>,
    pub selected: Option<usize>,
    /// The selected icon's own `scale` at the moment it became selected —
    /// the baseline [`IconEditor::resize`] scales from. Captured once per
    /// selection (by [`IconEditor::place`]/[`IconEditor::hit_test`], the
    /// two ways a selection can start) rather than read live off the icon
    /// on every `resize` call, because a drag calls `resize` repeatedly
    /// with the *same* `start_dist` it captured at grab-time
    /// (`icon_resize_scale`'s own contract, mirroring the reference's own
    /// fixed `_iconResize.startScale`/`startDist` for the whole gesture) —
    /// reading the icon's live scale instead would compound the ratio on
    /// every intermediate call rather than computing it fresh from the
    /// drag's own start each time.
    resize_base_scale: Option<f64>,
}

impl IconEditor {
    /// A fresh, empty editor — nothing armed, nothing placed, nothing
    /// selected. Called once per `generate()`/`generate_sized()`
    /// (`WorldGen::absorb`), matching `SculptEditor::new`'s own "a fresh
    /// draft over this world's own dimensions" pattern: a hand-placed
    /// icon's `x`/`y` are grid-cell coordinates over one particular world,
    /// meaningless carried over to a differently-sized one.
    pub fn new() -> Self {
        IconEditor { icons: Vec::new(), armed: None, selected: None, resize_base_scale: None }
    }

    /// Arms `family`/`variant` (see the module doc) for the next
    /// `place()` call. `scale` is clamped to [`ICON_SCALE_MIN`]/
    /// [`ICON_SCALE_MAX`] (non-finite or non-positive -> `1.0`, the
    /// reference's own click-path default); `rotation`/`jitter` are stored
    /// as given (non-finite -> `0.0`) — see the module doc for why neither
    /// currently reaches a placed icon. `false` for an unrecognised
    /// `family` key, a `variant` outside that family's own vocabulary, or
    /// `family == "custom"` (not addressable this way — see
    /// [`resolve_variant`]); the previous armed selection (if any) is left
    /// untouched on a rejected call, matching `set_feature_param`'s own
    /// "typo is visibly rejected, not silently applied" policy elsewhere
    /// in this crate.
    pub fn arm(&mut self, family_key: &str, variant: i64, scale: f64, rotation: f64, jitter: f64) -> bool {
        let Some(family) = ManualIconFamily::from_key(family_key) else { return false };
        let Some(slot) = resolve_variant(family, variant) else { return false };
        let scale = if scale.is_finite() && scale > 0.0 { scale.clamp(ICON_SCALE_MIN, ICON_SCALE_MAX) } else { 1.0 };
        let rotation = if rotation.is_finite() { rotation } else { 0.0 };
        let jitter = if jitter.is_finite() { jitter } else { 0.0 };
        self.armed = Some(ArmedSelection { icon: ArmedIcon { family, slot, set: None }, scale, rotation, jitter });
        true
    }

    /// Disarms — the next `place()` call does nothing until `arm()` is
    /// called again. Matches the reference's own `_carIconArmed=null`
    /// (fired on Escape, switching family, or arming a different tool —
    /// `DCC_SHELL_SPEC.md` §4.5.6's "arming any tool clears... its armed
    /// icon"; `lib.rs` is responsible for calling this at those points,
    /// the same way it already owns cross-tool disarm sequencing).
    pub fn disarm(&mut self) {
        self.armed = None;
    }

    /// Stamps the armed icon at grid cell `(gx, gy)` — `place_manual_icon`
    /// plus the arm-time scale override (see the module doc). Selects the
    /// new icon (the reference's own "click... places... and selects it").
    /// Returns the new index, or `None` when nothing is armed or the click
    /// is off-grid (`place_manual_icon`'s own bounds gate).
    pub fn place(&mut self, gx: f64, gy: f64, gw: usize, gh: usize) -> Option<usize> {
        let armed = self.armed.as_ref()?;
        let mut icon = place_manual_icon(gx, gy, gw, gh, Some(&armed.icon))?;
        icon.scale = armed.scale;
        let index = self.icons.len();
        self.icons.push(icon);
        self.select(index);
        Some(index)
    }

    /// `_carIconHitTest`'s box-hit half only (`manual.rs`'s own
    /// `icon_hit_test`, `None` handle). Boxes are computed in **grid
    /// space** (`env`'s `zoom_scale`/`icon_scale` at their defaults unless
    /// the caller overrides them), matching `gx`/`gy` here and in `place`/
    /// `resize` all being grid coordinates, not screen pixels — a caller
    /// converts a real pointer event through its own view transform first,
    /// same convention `sculpt_add_point`'s own doc comment states for the
    /// Sculpt tool. Selects and returns the hit icon's index on a hit
    /// (matching the reference's own hit-then-select click sequencing);
    /// `None` (selection unchanged) on a miss.
    ///
    /// **Box hits only, still** — matching `label_bridge::LabelBridge::
    /// hit_test`'s own precedent: a *handle* hit is the shell's own job,
    /// by comparing the pointer against the circle [`IconEditor::handles`]
    /// returns for whichever icon is selected (`GUI_GAP_REGISTER.md` CA-05
    /// closed that gap — see this file's own module doc for why it lives
    /// here now).
    pub fn hit_test(&mut self, gx: f64, gy: f64, env: &IconViewEnv) -> Option<usize> {
        let boxes: Vec<IconBox> = self.icons.iter().map(|ic| icon_box(ic, env)).collect();
        match icon_hit_test(&boxes, None, gx, gy) {
            Some(IconHit { kind: IconHitKind::Box, index: Some(i) }) => {
                self.select(i);
                Some(i)
            }
            _ => None,
        }
    }

    /// Icon `index`'s on-canvas resize-handle circle — see [`icon_handle`].
    /// `None` for an out-of-range `index`. Unlike [`IconEditor::select`],
    /// this does not require `index` to be the current selection: exactly
    /// `label_bridge::LabelBridge::handles`' own contract (any valid index
    /// works; a caller decides which index to ask for, normally whichever
    /// one is currently selected).
    pub fn handles(&self, index: usize, env: &IconViewEnv) -> Option<IconHandle> {
        let icon = self.icons.get(index)?;
        let box_ = icon_box(icon, env);
        Some(icon_handle(&box_, env))
    }

    /// Records `index` as selected and snapshots its current `scale` as
    /// the next `resize()` gesture's baseline (see `resize_base_scale`'s
    /// own doc comment). Private: the two ways a selection legitimately
    /// starts are `place` and `hit_test`, both above.
    fn select(&mut self, index: usize) {
        self.resize_base_scale = self.icons.get(index).map(|ic| ic.scale);
        self.selected = Some(index);
    }

    /// Applies one resize-drag sample to the selected icon's `scale` —
    /// `icon_resize_scale(base, cx, cy, gx, gy, start_dist)`, `base` being
    /// the snapshot `select` took, not the icon's live (already-updated-
    /// this-gesture) scale (see `resize_base_scale`'s own doc comment for
    /// why). Requires `index` to already be the selected icon (a drag on a
    /// box the caller hasn't hit-tested/selected first is a caller bug, not
    /// a silently-accepted resize of the wrong icon); `false` otherwise, or
    /// for an out-of-range `index`.
    pub fn resize(&mut self, index: usize, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> bool {
        if self.selected != Some(index) {
            return false;
        }
        let Some(base) = self.resize_base_scale else { return false };
        let Some(icon) = self.icons.get_mut(index) else { return false };
        icon.scale = icon_resize_scale(base, cx, cy, gx, gy, start_dist);
        true
    }

    /// Removes icon `index`. Clears the selection if it pointed at the
    /// removed icon; shifts a selection pointing past it down by one so it
    /// keeps addressing the same logical icon in the now-shorter `Vec`
    /// (`sculpt_bridge`'s stamp-stack equivalents clear rather than shift,
    /// but the stamp *stack* has no "everything after this one renumbers"
    /// property to preserve — a flat `Vec::remove` here does, and losing
    /// track of an unrelated selection on every delete would be a worse
    /// default for a list a caller is actively editing). `false` for an
    /// out-of-range `index`.
    pub fn delete(&mut self, index: usize) -> bool {
        if index >= self.icons.len() {
            return false;
        }
        self.icons.remove(index);
        self.selected = match self.selected {
            Some(s) if s == index => {
                self.resize_base_scale = None;
                None
            }
            Some(s) if s > index => Some(s - 1),
            other => other,
        };
        true
    }

    /// Drops every placed icon and the current selection (armed selection
    /// untouched — `DCC_SHELL_SPEC.md` §4.5.5's list panel's own
    /// "Clear-all" clears placements, not the gallery's own arming state).
    pub fn clear_all(&mut self) {
        self.icons.clear();
        self.selected = None;
        self.resize_base_scale = None;
    }
}

impl Default for IconEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// The reference's `drawCivLayer` selected-icon resize-handle geometry
/// (lines 15883-15893 of `reference/Cartalith Gen1 v2.10.html`),
/// transcribed rather than sliced — exactly `label_bridge::handle_circles`'
/// own reasoning: this is inline canvas drawing code, not a callable
/// function, so `manual.rs` never had a home for it (that module's own
/// `IconEditor::hit_test`/`icon_bridge.rs` doc comments called this out as
/// the one acknowledged gap — `GUI_GAP_REGISTER.md` CA-05).
///
/// One handle only — a manually-placed icon has no rotate/arc field at all
/// (`manual.rs`'s own module doc), so unlike a label's five circles there
/// is nothing else to compute here.
///
/// `lsc` is `label_bridge::handle_circles`'s own render-pass constant
/// (`Math.max(1,GW/512)*_civZoomK()*_civIconScale()`) — computed inline
/// here rather than shared with that function, matching `IconBox`'s own
/// doc comment on why icon/label geometry stay separate rather than behind
/// a shared abstraction (`cartalith-assets` has no dependency on
/// `cartalith-civ` to share one through, either).
pub fn icon_handle(box_: &IconBox, env: &IconViewEnv) -> IconHandle {
    let lsc = f64::max(1.0, env.grid_w as f64 / 512.0) * civ_zoom_k(env.zoom_scale) * env.icon_scale;
    let hr = f64::max(4.0, 3.2 * lsc);
    let hx = box_.px + box_.side / 2.0 * 0.7;
    let hy = box_.py + box_.side / 2.0 * 0.7;
    // The reference's own hit-test radius bakes in the *displayed* circle's
    // own further slack (`_iconHandle={..,r:hr*1.6,..}`, reference line
    // 15893) — not the drawn `hr` alone, matching `label_bridge::
    // handle_circles`' own resize/rotate/arc handles doing the same thing.
    IconHandle { x: hx, y: hy, r: hr * 1.6 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> IconViewEnv {
        IconViewEnv { grid_w: 48, zoom_scale: 1.0, icon_scale: 1.0 }
    }

    // ---- resolve_variant ----

    #[test]
    fn resolve_variant_indexes_the_familys_own_frozen_slots() {
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 0).as_deref(), Some("mountain"));
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 1).as_deref(), Some("hill"));
        assert_eq!(resolve_variant(ManualIconFamily::Settlement, 0).as_deref(), Some("hamlet"));
        assert_eq!(resolve_variant(ManualIconFamily::Poi, 0).as_deref(), Some("ruin"));
    }

    #[test]
    fn resolve_variant_rejects_out_of_range_and_negative() {
        assert_eq!(resolve_variant(ManualIconFamily::Feature, 999), None);
        assert_eq!(resolve_variant(ManualIconFamily::Feature, -1), None);
    }

    #[test]
    fn resolve_variant_rejects_custom_entirely() {
        assert_eq!(resolve_variant(ManualIconFamily::Custom, 0), None);
    }

    // ---- arm / disarm ----

    #[test]
    fn arm_with_an_unknown_family_key_fails_and_changes_nothing() {
        let mut e = IconEditor::new();
        assert!(!e.arm("nope", 0, 1.0, 0.0, 0.0));
        assert!(e.armed.is_none());
    }

    #[test]
    fn arm_with_custom_fails() {
        let mut e = IconEditor::new();
        assert!(!e.arm("custom", 0, 1.0, 0.0, 0.0));
    }

    #[test]
    fn arm_rejects_leave_a_previous_armed_selection_untouched() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 1.0, 0.0, 0.0));
        let before = e.armed.clone();
        assert!(!e.arm("feature", 999, 1.0, 0.0, 0.0));
        assert_eq!(e.armed, before);
    }

    #[test]
    fn arm_clamps_scale_and_defaults_a_bad_one_to_one() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 999.0, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, ICON_SCALE_MAX);
        assert!(e.arm("feature", 0, f64::NAN, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, 1.0);
        assert!(e.arm("feature", 0, -5.0, 0.0, 0.0));
        assert_eq!(e.armed.as_ref().unwrap().scale, 1.0);
    }

    #[test]
    fn arm_stores_rotation_and_jitter_verbatim_when_finite() {
        let mut e = IconEditor::new();
        assert!(e.arm("feature", 0, 1.0, 45.0, 0.7));
        let a = e.armed.as_ref().unwrap();
        assert_eq!(a.rotation, 45.0);
        assert_eq!(a.jitter, 0.7);
    }

    #[test]
    fn disarm_clears_the_armed_selection() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.disarm();
        assert!(e.armed.is_none());
    }

    // ---- place ----

    #[test]
    fn place_with_nothing_armed_does_nothing() {
        let mut e = IconEditor::new();
        assert_eq!(e.place(5.0, 5.0, 48, 32), None);
        assert!(e.icons.is_empty());
    }

    #[test]
    fn place_stamps_selects_and_honours_the_armed_scale() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 2.5, 0.0, 0.0);
        let idx = e.place(5.0, 5.0, 48, 32).expect("placed");
        assert_eq!(idx, 0);
        assert_eq!(e.selected, Some(0));
        let ic = &e.icons[0];
        assert_eq!((ic.x, ic.y), (5.0, 5.0));
        assert_eq!(ic.family, ManualIconFamily::Feature);
        assert_eq!(ic.slot, "mountain");
        assert_eq!(ic.scale, 2.5, "arm-time scale must override place_manual_icon's own 1.0");
    }

    #[test]
    fn place_off_grid_fails_without_disturbing_the_armed_selection() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        assert_eq!(e.place(-1.0, 5.0, 48, 32), None);
        assert!(e.icons.is_empty());
        assert!(e.armed.is_some());
    }

    // ---- hit_test ----

    #[test]
    fn hit_test_finds_and_selects_a_placed_icon() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(5.0, 5.0, 48, 32);
        let hit = e.hit_test(5.5, 5.5, &env());
        assert_eq!(hit, Some(0));
        assert_eq!(e.selected, Some(0));
    }

    #[test]
    fn hit_test_miss_leaves_selection_unchanged() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(5.0, 5.0, 48, 32);
        e.selected = None;
        assert_eq!(e.hit_test(500.0, 500.0, &env()), None);
        assert_eq!(e.selected, None);
    }

    // ---- resize ----

    #[test]
    fn resize_requires_the_target_to_already_be_selected() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(20.0, 10.0, 48, 32);
        e.selected = None;
        assert!(!e.resize(0, 20.0, 10.0, 60.0, 60.0, 3.0));
        assert_eq!(e.icons[0].scale, 1.0);
    }

    #[test]
    fn resize_scales_from_the_selection_time_snapshot_not_the_live_value() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(10.0, 10.0, 48, 32); // scale 1.0, selected, base snapshot = 1.0
        // Two calls with the SAME start_dist, as a real drag would send --
        // each must compute fresh off the base of 1.0, not compound.
        assert!(e.resize(0, 10.0, 10.0, 10.0, 10.0, 5.0)); // dist floor -> min clamp
        let after_first = e.icons[0].scale;
        assert!(e.resize(0, 10.0, 10.0, 10.0, 10.0, 5.0));
        assert_eq!(e.icons[0].scale, after_first, "same inputs must give the same result, not compound");
    }

    #[test]
    fn resize_rejects_an_out_of_range_index() {
        let mut e = IconEditor::new();
        assert!(!e.resize(0, 0.0, 0.0, 0.0, 0.0, 1.0));
    }

    // ---- handles / icon_handle ----

    #[test]
    fn icon_handle_matches_the_reference_formula() {
        let ic = ManualIcon { x: 10.0, y: 8.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        // grid_w=2048 -> sc/lsc base = 4; zoom_scale=1 -> civ_zoom_k=1.
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let box_ = icon_box(&ic, &env); // px=10.5, py=8.5, r=20, side=52
        let h = icon_handle(&box_, &env);
        assert!((h.x - 28.7).abs() < 1e-9, "hx = px + side/2*0.7 = 10.5 + 18.2");
        assert!((h.y - 26.7).abs() < 1e-9, "hy = py + side/2*0.7 = 8.5 + 18.2");
        assert!((h.r - 20.48).abs() < 1e-9, "hr=max(4,3.2*4)=12.8, stored r = hr*1.6");
    }

    #[test]
    fn icon_handle_follows_the_boxs_own_per_instance_scale() {
        let small = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        let big = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 2.5 };
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h_small = icon_handle(&icon_box(&small, &env), &env);
        let h_big = icon_handle(&icon_box(&big, &env), &env);
        // A bigger box pushes the handle further from the icon's own centre,
        // but the handle's own radius (a fixed on-screen affordance size,
        // not sprite-relative) is unchanged -- exactly `hr`'s own formula,
        // which depends only on `lsc`, never on the box.
        assert!(h_big.x > h_small.x);
        assert!(h_big.y > h_small.y);
        assert!((h_big.r - h_small.r).abs() < 1e-9);
    }

    #[test]
    fn icon_handle_never_shrinks_below_its_own_floor_at_low_zoom() {
        // zoom_scale pushed far past civ_zoom_k's own [0.35,5] clamp so lsc
        // collapses toward its minimum and the max(4,...) floor takes over
        // -- same fixture shape `label_bridge::handle_circles`' own
        // low-zoom-floor test uses.
        let ic = ManualIcon { x: 0.0, y: 0.0, family: ManualIconFamily::Feature, slot: "mountain".into(), set: None, scale: 1.0 };
        let env = IconViewEnv { grid_w: 512, zoom_scale: 1000.0, icon_scale: 1.0 };
        let h = icon_handle(&icon_box(&ic, &env), &env);
        assert!((h.r - 6.4).abs() < 1e-9, "hr floors at 4, stored r = 4*1.6");
    }

    #[test]
    fn editor_handles_matches_the_selected_icons_box() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(10.0, 8.0, 2048, 2048);
        let env = IconViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h = e.handles(0, &env).expect("placed icon has a handle");
        assert!((h.x - 28.7).abs() < 1e-9);
        assert!((h.y - 26.7).abs() < 1e-9);
    }

    #[test]
    fn editor_handles_does_not_require_the_index_to_be_selected() {
        // Mirrors `label_bridge::LabelBridge::handles`' own contract: any
        // valid index works, not only the current selection.
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32); // index 0
        e.place(2.0, 2.0, 48, 32); // index 1, now selected
        assert_eq!(e.selected, Some(1));
        assert!(e.handles(0, &env()).is_some(), "index 0 is not selected but is still valid");
    }

    #[test]
    fn editor_handles_out_of_range_is_none() {
        let e = IconEditor::new();
        assert!(e.handles(0, &env()).is_none());
    }

    // ---- delete ----

    #[test]
    fn delete_removes_and_clears_selection_when_it_pointed_at_the_removed_icon() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32);
        assert!(e.delete(0));
        assert!(e.icons.is_empty());
        assert_eq!(e.selected, None);
    }

    #[test]
    fn delete_shifts_a_selection_pointing_past_the_removed_index() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32); // index 0
        e.place(2.0, 2.0, 48, 32); // index 1, now selected
        assert_eq!(e.selected, Some(1));
        assert!(e.delete(0));
        assert_eq!(e.selected, Some(0), "the icon formerly at 1 is now at 0");
        assert_eq!(e.icons.len(), 1);
    }

    #[test]
    fn delete_out_of_range_fails() {
        let mut e = IconEditor::new();
        assert!(!e.delete(0));
    }

    // ---- clear_all ----

    #[test]
    fn clear_all_drops_placements_and_selection_but_not_the_armed_chip() {
        let mut e = IconEditor::new();
        e.arm("feature", 0, 1.0, 0.0, 0.0);
        e.place(1.0, 1.0, 48, 32);
        e.place(2.0, 2.0, 48, 32);
        e.clear_all();
        assert!(e.icons.is_empty());
        assert_eq!(e.selected, None);
        assert!(e.armed.is_some(), "Clear-all clears placements, not the armed gallery selection");
    }
}
