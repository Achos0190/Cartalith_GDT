extends Node
## SP-3 authored events (Ruling AM): a real vault note with a ```chronos block,
## attached to a real generated settlement, must show on the Settlement
## Editor's Political history tab -- sorted by year, range/group/description
## drawn, the malformed line counted, and the no-note state stated.
## Windowed (reads real drawn Label text):
##   Godot_v4.7.1-stable_win64_console.exe --path . _pechronos_probe.tscn
## Exit 0 pass, 1 real failure, 2 could not run.

const VAULT_DIR := "user://_pechronos_vault"
var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("PECHRONOS  %s" % s)


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _labels(n: Node, out: Array) -> Array:
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children(true):
		_labels(c, out)
	return out


func _has_sub(labels: Array, sub: String) -> int:
	for i in labels.size():
		if String(labels[i]).find(sub) >= 0:
			return i
	return -1


func _press_tab(pe: Node, text: String) -> void:
	var stack: Array = [pe]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Button and (n as Button).text == text:
			(n as Button).pressed.emit()
			return
		stack.append_array(n.get_children(true))


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	if not _bridge.has_method("vault_entity_chronos") or not _bridge.world_gen.has_method("vault_entity_chronos"):
		_p("ABORT: no vault_entity_chronos on this binary -- stale .dll?")
		get_tree().quit(2)
		return

	_bridge.generate({"seed": 5521, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var places: Array = _bridge.settlements()
	var idx := -1
	for i in places.size():
		if int((places[i] as Dictionary).get("tid", 0)) > 0:
			idx = i
			break
	if idx == -1:
		_p("ABORT: no settlement with a tid")
		get_tree().quit(2)
		return
	var s: Dictionary = places[idx]
	var tid := int(s.get("tid", 0))
	var name := String(s.get("name", "?"))
	_p("settlement idx=%d tid=%d name=%s" % [idx, tid, name])

	var pe = _app.place_editor_window
	pe.open_for(idx)
	await _frames(6)
	_press_tab(pe, "Political history")
	await _frames(6)
	_check("on the Political history tab", pe._active_tab == "political")
	var l0 := _labels(pe, [])
	_check("Authored events section drawn", _has_sub(l0, "AUTHORED EVENTS") >= 0 or _has_sub(l0, "Authored events") >= 0)
	_check("no-note state says so", _has_sub(l0, "No vault note linked") >= 0)

	# -- A real vault, a real note --------------------------------------------
	DirAccess.make_dir_recursive_absolute(VAULT_DIR + "/Settlements")
	var note := "# %s\n\nA river town.\n\n## History\n\nOlder than the walls.\n\n```chronos\n" % name \
		+ "- [1100~1180] #blue {Rule} Ashfall regency | the regent's peace\n" \
		+ "- [-250] #red Siege | the walls held\n" \
		+ "- [twelve] Typo line\n" \
		+ "* [1200-06-01] Charter granted\n" \
		+ "```\n"
	var f := FileAccess.open(VAULT_DIR + "/Settlements/Note.md", FileAccess.WRITE)
	f.store_string(note)
	f.close()
	var info: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(VAULT_DIR), "Probe vault")
	_check("vault connected", bool(info.get("ok", false)), String(info.get("error", "")))
	var att: Dictionary = _bridge.vault_attach("settlement", tid, name, "Settlements/Note.md", "")
	_check("note attached", bool(att.get("ok", false)), String(att.get("error", "")))

	var raw: Dictionary = _bridge.vault_entity_chronos("settlement", tid)
	var evs: Array = raw.get("events", [])
	_check("bridge returns 3 events", evs.size() == 3, str(evs.size()))
	if evs.size() == 3:
		_check("sorted by year: -250 first", int(evs[0].get("start", 0)) == -250)
		_check("single-year event has NO end key", not (evs[0] as Dictionary).has("end"))
		_check("range event carries end 1180", int(evs[1].get("end", 0)) == 1180)
		_check("point kind kept", String(evs[2].get("kind", "")) == "point")
		_check("full date read as its year", int(evs[2].get("start", 0)) == 1200)

	_press_tab(pe, "Overview")
	await _frames(4)
	_press_tab(pe, "Political history")
	await _frames(6)
	var l1 := _labels(pe, [])
	var i_siege := _has_sub(l1, "Siege")
	var i_reg := _has_sub(l1, "Rule · Ashfall regency")
	_check("Siege row drawn", i_siege >= 0)
	_check("grouped range row drawn with its group", i_reg >= 0)
	_check("rows in year order (-250 before 1100)", i_siege >= 0 and i_reg > i_siege)
	_check("negative year drawn", l1.has("-250"))
	_check("range drawn", l1.has("1100 – 1180"))
	_check("description drawn", _has_sub(l1, "the walls held") >= 0)
	_check("point row labelled", _has_sub(l1, "Charter granted  (point)") >= 0)
	_check("malformed line counted, not fatal", _has_sub(l1, "1 line in the chronos block could not be read") >= 0)
	_check("no-note message gone", _has_sub(l1, "No vault note linked") < 0)

	var img := get_viewport().get_texture().get_image()
	img.save_png("user://_pechronos_after.png")
	_p("screenshot %s" % ProjectSettings.globalize_path("user://_pechronos_after.png"))

	_press_tab(pe, "Timeline")
	await _frames(6)
	var l2 := _labels(pe, [])
	_check("Timeline tab no longer argues for a new store", _has_sub(l2, "that store does not exist") < 0)
	_check("Timeline tab points to Political history", _has_sub(l2, "shown on its Political history tab") >= 0)

	_bridge.vault_disconnect()
	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
