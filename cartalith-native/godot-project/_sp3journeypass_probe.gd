extends Node
## SP-3's third mark (`STORY_PLANNING_SCOPE.md` §4): "Journeys from SP-2 that
## pass through the settlement appear as a third mark." A real generated world,
## a real committed route between two real settlements, a real saved journey on
## a stock preset -- then `civ_settlement_journey_passes` for every settlement,
## checked against the planner's own stop list, against SP-2's marker (the
## party is AT the settlement on the pass date), and drawn in the real Settlement
## Editor's Political history tab (read back as Label text).
##
## WINDOWED (reads real drawn Labels; the shell is the same as SP-2's run):
##   Godot_v4.7.1-stable_win64_console.exe --path . _sp3journeypass_probe.tscn
## Exit 0 pass, 1 failure.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SP3J %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _pos(id: int) -> Dictionary:
	for d in app.bridge.journey_positions():
		if int(d.get("id", -1)) == id:
			return d
	return {}

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

func _pass_for(bridge, tid: int, jid: int) -> Dictionary:
	for d in bridge.civ_settlement_journey_passes(tid):
		if int(d.get("id", -1)) == jid:
			return d
	return {}

## The cursor at `off` days after 1 January of `year0`.
func _goto(year0: int, off: int) -> void:
	app.tl_set_year(year0 + off / 365)
	app.tl_set_day(off % 365)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("SP3J WATCHDOG"); get_tree().quit(3))
	wd.start()
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("SP3J dll mtime: %s" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))
	if not app.bridge.world_gen.has_method("civ_settlement_journey_passes"):
		print("SP3J ABORT: stale .dll")
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
		print("SP3J !! generate failed")
		get_tree().quit(1)
		return
	var gs: Vector2 = bridge.grid_size()
	var radius := maxf(gs.x / 90.0, 3.0)   ## jp_stop_radius_cells
	print("SP3J world %dx%d, stop radius %.2f cells" % [gs.x, gs.y, radius])

	var towns: Array = bridge.settlements()
	var presets: Array = bridge.world_gen.tl_list("preset")
	var preset_id := String((presets[0] as Dictionary).get("id", "")) if not presets.is_empty() else ""
	var year0 := 412

	# -- negative control: nothing saved, nothing passes ----------------------
	var any_before := 0
	for t in towns:
		any_before += bridge.civ_settlement_journey_passes(int(t.get("tid", 0))).size()
	_check("control: with no saved journey, no settlement has a pass", any_before == 0, str(any_before))

	# -- a real long journey (farthest pairs first, until one plans) ----------
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var jid := -1
	var a: Dictionary = {}
	var b: Dictionary = {}
	for k in mini(8, pairs.size()):
		a = towns[pairs[k][1]]; b = towns[pairs[k][2]]
		bridge.route_begin("mixed")
		bridge.route_append_stop(float(a["x"]), float(a["y"]))
		bridge.route_append_stop(float(b["x"]), float(b["y"]))
		var ridx: int = bridge.route_commit()
		jid = bridge.journey_save("Probe caravan", preset_id, ridx, year0)
		var try := _pos(jid)
		if try.has("x") and float(try["total_days"]) >= 8.0:
			break
		bridge.journey_delete(jid)
		jid = -1
	_check("a dated journey saved on a real route", jid >= 0, "%s -> %s" % [a.get("name"), b.get("name")])
	if jid < 0:
		get_tree().quit(1)
		return
	var p0 := _pos(jid)
	var total := float(p0["total_days"])
	print("SP3J journey %d: %s -> %s, %.1f days, arrives %s" % [jid, a.get("name"), b.get("name"), total, p0.get("arrival")])

	# -- every settlement: which passes, and when ------------------------------
	var hits: Array = []   ## [town, pass]
	for t in towns:
		var ps := _pass_for(bridge, int(t.get("tid", 0)), jid)
		if not ps.is_empty():
			hits.append([t, ps])
	var names: Array = []
	for h in hits:
		names.append("%s@%s" % [h[0].get("name"), h[1].get("date")])
	print("SP3J passes (%d): %s" % [hits.size(), ", ".join(names)])
	_check("the origin and destination both pass", hits.any(func(h): return int(h[0]["tid"]) == int(a["tid"]))
		and hits.any(func(h): return int(h[0]["tid"]) == int(b["tid"])))
	_check("the pass count is the planner's own stop list (passes >= 2, < every town)",
		hits.size() >= 2 and hits.size() < towns.size(), "%d of %d" % [hits.size(), towns.size()])
	var origin_pass: Dictionary = _pass_for(bridge, int(a["tid"]), jid)
	_check("the origin is passed on departure day", int(origin_pass.get("day_offset", -1)) == 0
		and String(origin_pass.get("date", "")) == "%d-01-01" % year0, str(origin_pass))
	var dest_pass: Dictionary = _pass_for(bridge, int(b["tid"]), jid)
	_check("the destination is passed last, by the arrival date",
		int(dest_pass.get("day_offset", -1)) > 0 and int(dest_pass.get("day_offset", 0)) <= ceili(total),
		"%s vs total %.2f" % [dest_pass.get("day_offset"), total])
	var dated := 0
	for h in hits:
		if h[1].has("date"):
			dated += 1
	_check("every pass on a dated journey is dated", dated == hits.size(), "%d of %d" % [dated, hits.size()])

	# -- SP-2's marker agrees: on the pass date the party is AT the settlement --
	var agree := 0
	for h in hits:
		var off := int(h[1]["day_offset"])
		_goto(year0, off)
		var here := _pos(jid)
		_goto(year0, off + 1)
		var next := _pos(jid)
		var step := Vector2(here["x"], here["y"]).distance_to(Vector2(next["x"], next["y"]))
		var dist := Vector2(here["x"], here["y"]).distance_to(Vector2(h[0]["x"], h[0]["y"]))
		var ok := dist <= radius + step + 0.01
		if ok:
			agree += 1
		else:
			print("SP3J   %s: marker %.2f cells away on %s (radius %.2f, day step %.2f)" % [h[0]["name"], dist, h[1]["date"], radius, step])
	_check("on each pass date the journey marker is within the stop radius (+1 day's travel)", agree == hits.size(),
		"%d of %d" % [agree, hits.size()])

	# -- a settlement the route never comes near has no pass -------------------
	var far: Dictionary = {}
	var fard := -1.0
	for t in towns:
		var dd: float = minf(Vector2(t["x"], t["y"]).distance_to(Vector2(a["x"], a["y"])),
			Vector2(t["x"], t["y"]).distance_to(Vector2(b["x"], b["y"])))
		if dd > fard and not hits.any(func(h): return int(h[0]["tid"]) == int(t["tid"])):
			fard = dd; far = t
	_check("tid 0 / unknown tid: empty", bridge.civ_settlement_journey_passes(0).is_empty()
		and bridge.civ_settlement_journey_passes(9999999).is_empty())

	# -- drawn: the Settlement Editor's Political history tab ------------------
	## A THROUGH town if there is one (not origin/destination), else the destination.
	var show: Array = hits[hits.size() - 1]
	for h in hits:
		if int(h[0]["tid"]) != int(a["tid"]) and int(h[0]["tid"]) != int(b["tid"]):
			show = h
			break
	var idx := -1
	for i in towns.size():
		if int(towns[i]["tid"]) == int(show[0]["tid"]):
			idx = i
	var pe = app.place_editor_window
	pe.open_for(idx)
	await _frames(6)
	var tab = _find(pe, func(n): return n is Button and (n as Button).text == "Political history")
	if tab != null:
		(tab as Button).pressed.emit()
	await _frames(6)
	var labs := _labels(pe, [])
	var hi := -1
	for i in labs.size():
		if String(labs[i]).to_upper().ends_with("JOURNEYS PASSING"):
			hi = i
	_check("the section is drawn", hi >= 0, "")
	var di := labs.find(String(show[1]["date"]))
	_check("the pass date is drawn after the section header", di > hi and hi >= 0, "%s at %d" % [show[1]["date"], di])
	_check("with the journey's name beside it", di >= 0 and di + 1 < labs.size() and labs[di + 1] == "Probe caravan",
		str(labs.slice(maxi(0, di - 1), di + 3)))
	print("SP3J drawn for %s: %s" % [show[0]["name"], labs.slice(maxi(0, hi), mini(labs.size(), hi + 4))])

	pe.open_for(towns.find(far))
	await _frames(6)
	tab = _find(pe, func(n): return n is Button and (n as Button).text == "Political history")
	if tab != null:
		(tab as Button).pressed.emit()
	await _frames(6)
	var labs2 := _labels(pe, [])
	_check("a settlement the route does not pass (%s) draws the empty state instead" % far.get("name"),
		labs2.any(func(l): return String(l).begins_with("No saved journey passes here")) and not labs2.has("Probe caravan"),
		str(labs2.slice(maxi(0, labs2.size() - 6))))

	# -- after a delete, the mark is gone --------------------------------------
	bridge.journey_delete(jid)
	_check("deleting the journey removes the pass", _pass_for(bridge, int(show[0]["tid"]), jid).is_empty())

	print("SP3J done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
