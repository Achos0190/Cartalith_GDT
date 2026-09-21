extends Node
## **LOD-D0 · the zoom-sweep measurement harness** (`LOD_DETAIL_SCOPE.md`,
## "LOD-D0 · Zoom-sweep harness -- measurement only, no pixel change").
##
## Records today's figures as the baseline every later LOD milestone (D1-D6) is
## graded against. It changes no pixel: it drives the shell's own camera, reads
## the framebuffer back, and prints numbers.
##
## **MUST run WINDOWED.** `RenderingServer.frame_post_draw` never fires under
## the dummy display driver and `get_texture().get_image()` returns null there,
## so every metric below would abort or pass vacuously (`MISTAKES.md`, "Run a
## pixel probe"). The probe refuses to start headless rather than producing a
## number nobody can trust.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _lodsweep_probe.tscn -- \
##       --out C:/some/scratch/dir
##
## ## Arguments -- every one of these is READ BELOW; an unknown one ABORTS
##
## `MISTAKES.md`, "Write a probe's usage header": a documented flag the probe
## never reads is worse than no flag, so `_parse_args()` rejects anything not
## on this list rather than defaulting past it.
##
##   --out DIR          where the sheets and the JSON land.  Default
##                      `user://lodsweep/`; pass a scratch path to keep the
##                      live userdata directory clean.
##   --seeds a,b,c      the fixed seeds.  Default 483920,24601,71077345.
##   --grids WxH,...    grid sizes.  Default 2048x1311,512x384 -- the scope's
##                      own two.
##   --vp WxH           window size.  Default 1600x1000.
##   --width-km F       map width.  Default 1200.  Sets `_zoom_max`
##                      (`max(64, ceil(width_km/5))`), i.e. the sweep's top end.
##   --zoom-step F      geometric zoom per frame.  Default 1.02 (the scope's).
##   --tests a,b,c      any of `sweep`, `pan`, `zoompan`.  Default all three
##                      (the research note's Tests A-C).
##   --theme dark|light the FORCED palette.  Default dark.  A pixel threshold is
##                      palette-bound and this machine boots light, so the probe
##                      forces it IN MEMORY (never `DccSettings.set_theme_mode`,
##                      which would write the settings file) and aborts if the
##                      forcing did not take.
##   --plant            run the planted-defect positive controls INSTEAD of the
##                      baseline sweep.  Every metric must detect its defect.
##   --no-sheet         skip the Aletsch comparison sheet.
##
## ## Exit status (this project's convention)
##   0  every assertion held
##   1  an assertion failed
##   2  could not run

const DEF_SEEDS: PackedInt64Array = [483920, 24601, 71077345]
const DEF_GRIDS := "2048x1311,512x384"
const DEF_VP := Vector2i(1600, 1000)
const DEF_WIDTH_KM := 1200.0
const DEF_ZOOM_STEP := 1.02
const DEF_TESTS := "sweep,pan,zoompan"

## The owner's target image (`LOD_DETAIL_SCOPE.md`, "the same day, the target
## image"). Read only -- `design/` is read-only and this probe never writes there.
const ALETSCH := "res://../../design/owner-references-2026-09-12/lod-zoom-target-aletsch-sentinel2.jpg"

## The pan tests run at a zoom deep enough that the pyramid is certainly up:
## `_update_lod()` gates on `_zoom > LOD_AUTO_ZOOM` (2.2) AND
## `screen_px_per_cell > 1.0`, and every zoom-16 figure already recorded for
## this subsystem (`LOD_DETAIL_SCOPE.md`'s 0.0083 and its "at zoom 16" bar) is
## at this value, so the pan numbers land beside them rather than beside nothing.
const PAN_ZOOM := 16.0

## Screen pixels of pan per frame, and how many frames the pan tests run for.
const PAN_PX := 6.0
const PAN_FRAMES := 120

## `_update_lod()`'s own two entry gates, restated so the report can say which
## one held a frame out rather than reporting "LOD off" with no reason.
const LOD_AUTO_ZOOM := 2.2
const LOD_PX_PER_CELL_THRESHOLD := 1.0

## Every arm of `ViewportHost.set_layer_visible`'s own `match`, enumerated from
## the definition rather than from memory (`MISTAKES.md`, "Add a capability, or
## write a staleness/cache key" -- derive the list from the definition, every
## match arm). An overlay left on contaminates metric 1: a settlement pin is
## re-scaled by `map_overlay.gd`'s `_civ_zoom_k()` on every zoom notch, which is
## a real screen change that no camera warp can undo, so it would read as a pop
## at every frame of the sweep.
const OVERLAY_LAYERS: PackedStringArray = [
	"territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts",
]

const WATCHDOG_S := 3600

var _fail: Array[String] = []
var _app: Node
var _vh: Control
var _br: Node
var _m: Object          ## `LodSweepMetrics`, the Rust metric calculator.
var _out := "user://lodsweep/"
var _seeds: Array[int] = []
var _grids: Array[Vector2i] = []
var _vp := DEF_VP
var _width_km := DEF_WIDTH_KM
var _zoom_step := DEF_ZOOM_STEP
var _tests: Array[String] = []
var _want_dark := true
var _plant := false
var _sheet := true
var _report: Dictionary = {}


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server -- `frame_post_draw` never fires and")
		printerr("  `get_texture().get_image()` is null, so every metric would be vacuous.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(WATCHDOG_S).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog fired after %d s." % WATCHDOG_S)
		get_tree().quit(2))

	## The stale-`.dll` detector, and the reason it is worth its four lines:
	## `cargo test` does not rebuild `target/debug/cartalith_godot.dll`, so a
	## probe run after a `.rs` edit and before `cargo build` tests the previous
	## engine (`MISTAKES.md`, "Grade a Godot probe as evidence for a Rust
	## change"). If this class is missing, the loaded binary predates this
	## milestone and nothing below would mean what it says.
	if not ClassDB.class_exists("LodSweepMetrics"):
		printerr("PROBE-CANNOT-RUN: the loaded extension has no `LodSweepMetrics`.")
		printerr("  The .dll predates `lod_sweep.rs` -- run `cargo build` and try again.")
		get_tree().quit(2)
		return
	_m = ClassDB.instantiate("LodSweepMetrics")

	DisplayServer.window_set_size(_vp)
	await get_tree().process_frame

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _vh == null or _br == null:
		printerr("PROBE-CANNOT-RUN: shell exposed no viewport/bridge.")
		get_tree().quit(2)
		return
	## The welcome sheet covers the map on boot. Hiding it is its own
	## "Continue without a world" link, and skipping this step measures the
	## DIALOG -- a whole-frame detail measurement once reported the same value
	## to fourteen digits at every zoom for exactly this reason.
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await get_tree().process_frame

	if not await _force_palette():
		get_tree().quit(2)
		return

	DirAccess.make_dir_recursive_absolute(_out)
	print("out: ", _out)
	print("palette forced: %s   window: %dx%d" % ["dark" if _want_dark else "light", _vp.x, _vp.y])

	if _plant:
		await _run_plants()
	else:
		await _run_baseline()

	_write_json()
	for f in _fail:
		printerr("ASSERT-FAILED: ", f)
	print("\nPROBE-RESULT: ", "PASS" if _fail.is_empty() else "FAIL")
	get_tree().quit(0 if _fail.is_empty() else 1)


# ── arguments ────────────────────────────────────────────────────────────────

func _parse_args() -> bool:
	_seeds.assign(Array(DEF_SEEDS))
	_grids.assign(_parse_grids(DEF_GRIDS))
	_tests.assign(Array(DEF_TESTS.split(",")))
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		var has_next: bool = i + 1 < args.size()
		match a:
			"--out":
				if not has_next: return _argfail(a)
				_out = args[i + 1]
				if not _out.ends_with("/"): _out += "/"
				i += 1
			"--seeds":
				if not has_next: return _argfail(a)
				_seeds.clear()
				for s in args[i + 1].split(","):
					_seeds.append(int(s))
				i += 1
			"--grids":
				if not has_next: return _argfail(a)
				_grids.assign(_parse_grids(args[i + 1]))
				i += 1
			"--vp":
				if not has_next: return _argfail(a)
				var g := _parse_grids(args[i + 1])
				if g.is_empty(): return _argfail(a)
				_vp = g[0]
				i += 1
			"--width-km":
				if not has_next: return _argfail(a)
				_width_km = float(args[i + 1])
				i += 1
			"--zoom-step":
				if not has_next: return _argfail(a)
				_zoom_step = float(args[i + 1])
				i += 1
			"--tests":
				if not has_next: return _argfail(a)
				_tests.assign(Array(args[i + 1].split(",")))
				i += 1
			"--theme":
				if not has_next: return _argfail(a)
				if args[i + 1] not in ["dark", "light"]: return _argfail(a)
				_want_dark = args[i + 1] == "dark"
				i += 1
			"--plant": _plant = true
			"--no-sheet": _sheet = false
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument '%s'. See this file's header." % a)
				return false
		i += 1
	if _seeds.is_empty() or _grids.is_empty() or _tests.is_empty():
		printerr("PROBE-CANNOT-RUN: --seeds/--grids/--tests cannot be empty.")
		return false
	for t in _tests:
		if t not in ["sweep", "pan", "zoompan"]:
			printerr("PROBE-CANNOT-RUN: unknown test '%s'." % t)
			return false
	if _zoom_step <= 1.0:
		printerr("PROBE-CANNOT-RUN: --zoom-step must exceed 1.0 (got %f)." % _zoom_step)
		return false
	return true


func _argfail(flag: String) -> bool:
	printerr("PROBE-CANNOT-RUN: '%s' needs a value." % flag)
	return false


func _parse_grids(s: String) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for part in s.split(","):
		var xy := part.strip_edges().split("x")
		if xy.size() == 2:
			out.append(Vector2i(int(xy[0]), int(xy[1])))
	return out


## Forces the palette in memory and REFUSES to run if the forcing did not take
## (`MISTAKES.md`, "Assert on pixels": naming the palette in prose does not
## select it -- the harness must force it and refuse otherwise).
##
## `DccSettings.set_theme_mode()` is deliberately NOT called: it writes
## `user://cartalith_settings.cfg`, and a measurement probe that mutates the
## user's stored preferences is a side effect nobody asked for.
func _force_palette() -> bool:
	var was: bool = DccTheme.is_dark()
	if was != _want_dark:
		DccTheme.apply_theme(_want_dark)
		var shell := _find_with_method(_app, "rebuild_theme")
		if shell != null:
			shell.rebuild_theme(was)
		await get_tree().process_frame
	if DccTheme.is_dark() != _want_dark:
		printerr("PROBE-CANNOT-RUN: palette forcing did not take (wanted dark=%s)." % _want_dark)
		return false
	return true


func _find_with_method(root: Node, m: String) -> Node:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.has_method(m):
			return n
		for c in n.get_children(true):
			stack.append(c)
	return null


# ── the baseline run ─────────────────────────────────────────────────────────

func _run_baseline() -> void:
	var runs: Array = []
	for grid in _grids:
		for seed_v in _seeds:
			if not await _load_world(seed_v, grid):
				continue
			var pivot := await _pick_pivot()
			var ladder: Dictionary = await _zoom_ladder(seed_v, grid, pivot)
			runs.append(ladder)
			for t in _tests:
				var r: Dictionary = await _run_test(t, seed_v, grid, pivot)
				runs.append(r)
				_print_run(r)
			if _sheet:
				var sheet: Dictionary = await _aletsch_sheet(seed_v, grid)
				runs.append({"test": "aletsch", "seed": seed_v, "grid": "%dx%d" % [grid.x, grid.y], "sheet": sheet})
	_report["runs"] = runs
	_report["synthesis_us"] = await _time_synthesis()
	_summarise(runs)


func _load_world(seed_v: int, grid: Vector2i) -> bool:
	print("\n================ seed %d   grid %dx%d ================" % [seed_v, grid.x, grid.y])
	var t0 := Time.get_ticks_msec()
	_br.generate({
		"seed": seed_v, "width_km": _width_km, "grid_w": grid.x, "grid_h": grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if _br.generating:
		_fail.append("generation never finished for seed %d at %dx%d" % [seed_v, grid.x, grid.y])
		return false
	await get_tree().create_timer(1.0).timeout
	for l in OVERLAY_LAYERS:
		_vh.set_layer_visible(l, false)
	await _settle()
	print("  generated in %.1f s; overlays hidden; zoom_max = %.1f"
		% [(Time.get_ticks_msec() - t0) / 1000.0, _vh._zoom_max])
	return true


## The sweep's fixed pivot: the highest-relief patch of the fit-zoom frame.
##
## Derived from the picture rather than from a hand-picked cell, so it moves
## with the seed instead of landing on ocean for two worlds out of three. The
## statistic is the same adjacent-pixel luma step metric 4 uses, blocked 48 px
## at a time -- ocean is flat and scores near zero, so the winner is land with
## structure in it, which is what "high relief" has to mean for a harness that
## grades screen detail.
func _pick_pivot() -> Vector2i:
	_vh.reset_view()
	await _settle()
	var cap := await _capture()
	if not cap.get("ok", false):
		return Vector2i(-1, -1)
	var img: Image = cap["img"]
	var g: Vector2i = _br.grid_size()
	var map: Rect2 = _to_crop(_vh._map_display_rect(), cap)
	var best := -1.0
	var best_px := Vector2(-1, -1)
	var blk := 48
	var y := int(maxf(map.position.y, 0.0))
	while y + blk < mini(img.get_height(), int(map.end.y)):
		var x := int(maxf(map.position.x, 0.0))
		while x + blk < mini(img.get_width(), int(map.end.x)):
			var acc := 0.0
			for yy in range(y, y + blk, 4):
				var prev := _lum(img.get_pixel(x, yy))
				for xx in range(x + 1, x + blk):
					var l := _lum(img.get_pixel(xx, yy))
					acc += absf(l - prev)
					prev = l
			if acc > best:
				best = acc
				best_px = Vector2(x + blk * 0.5, y + blk * 0.5)
			x += blk
		y += blk
	if best_px.x < 0.0 or map.size.x <= 0.0:
		return Vector2i(-1, -1)
	var cell := Vector2i(
		int((best_px.x - map.position.x) / map.size.x * g.x),
		int((best_px.y - map.position.y) / map.size.y * g.y))
	print("  pivot cell %d,%d (block relief %.3f)" % [cell.x, cell.y, best])
	return cell


## **LOD-D2's and LOD-D3's acceptance numbers, measured directly.**
##
## The sweep above records a series; these are the specific readings the later
## milestones are graded on, taken at the zooms their bars name:
##
##   - *"Detail per screen pixel at zoom 16 and 40 is >= 50 % of the zoom-1
##     value (about 8 % today)"* -- the ladder below.
##   - *"Hiding the LOD layer at zoom 16 moves mean |dL| by >= 10x the 0.0083
##     baseline"* -- `layer_on_off_dluma` at zoom 16.
##   - *"At the LOD entry frame, mean |dL*| between the layer shown and hidden
##     is <= 2.0"* -- `entry_dlstar`.
##
## **Two framings are reported for detail, and the difference is not cosmetic.**
## `detail_window` measures the WHOLE WINDOW, which is what `_mapsharp_probe`
## did, so it is the figure comparable with the 0.0220 -> 0.0017 already in
## `LOD_DETAIL_SCOPE.md`. `detail_map` measures the map crop only. The central
## 30-70 % box lands on different ground in each, so the two are different
## numbers for the same frame and quoting one against the other's baseline
## would be a silent re-base.
##
## The zoom is reached with `zoom_step()`, i.e. `_zoom_at(size * 0.5, f)` --
## the same call the navpad's +/- makes, and the same one `_mapsharp_probe`
## used to produce the recorded figures.
func _zoom_ladder(seed_v: int, grid: Vector2i, pivot: Vector2i) -> Dictionary:
	print("\n  -- zoom ladder (LOD-D2's own acceptance readings) --")
	_vh.reset_view()
	if pivot.x >= 0:
		_vh.move_view_to(float(pivot.x), float(pivot.y))
	await _settle()

	var g: Vector2i = _br.grid_size()
	var native := minf(_vh.size.x / float(g.x), _vh.size.y / float(g.y))
	var rows: Array = []
	var entry: Dictionary = {}

	## LOD entry first, walking UP from the bottom the way a user does, so the
	## frame that is measured is the one the layer actually came up on.
	await _zoom_to(1.0)
	var guard := 0
	while not _vh.lod_active() and _vh.zoom() < _vh._zoom_max - 1e-6 and guard < 2000:
		_vh._zoom_at(_vh.size * 0.5, _zoom_step)
		await get_tree().process_frame
		guard += 1
	if _vh.lod_active():
		for i in 30:
			await RenderingServer.frame_post_draw
		entry = await _layer_on_off()
		entry["zoom"] = _vh.zoom()
		print("     LOD entry at zoom %.2f: mean |dL*| layer shown vs hidden = %s   (D2 bar: <= 2.0)"
			% [entry["zoom"], "%.4f" % entry["dlstar"] if entry.has("dlstar") else "-- not measured"])
	else:
		print("     LOD never became active on the way up -- entry reading omitted, not zeroed")

	for target: float in [1.0, 2.0, 4.0, 8.0, 16.0, 40.0]:
		if target > _vh._zoom_max:
			continue
		await _zoom_to(target)
		## Let the backlog drain: a half-built viewful is a real state, but it
		## is not the state a "detail at zoom 16" bar is about.
		for i in 45:
			await RenderingServer.frame_post_draw
		var full := await _capture_full()
		var cap := await _capture()
		if not full.get("ok", false) or not cap.get("ok", false):
			continue
		var row := {
			"zoom": _vh.zoom(),
			"px_per_cell": native * _vh.zoom(),
			"level": _br.lod_level_for_zoom(native * _vh.zoom()),
			"lod_active": _vh.lod_active(),
			"live_tiles": (_vh._lod_tiles as Dictionary).size(),
			"detail_window": _m.call("detail", full["data"], full["w"], full["h"]),
			"detail_map": _m.call("detail", cap["data"], cap["w"], cap["h"]),
		}
		if is_equal_approx(target, 16.0) or is_equal_approx(target, 40.0):
			var oo := await _layer_on_off()
			for k in oo.keys():
				row["layer_%s" % k] = oo[k]
		rows.append(row)
		print("     zoom %6.2f  px/cell %8.2f  level %2d  tiles %3d  detail window %s  map %s%s"
			% [row["zoom"], row["px_per_cell"], row["level"], row["live_tiles"],
				_num(row["detail_window"], "detail"), _num(row["detail_map"], "detail"),
				("  layer on/off |dL| %s" % _fnum(row.get("layer_dluma"))) if row.has("layer_dluma") else ""])

	var out := {"test": "ladder", "seed": seed_v, "grid": "%dx%d" % [grid.x, grid.y], "rows": rows}
	if not entry.is_empty():
		out["entry"] = entry
	## The D2 ratio, spelled out rather than left to the reader.
	var d1 = _row_detail(rows, 1.0)
	for z: float in [16.0, 40.0]:
		var dz = _row_detail(rows, z)
		if d1 != null and dz != null and float(d1) > 0.0:
			var frac: float = float(dz) / float(d1)
			out["detail_frac_z%d" % int(z)] = frac
			print("     detail at zoom %d is %.1f%% of the zoom-1 value   (D2 bar: >= 50%%)" % [int(z), frac * 100.0])
		else:
			print("     detail ratio at zoom %d: -- not both readings available" % int(z))
	return out


func _row_detail(rows: Array, zoom_target: float):
	for r: Dictionary in rows:
		if absf(float(r["zoom"]) - zoom_target) < 0.05 * zoom_target:
			var d = r["detail_window"]
			if d is Dictionary and (d as Dictionary).get("ok", false):
				return d["detail"]
	return null


## The deep-zoom layer's whole contribution to the picture at the current view:
## the frame with `_lod_layer` shown against the same frame with it hidden, in
## BOTH units the scope's bars are quoted in.
##
## `visible`, not `_lod_layer.modulate:a` -- which was a 0.15 s tween until
## LOD-D3 removed it, and a reading taken mid-tween would have been of an
## arbitrary alpha. `visible` is still the right switch for a different reason
## now: the layer's alpha is a straight 1, and what fades is each TILE's own
## morph against its parent, which is part of what this measurement is
## supposed to see rather than something to switch off.
func _layer_on_off() -> Dictionary:
	var on_full := await _capture_full()
	var on := await _capture()
	_vh._lod_layer.visible = false
	await _settle()
	var off_full := await _capture_full()
	var off := await _capture()
	_vh._lod_layer.visible = true
	await _settle()
	var out: Dictionary = {}
	if on_full.get("ok", false) and off_full.get("ok", false):
		var l: Dictionary = _m.call("luma_diff", on_full["data"], off_full["data"], on_full["w"], on_full["h"])
		if l.get("ok", false):
			out["dluma"] = l["mean_dluma"]
	if on.get("ok", false) and off.get("ok", false):
		var t: Dictionary = _m.call("temporal", on["data"], off["data"], on["w"], on["h"],
			on["campos"], on["zoom"], off["campos"], off["zoom"])
		if t.get("ok", false):
			out["dlstar"] = t["mean_dlstar"]
	return out


func _zoom_to(target: float) -> void:
	var cur: float = _vh.zoom()
	if cur <= 0.0:
		return
	_vh.zoom_step(target / cur)
	await _settle()


func _num(d, key: String) -> String:
	return ("%.5f" % float((d as Dictionary)[key])) if d is Dictionary and (d as Dictionary).get("ok", false) else "--"


func _fnum(v) -> String:
	return "%.5f" % float(v) if v != null else "--"


## One of the research note's Tests A-C, driven through the SHELL's own camera
## entry points -- `_zoom_at` (what the wheel handler calls) and the pan
## branch's `_camera.position += d; _update_lod()`. Deliberately not a shortcut
## past them: the whole point of the harness is that the numbers describe the
## path a real zoom takes.
func _run_test(kind: String, seed_v: int, grid: Vector2i, pivot: Vector2i) -> Dictionary:
	_vh.reset_view()
	if pivot.x >= 0:
		_vh.move_view_to(float(pivot.x), float(pivot.y))
	await _settle()

	if kind != "sweep":
		## The pan tests need the pyramid already up, so they walk the zoom in
		## through the same `_zoom_at` before starting to measure.
		while _vh.zoom() < PAN_ZOOM and _vh.zoom() < _vh._zoom_max:
			_vh._zoom_at(_vh.size * 0.5, _zoom_step)
			await get_tree().process_frame
		if pivot.x >= 0:
			_vh.move_view_to(float(pivot.x), float(pivot.y))
		await _settle()

	var ti: PackedFloat64Array = []
	var detail: PackedFloat64Array = []
	var seam_max: PackedFloat64Array = []
	var holes: PackedFloat64Array = []
	var holey: PackedInt32Array = []   ## Frame indices whose coverage was incomplete.
	var step_us: PackedFloat64Array = []
	var frame_us: PackedFloat64Array = []
	var zooms: PackedFloat64Array = []
	var frames_lod_on := 0
	var hole_frames := 0
	var entry_frame := -1
	var entry_step_us := -1.0
	var was_active: bool = _vh.lod_active()

	var levels: PackedInt32Array = []
	var g0: Vector2i = _br.grid_size()
	var native0 := minf(_vh.size.x / float(g0.x), _vh.size.y / float(g0.y))
	var prev_cap: Dictionary = {}
	var n := 0
	while true:
		if kind == "sweep" or kind == "zoompan":
			if _vh.zoom() >= _vh._zoom_max - 1e-6:
				if kind == "sweep":
					break
			else:
				_vh._zoom_at(_vh.size * 0.5, _zoom_step)
		if kind == "pan" or kind == "zoompan":
			_vh._camera.position += Vector2(-PAN_PX, -PAN_PX * 0.5)
			_vh._update_lod()
		if kind != "sweep" and n >= PAN_FRAMES:
			break
		if n > 4000:
			_fail.append("%s sweep did not terminate in 4000 frames" % kind)
			break

		var t_step := Time.get_ticks_usec()
		var t_frame := t_step
		await RenderingServer.frame_post_draw
		frame_us.append(float(Time.get_ticks_usec() - t_frame))

		var cap := await _capture()
		if not cap.get("ok", false):
			_fail.append("%s: the framebuffer could not be read at frame %d" % [kind, n])
			break
		zooms.append(cap["zoom"])
		levels.append(_br.lod_level_for_zoom(native0 * float(cap["zoom"])))

		var d: Dictionary = _m.call("detail", cap["data"], cap["w"], cap["h"])
		if d.get("ok", false):
			detail.append(d["detail"])

		if not prev_cap.is_empty():
			var t: Dictionary = _m.call("temporal",
				prev_cap["data"], cap["data"], cap["w"], cap["h"],
				prev_cap["campos"], prev_cap["zoom"], cap["campos"], cap["zoom"])
			if t.get("ok", false):
				ti.append(t["mean_dlstar"])

		var active: bool = _vh.lod_active()
		if active and not was_active and entry_frame < 0:
			entry_frame = n
		was_active = active
		if active:
			frames_lod_on += 1
			var tiles := _tile_rects(cap)
			var hr: Dictionary = _m.call("holes", cap["w"], cap["h"], _to_crop(_vh._map_display_rect(), cap), tiles["flat"])
			if hr.get("ok", false):
				holes.append(float(hr["holes"]))
				hole_frames += 1
				## **Which frames**, not just how many pixels.
				## `LOD_DETAIL_SCOPE.md` LOD-D3 asks for *"zero hole pixels
				## after the first built frame"*, and a max over every measured
				## frame cannot answer that: the frames right after the pyramid
				## comes up are a viewful arriving through
				## `MAX_LOD_TILES_PER_UPDATE` and the backlog, and they are the
				## ones the bar's own wording sets aside. Recording the indices
				## lets the reading say *where* the uncovered ground was rather
				## than only that there was some.
				if float(hr["holes"]) > 0.0:
					holey.append(n)
			var sr: Dictionary = _m.call("seam", cap["data"], cap["w"], cap["h"], tiles["cols"])
			if sr.get("ok", false):
				seam_max.append(sr["max"])

		prev_cap = cap
		step_us.append(float(Time.get_ticks_usec() - t_step))
		if entry_frame == n and entry_step_us < 0.0:
			entry_step_us = step_us[step_us.size() - 1]
		n += 1

	var p: Dictionary = _m.call("pops", ti)
	var out := {
		"test": kind, "seed": seed_v, "grid": "%dx%d" % [grid.x, grid.y],
		"frames": n, "frames_lod_on": frames_lod_on,
		"zoom_from": zooms[0] if zooms.size() > 0 else null,
		"zoom_to": zooms[zooms.size() - 1] if zooms.size() > 0 else null,
		"ti": _m.call("stats", ti),
		"pops": p,
		"detail": _m.call("stats", detail),
		"detail_first": detail[0] if detail.size() > 0 else null,
		"detail_last": detail[detail.size() - 1] if detail.size() > 0 else null,
		"frame_us": _m.call("stats", frame_us),
		"step_us": _m.call("stats", step_us),
	}
	## Absent, not zero: a run where the pyramid never came up has no seam and
	## no hole measurement, and a `0` there would pass every later bar for free.
	if seam_max.size() > 0:
		out["seam_max"] = _m.call("stats", seam_max)
	if hole_frames > 0:
		out["holes"] = _m.call("stats", holes)
		out["hole_frames"] = hole_frames
		out["holey_frames"] = holey.size()
		if holey.size() > 0:
			out["holey_first"] = holey[0]
			out["holey_last"] = holey[holey.size() - 1]
	if entry_frame >= 0:
		out["lod_entry_frame"] = entry_frame
		out["lod_entry_step_us"] = entry_step_us
	var lb := _level_boundary_ratio(ti, levels)
	if not lb.is_empty():
		out["level_boundary"] = lb
	return out


## **LOD-D3's second acceptance criterion**, measured: *"the worst `T_i` at any
## level boundary is <= 1.5x the median of the non-boundary frames on either
## side."*
##
## Strictly more discriminating than the pop count, and that is why it is here
## rather than left implicit. A pop needs `T_i` to clear 3x the trailing median
## AND 1.0 L\*; a smooth 1.02x sweep already carries a ~1.3 L\* per-frame
## residual on this build, so the pop bar sits near 4 L\* and a level change
## worth half that passes it. This ratio compares a boundary frame against its
## OWN neighbourhood, so it sees a change the pop rule cannot.
##
## `ti[j]` is the transition from frame `j` to frame `j+1`, so a level change
## observed at frame `i` is `ti[i-1]`. The reference neighbourhood is the eight
## frames either side that are not themselves boundaries; a boundary with fewer
## than four such frames is skipped rather than scored against a thin reference.
## `{}` when the run crossed no level boundary at all -- which is a fact about
## the run, not a ratio of zero.
func _level_boundary_ratio(ti: PackedFloat64Array, levels: PackedInt32Array) -> Dictionary:
	var edges: Array[int] = []
	for i in range(1, levels.size()):
		if levels[i] != levels[i - 1]:
			edges.append(i)
	if edges.is_empty() or ti.is_empty():
		return {}
	var is_edge: Dictionary = {}
	for e in edges:
		is_edge[e] = true
	var worst := -1.0
	var worst_at := -1
	var scored := 0
	for e in edges:
		var j: int = e - 1
		if j < 0 or j >= ti.size():
			continue
		var refs := PackedFloat64Array()
		for d in range(1, 9):
			for k in [e - d, e + d]:
				if k >= 1 and k - 1 < ti.size() and not is_edge.has(k):
					refs.append(ti[k - 1])
		if refs.size() < 4:
			continue
		var st: Dictionary = _m.call("stats", refs)
		if not st.get("ok", false) or float(st["median"]) <= 0.0:
			continue
		scored += 1
		var r: float = ti[j] / float(st["median"])
		if r > worst:
			worst = r
			worst_at = e
	if scored == 0:
		return {}
	return {"worst_ratio": worst, "at_frame": worst_at, "boundaries": edges.size(), "scored": scored}


func _print_run(r: Dictionary) -> void:
	print("\n  -- %s --  %d frames, %d with the pyramid up" % [r["test"], r["frames"], r["frames_lod_on"]])
	if r["zoom_from"] != null:
		print("     zoom %.2f -> %.2f" % [r["zoom_from"], r["zoom_to"]])
	print("     T_i (mean |dL*| per frame): %s" % _fmt(r["ti"]))
	var p: Dictionary = r["pops"]
	print("     pops: %d   (rule: T_i > %.1fx its trailing %d-frame median AND > %.1f L*)   judged %d, unjudged %d"
		% [(p["indices"] as PackedInt32Array).size(), p["factor"], p["window"], p["floor"], p["judged"], p["unjudged"]])
	print("     detail/screen px: %s   first %.4f -> last %.4f"
		% [_fmt(r["detail"]), r["detail_first"] if r["detail_first"] != null else NAN,
			r["detail_last"] if r["detail_last"] != null else NAN])
	print("     seam ratio (max per frame): %s" % (_fmt(r["seam_max"]) if r.has("seam_max") else "-- no scorable tile boundary in any frame"))
	print("     hole px per frame: %s" % (_fmt(r["holes"]) if r.has("holes") else "-- the pyramid was never up, so coverage is not defined"))
	if r.has("holey_frames"):
		## `LOD_DETAIL_SCOPE.md` LOD-D3's bar is *"zero hole pixels after the
		## first built frame"*, so the frames matter as much as the pixels:
		## the reading says how many frames were uncovered and over what span,
		## beside the LOD entry frame printed below, rather than leaving the
		## bar to be read off a maximum that includes the pyramid arriving.
		print("     frames with any hole: %d of %d measured   (first at frame %s, last at %s)"
			% [r["holey_frames"], r.get("hole_frames", 0),
				str(r.get("holey_first", "--")), str(r.get("holey_last", "--"))])
	print("     frame time us: %s" % _fmt(r["frame_us"]))
	print("     camera-step cost us (this is where a synthesis stall lands): %s" % _fmt(r["step_us"]))
	if r.has("lod_entry_frame"):
		print("     LOD entry at frame %d; that step cost %.0f us" % [r["lod_entry_frame"], r["lod_entry_step_us"]])
	else:
		print("     LOD never entered in this run")
	if r.has("level_boundary"):
		var lb: Dictionary = r["level_boundary"]
		print("     worst T_i at a pyramid-level boundary: %.2fx its own neighbourhood median (frame %d, %d boundaries, %d scored)   (D3 bar: <= 1.5x)"
			% [lb["worst_ratio"], lb["at_frame"], lb["boundaries"], lb["scored"]])
	else:
		print("     level-boundary ratio: -- this run crossed no pyramid-level boundary")


func _fmt(s) -> String:
	if s == null or not (s is Dictionary) or not (s as Dictionary).get("ok", false):
		return "-- %s" % [(s as Dictionary).get("reason", "no value") if s is Dictionary else "no value"]
	return "median %.4f (%.4f..%.4f, n=%d)" % [s["median"], s["min"], s["max"], s["n"]]


# ── capture and geometry ─────────────────────────────────────────────────────

## One framebuffer capture, cropped to `ViewportHost`'s own rect and carried
## with the camera state that produced it.
##
## The crop is what makes metric 1 mean anything: the docks and the app bar do
## not move with the camera, so including them would add a large constant
## |dL*| to every warped comparison and drown the thing being measured. Inside
## the crop, `ViewportHost`'s own contract `screen = _camera.position + local *
## zoom` holds with no further offset, because `_camera.position` is already
## measured from this node's origin (`viewport_host.gd::_zoom_at`'s own note on
## `global_position`).
func _capture() -> Dictionary:
	await RenderingServer.frame_post_draw
	var tex := get_viewport().get_texture()
	if tex == null:
		return {"ok": false}
	var img := tex.get_image()
	if img == null:
		return {"ok": false}
	var gp := _vh.global_position
	var want := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round()))
	var clipped := want.intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	if clipped.size.x < 8 or clipped.size.y < 8:
		return {"ok": false}
	var sub := img.get_region(clipped)
	if sub.get_format() != Image.FORMAT_RGBA8:
		sub.convert(Image.FORMAT_RGBA8)
	## The crop may have been clipped against the window; the camera origin
	## moves with it by exactly that much.
	var shift := Vector2(want.position - clipped.position)
	return {
		"ok": true, "img": sub, "data": sub.get_data(),
		"w": sub.get_width(), "h": sub.get_height(),
		"origin": Vector2(clipped.position), "shift": shift,
		"campos": _vh._camera.position + shift, "zoom": _vh.zoom(),
	}


## The WHOLE window, uncropped -- `_mapsharp_probe`'s own framing, kept so the
## detail figures in `LOD_DETAIL_SCOPE.md` remain comparable. Used only by the
## zoom ladder; the per-frame sweep uses the crop, because metric 1 needs the
## chrome out of the picture.
func _capture_full() -> Dictionary:
	await RenderingServer.frame_post_draw
	var tex := get_viewport().get_texture()
	if tex == null:
		return {"ok": false}
	var img := tex.get_image()
	if img == null:
		return {"ok": false}
	if img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	return {"ok": true, "data": img.get_data(), "w": img.get_width(), "h": img.get_height()}


## A `_camera`-local rect (what `_map_display_rect()` and `_lod_sprite_rect()`
## both return) in the capture's own pixel space.
func _to_crop(local: Rect2, cap: Dictionary) -> Rect2:
	var z: float = cap["zoom"]
	var o: Vector2 = cap["campos"]
	return Rect2(o + local.position * z, local.size * z)


## Every live tile's rect in capture space, plus the screen columns its left
## edges fall on -- metric 2's boundaries.
##
## Read off the live `Sprite2D`s exactly as `viewport_host.gd::_lod_sprite_rect`
## does, and for its stated reason: recomputing them would be a second
## implementation of the half-texel inset that function's own comment exists to
## justify, and a harness that disagrees with the tiles it is measuring is worse
## than no harness.
func _tile_rects(cap: Dictionary) -> Dictionary:
	var flat := PackedFloat32Array()
	var cols := PackedInt32Array()
	var seen: Dictionary = {}
	for key in (_vh._lod_tiles as Dictionary).keys():
		var s: Sprite2D = _vh._lod_tiles[key]
		if s == null or s.texture == null:
			continue
		var local := Rect2(s.position, s.texture.get_size() * s.scale)
		var r := _to_crop(local, cap)
		flat.append_array(PackedFloat32Array([r.position.x, r.position.y, r.size.x, r.size.y]))
		var c := int(round(r.position.x))
		if c > 0 and c < int(cap["w"]) and not seen.has(c):
			seen[c] = true
			cols.append(c)
	return {"flat": flat, "cols": cols}


func _settle() -> void:
	for i in 3:
		await RenderingServer.frame_post_draw


func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b


# ── metric 5 · tile synthesis timing ─────────────────────────────────────────

## Times `EngineBridge.lod_synthesize_tile` -- the SAME entry point
## `viewport_host.gd::_build_lod_tile` calls, on its first line, for every tile
## the shell ever draws.
##
## Measured here rather than by instrumenting the shell, because LOD-D0 is
## "measurement only, no pixel change" and adding a timer inside `_build_lod_tile`
## would be an edit to a shipping file for a probe's benefit. The cost is one
## disclosed gap: this misses the `ImageTexture` upload and the `Sprite2D`
## placement that `_build_lod_tile` does around the call. The camera-step cost
## in each run above is the figure that includes them.
func _time_synthesis() -> Dictionary:
	print("\n== metric 5: tile synthesis, timed alone ==")
	var out: Dictionary = {}
	for z in [4, 6, 8, 10]:
		var n: int = _br.lod_tiles_per_axis(z)
		if n <= 0:
			continue
		var samples := PackedFloat64Array()
		for i in 8:
			var col: int = (n * (2 * i + 1)) / 16
			var row: int = (n * (2 * ((i + 3) % 8) + 1)) / 16
			var t0 := Time.get_ticks_usec()
			var tex: Texture2D = _br.lod_synthesize_tile(z, col, row)
			var dt := float(Time.get_ticks_usec() - t0)
			if tex != null:
				samples.append(dt)
		if samples.size() > 0:
			var s: Dictionary = _m.call("stats", samples)
			out["z%d" % z] = s
			print("  z%-2d (%d tiles/axis): %s us" % [z, n, _fmt(s)])
		else:
			print("  z%-2d: no tile synthesised -- not timed" % z)
	if out.is_empty():
		_fail.append("metric 5 measured no synthesis at all (teeth check)")
	return out


# ── the Aletsch comparison sheet ─────────────────────────────────────────────

## `LOD_DETAIL_SCOPE.md`'s second D0 deliverable, built as far as today's fields
## allow and NO further.
##
## **What it does measure, from the engine's own shipped rule.** The scope names
## the rule that decides snow today: `material_weights`' snow term is
## `smoothstep(3, -5, t)` on temperature, *"in effect a temperature cutoff at
## grid resolution"*. That is computable from `sample_cell`'s `temperature_c`
## right now, so this sheet records it per elevation band, measures its 10-90 %
## transition against local relief, and correlates it with slope and with
## aspect -- the three numbers LOD-D4's acceptance is written in. Today's
## figures are expected to be BAD (a pure temperature function cannot correlate
## with aspect), and that is the point: D4 has to move them.
##
## **What it deliberately does not measure.** Ice and glacier fractions. The
## scope's own gap table says *"No ice or glacier field exists anywhere; the
## kernel keeps no mask"*, so there is nothing to read and a colour-classifier
## over rendered pixels would be an invented number wearing a measurement's
## clothes. The sheet records the absence with that reason instead
## (`MISTAKES.md`: never encode "no value" as a plausible value, and a wrong
## reason is worse than none).
func _aletsch_sheet(seed_v: int, grid: Vector2i) -> Dictionary:
	print("\n== Aletsch comparison sheet ==")
	var g: Vector2i = _br.grid_size()
	## The most glaciated ground this world has: the coldest high cell. Searched
	## on a lattice so the cost does not scale with the grid.
	var stride: int = maxi(1, g.x / 96)
	var best_score := -1e30
	var best := Vector2i(-1, -1)
	var y := 0
	while y < g.y:
		var x := 0
		while x < g.x:
			var s: Dictionary = _br.sample_cell(x, y)
			if not s.is_empty() and s.get("water", "land") == "land":
				var score: float = float(s.get("elevation", 0.0)) * 100.0 - float(s.get("temperature_c", 99.0))
				if score > best_score:
					best_score = score
					best = Vector2i(x, y)
			x += stride
		y += stride
	if best.x < 0:
		print("  -- no land cell found; the sheet is not built for this world")
		return {"ok": false, "reason": "no land cell"}

	## A view about 25 km across, as the scope asks.
	var km_per_cell: float = _width_km / float(g.x)
	var half: int = maxi(4, int(12.5 / maxf(km_per_cell, 1e-6)))
	var cells: Array = []
	for yy in range(maxi(0, best.y - half), mini(g.y, best.y + half)):
		for xx in range(maxi(0, best.x - half), mini(g.x, best.x + half)):
			var s: Dictionary = _br.sample_cell(xx, yy)
			if s.is_empty() or s.get("water", "land") != "land":
				continue
			cells.append(s)
	if cells.size() < 64:
		print("  -- fewer than 64 land cells in the 25 km view; not built")
		return {"ok": false, "reason": "view holds %d land cells" % cells.size()}

	## `material_weights`' own snow term, quoted from `LOD_DETAIL_SCOPE.md`'s
	## gap table: `smoothstep(3, -5, t)`. Reproduced, not improved -- this is a
	## baseline of what ships, and "fixing" it here would measure something the
	## renderer does not do.
	var snow: PackedFloat64Array = []
	var elev: PackedFloat64Array = []
	var slope: PackedFloat64Array = []
	var aspect_n: PackedFloat64Array = []   ## northness, cos(aspect)
	for s: Dictionary in cells:
		snow.append(smoothstep(3.0, -5.0, float(s.get("temperature_c", 0.0))))
		elev.append(float(s.get("elevation", 0.0)))
		slope.append(float(s.get("slope_deg", 0.0)))
		aspect_n.append(cos(deg_to_rad(float(s.get("aspect_deg", 0.0)))) if s.has("aspect_deg") else 0.0)

	var lo: float = elev[0]
	var hi: float = elev[0]
	for e in elev:
		lo = minf(lo, e)
		hi = maxf(hi, e)
	var relief: float = hi - lo

	## The 10-90 % snow transition, as a fraction of local relief -- D4's bar is
	## ">= 15%".
	##
	## **A view that is entirely above or entirely below the snowline has no
	## transition, and that is NOT a transition of width zero.** The first
	## version of this block reported `0.0%` for exactly that case -- two of the
	## six worlds in the first baseline run -- which reads as a maximally sharp
	## snowline and would have been recorded as the worst possible D4 baseline
	## when the truth is that the measurement does not apply to that view. Both
	## ends must actually be present in the sample before the width means
	## anything (`MISTAKES.md`: never encode "no value" as a plausible value).
	var has_bare := false
	var has_full := false
	for s in snow:
		if s < 0.10:
			has_bare = true
		if s > 0.90:
			has_full = true
	var band: Variant = null
	var band_reason := ""
	if not has_bare or not has_full:
		band_reason = "the whole view is %s the snowline, so there is no transition in it" \
			% ("above" if not has_bare else "below")
	elif relief <= 0.0:
		band_reason = "the view has no local relief to measure the transition against"
	else:
		var e10 = _elev_at_snow(elev, snow, 0.10)
		var e90 = _elev_at_snow(elev, snow, 0.90)
		if e10 == null or e90 == null:
			band_reason = "the snow fraction never crosses one of the two thresholds"
		else:
			band = absf(float(e90) - float(e10)) / relief

	var bands: Array = []
	for b in 5:
		var b0: float = lo + relief * b / 5.0
		var b1: float = lo + relief * (b + 1) / 5.0
		var acc := 0.0
		var n := 0
		for i in elev.size():
			if elev[i] >= b0 and (elev[i] < b1 or b == 4):
				acc += snow[i]
				n += 1
		if n > 0:
			bands.append({"band": b, "elev_lo": b0, "elev_hi": b1, "cells": n, "snow_frac": acc / n})

	var sheet := {
		"ok": true,
		"centre_cell": [best.x, best.y],
		"km_across": 2.0 * half * km_per_cell,
		"land_cells": cells.size(),
		"relief": relief,
		"snow_transition_frac_of_relief": band,
		"snow_transition_reason": band_reason if band == null else "",
		"snow_vs_slope_r": _pearson(snow, slope),
		"snow_vs_northness_r": _pearson(snow, aspect_n),
		"bands": bands,
		## Not measured, and why. See this function's own doc comment.
		"ice_fraction": null,
		"ice_fraction_reason": "no ice or glacier field exists in this engine (LOD_DETAIL_SCOPE.md's gap table); LOD-D4 builds it",
	}
	print("  centre cell %d,%d, %.1f km across, %d land cells, relief %.4f"
		% [best.x, best.y, sheet["km_across"], cells.size(), relief])
	print("  snow 10-90%% transition: %s   (D4 bar: >= 15%% of local relief)"
		% (("%.1f%% of local relief" % (float(band) * 100.0)) if band != null else ("-- " + band_reason)))
	print("  snow vs slope r = %s   snow vs northness r = %s   (D4 bar: |r| >= 0.2 on aspect)"
		% [_rfmt(sheet["snow_vs_slope_r"]), _rfmt(sheet["snow_vs_northness_r"])])
	for b: Dictionary in bands:
		print("    band %d  elev %.3f..%.3f  %5d cells  snow %.3f" % [b["band"], b["elev_lo"], b["elev_hi"], b["cells"], b["snow_frac"]])
	print("  ice fraction: -- %s" % sheet["ice_fraction_reason"])

	await _side_by_side(seed_v, grid, best)
	return sheet


func _rfmt(v) -> String:
	return "%.3f" % float(v) if v != null else "-- undefined (a constant input has no correlation)"


## The elevation at which the snow fraction first crosses `f`, by scanning the
## cells in elevation order. `null` when the view never crosses it -- which is
## itself a finding for a snow rule that is supposed to have a transition.
func _elev_at_snow(elev: PackedFloat64Array, snow: PackedFloat64Array, f: float):
	var idx: Array = []
	for i in elev.size():
		idx.append(i)
	idx.sort_custom(func(a, b) -> bool: return elev[a] < elev[b])
	for i in idx:
		if snow[i] >= f:
			return elev[i]
	return null


func _pearson(a: PackedFloat64Array, b: PackedFloat64Array):
	var n := a.size()
	if n < 3 or b.size() != n:
		return null
	var ma := 0.0
	var mb := 0.0
	for i in n:
		ma += a[i]
		mb += b[i]
	ma /= n
	mb /= n
	var num := 0.0
	var da := 0.0
	var db := 0.0
	for i in n:
		var x := a[i] - ma
		var y := b[i] - mb
		num += x * y
		da += x * x
		db += y * y
	if da <= 0.0 or db <= 0.0:
		return null   ## A constant input has no correlation; 0.0 would be a lie.
	return num / sqrt(da * db)


## The owner image and the render, side by side, written to `--out`.
func _side_by_side(seed_v: int, grid: Vector2i, centre: Vector2i) -> void:
	_vh.reset_view()
	await _settle()
	var g: Vector2i = _br.grid_size()
	## Zoom until roughly 25 km spans the viewport.
	var want_cells: float = 25.0 / (_width_km / float(g.x))
	while _vh.zoom() < _vh._zoom_max - 1e-6:
		var native := minf(_vh.size.x / float(g.x), _vh.size.y / float(g.y))
		if float(_vh.size.x) / (native * _vh.zoom()) <= want_cells:
			break
		_vh._zoom_at(_vh.size * 0.5, _zoom_step)
		await get_tree().process_frame
	_vh.move_view_to(float(centre.x), float(centre.y))
	await _settle()
	## Drain the tile backlog so the sheet shows a filled view, not a race.
	for i in 60:
		await RenderingServer.frame_post_draw
	var cap := await _capture()
	if not cap.get("ok", false):
		print("  -- the render could not be captured; no sheet image written")
		return
	var render: Image = cap["img"]
	var ref: Image = null
	if ResourceLoader.exists(ALETSCH):
		var t := load(ALETSCH) as Texture2D
		if t != null:
			ref = t.get_image()
	if ref == null:
		var f := FileAccess.open(ALETSCH, FileAccess.READ)
		if f != null:
			var im := Image.new()
			if im.load_jpg_from_buffer(f.get_buffer(f.get_length())) == OK:
				ref = im
	var name := "%s/aletsch_seed%d_%dx%d" % [_out.trim_suffix("/"), seed_v, grid.x, grid.y]
	render.save_png("%s_render.png" % name)
	if ref == null:
		print("  -- the owner image did not load from %s; the render alone was written" % ALETSCH)
		print("     wrote %s_render.png" % name)
		return
	var h := maxi(render.get_height(), ref.get_height())
	var scaled := Image.new()
	scaled.copy_from(ref)
	scaled.resize(int(ref.get_width() * float(h) / ref.get_height()), h, Image.INTERPOLATE_BILINEAR)
	if scaled.get_format() != Image.FORMAT_RGBA8:
		scaled.convert(Image.FORMAT_RGBA8)
	var sheet := Image.create(render.get_width() + scaled.get_width(), h, false, Image.FORMAT_RGBA8)
	sheet.blit_rect(render, Rect2i(0, 0, render.get_width(), render.get_height()), Vector2i.ZERO)
	sheet.blit_rect(scaled, Rect2i(0, 0, scaled.get_width(), scaled.get_height()), Vector2i(render.get_width(), 0))
	sheet.save_png("%s_sheet.png" % name)
	print("  wrote %s_sheet.png  (render left, owner image right)" % name)


# ── the planted-defect run ───────────────────────────────────────────────────

## `LOD_DETAIL_SCOPE.md`: *"a planted defect must be detected by every metric --
## for example, `_set_lod_active` without its tween must register a pop."*
##
## **Read as: every metric has a planted defect that it detects**, one defect
## per metric, not one defect that moves all five. Stated explicitly because the
## other reading is unsatisfiable: a sudden layer flip cannot move a synthesis
## timing, and a freed tile cannot move a frame's high-frequency energy in the
## direction a detail regression would. Picking the single-defect reading would
## have made four of the five checks impossible to perform.
##
## Each plant is made from OUTSIDE the shell -- no shipping file is edited, so
## the "no pixel change" condition on this milestone holds for the plant run too.
func _run_plants() -> void:
	## **LOD-D6.** The plant run mutates `_lod_tiles` from outside the shell
	## -- `_plant_holes` erases half its entries and `_plant_seam` nudges
	## sprites -- and then captures three frames later. Since LOD-D6 the
	## backlog drain can synthesise a replacement inside exactly that window
	## and undo the plant, so the positive control would measure the shell
	## healing rather than the metric detecting. `_lod_sync` puts synthesis
	## back on this thread, which is the premise these plants were written
	## against; it moves no pixel, only the moment a tile appears.
	##
	## **The SWEEP (`_run_sweep`) is deliberately left asynchronous** -- its
	## timings and pop counts are what LOD-D6 is graded on, and a sweep run
	## with this flag set would measure the build this milestone replaced.
	_vh._lod_sync = true
	if not await _load_world(_seeds[0], _grids[_grids.size() - 1]):
		return
	var pivot := await _pick_pivot()
	_vh.reset_view()
	if pivot.x >= 0:
		_vh.move_view_to(float(pivot.x), float(pivot.y))
	while _vh.zoom() < PAN_ZOOM and _vh.zoom() < _vh._zoom_max:
		_vh._zoom_at(_vh.size * 0.5, _zoom_step)
		await get_tree().process_frame
	for i in 60:
		await RenderingServer.frame_post_draw
	if not _vh.lod_active():
		_fail.append("the plant run needs the pyramid up at zoom %.1f and it is not" % PAN_ZOOM)
		return
	print("\n== planted defects: each metric against its own positive control ==")
	print("   pyramid up with %d live tiles at zoom %.2f" % [(_vh._lod_tiles as Dictionary).size(), _vh.zoom()])

	var plants: Dictionary = {}
	plants["pop"] = await _plant_pop()
	plants["holes"] = await _plant_holes()
	plants["seam"] = await _plant_seam()
	plants["detail"] = await _plant_detail()
	plants["timing"] = _plant_timing()
	_report["plants"] = plants


## Metric 1. A quiet pan, then one frame with the tile layer switched off --
## exactly the discontinuity the deep-zoom layer's fade exists to avoid. (That
## fade was `_set_lod_active`'s 0.15 s `modulate:a` tween when this was
## written; LOD-D3 replaced it with a per-tile morph against the parent level.
## The plant is unchanged either way: it is a whole layer vanishing for one
## frame, which no fade of either kind smooths.)
func _plant_pop() -> Dictionary:
	var ti := PackedFloat64Array()
	var prev: Dictionary = {}
	var flip := 24
	for i in 34:
		_vh._camera.position += Vector2(-1.0, 0.0)
		_vh._update_lod()
		if i == flip:
			_vh._lod_layer.visible = false
		elif i == flip + 1:
			_vh._lod_layer.visible = true
		await RenderingServer.frame_post_draw
		var cap := await _capture()
		if not cap.get("ok", false):
			break
		if not prev.is_empty():
			var t: Dictionary = _m.call("temporal", prev["data"], cap["data"], cap["w"], cap["h"],
				prev["campos"], prev["zoom"], cap["campos"], cap["zoom"])
			if t.get("ok", false):
				ti.append(t["mean_dlstar"])
		prev = cap
	_vh._lod_layer.visible = true
	await _settle()
	var p: Dictionary = _m.call("pops", ti)
	var hits: PackedInt32Array = p["indices"]
	print("  metric 1 (pop):   %d pop(s) at %s; series %s" % [hits.size(), hits, _fmt(_m.call("stats", ti))])
	_expect(hits.size() > 0, "metric 1 detects a layer flip planted mid-pan as a pop")
	return {"pops": hits.size(), "ti": _m.call("stats", ti)}


## Metric 3. Free half the live tiles behind the compositor's back -- the
## dropped-tile class M1 already shipped once.
func _plant_holes() -> Dictionary:
	var cap0 := await _capture()
	var t0 := _tile_rects(cap0)
	var before: Dictionary = _m.call("holes", cap0["w"], cap0["h"], _to_crop(_vh._map_display_rect(), cap0), t0["flat"])
	var keys: Array = (_vh._lod_tiles as Dictionary).keys()
	var killed := 0
	for i in keys.size():
		if i % 2 == 0:
			var s: Sprite2D = _vh._lod_tiles[keys[i]]
			_vh._lod_tiles.erase(keys[i])
			s.queue_free()
			killed += 1
	await _settle()
	var cap1 := await _capture()
	var t1 := _tile_rects(cap1)
	var after: Dictionary = _m.call("holes", cap1["w"], cap1["h"], _to_crop(_vh._map_display_rect(), cap1), t1["flat"])
	print("  metric 3 (holes): %d tiles freed -- holes %s -> %s"
		% [killed, before.get("holes", "n/a"), after.get("holes", "n/a")])
	_expect(before.get("ok", false) and after.get("ok", false)
		and int(after["holes"]) > int(before["holes"]),
		"metric 3 detects freed tiles as hole pixels (%s -> %s)" % [before.get("holes", "n/a"), after.get("holes", "n/a")])
	_vh._clear_lod_tiles()
	_vh._update_lod()
	for i in 60:
		await RenderingServer.frame_post_draw
	return {"before": before, "after": after, "freed": killed}


## Metric 2. Nudge **alternate tile columns** three pixels, so a tile no longer
## lines up with the one beside it -- which is what a seam IS.
##
## **Not every tile.** The first version of this plant shifted all of them and
## the metric correctly reported no new seam (max ratio 1.71 -> 1.28, measured):
## a uniform translation of the whole layer keeps every tile aligned with its
## neighbours and moves the boundary columns along with the content, so there is
## nothing discontinuous to find. The defect being planted has to be a RELATIVE
## misalignment, and the plant, not the metric, was wrong.
func _plant_seam() -> Dictionary:
	var cap0 := await _capture()
	var t0 := _tile_rects(cap0)
	var before: Dictionary = _m.call("seam", cap0["data"], cap0["w"], cap0["h"], t0["cols"])
	var moved := 0
	for key in (_vh._lod_tiles as Dictionary).keys():
		var s: Sprite2D = _vh._lod_tiles[key]
		## The key is `"%d,%d,%d" % [z, col, row]` (`viewport_host.gd`'s
		## `_update_lod`), so the tile's own COLUMN index decides whether it
		## moves -- every odd column steps away from its even neighbours and
		## both of its shared edges become discontinuous.
		var parts: PackedStringArray = str(key).split(",")
		if s != null and parts.size() == 3 and int(parts[1]) % 2 == 1:
			s.position += Vector2(3.0, 0.0)
			moved += 1
	await _settle()
	var cap1 := await _capture()
	var t1 := _tile_rects(cap1)
	var after: Dictionary = _m.call("seam", cap1["data"], cap1["w"], cap1["h"], t1["cols"])
	print("  metric 2 (seam):  %d of %d tiles (odd columns) nudged 3 px -- max ratio %s -> %s"
		% [moved, (_vh._lod_tiles as Dictionary).size(), before.get("max", "n/a"), after.get("max", "n/a")])
	_expect(before.get("ok", false) and after.get("ok", false)
		and float(after["max"]) > float(before["max"]),
		"metric 2 detects a 3 px RELATIVE tile displacement as a higher seam ratio (%s -> %s)"
			% [before.get("max", "n/a"), after.get("max", "n/a")])
	_vh._clear_lod_tiles()
	_vh._update_lod()
	for i in 60:
		await RenderingServer.frame_post_draw
	return {"before": before, "after": after, "moved": moved}


## Metric 4. Hide the deep-zoom layer, which is the whole of what the pyramid
## contributes to the picture at this zoom.
func _plant_detail() -> Dictionary:
	var on := await _capture()
	var d_on: Dictionary = _m.call("detail", on["data"], on["w"], on["h"])
	_vh._lod_layer.visible = false
	await _settle()
	var off := await _capture()
	var d_off: Dictionary = _m.call("detail", off["data"], off["w"], off["h"])
	_vh._lod_layer.visible = true
	await _settle()
	var delta: float = absf(float(d_on["detail"]) - float(d_off["detail"])) if d_on.get("ok", false) and d_off.get("ok", false) else 0.0
	print("  metric 4 (detail): layer on %.5f, off %.5f, |delta| %.5f"
		% [d_on.get("detail", NAN), d_off.get("detail", NAN), delta])
	_expect(d_on.get("ok", false) and d_off.get("ok", false) and delta > 0.0,
		"metric 4 moves when the deep-zoom layer is hidden (|delta| %.5f)" % delta)
	return {"on": d_on, "off": d_off, "delta": delta}


## Metric 5 has no plantable defect, and saying so is the honest answer.
##
## A timing measures how long real work took; there is no feature to remove that
## would make the number wrong rather than merely different, and "make it slower
## and check it got slower" tests the clock, not the metric. What CAN be
## asserted is the property a timing report needs to be worth printing: that it
## sampled something, and that its spread is real rather than a single value
## repeated -- which is exactly the failure mode `MISTAKES.md` records for
## single-sample timings.
func _plant_timing() -> Dictionary:
	var s: Dictionary = _report.get("synthesis_us", {}) as Dictionary
	var samples := PackedFloat64Array()
	var n: int = _br.lod_tiles_per_axis(8)
	for i in 8:
		var t0 := Time.get_ticks_usec()
		var tex: Texture2D = _br.lod_synthesize_tile(8, (n * (2 * i + 1)) / 16, n / 2)
		if tex != null:
			samples.append(float(Time.get_ticks_usec() - t0))
	var st: Dictionary = _m.call("stats", samples)
	print("  metric 5 (timing): z8 synthesis %s us   (no plantable defect -- see the probe's own note)" % _fmt(st))
	_expect(st.get("ok", false) and int(st["n"]) >= 4 and float(st["max"]) > float(st["min"]),
		"metric 5 sampled real synthesis with a real spread (teeth check, not a planted defect)")
	return st


func _expect(ok: bool, what: String) -> void:
	if ok:
		print("    ok   ", what)
	else:
		_fail.append(what)


func _summarise(runs: Array) -> void:
	print("\n================ BASELINE SUMMARY ================")
	var total_pops := 0
	var judged := 0
	var any_seam := false
	var any_hole := false
	for r: Dictionary in runs:
		if not r.has("pops"):
			continue
		total_pops += (r["pops"]["indices"] as PackedInt32Array).size()
		judged += int(r["pops"]["judged"])
		any_seam = any_seam or r.has("seam_max")
		any_hole = any_hole or r.has("holes")
	print("  pops: %d over %d judged frames, across %d runs" % [total_pops, judged, runs.size()])
	print("  (LOD-D3's bar is ZERO pops across the full sweep, the constant-zoom pan and zoom-while-pan.)")
	_expect(judged > 0, "the sweep judged at least one frame for pops (teeth check)")
	_expect(any_seam, "at least one run had a scorable tile boundary (teeth check on metric 2)")
	_expect(any_hole, "at least one run had the pyramid up, so coverage was measurable (teeth check on metric 3)")


func _write_json() -> void:
	var path := "%sbaseline.json" % _out
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		printerr("could not write %s" % path)
		return
	_report["meta"] = {
		"when": Time.get_datetime_string_from_system(),
		"vp": [_vp.x, _vp.y], "width_km": _width_km, "zoom_step": _zoom_step,
		"theme": "dark" if _want_dark else "light",
		"seeds": _seeds, "tests": _tests, "plant": _plant,
	}
	f.store_string(JSON.stringify(_report, "  "))
	f.close()
	print("\nwrote ", path)
