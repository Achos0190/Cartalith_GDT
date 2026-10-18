extends Node
## **EF-9's first per-world boost build must not run on the main thread**
## (Ruling CL item 4, `ef9_bridge.rs::BoostSlot`). Before the move
## `lod_ef9_boost()` ran the importance field and `BoostField::build` inline:
## 3.4 / 44.5 / 191 ms at 512x384 / 2048x1311 / 4096x2621 (measured here,
## 2026-10-18). Windowed (run: `godot --path . _ef9firstbuild_probe.tscn`).
## Asserts, per world: the first call answers `pending` and returns within
## HITCH_MS, polling is cheap, the field lands (n > 0), and the texture builds.
## HITCH_MS = 20 is a labelled judgement (a hitch is felt past about one 60 Hz
## frame; the measured first call is under 3 ms, the old inline one over 40).
const HITCH_MS := 20.0
var _fail := 0
var _app: Node
var _br: Node
func _ready() -> void:
	DisplayServer.window_set_size(Vector2i(1280, 720))
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	for grid in [Vector2i(512, 384), Vector2i(2048, 1311), Vector2i(4096, 2621)]:
		var firsts := []
		var seconds := []
		for seed_v in [24601, 483920, 71077345]:
			_br.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": grid.x, "grid_h": grid.y, "archetype": "", "villages": true, "sea_level": 0.42})
			var spins := 0
			while _br.generating and spins < 4800:
				await get_tree().create_timer(0.25).timeout
				spins += 1
			await get_tree().create_timer(1.0).timeout
			var t0 := Time.get_ticks_usec()
			var info: Dictionary = _br.lod_ef9_boost()
			var t1 := Time.get_ticks_usec()
			var first_pending := bool(info.get("pending", false))
			var worst_poll := 0.0
			var frames := 0
			while bool(info.get("pending", false)):
				await get_tree().process_frame
				var p0 := Time.get_ticks_usec()
				info = _br.lod_ef9_boost()
				worst_poll = maxf(worst_poll, (Time.get_ticks_usec() - p0) / 1000.0)
				frames += 1
			var t2 := Time.get_ticks_usec()
			var tex = _br.lod_ef9_boost_texture()
			var t3 := Time.get_ticks_usec()
			if not bool(first_pending) or (t1 - t0) / 1000.0 > HITCH_MS or worst_poll > HITCH_MS or int(info.get("n", 0)) <= 0 or tex == null:
				_fail += 1
				printerr("FB-FAIL grid %dx%d seed %d (pending=%s first=%.2f ms poll=%.3f ms n=%s tex=%s)" % [grid.x, grid.y, seed_v, str(first_pending), (t1 - t0) / 1000.0, worst_poll, str(info.get("n")), str(tex)])
			print("FB grid %dx%d seed %d: first call returned in %.2f ms (pending=%s), ready after %.1f ms / %d frames, worst poll %.3f ms, texture %.2f ms, n=%s any=%s" % [grid.x, grid.y, seed_v, (t1 - t0) / 1000.0, "yes" if first_pending else "no", (t2 - t0) / 1000.0, frames, worst_poll, (t3 - t2) / 1000.0, str(info.get("n")), str(info.get("any"))])
	print("FB %s" % ("PASS" if _fail == 0 else "FAIL (%d)" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)
