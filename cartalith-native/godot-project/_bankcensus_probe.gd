extends SceneTree
## Ruling HG census: how many settlement markers lie inside a drawn river, by class and by the path that placed them,
## and what `WorldGen.settlement_bank_offsets`' rule (`bank_site::bank_point`) leaves.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_bankcensus_probe.gd -- \
##       [--seeds a,b,c] [--gw N] [--gh N] [--view WxH] [--gap PX] [--layouts] [--rows] [--no-recovery]
##
## Headless: it reads engine state only (no pixels). Run from a project copy with its own `config/name`.
## `--view` is the map view's size in pixels (the stroke's zoom law reads the window; the default is the desktop
## shell's map view in a 1600 x 900 window, read from `ViewportHost` by `_bankframes_probe.gd`). x1 is the opening
## fit in that view, x4 four times it. `--gap` is the marker's gap from the drawn edge in pixels (default the app's,
## `bank_site::MARKER_GAP_PX`). `--layouts` lays out every settlement inside a stroke at x4 and reports which bank
## its market stands on. `--rows` prints a row per inside settlement.
##
## The placement paths: "generated" (the placement cascade, by class), "junction hamlet" (the positions the same world
## founds with villages off), "seeded village" (the other village-flagged rows), "recovery hamlet" (settlements a
## 300-year recovery run adds), "manual, snap to water" (`WorldGen.bank_snap_census`: simulated clicks, nothing placed).
## A measurement, with three gates (exit 1): an empty census, the switch (`civ.bank_sites` on for a new world, an offset
## per settlement with it on, none with it off) and the default gap (the answer for a negative gap equals the answer for
## 2 px and differs from the answer for 0 px).

const COLS := ["n", "in_w", "in_x1", "in_x4", "cell_x1", "cell_x4", "left_x1", "left_x4"]

func _world(seed: int, gw: int, gh: int, villages: bool) -> WorldGen:
	var wg: WorldGen = WorldGen.new()
	wg.set_villages_enabled(villages)
	wg.generate_sized(seed, 800.0, gw, gh)
	return wg

func _init() -> void:
	var seeds := PackedInt32Array([483920, 912345, 246371])
	var gw := 384
	var gh := 288
	var view := Vector2(1150, 800)
	var gap := 2.0
	var layouts := false
	var show_rows := false
	var recovery := true
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
			"--gap": gap = float(a[i + 1])
			"--view":
				var p := a[i + 1].split("x")
				view = Vector2(float(p[0]), float(p[1]))
			"--layouts":
				layouts = true
				i -= 1
			"--rows":
				show_rows = true
				i -= 1
			"--no-recovery":
				recovery = false
				i -= 1
			_:
				push_error("unknown argument " + a[i])
				quit(2)
				return
		i += 2
	var bad := false
	var fit := minf(view.x / gw, view.y / gh)
	for seed in seeds:
		## The junction hamlets: the village-flagged places of the same world with the village pass off.
		var jw := _world(seed, gw, gh, false)
		var junction := {}
		for s in jw.get_settlements():
			if String(s["kind"]) == "hamlet" or String(s["kind"]) == "village":
				junction[Vector2i(s["x"], s["y"])] = true
		jw = null
		var wg := _world(seed, gw, gh, true)
		var places: Array = wg.get_settlements()
		var n_gen := places.size()
		var rec_note := "not run"
		if recovery:
			var res: Dictionary = wg.civ_run_collapse_simulation({"mode": "recovery", "start_year": 0, "duration": 300, "step_years": 10, "confirm_overwrite": true})
			rec_note = "ok %s, hamlets %s" % [res.get("ok", false), res.get("hamlets", "-")]
			places = wg.get_settlements()
		var rows: Array = wg.bank_census(view.x, view.y, gap)
		## The switch and the default gap, as literals (a mutant of either turns this red).
		var on_sw: bool = wg.get_params().get("civ.bank_sites", false)
		var offs: PackedVector2Array = wg.settlement_bank_offsets(fit * 4.0, -1.0)
		var same_as_2px: bool = offs == wg.settlement_bank_offsets(fit * 4.0, 2.0)
		var differs_from_0px: bool = offs != wg.settlement_bank_offsets(fit * 4.0, 0.0)
		wg.set_params({"civ.bank_sites": false})
		var off_empty: bool = wg.settlement_bank_offsets(fit * 4.0, -1.0).is_empty()
		wg.set_params({"civ.bank_sites": true})
		if not (on_sw and offs.size() == places.size() and same_as_2px and differs_from_0px and off_empty):
			print("BANK FAIL: the switch (on for a new world %s, an offset per settlement %s, none when off %s) or the default gap (equals 2 px %s, differs from 0 px %s)" % [on_sw, offs.size() == places.size(), off_empty, same_as_2px, differs_from_0px])
			bad = true
		else:
			print("BANK   switch on for a new world, an offset per settlement, none with it off; the default gap is 2 px: ok")
		print("\nBANK seed %d at %dx%d, view %dx%d (x1 = %.3f px per cell), gap %.1f px: %d settlements (%d from generation; recovery run: %s)" % [
			seed, gw, gh, int(view.x), int(view.y), fit, gap, places.size(), n_gen, rec_note])
		if places.is_empty():
			print("BANK FAIL: no settlements")
			bad = true
			continue
		## Every settlement's path, from its census row (one per settlement).
		var by := {}
		var shift := {"x1": 0.0, "x4": 0.0}
		for r in rows:
			var k := int(r["i"])
			var key := "generated " + String(r["kind"])
			if k >= n_gen:
				key = "recovery hamlet"
			elif r["village"]:
				key = "junction hamlet" if junction.has(Vector2i(r["x"], r["y"])) else "seeded village"
			if not by.has(key):
				by[key] = {"n": 0, "in_w": 0, "in_x1": 0, "in_x4": 0, "cell_x1": 0, "cell_x4": 0, "left_x1": 0, "left_x4": 0}
			var c: Dictionary = by[key]
			c["n"] += 1
			c["in_w"] += int(r["in_w"])
			for z in ["x1", "x4"]:
				c["in_" + z] += int(r["in_" + z])
				c["cell_" + z] += int(r["in_" + z] and r["cell_" + z])
				c["left_" + z] += int(r["left_" + z])
				if r.has("shift_" + z):
					shift[z] = maxf(shift[z], float(r["shift_" + z]))
		print("BANK   %-26s %5s %8s %6s %6s | whole-cell move works x1 x4 | still inside after x1 x4" % ["path and class", "n", "in_width", "in_x1", "in_x4"])
		var tot := {}
		for cn in COLS:
			tot[cn] = 0
		var keys := by.keys()
		keys.sort()
		for key in keys:
			var c: Dictionary = by[key]
			print("BANK   %-26s %5d %8d %6d %6d | %14d %2d | %18d %2d" % [key, c["n"], c["in_w"], c["in_x1"], c["in_x4"], c["cell_x1"], c["cell_x4"], c["left_x1"], c["left_x4"]])
			for cn in COLS:
				tot[cn] += c[cn]
		print("BANK   %-26s %5d %8d %6d %6d | %14d %2d | %18d %2d" % ["ALL", tot["n"], tot["in_w"], tot["in_x1"], tot["in_x4"], tot["cell_x1"], tot["cell_x4"], tot["left_x1"], tot["left_x4"]])
		print("BANK   largest marker shift: x1 %.3f cells (%.1f px), x4 %.3f cells (%.1f px)" % [shift["x1"], shift["x1"] * fit, shift["x4"], shift["x4"] * fit * 4.0])
		var snap: Dictionary = wg.bank_snap_census(view.x, view.y, gap, 3)
		print("BANK   manual, snap to water: %d simulated clicks; land inside a stroke x1 %d, x4 %d; still inside after x1 %d, x4 %d" % [snap["clicks"], snap["in_x1"], snap["in_x4"], snap["left_x1"], snap["left_x4"]])
		if tot["in_x4"] == 0 and tot["in_x1"] == 0:
			print("BANK FAIL: no settlement inside any stroke: the census has no positive control on this world")
			bad = true
		if show_rows or layouts:
			var gpm := float(gw) / 800000.0
			var offs4: PackedVector2Array = wg.settlement_bank_offsets(fit * 4.0, gap)
			var side := {}
			var on_line := {}
			for r in rows:
				if not (r["in_w"] or r["in_x1"] or r["in_x4"]):
					continue
				var line := "BANK     #%d %s (%d,%d) d %.3f w %.3f rank %.2f in w/x1/x4 %d%d%d shift x1 %s x4 %s" % [r["i"], r["kind"], r["x"], r["y"], r["d"], r["w"], r["rank"],
					int(r["in_w"]), int(r["in_x1"]), int(r["in_x4"]), ("%.3f" % r["shift_x1"]) if r.has("shift_x1") else "-", ("%.3f" % r["shift_x4"]) if r.has("shift_x4") else "-"]
				if layouts and r["in_x4"]:
					var ls: Array = wg.urban_layouts(PackedInt32Array([r["i"]]))
					if ls.size() == 1:
						var l: Dictionary = ls[0]
						## From the foot of the centre on the drawn centreline: the layout's market, and the marker's bank
						## point at x4. On one bank when the two point the same way from the line.
						var foot_to_centre := Vector2(r["nx"], r["ny"]) * float(r["d"])
						var m: Vector2 = (l["market"] - l.get("pin", l["market"])) * gpm + foot_to_centre
						var b: Vector2 = offs4[r["i"]] + foot_to_centre
						var m_m := m.length() / gpm
						var verdict := "market on the line" if m_m < 1.0 else ("SAME bank" if m.dot(b) > 0.0 else "OTHER bank")
						line += " | market %.0f m off the line: %s" % [m_m, verdict]
						side[verdict] = int(side.get(verdict, 0)) + 1
						## For a centre on the centreline (`nx`, `ny` is then the segment's left normal, looking downstream):
						## which hand of the river the market took, so a tie-break can be judged against it.
						if float(r["d"]) < 0.0005 and m_m >= 1.0:
							var hand := "left of the flow" if m.dot(Vector2(r["nx"], r["ny"])) > 0.0 else "right of the flow"
							var compass := "east or south" if m.dot(Vector2(1.0, 0.13)) > 0.0 else "west or north"
							on_line[hand] = int(on_line.get(hand, 0)) + 1
							on_line[compass] = int(on_line.get(compass, 0)) + 1
				print(line)
			if layouts:
				print("BANK   the market against the marker's bank point at x4 (settlements inside a stroke at x4): %s" % str(side))
				print("BANK   centres on the centreline whose market left it: %s" % str(on_line))
		wg = null
	quit(1 if bad else 0)
