extends Node
## Ruling FS (owner, 2026-11-17), the routed row "Toggling an `urban_*` switch live does not drop cached town layouts":
## protects `EngineBridge.urban_params_changed` and its listener in `viewport_host.gd::setup`. Town layouts read the
## `urban_*` switches when they are laid out, so the map's cached layouts (`map_overlay.gd::_urban_layouts`) are stale
## once one moves; before this, an A/B toggle in the app showed the old town until the cache was cleared by hand.
##
## Usage (headless is enough: no pixel is read):
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . _urbancache_probe.tscn -- [--seed N]
## Run under `APPDATA=<empty dir>`. Prints PASS/FAIL per check and `RESULT: PASS|FAIL`; exits 1 on any failure.
##
## Checks, on the app's own world (800 km, 384 x 288): the map's cache holds the towns it was handed; an `urban_*`
## write empties it; a second toggle back empties it again (the bridge's `params_changed` latches until the next
## generate, so a listener on it alone would miss this one); a non-urban write leaves it; a keyed reset naming an
## `urban_*` key empties it; and the layout fetched after a toggle is the new switch's (a stretched town's box changes).

var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print(("  PASS  " if cond else "  FAIL  ") + what)


func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void: get_tree().quit(2))
	var seed_n := 483920
	var args := OS.get_cmdline_user_args()
	for i in range(0, args.size() - 1, 2):
		if args[i] == "--seed":
			seed_n = int(args[i + 1])
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": 384, "grid_h": 288, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout
	var overlay = app.viewport.overlay
	var places: Array = bridge.settlements()
	_ok(places.size() > 70, "the world generated (%d settlements)" % places.size())
	# A town the stretch reaches: the first with a planned box (the switch is on in a new world, Ruling FS).
	var boxes: Dictionary = bridge.urban_layout_boxes()
	var id := -1
	for k in places.size():
		if boxes.has(int(places[k].get("tid", -1))):
			id = k
			break
	_ok(id >= 0, "a new world plans at least one stretched town (%d boxes; town #%d)" % [boxes.size(), id])
	var batch := PackedInt32Array([0, 1, 2])
	if id >= 0 and not batch.has(id):
		batch.append(id)
	var fill := func() -> void: overlay.set_urban_layouts(batch, bridge.urban_layouts(batch))
	fill.call()
	_ok(overlay._urban_layouts.size() == batch.size(), "the map caches the %d towns it was handed (%d)" % [batch.size(), overlay._urban_layouts.size()])
	var before: Dictionary = overlay._urban_layouts.get(id, {})
	_ok(bridge.param_set("urban_river_stretch", false), "urban_river_stretch set off")
	_ok(overlay._urban_layouts.is_empty(), "an urban_* write empties the map's town cache (%d left)" % overlay._urban_layouts.size())
	fill.call()
	var after: Dictionary = overlay._urban_layouts.get(id, {})
	_ok(id < 0 or (Vector2(before.get("wm", 0.0), before.get("hm", 0.0)) != Vector2(after.get("wm", 0.0), after.get("hm", 0.0))),
		"town #%d is re-laid with the new switch: box %s -> %s" % [id, Vector2(before.get("wm", 0.0), before.get("hm", 0.0)), Vector2(after.get("wm", 0.0), after.get("hm", 0.0))])
	_ok(bridge.param_set("urban_river_stretch", true), "urban_river_stretch set back on")
	_ok(overlay._urban_layouts.is_empty(), "the second toggle empties it again (%d left)" % overlay._urban_layouts.size())
	fill.call()
	_ok(bridge.param_set("river_density", float(bridge.params_snapshot().get("river_density", 1.0))), "a non-urban parameter written")
	_ok(overlay._urban_layouts.size() == batch.size(), "a non-urban write keeps the cache (%d)" % overlay._urban_layouts.size())
	bridge.reset_params(["urban_wall_hug"])
	_ok(overlay._urban_layouts.is_empty(), "a keyed reset naming an urban_* key empties it (%d left)" % overlay._urban_layouts.size())
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
