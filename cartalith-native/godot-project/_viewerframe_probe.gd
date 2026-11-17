extends Node
## City Viewer opening frame versus the layout's water (OUTSTANDING_WORK.md "Town placement ... (d)"): for every
## settlement of one generated world, does the viewer's opening frame (`CityViewerWindow._fit_box()` letterboxed into
## the canvas, as `_view_transform()` does at zoom 1, pan 0) contain any of the layout's water?
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . _viewerframe_probe.tscn -- [--seed N] [--grid WxH] [--shots <dir> --ids 70,36] [--riverbank 0|1] [--stretch 0|1] [--quick 0|1] [--villages 0|1] [--dump <dir> --ids 4,0]
##
## Water geometry counted, each separately: `water_poly` (a closed body), `river` (a centreline, `river_w` wide),
## `water_mask_runs` (map-water row quads) and `bridges`/`ford` (crossing points). A river-kind town is any layout whose
## `site_kind` is `river` or `riverthrough`. Run under `APPDATA=<empty dir>`. With `--shots` it is windowed (no
## `--headless`) and saves the viewer canvas of each id to `<dir>/viewer_<seed>_<id>.png`.
## `--dump <dir> --ids -1` dumps every town of 1 500 and up (Ruling FS); `--ids -2` every settlement (2026-11-17). `--dump <dir>` (with `--ids`) also writes each id's layout as `<dir>/<seed>_<gw>_<id>.json` for the both-banks
## evidence's `plot.py` (2026-11-15): the app's own layout, since the census's CPU world is not the shell's.
## `--riverbank 0|1` sets `urban_riverbank` (a river town builds to its bank, 2026-11-15) after the world is made and
## before any layout is read (the switch is read live), for before/after shots of one world; absent, the app's default.
## `--stretch 0|1` does the same for `urban_river_stretch` (Ruling FN, 2026-11-16: river towns stretched along their river).
## `--quick 1` (Ruling FQ) skips the water-in-frame measure and its two bars and lists every town of 1 500 and up with its cell.
## Result line: `RESULT: PASS|FAIL`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

## Whether segment a-b touches `r` (Liang-Barsky), used for the river centreline.
func _seg_hits(a: Vector2, b: Vector2, r: Rect2) -> bool:
	if r.has_point(a) or r.has_point(b):
		return true
	var t0 := 0.0
	var t1 := 1.0
	var d := b - a
	var p := [-d.x, d.x, -d.y, d.y]
	var q := [a.x - r.position.x, r.end.x - a.x, a.y - r.position.y, r.end.y - a.y]
	for k in 4:
		if is_zero_approx(p[k]):
			if q[k] < 0.0:
				return false
		else:
			var t: float = q[k] / p[k]
			if p[k] < 0.0:
				t0 = maxf(t0, t)
			else:
				t1 = minf(t1, t)
			if t0 > t1:
				return false
	return true

## Which kinds of the layout's water touch the visible model-metre rect `vis`: a dictionary of booleans.
func _water_in(lay: Dictionary, vis: Rect2) -> Dictionary:
	var out := {"poly": false, "river": false, "mask": false, "cross": false}
	var wp: PackedVector2Array = lay.get("water_poly", PackedVector2Array())
	for i in wp.size():
		if vis.has_point(wp[i]) or _seg_hits(wp[i], wp[(i + 1) % wp.size()], vis):
			out["poly"] = true
			break
	var rv: PackedVector2Array = lay.get("river", PackedVector2Array())
	for i in rv.size() - 1:
		if _seg_hits(rv[i], rv[i + 1], vis):
			out["river"] = true
			break
	for run in lay.get("water_mask_runs", []):
		var q: PackedVector2Array = run
		if q.size() >= 4 and vis.intersects(Rect2(q[0], q[2] - q[0]).abs()):
			out["mask"] = true
			break
	for p in lay.get("bridges", PackedVector2Array()):
		if vis.has_point(p):
			out["cross"] = true
	if lay.has("ford") and vis.has_point(lay["ford"]):
		out["cross"] = true
	return out

## Distance in model metres from `vis` to the nearest water vertex (river centreline, water_poly, mask-run corners); INF when none.
func _gap(lay: Dictionary, vis: Rect2) -> float:
	var best := INF
	var pts: Array = []
	pts.append_array(lay.get("river", PackedVector2Array()))
	pts.append_array(lay.get("water_poly", PackedVector2Array()))
	for run in lay.get("water_mask_runs", []):
		var q: PackedVector2Array = run
		if q.size() >= 4:
			pts.append(q[0])
			pts.append(q[2])
	for p in pts:
		var c := Vector2(clampf(p.x, vis.position.x, vis.end.x), clampf(p.y, vis.position.y, vis.end.y))
		best = minf(best, c.distance_to(p))
	return best

## A packed polygon as `[[x, y], ...]` for the dump.
func _pts(v) -> Array:
	var out := []
	for p in PackedVector2Array(v):
		out.append([p.x, p.y])
	return out

## Write one layout in `plot.py`'s format (market, plaza, river, ring, buildings, edges by class, parcels, pop; the box
## `wm` x `hm` and the site kind since Ruling FN, whose stretched towns have a larger box). Street
## widths are the drawn class's nominal ones (the dictionary carries none); a plot, not a measurement.
func _dump_layout(lay: Dictionary, path: String) -> void:
	var widths := {"lane": 3.4, "street": 4.4, "quay": 4.0, "ringroad": 5.4, "primary": 6.0}
	var edges := []
	var streets: Dictionary = lay.get("streets", {})
	for cls in streets:
		var segs := PackedVector2Array(streets[cls])
		for k in range(0, segs.size() - 1, 2):
			edges.append([cls, widths.get(cls, 4.0), segs[k].x, segs[k].y, segs[k + 1].x, segs[k + 1].y])
	var parcels := []
	var pcls: Array = lay.get("parcel_cls", [])
	var ppoly: Array = lay.get("parcels", [])
	for k in ppoly.size():
		parcels.append([String(pcls[k]) if k < pcls.size() else "", _pts(ppoly[k])])
	var blds := []
	for b in lay.get("buildings", []):
		blds.append(_pts(b))
	var m: Vector2 = lay.get("market", Vector2.ZERO)
	var d := {"market": [m.x, m.y], "river_w": lay.get("river_w", 0.0), "river": _pts(lay.get("river", PackedVector2Array())),
		"water_poly": _pts(lay.get("water_poly", PackedVector2Array())), "ring": _pts(lay.get("wall_ring", PackedVector2Array())),
		"plaza": _pts(lay.get("plaza", PackedVector2Array())), "buildings": blds, "edges": edges, "parcels": parcels,
		"pop": lay.get("pop", 0.0), "target": lay.get("pop_target", 0.0), "stages": lay.get("stages", PackedStringArray()),
		"wm": lay.get("wm", 1700.0), "hm": lay.get("hm", 1250.0), "site_kind": lay.get("site_kind", "")}
	## Ruling FS (2026-11-17): the town's water and harbour, so a stretched town can be checked against its
	## unstretched layout (the 246371 #11 defect): the open-water runs as `[x0, y0, x1, y1]` boxes (box metres),
	## the harbour point, the pier count and the quay streets' length.
	var runs := []
	for run in lay.get("water_mask_runs", []):
		var q := PackedVector2Array(run)
		if q.size() >= 4:
			runs.append([q[0].x, q[0].y, q[2].x, q[2].y])
	d["water_runs"] = runs
	d["harbour_pt"] = _pts(PackedVector2Array([lay["harbour_pt"]])) if lay.has("harbour_pt") else []
	var hd: Dictionary = lay.get("harbour", {})
	d["piers"] = PackedVector2Array(hd.get("piers", PackedVector2Array())).size() / 2
	var quay := 0.0
	var qs := PackedVector2Array(streets.get("quay", PackedVector2Array()))
	for k in range(0, qs.size() - 1, 2):
		quay += qs[k].distance_to(qs[k + 1])
	d["quay_m"] = quay
	## Whether the water body triangulates as it is: a river band that crosses itself at a tight bend does not, and
	## the map look's fill then resolved it (`urban_layout_draw.gd::_fill_parts`; before 2026-11-17 a fan painted the
	## inside of the bend as water).
	var wp := PackedVector2Array(lay.get("water_poly", PackedVector2Array()))
	d["water_poly_simple"] = wp.size() < 3 or not Geometry2D.triangulate_polygon(wp).is_empty()
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(JSON.stringify(d))
	f.close()

## `CityViewerWindow.fit_with_water` against literal layouts: grows to a near river, leaves a far one, and an inside one.
func _literal_checks() -> void:
	var F := CityViewerWindow.fit_with_water
	var built := Rect2(0, 0, 100, 100)
	var near := {"river": PackedVector2Array([Vector2(130, -50), Vector2(130, 150)]), "river_w": 10.0}
	var g: Rect2 = F.call(built, near)
	_ok(is_equal_approx(g.end.x, 140.0) and is_equal_approx(g.position.x, 0.0), "a river 30 m off pulls the frame to it plus one river width, 140 m (%s)" % g)
	var far := {"river": PackedVector2Array([Vector2(200, -50), Vector2(200, 150)]), "river_w": 10.0}
	_ok(F.call(built, far) == built, "a river 100 m off (over 0.6 of 100 m) leaves the frame alone")
	var inside := {"river": PackedVector2Array([Vector2(50, -50), Vector2(50, 150)]), "river_w": 10.0}
	_ok(F.call(built, inside) == built, "a river through the built mass leaves the frame alone")
	_ok(F.call(built, {}) == built, "no water leaves the frame alone")
	var hamlet := Rect2(0, 0, 20, 20)
	var h: Rect2 = F.call(hamlet, {"river": PackedVector2Array([Vector2(45, -5), Vector2(45, 40)]), "river_w": 4.0})
	_ok(h.end.x >= 49.0, "a 20 m hamlet reaches a river 25 m off (the 30 m floor, %s)" % h)
	var pond := {"water_poly": PackedVector2Array([Vector2(120, 10), Vector2(140, 10), Vector2(140, 30), Vector2(120, 30)]), "river_w": 10.0}
	_ok((F.call(built, pond) as Rect2).end.x >= 130.0, "a water body 20 m off pulls the frame to it")
	var runs := {"water_mask_runs": [PackedVector2Array([Vector2(115, 40), Vector2(125, 40), Vector2(125, 44), Vector2(115, 44)])], "river_w": 10.0}
	_ok((F.call(built, runs) as Rect2).end.x >= 115.0, "a map-water mask run 15 m off pulls the frame to it")

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		get_tree().quit(2))
	var seed_n := 483920
	var gw := 384
	var gh := 288
	var shots := ""
	var dump := ""
	var riverbank := -1
	var stretch := -1
	## `--villages 0` makes the world without villages, as the FH-R2 harness
	## (`fh_r2_baseline.rs`) does, so its settlement indices match; default on,
	## as the shell's probes ask.
	var villages := true
	## `--quick` (Ruling FQ, 2026-11-16): skip the water-in-frame measure (it lays out every settlement, which on a
	## 2 048 world with the debug DLL is most of the run) and its two bars, and list every town of 1 500 and up with its
	## cell, so a census row's `@(x,y)` can be matched to this world's index; for shots only.
	var quick := false
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
			"--dump": dump = args[i + 1]
			"--riverbank": riverbank = int(args[i + 1])
			"--stretch": stretch = int(args[i + 1])
			"--villages": villages = int(args[i + 1]) != 0
			"--quick": quick = int(args[i + 1]) != 0
			"--ids":
				for t in String(args[i + 1]).split(",", false):
					ids.append(int(t))
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
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": villages})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	if riverbank >= 0:
		_ok(bridge.param_set("urban_riverbank", riverbank == 1), "urban_riverbank set to %d" % riverbank)
	if stretch >= 0:
		_ok(bridge.param_set("urban_river_stretch", stretch == 1), "urban_river_stretch set to %d" % stretch)
	var places: Array = bridge.settlements()
	_ok(places.size() > 70 or gw != 384, "the world generated (%d settlements)" % places.size())
	app.open_city_viewer(0)
	for _f in 10:
		await get_tree().process_frame
	var cv = app.city_viewer_window
	cv.size = Vector2i(1560, 960)
	for _f in 8:
		await get_tree().process_frame
	var csz: Vector2 = cv._canvas.size
	print("canvas %s" % csz)
	var n_river := 0
	var n_river_water := 0
	var n_river_has_geom := 0
	var misses: Array = []
	if quick:
		for k in places.size():
			if int(places[k].get("population", 0)) >= 1500:
				print("  TOWN #%d at (%d,%d) pop %d" % [k, places[k]["x"], places[k]["y"], places[k]["population"]])
	for k in (0 if quick else places.size()):
		var lay: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		var kind := String(lay.get("site_kind", ""))
		## The `bay` towns, listed so a shot of one can be asked for by index (2026-11-15, Ruling FB's
		## both-banks change: the census's six "wet" bay towns were a measurement artefact; the shot shows one).
		if kind == "bay":
			print("  BAY #%d" % k)
		if not (kind in ["river", "riverthrough"]):
			## Listed so a frame of a town off the river can be asked for by index (2026-11-15, the
			## waterfront and market-square changes needed two such towns beside the river towns).
			print("  NONRIVER #%d %s pop %d" % [k, kind, int(places[k].get("population", 0))])
			continue
		n_river += 1
		cv._layout = lay
		var vt: Dictionary = cv._view_transform()
		var sc: float = vt["scale"]
		var org: Vector2 = vt["origin"]
		var vis := Rect2(-org / sc, csz / sc)
		var w := _water_in(lay, vis)
		var has_geom: bool = PackedVector2Array(lay.get("river", PackedVector2Array())).size() >= 2 \
			or PackedVector2Array(lay.get("water_poly", PackedVector2Array())).size() >= 3 \
			or Array(lay.get("water_mask_runs", [])).size() > 0
		if has_geom:
			n_river_has_geom += 1
		var any: bool = w["poly"] or w["mask"] or w["river"]
		if any:
			n_river_water += 1
		elif has_geom:
			misses.append(k)
		if k == 70 or not any:
			print("  #%d %s vis %s gap_m %.0f fitbox %s water-in-frame %s has_geom %s river_pts %d runs %d poly %d" % [k, kind, vis, _gap(lay, vis), cv._fit_box(), w, has_geom,
				PackedVector2Array(lay.get("river", PackedVector2Array())).size(), Array(lay.get("water_mask_runs", [])).size(),
				PackedVector2Array(lay.get("water_poly", PackedVector2Array())).size()])
	print("MEASURE seed %d grid %dx%d: %d settlements, %d river-kind, %d with water geometry, %d with water in the opening frame, %d MISS: %s" % [
		seed_n, gw, gh, places.size(), n_river, n_river_has_geom, n_river_water, misses.size(), str(misses)])
	## What this protects: before the fix 37 of 221 river-kind towns (settlement #70 among them) opened with no water in
	## frame; after it the residue is the towns whose nearest water is beyond `fit_with_water`'s bound (6 on this world).
	## Both bars are literals, never read back from the constant under test.
	if not quick:
		_ok(not 70 in misses, "settlement #70 shows water in its opening frame (seed 483920 at 384 x 288 only)" if (seed_n == 483920 and gw == 384) else "skipped, not seed 483920 at 384")
		_ok(misses.size() <= 8 and (seed_n != 483920 or gw != 384 or n_river_water >= 213), "at most 8 river-kind towns open without water (%d)" % misses.size())
	_literal_checks()
	if dump != "":
		DirAccess.make_dir_recursive_absolute(dump)
		for k in places.size():
			if int(places[k].get("population", 0)) >= 1500:
				print("  TOWN #%d at (%d,%d) pop %d" % [k, places[k]["x"], places[k]["y"], places[k]["population"]])
		## `--ids -2` (2026-11-17, the town leftovers): every settlement, villages too, to find the towns a change moved.
		if ids == [-2]:
			ids.clear()
			for k in places.size():
				ids.append(k)
		## `--ids -1` (Ruling FS, 2026-11-17): every town of 1 500 and up, for the stretch class audit.
		if ids == [-1]:
			ids.clear()
			for k in places.size():
				if int(places[k].get("population", 0)) >= 1500:
					ids.append(k)
		for id in ids:
			print("  DUMP #%d %s" % [id, str(places[id])])
			_dump_layout(bridge.urban_layouts(PackedInt32Array([id]))[0], dump.path_join("%d_%d_%d.json" % [seed_n, gw, id]))
	if shots != "":
		DirAccess.make_dir_recursive_absolute(shots)
		for id in ids:
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
			img.get_region(Rect2i(rect).intersection(Rect2i(Vector2i.ZERO, img.get_size()))).save_png(shots.path_join(("viewer_%d_%d.png" % [seed_n, id]) if riverbank < 0 else ("viewer_%d_%d_riverbank%d.png" % [seed_n, id, riverbank])))
			cv.hide()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
