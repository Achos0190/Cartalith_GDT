extends SceneTree
## Lane URBANDRAW, step 1: what does `urban_layouts()` ACTUALLY emit, across
## several settlements -- walled and unwalled, and every culture scheme this
## port can actually reach?
##
## Pure data -- no drawing, so headless is correct (MISTAKES.md: windowed is
## for pixels; this reads a Dictionary).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _urbandraw_keys_probe.gd

const SEED := 24601
const SIZE_KM := 96.0
const GW := 96
const GH := 64

## Substring probes for the clutter this row is asking about -- run against
## the UNION of keys across every dumped layout, so one town's absence cannot
## hide a key a different town carries.
const WATCH_SUBSTRINGS := [
	"tree", "garden", "orchard", "well", "tower", "clutter", "detail",
	"fence", "crane", "bollard", "cross", "spoil", "rack", "boom",
]

func _init() -> void:
	var gen := WorldGen.new()
	gen.generate_sized(SEED, SIZE_KM, GW, GH)
	var settlements: Array = gen.get_settlements()
	print("URBANDRAWKEYS %d settlements, seed=%d" % [settlements.size(), SEED])

	# Force at least one settlement well past FORT_MIN/typical wall thresholds
	# (same bridge call the Place Editor's population field uses) and one held
	# at a small population, so this run does not depend on the seed already
	# happening to contain both a walled and an unwalled town.
	var populated: Array = []
	for i in settlements.size():
		var s: Dictionary = settlements[i]
		if int(s.get("population", 0)) > 0:
			populated.append(i)
	if populated.size() < 2:
		print("  FAIL  fewer than 2 populated settlements generated -- cannot compare walled/unwalled")
		quit(2)
		return
	var big_idx: int = populated[0]
	var small_idx: int = populated[1]
	print("URBANDRAWKEYS forcing #%d to pop=9000 (large/walled candidate), #%d to kind=hamlet pop=80 (unwalled candidate: rank 0 in um_wall_spec's ladder never returns anything but \"none\" absent the fortified trait)" % [
		big_idx, small_idx])
	gen.civ_edit_settlement(big_idx, {"population": 9000})
	gen.civ_edit_settlement(small_idx, {"population": 80, "kind": "hamlet"})

	var sample: Array = [big_idx, small_idx]
	for i in populated.size():
		if sample.size() >= 5:
			break
		if populated[i] != big_idx and populated[i] != small_idx:
			sample.append(populated[i])

	var idx_arr := PackedInt32Array()
	for i in sample:
		idx_arr.append(i)
	var layouts: Array = gen.urban_layouts(idx_arr)
	print("URBANDRAWKEYS requested %d layouts, got %d back" % [idx_arr.size(), layouts.size()])
	# Re-read settlements AFTER the edits above -- the array captured earlier
	# is a snapshot from before `civ_edit_settlement` mutated the live roster,
	# so printing population off it would show the pre-edit numbers.
	settlements = gen.get_settlements()

	var all_keys := {}
	for layout in layouts:
		var l: Dictionary = layout
		var idx: int = int(l.get("index", -1))
		var s: Dictionary = settlements[idx] if idx >= 0 and idx < settlements.size() else {}
		var ring: PackedVector2Array = l.get("wall_ring", PackedVector2Array())
		var walled: bool = l.has("wall_ring") and ring.size() >= 3
		var districts: PackedStringArray = l.get("building_district", PackedStringArray())
		var dist_counts := {}
		for d in districts:
			dist_counts[d] = int(dist_counts.get(d, 0)) + 1
		print("---- settlement #%d \"%s\" roster_pop=%d layout_pop=%d pop_target=%d ----" % [
			idx, String(s.get("name", "?")), int(s.get("population", 0)),
			int(l.get("pop", 0)), int(l.get("pop_target", 0))])
		print("  walled=%s wall_style=%s wall_spec=%s wall_ring_verts=%d" % [
			walled, String(l.get("wall_style", "<absent>")), String(l.get("wall_spec", "?")), ring.size()])
		print("  buildings=%d building_district counts=%s" % [
			(l.get("buildings", []) as Array).size(), dist_counts])
		print("  farmland=%d farmland_pasture=%d" % [
			(l.get("farmland", []) as Array).size(), (l.get("farmland_pasture", PackedByteArray())).size()])
		print("  ALL %d KEYS = %s" % [l.keys().size(), l.keys()])
		for k in l.keys():
			all_keys[String(k)] = true

	var all_keys_sorted: Array = all_keys.keys()
	all_keys_sorted.sort()
	print("URBANDRAWKEYS UNION of every key seen across %d dumped layouts (%d keys):" % [
		layouts.size(), all_keys_sorted.size()])
	print("  %s" % [all_keys_sorted])

	print("URBANDRAWKEYS substring watch -- any dumped key matching a clutter/tower term:")
	var found_any := false
	for term in WATCH_SUBSTRINGS:
		var hits: Array = []
		for k in all_keys_sorted:
			# Word-boundary-ish: reject a hit that is only a substring of an
			# unrelated key ("tree" inside "street"/"street_len_m") by requiring
			# the term to border a '_' or a string edge, not just appear.
			var ks := String(k).to_lower()
			var at := ks.find(term)
			if at < 0:
				continue
			var before_ok: bool = at == 0 or ks[at - 1] == "_"
			var after_ok: bool = (at + term.length() == ks.length()) or ks[at + term.length()] == "_"
			if before_ok and after_ok:
				hits.append(k)
		if hits.is_empty():
			print("  %-10s -- ABSENT (no key contains this substring)" % term)
		else:
			found_any = true
			print("  %-10s -- PRESENT: %s" % [term, hits])

	print("URBANDRAWKEYS verdict: %s" % (
		"at least one clutter/tower-shaped key crosses the bridge" if found_any
		else "NO trees/gardens/orchards/wells/towers key crosses the bridge -- only field/pasture farmland and the wall ring/gates/spurs/centroid vocabulary do"))

	quit(0)
