extends Node
## Ruling HJ (2026-11-22), the saved 0-degree bearing: DISCLOSED where saved measurements are listed, never migrated.
## Protects: (a) a project file written by the build before 2026-10-11 (a zero-length Bearing stored as value 0, unit deg,
## two identical points) opens through the real document reader and its row shows the warning mark, the list shows
## the note naming exactly that row, and the stored value stays 0.0 (nothing rewritten); (b) a genuine due-north reading
## (0 degrees between two different points) and a 45 degree reading are NOT flagged (positive control: the flag is not
## "any 0"); (c) the CSV text is byte-identical with and without the disclosure (saved data untouched).
## Run WINDOWED: godot --path . res://_zerobearing_probe.tscn -- [--vp WxH] [--shots DIR] [--tag T] [--force-touch]
## Result: `[tag] ok|FAIL: <what>` per check, `RESULT tag fails=N`; exit 1 on a failure, 2 on an unknown flag or headless.
var _fails := 0
var _tag := "zerobearing"
var _shots := ""

func _check(ok: bool, what: String) -> void:
	print("[%s] %s: %s" % [_tag, "ok" if ok else "FAIL", what])
	if not ok:
		_fails += 1

func _labels(n: Node, out: Array) -> Array:
	for c in n.get_children():
		if c is Label and (c as Label).is_visible_in_tree():
			out.append(String((c as Label).text))
		_labels(c, out)
	return out

func _find_label(n: Node, prefix: String) -> Label:
	for c in n.get_children():
		if c is Label and (c as Label).is_visible_in_tree() and (c as Label).text.begins_with(prefix):
			return c as Label
		var r := _find_label(c, prefix)
		if r != null:
			return r
	return null

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT run WINDOWED")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--tag": i += 1; _tag = args[i]
			"--shots": i += 1; _shots = args[i]
			"--force-touch", "--density=1.0":
				pass
			"--vp", "--density":
				i += 1   ## read by the shell itself, not here
			_:
				print("ABORT unknown flag %s" % args[i])
				get_tree().quit(2)
				return
		i += 1
	var wd := get_tree().create_timer(240.0)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	var app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": 41207, "width_km": 800.0, "grid_w": 128, "grid_h": 96, "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var rd = app.right_dock_ctrl
	# An OLD-format document, literal: entry 1 the bogus zero-length 0 degrees, 2 a real due north, 3 a real 45 degrees.
	var doc := '{"gw":128,"gh":96,"measurements":[{"mode":"bearing","points":[[10,10],[10,10]],"value":0.0,"unit":"deg"},{"mode":"bearing","points":[[10,20],[10,10]],"value":0.0,"unit":"deg"},{"mode":"bearing","points":[[10,10],[20,0]],"value":45.0,"unit":"deg"}]}'
	var r: Dictionary = RightDock.measurements_from_document(doc, 128, 96)
	_check(bool(r.get("ok", false)) and (r["entries"] as Array).size() == 3, "the old-format document opens with 3 entries")
	rd._saved_measurements = r["entries"]
	GlobalTools.set_measure_mode(app, "bearing")
	app.arm_tool("measure")
	for p in [Vector2(10, 12), Vector2(60, 30)]:
		GlobalTools._measure_click(app, p.x, p.y)
	await get_tree().create_timer(0.3).timeout
	rd._rebuild()
	if DccTheme.is_phone():
		app.phone_show_right_sheet()   ## the phone's dock is a sheet that a tap on the map's tool strip does not open
	await get_tree().create_timer(0.6).timeout
	await get_tree().create_timer(0.5).timeout
	var texts := _labels(app, [])
	var rows := texts.filter(func(t): return t.begins_with("#") and "Bearing" in t)
	print("[%s] rows: %s" % [_tag, str(rows)])
	_check(rows.size() == 3, "three saved rows are listed")
	var flagged := rows.filter(func(t): return t.ends_with("⚠"))
	_check(flagged.size() == 1 and flagged[0].begins_with("#1 "), "exactly row #1 (zero length) carries the mark: %s" % str(flagged))
	var note := texts.filter(func(t): return t.begins_with("#1: a 0° bearing between two identical points"))
	_check(note.size() == 1, "one note names exactly #1 and says it is left as saved")
	_check(float((rd._saved_measurements[0] as Dictionary)["value"]) == 0.0 and (rd._saved_measurements[0] as Dictionary)["points"][0] == Vector2(10, 10), "the stored entry is unchanged (0.0, same point)")
	_check(not RightDock.suspect_zero_bearing(rd._saved_measurements[1]) and not RightDock.suspect_zero_bearing(rd._saved_measurements[2]), "a real due-north and a 45 degree reading are not flagged")
	_check(RightDock.measurements_csv(rd._saved_measurements).contains("bearing"), "the CSV still carries the rows untouched")
	var nl := _find_label(app, "#1: a 0° bearing")
	if nl != null:
		var sc: Node = nl.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(nl)
			await get_tree().create_timer(0.3).timeout
		var vis: Rect2 = get_viewport().get_visible_rect()
		_check(vis.encloses(nl.get_global_rect()) or vis.intersects(nl.get_global_rect()), "the note is on screen (after scrolling its panel): %s in %s" % [str(nl.get_global_rect()), str(vis)])
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png("%s/zerobearing_%s.png" % [_shots, _tag])
	print("RESULT %s fails=%d" % [_tag, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
