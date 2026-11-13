extends Node
## CITY_MAP_LOOK_SCOPE.md evidence harness (2026-11-13): the same generated city drawn twice.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _citylook_probe.tscn -- --out <dir> [--vp 1600x1000] [--n 2]
##
## Windowed, never --headless (it reads rendered pixels). Asserts nothing; exit 0.
##
## **BEFORE** is the shipped City Viewer, unmodified (`app.open_city_viewer`, the canvas cropped out of the
## shell's frame). **AFTER** is a SCRATCH RESTYLE and is not shipped code: this file's own `_after_*`
## functions redraw the SAME `urban_layouts()` dictionary (every polygon, street, ring, tree and field is the
## engine's; nothing is invented) in the palette and primitives of the owner's reference (a WorldAnvil city
## map). The only invented content is the pin set, which is flagged below: the market, the place of worship,
## the citadel and the wells come from layout keys; the tavern pins are placed on the buildings nearest the
## market and the anchor pin on the water edge, as illustrations of where a kind->pin mapping would put them.
## Every colour here is a labelled judgement read off the reference screenshot, not a measurement.

const DRAW := preload("res://shell/urban_layout_draw.gd")

var _out := ""
var _vp: SubViewport
var _app: Node
var _font: Font

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var want := 2
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--vp":
				var wh := args[i + 1].split("x")
				vp_size = Vector2i(int(wh[0]), int(wh[1]))
			"--out":
				_out = args[i + 1]
			"--n":
				want = int(args[i + 1])
		i += 2
	DirAccess.make_dir_recursive_absolute(_out)
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	var sf := SystemFont.new()
	sf.font_names = PackedStringArray(["Georgia", "Palatino Linotype", "Times New Roman", "serif"])
	sf.font_weight = 700
	_font = sf

	# Score every settlement's layout for how much of the reference's vocabulary it has.
	var places: Array = bridge.settlements()
	var scored: Array = []
	for k in places.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if got.is_empty():
			continue
		var l: Dictionary = got[0]
		var ring: PackedVector2Array = l.get("wall_ring", PackedVector2Array())
		if ring.size() < 6 or String(l.get("wall_style", "")) != "curtain":
			continue
		var score := float((l.get("buildings", []) as Array).size()) * 0.3 \
			+ float((l.get("farmland", []) as Array).size()) * 3.0 \
			+ (1500.0 if (l.get("water_poly", PackedVector2Array()) as PackedVector2Array).size() >= 3 else 0.0) \
			+ float(mini(200, (l.get("detail_kind", PackedStringArray()) as PackedStringArray).size())) * 4.0
		scored.append([score, k])
	scored.sort()
	scored.reverse()
	print("CITYLOOK %d walled curtain towns of %d" % [scored.size(), places.size()])
	for n in mini(want, scored.size()):
		var k: int = scored[n][1]
		var s: Dictionary = places[k]
		print("CITYLOOK town #%d %s pop=%d" % [k, String(s.get("name", "?")), int(s.get("population", 0))])
		_app.open_city_viewer(k)
		await _frames(12)
		var cv = _app.city_viewer_window
		cv.size = Vector2i(1560, 960)
		cv.position = Vector2i(20, 20)
		await _frames(8)
		cv._reset_view()
		await _frames(6)
		## The canvas's rectangle in the SubViewport's own coordinates (an embedded window's content
		## is offset by its title bar, which `get_global_rect()` alone does not carry).
		var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
		var before := await _shot("before_%d" % n, rect)
		# AFTER: an opaque overlay over exactly the canvas, drawing the same layout restyled.
		var ov := _After.new()
		ov.host = self
		ov.layout = cv._layout
		ov.title = String(s.get("name", "Town"))
		var box: Rect2 = cv._fit_box()
		var sz: Vector2 = cv._canvas.size
		var fit: float = minf((sz.x - 32.0) / maxf(1.0, box.size.x), (sz.y - 32.0) / maxf(1.0, box.size.y))
		var vs: float = fit * cv._zoom
		ov.view_scale = vs
		ov.origin = (sz - box.size * vs) * 0.5 - box.position * vs + cv._pan
		## A child of the viewer's own canvas, because an embedded Window draws over its parent
		## viewport's ordinary nodes: an overlay anywhere else is hidden under it.
		ov.mouse_filter = Control.MOUSE_FILTER_IGNORE
		ov.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		cv._canvas.add_child(ov)
		await _frames(4)
		await _shot("after_%d" % n, rect)
		ov.queue_free()
		await _frames(2)
	print("CITYLOOK done")
	get_tree().quit(0)

func _shot(name: String, rect: Rect2) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	var crop := img.get_region(Rect2i(rect))
	var path := _out.path_join(name + ".png")
	crop.save_png(path)
	print("CITYLOOK shot %s %dx%d" % [path, crop.get_width(), crop.get_height()])
	return crop


## The restyled drawing. A Control so it can `_draw()` straight into the shell's SubViewport.
class _After extends Control:
	var host
	var layout: Dictionary
	var title := ""
	var view_scale := 1.0
	var origin := Vector2.ZERO

	# Reference palette, read off the screenshot by eye (labelled judgements).
	const GRASS := Color(0.36, 0.47, 0.20)
	const GRASS_DARK := Color(0.27, 0.38, 0.14)
	const DIRT := Color(0.80, 0.74, 0.55)
	const STREET := Color(0.84, 0.78, 0.60)
	const ROOF := Color(0.27, 0.16, 0.10)
	const ROOF_LIGHT := Color(0.52, 0.34, 0.24)
	const INK := Color(0.10, 0.07, 0.05)
	const STONE := Color(0.58, 0.55, 0.50)
	const STONE_DARK := Color(0.22, 0.20, 0.18)
	const SEA := Color(0.06, 0.17, 0.36)
	const SEA_SHORE := Color(0.20, 0.42, 0.55)
	const CANOPY := Color(0.24, 0.48, 0.14)
	const CANOPY_HI := Color(0.40, 0.62, 0.22)
	const CANOPY_RIM := Color(0.08, 0.17, 0.06)
	const PIN_ORANGE := Color(0.96, 0.58, 0.08)
	const PIN_WHITE := Color(0.93, 0.93, 0.90)

	func _p(m: Vector2) -> Vector2:
		return origin + m * view_scale

	func _poly(a: PackedVector2Array) -> PackedVector2Array:
		var o := PackedVector2Array()
		for p in a:
			o.append(_p(p))
		return o

	func _fill(pts: PackedVector2Array, c: Color) -> void:
		if pts.size() >= 3 and not Geometry2D.triangulate_polygon(pts).is_empty():
			draw_colored_polygon(pts, c)

	func _draw() -> void:
		var vs := view_scale
		draw_rect(Rect2(Vector2.ZERO, size), GRASS)
		# Grass mottling: deterministic blotches so the ground is not flat.
		var rng := RandomNumberGenerator.new()
		rng.seed = 7
		for n in 260:
			var c := Vector2(rng.randf() * size.x, rng.randf() * size.y)
			draw_circle(c, rng.randf_range(10.0, 34.0), Color(GRASS_DARK.r, GRASS_DARK.g, GRASS_DARK.b, 0.10))

		# Water: deep blue with a lighter shore band (the stroke is the "shoreline").
		var water: PackedVector2Array = layout.get("water_poly", PackedVector2Array())
		if water.size() >= 3:
			var wp := _poly(water)
			var wl := wp.duplicate()
			wl.append(wl[0])
			draw_polyline(wl, SEA_SHORE, maxf(10.0, 7.0 * vs), true)
			draw_polyline(wl, Color(0.86, 0.80, 0.62), maxf(4.0, 2.5 * vs), true)
			_fill(wp, SEA)
		var river: PackedVector2Array = layout.get("river", PackedVector2Array())
		if water.size() < 3 and river.size() >= 2:
			draw_polyline(_poly(river), SEA, maxf(4.0, float(layout.get("river_w", 20.0)) * vs), true)

		# Fields outside the walls: green strip, then furrow rows along each field's longest edge.
		for f in layout.get("farmland", []):
			var fp: PackedVector2Array = _poly(f)
			if fp.size() < 3:
				continue
			_fill(fp, Color(0.44, 0.50, 0.22))
			var best := 0.0
			var dir := Vector2.RIGHT
			for k in fp.size():
				var e := fp[(k + 1) % fp.size()] - fp[k]
				if e.length() > best:
					best = e.length()
					dir = e.normalized()
			var nrm := Vector2(-dir.y, dir.x)
			var lo := INF
			var hi := -INF
			for p in fp:
				lo = minf(lo, p.dot(nrm))
				hi = maxf(hi, p.dot(nrm))
			var step := 5.0
			var t := lo + step * 0.5
			var alt := false
			while t < hi:
				var a: Vector2 = nrm * t - dir * 2000.0
				var b: Vector2 = nrm * t + dir * 2000.0
				for seg in Geometry2D.intersect_polyline_with_polygon(PackedVector2Array([a, b]), fp):
					draw_polyline(seg, Color(0.16, 0.27, 0.09, 0.85) if alt else Color(0.58, 0.64, 0.30, 0.8), 1.6, true)
				alt = not alt
				t += step
			var ol := fp.duplicate()
			ol.append(ol[0])
			draw_polyline(ol, Color(INK.r, INK.g, INK.b, 0.35), 1.0, true)

		# Dirt inside the wall (and the blocks), the reference's tan.
		var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
		if ring.size() >= 3:
			_fill(_poly(ring), DIRT)
		for blk in layout.get("blocks", []):
			_fill(_poly(blk), DIRT)

		# Streets: tan with a faint darker edge.
		var streets: Dictionary = layout.get("streets", {})
		for pass_case in [true, false]:
			for cls in ["tertiary", "secondary", "primary"]:
				var segs: PackedVector2Array = streets.get(cls, PackedVector2Array())
				if segs.size() < 2:
					continue
				var w := (4.0 if cls == "tertiary" else 6.0 if cls == "secondary" else 8.0) * vs
				w = maxf(w * 1.5, 3.0)
				if pass_case:
					draw_multiline(_poly(segs), Color(0.55, 0.47, 0.30, 0.9), w + 2.0)
				else:
					draw_multiline(_poly(segs), STREET, w)

		# Plaza: paler paving.
		var plaza: PackedVector2Array = layout.get("plaza", PackedVector2Array())
		if plaza.size() > 2:
			_fill(_poly(plaza), Color(0.86, 0.81, 0.64))

		# Trees under roofs: outlined clumps, rim then body then a highlight.
		var kinds: PackedStringArray = layout.get("detail_kind", PackedStringArray())
		var geoms: Array = layout.get("detail_geom", [])
		var radii: PackedFloat32Array = layout.get("detail_r", PackedFloat32Array())
		var flags: PackedByteArray = layout.get("detail_flags", PackedByteArray())
		var tc := PackedVector2Array()
		var tr := PackedFloat32Array()
		for n in mini(kinds.size(), geoms.size()):
			if kinds[n] != "tree" or (geoms[n] as PackedVector2Array).is_empty():
				continue
			var fl: int = flags[n] if n < flags.size() else 0
			var rr: float = radii[n] if (fl & 1) != 0 and n < radii.size() else 2.0
			tc.append(_p((geoms[n] as PackedVector2Array)[0]))
			tr.append(maxf(3.5, rr * vs * 1.6))
		for n in tc.size():
			draw_circle(tc[n] + Vector2(2, 3), tr[n] + 2.0, Color(0, 0, 0, 0.25))
		for n in tc.size():
			draw_circle(tc[n], tr[n] + 1.8, CANOPY_RIM)
		for n in tc.size():
			draw_circle(tc[n], tr[n], CANOPY)
			draw_circle(tc[n] + Vector2(-tr[n] * 0.25, -tr[n] * 0.3), tr[n] * 0.55, CANOPY_HI)

		# Roofs: dark pitched brown, drop shadow, hatched slope lines parallel to the ridge, outline.
		var bld: Array = layout.get("buildings", [])
		var tones: PackedFloat32Array = layout.get("building_tone", PackedFloat32Array())
		var ridge: PackedVector2Array = layout.get("building_ridge", PackedVector2Array())
		for b in bld:
			var q := _poly(b)
			if q.size() < 3:
				continue
			var sh := PackedVector2Array()
			for p in q:
				sh.append(p + Vector2(1.6, 2.2))
			_fill(sh, Color(0, 0, 0, 0.32))
		for n in bld.size():
			var q := _poly(bld[n])
			var tone: float = tones[n] if n < tones.size() else 0.5
			var c := ROOF.lerp(ROOF_LIGHT, clampf(tone - 0.35, 0.0, 0.3))
			_fill(q, c)
		var ink_edges := PackedVector2Array()
		for b in bld:
			var q := _poly(b)
			for k in q.size():
				ink_edges.append(q[k])
				ink_edges.append(q[(k + 1) % q.size()])
		draw_multiline(ink_edges, INK, maxf(1.0, 0.45 * vs))
		for k in range(0, ridge.size() - 1, 2):
			var a := _p(ridge[k])
			var b := _p(ridge[k + 1])
			var d := b - a
			if d.length() < 1.0:
				continue
			var nrm := Vector2(-d.y, d.x).normalized()
			# Light slope lines each side of the ridge: the hand-drawn shingle read.
			for o in [-3.5, -1.8, 1.8, 3.5]:
				draw_line(a + nrm * o, b + nrm * o, Color(ROOF_LIGHT.r, ROOF_LIGHT.g, ROOF_LIGHT.b, 0.7), 1.0)
			draw_line(a, b, Color(0.72, 0.52, 0.36), 1.6)

		# Wall: stone band with dark outline, a crenel dash on top, round towers, gate pavings.
		if ring.size() >= 3:
			var rp := _poly(ring)
			rp.append(rp[0])
			draw_polyline(rp, STONE_DARK, maxf(7.0, 5.5 * vs), true)
			draw_polyline(rp, STONE, maxf(4.5, 3.6 * vs), true)
			var tw := maxf(4.0, 4.2 * vs)
			for v in ring:
				var c := _p(v)
				draw_circle(c, tw + 2.0, STONE_DARK)
				draw_circle(c, tw, STONE)
				draw_circle(c, tw * 0.45, Color(0.38, 0.36, 0.33))
			for g in layout.get("wall_gates", PackedVector2Array()):
				var c := _p(g)
				var hw := maxf(6.0, 4.0 * vs)
				draw_rect(Rect2(c - Vector2(hw, hw), Vector2(hw, hw) * 2.0), Color(0.62, 0.60, 0.57))
				draw_rect(Rect2(c - Vector2(hw, hw), Vector2(hw, hw) * 2.0), STONE_DARK, false, 1.4)

		# Props: wells become the reference's fountain/well (stone ring round blue water).
		var wells: Array = []
		for n in mini(kinds.size(), geoms.size()):
			if kinds[n] == "well" and not (geoms[n] as PackedVector2Array).is_empty():
				wells.append(_p((geoms[n] as PackedVector2Array)[0]))
		for w in wells:
			draw_circle(w, 7.0, STONE_DARK)
			draw_circle(w, 5.6, Color(0.66, 0.63, 0.58))
			draw_circle(w, 3.6, Color(0.20, 0.48, 0.80))
		var market: Vector2 = _p(layout.get("market", Vector2.ZERO))
		draw_circle(market, 12.0, STONE_DARK)
		draw_circle(market, 10.5, Color(0.70, 0.67, 0.61))
		draw_circle(market, 7.5, Color(0.20, 0.50, 0.85))
		draw_circle(market, 3.0, Color(0.85, 0.93, 1.0))

		# Title: serif, stroked, top-left, as the reference prints it.
		var f: Font = host._font
		draw_string_outline(f, Vector2(22, 38), "THE CITY OF", HORIZONTAL_ALIGNMENT_LEFT, -1, 26, 7, Color(0.06, 0.04, 0.02, 0.9))
		draw_string(f, Vector2(22, 38), "THE CITY OF", HORIZONTAL_ALIGNMENT_LEFT, -1, 26, Color(0.90, 0.86, 0.74))
		draw_string_outline(f, Vector2(22, 82), title.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 46, 10, Color(0.06, 0.04, 0.02, 0.9))
		draw_string(f, Vector2(22, 82), title.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 46, Color(0.93, 0.89, 0.78))

		# Markers. Real anchors: citadel centre, places of worship, the market. Illustrative: taverns on the
		# buildings nearest the market, the anchor pin at the water edge, the farm pin at a field.
		var pins: Array = []
		var cw: PackedVector2Array = layout.get("citadel_wall", PackedVector2Array())
		if cw.size() >= 3:
			var cc := Vector2.ZERO
			for p in cw:
				cc += p
			pins.append([_p(cc / cw.size()), "castle", false])
		var ws: Array = layout.get("worship_sites", [])
		if not ws.is_empty():
			pins.append([_p((ws[0] as Dictionary).get("center", Vector2.ZERO)), "tower", true])
		pins.append([market + Vector2(0, -16), "question", true])
		var near: Array = []
		var mk: Vector2 = layout.get("market", Vector2.ZERO)
		for b in bld:
			var bp: PackedVector2Array = b
			if bp.is_empty():
				continue
			near.append([bp[0].distance_to(mk), bp[0]])
		near.sort()
		var picked := 0
		var last := Vector2(-9999, -9999)
		for e in near:
			var sp := _p(e[1])
			if e[0] < 45.0 or sp.distance_to(last) < 140.0:
				continue
			pins.append([sp, "mug", true])
			last = sp
			picked += 1
			if picked >= 2:
				break
		if water.size() >= 3:
			var best := INF
			var bp2 := Vector2.ZERO
			for p in water:
				var d := p.distance_to(mk)
				if d < best:
					best = d
					bp2 = p
			pins.append([_p(bp2), "anchor", false])
		var farm: Array = layout.get("farmland", [])
		if not farm.is_empty():
			var fa: PackedVector2Array = farm[0]
			var fc := Vector2.ZERO
			for p in fa:
				fc += p
			pins.append([_p(fc / fa.size()), "wheat", false])
		# Round point markers on the wells.
		var shown_wells := 0
		for w in wells:
			shown_wells += 1
			if shown_wells % 3 != 0:
				continue
			draw_circle(w + Vector2(0, -14), 6.5, INK)
			draw_circle(w + Vector2(0, -14), 5.2, PIN_ORANGE)
			draw_circle(w + Vector2(0, -14), 1.8, Color(0.4, 0.2, 0.0))
		for pn in pins:
			_pin(pn[0], pn[1], pn[2])

	## A teardrop pin whose tip is at `tip`: orange for ordinary places, white for civic or special ones,
	## with a pictogram drawn from primitives (a mock: a built version would use vector glyphs).
	func _pin(tip: Vector2, kind: String, orange: bool) -> void:
		var r := 15.0
		var c := tip + Vector2(0, -r * 1.9)
		var fill := PIN_ORANGE if orange else PIN_WHITE
		var shape := PackedVector2Array()
		var a0 := deg_to_rad(-60.0)
		# Circle arc from the right tangent around the top to the left tangent, then the tip.
		var from := deg_to_rad(30.0)
		var to := deg_to_rad(150.0)
		for n in range(0, 25):
			var a: float = lerpf(to, from + TAU, float(n) / 24.0)
			shape.append(c + Vector2(cos(a), sin(a)) * r)
		shape.append(tip)
		var shadow := PackedVector2Array()
		for p in shape:
			shadow.append(p + Vector2(2, 3))
		_fill(shadow, Color(0, 0, 0, 0.3))
		_fill(shape, fill)
		var ol := shape.duplicate()
		ol.append(ol[0])
		draw_polyline(ol, Color(0.35, 0.20, 0.02) if orange else Color(0.45, 0.45, 0.42), 1.6, true)
		var ink := Color(0.30, 0.16, 0.02) if orange else Color(0.36, 0.36, 0.34)
		var ink_hi := Color(1.0, 0.95, 0.85) if orange else Color(0.30, 0.30, 0.28)
		match kind:
			"castle":
				draw_rect(Rect2(c + Vector2(-8, -3), Vector2(16, 10)), ink_hi)
				for dx in [-8, -2, 4]:
					draw_rect(Rect2(c + Vector2(dx, -8), Vector2(4, 5)), ink_hi)
				draw_rect(Rect2(c + Vector2(-2, 2), Vector2(4, 5)), ink)
			"tower":
				draw_rect(Rect2(c + Vector2(-5, -4), Vector2(10, 13)), ink_hi)
				for dx in [-5, -1, 3]:
					draw_rect(Rect2(c + Vector2(dx, -8), Vector2(3, 4)), ink_hi)
				draw_rect(Rect2(c + Vector2(-1.5, 3), Vector2(3, 6)), ink)
			"mug":
				draw_rect(Rect2(c + Vector2(-6, -6), Vector2(10, 13)), ink_hi)
				draw_arc(c + Vector2(5, 0), 4.0, -PI / 2.0, PI / 2.0, 10, ink_hi, 2.2, true)
				draw_rect(Rect2(c + Vector2(-6, -9), Vector2(10, 4)), Color(1, 1, 1))
			"question":
				draw_string(host._font, c + Vector2(-5, 8), "?", HORIZONTAL_ALIGNMENT_LEFT, -1, 24, ink)
			"anchor":
				draw_line(c + Vector2(0, -6), c + Vector2(0, 8), ink, 2.2)
				draw_arc(c + Vector2(0, -8), 2.6, 0, TAU, 12, ink, 1.8, true)
				draw_line(c + Vector2(-5, -2), c + Vector2(5, -2), ink, 2.0)
				draw_arc(c + Vector2(0, 2), 8.0, 0.2, PI - 0.2, 12, ink, 2.0, true)
			"wheat":
				for dx in [-4, 0, 4]:
					draw_line(c + Vector2(dx, 8), c + Vector2(dx, -6), ink, 1.6)
					for dy in [-5, -2, 1]:
						draw_line(c + Vector2(dx, dy + 2), c + Vector2(dx - 2.5, dy), ink, 1.2)
						draw_line(c + Vector2(dx, dy + 2), c + Vector2(dx + 2.5, dy), ink, 1.2)
