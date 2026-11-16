extends "res://_pcl_base.gd"
## Ruling FO item 2: ONE route name on every surface, "Route N" counting from 1 (`shell/route_name.gd`;
## `PC_SHELL_PROPOSAL.md` finding M2: the card said "Journey 1" while the planner said "Route #0").
##   godot --path . _fo_route_probe.tscn -- [--force-touch] [--shots <dir>]
## Commits two nameless routes and a third that the person names, then reads each surface that names a route as drawn:
##   A. the pure helper: index 0 -> "Route 1", a typed name wins, blank restores the fallback;
##   B. the Routes & ways list rows ("Route 1", "Route 2", the typed name), and each row's rename placeholder;
##   C. the context card's rows for a click on route 0 ("Open “Route 1” in Journey Planner", Rename, Delete);
##   D. the journey planner's route picker (the one DccWidgets.choice) and nothing in the planner reading "Route #";
##   E. a sweep of every Label, Button and LineEdit placeholder in the shell: no "Route #" and no "Journey 1" fallback.
## Mutants: HELPER_ZERO (`index + 1` -> `index`), CARD_OLD (the card's helper call back to "Journey %d"), PICKER_HASH
## (the picker's helper call back to "Route #%d"), ROW_OLD (the list row's fallback back to "Journey %d").

const RN := preload("res://shell/route_name.gd")


func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	elif n is LineEdit:
		out.append((n as LineEdit).placeholder_text)
		out.append((n as LineEdit).text)
	for c in n.get_children():
		_texts(c, out)


func _commit_route(a: int, b: int) -> int:
	var towns: Array = _bridge.settlements()
	_bridge.route_begin("mixed")
	_bridge.route_append_stop(float(towns[a]["x"]), float(towns[a]["y"]))
	_bridge.route_append_stop(float(towns[b]["x"]), float(towns[b]["y"]))
	return int(_bridge.route_commit())


func _run() -> void:
	_prefix = "fo_route"
	print("== A. the helper ==")
	_ok(RN.default_name(0) == "Route 1" and RN.default_name(4) == "Route 5", "index 0 is Route 1, index 4 is Route 5")
	_ok(RN.display({"name": ""}, 1) == "Route 2" and RN.display({}, 2) == "Route 3", "a blank or absent name falls back")
	_ok(RN.display({"name": "Salt Road"}, 0) == "Salt Road", "a typed name wins")
	print("== commit three routes ==")
	var i0 := _commit_route(0, 1)
	var i1 := _commit_route(1, 2)
	var i2 := _commit_route(0, 2)
	_ok(i0 == 0 and i1 == 1 and i2 == 2, "three routes committed (%d %d %d)" % [i0, i1, i2])
	_bridge.route_set_name(2, "Salt Road")
	var infra: Node = _app.find_children("*", "InfrastructureWorkspace", true, false)[0]
	var civ: Node = _app.find_children("*", "CivilizationWorkspace", true, false)[0]
	print("== B. the Routes list ==")
	_app.select_domain_category("civilization", "Routes & ways")
	await _settle(8)
	infra._refresh_routes_teaser()
	infra._refresh_manual_routes()
	await _settle(4)
	var tx: Array = []
	_texts(_app, tx)
	var joined := "\n".join(PackedStringArray(tx))
	_ok(joined.contains("➔ Route 1") and joined.contains("➔ Route 2") and joined.contains("➔ Salt Road"), "the list rows read Route 1, Route 2 and the typed name")
	_ok(not joined.contains("➔ Journey"), "no row says Journey N")
	var ph := false
	for t in tx:
		if String(t) == "Route 1":
			ph = true
	_ok(ph, "a rename field's placeholder is Route 1")
	print("== C. the context card ==")
	var r0: Dictionary = _bridge.route_get(0)
	var pts: PackedVector2Array = r0.get("points", PackedVector2Array())
	_ok(pts.size() > 0, "route 0 has points")
	var rows: Array = civ.context_actions({"domain": "civilization", "gx": pts[0].x, "gy": pts[0].y, "hits": []})
	var labels: Array = []
	for r in rows:
		labels.append(String((r as Dictionary).get("label", "")))
	print("card rows: %s" % str(labels))
	_ok(labels.has("Open “Route 1” in Journey Planner"), "the card says Route 1")
	_ok(labels.has("Rename “Route 1”…") and labels.has("Delete “Route 1”"), "Rename and Delete name Route 1")
	print("== D. the planner ==")
	_app.open_journey_planner_with_route(0)
	await _settle(14)
	var jpv = _app.journey_planner_view
	var ptx: Array = []
	_texts(_app, ptx)
	var all_p := "\n".join(PackedStringArray(ptx))
	var picker_ok := false
	for t in ptx:
		if String(t).begins_with("Route 1 — "):
			picker_ok = true
	_ok(picker_ok, "the planner names the route Route 1")
	_ok(not all_p.contains("Route #"), "nothing in the planner reads Route #")
	await _shot("planner")
	print("== E. a sweep of the whole shell ==")
	var sw: Array = []
	_texts(_app, sw)
	var bad := 0
	for t in sw:
		if String(t).contains("Route #") or String(t).begins_with("Journey 1"):
			bad += 1
			print("  stray: %s" % t)
	_ok(bad == 0, "no 'Route #' and no 'Journey 1' fallback anywhere (%d)" % bad)
