extends Node
## **The carved cell beside water** (`OUTSTANDING_WORK.md` row "The
## colour-texture repaint blocks the main thread ...; and the carved cell
## beside water keeps its step", second half, 2026-09-28). Frames the mouths
## of the largest rivers that reach the sea and saves them for looking, before
## and after RV-3's ring of land beside water was unfrozen. Measurement only;
## it changes nothing but its own world.
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _mouthstep_probe.tscn -- --out DIR [--seed 24601] [--mouths 3]
##
## Every flag above is read in `_ready`; an unknown one aborts.
##
## A mouth is the last traced cell of a river run (`bridge.rivers(1)`, the
## `points` key) that sits on land 4-adjacent to ocean (`sample_cell`'s
## `water`); the `--mouths` runs with the most discharge are framed. Each is
## saved at x8 and x32 with the Rivers layer off (the stroke would cover the
## ground being looked at) and on, every other overlay hidden:
## `s<seed>_m<k>_z<zoom>_<off|on>.png`. The frame's centre is the mouth cell.

const ZOOMS := [8.0, 32.0]
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]

var _out := "user://mouthstep/"
var _seed := 24601
var _mouths := 3
var _app: Node
var _vh: Control
var _br: Node


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--mouths": _mouths = int(args[i + 1]); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().quit(await _run())


func _water(x: int, y: int) -> String:
	return String(_br.sample_cell(x, y).get("water", "land"))


func _run() -> int:
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": 1024, "grid_h": 656,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _br.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if not _br.has_world:
		print("PROBE-RESULT: ABORT no world"); return 2
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	var found := []
	for r: Dictionary in _br.rivers(1):
		var pts: PackedVector2Array = r["points"]
		if pts.size() < 4:
			continue
		var m := pts[pts.size() - 1]
		var x := int(m.x); var y := int(m.y)
		if _water(x, y) != "land":
			continue
		var coast := false
		for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			if _water(x + d.x, y + d.y) == "ocean":
				coast = true
		if coast:
			found.append({"q": float(r.get("discharge", 0.0)), "x": x, "y": y})
	found.sort_custom(func(a, b): return a["q"] > b["q"])
	print("  mouths on the sea: %d" % found.size())
	if found.is_empty():
		print("PROBE-RESULT: ABORT no river mouth on the sea"); return 2
	for k in mini(_mouths, found.size()):
		var m: Dictionary = found[k]
		for z in ZOOMS:
			for on in [false, true]:
				_vh.set_layer_visible("rivers", on)
				_vh.overlay.set_labels([])
				_vh.reset_view()
				await _settle(3)
				_vh.zoom_step(z / _vh.zoom())
				_vh.move_view_to(m["x"] + 0.5, m["y"] + 0.5)
				await _settle(20)
				var t0 := Time.get_ticks_msec()
				while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
					await get_tree().process_frame
				await _settle(10)
				await RenderingServer.frame_post_draw
				var img := get_viewport().get_texture().get_image()
				var gp := _vh.global_position
				var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
				var tag := "s%d_m%d_z%02d_%s" % [_seed, k, int(z), "on" if on else "off"]
				img.get_region(r).save_png(_out.path_join(tag + ".png"))
				print("  saved %s at mouth (%d, %d)" % [tag, m["x"], m["y"]])
	print("PROBE-RESULT: DONE")
	return 0


func _settle(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw
