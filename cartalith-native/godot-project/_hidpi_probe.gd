extends Node
## The OnePlus 12 pass (`GUI_GAP_REGISTER.md` HD-01..HD-03). Drives the real
## shell at 1440x3168 -- `DccShell._phone_scale` 3.664, where every prior phone
## measurement in this repository was taken at 2.75 -- and measures the pixels
## rather than describing them.
##
##   godot --path . --resolution 1440x3168 _hidpi_probe.tscn -- --force-touch
##   godot --path . --resolution 1080x2400 _hidpi_probe.tscn -- --force-touch
##
## Three things are measured, all on the framebuffer:
##   [FONT]  max |dLum| between horizontally adjacent pixels inside a content-
##           scaled sub-Window. A natively-rasterised glyph steps ground->ink in
##           one pixel (~0.98); a bitmap resampled by the canvas transform caps
##           at about 1/factor (0.27 at 3.664, 0.36 at 2.75).
##   [ICON]  each `DccIcons` texture's real texel count against the size it is
##           drawn at. A ratio below 1 is a magnified bitmap.
##   [TAP]   the viewport chrome's hit boxes in millimetres.

## Hosted in a `SubViewport`, not in the real window: Windows clamps a window to
## the desktop work area, so `--resolution 1440x3168` came back as 1440x1031 and
## `DccShell` classified the result as a tablet (`phone_scale` 2.62, `_phone`
## false). A `SubViewport` has no such ceiling, `get_viewport_rect()` inside it
## reports its own size, and `gui_embed_subwindows` keeps the shell's dialogs
## rendering into the same texture so the framebuffer this measures is the whole
## composition. Pass `--vp WxH`.
var app: Node
var _vp: SubViewport
var _fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)
	return out

func _check(label: String, got, want) -> void:
	var ok: bool = (got == want)
	if not ok:
		_fails += 1
	print(("  ok   " if ok else "  FAIL ") + label + "  got=" + str(got) + " want=" + str(want))

## Max / p99 adjacent-pixel luminance delta and the count of hard edges, over a
## rect of the framebuffer. Ink pixels only matter as a sanity floor: a crop
## with no text in it would otherwise report a flattering 0.
func _edges(img: Image, rect: Rect2i, tag: String) -> Dictionary:
	var deltas: Array[float] = []
	var hard := 0
	var ink := 0
	var x0 := maxi(0, rect.position.x)
	var y0 := maxi(0, rect.position.y)
	var x1 := mini(img.get_width(), rect.position.x + rect.size.x)
	var y1 := mini(img.get_height(), rect.position.y + rect.size.y)
	for y in range(y0, y1):
		var prev := -1.0
		for x in range(x0, x1):
			var c := img.get_pixel(x, y)
			var lum := c.r * 0.299 + c.g * 0.587 + c.b * 0.114
			if lum > 0.45:
				ink += 1
			if prev >= 0.0:
				var d: float = absf(lum - prev)
				if d > 0.02:
					deltas.append(d)
				if d > 0.5:
					hard += 1
			prev = lum
	deltas.sort()
	var mx: float = deltas[deltas.size() - 1] if deltas.size() > 0 else 0.0
	var p99: float = deltas[int(deltas.size() * 0.99)] if deltas.size() > 0 else 0.0
	print("[FONT] %-22s max=%.4f p99=%.4f hard(>0.5)=%d brightpx=%d samples=%d rect=%s"
		% [tag, mx, p99, hard, ink, deltas.size(), str(rect)])
	return {"max": mx, "p99": p99, "hard": hard, "ink": ink}

func _grab() -> Image:
	return _vp.get_texture().get_image()

func _shot(name: String) -> void:
	await _frames(3)
	var out := "user://hidpi_%s_%d.png" % [name, int(_vp.size.x)]
	_grab().save_png(out)
	print("shot ", ProjectSettings.globalize_path(out))

func _ready() -> void:
	var w := 1440
	var h := 3168
	var args := OS.get_cmdline_user_args()
	var i := args.find("--vp")
	if i >= 0 and i + 1 < args.size():
		var parts: PackedStringArray = String(args[i + 1]).split("x")
		w = int(parts[0])
		h = int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.2).timeout

	var vp: Vector2 = app.get_viewport_rect().size
	var s: float = app.phone_scale()
	print("=== viewport=", vp, " phone=", app.is_phone(), " phone_scale=", s,
		" root.content_scale_factor=", get_tree().root.content_scale_factor,
		" root.get_oversampling()=", get_tree().root.get_oversampling(), " ===")

	# ── The welcome window: a content-scaled sub-Window, and the first screen ──
	var dlg: Window = app.open_project_dialog
	print("[WIN] visible=", dlg.visible, " size=", dlg.size,
		" content_scale_factor=", dlg.content_scale_factor,
		" oversampling=", dlg.oversampling,
		" oversampling_override=", dlg.oversampling_override,
		" get_oversampling()=", dlg.get_oversampling())
	print("[FILL] dlg.size=", dlg.size, " vs viewport=", vp,
		"  fill=%.1f%%" % (100.0 * dlg.size.y / vp.y))
	print("[FILL] is_embedded=", dlg.is_embedded(),
		" root.gui_embed_subwindows=", get_tree().root.gui_embed_subwindows,
		" subvp.gui_embed_subwindows=", _vp.gui_embed_subwindows,
		" setting=", ProjectSettings.get_setting("gui/common/embed_subwindows", "<unset>"))
	print("[FILL] screen_usable=", DisplayServer.screen_get_usable_rect(
		DisplayServer.window_get_current_screen()),
		" DS has SUBWINDOWS=", DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS))
	## Control: ask for the full height directly and see whether it sticks.
	dlg.size = Vector2i(int(vp.x), int(vp.y))
	await _frames(3)
	print("[FILL] after explicit size assign -> ", dlg.size)
	var body: Control = null
	for c in dlg.get_children():
		if c is Control:
			body = c as Control
			break
	if body != null:
		print("[FILL] body ", body.get_class(), " rect=", body.get_rect(),
			" min=", body.get_combined_minimum_size())
	await _frames(4)
	var img := _grab()
	## The upper third of the window, which is title + chips + search well --
	## all text, no map, no gradient.
	_edges(img, Rect2i(Vector2i(int(vp.x * 0.05), int(vp.y * 0.06)),
		Vector2i(int(vp.x * 0.9), int(vp.y * 0.14))), "welcome titles")
	await _shot("welcome")

	## Diagnostic: is the override being cleared by something after
	## `phone_present()` sets it, or was it never set? Set it here, from
	## outside, and see whether it sticks and whether the pixels move.
	dlg.oversampling_override = dlg.content_scale_factor
	await _frames(6)
	print("[WIN] after external set: override=", dlg.oversampling_override,
		" get_oversampling()=", dlg.get_oversampling())
	_edges(_grab(), Rect2i(Vector2i(int(vp.x * 0.05), int(vp.y * 0.06)),
		Vector2i(int(vp.x * 0.9), int(vp.y * 0.14))), "welcome EXTERNAL")
	await _shot("welcome_external")

	## Every `DccIcons` glyph on screen: texel count vs drawn size, in the
	## window's own space and then in physical pixels.
	var rects: Array = []
	_find_all(dlg, func(n): return n is TextureRect and (n as TextureRect).texture is ImageTexture, rects)
	for r in rects:
		var tr := r as TextureRect
		var it := tr.texture as ImageTexture
		var texels: Vector2i = it.get_image().get_size() if it.get_image() != null else Vector2i.ZERO
		var drawn_phys: Vector2 = tr.size * dlg.content_scale_factor
		print("[ICON] window rect=%s drawn_phys=%.1fpx texels=%s ratio=%.2f stretch=%d"
			% [str(tr.size), drawn_phys.x, str(texels),
				(float(texels.x) / maxf(1.0, drawn_phys.x)), tr.stretch_mode])
		print("[XF]     gtwc=", tr.get_global_transform_with_canvas().get_scale(),
			" screen=", tr.get_screen_transform().get_scale(),
			" final=", tr.get_viewport().get_final_transform().get_scale(),
			" gt=", tr.get_global_transform().get_scale(),
			" meta=", tr.get_meta("dcc_icon_magnify", "<none>"))
	## An `OptionButton`'s list is its own `Window`. Does it inherit the parent
	## window's content scale, and can it be given its own oversampling?
	var obs: Array = []
	_find_all(dlg, func(n): return n is OptionButton, obs)
	for o in obs.slice(0, 2):
		var pop: PopupMenu = (o as OptionButton).get_popup()
		print("[POP] OptionButton popup content_scale_factor=", pop.content_scale_factor,
			" get_oversampling()=", pop.get_oversampling(),
			" embedded=", pop.is_embedded(), " fs=", pop.get_theme_font_size("font_size"))
	dlg.hide()
	await _frames(3)

	# ── The viewport chrome, which lives in the UNSCALED main viewport ────────
	var host = app.viewport
	var ppmm: float = 510.0 / 25.4 if vp.x >= 1400.0 else 395.0 / 25.4
	for pair in [["layers button", host.get("_layers_btn")], ["navpad", host.get("_navpad")]]:
		var c = pair[1]
		if c != null and c is Control:
			var sz: Vector2 = (c as Control).size
			print("[TAP] %-14s size=%s -> %.2f x %.2f mm at %.0f ppi"
				% [pair[0], str(sz), sz.x / ppmm, sz.y / ppmm, ppmm * 25.4])
	var btns: Array = []
	if host.get("_navpad") != null:
		_find_all(host.get("_navpad"), func(n): return n is Button, btns)
	for b in btns:
		var bc := b as Button
		var it2 = bc.icon
		print("[TAP] navpad button size=%s icon_texels=%s"
			% [str(bc.size), str(it2.get_image().get_size() if it2 != null and it2 is ImageTexture and it2.get_image() != null else Vector2i.ZERO)])

	# ── A dock sheet: icons here are NOT content-scaled but ARE phone_fit ─────
	app._set_sheet_open("left", true)
	await get_tree().create_timer(0.7).timeout
	var tools: Array = []
	_find_all(app.left_dock, func(n): return n is Button and n.has_meta("dcc_tool_glyph"), tools)
	print("[ICON] dock tool buttons=", tools.size())
	for t in tools.slice(0, 4):
		var b2 := t as Button
		var ic = b2.icon
		var tex: Vector2i = (ic.get_image().get_size() if ic != null and ic is ImageTexture and ic.get_image() != null else Vector2i.ZERO)
		print("[ICON]   '%s' box=%s icon_reported=%s icon_texels=%s"
			% [b2.text, str(b2.size), str(ic.get_size() if ic != null else Vector2.ZERO), str(tex)])
	await _shot("leftsheet")
	app._set_sheet_open("left", false)
	await _frames(3)

	# ── PH-05 re-run: does a touch flick down the left sheet scroll? ─────────
	app._set_sheet_open("left", true)
	await get_tree().create_timer(0.7).timeout
	var sc := _find_scroll(app.left_dock)
	if sc == null:
		print("[SCROLL] no ScrollContainer under left_dock")
	else:
		var r := sc.get_global_rect()
		print("[SCROLL] rect=", r, " max=", sc.get_v_scroll_bar().max_value,
			" page=", sc.get_v_scroll_bar().page, " deadzone=", sc.scroll_deadzone,
			" touchscreen=", DisplayServer.is_touchscreen_available())
		var y: float = r.position.y + 40.0
		var moved := 0
		var tried := 0
		while y < r.end.y - 40.0:
			tried += 1
			if await _flick(sc, Vector2(r.get_center().x, y)) > 0:
				moved += 1
			y += 90.0 * s
		print("[SCROLL] points that scrolled: %d of %d" % [moved, tried])
		if moved == 0:
			_fails += 1
	app._set_sheet_open("left", false)
	await _frames(3)

	# ── GUI_GAP_REGISTER.md §46: a Popup is a Window, not a Control ──────────
	## **This block used to call `app._set_drawer_open(true)` and read
	## `app._phone_drawer`, neither of which exists any more.** The ☰ side drawer
	## was removed by the 412 dp phone migration in favour of the domain drill
	## (`dcc_shell.gd::_build_phone_menu_bar()` / `_phone_menu: PhoneMenu`); the
	## only trace left of the old names is a stale comment at
	## `dcc_shell.gd` (a doc comment naming `_pick_drawer_domain()` /
	## `_set_drawer_open()`). So this probe died here with "Nonexistent function
	## '_set_drawer_open'", never reached `get_tree().quit()`, and hung until the
	## caller's timeout -- a probe that cannot report is a probe that cannot
	## fail, which is the same defect as the bare `quit()` fixed below.
	##
	## The check itself is still live and still worth making: §46 is that the
	## Layers popover is a `PopupPanel`, i.e. a `Window`, which no Control walk
	## reaches, so opening a phone overlay used to leave both on screen. The
	## overlay is now a dock sheet, and `_set_sheet_open()` routes through
	## `_close_all_phone_overlays()`, which is where §46's fix lives. Same
	## assertion, against the surface that replaced the one it named -- and
	## `_set_sheet_open("left", ...)` is already used seven lines above, so this
	## block was the only stale caller left in the file.
	app.layers_popover.open()
	await _frames(4)
	var pop_before: bool = app.layers_popover.visible
	app._set_sheet_open("left", true)
	await _frames(4)
	print("[POPUP] layers popover visible before=", pop_before,
		" after opening the left dock sheet=", app.layers_popover.visible,
		" sheet visible=", app.left_dock.visible)
	_check("the popover opened at all", pop_before, true)
	_check("a phone overlay closes the popover", app.layers_popover.visible, false)
	_check("and the overlay it opened is up", app.left_dock.visible, true)
	app._set_sheet_open("left", false)
	await _frames(3)

	print("=== failures=", _fails, " ===")
	## Was a bare `quit()`, i.e. exit 0 whatever `_fails` held -- the same
	## defect `_deadwire_probe.gd` had fixed above its own `_verdict()`. A
	## counted failure nobody can gate on is a probe that cannot fail.
	get_tree().quit(1 if _fails > 0 else 0)

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var f := _find_scroll(c)
		if f != null:
			return f
	return null

## `_scrolldrag_probe.gd`'s idiom, with the input pushed into the SubViewport
## rather than the root and the drag distance scaled -- a 96 px flick is a real
## gesture at 393 dp and a twitch at 1440.
func _flick(sc: ScrollContainer, at: Vector2) -> int:
	sc.scroll_vertical = 0
	await _frames(1)
	var step: float = -12.0 * maxf(1.0, app.phone_scale())
	var hover := InputEventMouseMotion.new()
	hover.position = at
	_vp.push_input(hover)
	await _frames(1)
	var over := _vp.gui_get_hovered_control()
	var who := "<none>"
	if over != null:
		var chain := over.get_class()
		var q: Node = over.get_parent()
		for k in 3:
			if q == null:
				break
			chain += "<" + q.get_class()
			q = q.get_parent()
		who = "%s filter=%d" % [chain, over.mouse_filter]
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = at
	_vp.push_input(down)
	var p := at
	for i in 10:
		p += Vector2(0, step)
		var mm := InputEventMouseMotion.new()
		mm.position = p
		mm.relative = Vector2(0, step)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = p
	_vp.push_input(up)
	await _frames(1)
	var got: int = int(sc.scroll_vertical)
	print("[SCROLL]   y=%5d scrolled=%4d under=%s" % [int(at.y), got, who])
	return got
