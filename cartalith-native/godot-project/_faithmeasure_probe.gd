extends SceneTree

## Lane B measurement probe (not a deliverable). Establishes the CONTROL STATE
## the religion surfaces' design arguments rest on, rather than reasoning about
## it:
##
##   a) how many settlements are Unclaimed (faction 0) -- the set
##      `map_overlay._faith_diverged` and `_religion_divergence` both exclude;
##   b) how many of those Unclaimed settlements hold a faith after a run --
##      i.e. religion crossing into ungoverned land, which nothing surfaces;
##   c) how many ruled settlements diverge from their ruler's faith;
##   d) whether a per-faction plurality ever disagrees with the hand-set
##      state religion (RELIGION_DIFFUSION_SCOPE.md §4's fork, measured).

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(24601, 1400.0, 384, 256)
	var n_f := int(wg.civ_faction_count())
	print("factions=", n_f)
	# Give three factions three different faiths, leave the rest at none.
	wg.civ_set_faction_field(1, "religion", "sun_cult")
	wg.civ_set_faction_field(2, "religion", "old_gods")
	if n_f >= 3:
		wg.civ_set_faction_field(3, "religion", "sea_lords")
	for years in [0, 25, 50, 100, 300]:
		var wg2: WorldGen = WorldGen.new()
		wg2.generate_sized(24601, 1400.0, 384, 256)
		wg2.civ_set_faction_field(1, "religion", "sun_cult")
		wg2.civ_set_faction_field(2, "religion", "old_gods")
		if n_f >= 3:
			wg2.civ_set_faction_field(3, "religion", "sea_lords")
		var st: Dictionary = wg2.civ_belief_run(years)
		var places: Array = wg2.get_settlements()
		var rel := PackedStringArray()
		for r in wg2.get_factions():
			var d: Dictionary = r
			var id := int(d.get("id", 0))
			while rel.size() < id:
				rel.append("")
			rel[id - 1] = String(d.get("religion", ""))
		var unclaimed := 0
		var unclaimed_faith := 0
		var diverged := 0
		var mixed := 0
		var per_faction := {}
		for p in places:
			var s: Dictionary = p
			var f := int(s.get("faction", 0))
			var ad: Dictionary = s.get("adherents", {})
			if ad.size() >= 2:
				mixed += 1
			if f <= 0:
				unclaimed += 1
				if s.has("religion") and String(s["religion"]) != "none":
					unclaimed_faith += 1
				continue
			if s.has("religion") and f <= rel.size() and not rel[f - 1].is_empty():
				if String(s["religion"]) != rel[f - 1]:
					diverged += 1
			if not per_faction.has(f):
				per_faction[f] = {}
			for k in ad.keys():
				per_faction[f][k] = int(per_faction[f].get(k, 0)) + int(ad[k])
		var forks := 0
		for f in per_faction.keys():
			var best := ""
			var bn := -1
			for k in per_faction[f].keys():
				if int(per_faction[f][k]) > bn:
					bn = int(per_faction[f][k])
					best = String(k)
			if f <= rel.size() and not rel[f - 1].is_empty() and best != rel[f - 1]:
				forks += 1
		print("years=", years, " settlements=", places.size(),
			" unclaimed=", unclaimed, " unclaimed_with_faith=", unclaimed_faith,
			" diverged=", diverged, " mixed(>=2 faiths)=", mixed,
			" factions_whose_people_disagree_with_state_religion=", forks,
			" any_faith=", st.get("any_faith", null))
	quit(0)
