extends Node
## Windowed measurement + proof for the Asset Library window at Godot's
## default 1152x648 (`OUTSTANDING_WORK.md` §2.10). Counts visible controls whose
## drawn rect leaves the window, and reports the widest minimum widths.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1152x648 _alfit_probe.tscn
##
## Before the 2026-09-24 fix: window min 1403 px, header row 1371 (a fixed
## 340 px search well), body 1264 (a grid pane held at 668 by two text rows),
## 38 controls past the right edge at 1152 -- FAIL. After: PASS, 0 outside,
## min 1152; at 1920 the canvas layout is back (search 340, note shown).

var _fails := 0
var _outside := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("ALFIT %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _walk(n: Node, win_rect: Rect2, depth: int) -> void:
	for c in n.get_children():
		if c is Control and (c as Control).is_visible_in_tree():
			var r := (c as Control).get_global_rect()
			if r.size.x > 0.0 and r.end.x > win_rect.end.x + 0.5:
				_outside += 1
				if _outside <= 6:
					print("ALFIT outside: %s %s end.x %.0f > %.0f" % [c.get_class(), c.name, r.end.x, win_rect.end.x])
			if depth <= 3 and (c as Control).get_combined_minimum_size().x > 300.0:
				print("ALFIT min %d  depth %d  %s %s" % [int((c as Control).get_combined_minimum_size().x), depth, c.get_class(), c.name])
		_walk(c, win_rect, depth + 1)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ALFIT REFUSED: headless")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(4)
	app.open_asset_library("biomes")
	await _frames(20)
	var win: Window = app.asset_library_window
	print("ALFIT main %s  asset window size %s  min %s" % [get_window().size, win.size, win.get_contents_minimum_size()])
	print("ALFIT fit: search %.0f  sub %s  grid cols %d  grid children %d" % [win._hdr_search.custom_minimum_size.x if win._hdr_search else -1.0, str(win._hdr_sub.visible) if win._hdr_sub else "-", win._grid.columns, win._grid.get_child_count()])
	var wr := Rect2(Vector2.ZERO, Vector2(win.size))
	_walk(win, wr, 0)
	_check("no visible control extends past the window's right edge", _outside == 0, "%d outside" % _outside)
	_check("the window fits the 1152-wide main window", win.size.x <= get_window().size.x, str(win.size))
	print("ALFIT %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
