extends Node
## The same `CheckBox` icon modulate, measured on the palette this machine
## actually boots (light) and through the shell's own theme path, because
## `DccShell._recolor_project_theme()` REWRITES `dark_theme.tres` in memory on
## a flip -- so the resource's contribution is a different colour under each
## palette and only the shell can produce the live one.
##
##   Godot_v4.7.1-stable_win64_console.exe --rendering-driver vulkan \
##     --resolution 320x200 _cklight_probe.tscn
## Not `--headless`: the dummy rasteriser cannot read a pixel back.

var fails := 0
var app: Node
var _vp: SubViewport
var _img: Image

func _ok(cond: bool, what: String) -> void:
	if cond: print("  PASS  ", what)
	else:
		fails += 1
		print("  FAIL  ", what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _hex(c: Color) -> String:
	return "#%02x%02x%02x" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255))]

const GROUND := Color(1, 0, 1)

func _bbox(r: Rect2) -> Rect2i:
	var x0 := 9999; var y0 := 9999; var x1 := -1; var y1 := -1
	for y in range(int(r.position.y), int(r.end.y)):
		for x in range(int(r.position.x), int(r.end.x)):
			if x < 0 or y < 0 or x >= _img.get_width() or y >= _img.get_height():
				continue
			var p := _img.get_pixel(x, y)
			if absf(p.r - GROUND.r) + absf(p.g - GROUND.g) + absf(p.b - GROUND.b) < 0.02:
				continue
			x0 = mini(x0, x); y0 = mini(y0, y); x1 = maxi(x1, x); y1 = maxi(y1, y)
	return Rect2i(x0, y0, x1 - x0 + 1, y1 - y0 + 1)

func _sample(cb: CheckBox, tag: String) -> Dictionary:
	var b := _bbox(cb.get_global_rect())
	var on: bool = cb.button_pressed
	var knob: int = 18 if DccTheme.is_touch() else 13
	var kx: int = int(round((b.size.x - 2.0 - knob * 0.5) if on else (2.0 + knob * 0.5)))
	var tx: int = 3 if on else (b.size.x - 4)
	var cy: int = int(b.size.y * 0.5)
	var k := _img.get_pixel(b.position.x + kx, b.position.y + cy)
	var t := _img.get_pixel(b.position.x + tx, b.position.y + cy)
	print("  [", tag, "] bbox=", b, " knob=", _hex(k), "  track=", _hex(t))
	return {"knob": k, "track": t}

func _near(a: Color, hex: String) -> bool:
	var b := Color(hex)
	return absf(a.r - b.r) < 0.012 and absf(a.g - b.g) < 0.012 and absf(a.b - b.b) < 0.012

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	## Force LIGHT through the shell's own path, so `_recolor_project_theme()`
	## has run and the resource holds its light-palette values.
	if DccTheme.is_dark():
		app.toggle_theme()
		await _frames(4)
	if DccTheme.is_dark():
		print("[FATAL] could not reach the light palette"); get_tree().quit(1); return

	var th: Theme = load("res://theme/dark_theme.tres")
	print("[BOOT] palette=light  resource now supplies:")
	print("   CheckBox/checkbox_checked_color   = ",
		_hex(th.get_color("checkbox_checked_color", "CheckBox")))
	print("   CheckBox/checkbox_unchecked_color = ",
		_hex(th.get_color("checkbox_unchecked_color", "CheckBox")))

	_vp = SubViewport.new()
	_vp.size = Vector2i(300, 170)
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var ground := ColorRect.new()
	ground.color = GROUND
	ground.size = Vector2(300, 170)
	_vp.add_child(ground)
	var col := VBoxContainer.new()
	col.size = Vector2(300, 170)
	col.add_theme_constant_override("separation", 8)
	_vp.add_child(col)
	var on := DccWidgets.toggle(col, "On", true, func(_v): pass)
	var off := DccWidgets.toggle(col, "Off", false, func(_v): pass)
	await _frames(8)
	_img = _vp.get_texture().get_image()
	if _img == null:
		print("[FATAL] no image -- run WITHOUT --headless"); get_tree().quit(1); return

	print("\n[1] The switch as the light palette draws it")
	var s_on := _sample(on, "on ")
	var s_off := _sample(off, "off")

	## `Cartalith DCC Environment.dc.html` light block (`ENV:1818`):
	## `--ink:#111210`, `--acc:#a4650f`. The knob is `background:var(--ink)`
	## (`ENV:76`) and the ON track `var(--acc)` (`ENV:1354`). Literals, pinned.
	print("\n[2] Against the canvas's own light literals")
	_ok(_near(s_on["knob"], "#111210"), "ON knob is the canvas light --ink #111210")
	_ok(_near(s_on["track"], "#a4650f"), "ON track is the canvas light --acc #a4650f")
	_ok(_near(s_off["knob"], "#111210"), "OFF knob is the canvas light --ink #111210")

	## [3] The eighth item, `LineEdit/selection_color`, and the default-theme
	## item that sits on top of it. The resource supplies `--acc` at a=0.35 and
	## `_recolor_project_theme()` DOES remap it (RGB-only match, alpha kept),
	## so the band itself follows the palette. `font_selected_color` does not:
	## it is `#ffffff` from Godot's own default theme, which nothing in this
	## shell remaps -- so the light palette puts white glyphs on a pale tan
	## band. Measured, not computed.
	print("")
	print("[3] LineEdit selection, light palette")
	var le := LineEdit.new()
	le.text = "IIIIIIIIII"
	le.custom_minimum_size = Vector2(160, 24)
	DccWidgets.well(le)
	col.add_child(le)
	await _frames(4)
	le.grab_focus()
	le.deselect_on_focus_loss_enabled = false
	le.select_all()
	await _frames(6)
	_img = _vp.get_texture().get_image()
	var r := le.get_global_rect()
	var band := Color(0, 0, 0)
	var glyph := Color(0, 0, 0)
	var bl := -1.0
	var counts := {}
	for y in range(int(r.position.y) + 4, int(r.end.y) - 4):
		for x in range(int(r.position.x) + 4, int(r.end.x) - 4):
			var p := _img.get_pixel(x, y)
			var key := _hex(p)
			counts[key] = int(counts.get(key, 0)) + 1
			var l := p.r * 0.2126 + p.g * 0.7152 + p.b * 0.0722
			if l > bl:
				bl = l
				glyph = p
	## The band is the most common colour that is NOT the field ground: the
	## selection covers only the glyph run, so the ground still wins a plain
	## histogram (measured: 3832 ground px against a ~30x14 band).
	var field_bg := _hex(DccTheme.c("sunken"))
	var top := ""
	var topn := 0
	for k in counts:
		if String(k) == field_bg:
			continue
		if int(counts[k]) > topn:
			topn = int(counts[k]); top = String(k)
	band = Color(top)
	var lb: float = band.r * 0.2126 + band.g * 0.7152 + band.b * 0.0722
	var lg: float = glyph.r * 0.2126 + glyph.g * 0.7152 + glyph.b * 0.0722
	var ratio: float = (maxf(lb, lg) + 0.05) / (minf(lb, lg) + 0.05)
	print("  selection band = ", top, " (", topn, " px)   brightest glyph = ",
		_hex(glyph), "   contrast = %.2f:1" % ratio)
	_ok(ratio >= 4.5, "selected text clears 4.5:1 on the light palette")

	## [4] Why the two wrong modulates CANNOT be corrected in the Theme
	## resource, which is what makes this a `dcc_widgets.gd` fix and not a
	## `dcc_theme.gd` one. An identity modulate is pure white, and
	## `DccShell._recolor_project_theme()` puts every resource colour through
	## `DccTheme.remap()`. White has no exact-RGBA token, so it falls to the
	## RGB-only pass, where `line` is `Color(1,1,1,.10)` -- an RGB match. The
	## walk therefore rewrites an identity modulate to the LIGHT `line`'s RGB
	## at the original alpha. Computed by the shipping function, not by hand.
	print("")
	print("[4] What a palette flip would do to an identity (white) modulate")
	var got = DccTheme.remap(Color(1, 1, 1, 1), DccTheme.DARK)
	print("  DccTheme.remap(#ffffff, DARK) under the light palette -> ",
		("null" if got == null else _hex(got as Color)))
	_ok(got == null,
		"a white theme-resource entry survives a palette flip untouched")

	print("\n[RESULT] fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
