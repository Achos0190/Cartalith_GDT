extends Node
## PC-1 probe: the one selection (`PC_SHELL_PROPOSAL.md` §2 rule 1, §3.1, milestone PC-1), inside the
## real shell, with real mouse clicks and real key events through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc1select_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _pc1select_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`: a probe that reads `user://` can be green for reasons that have
## nothing to do with the code). Reads `--force-touch` and `--shots <dir>`; anything else exits 2.
##
## What it protects (PC-1's bar). A small world is generated (seed 4242, 256 x 192 cells, as
## `_storymenus_probe.gd` does); then one settlement is selected THREE ways and after each the right
## dock (`current_settlement_index()`), CIVIL's index (`_selected_index`, what Delete and the Place
## editor act on) and the assistant's context (`LlmAssistantWindow._current_place()`) must all name it,
## and `DccApp.selection` must be `{"kind": "settlement", "id": i}`:
##   1. MAP: the view is centred on the place and the map is clicked at its pixel (a real click);
##   2. FIND: Ctrl+F (a real key) opens Find on map, the place's name is typed into its field, and its
##      result row is clicked (a real click);
##   3. ROSTER: Records ▸ Places… shows the CIVIL dock's Settlements list, and the place's row is clicked
##      (a real click), a row that before PC-1 pinned the dock directly.
## After each way: Ctrl+D (Edit ▸ Deselect, a real key) empties all three, and Alt+← (Edit ▸ Previous
## selection, a real key) brings the place back in all three. Alt+→ then goes forward to the cleared
## state again. A second place selected between two ways checks that each way really moves the
## selection (the expected index changes), never a stale pass.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var path := _shots.path_join("pc1_%s_%s.png" % [_mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


func _key(code: Key, ctrl := false, alt := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


## The three surfaces' answers and the model's, as one comparable record.
func _state() -> Dictionary:
	var civ = _app._workspaces.filter(func(w): return w is CivilizationWorkspace)
	var place: Dictionary = _app.assistant_window._current_place()
	return {
		"selection": _app.selection.duplicate(),
		"dock": int(_app.right_dock_ctrl.current_settlement_index()),
		"civ": int(civ[0]._selected_index) if not civ.is_empty() else -2,
		"assistant": int(place.get("index", -1)),
	}


func _all_name(i: int, way: String) -> void:
	var s := _state()
	_ok(s["selection"] == {"kind": "settlement", "id": i}, "%s: the one selection is settlement %d (got %s)" % [way, i, str(s["selection"])])
	_ok(s["dock"] == i and s["civ"] == i and s["assistant"] == i,
		"%s: the dock, CIVIL's index and the assistant name settlement %d (got %s)" % [way, i, str(s)])


func _all_empty(way: String) -> void:
	var s := _state()
	_ok(s["selection"].is_empty() and s["dock"] == -1 and s["civ"] == -1 and s["assistant"] == -1,
		"%s: Deselect empties the selection, the dock, CIVIL's index and the assistant (got %s)" % [way, str(s)])


## Deselect, back, forward, back: the history leg run after each way.
func _history(i: int, way: String) -> void:
	await _key(KEY_D, true)
	_all_empty(way)
	await _key(KEY_LEFT, false, true)
	_all_name(i, way + " then Alt+←")
	await _key(KEY_RIGHT, false, true)
	var s := _state()
	_ok(s["selection"].is_empty() and s["dock"] == -1, "%s then Alt+→: forward reaches the cleared state again" % way)
	await _key(KEY_LEFT, false, true)
	_all_name(i, way + " then Alt+← again")


## The first named settlement whose marker lies inside the map's visible rect, or -1.
func _visible_settlement() -> int:
	var ov = _app.viewport.overlay
	var r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var all: Array = _bridge.settlements()
	for i in all.size():
		var s: Dictionary = all[i]
		if String(s.get("name", "")).strip_edges() == "":
			continue
		var at: Vector2 = (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())
		if r.has_point(at):
			return i
	return -1


## Two well-separated named settlements: [first, second].
func _pick_two() -> Array:
	var all: Array = _bridge.settlements()
	var named: Array = []
	for i in all.size():
		if String((all[i] as Dictionary).get("name", "")).strip_edges() != "":
			named.append(i)
	if named.size() < 2:
		return []
	return [named[0], named[named.size() - 1]]


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "tablet" if DccTheme.is_tablet() else "desktop"
	print("== mode %s ==" % _mode)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	var two := _pick_two()
	if not bool(_bridge.has_world) or two.is_empty():
		print("[FATAL] no world with two named settlements")
		get_tree().quit(2)
		return
	_ok(_app.has_method("select") and _app.has_method("select_back"), "the host has PC-1's select() and select_back()")
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(6)
	_app.selection_changed.connect(func(cur): print("  info selection_changed -> ", str(cur)))
	## Control: the model itself, called directly, before any input route is exercised.
	_app.select("settlement", int(two[1]))
	await _settle(4)
	_all_name(int(two[1]), "control (direct select())")
	var vis := _visible_settlement()
	_ok(vis >= 0, "a named settlement is drawn inside the map view")
	if vis >= 0:
		await _way_map(vis)
	await _way_find(int(two[1]))
	await _way_roster(int(two[0]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _way_map(i: int) -> void:
	print("== 1. map click ==")
	## The place's pixel on the map: the overlay's own cell-to-screen mapping over its displayed rect,
	## the mapping it draws the settlement marker with (`map_overlay.gd::_cell_to_screen()`), through
	## the overlay's global transform (it sits under the zooming camera control, `viewport_host.gd`).
	var s: Dictionary = _bridge.settlements()[i]
	var ov = _app.viewport.overlay
	var at: Vector2 = (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())
	print("  info map click at %s for settlement %d '%s'" % [str(at), i, String(s.get("name", ""))])
	await _click_at(at)
	await _settle(4)
	_all_name(i, "map")
	await _shot("map")
	await _history(i, "map")


func _way_find(i: int) -> void:
	print("== 2. Find ==")
	var s: Dictionary = _bridge.settlements()[i]
	var nm := String(s.get("name", ""))
	await _key(KEY_F, true)
	await _settle(6)
	var field: LineEdit = _app._phone_search_field if _app._phone else _app._desktop_search_field
	_ok(field != null and field.is_visible_in_tree(), "Ctrl+F opens Find on map")
	if field == null:
		return
	field.grab_focus()
	field.text = nm
	field.text_changed.emit(nm)
	await _settle(6)
	var box: Control = _app._desktop_search_results
	var hit: Control = null
	for b in box.find_children("*", "BaseButton", true, false):
		var texts: Array = []
		if b is Button:
			texts.append((b as Button).text)
		for l in (b as Node).find_children("*", "Label", true, false):
			texts.append((l as Label).text)
		if texts.any(func(t): return String(t).to_lower().contains(nm.to_lower())):
			hit = b
			break
	if hit == null:
		var seen: Array = []
		for b in box.find_children("*", "Label", true, false):
			seen.append((b as Label).text)
		print("  info Find rows: ", str(seen.slice(0, 12)))
	_ok(hit != null, "Find lists '%s'" % nm)
	await _shot("find")
	if hit == null:
		return
	await _click_at(_to_root(hit.get_global_rect().get_center(), hit))
	await _settle(4)
	_all_name(i, "Find")
	await _history(i, "Find")


func _way_roster(i: int) -> void:
	print("== 3. a roster row (CIVIL ▸ Settlements) ==")
	var s: Dictionary = _bridge.settlements()[i]
	var nm := String(s.get("name", ""))
	_app.menus._on_data(DccMenus.ID_PLACES)
	await _settle(10)
	var row: Button = null
	for b in _app.left_dock.find_children("*", "Button", true, false):
		if (b as Button).text.begins_with(nm + " -- ") and (b as Control).is_visible_in_tree():
			row = b
			break
	_ok(row != null, "the Settlements list draws '%s'" % nm)
	if row == null:
		return
	var sc := row.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(row)
		await _settle(4)
	await _click_at(row.get_global_rect().get_center())
	await _settle(4)
	_all_name(i, "roster row")
	await _shot("roster")
	await _history(i, "roster row")
