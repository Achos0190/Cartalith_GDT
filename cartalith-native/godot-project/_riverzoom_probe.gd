extends Node
## Rivers and lakes across zoom -- measurement only, no pixel change.
## (Since 2026-09-27 the switch rebuilds the deep-zoom tiles, which draw rivers, so
## the ON/OFF pair below is a re-render of both, and each capture waits for the
## pyramid to settle.)
##
## MUST run windowed (frame_post_draw / get_image are dead under headless):
##   Godot_v4.7.1-stable_win64_console.exe --path . _riverzoom_probe.tscn -- --out DIR
##
## Per seed, two targets (a river entering a lake, and the widest trunk), a zoom
## ladder from the opening view to deep LOD. At each zoom: one capture with the
## river overlay ON, one with it OFF (every other overlay hidden). Saved as PNGs.
##
## Measured, palette-agnostic:
##  - stroke fragments: the ON-vs-OFF diff mask (the vector strokes alone) split
##    into 8-connected components; "interior" = a component touching no crop
##    edge, i.e. a line that starts and stops inside the frame.
##  - carve offset (grid data, no pixels): distance from each traced D8 cell
##    centre (where `enforce_channel_descent` carves) to the drawn smoothed
##    render polyline, in cells.
##  - carve cross-section: elevation at the traced cell vs +-1..+-4 cells
##    perpendicular (sample_cell's elevation_m).
##  - RV-2 stroke data (`_stroke_stats`, grid data, no pixels), readable from
##    both the pre-RV-2 `get_rivers` (`lake_mask`, one `width_cells`) and the
##    RV-2 one (`pieces`, per-point `widths`), so one probe measures both:
##    pieces per drawn river, whether each break has water past it, whether the
##    width ever narrows downstream, confluence gap and width ratio, stroke ends
##    beside water that stop short of it, and headwaters under 1 px at x1.
##  - draw cost: median frame interval at the opening view, rivers on vs off.
##  - RIM-1/RIM-3/RIM-6 (`--paint-compare`, rivers painted into the map):
##    `river_paint_stats` (field build ms and bytes); the painted river's
##    coverage against the stroke's on the same ground at every zoom below the
##    deep-zoom switch (pixel counts at two thresholds and their overlap, from
##    the ON-vs-OFF diff of each path -- `WorldGen::debug_set_river_paint(false)`
##    makes the stroke path render the same world); seams (a centreline sample
##    that is LAND to the engine and unchanged by the river layer, counted at
##    river mouths, mid-course lake breaks and tributary joins, painted vs
##    stroke); frame time while the view pans (median with min..max) and the
##    GPU render time when the driver reports it. A path that paints no pixel
##    fails the probe (positive control).
##  - RIM-5 (`--rim5`, headwater streams fade out when zoomed out): over
##    `--rim5-zooms` (camera zoom, default 0.4..2 in fine steps, the view held
##    on the map centre) the river's ink pixels (ON-vs-OFF diff above 8 summed
##    levels, floodplain off) on the painted path and on the stroke path, which
##    keeps the unfaded order-1 look and so is the control. Two measures: `ink`
##    counts the pixels at all touched (a faint stream still counts), `mass`
##    sums the per-pixel colour change (a faded stream counts for less, which
##    is what the eye sees). `kept` / `kept_mass` = painted / stroke: their
##    fall below the fade's full density (`O1_FADE_FULL_PPC`) is the fade,
##    their smoothness (`max_step` between neighbouring zooms) the "no popping"
##    bar, and they must be about one wherever the fade is one. Each zoom's two frames are saved. Positive control: both paths must
##    draw ink at the deepest swept zoom.
##  - RIM-2 (`--rim2`, the bank outline): per preset (`--rim2-presets
##    "Ink,Natural Vibrant"`, default every `STYLE_PRESETS` entry, applied
##    through the render workspace's own `_apply_preset` so the shipped
##    strengths are what runs) at the fit view and at `--rim2-zoom` (default
##    2.1, just under the deep-zoom switch at 2.2 -- the tiles carry no outline,
##    RIM-7): the style as shipped, the same frame with the bank forced to 0
##    (`*_shipped.png` / `*_off.png`; ON-vs-OFF mean abs diff and the pixels
##    above 8 summed levels), and for a preset that ships none the frame with
##    the strength forced to `--rim2-force` (default 0.4, `*_forced.png`).
##    `--rim2-cost NAME` adds the pan frame time and GPU time for that preset
##    with the bank on and off. `--rim2-head` records shipped frames only (a
##    HEAD build has no `river_bank` uniform) so a HEAD run and this tree's
##    can be compared pixel for pixel. Controls: a preset with a bank must
##    move pixels at the zoomed view, and one frame grabbed twice must be
##    identical (MAD 0), or the leg fails. `--river-density X`,
##    `--width-km K`, `--geology-model` and `--metropolis` reproduce the
##    owner's world (seed 246371, 2048x1311, 1.55, 40075, both switches on).
##  - RIM-4 (river deltas, Ruling BU). `--rim4-mouths` (grid data, no pixels):
##    every drawn river whose last point stands on water is a MOUTH; prints
##    min / median / p90 / max of its order, discharge and discharge-over-grid
##    per water kind (the Part A measurement). `--rim4` (implies it) then
##    frames the `--rim4-n` (default 4) mouths the engine gives a fan at
##    `--rim4-zooms` (default 1.5,2.1,3.0), the fans ON and OFF
##    (`rim4_*_on.png` / `_off.png`) and prints `river_delta_stats`;
##    `--rim4-cost` adds the pan frame time and GPU time at the fit view and
##    z 2.1, ON/OFF alternated three times. `--rim4-head` records shipped
##    frames only and calls nothing a HEAD build lacks, so a HEAD DLL's frames
##    equal this tree's OFF frames pixel for pixel. `--rim4-lakes` (Part C)
##    frames the widest rivers that pass THROUGH a lake at z 2/3/4 and, at
##    z 2, the stroke path for contrast. Controls: the fans must move a pixel
##    in some ON/OFF pair, and an empty pick or no lake crossing fails.
##  - RIM-7, deltas (`--rim7`, implies `--rim4-mouths`): the same fans on the
##    deep-zoom tiles and in the export. Per picked mouth (`_rim4_pick`, up to
##    `--rim4-n`) and `--rim7-zooms` (default 1.5,2.1,2.3,3,4,8 -- the switch
##    is `viewport_host.gd`'s `LOD_AUTO_ZOOM` 2.2), the frame with the fans ON
##    and OFF, floodplain tint off in both (`rim7_*_on.png` is the frame as
##    the user sees it, tint on; `_off.png` its OFF twin). Measured inside a
##    disk of `--rim7-radius` cells (default 64) round the mouth, so frames at
##    different zooms compare the same ground: the fan's footprint in CELLS
##    (ON-vs-OFF pixels over 8 and over 24 summed levels, divided by pixels
##    per cell squared), the mean ON colour of the over-24 pixels, and the
##    colour change per cell. `continuity` pairs the last zoom below the
##    switch with the first above it, beside the same ratios for the network
##    round the mouth (`net_*`: fans off, Rivers layer on vs off -- the
##    yardstick, since the network changes path there too). `--rim7-export` also writes the real
##    region export (`WorldGen::export_snapshot_png`, the `export_raster_png`
##    assembly) round each mouth at 8 and 2 px a cell, ON and OFF
##    (`rim7x_*.png`), and diffs them. `--rim7-cost` adds the pan frame time
##    and GPU time at z 3 and 4 after the pyramid settles, and the synchronous
##    tile build (`lod_synthesize_tile`) of the mouth's tile at those zooms'
##    levels, ON/OFF alternated three rounds. Controls: some ON/OFF pair past
##    the switch must move a pixel (a build whose tiles draw no fan -- HEAD
##    before RIM-7 -- reports `PROBE-FAIL`), and a frame grabbed twice must be
##    identical. Run it on a HEAD DLL too: its OFF frames and exports must
##    equal this tree's OFF ones pixel for pixel.
##  - RIM-7, the painted river itself (`--rim7rest`): the deep-zoom tiles and the
##    export paint the river by the screen's law (`river_field::PaintSource`).
##    Three targets on land (`_rim7r_targets`: the widest trunk, the longest
##    order-2 river and order-1 stream), each framed at `--rim7r-zooms` (default
##    1,2.1,2.3,3,4,8; the switch is 2.2) with the river opacity at 1 and at 0
##    and with the Rivers layer off (`rim7r_<target>_z<zoom>_on/off/bare.png`:
##    opacity 0 removes the water on every path and keeps the floodplain tint and
##    bank line, so on-vs-off is the water, and off-vs-bare over 24 levels is the
##    bank line, 4..24 levels the floodplain tint).
##    Measured in CELL space (`_rim7r_mask`: a 0.25-cell grid in a disk of
##    `--rim7r-radius` cells, default 12): the river's area in cells and mean
##    colour per view, and per pair -- the switch pair 2.1->2.3 and the first tile
##    zoom against each deeper one -- the area ratio (a width ratio), IoU at zero
##    shift and the best shift within one cell (a river drawn in the same place
##    reads 0). `--rim7r-export` adds the real region export
##    (`export_snapshot_png`) at 4 and 8 px a cell against the tile view of the
##    nearest density, the same way. Identity frames for a HEAD comparison by
##    script (`rim7r_id_*`): fit and z2.1 (the screen path, which must not
##    change), z3 with the Rivers layer off and with the stroke look
##    (`debug_set_river_paint(false)`), and the stroke look's export at the trunk
##    and the largest delta mouth (`rim7r_id_*_stroke_export.png`, which also
##    shows whether an export's fans follow the export's own look).
##    `--rim7r-cost`: pan frame time at fit, z2.1 and z3 and the synchronous
##    tile build at the z3/z4/z8 levels, painted against stroked, three rounds.
##    `--rim7r-ident-only` records only the identity frames. Controls: a frame
##    grabbed twice must be identical; no target is a failure.
##  - RV-1 (`_lake_totals`, grid data): every lake on the map by size, split
##    into channel-shaped trenches and basins, and ocean cells on river paths.
##
## `--targets FILE` reuses the target cells of an earlier run's riverzoom.json,
## so a before/after pair frames the same ground. `--stats-only` skips the zoom
## sweep and its PNGs (grid statistics and draw cost only); `--zooms 1,32`
## narrows the sweep. `--vp WxH` sets the window (default 1600x1000; the map
## area is only the part of it the docks leave, so the fit density is lower
## than the window suggests). `--appearance key=v,key=v` sets appearance tunables after
## generation (e.g. `river_ink=1,river_ink_r=0,river_ink_g=0,river_ink_b=0` to
## measure the stroke's continuity in a full-contrast ink, independent of how
## far a style's colour sits from the ground).

var _out := "user://riverzoom/"
var _seeds: Array[int] = [483920, 24601, 71077345]
var _grid := Vector2i(1024, 656)
var _vp := Vector2i(1600, 1000)
var _width_km := 1200.0
var _app: Node
var _vh: Control
var _br: Node
var _report: Dictionary = {}
var _fixed_targets: Dictionary = {}
var _stats_only := false
var _paint_compare := false
var _rim5 := false
var _rim2 := false
var _rim2_head := false
var _rim2_presets: Array = []
var _rim2_zoom := 2.1
var _rim2_force := 0.4
var _rim2_cost_preset := ""
var _rim4_mouths := false
var _rim4 := false
var _rim4_head := false
var _rim4_n := 4
var _rim4_zooms: Array = [1.5, 2.1, 3.0]
var _rim4_lakes := false
var _rim4_cost := false
var _rim7 := false
var _rim7_zooms: Array = [1.5, 2.1, 2.3, 3.0, 4.0, 8.0]
var _rim7_radius := 64.0
var _rim7_export := false
var _rim7_cost := false
var _rim7r := false
var _rim7r_zooms: Array = [1.0, 2.1, 2.3, 3.0, 4.0, 8.0]
var _rim7r_radius := 12.0
var _rim7r_export := false
var _rim7r_cost := false
var _rim7r_ident_only := false
var _river_density := 1.0
var _geology_model := false
var _metropolis := false
var _rim5_zooms: Array = [0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.9, 1.0, 1.1, 1.2, 1.35, 1.5, 1.75, 2.0]
var _appearance := {}
var _zooms: Array = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 240.0]

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
			"--seeds":
				_seeds.clear()
				for s in args[i + 1].split(","): _seeds.append(int(s))
				i += 1
			"--grid":
				var p := args[i + 1].split("x"); _grid = Vector2i(int(p[0]), int(p[1])); i += 1
			"--stats-only": _stats_only = true
			"--paint-compare": _paint_compare = true
			"--rim5": _rim5 = true
			"--rim2": _rim2 = true
			"--rim2-head": _rim2_head = true
			"--rim2-presets":
				_rim2_presets.clear()
				if args[i + 1] != "all":
					for ps in args[i + 1].split(","): _rim2_presets.append(ps)
				i += 1
			"--rim2-zoom": _rim2_zoom = float(args[i + 1]); i += 1
			"--rim2-force": _rim2_force = float(args[i + 1]); i += 1
			"--rim2-cost": _rim2_cost_preset = args[i + 1]; i += 1
			"--rim4-mouths": _rim4_mouths = true
			"--rim4": _rim4 = true; _rim4_mouths = true
			"--rim4-head": _rim4_head = true
			"--rim4-n": _rim4_n = int(args[i + 1]); i += 1
			"--rim4-lakes": _rim4_lakes = true
			"--rim4-cost": _rim4_cost = true
			"--rim7": _rim7 = true; _rim4_mouths = true
			"--rim7-export": _rim7_export = true
			"--rim7-cost": _rim7_cost = true
			"--rim7-radius": _rim7_radius = float(args[i + 1]); i += 1
			"--rim7-zooms":
				_rim7_zooms.clear()
				for zs in args[i + 1].split(","): _rim7_zooms.append(float(zs))
				i += 1
			"--rim7rest": _rim7r = true
			"--rim7r-export": _rim7r_export = true
			"--rim7r-cost": _rim7r_cost = true
			"--rim7r-ident-only": _rim7r_ident_only = true
			"--rim7r-radius": _rim7r_radius = float(args[i + 1]); i += 1
			"--rim7r-zooms":
				_rim7r_zooms.clear()
				for zs in args[i + 1].split(","): _rim7r_zooms.append(float(zs))
				i += 1
			"--rim4-zooms":
				_rim4_zooms.clear()
				for zs in args[i + 1].split(","): _rim4_zooms.append(float(zs))
				i += 1
			"--river-density": _river_density = float(args[i + 1]); i += 1
			"--geology-model": _geology_model = true
			"--metropolis": _metropolis = true
			"--width-km": _width_km = float(args[i + 1]); i += 1
			"--rim5-zooms":
				_rim5_zooms.clear()
				for zs in args[i + 1].split(","): _rim5_zooms.append(float(zs))
				i += 1
			"--vp":
				var vp := args[i + 1].split("x"); _vp = Vector2i(int(vp[0]), int(vp[1])); i += 1
			"--zooms":
				_zooms.clear()
				for zs in args[i + 1].split(","): _zooms.append(float(zs))
				i += 1
			"--appearance":
				for kv in args[i + 1].split(","):
					var pr := kv.split("=")
					_appearance[pr[0]] = float(pr[1])
				i += 1
			"--targets":
				_fixed_targets = JSON.parse_string(FileAccess.get_file_as_string(args[i + 1])); i += 1
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
	for s in _seeds:
		await _run_seed(s)
	var f := FileAccess.open(_out.path_join("riverzoom.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(_report, "  "))
	print("\nPROBE-RESULT: DONE")
	get_tree().quit(0)


func _run_seed(seed_v: int) -> void:
	print("\n======== seed %d  grid %dx%d ========" % [seed_v, _grid.x, _grid.y])
	## The owner-world switches (`--river-density`, `--geology-model`,
	## `--metropolis`), set before generating exactly as `_roadsettle_probe`
	## does; each stays at the engine default unless asked for.
	if _river_density != 1.0:
		_br.param_set("river_density", _river_density)
	if _geology_model:
		_br.param_set("geology_model", true)
	_br.generate({"seed": seed_v, "width_km": _width_km, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "metropolis": _metropolis, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	if not _appearance.is_empty():
		var n: int = _br.set_appearance(_appearance)
		print("  appearance overrides applied: %d of %d %s" % [n, _appearance.size(), str(_appearance)])
		_vh.map_view.texture = _br.color_texture()
	var rivers: Array = _br.rivers(1)
	var drawn := 0
	for r: Dictionary in rivers:
		if r.has("width_cells") and not r.has("parallel_of"):
			drawn += 1
	print("  rivers traced %d, drawn %d" % [rivers.size(), drawn])
	var sr := {"traced": rivers.size(), "drawn": drawn, "targets": {}}
	sr["carve"] = _carve_stats(rivers)
	print("  carve: ", sr["carve"])

	sr["channel_lakes"] = _channel_lakes(rivers)
	print("  channel lakes: ", sr["channel_lakes"])
	sr["lakes"] = _lake_totals(rivers)
	print("  lakes: ", sr["lakes"])
	if _rim4_mouths:
		sr["rim4_mouths"] = _rim4_mouth_stats(rivers)
	_vh.reset_view()
	await _settle(3)
	var ppc_fit: float = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
	sr["stroke"] = _stroke_stats(rivers, ppc_fit)
	print("  stroke: ", sr["stroke"])
	## River mouths (owner, 2026-09-27): every free end beside water is carried
	## into it (`river_stroke::extend_shore_ends`). Protects: no drawn end stops
	## short of the sea or lake it meets -- 76-100 per world did before.
	var short_n := int(sr["stroke"]["free_ends_short_of_adjacent_water"])
	if short_n > 0:
		printerr("PROBE-FAIL: %d free ends stop short of adjacent water (seed %d)" % [short_n, seed_v])
		_report["fail"] = true
	else:
		print("  PROBE-OK: no free end stops short of adjacent water")
	sr["draw_cost_ms"] = await _draw_cost()
	print("  draw cost (median frame ms, rivers on/off): ", sr["draw_cost_ms"])
	var targets := {} if _stats_only else _pick_targets(rivers)
	if not _stats_only and _fixed_targets.has(str(seed_v)):
		targets = {}
		var ft: Dictionary = _fixed_targets[str(seed_v)]["targets"]
		for name: String in ft.keys():
			var c: Array = ft[name]["cell"]
			targets[name] = {"p": Vector2(c[0], c[1]), "idx": -1, "w": -1.0}
	for name: String in targets.keys():
		var t: Dictionary = targets[name]
		print("  target %s at cell %.1f,%.1f (river %d, width_cells %.2f)" % [name, t["p"].x, t["p"].y, t["idx"], t["w"]])
		var rows: Array = []
		for z: float in _zooms:
			if z > _vh._zoom_max + 1e-6:
				continue
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(t["p"].x, t["p"].y)
			await _settle(60)
			await _settle_lod()
			var on := await _grab()
			## Since 2026-09-27 the Rivers switch re-renders the base map and
			## rebuilds every deep-zoom tile (the rivers are IN them), so the
			## OFF frame must wait for the pyramid, not four frames: a tile
			## still rebuilding shows its parent, and the diff then reads the
			## whole terrain as "stroke".
			_vh.set_layer_visible("rivers", false)
			await _settle_lod()
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			await _settle_lod()
			if on == null or off == null:
				continue
			var tag := "s%d_%s_z%06.2f" % [seed_v, name, z]
			on.save_png(_out.path_join(tag + "_on.png"))
			off.save_png(_out.path_join(tag + "_off.png"))
			var frag := _fragments(on, off)
			frag["zoom"] = _vh.zoom()
			frag["lod"] = _vh.lod_active()
			frag["px_per_cell"] = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
			print("    z %6.1f  lod=%s  ppc %.1f  stroke_px %d  comps %d  interior %d  small(<40px) %d"
				% [frag["zoom"], str(frag["lod"]), frag["px_per_cell"], frag["px"], frag["comps"], frag["interior"], frag["small"]])
			rows.append(frag)
			## Positive control: the river layer must move pixels. A shell that
			## fails to draw them (a bad bridge call) reads 0 here, and every
			## "fewer fragments" figure would then be vacuous.
			if name == "trunk" and int(frag["px"]) == 0:
				printerr("PROBE-FAIL: rivers drew no pixels at zoom %.1f" % z)
				_report["fail"] = true
		sr["targets"][name] = {"cell": [t["p"].x, t["p"].y], "rows": rows}
	if _paint_compare and not _stats_only:
		sr["paint"] = await _paint_compare_run(rivers, targets)
	if _rim5 and not _stats_only:
		sr["rim5"] = await _rim5_run()
	if _rim2:
		sr["rim2"] = await _rim2_run(targets)
	if _rim4:
		sr["rim4"] = await _rim4_run(sr["rim4_mouths"])
	if _rim4_lakes:
		sr["rim4_lakes"] = await _rim4_lakes_run(rivers)
	if _rim7:
		sr["rim7"] = await _rim7_run(sr["rim4_mouths"])
	if _rim7r:
		sr["rim7rest"] = await _rim7r_run(rivers)
	_report[str(seed_v)] = sr


## A river's drawn pieces as `[start, end)` into `render_points`: RV-2's
## `pieces`, or -- for the pre-RV-2 data -- the dry runs of >= 2 points the old
## `_draw_rivers` stroked between `lake_mask` points.
func _pieces_of(r: Dictionary) -> Array[Vector2i]:
	var rp: PackedVector2Array = r["render_points"]
	var out: Array[Vector2i] = []
	if r.has("pieces"):
		var pc: PackedInt32Array = r["pieces"]
		for i in range(0, pc.size() - 1, 2):
			out.append(Vector2i(pc[i], pc[i + 1]))
		return out
	var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
	if m.size() != rp.size():
		out.append(Vector2i(0, rp.size()))
		return out
	var a := -1
	for idx in range(0, rp.size() + 1):
		var dry := idx < rp.size() and m[idx] == 0
		if dry:
			if a < 0:
				a = idx
		elif a >= 0:
			if idx - a >= 2:
				out.append(Vector2i(a, idx))
			a = -1
	return out


func _width_at(r: Dictionary, i: int) -> float:
	if r.has("widths"):
		return float((r["widths"] as PackedFloat32Array)[i])
	return float(r["width_cells"])


func _drawable(r: Dictionary) -> bool:
	return (r.has("widths") or (r.has("width_cells") and not r.has("pieces"))) and not r.has("parallel_of")


func _is_wet(p: Vector2) -> bool:
	var x := int(floor(p.x)); var y := int(floor(p.y))
	if x < 0 or y < 0 or x >= _grid.x or y >= _grid.y:
		return false
	var w := String(_br.sample_cell(x, y).get("water", "land"))
	return w == "lake" or w == "ocean"


func _stroke_stats(rivers: Array, ppc_fit: float) -> Dictionary:
	var drawn := 0; var hist := {}; var r3 := 0; var r2 := 0
	var breaks := 0; var breaks_wet := 0
	var narrows := 0; var narrow_rivers := 0; var const_rivers := 0
	var sub_px := 0
	var ends_on_shore := 0; var ends_short := 0
	## Spatial index of every drawn point: cell -> [river, point index].
	## `ends` marks each river's own first and last drawn point, so a join is
	## matched to the river whose stroke passes THROUGH the end, not to a
	## sibling tributary ending on the same confluence point.
	var grid := {}
	var ends := {}
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var all_pcs := _pieces_of(r)
		if not all_pcs.is_empty():
			ends[Vector2i(ri, all_pcs[0].x)] = true
			ends[Vector2i(ri, all_pcs[all_pcs.size() - 1].y - 1)] = true
		for pc in all_pcs:
			for i in range(pc.x, pc.y):
				var key := Vector2i(int(floor(rp[i].x)), int(floor(rp[i].y)))
				if not grid.has(key): grid[key] = []
				grid[key].append(Vector2i(ri, i))
	var short_gap := PackedFloat32Array()
	var short_where: Array = []
	## Draw rank: RV-2 draws by `get_rivers()`' `draw_rank`; the build before
	## it drew in list order.
	var rank := {}
	for k in rivers.size():
		rank[k] = int((rivers[k] as Dictionary).get("draw_rank", k))
	var continuations := 0; var continuation_narrows := 0; var shared_ends := 0
	var joins := 0; var join_gap_max := 0.0; var join_wider := 0; var join_trunk_drawn_over := 0
	var ratio := PackedFloat32Array()
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		drawn += 1
		var rp: PackedVector2Array = r["render_points"]
		var pcs := _pieces_of(r)
		hist[pcs.size()] = int(hist.get(pcs.size(), 0)) + 1
		if pcs.size() >= 3: r3 += 1
		if pcs.size() >= 2: r2 += 1
		for k in range(pcs.size() - 1):
			breaks += 1
			var e := rp[pcs[k].y - 1]
			var s := rp[pcs[k + 1].x]
			if _is_wet(e + (s - e).normalized() * 0.05):
				breaks_wet += 1
		## Width along the whole drawn stroke, head to mouth.
		var prev := -1.0; var nr := 0; var wmin := INF; var wmax := 0.0
		for pc in pcs:
			for i in range(pc.x, pc.y):
				var wv := _width_at(r, i)
				if prev >= 0.0 and wv < prev - 1e-5: nr += 1
				prev = wv
				wmin = minf(wmin, wv); wmax = maxf(wmax, wv)
		narrows += nr
		if nr > 0: narrow_rivers += 1
		if wmax - wmin < 1e-6: const_rivers += 1
		var mul := 0.55 if int(r.get("own_order", r.get("order", 2))) <= 1 else 1.0
		if wmin * ppc_fit * mul < 1.0: sub_px += 1
		if pcs.is_empty():
			continue
		var last: Vector2i = pcs[pcs.size() - 1]
		if last.y - last.x < 2:
			continue
		var E := rp[last.y - 1]
		var D := (E - rp[last.y - 2]).normalized()
		## Join: another drawn river's stroke passes through E.
		var best := INF; var best_w := 0.0; var best_ri := -1
		var ce := Vector2i(int(floor(E.x)), int(floor(E.y)))
		for dy in [-1, 0, 1]:
			for dx in [-1, 0, 1]:
				for h: Vector2i in grid.get(ce + Vector2i(dx, dy), []):
					if h.x == ri or ends.has(h): continue
					var o: Dictionary = rivers[h.x]
					var orp: PackedVector2Array = o["render_points"]
					for j in [h.y - 1, h.y]:
						if j < 0 or j + 1 >= orp.size(): continue
						var d := _seg_dist(E, orp[j], orp[j + 1])
						if d < best:
							## The joined river's width AT the join: interpolated
							## along the segment, as the stroke is drawn.
							var ab := orp[j + 1] - orp[j]
							var tt := clampf((E - orp[j]).dot(ab) / maxf(ab.length_squared(), 1e-12), 0.0, 1.0)
							best = d; best_w = lerpf(_width_at(o, j), _width_at(o, j + 1), tt); best_ri = h.x
		var o_first: Vector2 = (rivers[best_ri]["render_points"] as PackedVector2Array)[0] if best_ri >= 0 else Vector2.INF
		var o_last: Vector2 = (rivers[best_ri]["render_points"] as PackedVector2Array)[-1] if best_ri >= 0 else Vector2.INF
		if best < 0.05 and E.distance_to(o_last) < 0.05:
			## Two runs ending on one point (both run off the map edge there, or
			## meet end to end): neither is the other's trunk.
			shared_ends += 1
			continue
		if best < 0.05 and E.distance_to(o_first) < 0.05:
			continuations += 1
			if _width_at(r, last.y - 1) > _width_at(rivers[best_ri], 0) + 1e-5: continuation_narrows += 1
			continue
		if best < 0.05:
			joins += 1
			join_gap_max = maxf(join_gap_max, best)
			var tw := _width_at(r, last.y - 1)
			ratio.append(tw / maxf(best_w, 1e-6))
			if tw > best_w + 1e-5: join_wider += 1
			if int(rank[best_ri]) > int(rank[ri]):
				join_trunk_drawn_over += 1
			continue
		## Not a join: does it end on water, or stop short of water beside it?
		if _is_wet(E + D * 0.05):
			ends_on_shore += 1
		else:
			var near_wet := false
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					if _is_wet(Vector2(ce) + Vector2(dx + 0.5, dy + 0.5)): near_wet = true
			if near_wet:
				ends_short += 1
				## How far short: distance from E to the nearest wet cell's square.
				var g := INF
				for dy in [-1, 0, 1]:
					for dx in [-1, 0, 1]:
						var c := Vector2(ce) + Vector2(dx, dy)
						if _is_wet(c + Vector2(0.5, 0.5)):
							var q := Vector2(clampf(E.x, c.x, c.x + 1.0), clampf(E.y, c.y, c.y + 1.0))
							g = minf(g, E.distance_to(q))
				short_gap.append(g)
				## Where, for reading: the end, its direction and piece count.
				if short_where.size() < 8:
					short_where.append([E, D, pcs.size()])
	return {"drawn": drawn, "pieces_hist": hist, "rivers_2plus_pieces": r2, "rivers_3plus_pieces": r3,
		"breaks": breaks, "breaks_with_water_past_them": breaks_wet,
		"width_narrowing_steps": narrows, "rivers_that_narrow": narrow_rivers, "rivers_one_width": const_rivers,
		"rivers_under_1px_at_x1": sub_px,
		"continuations": continuations, "continuations_that_narrow": continuation_narrows, "shared_ends": shared_ends,
		"joins": joins, "join_gap_max_cells": join_gap_max, "joins_trib_wider_than_trunk": join_wider,
		"joins_joined_river_drawn_last": join_trunk_drawn_over, "join_width_ratio": _stats(ratio),
		"free_ends_on_shore": ends_on_shore, "free_ends_short_of_adjacent_water": ends_short,
		"short_end_gap_cells": _stats(short_gap), "short_ends_where": str(short_where)}


## Median frame interval over 30 frames at the opening view, rivers on vs off.
func _draw_cost() -> Dictionary:
	var out := {}
	for on in [true, false]:
		_vh.set_layer_visible("rivers", on)
		await _settle(5)
		var dt := PackedFloat32Array()
		var t0 := Time.get_ticks_usec()
		for f in 30:
			_vh.overlay.queue_redraw()
			await RenderingServer.frame_post_draw
			var t1 := Time.get_ticks_usec()
			dt.append((t1 - t0) / 1000.0)
			t0 = t1
		out["on" if on else "off"] = _stats(dt)["median"]
	_vh.set_layer_visible("rivers", true)
	return out


func _pick_targets(rivers: Array) -> Dictionary:
	var out := {}
	var best_w := -1.0
	var best_lake_w := -1.0
	for idx in rivers.size():
		var r: Dictionary = rivers[idx]
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var w: float = r["width_cells"]
		var mid := rp[int(rp.size() * 0.7)] if rp.size() > 0 else Vector2(-1, -1)
		if w > best_w and rp.size() > 8 and _inner(mid):
			best_w = w
			out["trunk"] = {"p": mid, "idx": idx, "w": w}
		var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
		if m.size() == rp.size():
			for k in range(1, m.size() - 20):
				if m[k - 1] == 0 and m[k] == 1 and w > best_lake_w and _inner(rp[k]):
					var run := 0
					while k + run < m.size() and m[k + run] == 1:
						run += 1
					if run >= 20:
						best_lake_w = w
						out["lake"] = {"p": rp[k], "idx": idx, "w": w}
						break
	return out


## Grid-data carve measurement over every drawn river.
func _carve_stats(rivers: Array) -> Dictionary:
	var dists := PackedFloat32Array()
	var depth1 := PackedFloat32Array()   ## centre vs +-1 cell
	var depth3 := PackedFloat32Array()   ## centre vs +-3..4 cells
	var sampled := 0
	for r: Dictionary in rivers:
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		var pts: PackedVector2Array = r["points"]
		var rp: PackedVector2Array = r["render_points"]
		for k in range(1, pts.size() - 1):
			var p := pts[k]
			var best := INF
			for j in range(rp.size() - 1):
				var d := _seg_dist(p, rp[j], rp[j + 1])
				if d < best: best = d
			dists.append(best)
			if sampled < 1500 and k % 3 == 0:
				var dir := (pts[k + 1] - pts[k - 1]).normalized()
				var n := Vector2(-dir.y, dir.x)
				var c := _elev(p)
				var a1 := (_elev(p + n) + _elev(p - n)) * 0.5
				var a3 := (_elev(p + n * 3.0) + _elev(p - n * 3.0) + _elev(p + n * 4.0) + _elev(p - n * 4.0)) * 0.25
				if not is_nan(c) and not is_nan(a1) and not is_nan(a3):
					depth1.append(a1 - c)
					depth3.append(a3 - c)
					sampled += 1
	return {
		"trace_to_line_cells": _stats(dists),
		"depth_vs_1cell_m": _stats(depth1),
		"depth_vs_3to4cell_m": _stats(depth3),
	}


func _inner(p: Vector2) -> bool:
	return p.x > 40 and p.y > 40 and p.x < _grid.x - 40 and p.y < _grid.y - 40


## Does the lake classification bead along carved channels? Walks every drawn
## river's traced D8 cells; counts cells sample_cell calls "lake" and the runs
## they form. Also counts how many pieces the drawn stroke's lake_mask cuts
## each river into (map_overlay.gd::_draw_rivers breaks the stroke there).
func _channel_lakes(rivers: Array) -> Dictionary:
	var cells := 0; var lake := 0; var ocean := 0; var runs := 0; var short_runs := 0
	var toggles := 0; var split_rivers := 0; var drawn := 0; var pieces_hist := {}
	for r: Dictionary in rivers:
		if not r.has("width_cells") or r.has("parallel_of"):
			continue
		drawn += 1
		var pts: PackedVector2Array = r["points"]
		var cur := 0
		for k in pts.size() - 1:   ## last point is the trunk's / sea cell
			var d: Dictionary = _br.sample_cell(int(pts[k].x), int(pts[k].y))
			cells += 1
			if String(d.get("water", "")) == "ocean":
				ocean += 1
			if String(d.get("water", "")) == "lake":
				lake += 1; cur += 1
			else:
				if cur > 0:
					runs += 1
					if cur <= 3: short_runs += 1
				cur = 0
		if cur > 0:
			runs += 1
			if cur <= 3: short_runs += 1
		var m: PackedByteArray = r.get("lake_mask", PackedByteArray())
		var t := 0
		for k in range(1, m.size()):
			if m[k] != m[k - 1]: t += 1
		toggles += t
		var pieces := 0
		var inrun := false
		for k in m.size():
			if m[k] == 0 and not inrun: pieces += 1
			inrun = m[k] == 0
		if pieces >= 3: split_rivers += 1
		pieces_hist[pieces] = int(pieces_hist.get(pieces, 0)) + 1
	return {"drawn": drawn, "traced_cells": cells, "lake_cells_on_path": lake, "ocean_cells_on_path": ocean, "lake_runs": runs,
		"lake_runs_len_le3": short_runs, "mask_toggles": toggles, "rivers_cut_into_3plus_pieces": split_rivers,
		"pieces_hist": pieces_hist}


## RV-1 (Ruling BD): every lake on the map, not just the ones a river crosses,
## so a carve fix can be seen not to drain the real ones. Lake-class cells from
## `sample_cell` (the drawn classification), split into 4-connected bodies --
## the connectivity `build_water_bodies` itself uses. A body of 10+ cells with
## at least 60% of them within one cell of a traced river cell is counted as
## channel-shaped: a trench the carve sealed, not a basin.
func _lake_totals(rivers: Array) -> Dictionary:
	var n := _grid.x * _grid.y
	var near := PackedByteArray(); near.resize(n)
	for r: Dictionary in rivers:
		for p: Vector2 in (r["points"] as PackedVector2Array):
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					var nx: int = int(p.x) + dx; var ny: int = int(p.y) + dy
					if nx >= 0 and ny >= 0 and nx < _grid.x and ny < _grid.y:
						near[ny * _grid.x + nx] = 1
	var chan := 0; var chan_area := 0; var basin_ge100 := PackedInt32Array()
	var lake := PackedByteArray(); lake.resize(n)
	for y in _grid.y:
		for x in _grid.x:
			if String(_br.sample_cell(x, y).get("water", "land")) == "lake":
				lake[y * _grid.x + x] = 1
	var sizes := PackedInt32Array()
	var stack := PackedInt32Array()
	for s in n:
		if lake[s] != 1:
			continue
		lake[s] = 2
		stack.append(s)
		var k := 0; var kn := 0
		while not stack.is_empty():
			var i: int = stack[stack.size() - 1]; stack.remove_at(stack.size() - 1)
			k += 1
			kn += near[i]
			var x := i % _grid.x; var y := i / _grid.x
			for d: Vector2i in [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]:
				var nx := x + d.x; var ny := y + d.y
				if nx < 0 or ny < 0 or nx >= _grid.x or ny >= _grid.y: continue
				var j := ny * _grid.x + nx
				if lake[j] == 1:
					lake[j] = 2; stack.append(j)
		sizes.append(k)
		if k >= 10 and kn * 10 >= k * 6:
			chan += 1; chan_area += k
		elif k >= 100:
			basin_ge100.append(k)
	var area := 0; var le3 := 0; var ge10 := 0; var ge10_area := 0; var ge100 := 0
	for k in sizes:
		area += k
		if k <= 3: le3 += 1
		if k >= 10: ge10 += 1; ge10_area += k
		if k >= 100: ge100 += 1
	basin_ge100.sort()
	return {"count": sizes.size(), "area_cells": area, "count_le3": le3, "count_ge10": ge10,
		"area_ge10": ge10_area, "count_ge100": ge100, "channel_shaped_ge10": chan,
		"channel_shaped_area": chan_area, "basins_ge100": Array(basin_ge100)}


func _elev(p: Vector2) -> float:
	var x := int(floor(p.x)); var y := int(floor(p.y))
	if x < 0 or y < 0 or x >= _grid.x or y >= _grid.y:
		return NAN
	var d: Dictionary = _br.sample_cell(x, y)
	return float(d.get("elevation_m", NAN))


func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var ab := b - a
	var l2 := ab.length_squared()
	if l2 <= 0.0:
		return p.distance_to(a)
	var t := clampf((p - a).dot(ab) / l2, 0.0, 1.0)
	return p.distance_to(a + ab * t)


func _stats(v: PackedFloat32Array) -> Dictionary:
	if v.is_empty():
		return {"n": 0}
	var a := Array(v); a.sort()
	return {"n": a.size(), "median": a[a.size() / 2], "p90": a[int(a.size() * 0.9)], "max": a[a.size() - 1]}


func _fragments(on: Image, off: Image) -> Dictionary:
	var w := on.get_width(); var h := on.get_height()
	var da := on.get_data(); var db := off.get_data()
	var mask := PackedByteArray(); mask.resize(w * h)
	var px := 0
	for i in w * h:
		var o := i * 4
		var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
		if dd > 24:
			mask[i] = 1; px += 1
	var comps := 0; var interior := 0; var small := 0
	var stack := PackedInt32Array()
	for s in w * h:
		if mask[s] != 1:
			continue
		comps += 1
		mask[s] = 2
		stack.append(s)
		var n := 0; var edge := false
		while not stack.is_empty():
			var i: int = stack[stack.size() - 1]; stack.remove_at(stack.size() - 1)
			n += 1
			var x := i % w; var y := i / w
			if x == 0 or y == 0 or x == w - 1 or y == h - 1: edge = true
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					var nx: int = x + dx; var ny: int = y + dy
					if nx < 0 or ny < 0 or nx >= w or ny >= h: continue
					var j := ny * w + nx
					if mask[j] == 1:
						mask[j] = 2; stack.append(j)
		if not edge: interior += 1
		if n < 40: small += 1
	return {"px": px, "comps": comps, "interior": interior, "small": small}


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


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0` -- a
## condition, not a frame count) and its per-tile morph has run; 20 s cap.
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


## ---- RIM-1 / RIM-3 / RIM-6: the rivers painted into the map ----------------

## Set the paint path (`true`) or the stroke path (`false`) and re-render the
## base map so the next capture shows it. `debug_set_river_paint` only flips
## the appearance flag, the colour texture is what carries the change.
func _set_paint(on: bool) -> void:
	_br.world_gen.debug_set_river_paint(on)
	_vh.map_view.texture = _br.color_texture()
	_vh._apply_shore_field()
	_vh.overlay.queue_redraw()
	await _settle(8)


## ON-vs-OFF diff mask at two thresholds (summed |dRGB| > 24, the stroke
## probe's own, and > 8, which sees the antialiased fringe).
func _diff_masks(on: Image, off: Image) -> Dictionary:
	var w := on.get_width(); var h := on.get_height()
	var da := on.get_data(); var db := off.get_data()
	var m24 := PackedByteArray(); m24.resize(w * h)
	var m8 := PackedByteArray(); m8.resize(w * h)
	for i in w * h:
		var o := i * 4
		var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
		if dd > 24: m24[i] = 1
		if dd > 8: m8[i] = 1
	return {"m24": m24, "m8": m8, "w": w, "h": h}


## Count of set pixels in `a`, in `b`, in both, and the intersection over the
## union. A mask of zero pixels gives `iou = -1` (never a plausible 0 or 1).
func _overlap(a: PackedByteArray, b: PackedByteArray) -> Dictionary:
	var na := 0; var nb := 0; var both := 0
	for i in a.size():
		var x := a[i] == 1; var y := b[i] == 1
		if x: na += 1
		if y: nb += 1
		if x and y: both += 1
	var uni := na + nb - both
	return {"painted": na, "stroke": nb, "both": both, "iou": (float(both) / float(uni)) if uni > 0 else -1.0}


## Frame interval while the view pans a few cells back and forth: median with
## min..max (a mean hides the stalls), plus the GPU render time the driver
## reports for the main viewport, when it does (0 where it does not).
func _pan_cost(centre: Vector2, z: float) -> Dictionary:
	var vp := get_viewport().get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(vp, true)
	_vh.reset_view()
	await _settle(3)
	_vh.zoom_step(z / _vh.zoom())
	var dt := PackedFloat32Array(); var gpu := PackedFloat32Array()
	var t0 := Time.get_ticks_usec()
	for f in 90:
		var a := float(f) * 0.35
		_vh.move_view_to(centre.x + sin(a) * 6.0, centre.y + cos(a) * 4.0)
		await RenderingServer.frame_post_draw
		var t1 := Time.get_ticks_usec()
		if f >= 10:
			dt.append((t1 - t0) / 1000.0)
			gpu.append(RenderingServer.viewport_get_measured_render_time_gpu(vp))
		t0 = t1
	RenderingServer.viewport_set_measure_render_time(vp, false)
	var s := _stats(dt)
	var a2 := Array(dt); a2.sort()
	var g2 := Array(gpu); g2.sort()
	return {"median_ms": s["median"], "min_ms": a2[0], "max_ms": a2[a2.size() - 1],
		"gpu_median_ms": g2[g2.size() / 2], "gpu_max_ms": g2[g2.size() - 1]}


## Centreline samples of one junction: the river body, 2.5 cells back from the
## end `e` along the unit downstream tangent `t`, every quarter cell.
func _junction_samples(e: Vector2, t: Vector2) -> Array:
	var out: Array = []
	for k in range(0, 11):
		out.append(e - t * (0.25 * float(k)))
	return out


## Junctions of the three kinds across the drawn network, up to `cap` each:
## {"kind", "e" (the end of a drawn piece), "t" (unit downstream tangent)}.
## `mouth`: the last piece ends in water. `lake_break`: a piece that ends where
## the next begins past a lake. `join`: the last piece ends on land (a
## tributary meeting another river).
func _junctions(rivers: Array, cap: int) -> Array:
	var out: Array = []
	var n := {"mouth": 0, "lake_break": 0, "join": 0}
	for r: Dictionary in rivers:
		if not _drawable(r):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var pcs := _pieces_of(r)
		for k in pcs.size():
			var pc: Vector2i = pcs[k]
			if pc.y - pc.x < 3:
				continue
			var e := rp[pc.y - 1]
			var t := (e - rp[pc.y - 2]).normalized()
			if not _inner(e) or t.length() < 0.5:
				continue
			var kind := "join"
			if k < pcs.size() - 1:
				kind = "lake_break"
			elif _is_wet(e + t * 0.05):
				kind = "mouth"
			if n[kind] < cap:
				n[kind] += 1
				out.append({"kind": kind, "e": e, "t": t})
	return out


## One full pass of the measurements for the CURRENT path (painted or stroke):
## ON/OFF diff masks per target and zoom below the deep-zoom switch, the seam
## counts, and the pan frame cost.
func _paint_pass(targets: Dictionary, junctions: Array) -> Dictionary:
	var out := {"masks": {}, "seams": {}, "cost": {}}
	var path_tag := "paint" if _br.world_gen.rivers_painted() else "stroke"
	for name: String in targets.keys():
		var p: Vector2 = targets[name]["p"]
		for z: float in _zooms:
			if z > 2.5 or z > _vh._zoom_max + 1e-6:
				continue
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(z / _vh.zoom())
			_vh.move_view_to(p.x, p.y)
			await _settle(30)
			## The floodplain tint (RIM-3) is part of the river's own
			## ON-vs-OFF diff, so the coverage masks are taken with it
			## switched off -- width and edge only, the stroke's own terms --
			## and the tinted frame is the one saved and its extra pixels
			## counted separately (`fp_px`).
			var mat := _vh.map_view.material as ShaderMaterial
			var tinted := await _grab()
			mat.set_shader_parameter("floodplain_strength", 0.0)
			await _settle(4)
			var on := await _grab()
			_vh.set_layer_visible("rivers", false)
			await _settle(10)
			var off := await _grab()
			_vh.set_layer_visible("rivers", true)
			mat.set_shader_parameter("floodplain_strength", 1.0)
			await _settle(10)
			if on == null or off == null or tinted == null:
				continue
			var tag := "%s_z%05.1f" % [name, z]
			out["masks"][tag] = _diff_masks(on, off)
			var fp := _diff_masks(tinted, off)
			out["masks"][tag]["fp_px"] = int(_overlap(fp["m8"], fp["m8"])["painted"]) - int(_overlap(out["masks"][tag]["m8"], out["masks"][tag]["m8"])["painted"])
			out["masks"][tag]["ppc"] = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
			tinted.save_png(_out.path_join("%s_%s_on.png" % [tag, path_tag]))
			off.save_png(_out.path_join("%s_%s_off.png" % [tag, path_tag]))
	## Seams at zoom 2 (the screen path), the view centred on each junction.
	var gap := {"mouth": 0, "lake_break": 0, "join": 0}
	var tot := {"mouth": 0, "lake_break": 0, "join": 0}
	var ctl := {"mouth": 0, "lake_break": 0, "join": 0}
	var shots := {"mouth": 0, "lake_break": 0, "join": 0}
	for j: Dictionary in junctions:
		_vh.reset_view()
		await _settle(2)
		_vh.zoom_step(2.0 / _vh.zoom())
		## move_view_to takes a cell INDEX (centres on index + 0.5); a render
		## point is continuous, so the half cell comes off.
		_vh.move_view_to(j["e"].x - 0.5, j["e"].y - 0.5)
		await _settle(8)
		var on := await _grab()
		_vh.set_layer_visible("rivers", false)
		await _settle(4)
		var off := await _grab()
		_vh.set_layer_visible("rivers", true)
		await _settle(4)
		if on == null or off == null:
			continue
		if int(shots[j["kind"]]) < 2:
			on.save_png(_out.path_join("junction_%s_%d_%s.png" % [j["kind"], int(shots[j["kind"]]), path_tag]))
			shots[j["kind"]] += 1
		var ppc: float = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
		var c := _vh.size * 0.5
		var da := on.get_data(); var db := off.get_data()
		var w := on.get_width()
		for s: Vector2 in _junction_samples(j["e"], j["t"]):
			var px: Vector2 = c + (s - j["e"]) * ppc
			var ix := int(floor(px.x)); var iy := int(floor(px.y))
			if ix < 0 or iy < 0 or ix >= w or iy >= on.get_height():
				continue
			var o := (iy * w + ix) * 4
			var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
			var d: Dictionary = _br.sample_cell(int(floor(s.x)), int(floor(s.y)))
			var land := String(d.get("water", "land")) == "land"
			tot[j["kind"]] += 1
			if dd > 6:
				ctl[j["kind"]] += 1
			elif land:
				gap[j["kind"]] += 1
	out["seams"] = {"gaps": gap, "samples": tot, "covered_samples": ctl}
	## Frame cost while panning, at the fit view and at the screen path's
	## deepest zoom.
	var c0: Vector2 = targets["trunk"]["p"] if targets.has("trunk") else Vector2(_grid.x * 0.5, _grid.y * 0.5)
	for z: float in [1.0, 2.0]:
		out["cost"]["z%.0f" % z] = await _pan_cost(c0, z)
	return out


func _paint_compare_run(rivers: Array, targets: Dictionary) -> Dictionary:
	var res := {"stats": _br.world_gen.river_paint_stats(), "painted_flag": _br.world_gen.rivers_painted()}
	print("  river_paint_stats: ", res["stats"], "  rivers_painted=", res["painted_flag"])
	if not bool(res["painted_flag"]):
		printerr("PROBE-FAIL: the river field was not built (rivers_painted false) -- nothing to compare")
		_report["fail"] = true
		return res
	var junctions := _junctions(rivers, 8)
	print("  junctions sampled: ", junctions.size())
	var paint := await _paint_pass(targets, junctions)
	await _set_paint(false)
	if _br.world_gen.rivers_painted():
		printerr("PROBE-FAIL: debug_set_river_paint(false) left the paint on")
		_report["fail"] = true
	var stroke := await _paint_pass(targets, junctions)
	await _set_paint(true)
	var rows := {}
	for tag: String in paint["masks"].keys():
		if not stroke["masks"].has(tag):
			continue
		var pm: Dictionary = paint["masks"][tag]
		var sm: Dictionary = stroke["masks"][tag]
		var o24 := _overlap(pm["m24"], sm["m24"])
		var o8 := _overlap(pm["m8"], sm["m8"])
		var ratio := (float(o8["painted"]) / float(o8["stroke"])) if int(o8["stroke"]) > 0 else -1.0
		rows[tag] = {"ppc": pm["ppc"], "gt24": o24, "gt8": o8, "coverage_ratio_gt8": ratio, "floodplain_px_gt8": pm.get("fp_px", 0)}
		print("    %s ppc %.2f  >24: painted %d stroke %d iou %.3f   >8: painted %d stroke %d iou %.3f  ratio %.3f  floodplain_px %d"
			% [tag, pm["ppc"], o24["painted"], o24["stroke"], o24["iou"], o8["painted"], o8["stroke"], o8["iou"], ratio, int(pm.get("fp_px", 0))])
		if int(o24["painted"]) == 0 or int(o24["stroke"]) == 0:
			printerr("PROBE-FAIL: a path drew no pixels at %s (positive control)" % tag)
			_report["fail"] = true
	res["coverage"] = rows
	res["seams_paint"] = paint["seams"]
	res["seams_stroke"] = stroke["seams"]
	res["cost_paint"] = paint["cost"]
	res["cost_stroke"] = stroke["cost"]
	print("  seams (gap = a land centreline sample the river layer did not change)")
	print("    painted: ", paint["seams"])
	print("    stroke : ", stroke["seams"])
	print("  pan frame cost painted: ", paint["cost"])
	print("  pan frame cost stroke : ", stroke["cost"])
	for k in ["mouth", "lake_break", "join"]:
		if int(paint["seams"]["covered_samples"][k]) == 0 and int(paint["seams"]["samples"][k]) > 0:
			printerr("PROBE-FAIL: no %s sample was ever covered by the painted river (positive control)" % k)
			_report["fail"] = true
	return res


## ---- RIM-5: headwater streams fade out as the map zooms out ---------------

## The river's ink on the CURRENT path at camera zoom `z`, the view centred on
## the map: the ON-vs-OFF diff above 8 summed levels (the antialiased fringe's
## own threshold), with the floodplain tint off so only the river's width and
## edge count. Saves the ON frame (tint on, as the user sees it) as
## `<tag>.png`. Returns `{ink, mass, ppc}`; `ink = -1` when a frame could not
## be grabbed, never a plausible 0.
func _rim5_ink(z: float, tag: String) -> Dictionary:
	_vh.reset_view()
	await _settle(3)
	_vh.zoom_step(z / _vh.zoom())
	_vh.move_view_to(_grid.x * 0.5, _grid.y * 0.5)
	await _settle(24)
	var mat := _vh.map_view.material as ShaderMaterial
	var tinted := await _grab()
	mat.set_shader_parameter("floodplain_strength", 0.0)
	await _settle(4)
	var on := await _grab()
	_vh.set_layer_visible("rivers", false)
	await _settle(8)
	var off := await _grab()
	_vh.set_layer_visible("rivers", true)
	mat.set_shader_parameter("floodplain_strength", 1.0)
	await _settle(6)
	if on == null or off == null or tinted == null:
		return {"ink": -1, "ppc": 0.0}
	tinted.save_png(_out.path_join(tag + ".png"))
	var m := _diff_masks(on, off)
	var n := 0
	for b in (m["m8"] as PackedByteArray):
		if b == 1: n += 1
	var da := on.get_data(); var db := off.get_data()
	var mass := 0
	for i in on.get_width() * on.get_height():
		var o := i * 4
		mass += absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
	var ppc: float = minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()
	return {"ink": n, "mass": mass, "ppc": ppc}


## The RIM-5 leg (see the header): painted ink against the stroke path's over a
## zoom sweep, `kept` per zoom and the largest step in `kept` between
## neighbouring zooms.
func _rim5_run() -> Dictionary:
	var paint := {}; var stroke := {}
	for path in ["paint", "stroke"]:
		await _set_paint(path == "paint")
		for z: float in _rim5_zooms:
			if z > _vh._zoom_max + 1e-6:
				continue
			var r := await _rim5_ink(z, "rim5_z%05.2f_%s" % [z, path])
			if path == "paint":
				paint[z] = r
			else:
				stroke[z] = r
	await _set_paint(true)
	var rows := []
	var prev := -1.0
	var max_step := 0.0
	var kept_min := 9.0
	for z: float in _rim5_zooms:
		if not paint.has(z) or not stroke.has(z):
			continue
		var pi: int = paint[z]["ink"]; var si: int = stroke[z]["ink"]
		var pm: int = paint[z].get("mass", -1); var sm: int = stroke[z].get("mass", -1)
		var kept := (float(pi) / float(si)) if si > 0 and pi >= 0 else -1.0
		var kept_mass := (float(pm) / float(sm)) if sm > 0 and pm >= 0 else -1.0
		rows.append({"z": z, "ppc": paint[z]["ppc"], "paint_ink": pi, "stroke_ink": si, "kept": kept,
			"paint_mass": pm, "stroke_mass": sm, "kept_mass": kept_mass})
		print("    rim5 z %.2f ppc %.3f  ink paint %d stroke %d kept %.3f   mass paint %d stroke %d kept %.3f"
			% [z, paint[z]["ppc"], pi, si, kept, pm, sm, kept_mass])
		if kept >= 0.0:
			if prev >= 0.0:
				max_step = maxf(max_step, absf(kept - prev))
			prev = kept
			kept_min = minf(kept_min, kept)
	print("  rim5: max step in kept between neighbouring zooms %.3f, min kept %.3f" % [max_step, kept_min])
	var last: Dictionary = rows[rows.size() - 1] if rows.size() > 0 else {}
	if rows.is_empty() or int(last["paint_ink"]) <= 0 or int(last["stroke_ink"]) <= 0:
		printerr("PROBE-FAIL: a path drew no river ink at the deepest rim5 zoom (positive control)")
		_report["fail"] = true
	return {"rows": rows, "max_step_kept": max_step, "min_kept": kept_min}


## ---- RIM-2: the bank outline -------------------------------------------------

## The render workspace instance (the one that owns `STYLE_PRESETS` and
## `_apply_preset`), found by what it is rather than by a path, so the probe
## applies the SHIPPED preset data -- bank strengths included -- through the
## shell's own code. `null` when no node carries them (the leg then fails
## loudly; it never falls back to a made-up preset).
func _find_render_workspace(n: Node) -> Node:
	var sc: Script = n.get_script()
	if sc != null and sc.get_script_constant_map().has("STYLE_PRESETS") and n.has_method("_apply_preset"):
		return n
	for c in n.get_children():
		var r := _find_render_workspace(c)
		if r != null:
			return r
	return null


## A file-name-safe tag for a preset name ("Ink wash" -> "ink_wash").
func _slug(s: String) -> String:
	return s.to_lower().replace(" / ", "_").replace(" ", "_").replace("/", "_")


## Mean absolute difference per channel (0..255) between two same-size
## frames, and the count of pixels whose summed |dRGB| exceeds 8 (the
## antialiased fringe's threshold, as `_diff_masks`). `mad = -1` when the
## frames are missing or differ in size -- never a plausible 0 (a comparison
## that could not be made must not read as "identical").
func _frame_diff(a: Image, b: Image) -> Dictionary:
	if a == null or b == null or a.get_size() != b.get_size():
		return {"mad": -1.0, "px8": -1, "px_any": -1}
	var da := a.get_data(); var db := b.get_data()
	var sum := 0; var px8 := 0; var any := 0
	for i in a.get_width() * a.get_height():
		var o := i * 4
		var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
		sum += dd
		if dd > 8: px8 += 1
		if dd > 0: any += 1
	return {"mad": float(sum) / float(a.get_width() * a.get_height() * 3), "px8": px8, "px_any": any}


## One captured frame of the current style at camera zoom `z` on `centre`,
## once the view (and, past the deep-zoom switch, the pyramid) has settled.
func _rim2_grab(centre: Vector2, z: float) -> Image:
	_vh.reset_view()
	await _settle(3)
	_vh.zoom_step(z / _vh.zoom())
	_vh.move_view_to(centre.x, centre.y)
	await _settle(24)
	await _settle_lod()
	return await _grab()


## The RIM-2 leg (see the header): per preset and view, the style as shipped,
## the same frame with the bank forced to 0 (when the preset has one), and --
## for presets that ship none -- the frame with it forced to `_rim2_force` so a
## clean style's outline can be read too. Every frame is saved; the report
## carries `{bank, view, ppc, lod, mad, px8, px_any}` per comparison.
## **`--rim2-head` skips every bank-specific step** (it records the shipped
## frames only), because a HEAD build has no `river_bank` uniform: the run on a
## HEAD tree and the run on this tree then produce frames a script can compare
## pixel for pixel (`off == HEAD`).
func _rim2_run(targets: Dictionary) -> Dictionary:
	var ws := _find_render_workspace(_app)
	if ws == null:
		printerr("PROBE-FAIL: no render workspace found (RIM-2 leg cannot apply presets)")
		_report["fail"] = true
		return {}
	var presets: Array = (ws.get_script() as Script).get_script_constant_map()["STYLE_PRESETS"]
	var names: Array = []
	for p in presets:
		names.append(String(p[0]))
	var wanted: Array = names if _rim2_presets.is_empty() else _rim2_presets
	var fit_c := Vector2(_grid.x * 0.5, _grid.y * 0.5)
	var zoom_c: Vector2 = targets["trunk"]["p"] if targets.has("trunk") else fit_c
	var views := [["fit", 1.0, fit_c], ["z%.1f" % _rim2_zoom, _rim2_zoom, zoom_c]]
	var mat: ShaderMaterial = null
	var rows := []
	var determinism := -2.0
	for nm: String in wanted:
		var idx := names.find(nm)
		if idx < 0:
			printerr("PROBE-FAIL: no preset named %s" % nm)
			_report["fail"] = true
			continue
		ws._apply_preset(idx)
		await _settle(8)
		await _settle_lod()
		mat = _vh.map_view.material as ShaderMaterial
		var bv: Variant = mat.get_shader_parameter("river_bank")
		var bank := float(bv) if bv != null else -1.0
		var ink: Variant = mat.get_shader_parameter("river_bank_color")
		for v in views:
			var tag := "rim2_%s_%s" % [_slug(nm), String(v[0])]
			var shipped := await _rim2_grab(v[2], float(v[1]))
			if shipped == null:
				continue
			shipped.save_png(_out.path_join(tag + "_shipped.png"))
			var row := {"preset": nm, "view": String(v[0]), "bank": bank, "lod": _vh.lod_active(),
				"ppc": minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * _vh.zoom()}
			if not _rim2_head:
				if bank > 0.0:
					mat.set_shader_parameter("river_bank", 0.0)
					await _settle(4)
					var off := await _grab()
					off.save_png(_out.path_join(tag + "_off.png"))
					row["vs_off"] = _frame_diff(shipped, off)
					## Determinism control: the same OFF frame again must be
					## pixel-identical, or every "MAD 0" below is luck.
					if determinism < -1.5:
						await _settle(4)
						var off2 := await _grab()
						determinism = float(_frame_diff(off, off2)["mad"])
						row["determinism_mad"] = determinism
					mat.set_shader_parameter("river_bank", bank)
					await _settle(4)
				elif _rim2_force > 0.0:
					mat.set_shader_parameter("river_bank", _rim2_force)
					await _settle(4)
					var forced := await _grab()
					forced.save_png(_out.path_join(tag + "_forced.png"))
					row["forced"] = _rim2_force
					row["vs_forced"] = _frame_diff(forced, shipped)
					mat.set_shader_parameter("river_bank", bank)
					await _settle(4)
			rows.append(row)
			print("    rim2 %-16s %-5s bank %.2f ppc %.2f lod=%s  %s" % [nm, String(v[0]), bank, row["ppc"], str(row["lod"]),
				str(row.get("vs_off", row.get("vs_forced", "(shipped only)")))])
		if nm == _rim2_cost_preset:
			var cost := {}
			for v in views:
				cost[String(v[0])] = {"shipped": await _pan_cost(v[2], float(v[1]))}
				if not _rim2_head and bank > 0.0:
					mat.set_shader_parameter("river_bank", 0.0)
					cost[String(v[0])]["off"] = await _pan_cost(v[2], float(v[1]))
					mat.set_shader_parameter("river_bank", bank)
			print("  rim2 frame cost (%s): %s" % [nm, str(cost)])
			rows.append({"preset": nm, "cost": cost})
	## Positive controls (never vacuous): a preset that ships a bank must move
	## pixels at the zoomed view, and the repeat of one frame must be identical.
	if not _rim2_head:
		var moved := false
		var had_bank := false
		for r in rows:
			if r.has("bank") and float(r["bank"]) > 0.0:
				had_bank = true
				if r["view"] != "fit" and int(r["vs_off"]["px8"]) > 0:
					moved = true
		if had_bank and not moved:
			printerr("PROBE-FAIL: a preset with a bank drew no outline pixels at the zoomed view (positive control)")
			_report["fail"] = true
		if had_bank and determinism != 0.0:
			printerr("PROBE-FAIL: the same frame grabbed twice differs (MAD %s); identity checks mean nothing" % str(determinism))
			_report["fail"] = true
	return {"rows": rows}


## ---- RIM-4 Part A: what real river mouths look like (grid data, no pixels) ----

## Every drawn river whose last drawn point stands on water (`_is_wet`: the
## engine's own `water` class at that cell -- `extend_shore_ends` carries every
## end that meets the sea or a lake one point INTO it, so a mouth is exactly an
## end that is wet; a confluence ends on land, on another river's cell) is a
## MOUTH. For each: the water it meets (`ocean` / `lake`), `own_order` (the
## run's Strahler order, 1 = a headwater trickle), the traced `order`, the
## reading of `WorldState::flow_discharge` at the outlet (`mouth_discharge`)
## and its largest on the run (`discharge`), that discharge as a fraction of
## the grid's cell count (`frac` from the outlet reading, `dfrac` from the
## largest -- the one the engine's fan rule reads),
## and the drawn full width at the mouth point in cells. Returns the raw rows
## and, per water kind and for all, the min / median / p90 / max of each
## measure, so the distribution (not a round number) picks the threshold
## (`RIVERS_IN_MAP_SCOPE.md` owner question 3). Never reports a river with no
## drawn piece as a mouth, and never a missing reading as 0 (a river without
## `mouth_discharge` is counted in `no_discharge` and left out of its stats).
func _rim4_mouth_stats(rivers: Array) -> Dictionary:
	var rows: Array = []
	var no_discharge := 0
	var cells := float(_grid.x * _grid.y)
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var pcs := _pieces_of(r)
		if pcs.is_empty():
			continue
		var rp: PackedVector2Array = r["render_points"]
		var e_i: int = pcs[pcs.size() - 1].y - 1
		var e := rp[e_i]
		if not _is_wet(e):
			continue
		var w := String(_br.sample_cell(int(floor(e.x)), int(floor(e.y))).get("water", "land"))
		var row := {"river": ri, "water": w, "own_order": int(r.get("own_order", 0)), "order": int(r.get("order", 0)),
			"width_cells": _width_at(r, e_i), "x": e.x, "y": e.y}
		if r.has("mouth_discharge"):
			row["mouth_discharge"] = float(r["mouth_discharge"])
			row["frac"] = float(r["mouth_discharge"]) / cells
		else:
			no_discharge += 1
		if r.has("discharge"):
			row["discharge"] = float(r["discharge"])
			## What `river_delta::delta_fans` reads: the run's largest discharge
			## (the `mouth_discharge` reading at the outlet cell is lower on
			## 26 % / 37 % / 57 % of the Part A worlds' mouths, so a rule keyed
			## to it would pick different mouths from the engine's).
			row["dfrac"] = float(r["discharge"]) / cells
		rows.append(row)
	var out := {"mouths": rows.size(), "drawn": 0, "no_discharge": no_discharge, "cells": cells}
	for r: Dictionary in rivers:
		if _drawable(r):
			out["drawn"] = int(out["drawn"]) + 1
	for kind in ["all", "ocean", "lake"]:
		var sel: Array = rows.filter(func(rw): return kind == "all" or rw["water"] == kind)
		var s := {"n": sel.size()}
		for key in ["own_order", "order", "mouth_discharge", "discharge", "frac", "dfrac", "width_cells"]:
			var v := PackedFloat32Array()
			for rw: Dictionary in sel:
				if rw.has(key):
					v.append(float(rw[key]))
			if v.is_empty():
				continue
			var a := Array(v); a.sort()
			s[key] = {"n": a.size(), "min": a[0], "median": a[a.size() / 2], "p90": a[int(a.size() * 0.9)], "max": a[a.size() - 1]}
		out[kind] = s
	out["rows"] = rows
	print("  rim4 mouths: ", {"mouths": out["mouths"], "drawn": out["drawn"], "ocean": (out["ocean"] as Dictionary)["n"], "lake": (out["lake"] as Dictionary)["n"]})
	for kind in ["all", "ocean", "lake"]:
		print("    ", kind, ": ", JSON.stringify(out[kind]))
	return out


## ---- RIM-4 Part B/C: the delta fans on the screen painted path ----------------

## Switch RIM-4's delta fans (`WorldGen::set_river_deltas`) and repaint so the
## next capture shows it, as `_set_paint` does for the paint path. A build
## before RIM-4 has no such method: `--rim4-head` never calls this. Since RIM-7
## the deep-zoom tiles stroke the fans too and their key carries the switch
## (`lod_cache_key`'s `dl`), so the live tiles are dropped and rebuilt after the
## repaint (which also re-keys the grid raster the tiles read); on a build
## before RIM-7 that rebuild draws the same tiles again.
func _set_deltas(on: bool) -> void:
	_br.world_gen.set_river_deltas(on)
	_vh.map_view.texture = _br.color_texture()
	_vh._apply_shore_field()
	_vh.overlay.queue_redraw()
	_vh.invalidate_lod_tiles()
	await _settle(8)


## Whether camera zoom `z` can CENTRE the view on cell `p` -- the whole view
## then lies inside the map. The camera clamps otherwise, and a target near the
## edge is framed off-centre (the capture is still valid, but a reader of the
## PNG then cannot tell where the target is).
func _rim4_centrable(p: Vector2, z: float) -> bool:
	var ppc := minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * z
	var half := _vh.size / (2.0 * ppc)
	return p.x >= half.x and p.y >= half.y and p.x <= _grid.x - half.x and p.y <= _grid.y - half.y


## The mouths a fan is expected at: the rows of `_rim4_mouth_stats` the engine's
## own rule keeps (`own_order` >= `min_order` and `dfrac` >= `min_frac`, read back
## from `river_delta_stats` so the probe cannot drift from the constants; on a
## HEAD build, which has no such stats, the caller passes the same pair so both
## builds pick the same mouths). Returns up to `n`: the largest lake mouth, one
## near the median of the eligible and then the largest ocean mouths, so the
## sample spans the range rather than being `n` look-alikes. Only mouths away
## from the map's edge. An empty return is a failure the caller reports, never
## a result.
func _rim4_pick(rows: Array, min_order: int, min_frac: float, n: int) -> Array:
	var el: Array = rows.filter(func(r): return int(r["own_order"]) >= min_order and r.has("dfrac") and float(r["dfrac"]) >= min_frac)
	el.sort_custom(func(a, b): return float(a["dfrac"]) > float(b["dfrac"]))
	var out: Array = []
	var seen := {}
	var add := func(r: Dictionary) -> void:
		if not seen.has(r["river"]) and out.size() < n:
			seen[r["river"]] = true
			out.append(r)
	## Centrable at z 2.1 (the painted path's upper range) first; the rest only
	## fill what is left, so the sample is mostly frames the target is IN the
	## middle of.
	var inner: Array = el.filter(func(r): return _inner(Vector2(r["x"], r["y"])) and _rim4_centrable(Vector2(r["x"], r["y"]), 2.1))
	if inner.size() < n:
		inner = el.filter(func(r): return _inner(Vector2(r["x"], r["y"])))
	var oc: Array = inner.filter(func(r): return r["water"] == "ocean")
	var lk: Array = inner.filter(func(r): return r["water"] == "lake")
	if lk.size() > 0: add.call(lk[0])
	if inner.size() > 2: add.call(inner[inner.size() / 2])
	for r in oc: add.call(r)
	for r in inner: add.call(r)
	return out


## The RIM-4 leg: per picked mouth and zoom, the frame with the fans ON (the
## shipped default) and with them OFF, saved as `rim4_<tag>_on.png` / `_off.png`.
## `--rim4-head` records one `_shipped.png` per view and calls nothing HEAD
## lacks, so a run on a HEAD DLL and the OFF frames of this tree compare pixel
## for pixel. Then (`--rim4-cost`) the pan frame time at the fit view and z 2.1,
## three alternating rounds so a drifting machine shows. The engine's own fan
## statistics are returned with the rows.
func _rim4_run(m: Dictionary) -> Dictionary:
	var rows: Array = m["rows"]
	var min_order := 3
	var min_frac := 2.0e-4
	var stats := {}
	if not _rim4_head:
		stats = _br.world_gen.river_delta_stats()
		if stats.is_empty():
			## The fans are built lazily, on the first paint that wants them.
			_vh.map_view.texture = _br.color_texture()
			await _settle(8)
			stats = _br.world_gen.river_delta_stats()
		if stats.is_empty():
			printerr("PROBE-FAIL: river_delta_stats is empty with the fans on (nothing was derived)")
			_report["fail"] = true
			return {}
		min_order = int(stats["min_order"])
		min_frac = float(stats["min_discharge_frac"])
		print("  rim4 engine stats: ", stats)
	var picks := _rim4_pick(rows, min_order, min_frac, _rim4_n)
	if picks.is_empty():
		printerr("PROBE-FAIL: no eligible mouth to look at (order>=%d, frac>=%.1e)" % [min_order, min_frac])
		_report["fail"] = true
		return {"stats": stats}
	var views := []
	var moved_any := false
	for pk: Dictionary in picks:
		var c := Vector2(pk["x"], pk["y"])
		for z: float in _rim4_zooms:
			var tag := "rim4_r%d_%s_z%.1f" % [int(pk["river"]), String(pk["water"]), z]
			var row := {"river": pk["river"], "water": pk["water"], "dfrac": pk["dfrac"], "own_order": pk["own_order"],
				"width_cells": pk["width_cells"], "cell": [c.x, c.y], "zoom": z}
			if _rim4_head:
				var sh := await _rim2_grab(c, z)
				sh.save_png(_out.path_join(tag + "_shipped.png"))
			else:
				await _set_deltas(true)
				var on := await _rim2_grab(c, z)
				on.save_png(_out.path_join(tag + "_on.png"))
				row["lod"] = _vh.lod_active()
				await _set_deltas(false)
				var off := await _rim2_grab(c, z)
				off.save_png(_out.path_join(tag + "_off.png"))
				var d := _frame_diff(on, off)
				row["on_vs_off"] = d
				if int(d["px_any"]) > 0:
					moved_any = true
				await _set_deltas(true)
			views.append(row)
			print("    rim4 ", tag, "  ", row.get("on_vs_off", "(shipped only)"))
	## Positive control (never vacuous): with eligible mouths and the fans on,
	## SOME frame at SOME zoom must differ from its OFF twin.
	if not _rim4_head and not moved_any:
		printerr("PROBE-FAIL: the fans moved no pixel in any ON-vs-OFF pair (positive control)")
		_report["fail"] = true
	var cost := {}
	if _rim4_cost:
		var c0 := Vector2(picks[0]["x"], picks[0]["y"])
		var cviews := [["fit", 1.0, Vector2(_grid.x * 0.5, _grid.y * 0.5)], ["z2.1", 2.1, c0]]
		for v in cviews:
			var key := String(v[0])
			cost[key] = {"on": [], "off": []}
			for round in 3:
				if _rim4_head:
					(cost[key]["on"] as Array).append(await _pan_cost(v[2], float(v[1])))
					continue
				await _set_deltas(true)
				(cost[key]["on"] as Array).append(await _pan_cost(v[2], float(v[1])))
				await _set_deltas(false)
				(cost[key]["off"] as Array).append(await _pan_cost(v[2], float(v[1])))
			if not _rim4_head:
				await _set_deltas(true)
		print("  rim4 frame cost: ", JSON.stringify(cost))
	return {"stats": stats, "views": views, "cost": cost, "min_order": min_order, "min_frac": min_frac}


## RIM-4 Part C: rivers that pass THROUGH a lake (two drawn pieces with the lake
## between them, a crossing of 12+ cells) on this world, the `_rim4_n` of the
## highest own order that a z-2 camera can centre on. Frames
## at the gap's midpoint with the painted path ON at z 2, 3 and 4, and at z 2
## the stroke path (paint OFF) as the contrast; saved as
## `rim4_lake_r<i>_z<z>.png` / `_stroke.png`. Records the gap's length in cells.
## Never reports a seam by itself: "no bead, no break" is not a number this
## probe can honestly compute, so the PNGs are read by eye.
func _rim4_lakes_run(rivers: Array) -> Dictionary:
	var cand: Array = []
	for ri in rivers.size():
		var r: Dictionary = rivers[ri]
		if not _drawable(r):
			continue
		var pcs := _pieces_of(r)
		if pcs.size() < 2:
			continue
		var rp: PackedVector2Array = r["render_points"]
		for k in pcs.size() - 1:
			var a := pcs[k].y - 1
			var b := pcs[k + 1].x
			if b <= a or b >= rp.size():
				continue
			var mid := (rp[a] + rp[b]) * 0.5
			var gap := rp[a].distance_to(rp[b])
			if not _inner(mid) or gap < 12.0 or not _rim4_centrable(mid, 2.0):
				continue
			cand.append({"river": ri, "w": _width_at(r, a), "mid": mid, "gap_cells": gap, "order": int(r.get("own_order", 0))})
	## Biggest rivers first (own order, then width), then the longest crossing:
	## a one-cell headwater across a puddle is not the case Part C is about.
	cand.sort_custom(func(x, y):
		if x["order"] != y["order"]: return x["order"] > y["order"]
		if x["w"] != y["w"]: return x["w"] > y["w"]
		return x["gap_cells"] > y["gap_cells"])
	print("  rim4 lakes: %d through-lake gaps on drawn rivers; imaging the %d highest-order" % [cand.size(), mini(cand.size(), _rim4_n)])
	var out: Array = []
	for i in mini(cand.size(), _rim4_n):
		var cd: Dictionary = cand[i]
		var c: Vector2 = cd["mid"]
		for z in [2.0, 3.0, 4.0]:
			var g := await _rim2_grab(c, z)
			g.save_png(_out.path_join("rim4_lake_r%d_z%.1f.png" % [int(cd["river"]), z]))
		await _set_paint(false)
		var st := await _rim2_grab(c, 2.0)
		st.save_png(_out.path_join("rim4_lake_r%d_z2.0_stroke.png" % int(cd["river"])))
		await _set_paint(true)
		out.append({"river": cd["river"], "width_cells": cd["w"], "own_order": cd["order"], "gap_cells": cd["gap_cells"], "cell": [c.x, c.y]})
	if cand.is_empty():
		printerr("PROBE-FAIL: no river passes through a lake on this world; Part C has nothing to image")
		_report["fail"] = true
	return {"gaps": cand.size(), "imaged": out}


## ---- RIM-7: the delta on the deep-zoom tiles and in the export ----------------

## Pixels per grid cell at camera zoom `z` (the same formula every leg uses).
func _ppc_at(z: float) -> float:
	return minf(_vh.size.x / float(_grid.x), _vh.size.y / float(_grid.y)) * z


## One view of `c` at camera zoom `z`, settled (pyramid included), grabbed
## twice: `[tinted, plain]` -- the frame as the user sees it, then the same
## frame with the painted path's floodplain tint off (no effect on a tile, which
## has none), so an ON-vs-OFF diff of `plain` frames is the fan's water alone.
func _rim7_grab(c: Vector2, z: float) -> Array:
	var tinted := await _rim2_grab(c, z)
	var mat := _vh.map_view.material as ShaderMaterial
	mat.set_shader_parameter("floodplain_strength", 0.0)
	await _settle(4)
	var plain := await _grab()
	mat.set_shader_parameter("floodplain_strength", 1.0)
	await _settle(4)
	return [tinted, plain]


## The fan inside a disk of `_rim7_radius` cells round `c`, from an ON and an
## OFF frame of the same view at `ppc` pixels per cell centred on `c`: pixel
## counts over 8 and over 24 summed levels and the same as areas in cells
## (`fp8_cells`, `fp24_cells` -- pixels / ppc^2, comparable across zooms), the
## mean ON and OFF colour of the over-24 pixels, the summed colour change per
## cell (`mass_cells`). `mean_on` is absent (never a plausible black) when no
## pixel is over 24.
func _rim7_measure(on: Image, off: Image, ppc: float) -> Dictionary:
	if on == null or off == null or on.get_size() != off.get_size():
		return {"px8": -1}
	var w := on.get_width(); var h := on.get_height()
	var da := on.get_data(); var db := off.get_data()
	var cpx := Vector2(w, h) * 0.5
	var r2 := (_rim7_radius * ppc) * (_rim7_radius * ppc)
	var n8 := 0; var n24 := 0; var mass := 0
	var son := Vector3.ZERO; var soff := Vector3.ZERO
	for y in h:
		var dy := float(y) + 0.5 - cpx.y
		for x in w:
			var dx := float(x) + 0.5 - cpx.x
			if dx * dx + dy * dy > r2:
				continue
			var o := (y * w + x) * 4
			var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
			mass += dd
			if dd > 8: n8 += 1
			if dd > 24:
				n24 += 1
				son += Vector3(da[o], da[o + 1], da[o + 2])
				soff += Vector3(db[o], db[o + 1], db[o + 2])
	var out := {"px8": n8, "px24": n24, "fp8_cells": n8 / (ppc * ppc), "fp24_cells": n24 / (ppc * ppc), "mass_cells": mass / (ppc * ppc)}
	if n24 > 0:
		var a := son / float(n24); var b := soff / float(n24)
		out["mean_on"] = [snappedf(a.x, 0.1), snappedf(a.y, 0.1), snappedf(a.z, 0.1)]
		out["mean_off"] = [snappedf(b.x, 0.1), snappedf(b.y, 0.1), snappedf(b.z, 0.1)]
	return out


## The RIM-7 leg (see the header). Returns `{stats, views, continuity, export,
## cost}`.
func _rim7_run(m: Dictionary) -> Dictionary:
	var stats: Dictionary = _br.world_gen.river_delta_stats()
	if stats.is_empty():
		_vh.map_view.texture = _br.color_texture()
		await _settle(8)
		stats = _br.world_gen.river_delta_stats()
	if stats.is_empty():
		printerr("PROBE-FAIL: river_delta_stats is empty with the fans on (nothing was derived)")
		_report["fail"] = true
		return {}
	print("  rim7 engine stats: ", stats)
	var picks := _rim4_pick(m["rows"], int(stats["min_order"]), float(stats["min_discharge_frac"]), _rim4_n)
	if picks.is_empty():
		printerr("PROBE-FAIL: no eligible mouth to look at")
		_report["fail"] = true
		return {"stats": stats}
	var views := []
	var past_switch_moved := false
	## Negative control: one frame grabbed twice must be the same frame, or
	## every ON-vs-OFF count below is noise.
	var c0 := Vector2(picks[0]["x"], picks[0]["y"])
	var g1 := await _rim2_grab(c0, 3.0)
	var g2 := await _grab()
	var same := _frame_diff(g1, g2)
	print("  rim7 negative control (z3 grabbed twice): ", same)
	if int(same["px_any"]) != 0:
		printerr("PROBE-FAIL: one frame grabbed twice differs (%d px); the ON-vs-OFF counts are noise" % int(same["px_any"]))
		_report["fail"] = true
	## The switch pair (the last zoom at or below 2.2, the first above it).
	var below := -1.0; var above := 99.0
	for z: float in _rim7_zooms:
		if z <= 2.2: below = maxf(below, z)
		else: above = minf(above, z)
	for pk: Dictionary in picks:
		var c := Vector2(pk["x"], pk["y"])
		for z: float in _rim7_zooms:
			var tag := "rim7_r%d_%s_z%.1f" % [int(pk["river"]), String(pk["water"]), z]
			await _set_deltas(true)
			var on: Array = await _rim7_grab(c, z)
			var lod: bool = _vh.lod_active()
			await _set_deltas(false)
			var off: Array = await _rim7_grab(c, z)
			## The control at the switch pair: the same view with the Rivers
			## layer off, so the NETWORK's own footprint either side of the
			## switch (fans off, rivers on vs rivers off) is measured the same
			## way as the fan's -- the yardstick the fan's continuity is read
			## against, since the network itself changes path there too.
			var bare: Array = []
			if is_equal_approx(z, below) or is_equal_approx(z, above):
				_vh.set_layer_visible("rivers", false)
				bare = await _rim7_grab(c, z)
				_vh.set_layer_visible("rivers", true)
				await _settle_lod()
			await _set_deltas(true)
			(on[0] as Image).save_png(_out.path_join(tag + "_on.png"))
			(off[0] as Image).save_png(_out.path_join(tag + "_off.png"))
			var ppc: float = _ppc_at(z)
			var row := {"river": pk["river"], "water": pk["water"], "own_order": pk["own_order"], "dfrac": pk["dfrac"],
				"cell": [c.x, c.y], "zoom": z, "lod": lod, "ppc": ppc, "centred": _rim4_centrable(c, z),
				"diff": _frame_diff(on[1], off[1]), "fan": _rim7_measure(on[1], off[1], ppc)}
			if not bare.is_empty():
				row["network"] = _rim7_measure(off[1], bare[1], ppc)
			if z > 2.2 and int(row["diff"]["px_any"]) > 0:
				past_switch_moved = true
			views.append(row)
			print("    rim7 %s lod=%s ppc %.2f centred=%s  %s  %s" % [tag, str(lod), ppc, str(row["centred"]), JSON.stringify(row["diff"]), JSON.stringify(row["fan"])])
	if not past_switch_moved:
		printerr("PROBE-FAIL: past the deep-zoom switch no ON/OFF pair moved a pixel -- the tiles draw no fan")
		_report["fail"] = true
	## Continuity across the switch: the last zoom at or below 2.2 against the
	## first above it, per mouth (footprint ratio in cells, colour shift), and
	## the same ratio for the network round it (`net_*`, the yardstick).
	var cont := []
	for pk: Dictionary in picks:
		var lo: Dictionary = {}; var hi: Dictionary = {}
		for v: Dictionary in views:
			if v["river"] != pk["river"]: continue
			if is_equal_approx(float(v["zoom"]), below): lo = v
			if is_equal_approx(float(v["zoom"]), above): hi = v
		if lo.is_empty() or hi.is_empty(): continue
		var fl: Dictionary = lo["fan"]; var fh: Dictionary = hi["fan"]
		var rec := {"river": pk["river"], "water": pk["water"], "below": below, "above": above,
			"lod_below": lo["lod"], "lod_above": hi["lod"],
			"fp24_cells": [fl.get("fp24_cells", -1), fh.get("fp24_cells", -1)], "fp8_cells": [fl.get("fp8_cells", -1), fh.get("fp8_cells", -1)],
			"mass_cells": [fl.get("mass_cells", -1), fh.get("mass_cells", -1)]}
		if float(fl.get("fp24_cells", 0.0)) > 0.0:
			rec["fp24_ratio"] = float(fh.get("fp24_cells", 0.0)) / float(fl["fp24_cells"])
		if float(fl.get("fp8_cells", 0.0)) > 0.0:
			rec["fp8_ratio"] = float(fh.get("fp8_cells", 0.0)) / float(fl["fp8_cells"])
		if float(fl.get("mass_cells", 0.0)) > 0.0:
			rec["mass_ratio"] = float(fh.get("mass_cells", 0.0)) / float(fl["mass_cells"])
		if lo.has("network") and hi.has("network"):
			var nl: Dictionary = lo["network"]; var nh: Dictionary = hi["network"]
			for k in ["fp24_cells", "fp8_cells", "mass_cells"]:
				if float(nl.get(k, 0.0)) > 0.0:
					rec["net_" + k.replace("_cells", "") + "_ratio"] = float(nh.get(k, 0.0)) / float(nl[k])
			if nl.has("mean_on") and nh.has("mean_on"):
				var na: Array = nl["mean_on"]; var nb: Array = nh["mean_on"]
				rec["net_colour_shift"] = absf(na[0] - nb[0]) + absf(na[1] - nb[1]) + absf(na[2] - nb[2])
		if fl.has("mean_on") and fh.has("mean_on"):
			var a: Array = fl["mean_on"]; var b: Array = fh["mean_on"]
			rec["mean_on"] = [a, b]
			rec["colour_shift"] = absf(a[0] - b[0]) + absf(a[1] - b[1]) + absf(a[2] - b[2])
		cont.append(rec)
		print("  rim7 continuity: ", JSON.stringify(rec))
	var res := {"stats": stats, "views": views, "continuity": cont}
	if _rim7_export:
		res["export"] = await _rim7_export_run(picks)
	if _rim7_cost:
		res["cost"] = await _rim7_cost_run(c0)
	return res


## The real region export (`export_snapshot_png`, which renders through
## `export_render` -- the `export_raster_png` assembly) round each mouth at
## 8 px a cell (radius 40 cells, 640 px) and 2 px a cell (radius 120, 480 px),
## fans ON and OFF; the diff of the decoded pixels per pair.
func _rim7_export_run(picks: Array) -> Dictionary:
	var out := {}
	var dir := ProjectSettings.globalize_path(_out)
	for pk: Dictionary in picks:
		var c := Vector2i(int(pk["x"]), int(pk["y"]))
		for spec in [[40, 640], [120, 480]]:
			var imgs := {}
			for st in ["on", "off"]:
				_br.world_gen.set_river_deltas(st == "on")
				var path := dir.path_join("rim7x_r%d_%s_r%d_%s.png" % [int(pk["river"]), String(pk["water"]), int(spec[0]), st])
				var r: Dictionary = _br.world_gen.export_snapshot_png(path, c.x, c.y, int(spec[0]), int(spec[1]))
				if not bool(r.get("ok", false)):
					printerr("PROBE-FAIL: export_snapshot_png failed: ", r)
					_report["fail"] = true
					continue
				imgs[st] = Image.load_from_file(path)
				if imgs[st] != null:
					(imgs[st] as Image).convert(Image.FORMAT_RGBA8)
			_br.world_gen.set_river_deltas(true)
			var key := "r%d_%s_radius%d" % [int(pk["river"]), String(pk["water"]), int(spec[0])]
			if imgs.has("on") and imgs.has("off"):
				out[key] = _frame_diff(imgs["on"], imgs["off"])
				print("    rim7 export %s  %s" % [key, JSON.stringify(out[key])])
	## Leave the session as it was: fans on, base texture repainted.
	await _set_deltas(true)
	return out


## Frame time while panning at z 3 and 4 AFTER the pyramid settles (median
## with min..max, and the GPU render time), and the synchronous build of the
## mouth's own tile at those zooms' levels (`lod_synthesize_tile`: the first
## call after a toggle includes the tile context's rebuild, reported apart;
## then five timed calls), ON and OFF alternated three rounds.
func _rim7_cost_run(c: Vector2) -> Dictionary:
	var out := {}
	for z: float in [3.0, 4.0]:
		var key := "z%.0f" % z
		out[key] = {"pan_on": [], "pan_off": [], "tile_on_ms": [], "tile_off_ms": [], "first_on_ms": [], "first_off_ms": []}
		var lvl: int = _br.world_gen.lod_level_for_zoom(_ppc_at(z))
		var n: int = _br.world_gen.lod_tiles_per_axis(lvl)
		var col := int(floor(c.x / (float(_grid.x - 1) / n)))
		var row := int(floor(c.y / (float(_grid.y - 1) / n)))
		out[key]["tile"] = [lvl, col, row]
		for round in 3:
			for st in ["on", "off"]:
				await _set_deltas(st == "on")
				_vh.reset_view()
				await _settle_lod()
				var t0 := Time.get_ticks_usec()
				_br.world_gen.lod_synthesize_tile(lvl, col, row)
				(out[key]["first_%s_ms" % st] as Array).append((Time.get_ticks_usec() - t0) / 1000.0)
				for k in 5:
					t0 = Time.get_ticks_usec()
					_br.world_gen.lod_synthesize_tile(lvl, col, row)
					(out[key]["tile_%s_ms" % st] as Array).append((Time.get_ticks_usec() - t0) / 1000.0)
				_vh.reset_view()
				await _settle(3)
				_vh.zoom_step(z / _vh.zoom())
				_vh.move_view_to(c.x, c.y)
				await _settle(24)
				await _settle_lod()
				(out[key]["pan_%s" % st] as Array).append(await _pan_cost_here(c))
		for k in ["tile_on_ms", "tile_off_ms", "first_on_ms", "first_off_ms"]:
			var a: Array = (out[key][k] as Array).duplicate(); a.sort()
			out[key][k + "_summary"] = {"median": a[a.size() / 2], "min": a[0], "max": a[a.size() - 1], "n": a.size()}
		print("  rim7 cost %s: %s" % [key, JSON.stringify(out[key])])
	await _set_deltas(true)
	return out


## `_pan_cost` without its reset: the view is already at the zoom and settled,
## so the frames measured are panning over built tiles, not building them.
func _pan_cost_here(centre: Vector2) -> Dictionary:
	var vp := get_viewport().get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(vp, true)
	var dt := PackedFloat32Array(); var gpu := PackedFloat32Array()
	var t0 := Time.get_ticks_usec()
	for f in 90:
		var a := float(f) * 0.35
		_vh.move_view_to(centre.x + sin(a) * 6.0, centre.y + cos(a) * 4.0)
		await RenderingServer.frame_post_draw
		var t1 := Time.get_ticks_usec()
		if f >= 10:
			dt.append((t1 - t0) / 1000.0)
			gpu.append(RenderingServer.viewport_get_measured_render_time_gpu(vp))
		t0 = t1
	RenderingServer.viewport_set_measure_render_time(vp, false)
	var a2 := Array(dt); a2.sort()
	var g2 := Array(gpu); g2.sort()
	return {"median_ms": a2[a2.size() / 2], "min_ms": a2[0], "max_ms": a2[a2.size() - 1],
		"gpu_median_ms": g2[g2.size() / 2], "gpu_max_ms": g2[g2.size() - 1]}


## ---- RIM-7 rest: the painted river itself on the tiles and in the export ----

## Screen pixels per grid cell at the current view, from the overlay's own
## displayed rect (the base map's on-screen size) -- not assumed from a formula.
func _rim7r_ppc() -> float:
	return _vh.overlay.displayed_rect().size.x / float(_grid.x) * _vh.zoom()


## Up to three targets on land, away from the map's edge: the widest trunk
## (`_pick_targets`), the longest order-2 river and the longest order-1 stream,
## each at the middle of its drawn line. Names are fixed so a HEAD run and this
## tree's frame the same ground (the choice reads only the river network, which
## no rendering change moves).
func _rim7r_targets(rivers: Array) -> Array:
	var out := []
	var pt := _pick_targets(rivers)
	if pt.has("trunk"):
		out.append({"name": "trunk", "p": pt["trunk"]["p"]})
	for want in [[2, "mid"], [1, "small"]]:
		var best := -1; var best_p := Vector2(-1, -1)
		for r: Dictionary in rivers:
			if not r.has("width_cells") or r.has("parallel_of") or int(r.get("order", 0)) != int(want[0]):
				continue
			var rp: PackedVector2Array = r["render_points"]
			if rp.size() < 40 or int(r.get("cells", 0)) <= best:
				continue
			var p := rp[rp.size() / 2]
			if _inner(p) and not _is_wet(p):
				best = int(r["cells"]); best_p = p
		if best > 0:
			out.append({"name": want[1], "p": best_p})
	return out


## The river's own pixels in a frame, resampled onto a common grid in CELL space
## (`step` cells apart, inside a disk of `_rim7r_radius` cells round `c`), so
## frames at different zooms -- and an export -- compare the same ground. A
## grid point is river where the ON frame differs from the OFF frame (river
## opacity 0: the water removed, everything else -- floodplain tint, bank line --
## kept) by more than 24 summed levels at the nearest pixel -- or, with `lo`
## and `hi`, by more than `lo` and at most `hi` (the floodplain tint's band).
## `origin` is the image position of cell-index coordinate (0, 0). Returns `{mask, area_cells, col}`:
## `mask` 1 river / 0 not / 2 outside the disk or the frame, `col` the mean ON
## colour of the river points (absent, never black, when there are none).
func _rim7r_mask(on: Image, off: Image, origin: Vector2, ppc: float, c: Vector2, step: float, lo: int = 24, hi: int = 1 << 30) -> Dictionary:
	var w := on.get_width(); var h := on.get_height()
	var da := on.get_data(); var db := off.get_data()
	var n := int(round(2.0 * _rim7r_radius / step)) + 1
	var mask := PackedByteArray(); mask.resize(n * n)
	var cnt := 0; var sc := Vector3.ZERO
	for gy in n:
		var cy := c.y - _rim7r_radius + gy * step
		for gx in n:
			var cx := c.x - _rim7r_radius + gx * step
			var k := gy * n + gx
			if (cx - c.x) * (cx - c.x) + (cy - c.y) * (cy - c.y) > _rim7r_radius * _rim7r_radius:
				mask[k] = 2; continue
			var px := int(round(origin.x + cx * ppc - 0.5)); var py := int(round(origin.y + cy * ppc - 0.5))
			if px < 0 or py < 0 or px >= w or py >= h:
				mask[k] = 2; continue
			var o := (py * w + px) * 4
			var dd: int = absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2])
			if dd > lo and dd <= hi:
				mask[k] = 1; cnt += 1
				sc += Vector3(da[o], da[o + 1], da[o + 2])
	var res := {"mask": mask, "n": n, "step": step, "area_cells": cnt * step * step}
	if cnt > 0:
		var m := sc / float(cnt)
		res["col"] = [snappedf(m.x, 0.1), snappedf(m.y, 0.1), snappedf(m.z, 0.1)]
	return res


## Two `_rim7r_mask` results compared: the area ratio (b over a -- a width
## ratio, the river length being the same ground), IoU at zero shift, and the
## shift of `b` (in cells, +-1 cell in quarter-cell steps) that maximises IoU
## with its IoU -- a river drawn in the same place reads a best shift of 0.
## Colour shift: summed |dRGB| of the two mean river colours. `-1`, never a
## plausible 0 or 1, for a ratio or IoU that cannot be formed.
func _rim7r_compare(a: Dictionary, b: Dictionary) -> Dictionary:
	var n: int = a["n"]; var ma: PackedByteArray = a["mask"]; var mb: PackedByteArray = b["mask"]
	var step: float = a["step"]
	var iou_at := func(sx: int, sy: int) -> float:
		var both := 0; var uni := 0
		for y in n:
			var y2 := y - sy
			if y2 < 0 or y2 >= n: continue
			for x in n:
				var x2 := x - sx
				if x2 < 0 or x2 >= n: continue
				var va := ma[y * n + x]; var vb := mb[y2 * n + x2]
				if va == 2 or vb == 2: continue
				if va == 1 and vb == 1: both += 1
				if va == 1 or vb == 1: uni += 1
		return float(both) / float(uni) if uni > 0 else -1.0
	var r := int(round(1.0 / step))
	var best := -2.0; var bs := Vector2i.ZERO
	for sy in range(-r, r + 1):
		for sx in range(-r, r + 1):
			var v: float = iou_at.call(sx, sy)
			if v > best + 1e-9 or (absf(v - best) <= 1e-9 and Vector2(sx, sy).length() < Vector2(bs).length()):
				best = v; bs = Vector2i(sx, sy)
	var out := {"area_a": a["area_cells"], "area_b": b["area_cells"],
		"area_ratio": (float(b["area_cells"]) / float(a["area_cells"])) if float(a["area_cells"]) > 0.0 else -1.0,
		"iou0": iou_at.call(0, 0), "best_iou": best, "best_shift_cells": [bs.x * step, bs.y * step]}
	if a.has("col") and b.has("col"):
		var ca: Array = a["col"]; var cb: Array = b["col"]
		out["col"] = [ca, cb]
		out["colour_shift"] = absf(ca[0] - cb[0]) + absf(ca[1] - cb[1]) + absf(ca[2] - cb[2])
	return out


## Set the session's river opacity and repaint the base map and the tiles, so
## the next capture shows it. Opacity 0 removes the river's WATER on every path
## (screen, tiles, export) and keeps everything else, which is what the masks
## above diff against. Restores with 1.0 (the shipped default of every preset
## the probe runs).
func _rim7r_opacity(op: float) -> void:
	_br.set_appearance({"river_opacity": op})
	_vh.map_view.texture = _br.color_texture()
	_vh._apply_shore_field()
	_vh.overlay.queue_redraw()
	_vh.invalidate_lod_tiles()
	await _settle(8)


## The view of `c` at camera zoom `z`, settled: `[image, origin, ppc]` where
## `origin` is the screen position, in the captured image, of cell-index
## coordinate (0, 0) (`move_view_to` centres cell `c`'s centre on the view).
func _rim7r_view(c: Vector2, z: float) -> Array:
	var img := await _rim2_grab(c, z)
	var ppc := _rim7r_ppc()
	var rect: Rect2 = _vh.overlay.displayed_rect()
	## The displayed rect is in the camera's native space; the screen position of
	## grid point g is camera.position + (rect.position + g / grid * rect.size) * zoom,
	## with cell-index coordinate i at g = i + 0.5. The capture starts at the
	## viewport host's own origin.
	var cam: Control = _vh._camera
	var o: Vector2 = cam.position + (rect.position + Vector2(0.5, 0.5) / Vector2(_grid) * rect.size) * _vh.zoom()
	return [img, o, ppc]


## The RIM-7 rest leg (see the header).
func _rim7r_run(rivers: Array) -> Dictionary:
	var targets := _rim7r_targets(rivers)
	if targets.is_empty():
		printerr("PROBE-FAIL: rim7rest found no target")
		_report["fail"] = true
		return {}
	var has_paint: bool = _br.world_gen.has_method("rivers_painted") and _br.world_gen.rivers_painted()
	print("  rim7rest targets: ", targets, "  painted look: ", has_paint)
	## The field build and, once the tiles have asked for one, the paint
	## source's build (`source_ms`, `source_segments`; absent on HEAD).
	print("  rim7rest river_paint_stats: ", _br.world_gen.river_paint_stats())
	var res := {"targets": [], "views": [], "seams": [], "identity_frames": []}
	## Negative control: one frame grabbed twice must be the same frame.
	var t0: Dictionary = targets[0]
	var g1 := await _rim2_grab(t0["p"], 3.0)
	var g2 := await _grab()
	var same := _frame_diff(g1, g2)
	print("  rim7rest negative control (z3 grabbed twice): ", same)
	if int(same["px_any"]) != 0:
		printerr("PROBE-FAIL: one frame grabbed twice differs (%d px)" % int(same["px_any"]))
		_report["fail"] = true
	var step := 0.25
	for t: Dictionary in targets:
		var c: Vector2 = t["p"]
		res["targets"].append({"name": t["name"], "cell": [c.x, c.y]})
		if _rim7r_ident_only:
			continue
		var frames := {}
		## `on`: as shipped; `off`: river opacity 0 (no water; tint and line
		## kept); `bare`: the Rivers layer off (no river at all).
		for st in ["on", "off", "bare"]:
			await _rim7r_opacity(0.0 if st == "off" else 1.0)
			if st == "bare":
				_vh.set_layer_visible("rivers", false)
				_vh.invalidate_lod_tiles()
			for z: float in _rim7r_zooms:
				var v: Array = await _rim7r_view(c, z)
				frames["%s_%.1f" % [st, z]] = v
				var tag := "rim7r_%s_z%.1f_%s" % [t["name"], z, st]
				(v[0] as Image).save_png(_out.path_join(tag + ".png"))
			if st == "bare":
				_vh.set_layer_visible("rivers", true)
				_vh.invalidate_lod_tiles()
		await _rim7r_opacity(1.0)
		var masks := {}
		var lines := {}
		var tints := {}
		for z: float in _rim7r_zooms:
			var on: Array = frames["on_%.1f" % z]; var off: Array = frames["off_%.1f" % z]; var bare: Array = frames["bare_%.1f" % z]
			var m := _rim7r_mask(on[0], off[0], on[1], float(on[2]), c, step)
			## The bank line (dark ink, over 24 levels) and the floodplain tint
			## (a lift of 4..24 levels) from the opacity-0 frame against the bare one.
			var ln := _rim7r_mask(off[0], bare[0], on[1], float(on[2]), c, step)
			var tn := _rim7r_mask(off[0], bare[0], on[1], float(on[2]), c, step, 3, 24)
			masks[z] = m; lines[z] = ln; tints[z] = tn
			var row := {"target": t["name"], "zoom": z, "ppc": on[2], "lod": z > 2.2, "area_cells": m["area_cells"], "col": m.get("col", []),
				"line_cells": ln["area_cells"], "tint_cells": tn["area_cells"]}
			res["views"].append(row)
			print("    rim7rest %s z%.1f ppc %.2f river area %.1f cells^2 col %s  line %.1f  tint %.1f" % [t["name"], z, float(on[2]), float(m["area_cells"]), str(m.get("col", "-")), float(ln["area_cells"]), float(tn["area_cells"])])
		## Seams: the switch pair (last zoom <= 2.2 against first > 2.2), and
		## each deeper zoom against the first tile zoom.
		var below := -1.0; var above := 99.0
		for z: float in _rim7r_zooms:
			if z <= 2.2: below = maxf(below, z)
			else: above = minf(above, z)
		var pairs := [[below, above]]
		for z: float in _rim7r_zooms:
			if z > above: pairs.append([above, z])
		for pr in pairs:
			if not masks.has(pr[0]) or not masks.has(pr[1]): continue
			var cmp := _rim7r_compare(masks[pr[0]], masks[pr[1]])
			cmp["target"] = t["name"]; cmp["pair"] = pr
			var ta: float = tints[pr[0]]["area_cells"]; var tb: float = tints[pr[1]]["area_cells"]
			cmp["tint_cells"] = [ta, tb]
			if float(lines[pr[0]]["area_cells"]) > 0.0 and float(lines[pr[1]]["area_cells"]) > 0.0:
				var lc := _rim7r_compare(lines[pr[0]], lines[pr[1]])
				cmp["line"] = {"area_ratio": lc["area_ratio"], "iou0": lc["iou0"], "best_iou": lc["best_iou"], "best_shift_cells": lc["best_shift_cells"]}
			res["seams"].append(cmp)
			print("    rim7rest seam %s z%.1f->z%.1f %s" % [t["name"], pr[0], pr[1], JSON.stringify(cmp)])
		if _rim7r_export:
			res["export_%s" % t["name"]] = await _rim7r_export_run(c, masks, step)
	## Frames the screen path and the off switches must keep pixel-identical to
	## HEAD (compared by script across a HEAD run and this tree's): the fit view
	## and z2.1 as shipped (screen path), z3 with the Rivers layer off (tiles,
	## no river), z3 with the stroke look (`debug_set_river_paint(false)`).
	for t: Dictionary in targets:
		var c: Vector2 = t["p"]
		for z: float in [1.0, 2.1]:
			var img := await _rim2_grab(c, z)
			img.save_png(_out.path_join("rim7r_id_%s_z%.1f_screen.png" % [t["name"], z]))
		_vh.set_layer_visible("rivers", false)
		_vh.invalidate_lod_tiles()
		var nr := await _rim2_grab(c, 3.0)
		nr.save_png(_out.path_join("rim7r_id_%s_z3.0_norivers.png" % t["name"]))
		_vh.set_layer_visible("rivers", true)
		_vh.invalidate_lod_tiles()
		await _set_paint(false)
		_vh.invalidate_lod_tiles()
		var sk := await _rim2_grab(c, 3.0)
		sk.save_png(_out.path_join("rim7r_id_%s_z3.0_stroke.png" % t["name"]))
		await _set_paint(true)
		_vh.invalidate_lod_tiles()
		await _settle_lod()
	## The export under the stroke look (`debug_set_river_paint(false)`) at
	## the trunk and at the largest delta mouth: that look draws no fan on any
	## path, so its export must equal HEAD's byte for byte -- the check that an
	## export's fans follow the EXPORT's look, not a default one.
	var stats: Dictionary = _br.world_gen.river_delta_stats()
	var mouths := _rim4_mouth_stats(rivers)
	var spots := [{"name": "trunk", "p": t0["p"]}]
	if not stats.is_empty():
		var mp := _rim4_pick(mouths["rows"], int(stats["min_order"]), float(stats["min_discharge_frac"]), 1)
		if not mp.is_empty():
			spots.append({"name": "mouth", "p": Vector2(mp[0]["x"], mp[0]["y"])})
	var dir := ProjectSettings.globalize_path(_out)
	for sp: Dictionary in spots:
		var pc: Vector2 = sp["p"]
		for look in ["paint", "stroke"]:
			await _set_paint(look == "paint")
			var path := dir.path_join("rim7r_%s_%s_export.png" % ["id" if look == "stroke" else "look", sp["name"] + "_" + look])
			var r: Dictionary = _br.world_gen.export_snapshot_png(path, int(round(pc.x)), int(round(pc.y)), 30, 488)
			if not bool(r.get("ok", false)):
				printerr("PROBE-FAIL: export_snapshot_png failed: ", r)
				_report["fail"] = true
	await _set_paint(true)
	_vh.invalidate_lod_tiles()
	if _rim7r_cost:
		res["cost"] = await _rim7r_cost_run(t0["p"])
	print("  rim7rest river_paint_stats (end): ", _br.world_gen.river_paint_stats())
	return res


## The real region export (`export_snapshot_png`, the `export_raster_png`
## assembly) round `c` at 4 and 8 px a cell, ON and with the river opacity at 0,
## its river mask in cell space (export pixel `col` samples cell
## `(x0 + col) * (gw - 1) / (out_w - 1)`, `render::bake_rect`), against the
## screen's mask at the tile zoom of the nearest density.
func _rim7r_export_run(c: Vector2, masks: Dictionary, step: float) -> Dictionary:
	var out := {}
	var dir := ProjectSettings.globalize_path(_out)
	var cx := int(round(c.x)); var cy := int(round(c.y))
	for ppc_want in [4.0, 8.0]:
		var radius := int(ceil(_rim7r_radius)) + 4
		var span := 2 * radius + 1
		var size := int(round(span * ppc_want))
		var imgs := {}
		for st in ["on", "off"]:
			await _rim7r_opacity(1.0 if st == "on" else 0.0)
			var path := dir.path_join("rim7rx_%d_%d_ppc%d_%s.png" % [cx, cy, int(ppc_want), st])
			var r: Dictionary = _br.world_gen.export_snapshot_png(path, cx, cy, radius, size)
			if not bool(r.get("ok", false)):
				printerr("PROBE-FAIL: export_snapshot_png failed: ", r)
				_report["fail"] = true
				continue
			var im := Image.load_from_file(path)
			if im != null:
				im.convert(Image.FORMAT_RGBA8)
				imgs[st] = im
		await _rim7r_opacity(1.0)
		if not (imgs.has("on") and imgs.has("off")):
			continue
		## `export_snapshot_png`'s own window arithmetic (export_raster.rs).
		var virt := func(g: int) -> int: return maxi(int(round(float(g - 1) * size / float(span))), 2) + 1
		var ow: int = virt.call(_grid.x); var oh: int = virt.call(_grid.y)
		var w := mini(size, ow); var h := mini(size, oh)
		var place := func(cc: int, g: int, o: int, win: int) -> int:
			var px := float(cc) * float(maxi(o, 2) - 1) / float(g - 1)
			return int(clampf(round(px - win / 2.0), 0.0, float(o - win)))
		var x0: int = place.call(cx, _grid.x, ow, w); var y0: int = place.call(cy, _grid.y, oh, h)
		var eppc := float(ow - 1) / float(_grid.x - 1)
		## Pixel col sits at cell (x0 + col) / eppc, so cell 0 is at pixel -x0;
		## `_rim7r_mask` samples pixel round(origin + cell * ppc - 0.5), and a
		## bake pixel's centre is its index, so the origin carries +0.5.
		var origin := Vector2(-x0 + 0.5, -y0 + 0.5)
		var em := _rim7r_mask(imgs["on"], imgs["off"], origin, eppc, c, step)
		## The screen zoom of the nearest density past the switch.
		var best_z := -1.0; var best_d := 1e9
		for z: float in masks.keys():
			if z <= 2.2: continue
			var vppc: float = float(_ppc_scale()) * z
			if absf(vppc - eppc) < best_d: best_d = absf(vppc - eppc); best_z = z
		var key := "ppc%d" % int(ppc_want)
		out[key] = {"export_ppc": eppc, "export_area_cells": em["area_cells"], "export_col": em.get("col", [])}
		if best_z > 0.0:
			var cmp := _rim7r_compare(masks[best_z], em)
			cmp["screen_zoom"] = best_z
			cmp["screen_ppc"] = float(_ppc_scale()) * best_z
			out[key]["vs_screen"] = cmp
		print("    rim7rest export %s %s" % [key, JSON.stringify(out[key])])
	return out


## Screen pixels per cell at camera zoom 1 (the displayed rect's native scale).
func _ppc_scale() -> float:
	return _vh.overlay.displayed_rect().size.x / float(_grid.x)


## Cost: the pan frame time at the fit view and z2.1 (the screen path, which
## this lane must not change) and at z3 over built tiles; and the synchronous
## build of the target's tile at the z3, z4 and z8 levels (`lod_synthesize_tile`,
## five timed calls after one warm-up), the painted look against the stroke
## look (`debug_set_river_paint(false)`, the HEAD tile path), alternated three
## rounds in this one process. Median with min..max.
func _rim7r_cost_run(c: Vector2) -> Dictionary:
	var out := {}
	for round in 3:
		for st in ["paint", "stroke"]:
			await _set_paint(st == "paint")
			_vh.invalidate_lod_tiles()
			for z: float in [1.0, 2.1, 3.0]:
				var k := "pan_%s_z%.1f" % [st, z]
				if not out.has(k): out[k] = []
				_vh.reset_view()
				await _settle(3)
				_vh.zoom_step(z / _vh.zoom())
				_vh.move_view_to(c.x, c.y)
				await _settle(24)
				await _settle_lod()
				(out[k] as Array).append(await _pan_cost_here(c))
			for z: float in [3.0, 4.0, 8.0]:
				var lvl: int = _br.world_gen.lod_level_for_zoom(_ppc_scale() * z)
				var n: int = _br.world_gen.lod_tiles_per_axis(lvl)
				var col := int(floor(c.x / (float(_grid.x - 1) / n)))
				var row := int(floor(c.y / (float(_grid.y - 1) / n)))
				var k := "tile_%s_z%.0f" % [st, z]
				if not out.has(k): out[k] = []
				_br.world_gen.lod_synthesize_tile(lvl, col, row)
				for i in 5:
					var t0 := Time.get_ticks_usec()
					_br.world_gen.lod_synthesize_tile(lvl, col, row)
					(out[k] as Array).append((Time.get_ticks_usec() - t0) / 1000.0)
	await _set_paint(true)
	_vh.invalidate_lod_tiles()
	var summary := {}
	for k: String in out.keys():
		var a: Array = (out[k] as Array).duplicate()
		if k.begins_with("pan_"):
			var med := []
			for d: Dictionary in a: med.append(float(d["median_ms"]))
			med.sort()
			summary[k] = {"median_of_medians_ms": med[med.size() / 2], "min": med[0], "max": med[med.size() - 1], "n": med.size()}
		else:
			a.sort()
			summary[k] = {"median_ms": a[a.size() / 2], "min_ms": a[0], "max_ms": a[a.size() - 1], "n": a.size()}
	print("  rim7rest cost: ", JSON.stringify(summary))
	return summary
