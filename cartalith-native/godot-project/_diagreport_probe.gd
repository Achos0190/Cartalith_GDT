extends Node
## Regression check for `Help ▸ Save diagnostic report`
## (`LARGE_ITEM_RULINGS.md`'s "Build" ruling on `Report an issue`).
##
## Exercises `DiagnosticReport.write()` end to end against a real running app
## -- not a parse-check -- and `EngineBridge.last_error()` against a REAL
## engine-reported failure (opening a project path that does not exist)
## rather than a synthetic `note_error()` call, so the probe proves the wiring
## from a genuine failure through to the written file, not just that the
## plumbing compiles.
##
## Run: godot4 --headless --path . _diagreport_probe.tscn

var _app: Node

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge

	var fails := 0

	# -- last_error() starts empty this session -------------------------------
	var e0: Dictionary = bridge.last_error()
	print("PROBE last_error before anything: ", e0)
	if not e0.is_empty():
		print("PROBE FAIL: last_error should start empty this session")
		fails += 1

	# -- a REAL engine-reported failure: open a project that does not exist ---
	var missing_path := ProjectSettings.globalize_path("user://__diagreport_probe_missing__.zip")
	var open_ok: bool = bridge.load_save(missing_path)
	print("PROBE load_save(missing) ok=", open_ok)
	if open_ok:
		print("PROBE FAIL: loading a nonexistent path should not report ok")
		fails += 1
	var e1: Dictionary = bridge.last_error()
	print("PROBE last_error after a real failure: ", e1)
	if e1.is_empty() or String(e1.get("text", "")).find(missing_path) < 0:
		print("PROBE FAIL: last_error did not capture the load failure and its path")
		fails += 1

	# -- generate a small world so section 1 has real content -----------------
	bridge.generate({
		"seed": 55019, "width_km": 900.0, "grid_w": 192, "grid_h": 144,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	# -- the actual menu action, exactly as _on_help dispatches it -------------
	DiagnosticReport.write(_app, bridge)
	## `write()` is synchronous end to end (no signal, no await inside it), so
	## the file exists the instant the call above returns.

	var dir := DccSettings.storage_root("exports")
	var newest_path := ""
	var newest_time := -1
	var da := DirAccess.open(dir)
	if da != null:
		da.list_dir_begin()
		var f := da.get_next()
		while f != "":
			if f.begins_with("cartalith_diagnostic_report_") and f.ends_with(".txt"):
				var full := dir.path_join(f)
				var t := FileAccess.get_modified_time(full)
				if t > newest_time:
					newest_time = t
					newest_path = full
			f = da.get_next()
		da.list_dir_end()

	print("PROBE report path: ", newest_path)
	if newest_path == "":
		print("PROBE FAIL: no report file found under ", dir)
		fails += 1
		get_tree().quit(1)
		return

	var rf := FileAccess.open(newest_path, FileAccess.READ)
	var text := rf.get_as_text()
	rf.close()

	var checks := [
		"Cartalith diagnostic report",
		"== Generation info · missing bindings · project format version ==",
		"== GPU state ==",
		"== Last error ==",
		"Bindings missing:",
		"Project format version:",
		"Godot renderer:",
		"Compute GPU (wgpu",
		missing_path,
	]
	for c in checks:
		var present: bool = text.find(c) >= 0
		print("PROBE contains \"", c, "\" -> ", present)
		if not present:
			fails += 1

	print("PROBE report bytes=", text.length())
	print("PROBE fails=", fails)
	get_tree().quit(0 if fails == 0 else 1)
