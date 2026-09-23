extends Node
## Windowed proof for tool windows' resize grip and remembered size
## (`OUTSTANDING_WORK.md` §2.10, 2026-09-24; `DccWidgets._desktop_window_size`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _wingrip_probe.tscn
##
## On the faction roster, at desktop density:
##   1. a grip sits in the window's bottom-right corner and DRAWS something
##      (more than one colour in its rect);
##   2. dragging the grip resizes the window by the drag;
##   3. closing the window stores that size;
##   4. a fresh window of the same kind opens at the stored size.
## The user's own stored value for this key is put back at the end.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("WINGRIP %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _grip_of(w: Window) -> Control:
	var stack: Array = [w]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			if c is Control and (c as Control).tooltip_text == "Drag to resize":
				return c
			stack.append(c)
	return null

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("WINGRIP REFUSED: headless")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(4)
	var w: Window = app.faction_roster_window
	var key := DccWidgets.window_key(w)
	var prior := DccSettings.window_size(key)
	print("WINGRIP key %s, prior stored %s" % [key, prior])

	w.open()
	await _frames(8)
	var grip := _grip_of(w)
	_check("a grip exists", grip != null)
	if grip == null:
		get_tree().quit(1)
		return
	var gr := grip.get_global_rect()
	## `get_global_rect()` is in the dialog's own viewport, so compare against
	## the window's own rect at the origin.
	var wr := Rect2(Vector2.ZERO, Vector2(w.size))
	_check("it sits in the bottom-right corner, small", gr.size.x <= 20.0 and gr.size.y <= 20.0
		and wr.end.x - gr.end.x <= 24.0 and wr.end.y - gr.end.y <= 24.0 and wr.encloses(gr),
		"grip %s in window %s" % [gr, wr])
	await RenderingServer.frame_post_draw
	## The dialog draws into its own viewport here, so read that one.
	var img := w.get_texture().get_image()
	var r := Rect2i(grip.get_global_rect())
	var colours := {}
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			colours[img.get_pixel(x, y)] = true
	_check("it draws something (more than one colour in its rect)", colours.size() > 1, "%d colours in %s" % [colours.size(), r])

	var before := w.size
	var ev := InputEventMouseMotion.new()
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT
	ev.relative = Vector2(60, 40)
	grip.gui_input.emit(ev)
	await _frames(4)
	_check("dragging the grip resizes by the drag", w.size == before + Vector2i(60, 40), "%s -> %s" % [before, w.size])

	var dragged := w.size
	w.hide()
	await _frames(2)
	_check("closing stores the size", DccSettings.window_size(key) == dragged, str(DccSettings.window_size(key)))

	var fresh := FactionRosterWindow.new()
	app.add_child(fresh)
	fresh.setup(app, app.bridge)
	_check("a fresh window opens at the stored size", fresh.size == dragged, "%s vs %s" % [fresh.size, dragged])
	fresh.queue_free()

	DccSettings.set_window_size(key, prior)
	print("WINGRIP %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
