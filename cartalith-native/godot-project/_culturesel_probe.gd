extends Node
## PC-4 follow-up probe: a culture is selectable through the one selection path (`DccApp.select()`,
## kind "culture"), from the GUI, and everything that reads the selection follows. Closes the owner's
## item "Culture selection is unreachable from the GUI" (OUTSTANDING_WORK.md, PC row).
##
##   desktop: godot --path . --resolution 1600x1000 _culturesel_probe.tscn -- --shots <dir>
##   touch:   godot --path . --resolution 1600x1000 _culturesel_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`: a probe that reads `user://` can be green for reasons that have
## nothing to do with the code). Exit 0 pass, 1 fail, 2 bad argument / watchdog / no world.
##
## What it protects. A small world (seed 4242, 256 x 192) is generated, and a culture is selected FIVE
## ways, each by a real click or key event, never by calling `select()`:
##   1. FIND: Ctrl+F, the culture's name typed, its result row clicked. The row has no `x`/`y` (a culture
##      has no map position) and the click must not pan the map to (0, 0). The `c ` scope lists cultures.
##   2. SETTLEMENT page: a settlement with a holder is selected; Inspect's "Culture" link is clicked.
##   3. FACTION page: a faction is selected; Inspect's "Culture" link is clicked.
##   4. FAITH module: the faction's Faith tab; its "Select culture" row is clicked.
##   5. MAP CARD: the settlement is right-clicked on the map and the card's "Select culture" row clicked.
## After each: `DccApp.selection` is `{culture, i}`, the panel's `shown_subject()` and header name it,
## Inspect draws the existing culture card (`CultureCard`, `_build_culture_card`, one implementation) and
## the factions list, the assistant's chip names it; Ctrl+D clears all; Alt+← brings it back, Alt+→ goes
## forward again. Then: the Faith and Links modules and the Note tab render for the culture subject,
## a pinned panel keeps its culture, an out-of-range culture selects nothing, an unowned settlement has
## no culture link (no "Common" for unowned land), and the phone-form rows are not drawn on the phone.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
## `--only-card`: run just leg 5, for iterating on the context card's click.
var _only_card := false


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
		img.save_png(_shots.path_join("culsel_%s_%s.png" % [_mode, tag]))


func _click_at(at: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = button
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


func _dock():
	return _app.right_dock_ctrl


## A visible button in the Context panel's body that carries `ctx_target` == `target`, or null.
func _link_to(target: Dictionary) -> Button:
	for b in _app.right_dock_body.find_children("*", "Button", true, false):
		if b.has_meta("ctx_target") and b.get_meta("ctx_target") == target and (b as Control).is_visible_in_tree():
			return b
	return null


## Scrolls the panel so `c` is on screen, then real-clicks its centre.
func _press(c: Control) -> void:
	var sc = _app._right_dock_scroll
	if sc != null:
		sc.ensure_control_visible(c)
		await _settle(4)
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


func _is_culture(i: int) -> bool:
	return _app.selection == {"kind": "culture", "id": i}


## What every surface that reads the selection must say about culture `i`.
func _all_name(i: int, way: String) -> void:
	_ok(_is_culture(i), "%s: the one selection is culture %d (got %s)" % [way, i, str(_app.selection)])
	_ok(_dock().shown_subject() == {"kind": "culture", "id": i}, "%s: the panel shows culture %d" % [way, i])
	var nm := String((_bridge.get_cultures()[i] as Dictionary).get("name", ""))
	_ok(nm != "" and _dock().subject_name({"kind": "culture", "id": i}) == nm, "%s: the header names it '%s'" % [way, nm])
	await _settle(2)
	_ok(_app.right_dock_body.find_child("CultureCard", true, false) != null, "%s: Inspect draws the existing culture card" % way)
	_ok(_app.right_dock_body.find_child("CultureFactions", true, false) != null, "%s: Inspect lists the factions with it" % way)
	_ok(_app.assistant_window._shown_selection_text().find(nm) >= 0, "%s: the assistant's selection chip names it (got '%s')" % [way, _app.assistant_window._shown_selection_text()])


func _history(i: int, way: String) -> void:
	await _key(KEY_D, true)
	_ok(_app.selection.is_empty() and _dock().shown_subject().is_empty(), "%s: Ctrl+D clears the selection and the panel" % way)
	_ok(_app.assistant_window._shown_selection_text() == "", "%s: Ctrl+D empties the assistant's chip" % way)
	await _key(KEY_LEFT, false, true)
	await _all_name(i, way + " then Alt+←")
	await _key(KEY_RIGHT, false, true)
	_ok(_app.selection.is_empty(), "%s then Alt+→: forward reaches the cleared state again" % way)
	await _key(KEY_LEFT, false, true)
	_ok(_is_culture(i), "%s then Alt+← again: culture back" % way)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--only-card":
			_only_card = true
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
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(6)

	## A settlement with a holder and its culture; and a second culture, for "a click moves it".
	var sidx := -1
	var ci := -1
	var fid := -1
	var all: Array = _bridge.settlements()
	for i in all.size():
		var s: Dictionary = all[i]
		if int(s.get("faction", 0)) > 0 and String(s.get("name", "")).strip_edges() != "":
			var c: int = _app.culture_index(String(_dock()._faction_roster(int(s["faction"])).get("culture", "")))
			if c >= 0 and _visible_settlement_at(i):
				sidx = i
				ci = c
				fid = int(s["faction"])
				break
	_ok(sidx >= 0 and ci >= 0, "a named, held settlement is drawn in the map view (index %d, culture %d, faction %d)" % [sidx, ci, fid])
	if sidx < 0:
		get_tree().quit(2)
		return
	print("  info cultures: %d" % _bridge.get_cultures().size())

	if _only_card:
		await _leg_map_card(sidx, ci)
		print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
		get_tree().quit(0 if _fails == 0 else 1)
		return
	await _leg_control(ci)
	await _leg_find(ci)
	await _leg_settlement_page(sidx, ci)
	await _leg_faction_page(fid, ci)
	await _leg_faith_module(fid, ci)
	await _leg_map_card(sidx, ci)
	await _leg_modules(ci)
	await _leg_guards(sidx)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _settlement_pixel(i: int) -> Vector2:
	var s: Dictionary = _bridge.settlements()[i]
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())


func _visible_settlement_at(i: int) -> bool:
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_rect().grow(-60.0).has_point(_settlement_pixel(i))


## Control: the model called directly, and the stale-id rule.
func _leg_control(ci: int) -> void:
	print("== control (direct select()) ==")
	_app.select("culture", ci)
	await _settle(4)
	await _all_name(ci, "control")
	## The boundary: the first id past the end (`size()`), then a far one; a neighbour must never be chosen.
	var n_cultures: int = _bridge.get_cultures().size()
	_app.select("culture", n_cultures)
	await _settle(2)
	_ok(_app.selection.is_empty(), "culture id %d (one past the %d the engine lists) selects nothing (got %s)" % [n_cultures, n_cultures, str(_app.selection)])
	_app.select("culture", 9999)
	await _settle(2)
	_ok(_app.selection.is_empty(), "a far culture id selects nothing (got %s)" % str(_app.selection))
	_app.clear_selection()
	await _settle(2)


func _leg_find(ci: int) -> void:
	print("== 1. Find ==")
	var nm := String((_bridge.get_cultures()[ci] as Dictionary).get("name", ""))
	## The index rows themselves: a culture row has no position keys, and the `c ` scope is cultures only.
	var res: Dictionary = _app._search_result_set("c ")
	var rows: Array = res.get("rows", [])
	_ok(rows.size() == _bridge.get_cultures().size() and rows.all(func(r): return String(r.get("entity", "")) == "culture"),
		"the 'c ' scope lists exactly the engine's %d cultures (got %d)" % [_bridge.get_cultures().size(), rows.size()])
	_ok(rows.all(func(r): return not r.has("x") and not r.has("y")), "a culture row carries no x or y (no fake position)")
	## Zoomed in and panned off the corner, so a pan to cell (0, 0) would be visible as a camera move.
	_app.viewport.zoom_step(4.0)
	await _settle(4)
	_app.viewport.move_view_to(150.0, 110.0)
	await _settle(4)
	var cam_before: Vector2 = _app.viewport._camera.position
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
	var hit: Control = null
	for b in _app._desktop_search_results.find_children("*", "BaseButton", true, false):
		var texts: Array = []
		if b is Button:
			texts.append((b as Button).text)
		for l in (b as Node).find_children("*", "Label", true, false):
			texts.append((l as Label).text)
		if texts.any(func(t): return String(t).to_lower().contains("culture")) and texts.any(func(t): return String(t).to_lower().contains(nm.to_lower())):
			hit = b
			break
	_ok(hit != null, "Find lists the culture '%s'" % nm)
	if hit == null:
		return
	await _click_at(_to_root(hit.get_global_rect().get_center(), hit))
	await _settle(4)
	await _all_name(ci, "Find")
	_ok(_app.viewport._camera.position == cam_before,
		"the Find hit did not move the map (a culture has no place; camera %s -> %s)" % [str(cam_before), str(_app.viewport._camera.position)])
	await _shot("find")
	await _history(ci, "Find")
	_app.viewport.reset_view()
	await _settle(6)


func _leg_settlement_page(sidx: int, ci: int) -> void:
	print("== 2. the settlement's Inspect page ==")
	_app.clear_selection()
	_app.select("settlement", sidx)
	await _settle(6)
	var b := _link_to({"kind": "culture", "id": ci})
	_ok(b != null, "the settlement page has a Culture link to culture %d" % ci)
	if b == null:
		return
	await _press(b)
	await _settle(4)
	await _all_name(ci, "settlement page")
	await _history(ci, "settlement page")


func _leg_faction_page(fid: int, ci: int) -> void:
	print("== 3. the faction's Inspect page ==")
	_app.clear_selection()
	_app.select("faction", fid)
	await _settle(6)
	var b := _link_to({"kind": "culture", "id": ci})
	_ok(b != null, "the faction page has a Culture link to culture %d" % ci)
	if b == null:
		return
	await _press(b)
	await _settle(4)
	await _all_name(ci, "faction page")
	await _history(ci, "faction page")


func _leg_faith_module(fid: int, ci: int) -> void:
	print("== 4. the Faith module ==")
	_app.clear_selection()
	_app.select("faction", fid)
	await _settle(4)
	_ok(_app.open_context_tab("faith"), "the Faith tab opens for a faction")
	await _settle(8)
	var b := _app.right_dock_body.find_child("ModuleCultureLink", true, false) as Button
	_ok(b != null and b.is_visible_in_tree(), "the Faith module has a 'Select culture' row")
	if b == null:
		return
	await _press(b)
	await _settle(4)
	_ok(_is_culture(ci), "the Faith module's row selects culture %d (got %s)" % [ci, str(_app.selection)])
	_ok(_dock().current_tab() == "faith", "the Faith tab stays on (a culture fits it; got '%s')" % _dock().current_tab())
	_ok(_app.right_dock_body.find_child("CultureCard", true, false) != null, "the Faith tab now draws the culture's card")
	await _shot("faith")
	_app.open_context_tab("inspect")
	await _settle(4)
	await _history(ci, "Faith module")


func _leg_map_card(sidx: int, ci: int) -> void:
	print("== 5. the map's context card ==")
	_app.clear_selection()
	_app.select_domain("civilization")
	await _settle(6)
	if not _visible_settlement_at(sidx):
		_app.viewport.reset_view()
		await _settle(10)
	var at := _settlement_pixel(sidx)
	await _click_at(at, MOUSE_BUTTON_RIGHT)
	await _settle(8)
	var card = _app.context_broker.card if _app.get("context_broker") != null else null
	var row: Control = null
	if card != null:
		for l in (card as Node).find_children("Text", "Label", true, false):
			if (l as Label).text.begins_with("Select culture") and (l as Control).is_visible_in_tree():
				row = l
				break
	_ok(row != null, "the settlement's context card has a 'Select culture' row")
	if row == null:
		await _key(KEY_ESCAPE)
		return
	## The touch card is taller than the window: scroll the row into the card's own scroller first, so
	## the click lands on the row and not on the edge of the screen.
	var csc := row.get_parent()
	while csc != null and not (csc is ScrollContainer):
		csc = csc.get_parent()
	if csc != null:
		(csc as ScrollContainer).ensure_control_visible(row)
		await _settle(4)
	await _shot("card")
	var target :=_to_root(row.get_global_rect().get_center(), row)
	print("  info card row at %s (window %s, card visible %s, selection before %s)" % [str(target), str(row.get_window()), str(card.visible), str(_app.selection)])
	await _click_at(target)
	await _settle(6)
	print("  info after click: selection %s, card visible %s, back %s" % [str(_app.selection), str(card.visible), str(_app._selection_back.slice(-3))])
	await _all_name(ci, "map card")
	await _history(ci, "map card")


func _leg_modules(ci: int) -> void:
	print("== modules and the Note tab for a culture ==")
	_app.clear_selection()
	_app.select("culture", ci)
	await _settle(4)
	for key in ["faith", "links"]:
		_ok(_app.open_context_tab(key), "%s opens for a culture subject" % key)
		await _settle(8)
		_ok(_dock().current_tab() == key, "%s is the tab drawn for a culture (got '%s')" % [key, _dock().current_tab()])
		var texts := 0
		for l in _app.right_dock_body.find_children("*", "Label", true, false):
			if (l as Label).text.strip_edges() != "" and (l as Control).is_visible_in_tree():
				texts += 1
		_ok(texts > 0, "%s draws text for a culture (%d labels)" % [key, texts])
		if key == "faith":
			_ok(_app.right_dock_body.find_child("CultureCard", true, false) != null, "faith draws the culture card for a culture")
		else:
			_ok(_app.right_dock_body.find_child("ModuleLinks", true, false) != null, "links draws its section for a culture")
	_ok(_app.open_context_tab("note"), "the Note tab opens")
	await _settle(8)
	var r: Dictionary = _dock()._note.resolve({"kind": "culture", "id": ci})
	_ok(String(r.get("state", "")) == "unsupported" and String(r.get("name", "")) == _dock().subject_name({"kind": "culture", "id": ci}),
		"the Note tab names the culture and says it has no binding (state %s)" % str(r.get("state", "")))
	_app.open_context_tab("inspect")
	await _settle(4)
	## A pin holds the culture while another subject is selected.
	_ok(_dock().pin(), "the panel pins a culture")
	_app.select("faction", 1)
	await _settle(4)
	_ok(_dock().is_pinned() and _dock().shown_subject() == {"kind": "culture", "id": ci}, "a pinned panel keeps its culture")
	_dock().follow()
	await _settle(4)
	_ok(_dock().shown_subject() == {"kind": "faction", "id": 1}, "FOLLOW shows the new selection")
	_app.clear_selection()
	await _settle(2)


func _leg_guards(sidx: int) -> void:
	print("== no culture is invented ==")
	_app.clear_selection()
	## An unowned settlement: none in this world may be forged, so test the helper and the rule it feeds.
	_ok(_app.culture_index("") == -1, "an empty culture key has no index")
	_ok(_app.culture_index("no-such-culture") == -1, "an unknown culture key has no index")
	var unowned := -1
	for i in _bridge.settlements().size():
		if int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) <= 0:
			unowned = i
			break
	if unowned < 0:
		## None generated unowned: drop one with faction 0 (the engine's "unclaimed"), a few cells off a
		## known place, so the rule is exercised and not skipped.
		var s0: Dictionary = _bridge.settlements()[sidx]
		var tries := 0
		for dy in range(-14, 15, 2):
			for dx in range(-14, 15, 2):
				if unowned >= 0 or tries > 120 or (dx == 0 and dy == 0):
					continue
				tries += 1
				var di: int = _bridge.civ_drop_settlement(float(s0["x"]) + dx, float(s0["y"]) + dy, "village", 0, "Unowned probe", false)
				if di >= 0:
					if int((_bridge.settlements()[di] as Dictionary).get("faction", 0)) <= 0:
						unowned = di
		print("  info dropped an unowned settlement: index %d" % unowned)
	_ok(unowned >= 0, "an unclaimed settlement exists to test the no-culture rule")
	if unowned >= 0:
		_app.select("settlement", unowned)
		await _settle(6)
		var any_link := false
		for b in _app.right_dock_body.find_children("*", "Button", true, false):
			if b.has_meta("ctx_target") and String((b.get_meta("ctx_target") as Dictionary).get("kind", "")) == "culture":
				any_link = true
		_ok(not any_link, "an unclaimed settlement's page has no culture link")
	_app.clear_selection()
