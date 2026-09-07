extends Node
## TABLET-SPEC lane, 2026-09-07. Measures the SHIPPED tablet composition's
## chrome so `TABLET_UI_SPEC.md`'s "what exists today" column is measured
## rather than read off a constant table.
##
##   godot --path . _tabspec_probe.tscn -- --force-touch --vp 1600x1000 --tag land
##   godot --path . _tabspec_probe.tscn -- --force-touch --vp 800x1280  --tag port
##
## Reports, per region, the laid-out pixel size against the tablet canvas's own
## figure (`Cartalith Tablet.dc.html`, `valsShell()`). Reads nothing; writes
## nothing; changes no behaviour.

var app: Node
var _vp: SubViewport
var _tag := "tab"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _w(label: String, node: Node, canvas_want: float) -> void:
	if node == null:
		print("  %-22s ABSENT" % label)
		return
	var c := node as Control
	print("  %-22s w=%7.1f  canvas=%6.1f  vis=%s" % [label, c.size.x, canvas_want, str(c.visible)])

func _h(label: String, node: Node, canvas_want: float) -> void:
	if node == null:
		print("  %-22s ABSENT" % label)
		return
	var c := node as Control
	print("  %-22s h=%7.1f  canvas=%6.1f  vis=%s" % [label, c.size.y, canvas_want, str(c.visible)])

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_tag = _arg("--tag", "tab")
	var vw := int(parts[0])
	var vh := int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)

	var portrait := vh > vw
	print("=== %s vp=%dx%d portrait=%s touch=%s tablet=%s phone=%s ===" % [
		_tag, vw, vh, str(portrait), str(DccTheme.is_touch()),
		str(DccTheme.is_tablet()), str(DccTheme.is_phone())])

	print("[heights]  (canvas: menuH 52, railH 56, sbH 28)")
	_h("menu_bar_row", app.menu_bar_row, 52.0)
	_h("tool_options_row", app.tool_options_row, 56.0)
	_h("timeline_bar", app.timeline_bar, 28.0)
	_h("status_row", app.status_row, 28.0)

	print("[widths]   (canvas: railW 52, ldW/rdW %d)" % (232 if portrait else 320))
	var want_dock := 232.0 if portrait else 320.0
	_w("rail_column", app.rail_column, 52.0)
	_w("left_dock", app.left_dock, want_dock)
	_w("right_dock", app.right_dock, want_dock)
	_w("viewport_area", app.viewport_area, 0.0)

	print("[fit]")
	var shell: Control = app.shell if "shell" in app else null
	if shell == null:
		for ch in app.get_children():
			if ch is Control and ch.get_class() == "Control":
				pass
	var vpa := app.viewport_area as Control
	print("  viewport_area  x=%.1f..%.1f   frame_w=%d" % [
		vpa.global_position.x, vpa.global_position.x + vpa.size.x, vw])
	var rd := app.right_dock as Control
	print("  right_dock     x=%.1f..%.1f   overflow=%.1f" % [
		rd.global_position.x, rd.global_position.x + rd.size.x,
		maxf(0.0, rd.global_position.x + rd.size.x - float(vw))])
	print("  sum rail+ld+vp+rd = %.1f" % [
		(app.rail_column as Control).size.x + (app.left_dock as Control).size.x
		+ vpa.size.x + rd.size.x])
	print("  viewport_area min.x=%.1f  left_dock min.x=%.1f  right_dock min.x=%.1f" % [
		vpa.get_combined_minimum_size().x,
		(app.left_dock as Control).get_combined_minimum_size().x,
		rd.get_combined_minimum_size().x])

	print("[menu titles]")
	var titles: Array[String] = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			titles.append((ch as MenuButton).text)
		elif ch is MenuBar:
			for i in (ch as MenuBar).get_menu_count():
				titles.append((ch as MenuBar).get_menu_title(i))
	print("  count=%d  %s" % [titles.size(), ", ".join(titles)])
	print("  canvas draws 3 (File, World, Data) + one overflow square")

	print("[role table, live]")
	print("  h_menu_bar=%s h_status=%s w_rail=%s w_left_dock=%s w_right_dock=%s" % [
		str(DccTheme.role_px("h_menu_bar")), str(DccTheme.role_px("h_status")),
		str(DccTheme.role_px("w_rail")), str(DccTheme.role_px("w_left_dock")),
		str(DccTheme.role_px("w_right_dock"))])
	print("  TOUCH_SCALE=%.2f" % DccTheme.TOUCH_SCALE)
	print("=== end %s ===" % _tag)
	get_tree().quit()
