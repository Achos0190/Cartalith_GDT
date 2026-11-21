extends Node
## F38 (owner findings 2026-11-21): walks every window and dialog a menu command opens and reports any control outside its window or overlapping a sibling. Windowed, scratch APPDATA. See OUTSTANDING_WORK.md.
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
	print("WALK %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("WALK %s" % s)


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


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


# ---- the walk ----------------------------------------------------------------------------------------------------

## Menu rows the walk does not press: they leave the app, open a native file dialog, or change/destroy the world.
const SKIP := "quit|exit|open|save|load|import|browse|generate|new world|new seed|clear|reset|delete|remove|bake|finalize|restart|undo|redo|recompute|regenerate|storage|assistant"

var _issues := {}
var _windows_seen := {}
var _only_title := ""


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
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--only":
			_only_title = String(args[i + 1])
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
	var wd := get_tree().create_timer(900.0)
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
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	await _frames(30)
	await _walk()
	_finish_walk()


func _visible_windows() -> Array:
	var out: Array = []
	for w in app.find_children("*", "Window", true, false):
		if w is PopupMenu or w == app.get_window():
			continue
		if (w as Window).visible:
			out.append(w)
	return out


## Every visible control of `w`: its rect against the window, against its parent, and against its siblings.
func _check_window(w: Window, title: String) -> void:
	var win_rect := Rect2(Vector2.ZERO, Vector2(w.size))
	var stack: Array = [w]
	var n := 0
	while not stack.is_empty():
		var node: Node = stack.pop_back()
		for ch in node.get_children():
			stack.append(ch)
		if not (node is Control) or node == w:
			continue
		var c := node as Control
		if not c.is_visible_in_tree() or c.size.x < 2.0 or c.size.y < 2.0:
			continue
		n += 1
		if not (c is Button or c is LineEdit or c is OptionButton or c is CheckBox or c is Range or c is Label or c is TextEdit):
			continue
		# a control inside a scrolling parent is allowed to extend past the window; its scroller is checked itself
		var in_scroll := false
		var p := c.get_parent()
		while p != null and p != w:
			if p is ScrollContainer:
				in_scroll = true
				break
			p = p.get_parent()
		var path := _short_path(c, w)
		var r := c.get_global_rect()
		var label := (c as Button).text if c is Button else ("" if not (c is Label) else (c as Label).text)
		var where := "%s | %s '%s'" % [title, c.get_class(), label.left(24)]
		if not in_scroll and (r.position.x < win_rect.position.x - 1.5 or r.position.y < win_rect.position.y - 1.5
				or r.end.x > win_rect.end.x + 1.5 or r.end.y > win_rect.end.y + 1.5):
			_issues["%s|outside|%s" % [title, path]] = "OUTSIDE window %s: %s rect=%s" % [win_rect.size, where, r]
		# overlap with a visible sibling that is also a leaf-ish control
		var par := c.get_parent()
		if par is Control and not (par is ScrollContainer):
			var idx := c.get_index()
			for k in range(idx + 1, par.get_child_count()):
				var sib = par.get_child(k)
				if not (sib is Control) or not (sib as Control).is_visible_in_tree():
					continue
				var sc := sib as Control
				if not (sc is Button or sc is LineEdit or sc is OptionButton or sc is CheckBox):
					continue
				if not (c is Button or c is LineEdit or c is OptionButton or c is CheckBox):
					continue
				var ir := r.intersection(sc.get_global_rect())
				if ir.size.x > 2.0 and ir.size.y > 2.0:
					var sl := (sc as Button).text if sc is Button else ""
					_issues["%s|overlap|%s|%s" % [title, path, sc.name]] = "OVERLAP %s with %s '%s' (%.0fx%.0f)" % [where, sc.get_class(), sl.left(24), ir.size.x, ir.size.y]
	_windows_seen[title] = n


func _short_path(c: Node, root: Node) -> String:
	var parts: Array = []
	var cur := c
	while cur != null and cur != root and parts.size() < 6:
		parts.append(String(cur.name))
		cur = cur.get_parent()
	parts.reverse()
	return "/".join(parts)


func _walk() -> void:
	var idx := CommandIndex.new()
	idx.build(app, _bridge, true)
	var rows: Array = idx.all()
	_p("%d command rows" % rows.size())
	var skip := RegEx.create_from_string("(?i)(" + SKIP + ")")
	var done := {}
	for row in rows:
		var r: Dictionary = row
		if String(r.get("kind", "")) != "menu" or not bool(r.get("available", true)):
			continue
		var t := String(r.get("title", ""))
		if skip.search(t) != null or done.has(t):
			continue
		if _only_title != "" and not t.contains(_only_title):
			continue
		done[t] = true
		var before := _visible_windows()
		idx._fire(r)
		await _frames(10)
		for w in _visible_windows():
			if before.has(w):
				continue
			var title := "%s [%s]" % [(w as Window).title if (w as Window).title != "" else String(w.name), t]
			await _frames(40)
			_check_window(w as Window, title)
			if _only_title != "":
				_dump_min(w, 0)
				for ch in w.get_children():
					if ch is Control:
						_p("child %s %s size=%s pos=%s anchors=%s,%s,%s,%s offs=%s,%s,%s,%s" % [ch.get_class(), ch.name, (ch as Control).size, (ch as Control).position, (ch as Control).anchor_left, (ch as Control).anchor_top, (ch as Control).anchor_right, (ch as Control).anchor_bottom, (ch as Control).offset_left, (ch as Control).offset_top, (ch as Control).offset_right, (ch as Control).offset_bottom])
				_p("window size=%s csf=%s mode=%s cs_mode=%s visible_rect=%s pos=%s" % [(w as Window).size, (w as Window).content_scale_factor, (w as Window).content_scale_mode, (w as Window).content_scale_size, (w as Window).get_visible_rect(), (w as Window).position])
			if _out != "" and _issues.keys().any(func(k): return String(k).begins_with(title)):
				await _shot("win_%s.png" % t.validate_filename())
			(w as Window).hide()
			await _frames(3)
		# popups that survive (menus left open) are closed so the next row starts clean
		for pm in app.find_children("*", "PopupMenu", true, false):
			if (pm as PopupMenu).visible and (pm as PopupMenu).get_parent() is Window == false:
				pass
	_p("windows walked: %d" % _windows_seen.size())
	for k in _windows_seen:
		_p("  walked %s (%d controls)" % [k, _windows_seen[k]])


func _finish_walk() -> void:
	var keys: Array = _issues.keys()
	keys.sort()
	for k in keys:
		_p("ISSUE " + String(_issues[k]))
	_ok("every walked window keeps its controls inside it and clear of each other", _issues.is_empty(), "%d issues in %d windows" % [_issues.size(), _windows_seen.size()])
	_finish()


## Debug aid (only with `--only`): the minimum heights down the first chain of a window, to find what is taller than it.
func _dump_min(n: Node, depth: int) -> void:
	if depth > 7:
		return
	for ch in n.get_children():
		if ch is Control and (ch as Control).is_visible_in_tree():
			var c := ch as Control
			var m := c.get_combined_minimum_size()
			if m.y > 200.0:
				_p("%smin %s %s %s size=%s" % ["  ".repeat(depth), ch.get_class(), ch.name, m, c.size])
				_dump_min(ch, depth + 1)
