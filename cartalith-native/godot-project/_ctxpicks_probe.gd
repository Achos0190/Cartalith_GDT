extends Node
## CM-7 (`MAP_CONTEXT_SCOPE.md` §9.2, §11; `OUTSTANDING_WORK.md` §2.12): the
## new context-card picks and rows this pass adds -- landmark, route, sculpt
## stamp -- plus the CM-2 residual CIVIL rows (settlement-class Tool param,
## Open city layout…, Start way/route here, territory draft commit/discard).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxpicks_probe.tscn
##
## Windowed, not headless: every row is reached through a real RMB gesture
## into the embedded card, the same discipline `_ctxcard_probe.gd` already
## follows -- never by calling a workspace's `context_actions()` directly.
##
## Legs:
##   LM  landmark pick + Inspect / Why here? rows
##   RT  route pick + Open in Journey Planner / Delete rows
##   ST  sculpt-stamp pick + Select / Hide / Move up / Move down / Delete rows
##   SC  CM-2 residual: settlement-class Tool param row
##   CL  CM-2 residual: Open city layout… row
##   WR  CM-2 residual: Start way here / Start route here rows
##   TR  CM-2 residual: Commit territory / Discard territory draft rows
##   REG regression -- `_ctxcard_probe.gd`'s own CX-01 rows (A) still land

const SEED := 719004

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


func _ok_true(what: String, got: bool) -> void:
	_ok(what, got, true)


func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _frames(2)


func _texts(kinds: Array = ["action", "disabled", "param"]) -> Array:
	var out: Array = []
	var c = _card()
	if c == null:
		return out
	for r in c.drawn_rows():
		if kinds.has(String(r["kind"])):
			out.append(String(r["text"]))
	return out


func _row(prefix: String, kinds: Array = ["action", "disabled", "param"]) -> Control:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if kinds.has(String(r["kind"])) and String(r["text"]).begins_with(prefix):
			return r["node"]
	return null


func _dump(tag: String) -> void:
	var c = _card()
	if c == null or not c.visible:
		print("ROW %s card=closed" % tag)
		return
	for r in c.drawn_rows():
		var extra := ""
		if r.has("reason"):
			extra = "  {reason: %s}" % r["reason"]
		if r.has("value"):
			extra = "  {value: %s}" % r["value"]
		print("ROW %s %s | %s%s" % [tag, r["kind"], r["text"], extra])


func _in_vp(n: Control) -> Rect2:
	var c = _card()
	var r := n.get_global_rect()
	return Rect2(r.position + Vector2(c.position), r.size)


func _click_at(p: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = p
	mv.global_position = p
	_vp.push_input(mv)
	await _frames(1)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = p
		ev.global_position = p
		_vp.push_input(ev)
		await _frames(1)
	await _frames(3)


func _click_row(n: Control) -> void:
	if n == null:
		return
	await _click_at(_in_vp(n).get_center())


func _rmb_press(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	ev.position = pos
	ov._gui_input(ev)


func _rmb_release(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = false
	ev.position = pos
	ov._gui_input(ev)


func _rmb(ov: Control, pos: Vector2) -> void:
	_rmb_press(ov, pos)
	_rmb_release(ov, pos)
	await _frames(4)


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for ch in n.get_children():
		var h := _find(ch, pred)
		if h != null:
			return h
	return null


func _ws(file: String) -> Node:
	return _find(app, func(n: Node) -> bool:
		var sc: Variant = n.get_script()
		return sc != null and String(sc.resource_path).ends_with(file))


## An empty LAND cell not too close to anything else, so "Drop settlement
## here" / "Start way here" / territory painting land clean.
func _empty_pos(ov: Control, gw: int, gh: int) -> Vector2:
	var bridge = app.bridge
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	for tries in 400:
		var gx := float(gw) * (0.3 + 0.001 * tries)
		var gy := float(gh) * 0.5
		var cell: Dictionary = bridge.sample_cell(int(gx), int(gy))
		if String(cell.get("water", "")) == "land":
			var p: Vector2 = ov._cell_to_screen(Vector2(gx, gy), rect)
			if inter.grow(-60.0).has_point(p) and ov.hits_at(p).is_empty():
				return p
	return Vector2.ZERO


func _run() -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var civ_ws = _ws("civilization_workspace.gd")
	var world_ws = _ws("world_workspace.gd")
	_ok_true("MODE the broker has a card presenter", app.get("context_broker") != null)

	app.select_domain("civilization")
	await _frames(4)

	# -- LM: landmark pick + Inspect / Why here? ---------------------------------
	print("### running landmark pass ###")
	await bridge.landmark_run()
	var lms: Array = bridge.landmarks()
	_ok_true("LM at least one landmark exists after landmark_run()", lms.size() > 0)
	if lms.size() > 0:
		var lm: Dictionary = lms[0]
		var rect: Rect2 = ov._displayed_rect()
		var lp: Vector2 = ov._cell_to_screen(Vector2(float(lm.get("x", 0)), float(lm.get("y", 0))), rect)
		await _rmb(ov, lp)
		_dump("LM_landmark")
		var kind_cap := String(lm.get("kind", "landmark")).capitalize()
		_ok_true("LM Inspect row is drawn", _row("Inspect %s" % kind_cap) != null)
		_ok_true("LM Why here? row is drawn", _row("Why here?") != null)
		await _click_row(_row("Inspect %s" % kind_cap))
		var hint1: String = String(app._status_labels["hint"].text)
		_ok_true("LM Inspect wrote the landmark's own kind into the hint", hint1.begins_with(kind_cap))
		## Re-resolved fresh: `Inspect`'s own status write can change the
		## status region's height, moving `_displayed_rect()` -- the same
		## staleness this pass's own RT/CL legs already chase down.
		var rect_lm2: Rect2 = ov._displayed_rect()
		var lp2: Vector2 = ov._cell_to_screen(Vector2(float(lm.get("x", 0)), float(lm.get("y", 0))), rect_lm2)
		await _rmb(ov, lp2)
		await _click_row(_row("Why here?"))
		var hint2: String = String(app._status_labels["hint"].text)
		_ok_true("LM Why here? wrote a placement funnel (contains 'placed')", hint2.contains("placed"))
	await _close()

	# -- RT: route pick + Open in Journey Planner / Delete -----------------------
	var gw := int(bridge.get_param("grid_w")) if bridge.has_method("get_param") else 256
	var gh := int(bridge.get_param("grid_h")) if bridge.has_method("get_param") else 192
	if gw <= 0:
		gw = 256
	if gh <= 0:
		gh = 192
	var rx0 := float(gw) * 0.25
	var ry0 := float(gh) * 0.25
	var rx1 := float(gw) * 0.35
	var ry1 := float(gh) * 0.30
	bridge.route_begin("mixed")
	bridge.route_append_stop(rx0, ry0)
	bridge.route_append_stop(rx1, ry1)
	var route_idx: int = bridge.route_commit()
	_ok_true("RT a route committed", route_idx >= 0 and bridge.route_count() > 0)
	if route_idx >= 0 and bridge.route_count() > 0:
		## The engine solves a path over the existing network between the two
		## stops (`route_begin("mixed")`'s own "mixed land and sea" doc
		## comment) -- the SOLVED polyline, not the straight line between
		## `(rx0,ry0)` and `(rx1,ry1)`, is what the pick tests against, so the
		## fixture reads its own real geometry back rather than assuming it.
		var solved: PackedVector2Array = bridge.route_get(route_idx).get("points", PackedVector2Array())
		_ok_true("RT the committed route has a solved polyline", solved.size() > 0)
		var mid: Vector2 = solved[solved.size() / 2] if solved.size() > 0 else Vector2((rx0 + rx1) * 0.5, (ry0 + ry1) * 0.5)
		var rect: Rect2 = ov._displayed_rect()
		var rp: Vector2 = ov._cell_to_screen(mid, rect)
		app.arm_tool("inspect")
		await _frames(2)
		await _rmb(ov, rp)
		_dump("RT_route")
		var rname := String(bridge.route_get(route_idx).get("name", ""))
		if rname.is_empty():
			rname = "Journey %d" % (route_idx + 1)
		var open_row := _row("Open “%s” in Journey Planner" % rname)
		var del_row := _row("Delete “%s”" % rname)
		_ok_true("RT Open in Journey Planner row is drawn", open_row != null)
		_ok_true("RT Delete row is drawn", del_row != null)
		await _click_row(open_row)
		_ok_true("RT clicking Open in Journey Planner switched to the CIVIL planner mode on this route",
			app.active_domain() == "civilization" and int(app.journey_planner_view.get("_route_index")) == route_idx)
		## Re-resolved fresh, not reusing `rp`: switching to the planner mode
		## can change the map's own `_displayed_rect()` (a dock layout change),
		## the same staleness CL's own settlement re-hit-test chased down.
		var rect2: Rect2 = ov._displayed_rect()
		var rp2: Vector2 = ov._cell_to_screen(mid, rect2)
		await _rmb(ov, rp2)
		var before_n: int = bridge.route_count()
		await _click_row(_row("Delete “%s”" % rname))
		_ok("RT Delete removed the committed route", bridge.route_count(), before_n - 1)

	# -- ST: sculpt-stamp pick + Select/Hide/Move up/down/Delete -----------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("sculpt")
	await _frames(2)
	var sx0 := float(gw) * 0.6
	var sy0 := float(gh) * 0.6
	var sx1 := float(gw) * 0.6 + 6.0
	var sy1 := float(gh) * 0.6 + 4.0
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(sx0, sy0)
	bridge.sculpt_add_point(sx1, sy1)
	var stamp_idx: int = bridge.sculpt_end_stroke()
	app.arm_tool("inspect")
	await _frames(2)
	_ok_true("ST a stamp landed on the draft", stamp_idx >= 0 and bridge.sculpt_stamp_count() > 0)
	if stamp_idx >= 0:
		var srow0 := {}
		for r in bridge.sculpt_list_stamps():
			if int((r as Dictionary).get("index", -1)) == stamp_idx:
				srow0 = r
		var slabel := String(srow0.get("label", "stamp"))
		var rect: Rect2 = ov._displayed_rect()
		var sp: Vector2 = ov._cell_to_screen(Vector2(sx0, sy0), rect)
		await _rmb(ov, sp)
		_dump("ST_stamp")
		_ok_true("ST Select row is drawn", _row("Select %s" % slabel) != null)
		_ok_true("ST Hide row is drawn", _row("Hide %s" % slabel) != null)
		_ok_true("ST Move up row is drawn", _row("Move %s up" % slabel) != null)
		_ok_true("ST Move down row is drawn", _row("Move %s down" % slabel) != null)
		_ok_true("ST Delete row is drawn", _row("Delete %s" % slabel) != null)
		await _click_row(_row("Select %s" % slabel))
		_ok("ST Select actually selected the stamp", bridge.sculpt_get_selected_stamp(), stamp_idx)
		await _rmb(ov, sp)
		await _click_row(_row("Hide %s" % slabel))
		var hidden_now := false
		for r in bridge.sculpt_list_stamps():
			if int((r as Dictionary).get("index", -1)) == stamp_idx:
				hidden_now = bool(r.get("hidden", false))
		_ok_true("ST Hide actually hid the stamp", hidden_now)
		await _rmb(ov, sp)
		_ok_true("ST the row now reads Show (hidden state round-trips)", _row("Show %s" % slabel) != null)
		await _click_row(_row("Delete %s" % slabel))
		_ok("ST Delete removed the stamp", bridge.sculpt_stamp_count(), 0)

	# -- SC: settlement-class Tool param -----------------------------------------
	app.select_domain("civilization")
	await _frames(4)
	app.arm_tool("settlement")
	await _frames(2)
	var epos := _empty_pos(ov, gw, gh)
	_ok_true("SC an empty land cell exists", epos != Vector2.ZERO)
	if epos != Vector2.ZERO:
		var before_kind := String(civ_ws.get("_settlement_kind"))
		await _rmb(ov, epos)
		_dump("SC_settlement_tool")
		var crow := _row("Class", ["param"])
		_ok_true("SC a Class param row is drawn under TOOL", crow != null)
		if crow != null:
			var plus: Button = null
			for b in crow.find_children("*", "Button", true, false):
				if (b as Button).text == "+":
					plus = b
			if plus != null:
				await _click_at(_in_vp(plus).get_center())
			var after_kind := String(civ_ws.get("_settlement_kind"))
			_ok_true("SC + stepped the settlement class", after_kind != before_kind)
		await _close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- CL: Open city layout… ----------------------------------------------------
	var settlements: Array = bridge.settlements()
	var sidx := -1
	var sp2 := Vector2.ZERO
	var ov_settlements: Array = ov.get("_settlements")
	var rect: Rect2 = ov._displayed_rect()
	var inter0: Rect2 = ov._interior_rect(rect)
	for i in ov_settlements.size():
		var cand: Vector2 = ov._cell_to_screen(Vector2(ov_settlements[i]["x"], ov_settlements[i]["y"]), rect)
		if inter0.grow(-80.0).has_point(cand) and ov._hit_test_settlement(cand, inter0, rect) == i:
			sidx = i
			sp2 = cand
			break
	if sidx >= 0:
		var s: Dictionary = settlements[sidx]
		await _rmb(ov, sp2)
		_dump("CL_settlement")
		var nm := String(s.get("name", "(unnamed)"))
		var cl_row := _row("Open city layout for %s" % nm)
		_ok_true("CL the row is drawn for a settlement hit", cl_row != null)
		await _click_row(cl_row)
		_ok_true("CL clicking it opened the city viewer", app.city_viewer_window != null and app.city_viewer_window.visible)
		if app.city_viewer_window != null:
			app.city_viewer_window.hide()
	else:
		print("SKIP CL: no settlements in this fixture")

	# -- WR: Start way here / Start route here -----------------------------------
	var epos2 := _empty_pos(ov, gw, gh)
	if epos2 != Vector2.ZERO:
		var before_way := int((civ_ws._infra.get("_way_points") as PackedVector2Array).size())
		await _rmb(ov, epos2)
		_dump("WR_start_way")
		_ok_true("WR Start way here row is drawn", _row("Start way here") != null)
		await _click_row(_row("Start way here"))
		var after_way := int((civ_ws._infra.get("_way_points") as PackedVector2Array).size())
		_ok("WR Start way here placed the first waypoint", after_way, before_way + 1)
		civ_ws._infra._commit_way()
		app.arm_tool("inspect")
		await _frames(2)

		var epos3 := _empty_pos(ov, gw, gh)
		var before_route := int((civ_ws._infra.get("_route_points") as PackedVector2Array).size())
		await _rmb(ov, epos3)
		_dump("WR_start_route")
		_ok_true("WR Start route here row is drawn", _row("Start route here") != null)
		await _click_row(_row("Start route here"))
		var after_route := int((civ_ws._infra.get("_route_points") as PackedVector2Array).size())
		_ok("WR Start route here placed the first stop", after_route, before_route + 1)
		civ_ws._infra._commit_route()
		app.arm_tool("inspect")
		await _frames(2)

	# -- TR: Commit territory / Discard territory draft --------------------------
	app.arm_tool("territory")
	await _frames(2)
	var epos4 := _empty_pos(ov, gw, gh)
	if epos4 != Vector2.ZERO:
		var trrect: Rect2 = ov._displayed_rect()
		var g: Dictionary = ov._grid_point(epos4, trrect, ov._interior_rect(trrect))
		bridge.civ_territory_paint_at(float(g["gx"]), float(g["gy"]), 1, 5.0, false)
		var before_claimed := int(bridge.civ_faction_territory_stats(1).get("claimed_cells", 0))
		await _rmb(ov, epos4)
		_dump("TR_territory")
		_ok_true("TR Commit territory row is drawn", _row("Commit territory") != null)
		_ok_true("TR Discard territory draft row is drawn", _row("Discard territory draft") != null)
		await _click_row(_row("Commit territory"))
		var after_claimed := int(bridge.civ_faction_territory_stats(1).get("claimed_cells", 0))
		_ok_true("TR Commit territory actually claimed cells", after_claimed > before_claimed)
	app.arm_tool("inspect")
	await _frames(2)

	# -- REG: CX-01's five rows still land (no regression from this pass) --------
	## Re-resolved fresh, not reusing CL's `sp2`: `_displayed_rect()` moves as
	## docks open/close across legs (a tool-options bar changing the map's own
	## vertical space), which is the exact bug CL's own hit-test chased down
	## a few legs back -- a screen position captured under one rect is not
	## valid under a later one.
	var reg_rect: Rect2 = ov._displayed_rect()
	var reg_inter: Rect2 = ov._interior_rect(reg_rect)
	var reg_sidx := -1
	var reg_pos := Vector2.ZERO
	for i in ov_settlements.size():
		var cand2: Vector2 = ov._cell_to_screen(Vector2(ov_settlements[i]["x"], ov_settlements[i]["y"]), reg_rect)
		if reg_inter.grow(-80.0).has_point(cand2) and ov._hit_test_settlement(cand2, reg_inter, reg_rect) == i:
			reg_sidx = i
			reg_pos = cand2
			break
	if reg_sidx >= 0:
		await _rmb(ov, reg_pos)
		_dump("REG_settlement")
		_ok_true("REG Edit row still lands", _row("Edit ") != null)
		_ok_true("REG Move viewer to row still lands", _row("Move viewer to ") != null)
		_ok_true("REG Delete row still lands", _row("Delete ") != null)
		await _close()


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		print("### CTXPICKS ABORT: unknown argument '%s' ###" % args[0])
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 900.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s at %s" % [form, _vp.size])
	if form == "phone":
		print("### CTXPICKS ABORT: booted as a phone -- use a desktop/tablet-sized viewport ###")
		get_tree().quit(2)
		return

	await _run()
	print("### CTXPICKS %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
