extends Node
## GF-1 (`GEOLOGY_FIRST_SCOPE.md` §9 Q3): the Sample dock shows the rock
## model's rows on a generated world.
##
## Boots the real app, generates a world at the app defaults (the geology model
## is on there), finds one two-layer cell and one single-layer cell through
## `sample_cell()`, drives the dock through its own `on_cursor_sampled()`, and
## asserts each rock row's drawn text against the dictionary the engine
## returned -- so the check is "the dock draws what the engine says", with the
## engine's own keys as the oracle, not a hand-typed rock name.
##
## Also asserts the no-value rule both ways: a single-layer cell has NO
## `rock_beneath` key and its row reads a dash with the engine's reason; no rock
## row ever reads a bare "0", "none" or an empty string.
##
## Windowed (it saves a screenshot, which needs a real rasteriser):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf1sample_probe.tscn
##
## Screenshot: user://gf1sample/sample_two_layer.png (the path is printed).

var app: Node
var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	var tag := "ok  " if cond else "FAIL"
	print("GF1S %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _row(rd: Node, label: String) -> String:
	var l: Label = rd._sample_rows.get(label)
	return "<missing row %s>" % label if l == null else String(l.text)

const ROCK_ROWS := ["Rock (surface)", "Beneath", "Strength", "Soluble · permeable", "Regolith"]

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("GF1S ABORT: run windowed -- the screenshot needs a real rasteriser")
		get_tree().quit(2)
		return
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3600:
		await get_tree().process_frame
		waited += 1
	print("GF1S world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("GF1S !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var gs: Vector2i = app.bridge.grid_size()
	# Scan a coarse lattice for one two-layer and one single-layer LAND cell.
	var two := Vector2i(-1, -1)
	var one := Vector2i(-1, -1)
	var two_d: Dictionary = {}
	var one_d: Dictionary = {}
	var rock_seen := 0
	var no_col := 0
	for y in range(3, gs.y - 3, 7):
		for x in range(3, gs.x - 3, 7):
			var d: Dictionary = app.bridge.sample_cell(x, y)
			if d.is_empty() or float(d.get("elevation_m", -1.0)) <= 0.0:
				continue
			if d.has("rock"):
				rock_seen += 1
			elif d.has("rock_reason"):
				no_col += 1
			if two.x < 0 and d.has("rock_beneath"):
				two = Vector2i(x, y)
				two_d = d
			if one.x < 0 and d.has("rock") and d.get("rock_beneath_reason", "") == "single-layer column":
				one = Vector2i(x, y)
				one_d = d
	print("GF1S scanned: land cells with a rock %d, without a column %d; two-layer %s, single-layer %s" % [rock_seen, no_col, two, one])
	_check("P0 the generated world carries a rock column", rock_seen > 0 and no_col == 0,
		"rock=%d no_column=%d" % [rock_seen, no_col])
	_check("P1 a two-layer land cell exists", two.x >= 0)
	_check("P2 a single-layer land cell exists", one.x >= 0)
	if two.x < 0 or one.x < 0:
		print("GF1S RESULT fail=%d checks=%d" % [_fail + 1, _checks])
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	rd._context = "sample"
	rd._rebuild()
	await _frames(2)
	for label in ROCK_ROWS:
		_check("R0 row '%s' exists in the dock" % label, rd._sample_rows.has(label))

	# == The two-layer cell ==================================================
	rd.on_cursor_sampled(float(two.x), float(two.y), true)
	await _frames(3)
	var want_rock := String(two_d["rock"])
	if two_d.has("volcanic_setting"):
		want_rock += " · %s volcanic" % String(two_d["volcanic_setting"])
	_check("T1 Rock row draws the engine's exposed rock", _row(rd, "Rock (surface)") == want_rock,
		"drawn=%s want=%s" % [_row(rd, "Rock (surface)"), want_rock])
	var want_beneath := "%.0f m down · %s" % [float(two_d["rock_contact_depth_m"]), String(two_d["rock_beneath"])]
	_check("T2 Beneath draws the substrate and the contact depth", _row(rd, "Beneath") == want_beneath,
		"drawn=%s want=%s" % [_row(rd, "Beneath"), want_beneath])
	_check("T3 the contact depth is a positive depth", float(two_d["rock_contact_depth_m"]) > 0.0,
		"%s m" % two_d["rock_contact_depth_m"])
	_check("T4 Strength draws the engine's Selby class", _row(rd, "Strength") == String(two_d["rock_strength"]),
		_row(rd, "Strength"))
	var want_sol := "%s · %s" % ["soluble" if bool(two_d["rock_soluble"]) else "insoluble", String(two_d["rock_permeability"])]
	_check("T5 Soluble · permeable draws both flags", _row(rd, "Soluble · permeable") == want_sol,
		"drawn=%s want=%s" % [_row(rd, "Soluble · permeable"), want_sol])
	_check("T6 Regolith is a real 0 m reading, labelled bare rock",
		two_d.has("regolith_m") and float(two_d["regolith_m"]) == 0.0 and _row(rd, "Regolith") == "0 m · bare rock",
		_row(rd, "Regolith"))
	var shot_dir := "user://gf1sample"
	DirAccess.make_dir_recursive_absolute(shot_dir)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var out := "%s/sample_two_layer.png" % shot_dir
	img.save_png(out)
	print("GF1S saved ", ProjectSettings.globalize_path(out))

	# == The single-layer cell ===============================================
	rd.on_cursor_sampled(float(one.x), float(one.y), true)
	await _frames(3)
	_check("S1 the engine omits rock_beneath on a single-layer cell",
		not one_d.has("rock_beneath") and not one_d.has("rock_contact_depth_m"), str(one_d.keys()))
	_check("S2 Beneath dashes with the engine's reason", _row(rd, "Beneath") == "— single-layer column",
		_row(rd, "Beneath"))
	_check("S3 Rock row still draws the rock", _row(rd, "Rock (surface)").begins_with(String(one_d["rock"])),
		_row(rd, "Rock (surface)"))
	await RenderingServer.frame_post_draw
	var img2 := get_viewport().get_texture().get_image()
	var out2 := "%s/sample_single_layer.png" % shot_dir
	img2.save_png(out2)
	print("GF1S saved ", ProjectSettings.globalize_path(out2))

	# == The no-value rule, over both cells ==================================
	for label in ROCK_ROWS:
		for cell in [two, one]:
			rd.on_cursor_sampled(float(cell.x), float(cell.y), true)
			await _frames(1)
			var t := _row(rd, label)
			_check("N '%s' at %s is never a bare 0/none/empty" % [label, cell],
				t != "" and t != "0" and t.to_lower() != "none" and t != "—", t)

	print("GF1S RESULT fail=%d checks=%d" % [_fail, _checks])
	get_tree().quit(1 if _fail > 0 else 0)
