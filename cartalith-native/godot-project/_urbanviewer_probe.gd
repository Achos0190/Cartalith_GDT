extends Node
## An open City Viewer and Place editor re-lay their town when an `urban_*` switch moves (2026-11-17, the follow-up to
## `990cf7fd`, which made `EngineBridge.urban_params_changed` drop the MAP's cached towns): before this, a viewer left open
## across the toggle kept the town it had been laid out with, and the editor's Layout section kept its old box chip.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . _urbanviewer_probe.tscn -- [--seed N]
## Run under `APPDATA=<empty dir>`. No pixel is read, so headless is enough. Exit 0 held, 1 failed, 2 could not run.
##
## On the app's own world (800 km, 384 x 288, villages on), the first town whose laid-out box differs between
## `urban_river_stretch` on and off (read straight from `bridge.urban_layouts`, so the choice does not depend on the windows):
##   V1 with the viewer open on it, switching the stretch re-lays the viewer's town: its layout's box becomes the new switch's
##   V2 and back again (the second toggle is seen too: the bridge's own `params_changed` latches, this signal does not)
##   V3 a non-urban parameter leaves the viewer's layout alone (the same dictionary object)
##   V4 a CLOSED viewer is not re-laid (`visible` gate: no engine work for a window nobody sees)
##   E1 with the Place editor open on the same town, its Layout section's "box" chip shows the new box after a toggle
##   E2 and back again
## Mutants (each must turn the named checks red): delete the `urban_params_changed` connect in `city_viewer_window.gd` (V1, V2);
## delete it in `place_editor_window.gd` (E1, E2); drop the `visible` test from the viewer's handler (V4).

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print(("  PASS  " if cond else "  FAIL  ") + what)

func _box(l: Dictionary) -> Vector2:
	return Vector2(float(l.get("wm", 0.0)), float(l.get("hm", 0.0)))

## The "W × H m" text of the Place editor's box chip, or "" when its Layout section shows no such chip.
func _chip_text(n: Node) -> String:
	if n is Label and (n as Label).text.contains(" × ") and (n as Label).text.ends_with(" m"):
		return (n as Label).text
	for c in n.get_children():
		var t := _chip_text(c)
		if t != "":
			return t
	return ""

func _ready() -> void:
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("RESULT: FAIL (timeout)")
		get_tree().quit(2))
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
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var places: Array = bridge.settlements()
	_ok(places.size() > 20, "the world generated (%d settlements)" % places.size())
	## A town the switch changes: the box on and off, from the bridge alone.
	var id := -1
	var box_on := Vector2.ZERO
	var box_off := Vector2.ZERO
	bridge.param_set("urban_river_stretch", true)
	var on_boxes := {}
	for k in places.size():
		var l: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if not l.is_empty():
			on_boxes[k] = _box(l[0])
	bridge.param_set("urban_river_stretch", false)
	for k in on_boxes.keys():
		var l: Array = bridge.urban_layouts(PackedInt32Array([int(k)]))
		if not l.is_empty() and _box(l[0]) != on_boxes[k]:
			id = int(k)
			box_on = on_boxes[k]
			box_off = _box(l[0])
			break
	_ok(id >= 0, "a town the stretch changes exists (#%d: %s with, %s without)" % [id, box_on, box_off])
	if id < 0:
		print("RESULT: FAIL")
		get_tree().quit(2)
		return
	bridge.param_set("urban_river_stretch", true)
	var cv = app.city_viewer_window
	cv.open(id)
	await get_tree().process_frame
	_ok(cv.visible, "the City Viewer is open")
	_ok(_box(cv._layout) == box_on, "the viewer opened on the stretched box %s (%s)" % [box_on, _box(cv._layout)])
	## Toggle with the viewer open.
	bridge.param_set("urban_river_stretch", false)
	await get_tree().process_frame
	_ok(_box(cv._layout) == box_off, "V1 the open viewer re-laid its town: box %s -> %s (wanted %s)" % [box_on, _box(cv._layout), box_off])
	bridge.param_set("urban_river_stretch", true)
	await get_tree().process_frame
	_ok(_box(cv._layout) == box_on, "V2 and back: viewer box %s (wanted %s)" % [_box(cv._layout), box_on])
	## A non-urban write changes nothing.
	var held: Dictionary = cv._layout
	bridge.param_set("river_density", float(bridge.params_snapshot().get("river_density", 1.0)))
	await get_tree().process_frame
	_ok(is_same(cv._layout, held), "V3 a non-urban parameter keeps the viewer's layout (same object)")
	## Closed windows are left alone.
	cv.hide()
	var stale: Dictionary = cv._layout
	bridge.param_set("urban_river_stretch", false)
	await get_tree().process_frame
	_ok(is_same(cv._layout, stale), "V4 a closed viewer is not re-laid")
	## The Place editor, on its Layout tab (one context window shows at a time on the desktop, so the viewer is closed first).
	bridge.param_set("urban_river_stretch", true)
	var pe = app.place_editor_window
	pe.open_for_tab(id, "layout")
	await get_tree().process_frame
	await get_tree().process_frame
	var chip_on := "%d × %d m" % [int(box_on.x), int(box_on.y)]
	var chip_off := "%d × %d m" % [int(box_off.x), int(box_off.y)]
	_ok(pe.visible, "the Place editor is open")
	_ok(_chip_text(pe) == chip_on, "the Place editor opened with the box chip '%s' (shows '%s')" % [chip_on, _chip_text(pe)])
	bridge.param_set("urban_river_stretch", false)
	await get_tree().process_frame
	await get_tree().process_frame
	_ok(_chip_text(pe) == chip_off, "E1 the open Place editor's box chip is '%s' (wanted '%s')" % [_chip_text(pe), chip_off])
	bridge.param_set("urban_river_stretch", true)
	await get_tree().process_frame
	await get_tree().process_frame
	_ok(_chip_text(pe) == chip_on, "E2 and back: editor chip '%s' (wanted '%s')" % [_chip_text(pe), chip_on])
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
