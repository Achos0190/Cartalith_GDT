extends SceneTree
## The town-on-bank census (Rulings HM and HN): every settlement's town laid out as the app lays it out, with
## `civ.bank_sites` off (the town round its cell, its own narrow channel, the reference's lake and sea cells) and on (the
## town on the drawn river's bank, its river as wide as the map draws it, its lake and sea edges the map's), measured
## by `WorldGen.town_bank_census` against the rulings' bars.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_towncensus_probe.gd -- ##       [--seeds a,b,c] [--gw N] [--gh N] [--view WxH] [--params FILE] [--rows] [--cell X,Y] [--cell-min-housed P] [--max-quartered N] [--no-villages]
##       [--time] [--slider] [--dump 'DIR|i,j']
##
## Headless: it reads engine state only (no pixels). Run from a project copy with its own `config/name`.
## `--params` is a text file of `key: value` lines (an owner's world, as the app lists its parameters); lines that are
## not a parameter are skipped and every refused key is printed. `--cell` prints the row of the settlement at that cell
## in both states. `--rows` prints a row per town off a bar or whose housing fell by a fifth.
## `--time`: what the marker path costs (a new world state, then a zoom step), alone. `--slider`: the river look's
## width must move no town. `--dump`: one JSON per listed town and state for `plot_town.py`
## (`docs/evidence/town-on-bank-hn-2026-11-23/`).
##
## The river towns are the settlements whose layout has a river with the switch OFF, so "lost its river" has a fixed
## denominator. Footprint over water is measured on lines sampled every 4 m, not on vertices
## (`town_measures::footprint_over`, which defines a real bridge and a gated river wall).
## Gates (exit 1), with the switch on: a river town without a river; more markets inside their town's river than
## with it off (river sites only); a building in the town's own water; a building, an ungated wall run or a street
## that is no bridge over a drawn river; any of the three over the map's drawn lake or sea; a marker across a drawn
## river from its market at x1, x4, x16 or x64; housed p10 under the switch-off p10; a town whose target moved; a
## walled town with no street within 150 m of its market that had one with the switch off (Ruling HQ); no town moved at
## all. Positive controls (also exit 1): no river town; no building over a drawn river with the switch
## off; no bridge counted with it on. The run's last line gives the bridge edges, the gated wall runs and the
## footprint over the map's lake or sea with the switch off (that measure's control).

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
	var time_only := false
	var slider := false
	var dump_dir := ""
	var dump := PackedInt32Array()
	var villages := true
	var cell := Vector2i(-1, -1)
	var cell_min := -1.0
	var max_quartered := -1
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
			"--cell-min-housed": cell_min = float(a[i + 1])
			"--max-quartered": max_quartered = int(a[i + 1])
			"--rows":
				show_rows = true
				i -= 1
			"--time":
				time_only = true
				i -= 1
			"--slider":
				slider = true
				i -= 1
			"--dump":
				## `--dump OUT_DIR:i,j,k`: write town_<seed>_<i>_<off|on>.json for those settlements and stop.
				dump_dir = a[i + 1].get_slice("|", 0)
				for t in a[i + 1].get_slice("|", 1).split(","):
					dump.append(int(t))
			"--no-villages":
				villages = false
				i -= 1
			_:
				push_error("unknown argument " + a[i])
				quit(2)
				return
		i += 2
	var bad := false
	## Across every world of the run: bridge edges and gated wall runs with the switch on, and footprint over the map's
	## lake and sea with it off (the lake and sea measure's positive control).
	var bridges_all := 0
	var gated_all := 0
	var open_off_all := 0
	var ferries_all := 0
	var quartered_all := 0
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
		if slider:
			## Ruling HN: the river look's width slider never moves a town. The look's `river_width` is taken from 1 to
			## 2.5 with the switch on; every town's box, the point it is laid round, its river's width, its market and
			## its houses must be the same, while the markers (which follow the stroke the look paints) must move: the
			## positive control that the slider reached the map. The first 60 settlements are laid out.
			wg.set_params({"civ.bank_sites": true})
			var some := all.slice(0, 60)
			var snap := func() -> Array:
				var rows: Array = []
				for r in wg.town_bank_census(some, view.x, view.y):
					rows.append([r.get("origin_m", -1.0), r.get("river_m", -1.0), r.get("wm", 0.0), r.get("hm", 0.0), r.get("river_w", 0.0), r.get("mx", 0.0), r.get("my", 0.0), r.get("pop", 0.0), r.get("of_b", 0), r.get("of_s", 0)])
				return [str(wg.urban_layout_boxes()), str(rows)]
			var marks0: PackedVector2Array = wg.settlement_bank_offsets(9.6, -1.0)
			var s0: Array = snap.call()
			var took := int(wg.set_appearance({"river_width": 2.5}))
			var marks1: PackedVector2Array = wg.settlement_bank_offsets(9.6, -1.0)
			var s1: Array = snap.call()
			var marks_moved := 0
			for k in mini(marks0.size(), marks1.size()):
				marks_moved += int(not marks0[k].is_equal_approx(marks1[k]))
			var moved_towns := (s0[1] as String).count("-1.0, -1.0")
			print("TOWN SLIDER seed %d: the look took the width (%d value set); markers moved %d of %d; town boxes %s; the %d laid towns %s" % [seed, took, marks_moved, marks0.size(),
				"the same" if s0[0] == s1[0] else "MOVED", some.size(), "the same" if s0[1] == s1[1] else "MOVED"])
			if took < 1 or marks_moved == 0:
				print("TOWN FAIL: the slider did not reach the map (no marker moved): the control failed")
				bad = true
			if s0[0] != s1[0] or s0[1] != s1[1]:
				print("TOWN FAIL: the river look's width moved a town")
				bad = true
			if (s0[0] as String).length() < 10 or moved_towns == some.size():
				print("TOWN FAIL: no town is laid round a bank point: nothing to hold still")
				bad = true
			wg = null
			continue
		if dump_dir != "":
			for on in [false, true]:
				wg.set_params({"civ.bank_sites": on})
				for k in dump:
					var f := FileAccess.open("%s/town_%d_%d_%s.json" % [dump_dir, seed, k, "on" if on else "off"], FileAccess.WRITE)
					f.store_string(wg.town_debug_json(k, 6.0))
					f.close()
					print("TOWN dumped #%d %s" % [k, "on" if on else "off"])
			wg = null
			continue
		if time_only:
			## Ruling HN leftover (f): what the town's point costs per zoom step (`settlement_bank_offsets`, one call a
			## step) and per `urban_layout_boxes` call, switch on, 15 calls each after one warm-up that builds the
			## stroke index. Run alone: a timing under another run is not a measurement.
			wg.set_params({"civ.bank_sites": true})
			## Cold: the first call for a world state lays out no town but builds every settlement's layout context and
			## market (`market_points`). Each sample flips `urban_wall_hug`, which is in the cache's key, so the whole
			## table is built again (five samples, both values).
			var cold: Array = []
			var hug: bool = wg.get_params()["urban_wall_hug"]
			for rep in 5:
				wg.set_params({"urban_wall_hug": (rep % 2 == 0) != hug})
				var t0 := Time.get_ticks_usec()
				wg.settlement_bank_offsets(2.4, -1.0)
				cold.append((Time.get_ticks_usec() - t0) / 1000.0)
			wg.set_params({"urban_wall_hug": hug})
			cold.sort()
			print("TOWN TIME settlement_bank_offsets COLD (a new world state), %d settlements: median %.0f ms (min %.0f .. max %.0f), 5 calls" % [places.size(), cold[2], cold[0], cold[-1]])
			wg.settlement_bank_offsets(2.4, -1.0)
			for what in ["settlement_bank_offsets", "urban_layout_boxes"]:
				var ms: Array = []
				for rep in 15:
					var t := Time.get_ticks_usec()
					if what == "urban_layout_boxes":
						wg.urban_layout_boxes()
					else:
						## Another density each call, as a zoom step asks.
						wg.settlement_bank_offsets(2.4 * pow(1.25, rep + 1), -1.0)
					ms.append((Time.get_ticks_usec() - t) / 1000.0)
				ms.sort()
				print("TOWN TIME %s, %d settlements: median %.2f ms (min %.2f .. max %.2f), 15 calls" % [what, places.size(), ms[7], ms[0], ms[-1]])
			wg = null
			continue
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
		var off_mkt := {}
		var off_target := {}
		var off_in_river := 0
		var off_p10 := 0.0
		var off_open := 0
		for on in [false, true]:
			var n := 0
			var lost := 0
			var in_river := 0
			var wet := 0
			var fronted := 0
			var reach := 0
			## River: buildings, ungated wall runs and non-bridge street edges over a drawn stroke; `open`: the same over the
			## map's drawn lake and sea, of EVERY laid town (a lake or sea town need not be a river town).
			var over := {"b": 0, "w": 0, "s": 0}
			var over_of := {"b": 0, "w": 0, "s": 0}
			var open := {"lake_b": 0, "lake_w": 0, "lake_s": 0, "sea_b": 0, "sea_w": 0, "sea_s": 0}
			var gated := 0
			var bridge := 0
			var span := 0.0
			var wet_span := 0.0
			## Ruling HQ: ferries (a crossing longer than the bridge limit), the longest; towns that kept no street round
			## their market where the switch off had one (HN's survivor U18); towns housing over 125 % of their target.
			var ferries := 0
			var ferry_towns := 0
			var ferry_m := 0.0
			var mkt_lost := 0
			var over_housed := 0
			## Ruling HQ (HN's survivor U16): river towns housing under a quarter of what they housed with the switch off
			## (of at least 20 %), across the run, against `--max-quartered` (a ratchet: Ruling HN's build has 6 on the
			## 384-cell worlds, 2 at 2 048 and 0 on the owner's world, the leftovers HN listed).
			var quartered := 0
			var housed: Array = []
			var housed_all: Array = []
			var sides := {"x1": [0, 0], "x4": [0, 0], "x16": [0, 0], "x64": [0, 0]}
			var moved := 0
			var far := 0
			var grown := 0
			var target_moved := 0
			var origin_m: Array = []
			for r in state[on]:
				var k := int(r["i"])
				var note := ""
				if r.get("laid", false):
					housed_all.append(100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])))
					for kk in open:
						open[kk] += int(r[kk.get_slice("_", 0) + "_over_" + kk.get_slice("_", 1)])
					var lk := int(r["lake_over_b"]) + int(r["lake_over_w"]) + int(r["lake_over_s"])
					var se := int(r["sea_over_b"]) + int(r["sea_over_w"]) + int(r["sea_over_s"])
					if lk > 0:
						note += " LAKE b %d w %d s %d (%s)" % [r["lake_over_b"], r["lake_over_w"], r["lake_over_s"], r["lake_s_by"]]
					if se > 0:
						note += " SEA b %d w %d s %d (%s)" % [r["sea_over_b"], r["sea_over_w"], r["sea_over_s"], r["sea_s_by"]]
					## The denominator of "housed" must be the same town's in both states.
					if not on:
						off_target[k] = float(r["pop_target"])
					elif off_target.has(k) and absf(off_target[k] - float(r["pop_target"])) > 0.5:
						target_moved += 1
						note += " TARGET %.0f from %.0f" % [r["pop_target"], off_target[k]]
				var is_river := river_towns.has(k)
				if is_river:
					n += 1
					if not r.get("river", false):
						lost += 1
						note += " NO RIVER"
					## Only on a river site: a coast or bay layout's "river" is its shoreline (`UrbanLayout::river`).
					if r.has("market_river_m") and String(r.get("kind", "")).begins_with("river") and float(r["market_river_m"]) < float(r["river_w"]) * 0.5:
						in_river += 1
						note += " MARKET IN RIVER (%.0f m of %.0f)" % [r["market_river_m"], r["river_w"]]
					if r.get("laid", false):
						wet += int(r["wet"])
						fronted += int(r["fronted"])
						reach += int(r["reach"])
						for kk in ["b", "w", "s"]:
							over[kk] += int(r["over_" + kk])
							over_of[kk] += int(r["of_" + kk])
						gated += int(r["gated_w"])
						bridge += int(r["bridge_s"])
						span = maxf(span, float(r["span_m"]))
						wet_span = maxf(wet_span, float(r.get("wet_span_m", 0.0)))
						ferries += int(r.get("ferries", 0))
						ferry_towns += int(int(r.get("ferries", 0)) > 0)
						ferry_m = maxf(ferry_m, float(r.get("ferry_m", 0.0)))
						if int(r.get("ferries", 0)) > 0:
							note += " FERRY %d (%.0f m)" % [r["ferries"], r["ferry_m"]]
						if not on:
							off_mkt[k] = int(r.get("mkt_s", 0))
						## Walled towns only: the miss this gate guards is the rampart sweep's (HN's U18), which acts on a wall's
						## ring; nine unwalled hamlets on the 384-cell worlds lay no street at all in either build.
						elif int(r.get("mkt_s", 0)) == 0 and off_mkt.get(k, 0) > 0 and int(r.get("of_w", 0)) > 0:
							mkt_lost += 1
							note += " NO STREET NEAR THE MARKET (off %d)" % off_mkt[k]
						housed.append(100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])))
						over_housed += int(housed[-1] > 125.0)
						if int(r["wet"]) > 0:
							note += " WET %d" % r["wet"]
						if int(r["over_b"]) + int(r["over_w"]) + int(r["over_s"]) > 0:
							note += " OVER b %d w %d s %d (%s; deepest %s)" % [r["over_b"], r["over_w"], r["over_s"], r["s_by"], r["depth"]]
						if on and off_housed.get(k, 0.0) >= 20.0 and housed[-1] < 0.25 * off_housed.get(k, 0.0):
							quartered += 1
						if on and housed[-1] < 0.8 * off_housed.get(k, 0.0):
							note += " HOUSED %.0f%% from %.0f%%" % [housed[-1], off_housed[k]]
						if not on:
							off_housed[k] = housed[-1]
					for z in sides:
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
				## Ruling HQ: `--cell-min-housed P` fails the run when the `--cell` town houses under P % with the switch on.
				if on and cell_min >= 0.0 and Vector2i(p["x"], p["y"]) == cell and r.get("laid", false) and 100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])) < cell_min:
					print("TOWN FAIL: the town at %s houses %.0f %% with the switch on, under %.0f %%" % [str(cell), 100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])), cell_min])
					bad = true
				if (show_rows and note != "") or Vector2i(p["x"], p["y"]) == cell:
					print("TOWN     %s #%d %s (%d,%d) pop %d:%s | box %.0fx%.0f river_w %.0f market-river %s market %s at (%.2f, %.2f) cells origin %s river_m %s over b %s/%s w %s s %s gated %s bridge %s wet %s fronted %s/%s housed %.0f%% of %.0f" % ["on " if on else "off", k, p["kind"], p["x"], p["y"], int(p.get("pop", 0)), note,
						r.get("wm", 0.0), r.get("hm", 0.0), r.get("river_w", 0.0), ("%.0f" % r["market_river_m"]) if r.has("market_river_m") else "-",
						String(r.get("market_prov", "-")).left(24), r.get("mx", 0.0), r.get("my", 0.0),
						("%.0f" % r["origin_m"]) if r.has("origin_m") else "-", ("%.0f" % r["river_m"]) if r.has("river_m") else "-",
						r.get("over_b", "-"), r.get("of_b", "-"), r.get("over_w", "-"), r.get("over_s", "-"), r.get("gated_w", "-"), r.get("bridge_s", "-"), r.get("wet", "-"), r.get("fronted", "-"), r.get("reach", "-"),
						100.0 * float(r.get("pop", 0.0)) / maxf(1.0, float(r.get("pop_target", 1.0))), r.get("pop_target", 0.0)])
			origin_m.sort()
			## The eight least-housed river towns, so a moved p10 can be read to its towns: index, housed, and with the
			## switch on what the same town housed with it off.
			var low: Array = []
			for r in state[on]:
				if river_towns.has(int(r["i"])) and r.get("laid", false):
					low.append([100.0 * float(r["pop"]) / maxf(1.0, float(r["pop_target"])), int(r["i"])])
			low.sort()
			var low_txt := ""
			for e in low.slice(0, 8):
				low_txt += " #%d %.0f%%%s" % [e[1], e[0], (" (off %.0f%%)" % off_housed.get(e[1], -1.0)) if on else ""]
			print("TOWN   %s least housed:%s" % ["on " if on else "off", low_txt])
			var p10 := _p10(housed)
			var open_all := 0
			for kk in open:
				open_all += open[kk]
			print("TOWN   %s: river towns %d | lost river %d | market in its river %d | buildings in water %d | fronted %d/%d (%.1f %%) | housed p10 %.0f %% (all towns %.0f %%) | over drawn river: buildings %d of %d, wall runs %d (gated %d), street edges %d of %d (bridge edges %d, longest bridge %.0f m, over the water %.0f m) | over drawn lake: b %d w %d s %d | over drawn sea: b %d w %d s %d | other bank x1 %d/%d x4 %d/%d x16 %d/%d x64 %d/%d | moved %d (median %.0f m, max %.0f m), past the 625 m reach %d, box grown %d | ferries %d in %d towns (longest %.0f m) | no street near the market %d | over-housed (over 125 %%) %d | housed under a quarter of off %d" % [
				"on " if on else "off", n, lost, in_river, wet, fronted, reach, 100.0 * fronted / maxf(1.0, reach), p10, _p10(housed_all), over["b"], over_of["b"], over["w"], gated, over["s"], over_of["s"], bridge, span, wet_span,
				open["lake_b"], open["lake_w"], open["lake_s"], open["sea_b"], open["sea_w"], open["sea_s"],
				sides["x1"][1], sides["x1"][0], sides["x4"][1], sides["x4"][0], sides["x16"][1], sides["x16"][0], sides["x64"][1], sides["x64"][0], moved,
				origin_m[origin_m.size() / 2] if not origin_m.is_empty() else 0.0, origin_m[-1] if not origin_m.is_empty() else 0.0, far, grown,
				ferries, ferry_towns, ferry_m, mkt_lost, over_housed, quartered])
			if on:
				var other := 0
				for z in sides:
					other += sides[z][1]
				if lost > 0 or in_river > off_in_river or wet > 0 or over["b"] + over["w"] + over["s"] > 0 or open_all > 0 or other > 0 or p10 < off_p10 or target_moved > 0 or mkt_lost > 0:
					print("TOWN FAIL: a bar is missed with the switch on (lost %d, market in river %d against %d off, wet %d, over the river b %d w %d s %d, over lake or sea %d, other bank %d, housed p10 %.1f against %.1f off, targets moved %d, no street near the market %d)" % [lost, in_river, off_in_river, wet, over["b"], over["w"], over["s"], open_all, other, p10, off_p10, target_moved, mkt_lost])
					bad = true
				if moved == 0:
					print("TOWN FAIL: no town moved: the switch reached nothing")
					bad = true
				## Positive controls: the exemptions are not vacuous (a bridge exists), and the water measures can see water
				## (with the switch off the towns stand over the drawn river).
				## Ruling HQ: a world whose every crossing is longer than the bridge limit has ferries and no bridge.
				if bridge == 0 and ferries == 0:
					print("TOWN FAIL: no bridge edge and no ferry counted: the crossing rules were not exercised")
					bad = true
				bridges_all += bridge
				ferries_all += ferries
				quartered_all += quartered
				gated_all += gated
				open_off_all += off_open
			else:
				off_in_river = in_river
				off_p10 = p10
				off_open = open_all
				if over["b"] == 0:
					print("TOWN FAIL: no building over a drawn river with the switch off: the measure sees nothing")
					bad = true
				if moved != 0:
					print("TOWN FAIL: a town moved with the switch off")
					bad = true
		wg = null
	print("\nTOWN run: bridge edges %d, ferries %d, gated wall runs %d (switch on); footprint over the map's lake or sea with the switch off %d" % [bridges_all, ferries_all, gated_all, open_off_all])
	if max_quartered >= 0 and quartered_all > max_quartered:
		print("TOWN FAIL: %d river towns house under a quarter of what they housed with the switch off, over the %d allowed" % [quartered_all, max_quartered])
		bad = true
	quit(1 if bad else 0)
