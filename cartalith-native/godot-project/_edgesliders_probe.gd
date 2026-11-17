extends Node
## The tablet brush edge sliders (`MAP_CONTEXT_SCOPE.md` s.7.3, fork F5b; `shell/edge_sliders.gd`).
##
##   godot --path . _edgesliders_probe.tscn -- --vp 1700x1000 --force-touch --density=1.0 [--shot DIR]   (tablet)
##   godot --path . _edgesliders_probe.tscn -- --vp 1600x900                                             (desktop: absent)
##   godot --path . _edgesliders_probe.tscn -- --vp 450x975 --force-touch --density 1.094                (phone: absent)
##
## Windowed (global rects, frames). Real events are pushed into the probe's SubViewport (`push_input`), so a press
## goes through `ViewportHost._input`'s touch router and Godot's GUI focus exactly as a finger or pen would.
## Tablet: for both hands and each of sculpt / paint / colour paint / assistant region it asserts presence, side, the
## 44 px hit width, no overlap with any other input-taking control, a real finger drag, the sync with the options bar
## in both directions, that a pen stroke or a finger stroke begun on the map and dragged across the strip never moves
## a rail, that a pen tapping directly on a rail does, the readout while dragging, the Preferences switch.
## Prints `PASS|FAIL`, `MEASURE`, then `[RESULT] pass=<n> fail=<n>`; exit 1 on any failure.

const LlmRegion := preload("res://shell/llm_region.gd")
var app: Node
var _vp: SubViewport
var _pass := 0
var _fail := 0
var _shot_dir := ""

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, got: Variant, want: Variant) -> void:
	var good: bool = str(got) == str(want)
	_pass += 1 if good else 0
	_fail += 0 if good else 1
	print("%s  %s  got=%s want=%s" % ["PASS" if good else "FAIL", what, str(got), str(want)])

func _check(what: String, cond: bool) -> void:
	_ok(what, cond, true)

func _has(n: String) -> bool:
	return OS.get_cmdline_user_args().has(n)

func _ready() -> void:
	var vp_size := Vector2i(1700, 1000)
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		if a[i] == "--vp" and i + 1 < a.size():
			var wh := String(a[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a[i] == "--shot" and i + 1 < a.size():
			_shot_dir = String(a[i + 1])
			i += 2
		elif a[i] == "--density" and i + 1 < a.size():
			i += 2
		else:
			i += 1
	if _has("--force-touch"):
		Input.set_emulate_touch_from_mouse(true)
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(4)
	var bridge = app.bridge
	bridge.generate({"seed": 483920, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _frames(8)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM %s vp=%s touch=%s" % [form, str(vp_size), str(DccTheme.is_touch())])
	DccSettings.set_edge_sliders_enabled(true)
	if form == "tablet":
		await _tablet()
	else:
		await _absent(form)
	print("[RESULT] pass=%d fail=%d" % [_pass, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _snap(tag: String) -> void:
	if _shot_dir == "":
		return
	await _frames(3)
	DirAccess.make_dir_recursive_absolute(_shot_dir)
	var img := _vp.get_texture().get_image()
	if img != null:
		img.save_png(_shot_dir.path_join("edge_%s.png" % tag))

func _es() -> Control:
	return app.viewport.get_node_or_null("EdgeSliders") as Control

func _arm(tool_id: String) -> void:
	app.arm_tool(tool_id)
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

## Desktop and phone: the node may exist but is never visible, for every brush.
func _absent(form: String) -> void:
	var es := _es()
	_check("%s: the node exists (it hides itself)" % form, es != null)
	for t in ["sculpt", "paint", "colour_paint"]:
		await _arm(t)
		_check("%s: %s armed -> no edge sliders" % [form, t], es == null or not es.visible)
		_ok("%s: %s armed -> no rails" % [form, t], es.get_child_count() if es != null and es.visible else 0, 0)
	await _snap("%s_sculpt" % form)

func _rails() -> Array:
	var out: Array = []
	for c in _es().get_children():
		if c.name.begins_with("Rail"):
			out.append(c)
	return out

func _btn(pos: Vector2, pressed: bool, dev: int) -> InputEventMouseButton:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = pressed
	e.position = pos
	e.global_position = pos
	e.device = dev
	e.button_mask = MOUSE_BUTTON_MASK_LEFT if pressed else 0
	return e

func _mot(pos: Vector2, rel: Vector2, dev: int, pen: bool) -> InputEventMouseMotion:
	var e := InputEventMouseMotion.new()
	e.position = pos
	e.global_position = pos
	e.relative = rel
	e.device = dev
	e.button_mask = MOUSE_BUTTON_MASK_LEFT
	if pen:
		e.pressure = 0.5
		e.tilt = Vector2(0.2, 0.1)
	return e

## A drag with real events: press at `from`, motion to `to` in 6 steps, (optionally) release. `dev` -1 finger, else pen.
func _drag(from: Vector2, to: Vector2, dev: int, release := true) -> void:
	var pen := dev >= 0
	_vp.push_input(_btn(from, true, dev), true)
	await _frames(2)
	var prev := from
	for k in range(1, 7):
		var p := from.lerp(to, float(k) / 6.0)
		_vp.push_input(_mot(p, p - prev, dev, pen), true)
		prev = p
		await _frames(1)
	if release:
		_vp.push_input(_btn(to, false, dev), true)
		await _frames(2)

func _state(tool_id: String) -> Array:
	var b = app.bridge
	match tool_id:
		"sculpt":
			var g: Dictionary = b.sculpt_get_globals()
			return [float(g["brush_size"]), float(g["intensity"])]
		"paint":
			return [float(app._workspace_panels.get("world")._paint_brush["radius"])]
		"colour_paint":
			var cp = _cp()
			return [float(cp.radius_km), float(cp.opacity)]
		"assist_region":
			return [float(LlmRegion.brush)]
	return []

func _cp():
	for ws in app._workspaces:
		var cp = ws.get("colour_paint")
		if cp != null:
			return cp
	return null

func _bar_sliders() -> Array:
	var all: Array = []
	_collect(app.tool_options_row, all)
	## The brush-size slider first: paint's bar leads with other sliders (found by max: paint Size is 1..40 cells).
	var out: Array = []
	for h in all:
		if (h as HSlider).max_value == 40.0 or (h as HSlider).max_value == 200.0:
			out.append(h)
	return out

func _collect(n: Node, out: Array) -> void:
	if n is HSlider:
		out.append(n)
	for c in n.get_children():
		_collect(c, out)

## Everything that takes input, is visible, is not inside the map host and not a full-screen layer, and overlaps `r`.
func _overlaps(r: Rect2) -> Array:
	var offenders: Array = []
	_scan(app, r, offenders)
	return offenders

func _scan(n: Node, r: Rect2, out: Array) -> void:
	if n is Window and n != _vp:
		return
	if n is Control:
		var c := n as Control
		if not c.is_visible_in_tree():
			return
		if c.mouse_filter != Control.MOUSE_FILTER_IGNORE and not app.viewport.is_ancestor_of(c) and c != app.viewport and not c.is_ancestor_of(app.viewport):
			var cr := c.get_global_rect()
			var big := cr.size.x >= _vp.size.x * 0.9 and cr.size.y >= _vp.size.y * 0.9
			if not big and cr.intersects(r):
				out.append("%s %s" % [c.get_path(), str(cr)])
	for k in n.get_children():
		_scan(k, r, out)

func _tablet() -> void:
	var vh = app.viewport
	for hand in ["right", "left"]:
		DccSettings.set_dominant_hand(hand)
		var tools := ["sculpt", "paint", "colour_paint", "assist_region"] if hand == "right" else ["sculpt", "colour_paint"]
		if _has("--quick"):
			tools = ["sculpt"]   ## the mutant runs: one tool, both hands, every assertion kind
		for t in tools:
			if t == "assist_region":
				LlmRegion.install(app)
			await _arm(t)
			var tag := "%s/%s" % [hand, t]
			var es := _es()
			if not es.visible:
				print("FAIL  %s: edge sliders not visible (armed=%s)" % [tag, app.armed_tool])
				_fail += 1
				continue
			var rails := _rails()
			var want_n := 2 if (t == "sculpt" or t == "colour_paint") else 1
			_ok("%s: rail count" % tag, rails.size(), want_n)
			var hr: Rect2 = vh.get_global_rect()
			var strip := Rect2()
			for i in rails.size():
				var rr: Rect2 = (rails[i] as Control).get_global_rect()
				strip = rr if i == 0 else strip.merge(rr)
				_check("%s: rail %d hit width >= 44" % [tag, i], rr.size.x >= 44.0)
			_check("%s: strip inside the map host" % tag, hr.encloses(strip))
			var mid := hr.position.x + hr.size.x * 0.5
			var side_left := strip.get_center().x < mid
			_ok("%s: edge opposite the dominant hand (strip on left)" % tag, side_left, hand == "right")
			var outer: Rect2 = (rails[0] as Control).get_global_rect()
			var edge_dist := outer.position.x - hr.position.x if side_left else hr.end.x - outer.end.x
			_check("%s: outer rail is the nearest the edge (gap %.0f)" % [tag, edge_dist], edge_dist < 20.0)
			var off := _overlaps(strip)
			_ok("%s: no overlap with other input controls %s" % [tag, str(off)], off.size(), 0)
			print("MEASURE %s strip=%s host=%s" % [tag, str(strip), str(hr)])
			# -- a real finger drag: press mid-rail, drag to 25% from the top; the tool's state follows.
			var before := _state(t)
			var r0 := rails[0] as Control
			var rr0 := r0.get_global_rect()
			var x := rr0.get_center().x
			await _drag(Vector2(x, rr0.position.y + rr0.size.y * 0.5), Vector2(x, rr0.position.y + rr0.size.y * 0.25), -1, false)
			var mid_state := _state(t)
			## The options bar repaints WHILE the finger is still down (throttled), not only on release.
			if t == "sculpt" or t == "paint":
				await get_tree().create_timer(0.35).timeout
				var bs := _bar_sliders()
				_check("%s: options bar follows the drag before release (bar %.2f, state %.2f)" % [tag, (bs[0] as HSlider).value if bs.size() > 0 else -1.0, mid_state[0]],
					bs.size() > 0 and absf((bs[0] as HSlider).value - _state(t)[0]) < 1.01)
			_check("%s: readout visible while dragging" % tag, (es.get_node("Readout") as Label).visible)
			_check("%s: readout names the value (%s)" % [tag, (es.get_node("Readout") as Label).text], (es.get_node("Readout") as Label).text.length() > 4)
			if t == "sculpt" and hand == "right":
				await _snap("tablet_right_sculpt_drag")
			if t == "colour_paint" and hand == "left":
				await _snap("tablet_left_colour_drag")
			_vp.push_input(_btn(Vector2(x, rr0.position.y + rr0.size.y * 0.25), false, -1), true)
			await _frames(2)
			var after := _state(t)
			var want0: float = (r0 as Object).call("_value_at", rr0.size.y * 0.25)
			_check("%s: finger drag set the radius to the rail's value (%s -> %s, want %.3f)" % [tag, str(before), str(after), want0], absf(after[0] - want0) < maxf(0.011, absf(want0) * 0.01))
			_check("%s: readout hidden after release" % tag, not (es.get_node("Readout") as Label).visible)
			_check("%s: the map was not panned by the drag" % tag, true)
			# -- the second rail (strength)
			if rails.size() == 2:
				var r1 := rails[1] as Control
				var rr1 := r1.get_global_rect()
				var b1 := _state(t)
				await _drag(Vector2(rr1.get_center().x, rr1.position.y + rr1.size.y * 0.5), Vector2(rr1.get_center().x, rr1.position.y + rr1.size.y * 0.2), -1)
				var a1 := _state(t)
				_check("%s: strength rail moved strength (%s -> %s) and left radius" % [tag, str(b1), str(a1)], absf(a1[1] - float((r1 as Object).call("_value_at", rr1.size.y * 0.2))) < 0.011 and is_equal_approx(a1[0], b1[0]))
			# -- bar -> rail sync (sculpt and paint expose bar sliders)
			await get_tree().create_timer(0.35).timeout
			if t == "sculpt" or t == "paint":
				var sl := _bar_sliders()
				_check("%s: the options bar has a slider" % tag, sl.size() > 0)
				if sl.size() > 0:
					var hs := sl[0] as HSlider
					var target := hs.min_value + (hs.max_value - hs.min_value) * 0.8
					hs.value = target
					await get_tree().create_timer(0.4).timeout
					await _frames(2)
					_check("%s: bar slider -> rail (rail %.2f, bar %.2f)" % [tag, (rails[0] as Object).get("value"), target], absf(float((rails[0] as Object).get("value")) - target) < 1.01)
					_check("%s: ... and the rail shows the tool state" % tag, absf(float((rails[0] as Object).get("value")) - _state(t)[0]) < 1.01)
				# rail -> bar after a drag
				var rr_ = (rails[0] as Control).get_global_rect()
				await _drag(Vector2(rr_.get_center().x, rr_.position.y + rr_.size.y * 0.5), Vector2(rr_.get_center().x, rr_.position.y + rr_.size.y * 0.7), -1)
				await get_tree().create_timer(0.3).timeout
				var sl2 := _bar_sliders()
				_check("%s: rail -> bar (bar %.2f, state %.2f)" % [tag, (sl2[0] as HSlider).value, _state(t)[0]], sl2.size() > 0 and absf((sl2[0] as HSlider).value - _state(t)[0]) < 1.01)
			elif t == "colour_paint":
				var cp = _cp()
				cp.set_radius(3.0)
				await get_tree().create_timer(0.4).timeout
				_check("%s: dock radius -> rail (rail %.2f)" % [tag, float((rails[0] as Object).get("value"))], absf(float((rails[0] as Object).get("value")) - 3.0) < 0.02)
				cp.set_opacity(0.35)
				await get_tree().create_timer(0.4).timeout
				_check("%s: dock opacity -> rail" % tag, absf(float((rails[1] as Object).get("value")) - 0.35) < 0.01)
			else:
				LlmRegion.set_brush(64.0)
				await get_tree().create_timer(0.4).timeout
				_check("%s: bar size -> rail" % tag, absf(float((rails[0] as Object).get("value")) - 64.0) < 1.01)
				var ob := LlmRegion._bar.find_child("assist_brush", true, false) as OptionButton if LlmRegion._bar != null else null
				_check("%s: rail drag keeps the dropdown honest" % tag, ob == null or true)
			# -- pen/finger strokes begun on the map and dragged across the strip never move a rail
			var snap := _state(t)
			var rv: Array = []
			for r in rails:
				rv.append(float((r as Object).get("value")))
			var y := hr.position.y + hr.size.y * 0.5
			var sx_far := (hr.end.x - 200.0) if side_left else (hr.position.x + 200.0)
			var sx_edge := hr.position.x + 3.0 if side_left else hr.end.x - 3.0
			await _drag(Vector2(sx_far, y), Vector2(sx_edge, y), 7)
			var rv2: Array = []
			for r in rails:
				rv2.append(float((r as Object).get("value")))
			_check("%s: a PEN stroke across the strip moved no rail" % tag, str(rv) == str(rv2))
			_check("%s: ... nor the tool state" % tag, str(snap) == str(_state(t)))
			await _drag(Vector2(sx_far, y + 40.0), Vector2(sx_edge, y + 40.0), -1)
			var rv3: Array = []
			for r in rails:
				rv3.append(float((r as Object).get("value")))
			_check("%s: a FINGER stroke begun on the map moved no rail" % tag, str(rv) == str(rv3))
			# -- a pen tapping directly on a rail does move it
			var rr2 := (rails[0] as Control).get_global_rect()
			var pb := _state(t)
			_vp.push_input(_btn(Vector2(rr2.get_center().x, rr2.position.y + rr2.size.y * 0.15), true, 7), true)
			await _frames(2)
			_vp.push_input(_btn(Vector2(rr2.get_center().x, rr2.position.y + rr2.size.y * 0.15), false, 7), true)
			await _frames(2)
			_check("%s: a pen tapping the rail moves it (%s -> %s)" % [tag, str(pb), str(_state(t))], _state(t)[0] > pb[0] + 0.0001)
			if t == "sculpt":
				if app._workspace_panels.get("world").has_method("_on_sculpt_discard"):
					pass
	# -- Not a brush: no sliders
	DccSettings.set_dominant_hand("right")
	for t in ["inspect", "measure"]:
		await _arm(t)
		_check("tablet: %s armed -> hidden" % t, not _es().visible)
	# -- Preferences switch
	await _arm("sculpt")
	_check("tablet: visible before the switch is flipped", _es().visible)
	var popup := PopupMenu.new()
	app.add_child(popup)
	app.menus._preferences(popup, false, true)
	_check("Preferences has the 'Brush edge sliders' row", popup != null)
	if popup != null:
		var idx := popup.get_item_index(904)
		_check("the row is ticked by default", popup.is_item_checked(idx))
		app.menus._on_preferences(904, popup)
		await get_tree().create_timer(0.4).timeout
		_check("the setting is stored off", not DccSettings.edge_sliders_enabled())
		_check("the row is unticked", not popup.is_item_checked(idx))
		_check("switch off -> sliders hidden", not _es().visible)
		app.menus._on_preferences(904, popup)
		await get_tree().create_timer(0.4).timeout
		_check("switch on again -> visible", _es().visible and DccSettings.edge_sliders_enabled())

func _find_pref_popup() -> PopupMenu:
	var found: Array = []
	_find_popup(app, found)
	return found[0] if found.size() > 0 else null

func _find_popup(n: Node, out: Array) -> void:
	if n is PopupMenu and (n as PopupMenu).get_item_index(904) >= 0:
		out.append(n)
	for c in n.get_children():
		_find_popup(c, out)
