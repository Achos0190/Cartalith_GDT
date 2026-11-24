extends RefCounted
## Owner report 2026-11-23 (*"the attachment from routes (sea and land) to the settlements when you zoom in"*): a land
## road ends where its town is drawn. Driven through the REAL shell on one form:
##   python -I cartalith-native/tools/fennara_drive.py GODOT <project copy> res://tools/road_attach_drive.gd \
##       --scene res://shell/app.tscn -- <the form's window and user args>
## (take `C:/cw/.win_lock` yourself, with an owner file, for the run). Inputs are environment variables:
##   RA_OUT     the directory the crops are saved to (required);
##   RA_SEED, RA_GW, RA_GH (default 483920, 384, 288); the world is made by the engine call at the app's defaults
##   (`MISTAKES.md`: never through `EngineBridge.generate`, which resets rules for a new seed);
##   RA_TOWNS   "i,j,k": the settlements to frame by index (required);
## Per town and per `civ.bank_sites` state (off = the road as before, on = after) it saves the map round the town at x4,
## x16 and x64 and the City Viewer at its opening frame: <RA_OUT>/<form>_t<i>_<off|on>_<map_x<z>|city_x1>.png.
## Checks (each prints `ok` or `FAIL`):
##   1. the switch reaches the roads: with it on, some framed town has a road naming it (`town_a`/`town_b`);
##   2. AT x4 (the town layer is not drawn there) every such road's end is drawn within 4 px of its town's marker, as
##      the overlay draws both (`MapOverlay._road_points_drawn`, `_settle_pos`);
##   3. AT x16 and x64 (the town layer is drawn) every such road's end is drawn within 4 px of its town's nearest
##      drawn street (the plan the overlay holds, `_urban_layouts`, placed by `_plan_to_grid`).
##   Control: with the switch off at least one framed town's road end misses by more than 4 px at one of those zooms.

var _fails := 0
var app: Node
var bridge
var _form := "desktop"
var _out := ""


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


func _crop(centre: Vector2, half: int) -> Image:
	var img: Image = app.get_viewport().get_texture().get_image()
	var k: float = float(img.get_width()) / app.get_viewport().get_visible_rect().size.x
	var r := Rect2i(Vector2i(centre * k) - Vector2i(half, half), Vector2i(half, half) * 2).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	return img.get_region(r)


## A grid point (the frame roads are drawn in) in window pixels.
func _grid_px(p: Vector2) -> Vector2:
	var ov = app.viewport.overlay
	return ov.get_global_transform_with_canvas() * ov._point_to_screen(p, ov.displayed_rect())


func _marker_px(s: Dictionary) -> Vector2:
	var ov = app.viewport.overlay
	return ov.get_global_transform_with_canvas() * ov._settle_pos(s, ov.displayed_rect())


## Window pixels from grid point `p` to settlement `k`'s entry as the overlay draws its plan: its nearest street, or its
## market where the plan has no street within 4 px of it; -1 when the overlay holds no plan for it.
func _street_gap_px(ov, k: int, p: Vector2) -> float:
	var layout = ov._urban_layouts.get(k)
	if not (layout is Dictionary):
		return -1.0
	var to_grid: Callable = ov._plan_to_grid(layout, ov._settlements[k])
	var q := _grid_px(p)
	var best := INF
	for cls in (layout.get("streets", {}) as Dictionary):
		var segs: PackedVector2Array = layout["streets"][cls]
		var j := 0
		while j + 1 < segs.size():
			var a := _grid_px(to_grid.call(segs[j]))
			var b := _grid_px(to_grid.call(segs[j + 1]))
			best = minf(best, q.distance_to(Geometry2D.get_closest_point_to_segment(q, a, b)))
			j += 2
	if best == INF:
		return -1.0
	## A plan with no street at its market (a layout finding the census counts): the market itself is the entry.
	var m := _grid_px(to_grid.call(layout["market"]))
	var m_street := INF
	for cls in (layout.get("streets", {}) as Dictionary):
		var segs: PackedVector2Array = layout["streets"][cls]
		var j := 0
		while j + 1 < segs.size():
			m_street = minf(m_street, m.distance_to(Geometry2D.get_closest_point_to_segment(m, _grid_px(to_grid.call(segs[j])), _grid_px(to_grid.call(segs[j + 1])))))
			j += 2
	return minf(best, q.distance_to(m)) if m_street > 4.0 else best


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	_out = _env("RA_OUT", "")
	if _out == "":
		ctx.error("RA_OUT is required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	_form = "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		var r: Array = cont["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		await ctx.wait(0.6)
	var seed := int(_env("RA_SEED", "483920"))
	bridge.world_gen.set_villages_enabled(true)
	bridge.world_gen.generate_sized(seed, 800.0, int(_env("RA_GW", "384")), int(_env("RA_GH", "288")))
	bridge.last_width_km = bridge.world_gen.get_map_width_km()
	bridge.last_height_km = bridge.world_gen.get_map_height_km()
	bridge.has_world = true
	bridge.params_dirty = false
	bridge.generation_finished.emit(true)
	bridge.params_applied.emit()
	await ctx.wait(2.0)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	var towns: Array = []
	for t in _env("RA_TOWNS", "").split(",", false):
		towns.append(int(t))
	_check(ctx, "a world and its towns", bool(bridge.has_world) and not towns.is_empty())
	var host = app.viewport
	var ov = host.overlay
	var named := 0
	var worst_on_x4 := 0.0
	var worst_off_x4 := 0.0
	var worst_on_deep := 0.0
	var worst_off_deep := 0.0
	var deep_named := 0
	for on in [false, true]:
		var state := "on" if on else "off"
		bridge.param_set("civ.bank_sites", on)
		await ctx.wait(1.0)
		host.refresh()
		await ctx.wait(1.0)
		var places: Array = bridge.settlements()
		for k in towns:
			var s: Dictionary = places[k]
			var tid := int(s["tid"])
			for z in [4.0, 16.0, 64.0]:
				host.reset_view()
				await ctx.wait(0.2)
				host._zoom_at(host.size * 0.5, z)
				host.move_view_to(float(s["x"]), float(s["y"]))
				await ctx.wait(3.0 if z >= 16.0 else 1.5)
				var rect: Rect2 = ov.displayed_rect()
				var shown: bool = ov._town_layer_zoom_reached(rect)
				var ds: Dictionary = ov._settlements[k]
				for way in ov._roads:
					var pts: PackedVector2Array = way["points"]
					if pts.size() < 2:
						continue
					var drawn: PackedVector2Array = ov._road_points_drawn(way, shown)
					for e in [["town_a", 0], ["town_b", pts.size() - 1]]:
						var mine: bool = way.has(e[0]) and int(way[e[0]]) == tid
						## Off: the way's own end at this town's cell (the adapter's end test at 384 cells is 1.5 cells).
						if not on:
							mine = pts[e[1]].distance_to(Vector2(float(s["x"]), float(s["y"]))) < 0.5
						if not mine:
							continue
						named += int(on)
						var gap: float = _grid_px(drawn[e[1]]).distance_to(_marker_px(ds))
						if z == 4.0:
							if on:
								worst_on_x4 = maxf(worst_on_x4, gap)
							else:
								worst_off_x4 = maxf(worst_off_x4, gap)
						elif shown:
							var sgap := _street_gap_px(ov, k, drawn[e[1]])
							if sgap >= 0.0:
								if on:
									deep_named += 1
									worst_on_deep = maxf(worst_on_deep, sgap)
								else:
									worst_off_deep = maxf(worst_off_deep, sgap)
				_crop(_marker_px(ds), 170).save_png(_out.path_join("%s_t%d_%s_map_x%d.png" % [_form, k, state, int(z)]))
			host.reset_view()
			app.open_city_viewer(k)
			await ctx.wait(2.5)
			var cv = app.city_viewer_window
			var r: Rect2 = DriveCtx.rect_of(cv._canvas)
			var img: Image = app.get_viewport().get_texture().get_image()
			var kk: float = float(img.get_width()) / app.get_viewport().get_visible_rect().size.x
			img.get_region(Rect2i(Vector2i(r.position * kk), Vector2i(r.size * kk)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))).save_png(_out.path_join("%s_t%d_%s_city_x1.png" % [_form, k, state]))
			cv.hide()
			await ctx.wait(0.4)
	_check(ctx, "1. with the switch on the framed towns' roads name them", named > 0, "%d ends" % named)
	_check(ctx, "2. at x4 every named road end is drawn within 4 px of its marker", named > 0 and worst_on_x4 <= 4.0, "worst %.1f px" % worst_on_x4)
	_check(ctx, "3. at x16 and x64 every named road end is drawn within 4 px of its town's nearest drawn street", deep_named > 0 and worst_on_deep <= 4.0, "worst %.1f px over %d ends" % [worst_on_deep, deep_named])
	_check(ctx, "(control) with the switch off a framed town's road end misses by more than 4 px, at x4 or at x16/x64", worst_off_x4 > 4.0 or worst_off_deep > 4.0, "worst %.1f px at x4, %.1f px deeper" % [worst_off_x4, worst_off_deep])
	ctx.log("RESULT %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
