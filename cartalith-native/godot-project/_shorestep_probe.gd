extends Node
## Probe for RV-4 and "Coastlines and lake shores are pixel-stepped"
## (OUTSTANDING_WORK.md, filed 2026-09-27 from the river-mouth screenshots):
## the owner's bar is *no pixelated or square artefacts at any zoom*
## (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`). This reads the SCREEN.
##
## **What "stair-stepped" is measured as.** Three screen measures, each
## chosen because the obvious one was tried and misled (see each function):
##
##  * `_offgrid` -- where the drawn shoreline crosses between two pixels, is
##    that inside one map texel or at a texel boundary? A coast drawn along
##    cell edges crosses only at texel boundaries: 0 by construction.
##  * `_offgrid`'s `aa` -- are the shoreline's pixels partly covered? A
##    nearest-cell edge never is.
##  * `_stairs` -- where the shoreline runs between the axes over a cell or
##    so, how much of it runs ALONG an axis pixel by pixel? Chance for a
##    direction unrelated to the axes is `2 * AXIS_WIN / 90` = 0.22; cell
##    steps raise it.
##
## A bare axis histogram of edge directions (the first metric written here) is
## printed beside them, never judged: it called a smooth lake shore that
## happens to run north-south a staircase, and at 1.5 px a cell it could not
## tell HEAD's cell coast from a smooth one (0.35 vs 0.29).
##
## "Near a shoreline" is chosen from the DATA, never from the picture
## (MISTAKES.md, "Measure an effect inside a subset of pixels"): a pixel whose
## ground position lies in a grid square with a water and a land corner, or
## in a square next to one, per the drawn-water mask
## (`WorldGen::river_water_mask`, the classification the map draws). Each
## such square is sea or lake by its water's `sample_cell` kind; a square
## touching both is left out.
##
## Legs, per preset in PRESETS:
##
##  G. fit and x1.4, the shoreline's GEOMETRY: the drawn-water mask shown in
##     place of the map texture, through the map's own `TextureRect` and
##     material, every overlay hidden -- so the capture is exactly the water
##     the map draws, whatever the terrain colours around it. Per kind (sea,
##     lake), two measures judged (see `_offgrid`): the fraction of shoreline
##     crossings that fall INSIDE a cell (a cell-edge coast scores exactly 0 --
##     the stair-step by definition) >= OFFGRID_BAR, and the fraction of edge
##     pixels drawn partly covered >= AA_BAR. Both fail on HEAD's nearest-cell
##     coast by construction. Protects: `map_shore.gdshader`,
##     `render::shore_field`. The axis fraction of the geometry and of the real
##     colours is printed beside them, not judged: at 1.5 screen pixels a cell
##     a one-cell step is a one-pixel jag, and a 3x3 gradient cannot tell the
##     two apart (measured: HEAD 0.35 sea / 0.43 lake at fit).
##  Z. z16 (the deep-zoom tiles) at VISITS sea and VISITS lake shore points:
##     the water's outline as the tiles draw it (`_waterness`: two captures
##     differing only in the water's colour), and on it the stair fraction
##     (`_stairs`) <= STAIR_BAR per kind over all visits. Lake protects
##     `render::is_lake_pixel`'s smooth band (fails on HEAD's `fq > 0.35` /
##     flat-stamp squares); sea is a control HEAD passes too.
##  I. fit, with the smoothing switched OFF on the map's material and on
##     again: every pixel outside the band is identical (the shader touches
##     nothing but shore pixels, so every preset's colours, the paper and the
##     grade are untouched), and band pixels move (positive control). Skipped
##     -- not passed -- on a build without the material (HEAD).
##  R. fit, the Rivers layer on vs off with the river painted into the map
##     (RIM-1): no open-water pixel moves, land pixels do (positive control).
##     Protects `map_shore.gdshader`'s "river is composed under the water".
##
## Crops for looking at: `<preset>_<kind><i>_<zoom>.png` around each visit
## (fit and x1.4 magnified 8x nearest; z16 at 1:1).
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _shorestep_probe.tscn -- --out DIR --seed N
##
## Arguments -- every one is read below; an unknown one aborts:
##   --out DIR     where the PNGs and shorestep.json land (required)
##   --seed N      world seed (default 483920)
##   --grid WxH    world size (default 1024x656)
##   --presets a,b only these presets (default PRESETS)
##
## Exit status: 0 every assertion held, 1 one failed, 2 could not run.

const PRESETS := ["Default", "Ink"]
const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict", "rivers"]
## The zooms below the deep-zoom switch, as multiples of the fit zoom: the
## opening view and x1.4, the owner's reported range.
const FIT_ZOOMS := [1.0, 1.4]
const DEEP := 16.0
## Shore points visited at z16 per kind.
const VISITS := 3
## Degrees either side of an axis that count as "axis-aligned". Labelled
## judgement: wide enough that a staircase's straight runs all fall in it
## (their Sobel direction is exact), narrow enough that a uniformly spread
## direction scores 0.22.
const AXIS_WIN := 10.0
## Edge pixels the metric needs before it means anything (a leg that sees
## fewer is reported as unable to measure, never as a pass).
const MIN_EDGE_PX := 150
## G's off-grid bar. Labelled judgement: a cell-edge coast scores exactly 0,
## and a sub-cell one `1 - 1/ppc` for evenly spread crossings (0.33 at fit's
## ~1.5 px a cell, 0.52 at x1.4), so 0.15 needs the shoreline to leave the
## cell edges at least half as often as an even spread would.
const OFFGRID_BAR := 0.15
## G's antialiasing bar. Labelled judgement: a hard nearest-cell edge scores
## exactly 0, an antialiased one nearly every edge pixel; 0.3 asks for a
## clear majority short of that.
const AA_BAR := 0.3
## `_stairs`' "between the axes": a box direction at least this many degrees
## from both axes. Labelled judgement: past AXIS_WIN with a margin, so a
## staircase's own runs can never count as its coast's direction.
const DIAG_MIN := 15.0
## Z's stair-fraction bar (`_stairs`). Labelled judgement: chance level (a
## shoreline whose own direction is unrelated to the axes) is 0.22, and a
## cell staircase raises it by as much of the shoreline as it covers --
## measured on seed 483920 (Default), the z16 lake outline at HEAD 0.325
## (one visit 0.445, where the steps run a whole shore) against 0.227 with the
## smooth band. 0.27 is chance plus a fifth: a shoreline with a visible run of
## cell steps fails it.
const STAIR_BAR := 0.27

var GRID := Vector2i(1024, 656)
var _seed := 483920
var _out := ""
var _only: PackedStringArray = []
var _fail := 0
var _app: Node
var _vh: Control
var _br: Node
var _report := {}
var _mask: Image
## Per cell: 0 land, 1 ocean, 2 lake (from the mask, kinds from `sample_cell`).
var _kind := PackedByteArray()
## Per grid square (index of its top-left cell): 0 none, 1 sea, 2 lake band.
var _band := PackedByteArray()
## Mixed squares only (the shoreline itself), for picking visits.
var _mixed := PackedByteArray()
var _comp_size := {}
## The texel-ID image (`_texel_ids`): each texel's own column and row, low
## byte, in R and G.
var _ids: Image
var _comp := PackedInt32Array()


func _ok(cond: bool, what: String) -> void:
	print(("SHORESTEP  ok    " if cond else "SHORESTEP  FAIL  ") + what)
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
			"--grid":
				var p := args[i + 1].split("x"); GRID = Vector2i(int(p[0]), int(p[1])); i += 1
			"--presets":
				_only = args[i + 1].split(","); i += 1
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
	await _frames(4)
	var mt: Texture2D = _br.river_water_mask() if _br.has_method("river_water_mask") else null
	if mt == null:
		printerr("PROBE-CANNOT-RUN: no drawn-water mask (river_water_mask) -- stale .dll?")
		get_tree().quit(2)
		return
	_mask = mt.get_image().duplicate()
	_mask.convert(Image.FORMAT_L8)
	if _mask.get_size() != GRID:
		printerr("PROBE-CANNOT-RUN: mask %s is not the grid %s" % [_mask.get_size(), GRID])
		get_tree().quit(2)
		return
	_build_kinds()
	_report["seed"] = _seed
	_report["has_shore_material"] = _vh.map_view.material != null
	var visits := _pick_visits()
	print("SHORESTEP  seed %d: %d visits %s" % [_seed, visits.size(), str(visits)])
	_ok(visits.size() >= 2 * VISITS, "shore points of both kinds to visit (%d of %d)" % [visits.size(), 2 * VISITS])

	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	var rw := _find(_app, "render_workspace.gd")
	if rw == null:
		printerr("PROBE-CANNOT-RUN: RenderWorkspace not reachable")
		get_tree().quit(2)
		return
	var tiles: Array = rw.get("_preset_tiles")
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
		await _frames(8)
		for z: float in FIT_ZOOMS:
			await _fit_legs(pname, z, visits)
		await _deep_legs(pname, visits)
	_vh.reset_view()
	_finish()


## Kinds per cell and the band, from the mask. Components are 4-connected
## water; each is named once from `sample_cell` (its first cell that
## `sample_cell` does not call land -- the two can disagree at single cells,
## `_rivstyle_probe.gd`'s `_water` note).
func _build_kinds() -> void:
	var W := GRID.x; var H := GRID.y
	var m := _mask.get_data()
	_kind.resize(W * H); _kind.fill(0)
	_comp.resize(W * H); _comp.fill(-1)
	var nc := 0
	var stack := PackedInt32Array()
	for s in W * H:
		if m[s] <= 127 or _comp[s] >= 0:
			continue
		var cells := PackedInt32Array()
		stack.clear(); stack.append(s); _comp[s] = nc
		while not stack.is_empty():
			var i := stack[stack.size() - 1]; stack.remove_at(stack.size() - 1)
			cells.append(i)
			var x := i % W; var y := i / W
			for q in [i - 1 if x > 0 else -1, i + 1 if x < W - 1 else -1, i - W if y > 0 else -1, i + W if y < H - 1 else -1]:
				if q >= 0 and m[q] > 127 and _comp[q] < 0:
					_comp[q] = nc; stack.append(q)
		var k := 1
		for t in mini(cells.size(), 25):
			var c := cells[t * cells.size() / mini(cells.size(), 25)]
			var w := String(_br.sample_cell(c % W, c / W).get("water", "land"))
			if w != "land":
				k = 2 if w == "lake" else 1
				break
		for c in cells:
			_kind[c] = k
		_comp_size[nc] = cells.size()
		nc += 1
	_band.resize(W * H); _band.fill(0)
	_mixed.resize(W * H); _mixed.fill(0)
	for j in H - 1:
		for i in W - 1:
			var a := _kind[j * W + i]; var b := _kind[j * W + i + 1]
			var c := _kind[(j + 1) * W + i]; var d := _kind[(j + 1) * W + i + 1]
			var nw := int(a > 0) + int(b > 0) + int(c > 0) + int(d > 0)
			if nw == 0 or nw == 4:
				continue
			var kk := maxi(maxi(a, b), maxi(c, d))
			var mn := 3
			for v in [a, b, c, d]:
				if v > 0:
					mn = mini(mn, v)
			## A square with both kinds of water in it is neither.
			_mixed[j * W + i] = kk if mn == kk else 255
	for j in H:
		for i in W:
			var v := _mixed[j * W + i]
			if v == 0:
				continue
			for dj in [-1, 0, 1]:
				for di in [-1, 0, 1]:
					var ii: int = i + di; var jj: int = j + dj
					if ii < 0 or jj < 0 or ii >= W or jj >= H:
						continue
					var o := _band[jj * W + ii]
					_band[jj * W + ii] = v if o == 0 or o == v else 255
	var ns := 0; var nl := 0
	for v in _mixed:
		ns += int(v == 1); nl += int(v == 2)
	print("SHORESTEP  shoreline squares: sea %d, lake %d" % [ns, nl])


## VISITS sea and VISITS lake shoreline squares: well inside the map frame,
## on a water body of at least 40 cells, at least 60 cells apart, scanned in
## a fixed order so every run of a seed visits the same ground.
func _pick_visits() -> Array:
	var W := GRID.x; var H := GRID.y
	var out: Array = []
	var per := {1: 0, 2: 0}
	var margin := int(ceil(0.08 * float(W)))
	for pass_i in 2:
		for j in range(margin, H - margin, 7):
			for i in range(margin, W - margin, 7):
				var v := _mixed[j * W + i]
				if v != 1 and v != 2 or per[v] >= VISITS:
					continue
				## The water corner's component size.
				var big := false
				for q in [j * W + i, j * W + i + 1, (j + 1) * W + i, (j + 1) * W + i + 1]:
					if _comp[q] >= 0 and int(_comp_size[_comp[q]]) >= 40:
						big = true
				if not big:
					continue
				var far := true
				for o: Dictionary in out:
					if (o["p"] as Vector2).distance_to(Vector2(i + 1, j + 1)) < (60.0 if pass_i == 0 else 30.0):
						far = false
				if far:
					out.append({"p": Vector2(i + 1, j + 1), "kind": "sea" if v == 1 else "lake"})
					per[v] += 1
	return out


func _grab_full() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle_tiles() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


func _no_labels() -> void:
	_vh.overlay.set("_labels", [])
	_vh.overlay.queue_redraw()


## The screen -> ground mapping of the current view: `(ax, bx, ay, by)` with
## ground x = ax * (px + 0.5) + bx, likewise y, through the overlay's own fit
## rect and canvas transform (no rotation anywhere in this shell).
func _mapping() -> Array:
	var ov: Control = _vh.overlay
	var rect: Rect2 = ov.displayed_rect()
	var inv := ov.get_global_transform_with_canvas().affine_inverse()
	var o: Vector2 = inv * Vector2.ZERO
	var e: Vector2 = inv * Vector2(1, 1)
	var sx := (e.x - o.x) / rect.size.x * float(GRID.x)
	var sy := (e.y - o.y) / rect.size.y * float(GRID.y)
	return [sx, (o.x - rect.position.x) / rect.size.x * float(GRID.x), sy, (o.y - rect.position.y) / rect.size.y * float(GRID.y)]


func _host_rect(img: Image) -> Rect2i:
	return Rect2i(Vector2i(_vh.global_position.round()), Vector2i(_vh.size.round())).intersection(Rect2i(Vector2i.ZERO, img.get_size())).grow(-3)


## Per pixel of `host`, from the mapping `mp`: the band kind of the grid
## square under it (1 sea, 2 lake, 255 both); `100 + kind` for a square whose
## four corners are all water of one kind (open water, the colour reference at
## z16); 0 for any other square; 254 for a pixel within 3.5 cells of the
## grid's edge or off it (the map frame, the letterbox), which no leg measures.
func _band_of(host: Rect2i, mp: Array) -> PackedByteArray:
	var W := GRID.x; var H := GRID.y
	var cols := PackedInt32Array(); cols.resize(host.size.x)
	for x in host.size.x:
		var gx: float = mp[0] * (float(host.position.x + x) + 0.5) + mp[1]
		cols[x] = int(floor(gx - 0.5)) if gx >= 3.5 and gx <= float(W) - 3.5 else -1
	var out := PackedByteArray(); out.resize(host.size.x * host.size.y); out.fill(254)
	for y in host.size.y:
		var gy: float = mp[2] * (float(host.position.y + y) + 0.5) + mp[3]
		if gy < 3.5 or gy > float(H) - 3.5:
			continue
		var j := int(floor(gy - 0.5))
		for x in host.size.x:
			var i := cols[x]
			if i < 0:
				continue
			var b := _band[j * W + i]
			if b != 0:
				out[y * host.size.x + x] = b
			else:
				var k := _kind[j * W + i]
				out[y * host.size.x + x] = (100 + k) if (k > 0 and _kind[j * W + i + 1] == k
					and _kind[(j + 1) * W + i] == k and _kind[(j + 1) * W + i + 1] == k) else 0
	return out


## Rec.709 luma of `img` over `host`, row-major.
func _luma(img: Image, host: Rect2i) -> PackedFloat32Array:
	var W := img.get_width(); var d := img.get_data()
	var L := PackedFloat32Array(); L.resize(host.size.x * host.size.y)
	for y in host.size.y:
		for x in host.size.x:
			var o := ((host.position.y + y) * W + host.position.x + x) * 3
			L[y * host.size.x + x] = 0.2126 * d[o] + 0.7152 * d[o + 1] + 0.0722 * d[o + 2]
	return L


## **How much of each pixel the tiles draw as water**, 0..255, from two
## captures of the same z16 view that differ ONLY in the water's colour: `a`
## as it is, `b` with `sea_ramp_strength` at 1 (the "Bathymetric sea ramp",
## which `render.rs::sea_color_core` mixes into every sea and lake colour and
## nothing else). A pixel's change, less the noise floor `ctl` (the largest
## change between two captures of one state), over the typical change on
## open water (the median over pixels above open-water squares, `_band_of`'s
## `100 + kind`). So a land pixel reads 0, a water pixel 255, and a pixel the
## tile's linear filtering spreads across the shore in between: the water's
## outline as drawn, whatever the palette -- the texture swap the fit legs use
## has no counterpart for the tiles, and reading "looks like water" off one
## capture's colours was tried and measured the land's own colour steps.
## Empty when fewer than 400 open-water pixels are on screen or the open
## water barely changed (no outline to read).
func _waterness(a: Image, b: Image, host: Rect2i, codes: PackedByteArray, ctl: int) -> PackedFloat32Array:
	var W := a.get_width(); var da := a.get_data(); var db := b.get_data()
	var hw := host.size.x; var hh := host.size.y
	var diff := PackedFloat32Array(); diff.resize(hw * hh)
	var ref := PackedFloat32Array()
	for y in hh:
		for x in hw:
			var o := ((host.position.y + y) * W + host.position.x + x) * 3
			var dd := float(absi(da[o] - db[o]) + absi(da[o + 1] - db[o + 1]) + absi(da[o + 2] - db[o + 2]))
			diff[y * hw + x] = dd
			var c := codes[y * hw + x]
			if (c == 101 or c == 102) and (x & 1) == 0 and (y & 1) == 0:
				ref.append(dd)
	if ref.size() < 400:
		return PackedFloat32Array()
	ref.sort()
	var full := ref[ref.size() / 2]
	## A water change at least 8 levels (summed channels) above the noise, or
	## there is no outline to read. Labelled judgement.
	if full < float(ctl) + 8.0:
		return PackedFloat32Array()
	var L := PackedFloat32Array(); L.resize(hw * hh)
	for i in hw * hh:
		L[i] = 255.0 * clampf((diff[i] - float(ctl)) / (full - float(ctl)), 0.0, 1.0)
	return L


## The weighted axis fraction for band kind `k` over scalar image `L` (host
## sized): Sobel direction and magnitude at every pixel of that band, only
## pixels whose magnitude is at least a quarter of the band's 99th percentile
## (the shoreline's own step, not texture noise), weighted by magnitude.
## Returns `[fraction, n]`.
func _axis_metric(L: PackedFloat32Array, hw: int, hh: int, codes: PackedByteArray, k: int) -> Array:
	var mags := PackedFloat32Array(); var angs := PackedFloat32Array()
	for y in range(1, hh - 1):
		for x in range(1, hw - 1):
			if codes[y * hw + x] != k:
				continue
			var i := y * hw + x
			var gx := (L[i - hw + 1] + 2.0 * L[i + 1] + L[i + hw + 1]) - (L[i - hw - 1] + 2.0 * L[i - 1] + L[i + hw - 1])
			var gy := (L[i + hw - 1] + 2.0 * L[i + hw] + L[i + hw + 1]) - (L[i - hw - 1] + 2.0 * L[i - hw] + L[i - hw + 1])
			var m := sqrt(gx * gx + gy * gy)
			if m <= 0.0:
				continue
			mags.append(m); angs.append(fposmod(rad_to_deg(atan2(gy, gx)), 90.0))
	if mags.is_empty():
		return [NAN, 0]
	var sorted := mags.duplicate(); sorted.sort()
	var thr := 0.25 * sorted[int(0.99 * float(sorted.size() - 1))]
	var wsum := 0.0; var wax := 0.0; var n := 0
	for t in mags.size():
		if mags[t] < thr:
			continue
		wsum += mags[t]; n += 1
		if angs[t] < AXIS_WIN or angs[t] > 90.0 - AXIS_WIN:
			wax += mags[t]
	return [wax / wsum if wsum > 0.0 else NAN, n]


## The texel-ID texture: at every texel, its own column (low byte) in R and
## row (low byte) in G. Shown in place of the map through the map's own
## nearest-filtered `TextureRect` with the smoothing switched off, a capture of
## it says which texel every screen pixel shows -- read off the screen, not
## computed (a computed mapping was tried first and put 15% of HEAD's fit
## crossings inside a cell, where HEAD by construction has none).
func _texel_ids() -> Image:
	if _ids == null:
		var b := PackedByteArray(); b.resize(GRID.x * GRID.y * 3)
		for y in GRID.y:
			for x in GRID.x:
				var o := (y * GRID.x + x) * 3
				b[o] = x & 255; b[o + 1] = y & 255; b[o + 2] = 0
		_ids = Image.create_from_data(GRID.x, GRID.y, false, Image.FORMAT_RGB8, b)
	return _ids


## **Off the cell grid, and antialiased** -- the geometry capture `geo`'s
## shoreline against the texel-ID capture `ids` (same view), for band kind
## `k`:
##
##  * `offgrid`: of every place where two side-by-side (or stacked) pixels
##    fall on opposite sides of the drawn shoreline (one below half coverage,
##    one above), the fraction where both pixels show the SAME texel. A coast
##    drawn along cell edges can change only where the texel changes, so it
##    scores exactly 0 -- the stair-step, by definition; a sub-cell shoreline
##    crosses inside cells as often as the cell is wider than a pixel,
##    `1 - 1/ppc` for crossings spread evenly (0.33 at fit, 0.52 at x1.4).
##  * `aa`: of the band pixels with both sides of the shoreline in their 3x3,
##    the fraction drawn partly covered (between 8 and 247): 0 for a hard
##    nearest-cell edge, most of them for an antialiased one.
##
## Returns `[offgrid, crossings, aa, edge_px]`.
func _offgrid(geo: Image, ids: Image, host: Rect2i, codes: PackedByteArray, k: int) -> Array:
	var hw := host.size.x; var hh := host.size.y
	var W := geo.get_width(); var d := geo.get_data(); var t := ids.get_data()
	var val := func(x: int, y: int) -> int: return d[((host.position.y + y) * W + host.position.x + x) * 3]
	var same := func(x0: int, y0: int, x1: int, y1: int) -> bool:
		var a := ((host.position.y + y0) * W + host.position.x + x0) * 3
		var b := ((host.position.y + y1) * W + host.position.x + x1) * 3
		return t[a] == t[b] and t[a + 1] == t[b + 1]
	var cross := 0; var inside := 0; var edge := 0; var partial := 0
	for y in range(1, hh - 1):
		for x in range(1, hw - 1):
			if codes[y * hw + x] != k:
				continue
			var v0: int = val.call(x, y)
			if codes[y * hw + x + 1] == k and (v0 > 127) != (int(val.call(x + 1, y)) > 127):
				cross += 1; inside += int(same.call(x, y, x + 1, y))
			if codes[(y + 1) * hw + x] == k and (v0 > 127) != (int(val.call(x, y + 1)) > 127):
				cross += 1; inside += int(same.call(x, y, x, y + 1))
			var lo := false; var hi := false
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					var q: int = val.call(x + dx, y + dy)
					lo = lo or q <= 127; hi = hi or q > 127
			if lo and hi:
				edge += 1; partial += int(v0 > 8 and v0 < 247)
	return [float(inside) / float(cross) if cross > 0 else NAN, cross, float(partial) / float(edge) if edge > 0 else NAN, edge]


## **The stair fraction** for band kind `k` over scalar image `L` (host
## sized). At every strong edge pixel on the outline's ramp (`L` between a
## fifth and four fifths of full; Sobel magnitude at least a quarter of the
## band's 99th percentile) two directions: the pixel's own, and the
## edge's over a box of `r` px either side (the structure tensor summed over
## it, about three cells across). Among the pixels whose box direction runs
## BETWEEN the axes (more than `DIAG_MIN` degrees from both), the
## magnitude-weighted fraction whose own direction lies ALONG an axis (within
## `AXIS_WIN`). That conjunction is a staircase exactly: a coast running at
## 20-45 degrees drawn as axis-parallel runs scores near 1, while a smooth or
## naturally ragged one scores about the chance level `2 * AXIS_WIN / 90` =
## 0.22, and a genuinely straight north-south shore is not counted at all
## (its box direction is on the axis) -- the two confounders a bare axis
## histogram (measured: 0.56 on a smooth diagonal lake shore beside a straight
## one) and a bare direction deviation (measured: higher on a ragged natural
## shore than on HEAD's steps) each fell to. Returns `[fraction, n]`.
func _stairs(L: PackedFloat32Array, hw: int, hh: int, codes: PackedByteArray, k: int, r: int) -> Array:
	var gxs := PackedFloat32Array(); gxs.resize(hw * hh)
	var gys := PackedFloat32Array(); gys.resize(hw * hh)
	for y in range(1, hh - 1):
		for x in range(1, hw - 1):
			var i := y * hw + x
			gxs[i] = (L[i - hw + 1] + 2.0 * L[i + 1] + L[i + hw + 1]) - (L[i - hw - 1] + 2.0 * L[i - 1] + L[i + hw - 1])
			gys[i] = (L[i + hw - 1] + 2.0 * L[i + hw] + L[i + hw + 1]) - (L[i - hw - 1] + 2.0 * L[i - hw] + L[i - hw + 1])
	## Integral images of the tensor's three terms, (hw+1) x (hh+1).
	var iw := hw + 1
	var sxx := PackedFloat64Array(); sxx.resize(iw * (hh + 1))
	var syy := PackedFloat64Array(); syy.resize(iw * (hh + 1))
	var sxy := PackedFloat64Array(); sxy.resize(iw * (hh + 1))
	for y in hh:
		var rxx := 0.0; var ryy := 0.0; var rxy := 0.0
		for x in hw:
			var i := y * hw + x
			rxx += gxs[i] * gxs[i]; ryy += gys[i] * gys[i]; rxy += gxs[i] * gys[i]
			var o := (y + 1) * iw + x + 1
			sxx[o] = sxx[o - iw] + rxx; syy[o] = syy[o - iw] + ryy; sxy[o] = sxy[o - iw] + rxy
	var mags := PackedFloat32Array(); var devs := PackedFloat32Array()
	for y in range(1, hh - 1):
		for x in range(1, hw - 1):
			if codes[y * hw + x] != k:
				continue
			var i := y * hw + x
			## On the outline's own ramp only (a fifth to four fifths of the
			## way from land to water): the water's interior and the land's
			## texture have gradients too, and they are not the shoreline.
			if L[i] < 51.0 or L[i] > 204.0:
				continue
			var m := sqrt(gxs[i] * gxs[i] + gys[i] * gys[i])
			if m <= 0.0:
				continue
			var x0 := maxi(0, x - r); var x1 := mini(hw, x + r + 1)
			var y0 := maxi(0, y - r); var y1 := mini(hh, y + r + 1)
			var jxx := sxx[y1 * iw + x1] - sxx[y0 * iw + x1] - sxx[y1 * iw + x0] + sxx[y0 * iw + x0]
			var jyy := syy[y1 * iw + x1] - syy[y0 * iw + x1] - syy[y1 * iw + x0] + syy[y0 * iw + x0]
			var jxy := sxy[y1 * iw + x1] - sxy[y0 * iw + x1] - sxy[y1 * iw + x0] + sxy[y0 * iw + x0]
			## The tensor's dominant direction is the edge NORMAL, like the
			## Sobel vector: both are compared as normals, modulo 90.
			var ts := fposmod(rad_to_deg(0.5 * atan2(2.0 * jxy, jxx - jyy)), 90.0)
			if ts < DIAG_MIN or ts > 90.0 - DIAG_MIN:
				continue
			var tl := fposmod(rad_to_deg(atan2(gys[i], gxs[i])), 90.0)
			mags.append(m); devs.append(1.0 if tl < AXIS_WIN or tl > 90.0 - AXIS_WIN else 0.0)
	if mags.is_empty():
		return [NAN, 0]
	var sorted := mags.duplicate(); sorted.sort()
	var thr := 0.25 * sorted[int(0.99 * float(sorted.size() - 1))]
	var ws := 0.0; var wd := 0.0; var n := 0
	for t in mags.size():
		if mags[t] >= thr:
			ws += mags[t]; wd += mags[t] * devs[t]; n += 1
	return [wd / ws if ws > 0.0 else NAN, n]


func _fit_legs(pname: String, z: float, visits: Array) -> void:
	_vh.reset_view()
	await _frames(3)
	if z != 1.0:
		_vh.zoom_step(z)
	_no_labels()
	await _frames(6)
	var zs := "x%.1f" % z
	_ok(not _vh.lod_active(), "%s %s is below the deep-zoom switch" % [pname, zs])
	var real := await _grab_full()
	var host := _host_rect(real)
	var mp := _mapping()
	var codes := _band_of(host, mp)
	## G: the mask through the map's own TextureRect and material.
	var map_tex: Texture2D = _vh.map_view.texture
	_vh.map_view.texture = ImageTexture.create_from_image(_mask)
	_vh.overlay.visible = false
	await _frames(3)
	var geo := await _grab_full()
	## The texel-ID capture, the smoothing off so each pixel shows one texel.
	var mat0 := _vh.map_view.material as ShaderMaterial
	if mat0 != null:
		mat0.set_shader_parameter("shore_on", false)
	_vh.map_view.texture = ImageTexture.create_from_image(_texel_ids())
	await _frames(3)
	var ids := await _grab_full()
	if mat0 != null:
		mat0.set_shader_parameter("shore_on", _br.shore_field_texture() != null)
	_vh.overlay.visible = true
	_vh.map_view.texture = map_tex
	await _frames(3)
	geo.save_png("%s/%s_geo_%s.png" % [_out, pname.to_snake_case(), zs])
	var Lg := _luma(geo, host)
	var Lr := _luma(real, host)
	for k in [1, 2]:
		var nm := "sea" if k == 1 else "lake"
		var og := _offgrid(geo, ids, host, codes, k)
		var ag := _axis_metric(Lg, host.size.x, host.size.y, codes, k)
		var ar := _axis_metric(Lr, host.size.x, host.size.y, codes, k)
		print("SHORESTEP  G %-8s %-5s %-4s off-grid crossings %.3f of %d, antialiased edge px %.3f of %d; axis fraction geometry %.3f/%d, colour %.3f/%d"
			% [pname, zs, nm, og[0], og[1], og[2], og[3], ag[0], ag[1], ar[0], ar[1]])
		_report["G_%s_%s_%s" % [pname, zs, nm]] = {"offgrid": og[0], "crossings": og[1], "aa": og[2], "edge_px": og[3],
			"axis_geo": ag[0], "axis_geo_n": ag[1], "axis_colour": ar[0], "axis_colour_n": ar[1]}
		_ok(int(og[1]) >= MIN_EDGE_PX, "G %s %s %s: enough shoreline crossings to measure (%d >= %d)" % [pname, zs, nm, og[1], MIN_EDGE_PX])
		if int(og[1]) >= MIN_EDGE_PX:
			_ok(og[0] >= OFFGRID_BAR, "G %s %s %s: the shoreline leaves the cell edges (off-grid %.3f >= %.2f)" % [pname, zs, nm, og[0], OFFGRID_BAR])
			_ok(og[2] >= AA_BAR, "G %s %s %s: the shoreline is antialiased (%.3f >= %.2f)" % [pname, zs, nm, og[2], AA_BAR])
	## I: the material's own off switch, at fit only.
	var mat := _vh.map_view.material as ShaderMaterial
	if z == 1.0:
		if mat == null:
			print("SHORESTEP  I %s: SKIPPED -- no shore material on the map (a build before RV-4)" % pname)
		else:
			mat.set_shader_parameter("shore_on", false)
			await _frames(3)
			var off := await _grab_full()
			mat.set_shader_parameter("shore_on", _br.shore_field_texture() != null)
			await _frames(3)
			var on2 := await _grab_full()
			var A := real.get_data(); var B := off.get_data(); var C := on2.get_data()
			var W := real.get_width()
			var out_moved := 0; var band_moved := 0; var repeat_moved := 0
			for y in host.size.y:
				for x in host.size.x:
					var o := ((host.position.y + y) * W + host.position.x + x) * 3
					var dab := absi(A[o] - B[o]) + absi(A[o + 1] - B[o + 1]) + absi(A[o + 2] - B[o + 2])
					repeat_moved += int(A[o] != C[o] or A[o + 1] != C[o + 1] or A[o + 2] != C[o + 2])
					var c := codes[y * host.size.x + x]
					if c == 0 or c >= 100 and c < 254:
						out_moved += int(dab > 0)
					elif c != 254:
						band_moved += int(dab > 0)
			print("SHORESTEP  I %s: smoothing off vs on -- band px moved %d, off-band px moved %d (repeat capture moved %d)" % [pname, band_moved, out_moved, repeat_moved])
			_ok(repeat_moved == 0, "I %s: two captures of the same state agree (control)" % pname)
			_ok(band_moved > 1000, "I %s: the smoothing moves shoreline pixels (positive control, %d > 1000)" % [pname, band_moved])
			_ok(out_moved == 0, "I %s: no pixel away from a shoreline moves (%d)" % [pname, out_moved])
	## R: the painted river (RIM-1, `map_shore.gdshader`) lies on land only. The
	## Rivers layer on vs off, with the smoothing on: no pixel inside open water
	## (a square whose four corners are all one water kind) may move -- the
	## river is composed under the shore's water, never on it -- and land pixels
	## do move (positive control). Fit only; skipped, not passed, where no river
	## field was built (the stroke path then owns the river).
	if z == 1.0:
		if not _br.rivers_painted():
			print("SHORESTEP  R %s: SKIPPED -- the rivers are not painted into the map (stroke path)" % pname)
		else:
			_vh.set_layer_visible("rivers", true)
			await _frames(4)
			var r_on := await _grab_full()
			_vh.set_layer_visible("rivers", false)
			await _frames(4)
			var r_off := await _grab_full()
			var RA := r_on.get_data(); var RB := r_off.get_data()
			var RW := r_on.get_width()
			var wet_moved := 0; var land_moved := 0; var wet_px := 0
			for y in host.size.y:
				for x in host.size.x:
					var c := codes[y * host.size.x + x]
					var o := ((host.position.y + y) * RW + host.position.x + x) * 3
					var moved := RA[o] != RB[o] or RA[o + 1] != RB[o + 1] or RA[o + 2] != RB[o + 2]
					if c >= 100 and c < 254:
						wet_px += 1
						wet_moved += int(moved)
					elif c == 0:
						land_moved += int(moved)
			print("SHORESTEP  R %s: river layer on vs off -- open-water px moved %d of %d, land px moved %d" % [pname, wet_moved, wet_px, land_moved])
			_report["R_%s" % pname] = {"open_water_moved": wet_moved, "open_water_px": wet_px, "land_moved": land_moved}
			_ok(wet_px > 1000, "R %s: open water to test (%d px > 1000)" % [pname, wet_px])
			_ok(land_moved > 500, "R %s: the river layer paints land (positive control, %d > 500)" % [pname, land_moved])
			_ok(wet_moved == 0, "R %s: the painted river never lies on open water (%d px moved)" % [pname, wet_moved])
	## Crops around each visit.
	for vi in visits.size():
		var v: Dictionary = visits[vi]
		var p: Vector2 = v["p"]
		var cx := int(round(((p.x - mp[1]) / mp[0]) - 0.5)); var cy := int(round(((p.y - mp[3]) / mp[2]) - 0.5))
		var half := int(ceil(10.0 / float(mp[0])))
		var r := Rect2i(cx - half, cy - half, 2 * half, 2 * half).intersection(host)
		if r.size.x < 8 or r.size.y < 8:
			continue
		var sub := real.get_region(r)
		sub.resize(sub.get_width() * 8, sub.get_height() * 8, Image.INTERPOLATE_NEAREST)
		sub.save_png("%s/%s_%s%d_%s.png" % [_out, pname.to_snake_case(), v["kind"], vi, zs])


func _deep_legs(pname: String, visits: Array) -> void:
	## Per kind: [sum of fraction * n, sum of n] over the visits.
	var acc := {1: [0.0, 0], 2: [0.0, 0]}
	for vi in visits.size():
		var v: Dictionary = visits[vi]
		var p: Vector2 = v["p"]
		_vh.reset_view()
		await _frames(3)
		_vh.zoom_step(DEEP / _vh.zoom())
		_vh.move_view_to(p.x - 0.5, p.y - 0.5)
		_no_labels()
		await _settle_tiles()
		if not _vh.lod_active():
			_ok(false, "Z %s: z%d is above the deep-zoom switch" % [pname, int(DEEP)])
			continue
		var real := await _grab_full()
		## The same view with only the water's colour changed, then back.
		## The shell's own repaint idiom (`map_view.texture =
		## bridge.color_texture()`) plus the tiles' invalidation -- not
		## `refresh()`, which also resets the view.
		_br.set_appearance({"sea_ramp_strength": 1.0})
		_vh.map_view.texture = _br.color_texture()
		_vh.invalidate_lod_tiles()
		await _settle_tiles()
		var ramp := await _grab_full()
		_br.drop_appearance_overrides(PackedStringArray(["sea_ramp_strength"]))
		_vh.map_view.texture = _br.color_texture()
		_vh.invalidate_lod_tiles()
		await _settle_tiles()
		var again := await _grab_full()
		var host := _host_rect(real)
		var mp := _mapping()
		var codes := _band_of(host, mp)
		## The noise floor: the most any pixel moved between two captures of
		## the same state (a round trip through the same appearance).
		var ctl := 0
		var W := real.get_width(); var d1 := real.get_data(); var d2 := again.get_data()
		for y in range(0, host.size.y, 2):
			for x in range(0, host.size.x, 2):
				var o := ((host.position.y + y) * W + host.position.x + x) * 3
				ctl = maxi(ctl, absi(d1[o] - d2[o]) + absi(d1[o + 1] - d2[o + 1]) + absi(d1[o + 2] - d2[o + 2]))
		var Lw := _waterness(real, ramp, host, codes, ctl)
		var line := "SHORESTEP  Z %-8s z%d visit %d (%s at %s): control %d" % [pname, int(DEEP), vi, v["kind"], p, ctl]
		if Lw.is_empty():
			line += "  (no water outline to read)"
		for k in [1, 2]:
			if Lw.is_empty():
				break
			var a := _axis_metric(Lw, host.size.x, host.size.y, codes, k)
			## A cell and a half either side: wide enough to span a low
			## staircase's longer runs.
			var r := maxi(2, int(round(1.5 / float(mp[0]))))
			var dv := _stairs(Lw, host.size.x, host.size.y, codes, k, r)
			line += "  %s stairs %.3f/%d (axis %.3f)" % ["sea" if k == 1 else "lake", dv[0], dv[1], a[0]]
			if int(dv[1]) > 0:
				acc[k][0] += float(dv[0]) * float(dv[1]); acc[k][1] += int(dv[1])
		if not Lw.is_empty():
			var wi := Image.create(host.size.x, host.size.y, false, Image.FORMAT_L8)
			var wb := PackedByteArray(); wb.resize(host.size.x * host.size.y)
			for i in wb.size():
				wb[i] = int(Lw[i])
			wi.set_data(host.size.x, host.size.y, false, Image.FORMAT_L8, wb)
			wi.save_png("%s/%s_%s%d_z%d_water.png" % [_out, pname.to_snake_case(), v["kind"], vi, int(DEEP)])
		print(line)
		var c := Vector2i(host.position + host.size / 2)
		var r := Rect2i(c - Vector2i(300, 220), Vector2i(600, 440)).intersection(host)
		real.get_region(r).save_png("%s/%s_%s%d_z%d.png" % [_out, pname.to_snake_case(), v["kind"], vi, int(DEEP)])
	for k in [1, 2]:
		var nm := "sea" if k == 1 else "lake"
		var n: int = acc[k][1]
		var f: float = acc[k][0] / float(n) if n > 0 else NAN
		print("SHORESTEP  Z %-8s z%d %-4s stair fraction %.3f over %d edge px" % [pname, int(DEEP), nm, f, n])
		_report["Z_%s_%s" % [pname, nm]] = {"stairs": f, "n": n}
		## Both kinds judged. The lake is the leg that fails on HEAD; the sea
		## is a control -- the tile's sea is unchanged code (the bilinear
		## height below sea level) that HEAD already draws smooth.
		_ok(n >= MIN_EDGE_PX, "Z %s %s: enough shoreline edge pixels to measure (%d >= %d)" % [pname, nm, n, MIN_EDGE_PX])
		if n >= MIN_EDGE_PX:
			_ok(f <= STAIR_BAR, "Z %s %s: no square steps at z%d (stair fraction %.3f <= %.2f)" % [pname, nm, int(DEEP), f, STAIR_BAR])


func _finish() -> void:
	var f := FileAccess.open(_out.path_join("shorestep.json"), FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(_report, "  "))
	print("SHORESTEP  === %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
