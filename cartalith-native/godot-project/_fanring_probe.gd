extends Node
## Pin rings, the Ruling FA follow-up (2026-11-16): a fan of more than four pins on ONE place stands on outer rings (a pin on a stalk, `PinMarker.fan_layout`),
## a knot up to `FAN_PIN_MAX` fans instead of staying upright, and two fans that would reach each other merge into one layout.
## Windowed (refuses --headless), under a wiped user directory (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _fanring_probe.tscn -- --vp 1600x1000 --out <dir> [--tag after]
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch --out <dir>      (the phone)
##
## Legs. U: `fan_layout` as a pure function (literal ring lifts and degrees; 2, 3, 4 unchanged from Ruling FA; a knot of
## `FAN_PIN_MAX` fans and one more stays upright; two stacks 4 head radii apart merge and clear). M: on the map, N = 2, 3, 5, 8,
## 12 and 20 POIs on ONE place, then two fans of 12 side by side: the heads of one fan clear one another, each head picks its own
## POI through the four pick paths, every tip is its true place, the stalk is drawn. V: the same in the City Viewer.
## Heads clear = centre distance over the radii sum at least `CLEAR` (1.0 = heads touch, none overlap); between two fans `NEAR`
## (0.75 = overlap under a quarter, `FAN_OVERLAP_KEEP`). Exit 0 = held, 1 = a check failed, 2 = could not run.
## Run it against the commit before pin rings (FA follow-up) (`--tag before`): it only prints the geometry and takes the frames then.

const CLEAR := 1.0
const NEAR := 0.75
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
var _tag := "after"
var _legs := "umv"   ## which legs run: u (pure), m (map), v (viewer); the mutant runs pick the cheapest that sees the mutant
var _quick := false   ## mutant runs: fewer cases (5 and 12 pins, one pair of fans)
var _before := false   ## the code under test has no `fan_layout` (the commit before pin rings (FA follow-up)): geometry only


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("FANR %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("FANR %s" % s)


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
		elif a == "--legs":
			_legs = String(args[i + 1])
			i += 2
		elif a == "--quick":
			_quick = true
			i += 1
		elif a == "--tag":
			_tag = String(args[i + 1])
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
func _free_spots(n: int, clear_r: float = 5.0) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-90)
	if DccTheme.is_phone():
		band = Rect2(80.0, 330.0, float(_vp.size.x) - 160.0, 700.0)
	var r: float = _ov._poi_pin_r()
	var clear_px := r * clear_r   ## the fan probe asks for 12: a fan of 20 reaches 8 radii, and a landmark's own pin under it is a different overlap (see _leg_m)
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




## Host pixels per overlay-local px: 1 on the desktop, the phone's UI multiplier there (a crop sized in local px is too tight otherwise).
func _hs() -> float:
	return maxf(0.01, (_host(Vector2(1.0, 0.0)) - _host(Vector2.ZERO)).x)


func _form() -> String:
	return ("phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")) + ("_touch" if _touch else "_desk")


## The drawn head centre and head radius of a pin whose true tip is `tip`, tilted `ang` and lifted `lift`: the same arithmetic
## `PinMarker.draw` and `hit_styled` use (`lifted_tip`, `shape`), written out so a wrong lift in either cannot hide.
func _head(tip: Vector2, ang: float, lift: float, r: float, kind: String) -> Array:
	var sh := Pin.shape(r, kind, DccSettings.marker_icon_set())
	var rh := float(sh["r"])
	var base := tip + Vector2(sin(ang), -cos(ang)) * lift
	return [base + Vector2(0.0, -float(sh["stem"]) * rh).rotated(ang), rh]


## The smallest centre distance over radii sum among `hs` (`[head, r]` pairs), and over pairs from different groups `grp`.
func _ratios(hs: Array, grp: Array) -> Array:
	var all := INF
	var cross := INF
	for a in hs.size():
		for b in range(a + 1, hs.size()):
			var q: float = ((hs[a][0] as Vector2).distance_to(hs[b][0] as Vector2)) / (float(hs[a][1]) + float(hs[b][1]))
			all = minf(all, q)
			if grp[a] != grp[b]:
				cross = minf(cross, q)
	return [all, cross]


func _run() -> void:
	_before = not _ov.has_method("_lift_poi")
	_p("FORM %s at %s touch=%s tag=%s before=%s" % [_form(), _vp.size, _touch, _tag, str(_before)])
	app.arm_tool("inspect")
	await _frames(4)
	if not _before and "u" in _legs:
		_leg_u()
	if "m" in _legs:
		await _leg_m()
	if "v" in _legs:
		await _leg_v()


func _stack(n: int, at: Vector2, r: float, base: int) -> Array:
	var o: Array = []
	for k in n:
		o.append({"id": base + k, "pos": at + Vector2(0.01 * k, 0.0), "r": r})
	return o


## The heads of `items` under layout `lay` (`fan_layout`'s `{id: Vector2(angle, lift)}`), as `[head, r]` pairs.
func _lay_heads(items: Array, lay: Dictionary) -> Array:
	var out: Array = []
	for it in items:
		var l: Vector2 = lay.get(it["id"], Vector2.ZERO)
		out.append(_head(it["pos"] as Vector2, l.x, l.y, float(it["r"]), ""))
	return out


func _leg_u() -> void:
	print("== U. the pure layout ==")
	var r := 9.0
	for n in [2, 3, 4, 5, 8, 12, 20, 36]:
		var it := _stack(n, Vector2(300, 300), r, 0)
		var lay := Pin.fan_layout(it)
		var rr := _ratios(_lay_heads(it, lay), it.map(func(_x): return 0))
		_ok("U1: %d pins on one tip all lay out and no two heads overlap (min %.3f >= %.2f)" % [n, rr[0], CLEAR], lay.size() == n and rr[0] >= CLEAR)
	# Ruling FA's looks are kept: literal degrees for 2, 3, 4.
	for spec in [[2, [-45.0, 45.0]], [3, [-60.0, 0.0, 60.0]], [4, [-90.0, -30.0, 30.0, 90.0]]]:
		var lay := Pin.fan_layout(_stack(int(spec[0]), Vector2(300, 300), r, 0))
		var degs: Array = []
		var any_lift := false
		for k in lay:
			degs.append(snappedf(rad_to_deg((lay[k] as Vector2).x), 0.01))
			any_lift = any_lift or (lay[k] as Vector2).y > 0.0
		degs.sort()
		_ok("U2: %d pins fan %s with no lift (Ruling FA's heart and fan are unchanged)" % [spec[0], str(spec[1])], degs == spec[1] and not any_lift, str(degs))
	# From five: ring 0 keeps the four of a half circle, ring 1 stands 2.3 radii further out.
	var l5 := Pin.fan_layout(_stack(5, Vector2(300, 300), r, 0))
	var up5 := 0
	var out5: Array = []
	for k in l5:
		if (l5[k] as Vector2).y == 0.0:
			up5 += 1
		else:
			out5.append([snappedf(rad_to_deg((l5[k] as Vector2).x), 0.01), snappedf((l5[k] as Vector2).y / r, 0.001)])
	_ok("U3: five pins are four on ring 0 and one on ring 1 at 0 degrees, lifted 2.3 radii (%s)" % str(out5), up5 == 4 and out5.size() == 1 and absf(float(out5[0][0])) < 0.01 and absf(float(out5[0][1]) - 2.3) < 0.001)
	var l20 := Pin.fan_layout(_stack(20, Vector2(300, 300), r, 0))
	var rings := [0, 0, 0, 0]
	for k in l20:
		rings[int(round((l20[k] as Vector2).y / (2.3 * r)))] += 1
	_ok("U3: twenty pins are 4 + 7 + 9 on rings 0, 1, 2 (%s)" % str(rings), rings == [4, 7, 9, 0])
	# The old stay-upright knot: 19 pins used to be left alone, 36 fan, 37 stay.
	_ok("U4: a knot of 19 (over the old limit of 18) now fans", Pin.fan_layout(_stack(19, Vector2(300, 300), r, 0)).size() == 19)
	_ok("U4: a knot of 37 (over FAN_PIN_MAX = %d) stays upright" % Pin.FAN_PIN_MAX, Pin.FAN_PIN_MAX == 36 and Pin.fan_layout(_stack(37, Vector2(300, 300), r, 0)).is_empty())
	# Neighbouring fans: the second stack is far enough that the upright pins never touched, yet its fan reaches the first.
	for d in [3.0, 5.0, 8.0, 12.0]:
		var it := _stack(12, Vector2(300, 300), r, 0) + _stack(12, Vector2(300.0 + d * r, 300), r, 100)
		var grp: Array = []
		for x in it:
			grp.append(0 if int(x["id"]) < 100 else 1)
		var lay := Pin.fan_layout(it)
		var rr := _ratios(_lay_heads(it, lay), grp)
		_ok("U5: two fans of 12, tips %.0f radii apart: all 24 lay out, heads of the two fans stay apart (cross min %.3f >= %.2f)" % [d, rr[1], NEAR], lay.size() == 24 and rr[1] >= NEAR)
	# Translation and the tie order.
	var a := Pin.fan_layout(_stack(12, Vector2(300, 300), r, 0))
	var b := Pin.fan_layout(_stack(12, Vector2(-77.5, 912.25), r, 0))
	var same := true
	for k in a:
		if not b.has(k) or ((a[k] as Vector2) - (b[k] as Vector2)).length() > 1e-6:
			same = false
	_ok("U6: a translation of every pin changes no angle or lift", same and a.size() == 12)


## Places `n` POIs on the grid point under overlay-local `at`, returns their indices.
func _place(at: Vector2, n: int, tag: String) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var g: Dictionary = _ov._grid_point(at, rect, interior)
	var kinds := ["tavern", "shop", "smithy", "gate"]
	var ids: Array = []
	for k in n:
		ids.append(_bridge.poi_add(float(g["gx"]), float(g["gy"]), kinds[k % kinds.size()], "%s %d" % [tag, k]))
	return ids


func _clear_pois() -> void:
	while _bridge.poi_list().size() > 0:
		_bridge.poi_delete(_bridge.poi_list().size() - 1)


## Measures the POIs `ids` as drawn now and asserts the four properties; `groups` gives each id's fan number.
func _measure_m(label: String, ids: Array, groups: Array, cross_bar: float) -> void:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var r: float = _ov._poi_pin_r()
	var hs: Array = []
	var heads: Array = []
	var tips_ok := true
	var lifted := 0
	var max_lift := 0.0
	for idx in ids:
		var pd: Dictionary = _bridge.poi_get(idx)
		var t: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), rect)
		if _ov._nudged_poi(idx, t, rect) != t:
			tips_ok = false
		var ang: float = _ov._tilt_poi(idx, rect)
		var lft: float = _ov._lift_poi(idx, rect) if _ov.has_method("_lift_poi") else 0.0
		if lft > 0.0:
			lifted += 1
			max_lift = maxf(max_lift, lft)
		var h := _head(t, ang, lft, r, String(pd.get("kind", "")))
		hs.append(h)
		heads.append(h[0])
	var rr := _ratios(hs, groups)
	_p("   %s: %d pins, %d on outer rings (longest stalk %.1f px = %.1f r), min head ratio %.3f, across fans %s" % [label, ids.size(), lifted, max_lift, max_lift / r, rr[0], "n/a" if rr[1] == INF else "%.3f" % rr[1]])
	if _before:
		return
	_ok("M %s: heads of one fan clear one another (min ratio %.3f >= %.2f, radius %.1f px)" % [label, rr[0] if rr[1] == INF else _within(hs, groups), CLEAR, r],
		(rr[0] if rr[1] == INF else _within(hs, groups)) >= CLEAR)
	if rr[1] != INF:
		_ok("M %s: heads of different fans overlap by under a quarter (cross min %.3f >= %.2f)" % [label, rr[1], cross_bar], rr[1] >= cross_bar)
	_ok("M %s: every tip is exactly its true place (no slide)" % label, tips_ok)
	var all_pick := true
	for k in ids.size():
		var res: Array = _picks_poi(heads[k], int(ids[k]))
		if res != [true, true, true, true]:
			all_pick = false
			_p("   pick %d at %s -> %s (_pick_mark %s, poi hit %s)" % [k, str(heads[k]), str(res), str(_ov._pick_mark(heads[k], interior, rect)), str(_ov._hit_test_poi(heads[k], interior, rect))])
	_ok("M %s: each drawn head picks its own POI through _hit_test_poi, hits_at, _pick_mark and marker_at" % label, all_pick)


## The smallest ratio among pairs of the SAME group.
func _within(hs: Array, grp: Array) -> float:
	var best := INF
	for a in hs.size():
		for b in range(a + 1, hs.size()):
			if grp[a] == grp[b]:
				best = minf(best, ((hs[a][0] as Vector2).distance_to(hs[b][0] as Vector2)) / (float(hs[a][1]) + float(hs[b][1])))
	return best


func _leg_m() -> void:
	print("== M. the map ==")
	DccSettings.set_marker_style("pins")
	await _close_card()
	app.arm_tool("inspect")
	await _frames(3)
	_clear_pois()
	app.viewport.refresh_annotations()
	await _frames(6)
	var spots: Array = []
	for clear in [12.0, 10.0, 8.0, 6.0, 5.0]:   ## as much room as the world has: a fan of 20 reaches 8 radii (a landmark under it is a different overlap)
		spots = _free_spots(12, clear)
		if spots.size() >= 8:
			_p("   spot clearance %.0f radii" % clear)
			break
	_ok("M0: free places for the cases", spots.size() >= 8, str(spots.size()))
	if spots.size() < 8:
		return
	var r: float = _ov._poi_pin_r()
	var cases := [5, 12] if _quick else [2, 3, 5, 8, 12, 20]
	for ci in cases.size():
		var n: int = cases[ci]
		_clear_pois()
		var ids := _place(spots[ci], n, "Fan")
		app.viewport.refresh_annotations()
		await _frames(8)
		_ok("M %d: the overlay holds %d POIs" % [n, n], _ov._pois.size() == n and not ids.has(-1), str(_ov._pois.size()))
		var grp: Array = ids.map(func(_x): return 0)
		_measure_m("%d on one tip" % n, ids, grp, NEAR)
		var rect: Rect2 = _ov._displayed_rect()
		var tip: Vector2 = _ov._point_to_screen(Vector2(float(_bridge.poi_get(ids[0])["x"]), float(_bridge.poi_get(ids[0])["y"])), rect)
		await _crop_shot("fanr_%s_map_%s_%02d.png" % [_tag, _form(), n], tip + Vector2(0.0, -r * 4.0), r * 11.0 * _hs())
		if not _before and n >= 5:
			# The stalk is drawn: a dark pixel on the stalk of a lifted pin, half way along it.
			await _frames(3)
			await RenderingServer.frame_post_draw
			var img := _vp.get_texture().get_image()
			var iset := DccSettings.marker_icon_set()
			# (1) Every drawn head is drawn WHERE it is picked: a pin-coloured (white or orange) pixel in a plus of seven points
			# round its head centre. A draw that ignored the lift would leave the head's place bare background.
			var seen := 0
			for idx in ids:
				var pdx: Dictionary = _bridge.poi_get(idx)
				var hh := _head(tip, _ov._tilt_poi(idx, rect), _ov._lift_poi(idx, rect), r, String(pdx.get("kind", "")))
				var found_px := false
				for off in [Vector2.ZERO, Vector2(1, 0), Vector2(-1, 0), Vector2(0, 1), Vector2(0, -1), Vector2(0.6, 0.6), Vector2(-0.6, 0.6)]:
					var at: Vector2 = _host((hh[0] as Vector2) + (off as Vector2) * float(hh[1]) * 0.55)
					var q := Vector2i(clampi(int(at.x), 0, img.get_width() - 1), clampi(int(at.y), 0, img.get_height() - 1))
					if _is_white(img.get_pixelv(q)) or _is_orange(img.get_pixelv(q)):
						found_px = true
				if found_px:
					seen += 1
			_ok("M %d: every one of the %d heads is drawn where it is picked (%d show pin colour)" % [n, ids.size(), seen], seen == ids.size())
			# (2) The stalks: sample each lifted pin's stalk where NO pin covers it (every pin's footprint, pad 2): there the pixel
			# must be ink (the stalk's line, within a pixel). Bar: at least one such point exists and every one is ink.
			var free_pts := 0
			var ink_pts := 0
			for idx in ids:
				var lft: float = _ov._lift_poi(idx, rect)
				if lft <= 0.0:
					continue
				var ang: float = _ov._tilt_poi(idx, rect)
				for f in [0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]:
					var sp: Vector2 = tip + Vector2(sin(ang), -cos(ang)) * lft * float(f)
					var covered := false
					for jdx in ids:
						var pj: Dictionary = _bridge.poi_get(jdx)
						if Pin.hit_styled(tip, r, sp, 1.0, false, _ov._tilt_poi(jdx, rect), String(pj.get("kind", "")), iset, _ov._lift_poi(jdx, rect)):
							covered = true
							break
					if covered:
						continue
					free_pts += 1
					var dark := false
					for dx in [-1, 0, 1]:
						for dy in [-1, 0, 1]:
							var at2: Vector2 = _host(sp + Vector2(dx, dy))
							var q2 := Vector2i(clampi(int(at2.x), 0, img.get_width() - 1), clampi(int(at2.y), 0, img.get_height() - 1))
							if img.get_pixelv(q2).get_luminance() < 0.3:
								dark = true
					if dark:
						ink_pts += 1
			var found := ink_pts
			var lifted_n := 0
			for idx in ids:
				if _ov._lift_poi(idx, rect) > 0.0:
					lifted_n += 1
			## An inner-ring stalk (one ring out, 2.3 radii) lies wholly under the ring-0 pins' stems, which tile the half circle, so only
			## the longer stalks (ring 2 on) show between the heads, and with 12 pins the one ring-2 pin's can sit behind a head: so the
			## bar applies from 20 pins, where eight ring-2 stalks fan out.
			_ok("M %d: the stalks of the %d lifted pins are inked where no pin covers them (%d of %d uncovered points)" % [n, lifted_n, ink_pts, free_pts],
				ink_pts == free_pts and (free_pts > 0 or n < 20))
	# Two fans near each other.
	for sep in ([4.0] if _quick else [4.0, 7.0]):
		_clear_pois()
		var rect: Rect2 = _ov._displayed_rect()
		var ids_a := _place(spots[6], 12, "A")
		var ids_b := _place((spots[6] as Vector2) + Vector2(sep * r, 0.0), 12, "B")
		app.viewport.refresh_annotations()
		await _frames(8)
		var ids: Array = ids_a + ids_b
		var grp: Array = []
		for k in ids.size():
			grp.append(0 if k < 12 else 1)
		_measure_m("two fans of 12, %.0f radii apart" % sep, ids, grp, NEAR)
		rect = _ov._displayed_rect()
		var tip: Vector2 = _ov._point_to_screen(Vector2(float(_bridge.poi_get(ids_a[0])["x"]), float(_bridge.poi_get(ids_a[0])["y"])), rect)
		await _crop_shot("fanr_%s_map_%s_two_%d.png" % [_tag, _form(), int(sep)], tip + Vector2(sep * r * 0.5, -r * 4.0), r * 16.0 * _hs())
	_clear_pois()
	app.viewport.refresh_annotations()
	await _frames(4)


func _cv_press(cv, local: Vector2, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = local
	ev.global_position = local
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	cv._on_canvas_input(ev)


func _town() -> int:
	var places: Array = _bridge.settlements()
	for k in places.size():
		var kind := String((places[k] as Dictionary).get("kind", ""))
		if kind in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				return k
	return -1


func _leg_v() -> void:
	print("== V. the City Viewer ==")
	await _close_card()
	var si := _town()
	_ok("V0: a town with a layout exists", si >= 0)
	if si < 0:
		return
	var s: Dictionary = _bridge.settlements()[si]
	var cv: CityViewerWindow = app.city_viewer_window
	var kinds := ["tavern", "shop", "smithy", "gate"]
	for n in ([5, 12] if _quick else [2, 3, 5, 8, 12, 20]):
		_clear_pois()
		var ids: Array = []
		for k in n:
			ids.append(_bridge.poi_add(float(s["x"]) + 0.5 + 0.12, float(s["y"]) + 0.5 + 0.05, kinds[k % kinds.size()], "V%d %d" % [n, k]))
		app.viewport.refresh_annotations()
		app.open_city_viewer(si)
		await _frames(14)
		var pins: Array = cv._pins()
		var poi_idx: Array = []
		for k in pins.size():
			if (pins[k] as Dictionary).has("poi"):
				poi_idx.append(k)
		_ok("V %d: the viewer shows the %d POIs as pins" % [n, n], poi_idx.size() == n, str(poi_idx.size()))
		var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
		var hs: Array = []
		var grp: Array = []
		var tips_ok := true
		var vt: Dictionary = cv._view_transform()
		for k in poi_idx:
			var true_k: Vector2 = (vt["origin"] as Vector2) + ((pins[k] as Dictionary)["at"] as Vector2) * float(vt["scale"])
			if cv.pin_tip(k) != true_k:
				tips_ok = false
			hs.append(_head(cv.pin_tip(k), cv.pin_tilt(k), cv.pin_lift(k) if cv.has_method("pin_lift") else 0.0, pr, String((pins[k] as Dictionary)["kind"])))
			grp.append(0)
		var rr := _ratios(hs, grp)
		var lifted := 0
		for k in poi_idx:
			if cv.has_method("pin_lift") and cv.pin_lift(k) > 0.0:
				lifted += 1
		_p("   V %d: %d pins on outer rings, min head ratio among the POIs %.3f" % [n, lifted, rr[0]])
		if not _before:
			_ok("V %d: the POIs' heads clear one another (min %.3f >= %.2f)" % [n, rr[0], CLEAR], rr[0] >= CLEAR)
			_ok("V %d: every tip is its true model place" % n, tips_ok)
			var all_pick := true
			for k in poi_idx.size():
				if cv.pin_at(hs[k][0] as Vector2) != poi_idx[k]:
					all_pick = false
					_p("   head %d at %s picks %d" % [poi_idx[k], str(hs[k][0]), cv.pin_at(hs[k][0] as Vector2)])
			_ok("V %d: each drawn head picks its own pin" % n, all_pick)
		var c := Vector2.ZERO
		for k in poi_idx:
			c += cv.pin_tip(k) / float(poi_idx.size())
		await _crop_shot("fanr_%s_viewer_%s_%02d.png" % [_tag, _form(), n], cv._canvas.get_global_transform_with_canvas() * (c + Vector2(0.0, -pr * 3.0)), pr * 12.0 * cv._canvas.get_global_transform_with_canvas().get_scale().x, true)
		await _shot("fanr_%s_viewerfull_%s_%02d.png" % [_tag, _form(), n])   ## the whole frame: the phone's viewer is a full-screen sheet, a crop of it needs the sheet's own offset
	_clear_pois()
