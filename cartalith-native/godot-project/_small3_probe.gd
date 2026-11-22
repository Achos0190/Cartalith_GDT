extends Node
## F23 and the ice-streak row (2026-11-22): real-app frames and the numbers their bars read. Windowed only (framebuffer
## reads are dead headless; exits 2 headless). Run: `_small3_probe.tscn -- --mode M --out DIR --tag T [--seed N --grid WxH
## --width-km K --zooms 1,4,16 --vp WxH --force-touch --ablate "base;k=v,k=v;..."]`. Modes:
##   river   the widest river's trunk at each zoom; `river_var` is the luma variance of the frame inside the blue-on-olive water
##           mask eroded 3 px (a solid river reads near 0, mottle reads high); `ring_*` is the land 4..10 px outside it.
##   riverabl  the same at the first `--zooms` entry once per `--ablate` entry (appearance keys through `set_appearance`).
##   ice     the whitest patch at each zoom; `streak` = mean |dI/dx| / mean |dI/dy| over snow pixels (1.0 isotropic).
##   ablate  `streak` over the fit view per `--ablate` entry.
##   dump    elevation_m rows through `sample_cell` and the grid-scale alternation of the app's own field above 2 400 m.
##   all     ice (seed 4242, 256x192, 2 400 km) then river (seed 483920, 1 024x656, 1 200 km, x1/x4/x16/x64): one launch.
## Prints `S3 ...` lines; PROBE-FAIL when a control is empty (no snow, no river interior at x16 or deeper); exit 0 / 1.
var _out := "user://small3/"
var _tag := "t"
var _mode := "river"
var _seed := 4242
var _grid := Vector2i(256, 192)
var _width_km := 2400.0
var _zooms: Array = [1.0, 4.0, 16.0]
var _vp := Vector2i(1600, 900)
var _app: Node
var _vh: Control
var _br: Node
var _fail := false
var _ablate: Array = []   ## each entry "k=v,k=v"; "base" is the shipped look
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes", "landmarks", "landmark_rejects", "urban_layouts", "conflict"]

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-FAIL: needs a window"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--mode": _mode = args[i + 1]; i += 1
			"--out": _out = args[i + 1]; i += 1
			"--tag": _tag = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--grid":
				var p: PackedStringArray = args[i + 1].split("x"); _grid = Vector2i(int(p[0]), int(p[1])); i += 1
			"--width-km": _width_km = float(args[i + 1]); i += 1
			"--zooms":
				_zooms = []
				for s in args[i + 1].split(","): _zooms.append(float(s))
				i += 1
			"--vp":
				var q: PackedStringArray = args[i + 1].split("x"); _vp = Vector2i(int(q[0]), int(q[1])); i += 1
			"--ablate":
				_ablate = Array(args[i + 1].split(";")); i += 1
			"--force-touch": pass
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
	if _mode == "all":
		# One launch (one window-lock turn) for both rows: the ice world, then the river world.
		await _generate()
		await _ice()
		_seed = 483920; _grid = Vector2i(1024, 656); _width_km = 1200.0; _zooms = [1.0, 4.0, 16.0, 64.0]
		await _generate()
		await _river()
		print("
PROBE-RESULT: DONE")
		get_tree().quit(1 if _fail else 0)
		return
	await _generate()
	if _mode == "dump":
		var el := PackedFloat32Array(); el.resize(_grid.x * _grid.y)
		for y in _grid.y:
			for x in _grid.x:
				el[y * _grid.x + x] = float(_br.sample_cell(x, y).get("elevation_m", -1e9))
		var thr := 2400.0
		var d2x := 0.0; var d1x := 0.0; var d2y := 0.0; var d1y := 0.0; var nn := 0
		for y in range(1, _grid.y - 1):
			for x in range(1, _grid.x - 1):
				var ok := true
				for dy in range(-1, 2):
					for dx in range(-1, 2):
						if el[(y + dy) * _grid.x + x + dx] <= thr: ok = false
				if ok:
					var c := el[y * _grid.x + x]
					d2x += absf(el[y * _grid.x + x - 1] - 2.0 * c + el[y * _grid.x + x + 1]); d1x += absf(el[y * _grid.x + x + 1] - c)
					d2y += absf(el[(y - 1) * _grid.x + x] - 2.0 * c + el[(y + 1) * _grid.x + x]); d1y += absf(el[(y + 1) * _grid.x + x] - c)
					nn += 1
		print("S3 alternation along_x=%.3f along_y=%.3f cells=%d (elevation_m > %d)" % [d2x / maxf(d1x, 1e-9), d2y / maxf(d1y, 1e-9), nn, int(thr)])
		for y in [80, 84, 88, 92]:
			var row := ""
			for x in range(160, 200):
				var d: Dictionary = _br.sample_cell(x, y)
				row += "%d " % int(d.get("elevation_m", -1))
			print("S3 dump y=%d %s" % [y, row])
		for y in [84]:
			var row2 := ""
			for x in range(160, 200):
				var d2: Dictionary = _br.sample_cell(x, y)
				row2 += "%.1f " % float(d2.get("temperature_c", -99))
			print("S3 temp y=%d %s" % [y, row2])
	elif _mode == "ablate":
		await _ablate_run()
	elif _mode == "riverabl":
		await _river_ablate()
	elif _mode == "river":
		await _river()
	else:
		await _ice()
	print("\nPROBE-RESULT: DONE")
	get_tree().quit(1 if _fail else 0)

func _generate() -> void:
	_br.generate({"seed": _seed, "width_km": _width_km, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": false, "metropolis": false, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var gp := _vh.global_position
	var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var sub := img.get_region(r)
	sub.convert(Image.FORMAT_RGBA8)
	return sub

func _settle(n: int) -> void:
	for k in n:
		await RenderingServer.frame_post_draw

func _settle_lod() -> void:
	await _settle(6)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 30000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _settle(4)

func _goto(z: float, c: Vector2) -> void:
	_vh.reset_view()
	await _settle(3)
	_vh.zoom_step(z / _vh.zoom())
	_vh.move_view_to(c.x, c.y)
	await _settle(30)
	await _settle_lod()

func _luma(img: Image, x: int, y: int) -> float:
	var c := img.get_pixel(x, y)
	return (0.299 * c.r + 0.587 * c.g + 0.114 * c.b) * 255.0

func _river_target() -> Vector2:
	var best_w := -1.0
	var mid := Vector2.ZERO
	for r: Dictionary in _br.rivers(1):
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var w: float = r["width_cells"]
		if rp.size() > 8 and w > best_w:
			var m := rp[int(rp.size() * 0.7)]
			if m.x > 20 and m.y > 20 and m.x < _grid.x - 20 and m.y < _grid.y - 20:
				best_w = w; mid = m
	print("S3 river target %s width_cells %.2f" % [str(mid), best_w])
	if best_w < 0.0:
		_fail = true
	return mid

## One river frame at zoom `z` on `mid` (view already there): ON vs OFF diff mask, its 3 px erosion (`river_var`:
## luma variance of the ON frame inside) and the land ring 4..10 px outside it (`ring_mean`, `ring_var`).
func _river_measure(label: String, z: float, save: String) -> void:
	var on := await _grab()
	if save != "":
		on.save_png(_out.path_join(save))
	var w := on.get_width(); var h := on.get_height()
	# Water mask by colour (the river is blue on olive ground; hiding the Rivers layer does not rebuild the deep
	# tiles' river, so the ON/OFF diff the first draft used read an empty mask): blue above red and above green.
	var mask := PackedByteArray(); mask.resize(w * h)
	var n_mask := 0
	for y in h:
		for x in w:
			var a := on.get_pixel(x, y)
			if (a.b - a.r) * 255.0 > 25.0 and a.b > a.g:
				mask[y * w + x] = 1; n_mask += 1
	var s := 0.0; var s2 := 0.0; var n := 0
	var rs := 0.0; var rs2 := 0.0; var rn := 0
	for y in range(10, h - 10, 1):
		for x in range(10, w - 10, 1):
			var m := mask[y * w + x]
			if m == 1:
				var ok := true
				for dy in range(-3, 4):
					for dx in range(-3, 4):
						if mask[(y + dy) * w + x + dx] == 0: ok = false; break
					if not ok: break
				if ok:
					var l := _luma(on, x, y); s += l; s2 += l * l; n += 1
			elif (x % 2 == 0) and (y % 2 == 0):
				# ring: not in the mask, but a mask pixel within 10 px along x or y, and none within 3
				var near10 := false; var near3 := false
				for d in range(1, 11):
					if mask[y * w + x + d] == 1 or mask[y * w + x - d] == 1 or mask[(y + d) * w + x] == 1 or mask[(y - d) * w + x] == 1:
						near10 = true
						if d <= 3: near3 = true
						break
				if near10 and not near3:
					var l2 := _luma(on, x, y); rs += l2; rs2 += l2 * l2; rn += 1
	var v := (s2 / n - (s / n) * (s / n)) if n > 0 else -1.0
	var rv := (rs2 / rn - (rs / rn) * (rs / rn)) if rn > 0 else -1.0
	print("S3 mode=river %s z=%d mask_px=%d interior_px=%d river_var=%.3f river_mean=%.2f ring_px=%d ring_mean=%.3f ring_var=%.3f" % [label, int(z), n_mask, n, v, (s / maxf(n, 1)), rn, rs / maxf(rn, 1), rv])
	if n < 200:
		if z >= 16.0:
			printerr("PROBE-FAIL: no river interior at z %d (%s)" % [int(z), label]); _fail = true

func _river() -> void:
	var mid := _river_target()
	if _fail: return
	for z: float in _zooms:
		await _goto(z, mid)
		await _river_measure("shipped", z, "%s_river_z%02d.png" % [_tag, int(z)])

func _river_ablate() -> void:
	var ap: Dictionary = _br.world_gen.get_appearance()
	for k in ap.keys():
		if str(k).begins_with("river") or str(k).begins_with("zoom_detail") or str(k).begins_with("local_contrast") or str(k).begins_with("floodplain"):
			print("S3 appearance %s = %s" % [str(k), str(ap[k])])
	var mid := _river_target()
	if _fail: return
	var z: float = _zooms[0]
	for spec: String in _ablate:
		_br.reset_appearance()
		if spec != "base":
			var d := {}
			for kv in spec.split(","):
				var p := kv.split("="); d[p[0]] = float(p[1])
			_br.set_appearance(d)
		_vh.map_view.texture = _br.color_texture()
		await _goto(z, mid)
		await _river_measure(spec, z, "%s_rabl_%s.png" % [_tag, spec.replace("=", "").replace(",", "_")])

func _snow(c: Color) -> bool:
	var mx := maxf(c.r, maxf(c.g, c.b)) * 255.0
	var mn := minf(c.r, minf(c.g, c.b)) * 255.0
	return mn > 196.0 and mx - mn < 40.0

func _ice() -> void:
	await _goto(1.0, Vector2(_grid.x * 0.5, _grid.y * 0.5))
	var f := await _grab()
	var w := f.get_width(); var h := f.get_height()
	var ppc := minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y))
	var best := -1; var bp := Vector2i.ZERO
	for y in range(40, h - 40, 16):
		for x in range(40, w - 40, 16):
			var c := 0
			for dy in range(-40, 41, 4):
				for dx in range(-40, 41, 4):
					if _snow(f.get_pixel(x + dx, y + dy)): c += 1
			if c > best: best = c; bp = Vector2i(x, y)
	var cell := (Vector2(bp) - Vector2(w, h) * 0.5) / ppc + Vector2(_grid) * 0.5
	print("S3 ice target px %s cell %s snow_samples %d" % [str(bp), str(cell), best])
	if best < 50:
		printerr("PROBE-FAIL: no snow patch"); _fail = true; return
	for z: float in _zooms:
		await _goto(z, cell)
		var img := await _grab()
		img.save_png(_out.path_join("%s_ice_z%02d.png" % [_tag, int(z)]))
		var w2 := img.get_width(); var h2 := img.get_height()
		var sx := 0.0; var sy := 0.0; var n := 0
		for y in range(1, h2 - 1):
			for x in range(1, w2 - 1):
				var c := img.get_pixel(x, y)
				if _snow(c) and _snow(img.get_pixel(x + 1, y)) and _snow(img.get_pixel(x, y + 1)):
					var l := _luma(img, x, y)
					sx += absf(_luma(img, x + 1, y) - l); sy += absf(_luma(img, x, y + 1) - l); n += 1
		var ratio := (sx / maxf(sy, 1e-6)) if n > 0 else -1.0
		print("S3 mode=ice z=%d snow_px=%d mean_dx=%.4f mean_dy=%.4f streak=%.3f" % [int(z), n, sx / maxf(n, 1), sy / maxf(n, 1), ratio])
		if n < 500:
			printerr("PROBE-FAIL: too little snow at z %d" % int(z)); _fail = true

func _ablate_run() -> void:
	await _goto(1.0, Vector2(_grid.x * 0.5, _grid.y * 0.5))
	for spec: String in _ablate:
		_br.reset_appearance()
		var d := {}
		if spec != "base":
			for kv in spec.split(","):
				var p := kv.split("=")
				d[p[0]] = float(p[1])
			_br.set_appearance(d)
		_vh.map_view.texture = _br.color_texture()
		await _settle(30)
		await _settle_lod()
		var img := await _grab()
		img.save_png(_out.path_join("%s_abl_%s.png" % [_tag, spec.replace("=", "").replace(",", "_")]))
		var w2 := img.get_width(); var h2 := img.get_height()
		var sx := 0.0; var sy := 0.0; var n := 0
		for y in range(1, h2 - 1):
			for x in range(1, w2 - 1):
				if _snow(img.get_pixel(x, y)) and _snow(img.get_pixel(x + 1, y)) and _snow(img.get_pixel(x, y + 1)):
					var l := _luma(img, x, y)
					sx += absf(_luma(img, x + 1, y) - l); sy += absf(_luma(img, x, y + 1) - l); n += 1
		print("S3 abl %s snow_px=%d dx=%.4f dy=%.4f streak=%.3f" % [spec, n, sx / maxf(n, 1), sy / maxf(n, 1), (sx / maxf(sy, 1e-6)) if n > 0 else -1.0])
