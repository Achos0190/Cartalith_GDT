extends Node
## Lane B probe: does CIVIL > Religion > Adherence's new per-faction block ever
## print a faction a composition it does not have?
##
## Everything here is compared against a CONTROL recomputed straight from
## `get_settlements()` in this file, never against the panel's own arithmetic:
##   A) each faction's rendered faith set vs. its summed `adherents` keys
##   B) each faction's rendered percentages vs. the control's, digit for digit
##   C) the disagreement headline vs. a control count of pluralities
##   D) no bare `0.0%` and no `100.0%` beside a second faith at faction scale
##   E) a faction emptied of settlements dashes with its reason instead of 0

var _app: Node
var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1

func _harvest(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		_harvest(c, out)

func _render(ws: Control) -> Array:
	var host := VBoxContainer.new()
	add_child(host)
	ws._fill_religion(host)
	var lines: Array = []
	_harvest(host, lines)
	remove_child(host)
	host.queue_free()
	return lines

## The control: {faction id -> {religion key -> adherents}} recomputed here.
func _control(places: Array) -> Dictionary:
	var per := {}
	for p in places:
		var d: Dictionary = p
		var f := int(d.get("faction", 0))
		if not d.has("religion"):
			continue
		if not per.has(f):
			per[f] = {}
		var ad: Dictionary = d.get("adherents", {})
		for k in ad.keys():
			per[f][k] = int(per[f].get(k, 0)) + int(ad[k])
	return per

func _pct(n: int, total: int) -> String:
	if total <= 0:
		return "—"
	var pct := 100.0 * float(n) / float(total)
	if n > 0 and pct < 0.05:
		return "<0.1%"
	if n < total and pct > 99.95:
		return ">99.9%"
	return "%.1f%%" % pct

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	var ws: Control = null
	for w in _app._workspaces:
		if w.name == "CivilizationWorkspace":
			ws = w
	if ws == null:
		print("PROBE FAIL: no CivilizationWorkspace")
		get_tree().quit(2)
		return

	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	bridge.civ_set_faction_field(1, "religion", "sun_cult")
	bridge.civ_set_faction_field(2, "religion", "old_gods")
	bridge.civ_set_faction_field(3, "religion", "sea_lords")
	ws._religion_years = 60
	ws._religion_run()

	var places: Array = bridge.settlements()
	var rows: Array = bridge.get_factions()
	var per := _control(places)
	print("settlements=", places.size(), " factions=", rows.size(),
		" status=", ws._religion_status)

	var lines := _render(ws)

	# ---- A/B: rendered faction composition vs. the control -----------------
	var set_bad := 0
	var pct_bad := 0
	var matched := 0
	var control_disagree := 0
	var control_compared := 0
	var examples: Array = []
	for r in rows:
		var d: Dictionary = r
		var f := int(d.get("id", 0))
		var fname := String(d.get("name", ""))
		var counts: Dictionary = per.get(f, {})
		var total := 0
		for k in counts.keys():
			total += int(counts[k])
		# control plurality (largest, ties to lower key)
		var best := ""
		var bn := -1
		for k in counts.keys():
			var n := int(counts[k])
			if n > bn or (n == bn and String(k) < best):
				bn = n
				best = String(k)
		if best != "":
			control_compared += 1
			if best != String(d.get("religion", "")):
				control_disagree += 1
		# locate the faction's header row, then its composition line
		var anchor := -1
		for i in lines.size():
			if String(lines[i]).begins_with(fname + " — state religion"):
				anchor = i
				break
		if anchor < 0:
			examples.append("NOROW: " + fname)
			set_bad += 1
			continue
		matched += 1
		var comp := ""
		for j in range(anchor + 1, mini(anchor + 4, lines.size())):
			var t := String(lines[j])
			if t.contains("◆") or t.contains("◇") or t.begins_with("—"):
				comp = t
				break
		if total == 0:
			if not comp.begins_with("—"):
				set_bad += 1
				examples.append("UNDASHED: " + fname + " -> " + comp)
			continue
		for k in counts.keys():
			var lbl: String = ws._religion_label(String(k))
			if not comp.contains(lbl):
				set_bad += 1
				examples.append("MISSING: " + fname + " lacks " + lbl + " -> " + comp)
			var want := _pct(int(counts[k]), total)
			if not comp.contains(lbl + " " + want):
				pct_bad += 1
				examples.append("PCT: " + fname + " " + lbl + " want " + want + " -> " + comp)
	for e in examples:
		print("   ", e)
	_chk(matched == rows.size(), "every faction has a rendered row (%d of %d)" % [matched, rows.size()])
	_chk(set_bad == 0, "no faction row omits or invents a faith (%d)" % set_bad)
	_chk(pct_bad == 0, "every rendered faction percentage equals the control's (%d off)" % pct_bad)

	# ---- C: the headline vs. the control -----------------------------------
	var head := ""
	for t in lines:
		var s := String(t)
		if s.contains("state religion first") or s.contains("hold the faith their ruler has set"):
			head = s
			break
	print("headline: ", head, "   control: disagree=", control_disagree,
		" compared=", control_compared)
	var head_ok := false
	if control_disagree == 0:
		head_ok = head == ("All %d faction%s with people in them hold the faith their ruler has set."
			% [control_compared, "" if control_compared == 1 else "s"])
	elif control_disagree == 1:
		head_ok = head == "1 faction's people no longer put its state religion first, of %d compared." % control_compared
	else:
		head_ok = head == ("%d factions' people no longer put their state religion first, of %d compared."
			% [control_disagree, control_compared])
	_chk(head_ok, "the headline matches the control count exactly")
	_chk(control_disagree > 0 or control_compared > 0,
		"the fixture reaches the comparison at all (compared=%d)" % control_compared)

	# ---- D: no fabricated zero / self-contradicting hundred ----------------
	var zero := 0
	var hund := 0
	for t in lines:
		var s := String(t)
		if s.contains("people · ") and (s.contains("◆") or s.contains("◇")):
			if s.contains(" 0.0%"):
				zero += 1
			if s.contains("100.0%") and (s.count("◆") + s.count("◇")) > 1:
				hund += 1
	_chk(zero == 0, "no faction row prints a bare 0.0%% for a real congregation (%d)" % zero)
	_chk(hund == 0, "no faction row reads 100.0%% while listing another faith (%d)" % hund)

	# ---- E1: a faction with no settlements dashes with its reason ----------
	var added: int = bridge.civ_add_faction()
	print("added faction id=", added)
	ws._religion_run()
	var l2 := _render(ws)
	var aname := ""
	for r in bridge.get_factions():
		if int((r as Dictionary).get("id", 0)) == added:
			aname = String((r as Dictionary).get("name", ""))
	var a2 := -1
	for i in l2.size():
		if String(l2[i]).begins_with(aname + " — state religion"):
			a2 = i
			break
	var next := String(l2[a2 + 1]) if a2 >= 0 and a2 + 1 < l2.size() else "(no row)"
	print("empty faction '", aname, "' row -> ", next)
	_chk(next.begins_with("— no settlements"),
		"a faction with no settlements dashes with its reason rather than showing 0")

	# ---- E2: settlements reverted to Unclaimed get their own row ----------
	# `FactionRoster::remove_last` sends the removed faction's settlements back
	# to `0`. Two removals: the first takes the one just added (no settlements),
	# the second takes a real faction and orphans its towns.
	bridge.civ_remove_faction()
	bridge.civ_remove_faction()
	ws._religion_run()
	var l3 := _render(ws)
	var orphans := 0
	for p in bridge.settlements():
		if int((p as Dictionary).get("faction", 0)) == 0:
			orphans += 1
	var uncl := -1
	for i in l3.size():
		if String(l3[i]).begins_with("Unclaimed — state religion"):
			uncl = i
			break
	print("orphaned settlements=", orphans, " unclaimed row index=", uncl,
		" -> ", (String(l3[uncl]) if uncl >= 0 else "(absent)"),
		" | ", (String(l3[uncl + 1]) if uncl >= 0 and uncl + 1 < l3.size() else ""))
	_chk(orphans > 0, "the fixture actually orphaned settlements (%d)" % orphans)
	_chk(uncl >= 0, "the now-unruled settlements get their own Unclaimed row")

	print("FAILS=", _fails)
	get_tree().quit(1 if _fails > 0 else 0)
