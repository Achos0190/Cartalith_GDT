extends Node
## Verifies the saved-measurements store added 2026-09-03: the fifth *slot*
## anything in this shell writes and the sixth caller-owned one the format
## defines (`annotations/measurements.json`).
##
## Reaches what no Rust test can. `cargo test` proves the slot rides the
## archive channel (`tests/project_document_channel.rs`); everything the store
## itself is -- what a measurement carries, what happens to one when the world
## under it is replaced, and whether the CSV moves with the Units preference --
## lives in GDScript over a live `WorldGen`, which is a cdylib `GodotClass` and
## cannot be constructed in a unit test.
##
## Run: godot4 --headless --path . _measurestore_probe.tscn

var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

var _fails := 0

func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok: %s" % what)
	else:
		print("  FAIL: %s" % what)
		_fails += 1

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	# --- the pure half first: no world, no bridge, no dock ------------------
	print("--- document shape (static) ---")
	var no_value: Array = [{"mode": "distance", "points": PackedVector2Array([Vector2(1, 2), Vector2(3, 4)])}]
	var nv_text := RightDock.measurements_document_text(no_value, 64, 48)
	_check(not nv_text.contains("\"value\""),
		"a reading with no single number omits `value` rather than writing a 0 that reads as a measurement")
	_check(not nv_text.contains("\"unit\""), "`unit` is omitted with it, never alone")
	var nv_csv := RightDock.measurements_csv(no_value)
	_check(nv_csv.split("\n")[1].contains(",2,,,"),
		"the CSV writes an EMPTY cell for an absent reading, not a zero (%s)" % nv_csv.split("\n")[1])
	_check(nv_csv.begins_with("index,mode,point_count,value,unit,points_cells"),
		"the CSV header is fixed and six columns wide")

	var mismatch := RightDock.measurements_from_document(nv_text, 999, 777)
	_check(not bool(mismatch.get("ok", true)), "a document from another grid is REFUSED, not carried")
	_check(String(mismatch.get("reason", "")).contains("64x48")
		and String(mismatch.get("reason", "")).contains("999x777"),
		"the refusal names both grids: %s" % String(mismatch.get("reason", "")))
	_check((mismatch.get("entries", []) as Array).is_empty(), "a refused document yields no entries")
	_check(bool(RightDock.measurements_from_document(nv_text, 64, 48).get("ok", false)),
		"the same document on its own grid is accepted")
	_check(not bool(RightDock.measurements_from_document("not json", 64, 48).get("ok", true)),
		"a document that is not JSON is refused rather than crashing")

	## A null `value` must cost that ONE reading's number, not the document.
	##
	## `float(<null>)` is a GDScript runtime error, not a conversion, so before
	## the 2026-09-03 guard it aborted the whole reader: the result carried no
	## `ok`, the caller took the `ok == false` branch, cleared the in-memory
	## list and returned no reason -- silent, total loss. And this build writes
	## that null itself, for a mode that produced a NaN. Verified by a verifier
	## as "0 entries recovered, silently" from a document holding one null entry
	## beside one healthy one; these two assertions are what makes reverting the
	## guard fail instead of merely misbehaving.
	var poisoned := JSON.stringify({
		"gw": 64, "gh": 48,
		"measurements": [
			{"mode": "distance", "points": [[1, 2], [3, 4]], "value": null, "unit": "km"},
			{"mode": "area", "points": [[5, 6], [7, 8], [9, 10]], "value": 12.5, "unit": "km2"},
			{"mode": "distance", "points": [[1, 2], [null, 4]], "value": 3.0, "unit": "km"},
		],
	})
	var poisoned_r: Dictionary = RightDock.measurements_from_document(poisoned, 64, 48)
	_check(bool(poisoned_r.get("ok", false)),
		"a null value does not abort the whole document")
	var pe: Array = poisoned_r.get("entries", [])
	_check(pe.size() == 3,
		"every entry survives a null beside it, got %d of 3" % pe.size())
	if pe.size() == 3:
		_check(not (pe[0] as Dictionary).has("value"),
			"the null-valued entry keeps its points and drops only its number")
		_check(is_equal_approx(float((pe[1] as Dictionary).get("value", -1.0)), 12.5),
			"and the healthy entry's own value is untouched")
		## The null *coordinate* costs its own point and nothing else. Three,
		## not two, is the reader's existing rule and not a new one: an entry is
		## dropped only when it ends up with NO points, so a two-point distance
		## whose second coordinate is malformed comes back as a one-point entry.
		## Whether a one-point "distance" is worth keeping is a separate policy
		## question this guard deliberately does not answer -- it was already
		## the behaviour for a short `points` array, and changing it here would
		## be a silent scope change riding on a crash fix.
		_check((pe[2] as Dictionary).get("points", PackedVector2Array()).size() == 1,
			"a null coordinate costs that point alone, got %d" \
				% (pe[2] as Dictionary).get("points", PackedVector2Array()).size())

	# --- now the live half --------------------------------------------------
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({
		"seed": 41207, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(4)

	var rd = _app.right_dock_ctrl
	var grid: Vector2i = bridge.grid_size()
	print("--- take two readings on a %dx%d world ---" % [grid.x, grid.y])

	GlobalTools.set_measure_mode(_app, "distance")
	_app.arm_tool("measure")
	for p in [Vector2(10, 12), Vector2(90, 30), Vector2(140, 88)]:
		GlobalTools._measure_click(_app, p.x, p.y)
	await _frames(2)
	var primary: Dictionary = rd._measure_primary()
	_check(not primary.is_empty() and String(primary.get("unit", "")) == "km",
		"a three-point distance chain has a primary reading in km: %s" % primary)
	rd._on_measure_save()

	GlobalTools.set_measure_mode(_app, "area")
	for p in [Vector2(20, 20), Vector2(80, 20), Vector2(80, 70), Vector2(20, 70)]:
		GlobalTools._measure_click(_app, p.x, p.y)
	await _frames(2)
	var area_primary: Dictionary = rd._measure_primary()
	_check(String(area_primary.get("unit", "")) == "km2",
		"an area ring reads in km2, not km: %s" % area_primary)
	rd._on_measure_save()
	await _frames(2)
	_check(rd._saved_measurements.size() == 2, "two measurements saved, got %d" % rd._saved_measurements.size())

	var saved_km := float((rd._saved_measurements[0] as Dictionary).get("value", 0.0))
	var saved_km2 := float((rd._saved_measurements[1] as Dictionary).get("value", 0.0))
	var saved_pts: PackedVector2Array = (rd._saved_measurements[0] as Dictionary).get("points", PackedVector2Array())
	print("  distance %.4f km over %d points; area %.1f km2" % [saved_km, saved_pts.size(), saved_km2])
	_check(saved_km > 0.0 and saved_km2 > 0.0, "both readings are non-zero")
	_check(saved_pts.size() == 3, "the distance entry kept all three clicked points")

	# --- canonical km, whatever the display unit is -------------------------
	print("--- units are display-only ---")
	var before := DccSettings.units_mode()
	DccSettings.set_units_mode("km")
	var text_km: String = rd._saved_value_text(rd._saved_measurements[0])
	var csv_km := RightDock.measurements_csv(rd._saved_measurements)
	DccSettings.set_units_mode("mi")
	var text_mi: String = rd._saved_value_text(rd._saved_measurements[0])
	var csv_mi := RightDock.measurements_csv(rd._saved_measurements)
	var doc_mi: String = rd.measurements_document()
	DccSettings.set_units_mode(before)
	print("  list row reads %s in km mode, %s in mi mode" % [text_km, text_mi])
	_check(text_km != text_mi, "the on-screen LIST does convert -- otherwise this test pins nothing")
	_check(csv_km == csv_mi, "the CSV does NOT: same bytes under km and mi")
	_check(csv_mi.contains("%.4f" % saved_km) and csv_mi.contains(",km,"),
		"the CSV carries the canonical km figure and says `km` in its unit column")
	_check(doc_mi.contains("\"unit\":\"km\""), "and so does the stored document")

	# --- the archive round trip, through the real writer ---------------------
	print("--- project round trip ---")
	var path := ProjectSettings.globalize_path("user://_measurestore_probe.zip")
	var documents: Dictionary = _app._project_documents()
	_check(documents.has("annotations/measurements.json"),
		"_project_documents() merges the slot alongside the engine's own four")
	_check(bool(bridge.save_project(path, documents)), "the project saved")
	_app._load_project(path)
	await _frames(4)
	_check(bridge.last_documents.has("annotations/measurements.json"),
		"project_open handed the slot back -- it rode the channel, it did not bypass it")
	_check(rd._saved_measurements.size() == 2,
		"both measurements came back, got %d" % rd._saved_measurements.size())
	if rd._saved_measurements.size() == 2:
		var back_km := float((rd._saved_measurements[0] as Dictionary).get("value", 0.0))
		var back_pts: PackedVector2Array = (rd._saved_measurements[0] as Dictionary).get("points", PackedVector2Array())
		_check(absf(back_km - saved_km) < 1e-9, "the km figure survived exactly (%.6f vs %.6f)" % [back_km, saved_km])
		_check(back_pts == saved_pts, "and so did every clicked point: %s" % back_pts)
		_check(String((rd._saved_measurements[1] as Dictionary).get("mode", "")) == "area",
			"the second entry is still an area measurement")

	# --- recall -------------------------------------------------------------
	print("--- recall ---")
	rd._on_measure_recall(0)
	await _frames(2)
	_check(GlobalTools.measure_mode() == "distance", "recall re-armed the entry's own mode")
	_check(GlobalTools.measure_points() == saved_pts,
		"and put its points back on the chain: %s" % GlobalTools.measure_points())

	# --- the world-anchor rule ----------------------------------------------
	print("--- a replaced world clears the store ---")
	bridge.generate({
		"seed": 90210, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	_check(rd._saved_measurements.is_empty(),
		"a regenerate drops readings taken on the previous world, got %d" % rd._saved_measurements.size())
	_check(rd.measurements_document() == "",
		"and an empty store writes NO slot rather than an empty document")

	print("=== SUMMARY fails=%d ===" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)
