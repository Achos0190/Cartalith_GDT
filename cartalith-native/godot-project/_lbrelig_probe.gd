extends Node
## Lane B probe: the Religion panel's world roll-up sentence, the diverged
## roster, and whether a layer left ON can be turned off after a regenerate.
##
## Every number printed is read off the ENGINE (`get_settlements()`'s own
## `adherents` dictionaries and `get_factions()`' religion column) and then
## compared against the RENDERED text, so a panel that agrees with itself but
## not with the world fails here.

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

func _find(lines: Array, needle: String) -> String:
	for l in lines:
		if String(l).find(needle) >= 0:
			return String(l)
	return ""

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 500.0
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

	## Seed/size chosen so the divergence count is NOT zero -- `_religionui_
	## probe.gd` measured 2 of 8 on exactly this world, and a naming check over
	## an empty set passes by construction.
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	bridge.civ_set_faction_field(1, "religion", "sun_cult")
	bridge.civ_set_faction_field(3, "religion", "old_gods")
	ws._religion_years = 80
	ws._religion_run()

	var places: Array = bridge.settlements()
	print("settlements: ", places.size(), "  status=", ws._religion_status)

	# ---------------- engine truth ----------------
	var keys := {}
	for p in places:
		var ad: Dictionary = (p as Dictionary).get("adherents", {})
		for k in ad.keys():
			keys[String(k)] = true
	var faith_keys := keys.keys().filter(func(k): return String(k) != "none")
	var has_none: bool = keys.has("none")
	print("engine: distinct adherent keys=", keys.keys(),
		"  faiths(excluding none)=", faith_keys.size(), "  none present=", has_none)

	var col: PackedStringArray = ws._religion_faction_column("religion")
	var diverged: Array = []
	for p in places:
		var d: Dictionary = p
		if not d.has("religion"):
			continue
		var f := int(d.get("faction", 0))
		if f <= 0 or f > col.size() or col[f - 1].is_empty():
			continue
		if String(d["religion"]) != col[f - 1]:
			diverged.append(String(d.get("name", "")))
	print("engine: roster column=", col, "  diverged=", diverged.size(), " -> ", diverged)

	# ---------------- rendered ----------------
	var lines := _render(ws)
	var roll := _find(lines, "people across")
	print("rendered roll-up: '", roll, "'")
	_chk(roll.find("in %d faith" % faith_keys.size()) >= 0,
		"the roll-up counts %d faith(s), not the unaffiliated slot" % faith_keys.size())

	var div_line := _find(lines, "no longer share their ruler")
	if div_line == "":
		div_line = _find(lines, "share their ruler")
	print("rendered divergence: '", div_line, "'")
	_chk(div_line.find("%d of " % diverged.size()) >= 0,
		"the divergence sentence carries the count AND its denominator (%d)" % diverged.size())

	_chk(diverged.size() > 0, "the probe's own world actually diverges (else the next "
		+ "check passes by construction)")
	var diverged_idx := -1
	for i in places.size():
		var dd: Dictionary = places[i]
		if not dd.has("religion"):
			continue
		var ff := int(dd.get("faction", 0))
		if ff <= 0 or ff > col.size() or col[ff - 1].is_empty():
			continue
		if String(dd["religion"]) != col[ff - 1]:
			diverged_idx = i
			break
	var named := 0
	for n in diverged:
		if _find(lines, n) != "":
			named += 1
	print("diverged settlements NAMED in the panel: ", named, " of ", diverged.size())
	_chk(named == diverged.size(), "every diverged settlement is named, not just counted")
	_chk(_find(lines, "%d DIVERGED" % diverged.size()) != ""
		or _find(lines, "%d diverged" % diverged.size()) != "",
		"the diverged group is headed with its own count")

	_chk(_find(lines, "Show on map") != "", "the map toggle is drawn while the layer is live")

	# ---------------- the ring's caption on the hover card ----------------
	var ov = _app.viewport.overlay
	var sample: Dictionary = places[diverged_idx] if diverged_idx >= 0 else {}
	ov.set_faith_divergence_visible(false)
	var off: Array = ov._faith_lines(sample)
	ov.set_faith_divergence_visible(true)
	var on: Array = ov._faith_lines(sample)
	print("hover card, layer off: ", off)
	print("hover card, layer on:  ", on)
	_chk(ov._faith_diverged(sample), "the sampled settlement really carries a ring")
	_chk(off.size() + 1 == on.size(), "the caption is one extra line, present only with the ring")
	_chk(String(on[on.size() - 1]).begins_with("Ruler's faith "),
		"and it names the ruler's faith rather than restating the settlement's")
	ov.set_faith_divergence_visible(false)

	# ---------------- the orphaned-layer case ----------------
	_app.viewport.overlay.set_faith_divergence_visible(true)
	bridge.generate({
		"seed": 991, "width_km": 2000.0, "grid_w": 192, "grid_h": 128,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	var after := _render(ws)
	print("after regenerate: layer on=", _app.viewport.overlay.faith_divergence_visible(),
		"  toggle drawn=", _find(after, "Show on map") != "")
	_chk(not _app.viewport.overlay.faith_divergence_visible()
		or _find(after, "Show on map") != "",
		"a layer left ON is still reachable after a world replacement")

	print("\nRESULT: ", "ALL PASS" if _fails == 0 else "%d FAIL" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
