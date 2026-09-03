extends Node
## What does building the rejected-candidate list cost the PASS itself?
##
## The boundary cost is measured in `_lmreject_probe.gd`. This one isolates the
## other half: three timings of `landmark_run()` at the shipping 2048x1311
## default, so the same run can be taken with `REJECT_LIST_MAX_PER_KIND` at its
## shipped 256 and again at 0 (which skips every push and every extra
## `nearest_conflict_sq` scan) and the two compared.
##
##   Godot_v4.7.1 --headless --path . _lmrunab_probe.tscn

func _ready() -> void:
	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	g.generate_sized(483920, 2400.0, 2048, 1311)
	for i in 3:
		var t := Time.get_ticks_usec()
		g.landmark_run()
		print("landmark_run #", i, " us: ", Time.get_ticks_usec() - t)
	var rows: int = g.landmark_rejects().size() if g.has_method("landmark_rejects") else -1
	print("reject rows retained: ", rows)
	get_tree().quit()
