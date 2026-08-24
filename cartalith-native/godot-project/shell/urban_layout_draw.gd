extends RefCounted
class_name UrbanLayoutDraw

## The one place a `urban_layouts()` dictionary is turned into strokes.
##
## Ported from the reference's `_umDrawLayout` (HTML line 22774) and
## `_umDrawLayoutPreview` (line 22901) — which are the same drawing twice,
## differing only in the metres→screen transform they receive. So this is one
## function taking that transform as a `Callable`: `map_overlay.gd` passes its
## rotate-about-the-market-then-project-to-grid mapping, and
## `city_viewer_window.gd` passes its fit-to-box one.
##
## ## What is drawn, and what still is not
##
## Milestone 12 added `buildBlocks`/`buildParcels`, so two of the reference's
## own layers are now real: `model.blocks` (the opaque urban ground between the
## streets) and the lots platted inside them. `model.buildings` and
## `model.wall` are still milestones 13 and 10 and are **not** substituted for
## — see `_draw_roofs()` for exactly what a "rooftop" is here and what it is
## not.
##
## ## The visual treatment
##
## The reference draws a flat technical plan. This draws an ink-outlined one,
## following the technique the owner's reference image is built on: every roof
## takes a slightly different brightness and saturation of one warm palette, so
## the town reads as weathered rather than uniform. The variation is **not**
## random per frame — it comes from the engine's own per-parcel `tone`, which
## is stable for a given settlement and different between settlements.
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
## own plaza/civic layers (milestones 8/14) sit there instead. This is the
## shell's `accent`, deliberately -- it is an annotation, not map ink.
const MARKET := Color(0.878, 0.639, 0.290)       # #e0a34a
const ROUTE_END := Color(0.729, 0.545, 0.271)
## The unbuilt ground. Muted rather than paper-white: this is a document shown
## inside a dark tool window, and a bright sheet would glare.
const GROUND := Color(0.761, 0.702, 0.580)       # rgb(194,179,148)
## The built interior of a block, a shade under the parchment.
const BLOCK_GROUND := Color(0.671, 0.604, 0.478) # rgb(171,154,122)

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

## How far each rooftop is inset from its parcel toward the parcel's centroid.
## This is the gap between neighbouring roofs, and it is a **drawing** choice,
## not a generated setback: a parcel is a lot, and lots tile their block edge
## to edge. Without it the roofs would fuse into one mass with lines on it.
const ROOF_INSET := 0.16
## The drop shadow, in model metres, down and to the right -- one light source
## for the whole map, from the upper left.
const SHADOW_OFF := Vector2(1.1, 1.4)
const SHADOW := Color(0.169, 0.129, 0.094, 0.30)

## `wFill` (reference line 22811) — the base stroke width per street class, in
## model metres. `ringroad`/`quay` are in the reference's table and are omitted
## here because they cannot be produced yet: `ringroad` needs `supersedeWall`
## to demolish a circuit (milestone 10) and `quay` needs `buildHarbour`
## (milestone 9).
const W_FILL := {"primary": 6.0, "street": 3.2, "lane": 1.8}
## `order` (reference line 22821), minus the two classes above.
const DRAW_ORDER: Array[String] = ["lane", "street", "primary"]
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
## per-roof passes (shadow, outline, ridge) are skipped and the roofs drawn as
## flat fills, because at map zoom a town is a few hundred pixels across and
## three extra passes over ~2000 lots buys nothing a viewer can see. The City
## Viewer passes 1.0.
static func draw_layout(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, show_route_ends: bool,
		detail: float = 1.0) -> void:
	if alpha <= 0.0:
		return
	var tint := func(c: Color) -> Color:
		return Color(c.r, c.g, c.b, c.a * alpha)

	# Water first, opaque, so the crossfade alpha is the only transparency --
	# the reference's own note: the town read see-through at full zoom because
	# the fills themselves were < 1 alpha.
	var water_poly: PackedVector2Array = layout.get("water_poly", PackedVector2Array())
	if water_poly.size() >= 3:
		var pts := PackedVector2Array()
		for p in water_poly:
			pts.append(to_screen.call(p))
		ci.draw_colored_polygon(pts, tint.call(WATER))

	# The river centreline. `_umDrawLayout` does not stroke this separately --
	# it relies on `waterPoly` covering it. A coastal site has no closed water
	# polygon at all (`buildSite` traces a shoreline, not a body), so without
	# this the water would simply not be visible on those sites.
	var river: PackedVector2Array = layout.get("river", PackedVector2Array())
	if river.size() >= 2 and water_poly.size() < 3:
		var rpts := PackedVector2Array()
		for p in river:
			rpts.append(to_screen.call(p))
		var rw: float = maxf(px_floor, float(layout.get("river_w", 20.0)) * m_scale)
		ci.draw_polyline(rpts, tint.call(RIVER_LINE), rw, true)

	# Block ground (milestone 12). Under the streets, so the street fills read
	# as channels cut through the built mass rather than as lines drawn on it.
	for blk: PackedVector2Array in layout.get("blocks", []) as Array:
		if blk.size() < 3:
			continue
		var bpts := PackedVector2Array()
		for p in blk:
			bpts.append(to_screen.call(p))
		ci.draw_colored_polygon(bpts, tint.call(BLOCK_GROUND))

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
			var screen := PackedVector2Array()
			for p in segs:
				screen.append(to_screen.call(p))
			ci.draw_multiline(screen, tint.call(color), width)

	_draw_roofs(ci, layout, to_screen, m_scale, px_floor, alpha, detail)

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


## A lot has to cover at least this many pixels before it is worth outlining.
## Below it the ink is wider than the roof it surrounds and a dense city
## renders as a black mass — measured, not guessed: a 4,370-parcel town fitted
## to a 900 px canvas puts a lot at ~3 px, and at that size the outline pass
## swallowed the tone variation completely.
const ROOF_INK_MIN_PX := 4.5
## And this many before the ridge and the drop shadow earn their passes.
const ROOF_DETAIL_MIN_PX := 9.0
## How many lots to sample when measuring that. The answer only picks between
## three treatments, so a sample is as good as a census and is O(1) per redraw
## rather than O(parcels).
const ROOF_SAMPLE := 24


## One rooftop per parcel.
##
## **A parcel is a lot, not a building.** `buildBuildings` is milestone 13 and
## does not exist; what it would add is a footprint *inside* the lot with its
## own grammar (burgage rows, courtyards, a ridge orientation per district) and
## a terrain-suitability gate that leaves some lots empty. Drawing the lot
## itself, inset, is a deliberate stand-in for that and is the one place in
## this file where the drawing is ahead of the generation. It is honest about
## the difference in three ways: every lot is built (a real town would have
## gaps), every roof is the same simple quad, and the City Viewer's own info
## panel says so in words.
##
## ## Why the passes are gated on measured pixels rather than on zoom
##
## A town's lot count runs from 11 (a hamlet) to 4,370 (a city), so "how big is
## a roof on screen" is not a function of zoom alone — the same canvas at the
## same fit shows one at ~40 px and the other at ~3 px. The ink outline is what
## makes this read as drawn, and it is also what destroys it when a roof is
## smaller than the line around it. So the treatment is chosen from the
## *measured* size of a sampled lot, and `detail` only caps it (the map overlay
## passes 0.0 to force flat fills at map zoom regardless).
static func _draw_roofs(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, detail: float) -> void:
	var parcels: Array = layout.get("parcels", [])
	if parcels.is_empty():
		return
	var tones: PackedFloat32Array = layout.get("parcel_tone", PackedFloat32Array())

	# Screen-space quads, built once and reused by every pass. A town runs to a
	# few thousand lots and `to_screen` is a `Callable`, so projecting each
	# corner once rather than per pass is the one thing here that would
	# actually cost something.
	var quads: Array[PackedVector2Array] = []
	quads.resize(parcels.size())
	for i in parcels.size():
		var par: PackedVector2Array = parcels[i]
		if par.size() < 3:
			quads[i] = PackedVector2Array()
			continue
		# Inset toward the centroid, in model space, before projecting: the gap
		# between roofs is a distance on the ground, not on the screen, so it
		# has to shrink with zoom like everything else.
		var c := Vector2.ZERO
		for p in par:
			c += p
		c /= float(par.size())
		var q := PackedVector2Array()
		for p in par:
			q.append(to_screen.call(p.lerp(c, ROOF_INSET)))
		quads[i] = q

	# How big is a lot, on screen, right now? Sampled, and taken as the median
	# so one freak parcel cannot decide the treatment for the whole town.
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
	# rather than a `draw_polyline` each: at a few thousand lots the per-roof
	# call was the single most expensive thing in this function (a 6-town sheet
	# took 577 ms a redraw, which is not a thing anyone can pan).
	var ink := Color(CASING.r, CASING.g, CASING.b, alpha)
	var edges := PackedVector2Array()
	for q in quads:
		var n := q.size()
		if n < 3:
			continue
		for k in n:
			edges.append(q[k])
			edges.append(q[(k + 1) % n])
	# Never wider than a third of the lot it surrounds, or the ink eats the
	# roof — this is the same failure `ROOF_INK_MIN_PX` guards, one step up.
	var lw: float = clampf(0.9 * m_scale, px_floor, maxf(px_floor, lot_px / 3.0))
	ci.draw_multiline(edges, ink, lw)

	if not want_detail:
		return

	# The ridge: the midline between the lot's street frontage and its back.
	# The engine hands the quad in the reference's own `[P0, P1, Q1, Q0]`
	# winding precisely so this is two midpoints and no geometry. One call
	# again, for the same reason.
	var ridges := PackedVector2Array()
	for q in quads:
		if q.size() != 4:
			continue
		ridges.append((q[0] + q[3]) * 0.5)
		ridges.append((q[1] + q[2]) * 0.5)
	if ridges.is_empty():
		return
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
