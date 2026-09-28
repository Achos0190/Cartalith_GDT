extends Node
## RV-3 (Ruling BD, `LARGE_ITEM_RULINGS.md`): is terrain shaded dark where no
## river is drawn? Measurement only; it changes nothing.
##
## The generation carve (`cartalith_hydrology::carve_channel_network`) cuts
## along the traced D8 cells, which drift up to ~2.5 cells from the smooth
## drawn line (RV-2) and include runs the map never draws. Before RV-3 the
## renderer shaded that stepped trench directly. After RV-3 it shades a
## valley re-cut along the drawn line (`valley_shade.rs`). This probe measures
## the difference on screen pixels.
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _rv3shade_probe.tscn -- --out DIR
##
## Per seed: the river layer is switched OFF (the stroke would otherwise cover
## the ground being measured), every other overlay is hidden, and two targets
## are visited at each zoom in `--zooms`:
##  - `trunk`: the widest drawn river, 70% of the way down;
##  - `undrawn`: the middle of the longest traced run the map does not draw.
## Each frame is saved (`s<seed>_<target>_z<zoom>_off.png`, plus `_on.png`
## with the layer on, for looking).
##
## Every resampled pixel (see "Dark" below) is mapped to its grid position
## (`visible_grid_rect`) and classified from grid data (`get_rivers(1)`):
##  - `on_line`: within the drawn half-width + 0.5 cell of a drawn centreline;
##  - `off_carve`: inside a traced cell (<= 0.9 cell from its centre, which
##    covers the cell and its diagonal corners' neighbourhood) but more than the
##    drawn half-width + 1 cell from every drawn centreline -- the carve that
##    no river covers: drift beside a drawn river, and runs never drawn;
##  - `control`: more than 6 cells from every traced cell and every drawn line.
## Water cells (sample_cell `water` != land) are left out of all three.
##
## "Dark" is a groove, palette-relative: the frame's luma is box-resampled to
## `K_PX_PER_CELL` px per cell, and a sample is dark when its black top-hat
## (a morphological closing with a square `TOPHAT_CELLS` cells in half-width,
## minus the image) exceeds `GROOVE_T` levels -- a dark line narrower than
## about three cells, whatever the style's brightness or the slope's shade.
## The reported figure is each class's dark fraction; `off_carve` minus
## `control` is the excess dark shading off the drawn river. `on_line` is the
## positive control: the valley along the drawn line must stay shaded (a fix
## that flattened every valley would also read "no excess").
##
## Grid statistics (no pixels): traced runs vs drawn runs, traced cells of
## undrawn runs farther than 1.5 cells from every drawn line, and the traced
## cell to drawn line distance for drawn runs (the carve drift).
##
## `--targets FILE` reuses an earlier run's rv3shade.json targets, so a
## before/after pair frames the same ground.

const K_PX_PER_CELL := 3.0 ## the frame is box-resampled to this many px per cell
                           ## (never upsampled), so the element is a ground width
const TOPHAT_CELLS := 1.5  ## structuring-element half-width in cells: a groove up
                           ## to ~3 cells across reads (the carve's disc is 1-9)
const GROOVE_T := 8.0      ## top-hat luma levels that count as a groove. Labelled
                           ## judgement: the faintest stepped line visible in the
                           ## baseline screenshots reads about 10

var _out := "user://rv3shade/"
var _seeds: Array[int] = [483920, 24601, 71077345]
var _grid := Vector2i(1024, 656)
var _vp := Vector2i(1600, 1000)
var _zooms: Array = [2.0, 4.0, 8.0, 16.0, 32.0]
var _app: Node
var _vh: Control
var _br: Node
var _report: Dictionary = {}
var _fixed_targets: Dictionary = {}

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
			"--zooms":
				_zooms.clear()
				for zs in args[i + 1].split(","): _zooms.append(float(zs))
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
	var f := FileAccess.open(_out.path_join("rv3shade.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	print("\nPROBE-RESULT: DONE")
	get_tree().quit(0)


func _run_seed(seed_v: int) -> void:
	print("\n======== seed %d  grid %dx%d ========" % [seed_v, _grid.x, _grid.y])
	_br.generate({"seed": seed_v, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	var rivers: Array = _br.rivers(1)
	var geo := _geometry(rivers)
	var water := _water_grid()
	var sr := {"grid": _grid_stats(rivers, geo), "targets": {}}
	print("  grid: ", sr["grid"])
	var targets := _pick_targets(rivers)
	if _fixed_targets.has(str(seed_v)):
		targets = {}
		var ft: Dictionary = _fixed_targets[str(seed_v)]["targets"]
		for name: String in ft.keys():
			var c: Array = ft[name]["cell"]
			targets[name] = Vector2(c[0], c[1])
	for name: String in targets.keys():
		var p: Vector2 = targets[name]
		var rows: Array = []
		for z: float in _zooms:
			_vh.set_layer_visible("rivers", false)
			## Generated place names are drawn over the terrain and read as
			## dark strokes; they are no layer switch, so they are cleared.
			_vh.overlay.set_labels([])
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(p.x, p.y)
			await _settle(30)
			await _settle_lod()
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			await _settle_lod()
			var on := await _grab()
			_vh.set_layer_visible("rivers", false)
			if off == null or on == null:
				continue
			var tag := "s%d_%s_z%06.2f" % [seed_v, name, z]
			off.save_png(_out.path_join(tag + "_off.png"))
			on.save_png(_out.path_join(tag + "_on.png"))
			var m := _measure(off, geo, water)
			m["zoom"] = z
			m["lod"] = _vh.lod_active()
			if m.has("off_carve") and m.has("control"):
				print("    %-8s z %5.1f lod=%-5s ppc %5.1f  dark: on_line %.3f (n %d)  off_carve %.3f (n %d)  control %.3f (n %d)  excess %+.3f"
					% [name, z, str(m["lod"]), m["ppc"], m["on_line"]["dark"], m["on_line"]["n"],
					m["off_carve"]["dark"], m["off_carve"]["n"], m["control"]["dark"], m["control"]["n"],
					float(m["off_carve"]["dark"]) - float(m["control"]["dark"])])
			else:
				print("    %-8s z %5.1f  (view not inside the map, or a class is empty) %s" % [name, z, str(m)])
			rows.append(m)
		sr["targets"][name] = {"cell": [p.x, p.y], "rows": rows}
	_report[str(seed_v)] = sr


## Drawn centrelines as segments in a cell hash, plus the traced cells of
## every run. A drawn segment carries its two half-widths.
func _geometry(rivers: Array) -> Dictionary:
	var segs := {}          ## Vector2i cell -> Array of [a, b, hwa, hwb]
	var traced := {}        ## Vector2i cell -> true
	for r: Dictionary in rivers:
		for p: Vector2 in (r["points"] as PackedVector2Array):
			traced[Vector2i(int(floor(p.x)), int(floor(p.y)))] = true
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var wd: PackedFloat32Array = r["widths"]
		var pc: PackedInt32Array = r["pieces"]
		for k in range(0, pc.size() - 1, 2):
			for j in range(pc[k], pc[k + 1] - 1):
				var a := rp[j]; var b := rp[j + 1]
				var s := [a, b, wd[j] * 0.5, wd[j + 1] * 0.5]
				var lo := Vector2i(int(floor(minf(a.x, b.x))), int(floor(minf(a.y, b.y))))
				var hi := Vector2i(int(floor(maxf(a.x, b.x))), int(floor(maxf(a.y, b.y))))
				for cy in range(lo.y, hi.y + 1):
					for cx in range(lo.x, hi.x + 1):
						var key := Vector2i(cx, cy)
						if not segs.has(key): segs[key] = []
						segs[key].append(s)
	## Chamfer-free "far from everything" test: cells within 6 of a traced cell
	## or a drawn segment's cell, by dilation of the two sets.
	var near := {}
	for src in [traced, segs]:
		for c: Vector2i in src.keys():
			for dy in range(-6, 7):
				for dx in range(-6, 7):
					near[c + Vector2i(dx, dy)] = true
	return {"segs": segs, "traced": traced, "near": near}


func _drawable(r: Dictionary) -> bool:
	return r.has("widths") and r.has("pieces") and not r.has("parallel_of")


## Per-cell water flag from the classification the map draws.
func _water_grid() -> PackedByteArray:
	var w := PackedByteArray(); w.resize(_grid.x * _grid.y)
	for y in _grid.y:
		for x in _grid.x:
			if String(_br.sample_cell(x, y).get("water", "land")) != "land":
				w[y * _grid.x + x] = 1
	return w


## Distance from `g` to the nearest drawn centreline within 3 cells, minus the
## half-width there (so <= 0 is under the drawn width). INF when none is near.
func _edge_dist(g: Vector2, segs: Dictionary) -> float:
	var best := INF
	var c := Vector2i(int(floor(g.x)), int(floor(g.y)))
	for dy in range(-3, 4):
		for dx in range(-3, 4):
			var lst = segs.get(c + Vector2i(dx, dy))
			if lst == null: continue
			for s: Array in lst:
				var a: Vector2 = s[0]; var b: Vector2 = s[1]
				var ab := b - a
				var l2 := ab.length_squared()
				var t := 0.0 if l2 <= 0.0 else clampf((g - a).dot(ab) / l2, 0.0, 1.0)
				var d := g.distance_to(a + ab * t) - lerpf(float(s[2]), float(s[3]), t)
				if d < best: best = d
	return best


## Distance from `g` to the nearest traced cell centre within 2 cells.
func _trace_dist(g: Vector2, traced: Dictionary) -> float:
	var best := INF
	var c := Vector2i(int(floor(g.x)), int(floor(g.y)))
	for dy in range(-2, 3):
		for dx in range(-2, 3):
			var k := c + Vector2i(dx, dy)
			if traced.has(k):
				best = minf(best, g.distance_to(Vector2(k) + Vector2(0.5, 0.5)))
	return best


func _measure(img: Image, geo: Dictionary, water: PackedByteArray) -> Dictionary:
	var vr: Dictionary = _vh.visible_grid_rect()
	if not vr.get("ok", false):
		return {"skipped": "no visible rect"}
	var w := img.get_width(); var h := img.get_height()
	var x0: float = vr["x0"]; var x1: float = vr["x1"]; var y0: float = vr["y0"]; var y1: float = vr["y1"]
	## The rect is clamped to the map; a view reaching past the map would map
	## pixels wrongly, so it is refused rather than measured.
	if x0 <= 0.0 or y0 <= 0.0 or x1 >= float(_grid.x) or y1 >= float(_grid.y):
		return {"skipped": "view reaches past the map"}
	var ppc := float(w) / (x1 - x0)
	## Luma, resampled (box average) to K px per cell, so one structuring
	## element means the same ground width at every zoom.
	var data := img.get_data()
	var W1 := w + 1
	var integ := PackedFloat64Array(); integ.resize(W1 * (h + 1))
	for y in h:
		var row := 0.0
		for x in w:
			var o := (y * w + x) * 4
			row += 0.299 * data[o] + 0.587 * data[o + 1] + 0.114 * data[o + 2]
			integ[(y + 1) * W1 + x + 1] = integ[y * W1 + x + 1] + row
	var f := maxf(1.0, ppc / K_PX_PER_CELL)
	var dw := int(w / f); var dh := int(h / f)
	var lum := PackedFloat32Array(); lum.resize(dw * dh)
	for y in dh:
		var ay := int(y * f); var by := maxi(ay + 1, int((y + 1) * f))
		for x in dw:
			var ax := int(x * f); var bx := maxi(ax + 1, int((x + 1) * f))
			lum[y * dw + x] = (integ[by * W1 + bx] - integ[ay * W1 + bx] - integ[by * W1 + ax] + integ[ay * W1 + ax]) / float((bx - ax) * (by - ay))
	## Black top-hat: closing (max then min, separable squares) minus the image
	## -- a dark feature narrower than the element, i.e. a groove, reads high.
	var rad := int(ceil(TOPHAT_CELLS * ppc / f))
	var clo := _filter(_filter(_filter(_filter(lum, dw, dh, rad, true, true), dw, dh, rad, false, true), dw, dh, rad, true, false), dw, dh, rad, false, false)
	var cls := {"on_line": [0, 0, 0.0], "off_carve": [0, 0, 0.0], "control": [0, 0, 0.0]}
	for py in dh:
		for px in dw:
			var g := Vector2(x0 + (px + 0.5) / dw * (x1 - x0), y0 + (py + 0.5) / dh * (y1 - y0))
			var ci := Vector2i(int(floor(g.x)), int(floor(g.y)))
			if water[ci.y * _grid.x + ci.x] != 0:
				continue
			var name := ""
			var ed := _edge_dist(g, geo["segs"])
			if ed <= 0.5:
				name = "on_line"
			elif not geo["near"].has(ci):
				name = "control"
			elif ed > 1.0 and _trace_dist(g, geo["traced"]) <= 0.9:
				name = "off_carve"
			else:
				continue
			var th: float = clo[py * dw + px] - lum[py * dw + px]
			var c: Array = cls[name]
			c[0] += 1
			if th > GROOVE_T: c[1] += 1
			c[2] += th
	var out := {"ppc": ppc, "px_per_sample": f, "element_px": rad}
	for k: String in cls.keys():
		var c: Array = cls[k]
		if c[0] > 0:
			out[k] = {"n": c[0], "dark": float(c[1]) / c[0], "mean_tophat": c[2] / c[0]}
	return out


## A separable square max (`dilate`) or min filter of radius `r`, along rows
## (`horiz`) or columns. Clamped at the image edge.
func _filter(src: PackedFloat32Array, w: int, h: int, r: int, horiz: bool, dilate: bool) -> PackedFloat32Array:
	var out := PackedFloat32Array(); out.resize(w * h)
	for y in h:
		for x in w:
			var v := src[y * w + x]
			for k in range(-r, r + 1):
				var xx := x; var yy := y
				if horiz: xx = clampi(x + k, 0, w - 1)
				else: yy = clampi(y + k, 0, h - 1)
				var s := src[yy * w + xx]
				if dilate: v = maxf(v, s)
				else: v = minf(v, s)
			out[y * w + x] = v
	return out


func _grid_stats(rivers: Array, geo: Dictionary) -> Dictionary:
	var traced := 0; var drawn := 0
	var undrawn_cells := 0; var undrawn_off := 0
	var drift := PackedFloat32Array()
	for r: Dictionary in rivers:
		traced += 1
		var pts: PackedVector2Array = r["points"]
		if _drawable(r):
			drawn += 1
			## Traced cell to its drawn centreline -- the carve's drift from
			## the line (RV-2's smoothing moves the line, not the carve).
			for k in range(1, pts.size() - 1):
				drift.append(_centre_dist(pts[k], geo["segs"]))
		else:
			for p: Vector2 in pts:
				undrawn_cells += 1
				if _centre_dist(p, geo["segs"]) > 1.5:
					undrawn_off += 1
	return {"runs_traced": traced, "runs_drawn": drawn, "undrawn_traced_cells": undrawn_cells,
		"undrawn_cells_off_every_drawn_line": undrawn_off, "drawn_trace_to_line_cells": _stats(drift)}


func _centre_dist(g: Vector2, segs: Dictionary) -> float:
	var best := INF
	var c := Vector2i(int(floor(g.x)), int(floor(g.y)))
	for dy in range(-3, 4):
		for dx in range(-3, 4):
			var lst = segs.get(c + Vector2i(dx, dy))
			if lst == null: continue
			for s: Array in lst:
				var a: Vector2 = s[0]; var b: Vector2 = s[1]
				var ab := b - a
				var l2 := ab.length_squared()
				var t := 0.0 if l2 <= 0.0 else clampf((g - a).dot(ab) / l2, 0.0, 1.0)
				best = minf(best, g.distance_to(a + ab * t))
	return best


func _pick_targets(rivers: Array) -> Dictionary:
	var out := {}
	var best_w := -1.0; var best_len := 0
	for r: Dictionary in rivers:
		var pts: PackedVector2Array = r["points"]
		if _drawable(r):
			var rp: PackedVector2Array = r["render_points"]
			var w: float = r.get("width_cells", 0.0)
			var mid := rp[int(rp.size() * 0.7)] if rp.size() > 8 else Vector2(-1, -1)
			if w > best_w and _inner(mid):
				best_w = w; out["trunk"] = mid
		elif pts.size() > best_len and _inner(pts[pts.size() / 2]):
			best_len = pts.size(); out["undrawn"] = pts[pts.size() / 2] + Vector2(0.5, 0.5)
	return out


func _inner(p: Vector2) -> bool:
	return p.x > 60 and p.y > 60 and p.x < _grid.x - 60 and p.y < _grid.y - 60


func _stats(v: PackedFloat32Array) -> Dictionary:
	if v.is_empty():
		return {"n": 0}
	var a := Array(v); a.sort()
	return {"n": a.size(), "median": a[a.size() / 2], "p90": a[int(a.size() * 0.9)], "max": a[a.size() - 1]}


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


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`) and its
## per-tile morph has run; 20 s cap.
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
