extends Node
## SP-2 residual (the journey's departure date) and Ruling EA item 10 (a saved journey's per-leg override
## fingerprints), driven through the planner's own controls. Windowed, because the buttons are pressed with real
## input events (a mouse press at the button's centre, and on the touch run an `InputEventScreenTouch`), and the
## assertions read the engine AND the planner's own state.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _jdate_probe.tscn
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _jdate_probe.tscn -- --density 2.625 --force-touch   (touch form)
##
## Every line is prefixed `JDATE `: `ok    <what>` or `PROBE-FAIL  <what>`; `RESULT GREEN|FAIL  (<n> failures)` ends
## it. Exit 0 on GREEN, 1 on FAIL, 2 under `--headless` (refused: nothing here can be pressed or sized headless), 3 on
## the watchdog.
##
## What it proves, in order:
##  1. SAVE: a journey saved from the planner departs on the Timeline cursor's date -- `journey_list` carries
##     `start_day` equal to the cursor's day and `departure` as that date's text. A journey saved by the bare engine
##     call (`journey_save`, no day) has NO `start_day` key at all (not 0) and reads "year N" in the list.
##  2. FINGERPRINT: the override set before saving is stored with the leg it was set on (`journey_get`'s plan carries
##     `override_legs`), and `journey_override_report` verifies it against the unchanged world.
##  3. SET TO CURSOR: moving the cursor and PRESSING the planner's "set to cursor" button (real input) moves the
##     journey's `start_day` to the cursor's new day, and the marker's `departure` follows.
##  4. DETECT AND DROP: the stored fingerprint is replaced by one for a leg the route does not have (a stand-in for
##     a regenerate, a sculpt or an engine change moving the legs; the real world is not edited here), the report
##     lists the override as `dropped`, LOADING the journey in the planner removes it from the working overrides and
##     leaves the notice naming stage 1, and the engine's own marker plan stays computed.
##  5. GO TO: pressing "go to" (a touch event on the touch run) puts the cursor on the journey's departure.
##  6. OLD SAVES: a journey with overrides and no fingerprint is "unverified" in the report, never dropped.
##
## Honest limits: the world is not actually re-cut (step 4 changes the stored key, not the terrain), which is what
## `check_override_legs`'s unit tests and the Rust-side tests cover at the decision level.

var _fails := 0
var app: Node
var bridge: Node
var _touch := false

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("JDATE ok    %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
	else:
		print("JDATE PROBE-FAIL  %s%s" % [what, ("  -- " + detail) if detail != "" else ""])
		_fails += 1

func _row(id: int) -> Dictionary:
	for r in bridge.journey_list():
		if int((r as Dictionary).get("id", -1)) == id:
			return r
	return {}

## The first node named `n` under `root`, depth first.
func _find(root: Node, n: String) -> Node:
	if root.name == n:
		return root
	for c in root.get_children():
		var f := _find(c, n)
		if f != null:
			return f
	return null

## A real press on `button`: mouse down and up at its centre, or a touch down and up there on the touch run.
func _press(button: Control) -> void:
	## The event goes to the viewport the button lives in (the planner can be its own window, whose coordinates are
	## not the main window's), at the button's centre in that viewport.
	## An embedded window (the planner's, on the desktop) takes its input through the viewport that embeds it, at
	## the button's centre shifted by the window's own position; a native window takes it directly.
	var vp := button.get_viewport()
	var c := button.get_global_rect().get_center()
	var win := button.get_window()
	if win != null and win.is_embedded():
		vp = win.get_parent().get_viewport()
		c += Vector2(win.position)
	if _touch:
		## One finger, as the shell sees it on a device: the touch event AND its emulated mouse event (the
		## shell's controls answer the mouse one; the touch one drives its gesture code).
		for down in [true, false]:
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.position = c
			t.pressed = down
			vp.push_input(t)
			var m := InputEventMouseButton.new()
			m.device = InputEvent.DEVICE_ID_EMULATION
			m.button_index = MOUSE_BUTTON_LEFT
			m.pressed = down
			m.position = c
			m.global_position = c
			m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
			vp.push_input(m)
			await _frames(2)
	else:
		## A move first, so the button is hovered when the press lands (a bare press at a position the pointer never
		## visited is not a click on every Control).
		var mv := InputEventMouseMotion.new()
		mv.position = c
		mv.global_position = c
		vp.push_input(mv)
		await _frames(2)
		for down in [true, false]:
			var m := InputEventMouseButton.new()
			m.button_index = MOUSE_BUTTON_LEFT
			m.position = c
			m.global_position = c
			m.pressed = down
			m.device = InputEvent.DEVICE_ID_EMULATION
			m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
			vp.push_input(m)
			await _frames(2)
	await _frames(4)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("JDATE REFUSED: headless")
		get_tree().quit(2)
		return
	## The touch run is the one started with `--force-touch` (and `--density`, both read by the shell itself).
	_touch = "--force-touch" in OS.get_cmdline_user_args()
	get_tree().create_timer(420.0).timeout.connect(func():
		print("JDATE PROBE-FAIL watchdog")
		get_tree().quit(3))
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("JDATE dll mtime: %s  touch run: %s" % [FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")), _touch])
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	bridge = app.bridge
	if not bridge.has_world:
		print("JDATE PROBE-FAIL generate failed")
		get_tree().quit(1)
		return
	var towns: Array = bridge.settlements()
	var gs: Vector2 = bridge.grid_size()
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	if pairs.is_empty():
		print("JDATE PROBE-FAIL no settlement pair")
		get_tree().quit(1)
		return
	var a: Dictionary = towns[pairs[0][1]]
	var b: Dictionary = towns[pairs[0][2]]
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(a["x"]), float(a["y"]))
	bridge.route_append_stop(float(b["x"]), float(b["y"]))
	var ridx: int = bridge.route_commit()
	app.tl_set_year(412)
	app.tl_set_day(73)
	app.open_journey_planner()
	await _frames(10)
	var view = app.journey_planner_view
	view._route_index = ridx
	view._compute()
	await _frames(6)

	# 1. SAVE through the planner's own save function, with a manual override on stage 0.
	view._set_stage_override(0, "hours", 9.0)
	await _frames(4)
	view._commit_saved_journey("Probe road")
	await _frames(6)
	var saved: Array = bridge.journey_list()
	_check("a journey was saved", not saved.is_empty())
	var id := int((saved.back() as Dictionary).get("id", -1))
	var r := _row(id)
	_check("the planner-saved journey carries the cursor's day", r.has("start_day") and int(r["start_day"]) == 73, str(r))
	_check("its departure text is the cursor's date (day 73 = 15 March)", String(r.get("departure", "")) == "412-03-15")
	var bare: int = bridge.journey_save("Bare", String(r.get("party_preset", "")), ridx, 400)
	var rb := _row(bare)
	_check("a journey saved without a day has NO start_day key", not rb.has("start_day"), str(rb))
	_check("... and is not reported as 1 January picked: its year is kept", int(rb.get("start_year", -1)) == 400)
	bridge.journey_delete(bare)

	# 2. FINGERPRINT stored and verified against the unchanged world.
	var full: Dictionary = bridge.journey_get(id)
	var legs: Dictionary = (full.get("plan", {}) as Dictionary).get("override_legs", {})
	_check("the override's leg fingerprint was saved", legs.has(0) and String(legs[0]).contains(":"), str(legs))
	var rep: Dictionary = bridge.journey_override_report(id)
	_check("the report ran", bool(rep.get("ok", false)), str(rep))
	_check("the override is verified", (rep.get("verified", []) as Array).has(0) and (rep.get("dropped", []) as Array).is_empty(), str(rep))

	# 3. SET TO CURSOR by pressing the real button.
	app.tl_set_year(430)
	app.tl_set_day(200)
	await _frames(3)
	view._refresh_route_choice()
	await _frames(4)
	var set_btn := _find(view._left_route_section, "DepartureSet") as Control
	_check("the departure row has a set-to-cursor button", set_btn != null)
	if set_btn != null:
		_check("... and it is visible in the tree at %s" % set_btn.get_global_rect(), set_btn.is_visible_in_tree() and set_btn.get_global_rect().size.x > 20.0)
	if set_btn != null:
		await _press(set_btn)
		r = _row(id)
		_check("pressing it moved the departure to the cursor", int(r.get("start_year", 0)) == 430 and int(r.get("start_day", -1)) == 200, str(r))
		var pos := {}
		for p in bridge.journey_positions():
			if int((p as Dictionary).get("id", -1)) == id:
				pos = p
		_check("the marker's departure follows", String(pos.get("departure", "")) == String(r.get("departure", "x")), str(pos.get("departure", "")))
		_check("... and says a day was set", pos.has("start_day"))

	# 5. GO TO: move the cursor away, then press "go to".
	app.tl_set_year(100)
	app.tl_set_day(0)
	await _frames(3)
	view._refresh_route_choice()
	await _frames(4)
	var go_btn := _find(view._left_route_section, "DepartureGo") as Control
	if go_btn != null:
		await _press(go_btn)
		_check("go to put the cursor on the departure", app.tl_year() == 430 and bridge.get_civ_day_of_year() == 200, "%d / %d" % [app.tl_year(), bridge.get_civ_day_of_year()])
	else:
		_check("the departure row has a go-to button", false)

	# 4. DETECT AND DROP: a fingerprint for a leg this route does not have.
	_check("tamper accepted", bridge.journey_set_override_legs(id, {0: "river:900-901@1,1"}))
	rep = bridge.journey_override_report(id)
	var dropped: Array = rep.get("dropped", [])
	_check("the changed leg is reported dropped", dropped.size() == 1 and int((dropped[0] as Dictionary).get("stage", -1)) == 0, str(rep))
	_check("it names the old leg's category", String((dropped[0] as Dictionary).get("was", "")) == "river")
	var idx := -1
	for i in view._journeys.size():
		if int((view._journeys[i] as Dictionary).get("engine_id", -1)) == id:
			idx = i
	_check("the planner holds the journey", idx >= 0)
	if idx >= 0:
		view._journeys[idx]["stage_overrides"] = {0: {"hours": 9.0}}
		view._load_journey(idx)
		await _frames(6)
		_check("loading it removed the stale override", not view._stage_overrides.has(0), str(view._stage_overrides))
		_check("the notice names stage 1", view._override_notice.contains("stage 1"), view._override_notice)
	var still := false
	for p in bridge.journey_positions():
		if int((p as Dictionary).get("id", -1)) == id:
			still = (p as Dictionary).has("x") or (p as Dictionary).has("error")
	_check("the marker plan still answers after the drop", still)

	# 6. OLD SAVES: overrides with no fingerprint are unverified, never dropped.
	_check("clearing the keys accepted", bridge.journey_set_override_legs(id, {}))
	rep = bridge.journey_override_report(id)
	_check("no fingerprint means unverified, not dropped", (rep.get("unverified", []) as Array).has(0) and (rep.get("dropped", []) as Array).is_empty(), str(rep))

	print("JDATE RESULT %s  (%d failures)" % ["GREEN" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
