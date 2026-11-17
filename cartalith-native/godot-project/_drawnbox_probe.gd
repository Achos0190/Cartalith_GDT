extends Node
## Town leftovers, third pass (2026-11-17), the stale-"planned box" row `096d1407` handed to the GDScript lane: protects
## `map_overlay.gd::_adopt_drawn_box`, which `set_urban_layouts` calls so the pin claim (`_town_half_km`) follows the box a
## town is DRAWN in once its layout has arrived. `WorldGen.urban_layout_boxes` answers the planned box (an upper bound) for
## a town the engine has not laid out, and `ViewportHost.refresh_urban_boxes` runs when the roster is applied, before
## any town is laid out; a stretched town the bars then refuse is drawn in the reference's box, so without the adoption
## the overlay kept claiming pins out to the planned box's reach.
##
## Usage (headless is enough: no pixel is read; run under an empty `APPDATA`):
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . _drawnbox_probe.tscn -- [--seed N]
## Prints PASS/FAIL per check and `RESULT: PASS|FAIL`; exit 0 pass, 1 fail, 2 timeout.
## Checks, on the app's world (800 km, 384 x 288, the app's own parameters): D0 positive control, at least one planned box
## is wider than the town's drawn box (else the probe proves nothing); D1 after the layouts arrive the claim equals the
## drawn reach for every town laid out, as a literal (`max(wm, hm) / 2000` km); D2 the box dictionary agrees with the
## engine's own report of the drawn box (`urban_layout_boxes()` after `urban_layouts`); D3 the revision bumps when an
## entry changed and not when a layout is handed in a second time.

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print(("  PASS  " if cond else "  FAIL  ") + what)

func _ready() -> void:
	get_tree().create_timer(1500.0).timeout.connect(func() -> void: get_tree().quit(2))
	var seed_n := 912345
	var args := OS.get_cmdline_user_args()
	for i in range(0, args.size() - 1, 2):
		if args[i] == "--seed":
			seed_n = int(args[i + 1])
		else:
			push_error("unknown argument " + args[i])
			get_tree().quit(2)
			return
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
	# The roster was applied before any town was laid out: planned boxes only.
	app.viewport.forget_all_urban_layouts()
	var planned: Dictionary = overlay._urban_boxes.duplicate()
	_ok(planned.size() > 0, "the first push holds planned boxes (%d)" % planned.size())
	var idx := PackedInt32Array()
	for k in places.size():
		if planned.has(int(places[k]["tid"])):
			idx.append(k)
	var rev0: int = overlay._urban_boxes_rev
	var got: Array = []
	for j in range(0, idx.size(), 4):
		var chunk := idx.slice(j, mini(j + 4, idx.size()))
		var lays: Array = bridge.urban_layouts(chunk)
		overlay.set_urban_layouts(chunk, lays)
		got.append_array(lays)
	_ok(got.size() > 0, "layouts came back for %d of %d planned towns" % [got.size(), idx.size()])
	var narrower := 0
	var bad := 0
	for l: Dictionary in got:
		var s: Dictionary = places[int(l["index"])]
		var pb: Vector2 = planned.get(int(s["tid"]), Vector2(1700.0, 1250.0))
		if maxf(pb.x, pb.y) > maxf(float(l["wm"]), float(l["hm"])) + 0.5:
			narrower += 1
		var want_km := maxf(float(l["wm"]), float(l["hm"])) * 0.0005
		if absf(overlay._town_half_km(s) - want_km) > 0.0005:
			bad += 1
	_ok(narrower > 0, "D0 control: %d of %d planned boxes were wider than the box drawn" % [narrower, got.size()])
	_ok(bad == 0, "D1 the pin claim equals max(wm, hm) / 2000 km for all %d towns laid out (%d differ)" % [got.size(), bad])
	var eng: Dictionary = bridge.urban_layout_boxes()
	var disagree := 0
	for l: Dictionary in got:
		var tid := int(places[int(l["index"])]["tid"])
		var e: Vector2 = eng.get(tid, Vector2(1700.0, 1250.0))
		var o: Vector2 = overlay._urban_boxes.get(tid, Vector2(1700.0, 1250.0))
		if e.distance_to(o) > 0.5:
			disagree += 1
	_ok(disagree == 0, "D2 the overlay's boxes equal the engine's reported drawn boxes (%d differ)" % disagree)
	var rev1: int = overlay._urban_boxes_rev
	_ok(rev1 > rev0, "D3 the revision moved when entries changed (%d -> %d)" % [rev0, rev1])
	overlay.set_urban_layouts(idx.slice(0, 1), got.slice(0, 1))
	_ok(overlay._urban_boxes_rev == rev1, "D3 handing a layout in again leaves the revision (%d)" % overlay._urban_boxes_rev)
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
