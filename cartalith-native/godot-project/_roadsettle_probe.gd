extends Node
## `OUTSTANDING_WORK.md` "Roads do not meet their settlements: way ends and
## the settlement pins are offset from each other". Owner: orientations are
## right, but a way's own endpoint and its settlement's pin do not land on
## the same screen point.
##
## Measures the screen distance between each way's endpoint (`get_roads()`'s
## `points[0]`/`points[-1]`, projected through `map_overlay.gd::
## _point_to_screen`) and the nearest settlement's pin (`get_settlements()`'s
## `x`/`y`, projected through `_cell_to_screen`) -- the same two-projection
## technique `_mapdata_probe.gd::_leg_labels` already uses for generated
## labels vs their settlement markers, applied here to roads instead.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _roadsettle_probe.tscn -- --out DIR
##
## Arguments:
##   --out DIR     where the PNGs land (required)
##   --seed N      world seed (default 483920)
##   --grid WxH    world size (default 512x384)
##
## Exit status: 0 always (measurement probe, not a pass/fail gate) unless it
## could not run (2).

const DEEP := 16.0

var GRID := Vector2i(512, 384)
var _seed := 483920
var _out := ""
var _app: Node
var _vh: Control
var _br: Node
## Owner-repro extras: when set, these override the plain 512x384 defaults
## so the probe reproduces the exact world the screenshot was taken on
## (2048x1311, seed 246371, metropolis on, geology_model on, river_density
## 1.55) instead of the smaller world the original pass used.
var _metropolis := false
var _river_density := 1.0
var _geology_model := false
## When set (grid-space), `_measure` also reports the nearest settlement to
## this point and its own nearest-endpoint distance, regardless of the
## 10-settlement/3-cell-skip limits below -- added to find "Thalefabough"
## by the owner-supplied map coordinate rather than by array order.
var _focus := Vector2.INF


func _p(s: String) -> void:
	print("ROADSETTLE  " + s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]; i += 1
			"--seed":
				_seed = int(args[i + 1]); i += 1
			"--grid":
				var p := args[i + 1].split("x"); GRID = Vector2i(int(p[0]), int(p[1])); i += 1
			"--metropolis":
				_metropolis = true
			"--river-density":
				_river_density = float(args[i + 1]); i += 1
			"--geology-model":
				_geology_model = true
			"--focus":
				var f := args[i + 1].split(","); _focus = Vector2(float(f[0]), float(f[1])); i += 1
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument %s" % args[i])
				return false
		i += 1
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


func _view(p: Vector2, deep: bool) -> void:
	_vh.reset_view()
	await _frames(3)
	if deep:
		_vh.zoom_step(DEEP / _vh.zoom())
		_vh.move_view_to(p.x, p.y)
	await _settle()


func _crop(img: Image, screen_p: Vector2, name: String, half: int = 160) -> void:
	var s := Vector2i(screen_p)
	var r := Rect2i(s - Vector2i(half, half), Vector2i(2 * half, 2 * half)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	img.get_region(r).save_png("%s/%s.png" % [_out, name])


## For each settlement, the nearest road endpoint (any way's `points[0]` or
## `points[points.size()-1]`) in grid space, and the distance between them
## projected through the overlay's own two projections, both at the current
## view.
func _measure(tag: String) -> void:
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var settlements: Array = _br.settlements()
	var roads: Array = _br.roads()
	var endpoints: Array = []
	for r: Dictionary in roads:
		var pts: PackedVector2Array = r.get("points", PackedVector2Array())
		if pts.size() < 2:
			continue
		endpoints.append(pts[0])
		endpoints.append(pts[pts.size() - 1])
	_p("%s: %d settlements, %d roads, %d endpoints" % [tag, settlements.size(), roads.size(), endpoints.size()])
	var n := 0
	var worst := 0.0
	var sum := 0.0
	var far_cell := []  ## settlements whose nearest endpoint is >2 CELLS away
	var focus_best_name := ""
	var focus_best_d := INF
	var focus_cell := Vector2.INF
	for s: Dictionary in settlements:
		var cell := Vector2(float(s["x"]), float(s["y"]))
		if _focus != Vector2.INF:
			var fd := cell.distance_to(_focus)
			if fd < focus_best_d:
				focus_best_d = fd
				focus_best_name = String(s.get("name", "?"))
				focus_cell = cell
		var best_pt := Vector2.INF
		var best_d_cell := INF
		for ep in endpoints:
			var d: float = cell.distance_to(ep)
			if d < best_d_cell:
				best_d_cell = d
				best_pt = ep
		if best_d_cell > 200.0:
			## No road anywhere near this settlement -- skip, not every
			## settlement has a generated way (isolated/edge cases).
			continue
		if best_d_cell > 2.0:
			far_cell.append("%s cell=%s nearest_endpoint=%s cell_dist=%.2f" % [
				String(s.get("name", "?")), cell, best_pt, best_d_cell])
		var marker_px: Vector2 = xf * ov._cell_to_screen(cell, rect)
		var road_px: Vector2 = xf * ov._point_to_screen(best_pt, rect)
		var dpx := marker_px.distance_to(road_px)
		worst = maxf(worst, dpx)
		sum += dpx
		n += 1
		if n <= 10:
			_p("%s  %s cell=%s endpoint=%s marker_px=%s road_px=%s dist=%.2fpx celldist=%.2f" % [
				tag, String(s.get("name", "?")), cell, best_pt, marker_px.round(), road_px.round(), dpx, best_d_cell])
	_p("%s  ---- n=%d worst=%.2fpx mean=%.2fpx  far(>2cell)=%d ----" % [tag, n, worst, (sum / n) if n > 0 else 0.0, far_cell.size()])
	for f in far_cell:
		_p("%s  FAR>2cell: %s" % [tag, f])

	## The per-SETTLEMENT nearest-any-endpoint search above is masked
	## whenever a settlement has more than one way: the first (busiest) way
	## always lands exactly on the pin, so "nearest endpoint" reports ~0
	## even when a SECOND/THIRD way out of the same settlement lands
	## somewhere else entirely. Check each way's OWN declared endpoints
	## (parsed from its "A -> B" name) against ITS OWN two settlements
	## instead -- this is the check that actually matches the owner's
	## screenshot (several long straight ways meeting at a point with no
	## settlement).
	var by_name := {}
	for s: Dictionary in settlements:
		by_name[String(s.get("name", ""))] = Vector2(float(s["x"]), float(s["y"]))
	var way_far := []
	for r: Dictionary in roads:
		var nm := String(r.get("name", ""))
		if not nm.contains(" → "):
			continue
		var parts := nm.split(" → ")
		if parts.size() != 2 or not by_name.has(parts[0]) or not by_name.has(parts[1]):
			continue
		var pts: PackedVector2Array = r.get("points", PackedVector2Array())
		if pts.size() < 2:
			continue
		var ca: Vector2 = by_name[parts[0]]
		var cb: Vector2 = by_name[parts[1]]
		var p0 := pts[0]
		var p1 := pts[pts.size() - 1]
		var straight := p0.distance_to(ca) + p1.distance_to(cb)
		var crossed := p0.distance_to(cb) + p1.distance_to(ca)
		var d0: float
		var d1: float
		if straight <= crossed:
			d0 = p0.distance_to(ca); d1 = p1.distance_to(cb)
		else:
			d0 = p0.distance_to(cb); d1 = p1.distance_to(ca)
		if d0 > 2.0 or d1 > 2.0:
			way_far.append("%s  way_p0=%s way_p1=%s  d(near-end to its own settlement)=%.2f/%.2f cells" % [
				nm, p0, p1, d0, d1])
	_p("%s  ---- per-way check: %d named ways, %d with an endpoint >2 cells from its OWN settlement ----" % [
		tag, roads.size(), way_far.size()])
	for f in way_far:
		_p("%s  WAY-FAR: %s" % [tag, f])
	if _focus != Vector2.INF and focus_best_name != "":
		_p("%s  FOCUS nearest settlement to %s: %s cell=%s dist_from_focus=%.2f" % [
			tag, _focus, focus_best_name, focus_cell, focus_best_d])
		var img2 := await _grab()
		var marker_f: Vector2 = xf * ov._cell_to_screen(focus_cell, rect)
		_crop(img2, marker_f, "roadsettle_%s_focus_%s" % [tag, focus_best_name.replace(" ", "_")], 220)
	if n > 0:
		var img := await _grab()
		## Crop around the first settlement measured this pass, at its marker.
		var first: Dictionary = settlements[0]
		var cell0 := Vector2(float(first["x"]), float(first["y"]))
		var marker0: Vector2 = xf * ov._cell_to_screen(cell0, rect)
		_crop(img, marker0, "roadsettle_%s_first" % tag, 160)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if _river_density != 1.0:
		_br.param_set("river_density", _river_density)
	if _geology_model:
		_br.param_set("geology_model", true)
	_br.generate({"seed": _seed, "width_km": 40075.0 * GRID.x / 2048.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "metropolis": _metropolis, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	_vh.set_layer_visible("settlements", true)
	_vh.set_layer_visible("roads", true)
	await _view(Vector2(GRID) * 0.5, false)
	await _measure("fit")
	## z16 centred on the first settlement, so the deep-zoom leg measures the
	## same feature the fit-zoom crop shows.
	var settlements: Array = _br.settlements()
	if _focus != Vector2.INF:
		## Deep-zoom on the owner-supplied map coordinate itself (not the
		## nearest settlement's cell) so the crop shows the junction AND the
		## pin together, whichever side of the offset they land on.
		await _view(_focus, true)
		await _measure("z16")
	elif settlements.size() > 0:
		var first: Dictionary = settlements[0]
		await _view(Vector2(float(first["x"]), float(first["y"])), true)
		await _measure("z16")
	_vh.reset_view()
	_p("---- done ----")
	get_tree().quit(0)
