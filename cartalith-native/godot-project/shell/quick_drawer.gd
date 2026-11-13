extends Control
## MS-11 (`MOBILE_SHELL_PROPOSAL.md` section 2.4, Ruling DY Q10 to Q12): the phone's quick drawer. The quick bar's
## pins in a side drawer that slides over the map from a grip at the middle of the map band's edge. Portrait: on the side
## AWAY from the dominant hand (left for a right-handed user). **Landscape: the screen's outer edge on the dominant-hand
## side** (right-handed: far right; left-handed: far left, beside the rail; owner, 2026-11-13). It replaces CQ-6's chip row in the MAP sheet.
##
## Open: tap the grip, or drag it inward. Close: tap the scrim, drag the grip back, or system back (the shell's
## back chain asks [`close_if_open`]).
##
## ## One pin list, no second runner (`CUSTOM_QUICK_BAR_SCOPE.md` section 0 rule 1)
##
## The drawer draws the entries it is handed by `QuickBarPhone.refresh()` (`QuickBarPhone.entries`, the same
## answer MENU's Quick section reads) and presses through `QuickBarPhone.press()`, which calls
## `DccQuickBar.run_pin_on()`: the very path the desktop bar's chip takes, resolving the stored `pin_key` against
## a fresh `CommandIndex` and emitting the owning popup's own `id_pressed`. Nothing here knows what a command does.
##
## ## Why a visible grip and not an edge swipe
##
## With Android's gesture navigation a swipe that starts at a screen edge is the system's Back, while a tap on a
## visible control is always delivered to the app. The grip is therefore a drawn tab, never an edge zone.
##
## ## The world pauses while it is open (depth, not a flag)
##
## An open drawer is a context surface like the card and the ring: it calls `app.pause_for_context_surface()` on
## opening and `resume_from_context_surface()` on closing (`app.gd`, ref-counted, so it composes with a card that
## is open at the same time). The depth is released on EVERY way out, including the drawer being freed while open
## (a class swap, [`release_surface`]) and the band becoming too short to draw it: an unbalanced pause would
## freeze timeline playback for good.
##
## ## What it must never do
##
## Draw anything while there is nothing to show (the shell never mounts it then: `QuickBarPhone.refresh()` frees
## it; CQ-6's pixel-identity bar). Hold the pause after it has closed. Run a command itself.
##
## No `class_name`: loaded by path from `quick_bar_phone.gd`, so it needs no new class-cache entry.

## Sizes are in phone units (the shell's `_pscale`, one unit = 1 dp at a 412 dp phone), applied once through
## `phone_px()`; floors go in before that multiply, which is `_ptap()`'s lesson.
## The grip's tap target is 44 x 88 (the proposal's figure; PHONE_TAP_MIN is 44), and never less than 44 x 88
## REAL dp: on a 360 dp phone one unit is 0.87 dp, so a unit-sized target would land under the floor
## (see [`_grip_px`]).
const GRIP_W_DP := 44.0
const GRIP_H_DP := 88.0
## The visible tab inside the target, against the edge, and the accent bar on it (the mockup's figures).
const TAB_W_UNITS := 22.0
const TAB_H_UNITS := 72.0
const BAR_W_UNITS := 4.0
const BAR_H_UNITS := 34.0
## The drawer's width (the mockup's `QD_W`) and the share of the band it may never exceed.
const DRAWER_W_UNITS := 248.0
const DRAWER_W_FRAC := 0.78
## A row's height: `H_PHONE_ROW`, the canvas's list row, which is above the 44 floor.
const ROW_UNITS := 52.0
## A drag that ends past this share of the width settles open (a labelled judgement: just under half).
const OPEN_FRAC := 0.4
## Below this a press on the grip is a tap, not a drag (a labelled judgement; Android's touch slop is 8 dp).
const SLOP_UNITS := 6.0
## No drawer when the map's free band is shorter than this: the open panel would be a few rows of nothing, and the
## sheet's FULL detent leaves one tap target of band (`phone_content_insets()`).
const MIN_BAND_UNITS := 180.0
const SCRIM_ALPHA := 0.35
const ANIM_S := 0.18

var _app
var _entries: Array = []
var _side := "left"
var _w := 0.0
## How far the panel is out, 0 .. `_w`, in px of this node's own space (the band).
var _travel := 0.0
## Where the drawer is heading or resting: the ONE flag the pause depth follows.
var _open := false
## Whether this drawer currently holds one level of the app's context-surface pause.
var _holding := false
var _drag: Dictionary = {}
var _tween: Tween
## The armed-tool strip's band (device px) the last time the layout was read; `_process` re-lays out when it moves.
var _reserve := 0.0

var _scrim: ColorRect
var _panel: PanelContainer
var _list: VBoxContainer
var _grip: Control

## Once, after `new()`: builds the scrim, the panel and the grip. `relayout()` is called once the node is in the
## tree. Listens for the phone's inset changes (rotation, sheet changes) and for the dominant hand.
func setup(app) -> void:
	_app = app
	name = "QuickDrawer"
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	## The band is this node's rect, so a sliding panel is cut at the band's edge instead of drawing over the
	## landscape rail or the status area while it moves.
	clip_contents = true
	_scrim = ColorRect.new()
	_scrim.name = "Scrim"
	_scrim.color = Color(0, 0, 0, 0)
	_scrim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_scrim.gui_input.connect(_on_scrim_input)
	add_child(_scrim)
	_panel = PanelContainer.new()
	_panel.name = "Panel"
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
	_grip.name = "Grip"
	_grip.mouse_filter = Control.MOUSE_FILTER_STOP
	_grip.gui_input.connect(_on_grip_input)
	_grip.draw.connect(_draw_grip)
	add_child(_grip)
	if _app != null and _app.has_signal("phone_insets_changed"):
		_app.phone_insets_changed.connect(relayout)
	DccSettings.on_hand_changed(relayout)

## New entries from `QuickBarPhone.refresh()`: rebuilds the rows. The caller frees the drawer when the list is
## empty, so an empty bar draws nothing.
func set_entries(es: Array) -> void:
	_entries = es
	for c in _list.get_children():
		_list.remove_child(c)
		c.queue_free()
	_list.add_child(DccTheme.mono_label("QUICK  %d" % es.size(), "text_dim", _font_px(10.0)))
	for i in es.size():
		var entry := es[i] as Dictionary
		var b := _row(String(entry["title"]), QuickBarPhoneList._sub_for(entry), bool(entry["live"]), _tap.bind(entry))
		## A pin key holds a colon, which a node name cannot: the row is named by position, and carries its entry.
		b.name = "Assistant" if String(entry["kind"]) == "assistant" else "Pin%d" % i
		b.set_meta("entry", entry)
		_list.add_child(b)
	var cust := _row("Customise…", "Pin, unpin and reorder commands", true, _customise)
	cust.name = "Customise"
	_list.add_child(cust)

## The entries last drawn (for tests).
func entries() -> Array:
	return _entries

## Whether the drawer is open or opening (for tests and the shell's back chain).
func is_open() -> bool:
	return _open

## The grip (for tests): the one tap target that exists while the drawer is closed.
func grip() -> Control:
	return _grip

## The scrim (for tests).
func scrim() -> Control:
	return _scrim

## The row buttons in order, without the heading (for tests).
func rows() -> Array[Button]:
	var out: Array[Button] = []
	for c in _list.get_children():
		if c is Button:
			out.append(c as Button)
	return out

## Re-reads the scale, the side and the map's free band, then places everything. Runs on `phone_insets_changed`
## (rotation, a sheet detent settling), when the dominant hand changes, and when the armed-tool strip appears or
## goes (polled in `_process`: the strip raises no signal). The band is the map's free band MINUS the strip's
## reserve: the strip is part of the chrome column, which is drawn above this node, so a panel reaching down
## behind it would leave its last rows untouchable (found by `_ms11_probe.gd` in landscape with a tool armed).
## If the band has become too short to draw the drawer while it is open, it closes (and releases the pause)
## rather than hiding itself open.
func relayout() -> void:
	if _app == null or not is_inside_tree():
		return
	_reserve = float(_app.call("phone_tool_pill_reserve"))
	## Landscape (owner, 2026-11-13, superseding "next to the camera": *"The button for the side menu now lands in the
	## middle of the screen in landscape mode, let's move this all the way left/right depending handedness"*): the
	## grip sits at the screen's OUTER edge on the dominant-hand side, the same setting the portrait grip reads but
	## mirrored (portrait puts it AWAY from the hand, landscape puts it AT the hand). Left-handed: the band's left
	## edge, which is the rail's right edge, so beside the rail and not under it. Right-handed: the screen's far
	## right edge, which is inside the docked sheet's column (see `relayout`'s band). Portrait keeps MS-11's rule.
	var land := bool(_app.call("phone_is_landscape"))
	_side = ("left" if DccSettings.dominant_hand() == "left" else "right") if land else 		("right" if DccSettings.dominant_hand() == "left" else "left")
	var vp := get_viewport_rect().size
	var ins: Dictionary = _app.call("phone_content_insets")
	var band := Rect2(float(ins["left"]), float(ins["top"]),
		vp.x - float(ins["left"]) - float(ins["right"]),
		vp.y - float(ins["top"]) - float(ins["bottom"]) - _reserve)
	## Landscape, right-handed: the band is stretched over the sheet's column to the screen's right edge, so the grip
	## and the sliding panel start at the edge itself (the band's own right edge is the sheet's left edge, mid-screen,
	## which is what the owner rejected). The node rides above the sheet (`z_index`) in that one case so the grip is
	## not hidden under it; every other case keeps the node below the sheet where `set_phone_quick_drawer` put it.
	var over_sheet := land and _side == "right"
	if over_sheet:
		band.size.x = vp.x - band.position.x
		## Below the app bar, which sits top-right above the sheet and is drawn over the drawer: a panel under it
		## would have rows no finger can reach (found by `_ms11_probe.gd`'s pin rows in landscape).
		var cut: float = (_app.call("phone_land_column_gap") as Vector2).x - band.position.y
		band.position.y += cut
		band.size.y -= cut
	z_index = 1 if over_sheet else 0
	position = band.position
	size = band.size
	_w = minf(_px(DRAWER_W_UNITS), band.size.x * DRAWER_W_FRAC)
	var fits := not _entries.is_empty() and band.size.y >= _px(MIN_BAND_UNITS)
	if not fits and _open:
		_set_open(false)
		_travel = 0.0
	visible = fits
	_travel = _w if _open and _tween == null else clampf(_travel, 0.0, _w)
	_place()

## The shell's system-back chain calls this (proposal section 4's order). True when it closed something.
func close_if_open() -> bool:
	if not _open:
		return false
	_animate_to(false)
	return true

func _process(_delta: float) -> void:
	if _app != null and absf(float(_app.call("phone_tool_pill_reserve")) - _reserve) > 0.5:
		relayout()

## Opens the drawer with no slide and no haptic: a class swap that re-presents it (`QuickBarPhone.represent_restore`),
## and the probes. The pause depth is taken exactly as a real opening takes it.
func open_now() -> void:
	if _tween != null:
		_tween.kill()
		_tween = null
	_set_open(true)
	_travel = _w
	_place()

## Releases the context-surface pause if this drawer holds it. Called before the drawer is freed, because a freed
## open drawer would otherwise leave the world paused for good. Safe to call twice.
func release_surface() -> void:
	if _tween != null:
		_tween.kill()
		_tween = null
	_set_open(false)

func _exit_tree() -> void:
	if _holding and _app != null and is_instance_valid(_app):
		_holding = false
		_app.resume_from_context_surface()

# -- layout -------------------------------------------------------------------------

func _px(units: float) -> float:
	return float(_app.call("phone_px", units))

## Type through the shell's `phone_font()`, so the OS font scale applies here as to every phone label. Boxes stay
## on `_px()`: a tap target does not grow with type.
func _font_px(units: float) -> int:
	return int(_app.call("phone_font", units))

## The grip's tap target in px: the proposal's 44 x 88 phone units, raised to 44 x 88 real dp where the device's
## density says a unit is smaller than a dp (a 360 dp phone). The density is the shell's own (`_density()`);
## when it is only a legacy fallback the unit figure stands, which is never smaller than the old floor.
func _grip_px() -> Vector2:
	var d := float((_app.call("_density") as Dictionary)["value"])
	return Vector2(maxf(_px(GRIP_W_DP), roundf(GRIP_W_DP * d)), maxf(_px(GRIP_H_DP), roundf(GRIP_H_DP * d)))

func _place() -> void:
	var g := _grip_px()
	var y_mid := size.y * 0.5
	## Landscape, right-handed, the grip lives in the docked sheet's column (the far right edge): centred in the free
	## strip between the app bar and the sheet's top edge, where nothing is drawn, so it overlaps neither. When the
	## sheet is open there is no strip, and it is centred on the column instead, floating over the sheet's edge
	## (a labelled trade: an unreachable drawer is worse). Otherwise the band's middle, as in portrait.
	if bool(_app.call("phone_is_landscape")) and _side == "right":
		var gap: Vector2 = _app.call("phone_land_column_gap")
		if gap.y - gap.x >= g.y:
			y_mid = (gap.x + gap.y) * 0.5 - position.y
	if _side == "left":
		_panel.position = Vector2(_travel - _w, 0.0)
		_grip.position = Vector2(_travel, y_mid - g.y * 0.5)
	else:
		_panel.position = Vector2(size.x - _travel, 0.0)
		_grip.position = Vector2(size.x - _travel - g.x, y_mid - g.y * 0.5)
	_panel.size = Vector2(_w, size.y)
	_panel.visible = _travel > 0.0
	_grip.size = g
	_grip.accessibility_name = "Close the quick drawer" if _open else "Open the quick drawer: pinned commands"
	_grip.queue_redraw()
	_scrim.position = Vector2.ZERO
	_scrim.size = size
	_scrim.color = Color(0, 0, 0, SCRIM_ALPHA * (_travel / maxf(1.0, _w)))
	_scrim.mouse_filter = Control.MOUSE_FILTER_STOP if _travel > 0.0 else Control.MOUSE_FILTER_IGNORE

## The visible tab against the edge it opens from, with the accent bar on it.
func _draw_grip() -> void:
	var tw := _px(TAB_W_UNITS)
	var th := _px(TAB_H_UNITS)
	var x := 0.0 if _side == "left" else _grip.size.x - tw
	var tab := Rect2(x, (_grip.size.y - th) * 0.5, tw, th)
	var box := DccWidgets.box("line_soft", "panel", 0, 0)
	box.set_corner_radius_all(int(_px(14.0)))
	_grip.draw_style_box(box, tab)
	var bar := Vector2(_px(BAR_W_UNITS), _px(BAR_H_UNITS))
	_grip.draw_rect(Rect2(tab.get_center() - bar * 0.5, bar), Color(DccTheme.c("accent"), 0.8))

# -- rows ---------------------------------------------------------------------------

## One row, styled as `QuickBarPhoneList._row()` styles its own so the drawer and MENU's list read as one list. A
## row that cannot run is dim but still tappable, so the tap can say why (a phone has no tooltip).
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
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright") if live else DccTheme.c("text_ghost"))
	var rest := DccWidgets.box("line_soft" if live else "line", "", 12, 6)
	for state in ["normal", "pressed", "disabled", "focus", "hover"]:
		b.add_theme_stylebox_override(state, rest)
	b.set_meta("live", live)
	b.pressed.connect(on_press)
	return b

## A pin's row: a live one closes the drawer and then runs through `QuickBarPhone.press` (the desktop bar's path);
## one that cannot run stays open and toasts why.
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

## The grip: a tap toggles, a drag follows the finger and settles open or shut on release. A touch on a handset
## arrives as a touch event AND its emulated mouse event, so the second press of one finger is ignored while a
## gesture is live, as `tool_fan.gd`'s pill does.
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
	var gx: float = (_grip.get_global_transform_with_canvas() * pos).x
	match kind:
		"press":
			if _drag.is_empty():
				_drag = {"x0": gx, "from": _travel, "moved": false}
		"move":
			if _drag.is_empty():
				return
			var dx: float = gx - float(_drag["x0"])
			if _side == "right":
				dx = -dx
			if absf(dx) > _px(SLOP_UNITS):
				_drag["moved"] = true
			if bool(_drag["moved"]):
				if _tween != null:
					_tween.kill()
					_tween = null
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

## Slides to open or shut. A haptic tick marks an opening (`detent`, the shell's kind for a surface settling).
func _animate_to(open: bool) -> void:
	_set_open(open)
	if open and _app != null and _app.has_method("_haptic"):
		_app.call("_haptic", "detent")
	if _tween != null:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_method(_set_travel, _travel, _w if open else 0.0, ANIM_S)
	_tween.finished.connect(func() -> void: _tween = null)

func _set_travel(v: float) -> void:
	_travel = v
	_place()

## The ONE place `_open` changes, so the pause depth follows it exactly: taken on false -> true, given back on
## true -> false, never twice either way.
func _set_open(open: bool) -> void:
	_open = open
	if open and not _holding and _app != null:
		_holding = true
		_app.pause_for_context_surface()
	elif not open and _holding and _app != null:
		_holding = false
		_app.resume_from_context_surface()
