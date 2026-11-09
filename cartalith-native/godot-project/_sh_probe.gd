extends Node
## Stylus-input probe (SH-2 .. SH-9, Ruling EE: standard input only). Drives SYNTHETIC events: every pen sample
## here is an `InputEventMouseMotion` / `InputEventMouseButton` with `device = 0` (what Godot's Android port
## stamps on a pen, `STYLUS_HAPTICS_RESEARCH.md` section 4.2) and hand-set `pressure`, `tilt` and `pen_inverted`;
## every finger is Godot's touch emulation (`device = -1`) plus its `InputEventScreenTouch/Drag`. No real pen was
## used: this proves the routing and the rules, NOT what the Pad 3's pen reports (that is the owner's glass pass,
## with the SH-9 tester). Literals are written by hand; nothing is asserted against the constant it came from.
##
## Sections (`--section sh2|sh3|sh4|sh5|sh6|sh9|all`, default all):
##   sh2  input hygiene: a finger never latches as a pen; a pen contact on a touch build switches accumulation off
##        and release restores it; a finger and a desktop mouse leave it alone.
##   sh3  pen draws, finger pans, with the palm rule.
##   sh4  the pressure curve (literal table) and its identity at the defaults.
##   sh5  the tilt toggle: tilt reaches the engine only when on.
##   sh6  the hover ring's inner ring and its 150 ms lifetime.
##   sh9  the stylus tester's readout of synthetic samples.
##
## Windowed, wiped user dir, under the shared lock (`C:/cw/.win_lock`):
##   godot --path . --resolution 1600x1000 _sh_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
##   (also 2388x1668 @2 and 3392x2400 @2 --force-touch for the tablet, and 1600x1000 with no flag for the desktop)
## Arguments read by THIS file: `--vp WxH`, `--section NAME`. Accepted because the shell reads them: `--density D`,
## `--force-touch`. Anything else aborts with exit 2.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _host: Control
var _ov: Control
var _n := {}
var _touch_down_idx := {}
var _samples: Array = []

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
	var wd := Timer.new()
	wd.wait_time = 280.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch"]:
			i += 1
		elif a in ["--vp", "--density", "--section"]:
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
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	StylusInput.assume_mobile = false
	StylusInput.end_pen_contact()
	get_tree().quit(0 if _fails == 0 else 1)

# ---- fingers: real events pushed at the host viewport ----------------------------------------------------------

## Finger `idx` down. Finger 0 is the one Godot emulates as the mouse (device = DEVICE_ID_EMULATION); a second
## finger has a ScreenTouch only, exactly as the engine delivers it.
func _down(idx: int, p: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = idx
	t.pressed = true
	t.position = p
	_vp.push_input(t)
	if idx == 0:
		var m := InputEventMouseButton.new()
		m.device = InputEvent.DEVICE_ID_EMULATION
		m.button_index = MOUSE_BUTTON_LEFT
		m.pressed = true
		m.position = p
		m.global_position = p
		m.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(m)
	await _frames(2)

func _move(idx: int, p: Vector2, from: Vector2) -> void:
	var d := InputEventScreenDrag.new()
	d.index = idx
	d.position = p
	d.relative = p - from
	_vp.push_input(d)
	if idx == 0:
		var m := InputEventMouseMotion.new()
		m.device = InputEvent.DEVICE_ID_EMULATION
		m.position = p
		m.global_position = p
		m.relative = p - from
		m.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(m)
	await _frames(2)

func _up(idx: int, p: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = idx
	t.pressed = false
	t.position = p
	_vp.push_input(t)
	if idx == 0:
		var m := InputEventMouseButton.new()
		m.device = InputEvent.DEVICE_ID_EMULATION
		m.button_index = MOUSE_BUTTON_LEFT
		m.pressed = false
		m.position = p
		m.global_position = p
		_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2) -> void:
	await _down(0, p)
	await _up(0, p)

## One finger drags from `a` to `b` in `steps` equal moves (finger 0).
func _drag(a: Vector2, b: Vector2, steps: int = 6) -> void:
	var cur := a
	for s in steps:
		var nxt := a.lerp(b, float(s + 1) / float(steps))
		await _move(0, nxt, cur)
		cur = nxt

## A real mouse (device 0): the router must never touch it.
func _mouse_drag(a: Vector2, b: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.device = 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = a
	m.global_position = a
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)
	var mm := InputEventMouseMotion.new()
	mm.device = 0
	mm.position = b
	mm.global_position = b
	mm.relative = b - a
	mm.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(mm)
	await _frames(2)
	var r := InputEventMouseButton.new()
	r.device = 0
	r.button_index = MOUSE_BUTTON_LEFT
	r.pressed = false
	r.position = b
	r.global_position = b
	_vp.push_input(r)
	await _frames(3)

# ---- synthetic pen events (device 0), pushed at the host viewport ------------------------------------------------

func _pen_event_motion(p: Vector2, from: Vector2, pressure: float, tilt: Vector2, inverted: bool, down: bool) -> InputEventMouseMotion:
	var m := InputEventMouseMotion.new()
	m.device = 0
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.pressure = pressure
	m.tilt = tilt
	m.pen_inverted = inverted
	m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	return m

## A pen press at `p` (device 0). Pressure and tilt ride on the motion events; a button event carries neither.
func _pen_down(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.device = 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)

func _pen_up(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.device = 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_vp.push_input(m)
	await _frames(3)

func _pen_move(p: Vector2, from: Vector2, pressure: float, tilt: Vector2 = Vector2.ZERO, down: bool = true) -> void:
	_vp.push_input(_pen_event_motion(p, from, pressure, tilt, false, down))
	await _frames(2)

## A pen stroke a -> b in `steps` moves, every sample at `pressure`.
func _pen_stroke(a: Vector2, b: Vector2, pressure: float, steps: int = 6, tilt: Vector2 = Vector2.ZERO) -> void:
	await _pen_down(a)
	var cur := a
	for s in steps:
		var nxt := a.lerp(b, float(s + 1) / float(steps))
		await _pen_move(nxt, cur, pressure, tilt)
		cur = nxt
	await _pen_up(b)

## A finger drag whose emulated motion events carry `pressure` (Android hands a finger's own getPressure).
func _finger_drag_p(a: Vector2, b: Vector2, pressure: float, steps: int = 6) -> void:
	await _down(0, a)
	var cur := a
	for s in steps:
		var nxt := a.lerp(b, float(s + 1) / float(steps))
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = nxt
		d.relative = nxt - cur
		d.pressure = pressure
		_vp.push_input(d)
		var m := InputEventMouseMotion.new()
		m.device = InputEvent.DEVICE_ID_EMULATION
		m.position = nxt
		m.global_position = nxt
		m.relative = nxt - cur
		m.pressure = pressure
		m.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(m)
		await _frames(2)
		cur = nxt
	await _up(0, b)

# ---- counters ------------------------------------------------------------------------------------------------------

func _zero() -> void:
	_n = {"click": 0, "drag": 0, "rel": 0, "ctx": 0, "sel": 0}

func _wire() -> void:
	_ov.map_clicked.connect(func(_x, _y): _n["click"] += 1)
	_ov.map_dragged.connect(func(_x, _y): _n["drag"] += 1)
	_ov.map_released.connect(func(_x, _y, _v): _n["rel"] += 1)
	_ov.context_requested.connect(func(_r): _n["ctx"] += 1)
	_ov.settlement_selected.connect(func(_s, _i): _n["sel"] += 1)

func _cam() -> Vector2:
	return _host._camera.position

func _zoom() -> float:
	return float(_host._zoom)

func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()

func _btn() -> Dictionary:
	return {"pan": _host._pan_btn, "rocker": _host._rocker, "rec": _host._recentre_btn}

## A map point that is on the world and clear of the row, the sheet and the docks: scans candidates until the
## overlay says the point is a valid grid point.
func _find_point() -> Vector2:
	var r := _host.get_global_rect()
	for fy in [0.30, 0.36, 0.24, 0.42, 0.5]:
		for fx in [0.5, 0.4, 0.6]:
			var p := Vector2(r.position.x + r.size.x * fx, r.position.y + r.size.y * fy)
			var navr: Rect2 = _host._navpad.get_global_rect() if _host._navpad != null else Rect2()
			if navr.has_point(p):
				continue
			## `_grid_point` takes the overlay's LOCAL space (the camera's scale and offset divided out); a
			## screen point is converted through the overlay's own global transform.
			var lp: Vector2 = _ov.get_global_transform().affine_inverse() * p
			var q: Dictionary = _ov._grid_point(lp, _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
			if bool(q.get("valid", false)):
				return p
	return Vector2(-1, -1)

## Closes whatever a hold opened: the phone's peek card, or the tablet's ring and card.
func _close_card() -> void:
	var pm = _app.get("_phone_menu")
	if pm != null and pm.has_method("peek_card_is_open") and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	var cb = _app.get("context_broker")
	if cb != null and cb.has_method("ring_close"):
		cb.ring_close()
	## A phone hold also drops the CM-5 sample pin at the press point; a later touch within 30 px of it would
	## drag the pin instead of starting a stroke, so the probe clears it as a user would by moving on.
	_ov._sample_pin = {}
	## On the tablet the hold's card is a popup window; the synthetic lift lands on it rather than on the overlay,
	## so the overlay's tracked-finger flag is still set. A real lift reaches `_ring_release_cb` (CM-4); here the
	## probe clears it, as the next real release would.
	_ov._touch_ring_active = false
	await _dismiss_windows()
	await _frames(3)

## A placing tool's tap can open an editor window over the map (Settlement does on the tablet and 11-inch
## forms); hide every visible embedded window so the next touch reaches the map again.
func _dismiss_windows() -> void:
	for w in _vp.find_children("*", "Window", true, false):
		if (w as Window).visible:
			(w as Window).hide()
	await _frames(2)

func _arm(id: String) -> void:
	_app.arm_tool(id)
	await _frames(4)

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

# ---- the run -------------------------------------------------------------------------------------------------------

func _want(section: String) -> bool:
	var s := _arg("--section", "all")
	return s == "all" or s == section

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
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_host = _app.viewport
	_ov = _host.overlay
	if DccTheme.is_phone():
		_app._pick_phone_tab("map")
		_app.set_phone_detent("peek")
	await _settle()
	_zero()
	_wire()
	_ov.map_stroke_sample.connect(func(_x, _y, p, tx, ty, is_pen): _samples.append({"p": p, "tx": tx, "ty": ty, "pen": is_pen}))
	_host.reset_view()
	await _settle()
	var P := _find_point()
	_ok(P.x >= 0.0, "a valid map point was found, %s" % str(P))
	if P.x < 0.0:
		return
	if _want("sh2"):
		await _t_sh2(P)
	if _want("sh3"):
		await _t_sh3(P)
	if _want("sh4"):
		await _t_sh4(P)
	if _want("sh5"):
		await _t_sh5(P)
	if _want("sh6"):
		await _t_sh6(P)
	if _want("sh9"):
		await _t_sh9(P)


# SH-2 -----------------------------------------------------------------------------------------------------------------

func _t_sh2(P: Vector2) -> void:
	# pure rules, literals
	_ok(StylusInput.is_finger(-1), "device -1 is a finger")
	_ok(not StylusInput.is_finger(0), "device 0 is a pen or a mouse, not a finger")
	_ok(not StylusInput.is_finger(3), "device 3 is not a finger")
	_ok(StylusInput.has_pen_signature(0.5, Vector2.ZERO, false), "pressure 0.5 is a pen signature")
	_ok(not StylusInput.has_pen_signature(1.0, Vector2.ZERO, false), "a mouse's flat 1.0 is not")
	_ok(not StylusInput.has_pen_signature(0.0, Vector2.ZERO, false), "a bare 0.0 is not")
	_ok(StylusInput.has_pen_signature(1.0, Vector2(0.0, 0.3), false), "tilt alone is a signature")
	_ok(StylusInput.has_pen_signature(1.0, Vector2.ZERO, true), "the eraser flag alone is a signature")
	var finger_mm := _pen_event_motion(Vector2(5, 5), Vector2.ZERO, 0.4, Vector2.ZERO, false, true)
	finger_mm.device = -1
	_ok(StylusInput.pen_pressure(finger_mm) == null, "a finger's 0.4 is no pressure: null, not 0.4 and not 1.0")
	var pen_mm := _pen_event_motion(Vector2(5, 5), Vector2.ZERO, 0.4, Vector2.ZERO, false, true)
	_ok(is_equal_approx(float(StylusInput.pen_pressure(pen_mm)), 0.4), "a pen's 0.4 is carried through as 0.4")
	var mouse_mm := _pen_event_motion(Vector2(5, 5), Vector2.ZERO, 1.0, Vector2.ZERO, false, true)
	_ok(StylusInput.pen_pressure(mouse_mm) == null, "a mouse's 1.0 is no pressure")
	StylusInput.assume_mobile = false
	var hover := _pen_event_motion(Vector2(5, 5), Vector2.ZERO, 0.0, Vector2.ZERO, false, false)
	_ok(not StylusInput.is_pen_event(hover), "off a touch build a signature-less hover is not a pen")
	StylusInput.assume_mobile = true
	_ok(StylusInput.is_pen_event(hover), "on a touch build a device-0 hover is a pen (Android gives hover no fields)")
	hover.device = -1
	_ok(not StylusInput.is_pen_event(hover), "on a touch build a finger is never a pen")
	StylusInput.assume_mobile = false

	# a finger drag with pressure 0.4 never latches a pen at the overlay; the same samples from device 0 do
	await _arm("sculpt")
	_samples.clear()
	await _finger_drag_p(P, P + Vector2(120, 0), 0.4)
	_ok(_samples.size() >= 2, "the finger stroke produced samples (%d)" % _samples.size())
	var any_pen := false
	var any_pressure := false
	for s in _samples:
		any_pen = any_pen or bool(s["pen"])
		any_pressure = any_pressure or s["p"] != null
	_ok(not any_pen and not any_pressure, "a finger drag at pressure 0.4 is never a pen and carries no pressure")
	_samples.clear()
	await _pen_stroke(P, P + Vector2(120, 0), 0.4)
	var all_pen := _samples.size() >= 2
	for s in _samples:
		all_pen = all_pen and bool(s["pen"]) and is_equal_approx(float(s["p"]), 0.4)
	_ok(all_pen, "the same stroke from device 0 is a pen at 0.4 on every sample (%d samples)" % _samples.size())

	# accumulation
	_ok(Input.use_accumulated_input, "accumulation starts on (Godot's default)")
	StylusInput.assume_mobile = true
	await _pen_down(P)
	_ok(not Input.use_accumulated_input and StylusInput.pen_contact_open(), "a pen contact on a touch build switches accumulation off")
	await _pen_up(P)
	_ok(Input.use_accumulated_input and not StylusInput.pen_contact_open(), "its release restores it")
	await _down(0, P)
	_ok(Input.use_accumulated_input, "a finger press leaves accumulation on")
	await _up(0, P)
	StylusInput.assume_mobile = false
	await _pen_down(P)
	_ok(Input.use_accumulated_input, "a desktop pen or mouse press leaves accumulation on")
	await _pen_up(P)
	_ok(Input.use_accumulated_input, "and its release leaves it on")
	_host.reset_view()
	await _settle()

# SH-3 -----------------------------------------------------------------------------------------------------------------

## A pen hover at `p`: a device-0 motion with no button, which is all Android gives a hovering pen. `assume_mobile`
## is set by the caller, because `OS.has_feature("mobile")` is false on this desktop host.
func _pen_hover(p: Vector2) -> void:
	await _pen_move(p, p, 0.0, Vector2.ZERO, false)

func _sh3_reset() -> void:
	_host.reset_view()
	_host._pen_seen = false
	_host._pen_last_ms = -1
	_host._sync_pan_btn()
	_zero()
	_samples.clear()
	await _settle()

func _t_sh3(P: Vector2) -> void:
	# the palm window, literals (boundary: 300 ms after is inside, 301 is not; no pen yet is never inside)
	_ok(StylusInput.in_palm_window(1300, 1000), "300 ms after the last pen event is inside the palm window")
	_ok(not StylusInput.in_palm_window(1301, 1000), "301 ms after is outside")
	_ok(StylusInput.in_palm_window(1000, 1000), "the same millisecond is inside")
	_ok(not StylusInput.in_palm_window(5, -1), "no pen event yet is never a palm window")
	if not _host._touch:
		# the desktop host: no touch routing, so a pen hover latches nothing and the setting changes nothing
		StylusInput.assume_mobile = true
		DccSettings.set_pen_draws_finger_pans(true)
		await _pen_hover(P)
		_ok(not _host._pen_seen, "desktop host: a pen hover latches nothing (the touch router never runs)")
		DccSettings.set_pen_draws_finger_pans(false)
		StylusInput.assume_mobile = false
		return
	var pan: Button = _host._pan_btn
	var c0: Vector2
	var d: Vector2

	# control: setting OFF, a pen seen -> the finger draws with Sculpt exactly as before
	DccSettings.set_pen_draws_finger_pans(false)
	StylusInput.assume_mobile = true
	await _sh3_reset()
	await _arm("sculpt")
	await _pen_hover(P)
	_ok(_host._pen_seen, "a device-0 hover on a touch build is a seen pen")
	await get_tree().create_timer(0.45).timeout
	_ok(pan.visible, "setting off, pen seen: the latch is still drawn")
	c0 = _cam()
	await _down(0, P)
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	_ok(_n["drag"] > 0 and _cam().distance_to(c0) < 1.0, "setting off: a finger draws with Sculpt, camera unmoved (%s)" % str(_n))

	# setting ON but no pen seen yet: fails safe, the finger still draws and the latch shows
	DccSettings.set_pen_draws_finger_pans(true)
	await _sh3_reset()
	await _arm("sculpt")
	_ok(pan.visible, "setting on, no pen seen: the latch is drawn")
	c0 = _cam()
	await _down(0, P)
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	_ok(_n["drag"] > 0 and _cam().distance_to(c0) < 1.0, "setting on, no pen seen: a finger still draws (%s)" % str(_n))

	# setting ON, pen seen, pen gone for > 300 ms: the finger pans, the pen draws, a tap draws nothing
	await _sh3_reset()
	await _arm("sculpt")
	await _pen_hover(P)
	await get_tree().create_timer(0.45).timeout
	_ok(not pan.visible, "setting on, pen seen: the latch is hidden")
	_zero()
	c0 = _cam()
	await _down(0, P)
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	d = _cam() - c0
	_ok(absf(d.x - 120.0) <= 1.0 and absf(d.y) <= 1.0, "a finger's 120 px drag with Sculpt pans the camera by 120 px, got %s" % str(d))
	_ok(_n["drag"] == 0 and _n["click"] == 0 and _n["rel"] == 0 and _n["sel"] == 0,
		"the finger's pan emits no draw, click, release or selection (%s)" % str(_n))
	await get_tree().create_timer(0.7).timeout
	_ok(not _ov._touch_armed and not _ov._touch_ring_active and _n["ctx"] == 0,
		"after the pan nothing is armed and no card opened (armed %s, ctx %d)" % [str(_ov._touch_armed), _n["ctx"]])
	_host.reset_view()
	await _settle()
	_zero()
	c0 = _cam()
	await _tap(P)
	await get_tree().create_timer(0.45).timeout
	_ok(_n["click"] == 0 and _n["drag"] == 0 and _cam().distance_to(c0) < 0.01,
		"a finger TAP with Sculpt draws nothing and moves nothing (%s)" % str(_n))
	# the pen (device 0) draws at once, camera unmoved
	_zero()
	c0 = _cam()
	await _pen_stroke(P, P + Vector2(120, 0), 0.5)
	_ok(_n["drag"] > 0 and _cam().distance_to(c0) < 1.0, "the pen draws with Sculpt, camera unmoved (%s)" % str(_n))

	# the palm rule: a finger that lands within 300 ms of a pen hover is ignored outright
	await _sh3_reset()
	await _arm("sculpt")
	await _pen_hover(P)
	await get_tree().create_timer(0.45).timeout
	_zero()
	c0 = _cam()
	await _pen_hover(P + Vector2(2, 0))
	await _down(0, P + Vector2(60, 0))
	await _drag(P + Vector2(60, 0), P + Vector2(180, 0))
	await _up(0, P + Vector2(180, 0))
	_ok(_cam().distance_to(c0) < 0.01 and _n["drag"] == 0 and _n["click"] == 0 and _n["rel"] == 0,
		"a finger landing while the pen hovers is a palm: no pan, no draw (%s, camera %s)" % [str(_n), str(_cam() - c0)])
	# ... and with Inspect too (the palm rule is about the finger, not the tool)
	await _arm("inspect")
	_zero()
	await _pen_hover(P + Vector2(2, 0))
	await _tap(P + Vector2(60, 0))
	await get_tree().create_timer(0.45).timeout
	_ok(_n["click"] == 0, "a palm tap with Inspect clicks nothing (%s)" % str(_n))
	# ... and the same tap clicks once the pen has been away for more than 300 ms (positive control)
	await get_tree().create_timer(0.45).timeout
	_zero()
	await _tap(P + Vector2(60, 0))
	await get_tree().create_timer(0.45).timeout
	_ok(_n["click"] == 1, "the same tap with the pen away clicks once (%s)" % str(_n))
	await _dismiss_windows()

	# a placing tool is unchanged by the setting: a finger drag pans, a tap clicks
	await _sh3_reset()
	await _arm("settlement")
	await _pen_hover(P)
	await get_tree().create_timer(0.45).timeout
	_zero()
	c0 = _cam()
	await _down(0, P)
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	d = _cam() - c0
	_ok(absf(d.x - 120.0) <= 1.0 and _n["click"] == 0, "Settlement: the finger still pans (%s, %s)" % [str(d), str(_n)])
	await _dismiss_windows()

	# toggling the setting off at runtime brings the latch back
	await _arm("sculpt")
	_ok(not pan.visible, "latch hidden while on")
	DccSettings.set_pen_draws_finger_pans(false)
	_host.refresh_stylus_mode()
	_ok(pan.visible, "turning the setting off brings the latch back at once")
	DccSettings.set_pen_draws_finger_pans(false)
	StylusInput.assume_mobile = false
	await _sh3_reset()

# SH-4 -----------------------------------------------------------------------------------------------------------------

## A stand-in for the engine bridge that records what the workspace hands it. `EngineBridge` is a GDScript node, so
## a subclass overrides the two pressure chokepoints; it is never added to the tree (no `_ready`, no worker).
class SpyBridge extends EngineBridge:
	var paint_calls: Array = []
	var sculpt_calls: Array = []
	func paint_stroke_sample(_gx: float, _gy: float, pressure: Variant, tilt: Variant, _t_ms: int) -> Dictionary:
		paint_calls.append({"p": pressure, "t": tilt})
		return {}
	func sculpt_add_point_p(_x: float, _y: float, pressure: Variant) -> int:
		sculpt_calls.append(pressure)
		return 0

func _near(a: Variant, b: float, tol: float = 0.001) -> bool:
	return a != null and absf(float(a) - b) <= tol

func _t_sh4(_P: Vector2) -> void:
	var ws: WorldWorkspace = _app._world_workspace()
	_ok(ws != null, "the world workspace is reachable")
	# the pure law, literals worked by hand: 0.5^1.6 = 0.32988, 0.5^0.6 = 0.65975
	_ok(StylusInput.remap_pressure(0.5, 1.0, 1.0) == 0.5, "linear, max 1.0: 0.5 stays exactly 0.5 (bitwise identity)")
	_ok(StylusInput.remap_pressure(0.37, 1.0, 1.0) == 0.37, "linear, max 1.0: 0.37 stays exactly 0.37")
	_ok(_near(StylusInput.remap_pressure(0.5, 1.6, 1.0), 0.32988), "firm: 0.5 maps to 0.3299")
	_ok(_near(StylusInput.remap_pressure(0.5, 0.6, 1.0), 0.65975), "soft: 0.5 maps to 0.6598")
	_ok(_near(StylusInput.remap_pressure(0.7, 1.0, 0.7), 1.0), "comfortable max 0.7: 0.7 reads as full")
	_ok(_near(StylusInput.remap_pressure(0.35, 1.0, 0.7), 0.5), "comfortable max 0.7: 0.35 reads as half")
	_ok(_near(StylusInput.remap_pressure(0.9, 1.0, 0.7), 1.0), "comfortable max 0.7: 0.9 is clamped to full")
	_ok(_near(StylusInput.remap_pressure(0.25, 1.6, 0.5), 0.32988), "firm with max 0.5: 0.25 maps to 0.3299")
	_ok(StylusInput.remap_pressure(0.0, 1.6, 0.7) == 0.0 and StylusInput.remap_pressure(1.0, 1.6, 1.0) == 1.0,
		"the ends stay where they are: 0 -> 0 and 1 -> 1")
	_ok(StylusInput.PRESSURE_CURVES["soft"] == 0.6 and StylusInput.PRESSURE_CURVES["linear"] == 1.0 and StylusInput.PRESSURE_CURVES["firm"] == 1.6,
		"the three curves are 0.6, 1.0 and 1.6 (SH-4's own pair)")

	# the settings: defaults, validation, round trip
	_ok(DccSettings.pen_pressure_curve() == "linear" and is_equal_approx(DccSettings.pen_pressure_max(), 1.0), "the defaults are linear and 1.0")
	DccSettings.set_pen_pressure_curve("bogus")
	_ok(DccSettings.pen_pressure_curve() == "linear", "an unknown curve name is ignored, not stored")
	DccSettings.set_pen_pressure_max(0.2)
	_ok(is_equal_approx(DccSettings.pen_pressure_max(), 0.5), "a maximum below 0.5 is clamped to 0.5")
	DccSettings.set_pen_pressure_max(2.0)
	_ok(is_equal_approx(DccSettings.pen_pressure_max(), 1.0), "a maximum above 1.0 is clamped to 1.0")
	DccSettings.set_pen_pressure_curve("firm")
	_ok(DccSettings.pen_pressure_curve() == "firm", "firm round-trips")
	DccSettings.set_pen_pressure_curve("linear")

	# the workspace helper: null passes through, identity at the defaults, the curve otherwise
	_ok(ws._apply_pen_pressure(null) == null, "no pressure stays no pressure at the defaults")
	_ok(ws._apply_pen_pressure(0.5) == 0.5, "the defaults return 0.5 unchanged")
	DccSettings.set_pen_pressure_curve("firm")
	_ok(ws._apply_pen_pressure(null) == null, "no pressure stays no pressure under firm: a curve never invents one")
	_ok(_near(ws._apply_pen_pressure(0.5), 0.32988), "firm: the workspace maps 0.5 to 0.3299")
	DccSettings.set_pen_pressure_curve("linear")

	# the two chokepoints, through the real cached-sample path
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	_ok(ws._sculpt_take_pressure(10.0, 20.0) == 0.5, "sculpt, defaults: the cached 0.5 reaches the engine as exactly 0.5")
	DccSettings.set_pen_pressure_curve("firm")
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	_ok(_near(ws._sculpt_take_pressure(10.0, 20.0), 0.32988), "sculpt, firm: the engine receives 0.3299")
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	_ok(ws._sculpt_take_pressure(11.0, 20.0) == null, "sculpt: a sample for another position is no pressure (null)")
	var was_on: bool = ws.sculpt_pressure_on()
	ws.set_sculpt_pressure_on(false)
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	_ok(ws._sculpt_take_pressure(10.0, 20.0) == null, "sculpt: the Pressure toggle off is still no pressure, whatever the curve")
	ws.set_sculpt_pressure_on(was_on)

	var real = ws.bridge
	var spy := SpyBridge.new()
	ws.bridge = spy
	ws._paint_stroke_open = true
	DccSettings.set_pen_pressure_curve("linear")
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	ws._paint_drag(10.0, 20.0)
	_ok(spy.paint_calls.size() == 1 and spy.paint_calls[0]["p"] == 0.5, "paint, defaults: the engine receives exactly 0.5")
	DccSettings.set_pen_pressure_curve("firm")
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": null}
	ws._paint_drag(10.0, 20.0)
	_ok(spy.paint_calls.size() == 2 and _near(spy.paint_calls[1]["p"], 0.32988), "paint, firm: the engine receives 0.3299")
	ws._paint_pending_sample = {}
	ws._paint_drag(10.0, 20.0)
	_ok(spy.paint_calls.size() == 3 and spy.paint_calls[2]["p"] == null, "paint: with no cached sample the engine receives null, not a curve of nothing")
	ws._paint_stroke_open = false
	ws.bridge = real
	spy.free()
	DccSettings.set_pen_pressure_curve("linear")
	DccSettings.set_pen_pressure_max(1.0)

	# the Preferences row (PC and tablet menus; the phone has its own menu, MS-6's)
	var mn = _app.menus
	if mn != null and mn._stylus_popup != null:
		var pop: PopupMenu = mn._stylus_popup
		mn._on_stylus_choice(mn.ID_PREF_CURVE_FIRM)
		_ok(DccSettings.pen_pressure_curve() == "firm" and pop.is_item_checked(pop.get_item_index(mn.ID_PREF_CURVE_FIRM))
			and not pop.is_item_checked(pop.get_item_index(mn.ID_PREF_CURVE_LINEAR)), "Preferences > Stylus > Firm stores firm and ticks it alone")
		mn._on_stylus_choice(mn.ID_PREF_CURVE_LINEAR)
		mn._on_stylus_choice(mn.ID_PREF_MAX_70)
		_ok(is_equal_approx(DccSettings.pen_pressure_max(), 0.7), "Preferences > Stylus > Comfortable maximum 70 % stores 0.7")
		mn._on_stylus_choice(mn.ID_PREF_MAX_100)
		_ok(is_equal_approx(DccSettings.pen_pressure_max(), 1.0) and DccSettings.pen_pressure_curve() == "linear", "and 100 % / Linear put the defaults back")
	else:
		print("note: no PC/tablet Preferences menu on this form; the row is not checked here")

# SH-5 -----------------------------------------------------------------------------------------------------------------

func _t_sh5(_P: Vector2) -> void:
	var ws: WorldWorkspace = _app._world_workspace()
	_ok(not DccSettings.pen_tilt_to_size(), "tilt-to-brush defaults off")
	var real = ws.bridge
	var spy := SpyBridge.new()
	ws.bridge = spy
	ws._paint_stroke_open = true
	# tilt (0.866, 0) = a pen 60 degrees from vertical (sin 60 = 0.866)
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": Vector2(0.866, 0.0)}
	ws._paint_drag(10.0, 20.0)
	_ok(spy.paint_calls[0]["t"] == null, "tilt off: the engine receives no tilt (null)")
	_ok(spy.paint_calls[0]["p"] == 0.5, "tilt off: pressure is unaffected")
	DccSettings.set_pen_tilt_to_size(true)
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": 0.5, "tilt": Vector2(0.866, 0.0)}
	ws._paint_drag(10.0, 20.0)
	var t = spy.paint_calls[1]["t"]
	_ok(t is Vector2 and absf(t.x - 0.866) < 0.0001 and absf(t.y) < 0.0001, "tilt on: the engine receives (0.866, 0)")
	ws._paint_pending_sample = {"gx": 10.0, "gy": 20.0, "pressure": null, "tilt": null}
	ws._paint_drag(10.0, 20.0)
	_ok(spy.paint_calls[2]["t"] == null, "tilt on, no tilt reported: still null, never (0, 0)")
	ws._paint_stroke_open = false
	ws.bridge = real
	spy.free()
	var mn = _app.menus
	if mn != null and mn._stylus_popup != null:
		mn._on_stylus_choice(mn.ID_PREF_TILT)
		_ok(not DccSettings.pen_tilt_to_size(), "the Tilt row toggles it (on -> off)")
		mn._on_stylus_choice(mn.ID_PREF_TILT)
		_ok(DccSettings.pen_tilt_to_size() and mn._stylus_popup.is_item_checked(mn._stylus_popup.get_item_index(mn.ID_PREF_TILT)), "and back on, ticked")
	DccSettings.set_pen_tilt_to_size(false)

# SH-6 -----------------------------------------------------------------------------------------------------------------

func _crop(p: Vector2, half: int = 200) -> PackedByteArray:
	var img: Image = _vp.get_texture().get_image()
	var r := Rect2i(Vector2i(p) - Vector2i(half, half), Vector2i(half * 2, half * 2))
	r = r.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	return img.get_region(r).get_data()

## The same motion event with its position taken into the overlay's local space, for a direct `_gui_input` call
## (as `_ctxring_probe` drives the overlay): deterministic where a viewport push is subject to its hover routing.
func _local_motion(mm: InputEventMouseMotion) -> InputEventMouseMotion:
	var m: InputEventMouseMotion = mm.duplicate()
	m.position = _ov.get_global_transform().affine_inverse() * mm.position
	return m

func _t_sh6(P: Vector2) -> void:
	## Ruling EM item 3: the inner (minimum-pressure) ring exists only while biome paint's Pressure controls is Size
	## (its default is off, where there is no span to show), so this section runs under Size.
	DccSettings.set_pressure_control("paint", "size")
	var to: Control = _host.tool_overlay
	# a grid point under P, so the ring is drawn where the camera shows it
	var lp: Vector2 = _ov.get_global_transform().affine_inverse() * P
	var q: Dictionary = _ov._grid_point(lp, _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
	var gx := float(q["gx"])
	var gy := float(q["gy"])
	_ok(StylusInput.HOVER_RING_MS == 150, "the hover ring's lifetime is 150 ms (SH-6's own figure)")
	await _arm("paint")
	var r := 20.0
	to.set_brush_cursor(true, gx, gy, r)
	_ok(to.brush_inner_cells == 0.0 and to._brush_expire_ms == -1, "the old call shape: no inner ring, never expires")
	# each variant is compared with a single-ring picture taken just before it, so nothing else animating on the map
	# between two captures can pass for a ring
	var cases := [["an inner ring at 0.2 of the radius", r * 0.2, false],
		["an inner radius not smaller than the outer", r, true], ["an inner radius larger than the outer", r * 1.2, true], ["inner 0", 0.0, true]]
	for c in cases:
		to.set_brush_cursor(true, gx, gy, r)
		await _frames(3)
		var single := _crop(P)
		to.set_brush_cursor(true, gx, gy, r, float(c[1]))
		await _frames(3)
		var variant := _crop(P)
		to.set_brush_cursor(true, gx, gy, r)
		await _frames(3)
		var single2 := _crop(P)
		if bool(c[2]):
			_ok(variant == single or variant == single2, "%s draws nothing: pixel-identical to the single ring" % c[0])
		else:
			_ok(variant != single and variant != single2, "%s changes the picture" % c[0])
	to.set_brush_cursor(false)

	# through the workspace: a mouse gets the single permanent ring, a pen the pair that expires
	StylusInput.last_pen_motion_ms = -1
	var mouse_mm := _pen_event_motion(P + Vector2(2, 0), P, 1.0, Vector2.ZERO, false, false)
	_ov._gui_input(_local_motion(mouse_mm))
	await _frames(3)
	mouse_mm = _pen_event_motion(P + Vector2(4, 0), P + Vector2(2, 0), 1.0, Vector2.ZERO, false, false)
	_ov._gui_input(_local_motion(mouse_mm))
	await _frames(3)
	print("DBG mouse: vis ", to.brush_visible, " inner ", to.brush_inner_cells, " exp ", to._brush_expire_ms, " armed ", _app.armed_tool, " r ", to.brush_radius_cells, " recent ", StylusInput.pen_recent(150), " last ", StylusInput.last_pen_motion_ms)
	_ok(to.brush_visible and to.brush_inner_cells == 0.0 and to._brush_expire_ms == -1, "mouse hover: one ring, no inner, no expiry")
	await get_tree().create_timer(0.4).timeout
	_ok(to.brush_visible, "the mouse's ring stays")
	var pen_mm := _pen_event_motion(P + Vector2(3, 0), P, 0.5, Vector2(0.1, 0.0), false, false)
	_ov._gui_input(_local_motion(pen_mm))
	await _frames(3)
	_ok(to.brush_visible and absf(to.brush_inner_cells - to.brush_radius_cells * 0.2) < 0.001 and to._brush_expire_ms >= 0,
		"pen hover: an inner ring at 0.2 of the radius and an expiry (inner %.3f of %.3f)" % [to.brush_inner_cells, to.brush_radius_cells])
	await get_tree().create_timer(0.4).timeout
	_ok(not to.brush_visible, "the pen's ring is gone 400 ms after the last hover")
	to.set_brush_cursor(false)
	# lifetime
	to.set_brush_cursor(true, gx, gy, r, r * 0.2, 150)
	await get_tree().create_timer(0.05).timeout
	_ok(to.brush_visible, "a pen-hover ring is still there 50 ms after its last sample")
	await get_tree().create_timer(0.4).timeout
	_ok(not to.brush_visible, "and gone well after 150 ms")
	to.set_brush_cursor(true, gx, gy, r)
	await get_tree().create_timer(0.4).timeout
	_ok(to.brush_visible, "a ring with no expiry (the mouse's) is still there after 400 ms")
	# a held button never lets it blink off mid-stroke
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	Input.parse_input_event(press)
	to.set_brush_cursor(true, gx, gy, r, r * 0.2, 150)
	await get_tree().create_timer(0.4).timeout
	_ok(to.brush_visible, "with the button held the ring does not expire (a stroke is not a hover)")
	var rel := InputEventMouseButton.new()
	rel.button_index = MOUSE_BUTTON_LEFT
	rel.pressed = false
	Input.parse_input_event(rel)
	await get_tree().create_timer(0.5).timeout
	_ok(not to.brush_visible, "released, the ring expires")

	StylusInput.last_pen_motion_ms = -1
	to.set_brush_cursor(false)

# SH-9 -----------------------------------------------------------------------------------------------------------------

func _t_sh9(_P: Vector2) -> void:
	var t := StylusTester.new()
	_vp.add_child(t)
	await _frames(2)
	var s := t.snapshot()
	_ok(s["pressure"] == "—" and s["tilt"] == "—" and s["pressure span"] == "—", "before any event every row is — (nothing is shown as a number)")
	# SYNTHETIC pen contact: pressure 0.4, tilt (0.5, 0) = 30 degrees
	t.feed("MouseMotion", 0, Vector2(50, 50), 0.4, Vector2(0.5, 0.0), false, true, true)
	s = t.snapshot()
	_ok(s["pressure"] == "0.4000" and s["source"].begins_with("pen"), "a pen at 0.4: pressure 0.4000, source pen (%s | %s)" % [s["pressure"], s["source"]])
	_ok(s["tilt"] == "(0.500, 0.000)" and s["tilt angle"] == "30.0 deg from vertical", "tilt (0.5, 0) reads 30.0 degrees (%s | %s)" % [s["tilt"], s["tilt angle"]])
	t.feed("MouseMotion", 0, Vector2(52, 50), 0.8, Vector2(0.5, 0.0), false, true, true)
	_ok(t.snapshot()["pressure span"] == "0.4000 .. 0.8000", "the span widens to 0.4000 .. 0.8000 (%s)" % t.snapshot()["pressure span"])
	t.feed("MouseMotion", 0, Vector2(54, 50), 0.0, Vector2.ZERO, false, false, true)
	s = t.snapshot()
	_ok(s["pressure"].begins_with("—") and s["source"].begins_with("pointer hovering") and s["hover"].begins_with("hovering"),
		"a hover with no fields reads pressure —, hovering (%s | %s)" % [s["pressure"], s["source"]])
	_ok(s["tilt"] == "—", "a hover's tilt is — (none reported), not (0, 0)")
	t.feed("MouseMotion", 0, Vector2(56, 50), 1.0, Vector2.ZERO, false, true, true)
	s = t.snapshot()
	_ok(s["source"].begins_with("mouse-like") and s["pressure"].contains("mouse-like"), "a held flat 1.0 is labelled mouse-like, not a pen (%s)" % s["pressure"])
	_ok(t.snapshot()["pressure span"] == "0.4000 .. 0.8000", "a mouse-like 1.0 does not enter the pen's span")
	t.feed("MouseMotion", -1, Vector2(58, 50), 0.3, Vector2.ZERO, false, true, true)
	s = t.snapshot()
	_ok(s["source"].begins_with("finger") and s["pressure"].contains("finger's own"), "a finger's 0.3 is labelled the finger's own (%s)" % s["pressure"])
	t.feed("ScreenTouch#0", -1, Vector2(58, 50), 0.0, Vector2.ZERO, false, true, false)
	s = t.snapshot()
	_ok(s["pressure"] == "—" and s["tilt"] == "—", "a touch event carries no fields: pressure and tilt are —")
	t.feed("MouseMotion", 0, Vector2(60, 50), 0.5, Vector2.ZERO, true, true, true)
	_ok(t.snapshot()["eraser flag"].begins_with("yes"), "the eraser flag reads yes when set")
	t.reset_observed()
	_ok(t.snapshot()["pressure span"] == "—", "Reset clears the span")
	# the real event path: _gui_input on a motion event
	var mm := _pen_event_motion(Vector2(70, 70), Vector2(68, 70), 0.6, Vector2.ZERO, false, true)
	t._gui_input(mm)
	_ok(is_equal_approx(float(t.snapshot()["pressure"]), 0.6) and t.snapshot()["event"] == "MouseMotion", "_gui_input reads a motion event's pressure")
	t.queue_free()
	await _frames(2)
	var mn = _app.menus
	if mn != null and mn._stylus_popup != null:
		var before := _vp.find_children("*", "StylusTester", true, false).size()
		mn._on_stylus_choice(mn.ID_PREF_TESTER)
		await _frames(3)
		var found := _vp.find_children("*", "StylusTester", true, false)
		_ok(found.size() == before + 1, "Preferences > Stylus > Stylus tester… opens the tester")
		for w in _vp.find_children("*", "Window", true, false):
			if (w as Window).title == "Stylus tester":
				w.queue_free()
		await _frames(3)
