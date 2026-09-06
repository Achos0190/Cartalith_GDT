extends Node
## Scratch measurement harness for owner ruling 27 (2026-09-06): what a
## `TimelineSnapshot` actually costs in resident memory, and how many of them a
## normal timeline produces.
##
## Run (the host poller is what reads the memory -- a process cannot see its own
## Rust allocations, so this file only *creates* the state and holds it still):
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _tlmem_probe.tscn \
##       -- [--years N] [--gw W] [--gh H] [--seed S] [--hold MS]
##          [--sim [--simdur YEARS] [--simstep YEARS]] [--save] [--scrub N]
##
## Every flag above is one this file's own `_arg`/`_flag` actually reads, and
## anything else is REFUSED rather than defaulted through: a header naming a
## flag the body ignores measures the same case N times and reports N cases.
##
## `--years N` adds N recorded years with `civ_add_year`. `--sim` instead runs a
## collapse request, defaulting to duration 100 / step 10 -- the values
## `civilization_workspace.gd` initialises `_tl_sim_duration`/`_tl_sim_step_years`
## to -- so the "normal timeline" count comes off the shipped defaults rather
## than an assumption; `--simdur`/`--simstep` override them. `--save` writes the
## project to an absolute path, reports its byte size, reopens it into a second
## `WorldGen` and compares the recorded years and the restored territory census.
##
## The last thing it does before quitting is hold still for `--hold` ms with the
## world and its timeline alive, so the poller's final samples are steady-state
## residency and not a transient mid-allocation peak. Peak and steady state are
## different numbers here and both matter: generation itself peaks well above
## what it settles at, so a peak-only reading cannot see the timeline at all
## until the timeline exceeds it.
##
## Density-independent: nothing here draws.

var _gen: WorldGen


func _arg(name: String, dflt: int) -> int:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	if i < 0 or i + 1 >= a.size():
		return dflt
	return int(a[i + 1])


func _flag(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)


func _ready() -> void:
	var known := ["--years", "--gw", "--gh", "--hold", "--seed", "--sim", "--save", "--simdur", "--simstep", "--scrub"]
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--") and not known.has(a):
			print("UNKNOWN ARG ", a, " -- refusing rather than defaulting through it")
			get_tree().quit(2)
			return
	var years := _arg("--years", 0)
	var gw := _arg("--gw", 2048)
	var gh := _arg("--gh", 1311)
	var hold := _arg("--hold", 3000)
	var seed_v := _arg("--seed", 483920)
	var sim := _flag("--sim")

	print("TLMEM cfg years=%d gw=%d gh=%d sim=%s seed=%d" % [years, gw, gh, sim, seed_v])
	_gen = WorldGen.new()
	_gen.generate_sized(seed_v, 800.0, gw, gh)

	var settlements: Array = _gen.get_settlements()
	var roads: Array = _gen.get_roads()
	var pts := 0
	for r in roads:
		var d: Dictionary = r
		for k in ["points", "pts", "polyline"]:
			if d.has(k):
				pts += (d[k] as Array).size() if d[k] is Array else (d[k] as PackedVector2Array).size()
				break
	print("TLMEM world settlements=%d roads=%d road_points=%d cells=%d" % [
		settlements.size(), roads.size(), pts, gw * gh])
	print("TLMEM road_keys=", ("" if roads.is_empty() else str((roads[0] as Dictionary).keys())))

	if sim:
		var res: Dictionary = _gen.civ_run_collapse_simulation({
			"mode": "collapse", "character": "mixed", "severity": 0.5, "rate": 0.01,
			"start_year": 0, "duration": _arg("--simdur", 100),
			"step_years": _arg("--simstep", 10), "confirm_overwrite": true,
		})
		print("TLMEM sim ok=", res.get("ok", false), " steps=", res.get("steps", -1),
			" end_year=", res.get("end_year", -1))
	else:
		for i in years:
			_gen.civ_add_year(i * 10)

	var rec: PackedInt64Array = _gen.get_civ_timeline_years()
	print("TLMEM recorded_years=%d" % rec.size())

	## `--scrub N` builds N recorded years that each really CHANGE the territory -- a dab of
	## `civ_territory_paint_at` + `civ_territory_commit` before every `civ_add_year`, because
	## `civ_add_year` on its own carries the raster forward unchanged and would time a chain of
	## empty deltas. Then it drags the year cursor over every recorded year, forwards and back,
	## and reports the median with min..max: a `civ_goto_year` is one `civ_territory_at` plus a
	## copy into the live grid, which is the cost `TERRITORY_KEYFRAME_INTERVAL` bounds.
	var scrub := _arg("--scrub", 0)
	if scrub > 0:
		for t in scrub:
			var cx := 200.0 + float(t % 40) * 30.0
			var cy := 200.0 + float(t % 25) * 30.0
			_gen.civ_territory_paint_at(cx, cy, 1 + (t % 5), 30.0, false)
			_gen.civ_territory_commit()
			_gen.civ_add_year(t * 10)
		rec = _gen.get_civ_timeline_years()
		print("TLMEM scrub_years=%d" % rec.size())
		var us := []
		for pass_i in 3:
			var order := []
			for y in rec:
				order.append(y)
			if pass_i == 1:
				order.reverse()
			for y in order:
				var t0 := Time.get_ticks_usec()
				_gen.civ_goto_year(y)
				us.append(Time.get_ticks_usec() - t0)
		us.sort()
		print("TLMEM scrub_goto_us n=%d median=%d min=%d max=%d p90=%d" % [
			us.size(), us[us.size() / 2], us[0], us[us.size() - 1],
			us[int(float(us.size()) * 0.9)]])

	## The archive half of ruling 27, measured rather than assumed: this batch changed the
	## in-memory shape only, so `history/territory/<year>.i32` must still be written one whole
	## raster per recorded year and the file must still round-trip. `--save` writes the project,
	## reports its byte size, reopens it, and reports the recorded years and the territory the
	## year cursor restores -- so a delta chain that survived encoding but not the archive would
	## show up as a different year count or a different restored raster.
	if _flag("--save"):
		## An absolute OS path, not a `user://` one: `project_save` is Rust `std::fs` and does
		## not resolve Godot's virtual prefixes -- passing one gets `os error 123` back.
		var abs := ProjectSettings.globalize_path("user://_tlmem_probe.cartalith")
		var path := abs
		var w: Dictionary = _gen.project_save(path)
		var sz := 0
		var f := FileAccess.open(abs, FileAccess.READ)
		if f != null:
			sz = f.get_length()
			f.close()
		print("TLMEM save ok=", w.get("ok", false), " bytes=", sz, " dict=", w, " path=", abs)
		var g2 := WorldGen.new()
		var r: Dictionary = g2.project_open(path)
		var rec2: PackedInt64Array = g2.get_civ_timeline_years()
		print("TLMEM open ok=", r.get("ok", false), " dict=", r, " recorded_years=", rec2.size(),
			" years_match=", str(rec2) == str(rec))
		## No binding returns the raw raster, so the restored territory is compared by its
		## per-faction cell census -- a histogram, not the bytes. The byte-exact property is
		## `project_bridge`'s own Rust round-trip test; this is the integration check that the
		## archive still carries a territory at all after the in-memory shape changed.
		if not rec.is_empty():
			var y := rec[rec.size() - 1]
			g2.civ_goto_year(y)
			_gen.civ_goto_year(y)
			var a := []
			var b := []
			for fid in range(0, 9):
				a.append(int((_gen.civ_faction_territory_stats(fid) as Dictionary).get("claimed_cells", -1)))
				b.append(int((g2.civ_faction_territory_stats(fid) as Dictionary).get("claimed_cells", -1)))
			print("TLMEM restored_census_identical=", a == b, " at_year=", y,
				" census=", a)
	print("TLMEM hold_start")
	OS.delay_msec(hold)
	print("TLMEM done")
	get_tree().quit(0)
