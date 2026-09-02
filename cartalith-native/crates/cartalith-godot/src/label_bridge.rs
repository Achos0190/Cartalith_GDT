//! The Label tool's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F, the CARTO domain's Label tool (`DCC_SHELL_SPEC.md` §4.5.5:
//! "Click an empty spot creates a label; click an existing one edits it in
//! place").
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`/`icon_bridge.rs` both argue for and follow: `lib.rs`
//! owns the thin `Variant`<->Rust conversion and the `#[func]` surface; this
//! module owns the actual state (the placed-label list and the edit
//! session), with its own `#[cfg(test)]` suite below, exercised by `cargo
//! test -p cartalith-godot`'s ordinary unit-test pass with no Godot runtime
//! involved.
//!
//! ## Why this lives on `WorldGen`, not a second `GodotClass`
//!
//! The same reasoning [`crate::sculpt_bridge::SculptEditor`] and
//! `crate::icon_bridge::IconEditor` already give: nothing about a label list
//! is independently constructible or reusable across worlds (grid
//! coordinates from one generation are meaningless carried over to
//! another), so it is a plain field (`labels: Option<LabelBridge>`) reset
//! every `absorb()`, exactly like `sculpt`/`icons`/`civ_tools`.
//!
//! ## What this module is built on, and what it adds
//!
//! `cartalith_civ::labels` (886 lines, `UNIFIED_TOOL_PLAN.md` milestone E)
//! already ports `MapLabel`, [`LabelEditSession`] (the two-commit-semantics
//! select/confirm/cancel state machine — position commits immediately, the
//! seven style fields are snapshot-and-revertible), the arc text layout
//! (`arc_label_layout`), the box/hit-test geometry (`label_box`,
//! `label_hit_test`) and the three drag-math formulas (resize/rotate/arc).
//! Per this port's own discipline (`cartalith-porting-discipline`): expose
//! tested Rust, don't rewrite it. This module adds only the bookkeeping
//! that file deliberately leaves to its caller (a `Vec<MapLabel>` — no
//! `PassBuffer`, matching milestone D/E's own finding that placing/editing
//! one label is a discrete action with its own confirm/cancel, not a brush
//! stroke) plus **one genuinely new piece of ported geometry**:
//! [`handle_circles`], the five on-canvas handle positions
//! (resize/rotate/arc/check/cross), transcribed below from the reference's
//! `drawCivLayer` (lines 15810-15875 of `reference/Cartalith Gen1
//! v2.10.html`). That code is inline canvas drawing, not a callable
//! function, so `labels.rs` never had a home for it — its own module doc is
//! explicit that it ports `drawArcLabel`, `_civLabelBox`,
//! `_civLabelHitTest` and the pointer-move handler's *drag-math* only
//! (lines 9686-9717); the handles' *rendered positions* (drawn a few
//! thousand lines later, in the render pass, not the input handler) were
//! still missing. This is exactly `STRANDED_TOOLS.md`'s "the verbs exist,
//! the noun cannot be created" gap for Label, closed here.
//!
//! ## Text measurement — still the one thing that can't live in Rust
//!
//! Per `labels.rs`'s own module doc: glyph advances are a property of the
//! loaded font, not the geometry, so real widths must come from a live
//! Godot `Font`. Three call shapes in this file react to that differently:
//!
//! - [`LabelBridge::hit_test`] and [`LabelBridge::handles`] take **no**
//!   width input at all — they pass `meas_w: 0.0` into `label_box`
//!   internally, a disclosed placeholder (the box narrows to
//!   `max(0, fsz*1.3)*1.25`, i.e. a font-height square centred on the
//!   label's origin — hit-testable, just not exactly the rendered
//!   footprint's *width*). This mirrors `icon_bridge::IconEditor::
//!   hit_test`'s own honestly-scoped "box hits only" precedent one step
//!   further: real UI wiring is on hold project-wide (root `CLAUDE.md`,
//!   2026-08-18) and no caller in this codebase yet has a live font to
//!   measure with, so inventing a fake width formula here would dress up a
//!   guess as data. `0.0` is visibly a placeholder; a plausible-looking
//!   guess would not be.
//! - [`LabelBridge::glyph_layout`] cannot take this shortcut: arc placement
//!   is fundamentally *about* per-glyph spacing (`labels.rs`'s own note
//!   that a port summing char widths instead of reading `total_w`
//!   separately drifts on any kerned string), so a `0.0` placeholder here
//!   would collapse every glyph onto the label's origin, not just narrow a
//!   box. This is the one call in the whole bridge that requires real
//!   measured widths as inputs — see `WorldGen::label_glyph_layout`'s own
//!   doc comment for the exact contract a caller must satisfy.

use cartalith_civ::labels::{
    arc_label_layout, civ_zoom_k, label_box, label_cull_rect, label_font_size, ArcLayout, GeneratedLabels,
    HandleCircle, LabelBox, LabelCullMetrics, LabelEditSession, LabelGenSettings, LabelHandles, LabelRect,
    LabelSizeMode, LabelTypography, LabelViewEnv, MapLabel, LABEL_SIZE_MAX, LABEL_SIZE_MIN,
    LABEL_TYPOGRAPHY_DEFAULTS,
};

/// The generated labelling pass's own `#[godot_api(secondary)]` surface.
///
/// **A submodule rather than a block in this file, and rather than a new
/// top-level module.** This file's header promises it is "deliberately free of
/// any `godot` dependency", the same isolation `sculpt_bridge.rs`/
/// `icon_bridge.rs` argue for, and that promise is worth keeping: the state
/// below is exercised by an ordinary unit-test pass with no Godot runtime. A
/// sibling of `label_bridge.rs` would have to be registered in `lib.rs`, which
/// two other agents are editing on this date. A child module is neither: it is
/// a new file, `label_bridge/generate.rs`, declared here.
mod generate;

// Re-exported so `lib.rs` can build a drag loop (`label_resize_size`/
// `label_rotate_deg`/`label_arc_value` are pure per-call math, no session to
// hold on this side — the caller keeps `cx`/`cy`/`start_size`/`start_dist`/
// `grab_angle_deg` locally between pointer events, the same way it already
// must for the sibling `icon_resize` call).
pub use cartalith_civ::labels::{label_arc_value as arc_value, label_resize_size as resize_size, label_rotate_deg as rotate_deg};

/// What writing one style field did — the same three-way shape
/// `sculpt_bridge::Outcome`/`icon_bridge`'s own outcomes use, kept as this
/// module's own type for the same "importable standalone" reason
/// `sculpt_bridge.rs` documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Stored exactly as given.
    Applied,
    /// Stored, but clamped to the field's own range.
    Clamped,
    /// Not stored — non-finite value, or (for `size_mode`) an unrecognised key.
    Rejected,
}

/// The placed-label list plus the edit session — the reference's
/// `state.labels` and `_civSelectedLabel`/`_civLabelEditSnapshot`, together —
/// **plus** the generated pass's own output beside them.
///
/// ## Two lists, not one
///
/// `labels` is what the Label tool created and what `label_select`/
/// `label_set`/`label_delete` edit; [`generated`](Self::generated) is what
/// [`cartalith_civ::labels::generate_labels`] last produced from the world's
/// own named features. They are kept apart deliberately. Appending generated
/// labels to `labels` would make every index the edit session, the undo trail
/// and `place_search.gd` hold shift the next time the pass ran, and a
/// hand-edited label would be silently replaced by a regenerate. The renderer
/// sees one list — [`WorldGen::labels_render_list`] concatenates them — and
/// nothing else does.
pub struct LabelBridge {
    pub labels: Vec<MapLabel>,
    pub session: LabelEditSession,
    /// The last generated run, or `None` if the pass has never been run over
    /// this world. `None` and "ran and placed nothing" are different claims —
    /// [`cartalith_civ::landmark::LandmarkStore::last`]'s own distinction, for
    /// the same reason: the panel must read `--`, not `0`.
    pub generated: Option<GeneratedLabels>,
    /// The five class type specs in force, [`LABEL_TYPOGRAPHY_DEFAULTS`] until
    /// a caller overrides them.
    ///
    /// **Reset with the rest of this struct on every `generate()`**, because
    /// `WorldGen::absorb` replaces the whole `LabelBridge`. That is not a lost
    /// setting in practice and the shape is deliberate: the CARTO panel must
    /// call [`WorldGen::labels_generate`] after a world change anyway (the old
    /// world's continents and settlements no longer exist), and that call
    /// carries the panel's own dial values, so the table is restored by the one
    /// action that has to happen regardless. Holding it anywhere that survived
    /// `absorb` would mean a second source of truth for a design value.
    pub typography: [LabelTypography; 5],
    /// What the last [`WorldGen::labels_generate`] was asked for. Retained so
    /// a re-run without options repeats the last one rather than silently
    /// reverting to defaults.
    pub gen_settings: LabelGenSettings,
}

impl LabelBridge {
    /// A fresh, empty bridge over a newly generated world — called once per
    /// `generate()`/`generate_sized()` (`WorldGen::absorb`), matching
    /// `IconEditor::new`'s own "a hand-placed icon's x/y are grid-cell
    /// coordinates over one particular world, meaningless carried over to a
    /// differently-sized one" reasoning. No dimensions to remember here
    /// (unlike `SculptEditor`'s draft buffers): a label's own `x`/`y` are
    /// already absolute grid coordinates, and every geometry call below
    /// takes the current `grid_w` as a parameter rather than storing it.
    pub fn new() -> Self {
        LabelBridge {
            labels: Vec::new(),
            session: LabelEditSession::new(),
            generated: None,
            typography: LABEL_TYPOGRAPHY_DEFAULTS,
            // **Culling on, where `LabelGenSettings::default()` has it off.**
            // The workspace's standing rule for anything that changes what the
            // engine emits: off in the engine's own defaults, on at the shell's
            // boundary. Here the shell side is also the design's — the toggle
            // is drawn checked (`parts.js:387`) — and `_regenerate_labels()`
            // sends the panel's own state on every run, so this is what a
            // freshly generated world looks like before the panel speaks.
            gen_settings: LabelGenSettings { cull: Some(LabelCullMetrics::default()), ..Default::default() },
        }
    }

    /// Run the generated pass over already-swept candidates and retain the
    /// result, replacing any previous run. Mirrors
    /// [`cartalith_civ::landmark::LandmarkStore::run`].
    ///
    /// Takes candidates rather than a
    /// [`cartalith_civ::labels::LabelWorld`] on purpose: the sweep reads
    /// `WorldGen`'s own fields and this writes one of them, so a caller that
    /// passed a world would be holding `&self` and `&mut self` at once. Sweep,
    /// then place.
    ///
    /// **The hand-placed list is handed to the pass as reservations**, which is
    /// the whole of "a hand-placed label is never culled by a generated one" on
    /// this side of the wall: `generate_labels` may suppress a generated label
    /// that lands on one, and can do nothing else with them. This is also why
    /// they are measured here rather than in the pass — a reservation's
    /// tracking comes from *this* bridge's live typography table, the same
    /// table `labels_render_list` stamps on the label when it draws it.
    pub fn place(&mut self, candidates: &[cartalith_civ::labels::LabelCandidate]) -> &GeneratedLabels {
        let reserved = self.reserved_rects();
        let g = cartalith_civ::labels::generate_labels(candidates, &self.gen_settings, &self.typography, &reserved);
        self.generated = Some(g);
        self.generated.as_ref().expect("just assigned")
    }

    /// Every hand-placed label's estimated footprint, or an empty list when
    /// culling is off — measuring boxes nothing will compare against is work
    /// nobody asked for, and `generate_labels` ignores the slice in that case
    /// anyway.
    fn reserved_rects(&self) -> Vec<LabelRect> {
        let Some(m) = self.gen_settings.cull.as_ref() else { return Vec::new() };
        self.labels
            .iter()
            .filter(|lb| !lb.name.is_empty())
            .map(|lb| label_cull_rect(lb, self.typography[lb.class.index()].tracking, m))
            .collect()
    }

    /// Drop the generated run. Call when the world moves underneath it, so the
    /// panel reads `--` rather than counts from a world that no longer exists.
    pub fn invalidate_generated(&mut self) {
        self.generated = None;
    }

    /// Generated labels first, hand-placed ones second — the order the renderer
    /// draws in, so a name the user typed is never buried under a generated
    /// one.
    pub fn render_order(&self) -> impl Iterator<Item = (&MapLabel, bool)> {
        self.generated
            .iter()
            .flat_map(|g| g.labels.iter().map(|lb| (lb, true)))
            .chain(self.labels.iter().map(|lb| (lb, false)))
    }

    /// `state.labels.push({...}); _civSelectLabel(lb)` — the click-on-empty-
    /// ground branch (reference line 9771). Stored as given, even an empty
    /// name: the reference's own `if(name&&name.trim())` gate is the
    /// prompt-cancelled check on the *caller's* side (a GDScript dialog),
    /// not an engine rule — nothing in `MapLabel` forbids an empty string,
    /// and rejecting one here would just be a second, redundant gate a
    /// caller could not tell apart from the first.
    pub fn create(&mut self, x: f64, y: f64, name: impl Into<String>) -> usize {
        self.labels.push(MapLabel::new(x, y, name));
        let index = self.labels.len() - 1;
        self.session.select(&self.labels, Some(index));
        index
    }

    /// Removes a label. `false` for an out-of-range `index`.
    ///
    /// Unlike `IconEditor::delete` (a plain `Option<usize>` it can freely
    /// decrement), [`LabelEditSession`] is deliberately sealed — `selected`/
    /// `snapshot` are private, reachable only through `select`, which
    /// **retakes the snapshot** whenever the passed index differs from the
    /// one already selected. Re-pointing a shifted selection at its new
    /// index through that same call would therefore silently discard and
    /// retake the in-progress edit's revert snapshot — the exact bug
    /// `LabelEditSession`'s own doc comment says the reference fixed once
    /// already (re-selecting must not retake the snapshot). So: any delete
    /// at or before the current selection clears the session outright
    /// (`select(.., None)`) rather than risk that. A safe, conservative
    /// default a caller can always recover from with one more
    /// `label_select` — silently mis-tracking which label a live snapshot
    /// belongs to is not recoverable at all.
    pub fn delete(&mut self, index: usize) -> bool {
        if index >= self.labels.len() {
            return false;
        }
        self.labels.remove(index);
        if let Some(sel) = self.session.selected()
            && sel >= index
        {
            self.session.select(&self.labels, None);
        }
        true
    }

    /// Drops every label and ends any edit session.
    pub fn clear_all(&mut self) {
        self.labels.clear();
        self.session.select(&self.labels, None);
    }

    /// `_civLabelDrag`'s per-move assignment (reference line 9718):
    /// `label.x = gx; label.y = gy`, unclamped, no selection side effect —
    /// the reference calls this on every `pointermove` sample of a box-drag
    /// and only selects once, on release (`WorldGen::label_move`'s own doc
    /// comment has the full sequencing). `false` for an out-of-range
    /// `index` or a non-finite `x`/`y` (silently dropped, not stored, the
    /// same "never let a NaN in" policy every setter in this workspace's
    /// bridge layer shares).
    pub fn move_to(&mut self, index: usize, x: f64, y: f64) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        let Some(lb) = self.labels.get_mut(index) else { return false };
        lb.x = x;
        lb.y = y;
        true
    }

    /// Box-only hit test (`cartalith_civ::labels::label_hit_test` with an
    /// empty [`LabelHandles`], reducing it to exactly the box scan —
    /// reusing the tested function rather than re-implementing the
    /// back-to-front "topmost wins" scan). Selects the hit label, matching
    /// `IconEditor::hit_test`'s own "a hit test through this tool also
    /// selects" convention. `None` on a miss.
    ///
    /// See the module doc's "text measurement" section: every label's box
    /// is measured at `meas_w = 0.0` here — a disclosed placeholder, not a
    /// claim about the label's real rendered width.
    pub fn hit_test(&mut self, gx: f64, gy: f64, env: &LabelViewEnv) -> Option<usize> {
        let boxes: Vec<LabelBox> = self.labels.iter().map(|lb| label_box(lb, env, 0.0)).collect();
        let hit = cartalith_civ::labels::label_hit_test(&boxes, &LabelHandles::default(), gx, gy)?;
        let index = hit.index?;
        self.session.select(&self.labels, Some(index));
        Some(index)
    }

    /// The five on-canvas handle circles for label `index`'s current box —
    /// see [`handle_circles`]. `None` for an out-of-range `index`.
    pub fn handles(&self, index: usize, env: &LabelViewEnv) -> Option<LabelHandles> {
        let lb = self.labels.get(index)?;
        let box_ = label_box(lb, env, 0.0);
        Some(handle_circles(lb, &box_, env))
    }

    /// `arc_label_layout` for label `index`'s current text/arc/size, given
    /// **real** measured glyph data (see the module doc's "text
    /// measurement" section — this is the one call in this file that
    /// cannot fall back to a placeholder). `None` for an out-of-range
    /// `index`.
    pub fn glyph_layout(&self, index: usize, env: &LabelViewEnv, char_widths: &[f64], total_w: f64) -> Option<ArcLayout> {
        let lb = self.labels.get(index)?;
        let fsz = label_font_size(lb, env);
        Some(arc_label_layout(char_widths, total_w, lb.arc, fsz))
    }
}

impl Default for LabelBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// The reference's `drawCivLayer` selection-box handle geometry (lines
/// 15810-15875), transcribed rather than sliced — it is inline canvas
/// drawing code, not a callable function. Every local offset is rotated by
/// the label's own `angle` before translating to the box centre
/// (`box_.px`/`.py`), exactly the reference's `rot(lx,ly)` closure, so the
/// whole manipulation box (resize/rotate/arc handles, confirm/cancel
/// buttons) turns together with the label.
///
/// `lsc` here is the render pass's own constant
/// (`Math.max(1,GW/512)*_civZoomK()*_civIconScale()`, reference line
/// 15673) — **not** `label_font_size`'s own per-label `lsc`, which also
/// depends on that label's `size_mode` (line 15285, inside `_civLabelBox`).
/// The reference draws every label's handles at the same screen-constant
/// scale regardless of that label's own fixed/zoom size mode; ported as
/// written, not "fixed" to track each label's mode, since that would be a
/// behavioural change with no reference basis.
pub fn handle_circles(lb: &MapLabel, box_: &LabelBox, env: &LabelViewEnv) -> LabelHandles {
    let th = lb.angle * std::f64::consts::PI / 180.0;
    let (ca, sa) = (th.cos(), th.sin());
    let rot = |lx: f64, ly: f64| (box_.px + lx * ca - ly * sa, box_.py + lx * sa + ly * ca);

    let base = f64::max(1.0, env.grid_w as f64 / 512.0) * env.icon_scale;
    let lsc = base * civ_zoom_k(env.zoom_scale);
    let side = box_.side;

    // Resize: bottom-right corner of the (rotated) box.
    let hr = f64::max(4.0, 3.2 * lsc);
    let (hx, hy) = rot(side / 2.0, side / 2.0);

    // Rotate: a stem from the top edge's centre, further out.
    let stem_len = f64::max(10.0, side * 0.25);
    let rr = f64::max(4.0, 3.2 * lsc);
    let (rx, ry) = rot(0.0, -side / 2.0 - stem_len);

    // Arc/curve: a diamond offset left of the top edge.
    let ar = f64::max(4.0, 3.4 * lsc);
    let (ax, ay) = rot(-side * 0.28, -side / 2.0);

    // Confirm/cancel: the top two corners, inset by their own radius.
    let br = f64::max(6.0, 4.2 * lsc);
    let (bx0, by0) = rot(-side / 2.0 + br + 2.0 * lsc, -side / 2.0 - br - 2.0 * lsc);
    let (bx1, by1) = rot(side / 2.0 - br - 2.0 * lsc, -side / 2.0 - br - 2.0 * lsc);

    LabelHandles {
        // The reference's own hit-test radii bake in the *displayed*
        // circle's own further slack (`_civLabelHandle={..,r:hr*1.6,..}`
        // etc, reference lines 15841/15851/15862) — not the drawn radius
        // alone (`hr`/`rr`/`ar`), which is only the fill/stroke size.
        resize: Some(HandleCircle { x: hx, y: hy, r: hr * 1.6 }),
        rotate: Some(HandleCircle { x: rx, y: ry, r: rr * 1.5 }),
        arc: Some(HandleCircle { x: ax, y: ay, r: ar * 1.5 }),
        // check/cross use their drawn radius directly for the stored hit
        // circle -- `_civLabelHitTest`'s own `*1.3` slack (reference line
        // 15301-15302, `LABEL_BUTTON_SLACK` in `labels.rs`) is applied at
        // hit-test time, not baked into the stored radius, unlike the three
        // above.
        check: Some(HandleCircle { x: bx0, y: by0, r: br }),
        cross: Some(HandleCircle { x: bx1, y: by1, r: br }),
    }
}

// ---------------------------------------------------------------------------
// Style-field setters — `label_set`'s per-key targets. Position (`x`/`y`) is
// deliberately absent: `LabelStyleSnapshot` excludes it for the same reason
// (dragging to reposition commits immediately, only the form fields are
// revertible) and `move_to` above is its own, separate call.
// ---------------------------------------------------------------------------

pub fn set_text(lb: &mut MapLabel, text: String) -> Outcome {
    lb.name = text;
    Outcome::Applied
}

/// Clamped to [`LABEL_SIZE_MIN`]/[`LABEL_SIZE_MAX`] — the same range the
/// resize handle itself clamps to (`label_resize_size`), so a direct
/// numeric-field edit and a drag can never disagree about the reachable
/// range.
pub fn set_size(lb: &mut MapLabel, size: f64) -> Outcome {
    if !size.is_finite() {
        return Outcome::Rejected;
    }
    let clamped = size.clamp(LABEL_SIZE_MIN, LABEL_SIZE_MAX);
    lb.size = clamped;
    if clamped == size { Outcome::Applied } else { Outcome::Clamped }
}

/// Clamped to `[-1, 1]`. `arc_label_layout` itself also clamps at layout
/// time (`labels.rs`'s own note: "clamped at layout time rather than on
/// assignment because the reference clamps at use") — this is a second,
/// defensive clamp at the write boundary, the same input-hygiene policy
/// every other setter in this workspace's bridge layer already applies
/// (`sculpt_bridge::set_feature_param`, `params::set`). It does not change
/// what `arc_label_layout` does with an in-range value; it only stops a
/// wildly out-of-range `Dictionary` value from being stored unclamped at
/// all, which the reference's own UI (a bounded slider) would never send
/// but a malformed API call could.
pub fn set_arc(lb: &mut MapLabel, arc: f64) -> Outcome {
    if !arc.is_finite() {
        return Outcome::Rejected;
    }
    let clamped = arc.clamp(-1.0, 1.0);
    lb.arc = clamped;
    if clamped == arc { Outcome::Applied } else { Outcome::Clamped }
}

/// Unrestricted degrees — rotation is periodic (`handle_circles`/
/// `label_rotate_deg` both take `cos`/`sin` of it), so unlike `size`/`arc`
/// there is no reachable-range floor/ceiling to clamp to. Only non-finite
/// is rejected.
pub fn set_angle(lb: &mut MapLabel, angle: f64) -> Outcome {
    if !angle.is_finite() {
        return Outcome::Rejected;
    }
    lb.angle = angle;
    Outcome::Applied
}

/// `"fixed"` / `"zoom"` (`get_label_size_modes`'s own keys, mirroring
/// `sculpt_get_freehand_modes`' pattern). Any other key -> `Rejected`.
pub fn set_size_mode(lb: &mut MapLabel, mode_key: &str) -> Outcome {
    lb.size_mode = match mode_key {
        "fixed" => LabelSizeMode::Fixed,
        "zoom" => LabelSizeMode::Zoom,
        _ => return Outcome::Rejected,
    };
    Outcome::Applied
}

/// The raw CSS font string (`DCC_SHELL_SPEC.md`'s "font role" tool-options
/// control — see this crate's own report on why a named-role vocabulary
/// isn't modelled here: `MapLabel` only ever carried the literal string the
/// reference's own `lb.font||'Georgia, serif'` reads, and inventing a
/// role->string table would be new engine semantics this binding is
/// chartered not to add). Empty string resets to `None`, which renders as
/// [`cartalith_civ::labels::DEFAULT_LABEL_FONT`] — the same "no override"
/// meaning the reference's own `||` fallback gives an absent `lb.font`.
pub fn set_font(lb: &mut MapLabel, font: String) -> Outcome {
    lb.font = if font.is_empty() { None } else { Some(font) };
    Outcome::Applied
}

/// As `set_font`, for `color` (resets to
/// [`cartalith_civ::labels::DEFAULT_LABEL_COLOR`] on an empty string).
pub fn set_color(lb: &mut MapLabel, color: String) -> Outcome {
    lb.color = if color.is_empty() { None } else { Some(color) };
    Outcome::Applied
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> LabelViewEnv {
        LabelViewEnv { grid_w: 512, zoom_scale: 1.0, icon_scale: 1.0 }
    }

    // ---- LabelBridge: create / delete / clear_all ----

    #[test]
    fn create_pushes_and_selects_the_new_label() {
        let mut b = LabelBridge::new();
        let i = b.create(3.0, 4.0, "Aldar");
        assert_eq!(i, 0);
        assert_eq!(b.labels.len(), 1);
        assert_eq!(b.labels[0].name, "Aldar");
        assert_eq!(b.session.selected(), Some(0));
    }

    #[test]
    fn create_accepts_an_empty_name() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "");
        assert_eq!(b.labels[0].name, "");
    }

    #[test]
    fn delete_out_of_range_fails_and_changes_nothing() {
        let mut b = LabelBridge::new();
        assert!(!b.delete(0));
    }

    #[test]
    fn delete_the_selected_label_clears_the_session() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A");
        assert!(b.delete(0));
        assert!(b.labels.is_empty());
        assert_eq!(b.session.selected(), None);
    }

    #[test]
    fn delete_before_the_selection_also_clears_it_rather_than_mis_point() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A"); // index 0
        b.create(1.0, 1.0, "B"); // index 1, now selected
        assert_eq!(b.session.selected(), Some(1));
        assert!(b.delete(0));
        assert_eq!(b.labels.len(), 1);
        assert_eq!(b.labels[0].name, "B");
        // Conservative: cleared, not silently re-pointed at the shifted index.
        assert_eq!(b.session.selected(), None);
    }

    #[test]
    fn delete_after_the_selection_leaves_it_untouched() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A"); // index 0
        b.create(1.0, 1.0, "B"); // index 1, `create` selects it too
        // Re-select A: `delete` at/before the selection always clears it
        // (see `LabelBridge::delete`'s own doc comment), so this test needs
        // A selected, not whichever label was created last.
        b.session.select(&b.labels, Some(0));
        assert_eq!(b.session.selected(), Some(0));
        assert!(b.delete(1));
        assert_eq!(b.session.selected(), Some(0));
        assert_eq!(b.labels[0].name, "A");
    }

    #[test]
    fn clear_all_drops_everything_and_the_session() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A");
        b.clear_all();
        assert!(b.labels.is_empty());
        assert_eq!(b.session.selected(), None);
    }

    // ---- move_to ----

    #[test]
    fn move_to_sets_position_without_touching_selection() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A");
        b.session.select(&b.labels, None); // deselect, as a mid-drag really would be until release
        assert!(b.move_to(0, 5.0, 6.0));
        assert_eq!((b.labels[0].x, b.labels[0].y), (5.0, 6.0));
        assert_eq!(b.session.selected(), None);
    }

    #[test]
    fn move_to_rejects_non_finite_and_out_of_range() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "A");
        assert!(!b.move_to(0, f64::NAN, 1.0));
        assert_eq!(b.labels[0].x, 0.0);
        assert!(!b.move_to(5, 1.0, 1.0));
    }

    // ---- hit_test ----

    #[test]
    fn hit_test_selects_the_hit_label() {
        let mut b = LabelBridge::new();
        b.create(10.0, 10.0, "A");
        b.session.select(&b.labels, None);
        let hit = b.hit_test(10.5, 10.5, &env());
        assert_eq!(hit, Some(0));
        assert_eq!(b.session.selected(), Some(0));
    }

    #[test]
    fn hit_test_a_miss_returns_none_and_selects_nothing() {
        let mut b = LabelBridge::new();
        b.create(10.0, 10.0, "A");
        assert_eq!(b.hit_test(-500.0, -500.0, &env()), None);
    }

    // ---- glyph_layout ----

    #[test]
    fn glyph_layout_out_of_range_is_none() {
        let b = LabelBridge::new();
        assert_eq!(b.glyph_layout(0, &env(), &[10.0], 10.0), None);
    }

    #[test]
    fn glyph_layout_straight_below_the_arc_threshold() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "Hi"); // arc defaults to 0.0
        let layout = b.glyph_layout(0, &env(), &[10.0, 9.0], 18.0).expect("in range");
        assert_eq!(layout, ArcLayout::Straight);
    }

    #[test]
    fn glyph_layout_one_glyph_per_width_when_arched() {
        let mut b = LabelBridge::new();
        b.create(0.0, 0.0, "Hi");
        set_arc(&mut b.labels[0], 0.5);
        let ArcLayout::Arc(g) = b.glyph_layout(0, &env(), &[10.0, 9.0], 18.0).expect("in range") else {
            panic!("expected an arc")
        };
        assert_eq!(g.len(), 2);
    }

    // ---- handle_circles ----

    #[test]
    fn handle_circles_at_zero_angle_match_the_reference_formulas() {
        let lb = MapLabel::new(0.0, 0.0, "A"); // angle defaults to 0
        let box_ = LabelBox { px: 100.0, py: 50.0, side: 40.0, fsz: 16.0 };
        // grid_w=2048 -> base=4; zoom_scale=1 -> civ_zoom_k=1 -> lsc=4.
        let env = LabelViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h = handle_circles(&lb, &box_, &env);

        let resize = h.resize.expect("resize handle");
        assert!((resize.x - 120.0).abs() < 1e-9);
        assert!((resize.y - 70.0).abs() < 1e-9);
        assert!((resize.r - 20.48).abs() < 1e-9); // hr=max(4,3.2*4)=12.8, *1.6

        let rotate = h.rotate.expect("rotate handle");
        assert!((rotate.x - 100.0).abs() < 1e-9);
        assert!((rotate.y - 20.0).abs() < 1e-9); // py - side/2 - stem_len = 50-20-10
        assert!((rotate.r - 19.2).abs() < 1e-9); // rr=12.8, *1.5

        let arc = h.arc.expect("arc handle");
        assert!((arc.x - 88.8).abs() < 1e-9); // px - side*0.28 = 100-11.2
        assert!((arc.y - 30.0).abs() < 1e-9); // py - side/2 = 50-20
        assert!((arc.r - 20.4).abs() < 1e-9); // ar=13.6, *1.5

        let check = h.check.expect("check button");
        assert!((check.x - 104.8).abs() < 1e-9);
        assert!((check.y - 5.2).abs() < 1e-9);
        assert!((check.r - 16.8).abs() < 1e-9); // br=max(6,4.2*4)=16.8

        let cross = h.cross.expect("cross button");
        assert!((cross.x - 95.2).abs() < 1e-9);
        assert!((cross.y - 5.2).abs() < 1e-9);
        assert!((cross.r - 16.8).abs() < 1e-9);
    }

    #[test]
    fn handle_circles_rotate_with_the_label_at_ninety_degrees() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        lb.angle = 90.0;
        let box_ = LabelBox { px: 100.0, py: 50.0, side: 40.0, fsz: 16.0 };
        let env = LabelViewEnv { grid_w: 2048, zoom_scale: 1.0, icon_scale: 1.0 };
        let h = handle_circles(&lb, &box_, &env);
        // rot(lx,ly) at th=90 degrees -> (px - ly, py + lx).
        let resize = h.resize.expect("resize handle");
        assert!((resize.x - (100.0 - 20.0)).abs() < 1e-9);
        assert!((resize.y - (50.0 + 20.0)).abs() < 1e-9);
    }

    #[test]
    fn handle_circles_never_shrink_below_their_own_floor_at_low_zoom() {
        // grid_w=512, zoom_scale clamped to [0.35,5] by civ_zoom_k -- pushed
        // far past 5 so lsc collapses toward its minimum and the max(4,...)/
        // max(6,...) floors take over.
        let lb = MapLabel::new(0.0, 0.0, "A");
        let box_ = LabelBox { px: 0.0, py: 0.0, side: 10.0, fsz: 16.0 };
        let env = LabelViewEnv { grid_w: 512, zoom_scale: 1000.0, icon_scale: 1.0 };
        let h = handle_circles(&lb, &box_, &env);
        assert!((h.check.unwrap().r - 6.0).abs() < 1e-9);
    }

    // ---- style setters ----

    #[test]
    fn set_size_clamps_and_reports_it() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        assert_eq!(set_size(&mut lb, 999.0), Outcome::Clamped);
        assert_eq!(lb.size, LABEL_SIZE_MAX);
        assert_eq!(set_size(&mut lb, -5.0), Outcome::Clamped);
        assert_eq!(lb.size, LABEL_SIZE_MIN);
        assert_eq!(set_size(&mut lb, 20.0), Outcome::Applied);
        assert_eq!(lb.size, 20.0);
    }

    #[test]
    fn set_size_rejects_non_finite_and_leaves_the_field_alone() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        lb.size = 20.0;
        assert_eq!(set_size(&mut lb, f64::NAN), Outcome::Rejected);
        assert_eq!(lb.size, 20.0);
    }

    #[test]
    fn set_arc_clamps_to_unit_range() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        assert_eq!(set_arc(&mut lb, 5.0), Outcome::Clamped);
        assert_eq!(lb.arc, 1.0);
        assert_eq!(set_arc(&mut lb, -5.0), Outcome::Clamped);
        assert_eq!(lb.arc, -1.0);
    }

    #[test]
    fn set_angle_accepts_any_finite_degree_value() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        assert_eq!(set_angle(&mut lb, 720.0), Outcome::Applied);
        assert_eq!(lb.angle, 720.0);
        assert_eq!(set_angle(&mut lb, f64::INFINITY), Outcome::Rejected);
        assert_eq!(lb.angle, 720.0);
    }

    #[test]
    fn set_size_mode_round_trips_both_keys() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        assert_eq!(set_size_mode(&mut lb, "fixed"), Outcome::Applied);
        assert_eq!(lb.size_mode, LabelSizeMode::Fixed);
        assert_eq!(set_size_mode(&mut lb, "zoom"), Outcome::Applied);
        assert_eq!(lb.size_mode, LabelSizeMode::Zoom);
        assert_eq!(set_size_mode(&mut lb, "nope"), Outcome::Rejected);
    }

    // ---- the generated list beside the hand-placed one ----

    fn gen_cand(class: cartalith_civ::labels::LabelClass, name: &str, x: f64) -> cartalith_civ::labels::LabelCandidate {
        cartalith_civ::labels::LabelCandidate { class, name: name.to_string(), x, y: 0.0, weight: 1.0 }
    }

    #[test]
    fn a_fresh_bridge_has_not_run_the_pass_and_says_so() {
        let b = LabelBridge::new();
        assert!(b.generated.is_none(), "`--`, not `0` -- see the field's own doc comment");
        assert_eq!(b.typography, LABEL_TYPOGRAPHY_DEFAULTS);
        assert_eq!(b.render_order().count(), 0);
    }

    #[test]
    fn placing_generated_labels_never_touches_the_hand_placed_list() {
        let mut b = LabelBridge::new();
        // Culling off: this pins the *list separation*, and a hand-placed
        // label's box reaching the pass is the one legitimate way the two
        // lists touch. `a_hand_placed_label_takes_space_from_the_pass_and_never
        // _the_reverse` below is where that coupling is pinned instead.
        b.gen_settings.cull = None;
        b.create(1.0, 1.0, "Mine");
        assert_eq!(b.session.selected(), Some(0));
        let g = b.place(&[
            gen_cand(cartalith_civ::labels::LabelClass::Continental, "Landmass", 5.0),
            gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Town", 6.0),
        ]);
        assert_eq!(g.labels.len(), 2);
        // The three things a second list must not disturb: the edit list, its
        // indices, and the live edit session.
        assert_eq!(b.labels.len(), 1);
        assert_eq!(b.labels[0].name, "Mine");
        assert_eq!(b.session.selected(), Some(0));
    }

    #[test]
    fn the_render_order_puts_hand_placed_labels_over_generated_ones() {
        let mut b = LabelBridge::new();
        b.gen_settings.cull = None; // ordering, not culling -- see the test above.
        b.create(1.0, 1.0, "Mine");
        b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Auto", 5.0)]);
        let seq: Vec<(String, bool)> = b.render_order().map(|(lb, g)| (lb.name.clone(), g)).collect();
        assert_eq!(seq, vec![("Auto".to_string(), true), ("Mine".to_string(), false)]);
    }

    /// The lane's second hard rule, at the boundary that actually supplies the
    /// reservations. The engine-side half is
    /// `labels::tests::a_hand_placed_label_is_never_culled_by_a_generated_one`;
    /// this pins that `place` really hands them over, which is the wiring that
    /// could silently go missing.
    #[test]
    fn a_hand_placed_label_takes_space_from_the_pass_and_never_the_reverse() {
        let mut b = LabelBridge::new();
        assert!(b.gen_settings.cull.is_some(), "the shell's own default is culling on");
        b.create(5.0, 0.0, "Author's own name");
        let g = b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Auto", 5.0)]);
        assert!(g.labels.is_empty(), "the generated label lands on the hand-placed one and yields");
        assert_eq!(g.counts[cartalith_civ::labels::LabelClass::Settlement.index()].suppressed, 1);
        // The hand-placed label is untouched -- still there, still index 0.
        assert_eq!(b.labels.len(), 1);
        assert_eq!(b.labels[0].name, "Author's own name");
        // Move it out of the way and the same candidate is placed.
        b.move_to(0, 400.0, 400.0);
        let g2 = b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Auto", 5.0)]);
        assert_eq!(g2.labels.len(), 1);
        assert_eq!(g2.counts[cartalith_civ::labels::LabelClass::Settlement.index()].suppressed, 0);
    }

    /// An unnamed hand-placed label draws nothing (`_draw_labels` skips an
    /// empty string), so it must not reserve anything either — otherwise a
    /// label the user created and has not typed into yet silently blanks a
    /// patch of the map.
    #[test]
    fn an_empty_hand_placed_label_reserves_no_space() {
        let mut b = LabelBridge::new();
        b.create(5.0, 0.0, "");
        assert!(b.reserved_rects().is_empty());
        assert_eq!(b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Auto", 5.0)]).labels.len(), 1);
    }

    /// Measuring boxes for a comparison that will not happen is work nobody
    /// asked for, and the pass ignores the slice anyway.
    #[test]
    fn no_reservations_are_measured_while_culling_is_off() {
        let mut b = LabelBridge::new();
        b.create(5.0, 0.0, "Mine");
        assert_eq!(b.reserved_rects().len(), 1);
        b.gen_settings.cull = None;
        assert!(b.reserved_rects().is_empty());
    }

    #[test]
    fn re_running_the_pass_replaces_its_output_rather_than_appending() {
        let mut b = LabelBridge::new();
        b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "A", 1.0)]);
        b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "B", 2.0)]);
        let g = b.generated.as_ref().expect("ran twice");
        assert_eq!(g.labels.len(), 1);
        assert_eq!(g.labels[0].name, "B");
    }

    #[test]
    fn clearing_the_generated_run_leaves_hand_placed_labels_alone() {
        let mut b = LabelBridge::new();
        b.create(1.0, 1.0, "Mine");
        b.place(&[gen_cand(cartalith_civ::labels::LabelClass::Settlement, "Auto", 5.0)]);
        b.invalidate_generated();
        assert!(b.generated.is_none());
        assert_eq!(b.labels.len(), 1);
        assert_eq!(b.render_order().count(), 1);
    }

    #[test]
    fn a_typography_override_reaches_the_next_run() {
        let mut b = LabelBridge::new();
        let region = cartalith_civ::labels::LabelClass::Region;
        b.typography[region.index()].set_field("size", 30.0);
        b.place(&[gen_cand(region, "Marches", 1.0)]);
        assert_eq!(b.generated.as_ref().unwrap().labels[0].size, 30.0);
        // ...and the other four classes are untouched by it.
        assert_eq!(b.typography[cartalith_civ::labels::LabelClass::Water.index()].size, 15.0);
    }

    #[test]
    fn set_font_and_color_empty_string_resets_to_the_default() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        set_font(&mut lb, "Comic Sans".to_string());
        assert_eq!(lb.font_or_default(), "Comic Sans");
        set_font(&mut lb, "".to_string());
        assert_eq!(lb.font_or_default(), cartalith_civ::labels::DEFAULT_LABEL_FONT);

        set_color(&mut lb, "#ff0000".to_string());
        assert_eq!(lb.color_or_default(), "#ff0000");
        set_color(&mut lb, "".to_string());
        assert_eq!(lb.color_or_default(), cartalith_civ::labels::DEFAULT_LABEL_COLOR);
    }
}
