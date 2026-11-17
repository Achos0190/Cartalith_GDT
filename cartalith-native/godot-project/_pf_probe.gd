extends Node
## Phonefix probe (2026-11-17): the phone long-press sample pin's drawn size across zoom, and the touch planner's
## crosshair flow. Windowed (a pixel/size probe: `ImageTexture.update()` is a no-op headless).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _pf_probe.tscn -- --leg pin --force-touch --vp 1080x2340 --density 2.625 --out DIR
##   ... -- --leg cross --force-touch --vp 1080x2340 --density 2.625 --out DIR     (and 1700x1000 --density=1.0)
##   ... -- --leg cross --vp 1600x900 --out DIR                                    (desktop: unchanged)
## Run under a scratch APPDATA. Exit 0 = GREEN, 1 = RED, 2 = could not run.

const SEED := 552017
var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _leg := "pin"
var _out := ""
var _tag := "x"

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("%s  %s  %s" % ["PASS" if cond else "FAIL", what, detail])

func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	_vp.get_texture().get_image().save_png("%s/pf_%s_%s.png" % [_out, _tag, name])

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

## The pin head's drawn radius in SCREEN px, read from the overlay's own resolved geometry (`sample_pin_screen_radius`).
func _pin_px(ov: Control) -> float:
	return float(ov.call("sample_pin_screen_radius"))

func _run_pin() -> void:
	app.select_domain("civilization")
	await _frames(4)
	var ov: Control = app.viewport.overlay
	var vh = app.viewport
	var radii := []
	var zooms := [1.0, 4.0, 16.0]
	for z in zooms:
		vh.reset_view()
		await _frames(3)
		vh.zoom_step(z)
		await _frames(4)
		var rect: Rect2 = ov._displayed_rect()
		var c: Vector2 = vh.size * 0.5
		var local: Vector2 = (c - vh._camera.position) / vh._zoom
		_mouse(ov, local, true)
		await get_tree().create_timer(0.85).timeout
		_mouse(ov, local, false)
		await _frames(4)
		var r: float = _pin_px(ov) if ov.has_method("sample_pin_screen_radius") else float(ov._SAMPLE_PIN_RADIUS) * float(vh._zoom)
		radii.append(r)
		print("zoom x%s: camera zoom %.3f, pin head radius %.2f screen px" % [str(z), vh._zoom, r])
		await _shot("pin_z%d" % int(z))
		app._phone_menu.go_back() if app._phone_menu != null else null
		await _frames(3)
		ov.clear_sample_pin()
	print("RADII ", radii)
	if ov.has_method("sample_pin_screen_radius"):
		var unit := 1.0
		_ok("x1 head >= the 9 dp floor in px", float(radii[0]) >= 9.0 * (DccTheme.phone_scale() if DccTheme.is_phone() else 1.0) * 0.99, "%.2f" % float(radii[0]))
		_ok("x16 head is under 3x the x1 head (cube-root law, not linear)", float(radii[2]) < 3.0 * float(radii[0]), "%.2f vs %.2f" % [float(radii[2]), float(radii[0])])
		_ok("x16 head is larger than x1 (it scales)", float(radii[2]) > float(radii[0]) * 1.2, "")
	## The 44 dp touch HIT area, kept apart from the drawn size: a press 45 screen px from the pin's tip grabs it at every zoom
	## (22 dp x 2.625 = 57.75 px radius on this density; 22 x 1 on a unit-1 form), a press 110 px away does not.
	## The tablet's hold left its radial ring open; a sticky ring swallows the next press, so dismiss it first.
	_mouse(ov, Vector2(2, 2), true)
	_mouse(ov, Vector2(2, 2), false)
	await _frames(3)
	for z2 in [1.0, 16.0]:
		vh.reset_view()
		await _frames(2)
		vh.zoom_step(z2)
		await _frames(4)
		var cp: Dictionary = vh.grid_at_view_centre()
		ov._sample_pin = {"gx": float(cp["gx"]), "gy": float(cp["gy"])}
		var unit := DccTheme.phone_scale() if DccTheme.is_phone() else 1.0
		var near: float = 22.0 * unit * 0.78
		var far: float = 22.0 * unit * 1.9
		var base: Vector2 = (vh.size * 0.5 - vh._camera.position) / vh._zoom
		for case in [[near, true], [-near, true], [far, false]]:
			ov._pin_drag_active = false
			ov._touch_ring_active = false   ## the tablet's hold opened the radial ring; the grab test starts from a closed one
			ov._sample_pin = {"gx": float(cp["gx"]), "gy": float(cp["gy"])}   ## a grab drags the pin, so each case starts from the centre
			_mouse(ov, base + Vector2(float(case[0]) / vh._zoom, 0.0), true)
			var hit: bool = ov._pin_drag_active
			_mouse(ov, base + Vector2(float(case[0]) / vh._zoom, 0.0), false)
			await _frames(2)
			_ok("x%d: a press %.0f screen px from the pin %s it" % [int(z2), float(case[0]), "grabs" if case[1] else "misses"], hit == bool(case[1]), "drag=%s" % hit)
		ov.clear_sample_pin()


## Two land settlements well inside the map, 25..90 cells apart, three of them in a rough path A -> B -> C.
func _pick_three() -> Array:
	var sets: Array = app.bridge.settlements()
	var ok: Array = []
	for st in sets:
		var x := float(st["x"])
		var y := float(st["y"])
		if x > 70.0 and x < 440.0 and y > 70.0 and y < 310.0:
			ok.append(Vector2(x, y))
	for a in ok:
		for b in ok:
			if a.distance_to(b) < 25.0 or a.distance_to(b) > 70.0:
				continue
			for c in ok:
				if c.distance_to(b) >= 25.0 and c.distance_to(b) <= 70.0 and c.distance_to(a) > 25.0:
					return [a, b, c]
	return []

func _centre_on(p: Vector2) -> void:
	app.viewport.move_view_to(p.x, p.y)
	await _frames(4)

func _press(b: Button) -> void:
	b.pressed.emit()
	await _frames(4)

func _run_cross() -> void:
	app.select_domain("civilization")
	await _frames(4)
	var pv = app.journey_planner_view
	var bridge = app.bridge
	var touch := DccTheme.is_touch()
	app.open_journey_planner()
	await _frames(6)
	var start_btn: Button = pv._left_route_section.get_node_or_null("CrosshairStart") if pv._left_route_section != null else null
	_ok("the crosshair entry exists exactly on touch forms", (start_btn != null) == touch, "touch=%s btn=%s" % [touch, start_btn != null])
	await _shot("1_planner")
	if not touch:
		_ok("desktop never built a crosshair control", pv._crosshair == null, "")
		return
	app.viewport.reset_view()
	await _frames(3)
	app.viewport.zoom_step(4.0)
	await _frames(4)
	var trio := _pick_three()
	_ok("three settlements found", trio.size() == 3, str(trio))
	if trio.size() != 3:
		return
	var routes0: int = bridge.route_count()
	_ok("the entry button is visible and 44 tall", start_btn.is_visible_in_tree() and start_btn.size.y >= 44.0, str(start_btn.size))
	await _press(start_btn)
	var xh = pv._crosshair
	_ok("crosshair mode is on", xh != null and xh.is_active() and xh.visible, "")
	_ok("the planner stepped aside (its centre panel is hidden)", not pv._center_panel.is_visible_in_tree(), "")
	await _centre_on(trio[0])
	print("DBG xh.size=", xh.size, " parent=", xh.get_parent().size, " pos=", xh.global_position, " bar=", xh.bar().size, " app.size=", app.size)
	await _shot("2_empty")
	var bar: Control = xh.bar()
	var cross_c: Vector2 = app.viewport.get_global_rect().get_center()
	_ok("the bar does not cover the crosshair", not bar.get_global_rect().has_point(cross_c), "bar %s crosshair %s" % [str(bar.get_global_rect()), str(cross_c)])
	var scr: Vector2 = get_viewport().get_visible_rect().size if false else Vector2(_vp.size)
	_ok("the bar is in the bottom half (thumb reach)", bar.get_global_rect().position.y > scr.y * 0.5, "")
	for nm in ["CrosshairAdd", "CrosshairUndo", "CrosshairCommit", "CrosshairCancel"]:
		var b: Button = bar.find_child(nm, true, false)
		_ok("%s is at least 44 tall and on screen" % nm, b != null and b.size.y >= 44.0 and _vp_rect().encloses(b.get_global_rect()), "%s" % (str(b.get_global_rect()) if b != null else "missing"))
	var add: Button = bar.find_child("CrosshairAdd", true, false)
	var undo: Button = bar.find_child("CrosshairUndo", true, false)
	var commit: Button = bar.find_child("CrosshairCommit", true, false)
	_ok("Undo and Commit are disabled with no waypoints", undo.disabled and commit.disabled, "")
	await _press(add)
	_ok("Add start drops the first waypoint at the centre", xh.waypoints().size() == 1 and xh.waypoints()[0].distance_to(trio[0] + Vector2(0.5, 0.5)) < 0.6, str(xh.waypoints()))
	_ok("Commit is still disabled with one waypoint", commit.disabled, "")
	await _press(add)
	_ok("a double press on the same spot adds nothing", xh.waypoints().size() == 1, "")
	await _centre_on(trio[1])
	await _press(add)
	await _shot("3_two_waypoints")
	_ok("two waypoints", xh.waypoints().size() == 2, "")
	await _centre_on(trio[2])
	await _press(add)
	_ok("three waypoints", xh.waypoints().size() == 3, "")
	await _shot("4_three_waypoints")
	await _press(undo)
	_ok("Undo last removes the last waypoint", xh.waypoints().size() == 2 and xh.waypoints()[1].distance_to(trio[1] + Vector2(0.5, 0.5)) < 0.6, str(xh.waypoints()))
	await _centre_on(trio[2])
	await _press(add)
	_ok("no route was added by placing waypoints", bridge.route_count() == routes0, "%d vs %d" % [bridge.route_count(), routes0])
	## Measured, for the decision to compute on commit: the engine call a live preview would repeat per press.
	var wg = bridge.world_gen
	var ms := []
	for _k in 5:
		var t0 := Time.get_ticks_usec()
		wg.route_begin("mixed")
		for q in xh.waypoints():
			wg.route_append_stop(q.x, q.y)
		var ix: int = wg.route_commit()
		ms.append(float(Time.get_ticks_usec() - t0) / 1000.0)
		wg.route_delete(ix)
	ms.sort()
	print("LIVE-ROUTE COST (route_begin+append+commit, 3 waypoints, 512x384), 5 runs sorted ms: ", ms, "  median ", ms[2])
	_ok("a throw-away solve per press is not cheap on this form (> 60 ms median), so the route is computed on commit", float(ms[2]) > 60.0, "median %.1f ms" % float(ms[2]))
	_ok("the throw-away solves left no route behind", bridge.route_count() == routes0, "")
	var pts: PackedVector2Array = xh.waypoints()
	await _press(commit)
	_ok("Commit added exactly one route", bridge.route_count() == routes0 + 1, "%d vs %d" % [bridge.route_count(), routes0])
	var r: Dictionary = bridge.route_get(routes0)
	_ok("the committed route is a real route", float(r.get("km", -1.0)) > 0.0, "%.2f km" % float(r.get("km", -1.0)))
	_ok("the crosshair mode ended and the planner opened on the route", not xh.is_active() and String(app.armed_tool) == "journey" and pv._route_index == routes0, "tool=%s idx=%d" % [app.armed_tool, pv._route_index])
	await _shot("5_committed")
	## Independent route model check: the same stops through the desktop's own three calls give the same length.
	bridge.world_gen.route_begin("mixed")
	for p in pts:
		bridge.world_gen.route_append_stop(p.x, p.y)
	var ref: int = bridge.world_gen.route_commit()
	var rk := float(bridge.world_gen.route_get(ref).get("km", -2.0))
	bridge.world_gen.route_delete(ref)
	_ok("same km as the Route tool's calls on the same stops", absf(rk - float(r.get("km", -1.0))) < 0.01, "%.2f vs %.2f" % [rk, float(r.get("km", -1.0))])
	## Cancel path.
	await _press(pv._left_route_section.get_node("CrosshairStart"))
	await _centre_on(trio[0])
	await _press(xh.bar().find_child("CrosshairAdd", true, false))
	await _press(xh.bar().find_child("CrosshairCancel", true, false))
	_ok("Cancel leaves no route and gives the planner back", bridge.route_count() == routes0 + 1 and not xh.is_active() and pv._center_panel.is_visible_in_tree(), "")
	await _shot("6_cancelled")

func _vp_rect() -> Rect2:
	return Rect2(Vector2.ZERO, Vector2(_vp.size))

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
		if a == "--leg" and i + 1 < args.size():
			_leg = args[i + 1]
			i += 2
			continue
		if a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
			continue
		if a == "--tag" and i + 1 < args.size():
			_tag = args[i + 1]
			i += 2
			continue
		if a == "--density" and i + 1 < args.size():
			i += 2
			continue
		if a.begins_with("--density=") or a == "--force-touch":
			i += 1
			continue
		print("### PF ABORT: unknown argument '%s' ###" % a)
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
	print("form: phone=%s tablet=%s touch=%s vp=%s" % [DccTheme.is_phone(), DccTheme.is_tablet(), DccTheme.is_touch(), str(vp_size)])
	if _tag == "x":
		_tag = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desk")
	if _leg == "pin":
		await _run_pin()
	else:
		await _run_cross()
	print("### PF %s %s  %d/%d checks passed ###" % [_leg, "GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
