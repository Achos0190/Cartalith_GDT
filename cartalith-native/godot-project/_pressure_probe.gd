extends Node
## Pressure-controls probe (Ruling EM item 3). Drives SYNTHETIC input only: a pen sample is an `InputEventMouseMotion`
## with `device = 0` and a hand-set `pressure` (what Godot's Android port stamps on a pen); the mouse is a device-0
## button+motion pair carrying Godot's flat 1.0 (which the shell must read as NO pressure). No real pen was used:
## this proves the mapping rules, NOT what the Pad 3's pen reports. Literals are written by hand.
##
## What it proves, against the REAL engine (no stub): per tool kind, `DccSettings.pressure_control` decides what a pen's
## pressure does. Sculpt strength puts the pen's pressure on the stamp and leaves the stamp's params (radius) alone;
## sculpt off puts none; a mouse puts none. Biome paint off ignores pressure (same painted area at 0.1 and 1.0), size
## shrinks the dab at low pressure, and a mouse paints the full-radius area. Colour paint offers only off.
##
## Windowed, wiped user dir, under the shared lock (`C:/cw/.win_lock`):
##   godot --path . --resolution 1600x1000 _pressure_probe.tscn -- --vp 1600x1000
##   (also --vp 2388x1668 --density 2 --force-touch, and --vp 1080x2340 --density 2.8125 --force-touch)
## Arguments: `--vp WxH`; accepted because the shell reads them: `--density D`, `--force-touch`.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _host: Control
var _ov: Control

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

func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()

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

func _arm(id: String) -> void:
	_app.arm_tool(id)
	await _frames(4)

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
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
		if a == "--force-touch":
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
	StylusInput.assume_mobile = false
	StylusInput.end_pen_contact()
	get_tree().quit(0 if _fails == 0 else 1)

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
	_host.reset_view()
	await _settle()
	var P := _find_point()
	_ok(P.x >= 0.0, "a valid map point was found, %s" % str(P))
	if P.x < 0.0:
		return
	_t_settings()
	_t_tester()
	await _t_controls()
	await _t_sculpt(P)
	await _t_paint(P)

## A mouse stroke (device 0, button + motion): Godot reports pressure 1.0 for it, which must read as no pressure.
func _mouse_stroke(a: Vector2, b: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.device = 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = a
	m.global_position = a
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)
	var cur := a
	for s in 6:
		var nxt := a.lerp(b, float(s + 1) / 6.0)
		var mm := InputEventMouseMotion.new()
		mm.device = 0
		mm.position = nxt
		mm.global_position = nxt
		mm.relative = nxt - cur
		mm.pressure = 1.0
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm)
		await _frames(2)
		cur = nxt
	var r := InputEventMouseButton.new()
	r.device = 0
	r.button_index = MOUSE_BUTTON_LEFT
	r.pressed = false
	r.position = b
	r.global_position = b
	_vp.push_input(r)
	await _frames(3)

func _t_settings() -> void:
	## The shipped defaults, read before anything is set (the harness wipes the user dir): Ruling EM item 3.
	_ok(DccSettings.pressure_control("sculpt") == "strength", "default: sculpt = strength")
	_ok(DccSettings.pressure_control("paint") == "off", "default: biome paint = off (the owner: area from the slider)")
	_ok(DccSettings.pressure_control("colour") == "off", "default: colour paint = off")
	DccSettings.set_pressure_control("sculpt", "strength")
	DccSettings.set_pressure_control("paint", "off")
	_ok(DccSettings.pressure_control("sculpt") == "strength", "sculpt reads strength once set to strength")
	_ok(DccSettings.pressure_control("paint") == "off", "paint reads off")
	_ok(DccSettings.pressure_control("colour") == "off", "colour paint reads off")
	_ok(not DccSettings.set_pressure_control("paint", "strength"), "paint refuses strength (categorical)")
	_ok(not DccSettings.set_pressure_control("sculpt", "size"), "sculpt refuses size (one radius per stamp)")
	_ok(not DccSettings.set_pressure_control("colour", "size"), "colour refuses size")
	_ok(not DccSettings.set_pressure_control("colour", "strength"), "colour refuses strength")
	_ok(not DccSettings.set_pressure_control("bogus", "off"), "an unknown kind is refused")
	_ok(DccSettings.pressure_control("paint") == "off" and DccSettings.pressure_control("sculpt") == "strength", "a refused write stores nothing")
	_ok(DccSettings.pressure_why_not("paint", "strength") != "" and DccSettings.pressure_why_not("paint", "size") == "", "why-not text only for an unoffered mapping")
	DccSettings.set_pressure_control("paint", "size")
	_ok(DccSettings.pressure_control("paint") == "size", "paint size round-trips")
	DccSettings.set_pressure_control("paint", "off")

func _t_tester() -> void:
	DccSettings.set_pressure_control("sculpt", "strength")
	DccSettings.set_pressure_control("paint", "size")
	DccSettings.set_pen_pressure_curve("linear")
	DccSettings.set_pen_pressure_max(1.0)
	_ok(DccSettings.pressure_effect_text("sculpt", 0.5) == "strength x 0.500", "tester: sculpt strength at 0.5 is x 0.500")
	_ok(DccSettings.pressure_effect_text("paint", 0.5) == "size x 0.600", "tester: paint size at 0.5 is 0.2 + 0.8 x 0.5 = x 0.600")
	_ok(DccSettings.pressure_effect_text("paint", 0.0) == "size x 0.200", "tester: paint size at zero pressure is the 0.2 floor (zero is a reading)")
	_ok(DccSettings.pressure_effect_text("sculpt", -1.0) == "no pen pressure: stroke unchanged", "tester: no pen pressure never yields a multiplier")
	_ok(DccSettings.pressure_effect_text("colour", 0.5) == "pressure ignored", "tester: colour paint ignores pressure")
	DccSettings.set_pressure_control("paint", "off")
	_ok(DccSettings.pressure_effect_text("paint", 0.5) == "pressure ignored", "tester: paint off is ignored")
	DccSettings.set_pen_pressure_curve("firm")
	_ok(DccSettings.pressure_effect_text("sculpt", 0.5) == "strength x 0.330", "tester: firm curve applies (0.5^1.6 = 0.330)")
	DccSettings.set_pen_pressure_curve("linear")
	var t := StylusTester.new()
	add_child(t)
	DccSettings.set_pressure_control("paint", "size")
	t.feed("MouseMotion", 0, Vector2(5, 5), 0.5, Vector2(0.2, 0.0), false, true, true)
	var s: Dictionary = t.snapshot()
	_ok(String(s.get("sculpt maps", "")) == "strength x 0.500" and String(s.get("paint maps", "")) == "size x 0.600",
		"the tester row shows the effective mapping for a pen reading (%s | %s)" % [s.get("sculpt maps"), s.get("paint maps")])
	t.feed("MouseMotion", 0, Vector2(5, 5), 1.0, Vector2.ZERO, false, true, true)
	s = t.snapshot()
	_ok(String(s.get("sculpt maps", "")) == "no pen pressure: stroke unchanged", "a mouse-like flat 1.0 reads as no pen pressure in the tester (%s)" % s.get("sculpt maps"))
	t.queue_free()
	DccSettings.set_pressure_control("paint", "off")

## The bar's Pressure controls dropdown, built on the live form factor, lists only what the kind offers and writes the
## per-install setting.
func _t_controls() -> void:
	var box := VBoxContainer.new()
	_vp.add_child(box)
	var sc: OptionButton = DccToolBar.build_pressure_control(box, _app, "sculpt")
	var pa: OptionButton = DccToolBar.build_pressure_control(box, _app, "paint")
	_ok(sc.item_count == 2 and sc.get_item_text(0) == "Strength" and sc.get_item_text(1) == "Off", "sculpt dropdown offers Strength, Off")
	_ok(pa.item_count == 2 and pa.get_item_text(0) == "Size" and pa.get_item_text(1) == "Off", "paint dropdown offers Size, Off")
	_ok(sc.selected == 0 and pa.selected == 1, "dropdowns show the stored choices: sculpt Strength, paint Off")
	pa.item_selected.emit(0)
	_ok(DccSettings.pressure_control("paint") == "size", "choosing Size in the paint dropdown stores size")
	pa.item_selected.emit(1)
	sc.item_selected.emit(1)
	_ok(DccSettings.pressure_control("sculpt") == "off" and DccSettings.pressure_control("paint") == "off", "choosing Off stores off")
	sc.item_selected.emit(0)
	box.queue_free()
	await _frames(2)
	var mn = _app.menus
	if mn != null and mn._stylus_popup != null:
		var pop: PopupMenu = mn._stylus_popup
		mn._on_stylus_choice(mn.ID_PRESSURE_CTRL_BASE + 3)
		_ok(DccSettings.pressure_control("paint") == "size" and pop.is_item_checked(pop.get_item_index(mn.ID_PRESSURE_CTRL_BASE + 3))
			and not pop.is_item_checked(pop.get_item_index(mn.ID_PRESSURE_CTRL_BASE + 4)), "Preferences > Stylus > Paint: pressure sets size stores size and ticks it alone")
		mn._on_stylus_choice(mn.ID_PRESSURE_CTRL_BASE + 4)
		_ok(DccSettings.pressure_control("paint") == "off", "Preferences > Stylus > Paint: pressure ignored stores off")
	else:
		print("note: no PC/tablet Preferences menu on this form")

func _last_stamp() -> Dictionary:
	var l: Array = _app.bridge.sculpt_list_stamps()
	return (l[0] as Dictionary) if l.size() > 0 else {}

func _t_sculpt(P: Vector2) -> void:
	var br = _app.bridge
	await _arm("sculpt")
	br.sculpt_discard()
	var a := P
	var b := P + Vector2(70.0, 0.0)
	DccSettings.set_pen_pressure_curve("linear")
	DccSettings.set_pen_pressure_max(1.0)
	DccSettings.set_pressure_control("sculpt", "strength")
	await _pen_stroke(a, b, 0.3)
	var lo := _last_stamp()
	_ok(not lo.is_empty() and lo.has("pressure"), "sculpt strength: a pen stroke leaves a stamp carrying its pressure")
	var lo_p: PackedFloat32Array = lo.get("pressure", PackedFloat32Array())
	_ok(lo_p.size() > 0 and absf(lo_p[lo_p.size() - 1] - 0.3) < 0.001, "sculpt strength: the stamp's pressure is the pen's 0.3 (%d values)" % lo_p.size())
	await _pen_stroke(a, b, 0.9)
	var hi := _last_stamp()
	var hi_p: PackedFloat32Array = hi.get("pressure", PackedFloat32Array())
	_ok(hi_p.size() > 0 and absf(hi_p[hi_p.size() - 1] - 0.9) < 0.001, "sculpt strength: a second stroke at 0.9 carries 0.9")
	_ok(var_to_str(lo.get("params")) == var_to_str(hi.get("params")), "sculpt strength: the stamp params (radius) are identical at 0.3 and 0.9")
	br.sculpt_discard()
	DccSettings.set_pressure_control("sculpt", "off")
	await _pen_stroke(a, b, 0.5)
	var off := _last_stamp()
	_ok(not off.is_empty() and not off.has("pressure"), "sculpt off: a pen stroke carries NO pressure (absent, not 1.0)")
	br.sculpt_discard()
	DccSettings.set_pressure_control("sculpt", "strength")
	await _mouse_stroke(a, b)
	var ms := _last_stamp()
	_ok(not ms.is_empty() and not ms.has("pressure"), "sculpt strength: a mouse stroke carries NO pressure (control flow, not a guessed value)")
	br.sculpt_discard()

## The open draft's footprint: the engine's preview-patch rectangle (grid cells, the bounding box of the dabs) and its
## dab count. Dabs are placed every 18 % of the dab DIAMETER, so a smaller dab on the same drag makes MORE dabs and a
## narrower rectangle: both move with the dab size and neither with the pen unless pressure drives size. (An alpha
## count of the patch's pixels read 0 here, so it is not used as evidence.)
func _painted_total() -> Vector3i:
	var patch: Dictionary = _app.bridge.build_paint_preview_patch()
	return Vector3i(int(patch.get("w", -1)), int(patch.get("h", -1)), _app.bridge.paint_draft_count())

func _paint_area(use_mouse: bool, pressure: float, P: Vector2) -> Vector3i:
	var br = _app.bridge
	br.paint_discard()
	var a := P
	var b := P + Vector2(40.0, 0.0)
	if use_mouse:
		await _mouse_stroke(a, b)
	else:
		await _pen_stroke(a, b, pressure)
	var n := _painted_total()
	print("footprint ", n)
	br.paint_discard()
	return n

func _t_paint(P: Vector2) -> void:
	await _arm("paint")
	DccSettings.set_pen_pressure_curve("linear")
	DccSettings.set_pen_pressure_max(1.0)
	DccSettings.set_pen_tilt_to_size(false)
	var ws: WorldWorkspace = _app._world_workspace()
	DccSettings.set_pressure_control("paint", "off")
	var off_lo: Vector3i = await _paint_area(false, 0.1, P)
	var off_hi: Vector3i = await _paint_area(false, 1.0, P)
	_ok(off_lo.z > 0 and off_lo.x > 0, "paint off: the stroke painted something %s" % str(off_lo))
	_ok(off_lo == off_hi, "paint off: the footprint is identical at pressure 0.1 and 1.0 (%s vs %s)" % [str(off_lo), str(off_hi)])
	_ok(ws.last_begin_options("paint").get("pressure_size", null) == false, "paint off: the engine was handed pressure_size = false")
	DccSettings.set_pressure_control("paint", "size")
	var sz_lo: Vector3i = await _paint_area(false, 0.1, P)
	var sz_hi: Vector3i = await _paint_area(false, 1.0, P)
	var sz_ms: Vector3i = await _paint_area(true, 1.0, P)
	_ok(ws.last_begin_options("paint").get("pressure_size", null) == true, "paint size: the engine was handed pressure_size = true")
	_ok(sz_lo.z > sz_hi.z and sz_lo.x < sz_hi.x, "paint size: pressure 0.1 makes a smaller dab than 1.0 (more dabs, narrower: %s vs %s)" % [str(sz_lo), str(sz_hi)])
	_ok(sz_ms == sz_hi, "paint size: a mouse (no pressure) makes exactly the full-radius footprint (%s vs %s)" % [str(sz_ms), str(sz_hi)])
	_ok(sz_hi == off_hi, "paint: size at full pressure equals off (the slider's footprint, %s)" % str(off_hi))
	DccSettings.set_pressure_control("paint", "off")
