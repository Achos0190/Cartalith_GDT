extends Node
## COMPOSITION lane, 2026-09-12; corrected 2026-09-13. Guards the tablet frame
## floor (`tool_options_row` no longer propagating its minimum). The landscape
## dock split this probe used to guard (400 -> 320) was itself **reverted**
## 2026-09-13 -- see `DccTheme.W_DOCK_TABLET`'s own header -- so the landscape
## expectation below is corrected back to 400, matching the reverted `ROLE`
## row, rather than left asserting a figure the shipped code no longer
## produces.
##
##   Godot_v4.7.1 --path . _compfit_probe.tscn -- --force-touch --vp 800x1280
##   Godot_v4.7.1 --path . _compfit_probe.tscn -- --force-touch --vp 1280x800
##   Godot_v4.7.1 --path . _compfit_probe.tscn -- --force-touch --vp 1024x768
##
## Run WINDOWED (no --headless) per the brief -- this measures drawn rects
## (`global_position`), not only declared minimums.
##
## Every "want" below is a literal written out, never read off the constant
## under test (`DccTheme.ROLE`/`TABLET_PORTRAIT`) -- mutating those constants
## must move the comparison on one side only. Portrait 232 is
## `Cartalith Tablet.dc.html`'s `--ldW:232px` (`valsShell()` portrait branch),
## unaffected by the landscape revert -- `TABLET_PORTRAIT` is a separate table
## from `ROLE`. Landscape is still 400, `ROLE`'s shipped figure: the canvas's
## own 320 landscape branch is deferred, not carried in, pending a content
## reflow (`DccTheme.W_DOCK_TABLET`'s header has the measurement that deferred
## it).

var _fail := 0
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _eq(label: String, got: float, want: float) -> void:
	var ok := absf(got - want) < 0.5
	if not ok:
		_fail += 1
	print("  [%s] %-46s got=%8.1f want=%8.1f" % ["ok" if ok else "FAIL", label, got, want])

func _le(label: String, got: float, cap: float) -> void:
	var ok := got <= cap + 0.5
	if not ok:
		_fail += 1
	print("  [%s] %-46s got=%8.1f  cap=%8.1f" % ["ok" if ok else "FAIL", label, got, cap])

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "800x1280").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(10)

	var portrait := vh > vw
	print("=== compfit vp=%dx%d portrait=%s tablet=%s tablet_portrait=%s ===" % [
		vw, vh, str(portrait), str(DccTheme.is_tablet()), str(DccTheme.is_tablet_portrait())])

	var ld := app.get("left_dock") as Control
	var rd := app.get("right_dock") as Control
	var rail := app.get("rail_column") as Control
	var dock_row := ld.get_parent() as Control
	var shell_vb := dock_row.get_parent() as Control
	var tor := app.get("tool_options_row") as Control

	print("[dock reservation -- literal, not read off the constant under test]")
	var want_dock := 232.0 if portrait else 400.0
	_eq("role_px(w_left_dock)", float(DccTheme.role_px("w_left_dock")), want_dock)
	_eq("role_px(w_right_dock)", float(DccTheme.role_px("w_right_dock")), want_dock)
	_eq("left_dock custom_minimum_size.x", ld.custom_minimum_size.x, want_dock)
	_eq("right_dock custom_minimum_size.x", rd.custom_minimum_size.x, want_dock)

	print("[tool_options_row -- must not propagate its own minimum]")
	var tor_min := tor.get_combined_minimum_size().x if tor != null else -1.0
	## `tor.get_parent()` is always `pad` (a `MarginContainer`, unconditional in
	## `_build_tool_options_bar()`) -- the ScrollContainer, when present, wraps
	## `pad`, not `tool_options_row` directly. One level further up is where a
	## tablet composition actually differs from desktop/laptop, so that is what
	## is printed -- still informational, not asserted, so this probe reports
	## what it found rather than assuming the fix's shape.
	var pad_node := tor.get_parent() if tor != null else null
	var wrapper := pad_node.get_parent() if pad_node != null else null
	print("  tool_options_row combined_min.x = %.1f   wrapper class = %s" % [
		tor_min, wrapper.get_class() if wrapper != null else "<none>"])

	print("[row minimums]")
	var menu_row := app.get("menu_bar_row") as Control
	var status_row := app.get("status_row") as Control
	var timeline := app.get("timeline_bar") as Control
	# tool_options_row's band is its nearest PanelContainer ancestor.
	var band: Control = tor
	while band != null and not (band is PanelContainer):
		band = band.get_parent() as Control
	for pair in [["menu_bar_row", menu_row], ["tool_options band", band],
			["dock_row (rail+left+right)", dock_row], ["timeline_bar", timeline],
			["status_row", status_row]]:
		var row := pair[1] as Control
		if row != null:
			print("  %-28s min.x=%7.1f  size.x=%7.1f" % [pair[0],
				row.get_combined_minimum_size().x, row.size.x])

	print("[shell VBox direct children, unfiltered]")
	for c in shell_vb.get_children():
		if c is Control:
			var cc := c as Control
			print("  %-24s min.x=%7.1f  size.x=%7.1f  vis=%s" % [
				c.get_class() + ":" + c.name, cc.get_combined_minimum_size().x,
				cc.size.x, str(cc.visible)])

	print("[menu bar interior -- what shares `row` with menu_bar_row]")
	if menu_row != null:
		var mrow := menu_row.get_parent() as Control  ## the `row` HBox in _build_menu_bar()
		if mrow != null:
			for c in mrow.get_children():
				if c is Control:
					var cc2 := c as Control
					print("  %-24s min.x=%7.1f  name=%s" % [
						c.get_class(), cc2.get_combined_minimum_size().x, c.name])

	print("[the fit itself]")
	_le("shell VBox combined minimum fits the frame", shell_vb.get_combined_minimum_size().x, float(vw))
	var right_edge := rd.global_position.x + rd.size.x
	_le("right dock DRAWN right edge fits the frame", right_edge, float(vw))
	var overflow := maxf(0.0, shell_vb.size.x - float(vw))
	print("  shell VBox size.x=%.1f  frame=%d  overflow=%.1f" % [shell_vb.size.x, vw, overflow])

	print("=== end compfit fails=%d ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
