extends Node
## Ruling FL item 1 (MS-3 as a tool palette only) probe. Boots the REAL shell as a phone in a `--vp WxH` host
## SubViewport, opens the domain sheet and checks the palette row at its top: the cells for each domain are the
## registry's (literal expectations written by hand), every cell is >= 44 dp, no second chip strip exists, arming from
## the sheet reaches `armed_tool`, the SCULPT-mode gate follows a real tap on the dock's SCULPT segment, and the row is
## freed when the Pins tool's palette replaces the sheet body.
##   godot --path . --resolution 1600x1000 _fl1_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch [--shot PATH]
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`; exit 0 / 1 / 2 (bad argument).
var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
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
		elif a in ["--vp", "--density", "--shot"]:
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
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
func _tap(p: Vector2) -> void:
	for pr in [true, false]:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = pr
		t.position = p
		_vp.push_input(t)
		var m := InputEventMouseButton.new()
		m.button_index = MOUSE_BUTTON_LEFT
		m.pressed = pr
		m.position = p
		m.global_position = p
		if pr:
			m.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(m)
		await _frames(2)
	await _frames(3)
func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)
## `{tool id: Button}` for the palette row's visible cells.
func _cells() -> Dictionary:
	var out := {}
	var row = _app._phone_tool_palette
	if row == null or not is_instance_valid(row):
		return out
	for b in (row as Control).find_children("*", "Button", true, false):
		out[String(b.get_meta(DccWidgets.TOOL_ID_META, ""))] = b
	return out
func _ids() -> Array:
	var k := _cells().keys()
	k.sort()
	return k
func _find_button(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null
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
	if not DccTheme.is_phone():
		## Tablet and desktop have no sheet: the palette bar at the top IS the palette, drawn from the same registry. In SCULPT
		## mode it carries Sculpt and Biome & Terrain Paint after the three global tools and the Pan legend.
		var ids := func() -> Array:
			var out: Array = []
			for b in _app._tool_strip_row.find_children("*", "Button", true, false):
				if b.has_meta(DccWidgets.TOOL_ID_META):
					out.append(String(b.get_meta(DccWidgets.TOOL_ID_META)))
			return out
		_ok(ids.call() == ["inspect", "measure", "region"], "%s: WORLD pipeline bar is inspect, measure, region (got %s)" % ["tablet" if DccTheme.is_tablet() else "desktop", str(ids.call())])
		_app.select_domain_mode("world", "b")
		await _settle()
		_ok(ids.call() == ["inspect", "measure", "region", "sculpt", "paint"], "SCULPT mode adds Sculpt and Biome & Terrain Paint (got %s)" % str(ids.call()))
		_ok(_app._phone_tool_palette == null, "no phone palette row exists off the phone")
		if _arg("--shot", "") != "":
			_vp.get_texture().get_image().save_png(_arg("--shot", "") + "_bar.png")
		return
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	var cell_c: Vector2 = (_app._phone_tab_cells["domain"] as Dictionary)["button"].get_global_rect().get_center()
	if _app._phone_detent == "peek":
		await _tap(cell_c)
		await _settle()
	var fl: float = 44.0 * float(_app.phone_scale()) - 1.0
	var host: Control = _app._phone_dom_host
	# WORLD, PIPELINE mode: the three global tools only
	_ok(_app._active_domain == "world", "boots in WORLD")
	_ok(_ids() == ["inspect", "measure", "region"], "WORLD pipeline palette is exactly inspect, measure, region (got %s)" % str(_ids()))
	_ok(_app._phone_tool_palette != null and _app._phone_tool_palette.get_parent() == host and _app._phone_tool_palette.get_index() == 0,
		"the palette is the FIRST child of the domain sheet's host")
	# the dock's SCULPT segment adds Sculpt and Biome & Terrain Paint
	var sculpt := _find_button(_app._left_dock_mode_switch, "SCULPT")
	_ok(sculpt != null, "the dock's SCULPT segment exists")
	if sculpt != null:
		await _tap(sculpt.get_global_rect().get_center())
		await _settle()
	_ok(_ids() == ["inspect", "measure", "paint", "region", "sculpt"], "WORLD sculpt palette adds paint and sculpt (got %s)" % str(_ids()))
	var worst := 1e9
	for b in _cells().values():
		worst = minf(worst, (b as Button).get_global_rect().size.y)
		worst = minf(worst, (b as Button).get_global_rect().size.x)
	_ok(worst >= fl, "every cell is at the 44 dp floor (smallest side %.1f px, floor %.1f)" % [worst, fl])
	# no second chip strip: the host's direct children are the palette, the dock's switch and its scroller (and CIVIL's door)
	var kinds: Array = []
	for c in host.get_children():
		kinds.append(c.name)
	_ok(kinds.count("DomToolPalette") == 1, "exactly one palette row in the host (%s)" % str(kinds))
	# arming from the sheet
	var sc: Button = _cells().get("sculpt")
	if sc != null:
		await _tap(sc.get_global_rect().get_center())
		await _settle()
		_ok(String(_app.armed_tool) == "sculpt", "tapping Sculpt arms the sculpt tool (armed %s)" % _app.armed_tool)
		_ok((_cells()["sculpt"] as Button).button_pressed, "the Sculpt cell is lit")
	var pipe := _find_button(_app._left_dock_mode_switch, "PIPELINE")
	if pipe != null:
		await _tap(pipe.get_global_rect().get_center())
		await _settle()
	_ok(_ids() == ["inspect", "measure", "region"], "back in PIPELINE the palette drops sculpt and paint (got %s)" % str(_ids()))
	var shot := _arg("--shot", "")
	if shot != "":
		_vp.get_texture().get_image().save_png(shot + "_world.png")
	# CARTO
	_app._select_domain("cartography")
	await _settle()
	_ok(_ids() == ["icon", "inspect", "label", "measure", "poi", "region"], "CARTO palette is icon, label, poi + globals (got %s)" % str(_ids()))
	if shot != "":
		_vp.get_texture().get_image().save_png(shot + "_carto.png")
	var pin: Button = _cells().get("poi")
	if pin != null:
		await _tap(pin.get_global_rect().get_center())
		await _settle()
		_ok(String(_app.armed_tool) == "poi", "tapping Pins arms poi (armed %s)" % _app.armed_tool)
		_ok(_app._phone_tool_palette == null, "while the Pins tool owns the sheet the palette row is freed")
	_app.arm_tool("inspect")
	await _settle()
	# CIVIL
	_app._select_domain("civilization")
	await _settle()
	_ok(_ids() == ["conflict", "inspect", "measure", "region", "route", "settlement", "territory", "territory_lasso", "way"],
		"CIVIL palette is its six tools + globals (got %s)" % str(_ids()))
	if shot != "":
		_vp.get_texture().get_image().save_png(shot + "_civil.png")
