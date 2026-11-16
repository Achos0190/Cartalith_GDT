extends Node
## Ruling FQ map-level probe (owner, 2026-11-16): a STRETCHED river town on the world map -- its pins and its town layer
## follow the town's own box (`WorldGen.urban_layout_boxes`, `map_overlay.gd::_town_half_km` / `_layout_half_km`), not the
## reference's 1.7 km box (`URBAN_SITE_BOX_KM`).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _stretchmap_probe.tscn -- [--seed N] [--grid WxH] [--id K] [--out <dir>]
##
## Windowed (not --headless: the frames are the evidence, and `frame_post_draw` never fires headless). Run under a scratch
## APPDATA. Makes the app's world (800 km, villages on, the app's parameters, then `urban_river_stretch` set ON), picks town `--id`
## or the first, away from the plate's frame, whose layout is laid out stretched (its `wm` or `hm` over the reference's
## 1 700 x 1 250 m), and places a
## point of interest on its river side half way between the reference's 0.85 km reach and its own half box.
## Bars (each printed `PASS`/`FAIL`):
##   S1  the town's planned box (`urban_layout_boxes`) is its laid-out box, and over the reference's
##   S2  the POI stands farther than 0.85 km from the town (the old rule would not claim it) and the overlay claims it
##       (`_town_pin_membership`): hidden one zoom step below the town layer, drawn at the zoom where the layer appears
##   S3  culling: with the town's centre panned off the left edge by more than 0.85 km but less than its own half box, the
##       town is still in the town layer's draw list (the old half-box slack would have dropped it)
## Frames (`--out`): `stretchmap_<seed>_<id>_{below,appears,closer,edge}.png`, the whole 1600 x 1000 viewport (`closer` is
## two zoom steps past `appears`).
## Result line `RESULT: PASS|FAIL`. Exit 0 pass, 1 fail, 2 could not run.

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for _f in n:
		await get_tree().process_frame

## The overlay's global (viewport) position of grid point `cell`.
func _global_of(overlay: Control, cell: Vector2) -> Vector2:
	var rect: Rect2 = overlay._displayed_rect()
	return overlay.get_global_transform() * overlay._point_to_screen(cell, rect)

func _shot(vp: SubViewport, path: String) -> void:
	await RenderingServer.frame_post_draw
	var img: Image = vp.get_texture().get_image()
	img.save_png(path)
	print("  shot %s" % path)

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("RESULT: FAIL (timeout)")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		print("RESULT: FAIL (windowed only: the frames are the evidence)")
		get_tree().quit(2)
		return
	var seed_n := 483920
	var gw := 384
	var gh := 288
	var want_id := -1
	var out := ""
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed": seed_n = int(args[i + 1])
			"--grid":
				var wh := String(args[i + 1]).split("x")
				gw = int(wh[0])
				gh = int(wh[1])
			"--id": want_id = int(args[i + 1])
			"--out": out = args[i + 1]
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	## Off in the app's defaults pending the owner (Ruling FQ); turned on here, read live by every layout.
	_ok(bridge.param_set("urban_river_stretch", true), "urban_river_stretch set on")
	## A live switch drops no cached town by itself (true of every `urban_*` switch); the shell's own path for "every cached
	## town is stale" is `forget_all_urban_layouts`, which also re-reads the planned boxes (`refresh_urban_boxes`).
	app.viewport.forget_all_urban_layouts()
	var places: Array = bridge.settlements()
	var boxes: Dictionary = bridge.urban_layout_boxes()
	print("planned stretched boxes: %d of %d settlements" % [boxes.size(), places.size()])
	## The town: `--id`, else the first planned-stretched town whose layout is laid out stretched.
	var id := -1
	var lay: Dictionary = {}
	for k in places.size():
		if want_id >= 0 and k != want_id:
			continue
		if want_id < 0 and not boxes.has(int(places[k]["tid"])):
			continue
		## Away from the plate's frame (`_border_frac`), which clips and fades the map's edge: 12 % of the grid.
		if want_id < 0 and (int(places[k]["x"]) < gw * 0.12 or int(places[k]["x"]) > gw * 0.88 or int(places[k]["y"]) < gh * 0.12 or int(places[k]["y"]) > gh * 0.88):
			continue
		var l: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		if want_id >= 0 or float(l["wm"]) > 1701.0 or float(l["hm"]) > 1251.0:
			id = k
			lay = l
			break
	if id < 0:
		print("RESULT: FAIL (no stretched town on this world)")
		get_tree().quit(2)
		return
	var s: Dictionary = places[id]
	var box = boxes.get(int(s["tid"]))
	print("town #%d %s pop %d at (%d,%d): planned box %s, laid out %.0f x %.0f m, pin %s" % [id, s["name"], s["population"],
		s["x"], s["y"], box, float(lay["wm"]), float(lay["hm"]), lay["pin"]])
	_ok(box is Vector2 and is_equal_approx((box as Vector2).x, float(lay["wm"])) and is_equal_approx((box as Vector2).y, float(lay["hm"])),
		"S1 the planned box is the laid-out box")
	var half_m: float = maxf(float(lay["wm"]), float(lay["hm"])) * 0.5
	_ok(half_m > 900.0, "S1 the town's half box (%.0f m) is past the reference's 850 m" % half_m)

	## S2: a POI on the river half way between 0.85 km and the half box, the farthest river point inside that ring.
	var pin: Vector2 = lay["pin"]
	var rv: PackedVector2Array = lay.get("river", PackedVector2Array())
	var target_m := (850.0 + half_m) * 0.5
	var best := Vector2.INF
	for p in rv:
		var d := p.distance_to(pin)
		if d > 850.0 + 60.0 and d < half_m - 60.0 and (best == Vector2.INF or absf(d - target_m) < absf(best.distance_to(pin) - target_m)):
			best = p
	_ok(best != Vector2.INF, "S2 the layout's river runs between 0.85 km and the half box")
	if best == Vector2.INF:
		print("RESULT: FAIL")
		get_tree().quit(1)
		return
	var gpm := float(gw) / 800000.0
	var cell := Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5) + (best - pin) * gpm
	var d_km := best.distance_to(pin) * 0.001
	var poi := int(bridge.poi_add(cell.x, cell.y, "well", "Ruling FQ probe"))
	_ok(poi >= 0, "S2 the POI is placed %.2f km from the town (over the old 0.85 km reach)" % d_km)
	app.viewport.refresh_annotations()
	await _frames(4)
	var vh = app.viewport
	var overlay: Control = vh.overlay
	_ok((overlay._town_pin_membership()["poi"] as Dictionary).has(poi), "S2 the overlay claims the POI for the town (its own box)")

	## Centre the town and zoom in until its layout draws.
	var centre_g: Vector2 = (vh as Control).get_global_rect().get_center()
	var zooms := 0
	var below_zoom := 0.0
	while zooms < 80:
		vh._camera.position += centre_g - _global_of(overlay, Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5))
		overlay.queue_redraw()
		await _frames(3)
		if (overlay._urban_draw_list as PackedInt32Array).has(id) or vh.zoom() >= vh._zoom_max:
			break
		below_zoom = vh.zoom()
		vh._zoom_at(vh.size * 0.5, 1.15)
		zooms += 1
	for _w in 40:
		await _frames(2)
		if (overlay._urban_draw_list as PackedInt32Array).has(id):
			break
	var appear_zoom: float = vh.zoom()
	_ok((overlay._urban_draw_list as PackedInt32Array).has(id), "the town layer draws the town at zoom %.2f" % appear_zoom)
	var rect: Rect2 = overlay._displayed_rect()
	_ok(not overlay._poi_town_hidden(poi, rect), "S2 at that zoom the far POI is drawn (the town shows)")
	if out != "":
		DirAccess.make_dir_recursive_absolute(out)
		await _frames(120)   ## the deep-zoom tiles and the layout's crossfade settle
		await _shot(vp, out.path_join("stretchmap_%d_%d_appears.png" % [seed_n, id]))
		## Two steps closer, so the town and the far POI are large enough to read.
		vh._zoom_at(vh.size * 0.5, 2.0)
		vh._camera.position += centre_g - _global_of(overlay, Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5))
		overlay.queue_redraw()
		await _frames(120)
		await _shot(vp, out.path_join("stretchmap_%d_%d_closer.png" % [seed_n, id]))
		vh._zoom_at(vh.size * 0.5, appear_zoom / vh.zoom())
		await _frames(4)
	## One step below: the layer is gone and the far POI hides with the town.
	vh._zoom_at(vh.size * 0.5, below_zoom / appear_zoom)
	vh._camera.position += centre_g - _global_of(overlay, Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5))
	overlay.queue_redraw()
	await _frames(4)
	rect = overlay._displayed_rect()
	_ok(not overlay._town_layer_zoom_reached(rect), "below: zoom %.2f is under the town layer's gate" % vh.zoom())
	_ok(overlay._poi_town_hidden(poi, rect), "S2 below: the far POI hides with its town")
	if out != "":
		await _frames(120)
		await _shot(vp, out.path_join("stretchmap_%d_%d_below.png" % [seed_n, id]))

	## S3: back to the appearing zoom, the town's centre off the left edge by the middle of (0.85 km, half box).
	vh._zoom_at(vh.size * 0.5, appear_zoom / vh.zoom())
	overlay.queue_redraw()
	await _frames(3)
	rect = overlay._displayed_rect()
	var m_scale: float = overlay._urban_m_scale(rect)
	var visible_area: Rect2 = overlay.get_global_transform().affine_inverse() * overlay.get_viewport_rect()
	visible_area = visible_area.intersection(overlay._interior_rect(rect)) if overlay._border_frac > 0.0 else visible_area
	## Just past the old slack, so the town's own buildings (which reach less far than its box) still show at the edge.
	var off_m := 850.0 + (half_m - 850.0) * 0.25
	var town_local: Vector2 = overlay._cell_to_screen(Vector2(s["x"], s["y"]), rect)
	var want_local := Vector2(visible_area.position.x - off_m * m_scale, town_local.y)
	## The camera scales the overlay, so a local shift is that times the zoom in viewport pixels.
	vh._camera.position += (want_local - town_local) * vh.zoom()
	overlay.queue_redraw()
	await _frames(4)
	print("  edge: town centre %.0f m left of the visible edge (old slack 850 m, own %.0f m)" % [off_m, half_m])
	_ok((overlay._urban_draw_list as PackedInt32Array).has(id), "S3 the town still draws with its centre %.0f m off screen" % off_m)
	if out != "":
		await _frames(120)
		await _shot(vp, out.path_join("stretchmap_%d_%d_edge.png" % [seed_n, id]))
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
