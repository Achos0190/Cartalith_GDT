extends Node
## PC-3 follow-up probe: a conflict row in the timeline strip's event list selects the conflict as well as
## moving the year (`PC_SHELL_PROPOSAL.md` 3.2: "a conflict row also selects the conflict", flow F7).
##
##   desktop: godot --path . --resolution 1600x1000 _confevent_probe.tscn
##
## **Windowed, never `--headless`**, from a wiped user dir (`MISTAKES.md`). Exit 0 pass, 1 fail, 2 bad
## argument / watchdog / no world.
##
## What it protects. A small world (seed 4242, 256 x 192) gets two conflicts added through the real
## `conflict_add` (different years and names). The strip is expanded, and each conflict's row in the event
## list (`TimelineEvents`, the button carrying `tl_conflict`) is REAL-CLICKED. After each click:
##   * the year is the conflict's start year (the row's old behaviour, unchanged);
##   * `DccApp.selection` is `{conflict, id}` for THAT conflict (not the other one), and the Context panel
##     shows it (`shown_subject()`), with the conflict's own name in its header;
##   * Ctrl+D clears it and Alt+← brings it back (history).
## Not covered here, by design: a chronos row (no conflict id) only moves the year. That needs a vault and a
## noted settlement; `_pc3strip_probe` clicks one and checks the year.
var _fails := 0
var _checks := 0
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _key(code: Key, ctrl := false, alt := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _row_for(cid: int) -> Button:
	var evrow: Node = _app.timeline_row.find_child("TimelineEvents", true, false)
	if evrow == null:
		return null
	for b in evrow.find_children("*", "Button", true, false):
		if (b as Button).has_meta("tl_conflict") and int((b as Button).get_meta("tl_conflict")) == cid:
			return b
	return null


func _ready() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		print("ABORT: unrecognised argument")
		get_tree().quit(2)
		return
	get_tree().create_timer(400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return
	_app.select_domain("civilization")
	await _settle(6)
	var facs: Array = _bridge.get_factions()
	var fa := int((facs[0] as Dictionary)["id"])
	var fb := int((facs[1] as Dictionary)["id"])
	var r1: Dictionary = _bridge.conflict_add({"name": "Probe war one", "kind": "battle", "start_year": 910,
		"sides": PackedInt32Array([fa, fb]), "points": PackedVector2Array([Vector2(60.0, 60.0)])})
	var r2: Dictionary = _bridge.conflict_add({"name": "Probe war two", "kind": "battle", "start_year": 960,
		"sides": PackedInt32Array([fa]), "points": PackedVector2Array([Vector2(90.0, 70.0)])})
	_ok(bool(r1.get("ok", false)) and bool(r2.get("ok", false)), "two conflicts were added")
	var c1 := int(r1.get("id", -1))
	var c2 := int(r2.get("id", -1))
	_app._tl_expanded = true
	_app._fill_timeline_strip()
	await _settle(10)
	for pair in [[c1, 910, "Probe war one"], [c2, 960, "Probe war two"], [c1, 910, "Probe war one"]]:
		var cid: int = pair[0]
		var yr: int = pair[1]
		var nm: String = pair[2]
		var row := _row_for(cid)
		_ok(row != null and row.is_visible_in_tree(), "the event list has a row for conflict %d ('%s')" % [cid, nm])
		if row == null:
			continue
		var sc := row.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(row)
			await _settle(4)
		await _click_at(row.get_global_rect().get_center())
		_ok(_app.tl_year() == yr, "the click moved the year to %d (got %d)" % [yr, _app.tl_year()])
		_ok(_app.selection == {"kind": "conflict", "id": cid}, "and selected conflict %d (got %s)" % [cid, str(_app.selection)])
		_ok(_app.right_dock_ctrl.shown_subject() == {"kind": "conflict", "id": cid}, "the Context panel shows it")
		_ok(_app.right_dock_ctrl.subject_name({"kind": "conflict", "id": cid}) == nm, "its header names it '%s'" % nm)
		await _settle(4)
		_expand_again()
		await _settle(6)
	## History: Ctrl+D clears, Alt+← brings the last one back.
	await _key(KEY_D, true)
	_ok(_app.selection.is_empty(), "Ctrl+D clears the selection")
	await _key(KEY_LEFT, false, true)
	_ok(_app.selection == {"kind": "conflict", "id": c1}, "Alt+← brings the conflict back (got %s)" % str(_app.selection))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## A selection or a year move can rebuild the strip; keep it expanded for the next row.
func _expand_again() -> void:
	if not _app._tl_expanded:
		_app._tl_expanded = true
		_app._fill_timeline_strip()
