extends Node
## Geology-scope drift fix (`OUTSTANDING_WORK.md` §2.13 "Code and doc drift
## found by the geology scope"): `sample_bridge::CellSample`'s `stress`,
## `resistance` and `drainage` used to be filled with `0.0` when the backing
## `WorldState` substrate field was short -- "no value" presented as a real
## reading, and the MISTAKES rule this probe checks (never encode "no value"
## as a plausible value; dash the field with its reason).
##
## Two things are checked, both against the real bridge:
##
## 1. **Positive control on a real generated world**: Resistance and Drainage
##    read real numbers, not a dash, when the world's substrate is whole (the
##    normal case for every generated world today).
## 2. **The dock's own dash-with-reason path**, exercised directly against
##    `right_dock.gd::_sample_field_text` with synthetic dictionaries shaped
##    like what `sample_cell()` actually emits for an omitted field:
##    - a `resistance_reason`/`drainage_reason` key paired with the omission
##      reads "— <the engine's reason>", never a bare dash and never "0".
##    - a real `0.0` reading (the key present, value zero) reads "0.000"/a
##      flow text -- NOT a dash, since zero is a real value distinct from
##      absence. This is the positive control the MISTAKES table asks for: a
##      dash-detector that cannot tell 0.0 from absent proves nothing.
##
## Windowed (it saves a screenshot after driving the dock, matching every
## other Sample-panel probe's own convention):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _dashreason_probe.tscn

var app: Node
var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	var tag := "ok  " if cond else "FAIL"
	print("DASHR %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _row(rd: Node, label: String) -> String:
	var l: Label = rd._sample_rows.get(label)
	return "<missing row %s>" % label if l == null else String(l.text)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("DASHR ABORT: run windowed -- the screenshot needs a real rasteriser")
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
	print("DASHR world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("DASHR !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	# == 1. Positive control: a real generated world's substrate is whole ====
	var gs: Vector2i = app.bridge.grid_size()
	var land := Vector2i(-1, -1)
	var land_d: Dictionary = {}
	for y in range(3, gs.y - 3, 5):
		for x in range(3, gs.x - 3, 5):
			var d: Dictionary = app.bridge.sample_cell(x, y)
			if not d.is_empty() and d.has("resistance") and d.has("drainage"):
				land = Vector2i(x, y)
				land_d = d
				break
		if land.x >= 0:
			break
	_check("P0 a sampled cell has both resistance and drainage", land.x >= 0, str(land))
	if land.x < 0:
		print("DASHR RESULT fail=%d checks=%d" % [_fail + 1, _checks])
		get_tree().quit(1)
		return
	_check("P1 the engine's own dictionary carries no *_reason for a present field",
		not land_d.has("resistance_reason") and not land_d.has("drainage_reason"), str(land_d.keys()))

	var rd = app.right_dock_ctrl
	rd._context = "sample"
	rd._rebuild()
	await _frames(2)
	rd.on_cursor_sampled(float(land.x), float(land.y), true)
	await _frames(3)
	var want_resistance := "%.3f" % float(land_d["resistance"])
	_check("P2 Resistance row draws the engine's real reading, not a dash",
		_row(rd, "Resistance") == want_resistance and not _row(rd, "Resistance").begins_with("—"),
		"drawn=%s want=%s" % [_row(rd, "Resistance"), want_resistance])
	_check("P3 Drainage row draws a real reading, not a dash",
		not _row(rd, "Drainage").begins_with("—"), _row(rd, "Drainage"))

	await RenderingServer.frame_post_draw
	var shot_dir := "user://dashreason"
	DirAccess.make_dir_recursive_absolute(shot_dir)
	var img := get_viewport().get_texture().get_image()
	var out := "%s/real_world_resistance.png" % shot_dir
	img.save_png(out)
	print("DASHR saved ", ProjectSettings.globalize_path(out))

	# == 2. The dock's dash-with-reason path, driven directly ================
	# Shaped like sample_bridge/lib.rs actually emit: a resistance_reason key
	# is paired with the omission of "resistance" itself, never both present.
	var missing_no_reason := {"x": 4, "y": 4, "elevation_m": 120.0}
	_check("N1 a field omitted with no reason key still dashes (never '0')",
		rd._sample_field_text("resistance", missing_no_reason) == "—",
		rd._sample_field_text("resistance", missing_no_reason))

	var missing_with_reason := {"x": 4, "y": 4, "elevation_m": 120.0,
		"resistance_reason": "substrate not computed for this cell"}
	var got_reasoned: String = rd._sample_field_text("resistance", missing_with_reason)
	_check("N2 resistance dashes WITH the engine's reason, not a bare dash",
		got_reasoned == "— substrate not computed for this cell", got_reasoned)
	_check("N2b the reasoned dash is never bare '0' or empty",
		got_reasoned != "0" and got_reasoned != "0.000" and got_reasoned != "", got_reasoned)

	var drainage_missing := {"x": 4, "y": 4, "elevation_m": 120.0,
		"drainage_reason": "substrate not computed for this cell"}
	var got_drainage: String = rd._sample_field_text("drainage", drainage_missing)
	_check("N3 drainage dashes WITH the engine's reason",
		got_drainage == "— substrate not computed for this cell", got_drainage)

	var stress_missing := {"x": 4, "y": 4, "elevation_m": 120.0}
	_check("N4 stress has no dock row (not in SAMPLE_FIELDS) -- checked at the raw text helper only",
		not rd._sample_field_text("stress", {"x": 4, "y": 4}).is_empty())

	# == 3. The zero-is-not-absent control ====================================
	# A real 0.0 reading must read as a real value, never a dash -- the
	# control the MISTAKES table asks a dash-detector to survive.
	var real_zero_resistance := {"x": 4, "y": 4, "resistance": 0.0}
	var got_zero: String = rd._sample_field_text("resistance", real_zero_resistance)
	_check("Z1 a real resistance of exactly 0.0 reads as a number, not a dash",
		got_zero == "0.000" and not got_zero.begins_with("—"), got_zero)

	var real_zero_drainage := {"x": 4, "y": 4, "drainage": 0.0}
	var got_zero_dr: String = rd._sample_field_text("drainage", real_zero_drainage)
	_check("Z2 a real drainage of exactly 0.0 reads as a number, not a dash",
		got_zero_dr == "0.0" and not got_zero_dr.begins_with("—"), got_zero_dr)

	await RenderingServer.frame_post_draw
	var img2 := get_viewport().get_texture().get_image()
	var out2 := "%s/dash_with_reason.png" % shot_dir
	img2.save_png(out2)
	print("DASHR saved ", ProjectSettings.globalize_path(out2))

	print("DASHR RESULT fail=%d checks=%d" % [_fail, _checks])
