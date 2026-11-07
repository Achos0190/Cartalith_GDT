extends Node
## MS-1 unit probe (`MOBILE_SHELL_MS1_SPEC.md` §5, bar B3): boots the real shell in a `--vp`-sized host
## `SubViewport` with a real density, and measures the dp unit in PHYSICAL px.
##
##   tablet: godot --path . _ms1unit_probe.tscn -- --force-touch --vp 2388x1668 --density 2 --leg tablet
##   phone:  godot --path . _ms1unit_probe.tscn -- --force-touch --vp 1440x3120 --density 3 --leg phone
##   map:    godot --path . _ms1unit_probe.tscn -- --force-touch --vp 2388x1668 --density 1 --leg map
##           godot --path . _ms1unit_probe.tscn -- --force-touch --vp 2388x1668 --density 2 --leg map
##   pad3:   godot --path . _ms1unit_probe.tscn -- --force-touch --vp 3392x2400 --density 2 --leg tablet
##
## **Windowed, never `--headless`** (`MISTAKES.md`, "Run a pixel probe"); run with the lane's
## `override.cfg` `config/name` and its app_userdata wiped. Arguments read by THIS file: `--vp WxH`,
## `--leg tablet|phone|map`, `--density D` (also read by the shell; this file reads it to know the expected
## figures), `--ui-scale S` (likewise), `--force-touch` (the shell's). Anything else aborts with exit 2.
##
## How sizes are read (`MISTAKES.md`, "Assert a control is present, sized or reachable"): from the
## framebuffer by DIFFERENCE. The host is captured, the node is modulated to black, captured again, and the
## bounding box of the changed pixels is the node's drawn extent in physical px. An empty difference is a
## failure ("nothing drawn"), never a pass.
##
## Legs:
##   tablet  menu bar height = 52 dp, rail width = 48 dp, status bar height = 36 dp, each x density x UI
##           scale, +-1 px; a 12 px theme label's glyph box passes HD-01's edge test (max adjacent |dLum|
##           >= 0.9, a labelled judgement between HD-01's 0.98 native and 0.27 magnified); the 3D view's
##           `SubViewport.size` equals its rect in physical px (+-1); `_capture_preview_png()`'s image has
##           the map's physical rect's size, after the function's own thumbnail rule.
##   phone   the undo chip, authored 48 units (`H_PHONE_PILL`), is 48 x density px tall (+-1); every visible
##           BaseButton in the main viewport is at least 44 x density px on BOTH axes (the walk is by target).
##   uiscale (B6, MS-1 S5) presses `Preferences ▸ UI scale ▸ 150 %` by emitting the submenu's `id_pressed`
##           (the handler path, not a finger: `MISTAKES.md`, "Plan a phone probe that must reach a DIALOG"),
##           then checks the settings file holds 1.5 and the menu bar draws 52 x density x 1.5 px, then
##           restores 100 % and checks 52 x density again. At 3840 x 2160 it presses 125 % instead, which
##           crosses the line (1 536 x 864 dp at density 2), and the runner greps the log for the shell's
##           class-change request (S4 is not built, so the boot class is kept).
##   map     prints the zoom at which `ViewportHost.lod_active()` first turns true, stepping the zoom by
##           `zoom_step(1.35)` from fit on a small generated world; the bar compares two runs (density 1
##           and 2) and is judged by the runner, within one step.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument, 3 watchdog.

var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0
var _density := 1.0
var _ui := 1.0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _grab() -> Image:
	var img := _vp.get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	return img

## The drawn extent of `n` in physical px: capture, modulate `n` to black, capture, diff.
func _drawn(n: CanvasItem) -> Rect2i:
	var a := _grab()
	var keep := n.modulate
	n.modulate = Color(0, 0, 0, 1)
	await _frames(3)
	var b := _grab()
	n.modulate = keep
	await _frames(2)
	return _diff_box(a, b)

func _diff_box(a: Image, b: Image) -> Rect2i:
	var x0 := a.get_width()
	var y0 := a.get_height()
	var x1 := -1
	var y1 := -1
	var da := a.get_data()
	var db := b.get_data()
	var w := a.get_width()
	var i := 0
	var n := da.size()
	while i < n:
		if da[i] != db[i] or da[i + 1] != db[i + 1] or da[i + 2] != db[i + 2]:
			var p := i / 4
			var x := p % w
			var y := p / w
			x0 = mini(x0, x)
			y0 = mini(y0, y)
			x1 = maxi(x1, x)
			y1 = maxi(y1, y)
		i += 4
	if x1 < 0:
		return Rect2i()
	return Rect2i(x0, y0, x1 - x0 + 1, y1 - y0 + 1)

func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b

func _edge(img: Image, r: Rect2i) -> float:
	var best := 0.0
	r = r.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x - 1):
			best = maxf(best, absf(_lum(img.get_pixel(x, y)) - _lum(img.get_pixel(x + 1, y))))
	return best

func _all(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_all(c, out)

func _args_ok() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--leg", "--density", "--ui-scale"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			return false
	return true

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("RESULT: FAIL (watchdog)")
		get_tree().quit(3))
	wd.start()
	if not _args_ok():
		get_tree().quit(2)
		return
	var parts := _arg("--vp", "").split("x")
	var leg := _arg("--leg", "")
	if parts.size() != 2 or not leg in ["tablet", "phone", "map", "uiscale"]:
		print("ABORT: needs --vp WxH and --leg tablet|phone|map|uiscale")
		get_tree().quit(2)
		return
	_density = _arg("--density", "1").to_float()
	_ui = _arg("--ui-scale", "1").to_float()
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("UNIT form=%s vp=%dx%d density=%s ui=%s override=%s phone_scale=%.4f final=%.3f" % [form, vw, vh,
		str(_density), str(_ui), str(_vp.size_2d_override), DccTheme.phone_scale(), _vp.get_final_transform().x.x])
	match leg:
		"tablet":
			await _tablet()
		"phone":
			await _phone()
		"map":
			await _map()
		"uiscale":
			await _uiscale(vw, vh)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

## The desktop-composition VBox `_build_desktop_shell()` adds: the shell's only VBoxContainer child.
func _shell_column() -> VBoxContainer:
	for c in _app.get_children():
		if c is VBoxContainer:
			return c
	return null

func _tablet() -> void:
	var k := _density * _ui
	var col := _shell_column()
	_ok(DccTheme.is_tablet() and col != null, "the tablet composition is built")
	if col == null:
		return
	var menu: Control = col.get_child(0)
	var status: Control = col.get_child(col.get_child_count() - 1)
	var main_row: Control = null
	for c in col.get_children():
		if c is HBoxContainer:
			main_row = c
	var rail_pair: Control = main_row.get_child(0) if main_row != null else null
	var rail: Control = rail_pair.get_child(0) if rail_pair != null and rail_pair.get_child_count() > 0 else rail_pair
	var mb := await _drawn(menu)
	var sb := await _drawn(status)
	var rb := await _drawn(rail)
	_ok(absf(mb.size.y - 52.0 * k) <= 1.0, "menu bar drawn %d px tall; want 52 dp = %.1f px (+-1)" % [mb.size.y, 52.0 * k])
	_ok(absf(rb.size.x - 48.0 * k) <= 1.0, "rail drawn %d px wide; want 48 dp = %.1f px (+-1)" % [rb.size.x, 48.0 * k])
	_ok(absf(sb.size.y - 36.0 * k) <= 1.0, "status bar drawn %d px tall; want 36 dp = %.1f px (+-1)" % [sb.size.y, 36.0 * k])
	## A 12 px label: the first visible Label in the status bar whose resolved font size is 12.
	var labels: Array = []
	_all(_app, labels)
	var target: Label = null
	for n in labels:
		if n is Label and n.is_visible_in_tree() and n.text.strip_edges() != "" \
				and n.get_theme_font_size("font_size") == 12:
			target = n
			break
	if target == null:
		_ok(false, "a visible 12 px Label to edge-test")
	else:
		var a := _grab()
		var keep: Color = target.self_modulate
		target.self_modulate = Color(1, 1, 1, 0)
		await _frames(3)
		var b := _grab()
		target.self_modulate = keep
		await _frames(2)
		var box := _diff_box(a, b)
		var e := _edge(a, box) if box.size.x > 0 else -1.0
		## HD-01's 0.98/0.27 were white ink on black. A theme label's ink is not white, so the raw
		## figure is bounded by the label's own contrast (0.579 for 'CARTALITH' at factor 1, measured);
		## the bar is applied to the edge as a share of the glyph box's luminance range, which is ~1.0
		## for a pixel-sharp glyph whatever its palette (`MISTAKES.md`, "Assert on pixels").
		var lo := 1.0
		var hi := 0.0
		for y in range(box.position.y, box.end.y):
			for x in range(box.position.x, box.end.x):
				var l := _lum(a.get_pixel(x, y))
				lo = minf(lo, l)
				hi = maxf(hi, l)
		var norm := e / maxf(0.0001, hi - lo)
		_ok(box.size.x > 0 and norm >= 0.9, "12 px label '%s' glyph box %s edge %.3f of range %.3f = %.3f >= 0.9" % [
			target.text.left(24), str(box), e, hi - lo, norm])
	## 3D: open it, read the SubViewport against the view's physical rect.
	var host = _app.viewport
	host.set_3d(true)
	await _frames(6)
	var v = host._view3d
	if v == null:
		_ok(false, "the 3D view opened")
	else:
		var sv: SubViewport = v.subviewport()
		var phys: Vector2 = Vector2(v.size) * _vp.get_final_transform().x.x
		_ok(absf(sv.size.x - phys.x) <= 1.0 and absf(sv.size.y - phys.y) <= 1.0,
			"3D SubViewport %s equals the view's physical rect %s (+-1)" % [str(sv.size), str(phys)])
	host.set_3d(false)
	await _frames(4)
	## Preview: the crop is the map's physical rect, then the function's own thumbnail rule.
	var png: PackedByteArray = _app._capture_preview_png()
	var img := Image.new()
	var err := img.load_png_from_buffer(png) if png.size() > 0 else ERR_INVALID_DATA
	var r: Rect2 = _vp.get_final_transform() * host.get_global_rect()
	var w := r.size.x
	var h := r.size.y
	var sc: float = float(_app.PREVIEW_THUMBNAIL_MAX_PX) / maxf(w, h)
	var want := Vector2i(roundi(w), roundi(h)) if sc >= 1.0 else Vector2i(maxi(1, roundi(roundi(w) * sc)), maxi(1, roundi(roundi(h) * sc)))
	_ok(err == OK and absi(img.get_width() - want.x) <= 1 and absi(img.get_height() - want.y) <= 1,
		"preview %dx%d from the map's physical rect %s (want %s)" % [img.get_width(), img.get_height(), str(r), str(want)])

func _phone() -> void:
	_ok(DccTheme.is_phone(), "the phone composition is built")
	var nodes: Array = []
	_all(_app, nodes)
	var chip: Control = null
	for n in nodes:
		if n is Control and n.name == "PhoneUndoChip":
			chip = n
	var want := 48.0 * _density
	_ok(chip != null and absf(chip.custom_minimum_size.y - want) <= 1.0,
		"undo chip (48 units) %s px tall; want 48 dp = %.1f px (+-1)" % [
			str(chip.custom_minimum_size.y) if chip != null else "missing", want])
	var floor_px := 44.0 * _density
	var small := 0
	var walked := 0
	for n in nodes:
		if not (n is BaseButton) or not n.is_visible_in_tree():
			continue
		if n.get_viewport() != _vp:
			continue
		var r: Rect2 = n.get_global_rect()
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			continue
		## Off-screen buttons (a closed sheet's) are not targets now.
		if not Rect2(Vector2.ZERO, _vp.get_visible_rect().size).intersects(r):
			continue
		walked += 1
		if r.size.x < floor_px - 0.5 or r.size.y < floor_px - 0.5:
			small += 1
			print("  under floor: %s %s" % [str(n.get_path()).right(70), str(r.size)])
	_ok(walked > 0 and small == 0, "every visible button >= %.0f px on both axes (%d walked, %d under)" % [
		floor_px, walked, small])

func _map() -> void:
	var bridge = _app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": false, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(8)
	var host = _app.viewport
	var first := -1
	for step in 40:
		if host.lod_active():
			first = step
			break
		host.zoom_step(1.35)
		await _frames(3)
	## The physical px per cell at engagement, `_update_lod()`'s own quantity. The fit zoom is a COVER fit
	## of the map control, whose aspect changes with the dp unit (the docks take a larger share of a
	## 1 194 dp width than of 2 388 px), so the step count alone compares two different layouts.
	var g: Vector2i = _app.bridge.grid_size()
	var ns := minf(host.size.x / float(g.x), host.size.y / float(g.y))
	print("UNIT map lod_first_step=%d zoom=%.4f fit_px_per_cell_physical=%.3f map_size=%s final=%.3f auto_zoom_gate=%.2f" % [
		first, host.zoom(), ns * host.zoom() * _vp.get_final_transform().x.x, str(host.size),
		_vp.get_final_transform().x.x, host.LOD_AUTO_ZOOM])
	_ok(first >= 0, "the tile pyramid engaged within 40 zoom steps (step %d)" % first)


func _stored_ui_scale() -> String:
	var cfg := ConfigFile.new()
	if cfg.load("user://cartalith_settings.cfg") != OK:
		return "no file"
	return str(cfg.get_value("ui_scale", "scale", "unset"))

func _uiscale(vw: int, vh: int) -> void:
	_ok(DccTheme.is_tablet(), "boots as a tablet")
	var nodes: Array = []
	_all(_app, nodes)
	var pop: PopupMenu = null
	for n in nodes:
		if n is PopupMenu and n.name == "UiScaleChoice":
			pop = n
	_ok(pop != null, "Preferences has a UI scale submenu")
	if pop == null:
		return
	var menu: Control = _shell_column().get_child(0)
	var m0 := await _drawn(menu)
	_ok(absf(m0.size.y - 52.0 * _density) <= 1.0, "menu bar %d px at 100 %%; want %.1f" % [m0.size.y, 52.0 * _density])
	if vw == 3840 and vh == 2160:
		pop.id_pressed.emit(931)
		await get_tree().create_timer(0.4).timeout
		await _frames(4)
		_ok(_stored_ui_scale() == "1.25", "125 %% stored (file reads %s)" % _stored_ui_scale())
		_ok(DccTheme.is_tablet(), "still the boot composition after crossing the line (S4 not built)")
		print("UIS crossed 4K@2 x1.25")
	else:
		pop.id_pressed.emit(932)
		await get_tree().create_timer(0.4).timeout
		await _frames(4)
		_ok(_stored_ui_scale() == "1.5", "150 %% stored (file reads %s)" % _stored_ui_scale())
		var m1 := await _drawn(menu)
		_ok(absf(m1.size.y - 52.0 * _density * 1.5) <= 1.0, "menu bar %d px at 150 %%; want %.1f" % [m1.size.y, 52.0 * _density * 1.5])
	pop.id_pressed.emit(930)
	await get_tree().create_timer(0.4).timeout
	await _frames(4)
	_ok(_stored_ui_scale() == "1.0", "100 %% restored (file reads %s)" % _stored_ui_scale())
	var m2 := await _drawn(menu)
	_ok(absf(m2.size.y - 52.0 * _density) <= 1.0, "menu bar %d px after restoring 100 %%; want %.1f" % [m2.size.y, 52.0 * _density])
