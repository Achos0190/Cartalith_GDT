extends Node
## Smooth sea/lake shorelines (owner, 2026-11-15, "Sea and lake should not draw as squares"; `shell/urban_shore.gd`).
## Real towns of one generated world (bay towns exist only at the 2 048 grid: `--grid 2048x1536`, whose first bay towns are #177, #191, #195; the audit's #32 and #165 need the FH-R2 harness's world and are not in the app's; plus the
## first lake/sea towns found), each checked for:
##   - the traced boundary exists (positive control: it HAS axis-aligned right-angle corners) and the smoothed rings
##     have NONE (no staircase) -- mutant `CHAIKIN_PASSES = 0` fails here;
##   - every smoothed vertex lies within 0.5 cell of the traced boundary, and every traced corner within 0.5 cell of
##     the smoothed shore (the owner's bound);
##   - the fill pieces are non-empty and cover the mask's area to within 6 %;
##   - drawing leaves `water_mask_runs` byte-identical (it is only read);
##   - (windowed only; `ImageTexture`/viewport pixels are a no-op headless, MISTAKES.md) the drawn mesh puts water
##     colour at a point inside a piece, through a ROTATED transform too, and background outside it; with a bridge
##     point released there the pixel goes back to background -- mutant "fill skipped" fails here.
##   Godot_v4.7.2-stable_win64_console.exe [--headless] --path . _shorepoly_probe.tscn -- [--seed N] [--grid WxH] [--villages 0|1] [--ids 32,165] [--shots <dir>]
## Run under `APPDATA=<empty dir>`. `--shots` saves the City Viewer canvas of each id to `<dir>/shore_<id>.png`
## (windowed). Result line `RESULT: PASS|FAIL`; exit 0 / 1 / 2 (bad argument or timeout).

const SHORE := preload("res://shell/urban_shore.gd")
const DRAW := preload("res://shell/urban_layout_draw.gd")

var _fails := 0
var _checks := 0
var _smoothed_polys := 0
var _smoothed_rivers := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

## Distance from `p` to the closed ring `ring`.
func _dist_ring(p: Vector2, ring: PackedVector2Array) -> float:
	var best := INF
	for k in ring.size():
		var c := Geometry2D.get_closest_point_to_segment(p, ring[k], ring[(k + 1) % ring.size()])
		best = minf(best, p.distance_to(c))
	return best

func _dist_rings(p: Vector2, rings: Array) -> float:
	var best := INF
	for r in rings:
		best = minf(best, _dist_ring(p, r))
	return best

## Vertices of `ring` where both edges are axis-aligned and the turn is exactly 90 degrees: the staircase signature.
func _right_angles(ring: PackedVector2Array) -> int:
	var n := 0
	for k in ring.size():
		var a := ring[(k + ring.size() - 1) % ring.size()]
		var b := ring[k]
		var c := ring[(k + 1) % ring.size()]
		var d1 := b - a
		var d2 := c - b
		var axis1 := is_zero_approx(d1.x) or is_zero_approx(d1.y)
		var axis2 := is_zero_approx(d2.x) or is_zero_approx(d2.y)
		if axis1 and axis2 and is_zero_approx(d1.dot(d2)) and d1.length() > 0.0 and d2.length() > 0.0:
			n += 1
	return n

func _area(p: PackedVector2Array) -> float:
	var a := 0.0
	for k in p.size():
		a += p[k].cross(p[(k + 1) % p.size()])
	return absf(a) * 0.5

## The pieces' largest triangle's centroid, a point certainly inside the water.
func _inside_point(pieces: Array) -> Vector2:
	var best := Vector2.ZERO
	var best_a := -1.0
	for p in pieces:
		var tri := Geometry2D.triangulate_polygon(p)
		for t in range(0, tri.size(), 3):
			var a: Vector2 = p[tri[t]]
			var b: Vector2 = p[tri[t + 1]]
			var c: Vector2 = p[tri[t + 2]]
			var ar := absf((b - a).cross(c - a)) * 0.5
			if ar > best_a:
				best_a = ar
				best = (a + b + c) / 3.0
	return best

func _check_town(id: int, lay: Dictionary, windowed: bool) -> void:
	var runs: Array = lay.get("water_mask_runs", [])
	var before := var_to_str(runs)
	var s: Dictionary = SHORE.smooth_rings(runs)
	var cell: float = s["cell"]
	var raw: Array = s["raw"]
	var smooth: Array = s["smooth"]
	print("#%d kind %s: %d runs, cell %.1f, %d rings" % [id, lay.get("site_kind", "?"), runs.size(), cell, raw.size()])
	_ok(not runs.is_empty() and not raw.is_empty() and not smooth.is_empty(), "#%d has a traced and a smoothed shore" % id)
	var raw_ra := 0
	var sm_ra := 0
	var worst := 0.0
	var worst_back := 0.0
	for r in raw:
		raw_ra += _right_angles(r)
		for v in r:
			worst_back = maxf(worst_back, _dist_rings(v, smooth))
	for r in smooth:
		sm_ra += _right_angles(r)
		for v in r:
			worst = maxf(worst, _dist_rings(v, raw))
	_ok(raw_ra >= 4, "#%d positive control: the traced boundary is a staircase (%d right-angle corners)" % [id, raw_ra])
	_ok(sm_ra == 0, "#%d the smoothed shore has no staircase (%d right-angle corners)" % [id, sm_ra])
	_ok(worst <= 0.5 * cell + 0.001, "#%d every smoothed vertex within 0.5 cell of the traced boundary (worst %.3f cell)" % [id, worst / cell])
	_ok(worst_back <= 0.5 * cell + 0.001, "#%d every traced corner within 0.5 cell of the smoothed shore (worst %.3f cell)" % [id, worst_back / cell])
	var cached: Dictionary = SHORE.cached(lay, [])
	var pieces: Array = cached["pieces"]
	var mask_area := 0.0
	for run in runs:
		var q: PackedVector2Array = run
		if q.size() >= 4:
			mask_area += absf((q[2].x - q[0].x) * (q[2].y - q[0].y))
	var fill_area := 0.0
	for p in pieces:
		fill_area += _area(p)
	_ok(not pieces.is_empty() and cached["mesh"] != null, "#%d fill pieces and a mesh exist (%d pieces)" % [id, pieces.size()])
	_ok(absf(fill_area - mask_area) <= 0.06 * mask_area, "#%d fill area %.0f within 6%% of the mask's %.0f" % [id, fill_area, mask_area])
	_ok(SHORE.cached(lay, []) == cached or SHORE.cached(lay, [])["mesh"] == cached["mesh"], "#%d the second call hits the per-layout cache" % id)
	## The outline polygon (`water_poly`, the shore band and the ink fill read it): where it is the traced staircase it
	## must come back smoothed and within 0.5 cell of the stored outline; where it is not it must come back as stored.
	var wp: PackedVector2Array = lay.get("water_poly", PackedVector2Array())
	var swp: PackedVector2Array = SHORE.smooth_water_poly(lay)
	if wp.size() >= 4:
		if swp != wp:
			_smoothed_polys += 1
			var wd := 0.0
			for v in swp:
				wd = maxf(wd, _dist_ring(v, wp))
			var wra := _right_angles(swp)
			_ok(wd <= 0.5 * cell + 0.001 and wra == 0, "#%d water_poly smoothed (%d -> %d pts), worst %.3f cell from the stored outline, %d right angles" % [id, wp.size(), swp.size(), wd / cell, wra])
		else:
			print("#%d water_poly (%d pts) is not a traced staircase, left as stored" % [id, wp.size()])
	## The `river` polyline of a coastal site is the traced shoreline (stroked as the ink plan's water edge).
	var rv: PackedVector2Array = lay.get("river", PackedVector2Array())
	var srv: PackedVector2Array = SHORE.smooth_river(lay)
	if rv.size() >= 4 and srv != rv:
		_smoothed_rivers += 1
		var rd := 0.0
		for v in srv:
			var best := INF
			for k in rv.size() - 1:
				best = minf(best, v.distance_to(Geometry2D.get_closest_point_to_segment(v, rv[k], rv[k + 1])))
			rd = maxf(rd, best)
		_ok(rd <= 0.5 * cell + 0.001 and srv[0] == rv[0] and srv[srv.size() - 1] == rv[rv.size() - 1], "#%d shoreline river polyline smoothed (%d -> %d pts), worst %.3f cell from the stored line, ends kept" % [id, rv.size(), srv.size(), rd / cell])
	if windowed:
		await _pixels(id, lay, pieces)
	_ok(var_to_str(lay.get("water_mask_runs", [])) == before, "#%d drawing left water_mask_runs untouched" % id)

## Draws the layout's water mask into a SubViewport through two transforms and reads the pixels back.
func _pixels(id: int, lay: Dictionary, pieces: Array) -> void:
	var inside := _inside_point(pieces)
	var bb := Rect2(pieces[0][0], Vector2.ZERO)
	for p in pieces:
		for v in p:
			bb = bb.expand(v)
	var sc := minf(560.0 / bb.size.x, 360.0 / bb.size.y)
	for rot in [0.0, 0.6]:
		var xf := Transform2D(rot, Vector2.ZERO).scaled_local(Vector2(sc, sc))
		xf.origin = Vector2(300, 200) - xf * bb.get_center()
		var project := func(poly: PackedVector2Array) -> PackedVector2Array:
			var out := PackedVector2Array()
			for v in poly:
				out.append(xf * v)
			return out
		for released in [false, true]:
			var l2 := lay.duplicate()
			l2.erase(SHORE.CACHE_KEY)
			if released:
				l2["bridges"] = PackedVector2Array([inside])
				l2["river_w"] = 8.0
			var vp := SubViewport.new()
			vp.size = Vector2i(600, 400)
			vp.transparent_bg = false
			vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
			add_child(vp)
			var bg := ColorRect.new()
			bg.color = Color(1, 0, 1)
			bg.size = Vector2(600, 400)
			vp.add_child(bg)
			var c := Control.new()
			c.size = Vector2(600, 400)
			vp.add_child(c)
			c.draw.connect(func() -> void:
				DRAW._draw_water_mask(c, l2, project, func(col: Color) -> Color: return col))
			c.queue_redraw()
			for _f in 4:
				await get_tree().process_frame
			await RenderingServer.frame_post_draw
			var img := vp.get_texture().get_image()
			var px := img.get_pixelv(Vector2i(xf * inside))
			## 8-bit read-back, so a per-channel tolerance of 2/255 rather than `is_equal_approx`.
			var is_water := absf(px.r - DRAW.WATER.r) < 0.01 and absf(px.g - DRAW.WATER.g) < 0.01 and absf(px.b - DRAW.WATER.b) < 0.01
			var far := img.get_pixel(1, 1)
			_ok(far.is_equal_approx(Color(1, 0, 1)), "#%d rot %.1f: outside the water is background" % [id, rot])
			if released:
				_ok(not is_water, "#%d rot %.1f: the bridge release clears the water at its point" % [id, rot])
			else:
				_ok(is_water, "#%d rot %.1f: the mesh paints water at a point inside the shore (%s)" % [id, rot, px])
			vp.queue_free()

## A literal 10 x 10 cell lake (cell 10 m) with a 2 x 2 island in the middle, and a release disc wholly inside the
## water: what the real towns here do not reach (their water has no island). Protects the hole path of
## `fill_pieces`: no returned polygon may be a hole, and the filled area must be the lake minus the island minus the disc.
func _literal_island() -> void:
	var runs: Array = []
	for j in 10:
		var xs: Array = [[0, 10]] if j < 4 or j > 5 else [[0, 4], [6, 10]]
		for x in xs:
			runs.append(PackedVector2Array([Vector2(x[0] * 10, j * 10), Vector2(x[1] * 10, j * 10), Vector2(x[1] * 10, j * 10 + 10), Vector2(x[0] * 10, j * 10 + 10)]))
	var s: Dictionary = SHORE.smooth_rings(runs)
	_ok((s["raw"] as Array).size() == 2, "island lake traces to an outer ring and an island ring")
	var pieces: Array = SHORE.fill_pieces(s["smooth"], [])
	var holes := 0
	var area := 0.0
	for p in pieces:
		holes += 1 if Geometry2D.is_polygon_clockwise(p) else 0
		area += _area(p)
	_ok(holes == 0 and not pieces.is_empty(), "island lake fills as %d hole-free pieces" % pieces.size())
	## 10 x 10 cells minus the 2 x 2 island = 96 cells = 9600 m2. Smoothing trims the lake's outer corners and the
	## island's too (which gives water back), so the two nearly cancel: within 3 %.
	_ok(absf(area - 9600.0) <= 0.03 * 9600.0, "island lake area %.0f within 3%% of 9600" % area)
	var rel: Array = SHORE.fill_pieces(s["smooth"], [[Vector2(20, 20), 8.0]])
	var area2 := 0.0
	var holes2 := 0
	for p in rel:
		holes2 += 1 if Geometry2D.is_polygon_clockwise(p) else 0
		area2 += _area(p)
	_ok(holes2 == 0 and absf((area - area2) - PI * 64.0) <= 12.0, "a release disc wholly inside the water removes its area (%.0f) with no hole polygon" % (area - area2))

## A literal triangle of water whose hypotenuse is a 3-cell-by-1-cell staircase (cell 10 m): the shape of a gentle
## real coast. On its middle stretch (6+ cells from every corner) the smoothed ring must turn by 25 degrees or less
## at every vertex; Chaikin alone leaves 45-degree turns there, an unsmoothed staircase 90. Protects the averaging
## pass, which the real-town checks (no exact right angles) cannot see.
func _literal_staircase() -> void:
	var runs: Array = []
	for j in 12:
		runs.append(PackedVector2Array([Vector2(0, j * 10), Vector2((j + 1) * 30, j * 10), Vector2((j + 1) * 30, j * 10 + 10), Vector2(0, j * 10 + 10)]))
	var s: Dictionary = SHORE.smooth_rings(runs)
	var ring: PackedVector2Array = s["smooth"][0]
	var corners := [Vector2(0, 0), Vector2(0, 120), Vector2(360, 120)]
	var worst := 0.0
	var seen := 0
	for k in ring.size():
		var v := ring[k]
		var mid := true
		for c in corners:
			if v.distance_to(c) < 60.0:
				mid = false
		if not mid or absf(v.x) < 1.0 or absf(v.y - 120.0) < 1.0:
			continue
		var a := ring[(k + ring.size() - 1) % ring.size()]
		var b := ring[(k + 1) % ring.size()]
		worst = maxf(worst, absf((v - a).angle_to(b - v)))
		seen += 1
	_ok(seen > 20, "the staircase fixture has a middle stretch (%d vertices)" % seen)
	_ok(rad_to_deg(worst) <= 25.0, "the smoothed staircase turns at most 25 degrees per vertex mid-stretch (worst %.1f)" % rad_to_deg(worst))

func _ready() -> void:
	get_tree().create_timer(1200.0).timeout.connect(func() -> void:
		get_tree().quit(2))
	var seed_n := 483920
	var gw := 384
	var gh := 288
	var shots := ""
	## `--villages 0` makes the world as the FH-R2 harness does, whose settlement indices #32 and #165 are the bay towns.
	var villages := true
	var ids: Array[int] = []
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed": seed_n = int(args[i + 1])
			"--grid":
				var wh := String(args[i + 1]).split("x")
				gw = int(wh[0])
				gh = int(wh[1])
			"--shots": shots = args[i + 1]
			"--villages": villages = int(args[i + 1]) != 0
			"--ids":
				ids.clear()
				for t in String(args[i + 1]).split(",", false):
					ids.append(int(t))
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	var windowed := DisplayServer.get_name() != "headless"
	print("windowed: %s" % windowed)
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": villages})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var places: Array = bridge.settlements()
	## Besides any `--ids`, the first three non-river towns (bay, coastal, lake) and two river towns that carry mask runs.
	var chosen: Array[int] = ids.duplicate()
	var extra := 0
	var extra_river := 0
	for k in places.size():
		if (extra >= 3 and extra_river >= 2) or k in chosen:
			continue
		var l: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		var kind := String(l.get("site_kind", ""))
		var nr := Array(l.get("water_mask_runs", [])).size()
		if nr > 20 and not (kind in ["river", "riverthrough"]) and extra < 3:
			chosen.append(k)
			extra += 1
		elif nr > 20 and kind in ["river", "riverthrough"] and extra_river < 2:
			chosen.append(k)
			extra_river += 1
	_literal_island()
	_literal_staircase()
	_ok(chosen.size() >= 3, "at least 3 towns with sea or lake water (%s)" % str(chosen))
	for k in chosen:
		var lay: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		await _check_town(k, lay, windowed)
	if shots != "":
		DirAccess.make_dir_recursive_absolute(shots)
		var cv = null
		app.open_city_viewer(0)
		for _f in 10:
			await get_tree().process_frame
		cv = app.city_viewer_window
		for id in chosen:
			app.open_city_viewer(id)
			for _f in 12:
				await get_tree().process_frame
			cv.size = Vector2i(1560, 960)
			cv.position = Vector2i(20, 20)
			for _f in 8:
				await get_tree().process_frame
			cv._reset_view()
			for _f in 6:
				await get_tree().process_frame
			await RenderingServer.frame_post_draw
			var img: Image = vp.get_texture().get_image()
			var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
			img.get_region(Rect2i(rect).intersection(Rect2i(Vector2i.ZERO, img.get_size()))).save_png(shots.path_join("shore_%d.png" % id))
			cv.hide()
	_ok(_smoothed_rivers >= 1, "at least one town's staircase shoreline river polyline was smoothed (%d)" % _smoothed_rivers)
	_ok(_smoothed_polys >= 1, "at least one town's staircase water_poly was smoothed (%d)" % _smoothed_polys)
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
