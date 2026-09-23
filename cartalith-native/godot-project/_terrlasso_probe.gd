extends Node
## Territory lasso (owner request, 2026-09-23), end to end through real input.
## Windowed, not headless -- it reads `territory_texture().get_image()`, which
## `--headless` leaves blank:
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _terrlasso_probe.tscn
##
## Arms the tool by clicking its palette button, places five vertices with real
## mouse clicks pushed through the viewport (so `map_overlay.gd::_gui_input` is
## what turns them into grid points), drops a stray sixth with a real Backspace,
## picks the faction and presses ✓ Commit through the tool options row, then:
## every changed territory pixel must lie inside the ring, and every cell inside
## it must now carry the chosen faction's colour -- against an oracle computed
## here with `Geometry2D.is_point_in_polygon` on cell centres, not by the engine's
## own rasteriser. A Subtract ring over the same polygon must then restore the
## pre-edit image byte for byte, and Esc / Discard must leave it untouched.

var app: Node
var fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("TLP %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		fails += 1

func _mouse(pos: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.global_position = pos
	get_viewport().push_input(mv)
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = pos
		ev.global_position = pos
		get_viewport().push_input(ev)
		await _frames(2)

func _key(code: Key) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.keycode = code
		k.physical_keycode = code
		k.pressed = pressed
		get_viewport().push_input(k)
		await _frames(2)

## Grid point -> viewport pixel, the inverse of `map_overlay.gd::_grid_point`.
func _screen_of(g: Vector2) -> Vector2:
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov.displayed_rect()
	var gs: Vector2i = app.bridge.grid_size()
	var local := rect.position + Vector2(g.x / gs.x, g.y / gs.y) * rect.size
	return ov.get_global_transform_with_canvas() * local

func _find_button(root: Node, pred: Callable) -> BaseButton:
	for b in root.find_children("*", "BaseButton", true, false):
		if (b as BaseButton).is_visible_in_tree() and pred.call(b):
			return b as BaseButton
	return null

func _options_button(text: String) -> BaseButton:
	return _find_button(app.tool_options_bar, func(b): return "text" in b and String(b.text) == text)

func _option_with_prefix(prefix: String) -> OptionButton:
	for o in app.tool_options_bar.find_children("*", "OptionButton", true, false):
		var ob := o as OptionButton
		if ob.item_count > 0 and ob.get_item_text(0).begins_with(prefix):
			return ob
	return null

func _pick(ob: OptionButton, idx: int) -> void:
	ob.select(idx)
	ob.item_selected.emit(idx)
	await _frames(3)

func _image() -> Image:
	return app.bridge.territory_texture().get_image()

func _ws() -> Node:
	for w in app._workspaces:
		if w.has_method("_lasso_commit"):
			return w
	return null

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app.bridge.generate({"seed": 131313, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	if not app.bridge.has_world:
		printerr("TLP generate FAILED")
		get_tree().quit(2)
		return
	app.select_domain("civilization")
	app._escape_action(true)
	await _frames(8)
	var gs: Vector2i = app.bridge.grid_size()
	var gw := gs.x
	var gh := gs.y

	# -- Arm by clicking the palette button ------------------------------------
	var tb := _find_button(app, func(b): return String(b.get_meta(DccWidgets.TOOL_ID_META, "")) == "territory_lasso")
	_check("palette draws a Territory lasso button", tb != null)
	if tb == null:
		get_tree().quit(1)
		return
	await _mouse(tb.get_global_rect().get_center())
	_check("clicking it arms territory_lasso", app.armed_tool == "territory_lasso", app.armed_tool)
	_check("right dock shows the Territory section", String(app.right_dock_ctrl._tool_section()) == "territory")
	var ws := _ws()

	# -- Ring through real clicks ----------------------------------------------
	var want := PackedVector2Array([Vector2(gw * 0.40, gh * 0.36), Vector2(gw * 0.61, gh * 0.33),
		Vector2(gw * 0.66, gh * 0.56), Vector2(gw * 0.50, gh * 0.69), Vector2(gw * 0.35, gh * 0.55)])
	var before := _image()
	for p in want:
		await _mouse(_screen_of(p))
	await _mouse(_screen_of(Vector2(gw * 0.9, gh * 0.9)))   # a stray vertex...
	_check("six clicks recorded six vertices", ws._lasso_points.size() == 6, str(ws._lasso_points.size()))
	await _key(KEY_BACKSPACE)                                  # ...taken back with ⌫
	var ring: PackedVector2Array = ws._lasso_points
	var max_err := 0.0
	for i in mini(ring.size(), want.size()):
		max_err = maxf(max_err, ring[i].distance_to(want[i]))
	_check("⌫ dropped the stray; clicks landed where aimed", ring.size() == 5 and max_err < 0.05,
		"n=%d err=%.4f" % [ring.size(), max_err])
	var drawn: PackedVector2Array = app.viewport.tool_overlay.path_preview
	_check("the ring is drawn closed", drawn.size() == 6 and drawn[5] == drawn[0], str(drawn.size()))

	# Oracle: cells whose centre is inside the recorded ring.
	var inside := {}
	for y in gh:
		for x in gw:
			if Geometry2D.is_point_in_polygon(Vector2(x + 0.5, y + 0.5), ring):
				inside[y * gw + x] = true

	# Target: the faction least present inside the ring, so the edit is large.
	var factions: Array = app.bridge.get_factions()
	var target := -1
	var fewest := 1 << 30
	var tcol := Color()
	## Never the faction already selected, so the choice below has to move.
	var start_faction: int = ws._territory_faction
	for f in factions:
		if int(f.id) == start_faction:
			continue
		var c := Color8(int(f.color_r), int(f.color_g), int(f.color_b))
		var n := 0
		for i in inside:
			var px := before.get_pixel(i % gw, i / gw)
			if px.a > 0.0 and px.r8 == c.r8 and px.g8 == c.g8 and px.b8 == c.b8:
				n += 1
		if n < fewest:
			fewest = n
			target = int(f.id)
			tcol = c
	var owned_before := 0
	for i in inside:
		if before.get_pixel(i % gw, i / gw).a > 0.0:
			owned_before += 1
	print("TLP ring cells already owned by some faction before the edit: %d" % owned_before)
	var fopt := _option_with_prefix("%d · " % int(factions[0].id))
	_check("the options row carries the brush's faction choice", fopt != null)
	if fopt != null:
		for i in fopt.item_count:
			if fopt.get_item_text(i).begins_with("%d · " % target):
				await _pick(fopt, i)
	_check("faction choice set the shared territory faction", ws._territory_faction == target,
		"%d vs %d" % [ws._territory_faction, target])
	var claimed_before := int(app.bridge.civ_faction_territory_stats(target).get("claimed_cells", -1))

	# -- Commit through the real button ----------------------------------------
	var commit := _options_button("✓ Commit")
	_check("✓ Commit is on the options row", commit != null)
	await _mouse(commit.get_global_rect().get_center())
	await _frames(4)
	var after := _image()
	var changed_outside := 0
	var inside_wrong := 0
	var changed := 0
	for y in gh:
		for x in gw:
			var i := y * gw + x
			var a := before.get_pixel(x, y)
			var b := after.get_pixel(x, y)
			if a != b:
				changed += 1
				if not inside.has(i):
					changed_outside += 1
			if inside.has(i) and not (b.a > 0.0 and b.r8 == tcol.r8 and b.g8 == tcol.g8 and b.b8 == tcol.b8):
				inside_wrong += 1
	var claimed_after := int(app.bridge.civ_faction_territory_stats(target).get("claimed_cells", -1))
	print("TLP ring cells=%d, pre-owned by target=%d, changed=%d, faction %d claimed %d -> %d" % [
		inside.size(), fewest, changed, target, claimed_before, claimed_after])
	_check("the ring encloses real ground", inside.size() > 500, str(inside.size()))
	_check("something changed", changed > 0)
	_check("nothing changed outside the ring", changed_outside == 0, str(changed_outside))
	_check("every cell inside now carries the chosen faction", inside_wrong == 0, str(inside_wrong))
	_check("claimed count rose by exactly the cells it did not already own",
		claimed_after - claimed_before == inside.size() - fewest,
		"%d vs %d" % [claimed_after - claimed_before, inside.size() - fewest])
	_check("commit cleared the ring", ws._lasso_points.is_empty() and app.viewport.tool_overlay.path_preview.is_empty())

	# -- Esc and Discard leave the committed state alone -----------------------
	for p in want:
		await _mouse(_screen_of(p))
	await _key(KEY_ESCAPE)
	_check("Esc clears the ring and stays armed", ws._lasso_points.is_empty() and app.armed_tool == "territory_lasso")
	for p in want:
		await _mouse(_screen_of(p))
	await _mouse(_options_button("Discard").get_global_rect().get_center())
	_check("Discard clears the ring", ws._lasso_points.is_empty())
	_check("Esc/Discard changed no territory", _image().get_data() == after.get_data())

	# -- Undo: a Subtract ring over the same polygon restores the base ---------
	var mopt := _option_with_prefix("Add")
	_check("the options row carries Add/Subtract", mopt != null)
	if mopt != null:
		await _pick(mopt, 1)
	for p in want:
		await _mouse(_screen_of(p))
	await _mouse(_options_button("✓ Commit").get_global_rect().get_center())
	await _frames(4)
	_check("a Subtract ring over the same polygon restores the pre-edit territory exactly",
		_image().get_data() == before.get_data())
	_check("...and the faction's claimed count", int(app.bridge.civ_faction_territory_stats(target)
		.get("claimed_cells", -1)) == claimed_before)

	# -- Leaving the tool drops a half-drawn ring ------------------------------
	await _pick(_option_with_prefix("Add"), 0)
	await _mouse(_screen_of(want[0]))
	await _mouse(_screen_of(want[1]))
	app.arm_tool("territory")
	await _frames(3)
	_check("arming the brush drops a half-drawn ring", ws._lasso_points.is_empty()
		and app.viewport.tool_overlay.path_preview.is_empty())

	print("TLP done, %d failure(s)" % fails)
	get_tree().quit(1 if fails > 0 else 0)
