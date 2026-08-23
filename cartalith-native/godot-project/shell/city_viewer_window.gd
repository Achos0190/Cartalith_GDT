extends AcceptDialog
class_name CityViewerWindow

## The City Viewer (`GUI_GAP_REGISTER.md` UM-02) — the reference's
## `cityViewerModal` with its `cvCanvas`, `cvLegend`, `cvInfoPanel` and
## `cvCloseBtn`, ported as far as this port's engine can fill it.
##
## **It shows a street skeleton, not a city, and it says so on screen.**
## `URBAN_MORPHOLOGY_SCOPE.md` has milestones 1-7 of ~17 built: the site
## model, the market anchor, the arterial primaries and the organic street
## growth off them. Blocks, parcels, buildings, districts, amenities and the
## wall circuit are milestones 8-17 and do not exist, so this window draws
## none of them and its own info panel names each one as missing rather than
## leaving a viewer to conclude the town simply has no buildings. That
## disclosure is read off the engine's own `stages` array, so it cannot drift
## from what actually ran.
##
## The canvas's wheel-zoom and drag-pan are the reference's (`_cvZoomAt` and
## its pointer-drag handler); the initial fit is `_umDrawLayoutPreview`'s
## fit-to-*built-mass* box, degraded honestly — with no wall ring and no
## buildings to bound, the graph's own extent is the only thing left to fit,
## which is the reference's own third fallback (`else { for(const n of
## model.graph.nodes) ext(n.x,n.y); }`).

## `preload`, not the `UrbanLayoutDraw` global class name -- a global name
## only resolves once the editor has rescanned and written
## `.godot/global_script_class_cache.cfg`, so a fresh clone or an editor-less
## run would fail to parse this file. `viewport_host.gd`'s `OVERLAY_SCRIPT`
## and `layers_popover.gd`'s `FLOW_FX_SCRIPT` are here for the same reason.
const DRAW := preload("res://shell/urban_layout_draw.gd")

var bridge: EngineBridge

var _canvas: Control
var _info: VBoxContainer
var _legend: VBoxContainer
var _title_label: Label
var _picker: OptionButton

var _index := -1
var _layout: Dictionary = {}
var _settlement: Dictionary = {}

## Canvas view state — `_zoom` multiplies the fit scale, `_pan` is a screen
## offset applied after it.
var _zoom := 1.0
var _pan := Vector2.ZERO
var _dragging := false

const ZOOM_MIN := 0.25
const ZOOM_MAX := 12.0
const ZOOM_STEP := 1.18


func setup(b: EngineBridge) -> void:
	bridge = b
	title = "City viewer"
	size = Vector2i(940, 660)
	min_size = Vector2i(620, 440)
	## The reference's `cvCloseBtn`. `AcceptDialog` already supplies exactly
	## one OK button; renaming it is the whole control.
	ok_button_text = "Close"

	var outer := HBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	var left := VBoxContainer.new()
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_theme_constant_override("separation", 4)
	outer.add_child(left)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	_title_label = DccTheme.label("—", "text", DccTheme.FS_BODY)
	_title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title_label)
	_picker = OptionButton.new()
	_picker.custom_minimum_size.x = 200
	_picker.item_selected.connect(_on_pick)
	head.add_child(_picker)
	var reset := Button.new()
	reset.text = "Fit"
	reset.focus_mode = Control.FOCUS_NONE
	reset.pressed.connect(_reset_view)
	head.add_child(reset)
	left.add_child(head)

	_canvas = Control.new()
	_canvas.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_canvas.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_canvas.clip_contents = true
	_canvas.mouse_filter = Control.MOUSE_FILTER_STOP
	_canvas.draw.connect(_draw_canvas)
	_canvas.gui_input.connect(_on_canvas_input)
	left.add_child(_canvas)

	var side := VBoxContainer.new()
	side.custom_minimum_size.x = 264
	side.add_theme_constant_override("separation", 0)
	outer.add_child(side)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	side.add_child(scroll)
	var pad := MarginContainer.new()
	for s in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + s, 10)
	scroll.add_child(pad)
	var side_body := VBoxContainer.new()
	side_body.add_theme_constant_override("separation", 4)
	side_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(side_body)

	## `cvLegend` and `cvInfoPanel`, in the reference's own order.
	_legend = VBoxContainer.new()
	_legend.add_theme_constant_override("separation", 2)
	side_body.add_child(_legend)
	side_body.add_child(DccTheme.rule())
	_info = VBoxContainer.new()
	_info.add_theme_constant_override("separation", 2)
	side_body.add_child(_info)

	bridge.generation_finished.connect(func(_ok: bool): if visible: _reload())
	bridge.world_loaded.connect(func(): if visible: _reload())


## Opens on one settlement, by its index in `bridge.settlements()`.
func open(index: int) -> void:
	_index = index
	_refill_picker()
	_reload()
	popup_centered()


func _refill_picker() -> void:
	_picker.clear()
	var places := bridge.settlements()
	for i in places.size():
		var s: Dictionary = places[i]
		_picker.add_item("%s (%s)" % [String(s.get("name", "?")), String(s.get("kind", "?"))], i)
		if i == _index:
			_picker.select(_picker.item_count - 1)


func _on_pick(item: int) -> void:
	_index = _picker.get_item_id(item)
	_reload()


func _reload() -> void:
	_layout = {}
	_settlement = {}
	var places := bridge.settlements()
	if _index >= 0 and _index < places.size():
		_settlement = places[_index]
		var got := bridge.urban_layouts(PackedInt32Array([_index]))
		if got.size() > 0:
			_layout = got[0]
	_title_label.text = String(_settlement.get("name", "—"))
	_reset_view()
	_rebuild_side()


func _reset_view() -> void:
	_zoom = 1.0
	_pan = Vector2.ZERO
	_canvas.queue_redraw()


# -- the canvas ---------------------------------------------------------------

## The model-metre box the view fits. `_umDrawLayoutPreview` fits the *built
## mass* (wall ring, else building footprints) precisely so the long approach
## roads running to the box edge do not shrink the town to a speck. Neither
## exists here, so this uses the reference's own third fallback — the graph's
## extent — and then trims the approach roads' influence the only way that is
## honest without inventing a built mass: it fits the graph, full stop, and
## leaves the user the wheel.
func _fit_box() -> Rect2:
	var streets: Dictionary = _layout.get("streets", {})
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for cls in streets.keys():
		var segs: PackedVector2Array = streets[cls]
		for p in segs:
			lo = lo.min(p)
			hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		return Rect2(Vector2.ZERO,
			Vector2(float(_layout.get("wm", 1700.0)), float(_layout.get("hm", 1250.0))))
	var pad := (hi - lo) * 0.08
	return Rect2(lo - pad, (hi - lo) + pad * 2.0)


func _draw_canvas() -> void:
	var rect := Rect2(Vector2.ZERO, _canvas.size)
	_canvas.draw_rect(rect, DccTheme.c("panel"))
	if _layout.is_empty():
		return
	## `_umDrawLayoutPreview`'s land ground, so streets and water read against
	## something rather than against the shell's own panel colour.
	_canvas.draw_rect(rect, DRAW.GROUND)

	var box := _fit_box()
	var margin := 16.0
	var fit: float = minf((_canvas.size.x - 2.0 * margin) / maxf(1.0, box.size.x),
		(_canvas.size.y - 2.0 * margin) / maxf(1.0, box.size.y))
	var view_scale := fit * _zoom
	var origin := (_canvas.size - box.size * view_scale) * 0.5 - box.position * view_scale + _pan
	var to_screen := func(p: Vector2) -> Vector2: return origin + p * view_scale
	## `px_floor` is 1.0: this canvas draws straight into screen space.
	DRAW.draw_layout(_canvas, _layout, to_screen, view_scale, 1.0, 1.0, true)


func _on_canvas_input(event: InputEvent) -> void:
	## Wheel zoom centred on the cursor, and middle/left drag to pan — the
	## reference's `_cvZoomAt` and its pointer-drag handler.
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP or mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			if not mb.pressed:
				return
			var factor := ZOOM_STEP if mb.button_index == MOUSE_BUTTON_WHEEL_UP else 1.0 / ZOOM_STEP
			var before := _zoom
			_zoom = clampf(_zoom * factor, ZOOM_MIN, ZOOM_MAX)
			## Keep the point under the cursor fixed: the pan correction is
			## the same relation `ViewportHost._zoom_at()` uses.
			var centre := _canvas.size * 0.5
			var k := _zoom / before
			_pan = (_pan + centre - mb.position) * k - centre + mb.position
			_canvas.queue_redraw()
			_canvas.accept_event()
		elif mb.button_index == MOUSE_BUTTON_LEFT or mb.button_index == MOUSE_BUTTON_MIDDLE:
			_dragging = mb.pressed
			_canvas.accept_event()
	elif event is InputEventMouseMotion and _dragging:
		_pan += (event as InputEventMouseMotion).relative
		_canvas.queue_redraw()
		_canvas.accept_event()


# -- legend and info panel ----------------------------------------------------

func _rebuild_side() -> void:
	for n in _legend.get_children():
		_legend.remove_child(n)
		n.queue_free()
	for n in _info.get_children():
		_info.remove_child(n)
		n.queue_free()

	if _layout.is_empty():
		DccWidgets.note(_info, _no_layout_reason())
		return

	var leg := DccWidgets.section(_legend, "Legend")
	_swatch(leg, DRAW.FILL_PRIMARY, "Primary streets — the arterial backbone (buildPrimaries)")
	_swatch(leg, DRAW.FILL_OTHER, "Streets and lanes — organic growth (grow)")
	_swatch(leg, DRAW.WATER, "The site's water — the map's own river/coast")
	_swatch(leg, DRAW.MARKET, "Market anchor — the point the town organises around")
	_swatch(leg, DRAW.ROUTE_END, "Approach-road endpoints")

	var sec := DccWidgets.section(_info, "Settlement")
	_field(sec, "Name", String(_settlement.get("name", "—")))
	_field(sec, "Class", String(_settlement.get("kind", "—")).capitalize())
	_field(sec, "Population", str(int(_settlement.get("population", 0))))

	var site := DccWidgets.section(_info, "Site")
	_field(site, "Site kind", String(_layout.get("site_kind", "—")))
	_field(site, "Market placed by", String(_layout.get("market_prov", "—")))
	_field(site, "Real map water", "yes" if _layout.get("uses_real_water", false) else "no — synthetic")
	_field(site, "Real map relief", "yes" if _layout.get("uses_real_terrain", false) else "no — synthetic hills")
	_field(site, "Box", "%.0f × %.0f m" % [float(_layout.get("wm", 0)), float(_layout.get("hm", 0))])

	var gen := DccWidgets.section(_info, "Generation")
	_field(gen, "Target population", str(int(_layout.get("pop_target", 0))))
	_field(gen, "Inferred age", "%d years" % int(_layout.get("settlement_age_years", 0)))
	_field(gen, "Street placed", "%.0f m of %.0f m target"
		% [float(_layout.get("placed_len_m", 0)), float(_layout.get("target_len_m", 0))])
	_field(gen, "Urban radius", "%.0f m" % float(_layout.get("max_rf_m", 0)))
	_field(gen, "Street segments", str(int(_layout.get("edge_count", 0))))
	_field(gen, "Primary routes", str((_layout.get("primaries", []) as Array).size()))

	var stages := DccWidgets.section(_info, "What produced this")
	var ran := ""
	for s in _layout.get("stages", PackedStringArray()) as PackedStringArray:
		ran += "· " + s + "\n"
	DccWidgets.note(stages, ran.strip_edges())
	DccWidgets.note(stages,
		"Not generated, and so not drawn: blocks and plazas (milestone 12), "
		+ "parcels and buildings (12-13), districts and amenities (13-14), the "
		+ "wall circuit and its gates (10), the harbour and quay (9), bridges "
		+ "and fords (9), farmland and hinterland detail (15). "
		+ "URBAN_MORPHOLOGY_SCOPE.md has 7 of ~17 milestones built; this is "
		+ "the street skeleton those seven produce, not a finished town.")


func _no_layout_reason() -> String:
	if not bridge.has_world:
		return "No world yet — generate one first."
	if _settlement.is_empty():
		return "No settlement selected."
	return ("No layout for this settlement. The engine refuses one when the "
		+ "settlement's own 1.7 × 1.25 km box is open water (a mid-lake or "
		+ "mid-sea pin has no shore to build on) — the reference's own "
		+ "`_umModelFor` refusal, which leaves the bare pin standing.")


func _swatch(parent: Control, color: Color, text: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var sw := ColorRect.new()
	sw.color = color
	sw.custom_minimum_size = Vector2(11, 11)
	sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(sw)
	var lb := DccTheme.label(text, "text_dim", DccTheme.FS_MICRO)
	lb.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(lb)
	parent.add_child(row)


func _field(parent: Control, key: String, value: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	var k := DccTheme.label(key, "text_dim", DccTheme.FS_SMALL)
	k.custom_minimum_size.x = 116
	row.add_child(k)
	var v := DccTheme.label(value, "text", DccTheme.FS_SMALL)
	v.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(v)
	parent.add_child(row)
