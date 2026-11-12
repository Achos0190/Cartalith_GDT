extends Node
## Ruling EU item 1 (owner, 2026-11-12): routes that come close should merge
## into one way. Measures the one number the Rust census
## (`cartalith-godot/src/way_census.rs`) cannot know -- how many SCREEN pixels
## one map cell is at the opening view -- and saves the frames the change is
## judged on: the whole map at the opening view, then x4 and x16 centred on
## `--focus`.
##
## It reads the opening view's scale off the overlay's own two projections
## (`map_overlay.gd::_cell_to_screen` through the overlay's canvas transform),
## the same technique `_roadsettle_probe.gd` uses, so the figure is what is
## drawn, not a formula.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _waybundle_probe.tscn -- --out DIR [options]
##
## Arguments (an unknown one stops the probe with exit 2):
##   --out DIR            where the PNGs land (required)
##   --tag NAME           file-name prefix (default "wb")
##   --seed N             world seed (default 246371)
##   --grid WxH           world size (default 2048x1311)
##   --km K               map width in km (default 40075)
##   --river-density D    (default 1.55)
##   --metropolis         metropolis on (default off)
##   --merge on|off       `civ.merge_ways` for this generation (default on)
##   --focus X,Y          grid point the x4/x16 frames centre on (default the map centre)
##
## Prints `WAYBUNDLE  <text>` lines and ends `WAYBUNDLE  ---- done ----`.
## Exit 0 when it measured a positive scale over a world with roads; 2 when it
## cannot run (headless, a bad argument, no roads, the 900 s watchdog), with
## `PROBE-CANNOT-RUN: ...` on stderr.

var GRID := Vector2i(2048, 1311)
var _seed := 246371
var _km := 40075.0
var _density := 1.55
var _metropolis := false
var _merge := true
var _out := ""
var _tag := "wb"
var _focus := Vector2.INF
var _app: Node
var _vh: Control
var _br: Node


func _p(s: String) -> void:
	print("WAYBUNDLE  " + s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]; i += 1
			"--tag":
				_tag = args[i + 1]; i += 1
			"--seed":
				_seed = int(args[i + 1]); i += 1
			"--grid":
				var g := args[i + 1].split("x"); GRID = Vector2i(int(g[0]), int(g[1])); i += 1
			"--km":
				_km = float(args[i + 1]); i += 1
			"--river-density":
				_density = float(args[i + 1]); i += 1
			"--metropolis":
				_metropolis = true
			"--merge":
				_merge = args[i + 1] == "on"; i += 1
			"--focus":
				var f := args[i + 1].split(","); _focus = Vector2(float(f[0]), float(f[1])); i += 1
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument %s" % args[i])
				return false
		i += 1
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


## Screen pixels per map cell at the current view, read through the overlay's
## own projection: the screen distance between cells (0, 0) and (100, 0),
## divided by 100.
func _px_per_cell() -> float:
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var a: Vector2 = xf * ov._cell_to_screen(Vector2(0, 0), rect)
	var b: Vector2 = xf * ov._cell_to_screen(Vector2(100, 0), rect)
	return a.distance_to(b) / 100.0


func _shot(name: String) -> void:
	var img := await _grab()
	if img == null:
		_p("%s: no frame (null viewport image)" % name)
		return
	var path := "%s/%s_%s.png" % [_out, _tag, name]
	img.save_png(path)
	_p("saved %s (%dx%d)" % [path, img.get_width(), img.get_height()])


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_br.param_set("river_density", _density)
	_br.param_set("civ.merge_ways", _merge)
	_br.generate({"seed": _seed, "width_km": _km, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "metropolis": _metropolis, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 3200:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	var roads: Array = _br.roads()
	if roads.is_empty():
		printerr("PROBE-CANNOT-RUN: the world has no roads")
		get_tree().quit(2)
		return
	_vh.set_layer_visible("settlements", true)
	_vh.set_layer_visible("roads", true)
	_vh.reset_view()
	await _settle()
	var ppc := _px_per_cell()
	var host: Vector2 = _vh.size
	var ft: float = get_viewport().get_final_transform().x.x
	_p("merge=%s seed=%d grid=%dx%d roads=%d" % [str(_merge), _seed, GRID.x, GRID.y, roads.size()])
	_p("fit px_per_cell=%.4f viewport_host=%dx%d final_transform=%.3f zoom=%.4f" % [ppc, host.x, host.y, ft, _vh.zoom()])
	if not (ppc > 0.0):
		printerr("PROBE-CANNOT-RUN: no positive scale at the opening view")
		get_tree().quit(2)
		return
	await _shot("x1")
	var focus := _focus if _focus != Vector2.INF else Vector2(GRID) * 0.5
	for z in [4.0, 16.0]:
		_vh.reset_view()
		await _frames(3)
		## `zoom_step` multiplies the current zoom, so from the opening view a
		## step of `z` is x`z` of the opening scale (not an absolute zoom).
		_vh.zoom_step(z)
		_vh.move_view_to(focus.x, focus.y)
		await _settle()
		_p("x%d px_per_cell=%.4f focus=%s" % [int(z), _px_per_cell(), focus])
		await _shot("x%d" % int(z))
	_vh.reset_view()
	_p("---- done ----")
	get_tree().quit(0)
