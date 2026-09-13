extends Control
## Lane URBANDRAW: windowed proof for round `curtain`-wall towers, plus a guard
## that NO district-based roof tint is drawn (the intramural/extramural tint built
## 2026-09-13 was reverted after verification: `building_district` is not a
## wall-containment test), plus the row's Gates (triangulation-error count).
##
## Run TWICE, byte-identical, against two DIFFERENT `res://` roots -- this
## file itself never changes between runs, only which `UrbanLayoutDraw`
## answers its calls:
##   1. against the live tree (this repo's `cartalith-native/godot-project`),
##      which HAS both new features;
##   2. against a HEAD-committed scratch copy (`git show HEAD:...` of just
##      `shell/urban_layout_draw.gd`, everything else identical), which does
##      not -- HEAD predates both features, so it doubles as the mutation
##      baseline the row's "Proof" section asks for (feature removed) AND the
##      Gates' "before" tree. World generation is untouched by this lane, so
##      `urban_layouts()`'s own output is byte-identical between the two runs;
##      only the drawing differs.
##
## Every colour/hue compared is a literal pinned independently in THIS file,
## never read off `UrbanLayoutDraw`'s own constants (MISTAKES.md: "Assert a
## drawn colour... never against `DccTheme.c(token)`... pin the canvas
## literal") -- doubly necessary here, since the HEAD run's `UrbanLayoutDraw`
## does not even define `WALL_TOWER`.
##
## Windowed only (MISTAKES.md: `ImageTexture.update()`/`frame_post_draw` are
## no-ops headless).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _towertint_probe.tscn -- --out <dir> --tag <tag>
##
## `--out` is an absolute OS directory (created if missing) both runs share so
## their PNGs can be diffed after the fact; `--tag` labels this run's files
## (e.g. "tree" / "head") and is echoed into every printed line so the two
## runs' logs are never confused for each other.

const SEED := 24601
const SIZE_KM := 96.0
const GW := 96
const GH := 64
const CANVAS := 900.0
const TIMING_REPEATS := 21
const VERTEX_STRIDE_TARGET := 8 # ~this many vertices sampled per wall, evenly spaced

## Pinned independently -- MUST equal `urban_layout_draw.gd`'s own constants,
## checked by eye against that file, not by importing it.
const LIT_WALL_TOWER := Color(0.212, 0.169, 0.114)
const LIT_ROOF_H_EXTRAMURAL := 0.058
const LIT_WALL_TOWER_R_M := 3.6
const LIT_WALL_W_CURTAIN_M := 4.5
## The offset along a vertex's outward normal this probe samples at, in model
## metres -- strictly between the plain wall stroke's own half-width
## (`LIT_WALL_W_CURTAIN_M/2 = 2.25`) and the tower disc's radius (`3.6`), so
## the sampled point can only ever be covered by a tower's fill, never by the
## bare stroke alone, regardless of any miter-join widening at the vertex
## (which -- being present identically whether or not a tower is drawn over
## it -- would bias both runs alike, not just one).
const NORMAL_OFFSET_M := 3.0

class TriLogger extends Logger:
	var count := 0
	var last_rationale := ""
	func _log_error(_function: String, _file: String, _line: int, code: String,
			rationale: String, _editor_notify: bool, _error_type: int,
			_script_backtraces: Array) -> void:
		if rationale.to_lower().contains("triangulation failed"):
			count += 1
			last_rationale = "%s (%s)" % [rationale, code]

var _fails := 0
var _tag := "run"
var _out_dir := ""
var _current_layout: Dictionary = {}
var _current_to_screen: Callable
var _current_scale := 1.0
var _last_draw_us := 0


func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  [%s] %s" % [_tag, what])
	else:
		_fails += 1
		print("  FAIL  [%s] %s" % [_tag, what])


func _close_rgb(a: Color, b: Color, eps: float = 0.03) -> bool:
	return absf(a.r - b.r) < eps and absf(a.g - b.g) < eps and absf(a.b - b.b) < eps


func _hue_close(h_have: float, h_want: float, eps: float) -> bool:
	var d := absf(fposmod(h_have - h_want + 0.5, 1.0) - 0.5)
	return d < eps


## Redraws the WHOLE layout through a tight close-up transform centred on
## `poly` (see `_building_closeup_transform`), then samples that building's own
## safe interior point. Returns a sentinel `alpha<0.5` colour if the transform
## or the sample point is unusable -- nothing this file ever draws is
## non-opaque, so that sentinel cannot collide with a real result.
func _sample_building_closeup(layout: Dictionary, poly: PackedVector2Array) -> Color:
	var xf := _building_closeup_transform(poly, Vector2(CANVAS, CANVAS))
	if xf.is_empty():
		return Color(0.0, 0.0, 0.0, 0.0)
	var scale_f: float = xf["scale"]
	var origin: Vector2 = xf["origin"]
	var to_screen2 := func(p: Vector2) -> Vector2: return origin + p * scale_f
	_current_layout = layout
	_current_to_screen = to_screen2
	_current_scale = scale_f
	queue_redraw()
	await _settle()
	var img2 := get_viewport().get_texture().get_image()
	var c_model := _safe_interior_point(poly)
	var c_screen: Vector2 = to_screen2.call(c_model)
	var px := int(round(c_screen.x))
	var py := int(round(c_screen.y))
	if px < 0 or py < 0 or px >= img2.get_width() or py >= img2.get_height():
		return Color(0.0, 0.0, 0.0, 0.0)
	return img2.get_pixel(px, py)


func _settle() -> void:
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw


## The projected screen-space size of `poly`'s bounding box (its longer side).
func _screen_extent(poly: PackedVector2Array, to_screen: Callable) -> float:
	if poly.size() < 2:
		return 0.0
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for p in poly:
		var sp: Vector2 = to_screen.call(p)
		lo = lo.min(sp)
		hi = hi.max(sp)
	return maxf(hi.x - lo.x, hi.y - lo.y)


## The LARGEST-projected building of the wanted intramural/extramural class,
## not merely the first. A roof's ink stroke is capped at `lot_px/3.0`
## (`_draw_roofs`'s own formula) and drawn CENTRED on the edge -- so it
## extends `lw/2` back into the interior too, and for a small-enough roof that
## inward half can reach clear across, swallowing even a geometrically-correct
## interior sample. `_draw_roofs`'s `want_ink` gate is a TOWN-WIDE median
## decision, so a below-median building inside an ink-on town still gets
## ink -- picking the largest available candidate is what actually avoids it,
## a per-district `min_px` floor did not (measured: the first artisan
## building in every one of 4 sampled towns read flat `CASING`, not a roof
## hue, at a 30px floor).
func _pick_building(buildings: Array, districts: PackedStringArray, want_extra: bool,
		to_screen: Callable) -> int:
	var best := -1
	var best_extent := -1.0
	for bi in buildings.size():
		var d := String(districts[bi])
		var is_extra: bool = d == "suburb" or d == "agrarian"
		if is_extra != want_extra:
			continue
		var ext := _screen_extent(buildings[bi], to_screen)
		if ext > best_extent:
			best_extent = ext
			best = bi
	return best


func _poly_centroid(p: PackedVector2Array) -> Vector2:
	var c := Vector2.ZERO
	for v in p:
		c += v
	return c / maxf(1.0, float(p.size()))


## A point GUARANTEED inside `poly` (for any simple polygon Godot's own
## triangulator accepts), unlike a bare vertex average -- which lands outside
## or right on the boundary ink for a concave footprint (`buildBuildings`'
## courtyard grammar has two wings, i.e. an L/U shape). Takes the LARGEST
## triangle of the triangulation and its centroid, so the sample also stays
## clear of a thin sliver near an edge.
func _safe_interior_point(poly: PackedVector2Array) -> Vector2:
	if poly.size() < 3:
		return Vector2.ZERO
	var idx := Geometry2D.triangulate_polygon(poly)
	var best_area := -1.0
	var best_c := Vector2.ZERO
	var t := 0
	while t + 2 < idx.size():
		var a: Vector2 = poly[idx[t]]
		var b: Vector2 = poly[idx[t + 1]]
		var c: Vector2 = poly[idx[t + 2]]
		var area := absf((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)) * 0.5
		if area > best_area:
			best_area = area
			best_c = (a + b + c) / 3.0
		t += 3
	if best_area > 0.0:
		return best_c
	return _poly_centroid(poly)


## The polygon's own outward normal at vertex `i` -- averaged from its two
## incident edge normals, sign-corrected against the direction away from
## `centroid` (the polygon need not be perfectly convex for this to be a
## reasonable "outward" estimate; every wall ring sampled below is close to
## convex by construction -- a hull-traced circuit).
func _outward_normal(ring: PackedVector2Array, i: int, centroid: Vector2) -> Vector2:
	var n := ring.size()
	var prev: Vector2 = ring[(i - 1 + n) % n]
	var cur: Vector2 = ring[i]
	var nxt: Vector2 = ring[(i + 1) % n]
	var d1: Vector2 = (cur - prev).normalized()
	var d2: Vector2 = (nxt - cur).normalized()
	var n1 := Vector2(-d1.y, d1.x)
	var n2 := Vector2(-d2.y, d2.x)
	var avg := n1 + n2
	if avg.length() < 0.0001:
		avg = n1
	avg = avg.normalized()
	if avg.dot(cur - centroid) < 0.0:
		avg = -avg
	return avg


func _fit_box_transform(box: Rect2, canvas_size: Vector2, margin: float) -> Dictionary:
	var fit: float = minf((canvas_size.x - 2.0 * margin) / maxf(1.0, box.size.x),
		(canvas_size.y - 2.0 * margin) / maxf(1.0, box.size.y))
	var origin: Vector2 = (canvas_size - box.size * fit) * 0.5 - box.position * fit
	return {"scale": fit, "origin": origin}


## `city_viewer_window.gd::_fit_box()`/`_draw_canvas()`'s own recipe,
## reproduced here rather than depended on -- that file belongs to no lane in
## this batch, but staying inside this lane's own granted files (and off
## anything another concurrent run could be mid-editing) is the safer
## default. Read-only citation, not a runtime dependency.
func _fit_transform(layout: Dictionary, canvas_size: Vector2) -> Dictionary:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	for p in ring:
		lo = lo.min(p)
		hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		for b in (layout.get("buildings", []) as Array):
			for p in (b as PackedVector2Array):
				lo = lo.min(p)
				hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		return {}
	var pad := (hi - lo) * 0.08
	var box := Rect2(lo - pad, (hi - lo) + pad * 2.0)
	return _fit_box_transform(box, canvas_size, 16.0)


## A tight close-up transform on one building's own footprint, so its roof
## fills enough of the canvas that the ink stroke (`_draw_roofs`'s own
## `lot_px/3.0` cap) is a small fraction of it rather than most of it -- see
## `_pick_building`'s doc comment for why the whole-town fit alone was not
## enough (max ~18px even for the largest building in a several-thousand-roof
## town at `CANVAS`). The WHOLE layout is still drawn through this transform
## (not just the one building) so every other pixel keeps meaning the same
## thing it does at every other zoom this file uses.
func _building_closeup_transform(poly: PackedVector2Array, canvas_size: Vector2) -> Dictionary:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for p in poly:
		lo = lo.min(p)
		hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		return {}
	var pad := (hi - lo) * 1.2 + Vector2(0.5, 0.5)
	var box := Rect2(lo - pad, (hi - lo) + pad * 2.0)
	return _fit_box_transform(box, canvas_size, 8.0)


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.761, 0.702, 0.580))
	if _current_layout.is_empty():
		return
	var t0 := Time.get_ticks_usec()
	UrbanLayoutDraw.draw_layout(self, _current_layout, _current_to_screen,
		_current_scale, 1.0, 1.0, true)
	_last_draw_us = Time.get_ticks_usec() - t0


func _median(v: Array) -> float:
	var a := v.duplicate()
	a.sort()
	var n := a.size()
	if n == 0:
		return 0.0
	return a[n / 2] if n % 2 == 1 else (a[n / 2 - 1] + a[n / 2]) / 2.0


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out_dir = args[i + 1]
		elif args[i] == "--tag" and i + 1 < args.size():
			_tag = args[i + 1]
	if _out_dir == "":
		print("URBANTINT ABORT -- no --out dir given")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out_dir)
	print("URBANTINT tag=%s out=%s" % [_tag, _out_dir])

	size = Vector2(CANVAS, CANVAS)
	position = Vector2.ZERO

	var gen := WorldGen.new()
	gen.generate_sized(SEED, SIZE_KM, GW, GH)
	var settlements: Array = gen.get_settlements()
	var idx_arr := PackedInt32Array()
	for i in mini(5, settlements.size()):
		idx_arr.append(i)
	var layouts: Array = gen.urban_layouts(idx_arr)
	print("URBANTINT [%s] %d settlements dumped" % [_tag, layouts.size()])

	var tri_logger := TriLogger.new()
	OS.add_logger(tri_logger)

	for layout_v in layouts:
		var layout: Dictionary = layout_v
		var idx: int = int(layout.get("index", -1))
		var xf := _fit_transform(layout, Vector2(CANVAS, CANVAS))
		if xf.is_empty():
			print("URBANTINT [%s] settlement #%d has no drawable extent -- skipped" % [_tag, idx])
			continue
		var scale_f: float = xf["scale"]
		var origin: Vector2 = xf["origin"]
		var to_screen := func(p: Vector2) -> Vector2: return origin + p * scale_f

		_current_layout = layout
		_current_to_screen = to_screen
		_current_scale = scale_f

		var times: Array = []
		for rep in TIMING_REPEATS:
			queue_redraw()
			await _settle()
			times.append(_last_draw_us)
		# Drop the first sample (JIT/cache warmup), same convention as any
		# other repeated-call timing in this codebase.
		var warm: Array = times.slice(1)
		warm.sort()
		var med := _median(warm)
		print("URBANTINT [%s] settlement #%d draw_layout us: median=%.0f min=%.0f max=%.0f (n=%d)" % [
			_tag, idx, med, warm[0], warm[warm.size() - 1], warm.size()])

		var img := get_viewport().get_texture().get_image()
		var png_path := "%s/%s_s%d.png" % [_out_dir, _tag, idx]
		img.save_png(png_path)
		print("URBANTINT [%s] settlement #%d saved %s (%dx%d)" % [_tag, idx, png_path, img.get_width(), img.get_height()])

		# -- tower proof: the wall's own vertices, sampled along the outward
		# normal at a distance only a disc (never the bare stroke) can cover --
		var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
		var style := String(layout.get("wall_style", ""))
		if ring.size() >= 3 and style == "curtain":
			var centroid := _poly_centroid(ring)
			var stride: int = maxi(1, ring.size() / VERTEX_STRIDE_TARGET)
			var checked := 0
			var tower_hits := 0
			for vi in range(0, ring.size(), stride):
				var nrm := _outward_normal(ring, vi, centroid)
				var probe_model: Vector2 = ring[vi] + nrm * NORMAL_OFFSET_M
				var probe_screen: Vector2 = to_screen.call(probe_model)
				var px := int(round(probe_screen.x))
				var py := int(round(probe_screen.y))
				if px < 0 or py < 0 or px >= img.get_width() or py >= img.get_height():
					continue
				checked += 1
				var got := img.get_pixel(px, py)
				var hit := _close_rgb(got, LIT_WALL_TOWER)
				if hit:
					tower_hits += 1
				# Per-vertex, not just the aggregate: a miss is not necessarily
				# the tower failing to draw -- `draw_layout` draws roofs AFTER
				# the wall, so a building built right up against the inside of
				# the circuit can legitimately paint over a tower at that exact
				# sample point. Printing the colour lets a reader tell "roof
				# occluded it" (a roof-family hue) from "nothing painted here at
				# all" (still background/ground) without re-running anything.
				print("    vertex[%d]=(%.0f,%.0f) probe_px=(%d,%d) color=%s %s" % [
					vi, ring[vi].x, ring[vi].y, px, py, got, "HIT" if hit else "miss"])
			print("URBANTINT [%s] settlement #%d tower probe: %d/%d vertex samples match WALL_TOWER literal" % [
				_tag, idx, tower_hits, checked])
			if checked > 0:
				# A strict majority, not every sample: an occasional miss from
				# a roof painted over a tower (see the per-vertex comment
				# above) is expected and does not mean the feature is absent --
				# the discriminating comparison is this run's hit RATE against
				# the HEAD run's (which cannot hit at all: `WALL_TOWER` is not
				# a colour anything paints there without this feature).
				_ok(float(tower_hits) / float(checked) > 0.5,
					"settlement #%d: a strict MAJORITY of sampled curtain-vertex outward points are WALL_TOWER-coloured (%d/%d)" % [idx, tower_hits, checked])
		elif ring.size() >= 3:
			print("URBANTINT [%s] settlement #%d wall_style=%s -- not curtain, tower probe skipped (by design: towers are curtain-only)" % [_tag, idx, style])
		else:
			print("URBANTINT [%s] settlement #%d unwalled -- tower probe skipped" % [_tag, idx])

		# -- intramural/extramural roof-tint proof --
		var buildings: Array = layout.get("buildings", [])
		var districts: PackedStringArray = layout.get("building_district", PackedStringArray())
		var have_wall: bool = ring.size() >= 3
		var have_districts: bool = districts.size() == buildings.size()
		if have_wall and have_districts:
			var intra_i := _pick_building(buildings, districts, false, to_screen)
			var extra_i := _pick_building(buildings, districts, true, to_screen)
			if intra_i >= 0:
				print("URBANTINT [%s] settlement #%d intramural pick: building #%d, screen extent=%.1fpx" % [
					_tag, idx, intra_i, _screen_extent(buildings[intra_i], to_screen)])
			if extra_i >= 0:
				print("URBANTINT [%s] settlement #%d extramural pick: building #%d, screen extent=%.1fpx" % [
					_tag, idx, extra_i, _screen_extent(buildings[extra_i], to_screen)])
			if intra_i >= 0:
				var poly: PackedVector2Array = buildings[intra_i]
				var got: Color = await _sample_building_closeup(layout, poly)
				if got.a >= 0.5:
					var h := got.h
					print("URBANTINT [%s] settlement #%d intramural building #%d (district=%s) close-up sample color=%s hue=%.4f" % [
						_tag, idx, intra_i, districts[intra_i], got, h])
					_ok(_hue_close(h, LIT_ROOF_H_EXTRAMURAL, 0.02),
						"settlement #%d: 'intramural' building #%d roof hue ~= ROOF_H (%.3f) -- no district tint" % [idx, intra_i, LIT_ROOF_H_EXTRAMURAL])
				else:
					print("URBANTINT [%s] settlement #%d intramural sample unusable (empty transform or off-canvas)" % [_tag, idx])
			else:
				print("URBANTINT [%s] settlement #%d has no intramural building to sample" % [_tag, idx])
			if extra_i >= 0:
				var poly2: PackedVector2Array = buildings[extra_i]
				var got2: Color = await _sample_building_closeup(layout, poly2)
				if got2.a >= 0.5:
					var h2 := got2.h
					print("URBANTINT [%s] settlement #%d extramural building #%d (district=%s) close-up sample color=%s hue=%.4f" % [
						_tag, idx, extra_i, districts[extra_i], got2, h2])
					_ok(_hue_close(h2, LIT_ROOF_H_EXTRAMURAL, 0.02),
						"settlement #%d: extramural building #%d roof hue ~= ROOF_H (%.3f, UNCHANGED from before this feature)" % [idx, extra_i, LIT_ROOF_H_EXTRAMURAL])
				else:
					print("URBANTINT [%s] settlement #%d extramural sample unusable (empty transform or off-canvas)" % [_tag, idx])
			else:
				print("URBANTINT [%s] settlement #%d has no extramural (suburb/agrarian) building to sample" % [_tag, idx])
		else:
			print("URBANTINT [%s] settlement #%d: have_wall=%s have_districts=%s -- tint probe skipped" % [
				_tag, idx, have_wall, have_districts])

	OS.remove_logger(tri_logger)
	print("URBANTINT [%s] triangulation-failure count across the whole run: %d last=%s" % [
		_tag, tri_logger.count, tri_logger.last_rationale])
	_ok(tri_logger.count == 0, "0 'triangulation failed' errors across every settlement/redraw this run")

	print("URBANTINT [%s] %s (%d failed)" % [_tag, "ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
