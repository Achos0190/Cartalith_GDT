extends Node
## `Preferences ▸ Graphics ▸ Lighting rig defaults` — §2.5's "azimuth,
## elevation, ambient, multidirectional on/off".
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _lighting_probe.tscn
##
## The row's own reason said the only missing piece was "the project-level
## default store those per-layer values would seed from — a settings key, not
## new rendering". So what this asserts is that the key exists AND that it
## actually reaches the renderer: a settings value nothing reads would satisfy
## every structural check and change nothing on screen.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _near(name: String, got: float, want: float, tol: float) -> void:
	var good := absf(got - want) <= tol
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name,
		"   got=%.3f want=%.3f (tol %.3f)" % [got, want, tol])

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
	DccSettings.reset_lighting_defaults()
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	print("[BOOT] shell up · defaults reset to the reference rig")

	print("")
	print("=== 1: the four ladders exist and none is disabled ===")
	var az := _find_popup(app, 170)
	var alt := _find_popup(app, 190)
	var amb := _find_popup(app, 210)
	var num := _find_popup(app, 230)
	_ok("Azimuth ladder", az != null, true)
	_ok("Elevation ladder", alt != null, true)
	_ok("Ambient ladder", amb != null, true)
	_ok("Multidirectional ladder", num != null, true)
	if az == null or alt == null or amb == null or num == null:
		get_tree().quit(1); return
	var any_disabled := false
	for pair in [[az, 170, 8], [alt, 190, 7], [amb, 210, 6], [num, 230, 6]]:
		var pm: PopupMenu = pair[0]
		for k in int(pair[2]):
			var i := pm.get_item_index(int(pair[1]) + k)
			if i >= 0 and pm.is_item_disabled(i):
				any_disabled = true
	_ok("no rung anywhere is disabled", any_disabled, false)
	print("  info azimuth rungs: ", az.item_count, "  elevation: ", alt.item_count,
		"  ambient: ", amb.item_count, "  lights: ", num.item_count)

	print("")
	print("=== 2: the reference rig is what an untouched install carries ===")
	var d: Dictionary = DccSettings.lighting_defaults()
	print("  info ", d)
	_near("azimuth 315", float(d["sun_az_deg"]), 315.0, 0.01)
	_near("elevation 45", float(d["sun_alt_deg"]), 45.0, 0.01)
	_near("one light (the reference's single sun)", float(d["relief_lights"]), 1.0, 0.01)

	print("")
	print("=== 3: the MENU writes the store ===")
	az.id_pressed.emit(170 + 4)       ## 180 deg
	num.id_pressed.emit(230 + 4)      ## 8 lights
	await _frames(4)
	var d2: Dictionary = DccSettings.lighting_defaults()
	_near("azimuth is now 180", float(d2["sun_az_deg"]), 180.0, 0.01)
	_near("lights is now 8", float(d2["relief_lights"]), 8.0, 0.01)
	az.about_to_popup.emit()
	await _frames(2)
	_ok("...and the ladder checks that rung", az.is_item_checked(az.get_item_index(174)), true)
	_ok("...and only that rung", az.is_item_checked(az.get_item_index(177)), false)

	print("")
	print("=== 4: a Generate actually applies it — the half a settings key alone would fake ===")
	bridge.generate({"seed": 771, "width_km": 600.0, "grid_w": 192, "grid_h": 144,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(16)
	var live: Dictionary = bridge.appearance()
	print("  info engine sun_az_deg=", live.get("sun_az_deg", "?"),
		"  relief_lights=", live.get("relief_lights", "?"))
	_near("the ENGINE now reports azimuth 180", float(live.get("sun_az_deg", -1.0)), 180.0, 0.01)
	_near("...and 8 light directions", float(live.get("relief_lights", -1.0)), 8.0, 0.01)

	print("")
	print("=== 5: reset erases rather than storing a copy of the default ===")
	## A stored copy of a default is a default that cannot move: change
	## LIGHTING_DEFAULTS later and anyone who once pressed Reset keeps the old
	## number forever. Erasing the section is what avoids that.
	_find_popup(app, 250).id_pressed.emit(250)
	await _frames(4)
	var d3: Dictionary = DccSettings.lighting_defaults()
	_near("azimuth back to 315", float(d3["sun_az_deg"]), 315.0, 0.01)
	_near("lights back to 1", float(d3["relief_lights"]), 1.0, 0.01)

	print("")
	print("_lighting_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
