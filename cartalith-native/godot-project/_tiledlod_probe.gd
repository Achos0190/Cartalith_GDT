extends Node
## `Preferences ▸ Tiles & LOD ▸ Tiled LOD` — §2.5's "auto on zoom (default) ·
## manual", the reference's `state.lodAuto`.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _tiledlod_probe.tscn
##
## The row was disabled with the reason *"there is no public suppressor, so a
## manual row here would be the second half of a radio pair that does
## nothing"*. So what this asserts is precisely that manual DOES something, and
## that it is not a trap: a suppressor with no way back in would make deep
## detail unreachable, which is worse than not offering the choice.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _find_popup(app: Node, id: int) -> PopupMenu:
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is PopupMenu and (c as PopupMenu).get_item_index(id) >= 0:
				return c as PopupMenu
	return null

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var was := DccSettings.lod_auto()
	DccSettings.set_lod_auto(true)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	var vp = app.get("viewport")
	print("[BOOT] shell up · stored lod_auto on entry: ", was)

	print("")
	print("=== 1: the rows exist and both modes are offered ===")
	var pop := _find_popup(app, 160)
	_ok("a menu carries the Tiled LOD submenu", pop != null, true)
	if pop == null:
		get_tree().quit(1); return
	_ok("Auto on zoom row", pop.get_item_index(160) >= 0, true)
	_ok("Manual row", pop.get_item_index(161) >= 0, true)
	_ok("Enter deep detail now row", pop.get_item_index(162) >= 0, true)
	_ok("Leave deep detail row", pop.get_item_index(163) >= 0, true)
	_ok("neither mode row is disabled",
		pop.is_item_disabled(pop.get_item_index(160))
			or pop.is_item_disabled(pop.get_item_index(161)), false)

	print("")
	print("=== 2: a world, and a camera zoomed past the threshold ===")
	bridge.generate({"seed": 5150, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	vp.zoom_step(8.0)
	await _frames(14)
	_ok("auto mode brought the pyramid up on its own", vp.lod_active(), true)

	print("")
	print("=== 3: MANUAL actually suppresses — the whole claim ===")
	pop.id_pressed.emit(161)
	await _frames(10)
	_ok("the mode took", vp.lod_auto(), false)
	_ok("...and it persisted", DccSettings.lod_auto(), false)
	## Back out and in again: auto would re-enter on the way in, manual must not.
	vp.zoom_step(1.0 / 8.0)
	await _frames(12)
	_ok("zooming out drops it (both modes do)", vp.lod_active(), false)
	vp.zoom_step(8.0)
	await _frames(14)
	_ok("zooming back IN does NOT re-enter under manual", vp.lod_active(), false)

	print("")
	print("=== 4: ...and manual is not a trap — there is a way in ===")
	pop.id_pressed.emit(162)          ## Enter deep detail now
	await _frames(10)
	_ok("the explicit request brought it up", vp.lod_active(), true)
	pop.id_pressed.emit(163)          ## Leave deep detail
	await _frames(8)
	_ok("and the explicit release drops it", vp.lod_active(), false)

	print("")
	print("=== 5: a request the camera cannot honour REPORTS, not silently fails ===")
	vp.zoom_step(1.0 / 8.0)
	await _frames(12)
	_ok("fitted view: nothing to enter", vp.request_lod_entry(), false)
	## And the refused request must not stay armed to fire on a later zoom the
	## user never connected to it.
	vp.zoom_step(8.0)
	await _frames(14)
	_ok("the refused request did not arm itself for later", vp.lod_active(), false)

	print("")
	print("=== 6: switching back to auto restores the old behaviour ===")
	pop.id_pressed.emit(160)
	await _frames(12)
	_ok("auto is back", vp.lod_auto(), true)
	_ok("...and the pyramid came up without asking", vp.lod_active(), true)

	print("")
	print("=== 7: the Leave row refuses under auto, with a reason ===")
	pop.about_to_popup.emit()
	await _frames(4)
	var li := pop.get_item_index(163)
	_ok("Leave is disabled while auto is on", pop.is_item_disabled(li), true)
	_ok("...and says why", pop.get_item_tooltip(li).to_lower().find("auto") >= 0, true)

	DccSettings.set_lod_auto(was)
	print("  info restored lod_auto to ", DccSettings.lod_auto())
	print("")
	print("_tiledlod_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
