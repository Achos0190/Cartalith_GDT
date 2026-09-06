extends Node
## Ruling 20's binding condition, measured rather than reasoned: **is every
## destination the phone app bar's `☰` and `▤` reach still reachable without
## them?**
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##       _appbar20_probe.tscn -- --force-touch --nowelcome
##
## `--force-touch` is required: `_build_phone_app_bar()` is only called by
## `DccShell._build_phone_shell()`, and without it the run gets a desktop shell
## and no app bar to walk. Headless is correct here -- nothing this probe reads
## is a pixel or a frame time; every claim is a drawn string or a shell flag.
##
## Runs unchanged **before and after** the glyphs are removed: a cell that is
## gone reports ABSENT rather than failing, so one script produces both halves
## of the before/after set.
##
## Method, deliberately the same shape as `_phonemore_reach_probe.gd`: press
## the REAL buttons on the REAL bar, read the REAL flags back, then dump ground
## truth from the live `MenuBar` and drive the alternate route by feeding
## `InputEventScreenTouch` into the drawn row's own `gui_input` -- not by
## calling the handler behind it, which would prove the handler works and say
## nothing about whether a finger can reach it.

var app: Node
var pm            ## PhoneMenu
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("REACH %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button and String((c as Button).text) != "":
			out.append(String((c as Button).text))
		_labels(c, out)

func _clean(t: String) -> String:
	var s := t.strip_edges()
	if s.ends_with("▸"):
		s = s.substr(0, s.length() - 1).strip_edges()
	return s

# -- The bar ------------------------------------------------------------------

func _collect_buttons(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button:
			out.append(c)
		_collect_buttons(c, out)

func _bar_buttons() -> Array:
	var out: Array = []
	var bar: Node = app.get("_phone_app_bar")
	if bar == null:
		return out
	_collect_buttons(bar, out)
	return out

func _bar_cell(tip: String) -> Button:
	for b in _bar_buttons():
		if String((b as Button).accessibility_name) == tip:
			return b
	return null

## Every string the bar draws, in tree order.
func _bar_strings() -> Array:
	var out: Array = []
	var bar: Node = app.get("_phone_app_bar")
	if bar != null:
		_labels(bar, out)
	return out

func _state() -> String:
	var picker: Node = app.get("_phone_panel_picker")
	return "left=%s right=%s picker=%s" % [
		str(bool(app.get("_left_sheet_open"))),
		str(bool(app.get("_right_sheet_open"))),
		"absent" if picker == null else str((picker as Control).visible)]

func _reset() -> void:
	app.call("_close_all_phone_overlays")

# -- Driving a drawn row ------------------------------------------------------

## The `PanelContainer` row inside `root` whose own labels carry `title`.
func _row_named(root: Node, title: String) -> Control:
	if root is PanelContainer:
		var mine: Array = []
		_labels(root, mine)
		for t in mine:
			if _clean(String(t)) == title:
				return root as Control
	for c in root.get_children():
		var hit := _row_named(c, title)
		if hit != null:
			return hit
	return null

## Press and release at the row's own centre, through `gui_input` -- the path a
## finger takes (`PhoneMenu._row_input`), not the callable behind it.
func _tap(row: Control) -> void:
	var at := row.size * 0.5
	for pressed in [true, false]:
		var ev := InputEventScreenTouch.new()
		ev.index = 0
		ev.pressed = pressed
		ev.position = at
		row.gui_input.emit(ev)

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _menu_popup(title: String) -> PopupMenu:
	for child in app.menu_bar_row.get_children():
		if child is MenuButton and String(child.text) == title:
			var p := (child as MenuButton).get_popup()
			p.about_to_popup.emit()
			return p
	return null

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	## `--size WxH`, defaulting to the tested handset. **One density per
	## launch**: `phone_scale()` is read once in `setup()` and nothing re-reads
	## it on a resize, so three sizes in one process is one layout measured
	## against three widths -- the shape of a measurement without being one
	## (`_phonemore_reach_probe.gd`'s own note, learned the same way).
	var parts := _arg("--size", "1080x2400").split("x")
	var want := Vector2i(int(parts[0]), int(parts[1]))
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

	if not app.is_phone():
		print("REACH  !! not a phone shell -- run with -- --force-touch")
		get_tree().quit(1)
		return
	pm = app.get("_phone_menu")
	print("REACH phone_scale=%.3f  palette=%s" % [app.phone_scale(), "dark" if DccTheme.is_dark() else "light"])

	## -- What the bar draws now ----------------------------------------------
	var tips := PackedStringArray()
	for b in _bar_buttons():
		var glyph := String((b as Button).text)
		tips.append("%s[%s]" % [String((b as Button).accessibility_name),
			glyph if glyph != "" else "icon"])
	print("REACH bar cells: %s" % " · ".join(tips))
	var drawn := PackedStringArray()
	for s in _bar_strings():
		drawn.append(String(s))
	print("REACH bar strings: %s" % " | ".join(drawn))

	## The row is one cell narrower than it was, so it cannot have started
	## overflowing -- but "cannot have" is the reasoning this repository keeps
	## catching, so measure it. A `Control` whose combined minimum exceeds the
	## screen has no scrollbar here to reveal it; it just pushes its siblings
	## off the edge.
	var bar := app.get("_phone_app_bar") as Control
	var min_w: float = 0.0 if bar == null else bar.get_combined_minimum_size().x
	_check("app bar fits the screen width", bar != null and min_w <= float(want.x),
		"min_w=%.0f  screen=%d  (%dx%d)" % [min_w, want.x, want.x, want.y])

	## -- BEFORE: what each glyph reaches -------------------------------------
	for tip in ["Domain panel", "Panels"]:
		var b := _bar_cell(String(tip))
		if b == null:
			print("REACH glyph '%s' ABSENT" % tip)
			continue
		_reset()
		await _frames(2)
		b.pressed.emit()
		await _frames(2)
		print("REACH glyph '%s' -> %s" % [tip, _state()])

	## The picker is a router, so its own rows are the destinations, not it.
	var picker: Node = app.get("_phone_panel_picker")
	if picker == null:
		print("REACH picker ABSENT")
	else:
		for row_title in ["Left panel", "Right panel"]:
			_reset()
			app.call("_set_panel_picker_open", true)
			await _frames(2)
			## `_phone_list_row()` builds a `Button` whose drawn title is
			## `to_upper()`ed and whose `accessibility_name` is the un-cased
			## string -- so the row is NOT the `PanelContainer` shape
			## `_row_named()` finds in `PhoneMenu`, and matching it by drawn
			## text would match "LEFT PANEL", not "Left panel".
			var rows: Array = []
			_collect_buttons(picker, rows)
			var row: Button = null
			for candidate in rows:
				if String((candidate as Button).accessibility_name) == String(row_title):
					row = candidate as Button
					break
			if row == null:
				_check("picker row '%s' is drawn" % row_title, false)
				continue
			row.pressed.emit()
			await _frames(2)
			print("REACH picker '%s' -> %s" % [row_title, _state()])

	## -- Ground truth from the live MenuBar ----------------------------------
	var win := _menu_popup("Window")
	if win == null:
		_check("Window menu exists on the live MenuBar", false)
	else:
		var rows := PackedStringArray()
		for i in win.item_count:
			if win.is_item_separator(i):
				continue
			rows.append("%s%s%s" % [_clean(win.get_item_text(i)),
				"(disabled)" if win.is_item_disabled(i) else "",
				"[on]" if win.is_item_checkable(i) and win.is_item_checked(i) else ""])
		print("REACH MenuBar Window rows: %s" % " · ".join(rows))

	## -- ALTERNATE: MORE > Window > Left dock / Right dock --------------------
	if pm == null:
		_check("PhoneMenu exists", false)
	else:
		for pair in [["Left dock", "_left_sheet_open"], ["Right dock", "_right_sheet_open"]]:
			var title := String(pair[0])
			var flag := String(pair[1])
			_reset()
			await _frames(2)
			pm.open()
			await _frames(3)
			var root_texts: Array = []
			_labels(pm._screen_body, root_texts)
			var root_row := _row_named(pm._screen_body, "Window")
			_check("MORE root draws a 'Window' row", root_row != null,
				"root drew %d strings" % root_texts.size())
			if root_row == null:
				continue
			_tap(root_row)
			await _frames(3)
			var win_texts: Array = []
			_labels(pm._screen_body, win_texts)
			var cleaned := PackedStringArray()
			for t in win_texts:
				cleaned.append(_clean(String(t)))
			print("REACH MORE > Window drew: %s" % " | ".join(cleaned))
			var dock_row := _row_named(pm._screen_body, title)
			_check("MORE > Window draws '%s'" % title, dock_row != null)
			if dock_row == null:
				continue
			var was := bool(app.get(flag))
			_tap(dock_row)
			await _frames(3)
			var now := bool(app.get(flag))
			_check("MORE > Window > %s opens the sheet" % title, now and not was,
				"%s: %s -> %s   %s" % [flag, str(was), str(now), _state()])

	_reset()
	await _frames(2)
	print("REACH failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
