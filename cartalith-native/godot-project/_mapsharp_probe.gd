extends Node
## The four owner reports of 2026-09-07, measured against a live shell rather
## than reasoned about from the scene graph.  Nothing here is fixed by this
## probe; it only establishes WHICH mechanism each report is.
##
##   R1  settlement names blurry     -- is the ink under `_camera` sharp at zoom?
##   R2  LOD/atlas debug layer       -- does `_lod_debug_layer` ever draw?
##   R3  zoom does not sharpen       -- does the level switch, and do the
##                                      tiles carry colour detail?
##   R4  settlements without roads   -- missing from the data, or not drawn?
##
## MUST run WINDOWED: `ImageTexture.update()` is a no-op under `--headless` and
## `RenderingServer.frame_post_draw` never fires there, so every pixel check
## below would either hang or pass vacuously (`MISTAKES.md`).
##   Godot_v4.7.1-stable_win64_console.exe --path . _mapsharp_probe.tscn
## No command-line arguments are read.
##
## Exit status (this project's convention):
##   0  every assertion held
##   1  an assertion failed
##   2  could not run
##   3  every assertion held and the surface still fails

const VP := Vector2i(1600, 1000)
const SEED := 483920
const GRID_W := 512
const GRID_H := 384
const WIDTH_KM := 1200.0

## How near a way polyline has to pass a settlement's own cell before that
## settlement counts as "reached by a road".  A settlement pin is a few cells
## across at fit zoom; `civ_consolidate_and_smooth_ways`' own endpoint snap
## (`snap_t2`) works to `max(6, 4/sc)` routing cells, so this is deliberately
## generous -- the question is whether ANY edge arrives, not how tidily.
const REACH_CELLS := 3.0

const WATCHDOG_S := 420

var _fail: Array[String] = []
var _app: Node
var _vh: Control
var _br: Node


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server -- no canvas is composited.")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(VP)
	## Watchdog. A probe that hangs must not be read as a failure it never
	## committed (`MISTAKES.md`), and a windowed run left open blocks the
	## shell it was launched from.
	get_tree().create_timer(WATCHDOG_S).timeout.connect(func():
		printerr("PROBE-CANNOT-RUN: watchdog fired after %d s." % WATCHDOG_S)
		get_tree().quit(2))
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
	## The welcome sheet (`open_project_dialog.gd`'s `open_welcome()`) covers
	## the map on boot -- its own "Continue without a world" link is exactly
	## this `hide()`.  Caught by looking at a capture rather than at the node
	## tree: a whole-frame detail measurement read the STATIC DIALOG and
	## reported the same value to fourteen digits at every zoom.
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await get_tree().process_frame

	_br.generate({
		"seed": SEED, "width_km": WIDTH_KM, "grid_w": GRID_W, "grid_h": GRID_H,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	var spins := 0
	while _br.generating and spins < 600:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if _br.generating:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	await get_tree().create_timer(1.0).timeout

	var places: Array = _br.settlements()
	var ways: Array = _br.roads()
	if places.is_empty():
		printerr("PROBE-CANNOT-RUN: the world has no settlements to measure.")
		get_tree().quit(2)
		return
	print("world: %d settlements, %d ways, grid %dx%d" % [places.size(), ways.size(), GRID_W, GRID_H])

	_r4_roads(places, ways)
	await _r3_lod()
	await _r2_debug_layer()
	await _r1_label_sharpness(places)

	for f in _fail:
		printerr("ASSERT-FAILED: ", f)
	print("PROBE-RESULT: ", "PASS" if _fail.is_empty() else "FAIL")
	print("images: ", ProjectSettings.globalize_path("user://"))
	get_tree().quit(0 if _fail.is_empty() else 1)


## ── R4 ──────────────────────────────────────────────────────────────────────
## Is a road MISSING FROM THE DATA for these settlements, or present and not
## drawn at that zoom?  `get_roads()` already drops `hidden` ways, so this is
## the drawable set -- exactly what the map can show at any zoom.  A screenshot
## cannot tell the two apart; a distance can.
func _r4_roads(places: Array, ways: Array) -> void:
	print("\n== R4: settlements vs. the drawable way network ==")
	var by_kind: Dictionary = {}
	var wt: Dictionary = {}
	for w: Dictionary in ways:
		var k: String = w["way_type"]
		wt[k] = int(wt.get(k, 0)) + 1
	print("  way types: ", wt)

	for i in places.size():
		var s: Dictionary = places[i]
		var kind: String = s["kind"]
		var p := Vector2(float(s["x"]), float(s["y"]))
		var best := 1e30
		for w: Dictionary in ways:
			var pts: PackedVector2Array = w["points"]
			for q in pts:
				var d := p.distance_squared_to(q)
				if d < best:
					best = d
		best = sqrt(best)
		## `map_overlay.gd`'s `VILLAGE_ADDON_POP` -- an addon village seeded by
		## `civ_seed_villages` carries population 0 and `kind == "hamlet"`, and
		## is the tier that appears only past `VILLAGE_ADDON_LOD = 2.4`. That
		## is the owner's "the ones that appear on zoom", so it is split out
		## rather than folded into the hamlet row.
		var addon: bool = kind == "hamlet" and int(s.get("population", 1)) == 0
		var key: String = "hamlet(addon)" if addon else kind
		if not by_kind.has(key):
			by_kind[key] = {"n": 0, "reached": 0, "worst": 0.0}
		by_kind[key]["n"] += 1
		if best <= REACH_CELLS:
			by_kind[key]["reached"] += 1
		by_kind[key]["worst"] = maxf(by_kind[key]["worst"], best)

	var orphans := 0
	var orphan_kinds: Dictionary = {}
	for kind in by_kind:
		var r: Dictionary = by_kind[kind]
		var miss: int = int(r["n"]) - int(r["reached"])
		orphans += miss
		if miss > 0:
			orphan_kinds[kind] = miss
		print("  %-11s n=%3d  reached=%3d  unreached=%3d  worst gap=%.1f cells"
			% [kind, r["n"], r["reached"], miss, r["worst"]])
	print("  unreached total: %d of %d  (%s)" % [orphans, _sum_n(by_kind), orphan_kinds])
	print("  --> the way list `get_roads()` returns is what the map draws at EVERY zoom;")
	print("      an unreached settlement here has no edge in the data, not a hidden one.")
	print("      `civ_seed_villages` (reference `_civSeedVillages`, line 25164) takes the")
	print("      road topology as an INPUT and its output never re-enters it, so an addon")
	print("      village having no way of its own is the reference's own shape.")


func _sum_n(by_kind: Dictionary) -> int:
	var n := 0
	for k in by_kind:
		n += int(by_kind[k]["n"])
	return n


## ── R3 ──────────────────────────────────────────────────────────────────────
## (a) does the level switch, (b) do deeper tiles carry more detail, (c) does a
## filter blur them?  This measures (a) and leaves (b)/(c) to the tile content,
## which is engine-side.
func _r3_lod() -> void:
	print("\n== R3: does the LOD level actually change with zoom? ==")
	var g: Vector2i = _br.grid_size()
	var native := minf(float(_vh.size.x) / float(g.x), float(_vh.size.y) / float(g.y))
	print("  native px/cell = %.3f, zoom_max = %.1f" % [native, _vh._zoom_max])
	var hf: Array = []
	for target: float in [1.0, 2.0, 2.5, 4.0, 8.0, 16.0, 40.0]:
		await _zoom_to(target)
		var z: float = _vh.zoom()
		var ppc := native * z
		var lvl: int = _br.lod_level_for_zoom(ppc)
		var img: Image = await _grab("lod_z%02d" % int(z))
		var e := _hf_energy(img)
		hf.append({"z": z, "e": e})
		print("  zoom %6.2f  px/cell %7.2f  level %2d  tiles/axis %5d  active=%s  live tiles=%3d  detail/px %.4f"
			% [z, ppc, lvl, _br.lod_tiles_per_axis(lvl), _vh.lod_active(),
				(_vh._lod_tiles as Dictionary).size(), e])
	var switched: bool = int(_br.lod_level_for_zoom(native * 4.0)) != int(_br.lod_level_for_zoom(native * 16.0))
	_expect(switched, "the pyramid level DOES change between zoom 4 and zoom 16 -- candidate (a) is not it")

	## Candidate (b) vs (c), asked of the picture rather than of the code: how
	## much of the frame does the deep-zoom layer own at all?  `_lod_layer`
	## carries every synthesized tile, so hiding it falls back to the base
	## raster and the difference is the entire contribution of the level that
	## was just switched to.
	await _zoom_to(16.0)
	var with_lod: Image = await _grab("lod_on")
	var lay: Control = _vh._lod_layer
	lay.visible = false
	await _settle()
	var without: Image = await _grab("lod_off")
	lay.visible = true
	await _settle()
	print("  at zoom %.1f the deep-zoom layer changes %d px, mean |dL| contribution %.4f"
		% [_vh.zoom(), _diff_px(with_lod, without), _mean_abs_diff(with_lod, without)])
	print("  detail/px across the zoom ladder: %s" % [hf])


## ── R2 ──────────────────────────────────────────────────────────────────────
## The chunk-debug layer.  Two questions, in order: does it draw at all when
## switched on over live tiles, and does it survive the camera moving?
func _r2_debug_layer() -> void:
	print("\n== R2: the LOD/atlas chunk-debug overlay ==")
	var dbg: Control = _vh._lod_debug_layer

	## (i) Teeth check.  Switch the overlay on where tiles are ALREADY live, so
	## `set_lod_debug()`'s own `queue_redraw()` has something to draw, and diff
	## the framebuffer.  Everything after this is worthless if this is 0.
	await _zoom_to(8.0)
	await _settle()
	var off: Image = await _grab("loddebug_off")
	for which in ["grid", "colors", "labels"]:
		_vh.set_lod_debug(which, true)
	await _settle()
	var on: Image = await _grab("loddebug_on")
	var ink := _diff_px(off, on)
	print("  toggled ON over %d live tiles: %d px of overlay ink" % [(_vh._lod_tiles as Dictionary).size(), ink])
	_expect(ink > 0, "the chunk-debug overlay CAN draw when toggled over live tiles (%d px)" % ink)

	## (ii) The order a user actually takes: switch it on at fit zoom, where
	## `_lod_tiles` is empty and `_draw_lod_debug()` returns at its own guard,
	## then zoom in until tiles exist.  Nothing on the tile path calls
	## `_lod_debug_layer.queue_redraw()`, so the question is whether the
	## overlay ever catches up on its own.
	for which in ["grid", "colors", "labels"]:
		_vh.set_lod_debug(which, false)
	await _zoom_to(1.0)
	await _settle()
	for which in ["grid", "colors", "labels"]:
		_vh.set_lod_debug(which, true)
	var tiles_at_toggle: int = (_vh._lod_tiles as Dictionary).size()
	await _zoom_to(8.0)
	await _settle()
	var user_order: Image = await _grab("loddebug_user_order")
	var tiles_now: int = (_vh._lod_tiles as Dictionary).size()
	print("  toggled ON at %d live tiles, then zoomed to %d tiles: overlay ink now %d px"
		% [tiles_at_toggle, tiles_now, _diff_px(off, user_order)])
	dbg.queue_redraw()
	await _settle()
	var user_forced: Image = await _grab("loddebug_user_order_forced")
	var moved := _diff_px(user_order, user_forced)
	print("  ... and a bare `queue_redraw()` with nothing else touched moves %d px" % moved)
	_expect(moved == 0,
		"toggle-then-zoom leaves nothing owed to a forced redraw (%d px owed)" % moved)

	## (iii) And now the camera moves, with the toggles left alone.  Panning
	## frees tiles that scrolled out and builds new ones (`_apply_lod_tiles`),
	## so the overlay's picture is of a tile set that no longer exists.
	_vh._camera.position += Vector2(-140.0, -90.0)
	_vh._update_lod()
	await _settle()
	var panned: Image = await _grab("loddebug_after_pan")
	dbg.queue_redraw()
	await _settle()
	var panned_forced: Image = await _grab("loddebug_after_pan_forced")
	var moved2 := _diff_px(panned, panned_forced)
	print("  after a pan, a bare `queue_redraw()` moves %d px" % moved2)
	_expect(moved2 == 0,
		"a pan leaves nothing owed to a forced redraw (%d px owed)" % moved2)

	for which in ["grid", "colors", "labels"]:
		_vh.set_lod_debug(which, false)
	await _settle()


## ── R1 ──────────────────────────────────────────────────────────────────────
## Settlement ink is isolated by flipping the layer off and diffing, so the
## measurement is the overlay's own pixels and not the terrain's.
func _r1_label_sharpness(places: Array) -> void:
	print("\n== R1: is the settlement layer's own ink sharp at zoom? ==")
	var big := _biggest(places)
	print("  subject: %s (%s) at cell %d,%d" % [big["name"], big["kind"], int(big["x"]), int(big["y"])])
	## A town whose generated layout is drawn fully opaque gives up its pin to
	## it (`map_overlay.gd`'s `_urban_revealed`), so at deep zoom the
	## settlements layer owns no pixels at all and the isolation below measures
	## an empty mask.  Measured once on this probe: 0 ink at zoom 12.
	_vh.set_layer_visible("urban_layouts", false)
	var out: Array = []
	for target: float in [1.0, 4.0, 12.0]:
		await _zoom_to(target)
		_vh.move_view_to(float(big["x"]), float(big["y"]))
		await _settle()
		var on: Image = await _grab("map_z%02d_on" % int(target))
		_vh.set_layer_visible("settlements", false)
		await _settle()
		var off: Image = await _grab("map_z%02d_off" % int(target))
		_vh.set_layer_visible("settlements", true)
		await _settle()
		var m := _mask_measure(on, off)
		m["z"] = _vh.zoom()
		out.append(m)
		print("  zoom %6.2f  settlement ink=%5d px  box=%s  top16 |dL|=%.3f"
			% [m["z"], m["ink"], m["box"], m["top"]])
	if out.size() >= 2 and float(out[0]["top"]) > 0.0:
		var keep: float = float(out[out.size() - 1]["top"]) / float(out[0]["top"])
		print("  edge contrast retained from zoom %.2f to %.2f: %.2f"
			% [out[0]["z"], out[out.size() - 1]["z"], keep])
		_expect(int(out[0]["ink"]) > 0 and int(out[out.size() - 1]["ink"]) > 0,
			"the settlement layer drew ink at both zooms (teeth check)")


func _biggest(places: Array) -> Dictionary:
	var best: Dictionary = places[0]
	for s: Dictionary in places:
		if int(s.get("population", 0)) > int(best.get("population", 0)):
			best = s
	return best


## ── helpers ─────────────────────────────────────────────────────────────────

func _zoom_to(target: float) -> void:
	var cur: float = _vh.zoom()
	if cur <= 0.0:
		return
	_vh.zoom_step(target / cur)
	await _settle()


func _settle() -> void:
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw


func _grab(name: String) -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("user://mapsharp_%s.png" % name)
	return img


## Mean absolute adjacent-pixel luminance step over the map area -- "how much
## detail is there per screen pixel".  A picture that is genuinely resolving
## finer ground holds this roughly steady as the camera zooms; one that is
## magnifying the same source loses it in proportion to the zoom.
func _hf_energy(img: Image) -> float:
	var acc := 0.0
	var n := 0
	var x0 := int(img.get_width() * 0.3)
	var x1 := int(img.get_width() * 0.7)
	var y0 := int(img.get_height() * 0.3)
	var y1 := int(img.get_height() * 0.7)
	for y in range(y0, y1, 2):
		var prev := _lum(img.get_pixel(x0, y))
		for x in range(x0 + 1, x1):
			var l := _lum(img.get_pixel(x, y))
			acc += absf(l - prev)
			n += 1
			prev = l
	return acc / maxf(1.0, float(n))


func _mean_abs_diff(a: Image, b: Image) -> float:
	var acc := 0.0
	var n := 0
	for y in range(0, a.get_height(), 2):
		for x in range(0, a.get_width(), 2):
			acc += absf(_lum(a.get_pixel(x, y)) - _lum(b.get_pixel(x, y)))
			n += 1
	return acc / maxf(1.0, float(n))


func _diff_px(a: Image, b: Image) -> int:
	var n := 0
	for y in range(0, a.get_height(), 2):
		for x in range(0, a.get_width(), 2):
			var ca := a.get_pixel(x, y)
			var cb := b.get_pixel(x, y)
			if absf(ca.r - cb.r) + absf(ca.g - cb.g) + absf(ca.b - cb.b) > 0.02:
				n += 1
	return n


## Edge contrast measured over ONLY the pixels the settlement layer owns --
## the difference between the layer-on and layer-off frames.  Terrain edges
## (coastlines, rivers) are magnified by the same camera and would otherwise
## dominate the statistic, which is the contamination this isolation removes.
func _mask_measure(on: Image, off: Image) -> Dictionary:
	var deltas := PackedFloat32Array()
	var ink := 0
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in on.get_height():
		var prev_in := false
		var prev := 0.0
		for x in on.get_width():
			var ca := on.get_pixel(x, y)
			var cb := off.get_pixel(x, y)
			var owned := absf(ca.r - cb.r) + absf(ca.g - cb.g) + absf(ca.b - cb.b) > 0.02
			var l := _lum(ca)
			if owned:
				ink += 1
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
				if prev_in:
					var d := absf(l - prev)
					if d > 0.02:
						deltas.append(d)
			prev_in = owned
			prev = l
	var arr := Array(deltas)
	arr.sort()
	arr.reverse()
	var n: int = mini(16, arr.size())
	var top := 0.0
	for i in n:
		top += float(arr[i])
	return {
		"ink": ink,
		"top": (top / float(n)) if n > 0 else 0.0,
		"box": Rect2i(x0, y0, maxi(0, x1 - x0 + 1), maxi(0, y1 - y0 + 1)) if x1 >= 0 else Rect2i(),
	}


func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b


func _expect(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		_fail.append(what)
