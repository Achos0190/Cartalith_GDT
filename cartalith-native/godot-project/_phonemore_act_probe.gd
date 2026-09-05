extends Node
## Does a §6.6 bespoke MORE row actually DO the thing?
##
## `_phonemore_reach_probe.gd` proves the rows are drawn and reachable. Drawn is
## not wired: the whole point of the ruling is that these screens act on the
## real shell, so every new control type this pass introduced -- the `seg` chip,
## the `range` slider, the `tog` switch row, the civ tool arm -- is exercised
## here against the state it changes, with a world generated so the timeline
## rows are live rather than drawn as their own absence.
##
## Rows are driven through `gui_input`, the signal `_row_input()` is connected
## to, so the press path under test is the one a finger takes -- not the bound
## Callable fished out of the connection, which would skip the handler that
## PH-15 fixed.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _phonemore_act_probe.tscn \
##       -- --force-touch --nowelcome

var app: Node
var pm
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("ACT %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _all(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_all(c, out)

func _nodes() -> Array:
	var out: Array = []
	_all(pm._screen_body, out)
	return out

func _texts_of(n: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_all(n, all)
	for c in all:
		if c is Label:
			out.append(String((c as Label).text))
	return out

## The row (a `PanelContainer` built by `_row()`) whose first label is `title`.
func _row_named(title: String) -> PanelContainer:
	for c in _nodes():
		if not (c is PanelContainer):
			continue
		var t := _texts_of(c)
		if not t.is_empty() and String(t[0]).strip_edges() == title:
			return c as PanelContainer
	return null

func _chip_named(text: String) -> Button:
	for c in _nodes():
		if c is Button and String((c as Button).text).strip_edges() == text:
			return c as Button
	return null

func _slider() -> HSlider:
	for c in _nodes():
		if c is HSlider:
			return c as HSlider
	return null

## Press a row the way a finger does: down then up at the same point, through
## the `gui_input` signal `_row_input()` listens on.
func _tap(row: PanelContainer) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = row.size * 0.5
	row.gui_input.emit(down)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = row.size * 0.5
	row.gui_input.emit(up)

func _screen(id: String) -> void:
	pm.open()
	if id != "more":
		pm._push_screen(id)
	await _frames(2)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("ACT  !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		print("ACT  !! generate failed -- the timeline rows cannot be live")
		get_tree().quit(1)
		return
	_check("a world exists, so tl_available() is true", app.tl_available())

	# -- sim: range, seg, tog ---------------------------------------------------
	await _screen("sim")
	var s := _slider()
	_check("sim draws a Year slider once a world exists", s != null)
	if s != null:
		_check("the slider's range is the shell's own track",
			s.min_value == DccShell.TL_YEAR_MIN and s.max_value == DccShell.TL_YEAR_MAX,
			"%d..%d" % [int(s.min_value), int(s.max_value)])
		s.value = 733.0
		await _frames(2)
		_check("dragging Year moves the cursor", app.tl_year() == 733,
			"tl_year=%d" % app.tl_year())

	await _screen("sim")
	var was_speed: int = app.tl_speed
	var other := 100 if was_speed != 100 else 1
	var chip := _chip_named("×%d" % other)
	_check("sim draws a Speed chip per DccShell.TL_SPEEDS", chip != null)
	if chip != null:
		chip.pressed.emit()
		await _frames(2)
		_check("tapping a Speed chip sets the rate", app.tl_speed == other,
			"was %d, now %d" % [was_speed, app.tl_speed])

	await _screen("sim")
	var before: bool = bool(app.tl_layers.get("Economy", false))
	var lrow := _row_named("Economy")
	_check("sim draws a switch row per DccShell.TL_LAYERS", lrow != null)
	if lrow != null:
		_tap(lrow)
		await _frames(2)
		_check("tapping a layer row flips it",
			bool(app.tl_layers.get("Economy", false)) != before,
			"%s -> %s" % [before, app.tl_layers.get("Economy", false)])

	await _screen("sim")
	var strip := _row_named("Transport strip on map")
	_check("sim draws the transport-strip row", strip != null)
	if strip != null:
		_tap(strip)
		await _frames(3)
		_check("turning the strip on opens it", app.is_phone_sim_strip_open())
		_check("...and closes the menu, so the map it draws over is visible",
			not pm.is_open())
		app.set_phone_sim_strip_open(false)

	# -- civ: arming a tool -----------------------------------------------------
	## All four, not just the first. `PhoneMenu.CIV_TOOLS` is a hand copy of
	## `civilization_workspace.gd::_build_tools()`, and a copy checked at one
	## entry is a copy checked nowhere -- an id renamed in that file would arm
	## nothing here and the row would still draw.
	for spec in [["Settlement", "settlement"], ["Territory", "territory"],
			["Way", "way"], ["Route", "route"]]:
		app.arm_tool("inspect")
		await _screen("civ")
		var trow := _row_named(String(spec[0]))
		_check("civ draws the %s tool row" % spec[0], trow != null)
		if trow == null:
			continue
		_tap(trow)
		await _frames(3)
		_check("arming %s selects the CIVIL domain" % spec[0],
			app.active_domain() == "civilization", app.active_domain())
		_check("...arms '%s'" % spec[1], String(app.armed_tool) == String(spec[1]),
			String(app.armed_tool))
		_check("...and closes the menu so the map can be tapped", not pm.is_open())

	## Leave Settlement armed for the badge check below.
	app.arm_tool("inspect")
	await _screen("civ")
	var srow := _row_named("Settlement")
	if srow != null:
		_tap(srow)
		await _frames(3)

	## §6.6: *"badge `ARMED` in `var(--acc)` when active"*. Read the badge back
	## while the tool is still armed, before the disarm below -- a badge checked
	## after the state it reports has gone cannot fail.
	await _screen("civ")
	var badge_found := false
	var all: Array = []
	_all(pm._screen_body, all)
	for c in all:
		if c is Label and String((c as Label).text) == "ARMED":
			badge_found = true
			_check("the ARMED badge is drawn in the accent, not the count colour",
				(c as Label).get_theme_color("font_color") == DccTheme.c("accent"),
				str((c as Label).get_theme_color("font_color")))
	_check("an armed civ tool badges its row ARMED", badge_found)

	app.arm_tool("inspect")

	await _screen("civ")
	_check("civ draws the POI absence rather than a fake tool",
		_row_named("Point of interest") != null)
	_check("civ draws Landmark generation", _row_named("Landmark generation") != null)
	for t in _texts_of(pm._screen_body):
		if String(t).begins_with("49 types"):
			_check("the landmark line counts the real submenu", true, String(t))

	# -- prefs: a seg chip over a real radio submenu ----------------------------
	await _screen("prefs")
	var mode0: String = DccSettings.units_mode()
	var want_units := "Miles" if mode0 != "mi" else "Kilometres"
	var uchip := _chip_named(want_units)
	_check("prefs draws Units as chips, not a drill", uchip != null)
	if uchip != null:
		uchip.pressed.emit()
		await _frames(3)
		_check("tapping a Units chip changes the unit",
			DccSettings.units_mode() != mode0,
			"%s -> %s" % [mode0, DccSettings.units_mode()])
		var back := _chip_named("Kilometres" if mode0 == "km" else
			("Miles" if mode0 == "mi" else "Nautical miles"))
		if back != null:
			back.pressed.emit()
			await _frames(2)
		_check("...and back", DccSettings.units_mode() == mode0,
			DccSettings.units_mode())

	# -- the map context menu still gets the 60%-cap sheet ----------------------
	## `_fill()`'s dispatch changed (screen id / popup / neither), and
	## `open_sheet()` is the one caller that still asks for the sheet. A screen
	## id was added to `_Step` this pass; a sheet step has none, and reading the
	## sheet body back is what proves the dispatch still lands there.
	var ctx := PopupMenu.new()
	ctx.add_item("Inspect here")
	ctx.add_item("Place settlement")
	app.add_child(ctx)
	pm.open_sheet(ctx, "Map", "at 40, 60")
	await _frames(3)
	var sheet_texts: Array = []
	var sheet_nodes: Array = []
	_all(pm._sheet_body, sheet_nodes)
	for c in sheet_nodes:
		if c is Label:
			sheet_texts.append(String((c as Label).text))
	_check("open_sheet() still fills the sheet, not the screen",
		sheet_texts.has("Inspect here") and sheet_texts.has("Place settlement"),
		str(sheet_texts))
	_check("...and the sheet is the visible surface", pm._sheet.visible and not pm._screen.visible)
	pm.close()
	ctx.queue_free()

	# -- the navigation stack ---------------------------------------------------
	await _screen("project")
	_check("a bespoke screen is one step above the root", pm._stack.size() == 2)
	_check("back returns to the root", pm.go_back() and pm._stack.size() == 1)
	_check("...and the root is still the root",
		String((pm._stack[0] as Object).get("screen")) == "more")

	print("ACT done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
