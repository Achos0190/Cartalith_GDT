extends Node
## Lane B, 2026-09-05: the CIVIL ▸ Religion category rebuilt to
## `design/proposed-2026-09-05/Religion.dc.html`.
##
## Everything below is read **off the drawn nodes** of the real left dock --
## a Label's `text`, a Button's resolved `font_color`, a `ColorRect`'s size --
## and every expected figure is recomputed in this file from
## `get_settlements()`' own dictionaries, never read back out of the shell
## variable that produced it.

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

func _find(lines: Array, needle: String) -> String:
	for l in lines:
		if String(l).find(needle) >= 0:
			return String(l)
	return ""

## Every table row in the drawn body: an HBox whose first child is a 12x12
## ColorRect swatch. Structural, so a prose line that happens to hold a dash
## cannot be mistaken for a dashed cell.
func _table_rows(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is HBoxContainer and c.get_child_count() >= 5:
			var first := c.get_child(0)
			if first is ColorRect and (first as ColorRect).custom_minimum_size == Vector2(12, 12):
				out.append(c)
		_table_rows(c, out)

## The three segment chips, by their drawn text.
func _segments(node: Node, out: Dictionary) -> void:
	for c in node.get_children():
		if c is Button and String(c.text) in ["ADHERENCE", "DIFFUSION", "DIVERGENCE"]:
			out[String(c.text)] = c
		_segments(c, out)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
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
	await get_tree().process_frame

	var places: Array = bridge.settlements()
	print("settlements: ", places.size(), "  status=", ws._religion_status)

	# ---------------- engine truth, recomputed here ----------------
	var counts := {}
	var total := 0
	for p in places:
		var d: Dictionary = p
		total += int(d.get("population", 0))
		var ad: Dictionary = d.get("adherents", {})
		for k in ad.keys():
			counts[String(k)] = int(counts.get(String(k), 0)) + int(ad[k])
	var faith_n := 0
	for k in counts.keys():
		if String(k) != "none" and int(counts[k]) > 0:
			faith_n += 1
	var row_n := 0
	for k in counts.keys():
		if int(counts[k]) > 0:
			row_n += 1
	var none_n := int(counts.get("none", 0))
	## Recomputed from literals this probe carries -- 100 * n / total, one
	## decimal -- rather than by calling `_religion_pct` back.
	var expect_unaff := "%.1f%%" % (100.0 * float(none_n) / float(total)) if total > 0 else "—"
	var top_key := ""
	var top_n := -1
	for k in counts.keys():
		if int(counts[k]) > top_n or (int(counts[k]) == top_n and String(k) < top_key):
			top_n = int(counts[k])
			top_key = String(k)
	var expect_top := "%.1f%%" % (100.0 * float(top_n) / float(total))
	print("engine: total=", total, " faiths(excl none)=", faith_n, " table rows=", row_n,
		" unaffiliated=", none_n, " (", expect_unaff, ")  top=", top_key, " ", expect_top)

	var body: Control = ws._religion_body
	_chk(body != null and is_instance_valid(body), "the Religion category body exists")

	# ---------------- 1. the §6 header count slot ----------------
	var head: Label = ws._religion_head_count
	_chk(head != null and is_instance_valid(head), "a count Label is parented to the header")
	if head != null:
		print("header count reads: '", head.text, "'  parent=", head.get_parent().get_class())
		_chk(head.get_parent() is Button, "and it is a child of the header Button itself")
		_chk(head.text == "%d faith%s" % [faith_n, "" if faith_n == 1 else "s"],
			"the header count is the engine's own faith count (%d)" % faith_n)
		_chk(head.text.find("year") < 0,
			"and it does NOT print a year -- the belief layer is not year-indexed")
		_chk(head.get_theme_color("font_color").is_equal_approx(DccTheme.c("text_faint")),
			"drawn in --faint, the ink §6 gives lmCatCount")

	# ---------------- 2. the three-way segment ----------------
	var segs := {}
	_segments(body, segs)
	print("segments drawn: ", segs.keys())
	_chk(segs.size() == 3, "all three segments are drawn")
	var lit := 0
	for k in segs.keys():
		var col: Color = (segs[k] as Button).get_theme_color("font_color")
		if col.is_equal_approx(DccTheme.c("accent")):
			lit += 1
			print("  lit segment: ", k)
	_chk(lit == 1, "exactly one segment is lit, read off its drawn font_color")

	# ---------------- 3. the totals strip ----------------
	var lines: Array = []
	_harvest(body, lines)
	var strip_ok := _find(lines, "FAITHS") != "" and _find(lines, "ADHERENTS") != "" 		and _find(lines, "UNAFFILIATED") != ""
	_chk(strip_ok, "the three totals captions are drawn")
	## The value Label is the sibling immediately above each caption.
	var vals := {}
	for cap in ["FAITHS", "ADHERENTS", "UNAFFILIATED"]:
		for i in lines.size():
			if String(lines[i]) == cap and i > 0:
				vals[cap] = String(lines[i - 1])
				break
	print("strip values: ", vals)
	_chk(vals.get("FAITHS", "") == "%d" % faith_n,
		"FAITHS shows the engine's faith count (%d)" % faith_n)
	_chk(vals.get("UNAFFILIATED", "") == expect_unaff,
		"UNAFFILIATED shows %s, recomputed here from the adherent counts" % expect_unaff)

	# ---------------- 4. the table, and its two dashed columns ----------------
	var rows: Array = []
	_table_rows(body, rows)
	print("table rows drawn: ", rows.size(), "  expected ", row_n)
	_chk(rows.size() == row_n, "one row per faith present, unaffiliated slot included")
	var dashed_type := 0
	var dashed_trend := 0
	for r in rows:
		if String((r.get_child(2) as Label).text) == "—":
			dashed_type += 1
		if String((r.get_child(4) as Label).text) == "—":
			dashed_trend += 1
	print("TYPE cells dashed: ", dashed_type, "/", rows.size(),
		"   TREND cells dashed: ", dashed_trend, "/", rows.size())
	_chk(dashed_type == rows.size(), "every TYPE cell is dashed -- CIV_RELIGIONS has no type")
	_chk(dashed_trend == rows.size(), "every TREND cell is dashed -- no belief history exists")
	if rows.size() > 0:
		var r0: HBoxContainer = rows[0]
		print("first row: name='", (r0.get_child(1) as Label).text, "' share='",
			(r0.get_child(3) as Label).text, "'")
		_chk(String((r0.get_child(3) as Label).text) == expect_top,
			"the top row's share is %s, recomputed here" % expect_top)
		_chk((r0.get_child(1) as Label).get_theme_color("font_color").is_equal_approx(
			DccTheme.c("text_bright")), "the top row's name is inked --ink")
	_chk(_find(lines, "SHARE ▾") != "", "the sorted column carries the caret by default")
	_chk(_find(lines, "TYPE") != "" and _find(lines, "TREND") != "",
		"TYPE and TREND are drawn as columns rather than deleted")

	# ---------------- 5. sorting by FAITH ----------------
	var faith_head: Button = null
	for c in body.find_children("*", "Button", true, false):
		if String((c as Button).text).begins_with("FAITH"):
			faith_head = c
	_chk(faith_head != null, "the FAITH heading is pressable")
	if faith_head != null:
		faith_head.pressed.emit()
		await get_tree().process_frame
		var rows2: Array = []
		_table_rows(ws._religion_body, rows2)
		var names := []
		for r in rows2:
			names.append(String((r.get_child(1) as Label).text))
		var sorted_names := names.duplicate()
		sorted_names.sort_custom(func(a, b): return String(a).naturalnocasecmp_to(String(b)) < 0)
		print("after FAITH press, drawn order: ", names)
		_chk(names == sorted_names, "the rows re-order alphabetically by the printed label")
		var lines2: Array = []
		_harvest(ws._religion_body, lines2)
		_chk(_find(lines2, "FAITH ▾") != "" and _find(lines2, "SHARE ▾") == "",
			"the caret moved to FAITH and left SHARE")
		ws._religion_set_sort("share")
		await get_tree().process_frame

	# ---------------- 6. the other two panes ----------------
	ws._religion_set_view("diffusion")
	await get_tree().process_frame
	var dl: Array = []
	_harvest(ws._religion_body, dl)
	_chk(_find(dl, "Years") != "" and _find(dl, "Run diffusion") != "",
		"DIFFUSION draws the years field and the run button")
	ws._religion_set_view("divergence")
	await get_tree().process_frame
	var vl: Array = []
	_harvest(ws._religion_body, vl)
	_chk(_find(vl, "Show on map") != "", "DIVERGENCE draws the ring's toggle")
	_chk(_find(vl, "Years") == "", "and does not also draw DIFFUSION's controls")
	ws._religion_set_view("adherence")
	await get_tree().process_frame

	# ---------------- 7. the footnote says what the code does ----------------
	var al: Array = []
	_harvest(ws._religion_body, al)
	var foot := _find(al, "Adherence is a result")
	print("footnote: '", foot, "'")
	_chk(foot != "", "the footnote's true half is kept verbatim")
	_chk(foot.find("not year-indexed") >= 0,
		"and it states the belief layer is not year-indexed")
	_chk(_find(al, "re-reads on the Timeline") == "",
		"the artboard's false claim is NOT shipped")
	var year_before: int = bridge.get_civ_year()
	ws._religion_run()
	await get_tree().process_frame
	print("civ year before run=", year_before, "  after=", bridge.get_civ_year())
	_chk(bridge.get_civ_year() == year_before,
		"running the diffusion never writes to the Timeline's year cursor")

	# ---------------- 8. §6's floor rule survives ----------------
	var relig_btn: Button = null
	var lm_body: Control = ws._landmarks_body
	for e in ws.categories:
		if String((e as Dictionary).get("title", "")) == "Religion":
			relig_btn = (e as Dictionary).get("button")
	relig_btn.pressed.emit()   ## open Religion
	await get_tree().process_frame
	relig_btn.pressed.emit()   ## re-click the OPEN header -> must fall to Landmarks
	await get_tree().process_frame
	var open_n := 0
	for e in ws.categories:
		if ((e as Dictionary).get("body") as Control).visible:
			open_n += 1
	print("after re-clicking the open Religion header: open categories=", open_n,
		"  landmarks visible=", lm_body.visible)
	_chk(open_n == 1 and lm_body.visible,
		"Landmarks is still the floor -- CIVIL never collapses to nothing")

	# ---------------- 9. width, over three worlds ----------------
	## `MISTAKES.md`: a panel width is content-dependent and one world is one
	## sample. The table's columns are fixed-width and its FAITH cell is an
	## un-wrapped mono Label, so its minimum is load-bearing -- a
	## `ScrollContainer` with the horizontal axis DISABLED folds a child's
	## minimum into its own, which is how the left dock grew to swallow the map
	## twice before. Measured against the LAPTOP dock width, the narrow one.
	for seed_v in [77021, 483920, 4242]:
		bridge.generate({
			"seed": seed_v, "width_km": 2000.0, "grid_w": 192, "grid_h": 128,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.6).timeout
		bridge.civ_set_faction_field(1, "religion", "sun_cult")
		bridge.civ_set_faction_field(2, "religion", "sea_lords")
		ws._religion_years = 60
		ws._religion_run()
		await get_tree().process_frame
		var w: float = ws._religion_body.get_combined_minimum_size().x
		print("seed ", seed_v, ": Religion body combined minimum width = ", w, " px")
		_chk(w <= float(DccTheme.W_LEFT_DOCK_MIN),
			"seed %d: the body fits the narrowest left dock (%d px)"
			% [seed_v, DccTheme.W_LEFT_DOCK_MIN])

	print("\nRESULT: ", "ALL PASS" if _fails == 0 else "%d FAIL" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
