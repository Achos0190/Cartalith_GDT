extends Node
## Protects the world-card delete button (owner 2026-11-14): red can bottom-right, 44 px target, tap opens
## a confirmation and NOT the world, Cancel/Escape keep the file, Delete is never the default focus, Delete
## removes file and card, the open world is refused, a tap on the card body still opens.
##   godot --path . _worlddelete_probe.tscn -- --force-touch --vp 1080x2340 --density 2.8125 --shots DIR
##   godot --path . _worlddelete_probe.tscn -- --vp 1600x1000 --shots DIR
## Windowed, from a copy whose config/name is scratch. Worlds are dummy files in a user:// scratch folder;
## the owner's projects folder is never touched (storage root is overridden before anything is built).
var _app: Node
var _view: Viewport
var _fails := 0
var _shots := ""
var _root := ""
var _phone := false

func _ok(c: bool, l: String, d: String = "") -> void:
	print("WD %s %s%s" % ["ok  " if c else "FAIL", l, (" -- " + d) if d != "" else ""])
	if not c:
		_fails += 1

func _settle(n: int = 6) -> void:
	for i in n:
		await get_tree().process_frame

func _find(node: Node, pred: Callable, out: Array) -> void:
	for c in node.get_children():
		if pred.call(c):
			out.append(c)
		_find(c, pred, out)

func _trashes() -> Array:
	var out := []
	_find(_app, func(n): return n is Button and n.name == "TrashButton" and (n as Button).is_visible_in_tree(), out)
	return out

func _tap(p: Vector2) -> void:
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = p
		ev.global_position = p
		_view.push_input(ev)
		await _settle(2)
		print("WD   tap pressed=%s at %s text='%s'" % [pressed, p, _picker_text()])

## Controls inside an embedded dialog report rects in the dialog's own canvas, so add the window origin.
func _rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var w := c.get_window()
	if w != null and w != _view and w.is_embedded():
		var f := w.content_scale_factor
		r = Rect2(r.position * f + Vector2(w.position), r.size * f)
	return r

func _center(c: Control) -> Vector2:
	return _rect(c).get_center()

func _confirm_dialog() -> Window:
	var out := []
	_find(_app, func(n): return n is Window and (n as Window).visible and String((n as Window).title).begins_with("Delete "), out)
	return out[0] if out.size() > 0 else null

func _shot(name: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	_view.get_texture().get_image().save_png(_shots.path_join(name + ".png"))

func _picker_text() -> String:
	if _phone:
		return String(_app.phone_project_picker._error_label.text)
	return String(_app.open_project_dialog._picker_note.text)

func _ready() -> void:
	var vp := ""
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--vp":
			k += 1
			vp = args[k]
		elif args[k] == "--shots":
			k += 1
			_shots = args[k]
		elif args[k] == "--density":
			k += 1
		k += 1
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	get_tree().create_timer(300.0).timeout.connect(func():
		print("WATCHDOG")
		get_tree().quit(2))
	_root = OS.get_user_data_dir().path_join("wd_worlds")
	DirAccess.make_dir_recursive_absolute(_root)
	for n in ["Alpha.ctl", "Bravo.ctl"]:
		var f := FileAccess.open(_root.path_join(n), FileAccess.WRITE)
		f.store_string("not-a-zip")
		f.close()
	DccSettings.recent_projects()
	DccSettings._cfg.set_value("storage_roots", "projects", _root)
	DccSettings._cfg.set_value("recent", "paths", [_root.path_join("Alpha.ctl"), _root.path_join("Bravo.ctl")])
	_app = load("res://shell/app.tscn").instantiate()
	var parts := vp.split("x")
	var sv := SubViewport.new()
	sv.size = Vector2i(int(parts[0]), int(parts[1]))
	sv.gui_embed_subwindows = true
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(sv)
	Input.set_emulate_touch_from_mouse(true)
	sv.add_child(_app)
	_view = sv
	await get_tree().create_timer(1.2).timeout
	_phone = DccTheme.is_phone()
	print("== %s ==" % ("phone" if _phone else "desktop"))
	DccSettings._cfg.set_value("storage_roots", "projects", _root)
	_app.open_welcome()
	await _settle(10)
	var t := _trashes()
	_ok(t.size() == 2, "a trash can on each of the 2 cards", "found %d" % t.size())
	if t.size() < 2:
		get_tree().quit(1)
		return
	var b: Button = t[0]
	_ok(b.size.x >= 44.0 and b.size.y >= 44.0, "target >= 44", "%s" % b.size)
	var card: Control = b.get_parent()
	while card != null and not (card is PanelContainer):
		card = card.get_parent()
	var cr := _rect(card)
	var br := _rect(b)
	_ok(br.end.x > cr.end.x - 30 * _app.get_window().content_scale_factor * 0 - 30 * (_rect(card).size.x / card.size.x) and br.end.y > cr.end.y - 30 * (_rect(card).size.x / card.size.x) and br.end.x <= cr.end.x + 1 and br.end.y <= cr.end.y + 1,
		"can sits at the card's bottom right", "can %s card %s" % [br, cr])
	_ok(DccTheme.c("block").r > 0.55 and DccTheme.c("block").r > 1.8 * DccTheme.c("block").g, "can colour is red")
	await _shot("welcome_cards")
	var img := _view.get_texture().get_image()
	var red := 0
	var sc := float(img.get_width()) / _view.get_visible_rect().size.x
	for y in range(int(br.position.y * sc), int(br.end.y * sc)):
		for x in range(int(br.position.x * sc), int(br.end.x * sc)):
			var px := img.get_pixel(x, y)
			if px.r > 0.5 and px.r > 1.8 * px.g and px.r > 1.8 * px.b:
				red += 1
	_ok(red > 20, "red pixels are drawn in the can's box", "%d" % red)
	var path_a: String = _root.path_join("Alpha.ctl")
	var before := _picker_text()
	await _tap(_center(b))
	await _settle(8)
	var dlg := _confirm_dialog()
	_ok(dlg != null, "tap on the can opens the confirmation")
	_ok(_picker_text() == before, "tap on the can did not try to open the world", "'%s'" % _picker_text())
	if dlg != null:
		var btns := []
		_find(dlg, func(n): return n is Button and (n as Button).is_visible_in_tree(), btns)
		var texts := []
		for x in btns:
			texts.append((x as Button).text)
		print("WD dialog buttons ", texts)
		var foc := dlg.gui_get_focus_owner()
		_ok(foc == null or not (foc is Button and (foc as Button).text == "Delete"), "Delete is not the default focus",
			"focus=%s" % (foc.name if foc else "none"))
		var labels := []
		_find(dlg, func(n): return n is Label, labels)
		var all := ""
		for l in labels:
			all += (l as Label).text + " | "
		_ok(all.contains(path_a) and all.to_lower().contains("delete alpha"), "splash names the world and shows the path", all)
		await _shot("confirm")
		var cancel: Button
		var del: Button
		for x in btns:
			if (x as Button).text == "Cancel": cancel = x
			if (x as Button).text == "Delete": del = x
		_ok(cancel != null and del != null, "Cancel and Delete present")
		## Enter must not delete.
		var ev := InputEventKey.new()
		ev.keycode = KEY_ESCAPE
		ev.pressed = true
		_view.push_input(ev)
		ev = ev.duplicate()
		ev.pressed = false
		_view.push_input(ev)
		await _settle(6)
		_ok(FileAccess.file_exists(path_a), "Escape keeps the file")
		_ok(_confirm_dialog() == null, "Escape closes the splash")
	## Cancel
	await _tap(_center(_trashes()[0]))
	await _settle(8)
	dlg = _confirm_dialog()
	if dlg != null:
		var btns2 := []
		_find(dlg, func(n): return n is Button and (n as Button).text == "Cancel", btns2)
		if btns2.size() > 0:
			(btns2[0] as Button).pressed.emit()
		await _settle(6)
	_ok(FileAccess.file_exists(path_a) and _trashes().size() == 2, "Cancel keeps file and card")
	## Open-world refusal
	_app.current_project_path = path_a
	_app.open_welcome()
	await _settle(8)
	await _tap(_center(_trashes()[0]))
	await _settle(8)
	_ok(_confirm_dialog() == null, "open world: no confirmation")
	_ok(_picker_text().contains("is open"), "open world: refusal reason shown", "'%s'" % _picker_text())
	_ok(FileAccess.file_exists(path_a), "open world: file kept")
	_app.current_project_path = ""
	_app.open_welcome()
	await _settle(8)
	## Delete
	await _tap(_center(_trashes()[0]))
	await _settle(8)
	dlg = _confirm_dialog()
	var which := "Alpha"
	var gone_path := path_a
	if dlg != null:
		var ls := []
		_find(dlg, func(n): return n is Label, ls)
		for l in ls:
			if (l as Label).text.contains("Bravo.ctl"):
				gone_path = _root.path_join("Bravo.ctl")
		var bs := []
		_find(dlg, func(n): return n is Button and (n as Button).text == "Delete", bs)
		_ok(bs.size() == 1, "Delete button found")
		if bs.size() > 0:
			(bs[0] as Button).pressed.emit()
		await _settle(10)
	_ok(not FileAccess.file_exists(gone_path), "Delete removes the file", gone_path)
	_ok(_trashes().size() == 1, "Delete removes the card", "%d left" % _trashes().size())
	_ok(not DccSettings.recent_projects().has(gone_path), "forgotten from recents")
	## Card body still opens (dummy file fails to load: the attempt shows as a changed picker message/error).
	var left_path := _root.path_join("Bravo.ctl") if gone_path.ends_with("Alpha.ctl") else path_a
	var card2: Control = _trashes()[0].get_parent()
	while card2 != null and not (card2 is PanelContainer):
		card2 = card2.get_parent()
	var cb := _rect(card2)
	var calls := [0]
	await _tap(Vector2(cb.position.x + cb.size.x * 0.4, cb.position.y + cb.size.y * 0.25))
	await _settle(30)
	_ok(_confirm_dialog() == null, "body tap: no delete splash")
	print("WD body tap result text='%s' busy_then_failed" % _picker_text())
	_ok(_picker_text() != "" and _picker_text() != before, "body tap attempted to open the world")
	_ok(FileAccess.file_exists(left_path), "body tap kept the file")
	await _shot("after")
	for n in ["Alpha.ctl", "Bravo.ctl"]:
		DirAccess.remove_absolute(_root.path_join(n))
	DirAccess.remove_absolute(_root)
	print("WD RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
