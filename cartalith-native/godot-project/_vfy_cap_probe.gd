extends Node
## VERIFIER capability probe (2026-09-05, independent of the three lanes).
##
## The ruling was DERIVE, not rewrite. This asks the only question that matters
## after a chrome pass: does each of the six surfaces still DO what it did?
## Every check drives a real public entry point (`app.open_*`, the window's own
## signal handlers, `DccWidgets.action`'s `Button.pressed`) and reads state back
## off the engine or the live tree -- never a comment.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_cap_probe.tscn
##
## No flags: the body parses none, so none are documented. Headless is sound
## because nothing reads a rendered pixel -- the city-viewer check reads
## `_layout` (the engine's own dict) rather than sampling the framebuffer, and
## `ImageTexture.update()` being a no-op under `--headless` therefore does not
## reach any assertion here.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("VFY %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_labels(c, out)

func _find(n: Node, cls: String, out: Array) -> void:
	for c in n.get_children():
		if c.is_class(cls):
			out.append(c)
		_find(c, cls, out)


func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var w := 0
	while app.bridge.generating and w < 2400:
		await get_tree().process_frame
		w += 1
	print("VFY world: has_world=%s settlements=%d" % [app.bridge.has_world, app.bridge.settlements().size()])
	await _frames(8)
	if not app.bridge.has_world:
		get_tree().quit(1)
		return

	# ---- 1. world data window: tabs, filter, sort ---------------------------
	app.open_world_data("Settlements")
	await _frames(6)
	var wd = app.world_data_window
	var all_rows: Array = []
	_labels(wd._settlements_body, all_rows)
	_ck("WD1: the Settlements tab draws rows", all_rows.size() > 6,
		"%d labels" % all_rows.size())
	var prov: Array = []
	_labels(wd._provinces_body, prov)
	var trade: Array = []
	_labels(wd._trade_body, trade)
	_ck("WD2: the Provinces tab draws rows", prov.size() > 3, "%d labels" % prov.size())
	_ck("WD3: the Economy tab draws rows", trade.size() > 3, "%d labels" % trade.size())

	var s_all: Array = app.bridge.settlements()
	var by_name := {}
	for s in s_all:
		by_name[String((s as Dictionary).get("name", ""))] = int((s as Dictionary).get("population", 0))
	var pops: Array = []
	for t in all_rows:
		if by_name.has(t):
			pops.append(by_name[t])
	var desc := true
	for i in range(1, pops.size()):
		if int(pops[i]) > int(pops[i - 1]):
			desc = false
			break
	_ck("WD4: still sorted by population descending", desc and pops.size() > 3,
		"first 6 = %s" % [pops.slice(0, 6)])

	var les: Array = []
	_find(wd, "LineEdit", les)
	_ck("WD5: the filter field exists", les.size() >= 1, "%d LineEdit" % les.size())
	var before_n := 0
	for t in all_rows:
		if by_name.has(t):
			before_n += 1
	if les.size() >= 1:
		var le: LineEdit = les[0]
		## A real settlement name, not `all_rows[1]` -- row 0 of the body is the
		## column HEADER, whose cells ("name", "pop") match no settlement.
		var target := ""
		for t in all_rows:
			if by_name.has(t) and String(t).length() >= 5:
				target = String(t)
				break
		var frag := target.substr(0, 5).to_lower()
		le.text = frag
		le.text_changed.emit(frag)
		await _frames(6)
		var after: Array = []
		_labels(wd._settlements_body, after)
		var kept := 0
		var all_match := true
		for t in after:
			if by_name.has(t):
				kept += 1
				if String(t).to_lower().find(frag) < 0:
					all_match = false
		_ck("WD6: the filter narrows the table", kept > 0 and kept < before_n,
			"'%s': %d -> %d settlement rows" % [frag, before_n, kept])
		_ck("WD7: every surviving row matches the filter", all_match)
		le.text = ""
		le.text_changed.emit("")
		await _frames(4)
		var restored: Array = []
		_labels(wd._settlements_body, restored)
		var back := 0
		for t in restored:
			if by_name.has(t):
				back += 1
		_ck("WD8: clearing the filter restores every row", back == before_n,
			"%d back of %d" % [back, before_n])
	wd.hide()
	await _frames(2)

	# ---- 2. faction roster: add / remove ------------------------------------
	app.faction_roster_window.open()
	await _frames(6)
	var fr = app.faction_roster_window
	var n0: int = app.bridge.get_factions().size()
	fr._add_faction()
	await _frames(6)
	var n1: int = app.bridge.get_factions().size()
	_ck("FR-A: + Add faction adds one", n1 == n0 + 1, "%d -> %d" % [n0, n1])
	var rlab: Array = []
	_labels(fr, rlab)
	_ck("FR-B: the roster redraws after an add", rlab.size() > 4, "%d labels" % rlab.size())
	var removed: bool = app.bridge.civ_remove_faction()
	await _frames(6)
	var n2: int = app.bridge.get_factions().size()
	_ck("FR-C: removing a faction removes one", removed and n2 == n1 - 1,
		"%d -> %d" % [n1, n2])
	fr.hide()
	await _frames(2)

	# ---- 3. place editor: save ----------------------------------------------
	var pe = app.place_editor_window
	pe.open_for(0)
	await _frames(6)
	var typed := "VFY_SAVE_CHECK"
	pe._apply({"name": typed})
	await _frames(6)
	_ck("PE-A: an edit still reaches the engine",
		String((app.bridge.settlements()[0] as Dictionary).get("name", "")) == typed)
	var btns: Array = []
	_find(pe, "Button", btns)
	var roll: Button = null
	for b in btns:
		if (b as Button).text.strip_edges() == "⟳":
			roll = b
	_ck("PE-B: the re-roll button is still present and connected",
		roll != null and roll.pressed.get_connections().size() > 0,
		"box=%s" % [roll.size if roll != null else Vector2.ZERO])
	if roll != null:
		roll.pressed.emit()
		await _frames(6)
		var after_roll := String((app.bridge.settlements()[0] as Dictionary).get("name", ""))
		_ck("PE-C: pressing it re-rolls the name", after_roll != typed and after_roll != "",
			"'%s' -> '%s'" % [typed, after_roll])
	pe.hide()
	await _frames(2)

	# ---- 4. city viewer: a plan is still drawn -------------------------------
	var found := -1
	for i in mini(24, app.bridge.settlements().size()):
		var got: Array = app.bridge.urban_layouts(PackedInt32Array([i]))
		if got.size() > 0 and not (got[0] as Dictionary).is_empty():
			found = i
			break
	_ck("CV-A: some settlement has a layout", found >= 0, "index %d" % found)
	if found >= 0:
		app.open_city_viewer(found)
		await _frames(8)
		var cv = app.city_viewer_window
		_ck("CV-B: the viewer holds the layout", not cv._layout.is_empty(),
			"keys=%d" % cv._layout.size())
		var nb: int = (cv._layout.get("buildings", []) as Array).size()
		_ck("CV-C: the plan still has a town in it", nb > 0, "%d building footprints" % nb)
		var leg: Array = []
		_labels(cv._legend, leg)
		_ck("CV-D: the legend still names every band", leg.size() >= 8,
			"%d legend labels" % leg.size())
		var inf: Array = []
		_labels(cv._info, inf)
		_ck("CV-E: the side column still reads out", inf.size() >= 10, "%d labels" % inf.size())
		cv.hide()
		await _frames(2)

	# ---- 5. generation info: it still dumps ---------------------------------
	app.open_gen_info()
	await _frames(6)
	var gi = app.gen_info_dialog
	_ck("GI-A: the dump is non-empty", gi._text.text.length() > 200,
		"%d chars, %d lines" % [gi._text.text.length(), gi._text.get_line_count()])
	_ck("GI-B: it is still read-only", not gi._text.editable)
	var gbtn: Array = []
	_find(gi, "Button", gbtn)
	var copy: Button = null
	for b in gbtn:
		if (b as Button).text.find("Copy") >= 0:
			copy = b
	_ck("GI-C: the Copy button survived the action() swap",
		copy != null and copy.pressed.get_connections().size() > 0)
	if copy != null:
		DisplayServer.clipboard_set("")
		copy.pressed.emit()
		await _frames(3)
		if DisplayServer.has_feature(DisplayServer.FEATURE_CLIPBOARD):
			_ck("GI-D: pressing it still copies the dump",
				DisplayServer.clipboard_get() == gi._text.text,
				"%d chars on the clipboard" % DisplayServer.clipboard_get().length())
		else:
			print("VFY --    GI-D skipped: no clipboard on this display server (run windowed)")
	gi.hide()
	await _frames(2)

	# ---- 6. performance window: gone ----------------------------------------
	## PW-A and PW-B are retired, not skipped: `LARGE_ITEM_RULINGS.md` ruling 19 (2026-09-06) folded `performance_window.gd`
## away -- no diagnostics window exists in this design language.
	## What that window read out now lives on the `Preferences` rows
	## `_perfwin_probe.gd` checks.

	print("VFY %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
