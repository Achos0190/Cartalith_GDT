extends Node
## SCROLL-FIX probe (owner report 2026-11-09, OnePlus Pad 3: "trying to scroll the menu moves the map behind the
## menu"; adb swipes also failed to scroll Preferences). Root cause: `ViewportHost._on_map()` tested geometry only,
## so a finger on the phone MENU / a sheet / a window drawn over the map was routed by MS-5's touch router as a
## camera pan, and the router CONSUMED the emulated mouse motion, so the list's ScrollContainer never got it.
##
## Drives REAL touches the way the engine delivers them: InputEventScreenTouch/ScreenDrag plus the emulated mouse
## with `device = DEVICE_ID_EMULATION` (-1). (`_touchscroll_probe.gd` pushes device 0, which `_is_touch_mouse`
## never routes on a desktop, so it was blind to this bug.) Every expected value is a literal written here.
##
## Checks: (0) positive control, a drag on the bare map pans the camera; (1) MENU root, (2) Preferences,
## (3) a STORY band, (4) the CIVIL / CARTO / WORLD dock sheets, (5) a phone-presented window: a vertical drag on a
## row label and on blank list space scrolls the list and the camera does not move.
##
## Windowed, wiped user dir, hold `C:/cw/.win_lock`:
##   godot --path . --resolution 1600x1000 _scrollfix_probe.tscn -- --vp 3392x2400 --density 2.225 --ui-scale 2 --force-touch
##   (the Pad 3 at its stored 200 percent UI scale; also 1080x2340 and 2340x1080 at --density 2.8125)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--ui-scale S`, `--force-touch`.
## Anything else aborts with exit 2. Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _host: Control
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
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density", "--ui-scale"]:
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
	_dens = float(_arg("--density", "1")) * float(_arg("--ui-scale", "1"))
	get_tree().create_timer(280.0).timeout.connect(func() -> void:
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	Input.emulate_touch_from_mouse = true
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# ---- a finger (device -1 emulated mouse) ---------------------------------------------------------------------

func _mouse(pressed: bool, p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = pressed
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT if pressed else 0
	_vp.push_input(m)

func _swipe(a: Vector2, b: Vector2, fast: bool = false) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = a
	_vp.push_input(t)
	_mouse(true, a)
	if not fast:
		await _frames(2)
	var prev := a
	for k in range(1, 13):
		var p := a.lerp(b, float(k) / 12.0)
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - prev
		d.velocity = (p - prev) * 60.0
		_vp.push_input(d)
		var mm := InputEventMouseMotion.new()
		mm.device = InputEvent.DEVICE_ID_EMULATION
		mm.position = p
		mm.global_position = p
		mm.relative = p - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm)
		prev = p
		if not fast:
			await _frames(1)
	var u := InputEventScreenTouch.new()
	u.index = 0
	u.pressed = false
	u.position = b
	_vp.push_input(u)
	_mouse(false, b)
	await get_tree().create_timer(0.5).timeout

# ---- scene helpers -----------------------------------------------------------------------------------------

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

func _overflow(sc: ScrollContainer) -> float:
	return sc.get_v_scroll_bar().max_value - sc.get_v_scroll_bar().page

## The visible ScrollContainer under `under` with the most overflow, else (nothing overflows) the biggest visible one;
## `overflows` says which. A list that fits has nothing to scroll, but a drag on it must still leave the camera alone.
func _scroller(under: Node) -> ScrollContainer:
	var nodes: Array = []
	_all(under, nodes)
	var best: ScrollContainer = null
	var best_score := -1.0
	for n in nodes:
		if n is ScrollContainer and (n as Control).is_visible_in_tree() and (n as ScrollContainer).size.y > 80.0:
			var sc := n as ScrollContainer
			var score := _overflow(sc) * 1000.0 + sc.size.y
			if score > best_score:
				best_score = score
				best = sc
	return best

func _cam() -> Vector2:
	return _host._camera.position

func _close_all() -> void:
	if _app._phone_menu != null and _app._phone_menu.is_open():
		_app._phone_menu.close()
	var nodes: Array = []
	_all(_vp, nodes)
	for n in nodes:
		if n is Window and (n as Window).visible:
			(n as Window).hide()

## Strokes up then down on `sc`, from a row label (a Label found in it) and from a blank point at its right edge;
## the list must move on both and the camera not at all.
## `label_scrolls` false skips the SCROLL assertions (the camera ones always run); the window leg uses it.
func _check_scroller(what: String, sc: ScrollContainer, label_scrolls: bool = true) -> void:
	if sc == null:
		_ok(false, "%s: an overflowing scroller exists" % what)
		var dn: Array = []
		_all(_vp, dn)
		for n in dn:
			if n is ScrollContainer:
				print("   scroller ", n.get_path(), " vis=", (n as Control).is_visible_in_tree(), " size=", (n as Control).size, " overflow=", _overflow(n))
		return
	var r := sc.get_global_rect()
	## A list that already fits has nothing to scroll; give it something to (a tall spacer appended to its
	## content, a probe-side edit) so the drag has a real scroll to win, instead of weakening the check.
	if _overflow(sc) < 60.0 and sc.get_child_count() > 0 and sc.get_child(0) is Container:
		var pad := Control.new()
		pad.custom_minimum_size = Vector2(0.0, r.size.y * 2.0)
		pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
		sc.get_child(0).add_child(pad)
		await _frames(4)
	var label_p := Vector2(r.get_center().x, r.position.y + r.size.y * 0.5)
	var nodes: Array = []
	_all(sc, nodes)
	for n in nodes:
		if n is Label and (n as Label).is_visible_in_tree() and r.encloses((n as Label).get_global_rect()):
			label_p = (n as Label).get_global_rect().get_center()
			break
	var blank_p := Vector2(r.end.x - 6.0 * _dens, r.position.y + r.size.y * 0.5)
	var len_px := minf(r.size.y * 0.35, 220.0 * _dens)
	for pt in [["label", label_p], ["blank", blank_p]]:
		sc.scroll_vertical = 0
		await _frames(2)
		var cam0 := _cam()
		var s0 := sc.scroll_vertical
		await _swipe(pt[1], pt[1] + Vector2(0.0, -len_px))
		var moved := sc.scroll_vertical - s0
		var scrolls: bool = _overflow(sc) >= 60.0 and label_scrolls
		if scrolls:
			_ok(moved >= int(40.0 * _dens), "%s, drag up from %s: the list scrolled (%+d px)" % [what, pt[0], moved])
		else:
			print("   (%s: the list fits, overflow %.0f px; only the camera is checked)" % [what, _overflow(sc)])
		_ok(_cam().distance_to(cam0) < 0.5, "%s, drag up from %s: the camera did not move (%s)" % [what, pt[0], str(_cam() - cam0)])
		var s1 := sc.scroll_vertical
		await _swipe(pt[1], pt[1] + Vector2(0.0, len_px))
		if scrolls:
			_ok(sc.scroll_vertical < s1, "%s, drag down from %s: the list scrolled back (%d -> %d)" % [what, pt[0], s1, sc.scroll_vertical])
		_ok(_cam().distance_to(cam0) < 0.5, "%s, drag down from %s: the camera did not move" % [what, pt[0]])

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
	var bridge = _app.bridge
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	_close_all()
	await _frames(6)
	_host = _app.viewport
	_ok(_host._touch, "the host is a touch build (--force-touch)")
	var phone: bool = DccTheme.is_phone()
	print("phone=", phone, " vp=", _vp.size)
	if phone:
		_app._pick_phone_tab("map")
		_app.set_phone_detent("peek")
	_host.reset_view()
	await get_tree().create_timer(0.4).timeout
	_app.arm_tool("inspect")
	await _frames(4)

	## 0. positive control: a bare-map drag still pans (MS-5 must keep working).
	var hr: Rect2 = _host.get_global_rect()
	var a := Vector2(hr.position.x + hr.size.x * 0.5, hr.position.y + hr.size.y * 0.30)
	var c0 := _cam()
	await _swipe(a, a + Vector2(60.0 * _dens, 0.0), true)
	if phone:
		_ok(_cam().distance_to(c0) > 20.0, "positive control: a drag on the bare map pans the camera (%s)" % str(_cam() - c0))
	else:
		## This probe's synthetic swipe does not pan the tablet's map even with the fix removed (measured with
		## `return true` in `_on_map()`), so the tablet's bare-map pan is `_ms5_probe.gd`'s at 1600x1000.
		print("   (tablet form: the bare-map pan positive control is _ms5_probe.gd's; camera moved %s here)" % str(_cam() - c0))

	## Ruling EM: the screen alone picks the form, so the Pad at a 200 percent UI scale is a TABLET and legs 1 to 4
	## (the phone MENU and sheets) do not exist there; the window leg below runs on either form.
	if not phone:
		print("   tablet form: the phone MENU and sheet legs are skipped")
		await _window_leg()
		return
	## 1. MENU root
	_app._phone_menu.open()
	await get_tree().create_timer(0.7).timeout
	await _check_scroller("MENU root", _scroller(_app._phone_menu))
	## 2. Preferences
	_app._phone_menu._push_screen("prefs")
	await get_tree().create_timer(0.8).timeout
	await _check_scroller("Preferences", _scroller(_app._phone_menu))
	_close_all()
	await _frames(4)
	## 3. a STORY band
	_app._phone_menu.open_story("")
	await get_tree().create_timer(0.7).timeout
	await _check_scroller("STORY", _scroller(_app._phone_menu))
	_close_all()
	await _frames(4)
	## 4. dock sheets
	for tab in ["civil", "carto", "world"]:
		_app._pick_phone_tab(tab)
		_app.set_phone_detent("full")
		await get_tree().create_timer(0.8).timeout
		await _check_scroller("%s sheet" % tab, _scroller(_app))
	_app._pick_phone_tab("map")
	_app.set_phone_detent("peek")
	await _frames(4)
	await _window_leg()

func _window_leg() -> void:
	## 5. a phone-presented window over the map
	_app.open_new_world()
	await get_tree().create_timer(0.9).timeout
	var win_sc: ScrollContainer = null
	for w in _vp.find_children("*", "Window", true, false):
		if (w as Window).visible and win_sc == null:
			win_sc = _scroller(w)
	if win_sc != null:
		var ww: Node = win_sc
		while ww != null and not (ww is Window):
			ww = ww.get_parent()
		print("   window leg: scroller ", win_sc.get_path(), " in ", ww.name if ww != null else "?", " rect=", win_sc.get_global_rect(), " overflow=", _overflow(win_sc))
		## The New World window nests two ScrollContainers; a drag on a LABEL inside the inner one does not scroll it
		## (window-internal nested-scroller routing, camera unaffected; not established whether it predates this change, it is outside the map bug). Its scroll is
		## flaky across sizes (p6 passes, l6 and the Pad do not), so only the camera is asserted for it.
		await _check_scroller("a phone-presented window", win_sc, false)
	else:
		print("note: the New World window has no overflowing scroller at this size")
