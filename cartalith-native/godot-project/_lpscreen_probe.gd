extends Node
## Owner bug 2026-11-15 (phone APK): "a long press on phone makes the whole map disappear and opens the settings
## menu". Root cause: `PhoneMenu.peek_card()` shows the menu's root (`visible = true`) so the peek sheet and its
## scrim draw, but it never hid the menu's own full-screen MENU pane `_screen` -- a `PanelContainer` that is visible
## by default and that `_render()` leaves visible after any earlier MENU/STORY screen. So the first long press
## after boot (or after any MENU screen was closed) drew the opaque, empty MENU pane (its back and close header)
## over the whole map, beside the peek card. `_ctxphone_probe.gd` never looked at `_screen`, so it passed.
##
## Legs (windowed, phone form only):
##   A  fresh boot, a 600 ms hold on empty ground: the peek card is open AND `_screen` / `_sheet` are not visible
##   B  MENU opened and closed first (the stale-`_render()` case), then a hold: the same
##   C  the fix must not break MENU itself: `open()` after a peek card still shows `_screen`
##
## Mutant (literal, run 2026-11-15, killed): M1 deletes the two lines `_screen.visible = false` and
## `_sheet.visible = false` that `PhoneMenu.peek_card()` gained, leaving the default-hidden build in place: leg B goes
## red (7/8), because `_render()` had left the pane visible. Leg A alone is red on the pre-fix code (6/8: A and B).
##
## Command: `_lpscreen_probe.tscn -- --vp 1080x2340 --force-touch`. Exit 0 on GREEN, 1 on RED, 2 on abort.

const SEED := 552017
var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, got: Variant, want: Variant) -> void:
	_checks += 1
	var pass_: bool = typeof(got) == typeof(want) and got == want
	if not pass_:
		_fails += 1
	print("%s  %s  got=%s want=%s" % ["PASS" if pass_ else "FAIL", what, str(got), str(want)])

func _menu():
	return app._phone_menu

func _mouse(ov: Control, pos: Vector2, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)

func _hold(ov: Control, pos: Vector2) -> void:
	_mouse(ov, pos, true)
	await get_tree().create_timer(0.85).timeout
	await _frames(2)
	_mouse(ov, pos, false)
	await _frames(4)

## A point on empty ground: no settlement pin within 80 px, inside the map's interior.
func _empty_point(ov: Control, rect: Rect2, inter: Rect2) -> Vector2:
	var sets: Array = ov.get("_settlements")
	for _t in 400:
		var c: Vector2 = inter.position + Vector2(randf() * inter.size.x, randf() * inter.size.y)
		if ov._hit_test_settlement(c, inter, rect) != -1:
			continue
		var near := false
		for s in sets:
			if c.distance_to(ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)) < 80.0:
				near = true
				break
		if not near:
			return c
	return Vector2(-1, -1)

func _run() -> void:
	app.select_domain("civilization")
	await _frames(4)
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	var at := _empty_point(ov, rect, inter)
	_ok("an empty point exists", at.x >= 0.0, true)
	if at.x < 0.0:
		return

	# A: the first long press after boot.
	await _hold(ov, at)
	_ok("A the peek card opened", _menu().peek_card_is_open(), true)
	_ok("A the MENU pane is NOT drawn behind it", _menu()._screen.is_visible_in_tree(), false)
	_ok("A the sheet pane is not drawn either", _menu()._sheet.is_visible_in_tree(), false)
	_menu().go_back()
	await _frames(4)

	# B: after a MENU screen was shown and closed, `_render()` leaves `_screen.visible` true.
	_menu().open()
	await _frames(3)
	_ok("B MENU's own pane shows while MENU is open", _menu()._screen.is_visible_in_tree(), true)
	_menu().close()
	await _frames(3)
	await _hold(ov, at)
	_ok("B the peek card opened", _menu().peek_card_is_open(), true)
	_ok("B the MENU pane is NOT drawn behind it", _menu()._screen.is_visible_in_tree(), false)
	_menu().go_back()
	await _frames(4)

	# C: MENU still works after a peek card.
	_menu().open()
	await _frames(3)
	_ok("C MENU's pane shows again after a peek card", _menu()._screen.is_visible_in_tree(), true)
	_menu().close()

func _ready() -> void:
	var vp_size := Vector2i(1080, 2340)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### LPSCREEN ABORT: unknown argument '%s' ###" % a)
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	if not DccTheme.is_phone() or _menu() == null:
		print("### LPSCREEN ABORT: needs the phone form (--force-touch, phone-sized --vp) ###")
		get_tree().quit(2)
		return
	await _run()
	print("### LPSCREEN %s  %d/%d checks passed ###" % ["GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
