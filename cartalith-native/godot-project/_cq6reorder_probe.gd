extends Node
## CQ-6's long-press reorder of quick-bar pins (`CUSTOM_QUICK_BAR_SCOPE.md` CQ-6; `shell/quick_pin_reorder.gd`, the lift in
## `DccQuickBar._long_press_input` and `QuickDrawer._row_input`). Driven with REAL touch events (an `InputEventScreenTouch` /
## `ScreenDrag` and the emulated mouse event Godot derives from each, `device = DEVICE_ID_EMULATION`) pushed at a `--vp`
## host SubViewport against the real shell. Windowed (refuses --headless), scratch user dir (`APPDATA=<empty dir>`).
##
##   godot --path . _cq6reorder_probe.tscn -- --vp 1600x900 --out <dir>                                (desktop)
##   ... -- --vp 1700x1000 --density=1.0 --force-touch --out <dir>                                    (tablet)
##   ... -- --vp 1080x2340 --density 2.625 --force-touch --out <dir>                                  (phone)
##
## What it protects (orders are literals written here; the disk read is the saved `[quick_bar]` section, not the in-memory store):
##   D  desktop: a held MOUSE button (not a finger) lifts nothing and reorders nothing; the Customise window's own path (the order
##      function it shares, `quick_bar_move_visible_to`) moves a pin to an index and refuses a no-op and an out-of-range index;
##   T  tablet: a finger resting 0.55 s on a pin lifts it (dimmed, a drop marker on the bar, the pin NOT run); dragging right past
##      the other pins and lifting writes the new order to the store AND to the saved settings and redraws the chips in it; the same
##      leftwards; a lift dropped back in its own place changes nothing and opens no menu; a rest that never moves opens the pin's menu
##      on the lift (CQ-5, kept); after every release no pin is dimmed and no marker remains; a held mouse lifts nothing;
##   P  phone: the same on the drawer's rows (vertical), and with 24 pins a lifted row dragged to the panel's bottom edge scrolls the
##      list and drops past the last visible row.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const SEED := 483921
const HOLD_S := 0.55

var _fails := 0
var _checks := 0
var _out := ""
var _vp: SubViewport
var _app: Node
var _bridge
var _dens := 1.0
var _keys: Array = []
var _vault_key := "menu:%d" % DccMenus.ID_VAULT
var _planner_key := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _frames(n: int = 3) -> void:
	for _i in n:
		await get_tree().process_frame


func _wait(s: float) -> void:
	await get_tree().create_timer(s).timeout


func _shot(tag: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var p := _out.path_join("cq6_%s.png" % tag)
	_vp.get_texture().get_image().save_png(p)
	print("SHOT %s" % p)


func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--out"]:
			i += 2
		elif a == "--density":
			i += 2
		elif a.begins_with("--density="):
			i += 1
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	var d := _arg("--density", "")
	for a2 in args:
		if String(a2).begins_with("--density="):
			d = String(a2).substr(10)
	_dens = float(d) if d != "" else 1.0
	_out = _arg("--out", "")
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed")
		get_tree().quit(2)
		return
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	Input.emulate_touch_from_mouse = true
	await _run(int(parts[0]), int(parts[1]))
	if _checks == 0:
		_fails += 1
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- the finger -------------------------------------------------------------------------------------------------

func _px(p: Vector2) -> Vector2:
	return _vp.get_final_transform() * p


func _centre(c: Control) -> Vector2:
	var w := c.get_window()
	var p := c.get_global_transform_with_canvas() * (c.size * 0.5)
	if w != null and w != get_tree().root and w != _vp and w.is_embedded():
		return Vector2(w.position) + w.get_final_transform() * p
	return p


func _down(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_vp.push_input(t, false)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m, false)
	await _frames(2)


func _drag(p_canvas: Vector2, prev_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var prev := _px(prev_canvas)
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - prev
	_vp.push_input(d, false)
	var m := InputEventMouseMotion.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.position = p
	m.global_position = p
	m.relative = p - prev
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m, false)


func _up(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_vp.push_input(t, false)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_vp.push_input(m, false)
	await _frames(3)


func _slide(a: Vector2, b: Vector2, steps: int) -> void:
	var prev := a
	for k in range(1, steps + 1):
		var p := a.lerp(b, float(k) / float(steps))
		_drag(p, prev)
		prev = p
		await _frames(2)


## A mouse (NOT an emulated finger: device 0) press, a drag of `by` canvas units, and a lift.
func _mouse_hold_drag(p_canvas: Vector2, by: Vector2, secs: float) -> void:
	var p := _px(p_canvas)
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m, false)
	await _wait(secs)
	var q := _px(p_canvas + by)
	var mv := InputEventMouseMotion.new()
	mv.position = q
	mv.global_position = q
	mv.relative = q - p
	mv.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(mv, false)
	await _frames(2)
	var u := m.duplicate() as InputEventMouseButton
	u.pressed = false
	u.button_mask = 0
	u.position = q
	u.global_position = q
	_vp.push_input(u, false)
	await _frames(3)


# -- reads --------------------------------------------------------------------------------------------------------

func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _store(keys: Array) -> void:
	var idx := _idx()
	for k in DccSettings.quick_bar_value("bar").duplicate():
		DccSettings.quick_bar_remove("bar", String(k["k"]))
	for k in keys:
		var row := idx.find_pin(String(k))
		DccSettings.quick_bar_add("bar", String(k), String(row.get("title", "ghost")))
	_refresh()


func _refresh() -> void:
	if _app._quick_bar != null:
		_app._quick_bar.refresh()
	if _app.phone_quick != null:
		_app.phone_quick.refresh()


func _order() -> Array:
	var out: Array = []
	for p in DccSettings.quick_bar_value("bar"):
		out.append(String(p["k"]))
	return out


## The `bar` list as SAVED: a fresh `ConfigFile` read of the settings file, not the in-memory copy.
func _disk_order() -> Array:
	var cf := ConfigFile.new()
	var out: Array = []
	if cf.load(DccSettings.CONFIG_PATH) != OK:
		return out
	for p in cf.get_value("quick_bar", "bar", []):
		out.append(String((p as Dictionary).get("k", "")))
	return out


func _chips() -> Array:
	var out: Array = []
	for c in _app._quick_bar.get_children():
		if c is Button and c.has_meta("pin_key") and (c as Button).visible and c.is_visible_in_tree():
			out.append(c)
	return out


func _chip_keys() -> Array:
	var out: Array = []
	for c in _chips():
		out.append(String(c.get_meta("pin_key")))
	return out


func _chip(key: String) -> Button:
	for c in _chips():
		if String(c.get_meta("pin_key")) == key:
			return c
	return null


func _clean(host: Node, tag: String) -> void:
	_ok(host.find_child("ReorderMarker", true, false) == null, "%s: no drop marker is left behind" % tag)


func _setup_keys() -> void:
	for r in _idx().all():
		if bool(r["pinnable"]):
			_keys.append(String(r["pin_key"]))


# -- the run ----------------------------------------------------------------------------------------------------

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _wait(1.6)
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(6)
	_bridge = _app.bridge
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await _wait(0.25)
	await _wait(1.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	_setup_keys()
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== %dx%d density %.4f: %s ==" % [vw, vh, _dens, form])
	_ok(_keys.has(_vault_key) and _keys.has(_planner_key), "the vault and planner pins are pinnable")
	if DccTheme.is_phone():
		await _phone()
	elif DccTheme.is_tablet():
		await _tablet()
	else:
		await _desktop()


# -- D: desktop -------------------------------------------------------------------------------------------------

func _desktop() -> void:
	print("== D. desktop: the mouse lifts nothing; the shared order function ==")
	var tools: Array = _keys.filter(func(k): return String(k).begins_with("tool:"))
	var four := [_vault_key, _planner_key, tools[0], tools[1]]
	_store(four)
	await _frames(6)
	_ok(_order() == four, "stored four pins in order: %s" % str(_order()))
	var chip := _chip(String(four[1]))
	_ok(chip != null, "the second pin has a chip")
	if chip == null:
		return
	await _mouse_hold_drag(_centre(chip), Vector2(200, 0), HOLD_S)
	_ok(_order() == four, "a held mouse dragged 200 px reorders nothing: %s" % str(_order()))
	_ok(_app._quick_bar._reorder == null or not _app._quick_bar._reorder.active, "and lifts nothing")
	_clean(_app._quick_bar, "D")
	_ok(DccSettings.quick_bar_move_visible_to("bar", String(four[0]), 2, ""), "the order function moves pin 0 to index 2")
	_ok(_order() == [four[1], four[2], four[0], four[3]], "... giving [planner, t0, vault, t1]: %s" % str(_order()))
	_ok(not DccSettings.quick_bar_move_visible_to("bar", String(four[0]), 2, ""), "a move to the pin's own index is refused (nothing written)")
	_ok(not DccSettings.quick_bar_move_visible_to("bar", String(four[0]), 9, "") and not DccSettings.quick_bar_move_visible_to("bar", "nope", 1, ""),
		"an out-of-range index and an unknown pin are refused")
	_ok(DccSettings.quick_bar_move_visible_to("bar", String(four[3]), 0, "") and _order() == [four[3], four[1], four[2], four[0]],
		"moving the last pin to index 0 gives [t1, planner, t0, vault]: %s" % str(_order()))
	await _shot("D_desktop")


# -- T: tablet ---------------------------------------------------------------------------------------------------

func _tablet() -> void:
	print("== T. tablet: long-press, drag, drop on the chip row ==")
	var tools: Array = _keys.filter(func(k): return String(k).begins_with("tool:"))
	var a: String = _vault_key
	var b: String = _planner_key
	var c: String = tools[0]
	var d: String = tools[1]
	_store([a, b, c, d])
	await _frames(8)
	_ok(_chip_keys() == [a, b, c, d], "four pin chips shown in stored order: %s" % str(_chip_keys()))
	var bar: DccQuickBar = _app._quick_bar
	var menu: PopupMenu = bar._chip_menu
	# T1: lift B, drag right past D, drop.
	var cb := _chip(b)
	var p0 := _centre(cb)
	await _down(p0)
	await _wait(HOLD_S)
	_ok(bar._reorder != null and bar._reorder.active and bar._reorder.item() == cb, "a finger resting 0.55 s lifts the planner pin")
	_ok(cb.modulate.a < 0.9, "the lifted chip is dimmed (alpha %.2f)" % cb.modulate.a)
	_ok(bar.find_child("ReorderMarker", true, false) != null, "a drop marker is on the bar")
	_ok(not menu.visible, "its menu has not opened yet")
	var dr := _centre(_chip(d))
	await _slide(p0, Vector2(dr.x + 30.0, p0.y), 6)
	_ok(bar._reorder.target() == 3, "dragged past the last pin the drop index is 3 (got %d)" % bar._reorder.target())
	var mk := bar.find_child("ReorderMarker", true, false) as Control
	var dend := _chip(d).get_global_rect().end.x
	_ok(mk != null and mk.global_position.x >= dend - 2.0, "the marker sits after the last pin (x %.1f vs end %.1f)" % [mk.global_position.x if mk else -1.0, dend])
	await _shot("T_lifted")
	await _up(Vector2(dr.x + 30.0, p0.y))
	await _frames(6)
	_ok(_order() == [a, c, d, b], "dropping writes [vault, t0, t1, planner]: %s" % str(_order()))
	_ok(_disk_order() == [a, c, d, b], "and the SAVED settings hold the same order: %s" % str(_disk_order()))
	_ok(_chip_keys() == [a, c, d, b], "the chips are redrawn in it: %s" % str(_chip_keys()))
	_ok(not menu.visible, "a drop opens no menu")
	_clean(bar, "T1")
	for ch in _chips():
		_ok(is_equal_approx(ch.modulate.a, 1.0), "chip %s is not dimmed after the drop" % String(ch.get_meta("pin_key")))
	# T2: lift D (index 2), drag left past A, drop.
	var cd := _chip(d)
	var q0 := _centre(cd)
	await _down(q0)
	await _wait(HOLD_S)
	var ca := _centre(_chip(a))
	await _slide(q0, Vector2(ca.x - 30.0, q0.y), 6)
	_ok(bar._reorder.target() == 0, "dragged left of the first pin the drop index is 0 (got %d)" % bar._reorder.target())
	await _up(Vector2(ca.x - 30.0, q0.y))
	await _frames(6)
	_ok(_order() == [d, a, c, b] and _disk_order() == [d, a, c, b], "dropping left writes [t1, vault, t0, planner]: %s / disk %s" % [str(_order()), str(_disk_order())])
	_clean(bar, "T2")
	# T3: lift and drop back in the same place after wandering: nothing changes, no menu.
	var cc := _chip(c)
	var r0 := _centre(cc)
	await _down(r0)
	await _wait(HOLD_S)
	await _slide(r0, r0 + Vector2(90, 0), 3)
	await _slide(r0 + Vector2(90, 0), r0, 3)
	await _up(r0)
	await _frames(4)
	_ok(_order() == [d, a, c, b], "a lift wandered away and dropped back in place changes nothing: %s" % str(_order()))
	_ok(not menu.visible, "and opens no menu (the finger travelled)")
	_ok(is_instance_valid(cc) and is_equal_approx(cc.modulate.a, 1.0), "the dropped-in-place chip is not left dimmed (alpha %.2f)" % cc.modulate.a)
	_clean(bar, "T3")
	# T4: a rest that never moves opens the menu on the lift (CQ-5 kept).
	var ce := _chip(a)
	var s0 := _centre(ce)
	await _down(s0)
	await _wait(HOLD_S)
	_ok(not menu.visible and bar._reorder.active, "resting lifts the pin; the menu waits")
	await _up(s0)
	await _wait(0.3)
	_ok(menu.visible, "a rest lifted without moving opens the pin's menu")
	_ok(not (_app.vault_window != null and _app.vault_window.visible), "and the pin (the vault) did not run")
	_ok(_order() == [d, a, c, b], "and reorders nothing")
	_ok(is_equal_approx(ce.modulate.a, 1.0), "the chip is not left dimmed after the menu-opening lift (alpha %.2f)" % ce.modulate.a)
	menu.hide()
	await _frames(3)
	_clean(bar, "T4")
	# T5: a held mouse (not a finger) lifts nothing.
	var cm := _chip(c)
	await _mouse_hold_drag(_centre(cm), Vector2(150, 0), HOLD_S)
	_ok(_order() == [d, a, c, b], "a held mouse dragged 150 px reorders nothing: %s" % str(_order()))
	_ok(not bar._reorder.active, "and lifts nothing")
	_clean(bar, "T5")


# -- P: phone ----------------------------------------------------------------------------------------------------

func _phone() -> void:
	print("== P. phone: long-press, drag, drop on the drawer's rows ==")
	var tools: Array = _keys.filter(func(k): return String(k).begins_with("tool:"))
	var a: String = _vault_key
	var b: String = _planner_key
	var c: String = tools[0]
	var d: String = tools[1]
	_app._pick_phone_tab("map")
	_app.set_phone_detent("peek")
	await _wait(0.4)
	_store([a, b, c, d])
	await _frames(6)
	var dr = _app.phone_quick.drawer()
	_ok(dr != null, "the drawer is mounted with four pins")
	if dr == null:
		return
	dr.open_now()
	await _wait(0.3)
	var rows := func() -> Array:
		var out: Array = []
		for r in dr.rows():
			if r.has_meta("entry") and String((r.get_meta("entry") as Dictionary)["kind"]) == "pin":
				out.append(r)
		return out
	var row_keys := func() -> Array:
		var out: Array = []
		for r in rows.call():
			out.append(String((r.get_meta("entry") as Dictionary)["key"]))
		return out
	_ok(row_keys.call() == [a, b, c, d], "four pin rows in stored order: %s" % str(row_keys.call()))
	var rb: Button = rows.call()[1]
	var p0 := _centre(rb)
	await _down(p0)
	await _wait(HOLD_S)
	_ok(dr._reorder != null and dr._reorder.active and dr._reorder.item() == rb, "a finger resting 0.55 s lifts the planner row")
	_ok(rb.modulate.a < 0.9, "the lifted row is dimmed (alpha %.2f)" % rb.modulate.a)
	_ok(dr.find_child("ReorderMarker", true, false) != null, "a drop marker is in the drawer")
	var last := _centre(rows.call()[3])
	await _slide(p0, Vector2(p0.x, last.y + 30.0), 6)
	_ok(dr._reorder.target() == 3, "dragged below the last row the drop index is 3 (got %d)" % dr._reorder.target())
	await _shot("P_lifted")
	await _up(Vector2(p0.x, last.y + 30.0))
	await _frames(6)
	_ok(_order() == [a, c, d, b] and _disk_order() == [a, c, d, b], "dropping writes [vault, t0, t1, planner]: %s / disk %s" % [str(_order()), str(_disk_order())])
	dr = _app.phone_quick.drawer()
	_ok(row_keys.call() == [a, c, d, b], "the rows are redrawn in it: %s" % str(row_keys.call()))
	_clean(dr, "P1")
	_ok(dr.is_open(), "the drawer stays open after a reorder")
	# upward
	var rd: Button = rows.call()[2]
	var q0 := _centre(rd)
	var first := _centre(rows.call()[0])
	await _down(q0)
	await _wait(HOLD_S)
	await _slide(q0, Vector2(q0.x, first.y - 30.0), 6)
	_ok(dr._reorder.target() == 0, "dragged above the first row the drop index is 0 (got %d)" % dr._reorder.target())
	await _up(Vector2(q0.x, first.y - 30.0))
	await _frames(6)
	_ok(_order() == [d, a, c, b] and _disk_order() == [d, a, c, b], "dropping upward writes [t1, vault, t0, planner]: %s / disk %s" % [str(_order()), str(_disk_order())])
	_clean(_app.phone_quick.drawer(), "P2")
	# a still rest and lift: nothing changes
	dr = _app.phone_quick.drawer()
	var rc: Button = rows.call()[2]
	var s0 := _centre(rc)
	await _down(s0)
	await _wait(HOLD_S)
	await _up(s0)
	await _frames(4)
	_ok(_order() == [d, a, c, b], "a rest and lift in place changes nothing: %s" % str(_order()))
	_clean(dr, "P3")
	_ok(dr.is_open(), "and the pin did not run and close the drawer")
	_ok(is_equal_approx(rc.modulate.a, 1.0), "the row is not left dimmed after the rest (alpha %.2f)" % rc.modulate.a)
	# 24 pins: edge scroll
	_store(_keys.filter(func(k): return not String(k).begins_with("extra:")).slice(0, 24))
	await _frames(8)
	dr = _app.phone_quick.drawer()
	dr.open_now()
	await _wait(0.3)
	var all: Array = _order()
	_ok(all.size() >= 12, "%d pins are stored (enough to overflow the drawer)" % all.size())
	var scroll: ScrollContainer = dr._scroll
	_ok(scroll.get_v_scroll_bar().max_value > scroll.get_v_scroll_bar().page, "the list overflows the panel")
	var r0: Button = rows.call()[0]
	var f0 := _centre(r0)
	var pr: Rect2 = dr._panel.get_global_rect()
	await _down(f0)
	await _wait(HOLD_S)
	var bottom := Vector2(f0.x, pr.end.y - 6.0)
	var before := scroll.scroll_vertical
	await _slide(f0, bottom, 4)
	for _k in 20:
		_drag(bottom, bottom - Vector2(0, 1))
		await _frames(1)
		_drag(bottom - Vector2(0, 1), bottom)
		await _frames(1)
	_ok(scroll.scroll_vertical > before + 40, "holding a lifted row at the bottom edge scrolls the list (%d -> %d)" % [before, scroll.scroll_vertical])
	await _up(bottom)
	await _frames(6)
	var now := _order()
	_ok(now.size() == all.size() and now[0] != all[0] and now.find(all[0]) >= 3, "the first pin was dropped further down: index %d" % now.find(all[0]))
	_ok(_disk_order() == now, "and it is saved")
	await _shot("P_scrolled")
