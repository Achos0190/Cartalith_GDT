extends Node
## **Painting the Assistant's edit region with a finger** (`LLM_ASSISTANT_SCOPE.md` LLM-7 remainder, s.5h item 7;
## `llm_region.gd`). The Paint region button is clicked for real, a real finger (touch events through
## `Input.parse_input_event`, Godot's own mouse emulation on finger 0) paints, a second finger must not, and Done
## brings the Assistant back. Not the stale `_llmedit_probe` window flow: nothing here reads `_win.visible` on a form
## that docks the panel.
##   desktop: godot --path . --resolution 1600x900 res://_llmtouchpaint_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1700x1000 res://_llmtouchpaint_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   phone:   godot --path . --resolution 393x852 res://_llmtouchpaint_probe.tscn -- --force-touch --shots <dir>
## **Windowed, never `--headless`**, with a wiped user dir. Arguments: `--shots <dir>`, `--force-touch`, `--density D`
## (the shell's), `--failfast`; anything else exits 2.
##
## What it protects (each check names it):
##   1. THE BUTTON: Paint region is enabled on every form with a world open (the phone's used to be disabled), its
##      tooltip says "one finger" on touch forms; a real click arms the region brush and shows the bar.
##   2. THE BAR: on touch its hint says one finger paints and two move the map, its controls meet the 44 px floor,
##      it lies inside the map and is not a tower of one-control lines.
##   3. A MOUSE drag paints ONE stroke (the control: `_click` no longer resets an open stroke).
##   4. A FINGER drag paints dabs as ONE stroke ending at the lift point, does not pan the map, and the overlay
##      draws exactly the dabs; a TAP paints one dab as its own stroke.
##   5. TWO FINGERS: a second finger landing mid-stroke adds no dabs (the stray-stroke guard) and the pinch is the
##      map's, not the brush's.
##   6. UNDO STROKE takes a whole finger stroke back; Done brings the Assistant back on Edit, disarms the brush
##      and removes the bar.
const _REGION := preload("res://shell/llm_region.gd")

var _fails := 0
var _checks := 0
var _failfast := false
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1
		if _failfast:
			_finish()


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(4)
	var path := _shots.path_join("touchpaint_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _generate() -> void:
	_bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(45)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


## A REAL mouse press and release at the centre of `c`, scrolled into view first.
func _click(c: Control) -> void:
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
			await _settle(2)
		p = p.get_parent()
	var at: Vector2 = c.get_viewport().get_final_transform() * c.get_global_rect().get_center()
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _mouse(at: Vector2, down, held: bool = false) -> void:
	if down == null:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT if held else 0
		Input.parse_input_event(mm)
	else:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = bool(down)
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
	await _settle(2)


## One finger event through the real input path; `pressed` null is a drag.
func _touch(idx: int, at: Vector2, pressed, rel := Vector2.ZERO) -> void:
	if pressed == null:
		var d := InputEventScreenDrag.new()
		d.index = idx
		d.position = at
		d.relative = rel
		Input.parse_input_event(d)
	else:
		var t := InputEventScreenTouch.new()
		t.index = idx
		t.pressed = bool(pressed)
		t.position = at
		Input.parse_input_event(t)
	await _settle(2)


func _finger_drag(a: Vector2, b: Vector2, steps := 8) -> void:
	await _touch(0, a, true)
	var prev := a
	for k in range(1, steps + 1):
		var nxt := a.lerp(b, float(k) / float(steps))
		await _touch(0, nxt, null, nxt - prev)
		prev = nxt
	await _touch(0, b, false)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--density" and i + 1 < args.size():
			i += 1
		elif args[i].begins_with("--density="):
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif args[i] == "--failfast":
			_failfast = true
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --density, --shots <dir>, --failfast" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe drives real input")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_window
	if not _bridge.llm_api or _win == null:
		_ok(false, "the library has LlmBridge and the panel exists")
		_finish()
		return
	for k in {"enabled": true, "edit": true, "ctx_region": true}:
		DccSettings.set_llm_value(k, true)
	await _generate()
	_REGION.clear(_app)
	_win.open()
	await _settle(10)
	_win._pick_task("edit")
	await _settle(6)

	print("== 1. the Paint region button ==")
	var paint_btn := _win.controls.get("region_paint") as Button
	_ok(paint_btn != null and not paint_btn.disabled, "Paint region is enabled with a world open on the %s form: '%s'" % [_mode, paint_btn.tooltip_text.left(60) if paint_btn != null else "missing"])
	if paint_btn == null:
		_finish()
		return
	if _mode != "desktop":
		_ok(paint_btn.tooltip_text.contains("one finger"), "its tooltip says one finger on a touch form")
	await _click(paint_btn)
	await _settle(8)
	_ok(_REGION.painting and _app.armed_tool == _REGION.TOOL_ID, "a real click armed the region brush (armed %s)" % _app.armed_tool)
	var vp = _app.viewport
	var bar = vp.get_node_or_null("AssistRegionBar")
	_ok(bar != null and (bar as Control).visible, "the painting bar is on the map")

	print("== 2. the bar ==")
	if bar != null:
		var texts: Array = []
		_all_labels(bar, texts)
		var said_finger := false
		for t in texts:
			if String(t).contains("One finger paints"):
				said_finger = true
		_ok(said_finger == (_mode != "desktop"), "the hint says 'One finger paints' exactly on touch forms (touch=%s)" % str(_mode != "desktop"))
		var br: Rect2 = (bar as Control).get_global_rect()
		var vr: Rect2 = vp.get_global_rect()
		_ok(vr.encloses(br) and br.size.y < (130.0 if _mode != "desktop" else 80.0), "the bar %s lies inside the map %s and is not a tower of lines" % [br, vr])
		if _mode == "phone":
			## The phone's app bar floats over the top `safe_insets().top` px of the map; the bar must sit below it.
			var top_inset := float(vp.safe_insets().get("top", 0.0))
			_ok(top_inset > 40.0 and br.position.y >= top_inset, "the bar clears the phone's app bar: top %.0f, inset %.0f" % [br.position.y, top_inset])
		if _mode != "desktop":
			## 44 dp: on the phone the main viewport is not content-scaled, so the floor is 44 px times the phone scale.
			var scale := float(vp.safe_insets().get("scale", 1.0)) if _mode == "phone" else 1.0
			for nm in ["assist_done", "assist_clear", "assist_undo_stroke", "assist_brush"]:
				var bb := (bar as Control).find_child(nm, true, false) as Control
				_ok(bb != null and bb.get_global_rect().size.y >= 43.5 * scale, "the bar's %s meets the 44 dp floor (%.1f px at scale %.2f): %s" % [nm, 43.5 * scale, scale, str(bb.get_global_rect().size) if bb != null else "missing"])
	if _mode == "phone":
		_ok(not _win.visible, "on the phone the Assistant dialog hid so the map is reachable")

	var rect: Rect2 = vp.overlay.displayed_rect()
	var g: Vector2i = _bridge.grid_size()
	var to_screen := func(gx: float, gy: float) -> Vector2:
		var rel := Vector2((gx + 0.5) / float(g.x), (gy + 0.5) / float(g.y))
		var local: Vector2 = vp._camera.position + (rect.position + rel * rect.size) * float(vp.zoom())
		var vpt: Viewport = vp.get_viewport()
		return vpt.get_final_transform() * (vp.get_global_transform() * local)
	var cam0: Vector2 = vp._camera.position

	print("== 3. a mouse drag is ONE stroke ==")
	## A phone has no mouse: with `--force-touch` at phone size the host does not deliver a mouse release to the brush
	## (measured: `_stroke_open` stayed true), so the mouse control runs on the desktop and tablet forms only.
	if _mode != "phone":
		var m0: Vector2 = to_screen.call(80.0, 60.0)
		await _mouse(m0, null)
		await _mouse(m0, true)
		for k in range(1, 9):
			await _mouse(to_screen.call(80.0 + 6.0 * k, 60.0 + 2.0 * k), null, true)
		await _mouse(to_screen.call(128.0, 76.0), false)
		_ok(_REGION.dabs.size() >= 3 and _REGION.strokes.size() == 1, "one mouse drag painted %d dabs as %d stroke" % [_REGION.dabs.size(), _REGION.strokes.size()])
		_ok(not _REGION._stroke_open, "the mouse release closed the stroke")
		_REGION.undo_stroke(_app)
		_ok(_REGION.dabs.is_empty(), "Undo stroke removed it whole")

	print("== 4. a finger drag, and a tap ==")
	var a: Vector2 = to_screen.call(110.0, 78.0)
	var b: Vector2 = to_screen.call(146.0, 90.0)
	var vr2: Rect2 = vp.get_global_rect()
	_ok(vr2.has_point(a) and vr2.has_point(b) and vr2.has_point(a + Vector2(0, 70)) and vr2.has_point(a + Vector2(30, -70)) and vr2.has_point(a + Vector2(30, 90)), "the finger path, the tap and the two-finger points lie on screen (%s to %s)" % [str(a), str(b)])
	await _finger_drag(a, b)
	var painted := _REGION.dabs.size()
	_ok(painted >= 3, "one finger dragged across the map painted dabs (%d)" % painted)
	_ok(_REGION.strokes.size() == 1, "...as ONE stroke (%d)" % _REGION.strokes.size())
	var last: Vector3 = _REGION.dabs[_REGION.dabs.size() - 1]
	_ok(absf(last.x - 146.0) <= 8.0 and absf(last.y - 90.0) <= 8.0, "...the last dab is at the lift point (%s, about 146, 90)" % str(last))
	_ok((vp._camera.position - cam0).length() < 1.0, "the stroke did not pan the map")
	_ok(vp.tool_overlay.assist_dabs.size() == painted, "the overlay draws exactly the dabs (%d)" % vp.tool_overlay.assist_dabs.size())
	await _shot("4_finger")
	var tap: Vector2 = a + Vector2(0, 70)
	await _touch(0, tap, true)
	await _touch(0, tap, false)
	_ok(_REGION.dabs.size() == painted + 1 and _REGION.strokes.size() == 2, "a tap painted one dab as its own stroke (%d dabs, %d strokes)" % [_REGION.dabs.size(), _REGION.strokes.size()])

	print("== 5. two fingers do not paint ==")
	var c: Vector2 = a + Vector2(30, -70)
	await _touch(0, c, true)
	await _touch(0, c + Vector2(14, 5), null, Vector2(14, 5))
	var mid := _REGION.dabs.size()
	var f2: Vector2 = a + Vector2(30, 90)
	await _touch(1, f2, true)
	for k in range(1, 5):
		await _touch(1, f2 + Vector2(0, 10 * k), null, Vector2(0, 10))
		await _touch(0, c + Vector2(14, 5) + Vector2(0, -8 * k), null, Vector2(0, -8))
	await _touch(1, f2 + Vector2(0, 40), false)
	await _touch(0, c + Vector2(14, -27), false)
	_ok(_REGION.dabs.size() == mid, "a second finger landing mid-stroke added no dabs (%d before, %d after)" % [mid, _REGION.dabs.size()])

	print("== 5b. a lost release does not glue the next stroke on ==")
	## The two-finger gesture above leaves its first finger's stroke open (the host's guard emits no release for it, see
	## `llm_region.gd::_drag`); after a pause longer than `STROKE_GAP_MS` (400) the next activity starts a fresh stroke.
	await get_tree().create_timer(0.5).timeout
	var strokes0 := _REGION.strokes.size()
	_app._drag_handlers[_REGION.TOOL_ID].call(110.0, 80.0)  ## a stroke opened by a drag handler whose release never comes
	_ok(_REGION._stroke_open and _REGION.strokes.size() == strokes0 + 1, "control: a stroke is open with no release (open %s, %d strokes, wanted %d)" % [str(_REGION._stroke_open), _REGION.strokes.size(), strokes0 + 1])
	await get_tree().create_timer(0.6).timeout  ## longer than `STROKE_GAP_MS` (400)
	await _finger_drag(a + Vector2(-40, 0), a + Vector2(40, 30))
	_ok(_REGION.strokes.size() == strokes0 + 2, "the next finger drag is a NEW stroke, not glued on (%d strokes, wanted %d)" % [_REGION.strokes.size(), strokes0 + 2])
	_REGION.undo_stroke(_app)
	_REGION.undo_stroke(_app)

	print("== 6. Undo stroke and Done ==")
	var before := _REGION.strokes.size()
	_REGION.undo_stroke(_app)
	_ok(_REGION.strokes.size() == before - 1, "Undo stroke removed one whole stroke (%d to %d)" % [before, _REGION.strokes.size()])
	var done_b := vp.get_node("AssistRegionBar").find_child("assist_done", true, false) as Button
	await _click(done_b)
	await _settle(10)
	_ok(not _REGION.painting and _app.armed_tool == "inspect" and vp.get_node_or_null("AssistRegionBar") == null, "Done (a real click) disarmed the brush and removed the bar (armed %s)" % _app.armed_tool)
	_ok(_win._task == "edit", "...and the Assistant is back on the Edit task")
	_ok(not _REGION.dabs.is_empty(), "...with the painted region kept (%d dabs)" % _REGION.dabs.size())
	await _shot("6_done")
	_finish()


func _all_labels(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	for ch in n.get_children(true):
		_all_labels(ch, out)
