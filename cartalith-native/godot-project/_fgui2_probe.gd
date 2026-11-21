extends Node
## F3/F4 (owner findings 2026-11-21): labels. Windowed, scratch APPDATA. See OUTSTANDING_WORK.md.
## CL-11 (`CITY_MAP_LOOK_SCOPE.md`, Ruling EW item 4: user-placed points of interest) and the owner's 2026-11-14 request to
## keep the old circle markers as an option. Windowed (refuses --headless), under a wiped user directory
## (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _cl11_probe.tscn -- --vp 1600x1000 --out <dir>
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch --out <dir>      (the phone)
##
## Desktop legs. A: the Point of interest tool places one of two kinds through the real click and dialog. B: a left click
## on the pin opens the card with a NOTE section, Create note binds it and the pin turns orange (framebuffer pixels).
## C: Move, Rename, Change kind and Delete from the card, each ONE undo step, the note still bound after a move and a
## rename, an undo of the delete restoring it. D: the world file (`annotations/pois.json` present only when a POI was ever
## placed), a reload keeping everything, an id never reissued. E: Find. F: the City Viewer. G: the marker style: the
## footprint follows the style for a landmark and for a POI, and the setting persists.
## Touch legs (--force-touch): a tap shows chips, a long-press opens the card or the peek sheet with the Note section.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _touch := false
var _checks := 0
var _fails := 0
var _root := ""
var _out := ""
var _ids := {}
var _only := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("FG2 %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("FG2 %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- events ------------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _press_at(at: Vector2, down: bool) -> void:
	if _touch:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = down
		t.position = at
		_vp.push_input(t)
		var em := InputEventMouseButton.new()
		em.button_index = MOUSE_BUTTON_LEFT
		em.pressed = down
		em.position = at
		em.global_position = at
		em.device = InputEvent.DEVICE_ID_EMULATION
		em.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(em)
	else:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		_vp.push_input(mm)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(ev)


func _click_host(at: Vector2) -> void:
	_press_at(at, true)
	await _frames(2)
	_press_at(at, false)
	await _frames(6)


func _click(local: Vector2) -> void:
	await _click_host(_host(local))


func _hold(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await get_tree().create_timer(0.8).timeout
	await _frames(3)
	_press_at(at, false)
	await _frames(8)


func _tap_it(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await _frames(3)
	_press_at(at, false)
	await _frames(10)


# ---- readouts ----------------------------------------------------------------------------------------------------

func _move(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(mm)


func _drag(from_local: Vector2, to_local: Vector2) -> void:
	var a := _host(from_local)
	var b := _host(to_local)
	_press_at(a, true)
	await _frames(2)
	for k in 8:
		_move(a.lerp(b, float(k + 1) / 8.0))
		await _frames(1)
	_press_at(b, false)
	await _frames(6)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


func _scr(cell: Vector2) -> Vector2:
	return _ov._cell_to_screen(cell, _ov._displayed_rect())


# ---- run ---------------------------------------------------------------------------------------------------------

func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--only":
			_only = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			_touch = true
			i += 1
		else:
			i += 1
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	await _run()
	_finish()


func _type(text: String) -> void:
	for ch in text:
		for down in [true, false]:
			var k := InputEventKey.new()
			k.pressed = down
			k.unicode = ch.unicode_at(0)
			k.keycode = ch.unicode_at(0)
			_vp.push_input(k)
		await _frames(1)


func _reveal(c: Control) -> void:
	var sc: Node = c.get_parent()
	while sc != null and not sc is ScrollContainer:
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(c)
	await _frames(4)


func _click_ctrl(c: Control) -> void:
	await _reveal(c)
	await _click_host(c.get_global_rect().get_center())


func _find(root: Node, nm: String) -> Control:
	return root.find_child(nm, true, false) as Control


## A generated label far from every other label and from the plate edge, at a cell the pointer can reach.
func _pick_auto() -> Dictionary:
	var all: Array = _bridge.auto_label_list()
	var best := {}
	var best_d := 0.0
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	for d in all:
		var p: Vector2 = _scr(Vector2(float(d["x"]), float(d["y"])))
		if not interior.grow(-120).has_point(p) or not (app.viewport as Control).get_global_rect().grow(-150).has_point(_host(p)):
			continue
		var nearest := 1e9
		for o in all:
			if o != d:
				nearest = minf(nearest, p.distance_to(_scr(Vector2(float(o["x"]), float(o["y"])))))
		if nearest > best_d:
			best_d = nearest
			best = d
	return best


func _run() -> void:
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	app.arm_tool("inspect")
	await _frames(4)
	if _only == "" or _only == "labels":
		await _leg_user()
		await _leg_auto()
	if (_only == "" or _only == "hover") and not DccTheme.is_touch():
		await _leg_hover()
	if _only == "" or _only == "pin":
		await _leg_pin()
	if _only == "" or _only == "reset":
		await _leg_reset()


func _leg_user() -> void:
	print("== user label drag (F4) ==")
	app.select_domain_category("cartography", "Labels")
	await _frames(6)
	_ok("F4: opening the Labels category arms the Label tool", app.armed_tool == "label", app.armed_tool)
	var idx: int = _bridge.label_create(200.0, 150.0, "Dragme")
	await _frames(4)
	_bridge.label_select(-1)
	app.viewport.refresh_annotations()
	await _frames(4)
	var p0 := _scr(Vector2(200.0, 150.0))
	await _drag(p0, p0 + Vector2(90.0, 40.0))
	var lb: Dictionary = _bridge.label_get(idx)
	_ok("F4: a hand-placed label moves under a real mouse drag", absf(float(lb.get("x", 0.0)) - 200.0) > 5.0, str(lb.get("x")))
	_bridge.label_delete(idx)
	_bridge.label_select(-1)
	await _frames(4)


func _leg_auto() -> void:
	print("== generated label (F3, F4) ==")
	var a: Dictionary = _pick_auto()
	_ok("a generated label is reachable on screen", not a.is_empty(), str(_bridge.auto_label_list().size()))
	if a.is_empty():
		return
	var key := String(a["key"])
	var user_n: int = _bridge.label_list().size()
	var render_n: int = _bridge.labels_render_list().size()
	var x0 := float(a["x"])
	var y0 := float(a["y"])
	_p("picked %s at %s,%s" % [key, x0, y0])
	await _shot("auto_before.png")
	var p0 := _scr(Vector2(x0, y0))
	await _drag(p0, p0 + Vector2(70.0, 35.0))
	var g: Dictionary = _bridge.auto_label_get(key)
	_ok("F4: a generated label moves under a real mouse drag", absf(float(g.get("x", x0)) - x0) > 5.0 and bool(g.get("edited", false)), str(g.get("x")))
	_ok("F3: no second label: render list and hand-placed list unchanged in size",
		_bridge.labels_render_list().size() == render_n and _bridge.label_list().size() == user_n,
		"%d/%d %d/%d" % [_bridge.labels_render_list().size(), render_n, _bridge.label_list().size(), user_n])
	await _shot("auto_moved.png")
	# rename through the form with real key events (the phone's form is in the sheet: the chip's tap raises it)
	if DccTheme.is_phone():
		app.set_phone_detent("full")
		await _frames(30)
	var le := _find(app, "AutoLabelText") as LineEdit
	_ok("F3: the form for the selected generated label is open", le != null and le.is_visible_in_tree())
	if le == null:
		return
	await _frames(4)
	await _click_ctrl(le)
	le.select_all()
	await _type("Zork")
	await _frames(4)
	_ok("F3: typing renames the generated label", String(_bridge.auto_label_get(key).get("text", "")) == "Zork",
		String(_bridge.auto_label_get(key).get("text", "")))
	var found := false
	for d in _bridge.labels_render_list():
		if String(d["text"]) == "Zork" and bool(d["generated"]):
			found = true
	_ok("F3: the renamed label is what the map draws", found)
	# undo steps
	_bridge.undo_last()
	await _frames(3)
	_ok("F3: undo reverts the rename", String(_bridge.auto_label_get(key).get("text", "")) != "Zork")
	# hide
	app.arm_tool("label")
	await _frames(2)
	if DccTheme.is_phone():
		app.set_phone_detent("peek")
		await _frames(30)
	p0 = _scr(Vector2(float(_bridge.auto_label_get(key)["x"]), float(_bridge.auto_label_get(key)["y"])))
	await _click_host(_host(p0))
	if DccTheme.is_phone():
		app.set_phone_detent("full")
		await _frames(30)
	var hb := _find(app, "AutoLabelHide") as Button
	_ok("F3: the Hide button is on screen", hb != null and hb.is_visible_in_tree())
	if hb != null:
		await _click_ctrl(hb)
	var hidden := true
	for d in _bridge.labels_render_list():
		if bool(d["generated"]) and absf(float(d["x"]) - float(_bridge.auto_label_get(key)["x"])) < 0.01 and absf(float(d["y"]) - float(_bridge.auto_label_get(key)["y"])) < 0.01:
			hidden = false
	_ok("F3: a hidden generated label is not drawn", hidden and bool(_bridge.auto_label_get(key).get("hidden", false)))
	await _shot("auto_hidden.png")
	# save and reload keeps the edit
	var path := "C:/cw/fgui2_scratch/auto.cartalith"
	_ok("the world saves", _bridge.save_project(path))
	_bridge.auto_label_reset_all()
	_ok("reset-all clears the override", not bool(_bridge.auto_label_get(key).get("edited", true)))
	_ok("the world reloads", _bridge.load_save(path))
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(20)
	_ok("F3: a hidden, moved generated label survives save and reload",
		bool(_bridge.auto_label_get(key).get("hidden", false)) and absf(float(_bridge.auto_label_get(key).get("x", x0)) - x0) > 5.0,
		str(_bridge.auto_label_get(key)))
	# the changed-labels list is the way back to a hidden label: its restore button
	var back: Button = null
	for b in app.find_children("*", "Button", true, false):
		if (b as Button).text == "↺" and (b as Button).is_visible_in_tree():
			back = b
	_ok("F3: the changed-labels list offers a restore for the hidden label", back != null)
	if back != null:
		back.get_parent().get_parent()
		var sc = back.get_parent()
		while sc != null and not sc is ScrollContainer:
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(back)
		await _frames(4)
		await _click_ctrl(back)
		_ok("F3: restore returns the generator's own label", not bool(_bridge.auto_label_get(key).get("edited", true)))
	await _shot("auto_restored.png")


func _pref_popup() -> PopupMenu:
	for pm in app.find_children("*", "PopupMenu", true, false):
		if (pm as PopupMenu).get_item_index(905) >= 0:
			return pm
	return null


func _leg_reset() -> void:
	print("== F40: Preferences > Reset to default settings ==")
	DccSettings.set_units_mode("mi")
	DccSettings.set_gpu_enabled(true)
	DccSettings.set_storage_root("worlds", "C:/cw/fgui2_scratch/keep_worlds")
	DccSettings.set_llm_value("model", "my-model")
	DccSettings.set_llm_value("key_account", "my-key-entry")
	DccSettings.set_shortcut_binding("menu", 4242, KEY_F9)
	var pp := _pref_popup()
	_ok("F40: Preferences has a Reset to default settings row", pp != null and pp.get_item_text(pp.get_item_index(905)).begins_with("Reset to default settings"))
	if pp == null:
		return
	pp.id_pressed.emit(905)
	await _frames(8)
	var dlg: ConfirmationDialog = null
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		if (d as ConfirmationDialog).visible and (d as ConfirmationDialog).title.contains("Reset to default"):
			dlg = d
	_ok("F40: a confirmation opens first and says what is reset and kept", dlg != null)
	if dlg == null:
		return
	var txt := ""
	for l in dlg.find_children("*", "Label", true, false):
		txt += (l as Label).text + " "
	_ok("F40: the text names shortcuts, worlds, vault and keys", txt.contains("shortcuts") and txt.contains("worlds") and txt.contains("vault") and txt.contains("keys"), txt.left(120))
	_ok("F40: nothing changed before OK", DccSettings.units_mode() == "mi")
	await _shot("reset_dialog.png")
	var okb := dlg.get_ok_button()
	var at := okb.get_screen_position() + okb.size * 0.5
	var before := FileAccess.get_file_as_string(DccSettings.CONFIG_PATH)
	_p("dialog at %s ok at %s" % [dlg.position, at])
	await _click_host(at)
	await _frames(6)
	_ok("F40: units back to the default", DccSettings.units_mode() != "mi", DccSettings.units_mode())
	_ok("F40: GPU choice forgotten", DccSettings.gpu_enabled() == -1)
	_ok("F40: the shortcut binding is back to its default", DccSettings.shortcut_binding("menu", 4242, KEY_F1) == KEY_F1)
	_ok("F40: the assistant's model is reset", String(DccSettings.llm_value("model")) == "")
	_ok("F40: the assistant's key name stays", String(DccSettings.llm_value("key_account")) == "my-key-entry")
	_ok("F40: the storage location stays", DccSettings.storage_root("worlds") == "C:/cw/fgui2_scratch/keep_worlds")
	_ok("F40: the old file is kept beside it, byte for byte", FileAccess.get_file_as_string(DccSettings.RESET_BACKUP_PATH) == before)
	var cf := ConfigFile.new()
	_ok("F40: the new file parses and holds no reset section", cf.load(DccSettings.CONFIG_PATH) == OK and not cf.has_section("gpu") and not cf.has_section("units"))
	# restoring the backup is the undo
	DirAccess.copy_absolute(DccSettings.RESET_BACKUP_PATH, DccSettings.CONFIG_PATH)
	var cf2 := ConfigFile.new()
	_ok("F40: copying the backup back restores the old settings", cf2.load(DccSettings.CONFIG_PATH) == OK and String(cf2.get_value("units", "mode", "")) == "mi", str(cf2.get_sections()))


func _leg_hover() -> void:
	print("== F14: the settlement hover card ==")
	app.arm_tool("inspect")
	await _frames(4)
	var rect: Rect2 = _ov._displayed_rect()
	var sl: Array = _bridge.settlements()
	var picked := {}
	for sd in sl:
		var p: Vector2 = _scr(Vector2(float(sd["x"]), float(sd["y"])))
		if _ov._interior_rect(rect).grow(-150).has_point(p):
			picked = sd
			if int(sd.get("population", 0)) > 3000:
				break
	_ok("a settlement is on screen", not picked.is_empty())
	if picked.is_empty():
		return
	var at := _host(_scr(Vector2(float(picked["x"]), float(picked["y"]))))
	for z in [1.0, 6.0]:
		if z > 1.0:
			app.viewport._zoom_at(at, z)
			await _frames(20)
			app.viewport.move_view_to(float(picked["x"]), float(picked["y"]))
			await _frames(30)
			at = _host(_scr(Vector2(float(picked["x"]), float(picked["y"]))))
			_p("settlement now at %s" % at)
		_vp.push_input(_mm(at + Vector2(3, 3)))
		await _frames(3)
		_vp.push_input(_mm(at))
		await _frames(8)
		_p("zoom %s hover index %s name %s" % [app.viewport.zoom(), _ov._hover_index, picked["name"]])
		await _shot("hover_%d.png" % int(z))
		var cr: Rect2 = _ov.last_card_rect
		_p("card rect %s (screen size %s)" % [cr, cr.size * app.viewport.zoom()])
		_ids["card_%d" % int(z)] = cr.size * app.viewport.zoom()
	_ok("the hover card is up", _ov._hover_index >= 0)
	var c1: Vector2 = _ids.get("card_1", Vector2.ZERO)
	var c6: Vector2 = _ids.get("card_6", Vector2.ZERO)
	_ok("F14: the card is the same size on screen at zoom 1 and 6 (it was x zoom)", c1.x > 0.0 and absf(c6.x - c1.x) < 2.0 and absf(c6.y - c1.y) < 2.0, "%s vs %s" % [c1, c6])
	_ok("F14: the card sizes to its content: between 140 and 380 px wide, and as tall as its lines", c1.x >= 140.0 and c1.x <= 380.0 and c1.y >= 40.0, str(c1))
	var hs: Dictionary = _ov._settlements[_ov._hover_index]
	hs["name"] = "A"
	_ov.queue_redraw()
	await _frames(6)
	var short_w: float = _ov.last_card_rect.size.x * app.viewport.zoom()
	_ok("F14: a very short card is not narrower than its 140 px floor (plus padding)", short_w >= 155.0, str(short_w))
	hs["name"] = "A very long settlement name that goes on and on and on and on and on and on"
	_ov.queue_redraw()
	await _frames(6)
	await _shot("hover_long.png")
	var cl: Rect2 = _ov.last_card_rect
	_ok("F14: a long line wraps within 380 px and makes the card taller", cl.size.x * app.viewport.zoom() <= 396.5 and cl.size.y * app.viewport.zoom() > c1.y + 10.0, str(cl.size * app.viewport.zoom()))



func _mm(at: Vector2) -> InputEventMouseMotion:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	return mm


func _rclick(at: Vector2) -> void:
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_RIGHT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_RIGHT if down else 0
		_vp.push_input(ev)
		await _frames(2)
	await _frames(8)


func _leg_pin() -> void:
	print("== F35: the sample pin ==")
	app.arm_tool("inspect")
	app.viewport.reset_view()
	await _frames(30)
	var rect: Rect2 = _ov._displayed_rect()
	var spot: Vector2 = _ov._interior_rect(rect).get_center() + Vector2(37.0, 23.0)
	if DccTheme.is_phone():
		app.set_phone_detent("peek")
		app.arm_tool("inspect")
		await _frames(30)
		var at := _host(spot)
		_press_at(at, true)
		await get_tree().create_timer(0.9).timeout
		await _frames(3)
		_press_at(at, false)
		await _frames(10)
		_ok("the phone's long-press drops a sample pin (what the phone has)", _ov.has_sample_pin())
		await _shot("pin_phone.png")
		return
	if DccTheme.is_touch():
		_p("tablet: the hold opens the cartwheel, no sample pin by design; desktop mouse path not applicable")
		return
	await _rclick(_host(spot))
	_ok("F35: a desktop right-click drops a sample pin", _ov.has_sample_pin())
	var pin: Dictionary = _ov.sample_pin_grid()
	_ok("F35: ...at the clicked cell", not pin.is_empty() and _scr(Vector2(float(pin["gx"]), float(pin["gy"]))).distance_to(spot) < 1.5, str(pin))
	var c = app.context_broker.card
	_ok("F35: ...and the card is open beside it", c != null and c.visible)
	await _shot("pin_desktop.png")
	var k := InputEventKey.new()
	k.pressed = true
	k.keycode = KEY_ESCAPE
	_vp.push_input(k)
	await _frames(8)
	if c != null and c.visible:
		c.hide()
		await _frames(4)
	_ok("F35: closing the card takes the pin away", not _ov.has_sample_pin())
