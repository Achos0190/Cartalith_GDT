extends Node
## SP-2 (`STORY_PLANNING_SCOPE.md` §4): "scrubbing the year cursor moves the
## party marker along its route and the supply readout falls as it goes, with
## the arrival date derived rather than typed." Plus Ruling AO's regenerate
## re-snap. A real generated world, a real committed route between two real
## settlements, a real saved journey on a stock party preset -- then the real
## timeline controls, read back through the engine AND the framebuffer.
##
## WINDOWED, not headless: `ImageTexture.update()` is a no-op under
## `--headless` (MISTAKES.md), and part of this is a pixel check.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _sp2journey_probe.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SP2 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _pos(id: int) -> Dictionary:
	for d in app.bridge.journey_positions():
		if int(d.get("id", -1)) == id:
			return d
	return {}

func _frame() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

## Pixels differing between two frames inside a box around `centre`.
func _diff_near(a: Image, b: Image, centre: Vector2, r: int) -> int:
	var n := 0
	for y in range(maxi(0, int(centre.y) - r), mini(a.get_height(), int(centre.y) + r)):
		for x in range(maxi(0, int(centre.x) - r), mini(a.get_width(), int(centre.x) + r)):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n

func _diff_all(a: Image, b: Image) -> int:
	var n := 0
	for y in range(0, a.get_height(), 2):
		for x in range(0, a.get_width(), 2):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n

## Grid cell -> window pixel, through the overlay's own mapping.
func _screen(gx: float, gy: float) -> Vector2:
	var ov = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = ov._point_to_screen(Vector2(gx, gy), rect)
	return ov.get_global_transform_with_canvas() * local

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("SP2 dll mtime: %s" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		print("SP2  !! generate failed")
		get_tree().quit(1)
		return
	var bridge = app.bridge
	var gs: Vector2 = bridge.grid_size()
	print("SP2 world %dx%d generated (%d frames)" % [gs.x, gs.y, waited])

	# -- a real route between two real settlements, and a real saved journey --
	var towns: Array = bridge.settlements()
	var presets: Array = bridge.world_gen.tl_list("preset")
	var preset_id := String((presets[0] as Dictionary).get("id", "")) if not presets.is_empty() else ""
	var year0 := 412
	app.tl_set_year(year0)
	app.tl_set_day(0)
	## Settlement pairs, farthest first within half the map, until one plans
	## (not blocked) to a trip of at least 8 days -- long enough that three
	## distinct scrub days land mid-route.
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var jid := -1
	var ridx := -1
	var a: Dictionary = {}
	var b: Dictionary = {}
	for k in mini(8, pairs.size()):
		a = towns[pairs[k][1]]; b = towns[pairs[k][2]]
		bridge.route_begin("mixed")
		bridge.route_append_stop(float(a["x"]), float(a["y"]))
		bridge.route_append_stop(float(b["x"]), float(b["y"]))
		ridx = bridge.route_commit()
		jid = bridge.journey_save("Probe road", preset_id, ridx, year0)
		var try := _pos(jid)
		print("SP2 candidate %s -> %s: %s" % [a.get("name"), b.get("name"), try.get("total_days", try.get("error", "?"))])
		if try.has("x") and float(try["total_days"]) >= 8.0:
			break
		bridge.journey_delete(jid)
		jid = -1
	app.viewport.refresh_journey_markers()
	_check("a journey of 8+ days saved on a real route and a stock preset", jid >= 0 and preset_id != "",
		"route=%d preset=%s id=%d  %s -> %s" % [ridx, preset_id, jid, a.get("name"), b.get("name")])
	if jid < 0:
		get_tree().quit(1)
		return

	var p0 := _pos(jid)
	print("SP2 at %s: %s" % [app.tl_date_text(), p0])
	_check("it has a position (not blocked)", p0.has("x") and p0.has("arrival"), str(p0.get("error", "")))
	if not p0.has("x"):
		get_tree().quit(1)
		return
	var total := float(p0["total_days"])
	var jg: Dictionary = bridge.journey_get(jid)
	var pts: PackedVector2Array = jg["points"]
	_check("day 0: en route at the route's first point", String(p0["phase"]) == "en_route"
		and Vector2(p0["x"], p0["y"]).distance_to(pts[0]) < 0.01, "%s vs %s" % [Vector2(p0["x"], p0["y"]), pts[0]])
	_check("arrival is derived: departure + ceil(total_days)",
		String(p0["arrival"]) == "%d-%s" % [year0, _md(ceili(total))] or ceili(total) >= 365,
		"arrival=%s total=%.2f" % [p0["arrival"], total])

	# -- scrub the DAY through the real strip slider --------------------------
	app.select_domain("civilization")
	await _frames(3)
	app._tl_expanded = true
	app._fill_timeline_strip()
	await _frames(3)
	_check("the expanded timeline has the day slider", is_instance_valid(app._tl_day_slider))
	app.viewport.move_view_to((pts[0].x + pts[pts.size() - 1].x) * 0.5, (pts[0].y + pts[pts.size() - 1].y) * 0.5)
	await _frames(6)
	var img0 := await _frame()
	var ctrl := await _frame()
	_check("control: an unchanged cursor redraws identically", _diff_all(img0, ctrl) == 0,
		"%d px differ" % _diff_all(img0, ctrl))

	var prev := p0
	var img_mid: Image = null
	var p_mid: Dictionary = {}
	var mid_day := clampi(int(total * 0.5), 1, 363)
	var moved := 0
	var times: Array = []
	for d in [int(total * 0.25), mid_day, int(total * 0.75)]:
		app._tl_day_slider.value = d   ## the real control, emitting value_changed
		await _frames(2)
		var t0 := Time.get_ticks_usec()
		var p := _pos(jid)
		times.append((Time.get_ticks_usec() - t0) / 1000.0)
		print("SP2 %s  phase=%s  (%.1f, %.1f)  km %.1f/%.1f  food %.1f/%.1f kg" % [app.tl_date_text(), p.get("phase"),
			p.get("x", -1), p.get("y", -1), p.get("km_done", -1), p.get("km", -1), p.get("food_kg_used", -1), p.get("food_kg", -1)])
		_check("day %d: the date label shows the cursor" % d, app._tl_date_label.text == app.tl_date_text(), app._tl_date_label.text)
		if p.has("x") and Vector2(p["x"], p["y"]).distance_to(Vector2(prev["x"], prev["y"])) > 0.01:
			moved += 1
		_check("day %d: distance and supply only grow" % d, float(p.get("km_done", -1)) >= float(prev["km_done"])
			and float(p.get("food_kg_used", -1)) >= float(prev["food_kg_used"]))
		prev = p
		if d == mid_day:
			await _frames(2)
			img_mid = await _frame()
			p_mid = p
	_check("the marker moved on every scrub step", moved == 3, "%d of 3" % moved)
	## The drawn readout: the map's own subline under the marker, from the
	## same dictionaries the overlay was handed.
	var ov = app.viewport.overlay
	var s0 := String(ov._journey_marker_subline(p0))
	var s1 := String(ov._journey_marker_subline(prev))
	print("SP2 readout day 0: '%s'  |  last: '%s'" % [s0, s1])
	_check("the map readout names the food left and the derived arrival",
		s0.begins_with("food %d of" % roundi(float(p0["food_kg"]))) and s0.ends_with(String(p0["arrival"])), s0)
	_check("and it falls as the cursor moves", s0 != s1 and s1.begins_with("food %d of" % roundi(float(prev["food_kg"]) - float(prev["food_kg_used"]))), s1)
	_check("the overlay holds the markers it was handed", (ov._journey_markers as Array).size() == 1)
	var remaining0 := float(p0["food_kg"]) - float(p0["food_kg_used"])
	var remaining1 := float(prev["food_kg"]) - float(prev["food_kg_used"])
	_check("the supply remaining FELL", remaining1 < remaining0, "%.1f -> %.1f kg" % [remaining0, remaining1])

	## Compared at the MID-route day, which the camera (centred on the route)
	## is sure to show; the day-20 spot can sit under the top chrome.
	var at0 := _screen(float(p0["x"]), float(p0["y"]))
	var at1 := _screen(float(p_mid["x"]), float(p_mid["y"]))
	_check("pixels: the marker left its day-0 spot", _diff_near(img0, img_mid, at0, 10) > 20, "%d px near %s" % [_diff_near(img0, img_mid, at0, 10), at0])
	_check("pixels: and appeared at its mid-route spot", _diff_near(img0, img_mid, at1, 10) > 20, "%d px near %s" % [_diff_near(img0, img_mid, at1, 10), at1])
	var out_dir := OS.get_environment("SP2_SHOTS")
	if out_dir != "":
		img0.save_png(out_dir.path_join("sp2_day0.png"))
		img_mid.save_png(out_dir.path_join("sp2_mid.png"))

	# -- the YEAR cursor: before departure, and after arrival ------------------
	app.tl_set_year(year0 - 1)
	await _frames(2)
	var pb := _pos(jid)
	_check("a year earlier: not departed, at the first point", String(pb.get("phase")) == "not_departed"
		and Vector2(pb["x"], pb["y"]).distance_to(pts[0]) < 0.01, str(pb.get("phase")))
	app.tl_set_year(year0 + 1 + int(total / 365.0))
	await _frames(2)
	var pa := _pos(jid)
	_check("after the arrival date: arrived, at the last point, all supply used",
		String(pa.get("phase")) == "arrived" and Vector2(pa["x"], pa["y"]).distance_to(pts[pts.size() - 1]) < 0.01
		and absf(float(pa["food_kg_used"]) - float(pa["food_kg"])) < 1e-6, str(pa.get("phase")))
	times.sort()
	print("SP2 journey_positions(): median %.1f ms (min %.1f .. max %.1f), n=%d" % [times[times.size() / 2], times[0], times[-1], times.size()])

	# -- regenerate: the journey is re-snapped, not dropped -------------------
	app._run_pipeline()
	waited = 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	var report: Array = bridge.journey_resnap_report()
	print("SP2 re-snap report: %s" % [report])
	var kept: Dictionary = bridge.journey_get(jid)
	_check("regenerate (same seed): the journey survives with its id", not kept.is_empty(), str(report))
	_check("and the report says resnapped", report.size() == 1 and String(report[0].get("outcome")) == "resnapped", str(report))
	app.tl_set_year(year0)
	app.tl_set_day(mid_day)
	var pr := _pos(jid)
	_check("and its party is still placed on the new world", pr.has("x"), str(pr.get("error", "")))

	print("SP2 done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

func _md(doy: int) -> String:
	var lens := [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
	var m := 0
	while doy >= lens[m]:
		doy -= lens[m]
		m += 1
	return "%02d-%02d" % [m + 1, doy + 1]
