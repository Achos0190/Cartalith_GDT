extends Node
## MS-5 probe (`MOBILE_SHELL_PROPOSAL.md` section 2.5 items 1 to 6, section 6 MS-5). Drives REAL touch events
## (InputEventScreenTouch / ScreenDrag with their emulated mouse button and motion, `device = -1` as Godot's
## emulation stamps them, and a second finger that has no mouse of its own) against the real shell in a
## `--vp`-sized host SubViewport, with a small generated world so the overlay has a displayed rect.
##
## What it checks, each with literals written by hand (nothing is asserted against the constant it came from):
##   1. the map row's ORDER from the dominant hand's edge (right-handed: latch, `-` `+`, recentre, left to right;
##      left-handed the mirror), that the latch's arrival moves the recentre by less than 2 px, and that no 3D/2D
##      pill is left in the row;
##   2. the latch's visibility by tool (Inspect, Measure, Settlement: hidden; Sculpt, Paint, Region, Way, an unknown
##      id: shown) and that arming or disarming clears the latch;
##   3. the one-finger pan: with Inspect or a placing tool a 120 px drag moves the camera by 120 px and emits NO
##      click, selection or release; a plain tap still clicks (positive control); with a drawing tool the same
##      drag draws (map_dragged, camera unmoved), and under the latch it pans;
##   4. the stray-stroke guard: a second finger lands during a stroke, before the slop, and after a pinch's lift;
##   5. the double-tap (x1.7 about the finger, only with Inspect or the latch, no card, nothing outside 300 ms);
##   6. thumb zoom (down zooms in, up out, one doubling per 0.15 of the viewport height, no card however long the
##      second press is held) and that a mouse (device 0) is never routed.
##
## Windowed, with a wiped user dir (the probe writes the dominant hand). This worktree's `config/name` is changed
## to a scratch name for the run and restored after; never stage `project.godot`:
##   godot --path . --resolution 1600x1000 _ms5_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
##   (also 2340x1080 @2.8125, 1080x2340 @2.625, 2340x1080 @2.625, 1440x3120 @3, 2388x1668 @2, 1080x1920 @3 = 360 dp,
##    and 1600x1000 --force-touch with no --density for the touch tablet)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
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
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	DccSettings.set_dominant_hand("right")
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

# ---- the run ---------------------------------------------------------------------------------------------------------

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
	DccSettings.set_dominant_hand("right")
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
	var phone: bool = DccTheme.is_phone()
	if phone:
		_app._pick_phone_tab("map")
		_app.set_phone_detent("peek")
	await _settle()
	_ok(_host._touch, "the host is a touch build (--force-touch)")
	_ok(_host._navpad != null, "the touch row exists")
	_wire()
	_host.reset_view()
	await _settle()
	var P := _find_point()
	_ok(P.x >= 0.0, "a valid map point was found, %s" % str(P))
	if P.x < 0.0 or _host._navpad == null:
		return

	await _t_order()
	await _t_latch_by_tool()
	await _t_one_finger_pan(P)
	await _t_stray(P)
	await _t_double_tap(P)
	await _t_thumb(P)
	await _t_mouse(P)

# 1. the row's order ---------------------------------------------------------------------------------------------

func _t_order() -> void:
	DccSettings.set_dominant_hand("right")
	await _arm("sculpt")
	await _settle()
	var b := _btn()
	var pan: Button = b["pan"]
	var rocker: Control = b["rocker"]
	var rec: Button = b["rec"]
	_ok(pan.visible, "right hand, Sculpt armed: the latch is drawn")
	var xp := pan.get_global_rect().position.x
	var xr := rocker.get_global_rect().position.x
	var xc := rec.get_global_rect().position.x
	_ok(xp < xr and xr < xc, "right hand: latch, rocker, recentre from left to right (%.0f %.0f %.0f)" % [xp, xr, xc])
	var vr := _host.get_global_rect()
	_ok(absf(rec.get_global_rect().end.x - (vr.end.x - maxf(float(_host._safe_insets.get("right", 10.0)), 12.0))) <= 2.0,
		"right hand: the recentre is the outermost member, flush with the right edge inset")
	var rec_with := rec.get_global_rect()
	await _arm("inspect")
	await _settle()
	_ok(not pan.visible, "Inspect armed: the latch is hidden")
	var rec_without := rec.get_global_rect()
	## MS-5 second step: HORIZONTAL distance. The tool strip is taller with Sculpt armed (its option and verb row)
	## than with Inspect, and the row sits above the strip, so the whole row now lifts by that height; what the latch
	## must not do is push the recentre sideways, which is what "where the thumb learned it" means.
	_ok(absf(rec_with.position.x - rec_without.position.x) < 2.0,
		"the latch's arrival moves the recentre sideways by %.2f px (< 2)" % absf(rec_with.position.x - rec_without.position.x))
	var pills: Array = _host._navpad.find_children("", "Button", true, false).filter(func(x): return x.text == "3D" or x.text == "2D")
	_ok(pills.is_empty(), "no 3D / 2D pill is left in the row")
	DccSettings.set_dominant_hand("left")
	await _arm("sculpt")
	await _settle()
	xp = pan.get_global_rect().position.x
	xr = rocker.get_global_rect().position.x
	xc = rec.get_global_rect().position.x
	_ok(xc < xr and xr < xp, "left hand: recentre, rocker, latch from left to right (%.0f %.0f %.0f)" % [xc, xr, xp])
	_ok(rec.get_global_rect().position.x >= vr.position.x + 11.0, "left hand: the row clears the left edge by the inset")
	var rec_left_with := rec.get_global_rect()
	await _arm("inspect")
	await _settle()
	_ok(absf(rec_left_with.position.x - rec.get_global_rect().position.x) < 2.0,
		"left hand: the latch's arrival moves the recentre sideways by less than 2 px (the strip's row lifts it; see above)")
	_ok(_host._coords_label.get_global_rect().position.x < vr.position.x + vr.size.x * 0.5,
		"left hand: the coordinate readout moved to the row's side")
	DccSettings.set_dominant_hand("right")
	await _settle()
	_ok(_host._coords_label.get_global_rect().position.x > vr.position.x + vr.size.x * 0.5,
		"right hand again: the readout is back on the right")

# 2. the latch follows the tool ----------------------------------------------------------------------------------------

func _t_latch_by_tool() -> void:
	var pan: Button = _host._pan_btn
	for id in ["inspect", "measure", "settlement", "label", "icon", "conflict", "route"]:
		await _arm(id)
		_ok(not pan.visible, "%s armed: the latch is hidden" % id)
	for id in ["sculpt", "paint", "region", "way", "territory", "colour_paint", "no_such_tool"]:
		await _arm(id)
		_ok(pan.visible, "%s armed: the latch is drawn" % id)
	await _arm("sculpt")
	_host.set_pan_mode(true)
	_ok(_host.pan_mode(), "the latch turns on")
	await _arm("paint")
	_ok(not _host.pan_mode(), "arming another tool clears the latch")
	_host.set_pan_mode(true)
	await _arm("inspect")
	_ok(not _host.pan_mode(), "disarming to Inspect clears the latch")

# 3. the one-finger pan --------------------------------------------------------------------------------------------------

func _t_one_finger_pan(P: Vector2) -> void:
	for id in ["inspect", "settlement", "measure", "route"]:
		await _arm(id)
		_zero()
		var c0 := _cam()
		await _tap(P)
		await get_tree().create_timer(0.45).timeout
		_ok(_n["click"] == 1, "%s: a plain tap clicks once (positive control), got %d" % [id, _n["click"]])
		await _dismiss_windows()
		_zero()
		c0 = _cam()
		await _down(0, P)
		await _drag(P, P + Vector2(120, 0))
		await _up(0, P + Vector2(120, 0))
		var d := _cam() - c0
		_ok(absf(d.x - 120.0) <= 1.0 and absf(d.y) <= 1.0, "%s: a 120 px drag pans the camera by 120 px, got %s" % [id, str(d)])
		_ok(_n["click"] == 0 and _n["sel"] == 0 and _n["rel"] == 0 and _n["drag"] == 0,
			"%s: the pan emits no click, selection, drag or release (%s)" % [id, str(_n)])
		## The dropped press must not survive to fire the hold: a long wait after the lift opens no card and
		## leaves the overlay with nothing armed (the hold is 480 ms phone, 500 ms tablet).
		await get_tree().create_timer(0.7).timeout
		_ok(not _ov._touch_armed and not _ov._touch_ring_active and _n["ctx"] == 0,
			"%s: after the pan nothing is armed and no card opened (armed %s, ctx %d)" % [id, str(_ov._touch_armed), _n["ctx"]])
		## Back to the fitted view: each pan moves the world 120 px, and on a form whose map is narrower than
		## the world's cover (the tablet) four of them would leave P over bare paper, where no tool clicks.
		_host.reset_view()
		await _settle()
	# a drag that stays inside the slop is a tap, not a pan
	await _arm("inspect")
	_zero()
	var cs := _cam()
	await _down(0, P)
	await _move(0, P + Vector2(10, 0), P)
	await _up(0, P + Vector2(10, 0))
	_ok(_cam().distance_to(cs) < 0.01 and _n["click"] == 1, "a 10 px wobble is a tap: no pan, one click (%s)" % str(_n))
	await get_tree().create_timer(0.45).timeout
	# drawing tools keep the finger
	for id in ["sculpt", "paint", "way", "region"]:
		await _arm(id)
		_zero()
		var c1 := _cam()
		await _down(0, P)
		await _drag(P, P + Vector2(120, 0))
		await _up(0, P + Vector2(120, 0))
		_ok(_cam().distance_to(c1) < 0.01, "%s: the finger draws, the camera does not move" % id)
		_ok(_n["drag"] >= 1 and _n["click"] == 1 and _n["rel"] == 1,
			"%s: the stroke reaches the tool (click 1, drag >= 1, release 1) %s" % [id, str(_n)])
		_host.reset_view()
		await _settle()
	# under the latch the same drag pans
	await _arm("sculpt")
	_host.set_pan_mode(true)
	_zero()
	var c2 := _cam()
	await _down(0, P)
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	var d2 := _cam() - c2
	_ok(absf(d2.x - 120.0) <= 1.0, "Sculpt + latch: the drag pans by 120 px, got %s" % str(d2))
	_ok(_n["drag"] == 0 and _n["click"] == 0, "Sculpt + latch: nothing reaches the tool (%s)" % str(_n))
	_host.set_pan_mode(false)
	_host.reset_view()
	await _settle()
	# a hold, then a drag, is the card's: no pan
	await _arm("inspect")
	_zero()
	var c3 := _cam()
	await _down(0, P)
	await get_tree().create_timer(0.75).timeout
	await _drag(P, P + Vector2(120, 0))
	await _up(0, P + Vector2(120, 0))
	_ok(_cam().distance_to(c3) < 0.01, "a drag after the hold has fired does not pan (the card owns the finger)")
	await _close_card()

# 4. the stray-stroke guard --------------------------------------------------------------------------------------------

func _t_stray(P: Vector2) -> void:
	_host.reset_view()
	await _settle()
	await _arm("sculpt")
	var r := _host.get_global_rect()
	var Q := Vector2(r.position.x + r.size.x * 0.25, r.position.y + r.size.y * 0.25)
	# (a) a second finger lands DURING a stroke: the stroke is closed once, then nothing more reaches the tool
	_zero()
	await _down(0, P)
	await _drag(P, P + Vector2(60, 0), 3)
	_ok(_n["click"] == 1 and _n["drag"] >= 1 and _n["rel"] == 0, "a stroke is under way (%s)" % str(_n))
	var drags_before: int = _n["drag"]
	await _down(1, Q)
	_ok(_n["rel"] == 1, "the second finger closes the stroke: exactly one release (%s)" % str(_n))
	await _move(0, P + Vector2(80, 0), P + Vector2(60, 0))
	await _move(0, P + Vector2(100, 0), P + Vector2(80, 0))
	_ok(_n["drag"] == drags_before, "the first finger's further motion draws nothing (%d -> %d)" % [drags_before, _n["drag"]])
	await _up(0, P + Vector2(100, 0))
	_ok(_n["rel"] == 1, "the first finger's lift is not a second release (%s)" % str(_n))
	await _move(1, Q + Vector2(10, 10), Q)
	await _up(1, Q + Vector2(10, 10))
	_zero()
	await _tap(P)
	_ok(_n["click"] == 1, "after every finger is up a new touch draws again (%s)" % str(_n))
	# (b) a second finger lands BEFORE the slop: the withheld press is dropped, no click ever
	_zero()
	await _down(0, P)
	_ok(_ov._touch_armed, "(b) the first finger's press is withheld by the overlay")
	await _down(1, Q)
	_ok(not _ov._touch_armed, "(b) the second finger discards the withheld press at once")
	await _drag(P, P + Vector2(80, 0), 3)
	await _up(0, P + Vector2(80, 0))
	await _up(1, Q)
	await get_tree().create_timer(0.1).timeout
	_ok(_n["click"] == 0 and _n["drag"] == 0 and _n["rel"] == 0, "a pinch before the slop reaches the tool as nothing (%s)" % str(_n))
	# (c) a pinch, one finger lifts, the other moves: no stray stroke
	_zero()
	await _down(0, P)
	await _down(1, Q)
	await _up(1, Q)
	await _drag(P, P + Vector2(90, 0), 3)
	await _up(0, P + Vector2(90, 0))
	_ok(_n["click"] == 0 and _n["drag"] == 0 and _n["rel"] == 0, "lifting one finger after a pinch draws no stroke (%s)" % str(_n))
	# (d) the first finger lifts first, then it is re-pressed while the second is still down: ignored
	_zero()
	await _down(0, P)
	await _down(1, Q)
	await _up(0, P)
	await _down(0, P)
	await _drag(P, P + Vector2(90, 0), 3)
	await _up(0, P + Vector2(90, 0))
	await _up(1, Q)
	_ok(_n["click"] == 0 and _n["drag"] == 0, "a re-press while another finger is down is not a stroke (%s)" % str(_n))
	# (e) a second finger on the ROW (not the map) cancels nothing
	_zero()
	var rocker_pt := _centre(_host._rocker)
	await _down(0, P)
	await _drag(P, P + Vector2(60, 0), 3)
	var d_before: int = _n["drag"]
	await _down(1, rocker_pt)
	await _move(0, P + Vector2(90, 0), P + Vector2(60, 0))
	_ok(_n["rel"] == 0 and _n["drag"] > d_before, "a second finger on the row does not end the stroke (%s)" % str(_n))
	await _up(1, rocker_pt)
	await _up(0, P + Vector2(90, 0))
	_ok(_n["rel"] == 1, "the stroke still ends on its own lift (%s)" % str(_n))
	# (f) with a tap-only tool the second finger drops a withheld tap
	await _arm("inspect")
	_zero()
	await _down(0, P)
	await _down(1, Q)
	await _up(0, P)
	await _up(1, Q)
	await get_tree().create_timer(0.1).timeout
	_ok(_n["click"] == 0, "Inspect: a two-finger touch never taps (%s)" % str(_n))

# 5. double-tap ---------------------------------------------------------------------------------------------------------

## The map-local point under `s` (screen) at the current camera, so a zoom's pivot can be checked.
func _local_at(s: Vector2) -> Vector2:
	return (s - _host.global_position - _cam()) / _zoom()

func _t_double_tap(P: Vector2) -> void:
	await _arm("inspect")
	_host.reset_view()
	await _settle()
	var z0 := _zoom()
	var l0 := _local_at(P)
	_zero()
	await _tap(P)
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.7) <= 0.02 * 1.7, "Inspect: a double-tap zooms x1.7 (ratio %.3f)" % (_zoom() / z0))
	_ok(_local_at(P).distance_to(l0) < 1.0, "the double-tap zooms about the finger (pivot drift %.2f px)" % _local_at(P).distance_to(l0))
	_ok(_n["ctx"] == 0, "the double-tap opens no card")
	_ok(_n["click"] == 1, "the first tap still acted, the second did not (click %d)" % _n["click"])
	# outside 300 ms: two taps
	_host.reset_view()
	await _settle()
	z0 = _zoom()
	await _tap(P)
	await get_tree().create_timer(0.45).timeout
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.0) < 0.001, "two taps 450 ms apart do not zoom (ratio %.3f)" % (_zoom() / z0))
	# too far apart (60 px, past the 34 px figure)
	await _tap(P)
	await _tap(P + Vector2(60, 0))
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.0) < 0.001, "two taps 60 px apart do not zoom (ratio %.3f)" % (_zoom() / z0))
	# placing tool: off
	await _arm("settlement")
	z0 = _zoom()
	_zero()
	await _tap(P)
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.0) < 0.001 and _n["click"] == 2, "Settlement: a double-tap is two placements, no zoom (click %d, ratio %.3f)" % [_n["click"], _zoom() / z0])
	# drawing tool, no latch: off
	await _arm("sculpt")
	z0 = _zoom()
	_zero()
	await _tap(P)
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.0) < 0.001 and _n["click"] == 2, "Sculpt without the latch: two taps, no zoom (click %d)" % _n["click"])
	# drawing tool with the latch: on
	_host.set_pan_mode(true)
	z0 = _zoom()
	_zero()
	await _tap(P)
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.7) <= 0.02 * 1.7 and _n["click"] == 0, "Sculpt + latch: a double-tap zooms x1.7 (ratio %.3f), the tool is not touched" % (_zoom() / z0))
	_host.set_pan_mode(false)
	# a second finger kills a pending double-tap
	await _arm("inspect")
	_host.reset_view()
	await _settle()
	z0 = _zoom()
	var r := _host.get_global_rect()
	var Q := Vector2(r.position.x + r.size.x * 0.25, r.position.y + r.size.y * 0.25)
	await _tap(P)
	await _down(0, P)
	await _down(1, Q)
	await _up(0, P)
	await _up(1, Q)
	await _tap(P)
	await get_tree().create_timer(0.5).timeout
	_ok(absf(_zoom() / z0 - 1.0) < 0.001, "a two-finger touch right after a tap cancels the double-tap (ratio %.3f)" % (_zoom() / z0))

# 6. thumb zoom ---------------------------------------------------------------------------------------------------------

func _t_thumb(P: Vector2) -> void:
	await _arm("inspect")
	_host.reset_view()
	await _settle()
	var z0 := _zoom()
	var l0 := _local_at(P)
	var h := _host.size.y
	_zero()
	await _tap(P)
	await _down(0, P)
	# held far past the card's 480/500 ms, then slid down by 0.15 of the viewport height: one doubling
	await get_tree().create_timer(0.7).timeout
	# the first 30 px (past the 28 px slop) start the slide and are not zoomed; the zoom counts from there
	var S := P + Vector2(0, 30)
	await _move(0, S, P)
	_ok(absf(_zoom() / z0 - 1.0) < 0.001, "thumb zoom: the slop's own 30 px do not zoom (ratio %.3f)" % (_zoom() / z0))
	await _drag(S, S + Vector2(0, 0.15 * h), 6)
	_ok(absf(_zoom() / z0 - 2.0) <= 0.04, "thumb zoom: sliding DOWN 0.15 of the height zooms in x2 (ratio %.3f)" % (_zoom() / z0))
	_ok(_local_at(P).distance_to(l0) < 1.0, "thumb zoom pivots on the second press (drift %.2f px)" % _local_at(P).distance_to(l0))
	await _drag(S + Vector2(0, 0.15 * h), S, 6)
	_ok(absf(_zoom() / z0 - 1.0) <= 0.03, "thumb zoom: sliding back UP zooms out again (ratio %.3f)" % (_zoom() / z0))
	await _drag(S, S - Vector2(0, 0.15 * h * 0.5), 4)
	_ok(_zoom() / z0 < 0.8, "thumb zoom: sliding UP past the start zooms out (ratio %.3f)" % (_zoom() / z0))
	await _up(0, S - Vector2(0, 0.15 * h * 0.5))
	await get_tree().create_timer(0.3).timeout
	_ok(_n["ctx"] == 0, "the held second press opened no card")
	_ok(_n["click"] == 1, "thumb zoom adds no click (click %d)" % _n["click"])
	var z1 := _zoom()
	await get_tree().create_timer(0.4).timeout
	_ok(absf(_zoom() - z1) < 0.0001, "lifting ends the zoom: nothing animates afterwards")
	# a press with no tap before it still opens the card at the hold
	_zero()
	await _down(0, P)
	await get_tree().create_timer(0.75).timeout
	await _up(0, P)
	if DccTheme.is_tablet():
		## The tablet's hold opens the ring and card together through `_ring_touch_open_cb` and emits no
		## `context_requested` (`map_overlay.gd::_process()`), so its sign is the tracked ring.
		_ok(_n["ctx"] == 0, "a plain hold on the tablet opens the ring, not the phone's card (ctx %d)" % _n["ctx"])
	else:
		_ok(_n["ctx"] >= 1, "a plain hold still opens the card (ctx %d)" % _n["ctx"])
	await _close_card()
	# a tap-hold-slide with a drawing tool and no latch is a stroke, not a zoom
	await _arm("sculpt")
	_host.reset_view()
	await _settle()
	z0 = _zoom()
	await _tap(P)
	await _down(0, P)
	await _drag(P, P + Vector2(0, 0.15 * h), 4)
	await _up(0, P + Vector2(0, 0.15 * h))
	_ok(absf(_zoom() / z0 - 1.0) < 0.001, "Sculpt without the latch: tap-hold-slide does not zoom (ratio %.3f)" % (_zoom() / z0))

# 7. a mouse is not a finger ----------------------------------------------------------------------------------------------

func _t_mouse(P: Vector2) -> void:
	await _arm("inspect")
	_zero()
	var c0 := _cam()
	await _mouse_drag(P, P + Vector2(120, 0))
	_ok(_cam().distance_to(c0) < 0.01, "a real mouse (device 0) dragging with Inspect does not pan (a pen draws)")
