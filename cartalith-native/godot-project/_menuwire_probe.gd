extends Node
## TEMPORARY, untracked probe for the 2026-08-25 "is every control wired" pass.
##
## The menu bar, driven properly. `_railpress_probe.gd`'s first cut read
## `is_item_disabled()` straight off the popup and got four false positives,
## because half this shell's menus compute their gating in `about_to_popup`
## (File's Save/Show-on-disk, the asset-pack stats row, every GPU submenu).
## A menu never popped is a menu that has not been asked the question.
##
## So: fire `about_to_popup` on every popup first, THEN
##   1. report every enabled item whose `id_pressed` has no connection at all
##      -- the item cannot do anything, whatever its handler says;
##   2. press every enabled, non-submenu, non-checkable, non-destructive item
##      and report the ones that changed nothing anywhere;
##   3. report every disabled item with no tooltip that is not a caption row.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _menuwire_probe.tscn

var _app: Node
var _bridge
var _fail := 0

const SKIP := [
	"quit", "close project", "revert", "new world", "open project",
	"export", "import", "save", "clear", "delete", "generate world",
	"bake", "unfinalize", "purge", "wipe", "reset",
]

## Disabled rows that are *captions*, not capability claims: a section heading
## and a live stat line inside the asset-pack submenu, and File's own static
## "imports live elsewhere" note. Named explicitly so a genuinely undisclosed
## gap cannot hide behind a similar shape.
const CAPTIONS := ["Active pack", "Schema 2 ·", "Imports live under", "— loading —", "·"]


func _p(s: String) -> void:
	print("MENUWIRE  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("MENUWIRE  FAIL  %s" % s)


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
		elif n is Button:
			var b := n as Button
			parts.append("%s|%s|%s" % [b.text, str(b.disabled), str(b.button_pressed)])
		elif n is LineEdit:
			parts.append((n as LineEdit).text)
		elif n is Range:
			parts.append(str((n as Range).value))
		if n is Control:
			parts.append("v" if (n as Control).is_visible_in_tree() else "h")
		if n is Window:
			parts.append("W" if (n as Window).visible else "w")
	return "".join(parts)


func _close_windows() -> void:
	var all: Array = []
	_walk(_app, all)
	for n in all:
		if n is Window and (n as Window).visible:
			(n as Window).hide()


func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)


func _skipped(s: String) -> bool:
	var l := s.to_lower()
	for k in SKIP:
		if l.find(k) >= 0:
			return true
	return false


func _is_caption(s: String) -> bool:
	for c in CAPTIONS:
		if s.find(c) >= 0:
			return true
	return false


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
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

	var pops: Array = []
	_collect_popups(_app, pops)
	var menu_pops: Array = []
	for p in pops:
		var owner := (p as Node).get_parent()
		if owner is MenuButton or owner is PopupMenu:
			menu_pops.append(p)
	## Ask every menu its own gating question first.
	for p in menu_pops:
		(p as PopupMenu).about_to_popup.emit()
	await _frames(4)
	_p("%d command popups, each asked about_to_popup" % menu_pops.size())

	# ------------------------------------------------------------ 1. unwired
	_p("=== items whose popup has NO id_pressed connection ===")
	for p in menu_pops:
		var pm := p as PopupMenu
		var conns := pm.get_signal_connection_list("id_pressed") \
			+ pm.get_signal_connection_list("index_pressed")
		if not conns.is_empty():
			continue
		var actionable := PackedStringArray()
		for i in pm.item_count:
			if pm.is_item_separator(i) or pm.is_item_disabled(i):
				continue
			if pm.get_item_submenu(i) != "":
				continue
			actionable.append(pm.get_item_text(i))
		if actionable.size() > 0:
			_bad("popup '%s' has no id_pressed handler, yet carries %d live item(s): %s" % [
				pm.name, actionable.size(), ", ".join(actionable)])

	# -------------------------------------------------------------- 2. press
	_p("=== every live item, pressed ===")
	var n_items := 0
	var n_pressed := 0
	for p in menu_pops:
		var pm := p as PopupMenu
		for i in pm.item_count:
			if pm.is_item_separator(i):
				continue
			n_items += 1
			var txt := pm.get_item_text(i)
			if pm.is_item_disabled(i) or pm.get_item_submenu(i) != "" or pm.is_item_checkable(i):
				continue
			if _skipped(txt):
				continue
			var before := _snapshot()
			pm.id_pressed.emit(pm.get_item_id(i))
			await _frames(6)
			var after := _snapshot()
			_close_windows()
			await _frames(2)
			n_pressed += 1
			if before == after:
				_p("   NOCHANGE  %s :: '%s'" % [pm.name, txt])

	# ----------------------------------------------------- 3. honesty contract
	_p("=== disabled without a stated reason ===")
	var noreason := 0
	for p in menu_pops:
		var pm := p as PopupMenu
		for i in pm.item_count:
			if pm.is_item_separator(i) or not pm.is_item_disabled(i):
				continue
			var txt := pm.get_item_text(i)
			if _is_caption(txt):
				continue
			if pm.get_item_tooltip(i).strip_edges() == "":
				noreason += 1
				_bad("disabled with no reason: %s :: '%s'" % [pm.name, txt])
	_p("%d items total, %d pressed, %d disabled-without-reason" % [n_items, n_pressed, noreason])

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
