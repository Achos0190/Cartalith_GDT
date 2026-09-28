extends Node
## `OUTSTANDING_WORK.md` "Four map-data defects found while fixing lake labels
## and POI clicks", items 1-3, and owner Ruling BO for item 2 ("draw forced
## lakes"), driven through the real shell and engine:
##
##  S  (item 1) Sample's `water` is the water the map draws. A pit is dug in
##     dry high ground and committed, which makes a lake the map draws; the
##     civ layer's copy (taken at generation, not refreshed by a sculpt) still
##     calls it land. Sample must say "lake" there. Fails on HEAD, which read
##     that copy.
##  F  (item 2 + Ruling BO) A Lake stamp in arid ground (the rain gate keeps
##     its bowl dry, so the map draws land), then the shell's own "Count
##     painted lakes as water" handler (`world_workspace.gd::_on_force_lake`):
##     the drawn-water mask, Sample, the screen pixels at fit zoom and at z16,
##     the river network's cache key and a lake label must all show the lake;
##     and it must survive a save and a reopen. HEAD forced only the civ copy,
##     so the map never drew it.
##  B  (the "Map-data residuals" row's `sdf_biomes` item, inside leg F) With
##     `sdf_biomes` on, the land ring round the forced lake changes colour
##     (the biome band); before the press, the same ring does not. Against a
##     build whose band ignores the mask the ring moves 0.00 after the press
##     too, and B2 fails.
##  L  (item 3) Generated labels at z16: each settlement's and region's label
##     anchor, projected by the overlay's own `_point_to_screen` (what
##     `_draw_labels` uses), against the settlement marker's own
##     `_cell_to_screen`. HEAD anchored every class at the cell index, half a
##     cell up and left -- at z16 that is many pixels.
##
## Pixel legs compare against a colour measured in the same capture (a
## natural lake's interior, and the same spot before the press), never a
## palette literal, and every capture is saved for looking at.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _mapdata_probe.tscn -- --out DIR
##
## Arguments -- every one is read below; an unknown one aborts:
##   --out DIR     where the PNGs and the saved project land (required)
##   --seed N      world seed (default 483920)
##   --grid WxH    world size (default 512x384)
##
## Exit status: 0 every check held, 1 one failed, 2 could not run.

## Overlays hidden for the pixel legs, so a pixel is the map's own.
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict", "rivers", "labels"]
## The deep zoom the tiles draw at (the owner's z16).
const DEEP := 16.0
## `cartalith_civ::LAKE_RAIN` is 0.22: ground drier than this never pools
## into a lake by itself, so a Lake stamp there is land until forced. 0.18
## leaves a margin for the rainfall's own variation across the bowl.
const ARID := 0.18
## `_water_move` above this (summed |dR|+|dG|+|dB|, noise floor already
## subtracted) means the pixel is drawn as water. Labelled judgement, the
## same 8-level floor `_shorestep_probe.gd` uses for "the water changed",
## doubled; F7 and F8 print the natural lake's and the land's own moves so the
## margin is visible in every run.
const WATER_MOVES := 16

var GRID := Vector2i(512, 384)
var _seed := 483920
var _out := ""
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node


func _ok(cond: bool, what: String, detail: String = "") -> void:
	_checks += 1
	print(("MAPDATA  ok    " if cond else "MAPDATA  FAIL  ") + what + (("  -- " + detail) if detail != "" else ""))
	if not cond:
		_fail += 1


func _p(s: String) -> void:
	print("MAPDATA  " + s)


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


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


## The shell's own repaint after an engine edit (`_on_sculpt_commit`'s two
## camera-preserving lines).
func _repaint() -> void:
	_vh.map_view.texture = _br.color_texture()
	_vh.invalidate_lod_tiles()
	await _settle()


## The screen pixel of ground point `p` (the point frame: cell `i` spans
## `[i, i + 1)`), through the overlay's own fit rect and canvas transform.
func _screen_of(p: Vector2) -> Vector2:
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var local := rect.position + Vector2(p.x / float(GRID.x), p.y / float(GRID.y)) * rect.size
	return ov.get_global_transform_with_canvas() * local


func _px(img: Image, p: Vector2) -> Color:
	var s := _screen_of(p)
	return img.get_pixel(clampi(int(s.x), 0, img.get_width() - 1), clampi(int(s.y), 0, img.get_height() - 1))


func _dist(a: Color, b: Color) -> int:
	return absi(a.r8 - b.r8) + absi(a.g8 - b.g8) + absi(a.b8 - b.b8)


func _crop(img: Image, p: Vector2, name: String, half: int = 160) -> void:
	var s := Vector2i(_screen_of(p))
	var r := Rect2i(s - Vector2i(half, half), Vector2i(2 * half, 2 * half)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	img.get_region(r).save_png("%s/%s.png" % [_out, name])


## The drawn-water mask (`river_water_mask`: 255 where `build_color_texture`
## drew water) as an L8 image, refreshed by the last repaint.
func _mask() -> Image:
	var t: Texture2D = _br.river_water_mask()
	if t == null:
		return null
	var m := t.get_image().duplicate()
	m.convert(Image.FORMAT_L8)
	return m


func _drawn_water(m: Image, c: Vector2i) -> bool:
	return m != null and m.get_pixel(c.x, c.y).r8 > 127


func _view(p: Vector2, deep: bool) -> void:
	_vh.reset_view()
	await _frames(3)
	if deep:
		_vh.zoom_step(DEEP / _vh.zoom())
		_vh.move_view_to(p.x - 0.5, p.y - 0.5)
	await _settle()


## How far the pixel at ground point `p` moves when only the WATER's colour
## changes, less the noise floor -- `_shorestep_probe.gd`'s `_waterness`,
## at one point. `sea_ramp_strength` (the bathymetric ramp) is mixed into
## every sea and lake colour and nothing else (`render.rs::sea_color_core`),
## so a pixel drawn as water moves and a land pixel does not, whatever the
## palette. Colour-matching one capture against a lake reference was tried
## first and measured the lake's own depth shading instead (a deep natural
## lake is darker than a shallow one). Uses the current view; restores the
## appearance.
func _water_move(p: Vector2) -> int:
	var a := await _grab()
	var sp := _screen_of(p)
	## Off screen is "cannot measure", which must never read as land.
	if sp.x < 0 or sp.y < 0 or sp.x >= a.get_width() or sp.y >= a.get_height():
		_p("water-move: %s is off screen at %s" % [p, sp])
		return -1000
	_br.set_appearance({"sea_ramp_strength": 1.0})
	await _repaint()
	var b := await _grab()
	_br.drop_appearance_overrides(PackedStringArray(["sea_ramp_strength"]))
	await _repaint()
	var c := await _grab()
	var ctl := _dist(_px(a, p), _px(c, p))
	return _dist(_px(a, p), _px(b, p)) - ctl


## The current view captured with the `sdf_biomes` leg off and then at full
## strength (`[off, on]`), restoring the appearance -- `_water_move`'s own
## override/drop pattern. The difference between the two is the biome band
## alone: `sdf_biomes` only widens `land_color`'s ecotone noise near a biome
## boundary (`render.rs::sdf_eco_k`), so a pixel far from any boundary is
## identical in both.
func _sdf_pair() -> Array:
	var off := await _grab()
	_br.set_appearance({"sdf_biomes": 1.0})
	await _repaint()
	var on := await _grab()
	_br.drop_appearance_overrides(PackedStringArray(["sdf_biomes"]))
	await _repaint()
	return [off, on]


## Mean colour move (summed |dR|+|dG|+|dB|) between an `_sdf_pair` at the
## cell centres in `cells`.
func _sdf_move(pair: Array, cells: Array) -> float:
	if cells.is_empty():
		return 0.0
	var s := 0
	for c: Vector2i in cells:
		var p := Vector2(c) + Vector2(0.5, 0.5)
		s += _dist(_px(pair[0], p), _px(pair[1], p))
	return float(s) / cells.size()


## Dry land for `r` cells all round `c` (every sample land, above `min_m`),
## and optionally arid.
func _dry(c: Vector2i, r: int, min_m: float, arid: bool) -> bool:
	for oy in range(-r, r + 1, 4):
		for ox in range(-r, r + 1, 4):
			var s: Dictionary = _br.sample_cell(c.x + ox, c.y + oy)
			if String(s.get("water", "")) != "land" or float(s.get("elevation_m", 0.0)) < min_m:
				return false
			if arid and float(s.get("precipitation", 1.0)) >= ARID:
				return false
	return true


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
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
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	await _repaint()
	var m := _mask()
	if m == null or m.get_size() != GRID:
		printerr("PROBE-CANNOT-RUN: no drawn-water mask of the grid's size -- stale .dll?")
		get_tree().quit(2)
		return
	await _leg_labels()
	await _leg_sample()
	## H and R run BEFORE `_leg_forced()`: that leg's own `load_save()` (a
	## flat/tree `load_save`, not `project_open`) drops `labels`/`infra`/`civ`
	## without regenerating them (`lib.rs::load_save`'s own doc comment on
	## each field it clears -- restoring them is `project_open`'s job, which
	## this probe's `load_save` calls do not go through), so `label_create`/
	## `way_begin`/`way_commit` would refuse on the world `_leg_forced` leaves
	## behind. Running here keeps `self.source` a fresh `Generated` world.
	await _leg_hitbox()
	await _leg_routing()
	await _leg_forced()
	_finish()


# ---- L: generated label anchors against their features at z16 --------------

func _leg_labels() -> void:
	_vh.set_layer_visible("labels", true)
	_vh.set_layer_visible("settlements", true)
	var r: Dictionary = _br.labels_generate({"cull": {"on": false}})
	_p("labels_generate: ok=%s total=%s" % [r.get("ok"), r.get("total")])
	var by_name := {}
	for s: Dictionary in _br.settlements():
		by_name[String(s.get("name", ""))] = s
	var rows: Array = []
	for lb: Dictionary in _br.labels_render_list():
		if not bool(lb.get("generated", false)):
			continue
		var cls := String(lb.get("class", ""))
		if cls != "settlement":
			continue
		var s: Dictionary = by_name.get(String(lb.get("text", "")), {})
		if s.is_empty():
			continue
		rows.append([lb, s])
		if rows.size() >= 6:
			break
	_ok(rows.size() >= 3, "L0 generated settlement labels matched to their settlements (%d)" % rows.size())
	var worst := 0.0
	var ppc := 0.0
	for i in rows.size():
		var lb: Dictionary = rows[i][0]
		var s: Dictionary = rows[i][1]
		var cell := Vector2(float(s["x"]), float(s["y"]))
		await _view(cell + Vector2(0.5, 0.5), true)
		var ov: Control = _vh.overlay
		var rect: Rect2 = ov.displayed_rect()
		var xf: Transform2D = ov.get_global_transform_with_canvas()
		## Exactly the two projections the overlay draws with.
		var label_px: Vector2 = xf * ov._point_to_screen(Vector2(float(lb["x"]), float(lb["y"])), rect)
		var marker_px: Vector2 = xf * ov._cell_to_screen(cell, rect)
		ppc = (xf * ov._point_to_screen(Vector2(1, 0), rect) - xf * ov._point_to_screen(Vector2(0, 0), rect)).x
		var d := label_px.distance_to(marker_px)
		worst = maxf(worst, d)
		_p("L %s cell %s: label anchor (%.2f, %.2f) at %s, marker at %s: %.2f px apart (%.1f px/cell)" % [
			lb["text"], cell, float(lb["x"]), float(lb["y"]), label_px.round(), marker_px.round(), d, ppc])
		if i == 0:
			var img := await _grab()
			_crop(img, cell + Vector2(0.5, 0.5), "L_settlement_label_z16", 200)
	_ok(ppc >= 10.0, "L1 premise: z%d magnifies a cell to >= 10 px (%.1f)" % [int(DEEP), ppc])
	_ok(rows.size() > 0 and worst < 0.5,
		"L2 every settlement label's anchor sits on its settlement's marker at z%d (worst %.2f px)" % [int(DEEP), worst])
	## Every region label names a capital and sits on it too.
	var regions := 0
	var region_worst := 0.0
	for lb: Dictionary in _br.labels_render_list():
		if bool(lb.get("generated", false)) and String(lb.get("class", "")) == "region":
			var best := INF
			for s: Dictionary in _br.settlements():
				best = minf(best, Vector2(float(lb["x"]), float(lb["y"])).distance_to(Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5)))
			region_worst = maxf(region_worst, best)
			regions += 1
	_ok(regions == 0 or region_worst < 1e-6,
		"L3 every region label is at a settlement's cell centre (%d labels, worst %.3f cells)" % [regions, region_worst])
	_vh.set_layer_visible("labels", false)
	_vh.set_layer_visible("settlements", false)
	_br.world_gen.labels_clear_generated()
	_vh.reset_view()
	await _settle()


# ---- S: Sample reads the water the map draws after a sculpt -----------------

func _leg_sample() -> void:
	var spot := Vector2i(-1, -1)
	for gy in range(60, GRID.y - 60, 16):
		for gx in range(60, GRID.x - 60, 16):
			if _dry(Vector2i(gx, gy), 40, 600.0, false):
				spot = Vector2i(gx, gy)
				break
		if spot.x >= 0:
			break
	_ok(spot.x >= 0, "S0 found dry high ground to dig in")
	if spot.x < 0:
		return
	## `_lakelabel_probe.gd`'s dig: a lattice of freehand Lower strokes, which
	## that probe measured makes a lake the map draws.
	_br.sculpt_set_feature("freehand")
	_br.sculpt_set_freehand_mode("lower")
	for oy in range(-6, 7, 3):
		for ox in range(-6, 7, 3):
			_br.sculpt_begin_stroke()
			_br.sculpt_add_point(spot.x + ox - 0.5, spot.y + oy)
			_br.sculpt_add_point(spot.x + ox, spot.y + oy)
			_br.sculpt_end_stroke()
	_br.sculpt_commit("mapdata probe dig")
	await _repaint()
	var m := _mask()
	## The dig's own lake cell: the drawn cell nearest the centre.
	var lake := Vector2i(-1, -1)
	for r in range(0, 8):
		for oy in range(-r, r + 1):
			for ox in range(-r, r + 1):
				if lake.x < 0 and _drawn_water(m, spot + Vector2i(ox, oy)):
					lake = spot + Vector2i(ox, oy)
	_ok(lake.x >= 0, "S1 premise: the dig made water the map draws near %s" % spot)
	if lake.x < 0:
		return
	var w := String(_br.sample_cell(lake.x, lake.y).get("water", "(absent)"))
	_ok(w == "lake", "S2 Sample calls the drawn lake at %s a lake" % lake, "water=%s" % w)
	## And the other way round: the nearest drawn land cell reads land.
	var land := Vector2i(-1, -1)
	for r in range(1, 60):
		for d in [Vector2i(r, 0), Vector2i(-r, 0), Vector2i(0, r), Vector2i(0, -r)]:
			var q: Vector2i = lake + d
			if land.x < 0 and q.x >= 0 and q.y >= 0 and q.x < GRID.x and q.y < GRID.y and not _drawn_water(m, q):
				land = q
	var wl := String(_br.sample_cell(land.x, land.y).get("water", "(absent)"))
	_ok(land.x >= 0 and wl == "land", "S3 Sample calls the drawn land at %s land" % land, "water=%s" % wl)


# ---- F: a forced lake is drawn, sampled, labelled and saved ------------------

func _leg_forced() -> void:
	var m0 := _mask()
	## A natural lake's interior: the reference colour for "lake" in the same
	## captures.
	var nat := Vector2i(-1, -1)
	for gy in range(8, GRID.y - 8, 2):
		for gx in range(8, GRID.x - 8, 2):
			if nat.x >= 0:
				break
			var all := true
			for oy in range(-3, 4):
				for ox in range(-3, 4):
					if not _drawn_water(m0, Vector2i(gx + ox, gy + oy)):
						all = false
			if all and String(_br.sample_cell(gx, gy).get("water", "")) == "lake":
				nat = Vector2i(gx, gy)
	_ok(nat.x >= 0, "F0 premise: a natural lake to take the lake colour from (%s)" % nat)
	var spot := Vector2i(-1, -1)
	for gy in range(40, GRID.y - 40, 8):
		for gx in range(40, GRID.x - 40, 8):
			if spot.x < 0 and _dry(Vector2i(gx, gy), 24, 300.0, true):
				spot = Vector2i(gx, gy)
	_ok(spot.x >= 0, "F1 premise: arid high ground for a Lake stamp (precipitation < %.2f)" % ARID)
	if spot.x < 0 or nat.x < 0:
		return
	var at := Vector2(spot) + Vector2(0.5, 0.5)
	_br.sculpt_set_feature("lake")
	_br.sculpt_set_globals({"brush_size": 9.0})
	_br.sculpt_begin_stroke()
	_br.sculpt_add_point(at.x, at.y)
	_br.sculpt_end_stroke()
	var c: Dictionary = _br.sculpt_commit("mapdata probe lake stamp")
	_p("lake stamp at %s: %s" % [spot, c])
	await _repaint()
	var m1 := _mask()
	_ok(not _drawn_water(m1, spot), "F2 premise: the stamped bowl is arid, so the map draws land there before the press")
	var key0 := String(_br.world_gen.river_network_key()) if _br.world_gen.has_method("river_network_key") else ""
	## Before: fit and z16 -- the spot's colour, and whether it moves with
	## the water's colour (it must not: it is land).
	await _view(at, false)
	var fit0 := await _grab()
	var mv_fit0 := await _water_move(at)
	## Leg B (the `sdf_biomes` residual): the band before the press, at fit
	## zoom, where the base raster -- the leg `with_map_scale_forced` feeds --
	## is what is drawn (the LOD tile builds its own lake-blind band,
	## `render_biome_tile_rgba`, by the reference's design).
	var sdf0 := await _sdf_pair()
	var mv_nat_fit := await _water_move(Vector2(nat) + Vector2(0.5, 0.5))
	await _view(at, true)
	var deep0 := await _grab()
	var mv_deep0 := await _water_move(at)
	_crop(deep0, at, "F_before_z16")
	await _view(Vector2(nat) + Vector2(0.5, 0.5), true)
	var natimg := await _grab()
	var mv_nat_deep := await _water_move(Vector2(nat) + Vector2(0.5, 0.5))
	_crop(natimg, Vector2(nat) + Vector2(0.5, 0.5), "F_natural_lake_z16")
	_ok(mv_nat_fit > WATER_MOVES and mv_nat_deep > WATER_MOVES,
		"F7 positive control: a natural lake's pixel moves with the water colour (fit %d, z%d %d)" % [mv_nat_fit, int(DEEP), mv_nat_deep])
	_ok(mv_fit0 <= WATER_MOVES and mv_deep0 <= WATER_MOVES,
		"F8 negative control: before the press the spot does not (fit %d, z%d %d)" % [mv_fit0, int(DEEP), mv_deep0])

	## The press, through the shell's own handler.
	var ww := _find(_app, "world_workspace.gd")
	_ok(ww != null and ww.has_method("_on_force_lake"), "F3 the World workspace's force-lake handler is reachable")
	if ww == null:
		return
	ww._on_force_lake()
	await _settle()
	var m2 := _mask()
	_ok(_drawn_water(m2, spot), "F4 after the press the map's own water mask draws the stamped cell as water")
	var w := String(_br.sample_cell(spot.x, spot.y).get("water", "(absent)"))
	_ok(w == "lake", "F5 Sample says lake at the forced cell", "water=%s" % w)
	if key0 != "":
		_ok(String(_br.world_gen.river_network_key()) != key0, "F6 the river network's cache key moved with the press")

	await _view(at, false)
	var fit1 := await _grab()
	var mv_fit1 := await _water_move(at)
	_crop(fit0, at, "F_before_fit", 120)
	_crop(fit1, at, "F_after_fit", 120)
	_p("fit: spot %s -> %s; water-move %d -> %d (natural lake %d)" % [_px(fit0, at).to_html(false), _px(fit1, at).to_html(false),
		mv_fit0, mv_fit1, mv_nat_fit])
	_ok(mv_fit1 > WATER_MOVES, "F9 after the press the spot is drawn as water at fit zoom (water-move %d)" % mv_fit1)
	## Leg B after the press. The ring: land cells (drawn mask after the
	## press) within 3 cells of a cell the press made water -- where a forced
	## lake's biome band must now fall. The same cells are read in the
	## before-press pair, so a natural biome boundary nearby moves both
	## numbers alike and only the forced lake's own band separates them.
	var sdf1 := await _sdf_pair()
	var ring: Array = []
	for gy in range(spot.y - 24, spot.y + 25):
		for gx in range(spot.x - 24, spot.x + 25):
			if _drawn_water(m2, Vector2i(gx, gy)):
				continue
			var near := false
			for oy in range(-3, 4):
				for ox in range(-3, 4):
					var q := Vector2i(gx + ox, gy + oy)
					if _drawn_water(m2, q) and not _drawn_water(m1, q):
						near = true
			if near:
				ring.append(Vector2i(gx, gy))
	var b0 := _sdf_move(sdf0, ring)
	var b1 := _sdf_move(sdf1, ring)
	_p("B sdf_biomes ring: %d cells; mean move off->on before the press %.2f, after %.2f; lod_active=%s" % [ring.size(), b0, b1, _vh.lod_active()])
	_ok(not _vh.lod_active(), "B0 premise: fit zoom is drawn by the base raster, not the tiles")
	_ok(ring.size() >= 12, "B1 premise: the forced lake has a land ring to measure (%d cells)" % ring.size())
	## `WATER_MOVES` (16) as the margin: the same labelled "this pixel moved"
	## floor the water legs use. Pre-fix both numbers are the same ring's
	## natural-boundary move, so the difference is ~0 there.
	_ok(b1 > b0 + WATER_MOVES, "B2 sdf_biomes draws a biome band round the forced lake's shore (ring move %.2f -> %.2f)" % [b0, b1])
	_crop(sdf1[0], at, "B_sdfbiome_forced_off_fit", 90)
	_crop(sdf1[1], at, "B_sdfbiome_forced_on_fit", 90)
	await _view(at, true)
	var deep1 := await _grab()
	var mv_deep1 := await _water_move(at)
	_crop(deep1, at, "F_after_z16")
	_p("z16: spot %s -> %s; water-move %d -> %d (natural lake %d); lod_active=%s" % [_px(deep0, at).to_html(false),
		_px(deep1, at).to_html(false), mv_deep0, mv_deep1, mv_nat_deep, _vh.lod_active()])
	_ok(_vh.lod_active(), "F10 premise: z%d is drawn by the deep-zoom tiles" % int(DEEP))
	_ok(mv_deep1 > WATER_MOVES, "F11 after the press the spot is drawn as water at z%d (water-move %d)" % [int(DEEP), mv_deep1])

	## The lake labels read the same classification.
	var r: Dictionary = _br.labels_generate({"enabled": {"water": true}, "cull": {"on": false}, "lake_min_cells": 4})
	var named := false
	for lb: Dictionary in _br.labels_render_list():
		if bool(lb.get("generated", false)) and String(lb.get("class", "")) == "water" \
				and Vector2(float(lb["x"]), float(lb["y"])).distance_to(at) <= 10.0:
			named = true
	_ok(named, "F12 Generate labels names the forced lake (%s labels)" % r.get("total"))
	_br.world_gen.labels_clear_generated()

	## Saved and reopened, it is still drawn.
	var path := "%s/mapdata_forced.ctl" % _out
	var sv: Dictionary = _br.project_save_with_documents(path, {})
	_ok(bool(sv.get("ok", false)), "F13 the project saves", str(sv.get("error", "")))
	_ok(_br.load_save(path), "F14 and reopens")
	while _br.generating:
		await get_tree().create_timer(0.25).timeout
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	await _repaint()
	var m3 := _mask()
	_ok(_drawn_water(m3, spot), "F15 the reopened project draws the forced lake")
	var w3 := String(_br.sample_cell(spot.x, spot.y).get("water", "(absent)"))
	_ok(w3 == "lake", "F16 and Sample still says lake there", "water=%s" % w3)
	await _view(at, true)
	var deep3 := await _grab()
	_crop(deep3, at, "F_reopened_z16")
	var mv3 := await _water_move(at)
	_ok(mv3 > WATER_MOVES, "F17 and draws it as water at z%d after the reopen (water-move %d)" % [int(DEEP), mv3])


# ---- H: a hand-placed label's hit box/handles sit ON the drawn glyph, not
# half a cell down-right of it (item 1, `OUTSTANDING_WORK.md` "Map-data
# residuals", `label_bridge.rs::shell_label_box`) ---------------------------

func _leg_hitbox() -> void:
	var lx := 96.0
	var ly := 72.0
	var idx: int = _br.label_create(lx, ly, "Whitfell Cairn")
	_ok(idx >= 0, "H0 hand-placed label created")
	if idx < 0:
		return
	## `map_overlay.gd`'s own `_labels` (what `_draw_labels` reads) is a copy
	## `set_labels()` pushes; `label_create` alone does not refresh it (the
	## real shell's Label tool relies on `refresh_annotations()`'s drag-path
	## tail, `viewport_host.gd::refresh_annotations`, `overlay.set_labels(
	## _bridge.labels_render_list())`) -- without this the crop below draws
	## no glyph at all.
	_vh.refresh_annotations()
	await _view(Vector2(lx, ly), true)
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var ppc: float = (xf * ov._point_to_screen(Vector2(1, 0), rect) - xf * ov._point_to_screen(Vector2(0, 0), rect)).x
	## The engine's own drawn anchor -- exactly what `_draw_labels` projects
	## (`_point_to_screen(Vector2(lb["x"], lb["y"]), rect)`, no added offset).
	var anchor_grid := Vector2(lx, ly)
	var anchor_screen: Vector2 = xf * ov._point_to_screen(anchor_grid, rect)

	var px_per_cell: float = ov.label_px_per_cell()
	var h: Dictionary = _br.label_handles(idx, _vh.zoom(), px_per_cell)
	var resize: Dictionary = h.get("resize", {})
	_ok(not resize.is_empty(), "H1 label_handles returns a resize handle")
	if resize.is_empty():
		_br.label_delete(idx)
		return
	var resize_grid := Vector2(float(resize["x"]), float(resize["y"]))
	## Unrotated (angle 0): the resize handle sits at local (side/2, side/2)
	## from the box centre, so its distance from the box centre is always
	## `side/2 * sqrt(2)` -- `_label_side_from_handles`'s own identity,
	## solved back out here since the box centre (this fix's own subject) is
	## exactly what is under test.
	var side_half := resize_grid.distance_to(anchor_grid) / sqrt(2.0)
	_ok(side_half > 0.05, "H2 premise: the box has real size to test a corner inside it (side/2=%.3f cells)" % side_half)

	## What the PRE-FIX `shell_label_box` centred the box on: the reference's
	## own extra `+0.5` cell, which this fix removed (see that function's own
	## doc comment) -- reconstructed analytically (same technique
	## `_labelboxmodel_probe.gd`'s `_old_side` already uses), not by reverting
	## code, since the box is otherwise identical (same `side`).
	var old_centre := anchor_grid + Vector2(0.5, 0.5)
	var old_resize_grid := old_centre + Vector2(side_half, side_half)
	var old_resize_screen: Vector2 = xf * ov._point_to_screen(old_resize_grid, rect)
	var new_resize_screen: Vector2 = xf * ov._point_to_screen(resize_grid, rect)
	_p("H3 resize handle vs drawn anchor at z%d (%.1f px/cell): NEW %.2f px from anchor %s -- OLD (pre-fix, reconstructed) %.2f px from anchor %s" % [
		int(DEEP), ppc, anchor_screen.distance_to(new_resize_screen), new_resize_screen.round(),
		anchor_screen.distance_to(old_resize_screen), old_resize_screen.round()])
	_ok(anchor_screen.distance_to(old_resize_screen) > anchor_screen.distance_to(new_resize_screen),
		"H4 the pre-fix (reconstructed) handle sits farther from the drawn anchor than the fixed one, at z%d" % int(DEEP))

	## The regression this fixes, made concrete: a click just inside the
	## FIXED box's own top-left corner. It must hit the label (H5) -- and the
	## reconstructed pre-fix box, centred 0.5 cells further down-right, always
	## misses this exact point on both axes (H6): the corner is
	## `side_half - eps` cells outside the fixed box, so it is
	## `side_half + 0.5 - eps` outside the pre-fix one, which exceeds
	## `side_half` for any `eps < 0.5`.
	var eps := minf(0.1, side_half * 0.1)
	var corner_grid := anchor_grid - Vector2(side_half - eps, side_half - eps)
	var hit: int = _br.label_hit_test_mode(corner_grid.x, corner_grid.y, px_per_cell, 0)
	_ok(hit == idx, "H5 a click just inside the drawn box's own top-left corner hits the label",
		"hit=%d idx=%d corner=%s" % [hit, idx, corner_grid])
	var old_hit := absf(corner_grid.x - old_centre.x) <= side_half and absf(corner_grid.y - old_centre.y) <= side_half
	_ok(not old_hit, "H6 the SAME point would have missed the pre-fix (reconstructed) box, half a cell down-right of the glyph")

	## "labels" is in `HIDE` for the water pixel legs above -- shown here only
	## for this crop, so the screenshot actually has a glyph in it to look at.
	_vh.set_layer_visible("labels", true)
	await _repaint()
	var img := await _grab()
	_crop(img, anchor_grid, "H_label_hitbox_z16", 160)
	_vh.set_layer_visible("labels", false)
	_br.label_hit_test_mode(-1e9, -1e9, px_per_cell, 0)   ## deselect, tidy for any legs added after this one
	_br.label_delete(idx)


# ---- R: infrastructure routing avoids a FORCED lake, not just a natural one
# (item 3, `OUTSTANDING_WORK.md` "Map-data residuals",
# `infra_tools_bridge.rs::RouteInputs::build`) ------------------------------

func _leg_routing() -> void:
	var spot := Vector2i(-1, -1)
	for gy in range(40, GRID.y - 40, 8):
		for gx in range(70, GRID.x - 70, 8):
			if spot.x < 0 and _dry(Vector2i(gx, gy), 20, 300.0, true):
				spot = Vector2i(gx, gy)
	_ok(spot.x >= 0, "R0 premise: arid dry ground for a routing test (same search as F1)")
	if spot.x < 0:
		return
	var a := Vector2(spot.x - 24, spot.y)
	var b := Vector2(spot.x + 24, spot.y)
	## Force a lake wide enough that a straight A-B road would cross its
	## centre -- same stamp technique as `_leg_forced`.
	_br.sculpt_set_feature("lake")
	_br.sculpt_set_globals({"brush_size": 12.0})
	_br.sculpt_begin_stroke()
	_br.sculpt_add_point(spot.x, spot.y)
	_br.sculpt_end_stroke()
	_br.sculpt_commit("mapdata probe routing lake stamp")
	await _repaint()
	_ok(not _drawn_water(_mask(), spot), "R1 premise: the stamped bowl is still arid land before the force (unforced)")

	var ww := _find(_app, "world_workspace.gd")
	_ok(ww != null and ww.has_method("_on_force_lake"), "R2 the World workspace's force-lake handler is reachable")
	if ww == null:
		return
	ww._on_force_lake()
	await _settle()
	_ok(String(_br.sample_cell(spot.x, spot.y).get("water", "")) == "lake",
		"R3 the forced spot now samples as lake (this leg depends on item 2's own fix to see it)")

	## A Land-mode way (`road`) straight across the forced lake's centre.
	## Before the fix, `RouteInputs::build` never applied `forced_lakes`, so
	## this exact stretch of "dry" (per its own unforced `build_water_bodies`
	## reading) flat ground was the cheapest path and the road went straight
	## through -- this leg is the one that would have caught it.
	_ok(_br.way_begin("road"), "R4 a road way begins")
	_br.way_append_point(a.x, a.y)
	_br.way_append_point(b.x, b.y)
	var idx: int = _br.way_commit()
	_ok(idx >= 0, "R5 the road commits")
	if idx < 0:
		return
	## The overlay's own road-drawing data is a copy `set_civ_data()` pushes
	## (`viewport_host.gd::refresh()`); `way_commit()` alone does not refresh
	## it, so the crops below would draw no road at all without this.
	_vh.refresh()
	## `way_commit`'s own doc: its returned index counts every committed way
	## (roads and sea lanes together), so it is NOT an offset into
	## `roads()`'s array -- the manual road just committed is instead the
	## LAST `manual: true` "road"-type entry `roads()` returns (it is the
	## only one this probe has committed). `EngineBridge.roads()` is the real
	## shell's own name for `WorldGen::get_roads()` (`engine_bridge.gd`).
	var roads: Array = _br.roads()
	var way: Dictionary = {}
	for r: Dictionary in roads:
		if bool(r.get("manual", false)) and String(r.get("way_type", "")) == "road":
			way = r
	var pts: PackedVector2Array = way.get("points", PackedVector2Array())
	_ok(pts.size() > 0, "R6 premise: the committed road has real points", "way=%s" % [way])
	var closest := INF
	var crossed := false
	for p in pts:
		var w := String(_br.sample_cell(int(round(p.x)), int(round(p.y))).get("water", ""))
		if w == "lake":
			crossed = true
		closest = minf(closest, p.distance_to(Vector2(spot)))
	_ok(not crossed, "R7 the committed road does not cross the forced lake (no route point samples 'lake')")
	_p("R routing: spot=%s a=%s b=%s road points=%d closest approach to spot=%.1f cells crossed=%s" % [
		spot, a, b, pts.size(), closest, crossed])

	## "roads" is in `HIDE` for the water pixel legs above -- shown here only
	## for these crops, so the screenshots actually have the detour in them.
	_vh.set_layer_visible("roads", true)
	await _view(Vector2(spot), true)
	var img := await _grab()
	_crop(img, Vector2(spot), "R_routing_forced_lake_z16", 200)
	## A wider view at a shallower zoom, since the detour itself (10 cells'
	## closest approach, printed above) is bigger than one z16 crop.
	_vh.reset_view()
	await _frames(3)
	_vh.zoom_step(6.0 / _vh.zoom())
	_vh.move_view_to(spot.x - 0.5, spot.y - 0.5)
	await _settle()
	var wide := await _grab()
	_crop(wide, Vector2(spot), "R_routing_detour_wide", 260)
	_vh.set_layer_visible("roads", false)


func _finish() -> void:
	_vh.reset_view()
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fail == 0 else "%d FAILED" % _fail])
	get_tree().quit(1 if _fail > 0 else 0)
