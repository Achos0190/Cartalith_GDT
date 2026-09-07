extends Node
## VERIFIER probe, 2026-09-07. Measures the tablet frame fit with **no
## reference to any symbol added by the batch under test**, so the identical
## file runs against the clean HEAD tree and against the working tree and the
## two runs are comparable. `is_tablet_portrait()` does not exist at HEAD; the
## lane's own `_tabfit_probe.gd` therefore cannot produce a before-state.
##
##   Godot --headless --path . _vfy_tabfit_probe.tscn -- --force-touch --vp 800x1280
##   Godot --headless --path . _vfy_tabfit_probe.tscn --              --vp 1920x1080
##
## Optional `--rotate` drives the orientation-flip path.
## Every printed figure is a measurement; assertions are literals off the
## canvas (`Cartalith Tablet.dc.html` `valsShell()` portrait `--ldW:232px`,
## `ENV:25` `--ldW:372px --rdW:304px`, `ENV:1819` `w1366` 330/280).

var app: Node
var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _arg(nm: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(nm)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _has(nm: String) -> bool:
	return nm in OS.get_cmdline_user_args()

func _eq(label: String, got: float, want: float) -> void:
	var ok := absf(got - want) < 0.5
	if not ok:
		_fail += 1
	print("  [%s] %-44s got=%8.1f want=%8.1f" % ["ok" if ok else "FAIL", label, got, want])

func _watchdog(sec: float) -> void:
	await get_tree().create_timer(sec).timeout
	print("[RESULT] WATCHDOG failures=%d" % (_fail + 1))
	get_tree().quit(2)

func _dump(tag: String, vw: int) -> void:
	var ld := app.left_dock as Control
	var rd := app.right_dock as Control
	var rail := app.rail_column as Control
	var dock_row := ld.get_parent() as Control
	var shell_vb := dock_row.get_parent() as Control
	print("[%s] touch=%s tablet=%s phone=%s laptop=%s" % [tag,
		str(DccTheme.is_touch()), str(DccTheme.is_tablet()),
		str(DccTheme.is_phone()), str(DccTheme.is_laptop())])
	print("   left  cmin=%7.1f size=%7.1f   right cmin=%7.1f size=%7.1f" % [
		ld.custom_minimum_size.x, ld.size.x, rd.custom_minimum_size.x, rd.size.x])
	print("   role_px  left=%d right=%d" % [
		DccTheme.role_px("w_left_dock"), DccTheme.role_px("w_right_dock")])
	print("   rail cmin=%7.1f  dock_row cmin=%7.1f  shellVB cmin=%7.1f size=%7.1f" % [
		rail.get_combined_minimum_size().x, dock_row.get_combined_minimum_size().x,
		shell_vb.get_combined_minimum_size().x, shell_vb.size.x])
	print("   OVERFLOW shellVB size.x - frame = %+.1f   right_dock right edge x=%.1f" % [
		shell_vb.size.x - float(vw), rd.global_position.x + rd.size.x])
	for pair in [["menu_bar_row", app.menu_bar_row], ["tool_options_row", app.tool_options_row],
			["status_row", app.status_row]]:
		var row := pair[1] as Control
		if row != null:
			print("   %-18s cmin.x=%7.1f" % [pair[0], row.get_combined_minimum_size().x])

func _ready() -> void:
	_watchdog(180.0)
	var parts: PackedStringArray = _arg("--vp", "800x1280").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(8)

	print("=== vfy_tabfit vp=%dx%d portrait=%s ===" % [vw, vh, str(vh > vw)])
	_dump("boot", vw)

	## Canvas literals. Portrait tablet 232 (`--ldW:232px`, Tablet canvas
	## `valsShell()` portrait branch); landscape tablet 400 (`W_DOCK_TABLET`,
	## unchanged this batch); pointer 372/304 (`ENV:25`); 1366 band 330/280
	## (`ENV:1819`). Written out, never read off the constant under test.
	var want_l := 372.0
	var want_r := 304.0
	if DccTheme.is_tablet():
		want_l = 232.0 if vh > vw else 400.0
		want_r = 232.0 if vh > vw else 400.0
	elif DccTheme.is_laptop():
		want_l = 330.0
		want_r = 280.0
	var ld := app.left_dock as Control
	var rd := app.right_dock as Control
	_eq("left_dock reservation", ld.custom_minimum_size.x, want_l)
	_eq("right_dock reservation", rd.custom_minimum_size.x, want_r)
	_eq("role_px(w_left_dock)", float(DccTheme.role_px("w_left_dock")), want_l)
	_eq("role_px(w_right_dock)", float(DccTheme.role_px("w_right_dock")), want_r)

	## §2 -- the flip path, and WI-04 in the same run.
	if _has("--rotate"):
		print("[rotate] --- WI-04: a dragged width, then a SAME-orientation resize")
		ld.custom_minimum_size.x = 355.0
		app.set("_left_width", 355.0)
		await _frames(4)
		_vp.size = Vector2i(vw + 120, vh + 40)
		await _frames(4)
		app.call("_on_window_resized")
		await _frames(6)
		_eq("same-orientation resize keeps the dragged width", ld.custom_minimum_size.x, 355.0)
		print("[rotate] --- the flip")
		_vp.size = Vector2i(vh, vw)
		await _frames(4)
		app.call("_on_window_resized")
		await _frames(8)
		var flipped_portrait := vw > vh
		var fl := 400.0
		if DccTheme.is_tablet():
			fl = 232.0 if flipped_portrait else 400.0
		_dump("after flip to %dx%d" % [vh, vw], vh)
		_eq("left_dock after flip", ld.custom_minimum_size.x, fl)
		_eq("right_dock after flip", rd.custom_minimum_size.x, fl)

	print("=== end vfy_tabfit fails=%d ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
