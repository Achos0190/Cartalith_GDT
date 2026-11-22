extends Node
## F33 follow-up (headless): is the switch on in a WorldGen made the way the shell makes one, and what does the planner
## call a stage, switch on versus off, on the same routes?
func _ready() -> void:
	var g := WorldGen.new()
	g.generate_sized(77021, 800.0, 512, 300)
	print("FENG1C switch param_get=", g.get_params().get("civ.planner_dominant_biome"))
	var rng := RandomNumberGenerator.new()
	rng.seed = 5
	var diff := 0
	var total := 0
	for trial in 40:
		g.set_params({"civ.planner_dominant_biome": true})
		var pts: Array = []
		for q in 2:
			pts.append(Vector2(rng.randf_range(0.05, 0.95) * 512, rng.randf_range(0.05, 0.95) * 300))
		g.route_begin("mixed")
		for p in pts:
			g.route_append_stop(p.x, p.y)
		var idx: int = g.route_commit()
		var on: Array = g.jp_compute({"route": idx, "plan": {}}).get("plan", {}).get("stages", [])
		g.set_params({"civ.planner_dominant_biome": false})
		var off: Array = g.jp_compute({"route": idx, "plan": {}}).get("plan", {}).get("stages", [])
		for i in mini(on.size(), off.size()):
			total += 1
			if String(on[i]["biome"]) != String(off[i]["biome"]):
				diff += 1
				print("FENG1C stage %d km %.1f: reference label '%s' -> '%s'" % [i, on[i]["km"], off[i]["biome"], on[i]["biome"]])
	print("FENG1C stages %d, renamed by the switch %d" % [total, diff])
	get_tree().quit(0)
