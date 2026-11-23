extends SceneTree
## Ruling HM census: every settlement's town laid out as the app lays it out, with `civ.bank_sites` off (the town round
## its cell, its own narrow channel: before the ruling) and on (the town round the drawn river's edge, its river as wide
## as the map draws it), measured by `WorldGen.town_bank_census` against the ruling's bars.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_towncensus_probe.gd -- \
##       [--seeds a,b,c] [--gw N] [--gh N] [--view WxH] [--params FILE] [--rows] [--cell X,Y] [--no-villages]
##
## Headless: it reads engine state only (no pixels). Run from a project copy with its own `config/name`.
## `--params` is a text file of `key: value` lines (an owner's world, as the app lists its parameters); lines that are
## not a parameter are skipped and every refused key is printed. `--cell` prints the row of the settlement at that cell
## in both states. `--rows` prints a row per town whose river, market bank or water footprint is off a bar.
##
## The river towns are the settlements whose layout has a river with the switch OFF, so "lost its river" has a fixed
## denominator. Gates (exit 1): no river town at all (an empty census); with the switch on, a river town without a
## river, more markets inside their town's river than with it off, a building in water, a building over a drawn river,
## or a marker on the other bank from its market; and no town moved at all (the switch reached nothing). Wall and
## street points over a drawn river are printed and not gated (a wall or a bridge that meets the river crosses its edge).

func _p10(v: Array) -> float:
	if v.is_empty():
		return NAN
	v.sort()
	return v[int(floor(v.size() * 0.1))]

func _init() -> void:
	var seeds := PackedInt32Array([483920, 912345, 246371])
	var gw := 384
	var gh := 288
	var view := Vector2(1150, 800)
	var params_file := ""
	var show_rows := false
	var villages := true
	var cell := Vector2i(-1, -1)
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--seeds":
				seeds = PackedInt32Array()
				for t in a[i + 1].split(","):
					seeds.append(int(t))
			"--gw": gw = int(a[i + 1])
			"--gh": gh = int(a[i + 1])
			"--view":
				var p := a[i + 1].split("x")
				view = Vector2(float(p[0]), float(p[1]))
			"--params": params_file = a[i + 1]
			"--cell":
				var c := a[i + 1].split(",")
				cell = Vector2i(int(c[0]), int(c[1]))
			"--rows":
				show_rows = true
				i -= 1
			"--no-villages":
				villages = false
				i -= 1
			_:
				push_error("unknown argument " + a[i])
				quit(2)
				return
		i += 2
	var bad := false
	for seed in seeds:
		var wg: WorldGen = WorldGen.new()
		wg.set_villages_enabled(villages)
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
				## The type the engine already holds for the key, so `1` stays an int and `1.0` a float.
				match typeof(known[k]):
					TYPE_BOOL: set[k] = v == "true"
					TYPE_INT: set[k] = int(v)
					TYPE_FLOAT: set[k] = float(v)
					_: set[k] = v
			var res = wg.set_params(set)
			print("TOWN params from %s: %d keys set; engine answered %s" % [params_file, set.size(), str(res).left(300)])
		wg.generate_sized(seed, 800.0, gw, gh)
		var places: Array = wg.get_settlements()
		print("\nTOWN seed %d at %dx%d, view %dx%d: %d settlements" % [seed, gw, gh, int(view.x), int(view.y), places.size()])
		var all := PackedInt32Array()
		for k in places.size():
			all.append(k)
		var state := {}
		for on in [false, true]:
			wg.set_params({"civ.bank_sites": on})
			var t0 := Time.get_ticks_msec()
			var rows: Array = wg.town_bank_census(all, view.x, view.y)
			state[on] = rows
			print("TOWN   bank_sites %s: %d rows in %.1f s" % ["on" if on else "off", rows.size(), (Time.get_ticks_msec() - t0) / 1000.0])
		var river_towns := {}
		for r in state[false]:
			if r.get("river", false):
				river_towns[int(r["i"])] = true
		if river_towns.is_empty():
			print("TOWN FAIL: no river town: an empty census")
			bad = true
			continue
		var off_housed := {}
		var off_in_river := 0
		for on in [false, true]:
			var n := 0
			var lost := 0
			var in_river := 0
			var wet := 0
			var fronted := 0
			var reach := 0
			var over := {"b": 0, "w": 0, "s": 0}
			var over_of := {"b": 0, "w": 0, "s": 0}
			var housed: Array = []
			var housed_all: Array = []
			var sides := {"x1": [0, 0], "x4": [0, 0]}
			var moved := 0
			var far := 0
			var grown := 0
			var origin_m: Array = []
			for r in state[on]:
				var k := int(r["i"])
				if r.get("laid", false):
					housed_all.append(100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])))
				if not river_towns.has(k):
					continue
				n += 1
				var note := ""
				if not r.get("river", false):
					lost += 1
					note += " NO RIVER"
				if r.has("market_river_m") and float(r["market_river_m"]) < float(r["river_w"]) * 0.5:
					in_river += 1
					note += " MARKET IN RIVER (%.0f m of %.0f)" % [r["market_river_m"], r["river_w"]]
				if r.get("laid", false):
					wet += int(r["wet"])
					fronted += int(r["fronted"])
					reach += int(r["reach"])
					for kk in ["b", "w", "s"]:
						over[kk] += int(r["over_" + kk])
						over_of[kk] += int(r["of_" + kk])
					housed.append(100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])))
					if int(r["wet"]) > 0:
						note += " WET %d" % r["wet"]
					if int(r["over_b"]) + int(r["over_w"]) > 0:
						note += " OVER b %d w %d" % [r["over_b"], r["over_w"]]
					if on and housed[-1] < 0.8 * off_housed.get(k, 0.0):
						note += " HOUSED %.0f%% from %.0f%%" % [housed[-1], off_housed[k]]
					if not on:
						off_housed[k] = housed[-1]
				for z in ["x1", "x4"]:
					if r.has("side_" + z):
						sides[z][0] += 1
						if int(r["side_" + z]) < 0:
							sides[z][1] += 1
							note += " OTHER BANK " + z
				if r.has("origin_m"):
					moved += 1
					origin_m.append(float(r["origin_m"]))
					far += int(float(r["river_m"]) + 100.0 > 625.0)
				grown += int(r.get("laid", false) and (float(r["wm"]) > 1700.5 or float(r["hm"]) > 1250.5))
				var p: Dictionary = places[k]
				if (show_rows and note != "") or Vector2i(p["x"], p["y"]) == cell:
					print("TOWN     %s #%d %s (%d,%d) pop %d:%s | box %.0fx%.0f river_w %.0f market-river %s origin %s river_m %s over b %s/%s w %s s %s wet %s fronted %s/%s housed %.0f%%" % ["on " if on else "off", k, p["kind"], p["x"], p["y"], int(p.get("pop", 0)), note,
						r.get("wm", 0.0), r.get("hm", 0.0), r.get("river_w", 0.0), ("%.0f" % r["market_river_m"]) if r.has("market_river_m") else "-",
						("%.0f" % r["origin_m"]) if r.has("origin_m") else "-", ("%.0f" % r["river_m"]) if r.has("river_m") else "-",
						r.get("over_b", "-"), r.get("of_b", "-"), r.get("over_w", "-"), r.get("over_s", "-"), r.get("wet", "-"), r.get("fronted", "-"), r.get("reach", "-"),
						100.0 * float(r.get("pop", 0.0)) / maxf(1.0, float(r.get("pop_target", 1.0)))])
			origin_m.sort()
			print("TOWN   %s: river towns %d | lost river %d | market in its river %d | buildings in water %d | fronted %d/%d (%.1f %%) | housed p10 %.0f %% (all towns %.0f %%) | over drawn river: buildings %d of %d, wall %d of %d, streets %d of %d | other bank x1 %d/%d x4 %d/%d | moved %d (median %.0f m, max %.0f m), past the 625 m reach %d, box grown %d" % [
				"on " if on else "off", n, lost, in_river, wet, fronted, reach, 100.0 * fronted / maxf(1.0, reach), _p10(housed), _p10(housed_all), over["b"], over_of["b"], over["w"], over_of["w"], over["s"], over_of["s"],
				sides["x1"][1], sides["x1"][0], sides["x4"][1], sides["x4"][0], moved,
				origin_m[origin_m.size() / 2] if not origin_m.is_empty() else 0.0, origin_m[-1] if not origin_m.is_empty() else 0.0, far, grown])
			if on:
				## The wall and street counts are printed, not gated: a wall or a bridge that meets the river crosses its edge.
				if lost > 0 or in_river > off_in_river or wet > 0 or over["b"] > 0 or sides["x1"][1] > 0 or sides["x4"][1] > 0:
					print("TOWN FAIL: a bar is missed with the switch on (lost %d, market in river %d against %d off, wet %d, buildings over %d, other bank x1 %d x4 %d)" % [lost, in_river, off_in_river, wet, over["b"], sides["x1"][1], sides["x4"][1]])
					bad = true
				if moved == 0:
					print("TOWN FAIL: no town moved: the switch reached nothing")
					bad = true
			else:
				off_in_river = in_river
				if moved != 0:
					print("TOWN FAIL: a town moved with the switch off")
					bad = true
		wg = null
	quit(1 if bad else 0)
