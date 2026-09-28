extends Node
## RV-3 follow-ups (`OUTSTANDING_WORK.md` row "RV-3 follow-ups: a sculpted
## channel with no drawn river is filled in the shading; the valley-shade build
## cost is unmeasured; ..."). Measurement only; it changes nothing but the
## probe's own world.
##
## MUST run windowed (the sculpt leg reads frames; frame_post_draw / get_image
## are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _rv3cost_probe.tscn -- --mode cost --grid 2048x2048 --out DIR
##   Godot_v4.7.1-stable_win64_console.exe --path . _rv3cost_probe.tscn -- --mode sculpt --out DIR
##
## `--mode cost` (item 2): generates one world through the real shell (the
## app's own `bridge.generate`, its default parameters, the given grid), then
## reads `WorldGen.valley_shade_stats()` -- the wall time of the last uncached
## `valley_shade_field` build and whether it ran on the main thread. The first
## reading is the app's own first repaint after generation. Then `--reps`
## small freehand-lower sculpt commits, each followed by the same repaint
## `world_workspace.gd::_on_sculpt_commit` does (`bridge.color_texture()`),
## and each rep's valley build and whole repaint are timed. Median (min..max).
##
## `--mode sculpt` (item 2 of the row's fix, the "sculpt-a-channel probe
## leg"): generates seed 24601 at 1024x656, finds an inland cell far from
## every traced river, sculpts a straight River-feature channel there
## (`CHANNEL_CELLS` long), commits it as the shell does, and frames it at x4
## and x8 with the Rivers layer off. Reported: whether any drawn river passes
## within 3 cells of the channel (the leg is only meaningful when none does),
## and the channel's shading relief -- the range of its along-averaged luma
## cross-section (`_relief`) against the same on a parallel control line
## `CONTROL_OFFSET` cells away. A channel the shading fills in reads flat
## inside its walls; a surviving trough reads its walls.

const CHANNEL_CELLS := 40.0   ## sculpted channel length in cells; a labelled judgement:
                              ## long enough to read on screen at x4, short enough to
                              ## stay inside one flat patch of land
const CONTROL_OFFSET := 30.0  ## the control line's offset from the channel, cells (clear of
                              ## the stamp's width, ~13 cells at the default brush)
const PROFILE_CELLS := 12     ## half-width of the sampled cross-section, cells: wider
                              ## than the default River stamp's trough

var _mode := "cost"
var _out := "user://rv3cost/"
var _grid := Vector2i(2048, 2048)
var _seed := 24601
var _reps := 5
var _vp := Vector2i(1600, 1000)
var _app: Node
var _vh: Control
var _br: Node
var _last_profile: Array = [] ## `_relief`'s last averaged cross-section, north to south

const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--mode": _mode = args[i + 1]; i += 1
			"--out": _out = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--reps": _reps = int(args[i + 1]); i += 1
			"--grid":
				var p := args[i + 1].split("x")
				_grid = Vector2i(int(p[0]), int(p[1])); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	DisplayServer.window_set_size(_vp)
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	DirAccess.make_dir_recursive_absolute(_out)
	var code := 0
	if _mode == "cost":
		code = await _cost()
	elif _mode == "sculpt":
		code = await _sculpt()
	else:
		printerr("unknown mode ", _mode); code = 2
	get_tree().quit(code)


func _generate(grid: Vector2i, seed_v: int) -> bool:
	var t0 := Time.get_ticks_msec()
	_br.generate({"seed": seed_v, "width_km": 1200.0, "grid_w": grid.x, "grid_h": grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 12000:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	print("  generated seed %d %dx%d in %.1f s, has_world %s" % [seed_v, grid.x, grid.y,
		(Time.get_ticks_msec() - t0) / 1000.0, str(_br.has_world)])
	return _br.has_world


func _stats() -> Dictionary:
	return _br.world_gen.valley_shade_stats()


func _cost() -> int:
	print("\n======== cost: seed %d grid %dx%d ========" % [_seed, _grid.x, _grid.y])
	if not await _generate(_grid, _seed):
		print("PROBE-RESULT: ABORT no world"); return 2
	var first := _stats()
	print("  first build (the app's own repaint after generation): ", first)
	if not first.get("built", false):
		print("PROBE-RESULT: ABORT the app's repaint built no valley field (smooth_valleys off, or no carve)")
		return 2
	_br.sculpt_set_feature("freehand")
	var reps := []
	for r in _reps:
		var cx := _grid.x * (0.3 + 0.1 * r)
		var cy := _grid.y * 0.5
		_br.sculpt_begin_stroke()
		for k in 5:
			_br.sculpt_add_point(cx + k * 2.0, cy + k * 2.0)
		_br.sculpt_end_stroke()
		if _br.sculpt_stamp_count() == 0:
			print("  rep %d: SKIPPED, no stamp landed" % r); continue
		var before := _stats()
		var t0 := Time.get_ticks_usec()
		_br.sculpt_commit("rv3cost")
		var t1 := Time.get_ticks_usec()
		_vh.map_view.texture = _br.color_texture()
		var t2 := Time.get_ticks_usec()
		var st := _stats()
		var fresh: bool = st != before
		var row := {"commit_ms": (t1 - t0) / 1000.0, "repaint_ms": (t2 - t1) / 1000.0,
			"valley_ms": st.get("ms", -1.0), "valley_core_ms": st.get("core_ms", -1.0),
			"main_thread": st.get("main_thread", null), "rebuilt": fresh}
		print("  rep %d: %s" % [r, row])
		if fresh:
			reps.append(row)
		_vh.invalidate_lod_tiles()
		await get_tree().create_timer(0.5).timeout
	var out := {"grid": [_grid.x, _grid.y], "seed": _seed, "first": first, "reps": reps}
	for k in ["valley_ms", "valley_core_ms", "repaint_ms", "commit_ms"]:
		var v := []
		for row: Dictionary in reps: v.append(float(row[k]))
		v.sort()
		if v.is_empty():
			continue
		out[k] = {"median": v[v.size() / 2], "min": v[0], "max": v[v.size() - 1], "n": v.size()}
		print("  %-15s median %.1f ms (%.1f..%.1f, n %d)" % [k, v[v.size() / 2], v[0], v[v.size() - 1], v.size()])
	var f := FileAccess.open(_out.path_join("rv3cost_%dx%d.json" % [_grid.x, _grid.y]), FileAccess.WRITE)
	f.store_string(JSON.stringify(out, "  "))
	print("PROBE-RESULT: DONE")
	return 0


## An inland cell at least 12 cells from every traced river point and from
## water, well above sea level, on a coarse lattice; the one farthest from
## rivers wins. Returns (-1, -1) when none qualifies.
func _pick_site(rivers: Array) -> Vector2:
	var traced := {}
	for r: Dictionary in rivers:
		for p: Vector2 in (r["points"] as PackedVector2Array):
			traced[Vector2i(int(p.x) / 4, int(p.y) / 4)] = true
	var best := Vector2(-1, -1); var best_d := 0.0
	var m := 40 + int(CHANNEL_CELLS)
	for gy in range(m, _grid.y - m, 8):
		for gx in range(m, _grid.x - m, 8):
			var ok := true
			for dx in [-CHANNEL_CELLS * 0.5, 0.0, CHANNEL_CELLS * 0.5]:
				var s: Dictionary = _br.sample_cell(gx + int(dx), gy)
				if String(s.get("water", "land")) != "land" or float(s.get("elevation", 0.0)) < 0.5:
					ok = false; break
			if not ok:
				continue
			## Distance, in 4-cell buckets, to the nearest traced river point.
			var d := 99.0
			for by in range(-8, 9):
				for bx in range(-(int(CHANNEL_CELLS) / 8 + 8), int(CHANNEL_CELLS) / 8 + 9):
					if traced.has(Vector2i(gx / 4 + bx, gy / 4 + by)):
						d = minf(d, Vector2(bx, by).length() * 4.0)
			if d > best_d:
				best_d = d; best = Vector2(gx, gy)
	print("  site ", best, " nearest traced river point ~", best_d, " cells")
	return best if best_d >= 12.0 else Vector2(-1, -1)


func _sculpt() -> int:
	_grid = Vector2i(1024, 656)
	print("\n======== sculpt: seed %d grid %dx%d ========" % [_seed, _grid.x, _grid.y])
	if not await _generate(_grid, _seed):
		print("PROBE-RESULT: ABORT no world"); return 2
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	var site := _pick_site(_br.rivers(1))
	if site.x < 0:
		print("PROBE-RESULT: ABORT no site far from every river"); return 2
	var a := site - Vector2(CHANNEL_CELLS * 0.5, 0.0)
	var b := site + Vector2(CHANNEL_CELLS * 0.5, 0.0)
	_br.sculpt_set_feature("river")
	_br.sculpt_begin_stroke()
	var n := int(CHANNEL_CELLS * 2.0)
	for k in n + 1:
		_br.sculpt_add_point(lerpf(a.x, b.x, float(k) / n), a.y)
	_br.sculpt_end_stroke()
	if _br.sculpt_stamp_count() == 0:
		print("PROBE-RESULT: ABORT the River stroke landed no stamp"); return 2
	var h0: float = _br.sample_cell(int(site.x), int(site.y)).get("elevation", NAN)
	var res: Dictionary = _br.sculpt_commit("rv3 sculpt leg")
	_vh.map_view.texture = _br.color_texture()
	_vh.invalidate_lod_tiles()
	var h1: float = _br.sample_cell(int(site.x), int(site.y)).get("elevation", NAN)
	print("  commit: ", res)
	print("  height at the channel's middle: %.5f -> %.5f (world field; the carve is real)" % [h0, h1])
	## Is any DRAWN river on the channel? (a drawn one gets RV-3's re-cut, which
	## would keep a groove there whatever the fill does)
	var drawn_near := 0
	for r: Dictionary in _br.rivers(1):
		if not (r.has("widths") and r.has("pieces") and not r.has("parallel_of")):
			continue
		for p: Vector2 in (r["render_points"] as PackedVector2Array):
			if p.x > a.x - 3 and p.x < b.x + 3 and absf(p.y - site.y - 0.5) < 3.0:
				drawn_near += 1
	print("  drawn river points within 3 cells of the channel: %d" % drawn_near)
	var rows := []
	for z in [4.0, 8.0]:
		_vh.set_layer_visible("rivers", false)
		_vh.overlay.set_labels([])
		_vh.reset_view()
		await _settle(3)
		_vh.zoom_step(z / _vh.zoom())
		_vh.move_view_to(site.x, site.y + 0.5)
		await _settle(30)
		await _settle_lod()
		var img := await _grab()
		if img == null:
			continue
		var tag := "s%d_channel_z%04.1f" % [_seed, z]
		img.save_png(_out.path_join(tag + ".png"))
		var line := _relief(img, a, b, 0.0)
		var line_prof := _last_profile.duplicate()
		var ctrl := _relief(img, a, b, CONTROL_OFFSET)
		var row := {"zoom": z, "lod": _vh.lod_active(), "channel_relief": line, "control_relief": ctrl,
			"channel_profile": line_prof}
		print("    channel cross-section, offsets -%d..+%d cells (north to south): %s" % [PROFILE_CELLS, PROFILE_CELLS, str(line_prof)])
		print("    z %4.1f lod=%s  cross-section luma range: channel %.2f  control %.2f" % [z, str(_vh.lod_active()), line, ctrl])
		rows.append(row)
	var out := {"seed": _seed, "site": [site.x, site.y], "drawn_near": drawn_near, "height": [h0, h1], "rows": rows}
	var f := FileAccess.open(_out.path_join("rv3sculpt.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(out, "  "))
	print("PROBE-RESULT: DONE")
	return 0


## The channel's shaded cross-section: luma sampled across the segment a..b
## (shifted `dy` cells) at offsets -PROFILE_CELLS..+PROFILE_CELLS in 1-cell
## steps, averaged along it (its tapered ends, 4 cells each side, skipped), and
## reported as that averaged profile's range (max - min). A trough with walls
## reads its lit and shaded walls; a filled bed reads flat. NAN when the view
## does not cover the segment.
func _relief(img: Image, a: Vector2, b: Vector2, dy: float) -> float:
	var vr: Dictionary = _vh.visible_grid_rect()
	if not vr.get("ok", false):
		return NAN
	var x0: float = vr["x0"]; var x1: float = vr["x1"]; var y0: float = vr["y0"]; var y1: float = vr["y1"]
	var w := img.get_width(); var h := img.get_height()
	var prof := PackedFloat64Array(); prof.resize(2 * PROFILE_CELLS + 1)
	var cnt := PackedInt32Array(); cnt.resize(2 * PROFILE_CELLS + 1)
	for k in range(4, int(CHANNEL_CELLS) - 3):
		var gx := a.x + float(k)
		for o in range(-PROFILE_CELLS, PROFILE_CELLS + 1):
			var gy := a.y + 0.5 + dy + o
			var px := int((gx - x0) / (x1 - x0) * w)
			var py := int((gy - y0) / (y1 - y0) * h)
			if px < 0 or py < 0 or px >= w or py >= h:
				continue
			var c := img.get_pixel(px, py)
			prof[o + PROFILE_CELLS] += 0.299 * c.r8 + 0.587 * c.g8 + 0.114 * c.b8
			cnt[o + PROFILE_CELLS] += 1
	var lo := INF; var hi := -INF
	_last_profile = []
	for i in prof.size():
		if cnt[i] == 0:
			return NAN
		var v := prof[i] / cnt[i]
		_last_profile.append(snappedf(v, 0.1))
		lo = minf(lo, v); hi = maxf(hi, v)
	return hi - lo


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	var gp := _vh.global_position
	var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var sub := img.get_region(r)
	sub.convert(Image.FORMAT_RGBA8)
	return sub


func _settle_lod() -> void:
	await _settle(6)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _settle(4)


func _settle(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw
