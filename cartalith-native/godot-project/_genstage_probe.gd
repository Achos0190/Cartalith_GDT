extends Node
## Committed probe for the staged generation readout (`ANDROID_BUILD_SCOPE.md`
## Generator: "one Generate button + staged progress readout (10 stages)").
## Modelled on `_cmdindex_probe.gd`.
##
##   Godot_v4.7.1... --headless --path . _genstage_probe.tscn
##
## Boots the real shell, connects to `EngineBridge.generation_stage` BEFORE
## calling `bridge.generate()` on a real (small) grid, collects every
## emission, and asserts the readout is REAL rather than a fake animation:
## at least 6 distinct stages observed, indices non-decreasing, the last
## emission is the final stage (index 9, "Resources & soils"), and every
## emission carries the same run token. A run that observes ZERO stages
## FAILS loudly -- this repository's most-repeated trap is silently-empty
## output, and a probe that quietly accepted "no signals fired" would be
## exactly that trap wearing a green checkmark.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _fail := 0
var _events: Array = []  ## each entry: {"index": int, "name": String, "total": int, "token": int}

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _on_stage(index: int, stage_name: String, total: int, bridge) -> void:
	## `_progress_seen_token` is private, so read the run token the same way
	## `_process` in `engine_bridge.gd` does: through `_progress.snapshot()`.
	var token := -1
	if bridge.progress_api:
		token = int(bridge._progress.snapshot().get("run_token", -1))
	_events.append({"index": index, "name": stage_name, "total": total, "token": token})
	print("  info generation_stage  index=%d name=%s total=%d token=%d" % [index, stage_name, total, token])

func _ready() -> void:
	## A watchdog: this probe must never hang the headless runner even if
	## something above it deadlocks (`_conform_probe.gd`'s own established
	## shape for this).
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("[WATCHDOG] timed out"); get_tree().quit(3))
	wd.start()

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	print("[BOOT] shell up  bridge=", bridge != null)

	print("\n=== 0: the new binding is actually present on this build ===")
	_ok("GenerationProgress registered", ClassDB.class_exists("GenerationProgress"), true)
	_ok("EngineBridge.progress_api resolved true", bridge.progress_api, true)

	print("\n=== 1: run a real (small) generate and collect every stage signal ===")
	bridge.generation_stage.connect(_on_stage.bind(bridge))
	## Big enough that each real stage's own compute (Hydrology's channel
	## build/Strahler/polyline-trace/carve loop in particular -- the one this
	## probe's first cut at a 320x240 grid found could finish inside a single
	## `_process` tick and go unobserved) reliably spans more than one
	## `_process` poll, without making the probe itself slow to run.
	bridge.generate({
		"seed": 133742, "width_km": 1400.0, "grid_w": 768, "grid_h": 576,
		"sea_level": 0.5, "villages": true,
	})
	## Poll rather than a single `await generation_finished` -- this loop is
	## what gives `EngineBridge._process` repeated chances to run while the
	## worker thread is up, the same shape `_conform_probe.gd` uses.
	var waited := 0
	while bridge.generating and waited < 1200:
		await _frames(1)
		waited += 1
	_ok("generation finished before the poll gave up", bridge.generating, false)
	_ok("the bridge produced a world", bridge.has_world, true)
	await _frames(4)

	print("\n=== 2: it did not silently emit nothing ===")
	print("  info total generation_stage emissions: ", _events.size())
	_ok("at least one generation_stage signal fired", _events.size() > 0, true)
	if _events.is_empty():
		print("\n_genstage_probe: FAIL -- zero stage signals observed (silently-empty output)")
		get_tree().quit(1)
		return

	print("\n=== 3: at least 6 distinct stages were observed ===")
	var distinct := {}
	for e in _events:
		distinct[int(e["index"])] = true
	print("  info distinct stage indices seen: ", distinct.keys())
	_ok("distinct stage count >= 6", distinct.size() >= 6, true)

	print("\n=== 4: indices are non-decreasing across the whole run ===")
	var non_decreasing := true
	for i in range(1, _events.size()):
		if int(_events[i]["index"]) < int(_events[i - 1]["index"]):
			non_decreasing = false
			print("  FAIL  index went backward at emission %d: %d -> %d" % [
				i, int(_events[i - 1]["index"]), int(_events[i]["index"])])
	_ok("no emission's index is lower than the one before it", non_decreasing, true)

	print("\n=== 5: the last emission is the final stage ===")
	var last: Dictionary = _events[_events.size() - 1]
	print("  info last emission: ", last)
	_ok("last stage index is stage_count - 1", int(last["index"]), int(last["total"]) - 1)
	_ok("last stage name is Resources & soils", String(last["name"]), "Resources & soils")

	print("\n=== 6: every emission from this run carries the same run token ===")
	var first_token := int(_events[0]["token"])
	var all_same_token := true
	for e in _events:
		if int(e["token"]) != first_token:
			all_same_token = false
	print("  info run token: ", first_token)
	_ok("run token is a real (nonzero) value", first_token > 0, true)
	_ok("every emission in this run shares one run token", all_same_token, true)

	print("\n=== 7: a second generate bumps the run token, not the stage sequence ===")
	_events.clear()
	bridge.generate({
		"seed": 909090, "width_km": 700.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true,
	})
	waited = 0
	while bridge.generating and waited < 1200:
		await _frames(1)
		waited += 1
	await _frames(4)
	_ok("the second run also produced emissions", _events.size() > 0, true)
	if not _events.is_empty():
		var second_token := int(_events[0]["token"])
		print("  info second run token: ", second_token)
		_ok("the second run's token is strictly greater than the first's", second_token > first_token, true)
		var last2: Dictionary = _events[_events.size() - 1]
		_ok("the second run also ends on the final stage", int(last2["index"]), int(last2["total"]) - 1)

	print("\n_genstage_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
