extends Node
## `Preferences > Tiles & LOD > Tile size · LOD levels > LOD levels` -- SS2.5's
## "levels 0-8", and the claim that it and `WORLD > Finalize > Bake depth` are
## ONE setting rather than two copies.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _lodlevels_probe.tscn
##
## That claim is the whole reason the row was disabled for six days, so it is
## what this asserts: write through the MENU, read it back from the DOCK, and
## the other way round.

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
	var start := DccSettings.bake_depth()
	print("[BOOT] stored bake depth on entry: ", start)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")

	print("")
	print("=== 1: the ladder exists, nine rungs, none disabled ===")
	var pop := _find_popup(app, 150)
	_ok("a menu carries the LOD levels ladder", pop != null, true)
	if pop == null:
		get_tree().quit(1); return
	var rungs := 0
	var disabled := 0
	for d in range(0, 9):
		var i := pop.get_item_index(150 + d)
		if i >= 0:
			rungs += 1
			if pop.is_item_disabled(i):
				disabled += 1
	_ok("nine rungs, 0 through 8", rungs, 9)
	_ok("none of them is disabled", disabled, 0)
	print("  info rung labels: ", pop.get_item_text(pop.get_item_index(150)),
		" ... ", pop.get_item_text(pop.get_item_index(158)))
	## SS2.5 says 0-8. The engine ceiling is higher, so a tenth rung would be
	## legal but off-spec; assert the spec's range exactly.
	_ok("no rung 9 (SS2.5 stops at 8)", pop.get_item_index(159), -1)

	print("")
	print("=== 2: the tile counts are the real pyramid sums ===")
	## depth 3 -> 1+4+16+64 = 85, the reference's own bakeAllDepth default.
	_ok("depth 3 says 85 tiles",
		pop.get_item_text(pop.get_item_index(153)).find("85") >= 0, true)
	_ok("depth 0 says 1 tile",
		pop.get_item_text(pop.get_item_index(150)).find("1") >= 0, true)

	print("")
	print("=== 3: MENU writes, DOCK reads -- one store ===")
	var ws = null
	for w in app.get("_workspaces"):
		if w.get("_bake_depth") != null:
			ws = w
	_ok("found the WORLD workspace that owns the Finalize foot", ws != null, true)
	pop.id_pressed.emit(155)          ## LOD 0-5
	await _frames(6)
	_ok("the settings store took it", DccSettings.bake_depth(), 5)
	if ws != null:
		ws.call("_refresh_finalize")
		await _frames(4)
		_ok("the DOCK now reads 5 without being told", ws.get("_bake_depth"), 5)
		var ob = ws.get("_bake_depth_choice")
		_ok("...and its OptionButton moved too", (ob.selected if ob != null else -1), 5)

	print("")
	print("=== 4: DOCK writes, MENU reads ===")
	DccSettings.set_bake_depth(2)
	pop.about_to_popup.emit()
	await _frames(4)
	_ok("the menu checks rung 2", pop.is_item_checked(pop.get_item_index(152)), true)
	_ok("...and only rung 2", pop.is_item_checked(pop.get_item_index(155)), false)

	print("")
	print("=== 5: it is what bake_all would actually be called with ===")
	## The point of the whole exercise: the number the ladder writes is the
	## number the Bake button passes to the engine.
	DccSettings.set_bake_depth(1)
	if ws != null:
		ws.call("_refresh_finalize")
		await _frames(4)
		var est: Dictionary = bridge.bake_estimate(int(ws.get("_bake_depth")))
		print("  info bake_estimate(dock depth) -> ", est)
		_ok("the dock depth is 1", ws.get("_bake_depth"), 1)
		if est.has("tiles"):
			_ok("and the engine agrees that is 5 tiles", int(est["tiles"]), 5)

	## Leave the user's setting as we found it.
	DccSettings.set_bake_depth(start)
	print("  info restored bake depth to ", DccSettings.bake_depth())

	print("")
	print("_lodlevels_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
