extends Node
## SP-2's journey timeline cache (`story_bridge.rs` `JourneyPlanCache`): the
## cached `journey_positions()` must be BYTE-IDENTICAL to a fresh plan at the
## same moment, in every scenario -- stays cached (day scrub, an unrelated
## journey added) and invalidates (preset edited, a way drawn along the route,
## regenerate). "Fresh" is `journey_timeline_cache_clear()` then the same call;
## identity is `var_to_bytes` equality of the whole returned Array.
## `journey_timeline_cache_stats()` counts which journeys were served cached.
##
## WINDOWED (the shell needs a real viewport to generate through):
##   Godot_v4.7.1-stable_win64_console.exe --path . _sp2cache_probe.tscn

var app: Node
var wg
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SP2C %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _stats() -> Dictionary:
	return wg.journey_timeline_cache_stats()

## One read through the cache, then a fresh one; returns
## [cached, hits delta, misses delta] and checks the two are byte-identical.
func _read(label: String) -> Array:
	var s0 := _stats()
	var cached: Array = wg.journey_positions()
	var s1 := _stats()
	var dh := int(s1["hits"]) - int(s0["hits"])
	var dm := int(s1["misses"]) - int(s0["misses"])
	wg.journey_timeline_cache_clear()
	var fresh: Array = wg.journey_positions()
	_check("%s: cached == fresh, byte for byte" % label, var_to_bytes(cached) == var_to_bytes(fresh),
		"hits+%d misses+%d, %d journeys" % [dh, dm, cached.size()])
	return [cached, dh, dm]

func _by_id(arr: Array, id: int) -> Dictionary:
	for d in arr:
		if int(d.get("id", -1)) == id:
			return d
	return {}

func _regen(seed_override: int) -> void:
	var req: Dictionary = app.new_world_dialog.request()
	if seed_override >= 0:
		req["seed"] = seed_override
	app.bridge.generate(req)
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("SP2C dll mtime: %s" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))
	await _regen(-1)
	if not app.bridge.has_world:
		print("SP2C  !! generate failed")
		get_tree().quit(1)
		return
	var bridge = app.bridge
	wg = bridge.world_gen
	var gs: Vector2 = bridge.grid_size()
	print("SP2C world %dx%d, seed %d" % [gs.x, gs.y, wg.get_seed()])

	# -- J1 on a CUSTOM preset (so it can be edited), on a real route ---------
	var stock: Array = wg.tl_list("preset")
	var stock_id := String((stock[0] as Dictionary).get("id", ""))
	var custom_id := String(wg.tl_duplicate("preset", stock_id).get("id", ""))
	var year0 := 412
	wg.civ_goto_year(year0)
	wg.civ_set_day_of_year(0)
	var towns: Array = bridge.settlements()
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var j1 := -1
	var ridx := -1
	for k in mini(8, pairs.size()):
		var a: Dictionary = towns[pairs[k][1]]
		var b: Dictionary = towns[pairs[k][2]]
		bridge.route_begin("mixed")
		bridge.route_append_stop(float(a["x"]), float(a["y"]))
		bridge.route_append_stop(float(b["x"]), float(b["y"]))
		ridx = bridge.route_commit()
		j1 = bridge.journey_save("Cache probe", custom_id, ridx, year0)
		var p := _by_id(wg.journey_positions(), j1)
		if p.has("x") and float(p["total_days"]) >= 8.0:
			break
		bridge.journey_delete(j1)
		j1 = -1
	_check("J1 saved on a custom preset, plans to 8+ days", j1 >= 0 and custom_id != "", "id=%d preset=%s" % [j1, custom_id])
	if j1 < 0:
		get_tree().quit(1)
		return
	var pts: PackedVector2Array = bridge.journey_get(j1)["points"]

	# -- warm: a repeat read is a hit ------------------------------------------
	var r := _read("repeat read")
	_check("repeat read: served cached", r[1] == 1 and r[2] == 0, "hits+%d misses+%d" % [r[1], r[2]])
	var base := _by_id(r[0], j1)
	var total := float(base["total_days"])

	# -- the DAY cursor: never an input ----------------------------------------
	var day_hits := 0
	var day_miss := 0
	var last_x := -1.0
	var moved := 0
	for d in [1, int(total * 0.25), int(total * 0.5), int(total * 0.75), 200, 364]:
		wg.civ_set_day_of_year(d)
		r = _read("day %d" % d)
		day_hits += r[1]
		day_miss += r[2]
		var p := _by_id(r[0], j1)
		if float(p.get("x", -1.0)) != last_x:
			moved += 1
		last_x = float(p.get("x", -1.0))
	_check("6 day moves: every read cached, none re-planned", day_hits == 6 and day_miss == 0, "hits %d misses %d" % [day_hits, day_miss])
	_check("and the marker still moved (positions are live, not cached)", moved >= 4, "%d distinct x of 6" % moved)
	## The real UI path too: the strip's day control fires timeline_changed ->
	## refresh_journey_markers -> journey_positions.
	var s0 := _stats()
	app.tl_set_day(30)
	app.tl_set_day(60)
	await _frames(2)
	var s1 := _stats()
	_check("UI day scrub (tl_set_day x2): no re-plan", int(s1["misses"]) == int(s0["misses"]),
		"hits+%d misses+%d" % [int(s1["hits"]) - int(s0["hits"]), int(s1["misses"]) - int(s0["misses"])])

	# -- the YEAR cursor: territory IS an input; report what happens ----------
	for y in [year0 - 1, year0 + 1, year0]:
		wg.civ_goto_year(y)
		r = _read("year %d" % y)
		print("SP2C year %d: hits+%d misses+%d (a miss means civ_goto_year changed territory)" % [y, r[1], r[2]])

	# -- an unrelated journey added: J1 stays cached, J2 is planned -----------
	var j2: int = bridge.journey_save("Unrelated", stock_id, ridx, year0 + 3)
	r = _read("J2 added")
	_check("J2 added: J1 hit, J2 miss", r[1] == 1 and r[2] == 1, "hits+%d misses+%d" % [r[1], r[2]])

	# -- the preset edited: J1 re-plans and reflects it; J2 untouched ----------
	var hours0 := float(wg.tl_get("preset", custom_id).get("hours", 0.0))
	var er: Dictionary = wg.tl_edit("preset", custom_id, {"hours": maxf(1.0, hours0 * 0.5)})
	r = _read("preset hours %.1f -> %.1f" % [hours0, maxf(1.0, hours0 * 0.5)])
	var after_preset := _by_id(r[0], j1)
	_check("preset edited: J1 miss, J2 hit", r[1] == 1 and r[2] == 1 and bool(er.get("ok", false)), "hits+%d misses+%d edit=%s" % [r[1], r[2], er])
	_check("and J1's plan reflects it (total_days moved)", float(after_preset["total_days"]) != total,
		"%.3f -> %.3f days" % [total, float(after_preset["total_days"])])

	# -- a hand-drawn way along the route: every journey on it re-plans --------
	var t_before := float(after_preset["total_days"])
	wg.way_begin("road")
	var step := maxi(1, pts.size() / 12)
	for i in range(0, pts.size(), step):
		wg.way_append_point(pts[i].x, pts[i].y)
	wg.way_append_point(pts[pts.size() - 1].x, pts[pts.size() - 1].y)
	var widx: int = wg.way_commit()
	r = _read("way %d drawn along the route" % widx)
	_check("way drawn: both journeys miss", widx >= 0 and r[1] == 0 and r[2] == 2, "hits+%d misses+%d" % [r[1], r[2]])
	print("SP2C way: J1 total_days %.3f -> %.3f" % [t_before, float(_by_id(r[0], j1).get("total_days", -1))])

	# -- J2 deleted: its entry is pruned; J1 still cached ----------------------
	bridge.journey_delete(j2)
	r = _read("J2 deleted")
	_check("J2 deleted: J1 hit, entry pruned", r[1] == 1 and r[2] == 0 and int(_stats()["entries"]) == 1 and _by_id(r[0], j2).is_empty(),
		"hits+%d misses+%d entries=%d" % [r[1], r[2], int(_stats()["entries"])])

	# -- territory (a key input the year cursor writes) -------------------------
	## Record the claim map as it is (year0), paint a claim over the whole
	## route and commit it, record that (year0+50), then move the year cursor
	## between the two. The paint must re-plan. The year moves are REPORTED,
	## not asserted as misses: whether `civ_goto_year` rewrites the live
	## territory here is its business, and whichever it does, cached == fresh
	## is the assertion that matters (a hit on unchanged bytes is correct).
	wg.civ_add_year(year0)
	var t_terr := float(_by_id(wg.journey_positions(), j1).get("total_days", -1))
	var lo := pts[0]
	var hi := pts[0]
	for q in pts:
		lo = Vector2(minf(lo.x, q.x), minf(lo.y, q.y))
		hi = Vector2(maxf(hi.x, q.x), maxf(hi.y, q.y))
	lo -= Vector2(4, 4)
	hi += Vector2(4, 4)
	var staged: int = wg.civ_territory_paint_polygon(PackedVector2Array([lo, Vector2(hi.x, lo.y), hi, Vector2(lo.x, hi.y)]), 1, false)
	wg.civ_territory_commit()
	r = _read("territory painted (%d cells staged)" % staged)
	_check("territory painted: J1 re-planned", staged > 0 and r[1] == 0 and r[2] == 1,
		"hits+%d misses+%d; total_days %.3f -> %.3f" % [r[1], r[2], t_terr, float(_by_id(r[0], j1).get("total_days", -1))])
	wg.civ_add_year(year0 + 50)
	for step_y in [[year0, "year cursor back to the unpainted year"], [year0 + 50, "year cursor onto the painted year"], [year0 + 50, "year cursor onto the same year again"]]:
		wg.civ_goto_year(step_y[0])
		r = _read(step_y[1])
		print("SP2C %s: hits+%d misses+%d, total_days %.3f" % [step_y[1], r[1], r[2], float(_by_id(r[0], j1).get("total_days", -1))])
	_check("the last move changed no territory: served cached", r[1] == 1 and r[2] == 0, "hits+%d misses+%d" % [r[1], r[2]])
	wg.civ_goto_year(year0)

	# -- timing: warm (hits) against cold (cleared before every call) ---------
	var warm: Array = []
	var cold: Array = []
	for i in 15:
		var t0 := Time.get_ticks_usec()
		wg.journey_positions()
		warm.append((Time.get_ticks_usec() - t0) / 1000.0)
	for i in 7:
		wg.journey_timeline_cache_clear()
		var t0 := Time.get_ticks_usec()
		wg.journey_positions()
		cold.append((Time.get_ticks_usec() - t0) / 1000.0)
	warm.sort()
	cold.sort()
	print("SP2C journey_positions() warm/cached: median %.2f ms (min %.2f .. max %.2f), n=%d" % [warm[warm.size() / 2], warm[0], warm[-1], warm.size()])
	print("SP2C journey_positions() cold/re-planned: median %.2f ms (min %.2f .. max %.2f), n=%d" % [cold[cold.size() / 2], cold[0], cold[-1], cold.size()])

	# -- regenerate, same seed then a new one ----------------------------------
	var seed0: int = wg.get_seed()
	await _regen(seed0)
	wg.civ_goto_year(year0)
	r = _read("regenerate, same seed %d" % seed0)
	print("SP2C same-seed regenerate: hits+%d misses+%d, J1 %s" % [r[1], r[2], "kept" if not _by_id(r[0], j1).is_empty() else "dropped"])
	await _regen(seed0 + 7919)
	wg.civ_goto_year(year0)
	r = _read("regenerate, new seed %d" % (seed0 + 7919))
	print("SP2C new-seed regenerate: hits+%d misses+%d, resnap %s" % [r[1], r[2], wg.journey_resnap_report()])
	_check("new seed: nothing served from the old world", r[1] == 0, "hits+%d misses+%d" % [r[1], r[2]])

	print("SP2C done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
