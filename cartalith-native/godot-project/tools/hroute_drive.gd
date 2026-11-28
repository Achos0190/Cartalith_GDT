extends RefCounted
## Owner report 2026-11-28: a route between two hand-placed POIs is drawn short of both. Driven through the real shell:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop res://tools/hroute_drive.gd --dll DLL --out OUT
## Env: HR_SAVE (a scratch copy of the save, required), HR_OUT (frames and the report, required).
## Checks: (1) the POI-to-POI route the planner commits (route_begin/append/commit on the POIs' own points) has its first
## and last stored point on the POIs; (2) the same at x1/x4/x16: each drawn end is within 1 px of its POI's drawn point.
## Also reports every stored route's ends against the POIs (the owner's save).
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

func _px(p: Vector2) -> Vector2:
	var ov = app.viewport.overlay
	return ov.get_global_transform_with_canvas() * ov._point_to_screen(p, ov.displayed_rect())

func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var out := _env("HR_OUT", "")
	DirAccess.make_dir_recursive_absolute(out)
	for e in DriveCtx.dump_ui(ctx, "Continue without a world"):
		var r: Array = e["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		await ctx.wait(0.6)
		break
	var ok: bool = await app.open_project_busy(_env("HR_SAVE", ""))
	await ctx.wait(3.0)
	_check(ctx, "the save opens", ok)
	var pois: Array = bridge.poi_list()
	_check(ctx, "two POIs", pois.size() == 2, str(pois.size()))
	var a := Vector2(float(pois[0]["x"]), float(pois[0]["y"]))
	var b := Vector2(float(pois[1]["x"]), float(pois[1]["y"]))
	for i in bridge.route_count():
		var pts: PackedVector2Array = bridge.route_get(i)["points"]
		ctx.log("stored route %d: %d pts, first %s (%.1f from POI1), last %s (%.1f from POI2)" % [i, pts.size(), str(pts[0]), pts[0].distance_to(a), str(pts[pts.size() - 1]), pts[pts.size() - 1].distance_to(b)])
	var idx: int = -1
	## At the fit zoom the snap reach is widest (58 cells here): the POI itself must still win (mutant M1 removes that).
	bridge.snap_zoom = 1.0
	if bridge.route_begin("mixed"):
		bridge.route_append_stop(a.x, a.y)
		bridge.route_append_stop(b.x, b.y)
		idx = bridge.route_commit()
	_check(ctx, "the route commits", idx >= 0)
	app.viewport.overlay.set_manual_routes(app.viewport.manual_routes())
	var pts2: PackedVector2Array = bridge.route_get(idx)["points"]
	ctx.log("fresh route: first %s last %s (POI1 %s POI2 %s)" % [str(pts2[0]), str(pts2[pts2.size() - 1]), str(a), str(b)])
	_check(ctx, "stored first point is POI1", pts2[0].distance_to(a) < 0.01, str(pts2[0].distance_to(a)))
	_check(ctx, "stored last point is POI2", pts2[pts2.size() - 1].distance_to(b) < 0.01, str(pts2[pts2.size() - 1].distance_to(b)))
	## A free point (no POI) 8 cells from a road, asked at x16: the reach is 58/16 = 3.6 cells, so it must stay where it was put
	## (mutant M2 ignores the zoom and pulls it onto the road).
	var free := Vector2(1126.0, 1374.0)
	bridge.snap_zoom = 16.0
	bridge.route_begin("mixed")
	bridge.route_append_stop(free.x, free.y)
	bridge.route_append_stop(b.x, b.y)
	var fidx: int = bridge.route_commit()
	var fp: PackedVector2Array = bridge.route_get(fidx)["points"]
	_check(ctx, "a free stop at x16 keeps its own point", fidx >= 0 and fp[0].distance_to(free) < 0.01, str(fp[0]) if fidx >= 0 else "no route")
	var rr: Dictionary = bridge.route_get(idx)
	var rp: PackedVector2Array = rr.get("render_points", rr["points"])
	_check(ctx, "the drawn (render) points start on POI1 and end on POI2", rp[0].distance_to(a) < 0.01 and rp[rp.size() - 1].distance_to(b) < 0.01, "%s %s" % [str(rp[0]), str(rp[rp.size() - 1])])
	var v = app.viewport
	for z in [1.0, 4.0, 16.0]:
		for which in [0, 1]:
			var poi: Vector2 = a if which == 0 else b
			v._zoom_at(v.size * 0.5, z / v._zoom)
			var ov = app.viewport.overlay
			v._camera.position = v.size * 0.5 - ov._point_to_screen(poi, ov.displayed_rect()) * v._zoom
			await ctx.wait(1.0)
			var end: Vector2 = rp[0] if which == 0 else rp[rp.size() - 1]
			var gap := _px(end).distance_to(_px(poi))
			_check(ctx, "x%d: the drawn route end is within 1 px of POI%d" % [int(z), which + 1], gap <= 1.0, "%.2f px" % gap)
			ctx.capture("x%d_poi%d" % [int(z), which + 1])
	if _fails > 0:
		ctx.error("FAIL %d" % _fails)
