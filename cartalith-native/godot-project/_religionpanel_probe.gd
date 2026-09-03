extends Node
## Boots the real shell and renders CIVIL > Religion over a real world, by
## calling `CivilizationWorkspace._fill_religion` itself -- not by
## re-implementing its formatting here, which is how a verification pass talks
## itself into agreeing with the bug (`_diagdash_probe.gd`'s own words).
##
## What only this can reach: that the category builds at all through
## `DccWidgets`, that every one of the three "no adherence" states renders its
## own reason rather than a blank, that the stacked bar is a real container of
## real segments, and that the culture-staleness warning -- the half of the
## guard the engine deliberately does not cover -- actually fires on a culture
## edit and only on one.
##
## Run: godot4 --headless --path . _religionpanel_probe.tscn

var _app: Node
var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1

## Every Label/Button/CheckBox text under `node`, depth-first, plus a marker
## for each stacked bar found (an HBoxContainer of ColorRects).
func _harvest(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		if c is HBoxContainer:
			var rects := 0
			for g in c.get_children():
				if g is ColorRect:
					rects += 1
			if rects > 0:
				out.append("<bar:%d>" % rects)
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

func _joined(lines: Array) -> String:
	return " | ".join(PackedStringArray(lines))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
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

	print("== state 'no_world' (built at launch, before any generate) ==")
	var l0 := _render(ws)
	print("  ", _joined(l0))
	_chk(_joined(l0).contains("generate a world first"),
		"an empty engine says what to do, and does not draw an empty roster")

	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	print("  generated: ", bridge.settlements().size(), " settlements")

	print("== state 'not_run' ==")
	var l1 := _render(ws)
	print("  ", _joined(l1))
	_chk(_joined(l1).contains("No diffusion has been run"),
		"a world with no layer says so; it does not show an empty breakdown")
	_chk(not _joined(l1).contains("<bar:"), "and draws no bar over data it does not have")
	_chk(_joined(l1).contains("Run diffusion"), "the run control is reachable in that state")

	print("== state 'secular' (roster is all None, as generated) ==")
	ws._religion_run()
	var l2 := _render(ws)
	print("  ", _joined(l2))
	_chk(_joined(l2).contains("every faction's religion is set to None"),
		"the engine's own reason string is what the panel shows")
	_chk(_joined(l2).contains("Faction Roster"), "with the action that fixes it")
	_chk(_joined(l2).contains("Last run:"), "and the run is still reported as having happened")

	print("== state 'live' ==")
	bridge.civ_set_faction_field(1, "religion", "sun_cult")
	bridge.civ_set_faction_field(3, "religion", "old_gods")
	ws._religion_run()
	var l3 := _render(ws)
	print("  ", _joined(l3).substr(0, 1400))
	var bars := 0
	var empties := 0
	for t in l3:
		if String(t).begins_with("<bar:"):
			bars += 1
		if String(t).strip_edges().is_empty():
			empties += 1
	_chk(bars > 1, "the roll-up and the settlement rows both draw a real bar (%d bars)" % bars)
	_chk(empties == 0, "no rendered row is blank (%d blanks)" % empties)
	_chk(not _joined(l3).contains("None / secular"),
		"the unaffiliated slot never renders with the vocabulary's own confusing label")
	_chk(_joined(l3).contains("No religion"), "it renders as words instead")
	_chk(_joined(l3).contains("follow"), "the divergence count is stated")
	_chk(_joined(l3).contains("Show on map"), "and the map layer is reachable from here")
	_chk(not _joined(l3).contains("STALE"),
		"no staleness warning fires immediately after a run")

	print("== state 'invalidated': a religion set AFTER a run drops the layer ==")
	bridge.civ_set_faction_field(4, "religion", "sea_lords")
	var li := _render(ws)
	print("  ", _joined(li).substr(0, 700))
	_chk(_joined(li).contains("last run has been discarded"),
		"the panel says the run was thrown away, not that none ever happened")
	_chk(not _joined(li).contains("No diffusion has been run"),
		"and does not contradict what the user just did")
	ws._religion_run()

	print("== the culture half of the guard, which the engine does not cover ==")
	var before_key: PackedStringArray = ws._religion_culture_key.duplicate()
	bridge.civ_set_faction_field(1, "name", "Renamed Polity")
	var l4 := _render(ws)
	_chk(not _joined(l4).contains("culture has changed"),
		"a rename does NOT raise the culture warning")
	print("  culture column before: ", before_key)
	var cults: PackedStringArray = bridge.civ_culture_vocabulary()
	var target := ""
	for c in cults:
		if String(c) != before_key[0]:
			target = String(c)
			break
	print("  setting faction 1 culture: ", before_key[0], " -> ", target)
	var ok: bool = bridge.civ_set_faction_field(1, "culture", target)
	var l5 := _render(ws)
	print("  ", _joined(l5).substr(0, 700))
	_chk(ok, "the culture edit was accepted by the roster")
	_chk(_joined(l5).contains("culture has changed"),
		"a CULTURE edit raises the warning the engine's belief_current cannot")
	_chk(_joined(l5).contains("STALE"), "and it is marked STALE where the reader will see it")
	_chk(bridge.civ_belief_run(0).get("seeded", true) == false,
		"while the ENGINE still reports the layer current -- which is the whole point")

	ws._religion_run()
	var l6 := _render(ws)
	_chk(not _joined(l6).contains("culture has changed"),
		"re-running clears the warning")

	print("")
	print("RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
