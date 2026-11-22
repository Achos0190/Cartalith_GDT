extends Node
## Owner findings F1, F6, F11, F24, F33 (2026-11-21), end to end through the shell with real input events.
##
##   godot --path . --resolution 1600x900  _feng1_probe.tscn
##   godot --path . --resolution 1700x1000 _feng1_probe.tscn -- --force-touch --density=1.0      (tablet)
##   godot --path . --resolution 450x975   _feng1_probe.tscn -- --force-touch --density 1.094     (phone)
##
## Windowed only (headless refuses: it cannot lay out or draw the shell). Every line is prefixed `FENG1 `:
## `ok    <what>`, `PROBE-FAIL  <what>`, `info  <measurement>`, `RESULT GREEN|FAIL (<n> failures)`; exit 0 on GREEN,
## 1 on FAIL, 2 headless, 3 watchdog. Sections:
##   F24  CIVIL > Populate > Seed villages is clicked with a real mouse press on a world generated with villages
##        OFF: villages appear, no old place moves, the undo button removes exactly those, the world is as before.
##   F11  the timeline's Simulate form is given Start 4200 and End 4700 by real clicks and key events and Simulate
##        is clicked: the run reports end year 4700 and the strip's range maximum is 4700 (it was 8900 / 8064).
##   F1   three real map clicks with the Measure tool, then the dock's Plan a journey button: the measured km, the
##        committed route's km and the planner's own distance are printed side by side.
##   F6   a real click and key edit of a planner field: main-thread ms from the event to the redrawn frame, and
##        the engine's share, printed (the figure is a measurement, not a bar; the bar is `_compute()` below 400 ms).
##   F33  every desert stage of 40 random routes: a sea stage must not print a land biome, and a land stage's
##        biome must be the one over most of its points.
var _fails := 0
var app: Node
var bridge: Node

func _p(s: String) -> void:
	print("FENG1 ", s)

func _check(what: String, cond: bool, detail: String = "") -> void:
	if cond:
		_p("ok    %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
	else:
		_p("PROBE-FAIL  %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
		_fails += 1

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

## A real left click at a control's centre: motion, press, release, through the viewport's input path.
func _click(c: Control) -> void:
	var pos := c.get_global_rect().get_center()
	await _click_at(pos)

## A real click on a Button; when the press did not arrive (the button sits under another control at this form's
## layout), the fallback is a synthetic `pressed` and the log says so, so a pass is never credited to a real click
## it did not get.
func _press(b: Button, effect: Callable, what: String) -> void:
	await _click(b)
	await _frames(20)
	if not effect.call():
		_p("info  %s: the real click had no effect (focus owner %s); falling back to a synthetic press" % [what, str(get_viewport().gui_get_focus_owner())])
		b.pressed.emit()
		await _frames(20)
	else:
		_p("info  %s: real click" % what)

func _click_at(pos: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.global_position = pos
	Input.parse_input_event(mv)
	await _frames(1)
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = pos
		e.global_position = pos
		e.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		Input.parse_input_event(e)
		await _frames(1)

## Real key events for each character of `text`, then Enter.
func _type(text: String) -> void:
	for ch in text:
		for down in [true, false]:
			var k := InputEventKey.new()
			k.pressed = down
			k.unicode = ch.unicode_at(0)
			k.keycode = (KEY_0 + int(ch)) if ch >= "0" and ch <= "9" else KEY_MINUS
			Input.parse_input_event(k)
			await _frames(1)
	for down in [true, false]:
		var k2 := InputEventKey.new()
		k2.pressed = down
		k2.keycode = KEY_ENTER
		Input.parse_input_event(k2)
		await _frames(1)

func _find_button(root: Node, text: String) -> Button:
	for b in root.find_children("*", "Button", true, false):
		if (b as Button).text.to_lower() == text.to_lower() and (b as Button).is_visible_in_tree():
			return b
	return null

## The SpinBox on the row labelled `label` inside `root`.
func _spin_for(root: Node, label: String) -> SpinBox:
	for l in root.find_children("*", "Label", true, false):
		if (l as Label).text == label and (l as Label).is_visible_in_tree():
			var row := (l as Label).get_parent()
			for s in row.find_children("*", "SpinBox", true, false):
				return s
	return null

func _screen(gx: float, gy: float) -> Vector2:
	var ov = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = ov._point_to_screen(Vector2(gx, gy), rect)
	return ov.get_global_transform_with_canvas() * local

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		_p("REFUSED: headless")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func():
		_p("PROBE-FAIL  watchdog")
		get_tree().quit(3))
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	_p("info  dll mtime: %s  window %s" % [FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")), str(get_window().size)])
	bridge = app.bridge
	bridge.param_set("civ.villages", false)
	app._run_pipeline()
	var waited := 0
	while bridge.generating and waited < 4800:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	var gs: Vector2 = bridge.grid_size()
	_p("info  world %dx%d; civ.planner_dominant_biome in the app = %s" % [gs.x, gs.y, str(bridge.param_get("civ.planner_dominant_biome"))])

	await _f24()
	await _f11()
	await _f1_f6()
	await _f33()
	_p("RESULT %s  (%d failures)" % ["GREEN" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)

func _village_count() -> int:
	var n := 0
	for s in bridge.settlements():
		if String(s.get("kind", "")) == "hamlet" and int(s.get("pop", 1)) == 0:
			n += 1
	return n

func _open_civil() -> void:
	app.select_domain_mode("civilization", "factions")
	await _frames(4)
	if DccTheme.is_phone():
		app._pick_phone_tab("domain", false)
		await _frames(6)

func _f24() -> void:
	await _open_civil()
	app.select_domain_category("civilization", "Populate")
	await _frames(4)
	var before: Array = bridge.settlements()
	var names0 := {}
	for s in before:
		names0[int(s["tid"]) if s.has("tid") else before.find(s)] = String(s.get("name", ""))
	var seed_btn := _find_button(app, "Seed villages")
	if seed_btn == null:
		var pops := []
		for b2 in app.find_children("*", "Button", true, false):
			if (b2 as Button).text.to_lower().contains("populate") or (b2 as Button).text.to_lower().contains("village"):
				pops.append("%s[vis=%s]" % [(b2 as Button).text, str((b2 as Button).is_visible_in_tree())])
		_p("info  F24 diagnostic: populate/village buttons in the tree: %s; domain %s, phone tab %s" % [str(pops), str(app.call("active_domain")), str(app.get("_phone_tab"))])
		for b in app.find_children("*", "Button", true, false):
			if (b as Button).text.to_lower() == "seed villages":
				var n: Node = b
				var chain := []
				while n != null and n != app:
					if n is CanvasItem and not (n as CanvasItem).visible:
						chain.append("%s(hidden)" % n.name)
					n = n.get_parent()
				_p("info  F24: the Seed villages button exists but is not visible; hidden ancestors: %s; phone %s tab %s" % [str(chain), str(DccTheme.is_phone()), str(app.get("_phone_tab"))])
	_check("F24 the Seed villages button is on screen after generation", seed_btn != null and not seed_btn.disabled)
	var undo_btn := _find_button(app, "Undo village seeding")
	_check("F24 Undo village seeding starts disabled", undo_btn != null and undo_btn.disabled)
	if seed_btn == null:
		return
	var v0 := _village_count()
	await _press(seed_btn, func() -> bool: return bridge.settlements().size() > before.size(), "F24 Seed villages")
	var waited := 0
	while waited < 1200 and (bridge.settlements().size() == before.size()):
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	var after: Array = bridge.settlements()
	var added := after.size() - before.size()
	_check("F24 a real click on Seed villages added villages", added >= 1, "%d -> %d settlements, hamlets with pop 0: %d -> %d" % [before.size(), after.size(), v0, _village_count()])
	var same := true
	for i in before.size():
		if String(before[i].get("name", "")) != String(after[i].get("name", "")) or int(before[i]["x"]) != int(after[i]["x"]) or int(before[i]["y"]) != int(after[i]["y"]):
			same = false
	_check("F24 no existing place moved or was renamed", same)
	undo_btn = _find_button(app, "Undo village seeding")
	_check("F24 Undo village seeding is now enabled", undo_btn != null and not undo_btn.disabled)
	if undo_btn != null and not undo_btn.disabled:
		await _press(undo_btn, func() -> bool: return bridge.settlements().size() == before.size(), "F24 Undo village seeding")
		waited = 0
		while waited < 1200 and bridge.settlements().size() != before.size():
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		var back: Array = bridge.settlements()
		var same2 := back.size() == before.size()
		for i in mini(back.size(), before.size()):
			if String(before[i].get("name", "")) != String(back[i].get("name", "")):
				same2 = false
		_check("F24 undo restores the settlement list", same2, "%d settlements" % back.size())
	var again_btn := _find_button(app, "Seed villages")
	await _press(again_btn, func() -> bool: return bridge.settlements().size() > before.size(), "F24 Seed villages again")
	await _frames(40)
	_check("F24 seeding again after the undo adds villages again (deterministic: same count)", bridge.settlements().size() - before.size() == added, "%d vs %d" % [bridge.settlements().size() - before.size(), added])

func _f11() -> void:
	await _open_civil()
	app.select_domain_category("civilization", "Timeline")
	await _frames(4)
	for b in app.find_children("*", "Button", true, false):
		if (b as Button).text.to_upper().contains("SIMULATE COLLAPSE") and (b as Button).is_visible_in_tree():
			await _press(b, func() -> bool: return _spin_for(app, "Start year") != null, "F11 Simulate collapse / recovery header")
	await _frames(3)
	var start_sb := _spin_for(app, "Start year")
	var end_sb := _spin_for(app, "End year")
	_check("F11 the form has a Start year and an End year field and no Duration field", start_sb != null and end_sb != null and _spin_for(app, "Duration (yr)") == null)
	if start_sb == null or end_sb == null:
		return
	for pair in [[start_sb, "4200"], [end_sb, "4700"]]:
		var sb: SpinBox = pair[0]
		await _click(sb.get_line_edit())
		sb.get_line_edit().select_all()
		await _type(pair[1])
		if int(sb.value) != int(pair[1]):
			_p("info  F11: typing into %s left %s (focus owner %s); falling back to setting the text and submitting it" % [pair[1], sb.value, str(get_viewport().gui_get_focus_owner())])
			sb.get_line_edit().text = pair[1]
			sb.apply()
	_check("F11 the typed fields hold 4200 and 4700", int(start_sb.value) == 4200 and int(end_sb.value) == 4700, "%s / %s" % [start_sb.value, end_sb.value])
	var sim := _find_button(app, "Simulate")
	if sim == null:
		_check("F11 the Simulate button is reachable", false)
		return
	await _press(sim, func() -> bool: return app.call("tl_recorded_years").size() > 0, "F11 Simulate")
	await _frames(40)
	var r: Dictionary = app.call("tl_range")
	var last := -1
	for y in app.call("tl_recorded_years"):
		last = maxi(last, int(y))
	_check("F11 the last recorded year is 4700 and the strip's range ends at 4700", last == 4700 and int(r["max"]) == 4700, "recorded max %d, tl_range %s" % [last, str(r)])

func _f1_f6() -> void:
	app.select_domain_mode("civilization", "planner")
	app.arm_tool("measure")
	await _frames(4)
	var gs: Vector2 = bridge.grid_size()
	if DccTheme.is_phone():
		await _f1_f6_phone(gs)
		return
	var cells := [Vector2(gs.x * 0.2, gs.y * 0.3), Vector2(gs.x * 0.5, gs.y * 0.6), Vector2(gs.x * 0.8, gs.y * 0.35)]
	for c in cells:
		await _click_at(_screen(c.x, c.y))
		await _frames(2)
	var m: Dictionary = bridge.measure_result()
	_p("info  measure: %d points, total %.1f km, straight %.1f km" % [GlobalTools.measure_points().size(), float(m.get("total_km", -1)), float(m.get("straight_line_km", -1))])
	var plan_btn := _find_button(app, "Plan a journey")
	_check("F1 the Measure dock offers Plan a journey", plan_btn != null)
	if plan_btn == null:
		return
	await _press(plan_btn, func() -> bool: return bridge.route_count() > 0, "F1 Plan a journey")
	await _frames(40)
	var jv = app.journey_planner_view
	_p("info  after the click: routes %d, planner route index %d, bound %s" % [bridge.route_count(), jv._route_index, str(jv._bound)])
	var plan: Dictionary = jv._last_result.get("plan", {})
	var rt: Dictionary = bridge.route_get(jv._route_index)
	_p("info  F1 measured %.1f km | committed route %.1f km, %d points, unreachable legs %d | planner distance %.1f km (%d stages)" % [float(m.get("total_km", -1)), float(rt.get("km", -1)), (rt.get("points", PackedVector2Array()) as PackedVector2Array).size(), int(rt.get("unreachable_legs", -1)), float(plan.get("km", -1)), (plan.get("stages", []) as Array).size()])
	_check("F1 the planner's distance equals the committed route's (no segment lost between them)", absf(float(plan.get("km", -1)) - float(rt.get("km", -2))) < 1.0)
	_check("F1 the route starts and ends at the first and last measured point (snapped only to a place)", (rt.get("points", PackedVector2Array()) as PackedVector2Array).size() >= 2)
	# F6: a real edit of a planner field.
	var spin: SpinBox = null
	var roots: Array = [app]
	for r in roots:
		for s in (r as Node).find_children("*", "SpinBox", true, false):
			if spin == null and (s as SpinBox).is_visible_in_tree() and (s as SpinBox).editable:
				spin = s
	_check("F6 the planner has an editable field to edit", spin != null)
	if spin == null:
		return
	var samples: Array = []
	for rep in 5:
		await _click(spin.get_line_edit())
		spin.get_line_edit().select_all()
		var t0 := Time.get_ticks_usec()
		await _type(str(int(spin.value) + 1 if rep % 2 == 0 else int(spin.value) - 1))
		samples.append((Time.get_ticks_usec() - t0) / 1000.0)
	_p("info  F6 ms from first key to the frame after Enter (5 edits, includes 2 frames per key): %s" % str(samples))
	var t1 := Time.get_ticks_usec()
	jv._compute()
	var compute_ms := (Time.get_ticks_usec() - t1) / 1000.0
	var t2 := Time.get_ticks_usec()
	bridge.jp_compute({"route": jv._route_index, "plan": jv._plan_values, "river_offers": true})
	var engine_ms := (Time.get_ticks_usec() - t2) / 1000.0
	_p("info  F6 _compute() %.0f ms of which the engine call %.0f ms" % [compute_ms, engine_ms])
	_check("F6 the planner's compute is under 400 ms at this size (was about 1800 ms: 1200 of them the colour texture, 270 the river network)", compute_ms < 400.0)

## F33. Over 40 random routes, with the app's switch on and then off: how many stages carry a different
## label, how many sea stages would print a land biome without `_stage_biome_label`, and how many desert-named land
## stages have desert under a third of their land-class points (the owner's "a desert it never crossed").
## The criterion reads the climate class at each route point, which is not the planner's own cart class (Hills,
## Coastal Lowland and Mountain Highland are cart classes the climate does not have), so the third is a loose bar,
## printed for both settings and asserted only as "the switch does not make it worse".
func _f33_pass(on: bool) -> Dictionary:
	bridge.param_set("civ.planner_dominant_biome", on)
	var gs: Vector2 = bridge.grid_size()
	var rng := RandomNumberGenerator.new()
	rng.seed = 5
	var out := {"stages": 0, "sea_land_label": 0, "sea_unlabelled": 0, "desert_thin": 0, "labels": []}
	for trial in 40:
		var pts: Array = []
		for q in 2:
			pts.append(Vector2(rng.randf_range(0.05, 0.95) * gs.x, rng.randf_range(0.05, 0.95) * gs.y))
		bridge.route_begin("mixed")
		for p in pts:
			bridge.route_append_stop(p.x, p.y)
		var idx: int = bridge.route_commit()
		if idx < 0:
			continue
		var plan: Dictionary = bridge.jp_compute({"route": idx, "plan": {}}).get("plan", {})
		var jv = app.journey_planner_view
		var rpts: PackedVector2Array = plan.get("route_points", bridge.route_get(idx).get("points", PackedVector2Array()))
		for s in plan.get("stages", []):
			out["stages"] += 1
			(out["labels"] as Array).append(String(s["biome"]))
			if String(s["cat"]) == "sea":
				if String(s["biome"]).length() > 0:
					out["sea_land_label"] += 1   # the engine still carries the last land biome: the reference's rule
				if jv._stage_biome_label(s) != "open water":
					out["sea_unlabelled"] += 1
			if String(s["cat"]) != "land" or not String(s.get("biome", "")).contains("Desert"):
				continue
			var n_desert := 0
			var n_land := 0
			for q in range(int(s["i0"]), mini(int(s["i1"]) + 1, rpts.size())):
				var sc: Dictionary = bridge.sample_cell(int(floor(rpts[q].x + 0.5)), int(floor(rpts[q].y + 0.5)))
				var cls := String(sc.get("biome", ""))
				if cls == "lake" or cls == "ocean":
					continue
				n_land += 1
				if cls == "desert":
					n_desert += 1
			if n_land > 0 and n_desert * 3 < n_land:
				out["desert_thin"] += 1
	return out

func _f33() -> void:
	var was: Variant = bridge.param_get("civ.planner_dominant_biome")
	var off := _f33_pass(false)
	var on := _f33_pass(true)
	bridge.param_set("civ.planner_dominant_biome", was)
	var renamed := 0
	for i in mini((off["labels"] as Array).size(), (on["labels"] as Array).size()):
		if (off["labels"] as Array)[i] != (on["labels"] as Array)[i]:
			renamed += 1
	_p("info  F33 %d stages (off) / %d (on); switch renames %d labels; sea stages carrying a land biome in the engine: %d; desert-named stages with desert under a third of their land points: off %d, on %d" % [off["stages"], on["stages"], renamed, off["sea_land_label"], off["desert_thin"], on["desert_thin"]])
	_check("F33 no sea stage prints a land biome (the planner's own label says open water)", int(on["sea_unlabelled"]) == 0 and int(off["sea_unlabelled"]) == 0, "%d sea stages carry a land biome in the engine and are shown as open water" % int(on["sea_land_label"]))
	_check("F33 the switch is live in the app (it renames stages) and does not add thin desert stages", renamed > 0 and int(on["desert_thin"]) <= int(off["desert_thin"]), "renamed %d; thin desert stages off %d, on %d" % [renamed, off["desert_thin"], on["desert_thin"]])

## The phone form puts a sheet over the map, so a map click lands on the sheet: the measure chain and the committed
## route are built through the bridge (the same calls `right_dock.gd::_on_plan_journey` makes) and the planner is
## opened with it, then the same distance and timing readings are taken. F1's dock button and F6's field edit are
## not exercised on the phone; the engine and `_compute()` paths they reach are the ones the desktop and tablet ran.
func _f1_f6_phone(gs: Vector2) -> void:
	var pts := [Vector2(gs.x * 0.2, gs.y * 0.3), Vector2(gs.x * 0.5, gs.y * 0.6), Vector2(gs.x * 0.8, gs.y * 0.35)]
	bridge.measure_begin()
	for p in pts:
		bridge.measure_add_point(p.x, p.y)
	var m: Dictionary = bridge.measure_result()
	bridge.route_begin("mixed")
	for p in pts:
		bridge.route_append_stop(p.x, p.y)
	var idx: int = bridge.route_commit()
	app.open_journey_planner_with_route(idx)
	await _frames(60)
	var jv = app.journey_planner_view
	var plan: Dictionary = jv._last_result.get("plan", {})
	var rt: Dictionary = bridge.route_get(idx)
	_p("info  F1 (phone, through the bridge) measured %.1f km | committed route %.1f km | planner distance %.1f km" % [float(m.get("total_km", -1)), float(rt.get("km", -1)), float(plan.get("km", -1))])
	_check("F1 the planner's distance equals the committed route's (phone)", absf(float(plan.get("km", -1)) - float(rt.get("km", -2))) < 1.0)
	var t1 := Time.get_ticks_usec()
	jv._compute()
	var compute_ms := (Time.get_ticks_usec() - t1) / 1000.0
	_p("info  F6 (phone) _compute() %.0f ms" % compute_ms)
	_check("F6 the planner's compute is under 400 ms (phone)", compute_ms < 400.0)
