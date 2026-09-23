extends Node
## Windowed proof that hand-anchored popovers stay on screen
## (`OUTSTANDING_WORK.md` §2.10, 2026-09-24; `DccWidgets.popup_anchored`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1152x648 _popclamp_probe.tscn
##
##   1. the real Layers popover opens wholly inside the visible area;
##   2. control: a medium popover opened the OLD way (a bare rect under a
##      trigger near the bottom) ends up over its own trigger -- Godot slides
##      an embedded popup back on screen -- so the check can see the defect;
##   3. the same popover through `popup_anchored` flips above, inside the
##      window and clear of the trigger;
##   4. a short popover under a mid-screen anchor still opens right below it
##      (the helper does not move a popover that already fits).

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("POPCLAMP %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _inside(p: Window) -> bool:
	var b := Rect2i(get_tree().root.get_visible_rect())
	return b.encloses(Rect2i(p.position, p.size))

func _panel(rows: int) -> PopupPanel:
	var p := PopupPanel.new()
	var col := VBoxContainer.new()
	for i in rows:
		var l := Label.new()
		l.text = "row %d" % i
		col.add_child(l)
	p.add_child(col)
	add_child(p)
	return p

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("POPCLAMP REFUSED: headless")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(4)
	print("POPCLAMP visible area %s" % [get_tree().root.get_visible_rect()])

	# 1. the real Layers popover
	app.layers_popover.open()
	await _frames(6)
	var lp: Window = app.layers_popover
	_check("the Layers popover opens inside the visible area", _inside(lp), "%s %s" % [lp.position, lp.size])
	lp.hide()
	await _frames(2)

	# 2. control: the OLD placement of a medium popover under a trigger near
	#    the bottom. Godot keeps an embedded popup on screen by sliding it up,
	#    so the failure is that it ends up COVERING its own trigger.
	var anchor := Rect2(1000, 560, 60, 22)
	var old := _panel(20)
	old.popup(Rect2i(Vector2i(anchor.position) + Vector2i(0, int(anchor.size.y) + 4), Vector2i(342, 0)))
	await _frames(4)
	var old_r := Rect2(old.position, old.size)
	_check("control: the OLD placement covers its trigger or leaves the window",
		old_r.intersects(anchor) or not _inside(old), "%s" % [old_r])
	old.hide()

	# 3. the helper flips it above, clear of the trigger, inside the window
	var med := _panel(20)
	DccWidgets.popup_anchored(med, anchor, 342)
	await _frames(4)
	var med_r := Rect2(med.position, med.size)
	_check("through popup_anchored it sits inside, clear of its trigger",
		_inside(med) and not med_r.intersects(anchor), "%s" % [med_r])
	med.hide()

	# 4. a popover that fits is left right below its anchor
	var mid := Rect2(300, 200, 60, 22)
	var small := _panel(4)
	DccWidgets.popup_anchored(small, mid, 342)
	await _frames(4)
	_check("a fitting popover still opens right below its anchor",
		small.position == Vector2i(300, 226) and _inside(small), "%s %s" % [small.position, small.size])
	print("POPCLAMP %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
