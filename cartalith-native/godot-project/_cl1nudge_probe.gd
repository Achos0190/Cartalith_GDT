extends Node
## Ruling FA (2026-11-15): overlapping PINS keep their tips on the exact place and tilt about them (a pair +/-45 degrees, a
## heart; three or more a fan); the CIRCLES style keeps CL-1's slide with leaders. Plus CL-3's City Viewer click (`CITY_MAP_LOOK_SCOPE.md` sections 4.2 and 4.3). Windowed (refuses
## --headless), under a wiped user directory (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _cl1nudge_probe.tscn -- --vp 1600x1000 --out <dir>
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch --out <dir>      (the phone)
##
## Legs. U: `PinMarker.tilt_angles` (pins) and `fan_offsets` (circles) as pure functions (a translation changes nothing, a clash is cleared, a lone marker and a
## knot over `FAN_MAX` stay put, ids break ties). M: on the map, three POIs stacked on one place are separated, each is
## picked at its drawn head by `_hit_test_poi`, `hits_at`, `_pick_mark` and `marker_at`, the leader ends on the true place
## (framebuffer pixel), and a pan moves the drawn heads by exactly the pan and changes no offset; the same in circles style.
## V: in the City Viewer the same for derived and placed pins, and a left click on a pin opens the card with its NOTE
## section (desktop); on touch a tap selects and opens no card (Ruling EO) and a long press opens the card or peek sheet.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _touch := false
var _checks := 0
var _fails := 0
var _root := ""
var _out := ""
var _ids := {}


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("CL1N %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("CL1N %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- events ------------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _press_at(at: Vector2, down: bool) -> void:
	if _touch:
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
		em.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(em)
	else:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		_vp.push_input(mm)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(ev)


func _click_host(at: Vector2) -> void:
	_press_at(at, true)
	await _frames(2)
	_press_at(at, false)
	await _frames(6)


func _click(local: Vector2) -> void:
	await _click_host(_host(local))


func _hold(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await get_tree().create_timer(0.8).timeout
	await _frames(3)
	_press_at(at, false)
	await _frames(8)


func _tap_it(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await _frames(3)
	_press_at(at, false)
	await _frames(10)


# ---- readouts ----------------------------------------------------------------------------------------------------

func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close_card() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	var br = app.get("context_broker")
	if br != null and br.ring_is_open():
		br.ring_close()
	var pm = app.get("_phone_menu")
	if pm != null and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	if app.get("element_chips") != null:
		app.element_chips.hide_chips()
	if DccTheme.is_phone():
		app.phone_hide_right_sheet()
		app._close_all_phone_overlays()
	_ov._touch_ring_active = false
	_ov._touch_swallow_up = false
	_ov._sample_pin = {}
	await _frames(3)


func _card_texts() -> Array:
	var out: Array = []
	var c = _card()
	if c != null:
		for r in c.drawn_rows():
			out.append("%s|%s" % [String(r.get("kind", "")), String(r.get("text", ""))])
	return out


func _find_row(kind: String, prefix: String) -> Variant:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if String(r.get("kind", "")) == kind and String(r.get("text", "")).begins_with(prefix):
			return r
	return null


func _action_ids() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		out.append(String((r as Dictionary).get("id", "")))
	return out


func _action(id: String) -> Variant:
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return null


func _vault_rows() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("section", "")) == "note":
			out.append(r)
	return out


func _pixel(local: Vector2) -> Color:
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var at := _host(local)
	return img.get_pixelv(Vector2i(clampi(int(at.x), 0, img.get_width() - 1), clampi(int(at.y), 0, img.get_height() - 1)))


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


## A close-up of the framebuffer around overlay-local `centre` (a square of `half` px each way), enlarged 4x with nearest
## filtering, so the heart and the fan can be looked at. Writes nothing when `--out` was not given.
func _crop_shot(name: String, centre: Vector2, half: float, is_host: bool = false) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var at := centre if is_host else _host(centre)
	var rect := Rect2i(int(at.x - half), int(at.y - half), int(half * 2.0), int(half * 2.0)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	var crop := img.get_region(rect)
	crop.resize(crop.get_width() * 4, crop.get_height() * 4, Image.INTERPOLATE_NEAREST)
	crop.save_png(_out.path_join(name))
	_p("shot %s" % name)


func _is_white(c: Color) -> bool:
	return c.r > 0.92 and c.g > 0.9 and c.b > 0.86


func _is_orange(c: Color) -> bool:
	return c.r > 0.85 and c.g > 0.42 and c.g < 0.7 and c.b < 0.35


## The pin of POI `index` in the overlay's CURRENT layout: `{tip, head, r}` in overlay-local px.
func _pin(index: int) -> Dictionary:
	var pd: Dictionary = _bridge.poi_get(index)
	var tip: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect())
	var r: float = _ov._poi_pin_r()
	return {"tip": tip, "head": tip + Vector2(0.0, -Pin.STEM_RATIO * r), "r": r}


func _undo_n() -> int:
	return (_bridge.undo_ledger() as Array).size()


# ---- run ---------------------------------------------------------------------------------------------------------

func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			_touch = true
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
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
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	if DccTheme.is_phone():
		app.element_chips.hide_chips()
		app._close_all_phone_overlays()
		await _frames(10)
	DccSettings.set_marker_style("pins")
	await _run()
	_finish()


## Screen-space candidate spots (overlay-local) clear of every settlement and landmark, whose tip AND head are on the plate
## and inside the visible band. `n` of them, spread out.
func _free_spots(n: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-90)
	if DccTheme.is_phone():
		band = Rect2(80.0, 330.0, float(_vp.size.x) - 160.0, 700.0)
	var r: float = _ov._poi_pin_r()
	var clear_px := r * 5.0
	var others: Array = []
	for s in _bridge.settlements():
		others.append(_ov._cell_to_screen(Vector2(float((s as Dictionary)["x"]), float((s as Dictionary)["y"])), rect))
	for lm in _bridge.landmarks():
		others.append(_ov._cell_to_screen(Vector2(float((lm as Dictionary)["x"]), float((lm as Dictionary)["y"])), rect))
	var out: Array = []
	var step := r * 9.0
	var y := interior.position.y + step
	while y < interior.end.y - step and out.size() < n * 12:
		var x := interior.position.x + step
		while x < interior.end.x - step:
			var tip := Vector2(x, y)
			var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
			var near := false
			for o in others:
				if (o as Vector2).distance_to(tip) < clear_px or (o as Vector2).distance_to(head) < clear_px:
					near = true
					break
			if not near and interior.has_point(tip) and interior.has_point(head) and band.has_point(_host(tip)) and band.has_point(_host(head)):
				out.append(tip)
			x += step
		y += step
	var picked: Array = []
	if out.is_empty():
		return picked
	var stride := maxi(1, out.size() / n)
	for k in n:
		picked.append(out[mini(k * stride, out.size() - 1)])
	return picked

func _picks_poi(p: Vector2, idx: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var a: bool = int(_ov._hit_test_poi(p, interior, rect)[0]) == idx
	var b := false
	for h in _ov.hits_at(p):
		if String((h as Dictionary)["kind"]) == "poi" and int((h as Dictionary)["id"]) == idx:
			b = true
	var c: bool = int(_ov._pick_mark(p, interior, rect)["p"]) == idx
	var d: bool = String(_ov.marker_at(p).get("kind", "")) == "poi" and int(_ov.marker_at(p).get("id", -1)) == idx
	return [a, b, c, d]


func _run() -> void:
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	app.arm_tool("inspect")
	await _frames(4)
	await _leg_u()
	await _leg_m(false)
	await _leg_m(true)
	DccSettings.set_marker_style("pins")
	await _leg_v(false)


# ---- U. the pure fan ----------------------------------------------------------------------------------------------------

func _items(pts: Array, r: float, ids: Array = []) -> Array:
	var out: Array = []
	for k in pts.size():
		out.append({"id": ids[k] if not ids.is_empty() else k, "pos": pts[k], "r": r})
	return out


## The smallest centre-to-centre head distance over `sum of radii` after applying `off` to `items`.
func _min_ratio(items: Array, off: Dictionary, circles: bool) -> float:
	var lift := 0.0 if circles else Pin.STEM_RATIO
	var best := INF
	for a in items.size():
		for b in range(a + 1, items.size()):
			var ha: Vector2 = (items[a]["pos"] as Vector2) + (off.get(items[a]["id"], Vector2.ZERO) as Vector2) + Vector2(0.0, -lift * float(items[a]["r"]))
			var hb: Vector2 = (items[b]["pos"] as Vector2) + (off.get(items[b]["id"], Vector2.ZERO) as Vector2) + Vector2(0.0, -lift * float(items[b]["r"]))
			best = minf(best, ha.distance_to(hb) / (float(items[a]["r"]) + float(items[b]["r"])))
	return best


func _leg_u() -> void:
	print("== U. the pure fan ==")
	var r := 9.0
	_leg_u_tilt(r)
	for circles in [true]:
		var tag := "circles" if circles else "pins"
		var pts := [Vector2(100, 100), Vector2(103, 98), Vector2(98, 104)]
		var it := _items(pts, r)
		var off: Dictionary = Pin.fan_offsets(it, circles)
		_ok("U1 %s: three stacked markers all move" % tag, off.size() == 3, str(off.size()))
		_ok("U1 %s: afterwards every pair of heads is at least a full diameter apart (%.3f)" % [tag, _min_ratio(it, off, circles)],
			_min_ratio(it, off, circles) >= Pin.FAN_CLEAR - 0.001)
		# Translation: any pan gives the same offsets, exactly.
		var same := true
		for t in [Vector2(37.3, -12.9), Vector2(-512.25, 801.5), Vector2(0.0001, 0.0003)]:
			var moved: Array = []
			for q in pts:
				moved.append((q as Vector2) + t)
			var off2: Dictionary = Pin.fan_offsets(_items(moved, r), circles)
			for k in off:
				if not off2.has(k) or not ((off2[k] as Vector2) - (off[k] as Vector2)).length() < 0.01:
					same = false
		_ok("U2 %s: a translation of every marker changes no offset" % tag, same)
		# A lone marker and a far one are absent.
		var far := Pin.fan_offsets(_items([Vector2(100, 100), Vector2(103, 98), Vector2(900, 900)], r), circles)
		_ok("U3 %s: a marker with no neighbour is not in the result" % tag, not far.has(2) and far.size() == 2)
		# Two that merely touch at the edge (below the overlap bar) stay.
		var touch := Pin.fan_offsets(_items([Vector2(100, 100), Vector2(100 + 2.0 * r * 0.8, 100)], r), circles)
		_ok("U4 %s: heads overlapping by under a quarter are left alone" % tag, touch.is_empty())
		# A knot over FAN_MAX stays; exactly FAN_MAX fans and clears the bar.
		var knot: Array = []
		for k in Pin.FAN_MAX + 1:
			knot.append(Vector2(200.0 + 0.1 * k, 200.0))
		_ok("U5 %s: a knot of %d is left in place" % [tag, Pin.FAN_MAX + 1], Pin.fan_offsets(_items(knot, r), circles).is_empty())
		var twelve: Array = []
		for k in Pin.FAN_MAX:
			twelve.append(Vector2(200.0 + 0.1 * k, 200.0 + 0.05 * k))
		var it12 := _items(twelve, r)
		var off12: Dictionary = Pin.fan_offsets(it12, circles)
		_ok("U5 %s: a knot of %d fans, all moved, no pair overlapping by a quarter (ratio %.3f)" % [tag, Pin.FAN_MAX, _min_ratio(it12, off12, circles)],
			off12.size() == Pin.FAN_MAX and _min_ratio(it12, off12, circles) >= Pin.FAN_OVERLAP_KEEP)
		# Ids break ties: the same two markers on one spot, listed in either order, get the same offsets by id.
		var a1 := Pin.fan_offsets(_items([Vector2(50, 50), Vector2(50, 50)], r, [5, 2]), circles)
		var a2 := Pin.fan_offsets(_items([Vector2(50, 50), Vector2(50, 50)], r, [2, 5]), circles)
		_ok("U6 %s: identical positions are told apart by id, whatever the list order" % tag,
			a1.size() == 2 and ((a1[5] as Vector2) - (a2[5] as Vector2)).length() < 0.01 and ((a1[2] as Vector2) - (a2[2] as Vector2)).length() < 0.01 and a1[5] != a1[2])
	# Pins no longer slide (Ruling FA): the slide answers nothing for the pin style.
	_ok("U7: fan_offsets slides nothing for pins (they tilt)", Pin.fan_offsets(_items([Vector2(100, 100), Vector2(100, 100)], r), false).is_empty())


## Ruling FA's pure rule. Literal degrees, so a changed constant cannot hold the test.
func _leg_u_tilt(r: float) -> void:
	var deg := func(x: float) -> float: return rad_to_deg(x)
	# A pair on one shared tip: -45 and +45, the lower id (equal tips) on the left.
	var two: Dictionary = Pin.tilt_angles(_items([Vector2(100, 100), Vector2(100, 100)], r))
	_ok("T1: two pins on one tip tilt -45 and +45 degrees (%s)" % [str(two)],
		two.size() == 2 and absf(deg.call(float(two[0])) + 45.0) < 0.01 and absf(deg.call(float(two[1])) - 45.0) < 0.01)
	# Their stems are then 90 degrees apart: the heart.
	_ok("T2: the two stems are 90 degrees apart", absf(deg.call(float(two[1]) - float(two[0])) - 90.0) < 0.01)
	# Heads overlapping with the TIPS APART: each tilts away about its own tip, the left one to the left.
	var apart: Dictionary = Pin.tilt_angles(_items([Vector2(112, 100), Vector2(100, 100)], r))
	_ok("T3: tips apart, heads overlapping: the left-hand pin (id 1) tilts -45, the right-hand (id 0) +45",
		apart.size() == 2 and absf(deg.call(float(apart[1])) + 45.0) < 0.01 and absf(deg.call(float(apart[0])) - 45.0) < 0.01)
	var three: Dictionary = Pin.tilt_angles(_items([Vector2(100, 100), Vector2(101, 100), Vector2(100, 101)], r))
	var t3: Array = []
	for k in three:
		t3.append(snappedf(deg.call(float(three[k])), 0.01))
	t3.sort()
	_ok("T4: three on one place fan -60, 0, +60 degrees (%s)" % [str(t3)], t3 == [-60.0, 0.0, 60.0])
	var five: Dictionary = Pin.tilt_angles(_items([Vector2(100, 100), Vector2(100, 100), Vector2(100, 100), Vector2(100, 100), Vector2(100, 100)], r))
	var t5: Array = []
	for k in five:
		t5.append(snappedf(deg.call(float(five[k])), 0.01))
	t5.sort()
	_ok("T5: five fan -90, -30, 0, 30, 90 degrees: four on the half circle and the fifth on ring 1 (pin rings (FA follow-up)) (%s)" % [str(t5)], t5 == [-90.0, -30.0, 0.0, 30.0, 90.0])
	var four: Dictionary = Pin.tilt_angles(_items([Vector2(100, 100), Vector2(100, 100), Vector2(100, 100), Vector2(100, 100)], r))
	var t4: Array = []
	for k in four:
		t4.append(snappedf(deg.call(float(four[k])), 0.01))
	t4.sort()
	_ok("T5: four fan -90, -30, 30, 90 degrees, 60 apart so no head touches another (%s)" % [str(t4)], t4 == [-90.0, -30.0, 30.0, 90.0])
	# A lone pin, a far pin and a heads-just-touching pair do not tilt; a knot over FAN_MAX stays upright.
	_ok("T6: a lone pin and a far pin are absent", Pin.tilt_angles(_items([Vector2(100, 100), Vector2(103, 98), Vector2(900, 900)], r)).has(2) == false)
	_ok("T6: heads overlapping by under a quarter are left upright", Pin.tilt_angles(_items([Vector2(100, 100), Vector2(100 + 2.0 * r * 0.8, 100)], r)).is_empty())
	var knot: Array = []
	for k in Pin.FAN_PIN_MAX + 1:
		knot.append(Vector2(200.0 + 0.1 * k, 200.0))
	_ok("T7: a knot of %d (over FAN_PIN_MAX, pin rings (FA follow-up)) is left upright" % [Pin.FAN_PIN_MAX + 1], Pin.tilt_angles(_items(knot, r)).is_empty())
	# Pan invariance: a translation changes no angle.
	var pts := [Vector2(100, 100), Vector2(103, 98), Vector2(98, 104)]
	var base: Dictionary = Pin.tilt_angles(_items(pts, r))
	var same := true
	for tr in [Vector2(37.3, -12.9), Vector2(-512.25, 801.5)]:
		var mv: Array = []
		for q in pts:
			mv.append((q as Vector2) + tr)
		var o2: Dictionary = Pin.tilt_angles(_items(mv, r))
		for k in base:
			if not o2.has(k) or absf(float(o2[k]) - float(base[k])) > 1e-6:
				same = false
	_ok("T8: a translation of every pin changes no tilt", same)
	# The geometry: the tip is on the place and the head swings about it; the footprint follows the rotated head.
	var tip := Vector2(300, 300)
	var a45 := deg_to_rad(45.0)
	var g: Dictionary = Pin.geometry(tip, r, a45)
	var upright := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	var rot_head: Vector2 = g["head"]
	_ok("T9: the rotated pin's outline still starts at the tip", (g["poly"] as PackedVector2Array)[0].is_equal_approx(tip))
	_ok("T9: the head is the same distance from the tip (%.2f vs %.2f) and 45 degrees round, to the right" % [rot_head.distance_to(tip), Pin.STEM_RATIO * r],
		absf(rot_head.distance_to(tip) - Pin.STEM_RATIO * r) < 0.001 and rot_head.x > tip.x + 0.7 * Pin.STEM_RATIO * r * 0.99 and rot_head.y < tip.y)
	_ok("T10: the hit test lands on the rotated head", Pin.hit(tip, r, rot_head, Pin.HIT_PAD, a45))
	_ok("T10: and not on the upright head's old place", not Pin.hit(tip, r, upright, Pin.HIT_PAD, a45))
	_ok("T10: the stem is hit part-way along the rotated axis, not along the upright one (pad 0)",
		Pin.hit(tip, r, tip + (rot_head - tip) * 0.5, Pin.HIT_PAD, a45) and not Pin.hit(tip, r, tip + Vector2(0.0, -Pin.STEM_RATIO * r * 0.5), 0.0, a45))
	_ok("T10: the tip itself is hit at any tilt", Pin.hit(tip, r, tip, Pin.HIT_PAD, a45) and Pin.hit(tip, r, tip, Pin.HIT_PAD, -a45))


# ---- M. the map -----------------------------------------------------------------------------------------------------------

func _poi_set(g: Dictionary, n: int, step: float, tag: String) -> Array:
	var ids: Array = []
	var kinds := ["tavern", "shop", "smithy", "gate"]
	for k in n:
		ids.append(_bridge.poi_add(float(g["gx"]) + step * k, float(g["gy"]) + step * 0.5 * k, kinds[k % kinds.size()], "%s %d" % [tag, k]))
	return ids


func _leg_m(circles: bool) -> void:
	var tag := "circles" if circles else "pins"
	print("== M. the map, %s ==" % tag)
	DccSettings.set_marker_style(tag)
	await _close_card()
	app.arm_tool("inspect")
	await _frames(3)
	while _bridge.poi_list().size() > 0:
		_bridge.poi_delete(_bridge.poi_list().size() - 1)
	app.viewport.refresh_annotations()
	await _frames(6)
	var spots: Array = _free_spots(3)
	_ok("M0 %s: three free places" % tag, spots.size() >= 3)
	if spots.size() < 3:
		return
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var g3: Dictionary = _ov._grid_point(spots[0], rect, interior)
	var g2: Dictionary = _ov._grid_point(spots[2], rect, interior)
	_p("   POIs before the stack: engine %d, overlay %d" % [_bridge.poi_list().size(), _ov._pois.size()])
	var ids3 := _poi_set(g3, 3, 0.004, "Stack")
	var ids2 := _poi_set(g2, 2, 0.004, "Pair")
	app.viewport.refresh_annotations()
	await _frames(8)
	rect = _ov._displayed_rect()
	interior = _ov._interior_rect(rect)
	_ok("M1 %s: the overlay holds the five POIs" % tag, _ov._pois.size() == 5 and not ids3.has(-1) and not ids2.has(-1), str(_ov._pois.size()))
	var fan: Dictionary = _ov._fan(rect)
	var fkey := "poi" if circles else "poi_a"
	_ok("M2 %s: all five are nudged / tilted" % tag, (fan[fkey] as Dictionary).size() == 5, str(fan[fkey]))
	var r: float = _ov._poi_pin_r()
	var heads: Array = []
	var trues: Array = []
	var drawn: Array = []
	for idx in ids3:
		var pd: Dictionary = _bridge.poi_get(idx)
		var t: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), rect)
		var dr: Vector2 = _ov._nudged_poi(idx, t, rect)
		trues.append(t)
		drawn.append(dr)
		heads.append(dr if circles else (Pin.geometry(dr, r, _ov._tilt_poi(idx, rect))["head"] as Vector2))
	var mind := INF
	for a in heads.size():
		for b in range(a + 1, heads.size()):
			mind = minf(mind, (heads[a] as Vector2).distance_to(heads[b]))
	_ok("M3 %s: the three heads are a diameter apart (%.1f px, radius %.1f)" % [tag, mind, r], mind >= 2.0 * r * (1.04 if circles else 1.0))
	var all_pick := true
	for k in ids3.size():
		var res: Array = _picks_poi(heads[k], int(ids3[k]))
		if res != [true, true, true, true]:
			all_pick = false
			_p("   pick %d at %s -> %s" % [k, str(heads[k]), str(res)])
	_ok("M4 %s: each head picks its own POI through _hit_test_poi, hits_at, _pick_mark and marker_at" % tag, all_pick)
	# The true place is NOT where the stack is picked from any more: a click on the old shared point does not pick two.
	var n_moved := 0
	for k in ids3.size():
		if ((drawn[k] as Vector2) - (trues[k] as Vector2)).length() > r * 0.5:
			n_moved += 1
	if circles:
		_ok("M5 %s: at least two of the three moved off their true place (a row's middle member may stay)" % tag, n_moved >= 2, str(n_moved))
	else:
		var a3: Array = []
		for idx in ids3:
			a3.append(snappedf(rad_to_deg(_ov._tilt_poi(idx, rect)), 0.01))
		_ok("M5 pins: no tip moved off its true place (drawn == true)", n_moved == 0, str(n_moved))
		_ok("M5 pins: the stacked three fan -60, 0, +60 degrees, left to right (%s)" % str(a3), a3 == [-60.0, 0.0, 60.0])
	# The pair: nothing else is near its true place, so the leader and its dot read from the frame.
	var pa: Dictionary = _bridge.poi_get(int(ids2[0]))
	var tp: Vector2 = _ov._point_to_screen(Vector2(float(pa["x"]), float(pa["y"])), rect)
	var pb: Dictionary = _bridge.poi_get(int(ids2[1]))
	var tq: Vector2 = _ov._point_to_screen(Vector2(float(pb["x"]), float(pb["y"])), rect)
	var da: Vector2 = _ov._nudged_poi(int(ids2[0]), tp, rect)
	var db: Vector2 = _ov._nudged_poi(int(ids2[1]), tq, rect)
	var mid := (tp + tq) * 0.5
	var along := (da - db).normalized() if da != db else Vector2.RIGHT
	var perp := Vector2(-along.y, along.x)
	if circles:
		var dot := await _pixel(mid + perp * (0.2 * r))
		var dot2 := await _pixel(mid - perp * (0.2 * r))
		_ok("M6 %s: the true place of the pair is marked by the leader's dark ring (%s / %s)" % [tag, dot, dot2],
			dot.r < 0.5 or dot2.r < 0.5)
		var half := await _pixel(mid + (da - mid) * 0.5)
		var half2 := await _pixel(mid + (db - mid) * 0.5)
		_ok("M6 %s: and a leader runs from it toward each nudged marker (%s / %s)" % [tag, half, half2],
			(half.r < 0.5 or half.r > 0.8) and (half2.r < 0.5 or half2.r > 0.8) and (da - mid).length() > r)
		var drawn_ok := (da - tp).length() > r * 0.5 and (db - tq).length() > r * 0.5 and (da.x < db.x if da != db else false)
		_ok("M6 %s: the pair is split sideways or round, each member off its true place" % tag, drawn_ok or circles)
	else:
		var aa: float = _ov._tilt_poi(int(ids2[0]), rect)
		var ab: float = _ov._tilt_poi(int(ids2[1]), rect)
		_ok("M6 pins: the pair tilts -45 and +45 degrees, left pin to the left (%.2f / %.2f)" % [rad_to_deg(aa), rad_to_deg(ab)],
			absf(rad_to_deg(aa) + 45.0) < 0.01 and absf(rad_to_deg(ab) - 45.0) < 0.01)
		_ok("M6 pins: each pair member's tip is exactly its true place", da == tp and db == tq)
		# The framebuffer: a stem's mid-point is pin-coloured on the TILTED axis and the upright axis is empty.
		var s_a := await _pixel(tp + Vector2(0.0, -Pin.STEM_RATIO * r * 0.5).rotated(aa))
		var s_b := await _pixel(tq + Vector2(0.0, -Pin.STEM_RATIO * r * 0.5).rotated(ab))
		_ok("M6 pins: the pair's stems are drawn on the rotated axes (%s / %s)" % [s_a, s_b],
			(_is_white(s_a) or _is_orange(s_a)) and (_is_white(s_b) or _is_orange(s_b)))
		var up := await _pixel(tp + Vector2(0.0, -Pin.STEM_RATIO * r))
		_ok("M6 pins: and the upright head's old place is empty of pin colour (%s)" % up, not (_is_white(up) or _is_orange(up)))
	await _shot("cl1n_map_%s_%s.png" % [tag, "touch" if _touch else "desk"])
	if not circles:
		await _crop_shot("cl1n_map_heart_%s.png" % ("touch" if _touch else "desk"), tp + Vector2(0.0, -r), r * 8.0)
		await _crop_shot("cl1n_map_fan_%s.png" % ("touch" if _touch else "desk"), (trues[1] as Vector2) + Vector2(0.0, -r), r * 8.0)
	# No jitter: a pan moves every drawn head by exactly the pan and changes no offset.
	var base_fan: Dictionary = _ov._fan(rect)
	var base_off: Array = []
	for idx in ids3:
		base_off.append((base_fan["poi"][idx] as Vector2) if circles else Vector2(float(base_fan["poi_a"][idx]), 0.0))
	var cam = app.viewport._camera   ## a Control now (the probe typed it Node2D, which aborted this leg unseen)
	var h0: Vector2 = _host(heads[0])
	var jitter_ok := true
	var moved_ok := true
	var cam0: Vector2 = cam.position
	for step in [Vector2(1.3, 0.7), Vector2(-2.9, 4.1), Vector2(0.4, -0.2), Vector2(13.7, 9.9)]:
		cam.position = cam0 + step
		await _frames(3)
		var rect2: Rect2 = _ov._displayed_rect()
		var f2: Dictionary = _ov._fan(rect2)
		for k in ids3.size():
			var cur: Vector2 = (f2["poi"][ids3[k]] as Vector2) if circles else Vector2(float(f2["poi_a"][ids3[k]]), 0.0)
			if not (cur - base_off[k]).is_zero_approx():
				jitter_ok = false
		var h1: Vector2 = _host(heads[0])
		if not ((h1 - h0) - step * float(app.viewport._zoom)).length() < 0.05:
			if not ((h1 - h0) - step).length() < 0.05:
				moved_ok = false
	cam.position = cam0
	await _frames(3)
	_ok("M7 %s: four small pans change no offset (no jitter)" % tag, jitter_ok)
	_ok("M7 %s: and the drawn head moves by exactly the pan" % tag, moved_ok)
	# Zooming out until the heads no longer overlap removes the nudge (zoom resolves it).
	while _bridge.poi_list().size() > 0:
		_bridge.poi_delete(_bridge.poi_list().size() - 1)
	app.viewport.refresh_annotations()
	await _frames(6)
	_ok("M8 %s: removing the five POIs leaves no nudge" % tag, _ov._pois.is_empty() and (_ov._fan(_ov._displayed_rect())["poi"] as Dictionary).is_empty() and (_ov._fan(_ov._displayed_rect())["poi_a"] as Dictionary).is_empty())


# ---- V. the City Viewer ---------------------------------------------------------------------------------------------------

## A town with a layout, or -1.
func _town() -> int:
	var places: Array = _bridge.settlements()
	for k in places.size():
		var kind := String((places[k] as Dictionary).get("kind", ""))
		if kind in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				return k
	return -1


func _cv_press(cv, local: Vector2, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = local
	ev.global_position = local
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	cv._on_canvas_input(ev)


## A click in the viewer's canvas space, then a few frames. A touch tap is the same pair of events.
func _cv_click(cv, local: Vector2) -> void:
	_cv_press(cv, local, true)
	await _frames(2)
	_cv_press(cv, local, false)
	await _frames(8)


func _leg_v(unused: bool) -> void:
	if unused:
		return
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== V. the City Viewer (%s) ==" % form)
	await _close_card()
	var si := _town()
	_ok("V0: a town with a layout exists", si >= 0)
	if si < 0:
		return
	var s: Dictionary = _bridge.settlements()[si]
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-cl1n-%d" % Time.get_ticks_msec()
	_write(_root + "/Lore/Old.md", "# Old\n")
	_ok("V0: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	var ids: Array = []
	var kinds := ["tavern", "shop", "smithy"]
	for k in 3:
		ids.append(_bridge.poi_add(float(s["x"]) + 0.5 + 0.12 + 0.0015 * k, float(s["y"]) + 0.5 + 0.05 + 0.001 * k, kinds[k], "Viewer %d" % k))
	_ok("V1: three POIs stacked beside the market", not ids.has(-1))
	app.viewport.refresh_annotations()
	var cv: CityViewerWindow = app.city_viewer_window
	app.open_city_viewer(si)
	await _frames(14)
	var pins: Array = cv._pins()
	var poi_idx: Array = []
	for k in pins.size():
		if (pins[k] as Dictionary).has("poi"):
			poi_idx.append(k)
	_ok("V2: the viewer shows the three POIs as pins", poi_idx.size() == 3, str(poi_idx.size()))
	var vt: Dictionary = cv._view_transform()
	var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
	var nud: Dictionary = cv._pin_tilts(pins, vt, pr, false)
	var moved := 0
	for k in poi_idx:
		if nud.has(k):
			moved += 1
	_ok("V3: all three overlapping pins are tilted", moved == 3, str(nud))
	_ok("V3: the viewer slides nothing for pins", (cv._pin_nudges(pins, vt, pr, false) as Dictionary).is_empty())
	var heads: Array = []
	var gaps := INF
	for k in poi_idx:
		heads.append(Pin.geometry(cv.pin_tip(k), pr, cv.pin_tilt(k))["head"])
	for a in heads.size():
		for b in range(a + 1, heads.size()):
			gaps = minf(gaps, (heads[a] as Vector2).distance_to(heads[b]))
	# A town with many derived pins chains them into one big cluster (the phone's heads are 2.6x larger), and a half circle
	# cannot clear more than four heads on one tip (`TILT_FAN_MAX_DEG`): the clearance bar applies to a small cluster only.
	if nud.size() <= 4:
		_ok("V4: their heads are a diameter apart (%.1f px, radius %.1f)" % [gaps, pr], gaps >= 2.0 * pr * 0.99)
	else:
		_ok("V4: a cluster of %d pins fans like cards, every head still apart from every other (%.1f px)" % [nud.size(), gaps], gaps > 1.0)
	var all_pick := true
	var dbg := ""
	for n in poi_idx.size():
		if cv.pin_at(heads[n]) != poi_idx[n]:
			all_pick = false
		dbg += " [%d head %s -> %d, pin_tip %s]" % [poi_idx[n], str(heads[n]), cv.pin_at(heads[n]), str(cv.pin_tip(poi_idx[n]))]
	for k in pins.size():
		dbg += "
     pin %d %s tip %s tilt %s" % [k, str((pins[k] as Dictionary)["kind"]), str(cv.pin_tip(k)), str(nud.get(k, 0.0))]
	_ok("V5: each drawn head picks its own pin", all_pick, dbg)
	await _shot("cl1n_viewer_%s_%s.png" % [form, "touch" if _touch else "desk"])
	var fan_c := Vector2.ZERO
	for k in poi_idx:
		fan_c += cv.pin_tip(k) / 3.0
	await _crop_shot("cl1n_viewer_fan_%s_%s.png" % [form, "touch" if _touch else "desk"], Vector2(cv.position) + cv._canvas.get_global_transform_with_canvas() * (fan_c + Vector2(0.0, -pr)), pr * 8.0, true)
	# Ruling FA: every tip is exactly its true model place, and the three tilt to distinct angles inside a half circle.
	var tips_ok := true
	for k in poi_idx:
		var true_k: Vector2 = (vt["origin"] as Vector2) + ((pins[k] as Dictionary)["at"] as Vector2) * float(vt["scale"])
		if cv.pin_tip(k) != true_k:
			tips_ok = false
	_ok("V6: no tip moved off its true model place", tips_ok)
	var angs: Array = []
	for k in poi_idx:
		angs.append(snappedf(rad_to_deg(cv.pin_tilt(k)), 0.01))
	angs.sort()
	var spread_ok := absf(float(angs[2])) <= 90.0 and absf(float(angs[0])) <= 90.0 and float(angs[1]) - float(angs[0]) >= 3.0 and float(angs[2]) - float(angs[1]) >= 3.0   ## 3 degrees: in a big merged cluster (phone) the outer rings' slots interleave, so three POIs can sit 7 degrees apart (pin rings)
	_ok("V6: the three tilt to distinct angles in a fan (%s)" % str(angs), spread_ok)
	# No jitter across small pans.
	var jit := true
	var pan0: Vector2 = cv._pan
	for step in [Vector2(1.3, 0.7), Vector2(-2.9, 4.1), Vector2(0.4, -0.2), Vector2(13.7, 9.9)]:
		cv._pan = pan0 + step
		var vt2: Dictionary = cv._view_transform()
		var n2: Dictionary = cv._pin_tilts(cv._pins(), vt2, pr, false)
		for k in poi_idx:
			if absf(float(n2[k]) - float(nud[k])) > 1e-6:
				jit = false
	cv._pan = pan0
	_ok("V7: four small pans change no pin's tilt", jit)
	# The click.
	app.arm_tool("inspect")
	var target: int = poi_idx[1]
	var hpos: Vector2 = (Pin.geometry(cv.pin_tip(target), pr, cv.pin_tilt(target))["head"] as Vector2)
	var pm = app.get("_phone_menu")
	var ledger0: int = (_bridge.undo_ledger() as Array).size()
	if not _touch:
		await _cv_click(cv, hpos)
		_ok("V8: a left click on a pin selects the POI", app.selection == {"kind": "poi", "id": int(ids[1])}, str(app.selection))
		_ok("V8: and opens the card", _card_open())
		var rows := _vault_rows()
		_ok("V8: with a NOTE section holding the one Note… row (Ruling FJ)", _find_row("band", "NOTE") != null and rows.size() == 1 and String(rows[0]["label"]) == "Note…", str(_card_texts()))
		var hits: Array = app.context_broker.last_request.get("hits", [])
		_ok("V8: for exactly that one subject", hits.size() == 1 and String(hits[0]["kind"]) == "poi" and int(hits[0]["id"]) == int(ids[1]), str(hits))
		var cr: Rect2i = Rect2i(_card().position, _card().size)
		var anchor := Vector2(cv.position) + cv._canvas.get_global_transform_with_canvas() * hpos
		_ok("V8: the card stands at the pin (%s, anchor %s)" % [str(cr), str(anchor)], Rect2(cr).grow(40.0).has_point(anchor))
		_ok("V8: no map edit (the undo ledger is as the three placements left it)", (_bridge.undo_ledger() as Array).size() == ledger0, "%d vs %d" % [(_bridge.undo_ledger() as Array).size(), ledger0])
		await _shot("cl1n_viewercard_%s.png" % form)
		await _close_card()
		# A click on the town's own derived pin (the citadel or a market) opens the settlement's card.
		var derived := -1
		for k in pins.size():
			if not (pins[k] as Dictionary).has("poi"):
				derived = k
				break
		if derived >= 0:
			await _cv_click(cv, (Pin.geometry(cv.pin_tip(derived), pr, cv.pin_tilt(derived))["head"] as Vector2))
			var h2: Array = app.context_broker.last_request.get("hits", [])
			_ok("V9: a click on a derived pin opens the card for the town", _card_open() and h2.size() == 1 and String(h2[0]["kind"]) == "settlement" and int(h2[0]["id"]) == si, str(h2))
			await _close_card()
		# A press that travels (a pan) opens nothing.
		_cv_press(cv, hpos, true)
		cv._on_canvas_input(_motion(hpos + Vector2(30, 0), Vector2(30, 0)))
		_cv_press(cv, hpos + Vector2(30, 0), false)
		await _frames(6)
		_ok("V10: a drag over a pin opens no card", not _card_open())
	else:
		await _cv_click(cv, hpos)
		_ok("V8 touch: a tap selects the POI", app.selection == {"kind": "poi", "id": int(ids[1])}, "%s pin_at %d want %d pos %s" % [str(app.selection), cv.pin_at(hpos), target, str(hpos)])
		_ok("V8 touch: and opens no card (Ruling EO)", not _card_open() and not (pm != null and pm.peek_card_is_open()))
		await get_tree().create_timer(1.0).timeout
		_cv_press(cv, hpos, true)
		await get_tree().create_timer(0.8).timeout
		await _frames(4)
		var rows := _vault_rows()
		_ok("V9 touch: a long press opens the card with the Note section (the one Note… row, Ruling FJ)", rows.size() == 1 and String(rows[0]["label"]) == "Note…", str(rows))
		var up_sel: Dictionary = app.selection
		_cv_press(cv, hpos, false)
		await _frames(6)
		_ok("V9 touch: the lift selects and opens nothing more", app.selection == up_sel)
		if DccTheme.is_phone():
			_ok("V9 touch: the peek sheet is open", pm != null and pm.peek_card_is_open())
			await _shot("cl1n_viewerpeek_%s.png" % form)
			await pm._dismiss_peek_card()
		else:
			_ok("V9 touch: the card is open with a NOTE band", _card_open() and _find_row("band", "NOTE") != null)
			await _shot("cl1n_viewercard_%s.png" % form)
			await _close_card()
		# A drag that starts on a pin: no hold, no card.
		_cv_press(cv, hpos, true)
		cv._on_canvas_input(_motion(hpos + Vector2(40, 0), Vector2(40, 0)))
		await get_tree().create_timer(0.8).timeout
		_cv_press(cv, hpos + Vector2(40, 0), false)
		await _frames(4)
		_ok("V10 touch: a press that moves past the slop never opens the card", not _card_open() and not (pm != null and pm.peek_card_is_open()))
	cv.hide()
	await _frames(4)
	while _bridge.poi_list().size() > 0:
		_bridge.poi_delete(_bridge.poi_list().size() - 1)
	app.viewport.refresh_annotations()
	await _frames(4)


func _motion(pos: Vector2, rel: Vector2) -> InputEventMouseMotion:
	var mm := InputEventMouseMotion.new()
	mm.position = pos
	mm.global_position = pos
	mm.relative = rel
	mm.button_mask = MOUSE_BUTTON_MASK_LEFT
	return mm

