extends Node
## The planned-versus-drawn box row (`OUTSTANDING_WORK.md`, found 2026-11-17 re-pointing `_stretchmap_probe`): does
## `WorldGen.urban_layout_boxes()` report the box each stretched river town is DRAWN in, and what does it cost?
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . _stretchbox_probe.tscn -- [--seed N] [--grid WxH] [--warm K]
##
## Run under a scratch APPDATA. Makes the app's world (800 km, villages on, the app's parameters: `urban_river_stretch` is
## on in them since Ruling FS), then:
##   B1  once every settlement has been laid out (`urban_layouts([k])`, which keeps each town's stretch verdict), its
##       reported box (`urban_layout_boxes`, absent = the reference's 1 700 x 1 250 m) equals the box it is laid out in
##       (`wm`/`hm`); before that a refused town answers its planned box (printed, not a bar: the call never lays out);
##   B2  the same after a place edit on a stretched town (`civ_edit_settlement` walls none, then `civ_set_town_plan`
##       rules preset `market_town`), each followed by a fresh box call;
##   B3  the same for every town after an `urban_*` toggle (`urban_wall_hug` off), and again with it back on;
##   B4  timings, printed, never asserted (a probe is not a benchmark): the first call (already paid by the shell's
##       regenerate path), a cold call (after a switch change, new keys), then `--warm` (default 5) warm calls as
##       median (min..max) milliseconds; and `urban_layouts` of the reported-stretched towns after the box call.
## Before the fix B1 fails on 483920 at 384 x 288 (four towns reported at their planned stretched box, laid out at the
## reference's: #4, #11, #15, #23, the row's own reading). Result line `RESULT: PASS|FAIL`. Exit 0 pass, 1 fail, 2 could not run.

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

## Every settlement whose reported box differs from its laid-out box, as "#k box (w,h) laid (w,h)" strings. Each town in
## `only` (every town when empty) is laid out first (`urban_layouts`, which keeps its stretch verdict), then the boxes
## are read once (`boxes`, when given, instead: the answer before any layout).
func _mismatches(wg: WorldGen, places: Array, only: Array = [], boxes = null) -> Array:
	var lays := {}
	for k in places.size():
		if not only.is_empty() and not only.has(k):
			continue
		var got: Array = wg.urban_layouts(PackedInt32Array([k]))
		if not got.is_empty():
			lays[k] = got[0]
	var bx: Dictionary = boxes if boxes is Dictionary else wg.urban_layout_boxes()
	var bad: Array = []
	for k in lays:
		var l: Dictionary = lays[k]
		var b: Vector2 = bx.get(int(places[k]["tid"]), Vector2(1700.0, 1250.0))
		if absf(b.x - float(l["wm"])) > 0.5 or absf(b.y - float(l["hm"])) > 0.5:
			bad.append("#%d box (%.0f,%.0f) laid (%.0f,%.0f)" % [k, b.x, b.y, float(l["wm"]), float(l["hm"])])
	return bad

func _ms(t0: int) -> float:
	return float(Time.get_ticks_usec() - t0) / 1000.0

func _median(v: Array) -> String:
	var s := v.duplicate()
	s.sort()
	return "%.1f (%.1f..%.1f)" % [s[s.size() / 2], s[0], s[s.size() - 1]]

func _ready() -> void:
	get_tree().create_timer(1800.0).timeout.connect(func() -> void:
		print("RESULT: FAIL (timeout)")
		get_tree().quit(2))
	var seed_n := 483920
	var gw := 384
	var gh := 288
	var warm := 5
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed": seed_n = int(args[i + 1])
			"--grid":
				var wh := String(args[i + 1]).split("x")
				gw = int(wh[0])
				gh = int(wh[1])
			"--warm": warm = int(args[i + 1])
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	var wg: WorldGen = bridge.world_gen
	if not wg.has_method("urban_layout_boxes"):
		print("RESULT: FAIL (no WorldGen.urban_layout_boxes in this build)")
		get_tree().quit(2)
		return
	var places: Array = wg.get_settlements()
	if places.is_empty():
		print("RESULT: FAIL (no settlements)")
		get_tree().quit(2)
		return
	## B4: the first call here is already warm when the shell's own regenerate path asked for the boxes
	## (`ViewportHost.refresh_urban_boxes`), so it is printed as such; a cold call is measured after a switch the
	## verdicts read (`urban_wall_hug`), which gives every stretched town a new key, and the switch is put back.
	var t0 := Time.get_ticks_usec()
	var boxes: Dictionary = wg.urban_layout_boxes()
	var first := _ms(t0)
	## Straight to the engine (`WorldGen.set_params`), not `EngineBridge.param_set`: the bridge's write also tells the
	## shell to drop its towns, which re-asks for the boxes inside that call and would hide the cold cost.
	var hug := bool(bridge.param_get("urban_wall_hug"))
	wg.set_params({"urban_wall_hug": not hug})
	t0 = Time.get_ticks_usec()
	wg.urban_layout_boxes()
	var cold := _ms(t0)
	wg.set_params({"urban_wall_hug": hug})
	t0 = Time.get_ticks_usec()
	wg.urban_layout_boxes()
	print("  first call after the shell's own: %.1f ms; cold (new keys after a switch): %.1f ms; back to the first keys: %.1f ms" % [first, cold, _ms(t0)])
	var warms: Array = []
	for _w in warm:
		t0 = Time.get_ticks_usec()
		wg.urban_layout_boxes()
		warms.append(_ms(t0))
	print("seed %d %dx%d: %d settlements, %d reported stretched; urban_layout_boxes cold %.1f ms, warm %s ms (n=%d)" % [seed_n, gw, gh,
		places.size(), boxes.size(), cold, _median(warms), warm])
	var stretched: Array = []
	for k in places.size():
		if boxes.has(int(places[k]["tid"])):
			stretched.append(k)
	t0 = Time.get_ticks_usec()
	wg.urban_layouts(PackedInt32Array(stretched))
	print("  urban_layouts of the %d reported-stretched towns after the box call: %.1f ms" % [stretched.size(), _ms(t0)])
	## Before any town is laid out a town the bars refuse still answers its planned box (an upper bound; the call never
	## lays a town out): printed, not a bar.
	var early := _mismatches(wg, places, [], boxes)
	print("  before the towns are laid out, planned box not the drawn one: %s" % [early])
	var bad := _mismatches(wg, places)
	print("  mismatches: %s" % [bad])
	_ok(bad.is_empty(), "B1 once laid out, every reported box is the laid-out box (%d towns)" % places.size())
	## B2: a place edit on a stretched town, then a rules preset.
	if stretched.is_empty():
		_ok(false, "B2 premise: a stretched town on this world")
	else:
		var k: int = stretched[0]
		_ok(wg.civ_edit_settlement(k, {"walls": 0}), "B2 premise: walls set to none on #%d" % k)
		var bad2 := _mismatches(wg, places, [k])
		print("  after walls none: %s" % [bad2])
		_ok(bad2.is_empty(), "B2 the box follows a walls edit (#%d)" % k)
		_ok(wg.civ_set_town_plan(k, {"rules_preset": "market_town"}), "B2 premise: rules preset set on #%d" % k)
		var bad3 := _mismatches(wg, places, [k])
		print("  after the rules preset: %s" % [bad3])
		_ok(bad3.is_empty(), "B2 the box follows a rules edit (#%d)" % k)
	## B3: an urban_* toggle and back.
	for on in [false, true]:
		_ok(bridge.param_set("urban_wall_hug", on), "B3 premise: urban_wall_hug set %s" % on)
		var bad4 := _mismatches(wg, places)
		print("  urban_wall_hug %s: %s" % [on, bad4])
		_ok(bad4.is_empty(), "B3 every reported box is the laid-out box with urban_wall_hug %s" % on)
	print("RESULT: %s (%d checks, %d failed)" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	get_tree().quit(0 if _fails == 0 else 1)
