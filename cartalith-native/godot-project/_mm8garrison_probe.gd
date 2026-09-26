extends Node
## Ruling AW -- MM-6 (derived garrisons) and MM-8 (manpower across the year
## cursor), end to end through the real shell (`MILITARY_MANPOWER_SCOPE.md`
## §5.6-5.7):
##
##   - generate; record year y0 from the live world; move to y0+10, triple one
##     faction-f1 town's population and paint a faction-f2 dab over f1's land,
##     and record y0+10;
##   - `civ_military_summary_at` reads different, LITERAL standing armies and
##     garrisons at the two recorded years; an unrecorded year reads the year
##     in force (y0+5 == y0, y0+15 == y0+10); a year before every record has
##     no reading (`none_in_force`, empty, never zeros);
##   - every faction's garrisons sum to its rounded standing army exactly, at
##     both years; `civ_settlement_garrison` agrees with the summary row;
##   - SHELL: CIVIL ▸ Military's reading line names the year in force, and a
##     cursor move across a record refills it (and a move inside one does
##     not); the right dock's Settlement section draws a Garrison row;
##   - the Conflict layer's siege row carries the besieged town's garrison.
##
## Windowed (the shell path), per `MISTAKES.md`:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _mm8garrison_probe.tscn
##
## Exit 0 = all passed, 1 = a real failure, 2 = could not run.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("MM  %s" % s)


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


func _standing(d: Dictionary) -> Dictionary:
	var out := {}
	for f: Dictionary in d.get("factions", []):
		out[int(f["faction"])] = int(round(float((f.get("manpower", {}) as Dictionary).get("standing_army", -1.0))))
	return out


func _garrison_sums(d: Dictionary) -> Dictionary:
	var out := {}
	for s: Dictionary in d.get("settlements", []):
		if s.has("garrison"):
			var f := int(s["faction"])
			out[f] = int(out.get(f, 0)) + int(s["garrison"])
	return out


func _garrison_of(d: Dictionary, tid: int) -> int:
	for s: Dictionary in d.get("settlements", []):
		if int(s["tid"]) == tid:
			return int(s["garrison"]) if s.has("garrison") else -1
	return -2


func _labels_text(root: Node) -> String:
	var parts := PackedStringArray()
	for n in _walk(root, []):
		if n is Label:
			parts.append((n as Label).text)
		elif n is Button:
			parts.append((n as Button).text)
	return "\n".join(parts)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 400.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	for m in ["civ_military_summary_at", "civ_settlement_garrison", "civ_year_in_force"]:
		if not _bridge.world_gen.has_method(m):
			_p("ABORT: this binary has no %s #[func] -- cargo build -p cartalith-godot" % m)
			get_tree().quit(2)
			return
	_p("display: %s" % DisplayServer.get_name())

	_bridge.generate({"seed": 5521, "width_km": 400.0, "grid_w": 320, "grid_h": 240,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_app.select_domain("civilization")
	await _frames(8)

	# -- 0. Before any record: the live world, garrisons already exact.
	var live: Dictionary = _bridge.civ_military_summary_at(0)
	_check("an empty timeline reads the live world", String(live.get("reading", "")) == "live"
		and not live.has("year_in_force"), str(live.get("reading")))
	var live_plain: Dictionary = _bridge.civ_military_summary()
	_check("summary_at on an empty timeline == the live summary's standing armies",
		_standing(live) == _standing(live_plain))

	var places: Array = _bridge.settlements()
	var g0: Vector2i = _bridge.grid_size()
	var s0: Dictionary = {}
	var i0 := -1
	for i in places.size():
		var pl: Dictionary = places[i]
		if int(pl.get("faction", 0)) <= 0:
			continue
		## The most populous non-capital: a real share of its faction's army.
		if bool(pl.get("capital", false)):
			continue
		if s0.is_empty() or int(pl.get("population", 0)) > int(s0.get("population", 0)):
			s0 = pl
			i0 = i
	var tid := int(s0["tid"])
	var f1 := int(s0["faction"])
	var f2 := -1
	for f: Dictionary in _bridge.get_factions():
		if int(f.get("id", 0)) > 0 and int(f.get("id", 0)) != f1:
			f2 = int(f["id"])
			break
	_p("town tid=%d idx=%d '%s' pop=%d faction %d; other %d" % [tid, i0, String(s0.get("name", "")),
		int(s0.get("population", 0)), f1, f2])

	# -- 1. Two recorded years that differ in settlements AND territory.
	var y0: int = _bridge.get_civ_year()
	_bridge.civ_add_year(y0)
	_bridge.civ_add_year(y0 + 10)
	var pop0 := int(s0.get("population", 0))
	var edited: bool = _bridge.civ_edit_settlement(i0, {"population": pop0 * 3})
	var staged: bool = _bridge.civ_territory_paint_at(float(s0["x"]) + 4.0, float(s0["y"]), f2, 6.0, false)
	var committed: bool = _bridge.civ_territory_commit()
	_bridge.civ_add_year(y0 + 10)
	_check("town pop tripled and a dab committed, both recorded at y0+10", edited and staged and committed,
		"edit=%s stage=%s commit=%s" % [edited, staged, committed])

	var a: Dictionary = _bridge.civ_military_summary_at(y0)
	var b: Dictionary = _bridge.civ_military_summary_at(y0 + 10)
	var a_mid: Dictionary = _bridge.civ_military_summary_at(y0 + 5)
	var b_after: Dictionary = _bridge.civ_military_summary_at(y0 + 15)
	var before: Dictionary = _bridge.civ_military_summary_at(y0 - 5)
	var sa := _standing(a)
	var sb := _standing(b)
	_p("  y0=%d standing %s" % [y0, str(sa)])
	_p("  y0+10 standing %s" % str(sb))
	_p("  town garrison y0=%d y0+10=%d" % [_garrison_of(a, tid), _garrison_of(b, tid)])
	for d: Dictionary in [a, b]:
		for s: Dictionary in d.get("settlements", []):
			if int(s["tid"]) == tid:
				_p("  town row at %d: %s" % [int(d.get("year_in_force")), str(s)])
	_check("y0 reads as recorded", String(a.get("reading")) == "recorded" and int(a.get("year_in_force", -1)) == y0)
	_check("y0+10 reads as recorded", String(b.get("reading")) == "recorded" and int(b.get("year_in_force", -1)) == y0 + 10)
	_check("LITERAL: faction %d standing army at y0" % f1, int(sa.get(f1, -1)) == STANDING_F1_Y0, str(sa.get(f1)))
	_check("LITERAL: faction %d standing army at y0+10" % f1, int(sb.get(f1, -1)) == STANDING_F1_Y10, str(sb.get(f1)))
	_check("LITERAL: the town's garrison at y0", _garrison_of(a, tid) == GARRISON_TOWN_Y0, str(_garrison_of(a, tid)))
	_check("LITERAL: the town's garrison at y0+10", _garrison_of(b, tid) == GARRISON_TOWN_Y10, str(_garrison_of(b, tid)))
	_check("the two recorded years differ", sa != sb)
	_check("an unrecorded y0+5 reads y0 (the year in force)",
		int(a_mid.get("year_in_force", -1)) == y0 and _standing(a_mid) == sa)
	_check("an unrecorded y0+15 reads y0+10",
		int(b_after.get("year_in_force", -1)) == y0 + 10 and _standing(b_after) == sb)
	_check("a year before every record has no reading, and no rows (not zeros)",
		String(before.get("reading")) == "none_in_force" and int(before.get("earliest_year", -1)) == y0
			and (before.get("factions", []) as Array).is_empty())

	# -- 2. Exact sums, and the per-settlement call agrees with the summary.
	for d: Dictionary in [a, b]:
		var st := _standing(d)
		var gs := _garrison_sums(d)
		var bad := []
		for f in st:
			if gs.has(f) and int(gs[f]) != int(st[f]):
				bad.append("%d: %d vs %d" % [f, gs[f], st[f]])
		_check("every faction's garrisons sum to its rounded standing army (year %d)" % int(d.get("year_in_force")),
			bad.is_empty() and not gs.is_empty(), str(bad) + " sums=" + str(gs))
	var one: Dictionary = _bridge.civ_settlement_garrison(tid, y0 + 10)
	_check("civ_settlement_garrison agrees with the summary row",
		int(one.get("garrison", -9)) == _garrison_of(b, tid) and int(one.get("faction_garrison", -9)) == int(sb.get(f1, -8)),
		str(one))
	var none_one: Dictionary = _bridge.civ_settlement_garrison(tid, y0 - 5)
	_check("before every record the settlement call says why, with no figure",
		String(none_one.get("absent", "")) == "no_reading" and not none_one.has("garrison"), str(none_one))

	# -- 3. SHELL: CIVIL ▸ Military follows the cursor.
	var ws = _app.workspace_panel("civilization")
	_bridge.civ_goto_year(y0)
	_app.timeline_changed.emit()
	await _frames(4)
	var body: Control = ws._military_body
	var t0 := _labels_text(body)
	_check("Military names year y0 as the reading", t0.contains("Reading: year %d, as recorded." % y0))
	_check("Military draws a Garrisons section", t0.contains("GARRISONS") or t0.to_lower().contains("garrisons"))
	_check("Military's Not built no longer lists garrisons as unbuilt",
		not t0.contains("Per-settlement garrisons and manpower across the year cursor are scheduled"))
	var key0: String = ws._military_reading_key
	_bridge.civ_goto_year(y0 + 5)
	_app.timeline_changed.emit()
	await _frames(4)
	_check("a cursor step inside a record does not refill (same key)", ws._military_reading_key == key0)
	_check("and the old body is still the one drawn", _labels_text(ws._military_body) == t0)
	_bridge.civ_goto_year(y0 + 12)
	_app.timeline_changed.emit()
	await _frames(4)
	var t1 := _labels_text(ws._military_body)
	_check("a step across a record refills Military, naming y0+10 in force",
		t1.contains("Reading: year %d, the record in force at the cursor's %d." % [y0 + 10, y0 + 12]))

	# -- 4. SHELL: the right dock's Settlement section draws the Garrison row.
	var roster: Array = _bridge.settlements()
	_app.right_dock_ctrl.on_settlement_selected(roster[i0], i0)
	await _frames(4)
	var dock := _labels_text(_app.right_dock_body)
	_check("the right dock draws a Garrison row with the y0+10 figure",
		dock.contains("Garrison") and dock.contains(CivilizationWorkspace._head(float(_garrison_of(b, tid)))),
		dock.substr(maxi(0, dock.find("Capital")), 160).replace("\n", " | "))

	# -- 4b. SHELL: the place editor's Classification section, same figure.
	var pe = _app.place_editor_window
	pe.open_for(i0)
	await _frames(8)
	var pet := _labels_text(pe)
	var want := "Garrison: %s" % CivilizationWorkspace._head(float(_garrison_of(b, tid)))
	_check("the place editor draws '%s'" % want, pet.contains(want))
	pe.hide()

	# -- 5. The Conflict layer's siege names the besieged garrison.
	var r: Dictionary = _bridge.conflict_add({"name": "Siege of the probe", "kind": "siege",
		"start_year": y0, "end_year": y0 + 20, "sides": PackedInt32Array([f1, f2]),
		"points": PackedVector2Array([Vector2(float(s0["x"]), float(s0["y"]))]),
		"anchor_kind": "settlement", "anchor_tid": tid})
	var rows: Array = _bridge.conflict_campaigns(y0 + 12)
	var sg: Dictionary = (rows[0] as Dictionary).get("siege", {}) if rows.size() > 0 else {}
	_check("the siege row carries the besieged town's y0+10 garrison",
		bool(r.get("ok", false)) and int(sg.get("garrison", -1)) == _garrison_of(b, tid), str(sg))

	_p("RESULT %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)


## Pinned from the first run of this probe on seed 5521 (see the report).
const STANDING_F1_Y0 := 4741
const STANDING_F1_Y10 := 5292
const GARRISON_TOWN_Y0 := 2194
const GARRISON_TOWN_Y10 := 3679
