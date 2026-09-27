extends Node
## Rivers and lakes across zoom -- measurement only, no pixel change.
## (Since 2026-09-27 the switch rebuilds the deep-zoom tiles, which draw rivers, so
## the ON/OFF pair below is a re-render of both, and each capture waits for the
## pyramid to settle.)
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
##  - RV-2 stroke data (`_stroke_stats`, grid data, no pixels), readable from
##    both the pre-RV-2 `get_rivers` (`lake_mask`, one `width_cells`) and the
##    RV-2 one (`pieces`, per-point `widths`), so one probe measures both:
##    pieces per drawn river, whether each break has water past it, whether the
##    width ever narrows downstream, confluence gap and width ratio, stroke ends
##    beside water that stop short of it, and headwaters under 1 px at x1.
##  - draw cost: median frame interval at the opening view, rivers on vs off.
##  - RV-1 (`_lake_totals`, grid data): every lake on the map by size, split
##    into channel-shaped trenches and basins, and ocean cells on river paths.
##
## `--targets FILE` reuses the target cells of an earlier run's riverzoom.json,
## so a before/after pair frames the same ground. `--stats-only` skips the zoom
## sweep and its PNGs (grid statistics and draw cost only); `--zooms 1,32`
## narrows the sweep. `--appearance key=v,key=v` sets appearance tunables after
## generation (e.g. `river_ink=1,river_ink_r=0,river_ink_g=0,river_ink_b=0` to
## measure the stroke's continuity in a full-contrast ink, independent of how
## far a style's colour sits from the ground).

var _out := "user://riverzoom/"
var _seeds: Array[int] = [483920, 24601, 71077345]
var _grid := Vector2i(1024, 656)
var _vp := Vector2i(1600, 1000)
var _width_km := 1200.0
var _app: Node
var _vh: Control
var _br: Node
var _report: Dictionary = {}
var _fixed_targets: Dictionary = {}
var _stats_only := false
var _appearance := {}
var _zooms: Array = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 240.0]

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
			"--stats-only": _stats_only = true
			"--zooms":
				_zooms.clear()
				for zs in args[i + 1].split(","): _zooms.append(float(zs))
				i += 1
			"--appearance":
				for kv in args[i + 1].split(","):
					var pr := kv.split("=")
					_appearance[pr[0]] = float(pr[1])
				i += 1
			"--targets":
				_fixed_targets = JSON.parse_string(FileAccess.get_file_as_string(args[i + 1])); i += 1
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
	if not _appearance.is_empty():
		var n: int = _br.set_appearance(_appearance)
		print("  appearance overrides applied: %d of %d %s" % [n, _appearance.size(), str(_appearance)])
		_vh.map_view.texture = _br.color_texture()
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
	sr["lakes"] = _lake_totals(rivers)
	print("  lakes: ", sr["lakes"])
	_vh.reset_view()
	await _settle(3)
	var ppc_fit: float = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
	sr["stroke"] = _stroke_stats(rivers, ppc_fit)
	print("  stroke: ", sr["stroke"])
	## River mouths (owner, 2026-09-27): every free end beside water is carried
	## into it (`river_stroke::extend_shore_ends`). Protects: no drawn end stops
	## short of the sea or lake it meets -- 76-100 per world did before.
	var short_n := int(sr["stroke"]["free_ends_short_of_adjacent_water"])
	if short_n > 0:
		printerr("PROBE-FAIL: %d free ends stop short of adjacent water (seed %d)" % [short_n, seed_v])
		_report["fail"] = true
	else:
		print("  PROBE-OK: no free end stops short of adjacent water")
	sr["draw_cost_ms"] = await _draw_cost()
	print("  draw cost (median frame ms, rivers on/off): ", sr["draw_cost_ms"])
	var targets := {} if _stats_only else _pick_targets(rivers)
	if not _stats_only and _fixed_targets.has(str(seed_v)):
		targets = {}
		var ft: Dictionary = _fixed_targets[str(seed_v)]["targets"]
		for name: String in ft.keys():
			var c: Array = ft[name]["cell"]
			targets[name] = {"p": Vector2(c[0], c[1]), "idx": -1, "w": -1.0}
	for name: String in targets.keys():
		var t: Dictionary = targets[name]
		print("  target %s at cell %.1f,%.1f (river %d, width_cells %.2f)" % [name, t["p"].x, t["p"].y, t["idx"], t["w"]])
		var rows: Array = []
		for z: float in _zooms:
			if z > _vh._zoom_max + 1e-6:
				continue
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(t["p"].x, t["p"].y)
			await _settle(60)
			await _settle_lod()
			var on := await _grab()
			## Since 2026-09-27 the Rivers switch re-renders the base map and
			## rebuilds every deep-zoom tile (the rivers are IN them), so the
			## OFF frame must wait for the pyramid, not four frames: a tile
			## still rebuilding shows its parent, and the diff then reads the
			## whole terrain as "stroke".
			_vh.set_layer_visible("rivers", false)
			await _settle_lod()
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			await _settle_lod()
			if on == null or off == null:
				continue
			var tag := "s%d_%s_z%06.2f" % [seed_v, name, z]
			on.save_png(_out.path_join(tag + "_on.png"))
			off.save_png(_out.path_join(tag + "_off.png"))
			var frag := _fragments(on, off)
			frag["zoom"] = _vh.zoom()
			frag["lod"] = _vh.lod_active()
			frag["px_per_cell"] = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
			print("    z %6.1f  lod=%s  ppc %.1f  stroke_px %d  comps %d  interior %d  small(<40px) %d"
				% [frag["zoom"], str(frag["lod"]), frag["px_per_cell"], frag["px"], frag["comps"], frag["interior"], frag["small"]])
			rows.append(frag)
			## Positive control: the river layer must move pixels. A shell that
			## fails to draw them (a bad bridge call) reads 0 here, and every
			## "fewer fragments" figure would then be vacuous.
			if name == "trunk" and int(frag["px"]) == 0:
				printerr("PROBE-FAIL: rivers drew no pixels at zoom %.1f" % z)
				_report["fail"] = true
		sr["targets"][name] = {"cell": [t["p"].x, t["p"].y], "rows": rows}
	_report[str(seed_v)] = sr


## A river's drawn pieces as `[start, end)` into `render_points`: RV-2's
## `pieces`, or -- for the pre-RV-2 data -- the dry runs of >= 2 points the old
## `_draw_rivers` stroked between `lake_mask` points.
func _pieces_of(r: Dictionary) -> Array[Vector2i]:
	var rp: PackedVector2Array = r["render_points"]
	var out: Array[Vector2i] = []
	if r.has("pieces"):
		var pc: PackedInt32Array = r["pieces"]
		for i in range(0, pc.size() - 1, 2):
			out.append(Vector2i(pc[i], pc[i + 1]))
		return out
	var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
	if m.size() != rp.size():
		out.append(Vector2i(0, rp.size()))
		return out
	var a := -1
	for idx in range(0, rp.size() + 1):
		var dry := idx < rp.size() and m[idx] == 0
		if dry:
			if a < 0:
				a = idx
		elif a >= 0:
			if idx - a >= 2:
				out.append(Vector2i(a, idx))
			a = -1
	return out


func _width_at(r: Dictionary, i: int) -> float:
	if r.has("widths"):
		return float((r["widths"] as PackedFloat32Array)[i])
	return float(r["width_cells"])


func _drawable(r: Dictionary) -> bool:
	return (r.has("widths") or (r.has("width_cells") and not r.has("pieces"))) and not r.has("parallel_of")


func _is_wet(p: Vector2) -> bool:
	var x := int(floor(p.x)); var y := int(floor(p.y))
	if x < 0 or y < 0 or x >= _grid.x or y >= _grid.y:
		return false
	var w := String(_br.sample_cell(x, y).get("water", "land"))
	return w == "lake" or w == "ocean"


func _stroke_stats(rivers: Array, ppc_fit: float) -> Dictionary:
	var drawn := 0; var hist := {}; var r3 := 0; var r2 := 0
	var breaks := 0; var breaks_wet := 0
	var narrows := 0; var narrow_rivers := 0; var const_rivers := 0
	var sub_px := 0
	var ends_on_shore := 0; var ends_short := 0
	## Spatial index of every drawn point: cell -> [river, point index].
	## `ends` marks each river's own first and last drawn point, so a join is
	## matched to the river whose stroke passes THROUGH the end, not to a
	## sibling tributary ending on the same confluence point.
	var grid := {}
	var ends := {}
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var all_pcs := _pieces_of(r)
		if not all_pcs.is_empty():
			ends[Vector2i(ri, all_pcs[0].x)] = true
			ends[Vector2i(ri, all_pcs[all_pcs.size() - 1].y - 1)] = true
		for pc in all_pcs:
			for i in range(pc.x, pc.y):
				var key := Vector2i(int(floor(rp[i].x)), int(floor(rp[i].y)))
				if not grid.has(key): grid[key] = []
				grid[key].append(Vector2i(ri, i))
	var short_gap := PackedFloat32Array()
	var short_where: Array = []
	## Draw rank: RV-2 draws by `get_rivers()`' `draw_rank`; the build before
	## it drew in list order.
	var rank := {}
	for k in rivers.size():
		rank[k] = int((rivers[k] as Dictionary).get("draw_rank", k))
	var continuations := 0; var continuation_narrows := 0; var shared_ends := 0
	var joins := 0; var join_gap_max := 0.0; var join_wider := 0; var join_trunk_drawn_over := 0
	var ratio := PackedFloat32Array()
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		drawn += 1
		var rp: PackedVector2Array = r["render_points"]
		var pcs := _pieces_of(r)
		hist[pcs.size()] = int(hist.get(pcs.size(), 0)) + 1
		if pcs.size() >= 3: r3 += 1
		if pcs.size() >= 2: r2 += 1
		for k in range(pcs.size() - 1):
			breaks += 1
			var e := rp[pcs[k].y - 1]
			var s := rp[pcs[k + 1].x]
			if _is_wet(e + (s - e).normalized() * 0.05):
				breaks_wet += 1
		## Width along the whole drawn stroke, head to mouth.
		var prev := -1.0; var nr := 0; var wmin := INF; var wmax := 0.0
		for pc in pcs:
			for i in range(pc.x, pc.y):
				var wv := _width_at(r, i)
				if prev >= 0.0 and wv < prev - 1e-5: nr += 1
				prev = wv
				wmin = minf(wmin, wv); wmax = maxf(wmax, wv)
		narrows += nr
		if nr > 0: narrow_rivers += 1
		if wmax - wmin < 1e-6: const_rivers += 1
		var mul := 0.55 if int(r.get("own_order", r.get("order", 2))) <= 1 else 1.0
		if wmin * ppc_fit * mul < 1.0: sub_px += 1
		if pcs.is_empty():
			continue
		var last: Vector2i = pcs[pcs.size() - 1]
		if last.y - last.x < 2:
			continue
		var E := rp[last.y - 1]
		var D := (E - rp[last.y - 2]).normalized()
		## Join: another drawn river's stroke passes through E.
		var best := INF; var best_w := 0.0; var best_ri := -1
		var ce := Vector2i(int(floor(E.x)), int(floor(E.y)))
		for dy in [-1, 0, 1]:
			for dx in [-1, 0, 1]:
				for h: Vector2i in grid.get(ce + Vector2i(dx, dy), []):
					if h.x == ri or ends.has(h): continue
					var o: Dictionary = rivers[h.x]
					var orp: PackedVector2Array = o["render_points"]
					for j in [h.y - 1, h.y]:
						if j < 0 or j + 1 >= orp.size(): continue
						var d := _seg_dist(E, orp[j], orp[j + 1])
						if d < best:
							## The joined river's width AT the join: interpolated
							## along the segment, as the stroke is drawn.
							var ab := orp[j + 1] - orp[j]
							var tt := clampf((E - orp[j]).dot(ab) / maxf(ab.length_squared(), 1e-12), 0.0, 1.0)
							best = d; best_w = lerpf(_width_at(o, j), _width_at(o, j + 1), tt); best_ri = h.x
		var o_first: Vector2 = (rivers[best_ri]["render_points"] as PackedVector2Array)[0] if best_ri >= 0 else Vector2.INF
		var o_last: Vector2 = (rivers[best_ri]["render_points"] as PackedVector2Array)[-1] if best_ri >= 0 else Vector2.INF
		if best < 0.05 and E.distance_to(o_last) < 0.05:
			## Two runs ending on one point (both run off the map edge there, or
			## meet end to end): neither is the other's trunk.
			shared_ends += 1
			continue
		if best < 0.05 and E.distance_to(o_first) < 0.05:
			continuations += 1
			if _width_at(r, last.y - 1) > _width_at(rivers[best_ri], 0) + 1e-5: continuation_narrows += 1
			continue
		if best < 0.05:
			joins += 1
			join_gap_max = maxf(join_gap_max, best)
			var tw := _width_at(r, last.y - 1)
			ratio.append(tw / maxf(best_w, 1e-6))
			if tw > best_w + 1e-5: join_wider += 1
			if int(rank[best_ri]) > int(rank[ri]):
				join_trunk_drawn_over += 1
			continue
		## Not a join: does it end on water, or stop short of water beside it?
		if _is_wet(E + D * 0.05):
			ends_on_shore += 1
		else:
			var near_wet := false
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					if _is_wet(Vector2(ce) + Vector2(dx + 0.5, dy + 0.5)): near_wet = true
			if near_wet:
				ends_short += 1
				## How far short: distance from E to the nearest wet cell's square.
				var g := INF
				for dy in [-1, 0, 1]:
					for dx in [-1, 0, 1]:
						var c := Vector2(ce) + Vector2(dx, dy)
						if _is_wet(c + Vector2(0.5, 0.5)):
							var q := Vector2(clampf(E.x, c.x, c.x + 1.0), clampf(E.y, c.y, c.y + 1.0))
							g = minf(g, E.distance_to(q))
				short_gap.append(g)
				## Where, for reading: the end, its direction and piece count.
				if short_where.size() < 8:
					short_where.append([E, D, pcs.size()])
	return {"drawn": drawn, "pieces_hist": hist, "rivers_2plus_pieces": r2, "rivers_3plus_pieces": r3,
		"breaks": breaks, "breaks_with_water_past_them": breaks_wet,
		"width_narrowing_steps": narrows, "rivers_that_narrow": narrow_rivers, "rivers_one_width": const_rivers,
		"rivers_under_1px_at_x1": sub_px,
		"continuations": continuations, "continuations_that_narrow": continuation_narrows, "shared_ends": shared_ends,
		"joins": joins, "join_gap_max_cells": join_gap_max, "joins_trib_wider_than_trunk": join_wider,
		"joins_joined_river_drawn_last": join_trunk_drawn_over, "join_width_ratio": _stats(ratio),
		"free_ends_on_shore": ends_on_shore, "free_ends_short_of_adjacent_water": ends_short,
		"short_end_gap_cells": _stats(short_gap), "short_ends_where": str(short_where)}


## Median frame interval over 30 frames at the opening view, rivers on vs off.
func _draw_cost() -> Dictionary:
	var out := {}
	for on in [true, false]:
		_vh.set_layer_visible("rivers", on)
		await _settle(5)
		var dt := PackedFloat32Array()
		var t0 := Time.get_ticks_usec()
		for f in 30:
			_vh.overlay.queue_redraw()
			await RenderingServer.frame_post_draw
			var t1 := Time.get_ticks_usec()
			dt.append((t1 - t0) / 1000.0)
			t0 = t1
		out["on" if on else "off"] = _stats(dt)["median"]
	_vh.set_layer_visible("rivers", true)
	return out


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
	var cells := 0; var lake := 0; var ocean := 0; var runs := 0; var short_runs := 0
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
			if String(d.get("water", "")) == "ocean":
				ocean += 1
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
	return {"drawn": drawn, "traced_cells": cells, "lake_cells_on_path": lake, "ocean_cells_on_path": ocean, "lake_runs": runs,
		"lake_runs_len_le3": short_runs, "mask_toggles": toggles, "rivers_cut_into_3plus_pieces": split_rivers,
		"pieces_hist": pieces_hist}


## RV-1 (Ruling BD): every lake on the map, not just the ones a river crosses,
## so a carve fix can be seen not to drain the real ones. Lake-class cells from
## `sample_cell` (the drawn classification), split into 4-connected bodies --
## the connectivity `build_water_bodies` itself uses. A body of 10+ cells with
## at least 60% of them within one cell of a traced river cell is counted as
## channel-shaped: a trench the carve sealed, not a basin.
func _lake_totals(rivers: Array) -> Dictionary:
	var n := _grid.x * _grid.y
	var near := PackedByteArray(); near.resize(n)
	for r: Dictionary in rivers:
		for p: Vector2 in (r["points"] as PackedVector2Array):
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					var nx: int = int(p.x) + dx; var ny: int = int(p.y) + dy
					if nx >= 0 and ny >= 0 and nx < _grid.x and ny < _grid.y:
						near[ny * _grid.x + nx] = 1
	var chan := 0; var chan_area := 0; var basin_ge100 := PackedInt32Array()
	var lake := PackedByteArray(); lake.resize(n)
	for y in _grid.y:
		for x in _grid.x:
			if String(_br.sample_cell(x, y).get("water", "land")) == "lake":
				lake[y * _grid.x + x] = 1
	var sizes := PackedInt32Array()
	var stack := PackedInt32Array()
	for s in n:
		if lake[s] != 1:
			continue
		lake[s] = 2
		stack.append(s)
		var k := 0; var kn := 0
		while not stack.is_empty():
			var i: int = stack[stack.size() - 1]; stack.remove_at(stack.size() - 1)
			k += 1
			kn += near[i]
			var x := i % _grid.x; var y := i / _grid.x
			for d: Vector2i in [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]:
				var nx := x + d.x; var ny := y + d.y
				if nx < 0 or ny < 0 or nx >= _grid.x or ny >= _grid.y: continue
				var j := ny * _grid.x + nx
				if lake[j] == 1:
					lake[j] = 2; stack.append(j)
		sizes.append(k)
		if k >= 10 and kn * 10 >= k * 6:
			chan += 1; chan_area += k
		elif k >= 100:
			basin_ge100.append(k)
	var area := 0; var le3 := 0; var ge10 := 0; var ge10_area := 0; var ge100 := 0
	for k in sizes:
		area += k
		if k <= 3: le3 += 1
		if k >= 10: ge10 += 1; ge10_area += k
		if k >= 100: ge100 += 1
	basin_ge100.sort()
	return {"count": sizes.size(), "area_cells": area, "count_le3": le3, "count_ge10": ge10,
		"area_ge10": ge10_area, "count_ge100": ge100, "channel_shaped_ge10": chan,
		"channel_shaped_area": chan_area, "basins_ge100": Array(basin_ge100)}


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


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0` -- a
## condition, not a frame count) and its per-tile morph has run; 20 s cap.
func _settle_lod() -> void:
	await _settle(6)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _settle(4)


func _settle(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw
