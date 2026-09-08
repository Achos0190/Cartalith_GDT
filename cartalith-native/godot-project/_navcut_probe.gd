extends Node
## Lane NAVBAR forensic probe -- OnePlus screenshot, GENERATE tab, 1080x2340:
## bottom nav bar opaque band begins y=2116; the PIPELINE/SCULPT mode-switch
## chip is ~85% hidden behind it (only the top ~3 px of a ~20 px label shows).
##
## Reproduces the shell's own phone composition WINDOWED at the device's exact
## resolution (MISTAKES.md, "Assert on pixels": headless returns a null
## texture) and measures every "PIPELINE"/"SCULPT" instance this shell draws --
## the GENERATE sheet's mode segment (`WorldWorkspace._pg_mode_segment`, inside
## `_phone_gen_col`) and the LEFT DOCK's own mode switch
## (`DccShell._build_mode_switch()`) -- against both their RECT (unclipped,
## MISTAKES.md line ~119) and the ACTUAL DRAWN pixels (accent-colour row scan,
## same method the device measurement used).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _navcut_probe.tscn -- --force-touch
##
## Not headless. Real window, real render, real `get_viewport().get_texture()`.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[navcut] ", s)

func _find_all(root: Node, pred: Callable, out: Array) -> void:
	for c in root.get_children():
		if pred.call(c):
			out.append(c)
		_find_all(c, pred, out)

func _buttons_with_text(root: Node, text: String) -> Array:
	var out: Array = []
	_find_all(root, func(n): return n is Button and String((n as Button).text) == text, out)
	return out

func _find_label(root: Node, text: String) -> Label:
	var out: Array = []
	_find_all(root, func(n): return n is Label and String((n as Label).text) == text, out)
	return out[0] if not out.is_empty() else null

func _rect_str(r: Rect2) -> String:
	return "y=%.0f..%.0f x=%.0f..%.0f (%.0fx%.0f)" % [
		r.position.y, r.end.y, r.position.x, r.end.x, r.size.x, r.size.y]

## MISTAKES.md ~line 119: `get_global_rect()` reports the UNCLIPPED rect --
## assert the drawn rect against the clipping ancestor's own visible rect.
func _clip_fraction(btn: Control, clip: Control) -> Dictionary:
	var br := btn.get_global_rect()
	var cr := clip.get_global_rect()
	var inter := br.intersection(cr)
	var frac := 0.0
	if br.size.y > 0.0:
		frac = inter.size.y / br.size.y
	return {"button": br, "clip_by": cr, "visible": inter, "frac": frac}

## Row-wise accent-pixel count across [x0,x1) for y in [y0,y1) -- the same
## "amber pixel count per row" method the device capture used. Returns the
## first/last row with any accent pixel and the per-row counts.
func _accent_rows(img: Image, x0: int, x1: int, y0: int, y1: int, accent: Color) -> Dictionary:
	var w := img.get_width()
	var h := img.get_height()
	x0 = clampi(x0, 0, w - 1)
	x1 = clampi(x1, 0, w)
	y0 = clampi(y0, 0, h - 1)
	y1 = clampi(y1, 0, h)
	var counts: Array = []
	var first := -1
	var last := -1
	for y in range(y0, y1):
		var n := 0
		for x in range(x0, x1):
			var p := img.get_pixel(x, y)
			var d := absf(p.r - accent.r) + absf(p.g - accent.g) + absf(p.b - accent.b)
			if d < 0.30:
				n += 1
		counts.append(n)
		if n > 0:
			if first < 0:
				first = y
			last = y
	return {"y0": y0, "y1": y1, "counts": counts, "first": first, "last": last}

## Row-wise "is there ANY ink here" scan against a measured background
## sample, colour-agnostic (catches the label whichever of accent/line/
## text_secondary it is actually drawn in). `bg` is a pixel sampled from a
## definitely-empty spot just outside the region under test.
func _ink_rows(img: Image, x0: int, x1: int, y0: int, y1: int, bg: Color) -> Dictionary:
	var w := img.get_width()
	var h := img.get_height()
	x0 = clampi(x0, 0, w - 1)
	x1 = clampi(x1, 0, w)
	y0 = clampi(y0, 0, h - 1)
	y1 = clampi(y1, 0, h)
	var counts: Array = []
	var first := -1
	var last := -1
	for y in range(y0, y1):
		var n := 0
		for x in range(x0, x1):
			var p := img.get_pixel(x, y)
			var d := absf(p.r - bg.r) + absf(p.g - bg.g) + absf(p.b - bg.b)
			if d > 0.06:
				n += 1
		counts.append(n)
		if n > 0:
			if first < 0:
				first = y
			last = y
	return {"y0": y0, "y1": y1, "counts": counts, "first": first, "last": last}

func _print_rows(tag: String, r: Dictionary) -> void:
	var counts: Array = r["counts"]
	var y0: int = r["y0"]
	_log("  %s accent-row scan y=%d..%d  first_accent=%d last_accent=%d" %
		[tag, y0, r["y1"], r["first"], r["last"]])
	## Print every row from 10 above first_accent to 10 below last_accent, so
	## the transition itself (taper vs flat cut) is on the record verbatim.
	if r["first"] < 0:
		_log("    (no accent pixels found in this band at all)")
		return
	var lo: int = maxi(y0, int(r["first"]) - 5)
	var hi: int = mini(y0 + counts.size(), int(r["last"]) + 5)
	var line := ""
	for y in range(lo, hi):
		line += "%d:%d " % [y, counts[y - y0]]
	_log("    " + line)

func _measure_generate_segment(tag: String, app: Node, accent: Color) -> void:
	var buttons := _buttons_with_text(app._phone_tool_sheet, "PIPELINE")
	if buttons.is_empty():
		_log("%s: no PIPELINE button found under _phone_tool_sheet" % tag)
		return
	var btn: Control = buttons[0]
	var vs_scroll := _clip_fraction(btn, app._phone_gen_scroll)
	var vs_sheet := _clip_fraction(btn, app._phone_tool_sheet)
	var bar_rect: Rect2 = (app._phone_menu_bar as Control).get_global_rect()
	_log("%s: sheet=%s  gen_scroll=%s" % [tag,
		_rect_str((app._phone_tool_sheet as Control).get_global_rect()),
		_rect_str((app._phone_gen_scroll as Control).get_global_rect())])
	_log("%s: PIPELINE button global_rect=%s" % [tag, _rect_str(vs_scroll["button"])])
	_log("%s: visible-vs-scroll frac=%.3f  visible-vs-sheet frac=%.3f  bar_top=%.0f" %
		[tag, vs_scroll["frac"], vs_sheet["frac"], bar_rect.position.y])
	## The pixel truth, not just the rect. A generous window around the button
	## so a large clip (as reported on the device) is fully captured.
	var img := get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	var br: Rect2 = vs_scroll["button"]
	var cx := int(br.position.x + br.size.x * 0.5)
	var scan := _accent_rows(img, cx - 24, cx + 24,
		int(br.position.y) - 10, int(br.position.y) + int(br.size.y) + 40, accent)
	_print_rows(tag + " PIPELINE accent", scan)
	## Colour-agnostic: whichever ink the label is actually drawn in (accent
	## when selected, line/text_secondary when not), against the sheet's own
	## background sampled 6 px above the button (inside the sheet, no control
	## there -- the `gen_pad` margin).
	var bg := img.get_pixel(clampi(cx, 0, img.get_width() - 1),
		clampi(int(br.position.y) - 6, 0, img.get_height() - 1))
	var ink := _ink_rows(img, int(br.position.x), int(br.position.x + br.size.x),
		int(br.position.y) - 10, int(br.position.y) + int(br.size.y) + 40, bg)
	_log("%s: bg sample=%s" % [tag, bg])
	_print_rows(tag + " PIPELINE ink(full-width)", ink)

func _measure_dock_switch(tag: String, app: Node, accent: Color) -> void:
	var buttons := _buttons_with_text(app.left_dock, "PIPELINE")
	if buttons.is_empty():
		_log("%s: no PIPELINE button found under left_dock" % tag)
		return
	var btn: Control = buttons[0]
	var br := btn.get_global_rect()
	var dock_rect: Rect2 = (app.left_dock as Control).get_global_rect()
	var bar_rect: Rect2 = (app._phone_menu_bar as Control).get_global_rect()
	_log("%s: left_dock rect=%s" % [tag, _rect_str(dock_rect)])
	_log("%s: left_dock PIPELINE button rect=%s" % [tag, _rect_str(br)])
	_log("%s: bar rect=%s" % [tag, _rect_str(bar_rect)])
	_log("%s: dock.offset_bottom=%.1f  dock bottom=%.1f vs bar top=%.1f  gap(bar_top - dock_bottom)=%.1f" %
		[tag, (app.left_dock as Control).offset_bottom, dock_rect.end.y, bar_rect.position.y,
			bar_rect.position.y - dock_rect.end.y])
	## Whose paint wins in the overlap band, measured, not reasoned:
	## MISTAKES.md line 46, "flip the flag and diff the framebuffer" -- sample
	## the pixel colour at the dock/bar overlap centre and compare it against
	## each candidate's own panel colour.
	if dock_rect.end.y > bar_rect.position.y + 2.0:
		var img := get_viewport().get_texture().get_image()
		img.convert(Image.FORMAT_RGBA8)
		var sample_y := int((bar_rect.position.y + minf(dock_rect.end.y, bar_rect.end.y)) * 0.5)
		var sample_x := int(bar_rect.position.x + bar_rect.size.x * 0.5)
		var p := img.get_pixel(clampi(sample_x, 0, img.get_width() - 1),
			clampi(sample_y, 0, img.get_height() - 1))
		_log("%s: OVERLAP %.1f px -- sample at (%d,%d)=%s  dock.panel=%s  bar.panel=%s" %
			[tag, dock_rect.end.y - bar_rect.position.y, sample_x, sample_y, p,
				DccTheme.c("panel"), DccTheme.c("panel")])
	else:
		_log("%s: no overlap -- dock stops %.1f px above the bar's top" %
			[tag, bar_rect.position.y - dock_rect.end.y])

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 150.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2340)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	var screen: Vector2 = app.get_viewport_rect().size
	_log("phone=%s scale=%.4f screen=%s" % [app.is_phone(), app.phone_scale(), screen])
	if not app.is_phone():
		_log("ABORT not phone -- pass --force-touch")
		get_tree().quit(2)
		return

	var accent: Color = DccTheme.c("accent")
	var bar: Control = app._phone_menu_bar
	_log("accent=%s" % accent)
	_log("bar rect=%s  H_PHONE_BOTTOM_NAV=%.1fpx  safe_bottom=%dpx  nav_reserve=%.1fpx  phone_tab=%s  detent=%s" % [
		_rect_str(bar.get_global_rect()), app._ptap(DccTheme.H_PHONE_BOTTOM_NAV),
		app._safe_bottom(), app._phone_nav_reserve(), app._phone_tab, app.phone_detent()])

	# -- Scenario 1: boot state, GENERATE tab, nothing tapped (peek) ----------
	await _frames(2)
	_measure_generate_segment("S1 boot/peek", app, accent)

	# -- Scenario 2: one tap lifts peek -> half --------------------------------
	app._pick_phone_tab("gen")
	await get_tree().create_timer(0.4).timeout
	await _frames(3)
	_measure_generate_segment("S2 tap->half", app, accent)

	# -- Scenario 3: full detent ------------------------------------------------
	app.set_phone_detent("full")
	await get_tree().create_timer(0.4).timeout
	await _frames(3)
	_measure_generate_segment("S3 full", app, accent)

	app.set_phone_detent("peek")
	await get_tree().create_timer(0.4).timeout
	await _frames(3)

	# -- Scenario 4: LEFT DOCK sheet opened (MORE (' Window (' Left dock) -----
	## Z-order, measured rather than reasoned (MISTAKES.md line 46: "flip the
	## flag and diff the framebuffer"). Sample the same pixel, at the bottom
	## bar's own "GENERATE" caption centre, before and after the dock opens --
	## if the dock (added to the tree after `chrome`) paints on top, the
	## caption's ink disappears; if the bar wins, it does not move.
	var gen_lbl := _find_label(bar, "GENERATE")
	var before_px := Color(0, 0, 0, 0)
	var sample_pt := Vector2.ZERO
	if gen_lbl != null:
		var lr := gen_lbl.get_global_rect()
		sample_pt = lr.get_center()
		var img0 := get_viewport().get_texture().get_image()
		img0.convert(Image.FORMAT_RGBA8)
		before_px = img0.get_pixel(clampi(int(sample_pt.x), 0, img0.get_width() - 1),
			clampi(int(sample_pt.y), 0, img0.get_height() - 1))
	else:
		_log("S4: could not find a 'GENERATE' Label under the bar -- skipping z-order sample")

	app._set_sheet_open("left", true)
	await _frames(8)
	_measure_dock_switch("S4 left-dock-open", app, accent)

	if gen_lbl != null:
		var img1 := get_viewport().get_texture().get_image()
		img1.convert(Image.FORMAT_RGBA8)
		var after_px := img1.get_pixel(clampi(int(sample_pt.x), 0, img1.get_width() - 1),
			clampi(int(sample_pt.y), 0, img1.get_height() - 1))
		var moved := absf(before_px.r - after_px.r) + absf(before_px.g - after_px.g) \
			+ absf(before_px.b - after_px.b) > 0.05
		_log("S4 z-order @ GENERATE caption %s: before=%s after=%s changed=%s (%s wins the overlap)" %
			[sample_pt, before_px, after_px, moved,
				("left_dock" if moved else "bottom bar")])

	_log("=== done ===")
	get_tree().quit(0)
