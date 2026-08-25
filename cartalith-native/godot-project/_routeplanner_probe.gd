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
	wg.generate_sized(24601, 2560.0, 256, 192)   ## >= 128 wide, so DECISIONS.md 7i corridors are live
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

	# DECISIONS.md 7j: per-stage auto-pick. Off and on over the same route, so
	# the difference is the feature and nothing else.
	# A laden merchant caravan, not the default lone walker: the per-stage
	# picker re-tacks a pack train, and a party without one has nothing to
	# re-tack (which is itself asserted in the Rust tests).
	var caravan: Dictionary = plan.duplicate(true)
	caravan["transport"] = "Baggage Train"
	caravan["group_size"] = 12
	caravan["cargo_kg"] = 900.0
	caravan["mule"] = 8
	caravan["horse"] = 2
	caravan["carts"] = 2
	var off: Dictionary = wg.jp_compute({"route": idx, "plan": caravan})
	var on: Dictionary = wg.jp_compute({"route": idx, "plan": caravan, "auto_stage": true})
	if not bool(on.get("ok", false)):
		print("  FAIL: auto_stage -> ", on.get("error", "?"))
		fails += 1
	else:
		var picks: Array = on.get("stage_picks", [])
		print("  auto_stage: %d pick(s)" % picks.size())
		for p in picks:
			var d: Dictionary = p
			print("    stage %d %s: %s%s%s -- %.1f -> %.1f km/day%s (%s)" % [
				int(d["stage"]) + 1, d["terrain"],
				d["species"], (" +" + String(d["vehicle"])) if String(d["vehicle"]) != "" else "",
				(" +" + String(d["transport"])) if String(d["transport"]) != "" else "",
				float(d["daily_km_before"]), float(d["daily_km_after"]),
				" [unblocks]" if bool(d["unblocks"]) else " (+%.0f%%)" % float(d["gain_pct"]),
				d["reason"]])
			# Every pick must be a real improvement, and never an empty one.
			if float(d["daily_km_after"]) <= float(d["daily_km_before"]):
				print("      FAIL: not an improvement")
				fails += 1
			if String(d["species"]) == "" and String(d["vehicle"]) == "" and String(d["transport"]) == "":
				print("      FAIL: an empty pick")
				fails += 1
		# The whole journey must be no worse for it. `total_days` is -1 when
		# blocked, so compare on the blocked flag first.
		var d_off: float = float((off.get("plan", {}) as Dictionary).get("total_days", -1.0))
		var d_on: float = float((on.get("plan", {}) as Dictionary).get("total_days", -1.0))
		print("  total days: %.1f -> %.1f" % [d_off, d_on])
		if picks.is_empty():
			if d_off != d_on:
				print("  FAIL: no picks, yet the plan changed")
				fails += 1
		elif d_off > 0.0 and d_on > 0.0 and d_on > d_off + 1e-6:
			print("  FAIL: re-packing made the journey SLOWER (%.3f -> %.3f days)" % [d_off, d_on])
			fails += 1

	# A second, hot world, so the per-stage picker has terrain that actually
	# rewards a different animal. The temperate world above correctly produces
	# no picks -- a mule with carts already IS the right train for paved road
	# and temperate forest -- which proves the gate but not the mechanism.
	fails += _hot_world_stage_picks()

	print("route-planner probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)

func _hot_world_stage_picks() -> int:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	wg.set_params({"tect.plates": 9, "climate.lat_n": 34.0, "climate.lat_s": 2.0, "climate.rain_k": 0.45})
	wg.generate_sized(777, 2560.0, 256, 192)
	wg.recompute_civilisation()
	var places: Array = wg.get_settlements()
	print("  [hot world] settlements: ", places.size())
	if places.size() < 2:
		print("  [hot world] FAIL: no settlements")
		return 1

	var a: Dictionary = places[0]
	var b: Dictionary = places[0]
	var best := -1.0
	for p in places:
		var d: float = abs(float(p["x"]) - float(a["x"])) + abs(float(p["y"]) - float(a["y"]))
		if d > best:
			best = d
			b = p
	wg.route_begin("mixed")
	wg.route_append_stop(float(a["x"]), float(a["y"]))
	wg.route_append_stop(float(b["x"]), float(b["y"]))
	var idx: int = wg.route_commit()
	if idx < 0:
		print("  [hot world] FAIL: route_commit refused")
		return 1

	var caravan: Dictionary = wg.jp_plan_for_route(idx)
	caravan["transport"] = "Baggage Train"
	caravan["group_size"] = 12
	caravan["cargo_kg"] = 900.0
	caravan["mule"] = 8
	caravan["horse"] = 2
	caravan["carts"] = 2
	var off: Dictionary = wg.jp_compute({"route": idx, "plan": caravan})
	var on: Dictionary = wg.jp_compute({"route": idx, "plan": caravan, "auto_stage": true})
	if not bool(on.get("ok", false)):
		print("  [hot world] FAIL: auto_stage -> ", on.get("error", "?"))
		return 1
	var picks: Array = on.get("stage_picks", [])
	var terrains: Array = []
	for s in (off.get("plan", {}) as Dictionary).get("stages", []):
		terrains.append((s as Dictionary).get("terrain", "?"))
	print("  [hot world] %d stages over %s" % [terrains.size(), ", ".join(PackedStringArray(terrains))])
	print("  [hot world] auto_stage: %d pick(s)" % picks.size())
	if picks.is_empty():
		print("    (expected on a road-connected route: the existing-way discount pulls the")
		print("     line onto Paved Road, where a mule with carts already is the best train.")
		print("     The firing case -- deep sand, camel + travois, unblocking a cart-blocked")
		print("     stage -- is pinned in cartalith-civ's own test, which can build that")
		print("     stage directly instead of hoping a generated world contains one.)")
	for p in picks:
		var d: Dictionary = p
		print("    stage %d %s / %s: %s%s%s -- %.1f -> %.1f km/day%s (%s)" % [
			int(d["stage"]) + 1, d["terrain"], d["biome"],
			d["species"], (" +" + String(d["vehicle"])) if String(d["vehicle"]) != "" else "",
			(" +" + String(d["transport"])) if String(d["transport"]) != "" else "",
			float(d["daily_km_before"]), float(d["daily_km_after"]),
			" [unblocks]" if bool(d["unblocks"]) else " (+%.0f%%)" % float(d["gain_pct"]),
			d["reason"]])
		if String(d["transport"]) != "" and String(d["transport"]) == "Walking":
			print("      FAIL: a laden train cannot 'walk' -- the availability gate leaked")
			fails += 1
	var d_off: float = float((off.get("plan", {}) as Dictionary).get("total_days", -1.0))
	var d_on: float = float((on.get("plan", {}) as Dictionary).get("total_days", -1.0))
	print("  [hot world] total days: %.1f -> %.1f" % [d_off, d_on])
	if picks.is_empty() and d_off != d_on:
		print("  [hot world] FAIL: no picks, yet the plan changed")
		fails += 1
	if not picks.is_empty() and d_off > 0.0 and d_on > 0.0 and d_on > d_off + 1e-6:
		print("  [hot world] FAIL: re-packing made the journey slower")
		fails += 1
	return fails
