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

func _l(c: Color) -> float:
	return c.r * 0.2126 + c.g * 0.7152 + c.b * 0.0722

func _cr(a: Color, b: Color) -> float:
	return (maxf(_l(a), _l(b)) + 0.05) / (minf(_l(a), _l(b)) + 0.05)

## The pixel in `r` whose luminance is furthest from `ref_lum` -- the ink,
## whichever way round the palette runs.
func _extreme(r: Rect2i, ref_lum: float) -> Color:
	var best := Color(0, 0, 0)
	var bd := -1.0
	for y in range(r.position.y, r.position.y + r.size.y):
		for x in range(r.position.x, r.position.x + r.size.x):
			if x < 0 or y < 0 or x >= _img.get_width() or y >= _img.get_height():
				continue
			var p := _img.get_pixel(x, y)
			if absf(_l(p) - ref_lum) > bd:
				bd = absf(_l(p) - ref_lum)
				best = p
	return best

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
	await _frames(6)
	## **The control, captured before anything is selected.** A contrast number
	## on its own cannot say whose defect it is, and at `FS_TINY` = 10 px a
	## Plex Mono stem never reaches full coverage -- the darkest pixel in this
	## field is an antialiased blend, not `text_bright` itself. So the same
	## measurement is taken on the UNSELECTED run first and the two are
	## compared: selecting text must not change its ink.
	_img = _vp.get_texture().get_image()
	var r0 := le.get_global_rect()
	var unsel_ground := DccTheme.c("sunken")
	var unsel_ink := _extreme(Rect2i(Vector2i(r0.position) + Vector2i(4, 4),
		Vector2i(r0.size) - Vector2i(8, 8)), _l(unsel_ground))
	print("  unselected: ground=", _hex(unsel_ground), "  darkest ink=",
		_hex(unsel_ink), "  contrast=%.2f:1" % _cr(unsel_ground, unsel_ink))
	le.grab_focus()
	le.deselect_on_focus_loss_enabled = false
	le.select_all()
	await _frames(6)
	_img = _vp.get_texture().get_image()
	var r := le.get_global_rect()
	var band := Color(0, 0, 0)
	var glyph := Color(0, 0, 0)
	var counts := {}
	for y in range(int(r.position.y) + 4, int(r.end.y) - 4):
		for x in range(int(r.position.x) + 4, int(r.end.x) - 4):
			var key := _hex(_img.get_pixel(x, y))
			counts[key] = int(counts.get(key, 0)) + 1
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
	## **The glyph is the pixel FURTHEST from the band in luminance, not the
	## brightest.** "Brightest" is a dark-palette habit and it silently broke
	## here the moment the fix landed: with the ink corrected to `--ink`
	## `#111210` the darkest thing in the field became the text and the
	## brightest became the field ground `#eceae4`, so the old measure
	## reported 1.22:1 for a pair that is genuinely 10:1. Search only inside
	## the band's own bounding box, so the ground outside the selection run
	## cannot be mistaken for ink either.
	var bx0 := 99999; var by0 := 99999; var bx1 := -1; var by1 := -1
	for y in range(int(r.position.y) + 4, int(r.end.y) - 4):
		for x in range(int(r.position.x) + 4, int(r.end.x) - 4):
			if _hex(_img.get_pixel(x, y)) != top:
				continue
			bx0 = mini(bx0, x); by0 = mini(by0, y)
			bx1 = maxi(bx1, x); by1 = maxi(by1, y)
	glyph = _extreme(Rect2i(bx0, by0, bx1 - bx0 + 1, by1 - by0 + 1), lb)
	var lg: float = _l(glyph)
	var ratio: float = _cr(band, glyph)
	print("  selection band = ", top, " (", topn, " px)   selected ink = ",
		_hex(glyph), "   contrast = %.2f:1" % ratio)
	## Three checks, and each answers something the others cannot.
	##
	## (a) **Selecting does not change how well the text reads.** Not the same
	##     as "the ink pixel is identical": at 10 px no stem reaches full
	##     coverage, so the darkest pixel is always a blend of the ink with
	##     whatever is behind it -- `#3f3f3c` over the field ground, `#39352d`
	##     over the band -- and those two can never be equal even when the ink
	##     is. What must hold is the *ratio*, which is what a reader gets:
	##     measured 3.27:1 unselected against 3.07:1 selected. With the
	##     default theme's white the selected run measures 1.24:1, two whole
	##     points away, so the tolerance cannot be passed by the defect.
	## (b) **Direction.** On the light palette the ink must be DARKER than the
	##     band. White-on-tan fails this by sign, so it cannot be passed by a
	##     threshold that happens to be loose.
	## (c) **The pair that WCAG actually governs**: the declared ink against
	##     the band as measured off the screen. Only one side is a token here,
	##     so this is not a constant asserted against itself -- flip
	##     `font_selected_color` back to white and the measured band stays put
	##     while the ratio collapses.
	var unsel_ratio := _cr(unsel_ground, unsel_ink)
	_ok(absf(ratio - unsel_ratio) < 0.5,
		"selecting holds the field's legibility (%.2f:1 selected vs %.2f:1 unselected)"
			% [ratio, unsel_ratio])
	_ok(lg < lb, "on the light palette the selected ink is darker than the band")
	var declared := DccTheme.c("text_bright")
	print("  declared font_selected_color = ", _hex(declared),
		"   against the measured band: %.2f:1" % _cr(declared, band))
	_ok(_cr(declared, band) >= 4.5,
		"the declared selected ink clears 4.5:1 against the measured band")

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
	## **The other side of that guard, and it was added because a mutant
	## survived without it.** Widening the skip from "an opaque value" to
	## "every value" passed every check above -- the switch, the selection and
	## the white case are all blind to it -- while silently stranding the
	## three white hairlines `dark_theme.tres` carries at a=.05/.06/.14/.20
	## against `line`'s .10 as WHITE lines on the light palette. So the pass
	## the guard must NOT break is pinned here too, in both of its shapes:
	## a translucent value against a translucent token, and a translucent
	## value against an OPAQUE one. Literals on both sides -- `DARK`'s
	## `line` is `Color(1,1,1,.10)` and `LIGHT`'s is `Color(0,0,0,.14)`;
	## `--acc` is `#e0a34a` dark and `#a4650f` light (`ENV:25`, `ENV:1818`).
	var hair = DccTheme.remap(Color(1, 1, 1, 0.14), DccTheme.DARK)
	print("  remap(rgba(255,255,255,.14), DARK) -> ",
		("null" if hair == null else "%s a=%.2f" % [_hex(hair as Color),
			(hair as Color).a]))
	_ok(hair != null and _hex(hair as Color) == "#000000"
			and absf((hair as Color).a - 0.14) < 0.01,
		"a translucent white hairline still follows the palette to #000000 a=.14")
	var wash = DccTheme.remap(Color(Color("#e0a34a"), 0.35), DccTheme.DARK)
	print("  remap(rgba(--acc dark,.35), DARK) -> ",
		("null" if wash == null else "%s a=%.2f" % [_hex(wash as Color),
			(wash as Color).a]))
	_ok(wash != null and _hex(wash as Color) == "#a4650f"
			and absf((wash as Color).a - 0.35) < 0.01,
		"an accent derivative still follows the palette to #a4650f a=.35")

	## [5] The leg the standing rule exists for: **flip the palette under
	## switches that already exist.** Everything above built its controls after
	## the palette was settled, which cannot tell an identity override that
	## survives a flip from one that was simply written last. These same two
	## instances go back to dark here, and the assertion is a value that
	## DIFFERS between palettes -- light `--ink` `#111210` against dark `--ink`
	## `#e8ebec` -- so a repaint that silently did nothing fails it.
	print("")
	print("[5] The SAME instances, flipped back to dark")
	app.toggle_theme()
	await _frames(8)
	if not DccTheme.is_dark():
		print("[FATAL] could not reach the dark palette"); get_tree().quit(1); return
	_img = _vp.get_texture().get_image()
	var f_on := _sample(on, "on ")
	var f_off := _sample(off, "off")
	_ok(_near(f_on["knob"], "#e8ebec"),
		"the ON knob repainted to the dark --ink #e8ebec (got %s)" % _hex(f_on["knob"]))
	_ok(_near(f_on["track"], "#e0a34a"),
		"the ON track repainted to the dark --acc #e0a34a (got %s)" % _hex(f_on["track"]))
	_ok(_near(f_off["knob"], "#e8ebec"),
		"the OFF knob repainted to the dark --ink #e8ebec (got %s)" % _hex(f_off["knob"]))
	## Restore the machine's own preference, which is `light`.
	app.toggle_theme()
	await _frames(4)
	print("  (palette left at ", "dark" if DccTheme.is_dark() else "light", ")")

	print("\n[RESULT] fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
