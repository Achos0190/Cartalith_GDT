extends Node
## `PARITY_AUDIT.md` §23 wiring item 1: the status rail's "Recompute" button,
## the shell's first caller of `recompute_stale_stages`.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _stalebtn_probe.tscn
##
## Shaped on `_stalegraph_ui_shot.gd`, which proves the readout half of SG-01.
## What only the real shell can show is that the button sits beside that
## readout, appears and disappears with it on the same one-second poll, and
## that pressing it actually settles a stale stage.

var _app: Node
var _bridge


func _check(tag: String, got, want) -> bool:
	var ok := str(got) == str(want)
	print("SBTN %s %s   got=%s want=%s" % ["ok " if ok else "!!", tag, got, want])
	return ok


func _ready() -> void:
	var ok := true
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(0.8).timeout
	_bridge = _app.bridge

	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.6).timeout
	_app.open_project_dialog.hide()

	var b: Button = _app._stale_recompute
	ok = _check("the button exists", b != null, true) and ok
	if b == null:
		get_tree().quit(1)
		return
	ok = _check("it lives in the status row", b.get_parent() == _app.status_row, true) and ok
	ok = _check("it sits next to the stale slot", _app.status_row.get_children().find(b), 2) and ok
	ok = _check("hidden on a clean world", b.visible, false) and ok
	ok = _check("and the slot it follows is clear", _app.status_slot_text("stale"), "") and ok

	## A moved climate dial, through the real bridge: a stage that really is
	## recomputable, unlike the `civ` a sculpt commit leaves behind (which this
	## call deliberately does not cascade -- UNIFIED_TOOL_PLAN.md milestone C).
	_bridge.param_set("climate.rain_k", 1.6)
	## The 1 s poll, not a direct refresh: the clock is the mechanism the
	## button's own visibility hangs off.
	await get_tree().create_timer(1.6).timeout
	var slot: String = _app.status_slot_text("stale")
	print("SBTN stale slot = '%s'" % slot)
	ok = _check("the dial went stale", slot.find("climate") >= 0, true) and ok
	ok = _check("the button appeared with it", b.visible, true) and ok
	ok = _check("and is pressable", b.disabled, false) and ok

	b.pressed.emit()
	await get_tree().create_timer(6.0).timeout
	print("SBTN hint  = '%s'" % _app.status_slot_text("hint"))
	print("SBTN stale = '%s'" % _app.status_slot_text("stale"))
	ok = _check("it reported what it ran",
		_app.status_slot_text("hint").begins_with("Recomputed "), true) and ok
	ok = _check("the stage it settled is gone",
		_app.status_slot_text("stale").find("climate") >= 0, false) and ok
	ok = _check("the label came back", b.text, "Recompute") and ok

	## The one thing an assertion cannot check: that a 26 px action chip in a
	## 26 px status bar does not push the bar out of shape.
	_bridge.param_set("climate.rain_k", 1.9)
	_app.refresh_staleness()
	await RenderingServer.frame_post_draw
	var shot := OS.get_user_data_dir().path_join("_stalebtn_probe.png")
	get_viewport().get_texture().get_image().save_png(shot)
	print("SBTN shot -> %s" % shot)

	print("SBTN RESULT %s" % ("PASS" if ok else "FAIL"))
	get_tree().quit(0 if ok else 1)
