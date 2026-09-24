extends Node
## Ruling AW -- CARTO ▸ Conflict, end to end through the real shell:
## generate, record two years with a border change between them (a real
## Territory brush dab and commit, snapshotted by `civ_add_year`), author a
## siege between the two factions, find the layer in the Layers popover by
## typing into its filter, toggle its row, and move the cursor.
##
## Checks:
##   - `conflict_campaigns()` reads a ring, a front and changed-hands cells at
##     an in-conflict year, and nothing outside the conflict's years;
##   - the Layers popover lists a "Conflict" row and toggling it turns
##     the layer off and on (`layer_visible("conflict")` follows);
##   - DRAWN: framebuffer with the layer on vs off (positive control) at an
##     in-conflict year moves pixels, both over the whole map and in a box
##     around the siege centre chosen from the input (the siege point), and
##     moves exactly 0 at a year outside the conflict.
##
## Windowed (pixel diff: `ImageTexture.update()` is a no-op headless), palette
## forced to light (what this machine boots) though the on/off diff is itself
## palette-agnostic:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _awconflict_probe.tscn
##
## Exit 0 = all passed, 1 = a real failure, 2 = could not run.

var _app: Node
var _bridge
var _fail := 0
var _vp: Viewport


func _p(s: String) -> void:
	print("AW  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _walk(n: Node, out: Array) -> Array:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)
	return out


func _grid_to_vp(gx: float, gy: float) -> Vector2:
	var ov: Control = _app.viewport.overlay
	var rect: Rect2 = ov.displayed_rect()
	var g: Vector2i = _bridge.grid_size()
	var local := rect.position + Vector2(gx / g.x, gy / g.y) * rect.size
	return ov.get_global_transform_with_canvas() * local


func _shot() -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	await _frames(1)
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()


## Pixels that differ between two captures, over the whole frame or inside
## `box` (viewport px).
func _moved(a: Image, b: Image, box: Rect2i = Rect2i()) -> int:
	var r := box if box.size != Vector2i.ZERO else Rect2i(0, 0, a.get_width(), a.get_height())
	r = r.intersection(Rect2i(0, 0, a.get_width(), a.get_height()))
	var n := 0
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


## The Conflict row of the Layers popover, found by typing into its filter.
func _popover_conflict_box() -> CheckBox:
	var pop: LayersPopover = _app.layers_popover
	pop.open()
	await _frames(4)
	pop._field.text = "conflict"
	pop._field.text_changed.emit("conflict")
	await _frames(4)
	for n in _walk(pop, []):
		if n is CheckBox:
			var row: Node = (n as Node).get_parent()
			while row != null and not (row is HBoxContainer):
				row = row.get_parent()
			if row == null:
				continue
			for m in _walk(row, []):
				if m is Label and (m as Label).text == "Conflict":
					return n
	return null


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	if DisplayServer.get_name() == "headless":
		_p("ABORT: headless -- the framebuffer is not readable. Run windowed.")
		get_tree().quit(2)
		return
	_vp = get_viewport()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	if not _bridge.world_gen.has_method("conflict_campaigns"):
		_p("ABORT: this binary has no conflict_campaigns #[func] -- cargo build -p cartalith-godot")
		get_tree().quit(2)
		return
	if DccTheme.is_dark():
		DccTheme.apply_theme(false)
		_app.rebuild_theme(true)
	_p("palette forced: %s" % ("dark" if DccTheme.is_dark() else "light"))

	## 400 km over 320 cells = 1.25 km per cell, so the attested 2.591 km siege
	## radius is 2.07 cells rather than the 1-cell floor.
	_bridge.generate({"seed": 5521, "width_km": 400.0, "grid_w": 320, "grid_h": 240,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_app.select_domain("civilization")
	await _frames(8)

	var places: Array = _bridge.settlements()
	var factions: Array = _bridge.get_factions()
	if places.is_empty() or factions.size() < 2:
		_p("ABORT: world has %d settlements, %d factions" % [places.size(), factions.size()])
		get_tree().quit(2)
		return
	var g0: Vector2i = _bridge.grid_size()
	var s0: Dictionary = {}
	for pl: Dictionary in places:
		if int(pl.get("faction", 0)) <= 0:
			continue
		if s0.is_empty() or Vector2(float(pl["x"]), float(pl["y"])).distance_to(Vector2(g0) * 0.5) \
				< Vector2(float(s0["x"]), float(s0["y"])).distance_to(Vector2(g0) * 0.5):
			s0 = pl
	var tid := int(s0.get("tid", 0))
	var f1 := int(s0.get("faction", 1))
	var f2 := -1
	for f: Dictionary in factions:
		if int(f.get("id", 0)) > 0 and int(f.get("id", 0)) != f1:
			f2 = int(f.get("id", 0))
			break
	var cx := float(s0["x"])
	var cy := float(s0["y"])
	_p("settlement tid=%d '%s' at (%d,%d), faction %d; other side %d" % [tid, String(s0.get("name", "")),
		int(cx), int(cy), f1, f2])

	# -- 1. Two recorded years, with a border change between them.
	var y0: int = _bridge.get_civ_year()
	_bridge.civ_add_year(y0)
	_bridge.civ_add_year(y0 + 10)
	_check("cursor moved to the second year", _bridge.get_civ_year() == y0 + 10, "year=%d" % _bridge.get_civ_year())
	var staged: bool = _bridge.civ_territory_paint_at(cx + 4.0, cy, f2, 6.0, false)
	var committed: bool = _bridge.civ_territory_commit()
	_check("a faction-%d dab over faction %d's town was staged and committed" % [f2, f1], staged and committed)
	_bridge.civ_add_year(y0 + 10)   ## snapshot the active year's live edit
	var years := Array(_bridge.get_civ_timeline_years())
	_check("both years are recorded", years.has(y0) and years.has(y0 + 10), str(years))

	# -- 2. A siege between the two, anchored to the town.
	var r: Dictionary = _bridge.conflict_add({"name": "Siege of the probe", "kind": "siege",
		"start_year": y0, "end_year": y0 + 20, "sides": PackedInt32Array([f1, f2]),
		"points": PackedVector2Array([Vector2(cx, cy)]), "anchor_kind": "settlement", "anchor_tid": tid})
	_check("the siege was authored", bool(r.get("ok", false)), str(r))
	var id := int(r.get("id", 0))

	# -- 3. The engine's reading, in and out of the conflict's years.
	var rows: Array = _bridge.conflict_campaigns(y0 + 10)
	_check("one campaign row at y0+10", rows.size() == 1, "n=%d" % rows.size())
	var row: Dictionary = rows[0] if rows.size() > 0 else {}
	var siege: Dictionary = row.get("siege", {})
	_p("  row: territory_year=%s baseline_year=%s front_cells=%s front_segments=%d changed=%d siege=%s" % [
		str(row.get("territory_year")), str(row.get("baseline_year")), str(row.get("front_cell_count")),
		(row.get("front_segments", PackedVector2Array()) as PackedVector2Array).size() / 2,
		(row.get("changed_cells", PackedInt32Array()) as PackedInt32Array).size(), str(siege)])
	_check("it reads territory at y0+10 against a y0 baseline",
		int(row.get("territory_year", -99999)) == y0 + 10 and int(row.get("baseline_year", -99999)) == y0)
	_check("the siege ring is centred on the town, 2.07 cells wide",
		siege.get("centre", Vector2(-1, -1)) == Vector2(cx, cy)
			and absf(float(siege.get("radius_cells", 0.0)) - 2.591042 / 1.25) < 1e-4,
		str(siege))
	_check("it names the besieged town", int(siege.get("besieged_tid", 0)) == tid)
	_check("there is a front", int(row.get("front_cell_count", 0)) > 0)
	var changed: PackedInt32Array = row.get("changed_cells", PackedInt32Array())
	var to: PackedInt32Array = row.get("changed_to", PackedInt32Array())
	_check("cells changed hands, all to faction %d" % f2, changed.size() > 0 and Array(to).all(func(t): return t == f2),
		"n=%d" % changed.size())
	for y in [y0 - 1, y0 + 21, y0 + 30]:
		_check("nothing in year %d" % y, _bridge.conflict_campaigns(y).is_empty())

	# -- 4. The layer is findable in the Layers popover and a click toggles it.
	_app.tl_set_year(y0 + 10)
	await _frames(4)
	var ov: Control = _app.viewport.overlay
	_check("the layer draws the siege at y0+10", ov.drawn_campaign_ids() == [id], str(ov.drawn_campaign_ids()))
	var box: CheckBox = await _popover_conflict_box()
	_check("the Layers popover lists a 'Conflict' row under its filter", box != null)
	if box == null:
		get_tree().quit(1)
		return
	_check("its switch starts on", box.button_pressed and _app.viewport.layer_visible("conflict"))
	## `button_pressed`, the SP-4 probe's own way of ticking a dock toggle: a
	## pushed mouse event does not reach a control inside the embedded popup
	## window (measured: the tap moved nothing). The setter fires `toggled`,
	## which is the callback the row wires to `set_layer_visible`.
	box.button_pressed = false
	await _frames(3)
	_check("unticking the row turns the layer off", not _app.viewport.layer_visible("conflict") and ov.drawn_campaign_ids().is_empty())
	box.button_pressed = true
	await _frames(3)
	_check("ticking it again turns it back on", _app.viewport.layer_visible("conflict") and ov.drawn_campaign_ids() == [id])
	_app.layers_popover.hide()
	await _frames(4)

	# -- 5. Pixels: on vs off at an in-conflict year, then outside it.
	var c_vp := _grid_to_vp(cx, cy)
	var ring_box := Rect2i(Vector2i(c_vp) - Vector2i(60, 60), Vector2i(120, 120))
	var on_in := await _shot()
	var shot := OS.get_environment("AW_SHOT")
	if shot != "":
		on_in.get_region(ring_box).save_png(shot)
	_app.viewport.set_layer_visible("conflict", false)
	var off_in := await _shot()
	_app.viewport.set_layer_visible("conflict", true)
	var whole_in := _moved(on_in, off_in)
	var near_in := _moved(on_in, off_in, ring_box)
	_p("  y0+10: on/off moved %d px over the frame, %d px in the 120x120 box at the siege" % [whole_in, near_in])
	_check("in the conflict's years the layer moves pixels (whole frame)", whole_in > 200, "moved=%d" % whole_in)
	_check("and around the siege centre", near_in > 50, "moved=%d" % near_in)

	_app.tl_set_year(y0 + 30)
	await _frames(4)
	_check("at y0+30 the layer has nothing to draw", ov.drawn_campaign_ids().is_empty())
	var on_out := await _shot()
	_app.viewport.set_layer_visible("conflict", false)
	var off_out := await _shot()
	_app.viewport.set_layer_visible("conflict", true)
	var whole_out := _moved(on_out, off_out)
	_p("  y0+30: on/off moved %d px over the frame" % whole_out)
	_check("outside the conflict's years the layer moves 0 pixels", whole_out == 0, "moved=%d" % whole_out)

	# -- 6. Back in: the cursor brings it back.
	_app.tl_set_year(y0 + 20)
	await _frames(4)
	_check("its last year draws it again", ov.drawn_campaign_ids() == [id])

	_p("RESULT %s (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
