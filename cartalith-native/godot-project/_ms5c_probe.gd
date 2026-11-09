extends Node
## MS-5 remainder probe (`MOBILE_SHELL_PROPOSAL.md` section 2.1, "Long-press on the map ... the card leads with four to six
## actions for the current domain", and section 5's "a submenu row becomes a jump"): the phone peek card's lead verbs.
## Drives REAL touch events (an `InputEventScreenTouch` / `ScreenDrag` plus its emulated mouse event, as `_ms5_probe.gd`
## does) at the real shell in a `--vp`-sized SubViewport.
##
## Checks, with literals written by hand (nothing is compared with the constant it came from):
##   1. a real hold on empty ground in WORLD, CIVIL and CARTO leads with the proposal's verbs, in order, and a real tap
##      on the first WORLD chip (Sculpt here) arms Sculpt;
##   2. the two CARTO submenu rows are jumps on the phone: a real tap on Style preset opens CARTO > Style with the sheet
##      at half, and a real tap on View field opens the Layers popover;
##   3. `GlobalTools.lead_chip` promotes nothing when the press hit something, and ranks when it did not.
##      (A claimed cell keeping the faction's Open row first is `_ctxphone_probe`'s FR leg.)
## Phone forms only: on a tablet or desktop form the probe asserts only that no tool fan exists, which is MS-5's own.
##
## Windowed with a wiped user dir. Arguments read by THIS file: `--vp WxH`; accepted because the shell reads them:
## `--density D`, `--force-touch`.
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
	get_tree().quit(0 if _fails == 0 else 1)

# ---- fingers -----------------------------------------------------------------------------------------------------

func _down(p: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)

func _move(p: Vector2, from: Vector2) -> void:
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - from
	_vp.push_input(d)
	var m := InputEventMouseMotion.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)

func _up(p: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2) -> void:
	await _down(p)
	await _up(p)

func _drag(a: Vector2, b: Vector2) -> void:
	var cur := a
	for s in 6:
		var nxt := a.lerp(b, float(s + 1) / 6.0)
		await _move(nxt, cur)
		cur = nxt

func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()

func _arm(id: String) -> void:
	_app.arm_tool(id)
	await _frames(4)
	await get_tree().create_timer(0.45).timeout

func _find_point() -> Vector2:
	var r := _host.get_global_rect()
	for fy in [0.30, 0.36, 0.24, 0.42, 0.5]:
		for fx in [0.5, 0.4, 0.6]:
			var p := Vector2(r.position.x + r.size.x * fx, r.position.y + r.size.y * fy)
			var navr: Rect2 = _host._navpad.get_global_rect() if _host._navpad != null else Rect2()
			if navr.has_point(p):
				continue
			var lp: Vector2 = _ov.get_global_transform().affine_inverse() * p
			var q: Dictionary = _ov._grid_point(lp, _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
			if bool(q.get("valid", false)):
				return p
	return Vector2(-1, -1)

func _chip_texts() -> Array:
	var pm = _app.get("_phone_menu")
	var out: Array = []
	for c in pm._peek_chip_row.get_children():
		if c is Button and not c.is_queued_for_deletion():
			out.append((c as Button).text)
	return out

func _close_card() -> void:
	var pm = _app.get("_phone_menu")
	if pm != null and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	_ov._sample_pin = {}
	await _frames(4)

## A real hold on empty ground (phone hold is 480 ms) and the card's chip texts.
func _hold_card(P: Vector2) -> Array:
	await _down(P)
	await get_tree().create_timer(0.8).timeout
	await _up(P)
	await _frames(6)
	var pm = _app.get("_phone_menu")
	var open: bool = pm != null and pm.peek_card_is_open()
	var texts: Array = _chip_texts() if open else []
	await _close_card()
	return texts

## A real hold on empty ground, then a real tap on the card's chip whose text is `text`.
func _hold_card_then_tap(P: Vector2, text: String) -> void:
	await _down(P)
	await get_tree().create_timer(0.8).timeout
	await _up(P)
	await _frames(8)
	var pm = _app.get("_phone_menu")
	var hit: Button = null
	if pm != null and pm.peek_card_is_open():
		for c in pm._peek_chip_row.get_children():
			if c is Button and (c as Button).text == text:
				hit = c
	_ok(hit != null, "the card has a '%s' chip to tap" % text)
	if hit != null:
		await _tap(_centre(hit))
		await get_tree().create_timer(0.6).timeout
	await _close_card()

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
	if not DccTheme.is_phone():
		_ok(_app._tool_fan == null, "off the phone there is no tool fan")
		return
	_app._pick_phone_tab("map")
	_app.set_phone_detent("peek")
	await get_tree().create_timer(0.4).timeout
	var P := _find_point()
	_ok(P.x >= 0.0, "a valid map point was found, %s" % str(P))
	if P.x < 0.0:
		return
	await _arm("inspect")
	_app.select_domain("world")
	await get_tree().create_timer(0.5).timeout
	var w := await _hold_card(P)
	_ok(w.size() >= 2 and w[0] == "Sculpt here" and w[1] == "Cross-section from here",
		"WORLD's card leads with Sculpt here, Cross-section from here: %s" % str(w))
	# a real tap on the lead chip does what it says: Sculpt here arms Sculpt, and only that
	_ok(String(_app.armed_tool) == "inspect", "Inspect is armed before the Sculpt here tap")
	await _hold_card_then_tap(P, "Sculpt here")
	_ok(String(_app.armed_tool) == "sculpt", "the Sculpt here chip arms Sculpt, got '%s'" % String(_app.armed_tool))
	await _arm("inspect")
	_app.select_domain("civilization")
	await get_tree().create_timer(0.5).timeout
	var c := await _hold_card(P)
	_ok(c == ["Drop settlement here", "Start way here", "Start route here", "Info here (settlement & ecology)"],
		"CIVIL's card leads with Drop settlement, Start way, Start route, Info here, ahead of the story chips: %s" % str(c))
	_app.select_domain_category("cartography", "Layers")
	await get_tree().create_timer(0.5).timeout
	var k := await _hold_card(P)
	_ok(k == ["Add label here…", "View field", "Style preset", "Start export region here"],
		"CARTO's card leads with Add label, View field, Style preset, Export region (Stamp is disabled with no pack): %s" % str(k))
	# the two submenu rows are jumps on the phone (a real tap on each chip)
	_app.set_phone_detent("peek")
	await _hold_card_then_tap(P, "Style preset")
	_ok(String(_app.active_mode("cartography")) == "style", "Style preset jumps to CARTO > Style, node '%s'" % String(_app.active_mode("cartography")))
	_ok(String(_app._phone_detent) == "half", "the Style jump lifts the sheet to half, got %s" % String(_app._phone_detent))
	_app.set_phone_detent("peek")
	var lp = _app.get("layers_popover")
	_ok(lp != null and not lp.visible, "the Layers popover is closed before the View field jump")
	await _hold_card_then_tap(P, "View field")
	_ok(lp != null and lp.visible, "View field opens the Layers popover")
	lp.hide()
	# a press that hit something promotes nothing (unit-level, hand-written literals)
	var hit_req := {"hits": [{"kind": "settlement", "id": 3}]}
	_ok(not GlobalTools.lead_chip({"id": "x"}, 1, hit_req).has("chip"), "lead_chip adds no rank when the press hit something")
	_ok(int(GlobalTools.lead_chip({"id": "x"}, 4, {"hits": []}).get("chip", -1)) == 4, "lead_chip ranks 4 on empty ground")
