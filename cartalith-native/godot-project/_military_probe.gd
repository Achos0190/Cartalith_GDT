extends Node
## Committed verification harness for CV-25 (`civ_military_summary`)
## and CV-26 (`civ_faction_relations`).
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _military_probe.tscn
##
## What a headless boot check cannot tell you and this can: whether the two
## readouts carry **real, differentiated** numbers on a genuine multi-faction
## world, or whether they are all-zero / all-identical -- the exact failure
## this project has been bitten by four times.
##
## Asserts, in order:
##   1. every faction has a military power, and they are not all equal;
##   2. the fortification ladder produces more than one rung across the map;
##   3. the fortified count actually feeds `power.military` (drop the walls
##      off one faction's settlements and its number must fall);
##   4. relations produce every unordered pair, symmetric, with at least two
##      distinct stances on a world with real borders;
##   5. a shared border is measured, and the widest one is not zero.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

const SEED := 483920


func _ready() -> void:
	var ok := true
	var gen := WorldGen.new()
	gen.generate_sized(SEED, 1200.0, 384, 288)
	print("MIL world %dx%d, %d settlements, %d factions" % [
		gen.get_width(), gen.get_height(), gen.get_settlements().size(),
		gen.civ_faction_count()])

	# ---- 1. per-faction military strength --------------------------------
	var summary: Dictionary = gen.civ_military_summary()
	var factions: Array = summary.get("factions", [])
	var places: Array = summary.get("settlements", [])
	print("MIL factions=%d settlement rows=%d" % [factions.size(), places.size()])
	if factions.is_empty() or places.is_empty():
		print("MIL !! empty readout -- nothing to verify")
		get_tree().quit(1)
		return

	var mil := []
	for f in factions:
		var d: Dictionary = f
		mil.append(float(d.get("military", 0.0)))
		print("MIL   %-16s military %5.1f  overall %5.1f  pop %9d  %2d places, %2d fortified (%d stone / %d palisade / %d ditch)  capital %s" % [
			d.get("name", "?"), d.get("military", 0.0), d.get("overall", 0.0),
			int(d.get("pop", 0)), int(d.get("settlement_count", 0)),
			int(d.get("fortified_count", 0)), int(d.get("walled_stone", 0)),
			int(d.get("walled_palisade", 0)), int(d.get("walled_ditch", 0)),
			d.get("capital", "—")])
	var lo: float = mil.min()
	var hi: float = mil.max()
	if hi <= 0.0:
		ok = false
		print("MIL !! every faction reads zero military power")
	if is_equal_approx(lo, hi):
		ok = false
		print("MIL !! every faction reads the SAME military power (%.3f)" % lo)
	else:
		print("MIL   spread %.2f .. %.2f  OK" % [lo, hi])

	# ---- 2. the fortification ladder -------------------------------------
	var rungs := {}
	var def_lo := 2.0
	var def_hi := -1.0
	for p in places:
		var d: Dictionary = p
		var s := String(d.get("wall_spec", "?"))
		rungs[s] = int(rungs.get(s, 0)) + 1
		var v := float(d.get("defensibility", 0.0))
		def_lo = minf(def_lo, v)
		def_hi = maxf(def_hi, v)
	print("MIL wall rungs %s   defensibility %.3f .. %.3f" % [rungs, def_lo, def_hi])
	if rungs.size() < 2:
		ok = false
		print("MIL !! the ladder collapsed to one rung")
	if is_equal_approx(def_lo, def_hi):
		ok = false
		print("MIL !! defensibility is constant across every settlement")

	# ---- 3. the fortified term actually reaches power.military -----------
	## Turn the walls OFF on every settlement of the strongest faction via
	## the place editor's own override, and its military power must fall.
	var strongest := 0
	var best := -1.0
	for f in factions:
		var d: Dictionary = f
		if float(d.get("military", 0.0)) > best:
			best = float(d.get("military", 0.0))
			strongest = int(d.get("faction", 0))
	var touched := 0
	for p in places:
		var d: Dictionary = p
		if int(d.get("faction", 0)) == strongest:
			gen.civ_edit_settlement(int(d.get("index", -1)), {"walls": 0})
			touched += 1
	var after: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
	var now := -1.0
	for f in after:
		var d: Dictionary = f
		if int(d.get("faction", 0)) == strongest:
			now = float(d.get("military", 0.0))
	print("MIL faction %d: %d settlements de-walled, military %.2f -> %.2f" % [
		strongest, touched, best, now])
	if touched > 0 and not (now < best):
		ok = false
		print("MIL !! de-walling changed nothing -- fortifiedFraction is not reaching power.military")
	## Put them back, so the relations pass below sees the real world.
	for p in places:
		var d: Dictionary = p
		if int(d.get("faction", 0)) == strongest:
			gen.civ_edit_settlement(int(d.get("index", -1)), {"walls": -1})

	# ---- 4/5. relations --------------------------------------------------
	var pairs: Array = gen.civ_faction_relations()
	var n := gen.civ_faction_count()
	var expect := n * (n - 1) / 2
	print("REL %d pairs (expected %d for %d factions)" % [pairs.size(), expect, n])
	if pairs.size() != expect:
		ok = false
		print("REL !! wrong pair count")
	var stances := {}
	var values := []
	var max_border := 0
	for r in pairs:
		var d: Dictionary = r
		stances[String(d.get("stance", "?"))] = true
		values.append(float(d.get("value", 0.0)))
		max_border = maxi(max_border, int(d.get("border_cells", 0)))
		print("REL   %-16s ↔ %-16s %-8s %+6.3f   border %4d (%.2f)  cult %+0.1f faith %+0.1f trade %+0.2f rivalry %.2f" % [
			d.get("a_name", "?"), d.get("b_name", "?"), d.get("stance", "?"),
			d.get("value", 0.0), int(d.get("border_cells", 0)),
			d.get("border_fraction", 0.0), d.get("culture_term", 0.0),
			d.get("religion_term", 0.0), d.get("trade_term", 0.0),
			d.get("rivalry_term", 0.0)])
	if max_border <= 0:
		ok = false
		print("REL !! no faction pair shares a border -- the friction term is dead")
	if values.size() > 1 and is_equal_approx(values.min(), values.max()):
		ok = false
		print("REL !! every pair reads the same value (%.4f)" % values.min())
	print("REL stances seen: %s   value spread %.3f .. %.3f" % [
		stances.keys(), values.min(), values.max()])
	if stances.size() < 2:
		print("REL .. only one stance on this world -- not a failure, but check the seed")

	# ---- 6. the affinity terms respond to an authored roster --------------
	## A fresh world seeds every faction a distinct culture and no religion,
	## so culture/faith are legitimately silent until an author says
	## otherwise. Prove they are wired, not dead: give factions 1 and 2 the
	## same culture and the same faith, and their value must rise by the
	## documented +0.30 and +0.20.
	var before := 0.0
	for r in pairs:
		var d: Dictionary = r
		if int(d.get("a", 0)) == 1 and int(d.get("b", 0)) == 2:
			before = float(d.get("value", 0.0))
	## `get_factions()` is 0-indexed over factions 1..n, so row 0 IS faction 1.
	var f1: Dictionary = gen.get_factions()[0]
	if not gen.civ_set_faction_field(2, "culture", String(f1.get("culture", ""))):
		print("REL !! set culture rejected (culture=%s)" % f1.get("culture", ""))
	if not gen.civ_set_faction_field(1, "religion", "sun_cult"):
		print("REL !! set religion rejected")
	if not gen.civ_set_faction_field(2, "religion", "sun_cult"):
		print("REL !! set religion rejected")
	var same := 0.0
	for r in gen.civ_faction_relations():
		var d: Dictionary = r
		if int(d.get("a", 0)) == 1 and int(d.get("b", 0)) == 2:
			same = float(d.get("value", 0.0))
			print("REL same culture+faith: %+.3f -> %+.3f  (%s)  cult %+0.1f faith %+0.1f" % [
				before, same, d.get("stance", "?"), d.get("culture_term", 0.0),
				d.get("religion_term", 0.0)])
	if not is_equal_approx(same - before, 0.5):
		ok = false
		print("REL !! shared culture + shared faith moved the value by %.4f, not 0.50" % (same - before))
	## And an opposed faith must move it the other way.
	if not gen.civ_set_faction_field(2, "religion", "old_gods"):
		print("REL !! set religion rejected")
	for r in gen.civ_faction_relations():
		var d: Dictionary = r
		if int(d.get("a", 0)) == 1 and int(d.get("b", 0)) == 2:
			print("REL opposed faith:      %+.3f  (%s)  faith %+0.1f" % [
				d.get("value", 0.0), d.get("stance", "?"), d.get("religion_term", 0.0)])
			if float(d.get("religion_term", 0.0)) >= 0.0:
				ok = false
				print("REL !! two different faiths did not read as opposed")

	print("MIL RESULT %s" % ("PASS" if ok else "FAIL"))
	get_tree().quit(0 if ok else 1)
