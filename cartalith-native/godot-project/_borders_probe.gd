extends Node
## Owner Ruling GO (2026-11-22) through the real shell: borders that follow
## terrain, and the drawn border line (O4).
##
## For one seed and grid width it generates the world twice through the
## bridge, `civ.border_friction` 0 (the old territory) then 0.6 (the app
## default), with the Territory layer on, and saves a fit-view frame of each.
## With `--look x,y` (grid cells, repeatable) it also saves a x4 close-up of
## each point at 0.6, and with `--zoomat sx,sy` (window pixels of the fit
## frame, repeatable) a x4 close-up about that point. Checks:
##   - at 0 the engine hands no border line (`border_lines()` empty), at 0.6
##     it hands some;
##   - DRAWN: at 0.6 the framebuffer with the line on vs off (positive
##     control) moves pixels; at 0 the same toggle moves exactly 0.
##
## Windowed (pixel diff: the framebuffer is not readable headless):
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _borders_probe.tscn --resolution 1600x900 -- --seed 912345 --gw 2048 --shots <dir> [--look 716,660] [--mutate noline]
##
## `--mutate noline` hands the overlay an empty line list at 0.6, so the
## DRAWN check must fail (exit 1). Every line is prefixed `BP  `.
## Exit 0 = all passed, 1 = a check failed, 2 = could not run, 3 = watchdog.

var _app: Node
var _bridge
var _fail := 0
var _vp: Viewport
var _seed := 912345
var _gw := 384
var _shots := ""
var _looks: Array[Vector2] = []
var _mutate := ""
var _zoomats: Array[Vector2] = []


func _p(s: String) -> void:
	print("BP  %s" % s)


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
			"--look":
				var xy := v.split(",")
				_looks.append(Vector2(float(xy[0]), float(xy[1])))
			"--zoomat":
				var sxy := v.split(",")
				_zoomats.append(Vector2(float(sxy[0]), float(sxy[1])))
			"--mutate": _mutate = v
			_:
				_p("ABORT: unknown argument %s" % k)
				return false
		i += 2
	return true


func _generate(strength: float) -> void:
	_bridge.param_set("civ.border_friction", strength)
	_bridge.generate({"seed": _seed, "width_km": 4000.0, "grid_w": _gw, "grid_h": int(round(_gw * 0.64)),
		"archetype": ""})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	## The welcome dialog opens after the first drawn frame and is not under
	## the root (`_lodshowcase_probe.gd` found the same); close it directly.
	var opd = _app.get("open_project_dialog")
	if opd != null and opd is Window and (opd as Window).visible:
		(opd as Window).hide()
	_app.viewport.set_layer_visible("territory", true)
	_app.viewport.reset_view()
	await _frames(4)


## The line on vs off on the current world: pixels moved.
func _line_diff() -> int:
	var on := await _shot()
	_app.viewport.overlay.set_show_border_lines(false)
	var off := await _shot()
	_app.viewport.overlay.set_show_border_lines(true)
	return _moved(on, off)


func _save(img: Image, name: String) -> void:
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png("%s/%s" % [_shots, name])
		_p("frame %s/%s" % [_shots, name])


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
	if not _bridge.world_gen.has_method("civ_border_lines"):
		_p("ABORT: this binary has no civ_border_lines #[func] -- cargo build -p cartalith-godot")
		get_tree().quit(2)
		return

	await _generate(0.0)
	var lines0: Array = _bridge.border_lines()
	_check("strength 0: no border line from the engine", lines0.is_empty(), "%d runs" % lines0.size())
	var d0 := await _line_diff()
	_check("strength 0: toggling the line moves nothing", d0 == 0, "%d px" % d0)
	_save(await _shot(), "%d_%d_before.png" % [_seed, _gw])

	await _generate(0.6)
	var lines: Array = _bridge.border_lines()
	_check("strength 0.6: the engine hands a border line", lines.size() > 0, "%d runs" % lines.size())
	if _mutate == "noline":
		_app.viewport.overlay.set_border_lines([])
	var d1 := await _line_diff()
	_check("strength 0.6: DRAWN, the line on vs off moves pixels", d1 > 0, "%d px" % d1)
	_save(await _shot(), "%d_%d_after.png" % [_seed, _gw])

	var k := 0
	for p in _looks:
		_app.viewport.reset_view()
		await _frames(2)
		## Zoom first: at the fit zoom `move_view_to` is clamped to keep the map
		## covering the view, so it cannot pan.
		var c: Vector2 = _app.viewport.size * 0.5
		_app.viewport._zoom_at(c, 4.0)  ## node-local, the method's own contract
		await _frames(2)
		_app.viewport.move_view_to(p.x, p.y)
		await _frames(6)
		_save(await _shot(), "%d_%d_after_x4_look%d.png" % [_seed, _gw, k])
		k += 1
	## `--zoomat sx,sy`: a x4 close-up about a point picked on the fit frame,
	## in window pixels (the fit view is the same frame `_save` wrote above).
	k = 0
	for sp in _zoomats:
		_app.viewport.reset_view()
		await _frames(2)
		var local: Vector2 = sp - _app.viewport.get_global_rect().position
		_app.viewport._zoom_at(local, 4.0)
		await _frames(6)
		_save(await _shot(), "%d_%d_after_x4_at%d.png" % [_seed, _gw, k])
		k += 1
	_p("RESULT fail=%d" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
