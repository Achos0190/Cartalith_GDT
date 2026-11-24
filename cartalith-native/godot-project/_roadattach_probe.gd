extends SceneTree
## Owner report 2026-11-23 (*"the attachment from routes (sea and land) to the settlements when you zoom in"*): how far
## each land road's drawn end is from the town it serves, read from the engine's own getters (`get_roads`,
## `get_settlements`, `settlement_bank_offsets`, `urban_layouts`), so the same script runs against an older build's dll
## (every call is guarded with `has_method`).
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_roadattach_probe.gd -- ##       [--seeds a,b] [--gw N] [--gh N] [--view WxH] [--params FILE] [--rows] [--gate-px N] [--no-villages] [--mutate KIND]
##
## Headless: engine state only, no pixels. For every road whose end the town reads as its own (the adapter's end test,
## `urban_adapter::um_primary_paths_reach`: an end within max(1, gw / 250) cells of the settlement's cell), with
## `civ.bank_sites` off (the control) and on:
## - `street`: metres from the road's drawn end to the nearest drawn street of its town (every class, on lines);
## - `market`: metres from the road's drawn end to the town's market on the map;
## - `marker xZ`: pixels from the road's drawn end to the marker drawn at map zoom Z (x1 is the fit of `--view`);
## - `along`: the median distance, metres, of the road's drawn points inside the town's box from the town's own
##   primary streets (the town's copy of that road): how far the drawn road runs beside the street it should be.
## Gate (exit 1) with the switch on: a crossing of a town's river the attach made (`made`); a median `along` over
## ALONG_MAX_M; any road end farther than `--gate-px` (default 4: under half the smallest pin head, `map_overlay.gd`'s `PIN_HEAD_MIN_PX` 9, a labelled
## judgement) from its town's nearest street at x16 or x64 (the town layer's zooms), or from its marker at x1 or x4 (only the marker is drawn there). Positive control
## (exit 1): no road end at a moved town, or no road end over the gate with the switch off (the measure must see the
## miss it gates). `--mutate end`: the probe reads each road's other end (must go red).
## The run's frame: a road's points are grid coordinates as `map_overlay.gd` draws them (`_point_to_screen`, no half
## cell); a marker is its cell centre plus the engine's offset; a town point is cell centre + R(orient)(p - pin) / m.

const ZOOMS := [1.0, 4.0, 16.0, 64.0]
## A wall ring with fewer buildings than this inside it counts as empty (the owner's "walls without anything inside
## them"). A labelled judgement: the smallest walled town houses several hundred people in dozens of houses.
const WALL_MIN_INSIDE := 5
## How near a town's own bridge point or ferry a drawn road's crossing of the town's river must be to be that crossing,
## metres. A labelled judgement: a bridge point is where a street meets the centreline, and the drawn road follows the
## town's copy of itself to a few metres (its `along`), well inside this.
const CROSS_NEAR_M := 60.0
## Gate: the median distance, metres, of a drawn road's points inside its town's box from the town's own copy of it
## (its primary streets). A labelled judgement: the copy is the same line, so only the layout's own smoothing and
## pruning part them (15 and 16 m measured at 384 and 4 096 cells with the fix); the half-cell frame alone was 95 m at
## 4 096 and 400 m at 384.
const ALONG_MAX_M := 50.0

func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var ab := b - a
	var t := 0.0 if ab.length_squared() < 1e-12 else clampf((p - a).dot(ab) / ab.length_squared(), 0.0, 1.0)
	return p.distance_to(a + ab * t)

func _lines_dist(p: Vector2, segs: PackedVector2Array) -> float:
	var best := INF
	var k := 0
	while k + 1 < segs.size():
		best = minf(best, _seg_dist(p, segs[k], segs[k + 1]))
		k += 2
	return best

## The index of the settlement whose cell is nearest `p` (the way's frame: the integer cell).
func _nearest(places: Array, p: Vector2) -> int:
	var best := -1
	var bd := INF
	for k in places.size():
		var d := p.distance_to(Vector2(float(places[k]["x"]), float(places[k]["y"])))
		if d < bd:
			bd = d
			best = k
	return best

## `[at a town crossing, with none]`: how often polyline `pts` crosses the town's river line `river` inside the box
## (`c` +- `half`), a crossing being the town's when a bridge point of `crossings` or a ferry segment (pairs in
## `ferries`) is within CROSS_NEAR_M of it.
func _crossings(wg, pts: PackedVector2Array, river: PackedVector2Array, crossings: PackedVector2Array, ferries: PackedVector2Array, c: Vector2, half: Vector2, mpc: float) -> Array:
	var ok_n := 0
	var bad_n := 0
	var engine: bool = wg.has_method("drawn_river_crosses")
	for k in pts.size() - 1:
		if absf(pts[k].x - c.x) > half.x or absf(pts[k].y - c.y) > half.y:
			continue
		var hits: Array = []
		if engine:
			## The engine's own test (`WorldGen.drawn_river_crosses`, what `bank_site::finish_end` reads), the crossing taken
			## at the segment's middle (a drawn segment is a quarter cell).
			if wg.drawn_river_crosses(pts[k], pts[k + 1]):
				hits.append((pts[k] + pts[k + 1]) * 0.5)
		else:
			var j := 0
			while j + 1 < river.size():
				var y = _cross(pts[k], pts[k + 1], river[j], river[j + 1])
				j += 1
				if y != null:
					hits.append(y)
		for x in hits:
			var ok := false
			for q in crossings:
				ok = ok or (x as Vector2).distance_to(q) * mpc < CROSS_NEAR_M
			var f := 0
			while f + 1 < ferries.size():
				ok = ok or _seg_dist(x, ferries[f], ferries[f + 1]) * mpc < CROSS_NEAR_M
				f += 2
			if ok:
				ok_n += 1
			else:
				bad_n += 1
	return [ok_n, bad_n]

## How many drawn river centrelines polyline `pts` crosses, segment by segment (`WorldGen.drawn_river_crosses`).
func _all_crossings(wg, pts: PackedVector2Array) -> int:
	var n := 0
	for k in pts.size() - 1:
		n += int(wg.drawn_river_crosses(pts[k], pts[k + 1]))
	return n

## Where segment a-b crosses segment c-d, or `null`.
func _cross(a: Vector2, b: Vector2, c: Vector2, d: Vector2):
	return Geometry2D.segment_intersects_segment(a, b, c, d)

func _median(v: Array) -> float:
	if v.is_empty():
		return NAN
	v.sort()
	return v[v.size() / 2]

func _init() -> void:
	var seeds := PackedInt32Array([483920])
	var gw := 384
	var gh := 288
	var view := Vector2(1150, 800)
	var params_file := ""
	var rows := false
	var gate_px := 4.0
	var villages := true
	var mutate := ""
	## `--trace N`: print settlement N's roads (their own points and as drawn), its river and crossings near it, cells
	## from its cell (a diagnosis aid; no gate reads it).
	var trace := -1
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
			"--gate-px": gate_px = float(a[i + 1])
			"--mutate": mutate = a[i + 1]
			"--trace": trace = int(a[i + 1])
			"--rows":
				rows = true
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
		if wg.has_method("set_villages_enabled"):
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
				match typeof(known[k]):
					TYPE_BOOL: set[k] = v == "true"
					TYPE_INT: set[k] = int(v)
					TYPE_FLOAT: set[k] = float(v)
					_: set[k] = v
			wg.set_params(set)
		wg.generate_sized(seed, 800.0, gw, gh)
		var places: Array = wg.get_settlements()
		var mpc := 800000.0 / gw
		var fit := minf(view.x / gw, view.y / gh)
		## The ends a town lays its own copy of: the adapter's end test, and inside the reference box's half long side
		## (850 m), from which the town lays the road (`bank_site::attach_full_cells`). Farther ends end at something
		## else and are counted apart (`near`).
		var full := 850.0 / mpc
		var eps_far := maxf(1.0, gw / 250.0)
		var eps := minf(eps_far, full)
		var all := PackedInt32Array()
		for k in places.size():
			all.append(k)
		print("\nROAD seed %d at %dx%d, %d settlements, %.0f m a cell, fit %.3f px a cell, end test %.1f cells" % [seed, gw, gh, places.size(), mpc, fit, eps])
		var roads_off: Array = []
		var moved_ends_on := 0
		var fails_off := 0
		for on in [false, true]:
			if wg.get_params().has("civ.bank_sites"):
				wg.set_params({"civ.bank_sites": on})
			elif on:
				print("ROAD   this build has no civ.bank_sites: the on state is the off state")
			var roads: Array = wg.get_roads()
			## Which end of which road is a town's is read from the way's own points (the switch-off state, which draws
			## them as they are), so a drawn end the fix moved is still that town's.
			if not on:
				roads_off = roads
			var marks: Array = []
			for z in ZOOMS:
				marks.append(wg.settlement_bank_offsets(fit * z, -1.0) if wg.has_method("settlement_bank_offsets") else PackedVector2Array())
			var lay := {}
			for l in wg.urban_layouts(all):
				lay[int(l["index"])] = l
			var gaps_street: Array = []
			var gaps_market: Array = []
			var gaps_px: Array = [[], [], [], []]
			var alongs: Array = []
			var n_ends := 0
			var n_moved := 0
			var n_fail := 0
			var through := 0
			var near := 0
			var cross_ok := 0
			var cross_bad := 0
			var cross_made := 0
			var n_keyed := 0
			var bare_market := 0
			var unkeyed := 0
			var gaps_deep: Array = [[], []]
			## Walled towns and how many of them have fewer than WALL_MIN_INSIDE buildings inside the ring.
			var walled := 0
			var empty_rings := 0
			for si in places.size():
				var s: Dictionary = places[si]
				var cell := Vector2(float(s["x"]), float(s["y"]))
				var c := cell + Vector2(0.5, 0.5)
				var l: Dictionary = lay.get(si, {})
				if l.is_empty():
					continue
				if l.get("walls", false) and l.has("wall_ring"):
					walled += 1
					var inside := 0
					for f in (l.get("building_intramural", PackedByteArray()) as PackedByteArray):
						inside += int(f != 0)
					if inside < WALL_MIN_INSIDE:
						empty_rings += 1
						print("ROAD     %s #%d EMPTY RING: %d buildings inside its wall" % ["on " if on else "off", si, inside])
				var pin: Vector2 = l["pin"]
				var orient := float(l.get("orient", 0.0))
				var to_grid := func(p: Vector2) -> Vector2: return c + (p - pin).rotated(orient) / mpc
				var streets := PackedVector2Array()
				var prim := PackedVector2Array()
				for cls in (l["streets"] as Dictionary):
					for p in (l["streets"][cls] as PackedVector2Array):
						streets.append(to_grid.call(p))
						if cls == "primary":
							prim.append(to_grid.call(p))
				var market: Vector2 = to_grid.call(l["market"])
				## The town's river (the map's drawn river since Ruling HN), its crossings and its ferries, on the map.
				var river := PackedVector2Array()
				for p in (l.get("river", PackedVector2Array()) as PackedVector2Array):
					river.append(to_grid.call(p))
				var crossings := PackedVector2Array()
				for p in (l.get("bridges", PackedVector2Array()) as PackedVector2Array):
					crossings.append(to_grid.call(p))
				var ferries := PackedVector2Array()
				var dk: Array = l.get("detail_kind", [])
				var dg: Array = l.get("detail_geom", [])
				for k in mini(dk.size(), dg.size()):
					if String(dk[k]) == "ferry" and (dg[k] as PackedVector2Array).size() == 2:
						ferries.append(to_grid.call(dg[k][0]))
						ferries.append(to_grid.call(dg[k][1]))
				var moved := market.distance_to(c) * mpc > 30.0
				var half := Vector2(float(l["wm"]), float(l["hm"])) * 0.5 / mpc
				for wi in roads.size():
					var w: Dictionary = roads[wi]
					if w.get("manual", false):
						continue
					var pts: PackedVector2Array = w["points"]
					var own: PackedVector2Array = roads_off[wi]["points"] if wi < roads_off.size() else pts
					if pts.size() < 2 or own.size() < 2:
						continue
					var e := -1
					## The end is this town's only if this is the nearest settlement to it (`WorldGen::route_attached`).
					if own[0].distance_to(cell) < eps and _nearest(places, own[0]) == si:
						e = 0
					elif own[own.size() - 1].distance_to(cell) < eps and _nearest(places, own[own.size() - 1]) == si:
						e = pts.size() - 1
					if e < 0 and (own[0].distance_to(cell) < eps_far or own[own.size() - 1].distance_to(cell) < eps_far):
						near += 1
						continue
					if e < 0:
						for p in own:
							if p.distance_to(c) < 0.5:
								through += 1
								break
						continue
					if mutate == "end":
						e = pts.size() - 1 - e
					var end := pts[e]
					n_ends += 1
					n_moved += int(moved)
					var g_street := _lines_dist(end, streets) * mpc
					var g_market := end.distance_to(market) * mpc
					gaps_street.append(g_street)
					gaps_market.append(g_market)
					var row := "ROAD     %s #%d %s road %d: end (%.2f, %.2f) cell (%d, %d) street %.0f m market %.0f m" % ["on " if on else "off", si, s.get("kind", ""), wi, end.x - cell.x, end.y - cell.y, int(cell.x), int(cell.y), g_street, g_market]
					var fails := []
					## The engine names the town an end meets (`town_a`/`town_b`); the map then draws that end at the marker
					## below the town layer and at the plan's market with it, unless the leg crosses the plan's river
					## (`MapOverlay._road_points_drawn`, which `tools/road_attach_drive.gd` checks in the real shell). An end
					## it does not name is drawn where the engine put it (at its own bank, for a road the move would have
					## taken over a river: `bank_site::finish_end`).
					var keyed: bool = int(w.get("town_a" if e == 0 else "town_b", -1)) == int(s.get("tid", -2))
					n_keyed += int(keyed)
					var deep_end := end
					if keyed:
						var nb := pts[1] if e == 0 else pts[pts.size() - 2]
						var j := 0
						var crossed := false
						while j + 1 < river.size():
							crossed = crossed or _cross(nb, market, river[j], river[j + 1]) != null
							j += 1
						if not crossed:
							deep_end = market
					## The town's entry: its nearest drawn street, or its market itself where the plan has no street at its
					## market (a layout finding, counted: the road can meet the town at nothing nearer).
					var g_deep := _lines_dist(deep_end, streets) * mpc
					if keyed and on and _lines_dist(market, streets) * mpc > gate_px / (fit * 64.0) * mpc:
						bare_market += 1
						row += " MARKET WITH NO STREET (%.0f m to its nearest)" % (_lines_dist(market, streets) * mpc)
						g_deep = minf(g_deep, deep_end.distance_to(market) * mpc)
					if not keyed and on:
						unkeyed += 1
					for zk in ZOOMS.size():
						var off: Vector2 = (marks[zk] as PackedVector2Array)[si] if (marks[zk] as PackedVector2Array).size() > si else Vector2.ZERO
						var px: float = 0.0 if keyed else end.distance_to(c + off) * fit * float(ZOOMS[zk])
						gaps_px[zk].append(px)
						row += " marker x%d %.1f px" % [int(ZOOMS[zk]), px]
						var street_px: float = g_deep / mpc * fit * float(ZOOMS[zk])
						if zk >= 2:
							gaps_deep[zk - 2].append(street_px)
						if (zk >= 2 and street_px > gate_px) or (zk < 2 and px > gate_px):
							fails.append("x%d" % int(ZOOMS[zk]))
					## The drawn road over the town's river inside its box: at the town's own bridge (a crossing point
					## within CROSS_NEAR_M) or ferry (its segment within CROSS_NEAR_M), else a crossing the town lacks.
					## `made`: such crossings the drawn road has and the way's own points do not (the attach made them).
					if on and si == trace:
						var fmt := func(v: PackedVector2Array) -> String:
							var t := PackedStringArray()
							for p in v:
								if p.distance_to(c) < 3.0:
									t.append("(%.2f,%.2f)" % [p.x - cell.x, p.y - cell.y])
							return " ".join(t)
						print("TRACE #%d road %d own %s" % [si, wi, fmt.call(own)])
						print("TRACE #%d road %d drawn %s" % [si, wi, fmt.call(pts)])
						for pair in [["drawn", pts], ["own", own]]:
							var v: PackedVector2Array = pair[1]
							var off := Vector2.ZERO if pair[0] == "drawn" else Vector2(0.5, 0.5)
							for q in v.size() - 1:
								if wg.has_method("drawn_river_crosses") and wg.drawn_river_crosses(v[q] + off, v[q + 1] + off):
									print("TRACE #%d road %d %s crosses at segment %d of %d: (%.3f,%.3f)-(%.3f,%.3f)" % [si, wi, pair[0], q, v.size(), v[q].x + off.x - cell.x, v[q].y + off.y - cell.y, v[q + 1].x + off.x - cell.x, v[q + 1].y + off.y - cell.y])
						print("TRACE #%d river %s | bridges %s | ferries %s | market (%.2f,%.2f)" % [si, fmt.call(river), fmt.call(crossings), fmt.call(ferries), market.x - cell.x, market.y - cell.y])
					var cr := _crossings(wg, pts, river, crossings, ferries, c, half, mpc)
					## The way's own points in the frame its town and rivers stand in (`bank_site::WorldGen::route_drawn`).
					var own_c := PackedVector2Array()
					for p in own:
						own_c.append(p + Vector2(0.5, 0.5))
					var cr_own := _crossings(wg, own_c, river, crossings, ferries, c, half, mpc)
					cross_ok += cr[0]
					cross_bad += cr[1]
					## Made by the attach: drawn river crossings along the whole road, as drawn against the way's own line
					## (`WorldGen.drawn_river_crosses`). Over the whole road, not the box: the move slides a crossing along
					## the line, across the box's edge (246371 #200 at 384 cells read one "made" that was a slide).
					var made := maxi(0, _all_crossings(wg, pts) - _all_crossings(wg, own_c)) if wg.has_method("drawn_river_crosses") else maxi(0, cr[1] - cr_own[1])
					cross_made += made
					if made > 0:
						row += " MADE %d drawn river crossing(s) the way itself does not have" % made
					var inbox := []
					for p in pts:
						if absf(p.x - c.x) < half.x and absf(p.y - c.y) < half.y and prim.size() >= 2:
							inbox.append(_lines_dist(p, prim) * mpc)
					var al := _median(inbox)
					if not is_nan(al):
						alongs.append(al)
						row += " along %.0f m" % al
					if on and not keyed:
						## Not the town's to meet: an end the engine left at its own bank (the town is across a river it has no
						## crossing of), or with no market. Counted above, never gated.
						fails.clear()
						row += " NOT NAMED (at its own bank or no market)"
					if not fails.is_empty():
						n_fail += 1
						row += " FAIL " + ",".join(fails)
					if rows and (moved or not fails.is_empty()):
						print(row)
			if on:
				moved_ends_on = n_moved
			else:
				fails_off = n_fail
			var line := "ROAD   %s: road ends at a town %d (at a moved town %d), ending near it with no copy in its box %d, through a town cell %d | street median %.0f max %.0f m | market median %.0f max %.0f m | along median %.0f m" % ["on " if on else "off", n_ends, n_moved, near, through, _median(gaps_street.duplicate()), gaps_street.max() if not gaps_street.is_empty() else NAN, _median(gaps_market.duplicate()), gaps_market.max() if not gaps_market.is_empty() else NAN, _median(alongs.duplicate())]
			for zk in ZOOMS.size():
				line += " | marker x%d max %.1f px" % [int(ZOOMS[zk]), (gaps_px[zk] as Array).max() if not (gaps_px[zk] as Array).is_empty() else NAN]
			line += " | ends the engine names %d (not named, at its own bank or with no market: %d; at a market with no street: %d)" % [n_keyed, unkeyed, bare_market]
			for zk in 2:
				line += " | drawn end to street x%d max %.1f px" % [16 if zk == 0 else 64, (gaps_deep[zk] as Array).max() if not (gaps_deep[zk] as Array).is_empty() else NAN]
			line += " | over its river in the box: at the town's crossing %d, with none %d (made by the attach %d)" % [cross_ok, cross_bad, cross_made]
			line += " | over %.1f px: %d | walled %d, rings with under %d inside %d" % [gate_px, n_fail, walled, WALL_MIN_INSIDE, empty_rings]
			print(line)
			if on and cross_made > 0:
				print("ROAD FAIL: %d drawn river crossings made by the attach (the way's own line does not cross there)" % cross_made)
				bad = true
			if on and not alongs.is_empty() and _median(alongs.duplicate()) > ALONG_MAX_M:
				print("ROAD FAIL: the drawn roads run a median %.0f m beside their towns' own streets (over %.0f m)" % [_median(alongs.duplicate()), ALONG_MAX_M])
				bad = true
			if on and n_keyed == 0:
				print("ROAD FAIL: no road end names its town with the switch on (town_a/town_b): the overlay has nothing to draw")
				bad = true
			if on and n_fail > 0:
				print("ROAD FAIL: %d road ends more than %.1f px from their town" % [n_fail, gate_px])
				bad = true
		if fails_off == 0:
			print("ROAD FAIL: no road end misses its town with the switch off: the measure sees nothing (control)")
			bad = true
		if moved_ends_on == 0:
			print("ROAD FAIL: no road ends at a moved town: the control failed")
			bad = true
		wg = null
	quit(1 if bad else 0)
