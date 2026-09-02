extends Node
## Committed probe -- the REAL app half of `_waycolor_probe.gd`.
##
##   Godot_v4.7.1-stable_win64.exe --path . _wayreal_probe.tscn
##
## Generates a real world, finds a way of every generated tier plus a sea lane,
## commits a Route deliberately along an existing road, and then measures the
## rendered pixels two shots at a time: one with the way layers on, one with
## them off. That second shot IS the local background, so at any drawn pixel
##
##   result = C * ae + dst * (1 - ae)
##
## has one unknown per channel (`ae`, the effective alpha at that pixel,
## coverage included). Solving it channel-by-channel and checking the three
## agree confirms C -- the type's own composited colour -- at whatever partial
## coverage a 1.2 px stroke actually lands, with no assumption that the stroke
## fully covers a pixel. Scoring every candidate type against the same pixel
## makes it a discrimination test, not just a match test.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge

## Reference `drawCivLayer` §2a/§2b constants -> full-coverage composite colour
## over an arbitrary background. Same table as `_waycolor_probe.gd`.
var _ref := {
	"highway":  {"u": [20, 10, 5], "ua": 0.55, "o": [210, 145, 55], "oa": 0.98},
	"regional": {"u": [25, 14, 5], "ua": 0.45, "o": [178, 118, 52], "oa": 0.88},
	"road":     {"u": [30, 20, 10], "ua": 0.40, "o": [160, 100, 60], "oa": 0.75},
	"track":    {"u": [30, 20, 10], "ua": 0.35, "o": [100, 120, 60], "oa": 0.75},
	"ancient":  {"u": [20, 10, 5], "ua": 0.35, "o": [120, 110, 100], "oa": 0.65},
	"sea_lane": {"u": [10, 30, 60], "ua": 0.40, "o": [30, 130, 200], "oa": 0.70},
	"route":    {"u": [40, 25, 5], "ua": 0.50, "o": [200, 160, 60], "oa": 0.85},
}


func _p(s: String) -> void:
	print("WAYREAL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


## The colour a fully-covered pixel of this type takes when composited over
## `dst` -- underlayer, then overlay, ordinary source-over. Returned 0-255.
func _composite(t: String, dst: Color) -> Vector3:
	var d: Dictionary = _ref[t]
	var ua: float = d["ua"]
	var oa: float = d["oa"]
	var out := Vector3()
	for i in 3:
		var bg: float = [dst.r, dst.g, dst.b][i] * 255.0
		var c1: float = float(d["u"][i]) * ua + bg * (1.0 - ua)
		out[i] = float(d["o"][i]) * oa + c1 * (1.0 - oa)
	return out


## Solve `result = C*ae + dst*(1-ae)` per channel and return
## [mean ae, spread of ae across channels]. A channel whose `C - dst` is tiny
## carries no information and is skipped.
func _solve(result: Color, dst: Color, c: Vector3) -> Array:
	var vals: Array = []
	for i in 3:
		var bg: float = [dst.r, dst.g, dst.b][i] * 255.0
		var rv: float = [result.r, result.g, result.b][i] * 255.0
		var denom: float = c[i] - bg
		if absf(denom) < 12.0:
			continue
		vals.append((rv - bg) / denom)
	if vals.is_empty():
		return [0.0, 99.0]
	var m := 0.0
	for v in vals:
		m += v
	m /= vals.size()
	var spread := 0.0
	for v in vals:
		spread = maxf(spread, absf(v - m))
	return [m, spread]


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_app.open_project_dialog.hide()
	await _frames(2)

	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)

	var gs: Vector2i = _bridge.grid_size()
	var vp = _app.viewport
	var ov = vp.overlay

	# ---- 1. what the world actually contains, by type ------------------------
	var roads: Array = _bridge.roads()
	var seas: Array = _bridge.sea_routes()
	var by_type := {}
	for w: Dictionary in roads:
		var t := String(w.get("way_type", "road"))
		if not by_type.has(t):
			by_type[t] = []
		by_type[t].append(w)
	var tally := ""
	for t in by_type:
		tally += "%s=%d " % [t, (by_type[t] as Array).size()]
	_p("world %s -- %d ways (%s), %d sea lanes" % [gs, roads.size(), tally.strip_edges(), seas.size()])

	# ---- 2. a route deliberately laid ALONG an existing road -----------------
	# Its stops are two vertices of the longest highway, so the solved path
	# should follow that road for most of its length -- the layering case.
	var host: Array = by_type.get("highway", by_type.get("regional", by_type.get("road", [])))
	host.sort_custom(func(a, b): return (a as Dictionary).points.size() > (b as Dictionary).points.size())
	var hp: PackedVector2Array = (host[0] as Dictionary)["points"]
	_p("route host way: %s, %d points, %.0f km" % [
		String((host[0] as Dictionary).get("way_type", "?")), hp.size(),
		float((host[0] as Dictionary).get("km", 0.0))])
	var ok: bool = _bridge.route_begin("land")
	var placed := 0
	for f in [0.05, 0.5, 0.95]:
		var v: Vector2 = hp[int(hp.size() * f)]
		if _bridge.route_append_stop(v.x, v.y):
			placed += 1
	var ridx: int = _bridge.route_commit()
	_p("route_begin(land)=%s, %d stops, commit -> %d" % [ok, placed, ridx])
	if ridx < 0:
		_p("NO ROUTE -- aborting")
		get_tree().quit(1)
		return
	var rdict: Dictionary = _bridge.route_get(ridx)
	var rpts: PackedVector2Array = rdict.get("render_points", rdict["points"])
	_p("route: %.0f km, %d render points, %d unreachable legs" % [
		float(rdict.get("km", 0.0)), rpts.size(), int(rdict.get("unreachable_legs", 0))])
	ov.set_manual_routes([rdict])

	# ---- 3. quiet the map down to the way layers -----------------------------
	vp.set_layer_visible("settlements", false)
	ov.set_labels([])
	ov.set_manual_icons([])
	await _frames(2)
	vp.reset_view()
	await _frames(2)
	var rect: Rect2 = ov.displayed_rect()

	## Whole-network overview -- the "tell the tiers apart at a glance" shot.
	await _center_on(vp, rect, Vector2(gs.x * 0.5, gs.y * 0.5), 2.5)
	await _shot("overview", true)

	# ---- 4. one centred view per type, sampled on its own ---------------------
	# A single shared view is what the first run of this probe tried, and it
	# both put four of the six types off-screen and let the committed route --
	# which by construction runs ALONG a highway -- contaminate the highway's
	# own sample. Each type gets its own framing, and the route layer is hidden
	# for every land/sea sample.
	_p("--- measured composite vs. every candidate type (best match must be the way's own) ---")
	var samples: Array = []
	for t in ["highway", "regional", "road", "track", "ancient"]:
		if not by_type.has(t):
			_p("%-9s (none in this world)" % t)
			continue
		var ws: Array = by_type[t]
		ws.sort_custom(func(a, b): return (a as Dictionary).points.size() > (b as Dictionary).points.size())
		samples.append([t, (ws[0] as Dictionary)["points"]])
	if not seas.is_empty():
		samples.append(["sea_lane", (seas[0] as Dictionary)["points"]])
	samples.append(["route", rpts])

	for s in samples:
		var t: String = s[0]
		var pts: PackedVector2Array = s[1]
		ov.set_manual_routes([rdict] if t == "route" else [])
		await _center_on(vp, rect, pts[pts.size() / 2], 6.0)
		var img_on := await _shot("type_%s_on" % t, t == "highway" or t == "route")
		ov.set_show_roads(false)
		ov.set_show_sea_routes(false)
		await _frames(3)
		var img_off := await _shot("type_%s_off" % t, false)
		ov.set_show_roads(true)
		ov.set_show_sea_routes(true)
		await _frames(3)

		## Every OTHER way's pixels, so a sample can never be taken where two
		## ways cross. The background shot hides them all at once, so a crossing
		## pixel carries two stacked strokes over one background and cannot be
		## explained by either type alone -- that was the first run's whole
		## error bar.
		var foreign := _foreign_pixels(by_type, seas, rdict, pts, rect, vp)
		var votes := {}
		var own_res: Array = []
		var own_ae: Array = []
		var sampled := 0
		for i in range(pts.size()):
			var w := _to_window(pts[i], rect, vp)
			var x := int(round(w.x))
			var y := int(round(w.y))
			if x < 2 or y < 2 or x >= img_on.get_width() - 2 or y >= img_on.get_height() - 2:
				continue
			if _near_foreign(foreign, x, y):
				continue
			var res: Color = img_on.get_pixel(x, y)
			var dst: Color = img_off.get_pixel(x, y)
			if absf(res.r - dst.r) + absf(res.g - dst.g) + absf(res.b - dst.b) < 0.25:
				continue   ## edge of the stroke, too little signal to fit
			sampled += 1
			var best := ""
			var best_r := 1e9
			for cand in _ref:
				var fit := _fit(res, dst, _composite(cand, dst))
				if cand == t:
					own_res.append(fit[1])
					own_ae.append(fit[0])
				if fit[1] < best_r:
					best_r = fit[1]
					best = cand
			votes[best] = int(votes.get(best, 0)) + 1
		if sampled == 0:
			_p("%-9s no clean (non-crossing) on-screen pixel found" % t)
			continue
		var vtally := ""
		for k in votes:
			vtally += "%s=%d " % [k, votes[k]]
		_p("%-9s %d clean pixels | own-type median residual %.1f/255, median ae %.2f | best-fit votes: %s %s" % [
			t, sampled, _median(own_res), _median(own_ae), vtally.strip_edges(),
			"OK" if int(votes.get(t, 0)) * 2 > sampled else "MISMATCH (expected %s to win)" % t])

	# ---- 5. layering: where the route runs over its host road ----------------
	_p("--- layering: pixels where the committed route overlaps its host way ---")
	ov.set_manual_routes([rdict])
	await _center_on(vp, rect, rpts[rpts.size() / 2], 8.0)
	var lay_on := await _shot("layer_on", true)
	ov.set_show_roads(false)
	ov.set_show_sea_routes(false)
	await _frames(3)
	var lay_off := await _shot("layer_off", false)
	ov.set_show_roads(true)
	ov.set_show_sea_routes(true)
	await _frames(3)
	await _layering_check(rpts, hp, rect, vp, lay_on, lay_off)

	# ---- 6. the CARTO dock's per-type filter must reach every type -----------
	_p("--- per-way-type filter: hiding a type must stop that type drawing ---")
	var carto := load("res://shell/workspaces/cartography_workspace.gd")
	var listed := {}
	for t in carto.WAY_TYPES:
		listed[String(t["key"])] = true
	_p("  CARTO 'Ways by type' lists: %s | world contains: %s" % [
		", ".join(PackedStringArray(listed.keys())), ", ".join(PackedStringArray(by_type.keys()))])
	ov.set_manual_routes([])
	for t in by_type:
		var ws2: Array = by_type[t]
		ws2.sort_custom(func(a, b): return (a as Dictionary).points.size() > (b as Dictionary).points.size())
		var wp: PackedVector2Array = (ws2[0] as Dictionary)["points"]
		await _center_on(vp, rect, wp[wp.size() / 2], 6.0)
		var shown_img := await _shot("filter_%s_shown" % t, false)
		vp.set_way_type_visible(t, false)
		await _frames(3)
		var hidden_img := await _shot("filter_%s_hidden" % t, false)
		vp.set_way_type_visible(t, true)
		await _frames(3)
		var changed := 0
		for i in range(wp.size()):
			var w := _to_window(wp[i], rect, vp)
			var x := int(round(w.x))
			var y := int(round(w.y))
			if x < 2 or y < 2 or x >= shown_img.get_width() - 2 or y >= shown_img.get_height() - 2:
				continue
			var a := shown_img.get_pixel(x, y)
			var b := hidden_img.get_pixel(x, y)
			if absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b) > 0.05:
				changed += 1
		_p("  %-9s listed in CARTO: %-5s | pixels that vanished when hidden: %d  %s" % [
			t, str(listed.has(t)), changed,
			"OK" if listed.has(t) and changed > 0 else "NOT FILTERABLE"])

	get_tree().quit(0)


## Reset, then zoom toward `zoom_target` keeping `cell` under the centre.
func _center_on(vp, rect: Rect2, cell: Vector2, zoom_target: float) -> void:
	var gs: Vector2i = _bridge.grid_size()
	vp.reset_view()
	await _frames(2)
	while vp.zoom() < zoom_target:
		vp.zoom_step(1.5)
		var l := rect.position + Vector2(cell.x / float(gs.x), cell.y / float(gs.y)) * rect.size
		vp._camera.position = vp.size * 0.5 - l * vp.zoom()
		await _frames(1)
	var l2 := rect.position + Vector2(cell.x / float(gs.x), cell.y / float(gs.y)) * rect.size
	vp._camera.position = vp.size * 0.5 - l2 * vp.zoom()
	await _frames(4)


## Overlay-local -> window pixel. `overlay` is a FULL_RECT child of `_camera`,
## a Control carrying `position` + uniform `scale`, itself a child of the
## viewport host, so the whole chain is one translate+scale.
func _to_window(p: Vector2, rect: Rect2, vp) -> Vector2:
	var gs: Vector2i = _bridge.grid_size()
	var local := rect.position + Vector2(p.x / float(gs.x), p.y / float(gs.y)) * rect.size
	return vp.global_position + local * vp.zoom() + vp._camera.position


func _median(a: Array) -> float:
	if a.is_empty():
		return -1.0
	a.sort()
	return a[a.size() / 2]


## Least-squares fit of `result = C*ae + dst*(1-ae)` over the three channels,
## `ae` clamped to a physically possible [0, 1.05]. Returns [ae, RMS residual
## in 0-255 units] -- residual, not alpha spread, is what says whether this
## candidate can explain the pixel at all.
func _fit(result: Color, dst: Color, c: Vector3) -> Array:
	var num := 0.0
	var den := 0.0
	for i in 3:
		var bg: float = [dst.r, dst.g, dst.b][i] * 255.0
		var rv: float = [result.r, result.g, result.b][i] * 255.0
		var d: float = c[i] - bg
		num += d * (rv - bg)
		den += d * d
	var ae: float = clampf(num / maxf(den, 1e-6), 0.0, 1.05)
	var sse := 0.0
	for i in 3:
		var bg2: float = [dst.r, dst.g, dst.b][i] * 255.0
		var rv2: float = [result.r, result.g, result.b][i] * 255.0
		var pred: float = bg2 + ae * (c[i] - bg2)
		sse += (rv2 - pred) * (rv2 - pred)
	return [ae, sqrt(sse / 3.0)]


## Window pixels covered by every way EXCEPT the one being sampled.
func _foreign_pixels(by_type: Dictionary, seas: Array, rdict: Dictionary,
		own: PackedVector2Array, rect: Rect2, vp) -> Dictionary:
	var out := {}
	var lists: Array = []
	for t in by_type:
		for w: Dictionary in by_type[t]:
			lists.append(w["points"])
	for s: Dictionary in seas:
		lists.append(s["points"])
	lists.append(rdict.get("render_points", rdict["points"]))
	for pts: PackedVector2Array in lists:
		if pts == own:
			continue
		for i in range(pts.size()):
			var w := _to_window(pts[i], rect, vp)
			out[Vector2i(int(round(w.x)), int(round(w.y)))] = true
	return out


func _near_foreign(foreign: Dictionary, x: int, y: int) -> bool:
	for dy in range(-2, 3):
		for dx in range(-2, 3):
			if foreign.has(Vector2i(x + dx, y + dy)):
				return true
	return false


## The pixel exactly under a way vertex -- rounded to nearest, never a windowed
## search. A ±3 px search was the first version and it systematically drifted
## off the stroke's core onto its dark underlayer fringe, because "most
## changed" over a light parchment background means "darkest", and the
## underlayer is always the darkest part of a two-stroke way. The vertex
## position is exact, so nearest-pixel is the honest sample; across vertices,
## the one that changed most is the one whose stroke centre landed closest to a
## pixel centre.
func _sample_strongest(pts: PackedVector2Array, rect: Rect2, vp, on: Image, off: Image) -> Dictionary:
	var best := {}
	var best_d := 0.0
	var step: int = maxi(1, pts.size() / 200)
	for i in range(0, pts.size(), step):
		var w := _to_window(pts[i], rect, vp)
		var x := int(round(w.x))
		var y := int(round(w.y))
		if x < 1 or y < 1 or x >= on.get_width() - 1 or y >= on.get_height() - 1:
			continue
		var a := on.get_pixel(x, y)
		var b := off.get_pixel(x, y)
		var d: float = absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)
		if d > best_d:
			best_d = d
			best = {"x": x, "y": y, "on": a, "off": b}
	return best


## The route must draw ON TOP of the way it follows. At every pixel where the
## route's own path coincides with the host way's, the rendered colour must be
## explainable as the route's composite and NOT as the host's.
func _layering_check(rpts: PackedVector2Array, hpts: PackedVector2Array, rect: Rect2, vp,
		on: Image, off: Image) -> void:
	var host_px := {}
	for i in range(hpts.size()):
		var w := _to_window(hpts[i], rect, vp)
		host_px[Vector2i(int(w.x), int(w.y))] = true
	var overlaps := 0
	var route_wins := 0
	var host_wins := 0
	var shown := 0
	for i in range(rpts.size()):
		var w := _to_window(rpts[i], rect, vp)
		var key := Vector2i(int(w.x), int(w.y))
		if not host_px.has(key):
			continue
		if key.x < 1 or key.y < 1 or key.x >= on.get_width() - 1 or key.y >= on.get_height() - 1:
			continue
		overlaps += 1
		var a := on.get_pixel(key.x, key.y)
		var b := off.get_pixel(key.x, key.y)
		var sr := _solve(a, b, _composite("route", b))
		var sh := _solve(a, b, _composite("highway", b))
		if sr[1] <= sh[1]:
			route_wins += 1
		else:
			host_wins += 1
		if shown < 5:
			shown += 1
			_p("  overlap @(%d,%d) on=(%3d,%3d,%3d) off=(%3d,%3d,%3d) route ae=%.2f s=%.2f | host ae=%.2f s=%.2f -> %s" % [
				key.x, key.y, int(a.r * 255), int(a.g * 255), int(a.b * 255),
				int(b.r * 255), int(b.g * 255), int(b.b * 255),
				sr[0], sr[1], sh[0], sh[1], "route on top" if sr[1] <= sh[1] else "HOST ON TOP"])
	_p("  %d coincident pixels: route explains %d, host explains %d  %s" % [
		overlaps, route_wins, host_wins,
		"OK" if overlaps > 0 and route_wins > host_wins else ("NO OVERLAP" if overlaps == 0 else "LAYERING WRONG")])


func _shot(name: String, keep: bool = true) -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if keep:
		img.save_png("user://wayreal_%s.png" % name)
		_p("shot -> %s" % ProjectSettings.globalize_path("user://wayreal_%s.png" % name))
	return img
