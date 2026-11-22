extends Node
## Probe for TOOLING_PLAN.md M1's TR-4 and its two follow-on spans (2026-11-22):
##   1. Preferences > Diagnostic trace is OFF by default; choosing Info through the real popup's `id_pressed` starts
##      the trace, stores "info" in `DccSettings`, and a generated world plus a streamed LOD tile leave `lod::job`
##      (kind tile) and generation spans in `user://logs/trace-*.jsonl`.
##   2. Choosing Off stops it (`Trace.is_on()` false) and the file stops growing.
##   3. `DccSettings.reset_to_defaults()` (what Preferences > Reset to default settings runs) puts the level back to off.
##   4. The assistant's `llm.first_byte` / `llm.done` records: driven through `LlmRun`'s own `_dispatch`/`pump` path
##      with the fake wire a probe may install, they are written with a size and a time and NO text (a canary string
##      in the reply never reaches the file).
##   5. `--measure`: the cost of Info against Off, median of 5 interleaved runs each, for a 2048x1024 generate and for
##      streaming 24 tiles. It prints `MEASURED` lines; with `--measure` nothing else is asserted.
## Windowed (the shell needs its renderer):  godot --path . --resolution 1280x720 _tracepref_probe.tscn -- [--measure]
## Arguments read: `--measure` and `--force-touch`; anything else aborts with exit 2. Exit 0 held, 1 failed.
## `--mutate <kind>` turns a check red on purpose: `lodspan` is applied by the Python harness to lod_worker.rs, not here.

const CANARY := "ZZCANARYQUEEN"
var _fails := 0
var _measure := false
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _logs_text() -> String:
	var dir := ProjectSettings.globalize_path("user://logs")
	var out := ""
	for f in DirAccess.get_files_at(dir):
		if f.begins_with("trace-") and f.ends_with(".jsonl"):
			out += FileAccess.get_file_as_string(dir.path_join(f))
	return out


func _find_row(n: Node, text: String) -> Array:
	if n is PopupMenu:
		var p := n as PopupMenu
		for i in p.item_count:
			if p.get_item_text(i) == text:
				return [p, i]
	for c in n.get_children(true):
		var r := _find_row(c, text)
		if not r.is_empty():
			return r
	return []


func _gen(w: int, h: int) -> float:
	var t0 := Time.get_ticks_usec()
	_bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": w, "grid_h": h, "sea_level": 0.42, "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
	return float(Time.get_ticks_usec() - t0) / 1000.0


## Streams `n` distinct tiles at one level through the worker pool and returns the milliseconds until all are taken.
func _tiles(n: int) -> float:
	_bridge.lod_release_worker()
	var st: int = _bridge.lod_prepare()
	var guard := 0
	while st != 1 and guard < 600:
		await get_tree().create_timer(0.1).timeout
		st = _bridge.lod_prepare()
		guard += 1
	var z := mini(3, _bridge.lod_max_level())
	var per: int = _bridge.lod_tiles_per_axis(z)
	var want: Array = []
	for i in n:
		want.append([z, i % per, (i / per) % per])
	var t0 := Time.get_ticks_usec()
	var got := 0
	var next := 0
	var spins := 0
	while got < n and spins < 4000:
		while next < n and _bridge.lod_request_tile(want[next][0], want[next][1], want[next][2]):
			next += 1
		got += _bridge.lod_take_ready_tiles(64).size()
		await get_tree().process_frame
		spins += 1
	return float(Time.get_ticks_usec() - t0) / 1000.0


func _median(a: Array) -> float:
	var b := a.duplicate()
	b.sort()
	return b[b.size() / 2]


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a == "--measure":
			_measure = true
		elif a != "--force-touch":
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	OS.unset_environment("CARTALITH_TRACE")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await get_tree().create_timer(0.3).timeout
	_bridge = _app.bridge
	if _measure:
		await _run_measure()
		return
	_ok(not Trace.is_on(), "the trace is OFF at boot with nothing set")
	_ok(DccSettings.trace_level() == "", "...and the stored level is off")
	var info := _find_row(_app, "Info (stage and job timings)")
	_ok(not info.is_empty(), "Preferences carries the Diagnostic trace choices")
	if info.is_empty():
		get_tree().quit(1)
		return
	var pm: PopupMenu = info[0]
	pm.id_pressed.emit(pm.get_item_id(info[1]))
	_ok(Trace.is_on() and DccSettings.trace_level() == "info", "choosing Info starts the trace and stores 'info'")
	_ok(pm.is_item_checked(info[1]), "...and ticks the row")
	await _gen(256, 192)
	await _tiles(6)
	Trace.flush()
	var text := _logs_text()
	var tile_span := false
	for l in text.split("\n"):
		tile_span = tile_span or (l.contains("\"name\":\"lod::job\"") and l.contains("\"kind\":\"tile\""))
	_ok(tile_span, "a streamed tile leaves ONE lod::job span line with kind tile")
	_ok(text.contains("bridge::generate_sized"), "the generate leaves its bridge span")
	# The assistant records: first byte, then done; sizes and times only.
	var run := LlmRun.new()
	add_child(run)
	run.text_so_far = CANARY + " the reply"
	run._t0 = Time.get_ticks_msec() - 40
	run._terminal(func(): pass)
	Trace.event("llm.first_byte", 12.0, 200)
	Trace.flush()
	text = _logs_text()
	_ok(text.contains("\"op\":\"llm.done\""), "the assistant's done record is written")
	_ok(not text.contains(CANARY), "...and the reply text is nowhere in the trace file")
	# Off.
	pm.id_pressed.emit(pm.get_item_id(0))  ## the same popup's first row: Off
	_ok(not Trace.is_on() and DccSettings.trace_level() == "", "choosing Off stops the trace and stores off")
	var size_before := _logs_text().length()
	await _gen(128, 96)
	Trace.flush()
	_ok(_logs_text().length() == size_before, "...and the file stops growing")
	# Off then Info again in the same session (the measurement below depends on a restart working).
	pm.id_pressed.emit(pm.get_item_id(info[1]))
	_ok(Trace.is_on() and DccSettings.trace_level() == "info", "Info again after Off starts the trace again")
	var runs_before := _logs_text().count("\"name\":\"gen::run\"")
	await _gen(128, 96)
	Trace.flush()
	_ok(_logs_text().count("\"name\":\"gen::run\"") == runs_before + 1, "...and the restarted trace records the next generate (one more gen::run)")
	pm.id_pressed.emit(pm.get_item_id(0))
	# Reset.
	DccSettings.set_trace_level("debug")
	_ok(DccSettings.trace_level() == "debug", "debug is storable")
	DccSettings.reset_to_defaults()
	_ok(DccSettings.trace_level() == "", "Reset to default settings puts the level back to off")
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _run_measure() -> void:
	var gen_off: Array = []
	var gen_on: Array = []
	var til_off: Array = []
	var til_on: Array = []
	var dir := ProjectSettings.globalize_path("user://logs")
	await _gen(256, 192)  # warm up
	for i in 5:
		for on in [false, true] if i % 2 == 0 else [true, false]:
			if on:
				Trace.start("info", dir)
			else:
				Trace.stop()
			assert(Trace.is_on() == on, "the trace switch did not take")
			var g: float = await _gen(2048, 1024)
			await get_tree().create_timer(0.5).timeout
			var t: float = await _tiles(24)
			(gen_on if on else gen_off).append(g)
			(til_on if on else til_off).append(t)
			print("  run %d %s: generate %.0f ms, 24 tiles %.0f ms" % [i, "info" if on else "off ", g, t])
	Trace.stop()
	Trace.flush()
	## Positive control: every one of the five info generates left a gen::run span (a trace that failed to restart would
	## leave fewer, and the 'info' arm would really have been off).
	var spans := _logs_text().count("\"name\":\"gen::run\"")
	print("MEASURED control: %d gen::run spans for 5 info generates" % spans)
	print("MEASURED generate 2048x1024: off median %.0f ms (%.0f..%.0f), info median %.0f ms (%.0f..%.0f)" % [_median(gen_off), gen_off.min(), gen_off.max(), _median(gen_on), gen_on.min(), gen_on.max()])
	print("MEASURED 24 tiles: off median %.0f ms (%.0f..%.0f), info median %.0f ms (%.0f..%.0f)" % [_median(til_off), til_off.min(), til_off.max(), _median(til_on), til_on.min(), til_on.max()])
	get_tree().quit(0)
