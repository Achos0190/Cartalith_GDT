extends Node
## Committed probe for **Ruling BI** (`LARGE_ITEM_RULINGS.md`, 2026-09-27):
## the eight researched map style presets (`MAP_STYLE_RESEARCH.md` §4) plus "Tanaka relief", built
## into `render_workspace.gd`'s `STYLE_PRESETS` beside the six shipped tiles
## and "Village".
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _stylepresets_probe.tscn -- --out DIR [--tag T]
##
## **Run WINDOWED** -- this probe screenshots the viewport, which a headless
## driver never draws (`MISTAKES.md`'s "cargo test cannot see a broken shell"
## row, the GDScript-side version of it).
##
## For every `STYLE_PRESETS` entry, in order:
##   1. applies it the same way a real click does -- `tiles[i]["button"]`'s own
##      `pressed` signal, not a direct `_apply_preset()` call, so this probe
##      exercises the exact path a user's finger does;
##   2. reads the live look/NPR/appearance back from the bridge and asserts
##      each equals what the preset's own bundle says it should be;
##   3. screenshots the viewport and asserts the frame differs from the
##      previous preset's by more than a stated pixel-fraction threshold;
##   4. saves the screenshot.
## Then builds one contact-sheet PNG of every preset in `STYLE_PRESETS` order.
##
## **Cel / Toon section** (2026-09-27, `render.rs`'s `toon_strength` /
## `toon_outline`), always run after the loop, or alone with `--cel-only`:
##   5. the Cel tile's bundle line names its toon keys (it read "No Painter
##      styles -- the quality tier's own image" before, which is false for it);
##   6. **mottle**: the median 3x3 luma standard deviation of the raw map
##      texture over flat land (land cells whose `slope_n` is in the lowest
##      quarter, windows wholly on land) must be under half of Default's --
##      protects the flat-albedo half of the style;
##   7. **bands**: with `bio_blend` and `relief_chroma` forced to 0 (the land
##      is then `185 * light`, grey) and the keyline and river symbol off (ink,
##      not light), vignette-corrected land luma must put at least 85% of land
##      pixels within +-3 levels of its four most common values, the steepest
##      land row must cross 2..4 levels, and the deep-zoom tile over the
##      steepest cell must show 2..4 -- protects the banded-light half; Default
##      measured the same way is the control and must score lower / show more;
##   8. saves `cel_*`/`default_*` raw textures, the flat-land/water mask and a
##      fit and a zoomed screenshot of each, for the before/after sheet.
## `--grid WxH` generates at that size instead of 384x288 (the app's own
## working size is 1024x656).

var app: Node
var bridge
var rw: Node
var _fail := 0
var _out := ""
var _tag := "run"
## OUTSTANDING_WORK.md "Style preset follow-ups", Nautical's offshore look:
## `--focus auto` finds an open-sea grid cell (coarse-scanned, `ocean`
## classified and far from any non-ocean sample) rather than whatever the
## default cover view happens to land on, which can be mostly land or a
## small lake -- `render.rs`'s `sea_ramp_strength` only tints true ocean.
var _focus_auto := false
var _fzoom := 3.0
## `--cel-only`: skip the per-preset loop and the contact sheet, run only the
## Cel / Toon section (the loop is ~16 windowed re-renders).
var _cel_only := false
## `--grid WxH`: the generated world's grid. 384x288 unless given.
var _grid := Vector2i(384, 288)

const KNOWN_FLAGS := ["--out", "--tag", "--focus", "--fzoom", "--cel-only", "--grid"]
## Fraction of RGB bytes that must differ by more than 2 levels between two
## consecutive presets' screenshots for them to count as visibly distinct --
## the same threshold shape `render_serial`'s Rust-side `moved()` uses in
## `ruling_bi_style_presets.rs`, chosen loosely (a whole-viewport screenshot
## carries UI chrome the render tests don't) rather than tuned to this image.
const DIFF_THRESHOLD := 0.01
const DIFF_TOL := 2


func _p(s: String) -> void:
	print("STYLEPRESETS  %s" % s)


func _ok(cond: bool, s: String) -> void:
	if cond:
		print("STYLEPRESETS  ok    %s" % s)
	else:
		_fail += 1
		print("STYLEPRESETS  FAIL  %s" % s)


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
	for i in args.size():
		var a := String(args[i])
		if not a.begins_with("--"):
			continue
		var known := false
		for f in KNOWN_FLAGS:
			if a == f or a.begins_with(f + "="):
				known = true
		if not known:
			print("STYLEPRESETS  UNKNOWN ARGUMENT %s -- known: %s" % [a, str(KNOWN_FLAGS)])
			return false
	for i in args.size():
		if String(args[i]) == "--out" and i + 1 < args.size():
			_out = String(args[i + 1])
		if String(args[i]) == "--tag" and i + 1 < args.size():
			_tag = String(args[i + 1])
		if String(args[i]) == "--focus" and i + 1 < args.size() and String(args[i + 1]) == "auto":
			_focus_auto = true
		if String(args[i]) == "--fzoom" and i + 1 < args.size():
			_fzoom = float(args[i + 1])
		if String(args[i]) == "--cel-only":
			_cel_only = true
		if String(args[i]) == "--grid" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			if wh.size() != 2 or int(wh[0]) < 16 or int(wh[1]) < 16:
				print("STYLEPRESETS  BAD --grid %s -- want WxH, e.g. 1024x656" % String(args[i + 1]))
				return false
			_grid = Vector2i(int(wh[0]), int(wh[1]))
	if _out == "":
		print("STYLEPRESETS  NO --out DIR given.")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


## `--focus auto`: a coarse grid scan (`bridge.sample_cell()`'s own "water"
## key -- "land"/"ocean"/"lake") that picks the sampled ocean cell farthest
## (Chebyshev, in scan steps) from the nearest sampled non-ocean cell --
## deep water, not a coastal pixel or a lake that only looks like one at a
## glance. `Vector2(-1, -1)` if the coarse scan finds no ocean cell at all.
func _find_open_sea() -> Vector2:
	var gw := _grid.x
	var gh := _grid.y
	var step := 8
	## A margin off every edge: a corner/edge ocean cell zooms the camera
	## in on the map texture's own boundary, showing blank canvas past it
	## rather than open water -- measured on this machine (first attempt
	## landed on `(376, 0)`, one step from the grid's own edge, and the
	## saved screenshot was blank).
	var margin := 40
	var pts: Array = []       # [Vector2i, is_ocean]
	for gy in range(margin, gh - margin, step):
		for gx in range(margin, gw - margin, step):
			var s: Dictionary = bridge.sample_cell(gx, gy)
			pts.append([Vector2i(gx, gy), String(s.get("water", "land")) == "ocean"])
	var best := Vector2(-1, -1)
	var best_d := -1
	for p in pts:
		if not bool(p[1]):
			continue
		var pos: Vector2i = p[0]
		var d := 1 << 30
		for q in pts:
			if bool(q[1]):
				continue
			var qpos: Vector2i = q[0]
			var dd := maxi(absi(pos.x - qpos.x), absi(pos.y - qpos.y))
			if dd < d:
				d = dd
		if d > best_d:
			best_d = d
			best = Vector2(pos.x, pos.y)
	_p("open-sea scan: best=%s, chebyshev dist to nearest non-ocean sample=%d steps" % [best, best_d])
	return best


## Pans/zooms the live camera onto grid cell `sea` -- `move_view_to()` alone
## is not enough: `_zoom_at()`/`reset_view()` both write `_camera.scale`
## alongside `_zoom`, and `move_view_to()`'s own position math is computed
## FROM `_zoom`. Writing `_zoom` without `_camera.scale` leaves the visible
## scale at whatever `reset_view()` left it while the computed position
## assumes the new one -- measured on this machine: a blank saved screenshot,
## the camera parked off every rendered pixel.
func _apply_focus(sea: Vector2) -> void:
	var vp = app.viewport
	vp._zoom = _fzoom
	vp._camera.scale = Vector2(_fzoom, _fzoom)
	vp.move_view_to(sea.x, sea.y)


func _generate() -> void:
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


func _shot() -> Image:
	await get_tree().create_timer(0.5).timeout
	await _frames(4)
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


## Fraction of RGB bytes differing by more than `DIFF_TOL` levels. `a`/`b`
## must be the same format and size; a size mismatch (a resized window
## between shots) is reported as "moved everything" rather than crashing.
func _pixel_diff_frac(a: Image, b: Image) -> float:
	if a.get_size() != b.get_size() or a.get_format() != b.get_format():
		return 1.0
	var da := a.get_data()
	var db := b.get_data()
	var n := mini(da.size(), db.size())
	if n == 0:
		return 1.0
	var d := 0
	for i in n:
		if absi(int(da[i]) - int(db[i])) > DIFF_TOL:
			d += 1
	return float(d) / float(n)


## Every key `values` (the full `STYLE_MANAGED` bundle merged with the
## preset's own overrides) names, checked against `live` (the bridge
## readback) with a small float tolerance -- `set_npr`/`set_appearance` clamp
## to their own declared ranges, and every value used here is already inside
## them, so an exact-ish match (not bitwise) is the right bar.
func _assert_dict_matches(live: Dictionary, want: Dictionary, ctx: String) -> void:
	for key in want:
		if not live.has(key):
			_ok(false, "%s: live dict has no '%s' key" % [ctx, key])
			continue
		var lv = live[key]
		var wv = want[key]
		if typeof(wv) == TYPE_BOOL or typeof(lv) == TYPE_BOOL:
			_ok(bool(lv) == bool(wv), "%s: %s live=%s want=%s" % [ctx, key, str(lv), str(wv)])
		else:
			_ok(absf(float(lv) - float(wv)) < 0.001, "%s: %s live=%f want=%f" % [ctx, key, float(lv), float(wv)])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG TIMEOUT"); get_tree().quit(3))
	wd.start()

	if not _parse_args():
		get_tree().quit(2)
		return

	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	bridge = app.bridge

	if DisplayServer.get_name() == "headless":
		_ok(false, "run WINDOWED -- see this probe's header")
		get_tree().quit(1)
		return

	await _generate()
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var _sea := Vector2(-1, -1)
	if _focus_auto:
		_sea = _find_open_sea()
		if _sea.x < 0.0:
			_p("--focus auto found no ocean cell in the coarse scan -- leaving the default view")
	await _frames(8)

	app.select_domain_category("cartography", "Style")
	await _frames(8)
	rw = _find(app, "render_workspace.gd")
	_ok(rw != null, "the RenderWorkspace node is reachable")
	if rw == null:
		get_tree().quit(1)
		return

	var tiles: Array = rw.get("_preset_tiles")
	_ok(tiles.size() == RenderWorkspace.STYLE_PRESETS.size(), "one tile per STYLE_PRESETS entry")

	var prev_img: Image = null
	var prev_name := ""
	var shots: Array = []  # [[name, Image]]

	## `--cel-only` runs none of the loop -- and so builds no contact sheet,
	## whose guard is `shots` being non-empty.
	var n_loop := 0 if _cel_only else RenderWorkspace.STYLE_PRESETS.size()
	for i in n_loop:
		var entry: Array = RenderWorkspace.STYLE_PRESETS[i]
		var name := String(entry[0])
		var btn: Button = tiles[i]["button"]
		## §1: through the real UI path -- the tile's own `pressed` signal,
		## which is exactly what `_preset_tile`'s `btn.pressed.connect(...)`
		## wires to a click.
		btn.emit_signal("pressed")
		await _frames(4)
		await get_tree().create_timer(0.3).timeout
		## `_apply_preset()`'s own `_refresh_map()` calls `ViewportHost.
		## refresh()`, which calls `reset_view()` -- so every preset click
		## snaps the camera back to the default cover fit, undoing `--focus
		## auto`'s pan from before the loop. Re-applied every iteration, not
		## once, for that reason.
		if _sea.x >= 0.0:
			_apply_focus(_sea)
			await _frames(4)

		## §2: live knobs vs. the preset's own definition.
		_ok(bridge.look() == String(entry[1]), "%s: live look == '%s'" % [name, String(entry[1])])
		var want_npr: Dictionary = RenderWorkspace.STYLE_MANAGED.duplicate()
		for key in Dictionary(entry[2]):
			want_npr[key] = entry[2][key]
		_assert_dict_matches(bridge.npr_settings(), want_npr, "%s NPR" % name)
		if entry.size() > 3:
			_assert_dict_matches(bridge.appearance(), Dictionary(entry[3]), "%s appearance" % name)
		if entry.size() > 4:
			var stops: Array = bridge.color_ramp()
			_ok(not stops.is_empty(), "%s: ramp '%s' loaded with stops" % [name, String(entry[4])])

		## §3: the frame differs from the previous preset's.
		var img := await _shot()
		if prev_img != null:
			var frac := _pixel_diff_frac(prev_img, img)
			_ok(frac > DIFF_THRESHOLD, "%s vs %s: pixel diff %.4f > %.4f" % [name, prev_name, frac, DIFF_THRESHOLD])
		else:
			_p("%s: first preset, no predecessor to diff against" % name)

		## §4: one screenshot per preset.
		var path := "%s/%s_preset_%02d_%s.png" % [_out, _tag, i, name.to_snake_case()]
		img.save_png(path)
		_p("saved %s" % path)
		shots.append([name, img])

		prev_img = img
		prev_name = name

	## Contact sheet: every preset's screenshot, scaled down, in one grid, in
	## `STYLE_PRESETS` order (reading order left-to-right, top-to-bottom).
	if not shots.is_empty():
		var cols := 4
		var rows := int(ceil(float(shots.size()) / float(cols)))
		var cell_w := 480
		var cell_h := 300
		var sheet := Image.create(cell_w * cols, cell_h * rows, false, Image.FORMAT_RGB8)
		sheet.fill(Color(0.08, 0.08, 0.09))
		for i in shots.size():
			var nm: String = shots[i][0]
			var im: Image = (shots[i][1] as Image).duplicate()
			im.resize(cell_w, cell_h - 24, Image.INTERPOLATE_BILINEAR)
			## The viewport screenshot format (typically RGBA8) does not
			## match the sheet's RGB8 -- `blit_rect` refuses a format
			## mismatch outright rather than converting, so this must.
			im.convert(sheet.get_format())
			var cx := (i % cols) * cell_w
			var cy := (i / cols) * cell_h
			sheet.blit_rect(im, Rect2i(Vector2i.ZERO, im.get_size()), Vector2i(cx, cy + 24))
			_p("contact sheet cell %d = %s" % [i, nm])
		var sheet_path := "%s/%s_contact_sheet.png" % [_out, _tag]
		## No burned-in labels: rendering glyph textures into `Image` cells
		## without a `Viewport` pass is not worth the machinery here. The
		## per-preset filenames already carry the name, and the "contact
		## sheet cell N = name" lines above give the same index -> name
		## mapping to read the sheet against.
		sheet.save_png(sheet_path)
		_p("saved contact sheet %s (%dx%d, %d cells)" % [sheet_path, sheet.get_width(), sheet.get_height(), shots.size()])

	await _cel_section(tiles)
	await _tanaka_section(tiles)

	_p("=== SUMMARY: %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## Index of the `STYLE_PRESETS` entry named `name`, or -1.
func _preset_index(name: String) -> int:
	for i in RenderWorkspace.STYLE_PRESETS.size():
		if String(RenderWorkspace.STYLE_PRESETS[i][0]) == name:
			return i
	return -1


## Presses preset tile `i` the way a click does and waits for the re-render.
func _press(tiles: Array, i: int) -> void:
	(tiles[i]["button"] as Button).emit_signal("pressed")
	await _frames(4)
	await get_tree().create_timer(0.3).timeout


## Rec.709-ish luma of one pixel, 0..255 -- the weights `render.rs`'s own
## `luma` uses, so a level here is a level there.
func _luma(c: Color) -> float:
	return (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b) * 255.0


## `render.rs`'s `vignette_at`, so the band measurement can divide the
## plate's own corner darkening back out of the grey image.
func _vignette(x: int, y: int, gw: int, gh: int) -> float:
	var vx := float(x) / float(maxi(gw, 2) - 1) - 0.5
	var vy := float(y) / float(maxi(gh, 2) - 1) - 0.5
	var d := sqrt(vx * vx + vy * vy)
	var t := clampf((d - 0.34) / (0.74 - 0.34), 0.0, 1.0)
	return 1.0 - t * t * (3.0 - 2.0 * t) * 0.42


## Median 3x3 luma standard deviation over `flat` cells whose whole window is
## land -- the mottle metric. `land`/`flat` are per-cell bools.
func _mottle(img: Image, land: PackedByteArray, flat: PackedByteArray, gw: int, gh: int) -> float:
	var sds := PackedFloat32Array()
	for y in range(1, gh - 1):
		for x in range(1, gw - 1):
			if flat[y * gw + x] == 0:
				continue
			var ok := true
			var s := 0.0
			var s2 := 0.0
			for dy in range(-1, 2):
				for dx in range(-1, 2):
					if land[(y + dy) * gw + x + dx] == 0:
						ok = false
					var l := _luma(img.get_pixel(x + dx, y + dy))
					s += l
					s2 += l * l
			if not ok:
				continue
			var m := s / 9.0
			sds.append(sqrt(maxf(s2 / 9.0 - m * m, 0.0)))
	if sds.is_empty():
		return -1.0
	sds.sort()
	return sds[sds.size() / 2]


## Fraction of land pixels whose vignette-corrected luma lies within +-3 of one
## of the image's four most common (binned-to-1) values -- 1.0 for a picture
## lit by exactly four light levels, low for a smooth hillshade.
func _band_share(img: Image, land: PackedByteArray, gw: int, gh: int) -> Array:
	var hist := PackedInt32Array()
	hist.resize(400)
	var vals := PackedInt32Array()
	for y in gh:
		for x in gw:
			if land[y * gw + x] == 0:
				continue
			var v := clampi(int(round(_luma(img.get_pixel(x, y)) / _vignette(x, y, gw, gh))), 0, 399)
			hist[v] += 1
			vals.append(v)
	## The four highest peaks, each at least 7 levels from the ones taken, so
	## one wide band cannot claim two of the four slots.
	var peaks: Array = []
	var h2 := hist.duplicate()
	for k in 4:
		var best := -1
		for v in 400:
			if best < 0 or h2[v] > h2[best]:
				best = v
		peaks.append(best)
		for v in range(maxi(0, best - 6), mini(400, best + 7)):
			h2[v] = -1
	var inside := 0
	for v in vals:
		for p in peaks:
			if absi(v - int(p)) <= 3:
				inside += 1
				break
	return [float(inside) / float(maxi(vals.size(), 1)), peaks]


## Distinct shading levels along the steepest land row segment: the row
## through the steepest land cell, +-40 cells, vignette-corrected luma of the
## grey image, runs of >= 2 cells whose neighbours differ by <= 2 levels,
## then those runs' levels merged within 4. Printed, and for Cel asserted to be
## at most `TOON_BANDS` (4): the transect need not cross every step.
func _transect_levels(img: Image, land: PackedByteArray, slope: PackedFloat32Array, gw: int, gh: int) -> Array:
	var bi := -1
	for i in gw * gh:
		var x := i % gw
		if land[i] == 1 and x > 45 and x < gw - 45 and (bi < 0 or slope[i] > slope[bi]):
			bi = i
	if bi < 0:
		return [-1, []]
	var y := bi / gw
	var seq := PackedFloat32Array()
	for x in range(bi % gw - 40, bi % gw + 41):
		if land[y * gw + x] == 1:
			seq.append(_luma(img.get_pixel(x, y)) / _vignette(x, y, gw, gh))
	var levels: Array = []
	var run := 1
	for k in range(1, seq.size()):
		if absf(seq[k] - seq[k - 1]) <= 2.0:
			run += 1
			if run == 2:
				var fresh := true
				for l in levels:
					if absf(float(l) - seq[k]) <= 4.0:
						fresh = false
				if fresh:
					levels.append(snappedf(seq[k], 0.1))
		else:
			run = 1
	return [levels.size(), levels]


## Distinct light levels on the deep-zoom tile over grid cell `(cx, cy)`, at
## the pyramid level `lod_level_for_zoom(8)` picks (about 8 screen px per
## cell). Counts luma values holding at least 0.5% of the tile's non-water
## pixels (water = blue-dominant, `b > r + 20`), merging values within 2 into
## one level. Saves the tile as `<tag>_<name>_tile_grey.png`. Returns
## `[levels, share of non-water pixels those levels hold]`; `[-1, 0.0]` when
## this build has no LOD tiles.
func _tile_levels(cx: int, cy: int, gw: int, gh: int, name: String) -> Array:
	var z: int = bridge.lod_level_for_zoom(8.0)
	var n: int = bridge.lod_tiles_per_axis(z)
	if n <= 0:
		return [-1, 0.0]
	var tex: Texture2D = bridge.lod_synthesize_tile(z, clampi(cx * n / gw, 0, n - 1), clampi(cy * n / gh, 0, n - 1))
	if tex == null:
		return [-1, 0.0]
	await _frames(2)
	var img := tex.get_image()
	img.save_png("%s/%s_%s_tile_grey.png" % [_out, _tag, name])
	var hist := PackedInt32Array()
	hist.resize(256)
	var total := 0
	for y in img.get_height():
		for x in img.get_width():
			var c := img.get_pixel(x, y)
			if c.b * 255.0 > c.r * 255.0 + 20.0:
				continue
			hist[clampi(int(round(_luma(c))), 0, 255)] += 1
			total += 1
	var levels := 0
	var last := -10
	var in_levels := 0
	for v in 256:
		if total > 0 and hist[v] >= 0.005 * total:
			if v - last > 2:
				levels += 1
			last = v
			in_levels += hist[v]
	## The share of pixels those levels hold is the control that separates the
	## two presets: a smooth hillshade spreads over ~200 values, so its few
	## >= 0.5% values hold a small share, and a count alone cannot say so
	## (measured: Default's tile counted 3 such "levels" to Cel's 4).
	return [levels, float(in_levels) / float(maxi(total, 1))]


## The Cel / Toon section -- see this file's header, items 5-8.
func _cel_section(tiles: Array) -> void:
	var ic := _preset_index("Cel / Toon")
	var idf := _preset_index("Default")
	_ok(ic >= 0, "a 'Cel / Toon' preset is in STYLE_PRESETS")
	if ic < 0 or idf < 0:
		return
	var entry: Array = RenderWorkspace.STYLE_PRESETS[ic]
	## 5. The tile names its own style.
	var blurb := String(rw.call("_bundle_line", ic))
	_p("Cel tile bundle line: %s" % blurb)
	_ok(blurb.contains("Toon light bands") and not blurb.begins_with("No Painter styles"),
		"Cel tile's bundle line names its toon keys")

	## Per-cell land / flat masks from the engine's own sample.
	var probe_img: Image = bridge.color_texture().get_image()
	var gw := probe_img.get_width()
	var gh := probe_img.get_height()
	var land := PackedByteArray()
	land.resize(gw * gh)
	var slope := PackedFloat32Array()
	slope.resize(gw * gh)
	var land_slopes := PackedFloat32Array()
	for y in gh:
		for x in gw:
			var s: Dictionary = bridge.sample_cell(x, y)
			var is_land := String(s.get("water", "")) == "land"
			land[y * gw + x] = 1 if is_land else 0
			slope[y * gw + x] = float(s.get("slope_n", 0.0))
			if is_land:
				land_slopes.append(float(s.get("slope_n", 0.0)))
	land_slopes.sort()
	_ok(land_slopes.size() > 1000, "the world has land to measure (%d cells)" % land_slopes.size())
	if land_slopes.size() <= 1000:
		return
	var flat_cut := land_slopes[land_slopes.size() / 4]
	var flat := PackedByteArray()
	flat.resize(gw * gh)
	var mask := Image.create(gw, gh, false, Image.FORMAT_RGB8)
	for i in gw * gh:
		flat[i] = 1 if (land[i] == 1 and slope[i] <= flat_cut) else 0
		mask.set_pixel(i % gw, i / gw, Color(float(land[i]), float(flat[i]), 0.0))
	mask.save_png("%s/%s_mask_land_r_flat_g.png" % [_out, _tag])
	_p("grid %dx%d, land cells %d, flat cut slope_n <= %.5f" % [gw, gh, land_slopes.size(), flat_cut])

	## A steep land cell to zoom on, for the zoomed screenshots.
	var steep := -1
	for i in gw * gh:
		if land[i] == 1 and (steep < 0 or slope[i] > slope[steep]):
			steep = i
	var results := {}
	for pair in [["default", idf], ["cel", ic]]:
		var tag := String(pair[0])
		await _press(tiles, int(pair[1]))
		var tex: Image = bridge.color_texture().get_image()
		tex.save_png("%s/%s_%s_texture.png" % [_out, _tag, tag])
		var fit := await _shot()
		fit.save_png("%s/%s_%s_fit.png" % [_out, _tag, tag])
		_apply_focus(Vector2(steep % gw, steep / gw))
		await _frames(6)
		await get_tree().create_timer(1.5).timeout
		var zoom := await _shot()
		zoom.save_png("%s/%s_%s_zoom.png" % [_out, _tag, tag])
		var mot := _mottle(tex, land, flat, gw, gh)
		## 7's grey image: the same preset with the colour taken out of the light.
		## The keyline and the river symbol are dark ink, not light: both off
		## here, or the four "most common values" would include the ink.
		bridge.set_appearance({"bio_blend": 0.0, "relief_chroma": 0.0, "toon_outline": 0.0, "river_opacity": 0.0})
		var grey: Image = bridge.color_texture().get_image()
		grey.save_png("%s/%s_%s_grey.png" % [_out, _tag, tag])
		var bs: Array = _band_share(grey, land, gw, gh)
		var tr: Array = _transect_levels(grey, land, slope, gw, gh)
		## The same grey light on the deep-zoom tile over the steepest cell --
		## the tile path synthesizes sub-cell relief the grid does not have, so
		## this is where more than the flat step and one shadow step can show.
		var tl: Array = await _tile_levels(steep % gw, steep / gw, gw, gh, tag)
		results[tag] = [mot, bs[0], tr[0], tl[0], tl[1]]
		_p("%s: mottle (median 3x3 luma sd, flat land) = %.3f; band share = %.4f at peaks %s; steepest-row transect levels = %d %s; deep-zoom tile light levels = %d holding %.4f of its land"
			% [tag, mot, float(bs[0]), str(bs[1]), int(tr[0]), str(tr[1]), int(tl[0]), float(tl[1])])
	## 6. / 7.
	_ok(float(results["cel"][0]) >= 0.0 and float(results["cel"][0]) < 0.5 * float(results["default"][0]),
		"Cel mottle %.3f < half of Default's %.3f" % [float(results["cel"][0]), float(results["default"][0])])
	_ok(float(results["cel"][1]) >= 0.85, "Cel band share %.4f >= 0.85" % float(results["cel"][1]))
	_ok(float(results["cel"][1]) > float(results["default"][1]),
		"Cel band share %.4f > Default's %.4f (the control)" % [float(results["cel"][1]), float(results["default"][1])])
	_ok(int(results["cel"][2]) >= 2 and int(results["cel"][2]) <= 4,
		"Cel steepest-row transect crosses 2..4 light levels (got %d)" % int(results["cel"][2]))
	## `-1` is "this build has no LOD tiles", reported rather than passed.
	_ok(int(results["cel"][3]) >= 2 and int(results["cel"][3]) <= 4,
		"Cel deep-zoom tile shows 2..4 light levels (got %d)" % int(results["cel"][3]))
	_ok(float(results["cel"][4]) >= 0.85 and float(results["default"][4]) < float(results["cel"][4]),
		"Cel's tile levels hold %.4f of its land (>= 0.85); Default's hold %.4f (the control, lower)"
		% [float(results["cel"][4]), float(results["default"][4])])
	## Leave the map as the Cel preset draws it, not the grey measurement.
	await _press(tiles, ic)


## The "Tanaka relief" preset (Ruling BI's illuminated contours as a style).
##
## Protects: the preset exists, really turns the Tanaka mode on (`tanaka` > 0
## AND `contours` > 0 -- Tanaka is a mode of the contour pass and is inert at
## `contours` 0, so either alone would be a preset that draws nothing new),
## names it on its tile and states its deep-zoom limit in the tooltip, and
## selecting it moves the picture. The thresholds are literals (0), not
## read back from the table, so a preset edited to 0 fails here rather than
## agreeing with itself.
##
## Three renders of the same world, raw colour texture (no UI chrome):
##   Default; "Tanaka relief"; and "Tanaka relief" with `tanaka` forced to 0.
## Tanaka relief differs from Default (the preset as a whole moves the map),
## and from itself with the mode off (the lit/shadow ink, not just the paper
## and plain contours, is what is on screen) -- the latter is the positive
## control that the `tanaka` key is wired to a visible effect through the shell.
func _tanaka_section(tiles: Array) -> void:
	_p("--- Tanaka relief ---")
	var it := _preset_index("Tanaka relief")
	var idf := _preset_index("Default")
	_ok(it >= 0 and idf >= 0, "STYLE_PRESETS has 'Tanaka relief' (%d) and 'Default' (%d)" % [it, idf])
	if it < 0 or idf < 0:
		return
	var over: Dictionary = RenderWorkspace.STYLE_PRESETS[it][2]
	_ok(float(over.get("tanaka", 0.0)) > 0.0, "the table sets tanaka > 0 (%s)" % str(over.get("tanaka", "absent")))
	_ok(float(over.get("contours", 0.0)) > 0.0, "the table sets contours > 0 (%s)" % str(over.get("contours", "absent")))
	## The tile: the bundle line names Tanaka and the tooltip states the limit.
	var bundle := String((tiles[it]["bundle"] as Label).text)
	_ok(bundle.contains("Tanaka lighting"), "tile bundle line names Tanaka lighting: '%s'" % bundle)
	var tip := String((tiles[it]["button"] as Button).tooltip_text)
	_ok(tip.contains("Best zoomed in"), "tile tooltip carries the deep-zoom limit: '%s'" % tip)
	_ok(not String((tiles[idf]["button"] as Button).tooltip_text).contains("Best zoomed in"),
		"Default's tooltip has no such limit (the tip is per preset)")

	await _press(tiles, idf)
	var base: Image = bridge.color_texture().get_image()
	await _press(tiles, it)
	var live: Dictionary = bridge.npr_settings()
	_ok(float(live.get("tanaka", 0.0)) > 0.0, "live bridge tanaka > 0 after the press (%s)" % str(live.get("tanaka", "absent")))
	_ok(float(live.get("contours", 0.0)) > 0.0, "live bridge contours > 0 after the press (%s)" % str(live.get("contours", "absent")))
	var tan: Image = bridge.color_texture().get_image()
	tan.save_png("%s/%s_tanaka_texture.png" % [_out, _tag])
	var vs_default := _pixel_diff_frac(base, tan)
	_ok(vs_default > 0.05, "Tanaka relief vs Default raw texture: diff fraction %.4f > 0.05" % vs_default)

	## Positive control: the same preset with only the mode switched off.
	var off: Dictionary = live.duplicate()
	off["tanaka"] = 0.0
	bridge.set_npr(off)
	await _frames(4)
	await get_tree().create_timer(0.3).timeout
	var plain: Image = bridge.color_texture().get_image()
	var vs_plain := _pixel_diff_frac(plain, tan)
	_ok(vs_plain > 0.002, "Tanaka on vs the same preset with tanaka 0: diff fraction %.4f > 0.002" % vs_plain)
	## Leave the map as the preset draws it.
	await _press(tiles, it)
