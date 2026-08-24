extends Node
## PH-05 probe: does a flick on a dock-sheet row scroll the sheet?
##
## Run:
##   godot --path . --resolution 393x852 _scrolldrag_probe.tscn -- --force-touch
##
## `Input.set_emulate_touch_from_mouse(true)` is what makes
## `DisplayServer.is_touchscreen_available()` true on this dev box, and that
## flag is the exact gate `ScrollContainer`'s own drag-to-scroll sits behind --
## without it the engine's touch scrolling is compiled-in but never armed, so
## the probe would report "no scroll" for reasons that have nothing to do with
## the shell. Verified against 4.7.1, not assumed.

var _scroll: ScrollContainer

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout
	app.open_project_dialog.hide()
	await get_tree().process_frame

	print("touchscreen_available=", DisplayServer.is_touchscreen_available())
	app._set_sheet_open("left", true)
	await get_tree().process_frame
	await get_tree().process_frame
	await get_tree().create_timer(0.4).timeout

	_scroll = _find_scroll(app.left_dock)
	if _scroll == null:
		print("no ScrollContainer under left_dock")
		get_tree().quit()
		return
	var r := _scroll.get_global_rect()
	print("scroll rect=", r, " content=", _scroll.get_child(0).size,
		" deadzone=", _scroll.scroll_deadzone)

	var y := r.position.y + 12.0
	while y < r.end.y - 12.0:
		await _flick(Vector2(r.get_center().x, y))
		y += 36.0

	## The other half of the contract: a `PASS` button must still press.
	var buttons: Array[Button] = []
	_find_buttons(_scroll, buttons)
	var vis: Array[Button] = []
	for b in buttons:
		if b.is_visible_in_tree() and r.intersects(b.get_global_rect()):
			vis.append(b)
	print("buttons under the scroll: ", buttons.size(), " visible in rect: ", vis.size())
	var only := int(OS.get_cmdline_user_args()[1]) if OS.get_cmdline_user_args().size() > 1 else -1
	for b in (vis if only < 0 else [vis[only]] as Array[Button]):
		var fired := [0]
		b.pressed.connect(func(): fired[0] += 1)
		await _tap(b)
		print("  tap %-28s filter=%d fired=%d" % [b.text.strip_edges().left(26),
			b.mouse_filter, fired[0]])
	get_tree().quit()

func _find_buttons(n: Node, out: Array[Button]) -> void:
	if n is Button:
		out.append(n)
	for c in n.get_children():
		_find_buttons(c, out)

func _tap(b: Button) -> void:
	var at := b.get_global_rect().get_center()
	var vp := get_viewport()
	var hover := InputEventMouseMotion.new()
	hover.position = at
	vp.push_input(hover)
	await get_tree().process_frame
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = at
	vp.push_input(down)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = at
	vp.push_input(up)
	await get_tree().process_frame

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var s := _find_scroll(c)
		if s != null:
			return s
	return null

func _flick(at: Vector2) -> void:
	_scroll.scroll_vertical = 0
	await get_tree().process_frame
	var vp := get_viewport()
	var hover := InputEventMouseMotion.new()
	hover.position = at
	vp.push_input(hover)
	await get_tree().process_frame
	var over := vp.gui_get_hovered_control()
	var who := "%s(%s)" % [over.get_class(), over.name] if over != null else "<none>"
	var mb := InputEventMouseButton.new()
	mb.button_index = MOUSE_BUTTON_LEFT
	mb.pressed = true
	mb.position = at
	vp.push_input(mb)
	var p := at
	for i in 8:
		p += Vector2(0, -12)
		var mm := InputEventMouseMotion.new()
		mm.position = p
		mm.relative = Vector2(0, -12)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		vp.push_input(mm)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = p
	vp.push_input(up)
	await get_tree().process_frame
	print("  y=%4d  scrolled=%3d  under=%s" % [int(at.y), _scroll.scroll_vertical, who])
