extends Node
## Idle-redraw measurement for `ANDROID_BUILD_SCOPE.md`'s "the app never idles"
## row (owner, OnePlus 12, 2026-09-07: 60 fps at rest, ~55% of a core at the
## project picker). Desktop proxy for the device's SurfaceFlinger reading.
##
##   godot --path . _idleredraw_probe.tscn            (WINDOWED -- headless never draws)
##   godot --path . _idleredraw_probe.tscn -- --hosted   (the probe-hosting side of the gate)
##
## Mounts `app.tscn` DIRECTLY under the window root, the way the shipped build
## boots it -- not in a `SubViewport` like every other probe, because
## `DccApp._ready()` only enables low-processor mode when it owns the window
## (see `_enable_idle_mode` there), and a SubViewport host would measure the
## probe configuration instead of the shipped one.
##
## Two rest states, each measured over `WINDOW` seconds with no input:
##   picker -- the Welcome / project picker, before any world exists
##   map    -- a generated 512x384 world, dialog closed
## Per state: frames DRAWN per second (`Engine.get_frames_drawn()`), main-loop
## iterations per second (`Engine.get_process_frames()`), and this process's
## CPU time per wall second, read from outside via `OS.execute` so the read
## itself is not charged to the process being measured.
##
## A positive control follows each: one real change (a hover-free status write
## that alters text) must be drawn, or a zero draw rate means "never draws",
## not "idles correctly".

const WINDOW := 5.0

var app: Node

func _cpu_ms() -> float:
	var out := []
	var pid := OS.get_process_id()
	OS.execute("powershell", ["-NoProfile", "-Command",
		"(Get-Process -Id %d).TotalProcessorTime.TotalMilliseconds" % pid], out)
	return float(String(out[0]).strip_edges().replace(",", ".")) if out.size() > 0 else -1.0

func _measure(tag: String) -> void:
	var f0 := Engine.get_frames_drawn()
	var p0 := Engine.get_process_frames()
	var c0 := _cpu_ms()
	var t0 := Time.get_ticks_usec()
	await get_tree().create_timer(WINDOW).timeout
	var c1 := _cpu_ms()
	var dt := (Time.get_ticks_usec() - t0) / 1e6
	var drawn := Engine.get_frames_drawn() - f0
	var iters := Engine.get_process_frames() - p0
	print("[IDLE %s] low_processor=%s  window=%.2fs  drawn=%d (%.1f/s)  iterations=%d (%.1f/s)  cpu=%.0f ms (%.1f%% of a core)" % [
		tag, str(OS.low_processor_usage_mode), dt, drawn, drawn / dt, iters, iters / dt,
		c1 - c0, (c1 - c0) / (dt * 10.0)])

	## Positive control: a real change must still reach the screen.
	var d0 := Engine.get_frames_drawn()
	app.set_status("hint", "idle probe %d" % Time.get_ticks_msec(), "text_ghost")
	await get_tree().create_timer(0.3).timeout
	print("[CONTROL %s] frames drawn after one status write: %d (must be >= 1)" % [
		tag, Engine.get_frames_drawn() - d0])

## `-- --hosted`: the gate's other side. Mounts the app the way the
## `UPDATE_ALWAYS` probes do (inside a `SubViewport`) and asserts
## low-processor mode stayed OFF and frames are still drawn every iteration --
## i.e. those probes' pacing is exactly what it was before the idle fix.
func _hosted() -> void:
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 900)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(4.0).timeout
	var f0 := Engine.get_frames_drawn()
	var p0 := Engine.get_process_frames()
	await get_tree().create_timer(2.0).timeout
	var drawn := Engine.get_frames_drawn() - f0
	var iters := Engine.get_process_frames() - p0
	var ok := (not OS.low_processor_usage_mode) and drawn == iters and drawn > 0
	print("[HOSTED] low_processor=%s drawn=%d iterations=%d -> %s" % [
		str(OS.low_processor_usage_mode), drawn, iters, "PASS" if ok else "FAIL"])
	get_tree().quit(0 if ok else 1)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a != "--hosted":
			print("[FATAL] unknown argument ", a); get_tree().quit(2); return
	if args.has("--hosted"):
		await _hosted()
		return
	app = load("res://shell/app.tscn").instantiate()
	get_tree().root.add_child.call_deferred(app)
	await get_tree().create_timer(4.0).timeout
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	await _measure("picker")

	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	app.bridge.generate({
		"seed": 483920, "width_km": 800.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(3.0).timeout
	await _measure("map")

	## The two per-frame layers must NOT idle: they animate by requesting a draw
	## every frame (`wind_fx_layer.gd` via `queue_redraw()`, `water_anim_layer.gd`
	## via a per-frame shader parameter). Both should read ~60 draws/s here.
	app.viewport.set_debug_layer("wind")
	await get_tree().create_timer(1.0).timeout
	await _measure("wind-view")
	app.viewport.set_debug_layer("off")
	var water = load("res://shell/water_anim_layer.gd").new()
	water.setup(app.bridge)
	app.viewport.overlay.add_child(water)
	print("[WATER] set_enabled(true) -> ", water.set_enabled(true))
	await get_tree().create_timer(1.0).timeout
	await _measure("water-anim")
	water.set_enabled(false)
	await get_tree().create_timer(1.0).timeout
	await _measure("map-again")
	get_tree().quit(0)
