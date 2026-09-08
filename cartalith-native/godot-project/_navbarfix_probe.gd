extends Node
## Lane NAVBAR fix proof. Asserts what the OnePlus screenshot measured:
## a phone dock sheet's own content bottom must sit ABOVE the bottom nav
## bar's top -- not merely above the system gesture inset, a different
## number from a different source (`_apply_phone_orientation()`'s own
## `bar_reserve` vs `_safe_bottom()`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _navbarfix_probe.tscn -- --force-touch
##
## Windowed on purpose (not --headless): MISTAKES.md, "Assert on pixels" --
## this specific claim is asked for windowed even though it is a rect
## assertion, so it is proven the same way the device capture was read.
## `get_global_rect()` on the SHEET's own outer Control is not subject to the
## "reports the UNCLIPPED rect" trap that same file warns about for a CHILD
## inside a scrolling ancestor -- there is no ancestor clipping the sheet
## itself, only the sheet's own offset math, which is exactly what is under
## test.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[navbarfix] ", s)

var _fail := 0

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2340)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	if not app.is_phone():
		_log("ABORT not phone -- pass --force-touch")
		get_tree().quit(2)
		return

	var bar: Control = app._phone_menu_bar
	_check(bar != null and bar.visible, "the app's own bottom bar (MAP/GENERATE/PLAN/MORE) exists and is visible")
	var bar_top: float = bar.get_global_rect().position.y
	_log("bar_top=%.1f  safe_bottom=%d  H_PHONE_BOTTOM_NAV=%.1f" %
		[bar_top, app._safe_bottom(), app._ptap(DccTheme.H_PHONE_BOTTOM_NAV)])

	for side in ["left", "right"]:
		app._set_sheet_open(side, true)
		await _frames(6)
		var sheet: Control = app.left_dock if side == "left" else app.right_dock
		var r := sheet.get_global_rect()
		_log("%s_dock rect=%s  offset_bottom=%.1f" % [side, r, sheet.offset_bottom])
		_check(r.end.y <= bar_top + 0.5,
			"%s dock sheet's content bottom (%.1f) is above the bar's top (%.1f)" %
				[side, r.end.y, bar_top])
		app._set_sheet_open(side, false)
		await _frames(2)

	_log("RESULT %s failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
