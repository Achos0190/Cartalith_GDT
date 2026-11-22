extends Node
## RR-T1 (Ruling FE item 4, `RIVER_REALISM_SCOPE.md`): the town's river is the curve the map draws, checked in the
## real shell against the real drawn rivers (`get_rivers`' `render_points`, the geometry the strokes rasterize).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _rivertown_probe.tscn -- --out <dir> [--seed N] [--mutate off] [--quick: measure only, no frames]
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user dir.
##
## What it protects (each check names itself, each pixel claim has a positive control):
##   1. THE SWITCH IS LIVE and ON in the app: `urban_river_smooth` changes the very next `urban_layouts()`.
##   2. ONE CURVE: over every river town, the layout river's vertices inside the box lie within `ON_OFFSET_BAR_M` (30 m) of the
##      drawn river for at least 90 % of towns with the switch on, and the same measure with it off is far above that
##      (the control: the OFF layouts are the raw trace, hundreds of metres from the drawn curve).
##   3. SMOOTH: with it on, the median town's sharpest turn is under `TURN_BAR_DEG` and the staircase (turns over it) is
##      cut by at least half against off.
##   4. NO WATER BUILDINGS and no collapse: buildings per town move by under 25 % in the median.
##   5. THE FRAMES: City Viewer and map frames for four river towns, off and on, at the opening frame and at x4, and a deep
##      map frame where the town plan is drawn over the map; the combined PNG is `<out>/combined.png`.
## `--mutate off`: the "on" state is left off, so check 2 must FAIL (a probe that passes its own mutant is decoration).
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

## Measured 2026-11-22: 204 of 220 towns lie within 30 m, 122 within 5 m; the 5-30 m tail is the layout nudging its river off the market (labelled judgement).
const ON_OFFSET_BAR_M := 30.0
const TURN_BAR_DEG := 30.0
const TOWNS_FRAMED := 4

var _out := ""
var _seed := 483920
var _mutate := ""
var _quick := false
var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _shot(rect: Rect2) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	return img.get_region(Rect2i(rect))

func _distinct(img: Image) -> int:
	var seen := {}
	for y in range(0, img.get_height(), 3):
		for x in range(0, img.get_width(), 3):
			var c := img.get_pixel(x, y)
			seen[Vector3i(int(c.r * 15.0), int(c.g * 15.0), int(c.b * 15.0))] = true
	return seen.size()

func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var d := b - a
	var l2 := d.length_squared()
	var t := 0.0 if l2 <= 0.0 else clampf((p - a).dot(d) / l2, 0.0, 1.0)
	return p.distance_to(a + d * t)

## Metres from each in-box vertex of `layout["river"]` to the nearest drawn river segment near the settlement;
## the per-town max, and the sharpest turn (degrees) over the in-box vertices.
func _measure(layout: Dictionary, s: Dictionary, runs: Array, gpm: float) -> Dictionary:
	var river: PackedVector2Array = layout.get("river", PackedVector2Array())
	var pin: Vector2 = layout.get("pin", layout.get("market", Vector2.ZERO))
	var wm := float(layout["wm"])
	var hm := float(layout["hm"])
	var centre := Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5)
	var segs: Array = []
	var near_d := INF
	var near_drawn := false
	var near_run: Dictionary = {}
	for r in runs:
		var rp: PackedVector2Array = r["render_points"]
		var drawn: bool = r.has("widths") and not r.has("parallel_of")
		for i in range(rp.size() - 1):
			if rp[i].distance_to(centre) < 6.0 or rp[i + 1].distance_to(centre) < 6.0:
				segs.append([rp[i], rp[i + 1]])
				var dd := _seg_dist(centre, rp[i], rp[i + 1])
				if dd < near_d:
					near_d = dd
					near_drawn = drawn
					near_run = r
	var inside: Array = []
	var worst := 0.0
	for v in river:
		if v.x < 0.0 or v.y < 0.0 or v.x > wm or v.y > hm:
			continue
		inside.append(v)
		var g := centre + (v - pin) * gpm
		var best := INF
		for sg in segs:
			best = minf(best, _seg_dist(g, sg[0], sg[1]))
		if best < INF:
			worst = maxf(worst, best / gpm)
	var turn := 0.0
	var stairs := 0
	for i in range(1, inside.size() - 1):
		var a: Vector2 = inside[i] - inside[i - 1]
		var b: Vector2 = inside[i + 1] - inside[i]
		if a.length() < 0.5 or b.length() < 0.5:
			continue
		var t := rad_to_deg(absf(a.angle_to(b)))
		turn = maxf(turn, t)
		if t > TURN_BAR_DEG:
			stairs += 1
	return {"near_run": near_run, "near_drawn": near_drawn, "near_d": near_d, "offset": worst, "turn": turn, "stairs": stairs, "n": inside.size(), "segs": segs.size()}

func _drawn_count(rs: Array) -> Array:
	var d := 0
	var bad := 0
	for r in rs:
		if r.has("widths") and not r.has("parallel_of"):
			d += 1
		elif r.has("widths") and r.has("parallel_of"):
			bad += 1
	return [d, bad]

func _quays(l: Dictionary) -> int:
	var q = (l.get("streets", {}) as Dictionary).get("quay", PackedVector2Array())
	return (q as PackedVector2Array).size()

func _median(v: Array) -> float:
	if v.is_empty():
		return NAN
	var s := v.duplicate()
	s.sort()
	return float(s[s.size() / 2])

func _ready() -> void:
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		push_error("rivertown probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_rivertown_probe must be windowed")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]
			"--seed": _seed = int(args[i + 1])
			"--mutate": _mutate = args[i + 1]
			"--quick": _quick = true; i -= 1
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	if _out == "":
		push_error("--out <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 1000)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": _seed, "width_km": 800.0, "grid_w": 384, "grid_h": 288, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	print("\n== 1. the switch is live ==")
	_ok(not bridge.param_info("urban_river_smooth").is_empty(), "urban_river_smooth is a registered parameter")
	_ok(bool(bridge.param_get("urban_river_smooth")), "urban_river_smooth is ON in the app's parameters (a new world)")
	var places: Array = bridge.settlements()
	var runs: Array = bridge.rivers(1)
	_ok(runs.size() > 0, "the world has drawn rivers (%d)" % runs.size())
	var gs: Vector2i = bridge.grid_size()
	var gpm := float(gs.x) / 800000.0
	## RR-T2: how many strokes the map draws with the switch on and off, and that no parallel duplicate came back.
	var drawn_on := _drawn_count(runs)
	bridge.param_set("urban_river_smooth", false)
	var drawn_off := _drawn_count(bridge.rivers(1))
	bridge.param_set("urban_river_smooth", true)
	print("  drawn strokes (widths, not parallel_of): off %d, on %d (new %d); runs with parallel_of AND widths: off %d, on %d" % [drawn_off[0], drawn_on[0], drawn_on[0] - drawn_off[0], drawn_off[1], drawn_on[1]])
	_ok(drawn_on[1] == drawn_off[1] and drawn_on[1] == 0, "NO PARALLEL DOUBLES: no parallel_of run carries a width, on or off (%d, %d)" % [drawn_off[1], drawn_on[1]])
	_ok(drawn_on[0] >= drawn_off[0], "the switch only adds strokes (%d -> %d)" % [drawn_off[0], drawn_on[0]])
	var qmax := 0.0
	for r0 in runs:
		qmax = maxf(qmax, float(r0.get("discharge", 0.0)))
	var on_state := _mutate != "off"

	print("\n== 2-4. one curve, smooth, nothing lost (every river town) ==")
	var rows: Array = []
	var stream_rows: Array = []
	for k in places.size():
		bridge.param_set("urban_river_smooth", false)
		var off_l: Array = bridge.urban_layouts(PackedInt32Array([k]))
		bridge.param_set("urban_river_smooth", on_state)
		var on_l: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if off_l.is_empty() or on_l.is_empty():
			continue
		var a: Dictionary = off_l[0]
		var b: Dictionary = on_l[0]
		var st := String(a.get("site_kind", ""))
		if (st != "river" and st != "riverthrough") or not bool(a.get("uses_real_water", false)):
			continue
		if (a.get("river", PackedVector2Array()) as PackedVector2Array).size() < 2:
			continue
		var ma := _measure(a, places[k], runs, gpm)
		var mb := _measure(b, places[k], runs, gpm)
		if int(ma["segs"]) == 0:
			continue
		if rows.size() < 12 or OS.get_environment("RTOWN_LIST") != "":
			print("  town #%d (%d,%d) nearest run drawn=%s at %.2f cells: offset off %.1f on %.1f m | turn off %.1f on %.1f | river vertices in box off %d on %d | river[0] off %s on %s" % [k, int(places[k]["x"]), int(places[k]["y"]), ma["near_drawn"], float(ma["near_d"]), float(ma["offset"]), float(mb["offset"]), float(ma["turn"]), float(mb["turn"]), int(ma["n"]), int(mb["n"]), (a["river"] as PackedVector2Array)[0], (b["river"] as PackedVector2Array)[0]])
		if not bool(ma["near_drawn"]):
			## The owner's question (2026-11-22): why does the map not draw this town's river? Order, discharge against
			## the largest river's, and which gate dropped it (`widths` absent = no discharge reading on its own cells;
			## `parallel_of` = a run beside a heavier one).
			var nr: Dictionary = ma["near_run"]
			print("  UNDRAWN town #%d (%d,%d): run order %s own_order %s discharge %.2f (Q/Q_max %.5f) cells %s keys widths=%s parallel_of=%s" % [k, int(places[k]["x"]), int(places[k]["y"]), nr.get("order"), nr.get("own_order"), float(nr.get("discharge", 0.0)), float(nr.get("discharge", 0.0)) / maxf(1e-9, qmax), nr.get("cells"), nr.has("widths"), nr.get("parallel_of", "none")])
		stream_rows.append({"k": k, "wa": float(a.get("river_w", 0.0)), "wb": float(b.get("river_w", 0.0)), "qa": _quays(a), "qb": _quays(b), "ha": a.has("harbour"), "hb": b.has("harbour")})
		rows.append({"k": k, "off": ma, "on": mb, "bo": (a.get("buildings", []) as Array).size(), "bn": (b.get("buildings", []) as Array).size()})
	## RR-T2 (Ruling HE): towns whose river is laid at stream width with the switch on.
	var st := 0
	var quay_off := 0
	var quay_on := 0
	var harb_off := 0
	var harb_on := 0
	var w_off: Array = []
	var w_on: Array = []
	for r in stream_rows:
		if float(r["wb"]) <= 5.5:
			st += 1
			quay_off += 1 if int(r["qa"]) > 0 else 0
			quay_on += 1 if int(r["qb"]) > 0 else 0
			harb_off += 1 if r["ha"] else 0
			harb_on += 1 if r["hb"] else 0
			w_off.append(float(r["wa"]))
			w_on.append(float(r["wb"]))
	print("  STREAM TOWNS (river_w <= 5.5 m with the switch on): %d of %d; with a quay street off %d, on %d; with a harbour off %d, on %d; river width median off %.1f m, on %.1f m" % [st, stream_rows.size(), quay_off, quay_on, harb_off, harb_on, _median(w_off), _median(w_on)])
	## The unit tests (`a_stream_has_no_quay_and_no_river_port_but_a_river_does`) are the gate; here a world with no stream town is reported, not failed.
	if st < 2:
		print("  NOTE: %d stream towns in this world: the in-app stream measurements need a world with at least two" % st)
	else:
		_ok(quay_on == 0 and harb_on == 0, "NO PORT: no stream town has a quay street or a harbour with the switch on (%d, %d)" % [quay_on, harb_on])
		_ok(quay_off > 0, "control: with the switch off some of the same towns had a quay street (%d)" % quay_off)
	var undrawn := 0
	for r in rows:
		undrawn += 0 if bool(r["off"]["near_drawn"]) else 1
	print("  %d river towns measured, %d of them beside a river the map does not draw (no channel width, or a parallel run)" % [rows.size(), undrawn])
	_ok(rows.size() >= 10, "at least ten river towns measured (silently empty is a failure)")
	var off_off: Array = []
	var on_off: Array = []
	var off_turn: Array = []
	var on_turn: Array = []
	var stairs_off := 0
	var stairs_on := 0
	var kept: Array = []
	var near := 0
	var drawn_rows := 0
	for r in rows:
		off_off.append(float(r["off"]["offset"]))
		on_off.append(float(r["on"]["offset"]))
		off_turn.append(float(r["off"]["turn"]))
		on_turn.append(float(r["on"]["turn"]))
		stairs_off += int(r["off"]["stairs"])
		stairs_on += int(r["on"]["stairs"])
		if bool(r["off"]["near_drawn"]):
			drawn_rows += 1
			near += 1 if float(r["on"]["offset"]) <= ON_OFFSET_BAR_M else 0
		kept.append(absf(float(r["bn"]) - float(r["bo"])) / maxf(1.0, float(r["bo"])))
	print("  offset to the drawn river, per-town max (m): median off %.1f, on %.1f; towns on the curve (<= %.0f m): %d of %d" % [_median(off_off), _median(on_off), ON_OFFSET_BAR_M, near, rows.size()])
	print("  sharpest turn: median off %.1f, on %.1f deg; staircase vertices over %.0f deg: off %d, on %d" % [_median(off_turn), _median(on_turn), TURN_BAR_DEG, stairs_off, stairs_on])
	print("  buildings moved by (median share): %.3f" % _median(kept))
	_ok(float(near) >= 0.9 * float(drawn_rows), "ONE CURVE: at least 90%% of the towns beside a DRAWN river have it within %.0f m of the drawn river with the switch on (%d of %d; the others sit on a run the map merged into a heavier one)" % [ON_OFFSET_BAR_M, near, drawn_rows])
	_ok(_median(off_off) > 50.0, "control: with it off the median town's river is over 50 m from the drawn river (%.1f m)" % _median(off_off))
	_ok(_median(on_turn) < TURN_BAR_DEG, "SMOOTH: the median town's sharpest turn is under %.0f deg with it on (%.1f)" % [TURN_BAR_DEG, _median(on_turn)])
	_ok(stairs_on * 2 <= stairs_off, "SMOOTH: the staircase is at least halved (%d -> %d vertices over %.0f deg)" % [stairs_off, stairs_on, TURN_BAR_DEG])
	_ok(_median(kept) < 0.25, "nothing collapses: buildings move by under 25%% in the median (%.3f)" % _median(kept))

	if _quick:
		bridge.param_set("urban_river_smooth", true)
		_finish()
		return
	print("\n== 5. the frames ==")
	## Four towns: the biggest by buildings among those whose river visibly changed (an off turn of 40 deg or more).
	var cand: Array = []
	for r in rows:
		if float(r["off"]["turn"]) >= 40.0 and float(r["off"]["offset"]) > 100.0 and int(r["bo"]) >= 150:
			cand.append(r)
	cand.sort_custom(func(x, y): return int(x["bo"]) > int(y["bo"]))
	_ok(cand.size() >= TOWNS_FRAMED, "at least %d river towns with a visibly bent raw river to frame (%d)" % [TOWNS_FRAMED, cand.size()])
	var cols := ["cv_off_z1", "cv_on_z1", "cv_off_z4", "cv_on_z4", "map_x1", "map_x4", "map_off_deep", "map_on_deep"]
	var sheet: Dictionary = {}
	var frame_towns: Array = []
	for ci in mini(TOWNS_FRAMED, cand.size()):
		frame_towns.append([int(cand[ci]["k"]), int(cand[ci]["bo"])])
	## Two stream towns as well (RR-T2): river_w at stream width with the switch on.
	var added := 0
	for r in rows:
		for sr in stream_rows:
			if int(sr["k"]) == int(r["k"]) and float(sr["wb"]) <= 5.5 and int(r["bo"]) >= 60 and added < 2:
				frame_towns.append([int(r["k"]), int(r["bo"])])
				added += 1
	if added < 2:
		print("  NOTE: %d stream towns framed" % added)
	for ft in frame_towns:
		var k: int = int(ft[0])
		var s: Dictionary = places[k]
		print("  town #%d (%d, %d), %d buildings" % [k, int(s["x"]), int(s["y"]), int(ft[1])])
		var cvs := {}
		for state in ["off", "on"]:
			bridge.param_set("urban_river_smooth", on_state if state == "on" else false)
			_app.viewport.forget_all_urban_layouts()
			_app.open_city_viewer(k)
			await _frames(12)
			var cv = _app.city_viewer_window
			cv.size = Vector2i(1560, 960)
			cv.position = Vector2i(20, 20)
			await _frames(8)
			cv._look_picker.select(0)
			cv._look_picker.item_selected.emit(0)
			await _frames(3)
			var focus: Vector2 = cv._layout.get("market", Vector2.ZERO)
			for z in [1.0, 4.0]:
				cv._reset_view()
				await _frames(2)
				cv._zoom = z
				var box: Rect2 = cv._fit_box()
				var sz: Vector2 = cv._canvas.size
				var fit: float = minf((sz.x - 32.0) / maxf(1.0, box.size.x), (sz.y - 32.0) / maxf(1.0, box.size.y))
				var vs: float = fit * z
				if z > 1.0:
					cv._pan = sz * 0.5 - focus * vs - ((sz - box.size * vs) * 0.5 - box.position * vs)
				else:
					cv._pan = Vector2.ZERO
				cv._canvas.queue_redraw()
				await _frames(4)
				var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
				var im := await _shot(rect)
				im.save_png(_out.path_join("town%d_cv_%s_z%d.png" % [k, state, int(z)]))
				cvs["%s_z%d" % [state, int(z)]] = im
			cv.hide()
			await _frames(4)
			## The map: opening frame and x4 (identical off and on: the map draws no town plan there), then deep.
			var host = _app.viewport
			var vrect: Rect2 = host.get_global_rect()
			if state == "off":
				host.reset_view()
				await _frames(6)
				host.move_view_to(float(s["x"]), float(s["y"]))
				await _frames(6)
				var m1 := await _shot(vrect)
				m1.save_png(_out.path_join("town%d_map_x1.png" % k))
				cvs["map_x1"] = m1
				host.reset_view()
				await _frames(4)
				host._zoom_at(host.size * 0.5, 4.0)
				host.move_view_to(float(s["x"]), float(s["y"]))
				await _frames(8)
				cvs["map_x4"] = await _shot(vrect)
				cvs["map_x4"].save_png(_out.path_join("town%d_map_x4.png" % k))
			host.reset_view()
			await _frames(4)
			var steps := 0
			while steps < 90 and host.overlay._urban_layout_alpha(host.overlay.displayed_rect()) < 0.99:
				host.zoom_step(1.35)
				host.move_view_to(float(s["x"]), float(s["y"]))
				steps += 1
			host.move_view_to(float(s["x"]), float(s["y"]))
			await _frames(40)
			var md := await _shot(vrect)
			md.save_png(_out.path_join("town%d_map_%s_deep.png" % [k, state]))
			cvs["map_%s_deep" % state] = md
			_ok(_distinct(md) > 30, "town #%d deep map (%s) drew (%d colours, %d zoom steps)" % [k, state, _distinct(md), steps])
		var d_cv := 0
		var a1: Image = cvs["off_z1"]
		var b1: Image = cvs["on_z1"]
		for y in range(0, mini(a1.get_height(), b1.get_height()), 4):
			for x in range(0, mini(a1.get_width(), b1.get_width()), 4):
				if absf(a1.get_pixel(x, y).r - b1.get_pixel(x, y).r) + absf(a1.get_pixel(x, y).g - b1.get_pixel(x, y).g) + absf(a1.get_pixel(x, y).b - b1.get_pixel(x, y).b) > 0.03:
					d_cv += 1
		_ok(d_cv > 50, "town #%d: the City Viewer frames differ off to on (%d sampled pixels; control)" % [k, d_cv])
		sheet[k] = cvs
	## The combined sheet: a row per town, eight columns, each cell 400 px wide.
	var cw := 400
	var rows_n := sheet.size()
	if rows_n > 0:
		var cell_h := 250
		var out_img := Image.create(cw * cols.size(), cell_h * rows_n, false, Image.FORMAT_RGBA8)
		out_img.fill(Color.WHITE)
		var ri := 0
		for k in sheet.keys():
			var cvs: Dictionary = sheet[k]
			for ci in cols.size():
				var key: String = cols[ci]
				var src = cvs.get(key.substr(3) if key.begins_with("cv_") else key)
				if src == null:
					continue
				var im: Image = src.duplicate()
				var sc := minf(float(cw) / float(im.get_width()), float(cell_h) / float(im.get_height()))
				im.resize(int(im.get_width() * sc), int(im.get_height() * sc), Image.INTERPOLATE_LANCZOS)
				out_img.blit_rect(im, Rect2i(0, 0, im.get_width(), im.get_height()), Vector2i(ci * cw, ri * cell_h))
			ri += 1
		out_img.save_png(_out.path_join("combined.png"))
	bridge.param_set("urban_river_smooth", true)
	_finish()

func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
