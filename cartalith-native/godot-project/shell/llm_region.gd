extends RefCounted
## LLM-7 (`LLM_ASSISTANT_SCOPE.md` s.5c; owner Ruling DD Q10): the REGION INPUT of the Assistant's
## edit actions -- the area of the map the user wants the model to reshape.
##
## ## What it is, and why it is not the Region select tool alone
##
## The existing Region select tool (`global_tools.gd`, "R") draws one RECTANGLE and keeps it in the
## engine for the export (`WorldGen.region_set`); it is an export marquee, and arming any other tool
## leaves it alone but arming Region again clears it. The owner asked for a way to PAINT an area, so
## this adds a small brush beside it instead of changing it:
##
## * **Paint** hides the Assistant, arms the unlisted tool id `assist_region` (an id with no palette
##   cell is legal: `app.gd` documents that clicks on an unregistered id stay inert, and this file
##   registers its own handlers) and shows a bar on the map (brush size, Undo stroke, Clear, Done).
##   Dragging adds round dabs spaced at half the brush radius; a click adds one. Done re-opens the
##   Assistant on the Edit task.
## * **Use the box** copies the export marquee (`bridge.region_get()`) as the region, so a box the user
##   already drew costs one click.
## * The region is the UNION of the box (if any) and the dabs. It is a request input only: it lives in
##   this file's statics for the session, is never written into a world, a save or the settings, and
##   is dropped by `clear()`.
##
## ## What leaves the machine
##
## Nothing from here by itself. `facts()` asks the engine to describe the region
## (`WorldGen.region_describe`: bounding box, cell count, run-length rows, measured heights in metres
## -- numbers only, never a picture, Ruling DI P5) and the Assistant sends that only when the
## "Send: the painted region and its heights" source is ticked and the endpoint is on this computer
## or network (owner question E3's default; both enforced in Rust, `LlmStream.prepare_edit`).
##
## ## What it must never do
##
## Touch the world (every call here is read-only or UI); keep painting after the world changes (a
## generate or open calls `clear()` through `on_world_changed`); hold more dabs than the engine
## accepts (`MAX_DABS`, a labelled twin of `region_edit::MAX_DABS`; the engine re-checks); grow a
## dab radius outside the sculpt brush's own 6..200 cell range.
##
## No `class_name`: preloaded by its two users, so adding it needs no `--import` pass.

## The unlisted tool id armed while painting.
const TOOL_ID := "assist_region"
## Most dabs a region may hold. A twin of `cartalith_terrain::region_edit::MAX_DABS` (512): the shell
## stops adding at the cap and says so; the engine refuses more anyway.
const MAX_DABS := 512
## The brush sizes the bar offers, in grid cells (dab radius). A labelled judgement: all inside the
## engine's accepted 6..200 range (the sculpt brush's own slider range); 16 cells is 0.8 percent of a
## default 2048-cell-wide grid, enough to follow a coast without hundreds of strokes.
const BRUSH_SIZES := [8, 16, 32, 64]
## Dab spacing along a drag, as a share of the radius. Half a radius overlaps neighbours enough to
## read as one continuous stroke without piling up dabs.
const SPACING := 0.5

## The painted dabs, `Vector3(cx, cy, radius)` in grid cells.
static var dabs: Array = []
## Index into `dabs` where each stroke began (for Undo stroke).
static var strokes: Array = []
## The adopted box (grid cells); a zero size means none.
static var rect := Rect2()
## The brush radius in cells.
static var brush := 16.0
## True while the tool is armed and the bar shown.
static var painting := false
## Set when the cap stopped a dab this gesture (so the bar can say so once).
static var capped := false

static var _last := Vector2(-1, -1)
static var _stroke_open := false
static var _installed := false
static var _bar: Control
static var _status: Label
static var _done: Callable


## True when there is no region at all.
static func is_empty() -> bool:
	return dabs.is_empty() and rect.size.x <= 0.0


## The region as the engine reads it (`WorldGen.region_describe`'s `spec`).
static func spec() -> Dictionary:
	var d := {}
	if rect.size.x > 0.0 and rect.size.y > 0.0:
		d["rect"] = [rect.position.x, rect.position.y, rect.size.x, rect.size.y]
	if not dabs.is_empty():
		var a: Array = []
		for v in dabs:
			var dv := v as Vector3
			a.append([dv.x, dv.y, dv.z])
		d["dabs"] = a
	return d


## Drops the whole region (dabs, strokes and the adopted box) and redraws the map overlay.
static func clear(app) -> void:
	dabs.clear()
	strokes.clear()
	rect = Rect2()
	_last = Vector2(-1, -1)
	_stroke_open = false
	capped = false
	_sync(app)
	_refresh_bar()


## A new world replaces the old one: a region painted on it means nothing on the next. Called by the
## Assistant window whenever it notices the world changed (its `open()`), and by `begin_paint`.
static func on_world_changed(app) -> void:
	if not is_empty():
		clear(app)


## Removes the newest stroke (all the dabs it added). No-op when there is none.
static func undo_stroke(app) -> void:
	if strokes.is_empty():
		return
	var from: int = int(strokes.pop_back())
	dabs.resize(mini(from, dabs.size()))
	capped = false
	_sync(app)
	_refresh_bar()


## Adopts the export marquee (`bridge.region_get()`) as the region's box. Returns the reason it
## could not (empty string on success): no box drawn, or a world that is not open.
static func use_box(app, bridge) -> String:
	var g: Dictionary = bridge.region_get()
	if g.is_empty() or int(g.get("w", 0)) <= 0 or int(g.get("h", 0)) <= 0:
		return "Draw a box first: arm Region select (R) in the tool palette and drag on the map."
	rect = Rect2(float(g["x"]), float(g["y"]), float(g["w"]), float(g["h"]))
	_sync(app)
	_refresh_bar()
	return ""


## The brush size as the bar's option index.
static func brush_index() -> int:
	var i := BRUSH_SIZES.find(int(brush))
	return i if i >= 0 else 1


static func set_brush(cells: float) -> void:
	brush = clampf(cells, 6.0, 200.0)


## Registers this tool's handlers once. Safe to call repeatedly.
static func install(app) -> void:
	if _installed:
		return
	_installed = true
	app.register_tool_click_handler(TOOL_ID, func(gx, gy): _click(app, gx, gy))
	app.register_tool_drag_handler(TOOL_ID, func(gx, gy): _drag(app, gx, gy))
	app.register_tool_release_handler(TOOL_ID, func(_gx, _gy, _valid): _release())
	app.register_tool_escape_handler(TOOL_ID, func(): end_paint(app))


## Starts painting: arms the tool, shows the bar, and calls `done` when the user presses Done (or
## Escape). The caller hides its own window first.
static func begin_paint(app, done: Callable) -> void:
	install(app)
	painting = true
	_done = done
	app.arm_tool(TOOL_ID)
	_build_bar(app)
	_sync(app)
	_refresh_bar()


## Stops painting: disarms the tool (back to Inspect), removes the bar and calls the `done` callback
## once.
static func end_paint(app) -> void:
	if not painting:
		return
	painting = false
	_last = Vector2(-1, -1)
	_stroke_open = false
	if _bar != null and is_instance_valid(_bar):
		if _bar.get_parent() != null:
			_bar.get_parent().remove_child(_bar)
		_bar.queue_free()
	_bar = null
	_status = null
	var btn: BaseButton = app.tool_group.get_pressed_button() if app.get("tool_group") != null else null
	if btn != null:
		btn.button_pressed = false
	app.arm_tool("inspect")
	var cb := _done
	_done = Callable()
	if cb.is_valid():
		cb.call()


static func _add_dab(app, gx: float, gy: float) -> void:
	if dabs.size() >= MAX_DABS:
		capped = true
		_refresh_bar()
		return
	if not _stroke_open:
		_stroke_open = true
		strokes.append(dabs.size())
	dabs.append(Vector3(gx, gy, brush))
	_last = Vector2(gx, gy)
	_sync(app)
	_refresh_bar()


## A press (`map_clicked` fires on the PRESS for a mouse, `map_overlay.gd`, and on the release of a tap
## for touch) starts a new stroke and paints its first dab; the drag samples that follow continue it,
## and `_release` closes it. A bare click is therefore a one-dab stroke.
static func _click(app, gx: float, gy: float) -> void:
	if not painting:
		return
	_stroke_open = false
	_add_dab(app, gx, gy)


static func _drag(app, gx: float, gy: float) -> void:
	if not painting:
		return
	var p := Vector2(gx, gy)
	if _stroke_open and _last.x >= 0.0 and p.distance_to(_last) < brush * SPACING:
		return
	_add_dab(app, gx, gy)


static func _release() -> void:
	_stroke_open = false
	_last = Vector2(-1, -1)


## Pushes the region to the map overlay: the dabs, and the adopted box as the overlay's marquee.
static func _sync(app) -> void:
	var vp = app.get("viewport")
	if vp == null or vp.get("tool_overlay") == null:
		return
	vp.tool_overlay.set_assist_region(dabs)
	if rect.size.x > 0.0:
		vp.tool_overlay.set_region(rect, false)


## The bar's text and the buttons' enabled state, from the statics.
static func _refresh_bar() -> void:
	if _bar == null or not is_instance_valid(_bar) or _status == null:
		return
	var t := "%d dab%s" % [dabs.size(), "" if dabs.size() == 1 else "s"]
	if rect.size.x > 0.0:
		t += " + box %d x %d" % [int(rect.size.x), int(rect.size.y)]
	if capped:
		t += "  (the most a region may hold is %d dabs)" % MAX_DABS
	_status.text = t
	var undo_b := _bar.find_child("assist_undo_stroke", true, false) as Button
	if undo_b != null:
		undo_b.disabled = strokes.is_empty()
	var clear_b := _bar.find_child("assist_clear", true, false) as Button
	if clear_b != null:
		clear_b.disabled = is_empty()


## The bar on the map: brush size, Undo stroke, Clear, Done. Top centre of the viewport so it never
## covers the layers button (top left) or the readouts.
static func _build_bar(app) -> void:
	## Removed from the tree NOW, not only queued for freeing: a node still in the tree keeps its name, and
	## the new bar would be renamed `@PanelContainer@N` and lose the stable name probes and tests look up.
	if _bar != null and is_instance_valid(_bar):
		if _bar.get_parent() != null:
			_bar.get_parent().remove_child(_bar)
		_bar.queue_free()
	var vp = app.get("viewport")
	if vp == null:
		return
	var panel := PanelContainer.new()
	panel.name = "AssistRegionBar"
	panel.add_theme_stylebox_override("panel", DccWidgets.box("warn", "panel", 8, 5))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	## A single row where the viewport is wide enough (640 px: measured need of the six controls at desktop
	## density is about 600), else a wrapping flow with a stated width (a flow with no width wraps every
	## control onto its own line, seen in the first desktop capture).
	var avail: float = vp.size.x - 24.0
	var row: Container = HBoxContainer.new() if avail >= 640.0 else HFlowContainer.new()
	row.custom_minimum_size.x = minf(avail, 640.0)
	row.add_theme_constant_override("separation" if row is HBoxContainer else "h_separation", 8)
	if row is HFlowContainer:
		row.add_theme_constant_override("v_separation", 4)
	panel.add_child(row)
	var title := DccTheme.label("ASSISTANT REGION", "warn", DccTheme.FS_MICRO)
	row.add_child(title)
	var hint := DccTheme.label("Drag on the map to paint.", "text_secondary", DccTheme.FS_SMALL)
	row.add_child(hint)
	_status = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_status.name = "assist_status"
	row.add_child(_status)
	var sizes := OptionButton.new()
	sizes.name = "assist_brush"
	for s in BRUSH_SIZES:
		sizes.add_item("Brush %d cells" % int(s))
	sizes.select(brush_index())
	sizes.item_selected.connect(func(i: int): set_brush(float(BRUSH_SIZES[i])))
	row.add_child(sizes)
	var undo_b := DccWidgets.action(row, "Undo stroke", func(): undo_stroke(app))
	undo_b.name = "assist_undo_stroke"
	var clear_b := DccWidgets.action(row, "Clear", func(): clear(app))
	clear_b.name = "assist_clear"
	var done_b := DccWidgets.action(row, "Done", func(): end_paint(app), true)
	done_b.name = "assist_done"
	## Touch floors for the tablet/phone density (the shell's own sizing helper), both axes.
	for b in [sizes, undo_b, clear_b, done_b]:
		var c := b as Control
		c.custom_minimum_size.y = maxi(int(c.custom_minimum_size.y), DccTheme.PHONE_TAP_MIN if DccTheme.is_phone() else (DccTheme.role_px("btn_min_h") if DccTheme.is_tablet() else 26))
	vp.add_child(panel)
	panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP, Control.PRESET_MODE_MINSIZE, 8)
	panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_bar = panel


## The facts the model is given, as the dictionary Rust's `EditCatalogue::from_json` reads:
## `{world, region}` from the engine, or `{withheld}` with the reason (no region, a generation
## running, a build without the binding). Read-only. A region that the engine refuses (too large,
## outside the world) comes back as `{withheld: reason}` so the Assistant shows the reason itself.
static func facts(bridge) -> Dictionary:
	if is_empty():
		return {"withheld": "No region is selected. Paint an area or use a box first."}
	var d: Dictionary = bridge.region_describe(spec())
	if not bool(d.get("ok", false)):
		return {"withheld": String(d.get("reason", "The region could not be read."))}
	var region := {}
	for k in ["id", "shape", "bbox", "cells", "clipped", "rows", "row_step", "approximate", "ground", "area_km2", "bbox_km"]:
		if d.has(k):
			region[k] = d[k]
	return {"world": d["world"], "region": region}


## `facts()` as the JSON text `LlmStream` takes as `args.edit`.
static func facts_json(bridge) -> String:
	return JSON.stringify(facts(bridge))
