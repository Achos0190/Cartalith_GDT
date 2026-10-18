extends Node
## `GEOLOGY_FIRST_SCOPE.md` GF-6 §4.10 (2026-10-18): rock-styled deep-zoom detail,
## on the real shell. The styling (`render::rock_styled_tile`) is reached only
## through `WorldGen::lod_snapshot_inputs` -> `rock_detail_for_tiles`, which a
## unit test cannot drive; this drives it through the shell's own bindings and
## its own viewport.
##
## Steps, on one seed:
##   A. The app's own world (`geology_processes` off, set explicitly): the
##      styling must be inactive (`rock_lod_detail_active() == false`), and
##      flipping the view switch (`set_rock_lod_detail`) must move NO pixel at
##      x16 or x64 -- the identity half.
##   B. The same seed with `geology_processes` on: the styling must be active;
##      with the switch on and off, the x1 frame (the base map, not tiles) must
##      be identical, and at x16 and x64 the frames over the two targets must
##      differ in at least MIN_MOVED_SHARE of pixels -- the positive control.
## Targets are picked by INPUTS only, never by the value under test. Phase A (no
## rock detail to ask) reads `sample_cell`'s `rock`, `rock_beneath` and
## `slope_deg` on an 8-cell stride kept 140 cells from every edge: `granite` =
## the steepest cell exposing granite with no water within 24 cells; `cap` = the
## steepest cell exposing a bedded rock (limestone, sandstone, tuff) over a
## substrate. Phase B takes the engine's picks from `rock_lod_detail_stats`
## (the steepest inland granite and the steepest bedded cap-edge cell, the rule
## `tests/gf6_rock_ladder.rs` uses), or `--at` targets. Every capture is the map viewport only, overlays but rivers hidden
## and labels cleared (`_zoomdetail_probe.gd`'s method), written as
## `<phase>_<target>_x<zoom>_<on|off>.png` with `gf6lod.json`.
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf6lod_probe.tscn -- \
##       --out DIR [--seed 24601] [--grid 2048x1311] [--vp 1600x1000] [--at NAME:X,Y ...]
## An unknown argument exits 2.
##
## Result: one `GF6LOD  ok  |FAIL  <check>` line per check and a final
## `GF6LOD  PASS|FAIL  (<n> checks, <m> failure(s))`. Exit 0 on PASS, 1 on FAIL.
## `PROBE-CANNOT-RUN: headless` exits 2; on a build without `set_rock_lod_detail`
## (before GF-6 §4.10) it captures the switch-on frames only, prints
## `PROBE-CANNOT-RUN: no set_rock_lod_detail` and exits 2 -- those frames are for
## an identity comparison against a later build, never a pass.

const ZOOMS := [1.0, 16.0, 64.0]
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]
const BEDDED := ["Limestone", "Sandstone", "Tuff / volcaniclastic"]
## The share of a deep-zoom frame the switch must move on a processes-on world.
## A floor for "the styling reached the screen", not a quality bar: the Rust
## ladder (`tests/gf6_rock_ladder.rs`) moved 25-97 % of x16/x64 frames over
## these kinds of target, so 2 % fails only an inert switch.
const MIN_MOVED_SHARE := 0.02

var _out := ""
var _seed := 24601
var _grid := Vector2i(2048, 1311)
var _vp := Vector2i(1600, 1000)
## `--at NAME:X,Y` targets (repeatable), used in place of the probe's own picks
## in phase B -- e.g. `tests/gf6_rock_ladder.rs`' TARGETS line for the seed, so
## the shell frames the cells the Rust ladder framed. Each is printed with the
## rock and slope `sample_cell` reports there.
var _at: Array = []
var _app: Node
var _vh: Control
var _br: Node
var _fail := 0
var _checks := 0
var _report: Dictionary = {}


func _p(s: String) -> void:
	print("GF6LOD  %s" % s)


func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--grid":
				var p := args[i + 1].split("x"); _grid = Vector2i(int(p[0]), int(p[1])); i += 1
			"--vp":
				var v := args[i + 1].split("x"); _vp = Vector2i(int(v[0]), int(v[1])); i += 1
			"--at":
				var a := args[i + 1].split(":")
				var xy := a[1].split(",") if a.size() == 2 else PackedStringArray()
				if xy.size() != 2:
					printerr("--at wants NAME:X,Y, got ", args[i + 1]); get_tree().quit(2); return
				_at.append([a[0], int(xy[0]), int(xy[1])]); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	if _out == "":
		printerr("PROBE-RESULT: ABORT no --out"); get_tree().quit(2); return
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
	var wg: Object = _br.world_gen
	var has_switch: bool = wg.has_method("set_rock_lod_detail")

	# -- A. The app's own world.
	if not await _generate(false):
		return
	var picks_a := _pick_targets()
	_report["A_targets"] = picks_a
	_p("A targets: %s" % str(picks_a))
	if has_switch:
		_check("A: styling inactive on the app's world", not wg.rock_lod_detail_active())
	var a_on := await _capture_all("A", picks_a, "on")
	if not has_switch:
		_finish_cannot_run()
		return
	await _set_switch(false)
	var a_off := await _capture_all("A", picks_a, "off")
	await _set_switch(true)
	for k in a_on.keys():
		var moved := _moved(a_on[k], a_off[k])
		_check("A %s: the switch moves nothing on the app's world" % k, moved == 0, "%d px" % moved)

	# -- B. The same seed, processes reading rock.
	if not await _generate(true):
		return
	_check("B: styling active with the processes on", wg.rock_lod_detail_active())
	# Phase B frames what the styling acts on: the engine's own picks
	# (`rock_lod_detail_stats`, `tests/gf6_rock_ladder.rs`' rule: the steepest
	# inland granite and the steepest bedded cap-edge cell), unless `--at`.
	var stats: Dictionary = wg.rock_lod_detail_stats()
	_report["B_stats"] = stats
	_p("B stats: land %s, cap edge %.2f %%, ridged %.1f %%, terraced %.2f %%" % [str(stats.get("land", 0)),
		100.0 * float(stats.get("cap_edge", 0.0)), 100.0 * float(stats.get("ridged", 0.0)), 100.0 * float(stats.get("terraced", 0.0))])
	var picks_b: Array = []
	for k in ["granite", "cap"]:
		if stats.has(k):
			var c: Vector2i = stats[k]
			picks_b.append([k, c.x, c.y])
	if not _at.is_empty():
		picks_b = _at
	for t in picks_b:
		var d: Dictionary = _br.sample_cell(int(t[1]), int(t[2]))
		_p("B target %s (%d, %d): %s, beneath %s" % [t[0], t[1], t[2], d.get("rock", "?"), d.get("rock_beneath", "none")])
	_report["B_targets"] = picks_b
	_p("B targets: %s" % str(picks_b))
	_check("B: two targets to frame", picks_b.size() == 2, str(picks_b))
	var b_on := await _capture_all("B", picks_b, "on")
	await _set_switch(false)
	var b_off := await _capture_all("B", picks_b, "off")
	await _set_switch(true)
	for k in b_on.keys():
		var moved := _moved(b_on[k], b_off[k])
		var share := float(moved) / float(maxi(1, (b_on[k] as Image).get_width() * (b_on[k] as Image).get_height()))
		_report["B_moved_" + k] = share
		if k.ends_with("_x1"):
			_check("B %s: the base map is not styled" % k, moved == 0, "%d px" % moved)
		else:
			_check("B %s: the switch moves the deep-zoom frame (positive control)" % k, share >= MIN_MOVED_SHARE, "%.1f %% of pixels" % (100.0 * share))
	var f := FileAccess.open(_out.path_join("gf6lod.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	_restore()
	_p("%s  (%d checks, %d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _checks, _fail])
	get_tree().quit(0 if _fail == 0 else 1)


## Puts back what this probe changed on the engine: the app's own
## `geology_processes` (off, `params::defaults()`) and the view switch (on), so
## a settings save after the run cannot carry the probe's world.
func _restore() -> void:
	_br.world_gen.set_params({"geology_processes": false})
	if _br.world_gen.has_method("set_rock_lod_detail"):
		_br.world_gen.set_rock_lod_detail(true)


func _finish_cannot_run() -> void:
	var f := FileAccess.open(_out.path_join("gf6lod.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	_restore()
	printerr("PROBE-CANNOT-RUN: no set_rock_lod_detail (a build before GF-6 4.10); switch-on frames captured for comparison only")
	get_tree().quit(2)


## Generates `_seed` with `geology_processes` set explicitly; aborts (exit 2)
## if the key is rejected, so a frame is never drawn under a world its name
## does not describe.
func _generate(processes: bool) -> bool:
	var rep: Dictionary = _br.world_gen.set_params({"geology_processes": processes})
	if not (rep.get("rejected", PackedStringArray()) as PackedStringArray).is_empty():
		printerr("PROBE-RESULT: ABORT set_params rejected geology_processes: %s" % str(rep))
		get_tree().quit(2)
		return false
	var t0 := Time.get_ticks_msec()
	_br.generate({"seed": _seed, "width_km": 800.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "metropolis": false, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	_p("generated (processes %s) in %.1f s" % ["on" if processes else "off", (Time.get_ticks_msec() - t0) / 1000.0])
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	_vh.overlay.set_labels([])
	return true


## The view switch, then the repaint and tile drop `set_rock_lod_detail`'s doc
## comment asks of the shell.
func _set_switch(on: bool) -> void:
	_br.world_gen.set_rock_lod_detail(on)
	_vh.map_view.texture = _br.color_texture()
	_vh.invalidate_lod_tiles()
	await _settle(8)


func _capture_all(phase: String, picks: Array, arm: String) -> Dictionary:
	var out := {}
	for t in picks:
		var name: String = t[0]
		var p := Vector2(float(t[1]), float(t[2]))
		for z in ZOOMS:
			_vh.overlay.set_labels([])
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(p.x, p.y)
			await _settle_lod()
			var img := await _grab()
			if img == null:
				_check("%s %s x%d %s: capture" % [phase, name, int(z), arm], false, "null image")
				continue
			var key := "%s_x%d" % [name, int(z)]
			img.save_png(_out.path_join("%s_%s_%s.png" % [phase, key, arm]))
			out[key] = img
			_p("%s %s %s: zoom %.2f lod %s" % [phase, key, arm, _vh.zoom(), str(_vh.lod_active())])
	return out


## Pixels whose RGB differs at all between two same-sized captures; -1 when
## the sizes differ (which then fails every check that reads it).
func _moved(a: Image, b: Image) -> int:
	if a.get_width() != b.get_width() or a.get_height() != b.get_height():
		return -1
	var da := a.get_data()
	var db := b.get_data()
	var n := 0
	for k in range(0, da.size(), 4):
		if da[k] != db[k] or da[k + 1] != db[k + 1] or da[k + 2] != db[k + 2]:
			n += 1
	return n


func _pick_targets() -> Array:
	var m := 140
	var st := 8
	var nx := (_grid.x - 2 * m) / st
	var ny := (_grid.y - 2 * m) / st
	var wet := PackedByteArray(); wet.resize(nx * ny)
	var cells: Array = []
	for j in ny:
		for i in nx:
			var x := m + i * st
			var y := m + j * st
			var d: Dictionary = _br.sample_cell(x, y)
			var land: bool = String(d.get("water", "ocean")) == "land"
			wet[j * nx + i] = 0 if land else 1
			if land and d.has("rock") and d.has("slope_deg"):
				cells.append([x, y, String(d["rock"]), float(d["slope_deg"]), d.has("rock_beneath"), i, j])
	var gran := [-1.0, 0, 0, ""]
	var cap := [-1.0, 0, 0, ""]
	for c in cells:
		# Inland for the granite pick (no water within 3 lattice steps, 24
		# cells), as `tests/gf6_rock_ladder.rs` keeps its granite pick 20 cells
		# from the sea: the steepest granite beside the sea is a sea cliff.
		var inland := true
		for dj in range(-3, 4):
			for di in range(-3, 4):
				var ii: int = c[5] + di
				var jj: int = c[6] + dj
				if ii >= 0 and jj >= 0 and ii < nx and jj < ny and wet[jj * nx + ii] == 1:
					inland = false
		if String(c[2]).begins_with("Granite") and inland and c[3] > gran[0]:
			gran = [c[3], c[0], c[1], c[2]]
		elif c[2] in BEDDED and c[4] and c[3] > cap[0]:
			cap = [c[3], c[0], c[1], c[2]]
	var out: Array = []
	if gran[0] >= 0.0:
		out.append(["granite", gran[1], gran[2]])
		_p("granite pick: %s at (%d, %d), slope %.1f deg" % [gran[3], gran[1], gran[2], gran[0]])
	if cap[0] >= 0.0:
		out.append(["cap", cap[1], cap[2]])
		_p("cap pick: %s over a substrate at (%d, %d), slope %.1f deg" % [cap[3], cap[1], cap[2], cap[0]])
	return out


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


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`); 20 s cap.
## `_zoomdetail_probe.gd`'s own method.
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
