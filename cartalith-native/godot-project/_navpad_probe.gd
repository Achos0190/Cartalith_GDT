extends Node
## TEMPORARY verification harness for the touch navpad (`GUI_GAP_REGISTER.md`
## SH-14) and the cover-semantics `reset_view()` the ⟳ button calls.
##
##   godot4 --path . --resolution 393x852 _navpad_probe.tscn -- --force-touch --nowelcome
##
## Measures rather than eyeballs: the reset's zoom against the cover scale
## computed independently from the fit rect, the two step buttons against
## 1.35, and a real synthetic one-finger drag with pan mode off then on.

var app: Node
var vp: Control

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)
	return out

## A one-finger drag, as `emulate_mouse_from_touch` delivers it.
##
## Fed to `_input()` directly rather than through `Viewport.push_input()`:
## measured in this harness, `push_input` reaches GUI dispatch (a pushed tap
## does press a Button -- `_scrolldrag_probe.gd` relies on exactly that) but
## never reaches any node's `_input`. Proven against code this pass did not
## touch: a pushed `WHEEL_UP` at the viewport centre leaves `zoom()` bit-
## identical, and wheel zoom demonstrably works in the shipped build. So the
## dispatch stage is the harness's limit, not the handler's -- these calls
## exercise the real branch, and the wiring is what the device pass verifies.
func _drag(from: Vector2, to: Vector2) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = from
	vp._input(down)
	await _frames(1)
	var steps := 8
	for i in range(1, steps + 1):
		var mm := InputEventMouseMotion.new()
		mm.position = from + (to - from) * (float(i) / steps)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		vp._input(mm)
		await _frames(1)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = to
	vp._input(up)
	await _frames(2)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	vp = app.viewport

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1200:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	print("=== has_world=", app.bridge.has_world, " grid=", app.bridge.grid_size(), " ===")

	# --- The navpad exists, is 44 dp, and clears the chrome ------------------
	var pad: VBoxContainer = vp._navpad
	print("[pad] built=", pad != null)
	if pad == null:
		get_tree().quit(1)
		return
	print("[pad] rect=", pad.get_global_rect(), " vp size=", vp.size)
	var btns: Array = []
	_find_all(pad, func(n): return n is Button, btns)
	print("[pad] buttons=", btns.size())
	for b in btns:
		var bb := b as Button
		print("[pad]   '", bb.tooltip_text, "' rect=", bb.get_global_rect(),
			" icon=", (bb.icon.get_size() if bb.icon != null else Vector2.ZERO))
	print("[pad] coords_label rect=", vp._coords_label.get_global_rect())

	# --- Reset is COVER, computed independently ------------------------------
	var fit: Rect2 = vp.overlay.displayed_rect()
	var want_cover: float = maxf(vp.size.x / fit.size.x, vp.size.y / fit.size.y)
	vp.reset_view()
	await _frames(2)
	print("[cover] fit_rect=", fit, " expected_cover=", want_cover,
		" zoom=", vp.zoom(), " camera_pos=", vp._camera.position)
	## The map's on-screen rect after the reset: fit rect through the camera.
	var tl: Vector2 = vp._camera.position + fit.position * vp.zoom()
	var br: Vector2 = vp._camera.position + fit.end * vp.zoom()
	print("[cover] map on screen tl=", tl, " br=", br,
		" (viewport 0,0 .. ", vp.size, ")")
	print("[cover] covers_x=", tl.x <= 0.01 and br.x >= vp.size.x - 0.01,
		" covers_y=", tl.y <= 0.01 and br.y >= vp.size.y - 0.01,
		" centred=", is_equal_approx(tl.x + br.x, vp.size.x) and is_equal_approx(tl.y + br.y, vp.size.y))

	# --- Zoom step buttons ---------------------------------------------------
	var z0: float = vp.zoom()
	(btns[0] as Button).emit_signal("pressed")
	await _frames(2)
	print("[zoom] in  ", z0, " -> ", vp.zoom(), " ratio=", vp.zoom() / z0, " (want 1.35)")
	var z1: float = vp.zoom()
	(btns[1] as Button).emit_signal("pressed")
	await _frames(2)
	print("[zoom] out ", z1, " -> ", vp.zoom(), " ratio=", vp.zoom() / z1, " (want 0.7407)")

	## Sanity: does a pushed event reach `_input` at all in this harness?
	var zw: float = vp.zoom()
	var wheel := InputEventMouseButton.new()
	wheel.button_index = MOUSE_BUTTON_WHEEL_UP
	wheel.pressed = true
	wheel.position = vp.size * 0.5
	get_viewport().push_input(wheel)
	await _frames(2)
	print("[dbg] wheel ", zw, " -> ", vp.zoom(),
		" processing_input=", vp.is_processing_input(),
		" same_viewport=", vp.get_viewport() == get_viewport(),
		" vp_class=", vp.get_viewport().get_class(),
		" root_class=", get_viewport().get_class())

	# --- Pan mode ------------------------------------------------------------
	var centre := vp.get_global_rect().position + vp.size * 0.5
	var p0: Vector2 = vp._camera.position
	await _drag(centre, centre + Vector2(-120, 0))
	var p1: Vector2 = vp._camera.position
	print("[pan] mode=off  camera ", p0, " -> ", p1, " delta=", p1 - p0, " (want 0,0)")

	vp.set_pan_mode(true)
	await _frames(2)
	var pb := btns[2] as Button
	print("[pan] latched=", vp.pan_mode(), " btn_pressed=", pb.button_pressed,
		" fill=", (pb.get_theme_stylebox("normal") as StyleBoxFlat).bg_color,
		" ink=", pb.get_theme_color("icon_normal_color"),
		" (accent=", DccTheme.c("accent"), ")")
	for w in _find_all(app, func(n): return n is Window, []):
		(w as Window).hide()
	await _frames(4)
	get_viewport().get_texture().get_image().save_png("user://navpad_latched.png")
	await _drag(centre, centre + Vector2(-120, 0))
	var p2: Vector2 = vp._camera.position
	print("[pan] mode=on   camera ", p1, " -> ", p2, " delta=", p2 - p1, " (want -120,0)")

	## A tap on the navpad itself must not also drag the map.
	var pad_pt := (btns[0] as Control).get_global_rect().get_center()
	var p3: Vector2 = vp._camera.position
	await _drag(pad_pt, pad_pt + Vector2(-60, 0))
	print("[pan] tap-on-pad delta=", vp._camera.position - p3, " (want 0,0)")

	# --- Reset clears pan mode ----------------------------------------------
	(btns[3] as Button).emit_signal("pressed")
	await _frames(2)
	print("[reset] pan_mode=", vp.pan_mode(), " btn_pressed=", (btns[2] as Button).button_pressed,
		" zoom=", vp.zoom(), " (want cover ", want_cover, ")")

	for w in _find_all(app, func(n): return n is Window, []):
		(w as Window).hide()
	await _frames(4)
	var img := get_viewport().get_texture().get_image()
	img.save_png("user://navpad.png")
	print("shot ", ProjectSettings.globalize_path("user://navpad.png"))
	print("=== done ===")
	get_tree().quit()
