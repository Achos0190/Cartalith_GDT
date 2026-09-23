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
##      rebuilt from defaults would fail the last check.

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
	DirAccess.remove_absolute(path)
	print("JPRESTORE %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
