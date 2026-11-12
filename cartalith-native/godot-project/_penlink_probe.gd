extends Node
## Pen link probe (Ruling EK, SP-2..SP-6). There is NO BLE on this PC, so every pen event here is a FAKE backend's
## signal: the probe proves the facade's policy (absent without error, off by default, permission only on switch-on,
## the defaults, the per-kind mappings through the real entry points, the slide accumulator, the misfire guard, the
## battery chip, the menu and the tester block). It does NOT prove the Kotlin client or what the Pad 3's pen sends:
## that is the owner's device pass (STYLO2_PEN_PROPOSAL.md section 11). Literals are written by hand.
##
## Windowed, wiped user dir, under the shared lock (`C:/cw/.win_lock`):
##   godot --path . --resolution 1600x1000 _penlink_probe.tscn -- --vp 1600x1000
##   (also --vp 1080x2340 --density 2.8125 --force-touch for the phone form)
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _app: Node
var _link: PenLink

## A stand-in for the CartalithPen singleton: same four signals, same methods, counts every call.
class FakeBackend extends Node:
	signal pen_state(state: String)
	signal pen_double_tap
	signal pen_slide(pos: int)
	signal pen_info(battery: int, firmware: String, name: String)
	var starts := 0
	var stops := 0
	var supported := true
	var permitted := true
	func is_supported() -> bool: return supported
	func has_permission() -> bool: return permitted
	func start() -> String:
		starts += 1
		return "connecting"
	func stop() -> void: stops += 1
	func query_battery() -> void: pass
	func query_info() -> void: pass

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
	var parts := _arg("--vp", "1600x1000").split("x")
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	var vp := SubViewport.new()
	vp.size = Vector2i(vw, vh)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	_app = load("res://shell/app.tscn").instantiate()
	vp.add_child(_app)
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
	_link = _app.pen_link
	_ok(_link != null, "the app owns a PenLink child")
	await _t_absent()
	await _t_defaults()
	await _t_off_by_default()
	await _t_permission()
	await _t_tap()
	await _t_slide()
	await _t_chip()
	await _t_menu()
	await _t_tester()

# ---- absent: no plugin, no error ---------------------------------------------------------------------------------

func _t_absent() -> void:
	_ok(not _link.available(), "no plugin on this PC: available() is false")
	_ok(_link.state() == "absent", "state() is absent, not off or an error")
	_ok(_link.reason() == "Needs a OnePlus Stylo 2 on Android", "the absent reason names the requirement")
	_ok(_link.battery() == -1, "battery is the absent marker -1, never 0")
	_link.apply_setting()
	_ok(true, "apply_setting() with no backend is a no-op")
	_ok(not Engine.has_singleton("CartalithPen"), "this PC has no CartalithPen singleton")

# ---- the defaults (Ruling EK) ---------------------------------------------------------------------------------------

func _t_defaults() -> void:
	_ok(not DccSettings.pen_link_on(), "Pen link defaults to OFF")
	for k in ["drawing", "placing", "none"]:
		_ok(DccSettings.pen_double_tap(k) == "last_tool", "double tap defaults to last_tool for " + k)
	_ok(DccSettings.pen_slide("drawing") == "brush_size", "slide defaults to brush_size for drawing tools")
	_ok(DccSettings.pen_slide("placing") == "off" and DccSettings.pen_slide("none") == "off", "slide is off for placing and none")
	_ok(DccSettings.pen_low_battery_chip(), "low-battery chip defaults on")
	_ok(not DccSettings.pen_slide_invert(), "slide direction defaults to not flipped")
	DccSettings.set_pen_double_tap("drawing", "vibrate")
	_ok(DccSettings.pen_double_tap("drawing") == "last_tool", "an unknown double-tap action is refused, not stored")
	DccSettings.set_pen_double_tap("drawing", "pin:")
	_ok(DccSettings.pen_double_tap("drawing") == "last_tool", "an empty pin key is refused")
	DccSettings._ensure_loaded()
	DccSettings._cfg.set_value("stylus", "double_tap_drawing", "vibrate")
	_ok(DccSettings.pen_double_tap("drawing") == "last_tool", "a hand-edited unknown action reads back as the default")
	DccSettings._cfg.set_value("stylus", "double_tap_drawing", "last_tool")
	DccSettings.set_pen_slide("drawing", "vibrate")
	_ok(DccSettings.pen_slide("drawing") == "brush_size", "an unknown slide action is refused")

# ---- off by default: nothing starts, no permission is asked -----------------------------------------------------

func _t_off_by_default() -> void:
	var fb := FakeBackend.new()
	add_child(fb)
	_link.attach_backend(fb)
	_ok(_link.available() and _link.state() == "off", "with a backend attached and the switch off, state is off")
	_link.apply_setting()
	_ok(fb.starts == 0, "switch off: start() is never called")
	_ok(fb.stops == 1, "switch off: stop() is called (a no-op in the plugin)")
	DccSettings.set_pen_link_on(true)
	fb.supported = false
	_link.apply_setting()
	_ok(fb.starts == 0 and _link.state() == "unsupported", "below API 33 (is_supported false): not started, state unsupported")
	fb.supported = true
	_link.apply_setting()
	_ok(fb.starts == 1, "switch on, supported and permitted: start() once")
	fb.pen_state.emit("connected")
	_ok(_link.state() == "connected" and _link.reason() == "", "connected has no reason text")
	fb.pen_info.emit(87, "4D45.03.00.12", "OnePlus Stylo 2")
	_ok(_link.battery() == 87 and _link.firmware() == "4D45.03.00.12", "info is stored")
	fb.pen_info.emit(250, "", "")
	_ok(_link.battery() == -1, "an out-of-range battery is absent, not stored")
	fb.pen_state.emit("lost")
	_ok(_link.battery() == -1 and _link.firmware() == "", "a lost link forgets battery and firmware")
	DccSettings.set_pen_link_on(false)
	_link._chip_shown = false

# ---- permission flow -------------------------------------------------------------------------------------------------

func _t_permission() -> void:
	var fb: FakeBackend = _link._backend
	fb.permitted = false
	DccSettings.set_pen_link_on(true)
	fb.starts = 0
	_link._on_permission_result("android.permission.CAMERA", false)
	_ok(DccSettings.pen_link_on(), "a different permission's answer is ignored")
	_link._on_permission_result("android.permission.BLUETOOTH_CONNECT", false)
	_ok(not DccSettings.pen_link_on(), "BLUETOOTH_CONNECT denied: the switch returns to off")
	_ok(_link.state() == "no_permission" and _link.reason() == "Bluetooth permission denied", "denied: state and reason")
	DccSettings.set_pen_link_on(true)
	_link._on_permission_result("android.permission.BLUETOOTH_CONNECT", true)
	_ok(fb.starts == 1, "BLUETOOTH_CONNECT granted: start() once")
	DccSettings.set_pen_link_on(false)
	fb.permitted = true

# ---- double tap through the real entry points ------------------------------------------------------------------------

func _tool(id: String) -> void:
	_app.arm_tool(id)

func _t_tap() -> void:
	var fb: FakeBackend = _link._backend
	# Last tool: sculpt -> measure -> tap -> sculpt -> tap -> measure (swaps).
	_tool("sculpt")
	_tool("measure")
	_ok(_link.prev_tool() == "sculpt", "the facade tracks the previous tool")
	DccSettings.set_pen_double_tap("placing", "last_tool")
	fb.pen_double_tap.emit()
	_ok(_app.armed_tool == "sculpt", "double tap = Last tool re-arms the previous tool through arm_tool")
	fb.pen_double_tap.emit()
	_ok(_app.armed_tool == "measure", "a second tap swaps back")
	# No earlier tool: say so, arm nothing.
	var keep: String = _link._prev_tool
	_link._prev_tool = ""
	_ok(_link.run_tap_action("last_tool") == "No earlier tool to go back to." and _app.armed_tool == "measure",
		"Last tool with no earlier tool answers why and arms nothing")
	_link._prev_tool = keep
	# Off.
	DccSettings.set_pen_double_tap("placing", "off")
	fb.pen_double_tap.emit()
	_ok(_app.armed_tool == "measure", "Off does nothing")
	# Eraser on Paint and on Sculpt.
	DccSettings.set_pen_double_tap("drawing", "eraser")
	_tool("paint")
	await _frames(2)
	var ws = _app._world_workspace()
	var before: bool = bool(ws._paint_brush["erase"])
	fb.pen_double_tap.emit()
	_ok(bool(ws._paint_brush["erase"]) == not before, "eraser: Paint's Erase switch flips")
	fb.pen_double_tap.emit()
	_ok(bool(ws._paint_brush["erase"]) == before, "eraser: a second tap flips it back")
	_tool("sculpt")
	await _frames(2)
	_ok(_app.bridge.sculpt_set_feature("freehand"), "Sculpt's feature set to Freehand")
	var m0: String = _app.bridge.sculpt_get_freehand_mode()
	if m0 == "raise" or m0 == "lower":
		fb.pen_double_tap.emit()
		var m1: String = _app.bridge.sculpt_get_freehand_mode()
		_ok(m1 == ("lower" if m0 == "raise" else "raise"), "eraser: Sculpt's Freehand mode swaps raise/lower (%s -> %s)" % [m0, m1])
		fb.pen_double_tap.emit()
		_ok(_app.bridge.sculpt_get_freehand_mode() == m0, "eraser: and back")
	else:
		_ok(false, "sculpt starts in raise or lower, got '%s'" % m0)
	# An eraser mapping on a tool with no erase says why and moves nothing.
	DccSettings.set_pen_double_tap("placing", "eraser")
	_tool("measure")
	fb.pen_double_tap.emit()
	_ok(_app.armed_tool == "measure", "eraser on a placing tool changes no tool")
	_ok(_link.run_tap_action("eraser") == "This tool has no erase mode.", "the refusal names the reason (never silent)")
	# Undo: runs the single undo entry point without error.
	DccSettings.set_pen_double_tap("placing", "undo")
	fb.pen_double_tap.emit()
	_ok(_link.last_action().begins_with("double tap: undo"), "undo runs through undo_last")
	# A pinned quick-bar command runs through run_pin_on; an unknown one is refused with its sentence.
	DccSettings.set_pen_double_tap("placing", "pin:tool:inspect")
	fb.pen_double_tap.emit()
	_ok(_app.armed_tool == "inspect", "pin:tool:inspect arms Inspect through the quick bar's path")
	_ok(_link.run_tap_action("pin:nope:zzz") == "No longer available.", "an unknown pinned command is refused with the quick bar's sentence")
	# The misfire guard.
	DccSettings.set_pen_double_tap("none", "last_tool")
	_tool("measure")
	_tool("inspect")
	var tip := InputEventMouseButton.new()
	tip.button_index = MOUSE_BUTTON_LEFT
	tip.device = 0
	tip.pressed = true
	_link._input(tip)
	_ok(_link.handle_double_tap(Time.get_ticks_msec()) != "", "tip on the glass: the double tap is ignored")
	_ok(_app.armed_tool == "inspect", "...and nothing changed")
	tip.pressed = false
	_link._input(tip)
	var lifted: int = _link._tip_up_ms
	_ok(_link.handle_double_tap(lifted + 149) != "", "149 ms after the lift: still ignored")
	_ok(_link.handle_double_tap(lifted + 151) == "", "151 ms after the lift: acts")
	_ok(_app.armed_tool == "measure", "...Last tool took effect")
	_tool("inspect")
	for k in ["drawing", "placing", "none"]:
		DccSettings.set_pen_double_tap(k, "last_tool")

# ---- slide -----------------------------------------------------------------------------------------------------------------

func _t_slide() -> void:
	var fb: FakeBackend = _link._backend
	# The pure accumulator, literals by hand.
	var st := {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_ok(PenLink.slide_steps(st, 100, 1000, 6) == 0, "the first report is a new contact: 0 steps")
	_ok(PenLink.slide_steps(st, 105, 1010, 6) == 0, "5 positions: below one step")
	_ok(PenLink.slide_steps(st, 106, 1020, 6) == 1, "6 positions in total: one step")
	_ok(PenLink.slide_steps(st, 130, 1030, 6) == 4, "24 more positions: four steps")
	_ok(PenLink.slide_steps(st, 112, 1040, 6) == -3, "18 back: three steps down")
	_ok(PenLink.slide_steps(st, 200, 1600, 6) == 0, "a 560 ms gap starts a new contact: no delta across a lift")
	var still := {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	var moved := 0
	var t := 0
	PenLink.slide_steps(still, 100, t, 6)
	for n in 400:  # 10 s at 40 Hz of +-1 jitter
		t += 25
		moved += absi(PenLink.slide_steps(still, 100 + (n % 2), t, 6))
	_ok(moved == 0, "a still finger jittering +-1 for 10 s moves 0 steps")
	# Through the facade: Paint's radius moves exactly one cell per 6 positions.
	_tool("paint")
	await _frames(2)
	var ws = _app._world_workspace()
	DccSettings.set_pen_slide("drawing", "brush_size")
	ws._paint_brush["radius"] = 10.0
	var base := Time.get_ticks_msec()
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_ok(_link.handle_slide(100, base) == 0, "slide: contact starts")
	_ok(_link.handle_slide(118, base + 20) == 3, "slide: +18 positions = +3 steps")
	_ok(is_equal_approx(float(ws._paint_brush["radius"]), 13.0), "Paint radius 10 -> 13 through the workspace stepper")
	_ok(_link.handle_slide(100, base + 40) == -3, "slide: -18 = -3 steps")
	_ok(is_equal_approx(float(ws._paint_brush["radius"]), 10.0), "radius back to 10")
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	ws._paint_brush["radius"] = 10.0
	_link.handle_slide(100, base + 500)
	_ok(_link.handle_slide(117, base + 510) == 2 and is_equal_approx(float(ws._paint_brush["radius"]), 12.0),
		"17 positions = 2 steps (6 positions per step; the remainder is carried)")
	ws._paint_brush["radius"] = 10.0
	DccSettings.set_pen_slide_invert(true)
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_link.handle_slide(100, base + 1000)
	_link.handle_slide(112, base + 1010)
	_ok(is_equal_approx(float(ws._paint_brush["radius"]), 8.0), "flipped direction: +12 positions lowers the radius by 2")
	DccSettings.set_pen_slide_invert(false)
	# Sculpt's size.
	_tool("sculpt")
	await _frames(2)
	var s0 := float(_app.bridge.sculpt_get_globals().get("brush_size", 0.0))
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_link.handle_slide(100, base + 3000)
	_link.handle_slide(112, base + 3010)
	var s1 := float(_app.bridge.sculpt_get_globals().get("brush_size", 0.0))
	_ok(s1 > s0, "Sculpt brush size grows with a positive slide (%.1f -> %.1f)" % [s0, s1])
	# Off for a placing tool by default: a slide does nothing.
	_tool("measure")
	var z0: float = _app.viewport._zoom
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_link.handle_slide(100, base + 5000)
	_ok(_link.handle_slide(160, base + 5010) == 0 and is_equal_approx(_app.viewport._zoom, z0), "placing tool: slide is Off by default")
	# Zoom.
	DccSettings.set_pen_slide("placing", "zoom")
	_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
	_link.handle_slide(100, base + 7000)
	_link.handle_slide(112, base + 7010)
	_ok(_app.viewport._zoom > z0, "slide = Zoom zooms in through ViewportHost.zoom_step (%.3f -> %.3f)" % [z0, _app.viewport._zoom])
	# Year.
	DccSettings.set_pen_slide("placing", "year")
	if _app.tl_available():
		var y0: int = _app.tl_year()
		_link._slide = {"last_pos": -1, "last_ms": -1000000, "carry": 0}
		_link.handle_slide(100, base + 9000)
		_link.handle_slide(110, base + 9010)
		_ok(_app.tl_year() == y0 + 10, "slide = Timeline year: +10 positions = +10 years via tl_set_year (%d -> %d)" % [y0, _app.tl_year()])
	else:
		_ok(false, "the timeline is available after generating")
	# The signal path.
	DccSettings.set_pen_slide("placing", "off")
	fb.pen_slide.emit(120)
	_ok(true, "a pen_slide signal reaches the facade without error")
	for k in ["placing", "none"]:
		DccSettings.set_pen_slide(k, "off")
	_tool("inspect")

# ---- battery chip -------------------------------------------------------------------------------------------------------

func _t_chip() -> void:
	var fb: FakeBackend = _link._backend
	_link._chip_shown = false
	_app.set_status("hint", "", "text_ghost")
	fb.pen_info.emit(21, "", "")
	_ok(not _link._chip_shown, "21 %: no chip")
	fb.pen_info.emit(50, "", "")
	_ok(not _link._chip_shown, "50 %: no chip")
	fb.pen_info.emit(-1, "", "")
	_ok(not _link._chip_shown, "unknown battery: no chip")
	fb.pen_info.emit(20, "", "")
	_ok(_link._chip_shown, "20 %: the chip fires")
	var text: String = _app._status_labels["hint"].text
	_ok(text.contains("20 %"), "the chip says the level on the status line ('%s')" % text)
	_app.set_status("hint", "", "text_ghost")
	fb.pen_info.emit(15, "", "")
	_ok(_app._status_labels["hint"].text == "", "once per session: a second low reading does not repeat it")
	_link._chip_shown = false
	DccSettings.set_pen_low_battery_chip(false)
	fb.pen_info.emit(5, "", "")
	_ok(not _link._chip_shown, "chip switched off: nothing at 5 %")
	DccSettings.set_pen_low_battery_chip(true)
	fb.pen_state.emit("off")

# ---- menu ----------------------------------------------------------------------------------------------------------------------

func _t_menu() -> void:
	var menus = _app.menus
	var pm: PopupMenu = menus._penlink_popup
	_ok(pm != null, "Preferences > Stylus > Pen link exists")
	var pop: PopupMenu = menus._pen_kind_popups["tap_drawing"]
	_ok(pop.item_count == 4, "double tap offers Last tool, Off, Eraser, Undo with an empty quick bar (4 rows, got %d)" % pop.item_count)
	_ok(pop.is_item_checked(0), "Last tool is ticked by default")
	_ok(not pm.is_item_checked(pm.get_item_index(menus.ID_PEN_LINK_ON)), "Pen link on is unticked by default")
	menus._on_penlink_choice(menus.ID_PEN_TAP_BASE + 0 * 10 + 1)
	_ok(DccSettings.pen_double_tap("drawing") == "off", "choosing Off for drawing tools stores it")
	menus._on_penlink_choice(menus.ID_PEN_SLIDE_BASE + 2 * 10 + 2)
	_ok(DccSettings.pen_slide("none") == "zoom", "choosing Zoom for Inspect's slide stores it")
	DccSettings.set_pen_slide("none", "off")
	DccSettings.set_pen_double_tap("drawing", "last_tool")
	menus._refresh_penlink_menu()
	var status := pm.get_item_text(pm.get_item_index(menus.ID_PEN_STATUS))
	_ok(status.begins_with("Status: "), "the status row reads: " + status)
	# No vibration anywhere in the menu.
	for sub in menus._pen_kind_popups.values():
		for k in (sub as PopupMenu).item_count:
			_ok(not (sub as PopupMenu).get_item_text(k).to_lower().contains("vibrat"), "no vibration row: " + (sub as PopupMenu).get_item_text(k))
	# The quick-bar pins are offered as commands.
	DccSettings.quick_bar_add("bar", "tool:measure", "Measure")
	menus._refresh_penlink_menu()
	_ok(menus._pen_kind_popups["tap_drawing"].item_count == 5, "a quick-bar pin adds a command row")
	menus._on_penlink_choice(menus.ID_PEN_TAP_BASE + 0 * 10 + 4)
	_ok(DccSettings.pen_double_tap("drawing") == "pin:tool:measure", "choosing the pin stores pin:<key>")
	DccSettings.set_pen_double_tap("drawing", "last_tool")
	DccSettings.quick_bar_remove("bar", "tool:measure")

# ---- tester block ------------------------------------------------------------------------------------------------------------

func _t_tester() -> void:
	var fb: FakeBackend = _link._backend
	var t := StylusTester.open_in_window(_app)
	await _frames(3)
	var snap := t.snapshot()
	_ok(String(snap["pen battery"]).begins_with("—"), "tester: battery reads a dash with a reason when not connected ('%s')" % snap["pen battery"])
	fb.pen_state.emit("connected")
	fb.pen_info.emit(64, "4D45.03.00.12", "OnePlus Stylo 2")
	snap = t.snapshot()
	_ok(String(snap["pen battery"]) == "64 %", "tester: battery 64 %% ('%s')" % snap["pen battery"])
	_ok(String(snap["pen firmware"]) == "4D45.03.00.12", "tester: firmware")
	_ok(String(snap["pen link"]) == "connected", "tester: link connected")
	fb.pen_state.emit("lost")
	snap = t.snapshot()
	_ok(String(snap["pen battery"]).begins_with("—"), "tester: battery back to a dash when the link is lost")
