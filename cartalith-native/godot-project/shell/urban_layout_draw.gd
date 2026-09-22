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
## **The clutter absence is re-verified 2026-09-13, at the symbols, not just
## carried forward.** `urban_bridge.rs::layout_dict` never reads
## `cartalith_civ::urban_adapter::UrbanLayout::farmland`'s siblings — there are
## none: that struct's own `farmland` field comment says outright "The rest of
## `build_details`' output — trees, fences, spoil heaps, drying racks, log
## booms — stays engine-side", and `run_layout` builds it with
## `t.details.into_iter().filter(|d| d.kind == "field" || d.kind == "pasture")`,
## which drops every well/cross/crane/bollard/tree/fence/spoilheap/dryingrack/
## logboom `Detail` `hinterland.rs::build_details` generates. `_urbandraw_keys_probe.gd`
## dumps five live `urban_layouts()` towns (one forced unwalled) and confirms
## by exhaustive key search: 54 keys total, none of them tree/garden/orchard/
## well/tower/clutter/detail/fence/crane/bollard/cross/spoil/rack/boom-shaped.
## Drawing that clutter needs a `cartalith-civ`/`cartalith-godot` change first —
## out of this file's reach alone.
##
## **One feature new 2026-09-13 needs no such change, because its data was
## already crossing the bridge.** Round towers at every vertex of a `curtain`
## wall (`_draw_wall`, below) read nothing but the `wall_ring` already drawn as
## the circuit's own stroke. It is not a reference port — see its own doc
## comment for what it is instead. (An intramural/extramural roof tint built
## the same day was reverted: `building_district` is not a wall-containment
## test — `assign_districts` gives market/craftriver/harbour and the economy
## retags regardless of the wall — so a tint needs a real in-wall test.)
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
##
## `faubourg` is **not the reference's** — the poor quarter built against the
## wall's outer face (`cartalith_urban::wallside`, a Ruling H departure, owner
## 2026-09-22). Derived, not picked: the midpoint of `suburb` and `artisan` (the
## districts on either side of the wall from it) darkened by 12%, at the floor
## of `craftriver`/`harbour`'s 0.26-0.30 alpha band, so the dense row of hovels
## reads as a quarter of its own rather than as more suburb.
const DISTRICT_FILL := {
	"market": Color(0.847, 0.788, 0.588, 0.32),
	"burgher": Color(0.776, 0.706, 0.549, 0.26),
	"artisan": Color(0.690, 0.643, 0.541, 0.22),
	"craftriver": Color(0.588, 0.675, 0.675, 0.30),
	"harbour": Color(0.549, 0.635, 0.745, 0.30),
	"suburb": Color(0.659, 0.690, 0.549, 0.20),
	"agrarian": Color(0.667, 0.737, 0.510, 0.20),
	"faubourg": Color(0.594, 0.587, 0.480, 0.26),
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
## Round towers at a `curtain` wall's vertices. **Not a reference port —**
## grep of the whole reference finds no vertex ornament on an ordinary
## circuit at all: its only "tower" objects are the harbour chain/mole towers
## (`_umBuildDefences`, unrelated) and the minaret/dome/spire glyphs
## `buildFaithSites` puts on a place of worship. `_draw_wall`'s existing
## `palisade` branch already ticks a post at every second vertex (immediately
## below) and a `ditch` has no masonry to mount one on, so this is new ink for
## the one style that has neither: `curtain` only. Sourced from the owner's
## target image instead
## (`design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`,
## Ruling H (b)), which draws a dark roundel at every vertex, clearly larger
## than the wall's own line — a shade darker than `WALL_STONE` and a radius
## comfortably wider than `WALL_W["curtain"]`'s stroke.
const WALL_TOWER := Color(0.212, 0.169, 0.114) # a shade under WALL_STONE
const WALL_TOWER_R_M := 3.6
## Base stroke widths in model metres, from the reference's own three branches:
## `4.5` stone, `2.2` palisade, `1.6` ditch.
const WALL_W := {"curtain": 4.5, "palisade": 2.2, "ditch": 1.6}

## The bastioned trace italienne, ported from `_umDrawLayout` lines 23348-23350:
## `strokePolys([W.fort.trace],true,Math.max(1.6,5.5*mScale),'rgb(60,50,34)')`
## then, if any, `strokePolys(W.fort.ravelins,true,Math.max(1.0,3*mScale),
## 'rgb(74,62,44)')`. A heavier, darker ink than `WALL_STONE`'s ordinary
## curtain -- a c.1500 gunpowder-artillery enceinte reads as more massive than
## a medieval wall, not the same stroke around a different polygon.
const WALL_FORT_TRACE := Color(0.2353, 0.1961, 0.1333) # rgb(60,50,34)
const WALL_FORT_RAVELIN := Color(0.2902, 0.2431, 0.1725) # rgb(74,62,44)
## The reference's `bastioned` branch is not here -- **stale reasoning
## corrected 2026-09-13.** This used to say a star fort was unreachable
## because "this port's settlements carry no traits", written before trait
## toggling shipped. It is wrong now, on both halves.
##
## The trait reaches the engine: the Place Editor's chips call
## `civ_settlement_toggle_trait`, which flips it in
## `civ_roster_bridge::PlaceExtrasTable`; `urban_bridge.rs::urban_layouts`
## reads it back as `fortified_trait` on every call and threads it through
## `PlaceOverrides` into `GenOpts.fortified`; `generate.rs` grants the fort
## once population clears `FORT_MIN` (2500), walls are on, and the culture's
## `wall_gates_scheme` is `"organic"`; and `cartalith_urban::fortify::
## apply_star_fort` builds the whole trace italienne -- bastions, curtains,
## ditch, glacis, ravelins -- exactly as the reference does. Confirmed live,
## not read off the source alone: toggling a large "organic"-scheme town
## Fortified through that exact call and re-reading `urban_layouts()` flips
## `wall_style` from `"curtain"` to `"bastioned"` and `wall_ring` from a
## 289-vertex hull to an 18-vertex gorge polygon (`_bastionwall_probe.gd`).
##
## **What is actually missing is one layer up.** `urban_bridge.rs::
## layout_dict` never reads `l.wall.fort` (the `Fort` struct carrying
## `trace`/`bastions`/`ravelins`/`ditch`/`glacis`) onto the dictionary this
## file receives -- confirmed by the same probe dumping the layout's full key
## list: `wall_style`/`wall_ring`/`wall_gates`/`wall_water_gates`/
## `wall_spurs`/`wall_centroid` are the whole wall vocabulary that crosses,
## and `wall_ring` on a bastioned town is the GORGE (the containment polygon
## through the bastion throats), not the star-shaped trace the reference
## strokes -- drawing it as the wall would render the wrong shape, a
## rounded-off hexagon rather than a star fort. A `bastioned` branch here
## needs `fort.trace` (and, for parity with `_umDrawLayout`, `fort.ravelins`)
## added to that dictionary first: a `cartalith-godot` change, out of this
## lane's reach (no Rust edits, no `cargo build` of `cartalith_godot` while
## other probes hold the loaded `.dll`).

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
		_fill_ground_polygon(ci, project.call(blk),
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
			_fill_ground_polygon(ci, project.call(par), tint.call(DISTRICT_FILL[districts[i]]))

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

	# v2.71's water clip (`RC_ENGINE_CHANGES.md` §8.2) -- the site's real
	# water, drawn LAST so nothing this function laid down over it (measured:
	# only a street's stroked WIDTH does; centrelines, blocks, parcels,
	# buildings and the fringe never sample inside water) stays visible on
	# top of it. See `_draw_water_mask` for why this is `water_mask_runs`,
	# not `water_poly`, and why bridges and fords are released from it.
	_draw_water_mask(ci, layout, project, tint)

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


## A ground fill that does not go silently missing at deep zoom.
##
## **The failure.** `draw_colored_polygon` routes through
## `Geometry2D.triangulate_polygon`; when that returns empty, Godot logs "Invalid
## polygon data, triangulation failed" and draws nothing. Windowed on
## `_settlepix_probe.tscn` (seed 24601, "Sevjuniana") it fired once, at the
## block-ground fill, for block 324: an ordinary ~5 m model-space quad (sides
## 4.96/5.37/1.69/4.40 m, no duplicate vertices, no NaN) projected to four screen
## points only **0.0465 px** across, at an absolute offset near (276, 33).
##
## **The cause is float precision at that offset, not the shape.** The same float32
## points triangulate when translated to the origin, and fail only at the offset:
## the shoelace products there are ~9 250, whose float32 ulp (~0.001) is as large as
## the true winding sum (0.0011 px^2), so the sum rounds to zero. It is the same
## collapsed-below-a-pixel family `_draw_roofs` (below) guards against.
##
## **Why the response differs from `_draw_roofs`.** A roof is one of thousands of
## footprints; skipping one is invisible. A block's ground fill is the only thing
## that colours its lot, so skipping it is the bug. This keeps the roof guard's
## detection (the triangulate check) and, where it fails, fans the polygon from
## vertex 0 into triangles drawn with `draw_primitive`, which never triangulates.
## A fan is exact for a convex polygon. **Blocks are NOT convex by construction**
## -- 127-169 per town are not, up to 107 vertices -- so on a non-convex polygon the
## fan can overdraw its notches; that only happens on this sub-pixel branch.
##
## **The parcel district fills are routed here too, 2026-09-13.** They draw
## only at `detail >= 1` (this file's own gate below), which the block repro's
## zoom never reached -- reproduced separately at z64 (`box_px=652.8` against
## `map_overlay.gd`'s `URBAN_FINE_BOX_PX=620`), same town, same seed: **42
## `Geometry2D.triangulate_polygon` failures over 3 redraws -- 14 of
## Sevjuniana's 4 683 parcels, each failing on every one of the three redraws
## (14 x 3 = 42, exactly)**. B1's own check found zero of those 4 683 parcels
## with a NaN or a near-duplicate vertex in their RAW model-space polygon --
## the same transform-precision cause as block 324 above, not malformed data. Verified
## against a pure HEAD copy (`_settlepix_probe.gd`'s own PART_B block):
## 42 -> 0 after routing through this helper, the full 576x384 capture
## **byte-identical** before and after (every failing parcel's screen
## footprint is sub-pixel, the same reason the block fix moved 0 of 746 496 px
## at its own repro zoom), and the mutation that reverts this one line back to
## a bare `draw_colored_polygon` reproduces exactly 42 again.
##
## **Market, farmland and water fills do not fail in this town at this zoom --
## measured, not assumed.** All three draw unconditionally (no `detail` gate)
## inside the very same z64 redraw the 42/0 count above was taken over, so any
## failure of theirs would already be inside that count. It resolves to
## exactly 14 parcels x 3 draws with nothing left over, both before this fix
## (42) and after (0) -- a residual would have shown up as a number other
## than 0 after routing only the parcels. A different town or a deeper zoom
## could still find one; this is a measurement of Sevjuniana at z64, not a
## proof for every settlement.

static func _fill_ground_polygon(ci: CanvasItem, pts: PackedVector2Array, color: Color) -> void:
	if pts.size() < 3:
		return
	if not Geometry2D.triangulate_polygon(pts).is_empty():
		ci.draw_colored_polygon(pts, color)
		return
	var tri_colors := PackedColorArray([color, color, color])
	var uvs := PackedVector2Array()
	for i in range(1, pts.size() - 1):
		ci.draw_primitive(PackedVector2Array([pts[0], pts[i], pts[i + 1]]), tri_colors, uvs)


## v2.71's water clip (`RC_ENGINE_CHANGES.md` §8.2) — the settlement's real
## water, painted last over anything this file drew on top of it.
##
## **Why this exists at all.** `water_poly` is not the site's real water: on
## the real-map coastal path `buildSite` leaves it empty ON PURPOSE (the
## terrain map underneath already paints the sea, so this file must not paint
## a second one there), and on a real-map river-like site it is only an
## approximate band offset from the centreline, not the actual mask. Neither
## form is what the engine itself checked when it kept blocks, parcels and
## buildings off the water — that was `Site::is_water`, a per-cell lookup
## into the site's real 22 m mask, and `water_mask_runs` is that same mask,
## run-length encoded as axis-aligned rectangles in this layout's own LOCAL
## box frame (`cartalith_urban::site::WaterCtx::water_runs`). It is empty on
## a synthetic site — every headless/preview render is therefore byte-for-byte
## unchanged by this pass — and on a real one it is what actually generation
## queried, so it wins over `water_poly` wherever the two disagree (a town can
## carry both: a river band from the centreline alongside dozens of real mask
## runs).
##
## **Why last, not first.** The measured overdraw is a STREET's stroked
## WIDTH crossing the mask, not its centreline (`RC_ENGINE_CHANGES.md` §8.2:
## zero blocks/parcels/buildings/fringe parcels or street centrelines sample
## inside real water). Painting the mask back over the whole layout at the
## very end covers that overdraw wherever it occurs — including the fringe,
## which must stay drawn BENEATH the water fill (v2.68's own rule) — without
## having to find and clip every individual drawing pass that could someday
## bleed onto it.
##
## **Why corners, not a screen-space rect.** Each run's four LOCAL-frame
## corners go through `project` individually, exactly like a block or a
## parcel polygon — a rectangle in the layout's own rotated frame is a
## quadrilateral on screen, and projecting only two opposite corners (a
## screen-space `Rect2`) would be axis-aligned and wrong the moment the
## layout is rotated (`map_overlay.gd`'s `to_screen` does rotate it).
##
## **Bridges and fords are released.** A road that genuinely crosses the
## river IS the bridge (or, with no crossing road, the flattest ford) — it is
## meant to span the water, so this must not paint back over it. The release
## is a plain distance test in LOCAL metres against each crossing's own
## point, sized off `river_w` the same way `urban_bridge.rs`'s own doc
## comment sizes a drawn span (`rw/2 + 10`ish), widened a little further
## because this is releasing an AREA around the point rather than striking a
## single span from it.
static func _draw_water_mask(ci: CanvasItem, layout: Dictionary, project: Callable,
		tint: Callable) -> void:
	var runs: Array = layout.get("water_mask_runs", [])
	if runs.is_empty():
		return

	var release_pts: PackedVector2Array = PackedVector2Array()
	for p in layout.get("bridges", PackedVector2Array()):
		release_pts.append(p)
	if layout.has("ford"):
		release_pts.append(layout["ford"])

	var river_w: float = float(layout.get("river_w", 20.0))
	var release_r2: float = pow(river_w * 0.5 + 20.0, 2.0)
	var water: Color = tint.call(WATER)

	for run in runs:
		var quad: PackedVector2Array = run
		if quad.size() < 4:
			continue
		var center: Vector2 = (quad[0] + quad[2]) * 0.5
		var released := false
		for p in release_pts:
			if center.distance_squared_to(p) < release_r2:
				released = true
				break
		if released:
			continue
		_fill_ground_polygon(ci, project.call(quad), water)


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
##
## **`bastioned` is a fourth branch, not a stone-family variant.** `wall_ring`
## is `wall.fort`'s GORGE polygon (the containment ring through the bastion
## throats -- reference line 30667's own comment: "the closed bastioned trace
## is the drawn wall (all sides)"), never what a fortified town is drawn as.
## `_draw_bastioned_wall`, below, strokes the real trace and returns before
## any of the ring/gate code in this function runs -- matching the reference,
## which draws no gate markers for a bastioned enceinte at all (line 23348's
## `if` has no `else if` that reaches the plain-gate code lower down).
static func _draw_wall(ci: CanvasItem, layout: Dictionary, project: Callable,
		to_screen: Callable, m_scale: float, px_floor: float, alpha: float) -> void:
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	if ring.size() < 3:
		return
	var style := String(layout.get("wall_style", "curtain"))
	if style == "bastioned":
		_draw_bastioned_wall(ci, layout, project, m_scale, px_floor, alpha)
		return
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

		# Round towers at every vertex -- `curtain` only. `bastioned` no longer
		# reaches this `else` branch at all (`_draw_wall` returns to
		# `_draw_bastioned_wall` before this code runs), so towers on the
		# gorge polygon were never drawn and there is no undocumented
		# departure to guard against here any more. This stays scoped to
		# `curtain` because that is exactly what Ruling H (b) asked for.
		if style == "curtain":
			var tr: float = maxf(px_floor * 1.1, WALL_TOWER_R_M * m_scale)
			var tower_col := Color(WALL_TOWER.r, WALL_TOWER.g, WALL_TOWER.b, alpha)
			for v in ring:
				ci.draw_circle(to_screen.call(v), tr, tower_col)

	var gates: PackedVector2Array = layout.get("wall_gates", PackedVector2Array())
	if gates.is_empty():
		return
	var gc: Color = GATE_PALISADE if style == "palisade" else GATE_STONE
	var gr: float = maxf(px_floor * 0.8, (1.8 if style == "palisade" else 2.2) * m_scale)
	for g in gates:
		ci.draw_circle(to_screen.call(g), gr, Color(gc.r, gc.g, gc.b, alpha))


## The bastioned branch of `_draw_wall`, ported from `_umDrawLayout` lines
## 23348-23350:
## ```
## if(W.style==='bastioned'&&W.fort&&W.fort.trace&&W.fort.trace.length>2){
##   strokePolys([W.fort.trace],true,Math.max(1.6,5.5*mScale),'rgb(60,50,34)');
##   if(W.fort.ravelins&&W.fort.ravelins.length)
##     strokePolys(W.fort.ravelins,true,Math.max(1.0,3*mScale),'rgb(74,62,44)');
## }
## ```
## `"fort_trace"`/`"fort_ravelins"` are `urban_bridge.rs::layout_dict`'s bridge
## keys for `wall.fort.trace`/`wall.fort.ravelins` -- present exactly when the
## reference's own guard above passes, so an absent `"fort_trace"` here (a
## bastioned town whose `fort` never got that far) draws nothing rather than
## falling back to the gorge ring.
##
## No gate markers: the reference draws none for this style (see the header
## comment on `_draw_wall`), so this function does not read `"wall_gates"`.
static func _draw_bastioned_wall(ci: CanvasItem, layout: Dictionary, project: Callable,
		m_scale: float, px_floor: float, alpha: float) -> void:
	var trace: PackedVector2Array = layout.get("fort_trace", PackedVector2Array())
	if trace.size() < 3:
		return
	var trace_lw: float = maxf(px_floor * 1.6, 5.5 * m_scale)
	var pts: PackedVector2Array = project.call(trace)
	pts.append(pts[0])
	ci.draw_polyline(pts, Color(WALL_FORT_TRACE.r, WALL_FORT_TRACE.g,
		WALL_FORT_TRACE.b, alpha), trace_lw, true)

	var ravelins: Array = layout.get("fort_ravelins", [])
	if ravelins.is_empty():
		return
	var rav_lw: float = maxf(px_floor, 3.0 * m_scale)
	var rav_col := Color(WALL_FORT_RAVELIN.r, WALL_FORT_RAVELIN.g, WALL_FORT_RAVELIN.b, alpha)
	for i in ravelins.size():
		var rav: PackedVector2Array = ravelins[i]
		if rav.size() < 3:
			continue
		var rp: PackedVector2Array = project.call(rav)
		rp.append(rp[0])
		ci.draw_polyline(rp, rav_col, rav_lw, true)


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
		# `buildBuildings`' footprints are never degenerate in their own metres
		# -- measured on seed 24601's "Sevjuniana" (5 009 buildings, 0 zero-area
		# or duplicate-vertex) -- but a deep-zoomed map draws a lot at a
		# fraction of a screen pixel while `to_screen` still carries it tens of
		# local units from the origin (rotated about the market, then
		# projected). At that ratio the shoelace terms that recover the
		# footprint's tiny screen-space area subtract two large, nearly-equal
		# products, and Godot's own triangulator loses the polygon: measured
		# 133/5 009 (2.7 %) untriangulable this way, every one with a
		# post-transform bounding-box diagonal under 0.09 px, versus 0/5 009 at
		# either the original metres or a fit-to-box scale with real screen
		# extent. `canvas_item_add_polygon` would log "Invalid polygon data,
		# triangulation failed" and skip the draw anyway for these -- checking
		# here just makes the skip deliberate instead of an engine error, for a
		# lot too small to read as anything once drawn regardless.
		# **Two narrowings from the verifier, kept because both are the kind a
		# later reader would otherwise re-derive at cost.**
		#
		# **The 133 above is the DIAGNOSTIC probe's figure, not this draw's.**
		# The real windowed render throws **151**. `_roofgeom_probe` builds
		# `Rect2(ZERO, size)` while `map_overlay.gd:3686` draws through the
		# INSET content rect (`_border_frac`, `:1660-1663`); `_point_to_screen`
		# itself matches the probe exactly, so only the rect differs and the
		# probe measures a slightly less-squeezed transform. The ratio and the
		# conclusion hold; the count does not transfer.
		#
		# **This guard is NOT structurally unable to lose ink** -- an earlier
		# note here claimed it was. True for the fill and shadow passes, which
		# both go through `draw_colored_polygon` and so both need triangulation
		# anyway. **Not true of the ink pass**, which strokes `q`'s edges via
		# `draw_multiline` -- that never triangulates, so pre-fix these quads
		# DID contribute ink. Emptying the quad also drops it from the `lot_px`
		# median sample, which can shift `want_ink`/`want_detail`.
		#
		# **Unreachable in both measured configurations**, which is why the
		# before/after frames are byte-identical: deep zoom passes `detail=0.0`
		# so the ink pass does not run, and at fit-to-box scale there are zero
		# degenerate quads to skip. **A change that draws ink at deep zoom would
		# make it reachable**, and then the right fix is to keep the quad for
		# the ink pass and skip only the two polygon passes.
		if Geometry2D.triangulate_polygon(q).is_empty():
			quads[i] = PackedVector2Array()
			continue
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
