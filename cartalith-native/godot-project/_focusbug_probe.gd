extends Node
## TEMPORARY, untracked probe -- the same teardown-fires-focus_exited defect
## found in the Place editor (PE-01), tested where it should be worse: the
## Faction roster.
##
## `_rebuild_inspector()` clears the pane while the name LineEdit still holds
## focus. The handler then writes that field's text through
## `civ_set_faction_field(_selected, "name", …)` -- and `_selected` has ALREADY
## been reassigned to the faction just clicked. So the prediction is: click
## faction B while A's name field has focus, and B is renamed to A's name.
##
## Read the roster from the engine before and after, so the answer is data.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _focusbug_probe.tscn

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("FOCUSBUG  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _names() -> Array:
	var out: Array = []
	for f in _bridge.get_factions():
		out.append("%d:%s" % [int(f.get("id", 0)), String(f.get("name", "?"))])
	return out


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
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var fr = _app.faction_roster_window
	fr.open()
	await _frames(10)
	_p("roster before: %s" % str(_names()))
	_p("selected=%d" % fr._selected)

	## Focus the inspector's name field, the way a user who clicked into it
	## (or tabbed to it) would be.
	var edits: Array = []
	var all: Array = []
	_walk(fr._inspector_body, all)
	for n in all:
		if n is LineEdit:
			edits.append(n)
	if edits.is_empty():
		_p("no LineEdit in the inspector -- cannot test")
		get_tree().quit(0)
		return
	var name_edit: LineEdit = edits[0]
	name_edit.grab_focus()
	await _frames(4)
	_p("focused the name field, text='%s' hasFocus=%s" % [name_edit.text, str(name_edit.has_focus())])

	## Click a DIFFERENT faction in the list.
	var list_all: Array = []
	_walk(fr._list_body, list_all)
	var target: Button = null
	for n in list_all:
		if n is Button and (n as Button).text.find("—") >= 0:
			## the second row, i.e. not the selected one
			if target == null and (n as Button).text.find(name_edit.text) < 0:
				target = n
	if target == null:
		_p("no second faction row found")
		get_tree().quit(0)
		return
	_p("clicking list row '%s'" % target.text)
	target.pressed.emit()
	await _frames(8)

	var after := _names()
	_p("roster after:  %s" % str(after))
	_p("selected=%d" % fr._selected)

	## Duplicate names are the fingerprint.
	var seen := {}
	var dupes: Array = []
	for e in after:
		var nm: String = String(e).split(":")[1]
		if seen.has(nm):
			dupes.append(nm)
		seen[nm] = true
	if dupes.is_empty():
		_p("PASS  no faction was renamed by the selection change")
	else:
		_fail += 1
		_p("FAIL  selecting a faction renamed it -- duplicate names now: %s" % str(dupes))


	# ---- PE-01: the Place editor's ⟳ on its very first press ---------------
	fr.hide()
	await _frames(4)
	var pe = _app.place_editor_window
	pe.open_for(6)
	await _frames(8)
	var pn0 := String(_bridge.settlements()[6].get("name", "?"))
	var rb: Button = null
	var pall: Array = []
	_walk(pe, pall)
	for n in pall:
		if n is Button and (n as Button).text == "⟳":
			rb = n
	if rb == null:
		_p("no reroll button")
	else:
		rb.pressed.emit()
		await _frames(8)
		var pn1 := String(_bridge.settlements()[6].get("name", "?"))
		if pn1 == pn0:
			_fail += 1
			_p("FAIL  PE-01: first ⟳ press still a no-op ('%s')" % pn0)
		else:
			_p("PASS  PE-01: first ⟳ press renamed '%s' -> '%s'" % [pn0, pn1])

	# ---- PE-01b: re-opening on another settlement must not carry text over -
	var hist_before := String(_bridge.civ_settlement_details(7).get("history", ""))
	## focus the history box on #6, then open #7 straight from the map path
	var te: TextEdit = null
	pall = []
	_walk(pe, pall)
	for n in pall:
		if n is TextEdit and not (n is LineEdit):
			te = n
	if te != null:
		te.grab_focus()
		te.text = "SENTINEL-FROM-SIX"
		await _frames(4)
		pe.open_for(7)
		await _frames(8)
		var hist_after := String(_bridge.civ_settlement_details(7).get("history", ""))
		if hist_after.find("SENTINEL-FROM-SIX") >= 0:
			_fail += 1
			_p("FAIL  PE-01b: settlement 6's history text landed on settlement 7")
		else:
			_p("PASS  PE-01b: settlement 7's history is untouched ('%s' -> '%s')" % [hist_before, hist_after])
		var hist_six := String(_bridge.civ_settlement_details(6).get("history", ""))
		_p("      (settlement 6 kept the edit: %s)" % str(hist_six.find("SENTINEL-FROM-SIX") >= 0))
	else:
		_p("no history TextEdit found")

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
