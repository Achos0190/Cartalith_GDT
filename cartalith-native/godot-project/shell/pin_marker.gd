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
## **Icon sets (owner, 2026-11-15, *"Can you use these and cut them correctly as options for the teardrops?"*).** Two more
## Marker styles draw the owner's cut teardrop PNGs (`icons/pins/{glyph,painted}/`, README there) in place of the drawn
## pin: `icon_set` is `"glyph"` or `"painted"` (`DccSettings.marker_icon_set()`), `""` is the drawn pin. A kind with no
## icon (`KIND_ICON`, one table) falls back to the drawn pin, so a map can mix the two and `shape()` says which a kind is.
## The texture is anchored at its TIP, rotated about it for Ruling FA's tilt, and **picked through the same shape**
## (`shape()` feeds `hit()`, `axis_distance()` and `tilt_angles()`), so the footprint that is drawn is the footprint that is
## hit.
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
	# Settlement tiers (Ruling FF item 1: the map's towns and villages are pins). `capital`/`metropolis` above; the rest read
	# smallest to largest: a dot (hamlet), a hut (village), a house (town), a skyline (city).
	"city": "skyline", "town": "house", "village": "hut", "hamlet": "dot",
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
	# Settlement tiers (Ruling FF item 1), labelled judgements of three silhouettes that read at 18 px.
	"skyline": {
		"fills": [PackedVector2Array([Vector2(-0.95, 0.9), Vector2(-0.95, -0.1), Vector2(-0.45, -0.1), Vector2(-0.45, -0.85),
			Vector2(0.1, -0.85), Vector2(0.1, 0.1), Vector2(0.45, 0.1), Vector2(0.45, -0.35), Vector2(0.95, -0.35),
			Vector2(0.95, 0.9)])],
	},
	"house": {
		"fills": [PackedVector2Array([Vector2(-0.8, 0.9), Vector2(-0.8, -0.05), Vector2(0, -0.8), Vector2(0.8, -0.05),
			Vector2(0.8, 0.9)])],
		"cuts": [PackedVector2Array([Vector2(-0.2, 0.9), Vector2(-0.2, 0.3), Vector2(0.2, 0.3), Vector2(0.2, 0.9)])],
	},
	"hut": {
		"fills": [PackedVector2Array([Vector2(-0.6, 0.8), Vector2(-0.6, 0.1), Vector2(0, -0.5), Vector2(0.6, 0.1),
			Vector2(0.6, 0.8)])],
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


## The pin's geometry for tip `tip`, head radius `r` and tilt `angle`: `{head: Vector2, r, tip, poly, angle}`. `poly` is the
## closed outline (tip, right tangent point, the arc over the top, left tangent point), convex, so it can be filled with
## `draw_colored_polygon`. The ONE definition both `draw()` and `hit()` read.
##
## **`angle` (radians, clockwise on screen, 0 = upright) rotates the whole pin about its TIP** (Ruling FA): the tip is the
## place and never moves; the stem and head swing about it. Everything is built upright and rotated last, so the drawn
## outline, the head centre and the hit footprint are one rotation of one shape and cannot disagree.
## `stem` is the head-centre distance from the tip in head radii: `STEM_RATIO` for the drawn pin, an icon's own ratio
## (`shape()`) for a textured one, so the hit footprint of either is its own circle plus tangent triangle.
static func geometry(tip: Vector2, r: float, angle: float = 0.0, stem: float = STEM_RATIO) -> Dictionary:
	var head := tip + Vector2(0.0, -stem * r)
	# Angle (from +x, y down) of the right tangent point as seen from the head centre: the tangent from the tip
	# touches the circle where cos(phi) = r / d, phi measured from the downward axis.
	var phi := acos(1.0 / stem)
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
	if angle != 0.0:
		head = tip + (head - tip).rotated(angle)
		for k in poly.size():
			poly[k] = tip + (poly[k] - tip).rotated(angle)
	return {"head": head, "r": r, "tip": tip, "poly": poly, "angle": angle}


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
static func axis_distance(tip: Vector2, r: float, p: Vector2, angle: float = 0.0, stem: float = STEM_RATIO) -> float:
	return _seg_dist(p, tip, tip + Vector2(0.0, -stem * r).rotated(angle))


## pin rings (FA follow-up): where the pin's own point stands when it is lifted `lift` px out along its tilt axis (the unit vector
## `(sin angle, -cos angle)`, the way an upright pin's stem points rotated clockwise by `angle`). `lift == 0` returns `tip`.
## The ONE place the lift is applied, so what is drawn (`draw`), picked (`hit_styled`, `rank_distance`) and measured by a probe agree.
static func lifted_tip(tip: Vector2, angle: float, lift: float) -> Vector2:
	if lift == 0.0:
		return tip
	return tip + Vector2(sin(angle), -cos(angle)) * lift


## pin rings (FA follow-up): the stalk of an outer-ring pin: the place's dot and a thin line out to the pin's point (`lifted_tip`), the
## circles' leader's look (`draw_leader`). Called in the caller's pass that runs BEFORE every pin is drawn, so every head
## covers every stalk. Draws nothing for `lift <= 0`. `true_tip` is the place; `w` and `r` as `draw_leader`.
static func draw_stalk(ci: CanvasItem, true_tip: Vector2, angle: float, lift: float, r: float, w: float = 1.0) -> void:
	if lift > 0.0:
		draw_leader(ci, true_tip, lifted_tip(true_tip, angle, lift), r, w)


## The pad (px, in the pin's units) that makes the head plus its slack at least `TOUCH_TARGET_DP * unit` across,
## where `unit` converts dp to the pin's units (the phone's density-based scale, divided by the camera zoom on the
## map). Never below `HIT_PAD`. A mouse uses `HIT_PAD` and never this.
static func touch_pad(r: float, unit_px: float) -> float:
	return maxf(HIT_PAD, TOUCH_TARGET_DP * 0.5 * unit_px - r)


## True when `p` is on the pin: inside the head circle grown by `pad`, or within `pad` of the stem (the triangle
## from the tip to the two tangent points). The head and the stem are both covered, which a disc around the anchor
## is not (the head floats `STEM_RATIO` radii above it). `pad` is `HIT_PAD` for a pointer, `touch_pad()` for a finger.
## `angle` is the pin's tilt about its tip (Ruling FA): the footprint is the DRAWN, rotated pin, head and stem.
static func hit(tip: Vector2, r: float, p: Vector2, pad: float, angle: float = 0.0, stem: float = STEM_RATIO) -> bool:
	var g := geometry(tip, r, angle, stem)
	var head: Vector2 = g["head"]
	# Cheap reject: farther than the whole pin's extent from the tip.
	if p.distance_to(tip) > (stem + 1.0) * r + pad:
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
## outline ink (the map's icon-treatment key; defaults to `INK`). Draws nothing for `r <= 0`. `angle` tilts the pin about
## its tip (Ruling FA); **the pictogram stays upright** inside the rotated head so it reads at any tilt. `fill_col` and
## `ink_col` (Ruling FF item 1, the map's settlement pins) override the fill and pictogram ink of a DRAWN pin; an icon set's
## texture carries its own colours and ignores both. `min_line` is the stroke floor in `ci`'s units (1.0 = one unit, right
## where a unit is a pixel); the map's settlement pins pass one SCREEN pixel (`1 / camera zoom`), since their head is small in
## local units at deep zoom and a one-unit floor would eat the head at x4.
static func draw(ci: CanvasItem, tip: Vector2, r: float, kind: String, has_note: bool, w: float = 1.0,
		alpha: float = 1.0, outline: Color = INK, angle: float = 0.0, icon_set: String = "",
		fill_col: Color = Color(0, 0, 0, 0), ink_col: Color = Color(0, 0, 0, 0), min_line: float = 1.0, lift: float = 0.0) -> void:
	if r <= 0.0:
		return
	## pin rings (FA follow-up): a pin on an OUTER ring of a big fan stands `lift` px out along its tilt axis. The stalk back to the true place
	## is NOT drawn here: `draw_stalk` is, by the caller's leader pass BEFORE every pin, so no stalk crosses a head.
	tip = lifted_tip(tip, angle, lift)
	## An icon set draws the owner's texture when `kind` has one and the file loads; otherwise it falls through to the
	## drawn pin below (the stated fallback, and exactly what `shape()` reports for the footprint).
	if icon_set != "" and draw_icon(ci, tip, r, kind, has_note, alpha, angle, icon_set):
		return
	var g := geometry(tip, r, angle)
	var head: Vector2 = g["head"]
	var poly: PackedVector2Array = g["poly"]
	var fill := COL_NOTE if has_note else COL_NONE
	## Ruling FF item 1: a settlement pin is filled with its faction's colour (`fill_col`, with the pictogram ink
	## `ink_col` chosen for contrast against it). A zero alpha means "not given" (every landmark, POI and City
	## Viewer caller), which keeps the note-state fill and the INK pictogram they always had.
	if fill_col.a > 0.0:
		fill = fill_col
	fill.a *= alpha
	var ink := INK if ink_col.a <= 0.0 else ink_col
	ink.a *= alpha
	var edge := outline
	edge.a *= alpha
	ci.draw_colored_polygon(poly, fill)
	var closed := poly.duplicate()
	closed.append(poly[0])
	ci.draw_polyline(closed, edge, maxf(min_line, 1.3 * w * r / 9.0), true)
	var pic: Dictionary = PICTOGRAMS[pictogram_for(kind)]
	var s := r * 0.62
	var lw := maxf(min_line, 1.4 * w * r / 9.0)
	for f in pic.get("fills", []):
		ci.draw_colored_polygon(_xf(f, head, s), ink)
	for c in pic.get("cuts", []):
		ci.draw_colored_polygon(_xf(c, head, s), fill)
	for l in pic.get("lines", []):
		ci.draw_polyline(_xf(l, head, s), ink, lw, true)
	for d in pic.get("dots", []):
		ci.draw_circle(head + (d[0] as Vector2) * s, float(d[1]) * s, ink, true, -1.0, true)


# -- Icon sets (owner request 2026-11-15) -----------------------------------------------------------------------------
# The cut PNGs are 128 px tall with the tip at the bottom-centre pixel and a circle head (`icons/pins/README.md`).

## The cut pins' height in source px (every file is this tall; the width differs by set, 107-110).
const ICON_H := 128.0
## The head circle and the tip distance are NOT constants: each texture's own are measured from its alpha when it loads
## (`_measure`), because the cut pins differ (the first 40 have a tip 1.37-1.41 head radii from the head centre, the painted
## extra sheet's 38 have 1.6-1.9). One constant would put the hit footprint off the picture.
## Mip-like raster heights (px) a texture is pre-scaled to; the smallest one at least as tall as the pin on screen is used,
## so a phone's 40 px pin is a downscale of the source by a good filter rather than a 3x bilinear minification (which
## shimmers). The 128 source is used above the largest bucket. Same idea as the landmark glyph rasters (`DccIcons`).
const ICON_BUCKETS := [32, 48, 64, 96, 128]
## The two icon sets (folders under `icons/pins/`).
const ICON_SETS := ["glyph", "painted"]
## Pixel offset (in head radii, upright) of the "has a note" badge from the head centre: the upper right rim.
const BADGE_OFFSET := 0.72

## **The one table: engine kind key -> the icon NAME to use when the set has no file of the kind's own name.** A set's own
## file named exactly like the kind always wins (`icon_name_for`: the painted extra sheet is named after the engine kinds, so
## shrine, fort, tomb... resolve there with no row here). These rows are for kinds whose picture is a CLOSE match to a
## differently named file: the 12 `cartalith_civ::poi::PoiKind` keys, the 52 landmark keys
## (`cartalith_civ::landmark::kinds()`), the City Viewer's derived kinds and the settlement tiers (the map's towns and villages
## draw as pins since Ruling FF item 1, CL-10). Owner, 2026-11-15, "Fort can't borrow": **nothing fortified borrows the castle
## (a fort draws its own `fort` icon where the set has one, else the drawn pin)** and no row stretches to a near miss.
## **A kind with neither its own file nor a row here draws the drawn pin.** Every row is a labelled judgement.
const KIND_ICON := {
	# Settlement tiers (painted has its own city and hamlet; glyph gets the nearest tier).
	"metropolis": "capital", "city": "town", "hamlet": "village",
	# The City Viewer's derived kinds (`citadel` is fortified: it keeps the drawn pin).
	"farm": "farmland",
	# Landmarks whose picture is a close match to a differently named file.
	"peak": "mountain", "glacial_feature": "glacier", "volcanic_feature": "volcano", "ancient_forest": "forest",
	"river_crossing": "bridge", "bridge_site": "bridge",
	"resource_extraction_site": "mine", "quarry": "mine", "salt_works": "mine",
	"market_site": "market", "trade_depot": "market", "shop": "market",
	"battlefield_historic": "battlefield", "abandoned_settlement": "ruin", "destroyed_fortress": "ruin",
}

## `{"<set>/<name>": {w: source width px, img: Image, buckets: {px: Texture2D}} or null}`; null is a remembered failure.
static var _icon_cache: Dictionary = {}


## The file name (no extension) of the icon `icon_set` has for `kind`, or `""` when the kind has none or the file will not
## load (so `shape()` and `draw()` agree: an unloadable file is the drawn pin, never a blank).
static func icon_name_for(icon_set: String, kind: String) -> String:
	if not ICON_SETS.has(icon_set):
		return ""
	var k := kind.to_lower()
	if k == "":
		return ""
	if _icon_entry(icon_set, k) != null:
		return k   # the set's own file named like the kind
	var alias := String(KIND_ICON.get(k, ""))
	if alias != "" and _icon_entry(icon_set, alias) != null:
		return alias
	return ""


## Loads (once) one pin PNG: through `load()` (the imported texture, what an export has) and, when the project has not been
## imported yet (a fresh checkout or a probe), straight from the file. Null when neither works.
static func _icon_entry(icon_set: String, name: String):
	var key := "%s/%s" % [icon_set, name]
	if _icon_cache.has(key):
		return _icon_cache[key]
	var path := "res://icons/pins/%s.png" % key
	var img: Image = null
	var res = load(path) if ResourceLoader.exists(path) else null
	if res is Texture2D:
		img = (res as Texture2D).get_image()
	if img == null:
		var disk := ProjectSettings.globalize_path(path)
		if FileAccess.file_exists(disk):
			img = Image.load_from_file(disk)
	if img == null or img.is_empty():
		_icon_cache[key] = null
		return null
	img.convert(Image.FORMAT_RGBA8)
	var m := _measure(img)
	var entry := {"w": float(img.get_width()), "img": img, "buckets": {}, "rh": m.x, "d": m.y}
	_icon_cache[key] = entry
	return entry


## `(head radius, tip-to-head-centre distance)` in the picture's own px, read from its alpha: the head radius is half the
## widest opaque row (alpha > 0.5; a teardrop's widest row is its head's diameter), the head centre is that radius below the
## topmost opaque row, and the tip is the bottom edge (the cut contract: tip at the bottom-centre pixel). Measured per
## picture so each pin's hit footprint is its own circle plus tangent triangle, not one set-wide constant.
static func _measure(img: Image) -> Vector2:
	var top := -1
	var widest := 0
	for y in img.get_height():
		var n := 0
		for x in img.get_width():
			if img.get_pixel(x, y).a > 0.5:
				n += 1
		if n > 0 and top < 0:
			top = y
		widest = maxi(widest, n)
	var rh := maxf(1.0, float(widest) * 0.5)
	return Vector2(rh, maxf(rh * 1.05, float(img.get_height()) - (float(maxi(top, 0)) + rh)))


## The texture for pin `key` at the smallest bucket at least `want_h` px tall (the source at the top): the source is
## resampled with Lanczos once per bucket and kept.
static func _icon_tex(icon_set: String, name: String, want_h: float) -> Texture2D:
	var entry = _icon_entry(icon_set, name)
	if entry == null:
		return null
	var px := int(ICON_BUCKETS[-1])
	for b: int in ICON_BUCKETS:
		if float(b) >= want_h:
			px = b
			break
	var buckets: Dictionary = entry["buckets"]
	if not buckets.has(px):
		var img: Image = (entry["img"] as Image).duplicate()
		if px != img.get_height():
			img.resize(maxi(1, roundi(img.get_width() * float(px) / float(img.get_height()))), px, Image.INTERPOLATE_LANCZOS)
		buckets[px] = ImageTexture.create_from_image(img)
	return buckets[px]


## The picture of pin `name` in `icon_set` for a UI (the pin palette's grid), at least `want_h` px tall, or null when the set
## has no file of that exact name. Public wrapper over the private cache so a menu never reaches into `_icon_tex`; no alias
## is applied (`icon_name_for` is for the map, where a near match is acceptable; a palette entry IS the file it names).
static func icon_texture(icon_set: String, name: String, want_h: float) -> Texture2D:
	if not ICON_SETS.has(icon_set) or name == "":
		return null
	return _icon_tex(icon_set, name, want_h)


## The pin shape for head radius `r` (the DRAWN pin's head radius, the size every caller already computes): `{r, stem, icon}`
## where `r` and `stem` feed `geometry()`, `hit()`, `axis_distance()` and the tilt, and `icon` is the file name or `""`.
## The drawn pin and an icon pin are the SAME HEIGHT (`(STEM_RATIO + 1) * r`), so a style change does not move the tip or
## change how tall a marker stands; the icon's head is bigger (about 1.3 r) because the cut pins are fatter (stem ratio
## 1.4 to 1.9 measured per picture, against 2.1).
static func shape(r: float, kind: String, icon_set: String = "") -> Dictionary:
	if icon_set != "":
		var name := icon_name_for(icon_set, kind)
		if name != "":
			var e: Dictionary = _icon_entry(icon_set, name)
			var h := (STEM_RATIO + 1.0) * r
			var rh: float = float(e["rh"]) * h / ICON_H
			return {"r": rh, "stem": float(e["d"]) / float(e["rh"]), "icon": name}
	return {"r": r, "stem": STEM_RATIO, "icon": ""}


## Draws the textured pin for `kind` (true) or nothing (false: the caller draws the drawn pin). The texture's tip is at
## `tip` and it is rotated about the tip by `angle` (Ruling FA), through `draw_set_transform` (reset to identity after,
## which every caller of this file expects of its canvas). A linked note shows as a small orange badge on the head's upper
## rim (the icons carry their own colours, so the drawn pin's orange/white fill cannot show it): `COL_NOTE` with an `INK`
## ring, a labelled judgement.
static func draw_icon(ci: CanvasItem, tip: Vector2, r: float, kind: String, has_note: bool, alpha: float, angle: float,
		icon_set: String) -> bool:
	var sh := shape(r, kind, icon_set)
	var name := String(sh["icon"])
	if name == "":
		return false
	var h := (STEM_RATIO + 1.0) * r
	var sc := ci.get_screen_transform().get_scale().y if ci.is_inside_tree() else 1.0
	var tex := _icon_tex(icon_set, name, h * maxf(sc, 0.001))
	if tex == null:
		return false
	var w: float = float((_icon_entry(icon_set, name) as Dictionary)["w"]) * h / ICON_H
	ci.draw_set_transform(tip, angle, Vector2.ONE)
	ci.draw_texture_rect(tex, Rect2(Vector2(-w * 0.5, -h), Vector2(w, h)), false, Color(1, 1, 1, alpha))
	ci.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	if has_note:
		var rh: float = sh["r"]
		var head := tip + (Vector2(0.0, -float(sh["stem"]) * rh)).rotated(angle)
		var at := head + (Vector2(BADGE_OFFSET, -BADGE_OFFSET) * rh).rotated(angle)
		var br := maxf(2.0, 0.26 * rh)
		var ink := INK
		ink.a *= alpha
		var fill := COL_NOTE
		fill.a *= alpha
		ci.draw_circle(at, br + maxf(1.0, 0.07 * rh), ink, true, -1.0, true)
		ci.draw_circle(at, br, fill, true, -1.0, true)
	return true


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
## `angle` is the pin's tilt (ignored by a circle: a disc has no stem to rotate).
static func draw_styled(ci: CanvasItem, tip: Vector2, r: float, kind: String, has_note: bool, circles: bool,
		w: float = 1.0, alpha: float = 1.0, outline: Color = INK, angle: float = 0.0, icon_set: String = "", lift: float = 0.0) -> void:
	if circles:
		draw_ring(ci, tip, r, kind, has_note, w, alpha, outline)
	else:
		draw(ci, tip, r, kind, has_note, w, alpha, outline, angle, icon_set, Color(0, 0, 0, 0), Color(0, 0, 0, 0), 1.0, lift)


## `hit()` or `hit_ring()` by style: the footprint that goes with `draw_styled()`'s shape, read by every pick.
## `kind` and `icon_set` select the footprint of a textured pin (`shape()`): the drawn pin's for `icon_set == ""` or a kind with
## no icon, the icon's own circle-plus-triangle otherwise. A circle ignores both.
static func hit_styled(tip: Vector2, r: float, p: Vector2, pad: float, circles: bool, angle: float = 0.0,
		kind: String = "", icon_set: String = "", lift: float = 0.0) -> bool:
	if circles:
		return hit_ring(tip, r, p, pad)
	var sh := shape(r, kind, icon_set)
	return hit(lifted_tip(tip, angle, lift), float(sh["r"]), p, pad, angle, float(sh["stem"]))


## The sort key of a hit by style: the distance to the circle's centre in circles mode, `axis_distance()` for a pin.
static func rank_distance(tip: Vector2, r: float, p: Vector2, circles: bool, angle: float = 0.0,
		kind: String = "", icon_set: String = "", lift: float = 0.0) -> float:
	if circles:
		return p.distance_to(tip)
	var sh := shape(r, kind, icon_set)
	return axis_distance(lifted_tip(tip, angle, lift), float(sh["r"]), p, angle, float(sh["stem"]))


# -- Head-overlap handling (CL-1's last bar, `CITY_MAP_LOOK_SCOPE.md` section 4.2; PINS REPLACED BY RULING FA) -------------
#
# Markers that stand close together at the current zoom hide one another's heads and leave all but one unclickable.
# **Pins (Ruling FA, owner 2026-11-15):** a cluster of overlapping pins keeps every TIP on its exact place and TILTS about
# the tip (`tilt_angles`): two make a heart (+/-45 degrees, stems meeting at 90), three or more a fan. CL-1's earlier
# head-slide with leaders is gone for pins: a slid head no longer pointed at its place.
# **Circles (`DccSettings.marker_style() == "circles"`, unchanged by the ruling):** a disc has no stem to tilt, so the
# screen-space SLIDE below (`fan_offsets`, a leader to the true point) still serves it.
#
# **Why neither can jitter while panning.** A pan translates every marker by the same vector. Both read only the markers'
# positions RELATIVE to one another (cluster membership is a distance test, the order is by relative x then y then the
# caller's stable `id`), so their results are the same for every pan and only a zoom or a data change can alter them. They
# must never read the screen position itself, a clock or a random number.

## Two heads overlap enough to need handling when their centres are closer than this share of the sum of their radii
## (0.75 = overlapping by more than a quarter, the scope's CL-1 bar "0 overlaps by more than 25 %"). A labelled judgement.
const FAN_OVERLAP_KEEP := 0.75
## Circles only: where a slid disc ends up, its neighbours' centres at least this share of the radii sum apart (a 5 % gap).
const FAN_CLEAR := 1.05
## The largest CIRCLES cluster that is handled (pin rings (FA follow-up): pins use `FAN_PIN_MAX`). A bigger knot (the whole-world view of a
## dense landmark field) is left in place: a fan of 40 would cover the map, and zooming in resolves it. Stated, not silent.
const FAN_MAX := 18
## Ruling FA: the tilt of each pin of an overlapping PAIR, about its own tip, in degrees. 45 each side makes the two stems
## 90 degrees apart, the owner's "straight line is at a 90 degree angle ... a heart like shape". Quoted from the ruling.
const TILT_PAIR_DEG := 45.0
## Ruling FA: the TOTAL spread of a fan of three, in degrees (+/-60 about the middle pin). The ruling asks for "a fan like
## shape" and gives no figure: this is a labelled judgement. Its basis is geometry: two pins on one tip keep their heads
## apart when their stems are at least 2*asin(1/STEM_RATIO) = 56.8 degrees apart, and 60 (the gap this gives) clears it.
const TILT_FAN3_SPREAD_DEG := 120.0
## Ruling FA: each pin beyond three widens the fan's total spread by this many degrees, keeping that 60 degree gap (a
## labelled judgement) ...
const TILT_FAN_STEP_DEG := 60.0
## ... up to this total (a half circle: a fan wider than that would point stems at the ground). A labelled judgement. So
## four pins on one tip still clear one another's heads (+/-90, +/-30). **From five, the rest stand on outer rings
## (pin rings (FA follow-up), `FAN_RING_STEP` below), which replaced the old fan-of-cards overlap**; this constant is every ring's half circle.
const TILT_FAN_MAX_DEG := 180.0


## pin rings (FA follow-up) (owner-delegated, 2026-11-16; `CITY_MAP_LOOK_SCOPE.md` build note): a fan of MORE than four pins on one place
## no longer overlaps like a hand of cards. Pins beyond the first four stand on OUTER RINGS: the same pin, lifted
## `FAN_RING_STEP` head diameters further out along its tilt axis per ring, on a thin stalk with the place's dot at the foot
## (`draw_leader`). Four rings hold `FAN_PIN_MAX` pins, which also replaces the old "a knot over 18 stays upright".
const FAN_RING_STEP := 2.3
## Rings of the fan, innermost first (ring 0 is the Ruling FA fan: no lift). A labelled judgement: four rings is a stalk about
## nine head radii long at most, which the map's pin size keeps under 100 px; a fifth would be a sunburst across the map.
const FAN_RINGS := 4
## The largest cluster that fans: the capacity of the four rings of a pin of the default shape (4 + 7 + 11 + 14, each ring a
## half circle with heads `FAN_CLEAR` apart). A bigger knot stays upright (zoom resolves it; the declutter thins it first).
const FAN_PIN_MAX := 36


## Per-item pin geometry the layout reads: `{rh: head radius, stem: tip-to-head-centre px}` from `shape()`.
static func _pin_dims(it: Dictionary, icon_set: String) -> Dictionary:
	var sh := shape(float(it["r"]), String(it.get("kind", "")), icon_set)
	var rh := float(sh["r"])
	return {"rh": rh, "stem": float(sh["stem"]) * rh}


## The layout of every overlapping pin, as `{id: Vector2(angle in radians, lift in px)}` holding ONLY pins that move (a lone
## pin is absent). `items` are `{id: int, pos: Vector2, r: float, kind?: String}`: the stable key, the TIP in screen px, the head
## radius. **Tips are never moved**: a pin rotates about `pos` and, on an outer ring, stands `lift` px out on a stalk from it.
##
## **Clusters.** The transitive closure of "heads overlap by more than a quarter, or one head sits on the other's stem"
## (`_pins_collide`). **A fan can reach a neighbouring cluster** (the old widening passes were removed with Ruling FA): so the
## clusters are laid out, every pin's FANNED head is compared with the other clusters' heads, and two clusters whose
## fanned heads would overlap are merged and laid out again, until none do or the merge would pass `FAN_PIN_MAX`.
##
## **Inside a cluster** (members ordered left to right by tip x, then y, then `id`): two pins make a heart (+/-45 degrees),
## three a fan (+/-60 and 0), four a half circle (+/-90, +/-30): Ruling FA's look, unchanged. From five, ring 0 keeps those four
## and the rest take ring 1 (up to 7), ring 2 (11), ring 3 (14); a ring's slots are `max(clear step, 180 / n)` degrees apart
## across a half circle, where the clear step is `2 asin(FAN_CLEAR / S)` for its stem `S` in head radii, so two heads of one
## ring never overlap. Members take the slots in angle order (ties: inner ring first), so a pin right of another leans right.
## Pure, deterministic and translation-invariant (it reads only relative tip positions).
static func fan_layout(items: Array, icon_set: String = "") -> Dictionary:
	var n := items.size()
	var out := {}
	if n < 2:
		return out
	var dims: Array = []
	for it in items:
		dims.append(_pin_dims(it, icon_set))
	var parent := PackedInt32Array()
	parent.resize(n)
	for i in n:
		parent[i] = i
	var any := false
	for i in n:
		for j in range(i + 1, n):
			if _pins_collide(items[i], items[j], icon_set):
				_union(parent, i, j)
				any = true
	if not any:
		return out
	var lay := {}
	for _pass in 8:
		var groups := _groups(parent)
		lay = {}
		var csize := {}   ## cluster root -> member count, kept current as clusters merge in this pass
		for root in groups:
			var members: Array = groups[root]
			csize[root] = members.size()
			if members.size() >= 2 and members.size() <= FAN_PIN_MAX:
				lay.merge(_ring_layout(items, dims, members), true)
		var merged := false
		# Sweep the heads by x: only pins within the largest possible clash distance of one another can overlap.
		var heads: Array = []
		var rmax := 0.0
		for i in n:
			var l: Vector2 = lay.get(i, Vector2.ZERO)
			var head := lifted_tip(items[i]["pos"] as Vector2, l.x, l.y) + Vector2(0.0, -float(dims[i]["stem"])).rotated(l.x)
			heads.append(head)
			rmax = maxf(rmax, float(dims[i]["rh"]))
		var order: Array = range(n)
		order.sort_custom(func(a, b) -> bool: return (heads[a] as Vector2).x < (heads[b] as Vector2).x)
		for oi in n:
			var a: int = order[oi]
			for oj in range(oi + 1, n):
				var b: int = order[oj]
				if (heads[b] as Vector2).x - (heads[a] as Vector2).x >= FAN_OVERLAP_KEEP * 2.0 * rmax:
					break
				var ra := _root(parent, a)
				var rb := _root(parent, b)
				if ra == rb:
					continue
				if (heads[a] as Vector2).distance_to(heads[b]) < FAN_OVERLAP_KEEP * (float(dims[a]["rh"]) + float(dims[b]["rh"])):
					if int(csize[ra]) + int(csize[rb]) <= FAN_PIN_MAX:
						_union(parent, a, b)
						csize[mini(ra, rb)] = int(csize[ra]) + int(csize[rb])
						merged = true
		if not merged:
			break
	for i in n:
		if lay.has(i):
			out[int(items[i]["id"])] = lay[i]
	return out


static func _root(parent: PackedInt32Array, i: int) -> int:
	var a := i
	while parent[a] != a:
		a = parent[a]
	return a


static func _union(parent: PackedInt32Array, i: int, j: int) -> void:
	var a := _root(parent, i)
	var b := _root(parent, j)
	if a != b:
		parent[maxi(a, b)] = mini(a, b)


## `{root: [member indices]}` of a union-find forest.
static func _groups(parent: PackedInt32Array) -> Dictionary:
	var groups := {}
	for i in parent.size():
		var r := _root(parent, i)
		if not groups.has(r):
			groups[r] = []
		(groups[r] as Array).append(i)
	return groups


## One cluster's rings, `{item index: Vector2(angle, lift)}` for every member (see `fan_layout`).
static func _ring_layout(items: Array, dims: Array, members: Array) -> Dictionary:
	members = members.duplicate()
	members.sort_custom(func(a, b) -> bool:
		var pa := items[a]["pos"] as Vector2
		var pb := items[b]["pos"] as Vector2
		# A 1e-3 px resolution so a sub-pixel wobble cannot reorder two members; `id` decides a tie.
		if absf(pa.x - pb.x) > 1e-3:
			return pa.x < pb.x
		if absf(pa.y - pb.y) > 1e-3:
			return pa.y < pb.y
		return int(items[a]["id"]) < int(items[b]["id"]))
	var m := members.size()
	var out := {}
	if m == 2 or m == 3 or m == 4:
		var spread := TILT_PAIR_DEG * 2.0 if m == 2 else minf(TILT_FAN3_SPREAD_DEG + TILT_FAN_STEP_DEG * float(m - 3), TILT_FAN_MAX_DEG)
		for k in m:
			out[members[k]] = Vector2(deg_to_rad(-spread * 0.5 + spread * float(k) / float(m - 1)), 0.0)
		return out
	var u := 0.0
	var stem := 0.0
	for i in members:
		u = maxf(u, float(dims[i]["rh"]))
		stem = maxf(stem, float(dims[i]["stem"]))
	var slots: Array = []   ## [angle degrees, ring]
	var left := m
	for k in FAN_RINGS:
		if left <= 0:
			break
		var s_k := (stem + float(k) * FAN_RING_STEP * u) / u   ## this ring's stem in head radii
		var step_min := rad2deg_(2.0 * asin(minf(1.0, FAN_CLEAR / s_k)))
		var cap := maxi(2, int(floor(TILT_FAN_MAX_DEG / step_min + 1e-6)) + 1)
		var cnt := mini(left, cap) if k < FAN_RINGS - 1 else left
		left -= cnt
		var step := 0.0
		if cnt > 1:
			step = minf(maxf(step_min, TILT_FAN_MAX_DEG / float(cnt)), 90.0)
			if step * float(cnt - 1) > TILT_FAN_MAX_DEG:
				step = TILT_FAN_MAX_DEG / float(cnt - 1)
		for j in cnt:
			slots.append([(float(j) - float(cnt - 1) * 0.5) * step, k])
	slots.sort_custom(func(a, b) -> bool:
		if absf(float(a[0]) - float(b[0])) > 1e-6:
			return float(a[0]) < float(b[0])
		return int(a[1]) < int(b[1]))
	for k in m:
		out[members[k]] = Vector2(deg_to_rad(float(slots[k][0])), float(slots[k][1]) * FAN_RING_STEP * u)
	return out


static func rad2deg_(x: float) -> float:
	return x * 180.0 / PI


## The tilt of each overlapping pin as `{id: angle in radians}`: `fan_layout`'s angle half (the shape every caller drew with
## before the outer rings; kept so a caller that has no lift to apply still reads one number per pin).
static func tilt_angles(items: Array, icon_set: String = "") -> Dictionary:
	var out := {}
	var lay := fan_layout(items, icon_set)
	for id in lay:
		out[id] = (lay[id] as Vector2).x
	return out


## The lift (px) of each pin that stands on an outer ring, `{id: px}`; ring 0 pins are absent.
static func fan_lifts(items: Array, icon_set: String = "") -> Dictionary:
	var out := {}
	var lay := fan_layout(items, icon_set)
	for id in lay:
		if (lay[id] as Vector2).y > 0.0:
			out[id] = (lay[id] as Vector2).y
	return out


## Whether two UPRIGHT pins collide: their heads overlap by more than the `FAN_OVERLAP_KEEP` bar, or one head sits on the
## other's stem (within 0.9 radius of its tip-to-head axis, a tip on a head: both would be pickable from one point).
## An item may carry `kind`: with an `icon_set`, a textured pin collides by ITS OWN head (`shape()`), not the drawn pin's.
static func _pins_collide(a: Dictionary, b: Dictionary, icon_set: String = "") -> bool:
	var sa := shape(float(a["r"]), String(a.get("kind", "")), icon_set)
	var sb := shape(float(b["r"]), String(b.get("kind", "")), icon_set)
	var ra := float(sa["r"])
	var rb := float(sb["r"])
	var ta: Vector2 = a["pos"]
	var tb: Vector2 = b["pos"]
	var ha := ta + Vector2(0.0, -float(sa["stem"]) * ra)
	var hb := tb + Vector2(0.0, -float(sb["stem"]) * rb)
	if ha.distance_to(hb) < FAN_OVERLAP_KEEP * (ra + rb):
		return true
	return axis_distance(ta, ra, hb, 0.0, float(sa["stem"])) < 0.9 * ra or axis_distance(tb, rb, ha, 0.0, float(sb["stem"])) < 0.9 * rb


## The slide for CIRCLE markers `items` (each `{id: int, pos: Vector2, r: float}`: the stable caller key, the CENTRE in
## screen px, the radius), as `{id: Vector2 offset}` holding ONLY the markers that move (a lone marker is absent). The offset
## is added to the drawn centre; the leader runs from the true `pos` to `pos + offset`. **Pins do not slide any more (Ruling
## FA): `circles == false` returns `{}`; pins use `tilt_angles`.** Pure, deterministic, translation-invariant.
static func fan_offsets(items: Array, circles: bool) -> Dictionary:
	var n := items.size()
	if n < 2 or not circles:
		return {}
	# A fanned cluster can land on a NEIGHBOUR the first pass did not link. So the link distance widens, pass by pass, until
	# no two drawn discs overlap by the bar or the passes run out, and the pass with the fewest residual overlaps wins. Every
	# pass reads only relative positions, so the result stays pan-invariant.
	var best := {}
	var best_bad := 1 << 30
	for mult in [1.0, 1.7, 2.6, 3.6]:
		var off := _fan_pass(items, FAN_OVERLAP_KEEP * mult)
		var bad := _overlaps(items, off)
		if bad < best_bad:
			best = off
			best_bad = bad
		if bad == 0:
			break
	return best


## The number of colliding CIRCLE pairs after applying `off`: two drawn discs closer than `FAN_OVERLAP_KEEP` times their
## radii sum.
static func _overlaps(items: Array, off: Dictionary) -> int:
	var rmax := 0.0
	var heads: Array = []
	for it in items:
		rmax = maxf(rmax, float(it["r"]))
		heads.append((it["pos"] as Vector2) + (off.get(it["id"], Vector2.ZERO) as Vector2))
	var cell := 2.0 * rmax
	if cell <= 0.0:
		return 0
	var grid := {}
	for i in heads.size():
		var key := Vector2i(floori((heads[i] as Vector2).x / cell), floori((heads[i] as Vector2).y / cell))
		if not grid.has(key):
			grid[key] = []
		(grid[key] as Array).append(i)
	var bad := 0
	for i in heads.size():
		var hi: Vector2 = heads[i]
		var ci := Vector2i(floori(hi.x / cell), floori(hi.y / cell))
		for dx in range(-1, 2):
			for dy in range(-1, 2):
				for j in grid.get(Vector2i(ci.x + dx, ci.y + dy), []):
					if int(j) > i and hi.distance_to(heads[j]) < FAN_OVERLAP_KEEP * (float(items[i]["r"]) + float(items[j]["r"])):
						bad += 1
	return bad


## One clustering-and-slide pass over CIRCLE markers with link threshold `keep` (a share of the radii sum).
static func _fan_pass(items: Array, keep: float) -> Dictionary:
	var out := {}
	var n := items.size()
	var rmax := 0.0
	for it in items:
		rmax = maxf(rmax, float(it["r"]))
	if rmax <= 0.0:
		return out
	# A grid of cells wide enough for the link distance finds every linked pair in the 3x3 neighbourhood.
	var cell := 2.0 * rmax * maxf(1.0, keep / FAN_OVERLAP_KEEP)
	var grid := {}
	var heads: Array = []
	for i in n:
		var h: Vector2 = items[i]["pos"]
		heads.append(h)
		var key := Vector2i(floori(h.x / cell), floori(h.y / cell))
		if not grid.has(key):
			grid[key] = []
		(grid[key] as Array).append(i)
	var parent := PackedInt32Array()
	parent.resize(n)
	for i in n:
		parent[i] = i
	for i in n:
		var hi: Vector2 = heads[i]
		var ci := Vector2i(floori(hi.x / cell), floori(hi.y / cell))
		for dx in range(-1, 2):
			for dy in range(-1, 2):
				var bucket: Array = grid.get(Vector2i(ci.x + dx, ci.y + dy), [])
				for j in bucket:
					if int(j) <= i:
						continue
					var sum_r := float(items[i]["r"]) + float(items[j]["r"])
					if hi.distance_to(heads[j]) < keep * sum_r:
						var a := i
						while parent[a] != a:
							a = parent[a]
						var b: int = int(j)
						while parent[b] != b:
							b = parent[b]
						if a != b:
							parent[maxi(a, b)] = mini(a, b)
	var clusters := {}
	for i in n:
		var root := i
		while parent[root] != root:
			root = parent[root]
		if not clusters.has(root):
			clusters[root] = []
		(clusters[root] as Array).append(i)
	for root in clusters:
		var members: Array = clusters[root]
		if members.size() < 2 or members.size() > FAN_MAX:
			continue
		out.merge(_fan_cluster(items, members), true)
	return out


## One CIRCLE cluster's slots; returns `{id: offset}` for every member. The mean of the members' true places is the centre.
## Slots on a ring (two when more than seven) whose neighbours are `FAN_CLEAR` diameters apart, members taking them in the
## angular order of their true places about the centre (ties on `id`) so the leaders do not cross.
static func _fan_cluster(items: Array, members: Array) -> Dictionary:
	var c := Vector2.ZERO
	var rmax := 0.0
	for m in members:
		c += items[m]["pos"] as Vector2
		rmax = maxf(rmax, float(items[m]["r"]))
	c /= float(members.size())
	var spacing := 2.0 * rmax * FAN_CLEAR
	var slots: Array = []
	var order: Array = members.duplicate()
	order.sort_custom(func(a, b) -> bool:
		var aa := ((items[a]["pos"] as Vector2) - c).angle()
		var bb := ((items[b]["pos"] as Vector2) - c).angle()
		# A 1e-4 rad resolution so a sub-pixel wobble cannot reorder two members; `id` decides a tie.
		if absf(aa - bb) > 1e-4:
			return aa < bb
		return int(items[a]["id"]) < int(items[b]["id"]))
	var remaining := order.size()
	var ring := 1
	var radius := 0.0
	while remaining > 0:
		var cap := remaining
		var rad := spacing / (2.0 * sin(PI / float(maxi(cap, 2)))) if cap > 1 else 0.0
		# A ring holds at most 7; the next one sits one spacing further out.
		if cap > 7:
			cap = 7
			rad = spacing / (2.0 * sin(PI / 7.0))
		radius = maxf(rad, radius + spacing) if ring > 1 else rad
		for k in cap:
			var ang := -PI * 0.5 + TAU * float(k) / float(cap) + (PI / float(cap) if ring > 1 else 0.0)
			slots.append(c + Vector2.from_angle(ang) * radius)
		remaining -= cap
		ring += 1
	var out := {}
	for k in order.size():
		out[int(items[order[k]]["id"])] = (slots[k] as Vector2) - (items[order[k]]["pos"] as Vector2)
	return out


## Draws the thin leader line and the true-point dot for a slid CIRCLE marker (pins no longer slide, Ruling FA): from
## `true_pos` to the drawn centre. Drawn BEFORE the markers so the head covers the line's far end. `w` is the stroke scale as in `draw()`.
static func draw_leader(ci: CanvasItem, true_pos: Vector2, drawn: Vector2, r: float, w: float = 1.0) -> void:
	if true_pos.distance_to(drawn) < 0.5:
		return
	var lw := maxf(1.0, 1.1 * w * r / 9.0)
	ci.draw_line(true_pos, drawn, Color(1, 1, 1, 0.85), lw + 2.0 * maxf(1.0, w * r / 18.0), true)
	ci.draw_line(true_pos, drawn, INK, lw, true)
	ci.draw_circle(true_pos, maxf(1.5, 0.28 * r), INK, true, -1.0, true)
	ci.draw_circle(true_pos, maxf(0.8, 0.14 * r), COL_NONE, true, -1.0, true)
