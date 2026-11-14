extends RefCounted
## CL-1 / CL-2 (`CITY_MAP_LOOK_SCOPE.md` section 4.2, milestones CL-1 and CL-2; Ruling EW item 3): the one teardrop
## pin, drawn and hit-tested by a single implementation that the map overlay (`map_overlay.gd`, generated
## landmarks) and the City Viewer (`city_viewer_window.gd`, derived pins) both call. A second copy of either half
## is the defect this file exists to prevent: the footprint that is DRAWN and the footprint that is PICKED must be
## the same shape, so `hit()` and `draw()` read the same `geometry()`.
##
## **Shape.** A head circle of radius `r` and a tip at the anchor, joined by the two tangent lines (a convex
## teardrop). The anchor is exactly the place; the head sits above it (`STEM_RATIO` radii up), as the reference's
## pins do. Nothing here reads the engine, a settings file or a node: everything arrives as an argument, so the same
## call draws the same pixels on the map, in the viewer and in an export.
##
## **Colour (Q6, the scope's default; the reference gives no legend).** Orange when the pin's subject has a linked
## vault note, white when it has none. `has_note` is the caller's answer; this file never asks the vault.
##
## **Pictograms** are vector strokes and fills in a unit square (x right, y down, extent about +/-1), not font glyphs:
## the settlement pin's own header notes Godot's default font lacks its dingbats. `pictogram_for()` maps a kind key
## onto one of them. **A kind with no pictogram draws the round dot, the reference's generic point marker, and says
## so in `pictogram_for()`'s doc: it is a stated fallback, never a wrong glyph.**
##
## **What it must never do:** write anything, hold state, or decide whether a click opens anything (that is
## `DccApp._on_element_clicked`, behind the armed-tool gate).
##
## Preloaded by its callers, no `class_name`, so it needs no `godot --headless --import` pass.

## Head-centre distance from the tip, in head radii. 2.1 gives the tangent lines a half-angle of about 28 degrees,
## the moderately pointed drop the reference draws (a labelled judgement read off the reference screenshot).
const STEM_RATIO := 2.1
## Orange: the reference's pin colour, sampled from the screenshot (a labelled judgement, section 2.1 of the scope).
const COL_NOTE := Color(0.941, 0.549, 0.133, 1.0)
## White: the "no linked note" state. Slightly warm so it does not glare on the dark theme.
const COL_NONE := Color(0.973, 0.965, 0.941, 1.0)
## The pictogram and outline ink. Against `COL_NOTE` this is about 6:1 and against `COL_NONE` about 17:1
## (relative luminance of #f08c21 is about 0.38, of #1b130b about 0.01), above the 4.5:1 floor for graphics.
const INK := Color(0.106, 0.075, 0.043, 1.0)
## Hit slack in the pin's own units (px), for a pointer: 2 gives "1 px outside the head picks, 4 px outside does
## not" (`CL-2`'s bar). The shipped disc pads (`HOVER_RADIUS_PAD` = 4) are looser; a pin's footprint is its shape.
const HIT_PAD := 2.0
## Touch target floor in dp (the 44 dp rule): the head plus its pad is never smaller than this across.
const TOUCH_TARGET_DP := 44.0

## Kind key to pictogram id. Settlement classes and the City Viewer's derived kinds are included so one table serves
## every caller. Keys are the engine's own (`landmark_kinds()`, `SETTLEMENT_CLASS`, the layout's worship/harbour/market
## entries); a kind absent here is `"dot"`.
const KIND_PICTOGRAM := {
	# Built places and the City Viewer's derived kinds.
	"citadel": "castle", "castle": "castle", "fort": "castle", "capital": "castle", "metropolis": "castle",
	"worship": "spire", "temple": "spire", "shrine": "spire", "sacred_grove": "spire",
	"sacred_mountain": "spire", "pilgrimage_site": "spire",
	"watchtower": "tower", "border_marker": "tower",
	"harbour": "anchor", "port": "anchor",
	"farm": "wheat", "farmland": "wheat",
	"market": "dot", "market_site": "dot", "trade_depot": "dot", "caravan_station": "dot", "road_junction": "dot",
	"well": "well",
	# Economic.
	"mine": "pick", "quarry": "pick", "salt_works": "pick", "resource_extraction_site": "pick",
	# Physical and transport.
	"peak": "peak", "ridge": "peak", "saddle": "peak", "mountain_pass": "peak", "fortified_pass": "peak",
	"cliff": "peak", "gorge": "peak", "volcanic_feature": "peak", "rock_formation": "peak", "glacial_feature": "peak",
	"cave": "cave",
	"spring": "drop", "lake": "drop", "waterfall": "drop", "delta": "drop", "river_confluence": "drop",
	"portage": "drop", "ford": "drop",
	"bridge_site": "bridge", "river_crossing": "bridge", "fortified_crossing": "bridge",
	"ancient_forest": "tree",
	"battlefield": "swords",
	# User-placed points of interest (Ruling EW item 4): the `poi` kinds with no landmark meaning of their own. `temple`,
	# `shrine`, `well` and `mine` are above and shared; `other` is the stated dot fallback.
	"tavern": "mug", "shop": "shop", "smithy": "anvil", "guild": "banner", "gate": "gate", "ruin": "ruin", "camp": "tent",
}

## The pictograms (a static var, not a const: a `PackedVector2Array(...)` is not a constant expression): `{lines: [polyline], fills: [polygon], cuts: [polygon], dots: [[centre, radius]]}` in the unit square.
## `lines` are stroked in `INK`, `fills` filled in `INK`, `cuts` filled in the head colour (a door, a hollow), `dots`
## are filled circles. Each is a labelled judgement of a small recognisable silhouette, drawn to read at 18 px.
static var PICTOGRAMS: Dictionary = {
	"castle": {
		"fills": [PackedVector2Array([Vector2(-1, 0.9), Vector2(-1, -0.6), Vector2(-0.6, -0.6), Vector2(-0.6, -0.3),
			Vector2(-0.2, -0.3), Vector2(-0.2, -0.6), Vector2(0.2, -0.6), Vector2(0.2, -0.3), Vector2(0.6, -0.3),
			Vector2(0.6, -0.6), Vector2(1, -0.6), Vector2(1, 0.9)])],
		"cuts": [PackedVector2Array([Vector2(-0.22, 0.9), Vector2(-0.22, 0.3), Vector2(0, 0.12), Vector2(0.22, 0.3),
			Vector2(0.22, 0.9)])],
	},
	"spire": {
		"fills": [PackedVector2Array([Vector2(-0.38, 0.9), Vector2(-0.38, -0.1), Vector2(0, -1), Vector2(0.38, -0.1),
			Vector2(0.38, 0.9)])],
		"cuts": [PackedVector2Array([Vector2(-0.13, 0.9), Vector2(-0.13, 0.4), Vector2(0.13, 0.4), Vector2(0.13, 0.9)])],
	},
	"tower": {
		"fills": [PackedVector2Array([Vector2(-0.45, 0.9), Vector2(-0.45, -0.4), Vector2(-0.7, -0.4), Vector2(-0.7, -0.95),
			Vector2(-0.3, -0.95), Vector2(-0.3, -0.65), Vector2(0.3, -0.65), Vector2(0.3, -0.95), Vector2(0.7, -0.95),
			Vector2(0.7, -0.4), Vector2(0.45, -0.4), Vector2(0.45, 0.9)])],
		"cuts": [PackedVector2Array([Vector2(-0.12, 0.9), Vector2(-0.12, 0.45), Vector2(0.12, 0.45), Vector2(0.12, 0.9)])],
	},
	"anchor": {
		"lines": [
			PackedVector2Array([Vector2(0, -0.55), Vector2(0, 0.85)]),
			PackedVector2Array([Vector2(-0.42, -0.25), Vector2(0.42, -0.25)]),
			PackedVector2Array([Vector2(-0.85, 0.2), Vector2(-0.6, 0.62), Vector2(-0.25, 0.85), Vector2(0, 0.9),
				Vector2(0.25, 0.85), Vector2(0.6, 0.62), Vector2(0.85, 0.2)]),
			PackedVector2Array([Vector2(0, -0.9), Vector2(0.14, -0.84), Vector2(0.18, -0.72), Vector2(0.14, -0.6),
				Vector2(0, -0.55), Vector2(-0.14, -0.6), Vector2(-0.18, -0.72), Vector2(-0.14, -0.84), Vector2(0, -0.9)]),
		],
	},
	"wheat": {
		"lines": [
			PackedVector2Array([Vector2(0, 0.95), Vector2(0, -0.75)]),
			PackedVector2Array([Vector2(0, 0.35), Vector2(-0.5, 0.0)]), PackedVector2Array([Vector2(0, 0.35), Vector2(0.5, 0.0)]),
			PackedVector2Array([Vector2(0, -0.1), Vector2(-0.5, -0.45)]), PackedVector2Array([Vector2(0, -0.1), Vector2(0.5, -0.45)]),
			PackedVector2Array([Vector2(0, -0.75), Vector2(-0.3, -1.0)]), PackedVector2Array([Vector2(0, -0.75), Vector2(0.3, -1.0)]),
		],
	},
	"pick": {
		"lines": [
			PackedVector2Array([Vector2(-0.55, 0.9), Vector2(0.4, -0.55)]),
			PackedVector2Array([Vector2(-0.75, -0.35), Vector2(-0.3, -0.8), Vector2(0.3, -0.9), Vector2(0.85, -0.45)]),
		],
	},
	"peak": {
		"fills": [PackedVector2Array([Vector2(-0.95, 0.8), Vector2(-0.15, -0.85), Vector2(0.35, -0.05), Vector2(0.6, -0.4),
			Vector2(1.0, 0.8)])],
	},
	"cave": {
		"fills": [PackedVector2Array([Vector2(-0.9, 0.8), Vector2(-0.8, -0.1), Vector2(-0.4, -0.6), Vector2(0, -0.75),
			Vector2(0.4, -0.6), Vector2(0.8, -0.1), Vector2(0.9, 0.8)])],
		"cuts": [PackedVector2Array([Vector2(-0.4, 0.8), Vector2(-0.35, 0.1), Vector2(0, -0.15), Vector2(0.35, 0.1),
			Vector2(0.4, 0.8)])],
	},
	"drop": {
		"fills": [PackedVector2Array([Vector2(0, -0.95), Vector2(0.55, 0.0), Vector2(0.62, 0.4), Vector2(0.35, 0.8),
			Vector2(0, 0.95), Vector2(-0.35, 0.8), Vector2(-0.62, 0.4), Vector2(-0.55, 0.0)])],
	},
	"bridge": {
		"lines": [
			PackedVector2Array([Vector2(-1, 0.2), Vector2(1, 0.2)]),
			PackedVector2Array([Vector2(-0.8, 0.2), Vector2(-0.6, -0.1), Vector2(-0.35, -0.32), Vector2(0, -0.4),
				Vector2(0.35, -0.32), Vector2(0.6, -0.1), Vector2(0.8, 0.2)]),
			PackedVector2Array([Vector2(-0.8, 0.2), Vector2(-0.8, 0.85)]), PackedVector2Array([Vector2(0.8, 0.2), Vector2(0.8, 0.85)]),
		],
	},
	"tree": {
		"fills": [PackedVector2Array([Vector2(0, -0.95), Vector2(0.65, 0.25), Vector2(0.2, 0.25), Vector2(0.2, 0.9),
			Vector2(-0.2, 0.9), Vector2(-0.2, 0.25), Vector2(-0.65, 0.25)])],
	},
	"swords": {
		"lines": [
			PackedVector2Array([Vector2(-0.8, -0.8), Vector2(0.8, 0.8)]), PackedVector2Array([Vector2(0.8, -0.8), Vector2(-0.8, 0.8)]),
			PackedVector2Array([Vector2(-0.5, 0.3), Vector2(-0.1, 0.7)]), PackedVector2Array([Vector2(0.5, 0.3), Vector2(0.1, 0.7)]),
		],
	},
	"well": {
		"lines": [PackedVector2Array([Vector2(0, -0.7), Vector2(0.5, -0.5), Vector2(0.7, 0), Vector2(0.5, 0.5),
			Vector2(0, 0.7), Vector2(-0.5, 0.5), Vector2(-0.7, 0), Vector2(-0.5, -0.5), Vector2(0, -0.7)])],
		"dots": [[Vector2(0, 0), 0.25]],
	},
	# User-placed points of interest (Ruling EW item 4, CL-11): the kinds `cartalith_civ::poi::PoiKind` lists that no
	# landmark kind already draws. Each is a labelled judgement of a small recognisable silhouette, drawn to read at 18 px.
	"mug": {
		"fills": [PackedVector2Array([Vector2(-0.65, -0.5), Vector2(0.4, -0.5), Vector2(0.4, 0.85), Vector2(-0.65, 0.85)])],
		"cuts": [PackedVector2Array([Vector2(-0.65, -0.12), Vector2(0.4, -0.12), Vector2(0.4, -0.02), Vector2(-0.65, -0.02)])],
		"lines": [PackedVector2Array([Vector2(0.4, -0.2), Vector2(0.85, -0.2), Vector2(0.85, 0.5), Vector2(0.4, 0.5)])],
	},
	"shop": {
		"fills": [
			PackedVector2Array([Vector2(-0.95, -0.2), Vector2(-0.65, -0.8), Vector2(0.65, -0.8), Vector2(0.95, -0.2)]),
			PackedVector2Array([Vector2(-0.75, -0.05), Vector2(0.75, -0.05), Vector2(0.75, 0.9), Vector2(-0.75, 0.9)]),
		],
		"cuts": [PackedVector2Array([Vector2(-0.25, 0.25), Vector2(0.25, 0.25), Vector2(0.25, 0.9), Vector2(-0.25, 0.9)])],
	},
	"anvil": {
		"fills": [PackedVector2Array([Vector2(-0.95, -0.45), Vector2(0.7, -0.45), Vector2(0.95, -0.15), Vector2(0.4, -0.1),
			Vector2(0.25, 0.3), Vector2(0.65, 0.85), Vector2(-0.65, 0.85), Vector2(-0.25, 0.3), Vector2(-0.4, -0.1),
			Vector2(-0.95, -0.1)])],
	},
	"banner": {
		"fills": [PackedVector2Array([Vector2(-0.6, -0.8), Vector2(0.8, -0.8), Vector2(0.45, -0.3), Vector2(0.8, 0.2),
			Vector2(-0.6, 0.2)])],
		"lines": [PackedVector2Array([Vector2(-0.6, -0.95), Vector2(-0.6, 0.95)])],
	},
	"gate": {
		"fills": [PackedVector2Array([Vector2(-0.85, 0.9), Vector2(-0.85, -0.2), Vector2(-0.5, -0.7), Vector2(0, -0.9),
			Vector2(0.5, -0.7), Vector2(0.85, -0.2), Vector2(0.85, 0.9)])],
		"cuts": [PackedVector2Array([Vector2(-0.4, 0.9), Vector2(-0.4, -0.05), Vector2(-0.2, -0.35), Vector2(0, -0.45),
			Vector2(0.2, -0.35), Vector2(0.4, -0.05), Vector2(0.4, 0.9)])],
		"lines": [PackedVector2Array([Vector2(-0.2, 0.9), Vector2(-0.2, -0.3)]), PackedVector2Array([Vector2(0.2, 0.9), Vector2(0.2, -0.3)])],
	},
	"ruin": {
		"fills": [
			PackedVector2Array([Vector2(-0.75, 0.9), Vector2(-0.75, -0.3), Vector2(-0.5, -0.45), Vector2(-0.25, -0.2), Vector2(-0.25, 0.9)]),
			PackedVector2Array([Vector2(0.1, 0.9), Vector2(0.1, 0.1), Vector2(0.4, -0.1), Vector2(0.7, 0.1), Vector2(0.7, 0.9)]),
		],
	},
	"tent": {
		"fills": [PackedVector2Array([Vector2(0, -0.85), Vector2(0.95, 0.85), Vector2(-0.95, 0.85)])],
		"cuts": [PackedVector2Array([Vector2(0, 0.05), Vector2(0.3, 0.85), Vector2(-0.3, 0.85)])],
	},
	"dot": {"dots": [[Vector2(0, 0), 0.5]]},
}


## The pictogram id for kind key `kind`. **A kind with no entry returns `"dot"`**, the reference's generic round point
## marker: stated, so a caller never mistakes the fallback for a mapped kind. Case-insensitive.
static func pictogram_for(kind: String) -> String:
	return String(KIND_PICTOGRAM.get(kind.to_lower(), "dot"))


## Whether `kind` has a mapped pictogram of its own (not the `dot` fallback). For probes and the legend.
static func is_mapped(kind: String) -> bool:
	return KIND_PICTOGRAM.has(kind.to_lower())


## The pin's geometry for tip `tip` and head radius `r`: `{head: Vector2, r, tip, poly}`. `poly` is the closed
## outline (tip, right tangent point, the arc over the top, left tangent point), convex, so it can be filled with
## `draw_colored_polygon`. The ONE definition both `draw()` and `hit()` read.
static func geometry(tip: Vector2, r: float) -> Dictionary:
	var head := tip + Vector2(0.0, -STEM_RATIO * r)
	# Angle (from +x, y down) of the right tangent point as seen from the head centre: the tangent from the tip
	# touches the circle where cos(phi) = r / d, phi measured from the downward axis.
	var phi := acos(1.0 / STEM_RATIO)
	var a_right := PI * 0.5 - phi
	var a_left := PI * 0.5 + phi
	var poly := PackedVector2Array()
	poly.append(tip)
	var steps := 20
	# From the right tangent point over the top to the left one: decreasing angle through -90 degrees.
	var a_end := a_left - TAU
	for i in steps + 1:
		var a := lerpf(a_right, a_end, float(i) / float(steps))
		poly.append(head + Vector2(cos(a), sin(a)) * r)
	return {"head": head, "r": r, "tip": tip, "poly": poly}


## The shortest distance from `p` to the segment `a`-`b`.
static func _seg_dist(p: Vector2, a: Vector2, b: Vector2) -> float:
	var ab := b - a
	var l2 := ab.length_squared()
	if l2 <= 0.0:
		return p.distance_to(a)
	return p.distance_to(a + ab * clampf((p - a).dot(ab) / l2, 0.0, 1.0))


## Distance from `p` to the pin's axis (the tip-to-head-centre segment): the one sort key `hits_at`, `_pick_mark` and
## the viewer's pick share, so a pin under the pointer ranks by how centrally it is hit, whatever part of it that is.
## 0 on the axis, growing sideways; a click on the head centre and a click on the tip both read 0.
static func axis_distance(tip: Vector2, r: float, p: Vector2) -> float:
	return _seg_dist(p, tip, tip + Vector2(0.0, -STEM_RATIO * r))


## The pad (px, in the pin's units) that makes the head plus its slack at least `TOUCH_TARGET_DP * unit` across,
## where `unit` converts dp to the pin's units (the phone's density-based scale, divided by the camera zoom on the
## map). Never below `HIT_PAD`. A mouse uses `HIT_PAD` and never this.
static func touch_pad(r: float, unit_px: float) -> float:
	return maxf(HIT_PAD, TOUCH_TARGET_DP * 0.5 * unit_px - r)


## True when `p` is on the pin: inside the head circle grown by `pad`, or within `pad` of the stem (the triangle
## from the tip to the two tangent points). The head and the stem are both covered, which a disc around the anchor
## is not (the head floats `STEM_RATIO` radii above it). `pad` is `HIT_PAD` for a pointer, `touch_pad()` for a finger.
static func hit(tip: Vector2, r: float, p: Vector2, pad: float) -> bool:
	var g := geometry(tip, r)
	var head: Vector2 = g["head"]
	# Cheap reject: farther than the whole pin's extent from the tip.
	if p.distance_to(tip) > (STEM_RATIO + 1.0) * r + pad:
		return false
	if p.distance_to(head) <= r + pad:
		return true
	var poly: PackedVector2Array = g["poly"]
	# The stem triangle is tip, the first arc point (right tangent) and the last (left tangent).
	var t1 := poly[1]
	var t2 := poly[poly.size() - 1]
	var tri := PackedVector2Array([tip, t1, t2])
	if Geometry2D.is_point_in_polygon(p, tri):
		return true
	return minf(minf(_seg_dist(p, tip, t1), _seg_dist(p, t1, t2)), _seg_dist(p, t2, tip)) <= pad


## Draws the pin onto `ci`: the teardrop in the state colour, its ink outline, and the pictogram for `kind` inside
## the head. `r` is the head radius in `ci`'s units, `has_note` picks orange or white, `w` scales the line widths
## (1.0 = a 1.3 px stroke at head radius 9), `alpha` fades the whole pin (a hover or a ghost), `outline` overrides the
## outline ink (the map's icon-treatment key; defaults to `INK`). Draws nothing for `r <= 0`.
static func draw(ci: CanvasItem, tip: Vector2, r: float, kind: String, has_note: bool, w: float = 1.0,
		alpha: float = 1.0, outline: Color = INK) -> void:
	if r <= 0.0:
		return
	var g := geometry(tip, r)
	var head: Vector2 = g["head"]
	var poly: PackedVector2Array = g["poly"]
	var fill := COL_NOTE if has_note else COL_NONE
	fill.a *= alpha
	var ink := INK
	ink.a *= alpha
	var edge := outline
	edge.a *= alpha
	ci.draw_colored_polygon(poly, fill)
	var closed := poly.duplicate()
	closed.append(poly[0])
	ci.draw_polyline(closed, edge, maxf(1.0, 1.3 * w * r / 9.0), true)
	var pic: Dictionary = PICTOGRAMS[pictogram_for(kind)]
	var s := r * 0.62
	var lw := maxf(1.0, 1.4 * w * r / 9.0)
	for f in pic.get("fills", []):
		ci.draw_colored_polygon(_xf(f, head, s), ink)
	for c in pic.get("cuts", []):
		ci.draw_colored_polygon(_xf(c, head, s), fill)
	for l in pic.get("lines", []):
		ci.draw_polyline(_xf(l, head, s), ink, lw, true)
	for d in pic.get("dots", []):
		ci.draw_circle(head + (d[0] as Vector2) * s, float(d[1]) * s, ink, true, -1.0, true)


## A pictogram polyline or polygon from the unit square to the head: scaled by `s` and moved to `head`.
static func _xf(pts: PackedVector2Array, head: Vector2, s: float) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in pts:
		out.append(head + p * s)
	return out


# -- The circle style (owner request 2026-11-14: "keep the old circle markers as an option") ---------------------------
# `DccSettings.marker_style()` is `pins` (everything above) or `circles`. A landmark in circles mode is the original class
# ring (`map_overlay.gd::_draw_landmark_ring`, restored from `d2a8b6a3^`); everything else that is drawn through this file
# (a user-placed point of interest, the City Viewer's derived markers) takes the circle form below: a disc centred on the
# anchor, in the same orange/white note state, holding the same pictogram. The footprint follows the shape: a circle's
# own radius in circles mode, the head plus stem in pins mode.

## Draws a circle marker centred on `centre`: a disc of radius `r` in the note-state colour, its outline and the pictogram
## for `kind` inside. Same arguments as `draw()` (`w` stroke scale, `alpha` fade, `outline` ink). Nothing for `r <= 0`.
static func draw_ring(ci: CanvasItem, centre: Vector2, r: float, kind: String, has_note: bool, w: float = 1.0,
		alpha: float = 1.0, outline: Color = INK) -> void:
	if r <= 0.0:
		return
	var fill := COL_NOTE if has_note else COL_NONE
	fill.a *= alpha
	var ink := INK
	ink.a *= alpha
	var edge := outline
	edge.a *= alpha
	ci.draw_circle(centre, r, fill, true, -1.0, true)
	ci.draw_arc(centre, r, 0.0, TAU, 28, edge, maxf(1.0, 1.3 * w * r / 9.0), true)
	var pic: Dictionary = PICTOGRAMS[pictogram_for(kind)]
	var s := r * 0.62
	var lw := maxf(1.0, 1.4 * w * r / 9.0)
	for f in pic.get("fills", []):
		ci.draw_colored_polygon(_xf(f, centre, s), ink)
	for c in pic.get("cuts", []):
		ci.draw_colored_polygon(_xf(c, centre, s), fill)
	for l in pic.get("lines", []):
		ci.draw_polyline(_xf(l, centre, s), ink, lw, true)
	for d in pic.get("dots", []):
		ci.draw_circle(centre + (d[0] as Vector2) * s, float(d[1]) * s, ink, true, -1.0, true)


## True when `p` is on a circle marker centred on `centre`: within `r + pad`. The circle style's whole footprint.
static func hit_ring(centre: Vector2, r: float, p: Vector2, pad: float) -> bool:
	return p.distance_to(centre) <= r + pad


## `draw()` or `draw_ring()` by style (`circles` is `DccSettings.markers_are_circles()`): the ONE branch every drawn
## marker goes through, so no caller picks a shape on its own and then picks a footprint for another.
static func draw_styled(ci: CanvasItem, tip: Vector2, r: float, kind: String, has_note: bool, circles: bool,
		w: float = 1.0, alpha: float = 1.0, outline: Color = INK) -> void:
	if circles:
		draw_ring(ci, tip, r, kind, has_note, w, alpha, outline)
	else:
		draw(ci, tip, r, kind, has_note, w, alpha, outline)


## `hit()` or `hit_ring()` by style: the footprint that goes with `draw_styled()`'s shape, read by every pick.
static func hit_styled(tip: Vector2, r: float, p: Vector2, pad: float, circles: bool) -> bool:
	return hit_ring(tip, r, p, pad) if circles else hit(tip, r, p, pad)


## The sort key of a hit by style: the distance to the circle's centre in circles mode, `axis_distance()` for a pin.
static func rank_distance(tip: Vector2, r: float, p: Vector2, circles: bool) -> float:
	return p.distance_to(tip) if circles else axis_distance(tip, r, p)
