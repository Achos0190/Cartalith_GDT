extends Node
## Owner bug 2026-11-15 (6T capture `C:/cw/lp_shots/5_fixed_after_lp.png`, `OUTSTANDING_WORK.md` "Phone: the
## long-press card's action buttons are far below the 44 dp touch floor"): the peek card's four chips (Sculpt here,
## Cross-section from here, Centre view here, Measure from here) were ~20 dp strips with 9 pt text in one row.
## `PhoneMenu._peek_chip_row` is now a 2-column grid of >= 44 dp chips with a wrapping 12 dp label.
##
## Legs (windowed, phone form): after a 0.85 s hold on empty ground,
##   A  four chips exist, in a 2x2 grid (two distinct x, two distinct y)
##   B  each chip is at least 44 dp tall in px (`44 * phone_scale()`), and its font is at least 12 dp
##   C  no label is clipped: `clip_text` is off and the wrapped text height fits inside the chip
##   D  every chip sits inside the peek panel and the panel inside the viewport
##
## Mutant (literal, run 2026-11-15): M1 removes `b.custom_minimum_size.y = _pt(_PEEK_CHIP_MIN_DP)` from
## `_rebuild_peek_chip_row`; leg B goes red. Desktop/tablet unchanged: `context_card.gd` and its `_action_row` are
## untouched by this change (`git diff --stat` names only `phone_menu.gd`), and `_ctxcard` desktop probes are re-run.
##
## Command: `_cardbtn_probe.tscn -- --vp 1080x2340 --force-touch --density 2.625`. Exit 0 GREEN, 1 RED, 2 abort.

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
	await _hold(ov, at)
	_ok("peek card open", _menu().peek_card_is_open(), true)
	var scale: float = app.phone_scale()
	var floor_px := int(round(44.0 * scale))
	var row: GridContainer = _menu().get("_peek_chip_row")
	var chips: Array = row.get_children().filter(func(c): return c is Button and not c.is_queued_for_deletion())
	_ok("A four chips", chips.size(), 4)
	var xs := {}
	var ys := {}
	var panel: Rect2 = _menu().peek_panel_rect()
	var vis := Rect2(Vector2.ZERO, Vector2(_vp.size))
	for c in chips:
		var b := c as Button
		var r := b.get_global_rect()
		xs[int(r.position.x)] = true
		ys[int(r.position.y)] = true
		_ok("B '%s' height %d >= %d px" % [b.text, int(r.size.y), floor_px], r.size.y >= floor_px, true)
		_ok("B '%s' font >= 12 dp" % b.text, b.get_theme_font_size("font_size") >= int(round(12.0 * scale)), true)
		_ok("C '%s' not clip_text" % b.text, b.clip_text, false)
		var f := b.get_theme_font("font")
		var avail := r.size.x - 2.0 * 9.0 * scale
		var th := f.get_multiline_string_size(b.text, HORIZONTAL_ALIGNMENT_CENTER, avail, b.get_theme_font_size("font_size"),
			-1, TextServer.BREAK_WORD_BOUND | TextServer.BREAK_MANDATORY).y
		_ok("C '%s' wrapped text %d fits chip %d" % [b.text, int(th), int(r.size.y)], th <= r.size.y, true)
		_ok("D '%s' inside the peek panel" % b.text, panel.grow(1.0).encloses(r), true)
	_ok("A 2 columns", xs.size(), 2)
	_ok("A 2 rows", ys.size(), 2)
	_ok("D panel inside viewport", vis.grow(1.0).encloses(panel), true)
	app.get_viewport().get_texture().get_image().save_png("C:/cw/cardbtn_shots/after_peek.png") if DirAccess.dir_exists_absolute("C:/cw/cardbtn_shots") else null
	_menu().go_back()
	await _frames(4)

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
		if a == "--density" and i + 1 < args.size():
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### CARDBTN ABORT: unknown argument '%s' ###" % a)
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
		print("### CARDBTN ABORT: needs the phone form (--force-touch, phone-sized --vp) ###")
		get_tree().quit(2)
		return
	await _run()
	print("### CARDBTN %s  %d/%d checks passed ###" % ["GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
