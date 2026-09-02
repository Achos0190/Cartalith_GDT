extends RefCounted
class_name UrbanLayoutDraw

## The one place a `urban_layouts()` dictionary is turned into strokes.
##
## Ported from the reference's `_umDrawLayout` (HTML line 22774) and
## `_umDrawLayoutPreview` (line 22901) — which are the same drawing twice,
## differing only in the metres→screen transform they receive. So this is one
## function taking that transform as a `Callable`: `map_overlay.gd` passes its
## rotate-about-the-market-then-project-to-grid mapping, and
## `city_viewer_window.gd` passes its fit-to-box one. Where those two draw
## nothing, the third renderer `_cvDrawCity` (line 23021) is the source: the
## per-parcel district fills and the market squares are its.
##
## ## What is drawn
##
## **As of 2026-09-02, the whole town.** `run_layout` became a caller of the
## reference's own `generate()` instead of a hand-ordered subset beside it, and
## five layers that had no key in the bridge arrived at once — so this file
## gained five draws: the farmland outside, the per-lot district fills, the wall
## circuit with its gates and spurs, the buildings, and the specialised market
## squares.
##
## The important one is the buildings, because it retires the one place where
## this drawing was knowingly ahead of the generator. A rooftop used to be a
## whole *parcel*, inset — a lot is not a building, every lot was built, and
## every roof was the same quad. A rooftop is now `buildBuildings`' own
## footprint, with its own generated ridge line, and a lot with no building on
## it stays empty ground because the engine left it empty.
##
## Three of the model's layers are still not drawn, and each is absent from the
## bridge rather than skipped here: the justified crossings (bridge decks and
## the stippled ford band, `_umDrawLayout` line 22854), the civic hall and
## places of worship, and the hinterland clutter (trees, fences, drying racks).
##
## ## The visual treatment
##
## The reference draws a flat technical plan. This draws an ink-outlined one,
## following the technique the owner's reference image is built on: every roof
## takes a slightly different brightness and saturation of one warm palette, so
## the town reads as weathered rather than uniform. The variation is **not**
## random per frame — it comes from the engine's own per-parcel `tone`, resolved
## onto each building by the adapter (`UrbanLayout::building_tone`) so this file
## never has to match a footprint to its lot.
##
## Map content keeps this warm ink-and-parchment language and does **not**
## follow the shell's light/dark theme — the same rule this file already
## recorded for streets and water, and `map_overlay.gd` for faction colour. The
## shell's amber `accent` appears only on annotation drawn *over* the map (the
## market anchor, approach-road ends), never as map ink, which is how the two
## visual languages sit together without competing.

## `_umDrawLayout`'s own palette, RGB for RGB — the reference's 8-bit values
## in the comment beside each, written as float components because `Color8()`
## is a runtime call and these are `const`.
const WATER := Color(0.427, 0.561, 0.675)        # rgb(109,143,172), the reference's
                                                 # rgb(92,130,172) pulled toward the
                                                 # parchment so it sits in the same light
const CASING := Color(0.169, 0.129, 0.094)       # rgb(43,33,24) -- the ink
const FILL_PRIMARY := Color(0.831, 0.784, 0.671) # rgb(212,200,171)
const FILL_OTHER := Color(0.792, 0.741, 0.624)   # rgb(202,189,159)
## Not in the reference's palette: it never strokes the centreline separately.
## See `draw_layout()` for why a coastal site needs it.
const RIVER_LINE := Color(0.325, 0.463, 0.588)
## Likewise this port's own: the reference draws no market marker because its
## own plaza/civic layers sit there instead. This is the shell's `accent`,
## deliberately -- it is an annotation, not map ink.
const MARKET := Color(0.878, 0.639, 0.290)       # #e0a34a
const ROUTE_END := Color(0.729, 0.545, 0.271)
## The unbuilt ground. Muted rather than paper-white: this is a document shown
## inside a dark tool window, and a bright sheet would glare.
const GROUND := Color(0.761, 0.702, 0.580)       # rgb(194,179,148)
## The built interior of a block, a shade under the parchment.
const BLOCK_GROUND := Color(0.671, 0.604, 0.478) # rgb(171,154,122)
## The market square: `_umDrawLayout` line 22804 fills a plaza block
## rgb(208,192,154) where an ordinary block is rgb(182,172,148), so this is
## `BLOCK_GROUND` moved by the same ratio rather than a colour picked here. The
## square has to read as *open ground* — lighter than the built mass around it
## and with no roofs on it, which is what `buildParcels` skipping it achieves.
const PLAZA_GROUND := Color(0.767, 0.674, 0.498)
## The plaza outline, likewise the reference's own rgb(150,128,86) (line 23046)
## pulled into this drawing's ink range.
const PLAZA_LINE := Color(0.588, 0.502, 0.337)

## `buildFarmland`'s strip and ring fields. **The reference has no colour for
## these** — neither `_umDrawLayout` nor `_cvDrawCity` draws a `field` or a
## `pasture` at all, so unlike everything above these two are derived from this
## file's own vocabulary rather than ported: `GROUND` shifted toward olive, and
## a touch greener again for grazing. They sit under everything, because that is
## what a town's hinterland is.
const FARM_FIELD := Color(0.702, 0.678, 0.510)
const FARM_PASTURE := Color(0.647, 0.667, 0.510)
## The furrow line between neighbouring strips. Strip fields read as strips
## because of their boundaries; without this a ring of them is one olive blob.
const FARM_LINE := Color(0.169, 0.129, 0.094, 0.22)

## `_UM_DISTRICT_FILL` (reference line 22987) — `_cvDrawCity`'s per-lot fills at
## its "city" LOD tier, translucent so streets and roofs stay legible over them.
## RGBA for RGBA, alpha included: the alphas are the whole reason the table
## works, and this drawing has the same layering problem.
##
## `church` has no entry **in the reference either** (`if(!fill) continue`), so
## a churchyard lot takes the plain block ground, and neither do the five
## economy districts — those are reachable only from a settlement
## `specialisation`, which this port's settlements do not carry, so a key for
## one would be a colour nothing can select.
const DISTRICT_FILL := {
	"market": Color(0.847, 0.788, 0.588, 0.32),
	"burgher": Color(0.776, 0.706, 0.549, 0.26),
	"artisan": Color(0.690, 0.643, 0.541, 0.22),
	"craftriver": Color(0.588, 0.675, 0.675, 0.30),
	"harbour": Color(0.549, 0.635, 0.745, 0.30),
	"suburb": Color(0.659, 0.690, 0.549, 0.20),
	"agrarian": Color(0.667, 0.737, 0.510, 0.20),
}

## The wall circuit, one colour per style — the reference's own three
## (`_umDrawLayout` lines 22836/22843/22848). They are already in `CASING`'s ink
## family, so unlike `WATER` these are carried straight across.
const WALL_STONE := Color(0.251, 0.204, 0.141)     # rgb(64,52,36)
const WALL_PALISADE := Color(0.376, 0.282, 0.173)  # rgb(96,72,44)
const WALL_DITCH := Color(0.471, 0.400, 0.290)     # rgb(120,102,74)
## The ditch's inner bank line, rgb(134,116,88).
const WALL_DITCH_INNER := Color(0.525, 0.455, 0.345)
## Gate markers, rgb(48,38,26) for stone and rgb(70,52,32) for a palisade.
const GATE_STONE := Color(0.188, 0.149, 0.102)
const GATE_PALISADE := Color(0.275, 0.204, 0.125)
## Base stroke widths in model metres, from the reference's own three branches:
## `4.5` stone, `2.2` palisade, `1.6` ditch.
const WALL_W := {"curtain": 4.5, "palisade": 2.2, "ditch": 1.6}
## The reference's `bastioned` branch is not here, and its absence is generated
## rather than chosen: a star fort is gated on `opts.fortified`, which reads
## `p.traits.includes('fortified')`, and this port's settlements carry no
## traits. No town it generates can have one, so a branch for it would be dead
## code claiming otherwise.

## The rooftop base, in HSV. Every roof is this hue with its brightness and
## saturation moved together by its own `tone` -- see `_roof_color()`.
const ROOF_H := 0.058
const ROOF_S := 0.46
const ROOF_V := 0.60
## How far `tone` swings each. Value up and saturation *down* together: a
## sun-bleached, weathered roof is lighter and less saturated at once, so one
## scalar driving both is truer than two independent jitters (and is why the
## engine emits one number rather than three).
const ROOF_V_SWING := 0.17
const ROOF_S_SWING := 0.13
## A slight hue drift, so the palette is a family rather than one chip.
const ROOF_H_SWING := 0.030

## The drop shadow, in model metres, down and to the right -- one light source
## for the whole map, from the upper left.
const SHADOW_OFF := Vector2(1.1, 1.4)
const SHADOW := Color(0.169, 0.129, 0.094, 0.30)

## `wFill` (reference line 22811) — the base stroke width per street class, in
## model metres. All five: `ringroad` and `quay` became reachable when
## `run_layout` stopped running its own stage subset (`supersedeWall` demolishes
## a superseded circuit into the first, `buildHarbour` lays the second).
const W_FILL := {"primary": 6.0, "ringroad": 4.6, "quay": 4.6, "street": 3.2, "lane": 1.8}
## `order` (reference line 22821), in full.
const DRAW_ORDER: Array[String] = ["lane", "street", "quay", "ringroad", "primary"]
## The widest the street casing's ink may get, in the drawing surface's own
## units. See the width calculation in `draw_layout()` for why it is capped
## rather than scaled.
const CASING_MAX_PX := 5.0


## `to_screen` maps a model-metre `Vector2` into `ci`'s own coordinate space.
## `m_scale` is that space's units per model metre, for stroke widths — the
## reference keeps position and width scaling separate for the same reason
## (widths do not pass through its own viewport crop the way coordinates do).
##
## `px_floor` is **one screen pixel expressed in that same space**, and it is
## what the reference's own `Math.max(1.0, …)`/`Math.max(0.5, …)` width floors
## mean: never thinner than a pixel. The City Viewer draws straight into screen
## space and passes `1.0`; `map_overlay.gd` draws into a control the camera
## then scales by `ViewportHost.zoom()`, so it passes `1.0 / zoom` — without
## that, every floored stroke would come out `zoom` times too thick and a town
## at deep zoom would render as a solid blob.
##
## `detail` is the one concession to the map's deep-zoom layer: below 1.0 the
## per-roof passes (shadow, outline, ridge) and the per-lot district fills are
## skipped, because at map zoom a town is a few hundred pixels across and those
## passes over ~2000 lots buy nothing a viewer can see. The City Viewer passes
## 1.0.
static func draw_layout(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, show_route_ends: bool,
		detail: float = 1.0) -> void:
	if alpha <= 0.0:
		return
	var tint := func(c: Color) -> Color:
		return Color(c.r, c.g, c.b, c.a * alpha)
	var project := func(poly: PackedVector2Array) -> PackedVector2Array:
		var out := PackedVector2Array()
		for p in poly:
			out.append(to_screen.call(p))
		return out

	# The hinterland, under everything: `buildFarmland`'s strips or rings. They
	# lie outside the urbanised core by construction (the engine's own `urban()`
	# guard rejects a field whose centroid is inside the wall, or within 0.7 of
	# the urban radius when there is no wall), so nothing else is drawn over
	# them -- but they are ground, and ground goes first.
	_draw_farmland(ci, layout, project, m_scale, px_floor, alpha, detail)

	# Water next, opaque, so the crossfade alpha is the only transparency --
	# the reference's own note: the town read see-through at full zoom because
	# the fills themselves were < 1 alpha.
	var water_poly: PackedVector2Array = layout.get("water_poly", PackedVector2Array())
	if water_poly.size() >= 3:
		ci.draw_colored_polygon(project.call(water_poly), tint.call(WATER))

	# The river centreline. `_umDrawLayout` does not stroke this separately --
	# it relies on `waterPoly` covering it. A coastal site has no closed water
	# polygon at all (`buildSite` traces a shoreline, not a body), so without
	# this the water would simply not be visible on those sites.
	var river: PackedVector2Array = layout.get("river", PackedVector2Array())
	if river.size() >= 2 and water_poly.size() < 3:
		var rw: float = maxf(px_floor, float(layout.get("river_w", 20.0)) * m_scale)
		ci.draw_polyline(project.call(river), tint.call(RIVER_LINE), rw, true)

	# Block ground. Under the streets, so the street fills read as channels cut
	# through the built mass rather than as lines drawn on it. The market square
	# is one of these, flagged and filled a shade lighter -- it is the same
	# layer, not an overlay, because it *is* a block: the one the engine kept
	# unbuilt.
	var blocks: Array = layout.get("blocks", [])
	var block_plaza: PackedByteArray = layout.get("block_plaza", PackedByteArray())
	for i in blocks.size():
		var blk: PackedVector2Array = blocks[i]
		if blk.size() < 3:
			continue
		var is_plaza: bool = i < block_plaza.size() and block_plaza[i] != 0
		ci.draw_colored_polygon(project.call(blk),
			tint.call(PLAZA_GROUND if is_plaza else BLOCK_GROUND))

	# `assignDistricts`' zoning, as translucent per-lot fills over that ground
	# -- `_cvDrawCity`'s "city" tier (reference line 23041). This is the layer
	# that makes a market quarter, a riverside craft strip and an outer agrarian
	# fringe visible as different places rather than as one uniform mass.
	if detail >= 1.0:
		var parcels: Array = layout.get("parcels", [])
		var districts: PackedStringArray = layout.get("parcel_district", PackedStringArray())
		for i in mini(parcels.size(), districts.size()):
			if not DISTRICT_FILL.has(districts[i]):
				continue
			var par: PackedVector2Array = parcels[i]
			if par.size() < 3:
				continue
			ci.draw_colored_polygon(project.call(par), tint.call(DISTRICT_FILL[districts[i]]))

	# Streets: casing (ink, wider) then fill (light, narrower), so the network
	# reads as continuous lines rather than loose segments. Both passes walk
	# the classes in the reference's own order so a primary always sits on top.
	var streets: Dictionary = layout.get("streets", {})
	for pass_case in [true, false]:
		for cls in DRAW_ORDER:
			var segs: PackedVector2Array = streets.get(cls, PackedVector2Array())
			if segs.size() < 2:
				continue
			var base: float = W_FILL.get(cls, 3.0)
			# The fill is the real carriageway width and scales with everything
			# else. The casing is an **outline**, so its extra width is capped
			# in pixels rather than scaled: the reference multiplies it by
			# `m_scale` too, but the reference never zooms as far as fitting an
			# 11-lot hamlet to a window does. Uncapped, a two-block hamlet
			# renders as a few roofs adrift in a black cross.
			var fill_w: float = maxf(px_floor * 0.5, base * m_scale)
			var width: float = fill_w + clampf(2.4 * m_scale, px_floor, CASING_MAX_PX) \
				if pass_case else fill_w
			var color: Color = CASING if pass_case \
				else (FILL_PRIMARY if cls == "primary" else FILL_OTHER)
			ci.draw_multiline(project.call(segs), tint.call(color), width)

	# The wall, over the streets it gates and under the roofs it contains --
	# the reference's own position for it (line 22824, between the street passes
	# and the building fills).
	_draw_wall(ci, layout, project, to_screen, m_scale, px_floor, alpha)

	_draw_roofs(ci, layout, to_screen, m_scale, px_floor, alpha, detail)

	# The plaza outline, over the roofs -- the square's edge is where the built
	# frontages stop, so it has to sit above them to be the boundary rather than
	# a line under them. Nothing is filled here; the fill is the flagged block
	# above.
	var plaza: PackedVector2Array = layout.get("plaza", PackedVector2Array())
	if plaza.size() > 2:
		var ppts: PackedVector2Array = project.call(plaza)
		ppts.append(ppts[0])
		ci.draw_polyline(ppts, tint.call(PLAZA_LINE), maxf(px_floor, 1.2 * m_scale), true)

	# `buildMarkets`' specialised squares -- the fish market, the cattle market,
	# the cloth hall, which multiply with rank (M-AMEN-1). The reference glyphs
	# and labels these (line 23124); this port draws the square itself, in the
	# plaza's own treatment, because they are the same kind of thing: swept open
	# ground the engine cleared lots and buildings off to make.
	var markets: Array = layout.get("markets", [])
	for m in markets:
		var mp: PackedVector2Array = m
		if mp.size() < 3:
			continue
		var pts: PackedVector2Array = project.call(mp)
		ci.draw_colored_polygon(pts, tint.call(PLAZA_GROUND))
		pts.append(pts[0])
		ci.draw_polyline(pts, tint.call(PLAZA_LINE), maxf(px_floor, 1.0 * m_scale), true)

	if show_route_ends:
		var ends: PackedVector2Array = layout.get("route_ends", PackedVector2Array())
		for p in ends:
			ci.draw_circle(to_screen.call(p), maxf(px_floor * 1.5, 4.0 * m_scale), tint.call(ROUTE_END))

	# The market anchor. `anchors.market` is the single most-read value the
	# engine produces (twenty-odd call sites downstream), and on the map it is
	# the point the whole layout is pinned to the settlement by, so it is worth
	# seeing where it landed.
	var market: Vector2 = layout.get("market", Vector2.ZERO)
	var mr: float = maxf(px_floor * 2.0, 6.0 * m_scale)
	ci.draw_arc(to_screen.call(market), mr, 0.0, TAU, 20, tint.call(MARKET),
		maxf(px_floor, mr * 0.34), true)


## `buildFarmland`'s fields and pastures, filled flat with a hairline furrow
## between them.
##
## Two colours and one boundary stroke, not a texture: what distinguishes a
## strip-field landscape is the *parcel pattern*, and the engine already emits
## every strip as its own polygon. Drawing them as one merged colour would throw
## away the only thing the stage generates.
static func _draw_farmland(ci: CanvasItem, layout: Dictionary, project: Callable,
		m_scale: float, px_floor: float, alpha: float, detail: float) -> void:
	var farm: Array = layout.get("farmland", [])
	if farm.is_empty():
		return
	var pasture: PackedByteArray = layout.get("farmland_pasture", PackedByteArray())
	# The furrows are gated the way the roof ink is, and for the same reason: at
	# map zoom a field is a couple of pixels across and its boundary is wider
	# than the strip it separates, so the whole ring reads as one grey haze
	# rather than as land. Decided before the loop so the edge list is not
	# accumulated for a pass that will not run.
	var want_furrows := detail >= 1.0
	var edges := PackedVector2Array()
	for i in farm.size():
		var f: PackedVector2Array = farm[i]
		if f.size() < 3:
			continue
		var pts: PackedVector2Array = project.call(f)
		var c: Color = FARM_PASTURE if (i < pasture.size() and pasture[i] != 0) else FARM_FIELD
		ci.draw_colored_polygon(pts, Color(c.r, c.g, c.b, alpha))
		if not want_furrows:
			continue
		# One `draw_multiline` for every furrow in the town, for the same reason
		# the roof ink is one call: a ring of fields runs to a couple of hundred
		# polygons and a stroke each was measurable.
		for k in pts.size():
			edges.append(pts[k])
			edges.append(pts[(k + 1) % pts.size()])
	if edges.is_empty():
		return
	ci.draw_multiline(edges, Color(FARM_LINE.r, FARM_LINE.g, FARM_LINE.b, FARM_LINE.a * alpha),
		maxf(px_floor * 0.6, 0.5 * m_scale))


## The wall circuit: the closed containment ring, its style, its spurs and its
## land gates.
##
## **The ring is drawn closed.** The reference's own note (line 22824) records
## why: it used to stroke `landArc` as an open path, so a landlocked town —
## where `landArc` *is* the full ring — showed a gap and read as "not going
## around the city".
##
## Water gates are carried by the bridge and deliberately not drawn, which is
## what all three reference renderers do (`if(gt&&gt.pt&&!gt.water)`): a water
## gate is the river passing under the circuit or the harbour mouth, and a
## marker there would read as a road entrance that is not one.
static func _draw_wall(ci: CanvasItem, layout: Dictionary, project: Callable,
		to_screen: Callable, m_scale: float, px_floor: float, alpha: float) -> void:
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	if ring.size() < 3:
		return
	var style := String(layout.get("wall_style", "curtain"))
	var lw: float = maxf(px_floor, float(WALL_W.get(style, 4.5)) * m_scale)
	var col: Color = WALL_PALISADE if style == "palisade" \
		else (WALL_DITCH if style == "ditch" else WALL_STONE)
	var pts: PackedVector2Array = project.call(ring)
	pts.append(pts[0])
	ci.draw_polyline(pts, Color(col.r, col.g, col.b, alpha), lw, true)

	if style == "ditch":
		# A ditch-and-bank is two earth lines, not one masonry one: the inner
		# bank is the ring pulled 3% toward the wall's own centroid (reference
		# line 22845). Without a centroid there is no inner line to draw, which
		# is the reference's own `W.centroid||{x:0,y:0}` degrading to a ring
		# collapsed at the origin -- so this skips it instead.
		if layout.has("wall_centroid"):
			var c: Vector2 = layout["wall_centroid"]
			var inner := PackedVector2Array()
			for p in ring:
				inner.append(to_screen.call(p + (c - p) * 0.03))
			inner.append(inner[0])
			ci.draw_polyline(inner, Color(WALL_DITCH_INNER.r, WALL_DITCH_INNER.g,
				WALL_DITCH_INNER.b, alpha), lw * 0.8, true)
	elif style == "palisade":
		# Post ticks every second ring vertex (reference line 22838) -- what
		# makes a timber stockade read as timber rather than as a thin wall.
		var pr: float = maxf(px_floor * 0.5, 1.2 * m_scale)
		var post := Color(col.r, col.g, col.b, alpha)
		for k in range(0, ring.size(), 2):
			ci.draw_circle(to_screen.call(ring[k]), pr, post)
	else:
		# Spurs: the short wall stubs `buildWall` runs down to the water on a
		# riverside circuit. Stone only, as in the reference.
		var spurs: PackedVector2Array = layout.get("wall_spurs", PackedVector2Array())
		if spurs.size() >= 2:
			ci.draw_multiline(project.call(spurs), Color(col.r, col.g, col.b, alpha), lw)

	var gates: PackedVector2Array = layout.get("wall_gates", PackedVector2Array())
	if gates.is_empty():
		return
	var gc: Color = GATE_PALISADE if style == "palisade" else GATE_STONE
	var gr: float = maxf(px_floor * 0.8, (1.8 if style == "palisade" else 2.2) * m_scale)
	for g in gates:
		ci.draw_circle(to_screen.call(g), gr, Color(gc.r, gc.g, gc.b, alpha))


## A footprint has to cover at least this many pixels before it is worth
## outlining. Below it the ink is wider than the roof it surrounds and a dense
## city renders as a black mass — measured, not guessed: a 4,370-parcel town
## fitted to a 900 px canvas puts a lot at ~3 px, and at that size the outline
## pass swallowed the tone variation completely.
const ROOF_INK_MIN_PX := 4.5
## And this many before the ridge and the drop shadow earn their passes.
const ROOF_DETAIL_MIN_PX := 9.0
## How many footprints to sample when measuring that. The answer only picks
## between three treatments, so a sample is as good as a census and is O(1) per
## redraw rather than O(buildings).
const ROOF_SAMPLE := 24


## One rooftop per **building**.
##
## This used to draw one per *parcel*, inset toward its centroid, and said so at
## length: `buildBuildings` did not exist, a lot is not a building, and drawing
## the lot was the one place in this file where the drawing was ahead of the
## generation. That is over. Every polygon here is `buildBuildings`' own
## footprint, produced by a grammar that varies with district (burgage rows,
## courtyard plots with a street range and two wings, sheds on a working yard)
## and gated on terrain suitability — so a town now has gaps in it, because the
## engine left them, and `ROOF_INSET` is gone because the setbacks are generated
## rather than drawn.
##
## The ridge is the engine's too (`Building::ridge`), one segment per footprint,
## rather than two midpoints this file computed off a quad.
##
## ## Why the passes are gated on measured pixels rather than on zoom
##
## A town's footprint count runs from a handful (a hamlet) to a few thousand (a
## city), so "how big is a roof on screen" is not a function of zoom alone — the
## same canvas at the same fit shows one at ~40 px and another at ~3 px. The ink
## outline is what makes this read as drawn, and it is also what destroys it
## when a roof is smaller than the line around it. So the treatment is chosen
## from the *measured* size of a sampled footprint, and `detail` only caps it
## (the map overlay passes 0.0 to force flat fills at map zoom regardless).
static func _draw_roofs(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, detail: float) -> void:
	var buildings: Array = layout.get("buildings", [])
	if buildings.is_empty():
		return
	var tones: PackedFloat32Array = layout.get("building_tone", PackedFloat32Array())

	# Screen-space footprints, built once and reused by every pass. A town runs
	# to a few thousand of these and `to_screen` is a `Callable`, so projecting
	# each corner once rather than per pass is the one thing here that would
	# actually cost something.
	var quads: Array[PackedVector2Array] = []
	quads.resize(buildings.size())
	for i in buildings.size():
		var b: PackedVector2Array = buildings[i]
		if b.size() < 3:
			quads[i] = PackedVector2Array()
			continue
		var q := PackedVector2Array()
		for p in b:
			q.append(to_screen.call(p))
		quads[i] = q

	# How big is a roof, on screen, right now? Sampled, and taken as the median
	# so one freak footprint cannot decide the treatment for the whole town.
	var sizes := PackedFloat32Array()
	var step: int = maxi(1, quads.size() / ROOF_SAMPLE)
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

	var want_ink := detail >= 1.0 and lot_px >= ROOF_INK_MIN_PX
	var want_detail := detail >= 1.0 and lot_px >= ROOF_DETAIL_MIN_PX

	if want_detail:
		var soff := SHADOW_OFF * m_scale
		var sh := Color(SHADOW.r, SHADOW.g, SHADOW.b, SHADOW.a * alpha)
		for q in quads:
			if q.size() < 3:
				continue
			var s := PackedVector2Array()
			for p in q:
				s.append(p + soff)
			ci.draw_colored_polygon(s, sh)

	for i in quads.size():
		var q: PackedVector2Array = quads[i]
		if q.size() < 3:
			continue
		var tone: float = tones[i] if i < tones.size() else 0.5
		var c := _roof_color(tone)
		ci.draw_colored_polygon(q, Color(c.r, c.g, c.b, alpha))

	if not want_ink:
		return

	# The ink outline, uniform weight — the pass that makes this read as drawn
	# rather than plotted. Every roof's edges go into **one** `draw_multiline`
	# rather than a `draw_polyline` each: at a few thousand footprints the
	# per-roof call was the single most expensive thing in this function (a
	# 6-town sheet took 577 ms a redraw, which is not a thing anyone can pan).
	var ink := Color(CASING.r, CASING.g, CASING.b, alpha)
	var edges := PackedVector2Array()
	for q in quads:
		var n := q.size()
		if n < 3:
			continue
		for k in n:
			edges.append(q[k])
			edges.append(q[(k + 1) % n])
	# Never wider than a third of the roof it surrounds, or the ink eats the
	# roof — this is the same failure `ROOF_INK_MIN_PX` guards, one step up.
	var lw: float = clampf(0.9 * m_scale, px_floor, maxf(px_floor, lot_px / 3.0))
	ci.draw_multiline(edges, ink, lw)

	if not want_detail:
		return

	# The ridge, straight off the engine: `buildBuildings` records the roof line
	# it laid each footprint's grammar along, and the reference strokes exactly
	# that (line 22880). One call again, for the same reason.
	var ridge: PackedVector2Array = layout.get("building_ridge", PackedVector2Array())
	if ridge.size() < 2:
		return
	var ridges := PackedVector2Array()
	for p in ridge:
		ridges.append(to_screen.call(p))
	var rw: float = clampf(0.55 * m_scale, px_floor * 0.6, maxf(px_floor, lot_px / 6.0))
	ci.draw_multiline(ridges, Color(CASING.r, CASING.g, CASING.b, alpha * 0.55), rw)


## `tone` (0..1) → one rooftop's colour.
##
## Brightness up and saturation down together, plus a slight hue drift. The
## ranges are deliberately narrow: the point is a family of weathered shades of
## one material, not a set of different materials.
static func _roof_color(tone: float) -> Color:
	var t := clampf(tone, 0.0, 1.0) * 2.0 - 1.0
	var v := clampf(ROOF_V + t * ROOF_V_SWING, 0.30, 0.86)
	var s := clampf(ROOF_S - t * ROOF_S_SWING, 0.18, 0.62)
	var h := fposmod(ROOF_H + (clampf(tone, 0.0, 1.0) - 0.5) * ROOF_H_SWING, 1.0)
	return Color.from_hsv(h, s, v)
