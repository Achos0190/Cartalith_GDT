extends Node
## Owner report, 2026-09-27: *"The POI markers aren't clickable for more
## information in the right pane."* This probe clicks real markers with REAL
## input -- an InputEventMouseButton (or, with --force-touch, an
## InputEventScreenTouch) pushed into the SubViewport the shell lives in, at the
## marker's drawn screen position -- so everything between the viewport and
## `map_overlay.gd::_gui_input` (siblings covering the map, the LOD tile layer,
## the ring, the fan) gets its chance to eat the event, which a direct
## `_gui_input()` call skips.
##
## What the right pane must show is read from the bridge
## (`bridge.landmarks()`, `bridge.icon_get()`), never from constants here.
##
##   Godot --path . _poiclick_probe.tscn -- [--vp 1600x1000] [--force-touch] [--out DIR]
##
## Windowed (the screenshots are evidence the marker is really drawn there).

const SEED := 552017

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _out := ""
var _touch := false
var _sel_l: Array = []
var _sel_s: Array = []
## The Control a real click on the map goes through, measured on the landmark leg.
var map_ctrl := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("POICLICK %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("POICLICK %s" % s)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	img.save_png(_out.path_join(name))
	_p("shot %s %dx%d" % [name, img.get_width(), img.get_height()])


func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label and (c as Label).is_visible_in_tree():
			out.append((c as Label).text)
		elif c is Button and (c as Button).is_visible_in_tree():
			out.append((c as Button).text)
		_collect(c, out)


## Every visible text in whatever the form calls its "right pane": the right
## dock on desktop/tablet; on the phone, the dock body wherever the phone
## shell has put it, plus the peek sheet.
func _pane_texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	var pm = app.get("_phone_menu")
	if pm != null:
		_collect(pm, out)
	return out


## A real click (or tap) at `local` in the overlay's own space.
func _click(local: Vector2) -> void:
	var ov: Control = app.viewport.overlay
	var at: Vector2 = ov.get_global_transform_with_canvas() * local
	for down in [true, false]:
		if _touch:
			## What a device delivers for one finger: the touch itself, then
			## the mouse event `Input` emulates from it (`DEVICE_ID_EMULATION`,
			## which `map_overlay.gd` reads as "this is a finger"). `push_input`
			## into a SubViewport does no emulation of its own, so both are sent.
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.pressed = down
			t.position = at
			_vp.push_input(t)
			var em := InputEventMouseButton.new()
			em.button_index = MOUSE_BUTTON_LEFT
			em.pressed = down
			em.position = at
			em.global_position = at
			em.device = InputEvent.DEVICE_ID_EMULATION
			_vp.push_input(em)
		else:
			var mm := InputEventMouseMotion.new()
			mm.position = at
			_vp.push_input(mm)
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.pressed = down
			ev.position = at
			ev.global_position = at
			_vp.push_input(ev)
		await _frames(2)
	await _frames(4)


func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
			continue
		if a == "--force-touch":
			_touch = true
			i += 1
			continue
		_p("ABORT unknown argument '%s'" % a)
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(900.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))

	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	if not bridge.has_world:
		_check("a world generated", false)
		_finish()
		return
	await _run(form)
	_finish()


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _run(form: String) -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var r: Dictionary = await bridge.landmark_run()
	await _frames(6)
	var lms: Array = bridge.landmarks()
	_check("L0 landmarks placed and on the overlay", lms.size() > 0 and ov._landmarks.size() == lms.size(),
		"bridge=%d overlay=%d reply_ok=%s" % [lms.size(), ov._landmarks.size(), r.get("ok")])
	## Generated icons too -- every family, so the survey sees what the owner sees.
	for fam in bridge.icon_placement_families():
		var res: Dictionary = bridge.icon_generate({"family": String(fam["key"])})
		_p("icon_generate %s -> placed %s" % [fam["key"], res.get("placed")])
	app.viewport.refresh_annotations()
	await _frames(6)
	var icons: Array = bridge.icon_list()
	var by_family := {}
	for ic: Dictionary in icons:
		var k := "%s/%s" % [ic.get("family"), ic.get("origin")]
		by_family[k] = int(by_family.get(k, 0)) + 1
	_p("SURVEY landmarks=%d icons=%s landmarks_visible=%s" % [lms.size(), by_family, ov._landmarks_visible])
	await _shot("poiclick_%s_fit.png" % form)

	app.viewport.landmark_selected.connect(func(d, idx): _sel_l.append([d, idx]))
	app.viewport.settlement_selected.connect(func(d, idx): _sel_s.append([d, idx]))

	var rect: Rect2 = ov._displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	## The overlay's visible part, in its own local space: a marker under the
	## phone's chrome is not one the user can tap.
	## Checked in the SubViewport's space, after the camera: a mark whose local
	## position is inside the overlay can still be off-screen once zoomed.
	var host_rect: Rect2 = (app.viewport as Control).get_global_rect().grow(-60)
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	_p("host rect %s, overlay xform scale %s" % [host_rect, xf.get_scale()])

	# -- L: a landmark ring ------------------------------------------------------
	var li := -1
	for k in lms.size():
		var lp: Vector2 = ov._cell_to_screen(Vector2(lms[k]["x"], lms[k]["y"]), rect)
		if interior.has_point(lp) and host_rect.has_point(xf * lp) and ov._pick_mark(lp, interior, rect)["l"] == k:
			li = k
			break
	_check("L1 an uncontested, uncovered landmark exists", li >= 0)
	if li >= 0:
		var truth: Dictionary = bridge.landmarks()[li]
		var lp: Vector2 = ov._cell_to_screen(Vector2(truth["x"], truth["y"]), rect)
		_p("L target #%d kind=%s class=%s at local %s" % [li, truth["kind"], truth["class"], lp])
		map_ctrl = await _hovered_at(lp)
		_p("L hovered at target: %s" % map_ctrl)
		_sel_l.clear()
		_sel_s.clear()
		await _click(lp)
		_check("L2 landmark_selected fired for #%d" % li,
			_sel_l.any(func(e): return int(e[1]) == li), "%s" % [_sel_l.map(func(e): return e[1])])
		_check("L3 right dock context is landmark", String(app.right_dock_ctrl._context) == "landmark",
			"context=%s" % app.right_dock_ctrl._context)
		var t := _pane_texts()
		var want := [app.right_dock_ctrl._landmark_label(String(truth["kind"]))]
		for j in Array(truth["causal"]).size():
			want.append("%d. %s" % [j + 1, String(truth["causal"][j])])
		var missing := want.filter(func(w): return not t.has(w))
		_check("L4 the pane shows bridge.landmarks()[%d]'s kind and causal chain (%d strings)" % [li, want.size()],
			missing.is_empty(), "missing=%s" % [missing])
		await _shot("poiclick_%s_landmark.png" % form)

	# -- I: one drawn icon glyph of every family on screen ----------------------
	if form == "phone":
		_check("L5 the phone's right sheet is open after the landmark tap", bool(app._right_sheet_open))
		app._set_sheet_open("right", false)
		await _frames(3)
	var ringed: Dictionary = ov._ringed_cells()
	var picked := {}
	for k in icons.size():
		var ic: Dictionary = icons[k]
		var fam := String(ic["family"])
		if picked.has(fam) or ov._icon_layer_hidden(ic) or ov._icon_shadowed_by_ring(ic, ringed):
			continue
		var ip: Vector2 = ov._point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		if not interior.has_point(ip) or not host_rect.has_point(xf * ip):
			continue
		var pk: Dictionary = ov._pick_mark(ip, interior, rect)
		if pk["s"] != -1 or pk["l"] != -1 or pk.get("i", -2) != k:
			continue
		## Under the same Control the landmark click went through, i.e. not
		## under the phone's app bar or a sheet.
		if await _hovered_at(ip) != map_ctrl:
			continue
		picked[fam] = k
	_p("I one uncontested drawn icon per family: %s" % [picked])
	_check("I0 at least one drawn icon to click", not picked.is_empty())
	for fam in picked:
		var ii: int = picked[fam]
		## The phone's right sheet opened on the previous leg and covers the
		## map, as it should; close it the way the user would before the next tap.
		if form == "phone":
			app._set_sheet_open("right", false)
			await _frames(3)
		var ic: Dictionary = icons[ii]
		var ip: Vector2 = ov._point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		## Start from Sample, so a context left by the previous leg cannot pass I1.
		app.right_dock_ctrl.on_settlement_selected(null, -1)
		await _frames(2)
		_p("I[%s] #%d hovered at target: %s" % [fam, ii, await _hovered_at(ip)])
		await _click(ip)
		var after := String(app.right_dock_ctrl._context)
		_check("I1[%s] clicking icon #%d opens it in the right pane" % [fam, ii], after == "icon", "context=%s" % after)
		var truth_i: Dictionary = bridge.icon_get(ii)
		var t2 := _pane_texts()
		var want_i := [String(truth_i["slot"]).capitalize(),
			"%d, %d" % [floori(float(truth_i["x"])), floori(float(truth_i["y"]))]]
		var miss := want_i.filter(func(w): return not t2.has(w))
		_check("I2[%s] the visible pane shows bridge.icon_get(%d)'s slot and cell" % [fam, ii], miss.is_empty(),
			"missing=%s texts=%s" % [miss, t2.slice(0, 20)])
		await _shot("poiclick_%s_icon_%s.png" % [form, fam])


## Which Control the viewport says is under `local` -- printed, not filtered
## on: a transparent Control over the map that eats the click IS a defect this
## probe must be able to see.
func _hovered_at(local: Vector2) -> String:
	var ov: Control = app.viewport.overlay
	var mm := InputEventMouseMotion.new()
	mm.position = ov.get_global_transform_with_canvas() * local
	_vp.push_input(mm)
	await _frames(2)
	var h: Control = _vp.gui_get_hovered_control()
	return "null" if h == null else "%s(%s)" % [h.name, h.get_class()]
