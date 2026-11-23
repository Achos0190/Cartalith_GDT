extends Node
## Owner Ruling HR (2026-11-23) through the real shell: provinces that follow
## the nations' border logic, drawn as a lighter line than a national border.
##
## Generates one world (seed, grid width) at `civ.border_friction` 0.6 (the app
## default) with the Territory and Provinces layers on, saves a fit-view frame
## and a x4 close-up centred between the two closest city-tier seats of one
## faction (the same point on any binary: the settlements do not move with the
## ruling). Checks, on a binary with `civ_province_lines`:
##   - the engine hands province lines, and no raster province line (the map
##     never shows both);
##   - DRAWN: the province line on vs off moves pixels (positive control);
##   - the province line is lighter than the national line: its ink alpha and
##     width are under the border line's.
## With `--before` (a binary from before the ruling, no `civ_province_lines`)
## it only saves the frames, for the before/after pair.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _provborders_probe.tscn --resolution 1600x900 -- --seed 912345 --gw 384 --shots <dir> --tag after [--before] [--mutate noline]
##
## `--mutate noline` hands the overlay an empty province line list, so DRAWN
## must fail (exit 1). Lines are prefixed `PB  `.
## Exit 0 = all passed, 1 = a check failed, 2 = could not run, 3 = watchdog.

var _app: Node
var _bridge
var _fail := 0
var _vp: Viewport
var _seed := 912345
var _gw := 384
var _shots := ""
var _tag := "after"
var _before := false
var _mutate := ""


func _p(s: String) -> void:
	print("PB  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _shot() -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	await _frames(1)
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()


func _moved(a: Image, b: Image) -> int:
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


## Reads the arguments after `--`; an unknown one aborts (exit 2).
func _args() -> bool:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		var k: String = a[i]
		var v: String = a[i + 1] if i + 1 < a.size() else ""
		match k:
			"--seed": _seed = int(v)
			"--gw": _gw = int(v)
			"--shots": _shots = v
			"--tag": _tag = v
			"--mutate": _mutate = v
			"--before":
				_before = true
				i -= 1
			_:
				_p("ABORT: unknown argument %s" % k)
				return false
		i += 2
	return true


func _save(img: Image, name: String) -> void:
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png("%s/%s" % [_shots, name])
		_p("frame %s/%s" % [_shots, name])


## The grid point midway between the two closest city-tier seats of one faction.
func _look() -> Vector2:
	var seats: Array = []
	for s in _bridge.settlements():
		if int(s.get("faction", 0)) > 0 and String(s.get("kind", "")) in ["metropolis", "capital", "city"]:
			seats.append(s)
	var best := INF
	var at := Vector2(_gw * 0.5, _gw * 0.32)
	for i in seats.size():
		for j in range(i + 1, seats.size()):
			if int(seats[i]["faction"]) != int(seats[j]["faction"]):
				continue
			var a := Vector2(float(seats[i]["x"]), float(seats[i]["y"]))
			var b := Vector2(float(seats[j]["x"]), float(seats[j]["y"]))
			if a.distance_to(b) < best:
				best = a.distance_to(b)
				at = (a + b) * 0.5
	return at


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()
	if not _args():
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_p("ABORT: headless -- the framebuffer is not readable. Run windowed.")
		get_tree().quit(2)
		return
	_vp = get_viewport()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	var has: bool = _bridge.world_gen.has_method("civ_province_lines")
	if has == _before:
		_p("ABORT: --before %s but the binary %s civ_province_lines" % [_before, "has" if has else "lacks"])
		get_tree().quit(2)
		return
	_bridge.param_set("civ.border_friction", 0.6)
	_bridge.generate({"seed": _seed, "width_km": 4000.0, "grid_w": _gw, "grid_h": int(round(_gw * 0.64)), "archetype": ""})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	var opd = _app.get("open_project_dialog")
	if opd != null and opd is Window and (opd as Window).visible:
		(opd as Window).hide()
	_app.viewport.set_layer_visible("territory", true)
	_app.viewport.set_layer_visible("provinces", true)
	_app.viewport.reset_view()
	await _frames(4)

	if not _before:
		var lines: Array = _bridge.province_lines()
		_check("the engine hands province lines", lines.size() > 0, "%d runs" % lines.size())
		_check("no raster province line beside them", _bridge.province_boundary_texture() == null)
		var ov = _app.viewport.overlay
		_check("lighter than the national line", ov.PROVINCE_LINE_INK.a < ov.BORDER_LINE_INK.a and ov.PROVINCE_LINE_PX < ov.BORDER_LINE_PX,
			"alpha %.2f < %.2f, width %.1f < %.1f" % [ov.PROVINCE_LINE_INK.a, ov.BORDER_LINE_INK.a, ov.PROVINCE_LINE_PX, ov.BORDER_LINE_PX])
		if _mutate == "noline":
			ov.set_province_lines([])
		var on := await _shot()
		ov.set_show_province_lines(false)
		var off := await _shot()
		ov.set_show_province_lines(true)
		var d := _moved(on, off)
		_check("DRAWN: the province line on vs off moves pixels", d > 0, "%d px" % d)
	_save(await _shot(), "%d_%d_x1_%s.png" % [_seed, _gw, _tag])

	var p := _look()
	_p("look at grid (%d, %d)" % [p.x, p.y])
	var c: Vector2 = _app.viewport.size * 0.5
	_app.viewport._zoom_at(c, 4.0)
	await _frames(2)
	_app.viewport.move_view_to(p.x, p.y)
	await _frames(6)
	_save(await _shot(), "%d_%d_x4_%s.png" % [_seed, _gw, _tag])
	_p("RESULT fail=%d" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
