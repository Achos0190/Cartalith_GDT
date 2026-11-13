extends Node
## Phone fixes of 2026-11-13 (owner, on the OnePlus 6T: "The drawer for map on mobile is empty and the button to select
## generate / carto / civil misses the indication balls. Also selection switches are ridiculously small.").
## Boots the real shell as a phone in a `--vp`-sized host `SubViewport` and drives REAL pointer events (a touch and its
## emulated mouse together, as the device delivers them). Three groups of checks, every expected figure a literal
## written by hand (nothing is asserted against the constant it came from):
##   1. THE MAP SHEET: tapping MAP lifts the sheet to half and its body holds the view row (ALL LAYERS, FIT WORLD,
##      2D / 3D), the view field (dropdown and opacity), the six Measure modes and ten visible-layer switches; every
##      button is at the 44 phone-unit floor; a real tap on a switch flips the host's layer and a second puts it back; a
##      real tap on a Measure mode arms the tool; FIT WORLD resets the zoom; the body is gone off the MAP tab.
##   2. THE DOMAIN DOTS: the domain tab carries three dots above its glyph, wholly inside the cell, at least 4 phone units
##      across, exactly one in the active colour and it is the active domain's (WORLD 0, CIVIL 1, CARTO 2), moving with a
##      real swipe, and never taking a tap (a tap on a dot opens the domain sheet, as a tap on the cell does).
##   3. THE SWITCH CENSUS: every CheckBox and CheckButton laid out on WORLD's, CIVIL's, CARTO's and MAP's sheets (every
##      category opened) is counted; each one's tap box is at least 44 dp of REAL density and its glyph (the theme icon's
##      size, times its window's content scale) at least 26 phone units tall and 48 wide. Prints the count and the
##      smallest glyph, so a before/after reads off one line.
## Windowed, with a wiped user dir:
##   godot --path . --resolution 1600x1000 _p13fix_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch [--shots DIR]
##   (also 2340x1080 @2.8125, 1080x1920 @3 = 360 dp, 1920x1080 @3 for its landscape)
## Arguments read by THIS file: `--vp WxH`, `--shots DIR`. Accepted because the shell reads them: `--density D`,
## `--force-touch`. Anything else aborts with exit 2.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0
var _shots := ""
var _tag := ""

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density", "--shots"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	_dens = float(_arg("--density", "1"))
	_shots = _arg("--shots", "")
	_tag = "%sx%s" % [parts[0], parts[1]]
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# ---- pointer helpers -----------------------------------------------------------------------------------------

func _press(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_vp.push_input(t)
	_vp.push_input(m)
	await _frames(2)

func _move(p: Vector2, from: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - from
	_vp.push_input(d)
	_vp.push_input(m)
	await _frames(2)

func _release(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_vp.push_input(t)
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2) -> void:
	await _press(p)
	await _release(p)

func _swipe(p: Vector2, to: Vector2) -> void:
	await _press(p)
	var cur := p
	for s in 3:
		var nxt := p.lerp(to, float(s + 1) / 3.0)
		await _move(nxt, cur)
		cur = nxt
	await _release(to)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	_vp.get_texture().get_image().save_png("%s/%s_%s.png" % [_shots, _tag, name])

# ---- the checks ----------------------------------------------------------------------------------------------

## Dots: three, inside the cell, at least 4 units across, exactly one in the lit colour, and it is index `want`.
func _check_dots(want: int, label: String) -> void:
	var cell: Dictionary = _app._phone_tab_cells.get("domain", {})
	var dots: Array = cell.get("dots", [])
	_ok(dots.size() == 3, "%s: the domain tab draws three dots (got %d)" % [label, dots.size()])
	if dots.size() != 3:
		return
	var cr: Rect2 = _cell("domain").get_global_rect()
	var u: float = float(_app.phone_scale())
	var cols: Array = []
	for k in 3:
		var d := dots[k] as Panel
		var r: Rect2 = d.get_global_rect()
		_ok(d.is_visible_in_tree() and cr.encloses(r), "%s: dot %d is visible and inside the cell (%s in %s)" % [label, k, str(r), str(cr)])
		_ok(r.size.x >= 4.0 * u - 1.0 and r.size.y >= 4.0 * u - 1.0, "%s: dot %d is at least 4 units across (%.1f x %.1f px, unit %.2f)" % [label, k, r.size.x, r.size.y, u])
		var sb := d.get_theme_stylebox("panel") as StyleBoxFlat
		cols.append(sb.bg_color if sb != null else Color(0, 0, 0, 0))
	var lit: Color = cols[want]
	var same_as_lit := 0
	for k in 3:
		if k != want and (cols[k] as Color).is_equal_approx(lit):
			same_as_lit += 1
	_ok(same_as_lit == 0, "%s: the dots other than %d differ in colour from it (%s)" % [label, want, str(cols)])
	_ok((cols[(want + 1) % 3] as Color).is_equal_approx(cols[(want + 2) % 3]), "%s: the two idle dots share one colour" % label)
	_ok(lit.a > 0.9, "%s: the lit dot is opaque (alpha %.2f)" % [label, lit.a])

## The real-density 44 dp tap box and the glyph of every switch laid out under the shell, counted.
func _census(label: String) -> Dictionary:
	var sw: Array = []
	_collect(_app, sw)
	var small_box := 0
	var small_glyph := 0
	var min_h := 99999.0
	var min_w := 99999.0
	var min_box := 99999.0
	var u: float = float(_app.phone_scale())
	for n in sw:
		var cb := n as BaseButton
		var scale := _window_scale(cb)
		var box := cb.get_global_rect().size * scale
		min_box = minf(min_box, minf(box.x, box.y))
		if box.x < 44.0 * _dens - 1.0 or box.y < 44.0 * _dens - 1.0:
			small_box += 1
			print("  small box: ", cb.get_path(), " ", box, " scale ", scale, " meta ", cb.get_meta("dcc_switch_unit", "none"), " fitted ", cb.has_meta("_phone_fitted"), " min ", cb.custom_minimum_size, " win ", cb.get_window().get_class(), " vis ", cb.get_window().visible, " csf ", cb.get_window().content_scale_factor, " treewin==root ", cb.get_window() == get_tree().root)
		var tex: Texture2D = cb.get_theme_icon("checked")
		var g := Vector2(tex.get_size()) * scale if tex != null else Vector2.ZERO
		min_h = minf(min_h, g.y)
		min_w = minf(min_w, g.x)
		## A painted switch is a pill (48 units wide); a bare check or radio glyph is square, so only its height counts.
		var pill := cb.has_meta("dcc_palette_dark") or cb.get_theme_icon("checked").get_width() > cb.get_theme_icon("checked").get_height()
		if g.y < 26.0 * u - 1.0 or (pill and g.x < 48.0 * u - 1.0):
			small_glyph += 1
			print("  small glyph: ", cb.get_path(), " ", g)
	print("CENSUS %s: %d switches, smallest box %.1f px (%.1f dp), smallest glyph %.1f x %.1f px (%.1f x %.1f dp)" % [
		label, sw.size(), min_box, min_box / _dens, min_w, min_h, min_w / _dens, min_h / _dens])
	return {"n": sw.size(), "small_box": small_box, "small_glyph": small_glyph}

func _collect(n: Node, out: Array) -> void:
	if (n is CheckBox or n is CheckButton) and (n as Control).is_visible_in_tree() \
			and (n as Control).get_global_rect().size.x > 1.0:
		out.append(n)
	for c in n.get_children():
		_collect(c, out)

## The content scale of the Window a control sits in, 1.0 when it is drawn straight onto the shell's viewport.
func _window_scale(c: Control) -> float:
	var p: Node = c
	while p != null:
		if p is Window and p != get_tree().root:
			return float((p as Window).content_scale_factor)
		p = p.get_parent()
	return 1.0

func _open_all(ws: Node) -> void:
	for e in ws.categories:
		var body: Control = (e as Dictionary).get("body") as Control
		if body != null:
			body.visible = true

func _buttons(n: Node, out: Array) -> void:
	if n is Button and (n as Control).is_visible_in_tree():
		out.append(n)
	for c in n.get_children():
		_buttons(c, out)

func _find_class(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_class(c, cls, out)

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
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
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	if not DccTheme.is_phone():
		return
	var cell_c := _centre("domain")
	var dx: float = 48.0 * float(_app.phone_scale()) + 24.0
	var swipe_vec := Vector2(-dx, 0)

	# ---- 2. the dots, on WORLD and after real swipes -----------------------------------------------------------
	_ok(_app._active_domain == "world", "boots on WORLD")
	_check_dots(0, "WORLD")
	_shot("domain_world")
	# a tap on a dot is a tap on the cell: it opens the domain sheet and changes nothing else
	if _app._phone_detent != "peek":
		await _tap(_centre("domain"))
		await _settle()
	var dots_a: Array = (_app._phone_tab_cells["domain"] as Dictionary).get("dots", [])
	var dot1: Control = (dots_a[1] as Control) if dots_a.size() == 3 else _cell("domain")
	await _tap(dot1.get_global_rect().get_center())
	await _settle()
	_ok(_app._phone_detent != "peek" and _app._phone_tab == "domain" and _app._active_domain == "world",
		"a tap on a dot behaves as a tap on the tab (detent %s, tab %s, domain %s)" % [_app._phone_detent, _app._phone_tab, _app._active_domain])
	_open_all(_app._workspace_panels.get("world"))
	await _settle()
	var cw := _census("WORLD sheet")
	_ok(int(cw["n"]) > 0, "WORLD sheet: the census found switches (%d)" % int(cw["n"]))
	_ok(int(cw["small_box"]) == 0, "WORLD sheet: no switch has a tap box under 44 dp (%d do)" % int(cw["small_box"]))
	_ok(int(cw["small_glyph"]) == 0, "WORLD sheet: no switch glyph is under 26 x 48 units (%d are)" % int(cw["small_glyph"]))

	var doms := ["civilization", "cartography"]
	for k in doms.size():
		await _swipe(cell_c, cell_c + swipe_vec)
		await _settle()
		_ok(_app._active_domain == doms[k], "swipe selects %s (got %s)" % [doms[k], _app._active_domain])
		_check_dots(k + 1, doms[k])
		if _app._phone_detent == "peek":
			await _tap(_centre("domain"))
			await _settle()
		_open_all(_app._workspace_panels.get(doms[k]))
		await _settle()
		_shot("domain_" + doms[k])
		var cd := _census(doms[k] + " sheet")
		_ok(int(cd["n"]) > 0, "%s sheet: the census found switches (%d)" % [doms[k], int(cd["n"])])
		_ok(int(cd["small_box"]) == 0, "%s sheet: no switch has a tap box under 44 dp (%d do)" % [doms[k], int(cd["small_box"])])
		_ok(int(cd["small_glyph"]) == 0, "%s sheet: no switch glyph is under 26 x 48 units (%d are)" % [doms[k], int(cd["small_glyph"])])
	await _swipe(cell_c, cell_c + swipe_vec)
	await _settle()
	_ok(_app._active_domain == "world", "the third swipe wraps to WORLD")
	_check_dots(0, "WORLD again")

	# ---- 1. the MAP sheet --------------------------------------------------------------------------------------
	await _tap(_centre("map"))
	await _settle()
	if _app._phone_detent == "peek":
		await _tap(_centre("map"))
		await _settle()
	_ok(_app._phone_tab == "map" and _app._phone_detent != "peek", "MAP: the sheet is up (tab %s, detent %s)" % [_app._phone_tab, _app._phone_detent])
	var body: ScrollContainer = _app.get("_phone_map_body")
	_ok(body != null and body.is_visible_in_tree(), "MAP: its body is visible")
	_shot("map")
	if body == null:
		_census("MAP sheet (no body: the shipped tool row only)")
		return
	_shot("map")
	var br: Rect2 = body.get_global_rect()
	_ok(br.size.y >= 100.0 * float(_app.phone_scale()) and br.end.y <= vh + 0.5, "MAP: the body is at least 100 units tall and on screen (%s)" % str(br))
	var row: Control = _app._phone_map_row
	var texts: Array = []
	if row != null:
		for c in row.get_children():
			texts.append((c as Button).text.to_upper())
	_ok(texts == ["ALL LAYERS", "FIT WORLD", "2D / 3D"], "MAP: the view row reads %s" % str(texts))
	var fl: float = 44.0 * float(_app.phone_scale()) - 1.0
	var buttons: Array = []
	_buttons(body, buttons)
	var names: Array = []
	for b in buttons:
		names.append((b as Button).text.to_upper())
	for want in ["FIT WORLD", "2D / 3D", "DISTANCE", "BEARING", "AREA", "RADIUS", "CROSS-SECTION", "Δ VERTICAL"]:
		_ok(names.has(want), "MAP: a '%s' button is in the body" % want)
	var shorter := 0
	for b in buttons:
		if (b as Button).get_global_rect().size.y < fl and not (b is CheckBox):
			shorter += 1
	_ok(shorter == 0, "MAP: every button in the body is at the 44 unit floor (%d under it, of %d)" % [shorter, buttons.size()])
	var toggles: Dictionary = _app._phone_map_toggles
	_ok(toggles.size() == 11, "MAP: ten visible-layer switches plus the chunk-grid switch (got %d: %s)" % [toggles.size(), str(toggles.keys())])
	_ok(toggles.has("lod_grid"), "MAP: the chunk-grid switch is there (the desktop has no North up or scale-bar control to mirror)")
	for id in ["settlements", "roads", "landmarks", "rivers", "provinces", "territory"]:
		_ok(toggles.has(id), "MAP: a switch for '%s'" % id)
	var opts: Array = []
	_find_class(body, "OptionButton", opts)
	_ok(opts.size() == 1 and (opts[0] as OptionButton).item_count >= 2, "MAP: one view-field dropdown with the base map and fields (%d items)" % [(opts[0] as OptionButton).item_count if opts.size() == 1 else -1])
	var sl: Array = []
	_find_class(body, "HSlider", sl)
	## Visible ones only: the "3D height" slider is built beside it but shown only while 3D is (`_p3dheight_probe.gd`).
	sl = sl.filter(func(n): return (n as Control).is_visible_in_tree())
	_ok(sl.size() == 1, "MAP: one opacity slider (%d)" % sl.size())
	# a real tap on a switch flips the host's layer and a second puts it back
	var vp: Node = _app.viewport
	var tcb := toggles.get("settlements") as CheckBox
	if tcb != null:
		body.ensure_control_visible(tcb)
		await _frames(3)
		var was: bool = vp.layer_visible("settlements")
		await _tap(tcb.get_global_rect().get_center())
		await _settle()
		_ok(vp.layer_visible("settlements") == (not was), "MAP: a tap on the Settlements switch flips the layer (was %s)" % str(was))
		_ok(tcb.button_pressed == (not was), "MAP: the switch shows the new state")
		await _tap(tcb.get_global_rect().get_center())
		await _settle()
		_ok(vp.layer_visible("settlements") == was, "MAP: a second tap puts it back")
	# a real tap on the chunk-grid switch writes the same state Preferences > Tiles & LOD > Chunk debug overlay > Grid reads
	var gcb := toggles.get("lod_grid") as CheckBox
	if gcb != null:
		body.ensure_control_visible(gcb)
		await _frames(3)
		var gwas: bool = vp.lod_debug_enabled("grid")
		await _tap(gcb.get_global_rect().get_center())
		await _settle()
		_ok(vp.lod_debug_enabled("grid") == (not gwas), "MAP: a tap on Chunk grid flips the host's lod-debug grid (was %s)" % str(gwas))
		await _tap(gcb.get_global_rect().get_center())
		await _settle()
		_ok(vp.lod_debug_enabled("grid") == gwas, "MAP: a second tap puts it back")
	# a real tap on a Measure mode (not the armed default) arms the Measure tool
	for b in buttons:
		if (b as Button).text.to_upper() == "AREA":
			body.ensure_control_visible(b as Control)
			await _frames(3)
			await _tap((b as Control).get_global_rect().get_center())
			await _settle()
			_ok(GlobalTools._measure_mode == "area", "MAP: a tap on Area selects the Area mode (got %s)" % GlobalTools._measure_mode)
			break
	# the census, then Fit world (the sheet may have dropped to peek when the tool armed)
	if _app._phone_detent == "peek" or not _app._phone_map_body.visible:
		await _tap(_centre("map"))
		await _settle()
	var cm := _census("MAP sheet")
	_ok(int(cm["n"]) >= 3, "MAP sheet: its switches are counted (%d)" % int(cm["n"]))
	_ok(int(cm["small_box"]) == 0, "MAP sheet: no switch has a tap box under 44 dp (%d do)" % int(cm["small_box"]))
	_ok(int(cm["small_glyph"]) == 0, "MAP sheet: no switch glyph is under 26 x 48 units (%d are)" % int(cm["small_glyph"]))
	if _app._phone_map_row != null and _app._phone_map_body.visible:
		_app._phone_map_body.scroll_vertical = 0   ## the Area tap scrolled the row away
		await _frames(3)
		var before: float = vp._zoom
		vp._zoom = before * 2.0
		for c in _app._phone_map_row.get_children():
			if (c as Button).text.to_upper() == "FIT WORLD":
				await _tap((c as Control).get_global_rect().get_center())
				await _settle()
		_ok(is_equal_approx(vp._zoom, before), "MAP: Fit world resets the view (zoom %.3f, was %.3f)" % [vp._zoom, before])
	# the body leaves with the MAP tab
	await _tap(_centre("domain"))
	await _settle()
	_ok(not _app._phone_map_body.visible, "the MAP body is hidden off the MAP tab")

	await _window_census()

## Census of switches inside the windows the probe opens (owner leftover, 2026-11-13): each is counted and must be at
## least 44 dp of real density in its box and carry the boosted glyph. Closes each window after counting.
func _window_census() -> void:
	var total := 0
	var opens := {
		"Assistant settings": func(): _app.open_assistant_settings(),
		"Quick bar": func(): _app.open_quick_bar_window(),
		"New World": func(): _app.open_new_world(),
		"Export": func(): _app.open_export_dialog(),
		"Travel library": func(): _app.open_travel_library(),
		"Journey planner": func(): _app.open_journey_planner(),
	}
	for k in opens.keys():
		(opens[k] as Callable).call()
		await get_tree().create_timer(0.6).timeout
		await _frames(4)
		var c := _census("window " + String(k))
		total += int(c["n"])
		_ok(int(c["small_box"]) == 0, "window %s: no switch box under 44 dp (%d do)" % [k, int(c["small_box"])])
		_ok(int(c["small_glyph"]) == 0, "window %s: no switch glyph under 26 x 48 units (%d are)" % [k, int(c["small_glyph"])])
		for w in _app.get_children():
			if w is Window and (w as Window).visible:
				(w as Window).hide()
		await _frames(3)
	_ok(total >= 150, "the windows' switch census found switches (%d in all)" % total)
