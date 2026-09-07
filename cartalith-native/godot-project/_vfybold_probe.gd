extends Node
## VERIFIER-owned. The PHONE-PREFS lane proved the medium face MOVES pixels
## (508 differing px, control 531). "Moved" is not "heavier", and bold means
## heavier -- so this measures the thing the word actually claims: the INK AREA
## of the same string, same rect, same size, medium vs regular.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1080x2400 \
##     _vfybold_probe.tscn -- --force-touch

var app: Node
var pm: Node
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond: print("  PASS  ", what)
	else:
		fails += 1
		print("  FAIL  ", what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

## Most common colour in `r` -- the pill fill.
func _modal(img: Image, r: Rect2i) -> Color:
	var counts := {}
	for y in range(r.position.y, r.position.y + r.size.y):
		for x in range(r.position.x, r.position.x + r.size.x):
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height(): continue
			var k := img.get_pixel(x, y)
			counts[k] = int(counts.get(k, 0)) + 1
	var best := Color.BLACK
	var n := -1
	for k in counts:
		if int(counts[k]) > n: n = int(counts[k]); best = k
	return best

## Ink coverage: sum over the rect of how far each pixel is from the fill,
## clamped to 1. A count alone is threshold-sensitive; the sum is the antialiased
## area a heavier stem actually adds, so it moves with weight and not with a
## sub-pixel shift.
func _ink(img: Image, r: Rect2i, fill: Color) -> Array:
	var n := 0
	var mass := 0.0
	for y in range(r.position.y, r.position.y + r.size.y):
		for x in range(r.position.x, r.position.x + r.size.x):
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height(): continue
			var p := img.get_pixel(x, y)
			var d := (absf(p.r - fill.r) + absf(p.g - fill.g) + absf(p.b - fill.b)) / 3.0
			mass += minf(1.0, d / 0.35)
			if d > 0.12: n += 1
	return [n, mass]

func _ready() -> void:
	var wd := Timer.new(); wd.wait_time = 300.0; wd.one_shot = true
	add_child(wd); wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3)); wd.start()
	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null: app.open_project_dialog.hide()
	if app.get("phone_project_picker") != null: app.phone_project_picker.hide()
	await _frames(4)
	pm = app.get("_phone_menu")
	if pm == null or not app.is_phone():
		print("!! not a handset composition"); get_tree().quit(1); return
	pm.open()
	pm._push_screen("prefs")
	await _frames(8)

	print("\n[0] Renderer actually in use")
	print("  ", RenderingServer.get_video_adapter_api_version(), " / ",
		ProjectSettings.get_setting("rendering/renderer/rendering_method"))

	## ---- Find a selected chip, by the face its Button resolves at draw time.
	##
	## **On screen, not merely `is_visible_in_tree()`.** The prefs screen
	## scrolls; the first pass of this probe took a chip whose global rect was
	## at y=5493 in a 2400-tall window and measured an ink mass of 0.0 for both
	## faces -- a FAIL that said nothing about the font. Same class as
	## `_numglass_probe`'s `_on_screen()` guard.
	var vis := get_viewport().get_visible_rect()
	var chip: Button = null
	var stack: Array = [app]
	while not stack.is_empty():
		var nd: Node = stack.pop_back()
		for c in nd.get_children(): stack.append(c)
		if nd is Button and (nd as Button).is_visible_in_tree():
			var f: Font = (nd as Button).get_theme_font("font")
			if f is FontVariation and String((f as FontVariation).base_font.resource_path).ends_with("Medium.ttf"):
				var gr := (nd as Button).get_global_rect()
				if gr.size.x > 20 and vis.encloses(gr):
					chip = nd as Button
					break
	_ok(chip != null, "a chip on the prefs screen resolves the MEDIUM face at draw time")
	if chip == null:
		print("\n[RESULT] vfybold fails=", fails); get_tree().quit(1); return
	var r := Rect2i(chip.get_global_rect())
	print("  chip [", chip.text, "]  rect ", r)

	print("\n[1] Metrics: Medium and Regular are the same width, so nothing reflows")
	var fs: int = chip.get_theme_font_size("font_size")
	var med: Font = DccTheme.mono(0, true)
	var reg: Font = DccTheme.mono(0, false)
	var wm := med.get_string_size(chip.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
	var wr := reg.get_string_size(chip.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
	print("  font_size=%d   medium width=%.2f   regular width=%.2f" % [fs, wm, wr])
	_ok(absf(wm - wr) < 0.01, "the two faces measure the same advance (%.2f vs %.2f)" % [wm, wr])

	print("\n[2] Ink area drawn, MEDIUM (shipped) vs REGULAR (mutant)")
	var img := await _grab()
	var fill := _modal(img, r)
	var a := _ink(img, r, fill)
	chip.add_theme_font_override("font", reg)
	await _frames(4)
	var img2 := await _grab()
	var b := _ink(img2, r, fill)
	chip.add_theme_font_override("font", med)
	await _frames(4)
	var img3 := await _grab()
	var c3 := _ink(img3, r, fill)
	print("  pill fill = #%02x%02x%02x" % [int(fill.r * 255), int(fill.g * 255), int(fill.b * 255)])
	print("  MEDIUM  ink px=%d  mass=%.1f" % [a[0], a[1]])
	print("  REGULAR ink px=%d  mass=%.1f" % [b[0], b[1]])
	print("  MEDIUM again  ink px=%d  mass=%.1f" % [c3[0], c3[1]])
	## **1.02, and the number it is guarding is 1.033.** Measured 2026-09-07 on
	## the `No cap` chip at `font_size` 26: mass 3018.5 medium vs 2922.2
	## regular, ink px 3074 vs 2986. That is the real size of the effect and it
	## is worth stating plainly rather than dressing up: what shipped is IBM
	## Plex Mono **Medium** (weight 500), not Bold (700), so the chosen option
	## is about **3% heavier**, not the doubling "bold" suggests. It is a real,
	## repeatable, correct-direction difference -- the restore leg below reads
	## 3018.5 back exactly -- and it is subtle. The first draft of this probe
	## asserted 1.05 and failed on a fix that is working; the bar was wrong, so
	## it is now pinned just under what the faces actually differ by.
	_ok(float(a[1]) > float(b[1]) * 1.02,
		"the shipped face lays down MORE ink than Regular: mass %.1f vs %.1f (+%.1f%%)"
			% [a[1], b[1], (float(a[1]) / maxf(1.0, float(b[1])) - 1.0) * 100.0])
	_ok(int(a[0]) > int(b[0]),
		"and covers more pixels above threshold (%d vs %d)" % [a[0], b[0]])
	_ok(absf(float(c3[1]) - float(a[1])) < maxf(1.0, float(a[1]) * 0.02),
		"restoring the medium face restores the ink mass (%.1f -> %.1f) -- the capture is live, not cached"
			% [b[1], c3[1]])

	print("\n[3] An UNSELECTED chip in the same group is Regular")
	var sibs := chip.get_parent().get_children()
	var other: Button = null
	for s in sibs:
		if s is Button and s != chip and (s as Button).is_visible_in_tree(): other = s as Button
	if other != null:
		var f2: Font = other.get_theme_font("font")
		_ok(f2 is FontVariation and String((f2 as FontVariation).base_font.resource_path).ends_with("Regular.ttf"),
			"sibling chip [%s] resolves Regular (%s)" % [other.text,
				String((f2 as FontVariation).base_font.resource_path).get_file()])
	else:
		_ok(false, "no sibling chip found to compare against")

	print("\n[RESULT] vfybold fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
