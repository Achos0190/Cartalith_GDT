extends Node
## Windowed proof of Ruling AT's manual viewshed refine (`OUTSTANDING_WORK.md`:
## "Ruling 16's manual 'recompute and refine' viewshed action is unbuilt").
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _viewshedrefine_probe.tscn
##
## Real world, real engine, the shell's own menu row and the app's own save and
## open paths. What it proves, each with a control that must move:
##   1. never refined: the status is `never`, and a landmark pass is the coarse
##      one -- no landmark's causal text says "(refined)" (the NEGATIVE control
##      for 3).
##   2. refusals do not store: a rectangle off the grid, an empty one and a
##      too-large one return `ok: false` with a reason and leave `never`.
##   3. a refine over a sub-rectangle is `fresh`, and the next landmark pass
##      reads it **inside the rectangle only**: causal text says "(refined)"
##      for landmarks inside, never for any outside (the POSITIVE control -- if
##      no landmark's text moves, the probe fails rather than passing
##      vacuously).
##   4. saved with the project: the written `entities/landmarks.json` holds the
##      `viewshed_refined` member, and after File > Open the status is `fresh`
##      again with the same rectangle and a landmark pass still reads it.
##   5. the shell row: the Atlas cache submenu carries "Refine viewshed for the
##      current view" beside "Refine detail ...", with a status readout under
##      it; pressing it (through `id_pressed`) reaches the engine; the pure
##      helpers `view_cell_rect` and `viewshed_status_text` are pinned.
##   6. stale falls back: regenerating a different world marks the stored
##      result stale (status says why) and a pass over it is the coarse one
##      again; `landmark_refine_clear` returns to `never`.
##   7. old files: a project written with no refine carries no
##      `viewshed_refined` member at all.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("VSREFINE %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

## How many landmarks say "(refined)" in their causal text, split by whether
## the landmark's cell is inside `rect`: `[inside, outside]`.
func _refined_counts(lms: Array, rect: Rect2i) -> Array:
	var inside := 0
	var outside := 0
	for l in lms:
		var d: Dictionary = l
		var txt := " | ".join(PackedStringArray(d["causal"]))
		if txt.contains("(refined)"):
			if rect.has_point(Vector2i(int(d["x"]), int(d["y"]))):
				inside += 1
			else:
				outside += 1
	return [inside, outside]

func _find_popup(n: Node, nm: String) -> PopupMenu:
	if n is PopupMenu and String(n.name) == nm:
		return n
	for c in n.get_children(true):
		var r := _find_popup(c, nm)
		if r != null:
			return r
	return null

func _row_index(p: PopupMenu, needle: String) -> int:
	for i in p.item_count:
		if p.get_item_text(i).findn(needle) >= 0:
			return i
	return -1

## Writes the project through the app's own path; returns whether the file exists.
func _save(app: Node, path: String) -> bool:
	var done := [false]
	app._write_project(path, func(): done[0] = true)
	for _i in 200:
		if done[0]:
			break
		await _frames(2)
	return FileAccess.file_exists(path)

## The text of the zip member whose name ends with `suffix`, or "" if absent.
func _member(path: String, suffix: String) -> String:
	var zip := ZIPReader.new()
	var out := ""
	if zip.open(path) == OK:
		for f in zip.get_files():
			if String(f).ends_with(suffix):
				out = zip.read_file(f).get_string_from_utf8()
		zip.close()
	return out

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("VSREFINE REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("VSREFINE WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	# 384 x 288 over 500 km: ~1.3 km cells, so the 40 km horizon is ~31 cells.
	var req := {"seed": 9137, "width_km": 500.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45}
	bridge.generate(req)
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	# ---- 1. never refined ------------------------------------------------
	var st: Dictionary = bridge.landmark_refine_status()
	_check("1: a new world reads 'never refined'", String(st.get("state", "")) == "never", str(st))
	_check("1: and carries no rectangle (absent, not zeros)", not st.has("w") and not st.has("x"), str(st))
	var base: Dictionary = await bridge.landmark_run()
	_check("1: the coarse landmark pass places landmarks", bool(base.get("ok", false)) and int(base.get("placed", 0)) > 0,
		str(base.get("error", base.get("placed", ""))))
	var coarse: Array = bridge.landmarks()
	var coarse_json := JSON.stringify(coarse)
	var c0: Array = _refined_counts(coarse, Rect2i(0, 0, 384, 288))
	_check("1: NEGATIVE control -- no landmark says '(refined)' before any refine", int(c0[0]) + int(c0[1]) == 0, str(c0))

	# ---- 2. refusals store nothing --------------------------------------
	for bad in [[-5, 0, 40, 40], [0, 0, 0, 10], [380, 280, 40, 40], [0, 0, 5000, 5000]]:
		var r: Dictionary = await bridge.landmark_refine_view(bad[0], bad[1], bad[2], bad[3])
		_check("2: refused %s with a reason" % str(bad), not bool(r.get("ok", true)) and String(r.get("error", "")) != "", str(r))
	_check("2: refusals left 'never refined'", String(bridge.landmark_refine_status().get("state", "")) == "never")

	# ---- 3. a sub-rectangle refine, read inside only --------------------
	var rect := Rect2i(96, 72, 192, 144)
	var t0 := Time.get_ticks_msec()
	var rr: Dictionary = await bridge.landmark_refine_view(rect.position.x, rect.position.y, rect.size.x, rect.size.y)
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0
	_check("3: the refine succeeded", bool(rr.get("ok", false)), "%s (%.1f s)" % [str(rr.get("error", "")), secs])
	_check("3: it is fresh and reports its rectangle and observers",
		String(rr.get("state", "")) == "fresh" and int(rr.get("w", 0)) == rect.size.x
		and int(rr.get("x", -1)) == rect.position.x and int(rr.get("observers", 0)) > 0, str(rr))
	var again: Dictionary = await bridge.landmark_run()
	_check("3: the landmark pass still runs", bool(again.get("ok", false)), str(again.get("error", "")))
	var refined: Array = bridge.landmarks()
	var c1: Array = _refined_counts(refined, rect)
	_check("3: POSITIVE control -- some landmark inside the view reads the refined result", int(c1[0]) > 0, str(c1))
	_check("3: and none outside it does", int(c1[1]) == 0, str(c1))
	_check("3: the refined pass really differs from the coarse one", JSON.stringify(refined) != coarse_json)

	# ---- 4. saved with the project, restored on open --------------------
	var path := OS.get_user_data_dir().path_join("_viewshedrefine_probe.zip")
	var wrote: bool = await _save(app, path)
	_check("4: the project was written", wrote, path)
	_check("4: entities/landmarks.json carries the viewshed_refined member",
		_member(path, "landmarks.json").contains("\"viewshed_refined\""))
	var opened: bool = app._load_project(path)
	await _frames(10)
	_check("4: the project opened", opened)
	var st2: Dictionary = bridge.landmark_refine_status()
	_check("4: after open the status is fresh again, same rectangle",
		String(st2.get("state", "")) == "fresh" and int(st2.get("x", -1)) == rect.position.x
		and int(st2.get("w", -1)) == rect.size.x and int(st2.get("h", -1)) == rect.size.y, str(st2))
	var re_run: Dictionary = await bridge.landmark_run()
	if bool(re_run.get("ok", false)):
		var c2: Array = _refined_counts(bridge.landmarks(), rect)
		_check("4: a pass over the reopened project still reads the refined result", int(c2[0]) > 0 and int(c2[1]) == 0, str(c2))
	else:
		_check("4: a pass over the reopened project", false, str(re_run.get("error", "")))

	# ---- 5. the shell row and its helpers -------------------------------
	var atlas := _find_popup(app, "AtlasCache")
	_check("5: the Atlas cache submenu exists", atlas != null)
	if atlas != null:
		atlas.about_to_popup.emit()
		await _frames(2)
		var detail := _row_index(atlas, "Refine detail for the current view")
		var row := _row_index(atlas, "Refine viewshed for the current view")
		_check("5: the row sits beside Refine detail", row >= 0 and detail >= 0 and row == detail + 1,
			"row %d, detail %d" % [row, detail])
		var status_row := _row_index(atlas, "Viewshed:")
		_check("5: a disabled status readout follows it, naming the state", status_row == row + 1
			and atlas.is_item_disabled(status_row) and atlas.get_item_text(status_row).contains("fresh"),
			atlas.get_item_text(status_row) if status_row >= 0 else "none")
		# Back to 'never' so a press is observable, then press the real row.
		bridge.landmark_refine_clear()
		atlas.about_to_popup.emit()
		await _frames(2)
		_check("5: the readout follows the engine back to 'never refined'",
			atlas.get_item_text(status_row).contains("never"), atlas.get_item_text(status_row))
		atlas.id_pressed.emit(atlas.get_item_id(row))
		var waited := 0
		## The status is an EMPTY dictionary while the worker holds the engine,
		## so wait for 'fresh' itself rather than for 'not never'.
		while String(bridge.landmark_refine_status().get("state", "")) != "fresh" and waited < 480:
			await get_tree().create_timer(0.25).timeout
			waited += 1
		var st3: Dictionary = bridge.landmark_refine_status()
		_check("5: pressing the row reached the engine (the view is now refined)",
			String(st3.get("state", "")) == "fresh", str(st3))
		await _frames(4)
		var hint := String(app.status_slot_text("hint")) if app.has_method("status_slot_text") else ""
		_check("5: the status bar says what happened", hint.findn("viewshed") >= 0, "hint=" + hint)
		atlas.about_to_popup.emit()
		await _frames(2)
		_check("5: the readout now says fresh", atlas.get_item_text(status_row).contains("fresh"), atlas.get_item_text(status_row))
	var vr := DccMenus.view_cell_rect({"x0": 10.4, "y0": 3.0, "x1": 20.2, "y1": 9.0001}, Vector2i(384, 288))
	_check("5: view_cell_rect floors the low corner and ceils the high one", vr == Rect2i(10, 3, 11, 7), str(vr))
	var vr2 := DccMenus.view_cell_rect({"x0": -4.0, "y0": 0.0, "x1": 999.0, "y1": 999.0}, Vector2i(384, 288))
	_check("5: view_cell_rect clamps to the grid", vr2 == Rect2i(0, 0, 384, 288), str(vr2))
	var vr3 := DccMenus.view_cell_rect({"x0": 5.0, "y0": 5.0, "x1": 5.0, "y1": 5.0}, Vector2i(384, 288))
	_check("5: an empty view is an empty rectangle", vr3.size == Vector2i.ZERO, str(vr3))
	for s in ["never", "fresh", "stale_terrain", "stale_observers", "stale_both"]:
		var txt := DccMenus.viewshed_status_text({"state": s, "radius_km": 40.0, "w": 5, "h": 6})
		var word := "never" if s == "never" else ("fresh" if s == "fresh" else "stale")
		_check("5: status text for %s says '%s'" % [s, word], txt.begins_with("Viewshed:") and txt.contains(word), txt)
	_check("5: an empty status reads as unknown, never as 'never refined'",
		DccMenus.viewshed_status_text({}) == "Viewshed: unknown")

	# ---- 6. a different world marks it stale and falls back ---------------
	var req2 := req.duplicate()
	req2["seed"] = 4242
	bridge.generate(req2)
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	var st4: Dictionary = bridge.landmark_refine_status()
	_check("6: after regenerating a different world the result is stale", String(st4.get("state", "")).begins_with("stale"), str(st4))
	var after: Dictionary = await bridge.landmark_run()
	_check("6: a pass over the stale result runs", bool(after.get("ok", false)), str(after.get("error", "")))
	var c4: Array = _refined_counts(bridge.landmarks(), Rect2i(0, 0, 384, 288))
	_check("6: and is the coarse one -- no landmark reads the stale result", int(c4[0]) + int(c4[1]) == 0, str(c4))
	bridge.landmark_refine_clear()
	_check("6: clearing returns to 'never refined'", String(bridge.landmark_refine_status().get("state", "")) == "never")

	# ---- 7. a never-refined project writes no member ---------------------
	var path2 := OS.get_user_data_dir().path_join("_viewshedrefine_probe2.zip")
	var wrote2: bool = await _save(app, path2)
	var lm_text := _member(path2, "landmarks.json")
	_check("7: a never-refined project is written, and its landmarks.json has no viewshed_refined member",
		wrote2 and lm_text != "" and not lm_text.contains("viewshed_refined"), "%d bytes" % lm_text.length())

	DirAccess.remove_absolute(path)
	DirAccess.remove_absolute(path2)
	print("VSREFINE %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
