extends Node
## Ruling CI item 1 -- the zoom ladder of RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md
## section 1, on the real shell. Measurement only: it changes no setting it does not
## name, and the statistics are computed afterwards from the PNGs it writes (by the
## same script that reads `tests/zoom_detail_ladder.rs`' frames).
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _zoomdetail_probe.tscn -- \
##       --out DIR --targets targets.json [--law 0|1] [--tag NAME] [--grid 2048x1311] [--vp 1600x1000]
##       [--rivers on|off] [--only target,target]
##
## `--targets` is a JSON object `{world_tag: {"seed": s, "archetype": "earth",
## "width_km": w}}`. The cells are picked here, on the shell's own generated world,
## by INPUTS only and never by the value under test (`MISTAKES.md`, "Measure an
## effect inside a subset of pixels"), over an 8-cell stride kept 140 cells from
## every edge so the x4 frame can centre: mountain = highest land cell (the
## assessment's rule); cold = coldest land cell at least 150 cells from it; coast =
## the land cell beside water nearest the map centre; forest = the wettest land
## cell between 5 and 20 C with no water within 24 cells. Generation is
## deterministic, so two DLLs pick the same cells; the picks are written to the
## JSON report so that is checkable. `--law V` sets the appearance key
## `zoom_detail_km` to V after generating (a build without the key answers 0
## recognised and the run says so; that is how a HEAD DLL is recorded). Every
## capture is the map viewport only, at camera zoom 1 / 4 / 16 / 64 (the
## assessment's ladder: `reset_view`, `zoom_step`, `move_view_to`, then until
## `lod_pending() == 0`), with every overlay but rivers hidden (rivers too under
## `--rivers off`) and the labels
## cleared, as the assessment did. Writes `<world>_<target>_x<zoom>_<tag>.png` and
## `zoomdetail_<tag>.json` (px per cell, LOD state, viewport size per capture).
##
## Result: a measurement harness, not a pass/fail probe: per world `  generated in N s`,
##         `  targets: [...]`, one `  <target> x<zoom>: zoom .. ppc .. lod ..` line per capture,
##         then `PROBE-RESULT: DONE` and exit 0. `PROBE-CANNOT-RUN: headless` exits 2;
##         `PROBE-RESULT: ABORT no --targets` exits 2. The statistics are computed offline.

var _out := "user://zoomdetail/"
var _targets: Dictionary = {}
var _law := -1.0
var _tag := "run"
var _grid := Vector2i(2048, 1311)
var _vp := Vector2i(1600, 1000)
var _rivers := true
var _only: PackedStringArray = []
var _app: Node
var _vh: Control
var _br: Node
var _report: Dictionary = {}
const ZOOMS := [1.0, 4.0, 16.0, 64.0]
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--targets": _targets = JSON.parse_string(FileAccess.get_file_as_string(args[i + 1])); i += 1
			"--law": _law = float(args[i + 1]); i += 1
			"--tag": _tag = args[i + 1]; i += 1
			"--grid":
				var p := args[i + 1].split("x"); _grid = Vector2i(int(p[0]), int(p[1])); i += 1
			"--vp":
				var v := args[i + 1].split("x"); _vp = Vector2i(int(v[0]), int(v[1])); i += 1
			## EDIFICE_AND_STREAKS_2026-10-18.md: `--rivers off` hides the river layer
			## (overlay stroke and in-tile river paint, `set_layer_visible("rivers")`)
			## so terrain streaks can be told from river strokes; `--only a,b` keeps
			## those targets. Defaults keep the assessment's own run.
			"--rivers":
				if args[i + 1] != "on" and args[i + 1] != "off":
					printerr("--rivers expects on|off"); get_tree().quit(2); return
				_rivers = args[i + 1] == "on"; i += 1
			"--only":
				_only = args[i + 1].split(","); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	if _targets.is_empty():
		printerr("PROBE-RESULT: ABORT no --targets"); get_tree().quit(2); return
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
	for wtag in _targets.keys():
		await _run_world(wtag, _targets[wtag])
	var f := FileAccess.open(_out.path_join("zoomdetail_%s.json" % _tag), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	print("\nPROBE-RESULT: DONE")
	get_tree().quit(0)


func _run_world(wtag: String, spec: Dictionary) -> void:
	print("\n======== %s ========" % wtag)
	var t0 := Time.get_ticks_msec()
	_br.generate({"seed": int(spec["seed"]), "width_km": float(spec["width_km"]), "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": String(spec["archetype"]), "villages": true, "metropolis": false, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	print("  generated in %.1f s" % ((Time.get_ticks_msec() - t0) / 1000.0))
	await get_tree().create_timer(1.0).timeout
	var rec := {"law_requested": _law, "law_recognised": -1, "captures": []}
	if _law >= 0.0:
		var n: int = _br.set_appearance({"zoom_detail_km": _law})
		rec["law_recognised"] = n
		print("  zoom_detail_km = %s: %d recognised" % [str(_law), n])
		_vh.map_view.texture = _br.color_texture()
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", _rivers)
	rec["rivers"] = _rivers
	_vh.overlay.set_labels([])
	var picks := _pick_targets()
	if not _only.is_empty():
		picks = picks.filter(func(t): return _only.has(String(t[0])))
	rec["targets"] = picks
	print("  targets: ", picks)
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
				printerr("  null capture"); continue
			var fn := "%s_%s_x%d_%s.png" % [wtag, name, int(z), _tag]
			img.save_png(_out.path_join(fn))
			var ppc: float = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
			var c := {"file": fn, "zoom": _vh.zoom(), "ppc": ppc, "lod": _vh.lod_active(), "vp": [_vh.size.x, _vh.size.y]}
			rec["captures"].append(c)
			print("  %s x%d: zoom %.2f ppc %.2f lod %s" % [name, int(z), c["zoom"], ppc, str(c["lod"])])
	_report[wtag] = rec


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
			cells.append([x, y, land, float(d.get("elevation", 0.0)), float(d.get("temperature_c", 0.0)), float(d.get("precipitation", 0.0)), i, j])
	var hi := [-INF, 0, 0]
	for c in cells:
		if c[2] and c[3] > hi[0]:
			hi = [c[3], c[0], c[1]]
	var cold := [INF, 0, 0]
	var coast := [INF, 0, 0]
	var forest := [-INF, 0, 0]
	var ctr := Vector2(_grid.x / 2.0, _grid.y / 2.0)
	for c in cells:
		if not c[2]:
			continue
		var i: int = c[6]
		var j: int = c[7]
		if Vector2(c[0] - hi[1], c[1] - hi[2]).length() > 150.0 and c[4] < cold[0]:
			cold = [c[4], c[0], c[1]]
		var near := false
		var inland := true
		for dj in range(-3, 4):
			for di in range(-3, 4):
				var ii := i + di
				var jj := j + dj
				if ii < 0 or jj < 0 or ii >= nx or jj >= ny:
					continue
				if wet[jj * nx + ii] == 1:
					inland = false
					if absi(di) <= 1 and absi(dj) <= 1:
						near = true
		if near:
			var dc := Vector2(c[0], c[1]).distance_to(ctr)
			if dc < coast[0]:
				coast = [dc, c[0], c[1]]
		if inland and c[4] >= 5.0 and c[4] <= 20.0 and c[5] > forest[0]:
			forest = [c[5], c[0], c[1]]
	var out: Array = [["mountain", hi[1], hi[2]], ["cold", cold[1], cold[2]]]
	if coast[0] < INF:
		out.append(["coast", coast[1], coast[2]])
	if forest[0] > -INF:
		out.append(["forest", forest[1], forest[2]])
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


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`) and its
## per-tile morph has run; 20 s cap. `_riverzoom_probe.gd`'s own method.
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
