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

static func install(app) -> void:
	app.register_tool_click_handler("measure", func(gx, gy): _measure_click(app, gx, gy))
	app.register_tool_escape_handler("measure", func(): _measure_escape(app))
	app.tool_armed.connect(func(id: String): _on_armed(app, id))

	app.register_tool_drag_handler("region", func(gx, gy): _region_drag(app, gx, gy))
	app.register_tool_release_handler("region", func(gx, gy, valid): _region_release(app, gx, gy, valid))
	app.register_tool_escape_handler("region", func(): _region_escape(app))

# -- Measure ----------------------------------------------------------------

## The chain's own points, tracked here in parallel with the engine: `
## measure_result()`'s `segments` are `{cells, km, bearing_deg}` per leg, with
## no positions of their own, so the drawn polyline needs a copy kept where
## the clicks actually land.
static var _measure_points: PackedVector2Array = PackedVector2Array()

static func _on_armed(app, id: String) -> void:
	if id == "measure":
		app.bridge.measure_begin()
		_measure_points = PackedVector2Array()
		app.viewport.tool_overlay.set_measure_points(_measure_points)
		_push_measure_result(app)
	elif id == "region":
		app.bridge.region_clear()
		app.viewport.tool_overlay.set_region(Rect2())
	else:
		## Leaving either tool clears its own on-canvas draft -- neither
		## Measure nor Region select has a "commit": §4.5.1 names Region's
		## own *Send to Data > Export* as what consumes its rect, reading
		## `region_get()` directly, independent of the tool overlay's own
		## drawing surviving.
		_measure_points = PackedVector2Array()
		app.viewport.tool_overlay.set_measure_points(_measure_points)

static func _measure_click(app, gx: float, gy: float) -> void:
	app.bridge.measure_add_point(gx, gy)
	_measure_points.append(Vector2(gx, gy))
	app.viewport.tool_overlay.set_measure_points(_measure_points)
	_push_measure_result(app)

static func _measure_escape(app) -> void:
	app.bridge.measure_clear()
	_measure_points = PackedVector2Array()
	app.viewport.tool_overlay.set_measure_points(_measure_points)
	_push_measure_result(app)
	## Escape clears the chain but leaves Measure armed -- re-measuring is
	## overwhelmingly the likely next action, so this does not fall through
	## to the default tool-manager disarm.

static func _push_measure_result(app) -> void:
	if app.right_dock_ctrl.has_method("show_measure"):
		app.right_dock_ctrl.show_measure(app.bridge.measure_result())

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
