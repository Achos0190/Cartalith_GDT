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
const ROAD_COLOR := Color(0.36, 0.29, 0.16, 0.55)
const ROAD_WIDTH := 1.6
const MARKER_OUTLINE := Color(0.101, 0.070, 0.023, 0.85) ## matches PrimaryButton's ink tone
const HOVER_RADIUS_PAD := 4.0 ## extra hit-test slack (px) beyond the drawn marker radius

var _settlements: Array = []
var _roads: Array = []
var _gw := 0
var _gh := 0
var _hover_index := -1


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	resized.connect(func(): queue_redraw())


## Called by `main.gd` right after a successful `generate()`/
## `generate_world_structure()`. `settlements` is `WorldGen.get_settlements()`'s
## `Array[Dictionary]`, `roads` is `get_roads()`'s `Array[PackedVector2Array]`
## (already in grid-cell coordinates, not screen space -- this overlay does
## the screen-space conversion itself, every frame it draws, from the
## current control size, so it stays correct across window resizes without
## needing to be told again).
func set_civ_data(settlements: Array, roads: Array, gw: int, gh: int) -> void:
	_settlements = settlements
	_roads = roads
	_gw = gw
	_gh = gh
	_hover_index = -1
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


func _cell_to_screen(cell: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2((cell.x + 0.5) / _gw, (cell.y + 0.5) / _gh) * rect.size


func _draw() -> void:
	if _settlements.is_empty() and _roads.is_empty():
		return
	var rect := _displayed_rect()
	if rect.size.x <= 0.0:
		return

	for road_path: PackedVector2Array in _roads:
		if road_path.size() < 2:
			continue
		var screen_points := PackedVector2Array()
		screen_points.resize(road_path.size())
		for i in road_path.size():
			screen_points[i] = _cell_to_screen(road_path[i], rect)
		draw_polyline(screen_points, ROAD_COLOR, ROAD_WIDTH, true)

	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
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
		_draw_hover_card(_settlements[_hover_index], rect)


func _draw_hover_card(s: Dictionary, rect: Rect2) -> void:
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
	card_pos.x = clampf(card_pos.x, 0.0, size.x - card_size.x)
	card_pos.y = clampf(card_pos.y, 0.0, size.y - card_size.y)

	draw_rect(Rect2(card_pos, card_size), Color(0.984, 0.960, 0.913, 0.96), true)
	draw_rect(Rect2(card_pos, card_size), Color(0.690, 0.498, 0.247, 1.0), false, 1.5)
	for j in lines.size():
		var text_pos := card_pos + Vector2(pad, pad + line_h * (j + 1) - font.get_descent(font_size))
		draw_string(font, text_pos, lines[j], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.164, 0.125, 0.082))


func _format_pop(pop: int) -> String:
	if pop >= 1000:
		return "%.1fk" % (pop / 1000.0)
	return str(pop)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			return
		var mouse: Vector2 = event.position
		var closest := -1
		var closest_dist := INF
		for i in _settlements.size():
			var s: Dictionary = _settlements[i]
			var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
			var radius: float = TIER_RADIUS.get(s["kind"], 3.0) + HOVER_RADIUS_PAD
			var d := mouse.distance_to(pos)
			if d <= radius and d < closest_dist:
				closest = i
				closest_dist = d
		if closest != _hover_index:
			_hover_index = closest
			queue_redraw()


func _notification(what: int) -> void:
	if what == NOTIFICATION_MOUSE_EXIT and _hover_index != -1:
		_hover_index = -1
		queue_redraw()
