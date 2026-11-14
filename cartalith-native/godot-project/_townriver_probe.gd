extends Node
## The owner's town-on-its-river report (2026-11-14: *"The river passes below it. It's not on the river. And for the
## same matter double check this on the bigger map that it sits correctly."*): frames of the real shell, the same
## settlement in the City Viewer and on the world map, for `docs/evidence/town-river-2026-11-14/`.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _townriver_probe.tscn -- --out <dir> [--seed N] [--viewer I] [--crops I,J,K] [--zoom Z] [--front 0|1] [--grid WxH]
##
## **Windowed, never `--headless`** (it reads rendered pixels). Defaults: seed 483920 at 384 x 288 over 800 km (`--grid` for another, e.g. the app's 2048 x 1311) (the
## grid `_wallhug_probe.gd` sent the owner's frame from), City Viewer for settlement 6 in the map look, world-map
## crops of settlement 6 at x16.
##
## What it checks (each names itself): the world generated, the City Viewer frame drew (many distinct colours), and
## each map crop drew. The comparison it exists for is by eye and by `town_river_census` (Rust), which measures the
## same towns: the frames are the evidence a number cannot give.
## Frames: `<out>/viewer_<seed>_<i>.png`, `<out>/map_<seed>_<i>_x<z>.png`.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

var _out := ""
var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _distinct(img: Image) -> int:
	var seen := {}
	for y in range(0, img.get_height(), 3):
		for x in range(0, img.get_width(), 3):
			var c := img.get_pixel(x, y)
			seen[Vector3i(int(c.r * 15.0), int(c.g * 15.0), int(c.b * 15.0))] = true
	return seen.size()

func _shot(rect: Rect2) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	return img.get_region(Rect2i(rect).intersection(Rect2i(Vector2i.ZERO, img.get_size())))

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("townriver probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_townriver_probe must be windowed")
		get_tree().quit(2)
		return
	var seed_n := 483920
	var viewer := 6
	var crops: Array[int] = [6]
	var zoom := 16.0
	var front := -1
	var gw := 384
	var gh := 288
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]
			"--seed":
				seed_n = int(args[i + 1])
			"--viewer":
				viewer = int(args[i + 1])
			"--crops":
				crops.clear()
				for t in String(args[i + 1]).split(",", false):
					crops.append(int(t))
			"--zoom":
				zoom = float(args[i + 1])
			"--front":
				front = int(args[i + 1])
			"--grid":
				var wh := String(args[i + 1]).split("x")
				gw = int(wh[0])
				gh = int(wh[1])
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	if _out == "":
		push_error("--out <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 1000)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	## `--front 0|1` sets Ruling EX's river frontage term (`civ.river_front`) before the world is made: 0 keeps the
	## placements the owner's frame came from, 1 is the shipped app. Unset leaves the app's own default. An engine
	## without the parameter (the before build) ignores it.
	if front >= 0 and not bridge.param_info("civ.river_front").is_empty():
		bridge.param_set("civ.river_front", front == 1)
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	var places: Array = bridge.settlements()
	_ok(places.size() > maxi(viewer, 0), "the world generated (%d settlements)" % places.size())

	## Every town of at least 3 000 people, with what its layout was given: the list a frame is chosen from.
	for k in places.size():
		if int(places[k].get("population", 0)) >= 3000:
			var lt: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
			## Market to the layout's river, in metres (`Geometry2D` on each segment); `-` without a river of two points.
			var rv: PackedVector2Array = lt.get("river", PackedVector2Array())
			var mk: Vector2 = lt.get("market", Vector2.ZERO)
			var dmin := INF
			for j in rv.size() - 1:
				dmin = minf(dmin, mk.distance_to(Geometry2D.get_closest_point_to_segment(mk, rv[j], rv[j + 1])))
			print("  town #%d cell (%d,%d) pop %d site_kind %s uses_real_water %s market-to-river %s m" % [k,
				int(places[k]["x"]), int(places[k]["y"]), int(places[k]["population"]), lt.get("site_kind", "?"),
				lt.get("uses_real_water", "?"), ("%.0f" % dmin) if dmin < INF else "-"])

	var host: Control = _app.viewport
	var ov: Control = host.overlay
	## The region names are cleared for the crops: at x16 a region label is larger than the town and covers it. The
	## crops are taken before the City Viewer opens, so no selection ring is drawn round the marker either.
	ov.set_labels([])
	for k in crops:
		if k < 0 or k >= places.size():
			continue
		var lk: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		print("\n== world map, settlement #%d at x%d (cell %d,%d, site_kind %s, uses_real_water %s) ==" % [k, int(zoom),
			int(places[k]["x"]), int(places[k]["y"]), lk.get("site_kind", "?"), lk.get("uses_real_water", "?")])
		host.reset_view()
		await _frames(6)
		var s: Dictionary = places[k]
		## `_cell_to_screen` is where the overlay draws the marker (the cell centre), in the overlay's own space.
		var at_ov: Vector2 = ov._cell_to_screen(Vector2(float(s["x"]), float(s["y"])), ov._displayed_rect())
		var at_global: Vector2 = ov.get_global_transform_with_canvas() * at_ov
		var local: Vector2 = host.get_global_transform_with_canvas().affine_inverse() * at_global
		host._zoom_at(local, zoom / host.zoom())
		await _frames(4)
		## Then pan so the marker sits at the middle of the map view, clear of the bars at its edges.
		var at1: Vector2 = host.get_global_transform_with_canvas().affine_inverse() * (ov.get_global_transform_with_canvas() * at_ov)
		host._camera.position += host.size * 0.5 - at1
		await _frames(20)
		var at2: Vector2 = ov.get_global_transform_with_canvas() * at_ov
		var im2 := await _shot(Rect2(at2 - Vector2(300, 300), Vector2(600, 600)))
		im2.save_png(_out.path_join("map_%d_%d_x%d.png" % [seed_n, k, int(zoom)]))
		print("  marker at %s (cell %d,%d), zoom %.1f" % [at2, int(s["x"]), int(s["y"]), host.zoom()])
		_ok(_distinct(im2) > 20, "the map crop of #%d drew (%d distinct colours)" % [k, _distinct(im2)])

	if viewer >= 0 and viewer < places.size():
		print("\n== City Viewer, settlement #%d (map look) ==" % viewer)
		## What the layout was given: its site kind, whether its water is the map's own, and its river.
		var lay: Dictionary = bridge.urban_layouts(PackedInt32Array([viewer]))[0]
		var riv: PackedVector2Array = lay.get("river", PackedVector2Array())
		print("  cell (%d,%d) site_kind %s uses_real_water %s river %d pts %s..%s market %s" % [
			int(places[viewer]["x"]), int(places[viewer]["y"]), lay.get("site_kind", "?"), lay.get("uses_real_water", "?"),
			riv.size(), riv[0] if riv.size() > 0 else Vector2.INF, riv[riv.size() - 1] if riv.size() > 0 else Vector2.INF,
			lay.get("market", "?")])
		_app.open_city_viewer(viewer)
		await _frames(12)
		var cv = _app.city_viewer_window
		cv.size = Vector2i(1560, 960)
		cv.position = Vector2i(20, 20)
		await _frames(8)
		cv._look_picker.select(0)
		cv._look_picker.item_selected.emit(0)
		await _frames(3)
		cv._reset_view()
		await _frames(6)
		var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
		var im := await _shot(rect)
		im.save_png(_out.path_join("viewer_%d_%d.png" % [seed_n, viewer]))
		_ok(_distinct(im) > 40, "the City Viewer frame drew (%d distinct colours)" % _distinct(im))
		cv.hide()
		await _frames(4)

	_finish()

func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
