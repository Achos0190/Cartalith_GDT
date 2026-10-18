extends Node
## WINDOWED, IN THE REAL SHELL: AP-10 (`ASSET_PACK_TOOLS_PROPOSAL.md` §9.3,
## Ruling CR) -- the Asset library inspector's REFINE ▸ line-art block, which
## converts a sprite variant's drawing into a clean one-ink sprite as a NEW
## variant (Ruling CQ Q7), and its Undo.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _lineart_probe.tscn
##   phone composition: ... _lineart_probe.tscn -- --vp 1080x2400 --force-touch
##
## `--vp WxH` hosts the app in a SubViewport of that size (`_seamfix_probe`'s
## pattern); any other argument aborts (exit 2). Windowed only: it saves
## screenshots for a human to read, and a headless run refuses (exit 2).
##
## The input is drawn here: a 1024² "bad scan" -- cream paper with a 25 %
## illumination fall-off and a pencil-grey drawing (a box with a window, a
## solid door, a 2 px line) -- so the engine's scan detection has something
## to detect.
##
## Steps, each asserted:
##   1. the drawing imports into Features ▸ mountain; focusing it shows REFINE
##      with the line-art block, the seam rows hidden, Convert enabled, and
##      three ink swatches whose colours are the live Look's (or the labelled
##      fallbacks under a look with no treatment);
##   2. choosing ink 2 and weight 1.5 then pressing Convert (its own `pressed`
##      signal) adds exactly one variant, leaves variant 1's hash and pixels
##      as they were, previews the new one, words the engine's result (ink
##      parts, read as a scan), and shows Undo; every inked pixel of the new
##      variant's 256² thumbnail is the chosen ink within one level (the
##      bake's known soft-alpha drift), and its paper is transparent;
##   3. Undo removes the new variant and nothing else;
##   4. a ground texture hides the line-art block and shows the seam rows;
##   5. blank paper is refused in words and adds nothing.
## Prints `LINEART ok|FAIL <what>`, `LINEART saved <png>` and
## `LINEART checks=<n> failures=<n>`. Exit 0 / 1 / 2.

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String, detail: String = "") -> void:
	_checks += 1
	print("LINEART %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://lineart")
	var path := "user://lineart/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else "desktop"]
	img.save_png(path)
	print("LINEART saved %s" % ProjectSettings.globalize_path(path))


## The probe's input: see the header. Deterministic (no randomness).
static func _scan_png() -> PackedByteArray:
	var n := 1024
	var img := Image.create_empty(n, n, false, Image.FORMAT_RGBA8)
	for y in n:
		for x in n:
			var light := 1.0 - 0.25 * (float(x) / n) * (float(y) / n)
			var ink := false
			# box outline 8 px, a window 6 px, a solid door, a 2 px line
			if (x >= 200 and x < 824 and ((y >= 300 and y < 308) or (y >= 816 and y < 824))) \
					or (y >= 300 and y < 824 and ((x >= 200 and x < 208) or (x >= 816 and x < 824))):
				ink = true
			if (x >= 300 and x < 420 and ((y >= 400 and y < 406) or (y >= 520 and y < 526))) \
					or (y >= 400 and y < 526 and ((x >= 300 and x < 306) or (x >= 414 and x < 420))):
				ink = true
			if x >= 560 and x < 700 and y >= 600 and y < 816:
				ink = true
			if y >= 200 and y < 202 and x >= 150 and x < 874:
				ink = true
			var c := Color(0.36, 0.36, 0.38) if ink else Color(0.95, 0.92, 0.85)
			img.set_pixel(x, y, Color(c.r * light, c.g * light, c.b * light, 1.0))
	return img.save_png_to_buffer()


static func _blank_png() -> PackedByteArray:
	var img := Image.create_empty(256, 256, false, Image.FORMAT_RGBA8)
	img.fill(Color(1, 1, 1, 1))
	return img.save_png_to_buffer()


static func _flat_png() -> PackedByteArray:
	var img := Image.create_empty(512, 512, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.4, 0.5, 0.3, 1.0))
	return img.save_png_to_buffer()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	var vp := Vector2i.ZERO
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			vp = Vector2i(int(parts[0]), int(parts[1]))
			i += 2
			continue
		if args[i] == "--force-touch":
			i += 1
			continue
		print("LINEART ABORT: unknown argument '%s'" % args[i])
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("LINEART REFUSED: headless (it saves screenshots; run windowed)")
		get_tree().quit(2)
		return
	app = load("res://shell/app.tscn").instantiate()
	_view = get_viewport()
	if vp != Vector2i.ZERO:
		var sv := SubViewport.new()
		sv.size = vp
		sv.gui_embed_subwindows = true
		sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(sv)
		Input.set_emulate_touch_from_mouse(true)
		sv.add_child(app)
		_view = sv
	else:
		add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
		await _frames(2)
	var b = app.bridge
	if not b.world_gen.has_method("as_line_art"):
		print("LINEART CANNOT RUN: this DLL has no as_line_art")
		get_tree().quit(2)
		return
	print("LINEART view=%s palette=%s phone=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone()])
	b.as_clear_library()
	var w: AssetLibraryWindow = app.asset_library_window

	# -- 1 ------------------------------------------------------------------
	var imp: Dictionary = b.as_import_item("icons:mountain", "drawing.png", _scan_png())
	_ok(bool(imp.get("ok", false)), "the drawing imports into Features ▸ mountain", str(imp))
	_ok(w.focus_slot("icons:mountain"), "focus_slot reaches the mountain slot")
	await _frames(4)
	var refine: Control = w.get("_insp_refine")
	var la = w.get("_lineart")
	_ok(la != null, "the window built the line-art block")
	if la == null:
		print("LINEART checks=%d failures=%d" % [_checks, _fails])
		get_tree().quit(1)
		return
	var seam_label: Label = w.get("_insp_seam_label")
	var convert: Button = la.get("_convert_btn")
	var undo: Button = la.get("_undo_btn")
	var readout: Label = la.get("_readout")
	_ok(refine.is_visible_in_tree() and (la.box as Control).is_visible_in_tree(), "REFINE shows the line-art block for a sprite")
	_ok(not seam_label.is_visible_in_tree(), "the seam rows are hidden for a sprite")
	_ok(not convert.disabled, "Convert is enabled on a variant with art")
	var inks: Array = la.look_inks()
	var sw: Array = la.get("_ink_swatches")
	_ok(inks.size() == 3 and sw.size() == 3 and la.swatch_colour(sw[1]) == inks[1][1],
		"three ink swatches, drawn in the inks the engine will be given", str(inks))
	var h0 := String(b.as_item_summary("icons:mountain", 0).get("hash", ""))
	var t0: PackedByteArray = b.as_thumbnail_png("icons:mountain", 0, 256)
	var sc_parent: Node = refine.get_parent()
	while sc_parent != null and not (sc_parent is ScrollContainer):
		sc_parent = sc_parent.get_parent()
	if sc_parent != null:
		(sc_parent as ScrollContainer).ensure_control_visible(convert)
		await _frames(3)
	await _snap("01_before")

	# -- 2 ------------------------------------------------------------------
	(la.get("_ink_buttons")[1] as Button).pressed.emit()
	(la.get("_weight_buttons")[1.5] as Button).pressed.emit()
	await _frames(2)
	convert.pressed.emit()
	await _frames(4)
	var s: Dictionary = b.as_slot_summary("icons:mountain")
	_ok(int(s.get("item_count", 0)) == 2, "Convert added exactly one variant", str(s.get("item_count")))
	_ok(String(b.as_item_summary("icons:mountain", 0).get("hash", "")) == h0
		and b.as_thumbnail_png("icons:mountain", 0, 256) == t0, "variant 1 is untouched (hash and pixels)")
	_ok(int(w.get("_preview_index")) == 1, "the new variant is previewed")
	_ok(readout.text.begins_with("Line art — ") and readout.text.contains("ink part") and readout.text.contains("read as a scan"),
		"the readout words the engine's result, including the scan detection", readout.text)
	_ok(undo.is_visible_in_tree(), "Undo is offered")
	var ink: Color = inks[1][1]
	var want := [int(round(ink.r * 255)), int(round(ink.g * 255)), int(round(ink.b * 255))]
	var timg := Image.new()
	timg.load_png_from_buffer(b.as_thumbnail_png("icons:mountain", 1, 256))
	var inked := 0
	var off := 0
	var clear := 0
	for y in timg.get_height():
		for x in timg.get_width():
			var p := timg.get_pixel(x, y)
			var a8 := int(round(p.a * 255))
			if a8 == 0:
				clear += 1
				continue
			inked += 1
			var got := [int(round(p.r * 255)), int(round(p.g * 255)), int(round(p.b * 255))]
			for c in 3:
				if absi(int(got[c]) - int(want[c])) > 1:
					off += 1
					break
	_ok(inked > 500 and off == 0, "every inked pixel is the chosen ink (within the bake's 1-level drift)", "inked=%d off=%d" % [inked, off])
	_ok(clear > 256 * 256 / 2, "the paper is transparent, not kept (no halo)", "clear=%d" % clear)
	## Dark ink on the dark checker is hard to read in a capture; show the
	## preview on white the way a user would (the inspector's own Preview bg
	## row, its first swatch), then bring the readout into view.
	var bg_row: Array = w.get("_insp_swatches")
	if bg_row != null and bg_row.size() > 0:
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.pressed = true
		(bg_row[0] as Control).gui_input.emit(click)
		await _frames(2)
	if sc_parent != null:
		(sc_parent as ScrollContainer).ensure_control_visible(readout)
		await _frames(3)
	await _snap("02_converted")

	# -- 3 ------------------------------------------------------------------
	undo.pressed.emit()
	await _frames(4)
	s = b.as_slot_summary("icons:mountain")
	_ok(int(s.get("item_count", 0)) == 1 and String(b.as_item_summary("icons:mountain", 0).get("hash", "")) == h0,
		"Undo removed the new variant and kept the original")
	_ok(not undo.is_visible_in_tree(), "Undo is withdrawn after use")

	# -- 4 ------------------------------------------------------------------
	b.as_import_item("biomes:jungle", "flat.png", _flat_png())
	w.focus_slot("biomes:jungle")
	await _frames(3)
	_ok(not (la.box as Control).is_visible_in_tree() and seam_label.is_visible_in_tree(),
		"a ground texture shows the seam rows and hides the line-art block")

	# -- 5 ------------------------------------------------------------------
	b.as_import_item("icons:hill", "blank.png", _blank_png())
	w.focus_slot("icons:hill")
	await _frames(3)
	convert.pressed.emit()
	await _frames(3)
	_ok(int(b.as_slot_summary("icons:hill").get("item_count", 0)) == 1 and readout.text.contains("no ink found"),
		"blank paper is refused in words and adds nothing", readout.text)

	print("LINEART checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
