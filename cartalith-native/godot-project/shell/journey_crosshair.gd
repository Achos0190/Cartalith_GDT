extends Control
## The touch planner's crosshair (owner, 2026-11-16: *"maybe it's easier to use a crosshair system for the journey planner on
## android to drop waypoints and then commit the journey"*; `OUTSTANDING_WORK.md`, `JOURNEY_PLANNER_SPEC.md` build note).
##
## **What it is.** On the touch forms (phone, tablet) the planner covers the map, and the only other ways to place stops were
## two taps through the context card (*Plan journey from here* / *Journey to here*) or the Route tool. This is a mode of the
## real map instead: the planner hides, the map comes back with a FIXED crosshair at the centre of the map view, and a
## thumb-reach bar of four buttons. The user pans and zooms the map under the crosshair exactly as always (this control
## swallows no touch outside its own buttons) and presses **Add waypoint** (the first is the start, each later one a stop),
## **Undo last**, **Commit journey**, or the cancel cross. Desktop never reaches this file: it keeps clicking
## (`JourneyPlannerView._touch_crosshair_wanted()`).
##
## **One route model, not a parallel planner.** The waypoints are a plain list of grid points kept here; **Commit** hands them
## to `JourneyPlannerView.commit_waypoints()`, which is `_commit_stops()` -- the three engine calls (`route_begin("mixed")`,
## `route_append_stop`, `route_commit`) the desktop's End here, the Route tool and the context card all make -- and the planner
## then opens on the committed route (`DccApp.open_journey_planner_with_route`). Nothing here computes a route of its own.
##
## **No live route, by measurement.** The owner asked for the route live as waypoints are added "if cheap". The only engine call
## that solves a route is `route_commit` (the draft keeps points; the solve runs at commit), so a live preview means committing a
## throw-away route and deleting it again, and that costs the engine's whole route-input build on every press: measured 2026-11-17
## on the phone-form harness at 512x384, `_pf_probe.gd --leg cross` prints it (about 110 ms per solve on this PC, before any phone
## slowdown, and the cost is the input build, not the number of legs). That drops several frames per press and grows with the grid,
## so the preview is straight legs between the numbered waypoints and the route is computed once, on **Commit journey**.
##
## **Must never** compute a route that is not the engine's, leave a preview route in the engine's list (the delete runs on every
## path, and a probe counts `route_count()` before and after), touch the vault or the save, swallow a map gesture, or sit on
## the crosshair (the bar is anchored to the bottom edge, the crosshair to the map centre).

const MIN_BTN := 44            ## Touch floor in UI units (`DccTheme.role_px("btn_min_h")` is 44 on touch; asserted by the probe).
const BAR_MAX_W := 640.0       ## Labelled judgement: about a hand-span of buttons on a 10 inch tablet held in two hands.
const SAME_POINT_CELLS := 0.75 ## A second waypoint closer than this to the last one is a double press, not a stop.

var app: DccApp
var bridge: EngineBridge
var planner: Node                ## the `JourneyPlannerView` that opened this; its `commit_waypoints()` is the one commit path.

## Waypoints, oldest first, grid points (continuous, `ViewportHost.grid_at_view_centre()`'s coordinates).
var _pts: PackedVector2Array = PackedVector2Array()

var _bar: PanelContainer
var _readout: Label
var _btn_add: Button
var _btn_undo: Button
var _btn_commit: Button
var _btn_cancel: Button
var _active := false


func is_active() -> bool:
	return _active


## Waypoints so far (a copy), for the probe and the readout.
func waypoints() -> PackedVector2Array:
	return _pts.duplicate()


func bar() -> Control:
	return _bar


func setup(a: DccApp, b: EngineBridge, p: Node) -> void:
	app = a
	bridge = b
	planner = p
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	name = "JourneyCrosshair"
	_build_bar()


func _build_bar() -> void:
	_bar = PanelContainer.new()
	_bar.name = "CrosshairBar"
	var sb := StyleBoxFlat.new()
	sb.bg_color = DccTheme.c("bg")
	sb.bg_color.a = 0.94
	sb.set_corner_radius_all(14)
	sb.set_content_margin_all(10)
	sb.border_color = DccTheme.c("accent")
	sb.set_border_width_all(1)
	_bar.add_theme_stylebox_override("panel", sb)
	_bar.mouse_filter = Control.MOUSE_FILTER_STOP
	## Bottom-anchored, full width less a 12 px gutter, lifted clear of the gesture inset: thumb reach, and nowhere near the
	## map centre where the crosshair is drawn.
	_bar.anchor_left = 0.5
	_bar.anchor_right = 0.5
	_bar.anchor_top = 1.0
	_bar.anchor_bottom = 1.0
	_bar.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_bar.offset_bottom = -12.0
	add_child(_bar)
	resized.connect(_fit_bar)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 8)
	_bar.add_child(col)
	_readout = DccTheme.label("", "text", DccTheme.FS_BODY)
	_readout.name = "CrosshairReadout"
	_readout.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(_readout)
	var r1 := HBoxContainer.new()
	r1.add_theme_constant_override("separation", 8)
	col.add_child(r1)
	_btn_undo = _mk(r1, "Undo last", "CrosshairUndo", _undo)
	_btn_add = _mk(r1, "Add waypoint", "CrosshairAdd", _add)
	_btn_add.size_flags_stretch_ratio = 2.0
	var r2 := HBoxContainer.new()
	r2.add_theme_constant_override("separation", 8)
	col.add_child(r2)
	_btn_cancel = _mk(r2, "Cancel", "CrosshairCancel", cancel)
	_btn_commit = _mk(r2, "Commit journey", "CrosshairCommit", _commit)
	_btn_commit.size_flags_stretch_ratio = 2.0


## The bar spans the screen less a 12 px gutter on a phone and is capped at `BAR_MAX_W` and centred on a wide window (a
## tablet's full-width bar would put the buttons beyond a thumb's reach).
func _fit_bar() -> void:
	if _bar == null or size.x <= 0.0:
		return
	var w := minf(size.x - 24.0, BAR_MAX_W)
	_bar.offset_left = -w * 0.5
	_bar.offset_right = w * 0.5


func _mk(row: HBoxContainer, text: String, node_name: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.name = node_name
	b.custom_minimum_size = Vector2(0.0, float(maxi(MIN_BTN, DccTheme.role_px("btn_min_h"))))
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.focus_mode = Control.FOCUS_NONE
	b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	b.pressed.connect(on_press)
	row.add_child(b)
	return b


## Starts the mode: the map back on screen (the planner is hidden by arming Inspect, as every route to a map from the planner
## does), the crosshair and bar shown, the waypoint list empty. `reset` false keeps the list (used by a re-entry).
func begin(reset: bool = true) -> void:
	if reset:
		_pts = PackedVector2Array()
	_active = true
	visible = true
	app.arm_tool("inspect")
	## Armed Inspect hides the planner's panels; on the phone it also closes the dock sheet the planner raised.
	_fit_bar()
	_refresh()
	queue_redraw()


## Leaves the mode and opens the planner back where it was (no route added).
func cancel() -> void:
	if not _active:
		return
	_end()
	app.open_journey_planner()


func _end() -> void:
	_active = false
	visible = false
	_pts = PackedVector2Array()


func _process(_dt: float) -> void:
	if not _active:
		return
	if bridge.generating:
		## A regenerate takes the engine: the stops are void. Same refusal the commit paths make.
		cancel()
		return
	queue_redraw()


func _centre_point() -> Dictionary:
	return app.viewport.grid_at_view_centre()


func _add() -> void:
	var c := _centre_point()
	if not bool(c.get("valid", false)):
		_set_hint("The crosshair is off the map -- pan the map so it sits on land or sea inside the frame.")
		return
	var p := Vector2(float(c["gx"]), float(c["gy"]))
	if _pts.size() > 0 and _pts[_pts.size() - 1].distance_to(p) < SAME_POINT_CELLS:
		_set_hint("That is the last waypoint already -- pan the map to the next place first.")
		return
	_pts.append(p)
	_refresh()


func _undo() -> void:
	if _pts.size() == 0:
		return
	_pts.remove_at(_pts.size() - 1)
	_refresh()


## Commits through the planner's own path; opens the planner on the new route. Refused (with the engine's reason) when the
## engine commits nothing.
func _commit() -> void:
	if _pts.size() < 2:
		return
	var stops: Array = []
	for p in _pts:
		stops.append({"x": p.x, "y": p.y})
	var idx: int = int(planner.call("commit_waypoints", stops))
	if idx < 0:
		_set_hint("The route could not be committed -- it needs two stops the map can route between.")
		return
	_end()
	app.arm_tool("inspect")
	app.open_journey_planner_with_route(idx)


func _set_hint(text: String) -> void:
	if _readout != null:
		_readout.text = text


func _refresh() -> void:
	var n := _pts.size()
	_btn_undo.disabled = n == 0
	_btn_commit.disabled = n < 2
	_btn_add.text = "Add waypoint" if n > 0 else "Add start"
	var t := ""
	if n == 0:
		t = "Pan the map so the crosshair is on the start, then press Add start."
	elif n == 1:
		t = "Start set. Pan to the first stop and press Add waypoint."
	else:
		t = "%d waypoints. Add more, or Commit journey (the route is computed then)." % n
	_readout.text = t


func _draw() -> void:
	if not _active or app == null or app.viewport == null:
		return
	var vp: ViewportHost = app.viewport
	var to_mine := get_global_transform().affine_inverse() * vp.get_global_transform()
	var halo: Color = DccTheme.c("bg")
	var ink: Color = DccTheme.c("accent")
	## Straight legs under the numbered waypoints (the engine's route is drawn after Commit).
	var line := PackedVector2Array()
	for p in _pts:
		var v: Vector2 = vp.grid_to_view(p.x, p.y)
		if v != Vector2.INF:
			line.append(to_mine * v)
	if line.size() > 1:
		draw_polyline(line, halo, 5.0, true)
		draw_polyline(line, ink, 2.5, true)
	var f := ThemeDB.fallback_font
	for i in _pts.size():
		var v: Vector2 = vp.grid_to_view(_pts[i].x, _pts[i].y)
		if v == Vector2.INF:
			continue
		var at: Vector2 = to_mine * v
		draw_circle(at, 13.0, halo, true, -1.0, true)
		draw_circle(at, 11.0, ink, true, -1.0, true)
		draw_string(f, at + Vector2(-4.0 if i < 9 else -8.0, 5.0), str(i + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 14, halo)
	## The crosshair: the centre of the map view, a ring and four ticks with a gap, so the exact point stays visible.
	var c := to_mine * (vp.size * 0.5)
	var r := 22.0
	for pass_i in 2:
		var col := halo if pass_i == 0 else ink
		var w := 5.0 if pass_i == 0 else 2.5
		draw_arc(c, r, 0.0, TAU, 40, col, w, true)
		for d in [Vector2.UP, Vector2.DOWN, Vector2.LEFT, Vector2.RIGHT]:
			draw_line(c + d * (r + 3.0), c + d * (r + 17.0), col, w, true)
			draw_line(c + d * 5.0, c + d * (r - 8.0), col, w, true)
