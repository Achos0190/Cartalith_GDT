extends Node
## MS-11 probe (`MOBILE_SHELL_PROPOSAL.md` section 2.4 and the MS-11 row of section 6): the phone's quick drawer, driven
## with REAL touch events (InputEventScreenTouch / ScreenDrag with their emulated mouse button and motion,
## `device = DEVICE_ID_EMULATION` as Godot stamps them) pushed at a `--vp`-sized host SubViewport, against the real shell.
##
## What it checks, each with literals written by hand (nothing is asserted against the constant it came from):
##   1. THE EMPTY BAR: no drawer, no node of that name, no grip, pause depth 0;
##   2. THE GRIP: at least 44 x 88 REAL dp (px / the shell's density) and at least 44 x 88 phone units, on the edge AWAY
##      from the dominant hand (left when right-handed, right when left-handed, and the swap is live), at the middle of the
##      map band's height, wholly on screen, closed by default. A positive control: the same walk flags a 40 dp rect;
##   3. OPEN: a tap opens it (panel visible and as wide as the figure, scrim at the 0.35 alpha, depth 1), a drag past
##      0.4 of the width opens it, a drag short of that settles shut;
##   4. CLOSE: a tap on the scrim, a drag of the grip back, and system back (`NOTIFICATION_WM_GO_BACK_REQUEST`) each
##      close it and give the pause depth back (0); a tap on the scrim does NOT reach the map;
##   5. EVERY PIN RUNS: for four pins (the vault, the planner and two tools) a real tap on the drawer's row has the same
##      observable effect as `DccQuickBar.run_pin_on` (the path the desktop bar's chip takes), the effect differs from the
##      state before it (so "nothing happened" cannot pass), the owning menu popup got exactly one `id_pressed` with the
##      row's id, and the drawer closed; a pin whose command is gone toasts, stays open and stays stored;
##   6. THE MAP: a closed drawer takes no tap that is not on its grip (a tap on the map still clicks);
##   7. EMPTY AGAIN: removing the last pin while the drawer is OPEN frees it and gives the depth back;
##   8. A CLASS SWAP: with it open, a same-class remount re-presents it open (a new instance, depth 1, not 0 or 2), a
##      remount to the tablet class leaves none and no depth, and back on the phone it is built again, closed.
## Windowed, with a wiped user dir (the probe writes pins and the dominant hand):
##   godot --path . --resolution 1600x1000 _ms11_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
##   (also 2340x1080 @2.8125, 1440x3120 @3, 1080x1920 @3 = 360 dp, and 1920x1080 @3 for its landscape)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

const PINS := ["menu:726", "menu:45", "tool:measure", "tool:region"]

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _host: Control
var _ov: Control
var _dens := 1.0
var _clicks := 0

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

# ---- fingers: real events pushed at the host viewport (as _ms5_probe.gd) -----------------------------------------

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

func _drag(a: Vector2, b: Vector2, steps: int = 6) -> void:
	await _down(0, a)
	var cur := a
	for s in steps:
		var nxt := a.lerp(b, float(s + 1) / float(steps))
		await _move(0, nxt, cur)
		cur = nxt
	await _up(0, b)

# ---- reads ----------------------------------------------------------------------------------------------------------

func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()

func _drawer() -> Control:
	return _app.phone_quick.drawer()

func _depth() -> int:
	return int(_app._context_surface_depth)

func _settle() -> void:
	await get_tree().create_timer(0.35).timeout
	await _frames(3)

## The map band the drawer lives in, in host pixels: what `phone_content_insets()` leaves, less the band the
## armed-tool strip takes at the bottom (`phone_tool_pill_reserve()`; the strip is drawn above the drawer).
func _band() -> Rect2:
	var ins: Dictionary = _app.phone_content_insets()
	var s := Vector2(_vp.size)
	## Landscape, right-handed (owner, 2026-11-13): the drawer's band runs over the sheet's column to the screen's far
	## right edge, so the grip and the panel start at the screen's edge. Every other case is what the insets leave.
	var right := float(ins["right"])
	if _app._landscape and DccSettings.dominant_hand() != "left":
		right = 0.0
	return Rect2(float(ins["left"]), float(ins["top"]), s.x - float(ins["left"]) - right,
		s.y - float(ins["top"]) - float(ins["bottom"]) - float(_app.phone_tool_pill_reserve()))

## What a pin visibly did: the visible windows, whether the planner view is up, the armed tool.
func _sig() -> Dictionary:
	var wins: Array = []
	for w in _vp.find_children("*", "Window", true, false):
		if (w as Window).visible:
			wins.append(String(w.name))
	wins.sort()
	var pv = _app.journey_planner_view
	return {"wins": wins, "planner": pv != null and is_instance_valid(pv) and pv.visible, "tool": String(_app.armed_tool)}

func _reset_effects() -> void:
	for w in _vp.find_children("*", "Window", true, false):
		if (w as Window).visible:
			(w as Window).hide()
	var pv = _app.journey_planner_view
	if pv != null and is_instance_valid(pv):
		pv.hide()
	_app.arm_tool("inspect")
	await _frames(4)

func _toasts() -> int:
	var n := 0
	for c in _app._phone_root.get_children():
		if c is PanelContainer and (c as Control).z_index == 50:
			n += 1
	return n

func _row_for(key: String) -> Button:
	for b in _drawer().rows():
		var e: Dictionary = b.get_meta("entry", {})
		if String(e.get("key", "")) == key:
			return b
	return null

## Opens the drawer with a real tap on its grip and waits for the slide.
func _open_by_tap() -> void:
	await _tap(_centre(_drawer().grip()))
	await _settle()

func _store_pins(keys: Array) -> void:
	for k in DccSettings.quick_bar_pins_for("bar", "").duplicate():
		DccSettings.quick_bar_remove("bar", String(k["k"]))
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	for k in keys:
		var row := idx.find_pin(String(k))
		DccSettings.quick_bar_add("bar", String(k), String(row.get("title", "")) if not row.is_empty() else String(k))
	_app.phone_quick.refresh()
	await _frames(6)

# ---- the run ----------------------------------------------------------------------------------------------------------

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
	_ok(DccTheme.is_phone(), "the shell is the phone form")
	if not DccTheme.is_phone():
		return
	_dens = float((_app._density() as Dictionary)["value"])
	print("  info  vp %s density %.4f phone_scale %.4f landscape %s" % [str(_vp.size), _dens, _app.phone_scale(), str(_app._landscape)])
	_app._pick_phone_tab("map")
	_app.set_phone_detent("peek")
	await _settle()
	_ov.map_clicked.connect(func(_x, _y): _clicks += 1)

	await _t_empty()
	await _store_pins(PINS)
	await _t_grip()
	await _t_open_close()
	await _t_drag()
	await _t_back()
	await _t_full_detent()
	await _t_pins()
	await _t_map()
	await _t_hand()
	await _t_swap()
	await _t_empty_again()

# 1 -----------------------------------------------------------------------------------------------------------

func _t_empty() -> void:
	print("== 1. the empty bar ==")
	_ok(_app.phone_quick.drawer() == null, "no drawer with nothing pinned and the assistant off")
	_ok(_app._phone_root.find_child("QuickDrawer", true, false) == null, "and no node of that name in the phone's tree")
	_ok(_depth() == 0, "the pause depth is 0")

# 2 -----------------------------------------------------------------------------------------------------------

## True when `r` (host px) is at least `w_dp` x `h_dp` real dp.
func _dp_ok(r: Rect2, w_dp: float, h_dp: float) -> bool:
	return r.size.x / _dens >= w_dp - 0.6 and r.size.y / _dens >= h_dp - 0.6

func _t_grip() -> void:
	print("== 2. the grip ==")
	var d := _drawer()
	_ok(d != null and d.visible, "pins mounted a visible drawer")
	if d == null:
		return
	var g: Control = d.grip()
	var gr := g.get_global_rect()
	_ok(_dp_ok(gr, 44.0, 88.0), "the grip is at least 44 x 88 real dp (%.1f x %.1f dp)" % [gr.size.x / _dens, gr.size.y / _dens])
	var u: float = _app.phone_scale()
	_ok(gr.size.x / u >= 44.0 - 0.6 and gr.size.y / u >= 88.0 - 0.6, "and at least 44 x 88 phone units (%.1f x %.1f)" % [gr.size.x / u, gr.size.y / u])
	_ok(not _dp_ok(Rect2(0, 0, 40.0 * _dens, 80.0 * _dens), 44.0, 88.0), "positive control: a 40 x 80 dp rect IS flagged by the same walk")
	var band := _band()
	if _app._landscape:
		_ok(absf(gr.end.x - float(_vp.size.x)) <= 1.5, "landscape right-handed (owner 2026-11-13): the grip touches the screen's far right edge (end x %.1f, screen %d)" % [gr.end.x, _vp.size.x])
		var sh: Rect2 = _app._phone_tool_sheet.get_global_rect()
		_ok(_app._phone_detent != "peek" or not gr.intersects(sh), "and, at peek, does not overlap the sheet")
	else:
		_ok(absf(gr.position.x - band.position.x) <= 1.5, "right-handed: the grip sits on the band's LEFT edge (x %.1f, band x %.1f)" % [gr.position.x, band.position.x])
		_ok(absf(gr.get_center().y - band.get_center().y) <= 2.0, "at the middle of the band's height (%.1f vs %.1f)" % [gr.get_center().y, band.get_center().y])
	_ok(Rect2(Vector2.ZERO, Vector2(_vp.size)).encloses(gr), "wholly on screen")
	_ok(not d.is_open() and not d.get("_panel").visible, "closed by default, the panel not drawn")
	_ok(d.scrim().mouse_filter == Control.MOUSE_FILTER_IGNORE, "and the scrim takes no input while closed")

# 3 -----------------------------------------------------------------------------------------------------------

func _t_open_close() -> void:
	print("== 3/4. open and close ==")
	var d := _drawer()
	await _open_by_tap()
	_ok(d.is_open(), "a tap on the grip opens it")
	var panel: Control = d.get("_panel")
	_ok(panel.visible, "the panel is drawn")
	var band := _band()
	var want_w := minf(roundf(248.0 * _app.phone_scale()), band.size.x * 0.78)
	_ok(absf(panel.size.x - want_w) <= 1.5, "the panel is %.0f px wide (248 units, at most 0.78 of the band): %.1f" % [want_w, panel.size.x])
	if _app._landscape:
		_ok(absf(panel.get_global_rect().end.x - band.end.x) <= 1.5, "and flush with the band's right edge once open (landscape)")
	else:
		_ok(absf(panel.get_global_rect().position.x - band.position.x) <= 1.5, "and flush with the band's left edge once open")
	_ok(absf(d.scrim().color.a - 0.35) < 0.01, "the scrim is at the 0.35 alpha (%.3f)" % d.scrim().color.a)
	_ok(_depth() == 1, "an open drawer holds one share of the world pause: depth 1 (got %d)" % _depth())
	var gr: Rect2 = d.grip().get_global_rect()
	if _app._landscape:
		_ok(absf(gr.end.x - panel.get_global_rect().position.x) <= 1.5, "the grip rides on the panel's inner edge (landscape: its left edge)")
	else:
		_ok(absf(gr.position.x - panel.get_global_rect().end.x) <= 1.5, "the grip rides on the panel's inner edge")
	# the scrim: a tap on the band to the right of the panel
	var clicks0 := _clicks
	var far := Vector2((band.position.x + 12.0) if _app._landscape else (band.end.x - 12.0), band.position.y + band.size.y * 0.3)
	await _tap(far)
	await _settle()
	_ok(not d.is_open() and not panel.visible, "a tap on the scrim closes it")
	_ok(_clicks == clicks0, "and that tap did not reach the map (%d clicks)" % (_clicks - clicks0))
	_ok(_depth() == 0, "depth back to 0 (got %d)" % _depth())

func _t_drag() -> void:
	print("== 3/4. drags ==")
	var d := _drawer()
	var g: Control = d.grip()
	var band := _band()
	var w := minf(roundf(248.0 * _app.phone_scale()), band.size.x * 0.78)
	## +1 when the grip is on the left edge (inward is rightward), -1 on the right edge (landscape).
	var s := -1.0 if _app._landscape else 1.0
	var c := _centre(g)
	await _drag(c, c + Vector2(s * w * 0.2, 0.0))
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "a drag of 0.2 of the width settles shut")
	c = _centre(g)
	await _drag(c, c + Vector2(s * w * 0.7, 0.0))
	await _settle()
	_ok(d.is_open() and _depth() == 1, "a drag of 0.7 of the width settles open, depth 1")
	c = _centre(g)
	await _drag(c, c - Vector2(s * w * 0.8, 0.0))
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "a drag of the grip back closes it, depth 0")
	# a 3 px wobble is a tap
	c = _centre(g)
	await _drag(c, c + Vector2(s * 3.0, 0.0), 1)
	await _settle()
	_ok(d.is_open(), "a 3 px wobble on the grip is a tap, not a drag (opens)")
	await _tap(_centre(g))
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "a second tap on the grip closes it")

func _t_back() -> void:
	print("== 4. system back ==")
	var d := _drawer()
	await _open_by_tap()
	_ok(d.is_open(), "opened for the back test")
	_app.notification(NOTIFICATION_WM_GO_BACK_REQUEST)
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "system back closes it and gives the depth back")

## At the sheet's FULL detent the map band is one tap target tall: the drawer is not drawn, and an OPEN one closes and
## gives the pause back instead of hiding itself open (portrait only: the landscape sheet has no detents).
func _t_full_detent() -> void:
	if _app._landscape:
		return
	print("== 4. the band becomes too short ==")
	var d := _drawer()
	await _open_by_tap()
	_ok(d.is_open() and _depth() == 1, "open, depth 1")
	_app.set_phone_detent("full")
	await _settle()
	_ok(not d.visible, "at the FULL detent the drawer is not drawn")
	_ok(not d.is_open() and _depth() == 0, "and an open one closed and gave the depth back (open %s, depth %d)" % [str(d.is_open()), _depth()])
	_app.set_phone_detent("peek")
	await _settle()
	_ok(d.visible and d.grip().is_visible_in_tree(), "back at the peek detent the grip is drawn again")

# 5 -----------------------------------------------------------------------------------------------------------

func _t_pins() -> void:
	print("== 5. every pin runs from the drawer ==")
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	var d := _drawer()
	_ok(d.rows().size() == PINS.size() + 1, "four pin rows and Customise (%d)" % d.rows().size())
	for k in PINS:
		var row := idx.find_pin(String(k))
		_ok(not row.is_empty() and bool(row.get("pinnable", false)), "%s is a live pinnable command" % k)
	for k in PINS:
		await _reset_effects()
		var before := _sig()
		var row := idx.find_pin(String(k))
		# the desktop bar's path
		var why := DccQuickBar.run_pin_on(_app, String(k))
		await _frames(6)
		var want := _sig()
		await _reset_effects()
		_ok(why == "", "%s: DccQuickBar.run_pin_on ran it" % k)
		_ok(want != before, "%s: running it changes what is visible (%s -> %s)" % [k, str(before), str(want)])
		# the drawer's path, counting the owning popup's id_pressed
		var seen: Array = []
		var popup = row.get("_popup")
		var cb := func(id: int): seen.append(id)
		if popup != null:
			(popup as PopupMenu).id_pressed.connect(cb)
		await _open_by_tap()
		var b := _row_for(String(k))
		_ok(b != null, "%s: the drawer has its row" % k)
		if b == null:
			continue
		(b.get_parent().get_parent() as ScrollContainer).ensure_control_visible(b)
		await _frames(3)
		_ok(_vp.get_visible_rect().encloses(b.get_global_rect()), "%s: its row is on screen" % k)
		_ok(b.size.y / _app.phone_scale() >= 52.0 - 0.6, "%s: its row is at least 52 units tall (%.1f)" % [k, b.size.y / _app.phone_scale()])
		await _tap(_centre(b))
		await _settle()
		var got := _sig()
		_ok(got == want, "%s: a tap on the row has the desktop bar's effect (%s)" % [k, str(got)])
		_ok(not _drawer().is_open() and _depth() == 0, "%s: the drawer closed and the depth is 0" % k)
		if popup != null:
			(popup as PopupMenu).id_pressed.disconnect(cb)
			_ok(seen == [int(row["_id"])], "%s: the owning popup got exactly one id_pressed with the row's id (%s vs %d)" % [k, str(seen), int(row["_id"])])
	await _reset_effects()
	print("-- a pin whose command is gone")
	DccSettings.quick_bar_add("bar", "menu:99999", "Gone command")
	_app.phone_quick.refresh()
	await _frames(4)
	await _open_by_tap()
	var ghost := _row_for("menu:99999")
	_ok(ghost != null and not bool(ghost.get_meta("live")), "its row is kept and marked not live")
	if ghost != null:
		(ghost.get_parent().get_parent() as ScrollContainer).ensure_control_visible(ghost)
		await _frames(3)
		var t0 := _toasts()
		## The armed-tool strip is drawn above the drawer, so the panel must end above it (found in landscape: the last
		## rows were covered and untouchable). A tool pin was the last thing run, so the strip is up.
		var strip = _app._tool_strip
		_ok(strip != null and strip.visible, "a tool is armed (%s) and its strip is up" % str(_app.armed_tool))
		if strip != null and strip.visible:
			var pb: float = _drawer().get("_panel").get_global_rect().end.y
			_ok(pb <= strip.get_global_rect().position.y + 1.0, "the panel ends above the strip (%.0f vs %.0f)" % [pb, strip.get_global_rect().position.y])
		_ok(_vp.get_visible_rect().encloses(ghost.get_global_rect()), "the gone pin's row is on screen")
		await _tap(_centre(ghost))
		await _settle()
		_ok(_toasts() == t0 + 1, "tapping it toasts (%d -> %d)" % [t0, _toasts()])
		_ok(_drawer().is_open() and _depth() == 1, "and the drawer stays open, depth 1")
	_ok(DccSettings.quick_bar_value("bar").size() == PINS.size() + 1, "the pin is still stored")
	DccSettings.quick_bar_remove("bar", "menu:99999")
	_app.phone_quick.refresh()
	await _frames(4)
	_drawer().close_if_open()
	await _settle()
	_ok(_depth() == 0, "closed: depth 0")

# 6 -----------------------------------------------------------------------------------------------------------

func _find_point() -> Vector2:
	var r := _host.get_global_rect()
	for fy in [0.30, 0.36, 0.24, 0.42, 0.5]:
		for fx in [0.5, 0.4, 0.6]:
			var p := Vector2(r.position.x + r.size.x * fx, r.position.y + r.size.y * fy)
			var navr: Rect2 = _host._navpad.get_global_rect() if _host._navpad != null else Rect2()
			if navr.has_point(p) or _drawer().grip().get_global_rect().grow(8.0).has_point(p):
				continue
			var lp: Vector2 = _ov.get_global_transform().affine_inverse() * p
			var q: Dictionary = _ov._grid_point(lp, _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
			if bool(q.get("valid", false)):
				return p
	return Vector2(-1, -1)

func _t_map() -> void:
	print("== 6. the closed drawer leaves the map alone ==")
	_host.reset_view()
	await _settle()
	var p := _find_point()
	_ok(p.x >= 0.0, "a valid map point away from the grip was found (%s)" % str(p))
	if p.x < 0.0:
		return
	var c0 := _clicks
	await _tap(p)
	await _settle()
	_ok(_clicks == c0 + 1, "a tap on the map still clicks it (positive control: %d -> %d)" % [c0, _clicks])

# 3 again, left-handed ---------------------------------------------------------------------------------------

func _t_hand() -> void:
	print("== 2/3. left-handed ==")
	var d := _drawer()
	DccSettings.set_dominant_hand("left")
	await _settle()
	var band := _band()
	if _app._landscape:
		## Landscape (owner, 2026-11-13): left-handed is the far LEFT, beside the rail (the band's left edge), opening
		## rightward; a real tap and a real drag both open it from there.
		var lg: Rect2 = d.grip().get_global_rect()
		_ok(absf(lg.position.x - band.position.x) <= 1.5 and lg.position.x > 0.0, "landscape, left-handed: the grip is on the band's LEFT edge, beside the rail (x %.1f)" % lg.position.x)
		await _open_by_tap()
		_ok(d.is_open() and absf(d.get("_panel").get_global_rect().position.x - band.position.x) <= 1.5, "a tap opens it from the left edge")
		d.close_if_open()
		await _settle()
		var lc := _centre(d.grip())
		await _drag(lc, lc + Vector2(float(d.get("_w")) * 0.7, 0.0))
		await _settle()
		_ok(d.is_open(), "a rightward drag opens it from the left edge")
		d.close_if_open()
		await _settle()
		DccSettings.set_dominant_hand("right")
		await _settle()
		_ok(absf(d.grip().get_global_rect().end.x - float(_vp.size.x)) <= 1.5, "back to right-handed: the far right edge")
		return
	var gr: Rect2 = d.grip().get_global_rect()
	_ok(absf(gr.end.x - band.end.x) <= 1.5, "the grip moved to the band's RIGHT edge at once (end x %.1f, band end %.1f)" % [gr.end.x, band.end.x])
	_ok(_dp_ok(gr, 44.0, 88.0), "still at least 44 x 88 dp")
	await _open_by_tap()
	_ok(d.is_open() and absf(d.get("_panel").get_global_rect().end.x - band.end.x) <= 1.5, "a tap opens it from the right")
	var c: Vector2 = _centre(d.grip())
	var w: float = d.get("_w")
	await _drag(c, c + Vector2(w * 0.8, 0.0))
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "a drag of the grip back to the right closes it")
	await _drag(_centre(d.grip()), _centre(d.grip()) - Vector2(w * 0.7, 0.0))
	await _settle()
	_ok(d.is_open(), "a leftward drag opens it")
	d.close_if_open()
	await _settle()
	DccSettings.set_dominant_hand("right")
	await _settle()
	_ok(absf(d.grip().get_global_rect().position.x - band.position.x) <= 1.5, "and back to the left edge")

# 8 -----------------------------------------------------------------------------------------------------------

func _t_swap() -> void:
	print("== 8. a class swap with it open ==")
	## A tablet-class host at this density: 1000 x 1100 dp, over the 900 dp shortest-side line (Ruling EG).
	var tab := Vector2i(int(1000.0 * _dens), int(1100.0 * _dens))
	var ph := _vp.size
	var old := _drawer()
	var old_id := old.get_instance_id()
	await _open_by_tap()
	_ok(old.is_open() and _depth() == 1, "open, depth 1")
	_app._remount_composition()
	await _frames(14)
	var d := _drawer()
	_ok(d != null and d.get_instance_id() != old_id and not is_instance_valid(old), "a same-class remount freed the old drawer and built a new one")
	if d == null:
		return
	_ok(d.is_open() and d.get("_panel").visible, "the new one is open, with its panel drawn")
	_ok(absf(d.get("_panel").size.x - minf(roundf(248.0 * _app.phone_scale()), _band().size.x * 0.78)) <= 1.5, "and as wide as before")
	_ok(_depth() == 1, "the depth is 1: the old share was given back and the new drawer took one (got %d)" % _depth())
	_ok(_dp_ok(d.grip().get_global_rect(), 44.0, 88.0), "its grip is still 44 x 88 dp")
	## Away from the panel: landscape's panel is on the right, so the scrim's LEFT part is the free part.
	await _tap(_centre(d.scrim()) + Vector2(d.scrim().size.x * (-0.4 if _app._landscape else 0.4), 0.0))
	await _settle()
	_ok(not d.is_open() and _depth() == 0, "a tap on the new scrim closes it, depth 0")
	await _tap(_centre(d.grip()))
	await _settle()
	_ok(d.is_open() and _depth() == 1, "and the new grip opens it again")
	_vp.size = tab
	await _frames(3)
	_app._remount_composition()
	await _frames(14)
	_ok(String(_app.class_key()) == "tablet", "the tablet class (%s)" % _app.class_key())
	_ok(_drawer() == null, "the tablet has no phone drawer")
	_ok(_depth() == 0, "and the open drawer's share of the pause was given back: depth 0 (got %d)" % _depth())
	_vp.size = ph
	await _frames(3)
	_app._remount_composition()
	await _frames(14)
	_ok(String(_app.class_key()) == "phone", "the phone class again (%s)" % _app.class_key())
	var e := _drawer()
	_ok(e != null and not e.is_open() and _depth() == 0, "the drawer is built again, closed, depth 0")
	if e != null:
		await _tap(_centre(e.grip()))
		await _settle()
		_ok(e.is_open() and _depth() == 1, "and works: a tap opens it, depth 1")
		e.close_if_open()
		await _settle()

# 7 -----------------------------------------------------------------------------------------------------------

func _t_empty_again() -> void:
	print("== 7. empty again, while open ==")
	var d := _drawer()
	await _open_by_tap()
	_ok(d.is_open() and _depth() == 1, "open with depth 1")
	for k in DccSettings.quick_bar_pins_for("bar", "").duplicate():
		DccSettings.quick_bar_remove("bar", String(k["k"]))
	_app.phone_quick.refresh()
	await _frames(6)
	_ok(_app.phone_quick.drawer() == null and not is_instance_valid(d), "removing the last pin freed the open drawer")
	_ok(_depth() == 0, "and gave the depth back (got %d)" % _depth())
	_ok(_app._phone_root.find_child("QuickDrawer", true, false) == null, "no node of that name is left")
