extends Node
## PC-1 follow-up probe: CIVIL ▸ Factions ▸ Open factions… opens the Factions window on the current
## faction (`PC_SHELL_PROPOSAL.md` §3.6 "Opening on the right subject", finding W6; the row is
## `civilization_workspace.gd::_open_roster_on_current()`), inside the real shell.
##
##   desktop: godot --path . --resolution 1600x1000 _pc1roster_probe.tscn
##   tablet:  godot --path . --resolution 1280x800  _pc1roster_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, with a wiped user dir (the runner sets `APPDATA` to an empty
## folder). Reads `--force-touch` only; anything else exits 2.
##
## What it protects. A small world is generated (seed 4242, 256 x 192, as `_pc1select_probe.gd`). Two
## named settlements held by DIFFERENT factions are found from the engine's own settlement list (their
## faction ids are read, never assumed). The row's button is found by its text in the CIVIL dock and
## pressed through its own `pressed` signal (a desktop regression probe: it does not prove a finger
## reaches the row, `MISTAKES.md` "Verify anything phone-shaped"). Then:
##   1. a settlement of faction A is selected; pressing the row shows the roster on A;
##   2. a settlement of faction B is selected; the roster closed; pressing it shows B (so the answer
##      follows the selection, not the last window state);
##   3. the faction ITSELF selected (`select("faction", A)`) opens on A;
##   4. nothing selected: the roster opens on the faction last viewed (A, from step 3), the behaviour
##      before this change, never on a guessed id.
## The expected ids are two different values read from the engine; the control (`A != B`) is asserted so a
## roster stuck on one faction cannot pass step 2 vacuously. Exit 0 pass, 1 fail, 2 abort.

var _fails := 0
var _checks := 0
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _row() -> Button:
	for b in _app.left_dock_body.find_children("*", "Button", true, false):
		if (b as Button).text == "Open factions…":
			return b
	return null


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a != "--force-touch":
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	## Two settlements whose factions differ.
	var all: Array = _bridge.settlements()
	var ia := -1
	var ib := -1
	for i in all.size():
		var s: Dictionary = all[i]
		if not s.has("faction") or int(s["faction"]) <= 0:
			continue
		if ia < 0:
			ia = i
		elif int(s["faction"]) != int((all[ia] as Dictionary)["faction"]):
			ib = i
			break
	if not bool(_bridge.has_world) or ia < 0 or ib < 0:
		print("[FATAL] no world with two settlements of different factions")
		get_tree().quit(2)
		return
	var fa := int((all[ia] as Dictionary)["faction"])
	var fb := int((all[ib] as Dictionary)["faction"])
	_ok(fa != fb and fa > 0 and fb > 0, "control: two real, different factions (%d, %d)" % [fa, fb])
	_app.select_domain("civilization")
	await _settle(6)
	var roster = _app.faction_roster_window
	var btn := _row()
	_ok(btn != null, "the CIVIL dock carries the Open factions… row")
	if btn == null:
		get_tree().quit(1)
		return
	## 1. a settlement of A.
	_app.select("settlement", ia)
	await _settle(4)
	btn.pressed.emit()
	await _settle(6)
	_ok(roster.visible and int(roster._selected) == fa, "1. settlement of A selected: the roster shows A")
	roster.hide()
	await _settle(3)
	## 2. a settlement of B.
	_app.select("settlement", ib)
	await _settle(4)
	btn.pressed.emit()
	await _settle(6)
	_ok(roster.visible and int(roster._selected) == fb, "2. settlement of B selected: the roster shows B")
	roster.hide()
	await _settle(3)
	## 3. the faction itself.
	_app.select("faction", fa)
	await _settle(4)
	btn.pressed.emit()
	await _settle(6)
	_ok(roster.visible and int(roster._selected) == fa, "3. faction A selected: the roster shows A")
	roster.hide()
	await _settle(3)
	## 4. nothing selected: the last faction viewed (A), as before this change.
	_app.select("", -1)
	await _settle(4)
	_ok(int(_app.current_faction()) == -1, "4. control: nothing selected, current_faction() is -1")
	btn.pressed.emit()
	await _settle(6)
	_ok(roster.visible and int(roster._selected) == fa, "4. nothing selected: the roster reopens on the last faction viewed (A)")
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
