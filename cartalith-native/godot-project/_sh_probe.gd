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
