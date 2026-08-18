extends Control
## Phase 2 civilisation-layer overlay (cartalith-civ): settlements + roads
## drawn on top of `%MapView`'s terrain texture. Godot computes nothing
## here beyond screen-space placement -- every value drawn (position,
## faction, tier, name, population) comes straight from `WorldGen.
## get_settlements()`/`get_roads()` (ARCHITECTURE.md: "Godot computes
## nothing beyond layout").
##
## Sibling of `%MapView` under the same `MarginContainer` (`MapMargin`),
## so both share an identical laid-out rect -- this control's own `size`
## is what `%MapView`'s texture is drawn into. `%MapView` uses
## `stretch_mode = 5` (`STRETCH_KEEP_ASPECT_CENTERED`), so the texture
## itself is letterboxed within that rect; `_displayed_rect()` reproduces
## that same fit/centering math so markers land on the real pixels they
## represent, not on the raw control bounds.

## Okabe-Ito colourblind-safe qualitative palette (Okabe & Ito, 2008) --
## 6 mutually distinct hues, matching `CIV_FACTION_COUNT` in
## `cartalith-godot` exactly (faction ids are 1-based, index with `- 1`).
## Chosen deliberately independent of the UI's own light-parchment theme:
## this is data-driven map content (which faction owns this settlement),
## not UI chrome -- the same reasoning the terrain renderer's own biome
## colours already follow (theme-independent, see CHANGELOG's UI-reskin
## entries).
const FACTION_COLORS: Array[Color] = [
	Color(0.902, 0.624, 0.0),   # orange
	Color(0.337, 0.706, 0.914), # sky blue
	Color(0.0, 0.620, 0.451),   # bluish green
	Color(0.941, 0.894, 0.259), # yellow
	Color(0.0, 0.447, 0.698),   # blue
	Color(0.835, 0.369, 0.0),   # vermillion
]

## Marker radius (px) and stroke width by settlement tier -- capitals
## must read as visually more important than a hamlet at a glance.
const TIER_RADIUS := {"capital": 9.0, "city": 6.5, "town": 5.0, "village": 3.8, "hamlet": 2.8}
const CAPITAL_RING_WIDTH := 2.5
## By `way_type` (`cartalith_civ::WayType`, peak-corridor-usage
## classification, Phase 2 milestone 14) -- a highway should read as more
## prominent than a track, the same "tier implies visual weight" principle
## `TIER_RADIUS` already applies to settlements.
const ROAD_COLOR := Color(0.36, 0.29, 0.16, 0.55)
const ROAD_WIDTH_BY_TYPE := {"highway": 2.6, "regional": 2.0, "road": 1.6, "track": 1.1}
const MARKER_OUTLINE := Color(0.101, 0.070, 0.023, 0.85) ## matches PrimaryButton's ink tone
const HOVER_RADIUS_PAD := 4.0 ## extra hit-test slack (px) beyond the drawn marker radius

## Sea-lane style: reference's own convention (reference HTML line ~15511)
## is a dark navy solid underlayer plus a lighter dashed overlay -- not the
## `ROAD_COLOR`/`ROAD_WIDTH_BY_TYPE` land-road styling, so sea routes read
## as visually distinct (shipping lanes, not roads) at a glance.
const SEA_ROUTE_UNDERLAY := Color(0.039, 0.118, 0.235, 0.4)
const SEA_ROUTE_UNDERLAY_WIDTH := 1.5
const SEA_ROUTE_DASH_COLOR := Color(0.118, 0.510, 0.784, 0.7)
const SEA_ROUTE_DASH_WIDTH := 0.85
const SEA_ROUTE_DASH_LENGTH := 2.6

## Emitted whenever the hovered settlement changes -- `null` on hover-exit.
## Lets `main.gd`'s Sample dock show the same data this overlay's own
## `_draw_hover_card` already draws on-canvas, without duplicating the
## hit-test logic in `_gui_input` below.
## `index` is the position in `get_settlements()`'s own array (`-1` on
## hover-exit) -- the same index `WorldGen.explain_settlement()` keys on, so
## the dock can ask why that settlement is there without re-running any hit
## test.
signal settlement_hovered(data: Variant, index: int)

## DCC shell milestone 1 (DCC_SHELL_SCOPE.md), click-to-pin (GUI_FEATURE_
## PARITY_SCOPE.md Category-1 item #10): emitted on a left click that hits
## a settlement, or on a click that hits nothing (`null`, `-1`) to unpin.
## Independent of `settlement_hovered` above -- the Properties dock holds
## this until the next click, unlike Sample's transient hover state.
signal settlement_selected(data: Variant, index: int)

## DCC shell milestone 1: emitted on every mouse motion with the grid-cell
## position under the cursor (`valid` false when the cursor is off the
## plate interior or nothing has been generated yet). Feeds the viewport's
## corner coordinate readout and the Sample dock -- real grid coordinates
## only, never a fabricated per-cell field (elevation/slope/biome) the
## engine doesn't expose per-cell yet.
signal cursor_sampled(gx: float, gy: float, valid: bool)

## §4.5's tool palette: the two primitives every armed tool needs, and
## nothing more specific than that -- a click-placement tool (Settlement,
## POI, Icon, Label, a Way/Route waypoint) wants `map_clicked`; a
## drag-painting tool (Biome paint, Territory, a Sculpt/Freehand stroke)
## wants `map_dragged`, which fires on every motion sample while the left
## button is held so a caller can accumulate a stroke or dab a brush
## continuously. Both fire in real grid-cell coordinates, the same values
## `cursor_sampled` already computes -- `_grid_point()` below is the one
## place that math lives now, shared by all three signals.
##
## This control stays tool-agnostic on purpose: it always emits both the
## selection signals above AND these two, on every click/drag, regardless of
## which tool is armed. The dispatch decision -- "is Inspect armed, so treat
## this as a selection, or is Settlement armed, so treat it as a placement"
## -- belongs to whoever owns the armed-tool state (`DccApp`), not here. That
## keeps this file ignorant of the tool system entirely, the same boundary
## `cursor_sampled`'s own doc comment already draws for per-cell fields.
signal map_clicked(gx: float, gy: float)
signal map_dragged(gx: float, gy: float)
signal map_released(gx: float, gy: float, valid: bool)   ## LMB release, ends a drag gesture.

var _settlements: Array = []
var _roads: Array = []
var _sea_routes: Array = []
var _gw := 0
var _gh := 0
var _hover_index := -1
## Layer-granularity split (GUI_FEATURE_PARITY_SCOPE.md Category-1 item
## #9): the old shell had one checkbox hiding this whole control (and with
## it, hover input) -- these three flags let Settlements/Roads/Sea routes
## toggle independently while the control itself, and hover/click input on
## settlements, stay live regardless of which are on.
var _show_settlements := true
var _show_roads := true
var _show_sea_routes := true
## Plate-frame width as a fraction of the terrain texture's own width
## (`WorldGen.get_border_inset_frac()`, Phase 3 milestone 4). `0.0` when the
## renderer draws no frame, which makes every use of it below an exact no-op.
var _border_frac := 0.0


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	resized.connect(func(): queue_redraw())


## Called by `main.gd` right after a successful `generate()`/
## `generate_world_structure()`. `settlements` is `WorldGen.get_settlements()`'s
## `Array[Dictionary]`, `roads` is `get_roads()`'s `Array[Dictionary]` --
## each entry `{points: PackedVector2Array, brks: PackedInt32Array,
## way_type: String, name: String}` (Phase 2 milestone 14's own
## consolidated/smoothed/classified output, not raw grid-cell indices --
## `points` are already continuous full-resolution coordinates, drawn via
## `_point_to_screen`, distinct from `_cell_to_screen`'s settlement-marker
## `+0.5` cell-centering, which would be wrong here). `sea_routes` is
## `get_sea_routes()`'s `Array[Dictionary]` -- same `{points, brks, name}`
## shape minus `way_type` (Phase 2 milestone 13, `SeaRoute` has no
## highway/regional/road/track tier). Screen-space conversion happens every
## frame from the current control size, so this stays correct across
## window resizes without needing to be told again.
## `border_frac` is `WorldGen.get_border_inset_frac()` -- the plate frame the
## terrain raster itself now carries, as a fraction of texture width. Passed
## in rather than hardcoded here because `render.rs` owns that geometry (see
## `_interior_rect`); defaults to `0.0` so a caller that doesn't know about
## the frame simply gets the old, uninset behaviour.
func set_civ_data(settlements: Array, roads: Array, sea_routes: Array, gw: int, gh: int, border_frac: float = 0.0) -> void:
	_settlements = settlements
	_roads = roads
	_sea_routes = sea_routes
	_gw = gw
	_gh = gh
	_border_frac = border_frac
	_hover_index = -1
	queue_redraw()


func set_show_settlements(shown: bool) -> void:
	_show_settlements = shown
	queue_redraw()


func set_show_roads(shown: bool) -> void:
	_show_roads = shown
	queue_redraw()


func set_show_sea_routes(shown: bool) -> void:
	_show_sea_routes = shown
	queue_redraw()


## Reproduces `%MapView`'s own `STRETCH_KEEP_ASPECT_CENTERED` fit math so
## grid-cell coordinates map onto the same pixels the terrain texture
## actually occupies (this control's `size` is identical to `%MapView`'s,
## both being plain siblings under one `MarginContainer` -- see this
## script's own top comment). Returns `Rect2(origin, displayed_size)`, or
## a zero rect if there's nothing to fit yet.
func _displayed_rect() -> Rect2:
	if _gw <= 0 or _gh <= 0 or size.x <= 0.0 or size.y <= 0.0:
		return Rect2()
	var scale := minf(size.x / float(_gw), size.y / float(_gh))
	var displayed_size := Vector2(_gw, _gh) * scale
	var origin := (size - displayed_size) * 0.5
	return Rect2(origin, displayed_size)

## Public alias of the above -- `ViewportHost`'s tool overlays (the Measure
## ruler, the Region marquee) need the exact same fit rect this control's own
## drawing already computes, and duplicating the letterbox math a second place
## would be the kind of drift `js_hypot`-style bugs come from. Grid-to-screen,
## not the reverse -- see `_grid_point()` for the inverse.
func displayed_rect() -> Rect2:
	return _displayed_rect()


## The plate *interior*: `_displayed_rect()` minus the frame the terrain
## raster draws over its own outermost cells (paper margin + thick and thin
## neatlines, `render.rs`'s `apply_border`, Phase 3 milestone 4). Everything
## outside this is bare paper with no map under it at all -- the terrain
## there is covered, not shown -- so nothing drawn from world data belongs
## on it.
##
## The frame is inset by the same number of *cells* on all four sides and
## `_displayed_rect()`'s fit scale is uniform, so one pixel inset serves both
## axes. `_border_frac` is a fraction of texture width rather than a cell
## count precisely so this needs no resolution knowledge.
func _interior_rect(rect: Rect2) -> Rect2:
	if _border_frac <= 0.0:
		return rect
	var inset := minf(_border_frac * rect.size.x, minf(rect.size.x, rect.size.y) * 0.45)
	return rect.grow(-inset)


func _cell_to_screen(cell: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2((cell.x + 0.5) / _gw, (cell.y + 0.5) / _gh) * rect.size


## Roads' own `points` are already continuous full-resolution coordinates
## (Catmull-Rom-smoothed, not raw cell indices) -- no `+0.5` centering,
## unlike `_cell_to_screen`'s settlement markers.
func _point_to_screen(p: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2(p.x / _gw, p.y / _gh) * rect.size


func _draw() -> void:
	if _settlements.is_empty() and _roads.is_empty() and _sea_routes.is_empty():
		return
	var rect := _displayed_rect()
	if rect.size.x <= 0.0:
		return
	var interior := _interior_rect(rect)

	# Linear features (roads, sea lanes) are *clipped* at the neatline: a
	# road that runs off the plate genuinely continues past the sheet edge,
	# and cutting it there is what an atlas plate does. Point symbols are
	# handled the opposite way below -- placed or not placed, never sliced.
	#
	# One scissor rect for the whole canvas item rather than hand-clipping
	# four different primitive types. `Control` re-sets both of these from
	# its own rect on every `NOTIFICATION_DRAW`, which fires immediately
	# before `_draw()`, so this override lasts exactly one frame and needs
	# no restore.
	if _border_frac > 0.0:
		var ci := get_canvas_item()
		RenderingServer.canvas_item_set_custom_rect(ci, true, interior)
		RenderingServer.canvas_item_set_clip(ci, true)

	if _show_sea_routes:
		for route: Dictionary in _sea_routes:
			var points: PackedVector2Array = route["points"]
			if points.size() < 2:
				continue
			var brks: PackedInt32Array = route["brks"]
			var start := 0
			for cut in brks:
				_draw_sea_route_segment(points, start, cut, rect)
				start = cut
			_draw_sea_route_segment(points, start, points.size(), rect)

	if _show_roads:
		for way: Dictionary in _roads:
			var points: PackedVector2Array = way["points"]
			if points.size() < 2:
				continue
			var width: float = ROAD_WIDTH_BY_TYPE.get(way["way_type"], 1.6)
			var brks: PackedInt32Array = way["brks"]
			# `brks` marks indices where this way's own path has a real gap
			# (two disjoint consolidated runs sharing one `Way`) -- draw each
			# run between breaks as its own stroke, not one polyline straight
			# through the gap.
			var start2 := 0
			for cut in brks:
				_draw_way_segment(points, start2, cut, rect, width)
				start2 = cut
			_draw_way_segment(points, start2, points.size(), rect, width)

	if _show_settlements:
		for i in _settlements.size():
			var s: Dictionary = _settlements[i]
			var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
			# A settlement whose cell is under the frame has no visible terrain
			# beneath it at all, so a marker there points at nothing -- it is off
			# the plate, and off-plate detail is omitted rather than trimmed to a
			# half-disc against the neatline. The clip above then trims the one
			# remaining case: a settlement just *inside* the interior whose
			# radius overhangs it (the actual defect this fixes -- markers
			# landing partly on the margin, seen in both test worlds).
			if not interior.has_point(pos):
				continue
			var faction: int = s["faction"]
			var color: Color = FACTION_COLORS[(faction - 1) % FACTION_COLORS.size()] if faction > 0 else Color(0.5, 0.5, 0.5)
			var radius: float = TIER_RADIUS.get(s["kind"], 3.0)
			if i == _hover_index:
				radius += 1.5

			draw_circle(pos, radius, color)
			draw_arc(pos, radius, 0, TAU, 24, MARKER_OUTLINE, 1.2, true)
			if s["capital"]:
				draw_arc(pos, radius + CAPITAL_RING_WIDTH, 0, TAU, 28, color, CAPITAL_RING_WIDTH, true)

		if _hover_index >= 0 and _hover_index < _settlements.size():
			_draw_hover_card(_settlements[_hover_index], rect, interior)


## Draws `points[start:end]` (exclusive) as one stroke, converted to
## screen space. `end - start < 2` is a real, legitimate no-op (a run with
## a single point either side of a break contributes nothing to draw).
func _draw_way_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2, width: float) -> void:
	if end - start < 2:
		return
	var screen_points := PackedVector2Array()
	screen_points.resize(end - start)
	for i in range(start, end):
		screen_points[i - start] = _point_to_screen(points[i], rect)
	draw_polyline(screen_points, ROAD_COLOR, width, true)


## Sea lane, reference's own two-pass style (reference HTML line ~15511):
## a solid dark-navy underlayer stroke first, then a lighter dashed overlay
## on top. The underlayer is one `draw_polyline` (phase doesn't matter, it's
## solid). The dash overlay is walked manually via `_draw_dashed_polyline`
## below, NOT one `draw_dashed_line` call per vertex pair -- a smoothed
## route's points are only a few px apart, shorter than one dash+gap cycle,
## so restarting the dash phase at every vertex left every segment landing
## inside the "on" portion and rendered as a solid line, not dashed (caught
## by the real-app screenshot verification this milestone requires, not
## assumed).
func _draw_sea_route_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2) -> void:
	if end - start < 2:
		return
	var screen_points := PackedVector2Array()
	screen_points.resize(end - start)
	for i in range(start, end):
		screen_points[i - start] = _point_to_screen(points[i], rect)
	draw_polyline(screen_points, SEA_ROUTE_UNDERLAY, SEA_ROUTE_UNDERLAY_WIDTH, true)
	_draw_dashed_polyline(screen_points, SEA_ROUTE_DASH_COLOR, SEA_ROUTE_DASH_WIDTH, SEA_ROUTE_DASH_LENGTH)


## Draws `points` as a dashed line with the dash phase carried continuously
## across every vertex (equal-length dash/gap, `dash_len` each) -- unlike
## `draw_dashed_line` per-segment, a dash or gap can span a vertex instead
## of always restarting "on" there.
func _draw_dashed_polyline(points: PackedVector2Array, color: Color, width: float, dash_len: float) -> void:
	var period := dash_len * 2.0
	var phase := 0.0
	for i in range(points.size() - 1):
		var p0 := points[i]
		var p1 := points[i + 1]
		var seg_vec := p1 - p0
		var seg_len := seg_vec.length()
		if seg_len <= 0.0:
			continue
		var dir := seg_vec / seg_len
		var traveled := 0.0
		while traveled < seg_len:
			var cycle_pos := fmod(phase, period)
			var on := cycle_pos < dash_len
			var remaining_in_state := (dash_len - cycle_pos) if on else (period - cycle_pos)
			# `remaining_in_state` is mathematically > 0 whenever the loop is
			# reached, but `phase` accumulates across every vertex of a long
			# route, and float drift can land `cycle_pos` close enough to a
			# `dash_len`/`period` boundary that subtraction rounds to exactly
			# 0.0 -- `step` then never advances `traveled`, spinning forever
			# and flooding the renderer's draw-command buffer until it
			# overflows (crashed a real run, not a hypothetical). Floor
			# `step` to a sub-pixel epsilon so every iteration makes forward
			# progress; the resulting overshoot is invisible.
			var step := maxf(minf(remaining_in_state, seg_len - traveled), 0.001)
			if on:
				draw_line(p0 + dir * traveled, p0 + dir * (traveled + step), color, width, true)
			traveled += step
			phase += step


func _draw_hover_card(s: Dictionary, rect: Rect2, interior: Rect2) -> void:
	var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := ["%s (%s)" % [s["name"], kind_label], "Population %s" % _format_pop(s["population"])]
	var font := get_theme_default_font()
	var font_size := 13
	var line_h := font.get_height(font_size)
	var w := 0.0
	for line in lines:
		w = maxf(w, font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x)
	var pad := 8.0
	var card_size := Vector2(w + pad * 2, line_h * lines.size() + pad * 2)
	var card_pos := pos + Vector2(10, -card_size.y - 10)
	# Clamped into the plate interior, not into this control's full bounds:
	# `_draw`'s scissor is still in force here, so a card clamped to the
	# control edge would be sliced by the neatline. Keeping it inside the
	# interior keeps it whole *and* keeps it on the map, which is where a
	# tooltip about map content belongs anyway.
	card_pos.x = clampf(card_pos.x, interior.position.x, maxf(interior.position.x, interior.end.x - card_size.x))
	card_pos.y = clampf(card_pos.y, interior.position.y, maxf(interior.position.y, interior.end.y - card_size.y))

	# Dark-viewport-compatible palette (GUI decluttering pass, 2026-08-17) --
	# was a light-parchment cream/brown card, the fourth stray light-styled
	# surface even though this control's independence from the Theme
	# resource is legitimate (map content, not chrome, see this file's own
	# top comment). Matches the shell's own surface/border/emphasis tokens
	# (`theme/dark_theme.tres`): near-black card fill, accent border,
	# emphasis-toned text.
	draw_rect(Rect2(card_pos, card_size), Color(0.051, 0.055, 0.059, 0.96), true)
	draw_rect(Rect2(card_pos, card_size), Color(0.878, 0.639, 0.290, 1.0), false, 1.5)
	for j in lines.size():
		var text_pos := card_pos + Vector2(pad, pad + line_h * (j + 1) - font.get_descent(font_size))
		draw_string(font, text_pos, lines[j], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.910, 0.922, 0.925))


func _format_pop(pop: int) -> String:
	if pop >= 1000:
		return "%.1fk" % (pop / 1000.0)
	return str(pop)


## Nearest settlement whose marker is within its own hit radius of `mouse`,
## or `-1`. Shared by hover (`_gui_input`'s motion branch) and click-to-pin
## (its button branch) so both use exactly one hit-test definition.
func _hit_test_settlement(mouse: Vector2, interior: Rect2, rect: Rect2) -> int:
	var closest := -1
	var closest_dist := INF
	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
		# Same predicate `_draw` uses: an off-plate settlement has no
		# marker, so it must not have a hit target either -- otherwise a
		# hover/click would fill in dock data from what looks like blank
		# paper.
		if not interior.has_point(pos):
			continue
		var radius: float = TIER_RADIUS.get(s["kind"], 3.0) + HOVER_RADIUS_PAD
		var d := mouse.distance_to(pos)
		if d <= radius and d < closest_dist:
			closest = i
			closest_dist = d
	return closest


## Returns `{valid, gx, gy}` -- the inverse of `_cell_to_screen`, shared by
## `cursor_sampled`, `map_clicked` and `map_dragged` so the coordinate math
## exists in exactly one place. `valid` is false outside the plate interior
## (bare paper, no world coordinate under it) or before anything has
## generated.
func _grid_point(mouse: Vector2, rect: Rect2, interior: Rect2) -> Dictionary:
	if _gw <= 0 or _gh <= 0 or not interior.has_point(mouse):
		return {"valid": false, "gx": 0.0, "gy": 0.0}
	return {
		"valid": true,
		"gx": (mouse.x - rect.position.x) / rect.size.x * _gw,
		"gy": (mouse.y - rect.position.y) / rect.size.y * _gh,
	}

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			cursor_sampled.emit(0.0, 0.0, false)
			return
		var interior := _interior_rect(rect)
		var mm := event as InputEventMouseMotion
		var mouse: Vector2 = mm.position

		var closest := _hit_test_settlement(mouse, interior, rect)
		if closest != _hover_index:
			_hover_index = closest
			queue_redraw()
			settlement_hovered.emit(_settlements[closest] if closest != -1 else null, closest)

		var p := _grid_point(mouse, rect, interior)
		cursor_sampled.emit(p["gx"], p["gy"], p["valid"])
		if p["valid"] and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			map_dragged.emit(p["gx"], p["gy"])

	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			return
		var interior := _interior_rect(rect)
		if mb.pressed:
			var hit := _hit_test_settlement(mb.position, interior, rect)
			settlement_selected.emit(_settlements[hit] if hit != -1 else null, hit)
			var p := _grid_point(mb.position, rect, interior)
			if p["valid"]:
				map_clicked.emit(p["gx"], p["gy"])
		else:
			## §4.5.1's Region select needs the release, not the press --
			## `map_dragged` already reported every point along the way, but
			## nothing marked the gesture's *end*, which is where a marquee
			## commits. `p["valid"]` is intentionally not required here: a
			## drag that ends off-plate still has to end the drag, or a
			## caller's own latched origin (`GlobalTools._region_origin`)
			## would never clear.
			var p := _grid_point(mb.position, rect, interior)
			map_released.emit(p["gx"], p["gy"], p["valid"])


func _notification(what: int) -> void:
	if what == NOTIFICATION_MOUSE_EXIT:
		cursor_sampled.emit(0.0, 0.0, false)
		if _hover_index != -1:
			_hover_index = -1
			queue_redraw()
			settlement_hovered.emit(null, -1)
