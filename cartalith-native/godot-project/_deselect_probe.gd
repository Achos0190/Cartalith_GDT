extends Node
## `Edit ▸ Deselect` (⌘D, `DCC_SHELL_SPEC.md` §2.2), driven through the real
## menu row rather than by calling `clear_selection()` directly — a row wired to
## the wrong id, or a `_todo` that was never converted, fails here.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _deselect_probe.tscn
##
## Windowed, not `--headless`: the shell has to be composited for the
## Cartography label and icon paths to have anything to select.
##
## The row was disabled with the reason *"no shared way to clear them: Escape
## disarms the active tool without touching what is selected"*. Both clauses
## were true. The second still is, and that is asserted here too — Escape and
## Deselect must NOT have become the same key.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _find_menu_item(app: Node, id: int) -> PopupMenu:
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
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	print("[BOOT] shell up")

	print("")
	print("=== 0: the row is LIVE, not a disabled _todo ===")
	var edit_pop := _find_menu_item(app, 82)   ## ID_DESELECT
	_ok("a menu carries the Deselect row", edit_pop != null, true)
	if edit_pop == null:
		get_tree().quit(1); return
	var di := edit_pop.get_item_index(82)
	_ok("it is enabled", edit_pop.is_item_disabled(di), false)
	_ok("its label is Deselect", edit_pop.get_item_text(di), "Deselect")

	print("")
	print("=== 1: with nothing selected it is an honest no-op ===")
	_ok("clear_selection() reports it cleared nothing", app.clear_selection(), false)

	print("")
	print("=== 2: generate a world so there is something to select ===")
	bridge.generate({"seed": 909, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_ok("a world landed", bridge.has_world, true)
	var settlements: Array = bridge.settlements()
	print("  info settlements: ", settlements.size())
	_ok("there are settlements to select", settlements.size() > 0, true)

	print("")
	print("=== 3: a placed ICON clears — the half the engine could not do ===")
	## `icon_deselect()` is the binding added for this row; before it, an icon
	## selection could be STARTED (place, hit_test) and never ended.
	var armed: bool = bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0)
	print("  info icon_arm -> ", armed)
	if armed:
		var idx: int = bridge.icon_place(40.0, 30.0)
		print("  info icon_place -> ", idx)
		_ok("placing an icon selects it", bridge.icon_get_selected() >= 0, true)
		_ok("Deselect reports it cleared something", app.clear_selection(), true)
		_ok("...and the icon is no longer selected", bridge.icon_get_selected(), -1)
		_ok("the icon itself was NOT deleted", bridge.icon_list().size() > 0, true)
	else:
		## Not a failure: `icon_arm` refuses without an asset pack
		## (`lib.rs`'s own `has_asset_pack()` guard) and none ships, which
		## `UNWIRED_FUNCTIONS.md` already records as CA-12. The LABEL leg
		## below covers the same `clear_selection()` path, so the row is
		## still proven; what is not proven here is the icon binding
		## specifically, and saying so beats a silent skip.
		print("  info icon_arm refused: no asset pack ships (CA-12). Icon leg not covered.")

	print("")
	print("=== 4: a LABEL clears ===")
	## **`label_create`, not `label_place`.** The first cut of this probe
	## guessed the name, got `-1` back from a `has_method()` that was simply
	## false, and printed "not on this build" — a skipped leg reading as a
	## pass, which is the silently-empty trap this repository keeps finding.
	## The binding was there the whole time under the other name.
	##
	## Labels carry the whole remaining weight here, since the icon leg cannot
	## run without an asset pack, so the three legs below use them.
	var lidx: int = bridge.label_create(60.0, 40.0, "Deselect probe")
	print("  info label_create -> ", lidx)
	_ok("a label was created", lidx >= 0, true)
	if lidx < 0:
		print("_deselect_probe: label_create refused — nothing left to test")
		get_tree().quit(1); return
	bridge.label_select(lidx)
	_ok("the label is selected", bridge.label_get_selected() >= 0, true)
	_ok("Deselect reports it cleared something", app.clear_selection(), true)
	_ok("...and it reports nothing selected", bridge.label_get_selected(), -1)
	_ok("the label itself was NOT deleted", bridge.label_list().size() > 0, true)

	print("")
	print("=== 5: the MENU ROW reaches it, not just the method ===")
	bridge.label_select(lidx)
	_ok("selected again", bridge.label_get_selected() >= 0, true)
	edit_pop.id_pressed.emit(82)
	await _frames(4)
	_ok("pressing the row cleared it", bridge.label_get_selected(), -1)

	print("")
	print("=== 6: Escape is still a DIFFERENT act ===")
	## The row's original reason said Escape "disarms the active tool without
	## touching what is selected". That is deliberate and must survive: Escape
	## means put the tool down, Deselect means forget what it pointed at. If a
	## later change makes Escape clear the selection too, this fails and the
	## menu row's tooltip becomes a lie.
	bridge.label_select(lidx)
	_ok("selected once more", bridge.label_get_selected() >= 0, true)
	var ev := InputEventKey.new()
	ev.keycode = KEY_ESCAPE
	ev.pressed = true
	app._unhandled_key_input(ev)
	await _frames(4)
	_ok("Escape left the selection alone", bridge.label_get_selected() >= 0, true)

	print("")
	print("_deselect_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
