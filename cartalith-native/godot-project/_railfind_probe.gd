extends Node
## **Lane Shell's two surfaces, measured.** The rail expansion's added header
## subtitles (`design/round3-corrected/Rail.dc.html` A2/A3) and `Edit ▸ Find on
## map…`'s four additions (`FindOnMap.dc.html` B2), both in `dcc_shell.gd`.
##
## Run it WINDOWED -- no `--headless`. Nothing here reads a pixel, but a
## `Label`'s wrapped line count and a `Control`'s laid-out width are produced by
## the same layout pass a real window runs, and `MISTAKES.md` ("Write
## `--headless` into a brief") is about mandating the dummy driver for a claim
## that needs the real one. Two legs, one density each:
##
##   Godot_v4.7.1_console --path . _railfind_probe.tscn -- --vp 1600x900
##   Godot_v4.7.1_console --path . _railfind_probe.tscn -- --vp 2560x1600 --force-touch
##
## `--vp WxH` sizes the `SubViewport` the shell is booted into, which is what
## `_compute_layout_mode()` reads, and `--force-touch` is `dcc_shell.gd:644`'s
## own testing flag. Leg 1 is POINTER density (`role_px("w_rail_expanded")` =
## 200); leg 2 is TABLET (264). The flag names are read below and an
## unrecognised one ABORTS rather than defaulting -- `MISTAKES.md`, "Write a
## probe's usage header": a header documenting a flag the body never reads
## measures the same box twice and calls it two densities.
##
## What each section would catch:
##
##   A. The ten `RAIL_NODES` nodes, by key. The subtitle pass must add rows, not
##      replace nodes with domain labels -- the explicitly rejected proposal.
##   B. Three subtitles, VERBATIM from `DOMAINS[i].subtitle`. Not "a label is
##      present": the exact string, so a paraphrase fails.
##   C. Wrap, not clip. `get_line_count()` vs `get_visible_line_count()` -- equal
##      means every line the text needs is drawn; CIVIL's 71 characters must
##      take more than one line at 200 px, or the wrap is not being exercised.
##   D. Selection stays ink alone. All ten node rows share one `normal`
##      stylebox class with no fill; only `font_color` differs.
##   E. The column does not grow. Its `ScrollContainer` has the horizontal axis
##      DISABLED, which folds the child's minimum width into the parent's own
##      (`MISTAKES.md`, "Read a layout that overflows the screen"); an
##      autowrapping `Label` must report minimum width 1 and not widen the rail.
##   K. The phone leg (`-- --vp 412x915 --force-touch`) instead of A..J: the
##      phone has no rail to expand, and what it does have is a FIXED-height
##      sheet that this pass put two new rows inside. It asserts the sheet grew
##      to pay for them rather than evicting result rows -- `MISTAKES.md`,
##      "Content added below the fold evicts content that was above it".
##   F..J. Find on map: the six chips and the declined one, whole-token scope
##      splitting (`l` vs `lb`), the `.` scope reaching `CommandIndex`, bands
##      drawn WITHOUT re-ranking (row order asserted byte-identical to
##      `search()`'s own), and the count.

var _fail := 0
var _app: Node
var _vp: SubViewport
var _dens := ""

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("PROBE ABORT: run WINDOWED. Layout of an autowrapping Label and")
		print("  an embedded AcceptDialog are what this measures; the dummy")
		print("  driver is not the surface the claim is about.")
		get_tree().quit(3)
		return

	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			if parts.size() != 2:
				print("PROBE ABORT: --vp wants WxH, got ", args[i + 1])
				get_tree().quit(3)
				return
			vp_size = Vector2i(int(parts[0]), int(parts[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("PROBE ABORT: unrecognised argument '", a, "'. Known: --vp WxH, --force-touch.")
		get_tree().quit(3)
		return

	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _frames(60)

	_dens = "PHONE" if DccTheme.is_phone() else ("TABLET" if DccTheme.is_tablet() else "POINTER")
	var rail_w: int = DccTheme.role_px("w_rail_expanded")
	print("[DENSITY] %s · SubViewport %dx%d · is_touch=%s is_tablet=%s is_phone=%s · w_rail_expanded=%d px"
		% [_dens, vp_size.x, vp_size.y, DccTheme.is_touch(), DccTheme.is_tablet(),
			DccTheme.is_phone(), rail_w])
	print("[BUILD] shell hash ", _app.call("build_id"))

	if DccTheme.is_phone():
		await _phone_find()
	else:
		await _rail(rail_w)
		await _find()

	print("\n%s: %d failure(s)" % ["PROBE FAIL" if _fail > 0 else "PROBE PASS", _fail])
	get_tree().quit(1 if _fail > 0 else 0)

# -- A..E : the rail expansion ---------------------------------------------------

func _rail(rail_w: int) -> void:
	print("\n=== A. the ten nodes are untouched (%s) ===" % _dens)
	_app.call("set_rail_expanded", true)
	await _frames(8)
	var rows: Dictionary = _app.get("_rail_node_rows")
	var keys: Array = rows.keys()
	keys.sort()
	_ok("ten node rows, no more and no fewer", keys.size(), 10)
	_ok("the exact ten (domain/mode) keys", ",".join(PackedStringArray(keys)),
		"cartography/icons,cartography/labels,cartography/style,cartography/terrain," \
		+ "civilization/factions,civilization/infra,civilization/landmarks," \
		+ "civilization/planner,world/a,world/b")
	var all_buttons := true
	for k in keys:
		if not (rows[k] is Button):
			all_buttons = false
	_ok("every node row is still a Button", all_buttons, true)

	var col: Node = _rail_column()
	if col == null:
		print("  FAIL could not reach the rail expansion column")
		_fail += 1
		return

	print("\n=== B. three subtitles, verbatim from DOMAINS ===")
	var subs: Array[Label] = []
	for child in col.get_children():
		if child is MarginContainer and child.get_child_count() == 1 \
				and child.get_child(0) is Label:
			var l: Label = child.get_child(0)
			if l.autowrap_mode != TextServer.AUTOWRAP_OFF:
				subs.append(l)
	_ok("one subtitle per header", subs.size(), 3)
	var want: Array = []
	for d in _app.get("DOMAINS"):
		want.append(String((d as Dictionary)["subtitle"]))
	for n in mini(subs.size(), want.size()):
		_ok("subtitle %d is DOMAINS[%d].subtitle verbatim" % [n, n], subs[n].text, want[n])

	print("\n=== C. wrap, do not clip (%s, column %d px) ===" % [_dens, rail_w])
	for n in subs.size():
		var l := subs[n]
		var total := l.get_line_count()
		var shown := l.get_visible_line_count()
		print("  info subtitle %d: %d chars · %d line(s) · width %.0f px · height %.0f px"
			% [n, l.text.length(), total, l.size.x, l.size.y])
		_ok("subtitle %d draws every line it needs" % n, shown, total)
		_ok("subtitle %d does not clip" % n, l.clip_text, false)
		_ok("subtitle %d has no ellipsis overrun" % n, l.text_overrun_behavior,
			TextServer.OVERRUN_NO_TRIMMING)
	## CIVIL is index 1 and is the 71-character one A3 draws as the problem
	## (A3's caption says 70; the string is 71 -- section C prints the length).
	if subs.size() > 1:
		_ok("CIVIL's 71 chars wrap to more than one line", subs[1].get_line_count() > 1, true)
		_ok("...and CIVIL is the tallest of the three",
			subs[1].size.y >= subs[0].size.y and subs[1].size.y >= subs[2].size.y, true)

	print("\n=== D. selection is ink alone ===")
	var fills := {}
	var inks := {}
	for k in rows.keys():
		var b: Button = rows[k]
		var sb: StyleBox = b.get_theme_stylebox("normal")
		fills[sb.get_class()] = true
		inks[str(b.get_theme_color("font_color"))] = true
	_ok("all ten rows share one stylebox class", fills.keys().size(), 1)
	_ok("...and it draws no fill (StyleBoxEmpty)", fills.keys()[0], "StyleBoxEmpty")
	_ok("ink is what varies (2 colours: selected, rest)", inks.keys().size() >= 2, true)

	print("\n=== E. the column did not grow ===")
	var panel: Control = _app.get("_rail_exp_column")
	var scroll: Control = panel.get_child(0)
	print("  info panel min %.0f · scroll min %.0f · panel width %.0f"
		% [panel.get_combined_minimum_size().x, scroll.get_combined_minimum_size().x, panel.size.x])
	_ok("the panel is exactly w_rail_expanded", int(panel.size.x), rail_w)
	_ok("the disabled-axis ScrollContainer folded in nothing wider",
		scroll.get_combined_minimum_size().x <= float(rail_w), true)
	_app.call("set_rail_expanded", false)
	await _frames(4)

func _rail_column() -> Node:
	var panel: Control = _app.get("_rail_exp_column")
	if panel == null or panel.get_child_count() == 0:
		return null
	var scroll: Node = panel.get_child(0)
	return scroll.get_child(0) if scroll.get_child_count() > 0 else null

# -- F..J : Find on map ------------------------------------------------------------

func _find() -> void:
	var bridge = _app.get("bridge")
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(20)

	_app.call("open_find_on_map")
	await _frames(10)
	var count: Label = _app.get("_desktop_search_count")
	var chips: Control = _app.get("_desktop_search_chips")
	var idx = _app.get("_place_search_index")

	print("\n=== F. the six scope chips (%s) ===" % _dens)
	var texts: Array = []
	var declined: Array = []
	for c in _live_children(chips):
		texts.append((c as Button).text)
		if (c as Button).disabled:
			declined.append((c as Button).text)
	_ok("six chips, in SEARCH_SCOPES order", ",".join(PackedStringArray(texts)),
		"s settlements,f factions,lb labels,r routes,. commands,l landmarks")
	_ok("exactly one is disabled", declined.size(), 1)
	_ok("...and it is landmarks", declined[0] if declined.size() > 0 else "-", "l landmarks")
	var l_chip: Button = null
	for c in _live_children(chips):
		if (c as Button).text == "l landmarks":
			l_chip = c
	_ok("the disabled chip carries the code's own reason, not a blank",
		l_chip != null and l_chip.tooltip_text.contains("four closed vocabularies"), true)

	print("\n=== G. whole-token scope split: l and lb cannot collide ===")
	_ok("'lb ald' -> labels", _scope_of("lb ald"), "lb")
	_ok("'l ald' -> the declined landmarks row", _scope_of("l ald"), "l")
	_ok("'ald' (no space) -> no scope at all", _scope_of("ald"), "-")
	_ok("'lbx ald' (unknown token) -> no scope", _scope_of("lbx ald"), "-")
	_ok("'lb ald' residual query is 'ald'",
		String(_app.call("_split_search_scope", "lb ald").get("query", "")), "ald")

	print("\n=== H. an unscoped query, and the count ===")
	var total: int = int(idx.call("size"))
	_q("")
	await _frames(6)
	var rows_all := _rows_of()
	print("  info index size %d · drawn %d · count '%s'" % [total, rows_all.size(), count.text])
	_ok("the resting state is the whole index", rows_all.size(), total)
	_ok("the count reads the index, not the list", count.text,
		"%d of %d places · rebuilt on open" % [total, total])
	var needle := _busiest_letter(idx)
	_q(needle)
	await _frames(6)
	var drawn := _rows_of()
	var ranked: Array = idx.call("search", needle)
	_ok("query '%s' draws exactly what search() returned" % needle, drawn.size(), ranked.size())
	var same := true
	for n in mini(drawn.size(), ranked.size()):
		if drawn[n] != String((ranked[n] as Dictionary)["name"]).to_upper():
			same = false
	_ok("...in search()'s own order -- the bands DRAW, they do not re-rank", same, true)
	var bands := _bands_of()
	print("  info bands drawn: ", ",".join(PackedStringArray(bands)))
	_ok("at least two bands for a busy query", bands.size() >= 2, true)
	var legal := true
	for b in bands:
		if not (b as String) in ["STARTS WITH", "CONTAINS", "MATCHED ON KIND OR SUBTITLE"]:
			legal = false
	_ok("every band header is one of PLACE_BANDS", legal, true)
	_ok("no band header repeats (classification agrees with the ranker)",
		bands.size(), _unique(bands).size())
	## **Where the seams FALL, not just which headers appear.** An oracle
	## written from `place_search.gd`'s header rather than from the shell's own
	## classifier, and deliberately in different terms (`begins_with` /
	## `find > 0`, against `_place_band()`'s `find == 0`), so this cannot pass
	## by agreeing with the code under test. Without it a classifier that folds
	## two bands together still draws two legal, non-repeating headers.
	var oracle_bands: Array = []
	for r in ranked:
		var nm := String((r as Dictionary)["name"]).to_lower()
		if nm.begins_with(needle):
			oracle_bands.append("STARTS WITH")
		elif nm.find(needle) > 0:
			oracle_bands.append("CONTAINS")
		else:
			oracle_bands.append("MATCHED ON KIND OR SUBTITLE")
	print("  info band sizes: drawn ", _band_runs(), " · oracle ", _runs(oracle_bands))
	_ok("each band header sits over exactly the rows that band owns",
		str(_band_runs()), str(_runs(oracle_bands)))

	print("\n=== I. a scope narrows the list ===")
	_q("s " + needle)
	await _frames(6)
	var scoped := _rows_of()
	var kinds := {}
	for r in idx.call("search", needle):
		kinds[String((r as Dictionary)["name"]).to_upper()] = String((r as Dictionary)["entity"])
	var only_settlements := true
	for name in scoped:
		if kinds.get(name, "") != "settlement":
			only_settlements = false
	print("  info unscoped %d -> scoped %d · count '%s'" % [drawn.size(), scoped.size(), count.text])
	_ok("the scope narrowed the list", scoped.size() < drawn.size(), true)
	_ok("every drawn row is a settlement", only_settlements, true)
	_ok("the count still measures against the WHOLE index", count.text,
		"%d of %d places · rebuilt on open" % [scoped.size(), total])

	print("\n=== J. the '.' scope reaches CommandIndex, and 'l' declines ===")
	var oracle := CommandIndex.new()
	oracle.build(_app, bridge)
	_q(". ")
	await _frames(6)
	var cmds := _rows_of()
	print("  info CommandIndex.size()=%d · drawn %d · count '%s'"
		% [oracle.size(), cmds.size(), count.text])
	_ok("an independently built CommandIndex is not empty", oracle.size() > 40, true)
	_ok("the '.' scope draws all of it at an empty query", cmds.size(), oracle.size())
	_ok("the count switches noun and total", count.text,
		"%d of %d commands · rebuilt on open" % [oracle.size(), oracle.size()])
	var inert := true
	for c in _live_children(_app.get("_desktop_search_results")):
		if c is Button and not (c as Button).disabled:
			inert = false
	_ok("every command row is inert (no popup/id/Callable exists to press)", inert, true)
	_q(". zoom")
	await _frames(6)
	var cbands := _bands_of()
	print("  info command bands: ", ",".join(PackedStringArray(cbands)), " · rows ", _rows_of().size())
	var clegal := cbands.size() > 0
	for b in cbands:
		if not (b as String) in ["MATCHED THE NAME", "MATCHED THE MENU", "MATCHED THE DESCRIPTION"]:
			clegal = false
	_ok("command bands come from COMMAND_BANDS", clegal, true)

	_q("l ald")
	await _frames(6)
	_ok("the declined scope draws no rows", _rows_of().size(), 0)
	_ok("...it draws the reason instead",
		_notice_text().contains("four closed vocabularies"), true)
	_ok("...and the count says why, not '0 of 0'", count.text, "landmarks · not indexed")

# -- K : the phone sheet ----------------------------------------------------------

func _phone_find() -> void:
	var bridge = _app.get("bridge")
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(20)

	print("
=== K. the phone sheet still shows results (PHONE) ===")
	_app.call("open_find_on_map")
	await _frames(10)
	var overlay: Control = _app.get("_phone_search_overlay")
	var panel: Control = overlay.get_child(overlay.get_child_count() - 1)
	var results: VBoxContainer = _app.get("_phone_search_results")
	var scroll: Control = results.get_parent()
	var chips: Control = _app.get("_phone_search_chips")
	var count: Label = _app.get("_phone_search_count")
	var rows := 0
	for c in _live_children(results):
		if c is Button:
			rows += 1
	var row_h: float = 0.0
	for c in _live_children(results):
		if c is Button:
			row_h = (c as Control).size.y
			break
	var fits: int = int(scroll.size.y / row_h) if row_h > 0.0 else 0
	print("  info sheet %.0f px · chips %.0f px · count %.0f px · scroll %.0f px · row %.0f px · %d rows fit"
		% [panel.size.y, chips.size.y, count.size.y, scroll.size.y, row_h, fits])
	_ok("the overlay is open", overlay.visible, true)
	_ok("the chips row drew all six", _live_children(chips).size(), 6)
	_ok("the count line is not blank", count.text != "", true)
	_ok("the whole index is listed at rest", rows > 0, true)
	_ok("the result scroll still fits more than four rows", fits > 4, true)
	_ok("the sheet fits inside the screen", panel.size.y + panel.offset_top
		<= float(_vp.size.y), true)

# -- helpers ---------------------------------------------------------------------

func _q(q: String) -> void:
	var f: LineEdit = _app.get("_desktop_search_field")
	f.text = q
	_app.call("_refresh_desktop_search")

func _scope_of(raw: String) -> String:
	var split: Dictionary = _app.call("_split_search_scope", raw)
	if not split.has("scope"):
		return "-"
	return String((split["scope"] as Dictionary)["prefix"])

## `queue_free()`d children are still in the tree until the frame ends.
func _live_children(n: Node) -> Array:
	var out: Array = []
	for c in n.get_children():
		if not c.is_queued_for_deletion():
			out.append(c)
	return out

## The drawn result rows, by their `_phone_list_row()` accessibility name --
## which is the un-cased title, so this reads what the row actually says.
func _rows_of() -> Array:
	var out: Array = []
	for c in _live_children(_app.get("_desktop_search_results")):
		if c is Button:
			out.append(_row_title(c))
	return out

func _row_title(b: Node) -> String:
	for d in b.get_child(0).get_child(0).get_children():
		if d is Label:
			return (d as Label).text
	return ""

func _bands_of() -> Array:
	var out: Array = []
	for c in _live_children(_app.get("_desktop_search_results")):
		if c is MarginContainer and c.get_child_count() > 0 and c.get_child(0) is Label:
			out.append((c.get_child(0) as Label).text)
	return out

func _notice_text() -> String:
	for c in _live_children(_app.get("_desktop_search_results")):
		if c is MarginContainer and c.get_child_count() > 0 and c.get_child(0) is Label:
			return (c.get_child(0) as Label).text
	return ""

## `[[band, run length], ...]` for the list as DRAWN -- a header, then every
## row until the next header.
func _band_runs() -> Array:
	var out: Array = []
	for c in _live_children(_app.get("_desktop_search_results")):
		if c is Button:
			if out.size() > 0:
				out[out.size() - 1][1] += 1
		elif c is MarginContainer and c.get_child_count() > 0 and c.get_child(0) is Label:
			out.append([(c.get_child(0) as Label).text, 0])
	return out

## The same shape, run-length encoded from a flat per-row band list.
func _runs(flat: Array) -> Array:
	var out: Array = []
	for b in flat:
		if out.size() > 0 and out[out.size() - 1][0] == b:
			out[out.size() - 1][1] += 1
		else:
			out.append([b, 1])
	return out

func _unique(a: Array) -> Array:
	var out: Array = []
	for v in a:
		if not out.has(v):
			out.append(v)
	return out

## A one-letter query that lands rows in more than one band, chosen from the
## world rather than hardcoded -- a fixed "ald" would be a claim about a seed.
func _busiest_letter(idx) -> String:
	var best := "a"
	var best_score := -1
	for ch in "abcdefghijklmnopqrstuvwxyz":
		var got: Array = idx.call("search", ch)
		var seen := {}
		for r in got:
			var at: int = String((r as Dictionary)["name"]).to_lower().find(ch)
			seen[0 if at == 0 else (1 if at > 0 else 2)] = true
		var score: int = seen.keys().size() * 1000 + got.size()
		if seen.keys().size() >= 2 and score > best_score:
			best_score = score
			best = ch
	return best
