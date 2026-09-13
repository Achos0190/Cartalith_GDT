extends Node
## Main-loop check for two batch-53 verifier findings, fixed after the batch:
##   phone: applying a layout whose regions are all ON must leave both sheets closed
##          (the Window checks now read the sheets' real state, so comparing a
##          layout against them opened a full-height right sheet);
##   desktop: the gallery's foot note must draw wide enough to read (a clip_text
##          Label beside an EXPAND_FILL spacer collapsed to 1 px).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _postfix53_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _postfix53_probe.tscn -- --force-touch
##
## Writes nothing to user://cartalith_settings.cfg: the layout is built inline, not
## read from or saved to the store.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = got == want
	if not good:
		_fail += 1
	print("  %s  %s  got=%s want=%s" % ["ok  " if good else "FAIL", name, str(got), str(want)])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var phone := "--force-touch" in OS.get_cmdline_user_args()
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(12)
	print("[BOOT] composition=", "phone" if phone else "desktop")

	if phone:
		_ok("is_phone", app.is_phone(), true)
		_ok("left sheet closed at boot", app._left_sheet_open, false)
		_ok("right sheet closed at boot", app._right_sheet_open, false)
		var regions := {}
		for rid in DccMenus.WIN_REGION_IDS:
			regions[rid] = true
		app.menus._apply_layout({"regions": regions, "domain": "world", "mode": "b"})
		await _frames(6)
		_ok("applying an all-ON layout leaves the left sheet closed", app._left_sheet_open, false)
		_ok("applying an all-ON layout leaves the right sheet closed", app._right_sheet_open, false)
		_ok("the layout still selects its mode", app.active_mode(), "b")
	else:
		var dlg = app.open_project_dialog
		dlg.open()
		await _frames(6)
		dlg._foot_note.text = "could not open corrupt.ctl — open corrupt.ctl failed: zip error: invalid Zip archive"
		await _frames(6)
		var w: float = dlg._foot_note.size.x
		print("  info foot note drawn width: ", w)
		_ok("gallery foot note draws at least 200 px wide", w >= 200.0, true)
		_ok("gallery foot note is visible in tree", dlg._foot_note.is_visible_in_tree(), true)
		dlg.hide()

	print("_postfix53_probe (%s): %s" % ["phone" if phone else "desktop",
		"PASS" if _fail == 0 else str(_fail) + " FAILURE(S)"])
	get_tree().quit(1 if _fail > 0 else 0)
