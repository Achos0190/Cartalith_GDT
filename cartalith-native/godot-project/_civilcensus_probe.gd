extends Node
## CIVIL census: every control the CIVIL dock draws, category by category, as a
## TSV that can be diffed control-for-control across the owner's 2026-09-13
## CIVIL re-sort (`LARGE_ITEM_RULINGS.md` Ruling L;
## `design/owner-references-2026-09-12/left_rail_tree_resorted.md`).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _civilcensus_probe.tscn -- --out C:/abs/civil_desktop.tsv
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _civilcensus_probe.tscn -- --force-touch --out C:/abs/civil_phone.tsv
##
## Two processes, because `DccShell._touch` is decided once per process from the
## command line. Without `--force-touch` this is the DESKTOP leg (1920x1080);
## with it, the PHONE leg (1080x2340), where the CIVIL dock is the left sheet
## and is opened the way a user opens it: `PhoneMenu._go_civilization()`, the
## MORE ▸ Civilization row.
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
## enough identity -- class, text, tooltip, disabled + reason, script-side
## signal wiring -- to be matched across a move by what it is rather than where
## it sits. The assertions below are about the *state* the shell reached, never
## about a value this probe wrote.
##
## **Exit 1** on any of: a category the shell's own jump
## (`select_domain_category`, which returns nothing) did not open, read back from
## the accordion -- body and wrap visible, visible in tree, and the only open body
## in the dock; a category with no content, or none of it visible; a dock with no
## categories; the wrong density; no world; the CIVIL dock not on screen; a
## Ruling L placement that does not hold (`PLACEMENTS`); the categories not in
## Ruling L's order (`RULING_L_ORDER`); CIVIL not entering on its Populate
## floor, or entering with a CIVIL mode other than its seeded `landmarks` (on
## desktop, also a rail foot not reading LANDMARKS); a category-level expander
## not drawn at the § inset (`INSET_EXPANDERS`); Travel asking for more width
## than HEAD's 216; Populate's two buttons not disabled before any world, or not
## rebuilt enabled over one; on the phone leg, MORE ▸ Simulation model not
## landing on Timeline; and
## **any SCRIPT ERROR anywhere in the process**, caught by an `OS.add_logger()`
## tap installed before the shell loads -- a handler that dies mid-build still
## leaves a half-filled category that a content check alone would pass. **Exit 3**
## is the watchdog. A parse error in this file cannot be caught from inside it:
## the scene then runs scriptless and never quits, so run it under a timeout.
##
## **What a record is.** One `CTL` row per Control, category bodies first and the
## dock outside them last, when the Control carries text or a tooltip -- or is
## interactive (button, range, text field, list), scripted, or has script-side
## signal connections, because a textless slider, a caption-less checkbox or a
## `gui_input` row is exactly what a text-only census loses unnoticed. `section`
## is the innermost `DccWidgets.section()` title and `groups` the
## `DccWidgets.group()`/`advanced()` titles outer >> inner, recovered from the
## structure those factories build (`_enclosing`), so both read as drawn
## (upper-cased). `nsig` counts script-side connections only -- `_sigs` says why
## the engine's own layout plumbing is left out. Empty cell = not applicable; a
## disabled control with no tooltip on itself or any ancestor has reason `NONE`.
##
## Other row kinds: `RAILHEAD`/`RAIL` (the CIVIL `RAIL_NODES` rows), `HEAD` (a
## category header, with the rail node that owns it and the one lit after the
## jump), `COUNT` (per category), `WIN` (a Window parented inside the dock), and
## `ERR` (every SCRIPT/ERROR/WARNING the tap saw).

const LEG_SIZE := {"desktop": Vector2i(1920, 1080), "phone": Vector2i(1080, 2340)}
const COLUMNS: Array[String] = ["kind", "leg", "cat_idx", "category", "section", "groups",
	"class", "row", "text", "tooltip", "disabled", "reason", "nsig", "sigs", "vis", "extra"]

## Ruling L's thirteen CIVIL categories in the tree's order
## (`left_rail_tree_resorted.md` L143-247), written out rather than read from the
## dock under test. A run on the pre-Ruling-L shell fails here, as it should.
const RULING_L_ORDER: Array[String] = ["Populate", "Settlements", "Landmarks",
	"Routes & ways", "Travel", "Factions", "Territories", "Relationships", "Military",
	"Culture", "Religion", "Economy", "Timeline"]

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
	## `Preferences ▸ Units` rewrites every distance this dock prints, so two
	## censuses are only comparable under the same setting. Recorded, not forced.
	_prefs = "units=%s" % DccSettings.units_mode()
	print("  info preferences: ", _prefs)

	# -- a world ---------------------------------------------------------------
	var bridge: Node = _app.get("bridge")
	## Ruling L's Populate section is built at boot, before any world, with both
	## buttons disabled, and only `_rebuild_readouts()` refills it (the plan's G4:
	## CIV-R). Read here and again over the world, so a refill that stops running
	## leaves this boot copy on screen and fails by name.
	_ok("before generation there is no world yet", not bool(bridge.get("has_world")))
	var pop_boot := _populate_buttons()
	_ok("before any world: Populate's two buttons exist once each, disabled",
		_populate_state(pop_boot, true), _populate_detail(pop_boot))
	var t0 := Time.get_ticks_msec()
	bridge.call("generate", {"seed": _seed, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bool(bridge.get("generating")):
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _frames(8)
	_ok("a world was generated", bool(bridge.get("has_world")),
		"seed=%d grid=256x192 %d ms" % [_seed, Time.get_ticks_msec() - t0])

	# -- CIVIL on screen -------------------------------------------------------
	if touch:
		var picker = _app.get("phone_project_picker")
		if picker != null and picker.visible:
			print("  info the phone project picker was still up after generation; the probe hid it")
			picker.hide()
		var pm = _app.get("_phone_menu")
		var can: bool = pm != null and pm.has_method("_go_civilization")
		_ok("PhoneMenu can open the CIVIL sheet (MORE ▸ Civilization)", can)
		if can:
			pm.call("_go_civilization")
	else:
		_app.call("select_domain", "civilization")
	await _frames(4)
	var panel: Control = _app.call("workspace_panel", "civilization")
	_ok("CIVIL is the active domain", String(_app.call("active_domain")) == "civilization")
	_ok("the CIVIL dock is on screen", panel != null and panel.is_visible_in_tree())
	if panel == null:
		return
	var pop_world := _populate_buttons()
	_ok("over the world: Populate's two buttons were rebuilt, enabled",
		_populate_state(pop_world, false), _populate_detail(pop_world))

	var cats: Array = panel.get("categories")
	_ok("the CIVIL dock builds at least one category", not cats.is_empty(), "n=%d" % cats.size())
	var titles := PackedStringArray()
	for e in cats:
		titles.append(String(e["title"]))
	print("  info %d categories in dock order: %s" % [cats.size(), " | ".join(titles)])
	_ok("the dock builds Ruling L's thirteen, in the tree's order",
		" | ".join(titles) == " | ".join(PackedStringArray(RULING_L_ORDER)),
		"got=[%s]" % " | ".join(titles))
	## Ruling L L144: Populate is "the tab's default-open floor", so entering CIVIL
	## -- the rail on desktop, MORE ▸ Civilization on phone -- lands on it. Read
	## before the jump loop below, which opens every category in turn.
	var open_at_entry := PackedStringArray()
	for e in cats:
		if (e["body"] as Control).visible:
			open_at_entry.append(String(e["title"]))
	_ok("entering CIVIL opens its floor, Populate, and nothing else",
		", ".join(open_at_entry) == "Populate", "open=[%s]" % ", ".join(open_at_entry))
	## ...and the rail. `DccShell._domain_mode` seeds CIVIL from its first
	## `RAIL_NODES` node, `landmarks`, and nothing writes it before entry --
	## HEAD's value, kept on purpose. Populate is owned by `factions`, so the lit
	## node does not own the open category; the one build that closed that gap (a
	## boot-time `apply_domain_mode()` in `CivilizationWorkspace._build()`) also
	## moved `app.gd::_on_workspace_changed()`'s re-entry baseline, and a Way
	## draft armed before the first in-CIVIL navigation then kept or lost its
	## Commit/Discard row differently from HEAD (2026-09-13 panel, B1/B2). How to
	## close the gap -- node order, or a Settlements mode id -- is the owner's
	## call (plan C1). A build that writes CIVIL's mode before entry fails here.
	var entry_mode := String(_app.call("active_mode", "civilization"))
	var entry_owner := String(_app.call("mode_for_category", "civilization",
		open_at_entry[0] if open_at_entry.size() == 1 else ""))
	_ok("entering CIVIL leaves the rail at its seeded node, landmarks (HEAD's entry mode; closing the gap to Populate's owner is plan C1, the owner's)",
		entry_mode == "landmarks", "mode=%s owner_of_open=%s" % [entry_mode, entry_owner])
	if not touch:
		var foot: Variant = _app.get("rail_foot")
		var foot_text := (foot as Label).text if foot is Label else "<none>"
		_ok("...and the rail foot reads LANDMARKS", foot_text == "LANDMARKS",
			"foot=[%s]" % foot_text)

	for n in (_app.get("RAIL_NODES") as Array):
		if String(n.get("domain", "")) != "civilization":
			continue
		if String(n.get("kind", "")) == "head":
			_emit({"kind": "RAILHEAD", "text": String(n["label"])})
		elif String(n.get("kind", "")) == "node":
			_rail_labels[String(n["mode"])] = String(n["label"])
			_emit({"kind": "RAIL", "category": String(n.get("category", "")), "text": String(n["label"]),
				"extra": "mode=%s;owns=%s" % [String(n["mode"]),
					",".join(PackedStringArray(n.get("owns", [])))]})

	# -- every category, opened by the shell's jump and read back by state -----
	var wraps := {}
	for e in cats:
		wraps[(e["body"] as Control).get_parent()] = true
	for i in cats.size():
		var e: Dictionary = cats[i]
		var title := String(e["title"])
		var idx := "%02d" % (i + 1)
		_app.call("select_domain_category", "civilization", title)
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
			and open_now.size() == 1 and domain == "civilization"
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
		_check_travel_width(idx, title, body, touch)

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
	if touch:
		await _check_phone_timeline_jump(cats)

# -- Ruling L --------------------------------------------------------------------

## Ruling L's moves, by the identity each control is recorded with
## (`design/owner-references-2026-09-12/left_rail_tree_resorted.md` CIVIL;
## `LARGE_ITEM_RULINGS.md` Ruling L): `[category, section, groups, class, text,
## how many]`, counted over the records the walk made, so a control the build
## dropped, duplicated or left in its old container fails here by name. `*`
## matches any category, section or groups; the dock-wide rows are the owner's
## three removals, each held at its one copy (or none). `-1` is "one when this
## world has what the row needs, else none" -- `_placement_want()`.
const PLACEMENTS: Array = [
	["Populate", "POPULATE", "", "Button", "Auto-populate world", 1],
	["Populate", "POPULATE", "", "Button", "Clear places & routes", 1],
	["Populate", "", "", "Button", "Placement model → File ▸ New world…", 1],
	["*", "*", "*", "Button", "Auto-populate world", 1],
	["Settlements", "", "", "Label", "§ DIAGNOSTICS", 1],
	["*", "*", "*", "Label", "§ DIAGNOSTICS", 1],
	["Settlements", "", "", "Label", "§ COASTAL SETTLEMENTS", 1],
	["Routes & ways", "", "", "Label", "§ NETWORK", 1],
	["*", "*", "*", "Label", "§ NETWORK", 1],
	["Routes & ways", "NETWORK", "", "Button", "Generate roads", 1],
	["Routes & ways", "NETWORK", "", "Button", "Clear ways & journeys", 1],
	["Routes & ways", "", "", "Button", "› SEA LANES", -1],
	["Travel", "JOURNEY PLANNING", "", "Button", "› ROUTES", 1],
	["Economy", "", "", "Label", "§ TRADE BALANCE", 1],
	["Economy", "", "", "Label", "§ TRADE FLOWS", 1],
	["Economy", "TRADE FLOWS", "", "Button", "Match trade flows", 1],
	["*", "*", "*", "Label", "§ FLOWS", 0],
	["Economy", "", "", "Button", "› BY FACTION", -1],
	["Culture", "PROFILES", "", "Button", "Which faction has which culture → Faction roster…", 1],
	["Timeline", "", "", "Button", "› SIMULATE COLLAPSE / RECOVERY", 1],
	["Timeline", "", "SIMULATE COLLAPSE / RECOVERY", "Button", "Simulate", 1],
]

## The two conditional rows ask the engine what the dock asks it.
func _placement_want(row: Array) -> int:
	var n := int(row[5])
	if n >= 0:
		return n
	var bridge: Node = _app.get("bridge")
	match String(row[4]):
		"› SEA LANES":
			return 0 if (bridge.call("sea_routes") as Array).is_empty() else 1
		"› BY FACTION":
			var balances: Array = bridge.call("trade_balances")
			var rows: Array = bridge.call("civ_faction_economy")
			return 0 if balances.is_empty() or rows.is_empty() else 1
	return 0

## Text is compared upper-cased: the phone composition draws every action
## button's caption in capitals (`CLEAR PLACES & ROUTES`), so a mixed-case row
## would miss a control that is there -- measured on the first phone run.
func _check_placements() -> void:
	for row: Array in PLACEMENTS:
		var want := _placement_want(row)
		var got := 0
		for r: Dictionary in _ctl:
			if String(r.get("kind", "")) != "CTL":
				continue
			if (row[0] == "*" or r["category"] == row[0]) and (row[1] == "*" or r["section"] == row[1]) \
					and (row[2] == "*" or r["groups"] == row[2]) and r["class"] == row[3] \
					and String(r["text"]).to_upper() == String(row[4]).to_upper():
				got += 1
		_ok("placed x%d: %s | %s | %s | %s \"%s\"" % [want, row[0], row[1], row[2], row[3], row[4]],
			got == want, "got=%d" % got)

## Ruling L's category-level expanders draw with a section's geometry: heading
## and rows at the § inset, 14 px in from the body's edge like the sections
## beside them (`CivilizationWorkspace.category_expander()`; Sea lanes L178, By
## faction L232, Simulate collapse / recovery L241). Measured while the category
## is open, against a `§` heading in the same body, so the number is the
## section's own rather than a literal; the rows' x is read off the expander's
## body pad (position plus its left margin), which is laid out whether or not
## the expander is open. The last column is the `PLACEMENTS` want: `-1` for the
## two the world may not build (`_placement_want`).
const INSET_EXPANDERS: Array = [["Routes & ways", "› SEA LANES", -1],
	["Economy", "› BY FACTION", -1], ["Timeline", "› SIMULATE COLLAPSE / RECOVERY", 1]]

func _check_inset(idx: String, title: String, body: Control) -> void:
	for row: Array in INSET_EXPANDERS:
		if String(row[0]) != title:
			continue
		if _placement_want(["*", "*", "*", "Button", row[1], int(row[2])]) == 0:
			print("  info [%s] %s builds no %s in this world; inset not measured" % [idx, title, row[1]])
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

## Travel asks for no more width than its own `§ Journey planning` notes do:
## `DccWidgets.note()`'s 190 floor plus the section's 14 + 12 margins = 216,
## which is what HEAD's Travel measured. Ruling L moved the Routes teaser into
## that section as an expander (L186), and a note nested one expander deeper
## costs 10 more (226) -- past the 232 px portrait-tablet dock, which draws
## content + 16. Desktop leg only: the floor is density-independent (a constant
## on the note and constant margins), but the phone composition sizes every row
## for touch and asks for far more; it is printed there, not asserted.
func _check_travel_width(idx: String, title: String, body: Control, touch: bool) -> void:
	if title != "Travel":
		return
	var bmin := body.get_combined_minimum_size().x
	if touch:
		print("  info [%s] Travel body minimum width on the phone leg: %d" % [idx, int(bmin)])
		return
	_ok("[%s] Travel asks for no more width than its Journey planning notes (190 + 14 + 12 = 216, HEAD's)" % idx,
		bmin <= 216.0, "body_min=%d" % int(bmin))

## MORE ▸ Simulation ▸ "Simulation model" (`PhoneMenu._go_simulation()`) names a
## CIVIL category by title, and Ruling L folds Simulation into Timeline
## (L236-247). So the row must land on Timeline, as the only open body, without
## `select_domain_category()`'s stale-pointer warning. Run last, after every
## record is taken, because it changes which category is open.
func _check_phone_timeline_jump(cats: Array) -> void:
	var pm = _app.get("_phone_menu")
	if pm == null or not pm.has_method("_go_simulation"):
		_ok("PhoneMenu can reach CIVIL ▸ Timeline (MORE ▸ Simulation ▸ Simulation model)", false)
		return
	var warned_before := (_tap.take()["warning"] as PackedStringArray).size()
	pm.call("_go_simulation")
	await _frames(4)
	var open_now := PackedStringArray()
	for e in cats:
		if (e["body"] as Control).visible:
			open_now.append(String(e["title"]))
	var warns: PackedStringArray = _tap.take()["warning"]
	var stale := PackedStringArray()
	for i in range(warned_before, warns.size()):
		if warns[i].contains("no category"):
			stale.append(warns[i])
	_ok("phone MORE ▸ Simulation model opens CIVIL ▸ Timeline, and only it",
		", ".join(open_now) == "Timeline" and stale.is_empty(),
		"open=[%s] stale=[%s]" % [", ".join(open_now), " | ".join(stale)])

## Populate's two buttons, found by caption inside the CIVIL dock's own Populate
## category -- upper-cased, since the phone draws action captions in capitals.
## Nodes already queued for deletion are skipped: `_clear_body()` frees the old
## copy when the section is refilled.
func _populate_buttons() -> Array:
	var out: Array = []
	var panel: Control = _app.call("workspace_panel", "civilization")
	if panel == null:
		return out
	for e in (panel.get("categories") as Array):
		if String(e["title"]) != "Populate":
			continue
		var stack: Array = [e["body"]]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			if n.is_queued_for_deletion():
				continue
			if n is BaseButton:
				var t := String(n.get("text")).to_upper()
				if t == "AUTO-POPULATE WORLD" or t == "CLEAR PLACES & ROUTES":
					out.append(n)
			stack.append_array(n.get_children())
	return out

## Exactly one of each button, every one in the wanted disabled state.
static func _populate_state(buttons: Array, want_disabled: bool) -> bool:
	var seen := {}
	for b in buttons:
		var t := String(b.get("text")).to_upper()
		seen[t] = int(seen.get(t, 0)) + 1
		if (b as BaseButton).disabled != want_disabled:
			return false
	return buttons.size() == 2 and int(seen.get("AUTO-POPULATE WORLD", 0)) == 1 \
		and int(seen.get("CLEAR PLACES & ROUTES", 0)) == 1

static func _populate_detail(buttons: Array) -> String:
	var parts := PackedStringArray()
	for b in buttons:
		parts.append("%s disabled=%s" % [String(b.get("text")), (b as BaseButton).disabled])
	return "n=%d [%s]" % [buttons.size(), "; ".join(parts)]

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
			f.store_line("# _civilcensus_probe leg=%s viewport=%s seed=%d %s godot=%s" % [
				_leg, LEG_SIZE[_leg], _seed, _prefs, Engine.get_version_info()["string"]])
			f.store_line("\t".join(PackedStringArray(COLUMNS)))
			for r in _rows:
				f.store_line(r)
			f.store_line("# verdict: %s" % ("PASS" if _fail == 0 else "%d FAILURE(S)" % _fail))
			f.close()
	print("\n_civilcensus_probe [%s]: %s" % [_leg, "PASS" if _fail == 0 else "%d FAILURE(S)" % _fail])
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
	var owner := String(_app.call("mode_for_category", "civilization", title))
	var lit := String(_app.call("active_mode", "civilization"))
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
