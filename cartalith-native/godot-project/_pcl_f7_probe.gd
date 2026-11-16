extends "res://_pcl_base.gd"
## PC-4 leftover, finding F7 (`PC_SHELL_PROPOSAL.md` 3.2 "What the year moves": "Picking a conflict moves the year to its
## first year, so its campaign draws"). Protects `DccApp.conflict_goto_start()` and the three list pickers that call it.
##   A. Story > Conflicts window: with the year at 100 (the conflict, years 250 to 260, is not active), a REAL click on
##      its row selects it AND moves the engine's year to 250 (not 260, not 0), exactly.
##   B. The same pick with the year already 250 changes nothing and still selects.
##   C. CIVIL's Military > Conflicts row (`CivilizationWorkspace._fill_conflicts`): same year move.
##   D. NEGATIVE: a conflict LINK on an Inspect page (Conflicts here) selects without moving the year (those keep it),
##      and `conflict_goto_start(-5)` (no such conflict) returns false and leaves the year alone.
## Mutants (literal, run by the harness script): F7_NO_MOVE (`conflict_goto_start(id)` call removed from `open_conflict`),
## F7_END_YEAR (`d["start_year"]` read as `end_year`), F7_CIVIL_ROW (CIVIL row without the year move).

func _run() -> void:
	_prefix = "f7"
	_app.select_domain("civilization")
	_bridge.civ_goto_year(100)
	await _settle(4)
	_ok(_year() == 100 and not _bridge.conflict_list().is_empty(), "start: year 100 and the harness conflict exists")
	print("== A. Story > Conflicts window ==")
	_app.open_conflicts_list()
	await _settle(8)
	var win = _app.conflicts_window
	var btn: Button = null
	for r in win.find_children("CfRow", "BoxContainer", true, false):
		if int((r as Node).get_meta("conflict_id", -1)) == _cid:
			btn = (r as Node).find_child("CfSelect", true, false) as Button
	_ok(btn != null, "the conflict's row button exists in the window")
	if await _press(btn, "the conflict row"):
		await _settle(6)
		_ok(_year() == 250, "a list pick moved the year to the conflict's START year, 250 (got %d)" % _year())
		_ok(_app.selection == {"kind": "conflict", "id": _cid}, "and selected it (got %s)" % str(_app.selection))
	await _shot("conflicts_window")
	print("== B. already at the start year ==")
	_app.select("", -1)
	_bridge.civ_goto_year(250)
	var fired := [0]
	_app.timeline_changed.connect(func() -> void: fired[0] += 1)
	if not win.visible:
		## The tablet's list is modal and closed itself on the pick (`ContextWindow.applies()` is false there).
		_app.open_conflicts_list()
		await _settle(8)
	btn = null
	for r in win.find_children("CfRow", "BoxContainer", true, false):
		if int((r as Node).get_meta("conflict_id", -1)) == _cid:
			btn = (r as Node).find_child("CfSelect", true, false) as Button
	if await _press(btn, "the conflict row again"):
		await _settle(6)
		_ok(_year() == 250 and fired[0] == 0, "year stays 250 and no timeline_changed fired (fired %d)" % fired[0])
		_ok(_app.selection == {"kind": "conflict", "id": _cid}, "and it is selected")
	win.hide()
	if DccTheme.is_phone():
		## The phone has no CIVIL dock rows and no Inspect page (`PC_SHELL_PROPOSAL.md` 3.11): legs C and D are desktop and tablet only.
		return
	print("== C. CIVIL Military > Conflicts row ==")
	_bridge.civ_goto_year(100)
	var ws = null
	for w in _app._workspaces:
		if w.has_method("select_conflict"):
			ws = w
	_ok(ws != null, "the CIVIL workspace is found")
	ws._military_body.visible = true   # the accordion category (opening it is not under test; the row press below is)
	ws.refresh_conflicts()
	await _settle(8)
	var civ_btn: Button = null
	for b in _app.find_children("*", "Button", true, false):
		if (b as Button).is_visible_in_tree() and (b as Button).text.contains("Probe War"):
			civ_btn = b
	_ok(civ_btn != null, "CIVIL lists the conflict as a row button")
	if civ_btn != null and await _press(civ_btn, "CIVIL's conflict row"):
		await _settle(6)
		_ok(_year() == 250, "the CIVIL row moved the year to 250 (got %d)" % _year())
	print("== D. negatives ==")
	_bridge.civ_goto_year(100)
	_app.select("settlement", _anchor)
	await _settle(8)
	var link: Button = null
	for b in _app.right_dock_body.find_children("*", "Button", true, false):
		if (b as Node).has_meta("ctx_target") and (b as Button).is_visible_in_tree():
			var t = (b as Node).get_meta("ctx_target")
			if typeof(t) == TYPE_DICTIONARY and String(t.get("kind", "")) == "conflict":
				link = b
	_ok(link != null, "the settlement page has a Conflicts here link")
	if await _press(link, "the Conflicts here link"):
		await _settle(6)
		_ok(_app.selection.get("kind", "") == "conflict" and _year() == 100, "an Inspect link selects but keeps the year (%d)" % _year())
	_ok(_app.conflict_goto_start(-5) == false and _year() == 100, "an unknown conflict id returns false and leaves the year")
