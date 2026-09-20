extends Node
## OUTSTANDING_WORK.md row (re-derived 2026-09-20): PLAN's phone-sheet header
## TITLE half ("PLAN · STAGE n") goes stale when a stage is isolated, because
## `_on_stage_clicked()` rebuilds the inspector/matrix but never reaches
## `_apply_result()` -- the one place `app._refresh_phone_sheet_header()` was
## wired in `554d953`'s sibling fix (that commit's own comment names this
## exact residual). `dcc_shell.gd::_refresh_phone_sheet_header()`'s PLAN
## branch reads `journey_planner_view.gd::phone_header_info()`, which derives
## the title from `_isolated_stage` directly -- so the header is only ever as
## fresh as the last `_refresh_phone_sheet_header()` call, not as fresh as
## `_isolated_stage` itself.
##
## This probe drives the REAL path: generate a world, commit a real route,
## open PLAN (title/subtitle correct on entry, per `_pick_phone_tab()`'s own
## call), then call `_on_stage_clicked(0, true)` -- the exact call
## `"isolate stage"`'s context-menu action makes (`journey_planner_view.gd`
## around the `DccWidgets.action(actions, "isolate stage", ...)` site) --
## with the PLAN tab left alone, exactly as the subtitle regression's own
## scenario left it alone. Asserts the title reads "PLAN · STAGE 1" after.
##
##   godot --path . --resolution 1080x2340 --rendering-driver opengl3 _stagetitle_probe.tscn -- --force-touch
##
## NOT --headless: `ImageTexture` / rendering aside, Label.text is engine
## state and *should* be readable headless per `_detent_probe.gd`'s own
## header -- but the coordinating brief asked for a windowed probe, and a
## real framebuffer is strictly more evidence than headless, so this one
## runs windowed. `--force-touch` is read by `dcc_shell.gd::_read_layout_env()`
## before this probe runs; without it `is_phone()` is false and the phone
## sheet header this probe reads does not exist.

func _log(s: String) -> void:
	print("[stagetitle] %s" % s)

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
	if places.size() < 2:
		get_tree().quit(_fail("need at least two settlements for a route"))
		return

	var a0: Dictionary = places[0]
	var a1: Dictionary = places[0]
	var best := -1.0
	for p in places:
		var d: float = abs(float(p["x"]) - float(a0["x"])) + abs(float(p["y"]) - float(a0["y"]))
		if d > best:
			best = d
			a1 = p

	bridge.route_begin("mixed")
	bridge.route_append_stop(float(a0["x"]), float(a0["y"]))
	bridge.route_append_stop(float(a1["x"]), float(a1["y"]))
	var idx_a: int = bridge.route_commit()
	_log("route #%d: %s -> %s" % [idx_a, a0["name"], a1["name"]])
	if idx_a < 0:
		get_tree().quit(_fail("route_commit refused the route"))
		return

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

	var title_on_entry: String = String(app._phone_sheet_title.text)
	_log("[after tab-pick, no stage isolated] title=%s" % title_on_entry)
	if title_on_entry != "PLAN":
		fails += _fail("entry title not plain PLAN -- got %s (cannot test isolate-staleness from here)" % title_on_entry)

	var plan: Dictionary = jp._last_result.get("plan", {}) if bool(jp._last_result.get("ok", false)) else {}
	var stages: Array = plan.get("stages", [])
	_log("stages in committed plan: %d" % stages.size())
	if stages.is_empty():
		get_tree().quit(_fail("committed plan has no stages -- cannot isolate one"))
		return

	# The regression scenario: isolate stage 0 via the exact call the
	# "isolate stage" context-menu action makes, PLAN tab never re-picked.
	jp._on_stage_clicked(0, true)
	await get_tree().process_frame

	if String(app._phone_tab) != "plan":
		get_tree().quit(_fail("something re-routed off the PLAN tab mid-probe"))
		return
	if jp._isolated_stage != 0:
		get_tree().quit(_fail("_on_stage_clicked(0, true) did not set _isolated_stage=0 -- got %d" % jp._isolated_stage))
		return

	var title_after: String = String(app._phone_sheet_title.text)
	var sub_after: String = String(app._phone_sheet_subtitle.text)
	_log("[after isolating stage 0, PLAN tab never re-picked] title=%s sub=%s" % [title_after, sub_after])
	var want_title := "PLAN · STAGE 1"
	if title_after != want_title:
		fails += _fail(
			"title did not advance on stage-isolate -- got %s want %s (this IS the row: header TITLE stays plain PLAN)"
			% [title_after, want_title])
	else:
		_log("OK: title advanced to PLAN · STAGE 1 with no tab re-pick")

	# Un-isolate and confirm the title reverts (same call path, isolate toggle).
	jp._on_stage_clicked(0, true)
	await get_tree().process_frame
	var title_reverted: String = String(app._phone_sheet_title.text)
	_log("[after un-isolating] title=%s" % title_reverted)
	if title_reverted != "PLAN":
		fails += _fail("title did not revert to plain PLAN after un-isolating -- got %s" % title_reverted)

	_log("RESULT %s  failures=%d" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(0 if fails == 0 else 1)
