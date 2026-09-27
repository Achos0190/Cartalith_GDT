extends Node
## Isolation build for `_undofixes_probe.gd`'s leg A alone -- headless,
## desktop shell, no phone form needed. See that file's own header for the
## full context; this exists only to separate "does the staleness fix work"
## from "does the phone chip repaint", after the combined probe hung.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _undofixlegA_probe.tscn

const SEED := 552017

var app: Node
var _fails := 0
var _checks := 0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("%s  %s%s" % ["PASS" if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	wd.start()

	print("STEP boot")
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	print("STEP app ready, bridge=%s" % [app.bridge])

	var bridge = app.bridge
	print("STEP generating")
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	print("STEP generated, has_world=%s" % [bridge.has_world])
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	print("=== A: undo_revert_to marks the height stage stale, like undo_last ===")

	var r1: Dictionary = bridge.carve_fjords()
	await _frames(3)
	var target_seq: int = int((bridge.undo_ledger()[-1] as Dictionary).get("seq", 0))
	var r2: Dictionary = bridge.carve_fjords()
	await _frames(3)
	_ok("A setup: both carves ran", bool(r1.get("ok", false)) and bool(r2.get("ok", false)),
		"r1=%s r2=%s" % [str(r1), str(r2)])
	_ok("A setup: the target row is real", target_seq > 0, "target_seq=%d" % target_seq)

	bridge.world_gen.recompute_stale_stages()
	await _frames(2)
	var before: Dictionary = bridge.stale_stages()
	# `civ` legitimately stays stale here -- `recompute_stale_stages()`'s own
	# doc comment: "'civ' in still_stale is the normal steady state after a
	# terrain edit ... UNIFIED_TOOL_PLAN.md milestone C measured why it is
	# not cascaded per stroke". The baseline this test needs clean is
	# hydrology/climate, the two stages a settle actually recomputes.
	_ok("A setup: hydrology and climate are clean before the revert",
		not before.has("hydrology") and not before.has("climate"),
		"stale_stages()=%s" % str(before))

	var done: int = bridge.undo_revert_to(target_seq)
	_ok("A: the revert popped both carves", done == 2, "done=%d" % done)

	var after: Dictionary = bridge.stale_stages()
	print("A stale_stages() after undo_revert_to: %s" % str(after))
	_ok("A: something is stale after the revert (the bug: this used to be empty)",
		not after.is_empty(), "after=%s" % str(after))
	var hydro: Dictionary = after.get("hydrology", {})
	_ok("A: hydrology (Height's own consumer) is the stale stage",
		not hydro.is_empty(), "after=%s" % str(after))
	_ok("A: its reason names the revert, not the carves it undid",
		String(hydro.get("reason", "")) == "undo_revert_to",
		"reason=%s" % String(hydro.get("reason", "")))

	print("### UNDOFIXLEGA %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
