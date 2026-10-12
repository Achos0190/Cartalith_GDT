extends Node
## Protects: GF-13's "the stage-06 age row is seen rendered in a running shell
## ... now reading the model-time wording under BM" (`GEOLOGY_FIRST_SCOPE.md`
## §7 GF-13; §5.8 left the row parse-checked and unit-tested but never drawn).
## Boots the real shell, generates a small world with the app's defaults, finds
## the `geological age ×` row that `world_workspace.gd`'s
## `_attach_geo_age_readout` built, makes its ancestors visible, and captures
## the row and its readout line from the SCREEN in two states:
##   1. app defaults (processes off): the slider is not editable, the row is
##      dimmed to `WS_OVERRIDE_DIM`, and the note starts with the engine's
##      inert reason (`GEO_CLOCK_INERT_REASON`, `erode_bridge.rs`) and lists
##      the stream-power count it would run;
##   2. `geology_processes` and `relief_model` set on the live `WorldGen`
##      (no regenerate; the readout reads the live parameters) and the slider
##      moved to 2.0: editable, undimmed, and the note reads the construction
##      clause `geo_clock_readout_parts` writes under the relief switch.
## Positive control: the two crops must differ (a screen read that cannot see
## the change would pass state 2 vacuously).
##
## MUST run WINDOWED from a scratch copy (`ImageTexture`/viewport reads are
## dead under `--headless`, `MISTAKES.md`), after the DLL is rebuilt from the
## `.rs` that carries the construction clause:
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _gf13agerow_probe.tscn -- --out DIR
## `--out DIR` (optional, default `user://gf13agerow`) names where the two
## crops and the two full frames go. An unknown argument aborts (exit 2).
## Exit status: 0 every check passed; 1 a check failed; 2 could not run;
## 3 watchdog.

var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	print("[%s] %s" % ["PASS" if ok else "FAIL", what])

## The world workspace is the node whose script declares `_geo_age_slider`;
## found by property rather than by path so a dock re-parenting does not
## silently point the probe at nothing.
func _find_ws(n: Node) -> Node:
	if "_geo_age_slider" in n and n.get("_geo_age_slider") != null:
		return n
	for c in n.get_children():
		var r := _find_ws(c)
		if r != null:
			return r
	return null

## Screen crop around the row and its note (their union, padded), saved with
## the full frame beside it so a reviewer can see where the crop sits.
func _capture(ctls: Array, out_dir: String, name: String) -> Image:
	await RenderingServer.frame_post_draw
	var frame := get_viewport().get_texture().get_image()
	frame.save_png("%s/%s_frame.png" % [out_dir, name])
	var r := Rect2()
	var first := true
	for c in ctls:
		var g := (c as Control).get_global_rect()
		r = g if first else r.merge(g)
		first = false
	r = r.grow(12).intersection(Rect2(Vector2.ZERO, Vector2(frame.get_size())))
	var crop := frame.get_region(Rect2i(r))
	var path := "%s/%s_row.png" % [out_dir, name]
	crop.save_png(path)
	print("[OUT] %s (rect %s)" % [ProjectSettings.globalize_path(path), r])
	return crop

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("[WATCHDOG] timed out"); get_tree().quit(3))
	wd.start()

	var out_dir := "user://gf13agerow"
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--out" and i + 1 < args.size():
			out_dir = args[i + 1]
			i += 2
		else:
			print("[FATAL] unknown argument %s (read: --out DIR)" % args[i]); get_tree().quit(2); return
	DirAccess.make_dir_recursive_absolute(out_dir)

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(2); return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	app.call("_on_rail_node_pressed", "world", "a")
	await _frames(10)
	var bridge = app.get("bridge")
	bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": 512, "grid_h": 328})
	var waited := 0
	while bridge.generating and waited < 3600:
		await _frames(1)
		waited += 1
	await _frames(10)
	_check(bool(bridge.has_world), "a world exists after generate (waited %d frames)" % waited)

	var ws := _find_ws(app)
	if ws == null:
		print("[FATAL] no node carries a built geological-age row"); get_tree().quit(2); return
	var s: HSlider = ws.get("_geo_age_slider")
	var note: Label = ws.get("_geo_age_note")
	var row: Control = ws.call("_current_row_ctl", ws.get("_geo_age_row"))
	# Open every collapsed ancestor so the row is on screen, then scroll to it.
	var a: Node = s
	while a != null and a != app:
		if a is CanvasItem:
			(a as CanvasItem).visible = true
		a = a.get_parent()
	await _frames(4)
	var sc: Node = s.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(note)
	await _frames(6)
	_check(s.is_visible_in_tree() and note.is_visible_in_tree(), "the age row and its note are visible in the tree")
	_check(get_viewport().get_visible_rect().encloses(note.get_global_rect()), "the note lies inside the window")

	# State 1: the app's defaults.
	var wg = bridge.world_gen
	var p: Dictionary = wg.get_params()
	_check(p.get("geology_processes") == false and p.get("relief_model") == false,
		"state 1 runs the app defaults (processes %s, relief %s)" % [p.get("geology_processes"), p.get("relief_model")])
	var crop1: Image = await _capture([row, note], out_dir, "state1_app")
	print("[NOTE1] ", note.text)
	_check(not s.editable, "state 1: the slider is not editable")
	_check(note.text.begins_with("Inert in this build:"), "state 1: the note opens with the inert reason")
	_check("stream power 9 it" in note.text, "state 1: the note lists the stream-power count it would run")
	_check(row != null and absf(row.modulate.a - 0.55) < 0.01, "state 1: the row is dimmed to 0.55 (read %s)" % (row.modulate.a if row != null else -1.0))

	# State 2: processes and relief on, age 2. No regenerate: the readout is a
	# read of the live parameters.
	var rep: Dictionary = wg.set_params({"geology_processes": true, "relief_model": true})
	_check((rep.get("rejected", PackedStringArray()) as PackedStringArray).is_empty(), "set_params took both switches: %s" % [rep])
	ws.call("_refresh_geo_age_row")
	s.value = 2.0
	await _frames(6)
	var crop2: Image = await _capture([row, note], out_dir, "state2_relief")
	print("[NOTE2] ", note.text)
	_check(s.editable, "state 2: the slider is editable")
	_check(note.text.begins_with("construction ×2.00 (in place of stream power)"), "state 2: the note reads the construction clause at x2.00")
	_check(row != null and absf(row.modulate.a - 1.0) < 0.01, "state 2: the row is not dimmed")
	# Positive control: the screen must show the difference. The two crops
	# differ in height (the inert note wraps to more lines), so they are
	# compared over their common top-left area, which holds the row and the
	# note's first line in both.
	var moved := 0
	for y in mini(crop1.get_height(), crop2.get_height()):
		for x in mini(crop1.get_width(), crop2.get_width()):
			var c1 := crop1.get_pixel(x, y)
			var c2 := crop2.get_pixel(x, y)
			if absf(c1.r - c2.r) + absf(c1.g - c2.g) + absf(c1.b - c2.b) > 24.0 / 255.0:
				moved += 1
	_check(moved > 200, "positive control: the two screen crops differ (%d px moved)" % moved)

	print("[DONE] %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
