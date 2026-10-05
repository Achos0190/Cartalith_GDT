extends Node
## Verifies `place_editor_window.gd`'s Batch A tab-strip restructuring
## (`lazy-riding-piglet.md`) did not regress anything the single-scroll form
## did, and specifically reproduces PE-01 (`GUI_GAP_REGISTER.md`) against the
## NEW teardown path -- a tab switch, not just a rebuild-on-reroll.
##
## **Protects** (the tab-switch contract, not the tab content): all five tabs
## exist; each switch shows the NEW tab's real built content and tears down the
## old tab's; an uncommitted name edit survives a tab switch (PE-01's mirror
## polarity). The Timeline and Political history checks assert the sections those
## tabs actually build (`_build_timeline`, `_build_timeline_strip` and its lists)
## -- they used to assert a "Not built yet" placeholder that stopped existing when
## those tabs were built, and they now also fail if that placeholder text ever
## comes back.
##
## Windowed, not `--headless`: this exercises a popup `AcceptDialog`'s focus
## behaviour, which is exactly the case `MISTAKES.md`'s "windowed for anything
## that rasterises or times a frame" / focus-owner row calls for a real
## viewport.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _petabstrip_probe.tscn
##
## Exit 0 = every check passed. Exit 1 = a real failure. Exit 2 = the probe
## itself could not run (a control precondition failed, e.g. no settlement
## generated) -- distinct from a real finding, same convention `_focusbug_probe.gd`
## and `_vaultprefs_probe.gd` use.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("PETABSTRIP  %s" % s)


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


func _buttons_text(n: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_walk(n, all)
	for c in all:
		if c is Button:
			out.append((c as Button).text)
	return out


func _find_buttons_by_text(n: Node, text: String) -> Array:
	var out: Array = []
	var all: Array = []
	_walk(n, all)
	for c in all:
		if c is Button and (c as Button).text == text:
			out.append(c)
	return out


func _find_line_edits(n: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_walk(n, all)
	for c in all:
		if c is LineEdit:
			out.append(c)
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
		"seed": 583920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
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
	_p("world generated: %d settlements, probing index %d ('%s')" %
		[places.size(), idx, String((places[idx] as Dictionary).get("name", "?"))])

	var pe = _app.place_editor_window
	pe.open_for(idx)
	await _frames(8)

	# -- 1. all five tabs present, Overview active by default -------------------
	var expected_labels := ["Overview", "Economy & notables", "Timeline", "Political history", "Vault notes"]
	var btn_texts := _buttons_text(pe)
	for lbl in expected_labels:
		_check("tab strip carries '%s'" % lbl, btn_texts.has(lbl))
	_check("opens on Overview", pe._active_tab == "overview", "active='%s'" % pe._active_tab)
	_check("Overview content present at open (Identity section)",
		_labels_text(pe).has("§ IDENTITY"))

	# -- 2. switching tabs shows the new tab and tears down the old one ---------
	## Overview -> Economy & notables.
	var econ_btns := _find_buttons_by_text(pe, "Economy & notables")
	_check("found the Economy & notables tab button", not econ_btns.is_empty())
	if not econ_btns.is_empty():
		(econ_btns[0] as Button).pressed.emit()
		await _frames(6)
		_check("switched to economy", pe._active_tab == "economy")
		_check("Economy tab shows Trade section", _labels_text(pe).has("§ TRADE"))
		_check("Overview's Identity section is torn down", not _labels_text(pe).has("§ IDENTITY"))

	## Economy -> Timeline (built: recorded years + the collapse/recovery simulator).
	var tl_btns := _find_buttons_by_text(pe, "Timeline")
	_check("found the Timeline tab button", not tl_btns.is_empty())
	if not tl_btns.is_empty():
		(tl_btns[0] as Button).pressed.emit()
		await _frames(6)
		_check("switched to timeline", pe._active_tab == "timeline")
		var texts := _labels_text(pe)
		var has_placeholder := false
		for t in texts:
			if String(t).find("Not built yet") >= 0:
				has_placeholder = true
		_check("Timeline is built: no 'Not built yet' placeholder", not has_placeholder, str(texts))
		_check("Timeline shows the Recorded years section and its empty-state note (fresh world)",
			texts.has("§ RECORDED YEARS") and texts.has("No recorded years yet. Add one below to start the timeline."),
			str(texts))
		_check("Timeline shows the Collapse / recovery simulation section",
			texts.has("§ COLLAPSE / RECOVERY SIMULATION"), str(texts))
		_check("Timeline carries its Add year / Go to year / Run simulation actions",
			_buttons_text(pe).has("Add year") and _buttons_text(pe).has("Go to year")
			and _buttons_text(pe).has("Run simulation"), str(_buttons_text(pe)))
		_check("Economy's Trade section is torn down", not texts.has("§ TRADE"))

	## Timeline -> Political history (built: combined strip, derived lists, journeys, conflicts).
	var ph_btns := _find_buttons_by_text(pe, "Political history")
	_check("found the Political history tab button", not ph_btns.is_empty())
	if not ph_btns.is_empty():
		(ph_btns[0] as Button).pressed.emit()
		await _frames(6)
		_check("switched to political", pe._active_tab == "political")
		var texts2 := _labels_text(pe)
		var has_placeholder2 := false
		for t in texts2:
			if String(t).find("Not built yet") >= 0:
				has_placeholder2 = true
		_check("Political history is built: no 'Not built yet' placeholder", not has_placeholder2, str(texts2))
		_check("Political history shows the combined timeline and says why it is empty (fresh world)",
			texts2.has("§ COMBINED TIMELINE")
			and texts2.any(func(t): return String(t).begins_with("Nothing to plot yet")), str(texts2))
		_check("Political history shows its derived lists, journeys and conflicts sections",
			texts2.has("§ POLITICAL HISTORY") and texts2.has("§ POPULATION & TIER TRAJECTORY")
			and texts2.has("§ AUTHORED EVENTS") and texts2.has("§ JOURNEYS PASSING")
			and texts2.has("§ CONFLICTS HERE"), str(texts2))
		_check("Timeline's own sections are torn down on the switch", not texts2.has("§ RECORDED YEARS"))

	## Political history -> Vault notes.
	var vn_btns := _find_buttons_by_text(pe, "Vault notes")
	_check("found the Vault notes tab button", not vn_btns.is_empty())
	if not vn_btns.is_empty():
		(vn_btns[0] as Button).pressed.emit()
		await _frames(6)
		_check("switched to vault", pe._active_tab == "vault")
		var texts3 := _labels_text(pe)
		var has_knowledge := texts3.has("§ KNOWLEDGE")
		_check("Vault notes tab renders the Knowledge section", has_knowledge)
		## Even with nothing linked, the "no notes" + attach affordance shows.
		var has_no_notes := false
		for t in texts3:
			if String(t).find("No Markdown notes linked") >= 0 or String(t).find("no stable id") >= 0:
				has_no_notes = true
		var has_attach := _buttons_text(pe).has("Attach a Markdown note…") \
			or _buttons_text(pe).has("Linked notes…")
		_check("Vault notes shows the no-notes-linked / attach affordance",
			has_no_notes or has_attach, str(texts3))

	## Back to Overview.
	var ov_btns := _find_buttons_by_text(pe, "Overview")
	_check("found the Overview tab button", not ov_btns.is_empty())
	if not ov_btns.is_empty():
		(ov_btns[0] as Button).pressed.emit()
		await _frames(6)
		_check("switched back to overview", pe._active_tab == "overview")
		_check("Overview content rebuilt (Identity section back)", _labels_text(pe).has("§ IDENTITY"))

	# -- 3. name field edit commits ----------------------------------------------
	var edits := _find_line_edits(pe)
	_check("name field present on Overview", not edits.is_empty())
	if not edits.is_empty():
		var name_edit: LineEdit = edits[0]
		var typed := "PROBE_TAB_RENAME"
		name_edit.text = typed
		name_edit.emit_signal("text_submitted", typed)
		await _frames(8)
		var engine_name := String((_bridge.settlements()[idx] as Dictionary).get("name", ""))
		_check("name field commit reaches the engine", engine_name == typed,
			"engine='%s'" % engine_name)

	# -- 4. trait chip toggle round-trips ----------------------------------------
	var vocab: Array = _bridge.civ_trait_vocabulary()
	if vocab.is_empty():
		_p("no trait vocabulary -- skipping trait round-trip (engine build older than civ_trait_vocabulary())")
	else:
		var first_key := String((vocab[0] as Dictionary).get("key", ""))
		var before_on: bool = (_bridge.civ_settlement_details(idx).get("traits", PackedStringArray()) as PackedStringArray).has(first_key)
		var trait_btns := _buttons_text(pe)
		var target_btn: Button = null
		var all_pe: Array = []
		_walk(pe, all_pe)
		for n in all_pe:
			if n is Button and String((n as Button).text).find(String((vocab[0] as Dictionary).get("label", "~~~")) as String) >= 0:
				target_btn = n
		_check("found the trait chip for '%s'" % first_key, target_btn != null)
		if target_btn != null:
			target_btn.pressed.emit()
			await _frames(8)
			var after_on: bool = (_bridge.civ_settlement_details(idx).get("traits", PackedStringArray()) as PackedStringArray).has(first_key)
			_check("trait toggle round-trips through the engine", after_on != before_on,
				"before=%s after=%s" % [before_on, after_on])
			## Toggle it back so the probe leaves no state behind for a re-run.
			var all_pe2: Array = []
			_walk(pe, all_pe2)
			for n in all_pe2:
				if n is Button and String((n as Button).text).find(String((vocab[0] as Dictionary).get("label", "~~~")) as String) >= 0:
					n.pressed.emit()
			await _frames(8)

	# -- 5. PE-01 regression, reproduced against BOTH teardown paths ------------
	## 5a. The original PE-01 case: re-roll with the name field focused. The
	## reroll changes the engine name directly, then rebuilds while the OLD
	## `_name_edit` (still holding the pre-roll text) is focused -- the
	## `_rebuilding` guard must make its `focus_exited` commit a no-op, not a
	## write of the stale text back over the fresh engine name.
	var ov_btns2 := _find_buttons_by_text(pe, "Overview")
	if not ov_btns2.is_empty():
		(ov_btns2[0] as Button).pressed.emit()
		await _frames(6)
	var edits2 := _find_line_edits(pe)
	if edits2.is_empty():
		_p("ABORT 5a: no name field found on Overview -- PE-01 reroll case not run")
		_fail += 1
	else:
		var ne: LineEdit = edits2[0]
		ne.grab_focus()
		await _frames(4)
		var pre_roll_name := String((_bridge.settlements()[idx] as Dictionary).get("name", ""))
		_check("5a control: name field is focused before the re-roll", ne.has_focus())
		var roll_btns: Array = []
		var all3: Array = []
		_walk(pe, all3)
		for n in all3:
			if n is Button and (n as Button).text == "⟳":
				roll_btns.append(n)
		_check("5a: found the re-roll button", not roll_btns.is_empty())
		if not roll_btns.is_empty():
			(roll_btns[0] as Button).pressed.emit()
			await _frames(8)
			var post_roll_name := String((_bridge.settlements()[idx] as Dictionary).get("name", ""))
			_check("5a PE-01 (reroll-while-focused): engine name changed, not held at the stale focused text",
				post_roll_name != pre_roll_name and post_roll_name != "",
				"pre='%s' post='%s'" % [pre_roll_name, post_roll_name])

	## 5b. The new case this batch could regress: rename with the field
	## focused (NOT submitted), then switch tabs instead of re-rolling. The
	## expected behaviour is the OPPOSITE polarity of 5a's guard -- here the
	## fresh, uncommitted text must reach the engine (via `_commit_focused_field()`
	## called before `_active_tab` moves), not be silently dropped by the same
	## `_rebuilding` guard that protects 5a.
	var edits3 := _find_line_edits(pe)
	if edits3.is_empty():
		_p("ABORT 5b: no name field found on Overview -- tab-switch commit case not run")
		_fail += 1
	else:
		var ne2: LineEdit = edits3[0]
		var old_text := ne2.text
		ne2.grab_focus()
		await _frames(4)
		var sentinel := "PROBE_SWITCH_SENTINEL"
		ne2.text = sentinel
		## Deliberately NOT submitted (no Enter, no focus_exited yet) -- the
		## whole point is that the tab switch itself is what must commit it.
		var econ_btns2 := _find_buttons_by_text(pe, "Economy & notables")
		_check("5b: found the Economy & notables tab button", not econ_btns2.is_empty())
		if not econ_btns2.is_empty():
			(econ_btns2[0] as Button).pressed.emit()
			await _frames(8)
			var engine_after_switch := String((_bridge.settlements()[idx] as Dictionary).get("name", ""))
			_check("5b: the uncommitted rename reached the engine on tab switch, not dropped",
				engine_after_switch == sentinel,
				"typed='%s' old='%s' engine='%s'" % [sentinel, old_text, engine_after_switch])
			_check("5b: the OLD text was not left standing", engine_after_switch != old_text,
				"old='%s' engine='%s'" % [old_text, engine_after_switch])

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
