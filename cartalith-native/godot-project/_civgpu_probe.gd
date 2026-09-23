extends Node
## **Phase 2 affordance fields on GPU, through the real shell**
## (`OUTSTANDING_WORK.md` §2.6). Generates one world with the shell's own
## `use_gpu` preference on, then re-populates civilisation on THAT SAME
## terrain twice more -- GPU off, then GPU on -- through `civ_populate`, the
## pass that re-derives placement from the suitability field.
##
## Why re-populate instead of generating twice: `use_gpu` also moves terrain
## stages to the GPU, which is a different world by design (`DECISIONS.md`
## §7c). Two generates would compare two terrains; this compares only what
## this row changed -- resource potentials and suitability -- on one.
##
## Asserts: the two new stage names are reported by the GPU generate; the
## GPU-on re-populate reproduces the generate's own settlement list exactly;
## and prints how the CPU list compares to the GPU list.
##
## Run: Godot_v4.7.1-stable_win64_console.exe --path . _civgpu_probe.tscn
## Windowed (no pixel is read, but it is the real shell at its real size).

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	var original_gpu := bool(bridge.param_get("use_gpu"))

	var failures := 0
	for size in [512, 1024]:
		bridge.param_set("use_gpu", true)
		bridge.generate({"seed": 4242, "width_km": 2000.0, "grid_w": size, "grid_h": size, "archetype": "", "villages": true, "sea_level": 0.45})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.5).timeout

		var stages: PackedStringArray = bridge.gpu_stages_used()
		print("PROBE %d  stages: %s" % [size, ", ".join(stages)])
		for want in ["resource_potentials", "settlement_suitability"]:
			if not stages.has(want):
				print("FAIL %d: stage %s not reported" % [size, want])
				failures += 1
		var gen := _snap(bridge.settlements())

		bridge.param_set("use_gpu", false)
		## The CPU leg is only a CPU leg if the write landed: `civ_populate`
		## reads `use_gpu` from the same `self.params` this writes.
		if bool(bridge.param_get("use_gpu")):
			print("FAIL %d: use_gpu did not turn off -- the CPU leg would be a second GPU leg" % size)
			failures += 1
		var r_cpu: Dictionary = bridge.world_gen.civ_populate()
		var cpu := _snap(bridge.settlements())
		bridge.param_set("use_gpu", true)
		var r_gpu: Dictionary = bridge.world_gen.civ_populate()
		var gpu := _snap(bridge.settlements())

		if gpu != gen:
			print("FAIL %d: GPU re-populate differs from the GPU generate's own list" % size)
			failures += 1
		var shared := 0
		for s in cpu:
			if gpu.has(s):
				shared += 1
		print("PROBE %d  settlements gen=%d cpu=%d gpu=%d  cpu==gpu: %s  shared=%d  populate ms cpu=%.0f gpu=%.0f" % [
			size, gen.size(), cpu.size(), gpu.size(), str(cpu == gpu), shared,
			float(r_cpu.get("ms", -1.0)), float(r_gpu.get("ms", -1.0))])
		if gen.is_empty():
			print("FAIL %d: no settlements at all" % size)
			failures += 1

	bridge.param_set("use_gpu", original_gpu)
	print("PROBE RESULT: %s" % ("PASS" if failures == 0 else "FAIL (%d)" % failures))
	get_tree().quit(0 if failures == 0 else 1)

## `x,y,kind,faction` per settlement, in list order -- names and populations
## are derived from these and would only restate them.
func _snap(list: Array) -> Array:
	var out := []
	for s in list:
		var d: Dictionary = s
		out.append("%d,%d,%s,%d" % [int(d.get("x", -1)), int(d.get("y", -1)), String(d.get("kind", "")), int(d.get("faction", -1))])
	return out
