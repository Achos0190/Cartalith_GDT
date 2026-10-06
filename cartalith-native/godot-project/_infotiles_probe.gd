extends Node
## LOD-D7 (Ruling AV) -- deep-zoom tiles for the info views, read back from
## the pixels the shell actually draws. WINDOWED ONLY.
##
##   Godot_v4.7.1 --path . --resolution 1280x800 --rendering-driver opengl3 _infotiles_probe.tscn -- --out=<dir>
##
## It generates a small world, zooms deep, selects each tileable info view and
## asserts, per view:
##   1. tiles for THAT view were produced and are live (every live tile carries
##      the engine's current tag for the view, and the tag holds the view id);
##   2. the frame the viewer sees is measurably different from the same frame
##      with the info layer hidden (the map-resolution raster), i.e. the tile
##      is not the raster upsampled -- POSITIVE CONTROL: a view with no tile
##      form (`lith`) draws the raster and is pixel-identical with the layer
##      shown and hidden, so the difference metric can be zero;
##   3. for the bilinear-then-ramp views the difference is concentrated where
##      the field varies: cells whose neighbours differ in the raster show a
##      large tile-vs-raster difference, flat cells almost none, and cell
##      centres (where bilinear equals the sample) none;
##   4. NEGATIVE CONTROL: switching view changes the pixels, never leaves the
##      old view's tiles installed, and a switch made the same frame the old
##      view's tiles were requested (cache dropped, so they really land late)
##      ends in exactly the frame a clean switch gives.
## It then reports synthesis latency (median with min..max, run alone), tile
## memory against the tier budget, and the main-thread cost of the layer.
##
## Prints PROBE-FAIL lines and exits 0 either way (a Godot probe cannot be
## trusted to set an exit code through a window); grep PROBE-FAIL and the
## RESULT line. ASCII only.

const VP := Vector2i(1280, 800)
const WATCHDOG_S := 280
const SEED := 483920
const GRID := Vector2i(256, 192)
const WIDTH_KM := 1200.0
const ZOOM := 16.0
const SAMPLES_PER_VIEW := 15

## Every `ViewportHost.set_layer_visible` arm (derived from the definition, as
## `_lodsweep_probe.gd` does): vectors drawn over the tiles would contaminate
## the pixel comparison.
const OVERLAY_LAYERS: PackedStringArray = [
	"territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "rivers", "conflict",
]

## The views the module tiles, in `info_tile::TILED_VIEWS` order, and the ones
## by class (the disclosure Ruling AV asks for, restated here as LITERALS so the
## probe cannot pass by asserting the engine's table against itself).
const VIEWS: PackedStringArray = ["temp", "rain", "flow", "elevation", "slope", "aspect", "age", "resistance"]
const INTERPOLATED: PackedStringArray = ["temp", "age", "resistance"]
const REFINED: PackedStringArray = ["elevation", "slope", "aspect"]
const MIXED: PackedStringArray = ["rain", "flow"]

var _fail := 0
var _app: Node
var _vh: Control
var _br: Node
var _mgr = null
var _out := "user://infotiles/"
var _frames_cache: Dictionary = {}


func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		_fail += 1
		print("PROBE-FAIL: ", what)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server; frame_post_draw never fires.")
		get_tree().quit(2)
		return
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			_out = a.substr(6)
	get_tree().create_timer(WATCHDOG_S).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog fired after %d s." % WATCHDOG_S)
		get_tree().quit(2))
	DisplayServer.window_set_size(VP)
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _vh == null or _br == null:
		printerr("PROBE-CANNOT-RUN: shell exposed no viewport/bridge.")
		get_tree().quit(2)
		return
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	## The stale-.dll detector: `cargo test` does not rebuild the cdylib.
	if not _br.info_tiles_available():
		printerr("PROBE-CANNOT-RUN: the loaded extension has no info_* functions; rebuild the .dll.")
		get_tree().quit(2)
		return
	_mgr = _vh.info_tiles_manager()
	DirAccess.make_dir_recursive_absolute(_out)

	print("=== disclosure (engine's own table) ===")
	var tv: Dictionary = _br.info_tile_views()
	print("  views: ", tv.get("views", {}))
	for k in (tv.get("deferred", {}) as Dictionary).keys():
		print("  deferred ", k, ": ", (tv["deferred"] as Dictionary)[k])
	for v in INTERPOLATED:
		_check(str((tv["views"] as Dictionary).get(v, "")) == "interpolated", "%s is disclosed as interpolated" % v)
	for v in REFINED:
		_check(str((tv["views"] as Dictionary).get(v, "")) == "refined", "%s is disclosed as refined" % v)
	for v in MIXED:
		_check(str((tv["views"] as Dictionary).get(v, "")) == "mixed", "%s is disclosed as mixed" % v)

	if not await _load_world():
		get_tree().quit(2)
		return
	for l in OVERLAY_LAYERS:
		_vh.set_layer_visible(l, false)
	_vh.set_debug_opacity(1.0)
	await _go_deep()

	print("")
	print("=== 1-3: per view, tiles live, finer than the raster, concentrated where the field varies ===")
	var avail: Array[String] = []
	for v in VIEWS:
		if await _view_pass(v):
			avail.append(v)
	_check(avail.size() >= 6, "at least six of the eight tiled views are available in a generated world (got %d)" % avail.size())

	print("")
	print("=== control: a deferred view draws the raster, pixel-identical with the layer shown and hidden ===")
	await _deferred_control()

	print("")
	print("=== 4: negative control, a view switch never shows another view's tile ===")
	await _switch_control()

	print("")
	print("=== memory and main-thread cost over a pan ===")
	await _pan_run()

	print("")
	print("=== synthesis latency, run alone (request to texture handed back) ===")
	await _timing(avail)

	print("")
	print("RESULT: ", "GREEN" if _fail == 0 else "RED (%d failure(s))" % _fail)
	get_tree().quit(0)


# -- world and camera -----------------------------------------------------------

func _load_world() -> bool:
	_br.generate({"seed": SEED, "width_km": WIDTH_KM, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if _br.generating:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		return false
	await get_tree().create_timer(1.0).timeout
	return true


## Zoom to `ZOOM` around the highest-relief block of the fit frame (derived
## from the picture, so it moves with the seed instead of landing on ocean).
func _go_deep() -> void:
	_vh.reset_view()
	await _settle()
	var cap := await _capture()
	var g: Vector2i = _br.grid_size()
	var pivot := Vector2i(g.x / 2, g.y / 2)
	if cap.get("ok", false):
		var map: Rect2 = _to_crop(_vh._map_display_rect(), cap)
		var d: PackedByteArray = cap["data"]
		var w: int = cap["w"]
		var best := -1.0
		var blk := 48
		var y := int(maxf(map.position.y, 0.0))
		while y + blk < mini(int(cap["h"]), int(map.end.y)):
			var x := int(maxf(map.position.x, 0.0))
			while x + blk < mini(w, int(map.end.x)):
				var acc := 0
				for yy in range(y, y + blk, 4):
					var prev := _l(d, w, x, yy)
					for xx in range(x + 1, x + blk, 2):
						var l := _l(d, w, xx, yy)
						acc += absi(l - prev)
						prev = l
				if float(acc) > best:
					best = float(acc)
					pivot = Vector2i(int((x + blk * 0.5 - map.position.x) / map.size.x * g.x),
						int((y + blk * 0.5 - map.position.y) / map.size.y * g.y))
				x += blk
			y += blk
	print("  pivot cell ", pivot)
	_vh.move_view_to(float(pivot.x), float(pivot.y))
	await _settle()
	_vh.zoom_step(ZOOM / _vh.zoom())
	await _settle()
	var guard := 0
	while (not _vh.lod_active() or _vh.lod_pending() > 0) and guard < 900:
		await get_tree().process_frame
		guard += 1
	_check(_vh.lod_active(), "deep-zoom terrain tiles are up at zoom %.1f" % _vh.zoom())
	print("  zoom %.2f  px/cell %.1f  level %d" % [_vh.zoom(), _px_per_cell(), _vh._lod_child_level])


func _px_per_cell() -> float:
	var g: Vector2i = _br.grid_size()
	return minf(_vh.size.x / float(g.x), _vh.size.y / float(g.y)) * _vh.zoom()


func _settle() -> void:
	for i in 3:
		await RenderingServer.frame_post_draw


## Wait until the info layer has drawn `view` over its whole wanted set and has
## nothing outstanding. A deadline, not a frame count: a frame count is a claim
## about machine speed.
func _wait_engaged(view: String, ms: int = 60000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		await get_tree().process_frame
		var st: Dictionary = _vh.info_tile_stats()
		if str(st.get("view", "")) == view and bool(st.get("engaged", false)) \
				and int(st.get("pending", 1)) == 0 and int(st.get("backlog", 1)) == 0 and _vh.lod_pending() == 0:
			await _settle()
			return true
	return false


# -- capture and metrics ------------------------------------------------------------

func _capture() -> Dictionary:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return {"ok": false}
	var gp: Vector2 = _vh.global_position
	var want := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round()))
	var clipped := want.intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	if clipped.size.x < 8 or clipped.size.y < 8:
		return {"ok": false}
	var sub := img.get_region(clipped)
	if sub.get_format() != Image.FORMAT_RGBA8:
		sub.convert(Image.FORMAT_RGBA8)
	var shift := Vector2(want.position - clipped.position)
	return {"ok": true, "img": sub, "data": sub.get_data(), "w": sub.get_width(), "h": sub.get_height(),
		"campos": _vh._camera.position + shift, "zoom": _vh.zoom()}


## A `_camera`-local rect in the capture's pixel space.
func _to_crop(local: Rect2, cap: Dictionary) -> Rect2:
	return Rect2(cap["campos"] + local.position * float(cap["zoom"]), local.size * float(cap["zoom"]))


func _l(d: PackedByteArray, w: int, x: int, y: int) -> int:
	var i := (y * w + x) * 4
	return (d[i] * 54 + d[i + 1] * 183 + d[i + 2] * 19) >> 8


func _px(d: PackedByteArray, w: int, x: int, y: int) -> Vector3i:
	var i := (y * w + x) * 4
	return Vector3i(d[i], d[i + 1], d[i + 2])


func _l1(a: Vector3i, b: Vector3i) -> int:
	return absi(a.x - b.x) + absi(a.y - b.y) + absi(a.z - b.z)


## Fraction of horizontally adjacent pixel pairs that are exactly equal, over
## every 4th row of the crop. A blocky raster scores near 1 - 1/px_per_cell; a
## smooth or detailed tile scores lower wherever the field varies.
func _eqfrac(cap: Dictionary) -> float:
	var d: PackedByteArray = cap["data"]
	var w: int = cap["w"]
	var eq := 0
	var n := 0
	for y in range(8, int(cap["h"]) - 8, 4):
		for x in range(8, w - 9):
			var i := (y * w + x) * 4
			n += 1
			if d[i] == d[i + 4] and d[i + 1] == d[i + 5] and d[i + 2] == d[i + 6]:
				eq += 1
	return float(eq) / float(maxi(n, 1))


## Mean absolute RGB difference between two captures of the same camera.
func _meandiff(a: Dictionary, b: Dictionary) -> float:
	var da: PackedByteArray = a["data"]
	var db: PackedByteArray = b["data"]
	var w: int = a["w"]
	var s := 0
	var n := 0
	for y in range(0, int(a["h"]), 3):
		for x in range(0, w, 3):
			var i := (y * w + x) * 4
			s += absi(da[i] - db[i]) + absi(da[i + 1] - db[i + 1]) + absi(da[i + 2] - db[i + 2])
			n += 3
	return float(s) / float(maxi(n, 1))


## The per-cell split behind assertion 3. For every cell whose centre and its
## four neighbours' centres lie inside the crop: `step` is the largest L1
## colour difference between the RASTER's cell colour and a neighbour's (0 where
## the field is locally constant); the tile-vs-raster difference is sampled on a
## 5x5 grid inside the cell. Returns means for flat cells (step 0), varying
## cells (step >= 8) and cell centres, with counts.
func _cell_split(t: Dictionary, r: Dictionary) -> Dictionary:
	var g: Vector2i = _br.grid_size()
	var map: Rect2 = _to_crop(_vh._map_display_rect(), t)
	var cs := map.size / Vector2(g)
	var td: PackedByteArray = t["data"]
	var rd: PackedByteArray = r["data"]
	var w: int = t["w"]
	var h: int = t["h"]
	var flat_s := 0.0
	var flat_n := 0
	var var_s := 0.0
	var var_n := 0
	var cen_s := 0.0
	var cen_n := 0
	var c0 := Vector2i(int(floor((0.0 - map.position.x) / cs.x)) + 2, int(floor((0.0 - map.position.y) / cs.y)) + 2)
	var c1 := Vector2i(int(floor((float(w) - map.position.x) / cs.x)) - 2, int(floor((float(h) - map.position.y) / cs.y)) - 2)
	for cy in range(maxi(c0.y, 1), mini(c1.y, g.y - 1)):
		for cx in range(maxi(c0.x, 1), mini(c1.x, g.x - 1)):
			var ctr := map.position + (Vector2(cx, cy) + Vector2(0.5, 0.5)) * cs
			var ix := int(ctr.x)
			var iy := int(ctr.y)
			if ix < 0 or iy < 0 or ix >= w or iy >= h:
				continue
			var rc := _px(rd, w, ix, iy)
			var step := 0
			var ok := true
			for off in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
				var p: Vector2 = ctr + Vector2(off) * cs
				if p.x < 0.0 or p.y < 0.0 or p.x >= float(w) or p.y >= float(h):
					ok = false
					break
				step = maxi(step, _l1(rc, _px(rd, w, int(p.x), int(p.y))))
			if not ok:
				continue
			cen_s += float(_l1(rc, _px(td, w, ix, iy)))
			cen_n += 1
			var acc := 0.0
			var m := 0
			for sy in range(-2, 3):
				for sx in range(-2, 3):
					var q := ctr + Vector2(sx, sy) * cs * 0.2
					var qx := clampi(int(q.x), 0, w - 1)
					var qy := clampi(int(q.y), 0, h - 1)
					acc += float(_l1(_px(td, w, qx, qy), _px(rd, w, qx, qy)))
					m += 1
			if step == 0:
				flat_s += acc / float(m)
				flat_n += 1
			elif step >= 8:
				var_s += acc / float(m)
				var_n += 1
	return {"flat": flat_s / float(maxi(flat_n, 1)), "flat_n": flat_n,
		"varying": var_s / float(maxi(var_n, 1)), "varying_n": var_n,
		"centre": cen_s / float(maxi(cen_n, 1)), "centre_n": cen_n}


## The same frame with the info layer hidden and the raster showing -- the
## pre-D7 picture. Restores the layer afterwards.
func _raster_only() -> Dictionary:
	_mgr.layer().visible = false
	_vh._debug_layer.visible = true
	await _settle()
	var cap := await _capture()
	_mgr.layer().visible = true
	_vh._sync_info_raster()
	await _settle()
	return cap


func _save(cap: Dictionary, name: String) -> void:
	if cap.get("ok", false):
		(cap["img"] as Image).save_png(_out.path_join(name + ".png"))


# -- 1-3 --------------------------------------------------------------------------

func _view_pass(view: String) -> bool:
	print("")
	print("-- ", view, " --")
	_vh.set_debug_layer(view)
	if str(_br.info_tile_tag(view)) == "":
		print("  info ", view, " has no tile input in this world: raster only (listed, not failed)")
		return false
	if not await _wait_engaged(view):
		_check(false, "%s: tiles covered the view within the deadline (%s)" % [view, _vh.info_tile_stats()])
		return false
	var st: Dictionary = _vh.info_tile_stats()
	var tag: String = _br.info_tile_tag(view)
	var tagged := true
	for key in (_mgr._tiles as Dictionary).keys():
		if str((_mgr._tiles[key] as Sprite2D).get_meta("info_tag", "")) != tag:
			tagged = false
	_check(int(st["live"]) > 0, "%s: %d live tiles drawn (%d bytes)" % [view, int(st["live"]), int(st["live_bytes"])])
	_check(tagged and tag.find(";" + view + ";") >= 0, "%s: every live tile carries the engine's current tag, which names the view (%s)" % [view, tag.substr(0, 40)])
	_check(not _vh._debug_layer.visible, "%s: the map-resolution raster is hidden once the tiles cover the view" % view)
	var t := await _capture()
	var r := await _raster_only()
	_save(t, "tile_" + view)
	_save(r, "raster_" + view)
	if not (t.get("ok", false) and r.get("ok", false)):
		_check(false, "%s: captures were read back" % view)
		return true
	_frames_cache[view] = t
	var eq_t := _eqfrac(t)
	var eq_r := _eqfrac(r)
	var md := _meandiff(t, r)
	var split := _cell_split(t, r)
	print("  info eqfrac tile %.4f raster %.4f   mean|T-R| %.3f   cells flat %d varying %d" % [eq_t, eq_r, md, int(split["flat_n"]), int(split["varying_n"])])
	print("  info tile-vs-raster diff: flat cells %.3f  varying cells %.3f  cell centres %.3f (L1 of 765)" % [split["flat"], split["varying"], split["centre"]])
	## Thresholds are LABELLED JUDGEMENTS set from the first windowed run of this
	## probe (numbers in the report), each far below what the views measured and
	## far above the zero a pixel-identical frame gives.
	_check(md > 0.5, "%s: the tile frame differs from the raster frame (mean %.3f > 0.5)" % [view, md])
	if view in INTERPOLATED:
		_check(eq_t < eq_r - 0.05, "%s: smoother than the raster's blocks (eqfrac %.4f < %.4f - 0.05)" % [view, eq_t, eq_r])
		_check(int(split["varying_n"]) >= 4 and int(split["flat_n"]) >= 0, "%s: enough varying cells to judge (%d)" % [view, int(split["varying_n"])])
		_check(float(split["centre"]) <= 3.0, "%s: cell centres agree with the raster (bilinear equals the sample), %.3f <= 3" % [view, split["centre"]])
		_check(float(split["varying"]) >= 4.0 * maxf(float(split["flat"]), 0.25), "%s: the difference is concentrated where the field varies (%.3f >= 4 x max(%.3f, 0.25))" % [view, split["varying"], split["flat"]])
	elif view in REFINED:
		_check(eq_t < eq_r - 0.05, "%s: carries detail the raster blocks do not (eqfrac %.4f < %.4f - 0.05)" % [view, eq_t, eq_r])
	else:
		_check(eq_t <= eq_r, "%s: no blockier than the raster (eqfrac %.4f <= %.4f)" % [view, eq_t, eq_r])
	return true


# -- control: a deferred view -------------------------------------------------------

func _deferred_control() -> void:
	_vh.set_debug_layer("lith")
	await _settle()
	await _settle()
	var st: Dictionary = _vh.info_tile_stats()
	_check(int(st.get("live", -1)) == 0 and str(st.get("view", "x")) == "", "lith (deferred): the info layer holds no tiles and no view")
	_check(_vh._debug_layer.visible and _vh._debug_layer.texture != null, "lith (deferred): the map-resolution raster is what draws")
	var a := await _capture()
	var b := await _raster_only()
	_check(a.get("ok", false) and b.get("ok", false), "lith: captures were read back")
	if a.get("ok", false) and b.get("ok", false):
		var md := _meandiff(a, b)
		_check(md == 0.0, "lith (deferred): frame is pixel-identical with the info layer shown and hidden (mean diff %.4f)" % md)
		_save(a, "deferred_lith")


# -- 4: view switch ------------------------------------------------------------------

func _switch_control() -> void:
	## Reference frames from clean, settled switches.
	_vh.set_debug_layer("temp")
	await _wait_engaged("temp")
	var temp_ref := await _capture()
	_vh.set_debug_layer("rain")
	var ok_r := await _wait_engaged("rain")
	var rain_ref := await _capture()
	_check(ok_r, "temp -> rain: the new view's tiles covered the view")
	var tag_rain: String = _br.info_tile_tag("rain")
	var wrong := 0
	for key in (_mgr._tiles as Dictionary).keys():
		if str((_mgr._tiles[key] as Sprite2D).get_meta("info_tag", "")) != tag_rain:
			wrong += 1
	_check(wrong == 0, "temp -> rain: no live tile carries the old view's tag (%d wrong)" % wrong)
	var d_tr := _meandiff(temp_ref, rain_ref)
	_check(d_tr > 1.0, "temp -> rain changes the pixels (mean diff %.3f > 1.0)" % d_tr)
	_save(rain_ref, "switch_rain")
	## Back to temp: the parked tiles come back, and they come back as TEMP.
	_vh.set_debug_layer("temp")
	await _wait_engaged("temp")
	var temp_again := await _capture()
	var d_back := _meandiff(temp_ref, temp_again)
	_check(d_back == 0.0, "rain -> temp restores exactly the first temp frame (mean diff %.4f)" % d_back)

	## The late-arrival case: ask for temp, switch to rain in the SAME frame,
	## with the parked cache dropped so rain's tiles are real requests and temp's
	## requests are genuinely in flight when the switch happens.
	_vh.set_debug_layer("off")
	await _settle()
	_mgr.drop_cache()
	var stale_before := int(_vh.info_tile_stats().get("dropped_stale", 0))
	_vh.set_debug_layer("temp")
	_vh.set_debug_layer("rain")
	var ok2 := await _wait_engaged("rain")
	var late := await _capture()
	_check(ok2, "same-frame temp -> rain: rain tiles covered the view")
	var d_late := _meandiff(late, rain_ref)
	_check(d_late == 0.0, "same-frame temp -> rain ends in exactly the clean rain frame (mean diff %.4f); no temp tile reached the screen" % d_late)
	var stale_after := int(_vh.info_tile_stats().get("dropped_stale", 0))
	print("  info stale arrivals dropped by the view/tag check in that switch: ", stale_after - stale_before)
	var tag2: String = _br.info_tile_tag("rain")
	var wrong2 := 0
	for key in (_mgr._tiles as Dictionary).keys():
		if str((_mgr._tiles[key] as Sprite2D).get_meta("info_tag", "")) != tag2:
			wrong2 += 1
	_check(wrong2 == 0, "same-frame switch: every live tile carries the rain tag")


# -- memory and main-thread cost ---------------------------------------------------

func _pan_run() -> void:
	var budget: Dictionary = _br.lod_budget()
	var cache_tiles := int(budget.get("cache_tiles", 0))
	var park_cap := cache_tiles / 2
	var g: Vector2i = _br.grid_size()
	var centre := Vector2(g.x * 0.5, g.y * 0.5)
	var stats_info: Array = []
	var frame_off: PackedFloat64Array = []
	var frame_on: PackedFloat64Array = []
	var max_live := 0
	var max_parked := 0
	var max_bytes := 0
	for pass_view in ["off", "temp"]:
		_vh.set_debug_layer(pass_view)
		await _settle()
		var t_prev := Time.get_ticks_usec()
		for i in 60:
			var a := float(i) / 60.0 * TAU
			_vh.move_view_to(centre.x + cos(a) * g.x * 0.35, centre.y + sin(a * 1.3) * g.y * 0.3)
			await get_tree().process_frame
			var now := Time.get_ticks_usec()
			(frame_off if pass_view == "off" else frame_on).append(float(now - t_prev) / 1000.0)
			t_prev = now
			if pass_view != "off":
				var st: Dictionary = _vh.info_tile_stats()
				max_live = maxi(max_live, int(st["live"]))
				max_parked = maxi(max_parked, int(st["parked"]))
				max_bytes = maxi(max_bytes, int(st["live_bytes"]) + int(st["parked_bytes"]))
	var tile_bytes := 256 * 256 * 4
	print("  info tile = %d bytes (256x256 RGBA8). budget: tier cache_tiles %d, info parked cap %d" % [tile_bytes, cache_tiles, park_cap])
	print("  info over a 60-step pan: max live %d, max parked %d, max live+parked %.2f MiB" % [max_live, max_parked, float(max_bytes) / 1048576.0])
	_check(max_parked <= park_cap, "parked info tiles never exceeded the cap (%d <= %d)" % [max_parked, park_cap])
	_check(max_bytes <= (park_cap + 64) * tile_bytes, "info tile memory stayed bounded (%.2f MiB <= %.2f MiB)" % [float(max_bytes) / 1048576.0, float((park_cap + 64) * tile_bytes) / 1048576.0])
	frame_off.sort()
	frame_on.sort()
	print("  info frame ms during the pan, info off: median %.2f  p95 %.2f  max %.2f" % [_pct(frame_off, 0.5), _pct(frame_off, 0.95), frame_off[frame_off.size() - 1]])
	print("  info frame ms during the pan, info on : median %.2f  p95 %.2f  max %.2f" % [_pct(frame_on, 0.5), _pct(frame_on, 0.95), frame_on[frame_on.size() - 1]])
	## Main-thread cost of the layer's own per-move work: `_update_lod()` is
	## idempotent, so it is timed directly, info on versus off.
	var on_us := PackedFloat64Array()
	var off_us := PackedFloat64Array()
	for pass_view in ["temp", "off"]:
		_vh.set_debug_layer(pass_view)
		await _wait_engaged(pass_view) if pass_view != "off" else await _settle()
		for i in 25:
			var t0 := Time.get_ticks_usec()
			_vh._update_lod()
			(on_us if pass_view == "temp" else off_us).append(float(Time.get_ticks_usec() - t0))
	on_us.sort()
	off_us.sort()
	print("  info _update_lod() us, info on: median %.0f (min %.0f max %.0f)   off: median %.0f (min %.0f max %.0f)" % [
		_pct(on_us, 0.5), on_us[0], on_us[on_us.size() - 1], _pct(off_us, 0.5), off_us[0], off_us[off_us.size() - 1]])
	_check(_pct(on_us, 0.5) - _pct(off_us, 0.5) < 4000.0, "the layer adds under 4 ms (labelled judgement: a quarter frame) to a steady-state camera update")


func _pct(a: PackedFloat64Array, p: float) -> float:
	if a.is_empty():
		return 0.0
	return a[clampi(int(floor(p * float(a.size() - 1) + 0.5)), 0, a.size() - 1)]


# -- synthesis latency --------------------------------------------------------------

## Request one tile, spin until it is handed back, record the wall time. The
## info layer is switched off first so nothing else is queued and the manager
## does not drain our tiles; tiles differ per rep so none is a cache hit.
func _timing(avail: Array[String]) -> void:
	_vh.set_debug_layer("off")
	await _settle()
	var z: int = _vh._lod_child_level
	var n: int = _br.lod_tiles_per_axis(z)
	print("  info level %d (%d tiles per axis), %d reps per view, one request at a time" % [z, n, SAMPLES_PER_VIEW])
	for view in avail:
		var us := PackedFloat64Array()
		var bad := 0
		for k in SAMPLES_PER_VIEW:
			var col := (k * 7 + 3) % n
			var row := (k * 5 + 1) % n
			var t0 := Time.get_ticks_usec()
			var asked: bool = _br.info_request_tile(view, z, col, row)
			var got := false
			var guard := 0
			while asked and not got and guard < 2000000:
				for e in _br.info_take_ready_tiles(4):
					if str((e as Dictionary).get("view", "")) == view and int((e as Dictionary).get("col", -1)) == col and int((e as Dictionary).get("row", -1)) == row:
						got = true
				guard += 1
			if got:
				us.append(float(Time.get_ticks_usec() - t0))
			else:
				bad += 1
		us.sort()
		_check(bad == 0 and us.size() == SAMPLES_PER_VIEW, "%s: all %d timing requests returned a tile" % [view, SAMPLES_PER_VIEW])
		if not us.is_empty():
			print("  info %-10s median %.2f ms   min..max %.2f..%.2f ms" % [view, _pct(us, 0.5) / 1000.0, us[0] / 1000.0, us[us.size() - 1] / 1000.0])
