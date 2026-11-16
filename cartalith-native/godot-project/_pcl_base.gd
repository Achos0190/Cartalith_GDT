extends Node
## Shared harness of the desktop-shell leftovers probes (`_pcl_*_probe.gd`, GUI_PLANS_AUDIT_2026-11-16.md PC rows).
## Each probe `extends "res://_pcl_base.gd"` and overrides `_run()`. This file boots the real shell (`app.tscn`),
## generates one small world (seed 4242, 256 x 192), draws one conflict ("Probe War", years 250 to 260, attached
## to a settlement of the first faction), and offers real-input helpers: `_click_at()` (a real mouse press through
## `Input.parse_input_event`), `_key()` (a real key), `_press()` (scroll a control into its scroll and click its
## centre; fails the check when the centre is clipped), `_shot()`.
##
##   godot --path . --resolution 1600x900 _pcl_xxx_probe.tscn -- [--force-touch] [--shots <dir>]
##
## **Windowed, never `--headless`**, under `C:/cw/.win_lock`, with a scratch APPDATA (`MISTAKES.md`). Reads
## `--force-touch` (accepted; the shell reads it itself) and `--shots <dir>`; anything else exits 2.
## Exit 0 pass, 1 a check failed, 2 bad argument, watchdog or no world.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _prefix := "pcl"
## The conflict this harness draws: its id, and the faction ids it sets as sides.
var _cid := -1
var _f1 := 1
var _f2 := 2
var _anchor := 0


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
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var path := _shots.path_join("%s_%s_%s.png" % [_prefix, _mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
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
	await _settle(4)


func _key(code: Key, ctrl := false, shift := false, alt := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.shift_pressed = shift
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


## Every ScrollContainer above `c`, innermost first.
func _scrolls_of(c: Control) -> Array:
	var out: Array = []
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			out.append(n)
		n = n.get_parent()
	return out


## Scrolls `c` into view in EVERY scroll container above it (innermost first) and clicks its centre for real. Fails the check
## (returns false) when the centre is not inside the INTERSECTION of those scrolls' visible rects (a control clipped by an outer
## scroll while inside an inner one is not clickable: a first version tested the inner scroll alone and clicked the panel's
## header instead, 2026-11-16) and the window or viewport.
func _press(c: Control, what: String) -> bool:
	if c == null or not is_instance_valid(c):
		_ok(false, "%s exists to be pressed" % what)
		return false
	for sc in _scrolls_of(c):
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(3)
	## A control inside an embedded `Window` reports its rect in THAT window's space; the press must go to the
	## screen point, which is the window's own position plus it (`_pc7windows_probe.gd::_in_win`).
	var win := c.get_window()
	var at := c.get_global_rect().get_center()
	var vis := Rect2(Vector2.ZERO, win.get_visible_rect().size if win != null else get_viewport().get_visible_rect().size)
	for sc in _scrolls_of(c):
		vis = vis.intersection((sc as ScrollContainer).get_global_rect())
	var inside := vis.has_point(at) and c.is_visible_in_tree()
	## The press goes to the control's centre in MAIN-WINDOW pixels: the screen transform folds in an embedded window's
	## position and any content scale (the phone's UI is scaled by short side / 412; an unscaled point missed the button).
	var screen_at: Vector2 = c.get_screen_transform() * (c.size * 0.5)
	at = screen_at
	_ok(inside, "%s is drawn inside its visible rect (centre %s in %s)" % [what, str(c.get_global_rect().get_center()), str(vis)])
	if not inside:
		return false
	await _click_at(at)
	return true


func _year() -> int:
	return int(_bridge.get_civ_year())


func _boot() -> bool:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch" or a.begins_with("--density=") or a == "--phone":
			pass   # read by the shell itself
		elif a == "--density" and k + 1 < args.size():
			k += 1
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return false
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return false
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s ==" % _mode)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 3 or _bridge.get_factions().size() < 2:
		print("[FATAL] no world with three settlements and two factions")
		get_tree().quit(2)
		return false
	var facs: Array = _bridge.get_factions()
	_f1 = int((facs[0] as Dictionary).get("id", 1))
	_f2 = int((facs[1] as Dictionary).get("id", 2))
	var all: Array = _bridge.settlements()
	_anchor = 0
	for i in all.size():
		if int((all[i] as Dictionary).get("faction", 0)) == _f1:
			_anchor = i
			break
	var a_s: Dictionary = all[_anchor]
	var r: Dictionary = _bridge.conflict_add({"kind": "battle",
		"points": PackedVector2Array([Vector2(float(a_s.get("x", 0)), float(a_s.get("y", 0)))]),
		"start_year": 250})
	_cid = int(r.get("id", -1))
	_bridge.conflict_update(_cid, {"name": "Probe War", "sides": PackedInt32Array([_f1, _f2]), "end_year": 260,
		"anchor_kind": "settlement", "anchor_tid": int(a_s.get("tid", 0))})
	_ok(bool(r.get("ok", false)) and _cid >= 0, "a conflict was drawn for the harness (id %d)" % _cid)
	_app.select_domain("civilization")
	await _settle(8)
	return true


func _run() -> void:
	pass


func _ready() -> void:
	if not await _boot():
		return
	await _run()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
