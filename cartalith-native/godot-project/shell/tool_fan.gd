extends Node
## CM-6 (`MAP_CONTEXT_SCOPE.md` §8.2, §11; Ruling AX F2): the phone's thumb
## fan -- tool switching without CM-5's noun-surface sheet. Blocked on F2,
## which the owner answered "yes, after CM-5" (`LARGE_ITEM_RULINGS.md`,
## Ruling AX, 2026-09-26); CM-5 shipped in `ba01574`.
##
## An always-shown pill (the armed-tool readout) sits above the phone's tool
## sheet. **Tap** toggles Inspect <-> the previously-armed tool. **Press and
## slide up** opens a half-fan of the SAME 8 compass slots CM-3's ring already
## computes -- `ContextBroker.ring_collect()` / `ring_domain_req()` -- reused
## rather than a second table, per this milestone's own brief and
## `MAP_CONTEXT_SCOPE.md` §9.1's "never a second implementation of arming"
## (§5.2 rule 5, written for the desktop ring but the same rule here: every
## leaf's `callable` is one a provider already built by calling the exact
## function its own dock control calls -- this file only decides which
## direction the gesture picked).
##
## ## Geometry: the newer canvas, not the scope's rounded figures
##
## `design/map-context-2026-09-25/Phone.dc.html` is later than
## `MAP_CONTEXT_SCOPE.md` §8.2 and wins where they differ (root `CLAUDE.md`'s
## own rule). Its own `FAN` table (`const FAN=[['W',180,FR],['NW',135,FR],
## ['N',90,FR],['NE',45,FR],['E',0,FR],['SW',150,FRI],['S',90,FRI],
## ['SE',30,FRI]]`, `FR=150,FRI=84,FSLOT=56`) is `OUTER`/`INNER`/`SLOT_D` below,
## verbatim. "Distance stands in for the south half" (§8.2): **S sits on the
## SAME ray as N**, just at the inner radius instead of the outer one --
## north/south become far/near because a half-fan cannot point south.
##
## ## Not `RadialRing`, and does not import it
##
## `radial_ring.gd` draws a full 360-degree compass **centred on the tap
## point** and is desktop/tablet-only by its own header. This is a half-fan
## anchored on a FIXED pill, phone-only, with different radii, a different
## dead zone and a different release grammar (a first release with no hover
## goes sticky -- exactly `RadialRing.release()`'s own rule, reimplemented
## here rather than shared because the two files' geometry does not overlap
## enough to factor out, and `radial_ring.gd`'s drawing helpers are private).
## Only the ring's PROVIDER data (`ContextBroker.ring_collect`) is shared,
## which is the part §9.1 actually forbids re-deriving.
##
## ## One correction against the canvas's own prototype
##
## The canvas's `onUp()` tests `if(fan.hover)`, which in JS is falsy for
## index `0` -- the FIRST sub-fan item could never be picked by a drag-release
## in that demo. This file tests the nearest-slot index with `>= 0` (`_hit_test()`'s `best >= 0`) the way
## `RadialRing.release()` already does for its own sub-ring (`if hi >= 0`),
## which is correct rather than a bug carried forward on purpose.
##
## ## Trigger gesture and why it cannot collide
##
## The pill is its own small `Control`, built into `_phone_content_gap`
## (the same container `DccShell._build_phone_undo_chip()` floats in, chosen
## there for the same reason: clear of both the bottom nav AND the tool
## sheet). It consumes every event that lands on it
## (`Control.MOUSE_FILTER_STOP`). `map_overlay.gd`'s own gesture classifier
## (`_TOUCH_HOLD_MS`/`_PHONE_TOUCH_HOLD_MS`, CM-5's 480 ms hold) only ever
## sees events that land on the MAP, a different `Control` entirely -- a
## press on the pill never reaches `map_overlay.gd::_gui_input()`. The two
## gestures also differ in every other dimension that could still collide if
## they somehow shared a surface: timing (260 ms here, 480 ms there), the
## slop that can pre-empt the timer (12 dp here, `map_overlay.gd`'s own
## `_RMB_CLICK_SLOP`-equivalent there), and what a plain tap does (toggle
## Inspect here; nothing -- CM-5's hold has no tap behaviour -- there). Pan
## and pinch arrive through `ViewportHost._input()`, a raw input hook that
## never consults a `Control`'s `mouse_filter` and therefore never reaches
## the pill either.

const FAN_HOLD_MS := 260   ## Canvas: `setTimeout(..., 260)`.
const FAN_SLOP := 12.0     ## Canvas: `Math.hypot(...)>12`.
const OUTER_R := 150.0     ## Canvas `FR`.
const INNER_R := 84.0      ## Canvas `FRI`.
const SLOT_D := 56.0       ## Canvas `FSLOT`.
const DEAD := 40.0         ## Canvas: `Math.hypot(x-FAX,y-FAY)<40`.
const HIT_MAX := 46.0      ## Canvas: `let best=null,bd=46`.

## `[dir, angle_deg, radius]`, the canvas's own `FAN` array, unchanged.
const FAN: Array = [
	["W", 180.0, OUTER_R], ["NW", 135.0, OUTER_R], ["N", 90.0, OUTER_R],
	["NE", 45.0, OUTER_R], ["E", 0.0, OUTER_R],
	["SW", 150.0, INNER_R], ["S", 90.0, INNER_R], ["SE", 30.0, INNER_R],
]

var app  ## DccApp (== DccShell).

## The launcher pill (inner class `_Pill`), created in `setup` and added to its host by `mount`.
var _pill: Control
## The fan's drawing and input layer (inner class `_Overlay`), created in `setup` and added as a
## top-level child of `app`.
var _overlay: Control

## True between `press` and release; `_process` uses it with `_press_ms` and `FAN_HOLD_MS` to open
## the fan on a hold.
var _pressed := false
## Whether the fan is open; `_process` closes it when the pill is hidden.
var _open := false
## True when the fan stays open as a sticky surface (its overlay then takes mouse input,
## `MOUSE_FILTER_STOP`); cleared by `press` and on close; `is_sticky` returns it and `debug_state`
## reports it.
var _sticky := false
var _press_pos := Vector2.ZERO   ## global
## `Time.get_ticks_msec()` at `press`; `_process` measures the hold against `FAN_HOLD_MS`.
var _press_ms := 0
var _last_pos := Vector2.ZERO    ## global; last known finger position while pressed
var _fan_center := Vector2.ZERO  ## global; the pill's own centre, captured at open
var _slots: Dictionary = {}      ## `ContextBroker.ring_collect()`'s 8-slot compass
var _hover := ""                 ## top-level hover dir, "" when none
var _sub: Dictionary = {}        ## {} closed; else {"dir","items","hover":int}
var _prev_tool := ""             ## for the tap-toggle's "return to" memory


## Wires the fan to the shell: builds the pill and the overlay, parents this node
## (its `_process()` timer needs the tree) and the top-level overlay, and
## refreshes the pill whenever a tool is armed. Call once, before `mount()`.
## Must never be called twice: it adds children to `app` each time.
func setup(app_ref) -> void:
	app = app_ref
	_pill = _Pill.new(self)
	_overlay = _Overlay.new(self)
	app.add_child(self)     ## `_process()`'s own 260 ms timer needs this node
		## inside the `SceneTree`; neither `_pill` nor `_overlay` carries it.
	app.add_child(_overlay)  ## Top-level, added last -- see `radial_ring.gd`'s
		## own `setup()` for the identical reasoning: drawn above everything
		## already in the tree by tree order, independent of its own transform.
	set_process(true)
	if app.has_signal("tool_armed"):
		app.tool_armed.connect(func(_id: String) -> void: _refresh_pill_text())
	_refresh_pill_text()


## Called once by `DccShell._build_phone_shell()`, after `_phone_content_gap`
## exists. Not folded into `setup()`: `_pill` needs a real parent with a real
## size before its anchors mean anything, the same ordering
## `_build_phone_undo_chip()` already relies on.
func mount(parent: Control) -> void:
	parent.add_child(_pill)


## Repaints the pill so its armed-tool label follows `app.armed_tool`. Safe
## before the pill exists (null guard); never changes any state.
func _refresh_pill_text() -> void:
	if _pill != null:
		_pill.queue_redraw()


## Canvas: `pillOn:!st.sheet` -- the pill (and so the fan it anchors) hides
## whenever CM-5's own noun-surface sheet is up, or the MORE menu covers the
## whole screen. Polled here rather than wired to a signal because neither
## `PhoneMenu` nor `DccShell` fires one for either transition today (grepped
## both -- `phone_present_peek_card()` and `_set_sheet_open()` alike are plain
## calls with no matching signal), and a per-frame boolean read is the same
## cost `DccShell._process()` already accepts for its own keyboard-height
## poll a few hundred lines up.
func _process(_dt: float) -> void:
	if _pill != null:
		_pill.visible = not _blocked_by_menu()
		if not _pill.visible and _open:
			_close()
	if _pressed and not _open and Time.get_ticks_msec() - _press_ms >= FAN_HOLD_MS:
		_open_fan()
		_update_hover_from_pos(_last_pos)


## True while the phone's MORE menu or CM-5 peek card covers the screen, which is
## when the pill (and any open fan) must hide. Duck-typed (`has_method`) because
## `PhoneMenu` is not a declared type here; a missing menu means not blocked.
## Read-only.
func _blocked_by_menu() -> bool:
	if app == null:
		return false
	var pm = app._phone_menu
	if pm == null:
		return false
	if pm.has_method("is_open") and pm.is_open():
		return true
	if pm.has_method("peek_card_is_open") and pm.peek_card_is_open():
		return true
	return false


# -- Probe-facing introspection (`_ctxfan_probe.gd`) --------------------------

## Probe seam (`_ctxfan_probe.gd`): whether the fan is currently drawn. Read-only.
func is_open() -> bool:
	return _open


## Probe seam: whether the fan has gone sticky (first release with no hover, or a
## sub-fan opened), i.e. the overlay now takes taps. Read-only.
func is_sticky() -> bool:
	return _sticky


## Probe seam: whether a sub-fan (a leaf with `children`) is showing. Read-only.
func sub_open() -> bool:
	return not _sub.is_empty()


## Probe seam: a snapshot of the gesture state (`open`, `sticky`, top-level
## `hover`, and the sub-fan's dir, hover index and item count) for the probe to
## assert on. Read-only; never feed it back into logic.
func debug_state() -> Dictionary:
	return {
		"open": _open, "sticky": _sticky, "hover": _hover,
		"sub_dir": String(_sub.get("dir", "")), "sub_hover": int(_sub.get("hover", -1)),
		"sub_count": (_sub.get("items", []) as Array).size(),
	}


## Probe seam: the pill's global rect, or an empty `Rect2` before it exists.
## Read-only.
func pill_global_rect() -> Rect2:
	return _pill.get_global_rect() if _pill != null else Rect2()


## The fan's own bounding rect at whatever radius is live right now (the
## outer arc, `OUTER_R`, is always the largest extent) -- global px. The
## probe converts to dp itself by dividing through `app.phone_scale()`,
## exactly as `_ctxphone_probe.gd::_run()`'s own "R" leg already does for the
## peek sheet.
func fan_bounds_rect() -> Rect2:
	if not _open:
		return Rect2()
	var scale: float = app.phone_scale()
	var r: float = (OUTER_R + SLOT_D * 0.5) * scale
	return Rect2(_fan_center - Vector2(r, r), Vector2(r, r) * 2.0)


# -- Gesture entry points (probe seam and `_Pill`/`_Overlay` alike) ----------
#
# Mirrors `RadialRing.arm()`/`pointer()`/`release()`'s own split between the
# gesture and the drawing, so `_ctxfan_probe.gd` can drive the real state
# machine with plain global positions instead of synthesizing input events
# whose transform through a small, offset `Control` would otherwise have to
# be reverse-engineered by the probe.

## Gesture entry: the finger went down on the pill. Resets every fan flag and
## records the press position and time so `_process()` can open the fan after
## `FAN_HOLD_MS` or `pointer()` after `FAN_SLOP` of travel. Does not open
## anything itself; a press-release with no opening is a tap.
func press(gpos: Vector2) -> void:
	_pressed = true
	_open = false
	_sticky = false
	_hover = ""
	_sub = {}
	_press_pos = gpos
	_last_pos = gpos
	_press_ms = Time.get_ticks_msec()


## Gesture entry: the finger moved while pressed (global position). Before the
## fan opens, travel beyond `FAN_SLOP` (scaled by `phone_scale()`, so it is the
## canvas's 12 dp) opens it early; once open, it only retargets the hover.
## Ignored when no press is active.
func pointer(gpos: Vector2) -> void:
	if not _pressed:
		return
	_last_pos = gpos
	if not _open:
		if gpos.distance_to(_press_pos) > FAN_SLOP * app.phone_scale():
			_open_fan()
			_update_hover_from_pos(gpos)
		return
	_update_hover_from_pos(gpos)


## The pill's own button-up. A tap (fan never opened) toggles Inspect; a
## release once the fan is open resolves through `release_at()`, the same
## function a sticky tap on the overlay uses.
func release() -> void:
	_pressed = false
	if not _open:
		_toggle_pill_tap()
		return
	release_at(_last_pos)


## The shared release/selection grammar -- the pill's own drag-release AND a
## later sticky tap on `_overlay` both resolve through this one function
## (mirrors `RadialRing.click_at()`: re-aim, then reuse the release rule).
func release_at(gpos: Vector2) -> void:
	if not _open:
		return
	var hit := _hit_test(gpos)
	match String(hit.get("kind", "")):
		"back":
			_sub = {}
			_sticky = true
			_overlay.mouse_filter = Control.MOUSE_FILTER_STOP
			_overlay.queue_redraw()
		"sub_leaf":
			_run_sub_leaf(int(hit["i"]))
		"leaf_dir":
			_run_leaf(String(hit["dir"]))
		_:
			## The dead zone, or outside every sector.
			if _sticky:
				_close()
			else:
				_sticky = true
				_overlay.mouse_filter = Control.MOUSE_FILTER_STOP
				_overlay.queue_redraw()


## The overlay's own scrim/slot tap, once sticky (`_overlay.mouse_filter`
## only goes `STOP` in that state -- see `release_at()` above). A tap that
## hits nothing cancels; RadialRing's own rule for "already sticky".
func click_at(gpos: Vector2) -> void:
	release_at(gpos)


## A tap on the pill: arm Inspect and remember the tool it replaced, or, if
## Inspect is already armed, re-arm that remembered tool. Goes through
## `app.arm_tool` (the dock controls' own path) and never arms by any other
## route. Does nothing when Inspect is armed and nothing is remembered.
func _toggle_pill_tap() -> void:
	if app.armed_tool != "inspect":
		_prev_tool = app.armed_tool
		app.arm_tool("inspect")
	elif _prev_tool != "" and _prev_tool != "inspect":
		app.arm_tool(_prev_tool)


## Opens the half-fan: collects the 8 compass slots from the broker for the
## active domain (`ring_collect(ring_domain_req())`, shared with the desktop
## ring rather than re-derived), captures the pill's centre as the fan origin,
## fits the overlay to the viewport and shows it. The overlay ignores the mouse
## until the fan goes sticky, since the pill still owns the drag.
func _open_fan() -> void:
	_slots = app.context_broker.ring_collect(app.context_broker.ring_domain_req())
	_fan_center = _pill.get_global_rect().get_center()
	_open = true
	_sticky = false
	_hover = ""
	_sub = {}
	_fit_overlay()
	_overlay.visible = true
	_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE  ## The pill still owns
		## the drag until release; see `release_at()` for when this flips.
	_overlay.queue_redraw()


## Replaces the top-level fan with a sub-fan of `items` (a slot's `children`).
## Forces sticky and makes the overlay take taps, because a sub-fan is only
## ever picked by a later tap, never by the drag that opened the parent.
func _open_sub(dir: String, items: Array) -> void:
	_sub = {"dir": dir, "items": items, "hover": -1}
	_sticky = true
	_overlay.mouse_filter = Control.MOUSE_FILTER_STOP
	_overlay.queue_redraw()


## Resolves a top-level slot pick. A missing or disabled slot just closes the
## fan. A slot with `children` opens its sub-fan instead of closing. Otherwise
## the fan is closed FIRST and then the provider's `callable` runs, so the
## action never executes under a still-open overlay. Must not build its own
## arming logic: the callable is the dock control's own function.
func _run_leaf(dir: String) -> void:
	var row: Dictionary = _slots.get(dir, {})
	if row.is_empty() or not bool(row.get("enabled", true)):
		_close()
		return
	if row.has("children"):
		_open_sub(dir, row["children"])
		return
	var cb: Callable = row.get("callable", Callable())
	_close()
	if cb.is_valid():
		cb.call()


## Resolves a sub-fan pick by index. Out-of-range or disabled closes the fan;
## otherwise closes first and then calls the item's `callable` (same ordering
## and same no-second-implementation rule as `_run_leaf()`).
func _run_sub_leaf(i: int) -> void:
	var items: Array = _sub.get("items", [])
	if i < 0 or i >= items.size():
		_close()
		return
	var row: Dictionary = items[i]
	if not bool(row.get("enabled", true)):
		_close()
		return
	var cb: Callable = row.get("callable", Callable())
	_close()
	if cb.is_valid():
		cb.call()


## Tears the fan down completely: every gesture flag, the slot cache and the
## sub-fan are reset, and the overlay is hidden and made click-through. Safe to
## call when already closed. It also clears `_pressed`, so a close during a drag
## ends that drag.
func _close() -> void:
	_open = false
	_sticky = false
	_sub = {}
	_hover = ""
	_slots = {}
	_pressed = false
	_overlay.visible = false
	_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_overlay.queue_redraw()


## Sizes the top-level overlay to the whole visible viewport so its scrim and
## sticky-tap surface cover everything. Must run on every open (the window may
## have resized).
func _fit_overlay() -> void:
	var r: Rect2 = app.get_viewport().get_visible_rect()
	_overlay.position = Vector2.ZERO
	_overlay.size = r.size


## Maps a global position to what it picked: `{kind: "back"}` (inside the dead
## zone of a sub-fan), `{kind: "sub_leaf", i}`, `{kind: "leaf_dir", dir}` or
## `{kind: "none"}`. A slot wins only if its centre is within `HIT_MAX` (scaled)
## of the finger, nearest wins; the dead zone around the fan origin picks
## nothing at top level. Sub-fan items are spread over the 180-degree arc at
## `OUTER_R`; a single item sits at 90 degrees (the `n > 1` guard avoids a
## divide by zero). Pure: reads state, changes none.
func _hit_test(gpos: Vector2) -> Dictionary:
	var scale: float = app.phone_scale()
	if not _sub.is_empty():
		if gpos.distance_to(_fan_center) <= DEAD * scale:
			return {"kind": "back"}
		var items: Array = _sub.get("items", [])
		var n := items.size()
		var best := -1
		var best_d: float = HIT_MAX * scale
		for i in n:
			var ang: float = deg_to_rad(180.0 - float(i) * 180.0 / float(n - 1)) if n > 1 \
				else deg_to_rad(90.0)
			var pos: Vector2 = _fan_center + Vector2(cos(ang), -sin(ang)) * OUTER_R * scale
			var d: float = gpos.distance_to(pos)
			if d < best_d:
				best_d = d
				best = i
		if best >= 0:
			return {"kind": "sub_leaf", "i": best}
		return {"kind": "none"}
	if gpos.distance_to(_fan_center) <= DEAD * scale:
		return {"kind": "none"}
	var best_dir := ""
	var best_d2: float = HIT_MAX * scale
	for f in FAN:
		var ang2: float = deg_to_rad(float(f[1]))
		var pos2: Vector2 = _fan_center + Vector2(cos(ang2), -sin(ang2)) * float(f[2]) * scale
		var d2: float = gpos.distance_to(pos2)
		if d2 < best_d2:
			best_d2 = d2
			best_dir = String(f[0])
	if best_dir != "":
		return {"kind": "leaf_dir", "dir": best_dir}
	return {"kind": "none"}


## Re-runs `_hit_test()` for a finger position and updates the hover (the
## top-level direction, or the sub-fan's index), redrawing the overlay only when
## the hover actually changed.
func _update_hover_from_pos(gpos: Vector2) -> void:
	var hit := _hit_test(gpos)
	if not _sub.is_empty():
		var hv := int(hit["i"]) if String(hit.get("kind", "")) == "sub_leaf" else -1
		if hv != int(_sub.get("hover", -1)):
			_sub["hover"] = hv
			_overlay.queue_redraw()
		return
	var hv2 := String(hit["dir"]) if String(hit.get("kind", "")) == "leaf_dir" else ""
	if hv2 != _hover:
		_hover = hv2
		_overlay.queue_redraw()


# -- The pill -----------------------------------------------------------------

class _Pill:
	extends Control

	var fan  ## ToolFan

	## Takes the owning `ToolFan` and makes the pill consume every event on it
	## (`MOUSE_FILTER_STOP`, no focus), which is why the map's own gesture
	## classifier never sees a press on the pill.
	func _init(fan_ref) -> void:
		fan = fan_ref
		mouse_filter = Control.MOUSE_FILTER_STOP
		focus_mode = Control.FOCUS_NONE

	## Scales a dp figure by `phone_scale()` and floors it at `PHONE_TAP_MIN`, for
	## sizes that must stay tappable (the pill's height).
	func _pt(px: float) -> int:
		return maxi(DccTheme.PHONE_TAP_MIN, int(round(px * fan.app.phone_scale())))

	## Scales a dp figure by `phone_scale()` with a 1 px floor, for sizes that need
	## no tap-target minimum (width, margins, corner padding).
	func _ps(px: float) -> int:
		return maxi(1, int(round(px * fan.app.phone_scale())))

	## Sizes and anchors the pill: 188 x 44 dp (canvas figures), bottom-centre of its
	## parent, 10 dp above the parent's bottom edge (a labelled judgement). Layout
	## only; must stay anchored to the parent so it clears the nav and tool sheet.
	func _ready() -> void:
		## Canvas: 188 x 44 dp, bottom-centre, `top:668` of a 728-tall canvas --
		## i.e. clear of the bottom nav below it. Anchored the same way
		## `_build_phone_undo_chip()` anchors into `_phone_content_gap`: that
		## container already excludes the tool sheet AND the bottom nav, so a
		## bottom-anchor here sits directly above the tool sheet, which is
		## "above the bottom nav" once the nav sits below the sheet.
		var w := _ps(188)
		var h := _pt(44)
		set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
		anchor_left = 0.5
		anchor_right = 0.5
		offset_left = -w / 2.0
		offset_right = w / 2.0
		offset_top = -h - _ps(10)
		offset_bottom = -_ps(10)
		custom_minimum_size = Vector2(w, h)

	## Paints the pill as a capsule (filled when a non-Inspect tool is armed) with
	## the armed-tool name centred in mono. Reads `armed_visual()` and
	## `pill_label()`; holds no state of its own.
	func _draw() -> void:
		var sb := DccTheme.pill(false, int(size.y * 0.5), _ps(8), _ps(4))
		if fan.armed_visual():
			sb = DccTheme.pill(true, int(size.y * 0.5), _ps(8), _ps(4))
		draw_style_box(sb, Rect2(Vector2.ZERO, size))
		var label: String = fan.pill_label()
		var font := DccTheme.mono(0, true)
		var fs := DccTheme.FS_SMALL
		var ink := DccTheme.c("accent_ink") if fan.armed_visual() else DccTheme.c("text_bright")
		var ts := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, Vector2((size.x - ts.x) * 0.5, size.y * 0.5 + ts.y * 0.32), label,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, ink)

	## Routes left-button, touch and motion/drag events into the fan's gesture entry
	## points (`press`, `pointer`, `release`) in GLOBAL coordinates (the fan's
	## geometry is global). Every handled event is accepted so nothing falls
	## through to the map. Right/other buttons are ignored.
	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			if mb.button_index != MOUSE_BUTTON_LEFT:
				return
			var g: Vector2 = get_global_transform_with_canvas() * mb.position
			if mb.pressed:
				fan.press(g)
			else:
				fan.release()
			accept_event()
			queue_redraw()
			return
		if event is InputEventScreenTouch:
			var st := event as InputEventScreenTouch
			var g2: Vector2 = get_global_transform_with_canvas() * st.position
			if st.pressed:
				fan.press(g2)
			else:
				fan.release()
			accept_event()
			queue_redraw()
			return
		if event is InputEventMouseMotion:
			fan.pointer(get_global_transform_with_canvas() * (event as InputEventMouseMotion).position)
			accept_event()
			return
		if event is InputEventScreenDrag:
			fan.pointer(get_global_transform_with_canvas() * (event as InputEventScreenDrag).position)
			accept_event()
			return


# -- The fan overlay -----------------------------------------------------------

class _Overlay:
	extends Control

	var fan  ## ToolFan

	## Takes the owning `ToolFan`; top-level so it draws above the tree regardless
	## of the parent's transform, hidden and click-through until the fan opens.
	func _init(fan_ref) -> void:
		fan = fan_ref
		top_level = true
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		visible = false

	## Handles only the sticky-state tap: a button/touch RELEASE is forwarded to
	## `fan.click_at()`. Presses are ignored so the same tap is not counted twice.
	## Reached only once `release_at()` has set this control to `MOUSE_FILTER_STOP`.
	func _gui_input(event: InputEvent) -> void:
		## Only reached once `fan.release_at()` has flipped this control to
		## `STOP` (sticky) -- see that function's own comment. A tap that hits
		## nothing cancels; one that hits a slot selects it.
		if event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			if mb.button_index != MOUSE_BUTTON_LEFT or mb.pressed:
				return
			fan.click_at(mb.position)
			accept_event()
			return
		if event is InputEventScreenTouch:
			var st := event as InputEventScreenTouch
			if st.pressed:
				return
			fan.click_at(st.position)
			accept_event()
			return

	## Draws the dim scrim, then the top-level slots or the open sub-fan, then the
	## caption. Draws nothing when the fan is closed. Reads fan state; never
	## mutates it.
	func _draw() -> void:
		if not fan._open:
			return
		## The dim scrim (canvas: `rgba(8,9,10,.55)`), drawn whenever the fan
		## exists at all, whether or not it is sticky yet -- the same "an open
		## surface is visibly open" cue the ring's own draw gives with its
		## caption band.
		draw_rect(Rect2(Vector2.ZERO, size), Color(0.031, 0.035, 0.039, 0.55))
		if fan._sub.is_empty():
			_draw_top(fan._slots, fan._hover)
		else:
			_draw_sub(fan._sub)
		_draw_caption()

	## Draws the 8 top-level slots at their `FAN` angle and radius (scaled),
	## highlighting `hover_dir`. Positions mirror `ToolFan._hit_test()`, which must
	## change in step.
	func _draw_top(slots: Dictionary, hover_dir: String) -> void:
		var scale: float = fan.app.phone_scale()
		for f in fan.FAN:
			var dir: String = f[0]
			var row: Dictionary = slots.get(dir, {})
			var ang: float = deg_to_rad(float(f[1]))
			var pos: Vector2 = fan._fan_center + Vector2(cos(ang), -sin(ang)) * float(f[2]) * scale
			_draw_slot(pos, row, dir == hover_dir, scale)

	## Draws the sub-fan's items on the outer arc (same spread as `_hit_test()`),
	## then the accent BACK button at the fan origin. Colour literals are the
	## canvas's own (`rgba(224,163,74,.25)` glow). Must stay consistent with
	## `_hit_test()`'s `back` zone.
	func _draw_sub(sub: Dictionary) -> void:
		var scale: float = fan.app.phone_scale()
		var items: Array = sub.get("items", [])
		var n := items.size()
		var hover_i := int(sub.get("hover", -1))
		for i in n:
			var ang: float = deg_to_rad(180.0 - float(i) * 180.0 / float(n - 1)) if n > 1 \
				else deg_to_rad(90.0)
			var pos: Vector2 = fan._fan_center + Vector2(cos(ang), -sin(ang)) * fan.OUTER_R * scale
			_draw_slot(pos, items[i], i == hover_i, scale)
		## The back affordance, at the pill's own centre -- canvas: a 56 px
		## accent-filled `‹ BACK` button with a glow ring (`box-shadow: 0 0 0
		## 4px rgba(224,163,74,.25)`), over the pill while a sub-fan is open.
		## Ruling BC / the 2026-09-25 refinement: "the phone fan's back button
		## is accent-filled and labelled too" -- this used to draw only the
		## glyph, at `fan.DEAD` (40 dp), not the canvas's own 56 px/28 radius.
		var r: float = (fan.SLOT_D * 0.5) * scale
		draw_circle(fan._fan_center, r + 4.0 * scale, Color(DccTheme.c("accent"), 0.25))
		draw_circle(fan._fan_center, r, DccTheme.c("accent"))
		draw_arc(fan._fan_center, r, 0.0, TAU, 32, DccTheme.c("accent"), 2.0, true)
		var font := DccTheme.mono(0, true)
		var gfs: int = maxi(10, int(r * 0.62))
		var ts := font.get_string_size("‹", HORIZONTAL_ALIGNMENT_LEFT, -1, gfs)
		draw_string(font, fan._fan_center - ts * 0.5 + Vector2(0.0, ts.y * 0.32 - r * 0.26), "‹",
			HORIZONTAL_ALIGNMENT_LEFT, -1, gfs, DccTheme.c("accent_ink"))
		var label := "BACK"
		var lfs := DccTheme.FS_MICRO
		var lts := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, lfs)
		draw_string(font, fan._fan_center + Vector2(-lts.x * 0.5, r * 0.58), label,
			HORIZONTAL_ALIGNMENT_LEFT, -1, lfs, DccTheme.c("accent_ink"))

	## Draws one slot circle: sunken ground (accent when armed), warm wash when
	## hovered, border, then glyph and label via `_draw_fitted_slot()`. An absent
	## slot (`row` empty) draws only the faint empty circle; a disabled one is
	## drawn at 0.35 alpha (a labelled judgement).
	func _draw_slot(pos: Vector2, row: Dictionary, hovered: bool, scale: float) -> void:
		var present := not row.is_empty()
		var enabled := present and bool(row.get("enabled", true))
		var armed := present and bool(row.get("armed", false))
		var d: float = fan.SLOT_D * scale * 0.5
		var alpha := 1.0 if enabled else 0.35
		var ground := DccTheme.c("accent") if armed else DccTheme.c("sunken")
		draw_circle(pos, d, Color(ground.r, ground.g, ground.b, ground.a * alpha))
		if hovered and not armed:
			## Refinement note: "the hover state is a warm accent tint"
			## (Ruling BC) -- the same `accent_wash_2` overlay
			## `radial_ring.gd::_draw_ring()` uses for the identical state.
			draw_circle(pos, d, DccTheme.c("accent_wash_2"))
		var border := DccTheme.c("accent") if (hovered or armed) else DccTheme.c("border")
		draw_arc(pos, d, 0.0, TAU, 24, Color(border.r, border.g, border.b, alpha),
			2.0 if hovered else 1.0, true)
		if not present:
			return
		var ink := DccTheme.c("accent_ink") if armed else \
			(DccTheme.c("accent_hover") if hovered else DccTheme.c("text_bright"))
		ink.a = alpha
		## Ruling BC: label inside the slot (mockup's flex column, glyph above
		## / label below, both inside the 56 px circle) rather than hanging
		## below it. `_draw_fitted_slot()` wraps/shrinks so the label's own
		## rect never crosses the circle -- `radial_ring.gd`'s own function,
		## duplicated here rather than imported (this file's own header:
		## "the two files' geometry does not overlap enough to factor out").
		_draw_fitted_slot(pos, d, String(row.get("glyph", "")), String(row.get("label", "")), ink)

	## Half the width of a circle's chord at vertical offset `dy` from its centre,
	## 0 at or beyond the rim. Used to decide whether a text line fits inside a
	## slot circle.
	static func _chord_half_width(radius: float, dy: float) -> float:
		var dd := absf(dy)
		if dd >= radius:
			return 0.0
		return sqrt(radius * radius - dd * dd)

	## Splits a label into two lines at the space or `/` nearest its middle, or
	## returns `[]` if there is none or either half would be empty.
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

	## Smallest font size `_draw_fitted_slot()` will shrink a label to
	## (a legibility floor; labelled judgement).
	const MIN_LABEL_FS := 7

	## `radial_ring.gd::_draw_fitted_slot()`'s own algorithm -- see that
	## copy's header for the wrap/shrink/margin reasoning.
	## Draws the glyph above the label inside the circle. Shrinks the font one
	## pixel at a time (down to `MIN_LABEL_FS`) and tries a two-line split until
	## every line fits the circle's chord with a 6 px margin; at the floor it
	## settles for the split (or the single line) even if it still overflows.
	## The fractions (0.32, 0.62, 0.10, 0.30, 1.15 line height) are layout
	## judgements copied from that function; keep the two in step.
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
		var lh2 := float(fs) * 1.15
		for i in lines.size():
			var dy := top_dy + lh2 * float(i) + lh2 * 0.5
			var line := String(lines[i])
			var ts := font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
			draw_string(font, centre + Vector2(-ts.x * 0.5, dy + ts.y * 0.32), line,
				HORIZONTAL_ALIGNMENT_LEFT, -1, fs, ink)

	## Draws a slot glyph: `sym:<name>` looks up a text symbol in
	## `DccIcons.SYMBOLS` (`?` if unknown); anything else is a `DccIcons` texture
	## tinted by `color`. Draws nothing for an empty name or a missing icon.
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

	## Draws the hint band above the fan: the gesture instruction by default, the
	## hovered slot's direction and label, the sub-fan's parent and hovered item,
	## or the sticky-state tap hint. The 40 dp rise above `OUTER_R` and the
	## `9 x 5` padding are layout judgements, not canvas figures.
	func _draw_caption() -> void:
		var text := "slide to a slot, lift to pick"
		if not fan._sub.is_empty():
			var hi := int(fan._sub.get("hover", -1))
			var items: Array = fan._sub.get("items", [])
			var picked := "pick one, or tap the centre for back"
			if hi >= 0 and hi < items.size():
				picked = String((items[hi] as Dictionary).get("label", ""))
			text = "%s ▸ %s" % [fan._sub.get("dir", ""), picked]
		elif fan._hover != "":
			var row: Dictionary = fan._slots.get(fan._hover, {})
			text = "%s · %s" % [fan._hover, String(row.get("label", ""))]
		elif fan._sticky:
			text = "tap a slot · tap outside to close"
		var font := DccTheme.mono()
		var fs := DccTheme.FS_SMALL
		var ts := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		var cap_centre: Vector2 = fan._fan_center + Vector2(0.0, -(fan.OUTER_R + 40.0) * fan.app.phone_scale())
		var pad := Vector2(9.0, 5.0)
		var rect := Rect2(cap_centre - ts * 0.5 - pad, ts + pad * 2.0)
		draw_rect(rect, DccTheme.c("panel"))
		draw_rect(rect, DccTheme.c("line"), false, 1.0)
		draw_string(font, cap_centre - Vector2(ts.x * 0.5, -ts.y * 0.32), text,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, DccTheme.c("text_bright"))


# -- Pill content, read by `_Pill._draw()` -------------------------------------

## Whether the pill should draw filled: true whenever any tool other than
## Inspect is armed. Read by `_Pill._draw()`.
func armed_visual() -> bool:
	return app != null and String(app.armed_tool) != "inspect"


## The pill's text: the armed tool id upper-cased (`INSPECT` before `app`
## exists). Read by `_Pill._draw()`.
func pill_label() -> String:
	if app == null:
		return "INSPECT"
	return String(app.armed_tool).capitalize().to_upper()
