extends Node
## PC-7's leftover probe (`PC_SHELL_PROPOSAL.md` §3.6 and §6's PC-7 and PC-5 notes): one host with a back stack for
## the five context windows, the refresh items of finding W7, Find above an open window, and private notes shown as
## private in the Note tab. Inside the real shell, windowed, with real mouse clicks and key events through
## `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc7nav_probe.tscn
##   tablet:  godot --path . --resolution 1600x1000 _pc7nav_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, with a wiped user dir (the runner sets `APPDATA` to an empty folder). Reads
## `--force-touch`, `--shots DIR` (saves PNGs) and `--mutant NAME` (prints a marker only; the harness reads the exit
## code); anything else exits 2. Exit 0 pass, 1 fail, 2 bad argument or watchdog, 3 a leg could not run.
##
## What it protects, desktop:
##   N. TRAIL. The shell's own open calls open Place editor (A), City viewer (A), then, after a real map click on B,
##      Place editor (B): the new window takes the previous one's place and the previous one is hidden, not closed;
##      the Back / Forward row shows ("n / m"), a real click on BACK shows the City viewer on B and then the Place
##      editor on A, FORWARD walks back; a pin survives the walk; a new open after a Back cuts Forward off; the
##      same window re-opened on another subject is a visit and on the same subject is not; the Close button and a
##      world change end the trail. A rename in the Place editor shows in the City viewer reached by Back.
##   I. IDENTITY. A window opened alone has no row drawn and lays out as it did (`ContextNavPad` hidden).
##   F. FIND. With the roster open and focused over Find's place, Ctrl+F draws Find above it (the pixels where the two
##      overlap are Find's, not the roster's) and a picked result selects and the following roster follows it.
##   W. REFRESH. A roster rename reaches the Context panel's faction page; an assistant save re-reads the open note
##      in the Note tab (clean) and leaves unsaved typing alone (dirty).
##   P. PRIVATE. A note with `private: true` shows the private line in the Note tab, a public note does not, and
##      typing the marker into a public note shows it live after the save (and removing it hides it).
## Tablet (`--force-touch`): opening two of the five registers no trail and draws no row (§3.11: the modal forms).

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



const NAV_PAD := "ContextNavPad"
const NAV_BACK := "ContextNavBack"
const NAV_FWD := "ContextNavForward"
const NAV_POS := "ContextNavPos"
const CHIP := "FollowChip"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _shots := ""
var _root := ""
## `--leg trail|find|refresh|private` runs one leg (a quick re-run while building); empty runs all four.
var _only := ""


func _wait_until(cond: Callable, ms: int) -> bool:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _shot(name: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	get_viewport().get_texture().get_image().save_png("%s/pc7nav_%s.png" % [_shots, name])


func _nav():
	return _app.context_nav


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


## Opens window `key` through the shell's own open call (what the menus, the card and the dock call).
func _open(key: String, idx := 0) -> AcceptDialog:
	match key:
		"roster":
			_app.open_faction_roster(idx if idx > 0 else -1)
		"place":
			_app.open_place_editor(idx)
		"city":
			_app.open_city_viewer(idx)
		"conflicts":
			_app.open_conflicts_list()
		"journeys":
			_app.open_saved_journeys()
	await _settle(10)
	return _win(key)


func _nav_btn(w: Window, nm: String) -> Button:
	return w.find_child(nm, true, false) as Button


func _pad(w: Window) -> Control:
	return w.find_child(NAV_PAD, true, false) as Control


func _pos_text(w: Window) -> String:
	var l := w.find_child(NAV_POS, true, false) as Label
	return l.text if l != null else "<none>"


func _bar_shown(w: Window) -> bool:
	var p := _pad(w)
	return p != null and p.is_visible_in_tree()


func _visible_hosted() -> Array:
	var out: Array = []
	for k in ["roster", "place", "city", "conflicts", "journeys"]:
		var w := _win(k)
		if w != null and w.visible:
			out.append(k)
	return out


func _hide_all() -> void:
	for k in ["roster", "place", "city", "conflicts", "journeys"]:
		var w := _win(k)
		if w != null:
			w.hide()
	await _settle(6)
	if _nav() != null:
		_nav().clear()


func _char(ch: String) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = ch.to_upper().unicode_at(0) as Key
		ev.physical_keycode = ev.keycode
		ev.pressed = down
		ev.unicode = ch.unicode_at(0) if down else 0
		Input.parse_input_event(ev)
		await _settle(2)


func _type_text(text: String) -> void:
	for i in text.length():
		var ch := text.substr(i, 1)
		if ch == "\n":
			await _key(KEY_ENTER)
		else:
			await _char(ch)


## Real-clicks the places `_clickable(w)` offers (clear of window `w`, not `not_idx`) until
## the selection becomes one of them; returns that settlement's index, or -1 when none took.
func _dismiss_card() -> void:
	var cb = _app.get("context_broker")
	if cb != null and cb.get("card") != null and is_instance_valid(cb.card) and cb.card.visible:
		cb.card.hide()
	await _settle(2)


func _select_by_click(w: Window, not_idx: int) -> int:
	var tried: Array = []
	for _attempt in 6:
		var cands := _clickable(w).filter(func(i): return i != not_idx and not tried.has(i))
		if cands.is_empty():
			return -1
		var i := int(cands[0])
		tried.append(i)
		await _dismiss_card()
		await _click_at(_screen_of(i))
		await _settle(8)
		await _dismiss_card()
		if _app.selection == {"kind": "settlement", "id": i}:
			return i
	return -1


func _following(w: Object) -> bool:
	return not bool(w.get_meta("_ctxwin_pinned", false))


## A real click on `c`, scrolled into its scroller's view first.
func _click_ctl(w: Window, c: Control) -> void:
	var sc: Node = c.get_parent()
	while sc != null and not (sc is ScrollContainer) and sc != w:
		sc = sc.get_parent()
	if sc is ScrollContainer:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(4)
	await _click_at(_in_win(w, c))


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--leg" and k + 1 < args.size():
			k += 1
			_only = args[k]
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
			DirAccess.make_dir_recursive_absolute(_shots)
		elif a == "--mutant" and k + 1 < args.size():
			k += 1
			print("MUTANT %s" % args[k])
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	if DisplayServer.get_name() == "headless":
		print("REFUSED: run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(800.0).timeout.connect(func() -> void:
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
		print("[FATAL] this probe has no phone leg (the phone is guarded by captures and the existing phone probes)")
		get_tree().quit(3)
		return
	_bridge = _app.bridge
	await _generate()
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 4 or _bridge.get_factions().size() < 2:
		print("[FATAL] no world with four settlements and two factions")
		get_tree().quit(2)
		return
	_ok(_app.get("context_nav") != null, "the shell holds a ContextNav")
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


# -- tablet -----------------------------------------------------------------------------------------

func _leg_tablet() -> void:
	print("== T. tablet: no trail, no row ==")
	var idx: int = _clickable(null)[0]
	_app.select("settlement", idx)
	var w1 := await _open("place", idx)
	var w2 := await _open("city", idx)
	_ok(w1 != null and w2 != null and w2.visible, "the tablet opens the City viewer over the Place editor")
	_ok(w1.visible and w1.exclusive and w2.exclusive, "both stay exclusive (the modal forms)")
	_ok(_nav().size() == 0, "no trail is recorded (size %d)" % _nav().size())
	_ok(_pad(w1) == null and _pad(w2) == null, "no nav row is built on either")
	_ok(w1.find_child("ContextNavWrap", true, false) == null, "the content is not wrapped")
	w2.hide()
	w1.hide()
	await _settle(6)


# -- desktop ----------------------------------------------------------------------------------------

func _leg_desktop() -> void:
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(10)
	_app.viewport.reset_view()
	await _settle(8)
	if _only == "" or _only.contains("trail"):
		await _leg_trail()
	if _only == "" or _only.contains("keys"):
		await _leg_keys()
	if _only == "" or _only.contains("find"):
		await _leg_find()
	if _only == "" or _only.contains("refresh"):
		await _leg_refresh()
	if _only == "" or _only.contains("private"):
		await _leg_private()


func _leg_trail() -> void:
	print("== N. the trail ==")
	var cl: Array = _clickable(null)
	if cl.size() < 3:
		print("[FATAL] fewer than three clickable settlements")
		get_tree().quit(3)
		return
	var ia: int = int(cl[0])
	var all: Array = _bridge.settlements()
	var name_a := String((all[ia] as Dictionary).get("name", ""))
	await _click_at(_screen_of(ia))
	await _dismiss_card()
	_ok(_app.selection == {"kind": "settlement", "id": ia}, "a real click selects A (selection %s)" % str(_app.selection))
	var pe := await _open("place", ia)
	_ok(pe != null and pe.visible and not pe.exclusive, "Place editor opens on A, not exclusive")
	var mr := (_app.viewport as Control).get_global_rect()
	pe.position = Vector2i(int(mr.end.x) - 420, 120)
	await _settle(4)
	_ok(_nav().size() == 1, "I: a window opened alone is one visit (size %d)" % _nav().size())
	_ok(not _bar_shown(pe), "I: and draws no Back / Forward row")
	var pe_pos := pe.position
	## A second window: the City viewer takes the Place editor's place, the Place editor is hidden and kept.
	var cv := await _open("city", ia)
	_ok(cv != null and cv.visible and not pe.visible, "N1: the City viewer shows and the Place editor is hidden, not closed")
	_ok(_nav().size() == 2 and _nav().cursor() == 1, "N1: two visits, the cursor on the second (%d, %d)" % [_nav().size(), _nav().cursor()])
	var vs := get_tree().root.size
	var want_x := mini(pe_pos.x, vs.x - cv.size.x)
	_ok(cv.position.x == want_x and cv.position.y == pe_pos.y,
		"N1: the new window stands where the old one did, kept on screen (%s, wanted x %d y %d)" % [str(cv.position), want_x, pe_pos.y])
	_ok(_bar_shown(cv) and _pos_text(cv) == "2 / 2", "N1: the row shows '2 / 2' ('%s')" % _pos_text(cv))
	_ok(not _nav_btn(cv, NAV_BACK).disabled and _nav_btn(cv, NAV_FWD).disabled, "N1: Back enabled, Forward disabled")
	_ok(_nav_btn(cv, NAV_BACK).tooltip_text.contains("Place: " + name_a), "N1: Back's tooltip names where it goes ('%s')" % _nav_btn(cv, NAV_BACK).tooltip_text)
	await _shot("trail_city")
	## A second selection, by a real click clear of the window; the following viewer moves with it.
	## The viewer is dragged aside (its right part hangs off screen, as a user may leave it) so the map's left is clear.
	cv.position = Vector2i(vs.x - 300, 120)
	await _settle(4)
	var tgt := await _select_by_click(cv, ia)
	var name_b := String((all[tgt] as Dictionary).get("name", "")) if tgt >= 0 else ""
	_ok(tgt >= 0 and int(cv._index) == tgt, "N2: a real map click selects B and the following City viewer shows it (B %d, viewer %d)" % [tgt, int(cv._index)])
	if tgt < 0:
		return
	pe = await _open("place", tgt)
	_ok(pe.visible and not cv.visible and int(pe._index) == tgt, "N2: Place editor on B shows and the City viewer is hidden")
	_ok(_nav().size() == 3 and _pos_text(pe) == "3 / 3", "N2: three visits ('%s')" % _pos_text(pe))
	var trail: PackedStringArray = _nav().trail()
	_ok(trail.size() == 3 and trail[0].contains("Place: " + name_a) and trail[1].contains("City: " + name_b) and trail[2].contains("Place: " + name_b) and trail[2].begins_with(">"),
		"N2: the trail reads Place A, City B (it followed), Place B: %s" % str(trail))
	## A rename in the Place editor (real typing) shows in the City viewer reached by Back (W7: re-read).
	var new_name := "Zzqwx"
	await _click_ctl(pe, pe._name_edit)
	await _key(KEY_A, 0, false, true)
	await _type_text(new_name)
	await _key(KEY_ENTER)
	await _settle(8)
	var engine_name := String((_bridge.settlements()[tgt] as Dictionary).get("name", ""))
	_ok(engine_name == new_name, "N2: the rename reached the engine ('%s')" % engine_name)
	## Back, by real clicks.
	await _press_in(pe, _nav_btn(pe, NAV_BACK), "BACK on the Place editor")
	_ok(cv.visible and not pe.visible and _nav().cursor() == 1, "N3: BACK shows the City viewer and hides the Place editor (cursor %d)" % _nav().cursor())
	_ok(int(cv._index) == tgt, "N3: the City viewer is on B")
	_ok(cv._picker.get_item_text(cv._picker.selected).contains(new_name), "N3 W7: and it re-read the rename ('%s')" % cv._picker.get_item_text(cv._picker.selected))
	_ok(cv.position.y == pe.position.y, "N3: where the Place editor stood")
	_ok(_pos_text(cv) == "2 / 3", "N3: '2 / 3' ('%s')" % _pos_text(cv))
	await _press_in(cv, _nav_btn(cv, NAV_BACK), "BACK on the City viewer")
	_ok(pe.visible and not cv.visible and int(pe._index) == ia, "N4: BACK again shows the Place editor on A (index %d, wanted %d)" % [int(pe._index), ia])
	_ok(_nav_btn(pe, NAV_BACK).disabled and not _nav_btn(pe, NAV_FWD).disabled, "N4: at the start Back is disabled and Forward is not")
	_ok(_nav().size() == 3, "N4: Back adds nothing to the trail (size %d)" % _nav().size())
	await _shot("trail_back_to_place")
	await _press_in(pe, _nav_btn(pe, NAV_FWD), "FORWARD on the Place editor")
	_ok(cv.visible and not pe.visible and _nav().cursor() == 1, "N5: FORWARD shows the City viewer")
	await _press_in(cv, _nav_btn(cv, NAV_FWD), "FORWARD on the City viewer")
	_ok(pe.visible and int(pe._index) == tgt and _nav().cursor() == 2, "N5: FORWARD again shows the Place editor on B")
	_ok(_nav_btn(pe, NAV_FWD).disabled, "N5: at the end Forward is disabled")
	## A pin survives the walk.
	await _press_chip(pe, "pin the Place editor")
	_ok(not _following(pe), "N6: the Place editor is PINNED")
	await _press_in(pe, _nav_btn(pe, NAV_BACK), "BACK from the pinned editor")
	await _press_in(cv, _nav_btn(cv, NAV_FWD), "FORWARD to the pinned editor")
	_ok(pe.visible and not _following(pe), "N6: the pin was kept through Back and Forward")
	await _press_chip(pe, "unpin the Place editor")
	_ok(_following(pe), "N6: and it follows again")
	## A new open after a Back cuts Forward off.
	await _press_in(pe, _nav_btn(pe, NAV_BACK), "BACK before a new open")
	var cf := await _open("conflicts")
	_ok(cf.visible and not cv.visible and _nav().size() == 3 and _nav_btn(cf, NAV_FWD).disabled,
		"N7: opening Conflicts from the City viewer cuts Forward off (size %d)" % _nav().size())
	## The same window on another subject is a visit; on the same subject it is not.
	var third := int(cl[2])
	pe = await _open("place", ia)
	var n0: int = _nav().size()
	pe = await _open("place", third)
	_ok(_nav().size() == n0 + 1, "N8: the Place editor re-opened on another place is a new visit (%d -> %d)" % [n0, _nav().size()])
	var n1: int = _nav().size()
	pe = await _open("place", third)
	_ok(_nav().size() == n1, "N8: and on the same place it is not (size %d)" % _nav().size())
	await _press_in(pe, _nav_btn(pe, NAV_BACK), "BACK within one window")
	_ok(int(pe._index) == ia and pe.visible, "N8: Back within one window shows the place it showed before (index %d, wanted %d)" % [int(pe._index), ia])
	## The cap.
	for i in 30:
		_app.open_place_editor(ia if i % 2 == 0 else third)
		await _settle(2)
	_ok(_nav().size() <= 24, "N9: the trail is capped at 24 (size %d)" % _nav().size())
	## Close ends it.
	var top := _nav().current_window() as AcceptDialog
	await _click_at(_in_win(top, top.get_ok_button()))
	await _settle(8)
	_ok(not top.visible and _visible_hosted().is_empty() and _nav().size() == 0,
		"N10: Close ends the trail and the stacked windows stay hidden (visible: %s, size %d)" % [str(_visible_hosted()), _nav().size()])
	## A world change ends it.
	await _open("place", ia)
	await _open("city", ia)
	_ok(_nav().size() == 2, "N11: set up, two visits")
	_bridge.world_loaded.emit()
	await _settle(4)
	_ok(_nav().size() == 0 and not _bar_shown(_win("city")), "N11: a world change ends the trail and the row hides (size %d)" % _nav().size())
	await _hide_all()
	## I. identity: a lone window carries a hidden row and the content in a wrapper.
	var lone := await _open("roster", -1)
	_ok(lone.visible and _nav().size() == 1 and not _bar_shown(lone), "I: a lone roster has one visit and no row")
	_ok(lone.find_child("ContextNavWrap", true, false) != null, "I: its content sits in the wrapper the row hides in")
	await _hide_all()


func _press_chip(w: Window, what: String) -> void:
	var chip := w.find_child(CHIP, true, false) as Button
	if chip == null:
		_ok(false, "%s: the chip exists" % what)
		return
	var vs := get_viewport().get_visible_rect().size
	var at := _in_win(w, chip)
	var moved := Vector2i.ZERO
	if at.x > vs.x - 8.0:
		moved = Vector2i(int(at.x - vs.x) + 60, 0)
		w.position -= moved
		await _settle(4)
	await _press_in(w, chip, what)
	if moved != Vector2i.ZERO:
		w.position += moved
		await _settle(4)


# -- Find ---------------------------------------------------------------------------------------------

func _patch(r: Rect2i) -> PackedByteArray:
	return get_viewport().get_texture().get_image().get_region(r).get_data()


## Row texts of the Window ▸ Open windows submenu, after the menu itself rebuilt it (its about_to_popup).
func _open_windows_rows() -> Array:
	var pm: PopupMenu = _app.menus._windows_popup
	pm.about_to_popup.emit()
	var out: Array = []
	for i in pm.item_count:
		out.append(pm.get_item_text(i))
	return out


## Edit-menu keys for the trail and Window ▸ Open windows listing the windows behind it (2026-11-15).
##   K1  Ctrl+[ (a real key event) shows the earlier window and hides the later one; Ctrl+] walks back.
##   K2  with nothing earlier, Ctrl+[ changes nothing.
##   K3  the Open windows submenu lists BOTH windows of the trail, the hidden one marked "in trail".
##   K4  picking the hidden one's row brings it forward and hides the other, with its state.
func _leg_keys() -> void:
	print("== K. keys and Open windows ==")
	await _hide_all()
	_nav().clear()
	var cl: Array = _clickable(null)
	var ia: int = int(cl[0])
	var pe := await _open("place", ia)
	var cv := await _open("city", ia)
	_ok(cv.visible and not pe.visible and _nav().size() == 2, "K0: a two-window trail, the City viewer on screen")
	await _key(KEY_BRACKETLEFT, 0, false, true)
	_ok(pe.visible and not cv.visible and _nav().cursor() == 0, "K1: Ctrl+[ shows the Place editor and hides the City viewer")
	await _key(KEY_BRACKETLEFT, 0, false, true)
	_ok(pe.visible and _nav().cursor() == 0, "K2: Ctrl+[ at the start changes nothing")
	await _key(KEY_BRACKETRIGHT, 0, false, true)
	_ok(cv.visible and not pe.visible and _nav().cursor() == 1, "K1: Ctrl+] walks forward to the City viewer")
	var rows := _open_windows_rows()
	rows.erase("Context panel")   ## the panel's own row (PC-4) is listed first while it is shown; not a trail window
	var trail_rows := rows.filter(func(t): return String(t).ends_with("in trail"))
	_ok(rows.size() == 2 and trail_rows.size() == 1, "K3: the list holds the window on screen and the one behind it (%s)" % str(rows))
	_ok(trail_rows.size() == 1 and String(trail_rows[0]).contains("Place"), "K3: the hidden row is the Place editor (%s)" % str(trail_rows))
	var pm: PopupMenu = _app.menus._windows_popup
	var idx := -1
	for i in pm.item_count:
		if pm.get_item_text(i).ends_with("in trail"):
			idx = i
	if idx >= 0:
		pm.id_pressed.emit(pm.get_item_id(idx))
		await _settle(10)
	_ok(pe.visible and not cv.visible and _nav().cursor() == 0, "K4: picking the row brings the Place editor forward and hides the other")
	var rows2 := _open_windows_rows()
	rows2.erase("Context panel")
	_ok(rows2.size() == 2 and rows2.filter(func(t): return String(t).ends_with("in trail")).size() == 1,
		"K4: and the list now marks the City viewer as the one in the trail (%s)" % str(rows2))
	await _hide_all()


func _leg_find() -> void:
	print("== F. Find above an open window ==")
	await _hide_all()
	var cl: Array = _clickable(null)
	var ro := await _open("roster", -1)
	var vs := get_tree().root.size
	## Find is centred 440 x 480; the roster is put over the middle of it.
	var fr := Rect2i(Vector2i((vs - Vector2i(440, 480)) / 2), Vector2i(440, 480))
	ro.position = Vector2i(fr.position.x - 100, fr.position.y - 40)
	await _settle(6)
	await _click_at(Vector2(ro.position) + Vector2(6, 6))
	## Two patches inside Find's rect and inside the roster's, away from Find's caret.
	var p1 := Rect2i(fr.position.x + 60, fr.position.y + 330, 40, 12)
	var p2 := Rect2i(fr.position.x + 300, fr.position.y + 200, 40, 12)
	var ro_rect := Rect2i(ro.position, ro.size)
	_ok(ro_rect.encloses(p1) and ro_rect.encloses(p2), "F: both patches are inside the roster's rect (%s vs %s)" % [str(ro_rect), str(p1)])
	await _settle(4)
	var roster_only: Array = [_patch(p1), _patch(p2)]
	await _key(KEY_F, 0, false, true)
	await _settle(10)
	var fd = _app._desktop_search_dialog
	_ok(fd != null and fd.visible, "F: Ctrl+F with the roster focused opens Find")
	if fd == null or not fd.visible:
		return
	_ok(Rect2i(fd.position, fd.size).encloses(p1), "F: Find's rect covers the patches (%s)" % str(Rect2i(fd.position, fd.size)))
	await _shot("find_over_roster")
	var both: Array = [_patch(p1), _patch(p2)]
	ro.hide()
	await _settle(8)
	var find_only: Array = [_patch(p1), _patch(p2)]
	_ok(roster_only[0] != find_only[0] and roster_only[1] != find_only[1], "F: CONTROL: the roster alone and Find alone differ in both patches")
	_ok(both[0] == find_only[0] and both[1] == find_only[1], "F: with both open the overlap is Find's pixels: Find is above the roster")
	## A picked result selects and the following roster follows it.
	fd.hide()
	await _settle(6)
	ro = await _open("roster", -1)
	var s_idx := int(cl[0])
	var s: Dictionary = _bridge.settlements()[s_idx]
	var holder := int(s.get("faction", 0))
	await _click_at(Vector2(ro.position) + Vector2(6, 6))
	await _key(KEY_F, 0, false, true)
	await _settle(6)
	_app._desktop_search_field.text = String(s.get("name", ""))
	_app._desktop_search_field.text_changed.emit(String(s.get("name", "")))
	await _settle(8)
	var row: Button = null
	for b in _app._desktop_search_results.find_children("*", "Button", true, false):
		var lbls := (b as Button).find_children("*", "Label", true, false)
		var hit := false
		for l in lbls:
			if String((l as Label).text).to_upper() == String(s.get("name", "")).to_upper():
				hit = true
		if (b as Button).is_visible_in_tree() and hit:
			row = b
			break
	_ok(row != null, "F: Find lists the place by name")
	if row != null:
		await _click_at(_in_win(_app._desktop_search_dialog, row))
		await _settle(8)
		_ok(not _app._desktop_search_dialog.visible, "F: picking a result closes Find")
		_ok(_app.selection == {"kind": "settlement", "id": s_idx}, "F: and selects the place (%s)" % str(_app.selection))
		_ok(ro.visible and int(ro._selected) == _app.current_faction(), "F: the following roster shows the selection's faction (%d, %d; holder %d)" % [int(ro._selected), _app.current_faction(), holder])
	await _hide_all()


# -- refresh (finding W7) -----------------------------------------------------------------------------

func _write_file(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _find_label(n: Node, needle: String) -> Label:
	if n is Label and (n as Label).is_visible_in_tree() and (n as Label).text.contains(needle):
		return n
	for c in n.get_children():
		var r := _find_label(c, needle)
		if r != null:
			return r
	return null


func _find_named(n: Node, nm: String) -> Node:
	if n.name == nm and (n is CanvasItem and (n as CanvasItem).is_visible_in_tree()):
		return n
	for c in n.get_children():
		var r := _find_named(c, nm)
		if r != null:
			return r
	return null


func _count_editors() -> int:
	var n := 0
	var stack: Array = [_app]
	while not stack.is_empty():
		var c: Node = stack.pop_back()
		var s: Script = c.get_script()
		if s != null and s.resource_path.ends_with("markdown_editor.gd") and not c.is_queued_for_deletion():
			n += 1
		for ch in c.get_children():
			stack.append(ch)
	return n


func _leg_refresh() -> void:
	print("== W. refresh ==")
	await _hide_all()
	## W1: a roster rename reaches the Context panel's faction page.
	var facs: Array = _bridge.get_factions()
	var fid := int((facs[0] as Dictionary).get("id", 1))
	var old_name := String((facs[0] as Dictionary).get("name", ""))
	_app.open_context_tab("inspect")
	_app.select("faction", fid)
	await _settle(8)
	_ok(_find_label(_app.right_dock_body, old_name) != null, "W1: the Context panel's faction page shows '%s'" % old_name)
	var ro := await _open("roster", fid)
	var field: LineEdit = null
	for le in ro._inspector_body.find_children("*", "LineEdit", true, false):
		if (le as LineEdit).text == old_name and (le as LineEdit).editable:
			field = le
			break
	_ok(field != null, "W1: the roster's identity page has the name field")
	if field != null:
		await _click_ctl(ro, field)
		await _key(KEY_A, 0, false, true)
		await _type_text("Qvexa")
		await _key(KEY_ENTER)
		await _settle(12)
		var now := String((_bridge.get_factions()[0] as Dictionary).get("name", ""))
		_ok(now == "Qvexa", "W1: the rename reached the engine ('%s')" % now)
		_ok(_find_label(_app.right_dock_body, "Qvexa") != null and _find_label(_app.right_dock_body, old_name) == null,
			"W1: the Context panel's faction page re-read it (new name shown, old gone)")
	await _hide_all()
	## W4: an assistant save re-reads the open note.
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-pc7nav-%d" % Time.get_ticks_msec()
	var ss: Array = _bridge.settlements()
	var cl: Array = _clickable(null)
	var ia: int = int(cl[0])
	var tid := int((ss[ia] as Dictionary).get("tid", 0))
	var nm := String((ss[ia] as Dictionary).get("name", ""))
	_write_file(_root + "/QnoteA.md", "# QnoteA\n\nAlpha text.\n")
	_write_file(_root + "/QnoteP.md", "---\nprivate: true\n---\n# QnoteP\n\nSecret text.\n")
	_ok(bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)), "W: the fixture vault binds")
	_ok(bool(_bridge.vault_attach("settlement", tid, nm, "QnoteA.md", "").get("ok", false)), "W: A's note attached")
	var tid_p := int((ss[int(cl[1])] as Dictionary).get("tid", 0))
	var nm_p := String((ss[int(cl[1])] as Dictionary).get("name", ""))
	_ok(bool(_bridge.vault_attach("settlement", tid_p, nm_p, "QnoteP.md", "").get("ok", false)), "W: B's private note attached")
	var vw = _app.vault_window
	_app.select("settlement", ia)
	_app.open_context_tab("note")
	await _settle(10)
	_ok(await _wait_until(func(): return vw.is_docked() and vw.docked_path() == "QnoteA.md", 3000), "W4: the Note tab holds QnoteA.md")
	var ed_text := func() -> String: return vw._editor.text_edit.text if vw._editor != null and is_instance_valid(vw._editor) else ""
	_ok(String(ed_text.call()).contains("Alpha text."), "W4: with the old text")
	var rd: Dictionary = _bridge.vault_read_file_for_edit("QnoteA.md")
	var w: Dictionary = _bridge.vault_write_file("QnoteA.md", "# QnoteA\n\nAlpha text.\n\nAssistant paragraph.\n", String(rd["hash"]))
	_ok(bool(w.get("ok", false)), "W4: the assistant's write lands (%s)" % str(w.get("error", "")))
	_app.assistant_window.note_saved.emit("QnoteA.md")
	await _settle(10)
	_ok(String(ed_text.call()).contains("Assistant paragraph."), "W4: a clean open note re-reads it ('%s')" % String(ed_text.call()).replace("\n", "|"))
	_ok(vw.is_docked() and _count_editors() == 1, "W4: still one editor, docked")
	## The same in the vault window itself.
	vw.open_note("QnoteA.md")
	await _settle(12)
	_ok(vw.visible and not vw.is_docked() and vw._editor != null, "W4b: the vault window holds the note's editor")
	var rd1: Dictionary = _bridge.vault_read_file_for_edit("QnoteA.md")
	var w1: Dictionary = _bridge.vault_write_file("QnoteA.md", "# QnoteA

Alpha text.

Assistant paragraph.

Second assistant paragraph.
", String(rd1["hash"]))
	_ok(bool(w1.get("ok", false)), "W4b: the second assistant write lands")
	_app.assistant_window.note_saved.emit("QnoteA.md")
	await _settle(10)
	_ok(vw._editor != null and is_instance_valid(vw._editor) and vw._editor.text_edit.text.contains("Second assistant paragraph."), "W4b: the window's editor re-read it")
	vw.hide()
	await _settle(10)
	_ok(await _wait_until(func(): return vw.is_docked() and vw.docked_path() == "QnoteA.md", 3000), "W4b: closing the window gives the editor back to the tab")
	## Dirty: typed text is never replaced.
	var te: TextEdit = vw._editor.text_edit
	await _click_at(te.get_global_rect().get_center())
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type_text("zzy")
	_ok(String(ed_text.call()).contains("zzy"), "W4: typed text is in the editor")
	var rd2: Dictionary = _bridge.vault_read_file_for_edit("QnoteA.md")
	var disk_text := "# QnoteA\n\nOutside words.\n"
	var w2: Dictionary = _bridge.vault_write_file("QnoteA.md", disk_text, String(rd2["hash"]))
	_ok(bool(w2.get("ok", false)), "W4: a second outside write lands")
	_app.assistant_window.note_saved.emit("QnoteA.md")
	await _settle(8)
	_ok(String(ed_text.call()).contains("zzy") and not String(ed_text.call()).contains("Outside words."), "W4: with unsaved typing the editor is left alone")
	## The hash guard is the user's way out: the autosave raises its banner and nothing of the outsider's is overwritten.
	_ok(await _wait_until(func(): return not vw._save_block.is_empty(), 6000), "W4: the autosave raises the conflict banner")
	_ok(FileAccess.get_file_as_string(_root + "/QnoteA.md") == disk_text, "W4: and the file on disk is still the outsider's, untouched")
	vw._on_banner_choice("take_theirs")
	await _settle(10)
	_ok(String(ed_text.call()).contains("Outside words."), "W4: Take theirs shows the outsider's text")


# -- private ------------------------------------------------------------------------------------------

func _leg_private() -> void:
	print("== P. a private note shows as private in the Note tab ==")
	var vw = _app.vault_window
	var cl: Array = _clickable(null)
	var ia: int = int(cl[0])
	var ib: int = int(cl[1])
	_app.select("settlement", ib)
	_app.open_context_tab("note")
	await _settle(10)
	_ok(await _wait_until(func(): return vw.is_docked() and vw.docked_path() == "QnoteP.md", 4000), "P1: the Note tab holds the private note")
	var pn := _find_named(_app, "VaultPrivateNotice") as Label
	_ok(pn != null and pn.text.contains("frontmatter private: true"), "P1: the private line is drawn, with the reason ('%s')" % (pn.text if pn != null else "<none>"))
	var slot := _app.find_child("NoteSlot", true, false)
	_ok(pn != null and slot != null and slot.is_ancestor_of(pn), "P1: and it is inside the Note tab's frame")
	_ok(pn != null and pn.text.contains("assistant"), "P1: it says the assistant never gets it")
	await _shot("private_note_tab")
	## A public note: no line.
	_app.select("settlement", ia)
	await _settle(10)
	_ok(await _wait_until(func(): return vw.is_docked() and vw.docked_path() == "QnoteA.md", 4000), "P2: the Note tab holds the public note")
	_ok(_find_named(_app, "VaultPrivateNotice") == null, "P2: no private line on a public note")
	## Typing the marker into it shows the line live after the save.
	var te: TextEdit = vw._editor.text_edit
	await _click_at(te.get_global_rect().get_center())
	await _key(KEY_A, 0, false, true)
	await _type_text("---\nprivate: true\n---\n# QnoteA\nplain")
	var shown := await _wait_until(func(): return _find_named(_app, "VaultPrivateNotice") != null, 6000)
	_ok(shown, "P3: after the autosave the private line appears without reopening the note")
	var pn2 := _find_named(_app, "VaultPrivateNotice") as Label
	_ok(pn2 != null and pn2.text.contains("private: true"), "P3: with its reason ('%s')" % (pn2.text if pn2 != null else "<none>"))
	_ok(FileAccess.get_file_as_string(_root + "/QnoteA.md").contains("private: true"), "P3: the note on disk holds the marker (%s)" % FileAccess.get_file_as_string(_root + "/QnoteA.md").replace("
", "|"))
	## Removing it hides the line.
	te = vw._editor.text_edit
	await _click_at(te.get_global_rect().get_center())
	await _key(KEY_A, 0, false, true)
	await _type_text("# QnoteA\nplain again")
	var gone := await _wait_until(func(): return _find_named(_app, "VaultPrivateNotice") == null, 6000)
	_ok(gone, "P4: removing the marker hides the line after the save")
	await _hide_all()

