extends AcceptDialog
class_name CityViewerWindow

## The City Viewer (`GUI_GAP_REGISTER.md` UM-02) — the reference's
## `cityViewerModal` with its `cvCanvas`, `cvLegend`, `cvInfoPanel` and
## `cvCloseBtn`, ported as far as this port's engine can fill it.
##
## **It shows a whole town.** `run_layout` became a caller of the reference's
## own `generate()` on 2026-09-02 — all 29 stages, in its own order — so this
## window draws the site, the streets by class, the blocks and the lots, the
## districts tinted on those lots, the buildings inside them, the wall circuit
## and its gates, the market squares and the farmland outside.
##
## The disclosure discipline is unchanged and now says less: the info panel is
## still read off the engine's own `stages` array, so it cannot drift from what
## actually ran, and it still names what is generated but not drawn (the
## crossings, the civic and religious buildings, the hinterland clutter). What
## it no longer has to say is that the *drawing* is ahead of the generator — the
## rooftop-is-a-whole-parcel stand-in is gone.
##
## The canvas's wheel-zoom and drag-pan are the reference's (`_cvZoomAt` and
## its pointer-drag handler); the initial fit is `_umDrawLayoutPreview`'s
## fit-to-*built-mass* box, which is now literally that — the wall ring first
## and the blocks second, which are the reference's own first two choices, with
## the street graph's extent still there as the third for a town that produced
## neither.
##
## ## Nothing draws this window; here is what it derives from instead
##
## `design/dcc-environment-2026-08-31/README.md`, verbatim: the two DCC files
## specify the shell frame and its information architecture and *"do not specify
## the dedicated windows: Faction roster, Place editor, City viewer, Data
## manager, Vault, Travel library, Asset library."* Each choice below therefore
## names the canvas vocabulary it came from:
##
## - **The legend row** — `05-right-dock-and-bars.md` §1.8, the paint legend,
##   which is the *only* legend the canvases draw. See `_swatch`.
## - **The key/value read row** — `06-phone.md` §6.7's Inspector drawer and
##   §6.6's `read` row type. See `_field`.
## - **`§`-headed noun-phrase section titles** — `DccTheme.header()`'s `§ TITLE`,
##   the L3 sigil the DCC Environment markup uses (`§ TOOLS`, `§ LAYERS`,
##   `§ RESULTS`, `§ BRUSH · GLOBAL`), and §6.6's `head` rows (`STATUS`,
##   `RECENT WORLDS`, `STORAGE LOCATIONS`). Every one of those is a noun
##   phrase, which is why `§ WHAT PRODUCED THIS` became `§ STAGES` on
##   2026-09-05: a question is not a label in this vocabulary. The words it
##   carried are in the section's own notes, unchanged.
## - **The phone header's title/subtitle voice** — §6.6's `_moreTitle()` table.
##
## **Open, and deliberately not resolved here.** On a phone, is this a window or
## a `moreStack` sub-screen? §6.6's `_moreTitle()` answers that for **three** of
## those seven windows and no more: the Data manager, the Travel library and the
## Asset library each get a sub-screen (`data`, `travel`, `assets`, plus their
## own leaves), reached by a `nav` row on the `root` screen. There is no row for
## the Vault, and none for the city viewer, the place editor or the faction
## roster — and §6.6's own `civ` screen, which is where an urban surface would
## sit, lists the three place tools, Landmark generation and the Journey Planner
## and nothing else. The stack exists and these three are not in it. That
## silence is not authority to rebuild them as sheets, so the window shape is
## unchanged.

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

## The info panel's key column. Deliberately the same number as
## `right_dock.gd`'s `_FIELD_LABEL_W`, because that dock's Settlement context is
## this window's sibling surface -- the two draw the same settlement, and a
## reader moving between them should not see the value column jump.
const FIELD_KEY_W := 116

## Phone (§13). The canvas and its companion column cannot sit side by side at
## 393 dp -- the column alone is 264 of them -- so they stack, canvas over
## column, and the canvas takes a fixed band rather than the leftover height
## (which under a scrolling column is not a well-defined quantity).
var _phone := false
## Dp. Enough to read a street skeleton at the fit scale, and under 40% of the
## screen (330 / 852 = 38.7%) so the column below it still opens on real content
## rather than on a caption.
##
## **Corrected 2026-09-05: no canvas carries this number.** It used to read "and
## the design canvas's own figure". `design/` was grepped whole for `330`; every
## hit is one of base64 font data, an SVG path coordinate in
## `landmark-generation/Phone.dc.html`, a 330 px desktop submenu/annotation width
## in `landmark-generation/Submenu.dc.html`, or the `--ldW` left-dock width on
## the LAPTOP 1366 frame (`01-frame-and-tokens.md`, `04-left-dock.md`,
## `05-right-dock-and-bars.md`). None of them is a phone map band. The
## proportion argument above is the whole justification and always was --
## `asset_library_window.gd`'s `PHONE_SHEET_PREVIEW` cites exactly that half of
## this comment and is unaffected.
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
		## Subtitle in §6.6 `_moreTitle()`'s voice -- lower case, `·`-separated,
		## naming what the screen holds. It read "urban morphology", which is the
		## subsystem's name rather than the screen's contents; the canvas's own
		## subtitles are contents lists (`timeline · layers`).
		DccWidgets.phone_head(outer, "City viewer", "plan · legend · stages")

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
## All three of its choices are available now and this is its order exactly
## (reference lines 22909-22911, repeated verbatim at 23003-23005 for the
## interactive viewer's own camera): the wall ring, else the building
## footprints, else the graph. The street-graph fallback below is the third; the
## site box under it is the reference's `if(!(maxx>minx))` guard.
##
## **The wall crops extramural buildings, and that is the reference's own
## trade-off, not an oversight.** A riverside suburb outside the circuit runs
## off the frame at the opening fit, exactly as the approach roads do — the
## comment at line 22906 accepts it in those words. The canvas pans and zooms,
## so nothing is unreachable.
func _fit_box() -> Rect2:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	var ring: PackedVector2Array = _layout.get("wall_ring", PackedVector2Array())
	for p in ring:
		lo = lo.min(p)
		hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		for b: PackedVector2Array in _layout.get("buildings", []) as Array:
			for p in b:
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
	_swatch(leg, DRAW._roof_color(0.62), "Rooftops",
		"One per building footprint, each a slightly different weathered shade (buildBuildings).")
	_swatch(leg, DRAW.BLOCK_GROUND, "Block ground",
		"The built interior between streets (buildBlocks).")
	## Only when the town has one: a site with no primary to widen gets no
	## plaza, and a swatch for a colour that is not on screen is a lie.
	if _layout.has("plaza") or not (_layout.get("markets", []) as Array).is_empty():
		_swatch(leg, DRAW.PLAZA_GROUND, "Open squares",
			"The market place and the specialised markets, swept clear of lots.")
	## The wall is the ladder's verdict, so its swatch appears only when this
	## settlement was actually walled -- and in the colour its own style got.
	if _layout.has("wall_ring"):
		var wstyle := String(_layout.get("wall_style", "curtain"))
		var wcol: Color = DRAW.WALL_PALISADE if wstyle == "palisade" \
			else (DRAW.WALL_DITCH if wstyle == "ditch" else DRAW.WALL_STONE)
		_swatch(leg, wcol, "The circuit",
			"A %s wall with its gates (buildWall)." % wstyle)
	_swatch(leg, DRAW.FILL_PRIMARY, "Primary streets",
		"The arterial backbone (buildPrimaries).")
	_swatch(leg, DRAW.FILL_OTHER, "Streets and lanes", "Organic growth (grow).")
	_swatch(leg, DRAW.DISTRICT_FILL["market"], "District tints",
		"On the lots: market, burgher, artisan, riverside craft, harbour, suburb, "
		+ "agrarian (assignDistricts). The swatch is the market tint; each district "
		+ "draws in its own.")
	_swatch(leg, DRAW.FARM_FIELD, "Farmland",
		"The strip or ring fields outside the town.")
	_swatch(leg, DRAW.WATER, "Water", "The site's own water — the map's river or coast.")
	_swatch(leg, DRAW.MARKET, "Market anchor",
		"The point the town organises around.")
	_swatch(leg, DRAW.ROUTE_END, "Approach roads",
		"Where the roads running to the town meet the frame.")

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
	## The head count `generate()` derives, 5.2 per built non-churchyard lot --
	## what the town actually houses, against what was asked for.
	_field(gen, "Housed", "%d in %d built lots"
		% [int(_layout.get("pop", 0)), int(round(float(_layout.get("pop", 0)) / 5.2))])
	_field(gen, "Inferred age", "%d years" % int(_layout.get("settlement_age_years", 0)))
	## `computeMetrics.totalLen` on the final graph, not `grow`'s own return --
	## the live network after the lane passes, `removeWaterCrossings`,
	## `privatizeAlleys` and `clearFortZone` have each removed edges from it.
	_field(gen, "Street network", "%.0f m live, %.0f m target"
		% [float(_layout.get("street_len_m", 0)), float(_layout.get("target_len_m", 0))])
	_field(gen, "Urban radius", "%.0f m" % float(_layout.get("max_rf_m", 0)))
	_field(gen, "Street segments", str(int(_layout.get("edge_count", 0))))
	_field(gen, "Blocks", str((_layout.get("blocks", []) as Array).size()))
	_field(gen, "Parcels", str((_layout.get("parcels", []) as Array).size()))
	_field(gen, "Buildings", str((_layout.get("buildings", []) as Array).size()))
	_field(gen, "Wall", "%s circuit, %d gates"
		% [String(_layout.get("wall_style", "—")),
			(_layout.get("wall_gates", PackedVector2Array()) as PackedVector2Array).size()] \
		if _layout.has("wall_ring") \
		else "none — %s on the wall ladder" % String(_layout.get("wall_spec", "?")))
	_field(gen, "Markets", str((_layout.get("markets", []) as Array).size()))
	_field(gen, "Farm plots", str((_layout.get("farmland", []) as Array).size()))

	## `§ STAGES`, not `§ WHAT PRODUCED THIS` (2026-09-05). Every section title
	## the canvases carry is a noun phrase -- `§ TOOLS`, `§ LAYERS`,
	## `§ RESULTS`, `§ BRUSH · GLOBAL` in the DCC Environment markup; `STATUS`,
	## `RECENT WORLDS`, `AUTOSAVE`, `STORAGE LOCATIONS` in `06-phone.md` §6.6's
	## `head` rows -- and none is a question. The section reads the engine's own
	## `stages` array, so `Stages` is also what it literally is. No other call
	## site used the old string (grepped repo-wide before the change).
	var stages := DccWidgets.section(_info, "Stages")
	var ran := ""
	for s in _layout.get("stages", PackedStringArray()) as PackedStringArray:
		ran += "· " + s + "\n"
	DccWidgets.note(stages, ran.strip_edges())
	## **Corrected 2026-09-05.** This read "Every line above is the reference's
	## own generate() — all 29 stages", against a list a reader can count: it is
	## ten lines (`_entwin_probe.gd` CV6 measures the array), because
	## `urban_bridge.rs` builds `stages` as ten entries whose own first line
	## already carries the 29. The run is 29; the list is the ten whose output
	## this window draws, which is what that file's own comment says it is.
	DccWidgets.note(stages,
		"Ten lines above, and twenty-nine stages behind them. run_layout calls "
		+ "the reference's own generate() — all 29, in its own order, "
		+ "golden-verified whole against its own hashModel; these ten are the "
		+ "stages whose output is visible in this window, each carrying what it "
		+ "actually produced. The list is built in urban_bridge.rs beside the "
		+ "code that ran, so it cannot drift from it. It replaced a hand-ordered "
		+ "subset on 2026-09-02, which is where the buildings, the wall, the "
		+ "districts, the markets and the farmland came from all at once.")
	DccWidgets.note(stages,
		"Three of the model's own layers are generated and not drawn: the "
		+ "justified crossings (a stone deck where a road really crosses the "
		+ "river, a stippled ford band where a through-town has none), the "
		+ "civic hall and places of worship, and the hinterland clutter — "
		+ "trees, fences, drying racks, log booms. They are on the engine's "
		+ "Town and are one bridge field each away.")
	DccWidgets.note(stages,
		"Nothing on screen is ahead of the generator any more. A rooftop was a "
		+ "whole parcel until 2026-09-02; it is now buildBuildings' own "
		+ "footprint, with the grammar its district calls for and the ridge "
		+ "line the engine laid, and a lot with no roof on it is a lot the "
		+ "engine left empty.")
	if not _layout.has("wall_ring"):
		DccWidgets.note(stages,
			"This settlement has no wall, and that is _umWallSpec's verdict on "
			+ "its tier, function, threat, wealth, age and command of ground — "
			+ "not a missing stage. A town of 1,200 or 260 years gets stone; a "
			+ "hamlet on flat ground gets nothing.")

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


## One legend row, against `05-right-dock-and-bars.md` §1.8's — the paint
## legend, which is the only legend the canvases draw:
##
## | canvas (`rdPaint` legend row) | here |
## |---|---|
## | `min-height:var(--row)` (28 px pointer, 44 px touch) | `role_px("row_min_h")` on tablet, 28 otherwise |
## | `gap:9px` | separation 9 |
## | swatch `10×10px; border-radius:3px` | a `Panel` at `DccTheme.flat(color, 3)` — a `ColorRect` cannot round a corner |
## | value name, `var(--body)` | `DccTheme.label(name, "text")` |
## | `padding:2px 4px` | the section's own body padding (`DccWidgets.section`) |
##
## **The name is a name now, and the sentence is the tooltip.** Every one of
## these rows used to be a whole clause of micro-text in `text_dim` — "Rooftops
## — one per building footprint, each a slightly different weathered shade" — in
## a 264 px column. The canvas's legend value is `temperate forest`: a name, at
## body ink, on one row. That is the part of this legend nobody had drawn, and
## deriving it is what the 2026-09-05 ruling asks for. Nothing was deleted; each
## sentence moved to the row's `tooltip_text`.
##
## **The canvas's fourth cell has no honest content here, so there is none.**
## Its row ends `spacer` + a per-value painted count at `var(--m2)`/`var(--dim)`.
## This legend's equivalent counts — blocks, parcels, buildings, markets, farm
## plots — are already rows of the `§ GENERATION` section a few lines below in
## the same column, and several swatches (Water, Market anchor, Approach roads)
## have no count at all. A column that is empty on a third of its rows and a
## duplicate on the rest is worse than no column: `MISTAKES`' rule is to omit
## the field rather than fill it with something plausible.
##
## **Not derived, because nothing settles it:** the canvas's legend rows are
## *interactive* (click arms that paint value, and the armed row takes
## `var(--wash)` behind it and `var(--acc)` ink). These rows arm nothing —
## there is no per-layer visibility toggle on this canvas to arm — so the
## armed/unarmed pair is left unimplemented rather than faked as a hover.
func _swatch(parent: Control, color: Color, name: String, tip: String = "") -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 9)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if DccTheme.is_tablet() else 28
	row.tooltip_text = tip
	## `Panel` over `ColorRect` for the 3 px radius the canvas specifies.
	## `mouse_filter` stays default so the row's own tooltip is what surfaces,
	## rather than the swatch swallowing the hover with no tooltip of its own.
	var sw := Panel.new()
	sw.add_theme_stylebox_override("panel", DccTheme.flat(color, 3))
	sw.custom_minimum_size = Vector2(10, 10)
	sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	sw.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(sw)
	var lb := DccTheme.label(name, "text", DccTheme.FS_SMALL)
	lb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	lb.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	lb.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(lb)
	parent.add_child(row)


## One key/value readout, against the two read rows the canvases do specify:
## `06-phone.md` §6.7's Inspector drawer (`Key 9.5px mono, .14em, var(--dim)`;
## `value 11px mono, var(--ink), right`) and §6.6's `read` row type
## (`min-height:42px`; key `9.5px mono, .1em, var(--dim)`; value `10.5px mono`,
## right-aligned).
##
## Derived from them, and each is a change:
##
## - **The value is mono.** It was proportional. Both canvas read rows set every
##   value in mono, text ones included — §6.7 puts `lithology`, `biome` and
##   `Kess basin` there beside the numbers — and this panel is all readouts:
##   nothing in it is editable.
## - **The row has a height.** It had none, so a one-line row was as tall as its
##   font. `--row` is 28 px on a pointer and 44 on touch
##   (`01-frame-and-tokens.md`); `role_px("row_min_h")` is the shell's own name
##   for the second.
## - **The key column stays 116 px**, which is what `right_dock.gd`'s own
##   `_FIELD_LABEL_W` already is — the sibling surface this window's Settlement
##   block sits beside. It was already that number; stating it so a future
##   change moves both.
##
## **Not derived, because nothing settles it: what a read row does with a value
## too long for its column.** Every value in both canvas tables is short
## (`{slope}° · {aspect}°`, a biome name, a path), so their `right` alignment
## never has to wrap. Several values here are sentences — `no — synthetic
## hills`, `none — palisade on the wall ladder` — in a 264 px column against a
## 116 px key. Right-aligning them would wrap right-aligned prose, and
## ellipsising them would drop the half that carries the reason. So the
## alignment and the autowrap are **left exactly as they were**, and this
## paragraph is the record that the canvas's `right` was read and not applied.
func _field(parent: Control, key: String, value: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if DccTheme.is_tablet() else 28
	var k := DccTheme.label(key, "text_dim", DccTheme.FS_SMALL)
	k.custom_minimum_size.x = FIELD_KEY_W
	k.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(k)
	var v := DccTheme.mono_label(value, "text", DccTheme.FS_SMALL)
	v.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	v.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(v)
	parent.add_child(row)
