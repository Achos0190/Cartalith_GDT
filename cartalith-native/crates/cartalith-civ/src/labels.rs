//! Region-name labels — `UNIFIED_TOOL_PLAN.md` milestone E, the Label (`T`)
//! tool's engine half.
//!
//! Ports `drawArcLabel` (reference line 15244), `_civLabelBox` (15280),
//! `_civLabelHitTest` (15296), `_civSelectLabel`/`_civConfirmLabel`/
//! `_civCancelLabel` (15356-15367) and the three on-canvas handle formulas
//! inline in the pointer-move handler (9686-9717).
//!
//! **Why `cartalith-civ`.** Milestone A's placement rule sends generic
//! machinery to `cartalith-spatial`, pipeline knowledge to `cartalith-engine`
//! and subsystem-domain work to the owning crate; milestone D put the whole
//! Civilization tool group here for the same reason. Labels are the reference's
//! own `_civ`-prefixed family, they live in `state.labels` beside
//! `state.places`, and they draw in `drawCivLayer` beside settlements, ways and
//! territory — all of which this crate already owns. There is no second
//! consumer that would justify a crate of their own (milestone B's argument
//! against `cartalith-sculpt`), and nothing here is generic: a label's box is
//! sized from *this map's* zoom-relative icon scale.
//!
//! # The one thing that cannot live in Rust: text measurement
//!
//! Both `drawArcLabel` and `_civLabelBox` call `ctx.measureText`. Glyph
//! advances are a property of the loaded font, not of the geometry, so this
//! module takes measured widths as **inputs** and computes everything else.
//! That is the whole seam: [`arc_label_layout`] returns where each glyph goes
//! in the label's own frame, and the renderer draws it. Nothing here touches a
//! canvas, and this crate stays free of Godot exactly as `ARCHITECTURE.md`
//! requires.
//!
//! # Two commit semantics in one tool, deliberately
//!
//! The reference's own comment on `_civSelectLabel` is explicit: *"x,y are
//! deliberately excluded — dragging to reposition commits immediately, like it
//! always did; only the form fields are revertible."* So [`LabelEditSession`]
//! snapshots the seven style fields and never the position, and a cancel
//! restores the label to how it looked when the session **started**, not to
//! the most recent tweak — re-selecting an already-selected label does not
//! retake the snapshot.
//!
//! # No `PassBuffer`, for milestone D's reason
//!
//! The plan predicted it (*"arguably unnecessary for the same reason as Place
//! settlement — placing/editing one label is a discrete action with its own
//! confirm/cancel, not a brush stroke"*) and it holds: [`LabelEditSession`] is
//! the staging mechanism the reference itself uses, and layering
//! `PassBuffer<…>` on top would be a second one over the same data with nobody
//! asking for it.

use crate::js_hypot;

/// How a label's on-screen size responds to zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelSizeMode {
    /// `sizeMode: 'zoom'` (the default). Holds a constant *on-screen* size as
    /// the map zooms — the `_civZoomK()` factor is applied.
    #[default]
    Zoom,
    /// `sizeMode: 'fixed'`. Drops the zoom factor, so the label grows and
    /// shrinks with the terrain like text printed on the map.
    Fixed,
}

/// The reference's default label font, applied wherever `lb.font` is absent.
pub const DEFAULT_LABEL_FONT: &str = "Georgia, serif";
/// The reference's default label colour (`lb.color || '#f0e4c8'`).
pub const DEFAULT_LABEL_COLOR: &str = "#f0e4c8";
/// The reference's default label size (`lb.size || 16`).
pub const DEFAULT_LABEL_SIZE: f64 = 16.0;
/// The floor and ceiling the resize handle clamps `size` to.
pub const LABEL_SIZE_MIN: f64 = 8.0;
pub const LABEL_SIZE_MAX: f64 = 48.0;

/// One placed region-name label — the reference's `state.labels[i]`.
///
/// `angle` is degrees; `arc` is the `[-1, 1]` bow (`+` domes, `-` valleys),
/// clamped at layout time rather than on assignment because the reference
/// clamps at use.
#[derive(Debug, Clone, PartialEq)]
pub struct MapLabel {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub angle: f64,
    pub arc: f64,
    pub size: f64,
    /// `None` renders as [`DEFAULT_LABEL_FONT`].
    pub font: Option<String>,
    /// `None` renders as [`DEFAULT_LABEL_COLOR`].
    pub color: Option<String>,
    pub size_mode: LabelSizeMode,
}

impl MapLabel {
    /// The object `_labelMode`'s click handler pushes: everything default
    /// except position and name (reference line 9771,
    /// `{x, y, name, angle: 0, arc: 0, size: 16}`).
    pub fn new(x: f64, y: f64, name: impl Into<String>) -> Self {
        MapLabel {
            x,
            y,
            name: name.into(),
            angle: 0.0,
            arc: 0.0,
            size: DEFAULT_LABEL_SIZE,
            font: None,
            color: None,
            size_mode: LabelSizeMode::Zoom,
        }
    }

    pub fn font_or_default(&self) -> &str {
        self.font.as_deref().unwrap_or(DEFAULT_LABEL_FONT)
    }

    pub fn color_or_default(&self) -> &str {
        self.color.as_deref().unwrap_or(DEFAULT_LABEL_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Arc text layout
// ---------------------------------------------------------------------------

/// One glyph's placement in the label's own frame, *before* the whole-label
/// rotation by `angle`.
///
/// Reproduces the reference's inner `translate(gx, gy); rotate(dir*theta)`
/// pair exactly, so a renderer applies them in that order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcGlyph {
    pub dx: f64,
    pub dy: f64,
    /// Radians.
    pub rot: f64,
}

/// What [`arc_label_layout`] decided to do with a label.
#[derive(Debug, Clone, PartialEq)]
pub enum ArcLayout {
    /// `|arc| < 0.01`: the whole string is drawn once at the origin, straight
    /// (still rotated as a whole by the label's `angle`). This is not an
    /// optimisation — an arc radius derived from a near-zero bow would be
    /// astronomically large and numerically useless.
    Straight,
    /// One entry per `char`, in order.
    Arc(Vec<ArcGlyph>),
}

/// The threshold below which `drawArcLabel` draws straight text.
pub const ARC_STRAIGHT_THRESHOLD: f64 = 0.01;

/// `drawArcLabel`'s glyph layout (reference line 15244), minus the canvas.
///
/// The reference's own description: *"straight/rotated (arc≈0) or laid along a
/// virtual circular arc (WordArt "arch up"/"arch down" style) so map names can
/// hug a coastline, mountain range or sea the way hand-drawn cartouches do."*
///
/// `char_widths` are the per-`char` advances and `total_w` is
/// `measureText(text).width` — **both** are needed and they are not the same
/// number, because a real font kerns: the reference reads `total_w` once for
/// the centring offset and the per-char widths inside the loop. A port that
/// summed the char widths instead would drift on any kerned string.
///
/// `size_px` is the *untruncated* size; note the reference truncates it only
/// for the CSS font string (`${sizePx|0}px`), so the measured widths come from
/// the truncated size while the arc radius floor uses the full one. Ported as
/// written.
pub fn arc_label_layout(char_widths: &[f64], total_w: f64, arc: f64, size_px: f64) -> ArcLayout {
    let a = arc.clamp(-1.0, 1.0);
    if a.abs() < ARC_STRAIGHT_THRESHOLD {
        return ArcLayout::Straight;
    }
    // The radius is whichever is larger: a floor proportional to the text
    // height (so a short string on a hard bow does not curl into a knot), or
    // the radius that spreads the string over ~1/2.2 of a circle at |a| = 1.
    let r = f64::max(size_px * 1.2, total_w / (2.2 * a.abs()));
    let dir = if a > 0.0 { 1.0 } else { -1.0 };
    let mut acc = -total_w / 2.0;
    let mut glyphs = Vec::with_capacity(char_widths.len());
    for &w in char_widths {
        let mid = acc + w / 2.0;
        let theta = mid / r;
        glyphs.push(ArcGlyph {
            dx: r * theta.sin(),
            dy: dir * r * (1.0 - theta.cos()),
            rot: dir * theta,
        });
        acc += w;
    }
    ArcLayout::Arc(glyphs)
}

/// `ctx.lineWidth = Math.max(1, sizePx * 0.16)` — the label's halo stroke.
pub fn arc_label_line_width(size_px: f64) -> f64 {
    f64::max(1.0, size_px * 0.16)
}

// ---------------------------------------------------------------------------
// Box geometry
// ---------------------------------------------------------------------------

/// The view state a label's box depends on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelViewEnv {
    /// `GW`, the grid width in cells.
    pub grid_w: usize,
    /// `viewT.scale`, the raw zoom before `_civZoomK`'s clamp.
    ///
    /// **This port's shell does not pass `viewT.scale`.** `label_handles`'
    /// callers (`cartography_workspace.gd`, three sites) pass
    /// `viewport_host.gd::zoom()` -- the *deep* camera zoom, whose ceiling is
    /// `max(64, width_km / ZOOM_TARGET_SPAN_KM)`. In the reference the two are
    /// different numbers: `viewT.scale` stays at 1 under Tiled LOD and the
    /// deep zoom lives in `_lodZoom`, which is why `_civZoomK`'s upper clamp
    /// is free there. Here it is not free -- see [`civ_zoom_k`] for what
    /// saturates.
    pub zoom_scale: f64,
    /// `state.viz.civIconScale`.
    pub icon_scale: f64,
}

impl Default for LabelViewEnv {
    fn default() -> Self {
        LabelViewEnv { grid_w: 512, zoom_scale: 1.0, icon_scale: 1.0 }
    }
}

/// `_civZoomK()` (reference line 14980) — the reciprocal of the zoom, clamped
/// to `[0.35, 5]` first, so screen-constant elements stop growing without
/// bound at either extreme.
///
/// # The shell dropped the upper clamp and this copy keeps it
///
/// Recorded 2026-09-01, having been an undisclosed divergence. There are
/// three ports of this one reference function: here, `cartalith-assets`'
/// `manual::civ_zoom_k` (an identical copy, for the reason its own doc
/// gives), and `godot-project/map_overlay.gd::_civ_zoom_k`, which is
/// `1.0 / maxf(_camera_zoom, 0.35)` -- **no `min(5, ...)`**, deliberately,
/// since 2026-08-24. That decision is measured and sound on its own terms:
/// the shell's zoom really does run past 5, and the clamp turned every pin
/// and glyph into magnified mush at deep zoom.
///
/// The consequence for *this* copy is the part that was never written down.
/// Above `zoom_scale == 5` the term saturates at `0.2` here while the
/// shell's keeps shrinking, so anything the engine sizes off it --
/// [`label_font_size`], [`label_box`]'s `side`, and through them
/// `label_handles`' handle radii -- describes a label larger than the one
/// the shell draws, and the gap widens with depth.
///
/// **The clamp is not removed here, and should not be.** It is what
/// `_civLabelBox` does, [`LabelViewEnv::zoom_scale`] documents the input as
/// `viewT.scale` (which never reaches 5 in the reference), and this module's
/// own `zoom_k_clamps_at_both_ends` test pins it. The honest fix is on the
/// caller's side of the boundary -- either the shell passes a
/// `viewT.scale`-equivalent, or handle geometry is read back from the engine
/// instead of being redrawn from a second formula. Until one of those lands,
/// treat engine-computed label geometry as faithful only for
/// `zoom_scale <= 5`.
pub fn civ_zoom_k(zoom_scale: f64) -> f64 {
    1.0 / zoom_scale.clamp(0.35, 5.0)
}

/// The rendered font size in pixels for a label — `_civLabelBox`'s `fsz`.
///
/// Needed **before** the text can be measured (the caller measures at
/// `fsz as i32` px, matching the reference's `${fsz|0}px`), which is why it is
/// public separately from [`label_box`].
///
/// **This is not the size this port's shell draws a label at.**
/// `map_overlay.gd`'s `_label_font_px` computes its own -- `size *
/// (rect.size.x / GW) / LABEL_ZOOM_BASE_PX_PER_CELL`, clamped to
/// `[LABEL_FONT_PX_MIN, LABEL_FONT_PX_MAX]` -- and never calls this. Two
/// formulas over one quantity, so the hit box and the on-canvas handles are
/// sized against a label nobody renders wherever they disagree;
/// [`civ_zoom_k`]'s note names the widest source of disagreement. Disclosed
/// 2026-09-01: choosing one owner for the number is a change on both sides of
/// the gdext boundary, not a change here.
pub fn label_font_size(lb: &MapLabel, env: &LabelViewEnv) -> f64 {
    let base = f64::max(1.0, env.grid_w as f64 / 512.0) * env.icon_scale;
    let lsc = match lb.size_mode {
        LabelSizeMode::Fixed => base,
        LabelSizeMode::Zoom => base * civ_zoom_k(env.zoom_scale),
    };
    let size = if lb.size == 0.0 { DEFAULT_LABEL_SIZE } else { lb.size };
    f64::max(9.0, size * lsc)
}

/// A label's screen box — `_civLabelBox`'s return value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelBox {
    pub px: f64,
    pub py: f64,
    /// The **full** side of the square hit box; hit-testing compares against
    /// `side / 2` on each axis.
    pub side: f64,
    pub fsz: f64,
}

/// `_civLabelBox(lb)` with the default identity mapping `(x+0.5, y+0.5)`.
///
/// `meas_w` is `measureText(lb.name).width` at [`label_font_size`]'s truncated
/// size. The reference's own note on why the box is measured rather than a
/// fixed circle: it *"fixes the old fixed-radius-circle hit-test that missed a
/// large/arched label's actual rendered footprint and fell through to 'add a
/// new label.'"*
pub fn label_box(lb: &MapLabel, env: &LabelViewEnv, meas_w: f64) -> LabelBox {
    label_box_at(lb.x + 0.5, lb.y + 0.5, lb, env, meas_w)
}

/// `_civLabelBox(lb, toScreenFn)` — the render-time call site, which supplies
/// its own LOD-aware screen mapping. Hit-testing always uses [`label_box`]'s
/// identity mapping, because label editing is gated off while LOD is on.
pub fn label_box_at(px: f64, py: f64, lb: &MapLabel, env: &LabelViewEnv, meas_w: f64) -> LabelBox {
    let fsz = label_font_size(lb, env);
    let side = f64::max(meas_w, fsz * 1.3) * 1.25;
    LabelBox { px, py, side, fsz }
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/// A circular on-canvas handle or button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandleCircle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

impl HandleCircle {
    fn hit(&self, px: f64, py: f64, slack: f64) -> bool {
        js_hypot(px - self.x, py - self.y) <= self.r * slack
    }
}

/// The transient on-canvas controls the renderer publishes for the selected
/// label, all `None` when nothing is selected.
///
/// The order these are tested in is load-bearing and is the reference's:
/// resize, then rotate, then arc, then the ✓/✗ buttons, and only then the
/// label boxes. A handle sitting over another label's box must win.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LabelHandles {
    pub resize: Option<HandleCircle>,
    pub rotate: Option<HandleCircle>,
    pub arc: Option<HandleCircle>,
    pub check: Option<HandleCircle>,
    pub cross: Option<HandleCircle>,
}

/// What `_civLabelHitTest` found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelHitKind {
    /// `'handle'` — the resize handle.
    Resize,
    /// `'rotateHandle'`.
    Rotate,
    /// `'arcHandle'`.
    Arc,
    /// `'check'` — confirm.
    Check,
    /// `'cross'` — cancel.
    Cross,
    /// `'box'` — the label's own rendered footprint.
    Box,
}

/// A hit, with the label index for [`LabelHitKind::Box`] only (the handle
/// kinds carry their own label reference in the reference, which the caller
/// here already knows because it armed them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelHit {
    pub kind: LabelHitKind,
    pub index: Option<usize>,
}

/// The slack multiplier on the ✓/✗ buttons' radius (`b.check.r * 1.3`) — a
/// deliberately fatter tap target than the drawn circle.
pub const LABEL_BUTTON_SLACK: f64 = 1.3;

/// `_civLabelHitTest(px, py)` (reference line 15296).
///
/// `boxes` must be parallel to `labels` (the caller measured them). Labels are
/// scanned **back to front**, so the most recently added label wins an overlap
/// — the same "topmost wins" rule the renderer's draw order implies.
pub fn label_hit_test(
    boxes: &[LabelBox],
    handles: &LabelHandles,
    px: f64,
    py: f64,
) -> Option<LabelHit> {
    let no_index = |kind| Some(LabelHit { kind, index: None });
    if let Some(h) = handles.resize
        && h.hit(px, py, 1.0)
    {
        return no_index(LabelHitKind::Resize);
    }
    if let Some(h) = handles.rotate
        && h.hit(px, py, 1.0)
    {
        return no_index(LabelHitKind::Rotate);
    }
    if let Some(h) = handles.arc
        && h.hit(px, py, 1.0)
    {
        return no_index(LabelHitKind::Arc);
    }
    if let Some(b) = handles.check
        && b.hit(px, py, LABEL_BUTTON_SLACK)
    {
        return no_index(LabelHitKind::Check);
    }
    if let Some(b) = handles.cross
        && b.hit(px, py, LABEL_BUTTON_SLACK)
    {
        return no_index(LabelHitKind::Cross);
    }
    for (k, b) in boxes.iter().enumerate().rev() {
        if (px - b.px).abs() <= b.side / 2.0 && (py - b.py).abs() <= b.side / 2.0 {
            return Some(LabelHit { kind: LabelHitKind::Box, index: Some(k) });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// The edit session
// ---------------------------------------------------------------------------

/// The seven revertible fields — `_civLabelEditSnapshot`'s exact contents.
/// `x`/`y` are deliberately absent.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyleSnapshot {
    pub name: String,
    pub angle: f64,
    pub arc: f64,
    pub size: f64,
    pub font: String,
    pub color: String,
    pub size_mode: LabelSizeMode,
}

impl LabelStyleSnapshot {
    fn of(lb: &MapLabel) -> Self {
        LabelStyleSnapshot {
            name: lb.name.clone(),
            angle: lb.angle,
            arc: lb.arc,
            size: if lb.size == 0.0 { DEFAULT_LABEL_SIZE } else { lb.size },
            font: lb.font_or_default().to_string(),
            color: lb.color_or_default().to_string(),
            size_mode: lb.size_mode,
        }
    }
}

/// `_civSelectedLabel` + `_civLabelEditSnapshot`, as one value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LabelEditSession {
    selected: Option<usize>,
    snapshot: Option<LabelStyleSnapshot>,
}

impl LabelEditSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// The index currently being edited, if any.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// The snapshot [`LabelEditSession::cancel`] would restore.
    pub fn snapshot(&self) -> Option<&LabelStyleSnapshot> {
        self.snapshot.as_ref()
    }

    /// `_civSelectLabel(lb)`.
    ///
    /// Re-selecting the label that is **already** selected is a no-op, which is
    /// the whole point: the reference's own comment says the snapshot is taken
    /// *"once per edit session (re-clicking/dragging an ALREADY-selected label
    /// does not retake the snapshot, so ✗ always reverts back to how it looked
    /// when the session started, not just the most recent tweak)."*
    /// Selecting `None` always clears, even from `None`.
    pub fn select(&mut self, labels: &[MapLabel], index: Option<usize>) {
        if let Some(i) = index {
            if self.selected == Some(i) {
                return;
            }
            self.selected = Some(i);
            self.snapshot = labels.get(i).map(LabelStyleSnapshot::of);
        } else {
            self.selected = None;
            self.snapshot = None;
        }
    }

    /// `_civConfirmLabel()` — keep the edits, end the session.
    pub fn confirm(&mut self) {
        self.snapshot = None;
        self.selected = None;
    }

    /// `_civCancelLabel()` — restore the seven style fields from the snapshot
    /// and end the session. The label's **position is not restored**: dragging
    /// to reposition committed immediately.
    ///
    /// Returns whether anything was actually reverted.
    pub fn cancel(&mut self, labels: &mut [MapLabel]) -> bool {
        let reverted = match (self.selected, self.snapshot.take()) {
            (Some(i), Some(snap)) => match labels.get_mut(i) {
                Some(lb) => {
                    lb.name = snap.name;
                    lb.angle = snap.angle;
                    lb.arc = snap.arc;
                    lb.size = snap.size;
                    lb.font = Some(snap.font);
                    lb.color = Some(snap.color);
                    lb.size_mode = snap.size_mode;
                    true
                }
                None => false,
            },
            _ => false,
        };
        self.selected = None;
        reverted
    }
}

// ---------------------------------------------------------------------------
// Handle drag math
// ---------------------------------------------------------------------------

/// The resize handle: `size = clamp(startSize * dist / startDist, 8, 48)`,
/// with `dist` floored at 1 so a drag onto the label's own centre does not
/// divide the size to nothing.
///
/// Transcribed from the pointer-move handler (reference lines 9686-9689), not
/// sliced — it is inline in a DOM event listener, not a callable function.
pub fn label_resize_size(start_size: f64, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> f64 {
    let dist = f64::max(1.0, js_hypot(gx + 0.5 - cx, gy + 0.5 - cy));
    (start_size * dist / start_dist).clamp(LABEL_SIZE_MIN, LABEL_SIZE_MAX)
}

/// The rotate handle: the whole-label angle in degrees, recomputed
/// **absolutely** every move rather than relative to a grab angle — the
/// reference's own note says this matches how a corner-resize handle behaves
/// in an office app.
///
/// The derivation, from the reference's comment: for `ctx.rotate(θ)`, a point
/// at local `(0, -1)` (straight up, the handle's neutral direction) maps to
/// world `(sinθ, -cosθ)`; solving for θ from a world delta gives
/// `atan2(wx, -wy)`, equivalently `atan2(wy, wx) + 90°`. Normalised to
/// `[-180, 180]`.
///
/// Transcribed (reference lines 9698-9702).
pub fn label_rotate_deg(cx: f64, cy: f64, gx: f64, gy: f64) -> f64 {
    let wx = gx + 0.5 - cx;
    let wy = gy + 0.5 - cy;
    let deg = wy.atan2(wx) * 180.0 / std::f64::consts::PI + 90.0;
    ((deg + 180.0) % 360.0 + 360.0) % 360.0 - 180.0
}

/// The arc/curve handle: bows the label per `drawArcLabel`'s
/// *"+ = dome, - = valley"* convention.
///
/// The handle sits at local `(0, -side/2)`. The pointer is inverse-rotated
/// into the label's own frame by `angle` — **captured at grab time**, since
/// letting it track a changing angle mid-drag would make the two handles fight
/// — and the local-Y delta from neutral maps to arc, dragging up (more
/// negative `ly`) giving a more positive (dome) arc.
///
/// Transcribed (reference lines 9711-9716).
pub fn label_arc_value(cx: f64, cy: f64, grab_angle_deg: f64, side: f64, gx: f64, gy: f64) -> f64 {
    let wx = gx + 0.5 - cx;
    let wy = gy + 0.5 - cy;
    let th = grab_angle_deg * std::f64::consts::PI / 180.0;
    let ly = -wx * th.sin() + wy * th.cos();
    let neutral_ly = -side / 2.0;
    let drag_range = f64::max(20.0, side * 0.9);
    ((neutral_ly - ly) / drag_range).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn widths(n: usize, w: f64) -> Vec<f64> {
        vec![w; n]
    }

    #[test]
    fn a_flat_label_lays_out_straight() {
        assert_eq!(arc_label_layout(&widths(5, 10.0), 50.0, 0.0, 16.0), ArcLayout::Straight);
    }

    #[test]
    fn the_straight_branch_extends_just_below_the_threshold() {
        assert_eq!(arc_label_layout(&widths(5, 10.0), 50.0, 0.009, 16.0), ArcLayout::Straight);
        assert!(matches!(arc_label_layout(&widths(5, 10.0), 50.0, 0.01, 16.0), ArcLayout::Arc(_)));
        // and symmetrically on the negative side
        assert_eq!(arc_label_layout(&widths(5, 10.0), 50.0, -0.009, 16.0), ArcLayout::Straight);
        assert!(matches!(arc_label_layout(&widths(5, 10.0), 50.0, -0.01, 16.0), ArcLayout::Arc(_)));
    }

    #[test]
    fn one_glyph_out_per_measured_width_in() {
        let ArcLayout::Arc(g) = arc_label_layout(&widths(7, 9.0), 63.0, 0.5, 16.0) else {
            panic!("expected an arc")
        };
        assert_eq!(g.len(), 7);
    }

    #[test]
    fn empty_text_lays_out_to_no_glyphs_rather_than_erroring() {
        let ArcLayout::Arc(g) = arc_label_layout(&[], 0.0, 0.7, 16.0) else { panic!("expected an arc") };
        assert!(g.is_empty());
    }

    #[test]
    fn arc_beyond_one_is_clamped_not_extrapolated() {
        let a = arc_label_layout(&widths(6, 10.0), 60.0, 1.0, 20.0);
        let b = arc_label_layout(&widths(6, 10.0), 60.0, 5.0, 20.0);
        assert_eq!(a, b);
        let c = arc_label_layout(&widths(6, 10.0), 60.0, -1.0, 20.0);
        let d = arc_label_layout(&widths(6, 10.0), 60.0, -3.0, 20.0);
        assert_eq!(c, d);
    }

    #[test]
    fn a_positive_arc_domes_and_a_negative_one_valleys() {
        let ArcLayout::Arc(up) = arc_label_layout(&widths(5, 10.0), 50.0, 0.6, 16.0) else {
            panic!()
        };
        let ArcLayout::Arc(down) = arc_label_layout(&widths(5, 10.0), 50.0, -0.6, 16.0) else {
            panic!()
        };
        // Mirror images in y and in rotation; x is untouched by `dir`.
        for (u, d) in up.iter().zip(down.iter()) {
            assert!((u.dx - d.dx).abs() < 1e-12);
            assert!((u.dy + d.dy).abs() < 1e-12);
            assert!((u.rot + d.rot).abs() < 1e-12);
        }
    }

    #[test]
    fn the_string_is_centred_on_the_label_origin() {
        let ArcLayout::Arc(g) = arc_label_layout(&widths(4, 10.0), 40.0, 0.5, 16.0) else { panic!() };
        // Symmetric widths -> the first and last glyph sit symmetrically about x=0.
        assert!((g[0].dx + g[3].dx).abs() < 1e-12);
        assert!((g[0].rot + g[3].rot).abs() < 1e-12);
    }

    #[test]
    fn total_width_is_read_separately_from_the_char_widths() {
        // A kerned string measures narrower than the sum of its glyph advances.
        // If the port summed the widths instead, these two would agree.
        let cw = widths(4, 10.0);
        let a = arc_label_layout(&cw, 40.0, 0.5, 16.0);
        let b = arc_label_layout(&cw, 37.0, 0.5, 16.0);
        assert_ne!(a, b);
    }

    #[test]
    fn the_radius_floor_stops_a_short_string_curling_into_a_knot() {
        // total_w/(2.2*1) = 4.5 would be below sizePx*1.2 = 48, so the floor wins
        // and the glyph offsets stay small relative to the text height.
        let ArcLayout::Arc(g) = arc_label_layout(&[5.0, 5.0], 10.0, 1.0, 40.0) else { panic!() };
        assert!(g.iter().all(|p| p.rot.abs() < 0.11), "{g:?}");
    }

    #[test]
    fn the_halo_stroke_never_goes_below_one_pixel() {
        assert_eq!(arc_label_line_width(24.0), 3.84);
        assert_eq!(arc_label_line_width(4.0), 1.0);
    }

    #[test]
    fn zoom_k_clamps_at_both_ends() {
        assert_eq!(civ_zoom_k(1.0), 1.0);
        assert_eq!(civ_zoom_k(0.2), 1.0 / 0.35);
        assert_eq!(civ_zoom_k(9.0), 1.0 / 5.0);
    }

    #[test]
    fn fixed_size_mode_ignores_zoom_and_zoom_mode_does_not() {
        let mut lb = MapLabel::new(1.0, 1.0, "A");
        let env = LabelViewEnv { grid_w: 512, zoom_scale: 4.0, icon_scale: 1.0 };
        let zoomed = label_font_size(&lb, &env);
        lb.size_mode = LabelSizeMode::Fixed;
        let fixed = label_font_size(&lb, &env);
        assert_ne!(zoomed, fixed);
        assert_eq!(fixed, DEFAULT_LABEL_SIZE);
    }

    #[test]
    fn the_font_size_never_drops_below_nine_pixels() {
        let mut lb = MapLabel::new(0.0, 0.0, "A");
        lb.size = 8.0;
        let env = LabelViewEnv { grid_w: 512, zoom_scale: 5.0, icon_scale: 1.0 };
        assert_eq!(label_font_size(&lb, &env), 9.0);
    }

    #[test]
    fn the_box_never_narrows_past_the_text_height() {
        let lb = MapLabel::new(0.0, 0.0, "");
        let b = label_box(&lb, &LabelViewEnv::default(), 0.0);
        assert_eq!(b.side, 16.0 * 1.3 * 1.25);
    }

    #[test]
    fn a_wide_name_widens_the_box() {
        let lb = MapLabel::new(0.0, 0.0, "a very long region name");
        let b = label_box(&lb, &LabelViewEnv::default(), 400.0);
        assert_eq!(b.side, 500.0);
    }

    #[test]
    fn the_default_screen_mapping_is_the_cell_centre() {
        let lb = MapLabel::new(10.0, 8.0, "A");
        let b = label_box(&lb, &LabelViewEnv::default(), 10.0);
        assert_eq!((b.px, b.py), (10.5, 8.5));
    }

    fn boxes(n: usize) -> Vec<LabelBox> {
        (0..n).map(|i| LabelBox { px: 10.0 * i as f64, py: 0.0, side: 4.0, fsz: 16.0 }).collect()
    }

    #[test]
    fn a_miss_returns_nothing() {
        assert_eq!(label_hit_test(&boxes(3), &LabelHandles::default(), 100.0, 100.0), None);
    }

    #[test]
    fn the_topmost_label_wins_an_overlap() {
        let bs = vec![
            LabelBox { px: 0.0, py: 0.0, side: 20.0, fsz: 16.0 },
            LabelBox { px: 1.0, py: 1.0, side: 20.0, fsz: 16.0 },
        ];
        let h = label_hit_test(&bs, &LabelHandles::default(), 0.5, 0.5).expect("hit");
        assert_eq!(h.index, Some(1));
    }

    #[test]
    fn a_handle_beats_a_label_box_underneath_it() {
        let bs = vec![LabelBox { px: 0.0, py: 0.0, side: 100.0, fsz: 16.0 }];
        let handles = LabelHandles {
            resize: Some(HandleCircle { x: 5.0, y: 5.0, r: 2.0 }),
            ..Default::default()
        };
        let h = label_hit_test(&bs, &handles, 5.0, 5.0).expect("hit");
        assert_eq!(h.kind, LabelHitKind::Resize);
        assert_eq!(h.index, None);
    }

    #[test]
    fn the_handle_priority_order_is_resize_rotate_arc_check_cross() {
        let all = HandleCircle { x: 0.0, y: 0.0, r: 5.0 };
        let mut h = LabelHandles {
            resize: Some(all),
            rotate: Some(all),
            arc: Some(all),
            check: Some(all),
            cross: Some(all),
        };
        assert_eq!(label_hit_test(&[], &h, 0.0, 0.0).unwrap().kind, LabelHitKind::Resize);
        h.resize = None;
        assert_eq!(label_hit_test(&[], &h, 0.0, 0.0).unwrap().kind, LabelHitKind::Rotate);
        h.rotate = None;
        assert_eq!(label_hit_test(&[], &h, 0.0, 0.0).unwrap().kind, LabelHitKind::Arc);
        h.arc = None;
        assert_eq!(label_hit_test(&[], &h, 0.0, 0.0).unwrap().kind, LabelHitKind::Check);
        h.check = None;
        assert_eq!(label_hit_test(&[], &h, 0.0, 0.0).unwrap().kind, LabelHitKind::Cross);
    }

    #[test]
    fn the_buttons_get_a_fatter_tap_target_than_the_drawn_circle() {
        let h = LabelHandles {
            check: Some(HandleCircle { x: 0.0, y: 0.0, r: 1.0 }),
            ..Default::default()
        };
        assert!(label_hit_test(&[], &h, 1.29, 0.0).is_some());
        assert!(label_hit_test(&[], &h, 1.31, 0.0).is_none());
    }

    fn one_label() -> Vec<MapLabel> {
        let mut lb = MapLabel::new(5.0, 5.0, "Aldar");
        lb.angle = 3.0;
        lb.arc = 0.1;
        lb.size = 20.0;
        vec![lb]
    }

    #[test]
    fn selecting_snapshots_the_style_fields_and_not_the_position() {
        let labels = one_label();
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        let snap = s.snapshot().expect("snapshot");
        assert_eq!(snap.name, "Aldar");
        assert_eq!(snap.size, 20.0);
        assert_eq!(snap.font, DEFAULT_LABEL_FONT);
    }

    #[test]
    fn reselecting_the_same_label_does_not_retake_the_snapshot() {
        let mut labels = one_label();
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        labels[0].name = "Aldar Reach".into();
        s.select(&labels, Some(0));
        assert_eq!(s.snapshot().expect("snapshot").name, "Aldar");
    }

    #[test]
    fn cancel_restores_the_style_and_leaves_the_position_alone() {
        let mut labels = one_label();
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        labels[0].name = "Aldar Reach".into();
        labels[0].angle = 45.0;
        labels[0].size = 33.0;
        labels[0].x = 11.0;
        labels[0].y = 12.0;
        assert!(s.cancel(&mut labels));
        assert_eq!(labels[0].name, "Aldar");
        assert_eq!(labels[0].angle, 3.0);
        assert_eq!(labels[0].size, 20.0);
        assert_eq!((labels[0].x, labels[0].y), (11.0, 12.0));
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn confirm_keeps_the_edits() {
        let mut labels = one_label();
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        labels[0].name = "Aldar Reach".into();
        s.confirm();
        assert_eq!(labels[0].name, "Aldar Reach");
        assert_eq!(s.selected(), None);
        assert!(s.snapshot().is_none());
    }

    #[test]
    fn cancelling_with_nothing_selected_reverts_nothing() {
        let mut labels = one_label();
        let mut s = LabelEditSession::new();
        assert!(!s.cancel(&mut labels));
    }

    #[test]
    fn deselecting_clears_the_snapshot() {
        let labels = one_label();
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        s.select(&labels, None);
        assert!(s.snapshot().is_none());
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn selecting_a_different_label_retakes_the_snapshot() {
        let mut labels = one_label();
        labels.push(MapLabel::new(1.0, 1.0, "Bree"));
        let mut s = LabelEditSession::new();
        s.select(&labels, Some(0));
        s.select(&labels, Some(1));
        assert_eq!(s.snapshot().expect("snapshot").name, "Bree");
    }

    #[test]
    fn resizing_clamps_between_eight_and_forty_eight() {
        assert_eq!(label_resize_size(16.0, 10.0, 10.0, 10.0, 10.0, 5.0), 8.0);
        assert_eq!(label_resize_size(16.0, 10.0, 10.0, 60.0, 60.0, 3.0), 48.0);
    }

    #[test]
    fn rotating_straight_up_is_zero_degrees() {
        // The handle's neutral direction: directly above the centre.
        assert!(label_rotate_deg(10.0, 10.0, 9.5, 0.0).abs() < 1e-12);
    }

    #[test]
    fn rotation_is_normalised_into_minus_180_to_180() {
        for (gx, gy) in [(0.0, 0.0), (20.0, 0.0), (0.0, 20.0), (20.0, 20.0), (9.5, 20.0)] {
            let d = label_rotate_deg(10.0, 10.0, gx, gy);
            assert!((-180.0..=180.0).contains(&d), "{d}");
        }
    }

    #[test]
    fn the_arc_handle_at_its_neutral_position_reads_zero() {
        // Neutral is local (0, -side/2): directly above the centre by half the
        // box. Pointer coordinates carry the +0.5 cell-centre offset the
        // reference applies to every hit test, so the neutral grid y is
        // cy - side/2 - 0.5, not cy - side/2.
        let side = 20.0;
        let v = label_arc_value(10.0, 10.0, 0.0, side, 9.5, 10.0 - side / 2.0 - 0.5);
        assert!(v.abs() < 1e-12, "{v}");
    }

    #[test]
    fn dragging_the_arc_handle_up_domes_and_down_valleys() {
        let side = 20.0;
        let up = label_arc_value(10.0, 10.0, 0.0, side, 9.5, -20.0);
        let down = label_arc_value(10.0, 10.0, 0.0, side, 9.5, 30.0);
        assert!(up > 0.0);
        assert!(down < 0.0);
    }

    #[test]
    fn the_arc_handle_clamps_to_the_unit_range() {
        for gy in [-1000.0, 1000.0] {
            let v = label_arc_value(10.0, 10.0, 0.0, 20.0, 9.5, gy);
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn a_new_label_carries_the_references_own_defaults() {
        let lb = MapLabel::new(3.0, 4.0, "Aldar");
        assert_eq!((lb.angle, lb.arc, lb.size), (0.0, 0.0, 16.0));
        assert_eq!(lb.size_mode, LabelSizeMode::Zoom);
        assert_eq!(lb.font_or_default(), "Georgia, serif");
        assert_eq!(lb.color_or_default(), "#f0e4c8");
    }
}
