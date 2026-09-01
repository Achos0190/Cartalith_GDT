extends Node
## Committed probe -- verifies the three defects the 2026-08-25
## "is every control wired" sweep found and fixed, each driven for real:
##
##   MN-10  Assets ▸ Asset pack ▸ Pack metadata… reached no handler at all.
##   RL-01  CIVIL ▸ Relationships pair rows opened one side of the pair, so
##          consecutive rows sharing that side were a visible no-op.
##   CA-20  CARTO ▸ Clear all labels / Clear all icons were live over an empty
##          list, with no count and no stated reason.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _wiredfix_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("WIREDFIX  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("WIREDFIX  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _texts(root: Node) -> String:
	var all: Array = []
	_walk(root, all)
	var parts := PackedStringArray()
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
		elif n is Button:
			parts.append("%s|%s" % [(n as Button).text, str((n as Button).disabled)])
	return "\n".join(parts)


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)


func _popup(named: String) -> PopupMenu:
	var pops: Array = []
	_collect_popups(_app, pops)
	for p in pops:
		if (p as Node).name == named:
			return p
	return null


func _find_button(n: Node, needle: String) -> Button:
	if n is Button and String((n as Button).text).find(needle) >= 0:
		return n as Button
	for c in n.get_children(true):
		var r := _find_button(c, needle)
		if r != null:
			return r
	return null


func _cat(ws: Node, title: String) -> Control:
	var cats: Array = []
	cats.append_array(ws.categories)
	for extra in ["_infra", "_render"]:
		if ws.get(extra) != null:
			cats.append_array((ws.get(extra) as Node).categories)
	for e in cats:
		var d: Dictionary = e
		if String(d["title"]) == title:
			for e2 in cats:
				((e2 as Dictionary)["body"] as Control).visible = false
			(d["body"] as Control).visible = true
			return d["body"]
	return null


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
	_p("world: %d settlements, %d factions" % [
		_bridge.settlements().size(), _bridge.get_factions().size()])

	# ================================================================= MN-10
	_p("=== MN-10 : Assets ▸ Asset pack ▸ Pack metadata… ===")
	var ap := _popup("AssetPack")
	if ap == null:
		_bad("no AssetPack popup")
	else:
		var conns := ap.get_signal_connection_list("id_pressed")
		_p("AssetPack.id_pressed connections: %d" % conns.size())
		if conns.is_empty():
			_bad("AssetPack still has no id_pressed handler")
		var idx := -1
		for i in ap.item_count:
			if ap.get_item_text(i).find("Pack metadata") >= 0:
				idx = i
		if idx < 0:
			_bad("no Pack metadata row")
		else:
			_app.asset_library_window.hide()
			await _frames(3)
			var was: bool = _app.asset_library_window.visible
			ap.id_pressed.emit(ap.get_item_id(idx))
			await _frames(10)
			var now: bool = _app.asset_library_window.visible
			_p("asset library visible: before=%s after=%s" % [str(was), str(now)])
			if was or not now:
				_bad("Pack metadata… did not open the Asset Library window")
			_app.asset_library_window.hide()
			await _frames(4)

	# ================================================================= RL-01
	_p("=== RL-01 : CIVIL ▸ Relationships pair rows ===")
	_app._select_domain("civilization")
	await _frames(4)
	var civ := _find(_app, "civilization_workspace.gd")
	var body := _cat(civ, "Relationships")
	await _frames(4)
	if body == null:
		_bad("no Relationships category")
	else:
		var rows: Array = []
		_walk(body, rows)
		var pair_rows: Array = []
		for n in rows:
			if n is Button and String((n as Button).text).find("↔") >= 0:
				pair_rows.append(n)
		_p("%d pair rows" % pair_rows.size())
		if pair_rows.size() < 3:
			_bad("expected several pair rows, found %d" % pair_rows.size())
		## The exact failure that was measured: press every row in list order
		## and assert the dock text moves on EVERY one, including the runs of
		## rows that share a left-hand faction.
		var dead := 0
		var prev := ""
		for b in pair_rows:
			(b as Button).pressed.emit()
			await _frames(6)
			var dock := _texts(_app.right_dock_body)
			if dock == prev:
				dead += 1
				_p("   DEAD  %s" % (b as Button).text)
			prev = dock
		if dead > 0:
			_bad("%d of %d pair rows changed the right dock not at all" % [dead, pair_rows.size()])
		else:
			_p("PASS  all %d pair rows moved the dock" % pair_rows.size())

		## And the dock really draws the pair, not just the one side.
		var first := pair_rows[0] as Button
		first.pressed.emit()
		await _frames(6)
		var dock_txt := _texts(_app.right_dock_body)
		## "Aurelia ↔ Korrath -- wary (-22)" -> both names must appear.
		var lhs := first.text.split(" ↔ ")[0].strip_edges()
		var rhs := first.text.split(" ↔ ")[1].split(" -- ")[0].strip_edges()
		_p("row '%s': lhs=%s rhs=%s" % [first.text, lhs, rhs])
		if dock_txt.find("RELATIONS") < 0:
			_bad("the faction dock has no Relations section")
		if dock_txt.find(rhs) < 0:
			_bad("the dock never names the other party '%s'" % rhs)
		if dock_txt.find("▸ %s" % rhs) < 0:
			_bad("the clicked pair is not marked in the dock")
		else:
			_p("PASS  the dock marks '▸ %s' among %s's relations" % [rhs, lhs])
		for line in dock_txt.split("\n"):
			if String(line).find(rhs) >= 0 or String(line).find("Relations") >= 0:
				_p("   dock> %s" % line)

	# ================================================================= CA-20
	_p("=== CA-20 : CARTO ▸ Clear all labels / icons ===")
	_app._select_domain("cartography")
	await _frames(4)
	var carto := _find(_app, "cartography_workspace.gd")
	for spec in [["Labels", "Clear all labels", "label"], ["Assets & landmarks", "Clear all icons", "icon"]]:
		var cbody := _cat(carto, String(spec[0]))
		await _frames(4)
		if cbody == null:
			_bad("no %s category" % spec[0])
			continue
		var btn := _find_button(cbody, String(spec[1]))
		if btn == null:
			_bad("no '%s' button" % spec[1])
			continue
		_p("%-22s empty: text='%s' disabled=%s tip=%d chars" % [
			spec[0], btn.text, str(btn.disabled), btn.tooltip_text.length()])
		if not btn.disabled:
			_bad("'%s' is live over an empty list" % spec[1])
		if btn.tooltip_text.strip_edges() == "":
			_bad("'%s' is disabled with no stated reason" % spec[1])

	## Place two real labels and one real icon, then assert both come back with
	## a count on them. A gate that never re-opens is RF-01 wearing a new hat.
	_bridge.label_create(60.0, 50.0, "Northreach")
	_bridge.label_create(90.0, 70.0, "Sundered Vale")
	await _frames(2)
	carto._rebuild_label_panel()
	await _frames(4)
	var lbody := _cat(carto, "Labels")
	await _frames(3)
	var lbtn := _find_button(lbody, "Clear all labels")
	_p("labels after 2 adds: text='%s' disabled=%s" % [lbtn.text, str(lbtn.disabled)])
	if lbtn.disabled:
		_bad("Clear all labels stayed dead after two real labels were placed")
	if lbtn.text.find("(2)") < 0:
		_bad("Clear all labels does not carry the count (%s)" % lbtn.text)
	## And it really clears.
	lbtn.pressed.emit()
	await _frames(6)
	lbtn = _find_button(_cat(carto, "Labels"), "Clear all labels")
	await _frames(3)
	_p("labels after clear: text='%s' disabled=%s  engine list=%d" % [
		lbtn.text, str(lbtn.disabled), _bridge.label_list().size()])
	if _bridge.label_list().size() != 0:
		_bad("Clear all labels left %d behind" % _bridge.label_list().size())
	if not lbtn.disabled:
		_bad("Clear all labels stayed live after clearing to zero")

	_check_bindings()
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
