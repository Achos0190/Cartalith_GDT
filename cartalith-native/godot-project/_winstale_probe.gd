extends Node
## Committed probe for the 2026-08-25 "is every control wired" pass.
##
## RF-01's other half. §23 asked "what re-runs this, and on which signal?" of
## every panel built at launch. This asks it of every WINDOW **left open while
## the world changes underneath it**. Only six of the shell's fourteen windows
## subscribe to `generation_finished`/`world_loaded` at all; the rest rebuild on
## `open()`, which is correct only if nothing can change while they are up.
##
## The dangerous ones are the ones keyed to an INDEX a regenerate renumbers --
## the place editor (settlement index), the faction roster (faction id), the
## vault (entity id) -- because a stale window is not merely out of date, it
## writes an edit to whatever now sits at that index. That is FR-02's failure
## mode with a different trigger.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _winstale_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("WINSTALE  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("WINSTALE  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _texts(root: Node) -> String:
	if root == null:
		return ""
	var all: Array = []
	_walk(root, all)
	var parts := PackedStringArray()
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
		elif n is OptionButton:
			parts.append("%s|%d" % [(n as OptionButton).text, (n as OptionButton).item_count])
		elif n is Button:
			parts.append("%s|%s" % [(n as Button).text, str((n as Button).disabled)])
		elif n is LineEdit:
			parts.append((n as LineEdit).text)
		elif n is ItemList:
			parts.append("IL%d" % (n as ItemList).item_count)
	return "\n".join(parts)


func _generate(seed: int, gw: int, gh: int, km: float) -> void:
	_bridge.generate({
		"seed": seed, "width_km": km, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	await _generate(483920, 384, 288, 2400.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var names_a := PackedStringArray()
	for s in _bridge.settlements():
		names_a.append(String((s as Dictionary).get("name", "")))
	var fac_a := PackedStringArray()
	for f in _bridge.get_factions():
		fac_a.append(String((f as Dictionary).get("name", "")))
	_p("world A: %d settlements (#0 '%s'), %d factions (#0 '%s')" % [
		names_a.size(), names_a[0] if names_a.size() > 0 else "-",
		fac_a.size(), fac_a[0] if fac_a.size() > 0 else "-"])

	## Every window, opened against world A and LEFT OPEN across a generate.
	var wins := [
		["PlaceEditor", _app.place_editor_window, "open_for", [0]],
		["FactionRoster", _app.faction_roster_window, "open", []],
		["CityViewer", _app.city_viewer_window, "open", [0]],
		["WorldData", _app.world_data_window, "open", []],
		["Performance", _app.performance_window, "open", []],
		["TravelLibrary", _app.travel_library_window, "open", []],
		["Vault", _app.vault_window, "open_overview", []],
		["DataManager", _app.data_manager_window, "open", []],
		["AssetLibrary", _app.asset_library_window, "open", []],
		["LayersPopover", _app.layers_popover, "open", []],
	]
	var seed_flip := true
	for row in wins:
		var w = row[1]
		if w == null:
			_p("%s :: null" % row[0])
			continue
		w.callv(row[2], row[3])
		await _frames(8)
		var before := _texts(w)
		## Alternate the two worlds so every window is measured across a real
		## identity change, not against whatever the last iteration left behind.
		if seed_flip:
			await _generate(771155, 256, 192, 900.0)
		else:
			await _generate(483920, 384, 288, 2400.0)
		seed_flip = not seed_flip
		await _frames(8)
		var after := _texts(w)
		var still_open: bool = w.visible if w is Window else true
		var verdict := "REBUILT" if before != after else "IDENTICAL"
		_p("%-14s open=%s across a generate -> %s (%d -> %d chars)" % [
			row[0], str(still_open), verdict, before.length(), after.length()])
		if still_open and before == after and before.length() > 40:
			## Not automatically a bug: a window reading nothing world-specific
			## (the travel library's reference tables) is correctly unchanged.
			## Reported so each can be judged.
			_p("   >> candidate: %s renders world data? check below" % row[0])
		if w.has_method("hide"):
			w.hide()
		await _frames(3)

	## ---- the sharp case, driven all the way -------------------------------
	## The place editor keyed to settlement index 0. Regenerate underneath it
	## and ask what its name field now claims -- and, if it still claims world
	## A's name, whether committing that field would write it onto world B's
	## settlement 0.
	## The name RNG is seeded the same way in every world, so settlement 0 and
	## faction 0 come out with the SAME NAME on both seeds -- a name comparison
	## here proves nothing. Population, coordinates and counts do.
	_p("=== place editor, index 0, across a generate ===")
	await _generate(483920, 384, 288, 2400.0)
	await _frames(6)
	var sa: Dictionary = _bridge.settlements()[0]
	_app.open_place_editor(0)
	await _frames(10)
	_p("world A settlement 0: '%s' pop=%d at (%.1f, %.1f)" % [
		String(sa.get("name", "")), int(sa.get("population", 0)),
		float(sa.get("x", 0.0)), float(sa.get("y", 0.0))])
	var shown_a := _texts(_app.place_editor_window)
	await _generate(771155, 256, 192, 900.0)
	await _frames(10)
	var sb: Dictionary = _bridge.settlements()[0]
	var shown_b := _texts(_app.place_editor_window)
	_p("world B settlement 0: '%s' pop=%d at (%.1f, %.1f)" % [
		String(sb.get("name", "")), int(sb.get("population", 0)),
		float(sb.get("x", 0.0)), float(sb.get("y", 0.0))])
	_p("editor open=%s; text moved: %s" % [
		str(_app.place_editor_window.visible), str(shown_a != shown_b)])
	var pa := int(sa.get("population", 0))
	var pb := int(sb.get("population", 0))
	_p("editor shows A's pop %d: %s / B's pop %d: %s" % [
		pa, "YES" if shown_b.find(str(pa)) >= 0 else "no",
		pb, "yes" if shown_b.find(str(pb)) >= 0 else "NO"])
	if _app.place_editor_window.visible and pa != pb and shown_a == shown_b:
		_bad("the place editor is still showing world A's settlement 0 (pop %d) over "
			% pa + "world B (pop %d) -- an edit committed here writes onto B" % pb)
	_app.place_editor_window.hide()
	await _frames(4)

	_p("=== faction roster, across a generate ===")
	await _generate(483920, 384, 288, 2400.0)
	await _frames(6)
	var stats_a := PackedStringArray()
	for f in _bridge.get_factions():
		stats_a.append("%s:%d" % [String((f as Dictionary).get("name", "")),
			int((f as Dictionary).get("settlement_count", 0))])
	_app.open_faction_roster()
	await _frames(10)
	var roster_a := _texts(_app.faction_roster_window)
	await _generate(771155, 256, 192, 900.0)
	await _frames(10)
	var stats_b := PackedStringArray()
	for f in _bridge.get_factions():
		stats_b.append("%s:%d" % [String((f as Dictionary).get("name", "")),
			int((f as Dictionary).get("settlement_count", 0))])
	var roster_b := _texts(_app.faction_roster_window)
	_p("A factions %s" % ", ".join(stats_a))
	_p("B factions %s" % ", ".join(stats_b))
	_p("roster open=%s; text moved: %s" % [
		str(_app.faction_roster_window.visible), str(roster_a != roster_b)])
	if _app.faction_roster_window.visible and stats_a != stats_b and roster_a == roster_b:
		_bad("the faction roster is still showing world A's roster over world B")
	_app.faction_roster_window.hide()

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
