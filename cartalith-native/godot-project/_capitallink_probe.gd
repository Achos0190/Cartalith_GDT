extends Node
## PC-7 follow-up probe: the faction roster's "Capital: X" is a link that selects the capital
## (`PC_SHELL_PROPOSAL.md` 3.6, flow F3), through the one selection path.
##
##   desktop: godot --path . --resolution 1600x1000 _capitallink_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 _capitallink_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, from a wiped user dir (`MISTAKES.md`). Exit 0 pass, 1 fail, 2 bad
## argument / watchdog / no world.
##
## What it protects. A small world is generated (seed 4242, 256 x 192); the Factions window is opened on a
## faction that holds settlements, on its Territory tab; its "Capital: X ›" link is REAL-CLICKED. Then:
##   * `DccApp.selection` is `{settlement, i}` where `i` is a settlement of THAT faction and its name is the
##     capital the tab's own note names ("Capital: <name>"), and the Context panel shows that settlement;
##   * on the desktop the window stays open (the context window follows, PC-7); on the touch tablet, where
##     the window is modal, it closes so the selection is visible;
##   * a faction that holds no settlement (added empty) has NO link and its note says "Capital: none", so no
##     settlement 0 is offered as a stand-in.
var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _touch := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


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


## Every label's text under `n`, joined, so a note can be searched.
func _texts(n: Node) -> String:
	var out := PackedStringArray()
	for l in n.find_children("*", "Label", true, false):
		out.append((l as Label).text)
	return "\n".join(out)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a == "--force-touch":
			_touch = true
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
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
	print("== mode %s ==" % ("tablet" if DccTheme.is_tablet() else "desktop"))
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
	await _settle(6)
	await _leg_link()
	await _leg_empty_faction()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _leg_link() -> void:
	print("== the capital link ==")
	var fid := -1
	for f in _bridge.get_factions():
		if int((f as Dictionary).get("settlement_count", 0)) >= 2:
			fid = int((f as Dictionary)["id"])
			break
	_ok(fid > 0, "a faction with settlements exists (id %d)" % fid)
	if fid < 0:
		return
	_app.clear_selection()
	_app.open_faction_roster(fid, "territory")
	await _settle(10)
	var win = _app.faction_roster_window
	_ok(win.visible, "the Factions window is open")
	var link := win.find_child("CapitalLink", true, false) as Button
	_ok(link != null and link.is_visible_in_tree(), "the Territory tab draws a 'Capital:' link")
	if link == null:
		return
	var note := _texts(win)
	var shown_name := link.text.trim_prefix("Capital: ").trim_suffix(" ›")
	_ok(note.find("Capital: %s" % shown_name) >= 0, "the tab's own note names the same capital '%s'" % shown_name)
	await _click_at(_to_root(link.get_global_rect().get_center(), link))
	var sel: Dictionary = _app.selection
	_ok(String(sel.get("kind", "")) == "settlement", "the click selected a settlement (got %s)" % str(sel))
	var idx := int(sel.get("id", -1))
	var all: Array = _bridge.settlements()
	_ok(idx >= 0 and idx < all.size() and int((all[idx] as Dictionary).get("faction", 0)) == fid,
		"it is a settlement of faction %d" % fid)
	_ok(idx >= 0 and idx < all.size() and String((all[idx] as Dictionary).get("name", "")) == shown_name,
		"its name is the capital's, '%s'" % shown_name)
	_ok(_app.right_dock_ctrl.shown_subject() == {"kind": "settlement", "id": idx}, "the Context panel shows it")
	if _touch:
		_ok(not win.visible, "on the modal (tablet) window the link closes it, so the selection is visible")
	else:
		_ok(win.visible, "on the desktop the context window stays open and follows")
	_app.clear_selection()
	win.hide()
	await _settle(4)


func _leg_empty_faction() -> void:
	print("== a faction with no settlement ==")
	var before: int = _bridge.civ_faction_count()
	var nid: int = _bridge.civ_add_faction()
	await _settle(6)
	_ok(nid > 0 and _bridge.civ_faction_count() == before + 1, "an empty faction was added (id %d)" % nid)
	if nid <= 0:
		return
	_app.open_faction_roster(nid, "territory")
	await _settle(10)
	var win = _app.faction_roster_window
	_ok(win.find_child("CapitalLink", true, false) == null, "no Capital link for a faction that holds no settlement")
	_ok(_texts(win).find("Capital: none") >= 0, "its note says 'Capital: none'")
	win.hide()
