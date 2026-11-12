extends Node
## Touch fling, editable text and Asset library probe (`OUTSTANDING_WORK.md` "Phone and touch", the central touch-scroll
## row; owner, 2026-11-12: the Asset library survey rows, editable `TextEdit`s that do not scroll while editing, and no
## inertia on a proxy-scrolled list).
##
## Boots the real shell in a `--vp`-sized host `SubViewport` (the method `_touchscroll_probe.gd` uses) and drives REAL
## events the way a finger arrives with `emulate_mouse_from_touch` on: an `InputEventScreenTouch`/`ScreenDrag` AND its
## emulated `InputEventMouseButton`/`MouseMotion`. The scroll offsets, caret and selection themselves are the checks.
##
## Windowed, with a wiped user dir (hold the `C:/cw/.win_lock` convention when sharing the machine):
##   godot --path . --resolution 1600x1000 _flingscroll_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch  (phone class)
##   godot --path . --resolution 1600x1000 _flingscroll_probe.tscn -- --vp 3392x2400 --density 2.225 --force-touch   (Pad 3 class)
##   godot --path . --resolution 1600x1000 _flingscroll_probe.tscn -- --vp 1600x1000                                  (desktop pointer)
## Arguments read by THIS file: `--vp WxH`, `--only <leg>` (`fling`, `text`, `asset`). Accepted because the shell reads
## them: `--density D`, `--force-touch`. Anything else aborts with exit 2. Mutation runs edit the code under test (literal
## replacements, restored after each run), never this file.
##
## ## What it protects (every expected value is a literal written here or an independent measurement, never the code's own)
## 1. FLING. A stroke that ends at speed v leaves a proxied list, text box and dropdown coasting about v^2 / (2 * 1000) px
##    (the constant deceleration Godot's own `ScrollContainer` uses), v measured here from this file's own clock; a stock
##    `ScrollContainer` given the identical stroke coasts the same distance within a stated band; a press stops it; a
##    finger that rested before lifting, or a stroke slower than Android's minimum fling (50 dp/s), starts none.
## 2. EDITABLE TEXT. In a focused editable `TextEdit` a drag scrolls the box and leaves the caret, the selection and the
##    focus where they were; a tap and a long-press (no travel) still place the caret under the finger and do not scroll;
##    an unfocused box does not gain focus (so no keyboard) from a scroll; a short box hands the drag to the scroller above.
## 3. ASSET LIBRARY. Every overflowing scroller of the library window, and every control kind a finger can start on inside
##    it, scrolls by touch; the rail, the grid and the inspector are each driven.
## 4. DESKTOP. No node carries a proxy on a pointer build; the mouse wheel scrolls.
## Honest limit: synthetic events have no multi-touch, no Android slop and a frame-paced velocity; the finger is the final
## proof (`ANDROID_BUILD_SCOPE.md` 3.13).
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0
var _phone := false
var _unit := 1.0
var _only := ""
## The viewport events are pushed into: the host viewport, or one embedded window (events then carry that window's own
## pixel coordinates, `push_input(ev, true)`), which avoids guessing where the shell placed a window's frame.
var _pv: Viewport


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)


func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame


func _wait(sec: float) -> void:
	await get_tree().create_timer(sec).timeout


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt


func _now() -> float:
	return float(Time.get_ticks_usec()) / 1000000.0


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density", "--only"]:
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
	_dens = float(_arg("--density", "1"))
	_only = _arg("--only", "")
	get_tree().create_timer(1400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	Input.emulate_touch_from_mouse = true
	## A frame pacing close to a device's: unpaced headless frames take a millisecond, which turns a 9-step stroke into a
	## 100 000 px/s flick and every coast into the end of the list.
	Engine.max_fps = 60
	await _run(int(parts[0]), int(parts[1]))
	## A run that checked nothing is a failure, not a pass (a script error mid-run ends `_run` early).
	if _checks == 0:
		_fails += 1
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# ---- a finger: touch + emulated mouse, pushed at the host viewport ---------------------------------------------

## Canvas coordinates (what `get_global_rect()` reports) -> the pixels an event pushed into the host viewport carries. The
## host applies its stretch (`get_final_transform()`: ~2.2x on the tablet class, 1 on the phone class, whose scale lives in
## the controls); a push into an embedded window is already in canvas coordinates.
func _px(p: Vector2) -> Vector2:
	if _pv != null and _pv != _vp:
		return p
	return _vp.get_final_transform() * p


## The same scale for a velocity or a distance.
func _pk() -> float:
	if _pv != null and _pv != _vp:
		return 1.0
	return _vp.get_final_transform().get_scale().x


func _push(ev: InputEvent) -> void:
	var v: Viewport = _pv if _pv != null else _vp
	v.push_input(ev, v != _vp)


func _down(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_push(t)
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_push(m)
	await _frames(2)


## One drag sample carrying `vel` (px/s) the way a device reports it.
func _drag(p_canvas: Vector2, prev_canvas: Vector2, vel_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var prev := _px(prev_canvas)
	var vel := vel_canvas * _pk()
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - prev
	d.velocity = vel
	_push(d)
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - prev
	m.velocity = vel
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_push(m)


func _tap(p: Vector2) -> void:
	await _down(p)
	await _up(p)
	await _wait(0.2)


func _up(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_push(t)
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_push(m)
	await _frames(3)


## A stroke from `a` to `b` in `steps` drag samples, one per frame plus `extra_wait` seconds, reporting the speed over
## the last 100 ms from THIS file's own clock. Does not lift the finger. Returns {speed (signed px/s of the finger), last (b)}.
func _stroke_down(a: Vector2, b: Vector2, steps: int, extra_wait: float = 0.0) -> Dictionary:
	await _down(a)
	var prev := a
	var times: Array = [Vector2(_now(), a.y)]
	var v := 0.0
	for k in range(1, steps + 1):
		await _frames(1)
		if extra_wait > 0.0:
			await _wait(extra_wait)
		var p := a.lerp(b, float(k) / float(steps))
		times.append(Vector2(_now(), p.y))
		## The speed over the last 100 ms (a velocity tracker's reading), not the last frame's: one stalled frame on a
		## shared machine makes the next interval a few ms and the instantaneous speed several times too high, which a
		## stock `ScrollContainer` (it keeps the LAST event's velocity) turns into a coast several times too long
		## (measured once: 3.3x).
		v = _window_speed(times)
		_drag(p, prev, Vector2(0.0, v))
		prev = p
	return {"speed": v, "last": b}


## Signed y speed (px/s) of the finger over the last 100 ms of `times` (Vector2(seconds, y)), from at least two samples.
func _window_speed(times: Array) -> float:
	var last: Vector2 = times[times.size() - 1]
	var first: Vector2 = times[maxi(0, times.size() - 2)]
	for k in range(times.size() - 2, -1, -1):
		var s: Vector2 = times[k]
		if last.x - s.x > 0.1:
			break
		first = s
	return (last.y - first.y) / maxf(last.x - first.x, 0.004)


# ---- scene helpers ---------------------------------------------------------------------------------------------

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)


func _overflow(sc: ScrollContainer) -> float:
	return sc.get_v_scroll_bar().max_value - sc.get_v_scroll_bar().page


## Puts `c` as the only thing on a full-rect layer above the shell, fits it as the shell fits added content and returns the layer.
func _mount(c: Control) -> Control:
	var layer := Control.new()
	layer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.z_index = 100
	_app.add_child(layer)
	c.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	layer.add_child(c)
	if _phone:
		_app.phone_fit(layer, _unit)
	else:
		_app.tablet_arbitrate(layer)
		_app.tablet_fit(layer)
	return layer


## Waits until `getter` has not changed for 0.45 s (the coast is over), at most 6 s; returns the settled value.
func _settled(getter: Callable) -> float:
	var v: float = getter.call()
	var still := 0.0
	var waited := 0.0
	while still < 0.45 and waited < 6.0:
		await _wait(0.05)
		waited += 0.05
		var n: float = getter.call()
		if absf(n - v) < 0.5:
			still += 0.05
		else:
			still = 0.0
		v = n
	return v


## Scrolls `sc` so that `c` sits `margin` px below the scroller's top (clamped), then waits for layout. The content
## position is the control's screen position PLUS the current scroll, not its screen position alone.
func _bring(c: Control, sc: ScrollContainer, margin: float) -> void:
	var content_y := c.get_global_rect().position.y - sc.get_global_rect().position.y + float(sc.scroll_vertical)
	sc.scroll_vertical = clampi(int(content_y - margin), 0, int(_overflow(sc)))
	await _frames(4)


func _centre(c: Control) -> Vector2:
	return c.get_global_rect().intersection(Rect2(Vector2.ZERO, Vector2(_vp.size) / _pk())).get_center()


func _close_windows() -> void:
	if _app._phone_menu != null and _app._phone_menu.is_open():
		_app._phone_menu.close()
	var nodes: Array = []
	_all(_vp, nodes)
	for n in nodes:
		if n is Window and (n as Window).visible and n != _vp:
			(n as Window).hide()


# ---- the run -----------------------------------------------------------------------------------------------------

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _wait(1.6)
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_phone = DccTheme.is_phone()
	_unit = _app.phone_scale() if _phone else 1.0
	print("== %dx%d density %.4f: %s, unit %.4f ==" % [vw, vh, _dens, "phone" if _phone else ("tablet" if DccTheme.is_tablet() else "desktop"), _unit])
	var b = _app.bridge
	b.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	while b.generating:
		await _wait(0.25)
	await _wait(0.8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	if not DccTheme.is_touch():
		await _leg_desktop()
		return
	if _only == "" or _only == "fling":
		await _leg_fling()
	if _only == "" or _only == "text":
		await _leg_text()
	if _only == "" or _only == "asset":
		await _leg_asset()


## Desktop pointer: nothing is proxied and the wheel scrolls.
func _leg_desktop() -> void:
	print("-- desktop pointer: unchanged --")
	var nodes: Array = []
	_all(_vp, nodes)
	var stray := 0
	for n in nodes:
		if n.has_meta("_touch_scroll"):
			stray += 1
	_ok(stray == 0, "no node carries a touch-scroll proxy on a pointer build (%d)" % stray)
	var sc := ScrollContainer.new()
	var box := VBoxContainer.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sc.add_child(box)
	for k in 80:
		var l := Label.new()
		l.text = "row %d" % k
		l.custom_minimum_size.y = 40
		box.add_child(l)
	var te := TextEdit.new()
	te.text = "line\n".repeat(200)
	te.custom_minimum_size.y = 200
	box.add_child(te)
	var layer := _mount(sc)
	await _frames(6)
	_ok(not te.has_meta("_touch_scroll"), "an editable text box on a pointer build has no proxy")
	var at := sc.get_global_rect().position + Vector2(40.0, 60.0)
	for k in 3:
		var w := InputEventMouseButton.new()
		w.button_index = MOUSE_BUTTON_WHEEL_DOWN
		w.pressed = true
		w.position = at
		w.global_position = at
		_push(w)
		await _frames(3)
	_ok(sc.scroll_vertical > 0, "three wheel notches scroll the container (%d px)" % sc.scroll_vertical)
	layer.queue_free()


# ============================================================================================================
# 1. FLING
# ============================================================================================================

## A tall `ItemList`, a stock `ScrollContainer` of labels and a tall read-only `TextEdit` take the same stroke; the coast
## is measured from the last drag sample to rest and compared with v^2 / 2000 and with the stock container.
func _leg_fling() -> void:
	print("-- fling --")
	## Stroke: 9 samples, ~150 dp long. Enough room below for a coast of several thousand px on every control.
	var stroke_len := 150.0 * _dens

	## ---- proxied ItemList ----
	var il := ItemList.new()
	for k in 1500:
		il.add_item("item %d" % k)
	var layer := _mount(il)
	await _frames(6)
	_ok(il.has_meta("_touch_scroll"), "the tall list carries the shared touch-scroll proxy")
	var get_il := func() -> float: return il.get_v_scroll_bar().value
	var from := Vector2(il.get_global_rect().position.x + il.size.x * 0.5, il.get_global_rect().position.y + il.size.y * 0.85)
	var r: Dictionary = await _stroke_down(from, from - Vector2(0.0, stroke_len), 9)
	var at_last: float = get_il.call()
	await _up(from - Vector2(0.0, stroke_len))
	var rest: float = await _settled(get_il)
	var v: float = -float(r["speed"])
	var expect := v * v / (2.0 * 1000.0)
	var coast_il := rest - at_last
	print("   list: speed %.0f px/s, stroke moved %.0f, coasted %.0f px, v^2/2000 = %.0f" % [v, at_last, coast_il, expect])
	_ok(v > 300.0 * _dens * 0.5, "the stroke is a flick (%.0f px/s)" % v)
	_ok(coast_il > 0.5 * expect and coast_il < 1.6 * expect, "a proxied list coasts about v^2/2000 after the finger lifts (%.0f px, expected %.0f)" % [coast_il, expect])
	var il_total := rest
	layer.queue_free()
	await _frames(4)

	## ---- stock ScrollContainer, identical stroke ----
	var sc := ScrollContainer.new()
	sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var box := VBoxContainer.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sc.add_child(box)
	for k in 800:
		var l := Label.new()
		l.text = "row %d" % k
		l.custom_minimum_size.y = 40
		box.add_child(l)
	layer = _mount(sc)
	await _frames(8)
	var get_sc := func() -> float: return float(sc.scroll_vertical)
	from = Vector2(sc.get_global_rect().position.x + sc.size.x * 0.5, sc.get_global_rect().position.y + sc.size.y * 0.85)
	r = await _stroke_down(from, from - Vector2(0.0, stroke_len), 9)
	at_last = get_sc.call()
	await _up(from - Vector2(0.0, stroke_len))
	rest = await _settled(get_sc)
	var v_stock: float = -float(r["speed"])
	var coast_stock := rest - at_last
	var expect_stock := v_stock * v_stock / 2000.0
	print("   stock: speed %.0f px/s, coasted %.0f px, v^2/2000 = %.0f" % [v_stock, coast_stock, expect_stock])
	_ok(coast_stock > 0.0, "the stock container coasts at all (%.0f px): the comparison below is against a real fling" % coast_stock)
	## The proxied list and the stock container must coast alike for the same speed: compare distance per v^2.
	var k_il := coast_il / maxf(v * v, 1.0)
	var k_sc := coast_stock / maxf(v_stock * v_stock, 1.0)
	print("   coast / v^2: list %.6f, stock %.6f (1/2000 = 0.000500)" % [k_il, k_sc])
	_ok(k_il > 0.7 * k_sc and k_il < 1.4 * k_sc, "the proxied list coasts like the stock container for the same speed (%.6f vs %.6f px per (px/s)^2)" % [k_il, k_sc])
	layer.queue_free()
	await _frames(4)

	## ---- read-only and editable TextEdit, a dropdown's list ----
	for kind in ["read-only", "editable"]:
		var te := TextEdit.new()
		te.text = "line of text\n".repeat(3000)
		te.editable = (kind == "editable")
		layer = _mount(te)
		await _frames(8)
		_ok(te.has_meta("_touch_scroll"), "the %s text box carries the shared proxy" % kind)
		var lh := float(maxi(1, te.get_line_height()))
		var get_te := func() -> float: return te.get_v_scroll_bar().value * lh
		from = Vector2(te.get_global_rect().position.x + te.size.x * 0.5, te.get_global_rect().position.y + te.size.y * 0.85)
		r = await _stroke_down(from, from - Vector2(0.0, stroke_len), 9)
		at_last = get_te.call()
		await _up(from - Vector2(0.0, stroke_len))
		rest = await _settled(get_te)
		var vt: float = -float(r["speed"])
		var et := vt * vt / 2000.0
		var coast_te := rest - at_last
		print("   %s text: speed %.0f px/s, coasted %.0f px, v^2/2000 = %.0f" % [kind, vt, coast_te, et])
		_ok(coast_te > 0.5 * et and coast_te < 1.6 * et, "a %s text box coasts about v^2/2000 (%.0f px, expected %.0f)" % [kind, coast_te, et])
		if kind == "read-only":
			## A press stops a coast: lift, let it run 0.12 s, press without moving; the offset must stop within a few px.
			te.get_v_scroll_bar().value = 0.0
			await _frames(3)
			r = await _stroke_down(from, from - Vector2(0.0, stroke_len), 9)
			await _up(from - Vector2(0.0, stroke_len))
			var just_up: float = get_te.call()
			await _wait(0.12)
			var before_press: float = get_te.call()
			await _down(from)
			var at_press: float = get_te.call()
			await _up(from)
			var after: float = await _settled(get_te)
			print("   press-stop: running %.0f, at press %.0f, settled %.0f" % [before_press, at_press, after])
			_ok(before_press > just_up + 20.0, "(control) the coast was still running 0.12 s after the lift (%.0f -> %.0f px)" % [just_up, before_press])
			_ok(absf(after - at_press) < 6.0, "a press stops the coast (moved %.1f px after it)" % (after - at_press))
			## A finger that rested 0.3 s before lifting starts no coast.
			te.get_v_scroll_bar().value = 0.0
			await _frames(3)
			r = await _stroke_down(from, from - Vector2(0.0, stroke_len), 9)
			await _wait(0.3)
			var held: float = get_te.call()
			await _up(from - Vector2(0.0, stroke_len))
			var after_rest: float = await _settled(get_te)
			_ok(absf(after_rest - held) < 4.0, "a finger that rested 0.3 s before lifting starts no coast (%.1f px)" % (after_rest - held))
			## A stroke slower than Android's 50 dp/s minimum fling starts none. 20 dp/s: 1 dp every 50 ms (a frame is ~17 ms,
			## the rest is waited), 30 samples. The speed is measured here, over the last 100 ms, and a run whose frames
			## stalled (another process on the machine) is staged again, up to 3 times, rather than judged.
			var slow_ok := false
			var held2 := 0.0
			var after_slow := 0.0
			var slow_speed := 0.0
			for attempt in 3:
				te.get_v_scroll_bar().value = 0.0
				await _frames(3)
				var slow_len := 30.0 * _unit
				r = await _stroke_down(from, from - Vector2(0.0, slow_len), 30, 0.05 - 0.0167)
				slow_speed = absf(float(r["speed"]))
				held2 = get_te.call()
				await _up(from - Vector2(0.0, slow_len))
				after_slow = await _settled(get_te)
				if slow_speed < 40.0 * _unit and held2 > 0.0:
					slow_ok = true
					break
			print("   slow stroke: %.0f px/s (limit %.0f), moved %.0f, coasted %.2f" % [slow_speed, 50.0 * _unit, held2, after_slow - held2])
			_ok(slow_ok, "the slow stroke is staged below the minimum fling and still scrolled (%.0f px/s, %.0f px)" % [slow_speed, held2])
			_ok(slow_ok and absf(after_slow - held2) < 0.5, "a stroke below the minimum fling speed starts no coast (%.2f px; a coast from %.0f px/s would be ~%.1f px)" % [after_slow - held2, slow_speed, slow_speed * slow_speed / 2000.0])
		layer.queue_free()
		await _frames(4)

	## ---- a coast that passes on to the scroller above ends at a touch anywhere in that scroller ----
	var bed := ScrollContainer.new()
	bed.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var bcol := VBoxContainer.new()
	bcol.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bed.add_child(bcol)
	var il2 := ItemList.new()
	for k in 14:
		il2.add_item("short item %d" % k)
	il2.custom_minimum_size.y = 260
	bcol.add_child(il2)
	var labs: Array = []
	for k in 300:
		var l := Label.new()
		l.text = "row %d" % k
		l.custom_minimum_size.y = 40
		bcol.add_child(l)
		labs.append(l)
	layer = _mount(bed)
	await _frames(8)
	var get_bed := func() -> float: return float(bed.scroll_vertical)
	var il_rect := il2.get_global_rect()
	var bfrom := Vector2(il_rect.position.x + il_rect.size.x * 0.5, il_rect.position.y + il_rect.size.y * 0.8)
	var blen := minf(stroke_len, il_rect.size.y * 0.7)
	## Control: the stroke runs the short list to its end and the rest of the coast carries the page.
	r = await _stroke_down(bfrom, bfrom - Vector2(0.0, blen), 9)
	await _up(bfrom - Vector2(0.0, blen))
	await _wait(0.12)
	var x1: float = get_bed.call()
	await _wait(0.3)
	var x2: float = get_bed.call()
	_ok(x2 > x1 + 40.0, "(control) a coast that outruns a short list carries on into the scroller above (%.0f -> %.0f px)" % [x1, x2])
	await _settled(get_bed)
	## Test: the same, then a tap on a row of the page.
	bed.scroll_vertical = 0
	il2.get_v_scroll_bar().value = 0.0
	await _frames(4)
	r = await _stroke_down(bfrom, bfrom - Vector2(0.0, blen), 9)
	await _up(bfrom - Vector2(0.0, blen))
	await _wait(0.12)
	var lab: Label = null
	for l in labs:
		if (l as Label).is_visible_in_tree() and (l as Label).get_global_rect().position.y > il_rect.end.y + 60.0:
			lab = l
			break
	_ok(lab != null, "(setup) a page row is in view to touch")
	if lab != null:
		await _down(_centre(lab))
		var at_tap: float = get_bed.call()
		await _up(_centre(lab))
		var final_x: float = await _settled(get_bed)
		_ok(absf(final_x - at_tap) < 8.0, "a touch on a row of the page stops the coast that was carrying it (%.0f px more)" % (final_x - at_tap))
	## Someone else scrolling the page (here a write, as a wheel notch or a second finger would) ends the coast too.
	bed.scroll_vertical = 0
	il2.get_v_scroll_bar().value = 0.0
	await _frames(4)
	r = await _stroke_down(bfrom, bfrom - Vector2(0.0, blen), 9)
	await _up(bfrom - Vector2(0.0, blen))
	await _wait(0.3)
	var running: float = get_bed.call()
	bed.scroll_vertical = 0
	var after_write: float = await _settled(get_bed)
	_ok(running > 40.0 and after_write < 25.0, "a scroll written by someone else ends the coast instead of fighting it (page was at %.0f px, ends at %.0f)" % [running, after_write])
	layer.queue_free()
	await _frames(4)

	## ---- a dropdown's popup list ----
	var ob := OptionButton.new()
	ob.name = "FlingDrop"
	for k in 600:
		ob.add_item("option %d" % k)
	ob.custom_minimum_size = Vector2(300, 60)
	var holder := VBoxContainer.new()
	holder.add_child(ob)
	layer = _mount(holder)
	await _frames(8)
	var pop := ob.get_popup()
	_ok(pop.has_meta("_touch_scroll"), "the dropdown's popup carries the shared proxy")
	await _tap(_centre(ob))
	await _frames(4)
	var by_tap := pop.visible
	if not by_tap:
		print("   (windowed run: the synthetic tap opened and re-closed the list; opening it with show_popup())")
		ob.show_popup()
		await _frames(4)
	var psc: ScrollContainer = null
	var stk: Array = [pop]
	while not stk.is_empty() and psc == null:
		var cur: Node = stk.pop_back()
		for ch in cur.get_children(true):
			if ch is ScrollContainer:
				psc = ch as ScrollContainer
				break
			stk.append(ch)
	_ok(pop.visible and psc != null, "the dropdown's list is open with its scroller")
	if pop.visible and psc != null:
		var pr := Rect2(Vector2(pop.position), Vector2(pop.size))
		var pfrom := pr.position + Vector2(pr.size.x * 0.5, pr.size.y * 0.85)
		var plen := pr.size.y * 0.5
		var get_p := func() -> float: return float(psc.scroll_vertical)
		r = await _stroke_down(pfrom, pfrom - Vector2(0.0, plen), 9)
		at_last = get_p.call()
		await _up(pfrom - Vector2(0.0, plen))
		rest = await _settled(get_p)
		var vp_: float = -float(r["speed"])
		var ep := vp_ * vp_ / 2000.0
		print("   popup: speed %.0f px/s, stroke moved %.0f, coasted %.0f, v^2/2000 = %.0f" % [vp_, at_last, rest - at_last, ep])
		_ok(rest - at_last > 0.5 * ep and rest - at_last < 1.6 * ep, "a dropdown's list coasts about v^2/2000 (%.0f px, expected %.0f)" % [rest - at_last, ep])
	elif pop.visible:
		print("   (windowed: the popup coast is asserted where the tap opened the list, as in _touchscroll_probe.gd)")
	if pop.visible:
		pop.hide()
	layer.queue_free()
	await _frames(4)


# ============================================================================================================
# 2. EDITABLE TEXT
# ============================================================================================================

func _line_text(n: int) -> String:
	var s := ""
	for k in n:
		s += "line %03d some words to tap on\n" % k
	return s


func _leg_text() -> void:
	print("-- editable text --")
	var bed := ScrollContainer.new()
	bed.name = "TextBed"
	bed.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bed.add_child(col)
	var head := Label.new()
	head.text = "bed"
	head.custom_minimum_size.y = 60
	col.add_child(head)
	var te := TextEdit.new()
	te.name = "Long"
	te.text = _line_text(200)
	te.custom_minimum_size.y = 420
	te.scroll_smooth = false
	col.add_child(te)
	var short := TextEdit.new()
	short.name = "Short"
	short.text = "one\ntwo\nthree"
	short.scroll_fit_content_height = true
	col.add_child(short)
	for k in 40:
		var l := Label.new()
		l.text = "filler %d" % k
		l.custom_minimum_size.y = 50
		col.add_child(l)
	var layer := _mount(bed)
	await _frames(8)
	_ok(te.has_meta("_touch_scroll") and short.has_meta("_touch_scroll"), "both EDITABLE text boxes carry the shared proxy (long: %s, short: %s)" % [te.has_meta("_touch_scroll"), short.has_meta("_touch_scroll")])
	_ok(te.editable and te.selecting_enabled and te.focus_mode != Control.FOCUS_NONE, "...and kept their editing: editable, selectable, focusable")
	await _bring(te, bed, 40.0)
	await _frames(4)
	var lh := float(maxi(1, te.get_line_height()))
	## Everything a gesture can move, so a coast is over when this stops changing.
	var all_pos := func() -> float: return te.get_v_scroll_bar().value * lh + float(bed.scroll_vertical) + short.get_v_scroll_bar().value
	var rect := te.get_global_rect()
	var len_ := minf(140.0 * _dens, rect.size.y * 0.6)
	var from := Vector2(rect.position.x + rect.size.x * 0.5, rect.position.y + rect.size.y * 0.85)

	## T1: focused, caret on line 7 column 3: a drag scrolls the box, caret and focus stay, no selection appears.
	te.grab_focus()
	te.set_caret_line(7, false)
	te.set_caret_column(3, false)
	te.scroll_vertical = 0
	await _frames(4)
	var bed0 := bed.scroll_vertical
	await _stroke_down(from, from - Vector2(0.0, len_), 12)
	await _up(from - Vector2(0.0, len_))
	await _settled(all_pos)
	var moved_lines := te.get_v_scroll_bar().value
	_ok(moved_lines * lh >= 30.0 * _dens, "a drag inside a focused editable box scrolls it (%.1f lines, %.0f px)" % [moved_lines, moved_lines * lh])
	_ok(te.get_caret_line() == 7 and te.get_caret_column() == 3, "...and leaves the caret at line 7 column 3 (line %d column %d)" % [te.get_caret_line(), te.get_caret_column()])
	_ok(not te.has_selection(), "...and selects nothing")
	_ok(te.has_focus(), "...and the box is still focused")
	_ok(bed.scroll_vertical == bed0, "...and the scroller above did not move (it only receives what the box cannot take: %d -> %d)" % [bed0, bed.scroll_vertical])

	## T2: a tap on a line places the caret on THAT line and does not scroll.
	te.scroll_vertical = 40
	await _frames(4)
	var tap_at := Vector2(rect.position.x + 120.0 * _dens * 0.5, rect.position.y + rect.size.y * 0.5)
	var expect_lc := te.get_line_column_at_pos(Vector2i(tap_at - te.get_global_rect().position))
	var sv0 := te.get_v_scroll_bar().value
	await _down(tap_at)
	await _up(tap_at)
	await _wait(0.5)
	_ok(te.get_caret_line() == expect_lc.y and expect_lc.y >= 40, "a tap places the caret on the line under the finger (line %d, the box says %d)" % [te.get_caret_line(), expect_lc.y])
	_ok(absf(te.get_v_scroll_bar().value - sv0) < 0.6, "...and the tap did not scroll (%.2f -> %.2f lines)" % [sv0, te.get_v_scroll_bar().value])

	## T3: a long-press (0.7 s, no travel) places the caret and does not scroll either.
	te.scroll_vertical = 70
	te.set_caret_line(2, false)
	await _frames(4)
	var tap2 := Vector2(rect.position.x + 150.0 * _dens * 0.5, rect.position.y + rect.size.y * 0.3)
	var expect2 := te.get_line_column_at_pos(Vector2i(tap2 - te.get_global_rect().position))
	var sv1 := te.get_v_scroll_bar().value
	await _down(tap2)
	await _wait(0.7)
	await _up(tap2)
	await _wait(0.4)
	_ok(te.get_caret_line() == expect2.y and expect2.y >= 70, "a long-press places the caret on the line under the finger (line %d, the box says %d)" % [te.get_caret_line(), expect2.y])
	_ok(absf(te.get_v_scroll_bar().value - sv1) < 0.6, "...and the long-press did not scroll (%.2f -> %.2f lines)" % [sv1, te.get_v_scroll_bar().value])

	## T4: an UNFOCUSED box scrolled by a drag does not gain focus (no keyboard) and keeps its caret.
	te.release_focus()
	te.set_caret_line(5, false)
	te.set_caret_column(2, false)
	te.scroll_vertical = 0
	await _frames(4)
	_ok(not te.has_focus(), "(setup) the box is not focused")
	await _stroke_down(from, from - Vector2(0.0, len_), 12)
	await _up(from - Vector2(0.0, len_))
	await _settled(all_pos)
	_ok(te.get_v_scroll_bar().value * lh >= 30.0 * _dens, "a drag on an unfocused editable box scrolls it (%.1f lines)" % te.get_v_scroll_bar().value)
	_ok(not te.has_focus(), "...and does not focus it (so no keyboard opens for a scroll)")
	_ok(te.get_caret_line() == 5 and te.get_caret_column() == 2, "...and keeps its caret (line %d column %d)" % [te.get_caret_line(), te.get_caret_column()])

	## T5: a selection survives a scroll.
	te.grab_focus()
	te.select(30, 0, 30, 9)
	te.scroll_vertical = 20
	await _frames(4)
	await _stroke_down(from, from - Vector2(0.0, len_), 12)
	await _up(from - Vector2(0.0, len_))
	await _settled(all_pos)
	_ok(te.has_selection() and te.get_selection_from_line() == 30 and te.get_selection_to_line() == 30 and te.get_selection_from_column() == 0 and te.get_selection_to_column() == 9,
		"a selection (line 30, columns 0-9) survives a drag (selection %s: %d:%d to %d:%d)" % [te.has_selection(), te.get_selection_from_line(), te.get_selection_from_column(), te.get_selection_to_line(), te.get_selection_to_column()])
	_ok(te.get_v_scroll_bar().value > 20.0 + 1.0, "...and the box scrolled meanwhile (%.1f lines)" % te.get_v_scroll_bar().value)
	te.deselect()

	## T6: a box at its end hands the drag to the scroller above.
	te.get_v_scroll_bar().value = te.get_v_scroll_bar().max_value
	await _bring(te, bed, bed.size.y * 0.5)
	rect = te.get_global_rect()
	from = Vector2(rect.position.x + rect.size.x * 0.5, rect.position.y + rect.size.y * 0.85)
	var bed1 := bed.scroll_vertical
	await _stroke_down(from, from - Vector2(0.0, len_), 12)
	await _up(from - Vector2(0.0, len_))
	await _settled(all_pos)
	_ok(bed.scroll_vertical - bed1 >= int(30.0 * _dens), "a box already at its end passes the drag to the scroller above (%+d px)" % (bed.scroll_vertical - bed1))

	## T7: a short box that never overflows is just part of the page: the drag scrolls the scroller above, caret unchanged.
	short.grab_focus()
	short.set_caret_line(1, false)
	short.set_caret_column(1, false)
	await _bring(short, bed, bed.size.y * 0.5)
	var srect := short.get_global_rect()
	var sfrom := Vector2(srect.position.x + srect.size.x * 0.5, srect.position.y + srect.size.y * 0.5)
	var bed2 := bed.scroll_vertical
	await _stroke_down(sfrom, sfrom - Vector2(0.0, len_), 12)
	await _up(sfrom - Vector2(0.0, len_))
	await _settled(all_pos)
	_ok(bed.scroll_vertical - bed2 >= int(30.0 * _dens), "a drag on a short editable box scrolls the scroller above (%+d px)" % (bed.scroll_vertical - bed2))
	_ok(short.get_caret_line() == 1 and short.get_caret_column() == 1 and not short.has_selection(), "...and leaves its caret at line 1 column 1 with no selection (line %d column %d)" % [short.get_caret_line(), short.get_caret_column()])

	## T8: after all of that, typing still lands at the caret and scrolls the view to it (the box is not left half-frozen).
	te.grab_focus()
	te.set_caret_line(120, true)
	te.set_caret_column(0, true)
	await _frames(3)
	te.insert_text_at_caret("Z")
	await _frames(3)
	_ok(te.get_line(120).begins_with("Z"), "typing still works after the touch scroll ('%s')" % te.get_line(120).substr(0, 6))
	layer.queue_free()
	await _frames(4)


# ============================================================================================================
# 3. ASSET LIBRARY
# ============================================================================================================

## A window-local canvas point as the event position for a push into that window (`push_input(ev, true)` takes canvas
## coordinates: the window's content scale is applied by the window, not by the caller).
func _to_vp(_w: Window, local: Vector2) -> Vector2:
	return local


## The topmost control a finger at window-local `local` would land on: reverse tree order, visible, not IGNORE, inside the
## rect of every clipping ancestor (Godot's own hit test, as `_phonescroll_probe.gd` mirrors it).
func _hit(w: Window, local: Vector2) -> Control:
	var all: Array = []
	_all(w, all)
	for k in range(all.size() - 1, -1, -1):
		var c := all[k] as Control
		if c == null or not c.is_visible_in_tree() or c.mouse_filter == Control.MOUSE_FILTER_IGNORE:
			continue
		if not c.get_global_rect().has_point(local):
			continue
		var clipped := false
		var a := c.get_parent()
		while a is Control:
			if (a as Control).clip_contents and not (a as Control).get_global_rect().has_point(local):
				clipped = true
				break
			a = a.get_parent()
		if not clipped:
			return c
	return null


func _sig(c: Control) -> String:
	if c == null:
		return "none"
	var scr := c.get_script() as Script
	return "%s|filter=%d|%s" % [c.get_class(), c.mouse_filter, scr.resource_path.get_file() if scr != null else ("SlotCell" if c.has_meta(DccWidgets.TAP_META) else "-")]


## Every visible, vertically overflowing ScrollContainer under `root` (more than 60 px of room, at least 80 px tall).
func _overflowing(root: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_all(root, all)
	for c in all:
		if c is ScrollContainer and (c as Control).is_visible_in_tree() and (c as ScrollContainer).vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED 				and (c as Control).size.y >= 80.0 and _overflow(c as ScrollContainer) > 60.0:
			out.append(c)
	return out


## One tile centre of the visible grid, window-local, or the zero vector when there is none.
func _first_tile(lib: Node) -> Control:
	var all: Array = []
	_all(lib, all)
	for c in all:
		if c is Control and (c as Control).has_meta(DccWidgets.TAP_META) and (c as Control).is_visible_in_tree() and (c as Control).size.y >= 40.0 and "uid" in c and String(c.uid) != "":
			return c as Control
	return null


func _leg_asset() -> void:
	print("-- asset library --")
	_close_windows()
	var lib = _app.asset_library_window
	lib.open("", false)
	await _wait(1.2)
	var phone_lib: bool = lib._phone
	_pv = lib
	print("   library composition: %s" % ("phone panes" if phone_lib else "desktop columns"))
	_ok(lib.visible, "the library window is open")
	if not phone_lib:
		var best := ""
		var best_n := -1
		for fam in lib.FAMILIES:
			lib._select_family(String(fam["key"]))
			await _frames(4)
			var n: int = lib._grid.get_child_count()
			if n > best_n:
				best_n = n
				best = String(fam["key"])
		lib._select_family(best)
		await _frames(8)
		print("   family with the most slots: %s (%d tiles)" % [best, best_n])
		## The desktop composition is a 1024x640 window (its `min_size`) in a tablet-sized screen, and the largest family
		## has 15 tiles in 6 columns: nothing overflows. Two columns stand in for a pack with more slots than the window
		## shows, which is what a person who has imported a pack has; the grid, rail and inspector then all run past their ends.
		lib._grid.columns = 2
		## The same reason for the inspector: a selected slot's panel (rows, swatches, variants, REFINE) fits the 640 px
		## window on some font metrics and not on others, so the window is made shorter than its `min_size` here rather than
		## leaving "does the inspector overflow" to the machine's fonts.
		lib.min_size = Vector2i(1024, 300)
		lib.size = Vector2i(1024, 360)
		await _frames(8)
	var panes: Array = ["slots", "families", "slot"] if phone_lib else ["slots", "slot"]
	for pane in panes:
		if phone_lib:
			lib._show_phone_pane(pane)
		if pane == "slot":
			## The inspector needs a focused slot: select one by a REAL tap first (the tap contract is checked here too).
			if phone_lib:
				lib._show_phone_pane("slots")
			await _frames(6)
			var t0 := _first_tile(lib)
			_ok(t0 != null, "(setup) the grid has a slot tile")
			if t0 == null:
				continue
			lib._selected.clear()
			var win0 := t0.get_viewport() as Window
			var at0 := _to_vp(win0, t0.get_global_rect().get_center())
			await _down(at0)
			_ok(lib._selected.is_empty(), "a touch-DOWN on a tile selects nothing yet (the tap acts on release)")
			await _up(at0)
			await _wait(0.4)
			_ok(lib._selected.size() == 1, "a tap on a tile selects it on release (%d selected)" % lib._selected.size())
			if phone_lib:
				_ok(lib._phone_pane == "slot", "...and the phone moves to the slot pane (%s)" % lib._phone_pane)
			await _frames(8)
			await _asset_pane(lib, "slot (inspector)", 1 if phone_lib else 2)
			continue
		await _frames(6)
		await _asset_pane(lib, pane)
	if phone_lib:
		lib._show_phone_pane("slots")
	await _frames(6)
	## A drag that starts on a tile scrolls the grid and neither selects it nor moves to the slot pane.
	lib._selected.clear()
	await _asset_pane(lib, "slots (after the tap)")
	_pv = null
	lib.hide()
	await _frames(4)
	_close_windows()


## Drives every distinct landing control of every overflowing scroller of the library as it stands.
func _asset_pane(lib: Node, label: String, want_min: int = 1) -> void:
	var scs := _overflowing(lib)
	print("   pane '%s': %d overflowing scroller(s)" % [label, scs.size()])
	var dump: Array = []
	_all(lib, dump)
	for c in dump:
		if c is ScrollContainer and (c as Control).is_visible_in_tree():
			print("   . visible ScrollContainer %s rect %s overflow %.0f mode %d window %s size %s scale %s" % [c.name, (c as Control).get_global_rect(), _overflow(c as ScrollContainer), (c as ScrollContainer).vertical_scroll_mode, (c.get_viewport() as Window).name, (c.get_viewport() as Window).size, (c.get_viewport() as Window).get_final_transform().get_scale()])
	_ok(scs.size() >= want_min, "pane '%s' has at least %d overflowing scroller(s) to drive (%d)" % [label, want_min, scs.size()])
	for sc_n in scs:
		var sc := sc_n as ScrollContainer
		var w := sc.get_viewport() as Window
		var vis := sc.get_global_rect().intersection(Rect2(Vector2.ZERO, Vector2(w.size) / w.get_final_transform().get_scale()))
		var scale_ := 1.0
		var len_loc := minf(110.0, vis.size.y * 0.45)
		var picked: Array = []
		var seen := {}
		for ry in 10:
			for cx in 5:
				var p := Vector2(vis.position.x + vis.size.x * (float(cx) + 0.5) / 5.0, vis.position.y + 6.0 + (vis.size.y - 12.0) * (float(ry) + 0.5) / 10.0)
				if p.y - len_loc < vis.position.y + 3.0:
					continue
				var h := _hit(w, p)
				var key := _sig(h)
				if seen.has(key):
					continue
				seen[key] = true
				picked.append([p, key])
		print("   scroller %s (overflow %.0f px): %d landing kinds" % [sc.name, _overflow(sc), picked.size()])
		for pk in picked:
			sc.scroll_vertical = 0
			await _frames(3)
			var sel0: int = lib._selected.size()
			var pane0 = lib._phone_pane if lib._phone else ""
			var from := _to_vp(w, pk[0] as Vector2)
			await _stroke_down(from, from - Vector2(0.0, len_loc * scale_), 12)
			## A tile that is a drag source would have started a drag-and-drop by now (Godot's 10 px threshold): a scroll
			## and a drag cannot both own the finger.
			var dragging := w.gui_is_dragging()
			await _up(from - Vector2(0.0, len_loc * scale_))
			var rest: float = await _settled(func() -> float: return float(sc.scroll_vertical))
			var sel1: int = lib._selected.size()
			var pane1 = lib._phone_pane if lib._phone else ""
			_ok(rest >= 0.4 * len_loc and sel1 == sel0 and pane1 == pane0 and not dragging, "a drag from %s scrolls the %s (%.0f px, wanted %.0f), starts no drag-and-drop (%s) and selects/navigates nothing (selected %d->%d, pane %s->%s)" % [pk[1], sc.name, rest, 0.4 * len_loc, dragging, sel0, sel1, pane0, pane1])
		sc.scroll_vertical = 0
