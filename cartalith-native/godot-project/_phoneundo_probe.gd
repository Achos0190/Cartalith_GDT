extends Node
## Protects: the phone undo chip answering the LOOK history as well as the height stack (OUTSTANDING_WORK
## "The phone undo chip answers only bridge.can_undo()", fixed 2026-10-12). Before the fix a look-only edit
## (ramp, biomes, style, adjustments) left the chip hidden and a tap did nothing: the chip read
## `bridge.can_undo()` alone, while the desktop squares and `Edit > Undo` read `DccApp.can_undo_any()`.
## Legs: (1) fresh boot: chip hidden, and `bridge.can_undo()` false (so only the look history can explain a
## later show: the positive control); (2) a look entry committed => chip VISIBLE with `bridge.can_undo()`
## still false; (3) a tap (button_down then button_up, the chip's own handlers) undoes the LOOK step and
## the chip hides again; (4) a redo tail alone does not show it. Mutation: revert `_refresh_phone_undo_chip`
## to `bridge.can_undo()` and leg 2 goes red.
## Run WINDOWED from a scratch copy: godot --path <copy> res://_phoneundo_probe.tscn -- --force-touch
## Exit 0 = held; 1 = a check failed; 2 = could not run.

var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fail += 1
	print("PHUN  %s  %s" % ["ok  " if cond else "FAIL", what])

func _ready() -> void:
	if not ("--force-touch" in OS.get_cmdline_user_args()) or DisplayServer.get_name() == "headless":
		print("[FATAL] run windowed with `-- --force-touch`")
		get_tree().quit(2)
		return
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	if not bool(app.is_phone()):
		print("[FATAL] not the phone composition")
		get_tree().quit(2)
		return
	var chip: Button = app.get("_phone_undo_chip")
	var lh = app.look_history
	_ok(chip != null and not chip.visible and not bool(app.bridge.can_undo()), "fresh boot: chip hidden and the height stack is empty")
	lh.commit("Test look edit", {"zz": 2}, app.bridge, {"zz": 1})
	await _frames(3)
	_ok(not bool(app.bridge.can_undo()) and lh.can_undo(), "a look entry exists and the height stack is STILL empty")
	_ok(chip.visible, "the chip is visible for a look-only history")
	chip.button_down.emit()
	await _frames(2)
	chip.button_up.emit()
	await _frames(3)
	_ok(not lh.can_undo() and lh.can_redo(), "a tap stepped the LOOK history back (cursor %d of %d)" % [lh.cursor, lh.entries.size()])
	_ok(not chip.visible, "the chip hides again once only a redo tail is left")
	print("PHUN  %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
