extends SceneTree
## CS-0 (COASTAL_SETBACK_RESEARCH.md, Ruling HQ, 2026-11-23): the layout side of the measurement. Every settlement of
## one world, laid out as the app lays it out today (`WorldGen.town_bank_census` and `WorldGen.town_debug_json`, both
## built on `town_batch` + `stretch_layout`, so the HM/HN origin, the drawn river and the map's drawn lake and sea are
## the app's), dumped as one JSON line per settlement for `cs0_hazard_census` (`town_river_census.rs`) to join with the
## terrain's hazard fields. Measures only; writes nothing to the world.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path <project copy> --script res://cs0_probe.gd -- \
##       --seed N --gw N --gh N --out FILE [--params FILE] [--step M] [--app]
##
## Headless: it reads engine state only (no pixels). `--params`: an owner's `key: value` list, applied with
## `set_params` before `generate_sized`, exactly as `_towncensus_probe.gd` applies it (MISTAKES row 1).
## `--app`: the world `EngineBridge.generate` makes for a New World request with its defaults (the shell's "app world":
## the four experimental flags off, sea level 0.5, metropolis and recovery off, biome K off), as `_viewerframe_probe.gd`
## generates it; the FK row's hamlet 154:136 is in that world, not in `generate_sized`'s default one.
## Each line: the settlement (`i`, `x`, `y`, `tid`, `kind`, `pop`, `coastal`, `capital`), the cell's `elev` (sample_cell,
## the identity check against the Rust side), the census row `c` (town_bank_census's keys) and, for a laid town, `d`
## (town_debug_json at `--step` metres: market, pin, box, buildings, edges with class, and the map's drawn
## land/sea/lake/river-stroke lattice). The first line is the world: seed, grid, bank_sites, settlement count.

func _init() -> void:
	var seed := 483920
	var gw := 384
	var gh := 288
	var out := ""
	var params_file := ""
	var step := 8.0
	var app := false
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--seed": seed = int(a[i + 1])
			"--gw": gw = int(a[i + 1])
			"--gh": gh = int(a[i + 1])
			"--out": out = a[i + 1]
			"--params": params_file = a[i + 1]
			"--step": step = float(a[i + 1])
			"--app":
				app = true
				i -= 1
			_:
				push_error("unknown argument " + a[i])
				quit(2)
				return
		i += 2
	if out == "":
		push_error("--out is required")
		quit(2)
		return
	var wg: WorldGen = WorldGen.new()
	wg.set_villages_enabled(true)
	if params_file != "":
		var f := FileAccess.open(params_file, FileAccess.READ)
		if f == null:
			push_error("cannot read " + params_file)
			quit(2)
			return
		var known: Dictionary = wg.get_params()
		var set := {}
		while not f.eof_reached():
			var line := f.get_line()
			var k := line.get_slice(": ", 0)
			if not known.has(k) or line.find(": ") < 0:
				continue
			var v := line.get_slice(": ", 1).strip_edges()
			match typeof(known[k]):
				TYPE_BOOL: set[k] = v == "true"
				TYPE_INT: set[k] = int(v)
				TYPE_FLOAT: set[k] = float(v)
				_: set[k] = v
		var res = wg.set_params(set)
		print("CS0 params from %s: %d keys set; engine answered %s" % [params_file, set.size(), str(res).left(200)])
	if app:
		## `EngineBridge.generate`'s own calls, with a New World request's defaults (`shell/engine_bridge.gd`).
		wg.set_experimental_flags(false, false, false, false)
		wg.set_metropolis_enabled(false)
		wg.set_recovery_phase(0)
		wg.set_biome_k_enabled(false)
		wg.set_sea_level(0.5)
	var t0 := Time.get_ticks_msec()
	wg.generate_sized(seed, 800.0, gw, gh)
	var places: Array = wg.get_settlements()
	var p: Dictionary = wg.get_params()
	print("CS0 seed %d at %dx%d: %d settlements, generated in %.1f s, civ.bank_sites %s" % [seed, gw, gh, places.size(), (Time.get_ticks_msec() - t0) / 1000.0, str(p.get("civ.bank_sites"))])
	var fo := FileAccess.open(out, FileAccess.WRITE)
	fo.store_line(JSON.stringify({"seed": seed, "gw": gw, "gh": gh, "n": places.size(), "bank_sites": p.get("civ.bank_sites"), "step": step, "params_file": params_file, "app": app, "all_params": p}))
	var all := PackedInt32Array()
	for k in places.size():
		all.append(k)
	t0 = Time.get_ticks_msec()
	var rows: Array = wg.town_bank_census(all, 1150.0, 800.0)
	print("CS0 census rows %d in %.1f s" % [rows.size(), (Time.get_ticks_msec() - t0) / 1000.0])
	var by_i := {}
	for r in rows:
		by_i[int(r["i"])] = r
	t0 = Time.get_ticks_msec()
	var laid := 0
	for k in places.size():
		var s: Dictionary = places[k]
		var cell: Dictionary = wg.sample_cell(int(s["x"]), int(s["y"]))
		var row := {"i": k, "x": s["x"], "y": s["y"], "tid": s["tid"], "kind": s["kind"], "pop": s["population"], "coastal": s["coastal"], "capital": s["capital"], "elev": cell.get("elevation", -1.0)}
		var c: Dictionary = by_i.get(k, {})
		row["c"] = c
		if c.get("laid", false):
			var js := wg.town_debug_json(k, step)
			if js != "":
				row["d"] = JSON.parse_string(js)
				laid += 1
		fo.store_line(JSON.stringify(row))
	fo.close()
	print("CS0 dumped %d settlements (%d laid) in %.1f s to %s" % [places.size(), laid, (Time.get_ticks_msec() - t0) / 1000.0, out])
	wg = null
	quit(0)
