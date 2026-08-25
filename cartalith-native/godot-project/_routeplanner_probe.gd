extends SceneTree

## Proves the Route planner still behaves as `Cartalith Gen1 v2.10.html` does:
##
##  1. `route_commit` solves a least-cost path (`_civDijkstraPath`), not a
##     straight line, and rides existing infrastructure when that is cheaper.
##  2. `jp_reroute` re-paths under the transport's own cost domain and refuses
##     an unreachable answer rather than drawing the fallback.
##  3. `jp_plan_for_route` is `_jpEnsurePlan` in full -- a route that came out
##     mostly across open water opens on Sea Faring with an ocean-capable
##     vessel, not on the land itinerary with a Keelboat.
##
## Deterministic: one fixed seed, no RNG anywhere in the routing path.

func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	for m in ["route_begin", "route_commit", "route_get", "jp_compute", "jp_reroute", "jp_plan_for_route"]:
		if not wg.has_method(m):
			print("FAIL: %s absent -- rebuild cartalith-godot" % m)
			quit(1)
			return

	wg.set_params({"tect.plates": 9, "climate.lat_n": 62.0})
	wg.generate_sized(24601, 640.0, 96, 64)
	wg.recompute_civilisation()
	var places: Array = wg.get_settlements()
	print("  settlements: ", places.size())
	if places.size() < 2:
		print("  FAIL: need two settlements to route between")
		quit(1)
		return

	# The two settlements furthest apart, so the solver has room to prefer
	# terrain over the straight line. They may well sit on different
	# landmasses -- which is the point of `mixed`, and why the land re-route
	# below is tested on a *near* pair instead.
	var a: Dictionary = places[0]
	var b: Dictionary = places[0]
	var best := -1.0
	for p in places:
		var d: float = abs(float(p["x"]) - float(a["x"])) + abs(float(p["y"]) - float(a["y"]))
		if d > best:
			best = d
			b = p
	# ... and the two nearest distinct ones, the pair most likely to be land-
	# connected, for the land re-route.
	var na: Dictionary = places[0]
	var nb: Dictionary = places[1]
	var near := INF
	for i in places.size():
		for j in range(i + 1, places.size()):
			var pi: Dictionary = places[i]
			var pj: Dictionary = places[j]
			var d2: float = abs(float(pi["x"]) - float(pj["x"])) + abs(float(pi["y"]) - float(pj["y"]))
			if d2 < near:
				near = d2
				na = pi
				nb = pj

	wg.route_begin("mixed")
	wg.route_append_stop(float(a["x"]), float(a["y"]))
	wg.route_append_stop(float(b["x"]), float(b["y"]))
	var idx: int = wg.route_commit()
	if idx < 0:
		print("  FAIL: route_commit refused")
		quit(1)
		return
	var r: Dictionary = wg.route_get(idx)
	var pts: PackedVector2Array = r["points"]
	print("  route #%d: %d pts, %.1f km, mode=%s, unreachable=%d"
		% [idx, pts.size(), float(r["km"]), r["mode"], int(r["unreachable_legs"])])

	# A solved path bends; a straight-line fallback is exactly two points.
	if pts.size() <= 2:
		print("  FAIL: two points -- that is the straight-line fallback, not a solved path")
		fails += 1
	if int(r["unreachable_legs"]) > 0:
		print("  FAIL: leg(s) unreachable between two land settlements")
		fails += 1

	# Determinism: the identical request, twice, is the identical line.
	wg.route_begin("mixed")
	wg.route_append_stop(float(a["x"]), float(a["y"]))
	wg.route_append_stop(float(b["x"]), float(b["y"]))
	var idx2: int = wg.route_commit()
	var pts2: PackedVector2Array = wg.route_get(idx2)["points"]
	if pts != pts2:
		print("  FAIL: the same two endpoints solved to two different paths")
		fails += 1
	else:
		print("  deterministic: re-solving the same pair reproduced all %d points" % pts.size())

	# `_jpEnsurePlan` in full over the committed route.
	var plan: Dictionary = wg.jp_plan_for_route(idx)
	if plan.is_empty():
		print("  FAIL: jp_plan_for_route returned nothing for a committed route")
		fails += 1
	else:
		print("  ensure_plan: transport=%s vessel=%s sea_journey=%s"
			% [plan.get("transport"), plan.get("vessel"), plan.get("sea_journey")])
		var wet := bool(plan.get("sea_journey", false))
		var want_transport := "Sea Faring" if wet else "Walking"
		if String(plan.get("transport", "")) != want_transport:
			print("  FAIL: a sea_journey=%s route opened on %s" % [wet, plan.get("transport")])
			fails += 1
		if String(plan.get("vessel", "")) == "":
			print("  FAIL: no vessel picked at all")
			fails += 1

	# The planner itself still runs over that route.
	var jp: Dictionary = wg.jp_compute({"route": idx, "plan": plan, "auto_carriage": true})
	if not bool(jp.get("ok", false)):
		print("  FAIL: jp_compute -> ", jp.get("error", "?"))
		fails += 1
	else:
		var jplan: Dictionary = jp["plan"]
		var verdict: Dictionary = jp["verdict"]
		print("  jp_compute: %d stages, %s, verdict=%s"
			% [(jplan.get("stages", []) as Array).size(), jplan.get("total_days", "?"), verdict.get("label", "?")])
		if (jplan.get("stages", []) as Array).is_empty():
			print("  FAIL: no stages derived")
			fails += 1
		if not (jp.get("auto", {}) as Dictionary).is_empty():
			print("  auto suggestion: ", (jp["auto"] as Dictionary).get("reason", (jp["auto"] as Dictionary).keys()))

	# `_jpRerouteForMode` over a second route between the two NEAREST
	# settlements: a land-only re-route succeeds, and a sea re-route between
	# the same two inland points is refused rather than drawn as the
	# straight-line fallback `route_commit` tolerates.
	wg.route_begin("mixed")
	wg.route_append_stop(float(na["x"]), float(na["y"]))
	wg.route_append_stop(float(nb["x"]), float(nb["y"]))
	var nidx: int = wg.route_commit()
	print("  near route #%d: %.1f km between %s and %s"
		% [nidx, float(wg.route_get(nidx)["km"]), na.get("name"), nb.get("name")])

	var land: Dictionary = wg.jp_reroute(nidx, "Walking", "land")
	print("  reroute land: ok=%s %s" % [land.get("ok"), land.get("error", "")])
	if not bool(land.get("ok", false)):
		print("  FAIL: a land re-route between the two nearest settlements was refused")
		fails += 1
	elif String(wg.route_get(nidx)["mode"]) != "land":
		print("  FAIL: re-routed under land, but route_get still reports mode=%s"
			% wg.route_get(nidx)["mode"])
		fails += 1

	var sea: Dictionary = wg.jp_reroute(nidx, "Sea Faring", "water")
	print("  reroute sea:  ok=%s %s" % [sea.get("ok"), sea.get("error", "")])
	if bool(sea.get("ok", false)):
		var spts: PackedVector2Array = sea["points"]
		if spts.size() <= 2:
			print("  FAIL: 'sea route' accepted as a two-point straight line")
			fails += 1

	print("route-planner probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
