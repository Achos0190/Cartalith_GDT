extends Node
## Windowed verification for the Data manager's Import ▸ GIS / GeoJSON route
## (`LARGE_ITEM_RULINGS.md` Ruling V, 2026-09-21 -- `GUI_GAP_REGISTER.md`
## DM-03's other half). Boots the real app, generates a small world, writes a
## synthetic GeoJSON document naming a faction this world does not have,
## drives the real Data manager window and its real "Import GeoJSON…" button
## through the shell's own file browser once (to prove the button is wired to
## a real picker, `_datapane_probe.gd`'s `_browse_is_the_shell_browser()` is
## the precedent for this half), then applies the fixture the way that
## dialog's own callback would (`dm._run_geojson_import(path)` directly --
## probes in this tree routinely call a window's own underscore-prefixed
## methods rather than automate the native file browser's row clicks, same
## precedent file, `_no_invented_counts()`'s direct `dm._gis_dest =` write).
##
## Asserts the resulting WORLD STATE, not just that no error was thrown: the
## named faction exists in `get_factions()` afterward and did not before, and
## the named settlement is actually in `settlements()` at the fixture's own
## grid cell, population and faction id. A second document naming an EXISTING
## faction is applied afterward as the control -- no duplicate is created.
##
## Run: godot4 --path . _geojsonimport_probe.tscn   (WINDOWED -- no
## `--headless`. `ImageTexture.update()` is a no-op under `--headless`, so a
## screenshot taken there would pass vacuously; MISTAKES.md's own rule for
## every pixel probe in this tree. The screenshot is evidence and is not
## committed -- delete it after review.)

var _app: Node

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_geojsonimport_probe: run WINDOWED, not --headless.")
		print("PROBE REFUSED: headless")
		get_tree().quit(2)
		return

	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_app.open_project_dialog.hide()
	await get_tree().process_frame

	var bridge = _app.bridge
	bridge.generate({
		"seed": 90121, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	var fails := 0
	var gs: Vector2i = bridge.grid_size()
	var gw := gs.x
	var gh := gs.y
	var cell_km := 2000.0 / float(gw)
	var pick_r := maxf(float(gw) / 50.0, 5.0)

	# -- find a real land cell, clear of every existing settlement's pick radius
	var places: Array = bridge.settlements()
	var gx := -1
	var gy := -1
	for cand_y in range(10, gh - 10, 7):
		for cand_x in range(10, gw - 10, 7):
			var s: Dictionary = bridge.sample_cell(cand_x, cand_y)
			if String(s.get("water", "")) != "land":
				continue
			var clear := true
			for p in places:
				var pd := p as Dictionary
				var dx := float(pd["x"]) - float(cand_x)
				var dy := float(pd["y"]) - float(cand_y)
				if sqrt(dx * dx + dy * dy) < pick_r + 2.0:
					clear = false
					break
			if clear:
				gx = cand_x
				gy = cand_y
				break
		if gx >= 0:
			break
	print("PROBE land cell: (%d, %d) of %dx%d, pick_r=%.1f" % [gx, gy, gw, gh, pick_r])
	if gx < 0:
		print("PROBE FAIL: could not find a clear land cell for the fixture")
		fails += 1
		_finish(fails)
		return

	const FACTION_NAME := "Whitestone Confederacy"
	const SETTLEMENT_NAME := "Port Whitestone"
	const FIXTURE_POP := 4200

	var before: Array = bridge.get_factions()
	var before_names := PackedStringArray()
	for f in before:
		before_names.append(String((f as Dictionary).get("name", "")))
	print("PROBE factions before (%d): %s" % [before.size(), ", ".join(before_names)])
	if before_names.has(FACTION_NAME):
		print("PROBE FAIL: fixture faction name already present in this world -- pick a different one")
		fails += 1

	var east := float(gx) * cell_km
	var north := (float(gh) - float(gy)) * cell_km
	var fixture_path := _write_fixture(east, north, SETTLEMENT_NAME, FACTION_NAME, FIXTURE_POP)
	print("PROBE fixture: %s" % fixture_path)

	# -- drive the REAL window and its REAL button ----------------------------
	_app.open_data_manager_route("import_gis")
	await get_tree().process_frame
	await get_tree().process_frame
	var dm = _app.data_manager_window

	var go: Button = _button_named(dm, "Import GeoJSON…")
	if go == null:
		print("PROBE FAIL: 'Import GeoJSON…' button not found in the pane")
		fails += 1
	else:
		print("PROBE ok: 'Import GeoJSON…' button present, disabled=%s" % go.disabled)
		if go.disabled:
			print("PROBE FAIL: the button is disabled with a world loaded")
			fails += 1
		go.emit_signal("pressed")
		await get_tree().process_frame
		await get_tree().process_frame
		var browsers := _count_of(dm, "DccBrowseDialog")
		print("PROBE the real button spawned %d DccBrowseDialog(s)" % browsers)
		if browsers < 1:
			print("PROBE FAIL: pressing Import GeoJSON… raised no file browser")
			fails += 1
		for c in dm.get_children():
			if c is DccBrowseDialog:
				c.queue_free()
		await get_tree().process_frame

	# -- apply the fixture the way that dialog's own callback would ----------
	dm._run_geojson_import(fixture_path)
	await get_tree().process_frame
	await get_tree().process_frame
	await get_tree().create_timer(0.3).timeout

	var after: Array = bridge.get_factions()
	var after_names := PackedStringArray()
	for f in after:
		after_names.append(String((f as Dictionary).get("name", "")))
	print("PROBE factions after (%d): %s" % [after.size(), ", ".join(after_names)])
	if not after_names.has(FACTION_NAME):
		print("PROBE FAIL: %s was not created by the import" % FACTION_NAME)
		fails += 1
	else:
		print("PROBE ok: %s created -- roster grew %d -> %d" % [FACTION_NAME, before.size(), after.size()])

	var new_faction_id := -1
	for f in after:
		if String((f as Dictionary).get("name", "")) == FACTION_NAME:
			new_faction_id = int((f as Dictionary)["id"])

	var placed: Dictionary = {}
	for p in bridge.settlements():
		var pd := p as Dictionary
		if String(pd.get("name", "")) == SETTLEMENT_NAME:
			placed = pd
	if placed.is_empty():
		print("PROBE FAIL: settlement '%s' was not placed" % SETTLEMENT_NAME)
		fails += 1
	else:
		print("PROBE ok: settlement '%s' placed at (%s,%s) pop=%s faction=%s (expected faction %d)"
			% [SETTLEMENT_NAME, placed.get("x"), placed.get("y"), placed.get("population"),
				placed.get("faction"), new_faction_id])
		if int(placed.get("x", -1)) != gx or int(placed.get("y", -1)) != gy:
			print("PROBE FAIL: settlement landed at the wrong cell")
			fails += 1
		if int(placed.get("population", -1)) != FIXTURE_POP:
			print("PROBE FAIL: settlement population is not the fixture's own %d" % FIXTURE_POP)
			fails += 1
		if new_faction_id < 0 or int(placed.get("faction", -1)) != new_faction_id:
			print("PROBE FAIL: settlement is not on the newly created faction")
			fails += 1

	# -- the control: a document naming an EXISTING faction creates nothing --
	var existing_faction_name := String((before[0] as Dictionary)["name"]) if not before.is_empty() else ""
	if existing_faction_name != "":
		var gx2 := gx
		var gy2 := gy - 5 if gy - 5 >= 10 else gy + 5
		var east2 := float(gx2) * cell_km
		var north2 := (float(gh) - float(gy2)) * cell_km
		var control_path := _write_fixture(east2, north2, "Control Hamlet", existing_faction_name, 300)
		dm._run_geojson_import(control_path)
		await get_tree().process_frame
		await get_tree().process_frame
		await get_tree().create_timer(0.3).timeout
		var after2: Array = bridge.get_factions()
		print("PROBE control: naming existing faction '%s' -- roster now %d (was %d after the first import)"
			% [existing_faction_name, after2.size(), after.size()])
		if after2.size() != after.size():
			print("PROBE FAIL: naming an existing faction created a duplicate")
			fails += 1
		else:
			print("PROBE ok: no duplicate faction created for an existing name")

	await get_tree().process_frame
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://_geojsonimport_probe.png")
	print("PROBE screenshot: res://_geojsonimport_probe.png (%dx%d)" % [img.get_width(), img.get_height()])

	_finish(fails)

func _finish(fails: int) -> void:
	print("=== GEOJSON IMPORT PROBE ", "OK" if fails == 0 else "FAILED (%d)" % fails, " ===")
	get_tree().quit(0 if fails == 0 else 1)

func _write_fixture(east: float, north: float, settlement_name: String,
		faction_name: String, pop: int) -> String:
	var doc_text := JSON.stringify({
		"type": "FeatureCollection",
		"features": [{
			"type": "Feature",
			"geometry": {"type": "Point", "coordinates": [east, north]},
			"properties": {"layer": "settlement", "name": settlement_name,
				"kind": "town", "pop": pop, "factionName": faction_name},
		}],
	})
	var path := ProjectSettings.globalize_path(
		"user://_geojsonimport_probe_%d.geojson" % Time.get_ticks_usec())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(doc_text)
	f.close()
	return path

func _button_named(root: Node, text: String) -> Button:
	if root is Button and (root as Button).text == text:
		return root
	for c in root.get_children(true):
		var found := _button_named(c, text)
		if found != null:
			return found
	return null

func _count_of(root: Node, cls: String) -> int:
	var n := 0
	if cls == "DccBrowseDialog" and root is DccBrowseDialog:
		n += 1
	for c in root.get_children(true):
		n += _count_of(c, cls)
	return n
