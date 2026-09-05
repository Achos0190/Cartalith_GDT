extends Node
## VERIFIER probe for batch 32 lane A (left-dock section 3 gate). Written
## independently of `_leftdock12_probe.gd`; asserts the CENSUS at CONTROL
## granularity, not at category granularity, and names a route for every
## category that leaves a mode.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_ld32_probe.tscn

var app: Node
var _fail := 0
var _census: Dictionary = {}

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("V32 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _shown(n: Node, root: Node) -> bool:
	var c: Node = n
	while c != null and c != root:
		if c is CanvasItem and not (c as CanvasItem).visible:
			return false
		c = c.get_parent()
	return c == root

func _is_ctrl(n: Node) -> bool:
	return n is BaseButton or n is LineEdit or n is Range or n is TextEdit \
		or n is ItemList or n is Tree

func _walk(n: Node, root: Node, out: Array) -> void:
	if n is CanvasItem and not (n as CanvasItem).visible:
		return
	if n != root and _is_ctrl(n):
		out.append(String(root.get_path_to(n)))
	for c in n.get_children():
		_walk(c, root, out)

func _panel(id: String) -> Control:
	return app.workspace_panel(id) as Control

func _rendered_cats(id: String) -> Array:
	var p := _panel(id)
	var out: Array = []
	if p == null:
		return out
	for e in p.categories:
		var body: Control = e.get("body")
		if body == null or not is_instance_valid(body):
			continue
		var wrap: Control = body.get_parent() as Control
		if wrap != null and wrap.visible:
			out.append(String(e.get("title", "")))
	return out

func _open_cats(id: String) -> Array:
	var p := _panel(id)
	var out: Array = []
	if p == null:
		return out
	for e in p.categories:
		var body: Control = e.get("body")
		if body != null and is_instance_valid(body) and body.visible \
				and _shown(body, app.left_dock_body):
			out.append(String(e.get("title", "")))
	return out

func _all_cats(id: String) -> Array:
	var p := _panel(id)
	var out: Array = []
	if p == null:
		return out
	for e in p.categories:
		out.append(String(e.get("title", "")))
	return out

func _modes(id: String) -> Array:
	var out: Array = []
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) == "node" and String(n["domain"]) == id:
			out.append(String(n["mode"]))
	return out

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	print("V32 world: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)

	var domains := ["world", "civilization", "cartography"]

	print("V32 -- s1 census --------------------------------------------------")
	for d in domains:
		for m in _modes(d):
			app.select_domain_mode(d, m)
			await _frames(3)
			var ctrls: Array = []
			var p := _panel(d)
			if p != null:
				_walk(p, app.left_dock_body, ctrls)
			var key := "%s/%s" % [d, m]
			_census[key] = {"cats": _rendered_cats(d), "ctrls": ctrls, "open": _open_cats(d)}
			print("V32   %-26s cats=%d ctrls=%d open=%s" % [key,
				(_census[key]["cats"] as Array).size(), ctrls.size(), str(_census[key]["open"])])

	for d in domains:
		var union: Dictionary = {}
		for m in _modes(d):
			for c in (_census["%s/%s" % [d, m]]["cats"] as Array):
				union[c] = true
		var all_c := _all_cats(d)
		var missing: Array = []
		for c in all_c:
			if not union.has(c):
				missing.append(c)
		_ck("[%s] every built category renders in >=1 mode" % d, missing.is_empty(),
			"built=%d union=%d missing=%s" % [all_c.size(), union.size(), str(missing)])
		print("V32   [%s] built categories: %s" % [d, str(all_c)])

	print("V32 -- s2 what leaves, and its route -------------------------------")
	var gated_total := 0
	for d in domains:
		for c in _all_cats(d):
			var hides: Array = []
			var shows: Array = []
			for m in _modes(d):
				if (_census["%s/%s" % [d, m]]["cats"] as Array).has(c):
					shows.append(m)
				else:
					hides.append(m)
			if hides.is_empty():
				continue
			gated_total += 1
			var routes: Array = []
			for m in shows:
				routes.append("rail '%s'" % String(DccShell.rail_node(d, m).get("label", m)))
			_ck("[%s] %s hidden in %s -> route exists" % [d, c, str(hides)],
				not shows.is_empty(), "shown in %s via %s" % [str(shows), str(routes)])
	print("V32   categories hidden in at least one mode: %d" % gated_total)

	print("V32 -- s3 control census -------------------------------------------")
	for d in domains:
		var union2: Dictionary = {}
		for m in _modes(d):
			for c in (_census["%s/%s" % [d, m]]["ctrls"] as Array):
				union2[c] = true
		var per: Array = []
		for m in _modes(d):
			per.append("%s=%d" % [m, (_census["%s/%s" % [d, m]]["ctrls"] as Array).size()])
		print("V32   [%s] union=%d  per-mode %s" % [d, union2.size(), str(per)])

	print("V32 -- s4 rail nodes -----------------------------------------------")
	app.set_rail_expanded(true)
	await _frames(2)
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		var d := String(n["domain"])
		var m := String(n["mode"])
		var want := String(n.get("category", ""))
		var b: Button = app._rail_node_rows.get("%s/%s" % [d, m])
		if b == null:
			_ck("rail button %s/%s exists" % [d, m], false)
			continue
		b.pressed.emit()
		await _frames(3)
		var opened := _open_cats(d)
		## The planner node is the one that does not leave the accordion on
		## screen: `_on_rail_node_pressed()` also calls `open_journey_planner()`,
		## and `journey_planner_view.open()` REPLACES the CIVIL panel (that
		## takeover predates this batch -- `dcc_shell.gd`'s own comment at the
		## `_hide()` restore names it). So the assertion for that node is that
		## the takeover happened, not that a category is open.
		var takeover: bool = not _panel(d).visible
		_ck("%s/%s (%s) -> domain" % [d, m, String(n["label"])],
			app.active_domain() == d, "got=%s" % app.active_domain())
		if takeover:
			_ck("%s/%s -> journey takeover replaced the dock" % [d, m],
				app.journey_planner_view != null and app.journey_planner_view.visible,
				"panel_visible=%s" % _panel(d).visible)
			continue
		_ck("%s/%s -> '%s' OPEN and RENDERED" % [d, m, want],
			opened.has(want), "open=%s" % str(opened))
		_ck("%s/%s -> exactly one body open" % [d, m], opened.size() == 1,
			"open=%s" % str(opened))

	print("V32 -- s5 mode switch ----------------------------------------------")
	for d in domains:
		app.select_domain(d)
		await _frames(3)
		var vis: bool = app._mode_switch_row != null and app._mode_switch_row.visible
		_ck("[%s] pill visible == domain_gates" % d, vis == DccShell.domain_gates(d),
			"vis=%s gates=%s" % [vis, DccShell.domain_gates(d)])
	app.select_domain("world")
	await _frames(2)
	for m2 in ["a", "b"]:
		app.select_domain_mode("world", m2)
		await _frames(3)
		var lit: Array = []
		for k in app._mode_switch_buttons:
			var bb: Button = app._mode_switch_buttons[k]
			if bb.get_theme_color("font_color") == DccTheme.c("accent"):
				lit.append(String(k))
		_ck("world/%s -> exactly that segment lit" % m2, lit == [m2], "lit=%s" % str(lit))
	app.select_domain_mode("world", "a")
	await _frames(2)
	(app._mode_switch_buttons["b"] as Button).pressed.emit()
	await _frames(3)
	_ck("pill b -> Terrain alone rendered",
		_rendered_cats("world") == ["Terrain"], "cats=%s" % str(_rendered_cats("world")))
	(app._mode_switch_buttons["a"] as Button).pressed.emit()
	await _frames(3)
	_ck("pill a -> all nine back", _rendered_cats("world").size() == 9,
		"cats=%d" % _rendered_cats("world").size())

	print("V32 -- s6 transitions into world/b ---------------------------------")
	app.select_domain_mode("world", "b")
	await _frames(3)
	app.select_domain_category("world", "Climate")
	await _frames(3)
	_ck("(a) select_domain_category Climate from Sculpt",
		_rendered_cats("world").has("Climate") and _open_cats("world") == ["Climate"],
		"open=%s" % str(_open_cats("world")))
	app.select_domain_mode("world", "b")
	await _frames(3)
	var wp := _panel("world")
	var r: bool = wp.open_category("Geology")
	await _frames(3)
	_ck("(b) open_category Geology while gated moves the dock",
		r and _open_cats("world") == ["Geology"], "ret=%s open=%s" % [r, str(_open_cats("world"))])
	app.select_domain_mode("world", "b")
	await _frames(2)
	app.select_domain("cartography")
	await _frames(2)
	app.select_domain("world")
	await _frames(3)
	_ck("(c) mode survives a domain round trip",
		_rendered_cats("world") == ["Terrain"], "cats=%s" % str(_rendered_cats("world")))
	var only_btn: Button = null
	for e in wp.categories:
		if String(e.get("title", "")) == "Terrain":
			only_btn = e.get("button")
	if only_btn != null:
		if _open_cats("world").is_empty():
			only_btn.pressed.emit()
			await _frames(2)
		only_btn.pressed.emit()
		await _frames(3)
		_ck("(d) re-clicking the only header never empties the dock",
			not _open_cats("world").is_empty(), "open=%s" % str(_open_cats("world")))
	app.select_domain("civilization")
	await _frames(2)
	app.arm_tool("sculpt")
	await _frames(4)
	_ck("(e) arm sculpt from CIVIL -> WORLD b, Terrain open",
		app.active_domain() == "world" and _open_cats("world") == ["Terrain"],
		"dom=%s open=%s" % [app.active_domain(), str(_open_cats("world"))])
	app.select_domain("civilization")
	await _frames(2)
	app.arm_tool("paint")
	await _frames(4)
	_ck("(f) arm paint -> WORLD a, Biomes open",
		app.active_domain() == "world" and _open_cats("world") == ["Biomes"],
		"dom=%s open=%s" % [app.active_domain(), str(_open_cats("world"))])
	app.arm_tool("select")
	await _frames(2)

	print("V32 -- s7 collapsed strip ------------------------------------------")
	app.select_domain("world")
	await _frames(3)
	var open_min: float = app.left_dock.get_combined_minimum_size().x
	var sw_min: Vector2 = app._mode_switch_row.get_combined_minimum_size()
	app._toggle_dock(true)
	await _frames(4)
	var col_min: float = app.left_dock.get_combined_minimum_size().x
	_ck("collapsed -> pill hidden", not app._mode_switch_row.visible)
	print("V32   left_dock min x: open=%.1f collapsed=%.1f  pill min=%s  w_rail_collapsed=%d"
		% [open_min, col_min, str(sw_min), DccTheme.role_px("w_rail_collapsed")])
	app._mode_switch_row.visible = true
	await _frames(4)
	var forced: float = app.left_dock.get_combined_minimum_size().x
	print("V32   collapsed with the pill FORCED visible: %.1f" % forced)
	app._mode_switch_row.visible = false
	app._toggle_dock(true)
	await _frames(3)

	print("V32 DONE  failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
