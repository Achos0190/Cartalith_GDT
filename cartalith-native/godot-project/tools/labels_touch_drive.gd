extends RefCounted
## Owner report 2026-11-23 (phone build v.26.11.23-001): "on the phone its impossible to modify existing labels."
## One scripted session per form that drives EVERY label action with real input and asserts the result through the engine
## (the override stored, the text changed, the position moved), never through a frame. Real input: on a touch form each
## action is a ScreenTouch/ScreenDrag with its emulated mouse (`device = -1`, exactly as Godot stamps a finger); on the
## desktop it is a mouse (`device 0`). Nothing here calls a handler directly.
## Run: `python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/labels_touch_drive.gd --dll DLL --out DIR`
## Env: `LBL_SHOTS=<dir>` saves a frame after each step (look at them; a frame is the renderer's answer, the asserts are the engine's).
## Mutants that must turn it red: `tools/labels_touch_mutants.py`.
## Hand-placed label H: select (tap), form shown, rename (typed), move (drag, camera must not pan), resize (drag its handle with the
## view zoomed out until the drawn circle is smaller than a finger; the press lands in the finger pad), delete (list x).
## Generated label G: select, form, rename, move, size (slider), hide, restore. On a touch form every button and slider is held to the
## 44 dp floor. Phone: reached by swiping the domain tab to CARTO and pressing the sheet's Label tool. Every check is a literal.
## Findings this drive exists for: see `docs/evidence/labels-touch-2026-11-23/`.

var _fails := 0
var _ctx
var _app
var _ov
var _vp: Viewport
var _bridge
var _carto
var _touch := false
var _form := ""
var _shots := ""


func _check(what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	_ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _frames(n: int) -> void:
	for _i in n:
		await _app.get_tree().process_frame


func _shot(name: String) -> void:
	await _frames(3)
	var img: Image = await _ctx.frame(4096)
	if img != null and _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png("%s/%s_%s.png" % [_shots, _form, name])


# ---- real input ----------------------------------------------------------------------------------------------------

func _dn(p: Vector2) -> void:
	if _touch:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = true
		t.position = p
		_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION if _touch else 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)


func _mv(p: Vector2, from: Vector2) -> void:
	if _touch:
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - from
		_vp.push_input(d)
	var m := InputEventMouseMotion.new()
	m.device = InputEvent.DEVICE_ID_EMULATION if _touch else 0
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(m)
	await _frames(2)


func _up(p: Vector2) -> void:
	if _touch:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = false
		t.position = p
		_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION if _touch else 0
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_vp.push_input(m)
	await _frames(4)


func _tap(p: Vector2) -> void:
	if not _touch:
		var hv := InputEventMouseMotion.new()
		hv.position = p
		hv.global_position = p
		_vp.push_input(hv)
		await _frames(2)
	await _dn(p)
	await _up(p)
	await _frames(4)


func _drag(a: Vector2, b: Vector2, steps: int = 8) -> void:
	await _dn(a)
	var cur := a
	for s in steps:
		var nxt := a.lerp(b, float(s + 1) / float(steps))
		await _mv(nxt, cur)
		cur = nxt
	await _up(b)
	await _frames(4)


func _hold(p: Vector2, secs: float = 0.9) -> void:
	await _dn(p)
	await _ctx.wait(secs)
	await _up(p)
	await _frames(6)


# ---- geometry and reads ----------------------------------------------------------------------------------------------

func _screen(cell: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * _ov._cell_to_screen(cell, _ov._displayed_rect())


func _cell_at(screen: Vector2) -> Vector2:
	var loc: Vector2 = _ov.get_global_transform_with_canvas().affine_inverse() * screen
	var rect: Rect2 = _ov._displayed_rect()
	var g: Dictionary = _ov._grid_point(loc, rect, _ov._interior_rect(rect))
	return Vector2(float(g["gx"]), float(g["gy"])) if bool(g["valid"]) else Vector2(-1, -1)


func _cam() -> Vector2:
	return _app.viewport._camera.position


func _unit() -> float:
	return DccTheme.phone_scale() if DccTheme.is_phone() else 1.0


## True when `c` can be seen and pressed: visible in the tree, wholly on the screen, and wholly inside the viewport of every
## ScrollContainer above it (a control scrolled out of its container still reports a rect and `is_visible_in_tree()`).
func _shown(c: Control) -> bool:
	if c == null or not c.is_visible_in_tree():
		return false
	var r := DriveCtx.rect_of(c)
	if r.size.x <= 0.0 or not Rect2(Vector2.ZERO, _vp.get_visible_rect().size).encloses(r):
		return false
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer and not DriveCtx.rect_of(n as Control).grow(1.0).encloses(r):
			return false
		n = n.get_parent()
	return true


## First visible control (fully on screen and unclipped) named `nm`, else null.
func _on_screen(nm: String) -> Control:
	for n in _app.find_children(nm, "Control", true, false):   # a rebuilt form leaves its dying twin in the tree for a frame
		if _shown(n as Control):
			return n as Control
	return null


func _find_entry(t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(_ctx, t):
		if (cls == "" or e["class"] == cls) and str(e["text"]) == t:
			return e
	return {}


func _entry_centre(e: Dictionary) -> Vector2:
	var r: Array = e["rect"]
	return Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)


## The visible LineEdit whose text is `want`, fully on screen.
func _edit_with(want: String) -> LineEdit:
	for n in _app.find_children("*", "LineEdit", true, false):
		var le := n as LineEdit
		if le.text == want and _shown(le):
			return le
	return null


func _type(s: String) -> void:
	for ch in s:
		for down in [true, false]:
			var k := InputEventKey.new()
			k.pressed = down
			k.unicode = ch.unicode_at(0)
			k.keycode = OS.find_keycode_from_string(ch.to_upper()) if ch.to_upper() != ch.to_lower() else ch.unicode_at(0)
			k.physical_keycode = k.keycode
			_vp.push_input(k)
		await _frames(1)
	await _frames(3)


func _ctrl_a() -> void:
	for down in [true, false]:
		var a := InputEventKey.new()
		a.pressed = down
		a.keycode = KEY_A
		a.physical_keycode = KEY_A
		a.ctrl_pressed = true
		_vp.push_input(a)
	await _frames(2)


## Scroll every ScrollContainer above `c` so it shows (set-up for a tap: a user would scroll the sheet with a finger).
func _reveal(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(c)
		n = n.get_parent()
	await _frames(6)


func _tap_ctrl(c: Control) -> void:
	await _tap(DriveCtx.rect_of(c).get_center())


func _min_tap_ok(c: Control) -> bool:
	if not DccTheme.is_touch():
		return true   # the 44 dp floor is a touch rule; a mouse form keeps its own sizes
	var r := DriveCtx.rect_of(c)
	var need := 44.0 * (_unit() if DccTheme.is_phone() else 1.0)
	return r.size.x >= need - 1.0 and r.size.y >= need - 1.0


# ---- the session -------------------------------------------------------------------------------------------------------

func _boot() -> void:
	var size: Vector2 = _vp.get_visible_rect().size
	if size.x < 700.0:
		var nw: Dictionary = {}
		for _i in 80:
			nw = _find_entry("+ NEW WORLD")
			if not nw.is_empty():
				break
			await _ctx.wait(0.25)
		await DriveCtx.click(_ctx, _entry_centre(nw))
		var cancel: Dictionary = {}
		for _i in 80:
			cancel = _find_entry("CANCEL")
			if not cancel.is_empty():
				break
			await _ctx.wait(0.25)
		await DriveCtx.click(_ctx, _entry_centre(cancel))
	else:
		var cont: Dictionary = {}
		for _i in 80:
			cont = _find_entry("Continue without a world")
			if not cont.is_empty():
				break
			await _ctx.wait(0.25)
		await DriveCtx.click(_ctx, _entry_centre(cont))
	await _ctx.wait(1.0)
	_bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await _ctx.wait(2.0)
	for _i in 60:
		if _app.busy_overlay == null or not _app.busy_overlay.visible:
			break
		await _ctx.wait(0.5)
	await _frames(20)


## The phone's real route: the CARTO domain tab, then the Labels row. Elsewhere the dock rail's Labels entry.
func _reach_labels() -> void:
	var done := false
	if DccTheme.is_phone():
		# the domain tab stands for WORLD, CIVIL and CARTO and a sideways swipe on it steps to the next (`DOMAIN_ORDER`)
		var tab := {}
		for cap in ["GENERATE", "CIVIL", "CARTO"]:
			var cands := DriveCtx.dump_ui(_ctx, cap).filter(func(e): return str(e["text"]) == cap)
			cands.sort_custom(func(x, y): return x["rect"][1] > y["rect"][1])   # the bottom bar, not a sheet title
			if not cands.is_empty():
				tab = cands[0]
				break
		_check("phone: the domain tab is on screen", not tab.is_empty())
		for _i in 3:
			if tab.is_empty() or str(tab["text"]) == "CARTO":
				break
			var c := _entry_centre(tab)
			await _drag(c, c + Vector2(-90.0, 0.0), 6)
			await _ctx.wait(0.8)
			for e in DriveCtx.dump_ui(_ctx):
				if str(e["text"]) in ["GENERATE", "CIVIL", "CARTO"] and e["rect"][1] > 700.0:
					tab = e
		_check("phone: swiping the domain tab reaches CARTO", not tab.is_empty() and str(tab["text"]) == "CARTO")
		# CARTO's sheet carries the tool row (Inspect, Measure, Region select, Icon, Label, Point of interest) above its
		# category list; the Labels category is a long scroll down, which a finger drag does not reach (see the report).
		var row := {}
		for e in DriveCtx.dump_ui(_ctx):
			if e["class"] == "Button" and str(e["text"]) == "Label" and e["rect"][1] > 400.0:
				row = e
		_check("phone: CARTO's sheet offers the Label tool", not row.is_empty())
		if not row.is_empty():
			await _tap(_entry_centre(row))
			await _ctx.wait(0.8)
			done = true
	if not done:
		_app.select_domain_category("cartography", "Labels")
		await _ctx.wait(0.8)
	_check("the Label tool is armed after CARTO > Labels", _app.armed_tool == "label", _app.armed_tool)
	_ctx.log("   sheet detent after CARTO > Labels: %s tab=%s" % [(_app.phone_detent() if DccTheme.is_phone() else "n/a"), str(_app.get("_phone_tab"))])


func run(ctx) -> void:
	_ctx = ctx
	_app = ctx.get_scene_root()
	_vp = _app.get_viewport()
	_shots = OS.get_environment("LBL_SHOTS")
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_touch = DccTheme.is_touch()
	ctx.log("form %s canvas %s touch=%s" % [_form, str(_vp.get_visible_rect().size), str(_touch)])
	_bridge = _app.bridge
	await _boot()
	_ov = _app.viewport.overlay
	for w in _app._workspaces:
		if (w as Object).get_script().get_global_name() == "CartographyWorkspace":
			_carto = w
	_check("setup: world generated, CARTO workspace built", _bridge.has_world and _carto != null)
	await _reach_labels()
	await _hand_leg()
	await _auto_leg()
	ctx.log("labels touch drive (%s): %d failure(s)" % [_form, _fails])


## A free spot in the map's upper half (the phone's sheet covers the lower part), as screen px.
func _spot(fx: float, fy: float) -> Vector2:
	var r: Rect2 = (_app.viewport as Control).get_global_rect()
	return r.position + Vector2(r.size.x * fx, r.size.y * fy)


func _hand_leg() -> void:
	_ctx.log("== hand-placed label ==")
	if DccTheme.is_phone():
		_app.set_phone_detent("peek")   # the user has the map to themselves; the tap that selects must bring the form up
		await _ctx.wait(0.6)
	var at := _spot(0.2, 0.26)   # left of the zoom rocker, which the handle would otherwise sit under
	var cell := _cell_at(at)
	_check("setup: the spot is on the plate", cell.x >= 0.0, str(cell))
	var idx: int = _bridge.label_create(cell.x, cell.y, "Hand")
	_app.viewport.refresh_annotations()
	await _frames(6)
	_bridge.label_select(-1)
	await _frames(3)
	var p0 := _screen(cell)
	await _shot("hand_0_before")
	# select
	await _tap(p0)
	_check("H1 a tap on the label selects it", _bridge.label_get_selected() == idx, str(_bridge.label_get_selected()))
	if DccTheme.is_phone():
		await _ctx.wait(0.5)
	await _shot("hand_1_selected")
	# form
	var le := _edit_with("Hand")
	_check("H2 the label's form (its text field) is on screen after the tap", le != null)
	# rename
	if le != null:
		await _tap_ctrl(le)
		_check("H3a tapping the text field focuses it", _vp.gui_get_focus_owner() == le)
		if _vp.gui_get_focus_owner() == le:   # never type with no focus: the keys are the shell's shortcuts
			await _ctrl_a()
			await _type("Zorp")
		_check("H3 typing renames the hand-placed label", String(_bridge.label_get(idx).get("text", "")) == "Zorp",
			String(_bridge.label_get(idx).get("text", "")))
	await _frames(10)
	# move
	var x0 := float(_bridge.label_get(idx)["x"])
	var y0 := float(_bridge.label_get(idx)["y"])
	var cam0 := _cam()
	p0 = _screen(Vector2(x0, y0))
	await _ctx.wait(1.0)   # the sheet has risen and the form has scrolled into view
	await _shot("hand_1b_settled")
	await _drag(p0, p0 + Vector2(60.0, 25.0))   # past the 28 px touch slop, which is what makes a press a drag
	var x1 := float(_bridge.label_get(idx)["x"])
	_check("H4 a drag on the selected label moves it", absf(x1 - x0) > 1.0, "%f -> %f" % [x0, x1])
	_check("H4b ... and the map did not pan", _cam().distance_to(cam0) < 1.0, str(_cam() - cam0))
	await _shot("hand_2_moved")
	# resize via the handle
	# set-up, not under test: zoom out until the handle's own circle is smaller than a fingertip, so what is grabbed is the pad
	if DccTheme.is_phone():
		_app.set_phone_detent("peek")
		await _ctx.wait(0.6)
	for _z in 6:
		_app.viewport.zoom_step(0.6)
		await _ctx.wait(0.3)
		_app.viewport.move_view_to(float(_bridge.label_get(idx)["x"]), float(_bridge.label_get(idx)["y"]))
		await _ctx.wait(0.3)
		var hh: Dictionary = _bridge.label_handles(idx, _app.viewport.zoom(), _app.viewport.label_px_per_cell()).get("resize", {})
		if not hh.is_empty() and float(hh["r"]) * (_screen(Vector2(1.0, 0.0)) - _screen(Vector2.ZERO)).x < 0.7 * 22.0 * _unit():
			break
	var h: Dictionary = _bridge.label_handles(idx, _app.viewport.zoom(), _app.viewport.label_px_per_cell()).get("resize", {})
	_check("H5 the selected label has a resize handle", not h.is_empty())
	if not h.is_empty():
		var s0 := float(_bridge.label_get(idx)["size"])
		cam0 = _cam()
		var hp := _screen(Vector2(float(h["x"]), float(h["y"])))
		var r_px: float = float(h["r"]) * (_screen(Vector2(1.0, 0.0)) - _screen(Vector2.ZERO)).x
		var pad_px := 22.0 * _unit() if DccTheme.is_touch() else 0.0
		_ctx.log("   H5 handle radius %.1f px, finger pad %.1f px (zoom %.2f)" % [r_px, pad_px, _app.viewport.zoom()])
		_check("H5a set-up: the drawn handle is smaller than the finger pad (so the pad is what is grabbed)", not DccTheme.is_phone() or r_px < pad_px * 0.8, "%.1f vs %.1f" % [r_px, pad_px])
		hp += Vector2(1.0, 1.0).normalized() * pad_px * 0.9   # a press that misses the drawn circle but is inside the finger pad
		_check("H5a2 set-up: the press point is free map", _app.viewport._on_map(hp), str(hp))
		await _drag(hp, hp + Vector2(60.0, 40.0))
		var s1 := float(_bridge.label_get(idx)["size"])
		_check("H5b dragging the handle resizes the label", absf(s1 - s0) > 0.5, "%f -> %f" % [s0, s1])
		_check("H5c ... and the map did not pan", _cam().distance_to(cam0) < 1.0)
	await _shot("hand_3_resized")
	# delete: the list's x button
	var n0: int = _bridge.label_list().size()
	if DccTheme.is_phone():
		_app.set_phone_detent("half")
		await _ctx.wait(0.6)
	var dels: Array = []
	for b in _app.find_children("*", "Button", true, false):
		if (b as Button).text == "×" and (b as Button).is_visible_in_tree():
			dels.append(b)
	var delb: Button = null
	for b in dels:
		var bb := b as Button
		await _reveal(bb)
		if _shown(bb):
			delb = bb
	_check("H6 the label list shows a delete button on screen", delb != null)
	if delb != null:
		_check("H6b ... at least 44 dp across", _min_tap_ok(delb), str(DriveCtx.rect_of(delb).size))
		await _tap_ctrl(delb)
		_check("H6c tapping it deletes the label", _bridge.label_list().size() == n0 - 1, "%d -> %d" % [n0, _bridge.label_list().size()])
	if _bridge.label_list().size() >= n0 and n0 > 0:
		_bridge.label_clear_all()   # isolate the next leg from a failed delete
		_app.viewport.refresh_annotations()
		await _frames(6)


func _auto_leg() -> void:
	_ctx.log("== generated label ==")
	var best := {}
	var best_d := 0.0
	var all: Array = _bridge.auto_label_list()
	var gw := float(_bridge.grid_size().x)
	var gh := float(_bridge.grid_size().y)
	for d in all:
		var c := Vector2(float(d["x"]), float(d["y"]))
		if c.x < gw * 0.3 or c.x > gw * 0.7 or absf(c.y - gh * 0.5) > gh * 0.12:
			continue   # mid-plate, so the view can be centred on it (the camera is held to cover the map)
		var nearest := 1e9
		for o in all:
			if o != d:
				nearest = minf(nearest, c.distance_to(Vector2(float(o["x"]), float(o["y"]))))
		if nearest > best_d:
			best_d = nearest
			best = d
	if not best.is_empty():
		# set-up, not under test: bring it to a third of the way down the view, above the sheet
		if DccTheme.is_phone():
			_app.set_phone_detent("peek")
			await _ctx.wait(0.6)
		_app.viewport.move_view_to(float(best["x"]), float(best["y"]))
		await _ctx.wait(0.3)
	_check("setup: a generated label is centred in the view", not best.is_empty(), str(all.size()))
	if best.is_empty():
		return
	var key := String(best["key"])
	var name0 := String(best["text"])
	var x0 := float(best["x"])
	var y0 := float(best["y"])
	var p0 := _screen(Vector2(x0, y0))
	await _shot("auto_0_before")
	# select
	await _tap(p0)
	_check("G1 a tap on a generated label selects it", _carto._auto_sel == key, _carto._auto_sel)
	await _ctx.wait(0.7)   # the form scrolls into view once the sheet has settled
	await _shot("auto_1_selected")
	var le := _on_screen("AutoLabelText") as LineEdit
	_check("G2 its form (the text field) is on screen after the tap", le != null)
	if le != null:
		await _tap_ctrl(le)
		_check("G3a tapping the text field focuses it", _vp.gui_get_focus_owner() == le)
		if _vp.gui_get_focus_owner() == le:
			await _ctrl_a()
			await _type("Zork")
		_check("G3 typing renames the generated label", String(_bridge.auto_label_get(key).get("text", "")) == "Zork",
			String(_bridge.auto_label_get(key).get("text", "")))
	await _frames(10)
	# move
	var cam0 := _cam()
	p0 = _screen(Vector2(float(_bridge.auto_label_get(key)["x"]), float(_bridge.auto_label_get(key)["y"])))
	await _drag(p0, p0 + Vector2(60.0, 25.0))
	var g: Dictionary = _bridge.auto_label_get(key)
	_check("G4 a drag on the selected generated label moves it (override stored)", absf(float(g.get("x", x0)) - x0) > 1.0 and bool(g.get("edited", false)), str(g.get("x")))
	_check("G4b ... and the map did not pan", _cam().distance_to(cam0) < 1.0, str(_cam() - cam0))
	await _shot("auto_2_moved")
	# size slider
	var s0 := float(_bridge.auto_label_get(key).get("size", 0.0))
	var slider: HSlider = null
	for n in (_carto._label_edit_body as Node).find_children("*", "HSlider", true, false):
		slider = n as HSlider
	_check("G5 the form has a size slider", slider != null)
	if slider != null:
		if DccTheme.is_phone():
			_app.set_phone_detent("half")
			await _ctx.wait(0.6)
		await _reveal(slider)
		var r := DriveCtx.rect_of(slider)
		_check("G5a the slider is on screen", Rect2(Vector2.ZERO, _vp.get_visible_rect().size).encloses(r))
		_check("G5b ... and at least 44 dp tall to grab", not DccTheme.is_touch() or r.size.y >= 44.0 * _unit() - 1.0, str(r.size))
		await _drag(r.position + Vector2(r.size.x * 0.3, r.size.y / 2.0), r.position + Vector2(r.size.x * 0.8, r.size.y / 2.0))
		var s1 := float(_bridge.auto_label_get(key).get("size", 0.0))
		_check("G5c dragging the slider resizes the generated label", absf(s1 - s0) > 0.5, "%f -> %f" % [s0, s1])
	# hide
	var hb := _on_screen("AutoLabelHide") as Button
	_check("G6 the Hide button is on screen", hb != null)
	if hb != null:
		_check("G6b ... at least 44 dp across", _min_tap_ok(hb), str(DriveCtx.rect_of(hb).size))
		await _tap_ctrl(hb)
		_check("G6c tapping Hide hides the generated label", bool(_bridge.auto_label_get(key).get("hidden", false)))
	await _shot("auto_3_hidden")
	# restore from the changed list
	var back: Button = null
	for b in _app.find_children("*", "Button", true, false):
		if (b as Button).text == "↺" and (b as Button).is_visible_in_tree():
			back = b
	_check("G7 the changed-labels list offers a restore", back != null)
	if back != null:
		await _reveal(back)
		_check("G7b ... at least 44 dp across", _min_tap_ok(back), str(DriveCtx.rect_of(back).size))
		await _tap_ctrl(back)
		_check("G7c tapping it restores the generator's own label", not bool(_bridge.auto_label_get(key).get("edited", true)))
	await _shot("auto_4_restored")
	_check("positive control: the name read back is the generator's again", String(_bridge.auto_label_get(key).get("text", "")) == name0)
