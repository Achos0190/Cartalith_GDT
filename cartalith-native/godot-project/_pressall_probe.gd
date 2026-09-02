extends Node
## Committed probe -- the generic form of the failure this session
## keeps finding: **a control that exists and does nothing.**
##
## For one window at a time: snapshot every rendered string in the whole app,
## press one enabled button, re-snapshot, and report the buttons that changed
## nothing anywhere. A no-change press is not automatically a bug (a pure
## internal-state toggle, a clipboard copy) but it is the short list worth
## reading, and it is a list nothing else produces.
##
## Destructive and blocking labels are skipped by name, listed in SKIP below --
## a probe that deletes the roster it is auditing proves nothing.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pressall_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
## Only `_check_bindings()` raises this today -- see its header.
var _fail := 0
var _big_town := 0

const SKIP := [
	"delete", "remove", "browse", "import", "export", "save", "open project",
	"new world", "revert", "close project", "quit", "clear caches", "reset",
	"bake", "generate world", "new seed", "discard", "commit", "recompute",
	"recalculate", "auto-populate", "wipe", "purge", "unfinalize",
]


func _p(s: String) -> void:
	print("PRESSALL  %s" % s)


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


## Everything the app currently renders, as one string. Cheap, and sensitive
## to almost any real state change: a label, a button caption, a disabled
## flag, a visibility flag, an item count.
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
		elif n is OptionButton:
			parts.append("%s|%d" % [(n as OptionButton).text, (n as OptionButton).selected])
		elif n is LineEdit:
			parts.append((n as LineEdit).text)
		elif n is Range:
			parts.append(str((n as Range).value))
		elif n is ItemList:
			parts.append("IL%d/%s" % [(n as ItemList).item_count, str((n as ItemList).get_selected_items())])
		if n is Control:
			parts.append("v" if (n as Control).is_visible_in_tree() else "h")
	return "".join(parts)


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


func _press_all(title: String, root: Node) -> void:
	if root == null:
		_p("%s :: null" % title)
		return
	var all: Array = []
	_walk(root, all)
	var targets: Array = []
	for n in all:
		if n is BaseButton and not (n is OptionButton) and not (n is ColorPickerButton) \
				and not (n as BaseButton).disabled and (n as Control).is_visible_in_tree():
			targets.append(n)
	var dead: Array = []
	var pressed := 0
	var skipped := 0
	for b in targets:
		if not is_instance_valid(b) or not b.is_inside_tree():
			continue   # an earlier press rebuilt the panel out from under it
		var label := (b as Button).text if b is Button else ""
		if _skipped(label):
			skipped += 1
			continue
		var before := _snapshot()
		b.emit_signal("pressed")
		await _frames(5)
		var after := _snapshot()
		pressed += 1
		if before == after:
			var where := _path_of(root, b) if (is_instance_valid(b) and b.is_inside_tree()) else "<rebuilt away>"
			dead.append("%s   text='%s'" % [where, label])
	_p("---- %s : %d buttons, %d pressed, %d skipped, %d CHANGED NOTHING" % [
		title, targets.size(), pressed, skipped, dead.size()])
	for d in dead:
		_p("   NOCHANGE  %s" % d)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
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

	var st: Array = _bridge.settlements()
	var best := 0
	var best_pop := -1.0
	for i in st.size():
		var p: float = float(st[i].get("population", 0.0))
		if p > best_pop:
			best_pop = p
			best = i
	_big_town = best

	var wins := [
		["TravelLibrary", _app.travel_library_window, "open", []],
		["DataManager", _app.data_manager_window, "open", []],
		["FactionRoster", _app.faction_roster_window, "open", []],
		["PlaceEditor", _app.place_editor_window, "open_for", [_big_town]],
		["CityViewer", _app.city_viewer_window, "open", [_big_town]],
		["AssetLibrary", _app.asset_library_window, "open", []],
		["Vault", _app.vault_window, "open_overview", []],
		["WorldData", _app.world_data_window, "open", []],
		["Performance", _app.performance_window, "open", []],
		["LayersPopover", _app.layers_popover, "open", []],
	]
	for row in wins:
		var w = row[1]
		if w == null:
			_p("%s :: null" % row[0])
			continue
		w.callv(row[2], row[3])
		await _frames(8)
		await _press_all(row[0], w)
		if w.has_method("hide"):
			w.hide()
		await _frames(3)

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
