extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: Right-click-to-reset on tablet-portrait slider rows survives compact/wide remounts (WorldWorkspace._wire_row_reset, _slider_reflow_entries). Reflow entries exist ONLY on the tablet geometry, so --force-touch is required (without it: 'found Continentality reflow entry' FAILs, 0 entries).
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): windowed: --path . _gridgesture_probe.tscn -- --force-touch
## Result: '_gridgesture_probe: 0 FAILURE(S)'. Note: it prints a SCRIPT ERROR at world_workspace _refresh_geo_age_row (geo_clock_readout, code 1337) when the debug DLL predates GF-7; rebuild the DLL before reading that as a code bug.
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## Throwaway diagnostic, lane GRID -- NOT part of the deliverable, deleted
## after use. Proves right-click-to-reset survives the tablet-portrait
## compact-cell reflow AND a live rotation: the reverted batch's own defect
## was "right-click-reset moved from the row to the track" -- this fires the
## gesture on the LABEL side specifically (never the slider itself), in both
## mount shapes, and after a remount.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Fabricates a right-click press/release and fires it on `ctl.gui_input`
## directly -- the same signal `_wire_row_reset` connects to, and the
## reliable way to exercise a specific Control's wiring without depending on
## real screen-space routing through whatever else is on top in a windowed
## automated run.
func _right_click(ctl: Control) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	ctl.emit_signal("gui_input", ev)

func _ready() -> void:
	await _frames(2)
	var vp := SubViewport.new()
	vp.size = Vector2i(800, 1280)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	var shell := app as DccShell
	shell.select_domain_mode("world", "a")
	await _frames(6)
	var panel = shell.workspace_panel("world")
	# Open "Generate" (World structure's dials, incl. "Continentality").
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == "Generate":
			if not (e["body"] as Control).visible:
				(e["button"] as Button).pressed.emit()
			break
	await _frames(10)

	var entries: Array = panel.get("_slider_reflow_entries")
	print("reflow entries found: ", entries.size())
	var target: Dictionary = {}
	for e in entries:
		var lbl := e["label"] as Label
		if lbl != null and lbl.text == "Continentality":
			target = e
			break
	_ok("found Continentality reflow entry", not target.is_empty(), true)
	if target.is_empty():
		get_tree().quit(1)
		return

	var s := target["slider"] as HSlider
	var default_v := s.value
	print("  default value = ", default_v)

	# --- Portrait boot: compact should already be mounted. ---
	_ok("initial mount is compact (portrait boot)", target["mounted"], "compact")
	s.value = 0.05
	await _frames(2)
	_ok("value diverged from default before any right-click", is_equal_approx(s.value, default_v), false)
	_right_click(target["line1"])
	await _frames(2)
	_ok("compact: right-click on line1 (label side) resets value",
		is_equal_approx(s.value, default_v), true)

	s.value = 0.05
	await _frames(2)
	_right_click(s)
	await _frames(2)
	_ok("compact: right-click on the slider itself still resets value",
		is_equal_approx(s.value, default_v), true)

	# --- Rotate to landscape: should remount wide. ---
	DccTheme.set_portrait(false)
	await _frames(4)
	_ok("after set_portrait(false): mounted is wide", target["mounted"], "wide")

	s.value = 0.05
	await _frames(2)
	_right_click(target["wide"])
	await _frames(2)
	_ok("wide: right-click on the row itself resets value",
		is_equal_approx(s.value, default_v), true)

	s.value = 0.05
	await _frames(2)
	_right_click(s)
	await _frames(2)
	_ok("wide: right-click on the slider itself still resets value",
		is_equal_approx(s.value, default_v), true)

	# --- Rotate back to portrait: should remount compact, gesture still wired. ---
	DccTheme.set_portrait(true)
	await _frames(4)
	_ok("after set_portrait(true): mounted is compact again", target["mounted"], "compact")

	s.value = 0.05
	await _frames(2)
	_right_click(target["line1"])
	await _frames(2)
	_ok("compact after round-trip: right-click on line1 resets value",
		is_equal_approx(s.value, default_v), true)

	print("\n_gridgesture_probe: ", _fail, " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
