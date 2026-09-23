extends Node
## Measurement harness for Ruling AI's follow-up (owner, 2026-09-23): options
## (b) -- the map-scale normalisation of `ecological_factor` -- and (c) -- the
## alpha-dependent soldier upkeep -- in `cartalith-civ/src/manpower.rs`.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _mpscale_probe.tscn
##
## The same 6 seeds x 3 world shapes `_ecoclamp_probe.gd` measured for owner
## ruling 11, so the 108-sample figures in `MILITARY_MANPOWER_SCOPE.md` can be
## re-taken on identical worlds. Plus the scope document's two named worlds:
## the sparse 33-settlement one (1200 km, 384x288) and the dense 233-settlement
## one (2400 km, 384x288, villages on).
##
## Prints one MPROW per faction, one MPSHAPE summary per world shape, and the
## pooled MPPOOL line. `land_capacity` / `land_reference` are read only if the
## bridge publishes them (after option (b)); before it they print as -1.
##
## Committed, like every probe scene in this folder (`STATUS.md` F8): kept as
## the evidence for the pass that wrote it.

const SEEDS := [483920, 7, 101, 202501, 999331, 31337]
const WORLDS := [
	["A 384x288 1200km", 1200.0, 384, 288],
	["B 512x384 800km", 800.0, 512, 384],
	["C 768x576 2000km", 2000.0, 768, 576],
]


func _row(label: String, sd: int, d: Dictionary, nset: int) -> Dictionary:
	var m: Dictionary = d.get("manpower", {})
	var r := {
		"eco": float(m.get("ecological_factor", 0.0)),
		"sv": String(m.get("era_standing_verdict", "?")),
		"mv": String(m.get("era_mobilization_verdict", "?")),
	}
	print("MPROW\t%s\t%d\t%s\t%s\t%s\teco=%.4f\tland=%.0f\tref=%.4f\tpop=%.0f\tstanding=%.0f\tfield=%.0f\tlevy=%.0f\tstand%%cit=%.3f\tmob%%cit=%.2f\t%s\t%s/%s\tsettlements=%d" % [
		label, sd, d.get("name", "?"), m.get("government", "?"), m.get("ag_tech", "?"),
		r["eco"], float(m.get("land_capacity", -1.0)), float(m.get("land_reference", -1.0)),
		float(m.get("total_population", 0.0)), float(m.get("standing_army", 0.0)),
		float(m.get("field_army", 0.0)), float(m.get("emergency_mobilization", 0.0)),
		100.0 * float(m.get("standing_citizen_share", 0.0)),
		100.0 * float(m.get("emergency_citizen_share", 0.0)),
		m.get("era", "?"), r["sv"], r["mv"], nset])
	return r


func _tally(label: String, rows: Array) -> void:
	var c := {"below": 0, "within": 0, "above": 0}
	var mc := {"below": 0, "within": 0, "above": 0}
	var below_low_eco := 0
	var pinned_hi := 0
	var pinned_lo := 0
	var ecos := []
	for r in rows:
		c[r["sv"]] = int(c.get(r["sv"], 0)) + 1
		mc[r["mv"]] = int(mc.get(r["mv"], 0)) + 1
		if r["sv"] == "below" and float(r["eco"]) < 1.0:
			below_low_eco += 1
		if float(r["eco"]) >= 4.0:
			pinned_hi += 1
		if float(r["eco"]) <= 0.25:
			pinned_lo += 1
		ecos.append(float(r["eco"]))
	ecos.sort()
	var med: float = ecos[ecos.size() / 2] if not ecos.is_empty() else NAN
	print("%s %-20s n=%d standing below/within/above=%d/%d/%d (below with eco<1: %d)  mobilization=%d/%d/%d  eco median=%.3f min=%.3f max=%.3f pinned@4=%d pinned@0.25=%d" % [
		"MPSHAPE", label, rows.size(), c["below"], c["within"], c["above"], below_low_eco,
		mc["below"], mc["within"], mc["above"], med,
		ecos[0] if not ecos.is_empty() else NAN, ecos[-1] if not ecos.is_empty() else NAN,
		pinned_hi, pinned_lo])


func _world(gen: WorldGen, label: String, sd: int) -> Array:
	var out := []
	var nset := gen.get_settlements().size()
	for f in (gen.civ_military_summary() as Dictionary).get("factions", []):
		var d: Dictionary = f
		if (d.get("manpower", {}) as Dictionary).is_empty():
			continue
		out.append(_row(label, sd, d, nset))
	return out


func _ready() -> void:
	var pooled := []
	for w in WORLDS:
		var shape := []
		for sd in SEEDS:
			var gen := WorldGen.new()
			gen.generate_sized(int(sd), float(w[1]), int(w[2]), int(w[3]))
			shape.append_array(_world(gen, w[0], int(sd)))
		_tally(w[0], shape)
		pooled.append_array(shape)
	_tally("POOLED", pooled)

	# The scope document's two named worlds (MILITARY_MANPOWER_SCOPE.md 3.2/3.2a).
	var sparse := WorldGen.new()
	sparse.generate_sized(483920, 1200.0, 384, 288)
	_tally("SPARSE 33", _world(sparse, "SPARSE", 483920))
	var dense := WorldGen.new()
	dense.set_villages_enabled(true)
	dense.generate_sized(483920, 2400.0, 384, 288)
	_tally("DENSE", _world(dense, "DENSE", 483920))
	print("MP done")
	get_tree().quit(0)
