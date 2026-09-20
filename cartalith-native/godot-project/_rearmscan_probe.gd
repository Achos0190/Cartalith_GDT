extends Node
## REARM rows A and B (lane CHROME, 2026-09-13 batch). Two defects, one cause:
## `dcc_shell.gd::_select_domain()` emits `workspace_changed` unconditionally
## on every call, including a bare re-entry that changes neither the (domain,
## mode) pair nor the armed tool.
##
##   Row A -- `app.gd::_on_workspace_changed()` rebuilt the tool-options row,
##   the rail foot and (for Journey specifically) `timeline_row` from the
##   domain's own idle default on every such call, silently discarding
##   whatever the actually-armed tool had painted there at arm time.
##
##   Row B -- `journey_planner_view.gd::_recompute_visibility()`'s old
##   `should_show` formula (bound && armed_tool=="journey" && domain==
##   "civilization") never noticed a mode-only change, so navigating to a
##   DIFFERENT civilization destination while Journey stayed the last-armed
##   tool (Landmarks/Factions/Military/phone Simulation-model row (Timeline), none of which go
##   through `_on_rail_node_pressed()`'s own disarm) left the planner's own
##   panels shown on top of the destination the user actually asked for. A
##   2026-09-12 attempt fixed the two NAMED re-entry points (a bare
##   `select_domain("civilization")` or `select_domain_mode("civilization",
##   "planner")`) by reasserting unconditionally on every unchanged call, which
##   swallowed those four destinations instead -- reverted
##   (`.claude/resume-2026-09-12/rearm_2026-09-12.patch`).
##
## This probe does five things, each independent of the ones before it so a
## later failure never hides an earlier result:
##
##   1. Empirical subscriber roster -- `app.tool_armed`/`app.workspace_changed`
##      connections read back from the live tree, ground truth for "who
##      subscribes" (a lambda's owning object is only knowable at runtime).
##   2. Phone-only positive control (needs `--force-touch`) -- the filed
##      repro: PLAN, close the phone sheet directly (`_set_sheet_open`, never
##      `close()`/`_hide()`), PLAN again.
##   3. Row B's two NAMED entry points, on whichever composition this runs
##      under: a bare `select_domain("civilization")` and a bare
##      `select_domain_mode("civilization","planner")`, plus the real
##      `_on_rail_node_pressed("civilization","planner")` as a control.
##   4. General `_workspace_panels` invariant sweep across every non-Journey
##      armable tool x 3 domains (unchanged from the 2026-09-12 lane).
##   5. NEW this batch -- Row A: re-enter Way's own domain/mode (both call
##      shapes) and assert `tool_options_row`'s text is untouched; same for
##      Journey's rail foot ("JOURNEY", never "PLANNER") and its
##      "JOURNEY PLANNER" options row; and JP-13's `timeline_row` band survives
##      a re-entry (checked by object identity, not text, since app.gd's own
##      rebuild frees the band view rather than merely renaming a label).
##   6. NEW this batch -- Row B: the four previously-swallowed destinations
##      (Landmarks, Factions, Military, and the phone Simulation-model row's
##      Timeline category -- "Simulation" until Ruling L -- reached via
##      `select_domain_category()` exactly as `menus.gd`, `faction_roster_
##      window.gd`, `cartography_workspace.gd` and `phone_menu.gd::
##      _go_simulation()` call it) each hide the planner and show the civ
##      panel while Journey is still the last-armed tool.
##
## Run desktop (rail foot exists only in this composition):
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _rearmscan_probe.tscn
## Run phone (adds Part 2 and the phone composition legs of parts 3/5/6):
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _rearmscan_probe.tscn -- --force-touch

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, msg: String) -> void:
	if cond:
		print("  OK    ", msg)
	else:
		_fail += 1
		print("  FAIL  ", msg)

func _civ_visible(app) -> bool:
	var p = app._workspace_panels.get("civilization")
	return p != null and p.visible

func _jp_left_visible(jp) -> bool:
	return bool(jp.get("_left_panel").visible)

func _jp_center_visible(jp) -> bool:
	return bool(jp.get("_center_panel").visible)

## `tool_options_row`'s own `_tool_options_label()` convention (both
## `civilization_workspace.gd` and `infrastructure_workspace.gd`, and
## `app.gd`/`journey_planner_view.gd` themselves) always adds the caption
## label first, so child 0's text is that row's identity.
func _row_text(app) -> String:
	var row: HBoxContainer = app.tool_options_row
	if row == null or row.get_child_count() == 0:
		return ""
	var c = row.get_child(0)
	return c.text if c is Label else ""

func _dump_connections(sig: Signal, label: String) -> void:
	var conns: Array = sig.get_connections()
	print("--- %s: %d connection(s) ---" % [label, conns.size()])
	for c in conns:
		var cb: Callable = c["callable"]
		var obj: Object = cb.get_object()
		if obj == null:
			print("    (freed target)  method=", cb.get_method())
			continue
		var scr: Script = obj.get_script() as Script
		var owner_desc: String = obj.get_class()
		if scr != null:
			owner_desc += " [%s]" % scr.resource_path.get_file()
		print("    %-40s method=%s" % [owner_desc, cb.get_method()])

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var phone_requested := "--force-touch" in OS.get_cmdline_user_args()
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340) if phone_requested else Vector2i(1600, 900)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	var phone := bool(app.is_phone())
	if phone_requested and not phone:
		print("[FATAL] --force-touch given but did not boot into phone composition")
		get_tree().quit(1); return
	if app.get("journey_planner_view") == null:
		print("[FATAL] app.journey_planner_view is null"); get_tree().quit(1); return
	var jp = app.get("journey_planner_view")
	if not bool(jp.get("_bound")):
		print("[FATAL] journey planner not bound -- jp_* not exposed by this build's GDExtension binary")
		get_tree().quit(1); return

	var dom0: String = str(app.call("active_domain"))
	var mode0: String = str(app.call("active_mode", dom0))
	var tool0: String = str(app.get("armed_tool"))
	print("[BOOT] composition=%s dom0=%s mode0=%s tool0=%s" % ["phone" if phone else "desktop", dom0, mode0, tool0])

	# ============================================================ Part 1 ====
	print("\n=== Part 1: empirical signal-subscriber roster ===")
	_dump_connections(app.tool_armed, "app.tool_armed")
	_dump_connections(app.workspace_changed, "app.workspace_changed")

	# ============================================================ Part 2 ====
	if phone:
		print("\n=== Part 2 (phone only): positive control -- PLAN -> sheet-close -> PLAN ===")
		app.call("select_domain", "world")
		app.call("arm_tool", "inspect")
		await _frames(3)
		app.call("open_journey_planner")
		await _frames(10)
		_ok(str(app.get("armed_tool")) == "journey", "journey armed after first PLAN")
		_ok(bool(jp.get("_active")), "planner _active after first PLAN")
		_ok(not _civ_visible(app), "civ panel hidden after first PLAN")
		_ok(_jp_left_visible(jp) and _jp_center_visible(jp), "planner panels shown after first PLAN")
		_ok(bool(app.left_dock.visible), "left sheet open after first PLAN")

		app.call("_set_sheet_open", "left", false)
		await _frames(5)
		_ok(str(app.get("armed_tool")) == "journey", "tool STILL armed after sheet close (never disarmed)")
		_ok(str(app.call("active_domain")) == "civilization", "domain STILL civilization after sheet close")
		_ok(bool(jp.get("_active")), "planner _active STILL true after sheet close (recompute never fired)")

		app.call("open_journey_planner")
		await _frames(10)
		_ok(not _civ_visible(app), "civ panel hidden after RE-open while armed (open_journey_planner path)")
		_ok(_jp_left_visible(jp) and _jp_center_visible(jp), "planner panels shown after RE-open while armed")
		_ok(bool(app.left_dock.visible), "left sheet reopened after RE-open while armed")
		print("  -- positive control subtotal: ", _fail, " failure(s) so far")
	else:
		print("\n=== Part 2 SKIPPED (needs `-- --force-touch`; run separately) ===")

	# ============================================================ Part 3 ====
	print("\n=== Part 3: Row B's two named entry points (composition=%s) ===" % ("phone" if phone else "desktop"))
	app.call("open_journey_planner")
	await _frames(10)
	_ok(str(app.get("armed_tool")) == "journey" and bool(jp.get("_active")) and not _civ_visible(app),
		"precondition: planner armed, active, civ hidden")

	print("--- 3a: bare select_domain(\"civilization\") (Window > Workspace, phone_menu.gd::_go_civilization()) ---")
	app.call("select_domain", "civilization")
	await _frames(10)
	_ok(not _civ_visible(app), "civ panel hidden after select_domain(\"civilization\") while journey armed+active")
	_ok(_jp_left_visible(jp) and _jp_center_visible(jp), "planner panels still shown after 3a")

	app.call("open_journey_planner")
	await _frames(10)
	print("--- 3b: bare select_domain_mode(\"civilization\",\"planner\") (a Journey layout; a rail re-click) ---")
	app.call("select_domain_mode", "civilization", "planner")
	await _frames(10)
	_ok(not _civ_visible(app), "civ panel hidden after select_domain_mode() while journey armed+active")
	_ok(_jp_left_visible(jp) and _jp_center_visible(jp), "planner panels still shown after 3b")

	app.call("open_journey_planner")
	await _frames(10)
	print("--- 3c: control -- the real _on_rail_node_pressed(\"civilization\",\"planner\") ---")
	if app.has_method("_on_rail_node_pressed"):
		app.call("_on_rail_node_pressed", "civilization", "planner")
		await _frames(10)
		_ok(not _civ_visible(app), "civ panel hidden after the real rail-node-press function (control)")
	else:
		print("  SKIP -- _on_rail_node_pressed not callable on this instance")

	# ============================================================ Part 4 ====
	print("\n=== Part 4: general _workspace_panels invariant sweep (non-journey tools) ===")
	## Not `mode0`: that is `dom0`'s (usually WORLD's) mode, e.g. "a", which is
	## not a civilization rail node and only ever produced a harmless
	## `push_warning` here -- "landmarks" is always a valid CIVIL node and this
	## sweep only needs a valid, non-planner starting mode.
	app.call("select_domain_mode", "civilization", "landmarks")
	app.call("arm_tool", "inspect")
	await _frames(5)
	var sweep_tools := ["sculpt", "paint", "measure", "settlement", "territory", "way", "route", "label", "icon", "region"]
	var sweep_domains := ["world", "cartography", "civilization"]
	for tid in sweep_tools:
		for d in sweep_domains:
			app.call("select_domain", d)
			app.call("arm_tool", tid)
			await _frames(2)
			var other := "world" if d != "world" else "cartography"
			app.call("select_domain", other)
			app.call("arm_tool", tid)  # no-op re-arm: already armed
			await _frames(2)
			app.call("select_domain", d)  # bare re-select, no dedicated opener
			await _frames(2)
			var bad := false
			for key in app._workspace_panels.keys():
				var want: bool = (String(key) == d)
				var got: bool = bool(app._workspace_panels[key].visible)
				if got != want:
					bad = true
			if bad:
				_fail += 1
				print("  FAIL  tool=%-11s domain=%-11s -- _workspace_panels mismatch" % [tid, d])
	print("  swept %d tools x %d domains" % [sweep_tools.size(), sweep_domains.size()])

	# ============================================================ Part 5 ====
	print("\n=== Part 5: Row A -- re-enter a tool's own domain/mode, chrome must not move ===")
	print("--- 5a: Way (INFRA), mode 'infra' ---")
	app.call("select_domain_mode", "civilization", "infra")
	app.call("arm_tool", "way")
	await _frames(5)
	var way_text := _row_text(app)
	_ok(way_text == "INFRA · WAY", "precondition: Way's own row painted (%s)" % way_text)
	app.call("select_domain", "civilization")
	await _frames(5)
	_ok(_row_text(app) == way_text, "Way's row unchanged after bare select_domain(\"civilization\") (got %s)" % _row_text(app))
	app.call("select_domain_mode", "civilization", "infra")
	await _frames(5)
	_ok(_row_text(app) == way_text, "Way's row unchanged after select_domain_mode(civ,\"infra\") re-entry (got %s)" % _row_text(app))
	app.call("arm_tool", "inspect")
	await _frames(3)

	print("--- 5b: Journey rail foot + options row, both re-entry shapes ---")
	app.call("open_journey_planner")
	await _frames(10)
	if app.rail_foot != null:
		_ok(String(app.rail_foot.text) == "JOURNEY", "precondition: rail foot reads JOURNEY (got %s)" % app.rail_foot.text)
	else:
		print("  (rail_foot is null on this composition -- phone; skipping the two rail_foot assertions below)")
	_ok(_row_text(app) == "JOURNEY PLANNER", "precondition: options row reads JOURNEY PLANNER (got %s)" % _row_text(app))

	app.call("select_domain", "civilization")
	await _frames(5)
	if app.rail_foot != null:
		_ok(String(app.rail_foot.text) == "JOURNEY", "rail foot STILL reads JOURNEY after bare select_domain (got %s)" % app.rail_foot.text)
	_ok(_row_text(app) == "JOURNEY PLANNER", "options row STILL reads JOURNEY PLANNER after bare select_domain (got %s)" % _row_text(app))

	app.call("select_domain_mode", "civilization", "planner")
	await _frames(5)
	if app.rail_foot != null:
		_ok(String(app.rail_foot.text) == "JOURNEY", "rail foot STILL reads JOURNEY after select_domain_mode re-entry (got %s)" % app.rail_foot.text)
	_ok(_row_text(app) == "JOURNEY PLANNER", "options row STILL reads JOURNEY PLANNER after select_domain_mode re-entry (got %s)" % _row_text(app))

	print("--- 5c: JP-13 -- timeline_row keeps the planner's own content across the same two re-entries ---")
	## No world is generated in this probe, so `_rebuild_timeline_band()`
	## (`journey_planner_view.gd`) takes its `_route_index < 0` branch and
	## timeline_row's content is the plain "no committed route selected"
	## label rather than a `_TimelineBandView` -- still real content this
	## file put there, and still wiped and replaced by a stray
	## `_fill_timeline_strip()` exactly as a real band would be, since
	## `_fill_timeline_strip()` frees every child of `timeline_row`
	## unconditionally before rebuilding the desktop/phone default strip.
	if app.timeline_row == null:
		print("  (timeline_row is null on this composition; skipping)")
	else:
		var marker := "no committed route selected"
		var has_marker := func() -> bool:
			for c in app.timeline_row.get_children():
				if c is Label and String(c.text) == marker:
					return true
			return false
		_ok(has_marker.call(), "precondition: timeline_row shows the planner's own '%s'" % marker)
		app.call("select_domain", "civilization")
		await _frames(5)
		_ok(has_marker.call(), "timeline_row STILL shows it after bare select_domain re-entry (not rebuilt to the standard strip)")
		app.call("select_domain_mode", "civilization", "planner")
		await _frames(5)
		_ok(has_marker.call(), "timeline_row STILL shows it after select_domain_mode re-entry")

	app.call("arm_tool", "inspect")
	await _frames(3)

	# ============================================================ Part 6 ====
	print("\n=== Part 6: Row B -- the four previously-swallowed destinations ===")
	var destinations := ["Landmarks", "Factions", "Military", "Timeline"]
	for category in destinations:
		app.call("open_journey_planner")
		await _frames(10)
		var pre_ok: bool = _jp_left_visible(jp) and _jp_center_visible(jp) and not _civ_visible(app)
		_ok(pre_ok, "precondition before -> %s: planner showing, civ hidden" % category)
		app.call("select_domain_category", "civilization", category)
		await _frames(10)
		_ok(not _jp_left_visible(jp) and not _jp_center_visible(jp),
			"planner panels HIDDEN after navigating to %s while journey was last-armed" % category)
		_ok(_civ_visible(app), "civ panel VISIBLE after navigating to %s" % category)
		print("    -> %s landed on mode '%s'" % [category, str(app.call("active_mode", "civilization"))])

	# ============================================================ Cleanup ====
	app.call("_set_sheet_open", "left", false)
	app.call("_set_sheet_open", "right", false)
	app.call("select_domain_mode", dom0, mode0)
	app.call("arm_tool", tool0)
	await _frames(10)
	if str(app.call("active_domain")) != dom0:
		_fail += 1
		print("  FAIL restore: domain left as ", app.call("active_domain"), " not ", dom0)
	if str(app.get("armed_tool")) != tool0:
		_fail += 1
		print("  FAIL restore: armed tool left as ", app.get("armed_tool"), " not ", tool0)

	print("\n%d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
