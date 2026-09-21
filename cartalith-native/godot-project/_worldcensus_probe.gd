extends Node
## WORLD census: every control the WORLD dock draws, category by category and
## **mode by mode**, as a TSV that can be diffed control-for-control across the
## owner's WORLD re-sort (`LARGE_ITEM_RULINGS.md` Ruling L;
## `design/owner-references-2026-09-12/left_rail_tree_resorted.md`).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _worldcensus_probe.tscn -- --out C:/abs/world.tsv
##
## Modelled on `_civilcensus_probe.gd` and sharing its record shape, with one
## structural difference that is WORLD's own: **WORLD gates.** `RAIL_NODES`'
## `shows` hides categories per mode, so a single pass over `categories` would
## walk whatever the last mode left visible and silently miss the other half.
## Every category is therefore walked once per mode (`a` = PIPELINE, `b` =
## SCULPT), and each record carries the mode it was taken in.
##
## **Desktop leg only, deliberately.** The phone WORLD surface is not this dock:
## `PhoneMenu`'s GENERATE tab calls `WorldWorkspace.build_phone_generate()`,
## a separate builder with its own `_pg_*` tree that Ruling L's left-rail tree
## does not describe. Censusing it here would compare two different surfaces.
##
## Flags, after `--`:
##   --out PATH      write the census TSV to this absolute path; omitted, no file
##   --seed N        world seed, default 77021 (256x192 grid, 2000 km)
##
## **Exit 1** on any of: a category the shell's own jump
## (`select_domain_category`) did not open, read back from the accordion (body
## and wrap visible, visible in tree, the only open body in the dock); a visible
## category with no content; a dock with no categories; the wrong density; no
## world; the WORLD dock not on screen; the categories not in Ruling L's order
## (`RULING_L_ORDER`); a mode rendering a category set other than Ruling L's
## (`MODE_SHOWS`); a Ruling L placement that does not hold (`PLACEMENTS`); the
## two controls Ruling L flags as wrongly noted "(disabled: no binding)" coming
## up disabled (`ENABLED_TRAPS`); the category-level expander not drawn at the
## section inset (`INSET_EXPANDERS`); and **any SCRIPT ERROR anywhere in the
## process**, caught by an `OS.add_logger()` tap installed before the shell
## loads. **Exit 3** is the watchdog.
##
## A run on the pre-Ruling-L shell fails the order, mode, placement and inset
## checks, as it should; it still writes its TSV, which is the point -- the
## before/after diff is the evidence, and the assertions are what name the
## differences that are deliberate.

const LEG := Vector2i(1920, 1080)
const COLUMNS: Array[String] = ["kind", "mode", "cat_idx", "category", "section", "groups",
	"class", "row", "text", "tooltip", "disabled", "reason", "nsig", "sigs", "vis", "extra"]

## Ruling L's nine WORLD categories in the tree's order -- seven PIPELINE then
## two SCULPT -- written out rather than read from the dock under test.
const RULING_L_ORDER: Array[String] = ["Generate", "Planet", "Geology", "Hydrology",
	"Climate", "Ecology", "World data", "Terrain", "Biomes"]

## Which categories each mode renders. `RAIL_NODES`' `shows` is the mechanism;
## this is the tree's own answer, asserted against what the dock actually draws.
const MODE_SHOWS: Dictionary = {
	"a": ["Generate", "Planet", "Geology", "Hydrology", "Climate", "Ecology", "World data"],
	"b": ["Terrain", "Biomes"],
}

## Ruling L's moves, by the identity each control is recorded with: `[mode,
## category, section, groups, class, text, how many]`, counted over the records
## the walk made in that mode, so a control the build dropped, duplicated or
## left in its old container fails here by name. `*` matches any category,
## section or groups. A `0` is a deliberate removal.
const PLACEMENTS: Array = [
	## Generate ▸ Run -- Center landmasses is here and not in Hydrology:
	## Ruling L's decision 1 says Hydrology, its tree says Generate ▸ Run, and
	## the owner ruled "Tree is leading".
	["a", "Generate", "RUN", "", "Button", "Generate world", 1],
	["a", "Generate", "RUN", "", "Button", "New seed", 1],
	["a", "Generate", "RUN", "", "Button", "Center landmasses", 1],
	["a", "*", "*", "*", "Button", "Center landmasses", 1],
	## Generate ▸ Import -- "both halves of it together", one copy each.
	["a", "Generate", "IMPORT", "", "Button", "Load heightmap…", 1],
	["a", "Generate", "IMPORT", "", "Button", "Infer tectonics from heightmap…", 1],
	["a", "*", "*", "*", "Button", "Load heightmap…", 1],
	["a", "*", "*", "*", "Button", "Infer tectonics from heightmap…", 1],
	["a", "*", "*", "*", "Label", "§ HEIGHTMAP", 0],
	["a", "*", "*", "*", "Label", "§ FROM AN IMPORTED SURFACE", 0],
	## Generate ▸ Finalize, unchanged, and the note the tree removes outright.
	["a", "Generate", "FINALIZE", "", "Button", "Bake ALL levels & finalize", 1],
	["a", "Generate", "FINALIZE", "", "Button", "Clear this world's atlas", 1],
	["a", "*", "*", "*", "Label", "§ NOT A GENERATION STAGE", 0],
	["a", "Generate", "", "", "Label", "§ LOD TERRAIN DATA", 1],
	## Planet -- the input half of the old World data, plus stage 03.
	["a", "Planet", "", "", "Label", "§ 01 PLANET", 1],
	["a", "Planet", "", "", "Label", "§ 02 EXTENT & SCALE", 1],
	["a", "Planet", "", "", "Label", "§ 03 WORLD STRUCTURE", 1],
	["a", "*", "*", "*", "Label", "§ 03 WORLD STRUCTURE", 1],
	## Geology -- two stages, then the creep half of 06 as a category expander.
	["a", "Geology", "", "", "Label", "§ 04 TECTONICS", 1],
	["a", "Geology", "", "", "Label", "§ 05 VOLCANISM & IMPACTS", 1],
	["a", "Geology", "", "", "Button", "› 06 EROSION · HILLSLOPE DIFFUSE", 1],
	["a", "*", "*", "*", "Button", "› HILLSLOPE DIFFUSE", 0],
	## Hydrology -- the water & ice half of 06, above the drainage it feeds.
	["a", "Hydrology", "", "", "Label", "§ 06 EROSION · WATER & ICE", 1],
	["a", "Hydrology", "06 EROSION · WATER & ICE", "DROPLET HYDRAULIC", "Button", "Erode (droplet)", 1],
	["a", "Hydrology", "06 EROSION · WATER & ICE", "GLACIAL", "Button", "Carve fjords", 1],
	["a", "*", "*", "*", "Button", "Erode (droplet)", 1],
	["a", "*", "*", "*", "Button", "Carve fjords", 1],
	["a", "Hydrology", "", "", "Label", "§ RIVER NETWORK", 1],
	["a", "Hydrology", "", "", "Label", "§ RIVER ENTITIES", 1],
	## Ecology absorbs stage 09; Resources (stage 10) is removed outright.
	["a", "Ecology", "", "", "Label", "§ 09 ECOLOGY & BIOMES", 1],
	["a", "Ecology", "PRODUCTIVITY", "", "Button", "Show productivity on the map", 1],
	["a", "Ecology", "FAUNA", "", "Button", "Show fauna on the map", 1],
	["a", "*", "*", "*", "Label", "§ 10 RESOURCES & SOILS", 0],
	## World data keeps only its readouts.
	["a", "World data", "READ THE FIELDS", "", "Button", "World data tables…", 1],
	["a", "World data", "READ THE FIELDS", "", "Button", "Export GeoJSON…", 1],
	["a", "World data", "", "", "Label", "§ COORDINATE SYSTEM", 1],
	## The two sections World data LOST, asserted as **exactly one copy
	## anywhere** rather than as zero. A `0` here would contradict the two
	## `Planet` rows above, which require the same two labels to exist: the
	## move is "out of World data and into Planet", not a removal, and only the
	## whole-dock count can say it was a move rather than a copy. This is the
	## same idiom as `§ 03 WORLD STRUCTURE`'s pair in the Planet block; the
	## rows that genuinely are removals (`§ HEIGHTMAP`, `§ FROM AN IMPORTED
	## SURFACE`, `§ NOT A GENERATION STAGE`, `§ 10 RESOURCES & SOILS`) keep
	## their `0`.
	["a", "*", "*", "*", "Label", "§ 01 PLANET", 1],
	["a", "*", "*", "*", "Label", "§ 02 EXTENT & SCALE", 1],
	## SCULPT: the sculpt block, and the brush given a real left-dock home.
	["b", "Terrain", "", "", "Label", "§ GEOLOGICAL FEATURE", 1],
	["b", "Terrain", "", "", "Label", "§ PRESETS", 1],
	["b", "Terrain", "DRAFT", "COMMIT", "Button", "Discard draft", 1],
	["b", "Terrain", "DRAFT", "PAINTED LAKES", "Button", "Count painted lakes as water", 1],
	["b", "Biomes", "", "", "Label", "§ BIOME PAINT", 1],
	["b", "Biomes", "BIOME PAINT", "COMMIT", "Button", "Discard draft", 1],
	["b", "*", "*", "*", "Button", "Discard draft", 2],
]

## Ruling L, verbatim: the tree's *"(disabled: no binding)"* notes on these two
## are WRONG -- both guards ask the live `WorldGen`, both methods are exported
## `#[func]`s, and both buttons are enabled. `[mode, text]`.
const ENABLED_TRAPS: Array = [["a", "Erode (droplet)"], ["b", "Count painted lakes as water"]]

## A category-level expander draws with a section's geometry: heading and rows
## at the § inset, 14 px in from the body's edge like the sections beside it
## (`CivilizationWorkspace.category_expander()`). `[mode, category, heading]`.
const INSET_EXPANDERS: Array = [["a", "Geology", "› 06 EROSION · HILLSLOPE DIFFUSE"]]

## Every error the engine reports, from any thread (the generation worker logs
## off the main thread, hence the mutex). No `class_name`: an inner class.
class ErrorTap extends Logger:
	var _mutex := Mutex.new()
	var _script := PackedStringArray()
	var _error := PackedStringArray()
	var _warning := PackedStringArray()

	func _log_error(function: String, file: String, line: int, code: String,
			rationale: String, _editor_notify: bool, error_type: int,
			_script_backtraces: Array) -> void:
		var msg := "%s:%d %s(): %s" % [file, line, function,
			rationale if not rationale.is_empty() else code]
		_mutex.lock()
		if error_type == ERROR_TYPE_SCRIPT:
			_script.append(msg)
		elif error_type == ERROR_TYPE_WARNING:
			_warning.append(msg)
		else:
			_error.append(msg)
		_mutex.unlock()

	func take() -> Dictionary:
		_mutex.lock()
		var out := {"script": _script.duplicate(), "error": _error.duplicate(),
			"warning": _warning.duplicate()}
		_mutex.unlock()
		return out

var _tap: ErrorTap
var _fail := 0
var _seed := 77021
var _prefs := ""
var _mode := ""
var _rows := PackedStringArray()
var _ctl: Array = []   ## every category record, for `_check_placements()`
var _app: Node
var _rail_labels := {}

func _arg(name: String, fallback: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return a[i + 1] if i >= 0 and i + 1 < a.size() else fallback

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(what: String, good: bool, detail: String = "") -> void:
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", what, ("   " + detail) if not detail.is_empty() else "")

func _ready() -> void:
	_tap = ErrorTap.new()
	OS.add_logger(_tap)
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func() -> void:
		print("[WATCHDOG] 300 s elapsed -- exit 3")
		OS.remove_logger(_tap)
		get_tree().quit(3))
	wd.start()
	await _run()
	await _finish()

func _run() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		_ok("the cartalith extension loaded", false)
		return
	_seed = int(_arg("--seed", "77021"))
	print("[BOOT] leg=desktop viewport=%dx%d seed=%d" % [LEG.x, LEG.y, _seed])

	var vp := SubViewport.new()
	vp.size = LEG
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	_app = (load("res://shell/app.tscn") as PackedScene).instantiate()
	vp.add_child(_app)
	await get_tree().create_timer(1.2).timeout
	await _frames(10)
	_ok("density is DESKTOP (pointer)", not DccTheme.is_touch(), "is_touch=%s" % DccTheme.is_touch())
	var opd = _app.get("open_project_dialog")
	if opd != null and opd.visible:
		opd.hide()
	_prefs = "units=%s" % DccSettings.units_mode()
	print("  info preferences: ", _prefs)

	# -- a world ---------------------------------------------------------------
	var bridge: Node = _app.get("bridge")
	_ok("before generation there is no world yet", not bool(bridge.get("has_world")))
	var t0 := Time.get_ticks_msec()
	bridge.call("generate", {"seed": _seed, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bool(bridge.get("generating")):
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _frames(8)
	_ok("a world was generated", bool(bridge.get("has_world")),
		"seed=%d grid=256x192 %d ms" % [_seed, Time.get_ticks_msec() - t0])

	# -- WORLD on screen -------------------------------------------------------
	_app.call("select_domain", "world")
	await _frames(4)
	var panel: Control = _app.call("workspace_panel", "world")
	_ok("WORLD is the active domain", String(_app.call("active_domain")) == "world")
	_ok("the WORLD dock is on screen", panel != null and panel.is_visible_in_tree())
	if panel == null:
		return

	var cats: Array = panel.get("categories")
	_ok("the WORLD dock builds at least one category", not cats.is_empty(), "n=%d" % cats.size())
	var titles := PackedStringArray()
	for e in cats:
		titles.append(String(e["title"]))
	print("  info %d categories in dock order: %s" % [cats.size(), " | ".join(titles)])
	_ok("the dock builds Ruling L's nine, in the tree's order",
		" | ".join(titles) == " | ".join(PackedStringArray(RULING_L_ORDER)),
		"got=[%s]" % " | ".join(titles))

	for n in (_app.get("RAIL_NODES") as Array):
		if String(n.get("domain", "")) != "world":
			continue
		if String(n.get("kind", "")) == "head":
			_emit({"kind": "RAILHEAD", "text": String(n["label"])})
		elif String(n.get("kind", "")) == "node":
			_rail_labels[String(n["mode"])] = String(n["label"])
			_emit({"kind": "RAIL", "category": String(n.get("category", "")), "text": String(n["label"]),
				"extra": "mode=%s;owns=%s;shows=%s" % [String(n["mode"]),
					",".join(PackedStringArray(n.get("owns", []))),
					",".join(PackedStringArray(n.get("shows", [])))]})

	# -- every mode, then every category it renders ---------------------------
	var wraps := {}
	for e in cats:
		wraps[(e["body"] as Control).get_parent()] = true
	for mode in ["a", "b"]:
		_mode = String(mode)
		_app.call("select_domain_mode", "world", _mode)
		await _frames(4)
		_ok("[%s] the shell reports mode %s" % [_mode, _mode],
			String(_app.call("active_mode", "world")) == _mode)
		var shown := PackedStringArray()
		for e in cats:
			var w := (e["body"] as Control).get_parent() as Control
			if w != null and w.visible:
				shown.append(String(e["title"]))
		var want: PackedStringArray = PackedStringArray(MODE_SHOWS[_mode])
		_ok("[%s] renders exactly Ruling L's categories for this mode" % _mode,
			" | ".join(shown) == " | ".join(want),
			"got=[%s] want=[%s]" % [" | ".join(shown), " | ".join(want)])
		_check_paint_pill(panel)

		for i in cats.size():
			var e: Dictionary = cats[i]
			var title := String(e["title"])
			var wrap := (e["body"] as Control).get_parent() as Control
			if wrap == null or not wrap.visible:
				continue
			var idx := "%02d" % (i + 1)
			_app.call("select_domain_category", "world", title)
			await _frames(3)
			var body: Control = e["body"]
			var open_now := PackedStringArray()
			for e2 in cats:
				if (e2["body"] as Control).visible:
					open_now.append(String(e2["title"]))
			var in_tree := body.is_visible_in_tree()
			var domain := String(_app.call("active_domain"))
			var opened: bool = body.visible and wrap.visible and in_tree \
				and open_now.size() == 1 and domain == "world"
			_ok("[%s/%s] %s opens" % [_mode, idx, title], opened, "" if opened else
				"body.visible=%s wrap.visible=%s visible_in_tree=%s open=[%s] domain=%s"
				% [body.visible, wrap.visible, in_tree, ", ".join(open_now), domain])
			_emit_head(idx, e)
			var recs: Array = []
			_walk(body, body, idx, title, {}, recs)
			var n_vis := 0
			var n_act := 0
			var n_dis := 0
			for r in recs:
				n_vis += 1 if r["vis"] == "1" else 0
				n_act += 1 if r["_act"] else 0
				n_dis += 1 if r["disabled"] == "1" else 0
				_ctl.append(r)
				_emit(r)
			var counts := "records=%d visible=%d interactive=%d disabled=%d" % [
				recs.size(), n_vis, n_act, n_dis]
			_ok("[%s/%s] %s has content" % [_mode, idx, title], recs.size() > 0 and n_vis > 0, counts)
			_emit({"kind": "COUNT", "cat_idx": idx, "category": title,
				"extra": counts.replace(" ", ";")})
			_check_inset(idx, title, body)

		# -- the dock outside every category (TOOLS block and the rest) -------
		var dock: Array = []
		_walk(panel, panel, "", "(dock)", wraps, dock)
		var d_vis := 0
		for r in dock:
			d_vis += 1 if r["vis"] == "1" else 0
			_ctl.append(r)
			_emit(r)
		var dcounts := "records=%d visible=%d" % [dock.size(), d_vis]
		print("  info [%s] (dock) outside every category: %s" % [_mode, dcounts])
		_emit({"kind": "COUNT", "category": "(dock)", "extra": dcounts.replace(" ", ";")})

	_check_placements()
	_check_traps()

# -- Ruling L --------------------------------------------------------------------

func _check_placements() -> void:
	for row: Array in PLACEMENTS:
		var want := int(row[6])
		var got := 0
		for r: Dictionary in _ctl:
			if String(r.get("kind", "")) != "CTL" or String(r.get("mode", "")) != String(row[0]):
				continue
			if (row[1] == "*" or r["category"] == row[1]) and (row[2] == "*" or r["section"] == row[2]) \
					and (row[3] == "*" or r["groups"] == row[3]) and r["class"] == row[4] \
					and String(r["text"]).to_upper() == String(row[5]).to_upper():
				got += 1
		_ok("placed x%d: [%s] %s | %s | %s | %s \"%s\"" % [want, row[0], row[1], row[2], row[3],
			row[4], row[5]], got == want, "got=%d" % got)

## Ruling L: the tree's "(disabled: no binding)" notes on these two are wrong.
func _check_traps() -> void:
	for row: Array in ENABLED_TRAPS:
		var seen := 0
		var disabled := 0
		for r: Dictionary in _ctl:
			if String(r.get("kind", "")) != "CTL" or String(r.get("mode", "")) != String(row[0]):
				continue
			if String(r["text"]).to_upper() == String(row[1]).to_upper():
				seen += 1
				disabled += 1 if r["disabled"] == "1" else 0
		_ok("Ruling L trap: \"%s\" exists and is ENABLED (the tree's disabled note is wrong)" % row[1],
			seen == 1 and disabled == 0, "seen=%d disabled=%d" % [seen, disabled])

## Measured while the category is open, against a `§` heading in the same body,
## so the number is the section's own rather than a literal; the rows' x is read
## off the expander's body pad (position plus its left margin), which is laid
## out whether or not the expander is open.
func _check_inset(idx: String, title: String, body: Control) -> void:
	for row: Array in INSET_EXPANDERS:
		if String(row[0]) != _mode or String(row[1]) != title:
			continue
		var hdr: Control = null
		var sec: Control = null
		var stack: Array = [body]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			if n.is_queued_for_deletion():
				continue
			if hdr == null and n is Button and (n as Button).is_visible_in_tree() \
					and String(n.get("text")).to_upper() == String(row[2]).to_upper():
				hdr = n
			elif sec == null and n is Label and (n as Label).is_visible_in_tree() \
					and (n as Label).text.begins_with("§ "):
				sec = n
			stack.append_array(n.get_children())
		var bx := body.get_global_rect().position.x
		var hx := (hdr.get_global_rect().position.x - bx) if hdr != null else -1.0
		var sx := (sec.get_global_rect().position.x - bx) if sec != null else -1.0
		var rx := -1.0
		if hdr != null and hdr.get_index() + 1 < hdr.get_parent().get_child_count():
			var pad := hdr.get_parent().get_child(hdr.get_index() + 1) as MarginContainer
			if pad != null:
				rx = pad.get_global_rect().position.x - bx + pad.get_theme_constant("margin_left")
		_ok("[%s/%s] %s's %s heading and rows draw at the § inset" % [_mode, idx, title, row[2]],
			hdr != null and sec != null and absf(hx - sx) < 0.5 and absf(rx - sx) < 0.5,
			"heading_x=%d rows_x=%d section_x=%d (%s)" % [int(hx), int(rx), int(sx),
				(sec as Label).text if sec != null else "no § heading"])

## Ruling L's tree puts `Biome paint (B)` in WORLD's Tools row "(SCULPT mode
## only)". The pill is found by its tooltip, which is its label
## (`DccWidgets.tool_button()`), anywhere in the dock outside the categories.
func _check_paint_pill(panel: Control) -> void:
	var seen := 0
	var vis := 0
	var stack: Array = [panel]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.is_queued_for_deletion():
			continue
		if n is Button and String((n as Button).tooltip_text) == "Biome paint (B)":
			seen += 1
			vis += 1 if (n as Button).is_visible_in_tree() else 0
		stack.append_array(n.get_children())
	var want := 1 if _mode == "b" else 0
	_ok("[%s] the Tools row's Biome paint (B) pill is %s" % [_mode,
		"on screen" if want == 1 else "hidden (SCULPT mode only)"],
		seen == 1 and vis == want, "built=%d visible=%d want_visible=%d" % [seen, vis, want])

func _finish() -> void:
	await _frames(5)
	var logs := _tap.take()
	var script_errors: PackedStringArray = logs["script"]
	for m in script_errors:
		print("  SCRIPT ERROR  ", m)
		_emit({"kind": "ERR", "class": "SCRIPT", "text": m})
	for m in (logs["error"] as PackedStringArray):
		print("  info ERROR  ", m)
		_emit({"kind": "ERR", "class": "ERROR", "text": m})
	for m in (logs["warning"] as PackedStringArray):
		_emit({"kind": "ERR", "class": "WARNING", "text": m})
	print("  info logged: %d ERROR, %d WARNING" % [
		(logs["error"] as PackedStringArray).size(), (logs["warning"] as PackedStringArray).size()])
	_ok("no SCRIPT ERROR anywhere in the run", script_errors.is_empty(),
		"script_errors=%d" % script_errors.size())
	var out := _arg("--out", "")
	if not out.is_empty():
		var f := FileAccess.open(out, FileAccess.WRITE)
		_ok("census written to " + out, f != null,
			"" if f != null else "open error %d" % FileAccess.get_open_error())
		if f != null:
			f.store_line("# _worldcensus_probe leg=desktop viewport=%s seed=%d %s godot=%s" % [
				LEG, _seed, _prefs, Engine.get_version_info()["string"]])
			f.store_line("\t".join(PackedStringArray(COLUMNS)))
			for r in _rows:
				f.store_line(r)
			f.store_line("# verdict: %s" % ("PASS" if _fail == 0 else "%d FAILURE(S)" % _fail))
			f.close()
	print("\n_worldcensus_probe: %s" % ("PASS" if _fail == 0 else "%d FAILURE(S)" % _fail))
	OS.remove_logger(_tap)
	get_tree().quit(1 if _fail > 0 else 0)

# -- records ---------------------------------------------------------------------

func _emit(rec: Dictionary) -> void:
	rec["mode"] = _mode
	var cells := PackedStringArray()
	for col in COLUMNS:
		cells.append(_cell(rec.get(col, "")))
	_rows.append("\t".join(cells))

static func _cell(v: Variant) -> String:
	return str(v).replace("\\", "\\\\").replace("\t", "\\t").replace("\r", "").replace("\n", "\\n")

func _emit_head(idx: String, e: Dictionary) -> void:
	var btn: Button = e.get("button")
	var count := ""
	var hp := btn.get_parent()
	if hp is HBoxContainer:
		for sib in hp.get_children():
			if sib is MarginContainer and sib.get_child_count() == 1 and sib.get_child(0) is Label:
				count = (sib.get_child(0) as Label).text
	var title := String(e["title"])
	var owner := String(_app.call("mode_for_category", "world", title))
	var lit := String(_app.call("active_mode", "world"))
	var sig := _sigs(btn)
	_emit({"kind": "HEAD", "cat_idx": idx, "category": title, "class": _class_of(btn),
		"text": btn.text, "tooltip": btn.tooltip_text, "disabled": "1" if btn.disabled else "0",
		"nsig": str(sig[0]), "sigs": sig[1], "vis": "1" if btn.is_visible_in_tree() else "0",
		"extra": "owner=%s:%s;lit=%s:%s;phone_count=%s" % [
			owner, _rail_labels.get(owner, ""), lit, _rail_labels.get(lit, ""), count]})

func _walk(node: Node, root: Node, idx: String, cat: String, skip: Dictionary, out: Array) -> void:
	for child in node.get_children():
		if skip.has(child) or child.is_queued_for_deletion():
			continue
		if child is Window:
			var w := child as Window
			out.append({"kind": "WIN", "cat_idx": idx, "category": cat, "class": _class_of(w),
				"text": w.title, "disabled": "", "vis": "1" if w.visible else "0", "_act": false})
			continue
		if child is Control:
			var rec := _record(child as Control, root, idx, cat)
			if not rec.is_empty():
				out.append(rec)
		_walk(child, root, idx, cat, skip, out)

func _record(c: Control, root: Node, idx: String, cat: String) -> Dictionary:
	var text := _text_of(c)
	var act := c is BaseButton or c is Range or c is LineEdit or c is TextEdit \
		or c is ItemList or c is Tree or c is TabBar
	var sig := _sigs(c)
	if text.is_empty() and c.tooltip_text.is_empty() and not act \
			and c.get_script() == null and int(sig[0]) == 0:
		return {}
	var enc := _enclosing(c, root)
	var dis := _disabled(c)
	return {"kind": "CTL", "mode": _mode, "cat_idx": idx, "category": cat,
		"section": enc[0], "groups": " >> ".join(enc[1] as PackedStringArray),
		"class": _class_of(c), "row": _row_label(c), "text": text, "tooltip": c.tooltip_text,
		"disabled": dis, "reason": _reason(c, root) if dis == "1" else "",
		"nsig": str(sig[0]), "sigs": sig[1], "vis": "1" if c.is_visible_in_tree() else "0",
		"extra": _extra(c), "_act": act}

static func _text_of(c: Control) -> String:
	if c is RichTextLabel:
		var parsed := (c as RichTextLabel).get_parsed_text()
		return parsed if not parsed.is_empty() else (c as RichTextLabel).text
	if c is Label:
		return (c as Label).text
	if c is Button:
		return (c as Button).text
	if c is LinkButton:
		return (c as LinkButton).text
	if c is LineEdit:
		return (c as LineEdit).text
	if c is TextEdit:
		return (c as TextEdit).text
	return ""

static func _disabled(c: Control) -> String:
	if c is BaseButton:
		return "1" if (c as BaseButton).disabled else "0"
	if c is SpinBox:
		return "0" if (c as SpinBox).editable else "1"
	if c is Slider:
		return "0" if (c as Slider).editable else "1"
	if c is LineEdit:
		return "0" if (c as LineEdit).editable else "1"
	if c is TextEdit:
		return "0" if (c as TextEdit).editable else "1"
	return ""

## The house rule is "disabled WITH its reason": the control's own tooltip, or
## the nearest ancestor's inside the walked root.
static func _reason(c: Control, root: Node) -> String:
	if not c.tooltip_text.strip_edges().is_empty():
		return "self:" + c.tooltip_text
	var n := c.get_parent()
	while n != null and n != root:
		if n is Control and not (n as Control).tooltip_text.strip_edges().is_empty():
			return "anc:" + (n as Control).tooltip_text
		n = n.get_parent()
	return "NONE"

## The caption `DccWidgets._row()` drew for a control: the first non-empty
## `Label` among its siblings in the row.
static func _row_label(c: Control) -> String:
	if c is Label or not (c.get_parent() is HBoxContainer):
		return ""
	for sib in c.get_parent().get_children():
		if sib != c and sib is Label and not (sib as Label).text.is_empty():
			return (sib as Label).text
	return ""

## Returns [innermost section, groups outer->inner].
static func _enclosing(c: Node, root: Node) -> Array:
	var section := ""
	var groups := PackedStringArray()
	var n: Node = c
	while n != null and n != root:
		var p := n.get_parent()
		if n is VBoxContainer and p is MarginContainer and p.get_child_count() == 1 \
				and p.get_parent() != null and p.get_index() > 0:
			var head := p.get_parent().get_child(p.get_index() - 1)
			if head is MarginContainer and head.get_child_count() == 1 and head.get_child(0) is Label:
				var t := (head.get_child(0) as Label).text
				if t.begins_with("§ ") and section.is_empty():
					section = t.substr(2)
			elif head is Button and (head as Button).flat:
				var g := (head as Button).text
				var sp := g.find(" ")
				if sp > 0 and g == g.to_upper():
					groups.insert(0, g.substr(sp + 1))
		n = p
	return [section, groups]

static func _class_of(o: Object) -> String:
	if o == null:
		return ""
	var s: Script = o.get_script()
	if s == null:
		return o.get_class()
	var gn := String(s.get_global_name())
	return "%s:%s" % [o.get_class(), gn if not gn.is_empty() else s.resource_path.get_file()]

## [count, "signal>TargetClass.method,..." sorted] -- script-side connections
## only; the engine's own layout plumbing names itself `Class::method`, which a
## GDScript method or lambda cannot.
static func _sigs(c: Control) -> Array:
	var n := 0
	var parts := PackedStringArray()
	for s in c.get_signal_list():
		var sname := String(s["name"])
		for conn in c.get_signal_connection_list(sname):
			var cb: Callable = conn["callable"]
			var method := String(cb.get_method())
			if method.is_empty() or method.contains("::"):
				continue
			n += 1
			parts.append("%s>%s.%s" % [sname, _class_of(cb.get_object()), method])
	parts.sort()
	return [n, ",".join(parts)]

static func _extra(c: Control) -> String:
	var parts := PackedStringArray()
	if not c.visible:
		parts.append("self_hidden")
	if c is OptionButton:
		var ob := c as OptionButton
		var items := PackedStringArray()
		for k in ob.item_count:
			items.append(ob.get_item_text(k))
		parts.append("selected=%d" % ob.selected)
		parts.append("items=" + "|".join(items))
	elif c is BaseButton and (c as BaseButton).toggle_mode:
		parts.append("pressed=%s" % (c as BaseButton).button_pressed)
	if c is Range:
		var r := c as Range
		parts.append("value=%s" % r.value)
		parts.append("range=%s..%s/%s" % [r.min_value, r.max_value, r.step])
	if c is LineEdit and not (c as LineEdit).placeholder_text.is_empty():
		parts.append("placeholder=" + (c as LineEdit).placeholder_text)
	if c is TextEdit and not (c as TextEdit).placeholder_text.is_empty():
		parts.append("placeholder=" + (c as TextEdit).placeholder_text)
	return ";".join(parts)
