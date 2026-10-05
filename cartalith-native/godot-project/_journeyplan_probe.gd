extends Node
## Windowed proof of Ruling AR (`LARGE_ITEM_RULINGS.md`, "keep the full plan"):
## a saved journey keeps its WHOLE planner plan, not just its party preset, and
## the saved plan is what dates it after a reopen.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _journeyplan_probe.tscn
##
## Real generated world, real committed route, the planner view's own state and
## its own save path (`_commit_saved_journey`, the function the Save button's
## prompt calls), the app's own project write and open:
##   1. three journeys are saved on one route: A on the DEFAULT plan, B on a
##      non-default plan -- `rest_cadence` plus one field from every group the
##      form reaches (party size, traveller pace, season, foraging, supply days,
##      hours, a stage override, a layover, the trim, both auto flags) -- and C,
##      B's plan with ONLY the rest cadence put back, so B-versus-C isolates
##      what the cadence does to the dates;
##   2. the project is written and opened;
##   3. (a) the planner list restores every one of those values, and the
##      restored entry is NOT flagged `restored` (the preset-only fallback);
##   4. (b) the SP-2 positions -- arrival date and total days -- equal the
##      pre-save ones journey for journey, B's dates differ from C's (the
##      non-default cadence MOVES the dates) and from A's, and re-loading the
##      restored B into the planner computes the same total days again.
## Every PROBE-FAIL line is a failure; the script still exits 0 so the caller
## greps for it. Headless is refused: it cannot size or draw the shell.

var _fails := 0
var app: Node
var bridge: Node

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("JPLAN ok    %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
	else:
		print("JPLAN PROBE-FAIL  %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
		_fails += 1

## One entry of `journey_positions()` by engine id, `{}` when absent.
func _pos(id: int) -> Dictionary:
	for d in bridge.journey_positions():
		if int(d.get("id", -1)) == id:
			return d
	return {}

## The first option of `key` that is not `avoid`, or "" when none.
func _other_option(opts: Dictionary, key: String, avoid: String) -> String:
	for o in opts.get(key, PackedStringArray()):
		if String(o) != avoid:
			return String(o)
	return ""

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("JPLAN REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 270.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("JPLAN PROBE-FAIL watchdog"); get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	bridge = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	print("JPLAN dll mtime: %s" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://../target/debug/cartalith_godot.dll")))

	# -- a real route between two real settlements, a good distance apart ------
	var gs: Vector2i = bridge.grid_size()
	var towns: Array = bridge.settlements()
	var best := [-1, -1, INF]
	for i in mini(60, towns.size()):
		for j in range(i + 1, mini(60, towns.size())):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if absf(d - gs.x * 0.12) < float(best[2]):
				best = [i, j, absf(d - gs.x * 0.12)]
	_check("setup: two settlements to route between", int(best[0]) >= 0)
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(towns[best[0]]["x"]), float(towns[best[0]]["y"]))
	bridge.route_append_stop(float(towns[best[1]]["x"]), float(towns[best[1]]["y"]))
	var ridx: int = bridge.route_commit()
	_check("setup: the route committed", ridx >= 0, "route %d" % ridx)
	app.tl_set_year(412)
	app.tl_set_day(0)

	app.open_journey_planner_with_route(ridx)
	await _frames(14)
	var jpv = app.journey_planner_view
	var opts: Dictionary = jpv._options

	# == 1. journey A: the default plan ========================================
	jpv._compute()
	var res_a: Dictionary = jpv._last_result
	_check("default plan computes", bool(res_a.get("ok", false)), String(res_a.get("error", "")))
	var days_a := float((res_a.get("plan", {}) as Dictionary).get("total_days", -1.0))
	_check("the default party can make this route (a blocked A would make every comparison vacuous)", days_a > 0.0, "%.3f days" % days_a)
	jpv._commit_saved_journey("Default road")
	var id_a := int(jpv._journeys[jpv._journeys.size() - 1]["engine_id"])
	_check("journey A saved with an engine id", id_a >= 0, str(id_a))
	_check("journey A stored a plan (the default one is a real plan)",
		(bridge.journey_get(id_a) as Dictionary).has("plan"))

	# == 1b. journey B: rest cadence + one field per reachable group ===========
	var heavy := ""
	for o in opts.get("rest_cadence", PackedStringArray()):
		if String(o).begins_with("Heavy"):
			heavy = String(o)
	_check("the rest-cadence option list offers Heavy", heavy != "", str(opts.get("rest_cadence")))
	var def_plan: Dictionary = jpv._plan_values.duplicate(true)
	var edits := {
		"rest_cadence": heavy,
		"hours": 6.0,
		"group_size": int(def_plan.get("group_size", 10)) + 3,
		"supply_days": int(def_plan.get("supply_days", 7)) + 9,
		"pace": _other_option(opts, "pace", String(def_plan.get("pace", ""))),
		"season": _other_option(opts, "season", String(def_plan.get("season", ""))),
		"foraging": _other_option(opts, "foraging", String(def_plan.get("foraging", ""))),
	}
	for k in edits.keys():
		jpv._plan_values[k] = edits[k]
		_check("edit %s is a real change from the default" % k, edits[k] != def_plan.get(k), "%s -> %s" % [def_plan.get(k), edits[k]])
	jpv._carriage_auto = false   ## default true
	jpv._stage_auto = true       ## default false
	jpv._trim = Vector2(0.1, 0.9)
	jpv._compute()
	var stops: Array = (jpv._last_result.get("plan", {}) as Dictionary).get("stops", [])
	var layover_key := String((stops[0] as Dictionary).get("key", "")) if not stops.is_empty() else ""
	if layover_key != "":
		jpv._layovers[layover_key] = 2
	jpv._set_stage_override(0, "hours", 5.5)   ## also recomputes
	var res_b: Dictionary = jpv._last_result
	_check("non-default plan computes", bool(res_b.get("ok", false)), String(res_b.get("error", "")))
	var days_b := float((res_b.get("plan", {}) as Dictionary).get("total_days", -1.0))
	_check("the non-default plan plans a different length of trip than the default", not is_equal_approx(days_a, days_b),
		"%.3f vs %.3f days" % [days_a, days_b])
	jpv._commit_saved_journey("Heavy-rest road")
	var id_b := int(jpv._journeys[jpv._journeys.size() - 1]["engine_id"])
	_check("journey B saved with an engine id", id_b >= 0, str(id_b))

	# == 1c. journey C: B's plan with ONLY the cadence put back ================
	## If the cadence did not move the planner's dates, B and C would be equal
	## and every "cadence moves the dates" claim below would be vacuous.
	var keep_cadence: String = jpv._plan_values["rest_cadence"]
	jpv._plan_values["rest_cadence"] = def_plan.get("rest_cadence")
	jpv._compute()
	var days_c := float((jpv._last_result.get("plan", {}) as Dictionary).get("total_days", -1.0))
	_check("cadence alone moves the planner's days", days_c > 0.0 and not is_equal_approx(days_b, days_c), "B %.3f vs C %.3f" % [days_b, days_c])
	jpv._commit_saved_journey("Default-cadence road")
	var id_c := int(jpv._journeys[jpv._journeys.size() - 1]["engine_id"])
	jpv._plan_values["rest_cadence"] = keep_cadence
	jpv._compute()

	# -- the dates the engine reports for the three saved journeys, pre-save ---
	bridge.civ_set_day_of_year(3)
	var pa0 := _pos(id_a)
	var pb0 := _pos(id_b)
	var pc0 := _pos(id_c)
	_check("A, B and C all have SP-2 positions", pa0.has("arrival") and pb0.has("arrival") and pc0.has("arrival"),
		"%s | %s | %s" % [pa0.get("error", ""), pb0.get("error", ""), pc0.get("error", "")])
	_check("pre-save: the cadence alone moves the engine's total days (B vs C)",
		pb0.has("total_days") and pc0.has("total_days") and not is_equal_approx(float(pb0["total_days"]), float(pc0["total_days"])),
		"B %s  C %s" % [pb0.get("total_days"), pc0.get("total_days")])
	_check("pre-save: B's saved plan dates it differently from A's default",
		pa0.has("total_days") and pb0.has("total_days") and not is_equal_approx(float(pa0["total_days"]), float(pb0["total_days"])),
		"A %s  B %s" % [pa0.get("total_days"), pb0.get("total_days")])
	_check("pre-save: B's engine total days equal the planner's own for the same plan",
		is_equal_approx(float(pb0.get("total_days", -1)), days_b), "%s vs %.4f" % [pb0.get("total_days"), days_b])
	_check("pre-save: A's engine total days equal the planner's own", is_equal_approx(float(pa0.get("total_days", -1)), days_a),
		"%s vs %.4f" % [pa0.get("total_days"), days_a])

	# == 2. write and open the project ==========================================
	var path := OS.get_user_data_dir().path_join("_journeyplan_probe.zip")
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

	# == 3. (a) the planner list restores the values ============================
	jpv = app.journey_planner_view
	var list: Array = jpv._journeys
	_check("the planner lists all three journeys again", list.size() == 3, "%d entries" % list.size())
	var ea: Dictionary = {}
	var eb: Dictionary = {}
	var ec: Dictionary = {}
	for e in list:
		match String(e.get("name", "")):
			"Default road":
				ea = e
			"Heavy-rest road":
				eb = e
			"Default-cadence road":
				ec = e
	_check("all three entries found by name", not ea.is_empty() and not eb.is_empty() and not ec.is_empty())
	if not eb.is_empty():
		var bp: Dictionary = eb.get("plan", {})
		_check("B is not flagged restored (it came back from a stored plan, not a preset)", not bool(eb.get("restored", false)))
		for k in edits.keys():
			_check("B restored plan.%s" % k, bp.get(k) == edits[k] or (typeof(edits[k]) == TYPE_FLOAT and is_equal_approx(float(bp.get(k, -1)), float(edits[k]))),
				"%s vs %s" % [bp.get(k), edits[k]])
		_check("B restored the trim", (eb.get("trim", Vector2()) as Vector2).is_equal_approx(Vector2(0.1, 0.9)), str(eb.get("trim")))
		_check("B restored carriage_auto=false and stage_auto=true", not bool(eb.get("carriage_auto", true)) and bool(eb.get("stage_auto", false)))
		_check("B restored the stage override", is_equal_approx(float(((eb.get("stage_overrides", {}) as Dictionary).get(0, {}) as Dictionary).get("hours", -1.0)), 5.5),
			str(eb.get("stage_overrides")))
		if layover_key != "":
			_check("B restored the layover", int((eb.get("layovers", {}) as Dictionary).get(layover_key, -1)) == 2, str(eb.get("layovers")))
	if not ea.is_empty():
		_check("A restored with the default cadence", String((ea.get("plan", {}) as Dictionary).get("rest_cadence", "")) == String(def_plan.get("rest_cadence", "")),
			String((ea.get("plan", {}) as Dictionary).get("rest_cadence", "")))
		_check("A restored carriage_auto=true, stage_auto=false", bool(ea.get("carriage_auto", false)) and not bool(ea.get("stage_auto", true)))
		_check("A restored an untrimmed route", (ea.get("trim", Vector2()) as Vector2).is_equal_approx(Vector2(0.0, 1.0)))

	# == 4. (b) the dates are the same, and differ between the two ==============
	bridge.civ_set_day_of_year(3)
	var pa1 := _pos(int(ea.get("engine_id", -1)))
	var pb1 := _pos(int(eb.get("engine_id", -1)))
	var pc1 := _pos(int(ec.get("engine_id", -1)))
	_check("after reopen: A's position is identical to before the save", JSON.stringify(pa0) == JSON.stringify(pa1),
		"before %s\nafter  %s" % [JSON.stringify(pa0), JSON.stringify(pa1)])
	_check("after reopen: B's position (arrival date, total days) is identical to before the save",
		JSON.stringify(pb0) == JSON.stringify(pb1), "before %s\nafter  %s" % [JSON.stringify(pb0), JSON.stringify(pb1)])
	_check("after reopen: C's position is identical to before the save", JSON.stringify(pc0) == JSON.stringify(pc1),
		"before %s\nafter  %s" % [JSON.stringify(pc0), JSON.stringify(pc1)])
	_check("after reopen: the non-default cadence still dates B differently from C (days and arrival)",
		pb1.has("total_days") and pc1.has("total_days") and not is_equal_approx(float(pb1["total_days"]), float(pc1["total_days"]))
		and str(pb1.get("arrival")) != str(pc1.get("arrival")),
		"B %s/%s  C %s/%s" % [pb1.get("arrival"), pb1.get("total_days"), pc1.get("arrival"), pc1.get("total_days")])
	# Loading the restored entry back into the planner reproduces the days.
	if not eb.is_empty():
		jpv._load_journey(list.find(eb))
		await _frames(6)
		var days_b2 := float((jpv._last_result.get("plan", {}) as Dictionary).get("total_days", -1.0))
		_check("loading the restored B into the planner computes the pre-save days", is_equal_approx(days_b2, days_b), "%.4f vs %.4f" % [days_b2, days_b])

	DirAccess.remove_absolute(path)
	print("JPLAN RESULT %s  (%d failures)" % ["GREEN" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
