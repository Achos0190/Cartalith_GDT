extends Node
## Rivers and lakes across zoom -- measurement only, no pixel change.
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _riverzoom_probe.tscn -- --out DIR
##
## Per seed, two targets (a river entering a lake, and the widest trunk), a zoom
## ladder from the opening view to deep LOD. At each zoom: one capture with the
## river overlay ON, one with it OFF (every other overlay hidden). Saved as PNGs.
##
## Measured, palette-agnostic:
##  - stroke fragments: the ON-vs-OFF diff mask (the vector strokes alone) split
##    into 8-connected components; "interior" = a component touching no crop
##    edge, i.e. a line that starts and stops inside the frame.
##  - carve offset (grid data, no pixels): distance from each traced D8 cell
##    centre (where `enforce_channel_descent` carves) to the drawn smoothed
##    render polyline, in cells.
##  - carve cross-section: elevation at the traced cell vs +-1..+-4 cells
##    perpendicular (sample_cell's elevation_m).

var _out := "user://riverzoom/"
var _seeds: Array[int] = [483920, 24601, 71077345]
var _grid := Vector2i(1024, 656)
var _vp := Vector2i(1600, 1000)
var _width_km := 1200.0
var _app: Node
var _vh: Control
var _br: Node
var _report: Dictionary = {}

const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--seeds":
				_seeds.clear()
				for s in args[i + 1].split(","): _seeds.append(int(s))
				i += 1
			"--grid":
				var p := args[i + 1].split("x"); _grid = Vector2i(int(p[0]), int(p[1])); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	DisplayServer.window_set_size(_vp)
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	DirAccess.make_dir_recursive_absolute(_out)
	for s in _seeds:
		await _run_seed(s)
	var f := FileAccess.open(_out.path_join("riverzoom.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	print("\nPROBE-RESULT: DONE")
	get_tree().quit(0)


func _run_seed(seed_v: int) -> void:
	print("\n======== seed %d  grid %dx%d ========" % [seed_v, _grid.x, _grid.y])
	_br.generate({"seed": seed_v, "width_km": _width_km, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	var rivers: Array = _br.rivers(1)
	var drawn := 0
	for r: Dictionary in rivers:
		if r.has("width_cells") and not r.has("parallel_of"):
			drawn += 1
	print("  rivers traced %d, drawn %d" % [rivers.size(), drawn])
	var sr := {"traced": rivers.size(), "drawn": drawn, "targets": {}}
	sr["carve"] = _carve_stats(rivers)
	print("  carve: ", sr["carve"])

	sr["channel_lakes"] = _channel_lakes(rivers)
	print("  channel lakes: ", sr["channel_lakes"])
	var targets := _pick_targets(rivers)
	for name: String in targets.keys():
		var t: Dictionary = targets[name]
		print("  target %s at cell %.1f,%.1f (river %d, width_cells %.2f)" % [name, t["p"].x, t["p"].y, t["idx"], t["w"]])
		var rows: Array = []
		for z: float in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 240.0]:
			if z > _vh._zoom_max + 1e-6:
				continue
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(t["p"].x, t["p"].y)
			await _settle(60)
			var on := await _grab()
			_vh.set_layer_visible("rivers", false)
			await _settle(4)
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			await _settle(2)
			if on == null or off == null:
				continue
			var tag := "s%d_%s_z%03d" % [seed_v, name, int(z)]
			on.save_png(_out.path_join(tag + "_on.png"))
			off.save_png(_out.path_join(tag + "_off.png"))
			var frag := _fragments(on, off)
			frag["zoom"] = _vh.zoom()
			frag["lod"] = _vh.lod_active()
			frag["px_per_cell"] = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
			print("    z %6.1f  lod=%s  ppc %.1f  stroke_px %d  comps %d  interior %d  small(<40px) %d"
				% [frag["zoom"], str(frag["lod"]), frag["px_per_cell"], frag["px"], frag["comps"], frag["interior"], frag["small"]])
			rows.append(frag)
		sr["targets"][name] = {"cell": [t["p"].x, t["p"].y], "rows": rows}
	_report[str(seed_v)] = sr


func _pick_targets(rivers: Array) -> Dictionary:
	var out := {}
	var best_w := -1.0
	var best_lake_w := -1.0
	for idx in rivers.size():
		var r: Dictionary = rivers[idx]
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var w: float = r["width_cells"]
		var mid := rp[int(rp.size() * 0.7)] if rp.size() > 0 else Vector2(-1, -1)
		if w > best_w and rp.size() > 8 and _inner(mid):
			best_w = w
			out["trunk"] = {"p": mid, "idx": idx, "w": w}
		var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
		if m.size() == rp.size():
			for k in range(1, m.size() - 20):
				if m[k - 1] == 0 and m[k] == 1 and w > best_lake_w and _inner(rp[k]):
					var run := 0
					while k + run < m.size() and m[k + run] == 1:
						run += 1
					if run >= 20:
						best_lake_w = w
						out["lake"] = {"p": rp[k], "idx": idx, "w": w}
						break
	return out


## Grid-data carve measurement over every drawn river.
func _carve_stats(rivers: Array) -> Dictionary:
	var dists := PackedFloat32Array()
	var depth1 := PackedFloat32Array()   ## centre vs +-1 cell
	var depth3 := PackedFloat32Array()   ## centre vs +-3..4 cells
	var sampled := 0
	for r: Dictionary in rivers:
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		var pts: PackedVector2Array = r["points"]
		var rp: PackedVector2Array = r["render_points"]
		for k in range(1, pts.size() - 1):
			var p := pts[k]
			var best := INF
			for j in range(rp.size() - 1):
				var d := _seg_dist(p, rp[j], rp[j + 1])
				if d < best: best = d
			dists.append(best)
			if sampled < 1500 and k % 3 == 0:
				var dir := (pts[k + 1] - pts[k - 1]).normalized()
				var n := Vector2(-dir.y, dir.x)
				var c := _elev(p)
				var a1 := (_elev(p + n) + _elev(p - n)) * 0.5
				var a3 := (_elev(p + n * 3.0) + _elev(p - n * 3.0) + _elev(p + n * 4.0) + _elev(p - n * 4.0)) * 0.25
				if not is_nan(c) and not is_nan(a1) and not is_nan(a3):
					depth1.append(a1 - c)
					depth3.append(a3 - c)
					sampled += 1
	return {
		"trace_to_line_cells": _stats(dists),
		"depth_vs_1cell_m": _stats(depth1),
		"depth_vs_3to4cell_m": _stats(depth3),
	}


func _inner(p: Vector2) -> bool:
	return p.x > 40 and p.y > 40 and p.x < _grid.x - 40 and p.y < _grid.y - 40


## Does the lake classification bead along carved channels? Walks every drawn
## river's traced D8 cells; counts cells sample_cell calls "lake" and the runs
## they form. Also counts how many pieces the drawn stroke's lake_mask cuts
## each river into (map_overlay.gd::_draw_rivers breaks the stroke there).
func _channel_lakes(rivers: Array) -> Dictionary:
	var cells := 0; var lake := 0; var runs := 0; var short_runs := 0
	var toggles := 0; var split_rivers := 0; var drawn := 0; var pieces_hist := {}
	for r: Dictionary in rivers:
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		drawn += 1
		var pts: PackedVector2Array = r["points"]
		var cur := 0
		for k in pts.size() - 1:   ## last point is the trunk's / sea cell
			var d: Dictionary = _br.sample_cell(int(pts[k].x), int(pts[k].y))
			cells += 1
			if String(d.get("water", "")) == "lake":
				lake += 1; cur += 1
			else:
				if cur > 0:
					runs += 1
					if cur <= 3: short_runs += 1
				cur = 0
		if cur > 0:
			runs += 1
			if cur <= 3: short_runs += 1
		var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
		var t := 0
		for k in range(1, m.size()):
			if m[k] != m[k - 1]: t += 1
		toggles += t
		var pieces := 0
		var inrun := false
		for k in m.size():
			if m[k] == 0 and not inrun: pieces += 1
			inrun = m[k] == 0
		if pieces >= 3: split_rivers += 1
		pieces_hist[pieces] = int(pieces_hist.get(pieces, 0)) + 1
	return {"drawn": drawn, "traced_cells": cells, "lake_cells_on_path": lake, "lake_runs": runs,
		"lake_runs_len_le3": short_runs, "mask_toggles": toggles, "rivers_cut_into_3plus_pieces": split_rivers,
		"pieces_hist": pieces_hist}


func _elev(p: Vector2) -> float:
	var x := int(floor(p.x)); var y := int(floor(p.y))
	if x < 0 or y < 0 or x >= _grid.x or y >= _grid.y:
		return NAN
	var d: Dictionary = _br.sample_cell(x, y)
	return float(d.get("elevation_m", NAN))


func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var ab := b - a
	var l2 := ab.length_squared()
	if l2 <= 0.0:
		return p.distance_to(a)
	var t := clampf((p - a).dot(ab) / l2, 0.0, 1.0)
	return p.distance_to(a + ab * t)


func _stats(v: PackedFloat32Array) -> Dictionary:
	if v.is_empty():
		return {"n": 0}
	var a := Array(v); a.sort()
	return {"n": a.size(), "median": a[a.size() / 2], "p90": a[int(a.size() * 0.9)], "max": a[a.size() - 1]}


func _fragments(on: Image, off: Image) -> Dictionary:
	var w := on.get_width(); var h := on.get_height()
	var da := on.get_data(); var db := off.get_data()
	var mask := PackedByteArray(); mask.resize(w * h)
	var px := 0
	for i in w * h:
		var o := i * 4
		var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
		if dd > 24:
			mask[i] = 1; px += 1
	var comps := 0; var interior := 0; var small := 0
	var stack := PackedInt32Array()
	for s in w * h:
		if mask[s] != 1:
			continue
		comps += 1
		mask[s] = 2
		stack.append(s)
		var n := 0; var edge := false
		while not stack.is_empty():
			var i: int = stack[stack.size() - 1]; stack.remove_at(stack.size() - 1)
			n += 1
			var x := i % w; var y := i / w
			if x == 0 or y == 0 or x == w - 1 or y == h - 1: edge = true
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					var nx: int = x + dx; var ny: int = y + dy
					if nx < 0 or ny < 0 or nx >= w or ny >= h: continue
					var j := ny * w + nx
					if mask[j] == 1:
						mask[j] = 2; stack.append(j)
		if not edge: interior += 1
		if n < 40: small += 1
	return {"px": px, "comps": comps, "interior": interior, "small": small}


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	var gp := _vh.global_position
	var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var sub := img.get_region(r)
	sub.convert(Image.FORMAT_RGBA8)
	return sub


func _settle(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw
