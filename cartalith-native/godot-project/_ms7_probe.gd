extends Node
## MS-7 probe (`MOBILE_SHELL_PROPOSAL.md` section 2.2 and section 6's MS-7 row): the tablet's docking rule and its
## menu bar, in the real shell, in a `--vp`-sized host SubViewport with a real density, with REAL touch events
## (an InputEventScreenTouch and its emulated mouse, as `_ms5b_probe.gd` pushes them). Every expected value is a
## literal written by the runner on the command line or in this file, never read back from the code under test
## (`MISTAKES.md`, "Write a test that pins a constant").
##
## Run (windowed, never `--headless`; a wiped user dir; the runner holds `C:/cw/.win_lock`):
##   godot --path . --resolution 1600x1000 _ms7_probe.tscn -- --force-touch --vp 3392x2400 --density 2 \
##         --expect-left docked --expect-right docked --expect-bar whole
## Arguments read by THIS file: `--vp WxH`, `--expect-left docked|floating`, `--expect-right docked|floating`,
## `--expect-bar whole|short`, `--pure-only` (the literal tables alone, for the mutation runs). Accepted because the shell reads them: `--density D`, `--ui-scale S`,
## `--force-touch`. Anything else aborts with exit 2.
##
## What it asserts, per run:
##   1. the tablet class (never phone, never desktop);
##   2. `DccTheme.tablet_dock_plan()`'s and `tablet_bar_whole()`'s truth table, on literals (once per run);
##   3. each dock is docked or floating exactly as the runner says; a docked dock has its own width; a floating one
##      is closed to start with, its slot is the 44 dp tab and the tab is at least 44 x 44 dp;
##   4. THE FLOOR: the map is at least 520 dp wide in every state, including with a floating dock opened over it
##      (opening a floating dock must not move the map by one pixel: the map's own width is compared before and
##      after);
##   5. a real tap on a floating dock's tab opens it over the map, a real tap on the map closes it, and a tap inside
##      the open dock does not;
##   6. the menu bar: the titles are the whole bar's eight, or the short bar's five, as the runner says, every
##      title is at least 44 dp tall, and the bar's own minimum width fits the viewport (nothing is clipped);
##   7. the documented menu-open time: File's popup, shown and drawn, in milliseconds (recorded, not judged:
##      the performance pass is a later task).
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument, 3 watchdog.

var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _args_ok() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density", "--ui-scale", "--expect-left", "--expect-right", "--expect-bar"]:
			i += 2
		elif a == "--pure-only":
			i += 1
		else:
			print("ABORT: unknown argument ", a)
			return false
	return true

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("RESULT: FAIL (watchdog)")
		get_tree().quit(3))
	wd.start()
	if not _args_ok():
		get_tree().quit(2)
		return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2 or _arg("--expect-left", "") == "" or _arg("--expect-right", "") == "" or _arg("--expect-bar", "") == "":
		print("ABORT: needs --vp WxH --expect-left --expect-right --expect-bar")
		get_tree().quit(2)
		return
	var vw := int(parts[0])
	var vh := int(parts[1])
	_pure()
	if "--pure-only" in OS.get_cmdline_user_args():
		print("RESULT: %s (%d failed, %d checks) pure only" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
		get_tree().quit(0 if _fails == 0 else 1)
		return
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(8)
	await _shell(vw, vh)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

## The pure functions, on literals. Rail 48 and the docks 400 are the landscape tablet's own figures; 232 is the
## portrait figure; every row is worked by hand in the comment beside it.
func _pure() -> void:
	var plan := func(row: float, fixed: float, l: float, r: float) -> String:
		var p: Dictionary = DccTheme.tablet_dock_plan(row, fixed, l, r)
		return "%s/%s/%d" % ["D" if p["left"] else "F", "D" if p["right"] else "F", int(p["map"])]
	_ok(plan.call(1696.0, 48.0, 400.0, 400.0) == "D/D/848", "plan 1696: both dock, map 848 (1696-48-800)")
	_ok(plan.call(1280.0, 48.0, 400.0, 400.0) == "D/F/788", "plan 1280: left docks, right floats, map 788 (1280-48-400-44)")
	_ok(plan.call(900.0, 48.0, 232.0, 232.0) == "D/F/576", "plan 900 portrait: left docks, right floats, map 576 (900-48-232-44)")
	_ok(plan.call(900.0, 48.0, 400.0, 400.0) == "F/F/764", "plan 900 with 400 docks: both float, map 764 (900-48-88)")
	## The boundary, one dp either side: both dock at exactly 520 of map, not at 519.
	_ok(plan.call(1368.0, 48.0, 400.0, 400.0) == "D/D/520", "plan: a map of exactly 520 keeps both docked")
	_ok(plan.call(1367.0, 48.0, 400.0, 400.0) == "D/F/875", "plan: a map of 519 floats the right dock")
	_ok(DccTheme.tablet_bar_whole(1180.0) and not DccTheme.tablet_bar_whole(1179.0), "bar: whole from 1180, short at 1179")
	_ok(not DccTheme.tablet_bar_whole(1190.0, 0) and DccTheme.tablet_bar_whole(1196.0, 0), "bar: a short bar needs 1196 (1180 + 16) to turn whole")
	_ok(DccTheme.tablet_bar_whole(1170.0, 1) and not DccTheme.tablet_bar_whole(1163.0, 1), "bar: a whole bar holds to 1164 (1180 - 16)")

func _row_titles() -> Array:
	var out: Array = []
	if _app.menu_bar_row == null:
		return out
	for c in _app.menu_bar_row.get_children():
		if c is MenuButton:
			out.append((c as MenuButton).text)
	return out

## `p` is in dp (canvas units, what `get_global_rect()` answers); the host viewport takes physical px, so it is
## scaled by the host's own final transform (the tablet's content scale, density x UI scale). Tapping at the dp
## figure itself landed on the wrong control: the first run of this probe did exactly that.
func _tap(dp_point: Vector2) -> void:
	var p := dp_point * _vp.get_final_transform().x.x
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)
	var u := InputEventScreenTouch.new()
	u.index = 0
	u.pressed = false
	u.position = p
	_vp.push_input(u)
	var mu := InputEventMouseButton.new()
	mu.device = InputEvent.DEVICE_ID_EMULATION
	mu.button_index = MOUSE_BUTTON_LEFT
	mu.pressed = false
	mu.position = p
	mu.global_position = p
	_vp.push_input(mu)
	await _frames(3)

func _shell(vw: int, vh: int) -> void:
	var dpw: float = _app.get_viewport_rect().size.x
	print("MS7 vp=%dx%d dp_width=%.1f density=%s ui=%s tablet=%s phone=%s" % [vw, vh, dpw,
		_arg("--density", "?"), _arg("--ui-scale", "1"), str(DccTheme.is_tablet()), str(DccTheme.is_phone())])
	_ok(DccTheme.is_tablet() and not DccTheme.is_phone(), "the tablet class")
	var map0: float = _app.viewport_area.size.x
	print("MS7 map=%.1f plan=%s" % [map0, str(_app.last_dock_plan)])
	_ok(map0 >= 520.0, "the map keeps at least 520 dp (is %.1f)" % map0)
	var want := {"left": _arg("--expect-left", ""), "right": _arg("--expect-right", "")}
	for side in ["left", "right"]:
		var is_left: bool = side == "left"
		var floating: bool = _app._left_floating if is_left else _app._right_floating
		var dock: Control = _app.left_dock if is_left else _app.right_dock
		var slot: Control = _app._left_slot if is_left else _app._right_slot
		var tab: Button = _app._left_tab if is_left else _app._right_tab
		_ok((floating and want[side] == "floating") or (not floating and want[side] == "docked"),
			"%s dock is %s (runner expects %s)" % [side, "floating" if floating else "docked", want[side]])
		if not floating:
			_ok(not _app.is_dock_collapsed(side), "%s docked dock is open" % side)
			_ok(not tab.visible, "%s docked: no edge tab" % side)
			_ok(absf(slot.size.x - dock.size.x) < 1.0 and dock.size.x >= 232.0 - 1.0,
				"%s docked: the dock fills its slot (%.1f in %.1f)" % [side, dock.size.x, slot.size.x])
			continue
		_ok(_app.is_dock_collapsed(side), "%s floating dock starts closed" % side)
		_ok(absf(slot.size.x - 44.0) < 1.0, "%s floating: its slot is the 44 dp tab (is %.1f)" % [side, slot.size.x])
		_ok(tab.visible and tab.size.x >= 44.0 - 0.5 and tab.size.y >= 44.0 - 0.5,
			"%s tab is at least 44 x 44 dp (is %.1f x %.1f)" % [side, tab.size.x, tab.size.y])
		_ok(absf(_app.viewport_area.size.x - map0) < 0.5, "%s: closed leaves the map as it was" % side)
		## A real tap on the tab opens the dock OVER the map.
		await _tap(tab.get_global_rect().get_center())
		_ok(not _app.is_dock_collapsed(side), "%s: a tap on the tab opens the dock" % side)
		_ok(absf(_app.viewport_area.size.x - map0) < 0.5,
			"%s: open over the map, the map's width unchanged (%.1f vs %.1f)" % [side, _app.viewport_area.size.x, map0])
		_ok(dock.size.x >= 232.0 - 1.0 and dock.get_global_rect().intersects(_app.viewport_area.get_global_rect()),
			"%s: the open dock overlaps the map and is its own width (%.1f)" % [side, dock.size.x])
		_ok(_app.viewport_area.size.x >= 520.0, "%s: the floor holds with the dock open" % side)
		## A tap inside the open dock does not close it.
		await _tap(dock.get_global_rect().get_center())
		_ok(not _app.is_dock_collapsed(side), "%s: a tap inside the open dock leaves it open" % side)
		## A tap on the map, clear of the dock, closes it.
		var map_rect: Rect2 = _app.viewport_area.get_global_rect()
		var clear_x: float = map_rect.end.x - 40.0 if is_left else map_rect.position.x + 40.0
		await _tap(Vector2(clear_x, map_rect.get_center().y))
		_ok(_app.is_dock_collapsed(side), "%s: a tap on the map closes the dock" % side)
	var titles := _row_titles()
	var whole := ["File", "Edit", "Assets", "Records", "Story", "Preferences", "Window", "Help"]
	var short := ["☰", "File", "World", "Records", "Story"]
	var bar_want := whole if _arg("--expect-bar", "") == "whole" else short
	_ok(titles == bar_want, "the menu bar is the %s bar: %s" % [_arg("--expect-bar", ""), str(titles)])
	for c in _app.menu_bar_row.get_children():
		if c is MenuButton and (c as MenuButton).visible:
			_ok((c as MenuButton).size.y >= 44.0 - 0.5 and (c as MenuButton).size.x >= 44.0 - 0.5,
				"menu '%s' is at least 44 x 44 dp (is %.1f x %.1f)" % [(c as MenuButton).text, (c as MenuButton).size.x, (c as MenuButton).size.y])
	var root: Control = _app._desktop_root
	_ok(root.get_combined_minimum_size().x <= dpw + 0.5, "nothing forces the shell wider than the viewport (min %.1f, viewport %.1f)" % [root.get_combined_minimum_size().x, dpw])
	## Menu-open time: recorded, not judged (this is not the performance pass).
	for c in _app.menu_bar_row.get_children():
		if c is MenuButton and (c as MenuButton).text == "File":
			var t0 := Time.get_ticks_usec()
			(c as MenuButton).show_popup()
			await RenderingServer.frame_post_draw
			var ms := (Time.get_ticks_usec() - t0) / 1000.0
			print("MS7 MENUOPEN File show_popup to first drawn frame: %.1f ms" % ms)
			(c as MenuButton).get_popup().hide()
			break
