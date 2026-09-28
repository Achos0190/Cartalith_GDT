extends Node
## **Main-thread cost on the zoom, sharpen and pan paths** (owner ruling
## 2026-09-28, relayed with the repaint row `OUTSTANDING_WORK.md` "The
## colour-texture repaint blocks the main thread ...": *a freeze during
## generation or simulation is acceptable; what must be fast is zooming and
## sharpening*). Measurement only; it changes nothing but its own world and
## the window's vsync.
##
## MUST run windowed (frame_post_draw never fires under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _zoomcost_probe.tscn -- --out DIR
##     [--grid 2048x1311] [--seed 24601] [--vsync off|on] [--overlays shown|hidden]
##     [--notches N] [--layer-costs] [--shots] [--vp WxH] [--force-touch] [--tag NAME]
##
## Every flag above is read in `_ready`; an unknown one aborts. `--force-touch`
## is not read here: `dcc_shell.gd` reads it from the same argument list and
## builds the phone shell. `--vp WxH` hosts the app in a `SubViewport` of that
## size (the phone is 1080x2340, taller than most desktop screens, so a window
## of that size would be clamped); without it the app fills a 1600x1000
## window. `--tag` names the output files.
##
## **What "sharp" means (2026-09-28, overlay layer cache).** Since the overlay
## caches its costly layers and rebuilds them over several frames after a zoom
## (`map_overlay.gd`'s "Layer cache" block), a notch is not finished when the
## deep-zoom tiles have landed: the overlay may still be rebuilding. A notch
## and the pan now wait for both (`lod_pending() == 0` and the overlay's
## `pending_builds() == 0`), so the worst frame recorded includes every frame
## of that rebuild -- the cost is not allowed to hide after the measurement.
## `overlay_frames` is how many frames the overlay took to catch up.
##
## `--shots`: instead of the timing legs, every layer is switched on and the
## map is captured at fit and at zoom 4, 16 and 32 (absolute camera zoom, the
## navpad's `zoom_step` at the view centre), each once everything is sharp and
## 10 more frames have drawn, plus once more at zoom 16 after the pan leg's own
## 120-frame pan (`pan16`) -- the view a cached layer must still draw right.
## PNGs go to `--out` as `shot_<tag>_<name>.png`; compare two runs' PNGs with
## any image diff (the overlay change that added this measured its own with
## `_zoomcost_diff.py`, kept in that lane's scratch, not in the tree).
##
## Vsync is OFF by default, so a frame's wall time is the work in it, not the
## wait for the display: a frame over 16.7 ms here is a frame that could not
## have made 60 Hz. `--overlays shown` (the default) keeps the layers a fresh
## app shows, because that is what the user zooms; `hidden` is the control that
## separates the overlay's redraw from the map's own work.
##
## Legs:
## - `idle`: 60 frames with nothing moving -- the floor every other number sits on.
## - `redraw`: 10 frames, each asking the overlay to redraw at a fixed camera --
##   the cost a camera move pays for the overlay alone (`_update_lod()` queues
##   it on every move).
## - `notch`: from fit, `--notches` wheel notches (`ZOOM_WHEEL_STEP`, 1.15, at
##   the viewport centre -- the navpad's `zoom_step`), each followed by frames
##   until every requested deep-zoom tile has landed ("sharpened") or 300
##   frames. Per notch: the synchronous `zoom_step` call, the first frame after
##   it, the worst frame while sharpening, frames and wall time to sharp.
## - `pan`: 120 frames of a 6 px/frame pan at zoom 16 (the pyramid is up),
##   `_lodsweep_probe.gd`'s own pan.
## - `--layer-costs` (instead of the legs above): the overlay redraw with ONE
##   layer shown at a time, at fit and at zoom 16, 10 redraws each -- which
##   layer the overlay's per-move cost belongs to. `none` is every layer hidden.
##
## A frame's time is the wall time between two `frame_post_draw`s; its
## `process_ms` is Godot's own `Performance.TIME_PROCESS` for it.

const WHEEL := 1.15
const PAN_ZOOM := 16.0
const PAN_PX := 6.0
const PAN_FRAMES := 120
const SHARP_FRAMES := 300
const BAR_MS := 1000.0 / 60.0
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "rivers", "conflict"]

var _out := "user://zoomcost/"
var _grid := Vector2i(2048, 1311)
var _seed := 24601
var _vsync := false
var _overlays := true
var _notches := 24
var _layer_costs := false
var _shots := false
var _tag := "desk"
## `--vp`'s host, or null for the window itself.
var _vp: SubViewport = null
var _app: Node
var _vh: Control
var _br: Node
var _last_us := 0


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--notches": _notches = int(args[i + 1]); i += 1
			"--vsync": _vsync = args[i + 1] == "on"; i += 1
			"--overlays": _overlays = args[i + 1] != "hidden"; i += 1
			"--layer-costs": _layer_costs = true
			"--shots": _shots = true
			"--force-touch": pass
			"--tag": _tag = args[i + 1]; i += 1
			"--vp":
				var v := args[i + 1].split("x")
				_vp = SubViewport.new()
				_vp.size = Vector2i(int(v[0]), int(v[1]))
				_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
				_vp.gui_embed_subwindows = true
				i += 1
			"--grid":
				var p := args[i + 1].split("x")
				_grid = Vector2i(int(p[0]), int(p[1])); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	DisplayServer.window_set_size(Vector2i(1600, 1000) if _vp == null else Vector2i(540, 540))
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_ENABLED if _vsync else DisplayServer.VSYNC_DISABLED)
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	if _vp != null:
		add_child(_vp)
		_vp.add_child(_app)
	else:
		add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().quit(await _run())


## Waits for the next drawn frame; returns its wall time and process time, ms.
func _frame() -> Dictionary:
	await RenderingServer.frame_post_draw
	var now := Time.get_ticks_usec()
	var ms := (now - _last_us) / 1000.0
	_last_us = now
	return {"ms": ms, "process_ms": Performance.get_monitor(Performance.TIME_PROCESS) * 1000.0}


## Overlay layers still rebuilding after a camera move; 0 for an overlay that
## predates the layer cache, which rebuilt everything inside the move's frame.
func _ov_pending() -> int:
	return int(_vh.overlay.pending_builds()) if _vh.overlay.has_method("pending_builds") else 0


## Tiles landed AND the overlay caught up -- see the header's "What sharp means".
func _sharp() -> bool:
	return _vh.lod_pending() == 0 and _ov_pending() == 0


func _stats(v: PackedFloat64Array) -> Dictionary:
	if v.is_empty():
		return {}
	var s := v.duplicate()
	s.sort()
	return {"median": s[s.size() / 2], "min": s[0], "max": s[s.size() - 1], "n": s.size()}


func _run() -> int:
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _br.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.5).timeout
	if not _br.has_world:
		print("PROBE-RESULT: ABORT no world"); return 2
	if not _overlays:
		for l in HIDE:
			_vh.set_layer_visible(l, false)
	_vh.reset_view()
	for k in 10:
		await _frame()
	var out := {"grid": [_grid.x, _grid.y], "seed": _seed, "vsync": _vsync, "overlays": _overlays,
		"tag": _tag, "vp": [_vh.size.x, _vh.size.y]}
	if _layer_costs:
		return await _run_layer_costs(out)
	if _shots:
		return await _run_shots()

	## idle
	var idle := PackedFloat64Array()
	for k in 60:
		idle.append((await _frame())["ms"])
	out["idle"] = _stats(idle)

	## redraw: the overlay alone, at fit, at a fixed camera
	var redraw := PackedFloat64Array()
	for k in 10:
		_vh.overlay.queue_redraw()
		redraw.append((await _frame())["ms"])
	out["redraw_fit"] = _stats(redraw)

	## notch: from fit toward the deepest zoom
	_vh.reset_view()
	for k in 5:
		await _frame()
	var notches := []
	var breaches := []
	for n in _notches:
		var t0 := Time.get_ticks_usec()
		_vh.zoom_step(WHEEL)
		var call_ms := (Time.get_ticks_usec() - t0) / 1000.0
		var frames := PackedFloat64Array()
		var procs := PackedFloat64Array()
		var t_start := Time.get_ticks_usec()
		var f := 0
		var ov_frames := 0
		while f < SHARP_FRAMES:
			var fr := await _frame()
			frames.append(fr["ms"]); procs.append(fr["process_ms"])
			f += 1
			if _ov_pending() > 0:
				ov_frames = f
			if _sharp():
				break
		var row := {"notch": n + 1, "zoom": _vh.zoom(), "lod": _vh.lod_active(), "call_ms": call_ms,
			"first_frame_ms": frames[0], "worst_frame_ms": _stats(frames)["max"],
			"worst_process_ms": _stats(procs)["max"], "frames_to_sharp": f, "overlay_frames": ov_frames,
			"ms_to_sharp": (Time.get_ticks_usec() - t_start) / 1000.0, "sharp": _sharp()}
		notches.append(row)
		for j in frames.size():
			if frames[j] > BAR_MS:
				breaches.append({"leg": "notch", "notch": n + 1, "frame": j, "ms": frames[j], "process_ms": procs[j], "zoom": row["zoom"]})
		print("  notch %2d  zoom %6.2f lod %-5s  call %6.2f ms  first %6.2f  worst %6.2f (process %6.2f)  sharp in %3d frames (overlay %2d) / %7.1f ms" % [
			n + 1, row["zoom"], str(row["lod"]), call_ms, row["first_frame_ms"], row["worst_frame_ms"], row["worst_process_ms"], f, ov_frames, row["ms_to_sharp"]])
	out["notches"] = notches

	## pan at zoom 16, pyramid up
	_vh.reset_view()
	for k in 3:
		await _frame()
	_vh.zoom_step(PAN_ZOOM / _vh.zoom())
	for k in SHARP_FRAMES:
		await _frame()
		if _sharp():
			break
	var pan := PackedFloat64Array()
	var pan_proc := PackedFloat64Array()
	for k in PAN_FRAMES:
		_pan_one()
		var fr := await _frame()
		pan.append(fr["ms"]); pan_proc.append(fr["process_ms"])
		if fr["ms"] > BAR_MS:
			breaches.append({"leg": "pan", "frame": k, "ms": fr["ms"], "process_ms": fr["process_ms"]})
	## The frames after the last pan step until everything is sharp: a rebuild
	## the pan started must be paid for somewhere, and it is counted here.
	var settle := PackedFloat64Array()
	for k in SHARP_FRAMES:
		if _sharp():
			break
		var fr := await _frame()
		settle.append(fr["ms"])
		if fr["ms"] > BAR_MS:
			breaches.append({"leg": "pan_settle", "frame": k, "ms": fr["ms"], "process_ms": fr["process_ms"]})
	out["pan"] = _stats(pan)
	out["pan_process"] = _stats(pan_proc)
	out["pan_settle"] = _stats(settle)
	## The raw frame times too, so a reader can take any percentile.
	out["pan_frames"] = pan
	out["pan_settle_frames"] = settle
	out["breaches"] = breaches

	var worst := PackedFloat64Array()
	var firsts := PackedFloat64Array()
	var calls := PackedFloat64Array()
	for r: Dictionary in notches:
		worst.append(r["worst_frame_ms"]); firsts.append(r["first_frame_ms"]); calls.append(r["call_ms"])
	out["notch_worst"] = _stats(worst)
	out["notch_first"] = _stats(firsts)
	out["notch_call"] = _stats(calls)
	print("  idle frame ms: ", out["idle"])
	print("  overlay redraw at fit, ms/frame: ", out["redraw_fit"])
	print("  notch: zoom_step call ms ", out["notch_call"], "  first frame ms ", out["notch_first"], "  worst frame while sharpening ms ", out["notch_worst"])
	print("  pan at x16 frame ms: ", out["pan"], "  process ms: ", out["pan_process"], "  settle after pan ms: ", out["pan_settle"])
	print("  frames over %.1f ms: %d" % [BAR_MS, breaches.size()])
	var f := FileAccess.open(_out.path_join("zoomcost_%s_%dx%d_%s.json" % [_tag, _grid.x, _grid.y, "ov" if _overlays else "noov"]), FileAccess.WRITE)
	f.store_string(JSON.stringify(out, "  "))
	print("PROBE-RESULT: DONE")
	return 0


## `--layer-costs`: which overlay layer the per-move redraw belongs to.
func _run_layer_costs(out: Dictionary) -> int:
	var rows := {}
	for z in [1.0, PAN_ZOOM]:
		_vh.reset_view()
		for k in 3:
			await _frame()
		if z > 1.0:
			_vh.zoom_step(z / _vh.zoom())
			for k in SHARP_FRAMES:
				await _frame()
				if _vh.lod_pending() == 0:
					break
		for layer in ["none"] + HIDE:
			for l in HIDE:
				_vh.set_layer_visible(l, l == layer)
			for k in 3:
				await _frame()
			var v := PackedFloat64Array()
			for k in 10:
				_vh.overlay.queue_redraw()
				v.append((await _frame())["ms"])
			var st := _stats(v)
			rows["%s@x%d" % [layer, int(z)]] = st
			print("  redraw with only %-16s at x%-3d  median %6.2f ms (%.2f..%.2f)" % [layer, int(z), st["median"], st["min"], st["max"]])
	out["layer_costs"] = rows
	var f := FileAccess.open(_out.path_join("zoomcost_layers_%s_%dx%d.json" % [_tag, _grid.x, _grid.y]), FileAccess.WRITE)
	f.store_string(JSON.stringify(out, "  "))
	print("PROBE-RESULT: DONE")
	return 0


## One step of the pan leg: `_lodsweep_probe.gd`'s own 6 px/frame diagonal,
## through the same `_update_lod()` every real camera move reaches.
func _pan_one() -> void:
	_vh._camera.position += Vector2(-PAN_PX, -PAN_PX * 0.5)
	_vh._update_lod()


## `--shots`: see the header. Every layer on, then fit / x4 / x16 / x32, then
## the pan leg's pan at x16. Waits for `_sharp()` and 10 more frames before each
## capture, so a capture never holds a half-built layer or a missing tile.
func _run_shots() -> int:
	for l in HIDE:
		_vh.set_layer_visible(l, true)
	## The map's own rectangle in the capture, so a comparison can leave out
	## the shell around it (the status bar and the pipeline panel print
	## timings, which differ run to run).
	var r: Rect2 = _vh.get_global_rect()
	var rf := FileAccess.open(_out.path_join("shot_%s_rect.json" % _tag), FileAccess.WRITE)
	rf.store_string(JSON.stringify([r.position.x, r.position.y, r.size.x, r.size.y]))
	rf.close()
	var shots :=[["fit", 0.0], ["x4", 4.0], ["x16", 16.0], ["x32", 32.0], ["pan16", 16.0]]
	for sh in shots:
		_vh.reset_view()
		for k in 3:
			await _frame()
		if float(sh[1]) > 0.0:
			_vh.zoom_step(float(sh[1]) / _vh.zoom())
		for k in SHARP_FRAMES:
			await _frame()
			if _sharp():
				break
		if sh[0] == "pan16":
			for k in PAN_FRAMES:
				_pan_one()
				await _frame()
			for k in SHARP_FRAMES:
				if _sharp():
					break
				await _frame()
		for k in 10:
			await _frame()
		var img: Image = (_vp.get_texture() if _vp != null else get_viewport().get_texture()).get_image()
		var path := _out.path_join("shot_%s_%s.png" % [_tag, sh[0]])
		img.save_png(path)
		print("  shot %-6s zoom %6.2f sharp %s (tiles pending %d) -> %s" % [sh[0], _vh.zoom(), str(_sharp()), _vh.lod_pending(), path])
	print("PROBE-RESULT: DONE")
	return 0
