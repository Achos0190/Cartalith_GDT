extends Node
## Probe for the owner's 2026-09-27 river messages: *"the only issue I have
## with the rivers: they're drawn on top of the style"* and *"can we put the
## rivers into the map again with the same vector approach?"*.
##
## Since then the rivers take the style. Below the deep-zoom switch the overlay
## draws RV-2's vector stroke textured with `WorldGen::river_color_texture` (the
## map with every river composited at full coverage through
## `render.rs::land_color`, the Painter styles, the paper and the grade); every
## deep-zoom tile rasterizes its own (`river_stroke.rs::rasterize`). Each
## preset carries a river treatment (`render_workspace.gd::STYLE_PRESETS`'
## `river_*` keys). This probe asserts that on a real generated world through
## the real shell:
##
##   A. the river colour texture carries the rivers and the map texture does
##      not: at known river cells (a render point within 0.3 cells of a cell
##      centre, full width >= 2.4 cells; the cell and its eight neighbours dry
##      land) the two differ a lot, and the map texture is byte-identical with
##      the Rivers layer on and off;
##   B. for each preset in PRESETS the river pixel carries THAT preset's river
##      colour -- each against a literal bar written from the preset's intent,
##      never against the preset's own constant (see `_judge`);
##   C. a deep-zoom tile at the same cell carries the same colour as the base
##      texture (the tile path draws the rivers too, at its own resolution);
##   D. the Rivers switch's cost: `set_layer_visible("rivers", ...)` timed
##      end to end (engine + base re-render), and `color_texture()` alone with
##      rivers on vs off, each a median with min..max over REPS runs.
##   S. the DRAWN river on SCREEN (see `_screen_leg`): at fit and at x1.4, for
##      SCREEN_PRESETS, the viewport's own pixels on the stroke's solid core
##      carry the river colour texture's colour, within a tolerance measured
##      from a control (the same pixels with the Rivers layer off against the
##      map texture), and meet the preset's intent bar -- both right after the
##      preset is applied and after a repaint the way half the shell does it
##      (`map_view.texture = bridge.color_texture()`, no overlay redraw).
##      Sections A-C read textures; only this one reads what a user sees,
##      which is why white rivers on screen (OUTSTANDING_WORK.md, "Rivers draw
##      WHITE at fit zoom", a regression from 2cf0143) passed A-C.
##
## Crops of the base texture around a trunk are saved per preset, on and off,
## magnified 4x nearest, for looking at.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _rivstyle_probe.tscn -- --out DIR
##
## Arguments -- every one is read below; an unknown one aborts:
##   --out DIR     where the PNGs and rivstyle.json land (required)
##   --seed N      world seed (default 483920)
##   --no-zoom     skip section C (the deep-zoom tile check)
##   --presets a,b only these presets (names as in PRESETS); default all
##   --grid WxH    world size (default 1024x656)
##   --timing-only only section D, the Rivers switch's cost
##   --screen-only only section S, the drawn river on screen
##
## Exit status: 0 every assertion held, 1 one failed, 2 could not run.

const PRESETS := ["Natural Vibrant", "Default", "Antique", "Ink", "Woodcut",
	"Blueprint", "Ink wash", "Vintage atlas", "Night"]
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]
var GRID := Vector2i(1024, 656)
const REPS := 5
const ZOOM := 16.0
## Section S's presets: the brief's four -- the default look (RV-2's blue), a
## white line, a dark pen line, and the one whose grade darkens everything.
const SCREEN_PRESETS := ["Default", "Blueprint", "Ink", "Night"]
## Section S's zooms, as multiples of the fit (`reset_view`) zoom: the opening
## view, and x1.4, the owner's reported range below the deep-zoom switch.
const SCREEN_ZOOMS := [1.0, 1.4]

var _out := ""
var _seed := 483920
var _zoom_check := true
var _only: PackedStringArray = []
var _timing_only := false
var _screen_only := false
var _fail := 0
var _app: Node
var _vh: Control
var _br: Node
var _report := {}


func _ok(cond: bool, what: String) -> void:
	print(("RIVSTYLE  ok    " if cond else "RIVSTYLE  FAIL  ") + what)
	if not cond:
		_fail += 1


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]; i += 1
			"--seed":
				_seed = int(args[i + 1]); i += 1
			"--no-zoom":
				_zoom_check = false
			"--presets":
				_only = args[i + 1].split(","); i += 1
			"--timing-only":
				_timing_only = true
			"--screen-only":
				_screen_only = true
			"--grid":
				var p := args[i + 1].split("x"); GRID = Vector2i(int(p[0]), int(p[1])); i += 1
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument %s" % args[i])
				return false
		i += 1
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(1800.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if not _br.world_gen.has_method("set_rivers_in_map"):
		printerr("PROBE-CANNOT-RUN: the loaded binary has no set_rivers_in_map -- stale .dll")
		get_tree().quit(2)
		return
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	_ok(_vh.layer_visible("rivers"), "the Rivers layer reads back on")

	if _timing_only:
		await _timing()
		_finish()
		return
	var samples := _pick_samples(_br.rivers(1))
	print("RIVSTYLE  %d sample cells (solid-core river pixels on dry land)" % samples.size())
	_ok(samples.size() >= 12, "enough sample cells to measure (%d >= 12)" % samples.size())
	if samples.size() < 12:
		_finish()
		return

	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	var rw := _find(_app, "render_workspace.gd")
	_ok(rw != null, "the RenderWorkspace is reachable")
	if rw == null:
		_finish()
		return
	var tiles: Array = rw.get("_preset_tiles")

	if _screen_only:
		await _screen_leg(tiles)
		_finish()
		return

	var results := {}
	for pname in PRESETS:
		if not _only.is_empty() and not _only.has(pname):
			continue
		var idx := -1
		for i in RenderWorkspace.STYLE_PRESETS.size():
			if String(RenderWorkspace.STYLE_PRESETS[i][0]) == pname:
				idx = i
		if idx < 0:
			_ok(false, "preset %s is in STYLE_PRESETS" % pname)
			continue
		(tiles[idx]["button"] as Button).emit_signal("pressed")
		await _frames(6)
		var on := _river_image()
		var off := _tex_image()
		_vh.set_layer_visible("rivers", false)
		await _frames(2)
		var off2 := _tex_image()
		_vh.set_layer_visible("rivers", true)
		await _frames(2)
		_ok(on != null and off != null, "%s: river colour texture and map texture readable" % pname)
		if on == null or off == null:
			continue
		_ok(off2 != null and off2.get_data() == off.get_data(), "%s: the map texture holds no river (identical with the layer off)" % pname)
		var m := _measure(on, off, samples)
		results[pname] = m
		print("RIVSTYLE  %-15s on median %s  off median %s  |on-off| median %.1f  luma on %.0f off %.0f  chroma on %.0f"
			% [pname, str(m["on"]), str(m["off"]), m["delta"], m["luma_on"], m["luma_off"], m["chroma_on"]])
		_ok(m["delta"] > 40.0, "%s: the river colour texture carries the rivers (|river-map| median %.1f > 40)" % [pname, m["delta"]])
		_save_crop(on, samples, "%s/%s_on.png" % [_out, pname.to_snake_case()])
		_save_crop(off, samples, "%s/%s_off.png" % [_out, pname.to_snake_case()])
		if _zoom_check and pname in ["Natural Vibrant", "Blueprint", "Woodcut", "Night"]:
			await _zoom_tile_check(pname, on, samples)
	_judge(results)
	## S. what the user sees -- after A-C, whose texture reads cannot see it.
	await _screen_leg(tiles)

	## D. the cost of the switch, back on the default preset.
	(tiles[0]["button"] as Button).emit_signal("pressed")
	await _frames(6)
	await _timing()
	## E. the cached network follows an edit (`WorldGen::river_geometry`'s
	## key): a sculpt far from tile 0 that moves the rivers must move the
	## rivers the map draws.
	await _edit_check(samples)
	_report["presets"] = results
	_finish()


## Section D: the Rivers switch's cost, `color_texture()` (map plus river
## colour texture), one fit-view `river_view_mesh`, and the colour field's
## memory -- each a median with min..max of REPS runs, printed and written to
## the JSON. Asserts nothing: these are measurements for the report.
func _timing() -> void:
	## The switch: engine flag + overlay redraw + tile rebuild request. The
	## base texture is not re-rendered any more (it holds no river).
	var t_off := PackedFloat64Array()
	var t_on := PackedFloat64Array()
	for r in REPS:
		var t0 := Time.get_ticks_usec()
		_vh.set_layer_visible("rivers", false)
		t_off.append((Time.get_ticks_usec() - t0) / 1000.0)
		await _frames(2)
		t0 = Time.get_ticks_usec()
		_vh.set_layer_visible("rivers", true)
		t_on.append((Time.get_ticks_usec() - t0) / 1000.0)
		await _frames(2)
	## `color_texture()` now also builds the river colour texture.
	var b := PackedFloat64Array()
	for r in REPS:
		var t0 := Time.get_ticks_usec()
		_br.color_texture()
		b.append((Time.get_ticks_usec() - t0) / 1000.0)
	## The base view's stroke, one fit-view mesh per redraw.
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov._displayed_rect()
	var g: Vector2i = _br.grid_size()
	var sc := Vector2(rect.size.x / float(g.x), rect.size.y / float(g.y))
	var mesh_ms := PackedFloat64Array()
	var nidx := 0
	for r in REPS:
		var t0 := Time.get_ticks_usec()
		var m: Dictionary = _br.river_view_mesh(sc, rect.position, rect)
		mesh_ms.append((Time.get_ticks_usec() - t0) / 1000.0)
		nidx = (m.get("indices", PackedInt32Array()) as PackedInt32Array).size()
	var st: Dictionary = _br.world_gen.river_field_stats()
	var t_gr := PackedFloat64Array()
	for r in REPS:
		var t0 := Time.get_ticks_usec()
		_br.rivers(1)
		t_gr.append((Time.get_ticks_usec() - t0) / 1000.0)
	_report["get_rivers_1_ms"] = _stats(t_gr)
	_report["toggle_off_ms"] = _stats(t_off)
	_report["toggle_on_ms"] = _stats(t_on)
	_report["color_texture_ms"] = _stats(b)
	_report["view_mesh_ms"] = _stats(mesh_ms)
	_report["field"] = st
	print("RIVSTYLE  get_rivers(1) ms (the refresh no longer calls it) %s" % str(_report["get_rivers_1_ms"]))
	print("RIVSTYLE  toggle OFF ms %s   toggle ON ms %s" % [str(_report["toggle_off_ms"]), str(_report["toggle_on_ms"])])
	print("RIVSTYLE  color_texture ms (map + river colour texture) %s" % str(_report["color_texture_ms"]))
	print("RIVSTYLE  river_view_mesh ms at fit %s, %d triangles" % [str(_report["view_mesh_ms"]), nidx / 3])
	print("RIVSTYLE  colour field: %d px covered of %d, %.1f MB in blocks" % [int(st.get("covered", 0)), g.x * g.y, float(st.get("allocated_bytes", 0)) / 1048576.0])


## Section E. Digs a trench across the map at a sample river cell far from
## grid tile 0 (the cache key must see a mark anywhere), commits, and checks
## that the drawn network follows the engine's: every cell the fresh
## `get_rivers(1)` newly routes a solid-core stroke through is river-coloured
## in the re-rendered texture, and every sample the network left is not.
func _edit_check(samples: Array) -> void:
	var before := {}
	for s in samples:
		before[s["cell"]] = true
	var spot: Vector2i = Vector2i(-1, -1)
	for s in samples:
		var c: Vector2i = s["cell"]
		if c.x > 200 and c.y > 200:
			spot = c
			break
	if spot.x < 0:
		_ok(false, "E: a sample river cell away from tile 0 to edit at")
		return
	var off0 := _tex_image()
	var riv0 := _river_image()
	var key0 := String(_br.world_gen.river_network_key())
	_ok(_br.sculpt_set_feature("freehand") and _br.sculpt_set_freehand_mode("raise"), "E: sculpt set to freehand raise")
	for oy in range(-8, 9, 2):
		for ox in range(-8, 9, 2):
			_br.sculpt_begin_stroke()
			_br.sculpt_add_point(spot.x + ox - 0.5, spot.y + oy)
			_br.sculpt_add_point(spot.x + ox, spot.y + oy)
			_br.sculpt_end_stroke()
	var summary: Dictionary = _br.sculpt_commit("rivstyle probe")
	await _frames(4)
	var key1 := String(_br.world_gen.river_network_key())
	print("RIVSTYLE  E: river network key %s -> %s" % [key0, key1])
	## The edit is >= 200 cells from the origin, so it touches none of the
	## 64-cell staleness tiles at 0: a key read at tile 0 alone would not move.
	_ok(key0 != key1, "E: an edit away from tile 0 moves the river network's cache key")
	_vh.map_view.texture = _br.color_texture()
	await _frames(2)
	var on := _river_image()
	var off := _tex_image()
	var fresh_rivers: Array = _br.rivers(1)
	var fresh := _pick_samples(fresh_rivers)
	## A cell the network LEFT: no drawn render point of the fresh network
	## within 3 cells of its centre (the widest stroke here is ~3 cells), so
	## the cell's pixel is beyond any stroke's fringe.
	var pts_near := func(c: Vector2i) -> bool:
		var cc := Vector2(c) + Vector2(0.5, 0.5)
		for r: Dictionary in fresh_rivers:
			if not r.has("widths") or r.has("parallel_of"):
				continue
			for q: Vector2 in (r["render_points"] as PackedVector2Array):
				if q.distance_to(cc) < 3.0:
					return true
		return false
	var gone := 0; var gone_clear := 0
	for c in before.keys():
		if (c as Vector2i).distance_to(spot) < 12.0 and not pts_near.call(c):
			gone += 1
			var a := _px(on, c); var b := _px(off, c)
			if absf(a.x - b.x) + absf(a.y - b.y) + absf(a.z - b.z) < 30.0:
				gone_clear += 1
	var drawn := 0; var drawn_ok := 0
	for s in fresh:
		var a := _px(on, s["cell"]); var b := _px(off, s["cell"])
		drawn += 1
		if absf(a.x - b.x) + absf(a.y - b.y) + absf(a.z - b.z) > 40.0:
			drawn_ok += 1
	var d0 := off0.get_data(); var d1 := off.get_data()
	print("RIVSTYLE  E: edit at %s, commit %s; fresh solid-core samples %d, river-coloured %d; samples near the edit the network left %d, clear of ink %d"
		% [str(spot), str(summary.get("still_stale", "?")), drawn, drawn_ok, gone, gone_clear])
	_ok(d0 != d1, "E: the edit changed the terrain the map draws (premise)")
	_ok(riv0.get_data() != on.get_data(), "E: the river colour texture was rebuilt")
	_ok(drawn >= 12 and drawn_ok == drawn, "E: every fresh river cell is river-coloured after the edit (%d of %d)" % [drawn_ok, drawn])
	if gone > 0:
		_ok(gone_clear == gone, "E: every river cell the edit removed is clear of river ink (%d of %d)" % [gone_clear, gone])
	else:
		print("RIVSTYLE  E: the edit removed no sample river near it -- the 'left' half is not exercised")


## **Section S: the drawn river, read off the SCREEN.** For each of
## SCREEN_PRESETS at each of SCREEN_ZOOMS (below the deep-zoom switch, where
## `map_overlay.gd::_draw_rivers` is what draws the rivers), twice: right after
## the preset is applied ("applied"), and after one more repaint the way
## `app.gd`, `tool_bar.gd`, `menus.gd`, `right_dock.gd` and
## `world_workspace.gd` all do it -- `map_view.texture = bridge.color_texture()`
## with no overlay redraw ("repainted"). Then a deep-zoom screenshot per preset,
## for looking at only.
##
## Why: sections A-C read the river colour TEXTURE, which was right while the
## screen drew the rivers pure white (OUTSTANDING_WORK.md, "Rivers draw WHITE
## at fit zoom", a regression from 2cf0143). Must never judge the texture
## here -- the point of this section is that it reads the viewport.
func _screen_leg(tiles: Array) -> void:
	for pname in SCREEN_PRESETS:
		if not _only.is_empty() and not _only.has(pname):
			continue
		var idx := -1
		for i in RenderWorkspace.STYLE_PRESETS.size():
			if String(RenderWorkspace.STYLE_PRESETS[i][0]) == pname:
				idx = i
		if idx < 0:
			_ok(false, "S: preset %s is in STYLE_PRESETS" % pname)
			continue
		(tiles[idx]["button"] as Button).emit_signal("pressed")
		await _frames(6)
		for z: float in SCREEN_ZOOMS:
			_vh.reset_view()
			await _frames(3)
			## `zoom_step` multiplies about the viewport centre; 1.0 is skipped
			## so the fit case is exactly `reset_view`'s own zoom.
			if z != 1.0:
				_vh.zoom_step(z)
			await _frames(4)
			_ok(not _vh.lod_active(), "S: %s x%.1f is below the deep-zoom switch (the overlay draws the rivers)" % [pname, z])
			await _screen_case(pname, z, "applied")
			_vh.map_view.texture = _br.color_texture()
			await _frames(4)
			await _screen_case(pname, z, "repainted")
		## Deep zoom, for looking at: the tiles draw the rivers there.
		_vh.reset_view()
		await _frames(3)
		_vh.zoom_step(ZOOM / _vh.zoom())
		await _settle_tiles()
		var deep := await _grab()
		if deep != null:
			deep.save_png("%s/screen_%s_z%d.png" % [_out, pname.to_snake_case(), int(ZOOM)])
	_vh.reset_view()
	await _frames(3)


## One case of section S. Takes the solid core of the stroke the overlay drew
## (`_solid_core_pixels`, off the same `river_view_mesh` call `_draw_rivers`
## makes), grabs the viewport with the Rivers layer on and then off, and judges
## the ON pixels:
##
##  - **against the river colour texture** at each pixel's cell: the stroke is
##    textured with it, so the drawn pixel must be that colour. The tolerance
##    is the CONTROL's 90th percentile: the same pixels with the layer off
##    against the map texture at the same cells -- the display path's own
##    error (filtering, resampling) measured on this run, not a number chosen.
##  - **against the preset's intent**, the same literal bars `_judge` holds the
##    texture to (Default: blue, not white; Blueprint: white; Ink: dark; Night:
##    luminous blue over dark ground), applied to the screen median.
##
## Saves the ON and OFF viewport crops for looking at.
func _screen_case(pname: String, z: float, when: String) -> void:
	var tag := "%s x%.1f %s" % [pname, z, when]
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var g: Vector2i = _br.grid_size()
	## The same arguments `_draw_rivers` passes (its crisp `k` is the camera
	## zoom, `_crisp_begin`).
	var k: float = maxf(float(ov.get("_camera_zoom")), 0.001)
	var vis: Rect2 = ov.get("_visible_local")
	var sc := Vector2(rect.size.x / float(g.x), rect.size.y / float(g.y)) * k
	var mesh: Dictionary = _br.river_view_mesh(sc, rect.position * k, Rect2(vis.position * k, vis.size * k))
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var on_img := await _grab_full()
	_vh.set_layer_visible("rivers", false)
	await _frames(3)
	var off_img := await _grab_full()
	_vh.set_layer_visible("rivers", true)
	await _frames(3)
	var riv := _river_image()
	var map := _tex_image()
	if on_img == null or off_img == null or riv == null or map == null:
		_ok(false, "S: %s: viewport and textures readable" % tag)
		return
	var host := Rect2i(Vector2i(_vh.global_position.round()), Vector2i(_vh.size.round())).intersection(Rect2i(Vector2i.ZERO, on_img.get_size()))
	var fname := "%s/screen_%s_x%s_%s" % [_out, pname.to_snake_case(), str(z).replace(".", "p"), when]
	on_img.get_region(host).save_png(fname + "_on.png")
	off_img.get_region(host).save_png(fname + "_off.png")
	var pix := _solid_core_pixels(mesh, xf, k, host.grow(-2))
	var inv := xf.affine_inverse()
	var d_river := []; var d_ctrl := []; var d_white := []
	var ch := [[], [], []]; var lum_on := []; var lum_off := []
	for p: Vector2i in pix:
		var local: Vector2 = inv * (Vector2(p) + Vector2(0.5, 0.5))
		var gp := (local - rect.position) / rect.size * Vector2(g)
		var c := Vector2i(clampi(int(floor(gp.x)), 0, g.x - 1), clampi(int(floor(gp.y)), 0, g.y - 1))
		var a := _px(on_img, p); var b := _px(off_img, p)
		var r := _px(riv, c); var m := _px(map, c)
		d_river.append((absf(a.x - r.x) + absf(a.y - r.y) + absf(a.z - r.z)) / 3.0)
		d_ctrl.append((absf(b.x - m.x) + absf(b.y - m.y) + absf(b.z - m.z)) / 3.0)
		d_white.append((765.0 - a.x - a.y - a.z) / 3.0)
		for q in 3:
			ch[q].append(a[q])
		lum_on.append(_luma(a)); lum_off.append(_luma(b))
	_ok(pix.size() >= 200, "S: %s: enough solid-core stroke pixels on screen (%d >= 200)" % [tag, pix.size()])
	if pix.size() < 200:
		return
	var tol := _pct(d_ctrl, 0.9)
	var med := _median(d_river)
	var on := Vector3(_median(ch[0]), _median(ch[1]), _median(ch[2]))
	var luma := _luma(on)
	print("RIVSTYLE  S %-26s px %5d  screen median %s luma %.0f  |screen-rivtex| median %.1f  tol (control p90) %.1f  |screen-white| median %.1f  luma over ground %.0f"
		% [tag, pix.size(), str(on), luma, med, tol, _median(d_white), _median(lum_on) - _median(lum_off)])
	_ok(med <= tol, "S: %s: the drawn river is the river colour texture's colour (median %.1f <= control p90 %.1f)" % [tag, med, tol])
	match pname:
		"Default":
			## Labelled judgement: RV-2's palette is blue (b well over r);
			## white has b - r = 0, which is the regression.
			_ok(on.z - on.x >= 30.0, "S: %s: blue, not white (b-r %.0f >= 30)" % [tag, on.z - on.x])
		"Blueprint":
			var mn: float = minf(on.x, minf(on.y, on.z))
			_ok(luma >= 200.0 and mn >= 185.0, "S: %s: white line (luma %.0f >= 200, min channel %.0f >= 185)" % [tag, luma, mn])
		"Ink":
			_ok(luma <= 90.0, "S: %s: dark pen line (luma %.0f <= 90)" % [tag, luma])
		"Night":
			var lg: float = _median(lum_on) - _median(lum_off)
			_ok(on.z - on.x >= 40.0 and on.z - on.y >= 12.0 and lg >= 45.0,
				"S: %s: luminous blue over dark ground (b-r %.0f >= 40, b-g %.0f >= 12, luma over ground %.0f >= 45)"
				% [tag, on.z - on.x, on.z - on.y, lg])


## The viewport pixels the stroke's SOLID core covers: every triangle of
## `river_view_mesh` whose three vertices are fully opaque (not the 1 px
## fringe, not an order-1 run's de-emphasised alpha), taken to viewport pixels
## (`xf * (point / k)`, the inverse of `_draw_rivers`' crisp scale), and every
## pixel whose CENTRE falls inside it -- the rasterizer's own coverage rule,
## so these pixels are drawn with the stroke's colour and nothing blended.
## Inside `clip` only. At most ~4000, evenly thinned.
func _solid_core_pixels(mesh: Dictionary, xf: Transform2D, k: float, clip: Rect2i) -> Array:
	var pts: PackedVector2Array = mesh.get("points", PackedVector2Array())
	var cols: PackedColorArray = mesh.get("colors", PackedColorArray())
	var idx: PackedInt32Array = mesh.get("indices", PackedInt32Array())
	var seen := {}
	for t in range(0, idx.size() - 2, 3):
		var i0 := idx[t]; var i1 := idx[t + 1]; var i2 := idx[t + 2]
		if cols[i0].a < 0.999 or cols[i1].a < 0.999 or cols[i2].a < 0.999:
			continue
		var a: Vector2 = xf * (pts[i0] / k)
		var b: Vector2 = xf * (pts[i1] / k)
		var c: Vector2 = xf * (pts[i2] / k)
		var area := (b - a).cross(c - a)
		if absf(area) < 1e-6:
			continue
		var x0 := maxi(clip.position.x, int(floor(minf(a.x, minf(b.x, c.x)))))
		var x1 := mini(clip.end.x - 1, int(ceil(maxf(a.x, maxf(b.x, c.x)))))
		var y0 := maxi(clip.position.y, int(floor(minf(a.y, minf(b.y, c.y)))))
		var y1 := mini(clip.end.y - 1, int(ceil(maxf(a.y, maxf(b.y, c.y)))))
		for y in range(y0, y1 + 1):
			for x in range(x0, x1 + 1):
				var p := Vector2(x + 0.5, y + 0.5)
				var w0 := (b - a).cross(p - a) * signf(area)
				var w1 := (c - b).cross(p - b) * signf(area)
				var w2 := (a - c).cross(p - c) * signf(area)
				if w0 >= 0.0 and w1 >= 0.0 and w2 >= 0.0:
					seen[Vector2i(x, y)] = true
	var all: Array = seen.keys()
	var step := maxi(1, all.size() / 4000)
	var out: Array = []
	for i in range(0, all.size(), step):
		out.append(all[i])
	return out


## The whole viewport as RGB8, after the frame has drawn.
func _grab_full() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	img.convert(Image.FORMAT_RGB8)
	return img


## The `q` quantile (nearest rank) of `a`; NAN when empty.
func _pct(a: Array, q: float) -> float:
	if a.is_empty():
		return NAN
	var b := a.duplicate(); b.sort()
	return float(b[clampi(int(q * float(b.size() - 1)), 0, b.size() - 1)])


func _finish() -> void:
	var f := FileAccess.open(_out.path_join("rivstyle.json"), FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(_report, "  "))
	print("RIVSTYLE  === %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## `WorldGen::river_color_texture` as RGB8 -- the styled river at full
## coverage, which the base view's stroke is textured with.
func _river_image() -> Image:
	var t: Texture2D = _br.river_color_texture()
	if t == null:
		return null
	var img := t.get_image()
	if img == null:
		return null
	img = img.duplicate()
	img.convert(Image.FORMAT_RGB8)
	return img


func _tex_image() -> Image:
	var t: Texture2D = _vh.map_view.texture
	if t == null:
		return null
	var img := t.get_image()
	if img == null:
		return null
	img = img.duplicate()
	img.convert(Image.FORMAT_RGB8)
	return img


## Cells whose texture pixel is inside a river's solid core: a render point
## within 0.3 cells of the cell's centre on a run at least 2.4 cells wide
## there (solid to 1.2 cells from the centreline, `river_stroke::stroke_mesh`),
## the cell and its eight neighbours dry land, away from the plate frame.
func _pick_samples(rivers: Array) -> Array:
	var out: Array = []
	var seen := {}
	for r: Dictionary in rivers:
		if not r.has("widths") or r.has("parallel_of"):
			continue
		var rp: PackedVector2Array = r["render_points"]
		var w: PackedFloat32Array = r["widths"]
		var cols: PackedColorArray = r["colors"]
		for i in range(0, rp.size(), 3):
			if w[i] < 2.4:
				continue
			var c := Vector2i(int(floor(rp[i].x)), int(floor(rp[i].y)))
			if rp[i].distance_to(Vector2(c) + Vector2(0.5, 0.5)) > 0.3:
				continue
			if c.x < 60 or c.y < 60 or c.x >= GRID.x - 60 or c.y >= GRID.y - 60 or seen.has(c):
				continue
			var dry := true
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					if String(_br.sample_cell(c.x + dx, c.y + dy).get("water", "")) != "land":
						dry = false
			if not dry:
				continue
			seen[c] = true
			out.append({"cell": c, "p": rp[i], "rv2": cols[i]})
			if out.size() >= 80:
				return out
	return out


func _px(img: Image, c: Vector2i) -> Vector3:
	var k := img.get_pixel(c.x, c.y)
	return Vector3(k.r8, k.g8, k.b8)


func _luma(v: Vector3) -> float:
	return 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z


func _chroma(v: Vector3) -> float:
	return maxf(v.x, maxf(v.y, v.z)) - minf(v.x, minf(v.y, v.z))


func _median(a: Array) -> float:
	if a.is_empty():
		return NAN
	var b := a.duplicate(); b.sort()
	return float(b[b.size() / 2])


func _measure(on: Image, off: Image, samples: Array) -> Dictionary:
	var ch := [[], [], []]
	var ch_off := [[], [], []]
	var d := []; var lon := []; var loff := []; var cron := []; var rv2d := []; var rv2d_off := []
	for s in samples:
		var a := _px(on, s["cell"])
		var b := _px(off, s["cell"])
		var rv: Color = s["rv2"]
		var rv3 := Vector3(rv.r8, rv.g8, rv.b8)
		for k in 3:
			ch[k].append(a[k]); ch_off[k].append(b[k])
		d.append(absf(a.x - b.x) + absf(a.y - b.y) + absf(a.z - b.z))
		lon.append(_luma(a)); loff.append(_luma(b)); cron.append(_chroma(a))
		rv2d.append((absf(a.x - rv3.x) + absf(a.y - rv3.y) + absf(a.z - rv3.z)) / 3.0)
		rv2d_off.append((absf(b.x - rv3.x) + absf(b.y - rv3.y) + absf(b.z - rv3.z)) / 3.0)
	return {
		"on": [_median(ch[0]), _median(ch[1]), _median(ch[2])],
		"off": [_median(ch_off[0]), _median(ch_off[1]), _median(ch_off[2])],
		"delta": _median(d), "luma_on": _median(lon), "luma_off": _median(loff),
		"chroma_on": _median(cron),
		"rv2_dist_on": _median(rv2d), "rv2_dist_off": _median(rv2d_off),
	}


## Section B: each preset's river against a bar written from its intent.
## Literal thresholds on the measured pixel, never the preset's own ink
## constant read back (MISTAKES.md: "never assert a constant against itself").
## Every threshold below is a labelled judgement of what the style's own
## research asks for (`MAP_STYLE_RESEARCH.md` §2.x): "white" as luma >= 200,
## "black" as luma <= 70, "muted" as 15 chroma below the default's, and so on;
## each was written before the run that first measured it, except Night's
## (see its note).
func _judge(r: Dictionary) -> void:
	if r.has("Natural Vibrant"):
		var m: Dictionary = r["Natural Vibrant"]
		## The default look is RV-2's: the pixel is the run's own palette
		## colour, moved only by the sheet (paper tint, vignette, grade).
		_ok(m["rv2_dist_on"] < 30.0, "Natural Vibrant: river pixel within 30 levels/channel of RV-2's palette colour (%.1f)" % m["rv2_dist_on"])
		_ok(m["rv2_dist_off"] > m["rv2_dist_on"] + 25.0, "Natural Vibrant: ...and the ground is not (%.1f)" % m["rv2_dist_off"])
	if r.has("Default"):
		var m: Dictionary = r["Default"]
		_ok(m["rv2_dist_on"] < 30.0, "Default: river pixel within 30 levels/channel of RV-2's palette colour (%.1f)" % m["rv2_dist_on"])
	if r.has("Blueprint"):
		var m: Dictionary = r["Blueprint"]
		var mn: float = minf(m["on"][0], minf(m["on"][1], m["on"][2]))
		_ok(m["luma_on"] >= 200.0 and mn >= 185.0, "Blueprint: white line (luma %.0f >= 200, min channel %.0f >= 185)" % [m["luma_on"], mn])
	if r.has("Woodcut"):
		var m: Dictionary = r["Woodcut"]
		_ok(m["luma_on"] <= 70.0, "Woodcut: black keyline (luma %.0f <= 70)" % m["luma_on"])
	if r.has("Ink"):
		var m: Dictionary = r["Ink"]
		_ok(m["luma_on"] <= 90.0, "Ink: dark pen line (luma %.0f <= 90)" % m["luma_on"])
	if r.has("Ink wash"):
		var m: Dictionary = r["Ink wash"]
		_ok(m["luma_on"] <= 100.0 and m["chroma_on"] <= 40.0, "Ink wash: black ink, no hue (luma %.0f <= 100, chroma %.0f <= 40)" % [m["luma_on"], m["chroma_on"]])
	if r.has("Night"):
		## Blue by hue, not by an absolute channel: Night's grade (exposure
		## -0.35, gamma -0.2) takes a full 255 blue down to ~131 on this world
		## (measured, run 1: ink (150, 215, 255) read back (90, 123, 131)), so
		## the first version of this bar -- b >= 140 -- could not be met by any
		## ink that goes through the grade, which is the point of the change.
		## What "a luminous blue" can mean under the grade: blue the dominant
		## channel by a clear margin over both others, and far brighter than
		## the dark ground around it.
		var m: Dictionary = r["Night"]
		var b: float = m["on"][2]
		_ok(b - m["on"][0] >= 40.0 and b - m["on"][1] >= 12.0 and m["luma_on"] - m["luma_off"] >= 45.0,
			"Night: luminous blue that stands out (b-r %.0f >= 40, b-g %.0f >= 12, luma over ground %.0f >= 45)"
			% [b - m["on"][0], b - m["on"][1], m["luma_on"] - m["luma_off"]])
	if r.has("Vintage atlas") and r.has("Natural Vibrant"):
		var m: Dictionary = r["Vintage atlas"]
		var nv: Dictionary = r["Natural Vibrant"]
		_ok(m["chroma_on"] <= nv["chroma_on"] - 15.0, "Vintage atlas: muted (chroma %.0f <= Natural Vibrant's %.0f - 15)" % [m["chroma_on"], nv["chroma_on"]])
	if r.has("Antique") and r.has("Natural Vibrant"):
		## Sepia (a Painter style) and the parchment reach the river: the
		## pixel is warmer than RV-2's cool blue -- red closer to blue.
		var m: Dictionary = r["Antique"]
		var nv: Dictionary = r["Natural Vibrant"]
		_ok(m["on"][2] - m["on"][0] <= nv["on"][2] - nv["on"][0] - 20.0,
			"Antique: sepia/parchment reach the river (b-r %.0f <= Natural Vibrant's %.0f - 20)" % [m["on"][2] - m["on"][0], nv["on"][2] - nv["on"][0]])


## Section C: the deep-zoom tile at a sample cell carries the base texture's
## river colour, and loses it with the layer off.
func _zoom_tile_check(pname: String, river_img: Image, samples: Array) -> void:
	var s: Dictionary = samples[samples.size() / 2]
	_vh.reset_view()
	await _frames(3)
	_vh.zoom_step(ZOOM / _vh.zoom())
	_vh.move_view_to(s["p"].x, s["p"].y)
	await _settle_tiles()
	var on := await _grab_centre()
	var shot_on := await _grab()
	_vh.set_layer_visible("rivers", false)
	await _settle_tiles()
	var off := await _grab_centre()
	var shot_off := await _grab()
	_vh.set_layer_visible("rivers", true)
	await _settle_tiles()
	if shot_on != null:
		shot_on.save_png("%s/%s_z%d_on.png" % [_out, pname.to_snake_case(), int(ZOOM)])
	if shot_off != null:
		shot_off.save_png("%s/%s_z%d_off.png" % [_out, pname.to_snake_case(), int(ZOOM)])
	## The river colour texture at THIS sample's cell -- not the preset's
	## median over all 80 samples, which a palette that varies by order
	## (Natural Vibrant's) does not share with any one cell.
	var base := _px(river_img, s["cell"])
	var dt := (absf(on.x - base.x) + absf(on.y - base.y) + absf(on.z - base.z)) / 3.0
	var doff := (absf(on.x - off.x) + absf(on.y - off.y) + absf(on.z - off.z)) / 3.0
	print("RIVSTYLE  %-15s z%d tile centre on %s off %s  base %s  |tile-base| %.1f  |on-off| %.1f  lod=%s"
		% [pname, int(ZOOM), str(on), str(off), str(base), dt, doff, str(_vh.lod_active())])
	_ok(_vh.lod_active(), "%s: the deep-zoom tiles are up at z%d" % [pname, int(ZOOM)])
	_ok(doff > 25.0, "%s: the tile at the river cell changes with the Rivers layer (%.1f > 25)" % [pname, doff])
	_ok(dt < 30.0, "%s: the tile's river is the base texture's river colour (%.1f < 30 levels/channel)" % [pname, dt])
	_vh.reset_view()
	await _frames(3)


## Until the pyramid has caught up with the camera (`lod_pending() == 0`, a
## condition, not a frame count) and the per-tile morph has finished, capped
## at 20 s.
func _settle_tiles() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	var gp := _vh.global_position
	var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var sub := img.get_region(r)
	sub.convert(Image.FORMAT_RGB8)
	return sub


## Median colour of the 5x5 block at the viewport's centre, where
## `move_view_to` put the sample point.
func _grab_centre() -> Vector3:
	var img := await _grab()
	if img == null:
		return Vector3(NAN, NAN, NAN)
	var c := Vector2i(img.get_width() / 2, img.get_height() / 2)
	var ch := [[], [], []]
	for dy in range(-2, 3):
		for dx in range(-2, 3):
			var v := _px(img, c + Vector2i(dx, dy))
			for k in 3:
				ch[k].append(v[k])
	return Vector3(_median(ch[0]), _median(ch[1]), _median(ch[2]))


func _save_crop(img: Image, samples: Array, path: String) -> void:
	var c: Vector2i = samples[samples.size() / 2]["cell"]
	var r := Rect2i(c - Vector2i(48, 32), Vector2i(96, 64)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	var sub := img.get_region(r)
	sub.resize(sub.get_width() * 4, sub.get_height() * 4, Image.INTERPOLATE_NEAREST)
	sub.save_png(path)


func _stats(v: PackedFloat64Array) -> Dictionary:
	var a := Array(v); a.sort()
	return {"median": a[a.size() / 2], "min": a[0], "max": a[a.size() - 1], "n": a.size()}
