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

## §10's "brush ring for paint and sculpt" -- a hollow circle at the cursor,
## radius in grid cells so it scales correctly under zoom the same way the
## brush itself does. Any tool that paints/stamps calls `set_brush_cursor()`
## from its own `on_cursor_sampled` (every `Workspace` already gets this
## forwarded, see `app.gd`'s `_wire_selection`); nothing here knows which
## tool is asking.
var brush_visible := false
var brush_center := Vector2.ZERO   ## Grid coords.
var brush_radius_cells := 0.0

## Shared by every domain-specific click-chain tool that builds up a path
## before committing it -- Way/Route (§4.5.4, appended stop by stop via
## `way_append_point`/`route_append_stop`) and Sculpt's Freehand mode
## (§5.2, `sculpt_add_point`). One primitive rather than one per domain:
## none of these three know about each other, and whichever is armed is the
## only one that ever calls `set_path_preview` at a time (`app.gd`'s
## armed-tool exclusivity already guarantees that).
var path_preview: PackedVector2Array = []

## Shared by any tool with on-canvas drag handles beyond Region's own
## (hardcoded corner handles, drawn separately below) -- primarily the
## Label tool's resize/rotate/arc handles (`label_handles`'s three
## `HandleCircle`s, §4.5.5). Each entry is `{x, y, r}` in grid-cell coords,
## matching `handle_circle_dict`'s own shape exactly so a caller can pass
## `bridge.label_handles(...)` values through with no reshaping.
var handles: Array = []

const MEASURE_COLOR := Color(0.878, 0.639, 0.290, 0.95)   ## DccTheme accent.
const MEASURE_POINT_RADIUS := 3.0
const REGION_COLOR := Color(0.878, 0.639, 0.290, 0.85)
const REGION_FILL := Color(0.878, 0.639, 0.290, 0.10)
const REGION_DASH := 6.0
## Distinct from `MEASURE_COLOR` (a query tool's ruler) -- this is a
## construction preview, an in-progress edit, so it reads as "not committed
## yet" rather than "read-only measurement."
const PATH_PREVIEW_COLOR := Color(0.427, 0.788, 0.667, 0.9)   ## teal
const PATH_PREVIEW_POINT_RADIUS := 2.6
const HANDLE_COLOR := Color(0.549, 0.816, 1.0, 0.95)   ## `#8fd0ff`, matches the reference's own rotate-handle blue
const HANDLE_OUTLINE := Color(0.031, 0.024, 0.016, 0.8)

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

func set_brush_cursor(visible_: bool, gx: float = 0.0, gy: float = 0.0, radius_cells: float = 0.0) -> void:
	brush_visible = visible_
	brush_center = Vector2(gx, gy)
	brush_radius_cells = radius_cells
	queue_redraw()

func set_path_preview(points: PackedVector2Array) -> void:
	path_preview = points
	queue_redraw()

## `raw` is an `Array` of `{x, y, r}` dicts (grid-cell coords) -- exactly
## `handle_circle_dict`'s shape, so `[bridge.label_handles(...).resize,
## ...rotate, ...arc]` (each a single dict, not the wrapping dict itself)
## can be passed straight through. An entry that is an empty `Dictionary`
## (a `None` handle, e.g. `label_handles` on a `size_mode == "fixed"` label
## which has no resize handle) is silently skipped, not drawn as a
## zero-radius circle.
func set_handles(raw: Array) -> void:
	handles = raw
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

	if brush_visible and brush_radius_cells > 0.0:
		var center_screen := _grid_to_screen(brush_center, rect)
		## Radius scales by the SAME fit factor the rest of this control
		## already uses (`rect.size.x / _gw`), not a fixed pixel size --
		## a brush is a real distance on the map, so it has to shrink/grow
		## with zoom exactly like the terrain under it does.
		var px_radius: float = brush_radius_cells * (rect.size.x / float(_gw))
		draw_arc(center_screen, px_radius, 0, TAU, 48, MEASURE_COLOR, 1.2, true)

	if measure_points.size() > 0:
		var screen_pts := PackedVector2Array()
		for p in measure_points:
			screen_pts.append(_grid_to_screen(p, rect))
		if screen_pts.size() > 1:
			draw_polyline(screen_pts, MEASURE_COLOR, 1.6, true)
		## `antialiased` (`draw_circle`'s trailing positional arg) defaults to
		## `false` in Godot 4 -- same jagged-edge-at-zoom fidelity issue
		## `map_overlay.gd`'s settlement pins had (see that file's own
		## antialiasing comment), fixed here for these point/handle markers
		## too since they're the same kind of on-canvas circle.
		for sp in screen_pts:
			draw_circle(sp, MEASURE_POINT_RADIUS, MEASURE_COLOR, true, -1.0, true)
			draw_circle(sp, MEASURE_POINT_RADIUS, Color(0, 0, 0, 0.6), false, 1.0, true)

	if path_preview.size() > 0:
		var pp_screen := PackedVector2Array()
		for p in path_preview:
			pp_screen.append(_grid_to_screen(p, rect))
		if pp_screen.size() > 1:
			draw_polyline(pp_screen, PATH_PREVIEW_COLOR, 1.8, true)
		for sp in pp_screen:
			draw_circle(sp, PATH_PREVIEW_POINT_RADIUS, PATH_PREVIEW_COLOR, true, -1.0, true)

	for h in handles:
		var hd: Dictionary = h
		if hd.is_empty():
			continue
		var hp := _grid_to_screen(Vector2(hd["x"], hd["y"]), rect)
		var hr: float = maxf(4.0, float(hd["r"]))
		draw_circle(hp, hr, HANDLE_COLOR, true, -1.0, true)
		draw_arc(hp, hr, 0, TAU, 16, HANDLE_OUTLINE, 1.0, true)

func _draw_dashed_rect(r: Rect2, color: Color, width: float, dash: float) -> void:
	var corners := [r.position, r.position + Vector2(r.size.x, 0), r.position + r.size, r.position + Vector2(0, r.size.y)]
	for i in 4:
		draw_dashed_line(corners[i], corners[(i + 1) % 4], color, width, dash)
