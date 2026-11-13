extends Node
## **Preferences ▸ Graphics ▸ 3D height probe** (3D-2's Preferences control; Ruling EB items 5-6). Boots the real shell in a
## `--vp`-sized host `SubViewport` (desktop or phone/touch), picks a rung in the REAL Preferences submenu (`Height3D`, the
## popup's own `id_pressed`), and asserts the live `View3D`, the stored preference and the form's HEIGHT control (the card's
## slider on desktop/tablet, the MAP sheet's on the phone) all read the literal 3.0; then moves the HEIGHT control to 1.5
## and asserts Preferences' checked rung reads it back. Also: 13 rungs 1.0..4.0 step 0.25, one checked, and on touch the
## rows are at least 44 dp tall. Expected figures are literals, never the constants they came from.
## Windowed, wiped user dir: godot --path . --resolution 1600x1000 _prefheight_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
## (desktop: `--vp 1600x1000`). Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print(("ok:   " if cond else "FAIL: "), what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _settle() -> void:
	await get_tree().create_timer(0.4).timeout
	await _frames(3)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density"]:
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
	DccSettings.set_view3d_exaggeration(1.0)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _find_node(n: Node, nm: String) -> Node:
	if n.name == nm:
		return n
	for c in n.get_children():
		var r := _find_node(c, nm)
		if r != null:
			return r
	return null

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

## Checked-rung indices of the Preferences popup (must be exactly one for an on-ladder value).
func _checked(pm: PopupMenu) -> Array:
	var a := []
	for i in pm.item_count:
		if pm.is_item_checked(i):
			a.append(i)
	return a

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
	var phone := DccTheme.is_phone()
	var dens := float(_arg("--density", "1"))
	print("form: phone=", phone, " touch=", DccTheme.is_touch())
	DccSettings.clear_view3d_exaggeration()
	var vp = _app.get("viewport")
	var v3 = vp.ensure_view3d()
	var pm = _app.menus.get("_h3d_popup") as PopupMenu
	_ok(pm != null, "Preferences has a 3D height submenu")
	if pm == null:
		return
	_ok(pm.item_count == 13, "13 rungs (1.0 to 4.0 by 0.25)")
	_ok(pm.get_item_text(0) == "1.00x" and pm.get_item_text(12) == "4.00x" and pm.get_item_text(5) == "2.25x", "rung labels are 1.00x, 2.25x, 4.00x at 0, 5, 12")
	if DccTheme.is_touch():
		var per := pm.get_contents_minimum_size().y / 13.0 * maxf(float(_app._phone_scale), 1.0)   ## logical px to real px
		print("  popup row ~", per, " px, floor ", 44.0 * dens)
		_ok(per >= 44.0 * dens - 0.5, "touch: each rung row is at least 44 dp (%.1f px vs %.1f)" % [per, 44.0 * dens])
	if phone:
		var mp: Vector2 = _cell("map").get_global_rect().get_center()
		var t := InputEventScreenTouch.new()
		t.pressed = true
		t.position = mp
		_vp.push_input(t)
		var m := InputEventMouseButton.new()
		m.button_index = MOUSE_BUTTON_LEFT
		m.pressed = true
		m.position = mp
		_vp.push_input(m)
		await _frames(2)
		t = t.duplicate()
		t.pressed = false
		m = m.duplicate()
		m.pressed = false
		_vp.push_input(t)
		_vp.push_input(m)
		await _settle()
	## Preferences -> everything (rung 8 = 1.0 + 8 * 0.25 = 3.0)
	pm.id_pressed.emit(680 + 8)
	await _frames(2)
	_ok(absf(float(v3.current_height()) - 3.0) < 0.001, "Preferences rung 3.00x: the view reads 3.0 (%.2f)" % float(v3.current_height()))
	_ok(absf(float(DccSettings.view3d_exaggeration(1.0)) - 3.0) < 0.001, "the stored preference is 3.0")
	var row: HSlider = (_app._phone_height["slider"] as HSlider) if phone else (v3.get("_height_slider") as HSlider)
	_ok(row != null and absf(row.value - 3.0) < 0.001, "the form's HEIGHT control reads 3.0 (%s)" % [str(row.value) if row != null else "none"])
	pm.about_to_popup.emit()
	_ok(_checked(pm) == [8], "only the 3.00x rung is checked (%s)" % str(_checked(pm)))
	## HEIGHT control -> Preferences (1.5 = rung 2)
	if row != null:
		row.value = 1.5
	await _frames(2)
	_ok(absf(float(v3.current_height()) - 1.5) < 0.001, "HEIGHT control to 1.5: the view reads 1.5")
	pm.about_to_popup.emit()
	_ok(_checked(pm) == [2], "Preferences reads it back: only the 1.50x rung is checked (%s)" % str(_checked(pm)))
	## Preferences -> the last rung and the first (clamp ends)
	pm.id_pressed.emit(680 + 12)
	_ok(absf(float(v3.current_height()) - 4.0) < 0.001, "rung 4.00x reaches the view")
	pm.id_pressed.emit(680)
	_ok(absf(float(v3.current_height()) - 1.0) < 0.001, "rung 1.00x reaches the view")
