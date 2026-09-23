extends SceneTree

## `civ_iterative_network` (`_civIterativeAutoWorld`'s centrality -> tier
## loop) through the real `compute_civilisation`, over the gdext boundary.
## Prints each world's tier histogram and asserts: settlements exist, the
## loop produced hamlets (the one-pass rank cascade cannot on a 15-place map:
## every rank < 24 is village or above), and every faction that owns a
## settlement still has its `capital` seat (the loop moves `kind` only).
## godot --headless --path godot-project -s _tierloop_probe.gd

func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	for seed in [12345, 24601, 314159]:
		wg.generate_sized(seed, 800.0, 256, 192)
		var places: Array = wg.get_settlements()
		var hist := {}
		var seats := {}
		var factions := {}
		for p in places:
			hist[p["kind"]] = int(hist.get(p["kind"], 0)) + 1
			factions[int(p["faction"])] = true
			if bool(p["capital"]):
				seats[int(p["faction"])] = true
		print("  seed ", seed, ": ", places.size(), " settlements ", hist)
		if places.is_empty():
			print("  FAIL: no settlements"); fails += 1
		if int(hist.get("hamlet", 0)) == 0:
			print("  FAIL: no hamlets -- the loop did not run"); fails += 1
		for f in factions:
			if not seats.has(f):
				print("  FAIL: faction ", f, " lost its seat"); fails += 1
	print("PASS" if fails == 0 else "FAIL (%d)" % fails)
	quit(1 if fails else 0)
