extends SceneTree
## Ruling HJ: the way tool's waypoint spacing. Measures, on real generated land, how far a routed drag's committed road
## leaves the drawn line for a grid of (spacing, corridor, pull), driving the engine's own drag API exactly as the shell
## does (`way_stroke_begin` with the shell's options, one sample per cell, `way_stroke_end`).
## Protects: Corridor and Pull are not inert at the shell's DEFAULT spacing -- the spread of the road over the corridor
## settings and over the pull settings, at the spacing `InfrastructureWorkspace.WAY_DRAG_SPACING_CELLS` hands the engine,
## is above 0.25 cell mean-deviation (a cell is the grid's finest step; below it the road is the same road).
## Default run: the shell's own spacing (`InfrastructureWorkspace.WAY_DRAG_SPACING_CELLS`, which must equal
## `DccToolBar.WAY_SPACING_DEFAULT`) must give a spread of at least 0.15 cell over corridors AND over pulls (a literal
## bar: 2.0 measured 0.007 and 0.021, 6.0 measures 0.296 and 0.372). `--table` prints 2, 4, 6 and 8 for the record.
## Run: godot --headless --path . --script _waysp_probe.gd -- [--table] [--spacing S]   (headless is fine: engine only)
## Result: a line per spacing, then `PASS|FAIL` lines, `RESULT fails=N`; exit 1 on failure.
const GW := 192
const GH := 128
var fails := 0
var wg

func _land(x: int, y: int) -> bool:
	return x >= 0 and y >= 0 and x < GW and y < GH and String(wg.sample_cell(x, y).get("water", "")) == "land"

## Distance from p to the polyline `line`.
func _dist(p: Vector2, line: PackedVector2Array) -> float:
	var best := 1e9
	for k in line.size() - 1:
		var c := Geometry2D.get_closest_point_to_segment(p, line[k], line[k + 1])
		best = minf(best, p.distance_to(c))
	return best

## A zigzag drag of ~`len` cells starting at (x0, y0): the straight least-cost route would cut its corners.
func _drag_line(x0: float, y0: float, amp: float, len: int) -> PackedVector2Array:
	var out := PackedVector2Array()
	for i in len:
		out.append(Vector2(x0 + float(i), y0 + amp * sin(float(i) / 30.0 * TAU)))
	return out

## Mean and max distance of the committed road from the drawn line, or [] when the drag was refused.
func _run(line: PackedVector2Array, spacing, corridor: float, pull: float) -> Array:
	var opts := {"way_type": "road"}
	if spacing != null:
		opts["spacing"] = float(spacing)
	opts["corridor"] = corridor
	opts["pull"] = pull
	if not wg.way_stroke_begin(opts):
		return []
	var t := 0
	for p in line:
		wg.way_stroke_sample(p.x, p.y, t)
		t += 16
	var r: Dictionary = wg.way_stroke_end()
	if not bool(r.get("committed", false)):
		return []
	var idx := int(r["first"])
	var w: Dictionary = wg.way_get("manual", idx)
	var pts: PackedVector2Array = w.get("points", PackedVector2Array())
	var sum := 0.0
	var mx := 0.0
	for p in pts:
		var d := _dist(p, line)
		sum += d
		mx = maxf(mx, d)
	wg.way_stroke_undo()
	return [sum / maxf(float(pts.size()), 1.0), mx, pts.size()]

func _ok(c: bool, what: String) -> void:
	print("%s  %s" % ["PASS" if c else "FAIL", what])
	if not c:
		fails += 1

func _init() -> void:
	var spacing_arg := -1.0
	var table := false
	var ua := OS.get_cmdline_user_args()
	for i in ua.size():
		if ua[i] == "--spacing":
			spacing_arg = float(ua[i + 1])
		elif ua[i] == "--table":
			table = true
	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)
	wg = bridge.world_gen
	wg.generate_sized(4242, 1200.0, GW, GH)
	# Lines: every 14th row from 20 whose whole zigzag band is land.
	var lines: Array = []
	for y in range(16, GH - 16, 4):
		for x0 in range(4, GW - 80, 12):
			var l := _drag_line(float(x0), float(y), 6.0, 70)
			var ok := true
			for p in l:
				if not _land(int(p.x), int(p.y)):
					ok = false
					break
			if ok:
				lines.append(l)
			if lines.size() >= 6:
				break
		if lines.size() >= 6:
			break
	print("lines on land: %d" % lines.size())
	if lines.size() < 3:
		print("FAIL no land fixture")
		quit(1)
		return
	var shell_sp: float = InfrastructureWorkspace.WAY_DRAG_SPACING_CELLS
	_ok(is_equal_approx(shell_sp, DccToolBar.WAY_SPACING_DEFAULT), "the toolbar's default spacing equals the workspace's (%.1f)" % shell_sp)
	var spacings: Array = [spacing_arg] if spacing_arg > 0.0 else ([2.0, 4.0, 6.0, 8.0] if table else [shell_sp])
	for sp in spacings:
		var by_corr := {}
		var by_pull := {}
		for c in [1.5, 3.0, 8.0, 20.0]:
			var m := 0.0
			for l in lines:
				var r := _run(l, sp, c, 2.0)
				m += float(r[0]) if not r.is_empty() else 0.0
			by_corr[c] = m / lines.size()
		for pu in [0.0, 2.0, 20.0, 100.0]:
			var m := 0.0
			for l in lines:
				var r := _run(l, sp, 8.0, pu)
				m += float(r[0]) if not r.is_empty() else 0.0
			by_pull[pu] = m / lines.size()
		var cv: Array = by_corr.values()
		var pv: Array = by_pull.values()
		var cs: float = cv.max() - cv.min()
		var ps: float = pv.max() - pv.min()
		if not table and spacing_arg < 0.0:
			_ok(cs >= 0.15, "at the shell's spacing %.1f the corridor moves the road (spread %.3f cell >= 0.15)" % [float(sp), cs])
			_ok(ps >= 0.15, "at the shell's spacing %.1f the pull moves the road (spread %.3f cell >= 0.15)" % [float(sp), ps])
		print("spacing %.1f  mean deviation (cells) by corridor %s  spread %.3f | by pull %s  spread %.3f" % [float(sp), str(by_corr), cs, str(by_pull), ps])
	print("RESULT fails=%d" % fails)
	quit(1 if fails > 0 else 0)
