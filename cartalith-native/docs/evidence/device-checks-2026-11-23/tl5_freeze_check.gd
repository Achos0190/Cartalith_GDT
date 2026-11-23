extends SceneTree
## TL-5 x86-64 leg: generate, freeze every town, save to --out, reopen, check.
func _init() -> void:
	var a := OS.get_cmdline_user_args()
	var seed_v := int(a[0]); var out: String = a[1]
	var mode: String = a[2] # "make" or "check"
	var wg: WorldGen = WorldGen.new()
	if mode == "make":
		wg.generate_sized(seed_v, 800.0, 384, 288)
		var n := wg.get_settlements().size()
		var frozen := 0
		for i in n:
			if wg.town_freeze(i): frozen += 1
		print("TL5 settlements %d frozen %d" % [n, frozen])
		var r: Dictionary = wg.project_save(out)
		print("TL5 save ", r.get("ok"), " ", r.get("bytes"), " ", r.get("error"))
		var c: Dictionary = wg.towns_check_all()
		print("TL5 same-session check: count %s unchanged %s rounding %s changed %s" % [c.count, c.unchanged, c.rounding, c.changed])
		wg = WorldGen.new()
	var o: Dictionary = wg.project_open(out)
	print("TL5 open ok=", o.get("ok"), " ", str(o.get("error", "")).left(200))
	var c2: Dictionary = wg.towns_check_all()
	print("TL5 reopen check: count %s unchanged %s rounding %s changed %s damaged %s gone %s" % [c2.count, c2.unchanged, c2.rounding, c2.changed, c2.damaged, c2.gone])
	for row in c2.rows:
		if row.verdict != "unchanged": print("TL5 row ", row)
	quit(0)
