extends SceneTree

## RELIGION_DIFFUSION_SCOPE.md milestone 1, over the real gdext boundary.
##
## What no unit test in this repository can reach: `civ_belief_run` and
## `get_settlements` are `#[func]`s on a cdylib `GodotClass`, so the nested
## `adherents` Dictionary, the omitted-key contract and the status dictionary
## have never been marshalled until this runs.

func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	if not wg.has_method("civ_belief_run"):
		print("FAIL: civ_belief_run absent from the extension")
		quit(1)
		return

	wg.generate_sized(24601, 640.0, 96, 64)
	var places: Array = wg.get_settlements()
	print("  settlements: ", places.size())
	if places.is_empty():
		print("  FAIL: no settlements to diffuse over")
		quit(1)
		return

	# 1. Before any run, both keys must be ABSENT -- not "none" and not {}.
	for p in places:
		if p.has("religion") or p.has("adherents"):
			print("  FAIL: an unrun world already carries a religion key: ", p)
			fails += 1
			break
	print("  before any run: religion key present = ", places[0].has("religion"))

	# 2. The shipped roster is all-`none`, so a run reports no faith AND a
	#    reason. This is the finding the engine half documents.
	var r: Dictionary = wg.civ_belief_run(50)
	print("  run on the shipped roster -> ", r)
	if bool(r.get("any_faith", true)):
		print("  FAIL: a world whose factions are all `none` reported a faith")
		fails += 1
	if not r.has("reason"):
		print("  FAIL: any_faith false with no reason to show the user")
		fails += 1
	if not bool(r.get("seeded", false)):
		print("  FAIL: the first run must seed")
		fails += 1

	# The keys ARE present now -- the layer exists, it is just all-secular.
	places = wg.get_settlements()
	if not places[0].has("religion"):
		print("  FAIL: after a run the religion key must be present")
		fails += 1
	elif String(places[0]["religion"]) != "none":
		print("  FAIL: expected `none` plurality, got ", places[0]["religion"])
		fails += 1

	# 3. Hand-set a faction religion -- the only thing that makes the layer do
	#    anything -- and re-run.
	if not wg.civ_set_faction_field(1, "religion", "sun_cult"):
		print("  FAIL: could not set faction 1's religion")
		quit(1)
		return
	var r2: Dictionary = wg.civ_belief_run(60)
	print("  after setting faction 1 to sun_cult -> ", r2)
	if not bool(r2.get("any_faith", false)):
		print("  FAIL: a hand-set religion did not reach the model")
		fails += 1
	if r2.has("reason"):
		print("  FAIL: `reason` must be omitted once there is a faith to show")
		fails += 1

	# 4. The marshalled shape: adherents is a Dictionary summing to population,
	#    with no zero entries.
	places = wg.get_settlements()
	var faiths := {}
	var checked := 0
	for p in places:
		if not p.has("adherents"):
			print("  FAIL: adherents missing on ", p.get("name", "?"))
			fails += 1
			break
		var a: Dictionary = p["adherents"]
		var total := 0
		for k in a.keys():
			var n := int(a[k])
			if n <= 0:
				print("  FAIL: zero-adherent entry emitted for ", k)
				fails += 1
			total += n
			faiths[k] = true
		if total != int(p["population"]):
			print("  FAIL: adherents sum ", total, " != population ", int(p["population"]),
				" at ", p.get("name", "?"))
			fails += 1
			break
		checked += 1
	print("  adherent totals verified on ", checked, " settlements; faiths seen: ", faiths.keys())
	if not faiths.has("sun_cult"):
		print("  FAIL: the sun cult never appeared in any settlement")
		fails += 1

	# 5. It actually spread beyond the faction that holds it.
	var outside := 0
	for p in places:
		if int(p["faction"]) != 1 and p.has("adherents") and int(p["adherents"].get("sun_cult", 0)) > 0:
			outside += 1
	print("  settlements outside faction 1 holding sun_cult adherents: ", outside)
	if outside == 0:
		print("  NOTE: no cross-faction spread on this world -- not a failure by itself")

	if fails == 0:
		print("PASS: belief layer marshals, omits its keys until run, and conserves population")
	else:
		print("FAIL: ", fails, " check(s) failed")
	quit(0 if fails == 0 else 1)
