extends Node
## PHONE leg of the batch-32 lane-A verification: the same gate, the same pill,
## read out of the phone SHEET rather than the desktop dock. A restructure that
## is right on desktop and empty on the sheet is half done.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     --resolution 1080x2340 _vfy_ld32_phone.tscn -- --force-touch

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("VP32 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _rendered_cats(id: String) -> Array:
	var p: Control = app.workspace_panel(id) as Control
	var out: Array = []
	for e in p.categories:
		var body: Control = e.get("body")
		if body == null or not is_instance_valid(body):
			continue
		var wrap: Control = body.get_parent() as Control
		if wrap != null and wrap.visible:
			out.append(String(e.get("title", "")))
	return out

func _open_cats(id: String) -> Array:
	var p: Control = app.workspace_panel(id) as Control
	var out: Array = []
	for e in p.categories:
		var body: Control = e.get("body")
		if body != null and is_instance_valid(body) and body.visible:
			out.append(String(e.get("title", "")))
	return out

func _ready() -> void:
	print("VP32 force-touch=%s" % ("--force-touch" in OS.get_cmdline_user_args()))
	## Booted into a SubViewport sized like a handset, because
	## `DccShell._compute_layout_mode()` reads `get_viewport_rect().size` and a
	## real 1080x2340 window is clamped by the desktop before it is ever read
	## (measured: --resolution 1080x2340 windowed classified as TABLET, phone
	## scale 2.502 off a 1030-wide clamped window).
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_ck("the phone composition actually built", DccTheme.is_phone(),
		"is_phone=%s is_touch=%s" % [DccTheme.is_phone(), DccTheme.is_touch()])
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)

	## The sheet is built but hidden until opened; the gate does not care, but
	## a measurement does, so open it.
	app.select_domain("world")
	await _frames(3)
	if app.left_dock != null:
		app.left_dock.visible = true
	await _frames(4)

	_ck("the mode pill exists in the phone SHEET",
		app._mode_switch_row != null and is_instance_valid(app._mode_switch_row))
	_ck("pill is inside the sheet, not the desktop dock",
		app._mode_switch_row != null and app.left_dock.is_ancestor_of(app._mode_switch_row))
	_ck("pill shown in WORLD", app._mode_switch_row.visible)
	var seg_a: Button = app._mode_switch_buttons["a"]
	var seg_b: Button = app._mode_switch_buttons["b"]
	print("VP32   phone_scale=%.3f  segment size=%s / %s  pill min=%s" % [
		DccTheme._phone_scale, str(seg_a.get_combined_minimum_size()),
		str(seg_b.get_combined_minimum_size()),
		str(app._mode_switch_row.get_combined_minimum_size())])
	var floor_px: float = 44.0 * DccTheme._phone_scale
	_ck("segment clears the phone tap floor",
		seg_a.get_combined_minimum_size().y >= floor_px - 0.5,
		"got=%.1f floor=%.1f" % [seg_a.get_combined_minimum_size().y, floor_px])

	app.select_domain_mode("world", "a")
	await _frames(3)
	_ck("phone world/a renders nine", _rendered_cats("world").size() == 9,
		"cats=%d" % _rendered_cats("world").size())
	seg_b.pressed.emit()
	await _frames(4)
	_ck("phone pill b -> Terrain alone", _rendered_cats("world") == ["Terrain"],
		"cats=%s open=%s" % [str(_rendered_cats("world")), str(_open_cats("world"))])
	_ck("phone world/b leaves a body open", _open_cats("world") == ["Terrain"],
		"open=%s" % str(_open_cats("world")))
	seg_a.pressed.emit()
	await _frames(4)
	_ck("phone pill a -> nine back", _rendered_cats("world").size() == 9,
		"cats=%d" % _rendered_cats("world").size())

	app.select_domain("civilization")
	await _frames(4)
	_ck("phone CIVIL hides the pill", not app._mode_switch_row.visible)
	_ck("phone CIVIL still renders fifteen", _rendered_cats("civilization").size() == 15,
		"cats=%d" % _rendered_cats("civilization").size())
	app.select_domain("cartography")
	await _frames(4)
	_ck("phone CARTO hides the pill", not app._mode_switch_row.visible)
	_ck("phone CARTO still renders ten", _rendered_cats("cartography").size() == 10,
		"cats=%d" % _rendered_cats("cartography").size())

	print("VP32 DONE  failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
