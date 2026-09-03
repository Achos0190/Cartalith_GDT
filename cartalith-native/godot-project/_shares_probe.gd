extends SceneTree

## OUTSTANDING_WORK.md §2.2, over the real gdext boundary: does
## `get_settlements()` now carry `SettlementReligionState::share`, and does
## its absence still mean "nobody ran the model"?
##
## No unit test in this repository can reach this. `get_settlements` is a
## `#[func]` on a cdylib `GodotClass` and every `VarDictionary` call panics
## without a live engine, so the *decision* about which shares to emit is
## unit-tested in `lib.rs` (`civ_timeline_tests`) and the marshalling of it
## is tested only here.
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _shares_probe.gd

var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	# The same world `_faithdenom_probe.gd` measured the population-0 count on,
	# built here straight from `generate_sized` so this probe depends on no
	# shell file. `civ.villages` defaults OFF in the engine and ON in
	# `engine_bridge.generate()`'s own request dictionary -- without this line
	# the world has 15 settlements and not one of them is population 0, which
	# is a different world from the one the row was measured on.
	wg.set_villages_enabled(true)
	wg.generate_sized(77021, 2000.0, 256, 192)
	var places: Array = wg.get_settlements()
	print("settlements: ", places.size())
	if places.is_empty():
		print("FAIL: no settlements on this world")
		quit(1)
		return

	# 1. ABSENCE. Before any run there is no share vector, and the key is
	#    missing rather than present-and-empty -- an empty dictionary is what a
	#    settlement of pure `none` would look like if `none` were suppressed,
	#    and the two must not collide.
	var unrun := 0
	for p in places:
		if p.has("religion_shares"):
			unrun += 1
	_chk(unrun == 0, "no `religion_shares` key on any of %d settlements before a run" % places.size())

	# 2. A run with a real faith set. (The shipped roster is all-`none`; that
	#    path is `_belief_probe.gd`'s.)
	wg.civ_set_faction_field(1, "religion", "sun_cult")
	wg.civ_set_faction_field(2, "religion", "old_gods")
	var st: Dictionary = wg.civ_belief_run(60)
	_chk(bool(st.get("any_faith", false)), "the run reports a faith: " + str(st))
	places = wg.get_settlements()

	var missing := 0
	var zero_entry := 0
	var out_of_range := 0
	var bad_sum := 0
	var counted_but_unshared := 0
	var worst_sum := 0.0
	for p in places:
		if not p.has("religion_shares"):
			missing += 1
			continue
		var sh: Dictionary = p["religion_shares"]
		var total := 0.0
		for k in sh.keys():
			var v := float(sh[k])
			if v == 0.0:
				zero_entry += 1
			if v <= 0.0 or v > 1.0:
				out_of_range += 1
			total += v
		if absf(total - 1.0) > 1e-9:
			bad_sum += 1
		worst_sum = maxf(worst_sum, absf(total - 1.0))
		# Every religion with a head-count must have a share behind it.
		var ad: Dictionary = p.get("adherents", {})
		for k in ad.keys():
			if not sh.has(k):
				counted_but_unshared += 1
	_chk(missing == 0, "`religion_shares` present on every settlement after a run")
	_chk(zero_entry == 0, "no religion emitted with a share of exactly 0.0")
	_chk(out_of_range == 0, "every emitted share is in (0, 1]")
	_chk(bad_sum == 0, "every settlement's emitted shares sum to 1.0 (worst drift " + str(worst_sum) + ")")
	_chk(counted_but_unshared == 0, "every religion in `adherents` also appears in `religion_shares`")

	# 3. THE ROW'S OWN CASE. Population-0 settlements: `adherents` empty,
	#    shares real. How many of them hold a *mixture* is measured, not
	#    asserted -- on this world none do, and the reason is
	#    `belief_exposure`'s population weighting, not a marshalling fault
	#    (the mechanism is unit-tested in lib.rs by
	#    `a_population_zero_neighbour_holds_a_minority_only_the_shares_can_show`).
	var pop0 := 0
	var pop0_empty_adherents := 0
	var pop0_with_shares := 0
	var pop0_minority := 0
	var pop0_wholly := 0
	var pop0_wholly_faith := 0
	var mixed := 0
	var plurality_disagrees := 0
	var example := ""
	for p in places:
		var sh: Dictionary = p.get("religion_shares", {})
		if sh.size() >= 2:
			mixed += 1
		# The plurality key and the share vector must describe the same
		# settlement: `religion` is `argmax(share)`, ties to the lower roster
		# index, and a marshalling slip that paired one settlement's label
		# with another's vector shows up here and nowhere else.
		if p.has("religion"):
			var best := ""
			var best_v := -1.0
			for k in sh.keys():
				if float(sh[k]) > best_v:
					best_v = float(sh[k])
					best = String(k)
			if best != String(p["religion"]):
				plurality_disagrees += 1
		if int(p.get("population", 0)) != 0:
			continue
		pop0 += 1
		var ad: Dictionary = p.get("adherents", {})
		if ad.is_empty():
			pop0_empty_adherents += 1
		if sh.size() >= 1:
			pop0_with_shares += 1
		if sh.size() >= 2:
			pop0_minority += 1
		if sh.size() == 1 and absf(float(sh.values()[0]) - 1.0) < 1e-12:
			pop0_wholly += 1
			if String(sh.keys()[0]) != "none":
				pop0_wholly_faith += 1
				if example.is_empty():
					example = "%s -> %s" % [String(p.get("name", "?")), str(sh)]
	print("population-0 settlements: %d of %d" % [pop0, places.size()])
	print("  wholly a faith / wholly unaffiliated: %d / %d" % [pop0_wholly_faith, pop0 - pop0_wholly_faith])
	print("  with an empty `adherents`: ", pop0_empty_adherents)
	print("  with at least one share:   ", pop0_with_shares)
	print("  wholly one faith (1.0):    ", pop0_wholly)
	print("  holding a mixture:         ", pop0_minority)
	print("settlements holding a mixture, whole world: ", mixed)
	if not example.is_empty():
		print("  example: ", example)
	_chk(pop0 > 0, "this world has population-0 settlements to test with")
	_chk(pop0_empty_adherents == pop0, "every population-0 settlement's `adherents` is empty (unchanged)")
	_chk(pop0_with_shares == pop0, "every population-0 settlement carries its share vector")
	_chk(plurality_disagrees == 0, "`religion` is the argmax of `religion_shares` on every settlement")
	_chk(mixed > 0, "a real mixture crosses the boundary (%d settlements hold more than one faith)" % mixed)

	# 4. Where there ARE people the two agree: `adherents` is the rounding of
	#    share x population, never further than one person from it.
	var drift := 0
	var checked := 0
	for p in places:
		var pop := int(p.get("population", 0))
		if pop <= 0 or not p.has("religion_shares"):
			continue
		checked += 1
		var sh: Dictionary = p["religion_shares"]
		var ad: Dictionary = p.get("adherents", {})
		for k in sh.keys():
			if absf(float(sh[k]) * float(pop) - float(int(ad.get(k, 0)))) >= 1.0:
				drift += 1
	_chk(drift == 0, "on %d populated settlements every share x population is within one person of its count" % checked)

	# 5. A second `get_settlements()` call is the same answer (nothing is
	#    consumed or moved by the marshalling).
	var again: Array = wg.get_settlements()
	_chk(str(again[0].get("religion_shares", {})) == str(places[0].get("religion_shares", {})),
		"a second call marshals the same shares")

	# 6. The control state the doc comment on `get_settlements` quotes: at no
	#    horizon does a population-0 settlement on this world mix. Regenerated
	#    per horizon rather than continued, so each is a clean run.
	for years in [0, 600]:
		var w2: WorldGen = WorldGen.new()
		w2.set_villages_enabled(true)
		w2.generate_sized(77021, 2000.0, 256, 192)
		w2.civ_set_faction_field(1, "religion", "sun_cult")
		w2.civ_set_faction_field(2, "religion", "old_gods")
		w2.civ_belief_run(years)
		var m0 := 0
		var mp := 0
		for p in w2.get_settlements():
			var sh: Dictionary = p.get("religion_shares", {})
			if sh.size() < 2:
				continue
			if int(p.get("population", 0)) == 0:
				m0 += 1
			else:
				mp += 1
		print("years=%d: population-0 settlements holding a mixture = %d; populated = %d" % [years, m0, mp])

	if _fails == 0:
		print("PASS: the share vector crosses the boundary, and its absence still means `unrun`")
	else:
		print("FAIL: ", _fails, " check(s) failed")
	quit(0 if _fails == 0 else 1)
