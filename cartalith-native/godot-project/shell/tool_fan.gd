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
## in that demo. This file tests `hover_index >= -1` the way `RadialRing.
## release()` already does for its own sub-ring (`if hi >= 0`), which is
## correct rather than a bug carried forward on purpose.
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

var _pill: Control
var _overlay: Control

var _pressed := false
var _open := false
var _sticky := false
var _press_pos := Vector2.ZERO   ## global
var _press_ms := 0
var _last_pos := Vector2.ZERO    ## global; last known finger position while pressed
var _fan_center := Vector2.ZERO  ## global; the pill's own centre, captured at open
var _slots: Dictionary = {}      ## `ContextBroker.ring_collect()`'s 8-slot compass
var _hover := ""                 ## top-level hover dir, "" when none
var _sub: Dictionary = {}        ## {} closed; else {"dir","items","hover":int}
var _prev_tool := ""             ## for the tap-toggle's "return to" memory


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

func is_open() -> bool:
	return _open


func is_sticky() -> bool:
	return _sticky


func sub_open() -> bool:
	return not _sub.is_empty()


func debug_state() -> Dictionary:
	return {
		"open": _open, "sticky": _sticky, "hover": _hover,
		"sub_dir": String(_sub.get("dir", "")), "sub_hover": int(_sub.get("hover", -1)),
		"sub_count": (_sub.get("items", []) as Array).size(),
	}


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

func press(gpos: Vector2) -> void:
	_pressed = true
	_open = false
	_sticky = false
	_hover = ""
	_sub = {}
	_press_pos = gpos
	_last_pos = gpos
	_press_ms = Time.get_ticks_msec()


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


func _toggle_pill_tap() -> void:
	if app.armed_tool != "inspect":
		_prev_tool = app.armed_tool
		app.arm_tool("inspect")
	elif _prev_tool != "" and _prev_tool != "inspect":
		app.arm_tool(_prev_tool)


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


func _open_sub(dir: String, items: Array) -> void:
	_sub = {"dir": dir, "items": items, "hover": -1}
	_sticky = true
	_overlay.mouse_filter = Control.MOUSE_FILTER_STOP
	_overlay.queue_redraw()


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


func _fit_overlay() -> void:
	var r: Rect2 = app.get_viewport().get_visible_rect()
	_overlay.position = Vector2.ZERO
	_overlay.size = r.size


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

	func _init(fan_ref) -> void:
		fan = fan_ref
		mouse_filter = Control.MOUSE_FILTER_STOP
		focus_mode = Control.FOCUS_NONE

	func _pt(px: float) -> int:
		return maxi(DccTheme.PHONE_TAP_MIN, int(round(px * fan.app.phone_scale())))

	func _ps(px: float) -> int:
		return maxi(1, int(round(px * fan.app.phone_scale())))

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

	func _init(fan_ref) -> void:
		fan = fan_ref
		top_level = true
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		visible = false

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

	func _draw_top(slots: Dictionary, hover_dir: String) -> void:
		var scale: float = fan.app.phone_scale()
		for f in fan.FAN:
			var dir: String = f[0]
			var row: Dictionary = slots.get(dir, {})
			var ang: float = deg_to_rad(float(f[1]))
			var pos: Vector2 = fan._fan_center + Vector2(cos(ang), -sin(ang)) * float(f[2]) * scale
			_draw_slot(pos, row, dir == hover_dir, scale)

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
		## The back affordance, at the pill's own centre -- canvas: a `BACK`
		## button drawn over the pill while a sub-fan is open.
		var r: float = fan.DEAD * scale
		draw_circle(fan._fan_center, r, DccTheme.c("accent"))
		var font := DccTheme.mono()
		var ts := font.get_string_size("‹", HORIZONTAL_ALIGNMENT_LEFT, -1, DccTheme.FS_SMALL)
		draw_string(font, fan._fan_center - ts * 0.5 + Vector2(0.0, ts.y * 0.32), "‹",
			HORIZONTAL_ALIGNMENT_LEFT, -1, DccTheme.FS_SMALL, DccTheme.c("accent_ink"))

	func _draw_slot(pos: Vector2, row: Dictionary, hovered: bool, scale: float) -> void:
		var present := not row.is_empty()
		var enabled := present and bool(row.get("enabled", true))
		var armed := present and bool(row.get("armed", false))
		var d: float = fan.SLOT_D * scale * 0.5
		var alpha := 1.0 if enabled else 0.35
		var ground := DccTheme.c("accent") if armed else DccTheme.c("sunken")
		draw_circle(pos, d, Color(ground.r, ground.g, ground.b, ground.a * alpha))
		var border := DccTheme.c("accent") if (hovered or armed) else DccTheme.c("line")
		draw_arc(pos, d, 0.0, TAU, 24, Color(border.r, border.g, border.b, alpha),
			2.0 if hovered else 1.0, true)
		if not present:
			return
		var ink := DccTheme.c("accent_ink") if armed else DccTheme.c("text_bright")
		ink.a = alpha
		_draw_glyph(String(row.get("glyph", "")), pos, d * 1.0, ink)
		_draw_label(String(row.get("label", "")), pos + Vector2(0.0, d + 11.0 * scale), ink)

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

	func _draw_label(text: String, centre: Vector2, color: Color) -> void:
		if text.is_empty():
			return
		var font := DccTheme.mono()
		var fs := DccTheme.FS_MICRO
		var ts := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, centre - Vector2(ts.x * 0.5, 0.0), text,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, color)

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

func armed_visual() -> bool:
	return app != null and String(app.armed_tool) != "inspect"


func pill_label() -> String:
	if app == null:
		return "INSPECT"
	return String(app.armed_tool).capitalize().to_upper()
