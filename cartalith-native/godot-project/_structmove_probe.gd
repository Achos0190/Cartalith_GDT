extends Node
## Verification harness for the owner's 2026-09-05 structural-move ruling
## (`LARGE_ITEM_RULINGS.md`, "the five held groups", item 4):
##
##   1. Journey planner -- `Data ▸ Journey planner… ⇧J` becomes the CIVIL rail
##      node `planner` (`00-REPLACEMENT-PLAN.md` §1.1).
##   2. `Refine detail for the current view` -- `Preferences ▸ Tiles & LOD ▸
##      Atlas cache` becomes a button on the WORLD tool-options bar beside
##      `Bake ALL & finalize` (v3 `WORLD ▸ GENERATE ▸ › Bake & finalize`).
##   3. `Assets ▸ Asset pack ▸` -- four labelled bands become `03-menu-bar.md`
##      §6.3a's flat nine rows.
##
## Each move is checked three ways, which is what the brief asked for and what a
## relabel would fail: **gone from the old home**, **present at the new one**,
## and **still does the same thing**. The last one is the part reasoning cannot
## supply -- a rail node that lights but arms nothing looks identical in the
## scene graph to one that works.
##
## `Clear library…` gets its own section, because the ruling names it: the flat
## shape has no room for it and it must not be silently dropped. What this
## asserts is the claim the code makes -- that it was never only in the
## expansion, so the surviving row is the design's own §6.3 row 13.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _structmove_probe.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SM %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Every `PopupMenu` in the tree, including a `MenuButton`'s internal child --
## `get_children(true)`, per `_v3menu_probe.gd`'s own finding that the default
## walk skips it entirely.
func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)

func _popup(nm: String) -> PopupMenu:
	var found: Array = []
	_collect_popups(app, found)
	for p in found:
		if String((p as Node).name) == nm:
			return p
	return null

## The `MenuBar` popup whose `MenuButton` carries `title`. Located by the button,
## not by one of its own rows -- `_v3menu_probe.gd:387` finds the Data popup by
## searching it for "Journey planner", which this pass deletes, and that locator
## needs re-pointing (reported).
func _menu(title: String) -> PopupMenu:
	var buttons: Array = []
	_gather(app, buttons)
	for mb in buttons:
		if String((mb as MenuButton).text) == title:
			return (mb as MenuButton).get_popup()
	return null

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _rows(p: PopupMenu) -> Array:
	var out: Array = []
	if p == null:
		return out
	for i in p.item_count:
		out.append(p.get_item_text(i))
	return out

func _has_row(p: PopupMenu, needle: String) -> bool:
	for t in _rows(p):
		if String(t).findn(needle) >= 0:
			return true
	return false

func _row_index(p: PopupMenu, needle: String) -> int:
	for i in p.item_count:
		if p.get_item_text(i).findn(needle) >= 0:
			return i
	return -1

## Every `Button` under `n` whose text is exactly `text`.
func _button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text) == text:
		return n as Button
	for c in n.get_children(true):
		var r := _button(c, text)
		if r != null:
			return r
	return null

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
	print("SM world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("SM  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	# =====================================================================
	print("\n=== 1: Journey planner -- Data row gone, CIVIL rail node armed ===")

	var data := _menu("Data")
	_check("the Data menu is still there", data != null)
	if data != null:
		_check("1a OLD HOME: Data has no Journey planner row",
			not _has_row(data, "Journey planner"), "rows=%s" % [_rows(data)])
		## The neighbours it sat between must survive -- a deletion that took
		## the wrong row would also satisfy the check above.
		_check("1a control: Data still has World data tables and Travel library",
			_has_row(data, "World data tables") and _has_row(data, "Travel library"))

	var node_rows: Dictionary = app.get("_rail_node_rows")
	_check("1b NEW HOME: a civilization/planner rail row exists",
		node_rows.has("civilization/planner"), "keys=%s" % [node_rows.keys()])

	## Start from WORLD, so the press has to do the domain switch too.
	app.select_domain("world")
	await _frames(4)
	_check("1c precondition: WORLD active, journey not armed",
		app.active_domain() == "world" and app.armed_tool != "journey",
		"domain=%s tool=%s" % [app.active_domain(), app.armed_tool])

	var planner_row: Button = node_rows.get("civilization/planner")
	planner_row.pressed.emit()
	await _frames(12)
	_check("1c SAME THING: pressing the planner node arms the Journey tool",
		app.armed_tool == "journey", "tool=%s" % app.armed_tool)
	_check("1c: it selected CIVIL on the way",
		app.active_domain() == "civilization", "domain=%s" % app.active_domain())
	_check("1c: the planner's centre panel is on screen",
		app.journey_planner_view._center_panel.visible)
	_check("1c: the rail's own mode is planner",
		String(app.active_mode("civilization")) == "planner",
		"mode=%s" % app.active_mode("civilization"))

	## The sibling-node release. Without it the civ dock comes back *underneath*
	## a still-visible planner panel -- two left docks at once.
	var landmarks_row: Button = node_rows.get("civilization/landmarks")
	landmarks_row.pressed.emit()
	await _frames(12)
	_check("1d: a sibling CIVIL node releases the Journey takeover",
		app.armed_tool != "journey", "tool=%s" % app.armed_tool)
	_check("1d: the planner panel is down",
		not app.journey_planner_view._center_panel.visible)
	_check("1d: the civilization dock is back and alone",
		(app._workspace_panels["civilization"] as Control).visible
			and not app.journey_planner_view._left_panel.visible)

	## Every other route into the planner still lands, and now lights the node.
	app.select_domain("world")
	await _frames(4)
	app.open_journey_planner()
	await _frames(12)
	_check("1e: open_journey_planner() still arms it from WORLD",
		app.armed_tool == "journey" and app.active_domain() == "civilization")
	_check("1e: and lights the planner node it now lives on",
		String(app.active_mode("civilization")) == "planner")

	## The accelerator, driven through the same handler the OS reaches.
	app.arm_tool("inspect")
	app.select_domain("world")
	await _frames(6)
	var ev := InputEventKey.new()
	ev.keycode = KEY_J
	ev.shift_pressed = true
	ev.pressed = true
	app._unhandled_key_input(ev)
	await _frames(12)
	_check("1f: Shift+J still opens the planner with no menu row behind it",
		app.armed_tool == "journey" and app.active_domain() == "civilization",
		"tool=%s domain=%s" % [app.armed_tool, app.active_domain()])
	## A control that must NOT fire: bare J is the letter, not the command.
	app.arm_tool("inspect")
	await _frames(4)
	var plain := InputEventKey.new()
	plain.keycode = KEY_J
	plain.pressed = true
	app._unhandled_key_input(plain)
	await _frames(6)
	_check("1f control: bare J does nothing", app.armed_tool != "journey",
		"tool=%s" % app.armed_tool)

	# =====================================================================
	print("\n=== 2: Refine detail -- off the Atlas submenu, onto the WORLD bar ===")

	var atlas := _popup("AtlasCache")
	_check("the Atlas cache submenu is still there", atlas != null)
	if atlas != null:
		_check("2a OLD HOME: Atlas cache has no Refine row",
			not _has_row(atlas, "Refine"), "rows=%s" % [_rows(atlas)])
		_check("2a control: its other rows survived",
			_has_row(atlas, "Export atlas") and _has_row(atlas, "Import atlas")
				and _has_row(atlas, "Clear atlas cache"))

	## The WORLD bar is rebuilt on every domain switch, so select WORLD first.
	app.select_domain("world")
	await _frames(8)
	var refine := _button(app.tool_options_row, "Refine detail")
	_check("2b NEW HOME: a Refine detail button is on the WORLD tool bar",
		refine != null)
	var bake := _button(app.tool_options_row, "Bake ALL & finalize")
	_check("2b: beside Bake ALL & finalize, in the same row",
		refine != null and bake != null and refine.get_parent() == bake.get_parent(),
		"bake=%s" % [bake != null])
	if refine != null:
		_check("2b: it carries the shared limit text, not a fresh one",
			refine.tooltip_text == DccMenus.REFINE_TOOLTIP)

	## SAME THING. At a fitted view the pyramid is not up, so the honest answer
	## is the "nothing to refine at this zoom" refusal -- which is still proof
	## the press reaches `refine_current_view()`, because nothing else in the
	## shell writes that sentence. A silent press would leave the hint blank.
	if refine != null:
		app.set_status("hint", "", "text_dim")
		await _frames(3)
		refine.pressed.emit()
		await _frames(10)
		var hint := String(app.status_slot_text("hint"))
		_check("2c SAME THING: the press reaches refine_current_view()",
			hint.findn("refine") >= 0, "hint=%s" % hint)

	# =====================================================================
	print("\n=== 3: Asset pack -- four bands become nine flat rows ===")

	var ap := _popup("AssetPack")
	_check("the Asset pack submenu is still there", ap != null)
	if ap != null:
		ap.about_to_popup.emit()
		await _frames(2)
		var rows := _rows(ap)
		_check("3a: exactly nine rows", ap.item_count == 9,
			"count=%d rows=%s" % [ap.item_count, rows])
		_check("3a: row 1 is the ACTIVE PACK head",
			ap.is_item_separator(0) and String(rows[0]) == "ACTIVE PACK",
			"row0=%s sep=%s" % [rows[0], ap.is_item_separator(0)])
		var want := ["NAME", "AUTHOR", "LICENSE", "SCHEMA", "FILLED"]
		for i in 5:
			_check("3a: row %d is the %s readout" % [i + 2, want[i]],
				String(rows[i + 1]).begins_with(want[i])
					and ap.is_item_disabled(i + 1)
					and String(ap.get_item_metadata(i + 1)) == DccMenus.META_READOUT,
				"text=%s meta=%s" % [rows[i + 1], ap.get_item_metadata(i + 1)])
		for pair in [[6, "Pack metadata…"], [7, "Validate pack"], [8, "Export pack .zip…"]]:
			var idx: int = pair[0]
			_check("3a: row %d is %s, live" % [idx + 1, pair[1]],
				String(rows[idx]) == String(pair[1]) and not ap.is_item_disabled(idx),
				"text=%s" % rows[idx])
		## The bands themselves are gone, not merely re-ordered.
		for band in ["EDIT", "BATCH", "BUILD"]:
			_check("3b OLD SHAPE: no %s band" % band, not _has_row(ap, band),
				"rows=%s" % [rows])
		## And the nine window shortcuts with them.
		for gone in ["Tag…", "Collect into set", "Duplicate", "Slot transform",
				"Preview background", "Add variant", "Apply to map", "Import pack"]:
			_check("3b: %s is gone from the expansion" % gone, not _has_row(ap, gone))
		## FILLED must be a measurement, not a placeholder.
		var filled := String(rows[5])
		_check("3c: FILLED reports real slot counts",
			filled.findn("of") >= 0 and filled.findn("loading") < 0,
			"row=%s" % filled)
		_check("3c: and no invented byte size",
			filled.findn("MB") < 0 and filled.findn("GB") < 0, "row=%s" % filled)

	# -- `Clear library…`, the ruling's named consequence ------------------
	var assets := _menu("Assets")
	_check("the Assets menu is still there", assets != null)
	if assets != null and ap != null:
		_check("4: the expansion no longer carries Clear library",
			not _has_row(ap, "Clear library"))
		var ci := _row_index(assets, "Clear library")
		_check("4 NEW HOME: Assets ▸ Clear library… is present and live",
			ci >= 0 and not assets.is_item_disabled(ci), "index=%d" % ci)
		if ci >= 0:
			_check("4: and marked destructive, the design's danger:1",
				assets.get_item_text(ci).findn("destructive") >= 0,
				"text=%s" % assets.get_item_text(ci))
			_check("4 control: exactly one Clear library row in the whole menu",
				_count_rows(assets, "Clear library") == 1,
				"count=%d" % _count_rows(assets, "Clear library"))
		## The confirmation, checked rather than assumed to have survived: press
		## the row and assert a ConfirmationDialog appears and the library is
		## still intact until it is answered.
		var before: int = int(app.bridge.as_pack_info().get("total_items", 0))
		assets.id_pressed.emit(assets.get_item_id(ci))
		await _frames(8)
		var dlg := _find_confirm(app)
		_check("4 SAME THING: Clear library… still raises a confirmation",
			dlg != null)
		_check("4: and clears nothing until it is answered",
			int(app.bridge.as_pack_info().get("total_items", 0)) == before)
		if dlg != null:
			dlg.get_cancel_button().pressed.emit()
			await _frames(4)

	print("\nSM %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

func _count_rows(p: PopupMenu, needle: String) -> int:
	var n := 0
	for t in _rows(p):
		if String(t).findn(needle) >= 0:
			n += 1
	return n

func _find_confirm(n: Node) -> ConfirmationDialog:
	if n is ConfirmationDialog and (n as ConfirmationDialog).visible:
		return n as ConfirmationDialog
	for c in n.get_children(true):
		var r := _find_confirm(c)
		if r != null:
			return r
	return null
