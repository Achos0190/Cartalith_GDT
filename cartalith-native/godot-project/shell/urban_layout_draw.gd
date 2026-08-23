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
## **Two of the reference's own layers are absent, and the absence is the
## point.** `_umDrawLayout` also fills `model.blocks` (the opaque urban ground
## between the streets) and `model.buildings` (roofs with a ridge line), and
## strokes `model.wall`. None of those exist: blocks are milestone 12,
## buildings milestone 13, the wall circuit milestone 10
## (`URBAN_MORPHOLOGY_SCOPE.md`). Nothing here substitutes for them —
## `Graph.extract_faces()` is real and would give *polygons*, but
## `buildBlocks` filters, insets and marks plazas, so drawing raw faces as
## blocks would be inventing a stage rather than drawing one.
##
## What is drawn is exactly what milestones 5-7 produce: the site's water, the
## street graph by class, the market anchor, and (on request) the approach-road
## endpoints the town was grown toward.

## `_umDrawLayout`'s own palette, RGB for RGB — the reference's 8-bit values
## in the comment beside each, written as float components because `Color8()`
## is a runtime call and these are `const`.
##
## Theme-independent on purpose, the same reasoning `map_overlay.gd`'s
## `FACTION_COLORS` already records: this is data-driven map content (a
## generated town's own streets and water), not UI chrome.
const WATER := Color(0.361, 0.510, 0.675)        # rgb(92,130,172)
const CASING := Color(0.306, 0.259, 0.188)       # rgb(78,66,48)
const FILL_PRIMARY := Color(0.878, 0.824, 0.675) # rgb(224,210,172)
const FILL_OTHER := Color(0.808, 0.761, 0.643)   # rgb(206,194,164)
## Not in the reference's palette: it never strokes the centreline separately.
## See `draw_layout()` for why a coastal site needs it.
const RIVER_LINE := Color(0.275, 0.408, 0.580)
## Likewise this port's own: the reference draws no market marker because its
## own plaza/civic layers (milestones 8/14) sit there instead.
const MARKET := Color(0.588, 0.235, 0.157)
const ROUTE_END := Color(0.471, 0.408, 0.290)
## `_umDrawLayoutPreview`'s land ground, rgb(150,168,120); only the fit-to-box
## viewer paints one (the map already has real terrain underneath).
const GROUND := Color(0.588, 0.659, 0.471)

## `wFill` (reference line 22811) — the base stroke width per street class, in
## model metres. `ringroad`/`quay` are in the reference's table and are omitted
## here because milestones 1-7 cannot produce either: `ringroad` needs
## `supersedeWall` to demolish a circuit (milestone 10) and `quay` needs
## `buildHarbour` (milestone 9).
const W_FILL := {"primary": 6.0, "street": 3.2, "lane": 1.8}
## `order` (reference line 22821), minus the two classes above.
const DRAW_ORDER: Array[String] = ["lane", "street", "primary"]


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
static func draw_layout(ci: CanvasItem, layout: Dictionary, to_screen: Callable,
		m_scale: float, px_floor: float, alpha: float, show_route_ends: bool) -> void:
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

	# Streets: casing (dark, wider) then fill (light), so the network reads as
	# continuous lines rather than loose segments. Both passes walk the classes
	# in the reference's own order so a primary always sits on top.
	var streets: Dictionary = layout.get("streets", {})
	for pass_case in [true, false]:
		for cls in DRAW_ORDER:
			var segs: PackedVector2Array = streets.get(cls, PackedVector2Array())
			if segs.size() < 2:
				continue
			var base: float = W_FILL.get(cls, 3.0)
			var width: float = maxf(px_floor, (base + 2.4) * m_scale) if pass_case \
				else maxf(px_floor * 0.5, base * m_scale)
			var color: Color = CASING if pass_case \
				else (FILL_PRIMARY if cls == "primary" else FILL_OTHER)
			var screen := PackedVector2Array()
			for p in segs:
				screen.append(to_screen.call(p))
			ci.draw_multiline(screen, tint.call(color), width)

	if show_route_ends:
		var ends: PackedVector2Array = layout.get("route_ends", PackedVector2Array())
		for p in ends:
			ci.draw_circle(to_screen.call(p), maxf(px_floor * 1.5, 4.0 * m_scale), tint.call(ROUTE_END))

	# The market anchor. `anchors.market` is the single most-read value the
	# engine produces (twenty-odd call sites downstream), and on the map it is
	# the point the whole layout is pinned to the settlement by, so it is worth
	# seeing where it landed.
	var market: Vector2 = layout.get("market", Vector2.ZERO)
	ci.draw_circle(to_screen.call(market), maxf(px_floor * 2.0, 6.0 * m_scale), tint.call(MARKET))
