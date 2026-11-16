extends "res://_pcl_base.gd"
## PC-4 leftover (PQ9, Ruling DY; `PC_SHELL_PROPOSAL.md` 3.3 "Inspect's edit mode ... name, class, faction and fortified").
## Protects `RightDock._build_settlement_edit_rows` and `_inspect_edit_apply`.
##   A. Edit is OFF by default; a REAL click on `InspectEditToggle` turns it on: Name, Class, Faction and Walls editors exist.
##   B. Typing a name and pressing Enter (a real key in the LineEdit) renames the settlement in the ENGINE, and the CIVIL list reads it.
##   C. Class: the choice (item_selected) writes `kind` in the engine; "capital" also sets the capital flag.
##   D. Walls: Fortified writes walls 1, No fortifications 0, Auto -1 (read back by `civ_settlement_details`).
##   E. Faction: picking another faction opens the Place editor's confirm (nothing moves until confirmed): the settlement's
##      faction is unchanged while the confirm is open.
##   F. Selecting a DIFFERENT settlement turns edit mode off (it belongs to the place it was turned on for).
##   G. The Edit toggle is absent on the phone form (`_panel_chrome()` false): not run here.
## Mutants: EDIT_NAME_LITERAL (the commit writes "X"), EDIT_WALLS_FORTIFIED (1 -> 0), EDIT_STICKY (the reset of `_inspect_edit`
## in `on_settlement_selected` removed), EDIT_NO_EMIT (`place_changed.emit()` removed from `_inspect_edit_apply`).

func _run() -> void:
	_prefix = "edit"
	var idx := _anchor
	var s0: Dictionary = _bridge.settlements()[idx]
	_app.select("settlement", idx)
	await _settle(10)
	var body: Control = _app.right_dock_body
	print("== A. toggle ==")
	_ok(body.find_child("InspectEditName", true, false) == null, "edit mode is off by default (no name editor)")
	var tg := body.find_child("InspectEditToggle", true, false) as Button
	_ok(tg != null and tg.text == "Edit", "the Edit toggle exists and reads Edit")
	if not await _press(tg, "the Edit toggle"):
		return
	await _settle(8)
	for nm in ["InspectEditName", "InspectEditClass", "InspectEditFaction", "InspectEditWalls"]:
		_ok(body.find_child(nm, true, false) != null, "edit mode draws %s" % nm)
	await _shot("edit_on")
	print("== B. name ==")
	var le := body.find_child("InspectEditName", true, false) as LineEdit
	le.grab_focus()
	le.text = "Probeton"
	if not DccTheme.is_tablet():
		await _key(KEY_ENTER)
		await _settle(8)
		_ok(String((_bridge.settlements()[idx] as Dictionary).get("name", "")) == "Probeton", "Enter (a real key) writes the engine's name (got %s)" % String((_bridge.settlements()[idx] as Dictionary).get("name", "")))
		var hn := _app.right_dock_body.find_child("CtxName", true, false) as Label
		_ok(hn != null and hn.text == "Probeton", "and place_changed re-read the panel: the header name is Probeton (got %s)" % (hn.text if hn != null else "none"))
	else:
		## A touch LineEdit takes no hardware Enter from the probe; the Done editing path below is the commit under test there.
		pass
	## The DCC buttons take no focus, so Done editing must commit a name still being typed (focus_exited never fires).
	body = _app.right_dock_body
	le = body.find_child("InspectEditName", true, false) as LineEdit
	le.text = "Probeton2"
	le.grab_focus()
	var done := body.find_child("InspectEditToggle", true, false) as Button
	_ok(done != null and done.text == "Done editing", "the toggle reads Done editing")
	await _press(done, "Done editing")
	await _settle(8)
	_ok(String((_bridge.settlements()[idx] as Dictionary).get("name", "")) == "Probeton2", "Done editing committed the typed name (got %s)" % String((_bridge.settlements()[idx] as Dictionary).get("name", "")))
	body = _app.right_dock_body
	tg = body.find_child("InspectEditToggle", true, false) as Button
	await _press(tg, "Edit again")
	await _settle(8)
	print("== C. class ==")
	body = _app.right_dock_body
	var kc := body.find_child("InspectEditClass", true, false) as OptionButton
	var kinds: Array = PlaceEditorWindow.KIND_ORDER
	var want_kind := "village" if String(s0.get("kind", "")) != "village" else "hamlet"
	kc.item_selected.emit(kinds.find(want_kind))
	await _settle(8)
	_ok(String((_bridge.settlements()[idx] as Dictionary).get("kind", "")) == want_kind, "class written: %s" % want_kind)
	print("== D. walls ==")
	body = _app.right_dock_body
	var wc := body.find_child("InspectEditWalls", true, false) as OptionButton
	wc.item_selected.emit(2)
	await _settle(6)
	_ok(int(_bridge.civ_settlement_details(idx).get("walls", -9)) == 1, "Fortified writes walls 1")
	body = _app.right_dock_body
	wc = body.find_child("InspectEditWalls", true, false) as OptionButton
	wc.item_selected.emit(1)
	await _settle(6)
	_ok(int(_bridge.civ_settlement_details(idx).get("walls", -9)) == 0, "No fortifications writes walls 0")
	body = _app.right_dock_body
	wc = body.find_child("InspectEditWalls", true, false) as OptionButton
	wc.item_selected.emit(0)
	await _settle(6)
	_ok(int(_bridge.civ_settlement_details(idx).get("walls", -9)) == -1, "Auto writes walls -1")
	print("== E. faction ==")
	body = _app.right_dock_body
	var fc := body.find_child("InspectEditFaction", true, false) as OptionButton
	var cur := int((_bridge.settlements()[idx] as Dictionary).get("faction", 0))
	var other := _f2 if cur == _f1 else _f1
	var ids: Array = _bridge.get_factions().map(func(f): return int((f as Dictionary).get("id", 0)))
	fc.item_selected.emit(ids.find(other))
	await _settle(8)
	_ok(int((_bridge.settlements()[idx] as Dictionary).get("faction", 0)) == cur, "the faction did not change before the Place editor's confirm is answered")
	await _shot("edit_faction_confirm")
	for d in _app.find_children("*", "AcceptDialog", true, false):
		if (d as Window).visible and d != _app.place_editor_window and d != _app.conflicts_window:
			(d as Window).hide()
	print("== F. selection ==")
	var other_idx: int = (idx + 1) % _bridge.settlements().size()
	le = _app.right_dock_body.find_child("InspectEditName", true, false) as LineEdit
	le.text = "Probeton3"
	_app.select("settlement", other_idx)
	await _settle(10)
	_ok(String((_bridge.settlements()[idx] as Dictionary).get("name", "")) == "Probeton3", "a name still being typed was written to ITS place when the selection moved (got %s)" % String((_bridge.settlements()[idx] as Dictionary).get("name", "")))
	body = _app.right_dock_body
	_ok(body.find_child("InspectEditName", true, false) == null, "another place is shown with edit mode off")
	tg = body.find_child("InspectEditToggle", true, false) as Button
	_ok(tg != null and tg.text == "Edit", "and the toggle reads Edit again")
