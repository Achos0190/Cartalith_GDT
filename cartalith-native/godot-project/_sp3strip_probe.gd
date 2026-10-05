extends Node
## SP-3's combined timeline strip (`OUTSTANDING_WORK.md`, Ruling AQ): ONE strip
## over a settlement's recorded years, four lanes interleaved by date (ownership,
## population/tier, authored `chronos` events, journey passes), on the Settlement
## Editor's Political history tab, with the existing lists kept beneath it.
##
## Protects: (1) the number of marks in each lane equals what the underlying
## bridge calls return -- the oracle is read from the same four calls the shell
## reads, never hard-coded; (2) a settlement whose lane has no data draws NO mark
## in it (no fake mark for a year with nothing recorded); (3) marks are placed on
## one shared year axis (x is monotonic in year across all four lanes) and never
## overlap within a lane; (4) a click on a mark fills the readout line (the phone's
## stand-in for a hover tooltip); (5) the strip is real pixels (a palette-agnostic
## distinct-colour count over its rect), and (6) on a phone the page does not
## overflow horizontally -- the plot scrolls inside its own container instead.
##
## WINDOWED (reads drawn nodes and the framebuffer; `ImageTexture` is a no-op
## headless):
##   desktop: Godot_v4.7.1-stable_win64_console.exe --path . _sp3strip_probe.tscn
##   phone:   ... --path . _sp3strip_probe.tscn -- --force-touch --sp3strip-phone
## Prints SP3S ... FAIL lines; exits 1 on any failure (grep "FAIL").

var _app: Node
var _bridge
var _fail := 0
var _phone_run := false
var _tag := "desktop"


func _p(s: String) -> void:
	print("SP3S  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _walk(n: Node, out: Array) -> Array:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)
	return out


func _labels(n: Node) -> Array:
	var out: Array = []
	for c in _walk(n, []):
		if c is Label:
			out.append((c as Label).text)
	return out


func _press_tab(pe: Node, text: String) -> void:
	for n in _walk(pe, []):
		if n is Button and (n as Button).text == text:
			(n as Button).pressed.emit()
			return


## Every strip mark under `pe`, as `{lane: [Control, ...]}`.
func _marks(pe: Node) -> Dictionary:
	var by := {"ownership": [], "population": [], "events": [], "journeys": []}
	for n in _walk(pe, []):
		if n is Control and (n as Control).has_meta("strip_lane"):
			(by[String((n as Control).get_meta("strip_lane"))] as Array).append(n)
	return by


func _pass_for(tid: int, jid: int) -> Dictionary:
	for d in _bridge.civ_settlement_journey_passes(tid):
		if int(d.get("id", -1)) == jid:
			return d
	return {}


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 280.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_phone_run = "--sp3strip-phone" in OS.get_cmdline_user_args()
	_tag = "phone" if _phone_run else "desktop"
	var want := Vector2i(1080, 2400) if _phone_run else Vector2i(1360, 940)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.4).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(4)
	_bridge = _app.bridge
	_p("=== %s run: is_phone=%s screen=%s ===" % [_tag, _app.is_phone(), _app.get_viewport_rect().size])
	if _phone_run and not _app.is_phone():
		_p("FAIL  NOT PHONE -- the resize did not register (--force-touch missing?); nothing asserted")
		get_tree().quit(1)
		return
	if not _phone_run and _app.is_phone():
		_p("FAIL  desktop run came up as a phone")
		get_tree().quit(1)
		return
	_p("dll mtime: %s" % FileAccess.get_modified_time(
		ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))
	if not _bridge.world_gen.has_method("civ_settlement_journey_passes"):
		_p("FAIL  ABORT: stale .dll")
		get_tree().quit(1)
		return

	_app._run_pipeline()   ## the default world, as `_sp3journeypass_probe.gd` does -- a journey long enough to plan needs its reach
	var waited := 0
	while _bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not _bridge.has_world:
		_p("FAIL  generate failed")
		get_tree().quit(1)
		return
	var towns: Array = _bridge.settlements()
	var pe = _app.place_editor_window

	# -- Control: nothing recorded -> no marks, a stated empty state ----------------
	pe.open_for(0)
	await _frames(6)
	_press_tab(pe, "Political history")
	await _frames(6)
	var m0 := _marks(pe)
	var total0 := 0
	for ln in m0:
		total0 += (m0[ln] as Array).size()
	_check("control: a world with no recorded year draws no mark at all", total0 == 0, str(total0))
	_check("control: ...and says so rather than showing an empty frame",
		_labels(pe).any(func(l): return String(l).begins_with("Nothing to plot yet")), "")

	# -- Fixture: a collapse run, a faction change, a journey, an authored event -----
	var gs: Vector2 = _bridge.grid_size()

	var presets: Array = _bridge.world_gen.tl_list("preset")
	var preset_id := String((presets[0] as Dictionary).get("id", "")) if not presets.is_empty() else ""
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var d := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if d < gs.x * 0.45:
				pairs.append([d, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var jid := -1
	## Departs in year 40, INSIDE the recorded 0..100 run, so the journey lane
	## shares the axis with the other three instead of stretching it.
	for k in mini(8, pairs.size()):
		var a: Dictionary = towns[pairs[k][1]]
		var b: Dictionary = towns[pairs[k][2]]
		_bridge.route_begin("mixed")
		_bridge.route_append_stop(float(a["x"]), float(a["y"]))
		_bridge.route_append_stop(float(b["x"]), float(b["y"]))
		var ridx: int = _bridge.route_commit()
		jid = _bridge.journey_save("Probe caravan", preset_id, ridx, 40)
		var ok := false
		for d in _bridge.journey_positions():
			if int(d.get("id", -1)) == jid and d.has("x") and float(d["total_days"]) >= 8.0:
				ok = true
		if ok:
			break
		_bridge.journey_delete(jid)
		jid = -1
	_check("a dated journey saved on a real route", jid >= 0, "towns=%d pairs=%d" % [towns.size(), pairs.size()])
	if jid < 0:
		get_tree().quit(1)
		return

	## The journey is saved FIRST, then the run: the run re-places settlements, so
	## `towns` is re-read after it (a pre-run list holds ids the run may have
	## removed), and a pass is then asked of the settlements that survived.
	_bridge.civ_add_year(0)
	var sim: Dictionary = _bridge.civ_run_collapse_simulation({
		"mode": "collapse", "character": "conflict", "severity": 0.2,
		"start_year": 0, "duration": 100, "step_years": 10, "confirm_overwrite": true})
	_check("a collapse run recorded years", bool(sim.get("ok", false)), str(sim))
	towns = _bridge.settlements()

	## The settlement under test: passed by the journey, with the longest
	## recorded trajectory (so population has several points).
	var idx := -1
	var best := -1
	var far_idx := -1
	for i in towns.size():
		var tid_i := int((towns[i] as Dictionary).get("tid", 0))
		var n_pts: int = _bridge.civ_settlement_population_trajectory(tid_i).size()
		if not _pass_for(tid_i, jid).is_empty() and n_pts > best:
			best = n_pts
			idx = i
		elif _pass_for(tid_i, jid).is_empty() and far_idx == -1 and n_pts > 0:
			far_idx = i
	_check("a passed settlement with recorded years exists", idx >= 0 and best > 0, "best=%d" % best)
	if idx < 0:
		get_tree().quit(1)
		return
	var tid := int((towns[idx] as Dictionary).get("tid", 0))
	var tname := String((towns[idx] as Dictionary).get("name", "?"))

	## A faction change AFTER the run: `civ_add_year` re-saves the cursor's year
	## from live state, then records a new one (`_pepolitical_probe.gd`'s rule).
	var factions: Array = _bridge.get_factions()
	var orig_f := int((towns[idx] as Dictionary).get("faction", 1))
	var new_f := -1
	for f in factions:
		if int((f as Dictionary).get("id", -1)) != orig_f:
			new_f = int((f as Dictionary).get("id", -1))
			break
	if new_f != -1:
		_bridge.civ_edit_settlement(idx, {"faction": new_f})
		_bridge.civ_add_year(110)

	DirAccess.make_dir_recursive_absolute("user://_sp3strip_vault/Settlements")
	var note := "# %s\n\nA river town.\n\n```chronos\n" % tname \
		+ "- [20~60] #blue {Rule} Ashfall regency | the regent's peace\n" \
		+ "- [75] #red Siege | the walls held\n" \
		+ "* [50-06-01] Charter granted\n" \
		+ "```\n"
	var f := FileAccess.open("user://_sp3strip_vault/Settlements/Note.md", FileAccess.WRITE)
	f.store_string(note)
	f.close()
	var info: Dictionary = _bridge.vault_connect(
		ProjectSettings.globalize_path("user://_sp3strip_vault"), "Probe vault")
	_check("vault connected", bool(info.get("ok", false)), String(info.get("error", "")))
	var att: Dictionary = _bridge.vault_attach("settlement", tid, tname, "Settlements/Note.md", "")
	_check("note attached", bool(att.get("ok", false)), String(att.get("error", "")))

	# -- The oracle: the same four bridge calls the shell reads -----------------------
	var periods: Array = _bridge.civ_settlement_ownership_periods(tid)
	var traj: Array = _bridge.civ_settlement_population_trajectory(tid)
	var evs: Array = _bridge.vault_entity_chronos("settlement", tid).get("events", [])
	var passes: Array = _bridge.civ_settlement_journey_passes(tid)
	var dated := 0
	for ps in passes:
		if (ps as Dictionary).has("year"):
			dated += 1
	var ruined := 0
	for pt in traj:
		if bool((pt as Dictionary).get("ruins", false)):
			ruined += 1
	_p("%s (tid %d): periods=%d trajectory=%d (ruined years %d) events=%d passes=%d (dated %d)" % [
		tname, tid, periods.size(), traj.size(), ruined, evs.size(), passes.size(), dated])
	_check("fixture: every lane has data on this settlement",
		periods.size() >= 1 and traj.size() >= 3 and evs.size() == 3 and dated >= 1,
		"%d/%d/%d/%d" % [periods.size(), traj.size(), evs.size(), dated])
	_check("fixture: ownership has two spans (a faction change)", periods.size() >= 2, str(periods.size()))

	# -- Drawn ------------------------------------------------------------------------
	pe.open_for(idx)
	await _frames(6)
	_press_tab(pe, "Political history")
	await _frames(8)
	var marks := _marks(pe)
	_check("ownership marks == periods the bridge returns", (marks["ownership"] as Array).size() == periods.size(),
		"%d vs %d" % [(marks["ownership"] as Array).size(), periods.size()])
	_check("population marks == recorded years the bridge returns", (marks["population"] as Array).size() == traj.size(),
		"%d vs %d" % [(marks["population"] as Array).size(), traj.size()])
	_check("event marks == authored events the bridge returns", (marks["events"] as Array).size() == evs.size(),
		"%d vs %d" % [(marks["events"] as Array).size(), evs.size()])
	_check("journey marks == DATED passes the bridge returns", (marks["journeys"] as Array).size() == dated,
		"%d vs %d" % [(marks["journeys"] as Array).size(), dated])
	_p("marks drawn: ownership %d, population %d, events %d, journeys %d" % [
		(marks["ownership"] as Array).size(), (marks["population"] as Array).size(),
		(marks["events"] as Array).size(), (marks["journeys"] as Array).size()])

	# Marks carry the year the data says (read back from meta, not the position).
	var years_ok := true
	for pt in traj:
		var y := int((pt as Dictionary).get("year", 0))
		if not (marks["population"] as Array).any(func(m): return int((m as Control).get_meta("strip_year")) == y):
			years_ok = false
	_check("every recorded year has a population mark at that year", years_ok, "")

	# One shared axis: anchor x is monotonic in year across ALL lanes together.
	var anchors: Array = []   ## [year, x]
	for ln in marks:
		for m in marks[ln]:
			var c: Control = m
			var ax := c.position.x if ln == "ownership" else c.position.x + 8.0 * (3.5 if _phone_run else 1.0) \
				if false else c.position.x + (14.0 if _phone_run else 8.0)
			anchors.append([int(c.get_meta("strip_year")), ax])
	anchors.sort_custom(func(u, v): return u[0] < v[0] or (u[0] == v[0] and u[1] < v[1]))
	var mono := true
	for i in range(1, anchors.size()):
		if int(anchors[i][0]) > int(anchors[i - 1][0]) and float(anchors[i][1]) <= float(anchors[i - 1][1]) + 0.01:
			mono = false
	_check("all four lanes share one year axis (x strictly rises with year)", mono and anchors.size() > 4, str(anchors.size()))

	# No overlap inside a lane; lanes occupy distinct vertical bands in the stated order.
	var overlap := 0
	var band: Array = []
	for ln in ["ownership", "population", "events", "journeys"]:
		var ms: Array = marks[ln]
		var top := 1e9
		var bot := -1e9
		for i in ms.size():
			var ri := Rect2((ms[i] as Control).position, (ms[i] as Control).size)
			top = minf(top, ri.position.y)
			bot = maxf(bot, ri.end.y)
			for j in range(i + 1, ms.size()):
				if ri.intersects(Rect2((ms[j] as Control).position, (ms[j] as Control).size)):
					overlap += 1
		band.append([top, bot])
	_check("no two marks in one lane overlap", overlap == 0, str(overlap))
	var bands_ok := true
	for i in range(1, band.size()):
		if float(band[i][0]) < float(band[i - 1][1]) - 0.01:
			bands_ok = false
	_check("lanes are stacked in order, one band each (ownership above population above events above journeys)",
		bands_ok, str(band))

	# Lane titles drawn, distinct shapes named in them.
	var labs := _labels(pe)
	for t in ["▬ OWNERSHIP", "● POP · TIER", "◆ EVENTS", "▲ JOURNEYS"]:
		_check("lane title '%s' is drawn" % t, labs.has(t), "")

	# The existing lists are still beneath it.
	## `DccWidgets.group` draws its header as a collapsible Button, not a Label, so
	## the ownership list is found by its header Button plus its own period rows.
	var has_owner_hdr := false
	for n in _walk(pe, []):
		if n is Button and String((n as Button).text).to_upper().ends_with("OWNERSHIP PERIODS"):
			has_owner_hdr = true
	_check("the ownership period list is still drawn (header + its range rows)",
		has_owner_hdr and labs.has("%d – %d" % [int(periods[0]["start_year"]), int(periods[0]["end_year"])]), "")
	_check("the population list is still drawn",
		labs.any(func(l): return String(l).to_upper().ends_with("POPULATION & TIER TRAJECTORY")), "")
	_check("the authored-events section is still drawn",
		labs.any(func(l): return String(l).to_upper().ends_with("AUTHORED EVENTS")), "")
	_check("the journeys-passing list is still drawn and holds the pass date",
		labs.has(String(_pass_for(tid, jid).get("date", "#"))) and labs.has("Probe caravan"), "")

	# A click fills the readout (the touch stand-in for a hover tooltip).
	var any_mark: Control = (marks["events"] as Array)[0]
	var readout_before := labs.has("Hover or tap a mark for its detail.")
	_check("readout line starts with its prompt", readout_before, "")
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	ev.position = any_mark.size * 0.5
	any_mark.gui_input.emit(ev)
	await _frames(2)
	var tip := String(any_mark.get_meta("strip_text"))
	_check("clicking a mark puts its detail in the readout", _labels(pe).has(tip) and tip != "", tip)

	# Real pixels under the strip.
	await _frames(4)
	var img := get_viewport().get_texture().get_image()
	var plot: Control = any_mark.get_parent()
	var scroll: Control = plot.get_parent()
	var gr := scroll.get_global_rect()
	var seen := {}
	var step := 2
	var x0 := maxi(0, int(gr.position.x))
	var y0 := maxi(0, int(gr.position.y))
	var x1 := mini(img.get_width(), int(gr.end.x))
	var y1 := mini(img.get_height(), int(gr.end.y))
	for yy in range(y0, y1, step):
		for xx in range(x0, x1, step):
			seen[img.get_pixel(xx, yy).to_html(false)] = true
	_check("the strip's rect holds drawn pixels (>= 6 distinct colours)", seen.size() >= 6,
		"%d colours in %dx%d at (%d,%d)" % [seen.size(), x1 - x0, y1 - y0, x0, y0])
	var shot := "user://_sp3strip_%s.png" % _tag
	img.save_png(shot)
	_p("screenshot (%s): %s" % [_tag, ProjectSettings.globalize_path(shot)])
	var sb: HScrollBar = (scroll as ScrollContainer).get_h_scroll_bar()
	_p("strip scroll: viewport %.0f px, content %.0f px, h-scroll max %.0f" % [
		scroll.size.x, plot.custom_minimum_size.x, sb.max_value])

	# Phone: the page itself must not overflow; the plot scrolls inside its own container.
	if _phone_run:
		var body: Control = pe._body
		var min_x := body.get_combined_minimum_size().x
		_check("phone: the tab body fits the window width (no page-level horizontal overflow)",
			min_x <= pe.size.x + 0.5, "body min %.1f vs window %d" % [min_x, pe.size.x])
		_check("phone: the plot is wider than its viewport and scrolls inside the strip",
			plot.custom_minimum_size.x > scroll.size.x and sb.max_value > scroll.size.x,
			"%.0f > %.0f" % [plot.custom_minimum_size.x, scroll.size.x])
		var shown := Rect2(Vector2.ZERO, Vector2(get_viewport().get_visible_rect().size))
		_check("phone: the strip lies inside the screen", shown.encloses(gr), str(gr))

	# -- A settlement the journey does not pass / with no events: empty lanes, no marks --
	if far_idx >= 0:
		pe.open_for(far_idx)
		await _frames(6)
		_press_tab(pe, "Political history")
		await _frames(8)
		var ftid := int((towns[far_idx] as Dictionary).get("tid", 0))
		var fm := _marks(pe)
		_check("a settlement the journey never passes draws NO journey mark",
			(fm["journeys"] as Array).is_empty() and _pass_for(ftid, jid).is_empty(), "")
		_check("...nor an event mark (its note has no chronos block)", (fm["events"] as Array).is_empty()
			and _bridge.vault_entity_chronos("settlement", ftid).get("events", []).is_empty(), "")
		_check("...but still draws its own recorded years",
			(fm["population"] as Array).size() == _bridge.civ_settlement_population_trajectory(ftid).size()
			and (fm["population"] as Array).size() > 0, "")
		img = get_viewport().get_texture().get_image()
		img.save_png("user://_sp3strip_%s_sparse.png" % _tag)
		_p("screenshot (%s sparse): %s" % [_tag, ProjectSettings.globalize_path("user://_sp3strip_%s_sparse.png" % _tag)])

	_bridge.journey_delete(jid)
	_bridge.vault_disconnect()
	_p("DONE %s: %d failure(s) (PROBE-FAIL count %d)" % [_tag, _fail, _fail])
	if _fail > 0:
		print("PROBE-FAIL %d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
