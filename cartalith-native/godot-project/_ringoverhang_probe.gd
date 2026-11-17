extends Node
## Probe: a real left click on a part of the sticky cartwheel that hangs over a dock panel reaches the RING, not the
## panel (`app.gd::_input`, `context_broker.gd::ring_click_over_part`, `radial_ring.gd::hits_part`). CQ-4's
## "found, not fixed" (`CUSTOM_QUICK_BAR_SCOPE.md` s.14), closed 2026-11-17.
##
##   desktop: godot --path . --resolution 1600x900  _ringoverhang_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1700x1000 _ringoverhang_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   phone:   the phone has no cartwheel (MOBILE_SHELL_PROPOSAL.md); the probe says so and passes only the structural checks.
##
## **Windowed, never `--headless`** (real input events, the hovered control, the framebuffer). Run from a scratch
## worktree with a wiped user dir and a scratch asset library.
##
## What it protects, against the real shell and real `Input.parse_input_event` presses:
##   1. an OVERHANG exists: a ring slot whose centre has a control other than the map overlay under it (found by
##      scanning the ring centre across the window; the probe fails if the layout offers none, so it cannot pass
##      vacuously);
##   2. a real click on that slot runs its callable and closes the ring (before the fix the dock panel took the press
##      and the ring stayed open);
##   3. the same for a dock chip that overhangs a panel;
##   4. a real click on the same panel at a point that is NOT on a ring part still reaches the panel and leaves the
##      ring open (the fix must not swallow the panel);
##   5. a non-sticky ring (not yet released) takes no press from `_input`.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const SEED := 483920
var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _flag := ""


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("ro_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _move_to(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)


func _button(at: Vector2, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	ev.button_mask = 1 if down else 0
	Input.parse_input_event(ev)


func _click_at(at: Vector2) -> void:
	_move_to(at)
	await _settle(2)
	_button(at, true)
	await _settle(2)
	_button(at, false)
	await _settle(4)


func _broker():
	return _app.context_broker


func _state() -> Dictionary:
	var r = _broker().ring
	return r.debug_state() if r != null else {}


func _local(global: Vector2) -> Vector2:
	return _app.viewport.overlay.get_global_transform_with_canvas().affine_inverse() * global


## The ring rows used: a W slot and an E slot that set `_flag`, and a dock of two chips that do the same.
func _slots() -> Dictionary:
	return {
		"W": {"label": "Probe W", "enabled": true, "callable": func() -> void: _flag = "W"},
		"E": {"label": "Probe E", "enabled": true, "callable": func() -> void: _flag = "E"},
	}


func _dock() -> Array:
	return [
		{"title": "Dock A", "label": "Dock A", "enabled": true, "callable": func() -> void: _flag = "dockA"},
		{"title": "Dock B", "label": "Dock B", "enabled": true, "callable": func() -> void: _flag = "dockB"},
	]


## Opens a STICKY ring (Q form, then released with nothing hovered) at `global`.
func _open_sticky(global: Vector2, dock: Array) -> Dictionary:
	if _broker().ring != null:
		_broker().ring_close()
	await _settle(2)
	_flag = ""
	_broker()._ensure_ring().arm(global, _slots(), true, dock)
	await _settle(2)
	_broker().ring.release()
	await _settle(3)
	return _state()


func _slot_global(st: Dictionary, dir: String) -> Vector2:
	var a := deg_to_rad(0.0 if dir == "E" else 180.0)
	return (st["centre"] as Vector2) + Vector2(cos(a), sin(a)) * float(st["radius"])


## True when the control under the pointer at `at` is neither the map overlay nor inside it.
func _over_other_control(at: Vector2) -> bool:
	_move_to(at)
	await _settle(2)
	var c := get_viewport().gui_get_hovered_control()
	var ov: Control = _app.viewport.overlay
	return c != null and c != ov and not ov.is_ancestor_of(c)


func _hovered_name(at: Vector2) -> String:
	var c := get_viewport().gui_get_hovered_control()
	return str(c.get_path()) if c != null else "<none>"


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch" or a.begins_with("--density"):
			pass
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	if _mode == "phone":
		_ok(true, "the phone has no cartwheel, so there is no ring to overhang a panel (structural: broker exists %s)" % str(_broker() != null))
		_finish()
		return
	_app.bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)

	var vp := get_viewport().get_visible_rect().size
	var cy := vp.y * 0.45
	# -- 1/2. a slot that overhangs a panel ------------------------------------------------------
	var found := {}
	var x := 40.0
	while x < vp.x - 40.0 and found.is_empty():
		var st := await _open_sticky(Vector2(x, cy), [])
		for dir in ["W", "E"]:
			var p := _slot_global(st, dir)
			if await _over_other_control(p):
				found = {"x": x, "dir": dir, "name": _hovered_name(p)}
				break
		x += 24.0
	_ok(not found.is_empty(), "a ring slot that overhangs another control exists in this layout: %s" % str(found))
	if not found.is_empty():
		var st := await _open_sticky(Vector2(float(found["x"]), cy), [])
		var p := _slot_global(st, String(found["dir"]))
		_ok(bool(st.get("sticky", false)) and bool(st.get("active", false)), "the ring is open and sticky")
		await _shot("slot_before_click")
		await _click_at(p)
		_ok(_flag == String(found["dir"]), "a real click on the overhanging %s slot ran its callable (flag '%s')" % [found["dir"], _flag])
		_ok(not bool(_state().get("active", true)), "and closed the ring")
		_broker().ring_close()
	# -- 3. a dock chip that overhangs a panel ---------------------------------------------------
	var found_dock := {}
	x = 40.0
	while x < vp.x - 40.0 and found_dock.is_empty():
		var st := await _open_sticky(Vector2(x, cy), _dock())
		var rects: Array = st["dock_rects"]
		for k in rects.size():
			var c: Vector2 = (rects[k] as Rect2).get_center()
			## Outside the map overlay's own rectangle, so the press cannot reach `map_overlay.gd::_gui_input`
			## (the hovered control is not used here: it reads `null` over the tablet's lower strip).
			if not (_app.viewport.overlay as Control).get_global_rect().has_point(c):
				found_dock = {"x": x, "k": k}
				break
		x += 24.0
	_ok(not found_dock.is_empty(), "a dock chip that overhangs another control exists in this layout: %s" % str(found_dock))
	if not found_dock.is_empty():
		var st := await _open_sticky(Vector2(float(found_dock["x"]), cy), _dock())
		var c: Vector2 = ((st["dock_rects"] as Array)[int(found_dock["k"])] as Rect2).get_center()
		await _shot("dock_before_click")
		await _click_at(c)
		_ok(_flag == ("dockA" if int(found_dock["k"]) == 0 else "dockB"), "a real click on the overhanging dock chip ran it (flag '%s')" % _flag)
		_ok(not bool(_state().get("active", true)), "and closed the ring")
	# -- 5. not sticky: no press taken -----------------------------------------------------------
	if _broker().ring != null:
		_broker().ring_close()
	await _settle(2)
	_broker()._ensure_ring().arm(Vector2(vp.x * 0.5, cy), _slots(), true, [])  ## Q form: open and drawn, but not sticky yet
	await _settle(2)
	_ok(bool(_state().get("active", false)) and not bool(_state().get("sticky", true)), "precondition: the ring is open and not sticky")
	_ok(not _broker().ring_click_over_part(Vector2(vp.x * 0.5, cy)), "a ring that is open but not sticky yet (Q held) takes no press from the panel path")
	_broker().ring_close()
	# -- 4. a click on the panel that is NOT on a ring part. Last: a panel click may open a window.
	if not found.is_empty():
		var st4 := await _open_sticky(Vector2(float(found["x"]), cy), [])
		var anchor := _slot_global(st4, String(found["dir"]))
		var away := Vector2(anchor.x, cy + float(st4["disc_radius"]) + 120.0)
		var on_panel: bool = not (_app.viewport.overlay as Control).get_global_rect().has_point(away)
		_ok(not _broker().ring.hits_part(away), "a point %.0f px below the ring is not a ring part" % (away.y - cy))
		_ok(on_panel, "and it lies over a panel, outside the map overlay (so the click below is the panel's to take)")
		var open_before := bool(_state().get("active", false))
		await _click_at(away)
		_ok(open_before and _flag == "", "a click on the panel beside the ring is not taken by the ring: no callable ran")
		_ok(bool(_state().get("active", false)), "and the ring is still open (the panel took the click, as before)")
		if _broker().ring != null:
			_broker().ring_close()
	_finish()
