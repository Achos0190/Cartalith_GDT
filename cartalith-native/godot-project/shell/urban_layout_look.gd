extends RefCounted
## The map look for a town plan (CITY_MAP_LOOK_SCOPE.md milestone CL-1, Ruling EW).
##
## **What this is.** The second of two town styles. `urban_layout_draw.gd` is the ONE shared entry point (the
## City Viewer, the map's town layer, the right-dock thumbnail and the 16K/32K export copy all call its
## `draw_layout`); when `UrbanLayoutDraw.look_is_map()` it hands the whole draw to `draw_layout` below, with the
## same arguments and the same contract, so no surface can show a different look from another. The shipped
## ink-and-parchment plan stays in that file, byte-identical, and is the default (Ruling EW Q2: keep both).
##
## **What it must never do.** It draws only what the engine's `urban_layouts()` dictionary holds (every polygon,
## street, ring, tree and field is the engine's, nothing is invented), it never reads the engine itself, and it
## never changes geometry: this is a render-only restyle (CL-0 proves the layout bytes are unchanged). The
## wall-hugs-the-town layout change of Ruling EW item 1 is a separate engine step and is NOT here.
##
## **What differs from the ink plan** (the milestone's four asks):
##   1. Ground from the terrain: `layout["ground_color"]` is the base map's own colour at the town
##      (`EngineBridge.settlement_ground_colour`); the fields, the dirt inside the wall and the tree canopies are
##      derived from it, so a grassland town, a desert one and a snowy one differ. A layout without the key gets
##      the neutral parchment `UrbanLayoutDraw.GROUND`, never a stand-in green.
##   2. The citadel is drawn as a fortification: curtain band with crenellations, round towers, a gate with
##      flanking gatehouse towers and a paved threshold, a paved court and a keep with turrets, all from the
##      layout's `citadel_*` geometry. It is not a pin.
##   3. Dark outlined roofs with ridge and slope lines; a stone wall band with outlined round towers and paved
##      gates; tan streets and plazas; outlined clumped trees; furrowed fields.
##
## **Every colour below is a labelled judgement** read off the owner's reference image (a WorldAnvil city map,
## screenshot not stored: third-party art) and then checked against the scratch render
## `cartalith-native/docs/evidence/city-look-2026-11-13/example_before_after.png`. None is a measurement.
##
## **Fill guard.** Every polygon fill goes through `UrbanLayoutDraw._fill_ground_polygon` (which survives a
## triangulation failure at deep zoom: 42 failures at x64 once) or checks `Geometry2D.triangulate_polygon` first.

const BASE := preload("res://shell/urban_layout_draw.gd")

## Dirt and street tan of the reference. The terrain ground is mixed toward this to make the dirt inside the
## wall, so the paved and worn areas read as one family on every terrain.
const TAN := Color(0.80, 0.74, 0.55)
## How far the dirt inside the wall and inside blocks moves from the terrain toward `TAN`. Judgement: enough
## that a green meadow town has clearly tan streets and yards, little enough that a snowy ground still reads
## pale rather than becoming brown.
const DIRT_MIX := 0.62
## And the wall's whole interior, a shade less tan than the blocks so the built mass reads against it.
const INTERIOR_MIX := 0.38
## Street fill and its edge ("casing"), the reference's tan paving and its darker kerb.
const STREET := Color(0.84, 0.78, 0.60)
const STREET_EDGE := Color(0.55, 0.47, 0.30, 0.9)
## Plaza paving: paler than the street.
const PLAZA := Color(0.86, 0.81, 0.64)
const PLAZA_EDGE := Color(0.50, 0.43, 0.28)
## Roofs: dark brown with a lighter slope line each side of the ridge.
const ROOF := Color(0.27, 0.16, 0.10)
const ROOF_LIGHT := Color(0.52, 0.34, 0.24)
const ROOF_RIDGE := Color(0.72, 0.52, 0.36)
const INK := Color(0.10, 0.07, 0.05)
## Stone for the wall and the citadel: a light dressed stone, a dark outline, a tower fill and a keep stone.
const STONE := Color(0.58, 0.55, 0.50)
const STONE_DARK := Color(0.22, 0.20, 0.18)
const TOWER := Color(0.50, 0.47, 0.43)
const TOWER_TOP := Color(0.66, 0.63, 0.58)
const KEEP_STONE := Color(0.60, 0.56, 0.50)
const KEEP_ROOF := Color(0.42, 0.40, 0.38)
const PAVING := Color(0.78, 0.74, 0.64)
const COURT := Color(0.84, 0.80, 0.68)
## Trees: canopy, highlight and the dark rim that makes a clump read as drawn.
const CANOPY := Color(0.24, 0.48, 0.14)
const CANOPY_HI := Color(0.40, 0.62, 0.22)
const CANOPY_RIM := Color(0.08, 0.17, 0.06)
## How far a canopy is pulled toward the ground colour, so a tree on pale or dry ground is not a saturated
## green blob. Judgement.
const CANOPY_GROUND_MIX := 0.15
## Water: deep blue with a lighter shore and a sand line, the reference's sea.
const SEA := Color(0.06, 0.17, 0.36)
const SEA_SHORE := Color(0.20, 0.42, 0.55)
const SAND := Color(0.86, 0.80, 0.62)
## Field wheat tone mixed toward the ground, and how far.
const FIELD_TONE := Color(0.62, 0.58, 0.30)
const FIELD_MIX := 0.48
## Rows drawn per field are capped: a field's furrows are a polygon clip per row, redone on every pan.
const FURROW_ROWS_MAX := 36
## Smallest furrow spacing in screen pixels.
const FURROW_STEP_MIN_PX := 4.0
## A field smaller than this many pixels across gets no furrows (they would be a haze).
const FURROW_MIN_FIELD_PX := 12.0
## A roof must be this many pixels across before it earns its slope lines (the ink plan's own
## `ROOF_DETAIL_MIN_PX` is 9; the extra lines need a little more room).
const SLOPE_MIN_PX := 12.0
## The wall band must be this many pixels wide before the crenel dashes on top of it are drawn.
const CRENEL_MIN_BAND_PX := 5.0


## The town's ground colour: the layout's own `ground_color` when the bridge supplied one, else the neutral
## parchment. Never a stand-in green (Ruling EW item 1).
static func ground_of(layout: Dictionary) -> Color:
	var g: Variant = layout.get("ground_color", null)
	return g if g is Color else BASE.GROUND


## The dirt colour for `ground` (see `DIRT_MIX`).
static func dirt_of(ground: Color) -> Color:
	return ground.lerp(TAN, DIRT_MIX)


## Fills `rect` with the town's ground and mottles it with deterministic darker blotches so a large flat
## area does not read as a flat swatch. The blotch seed is a constant, so the same rect gives the same pixels
## on every call and on every surface. Used by the City Viewer and the thumbnail; the map's town layer has the
## map under it and calls neither.
static func draw_ground(ci: CanvasItem, rect: Rect2, layout: Dictionary) -> void:
	var g := ground_of(layout)
	ci.draw_rect(rect, g)
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	var n := clampi(int(rect.size.x * rect.size.y / 6000.0), 12, 260)
	var dark := g.darkened(0.25)
	for i in n:
		var c := rect.position + Vector2(rng.randf() * rect.size.x, rng.randf() * rect.size.y)
		ci.draw_circle(c, rng.randf_range(10.0, 34.0), Color(dark.r, dark.g, dark.b, 0.10))


## The same contract as `UrbanLayoutDraw.draw_layout` (read its doc comment for `to_screen`, `m_scale`,
## `px_floor`, `detail` and `draw_water`): draws the whole town in the map look. Draws nothing at
## `alpha <= 0`. Never mutates `layout`.
static func draw_layout(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, show_route_ends: bool,
		detail: float = 1.0, draw_water: bool = true) -> void:
	if alpha <= 0.0:
		return
	var ground := ground_of(layout)
	var tint := func(c: Color) -> Color:
		return Color(c.r, c.g, c.b, c.a * alpha)
	var project := func(poly: PackedVector2Array) -> PackedVector2Array:
		var out := PackedVector2Array()
		for p in poly:
			out.append(to_screen.call(p))
		return out

	# Ground first: the fields, then the water over them, then the built-up ground.
	_draw_farmland(ci, layout, project, m_scale, px_floor, alpha, detail, ground)
	if draw_water:
		_draw_water(ci, layout, project, m_scale, px_floor, tint)
	_draw_town_ground(ci, layout, project, tint, ground)
	if detail >= 1.0:
		BASE._draw_fences(ci, layout, project, m_scale, px_floor, alpha)
	_draw_streets(ci, layout, project, m_scale, px_floor, tint)

	# The wall: the stone band for a curtain; the ink plan's own palisade, ditch and bastioned strokes
	# otherwise (those three have no reference-image counterpart to restyle toward).
	var style := String(layout.get("wall_style", "curtain"))
	if style == "curtain":
		_draw_curtain(ci, layout, project, to_screen, m_scale, px_floor, alpha, detail)
	else:
		BASE._draw_wall(ci, layout, project, to_screen, m_scale, px_floor, alpha)

	_draw_trees(ci, layout, to_screen, m_scale, px_floor, alpha, detail, ground)
	_draw_roofs(ci, layout, to_screen, m_scale, px_floor, alpha, detail)
	BASE._draw_worship(ci, layout, project, to_screen, m_scale, px_floor, alpha, detail)
	_draw_citadel(ci, layout, project, to_screen, m_scale, px_floor, alpha, detail)
	if draw_water:
		# The mask paints `tint.call(WATER)`; this tint ignores its argument and answers the look's own sea.
		BASE._draw_water_mask(ci, layout, project, func(_c: Color) -> Color:
			return Color(SEA.r, SEA.g, SEA.b, alpha))
	# Props after the water mask, as in the ink plan: the crane and bollards belong on the water.
	BASE._draw_details(ci, layout, to_screen, m_scale, px_floor, alpha, detail, false)

	var plaza: PackedVector2Array = layout.get("plaza", PackedVector2Array())
	if plaza.size() > 2:
		var pp: PackedVector2Array = project.call(plaza)
		pp.append(pp[0])
		ci.draw_polyline(pp, tint.call(PLAZA_EDGE), maxf(px_floor, 1.2 * m_scale), true)
	for m in layout.get("markets", []):
		var mp: PackedVector2Array = m
		if mp.size() < 3:
			continue
		var mpts: PackedVector2Array = project.call(mp)
		BASE._fill_ground_polygon(ci, mpts, tint.call(PLAZA))
		mpts.append(mpts[0])
		ci.draw_polyline(mpts, tint.call(PLAZA_EDGE), maxf(px_floor, 1.0 * m_scale), true)
	if show_route_ends:
		for p in layout.get("route_ends", PackedVector2Array()):
			ci.draw_circle(to_screen.call(p), maxf(px_floor * 1.5, 4.0 * m_scale), tint.call(BASE.ROUTE_END))
	_draw_fountain(ci, layout, to_screen, m_scale, px_floor, tint)


## Fields outside the town: a flat fill mixed from the ground, and (at `detail >= 1`) furrow rows along each
## field's longest edge, alternately dark and light, plus a faint outline. Furrows are one `draw_multiline` each
## for dark and light across the whole town, and capped per field (`FURROW_ROWS_MAX`) because every row is a
## polygon clip redone on every pan.
static func _draw_farmland(ci: CanvasItem, layout: Dictionary, project: Callable, m_scale: float,
		px_floor: float, alpha: float, detail: float, ground: Color) -> void:
	var farm: Array = layout.get("farmland", [])
	if farm.is_empty():
		return
	var pasture: PackedByteArray = layout.get("farmland_pasture", PackedByteArray())
	var base_field := ground.lerp(FIELD_TONE, FIELD_MIX)
	var pasture_field := ground.lerp(Color(0.40, 0.52, 0.26), FIELD_MIX)
	var dark_lines := PackedVector2Array()
	var light_lines := PackedVector2Array()
	var edges := PackedVector2Array()
	for i in farm.size():
		var f: PackedVector2Array = farm[i]
		if f.size() < 3:
			continue
		var fp: PackedVector2Array = project.call(f)
		var is_pasture: bool = i < pasture.size() and pasture[i] != 0
		var c: Color = pasture_field if is_pasture else base_field
		BASE._fill_ground_polygon(ci, fp, Color(c.r, c.g, c.b, alpha))
		if detail < 1.0:
			continue
		for k in fp.size():
			edges.append(fp[k])
			edges.append(fp[(k + 1) % fp.size()])
		if is_pasture:
			continue
		_furrows(fp, m_scale, dark_lines, light_lines)
	if not edges.is_empty():
		ci.draw_multiline(edges, Color(INK.r, INK.g, INK.b, 0.35 * alpha), maxf(px_floor * 0.7, 0.5 * m_scale))
	var fw := maxf(px_floor * 1.1, 0.5 * m_scale * 2.0)
	if not dark_lines.is_empty():
		var d := base_field.darkened(0.45)
		ci.draw_multiline(dark_lines, Color(d.r, d.g, d.b, 0.85 * alpha), fw)
	if not light_lines.is_empty():
		var l := base_field.lightened(0.22)
		ci.draw_multiline(light_lines, Color(l.r, l.g, l.b, 0.8 * alpha), fw)


## Appends one field's furrow segments (screen space) to `dark` and `light`. Rows run along the longest edge,
## spaced `max(FURROW_STEP_MIN_PX, 3 m)` apart or wider so at most `FURROW_ROWS_MAX` fit, clipped to the polygon.
static func _furrows(fp: PackedVector2Array, m_scale: float, dark: PackedVector2Array,
		light: PackedVector2Array) -> void:
	var best := 0.0
	var dir := Vector2.RIGHT
	var lo_p := fp[0]
	var hi_p := fp[0]
	for k in fp.size():
		var e := fp[(k + 1) % fp.size()] - fp[k]
		if e.length() > best:
			best = e.length()
			dir = e.normalized()
		lo_p = Vector2(minf(lo_p.x, fp[k].x), minf(lo_p.y, fp[k].y))
		hi_p = Vector2(maxf(hi_p.x, fp[k].x), maxf(hi_p.y, fp[k].y))
	var diag := (hi_p - lo_p).length()
	if diag < FURROW_MIN_FIELD_PX:
		return
	var nrm := Vector2(-dir.y, dir.x)
	var lo := INF
	var hi := -INF
	for p in fp:
		lo = minf(lo, p.dot(nrm))
		hi = maxf(hi, p.dot(nrm))
	var step := maxf(maxf(FURROW_STEP_MIN_PX, 3.0 * m_scale), (hi - lo) / float(FURROW_ROWS_MAX))
	var t := lo + step * 0.5
	var alt := false
	while t < hi:
		var a := nrm * t - dir * diag * 2.0
		var b := nrm * t + dir * diag * 2.0
		for seg in Geometry2D.intersect_polyline_with_polygon(PackedVector2Array([a, b]), fp):
			var sp: PackedVector2Array = seg
			if sp.size() < 2:
				continue
			(light if alt else dark).append(sp[0])
			(light if alt else dark).append(sp[sp.size() - 1])
		alt = not alt
		t += step


## The site's water: deep blue over a lighter shore band and a sand line. `water_poly` is a body; a coastal site
## has none, so the river centreline is stroked instead (the ink plan's own rule). Only when `draw_water`: on the
## map the base map already paints the water underneath.
static func _draw_water(ci: CanvasItem, layout: Dictionary, project: Callable, m_scale: float,
		px_floor: float, tint: Callable) -> void:
	var water: PackedVector2Array = layout.get("water_poly", PackedVector2Array())
	if water.size() >= 3:
		var wp: PackedVector2Array = project.call(water)
		var wl := wp.duplicate()
		wl.append(wl[0])
		ci.draw_polyline(wl, tint.call(SEA_SHORE), maxf(px_floor * 8.0, 7.0 * m_scale), true)
		ci.draw_polyline(wl, tint.call(SAND), maxf(px_floor * 3.0, 2.5 * m_scale), true)
		BASE._fill_ground_polygon(ci, wp, tint.call(SEA))
	var river: PackedVector2Array = layout.get("river", PackedVector2Array())
	if water.size() < 3 and river.size() >= 2:
		ci.draw_polyline(project.call(river), tint.call(SEA),
			maxf(px_floor * 3.0, float(layout.get("river_w", 20.0)) * m_scale), true)


## Dirt and paving inside the town: the wall ring's whole interior in the lighter interior tone, then every
## block in the dirt tone (so the built mass reads against the open interior), the plaza block paler. Both follow
## the layout: an unwalled town has blocks only.
static func _draw_town_ground(ci: CanvasItem, layout: Dictionary, project: Callable, tint: Callable,
		ground: Color) -> void:
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	if ring.size() >= 3:
		BASE._fill_ground_polygon(ci, project.call(ring), tint.call(ground.lerp(TAN, INTERIOR_MIX)))
	var dirt := dirt_of(ground)
	var blocks: Array = layout.get("blocks", [])
	var plaza_flags: PackedByteArray = layout.get("block_plaza", PackedByteArray())
	for i in blocks.size():
		var blk: PackedVector2Array = blocks[i]
		if blk.size() < 3:
			continue
		var is_plaza: bool = i < plaza_flags.size() and plaza_flags[i] != 0
		BASE._fill_ground_polygon(ci, project.call(blk), tint.call(PLAZA if is_plaza else dirt))


## Streets: a darker kerb pass then the tan fill, every class, the ink plan's own class order so a primary sits
## on top. Widths are the real carriageway widths widened a quarter (the reference draws its lanes broad), floored
## so a hamlet's lanes stay visible.
static func _draw_streets(ci: CanvasItem, layout: Dictionary, project: Callable, m_scale: float,
		px_floor: float, tint: Callable) -> void:
	var streets: Dictionary = layout.get("streets", {})
	for pass_case in [true, false]:
		for cls in BASE.DRAW_ORDER:
			var segs: PackedVector2Array = streets.get(cls, PackedVector2Array())
			if segs.size() < 2:
				continue
			var w := maxf(px_floor * 1.6, float(BASE.W_FILL.get(cls, 3.0)) * 1.25 * m_scale)
			if pass_case:
				ci.draw_multiline(project.call(segs), tint.call(STREET_EDGE),
					w + clampf(2.0 * m_scale, px_floor, 4.0 * px_floor))
			else:
				ci.draw_multiline(project.call(segs), tint.call(STREET), w)


## The curtain wall in the map look: a dark-outlined stone band round the closed ring, spurs, a light crenel dash
## along its top when the band is wide enough, a round outlined tower at every ring vertex, and each land gate as a
## paved threshold set across the wall. Water gates are not drawn (the bridge omits them; see the ink plan's
## `_draw_wall` for why).
static func _draw_curtain(ci: CanvasItem, layout: Dictionary, project: Callable, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, detail: float) -> void:
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	if ring.size() < 3:
		return
	var band := maxf(px_floor * 2.4, float(BASE.WALL_W["curtain"]) * 1.2 * m_scale)
	var outline := clampf(1.3 * m_scale, px_floor * 1.2, band)
	var rp: PackedVector2Array = project.call(ring)
	rp.append(rp[0])
	var spurs: PackedVector2Array = project.call(layout.get("wall_spurs", PackedVector2Array()))
	ci.draw_polyline(rp, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha), band + outline * 2.0, true)
	if spurs.size() >= 2:
		ci.draw_multiline(spurs, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha), band + outline * 2.0)
	ci.draw_polyline(rp, Color(STONE.r, STONE.g, STONE.b, alpha), band, true)
	if spurs.size() >= 2:
		ci.draw_multiline(spurs, Color(STONE.r, STONE.g, STONE.b, alpha), band)
	if detail >= 1.0 and band >= CRENEL_MIN_BAND_PX:
		_crenel_dashes(ci, rp, band, Color(TOWER_TOP.r, TOWER_TOP.g, TOWER_TOP.b, alpha * 0.9))

	var tr := maxf(px_floor * 1.8, float(BASE.WALL_TOWER_R_M) * 1.25 * m_scale)
	for v in ring:
		_round_tower(ci, to_screen.call(v), tr, px_floor, alpha)

	var gates: PackedVector2Array = layout.get("wall_gates", PackedVector2Array())
	var hl := maxf(px_floor * 4.0, 6.0 * m_scale)
	var hd := maxf(px_floor * 3.0, 4.0 * m_scale)
	for g in gates:
		var t := _tangent_at(ring, g)
		var gs: Vector2 = to_screen.call(g)
		# `t` is a model-space direction; the screen direction of the same edge is the difference of two
		# projected points, which stays right whatever rotation `to_screen` carries.
		var ts: Vector2 = (to_screen.call(g + t) as Vector2) - gs
		var ts_n := ts.normalized() if ts.length() > 0.0001 else Vector2.RIGHT
		var ns := Vector2(-ts_n.y, ts_n.x)
		var quad := PackedVector2Array([gs - ts_n * hl - ns * hd, gs + ts_n * hl - ns * hd,
			gs + ts_n * hl + ns * hd, gs - ts_n * hl + ns * hd])
		BASE._fill_ground_polygon(ci, quad, Color(PAVING.r, PAVING.g, PAVING.b, alpha))
		quad.append(quad[0])
		ci.draw_polyline(quad, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha), maxf(px_floor, 1.2), true)


## Direction (unit, model space) of the ring edge nearest `p`, for orienting a gate across its wall. Falls back
## to +X on a degenerate ring.
static func _tangent_at(ring: PackedVector2Array, p: Vector2) -> Vector2:
	var best := INF
	var out := Vector2.RIGHT
	for k in ring.size():
		var a := ring[k]
		var b := ring[(k + 1) % ring.size()]
		var d := p.distance_squared_to(Geometry2D.get_closest_point_to_segment(p, a, b))
		if d < best and a.distance_squared_to(b) > 0.0001:
			best = d
			out = (b - a).normalized()
	return out


## One round tower: dark rim, stone body and a lighter roof cap. `r` is the body radius in screen units.
static func _round_tower(ci: CanvasItem, c: Vector2, r: float, px_floor: float, alpha: float) -> void:
	var rim := clampf(r * 0.28, px_floor, r)
	ci.draw_circle(c, r + rim, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha))
	ci.draw_circle(c, r, Color(TOWER.r, TOWER.g, TOWER.b, alpha))
	ci.draw_circle(c + Vector2(-r * 0.12, -r * 0.14), r * 0.55, Color(TOWER_TOP.r, TOWER_TOP.g, TOWER_TOP.b, alpha))


## Light dashes along the top of a wall band (`pts` is its screen polyline), the crenellation read. One
## `draw_multiline` for the whole ring: a dash per `2 x band` of length.
static func _crenel_dashes(ci: CanvasItem, pts: PackedVector2Array, band: float, col: Color) -> void:
	var seg := PackedVector2Array()
	var dash := band * 0.9
	for k in pts.size() - 1:
		var a := pts[k]
		var b := pts[k + 1]
		var d := b - a
		var len := d.length()
		if len < 0.5:
			continue
		var u := d / len
		var t := dash * 0.5
		while t + dash < len:
			seg.append(a + u * t)
			seg.append(a + u * (t + dash))
			t += dash * 2.0
	if not seg.is_empty():
		ci.draw_multiline(seg, col, maxf(1.0, band * 0.34))


## Trees under the roofs, as outlined clumps: shadow, dark rim, canopy, highlight. Rims first then bodies so
## overlapping crowns merge into one outline. Below `detail` 1 (map zoom) only the canopy disc is drawn, floored
## to a pixel, so a wooded hinterland still shows its greenery without per-tree passes.
static func _draw_trees(ci: CanvasItem, layout: Dictionary, to_screen: Callable, m_scale: float,
		px_floor: float, alpha: float, detail: float, ground: Color) -> void:
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
		tc.append(to_screen.call((geoms[n] as PackedVector2Array)[0]))
		tr.append(maxf(px_floor * 1.0, rr * m_scale * 1.3))
	if tc.is_empty():
		return
	var body := CANOPY.lerp(ground, CANOPY_GROUND_MIX)
	var hi := CANOPY_HI.lerp(ground, CANOPY_GROUND_MIX)
	var fine := detail >= 1.0
	if fine:
		for n in tc.size():
			ci.draw_circle(tc[n] + Vector2(0.45, 0.65) * tr[n], tr[n] + px_floor, Color(0, 0, 0, 0.25 * alpha))
		for n in tc.size():
			ci.draw_circle(tc[n], tr[n] + maxf(px_floor, tr[n] * 0.22), Color(CANOPY_RIM.r, CANOPY_RIM.g, CANOPY_RIM.b, alpha))
	for n in tc.size():
		ci.draw_circle(tc[n], tr[n], Color(body.r, body.g, body.b, alpha))
		if fine:
			ci.draw_circle(tc[n] + Vector2(-0.25, -0.3) * tr[n], tr[n] * 0.55, Color(hi.r, hi.g, hi.b, alpha))


## One dark-brown outlined roof per building footprint, with a drop shadow, a ridge line and a pair of lighter
## slope lines each side of it. The treatment is chosen from the MEASURED size of a sampled footprint (the same
## rule as the ink plan's `_draw_roofs`, for the same reason: a roof smaller than its own outline turns a dense
## town into a black mass), so `detail` only caps it. A footprint whose screen quad the triangulator refuses is
## skipped, the ink plan's own sub-pixel guard.
static func _draw_roofs(ci: CanvasItem, layout: Dictionary, to_screen: Callable, m_scale: float,
		px_floor: float, alpha: float, detail: float) -> void:
	var buildings: Array = layout.get("buildings", [])
	if buildings.is_empty():
		return
	var tones: PackedFloat32Array = layout.get("building_tone", PackedFloat32Array())
	var quads: Array[PackedVector2Array] = []
	quads.resize(buildings.size())
	for i in buildings.size():
		var b: PackedVector2Array = buildings[i]
		var q := PackedVector2Array()
		if b.size() >= 3:
			for p in b:
				q.append(to_screen.call(p))
			if Geometry2D.triangulate_polygon(q).is_empty():
				q = PackedVector2Array()
		quads[i] = q
	var sizes := PackedFloat32Array()
	var step: int = maxi(1, quads.size() / BASE.ROOF_SAMPLE)
	for i in range(0, quads.size(), step):
		var q: PackedVector2Array = quads[i]
		if q.size() < 3:
			continue
		var r := Rect2(q[0], Vector2.ZERO)
		for p in q:
			r = r.expand(p)
		sizes.append(maxf(r.size.x, r.size.y))
	if sizes.is_empty():
		return
	sizes.sort()
	var lot_px: float = sizes[sizes.size() / 2]
	var want_ink := detail >= 1.0 and lot_px >= BASE.ROOF_INK_MIN_PX
	var want_detail := detail >= 1.0 and lot_px >= BASE.ROOF_DETAIL_MIN_PX

	if want_detail:
		var soff := Vector2(1.4, 1.9) * m_scale
		for q in quads:
			if q.size() < 3:
				continue
			var s := PackedVector2Array()
			for p in q:
				s.append(p + soff)
			ci.draw_colored_polygon(s, Color(0, 0, 0, 0.32 * alpha))
	for i in quads.size():
		var q: PackedVector2Array = quads[i]
		if q.size() < 3:
			continue
		var tone: float = tones[i] if i < tones.size() else 0.5
		var c := ROOF.lerp(ROOF_LIGHT, clampf(tone - 0.35, 0.0, 0.3))
		ci.draw_colored_polygon(q, Color(c.r, c.g, c.b, alpha))
	if not want_ink:
		return
	var edges := PackedVector2Array()
	for q in quads:
		var n := q.size()
		if n < 3:
			continue
		for k in n:
			edges.append(q[k])
			edges.append(q[(k + 1) % n])
	ci.draw_multiline(edges, Color(INK.r, INK.g, INK.b, alpha),
		clampf(0.5 * m_scale, px_floor, maxf(px_floor, lot_px / 3.0)))
	if not want_detail:
		return
	var ridge: PackedVector2Array = layout.get("building_ridge", PackedVector2Array())
	if ridge.size() < 2:
		return
	var rl := PackedVector2Array()
	var slopes := PackedVector2Array()
	var off1 := clampf(1.2 * m_scale, px_floor, lot_px * 0.18)
	var with_slopes := lot_px >= SLOPE_MIN_PX
	for k in range(0, ridge.size() - 1, 2):
		var a: Vector2 = to_screen.call(ridge[k])
		var b: Vector2 = to_screen.call(ridge[k + 1])
		rl.append(a)
		rl.append(b)
		var d := b - a
		if with_slopes and d.length() > 1.0:
			var nrm := Vector2(-d.y, d.x).normalized()
			for o in [-2.0, -1.0, 1.0, 2.0]:
				slopes.append(a + nrm * off1 * o)
				slopes.append(b + nrm * off1 * o)
	if not slopes.is_empty():
		ci.draw_multiline(slopes, Color(ROOF_LIGHT.r, ROOF_LIGHT.g, ROOF_LIGHT.b, 0.7 * alpha), maxf(px_floor, 1.0))
	ci.draw_multiline(rl, Color(ROOF_RIDGE.r, ROOF_RIDGE.g, ROOF_RIDGE.b, alpha), clampf(0.7 * m_scale, px_floor, 2.0 * px_floor))


## The citadel as a walled fortification (Ruling EW item 1; the layout's `citadel_*` keys,
## `cartalith_urban::Citadel`). Drawn, in order:
##   * the paved court (`citadel_court`, else the whole enclosure) over the town beneath;
##   * the curtain: a dark-outlined stone band round the four corners with a gap at `citadel_gate`, crenel dashes
##     along its top;
##   * the keep (`citadel_keep`): dark-outlined stone with a lighter inset roof and a turret at each corner;
##   * the gate: a paved threshold across the gap, with a small gatehouse tower either side;
##   * every `citadel_towers` point as a round outlined tower of `citadel_tower_r` (the four corners AND each place
##     the enclosure meets the town circuit).
## A town without a citadel carries no `citadel_wall`, so nothing is drawn. Radii and widths are in metres times
## `m_scale` with pixel floors, so it reads at every zoom: at map zoom (`detail < 1`) the court, band, keep and
## towers draw without the dashes and turrets.
static func _draw_citadel(ci: CanvasItem, layout: Dictionary, project: Callable, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, detail: float) -> void:
	var wall: PackedVector2Array = layout.get("citadel_wall", PackedVector2Array())
	if wall.size() != 4:
		return
	var court: PackedVector2Array = layout.get("citadel_court", PackedVector2Array())
	BASE._fill_ground_polygon(ci, project.call(court if court.size() >= 3 else wall),
		Color(COURT.r, COURT.g, COURT.b, alpha))
	# The enclosure's own footprint, under the court, so a court smaller than the wall leaves a stone apron.
	var gate: Vector2 = layout.get("citadel_gate", wall[0].lerp(wall[1], 0.5))
	var half_gap: Vector2 = (wall[1] - wall[0]).normalized() * (BASE.CITADEL_GATE_GAP_M * 0.5)
	var path := PackedVector2Array([gate + half_gap, wall[1], wall[2], wall[3], wall[0], gate - half_gap])
	var band := maxf(px_floor * 2.8, float(BASE.WALL_W["curtain"]) * 1.4 * m_scale)
	var outline := clampf(1.4 * m_scale, px_floor * 1.2, band)
	var sp: PackedVector2Array = project.call(path)
	ci.draw_polyline(sp, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha), band + outline * 2.0, true)
	ci.draw_polyline(sp, Color(STONE.r, STONE.g, STONE.b, alpha), band, true)
	if detail >= 1.0 and band >= CRENEL_MIN_BAND_PX:
		_crenel_dashes(ci, sp, band, Color(TOWER_TOP.r, TOWER_TOP.g, TOWER_TOP.b, alpha * 0.9))

	var keep: PackedVector2Array = layout.get("citadel_keep", PackedVector2Array())
	if keep.size() >= 3:
		var kp: PackedVector2Array = project.call(keep)
		BASE._fill_ground_polygon(ci, kp, Color(KEEP_STONE.r, KEEP_STONE.g, KEEP_STONE.b, alpha))
		# The lighter roof, inset: the offset can return nothing for a sliver, which just skips it.
		var inset := Geometry2D.offset_polygon(kp, -maxf(px_floor * 1.5, 3.0 * m_scale))
		if not inset.is_empty():
			BASE._fill_ground_polygon(ci, inset[0], Color(KEEP_ROOF.r, KEEP_ROOF.g, KEEP_ROOF.b, alpha))
		var ko := kp.duplicate()
		ko.append(ko[0])
		ci.draw_polyline(ko, Color(STONE_DARK.r, STONE_DARK.g, STONE_DARK.b, alpha),
			maxf(px_floor * 1.3, 1.4 * m_scale), true)
		if detail >= 1.0:
			var kt := maxf(px_floor * 1.6, 2.4 * m_scale)
			for p in kp:
				_round_tower(ci, p, kt, px_floor, alpha)

	var tr := maxf(px_floor * 2.0, float(layout.get("citadel_tower_r", 6.0)) * m_scale)
	for p in layout.get("citadel_towers", PackedVector2Array()):
		_round_tower(ci, to_screen.call(p), tr, px_floor, alpha)

	# The gate: a paved threshold across the gap and a small gatehouse tower on each side of it.
	var gs: Vector2 = to_screen.call(gate)
	var ga: Vector2 = to_screen.call(gate + half_gap) - gs
	var gl := ga.length()
	if gl > 0.001:
		var u := ga / gl
		var nv := Vector2(-u.y, u.x)
		var hd := band * 0.6
		var quad := PackedVector2Array([gs - u * gl - nv * hd, gs + u * gl - nv * hd,
			gs + u * gl + nv * hd, gs - u * gl + nv * hd])
		BASE._fill_ground_polygon(ci, quad, Color(PAVING.r, PAVING.g, PAVING.b, alpha))
		var gt := maxf(px_floor * 1.6, tr * 0.7)
		_round_tower(ci, gs + u * gl, gt, px_floor, alpha)
		_round_tower(ci, gs - u * gl, gt, px_floor, alpha)


## The market fountain: a stone ring round blue water at the market anchor, the reference's civic centre mark.
## Always drawn (the ink plan draws a ring there too), sized from metres with a floor.
static func _draw_fountain(ci: CanvasItem, layout: Dictionary, to_screen: Callable, m_scale: float,
		px_floor: float, tint: Callable) -> void:
	var m: Vector2 = to_screen.call(layout.get("market", Vector2.ZERO))
	var r := maxf(px_floor * 2.4, 4.5 * m_scale)
	ci.draw_circle(m, r + maxf(px_floor, r * 0.18), tint.call(STONE_DARK))
	ci.draw_circle(m, r, tint.call(Color(0.70, 0.67, 0.61)))
	ci.draw_circle(m, r * 0.72, tint.call(Color(0.20, 0.50, 0.85)))
	ci.draw_circle(m, r * 0.28, tint.call(Color(0.85, 0.93, 1.0)))
