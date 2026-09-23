extends Node
## Verifies `place_editor_window.gd`'s Political history tab, population/tier
## trajectory sub-section (`OUTSTANDING_WORK.md` §2.3 SP-3, the small/
## additive/shovel-ready piece): the new
## `civ_settlement_population_trajectory` bridge call, and the plain
## per-recorded-year list `_build_political` renders from it.
##
## Windowed, not `--headless`: this presses real `Button` controls inside a
## popup `AcceptDialog` and reads real drawn `Label` text -- same reasoning
## `_pepolitical_probe.gd` (this tab's own ownership-periods probe) already
## states.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _peptraj_probe.tscn
##
## Exit 0 = every check passed. Exit 1 = a real failure. Exit 2 = the probe
## itself could not run (no settlement generated, or the new #[func] is
## absent from this binary) -- distinct from a real finding, same convention
## `_pepolitical_probe.gd` uses.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("PEPTRAJ  %s" % s)


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


func _thousands(n: int) -> String:
	# Same formatter `place_editor_window.gd` reuses from
	# `FactionRosterWindow._thousands` -- re-stated here rather than
	# reaching into another window's static, so this probe's oracle does not
	# depend on that window loading.
	var s := str(absi(n))
	var out := ""
	var c := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		c += 1
		if c % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if n < 0 else "") + out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge

	if not _bridge.has_method("civ_settlement_population_trajectory"):
		_p("ABORT: this binary has no civ_settlement_population_trajectory #[func] -- nothing here can run")
		get_tree().quit(2)
		return

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
		_p("ABORT: no settlement generated -- nothing here can run")
		get_tree().quit(2)
		return

	var idx := 0
	var s0: Dictionary = places[idx]
	var tid := int(s0.get("tid", 0))
	var orig_pop := int(s0.get("population", s0.get("pop", 0)))
	_p("settlement idx=%d tid=%d name='%s' pop=%d" % [idx, tid, String(s0.get("name", "?")), orig_pop])

	# -- Build a real multi-year timeline with both a tier change AND a
	#    population change that survives the tier change -- the shape the
	#    unit tests (`timeline.rs`'s own
	#    population_trajectory_returns_every_recorded_year_flat_not_collapsed_into_spans`)
	#    already cover synthetically; this drives the same fixture through
	#    the real engine + real UI. Same `civ_add_year` staging as
	#    `_pepolitical_probe.gd`: an edit only lands in a recorded snapshot
	#    on the NEXT `civ_add_year` call once the cursor sits on that year.
	var pop_50 := orig_pop + 500
	var pop_100 := pop_50 + 700
	_bridge.civ_add_year(0)          # year 0 = live (original pop/kind), cursor -> 0
	_bridge.civ_add_year(50)         # year 50 = clone of year 0, cursor -> 50
	_check("civ_edit_settlement accepted the population+kind change",
		_bridge.civ_edit_settlement(idx, {"population": pop_50, "kind": "town"}))
	_bridge.civ_add_year(100)        # re-saves year 50 from LIVE (edited), cursor -> 100
	_check("civ_edit_settlement accepted the second population change",
		_bridge.civ_edit_settlement(idx, {"population": pop_100}))
	_bridge.civ_add_year(150)        # re-saves year 100 from LIVE (edited), cursor -> 150

	# -- Direct bridge call: this is the oracle the rendered tab is checked
	#    against below, not a second guess at what it "should" say. --------
	var expected: Array = _bridge.civ_settlement_population_trajectory(tid)
	_p("civ_settlement_population_trajectory(%d) = %s" % [tid, str(expected)])
	# 4 points, not 3: the final `civ_add_year(150)` re-saves year 100's
	# (unedited-since) state into year 150 too -- a real, distinct recorded
	# year, not a duplicate to be collapsed. Same "every recorded year
	# survives" contract `population_trajectory_returns_every_recorded_year_
	# flat_not_collapsed_into_spans` proves synthetically.
	_check("bridge returns exactly four points (years 0, 50, 100, 150)", expected.size() == 4,
		"got %d" % expected.size())
	if expected.size() == 4:
		var p0: Dictionary = expected[0]
		var p1: Dictionary = expected[1]
		var p2: Dictionary = expected[2]
		var p3: Dictionary = expected[3]
		_check("point 0 is year 0 at the original population", int(p0.get("year", -1)) == 0
			and int(p0.get("pop", -1)) == orig_pop)
		_check("point 1 is year 50 at the FIRST changed population and the NEW tier",
			int(p1.get("year", -1)) == 50 and int(p1.get("pop", -1)) == pop_50
			and String(p1.get("kind", "")) == "town")
		_check("point 2 is year 100 at the SECOND changed population, tier unchanged",
			int(p2.get("year", -1)) == 100 and int(p2.get("pop", -1)) == pop_100
			and String(p2.get("kind", "")) == "town")
		_check("point 3 is year 150, unedited since year 100 (same pop/tier carried through)",
			int(p3.get("year", -1)) == 150 and int(p3.get("pop", -1)) == pop_100
			and String(p3.get("kind", "")) == "town")

	# -- Now drive the real UI and cross-check what it draws -----------------
	var pe = _app.place_editor_window
	pe.open_for(idx)
	await _frames(8)

	var pol_btns := _find_buttons_by_text(pe, "Political history")
	_check("found the Political history tab button", not pol_btns.is_empty())
	if pol_btns.is_empty():
		_p("ABORT: cannot reach the Political history tab")
		get_tree().quit(2)
		return
	(pol_btns[0] as Button).pressed.emit()
	await _frames(6)
	_check("switched to Political history", pe._active_tab == "political")

	var labels := _labels_text(pe)

	# `DccWidgets.section()` draws its title via `DccTheme.header()`, which
	# upper-cases it and prefixes the `§` sigil -- the same convention every
	# other section title in this window uses.
	_check("section header '§ POPULATION & TIER TRAJECTORY' is drawn on screen",
		labels.has("§ POPULATION & TIER TRAJECTORY"))

	for p in expected:
		var pd: Dictionary = p
		var year_str := str(int(pd.get("year", 0)))
		var pop_str := _thousands(int(pd.get("pop", 0)))
		var kind_str := String(pd.get("kind", "")).capitalize()
		_check("trajectory row year '%s' is drawn on screen" % year_str, labels.has(year_str))
		_check("trajectory row population '%s' is drawn on screen" % pop_str, labels.has(pop_str))
		_check("trajectory row tier '%s' is drawn on screen" % kind_str, labels.has(kind_str))

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
