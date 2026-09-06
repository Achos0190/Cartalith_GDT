extends Node
## Owner ruling 21: `statusMid`'s `repaint NN ms` measures `_refresh_map()`'s
## wall time. This probe establishes the two things that claim rests on.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _repaintbracket_probe.tscn \
##       -- --nowelcome --runs 9
##
## **Windowed, deliberately, and NOT `--headless`.** `ViewportHost.refresh()`
## assigns textures the bridge builds; under the dummy driver
## `ImageTexture.update()` is a no-op (`MISTAKES.md`'s pixel-probe row), so a
## headless run times a path the shipping build does not take. Logic-only
## checks would be fine headless; a duration is not.
##
## ## 1. The bracket encloses exactly one handler, and it is `ViewportHost`'s
##
## `app.gd` opens the clock on `generation_finished`/`world_loaded` immediately
## before `viewport.setup(bridge)` and closes it immediately after, and Godot
## delivers a signal in connection order -- so the bracket times whatever is
## registered between those two statements. That is an assumption about
## ordering, so it is asserted rather than asserted-in-prose: this walks
## `Signal.get_connections()` in order and names every connection that falls
## between the two probes.
##
## ## 2. The duration itself
##
## `--runs N` calls `DccApp.repaint_map()` N times over one generated world and
## reads `_repaint_ms` back after each. Reported as a median with min..max --
## a point estimate is not a measurement here, and three have already had to be
## withdrawn from this repository. Run the harness alone.
##
## `--runs` defaults to 9 and the first sample is reported separately and
## excluded from the median: it is the only one that pays for first-touch
## allocation of the textures and of `MapOverlay`'s arrays, so folding it in
## measures the boot as well as the repaint.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("RPT %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _generate(seed_v: int) -> bool:
	app.bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	return app.bridge.has_world

## The `mid` slot as DRAWN, not as composed: `set_status()` writes a Label and
## `_apply_status_omission()` can take the slot out entirely, so reading the
## Label is the only reading that says what a user sees.
func _mid_text() -> String:
	var slot: Node = app._status_labels.get("mid")
	return "" if slot == null else String((slot as Label).text)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	## -- 1. What the bracket encloses ---------------------------------------
	for sig_name in ["generation_finished", "world_loaded"]:
		var sig: Signal = app.bridge.get(sig_name)
		var conns: Array = sig.get_connections()
		var open_at := -1
		var close_at := -1
		var between := PackedStringArray()
		for i in conns.size():
			var cb: Callable = conns[i]["callable"]
			if cb.get_object() != app:
				continue
			## Both halves are NAMED methods on `DccApp` -- `_repaint_open` /
			## `_repaint_close` on `world_loaded`, and the `_if` pair that
			## carries `generation_finished`'s `ok` guard. That is why they are
			## named and not lambdas: a lambda reports its enclosing function
			## as `get_method()`, so this walk could only have guessed.
			var m := cb.get_method()
			if m == "_repaint_open" or m == "_repaint_open_if":
				open_at = i
			elif m == "_repaint_close" or m == "_repaint_close_if":
				close_at = i
		var names := PackedStringArray()
		for i in conns.size():
			var cb3: Callable = conns[i]["callable"]
			var obj: Object = cb3.get_object()
			names.append("%d:%s.%s" % [i,
				"<null>" if obj == null else obj.get_class(), cb3.get_method()])
		print("RPT %s connections: %s" % [sig_name, " | ".join(names)])
		if open_at < 0 or close_at < 0:
			_check("%s: both clock probes are connected" % sig_name, false,
				"open_at=%d close_at=%d" % [open_at, close_at])
			continue
		## **By object identity, not by class name.** `Object.get_class()`
		## reports the NATIVE base -- a `ViewportHost` answers `Control` -- so a
		## `begins_with("ViewportHost.")` test fails on the correct answer. The
		## question here is "is the one enclosed handler the viewport's", and
		## `app.viewport` is the object to compare against.
		var all_viewport := true
		for i in range(open_at + 1, close_at):
			var cb4: Callable = conns[i]["callable"]
			var obj4: Object = cb4.get_object()
			if obj4 != app.viewport:
				all_viewport = false
			between.append("%s(%s).%s" % [
				"<null>" if obj4 == null else obj4.get_class(),
				"app.viewport" if obj4 == app.viewport else "other", cb4.get_method()])
		_check("%s: the bracket encloses exactly one handler" % sig_name,
			between.size() == 1, "enclosed: %s" % " | ".join(between))
		_check("%s: and it is the ViewportHost's" % sig_name,
			between.size() == 1 and all_viewport,
			"enclosed: %s" % " | ".join(between))

	## -- A world to repaint --------------------------------------------------
	var ok: bool = await _generate(70021)
	_check("world generated", ok)
	if not ok:
		print("RPT failures=%d" % _fail)
		get_tree().quit(1)
		return

	## The generate itself went through the bracket, so this is the first real
	## reading and it is a `generation_finished` one, not a `repaint_map()` one.
	print("RPT after generate: _repaint_ms=%.3f  mid=%s" % [
		float(app.get("_repaint_ms")), _mid_text()])
	_check("`repaint NN ms` is drawn in the mid slot after a generate",
		_mid_text().contains("repaint ") and _mid_text().contains(" ms"),
		"mid=%s" % _mid_text())

	## -- 2. The duration -----------------------------------------------------
	var runs := maxi(2, int(_arg("--runs", "9")))
	var samples: Array[float] = []
	for i in runs:
		app.repaint_map()
		samples.append(float(app.get("_repaint_ms")))
		await _frames(2)
	var first: float = samples[0]
	var rest: Array[float] = samples.slice(1)
	rest.sort()
	var med: float = rest[rest.size() / 2] if rest.size() % 2 == 1 \
		else (rest[rest.size() / 2 - 1] + rest[rest.size() / 2]) * 0.5
	var raw := PackedStringArray()
	for s in samples:
		raw.append("%.2f" % s)
	print("RPT repaint_map() samples (ms, in order): %s" % " ".join(raw))
	print("RPT first=%.2f ms (excluded)  median=%.2f ms (%.2f..%.2f) over %d runs"
		% [first, med, rest[0], rest[rest.size() - 1], rest.size()])
	print("RPT mid after repaint_map(): %s" % _mid_text())
	_check("every sample is a real duration, not the -1.0 sentinel",
		rest[0] >= 0.0, "min=%.3f" % rest[0])

	## -- "No value" must not be a plausible value ----------------------------
	## **Both branches**, not just the live one. A guard that reads `>= 0.0`
	## looks obviously right and is worth nothing unless the OTHER branch is
	## exercised: if the sentinel were `0.0`, the first check would still pass
	## while a never-measured shell silently printed `repaint 0 ms`. So drive
	## the composite at the sentinel and at a real zero and require they differ.
	var keep: float = float(app.get("_repaint_ms"))
	app.set("_repaint_ms", -1.0)
	app._refresh_status_mid()
	var at_sentinel := _mid_text()
	app.set("_repaint_ms", 0.0)
	app._refresh_status_mid()
	var at_zero := _mid_text()
	app.set("_repaint_ms", keep)
	app._refresh_status_mid()
	_check("the sentinel omits the field", not at_sentinel.contains("repaint"),
		"mid=%s" % at_sentinel)
	_check("a real 0 ms still DRAWS the field", at_zero.contains("repaint 0 ms"),
		"mid=%s" % at_zero)

	print("RPT failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
