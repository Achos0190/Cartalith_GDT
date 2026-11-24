extends RefCounted
## Owner report 2026-11-23: *"the attachment from routes (sea and land) to the settlements when you zoom in"*. The sea
## half: every sea lane must end on its port's marker as drawn, at every zoom. Driven through the REAL shell on one
## form (see `docs/evidence/sea-lane-ends-2026-11-23/run_drive.py`, which holds `C:/cw/.win_lock` with an owner file).
## Inputs are environment variables, since the shell reads its own user arguments:
##   SL_OUT    the directory frames are saved to (required);
##   SL_SEED, SL_GW, SL_GH (default 51203, 2048, 1311: the owner's seed at the New World defaults, 800 km), made by
##             `EngineBridge.generate` with no world open, so the parameter table is the app's (`params::defaults`);
##   SL_ZOOMS  (default "1,4,16,64"); SL_FRAMES (default 3: lanes whose first end is framed at each zoom).
## For each zoom and each lane end that lies on a settlement's cell (all of them, on an engine-made lane) it measures,
## in window pixels, the gap from the end the overlay DREW the stroke with (`MapOverlay._sea_drawn_ends`, recorded by
## `_draw_sea_chunk` while it draws) to the settlement's marker (`MapOverlay._settle_pos`, what the pin is drawn at),
## and the same gap for the engine's own end (`sea_routes()`, the stroke before this fix: the "before" column).
## Checks (each prints `ok` or `FAIL`; the launcher exits 1 on any FAIL):
##   1. every drawn end is within END_GAP_PX of its marker, at every zoom;
##   2. the positive control: at the deepest zoom the engine's end is more than 5 px from the marker for some lane
##      (so check 1 can see an unattached end);
##   2b. no drawn lane runs past its marker and back (a spur to the cell corner): `_spur_px` within END_GAP_PX;
##   3. the new last segment crosses no land: sampled every quarter cell from the stroke's penultimate vertex to the
##      drawn end, it has no run of land over END_GAP_PX on screen with sea on both sides (a spit crossed) where the
##      engine's own last segment had none. Gated at x4 and deeper, where the owner's report is; at x1 the count is
##      printed (one end on the phone: 1.5 cells of land, about 1.3 px, between the coast lane and a marker Ruling HG
##      stands 4 cells inland on a river bank at that density; 2026-11-24).
## Mutant: remove `points = _attach_route_ends(points)` from `map_overlay.gd` in the project copy; check 1 goes red.

## The gate: a drawn sea-lane end may stand this many window pixels from its marker. A labelled judgement: one pixel,
## so the stroke's end is under the pin's own point at any zoom (the pin's head is ~16 px; the stroke 2-3 px wide).
const END_GAP_PX := 1.0

var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _past_welcome(ctx) -> void:
	if app.get_viewport().get_visible_rect().size.x < 700.0:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		if not nw.is_empty():
			var r: Array = nw["rect"]
			await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
			var cancel := await _wait_for(ctx, "CANCEL")
			if not cancel.is_empty():
				var r2: Array = cancel["rect"]
				await DriveCtx.click(ctx, Vector2(r2[0] + r2[2] / 2.0, r2[1] + r2[3] / 2.0))
		await ctx.wait(1.0)
		return
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		var r3: Array = cont["rect"]
		await DriveCtx.click(ctx, Vector2(r3[0] + r3[2] / 2.0, r3[1] + r3[3] / 2.0))
		await ctx.wait(0.6)


## A grid point (way units: no +0.5) in window pixels, as the overlay draws it now.
func _px(p: Vector2) -> Vector2:
	var ov = app.viewport.overlay
	return ov.get_global_transform_with_canvas() * ov._point_to_screen(p, ov.displayed_rect())


func _is_land(p: Vector2) -> bool:
	return String(bridge.world_gen.sample_cell(int(floor(p.x)), int(floor(p.y))).get("water", "land")) == "land"


## Whether segment a->b, sampled every quarter cell, crosses land that shows on screen: a run of land with sea on both
## sides of it (a spit or a headland cut) longer than END_GAP_PX window pixels at the zoom on screen. Ending on land
## (the town) is not crossing it, and a sea cell's corner clipped between two land samples (land, sea, land) is crossing
## water, not land. A labelled judgement on the pixel floor: at x1 on the phone a cell is half a pixel, and a run of land
## under a pixel cannot be seen as a crossing (one such run, 1.5 cells at x1 on the phone, measured 2026-11-24).
func _recrosses(a: Vector2, b: Vector2) -> bool:
	var n := maxi(2, int(ceil(a.distance_to(b) * 4.0)))
	var px_per_step := _px(a).distance_to(_px(b)) / n
	var sea_before := false
	var run := 0
	for k in n + 1:
		var land := _is_land(a.lerp(b, float(k) / n))
		if land and sea_before:
			run += 1
		elif not land:
			if run > 0 and run * px_per_step > END_GAP_PX:
				return true
			run = 0
			sea_before = true
	return false


## How far, in window pixels, the tail of `line` at end `e` (0 the first point, else the last) runs past its closest
## approach to `m` and back: walking the vertices within `reach` grid cells of that end toward it, the largest rise of
## the distance to `m` above the least distance seen so far. 0 for a tail that closes on `m` all the way.
func _spur_px(line: PackedVector2Array, e: int, m: Vector2, reach: float) -> float:
	var n := line.size()
	var end_p: Vector2 = line[0] if e == 0 else line[n - 1]
	var tail: Array = []
	for j in n:
		var v: Vector2 = line[j if e != 0 else n - 1 - j]   ## toward the end: last index last
		if v.distance_to(end_p) <= reach:
			tail.append(v)
	var mp := _px(m)
	var least := INF
	var rise := 0.0
	for v in tail:
		var d := _px(v).distance_to(mp)
		if d > least:
			rise = maxf(rise, d - least)
		least = minf(least, d)
	return rise


func _median(v: Array) -> float:
	if v.is_empty():
		return NAN
	v.sort()
	return float(v[v.size() / 2])


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var out := _env("SL_OUT", "")
	if out == "":
		ctx.error("SL_OUT is required")
		return
	DirAccess.make_dir_recursive_absolute(out)
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	var form := "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	ctx.log("form %s, window %s" % [form, str(size)])
	await _past_welcome(ctx)
	var req := {"seed": int(_env("SL_SEED", "51203")), "width_km": 800.0, "grid_w": int(_env("SL_GW", "2048")),
		"grid_h": int(_env("SL_GH", "1311")), "archetype": "", "villages": true}
	bridge.generate(req)
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 900000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	_check(ctx, "a world exists", bool(bridge.has_world))
	ctx.log("params: civ.sea_lanes %s, civ.bank_sites %s" % [str(bridge.param_get("civ.sea_lanes")), str(bridge.param_get("civ.bank_sites"))])
	var host = app.viewport
	var ov = host.overlay
	var routes: Array = bridge.sea_routes()
	var places: Array = bridge.settlements()
	var by_cell := {}
	for i in places.size():
		by_cell[Vector2i(int(places[i]["x"]), int(places[i]["y"]))] = i
	## (route, end 0|1, settlement) for every engine end on a settlement's cell.
	var ends: Array = []
	for ri in routes.size():
		var pts: PackedVector2Array = routes[ri]["points"]
		if pts.size() < 2:
			continue
		for e in [0, 1]:
			var p: Vector2 = pts[0] if e == 0 else pts[pts.size() - 1]
			var c := Vector2i(roundi(p.x), roundi(p.y))
			if p.is_equal_approx(Vector2(c)) and by_cell.has(c):
				ends.append([ri, e, int(by_cell[c])])
	ctx.log("sea lanes %d, ends on a settlement's cell %d" % [routes.size(), ends.size()])
	_check(ctx, "the world has lane ends to measure", ends.size() >= 4, str(ends.size()))
	var frames := int(_env("SL_FRAMES", "3"))
	var zooms: Array = []
	for z in _env("SL_ZOOMS", "1,4,16,64").split(",", false):
		zooms.append(float(z))
	## SL_HIDE_PINS=1: the settlement layer off, so a frame shows the lane's end that the pin otherwise covers (the
	## marker point is still `_settle_pos`, which does not depend on the layer being shown).
	if _env("SL_HIDE_PINS", "") == "1":
		ov.set_show_settlements(false)
	var crossed := 0       ## at x4 and deeper: the gate
	var crossed_x1 := 0    ## at the opening view: reported
	var raw_max_deep := 0.0
	for z in zooms:
		var drawn_gaps: Array = []
		var raw_gaps: Array = []
		var spurs: Array = []
		var raw_spurs: Array = []
		## Every end is measured with the view centred on it, so it is drawn (not culled) at this zoom.
		for k in ends.size():
			var ri: int = ends[k][0]
			var e: int = ends[k][1]
			## The overlay's own roster entry (same order as `settlements()`): it carries the bank offset the pin is drawn with.
			var s: Dictionary = ov._settlements[ends[k][2]]
			host.reset_view()
			if z > 1.0:
				host._zoom_at(host.size * 0.5, z)
				host.move_view_to(float(s["x"]), float(s["y"]))
			ov._sea_drawn.erase(ri)
			ov.queue_redraw()
			await ctx.wait(0.15 if k >= frames else (3.0 if z >= 16.0 else 1.5))
			var marker: Vector2 = ov.get_global_transform_with_canvas() * ov._settle_pos(s, ov.displayed_rect())
			var pts: PackedVector2Array = routes[ri]["points"]
			var raw: Vector2 = pts[0] if e == 0 else pts[pts.size() - 1]
			raw_gaps.append(_px(raw).distance_to(marker))
			var m_grid: Vector2 = ov._settle_grid(s)
			var reach: float = raw.distance_to(m_grid) + 1.0
			raw_spurs.append(_spur_px(pts, e, m_grid, reach))
			if not ov._sea_drawn.has(ri):
				drawn_gaps.append(INF)
				spurs.append(INF)
				continue
			var dl: PackedVector2Array = ov._sea_drawn[ri]
			var drawn: Vector2 = dl[0] if e == 0 else dl[dl.size() - 1]
			drawn_gaps.append(_px(drawn).distance_to(marker))
			spurs.append(_spur_px(dl, e, m_grid, reach))
			## Every zoom: the bank offset, and so the move, is largest at x1.
			var pen: Vector2 = dl[1] if e == 0 else dl[dl.size() - 2]
			if _recrosses(pen, drawn) and not _recrosses(pen, raw):
				if z > 1.0:
					crossed += 1
				else:
					crossed_x1 += 1
				var prof := ""
				for q in 17:
					prof += "L" if _is_land(pen.lerp(drawn, q / 16.0)) else "~"
				ctx.log("recross: lane %d end %d settlement %d (%s) pen %s drawn %s engine %s profile %s" % [ri, e, ends[k][2], String(s.get("name", "")), str(pen), str(drawn), str(raw), prof])
				var img2: Image = app.get_viewport().get_texture().get_image()
				var k2: float = float(img2.get_width()) / size.x
				var r2 := Rect2i(Vector2i(marker * k2) - Vector2i(150, 150), Vector2i(300, 300)).intersection(Rect2i(Vector2i.ZERO, img2.get_size()))
				img2.get_region(r2).save_png(out.path_join("%s_recross_lane%d_end%d_x%d.png" % [form, ri, e, int(z)]))
			if k < frames:
				ctx.log("frame x%d lane %d end %d: window px marker %s drawn end %s engine end %s drawn penultimate %s" % [int(z), ri, e, str(marker), str(_px(drawn)), str(_px(raw)), str(_px(pen))])
				var img: Image = app.get_viewport().get_texture().get_image()
				var kk: float = float(img.get_width()) / size.x
				var r := Rect2i(Vector2i(marker * kk) - Vector2i(150, 150), Vector2i(300, 300)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
				img.get_region(r).save_png(out.path_join("%s_lane%d_end%d_x%d.png" % [form, ri, e, int(z)]))
		var dmax: float = drawn_gaps.max() if not drawn_gaps.is_empty() else INF
		var rmax: float = raw_gaps.max() if not raw_gaps.is_empty() else 0.0
		ctx.log("x%d: ends %d | drawn gap px median %.2f max %.2f | engine end (before) median %.2f max %.2f" % [int(z), drawn_gaps.size(), _median(drawn_gaps.duplicate()), dmax, _median(raw_gaps.duplicate()), rmax])
		_check(ctx, "x%d: every drawn sea-lane end within %.0f px of its marker" % [int(z), END_GAP_PX], dmax <= END_GAP_PX, "max %.2f" % dmax)
		var smax: float = spurs.max() if not spurs.is_empty() else INF
		ctx.log("x%d: spur past the marker px | drawn median %.2f max %.2f | engine (before) median %.2f max %.2f" % [int(z), _median(spurs.duplicate()), smax, _median(raw_spurs.duplicate()), raw_spurs.max() if not raw_spurs.is_empty() else 0.0])
		_check(ctx, "x%d: no drawn sea-lane runs past its marker and back (spur within %.0f px)" % [int(z), END_GAP_PX], smax <= END_GAP_PX, "max %.2f" % smax)
		raw_max_deep = rmax
	_check(ctx, "positive control: the engine's own end stands off the marker at the deepest zoom", raw_max_deep > 5.0, "max %.2f px" % raw_max_deep)
	_check(ctx, "no moved end crosses land the engine's end did not, at x4 and deeper", crossed == 0, str(crossed))
	ctx.log("moved ends crossing land at x1 (reported, not gated): %d" % crossed_x1)
	ctx.log("done, %d failed" % _fails)
