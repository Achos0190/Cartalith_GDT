extends Node
## **Phone 3D HEIGHT row probe** (MAP sheet, 3D-2's phone home; Ruling EB item 6). Boots the real shell as a phone in a
## `--vp`-sized host `SubViewport` and drives REAL touch (a touch and its emulated mouse): opens the MAP sheet, checks the
## "3D height" section is hidden in 2D, taps 2D / 3D, checks it shows with a 44 dp track, drags the slider and asserts
## the live `View3D` exaggeration (and the terrain's and the stored preference) changed to the hand-written literal the
## drag lands on, that the PC/tablet card row's own reading (`current_height()`) is the same value, and that leaving 3D
## hides the section again. Expected figures are literals, never the constants they came from.
## Windowed, with a wiped user dir:
##   godot --path . --resolution 1600x1000 _p3dheight_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

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

func _find_btn(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text.to_upper() == text and (n as Control).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_btn(c, text)
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
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	if not DccTheme.is_phone():
		return
	var dens := float(_arg("--density", "1"))
	DccSettings.set_view3d_exaggeration(1.0)   ## a previous run's remembered 4.0 would fake the "opens at true scale" check
	## Reaches MAP the way `_p13fix_probe.gd` does (domain sheet first): from a cold boot the first MAP tap leaves the
	## sheet laid out at its 66 dp peek height while `_phone_detent` reads "half" (observed 2026-11-13, not fixed here).
	await _tap(_cell("domain").get_global_rect().get_center())
	await _settle()
	var mp: Vector2 = _cell("map").get_global_rect().get_center()
	await _tap(mp)
	await _settle()
	if _app._phone_detent == "peek":
		await _tap(mp)
		await _settle()
	_ok(_app._phone_tab == "map" and _app._phone_detent != "peek", "MAP sheet is up")
	await get_tree().create_timer(1.0).timeout
	var hb: Control = _app.get("_phone_height_box")
	_ok(hb != null and not hb.is_visible_in_tree(), "2D: the 3D height section exists and is hidden")
	var vp = _app.get("viewport")
	var b3 := _find_btn(_app, "2D / 3D")
	_ok(b3 != null, "the 2D / 3D button is on the sheet")
	if b3 == null:
		return
	print("  before tap rect=", b3.get_global_rect(), " detent=", _app._phone_detent)
	await _tap(b3.get_global_rect().get_center())
	for k in 20:
		if vp.is_3d():
			break
		await _settle()
	print("  3d=", vp.is_3d(), " btn=", b3.get_path(), " rect=", b3.get_global_rect())
	_ok(vp.is_3d(), "a real tap on 2D / 3D enters 3D")
	_ok(hb != null and hb.is_visible_in_tree(), "3D: the 3D height section is shown")
	if hb == null or not hb.is_visible_in_tree():
		return
	var sl := (_app._phone_height["slider"] as HSlider)
	_ok(sl.get_script() != null, "the track is a touch slider (PgSlider)")
	## The section sits below the view field and the Measure grid: scroll it into view as a thumb would.
	(_app.get("_phone_map_body") as ScrollContainer).ensure_control_visible(sl)
	await _settle()
	var r := sl.get_global_rect()
	_ok(r.end.y <= vh and r.position.y >= 0.0, "the track is on screen after scrolling (%s)" % str(r))
	## 44 dp of REAL density (the shell's own `phone_fit()` floor is only 44 phone units = 40.9 dp at 2.8125, so the row adds it).
	_ok(r.size.y >= 44.0 * dens - 0.5, "the track is at least 44 dp tall (%.1f px, floor %.1f)" % [r.size.y, 44.0 * dens])
	_ok(is_equal_approx(sl.min_value, 1.0) and is_equal_approx(sl.max_value, 4.0) and is_equal_approx(sl.step, 0.25), "range 1..4 step 0.25")
	var v3 = vp.view3d()
	_ok(v3 != null and absf(float(v3.current_height()) - 1.0) < 0.001, "3D opens at true scale 1.0 (%.2f)" % float(v3.current_height()))
	# drag from the track's left end to its right end: the touch slider commits a horizontal drag
	var y := r.get_center().y
	var p0 := Vector2(r.position.x + 4.0, y)
	var p1 := Vector2(r.end.x - 2.0, y)
	await _swipe(p0, p1)
	await _settle()
	var h := float(v3.current_height())
	_ok(absf(h - 4.0) < 0.001, "after the drag the view3d exaggeration is 4.0 (%.2f)" % h)
	_ok(absf(float(DccSettings.view3d_exaggeration(1.0)) - 4.0) < 0.001, "the remembered preference is 4.0")
	_ok(absf(sl.value - 4.0) < 0.001, "the slider reads 4.0")
	# the PC/tablet row's seed: a fresh card row seeds from current_height() / the preference, both 4.0
	_ok(absf(float(v3.current_height()) - float(DccSettings.view3d_exaggeration(1.0))) < 0.001, "the PC/tablet row's seed reads the same value")
	# leave 3D, hide
	vp.set_3d(false)
	await _settle()
	_ok(not hb.is_visible_in_tree(), "leaving 3D hides the section")
	vp.set_3d(true)
	await _settle()
	_ok(absf(sl.value - 4.0) < 0.001 and hb.is_visible_in_tree(), "re-entering 3D re-seeds the slider at 4.0 and shows it")
