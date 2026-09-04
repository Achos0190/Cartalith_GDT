extends Node
## VERIFIER. The one number in `group()`'s new doc comment that a bare-host
## replica disagreed with: "autowrap ... lowers its minimum, and it lowers it
## to **0**". Measured on LIVE headers in the real dock, not a replica chain.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _vfy_grpmin2_probe.tscn

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _is_group_header(c: Node) -> bool:
	if not (c is Button):
		return false
	var b := c as Button
	if not b.flat or b.focus_mode != Control.FOCUS_NONE:
		return false
	var want: int = DccTheme.role_px("fs_dock_header") if DccTheme.is_tablet() else DccTheme.FS_HEADER
	if b.get_theme_font_size("font_size") != want:
		return false
	var p := b.get_parent()
	if p == null or b.get_index() + 1 >= p.get_child_count():
		return false
	var pad := p.get_child(b.get_index() + 1)
	if not (pad is MarginContainer) or pad.get_child_count() == 0:
		return false
	return pad.get_child(0) is VBoxContainer

func _headers(root: Node, out: Array) -> void:
	for c in root.get_children():
		if _is_group_header(c):
			out.append(c)
		_headers(c, out)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	bridge.generate({"seed": 483920, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(12)
	app.select_domain_mode("civilization", "landmarks")
	await _frames(14)

	var hs: Array = []
	_headers(app.left_dock_body, hs)
	print("VM live group headers in the left dock: %d   (autowrap on = shipped)" % hs.size())
	var zero := 0
	var nonzero := 0
	var worst := 0.0
	for c in hs:
		var b := c as Button
		var on: float = b.get_minimum_size().x
		b.autowrap_mode = TextServer.AUTOWRAP_OFF
		var off: float = b.get_minimum_size().x
		b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		await _frames(1)
		if on <= 0.01:
			zero += 1
		else:
			nonzero += 1
			worst = maxf(worst, on)
		if off > 230.0:
			print("VM   min.x  ON=%6.2f  OFF=%6.2f  size.x=%6.1f  clip=%s  '%s'"
				% [on, off, b.size.x, b.clip_text, String(b.text)])
	print("VM headers whose autowrapped min.x is exactly 0: %d of %d   (non-zero: %d, largest %.2f)"
		% [zero, hs.size(), nonzero, worst])
	get_tree().quit(0)
