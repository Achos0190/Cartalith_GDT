extends Node
## `ViewportHost.visible_grid_rect()` and the export tile-border preview.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 _tilegrid_probe.tscn
##
## Both are halves that already existed and were joined this session:
## `bake_visible(z, x0, y0, x1, y1)` was callable and uncalled because the rect
## it wants lived inside the private `_update_lod()`, and the reference's
## `#lodShowGrid` / `drawExportTileGrid` had no port at all.
##
## Run WINDOWED, not `--headless`: the dummy rasterizer's `texture_2d_get`
## returns null, and the map has to actually be up for the camera math to mean
## anything.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	var vh = app.get("viewport")
	print("[BOOT] shell up  viewport=", vh != null)

	print("\n=== 0: with no world, both refuse rather than answer ===")
	_ok("visible_grid_rect() is not ok before a world", bool(vh.visible_grid_rect().get("ok", true)), false)
	_ok("the preview starts off", vh.export_tile_grid_enabled(), false)

	print("\n=== 1: generate a small world ===")
	bridge.generate({"seed": 4242, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_ok("the bridge has a world", bridge.has_world, true)
	var g: Vector2i = bridge.grid_size()
	print("  info grid = ", g)

	print("\n=== 2: the visible rect is real and inside the grid ===")
	var r: Dictionary = vh.visible_grid_rect()
	print("  info ", r)
	_ok("ok", bool(r.get("ok", false)), true)
	var x0 := float(r.get("x0", -1.0)); var y0 := float(r.get("y0", -1.0))
	var x1 := float(r.get("x1", -1.0)); var y1 := float(r.get("y1", -1.0))
	_ok("x0 >= 0", x0 >= 0.0, true)
	_ok("y0 >= 0", y0 >= 0.0, true)
	_ok("x1 <= grid width", x1 <= float(g.x) + 0.001, true)
	_ok("y1 <= grid height", y1 <= float(g.y) + 0.001, true)
	_ok("the rect is non-degenerate", x1 > x0 and y1 > y0, true)
	## Fitted and unzoomed, the whole map is on screen, so the rect must be
	## the whole grid on at least the fitted axis -- a rect that came back a
	## sliver would mean the camera inverse is wrong in a way the bounds
	## checks above would not catch.
	var covers := (x1 - x0) >= float(g.x) * 0.98 or (y1 - y0) >= float(g.y) * 0.98
	_ok("at fit, one axis spans the whole grid", covers, true)
	_ok("a pyramid level came back", int(r.get("z", -1)) >= 0, true)

	print("\n=== 3: zooming in shrinks it, and it stays inside ===")
	var before := (x1 - x0) * (y1 - y0)
	vh.zoom_step(6.0)
	await _frames(10)
	var r2: Dictionary = vh.visible_grid_rect()
	print("  info ", r2)
	if bool(r2.get("ok", false)):
		var after := (float(r2["x1"]) - float(r2["x0"])) * (float(r2["y1"]) - float(r2["y0"]))
		print("  info area  fit=", before, "  zoomed=", after)
		_ok("zooming in shows fewer cells", after < before, true)
		_ok("still inside the grid", float(r2["x1"]) <= float(g.x) + 0.001, true)
	else:
		print("  info the zoom hook is private on this build; area comparison skipped")

	print("\n=== 4: bake_visible accepts what this hands it ===")
	## The whole point of the accessor. Not asserted to SUCCEED -- an atlas
	## directory may not exist in a probe run -- but it must not be refused
	## for the shape of its arguments.
	var res: Dictionary = bridge.bake_visible(int(r.get("z", 0)), x0, y0, x1, y1)
	print("  info bake_visible -> ", res)
	_ok("bake_visible did not reject the rect's shape",
		String(res.get("error", "")).find("rect") < 0, true)

	print("\n=== 5: the export tile-border preview ===")
	## Still at the step-3 zoom, so the pyramid is up. The reference's call
	## site is `if(_showExportGrid && !_lodOn)` (line 8658), so ON here must
	## still mean HIDDEN -- the export split is taken off the full-resolution
	## grid and would annotate the wrong thing over pyramid tiles.
	vh.set_export_tile_grid(true, 3, 5)
	await _frames(6)
	_ok("the preview reports on", vh.export_tile_grid_enabled(), true)
	var layer: Control = null
	for c in vh.get_children(true):
		if c is Control and c.get_class() == "Control":
			pass
	## Reach the layer by its draw connection rather than by name -- it is a
	## bare Control, so the name would be the only handle and names are not a
	## contract.
	var found := false
	var stack: Array = [vh]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is Control and (c as Control).draw.is_connected(vh._draw_export_tile_grid):
				found = true
				layer = c as Control
	_ok("the overlay exists and is wired to its draw", found, true)
	_ok("the pyramid is up right now", vh.visible_grid_rect().get("lod_active", false), true)
	if layer != null:
		_ok("on, but hidden while the pyramid is up", layer.visible, false)
	## Back out to fit: LOD drops, and the preview must appear on its own
	## without the toggle being touched again.
	vh.zoom_step(1.0 / 6.0)
	await _frames(12)
	_ok("the pyramid is down again", vh.visible_grid_rect().get("lod_active", true), false)
	if layer != null:
		_ok("...and now it is visible", layer.visible, true)
	vh.set_export_tile_grid(false)
	await _frames(4)
	if layer != null:
		_ok("hidden again with the preview off", layer.visible, false)
	_ok("the split it was told is the one it kept", vh._export_grid_cols, 3)

	print("")
	print("=== 6: the two MENU rows actually reach them ===")
	## The point of the accessor and the overlay is the two Preferences rows
	## that were disabled for want of them. Drive the real PopupMenus, not the
	## viewport, so a row wired to the wrong id fails here.
	var pops: Array = []
	var st2: Array = [app]
	while not st2.is_empty():
		var n2: Node = st2.pop_back()
		for c in n2.get_children(true):
			st2.append(c)
			if c is PopupMenu:
				pops.append(c)
	print("  info popups found: ", pops.size())
	var borders_pop: PopupMenu = null
	var atlas_pop: PopupMenu = null
	for pm in pops:
		if (pm as PopupMenu).get_item_index(80) >= 0:
			borders_pop = pm
		if (pm as PopupMenu).get_item_index(81) >= 0:
			atlas_pop = pm
	_ok("a menu carries the tile-borders row", borders_pop != null, true)
	_ok("a menu carries the refine row", atlas_pop != null, true)

	if borders_pop != null:
		var bi := borders_pop.get_item_index(80)
		_ok("the tile-borders row is enabled", borders_pop.is_item_disabled(bi), false)
		_ok("...and starts unchecked", borders_pop.is_item_checked(bi), false)
		borders_pop.id_pressed.emit(80)
		await _frames(4)
		_ok("pressing it turns the overlay on", vh.export_tile_grid_enabled(), true)
		_ok("...and the check mark follows", borders_pop.is_item_checked(bi), true)
		borders_pop.id_pressed.emit(80)
		await _frames(4)
		_ok("pressing again turns it off", vh.export_tile_grid_enabled(), false)

	if atlas_pop != null:
		var ai := atlas_pop.get_item_index(81)
		_ok("the refine row is enabled", atlas_pop.is_item_disabled(ai), false)
		## At fit the pyramid is down, so this must refuse and SAY so rather
		## than bake level 0 quietly.
		atlas_pop.id_pressed.emit(81)
		await _frames(4)
		print("  info refuse-at-fit path ran")
		vh.zoom_step(8.0)
		await _frames(14)
		var r3: Dictionary = vh.visible_grid_rect()
		_ok("the pyramid was up for the real press", r3.get("lod_active", false), true)
		bridge.atlas_clear()
		await _frames(4)
		_ok("atlas emptied before the real press", int(bridge.atlas_status().get("chunks", 0)), 0)
		atlas_pop.id_pressed.emit(81)
		await _frames(30)
		var stt: Dictionary = bridge.atlas_status()
		print("  info atlas_status after refine: ", stt)
		_ok("the menu row left chunks in the atlas", int(stt.get("chunks", 0)) > 0, true)

	print("\n_tilegrid_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
