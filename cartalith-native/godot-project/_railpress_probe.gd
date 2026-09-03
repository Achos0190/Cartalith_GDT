extends Node
## Committed probe for the 2026-08-25 "is every control wired" pass.
##
## `_pressall_probe.gd` pressed every button in every WINDOW. It never touched
## the three domain rails (33 categories after the v3 restructure), the right
## dock in any of its eleven contexts, the tool-options row for any of the
## eleven tools, or the menu bar. This does.
##
## For each: snapshot every rendered string in the whole app, press one enabled
## control, close any window the press opened, re-snapshot, and report the ones
## that changed nothing anywhere. A no-change press is not automatically a bug
## -- a pure internal toggle, a clipboard copy -- but it is the short list
## nothing else produces.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _railpress_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
## Only `_check_bindings()` raises this today -- see its header.
var _fail := 0

## Destructive or long-blocking labels. A probe that regenerates the world it
## is auditing proves nothing.
const SKIP := [
	"delete", "remove", "browse", "import", "export", "save", "open project",
	"new world", "revert", "close project", "quit", "clear caches", "reset",
	"bake", "generate world", "new seed", "discard", "commit", "recompute",
	"recalculate", "auto-populate", "wipe", "purge", "unfinalize", "generate provinces",
	"run pipeline", "centre landmasses", "center landmasses", "simulate",
	"rebuild", "regenerate", "erode", "match trade flows", "borders & influence",
	"undo", "redo",
]

const DOMAINS := ["world", "civilization", "cartography"]
const TOOLS := ["inspect", "sculpt", "paint", "measure", "region",
	"settlement", "territory", "way", "route", "icon", "label"]


func _p(s: String) -> void:
	print("RAILPRESS  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	_p("FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _snapshot() -> String:
	var all: Array = []
	_walk(_app, all)
	var parts := PackedStringArray()
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
		elif n is OptionButton:
			parts.append("%s|%d" % [(n as OptionButton).text, (n as OptionButton).selected])
		elif n is Button:
			var b := n as Button
			parts.append("%s|%s|%s" % [b.text, str(b.disabled), str(b.button_pressed)])
		elif n is LineEdit:
			parts.append((n as LineEdit).text)
		elif n is Range:
			parts.append(str((n as Range).value))
		elif n is ItemList:
			parts.append("IL%d" % (n as ItemList).item_count)
		if n is Control:
			parts.append("v" if (n as Control).is_visible_in_tree() else "h")
		if n is Window:
			parts.append("W" if (n as Window).visible else "w")
	return "".join(parts)


func _path_of(root: Node, c: Node) -> String:
	var parts: Array = []
	var cur := c
	while cur != null and cur != root:
		parts.push_front(cur.name)
		cur = cur.get_parent()
	return "/".join(parts)


func _skipped(label: String) -> bool:
	var l := label.to_lower()
	for s in SKIP:
		if l.find(s) >= 0:
			return true
	return false


## Any Window the press put on screen, put back. Otherwise one modal swallows
## the rest of the run.
func _close_windows() -> void:
	var all: Array = []
	_walk(_app, all)
	for n in all:
		if n is Window and (n as Window).visible:
			(n as Window).hide()


func _press_all(title: String, root: Node) -> void:
	if root == null:
		_p("%s :: null" % title)
		return
	var all: Array = []
	_walk(root, all)
	var targets: Array = []
	var n_bb := 0
	var n_off := 0
	for n in all:
		if n is BaseButton and not (n is OptionButton) and not (n is ColorPickerButton):
			n_bb += 1
			if (n as BaseButton).disabled:
				continue
			if not (n as Control).is_visible_in_tree():
				n_off += 1
				continue
			targets.append(n)
	if n_bb > 0 and targets.is_empty():
		_p("   ?? %s : %d BaseButtons but 0 reachable (%d invisible), root.visible=%s vis_in_tree=%s" % [
			title, n_bb, n_off, str((root as Control).visible),
			str((root as Control).is_visible_in_tree())])
	var dead: Array = []
	var pressed := 0
	var skipped := 0
	for b in targets:
		if not is_instance_valid(b) or not b.is_inside_tree():
			continue
		var label := (b as Button).text if b is Button else ""
		var tip := (b as Control).tooltip_text
		if label.strip_edges() == "":
			## A `DccWidgets.toggle` is a text-less CheckBox whose caption is the
			## row Label beside it -- name it by that, or the report is a wall of
			## anonymous node ids.
			label = _row_label(b)
		if _skipped(label):
			skipped += 1
			continue
		var before := _snapshot()
		## `emit_signal("pressed")` on a toggle changes NO state and fires no
		## `toggled` -- every checkbox in the shell would read as dead. Drive the
		## real property instead, and put it back.
		var is_toggle: bool = (b as BaseButton).toggle_mode
		if is_toggle:
			(b as BaseButton).button_pressed = not (b as BaseButton).button_pressed
		else:
			b.emit_signal("pressed")
		await _frames(5)
		var after := _snapshot()
		if is_toggle and is_instance_valid(b):
			(b as BaseButton).button_pressed = not (b as BaseButton).button_pressed
			await _frames(2)
		_close_windows()
		await _frames(2)
		pressed += 1
		if before == after:
			var where := _path_of(root, b) if (is_instance_valid(b) and b.is_inside_tree()) else "<rebuilt away>"
			dead.append("%s   text='%s'   tip='%s'" % [where, label, tip.substr(0, 70)])
	_p("---- %s : %d buttons, %d pressed, %d skipped, %d CHANGED NOTHING" % [
		title, targets.size(), pressed, skipped, dead.size()])
	for d in dead:
		_p("   NOCHANGE  %s" % d)


## The caption of a `DccWidgets._row()` -- the first Label among the control's
## siblings.
func _row_label(c: Node) -> String:
	var p := c.get_parent()
	if p == null:
		return ""
	for s in p.get_children():
		if s is Label:
			return "«%s»" % (s as Label).text
	return ""


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _all_categories(ws: Node) -> Array:
	var out: Array = []
	if ws == null:
		return out
	out.append_array(ws.categories)
	for extra in ["_infra", "_render"]:
		if ws.get(extra) != null:
			out.append_array((ws.get(extra) as Node).categories)
	return out


func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)


## Every menu item, in the menu BAR only (nested submenus included), driven
## through `id_pressed` -- the same signal a real click emits.
func _drive_menus() -> void:
	var bar := _app.get_node_or_null("%s" % "") # unused; menus live under the header
	var pops: Array = []
	_collect_popups(_app, pops)
	## Only the seven program menus and their submenus: a PopupMenu owned by an
	## OptionButton is a value list, not a command.
	var menu_pops: Array = []
	for p in pops:
		var owner := (p as Node).get_parent()
		if owner is MenuButton or owner is PopupMenu:
			menu_pops.append(p)
	_p("---- menu bar : %d command popups" % menu_pops.size())
	var dead: Array = []
	var disabled_no_reason: Array = []
	var n_items := 0
	for p in menu_pops:
		var pm := p as PopupMenu
		for i in pm.item_count:
			if pm.is_item_separator(i):
				continue
			var txt := pm.get_item_text(i)
			n_items += 1
			if pm.is_item_disabled(i):
				if pm.get_item_tooltip(i).strip_edges() == "":
					disabled_no_reason.append("%s :: %s" % [pm.name, txt])
				continue
			if pm.get_item_submenu(i) != "" or pm.is_item_checkable(i):
				continue
			if _skipped(txt):
				continue
			var before := _snapshot()
			pm.id_pressed.emit(pm.get_item_id(i))
			await _frames(6)
			var after := _snapshot()
			_close_windows()
			await _frames(2)
			if before == after:
				dead.append("%s :: '%s'" % [pm.name, txt])
	_p("---- menu bar : %d items, %d CHANGED NOTHING, %d disabled-without-reason" % [
		n_items, dead.size(), disabled_no_reason.size()])
	for d in dead:
		_p("   MENU-NOCHANGE  %s" % d)
	for d in disabled_no_reason:
		_p("   MENU-NOREASON  %s" % d)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1500.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_p("world: %d settlements, %d factions, %d ways" % [
		_bridge.settlements().size(), _bridge.get_factions().size(), _bridge.roads().size()])

	## ---- the three rails, category by category ----------------------------
	var wss := {
		"world": _find(_app, "world_workspace.gd"),
		"civilization": _find(_app, "civilization_workspace.gd"),
		"cartography": _find(_app, "cartography_workspace.gd"),
	}
	for domain in DOMAINS:
		_app._select_domain(domain)
		await _frames(4)
		var ws: Node = wss[domain]
		## Opened one at a time, immediately before its own sweep: the accordion
		## closes siblings, and a press that rebuilds a category re-collapses the
		## rest -- the first cut of this probe reported "0 buttons" for twelve of
		## CIVIL's fourteen categories for exactly that reason.
		for e in _all_categories(ws):
			var entry2: Dictionary = e
			## Re-selected every time: the rails are full of jump buttons
			## ("→ Cartography ▸ Political display"), and one press leaves a
			## different workspace on screen -- every later category then reads
			## `is_visible_in_tree() == false` and the sweep silently covers
			## nothing. That is how the first run "passed" twelve of CIVIL's
			## fourteen categories.
			_app._select_domain(domain)
			await _frames(2)
			for e2 in _all_categories(ws):
				((e2 as Dictionary)["body"] as Control).visible = false
			(entry2["body"] as Control).visible = true
			await _frames(4)
			await _press_all("%s ▸ %s" % [domain.to_upper(), entry2["title"]], entry2["body"])

	## ---- the right dock, in each context it can be put into ---------------
	var rd = _app.right_dock_ctrl
	var st: Array = _bridge.settlements()
	var contexts := [
		["sample", func(): rd.on_cursor_sampled(100.0, 80.0, true)],
		["settlement", func(): rd.on_settlement_selected(st[0], 0)],
		["faction", func(): rd.show_faction(1)],
		["route", func(): rd.show_route(_bridge.roads()[0], "road")],
		# Labelled for the section it opens, not the armed tool: this calls
		# show_sculpt_stack() without arming sculpt or making a draft, so the
		# Stamp stack is legitimately absent and the pass is a cold one.
		["sculpt", func(): rd.show_sculpt_stack()],
		["history", func(): rd.show_history()],
		["measure", func(): rd.show_measure({}, "distance")],
	]
	for row in contexts:
		(row[1] as Callable).call()
		await _frames(6)
		## `RightDock` is a plain controller Node -- the panel it fills is
		## `app.right_dock_body`. Walking the controller finds no Controls at all.
		await _press_all("RightDock/%s" % row[0], _app.right_dock_body)

	## ---- every tool-options row, on the rail that owns the tool -----------
	var tool_domain := {
		"inspect": "world", "sculpt": "world", "paint": "world",
		"measure": "world", "region": "world",
		"settlement": "civilization", "territory": "civilization",
		"way": "civilization", "route": "civilization",
		"icon": "cartography", "label": "cartography",
	}
	for t in TOOLS:
		_app._select_domain(tool_domain[t])
		await _frames(3)
		_app.arm_tool(t)
		await _frames(5)
		await _press_all("ToolOptions/%s" % t, _app.tool_options_row)
	_app._select_domain("world")
	_app.arm_tool("inspect")
	await _frames(3)

	## ---- the section strip and the menu bar -------------------------------
	await _press_all("SectionStrip", _app.section_strip)
	await _drive_menus()

	_check_bindings()
	## Was `quit(0)` unconditionally. This probe reports rather than asserts,
	## so `_fail` is only ever set by `_check_bindings()` -- but that one check
	## is the difference between a census of the real shell and a census of a
	## shell running against a stale library, and a census taken against the
	## wrong binary is worse than no census.
	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## The staleness fingerprint, read off the shell instead of guessed at.
##
## `EngineBridge._has()` (`shell/engine_bridge.gd`) is the one choke point
## every binding guard in the shell goes through, and it records the name of
## each method the shell asked for that this build does not export;
## `EngineBridge.missing_bindings()` hands back the set. Nothing in this probe
## suite read it -- and a stale `target/debug/cartalith_godot.dll` has twice
## sent every `_has()` guard in a run down its degraded-fallback branch, which
## turns a whole sweep into a clean report over code that was never exercised.
## That is the failure mode this suite is least able to notice on its own, and
## the shell was already carrying the answer.
##
## Called last, after every surface this run drives has been driven: the set
## only fills as guards are reached, so an early read reports an empty one.
func _check_bindings() -> void:
	var mb: PackedStringArray = _bridge.missing_bindings()
	if mb.is_empty():
		return
	_bad("stale extension -- the shell asked for %d binding(s) this build "
		% mb.size()
		+ "does not export (%s). " % ", ".join(mb)
		+ "Every result above was measured against a degraded shell; rebuild "
		+ "the crates and re-run before believing any of it.")
