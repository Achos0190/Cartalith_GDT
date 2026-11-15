extends Node
## Frame-gap measurement (OUTSTANDING_WORK.md "City Viewer: a river town's water draws as square blocks ...",
## sub-bullet "Coordinator check of ed6e3b96's frames"). MEASURES ONLY; it asserts nothing about the engine.
##
## ed6e3b96 (`urban_riverbank`) reports the nearest house's gap to the bank falling from 39 m to 7 m (median over
## river-kind towns), yet its frame shows every house blocks from the quay. For each river-kind town of one world
## (default seed 483920, 384 x 288) this prints one `TOWN` line and a `DIST` summary:
##   (a) whole-layout nearest-house gap to the bank = distance from a building vertex to the `river` centreline minus
##       `river_w / 2` (floored at 0), the census definition (`town_river_census.rs` `bank_d`);
##   (b) the same, over houses whose centroid is inside the City Viewer's opening frame (`CityViewerWindow._view_transform`
##       at zoom 1, pan 0, as `_viewerframe_probe.gd` computes it; `_fit_box` and `fit_with_water` decide the box);
##   (c) the share of the town's houses within 1 and 2 street widths of the bank (street width = 5.4 m, the widest
##       street-class edge the census reports for every river town; the dictionary does not carry edge widths);
##   (d) the distance from the nearest house to the nearest `bridges` point (or `ford`), and whether the town has one.
## Also `seen`: the nearest house to the DRAWN water = min of (a) and the distance to the nearest `water_mask_runs`
## rectangle, since the viewer paints the mask runs.
## House gap uses vertices for houses whose centroid is within 40 m of the centreline and the centroid less 6 m
## (a labelled judgement: a footprint's half-extent) otherwise, which cannot change a gap below 34 m.
##
##   APPDATA=<scratch> Godot_v4.7.2-stable_win64_console.exe --headless --path . _framegap_probe.tscn -- [--seed N] [--grid WxH] [--riverbank 0|1]
## Exit 0 always unless a bad argument (2). Result lines start `TOWN`, `DIST`, `WORST`, `RESULT`.

const STREET_W := 5.4

func _pct(a: Array, q: float) -> float:
	if a.is_empty():
		return NAN
	var s := a.duplicate()
	s.sort()
	return float(s[clampi(int(round(q * (s.size() - 1))), 0, s.size() - 1)])

func _summ(name: String, a: Array) -> String:
	return "DIST %-34s n=%3d median %7.1f  p10 %7.1f  p90 %7.1f  max %7.1f" % [name, a.size(), _pct(a, 0.5), _pct(a, 0.1), _pct(a, 0.9), _pct(a, 1.0)]

## Distance from point `p` to the polyline `pl`; INF when it has under two points.
func _dline(p: Vector2, pl: PackedVector2Array) -> float:
	var best := INF
	for i in pl.size() - 1:
		best = minf(best, p.distance_to(Geometry2D.get_closest_point_to_segment(p, pl[i], pl[i + 1])))
	return best

func _dmask(p: Vector2, runs: Array) -> float:
	var best := INF
	for run in runs:
		var q: PackedVector2Array = run
		if q.size() < 4:
			continue
		var r := Rect2(q[0], q[2] - q[0]).abs()
		var c := Vector2(clampf(p.x, r.position.x, r.end.x), clampf(p.y, r.position.y, r.end.y))
		best = minf(best, c.distance_to(p))
	return best

func _ready() -> void:
	get_tree().create_timer(1500.0).timeout.connect(func() -> void: get_tree().quit(2))
	var seed_n := 483920
	var gw := 384
	var gh := 288
	var riverbank := -1
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed": seed_n = int(args[i + 1])
			"--grid":
				var wh := String(args[i + 1]).split("x")
				gw = int(wh[0])
				gh = int(wh[1])
			"--riverbank": riverbank = int(args[i + 1])
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	if riverbank >= 0:
		print("urban_riverbank set to %d: %s" % [riverbank, str(bridge.param_set("urban_riverbank", riverbank == 1))])
	var places: Array = bridge.settlements()
	app.open_city_viewer(0)
	for _f in 10:
		await get_tree().process_frame
	var cv = app.city_viewer_window
	cv.size = Vector2i(1560, 960)
	for _f in 8:
		await get_tree().process_frame
	var csz: Vector2 = cv._canvas.size
	var a_all: Array = []
	var b_frame: Array = []
	var seen_all: Array = []
	var seen_frame: Array = []
	var share1: Array = []
	var share2: Array = []
	var br_gap: Array = []
	var n_bridge := 0
	var n_near_bridge := 0
	var frac_in_frame: Array = []
	var rows: Array = []
	for k in places.size():
		var lay: Dictionary = bridge.urban_layouts(PackedInt32Array([k]))[0]
		var kind := String(lay.get("site_kind", ""))
		if not (kind in ["river", "riverthrough"]):
			continue
		var river: PackedVector2Array = lay.get("river", PackedVector2Array())
		if river.size() < 2:
			continue
		var rw := float(lay.get("river_w", 20.0))
		var runs: Array = lay.get("water_mask_runs", [])
		cv._layout = lay
		var vt: Dictionary = cv._view_transform()
		var sc: float = vt["scale"]
		var org: Vector2 = vt["origin"]
		var vis := Rect2(-org / sc, csz / sc)
		var blds: Array = lay.get("buildings", [])
		if blds.is_empty():
			continue
		var gap_all := INF
		var gap_frame := INF
		var near_pt := Vector2.ZERO
		var cands: Array = []   # [gap, centroid, in_frame]
		var n_in_frame := 0
		var n1 := 0
		var n2 := 0
		for b in blds:
			var poly: PackedVector2Array = b
			var c := Vector2.ZERO
			for v in poly:
				c += v
			c /= maxf(1.0, poly.size())
			var dc := _dline(c, river) - rw / 2.0
			var g := maxf(0.0, dc - 6.0)
			if dc < 40.0:
				g = INF
				for v in poly:
					g = minf(g, maxf(0.0, _dline(v, river) - rw / 2.0))
			var inf_: bool = vis.has_point(c)
			if inf_:
				n_in_frame += 1
			if g <= STREET_W:
				n1 += 1
			if g <= 2.0 * STREET_W:
				n2 += 1
			if g < gap_all:
				gap_all = g
				near_pt = c
			if inf_ and g < gap_frame:
				gap_frame = g
			if g < 40.0:
				cands.append([g, c, inf_])
		# the drawn water: also the mask rectangles, for the houses near the centreline only
		var seen_a := gap_all
		var seen_f := gap_frame
		for cd in cands:
			var dm := _dmask(cd[1], runs)
			if dm < seen_a:
				seen_a = dm
			if cd[2] and dm < seen_f:
				seen_f = dm
		var pts: PackedVector2Array = lay.get("bridges", PackedVector2Array())
		var cross: Array = []
		cross.append_array(pts)
		if lay.has("ford"):
			cross.append(lay["ford"])
		var bd := INF
		for p in cross:
			bd = minf(bd, near_pt.distance_to(p))
		if not cross.is_empty():
			n_bridge += 1
			br_gap.append(bd)
			if bd <= 40.0:
				n_near_bridge += 1
		a_all.append(gap_all)
		if gap_frame < INF:
			b_frame.append(gap_frame)
		seen_all.append(seen_a)
		if seen_f < INF:
			seen_frame.append(seen_f)
		share1.append(100.0 * n1 / blds.size())
		share2.append(100.0 * n2 / blds.size())
		frac_in_frame.append(100.0 * n_in_frame / blds.size())
		rows.append([k, gap_all, gap_frame, seen_a, 100.0 * n1 / blds.size(), 100.0 * n2 / blds.size(), bd, blds.size(), vis.size.x])
		print("TOWN #%3d %-12s houses %4d in_frame %4d  a_all %6.1f  b_frame %6.1f  seen %6.1f  share<=1st %5.1f%%  share<=2st %5.1f%%  bridges %d  nearest_house_to_crossing %s  frame_w %.0f m" % [
			k, kind, blds.size(), n_in_frame, gap_all, gap_frame, seen_a, 100.0 * n1 / blds.size(), 100.0 * n2 / blds.size(), cross.size(), ("%.0f m" % bd) if bd < INF else "none", vis.size.x])
	print("MEASURE seed %d grid %dx%d urban_riverbank %d: %d river-kind towns with a centreline and houses" % [seed_n, gw, gh, riverbank, rows.size()])
	print(_summ("(a) whole-layout nearest house (m)", a_all))
	print(_summ("(b) nearest house in opening frame", b_frame))
	print(_summ("seen: nearest to drawn water, whole", seen_all))
	print(_summ("seen: nearest to drawn water, frame", seen_frame))
	print(_summ("(c) % houses <= 1 street width", share1))
	print(_summ("(c) % houses <= 2 street widths", share2))
	print(_summ("% of houses inside the frame", frac_in_frame))
	print("(d) towns with a bridge or ford: %d of %d; nearest house within 40 m of one: %d" % [n_bridge, rows.size(), n_near_bridge])
	print(_summ("(d) nearest house to crossing (m)", br_gap))
	var n_b_over := 0
	for v in b_frame:
		if v > STREET_W * 2.0:
			n_b_over += 1
	print("towns whose frame's nearest house is more than 2 street widths from the bank: %d of %d" % [n_b_over, b_frame.size()])
	rows.sort_custom(func(x, y) -> bool: return (x[2] if x[2] < INF else 1e9) > (y[2] if y[2] < INF else 1e9))
	for j in mini(5, rows.size()):
		print("WORST(b) #%d  b_frame %.1f  a_all %.1f  houses %d" % [rows[j][0], rows[j][2], rows[j][1], rows[j][7]])
	print("RESULT: DONE")
	get_tree().quit(0)
