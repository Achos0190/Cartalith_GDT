extends Node
## Windowed proof that five tool windows lost their leftover `max_size` cap
## without starting to grow to their content (`OUTSTANDING_WORK.md` §2.10,
## 2026-09-24; the vault window's `3736fe7` fix, repeated).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _wincap_probe.tscn
##
## For each window, on a generated world at desktop density:
##   1. no `max_size` and `wrap_controls` off (the cause the cap stood in for);
##   2. it opens at its own default size -- content did not grow it;
##   3. it opens inside the main window, so its footer is on screen;
##   4. it accepts a size larger than the OLD cap -- the thing the cap blocked.

const OLD_CAP := {
	"place_editor_window": Vector2i(560, 760),
	"faction_roster_window": Vector2i(1000, 700),
	"culture_profiles_window": Vector2i(1200, 900),
	"settlement_types_window": Vector2i(1180, 780),
	"generation_rules_window": Vector2i(1180, 900),
}

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("WINCAP %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("WINCAP REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("WINCAP WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var main_rect := Rect2i(Vector2i.ZERO, get_window().size)
	print("WINCAP main window %s" % [get_window().size])

	for key in OLD_CAP.keys():
		var w: Window = app.get(key)
		var default_size := w.size
		if key == "place_editor_window":
			w.open_for(0)
		else:
			w.open()
		await _frames(12)
		_check("%s: no max_size, wrap_controls off" % key,
			(w.max_size == Vector2i.ZERO or (w.max_size.x >= 16384 and w.max_size.y >= 16384)) and not w.wrap_controls, "max %s wrap %s" % [w.max_size, w.wrap_controls])
		_check("%s: opens at its default size, not its content's" % key,
			w.size == default_size, "%s vs default %s" % [w.size, default_size])
		var r := Rect2i(w.position, w.size)
		_check("%s: opens inside the main window" % key, main_rect.encloses(r), "%s in %s" % [r, main_rect])
		var want: Vector2i = OLD_CAP[key] + Vector2i(80, 60)
		w.size = want
		await _frames(4)
		_check("%s: accepts %s, past the old cap %s" % [key, want, OLD_CAP[key]], w.size == want, str(w.size))
		w.hide()
		await _frames(2)

	print("WINCAP %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
