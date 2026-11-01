## DRAFT, not loaded by Godot. MS-11: the phone's quick drawer (`MOBILE_SHELL_PROPOSAL.md` §2.4, Q10 and Q11).
## The quick bar's pins in a side drawer, opened from a grip at the middle of the map's edge on the side away
## from the dominant hand: tap the grip, or drag it inward. Tap the scrim, drag the grip back, or press system
## back to close it.
## Target: a new file, `shell/quick_drawer.gd`, loaded by path from `quick_bar_phone.gd` (draft 07). It has no
## `class_name`.
##
## It is one more place the ONE pin list is drawn (`CUSTOM_QUICK_BAR_SCOPE.md` rule 1): it reads
## `QuickBarPhone.entries()` and presses through `QuickBarPhone.press()`, exactly as MENU's Quick section and
## the `All…` list sheet do. It runs nothing itself.
##
## Why a visible grip and not an edge swipe: with Android's gesture navigation a swipe that starts at a
## screen edge is the system's Back, while a tap is always delivered to the app.
##
## NOT parse-checked (no Godot binary in the session that wrote it).
extends Control

## Figures in phone units, multiplied once by the shell's `phone_scale()`. Floors are applied in units
## before that multiply, which is `_ptap()`'s lesson (`dcc_shell.gd`).
const GRIP_W_UNITS := 44.0        ## the tap target's width, PHONE_TAP_MIN
const GRIP_H_UNITS := 88.0        ## the tap target's height (proposal §2.4)
const TAB_W_UNITS := 22.0         ## the visible tab inside the target, against the edge
const TAB_H_UNITS := 72.0
const BAR_W_UNITS := 4.0          ## the accent bar on the tab
const BAR_H_UNITS := 34.0
const DRAWER_W_UNITS := 248.0     ## the mockup's `QD_W`
const DRAWER_W_FRAC := 0.78       ## never wider than this share of the window
const ROW_UNITS := 52.0           ## H_PHONE_ROW, the canvas's list row
const OPEN_FRAC := 0.4            ## a drag past this share of the width stays open on release
const SLOP_UNITS := 6.0           ## below this a press on the grip is a tap
const MIN_BAND_UNITS := 180.0     ## no grip when the map's free band is shorter than this
const SCRIM_ALPHA := 0.35
const ANIM_S := 0.18

var _app                          ## DccShell / DccApp
var _scale := 1.0
var _entries: Array = []
var _side := "left"
var _band := Rect2()              ## the map's free band, canvas px
var _w := 0.0                     ## drawer width, px
var _travel := 0.0                ## how far the drawer is out, 0 .. _w
var _open := false
var _drag: Dictionary = {}
var _tween: Tween

var _scrim: ColorRect
var _panel: PanelContainer
var _list: VBoxContainer
var _grip: Control

## Once, after `new()`. It builds the scrim, the panel and the grip, and listens for layout changes; the shell
## calls `relayout()` once the node is in the tree (draft 07).
func setup(app) -> void:
	_app = app
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_scrim = ColorRect.new()
	_scrim.color = Color(0, 0, 0, 0)
	_scrim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_scrim.gui_input.connect(_on_scrim_input)
	add_child(_scrim)
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", DccWidgets.box("line_soft", "panel", 8, 6))
	_panel.visible = false
	add_child(_panel)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_panel.add_child(scroll)
	_list = VBoxContainer.new()
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_list)
	_grip = Control.new()
	_grip.mouse_filter = Control.MOUSE_FILTER_STOP
	_grip.gui_input.connect(_on_grip_input)
	_grip.draw.connect(_draw_grip)
	add_child(_grip)
	if _app != null and _app.has_signal("phone_insets_changed"):
		_app.phone_insets_changed.connect(relayout)

## New entries from `QuickBarPhone.refresh()`: rebuilds the rows. The caller removes the drawer when the list
## is empty, so an empty bar draws nothing (CQ-6's rule).
func set_entries(es: Array) -> void:
	_entries = es
	for c in _list.get_children():
		c.queue_free()
	var head := DccTheme.mono_label("QUICK · %d" % es.size(), "text_dim", _font_px(10.0))
	_list.add_child(head)
	for e in es:
		var entry := e as Dictionary
		_list.add_child(_row(String(entry["title"]), QuickBarPhoneList._sub_for(entry),
			bool(entry["live"]), _tap.bind(entry)))
	_list.add_child(_row("Customise…", "Pin, unpin and reorder commands", true, _customise))

## Re-reads the scale, the side (`DccSettings.dominant_hand()`; the drawer sits away from it) and the
## map's free band (`phone_content_insets()`), then places everything. Called on `phone_insets_changed`, which
## fires on rotation and on tool-sheet changes. Whether it also fires on every sheet detent change is checked
## in MS-11 (README).
func relayout() -> void:
	if _app == null or not is_inside_tree():
		return
	_scale = float(_app.call("phone_scale")) if _app.has_method("phone_scale") else 1.0
	_side = "right" if DccSettings.dominant_hand() == "left" else "left"
	var vp := get_viewport_rect().size
	var ins: Dictionary = _app.call("phone_content_insets")
	_band = Rect2(float(ins.get("left", 0.0)), float(ins.get("top", 0.0)),
		vp.x - float(ins.get("left", 0.0)) - float(ins.get("right", 0.0)),
		vp.y - float(ins.get("top", 0.0)) - float(ins.get("bottom", 0.0)))
	_w = minf(_px(DRAWER_W_UNITS), vp.x * DRAWER_W_FRAC)
	visible = not _entries.is_empty() and _band.size.y >= _px(MIN_BAND_UNITS)
	_travel = _w if _open else clampf(_travel, 0.0, _w)
	_place()

## Closes the drawer if it is open; the shell's system-back chain calls this (proposal §4's order).
func close_if_open() -> bool:
	if not _open and _travel <= 0.0:
		return false
	_animate_to(false)
	return true

# -- layout -------------------------------------------------------------------------

func _px(units: float) -> float:
	return roundf(units * _scale)

## Type through the shell's `_pfont()`, so the phone's dynamic type (the OS font scale) applies here as it does
## to every other phone label. Boxes stay on `_px()`, because a tap target does not grow with type.
func _font_px(units: float) -> int:
	if _app != null and _app.has_method("_pfont"):
		return int(_app.call("_pfont", units))
	return int(_px(units))

func _place() -> void:
	var gw := _px(GRIP_W_UNITS)
	var gh := _px(GRIP_H_UNITS)
	var y_mid := _band.position.y + _band.size.y * 0.5
	if _side == "left":
		_panel.position = Vector2(_band.position.x - _w + _travel, _band.position.y)
		_grip.position = Vector2(_band.position.x + _travel, y_mid - gh * 0.5)
	else:
		var right_edge := _band.position.x + _band.size.x
		_panel.position = Vector2(right_edge - _travel, _band.position.y)
		_grip.position = Vector2(right_edge - _travel - gw, y_mid - gh * 0.5)
	_panel.size = Vector2(_w, _band.size.y)
	_panel.visible = _travel > 0.0
	_grip.size = Vector2(gw, gh)
	_grip.accessibility_name = "Close the quick drawer" if _open else "Open the quick drawer: pinned commands"
	_grip.queue_redraw()
	_scrim.position = _band.position
	_scrim.size = _band.size
	_scrim.color = Color(0, 0, 0, SCRIM_ALPHA * (_travel / maxf(1.0, _w)))
	_scrim.mouse_filter = Control.MOUSE_FILTER_STOP if _travel > 0.0 else Control.MOUSE_FILTER_IGNORE

## The visible tab, against the edge it opens from, with the accent bar on it.
func _draw_grip() -> void:
	var tw := _px(TAB_W_UNITS)
	var th := _px(TAB_H_UNITS)
	var x := 0.0 if _side == "left" else _grip.size.x - tw
	var tab := Rect2(x, (_grip.size.y - th) * 0.5, tw, th)
	var box := DccWidgets.box("line_soft", "panel", 0, 0)
	box.set_corner_radius_all(int(_px(14.0)))
	_grip.draw_style_box(box, tab)
	var bw := _px(BAR_W_UNITS)
	var bh := _px(BAR_H_UNITS)
	_grip.draw_rect(Rect2(tab.get_center() - Vector2(bw, bh) * 0.5, Vector2(bw, bh)),
		Color(DccTheme.c("accent"), 0.8))

# -- rows ---------------------------------------------------------------------------

## One row, styled as `QuickBarPhoneList._row()` styles its own, so the drawer and the `All…` sheet look
## like one list. A row that cannot run is dim but still tappable, so the tap can say why.
func _row(title_text: String, sub: String, live: bool, on_press: Callable) -> Button:
	var b := Button.new()
	b.focus_mode = Control.FOCUS_NONE
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.text = title_text if sub == "" else "%s\n%s" % [title_text, sub]
	b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.custom_minimum_size = Vector2(0, _px(ROW_UNITS))
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", _font_px(float(DccTheme.FS_SMALL)))
	b.add_theme_color_override("font_color", DccTheme.c("text") if live else DccTheme.c("text_ghost"))
	var rest := DccWidgets.box("line_soft" if live else "line", "", 12, 6)
	for state in ["normal", "pressed", "disabled", "focus", "hover"]:
		b.add_theme_stylebox_override(state, rest)
	b.pressed.connect(on_press)
	return b

## A pin's row: run it and close, or say why it cannot run and stay open (QuickBarPhone.press decides).
func _tap(entry: Dictionary) -> void:
	if bool(entry.get("live", false)):
		_animate_to(false)
		QuickBarPhone.press(_app, entry, null)
	else:
		QuickBarPhone.press(_app, entry, _panel)

func _customise() -> void:
	_animate_to(false)
	if _app != null and _app.has_method("open_quick_bar_window"):
		_app.call("open_quick_bar_window")

# -- gestures -------------------------------------------------------------------------

## The grip: a tap toggles, a drag follows the finger and settles open or shut on release. It reads mouse and
## touch like `tool_fan.gd`'s pill; under mouse emulation the second press of one finger is ignored.
func _on_grip_input(event: InputEvent) -> void:
	var kind := ""
	var pos := Vector2.ZERO
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		kind = "press" if mb.pressed else "release"
		pos = mb.position
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		kind = "press" if st.pressed else "release"
		pos = st.position
	elif event is InputEventMouseMotion:
		kind = "move"
		pos = (event as InputEventMouseMotion).position
	elif event is InputEventScreenDrag:
		kind = "move"
		pos = (event as InputEventScreenDrag).position
	else:
		return
	_grip.accept_event()
	var g: Vector2 = _grip.get_global_transform_with_canvas() * pos
	match kind:
		"press":
			if _drag.is_empty():
				_drag = {"x0": g.x, "from": _travel, "moved": false}
		"move":
			if _drag.is_empty():
				return
			var dx: float = g.x - float(_drag["x0"])
			if _side == "right":
				dx = -dx
			if absf(dx) > _px(SLOP_UNITS):
				_drag["moved"] = true
			if bool(_drag["moved"]):
				if _tween != null:
					_tween.kill()
				_travel = clampf(float(_drag["from"]) + dx, 0.0, _w)
				_place()
		"release":
			if _drag.is_empty():
				return
			var moved := bool(_drag["moved"])
			_drag.clear()
			if not moved:
				_animate_to(not _open)
			else:
				_animate_to(_travel > _w * OPEN_FRAC)

## A release on the scrim closes the drawer.
func _on_scrim_input(event: InputEvent) -> void:
	var released := (event is InputEventMouseButton and not (event as InputEventMouseButton).pressed) \
		or (event is InputEventScreenTouch and not (event as InputEventScreenTouch).pressed)
	if released:
		_scrim.accept_event()
		_animate_to(false)

## Slides to open or shut. A haptic tick marks an opening; `detent` is the shell's existing kind for a
## surface settling into place.
func _animate_to(open: bool) -> void:
	_open = open
	if open and _app != null and _app.has_method("_haptic"):
		_app.call("_haptic", "detent")
	if _tween != null:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_method(_set_travel, _travel, _w if open else 0.0, ANIM_S)

## The tween's setter: one frame of the slide.
func _set_travel(v: float) -> void:
	_travel = v
	_place()
