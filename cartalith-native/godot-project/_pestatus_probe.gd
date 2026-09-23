extends Node
## Verifies SP-3's ruins/fortified column on `place_editor_window.gd`'s
## Political history tab: a REAL collapse run on a REAL generated world writes
## `TimelineSnapshot::collapse_flags`, `civ_settlement_population_trajectory`
## carries them as `"ruins"`/`"fortified"` keys (omitted for a year no run
## wrote), and the "Population & tier trajectory" list draws one status label
## per recorded year from them.
##
## Windowed, not `--headless`: presses a real tab `Button` and reads drawn
## `Label` text, same reasoning as `_peptraj_probe.gd`.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pestatus_probe.tscn
##
## Exit 0 = every check passed. Exit 1 = a real failure. Exit 2 = the probe
## could not run (no settlement, no binding, or the run demoted nothing, so
## there is no status change to check) -- distinct from a finding.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("PESTATUS  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _labels_text(n: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_walk(n, all)
	for c in all:
		if c is Label:
			out.append((c as Label).text)
	return out


func _find_buttons_by_text(n: Node, text: String) -> Array:
	var out: Array = []
	var all: Array = []
	_walk(n, all)
	for c in all:
		if c is Button and (c as Button).text == text:
			out.append(c)
	return out


## The oracle for one point's drawn status, restated from the bridge's key
## contract (not read back out of `place_editor_window.gd`).
func _status_of(pd: Dictionary) -> String:
	if not pd.has("ruins"):
		return "—"
	var parts: Array[String] = []
	if bool(pd["ruins"]):
		parts.append("ruins")
	if bool(pd["fortified"]):
		parts.append("fortified")
	return " · ".join(parts) if not parts.is_empty() else "standing"


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge

	_bridge.generate({
		"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var places: Array = _bridge.settlements()
	if places.is_empty():
		_p("ABORT: no settlement generated")
		get_tree().quit(2)
		return

	# Year 0 from live state (no flags), then a conflict collapse: ten 10-year
	# steps, each written with its step's flags. Severity is SEARCHED, gentlest
	# first, because the change this probe needs (a step year recorded
	# "standing", a later one "ruins") only exists when an exchange-tier town
	# survives the first step undemoted. Measured on this seed: at 0.9 and 0.5
	# every exchange tier demotes in step one, so every ruined settlement reads
	# "ruins" from year 10 on and nothing is ever seen changing.
	_bridge.civ_add_year(0)
	var pick_idx := -1
	var pick_traj: Array = []
	var any_flagged := 0
	for sev in [0.05, 0.1, 0.2, 0.3]:
		var res: Dictionary = _bridge.civ_run_collapse_simulation({
			"mode": "collapse", "character": "conflict", "severity": sev,
			"start_year": 0, "duration": 100, "step_years": 10, "confirm_overwrite": true,
		})
		_p("severity %.2f: civ_run_collapse_simulation = %s" % [sev, str(res)])
		_check("the collapse run at severity %.2f ran" % sev, bool(res.get("ok", false)), str(res))
		# Find a settlement whose status CHANGES across recorded years: some
		# step year recorded "standing", a later one "ruins · fortified".
		any_flagged = 0
		for i in places.size():
			var tid := int((places[i] as Dictionary).get("tid", 0))
			var traj: Array = _bridge.civ_settlement_population_trajectory(tid)
			var saw_standing := false
			for pt in traj:
				var st := _status_of(pt)
				if st != "—":
					any_flagged += 1
				if st == "standing":
					saw_standing = true
				elif saw_standing and st.begins_with("ruins"):
					pick_idx = i
					break
			if pick_idx == i:
				pick_traj = traj
				break
		if pick_idx >= 0:
			break
	_p("points carrying flags (settlements scanned up to the pick): %d" % any_flagged)
	if pick_idx < 0:
		# Diagnostic only: what the ruined settlements' status sequences looked like.
		var shown := 0
		for i in places.size():
			var traj: Array = _bridge.civ_settlement_population_trajectory(int((places[i] as Dictionary).get("tid", 0)))
			var seq: Array = []
			for pt in traj:
				seq.append("%d:%s:%s" % [int(pt.get("year", 0)), String(pt.get("kind", "")), _status_of(pt)])
			if " ".join(seq).contains("ruins") and shown < 6:
				_p("  seq %s" % " ".join(seq))
				shown += 1
	_check("at least one trajectory point carries recorded flags", any_flagged > 0)
	if pick_idx < 0:
		_p("ABORT: no settlement went standing -> ruins in this run; nothing to show changing")
		get_tree().quit(2 if _fail == 0 else 1)
		return

	var s0: Dictionary = places[pick_idx]
	_p("picked idx=%d tid=%d name='%s'" % [pick_idx, int(s0.get("tid", 0)), String(s0.get("name", "?"))])
	_p("trajectory = %s" % str(pick_traj))

	var p0: Dictionary = pick_traj[0]
	_check("year 0 (written from live state) omits the ruins/fortified keys",
		int(p0.get("year", -1)) == 0 and not p0.has("ruins") and not p0.has("fortified"), str(p0))
	var step_years_all_flagged := true
	for pt in pick_traj:
		var pd: Dictionary = pt
		if int(pd.get("year", 0)) > 0 and not (pd.has("ruins") and pd.has("fortified")):
			step_years_all_flagged = false
	_check("every step year the settlement survived carries both keys", step_years_all_flagged)
	# `fortified` is never cleared once set (CollapsePlace's own doc comment).
	var fort_seen := false
	var fort_monotone := true
	for pt in pick_traj:
		var pd: Dictionary = pt
		if not pd.has("fortified"):
			continue
		if fort_seen and not bool(pd["fortified"]):
			fort_monotone = false
		fort_seen = fort_seen or bool(pd["fortified"])
	_check("fortified never clears once set across the stored years", fort_monotone)

	# -- Real UI --------------------------------------------------------------
	var pe = _app.place_editor_window
	pe.open_for(pick_idx)
	await _frames(8)
	var pol_btns := _find_buttons_by_text(pe, "Political history")
	if pol_btns.is_empty():
		_p("ABORT: cannot reach the Political history tab")
		get_tree().quit(2)
		return
	(pol_btns[0] as Button).pressed.emit()
	await _frames(6)
	_check("switched to Political history", pe._active_tab == "political")

	var labels := _labels_text(pe)
	var want := {}
	for pt in pick_traj:
		var st := _status_of(pt)
		want[st] = int(want.get(st, 0)) + 1
	for st in want:
		var drawn := labels.count(st)
		_check("status '%s' drawn once per recorded year that has it" % st, drawn == int(want[st]),
			"want %d, drawn %d" % [int(want[st]), drawn])
	_check("the not-recorded note is drawn (year 0 has no flags)",
		labels.any(func(t): return String(t).begins_with("— in the last column")))

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
