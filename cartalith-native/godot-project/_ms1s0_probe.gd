extends Node
## MS-1 stage S0, the rows of the measurement sheet (`MOBILE_SHELL_MS1_SPEC.md` §4.1) that a desktop can
## answer: what a root `content_scale_factor` of 2 does on this Godot build. It boots no shell; it builds
## small reference scenes and measures them. The device rows (the 6T's `wm density`, `screen_get_dpi()`,
## the manifest, the cold boot) need the handset and are not here.
##
##   godot --path . --resolution 1200x800 _ms1s0_probe.tscn -- --out C:/abs/dir
##
## **Windowed, never `--headless`**: every row reads the framebuffer, which is not drawn there
## (`MISTAKES.md`, "Run a pixel probe"). Arguments read: `--out <dir>`; anything else aborts with exit 2.
##
## Rows (each prints one `S0 <row> ...` line):
##   text      a 12 px Label at root factor 2 in mode DISABLED and in CANVAS_ITEMS (aspect IGNORE): the visible
##             rect, the final transform, and HD-01's edge test (the largest adjacent-pixel |dLum| inside the
##             glyph box). Positive control: the same Label at factor 1 must also be measured non-empty.
##   windows   the drawn size, in physical px, of an embedded 300 x 200 AcceptDialog, a PopupMenu and a
##             ColorPickerButton's popup under root factor 2 (do they inherit the factor?).
##   input     a mouse press pushed at physical pixel (400, 300) under factor 2: the position `_gui_input`
##             receives, in the control's local units.
##   view3d    a `SubViewportContainer` with `stretch = true` 400 x 300 (canvas units) under factor 2: the
##             child `SubViewport.size` (is the 3D image rendered in dp and magnified?).
##   override  a SubViewport host with `size_2d_override` + stretch at factor 2 against the root factor 2
##             drawing the same scene: the number of differing pixels.
## Result: `RESULT: DONE (<n> rows)`; exit 0, 1 when a row's own positive control failed, 2 bad argument.

var _out := ""
var _bad := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw

func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b

## HD-01's edge test: the largest |dLum| between horizontally adjacent pixels inside `r` of `img`.
## Sharp native text reaches ~1.0 on a black ground; a magnified raster spreads every edge over two
## pixels and reads far lower (HD-01: 0.98 native, 0.27 magnified).
func _edge(img: Image, r: Rect2i) -> float:
	var best := 0.0
	r = r.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x - 1):
			best = maxf(best, absf(_lum(img.get_pixel(x, y)) - _lum(img.get_pixel(x + 1, y))))
	return best

## The bounding box of pixels that differ from `ground` by more than 0.1 in luminance.
func _ink_box(img: Image, ground: Color) -> Rect2i:
	var g := _lum(ground)
	var x0 := img.get_width()
	var y0 := img.get_height()
	var x1 := -1
	var y1 := -1
	for y in img.get_height():
		for x in img.get_width():
			if absf(_lum(img.get_pixel(x, y)) - g) > 0.1:
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
	if x1 < 0:
		return Rect2i()
	return Rect2i(x0, y0, x1 - x0 + 1, y1 - y0 + 1)

func _grab() -> Image:
	var img := get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	return img

func _ground() -> ColorRect:
	var g := ColorRect.new()
	g.color = Color.BLACK
	g.position = Vector2.ZERO
	g.size = Vector2(4000, 4000)
	return g

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
		else:
			print("ABORT: unknown argument ", args[i])
			get_tree().quit(2)
			return
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	var root := get_tree().root
	print("S0 build godot=%s window=%s dpi=%d screen_scale=%.3f" % [Engine.get_version_info()["string"],
		str(root.size), DisplayServer.screen_get_dpi(), DisplayServer.screen_get_scale()])
	var rows := 0

	# ---- text --------------------------------------------------------------------------------------
	var layer := Control.new()
	layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(layer)
	layer.add_child(_ground())
	var lab := Label.new()
	lab.text = "Hamlet 12 px"
	lab.add_theme_font_size_override("font_size", 12)
	lab.add_theme_color_override("font_color", Color.WHITE)
	lab.position = Vector2(20, 20)
	layer.add_child(lab)
	for spec in [["factor1", Window.CONTENT_SCALE_MODE_DISABLED, 1.0],
			["disabled_f2", Window.CONTENT_SCALE_MODE_DISABLED, 2.0],
			["canvas_items_f2", Window.CONTENT_SCALE_MODE_CANVAS_ITEMS, 2.0]]:
		root.content_scale_mode = spec[1]
		root.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_IGNORE
		root.content_scale_factor = spec[2]
		await _frames(4)
		var img := _grab()
		var box := _ink_box(img, Color.BLACK)
		var e := _edge(img, box) if box.size.x > 0 else -1.0
		if _out != "":
			img.get_region(Rect2i(0, 0, 300, 100)).save_png(_out.path_join("s0_text_%s.png" % spec[0]))
		print("S0 text %s visible_rect=%s final_scale=%.3f ink_box=%s edge_max_dlum=%.3f oversampling=%.3f" % [
			spec[0], str(root.get_visible_rect().size), root.get_final_transform().x.x, str(box), e,
			root.get_oversampling()])
		if spec[0] == "factor1" and box.size.x == 0:
			_bad += 1
			print("S0 text CONTROL FAILED: nothing drawn at factor 1")
		rows += 1
	## And with `oversampling_override` raised to the factor (HD-01's remedy on sub-windows).
	root.content_scale_mode = Window.CONTENT_SCALE_MODE_DISABLED
	root.content_scale_factor = 2.0
	if "oversampling_override" in root:
		root.oversampling_override = 2.0
		await _frames(4)
		var img2 := _grab()
		var box2 := _ink_box(img2, Color.BLACK)
		print("S0 text disabled_f2_oversample2 ink_box=%s edge_max_dlum=%.3f oversampling=%.3f" % [
			str(box2), _edge(img2, box2) if box2.size.x > 0 else -1.0, root.get_oversampling()])
		root.oversampling_override = 0.0
		rows += 1
	else:
		print("S0 text oversampling_override: not a Window property on this build")

	# ---- windows -----------------------------------------------------------------------------------
	root.content_scale_mode = Window.CONTENT_SCALE_MODE_DISABLED
	root.content_scale_factor = 2.0
	await _frames(2)
	var dlg := AcceptDialog.new()
	dlg.title = "S0"
	dlg.dialog_text = "three hundred"
	dlg.size = Vector2i(300, 200)
	layer.add_child(dlg)
	dlg.popup(Rect2i(40, 40, 300, 200))
	await _frames(4)
	var ib := _ink_box(_grab(), Color.BLACK)
	print("S0 windows acceptdialog embedded=%s size=%s content_scale=%.3f drawn_box_physical=%s" % [
		str(dlg.is_embedded()), str(dlg.size), dlg.content_scale_factor, str(ib)])
	dlg.hide()
	var pm := PopupMenu.new()
	for k in 5:
		pm.add_item("Item %d" % k)
	layer.add_child(pm)
	pm.popup(Rect2i(40, 40, 0, 0))
	await _frames(4)
	var pb := _ink_box(_grab(), Color.BLACK)
	print("S0 windows popupmenu embedded=%s size=%s drawn_box_physical=%s" % [str(pm.is_embedded()), str(pm.size), str(pb)])
	pm.hide()
	var cpb := ColorPickerButton.new()
	cpb.position = Vector2(20, 20)
	cpb.size = Vector2(40, 20)
	layer.add_child(cpb)
	await _frames(1)
	var pop := cpb.get_popup()
	cpb.pressed.emit()
	if not pop.visible:
		pop.popup(Rect2i(40, 40, 0, 0))
	await _frames(4)
	var cb := _ink_box(_grab(), Color.BLACK)
	print("S0 windows colorpicker_popup embedded=%s size=%s drawn_box_physical=%s" % [str(pop.is_embedded()), str(pop.size), str(cb)])
	pop.hide()
	cpb.queue_free()
	rows += 1

	# ---- input -------------------------------------------------------------------------------------
	var target := Control.new()
	target.position = Vector2.ZERO
	target.size = Vector2(1000, 1000)
	layer.add_child(target)
	var got: Array = []
	target.gui_input.connect(func(ev):
		if ev is InputEventMouseButton and ev.pressed:
			got.append(ev.position))
	await _frames(2)
	var mb := InputEventMouseButton.new()
	mb.button_index = MOUSE_BUTTON_LEFT
	mb.pressed = true
	mb.position = Vector2(400, 300)
	mb.global_position = Vector2(400, 300)
	Input.parse_input_event(mb)
	var up := mb.duplicate()
	up.pressed = false
	Input.parse_input_event(up)
	await _frames(3)
	print("S0 input pushed_physical=(400, 300) factor=2 gui_input_local=%s" % str(got))
	target.queue_free()
	rows += 1

	# ---- view3d ------------------------------------------------------------------------------------
	var box3 := SubViewportContainer.new()
	box3.stretch = true
	box3.position = Vector2(10, 10)
	box3.size = Vector2(400, 300)
	var sv3 := SubViewport.new()
	box3.add_child(sv3)
	layer.add_child(box3)
	await _frames(3)
	print("S0 view3d container_canvas=%s subviewport_size=%s (physical rect would be %s)" % [
		str(box3.size), str(sv3.size), str(box3.size * 2.0)])
	box3.queue_free()
	rows += 1

	# ---- override ----------------------------------------------------------------------------------
	## The reference scene: a ground, a 12 px Label and a 1 px border, drawn once under the root factor and
	## once inside a SubViewport host with `size_2d_override` + stretch, both at factor 2.
	layer.queue_free()
	await _frames(2)
	var scene_a := _ref_scene()
	add_child(scene_a)
	await _frames(4)
	var img_a := _grab().get_region(Rect2i(0, 0, 400, 300))
	scene_a.queue_free()
	root.content_scale_factor = 1.0
	await _frames(2)
	var host := SubViewport.new()
	host.size = Vector2i(root.size)
	host.size_2d_override = Vector2i(roundi(root.size.x / 2.0), roundi(root.size.y / 2.0))
	host.size_2d_override_stretch = true
	host.transparent_bg = false
	host.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(host)
	host.add_child(_ref_scene())
	await _frames(4)
	var img_b := host.get_texture().get_image()
	img_b.convert(Image.FORMAT_RGBA8)
	img_b = img_b.get_region(Rect2i(0, 0, 400, 300))
	var diff := 0
	var maxd := 0.0
	for y in 300:
		for x in 400:
			var d := absf(_lum(img_a.get_pixel(x, y)) - _lum(img_b.get_pixel(x, y)))
			maxd = maxf(maxd, d)
			if d > 0.006:
				diff += 1
	if _out != "":
		img_a.save_png(_out.path_join("s0_override_root.png"))
		img_b.save_png(_out.path_join("s0_override_host.png"))
	var ink_a := _ink_box(img_a, Color.BLACK)
	print("S0 override differing_px=%d of 120000 max_dlum=%.3f root_ink=%s host_ink=%s edge_root=%.3f edge_host=%.3f" % [
		diff, maxd, str(ink_a), str(_ink_box(img_b, Color.BLACK)), _edge(img_a, ink_a), _edge(img_b, _ink_box(img_b, Color.BLACK))])
	if ink_a.size.x == 0:
		_bad += 1
		print("S0 override CONTROL FAILED: the root capture is empty")
	rows += 1
	print("RESULT: DONE (%d rows)" % rows)
	get_tree().quit(0 if _bad == 0 else 1)

func _ref_scene() -> Control:
	var c := Control.new()
	c.set_anchors_preset(Control.PRESET_FULL_RECT)
	c.add_child(_ground())
	var l := Label.new()
	l.text = "Reference 12 px"
	l.add_theme_font_size_override("font_size", 12)
	l.add_theme_color_override("font_color", Color.WHITE)
	l.position = Vector2(20, 20)
	c.add_child(l)
	var b := ColorRect.new()
	b.color = Color.WHITE
	b.position = Vector2(20, 60)
	b.size = Vector2(100, 1)
	c.add_child(b)
	return c
