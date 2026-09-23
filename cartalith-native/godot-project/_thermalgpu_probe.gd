extends Node
## The GPU thermal pass through the REAL shell (`OUTSTANDING_WORK.md` §2.6,
## erosion's per-cell parts on GPU). `world_workspace.gd::_run_erode()` is the
## Erode button's own handler; it calls `WorldGen.erode_op`, which moves the
## thermal passes to `cartalith_gpu::thermal_grid_gpu_with` when the shell's
## `use_gpu` preference is on (`engine_bridge.gd` turns it on at boot).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _thermalgpu_probe.tscn
##
## **Windowed**, not `--headless`: it saves the framebuffer before and after
## each run (`MISTAKES.md`, "Run a pixel probe").
##
## Same world, erode twice from the same starting field -- once with the
## shell's GPU toggle on, once off, undoing in between -- with droplets at 0 so
## the op is thermal + rebound only, and 30 passes at talus 0.001 (the panel's
## own extremes) so thermal moves as much ground as it can, plus the
## reference default (8, 0.012) and a passes-0 control. Asserts on the
## elevation of every cell (`sample_cell`) -- what the kernel computes --
## and REPORTS drainage, precipitation and the map texture against
## GPU-vs-GPU and CPU-vs-CPU repeats (see `_round` for why those are not
## asserted).
##
## Exit 0 pass, 1 an assertion failed, 2 the premise could not be set up.

var _app: Node
var _bridge: Node
var _fail := 0

func _p(s: String) -> void:
	print(s)

func _ok(name: String, cond: bool, detail: String = "") -> void:
	print("  ", "ok  " if cond else "FAIL", " ", name, ("  -- " + detail) if detail != "" else "")
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Every cell's elevation, drainage and precipitation, as `sample_cell`
## reports them -- elevation to find the thermal deviation, the other two to
## see how far the flow/climate recompute that follows the op carries it.
func _fields() -> Dictionary:
	var gw: int = _bridge.world_gen.get_width()
	var gh: int = _bridge.world_gen.get_height()
	var e := PackedFloat64Array()
	var dr := PackedFloat64Array()
	var pr := PackedFloat64Array()
	e.resize(gw * gh)
	dr.resize(gw * gh)
	pr.resize(gw * gh)
	for y in gh:
		for x in gw:
			var s: Dictionary = _bridge.world_gen.sample_cell(x, y)
			e[y * gw + x] = float(s.get("elevation", -1.0))
			dr[y * gw + x] = float(s.get("drainage", -1.0))
			pr[y * gw + x] = float(s.get("precipitation", -1.0))
	return {"e": e, "d": dr, "p": pr}

func _shot(path: String) -> Image:
	await _frames(6)
	var img := get_viewport().get_texture().get_image()
	img.save_png(path)
	return img

func _erode(ws, gpu: bool, passes: int, talus: float) -> Dictionary:
	_bridge.param_set("use_gpu", gpu)
	_ok("use_gpu reads back %s" % gpu, bool(_bridge.world_gen.get_params().get("use_gpu", not gpu)) == gpu)
	ws._erode_op["droplets"] = 0
	ws._erode_op["thermal_passes"] = passes
	ws._erode_op["talus"] = talus
	## `_run_erode` discards the op's dictionary, so call the same engine
	## entry with the same dictionary for the counts, then let the handler
	## repaint. Two ops would erode twice -- so the engine call IS the op,
	## and the repaint lines are `_run_erode`'s own two, verbatim.
	var r: Dictionary = _bridge.world_gen.erode_op(ws._erode_op)
	_app.viewport.map_view.texture = _bridge.color_texture()
	_app.viewport.invalidate_lod_tiles()
	return r

func _count_diff(a: PackedFloat64Array, b: PackedFloat64Array) -> int:
	var n := 0
	for i in a.size():
		if a[i] != b[i]:
			n += 1
	return n

func _map() -> Image:
	return _app.viewport.map_view.texture.get_image()

func _px(a: Image, b: Image) -> Array:
	var n := 0
	var mx := 0.0
	for y in a.get_height():
		for x in a.get_width():
			var c := a.get_pixel(x, y)
			var g := b.get_pixel(x, y)
			if c != g:
				n += 1
				mx = maxf(mx, maxf(absf(g.r - c.r), maxf(absf(g.g - c.g), absf(g.b - c.b))))
	return [n, int(round(mx * 255.0))]

## One configuration: erode on GPU, undo, erode on GPU again (determinism
## control), undo, erode on CPU, undo. Compares elevation (the thing the
## kernel computes), then drainage/precipitation and the map texture (what
## the flow + climate recompute after the op makes of it).
func _round(ws, passes: int, talus: float) -> void:
	_p("
=== thermal_passes %d, talus %s, droplets 0 ===" % [passes, str(talus)])
	var f0 := _fields()
	var m0 := _map()
	var r_gpu := _erode(ws, true, passes, talus)
	var f_gpu := _fields()
	var m_gpu := _map()
	await _shot("user://thermalgpu_%d_gpu.png" % passes)
	_ok("GPU run reports thermal_on_gpu iff there were passes to run", bool(r_gpu.get("thermal_on_gpu", false)) == (passes > 0))
	_app.undo_last()
	await _frames(6)
	_ok("undo restored the starting field exactly", _fields()["e"] == f0["e"])
	var r_gpu2 := _erode(ws, true, passes, talus)
	var f_gpu2 := _fields()
	var m_gpu2 := _map()
	_app.undo_last()
	await _frames(6)
	var r_cpu := _erode(ws, false, passes, talus)
	var f_cpu := _fields()
	var m_cpu := _map()
	await _shot("user://thermalgpu_%d_cpu.png" % passes)
	_ok("CPU run does not report the GPU", not bool(r_cpu.get("thermal_on_gpu", true)))
	_app.undo_last()
	await _frames(6)
	var r_cpu2 := _erode(ws, false, passes, talus)
	var f_cpu2 := _fields()
	var m_cpu2 := _map()
	_app.undo_last()
	await _frames(6)
	_bridge.param_set("use_gpu", true)
	_p("  op counts  GPU changed/lowered/raised %d/%d/%d   CPU %d/%d/%d"
		% [r_gpu.cells_changed, r_gpu.cells_lowered, r_gpu.cells_raised, r_cpu.cells_changed, r_cpu.cells_lowered, r_cpu.cells_raised])

	var e0: PackedFloat64Array = f0["e"]
	var ec: PackedFloat64Array = f_cpu["e"]
	var eg: PackedFloat64Array = f_gpu["e"]
	var worst := 0.0
	for i in e0.size():
		worst = maxf(worst, absf(ec[i] - eg[i]))
	var moved := _count_diff(e0, ec)
	_p("  elevation: %d cells, CPU op moved %d, CPU-vs-GPU unequal %d, worst %s" % [e0.size(), moved, _count_diff(ec, eg), str(worst)])
	if passes > 0:
		_ok("the op actually moved ground (positive control)", moved > 1000, "%d cells" % moved)
	else:
		_ok("passes 0 control: elevation untouched", moved == 0, "%d cells" % moved)
	## `THERMAL_GPU_TOL` (1e-6) x 1.8 for the rebound, as the engine test holds it.
	_ok("GPU and CPU elevation agree within 1.8e-6", worst <= 1.8e-6, str(worst))
	_ok("GPU run twice: elevation bit-identical", f_gpu2["e"] == eg)
	_ok("CPU run twice: elevation bit-identical", f_cpu2["e"] == ec)
	_p("  drainage cells differing: GPU-vs-CPU %d, GPU-vs-GPU %d, CPU-vs-CPU %d; precipitation: GPU-vs-CPU %d, GPU-vs-GPU %d, CPU-vs-CPU %d"
		% [_count_diff(f_cpu["d"], f_gpu["d"]), _count_diff(f_gpu2["d"], f_gpu["d"]), _count_diff(f_cpu2["d"], f_cpu["d"]),
			_count_diff(f_cpu["p"], f_gpu["p"]), _count_diff(f_gpu2["p"], f_gpu["p"]), _count_diff(f_cpu2["p"], f_cpu["p"])])
	var ch := _px(m0, m_cpu)
	var gc := _px(m_gpu, m_cpu)
	var gg := _px(m_gpu, m_gpu2)
	var cc := _px(m_cpu, m_cpu2)
	var tot := m_cpu.get_width() * m_cpu.get_height()
	_p("  map texture %dx%d: op changed %d px; GPU-vs-CPU differ %d px (%.3f%%, max channel %d/255); GPU-vs-GPU differ %d px; CPU-vs-CPU differ %d px"
		% [m_cpu.get_width(), m_cpu.get_height(), ch[0], gc[0], 100.0 * gc[0] / tot, gc[1], gg[0], cc[0]])
	if passes > 0:
		_ok("the op visibly changed the map (positive control)", ch[0] > 0)
	## NOT asserted: the map texture is downstream of the flow + climate
	## recompute that follows every op, and that recompute is not a function
	## of elevation alone. Measured 2026-09-23: with elevation BIT-IDENTICAL,
	## two CPU runs still differ in drainage on ~all cells and precipitation on
	## ~20-30k, and two `use_gpu` runs differ by thousands of map pixels -- in
	## the passes-0 control too, where the thermal kernel never runs. So a
	## GPU-vs-CPU map diff here is judged against the GPU-vs-GPU one, not 0.

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world or not _bridge._has("erode_op"):
		_p("!! no world, or no erode_op in this build")
		get_tree().quit(2)
		return
	_ok("the shell booted with use_gpu on", bool(_bridge.world_gen.get_params().get("use_gpu", false)))
	var ws = _app._world_workspace()

	for cfg in [[8, 0.012], [30, 0.001], [0, 0.012]]:
		await _round(ws, cfg[0], cfg[1])

	_p("PASS" if _fail == 0 else "FAIL (%d)" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
