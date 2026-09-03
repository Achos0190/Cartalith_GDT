extends Node
## PH-15 verification probe: does a scroll flick that starts on a `PhoneMenu`
## L2 row activate the row it started on, and does a genuine tap on the same
## kind of row still fire? Modelled on `_scrolldrag_probe.gd`'s own
## `_flick()`/`_tap()` pattern (push synthetic `InputEventMouseButton` /
## `InputEventMouseMotion` with `Input.set_emulate_touch_from_mouse(true)`,
## which is how that probe verified `dcc_shell.gd`'s PH-05 fix on this exact
## Godot version) -- but the oracle is different: `phone_menu.gd`'s rows are
## not `Button`s with a `pressed` signal to count (`_row()`'s own header
## explains why), so the oracle here is `PhoneMenu._stack.size()`: every
## interactive L2 row's `on_press` is `_push(...)`, which appends to `_stack`
## (`phone_menu.gd:514`). A flick must leave `_stack` at its starting size; a
## tap must grow it by one.
##
## Run:
##   godot --headless --path . _ph15_probe.tscn -- --force-touch --h1080

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_all(node: Node, pred: Callable, out: Array) -> void:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(), " ===")
	if not app.is_phone():
		print("NOT PHONE -- nothing measured")
		get_tree().quit()
		return

	var pm = app._phone_menu
	pm.open()
	await _frames(4)
	print("menu open=", pm.is_open(), " stack=", pm._stack.size())

	var scroll: ScrollContainer = pm._screen_scroll
	var body: Control = pm._screen_body
	print("scroll size=", scroll.size, " body size=", body.size,
		" scrolls=", body.size.y > scroll.size.y)

	# Every interactive row is a PanelContainer with on_press.is_valid() ==
	# true, which is exactly what set its mouse_filter to STOP (`_row()`'s own
	# branch); a status row (IGNORE) has no `_row_input` connection at all and
	# would not exercise this fix.
	var rows: Array = []
	_find_all(body, func(n): return n is PanelContainer and (n as Control).mouse_filter == Control.MOUSE_FILTER_STOP, rows)
	print("interactive rows found=", rows.size())
	if rows.size() < 2:
		print("TOO FEW ROWS to test a multi-row flick")
		get_tree().quit()
		return

	var vp := get_viewport()
	var first: Control = rows[0]
	var start := first.get_global_rect().get_center()
	print("flick start row @ y=", start.y, " (", (first as PanelContainer).tooltip_text, ")")

	# -- 1. A flick starting on a row must scroll, not activate. -----------------
	scroll.scroll_vertical = 0
	await _frames(2)
	var stack_before: int = pm._stack.size()
	var hover := InputEventMouseMotion.new()
	hover.position = start
	vp.push_input(hover)
	await get_tree().process_frame
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = start
	vp.push_input(down)
	await get_tree().process_frame
	var over0 := vp.gui_get_hovered_control()
	print("  after down: hovered=", "%s(%s)" % [over0.get_class(), over0.name] if over0 != null else "<none>",
		" row.pressed_meta=", first.get_meta("pressed", "<none>"))
	var p := start
	# Eight samples of 12px, matching dcc_shell.gd's own PH-05 verification
	# (`phone_fit()`'s comment: "an eight-sample flick scrolls 96 px").
	for i in 8:
		p += Vector2(0, -12)
		var mm := InputEventMouseMotion.new()
		mm.position = p
		mm.relative = Vector2(0, -12)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		vp.push_input(mm)
		await get_tree().process_frame
	var over1 := vp.gui_get_hovered_control()
	print("  mid-drag (after 8 samples, 96px up): hovered=",
		"%s(%s)" % [over1.get_class(), over1.name] if over1 != null else "<none>",
		" row.pressed_meta=", first.get_meta("pressed", "<none>"),
		" scroll_now=", scroll.scroll_vertical)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = p
	vp.push_input(up)
	await _frames(3)
	print("[flick] scrolled=", scroll.scroll_vertical,
		" stack ", stack_before, " -> ", pm._stack.size(),
		"  (PASS if stack unchanged)")

	# -- 1b. The same flick, as *native* touch events (`InputEventScreenTouch`/
	#    `InputEventScreenDrag`) rather than mouse-emulated-as-touch. A real
	#    Android handset delivers these, not mouse events; `_row_input()`'s
	#    `InputEventScreenTouch`/`InputEventScreenDrag` branch is dead code in
	#    test 1 above (mouse events took the `InputEventMouseButton` branch),
	#    so this is what actually exercises it. -------------------------------
	rows.clear()
	_find_all(body, func(n): return n is PanelContainer and (n as Control).mouse_filter == Control.MOUSE_FILTER_STOP, rows)
	var touch_row: Control = rows[0]
	var tstart := touch_row.get_global_rect().get_center()
	scroll.scroll_vertical = 0
	await _frames(2)
	var stack_before_touch: int = pm._stack.size()
	var td := InputEventScreenTouch.new()
	td.index = 0
	td.pressed = true
	td.position = tstart
	vp.push_input(td)
	await get_tree().process_frame
	print("  [native] after touch-down: row.pressed_meta=", touch_row.get_meta("pressed", "<none>"))
	var tp := tstart
	for i in 8:
		var rel := Vector2(0, -12)
		tp += rel
		var tm := InputEventScreenDrag.new()
		tm.index = 0
		tm.position = tp
		tm.relative = rel
		vp.push_input(tm)
		await get_tree().process_frame
	print("  [native] mid-drag (96px up): row.pressed_meta=", touch_row.get_meta("pressed", "<none>"),
		" scroll_now=", scroll.scroll_vertical)
	var tu := InputEventScreenTouch.new()
	tu.index = 0
	tu.pressed = false
	tu.position = tp
	vp.push_input(tu)
	await _frames(3)
	print("[native flick] scrolled=", scroll.scroll_vertical,
		" stack ", stack_before_touch, " -> ", pm._stack.size(),
		"  (PASS if stack unchanged)")

	# -- 1c. A native touch tap (down/up, no drag) must still fire. --------------
	rows.clear()
	_find_all(body, func(n): return n is PanelContainer and (n as Control).mouse_filter == Control.MOUSE_FILTER_STOP, rows)
	var touch_tap_row: Control = rows[0]
	var tat := touch_tap_row.get_global_rect().get_center()
	var stack_before_ttap: int = pm._stack.size()
	var td2 := InputEventScreenTouch.new()
	td2.index = 0
	td2.pressed = true
	td2.position = tat
	vp.push_input(td2)
	await get_tree().process_frame
	var tu2 := InputEventScreenTouch.new()
	tu2.index = 0
	tu2.pressed = false
	tu2.position = tat
	vp.push_input(tu2)
	await _frames(3)
	print("[native tap] stack ", stack_before_ttap, " -> ", pm._stack.size(),
		"  (PASS if stack grew by exactly 1)")
	if pm._stack.size() > stack_before_ttap:
		pm.go_back()
		await _frames(2)

	# -- 2. A genuine tap on the same class of row must still fire. --------------
	scroll.scroll_vertical = 0
	await _frames(2)
	# Re-find the rows after the flick's own rebuild-on-scroll (there is none
	# here, but `_render()` is called by `_push`, so re-resolve defensively).
	rows.clear()
	_find_all(body, func(n): return n is PanelContainer and (n as Control).mouse_filter == Control.MOUSE_FILTER_STOP, rows)
	var tap_target: Control = rows[0]
	var at := tap_target.get_global_rect().get_center()
	var stack_before_tap: int = pm._stack.size()
	var hover2 := InputEventMouseMotion.new()
	hover2.position = at
	vp.push_input(hover2)
	await get_tree().process_frame
	var down2 := InputEventMouseButton.new()
	down2.button_index = MOUSE_BUTTON_LEFT
	down2.pressed = true
	down2.position = at
	vp.push_input(down2)
	await get_tree().process_frame
	var up2 := InputEventMouseButton.new()
	up2.button_index = MOUSE_BUTTON_LEFT
	up2.pressed = false
	up2.position = at
	vp.push_input(up2)
	await _frames(3)
	print("[tap] stack ", stack_before_tap, " -> ", pm._stack.size(),
		"  (PASS if stack grew by exactly 1)")

	# -- 3. A small wobble (well under the deadzone) must still count as a tap. --
	if pm._stack.size() > stack_before_tap:
		pm.go_back()
		await _frames(2)
	rows.clear()
	_find_all(body, func(n): return n is PanelContainer and (n as Control).mouse_filter == Control.MOUSE_FILTER_STOP, rows)
	var wobble_target: Control = rows[0]
	var at3 := wobble_target.get_global_rect().get_center()
	var stack_before_wobble: int = pm._stack.size()
	var hover3 := InputEventMouseMotion.new()
	hover3.position = at3
	vp.push_input(hover3)
	await get_tree().process_frame
	var down3 := InputEventMouseButton.new()
	down3.button_index = MOUSE_BUTTON_LEFT
	down3.pressed = true
	down3.position = at3
	vp.push_input(down3)
	await get_tree().process_frame
	var wob := InputEventMouseMotion.new()
	wob.position = at3 + Vector2(0, 2)
	wob.relative = Vector2(0, 2)
	wob.button_mask = MOUSE_BUTTON_MASK_LEFT
	vp.push_input(wob)
	await get_tree().process_frame
	var up3 := InputEventMouseButton.new()
	up3.button_index = MOUSE_BUTTON_LEFT
	up3.pressed = false
	up3.position = at3 + Vector2(0, 2)
	vp.push_input(up3)
	await _frames(3)
	print("[wobble 2px] stack ", stack_before_wobble, " -> ", pm._stack.size(),
		"  (PASS if stack grew by exactly 1 -- a real thumb wobble must not cancel a tap)")

	print("=== done ===")
	get_tree().quit()
