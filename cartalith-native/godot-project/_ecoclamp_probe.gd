extends Node
## Scratch measurement harness for owner ruling 11 (2026-09-06): the raw
## `land_capacity / total_pop` distribution that decides `ecological_factor`'s
## clamp in `cartalith-civ/src/manpower.rs`.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _ecoclamp_probe.tscn
##
## Prints one row per faction per world, then the pooled distribution. `eco`
## is whatever the CURRENT clamp reports, so this reads the RAW ratio only
## while both clamp ends are sentinels far outside any real value -- which is
## how the before-distribution was taken. Run it again after the ceiling moves
## for the after-distribution: the same worlds, now clamped by the chosen
## value, and the saturation count is the thing to compare.
##
## Six seeds x three world shapes on purpose: one world is one sample
## (`MISTAKES.md`), and `land_capacity` integrates cell area while
## `nucleated_pop` does not, so the ratio could in principle be scale-bound.
## It is: the pooled median moved 0.39 -> 5.04 between the 800 km and the
## 2 000 km shape at the same faction count.
##
## `ECOROW` is tab-separated, in this column order, so a run can be diffed
## against another run:
##   shape, seed, faction, government, ag_tech, ecological_factor,
##   total_population, citizen_population, standing_army, field_army,
##   emergency_mobilization, standing_citizen_share, emergency_citizen_share,
##   era, era_standing_lo, era_standing_hi, era_mobilization_lo,
##   era_mobilization_hi, era_standing_verdict, era_mobilization_verdict

const SEEDS := [483920, 7, 101, 202501, 999331, 31337]
## label, width km, grid w, grid h. B is the shipped default preset's width
## (`new_world_dialog.gd` SIZE_PRESETS "Province - 800 km"); C is "Region -
## 2 000 km"; A is the world `_manpower_probe.gd` already measures.
const WORLDS := [
	["A 384x288 1200km", 1200.0, 384, 288],
	["B 512x384 800km", 800.0, 512, 384],
	["C 768x576 2000km", 2000.0, 768, 576],
]


func _pct(sorted: Array, q: float) -> float:
	if sorted.is_empty():
		return NAN
	var i := int(round(q * float(sorted.size() - 1)))
	return float(sorted[i])


func _ready() -> void:
	var pooled := []
	for w in WORLDS:
		var label: String = w[0]
		for sd in SEEDS:
			var gen := WorldGen.new()
			gen.generate_sized(int(sd), float(w[1]), int(w[2]), int(w[3]))
			var factions: Array = (gen.civ_military_summary() as Dictionary).get("factions", [])
			var vals := []
			for f in factions:
				var d: Dictionary = f
				var m: Dictionary = d.get("manpower", {})
				if m.is_empty():
					continue
				var eco := float(m.get("ecological_factor", 0.0))
				vals.append(eco)
				pooled.append(eco)
				print("ECOROW	%s	%d	%s	%s	%s	%.6f	%.6f	%.6f	%.6f	%.6f	%.6f	%.8f	%.8f	%s	%.6f	%.6f	%.6f	%.6f	%s	%s" % [
					label, int(sd), d.get("name", "?"), m.get("government", "?"),
					m.get("ag_tech", "?"), eco,
					float(m.get("total_population", 0.0)),
					float(m.get("citizen_population", 0.0)),
					float(m.get("standing_army", 0.0)),
					float(m.get("field_army", 0.0)),
					float(m.get("emergency_mobilization", 0.0)),
					float(m.get("standing_citizen_share", 0.0)),
					float(m.get("emergency_citizen_share", 0.0)),
					m.get("era", "?"),
					float(m.get("era_standing_lo", 0.0)), float(m.get("era_standing_hi", 0.0)),
					float(m.get("era_mobilization_lo", 0.0)), float(m.get("era_mobilization_hi", 0.0)),
					m.get("era_standing_verdict", "?"), m.get("era_mobilization_verdict", "?")])
			vals.sort()
			print("ECOSEED %-18s seed=%-7d n=%d min=%.4f med=%.4f max=%.4f settlements=%d" % [
				label, int(sd), vals.size(), _pct(vals, 0.0), _pct(vals, 0.5),
				_pct(vals, 1.0), gen.get_settlements().size()])
	pooled.sort()
	print("ECOPOOL n=%d min=%.4f p10=%.4f p25=%.4f median=%.4f p75=%.4f p90=%.4f max=%.4f" % [
		pooled.size(), _pct(pooled, 0.0), _pct(pooled, 0.10), _pct(pooled, 0.25),
		_pct(pooled, 0.50), _pct(pooled, 0.75), _pct(pooled, 0.90), _pct(pooled, 1.0)])
	for t in [0.25, 1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0]:
		var n := 0
		for v in pooled:
			if float(v) >= float(t):
				n += 1
		print("ECOCUT >=%.2f : %d of %d (%.1f%%)" % [t, n, pooled.size(),
			100.0 * float(n) / float(max(1, pooled.size()))])
	print("ECO done")
	get_tree().quit(0)
