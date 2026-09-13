extends Node
## Timing witness for the TILECACHE lane's fix: how much does
## `invalidate_lod_tiles()` add to `world_workspace.gd::_on_sculpt_commit()`
## once the deep-zoom pyramid is up? Times ONLY the commit call itself, N
## repeated strokes in one run, median with min..max (`MISTAKES.md`'s timing
## rule) -- run this alone, never under a parallel suite, and re-run
## independently before trusting the number.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _sculptcommittime_probe.tscn
##
## Headless is fine -- `Time.get_ticks_usec()` around a plain function call,
## no framebuffer read.

const REPS := 10

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _median(vals: Array) -> float:
	var v := vals.duplicate()
	v.sort()
	var n := v.size()
	if n == 0:
		return -1.0
	if n % 2 == 1:
		return v[n / 2]
	return (v[n / 2 - 1] + v[n / 2]) * 0.5

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] the .gdextension did not load.")
		get_tree().quit(2)
		return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(30)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(15)

	var bridge = app.bridge
	var vh = app.viewport
	var ws = app._world_workspace()

	bridge.generate({
		"seed": 24601, "width_km": 640.0, "grid_w": 96, "grid_h": 64,
		"sea_level": 0.5, "villages": true,
	})
	await bridge.generation_finished
	await _frames(10)
	if not bridge.has_world:
		print("[ABORT] no world")
		get_tree().quit(2)
		return

	var guard := 0
	while vh.zoom() <= 2.3 and guard < 20:
		vh.zoom_step(1.35)
		guard += 1
		await _frames(2)
	await _frames(30)
	print("TIME setup: zoom=%s lod_active=%s tiles=%d"
		% [vh.zoom(), vh.lod_active(), (vh.get("_lod_tiles") as Dictionary).size()])
	if not vh.lod_active():
		print("[ABORT] never reached deep zoom -- this measurement would be meaningless")
		get_tree().quit(2)
		return

	var g = bridge.grid_size()
	var cx: float = g.x * 0.5
	var cy: float = g.y * 0.5
	var ms: Array = []
	for i in REPS:
		var dx := float(i % 5) - 2.0   ## Small spread so every rep is a real,
		var dy := float((i * 3) % 5) - 2.0   ## distinct stroke, not a no-op repeat.
		bridge.sculpt_begin_stroke()
		bridge.sculpt_add_point(cx + dx - 5.0, cy + dy - 5.0)
		bridge.sculpt_add_point(cx + dx, cy + dy)
		bridge.sculpt_add_point(cx + dx + 5.0, cy + dy + 5.0)
		bridge.sculpt_end_stroke()
		if bridge.sculpt_stamp_count() == 0:
			print("TIME rep %d: SKIPPED, no stamp landed" % i)
			continue
		var t0 := Time.get_ticks_usec()
		ws._on_sculpt_commit()
		var t1 := Time.get_ticks_usec()
		var dt_ms := (t1 - t0) / 1000.0
		ms.append(dt_ms)
		print("TIME rep %d: %.3f ms" % [i, dt_ms])
		await _frames(3)

	if ms.is_empty():
		print("[ABORT] no timed rep landed")
		get_tree().quit(2)
		return
	ms.sort()
	print("TIME RESULT n=%d median=%.3f ms  min=%.3f  max=%.3f  all=%s"
		% [ms.size(), _median(ms), ms[0], ms[ms.size() - 1], ms])
	get_tree().quit(0)
