extends Node
## PC-7 probe: the context windows that do not block the map and follow the selection until pinned
## (`PC_SHELL_PROPOSAL.md` §3.6, milestone PC-7), inside the real shell, with real mouse clicks,
## wheel notches, middle drags and key events through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc7windows_probe.tscn
##   tablet:  godot --path . --resolution 1600x1000 _pc7windows_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`** (the busy overlay and the strip need frames), with a wiped user
## dir (the runner sets `APPDATA` to an empty folder). Reads `--force-touch` and `--mutant` (prints a
## marker only; the harness reads the exit code); anything else exits 2.
##
## What it protects (PC-7's bar: "a windowed probe clicks the map while the roster is open, and the
## map takes the click; the roster follows a new selection until pinned; the busy overlay mutes it
## during a generate", plus the brief's hazards). A small world is generated (seed 4242, 256 x 192, as
## `_pc4context_probe.gd`), two years are recorded (100, 255) and one conflict (250..260) is drawn.
## Desktop, for EACH of the five windows (roster, Place editor, City viewer, Conflicts, Saved
## journeys), opened through the shell's own open call:
##   W. the window is visible and NOT exclusive, carries the map-press relay's key node, and (all but
##      Saved journeys) a FOLLOW chip;
##   M. MAP: with the window focused (a real click inside it), a real left click on a settlement
##      outside it selects that settlement and the window stays open; a wheel notch over the map zooms
##      it, and one over the window does not; a middle drag on the map pans it;
##   F. FOLLOW: the window shows the new selection (roster: the selection's faction; Place editor and
##      City viewer: the settlement; Conflicts: marks the selected conflict, selected by a real click on
##      its own row, which leaves the window open);
##   Y. YEAR: a real click on a strip pill moves the year, and the window re-reads (Conflicts' readout
##      and ACTIVE chip name the new year and count; the others are rebuilt, the City viewer keeping
##      the user's zoom);
##   P. PIN: a real click on the chip pins it; a new map selection leaves the window on its subject;
##      a second click follows again and shows the selection; a reopen follows.
## Then, on the roster:
##   K. KEYS: typed into the faction name field, Q, Shift+A, Delete and Backspace stay in the field (no
##      cartwheel, no tool change, no selection change); with no field focused, Q still opens nothing
##      (the chord relay passes no plain key); Ctrl+F (a chord) opens Find and the next letter typed
##      lands in Find's field; the relay's rule is pinned by a table of literals.
##   H. HELD: with the religion editor open and a name typed into it, a map selection of another
##      faction's place PINS the roster (its faction, the editor and the typed name kept); the chip
##      asks before following; Cancel keeps all three; the second time, Discard and follow closes the
##      editor and follows. The Place editor's unsaved-event draft pins it the same way (the draft is
##      set as setup: its form needs a live vault note). A year move while the name field is focused
##      keeps the typed text (the redraw waits for the field to give up the focus).
##   O. OPEN WINDOWS: Window ▸ Open windows lists the open roster, and picking it does not re-centre a
##      window the user dragged.
##   B. BUSY: a generate started with the roster open mutes it (`gui_disable_input`), a real click on
##      its chip changes nothing, a click on the map selects nothing and a wheel over the map relayed
##      from the window zooms nothing; after the generate the roster takes input again.
## Tablet (`--force-touch`): every one of the five stays modal (exclusive, no chip, no relay), and a
## map click while the roster is open selects nothing (§3.11: "Its windows stay modal sheets").
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world, 3 a leg could not run.

## Loaded, not preloaded, so the probe also runs (and fails, as it must) on the tree before PC-7, where
## neither script exists; every use goes through the helpers below.
var _Relay: Script = load("res://shell/context_key_relay.gd") if ResourceLoader.exists("res://shell/context_key_relay.gd") else null
var _CW: Script = load("res://shell/context_window.gd") if ResourceLoader.exists("res://shell/context_window.gd") else null
## The names `context_window.gd` gives its chip and relay (literals, so the pre-PC-7 run can look).
const CHIP := "FollowChip"
const RELAY := "ContextKeyRelay"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _cid := -1


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _move(at: Vector2, mask := 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 1)
	mm.button_mask = mask
	Input.parse_input_event(mm)
	await _settle(2)


func _button(at: Vector2, idx: MouseButton, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = idx
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	if idx == MOUSE_BUTTON_MIDDLE and down:
		ev.button_mask = MOUSE_BUTTON_MASK_MIDDLE
	Input.parse_input_event(ev)
	await _settle(2)


func _click_at(at: Vector2) -> void:
	await _move(at)
	await _button(at, MOUSE_BUTTON_LEFT, true)
	await _button(at, MOUSE_BUTTON_LEFT, false)
	await _settle(4)


func _wheel(at: Vector2) -> void:
	await _move(at)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, true)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, false)
	await _settle(4)


func _middle_drag(from: Vector2, by: Vector2) -> void:
	await _move(from)
	await _button(from, MOUSE_BUTTON_MIDDLE, true)
	for i in 4:
		await _move(from + by * float(i + 1) / 4.0, MOUSE_BUTTON_MASK_MIDDLE)
	await _button(from + by, MOUSE_BUTTON_MIDDLE, false)
	await _settle(4)


func _key(code: Key, unicode := 0, shift := false, ctrl := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.shift_pressed = shift
		ev.ctrl_pressed = ctrl
		ev.unicode = unicode if down else 0
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


## Root-viewport position of a control drawn inside embedded window `w`.
func _in_win(w: Window, c: Control) -> Vector2:
	return Vector2(w.position) + c.get_global_rect().get_center()


func _press_in(w: Window, c: Control, what: String) -> bool:
	if c == null or not c.is_visible_in_tree():
		_ok(false, "%s is drawn to be pressed" % what)
		return false
	## Scrolled into its scroller's view first, and pressed only when its centre is drawn there.
	var sc: Node = c.get_parent()
	while sc != null and not (sc is ScrollContainer) and sc != w:
		sc = sc.get_parent()
	if sc is ScrollContainer:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(4)
		if not (sc as ScrollContainer).get_global_rect().has_point(c.get_global_rect().get_center()):
			_ok(false, "%s is inside its scroller's visible rect" % what)
			return false
	await _click_at(_in_win(w, c))
	await _settle(4)
	return true


func _screen_of(i: int) -> Vector2:
	var s: Dictionary = _bridge.settlements()[i]
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())


## Named settlements whose marker the overlay's own hit test gives to them, inside the map view and
## clear of window `w`'s frame, optionally only those whose faction is (or is not) `fac`.
func _clickable(w: Window, fac := -1, other := false) -> Array:
	var ov = _app.viewport.overlay
	var map_r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var win_r := Rect2(Vector2(w.position), Vector2(w.size)).grow(40.0) if w != null else Rect2()
	var rect: Rect2 = ov._displayed_rect()
	var out: Array = []
	var all: Array = _bridge.settlements()
	var dbg := [0, 0, 0, 0]
	for i in all.size():
		var s: Dictionary = all[i]
		if String(s.get("name", "")).strip_edges() == "":
			continue
		dbg[0] += 1
		if map_r.has_point(_screen_of(i)):
			dbg[1] += 1
			if not (w != null and win_r.has_point(_screen_of(i))):
				dbg[2] += 1
				if not _covered(_screen_of(i)):
					dbg[3] += 1
		var f := int(s.get("faction", 0))
		if fac >= 0 and ((f == fac) == other):
			continue
		var p := _screen_of(i)
		if not map_r.has_point(p) or (w != null and win_r.has_point(p)) or _covered(p):
			continue
		## Leave room above the strip and the bottom bar.
		if p.y > float(get_viewport().get_visible_rect().size.y) - 140.0:
			continue
		var local: Vector2 = (ov as Control).get_global_transform().affine_inverse() * p
		var pk: Dictionary = ov._pick_mark(local, ov._interior_rect(rect), rect)
		if int(pk.get("s", -1)) == i and int(pk.get("l", -1)) == -1:
			out.append(i)
	print("  info clickable: named %d, in map %d, clear of window %d, uncovered %d, picked %d; map %s vp %s win %s" % [dbg[0], dbg[1], dbg[2], dbg[3], out.size(), str(map_r), str((_app.viewport as Control).get_global_rect()), str(win_r)])
	return out


## Whether screen point `p` is under the shell's chrome that draws over the map (the docks, the
## status row, the strip, the bar above the map), so a click there would not reach the map.
func _covered(p: Vector2) -> bool:
	if not (_app.viewport as Control).get_global_rect().grow(-30.0).has_point(p):
		return true
	for n in _app._region_nodes.values():
		if n is Control and (n as Control).is_visible_in_tree() and (n as Control).get_global_rect().grow(20.0).has_point(p):
			return true
	if _app.timeline_row != null and (_app.timeline_row as Control).is_visible_in_tree() 			and (_app.timeline_row as Control).get_global_rect().grow(20.0).has_point(p):
		return true
	return false


## Whether `w` follows the selection (`context_window.gd::is_following`); false before PC-7.
func _following(w: Object) -> bool:
	return _CW != null and bool(_CW.is_following(w))


func _chip(w: Window) -> Button:
	return w.find_child(CHIP, true, false) as Button


## Presses the chip of `w` for real. The chip sits at the button row's right end, which a window
## dragged to hang past the screen's edge (the setup in `_open()`) can carry off screen; the window
## is then dragged back on screen for the press, as a user would, and returned.
func _press_chip(w: Window, what: String) -> bool:
	var chip := _chip(w)
	if chip == null:
		_ok(false, "%s: the chip exists" % what)
		return false
	var vs := get_viewport().get_visible_rect().size
	var at := _in_win(w, chip)
	var moved := Vector2i.ZERO
	if at.x > vs.x - 8.0:
		moved = Vector2i(int(at.x - vs.x) + 60, 0)
		w.position -= moved
		await _settle(4)
	var ok := await _press_in(w, chip, what)
	if moved != Vector2i.ZERO:
		w.position += moved
		await _settle(4)
	return ok


func _strip_pill(y: int) -> Button:
	var rec: Node = _app.timeline_row.find_child("TimelineRecords", true, false)
	if rec == null:
		return null
	for b in rec.find_children("*", "Button", true, false):
		if (b as Button).has_meta("tl_year") and int((b as Button).get_meta("tl_year")) == y and (b as Control).is_visible_in_tree():
			return b
	return null


## Moves the year with a real click on the strip's pill for `y`. False (and a failed check) when the
## pill is not drawn.
func _year_by_pill(y: int) -> bool:
	var p := _strip_pill(y)
	if p == null:
		_ok(false, "the strip draws a pill for year %d" % y)
		return false
	await _click_at(p.get_global_rect().get_center())
	await _settle(8)
	return _app.tl_year() == y


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--mutant" and k + 1 < args.size():
			k += 1
			print("MUTANT %s" % args[k])
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
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
	var mode := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s ==" % mode)
	if mode == "phone":
		print("[FATAL] this probe has no phone leg (the phone is guarded by captures)")
		get_tree().quit(3)
		return
	_bridge = _app.bridge
	await _generate()
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 3 or _bridge.get_factions().size() < 2:
		print("[FATAL] no world with three settlements and two factions")
		get_tree().quit(2)
		return
	_ok(_relay_table(), "the chord relay's rule holds for every row of its literal table")
	if mode == "tablet":
		await _leg_tablet()
	else:
		await _leg_desktop()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _generate() -> void:
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)


## `context_key_relay.gd::relays()` against literals: [keycode, ctrl, alt, typing, expected].
func _relay_table() -> bool:
	var rows := [
		[KEY_Q, false, false, false, false],   # a plain key is the map's: never passed
		[KEY_Q, false, false, true, false],
		[KEY_DELETE, false, false, false, false],
		[KEY_F, true, false, false, true],     # Ctrl+F with no field: passed
		[KEY_F, true, false, true, true],      # Ctrl+F while typing: passed (a field has no Ctrl+F)
		[KEY_S, true, false, true, true],
		[KEY_Z, true, false, true, false],     # the field's undo
		[KEY_A, true, false, true, false],
		[KEY_V, true, false, true, false],
		[KEY_LEFT, true, false, true, false],
		[KEY_LEFT, false, true, true, false],  # Alt+Left while typing: PQ12
		[KEY_LEFT, false, true, false, true],  # Alt+Left with no field: Previous selection
		[KEY_Z, true, false, false, true],
	]
	if _Relay == null:
		print("  info context_key_relay.gd does not exist in this tree")
		return false
	var good := true
	for r in rows:
		var got: bool = _Relay.relays(int(r[0]), bool(r[1]), bool(r[2]), bool(r[3]))
		if got != bool(r[4]):
			print("  info relay row %s answered %s" % [str(r), str(got)])
			good = false
	return good


# -- tablet ----------------------------------------------------------------------------------------

func _leg_tablet() -> void:
	print("== T. tablet: the five stay modal ==")
	var idx: int = _clickable(null)[0]
	_app.select("settlement", idx)
	for key in ["roster", "place", "city", "conflicts", "journeys"]:
		var w: AcceptDialog = await _open(key, idx)
		if w == null:
			continue
		_ok(w.visible and w.exclusive, "%s: open and still exclusive on the tablet" % key)
		_ok(_chip(w) == null, "%s: no FOLLOW chip on the tablet" % key)
		_ok(w.find_child(RELAY, true, false) == null, "%s: no chord relay on the tablet" % key)
		if key == "roster":
			var others := _clickable(w).filter(func(i): return i != idx)
			_ok(not others.is_empty(), "roster: a place outside the modal window to click (got %d)" % others.size())
			if not others.is_empty():
				await _click_at(_screen_of(int(others[0])))
				_ok(_app.selection == {"kind": "settlement", "id": idx},
					"roster: a map click while it is open selects nothing on the tablet (selection %s)" % str(_app.selection))
		w.hide()
		await _settle(6)


# -- desktop ---------------------------------------------------------------------------------------

## Opens window `key` through the shell's own open call and returns it (or null and a failed check).
func _open(key: String, idx: int) -> AcceptDialog:
	match key:
		"roster":
			_app.open_faction_roster(-1)
		"place":
			_app.open_place_editor(idx)
		"city":
			_app.open_city_viewer(idx)
		"conflicts":
			_app.open_conflicts_list()
		"journeys":
			_app.open_saved_journeys()
	await _settle(12)
	var w: AcceptDialog = _win(key)
	_ok(w != null and w.visible, "%s: the window opened" % key)
	## Setup: moved to the right edge, as a user drags a window aside, so the map's left half is
	## clear (a centred 880-940 px window leaves almost no map between it and the docks).
	if w != null and w.visible:
		## Its left edge 250 px inside the map's right edge; the rest hangs over the right dock and,
		## for the widest, past the screen's edge (an embedded window may), its chip and Close on screen.
		var mr := (_app.viewport as Control).get_global_rect()
		w.position = Vector2i(int(mr.end.x) - 250, 90)
		await _settle(4)
	return w if (w != null and w.visible) else null


func _win(key: String) -> AcceptDialog:
	match key:
		"roster":
			return _app.faction_roster_window
		"place":
			return _app.place_editor_window
		"city":
			return _app.city_viewer_window
		"conflicts":
			return _app.conflicts_window
		"journeys":
			return _app.saved_journeys_window
	return null


## What window `key` shows now, for the follow checks: a faction id, a settlement index or the
## marked conflict id.
func _shows(key: String) -> int:
	var w := _win(key)
	match key:
		"roster":
			return int(w._selected)
		"place", "city":
			return int(w._index)
		"conflicts":
			return int(w._marked)
	return -1


## What the selection asks window `key` to show.
func _wants(key: String) -> int:
	match key:
		"roster":
			return int(_app.current_faction())
		"place", "city":
			return int(_app.selection.get("id", -1)) if String(_app.selection.get("kind", "")) == "settlement" else -1
		"conflicts":
			return int(_app.selection.get("id", -1)) if String(_app.selection.get("kind", "")) == "conflict" else -1
	return -1


func _focus_inside(w: Window) -> void:
	## A click on an empty spot of the window's own content: top-left inside its margin.
	await _click_at(Vector2(w.position) + Vector2(6, 6))
	await _settle(2)


func _leg_desktop() -> void:
	## Setup: two recorded years (the strip's pills are the real year writer pressed below) and one
	## conflict between the first two factions, 250..260, so it is active at 255 and not at 100.
	var facs: Array = _bridge.get_factions()
	var f1 := int((facs[0] as Dictionary).get("id", 1))
	var f2 := int((facs[1] as Dictionary).get("id", 2))
	var all: Array = _bridge.settlements()
	var a_s: Dictionary = all[0]
	var r: Dictionary = _bridge.conflict_add({"kind": "battle",
		"points": PackedVector2Array([Vector2(float(a_s.get("x", 0)), float(a_s.get("y", 0)))]), "start_year": 250})
	_cid = int(r.get("id", -1))
	_bridge.conflict_update(_cid, {"name": "Probe War", "sides": PackedInt32Array([f1, f2]), "end_year": 260})
	for y in [100, 255]:
		_bridge.civ_add_year(y)
	_app.tl_set_year(100, true)
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(10)
	## Expand the strip so its pills draw (a real click on its collapsed hit button).
	for b in _app.timeline_row.find_children("*", "Button", true, false):
		if (b as Button).tooltip_text.begins_with("Expand the timeline") and (b as Control).is_visible_in_tree():
			await _click_at((b as Control).get_global_rect().get_center())
			break
	await _settle(8)
	_ok(_strip_pill(100) != null and _strip_pill(255) != null, "setup: the strip draws pills for 100 and 255")

	for key in ["roster", "place", "city", "conflicts", "journeys"]:
		await _leg_window(key)
	await _leg_keys()
	await _leg_held()
	await _leg_open_windows()
	await _leg_busy()


func _leg_window(key: String) -> void:
	print("== %s ==" % key)
	## Setup: the view back to its fit, so each window's legs start from the same map (the wheel and
	## drag checks of the window before moved it).
	_app.viewport.reset_view()
	await _settle(8)
	var start: Array = _clickable(null)
	if start.is_empty():
		print("[FATAL] no clickable settlement")
		get_tree().quit(3)
		return
	_app.select("settlement", int(start[0]))
	await _settle(6)
	var w: AcceptDialog = await _open(key, int(start[0]))
	if w == null:
		return
	var follows := key != "journeys"
	## W.
	_ok(not w.exclusive, "%s W: not exclusive" % key)
	_ok(w.find_child(RELAY, true, false) != null, "%s W: carries the chord relay" % key)
	_ok((_chip(w) != null) == follows, "%s W: FOLLOW chip %s" % [key, "present" if follows else "absent (no subject)"])
	if follows:
		_ok(_chip(w).text == "FOLLOW", "%s W: it opens following ('%s')" % [key, _chip(w).text])
	## M. map click with the window focused.
	await _focus_inside(w)
	_ok(w.has_focus(), "%s M: a click inside focuses the window" % key)
	var cands := _clickable(w).filter(func(i): return i != int(start[0]))
	if key == "roster":
		## A place of another faction, so the roster has somewhere to follow to.
		var shown := _shows("roster")
		cands = cands.filter(func(i): return int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) != shown \
			and int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) > 0)
	if cands.size() < 2:
		_ok(false, "%s M: two clickable places outside the window (got %d)" % [key, cands.size()])
		w.hide()
		return
	var tgt := int(cands[0])
	await _click_at(_screen_of(tgt))
	_ok(_app.selection == {"kind": "settlement", "id": tgt},
		"%s M: a map click outside the open window selects that place (selection %s, wanted %d)" % [key, str(_app.selection), tgt])
	_ok(w.visible, "%s M: the window stays open" % key)
	## F. follow (Conflicts: by its own row, below).
	if follows and key != "conflicts":
		var floor_id := 0 if key == "roster" else -1
		_ok(_shows(key) == _wants(key) and _wants(key) > floor_id,
			"%s F: the window shows the new selection (shows %d, selection asks %d)" % [key, _shows(key), _wants(key)])
	## Wheel over the map, the window focused again.
	await _focus_inside(w)
	var cam = _app.viewport._camera
	var z0: Vector2 = cam.scale
	await _wheel(_screen_of(int(cands[1])))
	_ok(not is_equal_approx(cam.scale.x, z0.x), "%s M: a wheel notch over the map zooms it with the window focused (%.4f -> %.4f)" % [key, z0.x, cam.scale.x])
	await _focus_inside(w)
	z0 = cam.scale
	await _wheel(Vector2(w.position) + Vector2(w.size) * 0.5)
	_ok(is_equal_approx(cam.scale.x, z0.x), "%s M: a wheel notch over the window does not zoom the map (%.4f -> %.4f)" % [key, z0.x, cam.scale.x])
	## Pan: a middle drag on the map, the window focused.
	await _focus_inside(w)
	var p0: Vector2 = cam.position
	await _middle_drag(_screen_of(int(cands[1])) + Vector2(0, -20), Vector2(60, 30))
	_ok(cam.position.distance_to(p0) > 20.0, "%s M: a middle drag on the map pans it with the window open (moved %.1f px)" % [key, cam.position.distance_to(p0)])
	await _settle(6)
	if key == "conflicts":
		await _leg_conflict_row(w)
	## Y. year by a real strip pill.
	await _leg_year(key, w)
	## P. pin.
	if follows:
		await _leg_pin(key, w)
	## Close with its own Close button (a real click).
	await _press_in(w, w.get_ok_button(), "%s Close" % key)
	_ok(not w.visible, "%s: Close closes it" % key)


func _leg_conflict_row(w: Window) -> void:
	var row_btn: Button = null
	for b in w.find_children("CfSelect", "Button", true, false):
		if (b as Button).text.contains("Probe War"):
			row_btn = b
	if not await _press_in(w, row_btn, "conflicts F: the Probe War row"):
		return
	_ok(w.visible, "conflicts F: a row press leaves the window open on the desktop")
	_ok(_app.selection == {"kind": "conflict", "id": _cid}, "conflicts F: the row press selects the conflict (%s)" % str(_app.selection))
	var marked := false
	for rw in w.find_children("CfRow", "", true, false):
		if int(rw.get_meta("conflict_id", -1)) == _cid:
			marked = bool(rw.get_meta("selected", false))
	_ok(marked and _shows("conflicts") == _cid, "conflicts F: the selected conflict's row is marked")


func _leg_year(key: String, w: Window) -> void:
	var cam = _app.viewport._camera
	var before_body: Array = []
	for c in w.find_children("*", "Control", true, false):
		before_body.append((c as Object).get_instance_id())
	var city_zoom: float = float(w._zoom) * 1.7 if key == "city" else 0.0
	if key == "city":
		w._zoom = city_zoom
	var to := 255 if _app.tl_year() != 255 else 100
	var moved := await _year_by_pill(to)
	_ok(moved, "%s Y: a strip pill moved the year to %d (now %d)" % [key, to, _app.tl_year()])
	await _settle(8)
	if key == "journeys":
		return
	if key == "conflicts":
		var ro: Label = w.find_child("CfReadout", true, false)
		_ok(ro != null and ro.text.contains("active in year %d" % to), "conflicts Y: the readout names year %d ('%s')" % [to, ro.text if ro else ""])
		var chip: Button = w.find_child("CfActiveChip", true, false)
		var want := "ACTIVE IN %d · %d" % [to, 1 if to == 255 else 0]
		_ok(chip != null and chip.text == want, "conflicts Y: the ACTIVE chip reads '%s' (got '%s')" % [want, chip.text if chip else ""])
		return
	var fresh := 0
	for c in w.find_children("*", "Control", true, false):
		if not before_body.has((c as Object).get_instance_id()):
			fresh += 1
	_ok(fresh > 0, "%s Y: the window was redrawn for the new year (%d new controls)" % [key, fresh])
	if key == "city":
		_ok(is_equal_approx(float(w._zoom), city_zoom), "city Y: the year re-read keeps the user's zoom (%.3f, set %.3f)" % [float(w._zoom), city_zoom])


func _leg_pin(key: String, w: Window) -> void:
	var chip := _chip(w)
	if not await _press_chip(w, "%s P: the chip" % key):
		return
	_ok(chip.text == "PINNED" and not _following(w), "%s P: a click on the chip pins the window ('%s')" % [key, chip.text])
	var held := _shows(key)
	var cands := _clickable(w)
	if key == "roster":
		cands = cands.filter(func(i): return int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) != held \
			and int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) > 0)
	elif key != "conflicts":
		cands = cands.filter(func(i): return i != held)
	if cands.is_empty():
		_ok(false, "%s P: a clickable place to move the selection to" % key)
		return
	await _click_at(_screen_of(int(cands[0])))
	_ok(_app.selection == {"kind": "settlement", "id": int(cands[0])}, "%s P: the map click moved the selection (to %s, clicked %d at %s)" % [key, str(_app.selection), int(cands[0]), str(_screen_of(int(cands[0])))])
	_ok(_shows(key) == held, "%s P: pinned, the window keeps its subject (%d, selection asks %d)" % [key, _shows(key), _wants(key)])
	if not await _press_chip(w, "%s P: the PINNED chip" % key):
		return
	_ok(chip.text == "FOLLOW", "%s P: a second click follows again" % key)
	_ok(_shows(key) == _wants(key), "%s P: following again, it shows the selection (%d, wants %d)" % [key, _shows(key), _wants(key)])
	## A reopen follows: pin, close, reopen.
	await _press_chip(w, "%s P: pin before the reopen" % key)
	w.hide()
	await _settle(4)
	var w2: AcceptDialog = await _open(key, int(_app.selection.get("id", 0)))
	if w2 != null:
		_ok(_chip(w2).text == "FOLLOW", "%s P: a reopened window follows again" % key)


# -- K: keys -----------------------------------------------------------------------------------------

func _roster_name_field(w: Window) -> LineEdit:
	for n in w.find_children("*", "LineEdit", true, false):
		if (n as Control).is_visible_in_tree() and (n as LineEdit).editable:
			return n
	return null


func _leg_keys() -> void:
	print("== K. keys typed in a window ==")
	_app.viewport.reset_view()
	await _settle(8)
	var w: AcceptDialog = await _open("roster", -1)
	if w == null:
		return
	var le := _roster_name_field(w)
	if not await _press_in(w, le, "K: the faction name field"):
		return
	_ok(le.has_focus(), "K: the field has the focus")
	var text0 := le.text
	var sel0: Dictionary = _app.selection.duplicate()
	var tool0: String = _app.armed_tool
	await _key(KEY_Q, 113)
	await _key(KEY_A, 65, true)
	_ok(le.text == text0 + "qA", "K: Q and Shift+A typed into the field ('%s')" % le.text)
	_ok(not _app.context_broker.ring_is_open(), "K: Q in the field opened no cartwheel")
	_ok(_app.armed_tool == tool0, "K: Shift+A in the field armed nothing (%s)" % _app.armed_tool)
	await _key(KEY_BACKSPACE)
	await _key(KEY_DELETE)
	_ok(le.text == text0 + "q", "K: Backspace edited the field ('%s')" % le.text)
	_ok(_app.selection == sel0, "K: Delete and Backspace in the field changed no selection")
	## Undo the typing: put the name back (Escape would close; select all + retype through the field).
	le.text = text0
	le.release_focus()
	await _settle(4)
	_ok(w.has_focus() and w.gui_get_focus_owner() == null, "K: no field focused, the window still focused")
	await _key(KEY_Q, 113)
	_ok(not _app.context_broker.ring_is_open(), "K: Q with the window focused and no field opens no cartwheel (the relay passes no plain key)")
	await _key(KEY_F, 0, false, true)
	var sf: LineEdit = _app._desktop_search_field
	_ok(sf != null and sf.is_visible_in_tree(), "K: Ctrl+F with the window focused opens Find")
	await _key(KEY_K, 107)
	_ok(sf != null and sf.text == "k", "K: the next letter typed lands in Find's field ('%s')" % (sf.text if sf else ""))
	await _key(KEY_ESCAPE)
	await _settle(4)
	w.hide()
	await _settle(4)


# -- H: held changes ---------------------------------------------------------------------------------

func _leg_held() -> void:
	print("== H. held changes are pinned, never discarded ==")
	_app.viewport.reset_view()
	await _settle(8)
	var w: AcceptDialog = await _open("roster", -1)
	if w == null:
		return
	## Setup: whole, against the screen's right edge, so the whole roster (its inspector
	## column, where the religion library is) is on screen and the map's left part stays clear.
	w.position = Vector2i(int(get_viewport().get_visible_rect().size.x) - w.size.x - 4, 90)
	await _settle(6)
	_app.select("faction", int(w._selected))
	await _settle(6)
	var fid := int(w._selected)
	var nr: Button = w.find_child("NewReligion", true, false)
	if nr != null:
		var sc := nr.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(nr)
			await _settle(4)
	if not await _press_in(w, nr, "H: New religion..."):
		return
	var subs: Array = []
	for sw in get_viewport().get_embedded_subwindows():
		subs.append("%s(%s)" % [(sw as Window).name, (sw as Window).title])
	print("  info after New religion: window focused %s; embedded subwindows %s" % [str(w.has_focus()), str(subs)])
	var rname: LineEdit = w.find_child("ReligionName", true, false)
	if rname == null:
		for n in w.find_children("*", "LineEdit", true, false):
			if (n as Control).is_visible_in_tree() and (n as LineEdit).placeholder_text != "":
				rname = n
				break
	if not await _press_in(w, rname, "H: the religion name field"):
		return
	print("  info religion name field focused %s, window focused %s; field rect %s window %s %s; hovered %s" % [str(rname.has_focus()), str(w.has_focus()), str(rname.get_global_rect()), str(w.position), str(w.size), str(w.gui_get_hovered_control())])
	for ch in "Zor":
		await _key(OS.find_keycode_from_string(ch.to_upper()) as Key, ch.unicode_at(0))
	var again: LineEdit = w.find_child("ReligionName", true, false)
	print("  info after typing: held ref valid %s text '%s'; live field %s text '%s'" % [str(is_instance_valid(rname)), rname.text if is_instance_valid(rname) else "", str(again), again.text if again else ""])
	_ok(rname.text.ends_with("Zor"), "H: a name was typed into the editor ('%s')" % rname.text)
	var cands := _clickable(w).filter(func(i): return int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) != fid \
		and int((_bridge.settlements()[i] as Dictionary).get("faction", 0)) > 0)
	if cands.is_empty():
		_ok(false, "H: a place of another faction to click")
		return
	await _click_at(_screen_of(int(cands[0])))
	_ok(String(_app.selection.get("kind", "")) == "settlement", "H: the map click selected a place of another faction")
	_ok(not _following(w) and _chip(w).text == "PINNED", "H: the roster pinned itself instead of following ('%s')" % _chip(w).text)
	_ok(int(w._selected) == fid, "H: it kept its faction (%d)" % int(w._selected))
	_ok(is_instance_valid(rname) and rname.is_inside_tree() and rname.text.ends_with("Zor"), "H: the editor and the typed name were kept")
	_ok(_chip(w).tooltip_text.contains("religion editor"), "H: the chip says why it pinned ('%s')" % _chip(w).tooltip_text)
	## Follow asks; Cancel keeps.
	await _press_chip(w, "H: the PINNED chip")
	var dlg := _confirm()
	_ok(dlg != null, "H: following asks first")
	if dlg != null:
		await _click_at(dlg.get_cancel_button().get_global_rect().get_center() + Vector2(dlg.position))
		await _settle(6)
		_ok(not _following(w) and int(w._selected) == fid and is_instance_valid(rname) and rname.text.ends_with("Zor"),
			"H: Cancel keeps the pin, the faction and the typed name")
	await _press_chip(w, "H: the PINNED chip again")
	dlg = _confirm()
	if dlg != null:
		await _click_at(dlg.get_ok_button().get_global_rect().get_center() + Vector2(dlg.position))
		await _settle(8)
		_ok(_following(w) and int(w._selected) == int(_app.current_faction()) and int(w._religion_edit_id) == -1,
			"H: Discard and follow closes the editor and shows the selection's faction (%d, wants %d)" % [int(w._selected), int(_app.current_faction())])
	w.hide()
	await _settle(4)
	## The Place editor's event draft (set as setup: its form needs a live vault note).
	var start := _clickable(null)
	_app.select("settlement", int(start[0]))
	await _settle(4)
	var pe: AcceptDialog = await _open("place", int(start[0]))
	if pe == null:
		return
	pe._ae_name = "Siege of probe"
	var others := _clickable(pe).filter(func(i): return i != int(start[0]))
	if not others.is_empty():
		await _click_at(_screen_of(int(others[0])))
		_ok(not _following(pe) and int(pe._index) == int(start[0]) and pe._ae_name == "Siege of probe",
			"H: an unsaved event pins the Place editor on its place and keeps the draft (index %d)" % int(pe._index))
	## A year move while the name field is being typed in keeps the typed text.
	await _press_chip(pe, "H: follow the Place editor")
	var dlg2 := _confirm()
	if dlg2 != null:
		await _click_at(dlg2.get_ok_button().get_global_rect().get_center() + Vector2(dlg2.position))
		await _settle(6)
	var nm: LineEdit = pe._name_edit
	if await _press_in(pe, nm, "H: the place name field"):
		var t0 := nm.text
		await _key(KEY_X, 120)
		var to := 255 if _app.tl_year() != 255 else 100
		await _year_by_pill(to)
		await _settle(6)
		_ok(is_instance_valid(nm) and nm.is_inside_tree() and nm.text == t0 + "x",
			"H: a year move while the name field is being typed in keeps the typed text ('%s')" % (nm.text if is_instance_valid(nm) else "freed"))
		## Put the name back before the field commits.
		if is_instance_valid(nm):
			nm.text = t0
	pe.hide()
	await _settle(6)


## The ConfirmationDialog `ContextWindow.follow()` opened, or null.
func _confirm() -> ConfirmationDialog:
	for n in _app.find_children("*", "ConfirmationDialog", true, false):
		if (n as Window).visible and (n as Window).title == "Follow the selection?":
			return n
	return null


# -- O: Window ▸ Open windows ------------------------------------------------------------------------

func _leg_open_windows() -> void:
	print("== O. Window > Open windows ==")
	var w: AcceptDialog = await _open("roster", -1)
	if w == null:
		return
	w.position = w.position + Vector2i(-120, 40)
	await _settle(4)
	var moved_to := w.position
	var menus = _app.menus
	menus._refresh_open_windows()
	var idx := -1
	for i in menus._open_windows.size():
		if typeof(menus._open_windows[i]) == TYPE_OBJECT and menus._open_windows[i] == w:
			idx = i
	_ok(idx >= 0, "O: Open windows lists the open roster")
	if idx >= 0:
		menus._on_open_window(idx)
		await _settle(4)
		_ok(w.position == moved_to, "O: picking it raises it where the user put it (%s, was %s)" % [str(w.position), str(moved_to)])
	w.hide()
	await _settle(4)


# -- B: busy -----------------------------------------------------------------------------------------

func _leg_busy() -> void:
	print("== B. a generate mutes the open window ==")
	_app.viewport.reset_view()
	await _settle(8)
	var w: AcceptDialog = await _open("roster", -1)
	if w == null:
		return
	## Screen points taken BEFORE the generate: the settlement list is the worker's while it runs.
	var pts: Array = _clickable(w).map(func(i): return _screen_of(int(i)))
	var sel0: Dictionary = _app.selection.duplicate()
	## Focused BEFORE the generate (a muted window cannot be focused by a click), so the relay is the
	## only route a wheel over the map could take to the camera; with no window focused the wheel
	## reaches the map directly, a generate or not (measured: with no window focused a wheel zooms during a generate; `viewport_host.gd::_input` runs before the overlay).
	await _focus_inside(w)
	_ok(w.has_focus(), "B: the roster has the focus as the generate starts")
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _settle(6)
	if not bool(_bridge.generating):
		print("  info the generate finished before the busy checks could run")
		_ok(false, "B: the generate was still running for the checks")
		return
	_ok(w.gui_disable_input, "B: the busy overlay muted the open roster")
	var chip := _chip(w)
	var t0 := chip.text
	await _press_chip(w, "B: the chip, during the generate")
	_ok(chip.text == t0 and _following(w), "B: a click on the chip during the generate changes nothing ('%s')" % chip.text)
	if bool(_bridge.generating):
		## The relay must stay inert while the window is muted: a wheel over the map, the window focused.
		var cam = _app.viewport._camera
		var z0: Vector2 = cam.scale
		var focused_win := w.has_focus()
		if not pts.is_empty():
			await _wheel(pts[0])
		## Not a wheel step: the scale may still move if the generate ends inside this check (the new
		## world is fitted), which is not the relay's doing.
		var step: float = _app.viewport.ZOOM_WHEEL_STEP
		_ok(focused_win and absf(cam.scale.x / z0.x - step) > 0.01,
			"B: a wheel over the map from the muted, focused window is not relayed (focused %s, %.4f -> %.4f, a wheel step is x%.2f)" % [str(focused_win), z0.x, cam.scale.x, step])
	if not pts.is_empty() and bool(_bridge.generating):
		await _click_at(pts[0])
		_ok(_app.selection == sel0 or _app.selection.is_empty(), "B: a map click during the generate selects nothing (%s)" % str(_app.selection))
	var t1 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t1 < 240000:
		await _settle(10)
	await _settle(30)
	_ok(not w.gui_disable_input, "B: after the generate the roster takes input again")
	w.hide()
	await _settle(4)
