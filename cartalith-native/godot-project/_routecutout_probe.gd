extends Node
## Verifies the route-planner map-cutout feature added 2026-09-01:
## `JourneyPlannerView._route_map`'s backdrop should show a real crop of the
## engine's own `debug_texture()` raster for Water/Biome/Terrain/Wildlife,
## registered against the same world-space bounds `_fit()` uses for the route
## line, and clear back to nothing for "None".
##
## Run: godot4 --headless --path . _routecutout_probe.tscn

var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(4)

	var fails := 0
	var wg = bridge.world_gen
	var places: Array = wg.get_settlements()
	if places.size() < 2:
		print("FAIL: need two settlements to route between, got %d" % places.size())
		get_tree().quit(1)
		return
	wg.route_begin("mixed")
	wg.route_append_stop(float(places[0]["x"]), float(places[0]["y"]))
	wg.route_append_stop(float(places[1]["x"]), float(places[1]["y"]))
	var idx: int = wg.route_commit()
	if idx < 0:
		print("FAIL: route_commit refused")
		get_tree().quit(1)
		return
	print("committed route #%d" % idx)

	var jp = _app.journey_planner_view
	if jp == null:
		print("FAIL: app.journey_planner_view is null")
		get_tree().quit(1)
		return
	_app.select_domain("civilization")
	jp.open()
	await _frames(4)

	var route_map = jp._route_map
	var layer_btn = jp._route_map_layer_btn
	if route_map == null or layer_btn == null:
		print("FAIL: route map view or layer button not built")
		get_tree().quit(1)
		return
	print("route_map.pts: %d  layer_btn visible=%s" % [route_map.pts.size(), layer_btn.visible])
	if route_map.pts.size() < 2:
		print("FAIL: route map has no committed route geometry")
		fails += 1

	## The default layer id is "map" (real terrain), not "off" -- confirms it
	## actually shows something without a manual pick first.
	print("default layer id=%s  map_texture=%s" % [jp._route_map_layer_id, route_map.map_texture != null])
	if jp._route_map_layer_id != "map" or route_map.map_texture == null:
		print("FAIL: default layer should be \"map\" with a real texture already applied")
		fails += 1

	print("--- layer picker ---")
	var cases := [
		["map", 0, true],
		["water", 1, true],
		["bclass", 2, true],
		["cterrain", 3, true],
		["wildlife", 4, null],   ## availability world-dependent; log, don't hard-fail
		["off", 5, false],
	]
	for c in cases:
		var name: String = c[0]
		var id_index: int = c[1]
		var want_non_null = c[2]
		jp._on_route_map_layer_picked(id_index)
		await _frames(1)
		var tex = route_map.map_texture
		var got_non_null: bool = tex != null
		var note := ""
		if want_non_null != null and got_non_null != want_non_null:
			note = "  FAIL"
			fails += 1
		print("  %-9s -> map_texture=%s size=%s%s" % [
			name, got_non_null,
			(str(tex.get_size()) if got_non_null else "-"), note])

		## Exercise the exact crop math `_RouteMapView._draw()` runs, without
		## depending on a headless render pass actually invoking `_draw()`.
		if got_non_null:
			var b: Array = route_map._bounds()
			var minv: Vector2 = b[0]
			var maxv: Vector2 = b[1]
			var rect := Rect2(Vector2.ZERO, Vector2(600, 236))
			var fit: Callable = route_map._fit(rect, minv, maxv)
			var tsz := Vector2(tex.get_size())
			var src := Rect2(minv, maxv - minv).intersection(Rect2(Vector2.ZERO, tsz))
			if src.size.x <= 0.0 or src.size.y <= 0.0:
				print("    FAIL: %s crop rect is empty/out of texture bounds (minv=%s maxv=%s tex=%s)"
					% [name, minv, maxv, tsz])
				fails += 1
				continue
			var dest := Rect2(fit.call(src.position), fit.call(src.position + src.size) - fit.call(src.position))
			if dest.size.x <= 0.0 or dest.size.y <= 0.0:
				print("    FAIL: %s dest rect degenerate: %s" % [name, dest])
				fails += 1
			elif not rect.grow(2.0).encloses(dest):
				print("    FAIL: %s dest rect %s falls outside the panel rect %s" % [name, dest, rect])
				fails += 1
			else:
				print("    ok: src=%s -> dest=%s (panel %s)" % [src, dest, rect])

		if name == "map":
			var sprites: Array = route_map._lod_sprites
			print("    LOD sprites: %d" % sprites.size())
			if sprites.is_empty():
				print("    FAIL: \"map\" should synthesize at least one LOD tile sprite")
				fails += 1
			for sp in sprites:
				if not (sp is Sprite2D) or sp.texture == null or sp.material == null:
					print("    FAIL: LOD sprite malformed: %s" % sp)
					fails += 1
					continue
				var mat: ShaderMaterial = sp.material
				var uv0: Vector2 = mat.get_shader_parameter("base_uv0")
				var uv1: Vector2 = mat.get_shader_parameter("base_uv1")
				var base: Texture2D = mat.get_shader_parameter("base_tex")
				if base == null or base != tex:
					print("    FAIL: LOD sprite's base_tex should be the same color_texture() as map_texture")
					fails += 1
				if uv0.x < -0.001 or uv0.y < -0.001 or uv1.x > 1.001 or uv1.y > 1.001 or uv0.x >= uv1.x or uv0.y >= uv1.y:
					print("    FAIL: LOD sprite UVs out of range/degenerate: uv0=%s uv1=%s" % [uv0, uv1])
					fails += 1
				if sp.get_parent() != route_map:
					print("    FAIL: LOD sprite should be a child of route_map (the backdrop), not %s" % sp.get_parent())
					fails += 1
			print("    ok: %d LOD sprites, all UV/material-sane" % sprites.size())

			## Registration, the thing `lod_tile.gdshader`'s header says holds
			## "by construction": a tile's `base_uv*` footprint, taken back
			## into `map_texture` pixels, must be exactly the screen rect the
			## sprite occupies (`_fit` of the same numbers), and that
			## footprint must sit half a CELL off the pyramid's own sample
			## grid -- `pyramid_tile_bounds` splits `[0, gw-1]`, `pts`/`_fit`/
			## `map_texture` are all in cell-span `[0, gw]`. A tile placed on
			## the raw sample number instead slides the relief detail half a
			## cell off the colour it multiplies: ~1.6 px here, tens of px on
			## a short local route where the fit scale is much larger.
			var gsz: Vector2i = bridge.grid_size()
			var gv := Vector2(gsz.x, gsz.y)
			var rsize: Vector2 = route_map.size
			if rsize.x <= 1.0 or rsize.y <= 1.0:
				print("    skip: route_map has no laid-out size (%s), registration check needs one" % rsize)
			else:
				var rb: Array = route_map._bounds()
				var rfit: Callable = route_map._fit(Rect2(Vector2.ZERO, rsize), rb[0], rb[1])
				var bad := 0
				for sp in sprites:
					var mat: ShaderMaterial = sp.material
					var uv0: Vector2 = mat.get_shader_parameter("base_uv0")
					var uv1: Vector2 = mat.get_shader_parameter("base_uv1")
					## UV footprint -> texture pixels -> where `_fit` puts it.
					var w0: Vector2 = uv0 * gv
					var w1: Vector2 = uv1 * gv
					var want0: Vector2 = rfit.call(w0)
					var want1: Vector2 = rfit.call(w1)
					var got0: Vector2 = sp.position
					var got1: Vector2 = sp.position + sp.scale * Vector2(sp.texture.get_size())
					if got0.distance_to(want0) > 0.5 or got1.distance_to(want1) > 0.5:
						print("    FAIL: sprite screen rect %s..%s != its own base_uv footprint %s..%s"
							% [got0, got1, want0, want1])
						bad += 1
						continue
					## Half-cell offset off the sample grid: `(w0 - 0.5)` must
					## be a whole number of tile steps, where the step is the
					## tile's own footprint.
					var step: Vector2 = w1 - w0
					if step.x <= 0.0 or step.y <= 0.0:
						print("    FAIL: degenerate tile step %s" % step)
						bad += 1
						continue
					var kx: float = (w0.x - 0.5) / step.x
					var ky: float = (w0.y - 0.5) / step.y
					if absf(kx - roundf(kx)) > 1e-3 or absf(ky - roundf(ky)) > 1e-3:
						print("    FAIL: tile footprint %s is not sample-grid + half a cell (k=%s,%s step=%s)"
							% [w0, kx, ky, step])
						bad += 1
				fails += bad
				if bad == 0:
					print("    ok: all %d sprites register with base_uv and sit on sample-grid + 0.5" % sprites.size())
			## Draw order is TREE ORDER now, not z_index (a negative z_index
			## pushed sprites behind the whole panel's own background --
			## found live). `_route_line` must be route_map's own NEXT
			## SIBLING under the shared wrap for the line to draw on top of
			## both the flat crop and the LOD sprites.
			var wrap: Node = route_map.get_parent()
			var line = jp._route_line
			if line == null or line.get_parent() != wrap:
				print("    FAIL: _route_line should be a sibling of route_map under the same wrap")
				fails += 1
			elif line.get_index() <= route_map.get_index():
				print("    FAIL: _route_line (index %d) must come after route_map (index %d) in the wrap" \
					% [line.get_index(), route_map.get_index()])
				fails += 1
			else:
				print("    ok: _route_line is route_map's sibling, added after it (index %d > %d)" \
					% [line.get_index(), route_map.get_index()])
		else:
			if not route_map._lod_sprites.is_empty():
				print("    FAIL: %s should carry no leftover LOD sprites from a previous \"map\" pick" % name)
				fails += 1

	## --- The popup itself. Nothing above ever opened it: the picks went
	## straight to `_on_route_map_layer_picked`, so `String(...)` on
	## `debug_layers()`'s dictionaries -- a real `Nonexistent 'String'
	## constructor` class of failure in Godot 4, where `String(x)` only has
	## String/StringName/NodePath overloads -- had no coverage at all.
	print("--- popup build ---")
	jp._rebuild_route_map_layer_popup()
	var popup = jp._route_map_layer_popup
	print("  popup items: %d" % popup.item_count)
	if popup.item_count != 6:
		print("  FAIL: expected 6 rows, got %d" % popup.item_count)
		fails += 1
	var checked := -1
	for i in popup.item_count:
		if popup.is_item_checked(i):
			checked = i
	if checked != 5:   ## last pick above was "off", index 5
		print("  FAIL: checked row should track _route_map_layer_id (\"off\" = 5), got %d" % checked)
		fails += 1
	else:
		print("  ok: popup rebuilt, radio row follows _route_map_layer_id")

	## --- The layer button has to be ON the panel. `set_anchors_preset` then
	## a raw `position = Vector2(-26, 6)` is only right while the parent is
	## still zero-sized at build time; if it ever ran after layout the button
	## would sit 26 px off the LEFT edge instead of in from the right.
	var wrap: Control = route_map.get_parent()
	var wr: Rect2 = wrap.get_global_rect()
	var br: Rect2 = layer_btn.get_global_rect()
	if wr.size.x <= 1.0:
		print("  skip: wrap has no laid-out size, button placement not checkable")
	elif not wr.grow(1.0).encloses(br):
		print("  FAIL: layer button %s is outside the map panel %s" % [br, wr])
		fails += 1
	else:
		print("  ok: layer button %s sits inside the map panel %s" % [br, wr])

	## --- Repeated picks must not accumulate children. `_clear_lod_sprites`
	## `queue_free()`s, which is deferred, so this only holds if the array and
	## the real child list agree a frame later.
	print("--- repeat switches ---")
	for i in 3:
		jp._on_route_map_layer_picked(1)   ## water
		await _frames(1)
		jp._on_route_map_layer_picked(0)   ## map
		await _frames(1)
	await _frames(2)
	print("  after 3 water/map cycles: children=%d sprites=%d"
		% [route_map.get_child_count(), route_map._lod_sprites.size()])
	if route_map.get_child_count() != route_map._lod_sprites.size():
		print("  FAIL: %d stale children left behind by repeated layer switches"
			% (route_map.get_child_count() - route_map._lod_sprites.size()))
		fails += 1
	else:
		print("  ok: no sprite accumulation across repeated switches")

	## --- Degenerate routes. `_bounds()` indexes `pts[0]` unguarded, so every
	## caller has to hold the `size() < 2` gate; this walks the two shapes
	## that gate exists for with LOD still armed.
	print("--- degenerate routes ---")
	for case in [PackedVector2Array(), PackedVector2Array([Vector2(3, 4)])]:
		route_map.pts = case
		jp._refresh_route_map_layer_texture()
		route_map.queue_redraw()
		jp._route_line.queue_redraw()
		await _frames(2)
		print("  pts=%d -> sprites=%d children=%d (no crash)"
			% [case.size(), route_map._lod_sprites.size(), route_map.get_child_count()])
		if not route_map._lod_sprites.is_empty():
			print("  FAIL: a %d-point route must synthesize no tiles" % case.size())
			fails += 1

	print("=== SUMMARY fails=%d ===" % fails)
	get_tree().quit(1 if fails > 0 else 0)
