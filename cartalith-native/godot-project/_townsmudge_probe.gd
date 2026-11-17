extends Node
## The town layer's first zoom (owner row "Town layer's first zoom draws a dark teardrop smudge round the town pin", found by
## `872fe397`'s map-level probe; 2026-11-17): at the zoom where `map_overlay.gd`'s town layer first draws a town, the town read
## as a dark blob round its pin. Windowed only (the frames are the evidence; refuses --headless). Run under a scratch APPDATA.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _townsmudge_probe.tscn -- [--seed N] [--vp WxH] [--out <dir>] [--tag s]
##       [--towns 3] [--ids 4,7] [--ablate] [--poi well] [--force-touch --density 2.625]
##
## For each of `--towns` towns (the first not in the plate's frame margin, one per kind where possible) it zooms in on the town
## until `_urban_draw_list` holds it, then measures DARK, the count of near-black pixels in a square round the town's pin, in
## five frames: below (one step before the layer), appears (after the crossfade settles), x2, x4 and the City Viewer's own
## canvas is left to its own probes. Bars (each printed PASS/FAIL):
##   T1 below the layer's first zoom there is no dark mass (DARK <= DARK_MAX)
##   T2 at the zoom the layer appears, with the fade settled, there is none either
##   T3 at x2 and x4 there is none either
##   T4 the stroke floor a landmark or placed pin passes (`_pin_line_floor`) is one screen pixel, whatever the zoom
## `--poi KIND` places a point of interest 1.05 km from each town first (what `872fe397`'s probe did, which is where the mass came from).
## `--ablate` additionally erases each layout key in turn at the appearing zoom and prints the DARK count, which names the
## drawing responsible (a diagnostic, no bar). Exit 0 held, 1 failed, 2 could not run.

const DARK_LUMA := 0.13
const REGION_PX := 150      ## half side of the measured square, viewport pixels
## A pin's own ink (outline, glyph) and a label are a few hundred dark pixels; a smudge is thousands. Labelled judgement.
const DARK_MAX := 900
## The town layer may darken the square round the town to at most this share of the bare terrain's mean luminance (the same
## view with the layer switched off). Measured (this PC, 2026-11-17, six towns of seeds 912345 and 483920 at x2 and x4):
## 0.41 to 0.90 with the gate stroke unfixed (mutant `maxf(px_floor, 1.2)`; 4 of 6 towns under 0.80 at x4), 0.98 to 1.07 fixed.
## 0.80 sits between them, a labelled judgement.
const LUM_MIN_RATIO := 0.80

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

func _global_of(overlay: Control, cell: Vector2) -> Vector2:
	var rect: Rect2 = overlay._displayed_rect()
	return overlay.get_global_transform() * overlay._point_to_screen(cell, rect)

func _grab(vp: SubViewport) -> Image:
	await RenderingServer.frame_post_draw
	return vp.get_texture().get_image()

func _dark(img: Image, c: Vector2) -> int:
	var n := 0
	var x0 := maxi(0, int(c.x) - REGION_PX)
	var y0 := maxi(0, int(c.y) - REGION_PX)
	var x1 := mini(img.get_width(), int(c.x) + REGION_PX)
	var y1 := mini(img.get_height(), int(c.y) + REGION_PX)
	for y in range(y0, y1):
		for x in range(x0, x1):
			if img.get_pixel(x, y).get_luminance() < DARK_LUMA:
				n += 1
	return n

## Mean luminance of the square round `c` (a darkness measure that a brown fill moves and a near-black count does not).
func _mean_lum(img: Image, c: Vector2) -> float:
	var x0 := maxi(0, int(c.x) - REGION_PX)
	var y0 := maxi(0, int(c.y) - REGION_PX)
	var x1 := mini(img.get_width(), int(c.x) + REGION_PX)
	var y1 := mini(img.get_height(), int(c.y) + REGION_PX)
	var t := 0.0
	for y in range(y0, y1):
		for x in range(x0, x1):
			t += img.get_pixel(x, y).get_luminance()
	return t / float(maxi(1, (x1 - x0) * (y1 - y0)))

func _ready() -> void:
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("RESULT: FAIL (timeout)")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		print("RESULT: FAIL (windowed only)")
		get_tree().quit(2)
		return
	var seed_n := 912345
	var vsize := Vector2i(1600, 1000)
	var out := ""
	var tag := "s"
	var want := 3
	var ablate := false
	var ablate_x := 0.0
	var forced: Array = []
	var poi_kind := ""
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed": seed_n = int(args[i + 1]); i += 2
			"--vp":
				var wh := String(args[i + 1]).split("x")
				vsize = Vector2i(int(wh[0]), int(wh[1])); i += 2
			"--out": out = args[i + 1]; i += 2
			"--tag": tag = args[i + 1]; i += 2
			"--towns": want = int(args[i + 1]); i += 2
			"--ablate": ablate = true; i += 1
			"--ablate-x": ablate_x = float(args[i + 1]); i += 2
			"--poi": poi_kind = args[i + 1]; i += 2
			"--ids":
				for t in String(args[i + 1]).split(","):
					forced.append(int(t))
				i += 2
			"--force-touch": i += 1
			"--density": i += 2
			_: i += 1
	## The near-black count (T1-T3) is a desktop bar: the phone's toasts and sheets are near-black inside the measured square.
	## Its T4 (data) and T5 (the layer's own darkening, the same view with and without it) hold on both forms.
	var desktop := vsize.x >= 900
	if not desktop:
		print("  note: phone form, the near-black count (T1-T3) is not asserted; T4 and T5 are")
	var vp := SubViewport.new()
	vp.size = vsize
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": 384, "grid_h": 288, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	if out != "":
		DirAccess.make_dir_recursive_absolute(out)
	var places: Array = bridge.settlements()
	var picks: Array = []
	var kinds_seen := {}
	for pass_n in 2:
		for k in places.size():
			if picks.size() >= want or picks.has(k):
				continue
			var s: Dictionary = places[k]
			var inf: bool = int(s["x"]) < 384 * 0.15 or int(s["x"]) > 384 * 0.85 or int(s["y"]) < 288 * 0.15 or int(s["y"]) > 288 * 0.85
			if inf:
				continue
			if pass_n == 0 and kinds_seen.has(String(s["kind"])):
				continue
			if String(s["kind"]) in ["village", "hamlet"]:
				continue
			kinds_seen[String(s["kind"])] = true
			picks.append(k)
	if not forced.is_empty():
		picks = forced
	print("seed %d vp %s towns %s" % [seed_n, vsize, picks])
	var vh = app.viewport
	var overlay: Control = vh.overlay
	for id in picks:
		var s: Dictionary = places[id]
		var cell := Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5)
		if poi_kind != "":
			## `872fe397`'s map probe put a point of interest 1.05 km from the town; the same here, so the frame can say
			## whether the dark mass was the town or that marker (2026-11-17).
			var pcell := cell + Vector2(1050.0 * 384.0 / 800000.0, 0.0)
			var pid := int(bridge.poi_add(pcell.x, pcell.y, poi_kind, "smudge probe"))
			print("  poi %d (%s) at %s" % [pid, poi_kind, pcell])
			app.viewport.refresh_annotations()
			await _frames(4)
		var centre_g: Vector2 = (vh as Control).get_global_rect().get_center()
		## Back to the fit view, then in by 1.15 steps until the layer holds this town.
		vh._zoom_at(vh.size * 0.5, 1.0 / maxf(vh.zoom(), 0.001))
		var zooms := 0
		var below_zoom: float = vh.zoom()
		while zooms < 90:
			vh._camera.position += centre_g - _global_of(overlay, cell)
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
		print("town #%d %s (%s, pop %d): layer first draws at zoom %.2f (previous step %.2f)" % [id, s["name"], s["kind"], s["population"], appear_zoom, below_zoom])
		_ok((overlay._urban_draw_list as PackedInt32Array).has(id), "#%d the town layer draws the town" % id)
		var nm := "%s_%d_%d" % [tag, seed_n, id]
		# below
		vh._zoom_at(vh.size * 0.5, below_zoom / appear_zoom)
		vh._camera.position += centre_g - _global_of(overlay, cell)
		overlay.queue_redraw()
		await _frames(100)
		var img := await _grab(vp)
		var d_below := _dark(img, _global_of(overlay, cell))
		if out != "": img.save_png(out.path_join(nm + "_below.png"))
		# appears
		vh._zoom_at(vh.size * 0.5, appear_zoom / vh.zoom())
		vh._camera.position += centre_g - _global_of(overlay, cell)
		overlay.queue_redraw()
		await _frames(100)
		img = await _grab(vp)
		var d_app := _dark(img, _global_of(overlay, cell))
		if out != "": img.save_png(out.path_join(nm + "_appears.png"))
		print("  dark px: below %d, appears %d" % [d_below, d_app])
		## T4 the stroke floor a landmark or placed pin passes is one screen pixel: floor x camera zoom is 1 (a literal; the
		## default 1.0 local unit would give the camera zoom itself, 37.9 here).
		_ok(absf(overlay._pin_line_floor() * overlay._camera_zoom - 1.0) < 0.001, "T4 #%d pin stroke floor is one screen px (%.3f)" % [id, overlay._pin_line_floor() * overlay._camera_zoom])
		_ok(d_below <= DARK_MAX or not desktop, "T1 #%d below the layer: dark %d <= %d" % [id, d_below, DARK_MAX])
		_ok(d_app <= DARK_MAX or not desktop, "T2 #%d at first zoom: dark %d <= %d" % [id, d_app, DARK_MAX])
		if poi_kind != "":
			var rc: Rect2 = overlay._displayed_rect()
			print("    camera zoom %.2f; poi head r %.3f local px (x zoom = %.1f screen px); town head r %.3f (x zoom = %.1f); poi rows %s" % [
				overlay._camera_zoom, overlay._poi_pin_r(), overlay._poi_pin_r() * overlay._camera_zoom,
				overlay._settlement_head_r(String(s["kind"]), rc), overlay._settlement_head_r(String(s["kind"]), rc) * overlay._camera_zoom, overlay._pois])
		if ablate and poi_kind != "":
			## Which marker is the mass: hide the placed points of interest alone, then the settlements alone.
			var keep_p: Array = overlay._pois
			overlay._pois = []
			overlay.queue_redraw()
			await _frames(4)
			print("    placed POIs hidden: dark %d" % _dark(await _grab(vp), _global_of(overlay, cell)))
			overlay._pois = keep_p
			overlay.queue_redraw()
			await _frames(4)
		if ablate:
			var lay: Dictionary = overlay._urban_layouts[id]
			for key in lay.keys():
				var cp: Dictionary = lay.duplicate()
				cp.erase(key)
				overlay._urban_layouts[id] = cp
				overlay.queue_redraw()
				await _frames(3)
				var im2 := await _grab(vp)
				print("    erase %-22s dark %d" % [key, _dark(im2, _global_of(overlay, cell))])
			overlay._urban_layouts[id] = lay
			overlay.queue_redraw()
			await _frames(3)
		for m in [2.0, 4.0]:
			vh._zoom_at(vh.size * 0.5, m * appear_zoom / vh.zoom())
			vh._camera.position += centre_g - _global_of(overlay, cell)
			overlay.queue_redraw()
			await _frames(100)
			var im3 := await _grab(vp)
			var dm := _dark(im3, _global_of(overlay, cell))
			if out != "": im3.save_png(out.path_join(nm + "_x%d.png" % int(m)))
			if ablate_x == m:
				## Erase each layout key in turn at this multiplier and print the region's mean luminance: the key whose
				## removal lifts it is what paints the dark mass.
				print("    x%d mean luminance %.4f (unmodified)" % [int(m), _mean_lum(im3, _global_of(overlay, cell))])
				var lay2: Dictionary = overlay._urban_layouts[id]
				for key in lay2.keys():
					var cp2: Dictionary = lay2.duplicate()
					cp2.erase(key)
					overlay._urban_layouts[id] = cp2
					overlay.queue_redraw()
					await _frames(3)
					print("    x%d erase %-22s lum %.4f" % [int(m), key, _mean_lum(await _grab(vp), _global_of(overlay, cell))])
				overlay._urban_layouts[id] = lay2
				overlay.queue_redraw()
				await _frames(3)
			## T5: the layer's own darkening, against the same view with the layer off (data, not a frame).
			var cc := _global_of(overlay, cell)
			var lum_on := _mean_lum(im3, cc)
			overlay._show_urban_layouts = false
			overlay.queue_redraw()
			await _frames(6)
			var lum_off := _mean_lum(await _grab(vp), cc)
			overlay._show_urban_layouts = true
			overlay.queue_redraw()
			await _frames(6)
			print("    x%d mean luminance with the town layer %.4f, without %.4f (ratio %.3f)" % [int(m), lum_on, lum_off, lum_on / maxf(lum_off, 0.0001)])
			_ok(lum_on >= LUM_MIN_RATIO * lum_off, "T5 #%d x%d the town layer keeps %.2f of the terrain's luminance (>= %.2f)" % [id, int(m), lum_on / maxf(lum_off, 0.0001), LUM_MIN_RATIO])
			_ok(dm <= DARK_MAX or not desktop, "T3 #%d x%d: dark %d <= %d" % [id, int(m), dm, DARK_MAX])
			if m == 4.0 and out != "":
				## The City Viewer on the same town, for the frame (its strokes pass one unit per pixel, unchanged by the fix).
				app.open_city_viewer(id)
				await _frames(40)
				(await _grab(vp)).save_png(out.path_join(nm + "_viewer.png"))
				app.city_viewer_window.hide()
				await _frames(4)
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
