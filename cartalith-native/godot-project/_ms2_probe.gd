extends Node
## MS-2 probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-2: "all three domains reachable in one gesture; probe drives
## the hold and the slide"). Boots the real shell as a phone in a `--vp`-sized host `SubViewport` and drives REAL
## pointer events at the bottom bar's controls: the four cells, the domain tab's tap, hold-and-slide fan, sideways
## swipe, the STORY and MENU tabs, system back, Session mode's refusal and one finger arriving as both a touch and
## its emulated mouse event (`MISTAKES.md`, "cargo test cannot see a broken shell").
##
## Windowed, with a wiped user dir (the shell remembers coach marks and layout in the user dir):
##   godot --path . --resolution 1600x1000 _ms2_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch
##   godot --path . --resolution 1600x1000 _ms2_probe.tscn -- --vp 1440x3120 --density 3
##   godot --path . --resolution 1600x1000 _ms2_probe.tscn -- --vp 2388x1668 --density 2 --force-touch
##   godot --path . --resolution 1600x1000 _ms2_probe.tscn -- --vp 720x1560 --density 2
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## **Every expected value is a literal written by hand from the proposal** (the tab order and captions, 44 dp,
## the domain order WORLD -> CIVIL -> CARTO); nothing is asserted against the constant it came from. The bar must
## go red on the tree before MS-2 (it has no `domain` tab), and under the mutants run by hand and listed in the
## commit: `DOMAIN_HOLD_MS` 380 -> 5000 (the hold never fires), `DOMAIN_SWIPE_UNITS` 48 -> 0 (a tap swipes),
## `DOMAIN_ORDER` reversed, `_phone_tab_for_domain` returning "domain" for MAP, and `TARGET_UNITS` 64 -> 30.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch"]:
			i += 1
		elif a in ["--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	_dens = float(_arg("--density", "1"))
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# ---- pointer helpers: real events pushed at the host viewport ------------------------------------------------

## One finger as the project delivers it with mouse emulation on: a touch and its emulated mouse button.
func _press(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = true
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(2)

func _move(p: Vector2, from: Vector2, both: bool) -> void:
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - from
		_vp.push_input(d)
	_vp.push_input(m)
	await _frames(2)

func _release(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = false
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2, both: bool = false) -> void:
	await _press(p, both)
	await _release(p, both)

## Hold on `p` for `ms`, slide in three steps to `to`, and (if `lift`) release there.
func _hold_slide(p: Vector2, to: Vector2, ms: int, lift: bool, both: bool = false) -> void:
	await _press(p, both)
	await get_tree().create_timer(ms / 1000.0).timeout
	var cur := p
	for s in 3:
		var nxt := p.lerp(to, float(s + 1) / 3.0)
		await _move(nxt, cur, both)
		cur = nxt
	if lift:
		await _release(to, both)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

func _caption(id: String) -> String:
	var l := (_app._phone_tab_cells.get(id, {}) as Dictionary).get("label") as Label
	return l.text if l != null else "<none>"

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_texts(c, out)

func _menu_has(text: String) -> bool:
	var out: Array = []
	_texts(_app._phone_menu, out)
	for t in out:
		if String(t).contains(text):
			return true
	return false

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

# ---- the run -------------------------------------------------------------------------------------------------

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone (the 11-inch tablet included, Ruling EG)")
	if not DccTheme.is_phone():
		return

	# 1. the bar: four cells in this order, these captions, 44 dp targets (written out, not read from the shell)
	var order := ["map", "domain", "story", "menu"]
	var want_caps := {"map": "MAP", "domain": "GENERATE", "story": "STORY", "menu": "MENU"}
	var last_x := -1.0
	for id in order:
		var b := _cell(id)
		_ok(b != null and b.is_visible_in_tree(), "tab '%s' is drawn" % id)
		if b == null:
			return
		var r := b.get_global_rect()
		## dp = px / the run's density: the phone's cells are sized in physical px (no content scale).
		_ok(r.size.x / _dens >= 44.0 and r.size.y / _dens >= 44.0,
			"tab '%s' is at least 44 x 44 dp (%s px at density %s)" % [id, str(r.size), str(_dens)])
		## Portrait: a bottom bar read left to right. Landscape: a left rail read top to bottom.
		var along: float = r.position.y if _app._landscape else r.position.x
		_ok(along > last_x, "tab '%s' follows the one before it along the %s" % [id, "rail" if _app._landscape else "bar"])
		last_x = along
		_ok(_caption(id) == want_caps[id], "tab '%s' reads '%s' (got '%s')" % [id, want_caps[id], _caption(id)])
	_ok(_app._active_domain == "world" and _app._phone_tab == "domain", "boots on WORLD with the domain tab lit")

	# 2. a plain tap on the domain tab opens the domain sheet and changes no domain; a second tap folds it
	await _tap(_centre("domain"), true)
	await _settle()
	_ok(_app._active_domain == "world" and _app._phone_detent == "half", "tap: WORLD stays, sheet at half (%s)" % _app._phone_detent)
	_ok(_app._domain_fan == null or not bool(_app._domain_fan.is_open()), "tap: no fan opened")
	await _tap(_centre("domain"), true)
	await _settle()
	_ok(_app._phone_detent == "peek", "second tap folds the sheet to peek (%s)" % _app._phone_detent)

	# 3. HOLD, slide to CIVIL, lift: one gesture reaches a domain (hold 380 ms is the design figure)
	var cell_c := _centre("domain")
	await _hold_slide(cell_c, cell_c, 600, false, true)
	_ok(_app._domain_fan != null and bool(_app._domain_fan.is_open()), "hold 600 ms opens the domain fan")
	if _app._domain_fan == null:
		return
	var tgt: Dictionary = _app._domain_fan._centres
	_ok(tgt.size() == 3, "the fan offers three domains (got %d)" % tgt.size())
	var fan_dia: float = float(_app._domain_fan.TARGET_UNITS) * float(_app.phone_scale())
	_ok(fan_dia / _dens >= 44.0, "fan targets are at least 44 dp across (%.1f dp)" % (fan_dia / _dens))
	## Every target is wholly on screen (a finger cannot hit one off the edge) and none overlaps another.
	var half_t := fan_dia * 0.5
	var on_screen := true
	var keys: Array = tgt.keys()
	for k in keys:
		var c0: Vector2 = tgt[k]
		if c0.x - half_t < -0.5 or c0.y - half_t < -0.5 or c0.x + half_t > vw + 0.5 or c0.y + half_t > vh + 0.5:
			on_screen = false
	_ok(on_screen, "all three fan targets lie wholly on screen")
	var apart := true
	for a in keys.size():
		for b in range(a + 1, keys.size()):
			if (tgt[keys[a]] as Vector2).distance_to(tgt[keys[b]] as Vector2) < fan_dia - 1.0:
				apart = false
	_ok(apart, "no two fan targets overlap")
	var civ: Vector2 = tgt.get("civilization", Vector2.ZERO)
	_ok(civ != Vector2.ZERO and ((civ.x > cell_c.x) if _app._landscape else (civ.y < cell_c.y)),
		"the CIVIL target opens %s the domain cell" % ("beside" if _app._landscape else "above"))
	var cur := cell_c
	for s in 3:
		var nxt := cell_c.lerp(civ, float(s + 1) / 3.0)
		await _move(nxt, cur, true)
		cur = nxt
	await _release(civ, true)
	await _settle()
	_ok(_app._active_domain == "civilization", "slide + lift on CIVIL selects civilization (got %s)" % _app._active_domain)
	_ok(_caption("domain") == "CIVIL", "the domain tab now reads CIVIL (got %s)" % _caption("domain"))
	_ok(_app._phone_tab == "domain", "the domain tab stays lit")
	_ok(not bool(_app._domain_fan.is_open()), "the fan closed after the pick")

	# 4. hold, slide to CARTO, lift
	cell_c = _centre("domain")
	await _hold_slide(cell_c, cell_c, 600, false, true)
	var carto: Vector2 = _app._domain_fan._centres.get("cartography", Vector2.ZERO)
	cur = cell_c
	for s in 3:
		var nxt2 := cell_c.lerp(carto, float(s + 1) / 3.0)
		await _move(nxt2, cur, true)
		cur = nxt2
	await _release(carto, true)
	await _settle()
	_ok(_app._active_domain == "cartography" and _caption("domain") == "CARTO", "slide to CARTO: domain cartography, tab CARTO (got %s / %s)" % [_app._active_domain, _caption("domain")])

	# 5. a sideways swipe cycles WORLD -> CIVIL -> CARTO -> WORLD, left = next, right = previous; once per finger
	cell_c = _centre("domain")
	var dx: float = 48.0 * float(_app.phone_scale()) + 24.0
	await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
	await _settle()
	_ok(_app._active_domain == "world", "swipe left from CARTO wraps to WORLD, exactly one step (got %s)" % _app._active_domain)
	await _hold_slide(cell_c, cell_c + Vector2(dx, 0), 0, true, true)
	await _settle()
	_ok(_app._active_domain == "cartography", "swipe right from WORLD wraps back to CARTO (got %s)" % _app._active_domain)
	await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
	await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
	await _settle()
	_ok(_app._active_domain == "civilization", "two left swipes from CARTO: WORLD then CIVIL (got %s)" % _app._active_domain)
	_ok(_app._domain_fan == null or not bool(_app._domain_fan.is_open()), "a swipe never opens the fan")

	# 6. a short wobble inside the tap slop is a tap, not a swipe
	var dom_before: String = _app._active_domain
	await _hold_slide(cell_c, cell_c + Vector2(10, 0), 0, true, true)
	await _settle()
	_ok(_app._active_domain == dom_before, "a 10 px wobble does not change domain (got %s)" % _app._active_domain)

	# 7. system back closes an open fan first
	cell_c = _centre("domain")
	await _hold_slide(cell_c, cell_c, 600, false, true)
	_ok(bool(_app._domain_fan.is_open()), "fan open again for the back test")
	## `notification()`, not a direct `_notification()` call: the engine's dispatch runs the shell's handler AND
	## the app's, and a direct call reaches only the app's override (found by this probe's first run).
	_app.notification(NOTIFICATION_WM_GO_BACK_REQUEST)
	await _frames(2)
	_ok(not bool(_app._domain_fan.is_open()), "system back closes the fan")
	await _release(cell_c, true)
	await _settle()

	# 8. Session mode: a refused target is dim and cannot be picked
	_app._session_mode = true
	await _select_world_safe()
	cell_c = _centre("domain")
	await _hold_slide(cell_c, cell_c, 600, false, true)
	var entries: Array = _app._domain_fan._entries
	var refused_world := false
	for e in entries:
		if String(e["id"]) == "world":
			refused_world = bool(e["refused"])
	_ok(refused_world, "Session mode: the WORLD target is drawn refused")
	var wpos: Vector2 = _app._domain_fan._centres.get("world", Vector2.ZERO)
	_ok(String(_app._domain_fan._target_at(wpos)) == "", "Session mode: the refused target cannot be picked")
	await _release(cell_c.lerp(wpos, 1.0), true)
	await _settle()
	_app._session_mode = false

	# 9. STORY: opens the Story screen with its four bands; the planner is reachable under Journeys
	await _tap(_centre("story"), true)
	await _settle()
	_ok(_app._phone_menu.is_open() and _app._phone_tab == "story", "STORY tab opens the phone menu, lit")
	for lab in ["Timeline", "Journeys", "Conflicts & campaigns", "Notes"]:
		_ok(_menu_has(lab), "Story screen lists '%s'" % lab)
	_app._phone_menu.open_story("JOURNEYS")
	await _settle()
	_ok(_menu_has("Journey planner"), "STORY > Journeys reaches the Journey planner (the PLAN tab's old door)")
	await _tap(_centre("story"), true)
	await _settle()
	_ok(not _app._phone_menu.is_open(), "a second tap on the lit STORY tab closes it")

	# 10. MENU: still the old MORE root, titled Menu
	await _tap(_centre("menu"), true)
	await _settle()
	_ok(_app._phone_menu.is_open() and _app._phone_tab == "menu", "MENU tab opens the menu, lit")
	_ok(_menu_has("Preferences") and _menu_has("Help"), "MENU still reaches Preferences and Help")
	await _tap(_centre("menu"), true)
	await _settle()

	# 11. MAP still selects CARTOGRAPHY and stays lit
	await _tap(_centre("map"), true)
	await _settle()
	_ok(_app._active_domain == "cartography" and _app._phone_tab == "map", "MAP selects CARTOGRAPHY and stays lit (got %s / %s)" % [_app._active_domain, _app._phone_tab])

func _select_world_safe() -> void:
	## Session mode hides WORLD; start the refusal test from CIVIL so the shell is not on a hidden domain.
	_app._select_domain("civilization")
	await _settle()
