extends Node
## `STORY_PLANNING_SCOPE.md` SP-4 -- the conflict overlay, end to end through
## the real shell: arm the Conflict tool by a real click on its palette button,
## draw a front with three real clicks on the map, press the real ✓ Commit,
## then fill the right dock's Conflict form (name, attach to a settlement,
## tick two sides), and check:
##
##   - the entity the engine holds is what was drawn and typed;
##   - each side shows real manpower, equal to CIVIL ▸ Military's own figures
##     for that faction (`civ_military_summary`, a separate call path);
##   - the conflict is DRAWN: framebuffer diff with the layer on vs emptied
##     (a positive control that must move, not a scene-graph claim);
##   - it appears and disappears as the year cursor crosses its range;
##   - "what conflicts happened here" answers by tid;
##   - it survives save -> open, anchor and all.
##
## Windowed (pixel diff: `ImageTexture.update()` is a no-op headless):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _sp4conflict_probe.tscn
##
## Exit 0 = all passed, 1 = a real failure, 2 = could not run.

var _app: Node
var _bridge
var _fail := 0
var _vp: Viewport


func _p(s: String) -> void:
	print("SP4  %s" % s)


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


func _press(at: Vector2, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)


func _tap(at: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = at
	m.global_position = at
	_vp.push_input(m, true)
	await _frames(1)
	_press(at, true)
	await _frames(2)
	_press(at, false)
	await _frames(3)


func _tap_control(c: Control) -> void:
	await _tap(c.get_global_rect().get_center())


## Viewport pixel of grid point (gx, gy) -- the inverse of `_grid_point()`.
func _grid_to_vp(gx: float, gy: float) -> Vector2:
	var ov: Control = _app.viewport.overlay
	var rect: Rect2 = ov.displayed_rect()
	var g: Vector2i = _bridge.grid_size()
	var local := rect.position + Vector2(gx / g.x, gy / g.y) * rect.size
	return ov.get_global_transform_with_canvas() * local


func _dock_nodes() -> Array:
	return _walk(_app.right_dock_body, [])


## `DccWidgets.toggle()` draws its caption as a Label in the row, not as the
## CheckBox's own text: find the box whose row carries `caption`.
func _dock_toggle(caption: String) -> CheckBox:
	for n in _dock_nodes():
		if n is CheckBox:
			var row: Node = (n as Node).get_parent()
			while row != null and not (row is HBoxContainer):
				row = row.get_parent()
			if row == null:
				continue
			for m in _walk(row, []):
				if m is Label and (m as Label).text == caption:
					return n
	return null


func _dock_spins() -> Array:
	var out := []
	for n in _dock_nodes():
		if n is SpinBox:
			out.append(n)
	return out


func _dock_labels() -> Array:
	var out := []
	for n in _dock_nodes():
		if n is Label:
			out.append((n as Label).text)
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_vp = get_viewport()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	if not _bridge.world_gen.has_method("conflict_add"):
		_p("ABORT: this binary has no conflict_add #[func]")
		get_tree().quit(2)
		return

	_bridge.generate({"seed": 5521, "width_km": 1600.0, "grid_w": 320, "grid_h": 240,
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
	## The settlement nearest the map centre, so every click lands well inside
	## the plate's neatline.
	var g0: Vector2i = _bridge.grid_size()
	var s0: Dictionary = places[0]
	for pl: Dictionary in places:
		if Vector2(float(pl["x"]), float(pl["y"])).distance_to(Vector2(g0) * 0.5) 				< Vector2(float(s0["x"]), float(s0["y"])).distance_to(Vector2(g0) * 0.5):
			s0 = pl
	var tid := int(s0.get("tid", 0))
	var sname := String(s0.get("name", ""))
	var f1 := int(s0.get("faction", 1))
	var f2 := -1
	for f: Dictionary in factions:
		if int(f.get("id", 0)) > 0 and int(f.get("id", 0)) != f1:
			f2 = int(f.get("id", 0))
			break
	var fname := {}
	for f: Dictionary in factions:
		fname[int(f.get("id", 0))] = String(f.get("name", ""))
	_p("settlement tid=%d '%s' at (%d,%d), sides %d '%s' vs %d '%s'" % [tid, sname,
		int(s0["x"]), int(s0["y"]), f1, fname.get(f1, ""), f2, fname.get(f2, "")])

	# -- 1. Arm the tool by clicking its real palette button.
	var tool_btn: Button = null
	for n in _walk(_app, []):
		if n is Button and n.has_meta(DccWidgets.TOOL_ID_META) \
				and String(n.get_meta(DccWidgets.TOOL_ID_META)) == "conflict" and (n as Button).is_visible_in_tree():
			tool_btn = n
			break
	_check("the Conflict tool button is on screen", tool_btn != null)
	if tool_btn == null:
		get_tree().quit(1)
		return
	await _tap_control(tool_btn)
	_check("clicking it arms the conflict tool", _app.armed_tool == "conflict", "armed=%s" % _app.armed_tool)

	# -- 2. Three real map clicks near the settlement: a front line.
	var cx := float(s0["x"])
	var cy := float(s0["y"])
	var clicks := [Vector2(cx - 12, cy + 6), Vector2(cx, cy + 9), Vector2(cx + 12, cy + 6)]
	for c: Vector2 in clicks:
		var at := _grid_to_vp(c.x, c.y)
		_p("  click grid %s -> viewport %s (hovered %s)" % [str(c), str(at), str(_vp.gui_get_hovered_control())])
		await _tap(at)
	var preview: PackedVector2Array = _app.viewport.tool_overlay.path_preview
	_check("three clicks build a three-point draft preview", preview.size() == 3, "preview=%d" % preview.size())

	# -- 3. The real ✓ Commit in the options bar.
	var commit: Button = null
	for n in _walk(_app, []):
		if n is Button and (n as Button).text == "✓ Commit" and (n as Button).is_visible_in_tree():
			commit = n
	_check("✓ Commit is enabled", commit != null and not commit.disabled)
	await _tap_control(commit)
	await _frames(4)
	var list: Array = _bridge.conflict_list()
	_check("commit created one conflict", list.size() == 1, "n=%d" % list.size())
	if list.is_empty():
		get_tree().quit(1)
		return
	var c0: Dictionary = list[0]
	var id := int(c0["id"])
	var pts: PackedVector2Array = c0["points"]
	_check("it is a front with the three clicked points", String(c0["kind"]) == "front" and pts.size() == 3)
	var close := pts.size() == 3
	for i in mini(pts.size(), 3):
		close = close and pts[i].distance_to(clicks[i]) < 1.5
	_check("its points are where the clicks landed (within 1.5 cells)", close, str(pts))
	_check("it starts at the cursor's year", int(c0["start_year"]) == _bridge.get_civ_year())
	_check("the right dock shows the Conflict context", _app.right_dock_ctrl._context == "conflict")

	# -- 4. Name it through the dock's LineEdit.
	var name_edit: LineEdit = null
	for n in _dock_nodes():
		if n is LineEdit and (n as LineEdit).placeholder_text == "(unnamed)":
			name_edit = n
	_check("the Name field is in the dock", name_edit != null)
	if name_edit != null:
		name_edit.text = "War of the Salt Marches"
		name_edit.text_submitted.emit(name_edit.text)
		await _frames(4)
	_check("the typed name is stored", String(_bridge.conflict_get(id).get("name", "")) == "War of the Salt Marches")

	# -- 5. Attach to the settlement through the dock's picker.
	var picker: OptionButton = null
	var want := "Settlement · %s" % sname
	var at_i := -1
	for n in _dock_nodes():
		if n is OptionButton:
			for i in (n as OptionButton).item_count:
				if (n as OptionButton).get_item_text(i) == want:
					picker = n
					at_i = i
	_check("the Attached-to picker lists the settlement", picker != null)
	if picker != null:
		picker.select(at_i)
		picker.item_selected.emit(at_i)
		await _frames(4)
	var cg: Dictionary = _bridge.conflict_get(id)
	_check("anchored to that settlement by tid", String(cg.get("anchor_kind", "")) == "settlement"
		and int(cg.get("anchor_tid", 0)) == tid and bool(cg.get("anchor_resolved", false)),
		str([cg.get("anchor_kind"), cg.get("anchor_tid"), cg.get("anchor_resolved")]))
	_check("attaching did not move the drawing", (cg["points"] as PackedVector2Array) == pts)
	_check("what-happened-here finds it by tid", Array(_bridge.conflicts_attached_to("settlement", tid)) == [id])
	_check("and not under another tid", _bridge.conflicts_attached_to("settlement", tid + 100000).is_empty())

	# -- 6. Tick two sides.
	for fid in [f1, f2]:
		var box: CheckBox = _dock_toggle(String(fname.get(fid, "")))
		_check("side checkbox for faction %d is there" % fid, box != null)
		if box != null:
			box.button_pressed = true
			await _frames(4)
	var sides := Array(_bridge.conflict_get(id).get("sides", PackedInt32Array()))
	_check("both sides are stored, in the order ticked", sides == [f1, f2], str(sides))

	# -- 7. The numbers: real, and the same as CIVIL ▸ Military's.
	var mp: Array = _bridge.conflict_sides_manpower(id)
	var summary: Dictionary = _bridge.civ_military_summary()
	var by_f := {}
	for r: Dictionary in summary.get("factions", []):
		by_f[int(r["faction"])] = r.get("manpower", {})
	_check("two manpower rows", mp.size() == 2)
	var labels := _dock_labels()
	for r: Dictionary in mp:
		var f := int(r["faction"])
		var ref: Dictionary = by_f.get(f, {})
		var ok := r.has("standing_army") and not ref.is_empty() \
			and is_equal_approx(float(r["standing_army"]), float(ref["standing_army"])) \
			and is_equal_approx(float(r["field_army"]), float(ref["field_army"])) \
			and is_equal_approx(float(r["emergency_mobilization"]), float(ref["emergency_mobilization"])) \
			and is_equal_approx(float(r["field_duration_days"]), float(ref["field_duration_days"]))
		_check("faction %d's manpower equals CIVIL ▸ Military's" % f, ok,
			"standing %s vs %s" % [str(r.get("standing_army")), str(ref.get("standing_army"))])
		_check("faction %d's standing army is a real, positive headcount" % f, float(r.get("standing_army", 0.0)) > 0.0)
		var shown: String = _app.right_dock_ctrl._thousands(float(r.get("standing_army", 0.0)))
		_check("faction %d's standing army is drawn in the dock (%s)" % [f, shown], labels.has(shown))
		_p("  faction %d: standing %.0f  field %.0f  levy %.0f  field lasts %.0f d  era %s" % [f,
			float(r.get("standing_army", 0)), float(r.get("field_army", 0)),
			float(r.get("emergency_mobilization", 0)), float(r.get("field_duration_days", 0)), String(r.get("era", ""))])

	# -- 8. Drawn: flip the layer and diff the framebuffer (positive control).
	var ov: Control = _app.viewport.overlay
	_check("the overlay draws it this year", ov.drawn_conflict_ids() == [id])
	await RenderingServer.frame_post_draw
	await _frames(3)
	await RenderingServer.frame_post_draw
	var with_img := _vp.get_texture().get_image()
	var shot := OS.get_environment("SP4_SHOT")
	if shot != "":
		with_img.save_png(shot)
	var keep: Array = _bridge.conflict_list()
	ov.set_conflicts([])
	await _frames(3)
	await RenderingServer.frame_post_draw
	var without_img := _vp.get_texture().get_image()
	ov.set_conflicts(keep)
	var box_c := _grid_to_vp(cx, cy + 8)
	var moved := 0
	for y in range(int(box_c.y) - 60, int(box_c.y) + 60):
		for x in range(int(box_c.x) - 120, int(box_c.x) + 120):
			if x < 0 or y < 0 or x >= with_img.get_width() or y >= with_img.get_height():
				continue
			if not with_img.get_pixel(x, y).is_equal_approx(without_img.get_pixel(x, y)):
				moved += 1
	_check("removing the layer changes pixels around the front (it is really drawn)", moved > 50, "moved=%d" % moved)
	await _frames(3)

	# -- 9. The year cursor: set a range through the dock, then scrub.
	var y0: int = _bridge.get_civ_year()
	var ongoing_box: CheckBox = _dock_toggle("Ongoing")
	_check("the Ongoing toggle is in the dock", ongoing_box != null)
	if ongoing_box != null:
		ongoing_box.button_pressed = false   ## gives it an end year = start
		await _frames(4)
	var spins := []
	for n in _dock_nodes():
		if n is SpinBox:
			spins.append(n)
	_check("start and end year spinboxes are in the dock", spins.size() >= 2, "spins=%d" % spins.size())
	if spins.size() >= 2:
		## End first: the Ongoing toggle gave it end == start, so a later start
		## would be refused -- which is itself checked next.
		(spins[1] as SpinBox).value = y0 + 20
		await _frames(4)
		spins = _dock_spins()
		(spins[0] as SpinBox).value = y0 + 30
		await _frames(4)
		_check("a start after the end is refused (kept at %d)" % y0,
			int(_bridge.conflict_get(id).get("start_year", -1)) == y0)
		spins = _dock_spins()
		(spins[0] as SpinBox).value = y0 + 10
		await _frames(4)
	cg = _bridge.conflict_get(id)
	_check("range stored as [y0+10, y0+20]", int(cg.get("start_year", 0)) == y0 + 10 and int(cg.get("end_year", -1)) == y0 + 20,
		"%s..%s" % [str(cg.get("start_year")), str(cg.get("end_year"))])
	_app.tl_set_year(y0 + 9)
	await _frames(3)
	_check("hidden the year before it starts", ov.drawn_conflict_ids().is_empty())
	_app.tl_set_year(y0 + 10)
	await _frames(3)
	_check("shown in its first year", ov.drawn_conflict_ids() == [id])
	_app.tl_set_year(y0 + 20)
	await _frames(3)
	_check("shown in its last year", ov.drawn_conflict_ids() == [id])
	_app.tl_set_year(y0 + 21)
	await _frames(3)
	_check("hidden the year after it ends", ov.drawn_conflict_ids().is_empty())
	_app.tl_set_year(y0 + 15)
	await _frames(3)

	# -- 10. Save -> open.
	var before: Dictionary = _bridge.conflict_get(id)
	var path := OS.get_user_data_dir().path_join("_sp4conflict_probe.ctl")
	_check("the project saved", bool(_bridge.save_project(path)))
	_app._load_project(path)
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(10)
	var after: Dictionary = _bridge.conflict_get(id)
	for k in ["name", "kind", "start_year", "end_year", "sides", "points", "anchor_kind", "anchor_tid", "anchor_resolved"]:
		_check("reopened '%s' matches" % k, str(after.get(k)) == str(before.get(k)),
			"%s vs %s" % [str(after.get(k)), str(before.get(k))])
	_check("reopened project draws it at the saved cursor (%d, inside 10..20)" % _bridge.get_civ_year(),
		ov.drawn_conflict_ids() == [id])

	# -- 11. A regenerate empties the store and the map (the disclosed rule).
	_bridge.generate({"seed": 77, "width_km": 800.0, "grid_w": 160, "grid_h": 120,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(10)
	_check("a new world has no conflicts", _bridge.conflict_list().is_empty() and ov._conflicts.is_empty())

	_p("RESULT %s (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
