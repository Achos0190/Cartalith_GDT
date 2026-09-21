extends Node
## CARTO census: every control the CARTO dock draws, category by category, as a
## TSV that can be diffed control-for-control across the owner's 2026-09-13
## CARTO re-sort (`LARGE_ITEM_RULINGS.md` Ruling L;
## `design/owner-references-2026-09-12/left_rail_tree_resorted.md`, the CARTO
## half of the tree). The CIVIL half of the same re-sort was verified with
## `_civilcensus_probe.gd`, and this file is that probe with CIVIL's own
## assertions replaced by CARTO's; the record machinery at the bottom (`_walk`,
## `_record`, `_enclosing`, `_sigs`) is copied unchanged, so the two censuses
## are directly comparable in shape.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _cartocensus_probe.tscn -- --out C:/abs/carto_before.tsv
##
## **Desktop leg only, and that is the spec's own scope**: the tree is titled
## "LEFT RAIL (PC)" and `phone_menu.gd` has no CARTO row to enter the dock by
## (it has `_go_civilization`/`_go_simulation`/`_go_journey_planner` and nothing
## for cartography), so a phone leg would be entering the dock a way no user
## can. `--force-touch` is still accepted and recorded, and it then enters
## through `select_domain()` like the desktop leg.
##
## Flags, after `--`:
##   --force-touch   the phone leg (read by `dcc_shell.gd`; echoed here)
##   --out PATH      write the census TSV to this absolute path; omitted, no file
##   --seed N        world seed, default 77021 (256x192 grid, 2000 km)
##
## **Why the categories are walked, not listed.** Ruling L renames, merges and
## re-orders categories and moves controls between containers, and this one
## probe has to run on both sides of that change. So the category list comes
## from the dock's own `categories` array, and each control is recorded with
## enough identity -- class, text, row caption, tooltip, disabled + reason,
## script-side signal wiring -- to be matched across a move by what it is rather
## than where it sits.
##
## **Exit 1** on any of: a category the shell's own jump
## (`select_domain_category`) did not open, read back from the accordion; a
## category with no content, or none of it visible; a dock with no categories;
## the wrong density; no world; the CARTO dock not on screen; a Ruling L
## placement that does not hold (`PLACEMENTS`); the categories not in Ruling L's
## order (`RULING_L_ORDER`); CARTO not entering on its default-open floor Style
## with its seeded rail mode `style`; the rail's CARTO nodes not matching
## `RAIL_WANT`, not naming categories that exist, or not owning every category
## exactly once; and **any SCRIPT ERROR anywhere in the process**, caught by an
## `OS.add_logger()` tap installed before the shell loads. **Exit 3** is the
## watchdog. A parse error in this file cannot be caught from inside it: the
## scene then runs scriptless and never quits, so run it under a timeout.
##
## **A run against the pre-Ruling-L shell fails `RULING_L_ORDER`, `PLACEMENTS`
## and the rail checks, as it should** -- and still writes its TSV, which is the
## "before" half of the census.
##
## **What a record is.** One `CTL` row per Control, category bodies first and the
## dock outside them last, when the Control carries text or a tooltip -- or is
## interactive (button, range, text field, list), scripted, or has script-side
## signal connections. `section` is the innermost `DccWidgets.section()` title
## and `groups` the `DccWidgets.group()`/`advanced()` titles outer >> inner.
## `nsig` counts script-side connections only. Empty cell = not applicable; a
## disabled control with no tooltip on itself or any ancestor has reason `NONE`.
##
## Other row kinds: `RAILHEAD`/`RAIL` (the CARTO `RAIL_NODES` rows), `HEAD` (a
## category header, with the rail node that owns it and the one lit after the
## jump), `COUNT` (per category), `WIN` (a Window parented inside the dock), and
## `ERR` (every SCRIPT/ERROR/WARNING the tap saw).

const LEG_SIZE := {"desktop": Vector2i(1920, 1080), "phone": Vector2i(1080, 2340)}
const COLUMNS: Array[String] = ["kind", "leg", "cat_idx", "category", "section", "groups",
	"class", "row", "text", "tooltip", "disabled", "reason", "nsig", "sigs", "vis", "extra"]

## Ruling L's seven CARTO categories in the tree's order
## (`left_rail_tree_resorted.md` L249-341), written out rather than read from
## the dock under test.
const RULING_L_ORDER: Array[String] = ["Style", "Relief & light", "Colours",
	"Feature style", "Layers", "Labels", "Icons"]

## Ruling L's CARTO moves, by the identity each control is recorded with:
## `[category, section, groups, class, text, how many]`, counted over the
## records the walk made, so a control the build dropped, duplicated or left in
## its old container fails here by name. `*` matches any category, section or
## groups; a `*` row held at 1 is "exactly one copy anywhere in the dock", which
## is what catches a move that copied instead of moving, and `0` is a container
## the owner removed. A wanted text ending in `*` is a prefix match, for the two
## captions that carry a live number.
##
## **Captions are matched on the row's Label, not on the control.**
## `DccWidgets.toggle()` and `.slider()` build a caption `Label` beside a
## textless `CheckBox`/`HSlider`, so a toggle is asserted by its Label and an
## anonymous slider by counting `HSlider`s in its section.
const PLACEMENTS: Array = [
	# -- 1 Style: absorbs Map presets; Water is Water & light less the sun toggle
	["Style", "", "", "Label", "§ LOOK", 1],
	["Style", "", "", "Label", "§ PAINTER STYLES", 1],
	["Style", "", "", "Label", "§ WATER", 1],
	["Style", "", "", "Label", "§ SAVED LOOKS", 1],
	["Style", "", "", "Label", "§ STILL OWED", 1],
	["Style", "LOOK", "", "Label", "Base look", 1],
	["Style", "WATER", "", "Label", "Coastal wave lines", 1],
	["Style", "WATER", "", "Label", "Wave reach", 1],
	["Style", "WATER", "", "Label", "Animate water", 1],
	["Style", "SAVED LOOKS", "", "Button", "Save look", 1],
	["Style", "SAVED LOOKS", "", "Button", "Load look", 1],
	["*", "*", "*", "Label", "§ MAP STYLE", 0],
	["*", "*", "*", "Label", "§ WATER & LIGHT", 0],
	["*", "*", "*", "Label", "Coastal wave lines", 1],
	# -- 2 Relief & light: was Terrain appearance; gains Map view + Multi-sun
	["Relief & light", "", "", "Label", "§ MAP VIEW", 1],
	["Relief & light", "", "", "Label", "§ RENDERING - ADVANCED", 1],
	["Relief & light", "MAP VIEW", "", "Label", "Multi-sun lighting", 1],
	["Relief & light", "RENDERING - ADVANCED", "", "Button", "Reset to quality tier", 1],
	## `CheckBox`, not `Label`, and the only row here that has to be: the Look
	## gallery's tile blurbs are **derived** from `STYLE_PRESETS` via
	## `_bundle_line()`, and "Natural Vibrant" overrides `multi_sun` alone, so
	## its third tile line is a `Label` reading exactly "Multi-sun lighting".
	## That caption is a tile's description of a preset, not a second copy of
	## the toggle -- it predates Ruling L unchanged (`git show HEAD` has the
	## same `{"multi_sun": true}` and the same `_bundle_line`). Asserting the
	## `CheckBox` names the control itself and keeps the row's teeth: a move
	## that copied instead of moving still lands two of them.
	["*", "*", "*", "CheckBox", "Multi-sun lighting", 1],
	["*", "*", "*", "Button", "Reset to quality tier", 1],
	["*", "*", "*", "Label", "§ TERRAIN APPEARANCE", 0],
	# -- 3 Colours: gains Colour relief and the biome pair
	["Colours", "", "", "Label", "§ COLOUR RELIEF", 1],
	["Colours", "", "", "Label", "§ COLOUR GRADE", 1],
	["Colours", "", "", "Label", "§ BIOME COLOURS", 1],
	["Colours", "", "", "Label", "§ COLOUR MANAGEMENT", 1],
	["Colours", "COLOUR RELIEF", "", "Button", "Add stop", 1],
	["Colours", "COLOUR RELIEF", "", "Button", "Reverse", 1],
	["Colours", "BIOME COLOURS", "", "HSlider", "", 1],
	["Colours", "BIOME COLOURS", "", "Button", "Biome colour table*", 1],
	["Colours", "COLOUR MANAGEMENT", "", "Label", "Display", 1],
	["*", "*", "*", "Button", "Biome colour table*", 1],
	["*", "*", "*", "Button", "Add stop", 1],
	# -- 4 Feature style: the style halves of Roads & routes and Political display
	["Feature style", "", "", "Label", "§ WAYS", 1],
	["Feature style", "", "", "Label", "§ TERRITORIES", 1],
	["Feature style", "", "", "Label", "§ NOT BUILT", 1],
	["Feature style", "WAYS", "", "Label", "Line width", 1],
	["Feature style", "WAYS", "", "Label", "Opacity", 1],
	["Feature style", "WAYS", "", "Label", "Drop minor ways when zoomed out", 1],
	["Feature style", "WAYS", "", "Label", "Thicken ways by carried volume", 1],
	["Feature style", "WAYS", "", "Button", "Draw and edit ways → Civilization ▸ Routes & ways", 1],
	["Feature style", "TERRITORIES", "", "Label", "Fill opacity", 1],
	["Feature style", "TERRITORIES", "", "Button", "Reset to *", 1],
	["Feature style", "TERRITORIES", "", "Button", "Faction identity colours → Civilization ▸ Factions", 1],
	["Feature style", "TERRITORIES", "", "Button", "Edit territories → Civilization ▸ Territories", 1],
	["Feature style", "NOT BUILT", "", "Button", "Claim hatching and the influence ramp*", 1],
	["*", "*", "*", "Label", "Thicken ways by carried volume", 1],
	["*", "*", "*", "Label", "§ WAY STYLE", 0],
	["*", "*", "*", "Label", "§ TRADE LOAD", 0],
	["*", "*", "*", "Label", "§ TERRITORY TINT", 0],
	# -- 5 Layers: every visibility toggle, and nothing else claiming one
	["Layers", "", "", "Label", "§ VISIBLE LAYERS", 1],
	["Layers", "", "", "Label", "§ WAYS · BY TYPE", 1],
	["Layers", "", "", "Label", "§ POLITICAL LAYERS", 1],
	["Layers", "", "", "Label", "§ TERRAIN RASTER", 1],
	["Layers", "", "", "Label", "§ DATA OVERLAYS", 1],
	["Layers", "VISIBLE LAYERS", "", "Label", "Settlements", 1],
	["Layers", "VISIBLE LAYERS", "", "Label", "Landmark rejects (diagnostic)", 1],
	["Layers", "", "SETTLEMENTS · BY CLASS", "Label", "Villages", 1],
	["Layers", "WAYS · BY TYPE", "", "Label", "Trade highways", 1],
	["Layers", "WAYS · BY TYPE", "", "Label", "Ancient routes", 1],
	["Layers", "POLITICAL LAYERS", "", "Label", "Political — provinces", 1],
	["Layers", "POLITICAL LAYERS", "", "Label", "Political — territory", 1],
	["Layers", "DATA OVERLAYS", "", "Button", "Data overlays…", 1],
	["*", "*", "*", "Button", "Data overlays…", 1],
	["*", "*", "*", "Label", "Political — territory", 1],
	["*", "*", "*", "Label", "Ancient routes", 1],
	# -- 6/7 Labels unchanged; Icons is Assets & landmarks renamed
	["Labels", "", "", "Label", "§ LABEL CLASSES", 1],
	["Labels", "", "", "Label", "§ REGION LABELS", 1],
	["Labels", "REGION LABELS", "", "Button", "Clear all labels", 1],
	["Icons", "", "", "Label", "§ AUTOMATIC PLACEMENT", 1],
	["Icons", "", "", "Label", "§ PLACED ICONS", 1],
	["Icons", "PLACED ICONS", "", "Button", "Clear all icons", 1],
]

## The CARTO rail nodes Ruling L's node list names, `[mode, label, category]`
## (`left_rail_tree_resorted.md` L28-32). Asserted against `DccShell.RAIL_NODES`
## rather than against the rail's drawn buttons, because that table is what
## `select_domain_mode()` and `app.gd::_refresh_rail_foot()` both read -- the
## foot prints the mode id upper-cased, so this table is also what the footer
## says.
const RAIL_WANT: Array = [["style", "Style", "Style"], ["layers", "Layers", "Layers"],
	["labels", "Labels", "Labels"], ["icons", "Icons", "Icons"]]

## Category-level expanders draw with a section's geometry: heading and rows at
## the § inset, 14 px in from the body's edge like the sections beside them --
## the defect the CIVIL half of this same re-sort found and fixed
## (`category_expander()`, commit 5f839d7 L2). CARTO has one, and it is one the
## re-sort moves.
const INSET_EXPANDERS: Array = [["Layers", "› SETTLEMENTS · BY CLASS"]]

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
var _leg := "desktop"
var _seed := 77021
var _prefs := ""
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
	var touch := OS.get_cmdline_user_args().has("--force-touch")
	_leg = "phone" if touch else "desktop"
	_seed = int(_arg("--seed", "77021"))
	var size: Vector2i = LEG_SIZE[_leg]
	print("[BOOT] leg=%s viewport=%dx%d force-touch=%s seed=%d" % [_leg, size.x, size.y, touch, _seed])

	var vp := SubViewport.new()
	vp.size = size
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	_app = (load("res://shell/app.tscn") as PackedScene).instantiate()
	vp.add_child(_app)
	await get_tree().create_timer(1.2).timeout
	await _frames(10)
	if touch:
		_ok("density is PHONE", DccTheme.is_phone(),
			"is_phone=%s is_tablet=%s" % [DccTheme.is_phone(), DccTheme.is_tablet()])
	else:
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

	# -- CARTO on screen -------------------------------------------------------
	if touch:
		var picker = _app.get("phone_project_picker")
		if picker != null and picker.visible:
			print("  info the phone project picker was still up after generation; the probe hid it")
			picker.hide()
	_app.call("select_domain", "cartography")
	await _frames(4)
	var panel: Control = _app.call("workspace_panel", "cartography")
	_ok("CARTO is the active domain", String(_app.call("active_domain")) == "cartography")
	_ok("the CARTO dock is on screen", panel != null and panel.is_visible_in_tree())
	if panel == null:
		return

	var cats: Array = panel.get("categories")
	_ok("the CARTO dock builds at least one category", not cats.is_empty(), "n=%d" % cats.size())
	var titles := PackedStringArray()
	for e in cats:
		titles.append(String(e["title"]))
	print("  info %d categories in dock order: %s" % [cats.size(), " | ".join(titles)])
	_ok("the dock builds Ruling L's seven, in the tree's order",
		" | ".join(titles) == " | ".join(PackedStringArray(RULING_L_ORDER)),
		"got=[%s]" % " | ".join(titles))

	## Ruling L L257: Style is the tab's default-open category, and CARTO's
	## first `RAIL_NODES` node is what `DccShell._domain_mode` seeds the rail
	## with. Unlike CIVIL (plan C1), those two agree here: `style` owns Style.
	var open_at_entry := PackedStringArray()
	for e in cats:
		if (e["body"] as Control).visible:
			open_at_entry.append(String(e["title"]))
	_ok("entering CARTO opens its floor, Style, and nothing else",
		", ".join(open_at_entry) == "Style", "open=[%s]" % ", ".join(open_at_entry))
	var entry_mode := String(_app.call("active_mode", "cartography"))
	var entry_owner := String(_app.call("mode_for_category", "cartography",
		open_at_entry[0] if open_at_entry.size() == 1 else ""))
	_ok("entering CARTO lights the node that owns the open category",
		entry_mode == "style" and entry_owner == "style",
		"mode=%s owner_of_open=%s" % [entry_mode, entry_owner])
	if not touch:
		var foot: Variant = _app.get("rail_foot")
		var foot_text := (foot as Label).text if foot is Label else "<none>"
		_ok("...and the rail foot reads STYLE", foot_text == "STYLE", "foot=[%s]" % foot_text)

	_check_rail(titles)

	# -- every category, opened by the shell's jump and read back by state -----
	var wraps := {}
	for e in cats:
		wraps[(e["body"] as Control).get_parent()] = true
	for i in cats.size():
		var e: Dictionary = cats[i]
		var title := String(e["title"])
		var idx := "%02d" % (i + 1)
		_app.call("select_domain_category", "cartography", title)
		await _frames(3)
		var body: Control = e["body"]
		var wrap := body.get_parent() as Control
		var open_now := PackedStringArray()
		for e2 in cats:
			if (e2["body"] as Control).visible:
				open_now.append(String(e2["title"]))
		var in_tree := body.is_visible_in_tree()
		var domain := String(_app.call("active_domain"))
		var opened: bool = body.visible and wrap != null and wrap.visible and in_tree \
			and open_now.size() == 1 and domain == "cartography"
		_ok("[%s] %s opens" % [idx, title], opened, "" if opened else
			"body.visible=%s wrap.visible=%s visible_in_tree=%s open=[%s] domain=%s"
			% [body.visible, wrap != null and wrap.visible, in_tree, ", ".join(open_now), domain])
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
		var counts := "records=%d visible=%d interactive=%d disabled=%d" % [recs.size(), n_vis, n_act, n_dis]
		_ok("[%s] %s has content" % [idx, title], recs.size() > 0 and n_vis > 0, counts)
		_emit({"kind": "COUNT", "cat_idx": idx, "category": title, "extra": counts.replace(" ", ";")})
		_check_inset(idx, title, body)

	# -- the dock outside every category (TOOLS block and the rest) ------------
	var dock: Array = []
	_walk(panel, panel, "", "(dock)", wraps, dock)
	var d_vis := 0
	for r in dock:
		d_vis += 1 if r["vis"] == "1" else 0
		_emit(r)
	var dcounts := "records=%d visible=%d" % [dock.size(), d_vis]
	print("  info (dock) outside every category: ", dcounts)
	_emit({"kind": "COUNT", "category": "(dock)", "extra": dcounts.replace(" ", ";")})

	_check_placements()

# -- Ruling L --------------------------------------------------------------------

## The CARTO half of `RAIL_NODES`, against `RAIL_WANT` and against the dock's
## own category titles: a `category` or an `owns` entry that names a category
## this dock no longer builds is a silent no-op in `Workspace.open_category()`,
## which is the regression `_railfold_probe.gd` §2/§3 exist to catch and the one
## a rename makes most easily.
func _check_rail(titles: PackedStringArray) -> void:
	var got: Array = []
	for n in (_app.get("RAIL_NODES") as Array):
		if String(n.get("domain", "")) != "cartography":
			continue
		if String(n.get("kind", "")) == "head":
			_emit({"kind": "RAILHEAD", "text": String(n["label"])})
			continue
		_rail_labels[String(n["mode"])] = String(n["label"])
		got.append([String(n["mode"]), String(n["label"]), String(n.get("category", ""))])
		_emit({"kind": "RAIL", "category": String(n.get("category", "")), "text": String(n["label"]),
			"extra": "mode=%s;owns=%s" % [String(n["mode"]),
				",".join(PackedStringArray(n.get("owns", [])))]})
	_ok("the rail's CARTO nodes are Ruling L's four", str(got) == str(RAIL_WANT),
		"got=%s" % str(got))
	var owned := {}
	var bad := PackedStringArray()
	for n in (_app.get("RAIL_NODES") as Array):
		if String(n.get("domain", "")) != "cartography" or String(n.get("kind", "")) != "node":
			continue
		if not titles.has(String(n.get("category", ""))):
			bad.append("category %s" % String(n.get("category", "")))
		for c in (n.get("owns", []) as Array):
			if not titles.has(String(c)):
				bad.append("owns %s" % String(c))
			owned[String(c)] = int(owned.get(String(c), 0)) + 1
	for t in titles:
		if int(owned.get(t, 0)) != 1:
			bad.append("%s owned x%d" % [t, int(owned.get(t, 0))])
	_ok("every CARTO category is named by exactly one node, and every node names a real one",
		bad.is_empty(), "bad=[%s]" % " | ".join(bad))

## A wanted caption ending in `*` is a prefix match; everything else is exact,
## upper-cased. The caption is looked for on the record's own `text` **or** on
## the `row` Label `DccWidgets._row()` drew beside it, because a `toggle()` is a
## textless `CheckBox` and its caption is a sibling.
static func _caption_hit(rec: Dictionary, want: String) -> bool:
	var w := want.to_upper()
	var a := String(rec["text"]).to_upper()
	var b := String(rec["row"]).to_upper()
	if w.ends_with("*"):
		var p := w.substr(0, w.length() - 1)
		return a.begins_with(p) or (not b.is_empty() and b.begins_with(p))
	return a == w or (not w.is_empty() and b == w)

func _check_placements() -> void:
	for row: Array in PLACEMENTS:
		var want := int(row[5])
		var got := 0
		for r: Dictionary in _ctl:
			if String(r.get("kind", "")) != "CTL":
				continue
			if (row[0] == "*" or r["category"] == row[0]) and (row[1] == "*" or r["section"] == row[1]) \
					and (row[2] == "*" or r["groups"] == row[2]) and r["class"] == row[3] \
					and _caption_hit(r, String(row[4])):
				got += 1
		_ok("placed x%d: %s | %s | %s | %s \"%s\"" % [want, row[0], row[1], row[2], row[3], row[4]],
			got == want, "got=%d" % got)

## Measured while the category is open, against a `§` heading in the same body,
## so the number is the section's own rather than a literal; the rows' x is read
## off the expander's body pad (position plus its left margin), which is laid
## out whether or not the expander is open.
func _check_inset(idx: String, title: String, body: Control) -> void:
	for row: Array in INSET_EXPANDERS:
		if String(row[0]) != title:
			continue
		var hdr: Control = null
		var sec: Control = null
		var stack: Array = [body]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			if n.is_queued_for_deletion():
				continue
			if hdr == null and n is Button and (n as Button).is_visible_in_tree() \
					and String(n.get("text")).to_upper() == String(row[1]).to_upper():
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
		_ok("[%s] %s's %s heading and rows draw at the § inset" % [idx, title, row[1]],
			hdr != null and sec != null and absf(hx - sx) < 0.5 and absf(rx - sx) < 0.5,
			"heading_x=%d rows_x=%d section_x=%d (%s)" % [int(hx), int(rx), int(sx),
				(sec as Label).text if sec != null else "no § heading"])

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
	_ok("no SCRIPT ERROR anywhere in the run", script_errors.is_empty(), "script_errors=%d" % script_errors.size())
	var out := _arg("--out", "")
	if not out.is_empty():
		var f := FileAccess.open(out, FileAccess.WRITE)
		_ok("census written to " + out, f != null, "" if f != null else "open error %d" % FileAccess.get_open_error())
		if f != null:
			f.store_line("# _cartocensus_probe leg=%s viewport=%s seed=%d %s godot=%s" % [
				_leg, LEG_SIZE[_leg], _seed, _prefs, Engine.get_version_info()["string"]])
			f.store_line("\t".join(PackedStringArray(COLUMNS)))
			for r in _rows:
				f.store_line(r)
			f.store_line("# verdict: %s" % ("PASS" if _fail == 0 else "%d FAILURE(S)" % _fail))
			f.close()
	print("\n_cartocensus_probe [%s]: %s" % [_leg, "PASS" if _fail == 0 else "%d FAILURE(S)" % _fail])
	OS.remove_logger(_tap)
	get_tree().quit(1 if _fail > 0 else 0)

# -- records ---------------------------------------------------------------------

func _emit(rec: Dictionary) -> void:
	rec["leg"] = _leg
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
	## `"cartography"`, not `"civilization"`: this function came over from
	## `_civilcensus_probe.gd` with the rest of the record machinery and kept
	## that probe's domain id, so every HEAD row recorded `owner=` empty (no
	## CIVIL mode owns "Style") and `lit=landmarks` (whatever CIVIL mode was
	## last lit) -- a census column that looked like data and was another
	## domain's. Nothing asserts on it, which is why it survived the run.
	var owner := String(_app.call("mode_for_category", "cartography", title))
	var lit := String(_app.call("active_mode", "cartography"))
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
	return {"kind": "CTL", "cat_idx": idx, "category": cat,
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
## the nearest ancestor's inside the walked root (`DccWidgets._row()` puts a
## row's tooltip on the row, not on the control in it).
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

## `DccWidgets.section()` adds [MarginContainer > Label "§ TITLE"] then
## [MarginContainer > VBox body]; `group()`/`advanced()` add [flat Button
## "› TITLE"] then [MarginContainer > VBox body]. So a VBox that is the only
## child of a MarginContainer whose previous sibling is one of those two headers
## is a section or group body. Returns [innermost section, groups outer->inner].
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
## only. The engine wires its own layout plumbing into every Control through C++
## method pointers, which `Callable.get_method()` names `Class::method` --
## measured on this binary at HEAD de30275: `Container::_child_minsize_changed`,
## `Container::_child_desired_size_changed`, `Control::_size_changed`,
## `Label::_maximum_size_changed`, `Viewport::canvas_parent_mark_dirty`,
## `SpinBox::_update_buttons_state_for_current_value`. Those move with the
## container, not the control. A GDScript method or lambda cannot carry `::`.
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
