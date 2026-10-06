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
##   WY  way pick + Inspect / Delete, and Edit ▸ Undo restoring the way exactly
##       (Ruling BA), for a generated way and a hand-drawn one
##   RV  river pick as the branch to its mouth (Ruling BA); Trace downstream
##       measured on screen against the engine's own branch; Show catchment
##       against the engine's own upstream mask; Clear river highlight

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
		## Fixture, corrected 2026-10-06. This leg used to paint faction 1 at
		## whatever land cell `_empty_pos` found and assert faction 1's claimed
		## count grew -- but this world claims EVERY land cell (measured by
		## `_ctxcard_probe.gd`'s scans, which find no unclaimed land at SEED), and
		## the cell it picked belonged to faction 1 already (Aurelia: the card
		## itself read "Open Aurelia…"), so the commit re-claimed cells faction 1
		## owned and the count could not move: a fixture that could not pass, not
		## an engine fault. The brush now paints a faction that does NOT own the
		## cell, so a working commit must move cells from the owner to the
		## painter. An unclaimed-land fixture (the other way to make a claim
		## move) needs a regenerated world, which `_ctxcard_probe.gd` leg FU
		## already builds (`ISLAND_SEED`); stock worlds are not regenerated here.
		var tr_cell: Dictionary = bridge.sample_cell(int(g["gx"]), int(g["gy"]))
		_ok_true("TR fixture: the brush cell is claimed by someone (stock worlds claim all land)",
			tr_cell.has("controlling_faction"))
		if tr_cell.has("controlling_faction"):
			var tr_owner := int(tr_cell["controlling_faction"])
			var tr_painter := 2 if tr_owner != 2 else 1
			bridge.civ_territory_paint_at(float(g["gx"]), float(g["gy"]), tr_painter, 5.0, false)
			var before_painter := int(bridge.civ_faction_territory_stats(tr_painter).get("claimed_cells", -1))
			var before_owner := int(bridge.civ_faction_territory_stats(tr_owner).get("claimed_cells", -1))
			_ok_true("TR fixture: both factions' claim counts are readable", before_painter >= 0 and before_owner >= 0)
			await _rmb(ov, epos4)
			_dump("TR_territory")
			_ok_true("TR Commit territory row is drawn", _row("Commit territory") != null)
			_ok_true("TR Discard territory draft row is drawn", _row("Discard territory draft") != null)
			_faction_rows_match("TR", ov, epos4, true)
			await _click_row(_row("Commit territory"))
			var after_painter := int(bridge.civ_faction_territory_stats(tr_painter).get("claimed_cells", -1))
			var after_owner := int(bridge.civ_faction_territory_stats(tr_owner).get("claimed_cells", -1))
			_ok_true("TR Commit territory actually claimed cells", after_painter > before_painter)
			_ok_true("TR ... taking them from the cell's previous owner", after_owner < before_owner)
			_ok("TR the brushed cell now belongs to the painter",
				int(bridge.sample_cell(int(g["gx"]), int(g["gy"])).get("controlling_faction", -1)), tr_painter)
	app.arm_tool("inspect")
	await _frames(2)

	# -- GATE: the faction rows' gate, over OCEAN (the negative control) ---------
	## Protects: `civilization_workspace.gd::context_actions`'s gate on
	## `sample_cell(...).has("controlling_faction")`. TR and REG hold the
	## positive side (land, and a settlement pin); this is the side that fails
	## if the gate is removed. Mutation-run 2026-10-06 against the three edits
	## named in `_faction_rows_match`'s doc comment.
	var ocean_pos := _ocean_pos(ov, gw, gh)
	_ok_true("GATE fixture: an ocean cell clear of every pin exists", ocean_pos != Vector2.ZERO)
	if ocean_pos != Vector2.ZERO:
		await _rmb(ov, ocean_pos)
		_dump("GATE_ocean")
		_ok_true("GATE the card opened over ocean (the control: it is not simply closed)", _open())
		## The CIVIL rows AFTER the faction block must still be built. With the
		## gate removed, `cf["controlling_faction"]` is an invalid access on an
		## ocean cell (the key is omitted there), the provider aborts mid-list,
		## and the faction rows are absent for the wrong reason -- the absence
		## check below would pass on a crash. This row is what tells the two
		## apart (mutation 1 in `_faction_rows_match`'s doc comment).
		_ok_true("GATE the CIVIL rows after the faction block are still drawn over ocean",
			_row("Drop settlement here") != null)
		_faction_rows_match("GATE ocean", ov, ocean_pos, false)
		await _close()

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
		## The faction rows under a settlement pin: the default taken in
		## `context_actions` (owner unruled) -- the pin's own cell, in a world
		## that claims all land, must carry them. This is also the check that
		## fails if the old `hit < 0` clause is restored.
		_faction_rows_match("REG settlement", ov, reg_pos, true)
		await _close()

	await _run_ways(ov)
	await _run_rivers(ov)


## An OCEAN cell inside the plate with no pin, label or way within reach, so a
## right-click there is a bare-ground card. `Vector2.ZERO` when none is found
## (the caller asserts that, rather than skipping silently). Never returns a
## land or lake cell: `sample_cell`'s `water` must read exactly `"ocean"`.
func _ocean_pos(ov: Control, gw: int, gh: int) -> Vector2:
	var bridge = app.bridge
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	for yi in range(1, 24):
		for xi in range(1, 24):
			var gx := float(gw) * float(xi) / 24.0
			var gy := float(gh) * float(yi) / 24.0
			if String(bridge.sample_cell(int(gx), int(gy)).get("water", "")) != "ocean":
				continue
			var p: Vector2 = ov._cell_to_screen(Vector2(gx, gy), rect)
			if inter.grow(-60.0).has_point(p) and ov.hits_at(p).is_empty():
				return p
	return Vector2.ZERO


## The faction-row gate, asserted from the engine's own reading of the clicked
## cell. With a card open at screen position `pos`: re-derives the cell through
## the overlay's `_grid_point` (the broker's own path), reads `sample_cell`, and
## asserts that "Open <faction>…" and "Claim for <faction>" are drawn exactly
## when the key `controlling_faction` is present -- and, for the absent side,
## that NO faction's "Open <name>…" and no "Claim for " row is drawn.
## `expect_claimed` is the fixture's own expectation, asserted separately so an
## iff that holds vacuously (a world that stopped claiming the cell) is caught.
##
## Protects: `civilization_workspace.gd::context_actions`'s faction rows and
## their one gate. Verified by mutation 2026-10-06 (each run alone against
## `context_actions`, restored after; 139/140, 135/140 and 138/140 passed):
## (1) gate removed (`if true:`) -> only the GATE leg's "CIVIL rows after the
## faction block are still drawn" fails (the ocean cell's missing key aborts
## the provider, so the absence checks alone would pass on a crash);
## (2) gate inverted (`not cf.has(...)`) -> the TR, REG and GATE legs fail;
## (3) the old `hit < 0` clause restored -> the REG settlement leg fails.
func _faction_rows_match(tag: String, ov: Control, pos: Vector2, expect_claimed: bool) -> void:
	var rect: Rect2 = ov._displayed_rect()
	var g: Dictionary = ov._grid_point(pos, rect, ov._interior_rect(rect))
	var cell: Dictionary = app.bridge.sample_cell(int(g["gx"]), int(g["gy"]))
	var claimed: bool = cell.has("controlling_faction")
	_ok("%s fixture: the engine %s a controlling faction at the clicked cell" % [tag, "reads" if expect_claimed else "reads no"],
		claimed, expect_claimed)
	var texts := _texts(["action", "disabled"])
	if claimed:
		var fname := String(cell["controlling_faction_name"])
		_ok_true("%s 'Open <faction>…' is drawn when the engine reads a controlling faction" % tag,
			texts.has("Open %s…" % fname))
		_ok_true("%s 'Claim for <faction>' is drawn when the engine reads a controlling faction" % tag,
			texts.has("Claim for %s" % fname))
	else:
		var open_rows := 0
		for fd in app.bridge.get_factions():
			var roster_name := String((fd as Dictionary).get("name", ""))
			if roster_name != "" and texts.has("Open %s…" % roster_name):
				open_rows += 1
		var claim_rows := 0
		for t in texts:
			if String(t).begins_with("Claim for "):
				claim_rows += 1
		_ok("%s no 'Open <faction>…' row where the engine reads no controlling faction" % tag, open_rows, 0)
		_ok("%s no 'Claim for <faction>' row where the engine reads no controlling faction" % tag, claim_rows, 0)


## Every disabled row on the open card carries a non-empty reason.
func _disabled_have_reasons(tag: String) -> void:
	var c = _card()
	if c == null or not c.visible:
		return
	var n := 0
	var bad := 0
	for r in c.drawn_rows():
		if String(r["kind"]) == "disabled":
			n += 1
			if String(r.get("reason", "")).strip_edges() == "":
				bad += 1
	_ok("%s every disabled row (%d) carries a reason" % [tag, n], bad, 0)


func _header_text() -> String:
	var c = _card()
	if c == null:
		return ""
	for r in c.drawn_rows():
		if String(r["kind"]) == "header":
			return String(r["text"])
	return ""


## The drawn curve's middle, for a way from `bridge.roads()`.
func _mid(points: PackedVector2Array) -> Vector2:
	return points[points.size() / 2]


## WY: way pick, Inspect / Delete rows, an undoable delete (Ruling BA), for a
## generated way and a hand-drawn one.
func _run_ways(ov: Control) -> void:
	var bridge = app.bridge
	app.select_domain("civilization")
	app.arm_tool("inspect")
	await _frames(4)
	var ppc: float = ov.label_px_per_cell()
	var tol := 6.0 / ppc if ppc > 0.0 else 1.5

	## A generated way: the longest drawn road, at the middle of its curve.
	var best: Dictionary = {}
	for r in bridge.roads():
		if bool((r as Dictionary).get("manual", false)):
			continue
		if best.is_empty() or (r["points"] as PackedVector2Array).size() > (best["points"] as PackedVector2Array).size():
			best = r
	_ok_true("WY a generated road exists", not best.is_empty())
	if best.is_empty():
		return
	var gp := _mid(best["points"])
	var picked: Dictionary = bridge.way_pick(gp.x, gp.y, tol)
	_ok_true("WY way_pick hits the road at its own drawn midpoint", not picked.is_empty())
	if picked.is_empty():
		return
	print("WY picked %s" % str(picked))
	_ok("WY the pick lands on the line (dist < 0.05 cells)", float(picked["dist"]) < 0.05, true)
	var store := String(picked["store"])
	var widx := int(picked["index"])
	_ok("WY the longest drawn road is a generated way", store, "generated")
	var before: Dictionary = bridge.way_get(store, widx)
	var title: String = CivilizationWorkspace.way_title(before)
	var rp: Vector2 = ov._point_to_screen(gp, ov._displayed_rect())
	var req: Dictionary = ov.request_at(rp, "mouse")
	var way_hits: Array = (req.get("hits", []) as Array).filter(func(h): return String(h["kind"]) == "way")
	_ok_true("WY hits[] carries the way (so Select ▸ lists it)", way_hits.size() == 1)
	if way_hits.size() == 1:
		_ok("WY the hit carries the engine's store", String(way_hits[0].get("store", "")), store)
		_ok("WY the hit carries the engine's index", int(way_hits[0].get("index", -1)), widx)
	await _rmb(ov, rp)
	_dump("WY_generated")
	_disabled_have_reasons("WY generated card")
	_ok_true("WY Inspect row is drawn", _row("Inspect %s" % title) != null)
	var del := _row("Delete %s — generated; Generate roads brings it back" % title)
	_ok_true("WY Delete row says a generated way comes back with Generate roads", del != null)
	await _click_row(_row("Inspect %s" % title))
	var hint := String(app._status_labels["hint"].text)
	_ok_true("WY Inspect wrote the way's km and origin into the hint", hint.contains("generated network"))
	rp = ov._point_to_screen(gp, ov._displayed_rect())
	await _rmb(ov, rp)
	var n_roads: int = bridge.roads().size()
	var n_drawn: int = (ov.get("_roads") as Array).size()
	await _click_row(_row("Delete %s" % title))
	_ok("WY Delete removed one drawn road (engine)", bridge.roads().size(), n_roads - 1)
	_ok("WY Delete removed one drawn road (overlay re-read)", (ov.get("_roads") as Array).size(), n_drawn - 1)
	var after_del: Dictionary = bridge.way_get(store, widx)
	_ok_true("WY the index now holds a different way (or none)", after_del.is_empty() or
		(after_del["points"] as PackedVector2Array) != (before["points"] as PackedVector2Array))
	var rows: Array = bridge.undo_ledger()
	var last: Dictionary = rows[rows.size() - 1] if rows.size() > 0 else {}
	_ok("WY the ledger's newest row is a way row", String(last.get("kind", "")), "way")
	_ok("WY ...and it is reversible", bool(last.get("reversible", false)), true)
	_ok("WY ...labelled for the way", String(last.get("label", "")), "Delete %s" % title)
	_ok("WY Undo would revert the way next", bridge.undo_next_subsystem(), "civ")
	_ok("WY Undo's label names it", bridge.undo_label(), "Delete %s" % title)
	## Edit ▸ Undo's own destination (`menus.gd` ID_UNDO -> `app.undo_last()`).
	app.undo_last()
	await _frames(3)
	var restored: Dictionary = bridge.way_get(store, widx)
	_ok("WY Undo restored the same points", restored.get("points"), before.get("points"))
	_ok("WY Undo restored the same breaks", restored.get("brks"), before.get("brks"))
	_ok("WY Undo restored the same type", restored.get("way_type"), before.get("way_type"))
	_ok("WY Undo restored the same km", restored.get("km"), before.get("km"))
	_ok("WY Undo restored the same tid", restored.get("tid"), before.get("tid"))
	_ok("WY Undo restored the same name", restored.get("name"), before.get("name"))
	var pts: PackedVector2Array = restored.get("points", PackedVector2Array())
	var bpts: PackedVector2Array = before.get("points", PackedVector2Array())
	if pts.size() > 0 and bpts.size() > 0:
		_ok("WY same first endpoint", pts[0], bpts[0])
		_ok("WY same last endpoint", pts[pts.size() - 1], bpts[bpts.size() - 1])
	_ok("WY the road is drawn again (overlay)", (ov.get("_roads") as Array).size(), n_drawn)
	var rows2: Array = bridge.undo_ledger()
	_ok_true("WY the way row left the ledger with the undo",
		rows2.filter(func(r): return String(r.get("kind", "")) == "way").is_empty())
	_ok("WY nothing way-shaped is left to undo", bridge.undo_next_subsystem() != "civ", true)

	## A settlement takes the card; the road ending on it is reached through
	## Select ▸, which narrows the request to the way alone.
	var srect: Rect2 = ov._displayed_rect()
	var sinter: Rect2 = ov._interior_rect(srect)
	var ss: Array = ov.get("_settlements")
	var s_pos := Vector2.ZERO
	var s_way: Dictionary = {}
	for i in ss.size():
		var cand: Vector2 = ov._cell_to_screen(Vector2(ss[i]["x"], ss[i]["y"]), srect)
		if not sinter.grow(-80.0).has_point(cand):
			continue
		var hs: Array = ov.request_at(cand, "mouse").get("hits", [])
		var ws: Array = hs.filter(func(h): return String(h["kind"]) == "way")
		if not ws.is_empty() and not hs.filter(func(h): return String(h["kind"]) == "settlement").is_empty():
			s_pos = cand
			s_way = ws[0]
			break
	_ok_true("WY a settlement with a road under its pin exists", not s_way.is_empty())
	if not s_way.is_empty():
		var swt := String(s_way.get("label", ""))
		await _rmb(ov, s_pos)
		_dump("WY_settlement")
		_ok_true("WY on a settlement the road's Delete row is not drawn", _row("Delete %s" % swt) == null)
		var sel := _row("Select — ", ["select"])
		_ok_true("WY ...but Select ▸ is offered", sel != null)
		await _click_row(sel)
		var pick_row := _row("%s · way" % swt, ["action"])
		_ok_true("WY Select ▸ lists the road", pick_row != null)
		await _click_row(pick_row)
		_dump("WY_selected_way")
		_ok("WY picking it re-resolves the header to the way", _header_text(), "%s · way" % swt)
		_ok_true("WY ...with the way's Inspect row", _row("Inspect %s" % swt) != null)
		_ok_true("WY ...and its Delete row", _row("Delete %s" % swt) != null)
		_ok_true("WY ...and not the settlement's Edit row", _row("Edit ") == null)
		await _close()

	## A hand-drawn way: committed here, over two land cells ~20 cells apart.
	var g: Vector2i = bridge.grid_size()
	var a := Vector2(-1, -1)
	var b := Vector2(-1, -1)
	for tries in 200:
		var x := int(g.x * 0.2) + tries
		var y := int(g.y * 0.55)
		if x + 20 >= g.x:
			break
		if String(bridge.sample_cell(x, y).get("water", "")) == "land" \
				and String(bridge.sample_cell(x + 20, y).get("water", "")) == "land":
			a = Vector2(x + 0.5, y + 0.5)
			b = Vector2(x + 20.5, y + 0.5)
			break
	_ok_true("WY two land cells for a hand-drawn way", a.x >= 0.0)
	if a.x < 0.0:
		return
	bridge.way_begin("track")
	bridge.way_append_point(a.x, a.y)
	bridge.way_append_point(b.x, b.y)
	var midx: int = bridge.way_commit()
	_ok_true("WY a hand-drawn way committed", midx >= 0)
	if midx < 0:
		return
	civ_refresh()
	await _frames(3)
	var mb: Dictionary = bridge.way_get("manual", midx)
	var mroad: Dictionary = {}
	for r in bridge.roads():
		if bool((r as Dictionary).get("manual", false)):
			mroad = r
	var mp := _mid(mroad["points"])
	var mpick: Dictionary = bridge.way_pick(mp.x, mp.y, tol)
	_ok("WY the hand-drawn way picks as manual", String(mpick.get("store", "")), "manual")
	_ok("WY ...at its own index", int(mpick.get("index", -1)), midx)
	var mtitle: String = CivilizationWorkspace.way_title(mb)
	_ok("WY an unnamed way is described, not named", mtitle, "unnamed track")
	var mrp: Vector2 = ov._point_to_screen(mp, ov._displayed_rect())
	await _rmb(ov, mrp)
	_dump("WY_manual")
	_disabled_have_reasons("WY manual card")
	var mdel := _row("Delete %s" % mtitle)
	_ok_true("WY manual Delete row is drawn", mdel != null)
	if mdel != null:
		_ok_true("WY ...and does not claim a rebuild brings it back",
			not String(mdel.get_meta("card_row", {}).get("text", "")).contains("generated"))
	await _click_row(mdel)
	_ok_true("WY manual Delete removed it", bridge.way_get("manual", midx).is_empty())
	app.undo_last()
	await _frames(3)
	var mr: Dictionary = bridge.way_get("manual", midx)
	_ok("WY manual Undo restored the same points", mr.get("points"), mb.get("points"))
	_ok("WY manual Undo restored the same type", mr.get("way_type"), "track")
	_ok("WY manual Undo restored the same km", mr.get("km"), mb.get("km"))
	## The history panel's revert-to on a way row (`undo_revert_to`, the same
	## unified step as Edit ▸ Undo).
	_ok_true("WY a second delete of the hand-drawn way", bridge.way_delete("manual", midx))
	var lrows: Array = bridge.undo_ledger()
	var wrow: Dictionary = lrows[lrows.size() - 1]
	_ok("WY its ledger row offers exactly one step", int(wrow.get("steps", 0)), 1)
	_ok("WY revert-to that row reverts one step", bridge.undo_revert_to(int(wrow["seq"])), 1)
	_ok("WY ...and the way is back", bridge.way_get("manual", midx).get("points"), mb.get("points"))
	civ_refresh()
	await _close()


func civ_refresh() -> void:
	var civ_ws = _ws("civilization_workspace.gd")
	if civ_ws != null:
		civ_ws.on_ways_changed()


## Is `c` the trace ink (within a margin for antialiasing)?
func _is_trace_ink(c: Color, ink: Color) -> bool:
	return absf(c.r - ink.r) < 0.12 and absf(c.g - ink.g) < 0.12 and absf(c.b - ink.b) < 0.12


## Any trace-ink pixel within +-r px of `p`.
func _ink_near(img: Image, p: Vector2, ink: Color, r: int = 1) -> bool:
	for dy in range(-r, r + 1):
		for dx in range(-r, r + 1):
			var x := int(p.x) + dx
			var y := int(p.y) + dy
			if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height():
				if _is_trace_ink(img.get_pixel(x, y), ink):
					return true
	return false


## RV: river pick as the branch to its mouth (Ruling BA), Trace downstream
## measured on screen against the engine's own path, Show catchment against
## the engine's own upstream set.
func _run_rivers(ov: Control) -> void:
	var bridge = app.bridge
	app.select_domain("world")
	app.arm_tool("inspect")
	await _frames(4)
	var ppc: float = ov.label_px_per_cell()
	var tol := 6.0 / ppc if ppc > 0.0 else 1.5
	## The longest order >= 2 river, a third of the way down it -- far enough
	## from its head that there is a real branch below.
	var best: Dictionary = {}
	for r in bridge.rivers(1):
		if int(r.get("order", 0)) < 2 or r.has("parallel_of"):
			continue
		if best.is_empty() or (r["points"] as PackedVector2Array).size() > (best["points"] as PackedVector2Array).size():
			best = r
	_ok_true("RV an order >= 2 river exists", not best.is_empty())
	if best.is_empty():
		return
	var rpts: PackedVector2Array = best["points"]
	var gp: Vector2 = rpts[rpts.size() / 3]
	var t0 := Time.get_ticks_usec()
	var pick: Dictionary = bridge.river_pick(gp.x, gp.y, tol)
	var pick_us := Time.get_ticks_usec() - t0
	print("RV river_pick took %d us at %s (terminus %s, %d cells)" % [pick_us, str(gp), pick.get("terminus", "?"), int(pick.get("cells", 0))])
	_ok_true("RV river_pick hits the traced river at one of its own cells", not pick.is_empty())
	if pick.is_empty():
		return
	var g: Vector2i = bridge.grid_size()
	_ok("RV the picked cell is the clicked river cell", int(pick["cell"]), int(gp.y) * g.x + int(gp.x))
	var branch: PackedVector2Array = pick["branch"]
	_ok("RV the branch starts at the clicked cell", branch[0], gp)
	_ok_true("RV the branch runs downstream (more than one cell)", branch.size() > 1)
	var term := String(pick["terminus"])
	if term == "sea" or term == "lake":
		var last := branch[branch.size() - 1]
		var w := String(bridge.sample_cell(int(last.x), int(last.y)).get("water", ""))
		_ok("RV the branch's last cell is the water it names", w, "ocean" if term == "sea" else "lake")
	## The river's own traced run continues downstream along the same
	## receivers, so the branch contains the rest of that run's points in order.
	var k := rpts.size() / 3
	var run_ok := true
	for i in range(k, rpts.size()):
		if i - k >= branch.size() or branch[i - k] != rpts[i]:
			run_ok = false
			break
	_ok_true("RV the branch follows the river's own traced run to that run's end", run_ok)

	var rect: Rect2 = ov._displayed_rect()
	var sp: Vector2 = ov._point_to_screen(gp, rect)
	var req: Dictionary = ov.request_at(sp, "mouse")
	var rh: Array = (req.get("hits", []) as Array).filter(func(h): return String(h["kind"]) == "river")
	_ok_true("RV hits[] carries the river", rh.size() == 1)
	var want_title: String = load("res://shell/context_broker.gd").river_title(pick)
	if rh.size() == 1:
		_ok("RV the hit is described, not named", String(rh[0].get("label", "")), want_title)
		_ok_true("RV ...as 'River (order N, …)'", want_title.begins_with("River (order %d, " % int(pick["order"])))
	await _rmb(ov, sp)
	_dump("RV_river")
	_disabled_have_reasons("RV card")
	_ok("RV the card's header names the river's description", _header_text().begins_with(want_title), true)
	_ok_true("RV Inspect river row is drawn", _row("Inspect river") != null)
	_ok_true("RV Trace downstream row is drawn", _row("Trace downstream") != null)
	_ok_true("RV Show catchment row is drawn", _row("Show catchment") != null)
	await _click_row(_row("Inspect river"))
	var rd = app.right_dock_ctrl
	_ok("RV Inspect river opened the right dock's River context", String(rd.get("_context")), String(rd.CTX_RIVER))
	_ok("RV ...on a traced run through the clicked cell (its points include it)",
		(rd.get("_river").get("points", PackedVector2Array()) as PackedVector2Array).has(gp), true)
	await _frames(3)
	## Re-resolved after Inspect: the right dock's context change can move the
	## map's rect. Then the screen before any highlight -- the control state
	## the trace must move.
	rect = ov._displayed_rect()
	sp = ov._point_to_screen(gp, rect)
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var img0: Image = _vp.get_texture().get_image()
	var ink: Color = ov.RIVER_TRACE_INK
	var screen_pts: Array = []
	for i in range(1, branch.size() - 1):
		screen_pts.append(xf * ov._point_to_screen(branch[i], rect))
	var pre_hits := 0
	for p in screen_pts:
		if _ink_near(img0, p, ink):
			pre_hits += 1
	await _rmb(ov, sp)
	await _click_row(_row("Trace downstream"))
	_ok("RV the overlay's trace is exactly the engine's branch", ov.river_trace(), branch)
	await _frames(3)
	var img_lab: Image = _vp.get_texture().get_image()
	var lab_hits := 0
	for p in screen_pts:
		if _ink_near(img_lab, p, ink):
			lab_hits += 1
	## The trace is drawn over every mark and under the labels only
	## (`map_overlay.gd::_draw_river_trace`'s own placement), so a label's
	## glyphs legitimately cover some of it. The line-vs-path measurement is
	## taken with the label layer emptied, and the labelled figure is reported
	## beside it rather than folded into the threshold.
	ov.set_labels([])
	await _frames(3)
	var img1: Image = _vp.get_texture().get_image()
	var on_hits := 0
	for p in screen_pts:
		if _ink_near(img1, p, ink):
			on_hits += 1
	## The same river ABOVE the clicked cell is not part of the branch and
	## must not be inked -- the trace is the stretch downstream, not the river.
	var up_pts: Array = []
	for i in range(1, k - 1):
		up_pts.append(xf * ov._point_to_screen(rpts[i], rect))
	var up_hits := 0
	for p in up_pts:
		if _ink_near(img1, p, ink, 0):
			up_hits += 1
	var frac := float(on_hits) / maxf(1.0, float(screen_pts.size()))
	print("RV trace ink at %d of %d interior branch points on screen (%.1f%%) with labels hidden; %d with labels drawn; %d before the trace; %d of %d upstream points of the same river" % [
		on_hits, screen_pts.size(), 100.0 * frac, lab_hits, pre_hits, up_hits, up_pts.size()])
	_ok_true("RV trace ink was absent on the branch before the trace (control)", pre_hits == 0)
	_ok_true("RV trace ink covers >= 95% of the branch's own points on screen", frac >= 0.95)
	_ok_true("RV the river upstream of the click is not inked (upstream points exist)", up_pts.size() > 0 and up_hits == 0)
	ov.set_labels(bridge.labels_render_list())
	await _frames(2)
	## Off the line: 8 local px perpendicular to the first segment.
	if branch.size() >= 3:
		var a: Vector2 = ov._point_to_screen(branch[1], rect)
		var b: Vector2 = ov._point_to_screen(branch[2], rect)
		var nrm: Vector2 = (b - a).orthogonal().normalized()
		var off_px: Vector2 = xf * a + nrm * 8.0
		_ok_true("RV no trace ink 8 px off the line", not _ink_near(img1, off_px, ink, 0))

	## Show catchment: the engine's own upstream set, and its shading on screen.
	var cell := int(pick["cell"])
	var mask: PackedByteArray = bridge.river_catchment(cell)
	_ok("RV river_catchment is one byte per cell", mask.size(), g.x * g.y)
	var n_up := mask.count(1)
	print("RV catchment: %d cells upstream of cell %d" % [n_up, cell])
	_ok("RV the clicked cell is in its own catchment", mask[cell] if mask.size() > cell else -1, 1)
	_ok_true("RV the catchment is more than the cell itself", n_up > 1)
	## Pre-roll, the confound found 2026-10-06 (`_ctxpicks` failed "RV <= 2% of
	## unmasked samples moved" at HEAD with 5137 of 5461 moved): the sentence
	## `_catchment_river_ctx` writes to the status bar's `hint` slot is longer
	## than the one the trace left there, and the status bar's HBox takes the
	## SUM of its labels as its minimum width -- at this probe's 1600 px
	## viewport that is 1709 px, so the whole shell (map rect, right dock, top
	## bar) reflowed between the baseline frame and the catchment frame and
	## every pixel of the map moved. Measured by a control (6 idle frames: 0 of
	## 29 541 samples moved) and by walking the tree for min widths > 1601.
	## The shading assertions below are about the catchment wash, so the
	## baseline is taken AFTER the shell has already absorbed that sentence:
	## show the catchment once, read the sentence back, clear, restore the
	## sentence, then measure the real Show against a frame laid out the same
	## way. Nothing is loosened: the thresholds are unchanged, and the reflow
	## itself is asserted absent below (`RV the shell did not reflow ...`) so a
	## baseline that drifts again cannot pass silently. The defect itself --
	## a long hint widens the whole shell past a 1600 px window -- is fixed in
	## `dcc_shell.gd` (the status bar's `_status_tail`, `_fit_status_tail()`) and
	## is now a hard assertion just below, not a printed KNOWN line.
	sp = ov._point_to_screen(gp, ov._displayed_rect())
	await _rmb(ov, sp)
	_ok_true("RV Clear river highlight is offered once something is drawn", _row("Clear river highlight") != null)
	await _click_row(_row("Show catchment"))
	await _frames(3)
	var long_hint: String = app.status_slot_text("hint")
	_ok_true("RV the catchment wrote its sentence to the status hint", long_hint.begins_with("Catchment: "))
	var wide_min: float = app.status_row.get_combined_minimum_size().x
	_ok_true("Protects: a long status hint never raises the status bar's minimum width past the window (was 1703 px at 1600; measured %.0f px)" % wide_min,
		wide_min <= float(_vp.size.x))
	var hint_lbl: Label = app._status_labels["hint"]
	_ok_true("Protects: the hint label is still drawn (width %.0f px) -- clip_text alone would have collapsed it to nothing" % hint_lbl.size.x,
		hint_lbl.visible and hint_lbl.size.x > 40.0)
	_ok("Protects: the full catchment sentence is the status tail's tooltip when it is ellipsised",
		app._status_tail.tooltip_text, long_hint)
	sp = ov._point_to_screen(gp, ov._displayed_rect())
	await _rmb(ov, sp)
	await _click_row(_row("Clear river highlight"))
	app.set_status("hint", long_hint, "text_ghost")
	await _frames(3)
	rect = ov._displayed_rect()
	xf = ov.get_global_transform_with_canvas()
	var img_pre: Image = _vp.get_texture().get_image()
	var min_before: float = app.status_row.get_combined_minimum_size().x
	sp = ov._point_to_screen(gp, ov._displayed_rect())
	await _rmb(ov, sp)
	await _click_row(_row("Show catchment"))
	_ok_true("RV the overlay holds a catchment", ov.has_river_catchment())
	await _frames(3)
	_ok("RV the shell did not reflow between the baseline and the catchment frame (status bar min width)",
		app.status_row.get_combined_minimum_size().x, min_before)
	var img2: Image = _vp.get_texture().get_image()
	## Sample every cell of the grid at its centre, away from the branch line:
	## a masked cell's pixel must move toward the catchment ink, an unmasked
	## one must not move at all.
	var moved_in := 0
	var n_in := 0
	var moved_out := 0
	var n_out := 0
	var trace_px: Dictionary = {}
	for p in branch:
		trace_px[Vector2i(int(p.x), int(p.y))] = true
	var step := maxi(1, int(sqrt(float(g.x * g.y) / 4000.0)))
	for y in range(0, g.y, step):
		for x in range(0, g.x, step):
			if trace_px.has(Vector2i(x, y)):
				continue
			var q: Vector2 = xf * ov._cell_to_screen(Vector2(x, y), rect)
			if q.x < 0 or q.y < 0 or q.x >= img2.get_width() or q.y >= img2.get_height():
				continue
			var c0 := img_pre.get_pixelv(Vector2i(q))
			var c2 := img2.get_pixelv(Vector2i(q))
			var moved := (c2.r - c0.r) > 0.02 or (c0.g - c2.g) > 0.02
			if mask[y * g.x + x] == 1:
				n_in += 1
				if moved:
					moved_in += 1
			else:
				n_out += 1
				if (absf(c2.r - c0.r) + absf(c2.g - c0.g) + absf(c2.b - c0.b)) > 0.02:
					moved_out += 1
	print("RV catchment shading: %d of %d masked samples moved toward the ink; %d of %d unmasked samples moved" % [
		moved_in, n_in, moved_out, n_out])
	_ok_true("RV masked cells were sampled", n_in > 0)
	_ok_true("RV >= 90% of masked samples shaded", n_in > 0 and float(moved_in) / float(n_in) >= 0.9)
	_ok_true("RV <= 2% of unmasked samples moved", n_out > 0 and float(moved_out) / float(n_out) <= 0.02)

	sp = ov._point_to_screen(gp, ov._displayed_rect())
	await _rmb(ov, sp)
	await _click_row(_row("Clear river highlight"))
	_ok_true("RV Clear river highlight cleared both", not ov.river_highlight_active())
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
	if DisplayServer.get_name() == "headless":
		print("### CTXPICKS ABORT: headless -- the RV leg reads the framebuffer. Run windowed. ###")
		get_tree().quit(2)
		return
	## `MISTAKES.md` "Assert on pixels": force the palette rather than name it.
	## The trace/catchment inks are fixed map inks, but the frame they are read
	## from is not.
	if DccTheme.is_dark():
		DccTheme.apply_theme(false)
		app.rebuild_theme(true)
	print("palette forced: %s" % ("dark" if DccTheme.is_dark() else "light"))

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
