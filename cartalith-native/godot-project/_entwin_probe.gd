extends Node
## Verifies the 2026-09-05 derive-from-canvas pass on the three entity windows
## nobody drew: `faction_roster_window.gd`, `place_editor_window.gd`,
## `city_viewer_window.gd`.
##
## Every assertion reads the **live scene tree** rather than the source, because
## the claims are about what is on screen: that a legend row now carries a name
## and not a paragraph, that the sentence survived into the row's tooltip, that
## the section header is a noun phrase, that the re-roll button has the canvas's
## own box.
##
## Deliberately **not** a pixel probe, so `--headless` is safe: nothing here
## calls `get_image()` (MISTAKES -- `ImageTexture.update()` is a no-op headless).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _entwin_probe.tscn
##
## No flags; it reads none.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("EWN %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_labels(c, out)

func _texts(root: Node) -> Array:
	var out: Array = []
	_labels(root, out)
	return out

## Every `tooltip_text` in a subtree — the legend's sentences live here now.
func _tips(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Control and String((c as Control).tooltip_text) != "":
			out.append(String((c as Control).tooltip_text))
		_tips(c, out)

func _tip_texts(root: Node) -> Array:
	var out: Array = []
	_tips(root, out)
	return out

func _any_contains(arr: Array, needle: String) -> bool:
	for s in arr:
		if String(s).findn(needle) >= 0:
			return true
	return false


func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("EWN world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("EWN  !! generate failed")
		get_tree().quit(1)
		return

	var places: Array = app.bridge.settlements()
	if places.is_empty():
		print("EWN  !! no settlement")
		get_tree().quit(1)
		return

	# -- City viewer ------------------------------------------------------------
	## Pick a settlement that actually produces a layout: the legend only draws
	## over one, and asserting on an empty panel would pass vacuously.
	var cv_index := -1
	for i in mini(places.size(), 12):
		if app.bridge.urban_layouts(PackedInt32Array([i])).size() > 0:
			cv_index = i
			break
	_check("a settlement with a layout exists to open", cv_index >= 0,
		"scanned %d" % mini(places.size(), 12))
	if cv_index >= 0:
		app.open_city_viewer(cv_index)
		await _frames(6)
		var cv = app.city_viewer_window
		var cvt := _texts(cv)
		var cvtips := _tip_texts(cv)

		_check("CV1: the legend row is a NAME, not a sentence",
			cvt.has("Rooftops"), "labels=%s" % [cvt.slice(0, 14)])
		_check("CV2: the sentence is gone from the row's own text",
			not _any_contains(cvt, "one per building footprint"),
			"still a label")
		_check("CV3: the sentence survived into a tooltip",
			_any_contains(cvtips, "one per building footprint"),
			"tips=%d" % cvtips.size())
		_check("CV4: the section header is the noun phrase",
			cvt.has("§ STAGES"), "headers=%s" % [cvt.filter(func(t): return String(t).begins_with("§ "))])
		_check("CV5: the question-shaped header is gone",
			not cvt.has("§ WHAT PRODUCED THIS"))
		## The engine's own stage list still reaches the section it was renamed
		## in -- a rename that lost its body would pass CV4 and mean nothing.
		## Counted, not just found: `_rebuild_side` joins one `· ` line per entry
		## of the engine's `stages` array into a single `Label`, and a truncated
		## list would still contain a `· `.
		##
		## **Ten, not twenty-nine.** `urban_bridge.rs` builds `stages` as ten
		## entries — the stages whose output this window draws — whose own first
		## line carries "all 29 reference stages". This probe asserted `>= 20`
		## first and measured 10, which is how the window's note came to be
		## corrected: it claimed every line was one of the 29.
		##
		## **Twelve since `buildDetails`, `buildHarbour` and
		## `detectRiverCrossings` joined the list** (`urban_bridge.rs::
		## layout_dict`, re-counted 2026-09-24). The window's note now reads the
		## count off the array instead of hard-coding it, so this pins the
		## engine's list and checks the note quotes the same number.
		var stage_lines := 0
		var stage_head := ""
		for t in cvt:
			var s := String(t)
			if s.begins_with("· "):
				var lines := s.split("\n")
				stage_lines = lines.size()
				stage_head = String(lines[0])
				break
		_check("CV6: the stages list still draws under it", stage_lines == 12,
			"%d stage lines" % stage_lines)
		var count_quoted := false
		for t in cvt:
			if String(t).begins_with("%d lines above" % stage_lines):
				count_quoted = true
				break
		_check("CV6c: the note under it quotes the list's own length",
			count_quoted, "%d" % stage_lines)
		_check("CV6b: and its first line is the one that carries the 29",
			stage_head.find("29") >= 0, "head='%s'" % stage_head)

		## The legend swatch is a `Panel` with a rounded stylebox, not a square
		## `ColorRect`. Read off the node, since the radius is the derived part.
		var sw := _first_legend_swatch(cv)
		_check("CV7: the swatch is a rounded Panel at the canvas's 10x10",
			sw != null and sw.custom_minimum_size == Vector2(10, 10),
			"node=%s size=%s" % [sw, sw.custom_minimum_size if sw != null else Vector2.ZERO])
		if sw != null:
			var sb := sw.get_theme_stylebox("panel")
			_check("CV8: its corner radius is the canvas's 3",
				sb is StyleBoxFlat and (sb as StyleBoxFlat).corner_radius_top_left == 3,
				"radius=%d" % ((sb as StyleBoxFlat).corner_radius_top_left if sb is StyleBoxFlat else -1))
		cv.hide()
		await _frames(2)

	# -- Place editor -----------------------------------------------------------
	app.open_place_editor(0)
	await _frames(6)
	var pe = app.place_editor_window
	var roll := _find_button(pe, "⟳")
	_check("PE1: the re-roll button carries the canvas's 44 x 42 box",
		roll != null and roll.custom_minimum_size == Vector2(44, 42),
		"size=%s" % [roll.custom_minimum_size if roll != null else Vector2.ZERO])
	pe.hide()
	await _frames(2)

	# -- Faction roster ---------------------------------------------------------
	var fr = app.faction_roster_window
	fr.open()
	await _frames(6)
	var roster: Array = app.bridge.get_factions()
	if not roster.is_empty():
		var first: Dictionary = roster[0]
		fr._selected = int(first.get("id", 1))
		var sub := String(fr._phone_bar_sub_text(first))
		_check("FR1: the folded bar's sub names a position and a count",
			sub.begins_with("1 of %d · " % roster.size()) and sub.ends_with("settlements") \
				or sub.begins_with("1 of %d · " % roster.size()) and sub.ends_with("settlement"),
			"sub='%s'" % sub)
		## The position is looked up, not derived from the id. A faction whose
		## id is not in the roster has no position, so the clause must be
		## ABSENT rather than defaulted -- the "omit, do not fake" rule.
		fr._selected = 99999
		var orphan := String(fr._phone_bar_sub_text(first))
		_check("FR2: an id not in the roster prints no position at all",
			orphan.find(" of ") < 0 and orphan.ends_with("settlements") \
				or orphan.find(" of ") < 0 and orphan.ends_with("settlement"),
			"orphan='%s'" % orphan)
		_check("FR3: an empty faction prints nothing rather than a placeholder",
			fr._phone_bar_sub_text({}) == "",
			"got='%s'" % fr._phone_bar_sub_text({}))
	fr.hide()

	print("EWN %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## The first `Panel` under the city viewer's legend column.
func _first_legend_swatch(cv) -> Panel:
	return _find_panel(cv._legend)

func _find_panel(n: Node) -> Panel:
	for c in n.get_children():
		if c is Panel:
			return c
		var deeper := _find_panel(c)
		if deeper != null:
			return deeper
	return null

func _find_button(n: Node, text: String) -> Button:
	for c in n.get_children():
		if c is Button and String((c as Button).text) == text:
			return c
		var deeper := _find_button(c, text)
		if deeper != null:
			return deeper
	return null
