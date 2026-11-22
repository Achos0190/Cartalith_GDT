extends Node
## F1 follow-up: the owner's world shape (2048 x 1024 over 200 km) through the engine's measure, route and planner.
func _ready() -> void:
	var g := WorldGen.new()
	g.generate_sized(83744, 200.0, 2048, 1024)
	var rng := RandomNumberGenerator.new()
	rng.seed = 3
	for t in 8:
		var pts: Array = []
		for q in [2, 3, 5, 8, 12, 20, 30, 45][t]:
			pts.append(Vector2(rng.randf_range(0.05, 0.95) * 2048, rng.randf_range(0.05, 0.95) * 1024))
		g.measure_begin()
		for p in pts:
			g.measure_add_point(p.x, p.y)
		var m: Dictionary = g.measure_result()
		g.route_begin("mixed")
		for p in pts:
			g.route_append_stop(p.x, p.y)
		var idx: int = g.route_commit()
		var r: Dictionary = g.route_get(idx)
		var res: Dictionary = g.jp_compute({"route": idx, "plan": {}})
		print("FENG1B pts=%d measure=%.1f km route=%.1f km planner=%.1f km unreachable=%d" % [pts.size(), m.get("total_km", -1), r.get("km", -1), res.get("plan", {}).get("km", -1.0), r.get("unreachable_legs", -1)])
	get_tree().quit(0)
