extends Node
## Windowed proof that a saved journey comes back into the Journey Planner's
## list after File > Open project (`OUTSTANDING_WORK.md` §2.3, 2026-09-24).
## Until then the list started empty on every open while the journey itself
## survived only as a map marker.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _jprestore_probe.tscn
##
## Real world, real route, real save and open through the app's own paths:
##   1. a journey is saved against a committed route with a party preset whose
##      form differs from the default in one field;
##   2. the project is written (`_write_project`) and opened (`_load_project`);
##   3. the planner's list holds that journey again: its name, its engine id,
##      the SAME route index, and the preset's non-default field -- a list
##      rebuilt from defaults would fail the last check;
##   4. (owner Ruling AR, 2026-09-24) the reopened world is the saved world:
##      a journey plan over the route, the faction economy, the military
##      summary, trade flows, two town layouts and a Sample-panel reading all
##      come back byte-identical (as JSON text) to what the generated world
##      answered before the save. Until the substrate rasters were saved, every
##      one of them refused a reopened project. Also compared (2026-09-24,
##      `OUTSTANDING_WORK.md` §2.11): the SP-2 journey markers
##      (`journey_positions`) with the cursor three days into the journey, and
##      the landmark set of a landmark pass run before the save.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("JPRESTORE %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("JPRESTORE REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("JPRESTORE WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	# 1. a route, a non-default party preset, a journey
	var towns: Array = bridge.settlements()
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(towns[0]["x"]), float(towns[0]["y"]))
	bridge.route_append_stop(float(towns[1]["x"]), float(towns[1]["y"]))
	var ridx: int = bridge.route_commit()
	var plan: Dictionary = bridge.jp_default_plan()
	var key := ""
	for k in plan.keys():
		if typeof(plan[k]) == TYPE_INT and k != "party_fields":
			key = String(k)
			break
	plan[key] = int(plan[key]) + 3
	var cap: Dictionary = bridge.tl_capture_preset_from_plan("Probe party", plan)
	var jid: int = bridge.journey_save("Probe journey", String(cap.get("id", "")), ridx, 0)
	_check("setup: route committed and journey saved", ridx >= 0 and jid >= 0 and bool(cap.get("ok", false)),
		"route %d, journey %d, bumped %s to %d" % [ridx, jid, key, int(plan[key])])

	# A landmark pass, so the saved project carries a landmark set to restore.
	var lm: Dictionary = await bridge.landmark_run()
	_check("setup: a landmark pass placed landmarks", bool(lm.get("ok", false)) and int(lm.get("placed", 0)) > 0, str(lm.get("error", lm.get("placed", ""))))
	# The SP-2 markers are read at the cursor date: set it on both sides, three
	# days after the journey's departure, so the party is on the road.
	var day := 3

	# The formerly refusing readouts, on the generated world, before saving.
	var probe_pts := Vector2i(int(towns[0]["x"]), int(towns[0]["y"]))
	var readouts := func() -> Dictionary:
		return {
			"jp_compute": bridge.jp_compute({"route": ridx}),
			"civ_faction_economy": bridge.civ_faction_economy(),
			"civ_military_summary": bridge.civ_military_summary(),
			"civ_trade_flows": bridge.civ_trade_flows(),
			"urban_layouts": bridge.urban_layouts(PackedInt32Array([0, 1])),
			"sample_cell": bridge.sample_cell(probe_pts.x, probe_pts.y),
			"journey_positions": bridge.journey_positions(),
			"landmarks": bridge.landmarks(),
		}
	bridge.civ_set_day_of_year(day)
	var before: Dictionary = readouts.call()
	var markers: Array = before["journey_positions"]
	_check("generated world: the journey marker has a position", markers.size() == 1
		and (markers[0] as Dictionary).has("x") and (markers[0] as Dictionary).has("y"), str(markers))
	_check("generated world: jp_compute plans the route", bool((before["jp_compute"] as Dictionary).get("ok", false)),
		String((before["jp_compute"] as Dictionary).get("error", "")))
	for k in before.keys():
		var v = before[k]
		_check("generated world: %s is not empty" % k, (v is Dictionary and not (v as Dictionary).is_empty()) 			or (v is Array and not (v as Array).is_empty()))

	# 2. write and open the project through the app's own paths
	var path := OS.get_user_data_dir().path_join("_jprestore_probe.zip")
	var done := [false]
	app._write_project(path, func(): done[0] = true)
	for _i in 200:
		if done[0]:
			break
		await _frames(2)
	_check("the project was written", FileAccess.file_exists(path), path)
	var opened: bool = app._load_project(path)
	await _frames(10)
	_check("the project opened", opened)

	# 3. the planner's list holds the journey again
	var jp: Node = app.journey_planner_view
	var list: Array = jp._journeys
	_check("the planner lists one journey again", list.size() == 1, "%d entries" % list.size())
	if list.size() == 1:
		var e: Dictionary = list[0]
		var eid := int(e.get("engine_id", -9))
		var names_ok := String(e.get("name", "")) == "Probe journey"
		_check("same name and a live engine id", names_ok and eid >= 0 and not (bridge.journey_get(eid) as Dictionary).is_empty(),
			"%s / id %d" % [e.get("name"), eid])
		_check("the same route index", int(e.get("route", -9)) == ridx, "%d vs %d" % [int(e.get("route", -9)), ridx])
		_check("the preset's non-default field came back", int((e.get("plan", {}) as Dictionary).get(key, -1)) == int(plan[key]),
			"%s = %s" % [key, (e.get("plan", {}) as Dictionary).get(key)])
		_check("it is marked restored (its tooltip says what was not saved)", bool(e.get("restored", false)))

	# 4. the reopened world answers exactly as the generated one did
	bridge.civ_set_day_of_year(day)
	var after: Dictionary = readouts.call()
	# `civ_trade_flows` reports its own wall-clock `elapsed_ms`, which is a
	# measurement of this run and not a property of the world.
	for d in [before, after]:
		(d["civ_trade_flows"] as Dictionary).erase("elapsed_ms")
	for k in before.keys():
		var a := JSON.stringify(before[k])
		var b := JSON.stringify(after[k])
		_check("reopened project: %s is identical to the generated world's" % k, a == b,
			"%d vs %d chars; after starts %s" % [a.length(), b.length(), b.substr(0, 120)])
	# 5. a project saved before the substrate existed still opens, keeps its
	#    settlements, and refuses planning for the true reason
	var legacy := ProjectSettings.globalize_path("res://../crates/cartalith-godot/tests/fixtures/project_pre_substrate_2026-09-24.zip")
	_check("the pre-substrate fixture opens", app._load_project(legacy), legacy)
	await _frames(10)
	var old_towns: Array = bridge.settlements()
	_check("its civilisation layer is restored", old_towns.size() >= 2, "%d settlements" % old_towns.size())
	if old_towns.size() >= 2:
		var r: Dictionary = bridge.jp_compute({"points": PackedVector2Array([
			Vector2(float(old_towns[0]["x"]), float(old_towns[0]["y"])),
			Vector2(float(old_towns[1]["x"]), float(old_towns[1]["y"]))])})
		var err := String(r.get("error", ""))
		_check("it refuses to plan, naming the missing rasters", not bool(r.get("ok", true)) 			and err.contains("hydrology and tectonic rasters") and not err.contains("civilisation layer"), err)
		_check("and says what to do", err.contains("regenerate"), err)
		## A tree project, not a legacy import: `substrate::NEEDS_SUBSTRATE`,
		## which also says what the regenerate costs.
		_check("with the project wording, not the legacy-import one",
			not err.contains("legacy .zip") and err.contains("does not keep this project's labels and icons"), err)

	DirAccess.remove_absolute(path)
	print("JPRESTORE %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
