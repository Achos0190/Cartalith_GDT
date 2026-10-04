extends Node
## River colour continuity along one course -- measurement only.
##
## The OUTSTANDING_WORK.md row "Rivers change colour abruptly mid-course"
## (found 2026-09-27): a river's colour jumped between palette entries where
## its Strahler order changed, and a run carried on past a land pit onto the
## head of the next run (a `river_draw_plan` continuation) restarted at the
## headwater colour and the order-1 de-emphasis alpha. This probe measures
## both on the drawn data and on the screen.
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _rivcolour_probe.tscn -- --out DIR
## Options: --seeds a,b  --grid WxH  --zooms 1,4  --targets DIR/rivcolour.json
## (`--targets` reuses an earlier run's target cells, so a before/after pair
## frames the same ground).
## `--stroke-path` runs the SCREEN walk on the vector-stroke path instead of the
## default painted one (RIM-1/6, `WorldGen::debug_set_river_paint(false)` --
## a diagnostic switch, never a setting): the river's colour is then the
## overlay's own, not `map_shore.gdshader`'s sample of the colour field, so a
## painted/stroke pair of runs over the same `--targets` says whether painting
## the river into the map moved the colour continuity this probe polices. The
## report's `path` field names which path a run measured.
##
## Measured:
##  - DATA (`get_rivers(1)`, grid space, palette-agnostic):
##    * `abrupt`: consecutive drawn render points of one piece whose colours
##      differ by >= ABRUPT_LEVELS in any channel. A palette step is 26-46
##      levels (`RIVER_ORDER_RGB`); render points sit <= ~1 cell apart, so a
##      step drawn between two of them is a jump inside one cell.
##    * `cont_colour_jumps`: continuations (a run whose last render point is
##      another run's first) whose end/head colours differ by >= ABRUPT_LEVELS.
##    * `cont_alpha_mismatch`: continuations where exactly one side is an
##      order-1 run (`own_order <= 1`), i.e. drawn at the de-emphasis alpha.
##    Positive control: `drawn` rivers and `order_changes` (runs whose head
##    and mouth colours differ by >= ABRUPT_LEVELS, i.e. whose order rises
##    along the course, before or after any blend) must be > 0, or the counts
##    above are vacuous.
##  - SCREEN: at each target and zoom, the frame with rivers ON is saved; the
##    target river's centreline is sampled every 1 cell for +-TARGET_SPAN cells
##    around the target and the largest colour change between consecutive
##    samples is reported (`max_step`), beside the same walk over the rivers-OFF
##    frame (`max_step_off`, the ground's own variation -- the control). Only
##    stroke pixels are sampled (see `_walk`).
##    With no defect to target (the healthy case since 2026-09-28) the walk
##    runs on a `course` control target, the longest drawn river's midpoint.

var _out := "user://rivcolour/"
var _seeds: Array[int] = [483920, 24601, 1]
var _grid := Vector2i(1024, 656)
var _vp := Vector2i(1600, 1000)
var _zooms: Array = [1.0, 4.0]
var _fixed := {}
var _stroke_path := false
var _app: Node
var _vh: Control
var _br: Node
var _report := {}

## Levels (0-255) in one channel that count as a visible jump -- labelled
## judgement: under the smallest palette step (26 levels, orders 7->8), and
## above what a blend over `river_stroke::colour_ramp_cells` draws between two
## render points (measured 9-17 levels at most, 2026-09-28).
const ABRUPT_LEVELS := 15.0
## Cells either side of a target the screen walk covers -- labelled judgement,
## wide enough to hold a whole colour ramp either side of the step.
const TARGET_SPAN := 14
## Levels by which a pixel must differ between the rivers-on and rivers-off
## frames to count as stroke -- `_riverzoom_probe.gd`'s own diff threshold (24).
const STROKE_LEVELS := 24.0
## Cells around a drawn river end the screen walk skips (see `_walk`) --
## labelled judgement: a cap of the widest trunk here (~3 cells) plus the
## texture's one-texel linear blend.
const END_CELLS := 2.5
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
			"--zooms":
				_zooms.clear()
				for zs in args[i + 1].split(","): _zooms.append(float(zs))
				i += 1
			"--targets":
				_fixed = JSON.parse_string(FileAccess.get_file_as_string(args[i + 1])); i += 1
			"--stroke-path": _stroke_path = true
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
	var f := FileAccess.open(_out.path_join("rivcolour.json"), FileAccess.WRITE)
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
	if _stroke_path:
		## The appearance flag only changes what the NEXT colour render draws:
		## swap the texture in the way a style change does.
		_br.world_gen.debug_set_river_paint(false)
		_vh.map_view.texture = _br.color_texture()
		_vh._apply_shore_field()
		await _settle(8)
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	var rivers: Array = _br.rivers(1)
	var sr := _data_stats(rivers)
	sr["path"] = "paint" if _br.world_gen.rivers_painted() else "stroke"
	print("  river path measured: ", sr["path"])
	if _stroke_path == _br.world_gen.rivers_painted():
		printerr("PROBE-FAIL: --stroke-path=%s but rivers_painted()=%s (the switch did not take, or the field never built)" % [_stroke_path, _br.world_gen.rivers_painted()])
	print("  data: ", JSON.stringify(sr["summary"]))
	if int(sr["summary"]["drawn"]) == 0 or int(sr["summary"]["order_changes"]) == 0:
		printerr("PROBE-FAIL: positive control -- no drawn rivers or no order change (seed %d)" % seed_v)
	var targets: Dictionary = sr["targets"]
	if _fixed.has(str(seed_v)):
		targets = _fixed[str(seed_v)]["targets"]
	sr.erase("targets")
	sr["targets"] = targets
	sr["screen"] = {}
	for name: String in targets.keys():
		var t: Dictionary = targets[name]
		var cell := Vector2(t["cell"][0], t["cell"][1])
		var ri := _river_through(rivers, cell)
		for z: float in _zooms:
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(cell.x, cell.y)
			await _settle_lod()
			var on := await _grab()
			_vh.set_layer_visible("rivers", false)
			await _settle_lod()
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			await _settle_lod()
			if on == null or off == null:
				continue
			var tag := "s%d_%s_z%05.1f" % [seed_v, name, z]
			on.save_png(_out.path_join(tag + "_on.png"))
			var c := _to_screen(cell)
			var crop := Rect2i(Vector2i(c) - Vector2i(160, 100), Vector2i(320, 200)).intersection(Rect2i(Vector2i.ZERO, on.get_size()))
			var ci := on.get_region(crop)
			ci.resize(crop.size.x * 3, crop.size.y * 3, Image.INTERPOLATE_NEAREST)
			ci.save_png(_out.path_join(tag + "_crop3x.png"))
			var ms := _walk(on, off, rivers, ri, cell)
			var where := _step_at
			var row := {"zoom": _vh.zoom(), "lod": _vh.lod_active(), "river": ri,
				"max_step": ms, "max_step_at": [where.x, where.y],
				"max_step_off": _walk(off, on, rivers, ri, cell)}
			print("    %s z %.1f  river %d  max_step %.1f at (%.1f, %.1f)  (off %.1f)" % [name, z, ri, ms,
				where.x, where.y, row["max_step_off"]])
			sr["screen"][tag] = row
	_report[str(seed_v)] = sr
	if _stroke_path:
		## Leave the diagnostic switch as found for the next seed's world.
		_br.world_gen.debug_set_river_paint(true)


func _near_any(q: Vector2, pts: PackedVector2Array) -> bool:
	for p in pts:
		if p.distance_to(q) <= END_CELLS:
			return true
	return false


func _drawable(r: Dictionary) -> bool:
	return r.has("widths") and r.has("colors") and not r.has("parallel_of")


func _pieces(r: Dictionary) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var pc: PackedInt32Array = r.get("pieces", PackedInt32Array())
	for i in range(0, pc.size() - 1, 2):
		out.append(Vector2i(pc[i], pc[i + 1]))
	return out


func _dc(a: Color, b: Color) -> float:
	return 255.0 * maxf(absf(a.r - b.r), maxf(absf(a.g - b.g), absf(a.b - b.b)))


func _data_stats(rivers: Array) -> Dictionary:
	var drawn := 0; var abrupt := 0; var order_changes := 0; var max_dc := 0.0
	var worst_abrupt := {}; var worst_cont := {}
	var heads := {}
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		drawn += 1
		var rp: PackedVector2Array = r["render_points"]
		var cols: PackedColorArray = r["colors"]
		if rp.size() > 0:
			heads[Vector2i(rp[0].floor())] = ri
			if _dc(cols[0], cols[rp.size() - 1]) >= ABRUPT_LEVELS:
				order_changes += 1
		for pc in _pieces(r):
			for i in range(pc.x + 1, pc.y):
				var d := _dc(cols[i - 1], cols[i])
				max_dc = maxf(max_dc, d)
				if d >= ABRUPT_LEVELS:
					abrupt += 1
					if d > float(worst_abrupt.get("dc", -1.0)) and _inner(rp[i]):
						worst_abrupt = {"dc": d, "cell": [rp[i].x, rp[i].y], "river": ri}
	var conts := 0; var cjump := 0; var amis := 0
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		if rp.size() < 2:
			continue
		var e := rp[rp.size() - 1]
		var j: int = heads.get(Vector2i(e.floor()), -1)
		if j < 0 or j == ri:
			continue
		var o: Dictionary = rivers[j]
		var orp: PackedVector2Array = o["render_points"]
		if orp.is_empty() or orp[0].distance_to(e) > 0.75:
			continue
		conts += 1
		var d := _dc((r["colors"] as PackedColorArray)[rp.size() - 1], (o["colors"] as PackedColorArray)[0])
		var lo_a := int(r.get("own_order", 2)) <= 1
		var lo_b := int(o.get("own_order", 2)) <= 1
		if d >= ABRUPT_LEVELS:
			cjump += 1
		if lo_a != lo_b:
			amis += 1
		var score := d + (100.0 if lo_a != lo_b else 0.0)
		if score > float(worst_cont.get("score", -1.0)) and _inner(e) and (d >= ABRUPT_LEVELS or lo_a != lo_b):
			worst_cont = {"score": score, "dc": d, "cell": [e.x, e.y], "river": ri, "into": j}
	var targets := {}
	if not worst_abrupt.is_empty():
		targets["order_step"] = worst_abrupt
	if not worst_cont.is_empty():
		targets["continuation"] = worst_cont
	## The two defects above are fixed (2026-09-28), so a healthy world gives no
	## target at all and the screen walk never ran -- which left the painted
	## river (RIM-1) unmeasured here. A control target (the longest drawn
	## river's midpoint, `_longest_course`) keeps the screen walk running on a
	## fixed river whatever the data finds; it is never a defect.
	if targets.is_empty():
		var lc := _longest_course(rivers)
		if not lc.is_empty():
			targets["course"] = lc
	return {"summary": {"drawn": drawn, "order_changes": order_changes, "abrupt": abrupt, "max_dc": max_dc,
		"continuations": conts, "cont_colour_jumps": cjump, "cont_alpha_mismatch": amis}, "targets": targets}


## The inner-area midpoint of the longest drawable river, as a target dict
## (`{"cell", "river"}`), or `{}` when there is none -- never a made-up cell.
func _longest_course(rivers: Array) -> Dictionary:
	var best := {}
	var best_n := 0
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		if rp.size() < 2:
			continue
		var m := rp[rp.size() / 2]
		if rp.size() > best_n and _inner(m):
			best_n = rp.size()
			best = {"cell": [m.x, m.y], "river": ri}
	return best


func _inner(p: Vector2) -> bool:
	return p.x > 40 and p.y > 40 and p.x < _grid.x - 40 and p.y < _grid.y - 40


## The drawn river whose render polyline passes nearest `cell` (-1: none).
func _river_through(rivers: Array, cell: Vector2) -> int:
	var best := INF; var bi := -1
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		for p: Vector2 in r["render_points"]:
			var d := p.distance_to(cell)
			if d < best:
				best = d; bi = ri
	return bi


## River space (a cell's centre at x + 0.5) to this frame's pixels:
## `viewport_host.gd::move_view_to`'s own mapping.
func _to_screen(p: Vector2) -> Vector2:
	var rect: Rect2 = _vh.overlay.displayed_rect()
	var local := rect.position + Vector2(p.x / float(_grid.x), p.y / float(_grid.y)) * rect.size
	return local * _vh.zoom() + _vh._camera.position


## Where the last `_walk` found its largest step (river space), for the
## report.
var _step_at := Vector2(-1, -1)


## Largest colour change between consecutive 1-cell samples of river `ri`'s
## centreline within TARGET_SPAN cells of `cell`, including any continuation
## run beginning where it ends (so a course is walked across the bridge).
## Only samples where `img` and `other` (the same frame with the rivers
## toggled) differ by >= STROKE_LEVELS are kept -- pixels the stroke actually
## covers -- so a sample on bare ground beside a thin stroke, or under a
## label, is not read as a colour step; a step is only taken between kept
## samples at most 2 cells apart. Samples within END_CELLS of any drawn
## river's end are skipped, except the bridge this walk crosses: there a
## stroke's round cap blends with the ground, and a tributary meets its trunk
## in the trunk's colour -- which the hierarchy keeps, and is not the defect.
func _walk(img: Image, other: Image, rivers: Array, ri: int, cell: Vector2) -> float:
	_step_at = Vector2(-1, -1)
	if ri < 0:
		return -1.0
	var pts := PackedVector2Array()
	pts.append_array(rivers[ri]["render_points"])
	if pts.is_empty():
		return -1.0
	var e := pts[pts.size() - 1]
	var bridged := false
	for o: Dictionary in rivers:
		var orp: PackedVector2Array = o.get("render_points", PackedVector2Array())
		if _drawable(o) and o != rivers[ri] and not orp.is_empty() and orp[0].distance_to(e) < 0.75:
			pts.append_array(o["render_points"])
			bridged = true
			break
	var ends := PackedVector2Array()
	for o: Dictionary in rivers:
		var orp: PackedVector2Array = o.get("render_points", PackedVector2Array())
		if not _drawable(o) or orp.is_empty():
			continue
		for p: Vector2 in [orp[0], orp[orp.size() - 1]]:
			if not (bridged and p.distance_to(e) < 0.75) and p.distance_to(cell) < TARGET_SPAN + END_CELLS + 2.0:
				ends.append(p)
	var samples: Array[Color] = []
	var at: Array[float] = []
	var where: Array[Vector2] = []
	var walked := 0.0
	var carry := 0.0
	for i in range(1, pts.size()):
		var a := pts[i - 1]; var b := pts[i]
		var L := a.distance_to(b)
		var t := carry
		while t < L:
			var q := a.lerp(b, t / maxf(L, 1e-6))
			if q.distance_to(cell) <= TARGET_SPAN and not _near_any(q, ends):
				var s := _to_screen(q)
				var x := int(s.x); var y := int(s.y)
				if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height() \
						and _dc(img.get_pixel(x, y), other.get_pixel(x, y)) >= STROKE_LEVELS:
					samples.append(img.get_pixel(x, y))
					at.append(walked + t)
					where.append(q)
			t += 1.0
		carry = t - L
		walked += L
	var m := 0.0
	for i in range(1, samples.size()):
		if at[i] - at[i - 1] <= 2.01:
			var d := _dc(samples[i - 1], samples[i])
			if d > m:
				m = d
				_step_at = where[i]
	return m


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
