extends Node
## Committed measurement harness for EXPORT_SCOPE.md §7 E4 batch D -- a whole
## export with overlays at full size, through `ExportOverlay.run` inside the real
## shell, on the grid the earlier export measurements used (2048 x 1311).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _exportfull_probe.tscn -- --width 16384
##
## `--width` is required (a `BAKE_WIDTHS` rung); `--seed` (default 20260906, the
## seed `_exportbig_probe.gd` measured) and `--tile` (default `TILE_MAX`) are
## optional; anything else aborts with exit 2.
##
## **Windowed, never `--headless`**: the overlay tiles are `SubViewport`
## readbacks, and the dummy display driver draws nothing (`MISTAKES.md`, "Write
## `--headless` into a brief").
##
## What it prints, for a person and for `EXPORT_SCOPE.md`:
##   * the estimate (`export_image_estimate`: bands, rows, band peak);
##   * `PID <n>` before the export starts, so the host can poll this process's
##     peak resident set (`MISTAKES.md`: a raw buffer is not a peak -- poll it
##     from the host);
##   * the run's wall time, file size and band count;
##   * every frame's length while the run is in progress: the longest, and how
##     many exceeded 50 ms and 100 ms -- Ruling AZ's "no UI freeze", measured;
##   * `BANDROWS <n>` and the file path, for the host's band-boundary check.
##
## Exit 0 when the run wrote its file, 1 when it did not, 2 for bad arguments or
## the watchdog. It asserts nothing about the frame times: they are the
## measurement, recorded with their spread where the scope cites them.
##
## Result: a measurement with one gate (the export wrote its file). Prints `  seed <n> generated <w>x<h> in <s> s; <n> settlements`,
## `  estimate: ...`, `PID <n>`, `  run: ok, <w> x <h>, <n> bands, <MB> MB`, `  wall <s> s; frames <n>, median <ms> ms, longest <ms> ms,
## <n> over 50 ms, <n> over 100 ms`, `  longest main-thread step of each kind (ms): ...`, `BANDROWS <n>`, `FILE <path>` and
## `RESULT: PASS|FAIL`. The file is `user://_exportfull_probe/full_<width>.png`. Exit 0 on PASS, 1 on FAIL or
## `RESULT: FAIL -- no live overlay with a world`, 2 after an `ABORT:` (unrecognised argument, missing `--width`) or `WATCHDOG:`.

const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")

## Frame lengths while `on`, in ms.
class _FrameWatch extends Node:
	var on := false
	var lengths := PackedFloat32Array()
	var _last := 0
	func _process(_d: float) -> void:
		var now := Time.get_ticks_usec()
		if on and _last > 0:
			lengths.append((now - _last) / 1000.0)
		_last = now

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

func _ready() -> void:
	var width := -1
	var seed_value := 20260906
	var tile := EXPORT_OVERLAY.TILE_MAX
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--width":
				width = int(args[i + 1]) if i + 1 < args.size() else -1
				i += 2
			"--seed":
				seed_value = int(args[i + 1]) if i + 1 < args.size() else 0
				i += 2
			"--tile":
				tile = int(args[i + 1]) if i + 1 < args.size() else -1
				i += 2
			_:
				print("ABORT: unrecognised argument %s -- this probe reads --width, --seed and --tile only" % args[i])
				get_tree().quit(2)
				return
	if width <= 0:
		print("ABORT: --width is required, e.g. -- --width 16384")
		get_tree().quit(2)
		return
	get_tree().create_timer(3600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))

	var dir := ProjectSettings.globalize_path("user://_exportfull_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	var tg := Time.get_ticks_msec()
	bridge.generate({"seed": seed_value, "width_km": 1200.0, "grid_w": 2048, "grid_h": 1311,
		"archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	for _f in 6:
		await get_tree().process_frame
	var ov: Control = _find_script(app, "map_overlay.gd")
	if ov == null or int(ov.get("_gw")) < 2:
		print("RESULT: FAIL -- no live overlay with a world")
		get_tree().quit(1)
		return
	print("  seed %d generated %dx%d in %.1f s; %d settlements" % [seed_value, ov.get("_gw"), ov.get("_gh"),
		(Time.get_ticks_msec() - tg) / 1000.0, (ov.get("_settlements") as Array).size()])

	var content := {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true,
		"icons": true, "territory": true}
	var opts := {"width": width, "content": content}
	var est: Dictionary = bridge.world_gen.export_image_estimate(opts)
	print("  estimate: %s x %s, %s bands of %s rows (apron %s), band peak %.0f MB, monolithic peak %.0f MB, memory_available %s" % [
		est.get("width"), est.get("height"), est.get("bands"), est.get("band_rows"), est.get("apron_rows"),
		float(est.get("band_peak_bytes", 0)) / 1048576.0, float(est.get("peak_bytes", 0)) / 1048576.0,
		est.get("memory_available", "absent")])

	var runner := EXPORT_OVERLAY.new()
	add_child(runner)
	var watch := _FrameWatch.new()
	add_child(watch)
	var path := dir.path_join("full_%d.png" % width)
	print("PID %d" % OS.get_process_id())
	await get_tree().create_timer(2.0).timeout   ## the host's poller attaches
	watch.on = true
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = await runner.run(bridge, path, opts, ov,
		[app.viewport.territory_view, app.viewport.province_view], tile)
	var wall := Time.get_ticks_msec() - t0
	watch.on = false
	var L := watch.lengths
	var worst := 0.0
	var over50 := 0
	var over100 := 0
	for ms in L:
		worst = maxf(worst, ms)
		if ms > 50.0:
			over50 += 1
		if ms > 100.0:
			over100 += 1
	var sorted := Array(L)
	sorted.sort()
	var median: float = sorted[sorted.size() / 2] if sorted.size() > 0 else 0.0
	print("  run: %s" % (("ok, %d x %d, %d bands, %.1f MB" % [r["width"], r["height"], r["bands"], float(r["bytes"]) / 1e6])
		if bool(r.get("ok", false)) else "FAILED: %s" % r.get("error", "?")))
	print("  wall %.1f s; frames %d, median %.1f ms, longest %.1f ms, %d over 50 ms, %d over 100 ms" % [
		wall / 1000.0, L.size(), median, worst, over50, over100])
	print("  longest main-thread step of each kind (ms): %s" % runner.step_ms)
	print("BANDROWS %s" % est.get("band_rows", 0))
	print("FILE %s" % path)
	print("RESULT: %s" % ("PASS" if bool(r.get("ok", false)) else "FAIL"))
	get_tree().quit(0 if bool(r.get("ok", false)) else 1)
