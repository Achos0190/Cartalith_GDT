extends Control
## `MAP_CONTEXT_SCOPE.md` §5 and §9.1 -- CM-3, the desktop ring. Draws the
## 8-slot compass §5.1 defines plus its one sub-ring (§5.2 rule 2), and is the
## whole gesture state machine §6's table describes: every timing decision
## (the no-wait flick, the still-hold that opens the ring *and* the card, the
## sticky Q ring) lives here. Driven entirely by plain calls from
## `context_broker.gd` -- `arm()`/`pointer()`/`release()`/`click_at()` -- which
## itself is driven by `map_overlay.gd`'s raw RMB press/motion/release and
## `app.gd`'s Q key handling. This file never reads the mouse or the keyboard
## itself: `mouse_filter` stays `IGNORE` throughout, so map_overlay's own
## `_gui_input` keeps owning every real input event, exactly as CM-1/CM-2 left
## it, and forwards to this file only the calls it needs.
##
## **Never a second implementation of arming** (§5.2 rule 5). Every leaf
## slot's `callable` is a `Callable` a provider built (`GlobalTools.
## ring_cardinals`, each workspace's own `ring_slots`) by calling the exact
## function its own dock control already calls -- this file only decides
## WHICH callable the gesture picked and runs it. It draws the compass; it
## does not invent what any slot does.
##
## `faction_banner.gd`'s `_draw()` is the in-tree precedent for a hand-drawn
## `Control` this shape (§9.1's own table names it as such).

signal hold_fired  ## §6's "hold >= 300ms, still": the ring is now visible and
	## sticky; the caller (`context_broker.gd`) opens the card beside it.
## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1's own table: "a `tool_arm` pulse on
## crossing into a slot"). Fired the moment `_hover`/`_sub.hover` transitions
## from empty (or a different slot) to a real one -- never on every sample, or
## a still finger sitting on one slot would buzz every frame. Top ring and
## sub-ring share it; `context_broker.gd` does not need to know which.
signal hover_entered

## Ruling BC (`LARGE_ITEM_RULINGS.md`, 2026-09-29), coordinator correction
## same day: "the shape is not 1-to-1 yet" -- the first pass matched the
## mockup's *ratios* onto this shell's OLD, independently-tuned 60 px ring,
## which put every label straddling its slot's own border, half in and half
## out. These eight are now the mockup's own ABSOLUTE desktop figures
## (`design/map-context-2026-09-25/Main.dc.html`'s `RR=92, SLOT=60, DEAD=22,
## SUBR=84, SSLOT=56, SUBDEAD=32`), not tuned figures scaled by a ratio:
const RING_RADIUS := 92.0      ## mockup `RR`.
const SLOT_SIZE := 60.0        ## mockup `SLOT`.
const DEAD_ZONE := 22.0        ## mockup `DEAD`.
const SUB_RING_RADIUS := 84.0  ## mockup `SUBR`.
const SUB_DEAD_ZONE := 32.0    ## mockup `SUBDEAD`.
## `RD=RR+SLOT/2+12` and `SUBR+SSLOT/2+10` -- the disc's extra reach past the
## slot ring, the mockup's own absolute padding (not a proportion of slot
## size), touch-scaled alongside everything else in `_apply_touch_scale()`.
const DISC_MARGIN := 12.0
## The sub-ring disc's extra reach past its slot ring, from the mockup's `SUBR+SSLOT/2+10` (see the
## comment above); touch-scaled into `_r_sub_disc_margin`.
const SUB_DISC_MARGIN := 10.0
## `SSLOT/SLOT = 56/60`: with `SLOT_SIZE` now the mockup's own 60, this ratio
## resolves to the mockup's own literal 56 rather than approximating it.
const SUB_SLOT_RATIO := 56.0 / 60.0
## The centre CLOSE/BACK button's own diameter against the main slot's --
## `46/60`, which for the same reason now resolves to a literal 46.
const CENTRE_RATIO := 46.0 / 60.0

## The ring DOCK (`CUSTOM_QUICK_BAR_SCOPE.md` s.4.1, milestone CQ-4, Ruling DH Q3): up to four user
## chips in a row directly beneath the caption pill, outside the eight slots. These are the desktop
## figures; `_apply_touch_scale()` multiplies them by `DccTheme.TOUCH_SCALE` and floors both axes at
## `DOCK_TOUCH_FLOOR`. A chip is wider than tall so a short command title can wrap onto two lines
## (the scope's 40 px square cannot hold one); 40 px tall is the scope's figure, 56 px wide is a
## labelled judgement: four chips plus gaps (242 px) stay inside the disc's own width (268 px), so the
## dock never makes the ring wider than it already is. The conditional Assistant chip (CQ-7) is a fifth
## chip when shown: five chips and gaps are 304 px, 36 px wider than the disc. That is deliberate and
## measured on screen by `_quickbar_assistant_probe.gd` (the clamp reads `_dock_width()`, so it still
## fits at every edge); the alternative, a narrower chip for everything, would have changed the CQ-4
## geometry that is already verified.
const DOCK_CHIP_W := 56.0
const DOCK_CHIP_H := 40.0
const DOCK_GAP := 6.0
## `DESIGN_HANDOFF.md`: "Minimum touch target 44 px ... no exceptions".
const DOCK_TOUCH_FLOOR := 44.0
## Vertical gap between the caption pill's bottom edge and the dock's top edge. A labelled judgement
## (the pill's own padding is 5 px).
const DOCK_PILL_GAP := 4.0

## CM-3 residual (`OUTSTANDING_WORK.md`; `MAP_CONTEXT_SCOPE.md` §5.2 rule 2):
## the mockup's own `ringHover`'s `dist>RR+18` gate -- how far PAST the main
## ring's own radius a continuous RMB drag must travel, while hovering a
## `▸` slot, before that slot's sub-ring opens in place. Named and
## touch-scaled the same way every other mockup-absolute figure above is;
## kept as its own constant rather than folded into `DISC_MARGIN` (12.0)
## because the mockup uses two different literals for two different
## purposes -- the disc's own drawn edge vs. this gesture threshold -- and
## a future re-tune of one must not silently move the other.
const PUSH_PAST := 18.0

## CM-4 residual (`OUTSTANDING_WORK.md`; `MAP_CONTEXT_SCOPE.md` §7.2): "Ring
## radius scales with the finger. Slots sit at 96 dp on touch (60 px on
## desktop) ... 96 dp leaves generous gaps and a clear angular sector per
## slot." This file drew the desktop `RING_RADIUS`/`SLOT_SIZE` unconditionally
## until now -- a tablet's touch-hold gesture (`MAP_CONTEXT_SCOPE.md` §7.1)
## opened the same 60 px ring a mouse gets, well under the 96 dp target.
##
## `_r*` below are what every drawing/hit-test function in this file actually
## reads; the `RING_RADIUS`/etc. consts above stay as the authored desktop
## figures the scaling is computed FROM, exactly as `DccTheme.TABLET`'s own
## header keeps a figure's desktop key beside its touch value rather than
## overwriting it.
var _r_ring := RING_RADIUS
## Live slot size: `SLOT_SIZE`, multiplied by `DccTheme.TOUCH_SCALE` in `_apply_touch_scale`; read
## by `_disc_radius` and the drawing and hit-test code.
var _r_slot := SLOT_SIZE
## Live dead-zone radius: `DEAD_ZONE` times `DccTheme.TOUCH_SCALE` (set in `_apply_touch_scale`),
## reported by `debug_state`.
var _r_dead := DEAD_ZONE
## Live sub-ring radius: `SUB_RING_RADIUS` times `DccTheme.TOUCH_SCALE`; read by `_sub_disc_radius`
## and reported by `debug_state`.
var _r_sub_ring := SUB_RING_RADIUS
## Live sub-ring dead-zone radius: `SUB_DEAD_ZONE` times `DccTheme.TOUCH_SCALE`; reported by
## `debug_state`.
var _r_sub_dead := SUB_DEAD_ZONE
## Live disc margin: `DISC_MARGIN` times `DccTheme.TOUCH_SCALE`; read by `_disc_radius`.
var _r_disc_margin := DISC_MARGIN
## Live sub-ring disc margin: `SUB_DISC_MARGIN` times `DccTheme.TOUCH_SCALE`; read by
## `_sub_disc_radius`.
var _r_sub_disc_margin := SUB_DISC_MARGIN
## Live push-past threshold: `PUSH_PAST` times `DccTheme.TOUCH_SCALE`; reported by `debug_state`.
var _r_push_past := PUSH_PAST
## Live dock chip width, height and gap (the `DOCK_*` figures, touch-scaled and floored).
var _r_dock_w := DOCK_CHIP_W
var _r_dock_h := DOCK_CHIP_H
var _r_dock_gap := DOCK_GAP

## Still-hold threshold in milliseconds (MAP_CONTEXT_SCOPE.md §6, quoted
## beside it) after which an unmoved RMB press opens ring and card together.
const HOLD_MS := 300     ## §6: "hold >= 300ms, still" -> ring + card
## §6: "the ring appears only if the button is still held after 150ms" --
## measured from the moment travel first exceeded the click slop, not from
## the press. `map_overlay.gd`'s own `_RMB_CLICK_SLOP` is the same 8 px; kept
## as a separate named constant here (not shared code) because the two files
## must not import from each other for a single number, but must not drift
## either -- if one changes, grep the other's citation of this comment.
## Delay in milliseconds between first exceeding [`CLICK_SLOP`] and the ring
## being drawn (§6's no-wait flick: a quick flick selects before it is seen).
const FLICK_MS := 150
## Pointer travel in pixels below which a press is still a click, not a drag;
## same figure as `map_overlay.gd`'s `_RMB_CLICK_SLOP` (see above on why it is
## duplicated). Judgement carried from the spec, not measured here.
const CLICK_SLOP := 8.0

## The eight compass directions, clockwise from north; also the key set of the
## `slots` dictionary a provider hands to [`arm`] and the iteration order of
## every hover/draw loop. Must stay in step with [`ANGLES`].
const DIRS := ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
## Screen-space angle in degrees (0 = east, y down, so north is -90) of each
## [`DIRS`] slot's centre on the ring. Eight slots 45 degrees apart; the hover
## test picks the nearest of these, so sectors are 45 degrees wide.
const ANGLES := {
	"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0,
	"S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0,
}

## The owning `DccApp` (untyped, like sibling shell nodes). Used only for the
## context-surface pause/resume pair in [`_set_active`]; null-guarded there.
var app

## `_armed`: a press has been recorded and the gesture has not yet declared
## itself a drag or a hold (RMB only -- Q skips straight to `_active`).
## `_active`: the ring genuinely exists as a gesture in progress or a sticky
## open surface, whether or not it is currently *drawn* (`_visible`).
var _armed := false
## True while the ring exists as a gesture in progress or sticky open surface (see the comment
## above); `is_open` returns it and `_set_active` changes it.
var _active := false
## `_visible`: whether the ring is currently drawn (a flick may be `_active`
## but not yet drawn, see [`FLICK_MS`]). Shadows nothing: Control's own flag is
## `visible`, which is kept equal to this whenever either is written.
var _visible := false
## True once the gesture has ended and the ring stays open awaiting a click
## (a tap, or a still release); [`release`] reads it to tell first release from
## a later click.
var _sticky := false
## True for the Q-key path: visible at once, no hold/flick timing.
var _q := false
## The eight-entry compass the provider built: `dir -> {glyph, label,
## callable | children, enabled, armed, shortcut, reason}`. Read-only here.
var _slots: Dictionary = {}
## Ring centre in this control's space, already clamped on-screen.
var _centre := Vector2.ZERO
## Where and when (msec ticks) the press was recorded; the origin of slop and
## hold measurements.
var _press_pos := Vector2.ZERO
## `Time.get_ticks_msec()` at the press, set in `arm`; `_process` measures `HOLD_MS` from it.
var _press_ms := 0
## Tick at which travel first exceeded [`CLICK_SLOP`], or -1 if it has not.
var _moved_ms := -1
## Hovered direction key from [`DIRS`], or `""` for the dead zone / none.
var _hover := ""
var _sub: Dictionary = {}   ## {} closed; else {"dir","items","centre","hover"}
## The ring dock's items (`QuickBarRing.dock_items`: `{title, label, shortcut, enabled, reason,
## callable}`), set by [`arm`] and EMPTY unless the user docked something -- an empty dock draws
## nothing, reserves nothing and changes no hit test (the pixel-identity bar, scope s.7 CQ-4).
var _dock: Array = []
## Hovered dock chip index, or -1. Set only while the ring is DRAWN: a no-wait flick never selects a
## dock chip (the dock is not part of the compass, scope s.4.1).
var _dock_hover := -1


## One-time wiring: stores the app, makes this node a top-level, input-transparent
## overlay (it must never take mouse events; `map_overlay.gd` owns them), hides
## it and computes the touch geometry. Must run before the first [`arm`].
func setup(app_ref) -> void:
	app = app_ref
	## Drawn above the whole shell in its own space, independent of whatever
	## container `app.add_child(ring)` happened to land it under -- the same
	## reason `context_card.gd`'s popup converts through
	## `get_global_transform_with_canvas()` before it ever reaches this file.
	top_level = true
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	set_process(false)
	_apply_touch_scale()


## §7.2's touch figures, computed once -- `DccShell._compute_layout_mode()`'s
## own header is why: "Phone-vs-tablet is decided once ... a device's own form
## factor is not something that changes at runtime," so there is no resize
## hook to re-run this from, the same way `_phone_scale` itself is set once at
## boot and never revisited.
##
## `DccTheme.TOUCH_SCALE` (1.53) is this shell's one documented general-
## purpose touch multiplier -- reused here rather than inventing a ring-only
## factor, per its own header: "the fallback for any figure the table [below
## it] does not name." `RING_RADIUS` is now the mockup's own 92 px (Ruling BC's
## coordinator correction, above), so plain `TOUCH_SCALE` already clears the
## 96 dp floor §7.2 names (92 x 1.53 = 140.76) -- the `maxf(96.0, ...)` stays
## as a floor rather than being removed, the same "scale, then never let the
## result fall under the figure the spec states" shape `DccShell._ptap()`
## already uses, in case a future re-tune of `RING_RADIUS` ever put it back
## under 96 again. Every other figure here (slot size, dead zones, the
## sub-ring) has no such named target, so plain `TOUCH_SCALE` is all they take.
func _apply_touch_scale() -> void:
	if not DccTheme.is_tablet():
		return
	_r_ring = maxf(96.0, RING_RADIUS * DccTheme.TOUCH_SCALE)
	_r_slot = SLOT_SIZE * DccTheme.TOUCH_SCALE
	_r_dead = DEAD_ZONE * DccTheme.TOUCH_SCALE
	_r_sub_ring = SUB_RING_RADIUS * DccTheme.TOUCH_SCALE
	_r_sub_dead = SUB_DEAD_ZONE * DccTheme.TOUCH_SCALE
	_r_disc_margin = DISC_MARGIN * DccTheme.TOUCH_SCALE
	_r_sub_disc_margin = SUB_DISC_MARGIN * DccTheme.TOUCH_SCALE
	_r_push_past = PUSH_PAST * DccTheme.TOUCH_SCALE
	_r_dock_w = maxf(DOCK_TOUCH_FLOOR, DOCK_CHIP_W * DccTheme.TOUCH_SCALE)
	_r_dock_h = maxf(DOCK_TOUCH_FLOOR, DOCK_CHIP_H * DccTheme.TOUCH_SCALE)
	_r_dock_gap = DOCK_GAP * DccTheme.TOUCH_SCALE


## The disc's own radius (mockup `RD=RR+SLOT/2+12`) -- the panel-toned ground
## drawn behind the main ring's slots, live and touch-scaled.
func _disc_radius() -> float:
	return _r_ring + _r_slot * 0.5 + _r_disc_margin


## Diameter of one sub-ring slot (live, touch-scaled).
func _sub_slot_size() -> float:
	return _r_slot * SUB_SLOT_RATIO


## Radius of the sub-ring's backing disc (mockup `SUBR+SSLOT/2+10`).
func _sub_disc_radius() -> float:
	return _r_sub_ring + _sub_slot_size() * 0.5 + _r_sub_disc_margin


## Diameter of the centre CLOSE/BACK button (live, touch-scaled).
func _centre_size() -> float:
	return _r_slot * CENTRE_RATIO


## True while the ring is a live gesture or sticky surface, drawn or not.
## Callers use it to decide whether pointer events are theirs or the ring's.
func is_open() -> bool:
	return _active


## CM-2 residual (`MAP_CONTEXT_SCOPE.md` §9.4): "The world pauses under an
## open surface" -- `_active`, not `visible`/`_visible`, is the ring's own
## "genuinely open" state (this class's own doc comment above `_active`:
## "whether or not it is currently *drawn*"), so this is the one place every
## `_active` write goes through, guarded on an actual change so `arm()`'s own
## reset-to-false-then-true (Q path) or a `_process()` transition mid-gesture
## never double-counts against `app.gd`'s ref-counted depth.
func _set_active(v: bool) -> void:
	if _active == v:
		return
	_active = v
	if app == null:
		return
	if v:
		app.pause_for_context_surface()
	else:
		app.resume_from_context_surface()


## Probe-only introspection (`_ctxring_probe.gd`) -- everything a test needs
## to assert through the real state machine rather than by re-deriving it.
func debug_state() -> Dictionary:
	return {
		"active": _active, "visible": _visible, "sticky": _sticky, "q": _q,
		"centre": _centre, "hover": _hover,
		"sub_open": not _sub.is_empty(),
		"sub_centre": _sub.get("centre", Vector2.ZERO),
		"sub_hover": int(_sub.get("hover", -1)),
		"sub_count": (_sub.get("items", []) as Array).size(),
		## CM-4 residual: the LIVE, possibly touch-scaled geometry
		## (`_apply_touch_scale()`) -- so a probe measures what this node
		## actually drew rather than re-declaring the desktop constants and
		## asserting them against themselves (`MISTAKES.md`'s preflight rule).
		"radius": _r_ring, "slot": _r_slot, "dead": _r_dead,
		"sub_radius": _r_sub_ring, "sub_dead": _r_sub_dead,
		## Ruling BC geometry, live -- the disc, the sub-ring's own slot size
		## and the centre button, all read off the SAME touch-scaled state the
		## rest of this dictionary already reports (`MISTAKES.md`'s "never
		## assert a constant against itself").
		"disc_radius": _disc_radius(), "sub_disc_radius": _sub_disc_radius(),
		"sub_slot_size": _sub_slot_size(), "centre_size": _centre_size(),
		## CM-3 residual (continuous nested flick): the live push-past
		## threshold a probe presses out to, rather than re-declaring
		## `PUSH_PAST` and asserting it against itself.
		"push_past": _r_push_past,
		## CQ-4: the dock's LIVE geometry and hover, and the sub-ring's drawn slot centres, so
		## a probe measures what this node drew (pairwise distances, 44 px floors, edge clipping)
		## instead of re-deriving it -- `MISTAKES.md`: never assert a constant against itself.
		"dock_count": _dock.size(), "dock_hover": _dock_hover,
		"dock_rects": _dock_rects(), "dock_half_w": _dock_width() * 0.5,
		"dock_chip": Vector2(_r_dock_w, _r_dock_h),
		"sub_slot_centres": _sub_slot_centres(),
		"caption": _caption_text(),
	}


## Fills the whole viewport so `_centre`/`_clamp_centre()` can work in plain
## screen pixels. Re-run on every `arm()` rather than once, so a window
## resize between two ring opens is never stale.
func _fit_to_viewport() -> void:
	var r := get_viewport().get_visible_rect()
	position = Vector2.ZERO
	size = r.size


## Begin a gesture at `pos` (already converted to this control's space by
## `context_broker.gd`). `slots` is the 8-entry compass `ring_collect()`
## built; `q` is true only for the Q-key path, which is visible immediately
## and carries no hold/flick timing at all (§6's row for Q has none). `dock` is the ring
## dock's items (CQ-4, `QuickBarRing.dock_items`); `[]`, the default and the shipped state,
## leaves every pixel and hit test exactly as they were before the dock existed.
func arm(pos: Vector2, slots: Dictionary, q: bool, dock: Array = []) -> void:
	_fit_to_viewport()
	_slots = slots
	## The dock is set BEFORE `_clamp_centre` reads it, and reset on every arm so a dock from the
	## previous gesture can never outlive the setting that produced it.
	_dock = dock
	_dock_hover = -1
	_centre = _clamp_centre(pos)
	_press_pos = pos
	_press_ms = Time.get_ticks_msec()
	_moved_ms = -1
	_hover = ""
	_sub = {}
	_q = q
	_sticky = false
	if q:
		_armed = false
		_set_active(true)
		_visible = true
		visible = true
		set_process(false)
	else:
		_armed = true
		_set_active(false)
		_visible = false
		visible = false
		set_process(true)
	queue_redraw()


## Every pointer sample while the gesture is outstanding, RMB-held or
## Q-aiming alike (`context_broker.gd` forwards every `map_overlay.gd` motion
## event here regardless of button state, and this function no-ops unless a
## gesture is actually in progress).
func pointer(pos: Vector2) -> void:
	if _active:
		if not _sub.is_empty():
			_update_sub_hover(pos)
			queue_redraw()
			return
		_update_hover(pos)
		if _moved_ms < 0 and not _q and pos.distance_to(_press_pos) > CLICK_SLOP:
			_moved_ms = Time.get_ticks_msec()
		queue_redraw()
		return
	if _armed and pos.distance_to(_press_pos) > CLICK_SLOP:
		## Travel exceeded the slop before the still-hold timer fired: this is
		## a drag, and the ring itself now exists (though not necessarily
		## drawn yet -- see `_process()`'s flick-delay branch).
		_armed = false
		_set_active(true)
		_moved_ms = Time.get_ticks_msec()
		_update_hover(pos)
		queue_redraw()


## Timer driver, enabled only while a gesture is outstanding ([`arm`] turns it
## on, every settle path turns it off): fires the still-hold ([`HOLD_MS`]) and
## reveals a flicked ring ([`FLICK_MS`]). Must never run for the Q path or an
## idle ring -- it switches itself off in both cases.
func _process(_dt: float) -> void:
	if _q:
		set_process(false)
		return
	var now := Time.get_ticks_msec()
	if _armed and not _active and now - _press_ms >= HOLD_MS:
		## §6's third row: a still hold opens the ring AND the card. NOT
		## `_sticky = true` here -- that is `release()`'s own decision (a
		## release with no hover goes sticky; `release()` needs to see
		## `_sticky` still false to know this is the gesture's FIRST release
		## rather than a later click into an already-sticky ring).
		_armed = false
		_set_active(true)
		_visible = true
		visible = true
		hold_fired.emit()
		queue_redraw()
		return
	if _active and _moved_ms >= 0 and not _visible and now - _moved_ms >= FLICK_MS:
		_visible = true
		visible = true
		queue_redraw()
	if not _armed and not _active:
		set_process(false)


## The button-up (or Q key-up via `q_release()`). Returns the chosen leaf's
## `Callable`, or an invalid one when nothing was chosen -- `context_broker.
## gd` calls it if valid. Also the sticky click's own decision function
## (`click_at()` below re-uses it after re-aiming at the click position), so
## there is exactly one place that decides what a release means.
func release() -> Callable:
	_armed = false
	if not _active:
		set_process(false)
		return Callable()
	var already_sticky := _sticky
	if not _sub.is_empty():
		var hi := int(_sub.get("hover", -1))
		if hi >= 0:
			var items: Array = _sub["items"]
			var cb: Callable = (items[hi] as Dictionary).get("callable", Callable())
			close()
			return cb
		if already_sticky:
			## A sticky sub-ring, clicked outside every one of its own
			## slots: cancel rather than re-arm sticky a second time.
			close()
		else:
			_sticky = true
			_visible = true
			visible = true
			set_process(false)
		return Callable()
	if _dock_hover >= 0 and _dock_hover < _dock.size():
		## A dock chip chosen on release. `_dock_hover` is only ever set while the ring is
		## drawn (see `_update_hover`), so a no-wait flick, which releases before the ring is
		## visible, can never reach this branch. A disabled chip closes the ring and runs
		## nothing, exactly as a disabled slot does.
		var drow: Dictionary = _dock[_dock_hover]
		var dcb: Callable = drow.get("callable", Callable())
		close()
		return dcb if bool(drow.get("enabled", false)) else Callable()
	if _hover != "":
		var row: Dictionary = _slots.get(_hover, {})
		if row.is_empty() or not bool(row.get("enabled", true)):
			close()
			return Callable()
		if row.has("children"):
			_open_sub(_hover, row["children"])
			return Callable()
		var cb: Callable = row.get("callable", Callable())
		close()
		return cb
	## The dead zone, or outside every sector.
	if already_sticky:
		close()
	else:
		_sticky = true
		_visible = true
		visible = true
		set_process(false)
	return Callable()


## Q's key-up. A no-op once sticky (§6: "a tap leaves it open, sticky" -- a
## later key-up must not re-resolve an already-sticky ring).
func q_release() -> Callable:
	if not _active or not _q or _sticky:
		return Callable()
	return release()


## The sticky click path (§6/§7.1's "tap outside to dismiss", the design
## canvas's own per-slot click handler): a left click while the ring is open
## and sticky selects whatever it lands on. Re-aims at the click point first,
## since a sticky ring may not have had its hover updated by a plain click
## (no drag motion precedes it).
func click_at(pos: Vector2) -> Callable:
	if not _active:
		return Callable()
	pointer(pos)
	return release()


## Ends the gesture from any state: resets every flag, drops the sub-ring, hides
## the node and releases the world-pause via [`_set_active`]. Idempotent; the
## one exit every cancel and every completed pick goes through.
func close() -> void:
	_armed = false
	_set_active(false)
	_visible = false
	_sticky = false
	_sub = {}
	_dock_hover = -1
	visible = false
	set_process(false)
	queue_redraw()


## Opens `dir`'s sub-ring in place: its centre sits one ring radius out along
## that direction (clamped on-screen), and it replaces the main ring's hover
## handling until closed. `items` are leaf rows (`label`, optional `glyph`,
## `callable`); the sub-ring has no further nesting (§5.2 rule 2).
func _open_sub(dir: String, items: Array) -> void:
	var ang := deg_to_rad(ANGLES[dir])
	var sub_centre: Vector2 = _centre + Vector2(cos(ang), sin(ang)) * _r_ring
	## The dock is hidden while a sub-ring is up, so the sub-ring is clamped without the dock's extra
	## room (otherwise it would jump by the dock's height when the user has docked something).
	_dock_hover = -1
	_sub = {"dir": dir, "items": items, "centre": _clamp_centre(sub_centre, false), "hover": -1}
	_visible = true
	visible = true
	set_process(false)
	queue_redraw()


## Recomputes the hovered main slot from `pos`: dead zone, else the nearest
## [`ANGLES`] direction by angular distance; dragging past
## `_r_ring + _r_push_past` over an enabled `▸` slot opens its sub-ring
## instead. Emits [`hover_entered`] only on a change to a real slot.
func _update_hover(pos: Vector2) -> void:
	var prev := _hover
	var prev_dock := _dock_hover
	_dock_hover = -1
	var dist := pos.distance_to(_centre)
	## CQ-4 (scope s.4.1): the dock is hit-tested RECTANGLE FIRST, and only when the pointer is
	## outside the disc and the ring is DRAWN. Inside the disc the 45-degree sector rule below is
	## untouched; while the ring is not drawn (a flick inside `FLICK_MS`) the dock does not exist,
	## so a flick can never select a chip. Over the dock band the compass is neutral (`_hover` is
	## empty): without that, the gap between two chips would count as the SW or SE slot, whose
	## push-past would open a sub-ring under the pointer.
	if _visible and not _dock.is_empty() and dist > _disc_radius() and _in_dock_band(pos):
		_hover = ""
		_dock_hover = _dock_index_at(pos)
		if _dock_hover >= 0 and _dock_hover != prev_dock:
			hover_entered.emit()
		return
	if dist <= _r_dead:
		_hover = ""
		return
	var ang := rad_to_deg(atan2(pos.y - _centre.y, pos.x - _centre.x))
	var best := ""
	var best_d := 361.0
	for dir in DIRS:
		var d: float = absf(wrapf(ang - float(ANGLES[dir]), -180.0, 180.0))
		if d < best_d:
			best_d = d
			best = dir
	## CM-3 residual, continuous nested flick (`MAP_CONTEXT_SCOPE.md` §5.2
	## rule 2; mockup `ringHover`'s own `if(hv&&dist>RR+18){... if(def.
	## children){... sub:this.subFor(r,hv) ...}}`): dragging PAST a `▸`
	## slot -- not merely onto it -- opens its sub-ring in place, re-centred,
	## in the SAME RMB-down gesture, rather than waiting for a release and a
	## second click. Only a slot that is both present and `enabled` opens
	## this way: the mockup's own `ringHover` has no such guard, but every
	## OTHER path in this file that can open a sub-ring (`release()` below)
	## closes on a disabled slot instead, per its own `slotDisabled` check --
	## letting a drag alone bypass that would be a second, inconsistent rule
	## for the exact same slot.
	## CQ-4: while a dock is drawn, a drag from the centre toward it must not open the SW or SE
	## sub-ring on the way (the outer chips lie in those sectors). Inside the funnel between the
	## centre and the dock the push-past gate is suppressed; the hover still follows the compass and a
	## release there behaves as before. With no dock `_in_dock_funnel` is false and nothing changes.
	if best != "" and dist > _r_ring + _r_push_past and not (_visible and _in_dock_funnel(pos)):
		var row: Dictionary = _slots.get(best, {})
		if not row.is_empty() and row.has("children") and bool(row.get("enabled", true)):
			_hover = best
			_open_sub(best, row["children"])
			return
	_hover = best
	if _hover != "" and _hover != prev:
		hover_entered.emit()


## The sub-ring counterpart of [`_update_hover`]: slots are spread evenly from
## north (`360/n` apart, matching [`_draw_sub_ring`]), so the nearest-angle
## rule must use the same layout or hover and drawing disagree.
func _update_sub_hover(pos: Vector2) -> void:
	var c: Vector2 = _sub["centre"]
	var items: Array = _sub["items"]
	var n := items.size()
	var prev := int(_sub.get("hover", -1))
	if n == 0 or pos.distance_to(c) <= _r_sub_dead:
		_sub["hover"] = -1
		return
	var ang := rad_to_deg(atan2(pos.y - c.y, pos.x - c.x))
	var best := 0
	var best_d := 361.0
	for i in n:
		var a := -90.0 + 360.0 * float(i) / float(n)
		var d := absf(wrapf(ang - a, -180.0, 180.0))
		if d < best_d:
			best_d = d
			best = i
	_sub["hover"] = best
	if best != prev:
		hover_entered.emit()


## Keeps the ring's own outer edge, plus room for the caption band under it,
## inside the viewport at all four screen edges (§5's "the ring stays on
## screen at the four edges" -- the probe's own check).
func _clamp_centre(pos: Vector2, with_dock: bool = true) -> Vector2:
	## The disc's own radius plus room for the caption pill below it (mockup:
	## `capT: cy+RD+6`, then the pill's own ~20 px height/padding) -- was a
	## bare `_r_ring + _r_slot*0.5 + 40`, which the disc (`_disc_radius()`,
	## `RD` in the mockup) already exceeds by the touch-scaled margin alone.
	var pad := _disc_radius() + 30.0
	var sz := size
	if sz.x <= 0.0 or sz.y <= 0.0:
		return pos
	## CQ-4: with a dock the ring needs room for it, so it slides inward at an edge until the dock is
	## on screen (scope s.4.1). Side and bottom only: the dock hangs below the disc and is no wider
	## than it. With no dock `pad_x == pad_b == pad` and this is the function it was.
	var pad_x := pad
	var pad_b := pad
	if with_dock and not _dock.is_empty():
		pad_x = maxf(pad, _dock_width() * 0.5 + 6.0)
		pad_b = maxf(pad, _dock_bottom_offset() + 6.0)
	return Vector2(
		clampf(pos.x, pad_x, maxf(pad_x, sz.x - pad_x)),
		clampf(pos.y, pad, maxf(pad, sz.y - pad_b)))


# -- The dock (CQ-4) -----------------------------------------------------------------------

## Total width of the dock's chip row.
func _dock_width() -> float:
	if _dock.is_empty():
		return 0.0
	return _r_dock_w * float(_dock.size()) + _r_dock_gap * float(_dock.size() - 1)


## Height of the caption pill (`_draw_caption` pads the text by 5 px top and bottom). Computed from
## the font's line height, which does not depend on the caption text.
func _pill_height() -> float:
	return DccTheme.mono().get_string_size("Ag", HORIZONTAL_ALIGNMENT_LEFT, -1, DccTheme.FS_SMALL).y + 10.0


## Offset of the dock's TOP edge below the ring centre: past the disc, the pill (centred 6 px below
## the disc edge, so it reaches half its height further) and a small gap.
func _dock_top_offset() -> float:
	return _disc_radius() + 6.0 + _pill_height() * 0.5 + DOCK_PILL_GAP


## Offset of the dock's BOTTOM edge below the ring centre.
func _dock_bottom_offset() -> float:
	return _dock_top_offset() + _r_dock_h


## The dock chips' rectangles in this control's space, left to right; `[]` with no dock or while a
## sub-ring is open (the dock is not drawn then).
func _dock_rects() -> Array:
	var out: Array = []
	if _dock.is_empty() or not _sub.is_empty():
		return out
	var x0 := _centre.x - _dock_width() * 0.5
	var y0 := _centre.y + _dock_top_offset()
	for i in _dock.size():
		out.append(Rect2(x0 + float(i) * (_r_dock_w + _r_dock_gap), y0, _r_dock_w, _r_dock_h))
	return out


## Whether `pos` lies in the dock's row, gaps between chips included (see `_update_hover`).
func _in_dock_band(pos: Vector2) -> bool:
	var rs := _dock_rects()
	if rs.is_empty():
		return false
	var first: Rect2 = rs[0]
	var last: Rect2 = rs[rs.size() - 1]
	return Rect2(first.position, Vector2(last.end.x - first.position.x, first.size.y)).has_point(pos)


## Whether `pos` lies in the triangle from the ring centre to the dock row's lower corners: the
## straight-line approach to ANY chip, outer ones included (see `_update_hover`). False with no dock.
func _in_dock_funnel(pos: Vector2) -> bool:
	var rs := _dock_rects()
	if rs.is_empty():
		return false
	var dy := pos.y - _centre.y
	var base_dy: float = (rs[0] as Rect2).end.y - _centre.y
	if dy <= 0.0 or base_dy <= 0.0:
		return false
	return absf(pos.x - _centre.x) <= _dock_width() * 0.5 * dy / base_dy


## The dock chip under `pos`, else -1 (a gap).
func _dock_index_at(pos: Vector2) -> int:
	var rs := _dock_rects()
	for i in rs.size():
		if (rs[i] as Rect2).has_point(pos):
			return i
	return -1


## Centre of sub-ring slot `i` of `n` around `centre`: evenly spread from north, `360/n` apart. The
## one place that formula lives; drawing and the probe's pairwise-distance bar both read it.
func _sub_slot_pos(centre: Vector2, i: int, n: int) -> Vector2:
	var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, n)))
	return centre + Vector2(cos(ang), sin(ang)) * _r_sub_ring


## Centres of the open sub-ring's slots (`[]` when none is open), for `debug_state()`.
func _sub_slot_centres() -> Array:
	var out: Array = []
	if _sub.is_empty():
		return out
	var n: int = (_sub["items"] as Array).size()
	for i in n:
		out.append(_sub_slot_pos(_sub["centre"], i, n))
	return out


# -- Drawing ------------------------------------------------------------------

## Draws the whole ring in the mockup's DOM order (listed below). Draws nothing
## unless the gesture is both active and visible; never mutates state.
func _draw() -> void:
	if not _active or not _visible:
		return
	var sub_open := not _sub.is_empty()
	## DOM order in the mockup, reproduced literally: scrim, then the main
	## disc, then the hover wedge (so slots draw OVER it), then the slots,
	## then -- if open -- the sub-disc, its own wedge and its slots, then the
	## centre button, then the caption pill.
	_draw_scrim()
	_draw_disc(_centre, _disc_radius(), 0.44 if sub_open else 0.88, DccTheme.c("border"), 1.0)
	if not sub_open and _hover != "":
		_draw_wedge(_centre, _r_dead, _disc_radius() - 4.0, float(ANGLES[_hover]), 22.5)
	_draw_ring(_centre, _r_ring, _r_dead, _slots, _hover, not sub_open,
		String(_sub.get("dir", "")) if sub_open else "")
	if sub_open:
		var sc: Vector2 = _sub["centre"]
		var accent_border := DccTheme.c("accent")
		_draw_disc(sc, _sub_disc_radius(), 0.94, accent_border, 2.0)
		var hi := int(_sub.get("hover", -1))
		if hi >= 0:
			var n: int = (_sub["items"] as Array).size()
			var ang := -90.0 + 360.0 * float(hi) / float(maxi(1, n))
			_draw_wedge(sc, _r_sub_dead, _sub_disc_radius() - 4.0, ang, 180.0 / float(maxi(1, n)))
		_draw_sub_ring(sc, _sub["items"], hi)
	## CQ-4: the dock is drawn only when the user docked something and no sub-ring is up. With the
	## shipped empty dock this call returns at its first line and draws nothing.
	_draw_dock()
	_draw_centre_button()
	_draw_caption()


## The 30%-darkening scrim behind an open ring (mockup: `rgba(0,0,0,.3)` over
## the whole map). Kept as plain black rather than a `DccTheme` surface token
## on purpose: Ruling BC's "colours do not carry over" bars the mockup's
## SURFACE hex values (panel/slot/border greys, which this file takes from
## `DccTheme` throughout) but the scrim is not a surface -- it is a multiply-
## darken operator, and darkening has to mean the same thing in both themes.
## No `DccTheme` token is theme-invariant in that way: `bg` is near-black in
## `DARK` and near-white in `LIGHT`, so compositing it at any alpha would
## LIGHTEN a light theme's map instead of darkening it -- the opposite of the
## contrast relationship Ruling BC says to keep.
func _draw_scrim() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.0, 0.0, 0.0, 0.3))


## The panel-toned disc behind a ring's slots (mockup: `rgba(10,11,12,.88)`
## main / `rgba(10,11,12,.94)` sub). `alpha` is the mockup's own opacity
## figure -- `panel`'s RGB carries the theme, the alpha carries the
## refinement's stated relationship ("the disc is 88% opaque").
func _draw_disc(centre: Vector2, radius: float, alpha: float, border: Color, border_width: float) -> void:
	var fill := DccTheme.c("panel")
	fill.a = alpha
	draw_circle(centre, radius, fill)
	draw_arc(centre, radius, 0.0, TAU, 48, border, border_width, true)


## The hover wedge (mockup: an SVG annulus sector, `rgba(224,163,74,.18)` fill
## / `rgba(224,163,74,.55)` stroke) toward the hovered slot -- `accent_wash_2`
## is this shell's own "armed/about-to-act" warm tint (`DccTheme`'s own
## header names it exactly that use), at a weight (.16) already close to the
## mockup's .18.
func _draw_wedge(centre: Vector2, r0: float, r1: float, angle_deg: float, half_deg: float) -> void:
	var steps := 12
	var a0 := deg_to_rad(angle_deg - half_deg)
	var a1 := deg_to_rad(angle_deg + half_deg)
	var pts := PackedVector2Array()
	for i in range(steps + 1):
		var t: float = lerpf(a0, a1, float(i) / float(steps))
		pts.append(centre + Vector2(cos(t), sin(t)) * r1)
	for i in range(steps + 1):
		var t: float = lerpf(a1, a0, float(i) / float(steps))
		pts.append(centre + Vector2(cos(t), sin(t)) * r0)
	draw_colored_polygon(pts, DccTheme.c("accent_wash_2"))
	var border := DccTheme.c("accent")
	border.a = 0.55
	var closed := pts.duplicate()
	closed.append(pts[0])
	draw_polyline(closed, border, 1.0, true)


## The 46 px (mockup ratio) CLOSE/BACK centre button, always drawn -- at the
## main ring's own centre normally, and at the sub-ring's own centre once one
## is open, filled in the accent with a glow ring (mockup: `box-shadow: 0 0 0
## 4px rgba(224,163,74,.28)`), per Ruling BC / the refinement note ("an
## accent-filled `‹ BACK` button at the sub-ring's centre, with a glow ring").
func _draw_centre_button() -> void:
	var sub_open := not _sub.is_empty()
	var centre: Vector2 = _sub["centre"] if sub_open else _centre
	var r := _centre_size() * 0.5
	if sub_open:
		draw_circle(centre, r + 4.0, Color(DccTheme.c("accent"), 0.28))
	var bg := DccTheme.c("accent") if sub_open else DccTheme.c("sunken")
	draw_circle(centre, r, bg)
	var border := DccTheme.c("accent") if sub_open else DccTheme.c("border")
	draw_arc(centre, r, 0.0, TAU, 32, border, 2.0 if sub_open else 1.0, true)
	var ink := DccTheme.c("accent_ink") if sub_open else DccTheme.c("text_bright")
	var glyph := "‹" if sub_open else "✕"
	var label := "BACK" if sub_open else "CLOSE"
	var font := DccTheme.mono(0, true)
	var gfs: int = maxi(8, int(r * 0.62))
	var gts := font.get_string_size(glyph, HORIZONTAL_ALIGNMENT_LEFT, -1, gfs)
	draw_string(font, centre - gts * 0.5 + Vector2(0.0, gts.y * 0.32 - r * 0.26), glyph,
		HORIZONTAL_ALIGNMENT_LEFT, -1, gfs, ink)
	var lfs := DccTheme.FS_MICRO
	var lts := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, lfs)
	draw_string(font, centre + Vector2(-lts.x * 0.5, r * 0.58), label,
		HORIZONTAL_ALIGNMENT_LEFT, -1, lfs, ink)


## One ring's worth of slots, shared by the top-level compass and (via
## `_draw_sub_ring`, its own simpler pass below) the one sub-ring §5.2 rule 2
## allows. Ground is `sunken` normally and `accent` when the slot's own
## `armed` flag is set (§5.2 rule 3, MN-21's precedent: reversed
## accent-ink type on a filled accent surface -- `GUI_GAP_REGISTER.md`
## MN-21, `DccTheme`'s `accent_ink` token). A slot with no provider row
## (a domain workspace that returned nothing for this direction) or an
## explicitly `enabled: false` one is drawn at 35% opacity with its reason in
## the centre caption when hovered (§5.2 rule 1).
## The centre marker itself is `_draw_centre_button()`'s job now (Ruling BC:
## the mockup draws a real 46 px CLOSE/BACK button there, not a bare dead-zone
## disc) -- this function draws only the eight slots.
func _draw_ring(centre: Vector2, radius: float, _dead: float, slots: Dictionary,
		hover_dir: String, top_active: bool, sub_dir: String = "") -> void:
	for dir in DIRS:
		var row: Dictionary = slots.get(dir, {})
		var ang := deg_to_rad(float(ANGLES[dir]))
		var pos: Vector2 = centre + Vector2(cos(ang), sin(ang)) * radius
		var present := not row.is_empty()
		var enabled := present and bool(row.get("enabled", true))
		var armed := present and bool(row.get("armed", false))
		var hovered := top_active and present and enabled and String(dir) == hover_dir
		var alpha := 1.0 if enabled else 0.35
		## Mockup: `dim:!!r.sub&&r.sub.dir!==dir` -- every main-ring slot
		## OTHER than the one a sub-ring opened from dims to 30% while that
		## sub-ring is up. Without this, an unrelated main slot's own label
		## can sit fully visible just behind/beside the sub-ring's own disc
		## and read as a sub-slot label spilling outside its circle (leg L,
		## tablet, sub #4 -- measured `#9a9d95`, this shell's `text_ghost`,
		## bleeding through from a dimmed-in-the-mockup but undimmed-here
		## main slot at the tablet's larger touch radii).
		if sub_dir != "" and String(dir) != sub_dir:
			alpha *= 0.3
		## Refinement note: "the hover state is a warm accent tint with accent
		## text" -- a distinct state from `armed`'s solid fill, drawn as the
		## ordinary ground plus `accent_wash_2`'s own warm overlay on top,
		## exactly as `_draw_wedge()` uses the same token for the same reason.
		var ground := DccTheme.c("accent") if armed else DccTheme.c("sunken")
		draw_circle(pos, _r_slot * 0.5, Color(ground.r, ground.g, ground.b, ground.a * alpha))
		if hovered and not armed:
			draw_circle(pos, _r_slot * 0.5, DccTheme.c("accent_wash_2"))
		var border := DccTheme.c("accent") if hovered else DccTheme.c("border")
		draw_arc(pos, _r_slot * 0.5, 0.0, TAU, 28,
			Color(border.r, border.g, border.b, border.a * alpha), 2.0 if hovered else 1.0, true)
		if not present:
			continue
		## MN-21: reversed, paper-coloured ink on the filled accent surface.
		var ink := DccTheme.c("accent_ink") if armed else \
			(DccTheme.c("accent_hover") if hovered else DccTheme.c("text_bright"))
		ink.a = alpha
		## Ruling BC: "labels inside the slot" -- glyph and label both sit
		## inside the slot's own circle (mockup's flex column, gap 3px),
		## rather than the label hanging below it. `_draw_fitted_slot()`
		## wraps/shrinks so the label's own rect never crosses the circle.
		_draw_fitted_slot(pos, _r_slot * 0.5, String(row.get("glyph", "")), String(row.get("label", "")), ink)


## Draws the sub-ring's slots (evenly spaced from north, same layout as
## [`_update_sub_hover`]); the hovered slot is accent-filled. Text-only rows are
## normal -- see the note inside on glyphs.
func _draw_sub_ring(centre: Vector2, items: Array, hover_i: int) -> void:
	var n := items.size()
	var d := _sub_slot_size()
	for i in n:
		var row: Dictionary = items[i]
		var pos: Vector2 = _sub_slot_pos(centre, i, n)
		var hovered := i == hover_i
		## CQ-4: a child the USER appended (`shared`, from `QuickBarRing`) is told apart from the
		## built-ins by a second, inner ring, and is dimmed to 35 % when its command is unavailable
		## (the reason shows in the caption). A built-in child takes none of these branches, so every
		## pixel of a sub-ring without user pins is what it was.
		var shared := bool(row.get("shared", false))
		var dim := 0.35 if shared and not bool(row.get("enabled", true)) else 1.0
		var ground := DccTheme.c("accent") if hovered else DccTheme.c("sunken")
		ground.a *= dim
		draw_circle(pos, d * 0.5, ground)
		var border := DccTheme.c("accent") if hovered else DccTheme.c("border")
		border.a *= dim
		draw_arc(pos, d * 0.5, 0.0, TAU, 24, border, 2.0 if hovered else 1.0, true)
		if shared:
			draw_arc(pos, d * 0.5 - 4.0, 0.0, TAU, 24, Color(border.r, border.g, border.b, 0.6 * dim), 1.0, true)
		var ink := DccTheme.c("accent_ink") if hovered else DccTheme.c("text_bright")
		ink.a *= dim
		var glyph := String(row.get("glyph", ""))
		## The mockup's own sub-ring buttons carry text only, no icon
		## (`kidsOf()`'s children have no `glyph`) -- `_draw_fitted_slot()`
		## centres the label alone when this row has none either, and puts
		## it below the glyph (mirroring the main ring) when it does,
		## wrapping/shrinking either way so it stays inside the circle.
		_draw_fitted_slot(pos, d * 0.5, glyph, String(row.get("label", "")), ink)


## §5.2 rule 4: "the centre shows the hovered slot's full label and shortcut,
## so labels on the ring can stay short."
## The caption's text for the current state (centre-cancels hint, a hovered slot's readout, a hovered
## sub-ring child's, or -- CQ-4 -- a hovered dock chip's). Split out of `_draw_caption` so a probe can
## read what the user would read (`debug_state()["caption"]`) instead of inferring it from pixels.
func _caption_text() -> String:
	var text := "ring · centre cancels"
	if not _sub.is_empty():
		var items: Array = _sub["items"]
		var hi := int(_sub.get("hover", -1))
		var parent: Dictionary = _slots.get(String(_sub.get("dir", "")), {})
		var picked := "pick one"
		if hi >= 0 and hi < items.size():
			picked = String((items[hi] as Dictionary).get("label", ""))
			var kid: Dictionary = items[hi]
			if bool(kid.get("shared", false)):
				## A user pin: the full title (the slot shows a cut label), then why it cannot run.
				picked = String(kid.get("title", picked)) + "  (pinned)"
				if not bool(kid.get("enabled", true)):
					picked += "  — %s" % String(kid.get("reason", "unavailable"))
		text = "%s ▸ %s" % [String(parent.get("label", "")), picked]
	elif _dock_hover >= 0 and _dock_hover < _dock.size():
		## CQ-4 (scope s.4.1): hovering a dock chip puts its full title, accelerator and, when it
		## cannot run, the reason in the caption -- the same readout a slot gets.
		var drow: Dictionary = _dock[_dock_hover]
		text = "Dock · %s" % String(drow.get("title", ""))
		if String(drow.get("shortcut", "")) != "":
			text += "  (%s)" % String(drow["shortcut"])
		if not bool(drow.get("enabled", true)):
			text += "  — %s" % String(drow.get("reason", "unavailable"))
	elif _hover != "":
		var row: Dictionary = _slots.get(_hover, {})
		var shortcut := String(row.get("shortcut", ""))
		text = "%s · %s%s" % [_hover, String(row.get("label", "")), (" ▸" if row.has("children") else "")]
		if not shortcut.is_empty():
			text += "  (%s)" % shortcut
		if not bool(row.get("enabled", true)):
			text += "  — %s" % String(row.get("reason", "unavailable"))
	return text


## Draws the caption pill under whichever disc is showing; the text is `_caption_text()`.
func _draw_caption() -> void:
	var text := _caption_text()
	var below_centre: Vector2 = _sub["centre"] if not _sub.is_empty() else _centre
	var font := DccTheme.mono()
	var fs := DccTheme.FS_SMALL
	var ts := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
	## Anchored off the disc's own radius (mockup: `capT: r.cy+RD+6`), not the
	## bare slot ring -- so the pill sits just past the disc's edge whichever
	## disc is showing.
	var cap_r := _sub_disc_radius() if not _sub.is_empty() else _disc_radius()
	var cap_centre := below_centre + Vector2(0.0, cap_r + 6.0)
	var pad := Vector2(9.0, 5.0)
	var rect := Rect2(cap_centre - ts * 0.5 - pad, ts + pad * 2.0)
	draw_rect(rect, DccTheme.c("panel"))
	draw_rect(rect, DccTheme.c("line"), false, 1.0)
	draw_string(font, cap_centre - Vector2(ts.x * 0.5, -ts.y * 0.32), text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, fs, DccTheme.c("text_bright"))


## Draws the ring dock: one rounded chip per docked pin, in the sunken ground the slots use, the
## hovered chip in the accent wash, an unavailable one at 35 % opacity (disabled-with-reason, never
## removed, `MAP_CONTEXT_SCOPE.md` s.5.2 rule 1). Returns at once for an empty dock or while a
## sub-ring is open, so the shipped state draws nothing here.
func _draw_dock() -> void:
	var rects := _dock_rects()
	for i in rects.size():
		var r: Rect2 = rects[i]
		var row: Dictionary = _dock[i]
		var enabled := bool(row.get("enabled", true))
		var hovered := i == _dock_hover and enabled
		var alpha := 1.0 if enabled else 0.35
		var sb := StyleBoxFlat.new()
		var ground := DccTheme.c("sunken")
		ground.a *= alpha
		sb.bg_color = ground
		var border := DccTheme.c("accent") if hovered else DccTheme.c("border")
		border.a *= alpha
		sb.border_color = border
		sb.set_border_width_all(2 if hovered else 1)
		sb.set_corner_radius_all(int(round(6.0 * _r_dock_h / DOCK_CHIP_H)))
		sb.anti_aliasing = true
		draw_style_box(sb, r)
		if hovered:
			draw_rect(r.grow(-1.0), DccTheme.c("accent_wash_2"))
		var ink := DccTheme.c("accent_hover") if hovered else DccTheme.c("text_bright")
		ink.a *= alpha
		_draw_dock_label(r, String(row.get("label", row.get("title", ""))), ink)


## A dock chip's label: up to two lines inside the chip, wrapped at the space nearest the middle and
## shrunk (never below `MIN_LABEL_FS`) until each line fits the chip's width less a 6 px margin; a
## line that still does not fit at the smallest size is cut with an ellipsis, so no ink leaves the
## chip. The full title is in the caption when the chip is hovered.
func _draw_dock_label(r: Rect2, text: String, ink: Color) -> void:
	var font := DccTheme.mono()
	var fs := DccTheme.FS_MICRO
	var avail := r.size.x - 6.0
	var lines: Array = [text]
	while true:
		var w_one := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
		if w_one <= avail:
			lines = [text]
			break
		var parts := _split_label(text)
		if not parts.is_empty():
			var w1 := font.get_string_size(String(parts[0]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			var w2 := font.get_string_size(String(parts[1]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			if w1 <= avail and w2 <= avail:
				lines = parts
				break
		if fs <= MIN_LABEL_FS:
			lines = parts if not parts.is_empty() else [text]
			break
		fs -= 1
	for li in lines.size():
		var line := String(lines[li])
		while line.length() > 1 and font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > avail:
			line = line.substr(0, line.length() - 2) + "…"
		lines[li] = line
	var lh := float(fs) * 1.15
	var top := r.get_center().y - lh * float(lines.size()) * 0.5
	for li in lines.size():
		var line := String(lines[li])
		var ts := font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, Vector2(r.get_center().x - ts.x * 0.5, top + lh * float(li) + lh * 0.5 + ts.y * 0.32), line,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, ink)


## `name` is a `DccIcons` glyph name, or `"sym:<key>"` for one of
## `DccIcons.SYMBOLS`'s typographic characters (Undo's `↶`) -- the same
## text-vs-drawn split §12 draws everywhere else in this shell.
func _draw_glyph(name: String, centre: Vector2, px: float, color: Color) -> void:
	if name.is_empty():
		return
	if name.begins_with("sym:"):
		var ch: String = DccIcons.SYMBOLS.get(name.substr(4), "?")
		var font := DccTheme.mono()
		var fs := int(px)
		var ts := font.get_string_size(ch, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, centre - ts * 0.5 + Vector2(0.0, ts.y * 0.32), ch,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, color)
		return
	var tex := DccIcons.get_icon(name, int(px))
	if tex == null:
		return
	draw_texture_rect(tex, Rect2(centre - Vector2(px, px) * 0.5, Vector2(px, px)), false, color)


## Half the chord width of a circle of `radius` at vertical offset `dy` from
## its own centre -- `0.0` once `dy` reaches the radius (no chord left).
static func _chord_half_width(radius: float, dy: float) -> float:
	var d := absf(dy)
	if d >= radius:
		return 0.0
	return sqrt(radius * radius - d * d)


## Splits `text` into two pieces at the space or `/` nearest its own middle
## (`"Cliff / Escarpment"` -> `["Cliff", "Escarpment"]`) -- `[]` when there is
## nothing to break on.
static func _split_label(text: String) -> Array:
	var best := -1
	var best_dist := 1.0e9
	for i in text.length():
		var c := text[i]
		if c == " " or c == "/":
			var dist: float = absf(float(i) - float(text.length()) * 0.5)
			if dist < best_dist:
				best_dist = dist
				best = i
	if best < 0:
		return []
	var l1 := text.substr(0, best).strip_edges()
	var l2 := text.substr(best + 1).strip_edges()
	if l1.is_empty() or l2.is_empty():
		return []
	return [l1, l2]


## Smallest font size [`_draw_fitted_slot`] will shrink to. Judgement, per the
## trailing note: Plex Mono is unreadable below it.
const MIN_LABEL_FS := 7  ## Never shrunk past this -- Plex Mono is unreadable below it.

## Draws a glyph (optional) and its label inside a circle of `slot_radius`
## centred at `centre` -- Ruling BC's coordinator correction, 2026-09-29:
## "every label must sit fully inside its circle." Tries the mockup's own
## 9 px mono unwrapped first; if the text is too wide for the circle's own
## chord at that height, wraps onto a second line at a space or `/` near the
## middle (`"Cliff / Escarpment"`); if even a wrapped line still doesn't fit,
## shrinks the font (down to `MIN_LABEL_FS`) until it does. The probe's own
## inside-circle assert is what this function exists to satisfy, measured
## from the live drawn rect, not from this function's own arithmetic.
func _draw_fitted_slot(centre: Vector2, slot_radius: float, glyph_name: String,
		label_text: String, ink: Color) -> void:
	var glyph_present := not glyph_name.is_empty()
	var glyph_dy := -slot_radius * 0.32
	var glyph_px := slot_radius * 0.62
	if glyph_present:
		_draw_glyph(glyph_name, centre + Vector2(0.0, glyph_dy), glyph_px, ink)
	if label_text.is_empty():
		return
	var top_dy: float = (glyph_dy + glyph_px * 0.5 + slot_radius * 0.10) if glyph_present \
		else -slot_radius * 0.30
	var font := DccTheme.mono()
	var fs := DccTheme.FS_MICRO
	var lines: Array = [label_text]
	while true:
		var lh := float(fs) * 1.15
		var w_one := font.get_string_size(label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
		## The 6 px margin (not 2) leaves room for a glyph's own antialiased
		## edge and a descender past the line's nominal advance box --
		## measured: a 2 px margin let "Cliff / Escarpment" pass its own fit
		## check while still reading 1-2 px of ink just past the circle
		## (leg L, tablet sub-ring, light palette).
		if w_one <= _chord_half_width(slot_radius, top_dy + lh * 0.5) * 2.0 - 6.0:
			lines = [label_text]
			break
		var parts := _split_label(label_text)
		if not parts.is_empty():
			var w1 := font.get_string_size(String(parts[0]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			var w2 := font.get_string_size(String(parts[1]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			if w1 <= _chord_half_width(slot_radius, top_dy + lh * 0.5) * 2.0 - 6.0 and \
					w2 <= _chord_half_width(slot_radius, top_dy + lh * 1.5) * 2.0 - 6.0:
				lines = parts
				break
		if fs <= MIN_LABEL_FS:
			lines = parts if not parts.is_empty() else [label_text]
			break
		fs -= 1
	var lh := float(fs) * 1.15
	for i in lines.size():
		var dy := top_dy + lh * float(i) + lh * 0.5
		var line := String(lines[i])
		var ts := font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, centre + Vector2(-ts.x * 0.5, dy + ts.y * 0.32), line,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, ink)
