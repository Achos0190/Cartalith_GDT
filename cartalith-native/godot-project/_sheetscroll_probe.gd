extends Node
## Verification probe for the phone dock-sheet scroll-retention fix
## (`dcc_shell.gd::_set_sheet_open` / `_reset_dock_scroll`).
##
## Run:
##   godot4 --path . --resolution 393x852 _sheetscroll_probe.tscn -- --force-touch --nowelcome
##
## Opens the left sheet, scrolls it down, closes it, reopens it, and prints
## the scroll position both dock sheets land on. Also checks the right sheet.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var s := _find_scroll(c)
		if s != null:
			return s
	return null

func _check(app: Node, side: String, dock: Control, body: Control) -> void:
	app._set_sheet_open(side, true)
	await _frames(10)
	var scroll := _find_scroll(dock)
	if scroll == null:
		print("[%s] no ScrollContainer found under dock" % side)
		return

	## Guaranteed overflow regardless of whatever real content this dock
	## happens to hold right now (no world generated => a short right dock) --
	## a filler control decouples the scroll-reset check from panel content.
	var filler := Control.new()
	filler.custom_minimum_size = Vector2(0, 4000)
	body.add_child(filler)
	await _frames(3)

	## The body's real content can keep changing height for a few frames
	## after open (fonts, deferred rebuilds) -- wait for `max_value` to hold
	## steady across five consecutive frames before trusting it, rather than
	## fighting a moving target.
	var stable := 0
	var last := -1.0
	var settle_frames := 0
	while stable < 5 and settle_frames < 60:
		await get_tree().process_frame
		var mv: float = scroll.get_v_scroll_bar().max_value
		stable = (stable + 1) if mv == last else 0
		last = mv
		settle_frames += 1
	var max_v: float = scroll.get_v_scroll_bar().max_value
	print("[%s] scroll rect=%s content max_value=%s (settled after %d frames)" %
		[side, scroll.get_global_rect(), max_v, settle_frames])
	scroll.scroll_vertical = int(max_v)
	await _frames(1)
	print("[%s] after manual scroll-to-bottom: scroll_vertical=%d (max=%d)" %
		[side, scroll.scroll_vertical, int(scroll.get_v_scroll_bar().max_value)])

	app._set_sheet_open(side, false)
	await _frames(3)
	app._set_sheet_open(side, true)
	await _frames(3)
	print("[%s] after close+reopen: scroll_vertical=%d (expect 0)" % [side, scroll.scroll_vertical])
	app._set_sheet_open(side, false)
	await _frames(2)

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().process_frame

	print("=== phone=", app.is_phone(), " ===")
	await _check(app, "left", app.left_dock, app.left_dock_body)
	await _check(app, "right", app.right_dock, app.right_dock_body)

	print("=== done ===")
	get_tree().quit()
