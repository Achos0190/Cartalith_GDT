extends Node
## Windowed proof that the place editor's six tabs no longer run together at
## the window's 400 px default (`OUTSTANDING_WORK.md` §2.10, 2026-09-24).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _petabs_probe.tscn
##
## Measures the drawn button rects, not the code: at 400 px no two tabs overlap,
## same-line neighbours have a real gap, and no tab spills past the window's
## width; at 1000 px (control) all six sit on one line.

const LABELS := ["Overview", "Economy & notables", "Timeline", "Political history",
	"Vault notes", "Layout"]
var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("PETABS %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _find_buttons(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button and LABELS.has((c as Button).text):
			out.append(c)
		_find_buttons(c, out)

func _measure(pe: Window, label: String) -> void:
	var btns: Array = []
	_find_buttons(pe, btns)
	_check("%s: all six tabs found" % label, btns.size() == 6, str(btns.size()))
	var rects: Array = []
	for b in btns:
		rects.append(Rect2((b as Control).get_global_rect()))
	var overlap := false
	var min_gap := 1e9
	var lines := {}
	for i in rects.size():
		var a: Rect2 = rects[i]
		lines[int(a.position.y)] = true
		for j in range(i + 1, rects.size()):
			var b: Rect2 = rects[j]
			if a.intersects(b, false):
				overlap = true
			if absf(a.position.y - b.position.y) < 1.0:
				min_gap = minf(min_gap, maxf(b.position.x - a.end.x, a.position.x - b.end.x))
	var spill := false
	for r in rects:
		if (r as Rect2).end.x > float(pe.size.x) + 0.5:
			spill = true
	_check("%s: no two tabs overlap" % label, not overlap)
	_check("%s: same-line neighbours have a gap" % label, min_gap >= 1.0, "min gap %.1f px" % min_gap)
	_check("%s: no tab spills past the window" % label, not spill)
	print("PETABS %s: %d line(s)" % [label, lines.size()])
	return

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("PETABS REFUSED: headless")
		get_tree().quit(2)
		return
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
	var pe: Window = app.place_editor_window
	pe.open_for(0)
	await _frames(12)
	_check("editor opens at 400 px wide", pe.size.x == 400, str(pe.size))
	_measure(pe, "400 px")
	pe.size = Vector2i(1000, pe.size.y)
	await _frames(8)
	_measure(pe, "1000 px")
	var btns: Array = []
	_find_buttons(pe, btns)
	var ys := {}
	for b in btns:
		ys[int((b as Control).get_global_rect().position.y)] = true
	_check("control: at 1000 px all six sit on one line", ys.size() == 1, "%d lines" % ys.size())
	print("PETABS %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
