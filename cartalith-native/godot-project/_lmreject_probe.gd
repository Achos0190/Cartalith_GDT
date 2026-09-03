extends Node
## Lane B evidence probe: the landmark funnel's rejected-candidate list.
##
## Answers three questions the ruling's second half depends on, at the shell's
## own default grid (2048x1311) rather than a toy one:
##
##   1. How many candidates are offered and then rejected? That is the size of
##      the payload the new list has to carry.
##   2. What does `landmark_run()` cost, and what does pulling the rejects
##      across the gdext boundary add on top? The POI pass froze this app for
##      4.14 s by marshalling a per-cell payload from a worker; a per-candidate
##      coordinate list is the same shape, so the number is measured, not
##      assumed.
##   3. Do the reason counts agree with the funnel's own scalars? A dot that
##      cannot say why it was rejected is worse than the scalar it replaces, so
##      every reject carries a reason and the reasons must close on the funnel.
##
##   Godot_v4.7.1 --headless --path . _lmreject_probe.tscn

func _ready() -> void:
	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	var t0 := Time.get_ticks_msec()
	g.generate_sized(483920, 2400.0, 2048, 1311)
	print("generate ms: ", Time.get_ticks_msec() - t0)
	print("grid: ", g.get_width(), " x ", g.get_height())

	## Three samples: the pass is 3 s of real work and one reading of it is a
	## number, not a measurement.
	for pass_i in 3:
		var t1 := Time.get_ticks_usec()
		var ok: bool = g.landmark_run()
		print("landmark_run #", pass_i, " us: ", Time.get_ticks_usec() - t1, " returned=", ok)
	var r: Dictionary = g.landmark_last_run()
	print("last_run: ok=", r.get("ok"), " placed=", r.get("placed"),
		" seconds=", r.get("seconds"), " error=", r.get("error"))

	## The funnel's own arithmetic, summed over every kind. `offered` is what a
	## rejected-candidate list could ever contain: cells that entered `Pool.cands`
	## and did not become a placement. `rejected_constraint` is deliberately NOT
	## in it -- that bucket is "this cell is not a waterfall", counted per scanned
	## cell, and on this grid it runs to millions.
	var fn: Array = g.landmark_funnels()
	var c_all := 0
	var c_constraint := 0
	var c_score := 0
	var c_spacing := 0
	var c_cap := 0
	var c_placed := 0
	for f in fn:
		var d: Dictionary = f
		c_all += int(d.get("candidates", 0))
		c_constraint += int(d.get("rejected_constraint", 0))
		c_score += int(d.get("rejected_score", 0))
		c_spacing += int(d.get("rejected_spacing", 0))
		c_cap += int(d.get("rejected_cap", 0))
		c_placed += int(d.get("placed", 0))
	print("funnel totals: candidates=", c_all, " constraint=", c_constraint,
		" score=", c_score, " spacing=", c_spacing, " cap=", c_cap, " placed=", c_placed)
	print("offered-and-rejected (score+spacing+cap) = ", c_score + c_spacing + c_cap)

	if g.has_method("landmark_rejects"):
		var t2 := Time.get_ticks_usec()
		var rj: Array = g.landmark_rejects()
		var pull_us := Time.get_ticks_usec() - t2
		print("landmark_rejects us: ", pull_us, "  rows=", rj.size())
		var by_reason := {}
		var by_kind := {}
		var minx := 1 << 30
		var maxx := -1
		var min_needs := INF
		for e in rj:
			var d: Dictionary = e
			var w := String(d.get("reason", "?"))
			by_reason[w] = int(by_reason.get(w, 0)) + 1
			var k := String(d.get("kind", "?"))
			by_kind[k] = int(by_kind.get(k, 0)) + 1
			minx = mini(minx, int(d.get("x", 0)))
			maxx = maxi(maxx, int(d.get("x", 0)))
			## `has()`, not a default: the key is absent when there is no
			## figure, and `0.0` would be a plausible Crowding rather than an
			## obvious sentinel.
			if d.has("needs_crowding"):
				min_needs = minf(min_needs, float(d["needs_crowding"]))
		print("rejects by reason: ", by_reason)
		if rj.size() > 0:
			print("x range: ", minx, "..", maxx, "   first: ", rj[0])
		print("smallest needs_crowding across every spacing reject: ", min_needs)
		## The list is bounded per kind, so it closes on the funnel only for the
		## kinds that did not reach the bound. Checking it unconditionally was
		## wrong, and printed a misleading `false` on this probe's first run.
		var bound := 256
		var closed := 0
		var bounded := 0
		for f in fn:
			var d: Dictionary = f
			var kk := String(d.get("kind", ""))
			var total: int = (int(d.get("rejected_score", 0)) + int(d.get("rejected_spacing", 0))
				+ int(d.get("rejected_cap", 0)))
			var listed := int(by_kind.get(kk, 0))
			if total > bound:
				bounded += 1
				assert(listed == bound, "%s listed %d, bound is %d" % [kk, listed, bound])
			else:
				closed += 1
				assert(listed == total, "%s listed %d of %d" % [kk, listed, total])
		print("kinds closing exactly on the funnel: ", closed, "   kinds at the bound: ", bounded)

		var t3 := Time.get_ticks_usec()
		var pv: PackedVector2Array = g.landmark_reject_points("")
		print("landmark_reject_points('') us: ", Time.get_ticks_usec() - t3,
			"  points=", pv.size())
		var t4 := Time.get_ticks_usec()
		var pv2: PackedVector2Array = g.landmark_reject_points("spacing")
		print("landmark_reject_points('spacing') us: ", Time.get_ticks_usec() - t4,
			"  points=", pv2.size())
		print("landmark_reject_points('nonsense') points=",
			g.landmark_reject_points("nonsense").size(), " (must be 0)")

		## The layer itself, drawn for real. `_draw` is where a wrong argument
		## count or a bad `PackedVector2Array` shows up, and neither the parse
		## check nor the Rust tests can reach it.
		await _draw_the_layer(g)
	else:
		print("landmark_rejects: NOT BOUND (baseline run)")
	get_tree().quit()


## Instantiate the real `MapOverlay`, feed it a real world's rejects, and let it
## draw. Any error inside `_draw_landmark_rejects` prints as a SCRIPT ERROR and
## fails the run rather than passing silently.
func _draw_the_layer(g: WorldGen) -> void:
	var overlay: Control = load("res://map_overlay.gd").new()
	overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	overlay.size = Vector2(1280, 800)
	add_child(overlay)
	overlay.set_civ_data([], [], [], g.get_width(), g.get_height(), 0.0)
	overlay.set_landmarks(g.landmarks())
	var rejects := {}
	var total := 0
	for reason in ["spacing", "cap", "score"]:
		var pts: PackedVector2Array = g.landmark_reject_points(reason)
		rejects[reason] = pts
		total += pts.size()
	overlay.set_landmark_rejects(rejects)
	## Off by default: the layer must draw NOTHING until it is switched on.
	assert(not overlay._landmark_rejects_visible, "the diagnostic layer defaults on")
	overlay.set_landmark_rejects_visible(true)
	overlay.queue_redraw()
	await get_tree().process_frame
	await get_tree().process_frame
	print("overlay drew with ", total, " reject marks across ", rejects.size(), " reasons")
	overlay.queue_free()
