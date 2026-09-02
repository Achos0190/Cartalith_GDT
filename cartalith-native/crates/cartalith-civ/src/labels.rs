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
    /// Which of the design's five typographic classes this label belongs to
    /// ([`LabelClass`]).
    ///
    /// **Deliberately absent from [`LabelStyleSnapshot`]**, and for the same
    /// reason `x`/`y` are: the seven snapshot fields are the ones
    /// `_civLabelEditSnapshot` reverts, and this is not one of them — it did
    /// not exist in the reference at all. A class change commits immediately,
    /// like a reposition.
    ///
    /// [`LabelClass::Settlement`] on every hand-placed label, matching the
    /// design's own fallback (`parts.js:378` falls back to `CL[2]`, which is
    /// settlement).
    pub class: LabelClass,
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
            class: LabelClass::Settlement,
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

/// The line-box multiple a label's height is taken at — `_civLabelBox`'s own
/// `fsz * 1.3`, shared with [`label_cull_rect`] and restated by
/// `map_overlay.gd::_seed_label_occupancy` (`h = font_px * 1.3`).
pub const LABEL_BOX_LINE_HEIGHT: f64 = 1.3;

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
    let side = f64::max(meas_w, fsz * LABEL_BOX_LINE_HEIGHT) * 1.25;
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

// ===========================================================================
// The five label classes, their typography, and the generated labelling pass
// ===========================================================================
//
// `LARGE_ITEM_RULINGS.md`, owner ruling 2026-08-31, "CARTO ▸ Labels: the whole
// panel — all three steps, in order": (1) a `label_class` field on `MapLabel`,
// (2) a generated labelling pass emitting per-class placements, (3) a per-class
// typography record carrying size/halo/tracking.
//
// Everything above this line is a port of the reference. **Everything below it
// is not**, and cannot be: the reference has no label classes, no generated
// labelling pass and no per-class typography — `state.labels` is a flat array
// every entry of which got there through `_labelMode`'s click handler. The
// design that does describe those three things is the DCC prototype
// (`ENV:698`-`721`, `parts.js:363`/`:376`-`:398`), and the numeric values below
// are transcribed from it rather than invented here. Nothing in this section is
// golden-parity constrained, because there is no reference behaviour to match.
//
// ## The collision culler, and the one number it has to be given
//
// The same ruling files *"Label collision culling — build with the labelling
// pass. Measure-and-suppress rides in the same pass that places labels"*, and
// [`generate_labels`] is where it rides: [`LabelRect`], [`LabelCullMetrics`]
// and [`label_cull_rect`] below are its geometry, and
// [`LabelClassCount::suppressed`] is what it reports through.
//
// It needed one thing this module cannot have (see the header: glyph advances
// belong to the loaded font) and it takes that as an input rather than
// pretending to know it — [`LabelCullMetrics::advance_ratio`], a *mean* advance
// per glyph as a fraction of the font size. That makes every box here an
// estimate, and it is labelled an estimate everywhere it surfaces rather than
// dressed up as a measurement. What it is not is a guess: the shell measures
// the ratio off the font it actually draws with and sends it
// (`cartography_workspace.gd::_label_advance_ratio`).

/// One of the design's five typographic label classes (`parts.js:363`'s `CL`).
///
/// Ordered as the design lists them, largest-reading class first, and that
/// order is load-bearing twice: [`generate_labels`] emits in it, so a
/// continental name is drawn under a settlement name rather than over it, and
/// [`LabelClass::index`] indexes every `[T; 5]` table in this section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub enum LabelClass {
    Continental,
    Region,
    /// The default for a hand-placed label — `parts.js:378`'s own fallback.
    #[default]
    Settlement,
    Water,
    Landmark,
}

/// Every class, in drawing order.
pub const LABEL_CLASSES: [LabelClass; 5] =
    [LabelClass::Continental, LabelClass::Region, LabelClass::Settlement, LabelClass::Water, LabelClass::Landmark];

impl LabelClass {
    /// The stable key crossing the gdext boundary. Never shown to a user.
    pub const fn key(self) -> &'static str {
        match self {
            LabelClass::Continental => "continental",
            LabelClass::Region => "region",
            LabelClass::Settlement => "settlement",
            LabelClass::Water => "water",
            LabelClass::Landmark => "landmark",
        }
    }

    /// The row label the dock draws (`parts.js:363`).
    pub const fn label(self) -> &'static str {
        match self {
            LabelClass::Continental => "Continental",
            LabelClass::Region => "Region",
            LabelClass::Settlement => "Settlement",
            LabelClass::Water => "Water",
            LabelClass::Landmark => "Landmark",
        }
    }

    /// Position in [`LABEL_CLASSES`] and in every `[T; 5]` table here.
    pub const fn index(self) -> usize {
        match self {
            LabelClass::Continental => 0,
            LabelClass::Region => 1,
            LabelClass::Settlement => 2,
            LabelClass::Water => 3,
            LabelClass::Landmark => 4,
        }
    }

    /// `None` for an unrecognised key — including the empty string a project
    /// archive written before this field existed deserialises to, which is why
    /// callers resolve `None` to [`LabelClass::default`] rather than failing.
    pub fn from_key(key: &str) -> Option<Self> {
        LABEL_CLASSES.into_iter().find(|c| c.key() == key)
    }
}

/// A class's type spec — the three dials the design draws under the class list
/// plus the two fixed attributes that are not dials.
///
/// `parts.js:363`'s `spec` column is the compact form of exactly this:
/// `26/2.5 · .28 em` reads size / halo / tracking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelTypography {
    /// Nominal glyph size in px. The design's slider domain is
    /// [`LABEL_CLASS_SIZE_RANGE`].
    pub size: f64,
    /// Halo (outline) width in px **at [`size`](Self::size)**, not at the
    /// rendered size — see [`LabelTypography::halo_px`], which is what a
    /// renderer should call.
    pub halo: f64,
    /// Letter spacing in em. Applied by the renderer, for the reason this
    /// module's header gives: glyph advances belong to the loaded font, so
    /// there is nothing here to add them to.
    pub tracking: f64,
    /// `parts.js:363`'s water row is the only one whose spec says `italic`.
    pub italic: bool,
    /// The class's ink, straight from the design's own swatch column
    /// (`ENV:702`). Literals, deliberately: `#a9adb0` and `#6f9fb5` are not in
    /// the shell's token palette and routing them through it would substitute
    /// the nearest token.
    pub ink: &'static str,
}

impl LabelTypography {
    /// The halo width to stroke at an actual rendered `font_px`.
    ///
    /// The design states one halo px figure per class, measured at that class's
    /// own nominal [`size`](Self::size); a label drawn at another size (zoom
    /// mode, or a size override) needs the halo to travel with it, or a
    /// zoomed-out continental name ends up with an outline thicker than its
    /// stems. So the stored figure is used as a *ratio* of the nominal size.
    ///
    /// Floored at 1 px exactly as [`arc_label_line_width`] floors the
    /// reference's own halo — a sub-pixel outline does not survive
    /// rasterisation. `halo == 0` is the design's own "no halo" end of the
    /// slider ([`LABEL_CLASS_HALO_RANGE`] starts at 0) and returns `0`, not the
    /// floor: the floor exists to keep a halo visible, not to force one on.
    pub fn halo_px(&self, font_px: f64) -> f64 {
        if self.halo <= 0.0 || self.size <= 0.0 {
            return 0.0;
        }
        f64::max(1.0, font_px * self.halo / self.size)
    }

    /// Extra advance in px after each glyph at an actual rendered `font_px`.
    pub fn tracking_px(&self, font_px: f64) -> f64 {
        font_px * self.tracking
    }

    /// Write one of the three dials, clamped to its design range. Returns the
    /// stored value, or `None` for an unknown field or a non-finite input —
    /// the same "never let a NaN in" rule every setter at this port's bridge
    /// layer follows.
    pub fn set_field(&mut self, field: &str, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let (slot, range) = match field {
            "size" => (&mut self.size, LABEL_CLASS_SIZE_RANGE),
            "halo" => (&mut self.halo, LABEL_CLASS_HALO_RANGE),
            "tracking" => (&mut self.tracking, LABEL_CLASS_TRACKING_RANGE),
            _ => return None,
        };
        *slot = value.clamp(range.0, range.1);
        Some(*slot)
    }
}

/// The design's own slider domains, read off the inverse maps in
/// `parts.js:383`-`:385`: `size` is `Math.round(8 + p*26)`, `halo` is `p*4`,
/// `track` is `p*0.4`.
pub const LABEL_CLASS_SIZE_RANGE: (f64, f64) = (8.0, 34.0);
pub const LABEL_CLASS_HALO_RANGE: (f64, f64) = (0.0, 4.0);
pub const LABEL_CLASS_TRACKING_RANGE: (f64, f64) = (0.0, 0.40);

/// `parts.js:363`'s `CL` table, transcribed. Indexed by
/// [`LabelClass::index`].
///
/// **The design's per-class *counts* are not here**, and must not be added:
/// `4 · 11 · 48 · 22 · 37` is the prototype's mock data over its mock world.
/// The real counts come from [`generate_labels`] running over the real one.
pub const LABEL_TYPOGRAPHY_DEFAULTS: [LabelTypography; 5] = [
    LabelTypography { size: 26.0, halo: 2.5, tracking: 0.28, italic: false, ink: "#e0a34a" },
    LabelTypography { size: 18.0, halo: 2.0, tracking: 0.20, italic: false, ink: "#c8cbcd" },
    LabelTypography { size: 13.0, halo: 1.5, tracking: 0.06, italic: false, ink: "#a9adb0" },
    LabelTypography { size: 15.0, halo: 1.5, tracking: 0.14, italic: true, ink: "#6f9fb5" },
    LabelTypography { size: 11.0, halo: 1.2, tracking: 0.06, italic: false, ink: "#8d9296" },
];

/// This class's shipped type spec.
pub const fn label_typography_default(class: LabelClass) -> LabelTypography {
    LABEL_TYPOGRAPHY_DEFAULTS[class.index()]
}

// ---------------------------------------------------------------------------
// The generated pass
// ---------------------------------------------------------------------------

/// A label's axis-aligned footprint in **grid-cell** space, centred on the
/// label's own point — the unit the culler compares in.
///
/// Cell space, not screen space, and that is what makes a zoom-independent
/// pass able to answer a screen-space question at all. `map_overlay.gd` sizes a
/// zoom-mode label at `px = size * ppc / LABEL_ZOOM_BASE_PX_PER_CELL` where
/// `ppc` is the live pixels-per-cell, so its width in *cells* is
/// `px * ratio / ppc = size * ratio / LABEL_ZOOM_BASE_PX_PER_CELL` — the live
/// zoom cancels out exactly. Two zoom-mode labels that overlap at one zoom
/// overlap at every zoom, so there is one answer and the pass may compute it
/// without a camera.
///
/// Two things break that invariance and are stated rather than hidden:
/// `_label_font_px`'s `[LABEL_FONT_PX_MIN, LABEL_FONT_PX_MAX]` clamp, which
/// bites at extreme zoom, and [`LabelSizeMode::Fixed`], whose cell footprint
/// really does shrink as you zoom in — [`label_cull_rect`] measures a fixed
/// label at the one zoom where the two modes agree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelRect {
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
}

impl LabelRect {
    /// True when the two boxes share any area. Touching edges do not count.
    ///
    /// **A non-finite box never overlaps anything**, which is deliberate and is
    /// the NaN rule this workspace applies everywhere: `<` is false on NaN, so
    /// an unmeasurable label falls through to *drawn*. Culling is a
    /// simplification of the map and must fail towards showing the name, not
    /// towards silently deleting it.
    pub fn overlaps(&self, other: &LabelRect) -> bool {
        (self.cx - other.cx).abs() * 2.0 < self.w + other.w
            && (self.cy - other.cy).abs() * 2.0 < self.h + other.h
    }
}

/// The mean glyph advance as a fraction of the font size, used when the caller
/// supplies none.
///
/// Half an em is the standard rule of thumb for mixed-case Latin text and is
/// deliberately a round number: a spuriously precise default would read as a
/// measurement of a font this crate has never seen. The shell replaces it with
/// a real measurement of its own face.
pub const DEFAULT_LABEL_ADVANCE_RATIO: f64 = 0.5;

/// What [`label_cull_rect`] needs to turn a name into a box, and — being
/// `Some` — the switch that turns culling on at all.
///
/// Every field mirrors a constant on the renderer's side of the boundary and
/// says which, because two numbers over one quantity is how this module's
/// `civ_zoom_k`/`_civ_zoom_k` disagreement started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelCullMetrics {
    /// Mean glyph advance / font px. [`DEFAULT_LABEL_ADVANCE_RATIO`] until a
    /// caller measures its own font.
    pub advance_ratio: f64,
    /// Line-box height as a multiple of the font size —
    /// [`LABEL_BOX_LINE_HEIGHT`], which is `map_overlay.gd`'s own
    /// `h = font_px * 1.3`.
    pub line_height: f64,
    /// `map_overlay.gd::LABEL_ZOOM_BASE_PX_PER_CELL`. A zoom-mode label's font
    /// px is `size * ppc / this`, so this is the divisor that survives when
    /// `ppc` cancels; see [`LabelRect`].
    pub zoom_base_px_per_cell: f64,
}

impl Default for LabelCullMetrics {
    fn default() -> Self {
        LabelCullMetrics {
            advance_ratio: DEFAULT_LABEL_ADVANCE_RATIO,
            line_height: LABEL_BOX_LINE_HEIGHT,
            zoom_base_px_per_cell: 2.0,
        }
    }
}

/// One label's estimated footprint, in cells.
///
/// `tracking` is the label's **class's** tracking, which is how the renderer
/// resolves it too (`labels_render_list` stamps `tracking_em` from the class on
/// hand-placed and generated labels alike), so a hand-placed label is measured
/// the way it is drawn.
///
/// The box is centred on `(lb.x, lb.y)` — `map_overlay.gd::_point_to_screen`'s
/// frame, without [`label_box`]'s `+0.5`. The half-cell is uniform across every
/// box the culler compares, so it cancels in [`LabelRect::overlaps`]; using the
/// renderer's frame keeps the numbers comparable to what is on screen.
///
/// **`arc` is not modelled.** An arched label's glyphs bow outside this
/// rectangle. Generated labels are never arched ([`generate_labels`] leaves
/// `arc` at 0), so this only under-covers a hand-placed label the user bowed,
/// and it under-covers towards drawing.
pub fn label_cull_rect(lb: &MapLabel, tracking: f64, m: &LabelCullMetrics) -> LabelRect {
    let n = lb.name.chars().count() as f64;
    let size = if lb.size == 0.0 { DEFAULT_LABEL_SIZE } else { lb.size };
    // A degenerate base would hand the culler an infinity, and an infinite box
    // suppresses the entire map. Fall back rather than divide.
    let base = if m.zoom_base_px_per_cell > 0.0 {
        m.zoom_base_px_per_cell
    } else {
        LabelCullMetrics::default().zoom_base_px_per_cell
    };
    let advance = f64::max(0.0, m.advance_ratio);
    let mut w = size * (n * advance + tracking * f64::max(0.0, n - 1.0)) / base;
    let mut h = size * m.line_height / base;
    // The rotated box's own AABB. Skipped at angle 0 so the common path — every
    // generated label, and every hand-placed one nobody turned — stays exact
    // rather than passing through a sin/cos that returns 1.0 and 0.0 the long
    // way round.
    if lb.angle != 0.0 && lb.angle.is_finite() {
        let rad = lb.angle.to_radians();
        let (s, c) = (cartalith_jsmath::js_sin(rad).abs(), cartalith_jsmath::js_cos(rad).abs());
        let (w0, h0) = (w, h);
        w = w0 * c + h0 * s;
        h = w0 * s + h0 * c;
    }
    LabelRect { cx: lb.x, cy: lb.y, w, h }
}

/// One thing in the world that could carry a name, before the pass decides
/// whether it gets one.
///
/// Deliberately not any of the five entity types it is built from: the pass
/// needs a class, a string, a point and a rank, and taking whole records would
/// couple the labelling layer to the shape of five unrelated ones.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelCandidate {
    pub class: LabelClass,
    pub name: String,
    /// Grid-cell coordinates, the same frame [`MapLabel::x`]/`y` use.
    pub x: f64,
    pub y: f64,
    /// Rank **within the class**, larger first. Never compared across classes:
    /// a continent's cell count and a landmark's `0..1` importance are not the
    /// same quantity and the pass never puts them on one scale.
    pub weight: f64,
}

/// What the pass was asked to place.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelGenSettings {
    /// Per class, indexed by [`LabelClass::index`].
    pub enabled: [bool; 5],
    /// Per class; `0` means no cap.
    ///
    /// **Every class ships uncapped**, and that is the honest default rather
    /// than a missing one: a cap drops labels by rank alone, with no reference
    /// to what is actually on the map. [`Self::cull`] is the principled
    /// thinning, and the two are reported separately (`over_cap` against
    /// `suppressed`) precisely so a caller can tell which one took a name away.
    pub max_per_class: [usize; 5],
    /// Which size mode generated labels are emitted in. `Zoom` matches
    /// `MapLabel::new`'s own default, so a generated label and a hand-placed
    /// one behave the same way under the camera.
    pub size_mode: LabelSizeMode,
    /// Collision culling: `Some(metrics)` measures every label's box and
    /// suppresses one that overlaps a label already placed; `None` places
    /// everything.
    ///
    /// **`None` here and `Some` on the shell's side**, which is this
    /// workspace's standing rule for anything that changes what the engine
    /// emits: `LabelBridge::new` turns it on, matching the design's own toggle
    /// (drawn checked, `parts.js:387`), while an engine caller that asked for
    /// nothing gets every candidate it offered. One `Option` rather than a
    /// `bool` beside a metrics struct so "culling on, measured with nothing" is
    /// not a state anyone can construct.
    pub cull: Option<LabelCullMetrics>,
}

impl Default for LabelGenSettings {
    fn default() -> Self {
        LabelGenSettings {
            enabled: [true; 5],
            max_per_class: [0; 5],
            size_mode: LabelSizeMode::Zoom,
            cull: None,
        }
    }
}

/// What one class contributed to a run — the design's drawn-count column
/// (`ENV:706`), made real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelClassCount {
    pub class: LabelClass,
    /// Candidates this class offered.
    pub available: usize,
    /// Labels emitted.
    pub drawn: usize,
    /// Dropped by the caller's own `max_per_class`.
    pub over_cap: usize,
    /// Dropped by collision culling — this class's labels whose boxes hit
    /// something already placed. `0` when [`LabelGenSettings::cull`] is `None`,
    /// and `0` when nothing overlapped; those are the same number and the panel
    /// says which by whether the toggle is on.
    ///
    /// A label suppressed by a *different* class's label is counted here, on
    /// its own class's row — the row is "what this class lost", not "what this
    /// class did".
    pub suppressed: usize,
}

/// [`generate_labels`]'s whole answer.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedLabels {
    /// In [`LABEL_CLASSES`] order, and within a class by descending weight.
    pub labels: Vec<MapLabel>,
    pub counts: [LabelClassCount; 5],
}

/// Turn candidates into placed labels: filter by class, rank, cap, measure,
/// suppress what collides, and stamp each survivor with its class's typography.
///
/// The placement is the candidate's own point. That is stated rather than
/// dressed up: this culler suppresses, it never *displaces*. Nudging a label
/// off its feature to make room is a different feature with a different failure
/// mode (a name that no longer sits on the thing it names), and nothing asked
/// for it.
///
/// # Ordering, and why suppression is stable
///
/// Ordering is total and reproducible — classes in [`LABEL_CLASSES`] order,
/// then descending `weight`, ties broken by name and then by `(x, y)` — so two
/// runs over one world emit byte-identical lists and the drawn counts do not
/// flicker. The culler adds nothing to that: a label is kept exactly when its
/// box misses everything accepted *before* it, which is a pure function of the
/// prefix, so the same world and the same dials produce the same suppressed
/// set in the same order.
///
/// That the class order is also the design's own largest-reading-first order is
/// what makes the outcome sensible rather than merely stable: a continental
/// name wins against the landmark descriptor under it, never the other way
/// round.
///
/// # `reserved`
///
/// Boxes that are already on the map and are **not** the pass's to move — in
/// practice every hand-placed label ([`LabelBridge::place`] supplies them). A
/// generated label that hits one is suppressed; nothing in `reserved` is ever
/// suppressed, counted, or returned. The author put it there.
///
/// [`LabelBridge::place`]: ../../cartalith_godot/label_bridge/struct.LabelBridge.html#method.place
pub fn generate_labels(
    candidates: &[LabelCandidate],
    settings: &LabelGenSettings,
    typography: &[LabelTypography; 5],
    reserved: &[LabelRect],
) -> GeneratedLabels {
    let mut labels: Vec<MapLabel> = Vec::new();
    let mut counts = [LabelClassCount { class: LabelClass::Continental, available: 0, drawn: 0, over_cap: 0, suppressed: 0 }; 5];
    // Accepted boxes, in emit order, seeded with the untouchable ones. Only
    // built when culling is on: with `cull: None` this stays empty and the
    // measure/scan below never runs.
    let mut placed: Vec<LabelRect> = Vec::new();
    if settings.cull.is_some() {
        placed.extend_from_slice(reserved);
    }

    for class in LABEL_CLASSES {
        let ci = class.index();
        counts[ci].class = class;

        let mut mine: Vec<&LabelCandidate> =
            candidates.iter().filter(|c| c.class == class && !c.name.is_empty()).collect();
        counts[ci].available = mine.len();
        if !settings.enabled[ci] {
            continue;
        }
        mine.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.x.total_cmp(&b.x))
                .then_with(|| a.y.total_cmp(&b.y))
        });

        let cap = settings.max_per_class[ci];
        let keep = if cap == 0 { mine.len() } else { cap.min(mine.len()) };
        counts[ci].over_cap = mine.len() - keep;

        let ty = typography[ci];
        for c in mine.into_iter().take(keep) {
            let mut lb = MapLabel::new(c.x, c.y, c.name.clone());
            lb.class = class;
            lb.size = ty.size;
            lb.size_mode = settings.size_mode;
            lb.color = Some(ty.ink.to_string());

            if let Some(m) = settings.cull.as_ref() {
                let r = label_cull_rect(&lb, ty.tracking, m);
                if placed.iter().any(|p| r.overlaps(p)) {
                    counts[ci].suppressed += 1;
                    continue;
                }
                placed.push(r);
            }

            labels.push(lb);
            counts[ci].drawn += 1;
        }
    }
    GeneratedLabels { labels, counts }
}

// ---------------------------------------------------------------------------
// Where the candidates come from
// ---------------------------------------------------------------------------

/// A named water body — the one class of the five whose entity did not already
/// exist.
///
/// The other four label something the civilisation layer already names
/// ([`crate::Continent`], [`crate::Province`], [`crate::NamedSettlement`]) or
/// already types ([`crate::landmark::Landmark`]). `build_water_bodies` returns
/// a per-cell `0 = land / 1 = ocean / 2 = lake` classification and nothing
/// carries a name, so [`lake_features`] is what makes the Water class capable
/// of drawing anything at all.
#[derive(Debug, Clone, PartialEq)]
pub struct LakeFeature {
    pub name: String,
    /// Cell-space centroid.
    pub cx: f64,
    pub cy: f64,
    pub cells: usize,
}

/// A separate naming stream, for exactly the reason [`crate::civ_continent_name_rng`]
/// documents at length: a fixed-seed generator shared between two entity kinds
/// hands them the same first name, and the map then says one word twice and
/// reads as a defect. `13579` through the same `*31337 + 999` derivation the
/// reference uses, so this is that generator started elsewhere, not a second
/// scheme. Nothing golden-parity depends on it — the reference has no named
/// lakes.
pub const CIV_LAKE_NAME_RNG_SEED_INPUT: u32 = 13579;

pub fn civ_lake_name_rng() -> cartalith_rng::Mulberry32 {
    let raw = CIV_LAKE_NAME_RNG_SEED_INPUT.wrapping_mul(31337).wrapping_add(999);
    cartalith_rng::Mulberry32::new(if raw == 0 { 1 } else { raw })
}

/// The smallest lake worth a name.
///
/// A label needs a body big enough to sit on. The Water class's nominal size is
/// 15 px and `map_overlay.gd` renders a zoom-mode label at
/// `size * px_per_cell / LABEL_ZOOM_BASE_PX_PER_CELL` — 2 px per cell at the
/// base fit — so 15 px is about seven and a half cells of glyph height, and a
/// body under a 5x5-cell footprint cannot carry one without the text spilling
/// onto land. 24 cells is that footprint. Below it a pond gets no name, which
/// is what a real map does with a pond.
pub const LAKE_LABEL_MIN_CELLS: usize = 24;

/// Connected lake bodies in `water` (`build_water_bodies`' `2`), largest first,
/// dropping anything under `min_cells`, each named in its own stream.
///
/// 4-connected and non-wrapping, matching [`crate::label_land_components`]'s own
/// choice rather than `build_landmass_quality`'s 8-connected fill: a lake that
/// touches another diagonally is two lakes. No x-wrap, because a lake spanning
/// the seam would be two bodies on screen anyway and the map is drawn flat.
///
/// Returns no per-cell raster, for the memory reason [`crate::civ_continents`]
/// gives for the identical decision.
pub fn lake_features(water: &[u8], gw: usize, gh: usize, min_cells: usize) -> Vec<LakeFeature> {
    if gw == 0 || gh == 0 || water.len() < gw * gh {
        return Vec::new();
    }
    let mut comp = vec![-1i32; gw * gh];
    let mut acc: Vec<(usize, f64, f64)> = Vec::new(); // cells, sum x, sum y
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..gw * gh {
        if water[start] != 2 || comp[start] >= 0 {
            continue;
        }
        let id = acc.len() as i32;
        acc.push((0, 0.0, 0.0));
        comp[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % gw, i / gw);
            let a = &mut acc[id as usize];
            a.0 += 1;
            a.1 += x as f64;
            a.2 += y as f64;
            let visit = |j: usize, comp: &mut Vec<i32>, stack: &mut Vec<usize>| {
                if water[j] == 2 && comp[j] < 0 {
                    comp[j] = id;
                    stack.push(j);
                }
            };
            if x > 0 {
                visit(i - 1, &mut comp, &mut stack);
            }
            if x + 1 < gw {
                visit(i + 1, &mut comp, &mut stack);
            }
            if y > 0 {
                visit(i - gw, &mut comp, &mut stack);
            }
            if y + 1 < gh {
                visit(i + gw, &mut comp, &mut stack);
            }
        }
    }

    let floor = min_cells.max(1);
    let mut order: Vec<usize> = (0..acc.len()).filter(|&c| acc[c].0 >= floor).collect();
    // Largest first, ties by component index so the order is total and does not
    // depend on sort stability -- `civ_continents`' own rule.
    order.sort_by(|&a, &b| acc[b].0.cmp(&acc[a].0).then(a.cmp(&b)));

    let mut rng = civ_lake_name_rng();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    order
        .into_iter()
        .map(|c| {
            let (cells, sx, sy) = acc[c];
            LakeFeature {
                name: crate::naming::decorate(
                    &crate::naming::civ_settle_name_bounded(&mut rng, 1, &mut seen),
                    crate::naming::FeatureKind::Lake,
                    &mut rng,
                ),
                cx: sx / cells as f64,
                cy: sy / cells as f64,
                cells,
            }
        })
        .collect()
}

/// Everything the candidate sweep reads. Every field is optional in the sense
/// that an empty slice contributes nothing — a world with no civilisation layer
/// yields a settlement, region and water class that are honestly empty rather
/// than absent.
#[derive(Debug, Default, Clone, Copy)]
pub struct LabelWorld<'a> {
    pub continents: &'a [crate::Continent],
    pub provinces: &'a [crate::Province],
    pub settlements: &'a [crate::NamedSettlement],
    pub landmarks: &'a [crate::landmark::Landmark],
    /// `build_water_bodies`' per-cell classification, plus the grid it is over.
    /// `None` skips the Water class entirely, which is what a caller that has
    /// not run the water pass should pass.
    pub water: Option<&'a [u8]>,
    pub gw: usize,
    pub gh: usize,
    /// Floor for [`lake_features`]; [`LAKE_LABEL_MIN_CELLS`] is the default.
    pub lake_min_cells: usize,
}

/// Sweep a world for everything nameable, one [`LabelCandidate`] per feature.
///
/// Five sources, one per class:
///
/// | Class | Source | Text | Weight |
/// |---|---|---|---|
/// | Continental | [`crate::Continent`] | its name | cells |
/// | Region | [`crate::Province`] | its name | its capital's population |
/// | Settlement | [`crate::NamedSettlement`] | its name | population |
/// | Water | [`lake_features`] | its name | cells |
/// | Landmark | [`crate::landmark::Landmark`] | its **type label** | importance |
///
/// **The landmark row is the one that is not a proper name, and says so.**
/// `Landmark` carries `kind`, `class`, `importance` and a stable `seed`, and no
/// name — §27 of the landmark scope reserves that seed for "a later cultural or
/// naming pass", which does not exist. A generic descriptor is what a real map
/// puts on an unnamed feature ("Falls", "The Pass"), so the kind's own label is
/// used verbatim and nothing is invented. When the naming pass lands, this is
/// the one line that changes.
///
/// A province whose `capital_settlement_index` is out of range is skipped
/// rather than placed at the origin: it has no position to be labelled at.
pub fn label_candidates(world: &LabelWorld<'_>) -> Vec<LabelCandidate> {
    let mut out: Vec<LabelCandidate> = Vec::new();

    for c in world.continents {
        out.push(LabelCandidate {
            class: LabelClass::Continental,
            name: c.name.clone(),
            x: c.cx,
            y: c.cy,
            weight: c.cells as f64,
        });
    }

    for p in world.provinces {
        let Some(seat) = world.settlements.get(p.capital_settlement_index) else { continue };
        out.push(LabelCandidate {
            class: LabelClass::Region,
            name: p.name.clone(),
            x: seat.placement.x as f64,
            y: seat.placement.y as f64,
            weight: seat.pop as f64,
        });
    }

    for s in world.settlements {
        out.push(LabelCandidate {
            class: LabelClass::Settlement,
            name: s.name.clone(),
            x: s.placement.x as f64,
            y: s.placement.y as f64,
            weight: s.pop as f64,
        });
    }

    if let Some(water) = world.water {
        let floor = if world.lake_min_cells == 0 { LAKE_LABEL_MIN_CELLS } else { world.lake_min_cells };
        for lake in lake_features(water, world.gw, world.gh, floor) {
            out.push(LabelCandidate {
                class: LabelClass::Water,
                name: lake.name,
                x: lake.cx,
                y: lake.cy,
                weight: lake.cells as f64,
            });
        }
    }

    for lm in world.landmarks {
        let Some(spec) = crate::landmark::kind_spec(&lm.kind) else { continue };
        out.push(LabelCandidate {
            class: LabelClass::Landmark,
            name: spec.label.to_string(),
            x: lm.x as f64,
            y: lm.y as f64,
            weight: lm.importance,
        });
    }

    out
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
        // Step 1 of the ruling: the field exists, and a hand-placed label takes
        // the design's own fallback class rather than a sixth "none" state.
        assert_eq!(lb.class, LabelClass::Settlement);
    }

    // =======================================================================
    // Label classes, typography, and the generated pass
    // =======================================================================

    #[test]
    fn every_class_key_round_trips_and_indexes_its_own_slot() {
        assert_eq!(LABEL_CLASSES.len(), 5);
        for (i, c) in LABEL_CLASSES.into_iter().enumerate() {
            assert_eq!(c.index(), i, "{} indexes the wrong slot", c.key());
            assert_eq!(LabelClass::from_key(c.key()), Some(c));
            assert!(!c.label().is_empty());
        }
        assert_eq!(LabelClass::from_key("nope"), None);
        // What a project archive written before the field existed deserialises
        // to. It must not resolve to a class by accident.
        assert_eq!(LabelClass::from_key(""), None);
        assert_eq!(LabelClass::default(), LabelClass::Settlement);
    }

    /// `parts.js:363`'s `spec` column, pinned digit for digit. Mutating any one
    /// of these fifteen numbers changes what the map looks like and nothing
    /// else in the workspace would notice, which is exactly why they are here.
    #[test]
    fn the_typography_table_is_the_designs_own_five_specs() {
        let t = |c| label_typography_default(c);
        // 26/2.5 · .28 em
        assert_eq!((t(LabelClass::Continental).size, t(LabelClass::Continental).halo, t(LabelClass::Continental).tracking), (26.0, 2.5, 0.28));
        // 18/2 · .20 em
        assert_eq!((t(LabelClass::Region).size, t(LabelClass::Region).halo, t(LabelClass::Region).tracking), (18.0, 2.0, 0.20));
        // 13/1.5 · .06 em
        assert_eq!((t(LabelClass::Settlement).size, t(LabelClass::Settlement).halo, t(LabelClass::Settlement).tracking), (13.0, 1.5, 0.06));
        // 15/1.5 · .14 em italic  -- the only italic row
        assert_eq!((t(LabelClass::Water).size, t(LabelClass::Water).halo, t(LabelClass::Water).tracking), (15.0, 1.5, 0.14));
        // 11/1.2 · .06 em
        assert_eq!((t(LabelClass::Landmark).size, t(LabelClass::Landmark).halo, t(LabelClass::Landmark).tracking), (11.0, 1.2, 0.06));

        for c in LABEL_CLASSES {
            assert_eq!(t(c).italic, c == LabelClass::Water, "{} disagrees about italic", c.key());
            let ink = t(c).ink;
            assert!(ink.len() == 7 && ink.starts_with('#'), "{} has a malformed swatch {ink}", c.key());
        }
        // ENV:702's five swatches, and they are all different -- five classes
        // sharing an ink would make the class list unreadable.
        let inks: std::collections::BTreeSet<&str> = LABEL_CLASSES.into_iter().map(|c| t(c).ink).collect();
        assert_eq!(inks.len(), 5);
        assert_eq!(t(LabelClass::Continental).ink, "#e0a34a");
        assert_eq!(t(LabelClass::Water).ink, "#6f9fb5");
    }

    #[test]
    fn every_shipped_spec_sits_inside_its_own_slider_range() {
        for c in LABEL_CLASSES {
            let t = label_typography_default(c);
            assert!((LABEL_CLASS_SIZE_RANGE.0..=LABEL_CLASS_SIZE_RANGE.1).contains(&t.size), "{}", c.key());
            assert!((LABEL_CLASS_HALO_RANGE.0..=LABEL_CLASS_HALO_RANGE.1).contains(&t.halo), "{}", c.key());
            assert!((LABEL_CLASS_TRACKING_RANGE.0..=LABEL_CLASS_TRACKING_RANGE.1).contains(&t.tracking), "{}", c.key());
        }
        // `parts.js:383`-`:385`'s own inverse maps: 8+p*26, p*4, p*0.4.
        assert_eq!(LABEL_CLASS_SIZE_RANGE, (8.0, 34.0));
        assert_eq!(LABEL_CLASS_HALO_RANGE, (0.0, 4.0));
        assert_eq!(LABEL_CLASS_TRACKING_RANGE, (0.0, 0.40));
    }

    #[test]
    fn the_halo_travels_with_the_rendered_size_and_floors_at_one_pixel() {
        let t = label_typography_default(LabelClass::Continental); // 26 px / 2.5 px
        assert!((t.halo_px(26.0) - 2.5).abs() < 1e-12, "at nominal size the halo is the stated figure");
        assert!((t.halo_px(52.0) - 5.0).abs() < 1e-12, "doubling the glyph doubles the halo");
        // 4 px of glyph would give 0.38 px of halo, which does not rasterise.
        assert!((t.halo_px(4.0) - 1.0).abs() < 1e-12);
        // The slider's own zero end means no halo, not a one-pixel one.
        let off = LabelTypography { halo: 0.0, ..t };
        assert_eq!(off.halo_px(26.0), 0.0);
    }

    #[test]
    fn tracking_is_ems_of_the_rendered_size() {
        let t = label_typography_default(LabelClass::Continental); // .28 em
        assert!((t.tracking_px(100.0) - 28.0).abs() < 1e-12);
        assert_eq!(label_typography_default(LabelClass::Settlement).tracking_px(0.0), 0.0);
    }

    #[test]
    fn set_field_clamps_each_dial_to_its_own_range_and_rejects_the_rest() {
        let mut t = label_typography_default(LabelClass::Region);
        assert_eq!(t.set_field("size", 999.0), Some(LABEL_CLASS_SIZE_RANGE.1));
        assert_eq!(t.set_field("size", -5.0), Some(LABEL_CLASS_SIZE_RANGE.0));
        assert_eq!(t.set_field("halo", 3.0), Some(3.0));
        assert_eq!(t.halo, 3.0);
        assert_eq!(t.set_field("tracking", 9.0), Some(LABEL_CLASS_TRACKING_RANGE.1));
        assert_eq!(t.set_field("italic", 1.0), None, "italic is a spec attribute, not a dial");
        assert_eq!(t.set_field("nope", 1.0), None);
        let before = t;
        assert_eq!(t.set_field("size", f64::NAN), None);
        assert_eq!(t, before, "a rejected write leaves every field alone");
    }

    // ---- generate_labels ----

    fn cand(class: LabelClass, name: &str, x: f64, y: f64, w: f64) -> LabelCandidate {
        LabelCandidate { class, name: name.to_string(), x, y, weight: w }
    }

    #[test]
    fn the_pass_emits_in_class_order_and_by_descending_weight_within_a_class() {
        let cands = vec![
            cand(LabelClass::Settlement, "Small", 1.0, 1.0, 100.0),
            cand(LabelClass::Continental, "Landmass", 5.0, 5.0, 9000.0),
            cand(LabelClass::Settlement, "Big", 2.0, 2.0, 5000.0),
            cand(LabelClass::Landmark, "Peak", 3.0, 3.0, 0.9),
        ];
        let g = generate_labels(&cands, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert_eq!(g.labels.len(), 4, "nothing is dropped by an uncapped run");
        let seq: Vec<(&str, &str)> = g.labels.iter().map(|l| (l.class.key(), l.name.as_str())).collect();
        assert_eq!(
            seq,
            vec![("continental", "Landmass"), ("settlement", "Big"), ("settlement", "Small"), ("landmark", "Peak")]
        );
    }

    #[test]
    fn a_generated_label_carries_its_classs_own_type_spec() {
        let g = generate_labels(
            &[cand(LabelClass::Water, "Lake Enn", 4.0, 6.0, 40.0)],
            &LabelGenSettings::default(),
            &LABEL_TYPOGRAPHY_DEFAULTS,
            &[],
        );
        assert_eq!(g.labels.len(), 1);
        let lb = &g.labels[0];
        assert_eq!(lb.class, LabelClass::Water);
        assert_eq!(lb.size, 15.0, "the Water class's own nominal size, not MapLabel::new's 16");
        assert_eq!(lb.color_or_default(), "#6f9fb5");
        assert_eq!((lb.x, lb.y), (4.0, 6.0), "placed on its own feature");
        assert_eq!((lb.angle, lb.arc), (0.0, 0.0), "generated labels are never arched or angled");
        assert_eq!(lb.size_mode, LabelSizeMode::Zoom);
    }

    #[test]
    fn a_cap_reports_what_it_dropped_rather_than_hiding_it() {
        let cands: Vec<LabelCandidate> =
            (0..10).map(|i| cand(LabelClass::Settlement, &format!("S{i}"), i as f64, 0.0, i as f64)).collect();
        let mut s = LabelGenSettings::default();
        s.max_per_class[LabelClass::Settlement.index()] = 3;
        let g = generate_labels(&cands, &s, &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        let c = g.counts[LabelClass::Settlement.index()];
        assert_eq!((c.available, c.drawn, c.over_cap), (10, 3, 7));
        assert_eq!(c.suppressed, 0, "a cap is not a cull, and must not be reported as one");
        // The three kept are the three heaviest, in order.
        assert_eq!(g.labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), vec!["S9", "S8", "S7"]);
    }

    #[test]
    fn a_disabled_class_draws_nothing_but_still_reports_what_it_had() {
        let mut s = LabelGenSettings::default();
        s.enabled[LabelClass::Landmark.index()] = false;
        let g = generate_labels(
            &[cand(LabelClass::Landmark, "Gorge", 1.0, 1.0, 0.5), cand(LabelClass::Settlement, "Town", 2.0, 2.0, 500.0)],
            &s,
            &LABEL_TYPOGRAPHY_DEFAULTS,
            &[],
        );
        assert_eq!(g.labels.len(), 1);
        assert_eq!(g.labels[0].class, LabelClass::Settlement);
        let c = g.counts[LabelClass::Landmark.index()];
        assert_eq!((c.available, c.drawn, c.over_cap, c.suppressed), (1, 0, 0, 0));
    }

    #[test]
    fn an_empty_world_reports_five_zeroed_classes_rather_than_nothing() {
        let g = generate_labels(&[], &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert!(g.labels.is_empty());
        assert_eq!(g.counts.len(), 5);
        for (i, c) in g.counts.iter().enumerate() {
            assert_eq!(c.class, LABEL_CLASSES[i], "the counts array is class-ordered");
            assert_eq!((c.available, c.drawn, c.over_cap, c.suppressed), (0, 0, 0, 0));
        }
    }

    #[test]
    fn equal_weights_still_order_totally_so_two_runs_agree() {
        let cands = vec![
            cand(LabelClass::Settlement, "Bee", 9.0, 9.0, 1.0),
            cand(LabelClass::Settlement, "Ant", 1.0, 1.0, 1.0),
            cand(LabelClass::Settlement, "Ant", 0.0, 5.0, 1.0),
        ];
        let run = |c: &[LabelCandidate]| {
            generate_labels(c, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[])
                .labels
                .iter()
                .map(|l| (l.name.clone(), l.x))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(&cands), vec![("Ant".to_string(), 0.0), ("Ant".to_string(), 1.0), ("Bee".to_string(), 9.0)]);
        let mut reversed = cands.clone();
        reversed.reverse();
        assert_eq!(run(&cands), run(&reversed), "input order must not reach the output");
    }

    #[test]
    fn an_unnamed_candidate_is_not_placed() {
        let g = generate_labels(&[cand(LabelClass::Region, "", 1.0, 1.0, 1.0)], &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert!(g.labels.is_empty());
        assert_eq!(g.counts[LabelClass::Region.index()].available, 0);
    }

    // ---- collision culling ----

    /// The default metrics, with one number pinned so a drifting constant is a
    /// failing test rather than a quietly different map.
    fn cull_on() -> LabelGenSettings {
        let m = LabelCullMetrics::default();
        assert_eq!(
            (m.advance_ratio, m.line_height, m.zoom_base_px_per_cell),
            (0.5, 1.3, 2.0),
            "map_overlay.gd restates all three; a change here is a change there"
        );
        LabelGenSettings { cull: Some(m), ..Default::default() }
    }

    /// A settlement label's estimated box, in cells, at the shipped metrics.
    /// `13 px * (n*0.5 + 0.06*(n-1)) / 2` wide, `13 * 1.3 / 2 = 8.45` tall.
    fn settlement_box_w(chars: usize) -> f64 {
        let n = chars as f64;
        13.0 * (n * 0.5 + 0.06 * (n - 1.0)) / 2.0
    }

    #[test]
    fn the_box_is_the_renderers_own_formula_divided_by_its_own_px_per_cell() {
        let mut lb = MapLabel::new(10.0, 20.0, "Ashfen");
        lb.class = LabelClass::Settlement;
        lb.size = 13.0;
        let r = label_cull_rect(&lb, 0.06, &LabelCullMetrics::default());
        assert_eq!((r.cx, r.cy), (10.0, 20.0), "the renderer's frame, no half-cell");
        assert!((r.w - settlement_box_w(6)).abs() < 1e-12, "w = {}", r.w);
        assert!((r.h - 13.0 * LABEL_BOX_LINE_HEIGHT / 2.0).abs() < 1e-12, "h = {}", r.h);
    }

    /// The invariance [`LabelRect`] claims: doubling the on-screen scale
    /// doubles the font px and the px-per-cell together, so the cell footprint
    /// is unchanged and the overlap answer cannot depend on the camera.
    #[test]
    fn a_zoom_mode_boxs_cell_footprint_does_not_move_with_the_zoom() {
        let lb = MapLabel::new(0.0, 0.0, "Ashfen");
        let base = label_cull_rect(&lb, 0.06, &LabelCullMetrics::default());
        // px_per_cell doubles => font px doubles => width in px doubles =>
        // width in cells is the same. That is the division this struct folds
        // into one constant, so the check is that the constant is the only
        // scale in the formula: halving it doubles the box exactly.
        let m = LabelCullMetrics { zoom_base_px_per_cell: 1.0, ..Default::default() };
        let half = label_cull_rect(&lb, 0.06, &m);
        assert!((half.w - base.w * 2.0).abs() < 1e-12);
        assert!((half.h - base.h * 2.0).abs() < 1e-12);
    }

    #[test]
    fn a_wider_font_makes_wider_boxes_and_culls_more() {
        // Two 3-glyph settlement names, 6 cells apart. At the shipped half-em
        // the boxes are 10.53 cells wide and 6 < 10.53 collides; at a quarter
        // em they are 5.655 and 6 clears. The measured ratio the shell sends is
        // therefore load-bearing, not decorative.
        let cands = vec![
            cand(LabelClass::Settlement, "Aaa", 0.0, 0.0, 9.0),
            cand(LabelClass::Settlement, "Bbb", 6.0, 0.0, 8.0),
        ];
        assert!((settlement_box_w(3) - 10.53).abs() < 1e-9, "{}", settlement_box_w(3));
        let narrow = LabelGenSettings {
            cull: Some(LabelCullMetrics { advance_ratio: 0.25, ..Default::default() }),
            ..Default::default()
        };
        assert_eq!(generate_labels(&cands, &narrow, &LABEL_TYPOGRAPHY_DEFAULTS, &[]).labels.len(), 2);
        assert_eq!(generate_labels(&cands, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]).labels.len(), 1);
    }

    #[test]
    fn the_heavier_candidate_survives_and_the_lighter_one_is_counted_suppressed() {
        let cands = vec![
            cand(LabelClass::Settlement, "Small", 1.0, 1.0, 10.0),
            cand(LabelClass::Settlement, "Big", 1.0, 1.0, 5000.0),
        ];
        let g = generate_labels(&cands, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert_eq!(g.labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), vec!["Big"]);
        let c = g.counts[LabelClass::Settlement.index()];
        assert_eq!(
            (c.available, c.drawn, c.over_cap, c.suppressed),
            (2, 1, 0, 1),
            "a suppressed label is counted, never silently dropped"
        );
    }

    /// The whole reason culling is off by default in this crate: the same
    /// world, the same dials, one flag apart.
    #[test]
    fn culling_off_places_the_overlapping_pair_and_reports_no_suppression() {
        let cands = vec![
            cand(LabelClass::Settlement, "Small", 1.0, 1.0, 10.0),
            cand(LabelClass::Settlement, "Big", 1.0, 1.0, 5000.0),
        ];
        let g = generate_labels(&cands, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert_eq!(g.labels.len(), 2);
        assert_eq!(g.counts[LabelClass::Settlement.index()].suppressed, 0);
        assert!(LabelGenSettings::default().cull.is_none(), "the engine default is off");
    }

    /// The class order is largest-reading first, and the culler inherits it:
    /// the continental name is placed before the landmark descriptor under it,
    /// so the descriptor is the one that goes.
    #[test]
    fn a_bigger_class_wins_against_a_smaller_one_under_it() {
        let cands = vec![
            cand(LabelClass::Landmark, "Falls", 40.0, 40.0, 0.9),
            cand(LabelClass::Continental, "Ardenne", 40.0, 40.0, 9000.0),
        ];
        let g = generate_labels(&cands, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert_eq!(g.labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), vec!["Ardenne"]);
        assert_eq!(g.counts[LabelClass::Landmark.index()].suppressed, 1, "counted on its own class's row");
        assert_eq!(g.counts[LabelClass::Continental.index()].suppressed, 0);
    }

    /// The lane's second hard rule. A hand-placed label is not in `candidates`
    /// and is not the pass's to move; it only ever takes space away.
    #[test]
    fn a_hand_placed_label_is_never_culled_by_a_generated_one() {
        let mut hand = MapLabel::new(40.0, 40.0, "Author's own name");
        hand.class = LabelClass::Settlement;
        let reserved = vec![label_cull_rect(&hand, 0.06, &LabelCullMetrics::default())];
        let cands = vec![cand(LabelClass::Continental, "Ardenne", 40.0, 40.0, 9000.0)];

        let g = generate_labels(&cands, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &reserved);
        assert!(g.labels.is_empty(), "the continental name yields to the hand-placed one, not the reverse");
        assert_eq!(g.counts[LabelClass::Continental.index()].suppressed, 1);
        // And nothing in `reserved` is ever counted or returned.
        let total: usize = g.counts.iter().map(|c| c.available).sum();
        assert_eq!(total, 1, "a reservation is not a candidate");
    }

    #[test]
    fn a_reservation_is_ignored_entirely_when_culling_is_off() {
        let hand = MapLabel::new(40.0, 40.0, "Author's own name");
        let reserved = vec![label_cull_rect(&hand, 0.06, &LabelCullMetrics::default())];
        let cands = vec![cand(LabelClass::Continental, "Ardenne", 40.0, 40.0, 9000.0)];
        let g = generate_labels(&cands, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &reserved);
        assert_eq!(g.labels.len(), 1);
    }

    /// Deterministic and stable, which is the lane's first hard rule: input
    /// order must not reach the suppressed set either.
    #[test]
    fn the_suppressed_set_is_the_same_set_in_the_same_order_however_the_input_arrives() {
        let cands: Vec<LabelCandidate> = (0..40)
            .map(|i| {
                cand(
                    LabelClass::Settlement,
                    &format!("Town{i:02}"),
                    (i % 8) as f64 * 5.0,
                    (i / 8) as f64 * 5.0,
                    // Deliberately many ties, so the name/x/y tie-break carries
                    // the whole ordering and with it the whole cull decision.
                    (i % 3) as f64,
                )
            })
            .collect();
        let run = |c: &[LabelCandidate]| {
            let g = generate_labels(c, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
            (g.labels.iter().map(|l| l.name.clone()).collect::<Vec<_>>(), g.counts[LabelClass::Settlement.index()])
        };
        let forward = run(&cands);
        assert!(forward.1.suppressed > 0, "the fixture has to actually collide, or it proves nothing");
        assert_eq!(forward.1.drawn + forward.1.suppressed, 40);

        let mut reversed = cands.clone();
        reversed.reverse();
        assert_eq!(forward, run(&reversed), "input order must not reach the suppressed set");
        assert_eq!(forward, run(&cands), "and two identical runs agree");
    }

    #[test]
    fn a_rotated_hand_placed_box_reserves_the_space_it_actually_covers() {
        let mut flat = MapLabel::new(0.0, 0.0, "Long name here");
        flat.size = 20.0;
        let m = LabelCullMetrics::default();
        let a = label_cull_rect(&flat, 0.0, &m);
        let mut turned = flat.clone();
        turned.angle = 90.0;
        let b = label_cull_rect(&turned, 0.0, &m);
        assert!((b.w - a.h).abs() < 1e-9, "a quarter turn swaps the sides: {b:?} vs {a:?}");
        assert!((b.h - a.w).abs() < 1e-9);
        // A label the user turned 90 degrees now blocks a tall column, and a
        // generated label above it is suppressed where the unrotated box would
        // have missed.
        let above = vec![cand(LabelClass::Settlement, "Under", 0.0, a.w * 0.4, 100.0)];
        assert!(generate_labels(&above, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[b]).labels.is_empty());
        assert_eq!(generate_labels(&above, &cull_on(), &LABEL_TYPOGRAPHY_DEFAULTS, &[a]).labels.len(), 1);
    }

    /// Culling must fail towards drawing. A box it cannot measure is not a
    /// licence to delete a name.
    #[test]
    fn an_unmeasurable_box_overlaps_nothing_and_suppresses_nothing() {
        let nan = LabelRect { cx: f64::NAN, cy: 0.0, w: 10.0, h: 10.0 };
        let real = LabelRect { cx: 0.0, cy: 0.0, w: 10.0, h: 10.0 };
        assert!(!nan.overlaps(&real));
        assert!(!real.overlaps(&nan));
        assert!(real.overlaps(&real));
        // And a caller that hands the metrics a degenerate scale gets the
        // shipped one back rather than an infinite box that culls the world.
        let lb = MapLabel::new(0.0, 0.0, "Ashfen");
        let bad = LabelCullMetrics { zoom_base_px_per_cell: 0.0, ..Default::default() };
        assert_eq!(label_cull_rect(&lb, 0.06, &bad), label_cull_rect(&lb, 0.06, &LabelCullMetrics::default()));
    }

    #[test]
    fn touching_edges_do_not_count_as_a_collision() {
        let a = LabelRect { cx: 0.0, cy: 0.0, w: 10.0, h: 4.0 };
        let b = LabelRect { cx: 10.0, cy: 0.0, w: 10.0, h: 4.0 };
        assert!(!a.overlaps(&b), "centres exactly one full width apart share an edge, not an area");
        let c = LabelRect { cx: 9.99, cy: 0.0, w: 10.0, h: 4.0 };
        assert!(a.overlaps(&c));
    }

    // ---- lake_features ----

    /// `gw x gh` of land with the listed cells set to lake.
    fn water_grid(gw: usize, gh: usize, lake: &[(usize, usize)]) -> Vec<u8> {
        let mut w = vec![0u8; gw * gh];
        for &(x, y) in lake {
            w[y * gw + x] = 2;
        }
        w
    }

    #[test]
    fn two_lakes_touching_only_diagonally_are_two_lakes() {
        // A 3x3 block at (1,1) and a single cell at (4,4) -- diagonal from the
        // block's corner (3,3) is (4,4), so an 8-connected fill would merge
        // them. 4-connected must not.
        let mut cells: Vec<(usize, usize)> = Vec::new();
        for y in 1..4 {
            for x in 1..4 {
                cells.push((x, y));
            }
        }
        cells.push((4, 4));
        let w = water_grid(8, 8, &cells);
        let lakes = lake_features(&w, 8, 8, 1);
        assert_eq!(lakes.len(), 2);
        assert_eq!(lakes[0].cells, 9, "largest first");
        assert_eq!(lakes[1].cells, 1);
        assert!((lakes[0].cx - 2.0).abs() < 1e-12 && (lakes[0].cy - 2.0).abs() < 1e-12);
        assert!((lakes[1].cx - 4.0).abs() < 1e-12);
    }

    #[test]
    fn a_pond_under_the_floor_gets_no_name() {
        let w = water_grid(8, 8, &[(2, 2), (2, 3)]);
        assert!(lake_features(&w, 8, 8, 3).is_empty());
        assert_eq!(lake_features(&w, 8, 8, 2).len(), 1);
        // `min_cells = 0` must not admit a zero-cell body; the floor is raised
        // to 1 rather than trusted.
        assert_eq!(lake_features(&w, 8, 8, 0).len(), 1);
    }

    /// The constant a caller gets when it does not choose one. It is a design
    /// value, so it is pinned; see `LAKE_LABEL_MIN_CELLS`' own reasoning for
    /// where 24 comes from.
    #[test]
    fn the_lake_floor_is_the_stated_footprint() {
        assert_eq!(LAKE_LABEL_MIN_CELLS, 24);
        let mut cells: Vec<(usize, usize)> = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                cells.push((x + 1, y + 1)); // 25 cells, one over the floor
            }
        }
        let w = water_grid(16, 16, &cells);
        assert_eq!(lake_features(&w, 16, 16, LAKE_LABEL_MIN_CELLS).len(), 1);
        cells.truncate(23);
        let w = water_grid(16, 16, &cells);
        assert!(lake_features(&w, 16, 16, LAKE_LABEL_MIN_CELLS).is_empty());
    }

    #[test]
    fn lake_names_are_non_empty_unique_and_read_as_water() {
        // Twelve separated single-cell lakes on a 16x16 grid.
        let cells: Vec<(usize, usize)> = (0..12).map(|i| (1 + (i % 4) * 4, 1 + (i / 4) * 4)).collect();
        let w = water_grid(16, 16, &cells);
        let lakes = lake_features(&w, 16, 16, 1);
        assert_eq!(lakes.len(), 12);
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for l in &lakes {
            assert!(!l.name.trim().is_empty(), "an unnamed lake is not a labellable feature");
            assert!(seen.insert(l.name.as_str()), "duplicate lake name {}", l.name);
            let watery = l.name.starts_with("Lake ") || l.name.ends_with(" Mere") || l.name.ends_with(" Water");
            assert!(watery, "{} does not read as a lake", l.name);
        }
        // Deterministic: the same grid names the same lakes.
        assert_eq!(lake_features(&w, 16, 16, 1), lakes);
    }

    /// What [`CIV_LAKE_NAME_RNG_SEED_INPUT`] exists for, asserted rather than
    /// assumed.
    ///
    /// [`crate::civ_continent_name_rng`]'s own doc records the bug that
    /// produced this rule: two entity kinds drawing from one fixed-seed
    /// generator get the *same first name*, and the map says one word twice.
    /// A test that only checked uniqueness within the lake list would pass on a
    /// seed that collided with the continent stream, so the assertion has to be
    /// across streams.
    #[test]
    fn the_lake_naming_stream_does_not_start_where_the_other_two_do() {
        let mut seen = std::collections::BTreeSet::new();
        let lake = crate::naming::civ_settle_name_bounded(&mut civ_lake_name_rng(), 1, &mut seen);
        let mut seen2 = std::collections::BTreeSet::new();
        let continent = crate::naming::civ_settle_name_bounded(&mut crate::civ_continent_name_rng(), 1, &mut seen2);
        let mut seen3 = std::collections::BTreeSet::new();
        let settlement = crate::naming::civ_settle_name_bounded(&mut crate::civ_name_rng(), 1, &mut seen3);
        assert_ne!(lake, continent, "a lake and a continent would be named the same thing");
        assert_ne!(lake, settlement, "a lake and a settlement would be named the same thing");
    }

    /// Every one of `FeatureKind::Lake`'s three forms is reachable. A weight
    /// driven to an endpoint would silently make a map's lakes all read alike,
    /// and nothing else here would notice.
    #[test]
    fn all_three_lake_forms_actually_occur() {
        let cells: Vec<(usize, usize)> = (0..40).map(|i| (1 + (i % 8) * 4, 1 + (i / 8) * 4)).collect();
        let w = water_grid(36, 36, &cells);
        let lakes = lake_features(&w, 36, 36, 1);
        assert_eq!(lakes.len(), 40);
        let (mut lake_form, mut mere, mut water) = (0, 0, 0);
        for l in &lakes {
            if l.name.starts_with("Lake ") {
                lake_form += 1;
            } else if l.name.ends_with(" Mere") {
                mere += 1;
            } else if l.name.ends_with(" Water") {
                water += 1;
            }
        }
        assert_eq!(lake_form + mere + water, 40, "every name took one of the three forms");
        assert!(lake_form > 0 && mere > 0 && water > 0, "forms: {lake_form}/{mere}/{water}");
        // "Lake X" is meant to dominate -- see `decorate`'s own comment.
        assert!(lake_form > mere && lake_form > water, "forms: {lake_form}/{mere}/{water}");
    }

    #[test]
    fn lake_features_refuse_a_grid_that_does_not_match_its_own_dimensions() {
        assert!(lake_features(&[2, 2, 2], 8, 8, 1).is_empty());
        assert!(lake_features(&[], 0, 0, 1).is_empty());
    }

    #[test]
    fn ocean_is_not_a_lake() {
        let mut w = vec![1u8; 64]; // all ocean
        w[9] = 2;
        let lakes = lake_features(&w, 8, 8, 1);
        assert_eq!(lakes.len(), 1);
        assert_eq!(lakes[0].cells, 1);
    }

    // ---- label_candidates ----

    fn settlement(name: &str, x: usize, y: usize, pop: u32) -> crate::NamedSettlement {
        crate::NamedSettlement {
            tid: 0,
            placement: crate::SettlementPlacement {
                x,
                y,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: crate::SettlementKind::Town,
                coastal: false,
            },
            name: name.to_string(),
            pop,
        }
    }

    #[test]
    fn the_sweep_finds_one_candidate_per_named_feature() {
        let settlements = vec![settlement("Aldar", 10, 20, 5000), settlement("Bryn", 30, 40, 900)];
        let continents = vec![crate::Continent {
            id: 1,
            name: "Greater Enn".to_string(),
            cells: 4000,
            min_x: 0,
            min_y: 0,
            max_x: 63,
            max_y: 63,
            cx: 31.5,
            cy: 31.5,
            faction: 1,
        }];
        let provinces = vec![
            crate::Province { id: 1, faction: 1, name: "Aldar Province".to_string(), capital_settlement_index: 0 },
            // Out of range: no seat, so no position to be labelled at.
            crate::Province { id: 2, faction: 1, name: "Nowhere".to_string(), capital_settlement_index: 99 },
        ];
        let landmarks = vec![crate::landmark::Landmark {
            id: 1,
            kind: "waterfall".to_string(),
            class: crate::landmark::LandmarkClass::Regional,
            x: 7,
            y: 8,
            elevation: 300.0,
            score: 0.8,
            importance: 0.6,
            causal: Vec::new(),
            seed: 1,
        }];
        let water = water_grid(16, 16, &(0..5).flat_map(|y| (0..5).map(move |x| (x + 1, y + 1))).collect::<Vec<_>>());

        let world = LabelWorld {
            continents: &continents,
            provinces: &provinces,
            settlements: &settlements,
            landmarks: &landmarks,
            water: Some(&water),
            gw: 16,
            gh: 16,
            lake_min_cells: 0, // -> LAKE_LABEL_MIN_CELLS
        };
        let cands = label_candidates(&world);
        let by_class = |c: LabelClass| cands.iter().filter(|k| k.class == c).count();
        assert_eq!(by_class(LabelClass::Continental), 1);
        assert_eq!(by_class(LabelClass::Region), 1, "the seatless province is skipped, not placed at the origin");
        assert_eq!(by_class(LabelClass::Settlement), 2);
        assert_eq!(by_class(LabelClass::Water), 1);
        assert_eq!(by_class(LabelClass::Landmark), 1);
        assert_eq!(cands.len(), 6);

        let region = cands.iter().find(|k| k.class == LabelClass::Region).unwrap();
        assert_eq!((region.x, region.y), (10.0, 20.0), "a region is labelled at its capital");
        assert_eq!(region.weight, 5000.0);
        let lm = cands.iter().find(|k| k.class == LabelClass::Landmark).unwrap();
        assert_eq!(lm.name, "Waterfall", "no landmark naming pass exists; the kind's own label is used");
        assert_eq!((lm.x, lm.y), (7.0, 8.0));
        assert_eq!(lm.weight, 0.6);

        // And the whole chain: candidates in, placed labels out.
        let g = generate_labels(&cands, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert_eq!(g.labels.len(), 6);
        assert_eq!(g.counts.iter().map(|c| c.drawn).sum::<usize>(), 6);
        assert_eq!(g.labels[0].name, "Greater Enn");
    }

    #[test]
    fn a_world_with_no_civilisation_layer_yields_nothing_and_does_not_panic() {
        let cands = label_candidates(&LabelWorld::default());
        assert!(cands.is_empty());
        let g = generate_labels(&cands, &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        assert!(g.labels.is_empty());
    }

    /// The pass over a **real generated world**, not a hand-built fixture.
    ///
    /// This exists because of this port's own repeated failure mode: four
    /// subsystems shipped tests that passed on silently-empty golden output.
    /// Every other test above builds its own water grid, so all of them would
    /// still pass if `build_water_bodies`' real classification never produced a
    /// single lake component this fill could find. This one asserts it does,
    /// against terrain the engine actually generated.
    #[test]
    fn a_real_generated_world_yields_real_water_labels() {
        let (gw, gh) = (192, 192);
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, 7);
        p.world = false;
        let ws = cartalith_engine::generate_terrain(&p);
        let wb = crate::build_water_bodies(&ws.field, gw, gh, ws.sea_level, p.world, Some(&ws.rainfall));

        // The fixture itself has to be non-degenerate, or the assertion below
        // would be vacuous: this world must actually contain lake cells.
        let lake_cells = wb.classification.iter().filter(|&&c| c == 2).count();
        assert!(lake_cells > 0, "the fixture world has no lake cells at all -- pick another seed");

        // At `min_cells = 1` every component is a feature, so the count is the
        // fill's own answer about this raster.
        let all = lake_features(&wb.classification, gw, gh, 1);
        assert!(!all.is_empty(), "{lake_cells} lake cells produced no components");
        assert_eq!(all.iter().map(|l| l.cells).sum::<usize>(), lake_cells, "every lake cell landed in exactly one body");
        for l in &all {
            assert!(l.cells > 0 && !l.name.is_empty());
            assert!(l.cx >= 0.0 && l.cx < gw as f64 && l.cy >= 0.0 && l.cy < gh as f64, "centroid off the grid");
        }
        // Sorted largest first, and the sort is total.
        for pair in all.windows(2) {
            assert!(pair[0].cells >= pair[1].cells);
        }

        let world = LabelWorld { water: Some(&wb.classification), gw, gh, lake_min_cells: 1, ..Default::default() };
        let g = generate_labels(&label_candidates(&world), &LabelGenSettings::default(), &LABEL_TYPOGRAPHY_DEFAULTS, &[]);
        let water = g.counts[LabelClass::Water.index()];
        assert_eq!(water.available, all.len());
        assert_eq!(water.drawn, all.len(), "uncapped, so every candidate is placed");
        assert!(g.labels.iter().all(|lb| lb.class == LabelClass::Water && lb.size == 15.0));
    }

    #[test]
    fn an_unknown_landmark_kind_is_skipped_rather_than_labelled_with_its_key() {
        let landmarks = vec![crate::landmark::Landmark {
            id: 1,
            kind: "not_a_kind".to_string(),
            class: crate::landmark::LandmarkClass::Local,
            x: 1,
            y: 1,
            elevation: 0.0,
            score: 0.0,
            importance: 0.0,
            causal: Vec::new(),
            seed: 0,
        }];
        let world = LabelWorld { landmarks: &landmarks, ..Default::default() };
        assert!(label_candidates(&world).is_empty());
    }
}
