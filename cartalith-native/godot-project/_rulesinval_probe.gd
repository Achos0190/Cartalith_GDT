extends Node
## Windowed proof that a Generation rules change redraws towns already cached
## on the map (`OUTSTANDING_WORK.md` §2.10, 2026-09-24). Before the fix the
## rules window changed the engine's rules and rebuilt only itself, so every
## already-cached town kept drawing under the old rules.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _rulesinval_probe.tscn
##
## On a REAL generated world:
##   1. a town is laid out and cached through the viewport's own request path;
##   2. control: rebuilding the rules window WITHOUT a rules change keeps it;
##   3. applying a different preset drops it from the map's cache;
##   4. re-requesting it gives a different street layout -- the rules reach the
##      generator, so dropping the cache was necessary, not cosmetic.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("RULESINV %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _fp(l: Dictionary) -> String:
	return "%d edges / %.1f m" % [int(l.get("edge_count", -1)), float(l.get("street_len_m", -1.0))]

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("RULESINV REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("RULESINV WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	var host: Node = app.viewport
	var ov: Node = host.overlay
	var win: Node = app.generation_rules_window
	win._do_reset()
	await _frames(2)

	# 1. find a town that lays out, and cache it through the real request path
	var idx := -1
	var n: int = (bridge.settlements() as Array).size()
	for i in range(mini(n, 40)):
		host._on_urban_layouts_needed(PackedInt32Array([i]))
		if ov._urban_layouts.has(i) and int((ov._urban_layouts[i] as Dictionary).get("edge_count", 0)) > 0:
			idx = i
			break
	_check("a town is laid out and cached", idx >= 0, "index %d of %d" % [idx, n])
	if idx < 0:
		get_tree().quit(1)
		return
	var before := _fp(ov._urban_layouts[idx])

	# 2. control: no rules change, no drop
	win._rebuild()
	await _frames(2)
	_check("control: rebuilding the window alone keeps the cache", ov._urban_layouts.has(idx))

	# 3. a real rules change drops it
	var wild: Dictionary = {}
	for p in GenerationRulesWindow._PRESETS:
		if String(p["label"]) == "Wild Frontier":
			wild = p
	win._apply_preset(wild)
	await _frames(2)
	_check("a preset change drops the cached town", not ov._urban_layouts.has(idx))

	# 4. and the re-requested town is genuinely different
	host._on_urban_layouts_needed(PackedInt32Array([idx]))
	var after := _fp(ov._urban_layouts.get(idx, {}))
	_check("the re-laid-out town differs under the new rules", after != before,
		"%s -> %s" % [before, after])

	win._do_reset()
	print("RULESINV %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
