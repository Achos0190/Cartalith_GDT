extends Node
## `compute_stress` on GPU, through the REAL shell (`OUTSTANDING_WORK.md`
## §2.6). The shell boots with `use_gpu` on (`engine_bridge.gd`), so an
## ordinary `generate()` runs the boundary gather (`gpu_stress.wgsl`) and both
## stress blurs on the GPU, and reports `"stress"` in `gpu_stages_used`.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _stressgpu_probe.tscn
##
## **Windowed**, not `--headless`: it saves the framebuffer of each world
## (`MISTAKES.md`, "Run a pixel probe").
##
## What it can and cannot compare. `use_gpu` also moves warp and plate
## assignment to the GPU, which gives a different plate map, so a GPU and a CPU
## generation are two different worlds and their stress fields are not
## comparable cell by cell. The cell-exact CPU-vs-GPU comparison therefore lives
## in `cartalith-engine`'s tests, on one plate map. Here the probe checks what
## only the shell can show:
##
## * the GPU generation reports `"stress"`, and the CPU one does not;
## * the GPU boundary mask equals one recomputed HERE, in GDScript, from that
##   same world's own plate map: a cell is on a boundary iff a 4-neighbour
##   is on another plate (no x-wrap, as this world is not a globe). The engine
##   builds that mask on the GPU only through the gather, so this is an
##   independent oracle for it;
## * both stress fields are normalised: every value in [-1, 1] and the largest
##   |v| exactly 1;
## * a second GPU generation of the same seed is bit-identical.
## It PRINTS the CPU-vs-GPU plate-map agreement, and the stress difference on
## the cells where the two maps agree.
##
## Exit 0 pass, 1 an assertion failed, 2 the premise could not be set up.

var _app: Node
var _bridge: Node
var _fail := 0

func _ok(name: String, cond: bool, detail: String = "") -> void:
	print("  ", "ok  " if cond else "FAIL", " ", name, ("  -- " + detail) if detail != "" else "")
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _generate(gpu: bool) -> Dictionary:
	_bridge.param_set("use_gpu", gpu)
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	var gw: int = _bridge.world_gen.get_width()
	var gh: int = _bridge.world_gen.get_height()
	var plate := PackedInt32Array()
	var boundary := PackedByteArray()
	var stress := PackedFloat64Array()
	plate.resize(gw * gh)
	boundary.resize(gw * gh)
	stress.resize(gw * gh)
	for y in gh:
		for x in gw:
			var s: Dictionary = _bridge.world_gen.sample_cell(x, y)
			plate[y * gw + x] = int(s.get("plate", -1))
			boundary[y * gw + x] = 1 if bool(s.get("boundary", false)) else 0
			stress[y * gw + x] = float(s.get("stress", 99.0))
	return {"gw": gw, "gh": gh, "plate": plate, "boundary": boundary, "stress": stress,
		"stages": _bridge.gpu_stages_used(), "use_gpu": bool(_bridge.param_get("use_gpu"))}

func _shot(path: String) -> void:
	await _frames(6)
	get_viewport().get_texture().get_image().save_png(path)

## The gather's own rule, restated independently of it.
func _mask_from_plates(w: Dictionary) -> PackedByteArray:
	var gw: int = w["gw"]
	var gh: int = w["gh"]
	var pl: PackedInt32Array = w["plate"]
	var m := PackedByteArray()
	m.resize(gw * gh)
	for y in gh:
		for x in gw:
			var p := pl[y * gw + x]
			var on := (x > 0 and pl[y * gw + x - 1] != p) or (x + 1 < gw and pl[y * gw + x + 1] != p) \
				or (y > 0 and pl[(y - 1) * gw + x] != p) or (y + 1 < gh and pl[(y + 1) * gw + x] != p)
			m[y * gw + x] = 1 if on else 0
	return m

func _check_normalised(label: String, st: PackedFloat64Array) -> void:
	var mx := 0.0
	var inside := true
	for v in st:
		mx = maxf(mx, absf(v))
		inside = inside and v >= -1.0 and v <= 1.0
	_ok("%s: stress in [-1, 1]" % label, inside)
	_ok("%s: max |stress| is exactly 1" % label, mx == 1.0, str(mx))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_bridge = _app.bridge
	var boot_gpu := bool(_bridge.param_get("use_gpu"))
	_ok("the shell booted with use_gpu on", boot_gpu)

	var g := await _generate(true)
	await _shot("user://stressgpu_gpu.png")
	if not _bridge.has_world:
		print("!! no world")
		get_tree().quit(2)
		return
	var stages: PackedStringArray = g["stages"]
	print("GPU generation stages: ", ", ".join(stages))
	_ok("GPU generation reports the stress stage", stages.has("stress"))
	var n: int = g["gw"] * g["gh"]
	var bmask: PackedByteArray = g["boundary"]
	var oracle := _mask_from_plates(g)
	var diff := 0
	var on := 0
	for i in n:
		if bmask[i] != oracle[i]:
			diff += 1
		on += bmask[i]
	_ok("GPU boundary mask == mask recomputed from the plate map", diff == 0, "%d of %d cells differ, %d boundary cells" % [diff, n, on])
	_ok("the world has boundaries (positive control)", on > 1000, str(on))
	_check_normalised("GPU", g["stress"])

	var g2 := await _generate(true)
	_ok("second GPU generation: stress bit-identical", g2["stress"] == g["stress"])
	_ok("second GPU generation: boundary mask bit-identical", g2["boundary"] == g["boundary"])

	var c := await _generate(false)
	await _shot("user://stressgpu_cpu.png")
	_ok("CPU leg really ran with use_gpu off", not c["use_gpu"])
	var cstages: PackedStringArray = c["stages"]
	_ok("CPU generation does not report the stress stage", not cstages.has("stress"), ", ".join(cstages))
	_ok("CPU boundary mask == mask recomputed from its plate map", c["boundary"] == _mask_from_plates(c))
	_check_normalised("CPU", c["stress"])

	var pg: PackedInt32Array = g["plate"]
	var pc: PackedInt32Array = c["plate"]
	var sg: PackedFloat64Array = g["stress"]
	var sc: PackedFloat64Array = c["stress"]
	var same := 0
	var worst := 0.0
	var mean := 0.0
	for i in n:
		if pg[i] == pc[i]:
			same += 1
			var d := absf(sg[i] - sc[i])
			worst = maxf(worst, d)
			mean += d
	print("CPU-vs-GPU world: plate id agrees on %d of %d cells (%.2f%%); on those cells stress worst %s, mean %s"
		% [same, n, 100.0 * same / n, str(worst), str(mean / maxf(1.0, same))])

	_bridge.param_set("use_gpu", boot_gpu)
	print("PASS" if _fail == 0 else "FAIL (%d)" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
