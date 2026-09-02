extends Node
## Committed verification harness for `MILITARY_MANPOWER_SCOPE.md`
## -- the four manpower outputs nested under `civ_military_summary()`.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _manpower_probe.tscn
##
## What a unit test cannot tell you and this can: whether the model produces
## **real, differentiated** numbers on a genuine multi-faction world, or
## whether every faction reads the same or reads zero -- the exact failure
## this project has been bitten by repeatedly.
##
## Asserts, in order:
##   1. every faction has a manpower row, and the four outputs are neither
##      all-zero nor all-identical;
##   2. the model's own ordering holds everywhere: standing < field < levy,
##      and the levy is a small share of the population;
##   3. the force/duration ladder decreases, is pool-capped at 30 days and
##      fiscally capped at 365;
##   4. ag-tech is genuinely live -- `AG_TECH_LEVELS.farmers_per_urbanite`
##      had NO consumer in this port before this pass, so a change must move
##      the numbers;
##   5. government is genuinely live -- likewise, and its own module doc says
##      nothing in either codebase read it;
##   6. geography is genuinely live -- two factions on identical institutions
##      must still differ, or the model is a technology lookup after all;
##   7. the era is derived and its band reported rather than enforced;
##   8. the citizen / free population (owner ruling, 2026-08-25) is the band
##      denominator, is differentiated by government, and moves NO headcount.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

const SEED := 483920

## One government per faction, so the citizen fraction has something to be
## differentiated BY -- a default roster seeds every faction `monarchy`, which
## would leave the denominator identical everywhere and prove nothing.
const GOVS := ["monarchy", "empire", "republic", "chiefdom", "oligarchy", "theocracy",
	"city_state", "tribal_confederacy"]


func _fmt_row(d: Dictionary) -> String:
	var m: Dictionary = d.get("manpower", {})
	return "%-16s standing %7d (pro %6d) field %7d levy %8d  pop %9d  cit %9d (%.0f%%)  %-30s %s/%s" % [
		d.get("name", "?"), int(round(float(m.get("standing_army", 0.0)))),
		int(round(float(m.get("professional_core", 0.0)))),
		int(round(float(m.get("field_army", 0.0)))),
		int(round(float(m.get("emergency_mobilization", 0.0)))),
		int(round(float(m.get("total_population", 0.0)))),
		int(round(float(m.get("citizen_population", 0.0)))),
		100.0 * float(m.get("citizen_fraction", 0.0)),
		m.get("era", "?"), m.get("era_standing_verdict", "?"),
		m.get("era_mobilization_verdict", "?")]


func _standing_of(rows: Array, fid: int) -> float:
	for r in rows:
		if int((r as Dictionary).get("faction", -1)) == fid:
			return float(((r as Dictionary).get("manpower", {}) as Dictionary).get("standing_army", 0.0))
	return -1.0


func _ready() -> void:
	var ok := true
	var gen := WorldGen.new()
	gen.generate_sized(SEED, 1200.0, 384, 288)
	print("MP world %dx%d, %d settlements, %d factions" % [
		gen.get_width(), gen.get_height(), gen.get_settlements().size(),
		gen.civ_faction_count()])

	var factions: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
	if factions.is_empty():
		print("MP !! empty readout")
		get_tree().quit(1)
		return

	# ---- 1. real, differentiated numbers ---------------------------------
	var standing := []
	var levies := []
	var shares := []
	for f in factions:
		var d: Dictionary = f
		var m: Dictionary = d.get("manpower", {})
		if m.is_empty():
			ok = false
			print("MP !! faction %s carries no manpower row" % d.get("name", "?"))
			continue
		print("MP   " + _fmt_row(d))
		print("MP        drivers: farming %.1f%% surplus/farmer %.3f extraction %.2f%% professional %.2f logistics %.3f  (state %.3f eco %.3f)  ag=%s gov=%s" % [
			100.0 * float(m.get("agricultural_labour_ratio", 0.0)),
			float(m.get("food_surplus_per_farmer", 0.0)),
			100.0 * float(m.get("fiscal_extraction_efficiency", 0.0)),
			float(m.get("professionalization", 0.0)),
			float(m.get("logistics_capacity", 0.0)),
			float(m.get("state_capacity", 0.0)), float(m.get("ecological_factor", 0.0)),
			m.get("ag_tech", "?"), m.get("government", "?")])
		print("MP        logistics from: road %.4f  navigable %.3f  sea %.3f" % [
			float(m.get("road_density", 0.0)), float(m.get("navigable_share", 0.0)),
			float(m.get("sea_share", 0.0))])
		var parts := PackedStringArray()
		for e in (m.get("force_ladder", []) as Array):
			var l: Dictionary = e
			parts.append("%dd %d%s" % [int(l.get("days", 0)),
				int(round(float(l.get("force", 0.0)))),
				"*" if bool(l.get("capped_by_pool", false)) else ""])
		print("MP        ladder: %s   durations field %dd levy %dd   concentration %.3f" % [
			" · ".join(parts), int(round(float(m.get("field_duration_days", 0.0)))),
			int(round(float(m.get("emergency_duration_days", 0.0)))),
			float(m.get("concentration_ratio", 0.0))])
		standing.append(float(m.get("standing_army", 0.0)))
		levies.append(float(m.get("emergency_mobilization", 0.0)))
		shares.append(float(m.get("emergency_share", 0.0)))

		# ---- 2. the model's own ordering ---------------------------------
		var s := float(m.get("standing_army", 0.0))
		var fa := float(m.get("field_army", 0.0))
		var lv := float(m.get("emergency_mobilization", 0.0))
		if not (s < fa and fa < lv):
			ok = false
			print("MP !! %s breaks standing<field<levy: %.0f / %.0f / %.0f" % [
				d.get("name", "?"), s, fa, lv])
		if float(m.get("emergency_share", 0.0)) > 0.30:
			ok = false
			print("MP !! %s can mobilise %.1f%% of itself -- above every era band" % [
				d.get("name", "?"), 100.0 * float(m.get("emergency_share", 0.0))])

		# ---- 3. the ladder -----------------------------------------------
		var ladder: Array = m.get("force_ladder", [])
		if ladder.size() != 4:
			ok = false
			print("MP !! ladder has %d rungs, not 4" % ladder.size())
		else:
			var prev := INF
			for e in ladder:
				var v := float((e as Dictionary).get("force", 0.0))
				if v > prev + 1e-6:
					ok = false
					print("MP !! ladder rose at %dd" % int((e as Dictionary).get("days", 0)))
				prev = v
			if not bool((ladder[0] as Dictionary).get("capped_by_pool", false)):
				ok = false
				print("MP !! %s: the 30-day rung is not pool-capped" % d.get("name", "?"))
			if bool((ladder[3] as Dictionary).get("capped_by_pool", false)):
				ok = false
				print("MP !! %s: the 365-day rung is pool-capped, so the fiscal curve never binds" % d.get("name", "?"))

	if standing.is_empty() or standing.max() <= 0.0:
		ok = false
		print("MP !! every faction reads a zero standing army")
	elif is_equal_approx(standing.min(), standing.max()):
		ok = false
		print("MP !! every faction reads the SAME standing army (%.1f)" % standing.min())
	else:
		print("MP standing spread %.0f .. %.0f   levy spread %.0f .. %.0f   levy share %.2f%% .. %.2f%%  OK" % [
			standing.min(), standing.max(), levies.min(), levies.max(),
			100.0 * shares.min(), 100.0 * shares.max()])

	# ---- 4. ag-tech is live ----------------------------------------------
	## `AG_TECH_LEVELS.farmers_per_urbanite` had no consumer anywhere in this
	## port. If this assertion fails, it still does not.
	var base_1 := _standing_of(factions, 1)
	if not gen.civ_set_faction_field(1, "ag_tech", "improvedAgrarian"):
		print("MP !! set ag_tech rejected")
	var after_ag := _standing_of((gen.civ_military_summary() as Dictionary).get("factions", []), 1)
	print("MP ag-tech traditionalAgrarian -> improvedAgrarian: standing %.0f -> %.0f" % [base_1, after_ag])
	if is_equal_approx(base_1, after_ag):
		ok = false
		print("MP !! ag-tech reaches nothing -- farmersPerUrbanite is still inert")
	gen.civ_set_faction_field(1, "ag_tech", "traditionalAgrarian")

	# ---- 5. government is live -------------------------------------------
	if not gen.civ_set_faction_field(1, "government", "chiefdom"):
		print("MP !! set government rejected")
	var weak := _standing_of((gen.civ_military_summary() as Dictionary).get("factions", []), 1)
	if not gen.civ_set_faction_field(1, "government", "empire"):
		print("MP !! set government rejected")
	var strong := _standing_of((gen.civ_military_summary() as Dictionary).get("factions", []), 1)
	print("MP government chiefdom -> empire: standing %.0f -> %.0f" % [weak, strong])
	if not (strong > weak * 1.5):
		ok = false
		print("MP !! government reaches nothing (or barely) -- CIV_GOVERNMENTS is still inert")
	gen.civ_set_faction_field(1, "government", "monarchy")

	# ---- 6. geography is live --------------------------------------------
	## Put every faction on identical institutions. If the answers collapse
	## to one number, the model is a technology lookup wearing five variables.
	for i in range(1, gen.civ_faction_count() + 1):
		gen.civ_set_faction_field(i, "government", "monarchy")
		gen.civ_set_faction_field(i, "ag_tech", "traditionalAgrarian")
	var levelled: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
	var lvl := []
	var logistics := []
	var eco := []
	for f in levelled:
		var m: Dictionary = (f as Dictionary).get("manpower", {})
		lvl.append(float(m.get("standing_army", 0.0)))
		logistics.append(float(m.get("logistics_capacity", 0.0)))
		eco.append(float(m.get("ecological_factor", 0.0)))
	print("MP identical institutions: standing %.0f .. %.0f   logistics %.3f .. %.3f   ecological %.3f .. %.3f" % [
		lvl.min(), lvl.max(), logistics.min(), logistics.max(), eco.min(), eco.max()])
	if is_equal_approx(lvl.min(), lvl.max()):
		ok = false
		print("MP !! identical institutions collapsed every faction to one answer -- geography is dead")
	if is_equal_approx(logistics.min(), logistics.max()):
		ok = false
		print("MP !! every faction has identical logistics -- the way network is not being read")

	# ---- 7. the era is derived, and its band reported ---------------------
	var eras := {}
	for f in levelled:
		var m: Dictionary = (f as Dictionary).get("manpower", {})
		eras[String(m.get("era", "?"))] = true
		var v := String(m.get("era_standing_verdict", ""))
		if not (v in ["within", "above", "below"]):
			ok = false
			print("MP !! bad era verdict '%s'" % v)
	print("MP eras on identical institutions: %s" % [eras.keys()])

	# ---- 8. the citizen / free population is the band denominator ---------
	## Owner ruling, 2026-08-25: the era table's percentages are shares of the
	## citizen/free body, not of the total, as the specification's own
	## Republican Rome citation ("17-29 % of its CITIZEN population") states.
	##
	## Two things have to be true and neither is checkable from a unit test on
	## a synthetic input: the denominator must be genuinely differentiated on a
	## real world (a default roster is all-`monarchy`, so this assigns one
	## government per faction), and it must move NO headcount -- the four
	## outputs are calibrated on the worked examples and were validated there.
	var before := {}
	for f in levelled:
		var d: Dictionary = f
		var m: Dictionary = d.get("manpower", {})
		before[int(d.get("faction", -1))] = [
			float(m.get("total_population", 0.0)), float(m.get("emergency_mobilization", 0.0))]
	for i in range(1, gen.civ_faction_count() + 1):
		gen.civ_set_faction_field(i, "government", GOVS[(i - 1) % GOVS.size()])
	var mixed: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
	var fracs := []
	for f in mixed:
		var d: Dictionary = f
		var m: Dictionary = d.get("manpower", {})
		fracs.append(float(m.get("citizen_fraction", 0.0)))
		print("MP   " + _fmt_row(d))
		print("MP        gov=%-20s citizens %d of %d (%.1f%%)  standing %.3f%% of citizens [%s, band %.1f-%.1f%%]  mobilization %.2f%% [%s, band %.0f-%.0f%%]   (of TOTAL: %.3f%% / %.2f%%)" % [
			m.get("government", "?"),
			int(round(float(m.get("citizen_population", 0.0)))),
			int(round(float(m.get("total_population", 0.0)))),
			100.0 * float(m.get("citizen_fraction", 0.0)),
			100.0 * float(m.get("standing_citizen_share", 0.0)),
			m.get("era_standing_verdict", "?"),
			100.0 * float(m.get("era_standing_lo", 0.0)),
			100.0 * float(m.get("era_standing_hi", 0.0)),
			100.0 * float(m.get("emergency_citizen_share", 0.0)),
			m.get("era_mobilization_verdict", "?"),
			100.0 * float(m.get("era_mobilization_lo", 0.0)),
			100.0 * float(m.get("era_mobilization_hi", 0.0)),
			100.0 * float(m.get("standing_share", 0.0)),
			100.0 * float(m.get("emergency_share", 0.0))])
		# The citizen body is a real subset, never the whole and never zero.
		var cf := float(m.get("citizen_fraction", 0.0))
		var cp := float(m.get("citizen_population", 0.0))
		var tp := float(m.get("total_population", 0.0))
		if cf <= 0.0 or cf > 0.98001 or cp <= 0.0 or cp >= tp:
			ok = false
			print("MP !! %s: citizen population %.0f of %.0f (%.3f) is not a real subset" % [
				d.get("name", "?"), cp, tp, cf])
		if abs(cp - tp * cf) > 1.0:
			ok = false
			print("MP !! %s: citizen population is not total x fraction" % d.get("name", "?"))
		# The DEMOGRAPHIC half of the model must be untouched by the ruling:
		# the government changed, so standing moves, but the total population
		# and the levy (pool x levy_reach) must not be moved by the citizen
		# fraction at all. levy_reach reads state_capacity, which the
		# government does move -- so only total population is a strict pin.
		var b: Array = before.get(int(d.get("faction", -1)), [])
		if b.size() == 2 and abs(float(b[0]) - tp) > 1.0:
			ok = false
			print("MP !! %s: total population moved when only the government did" % d.get("name", "?"))
	if fracs.is_empty() or is_equal_approx(fracs.min(), fracs.max()):
		ok = false
		print("MP !! every faction has the SAME citizen fraction -- the denominator is a constant")
	else:
		print("MP citizen fraction spread %.3f .. %.3f across %d governments  OK" % [
			fracs.min(), fracs.max(), mixed.size()])

	## And the pin the ruling most needs: with the roster put back exactly as
	## it was, every headcount must equal what it was before this section ran.
	for i in range(1, gen.civ_faction_count() + 1):
		gen.civ_set_faction_field(i, "government", "monarchy")
	var restored: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
	for f in restored:
		var d: Dictionary = f
		var m: Dictionary = d.get("manpower", {})
		var b: Array = before.get(int(d.get("faction", -1)), [])
		if b.size() == 2 and abs(float(b[1]) - float(m.get("emergency_mobilization", 0.0))) > 1.0:
			ok = false
			print("MP !! %s: levy did not restore (%.0f -> %.0f)" % [
				d.get("name", "?"), float(b[1]), float(m.get("emergency_mobilization", 0.0))])

	print("MP RESULT %s" % ("PASS" if ok else "FAIL"))
	get_tree().quit(0 if ok else 1)
