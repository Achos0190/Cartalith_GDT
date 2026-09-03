extends RefCounted
class_name GlobalTools

## §4.5.1's Measure and Region select -- present in every domain, so unlike
## Sculpt/Settlement/Way/Label etc. they have no single domain workspace to
## own their wiring. Built once here and installed by `DccApp`, rather than
## duplicated into all five workspace files or grafted onto whichever domain
## happened to be built first.
##
## Pan/zoom, §4.5.1's third global tool, needs no entry here: it is "always
## available as a modifier" (`viewport_host.gd`'s camera code), never armed,
## so it was never a candidate for this file.
##
## ## 2026-08-24: Measure grew six modes
##
## `design/Cartalith Measurement Toolbar.dc.html` landed and turned the single
## ruler this file used to own into a mode group -- Distance · Bearing · Area ·
## Radius · Cross-section · Δ vertical, with the tool group in the options bar
## (`tool_bar.gd`), the cross-section in a bottom strip (`section_strip.gd`)
## and the readouts in the right dock (`right_dock.gd`). The engine half is
## `measure_bridge.rs`; this file is the interaction model that drives it.
##
## **The interaction decisions the old version recorded all survive**, and are
## restated below where they now apply to six modes rather than one:
##
## - Measure still has **no commit**. A reading persists until it is cleared
##   or the tool is disarmed; nothing writes to the world.
## - **Escape clears the chain but leaves Measure armed** (§4.5.6 names Measure
##   as one of its three exceptions). Re-measuring is overwhelmingly the next
##   action, so it does not fall through to the default disarm.
## - **Region select is not one of those exceptions**, so its own Escape still
##   performs the plain "disarms back to Inspect" rule after cleaning up.
## - Leaving either tool clears its own on-canvas draft; Region's rect
##   survives in the engine because §4.5.1 names *Send to Data ▸ Export* as
##   what consumes it, reading `region_get()` directly.
##
## One canvas affordance is deliberately **not** built: its `⏎ close` for the
## Area ring. The ring here is *implicitly* closed -- three points already
## measure a triangle and the overlay draws the closing edge -- so there is no
## open/closed state for Enter to change. `⌫ drop last` is built, because
## there is no other way to take back a misplaced point without clearing the
## whole reading.

# -- Measure modes -------------------------------------------------------------

## `max_points == 0` means "unlimited". A click at the cap **starts a fresh
## reading from that point** rather than replacing the last one: replacing is
## indistinguishable from a mis-click, whereas starting over is what a person
## who has finished one two-point reading and clicked somewhere else meant.
const MEASURE_MODES: Array = [
	{"id": "distance", "label": "Distance", "max_points": 0, "closed": false, "needs_world": false,
		"hint": "Multi-segment ruler: click to add a point, ⌫ drops the last, Esc clears the chain."},
	{"id": "bearing", "label": "Bearing", "max_points": 2, "closed": false, "needs_world": false,
		"hint": "Two points: compass bearing and its reciprocal, on the map's own convention (0° = north, clockwise)."},
	{"id": "area", "label": "Area", "max_points": 0, "closed": true, "needs_world": true,
		"hint": "A closed ring: projected and true-surface area, perimeter, centroid, and the water inside it subtracted."},
	{"id": "radius", "label": "Radius", "max_points": 2, "closed": false, "needs_world": true,
		"hint": "Centre then rim: radius, diameter, circumference and enclosed area."},
	{"id": "section", "label": "Cross-section", "max_points": 2, "closed": false, "needs_world": true,
		"hint": "A→B: the elevation profile and its sampled fields, in the strip under the map."},
	{"id": "vertical", "label": "Δ vertical", "max_points": 2, "closed": false, "needs_world": true,
		"hint": "Two points: vertical difference, 3D distance, grade and angle."},
]

## The canvas's CROSS-SECTION row. Every one of these reads a field the
## engine already returns in **one** `measure_section()` call -- the choice
## only selects what the strip draws under the elevation line, so switching
## channels never re-crosses the boundary.
##
## The canvas's own sixth entry, `Custom ▾`, is not built: there is no
## user-defined field to bind it to.
const SECTION_CHANNELS: Array = [
	{"id": "elevation", "label": "Elevation", "key": "", "unit": "m"},
	{"id": "terrain", "label": "Terrain", "key": "biome", "unit": ""},
	{"id": "climate", "label": "Climate", "key": "rain", "unit": ""},
	{"id": "hydrology", "label": "Hydrology", "key": "flow", "unit": ""},
	{"id": "geology", "label": "Geology", "key": "lithology", "unit": ""},
]

static var _measure_mode := "distance"
static var _section_channel := "elevation"
static var _section_samples := 512
## The canvas's own figure is ×4. This starts at ×2 because exaggeration here
## grows the strip rather than narrowing its value window (`section_strip.gd`),
## and ×4 on first use takes most of the map before anyone has asked it to.
## ×4 and ×6 are one drag away.
static var _section_exaggeration := 2.0
## The last `measure_section()` result, kept so a channel or exaggeration
## change re-draws the strip without asking the engine again.
static var _section_result: Dictionary = {}

static func measure_mode() -> String:
	return _measure_mode

static func section_channel() -> String:
	return _section_channel

static func section_samples() -> int:
	return _section_samples

static func section_exaggeration() -> float:
	return _section_exaggeration

static func install(app) -> void:
	app.register_tool_click_handler("measure", func(gx, gy): _measure_click(app, gx, gy))
	app.register_tool_escape_handler("measure", func(): _measure_escape(app))
	app.register_tool_backspace_handler("measure", func(): _measure_drop_last(app))
	app.tool_armed.connect(func(id: String): _on_armed(app, id))

	app.register_tool_drag_handler("region", func(gx, gy): _region_drag(app, gx, gy))
	app.register_tool_release_handler("region", func(gx, gy, valid): _region_release(app, gx, gy, valid))
	app.register_tool_escape_handler("region", func(): _region_escape(app))

# -- Measure ----------------------------------------------------------------

## The chain's own points, tracked here in parallel with the engine: `
## measure_result()`'s `segments` are `{cells, km, bearing_deg}` per leg, with
## no positions of their own, so the drawn polyline needs a copy kept where
## the clicks actually land. The five non-Distance modes never touch the
## engine's chain at all -- they are stateless queries over these same points
## (`measure_bridge.rs`'s own "the caller owns the points") -- so this array
## is the single source of truth for every mode.
static var _measure_points: PackedVector2Array = PackedVector2Array()

## The chain as it stands. For `right_dock.gd`'s saved-measurements store,
## which is the one caller that has to keep a reading past the next click:
## `measure_result()`'s segments carry no positions, so the points a
## measurement was taken from exist only here.
##
## Safe to hand out without copying — `PackedVector2Array` is a value type in
## GDScript, so the caller's is already its own and a later click cannot reach
## back into a measurement someone has saved.
static func measure_points() -> PackedVector2Array:
	return _measure_points

## Puts a saved measurement back on the map: its mode, and the points it was
## taken from, through the same two entry points a live click uses
## (`measure_begin` / `measure_add_point`) rather than assigning the array and
## leaving the engine's own chain describing the previous reading.
##
## `arm_tool` first, because `_on_armed` resets `_measure_points` and calls
## `measure_begin()` itself — doing it in the other order would clear the
## points this just placed.
static func recall_measurement(app, mode: String, points: PackedVector2Array) -> void:
	_measure_mode = mode
	_section_result = {}
	app.arm_tool("measure")
	app.bridge.measure_begin()
	_measure_points = PackedVector2Array()
	for p in points:
		app.bridge.measure_add_point(p.x, p.y)
		_measure_points.append(p)
	if app.section_strip != null and mode != "section":
		app.section_strip.clear()
	_apply_style(app)
	_push(app)
	_refresh_bar()

static func _mode_meta() -> Dictionary:
	for m in MEASURE_MODES:
		if String((m as Dictionary)["id"]) == _measure_mode:
			return m
	return MEASURE_MODES[0]

static func _on_armed(app, id: String) -> void:
	if id == "measure":
		app.bridge.measure_begin()
		_measure_points = PackedVector2Array()
		_section_result = {}
		_apply_style(app)
		_push(app)
	elif id == "region":
		app.bridge.region_clear()
		app.viewport.tool_overlay.set_region(Rect2())
	else:
		## Leaving Measure clears its own on-canvas draft and takes the
		## cross-section strip down with it -- the strip is Measure's, not the
		## viewport's, and leaving it up under an unrelated tool would be a
		## reading with nothing driving it.
		_measure_points = PackedVector2Array()
		_section_result = {}
		app.viewport.tool_overlay.set_measure_points(_measure_points)
		app.viewport.tool_overlay.set_handles([])
		if app.section_strip != null:
			app.section_strip.clear()

## Called by `tool_bar.gd`'s Measure tool row.
static func set_measure_mode(app, id: String) -> void:
	if _measure_mode == id:
		return
	_measure_mode = id
	_measure_points = PackedVector2Array()
	_section_result = {}
	app.bridge.measure_begin()
	app.arm_tool("measure")
	_apply_style(app)
	if app.section_strip != null:
		app.section_strip.clear()
	_push(app)
	_refresh_bar()

static func set_section_channel(app, id: String) -> void:
	_section_channel = id
	if app.section_strip != null and not _section_result.is_empty():
		app.section_strip.show_profile(_section_result, _section_channel, _section_exaggeration)
	_refresh_bar()

static func set_section_exaggeration(app, v: float) -> void:
	_section_exaggeration = v
	if app.section_strip != null and not _section_result.is_empty():
		app.section_strip.show_profile(_section_result, _section_channel, _section_exaggeration)

## Stores the requested sample count. Re-sampling is deferred to
## `recompute_section` on the slider's release, matching the `input` vs
## `change` split every generation control in this shell already uses
## (`world_workspace.gd`'s own `tparam()` note): a 1 024-sample re-read per
## drag tick would be a boundary crossing per pixel of slider travel.
static func set_section_samples(_app, n: int) -> void:
	_section_samples = clampi(n, 32, 1024)

static func recompute_section(app) -> void:
	_push(app)

## The Clear chip, and `Esc` -- one path so the two cannot diverge.
static func measure_reset(app) -> void:
	_measure_escape(app)

## The bar's own trailing readout: what this mode is waiting for, or what it
## has. Deliberately a sentence about *state*, not a repeat of the numbers --
## those are the right dock's job.
static func measure_status_text() -> String:
	var meta := _mode_meta()
	var need := int(meta.get("max_points", 0))
	var n := _measure_points.size()
	if n == 0:
		return "click the map to start"
	if need > 0 and n < need:
		return "%d of %d points" % [n, need]
	if need == 0 and _measure_mode == "area" and n < 3:
		return "%d of 3 points minimum" % n
	return "%d point%s" % [n, "" if n == 1 else "s"]

static func _apply_style(app) -> void:
	var meta := _mode_meta()
	var radius_cells := 0.0
	if _measure_mode == "radius" and _measure_points.size() >= 2:
		radius_cells = _measure_points[0].distance_to(_measure_points[1])
	app.viewport.tool_overlay.set_measure_style(
		bool(meta.get("closed", false)), radius_cells, _measure_mode == "section")
	app.viewport.tool_overlay.set_measure_points(_measure_points)

static func _measure_click(app, gx: float, gy: float) -> void:
	var cap := int(_mode_meta().get("max_points", 0))
	if cap > 0 and _measure_points.size() >= cap:
		_measure_points = PackedVector2Array()
		app.bridge.measure_begin()
	app.bridge.measure_add_point(gx, gy)
	_measure_points.append(Vector2(gx, gy))
	_apply_style(app)
	_push(app)
	## The bar carries this mode's own "n of m points" state, so it is rebuilt
	## with the reading rather than only on a mode change -- a stale "click the
	## map to start" under five placed points was the first live pass's other
	## finding. Wholesale rebuild, the same discipline the docks use: the bar
	## is a dozen nodes and a map click is never mid-drag on one of them.
	_refresh_bar()

static func _measure_drop_last(app) -> void:
	if _measure_points.is_empty():
		return
	_measure_points.remove_at(_measure_points.size() - 1)
	## The engine's own chain has no "pop", so it is replayed. Cheap (a chain
	## is a handful of points) and keeps `measure_result()`'s segments in step
	## with the drawn polyline, which is the only thing that matters here.
	app.bridge.measure_begin()
	for p in _measure_points:
		app.bridge.measure_add_point(p.x, p.y)
	_apply_style(app)
	_push(app)
	_refresh_bar()

static func _measure_escape(app) -> void:
	app.bridge.measure_clear()
	_measure_points = PackedVector2Array()
	_section_result = {}
	_apply_style(app)
	app.viewport.tool_overlay.set_handles([])
	if app.section_strip != null:
		app.section_strip.clear()
	_push(app)
	_refresh_bar()
	## Escape clears the chain but leaves Measure armed -- re-measuring is
	## overwhelmingly the likely next action, so this does not fall through
	## to the default tool-manager disarm.

static func _refresh_bar() -> void:
	var bar := DccToolBar.instance()
	if bar != null:
		bar.refresh()

## One dispatch for six modes. Each one hands the right dock the same
## `(mode, data)` pair, so the dock has one Measure context rather than six --
## see `right_dock.gd`'s own `_build_measure`.
static func _push(app) -> void:
	if not app.right_dock_ctrl.has_method("show_measure"):
		return
	var pts := _measure_points
	var data: Dictionary = {}
	match _measure_mode:
		"area":
			if pts.size() >= 3:
				data = app.bridge.measure_area(pts)
		"radius":
			if pts.size() >= 2:
				data = app.bridge.measure_radius(pts[0].x, pts[0].y, pts[1].x, pts[1].y)
		"vertical":
			if pts.size() >= 2:
				data = app.bridge.measure_vertical(pts[0].x, pts[0].y, pts[1].x, pts[1].y)
		"section":
			if pts.size() >= 2:
				data = app.bridge.measure_section(pts[0].x, pts[0].y, pts[1].x, pts[1].y, _section_samples)
				_section_result = data
				if app.section_strip != null:
					app.section_strip.show_profile(data, _section_channel, _section_exaggeration)
		_:
			## Distance and Bearing read the same chain result; the dock
			## presents it differently (Bearing leads with the heading pair,
			## Distance with the total).
			data = app.bridge.measure_result()
	app.right_dock_ctrl.show_measure(data, _measure_mode)

# -- Region select ------------------------------------------------------------

## Latched on the gesture's first drag sample, cleared on release -- the
## press-drag-release lifecycle `map_dragged`/`map_released` give this file,
## since `map_overlay.gd` deliberately exposes no separate "press" signal of
## its own (`map_clicked` already means something else: a placement click).
static var _region_origin: Vector2 = Vector2(-1, -1)

static func _region_drag(app, gx: float, gy: float) -> void:
	if _region_origin.x < 0:
		_region_origin = Vector2(gx, gy)
	var rect := Rect2(_region_origin, Vector2(gx, gy) - _region_origin)
	app.viewport.tool_overlay.set_region(rect.abs(), true)

static func _region_release(app, gx: float, gy: float, valid: bool) -> void:
	if _region_origin.x < 0:
		return   ## A release with no matching drag (e.g. a plain click) -- nothing to commit.
	var end := Vector2(gx, gy) if valid else _region_origin
	var rect := Rect2(_region_origin, end - _region_origin).abs()
	_region_origin = Vector2(-1, -1)
	if rect.size.x < 1.0 or rect.size.y < 1.0:
		app.viewport.tool_overlay.set_region(Rect2())
		return
	app.bridge.region_set(rect.position.x, rect.position.y, rect.size.x, rect.size.y)
	app.viewport.tool_overlay.set_region(rect, false)
	if app.right_dock_ctrl.has_method("show_region"):
		app.right_dock_ctrl.show_region(app.bridge.region_get())

## §4.5.6 names only "way, route, measure" as Escape's special-cased tools --
## Region select is not one of them, so Escape here means the plain
## "otherwise disarms back to Inspect" rule, same as any tool with no
## registered handler at all. This handler exists only to clean up Region's
## own state (the marquee, the latched drag origin) *before* performing that
## same default disarm itself -- registering it at all, rather than leaving
## Region with no escape handler, would otherwise skip that cleanup.
static func _region_escape(app) -> void:
	_region_origin = Vector2(-1, -1)
	app.bridge.region_clear()
	app.viewport.tool_overlay.set_region(Rect2())
	var btn: BaseButton = app.tool_group.get_pressed_button()
	if btn != null:
		btn.button_pressed = false
	app.arm_tool("inspect")
