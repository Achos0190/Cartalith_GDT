extends Node
## Ruling EU leftovers (2026-11-13): the V-shaped detour on seed 483920, in the
## real shell. Generates seed 483920 at 1024 x 656, 1 200 km, commits planned
## routes between towns with the way join ON through the planner's own commit
## (`JourneyPlannerView._commit_stops`), and reads each committed route as runs of
## the drawn way each point lies on: a V is way A, then a stretch on another way
## B (or on none), then way A again, with the stretch at least `V_MIN_OFF` cells off A.
##
##   scan:  Godot_v4.7.2-stable_win64_console.exe --path . _wayv_probe.tscn -- --out <dir> --scan
##          lists the five town pairs of the app's world with the most V points (names,
##          where the V is, the way runs) and exits 0; no frame.
##   frame: ... -- --out <dir> --tag before|after --from <town> --to <town> [--at x,y]
##          commits that one route, prints its way runs and V points, and saves one
##          frame at x16 centred on `--at` (or on the first V point when the route has one).
##
## WINDOWED on purpose (`MISTAKES.md`: pixel probes run windowed); refuses to run
## headless. Prints `WAYV ...` lines; exit 0 when it finished, 1 on a failed check,
## 2 when it cannot run. The same arguments run against a build with the repair
## (`way_join::rejoin_way_detours`) and one without give the before and after frame.
##
## Why a scan: the verifier's town names (Orenelywash, Deleshshallows) are on this
## world but the routes between them were already one road in this build, so the
## scan finds a V on this world instead of assuming one.

const SEED := 483920
const GRID := Vector2i(1024, 656)
const KM := 1200.0
## Pairs scanned: the most populous towns, each pair between these shares of the
## map's width apart (the census's own span rule, widened a little for more routes).
const SCAN_TOWNS := 24
const MIN_SPAN := 0.03
const MAX_SPAN := 0.4
## A stretch this far off the way it left (cells) is a V (the verifier saw about 3), not a junction's wobble.
const V_MIN_OFF := 2.5
## A point within this many cells of a drawn way is on it.
const ON_WAY := 0.4

var app: Node
var out_dir := ""
var tag := "run"
var scan := false
var from_name := ""
var to_name := ""
var at_given := Vector2.ZERO
var ways: Array = []
## Each way's bounding box, padded by `ON_WAY`, so a point is only measured against the ways it can lie on.
var boxes: Array = []

func _p(s: String) -> void:
	print("WAYV " + s)

func _fail(s: String) -> void:
	print("WAYV FAIL: " + s)
	get_tree().quit(1)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The distance from `q` to way `wi`'s polyline.
func _dist_way(wi: int, q: Vector2) -> float:
	var w: PackedVector2Array = ways[wi]
	var best := INF
	for k in range(w.size() - 1):
		best = minf(best, Geometry2D.get_closest_point_to_segment(q, w[k], w[k + 1]).distance_to(q))
	return best

## `pts` as runs of the drawn way each point lies on: `[way or -1, points, first point]`.
func _runs(pts: PackedVector2Array) -> Array:
	var runs: Array = []
	for q in pts:
		var on_w := -1
		var best_d := ON_WAY
		for wi in ways.size():
			if not (boxes[wi] as Rect2).has_point(q):
				continue
			var d := _dist_way(wi, q)
			if d < best_d:
				best_d = d
				on_w = wi
		if runs.is_empty() or int(runs[runs.size() - 1][0]) != on_w:
			runs.append([on_w, 0, q])
		runs[runs.size() - 1][1] = int(runs[runs.size() - 1][1]) + 1
	return runs

## The V's of a route: `[count, first V's first point]`. A V is a run `r` between two
## runs of one way A (`runs[r-1]` and `runs[r+1]`, A >= 0), `r` itself on another
## way or none, whose points (from `pts`, `first` is the index of its first) stray at
## least `V_MIN_OFF` cells from A.
func _vs(pts: PackedVector2Array, runs: Array) -> Array:
	var n := 0
	var at := Vector2.ZERO
	var first := 0
	for r in runs.size():
		var cnt := int(runs[r][1])
		if r > 0 and r + 1 < runs.size() and int(runs[r - 1][0]) >= 0 and int(runs[r - 1][0]) == int(runs[r + 1][0]) and int(runs[r][0]) != int(runs[r - 1][0]):
			var far := 0.0
			var far_q := Vector2.ZERO
			for k in range(first, first + cnt):
				var d := _dist_way(int(runs[r - 1][0]), pts[k])
				if d > far:
					far = d
					far_q = pts[k]
			if far >= V_MIN_OFF:
				n += 1
				if at == Vector2.ZERO:
					at = far_q
		first += cnt
	return [n, at]

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("WAYV FAIL: run windowed, not --headless")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--out" and i + 1 < args.size():
			out_dir = String(args[i + 1])
			i += 1
		elif a == "--tag" and i + 1 < args.size():
			tag = String(args[i + 1])
			i += 1
		elif a == "--from" and i + 1 < args.size():
			from_name = String(args[i + 1])
			i += 1
		elif a == "--to" and i + 1 < args.size():
			to_name = String(args[i + 1])
			i += 1
		elif a == "--at" and i + 1 < args.size():
			var xy := String(args[i + 1]).split(",")
			at_given = Vector2(float(xy[0]), float(xy[1]))
			i += 1
		elif a == "--scan":
			scan = true
		else:
			print("WAYV FAIL: unknown argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if out_dir == "" or (not scan and (from_name == "" or to_name == "")):
		print("WAYV FAIL: --out <dir> and either --scan or --from/--to are required")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	get_window().size = Vector2i(1600, 1000)
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	var wg = bridge.world_gen
	bridge.generate({"seed": SEED, "width_km": KM, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "metropolis": false, "sea_level": 0.42})
	var spins := 0
	while bridge.generating and spins < 4000:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await _frames(10)
	if not bridge.has_world:
		_fail("generate failed")
		return
	for w in 20000:
		var d: Dictionary = bridge.way_get("generated", w)
		if d.is_empty():
			break
		if not bool(d.get("hidden", false)):
			ways.append(d.get("points", PackedVector2Array()))
	if ways.size() < 3:
		_fail("no network")
		return
	for w in ways:
		var wp: PackedVector2Array = w
		var box := Rect2(wp[0], Vector2.ZERO)
		for q in wp:
			box = box.expand(q)
		boxes.append(box.grow(ON_WAY))
	var setts: Array = bridge.settlements()
	var at := {}
	for s in setts:
		var sd: Dictionary = s
		at[String(sd.get("name", ""))] = Vector2(float(sd.get("x", 0)), float(sd.get("y", 0)))
	wg.set_route_join_ways(true)
	var jpv = app.journey_planner_view
	if scan:
		var idx: Array = range(setts.size())
		idx.sort_custom(func(a, b): return int(setts[a].get("population", 0)) > int(setts[b].get("population", 0)))
		idx = idx.slice(0, SCAN_TOWNS)
		var found: Array = []
		var routes := 0
		for a in idx.size():
			for b in range(a + 1, idx.size()):
				var pa := Vector2(float(setts[idx[a]].get("x", 0)), float(setts[idx[a]].get("y", 0)))
				var pb := Vector2(float(setts[idx[b]].get("x", 0)), float(setts[idx[b]].get("y", 0)))
				var dd := pa.distance_to(pb)
				if dd < MIN_SPAN * GRID.x or dd > MAX_SPAN * GRID.x:
					continue
				var ri: int = jpv._commit_stops([{"x": pa.x, "y": pa.y}, {"x": pb.x, "y": pb.y}])
				if ri < 0:
					continue
				routes += 1
				var pts: PackedVector2Array = bridge.route_get(ri).get("points", PackedVector2Array())
				## A fingerprint of the route (point count and a checksum of its
				## coordinates), so a scan of a build with the repair and one without
				## can be compared route by route: a route that differs was repaired.
				var sum := 0.0
				for q in pts:
					sum += q.x * 3.0 + q.y * 7.0
				_p("R|%s|%s|%d|%.2f" % [String(setts[idx[a]].get("name", "")), String(setts[idx[b]].get("name", "")), pts.size(), sum])
				var v := _vs(pts, _runs(pts))
				if int(v[0]) > 0:
					found.append([int(v[0]), String(setts[idx[a]].get("name", "")), String(setts[idx[b]].get("name", "")), v[1]])
		found.sort_custom(func(x, y): return int(x[0]) > int(y[0]))
		_p("%s: scanned %d routes, %d have a V" % [tag, routes, found.size()])
		for f in found.slice(0, 5):
			_p("%s: V x%d %s -> %s at %.1f,%.1f" % [tag, int(f[0]), String(f[1]), String(f[2]), (f[3] as Vector2).x, (f[3] as Vector2).y])
		get_tree().quit(0)
		return
	if not (at.has(from_name) and at.has(to_name)):
		_fail("%s or %s is not on this world" % [from_name, to_name])
		return
	var pa: Vector2 = at[from_name]
	var pb: Vector2 = at[to_name]
	var rj: int = jpv._commit_stops([{"x": pa.x, "y": pa.y}, {"x": pb.x, "y": pb.y}])
	if rj < 0:
		_fail("the route did not commit")
		return
	var rpts: PackedVector2Array = bridge.route_get(rj).get("points", PackedVector2Array())
	var runs := _runs(rpts)
	var seq := []
	for r in runs:
		seq.append("%d:%d" % [int(r[0]), int(r[1])])
	var v := _vs(rpts, runs)
	var flat := []
	for q in rpts:
		flat.append("%.1f,%.1f" % [q.x, q.y])
	_p("PTS|" + ";".join(flat))
	_p("%s: %s -> %s: %d points; way runs (way:points) %s; V stretches %d%s" % [tag, from_name, to_name, rpts.size(), " ".join(seq), int(v[0]),
		("; first at %.1f,%.1f" % [(v[1] as Vector2).x, (v[1] as Vector2).y]) if int(v[0]) > 0 else ""])
	var focus: Vector2 = at_given if at_given != Vector2.ZERO else (v[1] as Vector2)
	if focus == Vector2.ZERO:
		_fail("this route has no V to frame and no --at was given")
		return
	app.viewport.overlay.set_manual_routes([bridge.route_get(rj)])
	app.viewport.reset_view()
	await _frames(3)
	app.viewport.zoom_step(16.0)
	await _frames(3)
	app.viewport.move_view_to(focus.x, focus.y)
	await get_tree().create_timer(2.5).timeout
	await _frames(6)
	await RenderingServer.frame_post_draw
	var path := "%s/wayv_%s.png" % [out_dir, tag]
	get_viewport().get_texture().get_image().save_png(path)
	_p("shot %s" % path)
	_p("DONE")
	get_tree().quit(0)
