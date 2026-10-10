extends Control
## The elevation colour ramp as an interactive gradient bar (editing-tools plan,
## milestone B; modelled on PhotoCraft's gradient widget). No `class_name` on
## purpose: hosts load it with `preload("res://shell/ramp_bar.gd")` and call
## `.new()`, so registering it never needs the `godot --import` pass that strips
## `project.godot`'s comment block (CLAUDE.md, last working rule).
##
## Gestures, each previewing live and committing ONE logical step on release:
##   - drag a handle sideways: move that stop (clamped strictly between its
##     neighbours, so a drag never reorders; the first and last stop keep their
##     offsets -- the ramp's ends stay where the reference ramps put them);
##   - drag a handle more than `DELETE_DIST` off the bar: delete it (never below
##     `MIN_STOPS`); dragging back restores it;
##   - click empty bar: add a stop there, coloured with what the ramp shows at
##     that offset, and keep dragging it if the button stays down;
##   - double-click a handle: open the milestone-A OK/Cancel colour popup.
##
## The bar writes the ENGINE ramp itself (`bridge.set_color_ramp`) and then tells
## its host through `on_edited(stops, selected, final)` so the host can repaint
## its own copies and re-render the map. `final` is false for live ticks and
## true exactly once per gesture. It never rebuilds anything and never frees a
## widget: the host decides whether to rebuild on the `final` call (and should
## defer it, since this node is inside its own input handler when that fires).
##
## Undo: the engine has no ramp history (the global Undo is heightmap-only), so
## "one commit" here means one `final` callback. This file owns no history; the
## shared look history (`shell/look_history.gd`, Ruling BY item 2) is driven by the
## HOST: `on_begin` fires once when a gesture starts (BEFORE its first engine
## write) so the host can snapshot, and the host pushes the entry from its `final`
## callback. Never invent a history system in this file.

## Pixels beyond the bar's vertical extent at which a held handle means "delete".
## A labelled judgement, borrowed from PhotoCraft's gradient widget behaviour.
const DELETE_DIST := 24.0
## A ramp needs at least two stops to be a ramp (`right_dock.gd` section note).
const MIN_STOPS := 2
## Upper bound on clicks-to-add: a labelled judgement so a held-down click
## cannot fill the engine with stops; far above any preset.
const MAX_STOPS := 24
## Closest a dragged stop may come to a neighbour, in ramp offset. Labelled
## judgement: below the 0.005 row slider step, above float noise.
const EPS := 0.002
## Minimum milliseconds between live engine writes during a drag. A labelled
## judgement (about 30 Hz); the release always writes unthrottled.
const LIVE_MS := 33
const BAR_H := 16.0
const HANDLE_H := 16.0
## Touch target: the shell's 44 px minimum (DccTheme/DccWidgets floor).
const TOUCH_PX := 44.0

var bridge: Object
var on_edited: Callable
## Called once at the start of a gesture (drag press, add-click, colour popup
## open), before the first engine write. See the Undo paragraph above.
var on_begin: Callable
## `Callable(offset: float) -> float` giving metres for the drag label.
var metres_fn: Callable
var selected := -1
var _stops: Array = []   ## [[offset, Color], ...] sorted; what is drawn
var _mode := "Linear"
var _grad := Gradient.new()
var _tex := GradientTexture1D.new()
var _base: Array = []    ## the list as it was at press, for delete/restore
var _drag := -1          ## index in `_base` being dragged, -1 when idle
var _dragging_off_bar := false
var _changed := false
var _pending := false
var _last_write := 0
var _pick: ColorPickerButton
var _edit_idx := -1
var _drag_label := ""
var _drag_off := 0.0     ## offset of the dragged stop, for the label position

## Configure the bar and give it its first stops. `host_bridge` needs
## `color_ramp()`/`set_color_ramp()`. Call once, before adding to the tree.
## `begin` (optional, no arguments) is the gesture-start hook described above.
func setup(host_bridge: Object, edited: Callable, metres: Callable, begin: Callable = Callable()) -> void:
	bridge = host_bridge
	on_edited = edited
	metres_fn = metres
	on_begin = begin
	_tex.gradient = _grad
	_tex.width = 256
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	tooltip_text = "Drag a stop to move it. Drag it off the bar to delete it. Click the bar to add a stop. Double-click a stop to edit its colour."
	var touch := DccTheme.is_touch()
	custom_minimum_size.y = (BAR_H + TOUCH_PX) if touch else (BAR_H + HANDLE_H)
	set_process(false)

## Replace what is drawn with `stops` (any order) in blend `mode`. Ignored while
## a gesture is running, so a host's refresh can never yank a handle from under
## the pointer. Never writes the engine.
func set_stops(stops: Array, mode: String, select: int = -2) -> void:
	if _drag >= 0 or _edit_idx >= 0:
		return
	_mode = mode
	_stops = _sorted_copy(stops)
	if select != -2:
		selected = select
	_rebuild_gradient()
	queue_redraw()

## Deep copy sorted by offset, so edits never alias the caller's arrays.
func _sorted_copy(stops: Array) -> Array:
	var out: Array = []
	for s in stops:
		out.append([float(s[0]), s[1] as Color])
	out.sort_custom(func(a, b): return float(a[0]) < float(b[0]))
	return out

## Rebuild the display gradient from `_stops` -- the blend-mode logic is
## `right_dock.gd::_ramp_bar_texture`'s (Step constant, Ease cubic as an
## approximation of the renderer's smoothstep, anything else linear). Colours are
## drawn opaque: a stop's alpha is how far it takes part, not a swatch property.
func _rebuild_gradient() -> void:
	var offsets := PackedFloat32Array()
	var colors := PackedColorArray()
	for s in _stops:
		offsets.append(clampf(float(s[0]), 0.0, 1.0))
		var c: Color = s[1]
		colors.append(Color(c.r, c.g, c.b, 1.0))
	if offsets.is_empty():
		return
	if offsets.size() == 1:
		offsets.append(1.0)
		colors.append(colors[0])
	match _mode:
		"Step": _grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CONSTANT
		"Ease": _grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CUBIC
		_: _grad.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_LINEAR
	_grad.offsets = offsets
	_grad.colors = colors

## Draw the gradient, one handle per stop, and the metres label while dragging.
func _draw() -> void:
	var w := size.x
	if w <= 1.0:
		return
	if not _stops.is_empty():
		draw_texture_rect(_tex, Rect2(0, 0, w, BAR_H), false)
	draw_rect(Rect2(0, 0, w, BAR_H), DccTheme.c("text_dim"), false, 1.0)
	var font := ThemeDB.fallback_font
	for i in _stops.size():
		var x := clampf(float(_stops[i][0]), 0.0, 1.0) * w
		var col: Color = _stops[i][1]
		col = Color(col.r, col.g, col.b, 1.0)
		var top := BAR_H + 1.0
		var pts := PackedVector2Array([
			Vector2(x, top), Vector2(x - 5, top + 6), Vector2(x - 5, top + HANDLE_H - 1),
			Vector2(x + 5, top + HANDLE_H - 1), Vector2(x + 5, top + 6)])
		draw_colored_polygon(pts, col)
		var is_sel := i == selected and not _dragging_off_bar
		var ring := DccTheme.c("accent") if is_sel else DccTheme.c("text_dim")
		var closed := pts.duplicate()
		closed.append(pts[0])
		draw_polyline(closed, ring, 2.0 if is_sel else 1.0)
	if _drag >= 0 and _drag_label != "":
		var x := clampf(_drag_off, 0.0, 1.0) * w
		var tw := font.get_string_size(_drag_label, HORIZONTAL_ALIGNMENT_LEFT, -1, 11).x
		var lx := x + 9.0 if x + 9.0 + tw < w else x - 9.0 - tw
		draw_string(font, Vector2(maxf(lx, 0.0), BAR_H + HANDLE_H - 3.0), _drag_label,
			HORIZONTAL_ALIGNMENT_LEFT, -1, 11, DccTheme.c("text_bright"))

## Nearest handle within the hit radius of local x, or -1. The radius is the
## touch floor's half-width on touch, 8 px otherwise (judgement).
func _hit(px: float) -> int:
	var r := TOUCH_PX * 0.5 if DccTheme.is_touch() else 8.0
	var best := -1
	var bd := r
	for i in _stops.size():
		var d := absf(clampf(float(_stops[i][0]), 0.0, 1.0) * size.x - px)
		if d <= bd:
			bd = d
			best = i
	return best

## Pointer handling: press selects / starts a drag / adds; double-click opens the
## colour popup; motion drags; release commits. Consumes what it uses.
func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			var h := _hit(event.position.x)
			if h >= 0:
				if event.double_click:
					selected = h
					_open_colour(h)
					accept_event()
					return
				_begin_drag(h)
			elif _stops.size() < MAX_STOPS:
				_add_at(clampf(event.position.x / maxf(size.x, 1.0), 0.0, 1.0))
			accept_event()
		else:
			_end_drag()
			accept_event()
	elif event is InputEventMouseMotion and _drag >= 0:
		_drag_to(event.position)
		accept_event()

## Select stop `i` and start dragging it. `_base` is the list at press; every
## tick is computed from it so dragging off the bar and back is lossless.
func _begin_drag(i: int) -> void:
	if on_begin.is_valid():
		on_begin.call()   ## before any write: `_add_at` writes only after this returns
	selected = i
	_base = _sorted_copy(_stops)
	_drag = i
	_changed = false
	_dragging_off_bar = false
	_drag_off = float(_base[i][0])
	_drag_label = _label_for(_drag_off)
	queue_redraw()

## Insert a stop at `off` coloured by the current blend there, write it live and
## carry on as a drag of that stop (so click-and-slide works in one gesture).
func _add_at(off: float) -> void:
	var col: Color = _grad.sample(off)
	var list := _sorted_copy(_stops)
	var at := list.size()
	for i in list.size():
		if float(list[i][0]) > off:
			at = i
			break
	list.insert(at, [off, col])
	_stops = list
	_rebuild_gradient()
	_begin_drag(at)
	_changed = true
	_write(true, false)

## Apply a pointer position to the drag: sideways = move (clamped between the
## neighbours, ends fixed), far above/below = delete preview.
func _drag_to(p: Vector2) -> void:
	var list := _sorted_copy(_base)
	var i := _drag
	var off := float(list[i][0])
	var last := list.size() - 1
	if i != 0 and i != last:
		var lo := float(list[i - 1][0]) + EPS
		var hi := float(list[i + 1][0]) - EPS
		off = clampf(p.x / maxf(size.x, 1.0), lo, maxf(lo, hi))
	var limit := DELETE_DIST * (1.6 if DccTheme.is_touch() else 1.0)
	var off_bar := (p.y < -limit or p.y > size.y + limit) and list.size() > MIN_STOPS
	if off_bar != _dragging_off_bar or absf(off - float(list[i][0])) > 1e-6:
		_changed = true
	_dragging_off_bar = off_bar
	list[i][0] = off
	_drag_off = off
	if off_bar:
		list.remove_at(i)
		_drag_label = "release to delete"
	else:
		_drag_label = _label_for(off)
	_stops = list
	_rebuild_gradient()
	queue_redraw()
	_write(false, false)

## "NNN m" for the drag label, from the host's `metres_fn`.
func _label_for(off: float) -> String:
	if metres_fn.is_valid():
		return "%d m" % int(round(float(metres_fn.call(off))))
	return "%d%%" % int(round(off * 100.0))

## Write the current list to the engine. Live ticks are throttled to `LIVE_MS`
## (the remainder flushes from `_process`); `force` bypasses it. `final` marks
## the one callback per gesture.
func _write(force: bool, final: bool) -> void:
	var now := Time.get_ticks_msec()
	if not force and not final and now - _last_write < LIVE_MS:
		_pending = true
		set_process(true)
		return
	_pending = false
	_last_write = now
	if bridge == null or _stops.is_empty():
		return
	var out: Array = []
	for s in _stops:
		out.append([float(s[0]), s[1] as Color])
	if bridge.set_color_ramp(out) <= 0:
		return
	if on_edited.is_valid():
		on_edited.call(_stops.duplicate(true), selected, final)

## Flush a throttled live write once the interval has passed.
func _process(_d: float) -> void:
	if not _pending:
		set_process(false)
		return
	if Time.get_ticks_msec() - _last_write >= LIVE_MS:
		_write(true, false)
		set_process(false)

## Release: flush the last position, re-read the engine's own (sorted) list and
## report the single `final` step. A press-release with no movement commits
## nothing (it was only a selection), except an add, which already changed.
func _end_drag() -> void:
	if _drag < 0:
		return
	var changed := _changed
	if _dragging_off_bar:
		selected = -1
	_drag = -1
	_dragging_off_bar = false
	_drag_label = ""
	if changed:
		_write(true, false)
		if bridge != null:
			_stops = _sorted_copy(bridge.color_ramp())
		_rebuild_gradient()
		if on_edited.is_valid():
			on_edited.call(_stops.duplicate(true), selected, true)
	else:
		if on_edited.is_valid():
			on_edited.call(_stops.duplicate(true), selected, false)
	queue_redraw()

## Open the OK/Cancel colour popup (`DccWidgets.confirmable_color`) for stop `i`.
## A hidden `ColorPickerButton` child owns the popup; live colour samples write
## the engine like a drag tick, Cancel/Escape/outside-click revert through the
## same write, and closing reports the one `final` step.
func _open_colour(i: int) -> void:
	if i < 0 or i >= _stops.size():
		return
	if _pick == null:
		_pick = ColorPickerButton.new()
		_pick.edit_alpha = false
		_pick.focus_mode = Control.FOCUS_NONE
		_pick.mouse_filter = Control.MOUSE_FILTER_IGNORE
		_pick.modulate = Color(1, 1, 1, 0)
		_pick.custom_minimum_size = Vector2(1, 1)
		add_child(_pick)
		_pick.color_changed.connect(func(c: Color): _set_colour(_edit_idx, c, false))
		DccWidgets.confirmable_color(_pick, func(c: Color): _set_colour(_edit_idx, c, false))
		_pick.popup_closed.connect(_on_pick_closed)
	if on_begin.is_valid():
		on_begin.call()   ## the colour dialog's live ticks write the engine
	_edit_idx = i
	var c: Color = _stops[i][1]
	_pick.color = Color(c.r, c.g, c.b, 1.0)
	var x := clampf(float(_stops[i][0]), 0.0, 1.0) * size.x
	var origin := get_screen_position() + Vector2(x, BAR_H + HANDLE_H)
	_pick.get_popup().popup(Rect2i(Vector2i(origin), Vector2i.ZERO))

## Write a new colour (alpha kept) for displayed stop `i`.
func _set_colour(i: int, c: Color, final: bool) -> void:
	if i < 0 or i >= _stops.size():
		return
	var old: Color = _stops[i][1]
	_stops[i][1] = Color(c.r, c.g, c.b, old.a)
	_rebuild_gradient()
	queue_redraw()
	_write(true, final)

## Popup closed (OK, Cancel, Escape or outside click -- a cancel has already
## reverted through `_set_colour`): re-read the engine and report the one final
## step for the colour gesture.
func _on_pick_closed() -> void:
	_edit_idx = -1
	if bridge != null:
		_stops = _sorted_copy(bridge.color_ramp())
	_rebuild_gradient()
	queue_redraw()
	if on_edited.is_valid():
		on_edited.call(_stops.duplicate(true), selected, true)
