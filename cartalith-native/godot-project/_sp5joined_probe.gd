extends Node
## SP-5 (`STORY_PLANNING_SCOPE.md` §4): "a journey that passes a settlement
## during a conflict's active years says so; a settlement's strip shows the
## conflicts that touched it." A real generated world, a real saved journey
## (SP-3 probe's technique), and real conflicts added through the bridge:
##   ACTIVE  -- a siege at a through-town, years spanning the pass year
##   LATER   -- a battle at the same town, years after the pass
##   ELSE    -- a siege at a town the route never passes
##   PROV    -- a front attached to the through-town's PROVINCE (by its seed's
##              tid), when the town is not itself that seed
## Then `conflicts_touching_settlement` and the Settlement Editor's Political
## history tab, read back as drawn Label text.
##
## WINDOWED (reads real drawn Labels):
##   Godot_v4.7.1-stable_win64_console.exe --path . _sp5joined_probe.tscn
## Exit 0 pass, 1 failure, 3 watchdog.

var app: Node
var _fail := 0
var _n := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	_n += 1
	print("SP5 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> Array:
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children(true):
		_labels(c, out)
	return out

func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children(true):
		var r := _find(c, pred)
		if r != null:
			return r
	return null

func _pos(id: int) -> Dictionary:
	for d in app.bridge.journey_positions():
		if int(d.get("id", -1)) == id:
			return d
	return {}

func _ids(rows: Array) -> Array:
	return rows.map(func(r): return int(r.get("id", -1)))

func _open_political(idx: int) -> Array:
	var pe = app.place_editor_window
	pe.open_for(idx)
	await _frames(6)
	var tab = _find(pe, func(n): return n is Button and (n as Button).text == "Political history")
	if tab != null:
		(tab as Button).pressed.emit()
	await _frames(6)
	return _labels(pe, [])

func _siege(bridge, name: String, kind: String, t: Dictionary, y0: int, y1: int, akind: String, atid: int) -> int:
	var pts := PackedVector2Array([Vector2(t["x"], t["y"])])
	if kind == "front":
		pts.append(Vector2(float(t["x"]) + 3.0, float(t["y"]) + 3.0))
	var r: Dictionary = bridge.conflict_add({"name": name, "kind": kind, "start_year": y0, "end_year": y1,
		"points": pts, "anchor_kind": akind, "anchor_tid": atid})
	if not bool(r.get("ok", false)):
		print("SP5 conflict_add refused: %s" % r)
	return int(r.get("id", -1)) if bool(r.get("ok", false)) else -1

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("SP5 WATCHDOG"); get_tree().quit(3))
	wd.start()
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("SP5 dll mtime: %s" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))
	if not app.bridge.world_gen.has_method("conflicts_touching_settlement"):
		print("SP5 ABORT: stale .dll (no conflicts_touching_settlement)")
		get_tree().quit(1)
		return

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	var bridge = app.bridge
	if not bridge.has_world:
		print("SP5 !! generate failed")
		get_tree().quit(1)
		return
	var gs: Vector2 = bridge.grid_size()
	var towns: Array = bridge.settlements()
	var presets: Array = bridge.world_gen.tl_list("preset")
	var preset_id := String((presets[0] as Dictionary).get("id", "")) if not presets.is_empty() else ""
	var year0 := 412

	# -- a real dated journey (farthest pairs first, until one plans) ---------
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var jid := -1
	for k in mini(8, pairs.size()):
		var a: Dictionary = towns[pairs[k][1]]
		var b: Dictionary = towns[pairs[k][2]]
		bridge.route_begin("mixed")
		bridge.route_append_stop(float(a["x"]), float(a["y"]))
		bridge.route_append_stop(float(b["x"]), float(b["y"]))
		var ridx: int = bridge.route_commit()
		jid = bridge.journey_save("Probe caravan", preset_id, ridx, year0)
		var try := _pos(jid)
		print("SP5 try %d: jid %d ridx %d %s" % [k, jid, ridx, try])
		if try.has("x") and float(try["total_days"]) >= 8.0:
			break
		bridge.journey_delete(jid)
		jid = -1
	_check("a dated journey saved on a real route", jid >= 0)
	if jid < 0:
		get_tree().quit(1)
		return

	## Passed towns, with their pass rows; prefer a through-town that is NOT a
	## province seed, so the province leg is reachable.
	var provs: Array = bridge.provinces()
	var seed_tids := {}
	for p in provs:
		var ci := int(p["capital_settlement_index"])
		if ci >= 0 and ci < towns.size():
			seed_tids[int(towns[ci]["tid"])] = true
	var hit_idx := -1
	var hit_pass: Dictionary = {}
	var passed := {}
	for i in towns.size():
		for d in bridge.civ_settlement_journey_passes(int(towns[i]["tid"])):
			if int(d.get("id", -1)) == jid and d.has("year"):
				passed[int(towns[i]["tid"])] = true
				if hit_idx < 0 or (seed_tids.has(int(towns[hit_idx]["tid"])) and not seed_tids.has(int(towns[i]["tid"]))):
					hit_idx = i
					hit_pass = d
	_check("a dated pass exists", hit_idx >= 0)
	if hit_idx < 0:
		get_tree().quit(1)
		return
	var town: Dictionary = towns[hit_idx]
	var tid := int(town["tid"])
	var py := int(hit_pass["year"])
	print("SP5 town %s (tid %d) passed %s (year %d); seed? %s" % [town["name"], tid, hit_pass["date"], py, seed_tids.has(tid)])
	var far_idx := -1
	for i in towns.size():
		if not passed.has(int(towns[i]["tid"])):
			far_idx = i
			break

	# -- negative control: before any conflict, nothing touches it -------------
	_check("control: no conflicts, nothing touches the town", bridge.conflicts_touching_settlement(tid).is_empty())
	var labs0: Array = await _open_political(hit_idx)
	_check("control: the empty state is drawn",
		labs0.any(func(l): return String(l).begins_with("No conflict is attached here")))
	_check("control: no 'Passing during' note without a conflict",
		not labs0.any(func(l): return String(l).begins_with("Passing during")))

	# -- the conflicts ----------------------------------------------------------
	var active := _siege(bridge, "Siege of Probeholm", "siege", town, py - 2, py + 1, "settlement", tid)
	var later := _siege(bridge, "Battle of the Late Ford", "battle", town, py + 40, py + 45, "settlement", tid)
	var elsewhere := -1
	if far_idx >= 0:
		elsewhere = _siege(bridge, "Siege Elsewhere", "siege", towns[far_idx], py - 2, py + 1, "settlement", int(towns[far_idx]["tid"]))
	_check("three settlement-anchored conflicts added", active > 0 and later > 0 and elsewhere > 0,
		str([active, later, elsewhere]))

	## The province the town lies in, by its seed. Found here independently of
	## the engine's raster lookup: the seed is the nearest same-faction seed
	## (civ_generate_provinces' own rule), so compare against that.
	var prov_seed := -1
	var bestd := INF
	for p in provs:
		var ci := int(p["capital_settlement_index"])
		if int(p["faction"]) != int(town.get("faction", -1)) or ci < 0 or ci >= towns.size():
			continue
		var dd := Vector2(towns[ci]["x"], towns[ci]["y"]).distance_squared_to(Vector2(town["x"], town["y"]))
		if dd < bestd:
			bestd = dd
			prov_seed = int(towns[ci]["tid"])
	var prov := -1
	if prov_seed > 0 and prov_seed != tid:
		var seed_town: Dictionary = towns.filter(func(t): return int(t["tid"]) == prov_seed)[0]
		prov = _siege(bridge, "Border War", "front", seed_town, py, py, "province", prov_seed)
	print("SP5 province seed tid %d, province conflict %d" % [prov_seed, prov])

	# -- (b) the query ------------------------------------------------------------
	var rows: Array = bridge.conflicts_touching_settlement(tid)
	var ids := _ids(rows)
	print("SP5 touching %d: %s" % [tid, rows.map(func(r): return [r.get("id"), r.get("name"), r.get("via"), r.get("start_year")])])
	_check("both settlement conflicts touch the town", ids.has(active) and ids.has(later))
	_check("the other town's conflict does not", not ids.has(elsewhere))
	_check("and it touches the other town", _ids(bridge.conflicts_touching_settlement(int(towns[far_idx]["tid"]))) == [elsewhere])
	if prov > 0:
		_check("the province conflict touches the town, via province", ids.has(prov)
			and rows.any(func(r): return int(r["id"]) == prov and String(r.get("via", "")) == "province"))
	else:
		print("SP5 (province leg not reachable in this world: town is its own seed or unowned)")
	var ys: Array = rows.map(func(r): return int(r["start_year"]))
	var ys_sorted := ys.duplicate()
	ys_sorted.sort()
	_check("ordered by start year", ys == ys_sorted, str(ys))
	_check("tid 0 / unknown tid: empty", bridge.conflicts_touching_settlement(0).is_empty()
		and bridge.conflicts_touching_settlement(9999999).is_empty())

	# -- drawn ---------------------------------------------------------------------
	var labs: Array = await _open_political(hit_idx)
	var hi := -1
	for i in labs.size():
		if String(labs[i]).to_upper().ends_with("CONFLICTS HERE"):
			hi = i
	_check("the 'Conflicts here' section is drawn", hi >= 0)
	var yr_active := "%d – %d" % [py - 2, py + 1]
	var ai := labs.find(yr_active, maxi(hi, 0))
	_check("the active siege's years drawn under the header, name beside", hi >= 0 and ai > hi
		and ai + 1 < labs.size() and labs[ai + 1] == "Siege of Probeholm", str(labs.slice(maxi(0, hi), mini(labs.size(), hi + 10))))
	var li := labs.find("%d – %d" % [py + 40, py + 45], maxi(hi, 0))
	_check("the later battle drawn too", li > hi and hi >= 0 and labs[li + 1] == "Battle of the Late Ford")
	_check("the elsewhere siege is not drawn", not labs.has("Siege Elsewhere"))
	if prov > 0:
		_check("the province front drawn with its province named",
			labs.any(func(l): return String(l).begins_with("Border War  (") and String(l).ends_with("Province)")))

	# -- (a) the pass annotation ------------------------------------------------
	var notes: Array = labs.filter(func(l): return String(l).begins_with("Passing during"))
	print("SP5 pass notes: %s" % [notes])
	_check("the pass is annotated with the active siege", notes.size() == 1
		and String(notes[0]).contains("Siege of Probeholm (%s)" % yr_active))
	_check("not with the later battle", notes.size() == 1 and not String(notes[0]).contains("Late Ford"))
	_check("not with the elsewhere siege", notes.size() == 1 and not String(notes[0]).contains("Elsewhere"))
	if prov > 0:
		_check("and with the province war active that year", notes.size() == 1 and String(notes[0]).contains("Border War (%d – %d)" % [py, py]))
	var pi := labs.find("Probe caravan")
	var ni := labs.find(notes[0]) if notes.size() == 1 else -1
	_check("the note sits directly under the journey row", pi >= 0 and ni == pi + 1, "%d / %d" % [pi, ni])

	# -- the annotation follows the years: move the siege off the pass year ----
	bridge.conflict_update(active, {"start_year": py + 10, "end_year": py + 12})
	if prov > 0:
		bridge.conflict_delete(prov)
	var labs3: Array = await _open_political(hit_idx)
	_check("moving the siege off the pass year removes the annotation",
		not labs3.any(func(l): return String(l).begins_with("Passing during")))
	_check("while the section still lists it", labs3.has("Siege of Probeholm"))

	print("SP5 done: %d check(s), %d failure(s)" % [_n, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
