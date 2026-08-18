extends Control
class_name ToolOverlay

## §4.5.1's two global-tool overlays: the Region select marquee and the
## Measure ruler path. `map_overlay.gd` draws world DATA (settlements, roads);
## this draws TOOL feedback -- geometry that exists only while a tool is
## armed and is never part of `get_settlements()`/`get_roads()`. Kept as a
## separate control rather than added to `map_overlay.gd` for exactly that
## reason: the two have nothing in common except sharing the same fit rect,
## which `overlay.displayed_rect()` (a small public wrapper added for this)
## already makes available without duplicating the letterbox math.
##
## `MOUSE_FILTER_IGNORE` always -- this control is drawn on top of `overlay`
## so tool feedback isn't hidden behind the terrain, but it must never steal
## a click from `overlay`'s own hit-testing (`map_clicked`/`map_dragged`,
## which this file's own data ultimately comes from, one layer up in `DccApp`).

var overlay: Control   ## `map_overlay.gd` instance -- for `displayed_rect()` only.
var _gw := 0
var _gh := 0

var measure_points: PackedVector2Array = []
var region_rect := Rect2()   ## Grid-cell coords; zero size means "none".
var region_dragging := false

const MEASURE_COLOR := Color(0.878, 0.639, 0.290, 0.95)   ## DccTheme accent.
const MEASURE_POINT_RADIUS := 3.0
const REGION_COLOR := Color(0.878, 0.639, 0.290, 0.85)
const REGION_FILL := Color(0.878, 0.639, 0.290, 0.10)
const REGION_DASH := 6.0

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	resized.connect(func(): queue_redraw())

func set_grid(gw: int, gh: int) -> void:
	_gw = gw
	_gh = gh
	queue_redraw()

func set_measure_points(points: PackedVector2Array) -> void:
	measure_points = points
	queue_redraw()

func set_region(rect: Rect2, dragging: bool = false) -> void:
	region_rect = rect
	region_dragging = dragging
	queue_redraw()

func _grid_to_screen(p: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2(p.x / _gw, p.y / _gh) * rect.size

func _draw() -> void:
	if _gw <= 0 or _gh <= 0 or overlay == null:
		return
	## `overlay` is deliberately typed `Control` (matching every other file
	## that holds this dynamically-scripted node -- `viewport_host.gd` does
	## the same for the same reason), so GDScript's static analysis can't
	## prove `displayed_rect()` exists on it; plain `var` (no `:=`) sidesteps
	## that the same way `.set_civ_data()`/etc. calls elsewhere already do.
	var rect = overlay.displayed_rect()
	if rect.size.x <= 0.0:
		return

	if region_rect.size.x > 0.0 and region_rect.size.y > 0.0:
		var a := _grid_to_screen(region_rect.position, rect)
		var b := _grid_to_screen(region_rect.position + region_rect.size, rect)
		var screen_rect := Rect2(a, b - a).abs()
		draw_rect(screen_rect, REGION_FILL, true)
		_draw_dashed_rect(screen_rect, REGION_COLOR, 1.4, REGION_DASH)
		## Corner handles, per §4.5.1's own right-dock description ("handles
		## resize it") -- drawn even though resize-by-drag isn't wired yet,
		## so the affordance reads correctly once it is.
		for corner in [screen_rect.position, screen_rect.position + Vector2(screen_rect.size.x, 0),
				screen_rect.position + screen_rect.size, screen_rect.position + Vector2(0, screen_rect.size.y)]:
			draw_rect(Rect2(corner - Vector2(3, 3), Vector2(6, 6)), REGION_COLOR, true)

	if measure_points.size() > 0:
		var screen_pts := PackedVector2Array()
		for p in measure_points:
			screen_pts.append(_grid_to_screen(p, rect))
		if screen_pts.size() > 1:
			draw_polyline(screen_pts, MEASURE_COLOR, 1.6, true)
		for sp in screen_pts:
			draw_circle(sp, MEASURE_POINT_RADIUS, MEASURE_COLOR)
			draw_circle(sp, MEASURE_POINT_RADIUS, Color(0, 0, 0, 0.6), false, 1.0)

func _draw_dashed_rect(r: Rect2, color: Color, width: float, dash: float) -> void:
	var corners := [r.position, r.position + Vector2(r.size.x, 0), r.position + r.size, r.position + Vector2(0, r.size.y)]
	for i in 4:
		draw_dashed_line(corners[i], corners[(i + 1) % 4], color, width, dash)
