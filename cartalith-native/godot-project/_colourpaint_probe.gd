extends Node
## Protects: CP-2's colour-paint tool (`COLOUR_PAINT_SCOPE.md` CP-2, Ruling CN)
## and CP-1M's lava material, through the real shell on a generated world:
##
##   1. the section is drawn: CARTO ▸ Colours holds "Paint on the map" and the
##      Material picker (found by walking the tree from the CARTO workspace);
##   2. pressing "Paint on the map" arms `colour_paint`;
##   3. a stroke through the app's own map dispatch (`_on_map_clicked` /
##      `_on_map_dragged` / `_on_map_released`, the handlers the viewport's
##      signals reach) adds one mark and MOVES the map raster
##      (`bridge.color_texture()`) inside the stroke's box while a far box does
##      not move (positive and negative control);
##   4. a Lava stroke adds a second mark whose pixels include glowing ones (red
##      channel above green above blue, red >= 200) that no flat red stroke drew;
##   5. undo -> 1 mark, redo -> 2; hide -> the raster equals the unpainted one
##      everywhere; show; clear -> 0 marks and the unpainted raster; undo -> 2;
##   6. while a regenerate holds the engine, every tool action returns without
##      calling it (no panic, the marks unchanged after it finishes) and the
##      marks survive the regenerate (kilometres, §5);
##   7. CP-3: saved with the project, cleared, reopened -- both marks come back.
##   8. Ruling CS (`_layers_leg`): Colour is the default blend and a material
##      loads its own (Lava keeps Replace, the Blend row follows); the reopened
##      project is one "Paint 1" layer; New layer / a stroke into the active
##      layer / hide (raster exactly without it) / show / move down (order
##      changes the picture, undo restores it exactly) / lock (refuses stroke,
##      clear, delete; the list disables them) / rename / save + reopen (names,
##      hidden flag, lock) / delete + undo; on touch every layer-row and
##      layer-action button >= 44 px on both axes, measured with the CARTO
##      sheet opened on the phone (asserted non-vacuous).
##
## Pixel readback, so WINDOWED, never `--headless`:
##   godot --path <copy> res://_colourpaint_probe.tscn -- --tag desktop --out DIR
##   godot --path <copy> res://_colourpaint_probe.tscn -- --force-touch --vp 1080x2340 --tag phone --out DIR
##   godot --path <copy> res://_colourpaint_probe.tscn -- --force-touch --vp 2560x1600 --tag tablet --out DIR
## Flags read here: `--vp WxH` (SubViewport size, default 1600x900), `--tag NAME`
## (default `cpaint`), `--out DIR` (required: screenshots), `--force-touch` (read by
## `dcc_shell.gd`, accepted here). Any other `--flag` aborts.
## Prints `[<tag>]   OK|FAIL <what>` per check and `[<tag>] RESULT <tag> checks=<n> fail=<n>`.
## Exit 0 = every check held; 1 = a check failed; 2 = could not run.
##
## What it does NOT prove: a phone run proves the tool renders and can be
## operated through the app's dispatch, not that a finger can FIND it on glass
## (`MISTAKES.md`: `--force-touch` on desktop is not the phone).

var app: Node
var bridge: Node
var _vp: SubViewport
var _tag := "cpaint"
var _out := ""
var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag", "--out"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _finish() -> void:
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _cannot(why: String) -> void:
	printerr("PROBE-CANNOT-RUN: " + why)
	_log("CANNOT-RUN " + why)
	get_tree().quit(2)

func _cartography() -> Node:
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			return ws
	return null

func _all(root: Node, out: Array) -> void:
	for c in root.get_children(true):
		out.append(c)
		_all(c, out)

func _button(root: Node, text: String) -> Button:
	var all: Array = []
	_all(root, all)
	for n in all:
		## Case-insensitive: the phone density upper-cases action labels.
		if n is Button and String((n as Button).text).to_lower() == text.to_lower():
			return n
	return null

## The map raster as an Image, from the engine (never the screen: the probe
## asserts what the engine drew).
func _raster() -> Image:
	var t: Texture2D = bridge.color_texture()
	if t == null:
		return null
	var img := t.get_image()
	if img != null and img.is_compressed():
		img.decompress()
	return img

## How many pixels of box `r` (grid cells) differ by more than 6 levels on any
## channel between two rasters.
func _moved(a: Image, b: Image, r: Rect2i) -> int:
	var n := 0
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absi(p.r8 - q.r8) > 6 or absi(p.g8 - q.g8) > 6 or absi(p.b8 - q.b8) > 6:
				n += 1
	return n

## Pixels of box `r` that glow: red >= 200, red > green > blue + 20.
func _glowing(a: Image, r: Rect2i) -> int:
	var n := 0
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			var p := a.get_pixel(x, y)
			if p.r8 >= 200 and p.r8 > p.g8 and p.g8 > p.b8 + 20:
				n += 1
	return n

## A horizontal stroke through the app's own map dispatch, from `a` to `b` (grid cells).
func _stroke(a: Vector2, b: Vector2) -> void:
	app._on_map_clicked(a.x, a.y)
	for i in range(1, 25):
		var p := a.lerp(b, i / 24.0)
		app._on_map_dragged(p.x, p.y)
	app._on_map_released(b.x, b.y, true)
	await _frames(4)

func _shot(name: String) -> void:
	var img := _vp.get_texture().get_image()
	if img != null:
		img.save_png(_out.path_join("%s_%s.png" % [_tag, name]))

func _ready() -> void:
	_tag = _arg("--tag", "cpaint")
	_out = _arg("--out", "")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_cannot("run WINDOWED -- ImageTexture readback is a no-op under --headless")
		return
	if _out == "":
		_cannot("--out DIR is required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func() -> void: _cannot("watchdog"))
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
	if parts.size() != 2:
		_cannot("--vp wants WxH")
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_log("viewport %dx%d  phone=%s" % [_vp.size.x, _vp.size.y, app.is_phone()])
	if not bridge.colour_paint_api:
		_cannot("the loaded extension has no colour-paint binding")
		return
	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 512, "grid_h": 328,
		"sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	if not bridge.has_world:
		_cannot("no generated world")
		return
	var carto = _cartography()
	var tool = carto.colour_paint if carto != null else null
	_check(tool != null, "CARTO owns a colour-paint tool")
	if tool == null:
		_finish()
		return
	## 1. The section.
	var arm := _button(carto, "Paint on the map")
	_check(arm != null, "CARTO ▸ Colours draws 'Paint on the map'")
	if arm == null:
		var seen: Array = []
		var all: Array = []
		_all(carto, all)
		for n in all:
			if n is Button and String((n as Button).text).to_lower().contains("paint"):
				seen.append((n as Button).text)
			if n is Label and String((n as Label).text).to_lower().contains("paint colour"):
				seen.append("label:" + (n as Label).text)
		_log("  buttons/labels mentioning paint under CARTO: %s" % str(seen))
	var info: Dictionary = bridge.colour_paint_info()
	_check(info.has("km_per_cell") and float(info["km_per_cell"]) > 0.0, "the world has a scale (%s)" % str(info))
	_check(int(info.get("marks", -1)) == 0, "a fresh world has no marks")
	## 2. Arm by the button.
	if arm != null:
		arm.pressed.emit()
	await _frames(3)
	_check(app.armed_tool == "colour_paint", "pressing it arms colour_paint (%s)" % app.armed_tool)
	app.select_domain_category("cartography", "Colours")
	await _frames(6)
	## Scroll the dock to the section so the screenshot shows it (a viewing
	## aid for the human reading the capture; nothing is asserted on it).
	if arm != null:
		var sc: Node = arm.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(arm)
			## Then the section's last row, so the capture shows the whole section.
			var last = tool.get("_status")
			if last is Control and is_instance_valid(last):
				await _frames(2)
				(sc as ScrollContainer).ensure_control_visible(last)
	await _frames(6)
	## On the phone the docks live behind the bottom bar's sheets; reaching
	## CARTO ▸ Colours there is CP-4's (phone forms) and is NOT measured here --
	## said in the output rather than reported as a pass.
	if app.is_phone():
		_log("  NOT-MEASURED phone: whether a thumb can FIND CARTO ▸ Colours ▸ Paint colour (CP-4)")
	else:
		_check(arm != null and arm.is_visible_in_tree(), "the section's button is in the visible tree after opening Colours")
	_shot("0_section")
	## Touch targets: on a touch density every swatch and the picker are at least
	## 44 px on BOTH axes (laid size), and the swatch row is no wider than the
	## viewport (it wraps rather than widening the dock).
	var row = tool.get("_colour_row")
	if row is Control and is_instance_valid(row):
		var smallest := Vector2(INF, INF)
		for c in (row as Control).get_children():
			if c is Control:
				smallest = Vector2(minf(smallest.x, (c as Control).size.x), minf(smallest.y, (c as Control).size.y))
		_log("  swatch row: %d controls, smallest %s, row width %.0f" % [(row as Control).get_child_count(), smallest, (row as Control).size.x])
		if DccTheme.is_touch():
			_check(smallest.x >= 44.0 and smallest.y >= 44.0, "touch: every swatch and the picker are >= 44 px on both axes (%s)" % smallest)
		_check((row as Control).size.x <= float(_vp.size.x), "the swatch row fits the viewport width")
	## 3. A flat stroke. Land cells around the centre, found by the input height.
	var g: Vector2i = bridge.grid_size()
	var y0 := int(g.y * 0.5)
	var before := _raster()
	tool.material = "colour"
	tool.rgb = Color8(200, 30, 18)
	tool.radius_km = 6.0
	tool.hardness = 1.0
	tool.land_only = false
	var a := Vector2(g.x * 0.30, y0)
	var b := Vector2(g.x * 0.45, y0)
	await _stroke(a, b)
	var after1 := _raster()
	info = bridge.colour_paint_info()
	_check(int(info.get("marks", -1)) == 1, "the stroke added one mark (%d)" % int(info.get("marks", -1)))
	var box1 := Rect2i(int(a.x), y0 - 3, int(b.x - a.x), 6)
	var far := Rect2i(int(g.x * 0.8), int(g.y * 0.15), 20, 20)
	var m1 := _moved(before, after1, box1)
	_check(m1 > box1.get_area() / 2, "the stroke moved the map raster under it (%d of %d px)" % [m1, box1.get_area()])
	_check(_moved(before, after1, far) == 0, "and nothing far from it")
	await _frames(10)
	_shot("1_flat_stroke")
	## 4. A lava stroke below it.
	var mats: Array = tool.MATERIALS
	for i in mats.size():
		if String(mats[i]["key"]) == "lava":
			tool._set_material(i)
	tool.radius_km = 12.0
	tool.heat = 0.95
	var la := Vector2(g.x * 0.30, y0 + 25)
	var lb := Vector2(g.x * 0.45, y0 + 25)
	await _stroke(la, lb)
	var after2 := _raster()
	info = bridge.colour_paint_info()
	_check(int(info.get("marks", -1)) == 2 and int(info.get("emissive", -1)) == 1, "the lava stroke is the second, emissive mark (%s)" % str(info))
	var box2 := Rect2i(int(la.x), y0 + 25 - 4, int(lb.x - la.x), 8)
	var gl := _glowing(after2, box2)
	var gl_flat := _glowing(after1, box1)
	_check(gl > 10, "the lava stroke glows (%d glowing px; the flat red stroke had %d)" % [gl, gl_flat])
	await _frames(10)
	_shot("2_lava_stroke")
	## 5. Undo / redo / hide / clear.
	tool.undo()
	await _frames(2)
	_check(int(bridge.colour_paint_info().get("marks", -1)) == 1, "undo removes the lava stroke")
	tool.redo()
	await _frames(2)
	_check(int(bridge.colour_paint_info().get("marks", -1)) == 2, "redo puts it back")
	tool.set_hidden(true)
	await _frames(2)
	var hidden := _raster()
	_check(_moved(before, hidden, Rect2i(0, 0, g.x, g.y)) == 0, "hidden: the raster is the unpainted one")
	tool.set_hidden(false)
	tool.clear()
	await _frames(2)
	_check(int(bridge.colour_paint_info().get("marks", -1)) == 0, "clear removes every mark")
	_check(_moved(before, _raster(), Rect2i(0, 0, g.x, g.y)) == 0, "cleared: the raster is the unpainted one")
	tool.undo()
	await _frames(2)
	_check(int(bridge.colour_paint_info().get("marks", -1)) == 2, "undo brings both marks back")
	## 6. While generating: nothing reaches the engine, nothing panics.
	bridge.generate({"seed": 4243, "width_km": 800.0, "grid_w": 512, "grid_h": 328,
		"sea_level": 0.42, "villages": true})
	await _frames(2)
	var busy: bool = bridge.generating
	var r1: bool = tool.commit_line([Vector2(10, 10), Vector2(20, 20)])
	tool.undo()
	tool.clear()
	tool.set_hidden(true)
	app._on_map_clicked(a.x, a.y)
	app._on_map_dragged(b.x, b.y)
	app._on_map_released(b.x, b.y, true)
	_check(busy and not r1 and bridge.colour_paint_info().is_empty(), "while generating every action is refused without touching the engine (busy=%s)" % busy)
	await bridge.generation_finished
	await _frames(20)
	info = bridge.colour_paint_info()
	_check(int(info.get("marks", -1)) == 2 and bool(info.get("visible", false)), "the marks survive a regenerate, visible (%s)" % str(info))
	_shot("3_after_regenerate")
	## 7. CP-3: the paint is saved with the project and comes back on open.
	var path := OS.get_user_data_dir().path_join("cp_probe_save.zip")
	var saved: bool = bridge.save_project(path)
	tool.clear()
	_check(int(bridge.colour_paint_info().get("marks", -1)) == 0, "premise: cleared before reopening")
	var opened: bool = bridge.load_save(path)
	await _frames(20)
	info = bridge.colour_paint_info()
	_check(saved and opened and int(info.get("marks", -1)) == 2 and int(info.get("emissive", -1)) == 1,
		"saved and reopened: both marks and the lava material come back (saved=%s opened=%s %s)" % [saved, opened, str(info)])
	DirAccess.remove_absolute(path)
	if bridge.colour_paint_layers_api:
		await _layers_leg(tool, carto, a, b, y0, g)
	else:
		_check(false, "the loaded extension has Ruling CS's paint-layer binding (colour_paint_layers)")
	_finish()

## The touch floor's smallest laid size over `ctrls` (every one a Control).
func _smallest(ctrls: Array) -> Vector2:
	var s := Vector2(INF, INF)
	for c in ctrls:
		if c is Control and (c as Control).is_visible_in_tree():
			s = Vector2(minf(s.x, (c as Control).size.x), minf(s.y, (c as Control).size.y))
	return s

## The buttons of the layers list's rows, and the rows' count.
func _row_buttons(tool) -> Array:
	var out: Array = []
	var box = tool.get("_layers_box")
	if box is Control:
		for r in (box as Control).get_children():
			if r.has_meta("paint_layer"):
				for c in r.get_children():
					if c is Button:
						out.append(c)
	return out

func _rows(tool) -> int:
	var n := 0
	var box = tool.get("_layers_box")
	if box is Control:
		for r in (box as Control).get_children():
			if r.has_meta("paint_layer"):
				n += 1
	return n

## 8. Ruling CS's paint layers, through the tool's own verbs and the drawn list:
## the default blend and the per-material defaults; a reopened CP-3 project is one
## "Paint 1" layer; a new layer is drawn and active and takes the next stroke;
## hiding it restores the raster under it exactly; moving it below changes the
## picture where the two layers overlap and undo restores it exactly; a lock
## refuses a stroke, a clear and a delete; rename; delete; undo covers layer
## edits; save and reopen keep both layers, the hidden flag and the name; and on
## a touch density every layer-row and layer-action button is >= 44 px on both
## axes and the list fits the viewport width.
func _layers_leg(tool, carto, a: Vector2, b: Vector2, y0: int, g: Vector2i) -> void:
	_check(String(tool.BLENDS[0]["key"]) == "colour", "Ruling CS: Colour is the first blend listed")
	var mats: Array = tool.MATERIALS
	var lava_i := -1
	for i in mats.size():
		if String(mats[i]["key"]) == "lava":
			lava_i = i
	tool._set_material(lava_i)
	var ob = tool.get("_blend_ob")
	_check(tool.blend == "replace" and ob is OptionButton and (ob as OptionButton).selected == tool._blend_index(),
		"picking Lava loads its shipped Replace and the Blend row follows (%s, row %d)" % [tool.blend, (ob as OptionButton).selected if ob is OptionButton else -1])
	tool._set_material(0)
	_check(tool.blend == "colour" and (ob as OptionButton).selected == 0, "picking Colour loads the default Colour blend (%s)" % tool.blend)
	var ls: Array = tool.layers()
	_check(ls.size() == 1 and String((ls[0] as Dictionary).get("name", "")) == "Paint 1" and int((ls[0] as Dictionary).get("marks", -1)) == 2,
		"the reopened project is one 'Paint 1' layer with both marks (%s)" % str(ls))
	_check(_rows(tool) == 1, "the list draws one row (%d)" % _rows(tool))
	## A new layer, active, taking a blue Replace stroke over the red one.
	var r0 := _raster()
	_check(tool.add_layer(""), "New layer")
	ls = tool.layers()
	_check(ls.size() == 2 and String((ls[1] as Dictionary).get("name", "")) == "Paint 2" and tool.active_layer == 1 and _rows(tool) == 2,
		"it is 'Paint 2', on top, active, and drawn (%s, active %d, rows %d)" % [str(ls), tool.active_layer, _rows(tool)])
	tool.blend = "replace"
	tool.rgb = Color8(30, 60, 220)
	tool.radius_km = 6.0
	tool.hardness = 1.0
	_check(tool.commit_line([Vector2(a.x, y0) * float(bridge.colour_paint_info()["km_per_cell"]), Vector2(b.x, y0) * float(bridge.colour_paint_info()["km_per_cell"])]), "a stroke into the active layer")
	await _frames(2)
	ls = tool.layers()
	_check(int((ls[1] as Dictionary).get("marks", -1)) == 1 and int((ls[0] as Dictionary).get("marks", -1)) == 2, "it went into 'Paint 2' only (%s)" % str(ls))
	var r1 := _raster()
	var box := Rect2i(int(a.x) + 2, y0 - 2, int(b.x - a.x) - 4, 4)
	var whole := Rect2i(0, 0, g.x, g.y)
	_check(_moved(r0, r1, box) > box.get_area() / 2, "the blue stroke moved the raster over the red one (%d of %d)" % [_moved(r0, r1, box), box.get_area()])
	var c := r1.get_pixel(box.position.x + box.size.x / 2, y0)
	_check(c.b8 > c.r8, "on top, blue covers red (%s)" % str(c))
	await _frames(6)
	## On the phone the CARTO dock is the left sheet, closed unless opened (the
	## call `phone_menu.gd::_open_left_sheet` makes); open it so the list is laid
	## out and visible, or the touch-floor check below would measure nothing.
	if app.is_phone():
		app.select_domain_category("cartography", "Colours")
		app._set_sheet_open("left", true)
		await _frames(8)
	var sc: Node = tool.get("_layers_box")
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(tool.get("_layers_box"))
		await _frames(4)
	_shot("4_layers")
	## Hide: exactly the raster without it; show: exactly the raster with it.
	tool.set_layer_visible(1, false)
	await _frames(2)
	_check(_moved(r0, _raster(), whole) == 0 and int(bridge.colour_paint_info().get("drawn", -1)) == 2, "hiding 'Paint 2' draws exactly the raster without it")
	tool.set_layer_visible(1, true)
	await _frames(2)
	_check(_moved(r1, _raster(), whole) == 0, "showing it again draws exactly the raster with it")
	## Order: below the red stroke, red covers blue; undo restores exactly.
	_check(tool.move_layer(1, 0) and tool.active_layer == 0, "move down: the active layer follows the moved one (%d)" % tool.active_layer)
	await _frames(2)
	var r2 := _raster()
	var c2 := r2.get_pixel(box.position.x + box.size.x / 2, y0)
	_check(_moved(r1, r2, box) > box.get_area() / 2 and c2.r8 > c2.b8, "below, the red layer covers it (%s)" % str(c2))
	tool.undo()
	await _frames(2)
	_check(_moved(r1, _raster(), whole) == 0 and String((tool.layers()[1] as Dictionary).get("name", "")) == "Paint 2", "undo restores the order exactly")
	## Lock refuses stroke, clear and delete; rename is allowed.
	tool.select_layer(1)
	_check(tool.set_layer_locked(1, true), "lock 'Paint 2'")
	var n_before := int(bridge.colour_paint_info().get("marks", -1))
	_check(not tool.commit_line([Vector2(30, 30), Vector2(60, 30)]) and not tool.clear_layer(1) and not tool.delete_layer(1)
		and int(bridge.colour_paint_info().get("marks", -1)) == n_before, "a locked layer refuses a stroke, Clear layer and Delete layer")
	var acts: Dictionary = tool.get("_layer_actions")
	_check((acts["clear"] as Button).disabled and (acts["delete"] as Button).disabled, "and the list disables Clear layer and Delete layer for it")
	_check(tool.rename_layer(1, "Highlights") and String((tool.layers()[1] as Dictionary).get("name", "")) == "Highlights", "rename works on a locked layer")
	## Touch floor on the rows and actions.
	var rb := _row_buttons(tool)
	var acts_list: Array = acts.values()
	var sm := _smallest(rb + acts_list)
	_log("  layers: %d row buttons, %d actions, smallest %s" % [rb.size(), acts_list.size(), sm])
	## Non-vacuous: INF means no button was in the visible tree to measure.
	_check(sm.x < INF, "the layer buttons are in the visible tree, so they were measured")
	if DccTheme.is_touch():
		_check(sm.x >= 44.0 and sm.y >= 44.0, "touch: every layer-row and layer-action button is >= 44 px on both axes (%s)" % sm)
	var lb = tool.get("_layers_box")
	_check(lb is Control and (lb as Control).size.x <= float(_vp.size.x), "the layers list fits the viewport width")
	## Save with a hidden, renamed, locked layer; reopen.
	tool.set_layer_visible(1, false)
	var path := OS.get_user_data_dir().path_join("cp_probe_layers.zip")
	var saved: bool = bridge.save_project(path)
	tool.clear()
	var opened: bool = bridge.load_save(path)
	await _frames(20)
	ls = tool.layers()
	_check(saved and opened and ls.size() == 2 and String((ls[1] as Dictionary).get("name", "")) == "Highlights"
		and not bool((ls[1] as Dictionary).get("visible", true)) and bool((ls[1] as Dictionary).get("locked", false)),
		"saved and reopened: both layers, the name, the hidden flag and the lock come back (%s)" % str(ls))
	DirAccess.remove_absolute(path)
	## Delete (unlocked) and undo.
	tool.set_layer_locked(1, false)
	_check(tool.delete_layer(1) and tool.layers().size() == 1, "Delete layer removes it")
	tool.undo()
	_check(tool.layers().size() == 2, "undo brings the deleted layer back")
	await _frames(6)
	_shot("5_layers_after")
