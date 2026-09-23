extends Node
## Committed probe -- addon villages get a road (`_civConnectVillageAddons`,
## `OUTSTANDING_WORK.md`'s "149 of 200 addon villages have no road" row).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _villageroads_probe.tscn
##
## **Windowed, not `--headless`**: part B reads pixels, and a texture/viewport
## read under `--headless` passes vacuously (`MISTAKES.md`). Reads no
## command-line flags.
##
## A -- DATA. Generates seed 483920 at 512x384 with villages on (the seed and
## size the original 149/200 finding measured) through the real bridge, then
## for every addon village (`kind == "hamlet"`, `population == 0` -- the same
## test `map_overlay.gd`'s `_settlement_hidden` uses) measures the distance to
## the nearest segment of any `get_roads()` way. Counted twice: over every way
## ("after"), and over every way except the `village_addon` ones ("before" --
## those are the only ways this change adds, and the rest of the network is
## built exactly as it was).
##
## B -- PIXELS. Feeds the same real roads into the real `map_overlay.gd` in a
## SubViewport and diffs the framebuffer with and without the village
## connectors: at zoom 3.0 (past `VILLAGE_ADDON_LOD = 2.4`) they must move
## pixels; at zoom 1.0 (past `ancient`'s own 0.7 but below 2.4) they must move
## NONE -- a connector never draws before its village. Positive control at 1.0:
## removing the ordinary (non-connector) ways must move pixels, so the zero is
## not a blank capture.

const REACH := 1.5   ## cells: "has a road" = some way passes within this
const VW := 1024
const VH := 768

var _app: Node


func _p(s: String) -> void:
	print("VILLAGEROADS  %s" % s)


func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var ab := b - a
	var l2 := ab.length_squared()
	if l2 < 1e-12:
		return p.distance_to(a)
	var t := clampf((p - a).dot(ab) / l2, 0.0, 1.0)
	return p.distance_to(a + ab * t)


func _nearest(p: Vector2, roads: Array) -> float:
	var best := INF
	for w in roads:
		var pts: PackedVector2Array = w["points"]
		var brks: PackedInt32Array = w["brks"]
		for k in range(1, pts.size()):
			if brks.has(k):
				continue
			best = minf(best, _seg_dist(p, pts[k - 1], pts[k]))
		if pts.size() == 1:
			best = minf(best, p.distance_to(pts[0]))
	return best


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	var settlements: Array = bridge.settlements()
	var roads: Array = bridge.roads()
	var seas: Array = bridge.sea_routes()
	var base_roads: Array = roads.filter(func(w): return not bool(w.get("village_addon", false)))
	var conn: Array = roads.filter(func(w): return bool(w.get("village_addon", false)))

	## -- A: data ----------------------------------------------------------
	var n := 0
	var miss_before := 0
	var miss_after := 0
	var worst_after := 0.0
	var worst_before := 0.0
	var tiers := {}
	for s in settlements:
		var p := Vector2(float(s["x"]), float(s["y"]))
		var addon: bool = s["kind"] == "hamlet" and int(s["population"]) == 0
		var d_all := _nearest(p, roads)
		if not addon:
			var t: String = s["kind"]
			var row: Array = tiers.get(t, [0, 0])
			row[0] += 1
			if d_all <= REACH:
				row[1] += 1
			tiers[t] = row
			continue
		n += 1
		var d_base := _nearest(p, base_roads)
		if d_base > REACH:
			miss_before += 1
			worst_before = maxf(worst_before, d_base)
		if d_all > REACH:
			miss_after += 1
			worst_after = maxf(worst_after, d_all)
			var near_place := INF
			for q in settlements:
				if q != s:
					near_place = minf(near_place, p.distance_to(Vector2(float(q["x"]), float(q["y"]))))
			_p("  unreached addon village at (%d,%d): nearest way %.1f cells, nearest other settlement %.1f cells" % [
				int(p.x), int(p.y), d_all, near_place])
			## The connector routes on the 384-wide routing grid, whose cell
			## (round(x*sc), round(y*sc)) samples full-res cell
			## (rx/sc|0, ry/sc|0) -- `_civRoutingGrid`/`_civLandCostGrid`. If
			## THAT cell is water, the village's routing cell costs Infinity
			## and is unreachable in the reference too.
			var sc := minf(512.0, 384.0) / 512.0
			var rx := roundi(p.x * sc)
			var ry := roundi(p.y * sc)
			var fx := int(rx / sc)
			var fy := int(ry / sc)
			_p("    routing cell (%d,%d) samples full-res (%d,%d): water=%s; the village's own cell: water=%s" % [
				rx, ry, fx, fy, str(bridge.sample_cell(fx, fy).get("water", "?")),
				str(bridge.sample_cell(int(p.x), int(p.y)).get("water", "?"))])
	var ancient := 0
	for w in conn:
		if w["way_type"] == "ancient":
			ancient += 1
	_p("world: %d settlements, %d ways (%d village connectors, all 'ancient': %s), %d sea routes" % [
		settlements.size(), roads.size(), conn.size(), str(ancient == conn.size()), seas.size()])
	for t in tiers:
		_p("  tier %-10s reached %d/%d" % [t, tiers[t][1], tiers[t][0]])
	_p("addon villages: %d" % n)
	_p("  BEFORE (network without connectors): %d/%d with no way within %.1f cells, worst %.1f" % [miss_before, n, REACH, worst_before])
	_p("  AFTER  (with connectors):            %d/%d with no way within %.1f cells, worst %.1f" % [miss_after, n, REACH, worst_after])

	## -- B: pixels --------------------------------------------------------
	var vp := SubViewport.new()
	vp.size = Vector2i(VW, VH)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(VW, VH)
	bg.color = Color.BLACK
	vp.add_child(bg)
	var ov := Control.new()
	ov.set_script(load("res://map_overlay.gd"))
	ov.size = Vector2(VW, VH)
	vp.add_child(ov)
	ov.set_show_settlements(false)

	var hi_all := await _shot(ov, 3.0, roads)
	var hi_base := await _shot(ov, 3.0, base_roads)
	var lo_all := await _shot(ov, 1.0, roads)
	var lo_base := await _shot(ov, 1.0, base_roads)
	var lo_none := await _shot(ov, 1.0, conn)
	var d_hi := _diff(hi_all, hi_base)
	var d_lo := _diff(lo_all, lo_base)
	var d_ctl := _diff(lo_all, lo_none)
	_p("pixels moved by the connectors at zoom 3.0 (> 2.4): %d" % d_hi)
	_p("pixels moved by the connectors at zoom 1.0 (< 2.4): %d" % d_lo)
	_p("control: pixels moved by the ordinary ways at zoom 1.0: %d" % d_ctl)

	var ok := n > 0 and conn.size() > 0 and ancient == conn.size() \
		and miss_after < miss_before and d_hi > 0 and d_lo == 0 and d_ctl > 0
	_p("RESULT %s" % ("PASS" if ok else "FAIL"))
	get_tree().quit(0 if ok else 1)


func _shot(ov: Control, zoom: float, roads: Array) -> Image:
	ov.set_camera_zoom(zoom)
	ov.set_civ_data([], roads, [], 512, 384, 0.0)
	ov.queue_redraw()
	for i in 4:
		await RenderingServer.frame_post_draw
	return ov.get_viewport().get_texture().get_image()


func _diff(a: Image, b: Image) -> int:
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			var ca := a.get_pixel(x, y)
			var cb := b.get_pixel(x, y)
			if absf(ca.r - cb.r) + absf(ca.g - cb.g) + absf(ca.b - cb.b) > 0.004:
				n += 1
	return n
