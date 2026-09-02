extends Node
## Angle-2 evidence probe: does a landmark pass actually place anything, and
## are the coordinates in grid-cell space? Engine only -- no shell boot, so it
## is safe headless.
##
##   Godot_v4.7.1 --headless --path . _lmdraw_probe.tscn

func _ready() -> void:
	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	var t0 := Time.get_ticks_msec()
	g.generate_sized(483920, 2400.0, 384, 288)
	print("generate ms: ", Time.get_ticks_msec() - t0)
	print("grid: ", g.get_width(), " x ", g.get_height() if g.has_method("get_height") else -1)
	var t1 := Time.get_ticks_msec()
	var r: Dictionary = g.landmark_run()
	var run_ms := Time.get_ticks_msec() - t1
	print("landmark_run ms: ", run_ms, "  reply: ", r.get("ok"), " placed=", r.get("placed"),
		" seconds=", r.get("seconds"), " error=", r.get("error"))
	var lms: Array = g.landmarks()
	print("landmarks(): ", lms.size())
	var minx := 1 << 30
	var maxx := -1
	var miny := 1 << 30
	var maxy := -1
	var by_class := {}
	for l in lms:
		var d: Dictionary = l
		minx = mini(minx, int(d.get("x", 0)))
		maxx = maxi(maxx, int(d.get("x", 0)))
		miny = mini(miny, int(d.get("y", 0)))
		maxy = maxi(maxy, int(d.get("y", 0)))
		var c := String(d.get("class", "?"))
		by_class[c] = int(by_class.get(c, 0)) + 1
	if lms.size() > 0:
		print("x range: ", minx, "..", maxx, "   y range: ", miny, "..", maxy)
		print("by class: ", by_class)
		print("first: ", lms[0])
	var fn: Array = g.landmark_funnels()
	var placed_by_kind := {}
	for f in fn:
		var d: Dictionary = f
		if int(d.get("placed", 0)) > 0:
			placed_by_kind[String(d.get("kind", ""))] = int(d.get("placed", 0))
	print("funnels: ", fn.size(), "  kinds that placed: ", placed_by_kind)
	get_tree().quit()
