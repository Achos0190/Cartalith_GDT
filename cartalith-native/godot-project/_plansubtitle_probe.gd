extends Node
## OUTSTANDING_WORK.md §2.7, found 2026-09-13 by the wf53 verifier: PLAN's
## phone sheet subtitle ("journey · <from> → <to>") went stale when a new
## plan was computed while PLAN was already the open tab -- it only refreshed
## on the next tab press. `_detent_probe.gd`'s own PLAN header section (the
## "[plan/none]" / "[plan/route]" / "[plan/stage]" checks) pokes
## `journey_planner_view.gd::_last_result` directly with a hand-built
## dictionary and calls `app._refresh_phone_sheet_header()` itself -- proving
## the STRING FORMAT, never the wiring the row is about, since it never once
## calls the real `_compute()` and never leaves the tab already on "plan"
## while a second plan lands. This probe drives the real path instead: two
## real committed routes between real generated settlements, the real
## `EngineBridge.jp_compute()` through `journey_planner_view.gd::_compute()`
## (the exact statements `_refresh_route_choice()`'s own route-picker
## callback runs -- `journey_planner_view.gd:706-713` -- not a fabricated
## plan), with PLAN picked ONCE and never re-picked.
##
##   godot --headless --path . _plansubtitle_probe.tscn -- --force-touch
##
## Flags read: `--force-touch` only, consumed by `dcc_shell.gd`'s own
## `_read_layout_env()` (via `OS.get_cmdline_user_args()`) before this probe
## ever runs -- without it the shell boots desktop and `is_phone()` (and so
## the phone sheet header this probe reads) does not exist.
##
## Headless is sound here per `_detent_probe.gd`'s own header: nothing below
## reads a pixel, only `Control.size`/`.text`, which the layout pass computes
## under `--headless` same as windowed.

func _log(s: String) -> void:
	print("[plansub] %s" % s)

func _fail(msg: String) -> int:
	_log("FAIL: %s" % msg)
	return 1

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_log("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var fails := 0
	## A sized `SubViewport`, not the bare root -- `_detent_probe.gd`'s own
	## pattern. `_compute_layout_mode()` reads `get_viewport_rect().size` off
	## `app`'s nearest Viewport ancestor, and the root window under
	## `--headless` reports 0x0, which fails `_phone`'s aspect check no matter
	## how touch is forced. A real, portrait-shaped size is what makes
	## `is_phone()` true at all.
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().process_frame

	if not app.is_phone():
		_log("ABORT not phone mode -- pass --force-touch")
		get_tree().quit(2)
		return

	var bridge = app.bridge
	bridge.generate({
		"seed": 24601, "width_km": 2560.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.5,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout
	if not bridge.has_world:
		get_tree().quit(_fail("world generation did not land"))
		return

	var places: Array = bridge.settlements()
	_log("settlements: %d" % places.size())
	if places.size() < 3:
		get_tree().quit(_fail("need at least three settlements for two DISTINCT-endpoint routes"))
		return

	# Route A: the pair furthest apart (room for a real solved path).
	var a0: Dictionary = places[0]
	var a1: Dictionary = places[0]
	var best := -1.0
	for p in places:
		var d: float = abs(float(p["x"]) - float(a0["x"])) + abs(float(p["y"]) - float(a0["y"]))
		if d > best:
			best = d
			a1 = p
	# Route B: a DIFFERENT pair, sharing no endpoint with Route A, so the two
	# routes' first/last stop names are guaranteed to differ -- the whole
	# point of the "second, different plan" scenario.
	var b0: Dictionary = {}
	var b1: Dictionary = {}
	for p in places:
		if String(p["name"]) != String(a0["name"]) and String(p["name"]) != String(a1["name"]):
			if b0.is_empty():
				b0 = p
			elif b1.is_empty() and String(p["name"]) != String(b0["name"]):
				b1 = p
	if b0.is_empty() or b1.is_empty():
		get_tree().quit(_fail("could not find a second settlement pair disjoint from route A"))
		return

	bridge.route_begin("mixed")
	bridge.route_append_stop(float(a0["x"]), float(a0["y"]))
	bridge.route_append_stop(float(a1["x"]), float(a1["y"]))
	var idx_a: int = bridge.route_commit()
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(b0["x"]), float(b0["y"]))
	bridge.route_append_stop(float(b1["x"]), float(b1["y"]))
	var idx_b: int = bridge.route_commit()
	_log("route A #%d: %s -> %s   route B #%d: %s -> %s" % [
		idx_a, a0["name"], a1["name"], idx_b, b0["name"], b1["name"]])
	if idx_a < 0 or idx_b < 0:
		get_tree().quit(_fail("route_commit refused one of the two routes"))
		return

	# -- Real UI entry: pick the PLAN tab ONCE. ------------------------------
	# `_pick_phone_tab("plan")` -> `open_journey_planner()` ->
	# `journey_planner_view.open()`/`_show()` -> `_refresh_route_choice()`
	# defaults `_route_index` to 0 (route A) and `_show()`'s own trailing
	# `_compute()` computes A's plan; `_pick_phone_tab()`'s own tail then
	# calls `_refresh_phone_sheet_header()` -- so the header is correct here
	# ON ENTRY regardless of this row's bug. The bug is what happens next,
	# with the tab left alone.
	app._pick_phone_tab("plan")
	await get_tree().process_frame
	await get_tree().process_frame

	var jp = app.journey_planner_view
	if jp == null or not is_instance_valid(jp):
		get_tree().quit(_fail("app.journey_planner_view missing after PLAN tab pick"))
		return
	if String(app._phone_tab) != "plan":
		get_tree().quit(_fail("PLAN tab pick left _phone_tab=%s" % app._phone_tab))
		return

	var title_before: String = String(app._phone_sheet_title.text)
	var sub_after_a: String = String(app._phone_sheet_subtitle.text)
	_log("[after tab-pick, route A default]  title=%s  sub=%s" % [title_before, sub_after_a])
	var want_a := "journey · %s → %s" % [a0["name"], a1["name"]]
	if sub_after_a != want_a:
		_log("FAIL: tab-pick's own header did not carry route A -- got %s want %s" % [sub_after_a, want_a])
		fails += 1

	# -- The actual regression scenario: a SECOND, DIFFERENT plan computed --
	# while PLAN stays the open tab, no re-pick. This is jp._compute()'s
	# real real path -- the exact statements
	# `journey_planner_view.gd::_refresh_route_choice()`'s own route-picker
	# callback runs (lines 706-713 there), not a poke of `_last_result`.
	jp._route_index = idx_b
	jp._stage_overrides.clear()
	jp._layovers.clear()
	jp._selected_stage = 0
	jp._isolated_stage = -1
	jp._trim = Vector2(0.0, 1.0)
	jp._compute()
	await get_tree().process_frame

	if String(app._phone_tab) != "plan":
		get_tree().quit(_fail("something re-routed off the PLAN tab mid-probe"))
		return

	var sub_after_b: String = String(app._phone_sheet_subtitle.text)
	_log("[after computing route B's plan, PLAN tab NEVER re-picked]  sub=%s" % sub_after_b)
	var want_b := "journey · %s → %s" % [b0["name"], b1["name"]]
	if sub_after_b != want_b:
		fails += _fail(
			"header did not refresh on the second plan -- got %s want %s (this IS the row: %s)"
			% [sub_after_b, want_b, "stale until the PLAN tab is picked again"])
	elif sub_after_b == sub_after_a:
		fails += _fail("subtitle unchanged across two genuinely different routes -- %s" % sub_after_b)
	else:
		_log("OK: subtitle changed from route A's pair to route B's pair with no tab re-pick")

	_log("RESULT %s  failures=%d" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(0 if fails == 0 else 1)
