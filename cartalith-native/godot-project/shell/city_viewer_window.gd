extends AcceptDialog
class_name CityViewerWindow

## The City Viewer (`GUI_GAP_REGISTER.md` UM-02) — the reference's
## `cityViewerModal` with its `cvCanvas`, `cvLegend`, `cvInfoPanel` and
## `cvCloseBtn`, ported as far as this port's engine can fill it.
##
## **It shows a town plan, not a finished city, and it says so on screen.**
## `URBAN_MORPHOLOGY_SCOPE.md` has milestones 1-7 plus 12 of ~17 built: the
## site model, the market anchor, the arterial primaries, the organic street
## growth off them, and the blocks and parcels platted out of that graph.
## Buildings, districts, amenities and the wall circuit are milestones 10 and
## 13-17 and do not exist, so this window draws none of them and its own info
## panel names each one as missing rather than leaving a viewer to conclude
## the town simply has no buildings. That disclosure is read off the engine's
## own `stages` array, so it cannot drift from what actually ran — and the
## panel additionally names the two places where the *drawing* is ahead of
## the generator (a rooftop is a whole parcel; there is no open market
## square), which is the one thing `stages` cannot say for itself.
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

## Phone (§13). The canvas and its companion column cannot sit side by side at
## 393 dp -- the column alone is 264 of them -- so they stack, canvas over
## column, and the canvas takes a fixed band rather than the leftover height
## (which under a scrolling column is not a well-defined quantity).
var _phone := false
## Dp, and the design canvas's own figure: enough to read a street skeleton at
## the fit scale, and under 40% of the screen so the info column still opens on
## real content rather than on a caption.
const PHONE_CANVAS_H := 330


## `host` is the `DccApp` this window is parented to, needed for the phone
## treatment. Defaulted rather than added to the call site because `app.gd` is
## shared and this is the only fact this window wants from it -- the parent is
## already the right object, and `setup()` is called after `add_child()` at
## every one of its call sites.
func setup(b: EngineBridge, host = null) -> void:
	bridge = b
	title = "City viewer"
	size = Vector2i(940, 660)
	min_size = Vector2i(620, 440)
	## The reference's `cvCloseBtn`. `AcceptDialog` already supplies exactly
	## one OK button; renaming it is the whole control.
	ok_button_text = "Close"
	if host == null:
		host = get_parent()
	## Also turns `wrap_controls` off -- on with a canvas child that has no
	## natural size, which is the worst possible combination of the two.
	_phone = DccWidgets.phone_window(self, host)

	var outer: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	var left := VBoxContainer.new()
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if not _phone:
		left.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_theme_constant_override("separation", 4)
	outer.add_child(left)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	_title_label = DccTheme.label("—", "text", DccTheme.FS_BODY)
	_title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## The picker is the wide control on a phone, and the name is already in the
	## header above it -- so phone drops the label rather than letting two
	## variable-width strings fight over 393 dp.
	_title_label.visible = not _phone
	head.add_child(_title_label)
	_picker = OptionButton.new()
	if _phone:
		_picker.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_picker.clip_text = true
	else:
		_picker.custom_minimum_size.x = 200
	_picker.item_selected.connect(_on_pick)
	DccWidgets.style_popup(_picker.get_popup())
	head.add_child(_picker)
	var reset := Button.new()
	reset.text = "Fit"
	reset.focus_mode = Control.FOCUS_NONE
	reset.pressed.connect(_reset_view)
	head.add_child(reset)
	## A wheel is the only zoom the desktop canvas offers, and a phone has none.
	## Pinch is wired below and is the gesture people reach for first, but it is
	## also the one a single thumb cannot make -- so the two explicit steps are
	## here as well, at the canvas's own 44 dp icon-button size.
	if _phone:
		_zoom_button(head, "+", 1.0)
		_zoom_button(head, "−", -1.0)
	left.add_child(head)

	_canvas = Control.new()
	_canvas.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if _phone:
		_canvas.custom_minimum_size.y = PHONE_CANVAS_H
	else:
		_canvas.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_canvas.clip_contents = true
	_canvas.mouse_filter = Control.MOUSE_FILTER_STOP
	_canvas.draw.connect(_draw_canvas)
	_canvas.gui_input.connect(_on_canvas_input)
	left.add_child(_canvas)

	var side := VBoxContainer.new()
	if _phone:
		side.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
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

	if _phone:
		DccWidgets.phone_head(outer, "City viewer", "urban morphology")

	bridge.generation_finished.connect(func(_ok: bool): if visible: _reload())
	bridge.world_loaded.connect(func(): if visible: _reload())


## One 44 dp step of the wheel, for the pointer gesture a phone does not have.
## `dir` is +1 or -1; the zoom is anchored on the canvas centre, which is the
## only fixed point a button press has.
func _zoom_button(parent: Control, glyph: String, dir: float) -> Button:
	var b := Button.new()
	b.text = glyph
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(44, 44)
	b.pressed.connect(func(): _zoom_by(pow(ZOOM_STEP, dir), _canvas.size * 0.5))
	parent.add_child(b)
	return b


## The pan correction that keeps `anchor` fixed while the scale changes,
## factored out of the wheel handler so the pinch gesture and the two buttons
## use the identical relation rather than three copies of it.
func _zoom_by(factor: float, anchor: Vector2) -> void:
	var before := _zoom
	_zoom = clampf(_zoom * factor, ZOOM_MIN, ZOOM_MAX)
	if is_equal_approx(before, _zoom):
		return
	var centre := _canvas.size * 0.5
	var k := _zoom / before
	_pan = (_pan + centre - anchor) * k - centre + anchor
	_canvas.queue_redraw()


## Opens on one settlement, by its index in `bridge.settlements()`.
func open(index: int) -> void:
	_index = index
	_refill_picker()
	_reload()
	if not DccWidgets.phone_present(self, get_parent()):
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
## roads running to the box edge do not shrink the town to a speck.
##
## Milestone 12 is what finally makes that possible: the blocks *are* the built
## mass, near enough, so the fit is now the reference's own second choice
## rather than its third. The graph's extent stays as the fallback for a
## layout that produced no blocks at all — a hamlet whose faces all fell under
## the 120 m² floor, or a settlement whose growth never closed a loop.
func _fit_box() -> Rect2:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for blk: PackedVector2Array in _layout.get("blocks", []) as Array:
		for p in blk:
			lo = lo.min(p)
			hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		var streets: Dictionary = _layout.get("streets", {})
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
	## something rather than against the shell's own panel colour. It is a
	## muted parchment rather than the reference's green: this is a drawn plan
	## viewed inside a dark tool window, and the whole map palette is built
	## around it (`urban_layout_draw.gd`'s header).
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
			## Keep the point under the cursor fixed: the pan correction is
			## the same relation `ViewportHost._zoom_at()` uses.
			_zoom_by(ZOOM_STEP if mb.button_index == MOUSE_BUTTON_WHEEL_UP else 1.0 / ZOOM_STEP,
				mb.position)
			_canvas.accept_event()
		elif mb.button_index == MOUSE_BUTTON_LEFT or mb.button_index == MOUSE_BUTTON_MIDDLE:
			_dragging = mb.pressed
			_canvas.accept_event()
	elif event is InputEventMouseMotion and _dragging:
		_pan += (event as InputEventMouseMotion).relative
		_canvas.queue_redraw()
		_canvas.accept_event()
	elif event is InputEventMagnifyGesture:
		## Android's two-finger pinch, which reaches here because
		## `project.godot` turns `pointing/android/enable_pan_and_scale_gestures`
		## on. Anchored on the gesture's own centre, exactly as the wheel is
		## anchored on the cursor -- so the street under the fingers stays under
		## them. `accept_event()` matters more here than anywhere else in this
		## file: without it the pinch keeps travelling and the ScrollContainer
		## underneath takes the second half of the gesture (the canvas's own
		## "one primary gesture per region" rule).
		var mg := event as InputEventMagnifyGesture
		_zoom_by(mg.factor, mg.position)
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
	_swatch(leg, DRAW._roof_color(0.62),
		"Rooftops — one per parcel, each a slightly different weathered shade")
	_swatch(leg, DRAW.BLOCK_GROUND, "Block ground — the built interior between streets (buildBlocks)")
	## Only when the town has one: a site with no primary to widen gets no
	## plaza, and a swatch for a colour that is not on screen is a lie.
	if _layout.has("plaza"):
		_swatch(leg, DRAW.PLAZA_GROUND,
			"Market place — the block kept open, no lots platted (buildPlaza)")
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
	_field(gen, "Blocks", str((_layout.get("blocks", []) as Array).size()))
	_field(gen, "Parcels", str((_layout.get("parcels", []) as Array).size()))

	var stages := DccWidgets.section(_info, "What produced this")
	var ran := ""
	for s in _layout.get("stages", PackedStringArray()) as PackedStringArray:
		ran += "· " + s + "\n"
	DccWidgets.note(stages, ran.strip_edges())
	DccWidgets.note(stages,
		"Not generated, and so not drawn: buildings (milestone 13), districts "
		+ "and amenities (13-14), the wall circuit and its gates (10), the "
		+ "harbour and quay (9), bridges and fords (9), farmland and "
		+ "hinterland detail (15). URBAN_MORPHOLOGY_SCOPE.md has 8 of ~17 "
		+ "milestones built.")
	DccWidgets.note(stages,
		"One thing on screen is ahead of the generator, and it is drawing "
		+ "rather than data. A rooftop is a whole parcel, inset — "
		+ "buildBuildings (13) would put a smaller footprint inside each lot, "
		+ "with a grammar per district and a terrain gate that leaves some "
		+ "lots empty, so this town has no gaps and every roof is the same "
		+ "simple shape. The rooftop shading is real per-parcel engine output, "
		+ "not a drawing effect.")
	if _layout.has("plaza"):
		DccWidgets.note(stages,
			"The lighter, outlined square at the centre is the market place — "
			+ "buildPlaza (milestone 8) widening the principal street away "
			+ "from the water. It is real generated geometry: the engine "
			+ "flags that block and plats no lots on it, which is why it is "
			+ "the one piece of open ground in the town.")

	## Legend and info are both rebuilt here, so this is where the touch fit
	## belongs -- see `place_editor_window.gd`'s own call for the reasoning.
	if _phone:
		var host := get_parent()
		if host != null and host.has_method("phone_fit"):
			host.phone_fit(self, 1.0)


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
