extends Node
## PIXEL evidence for the `CheckBox` icon modulate.
##
##   Godot_v4.7.1-stable_win64_console.exe --rendering-driver vulkan \
##     --resolution 320x200 _ckpix_probe.tscn
##
## **Not `--headless`.** The dummy rasteriser returns a null texture from
## `SubViewport.get_texture().get_image()` (`texture_2d_get: Parameter "t" is
## null`), so a headless run cannot read a pixel back at all -- which is
## exactly how a stylebox-shaped test came to pass while the pixels were wrong.
## A real Vulkan context is the only configuration that can answer this.

var fails := 0
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

## Bounding box of everything drawn over the magenta ground inside `r`.
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

## The switch texture `DccWidgets._switch()` builds is `w x h` with the knob
## centred at `w-2-knob/2` when on and `2+knob/2` when off, `h/2` down. The
## track is sampled at the OPPOSITE end, one radius clear of the knob.
func _sample(cb: CheckBox, tag: String) -> Dictionary:
	var b := _bbox(cb.get_global_rect())
	var on: bool = cb.button_pressed
	var w: int = b.size.x
	var h: int = b.size.y
	var knob: int = 18 if DccTheme.is_touch() else 13
	var kx: int = int(round((w - 2.0 - knob * 0.5) if on else (2.0 + knob * 0.5)))
	var tx: int = 3 if on else (w - 4)
	var cy: int = int(h * 0.5)
	var knob_px := _img.get_pixel(b.position.x + kx, b.position.y + cy)
	var track_px := _img.get_pixel(b.position.x + tx, b.position.y + cy)
	print("  [", tag, "] bbox=", b, " knob@x+", kx, "=", _hex(knob_px),
		"  track@x+", tx, "=", _hex(track_px))
	return {"knob": knob_px, "track": track_px}

## Brightest non-ground pixel in `b` -- a glyph stem at full coverage.
func _brightest(b: Rect2i) -> Color:
	var best := Color(0, 0, 0)
	var bl := -1.0
	for y in range(b.position.y, b.position.y + b.size.y):
		for x in range(b.position.x, b.position.x + b.size.x):
			var p := _img.get_pixel(x, y)
			if absf(p.r - GROUND.r) + absf(p.g - GROUND.g) + absf(p.b - GROUND.b) < 0.02:
				continue
			var l := p.r * 0.2126 + p.g * 0.7152 + p.b * 0.0722
			if l > bl:
				bl = l
				best = p
	return best

func _near(a: Color, hex: String) -> bool:
	var b := Color(hex)
	return absf(a.r - b.r) < 0.012 and absf(a.g - b.g) < 0.012 and absf(a.b - b.b) < 0.012

func _ready() -> void:
	await _frames(2)
	DccTheme.apply_theme(true)
	print("[BOOT] godot ", Engine.get_version_info()["string"], "  dark=",
		DccTheme.is_dark(), " touch=", DccTheme.is_touch())

	_vp = SubViewport.new()
	_vp.size = Vector2i(300, 340)
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var ground := ColorRect.new()
	ground.color = GROUND
	ground.size = Vector2(300, 340)
	_vp.add_child(ground)
	var col := VBoxContainer.new()
	col.size = Vector2(300, 340)
	col.add_theme_constant_override("separation", 8)
	_vp.add_child(col)

	var on := DccWidgets.toggle(col, "On", true, func(_v): pass)
	var off := DccWidgets.toggle(col, "Off", false, func(_v): pass)
	## Control A: force the modulate to white. If the modulate is live, this
	## is the ONLY difference between this switch and `on`.
	var white := DccWidgets.toggle(col, "White", true, func(_v): pass)
	white.add_theme_color_override("checkbox_checked_color", Color(1, 1, 1))
	## Control B: force it to pure red -- the other direction of the same
	## mutation. A live modulate zeroes green and blue.
	var red := DccWidgets.toggle(col, "Red", true, func(_v): pass)
	red.add_theme_color_override("checkbox_checked_color", Color(1, 0, 0))
	await _frames(8)
	_img = _vp.get_texture().get_image()
	if _img == null:
		print("[FATAL] no image -- run WITHOUT --headless"); get_tree().quit(1); return

	print("\n[1] What the resource's modulate does to the drawn switch")
	var s_on := _sample(on, "on    ")
	var s_off := _sample(off, "off   ")
	var s_wh := _sample(white, "white ")
	var s_rd := _sample(red, "red   ")

	## Canvas literals, `Cartalith DCC Environment.dc.html`: the knob is
	## `background:var(--ink)` (`ENV:76`, `:365`, `:846`) and `--ink` is
	## `#e8ebec` (`ENV:25`); the ON track is `var(--acc)` = `#e0a34a`
	## (`ENV:1354`). Pinned as literals, not as `DccTheme.c()`.
	print("\n[2] Against the canvas literals")
	_ok(_near(s_on["knob"], "#e8ebec"), "ON knob is the canvas --ink #e8ebec")
	_ok(_near(s_on["track"], "#e0a34a"), "ON track is the canvas --acc #e0a34a")
	_ok(_near(s_off["knob"], "#e8ebec"), "OFF knob is the canvas --ink #e8ebec")

	print("\n[3] Mutation: the modulate is live, in BOTH directions")
	_ok(not _near(s_rd["knob"], _hex(s_on["knob"])),
		"forcing checkbox_checked_color=red moves the knob")
	_ok(s_rd["knob"].g < 0.10 and s_rd["knob"].b < 0.10,
		"red modulate zeroes the knob's green/blue (g=%.3f b=%.3f)"
			% [s_rd["knob"].g, s_rd["knob"].b])
	_ok(_near(s_wh["knob"], "#e8ebec"),
		"forcing checkbox_checked_color=white restores the canvas --ink knob")

	## [4] The inert/load-bearing split for the six `CheckBox` FONT items,
	## read off the pixels rather than off `cb.text`. A `toggle()` switch
	## paints exactly its 30x17 texture and nothing else, so no font colour of
	## any value can reach the screen through it; a BARE `CheckBox` -- the form
	## `travel_library_window.gd` and `journey_planner_view.gd` build -- paints
	## glyphs, and those glyphs carry the resource's own `#c8cbcd`.
	print("")
	print("[4] Which population the CheckBox font items actually reach")
	var bare := CheckBox.new()
	bare.text = "IIII"
	bare.add_theme_font_size_override("font_size", 13)
	col.add_child(bare)
	var bare_mut := CheckBox.new()
	bare_mut.text = "IIII"
	bare_mut.add_theme_font_size_override("font_size", 13)
	bare_mut.add_theme_color_override("font_color", Color("#3fa9f5"))
	col.add_child(bare_mut)
	await _frames(6)
	_img = _vp.get_texture().get_image()
	var b_on := _bbox(on.get_global_rect())
	_ok(b_on.size.x == 30 and b_on.size.y == 17,
		"a toggle() switch paints exactly the 30x17 texture, no glyph (%s)" % b_on.size)
	var lit := _brightest(_bbox(bare.get_global_rect()))
	print("  [bare  ] brightest glyph pixel = ", _hex(lit))
	_ok(_near(lit, "#c8cbcd"),
		"a BARE CheckBox draws the resource's CheckBox/font_color #c8cbcd")
	var lit2 := _brightest(_bbox(bare_mut.get_global_rect()))
	print("  [bare* ] brightest glyph pixel = ", _hex(lit2))
	_ok(_near(lit2, "#3fa9f5"),
		"overriding font_color to #3fa9f5 moves those glyphs -- the item is live")

	## [5] Does ONE pair of theme colours cover all FOUR icon slots?
	## `_paint_switch()` overrides `checked`, `unchecked`, `checked_disabled`
	## and `unchecked_disabled`. If Godot 4.7 modulates the disabled pair with
	## the same two colours, the fix is two lines; if it has its own items, it
	## is four. Measured rather than assumed.
	print("")
	print("[5] The disabled pair")
	var dis_on := DccWidgets.toggle(col, "DisOn", true, func(_v): pass)
	dis_on.disabled = true
	var dis_off := DccWidgets.toggle(col, "DisOff", false, func(_v): pass)
	dis_off.disabled = true
	var dis_fix := DccWidgets.toggle(col, "DisFix", true, func(_v): pass)
	dis_fix.disabled = true
	dis_fix.add_theme_color_override("checkbox_checked_color", Color(1, 1, 1))
	dis_fix.add_theme_color_override("checkbox_unchecked_color", Color(1, 1, 1))
	await _frames(6)
	_img = _vp.get_texture().get_image()
	var d1 := _sample(dis_on, "dis on ")
	var d2 := _sample(dis_off, "dis off")
	var d3 := _sample(dis_fix, "dis fix")
	## `_switch(enabled=false)` bakes track `sunken` #191c1e, knob
	## `text_ghost` #5f6468 -- `DccTheme.DARK`'s own values, pinned here as
	## the literals they are.
	_ok(not _near(d1["knob"], "#5f6468"),
		"the resource's modulate reaches the DISABLED icons too (knob %s, not #5f6468)"
			% _hex(d1["knob"]))
	_ok(_near(d3["knob"], "#5f6468"),
		"the SAME two overrides restore the disabled knob -- two lines, not four")
	print("  (disabled off knob = ", _hex(d2["knob"]), ")")

	print("\n[RESULT] fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
