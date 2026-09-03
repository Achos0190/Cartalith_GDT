extends Node
## VERIFIER (batch 20): three surfaces, one settlement, no disagreement.
##
## Lane A shipped the CIVIL > Religion panel and the map hover card; Lane B
## shipped the right dock's Faith row. All three describe the same settlement
## and are written in three files. This asserts they agree on the faith, on
## whether a share exists, and on the share itself where one does -- and that
## no surface anywhere in a live world turns a real congregation into 0%.

var app: Node
var fail := 0

func _chk(name: String, ok: bool, detail: String = "") -> void:
	print("XL %s  %s%s" % ["ok  " if ok else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not ok:
		fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		_collect(c, out)

func _dock_texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _panel_texts(ws: Control) -> Array:
	var host := VBoxContainer.new()
	add_child(host)
	ws._fill_religion(host)
	var out: Array = []
	_collect(host, out)
	remove_child(host)
	host.queue_free()
	return out

## The line printed immediately after the row whose text starts with `head`.
func _after(lines: Array, head: String) -> String:
	for i in lines.size():
		if String(lines[i]).begins_with(head):
			return String(lines[i + 1]) if i + 1 < lines.size() else ""
	return "<row not found>"

func _value_after(lines: Array, key: String) -> String:
	for i in lines.size():
		if String(lines[i]) == key:
			return String(lines[i + 1]) if i + 1 < lines.size() else ""
	return "<field not found>"

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("XL WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	var ws: Control = null
	for w in app._workspaces:
		if w.name == "CivilizationWorkspace":
			ws = w
	if ws == null:
		print("XL FAIL: no CivilizationWorkspace")
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
	bridge.civ_set_faction_field(3, "religion", "old_gods")
	ws._religion_years = 50
	ws._religion_run()
	await get_tree().create_timer(0.5).timeout

	var places: Array = bridge.settlements()
	var empty_idx := -1
	var pop_idx := -1
	var n_empty_faith := 0
	for i in places.size():
		var d: Dictionary = places[i]
		if not d.has("religion"):
			continue
		var pop := int(d.get("population", 0))
		var key := String(d["religion"])
		if pop <= 0 and key != "none":
			n_empty_faith += 1
			if empty_idx < 0:
				empty_idx = i
		if pop > 0 and pop_idx < 0:
			pop_idx = i
	_chk("X0 premise: the fixture has a population-0 settlement led by a faith",
		empty_idx >= 0, "n=%d of %d" % [n_empty_faith, places.size()])
	_chk("X0b premise: and a populated one to compare against", pop_idx >= 0)
	if empty_idx < 0 or pop_idx < 0:
		print("XL RESULT: PREMISE FAILED fail=%d" % fail)
		get_tree().quit(1 if fail > 0 else 0)
		return

	var panel := _panel_texts(ws)
	var ov = app.viewport.overlay
	var rd = app.right_dock_ctrl

	# ---- the population-0 settlement, on all three surfaces ----------------
	var e: Dictionary = places[empty_idx]
	var ename := String(e["name"])
	var ekey := String(e["religion"])
	var elabel: String = ws._religion_label(ekey)
	## `_religion_row`'s own name line is `"%s · %s people"`, so the middle dot
	## is part of the key -- without it a bare name prefix also matches the
	## divergence list and reads the wrong settlement's detail line.
	var panel_e := _after(panel, ename + " · ")
	var card_e: Array = ov._faith_lines(e)
	rd.on_settlement_selected(e, empty_idx)
	await _frames(4)
	var dock_e := _dock_texts()
	var dock_e_faith := _value_after(dock_e, "Faith")
	var dock_e_note := ""
	for t in dock_e:
		if String(t).begins_with("— no adherents to count"):
			dock_e_note = String(t)

	print("XL   pop-0 settlement %s religion=%s pop=%d adherents=%s"
		% [ename, ekey, int(e.get("population", 0)), e.get("adherents", {})])
	print("XL     panel : %s" % panel_e)
	print("XL     dock  : Faith=%s | %s" % [dock_e_faith, dock_e_note])
	print("XL     card  : %s" % [card_e])

	_chk("X1 the panel row names the faith the engine gave it",
		panel_e.find(elabel) >= 0, "row=%s expected label=%s" % [panel_e, elabel])
	_chk("X2 the dock Faith row names the same faith",
		dock_e_faith == elabel, "dock=%s panel label=%s" % [dock_e_faith, elabel])
	_chk("X3 the hover card names the same faith",
		String(card_e[0]).to_lower().find(elabel.to_lower()) >= 0,
		"card head=%s" % String(card_e[0]))
	_chk("X4 no surface prints a share for a settlement with nobody in it",
		panel_e.find("%") < 0 and String(card_e[0]).find("%") < 0
		and dock_e_note != "" and dock_e_note.find("%") < 0,
		"panel=%s card=%s docknote=%s" % [panel_e, String(card_e[0]), dock_e_note])
	_chk("X5 and all three say WHY there is none, rather than going silent",
		panel_e.to_lower().find("no population") >= 0
		and String(card_e[0]).to_lower().find("no population") >= 0
		and dock_e_note.to_lower().find("population") >= 0,
		"panel=%s card=%s dock=%s" % [panel_e, String(card_e[0]), dock_e_note])

	# ---- the populated settlement ------------------------------------------
	var p: Dictionary = places[pop_idx]
	var pname := String(p["name"])
	var ppop := int(p["population"])
	var pad: Dictionary = p.get("adherents", {})
	var pkey := String(p["religion"])
	var plabel: String = ws._religion_label(pkey)
	var panel_p := _after(panel, pname + " · ")
	var card_p: Array = ov._faith_lines(p)
	rd.on_settlement_selected(p, pop_idx)
	await _frames(4)
	var dock_p := _dock_texts()
	var dock_p_faith := _value_after(dock_p, "Faith")
	## Independent expected share string, computed here from `adherents` and
	## `population`, not read off either surface.
	var want_pct: String = ws._religion_pct(int(pad.get(pkey, 0)), ppop)
	print("XL   populated %s religion=%s pop=%d adherents=%s"
		% [pname, pkey, ppop, pad])
	print("XL     panel : %s" % panel_p)
	print("XL     dock  : Faith=%s" % dock_p_faith)
	print("XL     card  : %s" % [card_p])
	_chk("X6 the dock names the same faith as the panel for a populated town",
		dock_p_faith == plabel, "dock=%s expected=%s" % [dock_p_faith, plabel])
	_chk("X7 the panel row carries the share this settlement own numbers give",
		panel_p.find(plabel) >= 0 and panel_p.find(want_pct) >= 0,
		"row=%s wanted %s %s" % [panel_p, plabel, want_pct])
	var dock_row := ""
	for t in dock_p:
		if String(t).find(plabel) >= 0 and String(t).find("people") >= 0:
			dock_row = String(t)
	_chk("X8 and the dock adherence row carries the SAME share string",
		dock_row.find(want_pct) >= 0,
		"dockrow=%s wanted %s" % [dock_row, want_pct])

	# ---- world-wide: no real congregation is rendered as zero --------------
	var zero_panel := 0
	for t in panel:
		var s := String(t)
		if s.find("(0.0%)") >= 0 or s.find(" 0.0%") >= 0:
			zero_panel += 1
	var zero_card := 0
	var real_small := 0
	for d in places:
		if not (d as Dictionary).has("religion"):
			continue
		var ad: Dictionary = (d as Dictionary).get("adherents", {})
		var pp := int((d as Dictionary).get("population", 0))
		for k in ad.keys():
			if int(ad[k]) > 0 and pp > 0 and 100.0 * float(ad[k]) / float(pp) < 0.5:
				real_small += 1
		for line in ov._faith_lines(d):
			if String(line).find(" 0%") >= 0:
				zero_card += 1
	_chk("X9 premise: the world contains a congregation too small to round",
		real_small > 0, "n=%d" % real_small)
	_chk("X10 no panel row renders a real congregation as 0.0 percent",
		zero_panel == 0, "%d rows" % zero_panel)
	_chk("X11 no hover card renders a real congregation as 0 percent",
		zero_card == 0, "%d lines" % zero_card)

	print("XL RESULT fail=%d" % fail)
	get_tree().quit(1 if fail > 0 else 0)
