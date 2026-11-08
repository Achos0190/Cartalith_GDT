extends Node
## PC-9's Escape and desktop-search probe (`PC_SHELL_PROPOSAL.md` s.3.9 and finding S4), inside the
## real shell, with real key events through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc9esc_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 _pc9esc_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## user dir (`MISTAKES.md`, "Isolate a probe's user dir"). The only argument read is `--force-touch`;
## anything else aborts with exit 2.
##
## What it protects. Every expectation is a literal, never read back from `app.gd`:
##   A. Escape with Inspect armed and a settlement selected clears the selection (one press).
##   B. Escape with a tool armed that has an escape handler (Measure) leaves the selection alone;
##      the next Escape, with Inspect armed, clears it.
##   C. Escape typed into a focused LineEdit does not clear the selection, and neither does one the field
##      passes on to the app (the app's own guard, called directly).
##   D. Escape with the cartwheel open closes the ring and leaves the selection; the next clears it.
##   E. Escape with the Find dialog open closes the dialog and leaves the selection (a window keeps
##      its Escape).
##   F. With nothing selected Escape is a harmless no-op.
##   G. Desktop search runs commands: `. Deselect` shows a row that is NOT disabled, pressing it clears
##      the selection and hides the dialog; a row that cannot run says why in the dialog's count line.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

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


func _key(code: Key) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _selected() -> bool:
	return not _app.selection.is_empty()


func _pick() -> int:
	var cells: Array = _bridge.settlements()
	for i in cells.size():
		return i
	return -1


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a != "--force-touch":
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch only" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(420.0).timeout.connect(func() -> void:
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
	var si := _pick()
	if not bool(_bridge.has_world) or si < 0:
		print("[FATAL] no world with a settlement")
		get_tree().quit(2)
		return
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(6)

	print("== F. nothing selected: Escape is a no-op ==")
	await _key(KEY_ESCAPE)
	_ok(not _selected() and _app.armed_tool == "inspect", "no selection, Inspect armed, still so")

	print("== A. Inspect armed: one Escape clears ==")
	_app.select("settlement", si)
	await _settle(4)
	_ok(_selected(), "control: a settlement is selected")
	await _key(KEY_ESCAPE)
	_ok(not _selected(), "Escape cleared the selection")

	print("== B. a tool with an escape handler keeps the selection ==")
	_app.select("settlement", si)
	_app.arm_tool("measure")
	await _settle(4)
	await _key(KEY_ESCAPE)
	_ok(_selected(), "Escape with Measure armed left the selection")
	_app.arm_tool("inspect")
	await _settle(3)
	await _key(KEY_ESCAPE)
	_ok(not _selected(), "Escape with Inspect armed cleared it")

	print("== C. a focused LineEdit keeps its Escape ==")
	_app.select("settlement", si)
	var le := LineEdit.new()
	_app.add_child(le)
	await _settle(3)
	le.grab_focus()
	await _settle(3)
	_ok(get_viewport().gui_get_focus_owner() == le, "control: the LineEdit has focus")
	await _key(KEY_ESCAPE)
	_ok(_selected(), "Escape in the text field left the selection")
	## The field's own handling may swallow the key before the app sees it, which would leave the guard
	## in `_unhandled_key_input` untested: hand the app the event directly, as a key the field passed on.
	var ev := InputEventKey.new()
	ev.keycode = KEY_ESCAPE
	ev.pressed = true
	_app._unhandled_key_input(ev)
	await _settle(3)
	_ok(_selected(), "an Escape the field passed on to the app still left the selection (the guard)")
	le.release_focus()
	le.queue_free()
	await _settle(3)

	print("== D. an open cartwheel closes first ==")
	var cb = _app.context_broker
	cb.ring_key_press(Vector2(400, 300))
	await _settle(4)
	_ok(cb.ring_is_open(), "control: the ring is open")
	await _key(KEY_ESCAPE)
	_ok(not cb.ring_is_open() and _selected(), "Escape closed the ring and kept the selection")
	await _key(KEY_ESCAPE)
	_ok(not _selected(), "the next Escape cleared it")

	print("== E. an open window keeps its Escape ==")
	_app.select("settlement", si)
	await _settle(3)
	_app._open_desktop_find_on_map()
	await _settle(8)
	_ok(_app._desktop_search_dialog.visible, "control: the Find dialog is open")
	await _key(KEY_ESCAPE)
	_ok(not _app._desktop_search_dialog.visible and _selected(),
		"Escape closed the dialog and kept the selection")

	print("== G. desktop search runs a command ==")
	_app._open_desktop_find_on_map()
	await _settle(6)
	_app._desktop_search_field.text = ". Deselect"
	_app._refresh_desktop_search()
	await _settle(6)
	var row: Button
	for l in _app._desktop_search_results.find_children("*", "Label", true, false):
		if (l as Label).text.to_lower() == "deselect":
			var n: Node = l
			while n != null and not (n is Button):
				n = n.get_parent()
			row = n as Button
			break
	if row == null:
		print("  info count: ", _app._desktop_search_count.text)
		for l in _app._desktop_search_results.find_children("*", "Label", true, false):
			print("  info label: ", (l as Label).text)
	_ok(row != null, "the results hold a Deselect command row")
	if row != null:
		_ok(not row.disabled, "the desktop row is not a disabled readout")
		row.pressed.emit()
		await _settle(6)
		_ok(not _selected(), "pressing it ran Edit > Deselect (the selection is empty)")
		_ok(not _app._desktop_search_dialog.visible, "and closed the Find dialog")
	_app._open_desktop_find_on_map()
	await _settle(4)
	_app._run_search_command({"title": "x", "why": "Needs a world first.", "pin_key": "k", "pinnable": true},
		func(): pass)
	_ok(_app._desktop_search_count.text == "Needs a world first.",
		"a row that cannot run says why in the count line")
	_app._desktop_search_dialog.hide()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
