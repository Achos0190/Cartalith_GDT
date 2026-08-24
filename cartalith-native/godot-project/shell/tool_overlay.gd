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
## `design/Cartalith Measurement Toolbar.dc.html` state 3's Area tool draws
## the same point chain as a **closed ring** (`⏎ close · ⌥ subtract hole`),
## and its Radius tool draws a circle around the first point instead of a
## polyline. Both are a property of the armed measure MODE, not of a second
## set of geometry, so they are two flags on the one chain rather than two
## more `PackedVector2Array`s -- exactly the reasoning `path_preview`'s own
## "one primitive rather than one per domain" comment already applies.
var measure_closed := false
var measure_radius_cells := 0.0   ## `> 0` draws a circle at `measure_points[0]`.
## A/B end labels, drawn only when the mode wants them (Cross-section, which
## is the one measure mode whose two ends are referred to by name everywhere
## else on screen -- the strip's "SECTION A → B", the dock's SECTION LINE).
var measure_end_labels := false
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

## ── Rasterising at screen resolution, not control resolution ────────────────
##
## `ViewportHost` parents this control under `_camera` and scales that camera,
## exactly as it does `map_overlay.gd` -- and Godot does not re-run a
## `CanvasItem`'s draw commands when an ancestor's scale changes, it rescales
## the geometry those commands already produced. So a `draw_polyline` width, a
## `draw_circle` radius, a `draw_dashed_line` dash and a `draw_string`
## `font_size`, all written here in THIS control's local pixels, are magnified
## along with their rasterisation: measured live at 1600x1000, the 1.6 px
## measure ruler rendered **2 px at zoom 1 and 16 px at zoom 6**, and the
## 11 px `A` end-label's bounding box went from 17x18 px to 69x74 px at zoom 4
## -- an 11 px glyph bitmap stretched over four times its size, which is
## precisely the blur the owner reported against `map_overlay.gd` on
## 2026-08-24 (`GUI_GAP_REGISTER.md` §30 / MR-01).
##
## The fix is that file's, verbatim, because the defect is that file's: inside
## `_crisp_begin()`/`_crisp_end()` a `1/zoom` `draw_set_transform` is in force,
## so every coordinate passed to a draw call is a **screen** pixel and every
## size is left alone. `_crisp_begin()` returns the `k` that converts this
## control's local pixels into screen pixels; every position multiplies by it,
## and so does every radius that is a real distance on the map (the brush ring,
## the Radius reading's circle) -- those must keep scaling with zoom, only
## their *stroke* must not.
##
## Unlike `map_overlay.gd`, this is applied to the whole `_draw()` rather than
## to the text and linear layers alone: this control has one transform to enter
## and every primitive it emits is tool chrome, which is screen furniture by
## definition -- a 3 px ruler dot has no business being 24 px across because
## the map underneath it was magnified.
var _camera_zoom := 1.0

## **Observed, not pushed.** `map_overlay.gd` learns the zoom from two
## `set_camera_zoom()` calls `ViewportHost` makes in `_zoom_at()` and
## `reset_view()`; this control reads it off the camera itself. The parent
## *is* `ViewportHost._camera`, and its `scale.x` *is* `_zoom` -- one source,
## and no third call site for a future fourth zoom path (a pinch gesture, a
## "zoom to selection") to forget.
##
## The compare has to happen every frame rather than on a notification.
## `set_notify_transform(true)` was the first attempt and does not work here:
## Godot sends `NOTIFICATION_TRANSFORM_CHANGED` for a `Control`'s own
## transform, and an ancestor's `scale` change does not propagate it to
## children -- measured, the ruler went straight back to 16 px at zoom 6. What
## remains is one float compare per frame against a value that changes a few
## times a second at most, which is cheaper than the redraw it guards.
func _process(_delta: float) -> void:
	var p := get_parent() as Control
	if p == null or is_equal_approx(p.scale.x, _camera_zoom):
		return
	_camera_zoom = p.scale.x
	queue_redraw()

func _crisp_begin() -> float:
	var k := maxf(_camera_zoom, 0.001)
	draw_set_transform(Vector2.ZERO, 0.0, Vector2(1.0 / k, 1.0 / k))
	return k

func _crisp_end() -> void:
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	resized.connect(func(): queue_redraw())
	var p := get_parent() as Control
	if p != null:
		_camera_zoom = p.scale.x

func set_grid(gw: int, gh: int) -> void:
	_gw = gw
	_gh = gh
	queue_redraw()

func set_measure_points(points: PackedVector2Array) -> void:
	measure_points = points
	queue_redraw()

## The armed measure mode's own presentation, set once when the mode changes
## (`global_tools.gd`'s `_apply_measure_mode`) rather than on every click.
func set_measure_style(closed: bool, radius_cells: float = 0.0, end_labels: bool = false) -> void:
	measure_closed = closed
	measure_radius_cells = radius_cells
	measure_end_labels = end_labels
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

## `k` is `_crisp_begin()`'s return: local pixels -> screen pixels. Folded in
## here rather than at every call site so no caller can forget it.
func _grid_to_screen(p: Vector2, rect: Rect2, k: float = 1.0) -> Vector2:
	return (rect.position + Vector2(p.x / _gw, p.y / _gh) * rect.size) * k

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

	## Every coordinate below this line is a SCREEN pixel; every width, dash,
	## font size and marker radius is left in screen pixels by not being
	## multiplied at all. See `_crisp_begin()`'s own doc comment above.
	var k := _crisp_begin()
	## The one factor that turns a distance in grid cells into screen pixels --
	## the fit scale and the camera zoom together. Used by the two radii that
	## are real distances on the map (the brush ring, the Radius reading).
	var cell_px: float = (rect.size.x / float(_gw)) * k

	if region_rect.size.x > 0.0 and region_rect.size.y > 0.0:
		var a := _grid_to_screen(region_rect.position, rect, k)
		var b := _grid_to_screen(region_rect.position + region_rect.size, rect, k)
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
		var center_screen := _grid_to_screen(brush_center, rect, k)
		## Radius scales by the SAME fit factor the rest of this control
		## already uses (`rect.size.x / _gw`), not a fixed pixel size --
		## a brush is a real distance on the map, so it has to shrink/grow
		## with zoom exactly like the terrain under it does. `cell_px` carries
		## the camera zoom as well, since inside `_crisp_begin()` the transform
		## no longer supplies it.
		var px_radius: float = brush_radius_cells * cell_px
		draw_arc(center_screen, px_radius, 0, TAU, 48, MEASURE_COLOR, 1.2, true)

	if measure_points.size() > 0:
		var screen_pts := PackedVector2Array()
		for p in measure_points:
			screen_pts.append(_grid_to_screen(p, rect, k))
		if measure_radius_cells > 0.0:
			## Radius mode: the ring the reading describes, plus the spoke
			## that was actually dragged, so the number in the dock has a
			## visible cause.
			var r_px: float = measure_radius_cells * cell_px
			draw_arc(screen_pts[0], r_px, 0, TAU, 64, MEASURE_COLOR, 1.4, true)
			if screen_pts.size() > 1:
				draw_line(screen_pts[0], screen_pts[1], MEASURE_COLOR, 1.0, true)
		elif measure_closed and screen_pts.size() > 2:
			## `draw_colored_polygon` refuses a self-intersecting ring on some
			## drivers, and a measuring ring is allowed to bow-tie mid-draw --
			## so the fill is a plain closed outline plus nothing, not a
			## tessellated face.
			var ring := screen_pts.duplicate()
			ring.append(screen_pts[0])
			draw_polyline(ring, MEASURE_COLOR, 1.6, true)
		elif screen_pts.size() > 1:
			draw_polyline(screen_pts, MEASURE_COLOR, 1.6, true)
		if measure_end_labels and screen_pts.size() > 0:
			var font := DccTheme.mono(1, true)
			draw_string(font, screen_pts[0] + Vector2(6, -6), "A",
				HORIZONTAL_ALIGNMENT_LEFT, -1, 11, MEASURE_COLOR)
			if screen_pts.size() > 1:
				draw_string(font, screen_pts[screen_pts.size() - 1] + Vector2(6, -6), "B",
					HORIZONTAL_ALIGNMENT_LEFT, -1, 11, MEASURE_COLOR)
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
			pp_screen.append(_grid_to_screen(p, rect, k))
		if pp_screen.size() > 1:
			draw_polyline(pp_screen, PATH_PREVIEW_COLOR, 1.8, true)
		for sp in pp_screen:
			draw_circle(sp, PATH_PREVIEW_POINT_RADIUS, PATH_PREVIEW_COLOR, true, -1.0, true)

	## **`r` is in grid cells, not pixels.** Both producers build it in the same
	## space as `x`/`y`: `label_bridge::handle_circles` offsets it from
	## `LabelBox.px/py` (grid coords), and `icon_bridge::icon_handle` from
	## `IconBox.px/py`. Both floor it at `4.0` — four *cells* — and both are
	## hit-tested at that radius against a grid-space cursor
	## (`labels::label_hit_test`, `IconEditor::hit_test`). This file used to
	## pass it to `draw_circle` untouched, i.e. as four *pixels*, so the drawn
	## handle and the region you could actually grab were different sizes and
	## diverged further the more the map was zoomed. `cell_px` is the same
	## conversion the brush ring above uses, and it makes the circle you see
	## exactly the circle that answers the click.
	for h in handles:
		var hd: Dictionary = h
		if hd.is_empty():
			continue
		var hp := _grid_to_screen(Vector2(hd["x"], hd["y"]), rect, k)
		var hr: float = maxf(4.0, float(hd["r"]) * cell_px)
		draw_circle(hp, hr, HANDLE_COLOR, true, -1.0, true)
		draw_arc(hp, hr, 0, TAU, 16, HANDLE_OUTLINE, 1.0, true)

	_crisp_end()

func _draw_dashed_rect(r: Rect2, color: Color, width: float, dash: float) -> void:
	var corners := [r.position, r.position + Vector2(r.size.x, 0), r.position + r.size, r.position + Vector2(0, r.size.y)]
	for i in 4:
		draw_dashed_line(corners[i], corners[(i + 1) % 4], color, width, dash)
