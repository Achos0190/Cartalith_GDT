extends Node
## Windowed proof of Ruling AQ's LIVE sea level (`OUTSTANDING_WORK.md` §2.5,
## "Sea level is not a live control"), with Ruling 17's sculpt-draft re-stamp.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _sealive_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _sealive_probe.tscn -- --timing 2048
##
## Windowed only: it refuses `--headless` (`ImageTexture` is not rasterised
## there, so the texture comparison would pass vacuously). Real shell, real
## engine: the level is moved by the World data **Sea level slider itself**
## (its `value` is written, which fires the row's own `value_changed`, then
## its own `drag_ended` -- the release the row is wired to), never by calling
## the engine binding directly. What is checked, each against a control:
##   0. NEGATIVE control: before any move, two reads of the classification
##      agree, nothing is stale, and an unchanged release reports
##      `changed: false` and marks nothing.
##   1. raise: land falls, ocean rises (direction vs the base level), the
##      coastline cell count moves, the civ copy's land count equals the drawn
##      one (the civ layer re-derived), the map texture's bytes change
##      (POSITIVE control for 3's identity), the draft stamp's saved
##      `sea_level` equals the new level (Ruling 17), and climate + civ read
##      stale with reason `sea_level`.
##   2. lower: land rises above the base.
##   3. restore: counts, the military summary (navigable / sea shares) and the
##      map texture bytes return EXACTLY to the base (round-trip identity);
##      the military summary moved on at least one of raise/lower.
##   4. save -> reopen through the app's own paths keeps the live level, its
##      counts and the draft stamp's level; a project saved before any move
##      (5) reopens at the base level with the base counts -- the old-save
##      path unchanged.
## `--timing N` instead generates an N x N world and prints the median
## (min..max) of five live moves and five repaints; no checks.
## Ends with `SEALIVE RESULT GREEN` or `SEALIVE RESULT FAIL (n)`; every failed
## check prints `PROBE-FAIL`. Exits 0 either way (grep the lines).

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("SEALIVE %s  %s%s" % ["ok  " if cond else "PROBE-FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

## The World data Sea level `HSlider`: the one whose row carries a Label
## reading exactly "Sea level". `null` when the shell built none.
func _find_sea_slider(n: Node) -> HSlider:
	if n is HSlider:
		var row := n.get_parent()
		if row != null and _has_label(row, "Sea level"):
			return n
	for c in n.get_children(true):
		var r := _find_sea_slider(c)
		if r != null:
			return r
	return null

func _has_label(n: Node, text: String) -> bool:
	for c in n.get_children(true):
		if c is Label and (c as Label).text == text:
			return true
		if c is Container and not (c is HSlider) and _has_label(c, text):
			return true
	return false

## Moves the level the way a person does: the slider's value (its own
## `value_changed` -> the row's input handler), then its own release.
func _drag(s: HSlider, v: float) -> void:
	s.value = v
	await _frames(2)
	s.drag_ended.emit(true)
	await _frames(6)

func _tex_bytes(app: Node) -> PackedByteArray:
	var t: Texture2D = app.viewport.map_view.texture
	if t == null:
		return PackedByteArray()
	var img := t.get_image()
	return PackedByteArray() if img == null else img.get_data()

## `drafts/sculpt.json`'s first stamp's `sea_level`, or NAN when there is none.
func _draft_level(wg) -> float:
	var txt := String(wg.sculpt_document_json())
	var doc = JSON.parse_string(txt) if txt != "" else null
	if typeof(doc) != TYPE_DICTIONARY or (doc.get("stamps", []) as Array).is_empty():
		return NAN
	return float(doc["stamps"][0].get("sea_level", NAN))

func _save(app: Node, path: String) -> bool:
	var done := [false]
	app._write_project(path, func(): done[0] = true)
	for _i in 300:
		if done[0]:
			break
		await _frames(2)
	return FileAccess.file_exists(path)

func _member(path: String, suffix: String) -> String:
	var zip := ZIPReader.new()
	var out := ""
	if zip.open(path) == OK:
		for f in zip.get_files():
			if String(f).ends_with(suffix):
				out = zip.read_file(f).get_string_from_utf8()
		zip.close()
	return out

func _median(a: Array) -> String:
	var s := a.duplicate()
	s.sort()
	return "%.1f ms (%.1f..%.1f)" % [s[s.size() / 2], s[0], s[s.size() - 1]]

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("SEALIVE REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 260.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("SEALIVE WATCHDOG"); get_tree().quit(3))
	wd.start()

	var args := OS.get_cmdline_user_args()
	var timing := 0
	if args.size() >= 2 and args[0] == "--timing":
		timing = int(args[1])

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	var n := timing if timing > 0 else 256
	var req := {"seed": 7311, "width_km": 900.0, "grid_w": n, "grid_h": n if timing > 0 else 192,
		"archetype": "", "villages": true, "sea_level": 0.42}
	bridge.generate(req)
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var wg = bridge.world_gen
	print("SEALIVE dll binding present: ", wg.has_method("set_sea_level_live"))

	if timing > 0:
		var live_ms := []
		var paint_ms := []
		for i in 5:
			var lv := 0.48 if i % 2 == 0 else 0.42
			var t0 := Time.get_ticks_usec()
			var r: Dictionary = bridge.sea_level_live(lv)
			live_ms.append((Time.get_ticks_usec() - t0) / 1000.0)
			var t1 := Time.get_ticks_usec()
			app.viewport.map_view.texture = bridge.color_texture()
			paint_ms.append((Time.get_ticks_usec() - t1) / 1000.0)
			print("SEALIVE timing run %d: level %.2f ok=%s engine ms=%.1f" % [i, lv, r.get("ok"), float(r.get("ms", -1.0))])
		print("SEALIVE TIMING %dx%d live move %s; repaint (color_texture) %s" % [n, n, _median(live_ms), _median(paint_ms)])
		get_tree().quit(0)
		return

	var s := _find_sea_slider(app)
	_check("the World data Sea level slider exists", s != null)
	if s == null:
		print("SEALIVE RESULT FAIL (%d)" % _fails)
		get_tree().quit(0)
		return

	# A Coastline stamp on the draft: Plateau and Coastline read the level.
	wg.sculpt_set_feature("coastline")
	wg.sculpt_begin_stroke()
	for p in [Vector2(40, 96), Vector2(128, 96), Vector2(216, 96)]:
		wg.sculpt_add_point(p.x, p.y)
	wg.sculpt_end_stroke()

	# ---- 0. negative control ----------------------------------------------
	var c0: Dictionary = wg.sea_water_counts()
	var base := float(c0.get("level", NAN))
	_check("0: two reads agree before any move", JSON.stringify(c0) == JSON.stringify(wg.sea_water_counts()), str(c0))
	_check("0: nothing stale after the generate", bridge.stale_stages().is_empty(), str(bridge.stale_stages()))
	_check("0: the draft stamp carries the base level", is_equal_approx(_draft_level(wg), base), "%s vs %s" % [_draft_level(wg), base])
	var tex0 := _tex_bytes(app)
	var mil0 := JSON.stringify(wg.civ_military_summary())
	_check("0: the base map texture is readable", tex0.size() > 0)
	var same: Dictionary = bridge.sea_level_live(base)
	_check("0: an unchanged level reports changed=false", bool(same.get("ok", false)) and not bool(same.get("changed", true)), str(same))
	_check("0: and marks nothing stale", bridge.stale_stages().is_empty(), str(bridge.stale_stages()))
	var path_old := OS.get_user_data_dir().path_join("_sealive_base.zip")
	_check("5: a project at the base level is written", await _save(app, path_old))

	# ---- 1. raise through the slider --------------------------------------
	var hi := base + 0.06
	await _drag(s, hi)
	var c1: Dictionary = wg.sea_water_counts()
	_check("1: the effective level moved", is_equal_approx(float(c1.get("level", NAN)), hi), str(c1.get("level")))
	_check("1: the parameter moved with it", is_equal_approx(float(bridge.param_get("sea_level")), hi))
	_check("1: raising drowns land", int(c1["land_cells"]) < int(c0["land_cells"]), "%d vs %d" % [c1["land_cells"], c0["land_cells"]])
	_check("1: and adds ocean", int(c1["ocean_cells"]) > int(c0["ocean_cells"]), "%d vs %d" % [c1["ocean_cells"], c0["ocean_cells"]])
	_check("1: the coastline moved", int(c1["coast_cells"]) != int(c0["coast_cells"]), "%d vs %d" % [c1["coast_cells"], c0["coast_cells"]])
	_check("1: the civ layer's water re-derived (civ land = drawn land)", c1.has("civ_land_cells") and int(c1["civ_land_cells"]) == int(c1["land_cells"]), str(c1))
	var tex1 := _tex_bytes(app)
	_check("1: POSITIVE control -- the map texture changed", tex1.size() == tex0.size() and tex1 != tex0)
	_check("1: Ruling 17 -- the draft stamp follows the level", is_equal_approx(_draft_level(wg), hi), str(_draft_level(wg)))
	var st: Dictionary = bridge.stale_stages()
	_check("1: climate and civ read stale, reason sea_level",
		st.has("climate") and st.has("civ") and String((st["climate"] as Dictionary).get("reason", "")) == "sea_level", str(st))
	_check("1: height is not stale", not st.has("height"), str(st))
	var mil1 := JSON.stringify(wg.civ_military_summary())

	# ---- 2. lower ----------------------------------------------------------
	var lo := base - 0.06
	await _drag(s, lo)
	var c2: Dictionary = wg.sea_water_counts()
	_check("2: lowering exposes land", int(c2["land_cells"]) > int(c0["land_cells"]), "%d vs %d" % [c2["land_cells"], c0["land_cells"]])
	_check("2: civ land follows", int(c2.get("civ_land_cells", -1)) == int(c2["land_cells"]))
	var mil2 := JSON.stringify(wg.civ_military_summary())

	# ---- 3. restore: round-trip identity ----------------------------------
	await _drag(s, base)
	var c3: Dictionary = wg.sea_water_counts()
	_check("3: counts return exactly", JSON.stringify(c3) == JSON.stringify(c0), "%s vs %s" % [c3, c0])
	_check("3: the map texture returns byte for byte", _tex_bytes(app) == tex0)
	_check("3: the military summary returns exactly", JSON.stringify(wg.civ_military_summary()) == mil0)
	_check("3: the military water shares moved on raise or lower", mil1 != mil0 or mil2 != mil0)
	_check("3: the draft stamp is back at the base level", is_equal_approx(_draft_level(wg), base))

	# ---- 4. save -> reopen keeps the live level ----------------------------
	await _drag(s, hi)
	var c4: Dictionary = wg.sea_water_counts()
	var path := OS.get_user_data_dir().path_join("_sealive_moved.zip")
	_check("4: the moved project is written", await _save(app, path))
	var man = JSON.parse_string(_member(path, "project.json"))
	var saved_level := NAN
	if typeof(man) == TYPE_DICTIONARY and typeof(man.get("world")) == TYPE_DICTIONARY:
		saved_level = float((man["world"] as Dictionary).get("sea_level", NAN))
	_check("4: project.json's world.sea_level is the live level", is_equal_approx(saved_level, hi), str(saved_level))
	_check("4: the project opened", app._load_project(path))
	await _frames(10)
	var c5: Dictionary = bridge.world_gen.sea_water_counts()
	_check("4: reopened at the live level with the same counts", JSON.stringify(c5) == JSON.stringify(c4), "%s vs %s" % [c5, c4])
	_check("4: the reopened draft stamp keeps the live level", is_equal_approx(_draft_level(bridge.world_gen), hi), str(_draft_level(bridge.world_gen)))

	# ---- 5. a project saved before any move reopens unchanged --------------
	_check("5: the base project opened", app._load_project(path_old))
	await _frames(10)
	var c6: Dictionary = bridge.world_gen.sea_water_counts()
	_check("5: reopened at the base level with the base counts", JSON.stringify(c6) == JSON.stringify(c0), "%s vs %s" % [c6, c0])

	print("SEALIVE RESULT %s" % ("GREEN" if _fails == 0 else "FAIL (%d)" % _fails))
	get_tree().quit(0)
